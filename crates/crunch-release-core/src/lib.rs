#![no_std]
extern crate alloc;

mod error;
mod manifest;
mod reproducibility;

pub use error::ReleaseEvidenceError;
pub use manifest::BLAKE3_HEX_LENGTH_CHARS;
pub use manifest::BundledArtifact;
pub use manifest::BundledArtifactKind;
pub use manifest::CLAIM_SCOPE_PACKAGED_INTEGRITY;
pub use manifest::DEFAULT_PROOF_WORKFLOW_COMMAND;
pub use manifest::DEFAULT_PROOF_WORKFLOW_VERSION;
pub use manifest::FULL_SELF_HOSTING_PROOF_SCHEMA;
pub use manifest::FullSelfHostingProofIdentityFields;
pub use manifest::RELEASE_EVIDENCE_SCHEMA;
pub use manifest::ReleaseEvidenceManifest;
pub use manifest::ReleaseProofLinkage;
pub use manifest::ReleaseWorkflowIdentity;
pub use manifest::canonical_release_evidence_manifest;
pub use manifest::extract_full_self_hosting_proof_identity_fields;
pub use manifest::validate_bundled_artifact_record;
pub use reproducibility::RELEASE_REPRODUCIBILITY_REPORT_SCHEMA;
pub use reproducibility::RebuildWorkflowIdentity;
pub use reproducibility::ReleaseReproducibilityReport;
pub use reproducibility::ReleaseReproducibilityReportInit;
pub use reproducibility::ReleaseReproducibilityReportLinkage;
pub use reproducibility::ReproducibilityArtifactComparison;
pub use reproducibility::ReproducibilityComparisonResult;
pub use reproducibility::ReproducibilityComparisonVerdict;
pub use reproducibility::ReproducibilityProofClass;
pub use reproducibility::canonical_release_reproducibility_report;
pub use reproducibility::release_reproducibility_report_canonical_bytes;
pub use reproducibility::release_reproducibility_report_digest_blake3;
pub use reproducibility::validate_release_reproducibility_report_artifact_names;
pub use reproducibility::validate_release_reproducibility_report_linkage;
