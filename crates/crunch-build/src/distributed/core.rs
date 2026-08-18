// r[impl realization_routing.distributed_core_boundary]
// r[impl reality] Pure, infrastructure-free distributed decision core for the
// realization-routing boundary.
//
// This module owns deterministic decisions over Mantle-owned bounded facts:
// - remote build-service request validation (vendor request is projected by
//   the shell into `RemoteBuildServiceRequestFacts`),
// - remote failure-to-fallback classification,
// - remote goal-attachment planning over plain goal and realization keys.
//
// The boundary contract (guarded by `scripts/check-distributed-core-boundary.rs`):
// - no vendor types: no `snix_*`, store implementations, PathInfo, or provider
//   types;
// - no async and no runtime: no `async fn`, no `tokio`, no `async_trait`;
// - no effects: no filesystem, environment, clock, process, or network I/O;
// - no provider ports and no adapter selection.
//
// The shell owns vendor projection, service calls, time, cancellation, retries,
// observation recording, and concrete adapter selection.

use serde::Deserialize;
use serde::Serialize;

/// The maximum number of ready remote goals a single attachment plan admits.
///
/// The shell and core must agree on this bound; it is the same policy bound as
/// `MAX_READY_REMOTE_GOALS` in the realization-routing shell.
pub const MAX_READY_REMOTE_GOALS: usize = 4096;

const CORE_REQUEST_VALIDATION_PHASE: &str = "request-validation";

/// Mantle-owned bounded facts about one remote build-service request. The
/// shell projects the vendor `snix_build::buildservice::BuildRequest` into
/// this value before any core decision runs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoteBuildServiceRequestFacts {
    /// The number of declared command arguments.
    pub command_args_count: u32,
    /// The number of declared output paths.
    pub outputs_count: u32,
}

/// Typed domain error for a rejected remote build-service request.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("remote build service {phase} failed: {reason}")]
pub struct RequestValidationError {
    pub phase: &'static str,
    pub reason: String,
}

/// Pure decision: is this bounded request shape admissible for remote
/// dispatch? The decision reads only the projected facts. It never touches
/// the vendor request, a store, a process, or a clock.
///
/// r[impl realization_routing.distributed_core_boundary.scenario.vendor_type]
pub fn validate_remote_build_service_request(
    facts: &RemoteBuildServiceRequestFacts,
) -> Result<(), RequestValidationError> {
    if facts.command_args_count == 0 {
        return Err(RequestValidationError {
            phase: CORE_REQUEST_VALIDATION_PHASE,
            reason: "remote-build-service-raw-eval-request".to_string(),
        });
    }
    if facts.outputs_count == 0 {
        return Err(RequestValidationError {
            phase: CORE_REQUEST_VALIDATION_PHASE,
            reason: "remote-build-service-outputs-empty".to_string(),
        });
    }
    Ok(())
}

/// Mantle-owned fallback policy. The shell projects its own policy choices
/// into this value before the core decides.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FallbackPolicy {
    Never,
    OnRemoteFailure,
}

/// The typed disposition the core selects for a remote dispatch failure.
/// The shell executes the return or fallback effect; it never re-decides.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FallbackDecision {
    ReturnFailure { phase: &'static str, reason: String },
    FallbackToLocal { phase: &'static str, reason: String },
}

/// Pure decision: classify one remote dispatch failure against the fallback
/// policy into exactly one typed disposition.
///
/// r[impl realization_routing.distributed_core_boundary.scenario.effect_call]
pub fn classify_failure(
    policy: FallbackPolicy,
    phase: &'static str,
    reason: &str,
) -> FallbackDecision {
    let reason = reason.to_string();
    match policy {
        FallbackPolicy::Never => FallbackDecision::ReturnFailure { phase, reason },
        FallbackPolicy::OnRemoteFailure => FallbackDecision::FallbackToLocal { phase, reason },
    }
}

/// One planned goal attachment. All keys are plain Mantle-owned strings; the
/// shell projects typed `RealizationKey` values at its own boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GoalAttachmentPlan {
    pub goal_key: String,
    pub realization_key: String,
    pub owner_goal_key: Option<String>,
}

/// Typed domain error for goal-attachment planning.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AttachmentError {
    #[error("too many ready remote goals; maximum is {max}")]
    TooManyReadyGoals { max: usize },
    #[error("an active owner map contained an empty goal key")]
    EmptyOwnerGoalKey,
    #[error("a ready goal carried an empty goal key")]
    EmptyGoalKey,
}

/// Pure decision: plan goal attachments with duplicate realization-key
/// suppression over plain Mantle-owned string facts.
///
/// The input order is the priority order; the plan preserves it. Running jobs
/// already owned by the caller arrive as `active_owner_by_key`. The decision
/// never awaits a scheduler or queries a worker.
///
/// r[impl realization_routing.distributed_core_boundary]
pub fn plan_goal_attachments(
    ready: &[(String, String)],
    active_owner_by_key: &std::collections::BTreeMap<String, String>,
) -> Result<Vec<GoalAttachmentPlan>, AttachmentError> {
    if ready.len() > MAX_READY_REMOTE_GOALS {
        return Err(AttachmentError::TooManyReadyGoals {
            max: MAX_READY_REMOTE_GOALS,
        });
    }
    let mut owners = active_owner_by_key.clone();
    let mut plans = Vec::with_capacity(ready.len());
    for (goal_key, realization_key) in ready {
        if goal_key.is_empty() {
            return Err(AttachmentError::EmptyGoalKey);
        }
        let owner_goal_key = owners.get(realization_key).cloned();
        match &owner_goal_key {
            None => {
                owners.insert(realization_key.clone(), goal_key.clone());
            }
            Some(existing) if existing.is_empty() => return Err(AttachmentError::EmptyOwnerGoalKey),
            Some(_) => {}
        }
        plans.push(GoalAttachmentPlan {
            goal_key: goal_key.clone(),
            realization_key: realization_key.clone(),
            owner_goal_key,
        });
    }
    debug_assert_eq!(plans.len(), ready.len());
    debug_assert!(plans.len() <= MAX_READY_REMOTE_GOALS);
    Ok(plans)
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[verify realization_routing.distributed_core_boundary]

    fn valid_request_facts() -> RemoteBuildServiceRequestFacts {
        RemoteBuildServiceRequestFacts {
            command_args_count: 2,
            outputs_count: 1,
        }
    }

    #[test]
    fn valid_request_facts_are_admitted() {
        assert!(validate_remote_build_service_request(&valid_request_facts()).is_ok());
    }

    #[test]
    fn raw_eval_request_is_rejected() {
        let facts = RemoteBuildServiceRequestFacts {
            command_args_count: 0,
            outputs_count: 1,
        };
        let error = validate_remote_build_service_request(&facts).expect_err("raw request rejected");
        assert_eq!(error.phase, CORE_REQUEST_VALIDATION_PHASE);
        assert_eq!(error.reason, "remote-build-service-raw-eval-request");
    }

    #[test]
    fn outputless_request_is_rejected() {
        let facts = RemoteBuildServiceRequestFacts {
            command_args_count: 2,
            outputs_count: 0,
        };
        let error = validate_remote_build_service_request(&facts).expect_err("outputless request rejected");
        assert_eq!(error.phase, CORE_REQUEST_VALIDATION_PHASE);
        assert_eq!(error.reason, "remote-build-service-outputs-empty");
    }

    #[test]
    fn never_policy_returns_failure() {
        let decision = classify_failure(FallbackPolicy::Never, "dispatch", "boom");
        assert!(matches!(decision, FallbackDecision::ReturnFailure { reason, .. } if reason == "boom"));
    }

    #[test]
    fn on_failure_policy_falls_back() {
        let decision = classify_failure(FallbackPolicy::OnRemoteFailure, "dispatch", "boom");
        assert!(matches!(decision, FallbackDecision::FallbackToLocal { reason, .. } if reason == "boom"));
    }

    #[test]
    fn failure_classification_preserves_phase_and_reason() {
        let decision = classify_failure(FallbackPolicy::Never, "request-validation", "reason-x");
        assert_eq!(
            decision,
            FallbackDecision::ReturnFailure {
                phase: "request-validation",
                reason: "reason-x".to_string()
            }
        );
    }

    #[test]
    fn duplicate_realization_key_attaches_to_owner() {
        let plans = plan_goal_attachments(
            &[("a".to_string(), "k1".to_string()), ("b".to_string(), "k1".to_string())],
            &std::collections::BTreeMap::new(),
        )
        .expect("attachments planned");
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].owner_goal_key, None);
        assert_eq!(plans[1].owner_goal_key.as_deref(), Some("a"));
    }

    #[test]
    fn active_owner_is_respected_and_not_overwritten() {
        let mut active = std::collections::BTreeMap::new();
        active.insert("k1".to_string(), "external-owner".to_string());
        let plans = plan_goal_attachments(
            &[("a".to_string(), "k1".to_string())],
            &active,
        )
        .expect("attachments planned");
        assert_eq!(plans[0].owner_goal_key.as_deref(), Some("external-owner"));
    }

    #[test]
    fn unbounded_ready_set_is_rejected() {
        let ready: Vec<(String, String)> = (0..MAX_READY_REMOTE_GOALS + 1)
            .map(|index| (format!("goal-{index}"), format!("key-{index}")))
            .collect();
        let error = plan_goal_attachments(&ready, &std::collections::BTreeMap::new())
            .expect_err("unbounded ready set rejected");
        assert!(matches!(
            error,
            AttachmentError::TooManyReadyGoals {
                max: MAX_READY_REMOTE_GOALS
            }
        ));
    }

    #[test]
    fn empty_goal_key_is_rejected() {
        let error = plan_goal_attachments(
            &[("".to_string(), "k1".to_string())],
            &std::collections::BTreeMap::new(),
        )
        .expect_err("empty goal key rejected");
        assert_eq!(error, AttachmentError::EmptyGoalKey);
    }

    #[test]
    fn empty_owner_inject_into_map_is_rejected() {
        let mut active = std::collections::BTreeMap::new();
        active.insert("k1".to_string(), String::new());
        let error = plan_goal_attachments(&[("a".to_string(), "k1".to_string())], &active)
            .expect_err("empty owner rejected");
        assert_eq!(error, AttachmentError::EmptyOwnerGoalKey);
    }

    #[test]
    fn attachment_order_preserves_input_priority() {
        let prepared = [("g1", "k1"), ("g2", "k2"), ("g3", "k3")];
        let ready: Vec<(String, String)> = prepared
            .iter()
            .map(|(goal, key)| (goal.to_string(), key.to_string()))
            .collect();
        let plans = plan_goal_attachments(&ready, &std::collections::BTreeMap::new()).expect("plans");
        let keys: Vec<&str> = plans.iter().map(|plan| plan.goal_key.as_str()).collect();
        assert_eq!(keys, ["g1", "g2", "g3"]);
    }

    #[test]
    fn core_decision_module_is_deterministic() {
        let facts = valid_request_facts();
        assert!(validate_remote_build_service_request(&facts).is_ok());
        assert_eq!(
            classify_failure(FallbackPolicy::OnRemoteFailure, "p", "r"),
            classify_failure(FallbackPolicy::OnRemoteFailure, "p", "r")
        );
    }
}
