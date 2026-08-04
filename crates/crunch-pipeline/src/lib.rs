#![feature(register_tool)]
#![register_tool(tigerstyle)]
// r[impl foreign_derivation_import.realization_adapter]
// r[impl foreign_derivation_import.realization_receipt]
// r[impl foreign_derivation_import.cache_only_runtime_closure]
// r[impl foreign_derivation_import.live_nixpkgs_realization_proof]

mod derivation_file;

use std::collections::BTreeSet;
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

const EVAL_MESSAGE_CHANNEL_CAPACITY: usize = 16;
#[cfg(test)]
const EVAL_POLICY_TEST_MAX_JOBS: u32 = 4;

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
    /// Optional managed provenance for successful selected roots.
    pub root_registration: Option<crunch_store::RootRegistration>,
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
struct PipelineBuilderBundle<S> {
    builder: Builder<S>,
    output_lookup: crunch_store::OutputLookup,
    root_registry: crunch_store::RootRegistry,
    workspace_evidence_sink: crunch_build::WorkspaceReportCollector,
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

    let bundle = create_pipeline_builder(config, store)?;
    let mut builder = bundle.builder;
    let workspace_evidence_sink = bundle.workspace_evidence_sink;
    let _output_lookup = bundle.output_lookup;
    let root_registry = bundle.root_registry;

    let (tx, mut rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
    let mut known_paths = DerivationRegistry::new(&config.store_dir);
    let mut worker = Worker::with_scheduling_policy(config.max_jobs, config.scheduling_policy.clone())
        .map_err(|error| Error::Build(format!("scheduler policy: {error}")))?;
    let worker_run = worker.run_streaming(&mut builder, &mut known_paths, &mut rx);
    let eval_stream = stream_roots_into_worker(EvalStreamRequest {
        max_jobs: config.max_jobs,
        store_dir: &config.store_dir,
        root_force_policy: RootForceExecutionPolicy::PreferThreaded,
        root_file: &config.file,
        import_paths: &config.import_paths,
        session: &session,
        tx,
    });
    let (worker_run, eval_stream) = tokio::join!(worker_run, eval_stream);
    let worker_result = match worker_run {
        Ok(result) => result,
        Err(err) => return Err(Error::Build(format!("{err}"))),
    };
    let eval_stream = eval_stream?;
    let source_generation_paths = builder.source_generation_paths();
    let overlay_report = builder
        .overlay_report()
        .map_err(|error| Error::Build(format!("collecting overlay build evidence: {error}")))?;
    let store_layer_selections = builder.take_store_layer_selections();
    let mut hermeticity_audit_events = hermeticity_audit_events;
    hermeticity_audit_events.extend(builder.take_hermeticity_audit_events());
    let pipeline_evidence = PipelineRunEvidence {
        hermeticity_audit_events,
        build_environment_rows: builder.take_build_environment_reports(),
        network_policy_rows: builder.take_network_policy_reports(),
        workspace_rows: workspace_evidence_sink.take(),
        action_result_rows: builder.take_action_result_reports(),
        overlay_report,
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
fn create_pipeline_builder(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
) -> Result<PipelineBuilderBundle<impl snix_build::buildservice::BuildService + use<>>, Error> {
    let source_policy = if config.source_fetch_overrides.is_empty() {
        FetchSourcePolicy::AllowNetwork
    } else {
        FetchSourcePolicy::RequireOverride
    };
    create_pipeline_builder_with_source_policy(config, store, source_policy)
}

#[cfg(target_os = "linux")]
fn create_pipeline_builder_with_source_policy(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
    source_policy: FetchSourcePolicy,
) -> Result<PipelineBuilderBundle<impl snix_build::buildservice::BuildService + use<>>, Error> {
    debug_assert!(!config.store_dir.is_empty(), "store prefix must not be empty");
    debug_assert!(config.max_jobs >= 1, "builder requires at least one job");

    let crunch_store::PipelineStoreParts {
        build_store,
        action_results,
        build_service_store,
        output_lookup,
        root_registry,
    } = store.into_pipeline_store_parts();
    let state_dir = build_store.state_dir().to_path_buf();
    let workdir = std::env::temp_dir().join("crunch-builds");
    std::fs::create_dir_all(&workdir).map_err(|error| Error::Internal(format!("create workdir: {error}")))?;
    let fetch_service = FetchBuildService::new(build_service_store.clone())
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
    let local_bwrap =
        build_service_store.bubblewrap_build_service(std::env::temp_dir().join("crunch-builds-local"), None);
    let remote_bwrap = build_service_store.bubblewrap_build_service(workdir, None);
    let remote_realizer = LocalBuildServiceRealizer::new(remote_bwrap, profile);
    let sandbox_service =
        RemoteFirstBuildService::new(remote_realizer, local_bwrap, RemoteBuildFallbackPolicy::OnRemoteFailure);
    let dispatch = DispatchBuildService::new(fetch_service, sandbox_service);
    let workspace_evidence_sink = empty_workspace_report_collector();
    let build_service =
        crunch_build::StatefulWorkspaceBuildService::new(dispatch, &state_dir, workspace_evidence_sink.clone());
    let mut builder = Builder::from_store_parts(
        crunch_store::BuilderStoreParts {
            build_store,
            action_results,
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
            origin_drv_key: eval_failure_key(&eval_failure.label),
            error: eval_failure.error.clone(),
            origin_error: eval_failure.error.clone(),
            build_log: None,
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
        overlay_report: evidence.overlay_report,
        store_layer_selections: evidence.store_layer_selections,
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

struct EvalStreamRequest<'a> {
    max_jobs: u32,
    store_dir: &'a str,
    root_force_policy: RootForceExecutionPolicy,
    root_file: &'a std::path::Path,
    import_paths: &'a [OsString],
    session: &'a crunch_eval::session::EvaluationSession,
    tx: mpsc::Sender<EvalMessage>,
}

async fn stream_roots_into_worker(request: EvalStreamRequest<'_>) -> Result<EvalStreamResult, Error> {
    let requested_labels =
        request.session.root_labels().iter().map(|root_label| root_label.label.clone()).collect::<Vec<_>>();
    debug_assert!(!requested_labels.is_empty(), "must have at least one root label");

    let eval_parallelism = resolve_eval_parallelism(request.max_jobs, requested_labels.len() as u32);
    let worker_input = request.session.isolated_worker_input();
    let mut join_set = JoinSet::new();
    let mut next_label_index: usize = 0;
    let mut cache = ConversionCache::new(request.store_dir);
    let mut file_resolver = derivation_file::DerivationFileResolver::new(request.root_file, request.import_paths)?;
    let mut root_drv_paths = Vec::with_capacity(requested_labels.len());
    let mut first_failure: Option<EvalFailure> = None;

    spawn_eval_workers(
        &mut join_set,
        &worker_input,
        &requested_labels,
        &mut next_label_index,
        eval_parallelism,
        request.root_force_policy,
    );
    while let Some(join_result) = join_set.join_next().await {
        let worker_result = join_result.map_err(|e| Error::Internal(format!("eval worker panicked: {e}")))?;

        match worker_result {
            Ok((label, mut drv)) => {
                if first_failure.is_some() {
                    continue;
                }
                file_resolver.resolve_root_inputs(request.root_file, &mut drv, &mut cache)?;
                let (drv_path, _nix_drv) =
                    crunch_glue::convert(&drv, &mut cache).map_err(|e| Error::Convert(format!("{label}: {e}")))?;
                let new_entries = cache.drain_pending();
                info!(drv = %drv_path, label = %label, entries = new_entries.len(), "converted, sending to worker");
                request
                    .tx
                    .send(EvalMessage {
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
                request.root_force_policy,
            );
        }
    }

    drop(request.tx);
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
        network_policy_reports: Vec::new(),
        workspace_reports: Vec::new(),
        action_result_reports: Vec::new(),
        native_dynamic_plans: Vec::new(),
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

#[allow(tigerstyle::ambiguous_params)]
pub fn parse_drv_key(store_dir: &str, drv_key: &str) -> Option<StorePath<String>> {
    StorePath::from_absolute_path_with_prefix(drv_key.as_bytes(), store_dir).ok()
}

pub fn label_for_key<'a>(result: &'a PipelineResult, drv_key: &str) -> Option<&'a str> {
    result.root_labels.get(drv_key).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

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

    async fn collect_eval_message_labels(mut rx: mpsc::Receiver<EvalMessage>) -> Vec<String> {
        let mut labels = Vec::new();
        while let Some(message) = rx.recv().await {
            labels.push(message.label);
        }
        labels
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
        let inline_result = stream_roots_into_worker(EvalStreamRequest {
            max_jobs: EVAL_POLICY_TEST_MAX_JOBS,
            store_dir: "/crunch/store",
            root_force_policy: RootForceExecutionPolicy::Inline,
            root_file: &root_file,
            import_paths: &[],
            session: &session,
            tx: inline_tx,
        })
        .await
        .unwrap();
        let inline_labels = collect_eval_message_labels(inline_rx).await;

        let (preferred_tx, preferred_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let preferred_result = stream_roots_into_worker(EvalStreamRequest {
            max_jobs: EVAL_POLICY_TEST_MAX_JOBS,
            store_dir: "/crunch/store",
            root_force_policy: RootForceExecutionPolicy::PreferThreaded,
            root_file: &root_file,
            import_paths: &[],
            session: &session,
            tx: preferred_tx,
        })
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
        let (inline_tx, inline_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let inline_result = stream_roots_into_worker(EvalStreamRequest {
            max_jobs: EVAL_POLICY_TEST_MAX_JOBS,
            store_dir: "/crunch/store",
            root_force_policy: RootForceExecutionPolicy::Inline,
            root_file: &root_file,
            import_paths: &[],
            session: &session,
            tx: inline_tx,
        })
        .await
        .unwrap();
        let inline_labels = collect_eval_message_labels(inline_rx).await;

        let (preferred_tx, preferred_rx) = mpsc::channel::<EvalMessage>(EVAL_MESSAGE_CHANNEL_CAPACITY);
        let preferred_result = stream_roots_into_worker(EvalStreamRequest {
            max_jobs: EVAL_POLICY_TEST_MAX_JOBS,
            store_dir: "/crunch/store",
            root_force_policy: RootForceExecutionPolicy::PreferThreaded,
            root_file: &root_file,
            import_paths: &[],
            session: &session,
            tx: preferred_tx,
        })
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
