// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in the store-capability-migration change evidence and scheduled for the
// standalone hardening pass. Scoped to the lint categories present at recording time.
#![allow(
    tigerstyle::ambiguous_params,
    tigerstyle::assertion_density,
    tigerstyle::bool_naming,
    tigerstyle::too_many_parameters,
    tigerstyle::unbounded_collection_growth
)]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
// r[impl foreign_derivation_import.realization_adapter]
// r[impl foreign_derivation_import.realization_receipt]
// r[impl foreign_derivation_import.cache_only_runtime_closure]
// r[impl foreign_derivation_import.live_nixpkgs_realization_proof]

mod derivation_file;
mod evaluation_stream;
mod jobs_policy;
mod watch;

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

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
use crunch_build::NativePlanOutputBindingReport;
pub use crunch_build::PriorityCandidateEvidence;
pub use crunch_build::PriorityDecisionEvidence;
use crunch_build::RemoteBuildFallbackPolicy;
use crunch_build::RemoteFirstBuildService;
pub use crunch_build::SEARCH_PATH_DIGEST_ALGORITHM;
pub use crunch_build::SchedulingPolicy;
use crunch_build::Worker;
use crunch_build::WorkerResult;
use crunch_build::causal_trace::Trace;
use crunch_eval::session::RootForceExecutionPolicy;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_store::GcRootSource;
use crunch_store::StoreMutationGuard;
use evaluation_stream::EvalStreamRequest;
use evaluation_stream::EvalStreamResult;
use evaluation_stream::EvalWorkerControl;
pub use evaluation_stream::EvaluationCancellation;
use evaluation_stream::stream_roots_into_worker;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use tokio::sync::mpsc;
pub use watch::WatchNotice;
pub use watch::watch_build;

const EVAL_MESSAGE_CHANNEL_CAPACITY: usize = 16;
/// Live observation is bounded separately from the build scheduler's goal cap.
pub const MAX_LIVE_WORKER_GOALS: usize = 4_095;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerLiveGoal {
    pub drv_key: String,
    pub state: crunch_build::GoalState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerLiveSnapshot {
    pub goals: Vec<WorkerLiveGoal>,
}

/// A dropped/full observation queue degrades only the optional live view.
pub struct WorkerLiveObservation {
    sender: mpsc::Sender<Arc<WorkerLiveSnapshot>>,
    degraded: Arc<AtomicBool>,
}

impl WorkerLiveObservation {
    pub fn new(sender: mpsc::Sender<Arc<WorkerLiveSnapshot>>, degraded: Arc<AtomicBool>) -> Self {
        Self { sender, degraded }
    }
}

#[cfg(test)]
const EVAL_POLICY_TEST_MAX_JOBS: u32 = 4;

pub struct BuildConfig {
    pub file: PathBuf,
    pub import_paths: Vec<OsString>,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub store_dir: String,
    pub backend: crunch_store::StoreBackend,
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
    /// Optional managed provenance for successful selected roots.
    pub root_registration: Option<crunch_store::RootRegistration>,
    /// Optional local source-state payloads that can satisfy fixed-output fetchers.
    pub source_fetch_overrides: Vec<FetchSourceOverride>,
    /// When true, wrap the sandbox service in RemoteFirstBuildService
    /// to exercise the remote-build dispatch path through the scheduler.
    pub remote_enabled: bool,
    /// Optional directory for owner-emitted interchange records.
    pub interchange_dir: Option<PathBuf>,
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
    pub finish_gate_reports: Vec<crunch_build::FinishGateReport>,
    pub network_policy_reports: Vec<BuildNetworkPolicyReport>,
    pub workspace_reports: Vec<crunch_build::WorkspaceExecutionReport>,
    pub action_result_reports: Vec<crunch_build::ActionResultRuntimeReport>,
    pub native_dynamic_plans: Vec<NativeDynamicPlanReport>,
    pub plan_output_bindings: Vec<NativePlanOutputBindingReport>,
    pub priority_decisions: Vec<PriorityDecisionEvidence>,
    pub overlay_report: Option<crunch_store::StoreOverlayReport>,
    pub store_layer_selections: Vec<crunch_store::layer::StoreLayerSelection>,
}

#[derive(Debug, Clone)]
pub struct RegisteredOutputExpectation {
    pub unit_id: String,
    pub output_name: String,
    pub store_path: StorePath<String>,
}

#[derive(Debug, Clone)]
pub struct RegisteredOutputResult {
    pub unit_id: String,
    pub output_name: String,
    pub path_info: crunch_store::PathInfo,
}

pub struct RegisteredBuildRequest<'a> {
    pub roots: &'a [StorePath<String>],
    pub expected_outputs: &'a [RegisteredOutputExpectation],
    pub retained_outputs: &'a [StorePath<String>],
    pub source_policy: FetchSourcePolicy,
    pub cache_only: bool,
}

#[derive(Debug)]
pub struct RegisteredBuildResult {
    pub outcomes: Vec<BuildOutcome>,
    pub failed_roots: Vec<FailedGoal>,
    pub outputs: Vec<RegisteredOutputResult>,
    pub hermeticity_audit_events: Vec<HermeticityAuditEvent>,
    pub build_environment_reports: Vec<BuildEnvironmentReport>,
    pub finish_gate_reports: Vec<crunch_build::FinishGateReport>,
    pub network_policy_reports: Vec<BuildNetworkPolicyReport>,
    pub workspace_reports: Vec<crunch_build::WorkspaceExecutionReport>,
    pub action_result_reports: Vec<crunch_build::ActionResultRuntimeReport>,
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
    #[error("watch mode requires local cancellable sandbox builds; remote watch execution is not supported")]
    WatchRemoteUnsupported,
    #[error("{0}")]
    Internal(String),
}

/// Resolve the effective job limit for one build request.
///
/// The host parallelism is observed here once and passed to the pure policy;
/// the policy itself reads no ambient state.
pub fn resolve_max_jobs(user: Option<u32>) -> u32 {
    let observation = jobs_policy::JobsObservation::new(user, jobs_policy::observe_host_parallelism());
    match jobs_policy::resolve_jobs_policy(&observation) {
        Ok(jobs) => jobs,
        Err(error) => {
            debug_assert!(!error.reason_code().is_empty());
            jobs_policy::MIN_JOBS
        }
    }
}

pub use jobs_policy::DEFAULT_JOBS_CAP;
pub use jobs_policy::JobsObservation;
pub use jobs_policy::JobsPolicyError;
pub use jobs_policy::MIN_JOBS;
pub use jobs_policy::observe_host_parallelism;
pub use jobs_policy::resolve_jobs_policy;

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

/// A preflight result can exist before a worker goal is observable.
pub enum CausalTraceObservation {
    Recorded(Trace),
    WorkerNotStarted,
}

/// Build with an opt-in scheduler diagnostic, separate from report and receipts.
pub async fn build_with_causal_trace(config: &BuildConfig) -> Result<(PipelineResult, CausalTraceObservation), Error> {
    let mut observation = None;
    let result =
        build_with_stream_sender(config, EvaluationCancellation::new(), None, None, Some(&mut observation)).await?;
    Ok((result, observation.ok_or_else(|| Error::Internal("causal trace was not captured".to_string()))?))
}

/// Keep the live stream and worker snapshots active during a traced build.
pub async fn build_with_causal_trace_live(
    config: &BuildConfig,
    cancellation: EvaluationCancellation,
    stream_tx: mpsc::Sender<crunch_evaluation_stream_core::StreamRecordValue>,
    worker_observation: WorkerLiveObservation,
) -> Result<(PipelineResult, CausalTraceObservation), Error> {
    let mut observation = None;
    let result = build_with_stream_sender(
        config,
        cancellation,
        Some(stream_tx),
        Some(worker_observation),
        Some(&mut observation),
    )
    .await?;
    Ok((result, observation.ok_or_else(|| Error::Internal("causal trace was not captured".to_string()))?))
}

pub async fn build(config: &BuildConfig) -> Result<PipelineResult, Error> {
    build_with_stream_sender(config, EvaluationCancellation::new(), None, None, None).await
}

pub async fn build_with_evaluation_cancellation(
    config: &BuildConfig,
    cancellation: EvaluationCancellation,
) -> Result<PipelineResult, Error> {
    build_with_stream_sender(config, cancellation, None, None, None).await
}

pub async fn build_with_evaluation_stream(
    config: &BuildConfig,
    cancellation: EvaluationCancellation,
    stream_tx: mpsc::Sender<crunch_evaluation_stream_core::StreamRecordValue>,
) -> Result<PipelineResult, Error> {
    build_with_stream_sender(config, cancellation, Some(stream_tx), None, None).await
}

pub async fn build_with_live_observation(
    config: &BuildConfig,
    cancellation: EvaluationCancellation,
    stream_tx: mpsc::Sender<crunch_evaluation_stream_core::StreamRecordValue>,
    worker_observation: WorkerLiveObservation,
) -> Result<PipelineResult, Error> {
    build_with_stream_sender(config, cancellation, Some(stream_tx), Some(worker_observation), None).await
}

async fn build_with_stream_sender(
    config: &BuildConfig,
    cancellation: EvaluationCancellation,
    stream_tx: Option<mpsc::Sender<crunch_evaluation_stream_core::StreamRecordValue>>,
    worker_observation: Option<WorkerLiveObservation>,
    causal_trace_slot: Option<&mut Option<CausalTraceObservation>>,
) -> Result<PipelineResult, Error> {
    validate_build_config(config)?;
    let store_config = crunch_store::StoreConfig {
        backend: config.backend,
        state_dir: config.state_dir.clone(),
        output_dir: config.output_dir.clone(),
        remote_cache_urls: config.substituter_urls.clone(),
        fallback_mode: store_fallback_mode(config.hermeticity_mode),
        store_dir: config.store_dir.clone(),
        base_state_dirs: config.base_state_dirs.clone(),
    };
    store_config
        .preflight_backend_identity()
        .map_err(|error| Error::Internal(format!("opening store: {error}")))?;
    let _mutation_guard = StoreMutationGuard::acquire_wait(&config.state_dir)
        .map_err(|err| Error::Internal(format!("acquiring store mutation lock: {err}")))?;

    let mut session = crunch_eval::session::EvaluationSession::open_file(&config.file, &config.import_paths)
        .map_err(map_eval_error)?;

    let has_base_stores = !config.base_state_dirs.is_empty();
    let opened = if has_base_stores {
        crunch_store::StoreHandle::open_overlay(store_config).await
    } else {
        crunch_store::StoreHandle::open(store_config).await
    };
    let mut store = match opened {
        Ok(store) => store,
        Err(err @ crunch_store::Error::PathInfoFallbackRejected { .. }) => {
            let derivations = session.force_all_roots::<CrunchDerivation>().map_err(map_eval_error)?;
            debug_assert!(!derivations.is_empty(), "must have at least one derivation");
            let result = build_preflight_failure(config, &derivations, err.to_string())?;
            if let Some(slot) = causal_trace_slot {
                *slot = Some(CausalTraceObservation::WorkerNotStarted);
            }
            return Ok(result);
        }
        Err(error) => {
            let store_kind = if has_base_stores { "overlay store" } else { "store" };
            return Err(Error::Internal(format!("opening {store_kind}: {error}")));
        }
    };
    if config.backend == crunch_store::StoreBackend::Casita {
        store
            .recover_casita_gc_under_guard(&_mutation_guard)
            .await
            .map_err(|error| Error::Internal(format!("recovering Casita GC before build: {error}")))?;
    }
    let mut hermeticity_audit_events = mode_audit_events(config.hermeticity_mode);
    hermeticity_audit_events.extend(map_store_audit_events(store.startup_audit_events()));

    #[cfg(target_os = "linux")]
    {
        return build_linux(LinuxBuildRequest {
            config,
            store,
            session,
            hermeticity_audit_events,
            cancellation,
            stream_tx,
            worker_observation,
            causal_trace_slot,
        })
        .await;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (store, cancellation, stream_tx, worker_observation, causal_trace_slot);
        Err(Error::Build("building is only supported on Linux (requires bwrap)".to_string()))
    }
}

#[cfg(target_os = "linux")]
struct PipelineBuilderBundle<S> {
    builder: Builder<S>,
    output_lookup: crunch_store::OutputLookup,
    root_registry: crunch_store::RootRegistry,
    workspace_evidence_sink: crunch_build::WorkspaceReportCollector,
}

#[cfg(target_os = "linux")]
struct LinuxBuildRequest<'a> {
    config: &'a BuildConfig,
    store: crunch_store::StoreHandle,
    session: crunch_eval::session::EvaluationSession,
    hermeticity_audit_events: Vec<HermeticityAuditEvent>,
    cancellation: EvaluationCancellation,
    stream_tx: Option<mpsc::Sender<crunch_evaluation_stream_core::StreamRecordValue>>,
    worker_observation: Option<WorkerLiveObservation>,
    causal_trace_slot: Option<&'a mut Option<CausalTraceObservation>>,
}

#[cfg(target_os = "linux")]
fn capture_causal_trace(worker: &mut Worker, slot: Option<&mut Option<CausalTraceObservation>>) -> Result<(), Error> {
    if let Some(slot) = slot {
        let trace = worker
            .take_causal_trace()
            .map_err(|error| Error::Internal(format!("causal trace: {error}")))?
            .ok_or_else(|| Error::Internal("causal trace was not enabled".to_string()))?;
        *slot = Some(CausalTraceObservation::Recorded(trace));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
async fn build_linux(request: LinuxBuildRequest<'_>) -> Result<PipelineResult, Error> {
    let config = request.config;
    debug_assert!(!config.store_dir.is_empty());
    debug_assert!(config.max_jobs >= 1);

    let bundle = create_pipeline_builder(config, request.store)?;
    let mut builder = bundle.builder;
    let workspace_evidence_sink = bundle.workspace_evidence_sink;
    let _output_lookup = bundle.output_lookup;
    let root_registry = bundle.root_registry;

    let (tx, mut rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
    let mut known_paths = DerivationRegistry::new(&config.store_dir);
    let mut worker = Worker::with_scheduling_policy(config.max_jobs, config.scheduling_policy.clone())
        .map_err(|error| Error::Build(format!("scheduler policy: {error}")))?;
    if request.causal_trace_slot.is_some() {
        worker.enable_causal_trace(&config.store_dir);
    }
    let mut worker_observation = request.worker_observation.map(|observation| {
        let mut previous: Option<Arc<WorkerLiveSnapshot>> = None;
        move |registry: &crunch_build::GoalRegistry| {
            if observation.degraded.load(Ordering::Acquire) {
                return;
            }
            if registry.len() as usize > MAX_LIVE_WORKER_GOALS {
                observation.degraded.store(true, Ordering::Release);
                return;
            }
            if previous.as_ref().is_some_and(|last| {
                last.goals.len() == registry.len() as usize
                    && registry.iter().all(|(drv_key, goal)| {
                        last.goals
                            .binary_search_by(|row| row.drv_key.as_str().cmp(drv_key))
                            .is_ok_and(|index| last.goals[index].state == goal.state)
                    })
            }) {
                return;
            }
            let mut goals: Vec<_> = registry
                .iter()
                .map(|(drv_key, goal)| WorkerLiveGoal {
                    drv_key: drv_key.to_owned(),
                    state: goal.state.clone(),
                })
                .collect();
            goals.sort_unstable_by(|left, right| left.drv_key.cmp(&right.drv_key));
            let snapshot = Arc::new(WorkerLiveSnapshot { goals });
            previous = Some(Arc::clone(&snapshot));
            if observation.sender.try_send(snapshot).is_err() {
                observation.degraded.store(true, Ordering::Release);
            }
        }
    });
    let worker_run = worker.run_streaming_observed(
        &mut builder,
        &mut known_paths,
        &mut rx,
        worker_observation.as_mut().map(|observe| observe as &mut dyn FnMut(&crunch_build::GoalRegistry)),
    );
    let eval_stream = stream_roots_into_worker(EvalStreamRequest {
        max_jobs: config.max_jobs,
        store_dir: &config.store_dir,
        root_force_policy: RootForceExecutionPolicy::PreferThreaded,
        root_file: &config.file,
        import_paths: &config.import_paths,
        session: &request.session,
        tx,
        cancellation: request.cancellation,
        stream_tx: request.stream_tx,
        worker_control: EvalWorkerControl::default(),
    });
    let (worker_run, eval_stream) = tokio::join!(worker_run, eval_stream);
    let worker_result = match worker_run {
        Ok(result) => result,
        Err(err) => return Err(Error::Build(format!("{err}"))),
    };
    let eval_stream = eval_stream?;
    capture_causal_trace(&mut worker, request.causal_trace_slot)?;
    let source_generation_paths = builder.source_generation_paths();
    let overlay_evidence = builder
        .overlay_report()
        .map_err(|error| Error::Build(format!("collecting overlay build evidence: {error}")))?;
    let store_layer_selections = builder.take_store_layer_selections();
    let mut hermeticity_audit_events = request.hermeticity_audit_events;
    hermeticity_audit_events.extend(builder.take_hermeticity_audit_events());
    let pipeline_evidence = PipelineRunEvidence {
        hermeticity_audit_events,
        build_environment_rows: builder.take_build_environment_reports(),
        finish_gate_rows: builder.take_finish_gate_reports(),
        network_policy_rows: builder.take_network_policy_reports(),
        workspace_rows: workspace_evidence_sink.take(),
        action_result_rows: builder.take_action_result_reports(),
        overlay_report: overlay_evidence,
        store_layer_selections,
    };
    let result = finish_pipeline_result(
        &config.store_dir,
        config.hermeticity_mode,
        worker_result,
        eval_stream,
        pipeline_evidence,
    );
    register_managed_generation(
        config.root_registration.as_ref(),
        config.root_retention_source,
        &root_registry,
        &result.outcomes,
        &source_generation_paths,
        result.failed.is_empty(),
    )
    .await?;
    Ok(result)
}

pub async fn build_registered_derivations(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
    known_paths: &mut DerivationRegistry,
    request: RegisteredBuildRequest<'_>,
) -> Result<RegisteredBuildResult, Error> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (config, store, known_paths, request);
        return Err(Error::Build("building is only supported on Linux (requires bwrap)".to_string()));
    }
    #[cfg(target_os = "linux")]
    {
        if request.roots.is_empty() {
            return Err(Error::Build("registered derivation build requires at least one root".to_string()));
        }
        if request.cache_only {
            let bundle = create_cache_only_observer(config, store)?;
            return run_registered_builder(
                config,
                bundle.builder,
                known_paths,
                request,
                bundle.output_lookup,
                bundle.root_registry,
                bundle.workspace_evidence_sink,
            )
            .await;
        }
        let bundle = create_pipeline_builder_with_source_policy(config, store, request.source_policy)?;
        run_registered_builder(
            config,
            bundle.builder,
            known_paths,
            request,
            bundle.output_lookup,
            bundle.root_registry,
            bundle.workspace_evidence_sink,
        )
        .await
    }
}

#[cfg(target_os = "linux")]
async fn register_managed_generation(
    base_registration: Option<&crunch_store::RootRegistration>,
    root_retention_source: Option<GcRootSource>,
    root_registry: &crunch_store::RootRegistry,
    outcomes: &[BuildOutcome],
    source_paths: &[nix_compat::store_path::StorePath<String>],
    include_source_generation: bool,
) -> Result<(), Error> {
    let Some(base_registration) = base_registration else {
        return Ok(());
    };
    let local_source = root_retention_source.unwrap_or(GcRootSource::Build);
    let mut seen_paths = BTreeSet::new();
    let mut registrations = Vec::new();
    for outcome in outcomes {
        for (output_name, path_info) in &outcome.outputs {
            if !seen_paths.insert(path_info.store_path.clone()) {
                continue;
            }
            let source = if outcome.substitutions.contains_key(output_name) {
                GcRootSource::Remote
            } else {
                local_source
            };
            registrations.push((path_info.store_path.clone(), source, base_registration.clone()));
        }
    }
    if include_source_generation && base_registration.class == crunch_store::GcRootClass::ProjectOutputGeneration {
        for source_path in source_paths {
            if !seen_paths.insert(source_path.clone()) {
                continue;
            }
            let mut source_registration = base_registration.clone();
            source_registration.class = crunch_store::GcRootClass::ProjectSourceGeneration;
            source_registration.generation = None;
            source_registration.lease = None;
            registrations.push((source_path.clone(), GcRootSource::Source, source_registration));
        }
    }
    root_registry
        .register_managed_batch(registrations)
        .await
        .map_err(|error| Error::Build(format!("committing managed root generation: {error}")))?;
    Ok(())
}

#[cfg(target_os = "linux")]
async fn run_registered_builder<S: snix_build::buildservice::BuildService + 'static>(
    config: &BuildConfig,
    mut builder: Builder<S>,
    known_paths: &mut DerivationRegistry,
    request: RegisteredBuildRequest<'_>,
    output_lookup: crunch_store::OutputLookup,
    root_registry: crunch_store::RootRegistry,
    workspace_evidence_sink: crunch_build::WorkspaceReportCollector,
) -> Result<RegisteredBuildResult, Error> {
    let mut worker_result = builder
        .build_all_report(request.roots, known_paths, config.max_jobs)
        .await
        .map_err(|error| Error::Build(error.to_string()))?;
    let source_generation_paths = builder.source_generation_paths();
    normalize_failed_goal_keys(&mut worker_result.failed, &config.store_dir);
    let mut outputs = Vec::with_capacity(request.expected_outputs.len());
    for expected in request.expected_outputs {
        let path_info = output_lookup
            .find(&expected.store_path)
            .await
            .map_err(|error| Error::Build(format!("registered output lookup failed: {error}")))?;
        if let Some(path_info) = path_info {
            if path_info.store_path != expected.store_path {
                return Err(Error::Build(format!(
                    "registered output lookup returned a conflicting path: {}",
                    expected.store_path
                )));
            }
            outputs.push(RegisteredOutputResult {
                unit_id: expected.unit_id.clone(),
                output_name: expected.output_name.clone(),
                path_info,
            });
        }
    }
    register_managed_generation(
        config.root_registration.as_ref(),
        config.root_retention_source,
        &root_registry,
        &worker_result.outcomes,
        &source_generation_paths,
        worker_result.failed.is_empty(),
    )
    .await?;
    for retained_output in request.retained_outputs {
        root_registry
            .register_if_present(retained_output, GcRootSource::Build)
            .await
            .map_err(|error| Error::Build(format!("registering selected foreign root: {error}")))?;
    }
    assert_eq!(worker_result.outcomes.len().saturating_add(worker_result.failed.len()), request.roots.len());
    if worker_result.failed.is_empty() {
        assert_eq!(outputs.len(), request.expected_outputs.len());
    }
    Ok(RegisteredBuildResult {
        outcomes: worker_result.all_outcomes,
        failed_roots: worker_result.failed,
        outputs,
        hermeticity_audit_events: builder.take_hermeticity_audit_events(),
        build_environment_reports: builder.take_build_environment_reports(),
        finish_gate_reports: builder.take_finish_gate_reports(),
        network_policy_reports: builder.take_network_policy_reports(),
        workspace_reports: workspace_evidence_sink.take(),
        action_result_reports: builder.take_action_result_reports(),
    })
}

#[cfg(target_os = "linux")]
fn create_cache_only_observer(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
) -> Result<PipelineBuilderBundle<impl snix_build::buildservice::BuildService + use<>>, Error> {
    let crunch_store::PipelineStoreParts {
        build_store,
        action_results,
        slice_admission,
        build_service_store,
        output_lookup,
        root_registry,
    } = store.into_pipeline_store_parts();
    let service = FetchBuildService::new(build_service_store).with_source_policy(FetchSourcePolicy::RequireOverride);
    let workspace_evidence_sink = empty_workspace_report_collector();
    let mut builder = Builder::from_store_parts(
        crunch_store::BuilderStoreParts {
            build_store,
            action_results,
            slice_admission,
        },
        service,
        config.keypair.clone(),
        config.trusted_keys.clone(),
        false,
        config.verbose,
    );
    builder.set_hermeticity_mode(HermeticityMode::Strict);
    Ok(PipelineBuilderBundle {
        builder,
        output_lookup,
        root_registry,
        workspace_evidence_sink,
    })
}

#[cfg(target_os = "linux")]
fn source_policy_for_config(config: &BuildConfig) -> FetchSourcePolicy {
    if config.source_fetch_overrides.is_empty() {
        FetchSourcePolicy::AllowNetwork
    } else {
        FetchSourcePolicy::RequireOverride
    }
}

#[cfg(target_os = "linux")]
fn create_pipeline_builder(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
) -> Result<PipelineBuilderBundle<impl snix_build::buildservice::BuildService + use<>>, Error> {
    create_pipeline_builder_with_source_policy(config, store, source_policy_for_config(config))
}

#[cfg(target_os = "linux")]
fn create_pipeline_builder_with_source_policy(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
    source_policy: FetchSourcePolicy,
) -> Result<PipelineBuilderBundle<impl snix_build::buildservice::BuildService + use<>>, Error> {
    create_pipeline_builder_with_sandbox(config, store, source_policy, |build_service_store| {
        let workdir = std::env::temp_dir().join("crunch-builds");
        std::fs::create_dir_all(&workdir).map_err(|error| Error::Internal(format!("create workdir: {error}")))?;
        // Keep remote dispatch inside the lazy scheduler so dedupe, waiter
        // notification, job bounds, fallback, and terminal propagation stay shared.
        // r[impl remote_builds.production_scheduler_realization]
        let profile = crunch_build::RealizerProfileFacts {
            name: "local-sandbox".to_string(),
            version: 1,
            capabilities: Vec::new(),
            parameters: std::collections::BTreeMap::new(),
        };
        let local_bwrap =
            build_service_store.bubblewrap_build_service(std::env::temp_dir().join("crunch-builds-local"), None);
        let remote_bwrap = build_service_store.bubblewrap_build_service(workdir, None);
        let remote_realizer = LocalBuildServiceRealizer::new(remote_bwrap, profile);
        Ok(RemoteFirstBuildService::new(
            remote_realizer,
            local_bwrap,
            RemoteBuildFallbackPolicy::OnRemoteFailure,
        ))
    })
}

/// Watch admission uses only the local sandbox with proven owned teardown.
/// The one-shot builder retains its remote-first fallback unchanged.
#[cfg(target_os = "linux")]
fn create_watch_pipeline_builder(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
) -> Result<PipelineBuilderBundle<impl snix_build::buildservice::BuildService + use<>>, Error> {
    create_pipeline_builder_with_sandbox(config, store, source_policy_for_config(config), |build_service_store| {
        Ok(build_service_store.bubblewrap_build_service(std::env::temp_dir().join("crunch-builds-local"), None))
    })
}

#[cfg(target_os = "linux")]
fn create_pipeline_builder_with_sandbox<S, F>(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
    source_policy: FetchSourcePolicy,
    make_sandbox: F,
) -> Result<PipelineBuilderBundle<impl snix_build::buildservice::BuildService + use<S, F>>, Error>
where
    S: snix_build::buildservice::BuildService + 'static,
    F: FnOnce(&crunch_store::BuildServiceStore) -> Result<S, Error>,
{
    debug_assert!(!config.store_dir.is_empty(), "store prefix must not be empty");
    debug_assert!(config.max_jobs >= 1, "builder requires at least one job");

    let crunch_store::PipelineStoreParts {
        build_store,
        action_results,
        slice_admission,
        build_service_store,
        output_lookup,
        root_registry,
    } = store.into_pipeline_store_parts();
    let state_dir = build_store.state_dir().to_path_buf();
    let fetch_service = FetchBuildService::new(build_service_store.clone())
        .with_source_overrides(config.source_fetch_overrides.clone())
        .with_source_policy(source_policy);
    let sandbox_service = make_sandbox(&build_service_store)?;
    let dispatch = DispatchBuildService::new(fetch_service, sandbox_service);
    let workspace_evidence_sink = empty_workspace_report_collector();
    let build_service =
        crunch_build::StatefulWorkspaceBuildService::new(dispatch, &state_dir, workspace_evidence_sink.clone());
    let mut builder = Builder::from_store_parts(
        crunch_store::BuilderStoreParts {
            build_store,
            action_results,
            slice_admission,
        },
        build_service,
        config.keypair.clone(),
        config.trusted_keys.clone(),
        config.trust_unsigned,
        config.verbose,
    );
    builder.set_hermeticity_mode(config.hermeticity_mode);
    let has_managed_registration = config.root_registration.is_some();
    builder.set_root_retention_source(if has_managed_registration {
        None
    } else {
        config.root_retention_source
    });
    builder.set_root_registration(None);
    Ok(PipelineBuilderBundle {
        builder,
        output_lookup,
        root_registry,
        workspace_evidence_sink,
    })
}

struct PipelineRunEvidence {
    hermeticity_audit_events: Vec<HermeticityAuditEvent>,
    build_environment_rows: Vec<BuildEnvironmentReport>,
    finish_gate_rows: Vec<crunch_build::FinishGateReport>,
    network_policy_rows: Vec<BuildNetworkPolicyReport>,
    workspace_rows: Vec<crunch_build::WorkspaceExecutionReport>,
    action_result_rows: Vec<crunch_build::ActionResultRuntimeReport>,
    overlay_report: Option<crunch_store::StoreOverlayReport>,
    store_layer_selections: Vec<crunch_store::layer::StoreLayerSelection>,
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

struct EvalFailure {
    label: String,
    error: String,
}

fn finish_pipeline_result(
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
    mut worker_result: WorkerResult,
    eval_stream: EvalStreamResult,
    evidence: PipelineRunEvidence,
) -> PipelineResult {
    debug_assert!(!store_dir.is_empty(), "store prefix must not be empty");
    let eval_failures = evaluation_failures(&eval_stream.summary);
    let accounted_root_count = eval_stream.summary.counts().total().ok().and_then(|count| usize::try_from(count).ok());
    debug_assert_eq!(
        accounted_root_count,
        Some(eval_stream.summary.roots().len()),
        "evaluation summary must account for every selected root"
    );
    for eval_failure in &eval_failures {
        worker_result.failed.push(FailedGoal {
            drv_key: eval_failure_key(&eval_failure.label),
            origin_drv_key: eval_failure_key(&eval_failure.label),
            error: eval_failure.error.clone(),
            origin_error: eval_failure.error.clone(),
            build_log: None,
        });
    }
    normalize_failed_goal_keys(&mut worker_result.failed, store_dir);
    for binding in &mut worker_result.plan_output_bindings {
        if parse_drv_key(store_dir, &binding.consumer_drv_key).is_none() {
            binding.consumer_drv_key = normalized_failed_drv_key(&binding.consumer_drv_key, store_dir);
        }
    }
    let mut root_labels = build_root_labels(&eval_stream.root_drv_paths, store_dir);
    for eval_failure in &eval_failures {
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
        finish_gate_reports: evidence.finish_gate_rows,
        network_policy_reports: evidence.network_policy_rows,
        workspace_reports: evidence.workspace_rows,
        action_result_reports: evidence.action_result_rows,
        native_dynamic_plans: worker_result.native_dynamic_plans,
        plan_output_bindings: worker_result.plan_output_bindings,
        priority_decisions: worker_result.priority_decisions,
        overlay_report: evidence.overlay_report,
        store_layer_selections: evidence.store_layer_selections,
    }
}

fn evaluation_failures(summary: &crunch_evaluation_stream_core::RunSummary) -> Vec<EvalFailure> {
    summary
        .roots()
        .into_iter()
        .filter_map(|outcome| {
            if outcome.terminal_state() == crunch_evaluation_stream_core::TerminalState::Succeeded {
                return None;
            }
            let label = outcome.root().label();
            let error = outcome
                .diagnostic()
                .map(|diagnostic| diagnostic.text())
                .unwrap_or_else(|| terminal_state_diagnostic(outcome.terminal_state()).to_string());
            Some(EvalFailure { label, error })
        })
        .collect()
}

fn terminal_state_diagnostic(state: crunch_evaluation_stream_core::TerminalState) -> &'static str {
    match state {
        crunch_evaluation_stream_core::TerminalState::Succeeded => "evaluation succeeded",
        crunch_evaluation_stream_core::TerminalState::Failed => "evaluation failed",
        crunch_evaluation_stream_core::TerminalState::WorkerLost => "evaluation worker lost",
        crunch_evaluation_stream_core::TerminalState::Cancelled => "evaluation cancelled",
        crunch_evaluation_stream_core::TerminalState::NotStarted => "evaluation not started",
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
            origin_drv_key: drv_path.to_absolute_path_with_prefix(&config.store_dir),
            error: error.clone(),
            origin_error: error.clone(),
            build_log: None,
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
        finish_gate_reports: Vec::new(),
        network_policy_reports: Vec::new(),
        workspace_reports: Vec::new(),
        action_result_reports: Vec::new(),
        native_dynamic_plans: Vec::new(),
        plan_output_bindings: Vec::new(),
        priority_decisions: Vec::new(),
        overlay_report: None,
        store_layer_selections: Vec::new(),
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
        failed_goal.drv_key = normalized_failed_drv_key(&failed_goal.drv_key, store_dir);
        failed_goal.origin_drv_key = normalized_failed_drv_key(&failed_goal.origin_drv_key, store_dir);
    }
}

fn normalized_failed_drv_key(drv_key: &str, store_dir: &str) -> String {
    if parse_drv_key(store_dir, drv_key).is_some() {
        return drv_key.to_string();
    }
    let Ok(drv_path) = StorePath::from_absolute_path(drv_key.as_bytes()) else {
        return drv_key.to_string();
    };
    drv_key_for(store_dir, &drv_path)
}

fn collect_fod_mismatches(failed: &[FailedGoal]) -> Vec<FodMismatch> {
    failed.iter().filter_map(|failed_goal| parse_fod_mismatch_error(&failed_goal.error)).collect()
}

pub fn drv_key_for(store_dir: &str, drv_path: &StorePath<String>) -> String {
    drv_path.to_absolute_path_with_prefix(store_dir)
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "stable compatibility wrapper admits logical prefix and derivation key into distinct roles"
)]
pub fn parse_drv_key(store_dir: &str, drv_key: &str) -> Option<StorePath<String>> {
    let store_dir = crunch_build::LogicalStorePrefix::new(store_dir).ok()?;
    let drv_key = crunch_build::DerivationKey::new(drv_key).ok()?;
    StorePath::from_absolute_path_with_prefix(drv_key.as_str().as_bytes(), store_dir.as_str()).ok()
}

pub fn label_for_key<'a>(result: &'a PipelineResult, drv_key: &str) -> Option<&'a str> {
    result.root_labels.get(drv_key).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::PathInfoService;
    use tokio::sync::mpsc;

    use super::*;

    const MANAGED_TEST_STORE_PREFIX: &str = "/mantle/store";
    const MANAGED_TEST_NAR_DIGEST_BYTES: usize = 32;
    const MANAGED_TEST_NAR_SIZE: u64 = 1;
    const MANAGED_TEST_OUTPUT_DIGEST_BYTE: u8 = 31;
    const MANAGED_TEST_SOURCE_DIGEST_BYTE: u8 = 32;
    const MANAGED_TEST_DRV_DIGEST_BYTE: u8 = 33;
    const MANAGED_TEST_GENERATION: u64 = 1;
    const CANCELLATION_WAIT_TIMEOUT_MS: u64 = 30_000;
    const CANCELLATION_POLL_INTERVAL_MS: u64 = 1;
    const SINGLE_EVAL_JOB: u32 = 1;
    const SINGLE_ROOT_COUNT: u32 = 1;
    const TWO_ROOT_COUNT: u32 = 2;
    const THREE_ROOT_COUNT: u32 = 3;
    const ROOT_ALPHA_INDEX: usize = 0;
    const ROOT_BETA_INDEX: usize = 1;
    const ROOT_GAMMA_INDEX: usize = 2;

    async fn collect_eval_message_labels(mut rx: mpsc::Receiver<EvalMessage>) -> Vec<String> {
        let mut labels = Vec::new();
        while let Some(message) = rx.recv().await {
            labels.push(message.label);
        }
        labels
    }

    fn eval_stream_request<'a>(
        session: &'a crunch_eval::session::EvaluationSession,
        root_file: &'a std::path::Path,
        root_force_policy: RootForceExecutionPolicy,
        tx: mpsc::Sender<EvalMessage>,
    ) -> EvalStreamRequest<'a> {
        EvalStreamRequest {
            max_jobs: EVAL_POLICY_TEST_MAX_JOBS,
            store_dir: "/crunch/store",
            root_force_policy,
            root_file,
            import_paths: &[],
            session,
            tx,
            cancellation: EvaluationCancellation::new(),
            stream_tx: None,
            worker_control: EvalWorkerControl::default(),
        }
    }

    fn managed_test_path(name: &str, digest_byte: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [digest_byte; nix_compat::store_path::DIGEST_SIZE])
            .expect("valid managed-generation test path")
    }

    fn managed_test_path_info(store_path: StorePath<String>) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("managed-generation-target")
                    .expect("valid managed-generation symlink target"),
            },
            references: Vec::new(),
            nar_size: MANAGED_TEST_NAR_SIZE,
            nar_sha256: [MANAGED_TEST_OUTPUT_DIGEST_BYTE; MANAGED_TEST_NAR_DIGEST_BYTES],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        }
    }

    fn managed_test_registration() -> crunch_store::RootRegistration {
        crunch_store::RootRegistration {
            class: crunch_store::GcRootClass::ProjectOutputGeneration,
            owner_scope: "project:b3:pipeline-test".to_string(),
            project_identity: Some("b3:pipeline-test".to_string()),
            selector: Some("default".to_string()),
            generation: None,
            generation_identity: Some("b3:pipeline-lock-test".to_string()),
            lease: None,
            removal_requested: false,
        }
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
            origin_drv_key: drv_path.to_absolute_path(),
            error: "boom".to_string(),
            origin_error: "boom".to_string(),
            build_log: None,
        }];

        normalize_failed_goal_keys(&mut failed, "/crunch/store");

        assert_eq!(failed[0].drv_key, drv_path.to_absolute_path_with_prefix("/crunch/store"));
        assert_eq!(failed[0].origin_drv_key, failed[0].drv_key);
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

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn managed_generation_defers_sources_until_success_then_commits_output_and_source() {
        let state = tempfile::tempdir().expect("managed-generation state directory");
        let output = tempfile::tempdir().expect("managed-generation output directory");
        let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig::new(
            crunch_store::StoreBackend::Snix,
            state.path().to_path_buf(),
            output.path().to_path_buf(),
            MANAGED_TEST_STORE_PREFIX.to_string(),
        ))
        .await
        .expect("open managed-generation store");
        let pathinfo_service = store.pathinfo_service();
        let output_path = managed_test_path("managed-output", MANAGED_TEST_OUTPUT_DIGEST_BYTE);
        let source_path = managed_test_path("managed-source", MANAGED_TEST_SOURCE_DIGEST_BYTE);
        let output_path_info = managed_test_path_info(output_path.clone());
        pathinfo_service.put(output_path_info.clone()).await.expect("persist managed output PathInfo");
        pathinfo_service
            .put(managed_test_path_info(source_path.clone()))
            .await
            .expect("persist managed source PathInfo");
        let root_registry = store.into_pipeline_store_parts().root_registry;
        let outcome = BuildOutcome {
            drv_path: managed_test_path("managed.drv", MANAGED_TEST_DRV_DIGEST_BYTE),
            outputs: BTreeMap::from([("out".to_string(), output_path_info)]),
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };
        let registration = managed_test_registration();

        register_managed_generation(
            Some(&registration),
            Some(GcRootSource::Build),
            &root_registry,
            std::slice::from_ref(&outcome),
            std::slice::from_ref(&source_path),
            false,
        )
        .await
        .expect("failed pipeline keeps source generation deferred");
        let incomplete_roots = root_registry.list().expect("list incomplete generation roots");
        assert_eq!(incomplete_roots.len(), 1);
        assert_eq!(incomplete_roots[0].root_class, crunch_store::GcRootClass::ProjectOutputGeneration);

        register_managed_generation(
            Some(&registration),
            Some(GcRootSource::Build),
            &root_registry,
            std::slice::from_ref(&outcome),
            std::slice::from_ref(&source_path),
            true,
        )
        .await
        .expect("successful pipeline commits output and source generation");
        let complete_roots = root_registry.list().expect("list complete generation roots");
        assert_eq!(complete_roots.len(), 2);
        assert_eq!(complete_roots[0].generation, Some(MANAGED_TEST_GENERATION));
        assert_eq!(complete_roots[1].generation, Some(MANAGED_TEST_GENERATION));
        assert!(complete_roots.iter().any(|record| {
            record.logical_path == output_path.to_absolute_path_with_prefix(MANAGED_TEST_STORE_PREFIX)
                && record.root_class == crunch_store::GcRootClass::ProjectOutputGeneration
                && record.source == GcRootSource::Build
        }));
        assert!(complete_roots.iter().any(|record| {
            record.logical_path == source_path.to_absolute_path_with_prefix(MANAGED_TEST_STORE_PREFIX)
                && record.root_class == crunch_store::GcRootClass::ProjectSourceGeneration
                && record.source == GcRootSource::Source
        }));
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
        let (inline_tx, inline_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let inline_result = stream_roots_into_worker(eval_stream_request(
            &session,
            &root_file,
            RootForceExecutionPolicy::Inline,
            inline_tx,
        ))
        .await
        .unwrap();
        let inline_labels = collect_eval_message_labels(inline_rx).await;

        let (preferred_tx, preferred_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let preferred_result = stream_roots_into_worker(eval_stream_request(
            &session,
            &root_file,
            RootForceExecutionPolicy::PreferThreaded,
            preferred_tx,
        ))
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
        assert_eq!(inline_result.summary, preferred_result.summary);
        assert_eq!(inline_result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Success);
    }

    #[tokio::test]
    async fn evaluation_stream_emits_start_discovery_terminals_and_final_summary() {
        const STREAM_TEST_CHANNEL_CAPACITY: usize = 16;

        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (build_tx, build_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let (stream_tx, mut stream_rx) =
            mpsc::channel::<crunch_evaluation_stream_core::StreamRecordValue>(STREAM_TEST_CHANNEL_CAPACITY);
        let mut request = eval_stream_request(&session, &root_file, RootForceExecutionPolicy::Inline, build_tx);
        request.stream_tx = Some(stream_tx);
        let result = stream_roots_into_worker(request).await.unwrap();
        let labels = collect_eval_message_labels(build_rx).await;
        let mut records = Vec::new();
        while let Some(record) = stream_rx.recv().await {
            records.push(record);
        }

        assert_eq!(labels.len(), TWO_ROOT_COUNT as usize);
        assert_eq!(records.len(), 6);
        assert!(matches!(records.first(), Some(crunch_evaluation_stream_core::StreamRecordValue::RunStart(_))));
        assert!(matches!(records[1], crunch_evaluation_stream_core::StreamRecordValue::RootDiscovered(_)));
        assert!(matches!(records[2], crunch_evaluation_stream_core::StreamRecordValue::RootDiscovered(_)));
        assert!(matches!(records[3], crunch_evaluation_stream_core::StreamRecordValue::RootTerminal(_)));
        assert!(matches!(records[4], crunch_evaluation_stream_core::StreamRecordValue::RootTerminal(_)));
        let crunch_evaluation_stream_core::StreamRecordValue::RunSummary(summary) = &records[5] else {
            panic!("last stream record must be run-summary");
        };
        assert_eq!(summary, &result.summary);
        assert_eq!(summary.roots()[ROOT_ALPHA_INDEX].root().label(), "alpha");
        assert_eq!(summary.roots()[ROOT_BETA_INDEX].root().label(), "beta");
    }

    #[tokio::test]
    async fn closed_stream_consumer_requests_cancellation_before_dispatch() {
        const CLOSED_STREAM_CHANNEL_CAPACITY: usize = 1;

        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{ alpha = { name = "alpha", builder = "/bin/sh" } }"#,
            &[],
        )
        .unwrap();
        let (build_tx, mut build_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let (stream_tx, stream_rx) =
            mpsc::channel::<crunch_evaluation_stream_core::StreamRecordValue>(CLOSED_STREAM_CHANNEL_CAPACITY);
        drop(stream_rx);
        let cancellation = EvaluationCancellation::new();
        let mut request = eval_stream_request(&session, &root_file, RootForceExecutionPolicy::Inline, build_tx);
        request.cancellation = cancellation.clone();
        request.stream_tx = Some(stream_tx);
        let error = match stream_roots_into_worker(request).await {
            Ok(_) => panic!("closed stream consumer must fail"),
            Err(error) => error,
        };

        assert!(cancellation.is_requested());
        assert!(error.to_string().contains("consumer closed before run-summary"));
        assert!(build_rx.recv().await.is_none());
    }

    #[tokio::test]
    async fn malformed_sibling_does_not_suppress_independent_success() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  bad = { builder = "/bin/sh" },
  good = { name = "good", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (inline_tx, inline_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let mut inline_request = eval_stream_request(&session, &root_file, RootForceExecutionPolicy::Inline, inline_tx);
        inline_request.max_jobs = SINGLE_EVAL_JOB;
        let inline_result = stream_roots_into_worker(inline_request).await.unwrap();
        let inline_labels = collect_eval_message_labels(inline_rx).await;

        let (preferred_tx, preferred_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let mut preferred_request =
            eval_stream_request(&session, &root_file, RootForceExecutionPolicy::PreferThreaded, preferred_tx);
        preferred_request.max_jobs = SINGLE_EVAL_JOB;
        let preferred_result = stream_roots_into_worker(preferred_request).await.unwrap();
        let preferred_labels = collect_eval_message_labels(preferred_rx).await;

        assert_eq!(inline_labels, vec!["good".to_string()]);
        assert_eq!(preferred_labels, vec!["good".to_string()]);
        assert_eq!(inline_result.summary, preferred_result.summary);
        assert_eq!(inline_result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Partial);
        assert_eq!(inline_result.summary.counts().succeeded, SINGLE_ROOT_COUNT);
        assert_eq!(inline_result.summary.counts().failed, SINGLE_ROOT_COUNT);
        let bad = inline_result
            .summary
            .roots()
            .into_iter()
            .find(|outcome| outcome.root().label() == "bad")
            .expect("bad root outcome");
        assert_eq!(bad.terminal_state(), crunch_evaluation_stream_core::TerminalState::Failed);
        assert_eq!(bad.failure_scope(), Some(crunch_evaluation_stream_core::FailureScope::RootScoped));
    }

    #[tokio::test]
    async fn conversion_failure_does_not_suppress_later_root() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  a_bad = { name = "a-bad", builder = "/bin/sh", addressing_mode = "bogus" },
  z_good = { name = "z-good", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (tx, rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let mut request = eval_stream_request(&session, &root_file, RootForceExecutionPolicy::Inline, tx);
        request.max_jobs = SINGLE_EVAL_JOB;
        let result = stream_roots_into_worker(request).await.unwrap();
        let labels = collect_eval_message_labels(rx).await;
        let roots = result.summary.roots();

        assert_eq!(labels, vec!["z_good".to_string()]);
        assert_eq!(result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Partial);
        assert_eq!(roots[ROOT_ALPHA_INDEX].root().label(), "a_bad");
        assert_eq!(roots[ROOT_ALPHA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::Failed);
        assert_eq!(
            roots[ROOT_ALPHA_INDEX].failure_scope(),
            Some(crunch_evaluation_stream_core::FailureScope::RootScoped)
        );
        assert_eq!(roots[ROOT_BETA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::Succeeded);
    }

    #[tokio::test]
    async fn recursive_record_root_evaluates_without_suppressing_its_sibling() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  base = { name = "base", builder = "/bin/sh" },
  derived = { name = base.name ++ "-derived", builder = base.builder },
}"#,
            &[],
        )
        .unwrap();
        let (tx, rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let result = stream_roots_into_worker(eval_stream_request(
            &session,
            &root_file,
            RootForceExecutionPolicy::PreferThreaded,
            tx,
        ))
        .await
        .unwrap();
        let mut labels = collect_eval_message_labels(rx).await;
        labels.sort();

        assert_eq!(labels, vec!["base".to_string(), "derived".to_string()]);
        assert_eq!(result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Success);
        assert_eq!(result.summary.counts().succeeded, TWO_ROOT_COUNT);
        assert_eq!(result.summary.counts().total().unwrap(), TWO_ROOT_COUNT);
    }

    #[tokio::test]
    async fn worker_loss_stops_dispatch_and_accounts_for_every_root() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
  gamma = { name = "gamma", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (tx, rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let mut request = eval_stream_request(&session, &root_file, RootForceExecutionPolicy::Inline, tx);
        request.max_jobs = SINGLE_EVAL_JOB;
        request.worker_control = EvalWorkerControl::panic_on("alpha".to_string());
        let result = stream_roots_into_worker(request).await.unwrap();
        let labels = collect_eval_message_labels(rx).await;
        let roots = result.summary.roots();

        assert!(labels.is_empty());
        assert_eq!(result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Failed);
        assert_eq!(roots[ROOT_ALPHA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::WorkerLost);
        assert_eq!(roots[ROOT_BETA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::NotStarted);
        assert_eq!(roots[ROOT_GAMMA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::NotStarted);
        assert_eq!(result.summary.counts().total().unwrap(), THREE_ROOT_COUNT);
    }

    #[tokio::test]
    async fn shared_initialization_failure_prevents_dispatch_and_accounts_for_roots() {
        let directory = tempfile::tempdir().unwrap();
        let missing_root_file = directory.path().join("missing").join("root.ncl");
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (tx, rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let result = stream_roots_into_worker(eval_stream_request(
            &session,
            &missing_root_file,
            RootForceExecutionPolicy::Inline,
            tx,
        ))
        .await
        .unwrap();
        let labels = collect_eval_message_labels(rx).await;
        let roots = result.summary.roots();

        assert!(labels.is_empty());
        assert_eq!(result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Failed);
        assert!(roots.iter().all(|outcome| {
            outcome.terminal_state() == crunch_evaluation_stream_core::TerminalState::NotStarted
                && outcome.failure_scope() == Some(crunch_evaluation_stream_core::FailureScope::SharedFatal)
        }));
        assert_eq!(result.summary.counts().not_started, TWO_ROOT_COUNT);
        assert_eq!(result.summary.counts().total().unwrap(), TWO_ROOT_COUNT);
    }

    #[tokio::test]
    async fn opaque_evaluator_failure_stops_dispatch_as_shared_fatal() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  a_bad = { name | Number = "not-a-number", builder = "/bin/sh" },
  z_good = { name = "z-good", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let (tx, rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let mut request = eval_stream_request(&session, &root_file, RootForceExecutionPolicy::Inline, tx);
        request.max_jobs = SINGLE_EVAL_JOB;
        let result = stream_roots_into_worker(request).await.unwrap();
        let labels = collect_eval_message_labels(rx).await;
        let roots = result.summary.roots();

        assert!(labels.is_empty());
        assert_eq!(result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Failed);
        assert_eq!(roots[ROOT_ALPHA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::Failed);
        assert_eq!(
            roots[ROOT_ALPHA_INDEX].failure_scope(),
            Some(crunch_evaluation_stream_core::FailureScope::SharedFatal)
        );
        assert_eq!(roots[ROOT_BETA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::NotStarted);
        assert_eq!(
            roots[ROOT_BETA_INDEX].failure_scope(),
            Some(crunch_evaluation_stream_core::FailureScope::SharedFatal)
        );
    }

    #[tokio::test]
    async fn cancellation_rejects_late_success_and_stops_new_dispatch() {
        let (_directory, root_file) = resolver_fixture_file();
        let session = crunch_eval::session::EvaluationSession::open_str(
            r#"{
  alpha = { name = "alpha", builder = "/bin/sh" },
  beta = { name = "beta", builder = "/bin/sh" },
  gamma = { name = "gamma", builder = "/bin/sh" },
}"#,
            &[],
        )
        .unwrap();
        let cancellation = EvaluationCancellation::new();
        let evaluation_completed = Arc::new(AtomicBool::new(false));
        let release_evaluation = Arc::new(AtomicBool::new(false));
        let control = EvalWorkerControl::pause_after_evaluation(
            "beta".to_string(),
            evaluation_completed.clone(),
            release_evaluation.clone(),
        );
        let (tx, rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let mut request = eval_stream_request(&session, &root_file, RootForceExecutionPolicy::Inline, tx);
        request.max_jobs = SINGLE_EVAL_JOB;
        request.cancellation = cancellation.clone();
        request.worker_control = control;

        let controller = async {
            let reached_boundary = tokio::time::timeout(Duration::from_millis(CANCELLATION_WAIT_TIMEOUT_MS), async {
                while !evaluation_completed.load(Ordering::Acquire) {
                    tokio::time::sleep(Duration::from_millis(CANCELLATION_POLL_INTERVAL_MS)).await;
                }
            })
            .await
            .is_ok();
            if !reached_boundary {
                release_evaluation.store(true, Ordering::Release);
                panic!("evaluation did not reach the cancellation race boundary");
            }
            cancellation.request();
            release_evaluation.store(true, Ordering::Release);
        };
        let (result, ()) = tokio::join!(stream_roots_into_worker(request), controller);
        let result = result.unwrap();
        let labels = collect_eval_message_labels(rx).await;
        let roots = result.summary.roots();

        assert_eq!(labels, vec!["alpha".to_string()]);
        assert_eq!(result.summary.disposition(), crunch_evaluation_stream_core::RunDisposition::Cancelled);
        assert_eq!(roots[ROOT_ALPHA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::Succeeded);
        assert_eq!(roots[ROOT_BETA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::Cancelled);
        assert_eq!(roots[ROOT_GAMMA_INDEX].terminal_state(), crunch_evaluation_stream_core::TerminalState::NotStarted);
        assert_eq!(result.summary.counts().total().unwrap(), THREE_ROOT_COUNT);
    }
}
