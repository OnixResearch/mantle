use alloc::string::ToString;
use alloc::vec;

use super::contract;
use super::*;

const SOURCE_ARCHIVE_BYTES: u64 = 61_440;
const CANONICAL_BYTES: u64 = 2_088;
const SUPPORTED_CASES: u32 = 5_882;
const AUTHORITY_MEANING_MISMATCHES: u32 = 42;
const AUTHORITY_ORDER_MISMATCHES: u32 = 708;
const HISTORY_ORDER_MISMATCHES: u32 = 66;
const PHASE_REPORT_MISMATCHES: u32 = 20;
const RESULT_RETENTION_MISMATCHES: u32 = 2;

fn digest() -> String {
    "a".repeat(contract::BLAKE3_HEX_LENGTH)
}

fn report() -> TrellisRemoteAdmissionEvidenceReport {
    TrellisRemoteAdmissionEvidenceReport {
        schema: TRELLIS_ADMISSION_EVIDENCE_SCHEMA.to_string(),
        trellis: TrellisAdmissionSourceEvidence {
            repository: contract::TRELLIS_REPOSITORY.to_string(),
            revision: TRELLIS_ADMISSION_REVISION.to_string(),
            model_tree_oid: TRELLIS_ADMISSION_MODEL_TREE_OID.to_string(),
            source_archive_blake3: TRELLIS_ADMISSION_SOURCE_BLAKE3.to_string(),
            source_archive_bytes: SOURCE_ARCHIVE_BYTES,
        },
        mantle: TrellisAdmissionProjectionEvidence {
            projection_source_blake3: digest(),
            remote_attempt_source_blake3: digest(),
            oracle_manifest_blake3: digest(),
            oracle_matrix_blake3: digest(),
            oracle_case_count: TRELLIS_ADMISSION_ORACLE_CASES,
            supported_case_count: SUPPORTED_CASES,
            unsupported_case_count: TRELLIS_ADMISSION_ORACLE_CASES - SUPPORTED_CASES,
            projection_counts: TrellisAdmissionProjectionCounts {
                authority_meaning_mismatch: AUTHORITY_MEANING_MISMATCHES,
                authority_order_mismatch: AUTHORITY_ORDER_MISMATCHES,
                history_order_mismatch: HISTORY_ORDER_MISMATCHES,
                phase_report_mismatch: PHASE_REPORT_MISMATCHES,
                result_retention_mismatch: RESULT_RETENTION_MISMATCHES,
                supported: SUPPORTED_CASES,
            },
            policy_blake3: digest(),
        },
        proof: proof(),
        kamacite: TrellisAdmissionKamaciteEvidence {
            revision: contract::KAMACITE_REVISION.to_string(),
            profile: contract::KAMACITE_PROFILE.to_string(),
            producer_role: "formal-proof-candidate".to_string(),
            canonical_path: "evidence/canonical.preserves".to_string(),
            canonical_blake3: digest(),
            canonical_bytes: CANONICAL_BYTES,
            profile_identity_blake3: digest(),
            projection_path: "evidence/projection.json".to_string(),
            projection_blake3: digest(),
        },
        valence: TrellisAdmissionValenceEvidence {
            revision: contract::VALENCE_REVISION.to_string(),
            schema: contract::VALENCE_SCHEMA.to_string(),
            validation_role: "property".to_string(),
            outcome: "accepted_formal_proof".to_string(),
            valid: true,
            artifact_path: "evidence/valence.json".to_string(),
            artifact_blake3: digest(),
            receipt_hash_blake3: digest(),
        },
        assumptions: vec!["consumer identity admission is trusted".to_string()],
        claims: vec![TRELLIS_ADMISSION_CLAIM.to_string()],
        non_claims: TRELLIS_ADMISSION_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
        runtime_authority: TRELLIS_ADMISSION_RUNTIME_AUTHORITY.to_string(),
    }
}

fn proof() -> TrellisAdmissionProofEvidence {
    let properties = (0..contract::EXPECTED_PROPERTY_COUNT)
        .map(|index| TrellisAdmissionPropertyEvidence {
            id: alloc::format!("property-{index}"),
            requirement_id: alloc::format!("r[trellis.property-{index}]"),
            proof_function: alloc::format!("src/proofs.rs:{index}"),
        })
        .collect();
    let requirement_ids = (0..contract::EXPECTED_REQUIREMENT_COUNT)
        .map(|index| alloc::format!("r[trellis.requirement-{index}]"))
        .collect();
    TrellisAdmissionProofEvidence {
        proof_ir_blake3: digest(),
        verifier_receipt_blake3: digest(),
        properties,
        requirement_ids,
        verification_status: "passed".to_string(),
    }
}

#[test]
fn complete_bounded_evidence_passes() {
    let diagnostics = trellis_remote_admission_evidence_diagnostics(&report());

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(report().mantle.oracle_case_count, TRELLIS_ADMISSION_ORACLE_CASES);
}

#[test]
fn role_count_identity_and_claim_mutations_fail_closed() {
    let mut invalid = report();
    invalid.valence.validation_role = "recorded_only".to_string();
    invalid.mantle.supported_case_count = invalid.mantle.supported_case_count.saturating_sub(1);
    invalid.proof.proof_ir_blake3 = "invalid".to_string();
    invalid.claims = vec!["proves release eligibility".to_string()];

    let diagnostics = trellis_remote_admission_evidence_diagnostics(&invalid);

    assert!(diagnostics.iter().any(|value| value.contains("valence.validation_role")));
    assert!(diagnostics.iter().any(|value| value.contains("do not cover")));
    assert!(diagnostics.iter().any(|value| value.contains("proof.proof_ir_blake3")));
    assert!(diagnostics.iter().any(|value| value.contains("formal claim")));
}
