use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

mod authorities;
mod checks;
mod contract;
mod validation;

pub use contract::TRELLIS_ADMISSION_CLAIM;
pub use contract::TRELLIS_ADMISSION_EVIDENCE_SCHEMA;
pub use contract::TRELLIS_ADMISSION_MODEL_TREE_OID;
pub use contract::TRELLIS_ADMISSION_NON_CLAIMS;
pub use contract::TRELLIS_ADMISSION_ORACLE_CASES;
pub use contract::TRELLIS_ADMISSION_REVISION;
pub use contract::TRELLIS_ADMISSION_RUNTIME_AUTHORITY;
pub use contract::TRELLIS_ADMISSION_SOURCE_BLAKE3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisRemoteAdmissionEvidenceReport {
    pub schema: String,
    pub trellis: TrellisAdmissionSourceEvidence,
    pub mantle: TrellisAdmissionProjectionEvidence,
    pub proof: TrellisAdmissionProofEvidence,
    pub kamacite: TrellisAdmissionKamaciteEvidence,
    pub valence: TrellisAdmissionValenceEvidence,
    pub assumptions: Vec<String>,
    pub claims: Vec<String>,
    pub non_claims: Vec<String>,
    pub runtime_authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisAdmissionSourceEvidence {
    pub repository: String,
    pub revision: String,
    pub model_tree_oid: String,
    pub source_archive_blake3: String,
    pub source_archive_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisAdmissionProjectionEvidence {
    pub projection_source_blake3: String,
    pub remote_attempt_source_blake3: String,
    pub oracle_manifest_blake3: String,
    pub oracle_matrix_blake3: String,
    pub oracle_case_count: u32,
    pub supported_case_count: u32,
    pub unsupported_case_count: u32,
    pub projection_counts: TrellisAdmissionProjectionCounts,
    pub policy_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisAdmissionProjectionCounts {
    #[serde(rename = "authority-meaning-mismatch")]
    pub authority_meaning_mismatch: u32,
    #[serde(rename = "authority-order-mismatch")]
    pub authority_order_mismatch: u32,
    #[serde(rename = "history-order-mismatch")]
    pub history_order_mismatch: u32,
    #[serde(rename = "phase-report-mismatch")]
    pub phase_report_mismatch: u32,
    #[serde(rename = "result-retention-mismatch")]
    pub result_retention_mismatch: u32,
    pub supported: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisAdmissionProofEvidence {
    pub proof_ir_blake3: String,
    pub verifier_receipt_blake3: String,
    pub properties: Vec<TrellisAdmissionPropertyEvidence>,
    pub requirement_ids: Vec<String>,
    pub verification_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisAdmissionPropertyEvidence {
    pub id: String,
    pub requirement_id: String,
    pub proof_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisAdmissionKamaciteEvidence {
    pub revision: String,
    pub profile: String,
    pub producer_role: String,
    pub canonical_path: String,
    pub canonical_blake3: String,
    pub canonical_bytes: u64,
    pub profile_identity_blake3: String,
    pub projection_path: String,
    pub projection_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisAdmissionValenceEvidence {
    pub revision: String,
    pub schema: String,
    pub validation_role: String,
    pub outcome: String,
    pub valid: bool,
    pub artifact_path: String,
    pub artifact_blake3: String,
    pub receipt_hash_blake3: String,
}

// r[impl remote_builds.trellis_admission_evidence_boundary]
// r[impl remote_builds.trellis_admission_claim_boundary]
#[must_use]
pub fn trellis_remote_admission_evidence_diagnostics(report: &TrellisRemoteAdmissionEvidenceReport) -> Vec<String> {
    checks::diagnostics(report)
}

#[cfg(test)]
mod tests;
