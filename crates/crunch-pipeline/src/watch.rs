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
use crunch_evaluation_stream_core::RunDisposition;
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
const MAX_SNAPSHOT_BYTES: usize = 16 * 1024 * 1024;
const MAX_SNAPSHOT_FILE_BYTES: usize = 2 * 1024 * 1024;
const WORKER_EVENT_CAPACITY: usize = 16;

/// Admission notices describe the actual admitted goal set. A `Settled` notice
/// contains only the current Worker's terminal roots, not an evaluation result.
#[derive(Debug)]
pub enum WatchNotice {
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
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .and_then(|file| file.take((bound as u64) + 1).read_to_end(&mut bytes))
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
                if roots.len() <= MAX_LIVE_GOALS as usize {
                    roots.push(root);
                }
            }
            roots
        },
    );
    let result = result.map_err(|_| Error::Eval("watched root evaluation or conversion failed".to_string()))?;
    if result.summary.disposition() != RunDisposition::Success || roots.len() > MAX_LIVE_GOALS as usize {
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
async fn coordinate(
    config: &BuildConfig,
    mut watch_loop: WatchLoop,
    root_registry: &crunch_store::RootRegistry,
    workspace_reports: &crunch_build::WorkspaceReportCollector,
    shutdown: &mut watch::Receiver<bool>,
    notices: &mpsc::Sender<WatchNotice>,
    updates: mpsc::Sender<WatchWorkerUpdate>,
    mut worker_events: mpsc::Receiver<WatchWorkerEvent>,
) -> Result<(), Error> {
    let mut attempts = VecDeque::<Instant>::new();
    let mut candidate: Option<SourceSnapshot> = None;
    let mut committed_roots = Vec::<EvalMessage>::new();
    let mut pending_roots = Vec::<EvalMessage>::new();
    let mut rollback_pending = false;
    let mut baseline: Option<SourceSnapshot> = None;
    let mut pending: Option<u64> = None;
    let mut current_is_rollback = false;
    let mut current: Option<u64> = None;
    let mut dirty = true;
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
            event = worker_events.recv() => match event {
                Some(WatchWorkerEvent::CandidateReady { generation, commit }) => {
                    // No Worker dispatch or PathInfo publication is authorized
                    // until this candidate passes the source and generation gate.
                    match authorize_candidate(
                        config, &mut watch_loop, pending, candidate.as_ref(),
                        rollback_pending, generation, commit, notices,
                    ).await? {
                        CandidateAuthorization::Authorized => {}
                        CandidateAuthorization::Superseded => continue,
                        CandidateAuthorization::Changed => {
                            dirty = true;
                            continue;
                        }
                        CandidateAuthorization::Expired => {
                            rollback_pending = false;
                            dirty = true;
                            continue;
                        }
                    }
                    if let Some(snapshot) = candidate.take() {
                        baseline = Some(snapshot);
                    }
                    committed_roots = std::mem::take(&mut pending_roots);
                    let restoring = rollback_pending;
                    rollback_pending = false;
                    pending = None;
                    current = Some(generation);
                    current_is_rollback = restoring;
                    if source_snapshot(&config.file, &config.import_paths).ok().as_ref() != baseline.as_ref() {
                        dirty = true;
                    }
                }
                Some(WatchWorkerEvent::PendingCancellation { generation, stale }) => {
                    notices.send(WatchNotice::PendingCancellation { generation, stale }).await
                        .map_err(|_| Error::Internal("watch notice receiver closed".to_string()))?;
                }
                Some(WatchWorkerEvent::Settled { generation, outcomes, failed, source_generation_paths }) => {
                    // Watch notices intentionally contain terminal roots, not
                    // one-shot report vectors. Release retained workspace
                    // evidence for every terminal Worker generation.
                    drop(workspace_reports.take());
                    if current == Some(generation) && pending.is_none() {
                        // No roots preceded a rejected first candidate; the
                        // internal empty restoration has no result to publish.
                        if current_is_rollback && committed_roots.is_empty() {
                            continue;
                        }
                        if current_is_rollback
                            || (source_snapshot(&config.file, &config.import_paths).ok().as_ref() == baseline.as_ref() && !dirty)
                        {
                            register_managed_generation(
                                config.root_registration.as_ref(),
                                config.root_retention_source,
                                root_registry,
                                &outcomes,
                                &source_generation_paths,
                                failed.is_empty(),
                            ).await?;
                            notices.send(WatchNotice::Settled { generation, outcomes, failed }).await
                                .map_err(|_| Error::Internal("watch notice receiver closed".to_string()))?;
                        } else {
                            dirty = true;
                        }
                    }
                }
                None => return Err(Error::Internal("watch worker stopped before shutdown".to_string())),
            },
            _ = ticker.tick() => {
                let observed = source_snapshot(&config.file, &config.import_paths);
                if observed.as_ref().ok() != candidate.as_ref().or(baseline.as_ref()) {
                    dirty = true;
                }
                if !dirty || (rollback_pending && observed.is_err()) {
                    continue;
                }
                // A failed edit while teardown is underway must restore the
                // last committed roots, not merely clear the pure candidate.
                if observed.is_err() && pending.is_some() {
                    let generation = restore_committed(
                        &mut watch_loop, notices, &updates, &committed_roots,
                        "watched source scan, evaluation or conversion failed",
                    ).await?;
                    pending = Some(generation);
                    pending_roots = committed_roots.clone();
                    candidate = None;
                    rollback_pending = true;
                    continue;
                }
                let now = Instant::now();
                while attempts.front().is_some_and(|start| now.duration_since(*start) >= EVALUATION_WINDOW) {
                    attempts.pop_front();
                }
                if attempts.len() >= MAX_EVALUATIONS_PER_WINDOW as usize {
                    if pending.is_some() && !rollback_pending {
                        let generation = restore_committed(
                            &mut watch_loop, notices, &updates, &committed_roots,
                            "watched source evaluation rate limit reached",
                        ).await?;
                        pending = Some(generation);
                        pending_roots = committed_roots.clone();
                        candidate = None;
                        rollback_pending = true;
                    }
                    continue;
                }
                crunch_watch_core::admit_evaluation(attempts.len() as u32).map_err(watch_error)?;
                attempts.push_back(now);
                match evaluate(config).await {
                    Ok((goals, roots, snapshot)) => {
                        match watch_loop.offer(goals, MAX_TRANSITION_EVENTS) {
                            Ok(generation) => {
                                candidate = Some(snapshot);
                                // The Worker may prune stale derivation entries
                                // before a rollback. Keep complete conversion
                                // metadata for every last-admitted root.
                                pending_roots = roots.clone();
                                dirty = false;
                                rollback_pending = false;
                                pending = Some(generation);
                                updates.send(WatchWorkerUpdate { generation, roots }).await
                                    .map_err(|_| Error::Internal("watch worker closed update channel".to_string()))?;
                            }
                            Err(_) if pending.is_some() => {
                                let generation = restore_committed(
                                    &mut watch_loop, notices, &updates, &committed_roots,
                                    "watched goals exceed admission limits",
                                ).await?;
                                pending = Some(generation);
                                pending_roots = committed_roots.clone();
                                candidate = None;
                                rollback_pending = true;
                                dirty = true;
                            }
                            Err(_) => {
                                reject(&mut watch_loop, notices, "watched goals exceed admission limits").await?;
                                dirty = true;
                            }
                        }
                    }
                    Err(_) if pending.is_some() && !rollback_pending => {
                        let generation = restore_committed(
                            &mut watch_loop, notices, &updates, &committed_roots,
                            "watched source scan, evaluation or conversion failed",
                        ).await?;
                        pending = Some(generation);
                        pending_roots = committed_roots.clone();
                        candidate = None;
                        rollback_pending = true;
                        dirty = true;
                    }
                    Err(_) if rollback_pending => {
                        // Keep the in-flight restoration; retry after it is
                        // admitted instead of superseding it with an error.
                        dirty = true;
                    }
                    Err(_) => {
                        reject(&mut watch_loop, notices, "watched source scan, evaluation or conversion failed").await?;
                        dirty = true;
                    }
                }
            }
        }
    }
    drop(updates); // run_watch must cancel and drain sandbox/fetch tasks.
    // Keep draining the event receiver while the Worker cancels sandboxes.
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
pub async fn watch_build(
    config: &BuildConfig,
    run_identity: String,
    shutdown: &mut watch::Receiver<bool>,
    notices: &mpsc::Sender<WatchNotice>,
) -> Result<(), Error> {
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
    let workspace_reports = bundle.workspace_evidence_sink;
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
            &workspace_reports,
            shutdown,
            notices,
            updates_tx,
            worker_events_rx
        ),
    );
    // The Worker must be awaited even when evaluation/notices fail. Dropping
    // its future would discard the only authority for sandbox teardown.
    worker_result.map_err(|error| Error::Build(format!("watch worker: {error}")))?;
    coordinator_result
}

#[cfg(not(target_os = "linux"))]
pub async fn watch_build(
    _: &BuildConfig,
    _: String,
    _: &mut watch::Receiver<bool>,
    _: &mpsc::Sender<WatchNotice>,
) -> Result<(), Error> {
    Err(Error::Build("watch builds require Linux (bwrap)".to_string()))
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
            trusted_keys: crunch_build::build_trusted_keys(&keypair, None),
            keypair,
            trust_unsigned: false,
            root_retention_source: None,
            root_registration: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
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
