// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in .cairn/changes/complete-store-capability-migration/evidence/
// tigerstyle-remaining-2026-09-09.log and scheduled for the standalone store-shell
// hardening pass. Scoped to the lint categories present at recording time.
#![allow(
    tigerstyle::assertion_density,
    tigerstyle::function_length,
    tigerstyle::numeric_units,
    tigerstyle::unbounded_collection_growth
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use casita::experimental::MetadataStore;
use casita::experimental::ObjectKey;
use casita::experimental::RootName;
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
use crunch_gc_core::retention::UsageObjectObservation;
use crunch_gc_core::retention::aggregate_usage;
use crunch_gc_core::retention::plan_retention;
use crunch_gc_core::summarize_reclaim_observations;
use data_encoding::HEXLOWER;
use futures::StreamExt;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
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
use crate::casita::CasitaStore;

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
    pub(crate) casita_store: Option<Arc<CasitaStore>>,
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
    CasitaRootRemoval,
    CasitaCollection,
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
    casita_targets: BTreeMap<String, (RootName, ObjectKey)>,
    casita_castore_candidates: Vec<String>,
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

const CASITA_GC_FENCE: &str = "casita-gc-fence.json";
const CASITA_GC_FENCE_JOURNAL: &str = "casita-gc-fence.progress";
const MAX_CASITA_GC_FENCE_ENTRIES: usize = 65_536;
const MAX_CASITA_GC_FENCE_BYTES: u64 = 64_u64.saturating_mul(1024).saturating_mul(1024);
const CASITA_GC_OUTCOME_BYTES: usize = 5;
const MAX_CASITA_GC_JOURNAL_BYTES: usize =
    MAX_CASITA_GC_FENCE_ENTRIES.saturating_mul(CASITA_GC_OUTCOME_BYTES).saturating_mul(4);

#[derive(Debug)]
struct CasitaCastorePlan {
    core_plan_id: [u8; blake3::OUT_LEN],
    candidate_roots: Vec<String>,
    targets: BTreeMap<String, (RootName, ObjectKey)>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum CasitaFenceKind {
    Output,
    Castore,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CasitaFenceEntry {
    kind: CasitaFenceKind,
    path: String,
    root_name: String,
    expected_target: String,
    removed: bool,
    cleaned: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CasitaGcFence {
    plan_id: String,
    entries: Vec<CasitaFenceEntry>,
}

/// Distinguishes physical exports from the logical store prefix during fenced cleanup.
#[derive(Clone, Copy)]
pub(crate) struct CasitaGcPaths<'a> {
    pub(crate) output_dir_str: &'a str,
    pub(crate) store_dir: &'a str,
}

// r[impl store_lifecycle.safe_gc_execution]
// r[impl mantle.casita_store_backend.plan_bound_gc]
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
    if ctx.casita_store.is_some() && ctx.retained_castore_roots.len() > crunch_gc_core::MAX_GC_ROOTS {
        return Err(Error::Gc("Casita GC castore retention exceeds the GC core root limit".to_string()));
    }

    let is_execution_requested = accepted_plan_id.is_some();
    let plan = build_plan(ctx, is_execution_requested).await?;
    if let Some(accepted_plan_id) = accepted_plan_id
        && accepted_plan_id != plan.plan_id
    {
        let blocker = if ctx.casita_store.is_some() {
            "gc-plan-stale"
        } else {
            "stale-gc-plan"
        };
        return Err(Error::Gc(format!("{blocker}: accepted={accepted_plan_id} observed={}", plan.plan_id)));
    }
    let core_decision = plan.core_decision;
    let live_paths: BTreeSet<String> = plan
        .live_pathinfos
        .iter()
        .map(|path_info| path_info.store_path.to_absolute_path_with_prefix(ctx.store_dir))
        .collect();
    let mut gc_result = GcReport {
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
        candidate_path_count: saturating_u32(plan.candidate_paths.len()),
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
    if let Some(store) = ctx.casita_store.as_ref() {
        execute_casita_gc(ctx, store, ca_mappings, &plan, &mut gc_result).await?;
        return Ok(gc_result);
    }

    let pathinfo_result = rewrite_pathinfo_db(ctx.state_dir, &plan.live_pathinfos).await;
    if !record_gc_operation(&mut gc_result, GcOperationKind::PathInfoRewrite, pathinfo_result) {
        return Ok(gc_result);
    }
    let _exported_outputs_succeeded = record_gc_operation(
        &mut gc_result,
        GcOperationKind::ExportedOutputs,
        remove_exported_outputs(&plan.orphaned_on_disk),
    );
    let _artifact_attestations_succeeded = record_gc_operation(
        &mut gc_result,
        GcOperationKind::ArtifactAttestations,
        remove_files(&plan.artifact_attestation_paths),
    );
    let _closure_attestations_succeeded = record_gc_operation(
        &mut gc_result,
        GcOperationKind::ClosureAttestations,
        remove_files(&plan.closure_attestation_paths),
    );
    let directory_result = rewrite_directory_db(ctx.state_dir, plan.live_castore.directories.values()).await;
    if !record_gc_operation(&mut gc_result, GcOperationKind::DirectoryRewrite, directory_result) {
        return Ok(gc_result);
    }
    let _blob_indexes_succeeded =
        record_gc_operation(&mut gc_result, GcOperationKind::BlobIndexFiles, remove_files(&plan.blob_index_paths));
    let _blob_chunks_succeeded =
        record_gc_operation(&mut gc_result, GcOperationKind::BlobChunkFiles, remove_files(&plan.blob_chunk_paths));
    let action_result = remove_action_result_files(&plan.action_result_record_paths, &plan.action_result_index_paths);
    let _action_results_succeeded = record_gc_operation(&mut gc_result, GcOperationKind::ActionResults, action_result);
    ca_mappings.retain_output_paths(&live_paths);
    let ca_mapping_result = ca_mappings
        .save_checked(ctx.state_dir)
        .map_err(|error| Error::Gc(format!("saving CA mappings: {error}")));
    let _ca_mappings_succeeded = record_gc_operation(&mut gc_result, GcOperationKind::CaMappings, ca_mapping_result);

    gc_result.execution_complete = gc_result.failed_operations.is_empty();
    Ok(gc_result)
}

// r[impl store_lifecycle.gc_explanation]
async fn build_plan(ctx: &GcContext<'_>, is_execution_requested: bool) -> Result<GcPlan, Error> {
    assert!(!ctx.store_dir.is_empty(), "build_plan: store_dir must not be empty");
    assert!(ctx.store_dir.starts_with('/'), "build_plan: store_dir must be absolute");

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
    let (overlay_snapshot, casita_targets, composed_snapshot) = if let Some(store) = ctx.casita_store.as_ref() {
        let records = store.verified_gc_roots(ctx.store_dir).await?;
        if records.len() > crunch_gc_core::MAX_GC_ENTRIES {
            return Err(Error::Gc("Casita GC snapshot exceeds the entry limit".to_string()));
        }
        let mut targets = BTreeMap::new();
        let mut pathinfos = Vec::with_capacity(records.len());
        let mut ownership = BTreeMap::new();
        for (name, target, pathinfo) in records {
            let path = pathinfo.store_path.to_absolute_path_with_prefix(ctx.store_dir);
            if targets.insert(path.clone(), (name, target)).is_some() {
                return Err(Error::Gc(format!("casita-envelope-invalid: duplicate output {path}")));
            }
            ownership.insert(path, GcOwnership::Overlay);
            pathinfos.push(pathinfo);
        }
        for root in &retained_roots {
            if !targets.contains_key(&root.logical_path) {
                return Err(Error::Gc(format!("casita-root-missing: {}", root.logical_path)));
            }
        }
        let composed = ComposedGcSnapshot {
            pathinfos: pathinfos.clone(),
            ownership,
        };
        (pathinfos, targets, composed)
    } else {
        let snapshot = snapshot_pathinfos(ctx.overlay_pathinfo).await?;
        let overlay_trusted_keys = if ctx.overlay_plan_identity.is_some() {
            Some(crate::overlay::load_layer_trust_keys(ctx.state_dir)?)
        } else {
            None
        };
        let composed = compose_gc_snapshot(
            &snapshot,
            &retained_roots,
            ctx.composed_pathinfo,
            ctx.store_dir,
            overlay_trusted_keys.as_deref(),
        )
        .await?;
        (snapshot, BTreeMap::new(), composed)
    };
    let core_plan =
        plan_gc(core_plan_request(&retained_roots, &composed_snapshot, ctx.store_dir, !is_execution_requested)?)
            .map_err(|error| {
                if ctx.casita_store.is_some()
                    && matches!(error, GcPlanError::MissingRoot { .. } | GcPlanError::MissingReference { .. })
                {
                    Error::Gc(format!("casita-root-missing: {error:?}"))
                } else {
                    shell_gc_plan_error(error, ctx.store_dir)
                }
            })?;
    let live_paths = core_plan.retained_path_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut usage = build_usage_report(
        &composed_snapshot.pathinfos,
        &core_plan.retaining_roots,
        &retention_plan.decisions,
        ctx.store_dir,
    )?;
    let explanation_link_limit =
        crate::retention::store_retention_runtime_policy().limits.max_closure_links_per_explanation;
    let path_explanations = build_path_explanations(&core_plan, explanation_link_limit)?;
    let base_reachability = build_base_reachability(&core_plan.retaining_roots, &composed_snapshot.ownership);
    let core_plan_id = core_plan.plan_id.into_bytes();
    let retention_plan_id_bytes = retention_plan.plan_id.into_bytes();
    let retention_plan_id = encode_blake3_identity(&retention_plan_id_bytes);
    let retention_explanations = build_retention_explanations(&retention_plan.decisions, &all_roots)?;
    let (live_pathinfos, dead_pathinfos) = split_pathinfos(overlay_snapshot, &live_paths, ctx.store_dir);
    let live_castore = if ctx.casita_store.is_some() {
        LiveCastoreState::default()
    } else {
        collect_live_castore_state(
            &live_pathinfos,
            ctx.retained_castore_roots,
            ctx.overlay_directory_service,
            ctx.overlay_blob_service,
        )
        .await?
    };
    let orphaned_on_disk = collect_existing_exported_outputs(&dead_pathinfos, ctx.output_dir_str)?;
    let artifact_attestation_paths =
        collect_existing_artifact_attestation_paths(ctx.state_dir, ctx.store_dir, &dead_pathinfos)?;
    let closure_attestation_paths = collect_dead_closure_attestations(ctx.state_dir, &retained_roots)?;
    let (blob_index_paths, blob_chunk_paths) = if ctx.casita_store.is_some() {
        (Vec::new(), Vec::new())
    } else {
        (
            collect_dead_blob_files(ctx.state_dir, &live_castore.blob_index_digests, true)?,
            collect_dead_blob_files(ctx.state_dir, &live_castore.chunk_digests, false)?,
        )
    };
    let action_result_gc = local_action_result_gc_candidates(ctx.state_dir, &live_paths)
        .map_err(|error| Error::Gc(format!("planning action-result metadata collection: {error}")))?;
    let mut reclaim_summary = compute_reclaimable_bytes(ReclaimObservationPaths {
        orphaned_on_disk: &orphaned_on_disk,
        artifact_attestations: &artifact_attestation_paths,
        closure_attestations: &closure_attestation_paths,
        blob_indexes: &blob_index_paths,
        blob_chunks: &blob_chunk_paths,
        action_result_records: &action_result_gc.record_paths,
        action_result_indexes: &action_result_gc.index_marker_paths,
    })?;
    let mut candidate_paths = core_plan.candidate_path_ids.clone();
    let (casita_castore_candidates, casita_targets, castore_core_id) = if let Some(store) = ctx.casita_store.as_ref() {
        let castore = plan_casita_castore_roots(store, ctx.retained_castore_roots, is_execution_requested).await?;
        candidate_paths.extend(castore.candidate_roots.iter().cloned());
        usage.unknown_object_count = usage.unknown_object_count.saturating_add(castore.targets.len());
        let mut targets = casita_targets;
        for (id, root) in castore.targets {
            if targets.insert(id.clone(), root).is_some() {
                return Err(Error::Gc(format!("duplicate Casita GC target identity: {id}")));
            }
        }
        (castore.candidate_roots, targets, Some(castore.core_plan_id))
    } else {
        (Vec::new(), casita_targets, None)
    };
    if ctx.casita_store.is_some() && candidate_paths.len() > MAX_CASITA_GC_FENCE_ENTRIES {
        return Err(Error::Gc(format!(
            "Casita GC candidate count exceeds the bounded fence limit of {MAX_CASITA_GC_FENCE_ENTRIES}",
        )));
    }
    if ctx.casita_store.is_some() {
        reclaim_summary.observations.extend(candidate_paths.iter().map(|path| GcReclaimObservation {
            category: "casita-root-graph".to_string(),
            path: path.clone(),
            path_kind: None,
            bytes: None,
            blocker: Some("casita-physical-size-not-observed".to_string()),
        }));
    }
    let mut plan_id = execution_plan_id(ExecutionPlanIdentityInput {
        core_plan_id: &core_plan_id,
        retention_plan_id: &retention_plan_id_bytes,
        overlay_plan_identity: ctx.overlay_plan_identity.as_ref(),
        candidate_paths: &candidate_paths,
        reclaim_observations: &reclaim_summary.observations,
    });
    if let Some(castore_core_id) = castore_core_id.as_ref() {
        plan_id = casita_execution_plan_id(&plan_id, &casita_targets, castore_core_id);
    }
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
        live_pathinfos,
        dead_pathinfos,
        live_castore,
        orphaned_on_disk,
        artifact_attestation_paths,
        closure_attestation_paths,
        blob_index_paths,
        blob_chunk_paths,
        casita_targets,
        casita_castore_candidates,
        action_result_record_paths: action_result_gc.record_paths,
        action_result_index_paths: action_result_gc.index_marker_paths,
        candidate_paths,
        reclaimable_bytes_total: reclaim_summary.reclaimable_bytes_total,
        reclaim_observations: reclaim_summary.observations,
    })
}

async fn plan_casita_castore_roots(
    store: &CasitaStore,
    retained_nodes: &[Node],
    is_execution_requested: bool,
) -> Result<CasitaCastorePlan, Error> {
    let observed = store.verified_gc_castore_roots().await?;
    let roots = retained_nodes
        .iter()
        .map(|node| CasitaStore::castore_root_name(node).map(|name| casita_castore_path_id(&name)))
        .collect::<Result<Vec<_>, _>>()?;
    plan_casita_castore_facts(roots, observed, is_execution_requested)
}

fn casita_castore_path_id(name: &RootName) -> String {
    format!("/{name}")
}

fn plan_casita_castore_facts(
    roots: Vec<String>,
    observed: Vec<(RootName, ObjectKey, Node, u64)>,
    execute: bool,
) -> Result<CasitaCastorePlan, Error> {
    if observed.len() > crunch_gc_core::MAX_GC_ENTRIES {
        return Err(Error::Gc("Casita castore GC snapshot exceeds the entry limit".to_string()));
    }
    let mut entries = Vec::with_capacity(observed.len());
    let mut targets = BTreeMap::new();
    for (name, target, _, nar_bytes) in observed {
        let id = casita_castore_path_id(&name);
        entries.push(GcEntry {
            path_id: id.clone(),
            references: Vec::new(),
            declared_nar_bytes: nar_bytes,
            ownership: GcOwnership::Overlay,
        });
        if targets.insert(id.clone(), (name, target)).is_some() {
            return Err(Error::Gc(format!("casita-envelope-invalid: duplicate castore root {id}")));
        }
    }
    let core_plan = plan_gc(GcPlanRequest {
        roots,
        entries,
        execution_mode: if execute {
            GcExecutionMode::Execute
        } else {
            GcExecutionMode::DryRun
        },
    })
    .map_err(|error| match error {
        GcPlanError::MissingRoot { .. } => Error::Gc(format!("casita-root-missing: {error:?}")),
        _ => Error::Gc(format!("planning Casita castore reachability: {error:?}")),
    })?;
    Ok(CasitaCastorePlan {
        core_plan_id: core_plan.plan_id.into_bytes(),
        candidate_roots: core_plan.candidate_path_ids,
        targets,
    })
}

fn casita_execution_plan_id(
    base_id: &str,
    targets: &BTreeMap<String, (RootName, ObjectKey)>,
    castore_core_id: &[u8; blake3::OUT_LEN],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle.gc.casita-targets.v2");
    hasher.update(castore_core_id);
    hash_plan_string(&mut hasher, base_id);
    hasher.update(&(targets.len() as u128).to_be_bytes());
    for (path, (name, target)) in targets {
        hash_plan_string(&mut hasher, path);
        hash_plan_string(&mut hasher, name.as_ref());
        hash_plan_string(&mut hasher, &target.to_string());
    }
    encode_blake3_identity(hasher.finalize().as_bytes())
}

fn casita_fence_path(state_dir: &Path) -> PathBuf {
    state_dir.join(CASITA_GC_FENCE)
}
fn casita_fence_journal_path(state_dir: &Path) -> PathBuf {
    state_dir.join(CASITA_GC_FENCE_JOURNAL)
}

pub(crate) fn casita_gc_fence_pending(state_dir: &Path) -> Result<bool, Error> {
    let manifest = casita_fence_path(state_dir);
    match std::fs::symlink_metadata(&manifest) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(Error::Gc(format!("Casita GC fence is not a regular file: {}", manifest.display()))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            match std::fs::symlink_metadata(casita_fence_journal_path(state_dir)) {
                Ok(_) => Err(Error::Gc("orphan Casita GC outcome journal requires inspection".to_string())),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(error) => Err(Error::Gc(format!("reading Casita GC outcome journal: {error}"))),
            }
        }
        Err(error) => Err(Error::Gc(format!("reading Casita GC fence: {error}"))),
    }
}
// Append-only outcomes avoid rewriting an O(n) fence for every root removal.
fn append_casita_fence_outcome(state_dir: &Path, index: usize, cleaned: bool) -> Result<(), Error> {
    let index = u32::try_from(index).map_err(|_| Error::Gc("Casita GC fence index exceeds u32".to_string()))?;
    let mut record = [0_u8; CASITA_GC_OUTCOME_BYTES];
    record[0] = if cleaned { 2 } else { 1 };
    record[1..].copy_from_slice(&index.to_be_bytes());
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(casita_fence_journal_path(state_dir))
        .map_err(|error| Error::Gc(format!("opening Casita GC outcome journal: {error}")))?;
    file.write_all(&record).map_err(|error| Error::Gc(format!("writing Casita GC outcome: {error}")))?;
    file.sync_all().map_err(|error| Error::Gc(format!("syncing Casita GC outcome: {error}")))?;
    std::fs::File::open(state_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| Error::Gc(format!("syncing Casita GC outcome directory: {error}")))
}

fn persist_casita_fence(state_dir: &Path, fence: &CasitaGcFence) -> Result<(), Error> {
    let path = casita_fence_path(state_dir);
    let bytes = serde_json::to_vec(fence).map_err(|error| Error::Gc(format!("encoding Casita GC fence: {error}")))?;
    if bytes.len() as u64 > MAX_CASITA_GC_FENCE_BYTES || fence.entries.len() > MAX_CASITA_GC_FENCE_ENTRIES {
        return Err(Error::Gc("Casita GC fence exceeds the bounded persistence limit".to_string()));
    }
    if casita_fence_journal_path(state_dir).exists() {
        return Err(Error::Gc("orphan Casita GC outcome journal requires recovery".to_string()));
    }
    let mut temp = tempfile::NamedTempFile::new_in(state_dir)
        .map_err(|error| Error::Gc(format!("creating Casita GC fence: {error}")))?;
    temp.write_all(&bytes).map_err(|error| Error::Gc(format!("writing Casita GC fence: {error}")))?;
    temp.as_file().sync_all().map_err(|error| Error::Gc(format!("syncing Casita GC fence: {error}")))?;
    temp.persist(&path).map_err(|error| Error::Gc(format!("publishing Casita GC fence: {error}")))?;
    std::fs::File::open(state_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| Error::Gc(format!("syncing Casita GC fence directory: {error}")))?;
    Ok(())
}

fn load_casita_fence(state_dir: &Path) -> Result<Option<CasitaGcFence>, Error> {
    if !casita_gc_fence_pending(state_dir)? {
        return Ok(None);
    }
    let path = casita_fence_path(state_dir);
    let size = std::fs::metadata(&path)
        .map_err(|error| Error::Gc(format!("reading Casita GC fence size: {error}")))?
        .len();
    if size > MAX_CASITA_GC_FENCE_BYTES {
        return Err(Error::Gc("Casita GC fence exceeds the bounded persistence limit".to_string()));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(&path)
        .and_then(|file| file.take(MAX_CASITA_GC_FENCE_BYTES.saturating_add(1)).read_to_end(&mut bytes))
        .map_err(|error| Error::Gc(format!("reading Casita GC fence: {error}")))?;
    if bytes.len() as u64 > MAX_CASITA_GC_FENCE_BYTES {
        return Err(Error::Gc("Casita GC fence exceeds the bounded persistence limit".to_string()));
    }
    let mut fence: CasitaGcFence =
        serde_json::from_slice(&bytes).map_err(|error| Error::Gc(format!("decoding Casita GC fence: {error}")))?;
    if fence.entries.len() > MAX_CASITA_GC_FENCE_ENTRIES {
        return Err(Error::Gc("Casita GC fence exceeds the entry limit".to_string()));
    }
    let journal_path = casita_fence_journal_path(state_dir);
    let journal_metadata = match std::fs::symlink_metadata(&journal_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Some(fence)),
        Err(error) => return Err(Error::Gc(format!("reading Casita GC outcome journal: {error}"))),
    };
    if !journal_metadata.is_file() {
        return Err(Error::Gc("Casita GC outcome journal is not a regular file".to_string()));
    }
    if journal_metadata.len() > MAX_CASITA_GC_JOURNAL_BYTES as u64 {
        return Err(Error::Gc("Casita GC outcome journal exceeds the persistence limit".to_string()));
    }
    let mut journal = Vec::new();
    std::fs::File::open(&journal_path)
        .and_then(|file| file.take((MAX_CASITA_GC_JOURNAL_BYTES as u64).saturating_add(1)).read_to_end(&mut journal))
        .map_err(|error| Error::Gc(format!("reading Casita GC outcome journal: {error}")))?;
    if journal.len() > MAX_CASITA_GC_JOURNAL_BYTES {
        return Err(Error::Gc("Casita GC outcome journal exceeds the persistence limit".to_string()));
    }
    for record in journal.chunks_exact(CASITA_GC_OUTCOME_BYTES) {
        let encoded_index: [u8; 4] =
            record[1..].try_into().map_err(|_| Error::Gc("invalid Casita GC outcome index width".to_string()))?;
        let index = usize::try_from(u32::from_be_bytes(encoded_index))
            .map_err(|_| Error::Gc("Casita GC outcome index exceeds platform size".to_string()))?;
        let entry = fence
            .entries
            .get_mut(index)
            .ok_or_else(|| Error::Gc(format!("Casita GC outcome index out of bounds: {index}")))?;
        match record[0] {
            1 => entry.removed = true,
            2 => {
                entry.removed = true;
                entry.cleaned = true;
            }
            _ => return Err(Error::Gc("invalid Casita GC outcome status".to_string())),
        }
    }
    let valid_len = journal.len().saturating_sub(journal.len() % CASITA_GC_OUTCOME_BYTES);
    if valid_len != journal.len() {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&journal_path)
            .map_err(|error| Error::Gc(format!("opening partial Casita GC outcome: {error}")))?;
        file.set_len(valid_len as u64)
            .and_then(|()| file.sync_all())
            .map_err(|error| Error::Gc(format!("truncating partial Casita GC outcome: {error}")))?;
    }
    Ok(Some(fence))
}

fn finish_casita_fence(state_dir: &Path) -> Result<(), Error> {
    match std::fs::remove_file(casita_fence_journal_path(state_dir)) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(Error::Gc(format!("removing Casita GC outcome journal: {error}"))),
    }
    std::fs::remove_file(casita_fence_path(state_dir))
        .map_err(|error| Error::Gc(format!("removing Casita GC fence: {error}")))?;
    std::fs::File::open(state_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| Error::Gc(format!("syncing Casita GC fence removal: {error}")))?;
    Ok(())
}

fn fence_entry_identity(
    entry: &CasitaFenceEntry,
    store_dir: &str,
) -> Result<(Option<StorePath<String>>, RootName, ObjectKey), Error> {
    let (path, name) = match entry.kind {
        CasitaFenceKind::Output => {
            let path = StorePath::from_absolute_path_with_prefix(entry.path.as_bytes(), store_dir)
                .map_err(|error| Error::Gc(format!("invalid Casita GC fenced path: {error}")))?;
            let name = CasitaStore::root_name(&path)?;
            (Some(path), name)
        }
        CasitaFenceKind::Castore => {
            let Some(digest) = entry.path.strip_prefix("/mantle/castore/") else {
                return Err(Error::Gc("invalid Casita GC fenced castore root".to_string()));
            };
            if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err(Error::Gc("invalid Casita GC fenced castore digest".to_string()));
            }
            let name = RootName::try_from(entry.path.trim_start_matches('/'))
                .map_err(|error| Error::Gc(format!("invalid Casita GC fenced root: {error}")))?;
            (None, name)
        }
    };
    if entry.root_name != name.as_ref() {
        return Err(Error::Gc(format!("Casita GC fence root name differs from {}", entry.path)));
    }
    let target = entry
        .expected_target
        .parse::<ObjectKey>()
        .map_err(|error| Error::Gc(format!("invalid Casita GC fenced target: {error}")))?;
    Ok((path, name, target))
}

fn fenced_root_was_removed(entry: &CasitaFenceEntry, observed: Option<&ObjectKey>) -> Result<bool, Error> {
    let expected = entry
        .expected_target
        .parse::<ObjectKey>()
        .map_err(|error| Error::Gc(format!("invalid Casita GC fenced target: {error}")))?;
    match observed {
        Some(actual) if actual == &expected => Ok(false),
        Some(_) => Err(Error::Gc(format!("casita-root-conflict: {}", entry.path))),
        None => Ok(true),
    }
}

fn cleanup_casita_removed(state_dir: &Path, paths: CasitaGcPaths<'_>, entry: &CasitaFenceEntry) -> Result<(), Error> {
    let (path, _, _) = fence_entry_identity(entry, paths.store_dir)?;
    if let Some(path) = path {
        remove_path(&PathBuf::from(path.to_absolute_path_with_prefix(paths.output_dir_str)))?;
        remove_path(&artifact_attestation_file_path(state_dir, paths.store_dir, &path))?;
    }
    Ok(())
}

fn mark_casita_gc_failure(report: &mut GcReport, message: String) {
    if report.failed_operation.is_none() {
        report.failed_operation = Some(message.clone());
    }
    report.failed_operations.push(message);
}

async fn execute_casita_gc(
    ctx: &GcContext<'_>,
    store: &CasitaStore,
    ca_mappings: &mut CaMappings,
    plan: &GcPlan,
    report: &mut GcReport,
) -> Result<(), Error> {
    if load_casita_fence(ctx.state_dir)?.is_some() {
        return Err(Error::Gc("Casita GC fence requires recovery before a new execution".to_string()));
    }
    let observed = build_plan(ctx, true).await?;
    if observed.plan_id != plan.plan_id {
        return Err(Error::Gc(format!("gc-plan-stale: accepted={} observed={}", plan.plan_id, observed.plan_id)));
    }
    let entries = plan
        .candidate_paths
        .iter()
        .map(|path| {
            let (name, target) = plan.casita_targets.get(path).ok_or_else(|| {
                Error::Gc(format!("casita-root-missing: planned GC candidate has no verified root: {path}"))
            })?;
            Ok(CasitaFenceEntry {
                kind: if plan.casita_castore_candidates.binary_search(path).is_ok() {
                    CasitaFenceKind::Castore
                } else {
                    CasitaFenceKind::Output
                },
                path: path.clone(),
                root_name: name.to_string(),
                expected_target: target.to_string(),
                removed: false,
                cleaned: false,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let mut fence = CasitaGcFence {
        plan_id: plan.plan_id.clone(),
        entries,
    };
    persist_casita_fence(ctx.state_dir, &fence)?;
    for index in 0..fence.entries.len() {
        let (_, name, target) = fence_entry_identity(&fence.entries[index], ctx.store_dir)?;
        match store.repository.remove_root_if_matches(&name, &target).await {
            Ok(Some(_)) => {
                fence.entries[index].removed = true;
                append_casita_fence_outcome(ctx.state_dir, index, false)?;
                report.operations.push(GcOperationKind::CasitaRootRemoval);
                if let Err(error) = cleanup_casita_removed(
                    ctx.state_dir,
                    CasitaGcPaths {
                        output_dir_str: ctx.output_dir_str,
                        store_dir: ctx.store_dir,
                    },
                    &fence.entries[index],
                ) {
                    mark_casita_gc_failure(report, format!("ExportedOutputs/ArtifactAttestations: {error}"));
                } else {
                    fence.entries[index].cleaned = true;
                    append_casita_fence_outcome(ctx.state_dir, index, true)?;
                    if fence.entries[index].kind == CasitaFenceKind::Output {
                        report
                            .operations
                            .extend([GcOperationKind::ExportedOutputs, GcOperationKind::ArtifactAttestations]);
                    }
                }
            }
            Ok(None) => {
                mark_casita_gc_failure(report, format!("casita-root-conflict: {}", fence.entries[index].path));
                break;
            }
            Err(error) => {
                mark_casita_gc_failure(report, format!("Casita root removal: {error}"));
                break;
            }
        }
    }
    if fence.entries.iter().any(|entry| entry.removed && !entry.cleaned) {
        return Ok(());
    }
    let removed = fence
        .entries
        .iter()
        .filter(|entry| entry.removed && entry.kind == CasitaFenceKind::Output)
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();
    if let Err(error) = cleanup_casita_indexes(ctx.state_dir, store, ctx.store_dir, ca_mappings, &removed).await {
        mark_casita_gc_failure(report, format!("ActionResults/CaMappings: {error}"));
        return Ok(());
    }
    report.operations.extend([GcOperationKind::ActionResults, GcOperationKind::CaMappings]);
    if !report.failed_operations.is_empty() {
        return Ok(());
    }
    if let Err(error) = remove_files(&plan.closure_attestation_paths) {
        mark_casita_gc_failure(report, format!("ClosureAttestations: {error}"));
        return Ok(());
    }
    report.operations.push(GcOperationKind::ClosureAttestations);
    match store.repository.try_collect().await {
        Ok(_) => {
            finish_casita_fence(ctx.state_dir)?;
            report.operations.push(GcOperationKind::CasitaCollection);
            report.execution_complete = true;
        }
        Err(error) => mark_casita_gc_failure(report, format!("Casita collection/reclaim-incomplete: {error}")),
    }
    Ok(())
}

async fn cleanup_casita_indexes(
    state_dir: &Path,
    store: &CasitaStore,
    store_dir: &str,
    ca_mappings: &mut CaMappings,
    removed: &BTreeSet<String>,
) -> Result<(), Error> {
    let published = store
        .verified_gc_roots(store_dir)
        .await?
        .into_iter()
        .map(|(_, _, info)| info.store_path.to_absolute_path_with_prefix(store_dir))
        .collect::<BTreeSet<_>>();
    if !published.is_disjoint(removed) {
        return Err(Error::Gc("casita-root-conflict: removed root was republished before index cleanup".to_string()));
    }
    let action = local_action_result_gc_candidates(state_dir, &published)
        .map_err(|error| Error::Gc(format!("planning Casita action-result cleanup: {error}")))?;
    remove_action_result_files(&action.record_paths, &action.index_marker_paths)?;
    ca_mappings.retain_output_paths(&published);
    ca_mappings
        .save_checked(state_dir)
        .map_err(|error| Error::Gc(format!("saving Casita CA mappings: {error}")))
}

fn cleanup_casita_closure_attestations(state_dir: &Path) -> Result<(), Error> {
    let all_roots = roots::list_roots(state_dir)?;
    let retention_facts = crate::retention::records_to_core(&all_roots)?;
    let retention_plan =
        plan_retention(&crate::retention::core_retention_policy(), roots::current_unix_seconds()?, retention_facts)
            .map_err(|error| Error::Gc(format!("planning Casita recovery retention: {error:?}")))?;
    let retained = retention_plan
        .decisions
        .iter()
        .filter(|decision| decision.disposition.retains_path())
        .map(|decision| decision.path_id.as_str())
        .collect::<BTreeSet<_>>();
    let retained_roots = all_roots
        .into_iter()
        .filter(|root| retained.contains(root.logical_path.as_str()))
        .collect::<Vec<_>>();
    let dead = collect_dead_closure_attestations(state_dir, &retained_roots)?;
    remove_files(&dead)
}

// Recovery has no root-removal operation. A still-published fenced candidate remains published.
pub(crate) async fn recover_casita_gc(
    state_dir: &Path,
    paths: CasitaGcPaths<'_>,
    store: &CasitaStore,
    ca_mappings: &mut CaMappings,
) -> Result<(), Error> {
    let Some(mut fence) = load_casita_fence(state_dir)? else {
        return Ok(());
    };
    let snapshot = store
        .repository
        .metadata()
        .snapshot()
        .await
        .map_err(|error| Error::Gc(format!("reading Casita GC recovery roots: {error}")))?;
    let mut removed = BTreeSet::new();
    let mut root_observations = Vec::with_capacity(fence.entries.len());
    for index in 0..fence.entries.len() {
        let (path, name, _) = fence_entry_identity(&fence.entries[index], paths.store_dir)?;
        let observed =
            snapshot.root(&name).await.map_err(|error| Error::Gc(format!("reading fenced root: {error}")))?;
        root_observations.push((name, observed.clone()));
        if fenced_root_was_removed(&fence.entries[index], observed.as_ref())? {
            if let Some(path) = path {
                removed.insert(path.to_absolute_path_with_prefix(paths.store_dir));
            }
            if !fence.entries[index].cleaned {
                cleanup_casita_removed(state_dir, paths, &fence.entries[index])?;
                fence.entries[index].removed = true;
                fence.entries[index].cleaned = true;
                append_casita_fence_outcome(state_dir, index, true)?;
            }
        }
    }
    if !removed.is_empty() {
        cleanup_casita_indexes(state_dir, store, paths.store_dir, ca_mappings, &removed).await?;
        cleanup_casita_closure_attestations(state_dir)?;
    }
    let latest = store
        .repository
        .metadata()
        .snapshot()
        .await
        .map_err(|error| Error::Gc(format!("rechecking Casita GC recovery roots: {error}")))?;
    for (name, expected) in &root_observations {
        let actual = latest
            .root(name)
            .await
            .map_err(|error| Error::Gc(format!("rechecking fenced root {name}: {error}")))?;
        if &actual != expected {
            return Err(Error::Gc(format!("casita-root-conflict: fenced root {name} changed during recovery")));
        }
    }
    drop(latest);
    drop(snapshot);
    store
        .repository
        .try_collect()
        .await
        .map_err(|error| Error::Gc(format!("Casita GC reclaim-incomplete: {error}")))?;
    finish_casita_fence(state_dir)
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
    remove_file_iter(record_paths.iter().chain(index_paths))
}

fn build_retention_explanations(
    decisions: &[RetentionDecision],
    roots: &[GcRootRecord],
) -> Result<Vec<GcRetentionExplanation>, Error> {
    let roots_by_path = roots.iter().map(|root| (root.logical_path.as_str(), root)).collect::<BTreeMap<_, _>>();
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
    let report = aggregate_usage(decisions, observations)
        .map_err(|error| Error::Gc(format!("aggregating store usage: {error:?}")))?;
    Ok(GcUsageReport {
        observed_bytes: report.observed_bytes,
        retained_bytes: report.retained_bytes,
        reclaimable_bytes: report.reclaimable_bytes,
        quarantined_bytes: report.quarantined_bytes,
        unclassified_bytes: report.unclassified_bytes,
        shared_bytes: report.shared_bytes,
        unknown_object_count: report.unknown_object_count,
        roots: report
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

async fn compose_gc_snapshot(
    overlay_snapshot: &[PathInfo],
    retained_roots: &[GcRootRecord],
    composed_pathinfo: &dyn PathInfoService,
    store_dir: &str,
    overlay_trusted_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
) -> Result<ComposedGcSnapshot, Error> {
    let mut pathinfos_by_id = BTreeMap::new();
    let mut ownership = BTreeMap::new();
    let mut queue = Vec::new();
    for path_info in overlay_snapshot {
        if let Some(trusted_keys) = overlay_trusted_keys {
            crate::overlay::verify_pathinfo_trust(path_info, trusted_keys)
                .map_err(|error| Error::Gc(format!("overlay-gc-untrusted-layer: {error}")))?;
        }
        let path_id = path_info.store_path.to_absolute_path_with_prefix(store_dir);
        if pathinfos_by_id.insert(path_id.clone(), path_info.clone()).is_some()
            || ownership.insert(path_id.clone(), GcOwnership::Overlay).is_some()
        {
            return Err(Error::Gc(format!("duplicate overlay PathInfo during GC: {path_id}")));
        }
        queue.push(path_id);
    }
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
            let store_path: StorePath<String> =
                StorePath::from_absolute_path_with_prefix(path_id.as_bytes(), store_dir)
                    .map_err(|error| Error::Gc(format!("parsing composed GC path {path_id}: {error}")))?;
            let read = composed_pathinfo
                .get_with_layer(*store_path.digest())
                .await
                .map_err(|error| Error::Gc(format!("resolving composed GC path {path_id}: {error}")))?
                .ok_or_else(|| Error::MissingClosureFacts {
                    path: store_path.clone(),
                    store_dir: store_dir.to_string(),
                    detail: "composed GC path has no PathInfo".to_string(),
                })?;
            if read.layer_index == 0 {
                return Err(Error::Gc(format!(
                    "overlay PathInfo is readable but absent from overlay GC listing: {path_id}"
                )));
            }
            let observed_path = read.value.store_path.to_absolute_path_with_prefix(store_dir);
            if observed_path != path_id {
                return Err(Error::Gc(format!(
                    "composed GC digest collision: requested {path_id}, observed {observed_path}"
                )));
            }
            if pathinfos_by_id.len() >= crunch_gc_core::MAX_GC_ENTRIES {
                return Err(Error::Gc(format!(
                    "composed GC snapshot exceeds {} PathInfos",
                    crunch_gc_core::MAX_GC_ENTRIES
                )));
            }
            ownership.insert(path_id.clone(), GcOwnership::Base {
                layer_index: read.layer_index,
            });
            pathinfos_by_id.insert(path_id.clone(), read.value);
        }
        let path_info = pathinfos_by_id
            .get(&path_id)
            .ok_or_else(|| Error::Gc(format!("composed GC snapshot lost PathInfo {path_id}")))?;
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
    // Reclaim observations enter the plan hash in this order, not as a set.
    canonicalize_dead_blob_paths(&mut dead_paths);
    Ok(dead_paths)
}

fn canonicalize_dead_blob_paths(dead_paths: &mut [PathBuf]) {
    dead_paths.sort_unstable();
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
    Ok(PathSizeObservation {
        bytes: total,
        path_kind,
    })
}

fn remove_exported_outputs(paths: &[PathBuf]) -> Result<(), Error> {
    let mut failures = Vec::new();
    for path in paths {
        if let Err(error) = remove_path(path) {
            failures.push(error.to_string());
        }
    }
    cleanup_result(failures)
}

fn remove_files(paths: &[PathBuf]) -> Result<(), Error> {
    remove_file_iter(paths.iter())
}

fn remove_file_iter<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> Result<(), Error> {
    let mut failures = Vec::new();
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
            backend: crate::StoreBackend::Snix,
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

    fn casita_gc_trusted_signer(state_dir: &Path) -> nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey> {
        use nix_compat::narinfo::SigningKey;
        use nix_compat::narinfo::VerifyingKey;

        let raw = ed25519_dalek::SigningKey::from_bytes(&[23_u8; 32]);
        let verifying = VerifyingKey::new("casita-gc-output-fixture".to_string(), raw.verifying_key());
        std::fs::write(state_dir.join("casita-trusted-public-keys"), format!("{verifying}\n")).unwrap();
        SigningKey::new("casita-gc-output-fixture".to_string(), raw)
    }

    async fn casita_gc_signed_symlink_output(
        store: &StoreHandle,
        name: &str,
        target: &str,
        signer: &nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey>,
    ) -> PathInfo {
        use nix_compat::narinfo::fingerprint_with_store_dir;
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;
        use nix_compat::store_path::build_ca_path_with_store_dir;
        use snix_store::nar::NarCalculationService;
        use snix_store::nar::SimpleRenderer;

        let node = Node::Symlink {
            target: SymlinkTarget::try_from(target).unwrap(),
        };
        let (nar_size, nar_sha256) = SimpleRenderer::new(store.blob_service(), store.directory_service())
            .calculate_nar(&node)
            .await
            .unwrap();
        let ca = CAHash::Nar(NixHash::Sha256(nar_sha256));
        let store_path = build_ca_path_with_store_dir(name, &ca, Vec::<String>::new(), false, "/nix/store").unwrap();
        let mut info = PathInfo {
            store_path,
            node,
            references: Vec::new(),
            nar_size,
            nar_sha256,
            signatures: Vec::new(),
            deriver: None,
            ca: Some(ca),
        };
        let fingerprint = fingerprint_with_store_dir(
            &info.store_path.as_ref(),
            &nar_sha256,
            nar_size,
            std::iter::empty::<&nix_compat::store_path::StorePathRef>(),
            "/nix/store",
        );
        info.signatures.push(signer.sign(fingerprint.as_bytes()).to_owned());
        info
    }

    fn casita_gc_action_result(
        seed: &str,
        output: &StorePath<String>,
    ) -> crunch_action_result_core::SignedActionResultRecord {
        use crunch_action_result_core::ACTION_RECEIPT_REF_PREFIX;
        use crunch_action_result_core::ACTION_REF_PREFIX;
        use crunch_action_result_core::ActionResultOutput;
        use crunch_action_result_core::ActionResultRecordInput;
        use crunch_action_result_core::DetachedRecordSignature;
        use crunch_action_result_core::NETWORK_POLICY_REF_PREFIX;
        use crunch_action_result_core::OBJECT_REF_PREFIX;
        use crunch_action_result_core::PATH_INFO_REF_PREFIX;
        use crunch_action_result_core::PRODUCER_POLICY_REF_PREFIX;
        use crunch_action_result_core::PUBLICATION_POLICY_REF_PREFIX;
        use crunch_action_result_core::REFERENCE_SCAN_REF_PREFIX;
        use crunch_action_result_core::SANDBOX_POLICY_REF_PREFIX;
        use crunch_action_result_core::SIGNATURE_REF_PREFIX;
        use crunch_action_result_core::SignedActionResultRecord;
        use crunch_action_result_core::canonical_action_result;

        let typed_ref = |prefix: &str, value: &str| format!("{prefix}{}", blake3::hash(value.as_bytes()).to_hex());
        let record = canonical_action_result(ActionResultRecordInput {
            action_ref: typed_ref(ACTION_REF_PREFIX, seed),
            outputs: vec![ActionResultOutput {
                name: "out".to_string(),
                object_ref: typed_ref(OBJECT_REF_PREFIX, seed),
                store_path: output.to_absolute_path(),
                path_info_ref: typed_ref(PATH_INFO_REF_PREFIX, seed),
            }],
            action_receipt_ref: typed_ref(ACTION_RECEIPT_REF_PREFIX, seed),
            reference_scan_refs: vec![typed_ref(REFERENCE_SCAN_REF_PREFIX, seed)],
            sandbox_policy_ref: typed_ref(SANDBOX_POLICY_REF_PREFIX, "sandbox"),
            network_policy_ref: typed_ref(NETWORK_POLICY_REF_PREFIX, "network"),
            producer_identity: "builder-key-1".to_string(),
            producer_policy_ref: typed_ref(PRODUCER_POLICY_REF_PREFIX, "producer"),
            signature_refs: vec![typed_ref(SIGNATURE_REF_PREFIX, seed)],
            publication_policy_ref: typed_ref(PUBLICATION_POLICY_REF_PREFIX, "publication"),
            non_claims: vec![
                "ca-mapping-presence-is-not-output-trust".to_string(),
                "executor-correctness".to_string(),
                "index-presence-is-not-output-trust".to_string(),
            ],
        })
        .unwrap();
        SignedActionResultRecord {
            record,
            record_signatures: vec![DetachedRecordSignature {
                key_name: "builder-key-1".to_string(),
                signature: "signature".to_string(),
            }],
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

    #[test]
    fn reversed_blob_enumeration_has_the_same_reclaim_observations_and_plan_id() {
        let state_dir = tempfile::tempdir().unwrap();
        let blob_dir = state_dir.path().join("blobs").join("blobs").join("b3").join("aa");
        let blob_paths = ["11", "22", "33"].map(|suffix| blob_dir.join(format!("aa{}", suffix.repeat(31))));
        std::fs::create_dir_all(&blob_dir).unwrap();
        for path in &blob_paths {
            std::fs::write(path, b"orphaned-index").unwrap();
        }

        let mut forward = blob_paths.to_vec();
        let mut reversed = blob_paths.iter().rev().cloned().collect::<Vec<_>>();
        canonicalize_dead_blob_paths(&mut forward);
        canonicalize_dead_blob_paths(&mut reversed);
        let empty_paths: [PathBuf; 0] = [];
        let candidates: [String; 0] = [];
        let core_plan_id = [0_u8; blake3::OUT_LEN];
        let retention_plan_id = [1_u8; blake3::OUT_LEN];
        let report_for_paths = |blob_indexes: &[PathBuf]| {
            let reclaim = compute_reclaimable_bytes(ReclaimObservationPaths {
                orphaned_on_disk: &empty_paths,
                artifact_attestations: &empty_paths,
                closure_attestations: &empty_paths,
                blob_indexes,
                blob_chunks: &empty_paths,
                action_result_records: &empty_paths,
                action_result_indexes: &empty_paths,
            })
            .unwrap();
            let plan_id = execution_plan_id(ExecutionPlanIdentityInput {
                core_plan_id: &core_plan_id,
                retention_plan_id: &retention_plan_id,
                overlay_plan_identity: None,
                candidate_paths: &candidates,
                reclaim_observations: &reclaim.observations,
            });
            (reclaim.observations, plan_id)
        };
        let forward_plan = report_for_paths(&forward);
        let reversed_plan = report_for_paths(&reversed);
        assert_eq!(forward_plan.0.len(), blob_paths.len());
        assert!(forward_plan.0.iter().all(|observation| observation.category == RECLAIM_CATEGORY_BLOB_INDEX));
        assert_eq!(forward_plan, reversed_plan);
    }

    #[tokio::test]
    async fn blob_creation_order_does_not_change_plan_but_changed_blob_facts_stale_it() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut store = open_store(state_dir.path(), output_dir.path()).await;
        let blob_dir = state_dir.path().join("blobs").join("blobs").join("b3").join("aa");
        let blob_paths = ["11", "22", "33"].map(|suffix| blob_dir.join(format!("aa{}", suffix.repeat(31))));
        std::fs::create_dir_all(&blob_dir).unwrap();
        for path in &blob_paths {
            std::fs::write(path, b"orphaned-index").unwrap();
        }
        let first = store.garbage_collect(None).await.unwrap();
        assert_eq!(first.reclaim_observations.len(), blob_paths.len());
        assert!(
            first
                .reclaim_observations
                .iter()
                .all(|observation| observation.category == RECLAIM_CATEGORY_BLOB_INDEX)
        );

        for path in &blob_paths {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(&blob_dir).unwrap();
        std::fs::create_dir(&blob_dir).unwrap();
        for path in blob_paths.iter().rev() {
            std::fs::write(path, b"orphaned-index").unwrap();
        }
        let reordered = store.garbage_collect(None).await.unwrap();
        assert_eq!(first.reclaim_observations, reordered.reclaim_observations);
        assert_eq!(first.plan_id, reordered.plan_id);

        std::fs::write(&blob_paths[0], b"orphaned-index-with-changed-size").unwrap();
        let changed = store.garbage_collect(None).await.unwrap();
        assert_ne!(first.plan_id, changed.plan_id);
        let error = store
            .garbage_collect(Some(&first.plan_id))
            .await
            .expect_err("changed blob size must stale the plan");
        assert!(matches!(error, Error::Gc(message) if message.contains("stale-gc-plan")));
        assert!(blob_paths.iter().all(|path| path.exists()));
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
    fn casita_test_key(seed: u8) -> ObjectKey {
        ObjectKey::new("mantle.gc-fixture.v1".try_into().unwrap(), vec![seed]).unwrap()
    }

    fn casita_fence_entry(path: &StorePath<String>, target: &ObjectKey) -> CasitaFenceEntry {
        CasitaFenceEntry {
            kind: CasitaFenceKind::Output,
            path: path.to_absolute_path_with_prefix("/nix/store"),
            root_name: CasitaStore::root_name(path).unwrap().to_string(),
            expected_target: target.to_string(),
            removed: false,
            cleaned: false,
        }
    }

    #[cfg(unix)]
    #[test]
    fn casita_gc_fence_pending_rejects_dangling_orphan_journal() {
        let state = tempfile::tempdir().unwrap();
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        std::os::unix::fs::symlink("missing-journal-target", casita_fence_journal_path(state.path())).unwrap();
        assert!(casita_gc_fence_pending(state.path()).unwrap_err().to_string().contains("orphan"));
    }

    #[cfg(unix)]
    #[test]
    fn casita_fence_rejects_symlink_and_oversized_outcome_journal() {
        let state = tempfile::tempdir().unwrap();
        let entry = casita_fence_entry(&store_path("candidate", 33), &casita_test_key(4));
        persist_casita_fence(state.path(), &CasitaGcFence {
            plan_id: "planned".to_string(),
            entries: vec![entry],
        })
        .unwrap();
        let journal = casita_fence_journal_path(state.path());
        std::os::unix::fs::symlink("missing-journal-target", &journal).unwrap();
        assert!(load_casita_fence(state.path()).unwrap_err().to_string().contains("not a regular file"));
        std::fs::remove_file(&journal).unwrap();
        std::fs::File::create(&journal).unwrap().set_len(MAX_CASITA_GC_JOURNAL_BYTES as u64 + 1).unwrap();
        assert!(load_casita_fence(state.path()).unwrap_err().to_string().contains("persistence limit"));
    }

    #[test]
    fn casita_plan_identity_binds_exact_root_targets() {
        let path = store_path("candidate", 7);
        let name = CasitaStore::root_name(&path).unwrap();
        let mut targets =
            BTreeMap::from([(path.to_absolute_path_with_prefix("/nix/store"), (name, casita_test_key(1)))]);
        let node = Node::Symlink {
            target: SymlinkTarget::try_from("castore").unwrap(),
        };
        let castore_name = CasitaStore::castore_root_name(&node).unwrap();
        let castore_path = casita_castore_path_id(&castore_name);
        targets.insert(castore_path.clone(), (castore_name, casita_test_key(3)));
        let castore_id = *blake3::hash(b"retained-castore-plan").as_bytes();
        let initial = casita_execution_plan_id("unchanged-core-plan", &targets, &castore_id);
        targets.values_mut().next().unwrap().1 = casita_test_key(2);
        assert_ne!(initial, casita_execution_plan_id("unchanged-core-plan", &targets, &castore_id));
        let with_old_output = casita_execution_plan_id("unchanged-core-plan", &targets, &castore_id);
        targets.get_mut(&castore_path).unwrap().1 = casita_test_key(4);
        assert_ne!(with_old_output, casita_execution_plan_id("unchanged-core-plan", &targets, &castore_id));
    }

    #[test]
    fn casita_recovery_cleans_only_absent_roots_and_keeps_retained_exports() {
        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let dead = store_path("removed", 8);
        let retained = store_path("retained", 9);
        let target = casita_test_key(3);
        let removed = casita_fence_entry(&dead, &target);
        let intact = casita_fence_entry(&retained, &target);
        let dead_export = PathBuf::from(dead.to_absolute_path_with_prefix(exports.path().to_str().unwrap()));
        let retained_export = PathBuf::from(retained.to_absolute_path_with_prefix(exports.path().to_str().unwrap()));
        std::fs::write(&dead_export, b"remove me").unwrap();
        std::fs::write(&retained_export, b"keep me").unwrap();
        let fence = CasitaGcFence {
            plan_id: "planned".to_string(),
            entries: vec![removed.clone(), intact.clone()],
        };
        persist_casita_fence(state.path(), &fence).unwrap();
        assert!(fenced_root_was_removed(&removed, None).unwrap());
        assert!(!fenced_root_was_removed(&intact, Some(&target)).unwrap());
        cleanup_casita_removed(
            state.path(),
            CasitaGcPaths {
                output_dir_str: exports.path().to_str().unwrap(),
                store_dir: "/nix/store",
            },
            &removed,
        )
        .unwrap();
        assert!(!dead_export.exists());
        assert_eq!(std::fs::read(&retained_export).unwrap(), b"keep me");
        assert_eq!(load_casita_fence(state.path()).unwrap().unwrap().entries.len(), 2);
    }

    #[test]
    fn casita_fence_recovers_partial_progress_without_marking_other_roots_removed() {
        let state = tempfile::tempdir().unwrap();
        let first = casita_fence_entry(&store_path("first", 21), &casita_test_key(1));
        let untouched = casita_fence_entry(&store_path("second", 22), &casita_test_key(2));
        let fence = CasitaGcFence {
            plan_id: "planned".to_string(),
            entries: vec![first, untouched],
        };
        persist_casita_fence(state.path(), &fence).unwrap();
        append_casita_fence_outcome(state.path(), 0, false).unwrap();
        let mut file = std::fs::OpenOptions::new().append(true).open(casita_fence_journal_path(state.path())).unwrap();
        file.write_all(&[2, 0]).unwrap();
        file.sync_all().unwrap();

        let observed = load_casita_fence(state.path()).unwrap().unwrap();
        assert!(observed.entries[0].removed);
        assert!(!observed.entries[0].cleaned);
        assert!(!observed.entries[1].removed);
        assert_eq!(std::fs::metadata(casita_fence_journal_path(state.path())).unwrap().len(), 5);
        append_casita_fence_outcome(state.path(), 0, true).unwrap();
        let completed = load_casita_fence(state.path()).unwrap().unwrap();
        assert!(completed.entries[0].cleaned);
        assert!(!completed.entries[1].cleaned);
    }

    #[test]
    fn casita_recovery_rejects_changed_targets_and_preserves_published_roots() {
        let path = store_path("candidate", 10);
        let expected = casita_test_key(4);
        let changed = casita_test_key(5);
        let mut entry = casita_fence_entry(&path, &expected);
        assert!(
            fenced_root_was_removed(&entry, Some(&changed))
                .unwrap_err()
                .to_string()
                .contains("casita-root-conflict")
        );
        entry.removed = true;
        assert!(!fenced_root_was_removed(&entry, Some(&expected)).unwrap());
        assert!(fenced_root_was_removed(&entry, None).unwrap());
    }
    #[test]
    fn casita_castore_gc_keeps_retained_nodes_and_rejects_missing_roots() {
        let keep = Node::Symlink {
            target: SymlinkTarget::try_from("keep").unwrap(),
        };
        let dead = Node::Symlink {
            target: SymlinkTarget::try_from("dead").unwrap(),
        };
        let keep_name = CasitaStore::castore_root_name(&keep).unwrap();
        let dead_name = CasitaStore::castore_root_name(&dead).unwrap();
        let observed = vec![
            (keep_name.clone(), casita_test_key(1), keep.clone(), 23),
            (dead_name.clone(), casita_test_key(2), dead.clone(), 27),
        ];
        let retained =
            plan_casita_castore_facts(vec![casita_castore_path_id(&keep_name)], observed.clone(), false).unwrap();
        assert_eq!(retained.candidate_roots, vec![casita_castore_path_id(&dead_name)]);
        assert!(retained.targets.contains_key(&casita_castore_path_id(&keep_name)));
        let unretained = plan_casita_castore_facts(Vec::new(), observed, false).unwrap();
        let mut expected = vec![casita_castore_path_id(&dead_name), casita_castore_path_id(&keep_name)];
        expected.sort();
        assert_eq!(unretained.candidate_roots, expected);
        assert_ne!(retained.core_plan_id, unretained.core_plan_id);
        let missing = plan_casita_castore_facts(vec![casita_castore_path_id(&keep_name)], vec![], false).unwrap_err();
        assert!(missing.to_string().contains("casita-root-missing"));
    }

    #[test]
    fn casita_castore_fence_never_deletes_a_retained_output_export() {
        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let retained = store_path("retained", 12);
        let export = PathBuf::from(retained.to_absolute_path_with_prefix(exports.path().to_str().unwrap()));
        std::fs::write(&export, b"retained").unwrap();
        let node = Node::Symlink {
            target: SymlinkTarget::try_from("target").unwrap(),
        };
        let root = CasitaStore::castore_root_name(&node).unwrap();
        let target = casita_test_key(6);
        let entry = CasitaFenceEntry {
            kind: CasitaFenceKind::Castore,
            path: casita_castore_path_id(&root),
            root_name: root.to_string(),
            expected_target: target.to_string(),
            removed: true,
            cleaned: false,
        };
        cleanup_casita_removed(
            state.path(),
            CasitaGcPaths {
                output_dir_str: exports.path().to_str().unwrap(),
                store_dir: "/nix/store",
            },
            &entry,
        )
        .unwrap();
        assert_eq!(std::fs::read(export).unwrap(), b"retained");
        assert_eq!(fence_entry_identity(&entry, "/nix/store").unwrap().0, None);
    }
    fn casita_gc_test_context<'a>(
        state_dir: &'a Path,
        output_dir_str: &'a str,
        store: &'a Arc<CasitaStore>,
        pathinfo: &'a dyn PathInfoService,
        retained: &'a [Node],
    ) -> GcContext<'a> {
        GcContext {
            state_dir,
            output_dir_str,
            store_dir: "/nix/store",
            overlay_pathinfo: pathinfo,
            composed_pathinfo: pathinfo,
            overlay_directory_service: store.directory_service.as_ref(),
            overlay_blob_service: store.blob_service.as_ref(),
            overlay_plan_identity: None,
            retained_castore_roots: retained,
            casita_store: Some(store.clone()),
        }
    }

    #[tokio::test]
    async fn casita_gc_rejects_newly_retained_root_then_collects_only_unretained() {
        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let _guard = crate::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let store = CasitaStore::open(state.path(), "/nix/store").await.unwrap();
        let keep = Node::Symlink {
            target: SymlinkTarget::try_from("keep-output").unwrap(),
        };
        let dead = Node::Symlink {
            target: SymlinkTarget::try_from("dead-output").unwrap(),
        };
        store.admit_castore_payload_root(&keep).await.unwrap();
        store.admit_castore_payload_root(&dead).await.unwrap();
        let dead_name = CasitaStore::castore_root_name(&dead).unwrap();
        let dead_target = store
            .verified_gc_castore_roots()
            .await
            .unwrap()
            .into_iter()
            .find(|(name, _, _, _)| name == &dead_name)
            .unwrap()
            .1;
        assert!(store.repository.open_payload(&dead_target).await.unwrap().is_some());
        let empty_pathinfo = snix_store::pathinfoservice::LruPathInfoService::with_capacity(
            "casita-gc-test".to_string(),
            std::num::NonZeroUsize::new(2).unwrap(),
        );
        let output_dir_str = exports.path().to_str().unwrap();
        let retained = [keep.clone()];
        let ctx = casita_gc_test_context(state.path(), output_dir_str, &store, &empty_pathinfo, &retained);
        let mut ca_mappings = CaMappings::load(state.path());
        let planned = run_gc(&ctx, &mut ca_mappings, None).await.unwrap();
        assert_eq!(planned.candidate_paths, vec![casita_castore_path_id(
            &CasitaStore::castore_root_name(&dead).unwrap()
        )]);
        assert_eq!(planned.reclaimable_bytes_total, 0);
        assert!(planned.reclaim_observations.iter().any(|fact| fact.path == planned.candidate_paths[0]
            && fact.bytes.is_none()
            && fact.blocker.as_deref() == Some("casita-physical-size-not-observed")));

        let newly_retained = [keep.clone(), dead.clone()];
        let changed = casita_gc_test_context(state.path(), output_dir_str, &store, &empty_pathinfo, &newly_retained);
        let error = run_gc(&changed, &mut ca_mappings, Some(&planned.plan_id)).await.unwrap_err();
        assert!(error.to_string().contains("gc-plan-stale"));
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        store.rehydrate_castore_payload_root(&dead).await.unwrap();

        let executed = run_gc(&ctx, &mut ca_mappings, Some(&planned.plan_id)).await.unwrap();
        assert!(executed.execution_complete);
        assert!(executed.operations.contains(&GcOperationKind::CasitaCollection));
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        store.rehydrate_castore_payload_root(&keep).await.unwrap();
        let missing = store.rehydrate_castore_payload_root(&dead).await.unwrap_err();
        assert!(missing.to_string().contains("casita-root-missing"));
        assert!(store.repository.open_payload(&dead_target).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn casita_gc_busy_collector_keeps_reclaim_incomplete_until_recovery() {
        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let _guard = crate::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let store = CasitaStore::open(state.path(), "/nix/store").await.unwrap();
        let keep = Node::Symlink {
            target: SymlinkTarget::try_from("keep-busy").unwrap(),
        };
        let dead = Node::Symlink {
            target: SymlinkTarget::try_from("dead-busy").unwrap(),
        };
        store.admit_castore_payload_root(&keep).await.unwrap();
        store.admit_castore_payload_root(&dead).await.unwrap();
        let empty_pathinfo = snix_store::pathinfoservice::LruPathInfoService::with_capacity(
            "casita-busy-test".to_string(),
            std::num::NonZeroUsize::new(2).unwrap(),
        );
        let retained = [keep.clone()];
        let ctx =
            casita_gc_test_context(state.path(), exports.path().to_str().unwrap(), &store, &empty_pathinfo, &retained);
        let mut ca_mappings = CaMappings::load(state.path());
        let plan = run_gc(&ctx, &mut ca_mappings, None).await.unwrap();
        let lock_file = std::fs::File::options()
            .read(true)
            .write(true)
            .open(state.path().join("casita").join("gc.lock"))
            .unwrap();
        lock_file.lock().unwrap();
        let incomplete = run_gc(&ctx, &mut ca_mappings, Some(&plan.plan_id)).await.unwrap();
        assert!(!incomplete.execution_complete);
        assert!(incomplete.failed_operations.iter().any(|failure| failure.contains("reclaim-incomplete")));
        assert!(casita_gc_fence_pending(state.path()).unwrap());
        drop(lock_file);

        recover_casita_gc(
            state.path(),
            CasitaGcPaths {
                output_dir_str: exports.path().to_str().unwrap(),
                store_dir: "/nix/store",
            },
            &store,
            &mut ca_mappings,
        )
        .await
        .unwrap();
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        store.rehydrate_castore_payload_root(&keep).await.unwrap();
        assert!(
            store
                .rehydrate_castore_payload_root(&dead)
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-root-missing")
        );
    }

    #[tokio::test]
    async fn casita_recovery_before_first_root_removal_preserves_candidate() {
        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let _guard = crate::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let store = CasitaStore::open(state.path(), "/nix/store").await.unwrap();
        let node = Node::Symlink {
            target: SymlinkTarget::try_from("still-published").unwrap(),
        };
        store.admit_castore_payload_root(&node).await.unwrap();
        let plan =
            plan_casita_castore_facts(Vec::new(), store.verified_gc_castore_roots().await.unwrap(), true).unwrap();
        assert_eq!(plan.candidate_roots.len(), 1);
        let (name, target) = plan.targets.get(&plan.candidate_roots[0]).unwrap();
        let target = target.clone();
        persist_casita_fence(state.path(), &CasitaGcFence {
            plan_id: encode_blake3_identity(&plan.core_plan_id),
            entries: vec![CasitaFenceEntry {
                kind: CasitaFenceKind::Castore,
                path: plan.candidate_roots[0].clone(),
                root_name: name.to_string(),
                expected_target: target.to_string(),
                removed: false,
                cleaned: false,
            }],
        })
        .unwrap();
        drop(store);

        let reopened = CasitaStore::open(state.path(), "/nix/store").await.unwrap();
        let mut ca_mappings = CaMappings::load(state.path());
        recover_casita_gc(
            state.path(),
            CasitaGcPaths {
                output_dir_str: exports.path().to_str().unwrap(),
                store_dir: "/nix/store",
            },
            &reopened,
            &mut ca_mappings,
        )
        .await
        .unwrap();
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        reopened.rehydrate_castore_payload_root(&node).await.unwrap();
        assert!(reopened.repository.open_payload(&target).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn casita_recovery_replays_real_partial_root_removal_without_new_deletions() {
        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let _guard = crate::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let store = CasitaStore::open(state.path(), "/nix/store").await.unwrap();
        let retained = Node::Symlink {
            target: SymlinkTarget::try_from("retained").unwrap(),
        };
        let first = Node::Symlink {
            target: SymlinkTarget::try_from("first-candidate").unwrap(),
        };
        let second = Node::Symlink {
            target: SymlinkTarget::try_from("second-candidate").unwrap(),
        };
        for node in [&retained, &first, &second] {
            store.admit_castore_payload_root(node).await.unwrap();
        }
        let retained_name = CasitaStore::castore_root_name(&retained).unwrap();
        let observed = store.verified_gc_castore_roots().await.unwrap();
        let plan = plan_casita_castore_facts(vec![casita_castore_path_id(&retained_name)], observed, true).unwrap();
        assert_eq!(plan.candidate_roots.len(), 2);
        let entries = plan
            .candidate_roots
            .iter()
            .map(|root| {
                let (name, target) = plan.targets.get(root).unwrap();
                CasitaFenceEntry {
                    kind: CasitaFenceKind::Castore,
                    path: root.clone(),
                    root_name: name.to_string(),
                    expected_target: target.to_string(),
                    removed: false,
                    cleaned: false,
                }
            })
            .collect();
        persist_casita_fence(state.path(), &CasitaGcFence {
            plan_id: encode_blake3_identity(&plan.core_plan_id),
            entries,
        })
        .unwrap();
        let removed_name = &plan.targets.get(&plan.candidate_roots[0]).unwrap().0;
        let removed_target = &plan.targets.get(&plan.candidate_roots[0]).unwrap().1;
        assert!(store.repository.remove_root_if_matches(removed_name, removed_target).await.unwrap().is_some());
        assert!(!casita_fence_journal_path(state.path()).exists());
        drop(store);

        let reopened = CasitaStore::open(state.path(), "/nix/store").await.unwrap();
        let mut ca_mappings = CaMappings::load(state.path());
        recover_casita_gc(
            state.path(),
            CasitaGcPaths {
                output_dir_str: exports.path().to_str().unwrap(),
                store_dir: "/nix/store",
            },
            &reopened,
            &mut ca_mappings,
        )
        .await
        .unwrap();
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        let roots = reopened.verified_gc_castore_roots().await.unwrap();
        let survivors = roots.iter().map(|(name, _, _, _)| name.to_string()).collect::<BTreeSet<_>>();
        assert!(!survivors.contains(&plan.targets[&plan.candidate_roots[0]].0.to_string()));
        assert!(survivors.contains(&plan.targets[&plan.candidate_roots[1]].0.to_string()));
        assert!(survivors.contains(&retained_name.to_string()));
        reopened.rehydrate_castore_payload_root(&retained).await.unwrap();
    }

    #[tokio::test]
    async fn casita_gc_matches_snix_candidates_for_identical_published_outputs_and_retention() {
        use nix_compat::narinfo::SigningKey;
        use nix_compat::narinfo::VerifyingKey;
        use nix_compat::narinfo::fingerprint_with_store_dir;
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;
        use nix_compat::store_path::build_ca_path_with_store_dir;
        use snix_store::nar::NarCalculationService;
        use snix_store::nar::SimpleRenderer;

        let snix_state = tempfile::tempdir().unwrap();
        let snix_exports = tempfile::tempdir().unwrap();
        let casita_state = tempfile::tempdir().unwrap();
        let casita_exports = tempfile::tempdir().unwrap();
        let raw_signing = ed25519_dalek::SigningKey::from_bytes(&[19_u8; 32]);
        let verifying = VerifyingKey::new("gc-parity-fixture".to_string(), raw_signing.verifying_key());
        let signing = SigningKey::new("gc-parity-fixture".to_string(), raw_signing);
        std::fs::write(casita_state.path().join("casita-trusted-public-keys"), format!("{verifying}\n")).unwrap();
        let mut snix = open_store(snix_state.path(), snix_exports.path()).await;
        let guard = crate::StoreMutationGuard::acquire_wait(casita_state.path()).unwrap();
        let mut casita = StoreHandle::open(StoreConfig::new(
            crate::StoreBackend::Casita,
            casita_state.path().to_path_buf(),
            casita_exports.path().to_path_buf(),
            "/nix/store".to_string(),
        ))
        .await
        .unwrap();
        let mut paths = Vec::new();
        for (name, target) in [
            ("retained", "keep-link"),
            ("candidate-a", "drop-a"),
            ("candidate-b", "drop-b"),
        ] {
            let node = Node::Symlink {
                target: SymlinkTarget::try_from(target).unwrap(),
            };
            let (nar_size, nar_sha256) = SimpleRenderer::new(snix.blob_service(), snix.directory_service())
                .calculate_nar(&node)
                .await
                .unwrap();
            let ca = CAHash::Nar(NixHash::Sha256(nar_sha256));
            let store_path =
                build_ca_path_with_store_dir(name, &ca, Vec::<String>::new(), false, "/nix/store").unwrap();
            let mut info = PathInfo {
                store_path: store_path.clone(),
                node,
                references: Vec::new(),
                nar_size,
                nar_sha256,
                signatures: Vec::new(),
                deriver: None,
                ca: Some(ca),
            };
            let fingerprint = fingerprint_with_store_dir(
                &info.store_path.as_ref(),
                &info.nar_sha256,
                info.nar_size,
                std::iter::empty::<&nix_compat::store_path::StorePathRef>(),
                "/nix/store",
            );
            info.signatures.push(signing.sign(fingerprint.as_bytes()).to_owned());
            snix.pathinfo_service().put(info.clone()).await.unwrap();
            casita.pathinfo_service().put(info).await.unwrap();
            paths.push(store_path);
        }
        let mut observed = casita
            .casita_store
            .as_ref()
            .unwrap()
            .verified_gc_roots("/nix/store")
            .await
            .unwrap()
            .into_iter()
            .map(|(_, _, info)| info.store_path.to_absolute_path())
            .collect::<Vec<_>>();
        observed.sort();
        let mut expected_observed = paths.iter().map(StorePath::to_absolute_path).collect::<Vec<_>>();
        expected_observed.sort();
        assert_eq!(observed, expected_observed);
        snix.register_retained_root(&paths[0], GcRootSource::Build).await.unwrap();
        casita.register_retained_root(&paths[0], GcRootSource::Build).await.unwrap();

        let snix_plan = snix.garbage_collect(None).await.unwrap();
        let casita_plan = casita.garbage_collect_under_guard(&guard, None).await.unwrap();
        let mut expected_candidates = paths[1..].iter().map(StorePath::to_absolute_path).collect::<Vec<_>>();
        expected_candidates.sort();
        assert_eq!(snix_plan.candidate_paths, expected_candidates);
        assert_eq!(casita_plan.candidate_paths, snix_plan.candidate_paths);
        assert_eq!(snix_plan.retained_root_count, 1);
        assert_eq!(casita_plan.retained_root_count, snix_plan.retained_root_count);

        snix.register_retained_root(&paths[1], GcRootSource::Build).await.unwrap();
        casita.register_retained_root(&paths[1], GcRootSource::Build).await.unwrap();
        let snix_after_pin = snix.garbage_collect(None).await.unwrap();
        let casita_after_pin = casita.garbage_collect_under_guard(&guard, None).await.unwrap();
        assert_eq!(snix_after_pin.candidate_paths, vec![paths[2].to_absolute_path()]);
        assert_eq!(casita_after_pin.candidate_paths, snix_after_pin.candidate_paths);
        assert!(
            snix.garbage_collect(Some(&snix_plan.plan_id))
                .await
                .unwrap_err()
                .to_string()
                .contains("stale-gc-plan")
        );
        assert!(
            casita
                .garbage_collect_under_guard(&guard, Some(&casita_plan.plan_id))
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-plan-stale")
        );
        assert!(!casita_gc_fence_pending(casita_state.path()).unwrap());
        assert_eq!(casita.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn casita_gc_repointed_verified_output_rejects_old_plan_before_fencing() {
        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let signing = casita_gc_trusted_signer(state.path());
        let guard = crate::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let mut store = StoreHandle::open(StoreConfig::new(
            crate::StoreBackend::Casita,
            state.path().to_path_buf(),
            exports.path().to_path_buf(),
            "/nix/store".to_string(),
        ))
        .await
        .unwrap();
        let info = casita_gc_signed_symlink_output(&store, "repoint-candidate", "unchanged-content", &signing).await;
        store.pathinfo_service().put(info.clone()).await.unwrap();
        let initial = store.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap();
        assert_eq!(initial.len(), 1);
        let initial_target = initial[0].1.clone();
        let plan = store.garbage_collect_under_guard(&guard, None).await.unwrap();
        assert_eq!(plan.candidate_paths, vec![info.store_path.to_absolute_path()]);

        let mut repointed = info.clone();
        repointed.deriver = Some(store_path("different-deriver.drv", 41));
        store.pathinfo_service().put(repointed.clone()).await.unwrap();
        let observed = store.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap();
        assert_eq!(observed.len(), 1);
        assert_ne!(observed[0].1, initial_target);
        assert_eq!(observed[0].2, repointed);
        assert_eq!(store.pathinfo_service().get(*info.store_path.digest()).await.unwrap(), Some(repointed));

        let rejected = store.garbage_collect_under_guard(&guard, Some(&plan.plan_id)).await.unwrap_err();
        assert!(rejected.to_string().contains("gc-plan-stale"));
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        let still_published = store.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap();
        assert_eq!(still_published.len(), 1);
        assert_eq!(still_published[0].1, observed[0].1);
    }

    #[tokio::test]
    async fn casita_gc_executes_signed_output_plan_and_preserves_retained_nar_facts() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;
        use snix_store::nar::NarCalculationService;
        use snix_store::nar::SimpleRenderer;

        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let signing = casita_gc_trusted_signer(state.path());
        let guard = crate::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let mut store = StoreHandle::open(StoreConfig::new(
            crate::StoreBackend::Casita,
            state.path().to_path_buf(),
            exports.path().to_path_buf(),
            "/nix/store".to_string(),
        ))
        .await
        .unwrap();
        let kept = casita_gc_signed_symlink_output(&store, "kept-signed", "kept-nar-content", &signing).await;
        let removed = casita_gc_signed_symlink_output(&store, "collected-signed", "dead-nar-content", &signing).await;
        store.pathinfo_service().put(kept.clone()).await.unwrap();
        store.pathinfo_service().put(removed.clone()).await.unwrap();
        store.register_retained_root(&kept.store_path, GcRootSource::Build).await.unwrap();
        let roots = store.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap();
        assert_eq!(roots.len(), 2);
        let removed_target = roots.iter().find(|(_, _, info)| info.store_path == removed.store_path).unwrap().1.clone();

        let plan = store.garbage_collect_under_guard(&guard, None).await.unwrap();
        assert!(plan.is_dry_run);
        assert_eq!(plan.candidate_paths, vec![removed.store_path.to_absolute_path()]);
        assert_eq!(plan.retained_root_count, 1);
        let report = store.garbage_collect_under_guard(&guard, Some(&plan.plan_id)).await.unwrap();
        assert!(!report.is_dry_run);
        assert!(report.execution_complete, "{:?}", report.failed_operations);
        assert!(report.operations.contains(&GcOperationKind::CasitaCollection));
        assert_eq!(report.candidate_paths, plan.candidate_paths);
        assert!(!casita_gc_fence_pending(state.path()).unwrap());

        let observed = store.pathinfo_service().get(*kept.store_path.digest()).await.unwrap().unwrap();
        assert_eq!(observed, kept);
        let (nar_size, nar_sha256) = SimpleRenderer::new(store.blob_service(), store.directory_service())
            .calculate_nar(&observed.node)
            .await
            .unwrap();
        assert_eq!((nar_size, nar_sha256), (observed.nar_size, observed.nar_sha256));
        assert_eq!(observed.ca, Some(CAHash::Nar(NixHash::Sha256(nar_sha256))));
        assert!(store.pathinfo_service().get(*removed.store_path.digest()).await.unwrap().is_none());
        let surviving_roots = store.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap();
        assert_eq!(surviving_roots.len(), 1);
        assert_eq!(surviving_roots[0].2, kept);
        assert!(
            store
                .casita_store
                .as_ref()
                .unwrap()
                .repository
                .open_payload(&removed_target)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn casita_output_fence_recovery_preserves_unremoved_and_cleans_only_removed_metadata() {
        use crate::ActionResultStore;

        let state = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        let signing = casita_gc_trusted_signer(state.path());
        let guard = crate::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let config = || {
            StoreConfig::new(
                crate::StoreBackend::Casita,
                state.path().to_path_buf(),
                exports.path().to_path_buf(),
                "/nix/store".to_string(),
            )
        };
        let mut store = StoreHandle::open(config()).await.unwrap();
        let outputs = [
            casita_gc_signed_symlink_output(&store, "fenced-first", "content-first", &signing).await,
            casita_gc_signed_symlink_output(&store, "fenced-second", "content-second", &signing).await,
        ];
        let action_store = crate::LocalActionResultStore::new(state.path());
        let records = [
            casita_gc_action_result("fenced-first", &outputs[0].store_path),
            casita_gc_action_result("fenced-second", &outputs[1].store_path),
        ];
        let logical = outputs.each_ref().map(|info| info.store_path.to_absolute_path());
        let drv_paths = [
            store_path("fenced-first.drv", 42).to_absolute_path(),
            store_path("fenced-second.drv", 43).to_absolute_path(),
        ];
        let exported = outputs
            .each_ref()
            .map(|info| PathBuf::from(info.store_path.to_absolute_path_with_prefix(exports.path().to_str().unwrap())));
        for index in 0..outputs.len() {
            store.pathinfo_service().put(outputs[index].clone()).await.unwrap();
            action_store.publish(&records[index]).await.unwrap();
            let target = if index == 0 { "content-first" } else { "content-second" };
            std::os::unix::fs::symlink(target, &exported[index]).unwrap();
        }
        let mut mappings = CaMappings::load(state.path());
        for index in 0..outputs.len() {
            mappings.insert(&drv_paths[index], "out", &logical[index]);
        }
        mappings.save_checked(state.path()).unwrap();
        assert_eq!(CaMappings::load(state.path()).get(&drv_paths[0], "out"), Some(logical[0].as_str()));
        assert_eq!(CaMappings::load(state.path()).get(&drv_paths[1], "out"), Some(logical[1].as_str()));

        let plan = store.garbage_collect_under_guard(&guard, None).await.unwrap();
        assert_eq!(plan.candidate_path_count, 2);
        assert_eq!(plan.candidate_exported_output_count, 2);
        assert_eq!(plan.candidate_action_result_record_count, 2);
        assert_eq!(plan.candidate_action_result_index_count, 2);
        let roots = store.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap();
        let entries = plan
            .candidate_paths
            .iter()
            .map(|path| {
                let (_, target, info) =
                    roots.iter().find(|(_, _, info)| info.store_path.to_absolute_path() == *path).unwrap();
                casita_fence_entry(&info.store_path, target)
            })
            .collect::<Vec<_>>();
        let fence = CasitaGcFence {
            plan_id: plan.plan_id.clone(),
            entries,
        };

        // A restart before the first conditional removal must leave all output metadata intact.
        persist_casita_fence(state.path(), &fence).unwrap();
        drop(store);
        let mut reopened = StoreHandle::open(config()).await.unwrap();
        assert!(
            reopened
                .pathinfo_service()
                .get(*outputs[0].store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        reopened.recover_casita_gc_under_guard(&guard).await.unwrap();
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        assert_eq!(reopened.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap().len(), 2);
        for index in 0..outputs.len() {
            assert!(reopened.pathinfo_service().get(*outputs[index].store_path.digest()).await.unwrap().is_some());
            assert!(std::fs::read_link(&exported[index]).is_ok());
            assert_eq!(action_store.lookup(&records[index].record.action_ref).await.unwrap().records, vec![
                records[index].clone()
            ]);
            assert_eq!(CaMappings::load(state.path()).get(&drv_paths[index], "out"), Some(logical[index].as_str()));
        }

        // A second restart after root removal but before the outcome record replays only that removal.
        let replanned = reopened.garbage_collect_under_guard(&guard, None).await.unwrap();
        assert_eq!(replanned.candidate_paths, plan.candidate_paths);
        persist_casita_fence(state.path(), &CasitaGcFence {
            plan_id: replanned.plan_id,
            entries: fence.entries.clone(),
        })
        .unwrap();
        let removed = logical.iter().position(|path| path == &fence.entries[0].path).unwrap();
        let survivor = 1 - removed;
        let removable_metadata =
            local_action_result_gc_candidates(state.path(), &BTreeSet::from([logical[survivor].clone()])).unwrap();
        assert_eq!(removable_metadata.record_paths.len(), 1);
        assert_eq!(removable_metadata.index_marker_paths.len(), 1);
        let all_metadata = local_action_result_gc_candidates(state.path(), &BTreeSet::new()).unwrap();
        let kept_record = all_metadata
            .record_paths
            .iter()
            .find(|path| !removable_metadata.record_paths.contains(path))
            .unwrap()
            .clone();
        let kept_marker = all_metadata
            .index_marker_paths
            .iter()
            .find(|path| !removable_metadata.index_marker_paths.contains(path))
            .unwrap()
            .clone();
        let (_, name, target) = fence_entry_identity(&fence.entries[0], "/nix/store").unwrap();
        let repository = reopened.casita_store.as_ref().unwrap().repository.clone();
        assert!(repository.remove_root_if_matches(&name, &target).await.unwrap().is_some());
        assert!(!casita_fence_journal_path(state.path()).exists());
        drop(repository);
        drop(reopened);

        let mut recovered = StoreHandle::open(config()).await.unwrap();
        recovered.recover_casita_gc_under_guard(&guard).await.unwrap();
        assert!(!casita_gc_fence_pending(state.path()).unwrap());
        let published = recovered.casita_store.as_ref().unwrap().verified_gc_roots("/nix/store").await.unwrap();
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].2.store_path, outputs[survivor].store_path);
        assert!(std::fs::symlink_metadata(&exported[removed]).is_err());
        assert!(std::fs::read_link(&exported[survivor]).is_ok());
        assert!(!removable_metadata.record_paths[0].exists());
        assert!(!removable_metadata.index_marker_paths[0].exists());
        assert!(kept_record.is_file());
        assert!(kept_marker.is_file());
        assert!(action_store.lookup(&records[removed].record.action_ref).await.unwrap().records.is_empty());
        assert_eq!(action_store.lookup(&records[survivor].record.action_ref).await.unwrap().records, vec![
            records[survivor].clone()
        ]);
        let saved_mappings = CaMappings::load(state.path());
        assert_eq!(saved_mappings.get(&drv_paths[removed], "out"), None);
        assert_eq!(saved_mappings.get(&drv_paths[survivor], "out"), Some(logical[survivor].as_str()));
    }
}
