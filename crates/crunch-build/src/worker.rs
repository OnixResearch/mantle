//! Worker: imperative shell that drives Goal state machines.
//!
//! The Worker owns the `GoalRegistry` and orchestrates builds by:
//! 1. Creating goals lazily via `want()` — deduplicates by drv path
//! 2. Inspecting deps from `DerivationRegistry` to wire waiters
//! 3. Dispatching Ready goals through `Builder::prepare_build()`
//! 4. Spawning sandbox builds on a `JoinSet` with `Semaphore` concurrency
//! 5. Completing builds via `Builder::finish_build()`, notifying waiters
//! 6. Repeating until all root goals are terminal
//!
//! The Worker borrows `&mut Builder` for I/O — it never does I/O itself.
//! Goal state transitions are validated by the pure `Goal` API.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;

use crate::Error;
use crate::goal::Goal;
use crate::goal::GoalRegistry;
use crate::goal::GoalState;
use crate::goal::MAX_GOALS;
use crate::orchestrate::BuildOutcome;
use crate::orchestrate::Builder;
use crate::orchestrate::PrepareResult;
use crate::orchestrate::PreparedBuild;
use crate::registry::DerivationRegistry;

type PendingRegistryEntry = (
    StorePath<String>,
    [u8; 32],
    nix_compat::derivation::Derivation,
    bool,
    Option<crunch_attestation::Claims>,
);
type CreatedGoal = (String, Vec<StorePath<String>>);

/// A derivation arriving from the eval thread.
///
/// Contains the root drv_path plus all newly-converted entries
/// (the root and its transitive deps). The Worker inserts these
/// into `DerivationRegistry` before calling `want()`, enabling
/// true eval/build overlap: leaf deps start building while later
/// roots are still being converted.
#[derive(Debug, Clone)]
pub struct EvalMessage {
    /// Human-readable label (e.g., package name).
    pub label: String,
    /// The root derivation's store path.
    pub drv_path: StorePath<String>,
    /// Derivation entries discovered during this root's conversion.
    /// Each tuple: (drv_path, hash_derivation_modulo, Derivation, is_ca, provenance_claims).
    /// Inserted into `DerivationRegistry` before `want()` so deps
    /// are known. Diamond deps already in the registry are skipped
    /// (insert is idempotent by drv path).
    pub new_entries: Vec<PendingRegistryEntry>,
}

/// Maximum concurrent in-flight builds. Clamped by the semaphore but
/// tracked here for assertions.
const MAX_IN_FLIGHT: u32 = 64;

/// A root goal that failed, with error context.
#[derive(Debug, Clone)]
pub struct FailedGoal {
    /// Absolute drv-path key.
    pub drv_key: String,
    /// Human-readable error message.
    pub error: String,
}

/// Result of running the Worker: outcomes for root goals.
#[derive(Debug)]
pub struct WorkerResult {
    /// Build outcomes for root derivations that succeeded.
    pub outcomes: Vec<BuildOutcome>,
    /// Root goals that failed (build error or dep failure).
    pub failed: Vec<FailedGoal>,
}

/// Mutable loop state shared across worker helper methods.
///
/// Groups the recurring `&mut` references that flow through
/// dispatch, completion, and wait helpers. Keeps the method
/// signatures under the 5-parameter limit.
struct WorkerLoopState<'a> {
    sem: Arc<Semaphore>,
    join_set: JoinSet<(String, Result<snix_build::buildservice::BuildResult, Error>)>,
    pending_meta: HashMap<String, PreparedBuild>,
    outcomes: &'a mut Vec<BuildOutcome>,
    failed: &'a mut Vec<FailedGoal>,
    completed_count: u32,
}

/// Build scheduler that drives Goal state machines.
///
/// The Worker is the imperative shell. It holds mutable state
/// (goal registry, ready queue, in-flight tracking) and borrows
/// the Builder for all I/O. Goal is the functional core.
pub struct Worker {
    registry: GoalRegistry,
    ready_queue: VecDeque<String>,
    max_jobs: u32,
}

impl Worker {
    /// Create a new Worker with the given concurrency limit.
    pub fn new(max_jobs: u32) -> Self {
        debug_assert!(max_jobs >= 1, "max_jobs must be >= 1");
        debug_assert!(max_jobs <= MAX_IN_FLIGHT, "max_jobs exceeds MAX_IN_FLIGHT");

        Self {
            registry: GoalRegistry::new(),
            ready_queue: VecDeque::new(),
            max_jobs,
        }
    }

    /// Lazily create a goal for a derivation. If the goal already
    /// exists, this is a no-op. Inspects the derivation's
    /// `input_derivations` to discover deps, creating sub-goals
    /// and wiring waiters.
    ///
    /// `is_root`: whether this was explicitly requested by the user.
    ///
    /// Two-pass BFS: first creates all goals (so deps exist before
    /// wiring), then wires waiter edges and inspects states.
    /// Uses an explicit queue instead of recursion.
    pub fn want(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &DerivationRegistry,
        is_root: bool,
    ) -> Result<(), Error> {
        // Pass 1: BFS to create all goals. Collect (key, dep_paths)
        // for each newly created goal, in creation order.
        let created = self.create_goals_bfs(drv_path, known_paths, is_root)?;

        // Pass 2: Wire deps and inspect, in creation order (leaves
        // first since BFS processes deps before dependents).
        for (key, dep_drv_paths) in &created {
            self.wire_deps_and_inspect(key, dep_drv_paths)?;
        }

        Ok(())
    }

    /// BFS pass: create Goal structs for a root and all its transitive
    /// deps. Returns `(key, dep_paths)` for each newly created goal.
    fn create_goals_bfs(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &DerivationRegistry,
        is_root: bool,
    ) -> Result<Vec<CreatedGoal>, Error> {
        let mut queue: VecDeque<(StorePath<String>, bool)> = VecDeque::new();
        queue.push_back((drv_path.clone(), is_root));

        let mut created: Vec<CreatedGoal> = Vec::with_capacity(64);
        // Tiger Style: fixed iteration limit.
        let goal_count_max: u32 = MAX_GOALS;
        let mut iterations: u32 = 0;

        while let Some((sp, root)) = queue.pop_front() {
            iterations = iterations.saturating_add(1);
            if iterations > goal_count_max {
                return Err(Error::Store(format!("want() BFS exceeded iteration limit ({goal_count_max})")));
            }

            let key = sp.to_absolute_path();

            // Already tracked — just upgrade to root if needed.
            if let Some(goal) = self.registry.get_mut(&key) {
                if root {
                    goal.is_root = true;
                }
                continue;
            }

            // Look up derivation.
            let drv_abs = sp.to_absolute_path_with_prefix(known_paths.store_dir());
            let entry = known_paths
                .get_by_drv_path(&drv_abs)
                .ok_or_else(|| Error::DerivationNotFound { path: sp.clone() })?;
            let derivation = entry.derivation.clone();
            let dep_drv_paths: Vec<StorePath<String>> = derivation.input_derivations.keys().cloned().collect();

            let goal = if root {
                Goal::new_root(sp.clone(), derivation)
            } else {
                Goal::new(sp, derivation)
            };
            self.registry.insert(key.clone(), goal)?;

            // Enqueue deps for creation.
            for dep_sp in &dep_drv_paths {
                if !self.registry.contains(&dep_sp.to_absolute_path()) {
                    queue.push_back((dep_sp.clone(), false));
                }
            }

            created.push((key, dep_drv_paths));
        }

        debug_assert!(iterations <= goal_count_max);
        Ok(created)
    }

    /// Wire waiter edges for a goal's deps and inspect (Pending →
    /// Waiting/Ready). All deps must already exist in the registry.
    fn wire_deps_and_inspect(&mut self, key: &str, dep_drv_paths: &[StorePath<String>]) -> Result<(), Error> {
        let unbuilt_dep_keys: Vec<String> = dep_drv_paths
            .iter()
            .map(|sp| sp.to_absolute_path())
            .filter(|dep_key| self.registry.get(dep_key).map(|g| g.state != GoalState::Done).unwrap_or(false))
            .collect();

        for dep_key in &unbuilt_dep_keys {
            if let Some(dep_goal) = self.registry.get_mut(dep_key) {
                dep_goal.waiters.push(key.to_string());
            }
        }

        let goal = self
            .registry
            .get_mut(key)
            .ok_or_else(|| Error::Store(format!("worker: goal not found in registry: {key}")))?;
        let state = goal.inspect(unbuilt_dep_keys)?;

        if *state == GoalState::Ready {
            self.ready_queue.push_back(key.to_string());
        }

        Ok(())
    }

    /// Run the build loop until all root goals are terminal.
    ///
    /// Dispatches Ready goals through the Builder, spawns sandbox
    /// builds on a JoinSet, and processes completions. Cache hits
    /// and fetchers complete synchronously via `prepare_build`.
    pub async fn run<BServ>(
        &mut self,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<WorkerResult, Error>
    where
        BServ: BuildService + 'static,
    {
        let total_goals = self.registry.len();
        let root_count = self.registry.root_count();
        info!(goals = total_goals, roots = root_count, jobs = self.max_jobs, "worker starting");

        debug_assert!(root_count > 0, "no root goals to build");
        debug_assert!(total_goals <= MAX_GOALS, "goal count exceeds limit");

        let mut outcomes: Vec<BuildOutcome> = Vec::new();
        let mut failed: Vec<FailedGoal> = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(
                usize::try_from(self.max_jobs).map_err(|e| Error::Store(format!("max_jobs overflow: {e}")))?,
            )),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            failed: &mut failed,
            completed_count: 0,
        };
        let iteration_count_max: u32 = total_goals.saturating_mul(4).max(16);

        for _ in 0..iteration_count_max {
            let dispatched = self.dispatch_ready(builder, known_paths, &mut state).await?;

            if self.registry.all_roots_terminal() {
                info!(
                    completed = state.completed_count,
                    succeeded = state.outcomes.len(),
                    failed = state.failed.len(),
                    "worker finished"
                );
                return Ok(WorkerResult { outcomes, failed });
            }

            if state.join_set.is_empty() {
                if dispatched == 0 {
                    return Err(Error::Store("worker deadlock: no ready goals and no in-flight builds".into()));
                }
                continue;
            }

            let Some(join_result) = state.join_set.join_next().await else {
                continue;
            };
            self.process_join_result(join_result, builder, known_paths, &mut state).await?;
            state.completed_count = state.completed_count.saturating_add(1);
        }
        Err(Error::Store(format!("worker loop exceeded iteration limit ({iteration_count_max})")))
    }

    /// Run the build loop, receiving new roots from an eval channel.
    ///
    /// The eval thread converts derivations and sends `EvalMessage`s.
    /// The Worker calls `want()` for each arrival, interleaving new
    /// root discovery with build dispatch and completion. Builds
    /// start as soon as leaf derivations are ready — no waiting for
    /// all eval to finish.
    ///
    /// Terminates when: the channel is closed (eval done) AND all
    /// root goals are terminal.
    pub async fn run_streaming<BServ>(
        &mut self,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        rx: &mut mpsc::Receiver<EvalMessage>,
    ) -> Result<WorkerResult, Error>
    where
        BServ: BuildService + 'static,
    {
        let mut outcomes: Vec<BuildOutcome> = Vec::new();
        let mut failed: Vec<FailedGoal> = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(
                usize::try_from(self.max_jobs).map_err(|e| Error::Store(format!("max_jobs overflow: {e}")))?,
            )),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            failed: &mut failed,
            completed_count: 0,
        };
        let mut is_eval_done = false;
        let iteration_count_max: u32 = MAX_GOALS.saturating_mul(4);

        info!(jobs = self.max_jobs, "worker streaming started");

        for _ in 0..iteration_count_max {
            if !is_eval_done {
                self.drain_eval_messages(rx, known_paths, &mut is_eval_done)?;
            }

            self.dispatch_ready(builder, known_paths, &mut state).await?;

            if is_eval_done && self.registry.all_roots_terminal() {
                info!(
                    completed = state.completed_count,
                    succeeded = state.outcomes.len(),
                    failed = state.failed.len(),
                    roots = self.registry.root_count(),
                    "worker streaming finished"
                );
                return Ok(WorkerResult { outcomes, failed });
            }

            self.wait_for_event(&mut is_eval_done, rx, builder, known_paths, &mut state).await?;
        }
        Err(Error::Store(format!("worker loop exceeded iteration limit ({iteration_count_max})")))
    }

    /// Wait for either a build completion or an eval message.
    /// Centralized select! to avoid duplicating handle_build_completion.
    async fn wait_for_event<BServ>(
        &mut self,
        is_eval_done: &mut bool,
        rx: &mut mpsc::Receiver<EvalMessage>,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        if *is_eval_done {
            let Some(join_result) = state.join_set.join_next().await else {
                return Ok(());
            };
            self.process_join_result(join_result, builder, known_paths, state).await?;
            state.completed_count = state.completed_count.saturating_add(1);
        } else if state.join_set.is_empty() {
            match rx.recv().await {
                Some(msg) => self.accept_eval_message(msg, known_paths)?,
                None => *is_eval_done = true,
            }
        } else {
            tokio::select! {
                result = state.join_set.join_next() => {
                    let Some(join_result) = result else { return Ok(()); };
                    self.process_join_result(join_result, builder, known_paths, state).await?;
                    state.completed_count = state.completed_count.saturating_add(1);
                }
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => self.accept_eval_message(msg, known_paths)?,
                        None => *is_eval_done = true,
                    }
                }
            }
        }
        Ok(())
    }

    /// Drain buffered eval messages without blocking.
    /// Tiger Style: fixed limit prevents unbounded iteration.
    fn drain_eval_messages(
        &mut self,
        rx: &mut mpsc::Receiver<EvalMessage>,
        known_paths: &mut DerivationRegistry,
        is_eval_done: &mut bool,
    ) -> Result<(), Error> {
        let drain_count_max: u32 = 256;
        for _ in 0..drain_count_max {
            match rx.try_recv() {
                Ok(msg) => {
                    self.accept_eval_message(msg, known_paths)?;
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    *is_eval_done = true;
                    break;
                }
            }
        }
        Ok(())
    }

    /// Process a single eval message: insert new entries into the
    /// registry, then create a root goal.
    ///
    /// Entries arrive from the convert thread. Diamond deps that
    /// were already registered by an earlier message are silently
    /// skipped (DerivationRegistry.insert is idempotent by drv path).
    fn accept_eval_message(&mut self, msg: EvalMessage, known_paths: &mut DerivationRegistry) -> Result<(), Error> {
        debug!(
            label = %msg.label,
            drv = %msg.drv_path,
            new_entries = msg.new_entries.len(),
            "received derivation from eval",
        );

        for (drv_path, hdm, derivation, content_addressed, provenance_claims) in msg.new_entries {
            known_paths.insert(drv_path, hdm, derivation, content_addressed, provenance_claims);
        }

        self.want(&msg.drv_path, known_paths, true)
    }

    /// Process a JoinSet result: unwrap the join, then handle success
    /// or failure. Build errors are caught and routed to
    /// instead of aborting the entire Worker.
    async fn process_join_result<BServ>(
        &mut self,
        join_result: Result<(String, Result<snix_build::buildservice::BuildResult, Error>), tokio::task::JoinError>,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        match join_result {
            Ok((drv_key, Ok(build_result))) => {
                self.handle_build_completion(&drv_key, build_result, builder, known_paths, state).await
            }
            Ok((drv_key, Err(build_err))) => {
                let err_msg = format!("{build_err}");
                tracing::warn!(drv = %drv_key, err = %err_msg, "sandbox build failed");
                state.pending_meta.remove(&drv_key);
                self.fail_goal(&drv_key, &err_msg, state.failed)?;
                Ok(())
            }
            Err(join_err) => {
                tracing::error!(err = %join_err, "build task panicked");
                Err(Error::Store(format!("task join: {join_err}")))
            }
        }
    }

    /// Handle a completed sandbox build: finish_build, detect dynamic
    /// derivations, and notify waiters. Errors from finish_build are
    /// caught and routed to  for partial failure.
    async fn handle_build_completion<BServ>(
        &mut self,
        drv_key: &str,
        build_result: snix_build::buildservice::BuildResult,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let prepared = state
            .pending_meta
            .remove(drv_key)
            .ok_or_else(|| Error::Store(format!("BUG: completed build has no pending metadata: {drv_key}")))?;

        let outcome = match builder.finish_build(&prepared, build_result, known_paths).await {
            Ok(outcome) => outcome,
            Err(e) => {
                let err_msg = format!("{e}");
                tracing::warn!(drv = drv_key, err = %err_msg, "build failed, marking goal as failed");
                self.fail_goal(drv_key, &err_msg, state.failed)?;
                return Ok(());
            }
        };

        self.detect_dynamic_derivations(drv_key, &outcome, builder, known_paths).await?;

        self.complete_goal(drv_key, outcome, state.outcomes, state.failed)
    }

    /// Inspect build outputs for `.drv` files. If found, parse and
    /// register them as new goals (dynamic derivations).
    async fn detect_dynamic_derivations<BServ>(
        &mut self,
        producer_key: &str,
        outcome: &BuildOutcome,
        builder: &Builder<BServ>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let discovered = self.scan_dynamic_derivations(outcome, builder, known_paths).await?;
        if discovered.is_empty() {
            return Ok(());
        }
        self.activate_awaiting_dynamic_goals(producer_key, &discovered, known_paths)?;
        self.enqueue_discovered_dynamic_roots(&discovered, known_paths)?;
        Ok(())
    }

    async fn scan_dynamic_derivations<BServ>(
        &self,
        outcome: &BuildOutcome,
        builder: &Builder<BServ>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<Vec<crate::dynamic::DynamicDrv>, Error>
    where
        BServ: BuildService + 'static,
    {
        let mut discovered: Vec<crate::dynamic::DynamicDrv> = Vec::with_capacity(outcome.outputs.len());
        for (output_name, path_info) in &outcome.outputs {
            if !crate::dynamic::is_drv_output(&path_info.store_path, &path_info.node) {
                continue;
            }
            let content = builder.read_blob(&path_info.node).await?;
            if let Some(drv) = crate::dynamic::parse_drv_bytes(&content)? {
                let sd = known_paths.store_dir().to_string();
                let drv_path = crate::dynamic::register_dynamic_drv(&drv, known_paths, &sd)?;
                info!(
                    producer = %outcome.drv_path.name(),
                    dynamic_drv = %drv_path.name(),
                    output = %output_name,
                    "detected dynamic derivation in build output"
                );
                discovered.push(crate::dynamic::DynamicDrv {
                    output_name: output_name.clone(),
                    drv_store_path: drv_path,
                    derivation: drv,
                });
            }
        }
        Ok(discovered)
    }

    fn activate_awaiting_dynamic_goals(
        &mut self,
        producer_key: &str,
        discovered: &[crate::dynamic::DynamicDrv],
        known_paths: &DerivationRegistry,
    ) -> Result<(), Error> {
        let awaiting = self.registry.awaiting_producer(producer_key);
        let Some(dyn_drv) = discovered.first() else {
            return Ok(());
        };
        for awaiting_key in &awaiting {
            let goal = self
                .registry
                .get_mut(awaiting_key)
                .ok_or_else(|| Error::Store(format!("worker: awaiting goal not found: {awaiting_key}")))?;
            goal.set_derivation(dyn_drv.derivation.clone())?;
            let dep_drv_paths: Vec<StorePath<String>> = dyn_drv.derivation.input_derivations.keys().cloned().collect();
            for dep_sp in &dep_drv_paths {
                let dep_key = dep_sp.to_absolute_path();
                if !self.registry.contains(&dep_key) {
                    let dep_abs = dep_sp.to_absolute_path_with_prefix(known_paths.store_dir());
                    if let Some(entry) = known_paths.get_by_drv_path(&dep_abs) {
                        let dep_goal = crate::goal::Goal::new(dep_sp.clone(), entry.derivation.clone());
                        self.registry.insert(dep_key.clone(), dep_goal)?;
                    }
                }
            }
            self.wire_deps_and_inspect(awaiting_key, &dep_drv_paths)?;
        }
        Ok(())
    }

    fn enqueue_discovered_dynamic_roots(
        &mut self,
        discovered: &[crate::dynamic::DynamicDrv],
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        for dyn_drv in discovered {
            let key = dyn_drv.drv_store_path.to_absolute_path();
            if self.registry.contains(&key) {
                continue;
            }
            self.want(&dyn_drv.drv_store_path, known_paths, true)?;
        }
        Ok(())
    }

    /// Dispatch all Ready goals. Returns the number dispatched.
    ///
    /// For each Ready goal:
    /// - Call `prepare_build` (may resolve as cache hit or fetcher)
    /// - If cache/fetcher: complete immediately, notify waiters
    /// - If sandbox build: spawn on JoinSet with Semaphore
    async fn dispatch_ready<BServ>(
        &mut self,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<u32, Error>
    where
        BServ: BuildService + 'static,
    {
        let mut dispatched: u32 = 0;
        while let Some(drv_key) = self.ready_queue.pop_front() {
            let (drv_path, is_root, derivation) = self.ready_goal_inputs(&drv_key)?;
            let prepare_result = builder.prepare_build(&drv_path, derivation, known_paths, is_root).await;
            self.handle_prepare_result(&drv_key, prepare_result, builder, state)?;
            dispatched = dispatched.saturating_add(1);
        }
        Ok(dispatched)
    }

    fn ready_goal_inputs(
        &self,
        drv_key: &str,
    ) -> Result<(StorePath<String>, bool, Arc<nix_compat::derivation::Derivation>), Error> {
        let goal = self
            .registry
            .get(drv_key)
            .ok_or_else(|| Error::Store(format!("ready goal missing from registry: {drv_key}")))?;
        debug_assert_eq!(goal.state, GoalState::Ready, "goal in ready queue but state is {:?}", goal.state);
        let derivation = goal
            .derivation
            .as_ref()
            .ok_or_else(|| Error::Store(format!("goal {drv_key} in Ready state but has no derivation")))?
            .clone();
        Ok((goal.drv_path.clone(), goal.is_root, derivation))
    }

    fn handle_prepare_result<BServ>(
        &mut self,
        drv_key: &str,
        prepare_result: Result<PrepareResult, Error>,
        builder: &Builder<BServ>,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        match prepare_result {
            Ok(PrepareResult::Done(outcome)) => self.complete_goal(drv_key, outcome, state.outcomes, state.failed),
            Err(err) => {
                let err_msg = format!("{err}");
                tracing::warn!(drv = %drv_key, err = %err_msg, "prepare_build failed");
                self.fail_goal(drv_key, &err_msg, state.failed)
            }
            Ok(PrepareResult::NeedsBuild {
                prepared,
                build_request,
            }) => self.spawn_prepared_build(drv_key, prepared, *build_request, builder, state),
        }
    }

    fn spawn_prepared_build<BServ>(
        &mut self,
        drv_key: &str,
        prepared: PreparedBuild,
        build_request: snix_build::buildservice::BuildRequest,
        builder: &Builder<BServ>,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("worker: goal vanished during dispatch: {drv_key}")))?;
        goal.mark_building()?;

        let replaced = state.pending_meta.insert(drv_key.to_string(), prepared);
        debug_assert!(replaced.is_none(), "pending_meta already had entry for {drv_key}");

        let bs = builder.build_service();
        let sem = state.sem.clone();
        let key = drv_key.to_string();
        state.join_set.spawn(async move {
            let _permit = sem.acquire_owned().await.map_err(|e| Error::Store(format!("semaphore: {e}")));
            let build_result = match _permit {
                Ok(_p) => bs.do_build(build_request).await.map_err(|e| Error::Store(format!("build: {e}"))),
                Err(e) => Err(e),
            };
            (key, build_result)
        });
        Ok(())
    }

    /// Mark a goal as Done, collect its outcome, and notify waiters.
    /// If a waiter becomes Ready, push it to the ready queue.
    fn complete_goal(
        &mut self,
        drv_key: &str,
        outcome: BuildOutcome,
        outcomes: &mut Vec<BuildOutcome>,
        _failed: &mut Vec<FailedGoal>,
    ) -> Result<(), Error> {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("completing unknown goal: {drv_key}")))?;

        // Transition: Ready or Building → Done.
        // Cache hits are still in Ready state. Sandbox builds are in Building.
        match goal.state {
            GoalState::Ready => {
                // Cache hit / fetcher — skip Building, go directly to Done.
                goal.mark_building()?;
                goal.mark_done()?;
            }
            GoalState::Building => {
                goal.mark_done()?;
            }
            _ => {
                return Err(Error::Store(format!("completing goal {drv_key} in {:?} state", goal.state)));
            }
        }

        let is_root = goal.is_root;
        let waiters = std::mem::take(&mut goal.waiters);

        if is_root {
            outcomes.push(outcome);
        }

        debug!(drv = drv_key, waiters = waiters.len(), "goal completed");

        // Notify waiters.
        for waiter_key in &waiters {
            let waiter = self
                .registry
                .get_mut(waiter_key)
                .ok_or_else(|| Error::Store(format!("waiter goal missing: {waiter_key}")))?;

            let is_now_ready = waiter.notify_dep_done()?;
            if is_now_ready {
                self.ready_queue.push_back(waiter_key.clone());
            }
        }

        Ok(())
    }

    /// Mark a goal as failed and propagate failure to all waiters.
    ///
    /// The error message is stored in the  for root goals
    /// so callers can report per-package errors.
    fn fail_goal(&mut self, drv_key: &str, error_msg: &str, failed: &mut Vec<FailedGoal>) -> Result<(), Error> {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("failing unknown goal: {drv_key}")))?;

        goal.mark_build_failed()?;

        let is_root = goal.is_root;
        let waiters = std::mem::take(&mut goal.waiters);
        let drv_name = goal.drv_path.name().to_string();

        if is_root {
            failed.push(FailedGoal {
                drv_key: drv_key.to_string(),
                error: error_msg.to_string(),
            });
        }

        info!(
            drv = drv_key,
            name = %drv_name,
            waiters = waiters.len(),
            "goal failed"
        );

        // Propagate failure to waiters.
        for waiter_key in &waiters {
            self.propagate_failure(waiter_key, &drv_name, failed)?;
        }

        Ok(())
    }

    /// Recursively propagate dep failure to waiting goals.
    fn propagate_failure(
        &mut self,
        drv_key: &str,
        failed_dep_name: &str,
        failed: &mut Vec<FailedGoal>,
    ) -> Result<(), Error> {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("propagating failure to unknown goal: {drv_key}")))?;

        // Only propagate to goals still Waiting or AwaitingDerivation.
        if !matches!(goal.state, GoalState::Waiting { .. } | GoalState::AwaitingDerivation) {
            return Ok(());
        }

        goal.notify_dep_failed()?;

        let is_root = goal.is_root;
        let waiters = std::mem::take(&mut goal.waiters);
        let drv_name = goal.drv_path.name().to_string();

        if is_root {
            failed.push(FailedGoal {
                drv_key: drv_key.to_string(),
                error: format!("dependency {failed_dep_name} failed"),
            });
        }

        for waiter_key in &waiters {
            self.propagate_failure(waiter_key, &drv_name, failed)?;
        }

        Ok(())
    }

    /// Read-only access to the goal registry (for testing/inspection).
    pub fn registry(&self) -> &GoalRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use nix_compat::derivation::Derivation;
    use nix_compat::derivation::Output;

    use super::*;
    use crate::goal::GoalState;

    fn make_drv() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        Derivation {
            arguments: vec![],
            builder: "/bin/sh".to_string(),
            environment: BTreeMap::new(),
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        }
    }

    fn make_drv_with_deps(deps: &[StorePath<String>]) -> Derivation {
        let mut drv = make_drv();
        for dep in deps {
            drv.input_derivations.insert(dep.clone(), BTreeSet::from(["out".into()]));
        }
        drv
    }

    fn fake_sp(name: &str) -> StorePath<String> {
        let mut digest = [0u8; 20];
        for (i, b) in name.bytes().enumerate() {
            digest[i % 20] ^= b;
        }
        StorePath::from_name_and_digest_fixed(name, digest).unwrap()
    }

    fn register_drv(kp: &mut DerivationRegistry, name: &str, drv: &Derivation) -> StorePath<String> {
        let sp = fake_sp(name);
        let aterm = drv.to_aterm_bytes();
        let aterm_hash = {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&aterm);
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        };
        let hdm = aterm_hash; // Use same for simplicity in tests.
        kp.insert(sp.clone(), hdm, drv.clone(), false, None);
        sp
    }

    // ── want() tests ────────────────────────────────────────────

    #[test]
    fn want_single_leaf() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "leaf.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();

        assert_eq!(w.registry.len(), 1);
        let goal = w.registry.get(&sp.to_absolute_path()).unwrap();
        assert_eq!(goal.state, GoalState::Ready);
        assert!(goal.is_root);
        assert_eq!(w.ready_queue.len(), 1);
    }

    #[test]
    fn want_chain_creates_all_goals() {
        let mut kp = DerivationRegistry::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let mid_drv = make_drv_with_deps(std::slice::from_ref(&leaf_sp));
        let mid_sp = register_drv(&mut kp, "mid.drv", &mid_drv);

        let top_drv = make_drv_with_deps(std::slice::from_ref(&mid_sp));
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        assert_eq!(w.registry.len(), 3);

        // Leaf is Ready (no deps).
        let leaf = w.registry.get(&leaf_sp.to_absolute_path()).unwrap();
        assert_eq!(leaf.state, GoalState::Ready);

        // Mid is Waiting (dep on leaf).
        let mid = w.registry.get(&mid_sp.to_absolute_path()).unwrap();
        assert_eq!(mid.state, GoalState::Waiting { remaining_deps: 1 });

        // Top is Waiting (dep on mid).
        let top = w.registry.get(&top_sp.to_absolute_path()).unwrap();
        assert_eq!(top.state, GoalState::Waiting { remaining_deps: 1 });
        assert!(top.is_root);

        // Only leaf is in ready queue.
        assert_eq!(w.ready_queue.len(), 1);
    }

    #[test]
    fn want_diamond_deduplicates() {
        let mut kp = DerivationRegistry::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let left_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let left_sp = register_drv(&mut kp, "left.drv", &left_drv);

        let right_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let right_sp = register_drv(&mut kp, "right.drv", &right_drv);

        let top_drv = make_drv_with_deps(&[left_sp.clone(), right_sp.clone()]);
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(2);
        w.want(&top_sp, &kp, true).unwrap();

        // 4 goals, not 5 — shared is deduped.
        assert_eq!(w.registry.len(), 4);

        // Shared has two waiters: left and right.
        let shared = w.registry.get(&shared_sp.to_absolute_path()).unwrap();
        assert_eq!(shared.waiters.len(), 2);
        assert_eq!(shared.state, GoalState::Ready);
    }

    #[test]
    fn want_duplicate_is_noop() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "leaf.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, false).unwrap();
        w.want(&sp, &kp, false).unwrap();

        assert_eq!(w.registry.len(), 1);
    }

    #[test]
    fn want_upgrades_to_root() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "leaf.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, false).unwrap();

        let g = w.registry.get(&sp.to_absolute_path()).unwrap();
        assert!(!g.is_root);

        w.want(&sp, &kp, true).unwrap();

        let g = w.registry.get(&sp.to_absolute_path()).unwrap();
        assert!(g.is_root);
    }

    #[test]
    fn want_unknown_drv_errors() {
        let kp = DerivationRegistry::default();
        let sp = fake_sp("unknown.drv");

        let mut w = Worker::new(1);
        assert!(w.want(&sp, &kp, true).is_err());
    }

    #[test]
    fn want_disjoint_trees() {
        let mut kp = DerivationRegistry::default();

        let a_drv = make_drv();
        let a_sp = register_drv(&mut kp, "a.drv", &a_drv);

        let b_drv = make_drv();
        let b_sp = register_drv(&mut kp, "b.drv", &b_drv);

        let mut w = Worker::new(2);
        w.want(&a_sp, &kp, true).unwrap();
        w.want(&b_sp, &kp, true).unwrap();

        assert_eq!(w.registry.len(), 2);
        assert_eq!(w.ready_queue.len(), 2);
        assert_eq!(w.registry.root_count(), 2);
    }

    // ── complete_goal / waiter notification ────────────────────

    #[test]
    fn complete_goal_notifies_waiters() {
        let mut kp = DerivationRegistry::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let top_drv = make_drv_with_deps(std::slice::from_ref(&leaf_sp));
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        // Leaf is Ready, top is Waiting.
        let leaf_key = leaf_sp.to_absolute_path();
        let top_key = top_sp.to_absolute_path();
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Ready);
        assert!(matches!(w.registry.get(&top_key).unwrap().state, GoalState::Waiting { remaining_deps: 1 }));

        // Simulate completing the leaf.
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: leaf_sp.clone(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::new(),
            cached: false,
            log: None,
        };
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&leaf_key, outcome, &mut outcomes, &mut failed).unwrap();

        // Leaf should be Done.
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Done);
        assert!(w.registry.get(&leaf_key).unwrap().waiters.is_empty());
        // Top should now be Ready (its sole dep completed).
        assert_eq!(w.registry.get(&top_key).unwrap().state, GoalState::Ready);
        // Top should be in the ready queue.
        assert!(w.ready_queue.contains(&top_key));
        // Leaf is not a root, so outcomes should be empty.
        assert!(outcomes.is_empty());
    }

    #[test]
    fn complete_goal_collects_root_outcome() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "root.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();

        let key = sp.to_absolute_path();
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: sp.clone(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::new(),
            cached: true,
            log: None,
        };
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&key, outcome, &mut outcomes, &mut failed).unwrap();

        assert_eq!(outcomes.len(), 1);
        assert!(outcomes[0].cached);
    }

    #[test]
    fn ready_goal_inputs_reuses_shared_derivation_arc() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "root.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();

        let key = sp.to_absolute_path();
        let registry_arc = kp.get_by_drv_path(&key).unwrap().derivation.clone();
        let goal_arc = w.registry.get(&key).unwrap().derivation.as_ref().unwrap().clone();
        let (_, _, ready_arc) = w.ready_goal_inputs(&key).unwrap();

        assert!(std::sync::Arc::ptr_eq(&registry_arc, &goal_arc));
        assert!(std::sync::Arc::ptr_eq(&goal_arc, &ready_arc));
    }

    #[test]
    fn complete_diamond_shared_dep_unblocks_both() {
        let mut kp = DerivationRegistry::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let left_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let left_sp = register_drv(&mut kp, "left.drv", &left_drv);

        let right_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let right_sp = register_drv(&mut kp, "right.drv", &right_drv);

        let top_drv = make_drv_with_deps(&[left_sp.clone(), right_sp.clone()]);
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(2);
        w.want(&top_sp, &kp, true).unwrap();

        // Complete shared dep.
        let shared_key = shared_sp.to_absolute_path();
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: shared_sp.clone(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::new(),
            cached: false,
            log: None,
        };
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&shared_key, outcome, &mut outcomes, &mut failed).unwrap();

        // Both left and right should now be Ready.
        let left_key = left_sp.to_absolute_path();
        let right_key = right_sp.to_absolute_path();
        assert_eq!(w.registry.get(&left_key).unwrap().state, GoalState::Ready);
        assert_eq!(w.registry.get(&right_key).unwrap().state, GoalState::Ready);
        // Top still Waiting (needs left + right).
        let top_key = top_sp.to_absolute_path();
        assert!(matches!(w.registry.get(&top_key).unwrap().state, GoalState::Waiting { remaining_deps: 2 }));
    }

    // ── fail_goal / propagate_failure ─────────────────────────

    #[test]
    fn fail_goal_propagates_to_waiters() {
        let mut kp = DerivationRegistry::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let mid_drv = make_drv_with_deps(std::slice::from_ref(&leaf_sp));
        let mid_sp = register_drv(&mut kp, "mid.drv", &mid_drv);

        let top_drv = make_drv_with_deps(std::slice::from_ref(&mid_sp));
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        // Leaf is Ready — simulate it being dispatched and starting.
        let leaf_key = leaf_sp.to_absolute_path();
        w.ready_queue.pop_front(); // remove leaf from ready queue
        w.registry.get_mut(&leaf_key).unwrap().mark_building().unwrap();

        // Fail the leaf.
        let mut failed = Vec::new();
        w.fail_goal(&leaf_key, "leaf build error", &mut failed).unwrap();

        // Leaf should be Failed.
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Failed);
        assert!(w.registry.get(&leaf_key).unwrap().waiters.is_empty());
        // Mid should be Failed (dep failed).
        let mid_key = mid_sp.to_absolute_path();
        assert_eq!(w.registry.get(&mid_key).unwrap().state, GoalState::Failed);
        assert!(w.registry.get(&mid_key).unwrap().waiters.is_empty());
        // Top should be Failed (transitive dep failed).
        let top_key = top_sp.to_absolute_path();
        assert_eq!(w.registry.get(&top_key).unwrap().state, GoalState::Failed);
        // Top is a root, so it should be in the failed list.
        assert!(failed.iter().any(|f| f.drv_key == top_key));
    }

    #[test]
    fn fail_goal_only_propagates_to_waiting() {
        let mut kp = DerivationRegistry::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let good_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let good_sp = register_drv(&mut kp, "good.drv", &good_drv);

        let bad_drv = make_drv();
        let bad_sp = register_drv(&mut kp, "bad.drv", &bad_drv);

        let mut w = Worker::new(2);
        w.want(&good_sp, &kp, true).unwrap();
        w.want(&bad_sp, &kp, true).unwrap();

        // bad is Ready, shared is Ready, good is Waiting.
        let bad_key = bad_sp.to_absolute_path();
        w.ready_queue.retain(|k| k != &bad_key); // remove bad from ready queue
        w.registry.get_mut(&bad_key).unwrap().mark_building().unwrap();

        // Fail bad. It has no waiters, so good (waiting on shared) should be unaffected.
        let mut failed = Vec::new();
        w.fail_goal(&bad_key, "bad build error", &mut failed).unwrap();

        assert_eq!(w.registry.get(&bad_key).unwrap().state, GoalState::Failed);
        // good is still Waiting on shared, not failed.
        let good_key = good_sp.to_absolute_path();
        assert!(matches!(w.registry.get(&good_key).unwrap().state, GoalState::Waiting { .. }));
        // bad is a root, so it should be in failed list.
        assert!(failed.iter().any(|f| f.drv_key == bad_key));
    }

    // ── run() integration tests ───────────────────────────────

    use std::path::PathBuf;

    use snix_castore::blobservice::MemoryBlobService;

    use crate::orchestrate::Builder;
    use crate::test_support::MockBuildService;
    use crate::test_support::build_and_register;
    use crate::test_support::test_keypair;
    use crate::test_support::test_pis;
    use crate::test_support::test_trusted_keys;
    use crate::test_support::tmp_ds;

    #[tokio::test]
    async fn run_single_leaf_builds_once() {
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
        let (sp, _) = build_and_register("solo", &[], &mut kp);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        assert!(result.failed.is_empty());
        assert!(!result.outcomes[0].cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn run_chain_builds_in_order() {
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
        let (mid_path, _) = build_and_register("mid", &[(dep_path.clone(), "out")], &mut kp);
        let (top_path, _) = build_and_register("top", &[(mid_path.clone(), "out")], &mut kp);

        let mut w = Worker::new(1);
        w.want(&top_path, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        assert!(result.outcomes[0].drv_path.name().contains("top"));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3);

        // Verify ordering: dep before mid before top.
        let pos_dep = recorded.iter().position(|a| a.iter().any(|s| s.contains("dep"))).unwrap();
        let pos_mid = recorded.iter().position(|a| a.iter().any(|s| s.contains("mid"))).unwrap();
        let pos_top = recorded.iter().position(|a| a.iter().any(|s| s.contains("top"))).unwrap();
        assert!(pos_dep < pos_mid, "dep before mid");
        assert!(pos_mid < pos_top, "mid before top");
    }

    #[tokio::test]
    async fn run_diamond_builds_shared_once() {
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
        let (shared, _) = build_and_register("shared", &[], &mut kp);
        let (left, _) = build_and_register("left", &[(shared.clone(), "out")], &mut kp);
        let (right, _) = build_and_register("right", &[(shared.clone(), "out")], &mut kp);

        let mut w = Worker::new(2);
        w.want(&left, &kp, true).unwrap();
        w.want(&right, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3, "shared + left + right");
        let shared_builds = recorded.iter().filter(|a| a.iter().any(|s| s.contains("shared"))).count();
        assert_eq!(shared_builds, 1, "shared built exactly once");
    }

    #[tokio::test]
    async fn run_disjoint_trees() {
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
        let (a, _) = build_and_register("tree1-leaf", &[], &mut kp);
        let (b, _) = build_and_register("tree1-root", &[(a.clone(), "out")], &mut kp);
        let (c, _) = build_and_register("tree2-leaf", &[], &mut kp);
        let (d, _) = build_and_register("tree2-root", &[(c.clone(), "out")], &mut kp);

        let mut w = Worker::new(2);
        w.want(&b, &kp, true).unwrap();
        w.want(&d, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 4);

        // Each leaf before its root.
        let pos_a = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree1-leaf"))).unwrap();
        let pos_b = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree1-root"))).unwrap();
        let pos_c = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree2-leaf"))).unwrap();
        let pos_d = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree2-root"))).unwrap();
        assert!(pos_a < pos_b, "tree1-leaf before tree1-root");
        assert!(pos_c < pos_d, "tree2-leaf before tree2-root");
    }

    #[tokio::test]
    #[should_panic(expected = "no root goals to build")]
    async fn run_empty_worker_panics() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _) = MockBuildService::new(bs.clone());

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
        let mut w = Worker::new(1);

        // No roots — debug_assert catches this.
        let _ = w.run(&mut builder, &mut kp).await;
    }

    // ── run_streaming() tests ───────────────────────────────

    #[tokio::test]
    async fn streaming_single_root() {
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
        let (sp, _) = build_and_register("solo", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "solo".into(),
            drv_path: sp.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx); // Close channel — eval done.

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        assert!(result.failed.is_empty());

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn streaming_multiple_roots_arrive_incrementally() {
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
        let (a, _) = build_and_register("pkg-a", &[], &mut kp);
        let (b, _) = build_and_register("pkg-b", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);

        // Send first root.
        tx.send(EvalMessage {
            label: "pkg-a".into(),
            drv_path: a.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        // Send second root.
        tx.send(EvalMessage {
            label: "pkg-b".into(),
            drv_path: b.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2);
    }

    #[tokio::test]
    async fn streaming_chain_with_shared_deps() {
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
        let (shared, _) = build_and_register("shared", &[], &mut kp);
        let (top_a, _) = build_and_register("top-a", &[(shared.clone(), "out")], &mut kp);
        let (top_b, _) = build_and_register("top-b", &[(shared.clone(), "out")], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "top-a".into(),
            drv_path: top_a.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        tx.send(EvalMessage {
            label: "top-b".into(),
            drv_path: top_b.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        // shared + top-a + top-b = 3 builds (shared built once).
        assert_eq!(recorded.len(), 3);
        let shared_builds = recorded.iter().filter(|a| a.iter().any(|s| s.contains("shared"))).count();
        assert_eq!(shared_builds, 1);
    }

    #[tokio::test]
    async fn streaming_empty_channel_returns_empty() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _) = MockBuildService::new(bs.clone());

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
        let (_tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        drop(_tx); // Immediately close.

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert!(result.outcomes.is_empty());
        assert!(result.failed.is_empty());
    }

    #[tokio::test]
    async fn streaming_builds_start_before_channel_closes() {
        // Verify that builds begin while eval is still sending.
        // We send one root, let it build, then send another.
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
        let (a, _) = build_and_register("first", &[], &mut kp);
        let (b, _) = build_and_register("second", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);

        // Send first root only.
        tx.send(EvalMessage {
            label: "first".into(),
            drv_path: a.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();

        // Spawn a task that sends second root after a short delay
        // (simulating slow eval).
        let tx2 = tx.clone();
        let b2 = b.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            tx2.send(EvalMessage {
                label: "second".into(),
                drv_path: b2,
                new_entries: vec![],
            })
            .await
            .unwrap();
            // Drop tx2 but tx is still alive — don't close channel yet.
        });

        // Drop the original sender after a bit more delay.
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            drop(tx);
        });

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2);
    }

    // ── Entries-in-message tests ─────────────────────────────
    // These verify the eval/build streaming overlap: registry
    // entries arrive WITH the EvalMessage and are inserted by
    // accept_eval_message before want() is called.

    #[tokio::test]
    async fn streaming_entries_populate_registry_on_arrival() {
        // Registry starts EMPTY. Entries arrive via EvalMessage.
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

        // Build the derivation entries outside the registry.
        let mut scratch_kp = DerivationRegistry::default();
        let (sp, drv) = build_and_register("via-msg", &[], &mut scratch_kp);
        let hdm = scratch_kp.get_hdm_by_drv_path(&sp.to_absolute_path()).unwrap();

        // Empty registry — the Worker has no knowledge of "via-msg" yet.
        let mut kp = DerivationRegistry::default();

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "via-msg".into(),
            drv_path: sp.clone(),
            new_entries: vec![(sp.clone(), hdm, drv, false, None)],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        // Registry was populated by the message.
        assert!(!kp.is_empty(), "registry should have entries from message");
        assert_eq!(result.outcomes.len(), 1);
        assert!(result.failed.is_empty());

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn streaming_entries_with_deps_arrive_incrementally() {
        // Two roots with a shared dep. First message carries
        // [shared, root_a]; second carries [root_b] (shared already
        // in registry from first message).
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

        // Build derivation entries in a scratch registry.
        let mut scratch = DerivationRegistry::default();
        let (shared_sp, shared_drv) = build_and_register("shared", &[], &mut scratch);
        let (a_sp, a_drv) = build_and_register("root-a", &[(shared_sp.clone(), "out")], &mut scratch);
        let (b_sp, b_drv) = build_and_register("root-b", &[(shared_sp.clone(), "out")], &mut scratch);

        let shared_hdm = scratch.get_hdm_by_drv_path(&shared_sp.to_absolute_path()).unwrap();
        let a_hdm = scratch.get_hdm_by_drv_path(&a_sp.to_absolute_path()).unwrap();
        let b_hdm = scratch.get_hdm_by_drv_path(&b_sp.to_absolute_path()).unwrap();

        let mut kp = DerivationRegistry::default();

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        // First root brings shared + root-a.
        tx.send(EvalMessage {
            label: "root-a".into(),
            drv_path: a_sp.clone(),
            new_entries: vec![
                (shared_sp.clone(), shared_hdm, shared_drv, false, None),
                (a_sp.clone(), a_hdm, a_drv, false, None),
            ],
        })
        .await
        .unwrap();
        // Second root only brings root-b (shared already known).
        tx.send(EvalMessage {
            label: "root-b".into(),
            drv_path: b_sp.clone(),
            new_entries: vec![(b_sp.clone(), b_hdm, b_drv, false, None)],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);
        assert!(result.failed.is_empty());

        // shared + root-a + root-b = 3 builds, shared only once.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3);
        let shared_builds = recorded.iter().filter(|a| a.iter().any(|s| s.contains("shared"))).count();
        assert_eq!(shared_builds, 1);
    }

    // ── Dynamic derivation integration tests ─────────────────

    use std::collections::HashMap as StdHashMap;

    use crate::test_support::DrvProducingMockBuildService;

    /// Build a derivation that produces `.drv` ATerm output.
    /// The Worker should detect it, parse the inner .drv, register
    /// it, and build it automatically.
    #[tokio::test]
    async fn dynamic_drv_detected_and_built() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // Create the "inner" derivation that will be discovered dynamically.
        let mut inner_kp = DerivationRegistry::default();
        let (_, inner_drv) = build_and_register("inner-hello", &[], &mut inner_kp);
        let inner_aterm = inner_drv.to_aterm_bytes();

        // The producer build will output this ATerm content.
        let mut drv_outputs = StdHashMap::new();
        // Match on the producer name substring.
        drv_outputs.insert("producer-gen.drv".to_string(), inner_aterm.clone());

        let (mock, calls) = DrvProducingMockBuildService::new(bs.clone(), drv_outputs);

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

        // Register the producer derivation. Its output path name must
        // end in .drv for detection to work.
        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register("producer-gen.drv", &[], &mut kp);

        let mut w = Worker::new(1);
        w.want(&producer_sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        // The producer itself is a root, so at least 1 outcome.
        assert!(!result.outcomes.is_empty(), "should have at least the producer outcome");
        assert!(result.failed.is_empty(), "no failures expected");

        let recorded = calls.lock().unwrap();
        // Should have built the producer + the dynamically discovered inner drv.
        assert_eq!(
            recorded.len(),
            2,
            "expected 2 builds (producer + dynamic inner), got {}: {:?}",
            recorded.len(),
            *recorded
        );
    }

    /// Dynamic derivation via streaming: producer arrives over the
    /// eval channel, its output is a .drv, which is then built.
    #[tokio::test]
    async fn dynamic_drv_streaming() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let mut inner_kp = DerivationRegistry::default();
        let (_, inner_drv) = build_and_register("streamed-inner", &[], &mut inner_kp);
        let inner_aterm = inner_drv.to_aterm_bytes();

        let mut drv_outputs = StdHashMap::new();
        drv_outputs.insert("stream-producer.drv".to_string(), inner_aterm);

        let (mock, calls) = DrvProducingMockBuildService::new(bs.clone(), drv_outputs);

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
        let (producer_sp, _) = build_and_register("stream-producer.drv", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "stream-producer.drv".into(),
            drv_path: producer_sp.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert!(!result.outcomes.is_empty());
        assert!(result.failed.is_empty());

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "expected producer + dynamic inner: {:?}", *recorded);
    }

    /// Non-.drv output is NOT treated as a dynamic derivation.
    #[tokio::test]
    async fn non_drv_output_not_treated_as_dynamic() {
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
        // Name does NOT end in .drv.
        let (sp, _) = build_and_register("normal-pkg", &[], &mut kp);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        let recorded = calls.lock().unwrap();
        // Only 1 build — no dynamic discovery.
        assert_eq!(recorded.len(), 1);
    }

    // ── Partial failure integration tests ─────────────────────

    use crate::test_support::FailingMockBuildService;

    /// One root fails, the other succeeds. The Worker should continue
    /// building the successful root and report both outcomes.
    #[tokio::test]
    async fn partial_failure_one_fails_one_succeeds() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // "bad-pkg" will fail, "good-pkg" will succeed.
        let (mock, calls) = FailingMockBuildService::new(bs.clone(), vec!["bad-pkg".to_string()]);

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
        let (good_sp, _) = build_and_register("good-pkg", &[], &mut kp);
        let (bad_sp, _) = build_and_register("bad-pkg", &[], &mut kp);

        let mut w = Worker::new(2);
        w.want(&good_sp, &kp, true).unwrap();
        w.want(&bad_sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        // good-pkg should succeed.
        assert_eq!(result.outcomes.len(), 1, "one root should succeed");
        assert!(result.outcomes[0].drv_path.name().contains("good-pkg"), "successful outcome should be good-pkg");

        // bad-pkg should be in the failed list.
        assert_eq!(result.failed.len(), 1, "one root should fail");
        assert!(result.failed[0].drv_key.contains("bad-pkg"), "failed goal should be bad-pkg");
        assert!(
            result.failed[0].error.contains("simulated build failure"),
            "error message should describe the failure: {}",
            result.failed[0].error
        );

        // Both builds were attempted.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "both builds should be attempted");
    }

    /// A dep fails, its dependent root should also fail (propagated).
    /// An independent root should still succeed.
    #[tokio::test]
    async fn partial_failure_dep_fails_propagates() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // "bad-dep" will fail.
        let (mock, _calls) = FailingMockBuildService::new(bs.clone(), vec!["bad-dep".to_string()]);

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
        let (bad_dep, _) = build_and_register("bad-dep", &[], &mut kp);
        let (top_bad, _) = build_and_register("top-bad", &[(bad_dep.clone(), "out")], &mut kp);
        let (good_sp, _) = build_and_register("good-pkg", &[], &mut kp);

        let mut w = Worker::new(2);
        w.want(&top_bad, &kp, true).unwrap();
        w.want(&good_sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        // good-pkg should succeed.
        assert_eq!(result.outcomes.len(), 1, "one root should succeed");
        assert!(result.outcomes[0].drv_path.name().contains("good-pkg"), "successful outcome should be good-pkg");

        // top-bad should fail because its dep bad-dep failed.
        assert_eq!(result.failed.len(), 1, "one root should fail");
        assert!(
            result.failed[0].drv_key.contains("top-bad"),
            "failed goal should be top-bad, got: {}",
            result.failed[0].drv_key
        );
        assert!(
            result.failed[0].error.contains("dependency"),
            "error should mention dependency failure: {}",
            result.failed[0].error
        );
    }

    /// All roots fail — Worker should complete without panic.
    #[tokio::test]
    async fn partial_failure_all_fail() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (mock, _) = FailingMockBuildService::new(bs.clone(), vec!["fail-a".to_string(), "fail-b".to_string()]);

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
        let (a, _) = build_and_register("fail-a", &[], &mut kp);
        let (b, _) = build_and_register("fail-b", &[], &mut kp);

        let mut w = Worker::new(2);
        w.want(&a, &kp, true).unwrap();
        w.want(&b, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert!(result.outcomes.is_empty(), "no outcomes on total failure");
        assert_eq!(result.failed.len(), 2, "both roots should fail");
    }

    /// Streaming partial failure: one root fails, second arrives later
    /// and succeeds.
    #[tokio::test]
    async fn partial_failure_streaming() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (mock, _) = FailingMockBuildService::new(bs.clone(), vec!["stream-bad".to_string()]);

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
        let (bad_sp, _) = build_and_register("stream-bad", &[], &mut kp);
        let (good_sp, _) = build_and_register("stream-good", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "bad".into(),
            drv_path: bad_sp,
            new_entries: vec![],
        })
        .await
        .unwrap();
        tx.send(EvalMessage {
            label: "good".into(),
            drv_path: good_sp,
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 1, "one root should succeed");
        assert_eq!(result.failed.len(), 1, "one root should fail");
    }
}
