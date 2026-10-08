// r[impl evaluation_streaming.signal_cancellation]
use std::ffi::OsString;
use std::future::Future;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crunch_build::signing;
use crunch_evaluation_stream_core::ProcessMode;
use crunch_evaluation_stream_core::ProcessOutcome;
use crunch_evaluation_stream_core::RunDisposition;
use crunch_evaluation_stream_core::SignalCancellationAction;
use crunch_evaluation_stream_core::process_status;
use crunch_evaluation_stream_core::signal_cancellation_action;
use crunch_pipeline::BuildConfig;
use crunch_pipeline::CausalTraceObservation;
use crunch_pipeline::HermeticityAuditEvent;
use crunch_pipeline::HermeticityMode;
use crunch_pipeline::PipelineResult;
use crunch_pipeline::drv_key_for;
use crunch_pipeline::label_for_key;
use crunch_pipeline::parse_drv_key;
use crunch_store::GcRootSource;
use mantle_application_contract::EffectKind;
use nix_compat::store_path::StorePath;

use crate::build_failure::build_failure_envelopes;
use crate::build_failure::render_human_failure_summary;
use crate::build_failure::should_write_failure_log;
use crate::build_log::DiagnosticPersistenceFailure;
use crate::build_log::log_file_path;
use crate::build_log::write_log_file;
use crate::build_report::render_build_json_report;
use crate::errors::RunError;
use crate::evaluation_stream_output::EvaluationStreamOutputError;
use crate::evaluation_stream_output::EvaluationStreamWriter;
use crate::evaluation_stream_output::STREAM_EVENT_CHANNEL_CAPACITY;
use crate::remote_build::live_state::LiveBuildEmitAction;
use crate::remote_build::live_state::LiveBuildSnapshot;
use crate::remote_build::live_state::LiveStreamFactProjector;
use crate::remote_build::live_state::LiveStreamFactTransition;
use crate::remote_build::live_state::normalize_local_worker_live_facts;
use crate::remote_build::live_state::plan_remote_live_fact_changes;
use crate::remote_build::live_state::producer::BestEffortLivePublisher;
use crate::remote_build::live_state::producer::LivePublisherObservation;
use crate::remote_build::live_state::producer::LivePublisherStatus;
use crate::signing_key::load_configured_trusted_public_keys;
use crate::signing_key::load_or_generate_signing_keypair;

const MAX_HUMAN_PRIORITY_ROWS: usize = 64;
const PRIORITY_DIGEST_PREFIX_BYTES: usize = 12;
const PRIORITY_SUMMARY_EXTRA_LINES: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorSignal {
    Interrupt,
    Terminate,
}

impl OperatorSignal {
    fn name(self) -> &'static str {
        match self {
            Self::Interrupt => "SIGINT",
            Self::Terminate => "SIGTERM",
        }
    }
}

#[cfg(unix)]
struct OperatorSignalMonitor {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl OperatorSignalMonitor {
    fn new() -> Result<Self, RunError> {
        let interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .map_err(|error| RunError::Internal(format!("registering SIGINT listener: {error}")))?;
        let terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|error| RunError::Internal(format!("registering SIGTERM listener: {error}")))?;
        Ok(Self { interrupt, terminate })
    }

    async fn receive(&mut self) -> Result<OperatorSignal, RunError> {
        tokio::select! {
            received = self.interrupt.recv() => received
                .map(|()| OperatorSignal::Interrupt)
                .ok_or_else(|| RunError::Internal("SIGINT listener closed".to_string())),
            received = self.terminate.recv() => received
                .map(|()| OperatorSignal::Terminate)
                .ok_or_else(|| RunError::Internal("SIGTERM listener closed".to_string())),
        }
    }
}

#[cfg(not(unix))]
struct OperatorSignalMonitor;

#[cfg(not(unix))]
impl OperatorSignalMonitor {
    fn new() -> Result<Self, RunError> {
        Ok(Self)
    }

    async fn receive(&mut self) -> Result<OperatorSignal, RunError> {
        tokio::signal::ctrl_c()
            .await
            .map_err(|error| RunError::Internal(format!("waiting for Ctrl-C: {error}")))?;
        Ok(OperatorSignal::Interrupt)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildOutputMode {
    Human,
    Json,
    EvaluationStream,
}

impl BuildOutputMode {
    fn is_human(self) -> bool {
        matches!(self, Self::Human)
    }

    fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }

    fn is_evaluation_stream(self) -> bool {
        matches!(self, Self::EvaluationStream)
    }
}

// Stable CLI shell: dispatch supplies independently typed build policy and path fields.
#[allow(
    clippy::too_many_arguments,
    tigerstyle::ambiguous_params,
    tigerstyle::too_many_parameters,
    reason = "stable positional CLI adapter delegates immediately to the named BuildConfig boundary"
)]
pub fn cmd_build(
    file: &Path,
    import_paths: &[OsString],
    output_dir: &Path,
    state_dir: &Path,
    logs_dir: &Path,
    store_dir: &str,
    backend: crunch_store::StoreBackend,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_urls: &[String],
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: HermeticityMode,
    output_mode: BuildOutputMode,
    interchange_dir: Option<&Path>,
    causal_trace: bool,
) -> Result<(), RunError> {
    cmd_build_with_source_fetch_overrides(
        file,
        import_paths,
        output_dir,
        state_dir,
        logs_dir,
        store_dir,
        backend,
        verbose,
        fix,
        max_jobs,
        substituter_urls,
        signing_key_path,
        trusted_public_keys,
        trust_unsigned,
        hermeticity_mode,
        output_mode,
        interchange_dir,
        causal_trace,
        Vec::new(),
        Vec::new(),
        None,
    )
}

// Stable CLI shell: source-fetch overrides extend the same compatibility boundary.
#[allow(
    clippy::too_many_arguments,
    tigerstyle::ambiguous_params,
    tigerstyle::too_many_parameters,
    reason = "stable positional CLI adapter delegates immediately to the named BuildConfig boundary"
)]
pub fn cmd_build_with_source_fetch_overrides(
    file: &Path,
    import_paths: &[OsString],
    output_dir: &Path,
    state_dir: &Path,
    logs_dir: &Path,
    store_dir: &str,
    backend: crunch_store::StoreBackend,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_urls: &[String],
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: HermeticityMode,
    output_mode: BuildOutputMode,
    interchange_dir: Option<&Path>,
    causal_trace: bool,
    source_fetch_overrides: Vec<crunch_build::FetchSourceOverride>,
    base_state_dirs: Vec<PathBuf>,
    root_registration: Option<crunch_store::RootRegistration>,
) -> Result<(), RunError> {
    let config = prepare_local_build_config(
        LocalBuildCommandConfig {
            file,
            import_paths,
            output_dir,
            state_dir,
            store_dir,
            backend,
            verbose,
            max_jobs,
            substituter_urls,
            signing_key_path,
            trusted_public_keys,
            trust_unsigned,
            hermeticity_mode,
            base_state_dirs,
        },
        output_mode,
        interchange_dir,
        source_fetch_overrides,
        root_registration,
    )?;
    if output_mode.is_evaluation_stream() {
        let completion = run_build_with_evaluation_stream(&config)?;
        report_build_result(&config, &completion.result, fix, output_mode, logs_dir)?;
        return evaluation_stream_process_result(&completion);
    }
    if causal_trace {
        let (result, observation) = run_build_with_causal_trace(&config)?;
        match observation {
            CausalTraceObservation::Recorded(trace) => {
                crate::causal_trace_diagnostic::write_trace(state_dir, &trace).map_err(RunError::Internal)?;
            }
            CausalTraceObservation::WorkerNotStarted => {
                eprintln!("causal trace unavailable: worker not started (store preflight failed)");
            }
        }
        return report_build_result(&config, &result, fix, output_mode, logs_dir);
    }
    let result = run_build(&config)?;
    report_build_result(&config, &result, fix, output_mode, logs_dir)
}

/// The common local file build configuration. Watch and one-shot use the
/// same signing, scheduling, source, and store policy.
pub struct LocalBuildCommandConfig<'a> {
    pub file: &'a Path,
    pub import_paths: &'a [OsString],
    pub output_dir: &'a Path,
    pub state_dir: &'a Path,
    pub store_dir: &'a str,
    pub backend: crunch_store::StoreBackend,
    pub verbose: bool,
    pub max_jobs: u32,
    pub substituter_urls: &'a [String],
    pub signing_key_path: Option<&'a Path>,
    pub trusted_public_keys: Option<&'a [nix_compat::narinfo::VerifyingKey]>,
    pub trust_unsigned: bool,
    pub hermeticity_mode: HermeticityMode,
    pub base_state_dirs: Vec<PathBuf>,
}

fn prepare_local_build_config(
    input: LocalBuildCommandConfig<'_>,
    output_mode: BuildOutputMode,
    interchange_dir: Option<&Path>,
    source_fetch_overrides: Vec<crunch_build::FetchSourceOverride>,
    root_registration: Option<crunch_store::RootRegistration>,
) -> Result<BuildConfig, RunError> {
    let LocalBuildCommandConfig {
        file,
        import_paths,
        output_dir,
        state_dir,
        store_dir,
        backend,
        verbose,
        max_jobs,
        substituter_urls,
        signing_key_path,
        trusted_public_keys,
        trust_unsigned,
        hermeticity_mode,
        base_state_dirs,
    } = input;
    crunch_store::StoreConfig::preflight_backend_identity_for(backend, state_dir, store_dir, &base_state_dirs)
        .map_err(|error| RunError::Internal(format!("opening store: {error}")))?;
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, output_mode.is_human())?;
    let configured_trusted_keys = load_configured_trusted_public_keys(trusted_public_keys, state_dir)?;
    let trusted_keys = signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());

    let config = BuildConfig {
        file: file.to_path_buf(),
        import_paths: import_paths.to_vec(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        backend,
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        scheduling_policy: crunch_pipeline::SchedulingPolicy {
            schema: crunch_build::scheduling::SCHEDULING_POLICY_SCHEMA.to_string(),
            policy_id: crunch_build::scheduling::DEFAULT_SCHEDULING_POLICY_ID.to_string(),
            preference_order: vec![
                crunch_build::PreferenceField::KnownGraph,
                crunch_build::PreferenceField::ResourceFit,
                crunch_build::PreferenceField::LocalityTransfer,
            ],
            aged_after_epochs: crunch_build::scheduling::DEFAULT_AGED_AFTER_EPOCHS,
            protected_after_epochs: crunch_build::scheduling::DEFAULT_PROTECTED_AFTER_EPOCHS,
        },
        substituter_urls: substituter_urls.to_vec(),
        hermeticity_mode,
        keypair,
        trusted_keys,
        trust_unsigned,
        root_retention_source: Some(GcRootSource::Build),
        root_registration,
        source_fetch_overrides,
        remote_enabled: false,
        interchange_dir: interchange_dir.map(Path::to_path_buf),
        base_state_dirs,
    };

    debug_assert_eq!(config.file.as_path(), file);
    debug_assert_eq!(config.hermeticity_mode, hermeticity_mode);
    Ok(config)
}

fn print_watch_notice(config: &BuildConfig, notice: crunch_pipeline::WatchNotice) -> std::io::Result<()> {
    use std::io::Write;

    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    match notice {
        crunch_pipeline::WatchNotice::Goals { generation, diff } => {
            writeln!(output, "watch generation {generation}: {} admitted goal event(s)", diff.events.len())?;
            for event in diff.events.iter().take(32) {
                writeln!(output, "  {} {}", event.kind.as_str(), event.goal_identity.as_deref().unwrap_or(""))?;
            }
            if diff.events.len() > 32 {
                writeln!(output, "  ... {} more goal event(s)", diff.events.len() - 32)?;
            }
        }
        crunch_pipeline::WatchNotice::Rejected { diff } => {
            let diagnostic = diff
                .events
                .iter()
                .find_map(|event| event.diagnostic.as_deref())
                .unwrap_or("watched source edit rejected");
            writeln!(output, "watch edit rejected: {diagnostic}; previous goals remain admitted")?;
        }
        crunch_pipeline::WatchNotice::PendingCancellation { generation, stale } => {
            writeln!(
                output,
                "watch generation {generation}: cancellation pending for {} goal(s); waiting for teardown",
                stale.len()
            )?;
        }
        crunch_pipeline::WatchNotice::Settled {
            generation,
            outcomes,
            failed,
        } => {
            writeln!(
                output,
                "watch generation {generation}: settled {} successful, {} failed root(s)",
                outcomes.len(),
                failed.len()
            )?;
            let output_dir = config.output_dir.to_str().unwrap_or(&config.store_dir);
            for outcome in &outcomes {
                let is_multi = outcome.outputs.len() > 1;
                for (name, path_info) in &outcome.outputs {
                    let path = path_info.store_path.to_absolute_path_with_prefix(output_dir);
                    writeln!(output, "{path}{}", format_output_suffix(outcome, name, is_multi))?;
                }
            }
            for failure in &failed {
                writeln!(output, "  failed {}: {}", failure.drv_key, failure.error)?;
            }
        }
    }
    output.flush()
}

/// Run the persistent local Worker until a signal has requested and completed
/// owned teardown. A second signal never drops a still-running sandbox future.
pub fn cmd_build_watch(input: LocalBuildCommandConfig<'_>) -> Result<(), RunError> {
    let config = prepare_local_build_config(input, BuildOutputMode::Human, None, Vec::new(), None)?;
    let started = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| RunError::Internal(format!("watch run clock: {error}")))?;
    let run_identity = format!("watch-{}-{}", std::process::id(), started.as_nanos());
    let runtime =
        tokio::runtime::Runtime::new().map_err(|error| RunError::Internal(format!("watch runtime: {error}")))?;
    runtime.block_on(async {
        let mut signals = OperatorSignalMonitor::new()?;
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let (notice_tx, mut notice_rx) = tokio::sync::mpsc::channel(16);
        let watch = crunch_pipeline::watch_build(&config, run_identity, &mut shutdown_rx, &notice_tx);
        tokio::pin!(watch);
        let mut requested_shutdown = false;
        let mut receiving_notices = true;
        let mut output_error = None;
        let result = loop {
            tokio::select! {
                biased;
                signal = signals.receive(), if !requested_shutdown => {
                    requested_shutdown = true;
                    match signal {
                        Ok(signal) => eprintln!("received {}; waiting for watched builds to stop", signal.name()),
                        Err(error) => output_error = Some(error),
                    }
                    let _ = shutdown_tx.send(true);
                }
                notice = notice_rx.recv(), if receiving_notices => {
                    match notice {
                        Some(notice) if output_error.is_none() => {
                            if let Err(error) = print_watch_notice(&config, notice) {
                                output_error = Some(RunError::Internal(format!("writing watch notice: {error}")));
                                requested_shutdown = true;
                                let _ = shutdown_tx.send(true);
                            }
                        }
                        Some(_) => {}
                        None => {
                            receiving_notices = false;
                            output_error = Some(RunError::Internal("watch notice stream closed".to_string()));
                            requested_shutdown = true;
                            let _ = shutdown_tx.send(true);
                        }
                    }
                }
                finished = &mut watch => break finished,
            }
        };
        if output_error.is_none() {
            while let Ok(notice) = notice_rx.try_recv() {
                if let Err(error) = print_watch_notice(&config, notice) {
                    output_error = Some(RunError::Internal(format!("writing watch notice: {error}")));
                    break;
                }
            }
        }
        if let Some(error) = output_error {
            return Err(error);
        }
        result.map_err(Into::into)
    })
}

pub fn run_build(config: &BuildConfig) -> Result<PipelineResult, RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    if let Some(socket_path) = configured_live_socket() {
        return rt.block_on(run_build_with_live_observation(config, &socket_path, false)).map(|(result, _)| result);
    }
    rt.block_on(crunch_pipeline::build(config)).map_err(Into::into)
}

fn run_build_with_causal_trace(config: &BuildConfig) -> Result<(PipelineResult, CausalTraceObservation), RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|error| RunError::Internal(format!("tokio runtime: {error}")))?;
    if let Some(socket_path) = configured_live_socket() {
        let (result, observation) = rt.block_on(run_build_with_live_observation(config, &socket_path, true))?;
        let observation = observation.ok_or_else(|| RunError::Internal("causal trace was not captured".to_string()))?;
        return Ok((result, observation));
    }
    rt.block_on(crunch_pipeline::build_with_causal_trace(config)).map_err(Into::into)
}

pub(crate) fn configured_live_socket() -> Option<PathBuf> {
    std::env::var_os("MANTLE_LIVE_STATE_SOCKET").filter(|value| !value.is_empty()).map(PathBuf::from)
}

/// An evaluation run id is content-addressed; daemon ownership must instead
/// distinguish concurrent invocations of the same source and root selection.
fn next_live_owner_id(stream_run_id: &str) -> Option<String> {
    static NEXT_INVOCATION: AtomicU64 = AtomicU64::new(0);
    let invocation = NEXT_INVOCATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| current.checked_add(1))
        .ok()?;
    Some(format!("live-{stream_run_id}-{}-{invocation}", std::process::id()))
}

struct LiveStreamObserver {
    projection: LiveStreamFactProjector,
    worker_snapshot: LiveBuildSnapshot,
    publisher: BestEffortLivePublisher,
    degraded_reported: bool,
}

fn observe_live_stream_record(
    observer: &mut Option<LiveStreamObserver>,
    socket_path: Option<&Path>,
    record: &crunch_evaluation_stream_core::StreamRecordValue,
) {
    if observer.is_none() {
        let (Some(socket_path), crunch_evaluation_stream_core::StreamRecordValue::RunStart(start)) =
            (socket_path, record)
        else {
            return;
        };
        let run_id = start.run_id();
        let Some(owner_run_id) = next_live_owner_id(&run_id) else {
            eprintln!("live build observation degraded: owner invocation exhausted");
            return;
        };
        let Ok(projection) = LiveStreamFactProjector::new(&owner_run_id, &run_id) else {
            eprintln!("live build observation degraded: invalid run identity");
            return;
        };
        let Ok(publisher) = BestEffortLivePublisher::new_empty(socket_path, owner_run_id.as_str()) else {
            eprintln!("live build observation degraded: cannot admit owner");
            return;
        };
        *observer = Some(LiveStreamObserver {
            projection,
            publisher,
            worker_snapshot: LiveBuildSnapshot::empty(owner_run_id).expect("admitted live owner"),
            degraded_reported: false,
        });
    }
    let Some(active) = observer.as_mut() else { return };
    let transition = match active.projection.observe(record) {
        Ok(transition) => transition,
        Err(error) => {
            eprintln!("live build observation degraded: {error:?}");
            *observer = None;
            return;
        }
    };
    let emit = match transition {
        LiveStreamFactTransition::Unchanged => return,
        LiveStreamFactTransition::Discovered { fact_id, fact } => {
            active.publisher.enqueue(LiveBuildEmitAction::Publish {
                fact_id: &fact_id,
                fact: &fact,
            })
        }
        LiveStreamFactTransition::Terminal { retract, fact_id, fact } => {
            if let Some(retracted_id) = retract {
                let _ = active.publisher.enqueue(LiveBuildEmitAction::Retract { fact_id: &retracted_id });
            }
            active.publisher.enqueue(LiveBuildEmitAction::Publish {
                fact_id: &fact_id,
                fact: &fact,
            })
        }
    };
    if !active.degraded_reported {
        match emit {
            Ok(LivePublisherObservation::Degraded(reason)) => {
                eprintln!("live build observation degraded: {reason:?}");
                active.degraded_reported = true;
            }
            Err(error) => {
                eprintln!("live build observation degraded: {error:?}");
                active.degraded_reported = true;
            }
            _ => {}
        }
    }
}

fn observe_live_worker_snapshot(
    observer: &mut Option<LiveStreamObserver>,
    snapshot: &crunch_pipeline::WorkerLiveSnapshot,
) {
    let Some(active) = observer.as_mut() else { return };
    let next = match normalize_local_worker_live_facts(active.worker_snapshot.owner_run_id(), snapshot) {
        Ok(next) => next,
        Err(error) => {
            if !active.degraded_reported {
                eprintln!("live build observation degraded: worker snapshot {error:?}");
                active.degraded_reported = true;
            }
            return;
        }
    };
    let actions = plan_remote_live_fact_changes(&active.worker_snapshot, &next).expect("worker snapshot retains owner");
    for action in actions {
        let observation = active.publisher.enqueue(action);
        if !active.degraded_reported {
            match observation {
                Ok(LivePublisherObservation::Degraded(reason)) => {
                    eprintln!("live build observation degraded: {reason:?}");
                    active.degraded_reported = true;
                }
                Err(error) => {
                    eprintln!("live build observation degraded: {error:?}");
                    active.degraded_reported = true;
                }
                _ => {}
            }
        }
    }
    active.worker_snapshot = next;
}

fn report_live_worker_degradation(observer: &mut Option<LiveStreamObserver>, degraded: &AtomicBool) {
    if degraded.load(Ordering::Acquire) && !observer.as_ref().is_some_and(|active| active.degraded_reported) {
        eprintln!("live build observation degraded: worker observation queue full");
        if let Some(active) = observer.as_mut() {
            active.degraded_reported = true;
        }
    }
}

fn report_live_publisher_degradation(observer: &mut Option<LiveStreamObserver>) {
    let Some(active) = observer.as_mut() else { return };
    if active.degraded_reported {
        return;
    }
    if let LivePublisherStatus::Degraded(reason) = active.publisher.status() {
        eprintln!("live build observation degraded: {reason:?}");
        active.degraded_reported = true;
    }
}

async fn run_build_with_live_observation(
    config: &BuildConfig,
    socket_path: &Path,
    causal_trace: bool,
) -> Result<(PipelineResult, Option<CausalTraceObservation>), RunError> {
    let (stream_tx, mut stream_rx) = tokio::sync::mpsc::channel(STREAM_EVENT_CHANNEL_CAPACITY);
    let (worker_tx, mut worker_rx) = tokio::sync::mpsc::channel(16);
    let worker_degraded = Arc::new(AtomicBool::new(false));
    let cancellation = crunch_pipeline::EvaluationCancellation::new();
    let worker_observation = crunch_pipeline::WorkerLiveObservation::new(worker_tx, Arc::clone(&worker_degraded));
    let build_future = async {
        if causal_trace {
            crunch_pipeline::build_with_causal_trace_live(config, cancellation, stream_tx, worker_observation)
                .await
                .map(|(result, trace)| (result, Some(trace)))
        } else {
            crunch_pipeline::build_with_live_observation(config, cancellation, stream_tx, worker_observation)
                .await
                .map(|result| (result, None))
        }
    };
    tokio::pin!(build_future);
    let mut result = None;
    let mut stream_open = true;
    let mut worker_open = true;
    let mut observer = None;
    while stream_open || worker_open || result.is_none() {
        tokio::select! {
            biased;
            record = stream_rx.recv(), if stream_open => match record {
                Some(record) => observe_live_stream_record(&mut observer, Some(socket_path), &record),
                None => stream_open = false,
            },
            snapshot = worker_rx.recv(), if worker_open => match snapshot {
                Some(snapshot) => observe_live_worker_snapshot(&mut observer, &snapshot),
                None => worker_open = false,
            },
            completed = &mut build_future, if result.is_none() => result = Some(completed),
        }
    }
    report_live_worker_degradation(&mut observer, &worker_degraded);
    report_live_publisher_degradation(&mut observer);
    result.expect("build future always completed").map_err(Into::into)
}

struct EvaluationStreamBuildCompletion {
    result: PipelineResult,
    disposition: RunDisposition,
}

fn run_build_with_evaluation_stream(config: &BuildConfig) -> Result<EvaluationStreamBuildCompletion, RunError> {
    let runtime =
        tokio::runtime::Runtime::new().map_err(|error| RunError::Internal(format!("tokio runtime: {error}")))?;
    let stdout = std::io::stdout();
    let locked = stdout.lock();
    runtime.block_on(run_build_with_stream_output(config, locked))
}

async fn run_build_with_stream_output<W: std::io::Write>(
    config: &BuildConfig,
    output: W,
) -> Result<EvaluationStreamBuildCompletion, RunError> {
    let mut signal_monitor = OperatorSignalMonitor::new()?;
    let (stream_tx, mut stream_rx) = tokio::sync::mpsc::channel(STREAM_EVENT_CHANNEL_CAPACITY);
    let cancellation = crunch_pipeline::EvaluationCancellation::new();
    let socket_path = configured_live_socket();
    let mut worker_rx = None;
    let worker_degraded = socket_path.as_ref().map(|_| Arc::new(AtomicBool::new(false)));
    let worker_observation = worker_degraded.as_ref().map(|degraded| {
        let (worker_tx, receiver) = tokio::sync::mpsc::channel(16);
        worker_rx = Some(receiver);
        crunch_pipeline::WorkerLiveObservation::new(worker_tx, Arc::clone(degraded))
    });
    let build_future = async {
        match worker_observation {
            Some(observation) => {
                crunch_pipeline::build_with_live_observation(config, cancellation.clone(), stream_tx, observation).await
            }
            None => crunch_pipeline::build_with_evaluation_stream(config, cancellation.clone(), stream_tx).await,
        }
    };
    let mut writer = EvaluationStreamWriter::new(output);
    let mut observer = None;
    let build_result = drive_stream_with_signals(
        &mut writer,
        &mut stream_rx,
        LiveStreamDrive {
            worker_rx: worker_rx.as_mut(),
            observer: &mut observer,
            socket_path: socket_path.as_deref(),
        },
        &cancellation,
        &mut signal_monitor,
        build_future,
    )
    .await?;
    if let Some(degraded) = &worker_degraded {
        report_live_worker_degradation(&mut observer, degraded);
    }
    report_live_publisher_degradation(&mut observer);
    let disposition = writer.finish().map_err(stream_output_failure)?;
    let result = build_result.map_err(RunError::from)?;
    Ok(EvaluationStreamBuildCompletion { result, disposition })
}

struct LiveStreamDrive<'a> {
    worker_rx: Option<&'a mut tokio::sync::mpsc::Receiver<Arc<crunch_pipeline::WorkerLiveSnapshot>>>,
    observer: &'a mut Option<LiveStreamObserver>,
    socket_path: Option<&'a Path>,
}

async fn drive_stream_with_signals<W, F>(
    writer: &mut EvaluationStreamWriter<W>,
    stream_rx: &mut tokio::sync::mpsc::Receiver<crunch_evaluation_stream_core::StreamRecordValue>,
    live: LiveStreamDrive<'_>,
    cancellation: &crunch_pipeline::EvaluationCancellation,
    signal_monitor: &mut OperatorSignalMonitor,
    build_future: F,
) -> Result<Result<PipelineResult, crunch_pipeline::Error>, RunError>
where
    W: std::io::Write,
    F: Future<Output = Result<PipelineResult, crunch_pipeline::Error>>,
{
    tokio::pin!(build_future);
    let mut build_result = None;
    let mut is_stream_open = true;
    let mut observed_signal_count = 0_u32;
    let LiveStreamDrive {
        mut worker_rx,
        observer,
        socket_path,
    } = live;
    let mut is_worker_open = worker_rx.is_some();
    while is_stream_open || is_worker_open || build_result.is_none() {
        tokio::select! {
            biased;
            signal = signal_monitor.receive() => {
                match signal {
                    Ok(signal) => handle_operator_signal(signal, &mut observed_signal_count, cancellation)?,
                    Err(error) => {
                        cancellation.request();
                        return Err(error);
                    }
                }
            }
            record = stream_rx.recv(), if is_stream_open => {
                match record {
                    Some(record) => {
                        write_stream_record(writer, &record, cancellation)?;
                        observe_live_stream_record(observer, socket_path, &record);
                    }
                    None => is_stream_open = false,
                }
            }
            snapshot = async {
                match worker_rx.as_mut() {
                    Some(receiver) => receiver.recv().await,
                    None => None,
                }
            }, if is_worker_open => {
                match snapshot {
                    Some(snapshot) => observe_live_worker_snapshot(observer, &snapshot),
                    None => is_worker_open = false,
                }
            }
            result = &mut build_future, if build_result.is_none() => {
                build_result = Some(result);
            }
        }
    }
    build_result.ok_or_else(|| stream_output_failure(EvaluationStreamOutputError::MissingSummary))
}

fn handle_operator_signal(
    signal: OperatorSignal,
    observed_signal_count: &mut u32,
    cancellation: &crunch_pipeline::EvaluationCancellation,
) -> Result<(), RunError> {
    *observed_signal_count = observed_signal_count.saturating_add(1);
    let action = signal_cancellation_action(*observed_signal_count)
        .ok_or_else(|| RunError::Internal("signal policy rejected an observed signal".to_string()))?;
    match action {
        SignalCancellationAction::RequestCancellation => {
            eprintln!("received {}; requesting evaluation stream cancellation", signal.name());
            cancellation.request();
            debug_assert!(cancellation.is_requested());
            Ok(())
        }
        SignalCancellationAction::ForceInterruption => {
            eprintln!("received repeated {}; forcing evaluation stream interruption", signal.name());
            cancellation.request();
            Err(RunError::Reported(crunch_evaluation_stream_core::CANCELLED_EXIT_CODE))
        }
    }
}

fn write_stream_record<W: std::io::Write>(
    writer: &mut EvaluationStreamWriter<W>,
    record: &crunch_evaluation_stream_core::StreamRecordValue,
    cancellation: &crunch_pipeline::EvaluationCancellation,
) -> Result<(), RunError> {
    if let Err(error) = writer.write_record(record) {
        cancellation.request();
        return Err(stream_output_failure(error));
    }
    Ok(())
}

fn stream_output_failure(error: EvaluationStreamOutputError) -> RunError {
    eprintln!("error: evaluation stream output failed: {error}");
    RunError::Reported(crunch_evaluation_stream_core::INTERNAL_EXIT_CODE)
}

fn evaluation_stream_process_result(completion: &EvaluationStreamBuildCompletion) -> Result<(), RunError> {
    let stream_status = process_status(ProcessOutcome::Completed(completion.disposition), ProcessMode::Pipeline);
    let status =
        if stream_status == crunch_evaluation_stream_core::SUCCESS_EXIT_CODE && !completion.result.failed.is_empty() {
            crunch_evaluation_stream_core::PIPELINE_NON_SUCCESS_EXIT_CODE
        } else {
            stream_status
        };
    if status == crunch_evaluation_stream_core::SUCCESS_EXIT_CODE {
        return Ok(());
    }
    Err(RunError::Reported(status))
}

pub fn report_build_result(
    config: &BuildConfig,
    result: &PipelineResult,
    fix: bool,
    output_mode: BuildOutputMode,
    logs_dir: &Path,
) -> Result<(), RunError> {
    debug_assert!(!(output_mode.is_json() && output_mode.is_human()));
    debug_assert!(output_mode.is_human() || output_mode.is_json() || output_mode.is_evaluation_stream());
    debug_assert!(!config.store_dir.is_empty());
    let mut diagnostic_persistence_failures = Vec::new();
    let is_log_dir_ready = match prepare_logs_dir(logs_dir) {
        Ok(()) => true,
        Err(failure) => {
            diagnostic_persistence_failures.push(failure);
            false
        }
    };

    if is_log_dir_ready {
        diagnostic_persistence_failures.extend(write_success_logs(config, result, logs_dir, output_mode));
    }
    if output_mode.is_human() {
        print_hermeticity_summary(result);
        print_action_result_summary(result);
        if config.verbose {
            print_priority_summary(result);
        }
        print_success_outputs(config, result);
    }

    if result.failed.is_empty() {
        if output_mode.is_human() || output_mode.is_evaluation_stream() {
            print_diagnostic_persistence_failures(&diagnostic_persistence_failures);
        }
        if output_mode.is_json() {
            print_json_report(config, result, logs_dir, &diagnostic_persistence_failures)?;
        }
        emit_interchange(config, result, logs_dir)?;
        return Ok(());
    }

    if is_log_dir_ready {
        diagnostic_persistence_failures.extend(write_failure_logs(config, result, logs_dir));
    }
    if output_mode.is_json() {
        print_json_report(config, result, logs_dir, &diagnostic_persistence_failures)?;
    }
    emit_interchange(config, result, logs_dir)?;
    if output_mode.is_evaluation_stream() {
        print_diagnostic_persistence_failures(&diagnostic_persistence_failures);
        return Ok(());
    }

    if let Some(single_mismatch) = maybe_single_fod_mismatch(config, result, fix, output_mode) {
        return single_mismatch;
    }

    if output_mode.is_human() {
        print_failed_builds(result, &config.store_dir, logs_dir);
        print_diagnostic_persistence_failures(&diagnostic_persistence_failures);
    }

    Err(RunError::Reported(1))
}

fn maybe_single_fod_mismatch(
    config: &BuildConfig,
    result: &PipelineResult,
    fix: bool,
    output_mode: BuildOutputMode,
) -> Option<Result<(), RunError>> {
    if result.failed.len() != 1 {
        return None;
    }
    if result.fod_mismatches.len() != 1 {
        return None;
    }

    debug_assert_eq!(result.failed.len(), 1);
    debug_assert_eq!(result.fod_mismatches.len(), 1);
    let failed = &result.failed[0];
    let mismatch = &result.fod_mismatches[0];
    let drv_path = parse_drv_key(&config.store_dir, &failed.drv_key)?;
    let label = label_for_key(result, &failed.drv_key).unwrap_or(drv_path.name());

    Some(crate::fix::handle_fod_mismatch(
        mismatch,
        &drv_path,
        label,
        &config.file,
        fix,
        output_mode.is_human(),
    ))
}

fn write_success_logs(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    output_mode: BuildOutputMode,
) -> Vec<DiagnosticPersistenceFailure> {
    let mut failures = Vec::with_capacity(result.outcomes.len());
    for outcome in &result.outcomes {
        let drv_key = drv_key_for(&config.store_dir, &outcome.drv_path);
        let label = label_for_key(result, &drv_key).unwrap_or(outcome.drv_path.name());

        if let Some(log) = &outcome.log {
            if let Err(failure) = write_log(logs_dir, &outcome.drv_path, label, true, log) {
                failures.push(failure);
            }
            if config.verbose && output_mode.is_human() {
                eprintln!("--- build log: {label} ---");
                eprintln!("{log}");
                eprintln!("--- end log ---");
            }
        } else if !outcome.cached
            && let Err(failure) = write_log(logs_dir, &outcome.drv_path, label, true, "(no output captured)")
        {
            failures.push(failure);
        }
    }
    debug_assert!(failures.len() <= result.outcomes.len());
    debug_assert!(!(output_mode.is_json() && output_mode.is_human()));
    failures
}

fn print_success_outputs(config: &BuildConfig, result: &PipelineResult) {
    let output_dir_str = config.output_dir.to_str().unwrap_or(&config.store_dir);

    for outcome in &result.outcomes {
        let is_multi = outcome.outputs.len() > 1;
        let mut outputs: Vec<_> = outcome.outputs.iter().collect();
        outputs.sort_by(|left, right| left.0.cmp(right.0));
        for (output_name, path_info) in outputs {
            let path = path_info.store_path.to_absolute_path_with_prefix(output_dir_str);
            let suffix = format_output_suffix(outcome, output_name, is_multi);
            println!("{path}{suffix}");
        }
    }
}

fn format_output_suffix(outcome: &crunch_build::BuildOutcome, output_name: &str, is_multi: bool) -> String {
    let mut parts = Vec::<String>::new();
    if is_multi && output_name != "out" {
        parts.push(output_name.to_string());
    }
    if outcome.cached {
        parts.push("cached".to_string());
    }
    if let Some(report) = outcome.substitutions.get(output_name) {
        parts.push(format!(
            "substitution={}, transferred_bytes={}, reused_bytes={}",
            report.mode.as_str(),
            report.transferred_bytes,
            report.reused_bytes
        ));
        if let Some(reason) = &report.fallback_reason {
            parts.push(format!("fallback_reason={reason}"));
        }
    }
    debug_assert!(!output_name.is_empty());
    debug_assert!(parts.len() <= outcome.substitutions.len().saturating_add(3));
    if parts.is_empty() {
        return String::new();
    }
    format!(" ({})", parts.join(", "))
}

fn print_json_report(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    diagnostic_persistence_failures: &[DiagnosticPersistenceFailure],
) -> Result<(), RunError> {
    let serialized_build_document = render_build_json_report(config, result, logs_dir, diagnostic_persistence_failures)
        .map_err(|e| RunError::Internal(format!("serializing build report: {e}")))?;
    println!("{serialized_build_document}");
    Ok(())
}

/// Emit the owner interchange records when a directory was configured.
/// Emission establishes no execution authority, custody, or promotion.
fn emit_interchange(config: &BuildConfig, result: &PipelineResult, logs_dir: &Path) -> Result<(), RunError> {
    let Some(directory) = crate::build_interchange::configured_directory(config) else {
        return Ok(());
    };
    crate::build_interchange::emit_from_report(config, result, logs_dir, &directory)
        .map(|_paths| ())
        .map_err(|error| RunError::Internal(format!("build interchange: {error}")))
}

fn print_hermeticity_summary(result: &PipelineResult) {
    for line in format_hermeticity_summary(result.hermeticity_mode, &result.hermeticity_audit_events) {
        eprintln!("{line}");
    }
}

fn print_priority_summary(result: &PipelineResult) {
    for line in format_priority_summary(&result.priority_decisions) {
        eprintln!("{line}");
    }
}

fn print_action_result_summary(result: &PipelineResult) {
    for line in format_action_result_summary(&result.action_result_reports) {
        eprintln!("{line}");
    }
}

fn format_action_result_summary(reports: &[crunch_build::ActionResultRuntimeReport]) -> Vec<String> {
    let rejected_count_max =
        reports.iter().map(|report| report.candidate_decisions.len()).fold(0_usize, usize::saturating_add);
    let line_count_max = reports.len().saturating_add(rejected_count_max);
    let mut lines = Vec::with_capacity(line_count_max);
    for report in reports {
        let selected = report.selected_result_ref.as_deref().unwrap_or("none");
        let source = report.selected_source_class.as_deref().unwrap_or("none");
        let source_id = report.selected_source_id.as_deref().unwrap_or("none");
        let conflict = report.conflict_class.as_deref().unwrap_or("none");
        let trust_basis = if report.trust_basis.is_empty() {
            "none".to_string()
        } else {
            report.trust_basis.join(",")
        };
        lines.push(format!(
            "shared-action-result: phase={} disposition={} action={} selected={} source={} source_id={} trust={} conflict={} non_claims={}",
            report.phase,
            report.disposition,
            report.action_ref,
            selected,
            source,
            source_id,
            trust_basis,
            conflict,
            report.non_claims.join(",")
        ));
        for decision in &report.candidate_decisions {
            if decision.admitted {
                continue;
            }
            lines.push(format!(
                "shared-action-result-rejected: result={} source={} source_id={} diagnostics={}",
                decision.result_ref,
                decision.source_class,
                decision.source_id,
                decision.diagnostics.join(",")
            ));
        }
    }
    debug_assert!(lines.len() >= reports.len());
    debug_assert!(lines.capacity() >= lines.len());
    lines
}

fn format_priority_summary(decisions: &[crunch_pipeline::PriorityDecisionEvidence]) -> Vec<String> {
    if decisions.is_empty() {
        return Vec::new();
    }
    let rendered_count = decisions.len().min(MAX_HUMAN_PRIORITY_ROWS);
    let mut lines = Vec::with_capacity(rendered_count.saturating_add(PRIORITY_SUMMARY_EXTRA_LINES));
    lines.push(format!(
        "scheduler priority: decisions={} scope={} non-claims={}",
        decisions.len(),
        decisions[0].claim_scope,
        decisions[0].non_claims.join(",")
    ));
    for decision in decisions.iter().take(rendered_count) {
        let selected = decision
            .selected_goal_key_blake3
            .get(..PRIORITY_DIGEST_PREFIX_BYTES)
            .unwrap_or(&decision.selected_goal_key_blake3);
        let snapshot = decision
            .candidate_snapshot_digest_blake3
            .get(..PRIORITY_DIGEST_PREFIX_BYTES)
            .unwrap_or(&decision.candidate_snapshot_digest_blake3);
        lines.push(format!(
            "  - epoch={} selected={} snapshot={} policy={} reason={} operator={} age={}/{} graph={}:path={},work={},roots={} resource={} locality={} transfer={} history={}",
            decision.scheduling_epoch,
            selected,
            snapshot,
            decision.policy_id,
            decision.selection_reason.as_str(),
            decision.operator_policy_class.as_str(),
            decision.starvation_class.as_str(),
            decision.age_epochs,
            decision.known_graph_basis,
            decision.known_critical_path_nodes,
            decision.known_critical_path_work_units,
            decision.blocked_root_count,
            decision.resource_fit_class.as_str(),
            decision.content_locality_class.as_str(),
            decision.transfer_cost_class.as_str(),
            decision.history_basis.as_str(),
        ));
    }
    if rendered_count < decisions.len() {
        lines.push(format!(
            "  - omitted={} (machine report retains all bounded decisions)",
            decisions.len().saturating_sub(rendered_count)
        ));
    }
    debug_assert!(lines.len() <= MAX_HUMAN_PRIORITY_ROWS.saturating_add(PRIORITY_SUMMARY_EXTRA_LINES));
    debug_assert!(lines.iter().all(|line| !line.contains("token=")));
    lines
}

fn format_hermeticity_summary(mode: HermeticityMode, events: &[HermeticityAuditEvent]) -> Vec<String> {
    let mut lines = Vec::with_capacity(events.len().saturating_add(1));
    if events.is_empty() {
        lines.push(format!("hermeticity: {mode} (no degraded facts)"));
        return lines;
    }

    lines.push(format!(
        "WARNING: degraded hermeticity: {mode} ({} {})",
        events.len(),
        audit_event_label(events.len())
    ));
    for event in events {
        lines.push(format!("  - {}: {}", event.kind, event.detail));
    }
    lines
}

fn audit_event_label(event_count: usize) -> &'static str {
    if event_count == 1 {
        return "audit event";
    }
    "audit events"
}

fn write_failure_logs(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
) -> Vec<DiagnosticPersistenceFailure> {
    let mut failures = Vec::with_capacity(result.failed.len());
    for failed in &result.failed {
        if !should_write_failure_log(&failed.error) {
            continue;
        }
        let Some(drv_path) = parse_drv_key(&config.store_dir, &failed.drv_key) else {
            continue;
        };
        let label = label_for_key(result, &failed.drv_key)
            .map(str::to_owned)
            .unwrap_or_else(|| drv_path.name().to_string());
        if let Err(failure) = write_log(logs_dir, &drv_path, &label, false, &failed.error) {
            failures.push(failure);
        }
    }
    failures
}

fn print_failed_builds(result: &PipelineResult, store_dir: &str, logs_dir: &Path) {
    for envelope in build_failure_envelopes(result, store_dir, logs_dir) {
        for line in render_human_failure_summary(&envelope) {
            eprintln!("{line}");
        }
    }
}

fn prepare_logs_dir(logs_dir: &Path) -> Result<(), DiagnosticPersistenceFailure> {
    std::fs::create_dir_all(logs_dir).map_err(|error| DiagnosticPersistenceFailure::create_log_dir(logs_dir, &error))
}

fn print_diagnostic_persistence_failures(failures: &[DiagnosticPersistenceFailure]) {
    for failure in failures {
        eprintln!("WARNING: diagnostic persistence failed");
        eprintln!("  operation: {}", failure.operation);
        eprintln!("  artifact: {}", failure.artifact);
        if let Some(label) = &failure.label {
            eprintln!("  label: {label}");
        }
        eprintln!("  attempted_path: {}", failure.attempted_path);
        eprintln!("  error: {}", failure.error);
    }
}

pub fn write_log(
    log_dir: &Path,
    drv_path: &StorePath<String>,
    label: &str,
    success: bool,
    body: &str,
) -> Result<PathBuf, DiagnosticPersistenceFailure> {
    let attempted_path = log_file_path(log_dir, drv_path);
    write_log_file(log_dir, drv_path, label, success, body)
        .map_err(|error| DiagnosticPersistenceFailure::write_build_log(label, &attempted_path, &error))
}

/// An empty selected state directory or a relative environment directory fails
/// admission before filesystem access. Explicit CLI overrides may be relative.
/// Unselected fallback variables are not consulted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateDirAdmissionError {
    EmptyOverride,
    EmptyEnvironment(&'static str),
    RelativeEnvironment(&'static str),
}

impl std::fmt::Display for StateDirAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyOverride => write!(f, "empty-state-dir: --state-dir must not be empty"),
            Self::EmptyEnvironment(name) => write!(f, "empty-state-dir-env: {name} must not be empty"),
            Self::RelativeEnvironment(name) => write!(f, "relative-state-dir-env: {name} must be absolute"),
        }
    }
}

/// The selected path and authority/count actually observed by the environment
/// reader. An empty selected value remains an admission failure.
#[derive(Debug)]
pub struct ObservedStateDirAdmission {
    pub resolved: Result<PathBuf, StateDirAdmissionError>,
    pub environment_kind: Option<EffectKind>,
    pub environment_reads: u32,
}

/// Resolve the selected state directory before exporting an override or performing
/// any filesystem/store operation, recording each actual environment read.
pub fn admit_state_dir_with_observation(override_path: Option<&Path>) -> ObservedStateDirAdmission {
    admit_state_dir_with_reader(override_path, |name| std::env::var_os(name))
}

fn admitted_state_environment_path(
    name: &'static str,
    value: OsString,
    suffix: Option<&'static str>,
) -> Result<PathBuf, StateDirAdmissionError> {
    if value.is_empty() {
        return Err(StateDirAdmissionError::EmptyEnvironment(name));
    }
    let base = PathBuf::from(value);
    if !base.is_absolute() {
        return Err(StateDirAdmissionError::RelativeEnvironment(name));
    }
    Ok(match suffix {
        Some(suffix) => base.join(suffix),
        None => base,
    })
}

fn admit_state_dir_with_reader(
    override_path: Option<&Path>,
    mut read: impl FnMut(&str) -> Option<OsString>,
) -> ObservedStateDirAdmission {
    if let Some(path) = override_path {
        return ObservedStateDirAdmission {
            resolved: if path.as_os_str().is_empty() {
                Err(StateDirAdmissionError::EmptyOverride)
            } else {
                Ok(path.to_path_buf())
            },
            environment_kind: None,
            environment_reads: 0,
        };
    }
    let mut environment_kind = None;
    let mut environment_reads = 0;
    if let Some(value) =
        observe_state_environment("CRUNCH_STATE_DIR", &mut environment_kind, &mut environment_reads, &mut read)
    {
        return ObservedStateDirAdmission {
            resolved: admitted_state_environment_path("CRUNCH_STATE_DIR", value, None),
            environment_kind,
            environment_reads,
        };
    }
    if let Some(value) =
        observe_state_environment("XDG_STATE_HOME", &mut environment_kind, &mut environment_reads, &mut read)
    {
        return ObservedStateDirAdmission {
            resolved: admitted_state_environment_path("XDG_STATE_HOME", value, Some("crunch")),
            environment_kind,
            environment_reads,
        };
    }
    if let Some(value) = observe_state_environment("HOME", &mut environment_kind, &mut environment_reads, &mut read) {
        return ObservedStateDirAdmission {
            resolved: admitted_state_environment_path("HOME", value, Some(".local/state/crunch")),
            environment_kind,
            environment_reads,
        };
    }
    ObservedStateDirAdmission {
        resolved: Ok(PathBuf::from("/tmp/.local/state/crunch")),
        environment_kind,
        environment_reads,
    }
}

fn observe_state_environment(
    name: &str,
    kind: &mut Option<EffectKind>,
    environment_reads: &mut u32,
    read: &mut impl FnMut(&str) -> Option<OsString>,
) -> Option<OsString> {
    *kind = Some(EffectKind::ReadEnvironment);
    *environment_reads += 1;
    read(name)
}

/// The build-log directory selected at startup, including the observed
/// environment read. A selected empty override is never interpreted as CWD.
#[derive(Debug)]
pub struct ObservedLogDirAdmission {
    pub resolved: Result<PathBuf, StateDirAdmissionError>,
    pub environment_kind: Option<EffectKind>,
    pub environment_reads: u32,
}

pub fn admit_log_dir_with_observation(state_dir: &Path) -> ObservedLogDirAdmission {
    let selected = std::env::var_os("CRUNCH_LOG_DIR");
    let resolved = match selected {
        Some(value) if value.is_empty() => Err(StateDirAdmissionError::EmptyEnvironment("CRUNCH_LOG_DIR")),
        Some(value) => Ok(PathBuf::from(value)),
        None => Ok(state_dir.join("logs")),
    };
    ObservedLogDirAdmission {
        resolved,
        environment_kind: Some(EffectKind::ReadEnvironment),
        environment_reads: 1,
    }
}

pub fn build_import_paths(extra: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    let stdlib_dir =
        crunch_eval::stdlib::stdlib_import_path().map_err(|e| RunError::Internal(format!("stdlib: {e}")))?;
    let mut paths: Vec<OsString> = vec![stdlib_dir.into()];
    for path in extra {
        paths.push(path.into());
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use crunch_pipeline::HermeticityAuditKind;

    use super::*;

    const TEST_PRIORITY_GOAL: &str = "/private/priority-secret/root.drv";
    const TEST_PRIORITY_EPOCH: u32 = 1;
    const TEST_PRIORITY_PATH_NODES: u32 = 2;
    const TEST_PRIORITY_SUMMARY_LINE_COUNT: usize = 2;
    const FIRST_OBSERVED_SIGNAL_COUNT: u32 = 1;
    const REPEATED_OBSERVED_SIGNAL_COUNT: u32 = 2;

    #[test]
    fn state_directory_selection_counts_only_actual_environment_reads() {
        let overridden = admit_state_dir_with_reader(Some(Path::new("relative/state")), |_| {
            panic!("an explicit override must not read the environment")
        });
        assert_eq!(overridden.resolved.unwrap(), PathBuf::from("relative/state"));
        assert_eq!(overridden.environment_reads, 0);
        assert_eq!(overridden.environment_kind, None);

        for (values, selected, count) in [
            ([Some("/crunch"), Some("/xdg"), Some("/home")], Some("/crunch"), 1),
            ([None, Some("/xdg"), Some("/home")], Some("/xdg/crunch"), 2),
            ([None, None, Some("/home")], Some("/home/.local/state/crunch"), 3),
            ([None, None, None], Some("/tmp/.local/state/crunch"), 3),
            ([Some(""), Some("/xdg"), Some("/home")], None, 1),
        ] {
            let mut names = Vec::new();
            let admission = admit_state_dir_with_reader(None, |name| {
                names.push(name.to_string());
                let selected = match name {
                    "CRUNCH_STATE_DIR" => values[0],
                    "XDG_STATE_HOME" => values[1],
                    "HOME" => values[2],
                    other => panic!("unexpected environment read: {other}"),
                };
                selected.map(OsString::from)
            });
            assert_eq!(admission.environment_reads, count);
            assert_eq!(admission.environment_kind, Some(EffectKind::ReadEnvironment));
            assert_eq!(names.len(), count as usize);
            match selected {
                Some(path) => assert_eq!(admission.resolved.unwrap(), PathBuf::from(path)),
                None => assert_eq!(
                    admission.resolved.unwrap_err(),
                    StateDirAdmissionError::EmptyEnvironment("CRUNCH_STATE_DIR")
                ),
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn selected_non_utf8_state_environment_requires_absolute_path_at_each_priority() {
        use std::os::unix::ffi::OsStringExt;

        for (selected_name, reads, suffix) in [
            ("CRUNCH_STATE_DIR", 1, None),
            ("XDG_STATE_HOME", 2, Some("crunch")),
            ("HOME", 3, Some(".local/state/crunch")),
        ] {
            for (selected, is_absolute) in [
                (OsString::from_vec(b"/selected-\xff-state".to_vec()), true),
                (OsString::from_vec(b"relative-\xff-state".to_vec()), false),
            ] {
                let mut names = Vec::new();
                let observed = admit_state_dir_with_reader(None, |name| {
                    names.push(name.to_string());
                    (name == selected_name).then(|| selected.clone())
                });
                if is_absolute {
                    let expected = match suffix {
                        Some(suffix) => PathBuf::from(selected).join(suffix),
                        None => PathBuf::from(selected),
                    };
                    assert_eq!(observed.resolved.unwrap(), expected);
                } else {
                    assert_eq!(
                        observed.resolved.unwrap_err(),
                        StateDirAdmissionError::RelativeEnvironment(selected_name)
                    );
                }
                assert_eq!(observed.environment_reads, reads);
                assert_eq!(names.len(), reads as usize);
                assert_eq!(names.last().map(String::as_str), Some(selected_name));
            }
        }
    }

    #[test]
    fn operator_signal_handler_requests_then_forces_cancellation() {
        let cancellation = crunch_pipeline::EvaluationCancellation::new();
        let mut observed_signal_count = 0_u32;

        handle_operator_signal(OperatorSignal::Interrupt, &mut observed_signal_count, &cancellation)
            .expect("first signal requests cancellation");
        assert_eq!(observed_signal_count, FIRST_OBSERVED_SIGNAL_COUNT);
        assert!(cancellation.is_requested());

        let repeated = handle_operator_signal(OperatorSignal::Terminate, &mut observed_signal_count, &cancellation)
            .expect_err("repeated signal forces interruption");
        assert!(
            matches!(repeated, RunError::Reported(status) if status == crunch_evaluation_stream_core::CANCELLED_EXIT_CODE)
        );
        assert_eq!(observed_signal_count, REPEATED_OBSERVED_SIGNAL_COUNT);
    }

    fn sample_action_result_report(disposition: &str) -> crunch_build::ActionResultRuntimeReport {
        crunch_build::ActionResultRuntimeReport {
            schema: "mantle-action-result-runtime-report-v1".to_string(),
            phase: "discovery".to_string(),
            action_ref: "action-b3:demo".to_string(),
            unresolved_derivation: None,
            resolved_derivation: None,
            resolved_identity: None,
            disposition: disposition.to_string(),
            selected_result_ref: Some("result-b3:demo".to_string()),
            selected_source_id: Some("cache.example.invalid".to_string()),
            selected_source_class: Some("http".to_string()),
            trust_basis: vec!["record-signature-verified:builder-key-1".to_string()],
            conflict_class: None,
            candidate_decisions: Vec::new(),
            publication_result_refs: Vec::new(),
            transfer: None,
            diagnostics: Vec::new(),
            non_claims: vec!["index-presence-is-not-output-trust".to_string()],
        }
    }

    fn sample_priority_decision() -> crunch_pipeline::PriorityDecisionEvidence {
        let policy = crunch_build::SchedulingPolicy::default();
        let ready = crunch_build::ReadyGoalFacts::ordinary(TEST_PRIORITY_GOAL.to_string(), 0);
        let pressures =
            std::collections::BTreeMap::from([(TEST_PRIORITY_GOAL.to_string(), crunch_build::KnownGraphPressure {
                known_critical_path_nodes: TEST_PRIORITY_PATH_NODES,
                known_critical_path_work_units: TEST_PRIORITY_PATH_NODES,
                blocked_root_count: TEST_PRIORITY_EPOCH,
                blocked_root_count_saturated: false,
            })]);
        let ranked = crunch_build::rank_ready_goals(&policy, TEST_PRIORITY_EPOCH, &[ready], &pressures).unwrap();
        crunch_build::priority_decision_evidence(
            &policy,
            TEST_PRIORITY_EPOCH,
            &ranked,
            crunch_build::HistoryBasis::StructuralFallbackMissing,
            None,
        )
        .unwrap()
    }

    #[test]
    fn format_action_result_summary_reports_selected_source_and_omits_empty_input() {
        let lines = format_action_result_summary(&[sample_action_result_report("reused")]);

        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("disposition=reused"));
        assert!(lines[0].contains("selected=result-b3:demo"));
        assert!(lines[0].contains("source=http"));
        assert!(lines[0].contains("record-signature-verified:builder-key-1"));
        assert!(lines[0].contains("index-presence-is-not-output-trust"));
        assert!(format_action_result_summary(&[]).is_empty());
    }

    #[test]
    fn format_priority_summary_is_bounded_and_redacted() {
        let decisions = vec![sample_priority_decision()];
        let lines = format_priority_summary(&decisions);
        let rendered = lines.join("\n");

        assert_eq!(lines.len(), TEST_PRIORITY_SUMMARY_LINE_COUNT);
        assert!(rendered.contains("scope=configured-known-fact-ordering"));
        assert!(rendered.contains("graph=known-graph:path=2"));
        assert!(!rendered.contains(TEST_PRIORITY_GOAL));
        assert!(!rendered.contains("priority-secret/root.drv"));
    }

    #[test]
    fn format_priority_summary_caps_human_rows_without_dropping_machine_evidence() {
        let decision_count = MAX_HUMAN_PRIORITY_ROWS.saturating_add(1);
        let decisions = vec![sample_priority_decision(); decision_count];

        let lines = format_priority_summary(&decisions);

        assert_eq!(lines.len(), MAX_HUMAN_PRIORITY_ROWS.saturating_add(PRIORITY_SUMMARY_EXTRA_LINES));
        assert!(lines.last().is_some_and(|line| line.contains("omitted=1")));
        assert_eq!(decisions.len(), decision_count);
    }

    #[test]
    fn format_priority_summary_omits_empty_evidence() {
        assert!(format_priority_summary(&[]).is_empty());
        assert_eq!(format_priority_summary(&[]).len(), 0);
    }

    #[test]
    fn format_hermeticity_summary_reports_clean_mode() {
        let lines = format_hermeticity_summary(HermeticityMode::Strict, &[]);
        assert_eq!(lines, vec!["hermeticity: strict (no degraded facts)".to_string()]);
    }

    #[test]
    fn format_hermeticity_summary_reports_explicit_impure_mode() {
        let events = vec![HermeticityAuditEvent::new(
            HermeticityAuditKind::ImpureModeSelected,
            "explicit --impure mode permits ambient host dependencies",
        )];
        let lines = format_hermeticity_summary(HermeticityMode::Impure, &events);
        assert_eq!(lines[0], "WARNING: degraded hermeticity: impure (1 audit event)");
        assert_eq!(lines[1], "  - impure-mode-selected: explicit --impure mode permits ambient host dependencies");
    }

    #[test]
    fn format_hermeticity_summary_reports_degraded_events() {
        let events = vec![HermeticityAuditEvent::new(
            HermeticityAuditKind::HostToolFallback,
            "using external bwrap",
        )];
        let lines = format_hermeticity_summary(HermeticityMode::Practical, &events);
        assert_eq!(lines[0], "WARNING: degraded hermeticity: practical (1 audit event)");
        assert_eq!(lines[1], "  - host-tool-fallback: using external bwrap");
    }

    #[test]
    fn audit_event_label_pluralizes_count() {
        assert_eq!(audit_event_label(1), "audit event");
        assert_eq!(audit_event_label(2), "audit events");
    }

    #[test]
    fn format_output_suffix_includes_full_substitution_report() {
        let outcome = crunch_build::BuildOutcome {
            drv_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [1u8; 20]).unwrap(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::from([(
                "out".to_string(),
                crunch_store::OutputSubstitutionReport {
                    mode: crunch_store::OutputSubstitutionMode::Full,
                    transferred_bytes: 55,
                    reused_bytes: 0,
                    metadata_reused: false,
                    fallback_reason: Some("stream_application_failed".to_string()),
                },
            )]),
            cached: true,
            log: None,
        };

        let suffix = format_output_suffix(&outcome, "out", false);
        assert_eq!(
            suffix,
            " (cached, substitution=full, transferred_bytes=55, reused_bytes=0, fallback_reason=stream_application_failed)"
        );
    }

    #[test]
    fn format_output_suffix_includes_delta_substitution_report_without_fallback_reason() {
        let outcome = crunch_build::BuildOutcome {
            drv_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [2u8; 20]).unwrap(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::from([(
                "out".to_string(),
                crunch_store::OutputSubstitutionReport {
                    mode: crunch_store::OutputSubstitutionMode::Delta,
                    transferred_bytes: 12,
                    reused_bytes: 34,
                    metadata_reused: false,
                    fallback_reason: None,
                },
            )]),
            cached: true,
            log: None,
        };

        let suffix = format_output_suffix(&outcome, "out", false);
        assert_eq!(suffix, " (cached, substitution=delta, transferred_bytes=12, reused_bytes=34)");
    }
}
