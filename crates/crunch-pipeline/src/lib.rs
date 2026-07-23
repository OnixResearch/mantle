#![feature(register_tool)]
#![register_tool(tigerstyle)]

mod derivation_file;

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

pub use crunch_build::BUILD_ENVIRONMENT_DIGEST_ALGORITHM;
pub use crunch_build::BuildDeterminismControl;
pub use crunch_build::BuildDeterminismNormalizationReport;
pub use crunch_build::BuildEnvironmentRejection;
pub use crunch_build::BuildEnvironmentReport;
pub use crunch_build::BuildNetworkPolicyReport;
use crunch_build::BuildOutcome;
pub use crunch_build::BuildOutputDivergenceDiagnostic;
pub use crunch_build::BuildSearchPathAlias;
pub use crunch_build::BuildSearchPathEntry;
pub use crunch_build::BuildSearchPathReport;
use crunch_build::Builder;
pub use crunch_build::DETERMINISM_NORMALIZATION_DIGEST_ALGORITHM;
use crunch_build::DerivationRegistry;
use crunch_build::DispatchBuildService;
use crunch_build::EvalMessage;
use crunch_build::FailedGoal;
use crunch_build::FetchBuildService;
use crunch_build::FetchSourceOverride;
use crunch_build::FetchSourcePolicy;
pub use crunch_build::FixedOutputNetworkDeclaration;
pub use crunch_build::HermeticityAuditEvent;
pub use crunch_build::HermeticityAuditKind;
pub use crunch_build::HermeticityMode;
use crunch_build::KeyPair;
use crunch_build::LocalBuildServiceRealizer;
use crunch_build::NativeDynamicPlanReport;
pub use crunch_build::PriorityCandidateEvidence;
pub use crunch_build::PriorityDecisionEvidence;
use crunch_build::RemoteBuildFallbackPolicy;
use crunch_build::RemoteFirstBuildService;
pub use crunch_build::SEARCH_PATH_DIGEST_ALGORITHM;
pub use crunch_build::SchedulingPolicy;
use crunch_build::Worker;
use crunch_build::WorkerResult;
use crunch_eval::session::RootForceExecutionPolicy;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_store::GcRootSource;
use crunch_store::StoreMutationGuard;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::info;

pub struct BuildConfig {
    pub file: PathBuf,
    pub import_paths: Vec<OsString>,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub store_dir: String,
    pub verbose: bool,
    pub max_jobs: u32,
    /// Explicit typed policy for deterministic ready-goal ordering.
    pub scheduling_policy: SchedulingPolicy,
    /// Ordered list of substituter URLs. Empty = no remote substitution.
    pub substituter_urls: Vec<String>,
    pub hermeticity_mode: HermeticityMode,
    /// Ordered list of read-only base store state directories for overlay
    /// composition.  Empty = no overlay composition (default single-store).
    pub base_state_dirs: Vec<PathBuf>,
    /// Signing keypair — every build output gets signed.
    pub keypair: KeyPair,
    /// Trusted public keys for signature verification on cache hits.
    pub trusted_keys: Vec<VerifyingKey>,
    /// When true, skip signature verification on cache hits.
    pub trust_unsigned: bool,
    /// Optional retained-root source for successful top-level outputs.
    pub root_retention_source: Option<GcRootSource>,
    /// Optional local source-state payloads that can satisfy fixed-output fetchers.
    pub source_fetch_overrides: Vec<FetchSourceOverride>,
    /// When true, wrap the sandbox service in RemoteFirstBuildService
    /// to exercise the remote-build dispatch path through the scheduler.
    pub remote_enabled: bool,
}

#[derive(Debug)]
pub struct PipelineResult {
    pub outcomes: Vec<BuildOutcome>,
    pub failed: Vec<FailedGoal>,
    pub fod_mismatches: Vec<FodMismatch>,
    pub root_labels: HashMap<String, String>,
    pub hermeticity_mode: HermeticityMode,
    pub hermeticity_audit_events: Vec<HermeticityAuditEvent>,
    pub build_environment_reports: Vec<BuildEnvironmentReport>,
    pub network_policy_reports: Vec<BuildNetworkPolicyReport>,
    pub workspace_reports: Vec<crunch_build::WorkspaceExecutionReport>,
    pub action_result_reports: Vec<crunch_build::ActionResultRuntimeReport>,
    pub native_dynamic_plans: Vec<NativeDynamicPlanReport>,
    pub priority_decisions: Vec<PriorityDecisionEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FodMismatch {
    pub name: String,
    pub expected_sri: String,
    pub actual_sri: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Eval(String),
    #[error("{0}")]
    Deserialize(String),
    #[error("{0}")]
    Convert(String),
    #[error("{0}")]
    Build(String),
    #[error("{0}")]
    Internal(String),
}

pub fn resolve_max_jobs(user: Option<u32>) -> u32 {
    const MAX_JOBS_CAP: u32 = 16;
    match user {
        Some(j) => j.clamp(1, MAX_JOBS_CAP),
        None => std::thread::available_parallelism().map(|n| (n.get() as u32).min(MAX_JOBS_CAP)).unwrap_or(1),
    }
}

pub fn parse_fod_mismatch_error(err: &str) -> Option<FodMismatch> {
    let rest = err.strip_prefix("FOD hash mismatch for ")?;
    let (name, rest) = rest.split_once(": expected ")?;
    let (expected_sri, actual_sri) = rest.split_once(", got ")?;
    let normalized_name = name.strip_suffix(".drv").unwrap_or(name);
    Some(FodMismatch {
        name: normalized_name.to_string(),
        expected_sri: expected_sri.to_string(),
        actual_sri: actual_sri.to_string(),
    })
}

#[allow(tigerstyle::no_recursion)]
fn eval_error_is_deserialize(err: &crunch_eval::Error) -> bool {
    match err {
        crunch_eval::Error::Eval(_) | crunch_eval::Error::Io(_) => false,
        crunch_eval::Error::Boundary(_) | crunch_eval::Error::Serde(_) => true,
        crunch_eval::Error::Labeled { source, .. } => eval_error_is_deserialize(source),
    }
}

fn map_eval_error(err: crunch_eval::Error) -> Error {
    if eval_error_is_deserialize(&err) {
        return Error::Deserialize(err.to_string());
    }
    Error::Eval(err.to_string())
}

pub async fn build(config: &BuildConfig) -> Result<PipelineResult, Error> {
    validate_build_config(config)?;
    let _mutation_guard = StoreMutationGuard::acquire_wait(&config.state_dir)
        .map_err(|err| Error::Internal(format!("acquiring store mutation lock: {err}")))?;

    let mut session = crunch_eval::session::EvaluationSession::open_file(&config.file, &config.import_paths)
        .map_err(map_eval_error)?;

    let store = if config.base_state_dirs.is_empty() {
        match crunch_store::StoreHandle::open(crunch_store::StoreConfig {
            state_dir: config.state_dir.clone(),
            output_dir: config.output_dir.clone(),
            remote_cache_urls: config.substituter_urls.clone(),
            fallback_mode: store_fallback_mode(config.hermeticity_mode),
            store_dir: config.store_dir.clone(),
            base_state_dirs: Vec::new(),
        })
        .await
        {
            Ok(store) => store,
            Err(err @ crunch_store::Error::PathInfoFallbackRejected { .. }) => {
                let derivations = session.force_all_roots::<CrunchDerivation>().map_err(map_eval_error)?;
                debug_assert!(!derivations.is_empty(), "must have at least one derivation");
                return build_preflight_failure(config, &derivations, err.to_string());
            }
            Err(e) => return Err(Error::Internal(format!("opening store: {e}"))),
        }
    } else {
        match crunch_store::StoreHandle::open_overlay(crunch_store::StoreConfig {
            state_dir: config.state_dir.clone(),
            output_dir: config.output_dir.clone(),
            remote_cache_urls: config.substituter_urls.clone(),
            fallback_mode: store_fallback_mode(config.hermeticity_mode),
            store_dir: config.store_dir.clone(),
            base_state_dirs: config.base_state_dirs.clone(),
        })
        .await
        {
            Ok(store) => store,
            Err(err @ crunch_store::Error::PathInfoFallbackRejected { .. }) => {
                let derivations = session.force_all_roots::<CrunchDerivation>().map_err(map_eval_error)?;
                debug_assert!(!derivations.is_empty(), "must have at least one derivation");
                return build_preflight_failure(config, &derivations, err.to_string());
            }
            Err(e) => return Err(Error::Internal(format!("opening overlay store: {e}"))),
        }
    };
    let mut hermeticity_audit_events = mode_audit_events(config.hermeticity_mode);
    hermeticity_audit_events.extend(map_store_audit_events(store.startup_audit_events()));

    #[cfg(target_os = "linux")]
    {
        return build_linux(config, store, session, hermeticity_audit_events).await;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = store;
        Err(Error::Build("building is only supported on Linux (requires bwrap)".to_string()))
    }
}

#[cfg(target_os = "linux")]
async fn build_linux(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
    session: crunch_eval::session::EvaluationSession,
    hermeticity_audit_events: Vec<HermeticityAuditEvent>,
) -> Result<PipelineResult, Error> {
    debug_assert!(!config.store_dir.is_empty());
    debug_assert!(config.max_jobs >= 1);

    let (mut builder, workspace_evidence_sink) = create_pipeline_builder(config, store)?;

    let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
    let mut known_paths = DerivationRegistry::new(&config.store_dir);
    let mut worker = Worker::with_scheduling_policy(config.max_jobs, config.scheduling_policy.clone())
        .map_err(|error| Error::Build(format!("scheduler policy: {error}")))?;
    let worker_run = worker.run_streaming(&mut builder, &mut known_paths, &mut rx);
    let eval_stream = stream_roots_into_worker(
        config.max_jobs,
        &config.store_dir,
        RootForceExecutionPolicy::PreferThreaded,
        &config.file,
        &config.import_paths,
        &session,
        tx,
    );
    let (worker_run, eval_stream) = tokio::join!(worker_run, eval_stream);
    let worker_result = match worker_run {
        Ok(result) => result,
        Err(err) => return Err(Error::Build(format!("{err}"))),
    };
    let eval_stream = eval_stream?;
    let mut hermeticity_audit_events = hermeticity_audit_events;
    hermeticity_audit_events.extend(builder.take_hermeticity_audit_events());
    let pipeline_evidence = PipelineRunEvidence {
        hermeticity_audit_events,
        build_environment_rows: builder.take_build_environment_reports(),
        network_policy_rows: builder.take_network_policy_reports(),
        workspace_rows: workspace_evidence_sink.take(),
        action_result_rows: builder.take_action_result_reports(),
    };
    Ok(finish_pipeline_result(
        &config.store_dir,
        config.hermeticity_mode,
        worker_result,
        eval_stream,
        pipeline_evidence,
    ))
}

#[cfg(target_os = "linux")]
fn create_pipeline_builder(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
) -> Result<(Builder<impl snix_build::buildservice::BuildService + use<>>, crunch_build::WorkspaceReportCollector), Error>
{
    use snix_build::buildservice::BubblewrapBuildService;
    debug_assert!(!config.store_dir.is_empty(), "store prefix must not be empty");
    debug_assert!(config.max_jobs >= 1, "builder requires at least one job");

    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    let workdir = std::env::temp_dir().join("crunch-builds");
    std::fs::create_dir_all(&workdir).map_err(|error| Error::Internal(format!("create workdir: {error}")))?;
    let source_policy = if config.source_fetch_overrides.is_empty() {
        FetchSourcePolicy::AllowNetwork
    } else {
        FetchSourcePolicy::RequireOverride
    };
    let fetch_service = FetchBuildService::new(blob_service.clone(), directory_service.clone())
        .with_source_overrides(config.source_fetch_overrides.clone())
        .with_source_policy(source_policy);

    // Keep remote dispatch inside the lazy scheduler so dedupe, waiter
    // notification, job bounds, fallback, and terminal propagation stay shared.
    // r[impl remote_builds.production_scheduler_realization]
    let profile = crunch_build::RealizerProfileFacts {
        name: "local-sandbox".to_string(),
        version: 1,
        capabilities: Vec::new(),
        parameters: std::collections::BTreeMap::new(),
    };
    let local_bwrap = BubblewrapBuildService::new(
        std::env::temp_dir().join("crunch-builds-local"),
        blob_service.clone(),
        directory_service.clone(),
    );
    let remote_bwrap = BubblewrapBuildService::new(workdir, blob_service.clone(), directory_service.clone());
    let remote_realizer = LocalBuildServiceRealizer::new(remote_bwrap, profile);
    let sandbox_service =
        RemoteFirstBuildService::new(remote_realizer, local_bwrap, RemoteBuildFallbackPolicy::OnRemoteFailure);
    let dispatch = DispatchBuildService::new(fetch_service, sandbox_service);
    let workspace_evidence_sink = empty_workspace_report_collector();
    let build_service =
        crunch_build::StatefulWorkspaceBuildService::new(dispatch, store.state_dir(), workspace_evidence_sink.clone());
    let mut builder = Builder::from_store(
        store,
        build_service,
        config.keypair.clone(),
        config.trusted_keys.clone(),
        config.trust_unsigned,
        config.verbose,
    );
    builder.set_hermeticity_mode(config.hermeticity_mode);
    builder.set_root_retention_source(config.root_retention_source);
    Ok((builder, workspace_evidence_sink))
}

struct PipelineRunEvidence {
    hermeticity_audit_events: Vec<HermeticityAuditEvent>,
    build_environment_rows: Vec<BuildEnvironmentReport>,
    network_policy_rows: Vec<BuildNetworkPolicyReport>,
    workspace_rows: Vec<crunch_build::WorkspaceExecutionReport>,
    action_result_rows: Vec<crunch_build::ActionResultRuntimeReport>,
}

fn empty_workspace_report_collector() -> crunch_build::WorkspaceReportCollector {
    // The foreign type exposes no explicit constructor. Normalize its only
    // public construction path to the empty state required at this boundary.
    let construct_empty: fn() -> crunch_build::WorkspaceReportCollector = std::default::Default::default;
    let collector = construct_empty();
    debug_assert!(collector.take().is_empty(), "collector must start empty");
    debug_assert!(collector.take().is_empty(), "collector must remain empty after draining");
    collector
}

fn finish_pipeline_result(
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
    mut worker_result: WorkerResult,
    eval_stream: EvalStreamResult,
    evidence: PipelineRunEvidence,
) -> PipelineResult {
    debug_assert!(!store_dir.is_empty(), "store prefix must not be empty");
    debug_assert!(
        eval_stream.eval_failure.is_some() || !eval_stream.root_drv_paths.is_empty(),
        "pipeline must finish with a converted root or labeled evaluation failure"
    );

    if let Some(eval_failure) = &eval_stream.eval_failure {
        worker_result.failed.push(FailedGoal {
            drv_key: eval_failure_key(&eval_failure.label),
            error: eval_failure.error.clone(),
        });
    }
    normalize_failed_goal_keys(&mut worker_result.failed, store_dir);
    let mut root_labels = build_root_labels(&eval_stream.root_drv_paths, store_dir);
    if let Some(eval_failure) = &eval_stream.eval_failure {
        root_labels.insert(eval_failure_key(&eval_failure.label), eval_failure.label.clone());
    }

    PipelineResult {
        root_labels,
        fod_mismatches: collect_fod_mismatches(&worker_result.failed),
        outcomes: worker_result.outcomes,
        failed: worker_result.failed,
        hermeticity_mode,
        hermeticity_audit_events: evidence.hermeticity_audit_events,
        build_environment_reports: evidence.build_environment_rows,
        network_policy_reports: evidence.network_policy_rows,
        workspace_reports: evidence.workspace_rows,
        action_result_reports: evidence.action_result_rows,
        native_dynamic_plans: worker_result.native_dynamic_plans,
        priority_decisions: worker_result.priority_decisions,
    }
}

fn map_store_audit_events(store_events: &[crunch_store::StoreAuditEvent]) -> Vec<HermeticityAuditEvent> {
    store_events.iter().cloned().map(HermeticityAuditEvent::from).collect()
}

fn mode_audit_events(mode: HermeticityMode) -> Vec<HermeticityAuditEvent> {
    if !mode.is_impure() {
        return Vec::new();
    }
    vec![HermeticityAuditEvent::new(
        HermeticityAuditKind::ImpureModeSelected,
        "explicit --impure mode permits ambient host dependencies; output is not reproducibility-proof eligible",
    )]
}

fn store_fallback_mode(mode: HermeticityMode) -> crunch_store::StoreFallbackMode {
    if mode.is_strict() {
        return crunch_store::StoreFallbackMode::Strict;
    }
    crunch_store::StoreFallbackMode::Practical
}

fn validate_build_config(config: &BuildConfig) -> Result<(), Error> {
    if !config.output_dir.exists() {
        return Err(Error::Internal(format!(
            "output store directory {} does not exist.\nCreate it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            config.output_dir.display()
        )));
    }
    if config.max_jobs == 0 {
        return Err(Error::Internal("max_jobs must be at least 1".to_string()));
    }
    if config.store_dir.is_empty() {
        return Err(Error::Internal("store_dir must not be empty".to_string()));
    }
    if !config.store_dir.starts_with('/') {
        return Err(Error::Internal(format!("store_dir must be an absolute path: {}", config.store_dir,)));
    }
    Ok(())
}

struct EvalStreamResult {
    root_drv_paths: Vec<(String, StorePath<String>)>,
    eval_failure: Option<EvalFailure>,
}

struct EvalFailure {
    label: String,
    error: String,
}

async fn stream_roots_into_worker(
    max_jobs: u32,
    store_dir: &str,
    root_force_policy: RootForceExecutionPolicy,
    root_file: &std::path::Path,
    import_paths: &[OsString],
    session: &crunch_eval::session::EvaluationSession,
    tx: mpsc::Sender<EvalMessage>,
) -> Result<EvalStreamResult, Error> {
    let requested_labels = session.root_labels().iter().map(|root_label| root_label.label.clone()).collect::<Vec<_>>();
    debug_assert!(!requested_labels.is_empty(), "must have at least one root label");

    let eval_parallelism = resolve_eval_parallelism(max_jobs, requested_labels.len() as u32);
    let worker_input = session.isolated_worker_input();
    let mut join_set = JoinSet::new();
    let mut next_label_index: usize = 0;
    let mut cache = ConversionCache::new(store_dir);
    let mut file_resolver = derivation_file::DerivationFileResolver::new(root_file, import_paths)?;
    let mut root_drv_paths = Vec::with_capacity(requested_labels.len());
    let mut first_failure: Option<EvalFailure> = None;

    spawn_eval_workers(
        &mut join_set,
        &worker_input,
        &requested_labels,
        &mut next_label_index,
        eval_parallelism,
        root_force_policy,
    );
    while let Some(join_result) = join_set.join_next().await {
        let worker_result = join_result.map_err(|e| Error::Internal(format!("eval worker panicked: {e}")))?;

        match worker_result {
            Ok((label, mut drv)) => {
                if first_failure.is_some() {
                    continue;
                }
                file_resolver.resolve_root_inputs(root_file, &mut drv, &mut cache)?;
                let (drv_path, _nix_drv) =
                    crunch_glue::convert(&drv, &mut cache).map_err(|e| Error::Convert(format!("{label}: {e}")))?;
                let new_entries = cache.drain_pending();
                info!(drv = %drv_path, label = %label, entries = new_entries.len(), "converted, sending to worker");
                tx.send(EvalMessage {
                    label: label.clone(),
                    drv_path: drv_path.clone(),
                    new_entries,
                })
                .await
                .map_err(|e| Error::Internal(format!("channel send: {e}")))?;
                root_drv_paths.push((label, drv_path));
            }
            Err((label, error)) => {
                if first_failure.is_none() {
                    first_failure = Some(EvalFailure { label, error });
                }
            }
        }

        if first_failure.is_none() {
            spawn_eval_workers(
                &mut join_set,
                &worker_input,
                &requested_labels,
                &mut next_label_index,
                eval_parallelism,
                root_force_policy,
            );
        }
    }

    drop(tx);
    Ok(EvalStreamResult {
        root_drv_paths,
        eval_failure: first_failure,
    })
}

type EvalWorkerResult = Result<(String, CrunchDerivation), (String, String)>;

#[allow(tigerstyle::too_many_parameters)]
fn spawn_eval_workers(
    join_set: &mut JoinSet<EvalWorkerResult>,
    worker_input: &crunch_eval::session::IsolatedWorkerInput,
    labels: &[String],
    next_label_index: &mut usize,
    eval_parallelism: u32,
    root_force_policy: RootForceExecutionPolicy,
) {
    let max_inflight: usize = match usize::try_from(eval_parallelism) {
        Ok(n) => n,
        Err(_) => return, // u32 > usize only on 16-bit targets; nothing to spawn
    };
    while *next_label_index < labels.len() && join_set.len() < max_inflight {
        let worker_input = worker_input.clone();
        let label = labels[*next_label_index].clone();
        *next_label_index = next_label_index.saturating_add(1);
        join_set.spawn_blocking(move || {
            let labels = vec![label.clone()];
            match worker_input.force_selected_roots_with_policy::<CrunchDerivation>(&labels, 1, root_force_policy) {
                Ok(mut roots) => {
                    debug_assert_eq!(roots.len(), 1, "single-label request must return one root");
                    let (_returned_label, drv) = roots
                        .pop()
                        .ok_or_else(|| (label.clone(), "single-label request returned zero roots".to_string()))?;
                    Ok((label, drv))
                }
                Err(err) => Err((label.clone(), format!("root '{label}': {err}"))),
            }
        });
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn resolve_eval_parallelism(max_jobs: u32, requested_root_count: u32) -> u32 {
    assert!(max_jobs >= 1, "max_jobs must be at least 1");
    assert!(requested_root_count >= 1, "requested_root_count must be at least 1");
    max_jobs.min(requested_root_count).max(1)
}

fn eval_failure_key(label: &str) -> String {
    format!("eval-root:{label}")
}

fn build_root_labels(root_drv_paths: &[(String, StorePath<String>)], store_dir: &str) -> HashMap<String, String> {
    let mut labels = HashMap::with_capacity(root_drv_paths.len());
    for (label, drv_path) in root_drv_paths {
        let key = drv_path.to_absolute_path_with_prefix(store_dir);
        labels.insert(key, label.clone());
    }
    labels
}

fn build_preflight_failure(
    config: &BuildConfig,
    derivations: &[(String, CrunchDerivation)],
    error: String,
) -> Result<PipelineResult, Error> {
    debug_assert!(!config.store_dir.is_empty(), "store prefix must not be empty");
    debug_assert!(!derivations.is_empty(), "preflight failure must cover at least one root");
    debug_assert!(!error.is_empty(), "preflight failure must retain its cause");
    let root_drv_paths = convert_root_drv_paths(derivations, &config.store_dir)?;
    let failed = root_drv_paths
        .iter()
        .map(|(_, drv_path)| FailedGoal {
            drv_key: drv_path.to_absolute_path_with_prefix(&config.store_dir),
            error: error.clone(),
        })
        .collect();
    Ok(PipelineResult {
        outcomes: Vec::new(),
        failed,
        fod_mismatches: Vec::new(),
        root_labels: build_root_labels(&root_drv_paths, &config.store_dir),
        hermeticity_mode: config.hermeticity_mode,
        hermeticity_audit_events: Vec::new(),
        build_environment_reports: Vec::new(),
        network_policy_reports: Vec::new(),
        workspace_reports: Vec::new(),
        action_result_reports: Vec::new(),
        native_dynamic_plans: Vec::new(),
        priority_decisions: Vec::new(),
    })
}

fn convert_root_drv_paths(
    derivations: &[(String, CrunchDerivation)],
    store_dir: &str,
) -> Result<Vec<(String, StorePath<String>)>, Error> {
    let mut cache = ConversionCache::new(store_dir);
    let mut root_drv_paths = Vec::with_capacity(derivations.len());
    for (label, drv) in derivations {
        let (drv_path, _nix_drv) =
            crunch_glue::convert(drv, &mut cache).map_err(|e| Error::Convert(format!("{label}: {e}")))?;
        root_drv_paths.push((label.clone(), drv_path));
    }
    Ok(root_drv_paths)
}

fn normalize_failed_goal_keys(failed: &mut [FailedGoal], store_dir: &str) {
    for failed_goal in failed {
        if parse_drv_key(store_dir, &failed_goal.drv_key).is_some() {
            continue;
        }
        let Ok(drv_path) = StorePath::from_absolute_path(failed_goal.drv_key.as_bytes()) else {
            continue;
        };
        failed_goal.drv_key = drv_key_for(store_dir, &drv_path);
    }
}

fn collect_fod_mismatches(failed: &[FailedGoal]) -> Vec<FodMismatch> {
    failed.iter().filter_map(|failed_goal| parse_fod_mismatch_error(&failed_goal.error)).collect()
}

pub fn drv_key_for(store_dir: &str, drv_path: &StorePath<String>) -> String {
    drv_path.to_absolute_path_with_prefix(store_dir)
}

#[allow(tigerstyle::ambiguous_params)]
pub fn parse_drv_key(store_dir: &str, drv_key: &str) -> Option<StorePath<String>> {
    StorePath::from_absolute_path_with_prefix(drv_key.as_bytes(), store_dir).ok()
}

pub fn label_for_key<'a>(result: &'a PipelineResult, drv_key: &str) -> Option<&'a str> {
    result.root_labels.get(drv_key).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    async fn collect_eval_message_labels(mut rx: mpsc::Receiver<EvalMessage>) -> Vec<String> {
        let mut labels = Vec::new();
        while let Some(message) = rx.recv().await {
            labels.push(message.label);
        }
        labels
    }

    #[test]
    fn resolve_max_jobs_default_in_range() {
        let jobs = resolve_max_jobs(None);
        assert!(jobs >= 1);
        assert!(jobs <= 16);
    }

    #[test]
    fn resolve_max_jobs_clamps_user_value() {
        assert_eq!(resolve_max_jobs(Some(0)), 1);
        assert_eq!(resolve_max_jobs(Some(1)), 1);
        assert_eq!(resolve_max_jobs(Some(99)), 16);
    }

    #[test]
    fn resolve_eval_parallelism_stays_within_requested_roots() {
        assert_eq!(resolve_eval_parallelism(1, 4), 1);
        assert_eq!(resolve_eval_parallelism(4, 1), 1);
        assert_eq!(resolve_eval_parallelism(4, 3), 3);
    }

    #[test]
    fn eval_failure_key_uses_stable_prefix() {
        assert_eq!(eval_failure_key("alpha"), "eval-root:alpha");
        assert_eq!(eval_failure_key("pkg"), "eval-root:pkg");
    }

    #[test]
    fn map_eval_error_keeps_labeled_deserialize_failures_in_deserialize_class() {
        let err = crunch_eval::Error::Labeled {
            label: "alpha".to_string(),
            source: Box::new(crunch_eval::Error::Serde("bad shape".to_string())),
        };

        let mapped = map_eval_error(err);

        assert!(matches!(mapped, Error::Deserialize(_)), "expected deserialize mapping, got: {mapped}");
        assert!(mapped.to_string().contains("alpha"));
    }

    #[test]
    fn parse_fod_mismatch_valid() {
        let mismatch =
            parse_fod_mismatch_error("FOD hash mismatch for src: expected sha256-aaa, got sha256-bbb").unwrap();
        assert_eq!(mismatch.name, "src");
        assert_eq!(mismatch.expected_sri, "sha256-aaa");
        assert_eq!(mismatch.actual_sri, "sha256-bbb");
    }

    #[test]
    fn parse_fod_mismatch_invalid() {
        assert!(parse_fod_mismatch_error("something else").is_none());
        assert!(parse_fod_mismatch_error("FOD hash mismatch for x").is_none());
    }

    #[test]
    fn parse_fod_mismatch_edge_case_preserves_trailing_context() {
        let mismatch = parse_fod_mismatch_error(
            "FOD hash mismatch for src-1: expected sha256-aaa, got sha256-bbb (builder log follows)",
        )
        .unwrap();
        assert_eq!(mismatch.name, "src-1");
        assert_eq!(mismatch.expected_sri, "sha256-aaa");
        assert_eq!(mismatch.actual_sri, "sha256-bbb (builder log follows)");
    }

    #[test]
    fn parse_fod_mismatch_strips_drv_suffix() {
        let mismatch =
            parse_fod_mismatch_error("FOD hash mismatch for src-1.drv: expected sha256-aaa, got sha256-bbb").unwrap();
        assert_eq!(mismatch.name, "src-1");
        assert_eq!(mismatch.expected_sri, "sha256-aaa");
        assert_eq!(mismatch.actual_sri, "sha256-bbb");
    }

    #[test]
    fn parse_drv_key_round_trip() {
        let drv_path = StorePath::from_name_and_digest_fixed("hello.drv", [7u8; 20]).unwrap();
        let key = drv_key_for("/crunch/store", &drv_path);
        let reparsed = parse_drv_key("/crunch/store", &key).unwrap();
        assert_eq!(reparsed, drv_path);
    }

    #[test]
    fn normalize_failed_goal_keys_rewrites_nix_store_keys() {
        let drv_path: StorePath<String> = StorePath::from_name_and_digest_fixed("hello.drv", [9u8; 20]).unwrap();
        let mut failed = vec![FailedGoal {
            drv_key: drv_path.to_absolute_path(),
            error: "boom".to_string(),
        }];

        normalize_failed_goal_keys(&mut failed, "/crunch/store");

        assert_eq!(failed[0].drv_key, drv_path.to_absolute_path_with_prefix("/crunch/store"));
    }

    #[test]
    fn hermeticity_mode_display_uses_stable_strings() {
        assert_eq!(HermeticityMode::Practical.as_str(), "practical");
        assert_eq!(HermeticityMode::Strict.to_string(), "strict");
        assert!(HermeticityMode::Strict.is_strict());
        assert!(!HermeticityMode::Practical.is_strict());
    }

    #[test]
    fn hermeticity_audit_event_preserves_kind_and_detail() {
        let event = HermeticityAuditEvent::new(HermeticityAuditKind::HostToolFallback, "used host bwrap");
        assert_eq!(event.kind.as_str(), "host-tool-fallback");
        assert_eq!(event.detail, "used host bwrap");
    }

    #[test]
    fn map_store_audit_events_preserves_pathinfo_fallback() {
        let store_events = vec![crunch_store::StoreAuditEvent {
            kind: crunch_store::StoreAuditKind::PathInfoFallback,
            detail: "using in-memory fallback".to_string(),
        }];

        let mapped = map_store_audit_events(&store_events);

        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].kind, HermeticityAuditKind::PathInfoFallback);
        assert_eq!(mapped[0].detail, "using in-memory fallback");
    }

    #[test]
    fn map_store_audit_events_preserves_closure_degraded_kind() {
        let store_events = vec![crunch_store::StoreAuditEvent {
            kind: crunch_store::StoreAuditKind::ClosureResolutionDegraded,
            detail: "missing closure facts".to_string(),
        }];

        let mapped = map_store_audit_events(&store_events);

        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].kind, HermeticityAuditKind::ClosureResolutionDegraded);
        assert_eq!(mapped[0].detail, "missing closure facts");
    }

    #[test]
    fn store_fallback_mode_matches_hermeticity_mode() {
        assert_eq!(store_fallback_mode(HermeticityMode::Practical), crunch_store::StoreFallbackMode::Practical);
        assert_eq!(store_fallback_mode(HermeticityMode::Strict), crunch_store::StoreFallbackMode::Strict);
    }

    fn resolver_fixture_file() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("root.ncl");
        std::fs::write(&file, "{}").unwrap();
        assert!(file.is_file());
        assert!(file.starts_with(directory.path()));
        (directory, file)
    }

    #[tokio::test]
    async fn stream_roots_into_worker_matches_across_root_force_policies() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (inline_tx, inline_rx) = mpsc::channel::<EvalMessage>(16);
        let inline_result = stream_roots_into_worker(
            4,
            "/crunch/store",
            RootForceExecutionPolicy::Inline,
            &root_file,
            &[],
            &session,
            inline_tx,
        )
        .await
        .unwrap();
        let inline_labels = collect_eval_message_labels(inline_rx).await;

        let (preferred_tx, preferred_rx) = mpsc::channel::<EvalMessage>(16);
        let preferred_result = stream_roots_into_worker(
            4,
            "/crunch/store",
            RootForceExecutionPolicy::PreferThreaded,
            &root_file,
            &[],
            &session,
            preferred_tx,
        )
        .await
        .unwrap();
        let preferred_labels = collect_eval_message_labels(preferred_rx).await;

        let mut inline_sorted = inline_labels.clone();
        inline_sorted.sort();
        let mut preferred_sorted = preferred_labels.clone();
        preferred_sorted.sort();
        assert_eq!(inline_sorted, preferred_sorted);

        let mut inline_drv_sorted = inline_result.root_drv_paths.clone();
        inline_drv_sorted.sort();
        let mut preferred_drv_sorted = preferred_result.root_drv_paths.clone();
        preferred_drv_sorted.sort();
        assert_eq!(inline_drv_sorted, preferred_drv_sorted);

        assert!(inline_result.eval_failure.is_none());
        assert!(preferred_result.eval_failure.is_none());
    }

    #[tokio::test]
    async fn stream_roots_into_worker_reports_same_labeled_failure_across_policies() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  good = { name = "good", builder = "/bin/sh" },
  bad = { builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (inline_tx, inline_rx) = mpsc::channel::<EvalMessage>(16);
        let inline_result = stream_roots_into_worker(
            4,
            "/crunch/store",
            RootForceExecutionPolicy::Inline,
            &root_file,
            &[],
            &session,
            inline_tx,
        )
        .await
        .unwrap();
        let inline_labels = collect_eval_message_labels(inline_rx).await;

        let (preferred_tx, preferred_rx) = mpsc::channel::<EvalMessage>(16);
        let preferred_result = stream_roots_into_worker(
            4,
            "/crunch/store",
            RootForceExecutionPolicy::PreferThreaded,
            &root_file,
            &[],
            &session,
            preferred_tx,
        )
        .await
        .unwrap();
        let preferred_labels = collect_eval_message_labels(preferred_rx).await;

        assert!(inline_labels.len() <= 1, "inline should dispatch at most one root before failure");
        assert!(preferred_labels.len() <= 1, "preferred should dispatch at most one root before failure");
        let inline_failure = inline_result.eval_failure.expect("inline policy must report failure");
        let preferred_failure = preferred_result.eval_failure.expect("preferred policy must report failure");
        assert_eq!(inline_failure.label, preferred_failure.label);
        assert_eq!(inline_failure.label, "bad");
        assert_eq!(inline_failure.error, preferred_failure.error);
    }
}
