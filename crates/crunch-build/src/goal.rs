//! Goal state machine for build scheduling.
//!
//! Each derivation in the build graph becomes a `Goal`. Goals track
//! their state, dependencies (waitees), and reverse dependencies
//! (waiters). The Worker drives state transitions.
//!
//! This module is pure data — no I/O, no async, no Builder references.
//! State transitions are validated and return errors on invalid moves.

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use std::collections::HashMap;

use crate::error::Error;

/// Maximum number of goals a Worker can track. Prevents runaway
/// graphs from pathological inputs.
pub const MAX_GOALS: u32 = 10_000;

/// Lifecycle state of a build goal.
///
/// Transitions:
/// ```text
/// AwaitingDerivation → Pending  (producer built, .drv parsed and set)
/// Pending → Waiting  (has unbuilt deps)
/// Pending → Ready    (all deps done or no deps)
/// Waiting → Ready    (last dep completed)
/// Waiting → Failed   (a dep failed)
/// Ready   → Building (build slot acquired)
/// Building → Done    (build succeeded)
/// Building → Failed  (build error)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalState {
    /// Waiting for a producer build to complete and provide the
    /// `.drv` content. Used for dynamic derivations.
    AwaitingDerivation,
    /// Created but dependencies not yet inspected.
    Pending,
    /// Waiting for `remaining_deps` dependencies to complete.
    Waiting { remaining_deps: u32 },
    /// All dependencies satisfied, eligible for a build slot.
    Ready,
    /// Sandbox build dispatched, awaiting completion.
    Building,
    /// Build completed successfully.
    Done,
    /// Build or dependency failed.
    Failed,
}

/// A single build goal representing one derivation.
#[derive(Debug)]
pub struct Goal {
    /// The derivation's store path (the .drv path).
    pub drv_path: StorePath<String>,
    /// The derivation to build. `None` while in `AwaitingDerivation`
    /// state (dynamic derivations — producer hasn't finished yet).
    pub derivation: Option<Derivation>,
    /// Current lifecycle state.
    pub state: GoalState,
    /// Absolute drv-path keys of goals that this one waits on.
    /// Populated during `inspect()`.
    pub waitees: Vec<String>,
    /// Absolute drv-path keys of goals waiting for this one.
    /// Populated by the Worker when wiring dependencies.
    pub waiters: Vec<String>,
    /// Whether this goal was explicitly requested by the user.
    pub is_root: bool,
    /// The goal key of the producer that will provide our `.drv`.
    /// Set only for dynamic derivation goals in `AwaitingDerivation` state.
    pub producer_key: Option<String>,
}

impl Goal {
    /// Create a new goal in `Pending` state.
    pub fn new(drv_path: StorePath<String>, derivation: Derivation) -> Self {
        Self {
            drv_path,
            derivation: Some(derivation),
            state: GoalState::Pending,
            waitees: Vec::new(),
            waiters: Vec::new(),
            is_root: false,
            producer_key: None,
        }
    }

    /// Create a new root goal in `Pending` state.
    pub fn new_root(drv_path: StorePath<String>, derivation: Derivation) -> Self {
        let mut goal = Self::new(drv_path, derivation);
        goal.is_root = true;
        goal
    }

    /// Create a dynamic goal in `AwaitingDerivation` state.
    ///
    /// The goal has no derivation yet — a producer build must complete
    /// and provide the `.drv` content via `set_derivation()`.
    pub fn new_awaiting(
        drv_path: StorePath<String>,
        producer_key: String,
    ) -> Self {
        Self {
            drv_path,
            derivation: None,
            state: GoalState::AwaitingDerivation,
            waitees: Vec::new(),
            waiters: Vec::new(),
            is_root: false,
            producer_key: Some(producer_key),
        }
    }

    /// Provide the derivation for a dynamic goal. Transitions from
    /// `AwaitingDerivation` to `Pending`.
    ///
    /// Called by the Worker after the producer build completes and
    /// the `.drv` output has been parsed.
    pub fn set_derivation(&mut self, derivation: Derivation) -> Result<(), Error> {
        if self.state != GoalState::AwaitingDerivation {
            return Err(Error::Store(format!(
                "goal {}: set_derivation() called in {:?} state, expected AwaitingDerivation",
                self.drv_path.name(),
                self.state,
            )));
        }
        debug_assert!(
            self.derivation.is_none(),
            "AwaitingDerivation goal should not already have a derivation"
        );
        self.derivation = Some(derivation);
        self.state = GoalState::Pending;
        Ok(())
    }

    /// Inspect dependencies and transition from Pending to Waiting or Ready.
    ///
    /// `unbuilt_dep_keys`: absolute drv-path strings of deps that are
    /// NOT yet Done. If empty, the goal is Ready immediately.
    ///
    /// Returns the new state. Errors if not in Pending state.
    pub fn inspect(&mut self, unbuilt_dep_keys: Vec<String>) -> Result<&GoalState, Error> {
        // Tiger Style: assert precondition.
        if self.state != GoalState::Pending {
            return Err(Error::Store(format!(
                "goal {}: inspect() called in {:?} state, expected Pending",
                self.drv_path.name(),
                self.state,
            )));
        }

        let dep_count = unbuilt_dep_keys.len();
        debug_assert!(dep_count <= MAX_GOALS as usize, "dep count exceeds MAX_GOALS");

        self.waitees = unbuilt_dep_keys;

        if dep_count == 0 {
            self.state = GoalState::Ready;
        } else {
            self.state = GoalState::Waiting {
                remaining_deps: dep_count as u32,
            };
        }

        Ok(&self.state)
    }

    /// Notify that one dependency has completed. Decrements the
    /// remaining count. Transitions to Ready when it hits zero.
    ///
    /// Returns `true` if the goal just became Ready.
    /// Errors if not in Waiting state.
    pub fn notify_dep_done(&mut self) -> Result<bool, Error> {
        match &mut self.state {
            GoalState::Waiting { remaining_deps } => {
                // Tiger Style: assert positive space.
                debug_assert!(*remaining_deps > 0, "remaining_deps already zero");

                *remaining_deps = remaining_deps.saturating_sub(1);
                if *remaining_deps == 0 {
                    self.state = GoalState::Ready;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            other => Err(Error::Store(format!(
                "goal {}: notify_dep_done() called in {other:?} state, expected Waiting",
                self.drv_path.name(),
            ))),
        }
    }

    /// Notify that a dependency has failed. Transitions to Failed.
    ///
    /// Errors if not in Waiting or AwaitingDerivation state.
    pub fn notify_dep_failed(&mut self) -> Result<(), Error> {
        if !matches!(self.state, GoalState::Waiting { .. } | GoalState::AwaitingDerivation) {
            return Err(Error::Store(format!(
                "goal {}: notify_dep_failed() in {:?}, expected Waiting or AwaitingDerivation",
                self.drv_path.name(),
                self.state,
            )));
        }
        self.state = GoalState::Failed;
        Ok(())
    }

    /// Transition from Ready to Building.
    pub fn mark_building(&mut self) -> Result<(), Error> {
        if self.state != GoalState::Ready {
            return Err(Error::Store(format!(
                "goal {}: mark_building() in {:?}, expected Ready",
                self.drv_path.name(),
                self.state,
            )));
        }
        self.state = GoalState::Building;
        Ok(())
    }

    /// Transition from Building to Done.
    pub fn mark_done(&mut self) -> Result<(), Error> {
        if self.state != GoalState::Building {
            return Err(Error::Store(format!(
                "goal {}: mark_done() in {:?}, expected Building",
                self.drv_path.name(),
                self.state,
            )));
        }
        self.state = GoalState::Done;
        Ok(())
    }

    /// Transition from Building (or Ready, for inline failures) to Failed.
    ///
    /// Accepts `Ready` because `prepare_build` can fail inline (e.g.,
    /// fetcher hash mismatch) before the goal transitions to `Building`.
    pub fn mark_build_failed(&mut self) -> Result<(), Error> {
        if self.state != GoalState::Building && self.state != GoalState::Ready {
            return Err(Error::Store(format!(
                "goal {}: mark_build_failed() in {:?}, expected Building or Ready",
                self.drv_path.name(),
                self.state,
            )));
        }
        self.state = GoalState::Failed;
        Ok(())
    }
}

/// Pure goal registry. No I/O, no scheduling — just tracks goals
/// and their relationships. The Worker uses this as its core data
/// structure.
#[derive(Debug)]
pub struct GoalRegistry {
    /// Goals keyed by absolute drv path.
    goals: HashMap<String, Goal>,
}

impl GoalRegistry {
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
        }
    }

    /// Number of tracked goals.
    pub fn len(&self) -> u32 {
        self.goals.len() as u32
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.goals.is_empty()
    }

    /// Check if a goal exists for the given key.
    pub fn contains(&self, key: &str) -> bool {
        self.goals.contains_key(key)
    }

    /// Get a reference to a goal.
    pub fn get(&self, key: &str) -> Option<&Goal> {
        self.goals.get(key)
    }

    /// Get a mutable reference to a goal.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Goal> {
        self.goals.get_mut(key)
    }

    /// Insert a new goal. Returns error if MAX_GOALS exceeded or
    /// key already exists.
    pub fn insert(&mut self, key: String, goal: Goal) -> Result<(), Error> {
        if self.goals.len() as u32 >= MAX_GOALS {
            return Err(Error::Store(format!(
                "goal limit ({MAX_GOALS}) exceeded"
            )));
        }
        if self.goals.contains_key(&key) {
            return Err(Error::Store(format!(
                "goal already exists: {key}"
            )));
        }

        // Tiger Style: assert postcondition.
        let old_len = self.goals.len();
        self.goals.insert(key, goal);
        debug_assert_eq!(self.goals.len(), old_len + 1);

        Ok(())
    }

    /// Get or create a goal. Returns the key and whether it was newly
    /// created. This is the dedup mechanism — same drv path → same goal.
    pub fn get_or_insert(
        &mut self,
        key: String,
        make_goal: impl FnOnce() -> Goal,
    ) -> Result<(&mut Goal, bool), Error> {
        if self.goals.contains_key(&key) {
            let goal = self.goals.get_mut(&key).unwrap();
            Ok((goal, false))
        } else {
            if self.goals.len() as u32 >= MAX_GOALS {
                return Err(Error::Store(format!(
                    "goal limit ({MAX_GOALS}) exceeded"
                )));
            }
            self.goals.entry(key).or_insert_with(make_goal);
            // Re-borrow to satisfy the borrow checker.
            // entry().or_insert_with returns &mut V but we need
            // the borrow from self.goals, not the entry API.
            let goal = self.goals.values_mut().last().unwrap();
            Ok((goal, true))
        }
    }

    /// Iterate over all goals.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Goal)> {
        self.goals.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Count goals in a given state.
    pub fn count_in_state(&self, state: &GoalState) -> u32 {
        self.goals
            .values()
            .filter(|g| &g.state == state)
            .count() as u32
    }

    /// Count root goals.
    pub fn root_count(&self) -> u32 {
        self.goals.values().filter(|g| g.is_root).count() as u32
    }

    /// Check if all root goals are terminal (Done or Failed).
    pub fn all_roots_terminal(&self) -> bool {
        self.goals
            .values()
            .filter(|g| g.is_root)
            .all(|g| matches!(g.state, GoalState::Done | GoalState::Failed))
    }

    /// Collect outcomes: root goals that are Done.
    pub fn root_done_keys(&self) -> Vec<String> {
        self.goals
            .iter()
            .filter(|(_, g)| g.is_root && g.state == GoalState::Done)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Collect failed root goal keys.
    pub fn root_failed_keys(&self) -> Vec<String> {
        self.goals
            .iter()
            .filter(|(_, g)| g.is_root && g.state == GoalState::Failed)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Find goals in `AwaitingDerivation` whose producer matches the
    /// given key. Used after a build completes to discover dynamic
    /// derivation goals waiting on that producer.
    pub fn awaiting_producer(&self, producer_key: &str) -> Vec<String> {
        self.goals
            .iter()
            .filter(|(_, g)| {
                g.state == GoalState::AwaitingDerivation
                    && g.producer_key.as_deref() == Some(producer_key)
            })
            .map(|(k, _)| k.clone())
            .collect()
    }
}

impl Default for GoalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix_compat::derivation::Output;
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

    fn fake_sp(name: &str) -> StorePath<String> {
        let mut digest = [0u8; 20];
        for (i, b) in name.bytes().enumerate() {
            digest[i % 20] ^= b;
        }
        StorePath::from_name_and_digest_fixed(name, digest).unwrap()
    }

    // ── Goal state transitions ──────────────────────────────────

    #[test]
    fn new_goal_is_pending() {
        let g = Goal::new(fake_sp("a.drv"), make_drv());
        assert_eq!(g.state, GoalState::Pending);
        assert!(!g.is_root);
    }

    #[test]
    fn new_root_goal_is_pending_and_root() {
        let g = Goal::new_root(fake_sp("a.drv"), make_drv());
        assert_eq!(g.state, GoalState::Pending);
        assert!(g.is_root);
    }

    #[test]
    fn inspect_no_deps_goes_ready() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        let state = g.inspect(vec![]).unwrap();
        assert_eq!(*state, GoalState::Ready);
    }

    #[test]
    fn inspect_with_deps_goes_waiting() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        let state = g.inspect(vec!["dep1".into(), "dep2".into()]).unwrap();
        assert_eq!(*state, GoalState::Waiting { remaining_deps: 2 });
        assert_eq!(g.waitees.len(), 2);
    }

    #[test]
    fn inspect_not_pending_errors() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec![]).unwrap(); // → Ready
        let err = g.inspect(vec![]);
        assert!(err.is_err());
    }

    #[test]
    fn notify_dep_done_decrements() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec!["d1".into(), "d2".into()]).unwrap();

        let became_ready = g.notify_dep_done().unwrap();
        assert!(!became_ready);
        assert_eq!(g.state, GoalState::Waiting { remaining_deps: 1 });

        let became_ready = g.notify_dep_done().unwrap();
        assert!(became_ready);
        assert_eq!(g.state, GoalState::Ready);
    }

    #[test]
    fn notify_dep_done_not_waiting_errors() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec![]).unwrap(); // → Ready
        assert!(g.notify_dep_done().is_err());
    }

    #[test]
    fn notify_dep_failed_goes_failed() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec!["d1".into()]).unwrap();
        g.notify_dep_failed().unwrap();
        assert_eq!(g.state, GoalState::Failed);
    }

    #[test]
    fn notify_dep_failed_not_waiting_errors() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        assert!(g.notify_dep_failed().is_err());
    }

    #[test]
    fn mark_building_from_ready() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec![]).unwrap();
        g.mark_building().unwrap();
        assert_eq!(g.state, GoalState::Building);
    }

    #[test]
    fn mark_building_not_ready_errors() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        assert!(g.mark_building().is_err());
    }

    #[test]
    fn mark_done_from_building() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec![]).unwrap();
        g.mark_building().unwrap();
        g.mark_done().unwrap();
        assert_eq!(g.state, GoalState::Done);
    }

    #[test]
    fn mark_done_not_building_errors() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        assert!(g.mark_done().is_err());
    }

    #[test]
    fn mark_build_failed_from_building() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec![]).unwrap();
        g.mark_building().unwrap();
        g.mark_build_failed().unwrap();
        assert_eq!(g.state, GoalState::Failed);
    }

    #[test]
    fn mark_build_failed_from_ready() {
        // Ready → Failed is valid (inline failures in prepare_build).
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        g.inspect(vec![]).unwrap();
        assert_eq!(g.state, GoalState::Ready);
        g.mark_build_failed().unwrap();
        assert_eq!(g.state, GoalState::Failed);
    }

    #[test]
    fn mark_build_failed_not_building_or_ready_errors() {
        // Pending → mark_build_failed should still fail.
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        assert_eq!(g.state, GoalState::Pending);
        assert!(g.mark_build_failed().is_err());
    }

    // ── Full lifecycle ──────────────────────────────────────────

    #[test]
    fn full_lifecycle_leaf() {
        let mut g = Goal::new(fake_sp("leaf.drv"), make_drv());
        assert_eq!(g.state, GoalState::Pending);

        g.inspect(vec![]).unwrap();
        assert_eq!(g.state, GoalState::Ready);

        g.mark_building().unwrap();
        assert_eq!(g.state, GoalState::Building);

        g.mark_done().unwrap();
        assert_eq!(g.state, GoalState::Done);
    }

    #[test]
    fn full_lifecycle_with_deps() {
        let mut g = Goal::new(fake_sp("top.drv"), make_drv());

        g.inspect(vec!["dep.drv".into()]).unwrap();
        assert_eq!(g.state, GoalState::Waiting { remaining_deps: 1 });

        g.notify_dep_done().unwrap();
        assert_eq!(g.state, GoalState::Ready);

        g.mark_building().unwrap();
        g.mark_done().unwrap();
        assert_eq!(g.state, GoalState::Done);
    }

    // ── GoalRegistry ────────────────────────────────────────────

    #[test]
    fn registry_insert_and_get() {
        let mut reg = GoalRegistry::new();
        let sp = fake_sp("a.drv");
        let key = sp.to_absolute_path();

        reg.insert(key.clone(), Goal::new(sp, make_drv())).unwrap();

        assert_eq!(reg.len(), 1);
        assert!(reg.contains(&key));
        assert!(reg.get(&key).is_some());
    }

    #[test]
    fn registry_duplicate_insert_errors() {
        let mut reg = GoalRegistry::new();
        let sp = fake_sp("a.drv");
        let key = sp.to_absolute_path();

        reg.insert(key.clone(), Goal::new(sp.clone(), make_drv())).unwrap();
        let err = reg.insert(key, Goal::new(sp, make_drv()));
        assert!(err.is_err());
    }

    #[test]
    fn registry_get_or_insert_dedup() {
        let mut reg = GoalRegistry::new();
        let sp = fake_sp("a.drv");
        let key = sp.to_absolute_path();

        let (_, created1) = reg.get_or_insert(key.clone(), || Goal::new(sp.clone(), make_drv())).unwrap();
        assert!(created1);

        let (_, created2) = reg.get_or_insert(key, || Goal::new(sp, make_drv())).unwrap();
        assert!(!created2);

        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn registry_count_in_state() {
        let mut reg = GoalRegistry::new();

        let sp1 = fake_sp("a.drv");
        let sp2 = fake_sp("b.drv");
        reg.insert(sp1.to_absolute_path(), Goal::new(sp1, make_drv())).unwrap();
        reg.insert(sp2.to_absolute_path(), Goal::new(sp2, make_drv())).unwrap();

        assert_eq!(reg.count_in_state(&GoalState::Pending), 2);
        assert_eq!(reg.count_in_state(&GoalState::Ready), 0);
    }

    #[test]
    fn registry_all_roots_terminal() {
        let mut reg = GoalRegistry::new();

        let sp = fake_sp("root.drv");
        let key = sp.to_absolute_path();
        let mut g = Goal::new_root(sp, make_drv());
        g.inspect(vec![]).unwrap();
        g.mark_building().unwrap();
        g.mark_done().unwrap();
        reg.insert(key, g).unwrap();

        assert!(reg.all_roots_terminal());
    }

    #[test]
    fn registry_roots_not_terminal_when_building() {
        let mut reg = GoalRegistry::new();

        let sp = fake_sp("root.drv");
        let key = sp.to_absolute_path();
        let mut g = Goal::new_root(sp, make_drv());
        g.inspect(vec![]).unwrap();
        g.mark_building().unwrap();
        reg.insert(key, g).unwrap();

        assert!(!reg.all_roots_terminal());
    }

    #[test]
    fn registry_empty_is_terminal() {
        let reg = GoalRegistry::new();
        assert!(reg.all_roots_terminal());
    }

    // ── MAX_GOALS limit ─────────────────────────────────────────

    #[test]
    fn registry_max_goals_enforced() {
        let mut reg = GoalRegistry::new();

        // Fill to MAX_GOALS.
        for i in 0..MAX_GOALS {
            let name = format!("g{i}.drv");
            let sp = fake_sp(&name);
            let key = sp.to_absolute_path();
            reg.insert(key, Goal::new(sp, make_drv())).unwrap();
        }

        assert_eq!(reg.len(), MAX_GOALS);

        // One more should fail.
        let sp = fake_sp("overflow.drv");
        let key = sp.to_absolute_path();
        let err = reg.insert(key, Goal::new(sp, make_drv()));
        assert!(err.is_err());
        assert_eq!(reg.len(), MAX_GOALS);
    }

    #[test]
    fn registry_get_or_insert_respects_limit() {
        let mut reg = GoalRegistry::new();

        for i in 0..MAX_GOALS {
            let name = format!("g{i}.drv");
            let sp = fake_sp(&name);
            let key = sp.to_absolute_path();
            reg.get_or_insert(key, || Goal::new(sp, make_drv())).unwrap();
        }

        // New key should fail.
        let sp = fake_sp("overflow.drv");
        let key = sp.to_absolute_path();
        let err = reg.get_or_insert(key, || Goal::new(sp, make_drv()));
        assert!(err.is_err());

        // Existing key should still work (no insertion).
        let sp0 = fake_sp("g0.drv");
        let key0 = sp0.to_absolute_path();
        let (_, created) = reg.get_or_insert(key0, || Goal::new(sp0, make_drv())).unwrap();
        assert!(!created);
    }

    // ── Failure lifecycle ────────────────────────────────────────

    #[test]
    fn full_lifecycle_dep_failure() {
        let mut g = Goal::new(fake_sp("top.drv"), make_drv());

        g.inspect(vec!["dep.drv".into()]).unwrap();
        assert_eq!(g.state, GoalState::Waiting { remaining_deps: 1 });

        g.notify_dep_failed().unwrap();
        assert_eq!(g.state, GoalState::Failed);
    }

    #[test]
    fn full_lifecycle_build_failure() {
        let mut g = Goal::new(fake_sp("top.drv"), make_drv());

        g.inspect(vec![]).unwrap();
        g.mark_building().unwrap();
        g.mark_build_failed().unwrap();
        assert_eq!(g.state, GoalState::Failed);
    }

    #[test]
    fn failed_goal_rejects_further_transitions() {
        let mut g = Goal::new(fake_sp("top.drv"), make_drv());
        g.inspect(vec!["d.drv".into()]).unwrap();
        g.notify_dep_failed().unwrap();
        assert_eq!(g.state, GoalState::Failed);

        // Nothing should work on a Failed goal.
        assert!(g.inspect(vec![]).is_err());
        assert!(g.notify_dep_done().is_err());
        assert!(g.notify_dep_failed().is_err());
        assert!(g.mark_building().is_err());
        assert!(g.mark_done().is_err());
        assert!(g.mark_build_failed().is_err());
    }

    #[test]
    fn done_goal_rejects_further_transitions() {
        let mut g = Goal::new(fake_sp("d.drv"), make_drv());
        g.inspect(vec![]).unwrap();
        g.mark_building().unwrap();
        g.mark_done().unwrap();
        assert_eq!(g.state, GoalState::Done);

        assert!(g.inspect(vec![]).is_err());
        assert!(g.notify_dep_done().is_err());
        assert!(g.mark_building().is_err());
        assert!(g.mark_done().is_err());
        assert!(g.mark_build_failed().is_err());
    }

    // ── Registry: root tracking ─────────────────────────────────

    #[test]
    fn registry_root_done_and_failed_keys() {
        let mut reg = GoalRegistry::new();

        // Root that succeeds.
        let sp_ok = fake_sp("ok.drv");
        let key_ok = sp_ok.to_absolute_path();
        let mut g_ok = Goal::new_root(sp_ok, make_drv());
        g_ok.inspect(vec![]).unwrap();
        g_ok.mark_building().unwrap();
        g_ok.mark_done().unwrap();
        reg.insert(key_ok.clone(), g_ok).unwrap();

        // Root that fails.
        let sp_fail = fake_sp("fail.drv");
        let key_fail = sp_fail.to_absolute_path();
        let mut g_fail = Goal::new_root(sp_fail, make_drv());
        g_fail.inspect(vec![]).unwrap();
        g_fail.mark_building().unwrap();
        g_fail.mark_build_failed().unwrap();
        reg.insert(key_fail.clone(), g_fail).unwrap();

        // Non-root that succeeds (should not appear in root lists).
        let sp_nr = fake_sp("nonroot.drv");
        let key_nr = sp_nr.to_absolute_path();
        let mut g_nr = Goal::new(sp_nr, make_drv());
        g_nr.inspect(vec![]).unwrap();
        g_nr.mark_building().unwrap();
        g_nr.mark_done().unwrap();
        reg.insert(key_nr, g_nr).unwrap();

        let done = reg.root_done_keys();
        assert_eq!(done.len(), 1);
        assert!(done.contains(&key_ok));

        let failed = reg.root_failed_keys();
        assert_eq!(failed.len(), 1);
        assert!(failed.contains(&key_fail));

        assert!(reg.all_roots_terminal());
    }

    // ── Dynamic derivation (AwaitingDerivation) ────────────────

    #[test]
    fn new_awaiting_goal_state() {
        let g = Goal::new_awaiting(fake_sp("dyn.drv"), "/nix/store/xxx-producer.drv".into());
        assert_eq!(g.state, GoalState::AwaitingDerivation);
        assert!(g.derivation.is_none());
        assert!(!g.is_root);
        assert_eq!(g.producer_key.as_deref(), Some("/nix/store/xxx-producer.drv"));
    }

    #[test]
    fn set_derivation_transitions_to_pending() {
        let mut g = Goal::new_awaiting(fake_sp("dyn.drv"), "producer-key".into());
        g.set_derivation(make_drv()).unwrap();
        assert_eq!(g.state, GoalState::Pending);
        assert!(g.derivation.is_some());
    }

    #[test]
    fn set_derivation_not_awaiting_errors() {
        let mut g = Goal::new(fake_sp("a.drv"), make_drv());
        assert!(g.set_derivation(make_drv()).is_err());
    }

    #[test]
    fn awaiting_full_lifecycle() {
        let mut g = Goal::new_awaiting(fake_sp("dyn.drv"), "prod".into());
        assert_eq!(g.state, GoalState::AwaitingDerivation);

        g.set_derivation(make_drv()).unwrap();
        assert_eq!(g.state, GoalState::Pending);

        g.inspect(vec![]).unwrap();
        assert_eq!(g.state, GoalState::Ready);

        g.mark_building().unwrap();
        g.mark_done().unwrap();
        assert_eq!(g.state, GoalState::Done);
    }

    #[test]
    fn awaiting_with_deps_lifecycle() {
        let mut g = Goal::new_awaiting(fake_sp("dyn.drv"), "prod".into());
        g.set_derivation(make_drv()).unwrap();
        g.inspect(vec!["dep.drv".into()]).unwrap();
        assert_eq!(g.state, GoalState::Waiting { remaining_deps: 1 });

        g.notify_dep_done().unwrap();
        assert_eq!(g.state, GoalState::Ready);
    }

    #[test]
    fn awaiting_dep_failure_goes_failed() {
        let mut g = Goal::new_awaiting(fake_sp("dyn.drv"), "prod".into());
        // Producer failure while still awaiting.
        g.notify_dep_failed().unwrap();
        assert_eq!(g.state, GoalState::Failed);
    }

    #[test]
    fn awaiting_rejects_inspect_before_set_derivation() {
        let mut g = Goal::new_awaiting(fake_sp("dyn.drv"), "prod".into());
        assert!(g.inspect(vec![]).is_err());
    }

    #[test]
    fn awaiting_rejects_mark_building() {
        let g = Goal::new_awaiting(fake_sp("dyn.drv"), "prod".into());
        let mut g = g;
        assert!(g.mark_building().is_err());
    }

    // ── Registry: awaiting_producer ───────────────────────────

    #[test]
    fn registry_awaiting_producer_finds_matching() {
        let mut reg = GoalRegistry::new();

        let sp1 = fake_sp("dyn1.drv");
        let g1 = Goal::new_awaiting(sp1.clone(), "producer-A".into());
        reg.insert(sp1.to_absolute_path(), g1).unwrap();

        let sp2 = fake_sp("dyn2.drv");
        let g2 = Goal::new_awaiting(sp2.clone(), "producer-A".into());
        reg.insert(sp2.to_absolute_path(), g2).unwrap();

        let sp3 = fake_sp("dyn3.drv");
        let g3 = Goal::new_awaiting(sp3.clone(), "producer-B".into());
        reg.insert(sp3.to_absolute_path(), g3).unwrap();

        let awaiting_a = reg.awaiting_producer("producer-A");
        assert_eq!(awaiting_a.len(), 2);

        let awaiting_b = reg.awaiting_producer("producer-B");
        assert_eq!(awaiting_b.len(), 1);

        let awaiting_c = reg.awaiting_producer("producer-C");
        assert!(awaiting_c.is_empty());
    }

    #[test]
    fn registry_awaiting_not_terminal() {
        let mut reg = GoalRegistry::new();
        let sp = fake_sp("dyn.drv");
        let mut g = Goal::new_awaiting(sp.clone(), "prod".into());
        g.is_root = true;
        reg.insert(sp.to_absolute_path(), g).unwrap();
        assert!(!reg.all_roots_terminal());
    }
}
