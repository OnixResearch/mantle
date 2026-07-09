use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

const MAX_REFERENCE_COUNT: usize = 16;
const MAX_RELEASE_PROFILE_COUNT: usize = 16;
const MAX_WORKER_RECEIPT_COUNT: usize = 64;
const MAX_CANCELLATION_OUTCOME_COUNT: usize = 64;
const MAX_CLAIM_TEXT_COUNT: usize = 32;
const PROOF_AUTHORITY_NON_CLAIM_FRAGMENT: &str = "not proof authority";

const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "proves release correctness",
    "proves reproducibility",
    "proves build soundness",
    "proves worker cleanup",
    "proves artifact truth",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceIntake {
    pub source_repository: String,
    pub intended_adaptation: String,
    pub license_posture: String,
    pub trust_boundary: String,
    pub non_claim_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTargetProfile {
    pub artifact_identity: String,
    pub release_role: String,
    pub signed_metadata_role: String,
    pub expected_tags: Vec<String>,
    pub actual_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseRepositoryProfile {
    pub profile_id: String,
    pub signed_metadata_identity: String,
    pub expiration_policy: String,
    pub trust_root: String,
    pub metadata_expired: bool,
    pub targets: Vec<ReleaseTargetProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerReceiptProfile {
    pub job_id: String,
    pub input_identity: String,
    pub target_profile: String,
    pub worker_identity: String,
    pub log_identity: String,
    pub artifact_identities: Vec<String>,
    pub cleanup_outcome: String,
    pub replayable_event_id: String,
    pub candidate_policy_override: bool,
    pub trusted_policy_approved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CancellationStage {
    Cooperative,
    OutputUpload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancellationOutcomeProfile {
    pub stage: CancellationStage,
    pub cleanup_attempted: bool,
    pub cleanup_succeeded: bool,
    pub final_receipt_persisted: bool,
    pub artifact_integrity_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OxideReleaseWorkerFixture {
    pub references: Vec<ReferenceIntake>,
    pub release_profiles: Vec<ReleaseRepositoryProfile>,
    pub worker_receipts: Vec<WorkerReceiptProfile>,
    pub cancellation_outcomes: Vec<CancellationOutcomeProfile>,
    pub claim_texts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OxideReleaseWorkerReport {
    pub accepted: bool,
    pub diagnostics: Vec<String>,
}

// r[impl mantle.verification_evidence.oxide_release_worker.reference_inventory]
// r[impl mantle.verification_evidence.oxide_release_worker.tuf_bundles]
// r[impl mantle.verification_evidence.oxide_release_worker.ephemeral_workers]
// r[impl mantle.verification_evidence.oxide_release_worker.cancel_safety]
pub fn validate_oxide_release_worker_fixture(fixture: &OxideReleaseWorkerFixture) -> OxideReleaseWorkerReport {
    let mut diagnostics = Vec::new();
    validate_bounded_count("references", fixture.references.len(), MAX_REFERENCE_COUNT, &mut diagnostics);
    validate_bounded_count(
        "release_profiles",
        fixture.release_profiles.len(),
        MAX_RELEASE_PROFILE_COUNT,
        &mut diagnostics,
    );
    validate_bounded_count(
        "worker_receipts",
        fixture.worker_receipts.len(),
        MAX_WORKER_RECEIPT_COUNT,
        &mut diagnostics,
    );
    validate_bounded_count(
        "cancellation_outcomes",
        fixture.cancellation_outcomes.len(),
        MAX_CANCELLATION_OUTCOME_COUNT,
        &mut diagnostics,
    );
    validate_bounded_count("claim_texts", fixture.claim_texts.len(), MAX_CLAIM_TEXT_COUNT, &mut diagnostics);
    validate_reference_inventory(&fixture.references, &mut diagnostics);
    validate_release_profiles(&fixture.release_profiles, &mut diagnostics);
    validate_worker_receipts(&fixture.worker_receipts, &mut diagnostics);
    validate_cancellation_outcomes(&fixture.cancellation_outcomes, &mut diagnostics);
    validate_claim_texts(&fixture.claim_texts, &mut diagnostics);
    OxideReleaseWorkerReport {
        accepted: diagnostics.is_empty(),
        diagnostics,
    }
}

fn validate_bounded_count(field_name: &str, count: usize, max_count: usize, diagnostics: &mut Vec<String>) {
    if count > max_count {
        diagnostics.push(format!("{field_name} exceeds maximum count {max_count}"));
    }
}

fn validate_reference_inventory(references: &[ReferenceIntake], diagnostics: &mut Vec<String>) {
    for reference in references {
        push_nonempty(&reference.source_repository, "reference.source_repository", diagnostics);
        push_nonempty(&reference.intended_adaptation, "reference.intended_adaptation", diagnostics);
        push_nonempty(&reference.license_posture, "reference.license_posture", diagnostics);
        push_nonempty(&reference.trust_boundary, "reference.trust_boundary", diagnostics);
        push_required_fragment(
            &reference.non_claim_boundary,
            PROOF_AUTHORITY_NON_CLAIM_FRAGMENT,
            "reference.non_claim_boundary",
            diagnostics,
        );
        push_no_overclaim(&reference.non_claim_boundary, "reference.non_claim_boundary", diagnostics);
    }
}

fn validate_release_profiles(profiles: &[ReleaseRepositoryProfile], diagnostics: &mut Vec<String>) {
    for profile in profiles {
        push_nonempty(&profile.profile_id, "release_profile.profile_id", diagnostics);
        push_nonempty(&profile.signed_metadata_identity, "release_profile.signed_metadata_identity", diagnostics);
        push_nonempty(&profile.expiration_policy, "release_profile.expiration_policy", diagnostics);
        push_nonempty(&profile.trust_root, "release_profile.trust_root", diagnostics);
        if profile.metadata_expired {
            diagnostics.push("release_profile metadata is expired".to_string());
        }
        if profile.targets.is_empty() {
            diagnostics.push("release_profile targets must not be empty".to_string());
        }
        for target in &profile.targets {
            validate_release_target(target, diagnostics);
        }
    }
}

fn validate_release_target(target: &ReleaseTargetProfile, diagnostics: &mut Vec<String>) {
    push_nonempty(&target.artifact_identity, "release_target.artifact_identity", diagnostics);
    push_nonempty(&target.release_role, "release_target.release_role", diagnostics);
    push_nonempty(&target.signed_metadata_role, "release_target.signed_metadata_role", diagnostics);
    if target.expected_tags.is_empty() {
        diagnostics.push("release_target expected_tags must not be empty".to_string());
    }
    for tag in &target.expected_tags {
        if !target.actual_tags.iter().any(|actual| actual == tag) {
            diagnostics.push(format!("release_target tag mismatch: missing {tag}"));
        }
    }
}

fn validate_worker_receipts(receipts: &[WorkerReceiptProfile], diagnostics: &mut Vec<String>) {
    for receipt in receipts {
        push_nonempty(&receipt.job_id, "worker_receipt.job_id", diagnostics);
        push_nonempty(&receipt.input_identity, "worker_receipt.input_identity", diagnostics);
        push_nonempty(&receipt.target_profile, "worker_receipt.target_profile", diagnostics);
        push_nonempty(&receipt.worker_identity, "worker_receipt.worker_identity", diagnostics);
        push_nonempty(&receipt.log_identity, "worker_receipt.log_identity", diagnostics);
        push_nonempty(&receipt.replayable_event_id, "worker_receipt.replayable_event_id", diagnostics);
        if receipt.artifact_identities.is_empty() {
            diagnostics.push("worker_receipt artifact_identities must not be empty".to_string());
        }
        if receipt.cleanup_outcome != "complete" {
            diagnostics.push("worker_receipt cleanup outcome is not complete".to_string());
        }
        if receipt.candidate_policy_override && !receipt.trusted_policy_approved {
            diagnostics.push("worker_receipt candidate policy override is untrusted".to_string());
        }
    }
}

fn validate_cancellation_outcomes(outcomes: &[CancellationOutcomeProfile], diagnostics: &mut Vec<String>) {
    for outcome in outcomes {
        if !outcome.cleanup_attempted {
            diagnostics.push("cancellation outcome did not attempt cleanup".to_string());
        }
        if !outcome.cleanup_succeeded {
            diagnostics.push("cancellation cleanup failed".to_string());
        }
        if !outcome.final_receipt_persisted {
            diagnostics.push("cancellation final receipt was not persisted".to_string());
        }
        if outcome.stage == CancellationStage::OutputUpload && !outcome.artifact_integrity_verified {
            diagnostics.push("mid-upload cancellation left artifact integrity unverified".to_string());
        }
    }
}

fn validate_claim_texts(claim_texts: &[String], diagnostics: &mut Vec<String>) {
    for claim_text in claim_texts {
        push_no_overclaim(claim_text, "claim_text", diagnostics);
    }
}

fn push_nonempty(value: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{field_name} must not be empty"));
    }
}

fn push_required_fragment(value: &str, fragment: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if !value.to_ascii_lowercase().contains(fragment) {
        diagnostics.push(format!("{field_name} missing required fragment {fragment:?}"));
    }
}

fn push_no_overclaim(value: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    let lower = value.to_ascii_lowercase();
    for fragment in OVERCLAIM_FRAGMENTS {
        if lower.contains(fragment) {
            diagnostics.push(format!("{field_name} contains overclaim fragment {fragment:?}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    // r[verify mantle.verification_evidence.oxide_release_worker.reference_inventory]
    // r[verify mantle.verification_evidence.oxide_release_worker.validation]
    #[test]
    fn accepts_valid_oxide_release_worker_fixture() {
        let fixture = valid_fixture();

        let report = validate_oxide_release_worker_fixture(&fixture);

        assert!(report.accepted);
        assert!(report.diagnostics.is_empty());
    }

    // r[verify mantle.verification_evidence.oxide_release_worker.tuf_bundles]
    // r[verify mantle.verification_evidence.oxide_release_worker.ephemeral_workers]
    // r[verify mantle.verification_evidence.oxide_release_worker.cancel_safety]
    // r[verify mantle.verification_evidence.oxide_release_worker.validation]
    #[test]
    fn rejects_invalid_oxide_release_worker_fixture_matrix() {
        let cases: [(&str, fn(&mut OxideReleaseWorkerFixture), &str); 6] = [
            ("expired metadata", expire_metadata, "expired"),
            ("tag mismatch", remove_target_tag, "tag mismatch"),
            ("untrusted policy override", add_untrusted_policy_override, "policy override"),
            ("cleanup failure", fail_worker_cleanup, "cleanup outcome"),
            ("mid-upload cancellation", fail_mid_upload_cancellation, "mid-upload"),
            ("overclaim text", add_overclaim_text, "overclaim"),
        ];

        for (name, mutate, expected) in cases {
            let mut fixture = valid_fixture();
            mutate(&mut fixture);

            let report = validate_oxide_release_worker_fixture(&fixture);

            assert!(!report.accepted, "{name} should be rejected");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{name} diagnostics should mention {expected:?}: {:?}",
                report.diagnostics
            );
        }
    }

    fn valid_fixture() -> OxideReleaseWorkerFixture {
        OxideReleaseWorkerFixture {
            references: vec![ReferenceIntake {
                source_repository: "https://github.com/oxidecomputer/buildomat".to_string(),
                intended_adaptation: "ephemeral worker receipt shape".to_string(),
                license_posture: "reference-only review".to_string(),
                trust_boundary: "Mantle policy remains authoritative".to_string(),
                non_claim_boundary: "reference is not proof authority for Mantle".to_string(),
            }],
            release_profiles: vec![ReleaseRepositoryProfile {
                profile_id: "mantle-tuf-style-release-v1".to_string(),
                signed_metadata_identity: "metadata.blake3:abc".to_string(),
                expiration_policy: "expires-before-use".to_string(),
                trust_root: "mantle-release-root".to_string(),
                metadata_expired: false,
                targets: vec![ReleaseTargetProfile {
                    artifact_identity: "artifact.blake3:abc".to_string(),
                    release_role: "release-binary".to_string(),
                    signed_metadata_role: "targets".to_string(),
                    expected_tags: vec!["onix-stack".to_string()],
                    actual_tags: vec!["onix-stack".to_string()],
                }],
            }],
            worker_receipts: vec![WorkerReceiptProfile {
                job_id: "job-1".to_string(),
                input_identity: "input.blake3:abc".to_string(),
                target_profile: "x86_64-linux".to_string(),
                worker_identity: "worker-1".to_string(),
                log_identity: "log.blake3:abc".to_string(),
                artifact_identities: vec!["artifact.blake3:abc".to_string()],
                cleanup_outcome: "complete".to_string(),
                replayable_event_id: "event-1".to_string(),
                candidate_policy_override: false,
                trusted_policy_approved: false,
            }],
            cancellation_outcomes: vec![CancellationOutcomeProfile {
                stage: CancellationStage::Cooperative,
                cleanup_attempted: true,
                cleanup_succeeded: true,
                final_receipt_persisted: true,
                artifact_integrity_verified: true,
            }],
            claim_texts: vec!["Mantle records bounded worker evidence only".to_string()],
        }
    }

    fn expire_metadata(fixture: &mut OxideReleaseWorkerFixture) {
        fixture.release_profiles[0].metadata_expired = true;
    }

    fn remove_target_tag(fixture: &mut OxideReleaseWorkerFixture) {
        fixture.release_profiles[0].targets[0].actual_tags.clear();
    }

    fn add_untrusted_policy_override(fixture: &mut OxideReleaseWorkerFixture) {
        fixture.worker_receipts[0].candidate_policy_override = true;
        fixture.worker_receipts[0].trusted_policy_approved = false;
    }

    fn fail_worker_cleanup(fixture: &mut OxideReleaseWorkerFixture) {
        fixture.worker_receipts[0].cleanup_outcome = "failed".to_string();
    }

    fn fail_mid_upload_cancellation(fixture: &mut OxideReleaseWorkerFixture) {
        fixture.cancellation_outcomes[0].stage = CancellationStage::OutputUpload;
        fixture.cancellation_outcomes[0].artifact_integrity_verified = false;
    }

    fn add_overclaim_text(fixture: &mut OxideReleaseWorkerFixture) {
        fixture.claim_texts.push("Buildomat proves release correctness".to_string());
    }
}
