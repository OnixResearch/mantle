use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;

use crate::MAX_CHARGE_UNITS;
use crate::MAX_MACHINE_CLASSES;
use crate::MAX_MEASUREMENT_MILLISECONDS;
use crate::MAX_RETRY_HISTORY;
use crate::MachineClass;
use crate::OomEvidence;
use crate::OomEvidenceCategory;
use crate::RESOURCE_POLICY_NON_CLAIM;
use crate::RESOURCE_RETRY_PLAN_SCHEMA;
use crate::RESOURCE_RETRY_POLICY_SCHEMA;
use crate::RETRY_DOMAIN;
use crate::ResourcePolicyError;
use crate::ResourcePolicyMode;
use crate::ResourceRetryPlan;
use crate::ResourceRetryPolicy;
use crate::ResourceRetryRequest;
use crate::RetryReasonCode;
use crate::bounded_count;
use crate::canonical_blake3;
use crate::validate_digest;
use crate::validate_id;

pub fn plan_resource_retry(mut request: ResourceRetryRequest) -> Result<ResourceRetryPlan, ResourcePolicyError> {
    validate_retry_request(&request)?;
    require_retry_enabled(request.controls)?;
    require_positive_oom(&request.oom_evidence)?;
    request
        .eligible_classes
        .sort_by(|left, right| left.ordinal.cmp(&right.ordinal).then(left.class_id.cmp(&right.class_id)));
    let current = request
        .eligible_classes
        .iter()
        .find(|class| class.class_id == request.current_machine_class_id)
        .ok_or(ResourcePolicyError::RetryRejected(RetryReasonCode::LargerClassUnavailable))?;
    if !oom_matches_platform(request.oom_evidence.category, &current.platform) {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::OomPlatformMismatch));
    }
    let retry_count = u32::try_from(request.retry_history.len())
        .map_err(|_| ResourcePolicyError::RetryRejected(RetryReasonCode::RetryLimitExhausted))?;
    if retry_count >= request.policy.retry_count_max {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::RetryLimitExhausted));
    }
    let next = next_larger_class(current, &request.eligible_classes, request.policy.class_ordinal_max)?;
    let cumulative_charge_units = cumulative_charge(&request, next)?;
    let cumulative_wall_time_ms = cumulative_wall_time(&request)?;
    require_retry_quota(&request, next)?;
    let successor_fence_generation =
        request.predecessor_fence_generation.checked_add(1).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    let trigger_evidence_ref = request
        .oom_evidence
        .evidence_ref
        .clone()
        .ok_or(ResourcePolicyError::RetryRejected(RetryReasonCode::OomEvidenceMissing))?;
    let mut plan = ResourceRetryPlan {
        schema: RESOURCE_RETRY_PLAN_SCHEMA.to_string(),
        retry_intent_blake3: String::new(),
        predecessor_attempt_id: request.predecessor_attempt_id,
        predecessor_fence_generation: request.predecessor_fence_generation,
        successor_fence_generation,
        prior_machine_class_id: current.class_id.clone(),
        next_machine_class_id: next.class_id.clone(),
        trigger_evidence_ref,
        cumulative_charge_units,
        cumulative_wall_time_ms,
        new_fenced_attempt_required: true,
        reason: RetryReasonCode::PositiveOomEvidence,
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    };
    plan.retry_intent_blake3 = retry_plan_identity(&plan)?;
    debug_assert!(plan.successor_fence_generation > plan.predecessor_fence_generation);
    debug_assert_ne!(plan.prior_machine_class_id, plan.next_machine_class_id);
    Ok(plan)
}

pub fn validate_resource_retry_plan(plan: ResourceRetryPlan) -> Result<ResourceRetryPlan, ResourcePolicyError> {
    if plan.schema != RESOURCE_RETRY_PLAN_SCHEMA || plan.non_claim != RESOURCE_POLICY_NON_CLAIM {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    if !plan.new_fenced_attempt_required || plan.reason != RetryReasonCode::PositiveOomEvidence {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_digest(&plan.retry_intent_blake3)?;
    validate_id(&plan.predecessor_attempt_id)?;
    validate_id(&plan.prior_machine_class_id)?;
    validate_id(&plan.next_machine_class_id)?;
    validate_id(&plan.trigger_evidence_ref)?;
    let expected_fence_generation =
        plan.predecessor_fence_generation.checked_add(1).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    if plan.successor_fence_generation != expected_fence_generation {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if plan.cumulative_charge_units > MAX_CHARGE_UNITS || plan.cumulative_wall_time_ms > MAX_MEASUREMENT_MILLISECONDS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if retry_plan_identity(&plan)? != plan.retry_intent_blake3 {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(plan.successor_fence_generation > plan.predecessor_fence_generation);
    debug_assert_ne!(plan.prior_machine_class_id, plan.next_machine_class_id);
    Ok(plan)
}

fn validate_retry_request(request: &ResourceRetryRequest) -> Result<(), ResourcePolicyError> {
    validate_retry_policy(&request.policy)?;
    validate_id(&request.predecessor_attempt_id)?;
    validate_id(&request.current_machine_class_id)?;
    bounded_count(request.retry_history.len(), MAX_RETRY_HISTORY, ResourcePolicyError::TooManyHistoryEntries)?;
    bounded_count(request.eligible_classes.len(), MAX_MACHINE_CLASSES, ResourcePolicyError::TooManyMachineClasses)?;
    if request.predecessor_fence_generation == 0 {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    let mut attempts = BTreeSet::new();
    for history in &request.retry_history {
        validate_id(&history.attempt_id)?;
        validate_id(&history.machine_class_id)?;
        if !attempts.insert(history.attempt_id.as_str()) {
            return Err(ResourcePolicyError::DuplicateIdentity);
        }
        if history.charge_units > MAX_CHARGE_UNITS || history.wall_time_ms > MAX_MEASUREMENT_MILLISECONDS {
            return Err(ResourcePolicyError::InvalidBounds);
        }
    }
    let mut classes = BTreeSet::new();
    for class in &request.eligible_classes {
        validate_retry_class(class)?;
        if !classes.insert(class.class_id.as_str()) {
            return Err(ResourcePolicyError::DuplicateIdentity);
        }
    }
    debug_assert!(request.predecessor_fence_generation > 0);
    debug_assert!(u32::try_from(request.retry_history.len()).is_ok_and(|count| count <= MAX_RETRY_HISTORY));
    Ok(())
}

fn validate_retry_policy(policy: &ResourceRetryPolicy) -> Result<(), ResourcePolicyError> {
    if policy.schema != RESOURCE_RETRY_POLICY_SCHEMA {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_id(&policy.policy_id)?;
    if policy.retry_count_max == 0 || policy.retry_count_max > MAX_RETRY_HISTORY {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.class_ordinal_max == 0 {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.cumulative_charge_units_max == 0 || policy.cumulative_charge_units_max > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.wall_time_ms_max == 0 || policy.wall_time_ms_max > MAX_MEASUREMENT_MILLISECONDS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(policy.retry_count_max <= MAX_RETRY_HISTORY);
    debug_assert!(policy.wall_time_ms_max <= MAX_MEASUREMENT_MILLISECONDS);
    Ok(())
}

fn validate_retry_class(class: &MachineClass) -> Result<(), ResourcePolicyError> {
    validate_id(&class.class_id)?;
    validate_id(&class.platform)?;
    validate_digest(&class.onixos_source_blake3)?;
    if class.ordinal == 0 || class.reservation_charge_units == 0 || class.reservation_charge_units > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(class.ordinal > 0);
    debug_assert!(class.reservation_charge_units > 0);
    Ok(())
}

fn require_retry_enabled(controls: crate::ResourceFeatureControls) -> Result<(), ResourcePolicyError> {
    if controls.mode != ResourcePolicyMode::Enforce || !controls.project_opted_in || !controls.oom_retry_enabled {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::RetryDisabled));
    }
    debug_assert!(controls.project_opted_in);
    debug_assert!(controls.oom_retry_enabled);
    Ok(())
}

fn require_positive_oom(evidence: &OomEvidence) -> Result<(), ResourcePolicyError> {
    if !evidence.trusted {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::OomEvidenceUntrusted));
    }
    if evidence.evidence_ref.is_none()
        || matches!(evidence.category, OomEvidenceCategory::AmbiguousExit | OomEvidenceCategory::None)
    {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::OomEvidenceMissing));
    }
    validate_id(&evidence.platform)?;
    debug_assert!(evidence.trusted);
    debug_assert!(evidence.evidence_ref.is_some());
    Ok(())
}

fn oom_matches_platform(category: OomEvidenceCategory, platform: &str) -> bool {
    match category {
        OomEvidenceCategory::LinuxCgroupV2OomKill => platform.contains("linux"),
        OomEvidenceCategory::WindowsJobObjectMemoryLimit => platform.contains("windows"),
        OomEvidenceCategory::DarwinJetsam => platform.contains("darwin"),
        OomEvidenceCategory::AmbiguousExit | OomEvidenceCategory::None => false,
    }
}

fn next_larger_class<'a>(
    current: &MachineClass,
    classes: &'a [MachineClass],
    ordinal_max: u32,
) -> Result<&'a MachineClass, ResourcePolicyError> {
    let next = classes
        .iter()
        .filter(|class| class.available)
        .filter(|class| class.platform == current.platform)
        .filter(|class| class.architecture == current.architecture)
        .find(|class| class.ordinal > current.ordinal)
        .ok_or(ResourcePolicyError::RetryRejected(RetryReasonCode::LargerClassUnavailable))?;
    if next.ordinal > ordinal_max {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::ClassLimitExceeded));
    }
    debug_assert!(next.ordinal > current.ordinal);
    debug_assert!(next.ordinal <= ordinal_max);
    Ok(next)
}

fn cumulative_charge(request: &ResourceRetryRequest, next: &MachineClass) -> Result<u64, ResourcePolicyError> {
    let current = request.retry_history.iter().try_fold(0_u64, |total, item| {
        total.checked_add(item.charge_units).ok_or(ResourcePolicyError::ArithmeticOverflow)
    })?;
    let total = current.checked_add(next.reservation_charge_units).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    if total > request.policy.cumulative_charge_units_max {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::ChargeLimitExceeded));
    }
    debug_assert!(total >= next.reservation_charge_units);
    debug_assert!(total <= request.policy.cumulative_charge_units_max);
    Ok(total)
}

fn cumulative_wall_time(request: &ResourceRetryRequest) -> Result<u64, ResourcePolicyError> {
    let total = request.retry_history.iter().try_fold(0_u64, |sum, item| {
        sum.checked_add(item.wall_time_ms).ok_or(ResourcePolicyError::ArithmeticOverflow)
    })?;
    if total > request.policy.wall_time_ms_max {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::WallTimeLimitExceeded));
    }
    debug_assert!(total <= request.policy.wall_time_ms_max);
    debug_assert!(total <= MAX_MEASUREMENT_MILLISECONDS);
    Ok(total)
}

fn require_retry_quota(request: &ResourceRetryRequest, next: &MachineClass) -> Result<(), ResourcePolicyError> {
    if !request.controls.quota_enforcement_enabled {
        return Ok(());
    }
    if next.reservation_charge_units > request.quota.project_remaining_units
        || next.reservation_charge_units > request.quota.account_remaining_units
    {
        return Err(ResourcePolicyError::RetryRejected(RetryReasonCode::QuotaDenied));
    }
    debug_assert!(next.reservation_charge_units <= request.quota.project_remaining_units);
    debug_assert!(next.reservation_charge_units <= request.quota.account_remaining_units);
    Ok(())
}

fn retry_plan_identity(plan: &ResourceRetryPlan) -> Result<String, ResourcePolicyError> {
    let mut material = plan.clone();
    material.retry_intent_blake3.clear();
    let identity = canonical_blake3(RETRY_DOMAIN, &material)?;
    debug_assert!(crate::valid_blake3(&identity));
    debug_assert!(material.retry_intent_blake3.is_empty());
    Ok(identity)
}
