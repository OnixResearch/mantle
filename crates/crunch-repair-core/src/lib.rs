#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! Pure final-NAR repair admission, transaction, report, and rollback decisions.
//!
//! This crate does not render NARs, read services, sign metadata, mutate files,
//! persist sidecars, or execute rollback. A standard-library shell supplies
//! verified facts and performs only the returned ordered intents.

extern crate alloc;

pub mod legacy_archive;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub const SHA256_DIGEST_BYTES: usize = 32;
pub const MAX_REPAIR_PATH_ID_BYTES: usize = 4_096;
pub const REPAIR_PLAN_ID_BYTES: usize = blake3::OUT_LEN;

const REPAIR_PLAN_DOMAIN: &[u8] = b"mantle.final-nar.repair-plan.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinalNarFacts {
    pub nar_size: u64,
    pub nar_sha256: [u8; SHA256_DIGEST_BYTES],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinalNarRepairPlanningFacts {
    pub recorded: FinalNarFacts,
    pub observed: FinalNarFacts,
    pub signature_count: u32,
    pub is_content_complete: bool,
    pub is_ca_path_identity_valid: bool,
    pub is_attestation_valid: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalNarRepairPlan {
    Current,
    Repair,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalNarRepairRejection {
    Unsigned,
    InvalidObservedNarFacts,
    IncompleteContent,
    InvalidCaPathIdentity,
    InvalidArtifactAttestation,
    InvalidPathIdentity,
    IdentityEncodingOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairMutationIntent {
    PersistPathInfo,
    PublishArtifactAttestation,
    VerifyPathInfo,
    VerifyArtifactAttestation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairRollbackIntent {
    RemoveStagedArtifactAttestation,
    RestorePathInfo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairSigningDisposition {
    PreserveExisting,
    ReplaceWithConfiguredSigner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairSidecarDisposition {
    Absent,
    Current,
    Refresh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairReportStatus {
    Current,
    WouldRepair,
    Repaired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairExecutionMode {
    Inspect,
    Execute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RepairPlanId([u8; REPAIR_PLAN_ID_BYTES]);

impl RepairPlanId {
    #[must_use]
    pub const fn into_bytes(self) -> [u8; REPAIR_PLAN_ID_BYTES] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepairTransactionRequest {
    pub path_id: String,
    pub planning_facts: FinalNarRepairPlanningFacts,
    pub artifact_attestation_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepairTransactionPlan {
    pub plan_id: RepairPlanId,
    pub final_nar_plan: FinalNarRepairPlan,
    pub signing_disposition: RepairSigningDisposition,
    pub sidecar_disposition: RepairSidecarDisposition,
    pub mutation_intents: Vec<RepairMutationIntent>,
    pub rollback_intents: Vec<RepairRollbackIntent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepairReportRequest {
    pub final_nar_plan: FinalNarRepairPlan,
    pub artifact_attestation_present: bool,
    pub execution_mode: RepairExecutionMode,
    pub execution_completed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RepairReportDecision {
    pub status: RepairReportStatus,
    pub sidecar_disposition: RepairSidecarDisposition,
    pub execution_requested: bool,
    pub mutated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairFailurePhase {
    StageArtifactAttestation,
    PersistPathInfo,
    PublishArtifactAttestation,
    VerifyPersistedState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepairFailureDecision {
    pub rollback_intents: Vec<RepairRollbackIntent>,
    pub success_must_be_denied: bool,
}

pub fn plan_final_nar_repair(
    facts: FinalNarRepairPlanningFacts,
) -> Result<FinalNarRepairPlan, FinalNarRepairRejection> {
    debug_assert_eq!(facts.recorded.nar_sha256.len(), SHA256_DIGEST_BYTES);
    debug_assert_eq!(facts.observed.nar_sha256.len(), SHA256_DIGEST_BYTES);
    if facts.signature_count == 0 {
        return Err(FinalNarRepairRejection::Unsigned);
    }
    if facts.observed.nar_size == 0 {
        return Err(FinalNarRepairRejection::InvalidObservedNarFacts);
    }
    if !facts.is_content_complete {
        return Err(FinalNarRepairRejection::IncompleteContent);
    }
    if !facts.is_ca_path_identity_valid {
        return Err(FinalNarRepairRejection::InvalidCaPathIdentity);
    }
    if !facts.is_attestation_valid {
        return Err(FinalNarRepairRejection::InvalidArtifactAttestation);
    }
    if facts.recorded == facts.observed {
        return Ok(FinalNarRepairPlan::Current);
    }
    Ok(FinalNarRepairPlan::Repair)
}

pub fn plan_repair_transaction(
    request: RepairTransactionRequest,
) -> Result<RepairTransactionPlan, FinalNarRepairRejection> {
    validate_path_id(&request.path_id)?;
    let final_nar_plan = plan_final_nar_repair(request.planning_facts)?;
    let sidecar_disposition = sidecar_disposition(final_nar_plan, request.artifact_attestation_present);
    let signing_disposition = signing_disposition(final_nar_plan);
    let mutation_intents = mutation_intents(final_nar_plan, request.artifact_attestation_present);
    let rollback_intents = rollback_intents(final_nar_plan, request.artifact_attestation_present);
    let plan_id = repair_plan_id(RepairPlanIdentityInput {
        path_id: &request.path_id,
        facts: request.planning_facts,
        artifact_attestation_present: request.artifact_attestation_present,
        plan: final_nar_plan,
        mutation_intents: &mutation_intents,
        rollback_intents: &rollback_intents,
    })?;
    debug_assert!(
        final_nar_plan == FinalNarRepairPlan::Repair || mutation_intents.is_empty(),
        "current metadata must not produce mutation intents"
    );
    Ok(RepairTransactionPlan {
        plan_id,
        final_nar_plan,
        signing_disposition,
        sidecar_disposition,
        mutation_intents,
        rollback_intents,
    })
}

#[must_use]
pub fn plan_repair_report(request: RepairReportRequest) -> RepairReportDecision {
    let is_execution_requested = matches!(request.execution_mode, RepairExecutionMode::Execute);
    let is_mutation_completed = is_execution_requested
        && request.execution_completed
        && matches!(request.final_nar_plan, FinalNarRepairPlan::Repair);
    let status = match (request.final_nar_plan, is_mutation_completed) {
        (FinalNarRepairPlan::Current, _) => RepairReportStatus::Current,
        (FinalNarRepairPlan::Repair, false) => RepairReportStatus::WouldRepair,
        (FinalNarRepairPlan::Repair, true) => RepairReportStatus::Repaired,
    };
    let sidecar_disposition = match (request.artifact_attestation_present, status) {
        (false, _) => RepairSidecarDisposition::Absent,
        (true, RepairReportStatus::Current) => RepairSidecarDisposition::Current,
        (true, RepairReportStatus::WouldRepair | RepairReportStatus::Repaired) => RepairSidecarDisposition::Refresh,
    };
    let decision = RepairReportDecision {
        status,
        sidecar_disposition,
        execution_requested: is_execution_requested,
        mutated: matches!(status, RepairReportStatus::Repaired),
    };
    debug_assert!(!decision.mutated || decision.execution_requested);
    debug_assert!(!decision.mutated || request.execution_completed);
    decision
}

#[must_use]
pub fn classify_repair_failure(phase: RepairFailurePhase) -> RepairFailureDecision {
    let rollback_intents = match phase {
        RepairFailurePhase::StageArtifactAttestation => vec![RepairRollbackIntent::RemoveStagedArtifactAttestation],
        RepairFailurePhase::PersistPathInfo
        | RepairFailurePhase::PublishArtifactAttestation
        | RepairFailurePhase::VerifyPersistedState => vec![
            RepairRollbackIntent::RestorePathInfo,
            RepairRollbackIntent::RemoveStagedArtifactAttestation,
        ],
    };
    RepairFailureDecision {
        rollback_intents,
        success_must_be_denied: true,
    }
}

fn validate_path_id(path_id: &str) -> Result<(), FinalNarRepairRejection> {
    let has_valid_segments =
        path_id.split('/').skip(1).all(|segment| !segment.is_empty() && segment != "." && segment != "..");
    let is_valid = !path_id.is_empty()
        && path_id.len() <= MAX_REPAIR_PATH_ID_BYTES
        && path_id.starts_with('/')
        && !path_id.ends_with('/')
        && !path_id.chars().any(char::is_control)
        && has_valid_segments;
    if !is_valid {
        return Err(FinalNarRepairRejection::InvalidPathIdentity);
    }
    Ok(())
}

const fn sidecar_disposition(plan: FinalNarRepairPlan, artifact_attestation_present: bool) -> RepairSidecarDisposition {
    match (plan, artifact_attestation_present) {
        (_, false) => RepairSidecarDisposition::Absent,
        (FinalNarRepairPlan::Current, true) => RepairSidecarDisposition::Current,
        (FinalNarRepairPlan::Repair, true) => RepairSidecarDisposition::Refresh,
    }
}

const fn signing_disposition(plan: FinalNarRepairPlan) -> RepairSigningDisposition {
    match plan {
        FinalNarRepairPlan::Current => RepairSigningDisposition::PreserveExisting,
        FinalNarRepairPlan::Repair => RepairSigningDisposition::ReplaceWithConfiguredSigner,
    }
}

fn mutation_intents(plan: FinalNarRepairPlan, artifact_attestation_present: bool) -> Vec<RepairMutationIntent> {
    if plan == FinalNarRepairPlan::Current {
        return Vec::new();
    }
    let mut intents = vec![RepairMutationIntent::PersistPathInfo];
    if artifact_attestation_present {
        intents.push(RepairMutationIntent::PublishArtifactAttestation);
    }
    intents.push(RepairMutationIntent::VerifyPathInfo);
    if artifact_attestation_present {
        intents.push(RepairMutationIntent::VerifyArtifactAttestation);
    }
    intents
}

fn rollback_intents(plan: FinalNarRepairPlan, artifact_attestation_present: bool) -> Vec<RepairRollbackIntent> {
    if plan == FinalNarRepairPlan::Current {
        return Vec::new();
    }
    let mut intents = vec![RepairRollbackIntent::RestorePathInfo];
    if artifact_attestation_present {
        intents.push(RepairRollbackIntent::RemoveStagedArtifactAttestation);
    }
    intents
}

struct RepairPlanIdentityInput<'a> {
    path_id: &'a str,
    facts: FinalNarRepairPlanningFacts,
    artifact_attestation_present: bool,
    plan: FinalNarRepairPlan,
    mutation_intents: &'a [RepairMutationIntent],
    rollback_intents: &'a [RepairRollbackIntent],
}

fn repair_plan_id(input: RepairPlanIdentityInput<'_>) -> Result<RepairPlanId, FinalNarRepairRejection> {
    debug_assert!(!input.path_id.is_empty());
    debug_assert!(input.path_id.len() <= MAX_REPAIR_PATH_ID_BYTES);
    debug_assert_eq!(input.facts.recorded.nar_sha256.len(), SHA256_DIGEST_BYTES);
    let mut hasher = blake3::Hasher::new();
    hasher.update(REPAIR_PLAN_DOMAIN);
    let path_byte_count =
        u64::try_from(input.path_id.len()).map_err(|_| FinalNarRepairRejection::IdentityEncodingOverflow)?;
    hasher.update(&path_byte_count.to_be_bytes());
    hasher.update(input.path_id.as_bytes());
    hash_final_nar_facts(&mut hasher, input.facts.recorded);
    hash_final_nar_facts(&mut hasher, input.facts.observed);
    hasher.update(&input.facts.signature_count.to_be_bytes());
    hasher.update(&[
        u8::from(input.facts.is_content_complete),
        u8::from(input.facts.is_ca_path_identity_valid),
        u8::from(input.facts.is_attestation_valid),
        u8::from(input.artifact_attestation_present),
        final_nar_plan_code(input.plan),
    ]);
    hash_mutation_intents(&mut hasher, input.mutation_intents)?;
    hash_rollback_intents(&mut hasher, input.rollback_intents)?;
    Ok(RepairPlanId(*hasher.finalize().as_bytes()))
}

fn hash_final_nar_facts(hasher: &mut blake3::Hasher, facts: FinalNarFacts) {
    hasher.update(&facts.nar_size.to_be_bytes());
    hasher.update(&facts.nar_sha256);
}

fn hash_mutation_intents(
    hasher: &mut blake3::Hasher,
    intents: &[RepairMutationIntent],
) -> Result<(), FinalNarRepairRejection> {
    let count = u64::try_from(intents.len()).map_err(|_| FinalNarRepairRejection::IdentityEncodingOverflow)?;
    hasher.update(&count.to_be_bytes());
    for intent in intents {
        hasher.update(&[mutation_intent_code(*intent)]);
    }
    Ok(())
}

fn hash_rollback_intents(
    hasher: &mut blake3::Hasher,
    intents: &[RepairRollbackIntent],
) -> Result<(), FinalNarRepairRejection> {
    let count = u64::try_from(intents.len()).map_err(|_| FinalNarRepairRejection::IdentityEncodingOverflow)?;
    hasher.update(&count.to_be_bytes());
    for intent in intents {
        hasher.update(&[rollback_intent_code(*intent)]);
    }
    Ok(())
}

const fn final_nar_plan_code(plan: FinalNarRepairPlan) -> u8 {
    match plan {
        FinalNarRepairPlan::Current => 0,
        FinalNarRepairPlan::Repair => 1,
    }
}

const fn mutation_intent_code(intent: RepairMutationIntent) -> u8 {
    match intent {
        RepairMutationIntent::PersistPathInfo => 0,
        RepairMutationIntent::PublishArtifactAttestation => 1,
        RepairMutationIntent::VerifyPathInfo => 2,
        RepairMutationIntent::VerifyArtifactAttestation => 3,
    }
}

const fn rollback_intent_code(intent: RepairRollbackIntent) -> u8 {
    match intent {
        RepairRollbackIntent::RemoveStagedArtifactAttestation => 0,
        RepairRollbackIntent::RestorePathInfo => 1,
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    const CURRENT_SIZE: u64 = 10;
    const CHANGED_SIZE: u64 = 20;
    const CURRENT_BYTE: u8 = 1;
    const CHANGED_BYTE: u8 = 2;
    const SIGNATURE_COUNT: u32 = 1;
    const TARGET_PATH: &str = "/nix/store/aaaaaaaa-target";
    const SECOND_TARGET_PATH: &str = "/nix/store/bbbbbbbb-target";

    const CURRENT: FinalNarFacts = FinalNarFacts {
        nar_size: CURRENT_SIZE,
        nar_sha256: [CURRENT_BYTE; SHA256_DIGEST_BYTES],
    };
    const CHANGED: FinalNarFacts = FinalNarFacts {
        nar_size: CHANGED_SIZE,
        nar_sha256: [CHANGED_BYTE; SHA256_DIGEST_BYTES],
    };

    fn facts(recorded: FinalNarFacts, observed: FinalNarFacts) -> FinalNarRepairPlanningFacts {
        FinalNarRepairPlanningFacts {
            recorded,
            observed,
            signature_count: SIGNATURE_COUNT,
            is_content_complete: true,
            is_ca_path_identity_valid: true,
            is_attestation_valid: true,
        }
    }

    #[test]
    fn current_facts_preserve_signing_and_sidecar_state() {
        let plan = plan_repair_transaction(RepairTransactionRequest {
            path_id: TARGET_PATH.to_string(),
            planning_facts: facts(CURRENT, CURRENT),
            artifact_attestation_present: true,
        })
        .expect("current facts must plan");
        assert_eq!(plan.final_nar_plan, FinalNarRepairPlan::Current);
        assert_eq!(plan.signing_disposition, RepairSigningDisposition::PreserveExisting);
        assert_eq!(plan.sidecar_disposition, RepairSidecarDisposition::Current);
        assert!(plan.mutation_intents.is_empty());
        assert!(plan.rollback_intents.is_empty());
    }

    #[test]
    fn changed_facts_produce_ordered_mutation_and_rollback_intents() {
        let plan = plan_repair_transaction(RepairTransactionRequest {
            path_id: TARGET_PATH.to_string(),
            planning_facts: facts(CURRENT, CHANGED),
            artifact_attestation_present: true,
        })
        .expect("repairable facts must plan");
        assert_eq!(plan.final_nar_plan, FinalNarRepairPlan::Repair);
        assert_eq!(plan.signing_disposition, RepairSigningDisposition::ReplaceWithConfiguredSigner);
        assert_eq!(plan.sidecar_disposition, RepairSidecarDisposition::Refresh);
        assert_eq!(plan.mutation_intents, vec![
            RepairMutationIntent::PersistPathInfo,
            RepairMutationIntent::PublishArtifactAttestation,
            RepairMutationIntent::VerifyPathInfo,
            RepairMutationIntent::VerifyArtifactAttestation,
        ]);
        assert_eq!(plan.rollback_intents, vec![
            RepairRollbackIntent::RestorePathInfo,
            RepairRollbackIntent::RemoveStagedArtifactAttestation,
        ]);
    }

    #[test]
    fn absent_sidecar_is_not_fabricated() {
        let plan = plan_repair_transaction(RepairTransactionRequest {
            path_id: TARGET_PATH.to_string(),
            planning_facts: facts(CURRENT, CHANGED),
            artifact_attestation_present: false,
        })
        .expect("repairable facts must plan");
        assert_eq!(plan.sidecar_disposition, RepairSidecarDisposition::Absent);
        assert!(!plan.mutation_intents.contains(&RepairMutationIntent::PublishArtifactAttestation));
        assert!(!plan.mutation_intents.contains(&RepairMutationIntent::VerifyArtifactAttestation));
    }

    #[test]
    fn path_identity_is_validated_and_bound_to_plan_identity() {
        let request = RepairTransactionRequest {
            path_id: TARGET_PATH.to_string(),
            planning_facts: facts(CURRENT, CHANGED),
            artifact_attestation_present: false,
        };
        let first = plan_repair_transaction(request.clone()).expect("canonical path must plan");
        let second = plan_repair_transaction(RepairTransactionRequest {
            path_id: SECOND_TARGET_PATH.to_string(),
            ..request
        })
        .expect("second canonical path must plan");
        assert_ne!(first.plan_id, second.plan_id);

        let invalid = plan_repair_transaction(RepairTransactionRequest {
            path_id: "relative/path".to_string(),
            planning_facts: facts(CURRENT, CHANGED),
            artifact_attestation_present: false,
        });
        assert_eq!(invalid, Err(FinalNarRepairRejection::InvalidPathIdentity));
    }

    #[test]
    fn every_unsafe_candidate_is_rejected() {
        let mut candidate = facts(CURRENT, CURRENT);
        candidate.signature_count = 0;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::Unsigned));

        candidate = facts(CURRENT, CURRENT);
        candidate.observed.nar_size = 0;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::InvalidObservedNarFacts));

        candidate = facts(CURRENT, CURRENT);
        candidate.is_content_complete = false;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::IncompleteContent));

        candidate = facts(CURRENT, CURRENT);
        candidate.is_ca_path_identity_valid = false;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::InvalidCaPathIdentity));

        candidate = facts(CURRENT, CURRENT);
        candidate.is_attestation_valid = false;
        assert_eq!(plan_final_nar_repair(candidate), Err(FinalNarRepairRejection::InvalidArtifactAttestation));
    }

    #[test]
    fn report_never_claims_mutation_before_completion() {
        let pending = plan_repair_report(RepairReportRequest {
            final_nar_plan: FinalNarRepairPlan::Repair,
            artifact_attestation_present: true,
            execution_mode: RepairExecutionMode::Execute,
            execution_completed: false,
        });
        assert_eq!(pending.status, RepairReportStatus::WouldRepair);
        assert!(!pending.mutated);

        let complete = plan_repair_report(RepairReportRequest {
            final_nar_plan: FinalNarRepairPlan::Repair,
            artifact_attestation_present: true,
            execution_mode: RepairExecutionMode::Execute,
            execution_completed: true,
        });
        assert_eq!(complete.status, RepairReportStatus::Repaired);
        assert!(complete.mutated);

        let impossible_inspection_completion = plan_repair_report(RepairReportRequest {
            final_nar_plan: FinalNarRepairPlan::Repair,
            artifact_attestation_present: true,
            execution_mode: RepairExecutionMode::Inspect,
            execution_completed: true,
        });
        assert_eq!(impossible_inspection_completion.status, RepairReportStatus::WouldRepair);
        assert!(!impossible_inspection_completion.mutated);
    }

    #[test]
    fn every_failure_phase_denies_success_and_has_rollback_policy() {
        let phases = [
            RepairFailurePhase::StageArtifactAttestation,
            RepairFailurePhase::PersistPathInfo,
            RepairFailurePhase::PublishArtifactAttestation,
            RepairFailurePhase::VerifyPersistedState,
        ];
        for phase in phases {
            let decision = classify_repair_failure(phase);
            assert!(decision.success_must_be_denied);
            assert!(!decision.rollback_intents.is_empty());
        }
    }
}
