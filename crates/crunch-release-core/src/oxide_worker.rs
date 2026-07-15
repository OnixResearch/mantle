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

struct CountBound<'a> {
    field_name: &'a str,
    actual_count: usize,
    maximum_count: usize,
}

#[derive(Clone, Copy)]
struct DiagnosticText<'a> {
    value: &'a str,
    field_name: &'a str,
}

struct RequiredFragment<'a> {
    value: &'a str,
    fragment: &'a str,
    field_name: &'a str,
}

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
    validate_fixture_collection_bounds(fixture, &mut diagnostics);
    validate_reference_inventory(&fixture.references, &mut diagnostics);
    validate_release_profiles(&fixture.release_profiles, &mut diagnostics);
    validate_worker_receipts(&fixture.worker_receipts, &mut diagnostics);
    validate_cancellation_outcomes(&fixture.cancellation_outcomes, &mut diagnostics);
    validate_claim_texts(&fixture.claim_texts, &mut diagnostics);
    let outcome = OxideReleaseWorkerReport {
        accepted: diagnostics.is_empty(),
        diagnostics,
    };
    debug_assert_eq!(outcome.accepted, outcome.diagnostics.is_empty());
    if outcome.accepted {
        debug_assert!(outcome.diagnostics.is_empty());
    } else {
        debug_assert!(!outcome.diagnostics.is_empty());
    }
    outcome
}

fn validate_fixture_collection_bounds(fixture: &OxideReleaseWorkerFixture, diagnostics: &mut Vec<String>) {
    let diagnostic_count_before = diagnostics.len();
    for bound in [
        CountBound {
            field_name: "references",
            actual_count: fixture.references.len(),
            maximum_count: MAX_REFERENCE_COUNT,
        },
        CountBound {
            field_name: "release_profiles",
            actual_count: fixture.release_profiles.len(),
            maximum_count: MAX_RELEASE_PROFILE_COUNT,
        },
        CountBound {
            field_name: "worker_receipts",
            actual_count: fixture.worker_receipts.len(),
            maximum_count: MAX_WORKER_RECEIPT_COUNT,
        },
        CountBound {
            field_name: "cancellation_outcomes",
            actual_count: fixture.cancellation_outcomes.len(),
            maximum_count: MAX_CANCELLATION_OUTCOME_COUNT,
        },
        CountBound {
            field_name: "claim_texts",
            actual_count: fixture.claim_texts.len(),
            maximum_count: MAX_CLAIM_TEXT_COUNT,
        },
    ] {
        validate_bounded_count(bound, diagnostics);
    }
    debug_assert!(diagnostics.len() >= diagnostic_count_before);
    debug_assert!(MAX_WORKER_RECEIPT_COUNT >= MAX_REFERENCE_COUNT);
}

fn validate_bounded_count(bound: CountBound<'_>, diagnostics: &mut Vec<String>) {
    if bound.actual_count > bound.maximum_count {
        diagnostics.push(format!("{} exceeds maximum count {}", bound.field_name, bound.maximum_count));
    }
}

fn validate_reference_inventory(references: &[ReferenceIntake], diagnostics: &mut Vec<String>) {
    for reference in references {
        push_required_texts(
            &[
                DiagnosticText {
                    value: &reference.source_repository,
                    field_name: "reference.source_repository",
                },
                DiagnosticText {
                    value: &reference.intended_adaptation,
                    field_name: "reference.intended_adaptation",
                },
                DiagnosticText {
                    value: &reference.license_posture,
                    field_name: "reference.license_posture",
                },
                DiagnosticText {
                    value: &reference.trust_boundary,
                    field_name: "reference.trust_boundary",
                },
            ],
            diagnostics,
        );
        push_required_fragment(
            RequiredFragment {
                value: &reference.non_claim_boundary,
                fragment: PROOF_AUTHORITY_NON_CLAIM_FRAGMENT,
                field_name: "reference.non_claim_boundary",
            },
            diagnostics,
        );
        push_no_overclaim(
            DiagnosticText {
                value: &reference.non_claim_boundary,
                field_name: "reference.non_claim_boundary",
            },
            diagnostics,
        );
    }
}

fn validate_release_profiles(profiles: &[ReleaseRepositoryProfile], diagnostics: &mut Vec<String>) {
    for profile in profiles {
        push_required_texts(
            &[
                DiagnosticText {
                    value: &profile.profile_id,
                    field_name: "release_profile.profile_id",
                },
                DiagnosticText {
                    value: &profile.signed_metadata_identity,
                    field_name: "release_profile.signed_metadata_identity",
                },
                DiagnosticText {
                    value: &profile.expiration_policy,
                    field_name: "release_profile.expiration_policy",
                },
                DiagnosticText {
                    value: &profile.trust_root,
                    field_name: "release_profile.trust_root",
                },
            ],
            diagnostics,
        );
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
    let diagnostic_count_before = diagnostics.len();
    debug_assert!(MAX_RELEASE_PROFILE_COUNT > 0);
    push_required_texts(
        &[
            DiagnosticText {
                value: &target.artifact_identity,
                field_name: "release_target.artifact_identity",
            },
            DiagnosticText {
                value: &target.release_role,
                field_name: "release_target.release_role",
            },
            DiagnosticText {
                value: &target.signed_metadata_role,
                field_name: "release_target.signed_metadata_role",
            },
        ],
        diagnostics,
    );
    if target.expected_tags.is_empty() {
        diagnostics.push("release_target expected_tags must not be empty".to_string());
    }
    for tag in &target.expected_tags {
        if !target.actual_tags.iter().any(|actual| actual == tag) {
            diagnostics.push(format!("release_target tag mismatch: missing {tag}"));
        }
    }
    debug_assert!(diagnostics.len() >= diagnostic_count_before);
}

fn validate_worker_receipts(receipts: &[WorkerReceiptProfile], diagnostics: &mut Vec<String>) {
    for receipt in receipts {
        push_required_texts(
            &[
                DiagnosticText {
                    value: &receipt.job_id,
                    field_name: "worker_receipt.job_id",
                },
                DiagnosticText {
                    value: &receipt.input_identity,
                    field_name: "worker_receipt.input_identity",
                },
                DiagnosticText {
                    value: &receipt.target_profile,
                    field_name: "worker_receipt.target_profile",
                },
                DiagnosticText {
                    value: &receipt.worker_identity,
                    field_name: "worker_receipt.worker_identity",
                },
                DiagnosticText {
                    value: &receipt.log_identity,
                    field_name: "worker_receipt.log_identity",
                },
                DiagnosticText {
                    value: &receipt.replayable_event_id,
                    field_name: "worker_receipt.replayable_event_id",
                },
            ],
            diagnostics,
        );
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
        push_no_overclaim(
            DiagnosticText {
                value: claim_text,
                field_name: "claim_text",
            },
            diagnostics,
        );
    }
}

fn push_required_texts(fields: &[DiagnosticText<'_>], diagnostics: &mut Vec<String>) {
    let diagnostic_count_before = diagnostics.len();
    for field in fields {
        push_nonempty(*field, diagnostics);
    }
    debug_assert!(diagnostics.len() >= diagnostic_count_before);
    debug_assert!(fields.iter().all(|field| !field.field_name.is_empty()));
}

fn push_nonempty(field: DiagnosticText<'_>, diagnostics: &mut Vec<String>) {
    if field.value.trim().is_empty() {
        diagnostics.push(format!("{} must not be empty", field.field_name));
    }
}

fn push_required_fragment(field: RequiredFragment<'_>, diagnostics: &mut Vec<String>) {
    if !field.value.to_ascii_lowercase().contains(field.fragment) {
        diagnostics.push(format!("{} missing required fragment {:?}", field.field_name, field.fragment));
    }
}

fn push_no_overclaim(field: DiagnosticText<'_>, diagnostics: &mut Vec<String>) {
    let lower = field.value.to_ascii_lowercase();
    for fragment in OVERCLAIM_FRAGMENTS {
        if lower.contains(fragment) {
            diagnostics.push(format!("{} contains overclaim fragment {fragment:?}", field.field_name));
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    type OxideFixtureMutator = fn(&mut OxideReleaseWorkerFixture);
    type OxideFixtureCase = (&'static str, OxideFixtureMutator, &'static str);

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
        let cases: &[OxideFixtureCase] = &[
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
