use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use crunch_gc_core::GcEntry;
use crunch_gc_core::GcExecutionMode;
use crunch_gc_core::GcMutationDisposition;
use crunch_gc_core::GcOwnership;
use crunch_gc_core::GcPlanError;
use crunch_gc_core::GcPlanRequest;
use crunch_gc_core::GcReportDecision;
use crunch_gc_core::plan_gc;
use crunch_gc_core::report_decision;
use crunch_gc_core::retention::RetentionDecision;
use crunch_gc_core::retention::RetentionDisposition;
use crunch_gc_core::retention::RetentionPlan;
use crunch_gc_core::retention::UsageObjectObservation;
use crunch_gc_core::retention::aggregate_usage;
use crunch_gc_core::retention::plan_retention;
use crunch_gc_core::summarize_reclaim_observations;
use data_encoding::HEXLOWER;
use futures::StreamExt;
use nix_compat::store_path::StorePath;
use serde::Serialize;
use snix_castore::B3Digest;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::CaMappings;
use crate::Error;
use crate::action_result::local_action_result_gc_candidates;

/// Services and paths needed for GC operations.
pub struct GcContext<'a> {
    pub state_dir: &'a Path,
    pub output_dir_str: &'a str,
    pub store_dir: &'a str,
    pub overlay_pathinfo: &'a dyn PathInfoService,
    pub composed_pathinfo: &'a dyn PathInfoService,
    pub overlay_directory_service: &'a dyn DirectoryService,
    pub overlay_blob_service: &'a dyn BlobService,
    pub overlay_plan_identity: Option<[u8; blake3::OUT_LEN]>,
    pub retained_castore_roots: &'a [Node],
}
use crate::artifact_attestation_file_path;
use crate::roots;
use crate::roots::GcRootRecord;

const GC_REPORT_SCHEMA: &str = "mantle-store-gc-report-v4";
const GC_EXECUTION_PLAN_DOMAIN: &[u8] = b"mantle.gc.execution-plan.v3";
const MAX_GC_BYTES_WALK_ENTRIES: u32 = 100_000;
const MAX_GC_FILE_SCAN_ENTRIES: u32 = 200_000;
const MAX_GC_RETAINED_CASTORE_ROOTS: usize = 1_000_000;
const INITIAL_PATHINFO_CAPACITY: usize = 256;
#[cfg(unix)]
const OWNER_WRITE_PERMISSION_MODE: u32 = 0o200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GcOperationKind {
    ExportedOutputs,
    PathInfoRewrite,
    ArtifactAttestations,
    ClosureAttestations,
    DirectoryRewrite,
    BlobIndexFiles,
    BlobChunkFiles,
    ActionResults,
    CaMappings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcRetentionExplanation {
    pub path: String,
    pub root_class: String,
    pub owner_scope: String,
    pub policy_blake3: String,
    pub project_identity: Option<String>,
    pub selector: Option<String>,
    pub generation: Option<u64>,
    pub lease_id: Option<String>,
    pub transition_id: String,
    pub transition_reason: String,
    pub disposition: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcPathExplanation {
    pub path: String,
    pub reason: String,
    pub retaining_roots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcBaseReachability {
    pub path: String,
    pub layer_index: usize,
    pub retaining_roots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcRootUsage {
    pub root: String,
    pub inclusive_bytes: u64,
    pub unique_bytes: u64,
    pub unknown_object_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcUsageReport {
    pub observed_bytes: u64,
    pub retained_bytes: u64,
    pub reclaimable_bytes: u64,
    pub quarantined_bytes: u64,
    pub unclassified_bytes: u64,
    pub shared_bytes: u64,
    pub unknown_object_count: usize,
    pub roots: Vec<GcRootUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcReclaimObservation {
    pub category: String,
    pub path: String,
    pub path_kind: Option<String>,
    pub bytes: Option<u64>,
    pub blocker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GcReport {
    pub schema: String,
    pub plan_id: String,
    pub retention_plan_id: String,
    pub overlay_plan_blake3: Option<String>,
    pub is_dry_run: bool,
    pub execution_complete: bool,
    pub failed_operation: Option<String>,
    pub failed_operations: Vec<String>,
    pub retained_root_count: u32,
    pub retained_castore_root_count: u32,
    pub candidate_path_count: u32,
    pub reclaimable_bytes_total: u64,
    pub reclaim_observations: Vec<GcReclaimObservation>,
    pub candidate_paths: Vec<String>,
    pub retention_explanations: Vec<GcRetentionExplanation>,
    pub path_explanations: Vec<GcPathExplanation>,
    pub base_reachability: Vec<GcBaseReachability>,
    pub usage: GcUsageReport,
    pub candidate_blob_index_count: u32,
    pub candidate_blob_chunk_count: u32,
    pub candidate_artifact_attestation_count: u32,
    pub candidate_closure_attestation_count: u32,
    pub candidate_exported_output_count: u32,
    pub candidate_action_result_record_count: u32,
    pub candidate_action_result_index_count: u32,
    pub operations: Vec<GcOperationKind>,
}

struct ComposedGcSnapshot {
    pathinfos: Vec<PathInfo>,
    ownership: BTreeMap<String, GcOwnership>,
}

#[derive(Default)]
struct LiveCastoreState {
    directories: HashMap<B3Digest, Directory>,
    blob_index_digests: HashSet<B3Digest>,
    chunk_digests: HashSet<B3Digest>,
}

struct RootRetentionState {
    all_roots: Vec<GcRootRecord>,
    retention_plan: RetentionPlan,
    retained_roots: Vec<GcRootRecord>,
}

struct GcCandidateState {
    live_pathinfos: Vec<PathInfo>,
    dead_pathinfos: Vec<PathInfo>,
    live_castore: LiveCastoreState,
    orphaned_on_disk: Vec<PathBuf>,
    artifact_attestation_paths: Vec<PathBuf>,
    closure_attestation_paths: Vec<PathBuf>,
    blob_index_paths: Vec<PathBuf>,
    blob_chunk_paths: Vec<PathBuf>,
    action_result_record_paths: Vec<PathBuf>,
    action_result_index_paths: Vec<PathBuf>,
    reclaim_summary: ReclaimObservationSummary,
}

struct GcPlan {
    plan_id: String,
    retention_plan_id: String,
    core_decision: GcReportDecision,
    retained_roots: Vec<GcRootRecord>,
    retention_explanations: Vec<GcRetentionExplanation>,
    path_explanations: Vec<GcPathExplanation>,
    base_reachability: Vec<GcBaseReachability>,
    usage: GcUsageReport,
    live_pathinfos: Vec<PathInfo>,
    dead_pathinfos: Vec<PathInfo>,
    live_castore: LiveCastoreState,
    orphaned_on_disk: Vec<PathBuf>,
    artifact_attestation_paths: Vec<PathBuf>,
    closure_attestation_paths: Vec<PathBuf>,
    blob_index_paths: Vec<PathBuf>,
    blob_chunk_paths: Vec<PathBuf>,
    action_result_record_paths: Vec<PathBuf>,
    action_result_index_paths: Vec<PathBuf>,
    candidate_paths: Vec<String>,
    reclaimable_bytes_total: u64,
    reclaim_observations: Vec<GcReclaimObservation>,
}

// r[impl store_lifecycle.safe_gc_execution]
pub async fn run_gc(
    ctx: &GcContext<'_>,
    ca_mappings: &mut CaMappings,
    accepted_plan_id: Option<&str>,
) -> Result<GcReport, Error> {
    assert!(!ctx.store_dir.is_empty(), "store_dir must not be empty");
    assert!(ctx.store_dir.starts_with('/'), "store_dir must be absolute");
    if ctx.retained_castore_roots.len() > MAX_GC_RETAINED_CASTORE_ROOTS {
        return Err(Error::Gc(format!("retained castore root count exceeds {MAX_GC_RETAINED_CASTORE_ROOTS}")));
    }

    let is_execution_requested = accepted_plan_id.is_some();
    let plan = build_plan(ctx, is_execution_requested).await?;
    if let Some(accepted_plan_id) = accepted_plan_id
        && accepted_plan_id != plan.plan_id
    {
        return Err(Error::Gc(format!("stale-gc-plan: accepted={accepted_plan_id} observed={}", plan.plan_id)));
    }
    let core_decision = plan.core_decision;
    let gc_result = GcReport {
        schema: GC_REPORT_SCHEMA.to_string(),
        plan_id: plan.plan_id.clone(),
        retention_plan_id: plan.retention_plan_id.clone(),
        overlay_plan_blake3: ctx.overlay_plan_identity.as_ref().map(encode_blake3_identity),
        is_dry_run: !is_execution_requested,
        execution_complete: false,
        failed_operation: None,
        failed_operations: Vec::new(),
        retained_root_count: saturating_u32(plan.retained_roots.len()),
        retained_castore_root_count: saturating_u32(ctx.retained_castore_roots.len()),
        candidate_path_count: saturating_u32(plan.dead_pathinfos.len()),
        reclaimable_bytes_total: plan.reclaimable_bytes_total,
        reclaim_observations: plan.reclaim_observations.clone(),
        candidate_paths: plan.candidate_paths.clone(),
        retention_explanations: plan.retention_explanations.clone(),
        path_explanations: plan.path_explanations.clone(),
        base_reachability: plan.base_reachability.clone(),
        usage: plan.usage.clone(),
        candidate_blob_index_count: saturating_u32(plan.blob_index_paths.len()),
        candidate_blob_chunk_count: saturating_u32(plan.blob_chunk_paths.len()),
        candidate_artifact_attestation_count: saturating_u32(plan.artifact_attestation_paths.len()),
        candidate_closure_attestation_count: saturating_u32(plan.closure_attestation_paths.len()),
        candidate_exported_output_count: saturating_u32(plan.orphaned_on_disk.len()),
        candidate_action_result_record_count: saturating_u32(plan.action_result_record_paths.len()),
        candidate_action_result_index_count: saturating_u32(plan.action_result_index_paths.len()),
        operations: Vec::new(),
    };
    assert_eq!(core_decision.candidate_path_count, plan.dead_pathinfos.len());
    assert_eq!(
        core_decision.retained_path_count,
        plan.live_pathinfos.len().saturating_add(plan.base_reachability.len()),
    );
    if core_decision.mutation_disposition == GcMutationDisposition::ReportOnly {
        return Ok(gc_result);
    }
    execute_gc_plan(ctx, ca_mappings, &plan, gc_result).await
}

async fn execute_gc_plan(
    ctx: &GcContext<'_>,
    ca_mappings: &mut CaMappings,
    plan: &GcPlan,
    mut gc_result: GcReport,
) -> Result<GcReport, Error> {
    assert!(!gc_result.is_dry_run);
    assert_eq!(plan.core_decision.mutation_disposition, GcMutationDisposition::Execute);
    let live_paths = plan
        .live_pathinfos
        .iter()
        .map(|path_info| path_info.store_path.to_absolute_path_with_prefix(ctx.store_dir))
        .collect::<BTreeSet<_>>();
    let pathinfo_result = rewrite_pathinfo_db(ctx.state_dir, &plan.live_pathinfos).await;
    if !record_gc_operation(&mut gc_result, GcOperationKind::PathInfoRewrite, pathinfo_result) {
        return Ok(gc_result);
    }
    record_gc_operation(
        &mut gc_result,
        GcOperationKind::ExportedOutputs,
        remove_exported_outputs(&plan.orphaned_on_disk),
    );
    record_gc_operation(
        &mut gc_result,
        GcOperationKind::ArtifactAttestations,
        remove_files(&plan.artifact_attestation_paths),
    );
    record_gc_operation(
        &mut gc_result,
        GcOperationKind::ClosureAttestations,
        remove_files(&plan.closure_attestation_paths),
    );
    let directory_result = rewrite_directory_db(ctx.state_dir, plan.live_castore.directories.values()).await;
    if !record_gc_operation(&mut gc_result, GcOperationKind::DirectoryRewrite, directory_result) {
        return Ok(gc_result);
    }
    record_gc_operation(&mut gc_result, GcOperationKind::BlobIndexFiles, remove_files(&plan.blob_index_paths));
    record_gc_operation(&mut gc_result, GcOperationKind::BlobChunkFiles, remove_files(&plan.blob_chunk_paths));
    let action_result = remove_action_result_files(&plan.action_result_record_paths, &plan.action_result_index_paths);
    record_gc_operation(&mut gc_result, GcOperationKind::ActionResults, action_result);
    ca_mappings.retain_output_paths(&live_paths);
    let ca_mapping_result = ca_mappings
        .save_checked(ctx.state_dir)
        .map_err(|error| Error::Gc(format!("saving CA mappings: {error}")));
    record_gc_operation(&mut gc_result, GcOperationKind::CaMappings, ca_mapping_result);
    gc_result.execution_complete = gc_result.failed_operations.is_empty();
    Ok(gc_result)
}

fn plan_root_retention(ctx: &GcContext<'_>) -> Result<RootRetentionState, Error> {
    assert!(!ctx.store_dir.is_empty());
    assert!(ctx.store_dir.starts_with('/'));
    let all_roots = roots::list_roots(ctx.state_dir)?;
    let current_unix_s = roots::current_unix_seconds()?;
    let retention_policy = crate::retention::core_retention_policy();
    let retention_facts = crate::retention::records_to_core(&all_roots)?;
    let retention_plan = plan_retention(&retention_policy, current_unix_s, retention_facts)
        .map_err(|error| Error::Gc(format!("planning root retention: {error:?}")))?;
    let retained_root_ids = retention_plan
        .decisions
        .iter()
        .filter(|decision| decision.disposition.retains_path())
        .map(|decision| decision.path_id.as_str())
        .collect::<BTreeSet<_>>();
    let retained_roots = all_roots
        .iter()
        .filter(|root| retained_root_ids.contains(root.logical_path.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    Ok(RootRetentionState {
        all_roots,
        retention_plan,
        retained_roots,
    })
}

async fn collect_candidate_state(
    ctx: &GcContext<'_>,
    retained_roots: &[GcRootRecord],
    overlay_snapshot: Vec<PathInfo>,
    live_paths: &BTreeSet<String>,
) -> Result<GcCandidateState, Error> {
    assert!(retained_roots.len() <= crunch_gc_core::MAX_GC_ENTRIES);
    assert!(live_paths.len() <= crunch_gc_core::MAX_GC_ENTRIES);
    let (live_pathinfos, dead_pathinfos) = split_pathinfos(overlay_snapshot, live_paths, ctx.store_dir);
    let live_castore = collect_live_castore_state(
        &live_pathinfos,
        ctx.retained_castore_roots,
        ctx.overlay_directory_service,
        ctx.overlay_blob_service,
    )
    .await?;
    let orphaned_on_disk = collect_existing_exported_outputs(&dead_pathinfos, ctx.output_dir_str)?;
    let artifact_attestation_paths =
        collect_existing_artifact_attestation_paths(ctx.state_dir, ctx.store_dir, &dead_pathinfos)?;
    let closure_attestation_paths = collect_dead_closure_attestations(ctx.state_dir, retained_roots)?;
    let blob_index_paths = collect_dead_blob_files(ctx.state_dir, &live_castore.blob_index_digests, true)?;
    let blob_chunk_paths = collect_dead_blob_files(ctx.state_dir, &live_castore.chunk_digests, false)?;
    let action_result_gc = local_action_result_gc_candidates(ctx.state_dir, live_paths)
        .map_err(|error| Error::Gc(format!("planning action-result metadata collection: {error}")))?;
    let reclaim_summary = compute_reclaimable_bytes(ReclaimObservationPaths {
        orphaned_on_disk: &orphaned_on_disk,
        artifact_attestations: &artifact_attestation_paths,
        closure_attestations: &closure_attestation_paths,
        blob_indexes: &blob_index_paths,
        blob_chunks: &blob_chunk_paths,
        action_result_records: &action_result_gc.record_paths,
        action_result_indexes: &action_result_gc.index_marker_paths,
    })?;
    Ok(GcCandidateState {
        live_pathinfos,
        dead_pathinfos,
        live_castore,
        orphaned_on_disk,
        artifact_attestation_paths,
        closure_attestation_paths,
        blob_index_paths,
        blob_chunk_paths,
        action_result_record_paths: action_result_gc.record_paths,
        action_result_index_paths: action_result_gc.index_marker_paths,
        reclaim_summary,
    })
}

fn overlay_trusted_keys(ctx: &GcContext<'_>) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, Error> {
    if ctx.overlay_plan_identity.is_none() {
        return Ok(None);
    }
    crate::overlay::load_layer_trust_keys(ctx.state_dir).map(Some)
}

// r[impl store_lifecycle.gc_explanation]
async fn build_plan(ctx: &GcContext<'_>, is_execution_requested: bool) -> Result<GcPlan, Error> {
    assert!(!ctx.store_dir.is_empty(), "build_plan: store_dir must not be empty");
    assert!(ctx.store_dir.starts_with('/'), "build_plan: store_dir must be absolute");

    let RootRetentionState {
        all_roots,
        retention_plan,
        retained_roots,
    } = plan_root_retention(ctx)?;
    let overlay_snapshot = snapshot_pathinfos(ctx.overlay_pathinfo).await?;
    let overlay_trusted_keys = overlay_trusted_keys(ctx)?;
    let composed_snapshot = compose_gc_snapshot(
        &overlay_snapshot,
        &retained_roots,
        ctx.composed_pathinfo,
        ctx.store_dir,
        overlay_trusted_keys.as_deref(),
    )
    .await?;
    let core_plan =
        plan_gc(core_plan_request(&retained_roots, &composed_snapshot, ctx.store_dir, !is_execution_requested)?)
            .map_err(|error| shell_gc_plan_error(error, ctx.store_dir))?;
    let live_paths = core_plan.retained_path_ids.iter().cloned().collect::<BTreeSet<_>>();
    let usage = build_usage_report(
        &composed_snapshot.pathinfos,
        &core_plan.retaining_roots,
        &retention_plan.decisions,
        ctx.store_dir,
    )?;
    let maximum_explanation_link_count =
        crate::retention::store_retention_runtime_policy().limits.max_closure_links_per_explanation;
    let path_explanations = build_path_explanations(&core_plan, maximum_explanation_link_count)?;
    let base_reachability = build_base_reachability(&core_plan.retaining_roots, &composed_snapshot.ownership);
    let core_plan_id = core_plan.plan_id.into_bytes();
    let retention_plan_id_bytes = retention_plan.plan_id.into_bytes();
    let retention_plan_id = encode_blake3_identity(&retention_plan_id_bytes);
    let retention_explanations = build_retention_explanations(&retention_plan.decisions, &all_roots)?;
    let candidates = collect_candidate_state(ctx, &retained_roots, overlay_snapshot, &live_paths).await?;
    let candidate_paths = core_plan.candidate_path_ids.clone();
    let plan_id = execution_plan_id(ExecutionPlanIdentityInput {
        core_plan_id: &core_plan_id,
        retention_plan_id: &retention_plan_id_bytes,
        overlay_plan_identity: ctx.overlay_plan_identity.as_ref(),
        candidate_paths: &candidate_paths,
        reclaim_observations: &candidates.reclaim_summary.observations,
    });
    let core_decision = report_decision(core_plan);

    Ok(GcPlan {
        plan_id,
        retention_plan_id,
        core_decision,
        retained_roots,
        retention_explanations,
        path_explanations,
        base_reachability,
        usage,
        live_pathinfos: candidates.live_pathinfos,
        dead_pathinfos: candidates.dead_pathinfos,
        live_castore: candidates.live_castore,
        orphaned_on_disk: candidates.orphaned_on_disk,
        artifact_attestation_paths: candidates.artifact_attestation_paths,
        closure_attestation_paths: candidates.closure_attestation_paths,
        blob_index_paths: candidates.blob_index_paths,
        blob_chunk_paths: candidates.blob_chunk_paths,
        action_result_record_paths: candidates.action_result_record_paths,
        action_result_index_paths: candidates.action_result_index_paths,
        candidate_paths,
        reclaimable_bytes_total: candidates.reclaim_summary.reclaimable_bytes_total,
        reclaim_observations: candidates.reclaim_summary.observations,
    })
}

fn record_gc_operation(report: &mut GcReport, kind: GcOperationKind, result: Result<(), Error>) -> bool {
    match result {
        Ok(()) => {
            report.operations.push(kind);
            true
        }
        Err(error) => {
            let failure = format!("{kind:?}: {error}");
            if report.failed_operation.is_none() {
                report.failed_operation = Some(failure.clone());
            }
            report.failed_operations.push(failure);
            false
        }
    }
}

fn remove_action_result_files(record_paths: &[PathBuf], index_paths: &[PathBuf]) -> Result<(), Error> {
    let expected_path_count = record_paths
        .len()
        .checked_add(index_paths.len())
        .ok_or_else(|| Error::Gc("action-result cleanup path count overflowed usize".to_string()))?;
    remove_file_iter(record_paths.iter().chain(index_paths), expected_path_count)
}

fn build_retention_explanations(
    decisions: &[RetentionDecision],
    roots: &[GcRootRecord],
) -> Result<Vec<GcRetentionExplanation>, Error> {
    assert_eq!(decisions.len(), roots.len());
    let roots_by_path = roots.iter().map(|root| (root.logical_path.as_str(), root)).collect::<BTreeMap<_, _>>();
    assert!(roots_by_path.len() <= roots.len());
    let mut explanations = Vec::with_capacity(decisions.len());
    for decision in decisions {
        let root = roots_by_path
            .get(decision.path_id.as_str())
            .ok_or_else(|| Error::Gc(format!("retention decision has no root record: {}", decision.path_id)))?;
        explanations.push(GcRetentionExplanation {
            path: decision.path_id.clone(),
            root_class: decision.class.as_str().to_string(),
            owner_scope: decision.owner_scope.clone(),
            policy_blake3: root.policy_blake3.clone(),
            project_identity: root.project_identity.clone(),
            selector: root.selector.clone(),
            generation: root.generation,
            lease_id: root.lease.as_ref().map(|lease| lease.lease_id.clone()),
            transition_id: root.last_transition_id.clone(),
            transition_reason: root.last_transition_reason.clone(),
            disposition: retention_disposition_name(decision.disposition).to_string(),
            reason: decision.reason.as_str().to_string(),
        });
    }
    Ok(explanations)
}

const fn retention_disposition_name(disposition: RetentionDisposition) -> &'static str {
    match disposition {
        RetentionDisposition::Keep => "keep",
        RetentionDisposition::Expire => "expire",
        RetentionDisposition::Migrate => "migrate",
        RetentionDisposition::Quarantine => "quarantine",
        RetentionDisposition::Remove => "remove",
    }
}

fn build_path_explanations(
    core_plan: &crunch_gc_core::GcPlan,
    retaining_root_limit: usize,
) -> Result<Vec<GcPathExplanation>, Error> {
    if retaining_root_limit == 0 {
        return Err(Error::Gc("retaining-root explanation limit must be positive".to_string()));
    }
    assert!(retaining_root_limit > 0);
    assert!(core_plan.retaining_roots.len() <= crunch_gc_core::MAX_GC_ENTRIES);
    let mut explanations = Vec::with_capacity(core_plan.retaining_roots.len());
    for retained in &core_plan.retaining_roots {
        if retained.root_ids.len() > retaining_root_limit {
            return Err(Error::Gc(format!(
                "retaining-root explanation exceeds policy limit {retaining_root_limit}: {}",
                retained.path_id
            )));
        }
        explanations.push(GcPathExplanation {
            path: retained.path_id.clone(),
            reason: "reachable-from-retained-root".to_string(),
            retaining_roots: retained.root_ids.clone(),
        });
    }
    explanations.extend(core_plan.candidate_path_ids.iter().map(|path| GcPathExplanation {
        path: path.clone(),
        reason: "unreachable-from-retained-root".to_string(),
        retaining_roots: Vec::new(),
    }));
    explanations.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(explanations)
}

// r[impl store_lifecycle.usage_report]
fn build_usage_report(
    snapshot: &[PathInfo],
    retaining_roots: &[crunch_gc_core::GcRetainingRoots],
    decisions: &[RetentionDecision],
    store_dir: &str,
) -> Result<GcUsageReport, Error> {
    assert!(!store_dir.is_empty());
    assert!(store_dir.starts_with('/'));
    let roots_by_path = retaining_roots
        .iter()
        .map(|entry| (entry.path_id.as_str(), entry.root_ids.clone()))
        .collect::<BTreeMap<_, _>>();
    let observations = snapshot
        .iter()
        .map(|path_info| {
            let path = path_info.store_path.to_absolute_path_with_prefix(store_dir);
            UsageObjectObservation {
                object_id: path.clone(),
                bytes: Some(path_info.nar_size),
                unknown_reason: None,
                retaining_root_ids: roots_by_path.get(path.as_str()).cloned().unwrap_or_default(),
            }
        })
        .collect();
    let usage_aggregate = aggregate_usage(decisions, observations)
        .map_err(|error| Error::Gc(format!("aggregating store usage: {error:?}")))?;
    Ok(GcUsageReport {
        observed_bytes: usage_aggregate.observed_bytes,
        retained_bytes: usage_aggregate.retained_bytes,
        reclaimable_bytes: usage_aggregate.reclaimable_bytes,
        quarantined_bytes: usage_aggregate.quarantined_bytes,
        unclassified_bytes: usage_aggregate.unclassified_bytes,
        shared_bytes: usage_aggregate.shared_bytes,
        unknown_object_count: usage_aggregate.unknown_object_count,
        roots: usage_aggregate
            .roots
            .into_iter()
            .map(|root| GcRootUsage {
                root: root.root_id,
                inclusive_bytes: root.inclusive_bytes,
                unique_bytes: root.unique_bytes,
                unknown_object_count: root.unknown_object_count,
            })
            .collect(),
    })
}

struct ExecutionPlanIdentityInput<'a> {
    core_plan_id: &'a [u8; blake3::OUT_LEN],
    retention_plan_id: &'a [u8; blake3::OUT_LEN],
    overlay_plan_identity: Option<&'a [u8; blake3::OUT_LEN]>,
    candidate_paths: &'a [String],
    reclaim_observations: &'a [GcReclaimObservation],
}

fn execution_plan_id(input: ExecutionPlanIdentityInput<'_>) -> String {
    assert_eq!(input.core_plan_id.len(), blake3::OUT_LEN);
    assert!(input.candidate_paths.len() <= crunch_gc_core::MAX_GC_ENTRIES);
    let mut hasher = blake3::Hasher::new();
    hasher.update(GC_EXECUTION_PLAN_DOMAIN);
    hasher.update(input.core_plan_id);
    hasher.update(input.retention_plan_id);
    match input.overlay_plan_identity {
        Some(overlay_plan_identity) => {
            hasher.update(&[1]);
            hasher.update(overlay_plan_identity);
        }
        None => {
            hasher.update(&[0]);
        }
    };
    hash_plan_strings(&mut hasher, input.candidate_paths);
    hasher.update(&(input.reclaim_observations.len() as u128).to_be_bytes());
    for observation in input.reclaim_observations {
        hash_plan_string(&mut hasher, &observation.category);
        hash_plan_string(&mut hasher, &observation.path);
        match observation.path_kind.as_deref() {
            Some(path_kind) => {
                hasher.update(&[1]);
                hash_plan_string(&mut hasher, path_kind);
            }
            None => {
                hasher.update(&[0]);
            }
        }
        match observation.bytes {
            Some(bytes) => {
                hasher.update(&[1]);
                hasher.update(&bytes.to_be_bytes());
            }
            None => {
                hasher.update(&[0]);
            }
        }
        match observation.blocker.as_deref() {
            Some(blocker) => {
                hasher.update(&[1]);
                hash_plan_string(&mut hasher, blocker);
            }
            None => {
                hasher.update(&[0]);
            }
        }
    }
    encode_blake3_identity(hasher.finalize().as_bytes())
}

fn hash_plan_strings(hasher: &mut blake3::Hasher, values: &[String]) {
    hasher.update(&(values.len() as u128).to_be_bytes());
    for value in values {
        hash_plan_string(hasher, value);
    }
}

fn hash_plan_string(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u128).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn encode_blake3_identity(bytes: &[u8; blake3::OUT_LEN]) -> String {
    format!("b3:{}", HEXLOWER.encode(bytes))
}

struct GcSnapshotSeed {
    pathinfos_by_id: BTreeMap<String, PathInfo>,
    ownership: BTreeMap<String, GcOwnership>,
    queue: Vec<String>,
}

struct ComposedPathInfoRequest<'a> {
    path_id: &'a str,
    store_dir: &'a str,
}

fn seed_overlay_snapshot(
    overlay_snapshot: &[PathInfo],
    store_dir: &str,
    overlay_trusted_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
) -> Result<GcSnapshotSeed, Error> {
    assert!(!store_dir.is_empty());
    assert!(overlay_snapshot.len() <= crunch_gc_core::MAX_GC_ENTRIES);
    let mut pathinfos_by_id = BTreeMap::new();
    let mut ownership = BTreeMap::new();
    let mut queue = Vec::with_capacity(overlay_snapshot.len());
    for path_info in overlay_snapshot {
        if let Some(trusted_keys) = overlay_trusted_keys {
            crate::overlay::verify_pathinfo_trust(path_info, trusted_keys)
                .map_err(|error| Error::Gc(format!("overlay-gc-untrusted-layer: {error}")))?;
        }
        if pathinfos_by_id.len() >= crunch_gc_core::MAX_GC_ENTRIES {
            return Err(Error::Gc("overlay GC snapshot exceeds PathInfo limit".to_string()));
        }
        let path_id = path_info.store_path.to_absolute_path_with_prefix(store_dir);
        if pathinfos_by_id.insert(path_id.clone(), path_info.clone()).is_some()
            || ownership.insert(path_id.clone(), GcOwnership::Overlay).is_some()
        {
            return Err(Error::Gc(format!("duplicate overlay PathInfo during GC: {path_id}")));
        }
        queue.push(path_id);
    }
    Ok(GcSnapshotSeed {
        pathinfos_by_id,
        ownership,
        queue,
    })
}

async fn load_composed_pathinfo(
    composed_pathinfo: &dyn PathInfoService,
    request: ComposedPathInfoRequest<'_>,
) -> Result<(PathInfo, usize), Error> {
    let path_id = request.path_id;
    let store_dir = request.store_dir;
    let store_path: StorePath<String> = StorePath::from_absolute_path_with_prefix(path_id.as_bytes(), store_dir)
        .map_err(|error| Error::Gc(format!("parsing composed GC path {path_id}: {error}")))?;
    let read = composed_pathinfo
        .get_with_layer(*store_path.digest())
        .await
        .map_err(|error| Error::Gc(format!("resolving composed GC path {path_id}: {error}")))?
        .ok_or_else(|| Error::MissingClosureFacts {
            path: store_path,
            store_dir: store_dir.to_string(),
            detail: "composed GC path has no PathInfo".to_string(),
        })?;
    if read.layer_index == 0 {
        return Err(Error::Gc(format!("overlay PathInfo is readable but absent from overlay GC listing: {path_id}")));
    }
    let observed_path = read.value.store_path.to_absolute_path_with_prefix(store_dir);
    if observed_path != path_id {
        return Err(Error::Gc(format!("composed GC digest collision: requested {path_id}, observed {observed_path}")));
    }
    assert!(read.layer_index > 0);
    assert_eq!(observed_path, path_id);
    Ok((read.value, read.layer_index))
}

async fn compose_gc_snapshot(
    overlay_snapshot: &[PathInfo],
    retained_roots: &[GcRootRecord],
    composed_pathinfo: &dyn PathInfoService,
    store_dir: &str,
    overlay_trusted_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
) -> Result<ComposedGcSnapshot, Error> {
    assert!(!store_dir.is_empty());
    assert!(retained_roots.len() <= crunch_gc_core::MAX_GC_ENTRIES);
    let GcSnapshotSeed {
        mut pathinfos_by_id,
        mut ownership,
        mut queue,
    } = seed_overlay_snapshot(overlay_snapshot, store_dir, overlay_trusted_keys)?;
    queue
        .try_reserve(retained_roots.len())
        .map_err(|error| Error::Gc(format!("reserving composed GC roots: {error}")))?;
    queue.extend(retained_roots.iter().map(|root| root.logical_path.clone()));

    let mut visited = BTreeSet::new();
    let mut queue_index = 0_usize;
    while queue_index < queue.len() {
        let path_id = queue[queue_index].clone();
        queue_index = queue_index.saturating_add(1);
        if !visited.insert(path_id.clone()) {
            continue;
        }
        if !pathinfos_by_id.contains_key(&path_id) {
            if pathinfos_by_id.len() >= crunch_gc_core::MAX_GC_ENTRIES
                || ownership.len() >= crunch_gc_core::MAX_GC_ENTRIES
            {
                return Err(Error::Gc(format!(
                    "composed GC snapshot exceeds {} PathInfos",
                    crunch_gc_core::MAX_GC_ENTRIES
                )));
            }
            let (path_info, layer_index) = load_composed_pathinfo(composed_pathinfo, ComposedPathInfoRequest {
                path_id: &path_id,
                store_dir,
            })
            .await?;
            ownership.insert(path_id.clone(), GcOwnership::Base { layer_index });
            pathinfos_by_id.insert(path_id.clone(), path_info);
        }
        let path_info = pathinfos_by_id
            .get(&path_id)
            .ok_or_else(|| Error::Gc(format!("composed GC snapshot lost PathInfo {path_id}")))?;
        let pending_path_count = queue
            .len()
            .checked_add(path_info.references.len())
            .ok_or_else(|| Error::Gc("composed GC queue length overflowed usize".to_string()))?;
        if pending_path_count > crunch_gc_core::MAX_GC_ENTRIES {
            return Err(Error::Gc(format!("composed GC queue exceeds {} paths", crunch_gc_core::MAX_GC_ENTRIES)));
        }
        queue
            .try_reserve(path_info.references.len())
            .map_err(|error| Error::Gc(format!("reserving composed GC references: {error}")))?;
        queue.extend(path_info.references.iter().map(|reference| reference.to_absolute_path_with_prefix(store_dir)));
    }
    Ok(ComposedGcSnapshot {
        pathinfos: pathinfos_by_id.into_values().collect(),
        ownership,
    })
}

fn build_base_reachability(
    retaining_roots: &[crunch_gc_core::GcRetainingRoots],
    ownership: &BTreeMap<String, GcOwnership>,
) -> Vec<GcBaseReachability> {
    retaining_roots
        .iter()
        .filter_map(|entry| match ownership.get(&entry.path_id) {
            Some(GcOwnership::Base { layer_index }) => Some(GcBaseReachability {
                path: entry.path_id.clone(),
                layer_index: *layer_index,
                retaining_roots: entry.root_ids.clone(),
            }),
            Some(GcOwnership::Overlay) | None => None,
        })
        .collect()
}

fn core_plan_request(
    retained_roots: &[GcRootRecord],
    snapshot: &ComposedGcSnapshot,
    store_dir: &str,
    is_dry_run: bool,
) -> Result<GcPlanRequest, Error> {
    if store_dir.is_empty() || !store_dir.starts_with('/') {
        return Err(Error::Gc(format!("core GC store directory is invalid: {store_dir:?}")));
    }
    assert!(!store_dir.is_empty());
    assert!(store_dir.starts_with('/'));
    let roots = retained_roots
        .iter()
        .map(|root| {
            roots::parse_logical_store_path(roots::LogicalStorePathRef {
                logical_path: &root.logical_path,
                store_dir,
            })
            .map(|path| path.to_absolute_path_with_prefix(store_dir))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let entries = snapshot
        .pathinfos
        .iter()
        .map(|path_info| {
            let path_id = path_info.store_path.to_absolute_path_with_prefix(store_dir);
            let ownership = snapshot
                .ownership
                .get(&path_id)
                .copied()
                .ok_or_else(|| Error::Gc(format!("composed GC snapshot has no ownership for {path_id}")))?;
            Ok(GcEntry {
                path_id,
                references: path_info
                    .references
                    .iter()
                    .map(|reference| reference.to_absolute_path_with_prefix(store_dir))
                    .collect(),
                declared_nar_bytes: path_info.nar_size,
                ownership,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let execution_mode = if is_dry_run {
        GcExecutionMode::DryRun
    } else {
        GcExecutionMode::Execute
    };
    Ok(GcPlanRequest {
        roots,
        entries,
        execution_mode,
    })
}

fn shell_gc_plan_error(error: GcPlanError, store_dir: &str) -> Error {
    let missing_path_id = match &error {
        GcPlanError::MissingRoot { path_id } => Some(path_id.as_str()),
        GcPlanError::MissingReference { reference_path_id, .. } => Some(reference_path_id.as_str()),
        _ => None,
    };
    if let Some(path_id) = missing_path_id
        && let Ok(path) = StorePath::from_absolute_path_with_prefix(path_id.as_bytes(), store_dir)
    {
        return Error::MissingClosureFacts {
            path,
            store_dir: store_dir.to_string(),
            detail: format!("normalized GC graph is incomplete: {error:?}"),
        };
    }
    Error::Gc(format!("planning normalized reachability: {error:?}"))
}

pub(crate) async fn snapshot_pathinfos(pathinfo: &dyn PathInfoService) -> Result<Vec<PathInfo>, Error> {
    let mut snapshot = Vec::with_capacity(INITIAL_PATHINFO_CAPACITY);
    let mut stream = pathinfo.list();
    for observed_entry_count in 0..=crunch_gc_core::MAX_GC_ENTRIES {
        let Some(item) = stream.next().await else {
            break;
        };
        if observed_entry_count == crunch_gc_core::MAX_GC_ENTRIES {
            return Err(Error::Gc(format!("PathInfo snapshot exceeds {} entries", crunch_gc_core::MAX_GC_ENTRIES)));
        }
        let path_info = item.map_err(|err| Error::Gc(format!("listing PathInfo rows: {err}")))?;
        snapshot.push(path_info);
    }
    Ok(snapshot)
}

fn split_pathinfos(
    snapshot: Vec<PathInfo>,
    live_paths: &BTreeSet<String>,
    store_dir: &str,
) -> (Vec<PathInfo>, Vec<PathInfo>) {
    let mut live = Vec::with_capacity(snapshot.len());
    let mut dead = Vec::with_capacity(snapshot.len());
    for path_info in snapshot {
        let path_id = path_info.store_path.to_absolute_path_with_prefix(store_dir);
        if live_paths.contains(&path_id) {
            live.push(path_info);
        } else {
            dead.push(path_info);
        }
    }
    live.sort_by_key(|path_info| path_info.store_path.to_string());
    dead.sort_by_key(|path_info| path_info.store_path.to_string());
    (live, dead)
}

async fn collect_live_castore_state(
    live_pathinfos: &[PathInfo],
    retained_castore_roots: &[Node],
    directory_service: &dyn DirectoryService,
    blob_service: &dyn BlobService,
) -> Result<LiveCastoreState, Error> {
    let mut state = LiveCastoreState {
        directories: HashMap::new(),
        blob_index_digests: HashSet::new(),
        chunk_digests: HashSet::new(),
    };
    for path_info in live_pathinfos {
        collect_node_state(&path_info.node, directory_service, blob_service, &mut state).await?;
    }
    for node in retained_castore_roots {
        collect_node_state(node, directory_service, blob_service, &mut state).await?;
    }
    Ok(state)
}

async fn collect_node_state(
    node: &Node,
    directory_service: &dyn DirectoryService,
    blob_service: &dyn BlobService,
    state: &mut LiveCastoreState,
) -> Result<(), Error> {
    match node {
        Node::File { digest, .. } => collect_blob_state(*digest, blob_service, state).await,
        Node::Symlink { .. } => Ok(()),
        Node::Directory { digest, .. } => {
            collect_directory_state(*digest, directory_service, blob_service, state).await
        }
    }
}

async fn collect_blob_state(
    digest: B3Digest,
    blob_service: &dyn BlobService,
    state: &mut LiveCastoreState,
) -> Result<(), Error> {
    assert!(!digest.as_ref().is_empty(), "blob digest must not be empty");

    if state.blob_index_digests.contains(&digest) || state.chunk_digests.contains(&digest) {
        return Ok(());
    }
    let chunks = blob_service
        .chunks(&digest)
        .await
        .map_err(|err| Error::Gc(format!("reading blob metadata for {}: {err}", encode_digest(&digest))))?;
    let Some(chunks) = chunks else {
        return Err(Error::Gc(format!(
            "missing castore blob metadata for reachable digest {}",
            encode_digest(&digest)
        )));
    };
    if chunks.is_empty() {
        state.chunk_digests.insert(digest);
        return Ok(());
    }
    state.blob_index_digests.insert(digest);
    for chunk in chunks {
        let chunk_digest = B3Digest::try_from(chunk.digest)
            .map_err(|err| Error::Gc(format!("invalid chunk digest in blob {}: {err}", encode_digest(&digest))))?;
        state.chunk_digests.insert(chunk_digest);
    }
    let is_blob_recorded = state.blob_index_digests.contains(&digest) || state.chunk_digests.contains(&digest);
    assert!(is_blob_recorded, "processed blob must be recorded in state");
    Ok(())
}

async fn collect_directory_state(
    root_digest: B3Digest,
    directory_service: &dyn DirectoryService,
    blob_service: &dyn BlobService,
    state: &mut LiveCastoreState,
) -> Result<(), Error> {
    assert!(!root_digest.as_ref().is_empty(), "directory digest must not be empty");

    if state.directories.contains_key(&root_digest) {
        return Ok(());
    }

    const MAX_DIR_ENTRIES: usize = 1_000_000;
    let mut stream = directory_service.get_recursive(&root_digest);
    let mut has_seen_root_digest = false;
    for _ in 0..MAX_DIR_ENTRIES {
        let Some(item) = stream.next().await else { break };
        let directory =
            item.map_err(|err| Error::Gc(format!("reading directory {}: {err}", encode_digest(&root_digest))))?;
        let digest = directory.digest();
        if digest == root_digest {
            has_seen_root_digest = true;
        }
        if state.directories.insert(digest, directory.clone()).is_some() {
            continue;
        }
        for (_name, child) in directory.nodes() {
            match child {
                Node::File { digest, .. } => collect_blob_state(*digest, blob_service, state).await?,
                Node::Symlink { .. } => {}
                Node::Directory { .. } => {}
            }
        }
    }
    if !has_seen_root_digest {
        return Err(Error::Gc(format!(
            "missing directory metadata for reachable digest {}",
            encode_digest(&root_digest)
        )));
    }
    assert!(state.directories.contains_key(&root_digest), "root directory must be in state after walk");
    assert!(has_seen_root_digest, "directory stream must include the root");
    Ok(())
}

fn collect_existing_exported_outputs(dead_pathinfos: &[PathInfo], output_dir_str: &str) -> Result<Vec<PathBuf>, Error> {
    let mut paths = Vec::with_capacity(dead_pathinfos.len());
    for path_info in dead_pathinfos {
        let path = PathBuf::from(path_info.store_path.to_absolute_path_with_prefix(output_dir_str));
        if path.exists() || symlink_exists(&path)? {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn collect_existing_artifact_attestation_paths(
    state_dir: &Path,
    store_dir: &str,
    dead_pathinfos: &[PathInfo],
) -> Result<Vec<PathBuf>, Error> {
    let mut paths = Vec::with_capacity(dead_pathinfos.len());
    for path_info in dead_pathinfos {
        let path = artifact_attestation_file_path(state_dir, store_dir, &path_info.store_path);
        if path.exists() {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn collect_dead_closure_attestations(state_dir: &Path, retained_roots: &[GcRootRecord]) -> Result<Vec<PathBuf>, Error> {
    assert!(
        retained_roots.iter().all(|root| !root.logical_path.is_empty()),
        "gc root logical paths must not be empty"
    );

    let live_roots: BTreeSet<String> = retained_roots.iter().map(|root| root.logical_path.clone()).collect();
    let closure_dir = state_dir.join("attestations").join("closures");
    let mut dead_paths = Vec::with_capacity(64);
    if !closure_dir.exists() {
        return Ok(dead_paths);
    }
    let mut scanned_entries: u32 = 0;
    for entry in
        std::fs::read_dir(&closure_dir).map_err(|err| Error::Gc(format!("reading {}: {err}", closure_dir.display())))?
    {
        let entry = entry.map_err(|err| Error::Gc(format!("reading {} entry: {err}", closure_dir.display())))?;
        scanned_entries = scanned_entries.saturating_add(1);
        if scanned_entries > MAX_GC_FILE_SCAN_ENTRIES {
            return Err(Error::Gc(format!("closure attestation scan exceeded {} entries", MAX_GC_FILE_SCAN_ENTRIES)));
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let closure: crunch_attestation::ClosureAttestation = match serde_json::from_slice(&bytes) {
            Ok(closure) => closure,
            Err(_) => continue,
        };
        let is_closure_live = closure
            .facts
            .root_node_ids
            .iter()
            .filter_map(|node_id| node_id.strip_prefix("artifact:"))
            .all(|logical_path| live_roots.contains(logical_path));
        if !is_closure_live {
            dead_paths.push(path);
        }
    }
    // dead_paths.len() fits in u64 on any platform where usize <= u64.
    let dead_count = u64::try_from(dead_paths.len());
    debug_assert!(dead_count.is_ok(), "dead_paths.len() overflows u64");
    if let Ok(n) = dead_count {
        assert!(n <= u64::from(scanned_entries), "dead paths cannot exceed scanned entries");
    }
    Ok(dead_paths)
}

fn collect_dead_blob_files(
    state_dir: &Path,
    live_digests: &HashSet<B3Digest>,
    metadata_files: bool,
) -> Result<Vec<PathBuf>, Error> {
    let root = if metadata_files {
        state_dir.join("blobs").join("blobs").join("b3")
    } else {
        state_dir.join("blobs").join("chunks").join("b3")
    };
    let mut dead_paths = Vec::with_capacity(256);
    if !root.exists() {
        return Ok(dead_paths);
    }
    let mut scanned_entries: u32 = 0;
    scan_blob_dir(&root, &mut scanned_entries, live_digests, &mut dead_paths)?;
    Ok(dead_paths)
}

fn scan_blob_dir(
    root: &Path,
    scanned_entries: &mut u32,
    live_digests: &HashSet<B3Digest>,
    dead_paths: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    assert!(root.is_absolute(), "scan_blob_dir: root must be absolute");
    let dead_count_before = dead_paths.len();

    const MAX_SCAN_DEPTH: usize = 8;
    let mut worklist = Vec::with_capacity(MAX_SCAN_DEPTH);
    worklist.push(root.to_path_buf());

    while let Some(current_dir) = worklist.pop() {
        assert!(worklist.len() < MAX_SCAN_DEPTH, "blob scan directory nesting exceeded {MAX_SCAN_DEPTH}");
        for entry in std::fs::read_dir(&current_dir)
            .map_err(|err| Error::Gc(format!("reading {}: {err}", current_dir.display())))?
        {
            let entry = entry.map_err(|err| Error::Gc(format!("reading {} entry: {err}", current_dir.display())))?;
            *scanned_entries = scanned_entries.saturating_add(1);
            if *scanned_entries > MAX_GC_FILE_SCAN_ENTRIES {
                return Err(Error::Gc(format!("blob scan exceeded {} entries", MAX_GC_FILE_SCAN_ENTRIES)));
            }
            let path = entry.path();
            let file_type =
                entry.file_type().map_err(|err| Error::Gc(format!("reading {} file type: {err}", path.display())))?;
            if file_type.is_dir() {
                worklist.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.len() != B3Digest::LENGTH.saturating_mul(2) {
                continue;
            };
            let Ok(decoded) = HEXLOWER.decode(name.as_bytes()) else {
                continue;
            };
            let Ok(digest) = B3Digest::try_from(decoded) else {
                continue;
            };
            if !live_digests.contains(&digest) {
                dead_paths.push(path);
            }
        }
    }
    assert!(dead_paths.len() >= dead_count_before, "scan can only grow dead_paths, never shrink");
    Ok(())
}

const RECLAIM_CATEGORY_EXPORTED_OUTPUT: &str = "exported-output";
const RECLAIM_CATEGORY_ARTIFACT_ATTESTATION: &str = "artifact-attestation";
const RECLAIM_CATEGORY_CLOSURE_ATTESTATION: &str = "closure-attestation";
const RECLAIM_CATEGORY_BLOB_INDEX: &str = "blob-index";
const RECLAIM_CATEGORY_BLOB_CHUNK: &str = "blob-chunk";
const RECLAIM_CATEGORY_ACTION_RESULT_RECORD: &str = "action-result-record";
const RECLAIM_CATEGORY_ACTION_RESULT_INDEX: &str = "action-result-index";
const MAX_RECLAIM_PATH_WORKLIST: usize = 256;
const INITIAL_RECLAIM_PATH_WORKLIST_CAPACITY: usize = 16;

struct ReclaimObservationPaths<'a> {
    orphaned_on_disk: &'a [PathBuf],
    artifact_attestations: &'a [PathBuf],
    closure_attestations: &'a [PathBuf],
    blob_indexes: &'a [PathBuf],
    blob_chunks: &'a [PathBuf],
    action_result_records: &'a [PathBuf],
    action_result_indexes: &'a [PathBuf],
}

struct ReclaimObservationSummary {
    reclaimable_bytes_total: u64,
    observations: Vec<GcReclaimObservation>,
}

struct PathSizeObservation {
    bytes: u64,
    path_kind: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathSizeBlocker {
    MetadataUnreadable,
    DirectoryUnreadable,
    DirectoryEntryUnreadable,
    EntryLimitExceeded,
    WorklistLimitExceeded,
    ByteOverflow,
}

impl PathSizeBlocker {
    const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataUnreadable => "metadata-unreadable",
            Self::DirectoryUnreadable => "directory-unreadable",
            Self::DirectoryEntryUnreadable => "directory-entry-unreadable",
            Self::EntryLimitExceeded => "entry-limit-exceeded",
            Self::WorklistLimitExceeded => "worklist-limit-exceeded",
            Self::ByteOverflow => "byte-overflow",
        }
    }
}

fn compute_reclaimable_bytes(paths: ReclaimObservationPaths<'_>) -> Result<ReclaimObservationSummary, Error> {
    let categorized_paths = [
        (RECLAIM_CATEGORY_EXPORTED_OUTPUT, paths.orphaned_on_disk),
        (RECLAIM_CATEGORY_ARTIFACT_ATTESTATION, paths.artifact_attestations),
        (RECLAIM_CATEGORY_CLOSURE_ATTESTATION, paths.closure_attestations),
        (RECLAIM_CATEGORY_BLOB_INDEX, paths.blob_indexes),
        (RECLAIM_CATEGORY_BLOB_CHUNK, paths.blob_chunks),
        (RECLAIM_CATEGORY_ACTION_RESULT_RECORD, paths.action_result_records),
        (RECLAIM_CATEGORY_ACTION_RESULT_INDEX, paths.action_result_indexes),
    ];
    let observation_count = categorized_paths
        .iter()
        .try_fold(0_usize, |count, (_, category_paths)| count.checked_add(category_paths.len()));
    let Some(observation_count) = observation_count else {
        return Err(Error::Gc("reclaim observation count overflowed usize".to_string()));
    };
    if observation_count > crunch_gc_core::MAX_RECLAIM_OBSERVATIONS {
        return Err(Error::Gc(format!(
            "reclaim observation count exceeds {}",
            crunch_gc_core::MAX_RECLAIM_OBSERVATIONS
        )));
    }
    let mut known_sizes_bytes = Vec::with_capacity(observation_count);
    let mut observations = Vec::with_capacity(observation_count);
    for (category, category_paths) in categorized_paths {
        for path in category_paths {
            debug_assert!(path.is_absolute());
            match path_size_bytes(path) {
                Ok(observation) => {
                    known_sizes_bytes.push(observation.bytes);
                    observations.push(GcReclaimObservation {
                        category: category.to_string(),
                        path: path.to_string_lossy().into_owned(),
                        path_kind: Some(observation.path_kind.to_string()),
                        bytes: Some(observation.bytes),
                        blocker: None,
                    });
                }
                Err(blocker) => observations.push(GcReclaimObservation {
                    category: category.to_string(),
                    path: path.to_string_lossy().into_owned(),
                    path_kind: None,
                    bytes: None,
                    blocker: Some(blocker.as_str().to_string()),
                }),
            }
        }
    }
    let summary = summarize_reclaim_observations(known_sizes_bytes)
        .map_err(|error| Error::Gc(format!("summarizing reclaimable bytes: {error:?}")))?;
    Ok(ReclaimObservationSummary {
        reclaimable_bytes_total: summary.reclaimable_bytes,
        observations,
    })
}

fn path_size_bytes(root: &Path) -> Result<PathSizeObservation, PathSizeBlocker> {
    assert!(root.is_absolute(), "path_size_bytes: root must be absolute");
    let root_metadata = std::fs::symlink_metadata(root).map_err(|_| PathSizeBlocker::MetadataUnreadable)?;
    let path_kind = if root_metadata.file_type().is_symlink() {
        "symlink"
    } else if root_metadata.is_file() {
        "file"
    } else if root_metadata.is_dir() {
        "directory"
    } else {
        "special"
    };
    let mut total: u64 = 0;
    let mut seen_entries: u32 = 0;
    let mut worklist = Vec::with_capacity(INITIAL_RECLAIM_PATH_WORKLIST_CAPACITY);
    worklist.push(root.to_path_buf());

    while let Some(current) = worklist.pop() {
        seen_entries = seen_entries.saturating_add(1);
        if seen_entries > MAX_GC_BYTES_WALK_ENTRIES {
            return Err(PathSizeBlocker::EntryLimitExceeded);
        }

        let metadata = std::fs::symlink_metadata(&current).map_err(|_| PathSizeBlocker::MetadataUnreadable)?;
        if metadata.file_type().is_symlink() || metadata.is_file() {
            total = total.checked_add(metadata.len()).ok_or(PathSizeBlocker::ByteOverflow)?;
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }

        let entries = std::fs::read_dir(&current).map_err(|_| PathSizeBlocker::DirectoryUnreadable)?;
        for entry in entries {
            let entry = entry.map_err(|_| PathSizeBlocker::DirectoryEntryUnreadable)?;
            if worklist.len() >= MAX_RECLAIM_PATH_WORKLIST {
                return Err(PathSizeBlocker::WorklistLimitExceeded);
            }
            worklist.push(entry.path());
        }
    }
    assert!(seen_entries <= MAX_GC_BYTES_WALK_ENTRIES);
    Ok(PathSizeObservation {
        bytes: total,
        path_kind,
    })
}

fn remove_exported_outputs(paths: &[PathBuf]) -> Result<(), Error> {
    let mut failures = Vec::with_capacity(paths.len());
    for path in paths {
        if let Err(error) = remove_path(path) {
            failures.push(error.to_string());
        }
    }
    cleanup_result(failures)
}

fn remove_files(paths: &[PathBuf]) -> Result<(), Error> {
    remove_file_iter(paths.iter(), paths.len())
}

fn remove_file_iter<'a>(paths: impl Iterator<Item = &'a PathBuf>, expected_path_count: usize) -> Result<(), Error> {
    let mut failures = Vec::with_capacity(expected_path_count);
    for path in paths {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => failures.push(format!("removing {}: {error}", path.display())),
        }
    }
    cleanup_result(failures)
}

fn cleanup_result(failures: Vec<String>) -> Result<(), Error> {
    if failures.is_empty() {
        Ok(())
    } else {
        Err(Error::Gc(format!("{} independent cleanup failure(s): {}", failures.len(), failures.join("; "))))
    }
}

fn remove_path(path: &Path) -> Result<(), Error> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(Error::Gc(format!("reading metadata for {}: {err}", path.display()))),
    };
    if metadata.file_type().is_symlink() || metadata.is_file() {
        std::fs::remove_file(path).map_err(|err| Error::Gc(format!("removing {}: {err}", path.display())))?;
        return Ok(());
    }
    if metadata.is_dir() {
        prepare_directory_tree_for_removal(path)?;
        std::fs::remove_dir_all(path).map_err(|err| Error::Gc(format!("removing {}: {err}", path.display())))?;
    }
    Ok(())
}

fn prepare_directory_tree_for_removal(root: &Path) -> Result<(), Error> {
    if !root.is_dir() {
        return Err(Error::Gc(format!("removal preparation root is not a directory: {}", root.display())));
    }
    let mut pending = vec![root.to_path_buf()];
    let mut scanned_count = 0_u32;
    while let Some(path) = pending.pop() {
        if scanned_count >= MAX_GC_FILE_SCAN_ENTRIES {
            return Err(Error::Gc(format!(
                "removal preparation exceeded {MAX_GC_FILE_SCAN_ENTRIES} entries under {}",
                root.display()
            )));
        }
        scanned_count = scanned_count.saturating_add(1);
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|err| Error::Gc(format!("reading metadata for {}: {err}", path.display())))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        make_directory_owner_writable(&path, &metadata)?;
        for entry in
            std::fs::read_dir(&path).map_err(|err| Error::Gc(format!("reading directory {}: {err}", path.display())))?
        {
            let entry = entry.map_err(|err| Error::Gc(format!("reading {} entry: {err}", path.display())))?;
            pending.push(entry.path());
        }
    }
    assert!(scanned_count >= 1, "removal preparation must inspect its root");
    assert!(pending.is_empty(), "removal preparation must drain its worklist");
    Ok(())
}

#[cfg(unix)]
fn make_directory_owner_writable(path: &Path, metadata: &std::fs::Metadata) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;

    let current_mode = metadata.permissions().mode();
    let writable_mode = current_mode | OWNER_WRITE_PERMISSION_MODE;
    assert_ne!(writable_mode & OWNER_WRITE_PERMISSION_MODE, 0);
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(writable_mode))
        .map_err(|err| Error::Gc(format!("making {} removable: {err}", path.display())))
}

#[cfg(not(unix))]
fn make_directory_owner_writable(path: &Path, metadata: &std::fs::Metadata) -> Result<(), Error> {
    let mut permissions = metadata.permissions();
    permissions.set_readonly(false);
    std::fs::set_permissions(path, permissions)
        .map_err(|err| Error::Gc(format!("making {} removable: {err}", path.display())))
}

async fn rewrite_pathinfo_db(state_dir: &Path, live_pathinfos: &[PathInfo]) -> Result<(), Error> {
    use snix_store::pathinfoservice::RedbPathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

    assert!(state_dir.is_dir(), "rewrite_pathinfo_db: state_dir must exist");
    assert!(live_pathinfos.iter().all(|pi| !pi.signatures.is_empty()), "all retained PathInfos must be signed");

    let final_path = state_dir.join("pathinfo.redb");
    let tmp_path = state_dir.join("pathinfo.redb.gc-tmp");
    if tmp_path.exists()
        && let Err(err) = std::fs::remove_file(&tmp_path)
    {
        tracing::debug!(path = %tmp_path.display(), err = %err, "stale gc-tmp pathinfo already cleaned");
    }
    let tmp_service = RedbPathInfoService::new("crunch-gc-pathinfo".to_string(), RedbPathInfoServiceConfig {
        path: Some(tmp_path.clone()),
        read_only: false,
        cache_size: None,
    })
    .await
    .map_err(|err| Error::Gc(format!("opening {}: {err}", tmp_path.display())))?;
    for path_info in live_pathinfos {
        tmp_service
            .put(path_info.clone())
            .await
            .map_err(|err| Error::Gc(format!("writing rewritten PathInfo DB: {err}")))?;
    }
    drop(tmp_service);
    std::fs::rename(&tmp_path, &final_path)
        .map_err(|err| Error::Gc(format!("renaming {} -> {}: {err}", tmp_path.display(), final_path.display())))?;
    Ok(())
}

async fn rewrite_directory_db<'a>(
    state_dir: &Path,
    live_directories: impl Iterator<Item = &'a Directory>,
) -> Result<(), Error> {
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    assert!(state_dir.is_dir(), "rewrite_directory_db: state_dir must exist");

    let final_path = state_dir.join("directories.redb");
    let tmp_path = state_dir.join("directories.redb.gc-tmp");
    if tmp_path.exists()
        && let Err(err) = std::fs::remove_file(&tmp_path)
    {
        tracing::debug!(path = %tmp_path.display(), err = %err, "stale gc-tmp directories already cleaned");
    }
    let tmp_service = RedbDirectoryService::new("crunch-gc-directories".to_string(), RedbDirectoryServiceConfig {
        path: Some(tmp_path.clone()),
        read_only: false,
        cache_size: None,
    })
    .await
    .map_err(|err| Error::Gc(format!("opening {}: {err}", tmp_path.display())))?;
    for directory in live_directories {
        tmp_service
            .put(directory.clone())
            .await
            .map_err(|err| Error::Gc(format!("writing rewritten directory DB: {err}")))?;
    }
    drop(tmp_service);
    assert!(tmp_path.exists(), "rewrite_directory_db: tmp file must exist before rename");
    std::fs::rename(&tmp_path, &final_path)
        .map_err(|err| Error::Gc(format!("renaming {} -> {}: {err}", tmp_path.display(), final_path.display())))?;
    Ok(())
}

fn encode_digest(digest: &B3Digest) -> String {
    HEXLOWER.encode(digest.as_ref())
}

fn symlink_exists(path: &Path) -> Result<bool, Error> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(Error::Gc(format!("reading metadata for {}: {err}", path.display()))),
    }
}

#[allow(tigerstyle::sentinel_fallback)]
pub(crate) fn saturating_u32(len: usize) -> u32 {
    // Intentional saturation: signature/scan counts above u32::MAX are clamped.
    u32::try_from(len).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use futures::stream::BoxStream;
    use pretty_assertions::assert_eq;
    use snix_castore::SymlinkTarget;
    use snix_store::pathinfoservice;
    use tokio::io::AsyncWriteExt;
    use tokio_stream::wrappers::ReceiverStream;

    use super::*;
    use crate::StoreConfig;
    use crate::StoreHandle;
    use crate::handle::PersistOutputRequest;
    use crate::roots::GcRootSource;

    fn store_path(name: &str, seed: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [seed; 20]).unwrap()
    }

    async fn open_store(state_dir: &Path, output_dir: &Path) -> StoreHandle {
        StoreHandle::open(StoreConfig {
            state_dir: state_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            remote_cache_urls: Vec::new(),
            fallback_mode: crate::StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .unwrap()
    }

    async fn write_blob(store: &StoreHandle, contents: &[u8]) -> Node {
        let mut writer = store.blob_service().open_write().await;
        writer.write_all(contents).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: contents.len() as u64,
            executable: false,
        }
    }

    async fn write_directory_with_file(store: &StoreHandle, name: &str, contents: &[u8]) -> Node {
        let child = write_blob(store, contents).await;
        let directory = Directory::try_from_iter([(name.try_into().unwrap(), child)]).unwrap();
        let digest = store.directory_service().put(directory.clone()).await.unwrap();
        Node::Directory {
            digest,
            size: directory.size(),
        }
    }

    fn test_signature() -> nix_compat::narinfo::Signature<String> {
        let signing_key = nix_compat::narinfo::SigningKey::new(
            "gc-test-1".to_string(),
            ed25519_dalek::SigningKey::from_bytes(&[9u8; 32]),
        );
        signing_key.sign(b"gc-signed").to_owned()
    }

    fn signed_pathinfo(
        store_path: StorePath<String>,
        node: Node,
        refs: Vec<StorePath<String>>,
    ) -> snix_store::path_info::PathInfo {
        snix_store::path_info::PathInfo {
            store_path,
            node,
            references: refs,
            nar_size: 1,
            nar_sha256: [7u8; 32],
            signatures: vec![test_signature()],
            deriver: None,
            ca: None,
        }
    }

    async fn persist_output(
        store: &mut StoreHandle,
        output_name: &str,
        store_path: StorePath<String>,
        node: Node,
        refs: Vec<StorePath<String>>,
        is_root: bool,
        source: Option<GcRootSource>,
    ) {
        let path_info = signed_pathinfo(store_path.clone(), node.clone(), refs);
        store
            .persist_and_export_signed_output(PersistOutputRequest {
                output_name,
                output_path: &store_path,
                path_info,
                final_node: node,
                provenance: None,
                is_root,
                root_source: source,
            })
            .await
            .unwrap();
    }

    fn blob_chunk_path(state_dir: &Path, digest: &B3Digest) -> PathBuf {
        state_dir
            .join("blobs")
            .join("chunks")
            .join("b3")
            .join(HEXLOWER.encode(&digest.as_ref()[..2]))
            .join(HEXLOWER.encode(digest.as_ref()))
    }

    async fn reopen_store(state_dir: &Path, output_dir: &Path) -> StoreHandle {
        open_store(state_dir, output_dir).await
    }

    // r[verify store_lifecycle.gc_explanation]
    #[tokio::test]
    async fn dry_run_reports_same_candidates_as_real_run() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let keep_path = store_path("keep", 1);
        let drop_path = store_path("drop", 2);
        let keep_node = write_blob(&store, b"keep").await;
        let drop_node = write_blob(&store, b"drop").await;
        persist_output(&mut store, "out", keep_path.clone(), keep_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", drop_path.clone(), drop_node, vec![], true, None).await;

        let is_dry_run = store.garbage_collect(None).await.unwrap();
        assert!(is_dry_run.is_dry_run);
        assert!(is_dry_run.operations.is_empty());
        assert_eq!(is_dry_run.candidate_paths, vec![drop_path.to_absolute_path()]);
        assert!(is_dry_run.retention_explanations.iter().any(|item| item.path == keep_path.to_absolute_path()));
        assert!(is_dry_run.path_explanations.iter().any(|item| item.path == drop_path.to_absolute_path()));
        assert!(output_dir.path().join(drop_path.to_string()).exists());

        let accepted_plan_id = is_dry_run.plan_id.clone();
        let real = store.garbage_collect(Some(&accepted_plan_id)).await.unwrap();
        assert_eq!(real.candidate_paths, is_dry_run.candidate_paths);
        assert!(real.execution_complete);

        let reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let listed = crate::store_list(reopened.pathinfo_service().as_ref()).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].0, keep_path.to_string());
    }

    #[tokio::test]
    async fn operation_reporting_preserves_first_failure_and_records_later_work() {
        const EXPECTED_FAILURE_COUNT: usize = 2;
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let mut report = store.garbage_collect(None).await.unwrap();

        let first_succeeded = record_gc_operation(
            &mut report,
            GcOperationKind::ExportedOutputs,
            Err(Error::Gc("first deletion failure".to_string())),
        );
        let later_succeeded = record_gc_operation(&mut report, GcOperationKind::ArtifactAttestations, Ok(()));
        let second_succeeded = record_gc_operation(
            &mut report,
            GcOperationKind::ClosureAttestations,
            Err(Error::Gc("second deletion failure".to_string())),
        );

        assert!(!first_succeeded);
        assert!(later_succeeded);
        assert!(!second_succeeded);
        assert_eq!(report.operations, vec![GcOperationKind::ArtifactAttestations]);
        assert_eq!(report.failed_operations.len(), EXPECTED_FAILURE_COUNT);
        assert!(report.failed_operation.as_deref().is_some_and(|failure| failure.contains("first deletion failure")));
        assert!(report.failed_operations[1].contains("second deletion failure"));
    }

    // r[verify store_lifecycle.safe_gc_execution]
    // r[verify store_lifecycle.retention_validation]
    #[tokio::test]
    async fn stale_plan_is_rejected_after_root_change_without_deletion() {
        const KEEP_PATH_DIGEST_BYTE: u8 = 31;
        const CANDIDATE_PATH_DIGEST_BYTE: u8 = 32;
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let keep_path = store_path("keep", KEEP_PATH_DIGEST_BYTE);
        let candidate_path = store_path("candidate", CANDIDATE_PATH_DIGEST_BYTE);
        let keep_node = write_blob(&store, b"keep").await;
        let candidate_node = write_blob(&store, b"candidate").await;
        persist_output(&mut store, "out", keep_path, keep_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", candidate_path.clone(), candidate_node, vec![], true, None).await;

        let plan = store.garbage_collect(None).await.unwrap();
        assert_eq!(plan.candidate_paths, vec![candidate_path.to_absolute_path()]);
        store.pin_retained_root(&candidate_path.to_absolute_path()).await.unwrap();

        let error = store.garbage_collect(Some(&plan.plan_id)).await.expect_err("changed root must stale the plan");
        assert!(matches!(error, Error::Gc(message) if message.contains("stale-gc-plan")));
        assert!(output_dir.path().join(candidate_path.to_string()).exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stale_plan_is_rejected_after_export_symlink_substitution() {
        const KEEP_PATH_DIGEST_BYTE: u8 = 33;
        const CANDIDATE_PATH_DIGEST_BYTE: u8 = 34;
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let keep_path = store_path("keep-symlink-guard", KEEP_PATH_DIGEST_BYTE);
        let candidate_path = store_path("candidate-symlink-guard", CANDIDATE_PATH_DIGEST_BYTE);
        let keep_node = write_blob(&store, b"keep").await;
        let candidate_node = write_blob(&store, b"candidate").await;
        persist_output(&mut store, "out", keep_path.clone(), keep_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", candidate_path.clone(), candidate_node, vec![], true, None).await;
        let plan = store.garbage_collect(None).await.unwrap();
        let candidate_export = output_dir.path().join(candidate_path.to_string());
        let keep_export = output_dir.path().join(keep_path.to_string());
        std::fs::remove_file(&candidate_export).unwrap();
        std::os::unix::fs::symlink(&keep_export, &candidate_export).unwrap();

        let error = store.garbage_collect(Some(&plan.plan_id)).await.expect_err("symlink drift must stale the plan");
        assert!(matches!(error, Error::Gc(message) if message.contains("stale-gc-plan")));
        assert!(candidate_export.is_symlink());
        assert!(keep_export.exists());
    }

    #[tokio::test]
    async fn retained_root_keeps_transitive_closure_and_sidecars() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let dep_path = store_path("dep", 3);
        let root_path = store_path("root", 4);
        let dep_node = write_blob(&store, b"dep-bytes").await;
        let root_node = write_blob(&store, b"root-bytes").await;

        persist_output(&mut store, "out", dep_path.clone(), dep_node, vec![], false, None).await;
        persist_output(
            &mut store,
            "out",
            root_path.clone(),
            root_node,
            vec![dep_path.clone()],
            true,
            Some(GcRootSource::Build),
        )
        .await;
        let closure_path = crate::closure_attestation_file_path(
            state_dir.path(),
            "/nix/store",
            std::slice::from_ref(&root_path),
            crunch_attestation::ClosureSemantics::Runtime,
        );
        store.runtime_closure_attestation(std::slice::from_ref(&root_path)).await.unwrap();
        assert!(closure_path.exists());

        let report = store.garbage_collect(None).await.unwrap();
        assert_eq!(report.candidate_path_count, 0);
        assert!(closure_path.exists());
        assert!(crate::artifact_attestation_file_path(state_dir.path(), "/nix/store", &dep_path).exists());
        assert!(crate::artifact_attestation_file_path(state_dir.path(), "/nix/store", &root_path).exists());

        drop(store);
        let reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let listed = crate::store_list(reopened.pathinfo_service().as_ref()).await.unwrap();
        assert_eq!(listed.len(), 2);
    }

    #[tokio::test]
    async fn unreachable_output_removes_pathinfo_exports_and_attestations() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let root_path = store_path("root", 5);
        let drop_path = store_path("drop", 6);
        let root_node = write_blob(&store, b"root").await;
        let drop_node = write_blob(&store, b"drop").await;

        persist_output(&mut store, "out", root_path.clone(), root_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", drop_path.clone(), drop_node, vec![], true, None).await;
        let closure_path = crate::closure_attestation_file_path(
            state_dir.path(),
            "/nix/store",
            std::slice::from_ref(&drop_path),
            crunch_attestation::ClosureSemantics::Runtime,
        );
        store.runtime_closure_attestation(std::slice::from_ref(&drop_path)).await.unwrap();
        assert!(closure_path.exists());
        let export_path = output_dir.path().join(drop_path.to_string());
        assert!(export_path.exists());

        let plan = store.garbage_collect(None).await.unwrap();
        let report = store.garbage_collect(Some(&plan.plan_id)).await.unwrap();
        assert_eq!(report.candidate_paths, vec![drop_path.to_absolute_path()]);
        assert!(!export_path.exists());
        assert!(!crate::artifact_attestation_file_path(state_dir.path(), "/nix/store", &drop_path).exists());
        assert!(!closure_path.exists());
    }

    #[tokio::test]
    async fn shared_blob_survives_when_reachable_path_still_references_it() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let shared_node = write_blob(&store, b"shared").await;
        let shared_digest = match &shared_node {
            Node::File { digest, .. } => *digest,
            _ => panic!("expected file node"),
        };
        let root_path = store_path("root", 7);
        let drop_path = store_path("drop", 8);

        persist_output(
            &mut store,
            "out",
            root_path.clone(),
            shared_node.clone(),
            vec![],
            true,
            Some(GcRootSource::Build),
        )
        .await;
        persist_output(&mut store, "out", drop_path.clone(), shared_node, vec![], true, None).await;
        let chunk_path = blob_chunk_path(state_dir.path(), &shared_digest);
        assert!(chunk_path.exists());

        let plan = store.garbage_collect(None).await.unwrap();
        store.garbage_collect(Some(&plan.plan_id)).await.unwrap();

        assert!(chunk_path.exists());
    }

    #[tokio::test]
    async fn explicit_castore_root_survives_while_unreachable_blob_is_reclaimed() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let retained_node = write_blob(&store, b"retained-rust-result").await;
        let stale_node = write_blob(&store, b"stale-rust-result").await;
        let retained_digest = match &retained_node {
            Node::File { digest, .. } => *digest,
            _ => panic!("expected retained file node"),
        };
        let stale_digest = match &stale_node {
            Node::File { digest, .. } => *digest,
            _ => panic!("expected stale file node"),
        };
        let retained_path = blob_chunk_path(state_dir.path(), &retained_digest);
        let stale_path = blob_chunk_path(state_dir.path(), &stale_digest);
        assert!(retained_path.is_file());
        assert!(stale_path.is_file());

        let plan = store.garbage_collect_with_castore_roots(None, std::slice::from_ref(&retained_node)).await.unwrap();
        let report = store
            .garbage_collect_with_castore_roots(Some(&plan.plan_id), std::slice::from_ref(&retained_node))
            .await
            .unwrap();

        assert_eq!(report.retained_castore_root_count, 1);
        assert!(retained_path.is_file());
        assert!(!stale_path.exists());
    }

    #[tokio::test]
    async fn gc_aborts_when_retained_root_pathinfo_is_missing() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let root_path = store_path("root", 9);
        let root_node = write_blob(&store, b"root").await;
        persist_output(&mut store, "out", root_path.clone(), root_node, vec![], true, Some(GcRootSource::Build)).await;
        drop(store);
        std::fs::remove_file(state_dir.path().join("pathinfo.redb")).unwrap();

        let mut reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let err = reopened.garbage_collect(None).await.unwrap_err();
        assert!(matches!(err, Error::MissingClosureFacts { .. }));
    }

    #[tokio::test]
    async fn gc_aborts_when_root_registry_is_corrupt() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        std::fs::write(state_dir.path().join("gc-roots.json"), b"not json").unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;

        let err = store.garbage_collect(None).await.unwrap_err();
        assert!(matches!(err, Error::RootRegistry(_)));
    }

    #[tokio::test]
    async fn gc_operation_order_matches_design() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let root_path = store_path("root", 10);
        let drop_path = store_path("drop", 11);
        let root_node = write_blob(&store, b"root").await;
        let drop_node = write_blob(&store, b"drop").await;
        persist_output(&mut store, "out", root_path, root_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", drop_path.clone(), drop_node, vec![], true, None).await;

        let plan = store.garbage_collect(None).await.unwrap();
        let report = store.garbage_collect(Some(&plan.plan_id)).await.unwrap();

        assert_eq!(report.operations, vec![
            GcOperationKind::PathInfoRewrite,
            GcOperationKind::ExportedOutputs,
            GcOperationKind::ArtifactAttestations,
            GcOperationKind::ClosureAttestations,
            GcOperationKind::DirectoryRewrite,
            GcOperationKind::BlobIndexFiles,
            GcOperationKind::BlobChunkFiles,
            GcOperationKind::ActionResults,
            GcOperationKind::CaMappings,
        ]);
    }

    #[tokio::test]
    async fn directory_outputs_survive_reopen_and_gc() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let dir_path = store_path("dir-root", 12);
        let dir_node = write_directory_with_file(&store, "hello.txt", b"hello").await;
        persist_output(&mut store, "out", dir_path.clone(), dir_node, vec![], true, Some(GcRootSource::Build)).await;

        drop(store);
        let mut reopened = reopen_store(state_dir.path(), output_dir.path()).await;
        let report = reopened.garbage_collect(None).await.unwrap();
        assert_eq!(report.candidate_path_count, 0);
        assert!(output_dir.path().join(dir_path.to_string()).join("hello.txt").exists());
    }

    #[test]
    fn path_explanation_rejects_retaining_root_links_above_policy_limit() {
        const ROOT_LINK_LIMIT: usize = 1;
        let root_a = "/mantle/store/root-a".to_string();
        let root_b = "/mantle/store/root-b".to_string();
        let shared = "/mantle/store/shared".to_string();
        let plan = plan_gc(GcPlanRequest {
            roots: vec![root_a.clone(), root_b.clone()],
            entries: vec![
                GcEntry {
                    path_id: root_a,
                    references: vec![shared.clone()],
                    declared_nar_bytes: 1,
                    ownership: GcOwnership::Overlay,
                },
                GcEntry {
                    path_id: root_b,
                    references: vec![shared.clone()],
                    declared_nar_bytes: 1,
                    ownership: GcOwnership::Overlay,
                },
                GcEntry {
                    path_id: shared,
                    references: Vec::new(),
                    declared_nar_bytes: 1,
                    ownership: GcOwnership::Overlay,
                },
            ],
            execution_mode: GcExecutionMode::DryRun,
        })
        .expect("valid shared-closure plan");

        let error = build_path_explanations(&plan, ROOT_LINK_LIMIT)
            .expect_err("retaining-root links above the configured limit must fail");

        assert!(matches!(error, Error::Gc(message) if message.contains("exceeds policy limit")));
    }

    #[tokio::test]
    async fn pathinfo_rewrite_failure_stops_before_export_deletion() {
        const ROOT_PATH_DIGEST_BYTE: u8 = 12;
        const CANDIDATE_PATH_DIGEST_BYTE: u8 = 13;
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let root_path = store_path("rewrite-root", ROOT_PATH_DIGEST_BYTE);
        let candidate_path = store_path("rewrite-candidate", CANDIDATE_PATH_DIGEST_BYTE);
        let root_node = write_blob(&store, b"rewrite-root").await;
        let candidate_node = write_blob(&store, b"rewrite-candidate").await;
        persist_output(&mut store, "out", root_path, root_node, vec![], true, Some(GcRootSource::Build)).await;
        persist_output(&mut store, "out", candidate_path.clone(), candidate_node, vec![], true, None).await;
        let candidate_export = output_dir.path().join(candidate_path.to_string());
        let plan = store.garbage_collect(None).await.unwrap();
        let rewrite_blocker = state_dir.path().join("pathinfo.redb.gc-tmp");
        std::fs::create_dir(&rewrite_blocker).unwrap();

        let report = store.garbage_collect(Some(&plan.plan_id)).await.unwrap();

        assert!(!report.execution_complete);
        assert!(report.operations.is_empty());
        assert!(report.failed_operation.as_deref().is_some_and(|failure| failure.contains("PathInfoRewrite")));
        assert!(candidate_export.exists());
    }

    #[test]
    fn file_cleanup_reports_failure_after_attempting_later_independent_paths() {
        let cleanup_root = tempfile::tempdir().unwrap();
        let blocked_path = cleanup_root.path().join("blocked-directory");
        let removable_path = cleanup_root.path().join("removable-file");
        std::fs::create_dir(&blocked_path).unwrap();
        std::fs::write(&removable_path, b"remove-me").unwrap();

        let error = remove_files(&[blocked_path.clone(), removable_path.clone()])
            .expect_err("directory removal through the file rail must fail");

        assert!(matches!(error, Error::Gc(message) if message.contains("independent cleanup failure")));
        assert!(blocked_path.is_dir());
        assert!(!removable_path.exists());
    }

    #[test]
    fn reclaim_observation_preserves_unknown_bytes_and_plan_identity_binds_shape() {
        const FIRST_OBSERVED_BYTES: u64 = 7;
        const SECOND_OBSERVED_BYTES: u64 = 8;
        let observation_root = tempfile::tempdir().unwrap();
        let missing_root = observation_root.path().join("missing-reclaim-observation");
        assert!(matches!(path_size_bytes(&missing_root), Err(PathSizeBlocker::MetadataUnreadable)));
        let missing_paths = vec![missing_root.clone()];
        let empty_paths = Vec::new();
        let unknown_summary = compute_reclaimable_bytes(ReclaimObservationPaths {
            orphaned_on_disk: &missing_paths,
            artifact_attestations: &empty_paths,
            closure_attestations: &empty_paths,
            blob_indexes: &empty_paths,
            blob_chunks: &empty_paths,
            action_result_records: &empty_paths,
            action_result_indexes: &empty_paths,
        })
        .expect("unknown byte observations must remain reportable");
        assert_eq!(unknown_summary.reclaimable_bytes_total, 0);
        assert_eq!(unknown_summary.observations.len(), 1);
        assert!(unknown_summary.observations[0].bytes.is_none());
        assert_eq!(unknown_summary.observations[0].blocker.as_deref(), Some("metadata-unreadable"));

        const IDENTITY_OBSERVATION_PATH: &str = "/mantle/store/observed-candidate";
        let observations_a = vec![GcReclaimObservation {
            category: RECLAIM_CATEGORY_EXPORTED_OUTPUT.to_string(),
            path: IDENTITY_OBSERVATION_PATH.to_string(),
            path_kind: Some("file".to_string()),
            bytes: Some(FIRST_OBSERVED_BYTES),
            blocker: None,
        }];
        let observations_b = vec![GcReclaimObservation {
            category: RECLAIM_CATEGORY_EXPORTED_OUTPUT.to_string(),
            path: IDENTITY_OBSERVATION_PATH.to_string(),
            path_kind: Some("file".to_string()),
            bytes: Some(SECOND_OBSERVED_BYTES),
            blocker: None,
        }];
        let observations_c = vec![GcReclaimObservation {
            category: RECLAIM_CATEGORY_EXPORTED_OUTPUT.to_string(),
            path: IDENTITY_OBSERVATION_PATH.to_string(),
            path_kind: Some("symlink".to_string()),
            bytes: Some(FIRST_OBSERVED_BYTES),
            blocker: None,
        }];
        let core_plan_id = [0_u8; blake3::OUT_LEN];
        let retention_plan_id = [1_u8; blake3::OUT_LEN];
        let overlay_plan_id = [2_u8; blake3::OUT_LEN];
        let candidates = vec!["/mantle/store/candidate".to_string()];
        let plan_a = execution_plan_id(ExecutionPlanIdentityInput {
            core_plan_id: &core_plan_id,
            retention_plan_id: &retention_plan_id,
            overlay_plan_identity: None,
            candidate_paths: &candidates,
            reclaim_observations: &observations_a,
        });
        let plan_b = execution_plan_id(ExecutionPlanIdentityInput {
            core_plan_id: &core_plan_id,
            retention_plan_id: &retention_plan_id,
            overlay_plan_identity: None,
            candidate_paths: &candidates,
            reclaim_observations: &observations_b,
        });
        let plan_c = execution_plan_id(ExecutionPlanIdentityInput {
            core_plan_id: &core_plan_id,
            retention_plan_id: &retention_plan_id,
            overlay_plan_identity: None,
            candidate_paths: &candidates,
            reclaim_observations: &observations_c,
        });
        let plan_with_overlay = execution_plan_id(ExecutionPlanIdentityInput {
            core_plan_id: &core_plan_id,
            retention_plan_id: &retention_plan_id,
            overlay_plan_identity: Some(&overlay_plan_id),
            candidate_paths: &candidates,
            reclaim_observations: &observations_a,
        });

        const EXPECTED_PLAN_ID: &str = "b3:0548ff6cc95bec041ce51a773c67dd823983a1cfae5b7378adfb6ab940f5b6c2";
        assert_eq!(plan_a, EXPECTED_PLAN_ID);
        assert_ne!(plan_a, plan_b);
        assert_ne!(plan_a, plan_c);
        assert_ne!(plan_a, plan_with_overlay);
    }

    #[cfg(unix)]
    #[test]
    fn remove_path_removes_nested_read_only_export_tree() {
        use std::os::unix::fs::PermissionsExt;

        const READ_EXECUTE_MODE: u32 = 0o555;
        const READ_ONLY_MODE: u32 = 0o444;
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("read-only-output");
        let nested = root.join("share/data");
        std::fs::create_dir_all(&nested).unwrap();
        let payload = nested.join("payload.txt");
        std::fs::write(&payload, b"payload").unwrap();
        std::fs::set_permissions(&payload, std::fs::Permissions::from_mode(READ_ONLY_MODE)).unwrap();
        std::fs::set_permissions(&nested, std::fs::Permissions::from_mode(READ_EXECUTE_MODE)).unwrap();
        std::fs::set_permissions(nested.parent().unwrap(), std::fs::Permissions::from_mode(READ_EXECUTE_MODE)).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(READ_EXECUTE_MODE)).unwrap();

        remove_path(&root).unwrap();

        assert!(!root.exists());
        assert!(!payload.exists());
    }

    #[cfg(unix)]
    #[test]
    fn remove_path_does_not_follow_a_symlink_outside_the_export_tree() {
        use std::os::unix::fs::symlink;

        let parent = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let external_payload = external.path().join("keep.txt");
        std::fs::write(&external_payload, b"keep").unwrap();
        let root = parent.path().join("output-with-link");
        std::fs::create_dir(&root).unwrap();
        symlink(external.path(), root.join("external-link")).unwrap();

        remove_path(&root).unwrap();

        assert!(!root.exists());
        assert!(external_payload.exists());
    }

    #[test]
    fn remove_path_accepts_an_already_missing_export() {
        let parent = tempfile::tempdir().unwrap();
        let missing = parent.path().join("missing-output");
        assert!(!missing.exists());

        remove_path(&missing).unwrap();

        assert!(!missing.exists());
    }

    #[derive(Clone, Default)]
    struct SnapshotGuardPathInfoService {
        entries: Arc<std::sync::Mutex<Vec<PathInfo>>>,
        listing_active: Arc<std::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl pathinfoservice::PathInfoService for SnapshotGuardPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
            Ok(self.entries.lock().unwrap().iter().find(|entry| *entry.store_path.digest() == digest).cloned())
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
            if *self.listing_active.lock().unwrap() {
                return Err(std::io::Error::other("mutation while listing").into());
            }
            self.entries.lock().unwrap().push(path_info.clone());
            Ok(path_info)
        }

        fn list(&self) -> BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
            *self.listing_active.lock().unwrap() = true;
            let entries = self.entries.lock().unwrap().clone();
            let active = self.listing_active.clone();
            let (tx, rx) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                for entry in entries {
                    if tx.send(Ok(entry)).await.is_err() {
                        break;
                    }
                }
                *active.lock().unwrap() = false;
            });
            ReceiverStream::new(rx).boxed()
        }
    }

    #[tokio::test]
    async fn snapshot_pathinfos_finishes_before_later_mutation() {
        let service = SnapshotGuardPathInfoService::default();
        service.entries.lock().unwrap().push(signed_pathinfo(
            store_path("snap", 13),
            Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            vec![],
        ));

        let snapshot = snapshot_pathinfos(&service).await.unwrap();
        assert_eq!(snapshot.len(), 1);
        assert!(!*service.listing_active.lock().unwrap());
        service
            .put(signed_pathinfo(
                store_path("later", 14),
                Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                vec![],
            ))
            .await
            .unwrap();
    }
}
