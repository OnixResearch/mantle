//! Observation-only, invocation-scoped scheduler diagnostic collector.
//! The caller owns whether to allocate this; absence costs no scheduler work.
//! r[impl mantle.operator_diagnostics.causal_trace_records]

use std::collections::HashMap;

use mantle_causal_trace_core::ActionKind;
use mantle_causal_trace_core::Cause;
use mantle_causal_trace_core::InvalidTrace;
use mantle_causal_trace_core::MAX_ACTIONS;
use mantle_causal_trace_core::Record;
use mantle_causal_trace_core::SCHEMA;
pub use mantle_causal_trace_core::Trace;
use mantle_causal_trace_core::redact_diagnostic;
use mantle_causal_trace_core::validate;

#[derive(Default)]
struct GoalEvents {
    origin: Option<u32>,
    check: Option<u32>,
    attempt: Option<u32>,
    terminal: Option<u32>,
}

pub struct CausalTraceCollector {
    trace: Trace,
    goals: HashMap<String, GoalEvents>,
    failure: Option<InvalidTrace>,
    logical_store_prefix: Option<String>,
}

impl Default for CausalTraceCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalTraceCollector {
    pub fn new() -> Self {
        let seed = Record {
            action_id: 0,
            goal_blake3: None,
            kind: ActionKind::ExternalTrigger,
            cause: Some(Cause::ExternalTrigger),
            caused_by: Some(0),
            diagnostic: None,
        };
        Self {
            trace: Trace {
                schema: SCHEMA.to_string(),
                records: vec![seed],
            },
            goals: HashMap::new(),
            failure: None,
            logical_store_prefix: None,
        }
    }

    /// Worker goal keys use the Nix-compatible scheduler prefix even when the
    /// build report names a custom logical store. Hash the report's full
    /// logical derivation path, never the physical state directory.
    pub fn with_store_prefix(store_prefix: &str) -> Self {
        let mut collector = Self::new();
        collector.logical_store_prefix = Some(store_prefix.to_string());
        collector
    }

    fn goal_digest(&self, goal: &str) -> String {
        match (
            self.logical_store_prefix.as_deref(),
            goal.strip_prefix(nix_compat::store_path::STORE_DIR_WITH_SLASH),
        ) {
            (Some(store_prefix), Some(name)) => {
                let mut hasher = blake3::Hasher::new();
                hasher.update(store_prefix.as_bytes());
                hasher.update(b"/");
                hasher.update(name.as_bytes());
                hasher.finalize().to_hex().to_string()
            }
            _ => blake3::hash(goal.as_bytes()).to_hex().to_string(),
        }
    }

    fn emit(
        &mut self,
        goal: &str,
        kind: ActionKind,
        cause: Cause,
        parent: u32,
        diagnostic: Option<&str>,
    ) -> Option<u32> {
        if self.failure.is_some() {
            return None;
        }
        if self.trace.records.len() >= MAX_ACTIONS {
            self.failure = Some(InvalidTrace::TooManyActions);
            return None;
        }
        let action_id = self.trace.records.len() as u32;
        let goal_blake3 = self.goal_digest(goal);
        let record = Record {
            action_id,
            goal_blake3: Some(goal_blake3),
            kind,
            cause: Some(cause),
            caused_by: Some(parent),
            diagnostic: diagnostic.map(redact_diagnostic),
        };
        self.trace.records.push(record);
        debug_assert!(self.trace.records.len() <= MAX_ACTIONS);
        debug_assert!(self.trace.records.last().is_some_and(|record| record.goal_blake3.is_some()));
        Some(action_id)
    }

    fn origin(&mut self, goal: &str) -> u32 {
        match self.goals.get(goal).and_then(|events| events.origin) {
            Some(origin) => origin,
            None => {
                self.failure = Some(InvalidTrace::DanglingCause);
                0
            }
        }
    }

    pub fn root_required(&mut self, goal: &str) {
        if let Some(id) = self.emit(goal, ActionKind::RootRequirement, Cause::RootRequirement, 0, None) {
            self.goals.entry(goal.to_string()).or_default().origin.get_or_insert(id);
        }
    }

    pub fn dependency_required(&mut self, dependent: &str, dependency: &str) {
        let parent = self.origin(dependent);
        if self.failure.is_some() {
            return;
        }
        if let Some(id) = self.emit(dependency, ActionKind::DependencyReady, Cause::DependencyReady, parent, None) {
            self.goals.entry(dependency.to_string()).or_default().origin.get_or_insert(id);
        }
    }

    pub fn cache_checked(&mut self, goal: &str) {
        let retry_parent = self.goals.get(goal).and_then(|events| events.attempt).filter(|id| {
            usize::try_from(*id)
                .ok()
                .and_then(|index| self.trace.records.get(index))
                .is_some_and(|record| record.kind == ActionKind::Retry)
        });
        let parent = retry_parent.unwrap_or_else(|| self.origin(goal));
        if self.failure.is_some() {
            return;
        }
        if let Some(id) = self.emit(goal, ActionKind::CacheCheck, Cause::CacheDecision, parent, None) {
            self.goals.entry(goal.to_string()).or_default().check = Some(id);
        }
    }

    pub fn cache_decided(&mut self, goal: &str, hit: bool) {
        let parent = self.goals.get(goal).and_then(|events| events.check);
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        let kind = if hit {
            ActionKind::CacheHit
        } else {
            ActionKind::CacheMiss
        };
        if let Some(id) = self.emit(goal, kind, Cause::CacheDecision, parent, None) {
            self.goals.entry(goal.to_string()).or_default().attempt = Some(id);
        }
    }

    pub fn dispatched(&mut self, goal: &str) {
        let parent = self.goals.get(goal).and_then(|events| events.attempt.or(events.origin));
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        if let Some(id) = self.emit(goal, ActionKind::Dispatch, Cause::Dispatch, parent, None) {
            self.goals.entry(goal.to_string()).or_default().attempt = Some(id);
        }
    }

    pub fn completed(&mut self, goal: &str, cached: bool, diagnostic: Option<&str>) {
        let parent = self.goals.get(goal).and_then(|events| events.attempt);
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        let cause = if cached { Cause::CacheDecision } else { Cause::Dispatch };
        if let Some(id) = self.emit(goal, ActionKind::Succeeded, cause, parent, diagnostic) {
            self.goals.entry(goal.to_string()).or_default().terminal = Some(id);
        }
    }

    pub fn failed(&mut self, goal: &str, diagnostic: &str) {
        let parent = self.goals.get(goal).and_then(|events| events.attempt.max(events.check).or(events.origin));
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        let Some(parent_kind) = usize::try_from(parent)
            .ok()
            .and_then(|index| self.trace.records.get(index))
            .map(|record| record.kind)
        else {
            self.failure = Some(InvalidTrace::DanglingCause);
            return;
        };
        let cause = match parent_kind {
            ActionKind::Dispatch => Cause::Dispatch,
            ActionKind::Retry => Cause::Retry,
            ActionKind::CacheHit | ActionKind::CacheMiss | ActionKind::CacheCheck => Cause::CacheDecision,
            _ => Cause::DependencyReady,
        };
        if let Some(id) = self.emit(goal, ActionKind::Failed, cause, parent, Some(diagnostic)) {
            self.goals.entry(goal.to_string()).or_default().terminal = Some(id);
        }
    }

    pub fn dependency_failed(&mut self, dependent: &str, failed_dependency: &str, diagnostic: &str) {
        let parent = self.goals.get(failed_dependency).and_then(|events| events.terminal);
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        if let Some(id) = self.emit(dependent, ActionKind::Failed, Cause::DependencyReady, parent, Some(diagnostic)) {
            self.goals.entry(dependent.to_string()).or_default().terminal = Some(id);
        }
    }

    pub fn retried(&mut self, goal: &str) {
        let parent = self.goals.get(goal).and_then(|events| events.terminal);
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        if let Some(id) = self.emit(goal, ActionKind::Retry, Cause::Retry, parent, None) {
            self.goals.entry(goal.to_string()).or_default().attempt = Some(id);
        }
    }

    pub fn cancelled(&mut self, goal: &str) {
        let parent = self.goals.get(goal).and_then(|events| events.attempt.max(events.check).or(events.origin));
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        if let Some(id) = self.emit(goal, ActionKind::Cancellation, Cause::Cancellation, parent, None) {
            self.goals.entry(goal.to_string()).or_default().terminal = Some(id);
        }
    }

    pub fn cleaned_up(&mut self, goal: &str) {
        let parent = self.goals.get(goal).and_then(|events| events.terminal);
        let Some(parent) = parent else {
            self.failure = Some(InvalidTrace::MissingCause);
            return;
        };
        self.emit(goal, ActionKind::Cleanup, Cause::Cleanup, parent, None);
    }

    pub fn finish(self) -> Result<Trace, InvalidTrace> {
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        validate(&self.trace)?;
        Ok(self.trace)
    }
}

#[cfg(test)]
mod store_identity_tests {
    use super::*;

    #[test]
    fn scheduler_goal_key_hashes_configured_logical_identity() {
        let scheduler_key = "/nix/store/0123456789abcdef0123456789abcdef-dependency.drv";
        let logical_key = "/mantle/store/0123456789abcdef0123456789abcdef-dependency.drv";
        let mut collector = CausalTraceCollector::with_store_prefix("/mantle/store");
        collector.root_required(scheduler_key);
        collector.cache_checked(scheduler_key);
        collector.cache_decided(scheduler_key, true);
        collector.completed(scheduler_key, true, None);
        let expected = blake3::hash(logical_key.as_bytes()).to_hex().to_string();
        let trace = collector.finish().unwrap();
        assert!(trace.records.iter().skip(1).all(|record| record.goal_blake3.as_deref() == Some(expected.as_str())));
    }
}

#[cfg(test)]
mod tests {
    use mantle_causal_trace_core::chain_to_root;

    use super::*;

    #[test]
    fn real_collector_keeps_interleaved_failure_chain_and_redacts() {
        let mut collector = CausalTraceCollector::new();
        collector.root_required("/mantle/store/root-a.drv");
        collector.dependency_required("/mantle/store/root-a.drv", "/mantle/store/leaf.drv");
        collector.root_required("/mantle/store/root-b.drv");
        collector.cache_checked("/mantle/store/leaf.drv");
        collector.cache_decided("/mantle/store/leaf.drv", false);
        collector.dispatched("/mantle/store/leaf.drv");
        collector.failed("/mantle/store/leaf.drv", "authorization=LEAK /home/me/build");
        collector.dependency_failed("/mantle/store/root-a.drv", "/mantle/store/leaf.drv", "dependency failed");
        let trace = collector.finish().unwrap();
        assert_eq!(chain_to_root(&trace, 8).unwrap(), vec![8, 7, 6, 5, 4, 2, 1, 0]);
        assert_eq!(chain_to_root(&trace, 3).unwrap(), vec![3, 0]);
        let json = serde_json::to_string(&trace).unwrap();
        assert!(!json.contains("LEAK"));
        assert!(!json.contains("/home/me"));
        assert!(!json.contains("root-a.drv"));
    }
    #[test]
    fn cache_hit_names_admission_and_retry_cancellation_cleanup_are_typed() {
        let mut hit = CausalTraceCollector::new();
        hit.root_required("/mantle/store/cache-root.drv");
        hit.cache_checked("/mantle/store/cache-root.drv");
        hit.cache_decided("/mantle/store/cache-root.drv", true);
        hit.completed("/mantle/store/cache-root.drv", true, None);
        let trace = hit.finish().unwrap();
        assert_eq!(trace.records[3].caused_by, Some(2));
        assert_eq!(chain_to_root(&trace, 4).unwrap(), vec![4, 3, 2, 1, 0]);

        let mut attempt = CausalTraceCollector::new();
        attempt.root_required("/mantle/store/retry-root.drv");
        attempt.cache_checked("/mantle/store/retry-root.drv");
        attempt.cache_decided("/mantle/store/retry-root.drv", false);
        attempt.dispatched("/mantle/store/retry-root.drv");
        attempt.failed("/mantle/store/retry-root.drv", "retryable build failure");
        attempt.retried("/mantle/store/retry-root.drv");
        attempt.dispatched("/mantle/store/retry-root.drv");
        attempt.cancelled("/mantle/store/retry-root.drv");
        attempt.cleaned_up("/mantle/store/retry-root.drv");
        let trace = attempt.finish().unwrap();
        assert_eq!(chain_to_root(&trace, 9).unwrap(), vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn multi_hop_failure_names_immediate_failed_dependency() {
        let mut collector = CausalTraceCollector::new();
        let root = "/mantle/store/root.drv";
        let intermediate = "/mantle/store/intermediate.drv";
        let leaf = "/mantle/store/leaf.drv";
        collector.root_required(root);
        collector.dependency_required(root, intermediate);
        collector.dependency_required(intermediate, leaf);
        collector.cache_checked(leaf);
        collector.cache_decided(leaf, false);
        collector.dispatched(leaf);
        collector.failed(leaf, "leaf failed");
        collector.dependency_failed(intermediate, leaf, "dependency failed");
        collector.dependency_failed(root, intermediate, "dependency failed");
        let trace = collector.finish().unwrap();
        assert_eq!(chain_to_root(&trace, 9).unwrap(), vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn failed_retry_uses_latest_attempt_or_recheck_cause() {
        let goal = "/mantle/store/retry-failure.drv";
        let mut interrupted = CausalTraceCollector::new();
        interrupted.root_required(goal);
        interrupted.cache_checked(goal);
        interrupted.cache_decided(goal, false);
        interrupted.dispatched(goal);
        interrupted.failed(goal, "first failure");
        interrupted.retried(goal);
        interrupted.failed(goal, "retry could not start");
        let trace = interrupted.finish().unwrap();
        assert_eq!(trace.records[7].cause, Some(Cause::Retry));
        assert_eq!(chain_to_root(&trace, 7).unwrap(), vec![7, 6, 5, 4, 3, 2, 1, 0]);

        let mut recheck = CausalTraceCollector::new();
        recheck.root_required(goal);
        recheck.cache_checked(goal);
        recheck.cache_decided(goal, false);
        recheck.dispatched(goal);
        recheck.failed(goal, "first failure");
        recheck.retried(goal);
        recheck.cache_checked(goal);
        recheck.failed(goal, "recheck failed");
        let trace = recheck.finish().unwrap();
        assert_eq!(trace.records[8].cause, Some(Cause::CacheDecision));
        assert_eq!(chain_to_root(&trace, 8).unwrap(), vec![8, 7, 6, 5, 4, 3, 2, 1, 0]);

        let mut cancelled = CausalTraceCollector::new();
        cancelled.root_required(goal);
        cancelled.cache_checked(goal);
        cancelled.cache_decided(goal, false);
        cancelled.dispatched(goal);
        cancelled.failed(goal, "first failure");
        cancelled.retried(goal);
        cancelled.cache_checked(goal);
        cancelled.cancelled(goal);
        cancelled.cleaned_up(goal);
        let trace = cancelled.finish().unwrap();
        assert_eq!(chain_to_root(&trace, 9).unwrap(), vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn emitted_action_cap_fails_closed_without_wrapping_identity() {
        let mut collector = CausalTraceCollector::new();
        let goal = "/mantle/store/bounded.drv";
        collector.root_required(goal);
        for _ in 0..MAX_ACTIONS {
            collector.cache_checked(goal);
        }
        assert_eq!(collector.finish().err(), Some(InvalidTrace::TooManyActions));
    }
}
