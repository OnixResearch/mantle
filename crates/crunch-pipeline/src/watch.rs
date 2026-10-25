//! Opt-in file-backed Nickel watch. Only the persistent Worker admits builds;
//! this shell offers complete, stable source generations and publishes its
//! terminal outcomes, never outcomes inferred from evaluation alone.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crunch_build::BuildOutcome;
use crunch_build::DerivationRegistry;
use crunch_build::EvalMessage;
use crunch_build::FailedGoal;
use crunch_build::Worker;
use crunch_build::worker::WatchWorkerEvent;
use crunch_build::worker::WatchWorkerUpdate;
use crunch_eval::session::RootForceExecutionPolicy;
use crunch_eval::session::incremental::PersistentRootCache;
use crunch_eval::session::incremental::PreparedRoots;
use crunch_eval::session::incremental::ReuseDiagnostics;
use crunch_evaluation_stream_core::RunDisposition;
use crunch_glue::CrunchDerivation;
use crunch_watch_core::GoalDiff;
use crunch_watch_core::GoalSet;
use crunch_watch_core::MAX_EVALUATIONS_PER_WINDOW;
use crunch_watch_core::MAX_LIVE_GOALS;
use crunch_watch_core::MAX_TRANSITION_EVENTS;
use crunch_watch_core::WatchLoop;
use tokio::sync::mpsc;
use tokio::sync::watch;

use crate::BuildConfig;
use crate::EVAL_MESSAGE_CHANNEL_CAPACITY;
use crate::Error;
#[cfg(target_os = "linux")]
use crate::create_watch_pipeline_builder;
use crate::evaluation_stream::EvalStreamRequest;
use crate::evaluation_stream::EvalWorkerControl;
use crate::evaluation_stream::EvaluationCancellation;
use crate::evaluation_stream::stream_roots_into_worker;
#[cfg(target_os = "linux")]
use crate::register_managed_generation;
#[cfg(target_os = "linux")]
use crate::store_fallback_mode;
#[cfg(target_os = "linux")]
use crate::validate_build_config;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const EVALUATION_WINDOW: Duration = Duration::from_secs(60);
const _: () = assert!(MAX_EVALUATIONS_PER_WINDOW > 0);
const MAX_SNAPSHOT_BYTES: usize = 16_777_216;
const MAX_SNAPSHOT_FILE_BYTES: usize = 2_097_152;
const WORKER_EVENT_CAPACITY: usize = 16;

/// Admission notices describe the actual admitted goal set. A `Settled` notice
/// contains only the current Worker's terminal roots, not an evaluation result.
#[derive(Debug)]
pub enum WatchNotice {
    /// Evaluation observations, never scheduler admission or build success.
    Evaluation {
        mode: &'static str,
        diagnostics: ReuseDiagnostics,
    },
    Goals {
        generation: u64,
        diff: GoalDiff,
    },
    Rejected {
        diff: GoalDiff,
    },
    PendingCancellation {
        generation: u64,
        stale: Vec<String>,
    },
    Settled {
        generation: u64,
        outcomes: Vec<BuildOutcome>,
        failed: Vec<FailedGoal>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceSnapshot(Vec<(PathBuf, Vec<u8>)>);

/// Scan the parsed transitive import graph and fingerprint the exact source
/// bytes. Both the scanner and this read have independent 256-file/16-MiB
/// budgets. Lexically normalized paths match Nickel's source-cache identity.
fn source_snapshot(root: &Path, import_paths: &[OsString]) -> Result<SourceSnapshot, Error> {
    let paths = crunch_eval::watch_imports::source_dependencies(root, import_paths)
        .map_err(|_| Error::Eval("watched source is missing or invalid".to_string()))?;
    let mut remaining = MAX_SNAPSHOT_BYTES;
    let mut sources = Vec::with_capacity(paths.len());
    for path in paths {
        let bound = remaining.min(MAX_SNAPSHOT_FILE_BYTES);
        let read_limit_bytes = u64::try_from(bound)
            .map_err(|_| Error::Eval("watched sources exceed snapshot byte limit".to_string()))?
            .saturating_add(1);
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .and_then(|file| file.take(read_limit_bytes).read_to_end(&mut bytes))
            .map_err(|_| Error::Eval("watched source is missing or unreadable".to_string()))?;
        if bytes.len() > bound {
            return Err(Error::Eval("watched sources exceed snapshot byte limit".to_string()));
        }
        remaining -= bytes.len();
        sources.push((path, bytes));
    }
    Ok(SourceSnapshot(sources))
}

fn watch_error(error: crunch_watch_core::WatchError) -> Error {
    Error::Build(format!("watch admission: {error:?}"))
}

async fn reject(
    watch_loop: &mut WatchLoop,
    notices: &mpsc::Sender<WatchNotice>,
    diagnostic: &str,
) -> Result<(), Error> {
    let diff = watch_loop.reject(diagnostic.to_string(), MAX_TRANSITION_EVENTS).map_err(watch_error)?;
    notices
        .send(WatchNotice::Rejected { diff })
        .await
        .map_err(|_| Error::Internal("watch notice receiver closed".to_string()))
}

/// Fully buffer conversion messages before sending a generation to the Worker.
/// An unsuccessful evaluation never changes the Worker's admitted goal set.
async fn evaluate(config: &BuildConfig) -> Result<(GoalSet, Vec<EvalMessage>, SourceSnapshot), Error> {
    let before = source_snapshot(&config.file, &config.import_paths)?;
    let session = crunch_eval::session::EvaluationSession::open_file(&config.file, &config.import_paths)
        .map_err(|_| Error::Eval("watched source evaluation failed".to_string()))?;
    // The one-shot outcome ledger rejects empty root sets, but a watch edit
    // removing the last root is a valid retraction of the previous set.
    if session.root_labels().is_empty() {
        let after = source_snapshot(&config.file, &config.import_paths)?;
        if before != after {
            return Err(Error::Eval("watched sources changed during evaluation".to_string()));
        }
        return Ok((GoalSet::admit(Vec::new()).map_err(watch_error)?, Vec::new(), after));
    }
    let max_live_goals = usize::try_from(MAX_LIVE_GOALS)
        .map_err(|_| Error::Eval("watched root evaluation or conversion failed".to_string()))?;
    let (tx, mut rx) = mpsc::channel(EVAL_MESSAGE_CHANNEL_CAPACITY);
    let (result, roots) = tokio::join!(
        stream_roots_into_worker(EvalStreamRequest {
            max_jobs: config.max_jobs,
            store_dir: &config.store_dir,
            root_force_policy: RootForceExecutionPolicy::PreferThreaded,
            root_file: &config.file,
            import_paths: &config.import_paths,
            session: &session,
            tx,
            cancellation: EvaluationCancellation::new(),
            stream_tx: None,
            worker_control: EvalWorkerControl::default(),
        }),
        async {
            let mut roots = Vec::new();
            while let Some(root) = rx.recv().await {
                // Retain one excess root to reject the generation, but drain
                // the channel so evaluation can finish without growing memory.
                if roots.len() <= max_live_goals {
                    roots.push(root);
                }
            }
            roots
        },
    );
    let result = result.map_err(|_| Error::Eval("watched root evaluation or conversion failed".to_string()))?;
    if result.summary.disposition() != RunDisposition::Success || roots.len() > max_live_goals {
        return Err(Error::Eval("watched root evaluation or conversion failed".to_string()));
    }
    let after = source_snapshot(&config.file, &config.import_paths)?;
    if before != after {
        return Err(Error::Eval("watched sources changed during evaluation".to_string()));
    }
    let goals =
        GoalSet::admit(roots.iter().map(|root| root.drv_path.to_absolute_path()).collect()).map_err(watch_error)?;
    Ok((goals, roots, after))
}

type CachedEvaluation = (GoalSet, Vec<EvalMessage>, SourceSnapshot, Option<PreparedRoots<CrunchDerivation>>);

fn evaluation_configuration(config: &BuildConfig) -> Vec<u8> {
    format!("{}\0{}", config.store_dir, config.max_jobs).into_bytes()
}

fn cache_identity(config: &BuildConfig) -> Result<[u8; 32], Error> {
    crunch_eval::session::incremental::configuration_fingerprint(
        &config.file,
        &config.import_paths,
        &evaluation_configuration(config),
    )
    .map_err(|error| Error::Eval(error.to_string()))
}

async fn evaluate_cached(
    config: &BuildConfig,
    cache: &mut PersistentRootCache<CrunchDerivation>,
    shutdown: &watch::Receiver<bool>,
) -> Result<CachedEvaluation, Error> {
    let before = source_snapshot(&config.file, &config.import_paths)?;
    let configuration = evaluation_configuration(config);
    let Some(candidate) = cache
        .prepare(&config.file, &config.import_paths, &configuration, || *shutdown.borrow())
        .map_err(|error| Error::Eval(error.to_string()))?
    else {
        let (goals, roots, snapshot) = evaluate(config).await?;
        if *shutdown.borrow() {
            return Err(Error::Eval("watched evaluation cancelled".to_string()));
        }
        return Ok((goals, roots, snapshot, None));
    };
    let mut conversion = crunch_glue::ConversionCache::new(&config.store_dir);
    let mut resolver = crate::derivation_file::DerivationFileResolver::new(&config.file, &config.import_paths)?;
    let mut roots = Vec::new();
    for (label, value) in candidate.roots() {
        if *shutdown.borrow() {
            return Err(Error::Eval("watched evaluation cancelled".to_string()));
        }
        // Resolution/conversion is always fresh; retained values are immutable.
        let mut derivation = value.clone();
        resolver.resolve_root_inputs(&config.file, &mut derivation, &mut conversion)?;
        let (drv_path, _) =
            crunch_glue::convert(&derivation, &mut conversion).map_err(|error| Error::Convert(error.to_string()))?;
        roots.push(EvalMessage {
            label: label.to_string(),
            drv_path,
            new_entries: conversion.drain_pending(),
        });
    }
    candidate
        .validate(&configuration, || *shutdown.borrow())
        .map_err(|error| Error::Eval(error.to_string()))?;
    let after = source_snapshot(&config.file, &config.import_paths)?;
    if before != after {
        return Err(Error::Eval("watched sources changed during evaluation".to_string()));
    }
    let goals =
        GoalSet::admit(roots.iter().map(|root| root.drv_path.to_absolute_path()).collect()).map_err(watch_error)?;
    Ok((goals, roots, after, Some(candidate)))
}

async fn restore_committed(
    watch_loop: &mut WatchLoop,
    notices: &mpsc::Sender<WatchNotice>,
    updates: &mpsc::Sender<WatchWorkerUpdate>,
    committed_roots: &[EvalMessage],
    diagnostic: &str,
) -> Result<u64, Error> {
    reject(watch_loop, notices, diagnostic).await?;
    let previous = watch_loop.admitted().clone();
    let generation = watch_loop.offer(previous, MAX_TRANSITION_EVENTS).map_err(watch_error)?;
    updates
        .send(WatchWorkerUpdate {
            generation,
            roots: committed_roots.to_vec(),
        })
        .await
        .map_err(|_| Error::Internal("watch worker closed update channel".to_string()))?;
    Ok(generation)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateAuthorization {
    Authorized,
    Superseded,
    Changed,
    Expired,
}

/// Authorize exactly one stable candidate. Denial drops the one-shot token;
/// the Worker must not dispatch or publish any result for that candidate.
async fn authorize_candidate(
    config: &BuildConfig,
    watch_loop: &mut WatchLoop,
    pending: Option<u64>,
    candidate: Option<&SourceSnapshot>,
    restoring: bool,
    generation: u64,
    commit: tokio::sync::oneshot::Sender<()>,
    notices: &mpsc::Sender<WatchNotice>,
) -> Result<CandidateAuthorization, Error> {
    if pending != Some(generation) {
        drop(commit);
        if pending.is_none() {
            return Err(Error::Internal("watch worker prepared an unrequested generation".to_string()));
        }
        return Ok(CandidateAuthorization::Superseded);
    }
    if !restoring
        && (candidate.is_none() || source_snapshot(&config.file, &config.import_paths).ok().as_ref() != candidate)
    {
        drop(commit);
        return Ok(CandidateAuthorization::Changed);
    }
    let (_, diff) = watch_loop.pending().ok_or_else(|| Error::Internal("missing watch candidate".to_string()))?;
    let diff = diff.clone();
    let previous = watch_loop.admitted().clone();
    watch_loop.commit(generation).map_err(watch_error)?;
    if commit.send(()).is_err() {
        // The Worker's token expired before authorization. Undo the pure
        // speculative commit without publishing it, then queue a fresh
        // Worker-authorized generation on the next source poll.
        let rollback = watch_loop.offer(previous, MAX_TRANSITION_EVENTS).map_err(watch_error)?;
        watch_loop.commit(rollback).map_err(watch_error)?;
        return Ok(CandidateAuthorization::Expired);
    }
    if !restoring {
        notices
            .send(WatchNotice::Goals { generation, diff })
            .await
            .map_err(|_| Error::Internal("watch notice receiver closed".to_string()))?;
    }
    Ok(CandidateAuthorization::Authorized)
}

#[cfg(target_os = "linux")]
struct WatchCoordinator<'a> {
    config: &'a BuildConfig,
    watch_loop: WatchLoop,
    root_registry: &'a crunch_store::RootRegistry,
    workspace_evidence_sink: &'a crunch_build::WorkspaceReportCollector,
    notices: &'a mpsc::Sender<WatchNotice>,
    updates: mpsc::Sender<WatchWorkerUpdate>,
    attempts: VecDeque<Instant>,
    max_evaluations: usize,
    rate_rejection_reported: bool,
    candidate: Option<SourceSnapshot>,
    committed_roots: Vec<EvalMessage>,
    pending_roots: Vec<EvalMessage>,
    rollback_pending: bool,
    baseline: Option<SourceSnapshot>,
    pending: Option<u64>,
    current_is_rollback: bool,
    current: Option<u64>,
    dirty: bool,
    evaluation_cache: Option<PersistentRootCache<CrunchDerivation>>,
    pending_evaluation: Option<PreparedRoots<CrunchDerivation>>,
    shutdown: watch::Receiver<bool>,
    pending_configuration: Option<[u8; 32]>,
    baseline_configuration: Option<[u8; 32]>,
}

#[cfg(target_os = "linux")]
impl WatchCoordinator<'_> {
    async fn on_event(&mut self, event: WatchWorkerEvent) -> Result<(), Error> {
        match event {
            WatchWorkerEvent::CandidateReady { generation, commit } => self.candidate_ready(generation, commit).await,
            WatchWorkerEvent::Rejected { generation } => {
                if self.pending == Some(generation) {
                    self.restore_pending("watched scheduler capacity or input policy rejected generation").await?;
                }
                Ok(())
            }
            WatchWorkerEvent::PendingCancellation { generation, stale } => self
                .notices
                .send(WatchNotice::PendingCancellation { generation, stale })
                .await
                .map_err(|_| Error::Internal("watch notice receiver closed".to_string())),
            WatchWorkerEvent::Settled {
                generation,
                outcomes,
                failed,
                source_generation_paths,
            } => self.on_settled(generation, outcomes, failed, source_generation_paths).await,
        }
    }

    async fn candidate_ready(
        &mut self,
        generation: u64,
        commit: tokio::sync::oneshot::Sender<()>,
    ) -> Result<(), Error> {
        debug_assert!(!self.rollback_pending || self.pending.is_some());
        debug_assert!(self.candidate.is_none() || !self.rollback_pending);
        if !self.rollback_pending
            && self.evaluation_cache.is_some()
            && (*self.shutdown.borrow() || cache_identity(self.config).ok() != self.pending_configuration)
        {
            drop(commit);
            self.pending_evaluation = None;
            if let Some(cache) = &mut self.evaluation_cache {
                cache.clear();
            }
            self.dirty = true;
            return Ok(());
        }
        if self.pending_evaluation.as_ref().is_some_and(|candidate| {
            candidate.validate(&evaluation_configuration(self.config), || *self.shutdown.borrow()).is_err()
        }) {
            drop(commit);
            self.pending_evaluation = None;
            if let Some(cache) = &mut self.evaluation_cache {
                cache.clear();
            }
            self.dirty = true;
            return Ok(());
        }
        // No Worker dispatch or PathInfo publication is authorized until this
        // candidate passes the source and generation gate.
        match authorize_candidate(
            self.config,
            &mut self.watch_loop,
            self.pending,
            self.candidate.as_ref(),
            self.rollback_pending,
            generation,
            commit,
            self.notices,
        )
        .await?
        {
            CandidateAuthorization::Authorized => {}
            CandidateAuthorization::Superseded => return Ok(()),
            CandidateAuthorization::Changed => {
                self.pending_evaluation = None;
                if let Some(cache) = &mut self.evaluation_cache {
                    cache.clear();
                }
                self.dirty = true;
                return Ok(());
            }
            CandidateAuthorization::Expired => {
                self.pending_evaluation = None;
                if let Some(cache) = &mut self.evaluation_cache {
                    cache.clear();
                }
                self.rollback_pending = false;
                self.dirty = true;
                return Ok(());
            }
        }
        if let Some(candidate) = self.pending_evaluation.take()
            && let Some(cache) = &mut self.evaluation_cache
            && cache.commit(candidate, &evaluation_configuration(self.config), || *self.shutdown.borrow()).is_err()
        {
            cache.clear();
            self.dirty = true;
        }
        if let Some(snapshot) = self.candidate.take() {
            self.baseline = Some(snapshot);
        }
        if !self.rollback_pending {
            self.baseline_configuration = self.pending_configuration.take();
        }
        self.committed_roots = std::mem::take(&mut self.pending_roots);
        let restoring = self.rollback_pending;
        self.rollback_pending = false;
        self.pending = None;
        self.current = Some(generation);
        self.current_is_rollback = restoring;
        if source_snapshot(&self.config.file, &self.config.import_paths).ok().as_ref() != self.baseline.as_ref() {
            self.dirty = true;
        }
        Ok(())
    }

    async fn on_settled(
        &mut self,
        generation: u64,
        outcomes: Vec<BuildOutcome>,
        failed: Vec<FailedGoal>,
        source_generation_paths: Vec<nix_compat::store_path::StorePath<String>>,
    ) -> Result<(), Error> {
        debug_assert!(!self.rollback_pending || self.pending.is_some());
        debug_assert!(self.candidate.is_none() || !self.rollback_pending);
        // Release retained workspace evidence for every terminal Worker
        // generation, including stale generations that publish no notice.
        drop(self.workspace_evidence_sink.take());
        if self.current != Some(generation) || self.pending.is_some() {
            return Ok(());
        }
        // No roots preceded a rejected first candidate; the internal empty
        // restoration has no result to publish.
        if self.current_is_rollback && self.committed_roots.is_empty() {
            return Ok(());
        }
        if !self.current_is_rollback
            && (source_snapshot(&self.config.file, &self.config.import_paths).ok().as_ref() != self.baseline.as_ref()
                || self.dirty
                || (self.evaluation_cache.is_some() && cache_identity(self.config).ok() != self.baseline_configuration))
        {
            self.dirty = true;
            return Ok(());
        }
        register_managed_generation(
            self.config.root_registration.as_ref(),
            self.config.root_retention_source,
            self.root_registry,
            &outcomes,
            &source_generation_paths,
            failed.is_empty(),
        )
        .await?;
        self.notices
            .send(WatchNotice::Settled {
                generation,
                outcomes,
                failed,
            })
            .await
            .map_err(|_| Error::Internal("watch notice receiver closed".to_string()))
    }

    async fn restore_pending(&mut self, diagnostic: &str) -> Result<(), Error> {
        debug_assert!(self.pending.is_some());
        debug_assert!(self.attempts.len() <= self.max_evaluations);
        let generation =
            restore_committed(&mut self.watch_loop, self.notices, &self.updates, &self.committed_roots, diagnostic)
                .await?;
        self.pending = Some(generation);
        self.pending_roots = self.committed_roots.clone();
        self.candidate = None;
        self.rollback_pending = true;
        self.dirty = true;
        self.pending_evaluation = None;
        if let Some(cache) = &mut self.evaluation_cache {
            cache.clear();
        }
        Ok(())
    }

    async fn on_tick(&mut self) -> Result<(), Error> {
        debug_assert!(!self.rollback_pending || self.pending.is_some());
        debug_assert!(self.attempts.len() <= self.max_evaluations);
        let observed_configuration = if self.evaluation_cache.is_some() {
            match cache_identity(self.config) {
                Ok(identity) => Some(identity),
                Err(_) => return self.on_evaluation_failure().await,
            }
        } else {
            None
        };
        if observed_configuration != self.pending_configuration.or(self.baseline_configuration) {
            self.dirty = true;
        }
        let observed = source_snapshot(&self.config.file, &self.config.import_paths);
        if observed.is_err() {
            self.pending_evaluation = None;
            if let Some(cache) = &mut self.evaluation_cache {
                cache.clear();
            }
        }
        if observed.as_ref().ok() != self.candidate.as_ref().or(self.baseline.as_ref()) {
            self.dirty = true;
        }
        if !self.dirty || (self.rollback_pending && observed.is_err()) {
            return Ok(());
        }
        // A failed edit while teardown is underway restores the last committed
        // roots instead of merely clearing the speculative candidate.
        if observed.is_err() && self.pending.is_some() {
            self.restore_pending("watched source scan, evaluation or conversion failed").await?;
            return Ok(());
        }
        let now = watch_clock_now();
        while self.attempts.front().is_some_and(|start| now.duration_since(*start) >= EVALUATION_WINDOW) {
            self.attempts.pop_front();
        }
        if self.attempts.len() >= self.max_evaluations {
            if !self.rate_rejection_reported {
                if self.pending.is_some() && !self.rollback_pending {
                    self.restore_pending("watched source evaluation rate limit reached").await?;
                } else if self.pending.is_none() {
                    reject(&mut self.watch_loop, self.notices, "watched source evaluation rate limit reached").await?;
                }
                self.rate_rejection_reported = true;
            }
            return Ok(());
        }
        self.rate_rejection_reported = false;
        let evaluations = u32::try_from(self.attempts.len())
            .map_err(|_| Error::Internal("watch evaluation count exceeds platform limit".to_string()))?;
        crunch_watch_core::admit_evaluation(evaluations).map_err(watch_error)?;
        self.attempts.push_back(now);
        let previous_labels = self.evaluation_cache.as_ref().map_or_else(Vec::new, |cache| cache.retained_labels());
        let result = if let Some(cache) = &mut self.evaluation_cache {
            evaluate_cached(self.config, cache, &self.shutdown).await
        } else {
            evaluate(self.config).await.map(|(goals, roots, snapshot)| (goals, roots, snapshot, None))
        };
        match result {
            Ok((goals, roots, snapshot, candidate)) => {
                if self.evaluation_cache.is_some() && cache_identity(self.config).ok() != observed_configuration {
                    return self.on_evaluation_failure().await;
                }
                if self.evaluation_cache.is_some() {
                    let (mode, diagnostics) = candidate.as_ref().map_or(
                        ("fresh-fallback-unsupported-or-capacity", ReuseDiagnostics {
                            executed_roots: Vec::new(),
                            reused_roots: Vec::new(),
                            invalidated_roots: Vec::new(),
                            evicted_roots: previous_labels.clone(),
                            retained_value_bytes: 0,
                        }),
                        |candidate| ("independent-import-roots", candidate.diagnostics().clone()),
                    );
                    self.notices
                        .send(WatchNotice::Evaluation { mode, diagnostics })
                        .await
                        .map_err(|_| Error::Internal("watch notice receiver closed".to_string()))?;
                }
                self.pending_evaluation = candidate;
                self.pending_configuration = observed_configuration;
                self.offer_evaluation(goals, roots, snapshot).await
            }
            Err(_) => {
                if self.evaluation_cache.is_some() {
                    self.notices
                        .send(WatchNotice::Evaluation {
                            mode: "rejected-cache-evicted",
                            diagnostics: ReuseDiagnostics {
                                executed_roots: Vec::new(),
                                reused_roots: Vec::new(),
                                invalidated_roots: Vec::new(),
                                evicted_roots: previous_labels,
                                retained_value_bytes: 0,
                            },
                        })
                        .await
                        .map_err(|_| Error::Internal("watch notice receiver closed".to_string()))?;
                }
                self.on_evaluation_failure().await
            }
        }
    }

    async fn offer_evaluation(
        &mut self,
        goals: GoalSet,
        roots: Vec<EvalMessage>,
        snapshot: SourceSnapshot,
    ) -> Result<(), Error> {
        debug_assert!(!self.rollback_pending || self.pending.is_some());
        debug_assert!(self.attempts.len() <= self.max_evaluations);
        match self.watch_loop.offer(goals, MAX_TRANSITION_EVENTS) {
            Ok(generation) => {
                self.candidate = Some(snapshot);
                // The Worker may prune stale derivation entries before a
                // rollback. Retain conversion metadata for each admitted root.
                self.pending_roots = roots.clone();
                self.dirty = false;
                self.rollback_pending = false;
                self.pending = Some(generation);
                self.updates
                    .send(WatchWorkerUpdate { generation, roots })
                    .await
                    .map_err(|_| Error::Internal("watch worker closed update channel".to_string()))?;
            }
            Err(_) if self.pending.is_some() => {
                self.restore_pending("watched goals exceed admission limits").await?;
            }
            Err(_) => {
                reject(&mut self.watch_loop, self.notices, "watched goals exceed admission limits").await?;
                self.dirty = true;
            }
        }
        Ok(())
    }

    async fn on_evaluation_failure(&mut self) -> Result<(), Error> {
        self.pending_evaluation = None;
        if let Some(cache) = &mut self.evaluation_cache {
            cache.clear();
        }
        if self.pending.is_some() && !self.rollback_pending {
            self.restore_pending("watched source scan, evaluation or conversion failed").await?;
        } else if !self.rollback_pending {
            reject(&mut self.watch_loop, self.notices, "watched source scan, evaluation or conversion failed").await?;
        }
        // Keep an in-flight restoration; retry after it is admitted.
        self.dirty = true;
        Ok(())
    }
}

// This std watch shell owns polling cadence; the deterministic watch core
// receives the resulting bounded evaluation count, never the ambient clock.
#[cfg(target_os = "linux")]
#[allow(
    tigerstyle::ambient_clock,
    reason = "watch shell samples the monotonic clock for its local rate-limit window"
)]
fn watch_clock_now() -> Instant {
    Instant::now()
}

#[cfg(target_os = "linux")]
async fn coordinate(
    config: &BuildConfig,
    watch_loop: WatchLoop,
    root_registry: &crunch_store::RootRegistry,
    workspace_evidence_sink: &crunch_build::WorkspaceReportCollector,
    shutdown: &mut watch::Receiver<bool>,
    notices: &mpsc::Sender<WatchNotice>,
    updates: mpsc::Sender<WatchWorkerUpdate>,
    mut worker_events: mpsc::Receiver<WatchWorkerEvent>,
    incremental: bool,
) -> Result<(), Error> {
    debug_assert!(EVALUATION_WINDOW > Duration::ZERO);
    let max_evaluations = usize::try_from(MAX_EVALUATIONS_PER_WINDOW)
        .map_err(|_| Error::Internal("watch evaluation window exceeds platform limit".to_string()))?;
    debug_assert!(max_evaluations > 0);
    let mut coordinator = WatchCoordinator {
        config,
        watch_loop,
        root_registry,
        workspace_evidence_sink,
        notices,
        updates,
        attempts: VecDeque::new(),
        max_evaluations,
        rate_rejection_reported: false,
        candidate: None,
        committed_roots: Vec::new(),
        pending_roots: Vec::new(),
        rollback_pending: false,
        baseline: None,
        pending: None,
        current_is_rollback: false,
        current: None,
        dirty: true,
        evaluation_cache: incremental.then(PersistentRootCache::default),
        pending_evaluation: None,
        pending_configuration: None,
        baseline_configuration: None,
        shutdown: shutdown.clone(),
    };
    let mut ticker = tokio::time::interval(POLL_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        if *shutdown.borrow() {
            break;
        }
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            _ = notices.closed() => {
                return Err(Error::Internal("watch notice receiver closed".to_string()));
            }
            event = worker_events.recv() => {
                let event = event.ok_or_else(|| Error::Internal("watch worker stopped before shutdown".to_string()))?;
                coordinator.on_event(event).await?;
            }
            _ = ticker.tick() => coordinator.on_tick().await?,
        }
    }
    drop(coordinator); // run_watch must cancel and drain sandbox/fetch tasks.
    // A timeout is not terminal: only Worker sender closure proves teardown.
    while worker_events.recv().await.is_some() {}
    Ok(())
}

/// Poll parsed Nickel imports and persist one store, builder, Worker and goal
/// registry until `shutdown` becomes true or its sender closes. `notices` must
/// be drained by the caller; a closed receiver terminates the watch with error.
/// A normal return occurs only after the Worker has drained all pending builds.
/// No one-shot build API or its output registration is changed by this opt-in.
#[cfg(target_os = "linux")]
async fn watch_build_configured(
    config: &BuildConfig,
    run_identity: String,
    shutdown: &mut watch::Receiver<bool>,
    notices: &mpsc::Sender<WatchNotice>,
    incremental: bool,
) -> Result<(), Error> {
    if config.rust_wait_runtime.is_some() {
        return Err(Error::Build("Rust wait diagnostics are unsupported for watch execution".into()));
    }
    if config.remote_enabled {
        return Err(Error::WatchRemoteUnsupported);
    }
    validate_build_config(config)?;
    // Validate caller-supplied identity before obtaining the mutation lock.
    let empty = GoalSet::admit(Vec::new()).map_err(watch_error)?;
    let watch_loop = WatchLoop::new(run_identity, empty).map_err(watch_error)?;
    let store_config = crunch_store::StoreConfig {
        backend: config.backend,
        state_dir: config.state_dir.clone(),
        output_dir: config.output_dir.clone(),
        remote_cache_urls: config.substituter_urls.clone(),
        fallback_mode: store_fallback_mode(config.hermeticity_mode),
        store_dir: config.store_dir.clone(),
        base_state_dirs: config.base_state_dirs.clone(),
        source_admission: config.source_admission.clone(),
    };
    store_config
        .preflight_backend_identity()
        .map_err(|error| Error::Internal(format!("opening watch store: {error}")))?;
    let mutation_guard = crunch_store::StoreMutationGuard::acquire_wait(&config.state_dir)
        .map_err(|error| Error::Internal(format!("acquiring watch store mutation lock: {error}")))?;
    let mut store = if config.base_state_dirs.is_empty() {
        crunch_store::StoreHandle::open(store_config).await
    } else {
        crunch_store::StoreHandle::open_overlay(store_config).await
    }
    .map_err(|error| Error::Internal(format!("opening watch store: {error}")))?;
    if config.backend == crunch_store::StoreBackend::Casita {
        store
            .recover_casita_gc_under_guard(&mutation_guard)
            .await
            .map_err(|error| Error::Internal(format!("recovering Casita GC before watch: {error}")))?;
    }
    let bundle = create_watch_pipeline_builder(config, store)?;
    let mut builder = bundle.builder;
    let root_registry = bundle.root_registry;
    let workspace_evidence_sink = bundle.workspace_evidence_sink;
    let mut known_paths = DerivationRegistry::new(&config.store_dir);
    let mut worker = Worker::with_scheduling_policy(config.max_jobs, config.scheduling_policy.clone())
        .map_err(|error| Error::Build(format!("scheduler policy: {error}")))?;
    let (updates_tx, mut updates_rx) = mpsc::channel(1);
    let (worker_events_tx, worker_events_rx) = mpsc::channel(WORKER_EVENT_CAPACITY);
    let worker_run = async {
        let result = worker.run_watch(&mut builder, &mut known_paths, &mut updates_rx, &worker_events_tx).await;
        drop(worker_events_tx);
        result
    };
    let (worker_result, coordinator_result) = tokio::join!(
        worker_run,
        coordinate(
            config,
            watch_loop,
            &root_registry,
            &workspace_evidence_sink,
            shutdown,
            notices,
            updates_tx,
            worker_events_rx,
            incremental,
        ),
    );
    // The Worker must be awaited even when evaluation/notices fail. Dropping
    // its future would discard the only authority for sandbox teardown.
    worker_result.map_err(|error| Error::Build(format!("watch worker: {error}")))?;
    coordinator_result
}

#[cfg(not(target_os = "linux"))]
async fn watch_build_configured(
    _: &BuildConfig,
    _: String,
    _: &mut watch::Receiver<bool>,
    _: &mpsc::Sender<WatchNotice>,
    _: bool,
) -> Result<(), Error> {
    Err(Error::Build("watch builds require Linux (bwrap)".to_string()))
}

/// Existing fresh watch path, unchanged by the opt-in root cache.
pub async fn watch_build(
    config: &BuildConfig,
    run_identity: String,
    shutdown: &mut watch::Receiver<bool>,
    notices: &mpsc::Sender<WatchNotice>,
) -> Result<(), Error> {
    watch_build_configured(config, run_identity, shutdown, notices, false).await
}

/// Retain only independent imported roots in owned process memory.
pub async fn watch_build_incremental(
    config: &BuildConfig,
    run_identity: String,
    shutdown: &mut watch::Receiver<bool>,
    notices: &mpsc::Sender<WatchNotice>,
) -> Result<(), Error> {
    watch_build_configured(config, run_identity, shutdown, notices, true).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_import_snapshot_tracks_import_edits_and_recovers_deleted_source() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.ncl");
        let child = dir.path().join("child.ncl");
        std::fs::write(&root, "{ result = import \"child.ncl\" }").unwrap();
        std::fs::write(&child, "1").unwrap();
        assert_eq!(crunch_eval::watch_imports::source_dependencies(&root, &[]).unwrap().len(), 2);
        let initial = source_snapshot(&root, &[]).unwrap();
        std::fs::write(&child, "2").unwrap();
        assert_ne!(initial, source_snapshot(&root, &[]).unwrap());
        std::fs::remove_file(&child).unwrap();
        assert!(source_snapshot(&root, &[]).is_err());
        std::fs::write(&child, "1").unwrap();
        assert_eq!(initial, source_snapshot(&root, &[]).unwrap());
        std::fs::remove_file(&root).unwrap();
        assert!(source_snapshot(&root, &[]).is_err());
    }

    fn test_config(dir: &tempfile::TempDir, file: PathBuf) -> BuildConfig {
        let keypair = crunch_build::load_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        ).unwrap();
        BuildConfig {
            file,
            import_paths: Vec::new(),
            output_dir: dir.path().to_path_buf(),
            state_dir: dir.path().join("unopened-state"),
            store_dir: nix_compat::store_path::STORE_DIR.to_string(),
            backend: crunch_store::StoreBackend::Snix,
            verbose: false,
            max_jobs: 1,
            scheduling_policy: crunch_build::SchedulingPolicy::default(),
            substituter_urls: Vec::new(),
            hermeticity_mode: crunch_build::HermeticityMode::Practical,
            base_state_dirs: Vec::new(),
            source_admission: None,
            trusted_keys: crunch_build::build_trusted_keys(&keypair, None),
            keypair,
            trust_unsigned: false,
            root_retention_source: None,
            root_registration: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
            compile_driver_runtime: None,
            rust_wait_runtime: None,
            interchange_dir: None,
        }
    }

    #[tokio::test]
    async fn empty_record_is_valid_retraction_candidate() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("roots.ncl");
        std::fs::write(&file, "{}").unwrap();
        let config = test_config(&dir, file);
        let (goals, roots, _) = evaluate(&config).await.unwrap();
        assert!(goals.identities().is_empty());
        assert!(roots.is_empty());
    }

    #[tokio::test]
    async fn changed_import_denies_candidate_until_committed_roots_are_restored() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("roots.ncl");
        let child = dir.path().join("child.ncl");
        std::fs::write(&root, "{ result = import \"child.ncl\" }").unwrap();
        std::fs::write(&child, "1").unwrap();
        let config = test_config(&dir, root);
        let before = source_snapshot(&config.file, &config.import_paths).unwrap();
        let empty = GoalSet::admit(Vec::new()).unwrap();
        let mut loop_state = WatchLoop::new("snapshot-gate".to_string(), empty).unwrap();
        let selected = GoalSet::admit(vec!["/nix/store/candidate.drv".to_string()]).unwrap();
        let candidate = loop_state.offer(selected, MAX_TRANSITION_EVENTS).unwrap();
        let (notices, mut notice_rx) = mpsc::channel(4);

        std::fs::write(&child, "2").unwrap();
        let (deny, denied) = tokio::sync::oneshot::channel();
        assert_eq!(
            authorize_candidate(
                &config,
                &mut loop_state,
                Some(candidate),
                Some(&before),
                false,
                candidate,
                deny,
                &notices,
            )
            .await
            .unwrap(),
            CandidateAuthorization::Changed
        );
        assert!(denied.await.is_err(), "changed imported source must not receive dispatch authority");
        assert!(notice_rx.try_recv().is_err(), "denied generation must not publish Goals");
        assert!(loop_state.admitted().identities().is_empty());
        std::fs::remove_file(&config.file).unwrap();
        let (missing, missing_result) = tokio::sync::oneshot::channel();
        assert_eq!(
            authorize_candidate(&config, &mut loop_state, Some(candidate), None, false, candidate, missing, &notices,)
                .await
                .unwrap(),
            CandidateAuthorization::Changed
        );
        assert!(missing_result.await.is_err(), "missing snapshot cannot authorize a missing source");
        assert!(notice_rx.try_recv().is_err());
        std::fs::write(&config.file, "{ result = import \"child.ncl\" }").unwrap();

        let (updates, mut update_rx) = mpsc::channel(1);
        let restore =
            restore_committed(&mut loop_state, &notices, &updates, &[], "watched source changed before admission")
                .await
                .unwrap();
        let update = update_rx.recv().await.unwrap();
        assert_eq!(update.generation, restore);
        assert!(update.roots.is_empty());
        assert!(matches!(notice_rx.recv().await.unwrap(), WatchNotice::Rejected { .. }));
        let (restore_token, restored) = tokio::sync::oneshot::channel();
        assert_eq!(
            authorize_candidate(&config, &mut loop_state, Some(restore), None, true, restore, restore_token, &notices,)
                .await
                .unwrap(),
            CandidateAuthorization::Authorized
        );
        assert!(restored.await.is_ok());
        assert!(notice_rx.try_recv().is_err(), "internal rollback is not a new Goals notice");

        let fresh = source_snapshot(&config.file, &config.import_paths).unwrap();
        let selected = GoalSet::admit(vec!["/nix/store/candidate.drv".to_string()]).unwrap();
        let generation = loop_state.offer(selected, MAX_TRANSITION_EVENTS).unwrap();
        let (approve, approved) = tokio::sync::oneshot::channel();
        assert_eq!(
            authorize_candidate(
                &config,
                &mut loop_state,
                Some(generation),
                Some(&fresh),
                false,
                generation,
                approve,
                &notices,
            )
            .await
            .unwrap(),
            CandidateAuthorization::Authorized
        );
        assert!(approved.await.is_ok());
        assert!(matches!(
            notice_rx.recv().await.unwrap(),
            WatchNotice::Goals { generation: received, .. } if received == generation
        ));

        let expired_goal = GoalSet::admit(vec!["/nix/store/expired.drv".to_string()]).unwrap();
        let expired_generation = loop_state.offer(expired_goal, MAX_TRANSITION_EVENTS).unwrap();
        let (token, receiver) = tokio::sync::oneshot::channel();
        drop(receiver);
        assert_eq!(
            authorize_candidate(
                &config,
                &mut loop_state,
                Some(expired_generation),
                Some(&fresh),
                false,
                expired_generation,
                token,
                &notices,
            )
            .await
            .unwrap(),
            CandidateAuthorization::Expired
        );
        assert!(loop_state.admitted().contains("/nix/store/candidate.drv"));
        assert!(!loop_state.admitted().contains("/nix/store/expired.drv"));
        assert!(loop_state.pending().is_none());
        assert!(notice_rx.try_recv().is_err(), "expired token must not publish Goals");
        let retry = loop_state
            .offer(GoalSet::admit(vec!["/nix/store/expired.drv".to_string()]).unwrap(), MAX_TRANSITION_EVENTS)
            .unwrap();
        assert!(retry > expired_generation, "rollback generation gaps remain monotonic");
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn remote_watch_is_rejected_before_store_or_source_access() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = test_config(&dir, dir.path().join("absent.ncl"));
        config.remote_enabled = true;
        config.output_dir = dir.path().join("unopened-output");
        config.max_jobs = 0;
        let (_shutdown_sender, mut shutdown) = watch::channel(false);
        let (notices, _receiver) = mpsc::channel(1);
        assert!(matches!(
            watch_build(&config, "remote-must-not-start".to_string(), &mut shutdown, &notices).await,
            Err(Error::WatchRemoteUnsupported)
        ));
        assert!(!config.output_dir.exists());
        assert!(!config.state_dir.exists());
    }
}
