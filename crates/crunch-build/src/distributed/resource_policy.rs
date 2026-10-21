//! Resource-policy adapter. Class inventory is rechecked through the existing
//! named-resource/lease admission path. This does not register or authenticate
//! workers, consume UCAN grants, or publish/reuse a result by itself.
use crunch_remote_core::resource_policy as core;
use serde::Serialize;

use super::remote_resources::RemoteResourceLease;
use super::remote_resources::RemoteResourceRequirements;
use super::remote_resources::RemoteWorkerResourceInventory;
use super::remote_resources::canonical_remote_resource_requirements;
use super::remote_resources::plan_remote_resource_availability;

pub struct CatalogCandidate<'a> {
    pub class: core::MachineClass,
    pub worker_endpoint_id: &'a str,
    pub inventory: &'a RemoteWorkerResourceInventory,
    pub live_leases: &'a [RemoteResourceLease],
}
#[derive(Debug, Clone, Serialize)]
pub struct PolicyEvaluation {
    pub classes: Vec<core::MachineClass>,
    pub decision: core::RolloutDecision,
    pub replay_input_blake3: String,
    pub non_claims: &'static [&'static str],
    pub replay_input_json: Vec<u8>,
}

/// Family identity hashes only these reviewed public canonical features. The
/// caller supplies an admitted public action kind, never env, claims or logs.
pub fn action_family_identity(public_action_kind: &str, features: &core::Features) -> Result<String, core::Reason> {
    if !core::valid_id(public_action_kind) || !features.valid() {
        return Err(core::Reason::InvalidInput);
    }
    let bytes = serde_json::to_vec(&(public_action_kind, features)).map_err(|_| core::Reason::InvalidInput)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-public-action-family-v1\0");
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Canonicalizes class/history order before both hashing and decision. The
/// output index refers to `PolicyEvaluation.classes`, not caller input order.
// r[impl remote_builds.replayable_resource_selection]
// r[impl remote_builds.resource_policy_rollout]
#[allow(
    clippy::too_many_arguments,
    reason = "Explicit replay and authority inputs match the resource policy boundary"
)]
pub fn evaluate_resource_policy(
    requirements: &RemoteResourceRequirements,
    features: &core::Features,
    family: &str,
    candidates: Vec<CatalogCandidate<'_>>,
    history: &mut [core::Observation],
    now_unix_s: u64,
    quota: core::QuotaFacts,
    policy: core::SelectionPolicy,
    controls: core::Controls,
) -> Result<PolicyEvaluation, core::Reason> {
    if candidates.len() > core::MAX_CLASSES || history.len() > core::MAX_HISTORY {
        return Err(core::Reason::BoundExceeded);
    }
    let requirements = canonical_remote_resource_requirements(requirements).map_err(|_| core::Reason::InvalidInput)?;
    let declared = core::Quantities {
        cpu_units: requirements.quantities.cpu_units,
        memory_bytes: requirements.quantities.memory_bytes,
        scratch_bytes: requirements.quantities.scratch_bytes,
    };
    let mut selected_requirements = requirements.clone();
    let mut classes = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let mut class = candidate.class;
        // Both advertised class and actual worker must fit all declared named
        // resources; recheck quantities again after historical selection below.
        class.existing_admission_compatible &= plan_remote_resource_availability(
            candidate.worker_endpoint_id,
            candidate.inventory,
            &requirements,
            candidate.live_leases,
        )
        .is_ok();
        selected_requirements.quantities.cpu_units = class.quantities.cpu_units;
        selected_requirements.quantities.memory_bytes = class.quantities.memory_bytes;
        selected_requirements.quantities.scratch_bytes = class.quantities.scratch_bytes;
        class.available &= plan_remote_resource_availability(
            candidate.worker_endpoint_id,
            candidate.inventory,
            &selected_requirements,
            candidate.live_leases,
        )
        .is_ok();
        classes.push(class);
    }
    classes.sort_by(|a, b| a.id.cmp(&b.id));
    history.sort_by(|a, b| a.attempt_id.as_str().cmp(b.attempt_id.as_str()));
    if history.windows(2).any(|pair| pair[0].attempt_id == pair[1].attempt_id) {
        return Err(core::Reason::HistoryDuplicate);
    }
    #[derive(Serialize)]
    struct Replay<'a> {
        schema: &'static str,
        requirements: &'a RemoteResourceRequirements,
        features: &'a core::Features,
        family: &'a str,
        classes: &'a [core::MachineClass],
        history: &'a [core::Observation],
        now_unix_s: u64,
        project_remaining_units: u64,
        account_remaining_units: u64,
        policy: core::SelectionPolicy,
        switches: [bool; 6],
    }
    let effective_quota = if controls.quota_active() {
        quota
    } else {
        core::QuotaFacts {
            project_remaining_units: u64::MAX,
            account_remaining_units: u64::MAX,
        }
    };
    let replay = Replay {
        schema: "mantle-resource-policy-replay-v1",
        requirements: &requirements,
        features,
        family,
        classes: &classes,
        history,
        now_unix_s,
        project_remaining_units: quota.project_remaining_units,
        account_remaining_units: quota.account_remaining_units,
        policy,
        switches: [
            controls.mode == core::SelectionMode::ProjectOptIn,
            controls.project_opted_in,
            controls.selection_enabled,
            controls.oom_retry_enabled,
            controls.quota_enforcement_enabled,
            controls.sharing_enabled,
        ],
    };
    let bytes = serde_json::to_vec(&replay).map_err(|_| core::Reason::InvalidInput)?;
    let replay_input_blake3 = blake3::hash(&bytes).to_hex().to_string();
    let static_decision = core::select(declared, features, family, &classes, &[], now_unix_s, effective_quota, policy);
    let historical_decision =
        core::select(declared, features, family, &classes, history, now_unix_s, effective_quota, policy);
    Ok(PolicyEvaluation {
        classes,
        decision: core::rollout(controls, static_decision, historical_decision),
        replay_input_blake3,
        replay_input_json: bytes,
        non_claims: &core::NON_CLAIMS,
    })
}

/// The callback MUST run the existing signed result and output/CAS admission
/// path. It is deliberately required even for same-project reuse; this gate
/// cannot turn a caller's CAS presence hint into admitted result authority.
/// Errors preserve producer records: this function never mutates storage.
pub fn admit_authorized_result<T, E>(
    controls: core::Controls,
    producer: &str,
    consumer: &str,
    result: &str,
    scope: Option<core::SharingScope<'_>>,
    checks: core::SharingChecks,
    existing_admission: impl FnOnce() -> Result<T, E>,
) -> Result<(T, core::SharingLink), SharingAdmissionError<E>> {
    let link = core::authorize_sharing(controls, producer, consumer, result, scope, checks)
        .map_err(SharingAdmissionError::Policy)?;
    let admitted = existing_admission().map_err(SharingAdmissionError::ExistingAdmission)?;
    Ok((admitted, link))
}
#[derive(Debug)]
pub enum SharingAdmissionError<E> {
    Policy(core::Reason),
    ExistingAdmission(E),
}

#[cfg(test)]
#[path = "resource_policy_tests.rs"]
mod tests;
