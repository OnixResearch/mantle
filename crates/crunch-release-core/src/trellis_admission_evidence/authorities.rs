use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use super::TrellisAdmissionKamaciteEvidence;
use super::TrellisAdmissionValenceEvidence;
use super::TrellisRemoteAdmissionEvidenceReport;
use super::contract;
use super::validation;

pub(super) fn validate_kamacite(evidence: &TrellisAdmissionKamaciteEvidence, diagnostics: &mut Vec<String>) {
    for input in [
        validation::ExpectedValue {
            actual: &evidence.revision,
            expected: contract::KAMACITE_REVISION,
            field: "kamacite.revision",
        },
        validation::ExpectedValue {
            actual: &evidence.profile,
            expected: contract::KAMACITE_PROFILE,
            field: "kamacite.profile",
        },
        validation::ExpectedValue {
            actual: &evidence.producer_role,
            expected: "formal-proof-candidate",
            field: "kamacite.producer_role",
        },
    ] {
        validation::equal(input, diagnostics);
    }
    validation::revision(
        validation::LabeledValue {
            value: &evidence.revision,
            field: "kamacite.revision",
        },
        diagnostics,
    );
    for (value, field) in [
        (&evidence.canonical_blake3, "kamacite.canonical_blake3"),
        (&evidence.profile_identity_blake3, "kamacite.profile_identity_blake3"),
        (&evidence.projection_blake3, "kamacite.projection_blake3"),
    ] {
        validation::blake3(validation::LabeledValue { value, field }, diagnostics);
    }
    validation::relative_path(
        validation::LabeledValue {
            value: &evidence.canonical_path,
            field: "kamacite.canonical_path",
        },
        diagnostics,
    );
    validation::relative_path(
        validation::LabeledValue {
            value: &evidence.projection_path,
            field: "kamacite.projection_path",
        },
        diagnostics,
    );
    if evidence.canonical_bytes == 0 {
        diagnostics.push("kamacite.canonical_bytes must be positive".to_string());
    }
    debug_assert!(
        evidence.profile == contract::KAMACITE_PROFILE
            || diagnostics.iter().any(|value| value.contains("kamacite.profile"))
    );
    debug_assert!(
        evidence.producer_role == "formal-proof-candidate"
            || diagnostics.iter().any(|value| value.contains("kamacite.producer_role"))
    );
}

pub(super) fn validate_valence(evidence: &TrellisAdmissionValenceEvidence, diagnostics: &mut Vec<String>) {
    for input in [
        validation::ExpectedValue {
            actual: &evidence.revision,
            expected: contract::VALENCE_REVISION,
            field: "valence.revision",
        },
        validation::ExpectedValue {
            actual: &evidence.schema,
            expected: contract::VALENCE_SCHEMA,
            field: "valence.schema",
        },
        validation::ExpectedValue {
            actual: &evidence.validation_role,
            expected: "property",
            field: "valence.validation_role",
        },
        validation::ExpectedValue {
            actual: &evidence.outcome,
            expected: "accepted_formal_proof",
            field: "valence.outcome",
        },
    ] {
        validation::equal(input, diagnostics);
    }
    if !evidence.valid {
        diagnostics.push("valence.valid must be true".to_string());
    }
    validation::revision(
        validation::LabeledValue {
            value: &evidence.revision,
            field: "valence.revision",
        },
        diagnostics,
    );
    for (value, field) in [
        (&evidence.artifact_blake3, "valence.artifact_blake3"),
        (&evidence.receipt_hash_blake3, "valence.receipt_hash_blake3"),
    ] {
        validation::blake3(validation::LabeledValue { value, field }, diagnostics);
    }
    validation::relative_path(
        validation::LabeledValue {
            value: &evidence.artifact_path,
            field: "valence.artifact_path",
        },
        diagnostics,
    );
    debug_assert!(
        evidence.validation_role == "property"
            || diagnostics.iter().any(|value| value.contains("valence.validation_role"))
    );
    debug_assert!(
        evidence.outcome == "accepted_formal_proof"
            || diagnostics.iter().any(|value| value.contains("valence.outcome"))
    );
}

pub(super) fn validate_claims(report: &TrellisRemoteAdmissionEvidenceReport, diagnostics: &mut Vec<String>) {
    if report.assumptions.is_empty() || report.assumptions.len() > contract::ASSUMPTION_COUNT_MAX {
        diagnostics.push("assumption count is outside bounds".to_string());
    }
    if report.assumptions.iter().any(|value| value.is_empty()) {
        diagnostics.push("assumption text must not be empty".to_string());
    }
    if report.claims.as_slice() != [contract::TRELLIS_ADMISSION_CLAIM] {
        diagnostics.push("formal claim is unsupported".to_string());
    }
    let expected_non_claims =
        contract::TRELLIS_ADMISSION_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    if report.non_claims != expected_non_claims {
        diagnostics.push("formal non-claim boundary changed".to_string());
    }
    validation::equal(
        validation::ExpectedValue {
            actual: &report.runtime_authority,
            expected: contract::TRELLIS_ADMISSION_RUNTIME_AUTHORITY,
            field: "runtime_authority",
        },
        diagnostics,
    );
    debug_assert!(
        report.claims.as_slice() == [contract::TRELLIS_ADMISSION_CLAIM]
            || diagnostics.iter().any(|value| value.contains("formal claim"))
    );
    debug_assert!(
        report.runtime_authority == contract::TRELLIS_ADMISSION_RUNTIME_AUTHORITY
            || diagnostics.iter().any(|value| value.contains("runtime_authority"))
    );
}
