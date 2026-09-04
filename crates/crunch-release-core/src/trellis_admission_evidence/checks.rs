use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use super::TrellisAdmissionProjectionEvidence;
use super::TrellisAdmissionProofEvidence;
use super::TrellisRemoteAdmissionEvidenceReport;
use super::authorities;
use super::contract;
use super::validation;

pub(super) fn diagnostics(report: &TrellisRemoteAdmissionEvidenceReport) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_fixed_fields(report, &mut diagnostics);
    validate_projection(&report.mantle, &mut diagnostics);
    validate_proof(&report.proof, &mut diagnostics);
    authorities::validate_kamacite(&report.kamacite, &mut diagnostics);
    authorities::validate_valence(&report.valence, &mut diagnostics);
    authorities::validate_claims(report, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics.truncate(contract::DIAGNOSTIC_COUNT_MAX);
    debug_assert!(diagnostics.len() <= contract::DIAGNOSTIC_COUNT_MAX);
    debug_assert!(
        report.trellis.source_archive_bytes > 0
            || diagnostics.iter().any(|value| value.contains("source_archive_bytes"))
    );
    diagnostics
}

fn validate_fixed_fields(report: &TrellisRemoteAdmissionEvidenceReport, diagnostics: &mut Vec<String>) {
    for input in [
        validation::ExpectedValue {
            actual: &report.schema,
            expected: contract::TRELLIS_ADMISSION_EVIDENCE_SCHEMA,
            field: "schema",
        },
        validation::ExpectedValue {
            actual: &report.trellis.repository,
            expected: contract::TRELLIS_REPOSITORY,
            field: "trellis.repository",
        },
        validation::ExpectedValue {
            actual: &report.trellis.revision,
            expected: contract::TRELLIS_ADMISSION_REVISION,
            field: "trellis.revision",
        },
        validation::ExpectedValue {
            actual: &report.trellis.model_tree_oid,
            expected: contract::TRELLIS_ADMISSION_MODEL_TREE_OID,
            field: "trellis.model_tree_oid",
        },
        validation::ExpectedValue {
            actual: &report.trellis.source_archive_blake3,
            expected: contract::TRELLIS_ADMISSION_SOURCE_BLAKE3,
            field: "trellis.source_archive_blake3",
        },
    ] {
        validation::equal(input, diagnostics);
    }
    validation::revision(
        validation::LabeledValue {
            value: &report.trellis.revision,
            field: "trellis.revision",
        },
        diagnostics,
    );
    validation::revision(
        validation::LabeledValue {
            value: &report.trellis.model_tree_oid,
            field: "trellis.model_tree_oid",
        },
        diagnostics,
    );
    validation::blake3(
        validation::LabeledValue {
            value: &report.trellis.source_archive_blake3,
            field: "trellis.source_archive_blake3",
        },
        diagnostics,
    );
    if report.trellis.source_archive_bytes != contract::TRELLIS_SOURCE_ARCHIVE_BYTES {
        diagnostics.push("trellis.source_archive_bytes mismatch".to_string());
    }
    debug_assert!(
        report.schema == contract::TRELLIS_ADMISSION_EVIDENCE_SCHEMA
            || diagnostics.iter().any(|value| value.contains("schema"))
    );
    debug_assert!(
        report.trellis.source_archive_blake3 == contract::TRELLIS_ADMISSION_SOURCE_BLAKE3
            || diagnostics.iter().any(|value| value.contains("source_archive_blake3"))
    );
}

fn validate_projection(evidence: &TrellisAdmissionProjectionEvidence, diagnostics: &mut Vec<String>) {
    for (value, field) in [
        (&evidence.projection_source_blake3, "mantle.projection_source_blake3"),
        (&evidence.remote_attempt_source_blake3, "mantle.remote_attempt_source_blake3"),
        (&evidence.oracle_manifest_blake3, "mantle.oracle_manifest_blake3"),
        (&evidence.oracle_matrix_blake3, "mantle.oracle_matrix_blake3"),
        (&evidence.policy_blake3, "mantle.policy_blake3"),
    ] {
        validation::blake3(validation::LabeledValue { value, field }, diagnostics);
    }
    if evidence.oracle_case_count != contract::TRELLIS_ADMISSION_ORACLE_CASES {
        diagnostics.push("mantle.oracle_case_count mismatch".to_string());
    }
    let total = evidence.supported_case_count.checked_add(evidence.unsupported_case_count);
    if total != Some(contract::TRELLIS_ADMISSION_ORACLE_CASES) {
        diagnostics.push("mantle supported and unsupported counts do not cover the oracle".to_string());
    }
    if evidence.supported_case_count == 0 || evidence.unsupported_case_count == 0 {
        diagnostics.push("mantle supported and unsupported counts must be positive".to_string());
    }
    if validation::projection_count_total(evidence.projection_counts) != Some(contract::TRELLIS_ADMISSION_ORACLE_CASES)
    {
        diagnostics.push("mantle.projection_counts do not cover the oracle".to_string());
    }
    if evidence.projection_counts.supported != evidence.supported_case_count {
        diagnostics.push("mantle supported count disagrees with projection_counts".to_string());
    }
    debug_assert!(
        evidence.oracle_case_count == contract::TRELLIS_ADMISSION_ORACLE_CASES
            || diagnostics.iter().any(|value| value.contains("oracle_case_count"))
    );
    debug_assert!(
        total == Some(contract::TRELLIS_ADMISSION_ORACLE_CASES)
            || diagnostics.iter().any(|value| value.contains("do not cover"))
    );
}

fn validate_proof(evidence: &TrellisAdmissionProofEvidence, diagnostics: &mut Vec<String>) {
    for (value, field) in [
        (&evidence.proof_ir_blake3, "proof.proof_ir_blake3"),
        (&evidence.verifier_receipt_blake3, "proof.verifier_receipt_blake3"),
    ] {
        validation::blake3(validation::LabeledValue { value, field }, diagnostics);
    }
    if evidence.properties.len() != contract::EXPECTED_PROPERTY_COUNT {
        diagnostics.push("proof property count mismatch".to_string());
    }
    if evidence.requirement_ids.len() != contract::EXPECTED_REQUIREMENT_COUNT {
        diagnostics.push("proof requirement count mismatch".to_string());
    }
    validation::equal(
        validation::ExpectedValue {
            actual: &evidence.verification_status,
            expected: "passed",
            field: "proof.verification_status",
        },
        diagnostics,
    );
    if evidence.properties.iter().any(|property| {
        property.id.is_empty() || property.requirement_id.is_empty() || property.proof_function.is_empty()
    }) {
        diagnostics.push("proof property metadata is incomplete".to_string());
    }
    debug_assert!(
        evidence.properties.len() == contract::EXPECTED_PROPERTY_COUNT
            || diagnostics.iter().any(|value| value.contains("property count"))
    );
    debug_assert!(
        evidence.verification_status == "passed"
            || diagnostics.iter().any(|value| value.contains("verification_status"))
    );
}
