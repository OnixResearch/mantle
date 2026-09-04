//! Pure fail-closed projection from Mantle remote-attempt facts to the pinned
//! Trellis fenced-attempt executable-model contract.
//!
//! This module does not read proof artifacts or affect runtime admission.
//! r[impl remote_builds.trellis_admission_model]
//! r[impl remote_builds.trellis_admission_projection]
//! r[impl remote_builds.trellis_admission_evidence_boundary]

mod boundary;
mod model;
mod outcome;
mod projection;
mod support;

pub use boundary::TRELLIS_REMOTE_ADMISSION_CLAIM;
pub use boundary::TRELLIS_REMOTE_ADMISSION_NON_CLAIMS;
pub use boundary::trellis_claim_text_is_bounded;
pub use model::TRELLIS_FENCED_ATTEMPT_REVISION;
pub use model::TRELLIS_FENCED_ATTEMPT_SOURCE_ARCHIVE_BLAKE3;
pub use model::TRELLIS_REMOTE_ADMISSION_MATRIX_CASES;
pub use model::TRELLIS_REMOTE_ADMISSION_ORACLE_RECORD_BYTES;
pub use model::TRELLIS_REMOTE_ADMISSION_ORACLE_SCHEMA;
pub use model::TrellisEventClass;
pub use model::TrellisHistoryClass;
pub use model::TrellisIdentityClass;
pub use model::TrellisModelReportKind;
pub use model::TrellisNormalizedOutcome;
pub use model::TrellisOutcomeDisposition;
pub use model::TrellisProjectionError;
pub use model::TrellisProjectionPhase;
pub use model::TrellisProjectionReportKind;
pub use model::TrellisRejectClass;
pub use model::TrellisRemoteAttemptCase;
pub use model::TrellisRemoteAttemptProjection;
pub use model::TrellisResultEffect;
pub use model::TrellisResultLinkageClass;
pub use model::TrellisScopeClass;
pub use outcome::normalize_mantle_trellis_outcome;
pub use projection::project_remote_attempt_to_trellis;
pub use support::validate_supported_trellis_case;

#[cfg(test)]
mod tests;
