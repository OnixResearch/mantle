//! Adapters between resource-policy decisions and existing Mantle authorities.
//!
//! This module does not replace coordinator eligibility, fenced-attempt, lease,
//! action-result, or CAS checks. It consumes their explicit results.

// r[impl remote_builds.replayable_resource_selection]
// r[impl remote_builds.positive_oom_retry]
// r[impl remote_builds.authorized_result_sharing]
// r[impl remote_builds.resource_policy_rollout]

use std::collections::BTreeSet;

use crunch_build::distributed::RemoteAssignmentNonce;
use crunch_build::distributed::RemoteAttemptRetryPolicy;
use crunch_build::distributed::RemoteAttemptState;
use crunch_build::distributed::RemoteAttemptTimeFacts;
use crunch_build::distributed::RemoteJobId;
use crunch_build::distributed::plan_remote_attempt_assignment;
use crunch_resource_policy::MachineClass;
use crunch_resource_policy::ResourcePolicyError;
use crunch_resource_policy::ResourceRetryPlan;
use crunch_resource_policy::ResourceRetryRequest;
use crunch_resource_policy::ResourceSelectionDecision;
use crunch_resource_policy::ResultReuseDecision;
use crunch_resource_policy::ResultReuseFacts;
use crunch_resource_policy::ResultSharingPolicy;
use crunch_resource_policy::decide_result_reuse;
use crunch_resource_policy::plan_resource_retry;

const ACTION_RESULT_REF_PREFIX: &str = "mantle-action-result://blake3/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CoordinatorResourcePolicyPlan {
    pub(crate) decision: ResourceSelectionDecision,
    pub(crate) selected_worker_endpoint_id: String,
}

pub(crate) fn plan_coordinator_resource_policy(
    decision: ResourceSelectionDecision,
    machine_classes: Vec<MachineClass>,
    statically_eligible_endpoint_ids: Vec<String>,
) -> Result<CoordinatorResourcePolicyPlan, String> {
    let decision = crunch_resource_policy::validate_resource_selection_decision(decision)
        .map_err(|error| error.code().to_string())?;
    let eligible_endpoints = canonical_endpoint_set(statically_eligible_endpoint_ids)?;
    let selected_class = machine_classes
        .iter()
        .find(|class| class.class_id == decision.scheduled_class_id)
        .ok_or_else(|| "resource-policy-selected-class-missing".to_string())?;
    let mut endpoints = selected_class.endpoint_ids.clone();
    endpoints.sort();
    endpoints.dedup();
    let selected_worker_endpoint_id = endpoints
        .into_iter()
        .find(|endpoint| eligible_endpoints.contains(endpoint))
        .ok_or_else(|| "resource-policy-selected-class-has-no-statically-eligible-worker".to_string())?;
    debug_assert!(eligible_endpoints.contains(&selected_worker_endpoint_id));
    debug_assert_eq!(selected_class.class_id, decision.scheduled_class_id);
    Ok(CoordinatorResourcePolicyPlan {
        decision,
        selected_worker_endpoint_id,
    })
}

pub(crate) struct FencedResourceRetryInput {
    pub(crate) policy_request: ResourceRetryRequest,
    pub(crate) job_id: RemoteJobId,
    pub(crate) assignment_nonce: RemoteAssignmentNonce,
    pub(crate) previous_attempt: RemoteAttemptState,
    pub(crate) attempt_policy: RemoteAttemptRetryPolicy,
    pub(crate) time: RemoteAttemptTimeFacts,
    pub(crate) statically_eligible_endpoint_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FencedResourceRetryPlan {
    pub(crate) resource_retry: ResourceRetryPlan,
    pub(crate) next_attempt: RemoteAttemptState,
}

pub(crate) fn plan_fenced_resource_retry(input: FencedResourceRetryInput) -> Result<FencedResourceRetryPlan, String> {
    validate_retry_binding(&input)?;
    let classes = input.policy_request.eligible_classes.clone();
    let resource_retry = plan_resource_retry(input.policy_request).map_err(|error| error.code().to_string())?;
    let eligible_endpoints = canonical_endpoint_set(input.statically_eligible_endpoint_ids)?;
    let worker_endpoint_id = retry_worker_endpoint(&resource_retry, &classes, &eligible_endpoints)?;
    let next_attempt = plan_remote_attempt_assignment(
        &input.job_id,
        &worker_endpoint_id,
        input.assignment_nonce,
        Some(&input.previous_attempt),
        input.attempt_policy,
        input.time,
    )
    .map_err(|reason| reason.as_str().to_string())?;
    if next_attempt.fence_generation.get() != resource_retry.successor_fence_generation
        || next_attempt.attempt_id == input.previous_attempt.attempt_id
    {
        return Err("resource-policy-fenced-attempt-linkage-drift".to_string());
    }
    debug_assert!(next_attempt.fence_generation > input.previous_attempt.fence_generation);
    debug_assert_ne!(next_attempt.attempt_id, input.previous_attempt.attempt_id);
    Ok(FencedResourceRetryPlan {
        resource_retry,
        next_attempt,
    })
}

pub(crate) fn decide_result_reuse_after_strong_admission(
    policy: ResultSharingPolicy,
    mut facts: ResultReuseFacts,
    controls: crunch_resource_policy::ResourceFeatureControls,
    strong_plan: &crunch_action_result_core::StrongReusePlan,
) -> Result<ResultReuseDecision, ResourcePolicyError> {
    let expected_result_ref = format!("{ACTION_RESULT_REF_PREFIX}{}", facts.producer_result_blake3);
    facts.strong_reuse_admitted = strong_plan.conflict_class.is_none()
        && strong_plan.selected_result_ref.as_deref() == Some(expected_result_ref.as_str());
    let decision = decide_result_reuse(policy, facts, controls)?;
    debug_assert!(!decision.usable || strong_plan.conflict_class.is_none());
    debug_assert!(!decision.usable || strong_plan.selected_result_ref.as_deref() == Some(expected_result_ref.as_str()));
    Ok(decision)
}

fn canonical_endpoint_set(values: Vec<String>) -> Result<BTreeSet<String>, String> {
    let endpoint_count =
        u32::try_from(values.len()).map_err(|_| "resource-policy-static-endpoint-set-invalid".to_string())?;
    if endpoint_count == 0 || endpoint_count > crunch_resource_policy::MAX_CLASS_ENDPOINTS {
        return Err("resource-policy-static-endpoint-set-invalid".to_string());
    }
    let mut endpoints = BTreeSet::new();
    for value in values {
        if value.is_empty() || value.chars().any(char::is_control) || !endpoints.insert(value) {
            return Err("resource-policy-static-endpoint-set-invalid".to_string());
        }
    }
    debug_assert!(!endpoints.is_empty());
    debug_assert!(
        u32::try_from(endpoints.len()).is_ok_and(|count| count <= crunch_resource_policy::MAX_CLASS_ENDPOINTS)
    );
    Ok(endpoints)
}

fn validate_retry_binding(input: &FencedResourceRetryInput) -> Result<(), String> {
    if input.previous_attempt.job_id != input.job_id
        || input.previous_attempt.attempt_id.as_str() != input.policy_request.predecessor_attempt_id
        || input.previous_attempt.fence_generation.get() != input.policy_request.predecessor_fence_generation
    {
        return Err("resource-policy-retry-predecessor-binding-invalid".to_string());
    }
    input.attempt_policy.validate().map_err(|reason| reason.as_str().to_string())?;
    debug_assert_eq!(input.previous_attempt.job_id, input.job_id);
    debug_assert_eq!(input.previous_attempt.fence_generation.get(), input.policy_request.predecessor_fence_generation);
    Ok(())
}

fn retry_worker_endpoint(
    retry: &ResourceRetryPlan,
    classes: &[MachineClass],
    statically_eligible: &BTreeSet<String>,
) -> Result<String, String> {
    let class = classes
        .iter()
        .find(|class| class.class_id == retry.next_machine_class_id)
        .ok_or_else(|| "resource-policy-retry-class-missing".to_string())?;
    let mut endpoints = class.endpoint_ids.clone();
    endpoints.sort();
    endpoints.dedup();
    let endpoint = endpoints
        .into_iter()
        .find(|endpoint| statically_eligible.contains(endpoint))
        .ok_or_else(|| "resource-policy-retry-class-has-no-statically-eligible-worker".to_string())?;
    debug_assert!(statically_eligible.contains(&endpoint));
    debug_assert_eq!(class.class_id, retry.next_machine_class_id);
    Ok(endpoint)
}

#[cfg(test)]
mod tests {
    use crunch_build::distributed::RemoteAttemptPhase;
    use crunch_build::distributed::RemoteFenceGeneration;
    use crunch_resource_policy::*;

    use super::*;

    const NOW_UNIX_S: u64 = 10_000;
    const DEADLINE_UNIX_S: u64 = 20_000;
    const CPU_UNITS: u32 = 2;
    const SMALL_MEMORY_BYTES: u64 = 1_024;
    const LARGE_MEMORY_BYTES: u64 = 2_048;
    const SCRATCH_BYTES: u64 = 1_024;
    const SMALL_CHARGE_UNITS: u64 = 100;
    const LARGE_CHARGE_UNITS: u64 = 200;
    const QUOTA_UNITS: u64 = 1_000;
    const SMALL_CLASS_ORDINAL: u32 = 1;
    const LARGE_CLASS_ORDINAL: u32 = 2;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, BLAKE3_HEX_CHARS).collect()
    }

    #[allow(
        tigerstyle::ambiguous_params,
        reason = "fixture memory and charge units use named constants at every call site"
    )]
    fn class(id: &str, ordinal: u32, memory: u64, charge: u64) -> MachineClass {
        MachineClass {
            class_id: id.into(),
            endpoint_ids: vec![format!("worker-{id}")],
            ordinal,
            architecture: "x86_64".into(),
            platform: "x86_64-linux".into(),
            kvm_available: true,
            trust_tier: "trusted".into(),
            isolation: "microvm".into(),
            features: Vec::new(),
            capacity: ResourceQuantities {
                cpu_units: CPU_UNITS,
                memory_bytes: memory,
                scratch_bytes: SCRATCH_BYTES,
            },
            reservation_charge_units: charge,
            available: true,
            onixos_source_blake3: digest('a'),
        }
    }

    fn classes() -> Vec<MachineClass> {
        vec![
            class("small", SMALL_CLASS_ORDINAL, SMALL_MEMORY_BYTES, SMALL_CHARGE_UNITS),
            class("large", LARGE_CLASS_ORDINAL, LARGE_MEMORY_BYTES, LARGE_CHARGE_UNITS),
        ]
    }

    fn large_decision() -> ResourceSelectionDecision {
        let family = derive_action_family_identity(ActionFamilyInput {
            request_kind: "derivation".into(),
            system: "x86_64-linux".into(),
            builder_class: "rust".into(),
            sandbox_mode: "microvm".into(),
            network_mode: "none".into(),
            required_features: Vec::new(),
            semantic_accelerator_classes: Vec::new(),
        })
        .unwrap();
        select_resource_class(ResourceSelectionRequest {
            now_unix_s: NOW_UNIX_S,
            action_family: family,
            declared: DeclaredResourceRequirements {
                minima: ResourceQuantities {
                    cpu_units: CPU_UNITS,
                    memory_bytes: LARGE_MEMORY_BYTES,
                    scratch_bytes: SCRATCH_BYTES,
                },
                platform: PlatformRequirements {
                    architecture: "x86_64".into(),
                    platform: "x86_64-linux".into(),
                    kvm_required: true,
                    trust_tier: "trusted".into(),
                    isolation: "microvm".into(),
                    required_features: Vec::new(),
                },
            },
            machine_classes: classes(),
            quota: QuotaFacts {
                project_remaining_units: QUOTA_UNITS,
                account_remaining_units: QUOTA_UNITS,
            },
            observations: Vec::new(),
            policy: ResourceSelectionPolicy::default(),
            controls: ResourceFeatureControls {
                mode: ResourcePolicyMode::Enforce,
                project_opted_in: true,
                historical_selection_enabled: false,
                oom_retry_enabled: false,
                quota_enforcement_enabled: false,
                result_sharing_enabled: false,
            },
        })
        .unwrap()
    }

    #[test]
    fn coordinator_adapter_intersects_policy_with_static_eligibility() {
        let plan = plan_coordinator_resource_policy(large_decision(), classes(), vec![
            "worker-small".into(),
            "worker-large".into(),
        ])
        .unwrap();
        assert_eq!(plan.selected_worker_endpoint_id, "worker-large");
        assert_eq!(plan.decision.scheduled_class_id, "large");

        assert!(plan_coordinator_resource_policy(large_decision(), classes(), vec!["worker-small".into()],).is_err());
    }

    fn initial_attempt() -> RemoteAttemptState {
        plan_remote_attempt_assignment(
            &RemoteJobId::new("job-one").unwrap(),
            "worker-small",
            RemoteAssignmentNonce::new(digest('c')).unwrap(),
            None,
            RemoteAttemptRetryPolicy::default(),
            RemoteAttemptTimeFacts {
                now_unix_s: NOW_UNIX_S,
                failure_observed_unix_s: NOW_UNIX_S,
                overall_deadline_unix_s: DEADLINE_UNIX_S,
            },
        )
        .unwrap()
    }

    fn retry_request(previous: &RemoteAttemptState) -> ResourceRetryRequest {
        ResourceRetryRequest {
            predecessor_attempt_id: previous.attempt_id.as_str().to_string(),
            predecessor_fence_generation: previous.fence_generation.get(),
            current_machine_class_id: "small".into(),
            oom_evidence: OomEvidence {
                category: OomEvidenceCategory::LinuxCgroupV2OomKill,
                platform: "x86_64-linux".into(),
                evidence_ref: Some("cgroup-v2:oom-kill:one".into()),
                trusted: true,
            },
            retry_history: vec![RetryHistoryEntry {
                attempt_id: previous.attempt_id.as_str().to_string(),
                machine_class_id: "small".into(),
                charge_units: SMALL_CHARGE_UNITS,
                wall_time_ms: NOW_UNIX_S,
            }],
            eligible_classes: classes(),
            quota: QuotaFacts {
                project_remaining_units: QUOTA_UNITS,
                account_remaining_units: QUOTA_UNITS,
            },
            policy: ResourceRetryPolicy::default(),
            controls: ResourceFeatureControls {
                mode: ResourcePolicyMode::Enforce,
                project_opted_in: true,
                historical_selection_enabled: true,
                oom_retry_enabled: true,
                quota_enforcement_enabled: true,
                result_sharing_enabled: true,
            },
        }
    }

    #[test]
    fn retry_adapter_uses_existing_assignment_core_for_new_fence_and_attempt() {
        let mut previous = initial_attempt();
        previous.phase = RemoteAttemptPhase::Failed;
        let input = FencedResourceRetryInput {
            policy_request: retry_request(&previous),
            job_id: previous.job_id.clone(),
            assignment_nonce: RemoteAssignmentNonce::new(digest('d')).unwrap(),
            previous_attempt: previous.clone(),
            attempt_policy: RemoteAttemptRetryPolicy::default(),
            time: RemoteAttemptTimeFacts {
                now_unix_s: NOW_UNIX_S,
                failure_observed_unix_s: NOW_UNIX_S,
                overall_deadline_unix_s: DEADLINE_UNIX_S,
            },
            statically_eligible_endpoint_ids: vec!["worker-large".into()],
        };
        let plan = plan_fenced_resource_retry(input).unwrap();
        assert_eq!(
            plan.next_attempt.fence_generation,
            RemoteFenceGeneration::new(plan.resource_retry.successor_fence_generation).unwrap()
        );
        assert_ne!(plan.next_attempt.attempt_id, previous.attempt_id);
        assert_eq!(plan.resource_retry.next_machine_class_id, "large");
    }

    fn sharing_policy() -> ResultSharingPolicy {
        ResultSharingPolicy {
            schema: RESULT_SHARING_POLICY_SCHEMA.into(),
            policy_id: "sharing-v1".into(),
            scope: SharingScopeKind::Private,
            project_id: None,
            allowed_producers: Vec::new(),
            allowed_consumers: Vec::new(),
        }
    }

    fn sharing_facts() -> ResultReuseFacts {
        ResultReuseFacts {
            producer_project_id: "project-a".into(),
            consumer_project_id: "project-a".into(),
            producer_result_blake3: digest('e'),
            consumer_request_blake3: digest('f'),
            request_identity_matches: true,
            action_identity_matches: true,
            producer_signature_trusted: true,
            policy_compatible: true,
            platform_compatible: true,
            output_identity_verified: true,
            cas_available: true,
            strong_reuse_admitted: false,
            authorization_decision_blake3: digest('a'),
        }
    }

    fn sharing_controls() -> ResourceFeatureControls {
        ResourceFeatureControls {
            mode: ResourcePolicyMode::Enforce,
            project_opted_in: true,
            historical_selection_enabled: false,
            oom_retry_enabled: false,
            quota_enforcement_enabled: false,
            result_sharing_enabled: true,
        }
    }

    #[test]
    fn sharing_adapter_cannot_bypass_existing_strong_result_admission() {
        let facts = sharing_facts();
        let selected_ref = format!("{ACTION_RESULT_REF_PREFIX}{}", facts.producer_result_blake3);
        let admitted = crunch_action_result_core::StrongReusePlan {
            selected_result_ref: Some(selected_ref),
            conflict_class: None,
            admitted_output_set_digests_blake3: vec![digest('b')],
            candidate_decisions: Vec::new(),
            non_claims: Vec::new(),
        };
        let decision =
            decide_result_reuse_after_strong_admission(sharing_policy(), facts.clone(), sharing_controls(), &admitted)
                .unwrap();
        assert!(decision.usable);
        assert!(decision.evidence_link.is_some());

        let conflict = crunch_action_result_core::StrongReusePlan {
            conflict_class: Some("conflicting-action-results".into()),
            ..admitted
        };
        let denied =
            decide_result_reuse_after_strong_admission(sharing_policy(), facts, sharing_controls(), &conflict).unwrap();
        assert_eq!(denied.reason, ReuseReasonCode::StrongReuseRejected);
        assert!(!denied.usable);
    }
}
