//! Worker: imperative shell that drives Goal state machines.
//!
//! The Worker owns the `GoalRegistry` and orchestrates builds by:
//! 1. Creating goals lazily via `want()` — deduplicates by drv path
//! 2. Inspecting deps from `KnownPaths` to wire waiters
//! 3. Dispatching Ready goals through `Builder::prepare_build()`
//! 4. Spawning sandbox builds on a `JoinSet` with `Semaphore` concurrency
//! 5. Completing builds via `Builder::finish_build()`, notifying waiters
//! 6. Repeating until all root goals are terminal
//!
//! The Worker borrows `&mut Builder` for I/O — it never does I/O itself.
//! Goal state transitions are validated by the pure `Goal` API.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::pathinfoservice::PathInfoService;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{debug, info};

use crunch_glue::KnownPaths;

use crate::goal::{GoalRegistry, GoalState, Goal, MAX_GOALS};
use crate::orchestrate::{BuildOutcome, Builder};
use crate::Error;

/// Logical store prefix — must match orchestrate.rs.
const LOGICAL_STORE_DIR: &str = "/nix/store";

/// Maximum concurrent in-flight builds. Clamped by the semaphore but
/// tracked here for assertions.
const MAX_IN_FLIGHT: u32 = 64;

/// Result of running the Worker: outcomes for root goals.
#[derive(Debug)]
pub struct WorkerResult {
    /// Build outcomes for root derivations that succeeded.
    pub outcomes: Vec<BuildOutcome>,
    /// Absolute drv-path keys of root goals that failed.
    pub failed: Vec<String>,
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
    /// `input_derivations` to discover deps, recursively creating
    /// sub-goals and wiring waiters.
    ///
    /// `is_root`: whether this was explicitly requested by the user.
    ///
    /// This is pure graph construction — no I/O. Dependencies are
    /// looked up from `known_paths`, not from the filesystem.
    pub fn want(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &KnownPaths,
        is_root: bool,
    ) -> Result<(), Error> {
        let key = drv_path.to_absolute_path();

        // Already tracked — just upgrade to root if needed.
        if let Some(goal) = self.registry.get_mut(&key) {
            if is_root {
                goal.is_root = true;
            }
            return Ok(());
        }

        // Look up derivation from KnownPaths.
        let drv_abs = drv_path.to_absolute_path_with_prefix(LOGICAL_STORE_DIR);
        let entry = known_paths.get_by_drv_path(&drv_abs).ok_or_else(|| {
            Error::DerivationNotFound { path: drv_path.clone() }
        })?;
        let derivation = entry.derivation.clone();

        // Create the goal.
        let goal = if is_root {
            Goal::new_root(drv_path.clone(), derivation.clone())
        } else {
            Goal::new(drv_path.clone(), derivation.clone())
        };
        self.registry.insert(key.clone(), goal)?;

        // Recursively create goals for input derivations.
        // Collect dep keys BEFORE inspecting (need to know which
        // are unbuilt).
        let dep_drv_paths: Vec<StorePath<String>> = derivation
            .input_derivations
            .keys()
            .cloned()
            .collect();

        for dep_sp in &dep_drv_paths {
            // Recursive want — creates sub-goals and their sub-goals.
            self.want(dep_sp, known_paths, false)?;
        }

        // Determine which deps are not yet Done.
        let unbuilt_dep_keys: Vec<String> = dep_drv_paths
            .iter()
            .map(|sp| sp.to_absolute_path())
            .filter(|dep_key| {
                self.registry.get(dep_key)
                    .map(|g| g.state != GoalState::Done)
                    .unwrap_or(false)
            })
            .collect();

        // Wire waiter relationships: each unbuilt dep gains this
        // goal as a waiter.
        for dep_key in &unbuilt_dep_keys {
            if let Some(dep_goal) = self.registry.get_mut(dep_key) {
                dep_goal.waiters.push(key.clone());
            }
        }

        // Inspect: Pending → Waiting or Ready.
        let goal = self.registry.get_mut(&key).expect("just inserted");
        let state = goal.inspect(unbuilt_dep_keys)?;

        if *state == GoalState::Ready {
            self.ready_queue.push_back(key);
        }

        Ok(())
    }

    /// Run the build loop until all root goals are terminal.
    ///
    /// Dispatches Ready goals through the Builder, spawns sandbox
    /// builds on a JoinSet, and processes completions. Cache hits
    /// and fetchers complete synchronously via `prepare_build`.
    pub async fn run<BS, DS, BServ, PIS>(
        &mut self,
        builder: &mut Builder<BS, DS, BServ, PIS>,
        known_paths: &mut KnownPaths,
    ) -> Result<WorkerResult, Error>
    where
        BS: BlobService + Clone + 'static,
        DS: DirectoryService + Clone + 'static,
        BServ: BuildService + 'static,
        PIS: PathInfoService,
    {
        let total_goals = self.registry.len();
        let root_count = self.registry.root_count();
        info!(
            goals = total_goals,
            roots = root_count,
            jobs = self.max_jobs,
            "worker starting"
        );

        // Tiger Style: assert positive space.
        debug_assert!(root_count > 0, "no root goals to build");
        debug_assert!(total_goals <= MAX_GOALS, "goal count exceeds limit");

        let sem = Arc::new(Semaphore::new(self.max_jobs as usize));
        let mut join_set: JoinSet<
            Result<(String, snix_build::buildservice::BuildResult), Error>,
        > = JoinSet::new();

        // drv_key → PreparedBuild metadata for in-flight sandbox builds.
        let mut pending_meta: HashMap<String, PreparedBuild> = HashMap::new();
        let mut completed_count: u32 = 0;
        let mut outcomes: Vec<BuildOutcome> = Vec::new();
        let mut failed: Vec<String> = Vec::new();

        // Iteration limit to prevent infinite loops from bugs.
        let iteration_limit: u32 = total_goals.saturating_mul(4).max(16);
        let mut iterations: u32 = 0;

        loop {
            iterations = iterations.saturating_add(1);
            if iterations > iteration_limit {
                return Err(Error::Store(format!(
                    "worker loop exceeded iteration limit ({iteration_limit})"
                )));
            }

            // Phase 1: Dispatch all Ready goals.
            let dispatched = self
                .dispatch_ready(
                    builder, known_paths, &sem, &mut join_set,
                    &mut pending_meta, &mut outcomes, &mut failed,
                )
                .await?;

            // Phase 2: Check termination.
            if self.registry.all_roots_terminal() {
                break;
            }

            // Phase 3: Wait for one in-flight build to complete.
            if join_set.is_empty() {
                // No in-flight builds and no ready goals — should not
                // happen if the graph is valid.
                if dispatched == 0 {
                    return Err(Error::Store(
                        "worker deadlock: no ready goals and no in-flight builds".into(),
                    ));
                }
                continue;
            }

            let join_result = join_set.join_next().await;
            let Some(result) = join_result else {
                continue;
            };

            let (drv_key, build_result) = result
                .map_err(|e| Error::Store(format!("task join: {e}")))?
                .map_err(|e: Error| e)?;

            let prepared = pending_meta.remove(&drv_key).ok_or_else(|| {
                Error::Store(format!("BUG: completed build has no pending metadata: {drv_key}"))
            })?;

            let outcome = builder
                .finish_build(&prepared, build_result, known_paths)
                .await?;

            self.complete_goal(&drv_key, outcome, &mut outcomes, &mut failed)?;
            completed_count = completed_count.saturating_add(1);
        }

        info!(
            completed = completed_count,
            succeeded = outcomes.len(),
            failed = failed.len(),
            "worker finished"
        );

        Ok(WorkerResult { outcomes, failed })
    }

    /// Dispatch all Ready goals. Returns the number dispatched.
    ///
    /// For each Ready goal:
    /// - Call `prepare_build` (may resolve as cache hit or fetcher)
    /// - If cache/fetcher: complete immediately, notify waiters
    /// - If sandbox build: spawn on JoinSet with Semaphore
    async fn dispatch_ready<BS, DS, BServ, PIS>(
        &mut self,
        builder: &mut Builder<BS, DS, BServ, PIS>,
        known_paths: &mut KnownPaths,
        sem: &Arc<Semaphore>,
        join_set: &mut JoinSet<Result<(String, snix_build::buildservice::BuildResult), Error>>,
        pending_meta: &mut HashMap<String, PreparedBuild>,
        outcomes: &mut Vec<BuildOutcome>,
        failed: &mut Vec<String>,
    ) -> Result<u32, Error>
    where
        BS: BlobService + Clone + 'static,
        DS: DirectoryService + Clone + 'static,
        BServ: BuildService + 'static,
        PIS: PathInfoService,
    {
        let mut dispatched: u32 = 0;

        while let Some(drv_key) = self.ready_queue.pop_front() {
            let goal = self.registry.get(&drv_key).ok_or_else(|| {
                Error::Store(format!("ready goal missing from registry: {drv_key}"))
            })?;

            // Tiger Style: assert state precondition.
            debug_assert_eq!(
                goal.state, GoalState::Ready,
                "goal in ready queue but state is {:?}",
                goal.state
            );

            let drv_path = goal.drv_path.clone();
            let derivation = goal.derivation.as_ref()
                .ok_or_else(|| Error::Store(format!(
                    "goal {drv_key} has no derivation (dynamic derivations not yet supported)"
                )))?
                .clone();

            match builder.prepare_build(&drv_path, &derivation, known_paths).await? {
                PrepareResult::Done(outcome) => {
                    // Cache hit or fetcher — complete synchronously.
                    self.complete_goal(&drv_key, outcome, outcomes, failed)?;
                }
                PrepareResult::NeedsBuild(prepared) => {
                    // Mark Building before spawning.
                    let goal = self.registry.get_mut(&drv_key).expect("just checked");
                    goal.mark_building()?;

                    let build_request = prepared.build_request.clone();
                    pending_meta.insert(drv_key.clone(), prepared);

                    let bs = builder.build_service();
                    let sem = sem.clone();
                    let key = drv_key.clone();

                    join_set.spawn(async move {
                        let _permit = sem.acquire_owned().await
                            .map_err(|e| Error::Store(format!("semaphore: {e}")))?;
                        let result = bs.do_build(build_request).await
                            .map_err(|e| Error::Store(format!("build: {e}")))?;
                        Ok((key, result))
                    });
                }
            }

            dispatched = dispatched.saturating_add(1);
        }

        Ok(dispatched)
    }

    /// Mark a goal as Done, collect its outcome, and notify waiters.
    /// If a waiter becomes Ready, push it to the ready queue.
    fn complete_goal(
        &mut self,
        drv_key: &str,
        outcome: BuildOutcome,
        outcomes: &mut Vec<BuildOutcome>,
        _failed: &mut Vec<String>,
    ) -> Result<(), Error> {
        let goal = self.registry.get_mut(drv_key).ok_or_else(|| {
            Error::Store(format!("completing unknown goal: {drv_key}"))
        })?;

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
                return Err(Error::Store(format!(
                    "completing goal {drv_key} in {:?} state",
                    goal.state
                )));
            }
        }

        let is_root = goal.is_root;
        let waiters = goal.waiters.clone();

        if is_root {
            outcomes.push(outcome);
        }

        debug!(
            drv = drv_key,
            waiters = waiters.len(),
            "goal completed"
        );

        // Notify waiters.
        for waiter_key in &waiters {
            let waiter = self.registry.get_mut(waiter_key).ok_or_else(|| {
                Error::Store(format!("waiter goal missing: {waiter_key}"))
            })?;

            let became_ready = waiter.notify_dep_done()?;
            if became_ready {
                self.ready_queue.push_back(waiter_key.clone());
            }
        }

        Ok(())
    }

    /// Mark a goal as failed and propagate failure to waiters.
    /// Not yet called — build errors currently abort via `?`. This
    /// will be used when partial-failure (continue other roots) lands.
    #[allow(dead_code)]
    fn fail_goal(
        &mut self,
        drv_key: &str,
        failed: &mut Vec<String>,
    ) -> Result<(), Error> {
        let goal = self.registry.get_mut(drv_key).ok_or_else(|| {
            Error::Store(format!("failing unknown goal: {drv_key}"))
        })?;

        goal.mark_build_failed()?;

        let is_root = goal.is_root;
        let waiters = goal.waiters.clone();

        if is_root {
            failed.push(drv_key.to_string());
        }

        // Propagate failure to waiters.
        for waiter_key in &waiters {
            self.propagate_failure(waiter_key, failed)?;
        }

        Ok(())
    }

    /// Recursively propagate dep failure to waiting goals.
    #[allow(dead_code)]
    fn propagate_failure(
        &mut self,
        drv_key: &str,
        failed: &mut Vec<String>,
    ) -> Result<(), Error> {
        let goal = self.registry.get_mut(drv_key).ok_or_else(|| {
            Error::Store(format!("propagating failure to unknown goal: {drv_key}"))
        })?;

        // Only propagate to goals still Waiting.
        if !matches!(goal.state, GoalState::Waiting { .. }) {
            return Ok(());
        }

        goal.notify_dep_failed()?;

        let is_root = goal.is_root;
        let waiters = goal.waiters.clone();

        if is_root {
            failed.push(drv_key.to_string());
        }

        for waiter_key in &waiters {
            self.propagate_failure(waiter_key, failed)?;
        }

        Ok(())
    }

    /// Read-only access to the goal registry (for testing/inspection).
    pub fn registry(&self) -> &GoalRegistry {
        &self.registry
    }
}

// ── Type alias for PreparedBuild to avoid reaching into orchestrate internals ──
// The Worker needs PreparedBuild and PrepareResult from orchestrate.rs.
// For now we re-export them. Phase 4 cleanup will move them to a shared module.
use crate::orchestrate::PreparedBuild;
use crate::orchestrate::PrepareResult;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goal::GoalState;
    use nix_compat::derivation::{Derivation, Output};
    use std::collections::{BTreeMap, BTreeSet};

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

    fn register_drv(kp: &mut KnownPaths, name: &str, drv: &Derivation) -> StorePath<String> {
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
        kp.insert(aterm_hash, sp.clone(), hdm, drv.clone());
        sp
    }

    // ── want() tests ────────────────────────────────────────────

    #[test]
    fn want_single_leaf() {
        let mut kp = KnownPaths::default();
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
        let mut kp = KnownPaths::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let mid_drv = make_drv_with_deps(&[leaf_sp.clone()]);
        let mid_sp = register_drv(&mut kp, "mid.drv", &mid_drv);

        let top_drv = make_drv_with_deps(&[mid_sp.clone()]);
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
        let mut kp = KnownPaths::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let left_drv = make_drv_with_deps(&[shared_sp.clone()]);
        let left_sp = register_drv(&mut kp, "left.drv", &left_drv);

        let right_drv = make_drv_with_deps(&[shared_sp.clone()]);
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
        let mut kp = KnownPaths::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "leaf.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, false).unwrap();
        w.want(&sp, &kp, false).unwrap();

        assert_eq!(w.registry.len(), 1);
    }

    #[test]
    fn want_upgrades_to_root() {
        let mut kp = KnownPaths::default();
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
        let kp = KnownPaths::default();
        let sp = fake_sp("unknown.drv");

        let mut w = Worker::new(1);
        assert!(w.want(&sp, &kp, true).is_err());
    }

    #[test]
    fn want_disjoint_trees() {
        let mut kp = KnownPaths::default();

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
        let mut kp = KnownPaths::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let top_drv = make_drv_with_deps(&[leaf_sp.clone()]);
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        // Leaf is Ready, top is Waiting.
        let leaf_key = leaf_sp.to_absolute_path();
        let top_key = top_sp.to_absolute_path();
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Ready);
        assert!(matches!(
            w.registry.get(&top_key).unwrap().state,
            GoalState::Waiting { remaining_deps: 1 }
        ));

        // Simulate completing the leaf.
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: leaf_sp.clone(),
            outputs: std::collections::HashMap::new(),
            cached: false,
            log: None,
        };
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&leaf_key, outcome, &mut outcomes, &mut failed).unwrap();

        // Leaf should be Done.
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Done);
        // Top should now be Ready (its sole dep completed).
        assert_eq!(w.registry.get(&top_key).unwrap().state, GoalState::Ready);
        // Top should be in the ready queue.
        assert!(w.ready_queue.contains(&top_key));
        // Leaf is not a root, so outcomes should be empty.
        assert!(outcomes.is_empty());
    }

    #[test]
    fn complete_goal_collects_root_outcome() {
        let mut kp = KnownPaths::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "root.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();

        let key = sp.to_absolute_path();
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: sp.clone(),
            outputs: std::collections::HashMap::new(),
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
    fn complete_diamond_shared_dep_unblocks_both() {
        let mut kp = KnownPaths::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let left_drv = make_drv_with_deps(&[shared_sp.clone()]);
        let left_sp = register_drv(&mut kp, "left.drv", &left_drv);

        let right_drv = make_drv_with_deps(&[shared_sp.clone()]);
        let right_sp = register_drv(&mut kp, "right.drv", &right_drv);

        let top_drv = make_drv_with_deps(&[left_sp.clone(), right_sp.clone()]);
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(2);
        w.want(&top_sp, &kp, true).unwrap();

        // Complete shared dep.
        let shared_key = shared_sp.to_absolute_path();
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: shared_sp.clone(),
            outputs: std::collections::HashMap::new(),
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
        assert!(matches!(
            w.registry.get(&top_key).unwrap().state,
            GoalState::Waiting { remaining_deps: 2 }
        ));
    }

    // ── fail_goal / propagate_failure ─────────────────────────

    #[test]
    fn fail_goal_propagates_to_waiters() {
        let mut kp = KnownPaths::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let mid_drv = make_drv_with_deps(&[leaf_sp.clone()]);
        let mid_sp = register_drv(&mut kp, "mid.drv", &mid_drv);

        let top_drv = make_drv_with_deps(&[mid_sp.clone()]);
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        // Leaf is Ready — simulate it being dispatched and starting.
        let leaf_key = leaf_sp.to_absolute_path();
        w.ready_queue.pop_front(); // remove leaf from ready queue
        w.registry.get_mut(&leaf_key).unwrap().mark_building().unwrap();

        // Fail the leaf.
        let mut failed = Vec::new();
        w.fail_goal(&leaf_key, &mut failed).unwrap();

        // Leaf should be Failed.
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Failed);
        // Mid should be Failed (dep failed).
        let mid_key = mid_sp.to_absolute_path();
        assert_eq!(w.registry.get(&mid_key).unwrap().state, GoalState::Failed);
        // Top should be Failed (transitive dep failed).
        let top_key = top_sp.to_absolute_path();
        assert_eq!(w.registry.get(&top_key).unwrap().state, GoalState::Failed);
        // Top is a root, so it should be in the failed list.
        assert!(failed.contains(&top_key));
    }

    #[test]
    fn fail_goal_only_propagates_to_waiting() {
        let mut kp = KnownPaths::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let good_drv = make_drv_with_deps(&[shared_sp.clone()]);
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
        w.fail_goal(&bad_key, &mut failed).unwrap();

        assert_eq!(w.registry.get(&bad_key).unwrap().state, GoalState::Failed);
        // good is still Waiting on shared, not failed.
        let good_key = good_sp.to_absolute_path();
        assert!(matches!(
            w.registry.get(&good_key).unwrap().state,
            GoalState::Waiting { .. }
        ));
        // bad is a root, so it should be in failed list.
        assert!(failed.contains(&bad_key));
    }

    // ── run() integration tests ───────────────────────────────

    use crate::orchestrate::Builder;
    use crate::test_support::{self, MockBuildService, build_and_register, tmp_ds, test_pis};
    use snix_castore::blobservice::MemoryBlobService;
    use std::path::PathBuf;

    #[tokio::test]
    async fn run_single_leaf_builds_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
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
        let shared_builds = recorded.iter()
            .filter(|a| a.iter().any(|s| s.contains("shared")))
            .count();
        assert_eq!(shared_builds, 1, "shared built exactly once");
    }

    #[tokio::test]
    async fn run_disjoint_trees() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
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
            bs, ds, mock, test_pis(), PathBuf::from("/nix/store"), false,
        );

        let mut kp = crunch_glue::KnownPaths::default();
        let mut w = Worker::new(1);

        // No roots — debug_assert catches this.
        let _ = w.run(&mut builder, &mut kp).await;
    }
}
