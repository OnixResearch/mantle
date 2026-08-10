use std::ffi::OsString;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use crunch_build::EvalMessage;
use crunch_eval::session::RootForceExecutionPolicy;
use crunch_evaluation_stream_core::BoundedDiagnostic;
use crunch_evaluation_stream_core::DispatchDecision;
use crunch_evaluation_stream_core::FailureFact;
use crunch_evaluation_stream_core::FailureScope;
use crunch_evaluation_stream_core::IdentityContext;
use crunch_evaluation_stream_core::OutcomeLedger;
use crunch_evaluation_stream_core::RootReferences;
use crunch_evaluation_stream_core::RootSet;
use crunch_evaluation_stream_core::RunSummary;
use crunch_evaluation_stream_core::SelectedRoot;
use crunch_evaluation_stream_core::SourceSequence;
use crunch_evaluation_stream_core::TransitionResult;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use nix_compat::store_path::StorePath;
use tokio::sync::Notify;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::info;

use crate::Error;
use crate::derivation_file;
use crate::resolve_eval_parallelism;

const ALL_ROOTS_SELECTOR: &str = "all-roots";
const WORKER_PANIC_DIAGNOSTIC: &str = "evaluation worker panicked";
const WORKER_JOIN_DIAGNOSTIC: &str = "evaluation worker join failed";
const OPERATOR_CANCELLATION_DIAGNOSTIC: &str = "evaluation cancelled by operator";
const ROOT_ID_HEX_BYTES: usize = 64;

/// Cooperative cancellation handle for selected-root evaluation.
#[derive(Clone, Debug, Default)]
pub struct EvaluationCancellation {
    inner: Arc<CancellationInner>,
}

#[derive(Debug, Default)]
struct CancellationInner {
    requested: AtomicBool,
    notify: Notify,
}

impl EvaluationCancellation {
    /// Create an uncancelled handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation and wake the pipeline coordinator.
    pub fn request(&self) {
        self.inner.requested.store(true, Ordering::Release);
        self.inner.notify.notify_waiters();
    }

    /// Return true after cancellation is requested.
    pub fn is_requested(&self) -> bool {
        self.inner.requested.load(Ordering::Acquire)
    }

    async fn cancelled(&self) {
        if self.is_requested() {
            return;
        }
        let notified = self.inner.notify.notified();
        if self.is_requested() {
            return;
        }
        notified.await;
        debug_assert!(self.is_requested());
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EvalWorkerControl {
    #[cfg(test)]
    panic_label: Option<String>,
    #[cfg(test)]
    pause_after_evaluation_label: Option<String>,
    #[cfg(test)]
    evaluation_completed: Option<Arc<AtomicBool>>,
    #[cfg(test)]
    release_evaluation: Option<Arc<AtomicBool>>,
}

impl EvalWorkerControl {
    #[cfg(test)]
    pub(crate) fn panic_on(label: String) -> Self {
        Self {
            panic_label: Some(label),
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn pause_after_evaluation(
        label: String,
        evaluation_completed: Arc<AtomicBool>,
        release_evaluation: Arc<AtomicBool>,
    ) -> Self {
        Self {
            pause_after_evaluation_label: Some(label),
            evaluation_completed: Some(evaluation_completed),
            release_evaluation: Some(release_evaluation),
            ..Self::default()
        }
    }

    fn before_evaluation(&self, label: &str) {
        #[cfg(test)]
        if self.panic_label.as_deref() == Some(label) {
            panic!("injected evaluation worker loss");
        }
        #[cfg(not(test))]
        let _ = label;
    }

    fn after_evaluation(&self, label: &str) {
        #[cfg(test)]
        if self.pause_after_evaluation_label.as_deref() == Some(label) {
            let completed = self.evaluation_completed.as_ref().expect("test completion probe");
            let release = self.release_evaluation.as_ref().expect("test release probe");
            completed.store(true, Ordering::Release);
            while !release.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
        }
        #[cfg(not(test))]
        let _ = label;
    }
}

pub(crate) struct EvalStreamRequest<'a> {
    pub(crate) max_jobs: u32,
    pub(crate) store_dir: &'a str,
    pub(crate) root_force_policy: RootForceExecutionPolicy,
    pub(crate) root_file: &'a std::path::Path,
    pub(crate) import_paths: &'a [OsString],
    pub(crate) session: &'a crunch_eval::session::EvaluationSession,
    pub(crate) tx: mpsc::Sender<EvalMessage>,
    pub(crate) cancellation: EvaluationCancellation,
    pub(crate) worker_control: EvalWorkerControl,
}

pub(crate) struct EvalStreamResult {
    pub(crate) root_drv_paths: Vec<(String, StorePath<String>)>,
    pub(crate) summary: RunSummary,
}

#[derive(Debug)]
enum EvalWorkerCompletion {
    Succeeded {
        root: SelectedRoot,
        derivation: Box<CrunchDerivation>,
    },
    Failed {
        root: SelectedRoot,
        fact: FailureFact,
        error: String,
    },
    Cancelled {
        root: SelectedRoot,
    },
    WorkerLost {
        root: SelectedRoot,
        error: String,
    },
}

impl EvalWorkerCompletion {
    fn sequence(&self) -> SourceSequence {
        match self {
            Self::Succeeded { root, .. }
            | Self::Failed { root, .. }
            | Self::Cancelled { root }
            | Self::WorkerLost { root, .. } => root.sequence(),
        }
    }
}

struct EvalWorkerLaunch {
    worker_input: crunch_eval::session::IsolatedWorkerInput,
    root: SelectedRoot,
    root_force_policy: RootForceExecutionPolicy,
    cancellation: EvaluationCancellation,
    worker_control: EvalWorkerControl,
}

struct EvalStreamState<'a> {
    request: EvalStreamRequest<'a>,
    selected_roots: Vec<SelectedRoot>,
    ledger: OutcomeLedger,
    worker_input: crunch_eval::session::IsolatedWorkerInput,
    join_set: JoinSet<EvalWorkerCompletion>,
    next_root_index: usize,
    eval_parallelism: usize,
    cache: ConversionCache,
    file_resolver: derivation_file::DerivationFileResolver,
    root_drv_paths: Vec<(String, StorePath<String>)>,
    dispatch_stopped: bool,
}

impl<'a> EvalStreamState<'a> {
    fn new(
        request: EvalStreamRequest<'a>,
        root_set: RootSet,
        file_resolver: derivation_file::DerivationFileResolver,
    ) -> Result<Self, Error> {
        let eval_parallelism_u32 = resolve_eval_parallelism(request.max_jobs, root_set.root_count());
        let eval_parallelism = usize::try_from(eval_parallelism_u32)
            .map_err(|_| Error::Internal("evaluation parallelism does not fit usize".to_string()))?;
        let selected_roots = root_set.roots();
        Ok(Self {
            worker_input: request.session.isolated_worker_input(),
            cache: ConversionCache::new(request.store_dir),
            ledger: OutcomeLedger::new(root_set),
            join_set: JoinSet::new(),
            next_root_index: 0,
            root_drv_paths: Vec::with_capacity(selected_roots.len()),
            dispatch_stopped: false,
            selected_roots,
            eval_parallelism,
            file_resolver,
            request,
        })
    }

    fn spawn_available(&mut self) -> Result<(), Error> {
        while !self.dispatch_stopped
            && self.next_root_index < self.selected_roots.len()
            && self.join_set.len() < self.eval_parallelism
        {
            let root = self.selected_roots[self.next_root_index].clone();
            self.next_root_index = self.next_root_index.saturating_add(1);
            self.ledger = self.ledger.start(root.sequence()).map_err(outcome_error)?;
            spawn_eval_worker(&mut self.join_set, EvalWorkerLaunch {
                worker_input: self.worker_input.clone(),
                root,
                root_force_policy: self.request.root_force_policy,
                cancellation: self.request.cancellation.clone(),
                worker_control: self.request.worker_control.clone(),
            });
        }
        debug_assert!(self.join_set.len() <= self.eval_parallelism);
        debug_assert!(self.next_root_index <= self.selected_roots.len());
        Ok(())
    }

    async fn run(mut self) -> Result<EvalStreamResult, Error> {
        if self.request.cancellation.is_requested() {
            self.apply_stop(FailureScope::Cancellation, OPERATOR_CANCELLATION_DIAGNOSTIC)?;
        } else {
            self.spawn_available()?;
        }
        while !self.join_set.is_empty() {
            if self.dispatch_stopped {
                self.join_set.join_next().await;
                continue;
            }
            let cancellation = self.request.cancellation.clone();
            tokio::select! {
                _ = cancellation.cancelled() => {
                    self.apply_stop(FailureScope::Cancellation, OPERATOR_CANCELLATION_DIAGNOSTIC)?;
                }
                join_result = self.join_set.join_next() => {
                    self.handle_join_result(join_result).await?;
                }
            }
            self.spawn_available()?;
        }
        drop(self.request.tx);
        let summary = self.ledger.finish().map_err(outcome_error)?;
        Ok(EvalStreamResult {
            root_drv_paths: self.root_drv_paths,
            summary,
        })
    }

    async fn handle_join_result(
        &mut self,
        join_result: Option<Result<EvalWorkerCompletion, tokio::task::JoinError>>,
    ) -> Result<(), Error> {
        let Some(join_result) = join_result else {
            return Ok(());
        };
        let completion = match join_result {
            Ok(completion) => completion,
            Err(_) => {
                self.apply_stop(FailureScope::CoordinatorFailure, WORKER_JOIN_DIAGNOSTIC)?;
                return Ok(());
            }
        };
        if self.dispatch_stopped {
            return Ok(());
        }
        self.handle_completion(completion).await
    }

    async fn handle_completion(&mut self, completion: EvalWorkerCompletion) -> Result<(), Error> {
        if self.request.cancellation.is_requested() {
            return self.apply_failure(
                completion.sequence(),
                FailureFact::OperatorCancellation,
                OPERATOR_CANCELLATION_DIAGNOSTIC.to_string(),
            );
        }
        match completion {
            EvalWorkerCompletion::Succeeded { root, derivation } => self.handle_success(root, *derivation).await,
            EvalWorkerCompletion::Failed { root, fact, error } => self.apply_failure(root.sequence(), fact, error),
            EvalWorkerCompletion::Cancelled { root } => self.apply_failure(
                root.sequence(),
                FailureFact::OperatorCancellation,
                OPERATOR_CANCELLATION_DIAGNOSTIC.to_string(),
            ),
            EvalWorkerCompletion::WorkerLost { root, error } => {
                self.apply_failure(root.sequence(), FailureFact::Coordinator, error)
            }
        }
    }

    async fn handle_success(&mut self, root: SelectedRoot, mut derivation: CrunchDerivation) -> Result<(), Error> {
        if self.request.cancellation.is_requested() {
            return self.apply_failure(
                root.sequence(),
                FailureFact::OperatorCancellation,
                OPERATOR_CANCELLATION_DIAGNOSTIC.to_string(),
            );
        }
        if let Err(error) =
            self.file_resolver.resolve_root_inputs(self.request.root_file, &mut derivation, &mut self.cache)
        {
            return self.apply_failure(root.sequence(), FailureFact::RootConversion, error.to_string());
        }
        let label = root.label();
        let (drv_path, _nix_drv) = match crunch_glue::convert(&derivation, &mut self.cache) {
            Ok(converted) => converted,
            Err(error) => {
                return self.apply_failure(root.sequence(), FailureFact::RootConversion, format!("{label}: {error}"));
            }
        };
        self.send_converted_root(root, drv_path).await
    }

    async fn send_converted_root(&mut self, root: SelectedRoot, drv_path: StorePath<String>) -> Result<(), Error> {
        let label = root.label();
        let new_entries = self.cache.drain_pending();
        info!(drv = %drv_path, label = %label, entries = new_entries.len(), "converted, sending to worker");
        if let Err(error) = self
            .request
            .tx
            .send(EvalMessage {
                label: label.clone(),
                drv_path: drv_path.clone(),
                new_entries,
            })
            .await
        {
            return self.apply_failure(root.sequence(), FailureFact::Coordinator, format!("channel send: {error}"));
        }
        let logical_path = drv_path.to_absolute_path_with_prefix(self.request.store_dir);
        let references = RootReferences::new(Some(logical_path), None, None).map_err(outcome_error)?;
        let transition = self.ledger.record_success(root.sequence(), references).map_err(outcome_error)?;
        self.apply_transition(transition);
        self.root_drv_paths.push((label, drv_path));
        Ok(())
    }

    fn apply_failure(&mut self, sequence: SourceSequence, fact: FailureFact, error: String) -> Result<(), Error> {
        let transition =
            self.ledger.record_failure(sequence, fact, BoundedDiagnostic::new(error)).map_err(outcome_error)?;
        self.apply_transition(transition);
        Ok(())
    }

    fn apply_stop(&mut self, scope: FailureScope, diagnostic: &str) -> Result<(), Error> {
        let transition = self
            .ledger
            .stop_remaining(scope, BoundedDiagnostic::new(diagnostic.to_string()))
            .map_err(outcome_error)?;
        self.apply_transition(transition);
        Ok(())
    }

    fn apply_transition(&mut self, transition: TransitionResult) {
        if transition.decision() == DispatchDecision::Stop {
            self.dispatch_stopped = true;
        }
        self.ledger = transition.into_ledger();
        debug_assert!(!self.dispatch_stopped || self.ledger.finish().is_ok());
        debug_assert!(self.root_drv_paths.len() <= self.selected_roots.len());
    }
}

pub(crate) async fn stream_roots_into_worker(request: EvalStreamRequest<'_>) -> Result<EvalStreamResult, Error> {
    let root_set = admit_root_set(request.session)?;
    let file_resolver = match derivation_file::DerivationFileResolver::new(request.root_file, request.import_paths) {
        Ok(resolver) => resolver,
        Err(error) => return shared_initialization_failure(request.tx, root_set, error.to_string()),
    };
    EvalStreamState::new(request, root_set, file_resolver)?.run().await
}

fn admit_root_set(session: &crunch_eval::session::EvaluationSession) -> Result<RootSet, Error> {
    let labels = session.root_labels().iter().map(|root_label| root_label.label.clone()).collect::<Vec<_>>();
    let evaluator_cohort = format!("{}@{}", crunch_eval::EVALUATOR_ID, crunch_eval::EVALUATOR_VERSION);
    let context = IdentityContext::new(evaluator_cohort, session.source_blake3(), ALL_ROOTS_SELECTOR.to_string())
        .map_err(outcome_error)?;
    RootSet::admit(context, labels).map_err(outcome_error)
}

fn shared_initialization_failure(
    tx: mpsc::Sender<EvalMessage>,
    root_set: RootSet,
    error: String,
) -> Result<EvalStreamResult, Error> {
    let ledger = OutcomeLedger::new(root_set);
    let transition = ledger
        .stop_remaining(FailureScope::SharedFatal, BoundedDiagnostic::new(error))
        .map_err(outcome_error)?;
    drop(tx);
    let summary = transition.into_ledger().finish().map_err(outcome_error)?;
    Ok(EvalStreamResult {
        root_drv_paths: Vec::new(),
        summary,
    })
}

fn spawn_eval_worker(join_set: &mut JoinSet<EvalWorkerCompletion>, launch: EvalWorkerLaunch) {
    join_set.spawn_blocking(move || {
        let panic_root = launch.root.clone();
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            execute_eval_worker(
                launch.worker_input,
                launch.root,
                launch.root_force_policy,
                launch.cancellation,
                launch.worker_control,
            )
        }));
        match result {
            Ok(completion) => completion,
            Err(_) => EvalWorkerCompletion::WorkerLost {
                root: panic_root,
                error: WORKER_PANIC_DIAGNOSTIC.to_string(),
            },
        }
    });
}

fn execute_eval_worker(
    worker_input: crunch_eval::session::IsolatedWorkerInput,
    root: SelectedRoot,
    root_force_policy: RootForceExecutionPolicy,
    cancellation: EvaluationCancellation,
    worker_control: EvalWorkerControl,
) -> EvalWorkerCompletion {
    if cancellation.is_requested() {
        return EvalWorkerCompletion::Cancelled { root };
    }
    let label = root.label();
    debug_assert!(!label.is_empty());
    debug_assert_eq!(root.root_id().len(), ROOT_ID_HEX_BYTES);
    worker_control.before_evaluation(&label);
    let result = force_worker_root(&worker_input, &label, root_force_policy);
    worker_control.after_evaluation(&label);
    if cancellation.is_requested() {
        return EvalWorkerCompletion::Cancelled { root };
    }
    match result {
        Ok(derivation) => EvalWorkerCompletion::Succeeded {
            root,
            derivation: Box::new(derivation),
        },
        Err(error) => EvalWorkerCompletion::Failed {
            root,
            fact: failure_fact(error.failure_scope_hint()),
            error: format!("root '{label}': {error}"),
        },
    }
}

fn force_worker_root(
    worker_input: &crunch_eval::session::IsolatedWorkerInput,
    label: &str,
    root_force_policy: RootForceExecutionPolicy,
) -> Result<CrunchDerivation, crunch_eval::Error> {
    match root_force_policy {
        RootForceExecutionPolicy::Inline | RootForceExecutionPolicy::PreferThreaded => worker_input.force_root(label),
    }
}

fn failure_fact(scope: crunch_eval::FailureScopeHint) -> FailureFact {
    match scope {
        crunch_eval::FailureScopeHint::RootScoped => FailureFact::RootEvaluation,
        crunch_eval::FailureScopeHint::SharedFatal => FailureFact::SharedSource,
        crunch_eval::FailureScopeHint::CoordinatorFailure => FailureFact::Coordinator,
    }
}

fn outcome_error(error: crunch_evaluation_stream_core::OutcomeError) -> Error {
    Error::Internal(format!("evaluation outcome core: {error}"))
}
