//! Typed independent source-review rejection reasons, with stable wire diagnostics.
use alloc::string::String;
use core::fmt;

use super::SourceReviewStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceReviewError {
    AttachmentMalformed,
    PolicyMalformed,
    InsufficientApprovals,
    AuthenticationPolicyInvalid,
    DuplicateFullKey,
    AuthorExcluded,
    StatementInvalid,
    ReleaseAcquisitionOrObservationInvalid,
    SchemaUnsupported,
    PolicyBoundsInvalid,
    SubjectProfileUnsupported,
    SourceRevisionInvalid,
    ReleaseSourceMismatch,
    ReleaseRevisionUnavailable,
    ReleaseRevisionMismatch,
    ObservationSubjectMismatch,
    ReviewerKeyRoleInvalid,
    DuplicateAuthorKey,
    DispositionUnsupported,
    StaleSubjectRootOrPolicy,
    SignedScopeMismatch,
    NonClaimsInvalid,
    KeyOrSignatureMalformed,
    CanonicalizationFailed,
    ValenceMalformed,
    ValenceCanonicalizationFailed,
    ValenceReceiptIdentityMissing,
    ValenceReceiptIdentityMismatch,
    AuthenticationRejected(String),
    ValenceLinkageMismatch(&'static str),
}

impl SourceReviewError {
    pub(super) fn status(&self) -> SourceReviewStatus {
        match self {
            Self::InsufficientApprovals => SourceReviewStatus::Insufficient,
            Self::SchemaUnsupported | Self::SubjectProfileUnsupported | Self::DispositionUnsupported => {
                SourceReviewStatus::Unsupported
            }
            Self::ReleaseAcquisitionOrObservationInvalid
            | Self::ReleaseSourceMismatch
            | Self::ReleaseRevisionUnavailable
            | Self::ReleaseRevisionMismatch
            | Self::ObservationSubjectMismatch
            | Self::StaleSubjectRootOrPolicy => SourceReviewStatus::Stale,
            _ => SourceReviewStatus::Invalid,
        }
    }
}

impl fmt::Display for SourceReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttachmentMalformed => formatter.write_str("source-review-attachment-malformed"),
            Self::PolicyMalformed => formatter.write_str("source-review-policy-malformed"),
            Self::InsufficientApprovals => formatter.write_str("source-review-insufficient-approvals"),
            Self::AuthenticationPolicyInvalid => formatter.write_str("source-review-authentication-policy-invalid"),
            Self::DuplicateFullKey => formatter.write_str("source-review-duplicate-full-key"),
            Self::AuthorExcluded => formatter.write_str("source-review-author-excluded"),
            Self::StatementInvalid => formatter.write_str("source-review-statement-invalid"),
            Self::ReleaseAcquisitionOrObservationInvalid => {
                formatter.write_str("source-review-release-acquisition-or-observation-invalid")
            }
            Self::SchemaUnsupported => formatter.write_str("source-review-schema-unsupported"),
            Self::PolicyBoundsInvalid => formatter.write_str("source-review-policy-bounds-invalid"),
            Self::SubjectProfileUnsupported => formatter.write_str("source-review-subject-profile-unsupported"),
            Self::SourceRevisionInvalid => formatter.write_str("source-review-source-revision-invalid"),
            Self::ReleaseSourceMismatch => formatter.write_str("source-review-release-source-mismatch"),
            Self::ReleaseRevisionUnavailable => formatter.write_str("source-review-release-revision-unavailable"),
            Self::ReleaseRevisionMismatch => formatter.write_str("source-review-release-revision-mismatch"),
            Self::ObservationSubjectMismatch => formatter.write_str("source-review-observation-subject-mismatch"),
            Self::ReviewerKeyRoleInvalid => formatter.write_str("source-review-reviewer-key-role-invalid"),
            Self::DuplicateAuthorKey => formatter.write_str("source-review-duplicate-author-key"),
            Self::DispositionUnsupported => formatter.write_str("source-review-disposition-unsupported"),
            Self::StaleSubjectRootOrPolicy => formatter.write_str("source-review-stale-subject-root-or-policy"),
            Self::SignedScopeMismatch => formatter.write_str("source-review-signed-scope-mismatch"),
            Self::NonClaimsInvalid => formatter.write_str("source-review-non-claims-invalid"),
            Self::KeyOrSignatureMalformed => formatter.write_str("source-review-key-or-signature-malformed"),
            Self::CanonicalizationFailed => formatter.write_str("source-review-canonicalization-failed"),
            Self::ValenceMalformed => formatter.write_str("source-review-valence-malformed"),
            Self::ValenceCanonicalizationFailed => formatter.write_str("source-review-valence-canonicalization-failed"),
            Self::ValenceReceiptIdentityMissing => {
                formatter.write_str("source-review-valence-receipt-identity-missing")
            }
            Self::ValenceReceiptIdentityMismatch => {
                formatter.write_str("source-review-valence-receipt-identity-mismatch")
            }
            Self::AuthenticationRejected(codes) => write!(formatter, "source-review-authentication-rejected:{codes}"),
            Self::ValenceLinkageMismatch(field) => write!(formatter, "source-review-valence-linkage-mismatch:{field}"),
        }
    }
}

impl core::error::Error for SourceReviewError {}
