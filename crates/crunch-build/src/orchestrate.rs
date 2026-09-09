//! Build orchestration: recursively build derivations, check cache,
//! persist outputs.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crunch_action_result_core::DiscoveredActionResultCandidate;
use crunch_action_result_core::StrongReuseRequest;
use crunch_action_result_core::plan_strong_reuse;
use crunch_store::ArtifactProvenance;
use crunch_store::GcRootSource;
use crunch_store::OutputSubstitutionReport;
use crunch_store::layer::StoreLayer;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use snix_castore::Node;
#[cfg(test)]
use snix_castore::blobservice::BlobService;
#[cfg(test)]
use snix_castore::directoryservice::DirectoryService;
use snix_store::path_info::PathInfo;
#[cfg(test)]
use snix_store::pathinfoservice::PathInfoService;
use tracing::debug;
use tracing::info;

use crate::ActionResultRuntimeReport;
use crate::ActionResultTransferEvidence;
use crate::BuildNetworkPolicyReport;
use crate::Error;
use crate::HermeticityAuditEvent;
use crate::HermeticityMode;
use crate::action_result::ACTION_RESULT_DISPOSITION_CONFLICT;
use crate::action_result::ACTION_RESULT_DISPOSITION_MISS;
use crate::action_result::ACTION_RESULT_DISPOSITION_REUSED;
use crate::action_result::action_ref_for_derivation;
use crate::action_result::candidate_admission_facts;
use crate::action_result::discovery_runtime_report;
use crate::action_result::policy_refs_for_derivation;
use crate::action_result::publication_runtime_report;
use crate::action_result::signed_record_for_outputs;
use crate::action_result::trust_policy_for_action;
use crate::build_request::collect_input_paths;
use crate::build_request::derivation_to_build_request;
use crate::fod::verify_fod_hash;
use crate::references::resolve_references;
use crate::registry::DerivationRegistry;
use crate::signing::KeyPair;
use crate::signing::{self};

/// Apply a sequence of byte-level rewrites to a castore node.
/// Used for transitive CA input path replacement.
async fn apply_input_rewrites(
    node: &Node,
    rewrites: &[(String, String)],
    store: &crunch_store::BuildStore,
) -> Result<Node, Error> {
    store
        .apply_rewrites(node, rewrites)
        .await
        .map_err(|error| Error::Store(format!("rewriting build input paths: {error}")))
}

const DERIVATION_SUFFIX: &str = ".drv";

fn path_info_deriver(drv_path: &StorePath<String>) -> Result<StorePath<String>, Error> {
    let name = drv_path.name().strip_suffix(DERIVATION_SUFFIX).unwrap_or_else(|| drv_path.name());
    if name.is_empty() {
        return Err(Error::Store("PathInfo deriver name is empty after normalization".to_string()));
    }
    let deriver = StorePath::<String>::from_name_and_digest_fixed(name, *drv_path.digest())
        .map_err(|error| Error::Store(format!("normalizing PathInfo deriver {drv_path}: {error}")))?;
    debug_assert!(!deriver.name().is_empty());
    debug_assert_eq!(deriver.digest(), drv_path.digest());
    Ok(deriver)
}

fn push_unique_store_paths(
    ordered_paths: &mut Vec<StorePath<String>>,
    seen_paths: &mut HashSet<StorePath<String>>,
    new_paths: impl IntoIterator<Item = StorePath<String>>,
) {
    for path in new_paths {
        let was_inserted = seen_paths.insert(path.clone());
        if was_inserted {
            ordered_paths.push(path);
        }
    }
}

fn merge_sandbox_input_closures(
    mut direct_input_paths: BTreeSet<StorePath<String>>,
    source_paths: &[StorePath<String>],
    closure_sets: &[Vec<StorePath<String>>],
) -> BTreeSet<StorePath<String>> {
    let direct_count = direct_input_paths.len();
    direct_input_paths.extend(source_paths.iter().cloned());
    for closure in closure_sets {
        direct_input_paths.extend(closure.iter().cloned());
    }
    debug_assert!(direct_input_paths.len() >= direct_count);
    debug_assert!(source_paths.iter().all(|path| direct_input_paths.contains(path)));
    debug_assert!(closure_sets.iter().flatten().all(|path| direct_input_paths.contains(path)));
    direct_input_paths
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceClosureDisposition {
    Resolve,
    ReuseCurrentSessionOutput,
}

fn source_closure_disposition(is_current_session_output: bool) -> SourceClosureDisposition {
    if is_current_session_output {
        SourceClosureDisposition::ReuseCurrentSessionOutput
    } else {
        SourceClosureDisposition::Resolve
    }
}

fn derivation_uses_mutable_workspace(derivation: &Derivation) -> Result<bool, Error> {
    let Some(raw) = derivation.environment.get(crate::build_request::WORKSPACE_POLICY_ENV) else {
        return Ok(false);
    };
    let policy: crate::WorkspacePolicy = serde_json::from_slice(raw.as_ref())
        .map_err(|error| Error::Store(format!("invalid stateful workspace policy: {error}")))?;
    crate::validate_workspace_policy(&policy)
        .map_err(|reason| Error::Store(format!("invalid stateful workspace policy: {}", reason.as_str())))?;
    Ok(policy.mode == crate::WorkspaceMode::MutableSession)
}

/// The result of building a single derivation.
#[derive(Debug, Clone)]
pub struct BuildOutcome {
    /// The derivation store path (the .drv path).
    pub drv_path: StorePath<String>,
    /// Output name → PathInfo for each output.
    pub outputs: BTreeMap<String, PathInfo>,
    /// Output name → substitution reporting for successful remote cache hits.
    pub substitutions: BTreeMap<String, OutputSubstitutionReport>,
    /// Whether the build was served from cache (output already existed).
    pub cached: bool,
    /// Captured build stdout+stderr, if available.
    pub log: Option<String>,
}

struct CacheCheckHit {
    infos: BTreeMap<String, PathInfo>,
    substitutions: BTreeMap<String, OutputSubstitutionReport>,
}

struct SharedActionCandidates {
    action_ref: String,
    candidates: Vec<DiscoveredActionResultCandidate>,
    probes_by_result_ref: BTreeMap<String, crunch_store::ActionResultOutputProbe>,
    source_by_result_ref: BTreeMap<String, (String, String)>,
    diagnostics: Vec<String>,
}

fn shared_action_disposition(plan: &crunch_action_result_core::StrongReusePlan) -> &'static str {
    if plan.conflict_class.is_some() {
        return ACTION_RESULT_DISPOSITION_CONFLICT;
    }
    if plan.selected_result_ref.is_some() {
        return ACTION_RESULT_DISPOSITION_REUSED;
    }
    ACTION_RESULT_DISPOSITION_MISS
}

fn action_result_transfer_evidence(
    probe: &crunch_store::ActionResultOutputProbe,
) -> Result<ActionResultTransferEvidence, Error> {
    let output_count = u32::try_from(probe.outputs.len())
        .map_err(|_| Error::Store("shared action-result output count overflow".to_string()))?;
    if output_count == 0 {
        return Err(Error::Store("shared action-result output set empty".to_string()));
    }
    debug_assert!(probe.outputs.values().all(|path_info| !path_info.signatures.is_empty()));
    debug_assert!(probe.transferred_nar_bytes > 0 || probe.reused_nar_bytes > 0);
    Ok(ActionResultTransferEvidence {
        output_count,
        transferred_nar_bytes: probe.transferred_nar_bytes,
        reused_nar_bytes: probe.reused_nar_bytes,
    })
}

/// Metadata saved during `prepare_build`, consumed by `finish_build`.
/// Intermediate state for a single CA output during multi-output
/// CA derivation finalization (between pass 1 and pass 2).
struct CaOutputIntermediate {
    name: String,
    marked_node: Node,
    ca_path: StorePath<String>,
    marker_nar_size: u64,
    marker_nar_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FinalCaPathInfoFacts {
    final_nar_size: u64,
    final_nar_sha256: [u8; 32],
    path_identity_ca: nix_compat::nixhash::CAHash,
}

struct ResolvedOutput {
    output_path: StorePath<String>,
    final_node: Node,
    final_nar_size: u64,
    final_nar_sha256: [u8; 32],
    ca: Option<nix_compat::nixhash::CAHash>,
}

// r[impl store_transports.archive_export_closure]
// r[impl store_transports.archive_import_idempotent]
fn final_ca_path_info_facts(
    marker_nar_size: u64,
    marker_nar_sha256: [u8; 32],
    final_nar_size: u64,
    final_nar_sha256: [u8; 32],
) -> FinalCaPathInfoFacts {
    assert!(marker_nar_size > 0, "marker-normalized CA NAR size must be positive");
    assert!(final_nar_size > 0, "final CA NAR size must be positive");
    assert_eq!(marker_nar_sha256.len(), final_nar_sha256.len());
    FinalCaPathInfoFacts {
        final_nar_size,
        final_nar_sha256,
        path_identity_ca: nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(marker_nar_sha256)),
    }
}

pub(crate) struct PreparedBuild {
    pub(crate) drv_path: StorePath<String>,
    pub(crate) drv_name: String,
    pub(crate) derivation: Arc<Derivation>,
    pub(crate) refscan_needles: Vec<String>,
    pub(crate) sandbox_inputs: BTreeMap<StorePath<String>, Node>,
    pub(crate) input_rewrites: Vec<(String, String)>,
    pub(crate) is_ca: bool,
    /// Whether this derivation is a user-requested root.
    /// Root outputs get exported to disk; intermediate deps stay in castore.
    pub(crate) is_root: bool,
}

/// Result of `prepare_build`: either already done (cached/fetcher) or
/// needs a sandbox build.
pub(crate) enum PrepareResult {
    Done(BuildOutcome),
    NeedsBuild {
        prepared: PreparedBuild,
        build_request: Box<snix_build::buildservice::BuildRequest>,
    },
}

/// Orchestrates the build pipeline: evaluating dependencies, checking
/// cache, running builds, persisting results.
///
/// `BServ` is the only remaining generic: the sandbox dispatch point.
/// Build storage and action-result operations use concrete capability values.
pub struct Builder<BServ> {
    pub(crate) store: crunch_store::BuildStore,
    action_results: crunch_store::ActionResultPort,
    build_service: Arc<BServ>,
    /// Signing keypair — every PathInfo gets signed before persistence.
    keypair: KeyPair,
    /// Trusted public keys for signature verification on cache hits.
    trusted_keys: Vec<VerifyingKey>,
    /// When true, skip signature verification on cache hits.
    trust_unsigned: bool,
    verbose: bool,
    hermeticity_mode: HermeticityMode,
    hermeticity_audit_events: Vec<HermeticityAuditEvent>,
    build_environment_reports: Vec<crate::BuildEnvironmentReport>,
    network_policy_reports: Vec<BuildNetworkPolicyReport>,
    action_result_reports: Vec<ActionResultRuntimeReport>,
    source_closure_cache: HashMap<StorePath<String>, Vec<StorePath<String>>>,
    observed_source_paths: BTreeSet<StorePath<String>>,
    root_retention_source: Option<GcRootSource>,
}

impl<BServ> Builder<BServ>
where BServ: BuildService + 'static
{
    /// Create a builder from individual services for focused compatibility tests.
    #[cfg(test)]
    #[allow(tigerstyle::too_many_parameters)]
    pub fn new<BS, DS, PIS>(
        blob_service: BS,
        directory_service: DS,
        build_service: BServ,
        pathinfo_service: PIS,
        output_dir: PathBuf,
        store_dir: &str,
        keypair: KeyPair,
        trusted_keys: Vec<VerifyingKey>,
        trust_unsigned: bool,
        verbose: bool,
    ) -> Self
    where
        BS: BlobService + 'static,
        DS: DirectoryService + 'static,
        PIS: PathInfoService + 'static,
    {
        let output_dir_str = output_dir.to_str().unwrap_or(store_dir).to_string();
        let store = crunch_store::StoreHandle::from_services_with_store_dir(
            crunch_store::StoreHandleServices {
                blob_service: Arc::new(blob_service) as Arc<dyn BlobService>,
                directory_service: Arc::new(directory_service) as Arc<dyn DirectoryService>,
                pathinfo_service: Arc::new(pathinfo_service) as Arc<dyn PathInfoService>,
                remote_pathinfo: None,
                state_dir: PathBuf::from("/tmp/crunch-test"),
                output_dir_str,
                publishers: Vec::new(),
            },
            store_dir.to_string(),
        );
        Self::from_store_parts(
            store.into_builder_store_parts(),
            build_service,
            keypair,
            trusted_keys,
            trust_unsigned,
            verbose,
        )
    }

    /// Create a builder from explicit services for focused compatibility tests.
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    #[allow(tigerstyle::too_many_parameters)]
    pub fn with_state_dir(
        blob_service: Arc<dyn BlobService>,
        directory_service: Arc<dyn DirectoryService>,
        build_service: BServ,
        pathinfo_service: Arc<dyn PathInfoService>,
        output_dir: PathBuf,
        state_dir: Option<PathBuf>,
        remote_pathinfo: Option<Arc<dyn PathInfoService>>,
        store_dir: &str,
        keypair: KeyPair,
        trusted_keys: Vec<VerifyingKey>,
        trust_unsigned: bool,
        verbose: bool,
    ) -> Self {
        let output_dir_str = output_dir.to_str().unwrap_or(store_dir).to_string();
        let state_dir = state_dir.unwrap_or_else(|| PathBuf::from("/tmp/crunch-no-state"));
        let store = crunch_store::StoreHandle::from_services_with_store_dir(
            crunch_store::StoreHandleServices {
                blob_service,
                directory_service,
                pathinfo_service,
                remote_pathinfo,
                state_dir,
                output_dir_str,
                publishers: Vec::new(),
            },
            store_dir.to_string(),
        );
        Self::from_store_parts(
            store.into_builder_store_parts(),
            build_service,
            keypair,
            trusted_keys,
            trust_unsigned,
            verbose,
        )
    }

    #[allow(tigerstyle::too_many_parameters)]
    pub fn from_store_parts(
        store_parts: crunch_store::BuilderStoreParts,
        build_service: BServ,
        keypair: KeyPair,
        trusted_keys: Vec<VerifyingKey>,
        trust_unsigned: bool,
        verbose: bool,
    ) -> Self {
        Self {
            store: store_parts.build_store,
            action_results: store_parts.action_results,
            build_service: Arc::new(build_service),
            keypair,
            trusted_keys,
            trust_unsigned,
            verbose,
            hermeticity_mode: HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            action_result_reports: Vec::new(),
            source_closure_cache: HashMap::new(),
            observed_source_paths: BTreeSet::new(),
            root_retention_source: None,
        }
    }

    /// Build a derivation and all its dependencies. Returns the outcome.
    ///
    /// Delegates to `build_all` with a single root and max_jobs=1.
    /// Get a cloned Arc to the build service (for spawning tasks).
    pub(crate) fn build_service(&self) -> Arc<BServ> {
        self.build_service.clone()
    }

    /// The logical store directory prefix.
    pub fn store_dir(&self) -> &str {
        self.store.store_dir()
    }

    pub fn set_hermeticity_mode(&mut self, hermeticity_mode: HermeticityMode) {
        self.hermeticity_mode = hermeticity_mode;
    }

    pub fn set_root_retention_source(&mut self, root_retention_source: Option<GcRootSource>) {
        self.root_retention_source = root_retention_source;
    }

    pub fn set_root_registration(&mut self, registration: Option<crunch_store::RootRegistration>) {
        self.store.set_root_registration(registration);
    }

    pub fn source_generation_paths(&self) -> Vec<StorePath<String>> {
        self.observed_source_paths.iter().cloned().collect()
    }

    pub fn overlay_report(&self) -> Result<Option<crunch_store::StoreOverlayReport>, Error> {
        self.store.overlay_report().map_err(|error| Error::Store(error.to_string()))
    }

    pub fn take_store_layer_selections(&mut self) -> Vec<crunch_store::layer::StoreLayerSelection> {
        self.store.take_read_layer_selections()
    }

    pub fn take_hermeticity_audit_events(&mut self) -> Vec<HermeticityAuditEvent> {
        std::mem::take(&mut self.hermeticity_audit_events)
    }

    pub fn take_build_environment_reports(&mut self) -> Vec<crate::BuildEnvironmentReport> {
        std::mem::take(&mut self.build_environment_reports)
    }

    pub fn take_network_policy_reports(&mut self) -> Vec<BuildNetworkPolicyReport> {
        std::mem::take(&mut self.network_policy_reports)
    }

    pub fn take_action_result_reports(&mut self) -> Vec<ActionResultRuntimeReport> {
        let mut action_result_evidence = std::mem::take(&mut self.action_result_reports);
        action_result_evidence.sort_by(|left, right| {
            left.action_ref
                .cmp(&right.action_ref)
                .then(left.phase.cmp(&right.phase))
                .then(left.disposition.cmp(&right.disposition))
                .then(left.selected_result_ref.cmp(&right.selected_result_ref))
        });
        action_result_evidence
    }

    /// Read the full content of a blob from castore.
    ///
    /// Used by the Worker to read `.drv` files from build outputs
    /// (dynamic derivation detection).
    pub(crate) async fn read_blob(&self, node: &snix_castore::Node) -> Result<Vec<u8>, Error> {
        const MAX_BLOB_READ_BYTES: u64 = 8_388_608;
        self.store
            .read_file_node(node, MAX_BLOB_READ_BYTES)
            .await
            .map_err(|error| Error::Store(format!("reading build blob: {error}")))
    }

    /// Prefer `build_all` when building multiple roots.
    pub async fn build(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<BuildOutcome, Error> {
        let mut outcomes = self.build_all(std::slice::from_ref(drv_path), known_paths, 1).await?;
        outcomes.pop().ok_or_else(|| Error::DerivationNotFound { path: drv_path.clone() })
    }

    /// Build multiple root derivations and all their dependencies,
    /// using the lazy goal-based scheduler.
    ///
    /// Creates goals lazily via `Worker::want()`, then runs the
    /// dispatch loop which handles cache hits inline and spawns
    /// sandbox builds concurrently up to `max_jobs`.
    ///
    /// Returns outcomes for root derivations only.
    pub async fn build_all(
        &mut self,
        roots: &[StorePath<String>],
        known_paths: &mut DerivationRegistry,
        max_jobs: u32,
    ) -> Result<Vec<BuildOutcome>, Error> {
        let result = self.build_all_report(roots, known_paths, max_jobs).await?;
        if !result.failed.is_empty() {
            return Err(Error::Store(format!("{} root build(s) failed", result.failed.len(),)));
        }
        Ok(result.outcomes)
    }

    /// Build roots through the ordinary worker while preserving partial root results.
    pub async fn build_all_report(
        &mut self,
        roots: &[StorePath<String>],
        known_paths: &mut DerivationRegistry,
        max_jobs: u32,
    ) -> Result<crate::WorkerResult, Error> {
        use crate::worker::Worker;

        if roots.is_empty() {
            return Ok(crate::WorkerResult {
                outcomes: Vec::new(),
                all_outcomes: Vec::new(),
                failed: Vec::new(),
                native_dynamic_plans: Vec::new(),
                priority_decisions: Vec::new(),
            });
        }
        debug_assert!(max_jobs >= 1, "max_jobs must be at least 1");
        let mut worker = Worker::with_scheduling_policy(max_jobs, crate::scheduling::SchedulingPolicy::default())?;
        for root in roots {
            worker.want(root, known_paths, true)?;
        }
        worker.run(self, known_paths).await
    }

    /// Prepare a single derivation for building. Handles cache hits and
    /// fetchers inline; for sandbox builds, returns the prepared metadata
    /// needed to dispatch `do_build` and later `finish_build`.
    pub(crate) async fn prepare_build(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: Arc<Derivation>,
        known_paths: &mut DerivationRegistry,
        is_root: bool,
    ) -> Result<PrepareResult, Error> {
        let drv_name = drv_path.name().to_string();
        let derivation_ref = derivation.as_ref();

        // 1. Strong shared action-result admission. This precedes ordinary
        // PathInfo cache lookup so input-addressed results cannot bypass the
        // receipt, policy, reference-scan, conflict, and poisoning checks.
        // Mutable-history executions are practical-only and must never satisfy
        // this claim boundary. Every candidate is re-admitted before the
        // executor can be skipped.
        // r[impl build_correctness.shared_action_result_admission]
        let is_mutable_workspace = derivation_uses_mutable_workspace(derivation_ref)?;
        if !is_mutable_workspace
            && let Some(shared_hit) =
                self.check_shared_action_result(drv_path, derivation_ref, known_paths, is_root).await?
        {
            info!(drv = %drv_name, "shared action result admitted, skipping executor");
            return Ok(PrepareResult::Done(BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: shared_hit.infos,
                substitutions: BTreeMap::new(),
                cached: true,
                log: None,
            }));
        }

        // 2. Ordinary signed PathInfo cache fallback remains independent when
        // no shared action result is fully admitted.
        if !is_mutable_workspace && let Some(cached_hit) = self.check_cache(drv_path, derivation_ref, is_root).await? {
            self.record_cached_output_paths(drv_path, derivation_ref, &cached_hit.infos, known_paths)?;
            info!(drv = %drv_name, "all outputs cached, skipping build");
            return Ok(PrepareResult::Done(BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: cached_hit.infos,
                substitutions: cached_hit.substitutions,
                cached: true,
                log: None,
            }));
        }

        // 3. Ensure input derivation outputs are in castore.
        for input_drv_path in derivation_ref.input_derivations.keys() {
            let input_abs = input_drv_path.to_absolute_path_with_prefix(self.store.store_dir());
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs) {
                self.ensure_input_nodes(entry.derivation.as_ref()).await?;
            }
        }

        // 4. Resolve source inputs + closures.
        let all_source_paths = self.resolve_and_ingest_sources(derivation_ref).await?;

        // 5. Collect sandbox inputs.
        let sandbox_inputs = self.collect_sandbox_inputs(derivation_ref, known_paths, &all_source_paths).await?;

        // 6. Create build request from the registry-bound execution profile.
        let drv_absolute = drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        let execution_profile = known_paths
            .get_by_drv_path(&drv_absolute)
            .map(|entry| entry.execution_profile.clone())
            .ok_or_else(|| Error::DerivationNotFound { path: drv_path.clone() })?;
        let request_envelope = match derivation_to_build_request(
            derivation_ref,
            &sandbox_inputs,
            self.store.store_dir(),
            &execution_profile,
            self.hermeticity_mode,
        ) {
            Ok(envelope) => envelope,
            Err(error) => {
                match &error {
                    Error::NetworkPolicyDenied { report, .. } => {
                        self.network_policy_reports.push((**report).clone());
                    }
                    Error::DeniedEnvironmentVariable { report, .. } => {
                        self.build_environment_reports.push((**report).clone());
                    }
                    _ => {}
                }
                return Err(error);
            }
        };
        self.hermeticity_audit_events.extend(request_envelope.audit_events.iter().cloned());
        self.build_environment_reports.push(request_envelope.build_environment_report.clone());
        self.network_policy_reports.push(request_envelope.network_policy_report.clone());
        let build_request = request_envelope.build_request;
        let refscan_needles = build_request.refscan_needles.clone();

        info!(drv = %drv_name, "building");
        if self.verbose {
            debug!(
                drv = %drv_name,
                inputs = sandbox_inputs.len(),
                outputs = build_request.outputs.len(),
                "dispatching sandbox build"
            );
        }

        let input_rewrites = self.collect_ca_input_rewrites(derivation_ref, known_paths);
        let is_ca = derivation_ref.outputs.values().all(|o| o.path.is_none() && o.ca_hash.is_none());

        Ok(PrepareResult::NeedsBuild {
            prepared: PreparedBuild {
                drv_path: drv_path.clone(),
                drv_name,
                derivation,
                refscan_needles,
                sandbox_inputs,
                input_rewrites,
                is_ca,
                is_root,
            },
            build_request: Box::new(build_request),
        })
    }

    /// Process a completed sandbox build: apply rewrites, hash outputs,
    /// persist PathInfo, export to disk.
    ///
    /// For multi-output CA derivations, uses a two-pass approach:
    /// 1. Apply input rewrites + compute CA paths for all outputs
    /// 2. Rewrite cross-output references in all outputs
    pub(crate) async fn finish_build(
        &mut self,
        prepared: &PreparedBuild,
        build_result: snix_build::buildservice::BuildResult,
        known_paths: &mut DerivationRegistry,
    ) -> Result<BuildOutcome, Error> {
        let mut output_infos: BTreeMap<String, PathInfo> = BTreeMap::new();
        let output_names: Vec<String> = prepared.derivation.outputs.keys().cloned().collect();
        let is_multi_ca = prepared.is_ca && prepared.derivation.outputs.len() > 1;
        let artifact_provenance = self.build_artifact_provenance(&prepared.derivation, known_paths)?;

        if is_multi_ca {
            output_infos =
                self.finish_build_multi_ca(prepared, &build_result, known_paths, &artifact_provenance).await?;
        } else {
            for (i, (output_name, output)) in prepared.derivation.outputs.iter().enumerate() {
                let build_output = build_result.outputs.get(i).ok_or_else(|| Error::OutputMissing {
                    output: output_name.clone(),
                })?;

                let path_info = self
                    .process_output(
                        &prepared.drv_path,
                        &prepared.drv_name,
                        output_name,
                        output,
                        build_output,
                        &prepared.input_rewrites,
                        &prepared.sandbox_inputs,
                        &prepared.refscan_needles,
                        &prepared.derivation,
                        known_paths,
                        prepared.is_ca,
                        &artifact_provenance,
                        prepared.is_root,
                    )
                    .await?;

                output_infos.insert(output_name.clone(), path_info);
            }
        }

        if !derivation_uses_mutable_workspace(&prepared.derivation)? {
            self.publish_completed_action_result(&prepared.derivation, &output_infos).await;
        }

        info!(
            drv = %prepared.drv_name,
            outputs = ?output_names.iter()
                .filter_map(|n| {
                    output_infos.get(n).map(|pi| {
                        pi.store_path.to_absolute_path_with_prefix(self.store.output_dir_str())
                    })
                })
                .collect::<Vec<_>>(),
            "build succeeded"
        );

        Ok(BuildOutcome {
            drv_path: prepared.drv_path.clone(),
            outputs: output_infos,
            substitutions: BTreeMap::new(),
            cached: false,
            log: build_result.log,
        })
    }

    /// Multi-output CA: two-pass processing.
    ///
    /// Pass 1: For each output, apply input rewrites, then replace ALL
    ///         of the derivation's own output provisionals with distinct
    ///         markers. Hash the canonical form to get the CA path.
    /// Pass 2: For each output, replace all markers with the final CA
    ///         paths (handles cross-output references).
    async fn finish_build_multi_ca(
        &mut self,
        prepared: &PreparedBuild,
        build_result: &snix_build::buildservice::BuildResult,
        known_paths: &mut DerivationRegistry,
        artifact_provenance: &ArtifactProvenance,
    ) -> Result<BTreeMap<String, PathInfo>, Error> {
        let ca_plans = crate::ca_plan::plan_ca_outputs(
            &prepared.drv_name,
            &prepared.derivation.outputs,
            &prepared.derivation.environment,
        );
        assert_eq!(ca_plans.len(), prepared.derivation.outputs.len(), "CA plan count must match output count",);

        // Pass 1: marker replacement + CA path computation.
        let intermediates = self.compute_ca_intermediates(prepared, build_result, &ca_plans).await?;
        assert_eq!(
            intermediates.len(),
            prepared.derivation.outputs.len(),
            "intermediate count must match output count",
        );

        // Pass 2: replace markers with final paths, persist.
        self.finalize_ca_outputs(prepared, build_result, known_paths, artifact_provenance, &ca_plans, &intermediates)
            .await
    }

    /// Pass 1: apply input rewrites, replace provisionals with markers,
    /// compute NAR hashes, and derive CA store paths.
    async fn compute_ca_intermediates(
        &self,
        prepared: &PreparedBuild,
        build_result: &snix_build::buildservice::BuildResult,
        ca_plans: &[crate::ca_plan::CaOutputPlan],
    ) -> Result<Vec<CaOutputIntermediate>, Error> {
        let mut intermediates: Vec<CaOutputIntermediate> = Vec::with_capacity(prepared.derivation.outputs.len());

        for (i, (output_name, _output)) in prepared.derivation.outputs.iter().enumerate() {
            let build_output = build_result.outputs.get(i).ok_or_else(|| Error::OutputMissing {
                output: output_name.clone(),
            })?;

            let mut node = apply_input_rewrites(&build_output.node, &prepared.input_rewrites, &self.store).await?;

            for plan in ca_plans {
                if plan.provisional.is_empty() {
                    continue;
                }
                let (rewritten, _) = self
                    .store
                    .rewrite_node(&node, plan.provisional.as_bytes(), &plan.marker)
                    .await
                    .map_err(|error| Error::Store(format!("rewriting CA marker: {error}")))?;
                node = rewritten;
            }

            let (nar_size, nar_sha256) =
                self.store.calculate_nar(&node).await.map_err(|error| Error::NarCalculation(error.to_string()))?;

            let plan = ca_plans
                .iter()
                .find(|p| p.output_name == *output_name)
                .ok_or_else(|| Error::Store(format!("no CA plan for output '{output_name}'")))?;
            let ca_path = crate::ca_plan::compute_ca_store_path(&plan.path_name, nar_sha256, self.store.store_dir())?;

            intermediates.push(CaOutputIntermediate {
                name: output_name.clone(),
                marked_node: node,
                ca_path,
                marker_nar_size: nar_size,
                marker_nar_sha256: nar_sha256,
            });
        }

        Ok(intermediates)
    }

    /// Pass 2: replace markers with final CA paths, register outputs,
    /// persist PathInfo.
    #[allow(tigerstyle::too_many_parameters)] // multi-CA output pipeline: groups are structurally coupled
    async fn finalize_ca_outputs(
        &mut self,
        prepared: &PreparedBuild,
        build_result: &snix_build::buildservice::BuildResult,
        known_paths: &mut DerivationRegistry,
        artifact_provenance: &ArtifactProvenance,
        ca_plans: &[crate::ca_plan::CaOutputPlan],
        intermediates: &[CaOutputIntermediate],
    ) -> Result<BTreeMap<String, PathInfo>, Error> {
        let final_rewrites: Vec<(&[u8], Vec<u8>)> = ca_plans
            .iter()
            .filter_map(|plan| {
                let intermediate = intermediates.iter().find(|i| i.name == plan.output_name)?;
                let final_abs = intermediate.ca_path.to_absolute_path_with_prefix(self.store.store_dir());
                Some((plan.marker.as_slice(), final_abs.into_bytes()))
            })
            .collect();

        let drv_abs = prepared.drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        if intermediates.len() > prepared.derivation.outputs.len() {
            return Err(Error::Store("multi-CA intermediate count exceeds declared outputs".to_string()));
        }
        let mut output_infos: BTreeMap<String, PathInfo> = BTreeMap::new();

        for (idx, intermediate) in intermediates.iter().enumerate() {
            if output_infos.len() >= intermediates.len() {
                return Err(Error::Store("multi-CA output map exceeded intermediate bound".to_string()));
            }
            let final_node = self.rewrite_markers_to_final(&intermediate.marked_node, &final_rewrites).await?;

            self.register_ca_output(
                &drv_abs,
                &prepared.drv_name,
                &intermediate.name,
                &intermediate.ca_path,
                known_paths,
            )?;

            let build_output = build_result.outputs.get(idx).ok_or_else(|| Error::OutputMissing {
                output: intermediate.name.clone(),
            })?;
            let references = resolve_references(
                &build_output.output_needles,
                &prepared.refscan_needles,
                &prepared.derivation,
                &prepared.sandbox_inputs,
            );

            let facts = self
                .measure_final_ca_path_info_facts(
                    &final_node,
                    intermediate.marker_nar_size,
                    intermediate.marker_nar_sha256,
                )
                .await?;

            let path_info = self
                .persist_and_export_output(
                    &prepared.drv_path,
                    &intermediate.name,
                    &intermediate.ca_path,
                    final_node,
                    references,
                    facts.final_nar_size,
                    facts.final_nar_sha256,
                    Some(facts.path_identity_ca),
                    Some(artifact_provenance.clone()),
                    prepared.is_root,
                )
                .await?;

            output_infos.insert(intermediate.name.clone(), path_info);
        }

        Ok(output_infos)
    }

    async fn measure_final_ca_path_info_facts(
        &self,
        final_node: &Node,
        marker_nar_size: u64,
        marker_nar_sha256: [u8; 32],
    ) -> Result<FinalCaPathInfoFacts, Error> {
        let (final_nar_size, final_nar_sha256) = self
            .store
            .calculate_nar(final_node)
            .await
            .map_err(|error| Error::NarCalculation(error.to_string()))?;
        Ok(final_ca_path_info_facts(marker_nar_size, marker_nar_sha256, final_nar_size, final_nar_sha256))
    }

    /// Rewrite all same-length CA markers to their final absolute paths.
    async fn rewrite_markers_to_final(&self, node: &Node, final_rewrites: &[(&[u8], Vec<u8>)]) -> Result<Node, Error> {
        let mut final_node = node.clone();
        for (marker, final_bytes) in final_rewrites {
            if marker.len() == final_bytes.len() {
                let (rewritten, _) = self
                    .store
                    .rewrite_node(&final_node, marker, final_bytes)
                    .await
                    .map_err(|error| Error::Store(format!("rewriting final CA marker: {error}")))?;
                final_node = rewritten;
            }
        }
        Ok(final_node)
    }

    fn closure_fallback_mode(&self) -> crunch_store::StoreFallbackMode {
        if self.hermeticity_mode.is_strict() {
            crunch_store::StoreFallbackMode::Strict
        } else {
            crunch_store::StoreFallbackMode::Practical
        }
    }

    async fn ensure_declared_source_exists(&mut self, source_path: &StorePath<String>) -> Result<(), Error> {
        assert!(!source_path.name().is_empty(), "source path name must not be empty");
        assert!(source_path.to_string().contains('-'), "source path text must include a digest/name separator");

        let host_path = self.preferred_source_host_path(source_path);
        if host_path.exists() {
            return Ok(());
        }
        let stored = self
            .store
            .cached_node_for_path(source_path)
            .await
            .map_err(|error| Error::Store(format!("checking declared source PathInfo: {error}")))?;
        if stored.is_some() {
            return Ok(());
        }
        Err(Error::SourceNotFound {
            path: source_path.clone(),
            store_dir: self.store.store_dir().to_string(),
        })
    }

    async fn resolve_input_closure_paths(
        &mut self,
        input_path: &StorePath<String>,
    ) -> Result<Vec<StorePath<String>>, Error> {
        assert!(!input_path.name().is_empty(), "input path name must not be empty");
        assert!(input_path.to_string().contains('-'), "input path text must include a digest/name separator");

        if let Some(cached_paths) = self.source_closure_cache.get(input_path) {
            return Ok(cached_paths.clone());
        }

        let closure = self
            .store
            .resolve_closure(input_path, self.closure_fallback_mode())
            .await
            .map_err(|error| Error::Store(format!("closure resolution failed for {input_path}: {error}")))?;
        self.hermeticity_audit_events
            .extend(closure.audit_events.into_iter().map(HermeticityAuditEvent::from));
        assert!(!closure.paths.is_empty(), "closure must contain at least the root input path");

        let resolved_paths = closure.paths;
        self.source_closure_cache.insert(input_path.clone(), resolved_paths.clone());
        Ok(resolved_paths)
    }

    fn preferred_source_host_path(&self, source_path: &StorePath<String>) -> PathBuf {
        let physical_path = PathBuf::from(source_path.to_absolute_path_with_prefix(self.store.output_dir_str()));
        if physical_path.exists() {
            return physical_path;
        }
        PathBuf::from(source_path.to_absolute_path_with_prefix(self.store.store_dir()))
    }

    async fn cached_or_ingested_node_for_path(
        &mut self,
        path: &StorePath<String>,
        host_path: &Path,
    ) -> Result<Option<Node>, Error> {
        assert!(!path.name().is_empty(), "store path name must not be empty");
        assert!(host_path.is_absolute(), "host path must be absolute");

        if let Some(node) = self
            .store
            .cached_node_for_path(path)
            .await
            .map_err(|e| Error::Store(format!("reusing cached node for {path}: {e}")))?
        {
            return Ok(Some(node));
        }
        if !host_path.exists() {
            return Ok(None);
        }

        let node = self.store.ingest_build_input(path.clone(), host_path).await.map_err(|error| {
            Error::Sandbox(std::io::Error::other(format!("failed to ingest input {path}: {error}")))
        })?;
        Ok(Some(node))
    }

    /// Validate source inputs, resolve closures via PathInfo/narinfo,
    /// and ingest all paths into castore. Returns the full set of
    /// source paths (declared + transitive closure).
    async fn resolve_and_ingest_sources(&mut self, derivation: &Derivation) -> Result<Vec<StorePath<String>>, Error> {
        let mut all_source_paths = Vec::new();
        let mut seen_source_paths = HashSet::new();
        push_unique_store_paths(
            &mut all_source_paths,
            &mut seen_source_paths,
            derivation.input_sources.iter().cloned(),
        );

        for source_path in &derivation.input_sources {
            let physical_source_path = source_path.to_absolute_path_with_prefix(self.store.output_dir_str());
            let disposition = source_closure_disposition(self.store.has_built_output(&physical_source_path));
            if disposition == SourceClosureDisposition::ReuseCurrentSessionOutput {
                debug!(path = %source_path, "skipping closure resolution for current-session output");
                continue;
            }
            self.ensure_declared_source_exists(source_path).await?;
            let closure_paths = self.resolve_input_closure_paths(source_path).await?;
            push_unique_store_paths(&mut all_source_paths, &mut seen_source_paths, closure_paths);
        }

        for source_path in &all_source_paths {
            let host_path = self.preferred_source_host_path(source_path);
            if self.cached_or_ingested_node_for_path(source_path, &host_path).await?.is_none() {
                debug!(path = %source_path, "source path not found on disk, skipping");
                continue;
            }
            self.observed_source_paths.insert(source_path.clone());
        }

        Ok(all_source_paths)
    }

    /// Gather all castore nodes needed as sandbox inputs: built
    /// dependency outputs, their PathInfo closures, and source closures.
    async fn collect_sandbox_inputs(
        &mut self,
        derivation: &Derivation,
        known_paths: &DerivationRegistry,
        source_paths: &[StorePath<String>],
    ) -> Result<BTreeMap<StorePath<String>, Node>, Error> {
        let direct_input_paths = collect_input_paths(derivation, known_paths)?;
        let mut closure_sets = Vec::with_capacity(direct_input_paths.len());
        for input_path in &direct_input_paths {
            closure_sets.push(self.resolve_input_closure_paths(input_path).await?);
        }
        let input_paths = merge_sandbox_input_closures(direct_input_paths, source_paths, &closure_sets);

        let mut pairs: Vec<(StorePath<String>, Node)> = Vec::with_capacity(input_paths.len());
        for input_path in &input_paths {
            let is_source = source_paths.contains(input_path);
            let host_path = self.resolve_host_path(input_path, is_source);
            let Some(node) = self.cached_or_ingested_node_for_path(input_path, &host_path).await? else {
                return Err(Error::SourceNotFound {
                    path: input_path.clone(),
                    store_dir: self.store.store_dir().to_string(),
                });
            };
            pairs.push((input_path.clone(), node));
        }
        Ok(pairs.into_iter().collect())
    }

    /// Collect (old, new) path pairs for transitive CA input rewriting.
    /// If any input derivation is CA, its provisional env path may appear
    /// in our output and needs rewriting to the final resolved path.
    fn collect_ca_input_rewrites(
        &self,
        derivation: &Derivation,
        known_paths: &DerivationRegistry,
    ) -> Vec<(String, String)> {
        let total_output_names: usize = derivation.input_derivations.values().map(|v| v.len()).sum();
        let mut rewrites: Vec<(String, String)> = Vec::with_capacity(total_output_names);
        for (input_drv_path, output_names) in &derivation.input_derivations {
            let input_abs = input_drv_path.to_absolute_path_with_prefix(self.store.store_dir());
            if let Some(entry) = known_paths.get_by_drv_path(&input_abs)
                && entry.content_addressed
            {
                for on in output_names {
                    let placeholder = entry
                        .derivation
                        .environment
                        .get(on)
                        .map(|v| String::from_utf8_lossy(v).to_string())
                        .unwrap_or_default();
                    if let Some(resolved) = entry.resolved_outputs.get(on) {
                        // CA rewrites operate in sandbox space (logical prefix).
                        let resolved_abs = resolved.to_absolute_path_with_prefix(self.store.store_dir());
                        if placeholder.len() == resolved_abs.len() {
                            rewrites.push((placeholder, resolved_abs));
                        }
                    }
                }
            }
        }
        rewrites
    }

    /// Process a single build output: apply rewrites, compute paths,
    /// verify FOD hash, create PathInfo, persist, and export to disk.
    #[allow(clippy::too_many_arguments)]
    #[allow(tigerstyle::too_many_parameters)] // output pipeline threading derivation context through stages
    async fn process_output(
        &mut self,
        drv_path: &StorePath<String>,
        drv_name: &str,
        output_name: &str,
        output: &nix_compat::derivation::Output,
        build_output: &snix_build::buildservice::BuildOutput,
        input_rewrites: &[(String, String)],
        sandbox_inputs: &BTreeMap<StorePath<String>, Node>,
        refscan_needles: &[String],
        derivation: &Derivation,
        known_paths: &mut DerivationRegistry,
        is_ca: bool,
        artifact_provenance: &ArtifactProvenance,
        is_root: bool,
    ) -> Result<PathInfo, Error> {
        let working_node = apply_input_rewrites(&build_output.node, input_rewrites, &self.store).await?;
        let resolved = self
            .resolve_output_node(drv_path, drv_name, output_name, output, &working_node, derivation, known_paths, is_ca)
            .await?;
        self.verify_output_hash_if_needed(
            drv_name,
            output_name,
            output,
            resolved.final_nar_size,
            &resolved.final_nar_sha256,
            &resolved.final_node,
        )
        .await?;
        let references = resolve_references(&build_output.output_needles, refscan_needles, derivation, sandbox_inputs);
        self.persist_and_export_output(
            drv_path,
            output_name,
            &resolved.output_path,
            resolved.final_node,
            references,
            resolved.final_nar_size,
            resolved.final_nar_sha256,
            resolved.ca,
            Some(artifact_provenance.clone()),
            is_root,
        )
        .await
    }

    /// Resolve the final output path, node, and NAR hash for either a
    /// CA or input-addressed output.
    #[allow(clippy::too_many_arguments)]
    #[allow(tigerstyle::too_many_parameters)] // subset of process_output context
    async fn resolve_output_node(
        &mut self,
        drv_path: &StorePath<String>,
        drv_name: &str,
        output_name: &str,
        output: &nix_compat::derivation::Output,
        working_node: &Node,
        derivation: &Derivation,
        known_paths: &mut DerivationRegistry,
        is_ca: bool,
    ) -> Result<ResolvedOutput, Error> {
        if is_ca {
            return self
                .compute_ca_output(drv_path, drv_name, output_name, working_node, derivation, known_paths)
                .await;
        }

        let output_path = output
            .path
            .as_ref()
            .ok_or_else(|| Error::OutputNoPath {
                output: output_name.to_string(),
                drv_name: drv_name.to_string(),
            })?
            .clone();
        let (final_nar_size, final_nar_sha256) = self
            .store
            .calculate_nar(working_node)
            .await
            .map_err(|error| Error::NarCalculation(error.to_string()))?;
        Ok(ResolvedOutput {
            output_path,
            final_node: working_node.clone(),
            final_nar_size,
            final_nar_sha256,
            ca: output.ca_hash.clone(),
        })
    }

    /// Verify a fixed-output hash when the output declares one.
    #[allow(tigerstyle::too_many_parameters)] // FOD verification threading hash context
    async fn verify_output_hash_if_needed(
        &self,
        drv_name: &str,
        output_name: &str,
        output: &nix_compat::derivation::Output,
        nar_size: u64,
        nar_sha256: &[u8; 32],
        final_node: &Node,
    ) -> Result<(), Error> {
        let Some(ca_hash) = &output.ca_hash else {
            return Ok(());
        };
        verify_fod_hash(drv_name, output_name, ca_hash, nar_size, nar_sha256, final_node, &self.store).await
    }

    /// Compute the final output path and node for a CA derivation output.
    ///
    /// Performs self-reference rewriting:
    /// 1. Replace provisional path with zero marker
    /// 2. Hash the marker-replaced NAR for the content address
    /// 3. Compute the CA store path
    /// 4. Replace zero markers with the final path
    #[allow(tigerstyle::too_many_parameters)] // CA self-reference rewrite pipeline threading derivation context
    async fn compute_ca_output(
        &mut self,
        drv_path: &StorePath<String>,
        drv_name: &str,
        output_name: &str,
        working_node: &Node,
        derivation: &Derivation,
        known_paths: &mut DerivationRegistry,
    ) -> Result<ResolvedOutput, Error> {
        // 1. Get the provisional (input-addressed) path from the env.
        let provisional = derivation
            .environment
            .get(output_name)
            .map(|v| String::from_utf8_lossy(v).to_string())
            .unwrap_or_default();
        let provisional_bytes = provisional.as_bytes();
        let marker = crate::ca_plan::generate_ca_marker(output_name, provisional_bytes.len());

        // 2. Replace provisional with zero marker.
        let (marked_node, _has_self_refs) = self
            .store
            .rewrite_node(working_node, provisional_bytes, &marker)
            .await
            .map_err(|error| Error::Store(format!("rewriting provisional CA path: {error}")))?;

        // 3. Compute NAR hash of marker-replaced content (canonical form).
        let (marker_nar_size, marker_nar_sha256) = self
            .store
            .calculate_nar(&marked_node)
            .await
            .map_err(|error| Error::NarCalculation(error.to_string()))?;

        // 4. Compute CA store path from marker-replaced hash.
        let path_name = crate::ca_plan::ca_output_path_name(drv_name, output_name);
        let ca_path = crate::ca_plan::compute_ca_store_path(&path_name, marker_nar_sha256, self.store.store_dir())?;

        // 5. Replace zero markers with the final CA path (in sandbox/logical space).
        let final_abs = ca_path.to_absolute_path_with_prefix(self.store.store_dir());
        let final_bytes = final_abs.as_bytes();
        let final_node = if marker.len() == final_bytes.len() {
            let (node, _) = self
                .store
                .rewrite_node(&marked_node, &marker, final_bytes)
                .await
                .map_err(|error| Error::Store(format!("rewriting final CA path: {error}")))?;
            node
        } else {
            marked_node
        };

        let facts = self.measure_final_ca_path_info_facts(&final_node, marker_nar_size, marker_nar_sha256).await?;

        let drv_abs = drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        self.register_ca_output(&drv_abs, drv_name, output_name, &ca_path, known_paths)?;

        Ok(ResolvedOutput {
            output_path: ca_path,
            final_node,
            final_nar_size: facts.final_nar_size,
            final_nar_sha256: facts.final_nar_sha256,
            ca: Some(facts.path_identity_ca),
        })
    }

    /// Register a resolved CA output path, persist the CA mapping, and log it.
    fn register_ca_output(
        &mut self,
        drv_abs: &str,
        drv_name: &str,
        output_name: &str,
        ca_path: &StorePath<String>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        known_paths.resolve_output(drv_abs, output_name, ca_path.clone())?;
        let final_abs = ca_path.to_absolute_path_with_prefix(self.store.store_dir());
        self.store.insert_ca_mapping(drv_abs, output_name, &final_abs);
        let display_abs = ca_path.to_absolute_path_with_prefix(self.store.output_dir_str());
        info!(drv = %drv_name, output = %output_name, ca_path = %display_abs, "CA output path resolved");
        Ok(())
    }

    /// Build PathInfo, sign it, then delegate persistence to `BuildStore`.
    #[allow(clippy::too_many_arguments)]
    #[allow(tigerstyle::too_many_parameters)] // PathInfo assembly threading hash + reference context
    async fn persist_and_export_output(
        &mut self,
        drv_path: &StorePath<String>,
        output_name: &str,
        output_path: &StorePath<String>,
        final_node: Node,
        references: Vec<StorePath<String>>,
        nar_size: u64,
        nar_sha256: [u8; 32],
        ca: Option<nix_compat::nixhash::CAHash>,
        provenance: Option<ArtifactProvenance>,
        is_root: bool,
    ) -> Result<PathInfo, Error> {
        // Build the PathInfo locally so we can sign it before handing
        // it to the store for persistence.
        let mut path_info = PathInfo {
            store_path: output_path.clone(),
            node: final_node.clone(),
            references,
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: Some(path_info_deriver(drv_path)?),
            ca,
        };

        signing::sign_pathinfo_with_store_dir(&mut path_info, &self.keypair.signing_key, self.store.store_dir());

        self.store
            .persist_and_export_signed_output(crunch_store::PersistOutputRequest {
                output_name,
                output_path,
                path_info,
                final_node,
                provenance,
                is_root,
                root_source: self.root_retention_source,
            })
            .await
            .map_err(|e| Error::Store(format!("{e}")))
    }

    async fn check_shared_action_result(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        known_paths: &mut DerivationRegistry,
        is_root: bool,
    ) -> Result<Option<CacheCheckHit>, Error> {
        let collection = self.collect_shared_action_candidates(derivation).await?;
        let policy_refs = policy_refs_for_derivation(derivation, self.store.store_dir(), self.hermeticity_mode)
            .map_err(Error::Store)?;
        let request = StrongReuseRequest {
            action_ref: collection.action_ref.clone(),
            output_names: derivation.outputs.keys().cloned().collect(),
            policy: trust_policy_for_action(&policy_refs, &self.trusted_keys),
        };
        let plan = plan_strong_reuse(request, collection.candidates).map_err(Error::Store)?;
        let selected_source = plan
            .selected_result_ref
            .as_ref()
            .and_then(|result_ref| collection.source_by_result_ref.get(result_ref).cloned());
        let disposition = shared_action_disposition(&plan);
        let selected_result_ref = plan.selected_result_ref.clone();
        let conflict_class = plan.conflict_class.clone();
        let selected_probe = selected_result_ref
            .as_ref()
            .and_then(|result_ref| collection.probes_by_result_ref.get(result_ref).cloned());
        let transfer = selected_probe.as_ref().map(action_result_transfer_evidence).transpose()?;
        self.action_result_reports.push(discovery_runtime_report(
            collection.action_ref,
            disposition,
            plan,
            selected_source,
            transfer,
            collection.diagnostics,
        ));
        if let Some(conflict) = conflict_class {
            return Err(Error::Store(format!("{conflict}: strong shared action-result reuse rejected")));
        }
        if selected_result_ref.is_none() {
            return Ok(None);
        }
        let infos = selected_probe
            .map(|probe| probe.outputs)
            .ok_or_else(|| Error::Store("selected action result has no admitted outputs".to_string()))?;
        let remote_result_source = self.root_retention_source.map(GcRootSource::for_remote_result);
        self.store
            .admit_action_result_outputs(&infos, is_root, remote_result_source)
            .await
            .map_err(|error| Error::Store(format!("admitting shared action result: {error}")))?;
        self.persist_shared_ca_mapping(drv_path, derivation, &infos);
        self.record_cached_output_paths(drv_path, derivation, &infos, known_paths)?;
        Ok(Some(CacheCheckHit {
            infos,
            substitutions: BTreeMap::new(),
        }))
    }

    async fn collect_shared_action_candidates(&self, derivation: &Derivation) -> Result<SharedActionCandidates, Error> {
        let action_ref = action_ref_for_derivation(derivation, self.store.store_dir());
        let discovery = self.action_results.discover(&action_ref).await;
        let candidate_slots = discovery.lookups.iter().try_fold(0usize, |count, lookup| {
            count
                .checked_add(lookup.records.len())
                .ok_or_else(|| Error::Store("action-result candidate count overflow".to_string()))
        })?;
        let mut candidates = Vec::with_capacity(candidate_slots);
        let mut probes_by_result_ref = BTreeMap::new();
        let mut source_by_result_ref = BTreeMap::new();
        for lookup in discovery.lookups {
            for signed_record in lookup.records {
                let result_ref = signed_record.record.result_ref.clone();
                let probe = self.action_results.probe_outputs(&signed_record.record).await.ok();
                let facts = candidate_admission_facts(
                    lookup.source_id.clone(),
                    lookup.source_class.clone(),
                    &signed_record,
                    derivation,
                    probe.as_ref().map(|probe| &probe.outputs),
                    self.store.store_dir(),
                    self.hermeticity_mode,
                    &self.trusted_keys,
                );
                if let Some(probe) = probe {
                    probes_by_result_ref.entry(result_ref.clone()).or_insert(probe);
                }
                source_by_result_ref
                    .entry(result_ref)
                    .or_insert((lookup.source_id.clone(), lookup.source_class.clone()));
                candidates.push(DiscoveredActionResultCandidate { signed_record, facts });
            }
        }
        Ok(SharedActionCandidates {
            action_ref,
            candidates,
            probes_by_result_ref,
            source_by_result_ref,
            diagnostics: discovery.diagnostics,
        })
    }

    fn persist_shared_ca_mapping(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        infos: &BTreeMap<String, PathInfo>,
    ) {
        let is_ca = derivation.outputs.values().all(|output| output.path.is_none() && output.ca_hash.is_none());
        if !is_ca {
            return;
        }
        let drv_abs = drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        for (output_name, path_info) in infos {
            let output_abs = path_info.store_path.to_absolute_path_with_prefix(self.store.store_dir());
            self.store.insert_ca_mapping(&drv_abs, output_name, &output_abs);
        }
    }

    async fn publish_completed_action_result(&mut self, derivation: &Derivation, outputs: &BTreeMap<String, PathInfo>) {
        let signed = match signed_record_for_outputs(
            derivation,
            outputs,
            self.store.store_dir(),
            self.hermeticity_mode,
            &self.keypair,
        ) {
            Ok(signed) => signed,
            Err(error) => {
                tracing::warn!(error = %error, "shared action-result record construction failed");
                return;
            }
        };
        let diagnostics = match self.action_results.publish_local(&signed).await {
            Ok(_) => Vec::new(),
            Err(error) => {
                tracing::warn!(result_ref = %signed.record.result_ref, error = %error, "shared action-result publication failed");
                vec![error]
            }
        };
        self.action_result_reports.push(publication_runtime_report(
            signed.record.action_ref,
            signed.record.result_ref,
            diagnostics,
        ));
    }

    /// Check cache: every output must have PathInfo AND exist on disk.
    /// For CA derivations, uses ca_mappings to find the resolved path.
    /// Delegates to `self.store.check_cache()`, then verifies signatures.
    async fn check_cache(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        is_root: bool,
    ) -> Result<Option<CacheCheckHit>, Error> {
        let cached = self
            .store
            .check_cache(drv_path, derivation, is_root, self.root_retention_source)
            .await
            .map_err(|e| Error::Store(format!("{e}")))?;

        let Some(cached_infos) = cached else {
            return Ok(None);
        };
        let infos: BTreeMap<String, PathInfo> = cached_infos.into_iter().collect();

        let substitutions = infos
            .iter()
            .filter_map(|(output_name, path_info)| {
                self.store
                    .take_output_substitution_report(&path_info.store_path)
                    .map(|report| (output_name.clone(), report))
            })
            .collect::<BTreeMap<_, _>>();

        if self.trust_unsigned || self.store.is_overlay_composed() {
            // Overlay composition verifies the selected PathInfo against the
            // selected layer's accepted keys before this boundary.
            return Ok(Some(CacheCheckHit { infos, substitutions }));
        }

        // Verify signatures on every single-store cached output.
        for (output_name, path_info) in &infos {
            let result = signing::verify_pathinfo_signatures_with_store_dir(
                path_info,
                &self.trusted_keys,
                self.store.store_dir(),
            );
            if !result.is_trusted() {
                if result.total_sigs == 0 {
                    tracing::warn!(
                        drv = %drv_path.name(),
                        output = %output_name,
                        "cache hit rejected: unsigned PathInfo (run `crunch store sign --all` to fix)"
                    );
                } else {
                    tracing::warn!(
                        drv = %drv_path.name(),
                        output = %output_name,
                        untrusted = ?result.untrusted_names,
                        "cache hit rejected: no trusted signature"
                    );
                }
                return Ok(None);
            }
        }

        Ok(Some(CacheCheckHit { infos, substitutions }))
    }

    fn record_cached_output_paths(
        &self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        infos: &BTreeMap<String, PathInfo>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        let is_ca = derivation.outputs.values().all(|output| output.path.is_none() && output.ca_hash.is_none());
        if !is_ca {
            return Ok(());
        }

        let drv_abs = drv_path.to_absolute_path_with_prefix(self.store.store_dir());
        for (output_name, path_info) in infos {
            known_paths.resolve_output(&drv_abs, output_name, path_info.store_path.clone())?;
        }
        Ok(())
    }

    /// Collect build-input provenance for native attestations.
    fn build_artifact_provenance(
        &self,
        derivation: &Derivation,
        known_paths: &DerivationRegistry,
    ) -> Result<ArtifactProvenance, Error> {
        let declared_inputs = collect_input_paths(derivation, known_paths)?;
        let drv_abs = self.derivation_path_from_nix_derivation(derivation, known_paths.store_dir())?;
        let claims = known_paths.get_by_drv_path(&drv_abs).and_then(|entry| entry.provenance_claims.clone());
        let mut input_sources = Vec::with_capacity(declared_inputs.len());
        let mut input_artifacts = Vec::with_capacity(declared_inputs.len());

        for input_path in declared_inputs {
            if derivation.input_sources.contains(&input_path) {
                input_sources.push(input_path);
                continue;
            }
            input_artifacts.push(input_path);
        }

        Ok(ArtifactProvenance {
            claims,
            input_sources,
            input_artifacts,
            store_layer: StoreLayer::Overlay,
        })
    }

    fn derivation_path_from_nix_derivation(&self, derivation: &Derivation, store_dir: &str) -> Result<String, Error> {
        let name = derivation
            .environment
            .get("name")
            .map(|value| String::from_utf8_lossy(value.as_ref()).to_string())
            .ok_or_else(|| Error::Store("claims lookup requires derivation.environment.name".to_string()))?;
        let drv_path = derivation
            .calculate_derivation_path_with_store_dir(&name, store_dir)
            .map_err(|e| Error::Store(format!("calculating derivation path for claims lookup: {e}")))?;
        Ok(drv_path.to_absolute_path_with_prefix(store_dir))
    }

    /// Resolve a store path to its host filesystem location.
    /// Source inputs prefer the physical output authority, then the configured logical store.
    /// Built outputs use the physical output authority.
    fn resolve_host_path(&self, path: &StorePath<String>, is_source: bool) -> PathBuf {
        if is_source {
            self.preferred_source_host_path(path)
        } else {
            PathBuf::from(path.to_absolute_path_with_prefix(self.store.output_dir_str()))
        }
    }

    /// Make sure we have castore nodes for inputs that exist on disk
    /// (needed for passing as sandbox inputs to downstream builds).
    async fn ensure_input_nodes(&mut self, derivation: &Derivation) -> Result<(), Error> {
        for output in derivation.outputs.values() {
            let Some(path) = &output.path else {
                continue;
            };
            let abs = PathBuf::from(path.to_absolute_path_with_prefix(self.store.output_dir_str()));
            let _ = self.cached_or_ingested_node_for_path(path, &abs).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::io::Read as _;
    use std::io::Write as _;
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering as AtomicOrdering;
    use std::thread::JoinHandle;

    use async_trait::async_trait;
    use futures::stream::StreamExt;
    use nix_compat::derivation::Output;
    use nix_compat::store_path::StorePath;
    use snix_build::buildservice::BuildOutput;
    use snix_build::buildservice::BuildRequest;
    use snix_build::buildservice::BuildResult;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;
    use snix_store::pathinfoservice::{self};
    use tokio::io::AsyncWriteExt;

    use super::*;

    const DERIVER_DIGEST_BYTE: u8 = 7;

    #[test]
    fn path_info_deriver_uses_suffix_free_internal_identity() {
        let drv_path = StorePath::from_name_and_digest_fixed(
            "example-package.drv",
            [DERIVER_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
        )
        .unwrap();
        let deriver = path_info_deriver(&drv_path).unwrap();

        assert_eq!(deriver.name(), "example-package");
        assert_eq!(deriver.digest(), drv_path.digest());
    }

    #[test]
    fn path_info_deriver_removes_only_the_outer_suffix() {
        let drv_path = StorePath::from_name_and_digest_fixed(
            "dynamic-package.drv.drv",
            [DERIVER_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
        )
        .unwrap();
        let deriver = path_info_deriver(&drv_path).unwrap();

        assert_eq!(deriver.name(), "dynamic-package.drv");
        assert_eq!(deriver.digest(), drv_path.digest());
    }

    #[test]
    fn path_info_deriver_rejects_empty_name_after_suffix_removal() {
        let drv_path = StorePath::from_name_and_digest_fixed(
            DERIVATION_SUFFIX,
            [DERIVER_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
        )
        .unwrap();
        let error = path_info_deriver(&drv_path).unwrap_err();

        assert!(error.to_string().contains("deriver name is empty"));
        assert_eq!(drv_path.name(), DERIVATION_SUFFIX);
    }

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
    }

    fn test_fetch_build_service(
        blob_service: MemoryBlobService,
        directory_service: RedbDirectoryService,
    ) -> crate::fetch_build_service::FetchBuildService {
        let parts = crate::test_support::pipeline_store_parts(blob_service, directory_service);
        crate::fetch_build_service::FetchBuildService::new(parts.build_service_store)
    }

    /// Write bytes into the blob service and return the resulting
    /// Node::File. Used by cache tests to populate the castore so
    /// that `castore_has_content` finds the blob.
    async fn put_blob(bs: &MemoryBlobService, content: &[u8]) -> Node {
        let mut writer = BlobService::open_write(bs).await;
        writer.write_all(content).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: content.len() as u64,
            executable: false,
        }
    }

    fn test_pis() -> LruPathInfoService {
        LruPathInfoService::with_capacity("test".to_string(), std::num::NonZeroUsize::new(128).unwrap())
    }

    fn test_keypair() -> crate::signing::KeyPair {
        crate::signing::load_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        ).unwrap()
    }

    fn test_trusted_keys() -> Vec<nix_compat::narinfo::VerifyingKey> {
        crate::signing::build_trusted_keys(&test_keypair(), None)
    }

    async fn assert_builder_path_info_archive_round_trip(source: &crunch_store::StoreHandle, path_info: &PathInfo) {
        assert!(!path_info.signatures.is_empty());
        assert!(path_info.ca.is_some());
        let mut archive = Vec::new();
        let export_report = crunch_store::export_store_archive(
            source,
            std::slice::from_ref(path_info),
            &mut archive,
            &crunch_store::ArchiveExportOptions { trust_unsigned: false },
        )
        .await
        .unwrap();
        let destination_root = tempfile::tempdir().unwrap();
        let destination = crunch_store::StoreHandle::from_services_with_store_dir(
            crunch_store::StoreHandleServices {
                blob_service: Arc::new(MemoryBlobService::default()),
                directory_service: Arc::new(tmp_ds()),
                pathinfo_service: Arc::new(test_pis()),
                remote_pathinfo: None,
                state_dir: destination_root.path().join("state"),
                output_dir_str: destination_root.path().join("store").to_string_lossy().into_owned(),
                publishers: Vec::new(),
            },
            nix_compat::store_path::STORE_DIR.to_string(),
        );
        let mut reader = std::io::Cursor::new(archive);
        let import_report =
            crunch_store::import_store_archive(&destination, &mut reader, &crunch_store::ArchiveImportOptions {
                trust_unsigned: false,
                trusted_public_keys: test_trusted_keys(),
                materialize: false,
            })
            .await
            .unwrap();
        let persisted = destination.pathinfo_service().get(*path_info.store_path.digest()).await.unwrap().unwrap();

        assert_eq!(export_report.exported_count, 1);
        assert_eq!(import_report.imported_count, 1);
        assert_eq!(persisted, *path_info);
    }

    const STATIC_HTTP_READ_CAPACITY_BYTES: usize = 16_384;
    const STATIC_HTTP_MAX_FILES: usize = 64;
    const STATIC_HTTP_POLL_MS: u64 = 5;

    struct StaticHttpServer {
        base_url: String,
        stop: Arc<AtomicBool>,
        requests: Arc<Mutex<Vec<String>>>,
        thread: Option<JoinHandle<()>>,
    }

    impl StaticHttpServer {
        fn start(files: BTreeMap<String, Vec<u8>>) -> Self {
            assert!(!files.is_empty());
            assert!(files.len() <= STATIC_HTTP_MAX_FILES);
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let address = listener.local_addr().unwrap();
            let stop = Arc::new(AtomicBool::new(false));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let thread_stop = stop.clone();
            let thread_requests = requests.clone();
            let thread = std::thread::spawn(move || {
                while !thread_stop.load(AtomicOrdering::SeqCst) {
                    match listener.accept() {
                        Ok((stream, _)) => serve_static_http_request(stream, &files, &thread_requests),
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(std::time::Duration::from_millis(STATIC_HTTP_POLL_MS));
                        }
                        Err(_) => break,
                    }
                }
            });
            Self {
                base_url: format!("http://{address}"),
                stop,
                requests,
                thread: Some(thread),
            }
        }
    }

    impl Drop for StaticHttpServer {
        fn drop(&mut self) {
            self.stop.store(true, AtomicOrdering::SeqCst);
            let _ = TcpStream::connect(self.base_url.trim_start_matches("http://"));
            if let Some(thread) = self.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    fn serve_static_http_request(
        mut stream: TcpStream,
        files: &BTreeMap<String, Vec<u8>>,
        requests: &Arc<Mutex<Vec<String>>>,
    ) {
        let mut request = [0u8; STATIC_HTTP_READ_CAPACITY_BYTES];
        let bytes_read = stream.read(&mut request).unwrap_or(0);
        let request_text = String::from_utf8_lossy(&request[..bytes_read]);
        let path = request_text
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/")
            .split('?')
            .next()
            .unwrap_or("/")
            .to_string();
        requests.lock().unwrap().push(path.clone());
        let (status, body) =
            files.get(&path).map(|body| ("200 OK", body.as_slice())).unwrap_or(("404 Not Found", b"missing"));
        let response = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
    }

    fn collect_static_http_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            for entry in std::fs::read_dir(&path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    pending.push(entry.path());
                    continue;
                }
                let relative = entry.path().strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                files.insert(format!("/{relative}"), std::fs::read(entry.path()).unwrap());
                assert!(files.len() <= STATIC_HTTP_MAX_FILES);
            }
        }
        files
    }

    fn sign_with_test_key(path_info: &mut PathInfo) {
        let keypair = test_keypair();
        crate::signing::sign_pathinfo(path_info, &keypair.signing_key);
    }

    fn corrupt_signature_bytes(path_info: &mut PathInfo) {
        use nix_compat::narinfo::Signature;

        assert!(!path_info.signatures.is_empty(), "signature must exist before corruption");
        let bad_bytes = [0u8; 64];
        let bad_sig_str = format!("{}:{}", path_info.signatures[0].name(), data_encoding::BASE64.encode(&bad_bytes),);
        path_info.signatures[0] = Signature::<String>::parse(&bad_sig_str).unwrap();
    }

    /// A mock BuildService that records requests and returns a synthetic
    /// output node for each requested output.
    struct MockBuildService {
        calls: Arc<Mutex<Vec<Vec<String>>>>,
        blob_service: MemoryBlobService,
    }

    impl MockBuildService {
        fn new(blob_service: MemoryBlobService) -> (Self, Arc<Mutex<Vec<Vec<String>>>>) {
            let calls = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    calls: calls.clone(),
                    blob_service,
                },
                calls,
            )
        }
    }

    #[async_trait]
    impl BuildService for MockBuildService {
        async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
            self.calls.lock().unwrap().push(request.command_args.clone());

            let mut writer = BlobService::open_write(&self.blob_service).await;
            writer.write_all(b"mock output").await.unwrap();
            let digest = writer.close().await.unwrap();

            let outputs: Vec<BuildOutput> = request
                .outputs
                .iter()
                .map(|_| BuildOutput {
                    node: Node::File {
                        digest,
                        size: 11,
                        executable: false,
                    },
                    output_needles: BTreeSet::new(),
                })
                .collect();

            Ok(BuildResult { outputs, log: None })
        }
    }

    struct OutputPathEchoBuildService {
        blob_service: MemoryBlobService,
        output_names: Vec<String>,
    }

    #[async_trait]
    impl BuildService for OutputPathEchoBuildService {
        async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
            assert!(!self.output_names.is_empty());
            assert_eq!(request.outputs.len(), self.output_names.len());
            let mut output_names = self.output_names.clone();
            output_names.sort();
            let mut outputs = Vec::with_capacity(output_names.len());
            for output_name in output_names {
                let output_value = request
                    .environment_vars
                    .iter()
                    .find(|variable| variable.key == output_name)
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("missing {output_name} environment"),
                        )
                    })?
                    .value
                    .clone();
                let mut writer = BlobService::open_write(&self.blob_service).await;
                writer.write_all(&output_value).await?;
                let digest = writer.close().await?;
                outputs.push(BuildOutput {
                    node: Node::File {
                        digest,
                        size: u64::try_from(output_value.len()).map_err(std::io::Error::other)?,
                        executable: false,
                    },
                    output_needles: BTreeSet::new(),
                });
            }
            Ok(BuildResult { outputs, log: None })
        }
    }

    struct ErrorBuildService;

    #[async_trait]
    impl BuildService for ErrorBuildService {
        async fn do_build(&self, _request: BuildRequest) -> std::io::Result<BuildResult> {
            Err(std::io::Error::other("controlled executor failure"))
        }
    }

    #[derive(Default, Clone)]
    struct CountingPathInfoService {
        entries: Arc<Mutex<HashMap<[u8; 20], PathInfo>>>,
        get_counts: Arc<Mutex<HashMap<[u8; 20], u32>>>,
    }

    impl CountingPathInfoService {
        fn insert(&self, path_info: PathInfo) {
            self.entries.lock().unwrap().insert(*path_info.store_path.digest(), path_info);
        }

        fn get_count(&self, path: &StorePath<String>) -> u32 {
            self.get_counts.lock().unwrap().get(path.digest()).copied().unwrap_or(0)
        }
    }

    #[async_trait]
    impl PathInfoService for CountingPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
            let mut counts = self.get_counts.lock().unwrap();
            let next_count = counts.get(&digest).copied().unwrap_or(0).saturating_add(1);
            counts.insert(digest, next_count);
            drop(counts);
            Ok(self.entries.lock().unwrap().get(&digest).cloned())
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
            self.entries.lock().unwrap().insert(*path_info.store_path.digest(), path_info.clone());
            Ok(path_info)
        }

        fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
            let entries: Vec<_> = self.entries.lock().unwrap().values().cloned().collect();
            futures::stream::iter(entries.into_iter().map(Ok)).boxed()
        }
    }

    fn make_source_path(name: &str, digest_byte: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [digest_byte; 20]).unwrap()
    }

    fn make_source_path_info(path: &StorePath<String>, references: Vec<StorePath<String>>) -> PathInfo {
        PathInfo {
            store_path: path.clone(),
            node: Node::Symlink {
                target: SymlinkTarget::try_from("source-target").unwrap(),
            },
            references,
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[test]
    fn build_outcome_outputs_iterate_by_output_name() {
        const Z_OUTPUT_DIGEST_BYTE: u8 = 1;
        const A_OUTPUT_DIGEST_BYTE: u8 = 2;
        const M_OUTPUT_DIGEST_BYTE: u8 = 3;
        const DRV_DIGEST_BYTE: u8 = 4;
        let mut outputs = BTreeMap::new();
        let z_path = make_source_path("z-output", Z_OUTPUT_DIGEST_BYTE);
        let a_path = make_source_path("a-output", A_OUTPUT_DIGEST_BYTE);
        let m_path = make_source_path("m-output", M_OUTPUT_DIGEST_BYTE);
        outputs.insert("zzz".to_string(), make_source_path_info(&z_path, vec![]));
        outputs.insert("aaa".to_string(), make_source_path_info(&a_path, vec![]));
        outputs.insert("mid".to_string(), make_source_path_info(&m_path, vec![]));

        let outcome = BuildOutcome {
            drv_path: make_source_path("drv", DRV_DIGEST_BYTE),
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };
        let output_names: Vec<&str> = outcome.outputs.keys().map(String::as_str).collect();
        let insertion_order = vec!["zzz", "aaa", "mid"];
        let sorted_order = vec!["aaa", "mid", "zzz"];

        assert_eq!(output_names, sorted_order);
        assert_ne!(output_names, insertion_order);
    }

    /// Build a nix_compat::Derivation, compute paths, register in DerivationRegistry.
    fn build_and_register(
        name: &str,
        input_drvs: &[(StorePath<String>, &str)],
        kp: &mut crate::registry::DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        build_and_register_with_workspace(name, input_drvs, kp, None)
    }

    fn build_and_register_with_workspace(
        name: &str,
        input_drvs: &[(StorePath<String>, &str)],
        kp: &mut crate::registry::DerivationRegistry,
        workspace_policy_json: Option<String>,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        if let Some(policy) = workspace_policy_json {
            environment.insert(crate::build_request::WORKSPACE_POLICY_ENV.to_string(), policy.into());
        }

        let mut input_derivations = BTreeMap::new();
        for (dp, on) in input_drvs {
            input_derivations.entry(dp.clone()).or_insert_with(BTreeSet::new).insert(on.to_string());
        }

        let mut drv = Derivation {
            arguments: vec!["-c".into(), format!("echo {name} > $out")],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations,
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = drv.hash_derivation_modulo(|parent_path| {
            kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
        });
        drv.calculate_output_paths(name, &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path(name).unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in name.bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);

        (drv_path, drv)
    }

    #[tokio::test]
    async fn source_closure_resolution_is_memoized_per_session() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let pis = CountingPathInfoService::default();

        let root = make_source_path("shared-source", 41);
        let dep = make_source_path("shared-source-ref", 42);
        pis.insert(make_source_path_info(&dep, vec![]));
        pis.insert(make_source_path_info(&root, vec![dep.clone()]));

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis.clone(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let first = builder.resolve_input_closure_paths(&root).await.unwrap();
        let second = builder.resolve_input_closure_paths(&root).await.unwrap();

        assert_eq!(first, second, "memoized closure must match the first resolution");
        assert_eq!(pis.get_count(&root), 1, "root closure should be queried once");
        assert_eq!(pis.get_count(&dep), 1, "transitive reference should be queried once");
        assert_eq!(builder.source_closure_cache.len(), 1, "session should cache one source root");
    }

    #[test]
    fn sandbox_input_merge_includes_transitive_dependency_references() {
        const DIRECT_DIGEST_BYTE: u8 = 51;
        const TRANSITIVE_DIGEST_BYTE: u8 = 52;
        let direct = make_source_path("direct-input", DIRECT_DIGEST_BYTE);
        let transitive = make_source_path("transitive-reference", TRANSITIVE_DIGEST_BYTE);

        let merged = merge_sandbox_input_closures(BTreeSet::from([direct.clone()]), &[], &[vec![
            direct.clone(),
            transitive.clone(),
        ]]);

        assert!(merged.contains(&direct));
        assert!(merged.contains(&transitive));
    }

    #[test]
    fn sandbox_input_merge_keeps_direct_input_when_closures_are_empty() {
        const DIRECT_DIGEST_BYTE: u8 = 53;
        let direct = make_source_path("direct-input", DIRECT_DIGEST_BYTE);

        let merged = merge_sandbox_input_closures(BTreeSet::from([direct.clone()]), &[], &[]);

        assert_eq!(merged, BTreeSet::from([direct]));
    }

    #[test]
    fn source_closure_disposition_only_reuses_current_session_outputs() {
        assert_eq!(source_closure_disposition(true), SourceClosureDisposition::ReuseCurrentSessionOutput);
        assert_eq!(source_closure_disposition(false), SourceClosureDisposition::Resolve);
    }

    #[tokio::test]
    async fn physical_store_source_with_pathinfo_resolves_transitive_closure() {
        const IMPORTED_SOURCE_DIGEST_BYTE: u8 = 47;
        const IMPORTED_REFERENCE_DIGEST_BYTE: u8 = 48;

        let bs = MemoryBlobService::default();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let physical_store = tempfile::tempdir().unwrap();
        let pis = CountingPathInfoService::default();
        let source = make_source_path("cache-imported-source", IMPORTED_SOURCE_DIGEST_BYTE);
        let reference = make_source_path("cache-imported-reference", IMPORTED_REFERENCE_DIGEST_BYTE);
        pis.insert(make_source_path_info(&reference, vec![]));
        pis.insert(make_source_path_info(&source, vec![reference.clone()]));

        let physical_source =
            PathBuf::from(source.to_absolute_path_with_prefix(physical_store.path().to_str().unwrap()));
        std::fs::write(&physical_source, b"cache-imported source").unwrap();

        let mut builder = Builder::new(
            bs,
            tmp_ds(),
            mock,
            pis.clone(),
            physical_store.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut registry = DerivationRegistry::default();
        let (_drv_path, mut derivation) = build_and_register("cache-imported-source-consumer", &[], &mut registry);
        derivation.input_sources.insert(source.clone());

        let first_sources = builder.resolve_and_ingest_sources(&derivation).await.unwrap();
        let source_queries_after_first = pis.get_count(&source);
        let reference_queries_after_first = pis.get_count(&reference);
        let repeated_sources = builder.resolve_and_ingest_sources(&derivation).await.unwrap();

        assert_eq!(first_sources, vec![source.clone(), reference.clone()]);
        assert_eq!(repeated_sources, first_sources, "ingested imports must keep their complete closure");
        assert!(source_queries_after_first > 0, "imported source PathInfo must be queried");
        assert!(reference_queries_after_first > 0, "imported runtime reference PathInfo must be queried");
        assert_eq!(pis.get_count(&source), source_queries_after_first, "repeat use must reuse source facts");
        assert_eq!(
            pis.get_count(&reference),
            reference_queries_after_first,
            "repeat use must reuse runtime reference facts"
        );
    }

    #[tokio::test]
    async fn strict_physical_store_source_without_pathinfo_fails_closed() {
        const MISSING_PATHINFO_DIGEST_BYTE: u8 = 49;

        let bs = MemoryBlobService::default();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let physical_store = tempfile::tempdir().unwrap();
        let pis = CountingPathInfoService::default();
        let source = make_source_path("physical-source-without-pathinfo", MISSING_PATHINFO_DIGEST_BYTE);
        let physical_source =
            PathBuf::from(source.to_absolute_path_with_prefix(physical_store.path().to_str().unwrap()));
        std::fs::write(&physical_source, b"unbound physical source").unwrap();

        let mut builder = Builder::new(
            bs,
            tmp_ds(),
            mock,
            pis.clone(),
            physical_store.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        builder.set_hermeticity_mode(HermeticityMode::Strict);
        let mut registry = DerivationRegistry::default();
        let (_drv_path, mut derivation) = build_and_register("strict-source-consumer", &[], &mut registry);
        derivation.input_sources.insert(source.clone());

        let error = builder.resolve_and_ingest_sources(&derivation).await.unwrap_err().to_string();

        assert!(error.contains("missing closure facts"), "strict source must require PathInfo: {error}");
        assert_eq!(pis.get_count(&source), 1, "strict source PathInfo must be queried once");
    }

    #[tokio::test]
    async fn current_session_source_skips_closure_resolution() {
        const CURRENT_SESSION_SOURCE_DIGEST_BYTE: u8 = 50;

        let bs = MemoryBlobService::default();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let physical_store = tempfile::tempdir().unwrap();
        let pis = CountingPathInfoService::default();
        let source = make_source_path("current-session-source", CURRENT_SESSION_SOURCE_DIGEST_BYTE);

        let mut builder = Builder::new(
            bs,
            tmp_ds(),
            mock,
            pis.clone(),
            physical_store.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        builder.set_hermeticity_mode(HermeticityMode::Strict);
        let source_path_info = make_source_path_info(&source, vec![]);
        builder.store.insert_output_node(source.clone(), source_path_info.node.clone());
        let physical_source_path = source.to_absolute_path_with_prefix(builder.store.output_dir_str());
        builder.store.insert_built_output(physical_source_path, source_path_info);
        let mut registry = DerivationRegistry::default();
        let (_drv_path, mut derivation) = build_and_register("current-session-source-consumer", &[], &mut registry);
        derivation.input_sources.insert(source.clone());

        let sources = builder.resolve_and_ingest_sources(&derivation).await.unwrap();

        assert_eq!(sources, vec![source.clone()]);
        assert_eq!(pis.get_count(&source), 0, "current-session source must not query PathInfo");
    }

    #[tokio::test]
    async fn collect_sandbox_inputs_reuses_pathinfo_node_before_disk_ingest() {
        use snix_store::pathinfoservice::PathInfoService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let output_dir = tempfile::tempdir().unwrap();
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (dep_drv_path, dep_drv) = build_and_register("reuse-dep", &[], &mut kp);
        let (_root_drv_path, root_drv) = build_and_register("reuse-root", &[(dep_drv_path.clone(), "out")], &mut kp);
        let dep_output_path = dep_drv.outputs["out"].path.as_ref().unwrap().clone();

        let cached_node = put_blob(&bs, b"cached dependency output").await;
        let path_info = PathInfo {
            store_path: dep_output_path.clone(),
            node: cached_node.clone(),
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(dep_drv_path.clone()),
            ca: None,
        };
        pis.put(path_info).await.unwrap();

        let disk_path =
            PathBuf::from(dep_output_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
        std::fs::create_dir_all(disk_path.parent().unwrap()).unwrap();
        std::fs::write(&disk_path, b"disk fallback should stay unused").unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let sandbox_inputs = builder.collect_sandbox_inputs(&root_drv, &kp, &[]).await.unwrap();

        assert_eq!(sandbox_inputs.get(&dep_output_path), Some(&cached_node));
        assert_eq!(builder.store.output_node(&dep_output_path), Some(&cached_node));
    }

    #[tokio::test]
    async fn failed_build_never_publishes_an_action_result() {
        use crunch_store::ActionResultStore as _;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let mut registry = DerivationRegistry::default();
        let (drv_path, derivation) = build_and_register("failed-action-result", &[], &mut registry);
        let action_ref = action_ref_for_derivation(&derivation, nix_compat::store_path::STORE_DIR);
        let mut builder = Builder::with_state_dir(
            Arc::new(MemoryBlobService::default()),
            Arc::new(tmp_ds()),
            ErrorBuildService,
            Arc::new(test_pis()),
            output_dir.path().to_path_buf(),
            Some(state_dir.path().to_path_buf()),
            None,
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let result = builder.build(&drv_path, &mut registry).await;
        let local_results = crunch_store::LocalActionResultStore::new(state_dir.path());
        let lookup = local_results.lookup(&action_ref).await.unwrap();

        assert!(result.is_err());
        assert!(lookup.records.is_empty());
        assert!(lookup.index.result_refs.is_empty());
        assert!(!builder.take_action_result_reports().iter().any(|report| report.phase == "publication"));
    }

    #[tokio::test]
    async fn builder_single_drv_calls_do_build_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _) = build_and_register("solo", &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        assert_eq!(outcome.outputs.len(), 1);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build exactly once");
    }

    #[tokio::test]
    async fn builder_chain_builds_dep_first() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_path, _) = build_and_register("dep", &[], &mut kp);
        let (top_path, _) = build_and_register("top", &[(dep_path.clone(), "out")], &mut kp);

        let outcome = builder.build(&top_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "should build dep + top");
        assert!(recorded[0].iter().any(|a| a.contains("dep")), "first build should be dep: {:?}", recorded[0]);
        assert!(recorded[1].iter().any(|a| a.contains("top")), "second build should be top: {:?}", recorded[1]);
    }

    #[tokio::test]
    async fn builder_diamond_builds_shared_dep_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("aaa", &[], &mut kp);
        let (b_path, _) = build_and_register("bbb", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("ccc", &[(a_path.clone(), "out")], &mut kp);
        let (d_path, _) = build_and_register("ddd", &[(b_path.clone(), "out"), (c_path.clone(), "out")], &mut kp);

        let outcome = builder.build(&d_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 4, "should build A+B+C+D: {:?}", *recorded);
        let a_builds = recorded.iter().filter(|args| args.iter().any(|a| a.contains("aaa"))).count();
        assert_eq!(a_builds, 1, "shared dep A should build exactly once");
    }

    // ── Cache tests ────────────────────────────────────────────

    #[tokio::test]
    async fn builder_skips_build_when_castore_cached() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("cached-test", &[], &mut kp);

        // Put content into castore (no file on disk needed).
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached content").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should report as cached");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "should NOT call do_build for cached output");
    }

    #[tokio::test]
    async fn mutable_workspace_bypasses_strong_cache_discovery_and_publication() {
        use snix_store::pathinfoservice::PathInfoService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();
        let policy = crate::WorkspacePolicy {
            mode: crate::WorkspaceMode::MutableSession,
            workspace_id: Some("cargo-cache".to_string()),
            ..crate::WorkspacePolicy::default()
        };
        let mut registry = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register_with_workspace(
            "mutable-cached-test",
            &[],
            &mut registry,
            Some(serde_json::to_string(&policy).unwrap()),
        );
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"strong cache candidate").await;
        pis.put(PathInfo {
            store_path: out_path.clone(),
            node,
            references: Vec::new(),
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: Vec::new(),
            deriver: Some(drv_path.clone()),
            ca: None,
        })
        .await
        .unwrap();
        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let outcome = builder.build(&drv_path, &mut registry).await.unwrap();
        let action_result_reports = builder.take_action_result_reports();

        assert!(!outcome.cached);
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert!(
            action_result_reports.is_empty(),
            "mutable workspace execution must neither discover nor publish strong shared results"
        );
    }

    #[tokio::test]
    async fn build_persists_signed_pathinfo_that_verifies_on_reread() {
        use snix_store::pathinfoservice::PathInfoService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let pis = Arc::new(test_pis());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis.clone(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("signed-roundtrip", &[], &mut kp);
        let out_path = drv.outputs["out"].path.as_ref().unwrap().clone();
        let digest = *out_path.digest();

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        let signed_output = outcome.outputs.get("out").unwrap();
        assert_eq!(signed_output.signatures.len(), 1);

        let reread = pis.get(digest).await.unwrap().unwrap();
        assert_eq!(reread.signatures.len(), 1);

        let verify = crate::signing::verify_pathinfo_signatures(&reread, &test_trusted_keys());
        assert!(verify.is_trusted(), "persisted signature should verify on reread");
        assert_eq!(verify.trusted_count, 1);
    }

    #[tokio::test]
    async fn build_persists_attestation_with_declared_inputs() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let pis = Arc::new(test_pis()) as Arc<dyn snix_store::pathinfoservice::PathInfoService>;
        let output_dir = tempfile::tempdir().unwrap();
        let state_dir = tempfile::tempdir().unwrap();

        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            pis,
            output_dir.path().to_path_buf(),
            Some(state_dir.path().to_path_buf()),
            None,
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_drv_path, dep_drv) = build_and_register("attestation-dep", &[], &mut kp);
        let source_path = StorePath::from_name_and_digest_fixed("attestation-src", [42u8; 20]).unwrap();
        let source_disk_path =
            PathBuf::from(source_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
        std::fs::create_dir_all(source_disk_path.parent().unwrap()).unwrap();
        std::fs::write(&source_disk_path, b"source input").unwrap();

        let root_drv = {
            let mut outputs = BTreeMap::new();
            outputs.insert("out".to_string(), nix_compat::derivation::Output {
                path: None,
                ca_hash: None,
            });
            let mut environment = BTreeMap::new();
            environment.insert("name".to_string(), "attestation-root".into());
            environment.insert("system".to_string(), "x86_64-linux".into());
            environment.insert("builder".to_string(), "/bin/sh".into());
            environment.insert("out".to_string(), "".into());
            let mut input_derivations = BTreeMap::new();
            input_derivations.insert(dep_drv_path.clone(), BTreeSet::from(["out".to_string()]));
            let mut input_sources = BTreeSet::new();
            input_sources.insert(source_path.clone());
            let mut drv = Derivation {
                arguments: vec!["-c".into(), "echo root > $out".into()],
                builder: "/bin/sh".to_string(),
                environment,
                input_derivations,
                input_sources,
                outputs,
                system: "x86_64-linux".to_string(),
            };
            let hdm = drv.hash_derivation_modulo(|parent_path| {
                kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
            });
            drv.calculate_output_paths("attestation-root", &hdm).unwrap();
            let drv_path = drv.calculate_derivation_path("attestation-root").unwrap();
            kp.insert(
                drv_path.clone(),
                hdm,
                drv.clone(),
                false,
                Some(crunch_attestation::Claims {
                    supplier: Some("Example Supplier".to_string()),
                    homepage: Some("https://example.invalid/root".to_string()),
                    ..Default::default()
                }),
            );
            (drv_path, drv)
        };

        let outcome = builder.build(&root_drv.0, &mut kp).await.unwrap();
        assert!(!outcome.cached, "root should build locally");

        let root_output_path = root_drv.1.outputs["out"].path.as_ref().unwrap();
        let attestation = builder.store.get_artifact_attestation(root_output_path).await.unwrap().unwrap();
        let dep_output_path = dep_drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let source_logical_path = source_path.to_absolute_path();

        assert!(attestation.attestation.edges.iter().any(|edge| {
            edge.kind == crunch_attestation::EdgeKind::BuildInput
                && edge.to_node_id == format!("artifact:{dep_output_path}")
        }));
        assert!(attestation.attestation.edges.iter().any(|edge| {
            edge.kind == crunch_attestation::EdgeKind::FetchedFrom
                && edge.from_node_id == format!("source:{source_logical_path}")
        }));
        assert_eq!(attestation.attestation.claims.supplier.as_deref(), Some("Example Supplier"));
        assert_eq!(attestation.attestation.claims.homepage.as_deref(), Some("https://example.invalid/root"));
    }

    #[tokio::test]
    async fn corrupted_local_signature_triggers_rebuild() {
        use snix_store::pathinfoservice::PathInfoService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("corrupt-sig", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached content").await;
        let mut path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [9u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        sign_with_test_key(&mut path_info);
        corrupt_signature_bytes(&mut path_info);
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "corrupted local signature must be treated as cache miss");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "builder should rebuild after signature verification fails");
    }

    #[tokio::test]
    async fn cache_miss_when_pathinfo_but_no_castore_content() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("orphan", &[], &mut kp);

        // PathInfo references a blob digest that doesn't exist in the
        // blob service — castore content is missing.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let path_info = PathInfo {
            store_path: out_path.clone(),
            node: Node::File {
                digest: snix_castore::B3Digest::from(&[0u8; 32]),
                size: 0,
                executable: false,
            },
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should NOT be cached (castore content missing)");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    #[tokio::test]
    async fn cache_miss_when_no_pathinfo() {
        // No PathInfo in the service at all — always a miss,
        // regardless of what's on disk or in the castore.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("untracked", &[], &mut kp);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should NOT be cached (no PathInfo)");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    // ── CA derivation tests ──────────────────────────────────────

    fn build_and_register_ca(
        name: &str,
        kp: &mut crate::registry::DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        build_and_register_multi_ca(name, &["out"], kp)
    }

    fn build_and_register_multi_ca(
        name: &str,
        output_names: &[&str],
        kp: &mut crate::registry::DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        const TEST_CA_OUTPUT_COUNT_MAX: usize = 16;
        assert!(!output_names.is_empty());
        assert!(output_names.len() <= TEST_CA_OUTPUT_COUNT_MAX);
        let mut outputs = BTreeMap::new();
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("outputs".to_string(), output_names.join(" ").into());
        for output_name in output_names {
            outputs.insert((*output_name).to_string(), nix_compat::derivation::Output {
                path: None,
                ca_hash: None,
            });
            environment.insert((*output_name).to_string(), "".into());
        }

        let mut drv = Derivation {
            arguments: vec!["-c".into(), format!("echo {name} > $out")],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("CA drv has no input derivations"));
        drv.calculate_output_paths(name, &hdm).unwrap();
        for output in drv.outputs.values_mut() {
            output.path = None;
        }
        let drv_path = drv.calculate_derivation_path(name).unwrap();
        kp.insert(drv_path.clone(), hdm, drv.clone(), true, None);

        (drv_path, drv)
    }

    #[tokio::test]
    async fn cached_ca_dependency_records_resolved_output_for_later_builds() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut seed_registry = DerivationRegistry::default();
        let (seed_drv_path, _) = build_and_register_ca("cached-seed", &mut seed_registry);
        let seed_outcome = builder.build(&seed_drv_path, &mut seed_registry).await.unwrap();
        let seed_output_path = seed_outcome.outputs["out"].store_path.clone();

        let mut dependent_registry = DerivationRegistry::default();
        let (seed_drv_path_again, _) = build_and_register_ca("cached-seed", &mut dependent_registry);
        assert_eq!(seed_drv_path_again, seed_drv_path, "same CA derivation must keep same drv path");
        let (root_drv_path, _) =
            build_and_register("uses-cached-seed", &[(seed_drv_path.clone(), "out")], &mut dependent_registry);

        let root_outcome = builder.build(&root_drv_path, &mut dependent_registry).await.unwrap();
        assert!(!root_outcome.outputs.is_empty(), "dependent root must still build");

        let seed_abs = seed_drv_path.to_absolute_path();
        let recorded_output_path = dependent_registry
            .get_output_path(&seed_abs, "out")
            .expect("cached CA dependency must record resolved output path");
        assert_eq!(recorded_output_path, seed_output_path, "cached CA output path must match first build");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "only the seed build and dependent root build should dispatch");
    }

    #[tokio::test]
    async fn shared_action_result_reuses_ca_output_without_ca_mapping_or_execution() {
        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let pis = Arc::new(test_pis()) as Arc<dyn PathInfoService>;
        let keypair = test_keypair();
        let trusted_keys = test_trusted_keys();
        let mut producer_registry = DerivationRegistry::default();
        let (drv_path, _) = build_and_register_ca("shared-ca", &mut producer_registry);
        let (producer_service, producer_calls) = MockBuildService::new(bs.clone());
        let mut producer = Builder::with_state_dir(
            Arc::new(bs.clone()),
            Arc::new(ds.clone()),
            producer_service,
            pis.clone(),
            output_dir.path().to_path_buf(),
            Some(state_dir.path().to_path_buf()),
            None,
            nix_compat::store_path::STORE_DIR,
            keypair.clone(),
            trusted_keys.clone(),
            false,
            false,
        );
        let producer_outcome = producer.build(&drv_path, &mut producer_registry).await.unwrap();
        let publication_reports = producer.take_action_result_reports();
        drop(producer);
        std::fs::remove_file(state_dir.path().join("ca_mappings.json")).unwrap();

        let mut consumer_registry = DerivationRegistry::default();
        let (consumer_drv_path, _) = build_and_register_ca("shared-ca", &mut consumer_registry);
        let (consumer_service, consumer_calls) = MockBuildService::new(bs.clone());
        let mut consumer = Builder::with_state_dir(
            Arc::new(bs),
            Arc::new(ds),
            consumer_service,
            pis,
            output_dir.path().to_path_buf(),
            Some(state_dir.path().to_path_buf()),
            None,
            nix_compat::store_path::STORE_DIR,
            keypair,
            trusted_keys,
            false,
            false,
        );
        let consumer_outcome = consumer.build(&consumer_drv_path, &mut consumer_registry).await.unwrap();
        let consumer_reports = consumer.take_action_result_reports();

        assert_eq!(producer_calls.lock().unwrap().len(), 1);
        assert_eq!(consumer_calls.lock().unwrap().len(), 0);
        assert!(!producer_outcome.cached);
        assert!(consumer_outcome.cached);
        assert_eq!(producer_outcome.outputs, consumer_outcome.outputs);
        assert!(publication_reports.iter().any(|report| report.phase == "publication"));
        let reused = consumer_reports.iter().find(|report| report.disposition == "reused").unwrap();
        let transfer = reused.transfer.as_ref().unwrap();
        assert_eq!(transfer.transferred_nar_bytes, 0);
        assert!(transfer.reused_nar_bytes > 0);
    }

    // r[verify cache_substitution.shared_action_result_discovery]
    #[tokio::test]
    async fn fresh_input_addressed_client_prefers_http_action_result_and_objects_without_execution() {
        use crunch_store::ActionResultStore as _;

        let producer_state = tempfile::tempdir().unwrap();
        let producer_output = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let producer_blobs = MemoryBlobService::default();
        let producer_directories = tmp_ds();
        let keypair = test_keypair();
        let trusted_keys = test_trusted_keys();
        let (producer_service, producer_calls) = MockBuildService::new(producer_blobs.clone());
        let mut producer = Builder::with_state_dir(
            Arc::new(producer_blobs.clone()),
            Arc::new(producer_directories.clone()),
            producer_service,
            Arc::new(test_pis()),
            producer_output.path().to_path_buf(),
            Some(producer_state.path().to_path_buf()),
            None,
            nix_compat::store_path::STORE_DIR,
            keypair.clone(),
            trusted_keys.clone(),
            false,
            false,
        );
        let mut producer_registry = DerivationRegistry::default();
        let (drv_path, producer_derivation) =
            build_and_register("http-shared-input-addressed", &[], &mut producer_registry);
        assert!(producer_derivation.outputs["out"].path.is_some());
        let producer_outcome = producer.build(&drv_path, &mut producer_registry).await.unwrap();
        let output_infos = producer_outcome.outputs.values().cloned().collect::<Vec<_>>();
        let cache_export_shell = crate::test_support::store_handle(producer_blobs, producer_directories);
        crunch_store::export_paths_to_cache_dir(
            &cache_export_shell,
            &output_infos,
            cache_dir.path(),
            &crunch_store::PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();
        let local_results = crunch_store::LocalActionResultStore::new(producer_state.path());
        let producer_drv_abs = drv_path.to_absolute_path();
        let producer_derivation = &producer_registry.get_by_drv_path(&producer_drv_abs).unwrap().derivation;
        let action_ref = action_ref_for_derivation(producer_derivation, nix_compat::store_path::STORE_DIR);
        let lookup = local_results.lookup(&action_ref).await.unwrap();
        let signed = lookup.records.first().unwrap();
        write_http_action_result_sidecars(cache_dir.path(), &lookup.index, signed);
        let server = StaticHttpServer::start(collect_static_http_files(cache_dir.path()));

        let consumer_state = tempfile::tempdir().unwrap();
        let consumer_output = tempfile::tempdir().unwrap();
        let trusted_token =
            url::form_urlencoded::byte_serialize(trusted_keys[0].to_string().as_bytes()).collect::<String>();
        let mut store_config = crunch_store::StoreConfig::new(
            consumer_state.path().to_path_buf(),
            consumer_output.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR.to_string(),
        );
        store_config.remote_cache_urls = vec![format!("{}?trusted_public_keys[0]={trusted_token}", server.base_url)];
        let consumer_store = crunch_store::StoreHandle::open(store_config).await.unwrap();
        let mut consumer = Builder::from_store_parts(
            consumer_store.into_builder_store_parts(),
            PanicSandboxService,
            keypair,
            trusted_keys,
            false,
            false,
        );
        let mut consumer_registry = DerivationRegistry::default();
        let (consumer_drv_path, consumer_derivation) =
            build_and_register("http-shared-input-addressed", &[], &mut consumer_registry);
        assert!(consumer_derivation.outputs["out"].path.is_some());
        let consumer_outcome = consumer.build(&consumer_drv_path, &mut consumer_registry).await.unwrap();
        let reports = consumer.take_action_result_reports();
        let requests = server.requests.lock().unwrap().clone();

        assert_eq!(producer_calls.lock().unwrap().len(), 1);
        assert!(consumer_outcome.cached);
        assert_eq!(consumer_outcome.outputs, producer_outcome.outputs);
        let reused = reports
            .iter()
            .find(|report| {
                report.disposition == ACTION_RESULT_DISPOSITION_REUSED
                    && report.selected_source_class.as_deref() == Some("http")
            })
            .unwrap();
        let transfer = reused.transfer.as_ref().unwrap();
        assert!(transfer.transferred_nar_bytes > 0);
        assert_eq!(transfer.reused_nar_bytes, 0);
        let index_request = requests.iter().position(|path| path.contains("/action-results/v1/indexes/")).unwrap();
        let record_request = requests.iter().position(|path| path.contains("/action-results/v1/records/")).unwrap();
        let narinfo_request = requests.iter().position(|path| path.ends_with(".narinfo")).unwrap();
        let nar_request = requests.iter().position(|path| path.starts_with("/nar/")).unwrap();
        assert!(index_request < record_request);
        assert!(record_request < narinfo_request);
        assert!(narinfo_request < nar_request);
    }

    fn write_http_action_result_sidecars(
        cache_dir: &Path,
        index: &crunch_action_result_core::ActionResultIndex,
        signed: &crunch_action_result_core::SignedActionResultRecord,
    ) {
        let action_digest = index.action_ref.strip_prefix(crunch_action_result_core::ACTION_REF_PREFIX).unwrap();
        let result_digest =
            signed.record.result_ref.strip_prefix(crunch_action_result_core::ACTION_RESULT_REF_PREFIX).unwrap();
        let index_path = cache_dir.join("action-results/v1/indexes").join(format!("{action_digest}.json"));
        let record_path = cache_dir.join("action-results/v1/records").join(format!("{result_digest}.json"));
        std::fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(record_path.parent().unwrap()).unwrap();
        std::fs::write(index_path, crunch_action_result_core::canonical_index_bytes(index).unwrap()).unwrap();
        std::fs::write(record_path, crunch_action_result_core::canonical_signed_record_bytes(signed).unwrap()).unwrap();
    }

    // r[verify store_transports.archive_export_closure]
    // r[verify store_transports.archive_import_idempotent]
    #[test]
    fn final_ca_path_info_facts_separate_path_identity_from_final_nar() {
        const MARKER_HASH_BYTE: u8 = 0xA5;
        const FINAL_HASH_BYTE: u8 = 0x5A;
        const MARKER_NAR_SIZE: u64 = 80;
        const FINAL_NAR_SIZE: u64 = 96;
        let marker_nar_sha256 = [MARKER_HASH_BYTE; 32];
        let final_nar_sha256 = [FINAL_HASH_BYTE; 32];

        let facts = final_ca_path_info_facts(MARKER_NAR_SIZE, marker_nar_sha256, FINAL_NAR_SIZE, final_nar_sha256);

        assert_eq!(facts.final_nar_size, FINAL_NAR_SIZE);
        assert_eq!(facts.final_nar_sha256, final_nar_sha256);
        assert_eq!(
            facts.path_identity_ca,
            nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(marker_nar_sha256)),
        );
        assert_ne!(marker_nar_sha256, facts.final_nar_sha256);
    }

    // r[verify store_transports.archive_export_closure]
    // r[verify store_transports.archive_import_idempotent]
    #[tokio::test]
    async fn self_referential_ca_persists_final_nar_facts_and_marker_path_identity() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let service = OutputPathEchoBuildService {
            blob_service: bs.clone(),
            output_names: vec!["out".to_string()],
        };
        let mut builder = Builder::new(
            bs.clone(),
            ds.clone(),
            service,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut known_paths = DerivationRegistry::default();
        let (drv_path, _) = build_and_register_ca("ca-self-reference", &mut known_paths);

        let outcome = builder.build(&drv_path, &mut known_paths).await.unwrap();
        let path_info = &outcome.outputs["out"];
        let final_bytes = builder.read_blob(&path_info.node).await.unwrap();
        let final_path = path_info.store_path.to_absolute_path();
        assert_eq!(final_bytes, final_path.as_bytes());
        let (final_nar_size, final_nar_sha256) = builder.store.calculate_nar(&path_info.node).await.unwrap();
        let Some(nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(path_identity_hash))) =
            &path_info.ca
        else {
            panic!("self-referential CA output must preserve marker path identity");
        };

        assert_ne!(&path_info.nar_sha256, path_identity_hash);
        assert_eq!(path_info.nar_size, final_nar_size);
        assert_eq!(path_info.nar_sha256, final_nar_sha256);
        assert_ne!(path_identity_hash, &final_nar_sha256);
        assert!(path_info.store_path.to_absolute_path().starts_with("/nix/store/"));
        let archive_shell = crate::test_support::store_handle(bs, ds);
        assert_builder_path_info_archive_round_trip(&archive_shell, path_info).await;
    }

    // r[verify store_transports.archive_export_closure]
    #[tokio::test]
    async fn multi_ca_outputs_persist_final_nar_facts_and_marker_path_identities() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let output_names = ["out", "dev"];
        let service = OutputPathEchoBuildService {
            blob_service: bs.clone(),
            output_names: output_names.iter().map(ToString::to_string).collect(),
        };
        let mut builder = Builder::new(
            bs,
            ds,
            service,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut known_paths = DerivationRegistry::default();
        let (drv_path, _) = build_and_register_multi_ca("multi-ca-self-reference", &output_names, &mut known_paths);

        let outcome = builder.build(&drv_path, &mut known_paths).await.unwrap();

        assert_eq!(outcome.outputs.len(), output_names.len());
        for path_info in outcome.outputs.values() {
            let final_bytes = builder.read_blob(&path_info.node).await.unwrap();
            let final_path = path_info.store_path.to_absolute_path();
            let (final_nar_size, final_nar_sha256) = builder.store.calculate_nar(&path_info.node).await.unwrap();
            let Some(nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(path_identity_hash))) =
                &path_info.ca
            else {
                panic!("multi-CA output must preserve marker path identity");
            };

            assert_eq!(final_bytes, final_path.as_bytes());
            assert_eq!(path_info.nar_size, final_nar_size);
            assert_eq!(path_info.nar_sha256, final_nar_sha256);
            assert_ne!(path_identity_hash, &final_nar_sha256);
        }
    }

    #[tokio::test]
    async fn ca_derivation_gets_content_based_path() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register_ca("ca-test", &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        assert_eq!(outcome.outputs.len(), 1);

        let pi = outcome.outputs.get("out").unwrap();
        let path_str = pi.store_path.to_absolute_path();
        assert!(path_str.starts_with("/nix/store/"), "CA path in store: {path_str}");
        assert!(pi.ca.is_some(), "CA PathInfo should have ca field");
    }

    #[tokio::test]
    async fn ca_identical_outputs_same_path() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock1, _) = MockBuildService::new(bs.clone());
        let mut builder1 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock1,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp1 = DerivationRegistry::default();
        let (drv_path1, _) = build_and_register_ca("ca-a", &mut kp1);
        let outcome1 = builder1.build(&drv_path1, &mut kp1).await.unwrap();

        let (mock2, _) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock2,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp2 = DerivationRegistry::default();
        let (drv_path2, _) = build_and_register_ca("ca-b", &mut kp2);
        let outcome2 = builder2.build(&drv_path2, &mut kp2).await.unwrap();

        assert_ne!(drv_path1, drv_path2, "different drvs");

        // CA paths should differ because the path name includes the drv name.
        // Both should have ca field set.
        assert!(outcome1.outputs["out"].ca.is_some());
        assert!(outcome2.outputs["out"].ca.is_some());
    }

    #[tokio::test]
    async fn ca_same_name_same_content_same_path() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (mock1, _) = MockBuildService::new(bs.clone());
        let mut builder1 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock1,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp1 = DerivationRegistry::default();
        let (drv_path1, _) = build_and_register_ca("ca-same", &mut kp1);
        let outcome1 = builder1.build(&drv_path1, &mut kp1).await.unwrap();

        let (mock2, _) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs.clone(),
            ds.clone(),
            mock2,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp2 = DerivationRegistry::default();
        let (drv_path2, _) = build_and_register_ca("ca-same", &mut kp2);
        let outcome2 = builder2.build(&drv_path2, &mut kp2).await.unwrap();

        let path1 = outcome1.outputs["out"].store_path.to_absolute_path();
        let path2 = outcome2.outputs["out"].store_path.to_absolute_path();
        assert_eq!(path1, path2, "same name + same content = same CA path");
    }

    // ── Custom output_dir tests ───────────────────────────────────

    #[tokio::test]
    async fn custom_logical_store_prefix_controls_source_lookup_and_diagnostics() {
        const CUSTOM_SOURCE_DIGEST_BYTE: u8 = 43;
        const MISSING_SOURCE_DIGEST_BYTE: u8 = 44;

        let logical_store = tempfile::tempdir().unwrap();
        let physical_store = tempfile::tempdir().unwrap();
        let logical_store_dir = logical_store.path().to_str().unwrap();
        let bs = MemoryBlobService::default();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let mut builder = Builder::new(
            bs,
            tmp_ds(),
            mock,
            test_pis(),
            physical_store.path().to_path_buf(),
            logical_store_dir,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let source_path = make_source_path("custom-prefix-source", CUSTOM_SOURCE_DIGEST_BYTE);
        let logical_source = PathBuf::from(source_path.to_absolute_path_with_prefix(logical_store_dir));
        std::fs::write(&logical_source, b"logical source").unwrap();

        assert_eq!(builder.preferred_source_host_path(&source_path), logical_source);
        assert_eq!(builder.resolve_host_path(&source_path, true), logical_source);
        assert!(builder.ensure_declared_source_exists(&source_path).await.is_ok());

        let physical_source =
            PathBuf::from(source_path.to_absolute_path_with_prefix(physical_store.path().to_str().unwrap()));
        std::fs::write(&physical_source, b"physical source").unwrap();
        assert_eq!(builder.preferred_source_host_path(&source_path), physical_source);
        assert_eq!(builder.resolve_host_path(&source_path, true), physical_source);

        let missing_path = make_source_path("missing-custom-prefix-source", MISSING_SOURCE_DIGEST_BYTE);
        let error = builder.ensure_declared_source_exists(&missing_path).await.unwrap_err().to_string();
        assert!(
            error.contains(logical_store_dir),
            "missing-source diagnostic must preserve the configured prefix: {error}"
        );
        assert!(
            !error.contains("/nix/store"),
            "missing-source diagnostic must not inject the default prefix: {error}"
        );
    }

    #[tokio::test]
    async fn custom_logical_store_prefix_controls_transitive_source_inputs() {
        const CLOSURE_SOURCE_DIGEST_BYTE: u8 = 45;
        const MISSING_CLOSURE_SOURCE_DIGEST_BYTE: u8 = 46;

        let logical_store = tempfile::tempdir().unwrap();
        let physical_store = tempfile::tempdir().unwrap();
        let logical_store_dir = logical_store.path().to_str().unwrap();
        let bs = MemoryBlobService::default();
        let (mock, _calls) = MockBuildService::new(bs.clone());
        let mut builder = Builder::new(
            bs,
            tmp_ds(),
            mock,
            test_pis(),
            physical_store.path().to_path_buf(),
            logical_store_dir,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut registry = DerivationRegistry::default();
        let (_drv_path, derivation) = build_and_register("transitive-source-consumer", &[], &mut registry);
        let closure_source = make_source_path("transitive-custom-prefix-source", CLOSURE_SOURCE_DIGEST_BYTE);
        let logical_source = PathBuf::from(closure_source.to_absolute_path_with_prefix(logical_store_dir));
        std::fs::write(&logical_source, b"transitive logical source").unwrap();

        let inputs = builder
            .collect_sandbox_inputs(&derivation, &registry, std::slice::from_ref(&closure_source))
            .await
            .unwrap();
        assert!(inputs.contains_key(&closure_source), "transitive source must become a sandbox input");
        assert!(builder.store.output_node(&closure_source).is_some(), "transitive source must be ingested");

        let missing_source =
            make_source_path("missing-transitive-custom-prefix-source", MISSING_CLOSURE_SOURCE_DIGEST_BYTE);
        let error = builder
            .collect_sandbox_inputs(&derivation, &registry, &[missing_source])
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains(logical_store_dir), "transitive-source error must preserve the prefix: {error}");
        assert!(!error.contains("/nix/store"), "transitive-source error must not inject the default prefix: {error}");
    }

    #[tokio::test]
    async fn custom_output_dir_writes_output_there() {
        // Builder with output_dir = temp dir (not /nix/store).
        // DerivationRegistry uses /nix/store (default/logical).
        // Verifies output lands in the custom dir, not /nix/store.
        let output_tmp = tempfile::tempdir().unwrap();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            output_tmp.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _) = build_and_register("custom-dir-test", &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Output should exist under the custom output dir.
        let out_info = &outcome.outputs["out"];
        let custom_abs = out_info.store_path.to_absolute_path_with_prefix(output_tmp.path().to_str().unwrap());
        assert!(PathBuf::from(&custom_abs).exists(), "output should exist at custom dir: {custom_abs}");

        // And should NOT exist at the logical /nix/store path.
        let logical_abs = out_info.store_path.to_absolute_path();
        // (Only check if /nix/store is not the output_dir, which it
        // isn't since we used a temp dir.)
        assert_ne!(output_tmp.path().to_str().unwrap(), "/nix/store", "test requires output_dir != /nix/store");
        assert!(
            !PathBuf::from(&logical_abs).exists() || logical_abs == custom_abs,
            "output should NOT exist at /nix/store: {logical_abs}"
        );
    }

    #[tokio::test]
    async fn custom_output_dir_cache_hit_via_castore() {
        // Cache hit works even with a custom output dir — castore content
        // is the authority, not disk.
        let output_tmp = tempfile::tempdir().unwrap();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("cached-custom", &[], &mut kp);

        // Put content in castore, nothing on disk.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            output_tmp.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should be cached via castore");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "no build needed for cached output");
    }

    // ── Root-only export tests ──────────────────────────────────

    #[tokio::test]
    async fn only_root_output_exported_to_disk() {
        // dep (non-root) → root. Only root's output should appear on disk.
        let output_tmp = tempfile::tempdir().unwrap();
        let output_dir = output_tmp.path().to_str().unwrap().to_string();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            output_tmp.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_path, dep_drv) = build_and_register("dep-lib", &[], &mut kp);
        let (root_path, root_drv) = build_and_register("root-app", &[(dep_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(std::slice::from_ref(&root_path), &mut kp, 1).await.unwrap();

        assert_eq!(outcomes.len(), 1);
        assert!(!outcomes[0].cached);

        // Root output should exist on disk.
        let root_out = root_drv.outputs["out"].path.as_ref().unwrap();
        let root_abs = PathBuf::from(root_out.to_absolute_path_with_prefix(&output_dir));
        assert!(root_abs.exists(), "root output should be on disk");

        // Dep output should NOT exist on disk (stays in castore).
        let dep_out = dep_drv.outputs["out"].path.as_ref().unwrap();
        let dep_abs = PathBuf::from(dep_out.to_absolute_path_with_prefix(&output_dir));
        assert!(!dep_abs.exists(), "dep output should stay in castore only");
    }

    #[tokio::test]
    async fn read_only_output_dir_still_succeeds() {
        // With a non-writable output dir, builds should still succeed
        // (output lives in castore). No panic, no error.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        // Use a path that doesn't exist and can't be created.
        let fake_dir = PathBuf::from("/nonexistent/read-only-store");

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            fake_dir,
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("ro-test", &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);
        assert!(outcome.outputs.contains_key("out"));
    }

    // ── build_all tests ─────────────────────────────────────────

    #[tokio::test]
    async fn build_all_multiple_roots_shared_dep() {
        // A (leaf) → B (root), A → C (root).
        // build_all([B, C]) should build A once, then B and C.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("shared", &[], &mut kp);
        let (b_path, _) = build_and_register("root-b", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("root-c", &[(a_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(&[b_path.clone(), c_path.clone()], &mut kp, 2).await.unwrap();

        // Should return outcomes for roots only.
        assert_eq!(outcomes.len(), 2, "should return 2 root outcomes");
        let outcome_drv_names: Vec<String> = outcomes.iter().map(|o| o.drv_path.name().to_string()).collect();
        assert!(
            outcome_drv_names.iter().any(|n| n.contains("root-b")),
            "should include root-b: {outcome_drv_names:?}"
        );
        assert!(
            outcome_drv_names.iter().any(|n| n.contains("root-c")),
            "should include root-c: {outcome_drv_names:?}"
        );

        // Shared dep 'shared' should build exactly once.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3, "should build shared + root-b + root-c: {:?}", *recorded);
        let shared_builds = recorded.iter().filter(|args| args.iter().any(|a| a.contains("shared"))).count();
        assert_eq!(shared_builds, 1, "shared dep should build once");
    }

    #[tokio::test]
    async fn build_all_empty_roots() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let outcomes = builder.build_all(&[], &mut kp, 2).await.unwrap();
        assert!(outcomes.is_empty());
    }

    #[tokio::test]
    async fn build_all_ordering_deps_before_dependents() {
        // A (leaf) → B → C (root).
        // Build order must be A, B, C.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("leaf-a", &[], &mut kp);
        let (b_path, _) = build_and_register("mid-b", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("top-c", &[(b_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(std::slice::from_ref(&c_path), &mut kp, 2).await.unwrap();

        assert_eq!(outcomes.len(), 1, "one root");
        assert!(outcomes[0].drv_path.name().contains("top-c"));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3);

        // Verify ordering: leaf-a before mid-b before top-c.
        let pos_a = recorded.iter().position(|args| args.iter().any(|a| a.contains("leaf-a"))).unwrap();
        let pos_b = recorded.iter().position(|args| args.iter().any(|a| a.contains("mid-b"))).unwrap();
        let pos_c = recorded.iter().position(|args| args.iter().any(|a| a.contains("top-c"))).unwrap();
        assert!(pos_a < pos_b, "leaf-a must build before mid-b");
        assert!(pos_b < pos_c, "mid-b must build before top-c");
    }

    #[tokio::test]
    async fn build_all_disjoint_trees() {
        // Two independent trees: A→B and C→D.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a_path, _) = build_and_register("tree1-leaf", &[], &mut kp);
        let (b_path, _) = build_and_register("tree1-root", &[(a_path.clone(), "out")], &mut kp);
        let (c_path, _) = build_and_register("tree2-leaf", &[], &mut kp);
        let (d_path, _) = build_and_register("tree2-root", &[(c_path.clone(), "out")], &mut kp);

        let outcomes = builder.build_all(&[b_path.clone(), d_path.clone()], &mut kp, 2).await.unwrap();

        assert_eq!(outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 4, "all 4 nodes should build");

        // Each leaf must build before its root.
        let pos_a = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree1-leaf"))).unwrap();
        let pos_b = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree1-root"))).unwrap();
        let pos_c = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree2-leaf"))).unwrap();
        let pos_d = recorded.iter().position(|args| args.iter().any(|a| a.contains("tree2-root"))).unwrap();
        assert!(pos_a < pos_b, "tree1-leaf before tree1-root");
        assert!(pos_c < pos_d, "tree2-leaf before tree2-root");
    }

    // ── Multi-output tests ─────────────────────────────────────────

    #[tokio::test]
    async fn multi_output_build_returns_all_outputs() {
        use crate::test_support::build_and_register_multi;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register_multi("multi-out", &["out", "dev", "lib"], &[], &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Should have PathInfo for all 3 outputs.
        assert_eq!(outcome.outputs.len(), 3, "expected 3 outputs: {:?}", outcome.outputs.keys().collect::<Vec<_>>());
        assert!(outcome.outputs.contains_key("out"));
        assert!(outcome.outputs.contains_key("dev"));
        assert!(outcome.outputs.contains_key("lib"));

        // Each output should have a distinct store path.
        let paths: Vec<String> = outcome.outputs.values().map(|pi| pi.store_path.to_absolute_path()).collect();
        assert_eq!(paths.len(), 3);
        assert_ne!(paths[0], paths[1]);
        assert_ne!(paths[1], paths[2]);
        assert_ne!(paths[0], paths[2]);

        // Build should have been called exactly once.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn multi_output_env_has_outputs_var() {
        use crate::test_support::build_and_register_multi;

        let mut kp = DerivationRegistry::default();
        let (_drv_path, drv) = build_and_register_multi("env-test", &["out", "dev", "man"], &[], &mut kp);

        // The outputs env var should list all output names.
        let outputs_env = drv.environment.get("outputs").unwrap();
        let outputs_str = std::str::from_utf8(outputs_env.as_ref()).unwrap();
        assert_eq!(outputs_str, "out dev man");

        // Each output name should have a store path env var.
        for name in &["out", "dev", "man"] {
            let val = drv.environment.get(*name).unwrap();
            let s = std::str::from_utf8(val.as_ref()).unwrap();
            assert!(s.starts_with("/nix/store/"), "${name} should be a store path: {s}");
        }
    }

    #[tokio::test]
    async fn multi_output_distinct_store_paths() {
        use crate::test_support::build_and_register_multi;

        let mut kp = DerivationRegistry::default();
        let (_drv_path, drv) = build_and_register_multi("paths-test", &["out", "dev", "lib"], &[], &mut kp);

        let out = drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let dev = drv.outputs["dev"].path.as_ref().unwrap().to_absolute_path();
        let lib = drv.outputs["lib"].path.as_ref().unwrap().to_absolute_path();

        // "out" output uses the base name; others get a suffix.
        assert!(out.ends_with("-paths-test"), "out: {out}");
        assert!(dev.ends_with("-paths-test-dev"), "dev: {dev}");
        assert!(lib.ends_with("-paths-test-lib"), "lib: {lib}");

        assert_ne!(out, dev);
        assert_ne!(dev, lib);
    }

    #[tokio::test]
    async fn multi_output_dep_mounts_all_outputs() {
        // Parent depends on a multi-output dep. All outputs should be
        // available as sandbox inputs.
        use crate::test_support::build_and_register_multi;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_path, _dep_drv) = build_and_register_multi("multi-dep", &["out", "dev"], &[], &mut kp);

        // Parent depends on both "out" and "dev" of the dep.
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "consumer".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        environment.insert("outputs".to_string(), "out".into());

        let mut input_derivations = BTreeMap::new();
        let mut dep_outputs = BTreeSet::new();
        dep_outputs.insert("out".to_string());
        dep_outputs.insert("dev".to_string());
        input_derivations.insert(dep_path.clone(), dep_outputs);

        let mut parent_drv = Derivation {
            arguments: vec!["-c".into(), "echo consumer > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations,
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = parent_drv.hash_derivation_modulo(|parent_path| {
            kp.get_hdm_by_drv_path(&parent_path.to_absolute_path()).expect("parent should be in known_paths")
        });
        parent_drv.calculate_output_paths("consumer", &hdm).unwrap();
        let parent_drv_path = parent_drv.calculate_derivation_path("consumer").unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in "consumer".bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(parent_drv_path.clone(), hdm, parent_drv.clone(), false, None);

        let outcome = builder.build(&parent_drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached);

        // Should have built the dep and the consumer (2 builds total).
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "dep + consumer: {:?}", *recorded);
    }

    #[tokio::test]
    async fn multi_output_cache_requires_all_outputs() {
        // If only some outputs are cached (PathInfo + castore),
        // the derivation is a cache miss.
        use crate::test_support::build_and_register_multi;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register_multi("partial-cache", &["out", "dev"], &[], &mut kp);

        // Put only the "out" output in castore + PathInfo. "dev" has no PathInfo.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"cached out").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService;
        pis.put(path_info).await.unwrap();

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        // Cache miss because "dev" has no PathInfo.
        assert!(!outcome.cached, "partial cache should not count as hit");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should rebuild");
    }

    // ── Remote substitution tests ─────────────────────────────────

    #[tokio::test]
    async fn remote_cache_hit_skips_build() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("remote-hit", &[], &mut kp);

        // Put content in castore + remote PathInfo.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"remote content").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 14,
            nar_sha256: [1u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService as _;
        remote_pis.put(path_info).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "should be cached via remote substitution");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "should NOT call do_build");
    }

    #[tokio::test]
    async fn remote_signed_cache_hit_verifies_and_skips_build() {
        use snix_store::pathinfoservice::PathInfoService as _;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("remote-signed-hit", &[], &mut kp);

        let remote_keypair = crate::signing::generate_keypair().0;
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"remote signed content").await;
        let mut path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 21,
            nar_sha256: [4u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        crate::signing::sign_pathinfo(&mut path_info, &remote_keypair.signing_key);
        remote_pis.put(path_info).await.unwrap();

        let trusted_keys = crate::signing::build_trusted_keys(
            &test_keypair(),
            Some(std::slice::from_ref(&remote_keypair.verifying_key)),
        );
        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            trusted_keys,
            false,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached, "signed remote substitution should verify and skip build");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 0, "should NOT call do_build when remote signature is trusted");
    }

    #[tokio::test]
    async fn unsigned_remote_path_rejected_without_trust_unsigned() {
        use snix_store::pathinfoservice::PathInfoService as _;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("remote-unsigned", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"remote unsigned content").await;
        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 23,
            nar_sha256: [5u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        remote_pis.put(path_info).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            false,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "unsigned remote path must be rejected when trust_unsigned is false");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "builder should fall through to local build");
    }

    #[tokio::test]
    async fn remote_miss_falls_through_to_build() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        // Remote is empty — no PathInfo.
        let remote_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("remote-miss", &[], &mut kp);

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should build locally on remote miss");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    #[tokio::test]
    async fn remote_hit_persisted_to_local() {
        // After a remote cache hit, the PathInfo is written to the
        // local pathinfo service. A second build should hit local.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let remote_pis = test_pis();

        // Wrap local PIS in Arc so both builders share it.
        let local_pis = Arc::new(test_pis());

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("persist-local", &[], &mut kp);

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"persist me").await;

        let path_info = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 10,
            nar_sha256: [2u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService as _;
        remote_pis.put(path_info).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let (mock1, _) = MockBuildService::new(bs.clone());
        let mut builder = Builder::with_state_dir(
            Arc::new(bs.clone()) as Arc<dyn BlobService>,
            Arc::new(ds.clone()) as Arc<dyn DirectoryService>,
            mock1,
            Arc::new(local_pis.clone()) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // First build: remote hit.
        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome.cached);

        // Second build: fresh builder with NO remote. Local should have it.
        let (mock2, calls2) = MockBuildService::new(bs.clone());
        let mut builder2 = Builder::new(
            bs,
            ds,
            mock2,
            local_pis,
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let outcome2 = builder2.build(&drv_path, &mut kp).await.unwrap();
        assert!(outcome2.cached, "should hit local cache after write-through");

        let recorded2 = calls2.lock().unwrap();
        assert_eq!(recorded2.len(), 0, "no build on second pass");
    }

    #[tokio::test]
    async fn fod_skips_remote_cache() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();
        let remote_pis = test_pis();

        // Build a FOD derivation (has ca_hash on the output).
        let mut kp = DerivationRegistry::default();
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: Some(nix_compat::nixhash::CAHash::Flat(nix_compat::nixhash::NixHash::Sha256([42u8; 32]))),
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "fod-test".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        environment.insert("outputs".to_string(), "out".into());

        let mut drv = nix_compat::derivation::Derivation {
            arguments: vec!["-c".into(), "echo fod > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let hdm = drv.hash_derivation_modulo(|_| panic!("no input drvs"));
        drv.calculate_output_paths("fod-test", &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path("fod-test").unwrap();

        let mut fake_hash = [0u8; 32];
        for (i, b) in "fod-test".bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);

        // Put PathInfo in remote for the FOD output path.
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let node = put_blob(&bs, b"fod content").await;
        let pi = PathInfo {
            store_path: out_path.clone(),
            node,
            references: vec![],
            nar_size: 11,
            nar_sha256: [3u8; 32],
            signatures: vec![],
            deriver: Some(drv_path.clone()),
            ca: None,
        };
        use snix_store::pathinfoservice::PathInfoService as _;
        remote_pis.put(pi).await.unwrap();

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(remote_pis);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // FOD should bypass remote and attempt a local build.
        // The build itself may fail (mock output won't match ca_hash)
        // but the critical assertion is that do_build was called,
        // proving substitution was skipped.
        let _result = builder.build(&drv_path, &mut kp).await;

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "FOD should trigger sandbox build, not substitution");
    }

    /// A PathInfoService that always fails on get().
    struct FailingRemoteService;

    #[async_trait]
    impl snix_store::pathinfoservice::PathInfoService for FailingRemoteService {
        async fn get(&self, _digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "simulated network error")))
        }
        async fn put(&self, _pi: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            unimplemented!()
        }
        fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn remote_error_treated_as_miss() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let local_pis = test_pis();

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = build_and_register("remote-err", &[], &mut kp);

        let remote: Arc<dyn snix_store::pathinfoservice::PathInfoService> = Arc::new(FailingRemoteService);
        let mut builder = Builder::with_state_dir(
            Arc::new(bs) as Arc<dyn BlobService>,
            Arc::new(ds) as Arc<dyn DirectoryService>,
            mock,
            Arc::new(local_pis) as Arc<dyn snix_store::pathinfoservice::PathInfoService>,
            PathBuf::from("/nix/store"),
            None,
            Some(remote),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // Remote error should be swallowed — build proceeds.
        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();
        assert!(!outcome.cached, "should build locally on remote error");

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "should call do_build");
    }

    // ── Blob persistence across Builder instances ─────────────

    /// Helper: write bytes into an ObjectStoreBlobService-backed store.
    async fn put_blob_obj(bs: &snix_castore::blobservice::ObjectStoreBlobService, content: &[u8]) -> Node {
        let mut writer = BlobService::open_write(bs).await;
        writer.write_all(content).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: content.len() as u64,
            executable: false,
        }
    }

    #[tokio::test]
    async fn persistent_blob_cache_hit_across_builder_instances() {
        use snix_castore::blobservice::ObjectStoreBlobService;
        use snix_store::pathinfoservice::PathInfoService;
        use snix_store::pathinfoservice::RedbPathInfoService;
        use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

        let tmp = tempfile::tempdir().unwrap();
        let blob_dir = tmp.path().join("blobs");
        let pis_path = tmp.path().join("pathinfo.redb");
        std::fs::create_dir_all(&blob_dir).unwrap();

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = build_and_register("persist-test", &[], &mut kp);
        let out_path = drv.outputs["out"].path.as_ref().unwrap();

        // --- First Builder instance: populate castore + pathinfo ---
        {
            let bs = std::sync::Arc::new(ObjectStoreBlobService::new_local(&blob_dir).unwrap());
            let pis = RedbPathInfoService::new("test".to_string(), RedbPathInfoServiceConfig {
                path: Some(pis_path.clone()),
                read_only: false,
                cache_size: None,
            })
            .await
            .unwrap();

            let node = put_blob_obj(&bs, b"persistent content").await;

            let path_info = PathInfo {
                store_path: out_path.clone(),
                node,
                references: vec![],
                nar_size: 0,
                nar_sha256: [0u8; 32],
                signatures: vec![],
                deriver: Some(drv_path.clone()),
                ca: None,
            };
            pis.put(path_info).await.unwrap();
            // Builder and services dropped here
        }

        // --- Second Builder instance: same dirs, fresh services ---
        {
            let bs = std::sync::Arc::new(ObjectStoreBlobService::new_local(&blob_dir).unwrap());
            let ds = tmp_ds();
            let (mock, calls) = MockBuildService::new(MemoryBlobService::default());
            let pis = RedbPathInfoService::new("test".to_string(), RedbPathInfoServiceConfig {
                path: Some(pis_path.clone()),
                read_only: false,
                cache_size: None,
            })
            .await
            .unwrap();

            let mut builder = Builder::new(
                bs,
                ds,
                mock,
                pis,
                PathBuf::from("/nix/store"),
                nix_compat::store_path::STORE_DIR,
                test_keypair(),
                test_trusted_keys(),
                true,
                false,
            );

            let mut kp2 = DerivationRegistry::default();
            let (drv_path2, _) = build_and_register("persist-test", &[], &mut kp2);

            let outcome = builder.build(&drv_path2, &mut kp2).await.unwrap();
            assert!(outcome.cached, "should be cached from persistent blob store");

            let recorded = calls.lock().unwrap();
            assert_eq!(recorded.len(), 0, "should NOT call do_build — blobs persisted on disk",);
        }
    }

    // ── Fetcher through DispatchBuildService + finish_build ──────────────
    //
    // These verify that fetcher derivations flow through the full
    // prepare → DispatchBuildService → FetchBuildService → finish_build
    // pipeline for both flat and recursive (NAR) hash modes.

    /// Build a fetchurl-style derivation. No `unpack` flag set.
    fn make_fetcher_drv_for_build(
        name: &str,
        url: &str,
        ca_hash: Option<nix_compat::nixhash::CAHash>,
        kp: &mut DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output { path: None, ca_hash });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "builtin".into());
        environment.insert("builder".to_string(), "builtin:fetchurl".into());
        environment.insert("url".to_string(), url.into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec![],
            builder: "builtin:fetchurl".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "builtin".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths(name, &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path(name).unwrap();
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);
        (drv_path, drv)
    }

    /// Build a fetchurl-style derivation with `unpack=1` (tarball mode).
    /// Uses `CAHash::Nar` for recursive hash verification.
    fn make_fetcher_drv_for_build_unpack(
        name: &str,
        url: &str,
        ca_hash: Option<nix_compat::nixhash::CAHash>,
        kp: &mut DerivationRegistry,
    ) -> (StorePath<String>, Derivation) {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output { path: None, ca_hash });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "builtin".into());
        environment.insert("builder".to_string(), "builtin:fetchurl".into());
        environment.insert("url".to_string(), url.into());
        environment.insert("unpack".to_string(), "1".into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec![],
            builder: "builtin:fetchurl".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "builtin".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths(name, &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path(name).unwrap();
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);
        (drv_path, drv)
    }

    /// Create a gzipped tarball with a single file, return the path.
    fn create_test_tarball(file_name: &str, content: &[u8]) -> tempfile::NamedTempFile {
        let tmp_src = tempfile::tempdir().unwrap();
        let inner = tmp_src.path().join("project-v1");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(inner.join(file_name), content).unwrap();

        let tar_file = tempfile::NamedTempFile::new().unwrap();
        {
            let gz = flate2::write::GzEncoder::new(
                std::fs::File::create(tar_file.path()).unwrap(),
                flate2::Compression::fast(),
            );
            let mut builder = tar::Builder::new(gz);
            builder.append_dir_all("project-v1", &inner).unwrap();
            builder.into_inner().unwrap().finish().unwrap();
        }
        tar_file
    }

    /// Compute the NAR sha256 of a tarball's unpacked content, using
    /// the same extract+ingest path that FetchBuildService uses.
    async fn nar_sha256_of_tarball(tarball_path: &std::path::Path) -> [u8; 32] {
        let extract_dir = tempfile::tempdir().unwrap();
        let url = format!("file://{}", tarball_path.display());
        crate::fetcher::fetch_and_unpack(&url, extract_dir.path().to_str().unwrap()).unwrap();

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node =
            snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(bs.clone(), ds.clone(), extract_dir.path(), None)
                .await
                .unwrap();

        use snix_store::nar::NarCalculationService;
        let renderer = snix_store::nar::SimpleRenderer::new(
            std::sync::Arc::new(bs) as std::sync::Arc<dyn snix_castore::blobservice::BlobService>,
            std::sync::Arc::new(ds) as std::sync::Arc<dyn snix_castore::directoryservice::DirectoryService>,
        );
        let (_size, sha256) = renderer.calculate_nar(&node).await.unwrap();
        sha256
    }

    /// A mock sandbox service that panics if called. Proves the
    /// DispatchBuildService never routes fetcher requests here.
    struct PanicSandboxService;

    #[async_trait]
    impl BuildService for PanicSandboxService {
        async fn do_build(&self, _request: BuildRequest) -> std::io::Result<BuildResult> {
            panic!("sandbox service must NOT be called for fetcher derivations");
        }
    }

    fn restricted_fetch_test_builder(
        blob_service: MemoryBlobService,
        directory_service: RedbDirectoryService,
        output_dir: &Path,
    ) -> (
        Builder<
            crate::dispatch_build_service::DispatchBuildService<
                crate::fetch_build_service::FetchBuildService,
                PanicSandboxService,
            >,
        >,
        crunch_store::OutputLookup,
        crunch_store::RootRegistry,
    ) {
        let store = crunch_store::StoreHandle::from_services_with_store_dir(
            crunch_store::StoreHandleServices {
                blob_service: Arc::new(blob_service),
                directory_service: Arc::new(directory_service),
                pathinfo_service: Arc::new(test_pis()),
                remote_pathinfo: None,
                state_dir: output_dir.join("state"),
                output_dir_str: output_dir.to_string_lossy().into_owned(),
                publishers: Vec::new(),
            },
            nix_compat::store_path::STORE_DIR.to_string(),
        );
        let crunch_store::PipelineStoreParts {
            build_store,
            action_results,
            build_service_store,
            output_lookup,
            root_registry,
        } = store.into_pipeline_store_parts();
        let fetch_service = crate::fetch_build_service::FetchBuildService::new(build_service_store);
        let dispatch = crate::dispatch_build_service::DispatchBuildService::new(fetch_service, PanicSandboxService);
        let builder = Builder::from_store_parts(
            crunch_store::BuilderStoreParts {
                build_store,
                action_results,
            },
            dispatch,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        (builder, output_lookup, root_registry)
    }

    #[tokio::test]
    async fn fetcher_through_dispatch_service_hash_match() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;
        use sha2::Digest;

        let content = b"fetcher integration test content";
        let digest: [u8; 32] = sha2::Sha256::digest(content).into();
        let ca = CAHash::Flat(NixHash::Sha256(digest));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), content).unwrap();
        let url = format!("file://{}", tmp.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // DispatchBuildService wrapping FetchBuildService + PanicSandboxService.
        // If DispatchBuildService ever routes the fetcher to the sandbox,
        // PanicSandboxService will panic and fail the test.
        let fetch_svc = test_fetch_build_service(bs.clone(), ds.clone());
        let dispatch = crate::dispatch_build_service::DispatchBuildService::new(fetch_svc, PanicSandboxService);

        let output_dir = tempfile::tempdir().unwrap();
        let mut builder = Builder::new(
            bs,
            ds,
            dispatch,
            test_pis(),
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = make_fetcher_drv_for_build("fetch-hash-match", &url, Some(ca), &mut kp);

        // Build through the full prepare → dispatch → finish pipeline.
        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();

        // finish_build verified the FOD hash (would have returned
        // Error::FodHashMismatch otherwise) and persisted PathInfo.
        assert!(!outcome.cached, "first build should not be cached");
        assert!(outcome.outputs.contains_key("out"), "must produce 'out' output");
        let path_info = &outcome.outputs["out"];
        assert!(path_info.nar_size > 0, "PathInfo must have non-zero NAR size");
        // The CA field must match the declared flat hash.
        assert!(path_info.ca.is_some(), "PathInfo must have CA field for FOD output",);
        let reports = builder.take_network_policy_reports();
        assert_eq!(reports.len(), 1, "fetcher build should report one network-policy boundary");
        assert_eq!(reports[0].action_name, "fetch-hash-match");
        assert_eq!(reports[0].mode, crate::NETWORK_MODE_FIXED_OUTPUT_FETCHER);
        assert_eq!(reports[0].result, crate::NETWORK_RESULT_ALLOWED);
        let fixed_output = reports[0].fixed_output.as_ref().expect("fixed-output network declaration");
        assert_eq!(fixed_output.url.as_deref(), Some(url.as_str()));
        assert_eq!(fixed_output.retry_policy, crate::NETWORK_RETRY_POLICY_BOUNDED_TRANSIENT_FETCH);
    }

    #[tokio::test]
    async fn fetcher_through_dispatch_service_hash_mismatch() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        use crate::worker::Worker;

        // Deliberately wrong hash.
        let wrong_hash = CAHash::Flat(NixHash::Sha256([0xAA; 32]));

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"mismatch test content").unwrap();
        let url = format!("file://{}", tmp.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let output_dir = tempfile::tempdir().unwrap();
        let (mut builder, output_lookup, root_registry) = restricted_fetch_test_builder(bs, ds, output_dir.path());

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) = make_fetcher_drv_for_build("fetch-hash-mismatch", &url, Some(wrong_hash), &mut kp);

        // Use Worker directly to access FailedGoal.error (build_all
        // wraps it into a generic Store error).
        let mut worker = Worker::new(1);
        worker.want(&drv_path, &kp, true).unwrap();
        let result = worker.run(&mut builder, &mut kp).await.unwrap();

        // No successful outcomes.
        assert!(result.outcomes.is_empty(), "mismatch build must not succeed: {:?}", result.outcomes,);

        // Exactly one failure.
        assert_eq!(result.failed.len(), 1, "exactly one root must fail");
        let err_str = &result.failed[0].error;

        // Verify the error is specifically a FOD hash mismatch from
        // verify_fod_hash in finish_build, not a generic build failure.
        assert!(
            err_str.contains("FOD hash mismatch"),
            "error must be a FOD hash mismatch from verify_fod_hash, got: {err_str}"
        );
        // Verify expected hash is reported (the [0xAA; 32] we declared).
        assert!(
            err_str.contains("expected sha256-"),
            "error must report expected hash in SRI format, got: {err_str}"
        );
        // Verify actual hash is reported.
        assert!(err_str.contains(", got sha256-"), "error must report actual hash in SRI format, got: {err_str}");

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let persisted_path_info = output_lookup.find(out_path).await.unwrap();
        let persisted_roots = root_registry.list().unwrap();
        let physical_output = PathBuf::from(out_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
        let action_ref = action_ref_for_derivation(&drv, nix_compat::store_path::STORE_DIR);
        let state_dir = output_dir.path().join("state");
        let local_results = crunch_store::LocalActionResultStore::new(&state_dir);
        let lookup = crunch_store::ActionResultStore::lookup(&local_results, &action_ref).await.unwrap();

        assert!(builder.store.output_node(out_path).is_none(), "output_nodes must NOT contain the mismatched output");
        assert!(persisted_path_info.is_none(), "mismatched output must not publish PathInfo");
        assert!(!physical_output.exists(), "mismatched output bytes must not be materialized");
        assert!(builder.store.get_artifact_attestation(out_path).await.unwrap().is_none());
        assert!(persisted_roots.is_empty(), "mismatched output must not publish roots");
        assert!(lookup.records.is_empty(), "mismatched output must not publish action results");
        assert!(lookup.index.result_refs.is_empty(), "mismatched output must not publish action-result indexes");
        assert!(!builder.take_action_result_reports().iter().any(|report| report.phase == "publication"));
    }

    // ── Recursive (NAR) fetcher tests ─────────────────────────────

    #[tokio::test]
    async fn fetcher_recursive_through_dispatch_hash_match() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        // Create a tarball and pre-compute its NAR sha256.
        let tarball = create_test_tarball("hello.txt", b"recursive test content");
        let nar_sha256 = nar_sha256_of_tarball(tarball.path()).await;
        let ca = CAHash::Nar(NixHash::Sha256(nar_sha256));
        let url = format!("file://{}", tarball.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let fetch_svc = test_fetch_build_service(bs.clone(), ds.clone());
        let dispatch = crate::dispatch_build_service::DispatchBuildService::new(fetch_svc, PanicSandboxService);

        let output_dir = tempfile::tempdir().unwrap();
        let mut builder = Builder::new(
            bs,
            ds,
            dispatch,
            test_pis(),
            output_dir.path().to_path_buf(),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (drv_path, _drv) = make_fetcher_drv_for_build_unpack("fetch-recursive-match", &url, Some(ca), &mut kp);

        let outcome = builder.build(&drv_path, &mut kp).await.unwrap();

        assert!(!outcome.cached);
        assert!(outcome.outputs.contains_key("out"));
        let path_info = &outcome.outputs["out"];
        assert!(path_info.nar_size > 0);
        // CA must be NAR (recursive), not Flat.
        match &path_info.ca {
            Some(nix_compat::nixhash::CAHash::Nar(NixHash::Sha256(h))) => {
                assert_eq!(h, &nar_sha256, "persisted CA hash must match declared NAR hash");
            }
            other => panic!("expected CAHash::Nar(Sha256), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fetcher_recursive_through_dispatch_hash_mismatch() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        use crate::worker::Worker;

        // Tarball with wrong NAR hash.
        let tarball = create_test_tarball("data.txt", b"recursive mismatch content");
        let wrong_ca = CAHash::Nar(NixHash::Sha256([0xBB; 32]));
        let url = format!("file://{}", tarball.path().display());

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let output_dir = tempfile::tempdir().unwrap();
        let (mut builder, output_lookup, root_registry) = restricted_fetch_test_builder(bs, ds, output_dir.path());

        let mut kp = DerivationRegistry::default();
        let (drv_path, drv) =
            make_fetcher_drv_for_build_unpack("fetch-recursive-mismatch", &url, Some(wrong_ca), &mut kp);

        let mut worker = Worker::new(1);
        worker.want(&drv_path, &kp, true).unwrap();
        let result = worker.run(&mut builder, &mut kp).await.unwrap();

        assert!(result.outcomes.is_empty());
        assert_eq!(result.failed.len(), 1);
        let err_str = &result.failed[0].error;

        assert!(
            err_str.contains("FOD hash mismatch"),
            "recursive mismatch must report FOD hash mismatch, got: {err_str}"
        );
        assert!(err_str.contains("expected sha256-"), "must report expected NAR hash in SRI, got: {err_str}");
        assert!(err_str.contains(", got sha256-"), "must report actual NAR hash in SRI, got: {err_str}");

        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let persisted_path_info = output_lookup.find(out_path).await.unwrap();
        let persisted_roots = root_registry.list().unwrap();
        let physical_output = PathBuf::from(out_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
        let action_ref = action_ref_for_derivation(&drv, nix_compat::store_path::STORE_DIR);
        let state_dir = output_dir.path().join("state");
        let local_results = crunch_store::LocalActionResultStore::new(&state_dir);
        let lookup = crunch_store::ActionResultStore::lookup(&local_results, &action_ref).await.unwrap();

        assert!(
            builder.store.output_node(out_path).is_none(),
            "output_nodes must NOT contain the mismatched recursive output"
        );
        assert!(persisted_path_info.is_none(), "mismatched recursive output must not publish PathInfo");
        assert!(!physical_output.exists(), "mismatched recursive output bytes must not be materialized");
        assert!(builder.store.get_artifact_attestation(out_path).await.unwrap().is_none());
        assert!(persisted_roots.is_empty(), "mismatched recursive output must not publish roots");
        assert!(lookup.records.is_empty(), "mismatched recursive output must not publish action results");
        assert!(lookup.index.result_refs.is_empty(), "mismatched output must not publish action-result indexes");
        assert!(!builder.take_action_result_reports().iter().any(|report| report.phase == "publication"));
    }
}
