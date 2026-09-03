//! Bounded source-review attachment verification for release evidence.
//!
//! Mantle consumes externally produced review decisions. It never creates
//! reviewer statements, runs review workflow, or trusts producer status
//! fields. The pure core validates attachment structure, exact source
//! linkage, canonical identities, reviewer-key policy, role separation, and
//! distinct approval counting through the admitted Artifact Auth boundary.
//! Cryptographic observations are supplied by the imperative shell.
//!
//! r[impl verification_evidence.release_source_review_evidence]

// machine-artifact-public: release.source-review-attachment

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;

/// Attachment schema supported by this implementation.
pub const SOURCE_REVIEW_ATTACHMENT_SCHEMA: &str = "mantle.source-review.attachment.v1";
/// Statement domain every counted review statement MUST carry.
pub const SOURCE_REVIEW_STATEMENT_DOMAIN: &str = "mantle.release.source-review.v1";
/// Signature purpose carried by counted review statements.
pub const SOURCE_REVIEW_STATEMENT_PURPOSE: &str = "approve-release-source";
/// Compatibility profile shared by review statements and policy.
pub const SOURCE_REVIEW_PROFILE_ID: &str = "mantle-release-source-review-v1";
/// Policy schema.
pub const REVIEWED_SOURCE_POLICY_SCHEMA: &str = "mantle.reviewed-source-policy.v1";
/// Named StageX-inspired preset requiring two distinct reviewers.
pub const REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER: &str = "stagex-two-reviewer";
/// Distinct reviewer threshold defined by the StageX-inspired preset.
pub const STAGEX_TWO_REVIEWER_THRESHOLD: u16 = 2;
/// Reviewer-key generation bound by statements and policy.
pub const REVIEWER_KEY_GENERATION: u64 = 1;
/// Bundle-relative path where a release bundles its source-review attachment.
pub const RELEASED_SOURCE_REVIEW_ATTACHMENT_RELATIVE_PATH: &str = "source-review/attachment.json";
/// Source review is not required for a generic release.
pub const SOURCE_REVIEW_MODE_OPTIONAL: &str = "optional";
/// An operator selected an explicit reviewed-source policy.
pub const SOURCE_REVIEW_MODE_REQUIRED: &str = "required";
/// Status: no reviewed-source policy was selected and no attachment exists.
pub const SOURCE_REVIEW_STATUS_NOT_REQUIRED: &str = "not-required";
/// Status: optional attachment verified as a bounded fact.
pub const SOURCE_REVIEW_STATUS_VERIFIED_OPTIONAL: &str = "verified-optional";
/// Status: selected reviewed-source policy is satisfied.
pub const SOURCE_REVIEW_STATUS_SATISFIED: &str = "satisfied";
/// Status: selected or present review evidence failed.
pub const SOURCE_REVIEW_STATUS_FAILED: &str = "failed";
/// Review disposition that can satisfy a reviewed-source policy.
pub const SOURCE_REVIEW_DISPOSITION_APPROVED: &str = "approved";
/// Review disposition that fails a selected reviewed-source policy.
pub const SOURCE_REVIEW_DISPOSITION_NEEDS_REVISION: &str = "needs-revision";
/// Domain of build-witness statements that MUST NOT count as review approval.
pub const BUILD_WITNESS_STATEMENT_DOMAIN: &str = "mantle.release.build-witness.v1";

pub const SOURCE_REVIEW_BOUNDARY: &str = "Mantle verifies review attachment identity, source linkage, signatures, policy, and roles only; Cairn owns review workflow and Valence owns evidence identity";
pub const SOURCE_REVIEW_NON_CLAIM_SOURCE_CORRECTNESS: &str = "not source correctness";
pub const SOURCE_REVIEW_NON_CLAIM_REVIEW_COMPLETENESS: &str = "not review completeness";
pub const SOURCE_REVIEW_NON_CLAIM_REVIEWER_COMPETENCE: &str = "not reviewer competence";
pub const SOURCE_REVIEW_NON_CLAIM_BUILD_WITNESS_QUORUM: &str = "not build-witness quorum";
pub const SOURCE_REVIEW_NON_CLAIM_RELEASE_ELIGIBILITY: &str = "not release eligibility";
pub const SOURCE_REVIEW_NON_CLAIM_PULL_REQUEST_APPROVAL: &str = "not pull-request approval";

pub const MAX_SOURCE_REVIEW_STATEMENTS_COUNT: u32 = 32;
pub const MAX_TRUSTED_REVIEWERS_COUNT: u32 = 32;
pub const MAX_SOURCE_REVIEW_TEXT_BYTES: u32 = 256;
pub const MAX_SOURCE_REVIEW_NON_CLAIMS_COUNT: u32 = 16;
pub const MAX_SOURCE_REVIEW_DIAGNOSTICS_COUNT: u32 = 64;

const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const ED25519_PUBLIC_KEY_HEX_LENGTH_CHARS: usize = 64;
const ED25519_SIGNATURE_HEX_LENGTH_CHARS: usize = 128;
const ATTACHMENT_IDENTITY_DOMAIN: &[u8] = b"mantle.source-review.attachment-identity.v1\0";
const POLICY_IDENTITY_DOMAIN: &[u8] = b"mantle.reviewed-source-policy-identity.v1\0";
const CLAIM_ROOT_PROFILE: &str = "cairn-claim-root.v1";
const REVIEW_POLICY_PROFILE: &str = "review-policy.v1";
const RELEASE_SOURCE_PROFILE: &str = "release-source-archive.v1";
const VALENCE_EVIDENCE_PROFILE: &str = "valence-evidence.v1";
const POLICY_CURRENTNESS_PROFILE: &str = "reviewed-source-policy.v1";
const REQUIRED_NON_CLAIMS_COUNT: usize = 6;
const REQUIRED_NON_CLAIMS: [&str; REQUIRED_NON_CLAIMS_COUNT] = [
    "not source correctness",
    "not review completeness",
    "not reviewer competence",
    "not build-witness quorum",
    "not release eligibility",
    "not pull-request approval",
];

/// Selected source-review mode for one release verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceReviewMode {
    /// Review evidence is optional; absence is `not-required`.
    Optional,
    /// An explicit reviewed-source policy must be satisfied.
    RequiredReviewedSource,
}

impl SourceReviewMode {
    /// Parse an operator mode string.
    pub fn parse(value: &str) -> Result<Self, ReleaseEvidenceError> {
        match value {
            SOURCE_REVIEW_MODE_OPTIONAL => Ok(Self::Optional),
            SOURCE_REVIEW_MODE_REQUIRED => Ok(Self::RequiredReviewedSource),
            other => Err(crate::ReleaseEvidenceError::Validation(format!(
                "unsupported reviewed-source mode '{other}', expected 'optional' or 'required'"
            ))),
        }
    }

    /// Whether this mode requires satisfied review evidence.
    #[must_use]
    pub const fn is_required(self) -> bool {
        matches!(self, Self::RequiredReviewedSource)
    }
}

/// Currentness classification for one trusted reviewer key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ReviewerCurrentness {
    /// Current for signing and verification.
    Current,
    /// Bounded verification-only rotation overlap.
    VerificationOverlap,
    /// Superseded and no longer admitted.
    Superseded,
    /// Explicitly revoked.
    Revoked,
}

impl ReviewerCurrentness {
    fn to_artifact_auth(self) -> artifact_auth_core::KeyCurrentness {
        match self {
            Self::Current => artifact_auth_core::KeyCurrentness::Current,
            Self::VerificationOverlap => artifact_auth_core::KeyCurrentness::VerificationOverlap,
            Self::Superseded => artifact_auth_core::KeyCurrentness::Superseded,
            Self::Revoked => artifact_auth_core::KeyCurrentness::Revoked,
        }
    }
}

/// One operator-authorized reviewer key with its policy classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedReviewerKey {
    /// Producer identity that review statements from this key MUST carry.
    pub producer_id: String,
    /// Reviewer label that review statements from this key MUST carry.
    pub label: String,
    /// Lowercase hexadecimal Ed25519 public key.
    pub public_key_hex: String,
    /// Operator currentness classification.
    pub currentness: ReviewerCurrentness,
}

/// Typed reviewed-source policy selected by an operator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedSourcePolicy {
    /// Policy schema.
    pub schema: String,
    /// Optional named preset that defines the threshold.
    #[serde(default = "absent_policy_preset", skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    /// Required count of distinct verified authorized reviewer keys.
    pub required_distinct_reviewers: u16,
    /// Explicit reviewer-key authority set.
    pub reviewers: Vec<TrustedReviewerKey>,
    /// Reviewer key excluded from approval counting (the change author).
    #[serde(default = "absent_optional_string", skip_serializing_if = "Option::is_none")]
    pub excluded_author_public_key_hex: Option<String>,
}

/// Exact subject a review attachment MUST bind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewSubjectLink {
    /// BLAKE3 of the exact release source archive.
    pub source_archive_digest_blake3: String,
    /// Optional immutable source revision the review covered.
    #[serde(default = "absent_optional_string", skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    /// BLAKE3 identity of the reviewed Cairn claim root.
    pub claim_root_blake3: String,
    /// BLAKE3 identity of the review policy the statements were produced under.
    pub review_policy_digest_blake3: String,
}

/// One signed review statement inside an attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewApprovalStatement {
    /// Statement domain; only the source-review domain can count.
    pub statement_domain: String,
    /// Reviewer label bound by the statement.
    pub reviewer_label: String,
    /// Lowercase hexadecimal Ed25519 public key.
    pub public_key_hex: String,
    /// Lowercase hexadecimal detached Ed25519 signature.
    pub signature_hex: String,
    /// Review disposition.
    pub disposition: String,
    /// Key generation asserted by the statement.
    #[serde(default = "default_reviewer_generation")]
    pub generation: u64,
}

/// Externally produced review attachment consumed by Mantle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewAttachment {
    /// Attachment schema.
    pub schema: String,
    /// Exact subject binding.
    pub subject: SourceReviewSubjectLink,
    /// Producer-owned obligation identity (Cairn verification obligation).
    pub producer_obligation_id: String,
    /// Producer-reported disposition; never authoritative for Mantle decisions.
    pub producer_disposition: String,
    /// BLAKE3 identity of the Valence evidence record for this review.
    pub valence_evidence_blake3: String,
    /// Signed approval statements.
    pub approvals: Vec<SourceReviewApprovalStatement>,
    /// Required bounded non-claims.
    pub non_claims: Vec<String>,
    /// Canonical BLAKE3 identity of this attachment.
    pub attachment_blake3: String,
}

/// Result of evaluating source-review evidence for one release verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewVerification {
    /// Whether a reviewed-source policy was selected.
    pub required: bool,
    /// Whether the selected or present review evidence satisfied policy.
    pub valid: bool,
    /// One of `not-required`, `verified-optional`, `satisfied`, `failed`.
    pub status: String,
    /// Deterministic failure reason; absent on success.
    #[serde(default = "absent_optional_string", skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    /// Canonical attachment identity when an attachment was evaluated.
    #[serde(default = "absent_optional_string", skip_serializing_if = "Option::is_none")]
    pub attachment_blake3: Option<String>,
    /// Canonical policy identity.
    pub policy_blake3: String,
    /// Distinct counted reviewer full-key BLAKE3 identities.
    pub counted_reviewer_key_blake3: Vec<String>,
    /// Claim boundary text.
    pub boundary: String,
    /// Required bounded non-claims.
    pub non_claims: Vec<String>,
    /// Bounded deterministic diagnostics.
    pub diagnostics: Vec<String>,
}

/// Complete input for one source-review evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReviewVerificationInput<'a> {
    /// Selected mode.
    pub mode: SourceReviewMode,
    /// Operator policy.
    pub policy: &'a ReviewedSourcePolicy,
    /// Source archive digest remeasured from the release bundle.
    pub expected_source_archive_digest_blake3: String,
    /// Current Cairn claim root identity for the reviewed claim.
    pub expected_claim_root_blake3: String,
    /// Immutable source revision the release names, when present.
    pub expected_source_revision: Option<String>,
    /// Attachment under evaluation; `None` when absent.
    pub attachment: Option<&'a SourceReviewAttachment>,
    /// Shell-verified cryptographic observations, parallel to attachment approvals.
    pub cryptographic_observations: Vec<artifact_auth_core::CryptographicObservation>,
}

fn absent_optional_string() -> Option<String> {
    None
}

fn absent_policy_preset() -> Option<String> {
    None
}

const fn default_reviewer_generation() -> u64 {
    REVIEWER_KEY_GENERATION
}

fn is_lowercase_hex(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS && is_lowercase_hex(value)
}

fn valid_public_key_hex(value: &str) -> bool {
    value.len() == ED25519_PUBLIC_KEY_HEX_LENGTH_CHARS && is_lowercase_hex(value)
}

fn valid_signature_hex(value: &str) -> bool {
    value.len() == ED25519_SIGNATURE_HEX_LENGTH_CHARS && is_lowercase_hex(value)
}

fn text_fits_bound(value: &str) -> bool {
    u32::try_from(value.len()).is_ok_and(|length| length <= MAX_SOURCE_REVIEW_TEXT_BYTES)
}

fn push_framed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length_bytes = (bytes.len() as u64).to_le_bytes();
    hasher.update(&length_bytes);
    hasher.update(bytes);
}

fn required_non_claims_sorted() -> Vec<String> {
    let mut values = REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect::<Vec<_>>();
    values.sort();
    values
}

/// Compute the canonical BLAKE3 identity of a reviewed-source policy.
pub fn reviewed_source_policy_identity(policy: &ReviewedSourcePolicy) -> Result<String, ReleaseEvidenceError> {
    validate_reviewed_source_policy(policy)?;
    let mut hasher = blake3::Hasher::new();
    debug_assert_eq!(policy.schema, REVIEWED_SOURCE_POLICY_SCHEMA);
    hasher.update(POLICY_IDENTITY_DOMAIN);
    push_framed(&mut hasher, policy.schema.as_bytes());
    if let Some(preset) = &policy.preset {
        push_framed(&mut hasher, preset.as_bytes());
    } else {
        push_framed(&mut hasher, b"");
    }
    hasher.update(&policy.required_distinct_reviewers.to_le_bytes());
    let mut reviewers = policy.reviewers.clone();
    reviewers.sort_by(|left, right| {
        left.public_key_hex
            .cmp(&right.public_key_hex)
            .then(left.producer_id.cmp(&right.producer_id))
            .then(left.label.cmp(&right.label))
    });
    push_framed(&mut hasher, &reviewers.len().to_le_bytes());
    for reviewer in &reviewers {
        push_framed(&mut hasher, reviewer.producer_id.as_bytes());
        push_framed(&mut hasher, reviewer.label.as_bytes());
        push_framed(&mut hasher, reviewer.public_key_hex.as_bytes());
        let currentness: &[u8] = match reviewer.currentness {
            ReviewerCurrentness::Current => b"current",
            ReviewerCurrentness::VerificationOverlap => b"verification-overlap",
            ReviewerCurrentness::Superseded => b"superseded",
            ReviewerCurrentness::Revoked => b"revoked",
        };
        push_framed(&mut hasher, currentness);
    }
    if let Some(excluded) = &policy.excluded_author_public_key_hex {
        push_framed(&mut hasher, excluded.as_bytes());
    } else {
        push_framed(&mut hasher, b"");
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Compute the canonical BLAKE3 identity of an attachment.
///
/// The identity excludes the declared `attachment_blake3` field itself, so a
/// tampered attachment body is always detected by re-computation.
pub fn source_review_attachment_identity(attachment: &SourceReviewAttachment) -> String {
    attachment_identity_excluding_field(attachment)
}

/// Parse a source-review attachment from canonical JSON bytes.
pub fn parse_source_review_attachment(bytes: &[u8]) -> Result<SourceReviewAttachment, ReleaseEvidenceError> {
    serde_json::from_slice(bytes)
        .map_err(|error| crate::ReleaseEvidenceError::Validation(format!("parsing source review attachment: {error}")))
}

/// Structural validation used before any cryptographic or policy evaluation.
pub fn validate_source_review_attachment_structure(
    attachment: &SourceReviewAttachment,
) -> Result<(), ReleaseEvidenceError> {
    let invalid = |message: String| crate::ReleaseEvidenceError::Validation(message);
    debug_assert!(!attachment.producer_obligation_id.is_empty());
    if attachment.schema != SOURCE_REVIEW_ATTACHMENT_SCHEMA {
        return Err(invalid(format!("unsupported source review attachment schema '{}'", attachment.schema)));
    }
    if !valid_blake3(&attachment.subject.source_archive_digest_blake3) {
        return Err(invalid("source review subject source digest must be lowercase BLAKE3 hex".to_string()));
    }
    if !valid_blake3(&attachment.subject.claim_root_blake3) {
        return Err(invalid("source review subject claim root must be lowercase BLAKE3 hex".to_string()));
    }
    if !valid_blake3(&attachment.subject.review_policy_digest_blake3) {
        return Err(invalid("source review subject policy digest must be lowercase BLAKE3 hex".to_string()));
    }
    if !valid_blake3(&attachment.valence_evidence_blake3) {
        return Err(invalid("source review valence evidence identity must be lowercase BLAKE3 hex".to_string()));
    }
    if !text_fits_bound(&attachment.producer_obligation_id) || attachment.producer_obligation_id.is_empty() {
        return Err(invalid("source review producer obligation id must be 1..=256 UTF-8 bytes".to_string()));
    }
    if !text_fits_bound(&attachment.producer_disposition) {
        return Err(invalid("source review producer disposition must be at most 256 UTF-8 bytes".to_string()));
    }
    if let Some(revision) = &attachment.subject.source_revision
        && !text_fits_bound(revision)
    {
        return Err(invalid("source review subject revision must be at most 256 UTF-8 bytes".to_string()));
    }
    if !count_fits_limit(attachment.approvals.len(), MAX_SOURCE_REVIEW_STATEMENTS_COUNT) {
        return Err(invalid(format!(
            "source review attachment records {} approvals, limit is {MAX_SOURCE_REVIEW_STATEMENTS_COUNT}",
            attachment.approvals.len()
        )));
    }
    if attachment.approvals.is_empty() {
        return Err(invalid("source review attachment requires at least one approval statement".to_string()));
    }
    for approval in &attachment.approvals {
        if !text_fits_bound(&approval.reviewer_label) || approval.reviewer_label.is_empty() {
            return Err(invalid("source review reviewer label must be 1..=256 UTF-8 bytes".to_string()));
        }
        if !valid_public_key_hex(&approval.public_key_hex) {
            return Err(invalid("source review public key must be 64 lowercase hex characters".to_string()));
        }
        if !valid_signature_hex(&approval.signature_hex) {
            return Err(invalid("source review signature must be 128 lowercase hex characters".to_string()));
        }
        if !text_fits_bound(&approval.statement_domain) || approval.statement_domain.is_empty() {
            return Err(invalid("source review statement domain must be 1..=256 UTF-8 bytes".to_string()));
        }
        if approval.disposition != SOURCE_REVIEW_DISPOSITION_APPROVED
            && approval.disposition != SOURCE_REVIEW_DISPOSITION_NEEDS_REVISION
        {
            return Err(invalid(format!(
                "source review disposition '{}' is not '{SOURCE_REVIEW_DISPOSITION_APPROVED}' or '{SOURCE_REVIEW_DISPOSITION_NEEDS_REVISION}'",
                approval.disposition
            )));
        }
    }
    validate_required_non_claims(&attachment.non_claims)?;
    if attachment.attachment_blake3 != attachment_identity_excluding_field(attachment) {
        return Err(invalid("source review attachment identity does not match canonical bytes".to_string()));
    }
    Ok(())
}

fn attachment_identity_excluding_field(attachment: &SourceReviewAttachment) -> String {
    debug_assert!(!attachment.approvals.is_empty());
    let mut hasher = blake3::Hasher::new();
    hasher.update(ATTACHMENT_IDENTITY_DOMAIN);
    push_framed(&mut hasher, attachment.schema.as_bytes());
    push_framed(&mut hasher, attachment.subject.source_archive_digest_blake3.as_bytes());
    if let Some(revision) = &attachment.subject.source_revision {
        push_framed(&mut hasher, revision.as_bytes());
    } else {
        push_framed(&mut hasher, b"");
    }
    push_framed(&mut hasher, attachment.subject.claim_root_blake3.as_bytes());
    push_framed(&mut hasher, attachment.subject.review_policy_digest_blake3.as_bytes());
    push_framed(&mut hasher, attachment.producer_obligation_id.as_bytes());
    push_framed(&mut hasher, attachment.producer_disposition.as_bytes());
    push_framed(&mut hasher, attachment.valence_evidence_blake3.as_bytes());
    let mut approvals = attachment.approvals.clone();
    approvals.sort_by(|left, right| {
        left.public_key_hex
            .cmp(&right.public_key_hex)
            .then(left.reviewer_label.cmp(&right.reviewer_label))
            .then(left.signature_hex.cmp(&right.signature_hex))
    });
    push_framed(&mut hasher, &approvals.len().to_le_bytes());
    for approval in &approvals {
        push_framed(&mut hasher, approval.statement_domain.as_bytes());
        push_framed(&mut hasher, approval.reviewer_label.as_bytes());
        push_framed(&mut hasher, approval.public_key_hex.as_bytes());
        push_framed(&mut hasher, approval.signature_hex.as_bytes());
        push_framed(&mut hasher, approval.disposition.as_bytes());
        hasher.update(&approval.generation.to_le_bytes());
    }
    let mut non_claims = attachment.non_claims.clone();
    non_claims.sort();
    push_framed(&mut hasher, &non_claims.len().to_le_bytes());
    for claim in &non_claims {
        push_framed(&mut hasher, claim.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn validate_required_non_claims(non_claims: &[String]) -> Result<(), ReleaseEvidenceError> {
    if !count_fits_limit(non_claims.len(), MAX_SOURCE_REVIEW_NON_CLAIMS_COUNT) {
        return Err(crate::ReleaseEvidenceError::Validation(format!(
            "source review attachment records {} non-claims, limit is {MAX_SOURCE_REVIEW_NON_CLAIMS_COUNT}",
            non_claims.len()
        )));
    }
    for required in REQUIRED_NON_CLAIMS {
        if !non_claims.iter().any(|claim| claim == required) {
            return Err(crate::ReleaseEvidenceError::Validation(format!(
                "source review attachment is missing required non-claim '{required}'"
            )));
        }
    }
    Ok(())
}

/// Validate a typed reviewed-source policy.
pub fn validate_reviewed_source_policy(policy: &ReviewedSourcePolicy) -> Result<(), ReleaseEvidenceError> {
    let invalid = |message: String| crate::ReleaseEvidenceError::Validation(message);
    debug_assert!(policy.required_distinct_reviewers >= 1);
    if policy.schema != REVIEWED_SOURCE_POLICY_SCHEMA {
        return Err(invalid(format!("unsupported reviewed source policy schema '{}'", policy.schema)));
    }
    if let Some(preset) = &policy.preset {
        if preset != REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER {
            return Err(invalid(format!("unknown reviewed source preset '{preset}'")));
        }
        if policy.required_distinct_reviewers != STAGEX_TWO_REVIEWER_THRESHOLD {
            return Err(invalid(format!(
                "preset '{REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER}' requires threshold {STAGEX_TWO_REVIEWER_THRESHOLD}"
            )));
        }
    }
    if policy.required_distinct_reviewers == 0 {
        return Err(invalid("reviewed source policy requires at least one distinct reviewer".to_string()));
    }
    if !count_fits_limit(policy.reviewers.len(), MAX_TRUSTED_REVIEWERS_COUNT) {
        return Err(invalid(format!(
            "reviewed source policy lists {} reviewers, limit is {MAX_TRUSTED_REVIEWERS_COUNT}",
            policy.reviewers.len()
        )));
    }
    if !policy.reviewers.is_empty() && usize::from(policy.required_distinct_reviewers) > policy.reviewers.len() {
        return Err(invalid("reviewed source threshold exceeds the trusted reviewer count".to_string()));
    }
    let mut seen_keys = alloc::collections::BTreeSet::new();
    for reviewer in &policy.reviewers {
        if !text_fits_bound(&reviewer.producer_id) || reviewer.producer_id.is_empty() {
            return Err(invalid("reviewer producer id must be 1..=256 UTF-8 bytes".to_string()));
        }
        if !text_fits_bound(&reviewer.label) || reviewer.label.is_empty() {
            return Err(invalid("reviewer label must be 1..=256 UTF-8 bytes".to_string()));
        }
        if !valid_public_key_hex(&reviewer.public_key_hex) {
            return Err(invalid("reviewer public key must be 64 lowercase hex characters".to_string()));
        }
        if !seen_keys.insert(reviewer.public_key_hex.clone()) {
            return Err(invalid(format!(
                "reviewer key '{}' is listed more than once; duplicate keys cannot inflate the threshold",
                reviewer.public_key_hex
            )));
        }
    }
    if let Some(excluded) = &policy.excluded_author_public_key_hex
        && !valid_public_key_hex(excluded)
    {
        return Err(invalid("excluded author key must be 64 lowercase hex characters".to_string()));
    }
    Ok(())
}

fn count_fits_limit(count: usize, maximum_count: u32) -> bool {
    u32::try_from(count).is_ok_and(|count_u32| count_u32 <= maximum_count)
}

/// Build the release-evidence binding record for a bundled review attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewReleaseEvidence {
    /// Bundled attachment artifact.
    pub attachment: crate::BundledArtifact,
}

fn artifact_ref(field: RefField<'_>) -> artifact_auth_core::ArtifactRef {
    artifact_auth_core::ArtifactRef {
        profile: field.profile.to_string(),
        algorithm: artifact_auth_core::ALGORITHM_BLAKE3.to_string(),
        digest_hex: field.digest_hex.to_string(),
    }
}

struct RefField<'a> {
    profile: &'a str,
    digest_hex: &'a str,
}

/// Map one approval statement and attachment subject into an Artifact Auth statement.
pub fn map_source_review_statement(
    attachment: &SourceReviewAttachment,
    approval: &SourceReviewApprovalStatement,
) -> Result<artifact_auth_core::ArtifactStatement, ReleaseEvidenceError> {
    debug_assert!(!attachment.producer_obligation_id.is_empty());
    debug_assert!(!approval.reviewer_label.is_empty());
    let public_key_bytes = decode_hex(&approval.public_key_hex)
        .map_err(|error| crate::ReleaseEvidenceError::Validation(format!("decoding reviewer public key: {error}")))?;
    let scope = artifact_auth_core::AuthenticationScope {
        domain: SOURCE_REVIEW_STATEMENT_DOMAIN.to_string(),
        purpose: SOURCE_REVIEW_STATEMENT_PURPOSE.to_string(),
        profile_id: SOURCE_REVIEW_PROFILE_ID.to_string(),
        subject: artifact_ref(RefField {
            profile: RELEASE_SOURCE_PROFILE,
            digest_hex: &attachment.subject.source_archive_digest_blake3,
        }),
        parents: vec![
            artifact_ref(RefField {
                profile: CLAIM_ROOT_PROFILE,
                digest_hex: &attachment.subject.claim_root_blake3,
            }),
            artifact_ref(RefField {
                profile: REVIEW_POLICY_PROFILE,
                digest_hex: &attachment.subject.review_policy_digest_blake3,
            }),
        ],
        verifier_context: artifact_ref(RefField {
            profile: VALENCE_EVIDENCE_PROFILE,
            digest_hex: &attachment.valence_evidence_blake3,
        }),
    };
    Ok(artifact_auth_core::ArtifactStatement {
        schema: artifact_auth_core::STATEMENT_SCHEMA_V1.to_string(),
        scope,
        producer_id: attachment.producer_obligation_id.clone(),
        key_id: approval.reviewer_label.clone(),
        key_identity: artifact_auth_core::ArtifactRef {
            profile: artifact_auth_core::ED25519_PUBLIC_KEY_PROFILE_V1.to_string(),
            algorithm: artifact_auth_core::ALGORITHM_BLAKE3.to_string(),
            digest_hex: blake3_hex_of(&public_key_bytes),
        },
    })
}

fn blake3_hex_of(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if !value.len().is_multiple_of(2) {
        return Err("odd length".to_string());
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let chars = value.as_bytes();
    let mut index = 0;
    while index < chars.len() {
        let high = hex_digit_value(chars[index])?;
        let low = hex_digit_value(chars[index.checked_add(1).ok_or("hex index overflow")?])?;
        let combined = high.checked_mul(16).and_then(|shifted| shifted.checked_add(low));
        bytes.push(combined.ok_or_else(|| "hex digit pair overflow".to_string())?);
        index = index.checked_add(2).ok_or_else(|| "hex index overflow".to_string())?;
    }
    debug_assert_eq!(bytes.len(), value.len() / 2);
    Ok(bytes)
}

fn hex_digit_value(byte: u8) -> Result<u8, String> {
    match byte {
        b'0' => Ok(0),
        b'1' => Ok(1),
        b'2' => Ok(2),
        b'3' => Ok(3),
        b'4' => Ok(4),
        b'5' => Ok(5),
        b'6' => Ok(6),
        b'7' => Ok(7),
        b'8' => Ok(8),
        b'9' => Ok(9),
        b'a' => Ok(10),
        b'b' => Ok(11),
        b'c' => Ok(12),
        b'd' => Ok(13),
        b'e' => Ok(14),
        b'f' => Ok(15),
        _ => Err(format!("invalid hex digit '{}'", HEX_DIGITS[0])),
    }
}

/// Evaluate source-review evidence against the selected policy.
///
/// The function is pure: it consumes explicit values and shell-verified
/// cryptographic observations and returns a deterministic result. Each phase
/// is delegated to a focused helper so every decision stays locally visible.
pub fn evaluate_source_review_release_evidence(input: &SourceReviewVerificationInput<'_>) -> SourceReviewVerification {
    let policy_blake3 = match reviewed_source_policy_identity(input.policy) {
        Ok(identity) => identity,
        Err(error) => {
            return failed_verification(input.mode, String::new(), String::new(), "policy-invalid", vec![
                error.to_string(),
            ]);
        }
    };
    debug_assert_eq!(policy_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    if input.mode == SourceReviewMode::Optional && input.attachment.is_none() {
        return not_required_verification(policy_blake3);
    }
    let Some(attachment) = input.attachment else {
        return failed_verification(input.mode, policy_blake3, String::new(), "missing-attachment", vec![
            "reviewed-source policy is required but no attachment was supplied".to_string(),
        ]);
    };
    let attachment_blake3 = attachment.attachment_blake3.clone();
    let context = ReviewFailureContext {
        mode: input.mode,
        policy_blake3: policy_blake3.clone(),
        attachment_blake3: attachment_blake3.clone(),
    };
    if let Some(failure) = attachment_preflight_failure(attachment, &input.cryptographic_observations, &context) {
        return failure;
    }
    if let Some(failure) = subject_link_failure(input, attachment, &context) {
        return failure;
    }
    if let Some(failure) = role_domain_failure(attachment, &context) {
        return failure;
    }
    let mut diagnostics = Vec::new();
    let retained = match select_distinct_approvals(attachment, input.policy, &mut diagnostics) {
        Ok(retained) => retained,
        Err(SelectionFailure::NeedsRevision(reviewer)) => {
            return needs_revision_failure(&context, reviewer, &mut diagnostics);
        }
    };
    if retained.is_empty() {
        return context.fail(
            "insufficient-approvals",
            diagnostics_with(&mut diagnostics, "no distinct approvals remain after filtering".to_string()),
        );
    }
    let evidence = match build_signature_evidence(attachment, &retained, &input.cryptographic_observations) {
        Ok(evidence) => evidence,
        Err(error) => {
            return context.fail("malformed-attachment", diagnostics_with(&mut diagnostics, error.to_string()));
        }
    };
    debug_assert_eq!(evidence.len(), retained.len());
    finish_authentication(input, attachment, &context, &evidence, diagnostics)
}

fn needs_revision_failure(
    context: &ReviewFailureContext,
    reviewer: String,
    diagnostics: &mut Vec<String>,
) -> SourceReviewVerification {
    debug_assert!(!reviewer.is_empty());
    context.fail(
        "needs-revision",
        diagnostics_with(
            diagnostics,
            format!("reviewer '{reviewer}' recorded needs-revision for the current claim root"),
        ),
    )
}

fn finish_authentication(
    input: &SourceReviewVerificationInput<'_>,
    attachment: &SourceReviewAttachment,
    context: &ReviewFailureContext,
    evidence: &[artifact_auth_core::SignatureEvidence],
    mut diagnostics: Vec<String>,
) -> SourceReviewVerification {
    debug_assert!(!evidence.is_empty());
    let decision = evaluate_authentications(input, attachment, &context.policy_blake3, evidence);
    for issue in &decision.issues {
        push_diagnostic(&mut diagnostics, format!("{}: {}", issue.code, issue.field));
    }
    if !decision.passed {
        let reason = classify_auth_issues(&decision.issues);
        return context.fail(reason, diagnostics);
    }
    let status = if input.mode.is_required() {
        SOURCE_REVIEW_STATUS_SATISFIED
    } else {
        SOURCE_REVIEW_STATUS_VERIFIED_OPTIONAL
    };
    debug_assert!(!decision.verified_key_blake3.is_empty());
    SourceReviewVerification {
        required: input.mode.is_required(),
        valid: true,
        status: status.to_string(),
        reason_code: None,
        attachment_blake3: Some(context.attachment_blake3.clone()),
        policy_blake3: context.policy_blake3.clone(),
        counted_reviewer_key_blake3: decision.verified_key_blake3,
        boundary: SOURCE_REVIEW_BOUNDARY.to_string(),
        non_claims: required_non_claims_sorted(),
        diagnostics,
    }
}

fn not_required_verification(policy_blake3: String) -> SourceReviewVerification {
    debug_assert_eq!(policy_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    SourceReviewVerification {
        required: false,
        valid: true,
        status: SOURCE_REVIEW_STATUS_NOT_REQUIRED.to_string(),
        reason_code: None,
        attachment_blake3: None,
        policy_blake3,
        counted_reviewer_key_blake3: Vec::new(),
        boundary: SOURCE_REVIEW_BOUNDARY.to_string(),
        non_claims: required_non_claims_sorted(),
        diagnostics: Vec::new(),
    }
}

fn classify_attachment_structure_failure(attachment: &SourceReviewAttachment) -> &'static str {
    if attachment.schema != SOURCE_REVIEW_ATTACHMENT_SCHEMA {
        "unsupported-schema"
    } else if attachment.attachment_blake3 != attachment_identity_excluding_field(attachment) {
        "tampered-attachment-identity"
    } else {
        "malformed-attachment"
    }
}

fn subject_link_failure(
    input: &SourceReviewVerificationInput<'_>,
    attachment: &SourceReviewAttachment,
    context: &ReviewFailureContext,
) -> Option<SourceReviewVerification> {
    debug_assert_eq!(context.policy_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert!(!context.attachment_blake3.is_empty());
    if attachment.subject.source_archive_digest_blake3 != input.expected_source_archive_digest_blake3 {
        return Some(context.fail("stale-source-subject", vec![format!(
            "attachment binds source archive {} but the release source archive is {}",
            attachment.subject.source_archive_digest_blake3, input.expected_source_archive_digest_blake3
        )]));
    }
    if attachment.subject.claim_root_blake3 != input.expected_claim_root_blake3 {
        return Some(context.fail("stale-claim-root", vec![format!(
            "attachment binds claim root {} but the current claim root is {}",
            attachment.subject.claim_root_blake3, input.expected_claim_root_blake3
        )]));
    }
    if let Some(diagnostic) = source_revision_link_failure(
        input.expected_source_revision.as_deref(),
        attachment.subject.source_revision.as_deref(),
    ) {
        return Some(context.fail("stale-source-subject", vec![diagnostic]));
    }
    if attachment.subject.review_policy_digest_blake3 != context.policy_blake3 {
        return Some(context.fail("policy-mismatch", vec![format!(
            "attachment binds review policy {} but the selected policy identity is {}",
            attachment.subject.review_policy_digest_blake3, context.policy_blake3
        )]));
    }
    None
}

// r[impl mantle.release_provenance.source_observation_binding]
fn source_revision_link_failure(expected: Option<&str>, actual: Option<&str>) -> Option<String> {
    match (expected, actual) {
        (Some(expected), Some(actual)) if expected != actual => {
            Some(format!("attachment binds source revision {actual} but the release names {expected}"))
        }
        (Some(expected), None) => Some(format!("attachment omits source revision but the release names {expected}")),
        (None, Some(actual)) => {
            Some(format!("attachment binds source revision {actual} but the release names no immutable revision"))
        }
        (Some(_), Some(_)) | (None, None) => None,
    }
}

fn role_domain_failure(
    attachment: &SourceReviewAttachment,
    context: &ReviewFailureContext,
) -> Option<SourceReviewVerification> {
    debug_assert_eq!(context.policy_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert!(!context.attachment_blake3.is_empty());
    for approval in &attachment.approvals {
        if approval.statement_domain == BUILD_WITNESS_STATEMENT_DOMAIN {
            return Some(context.fail("role-confusion", vec![format!(
                "approval from '{}' carries the build-witness domain and cannot count as source review",
                approval.reviewer_label
            )]));
        }
        if approval.statement_domain != SOURCE_REVIEW_STATEMENT_DOMAIN {
            return Some(context.fail("role-confusion", vec![format!(
                "approval from '{}' carries unsupported statement domain '{}'",
                approval.reviewer_label, approval.statement_domain
            )]));
        }
    }
    None
}

/// Mode and identity context shared by every focused evaluation phase.
struct ReviewFailureContext {
    mode: SourceReviewMode,
    policy_blake3: String,
    attachment_blake3: String,
}

impl ReviewFailureContext {
    fn fail(&self, reason_code: &str, diagnostics: Vec<String>) -> SourceReviewVerification {
        failed_verification(
            self.mode,
            self.policy_blake3.clone(),
            self.attachment_blake3.clone(),
            reason_code,
            diagnostics,
        )
    }
}

fn attachment_preflight_failure(
    attachment: &SourceReviewAttachment,
    observations: &[artifact_auth_core::CryptographicObservation],
    context: &ReviewFailureContext,
) -> Option<SourceReviewVerification> {
    debug_assert_eq!(context.policy_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    if let Err(error) = validate_source_review_attachment_structure(attachment) {
        let reason = classify_attachment_structure_failure(attachment);
        return Some(failed_verification(context.mode, context.policy_blake3.clone(), String::new(), reason, vec![
            error.to_string(),
        ]));
    }
    if observations.len() != attachment.approvals.len() {
        return Some(context.fail("malformed-attachment", vec![format!(
            "cryptographic observation count {} does not match approval count {}",
            observations.len(),
            attachment.approvals.len()
        )]));
    }
    None
}

fn select_distinct_approvals(
    attachment: &SourceReviewAttachment,
    policy: &ReviewedSourcePolicy,
    diagnostics: &mut Vec<String>,
) -> Result<Vec<usize>, SelectionFailure> {
    let excluded_author = policy.excluded_author_public_key_hex.clone();
    let mut retained = Vec::with_capacity(attachment.approvals.len());
    let mut seen_keys = alloc::collections::BTreeSet::new();
    for (index, approval) in attachment.approvals.iter().enumerate() {
        if let Some(excluded) = &excluded_author
            && *excluded == approval.public_key_hex
        {
            push_diagnostic(diagnostics, format!("approval {index} from excluded author key was filtered"));
            continue;
        }
        if approval.disposition == SOURCE_REVIEW_DISPOSITION_NEEDS_REVISION {
            return Err(SelectionFailure::NeedsRevision(approval.reviewer_label.clone()));
        }
        if !seen_keys.insert(approval.public_key_hex.clone()) {
            push_diagnostic(
                diagnostics,
                format!("duplicate approval for key {} counted once", approval.public_key_hex),
            );
            continue;
        }
        retained.push(index);
    }
    debug_assert!(retained.len() <= attachment.approvals.len());
    Ok(retained)
}

/// Mid-selection failure that the caller renders with full mode and identity context.
enum SelectionFailure {
    NeedsRevision(String),
}

fn build_signature_evidence(
    attachment: &SourceReviewAttachment,
    retained: &[usize],
    observations: &[artifact_auth_core::CryptographicObservation],
) -> Result<Vec<artifact_auth_core::SignatureEvidence>, String> {
    let mut evidence = Vec::with_capacity(retained.len());
    for index in retained {
        let approval =
            attachment.approvals.get(*index).ok_or_else(|| format!("approval index {index} is out of bounds"))?;
        let statement = map_source_review_statement(attachment, approval).map_err(|error| error.to_string())?;
        let observation = observations
            .get(*index)
            .ok_or_else(|| format!("cryptographic observation index {index} is out of bounds"))?;
        evidence.push(artifact_auth_core::SignatureEvidence {
            statement,
            generation: approval.generation,
            cryptographic: observation.clone(),
        });
    }
    debug_assert_eq!(evidence.len(), retained.len());
    Ok(evidence)
}

fn evaluate_authentications(
    input: &SourceReviewVerificationInput<'_>,
    attachment: &SourceReviewAttachment,
    policy_blake3: &str,
    evidence: &[artifact_auth_core::SignatureEvidence],
) -> artifact_auth_core::AuthenticationDecision {
    debug_assert!(!attachment.attachment_blake3.is_empty());
    let scope = artifact_auth_core::AuthenticationScope {
        domain: SOURCE_REVIEW_STATEMENT_DOMAIN.to_string(),
        purpose: SOURCE_REVIEW_STATEMENT_PURPOSE.to_string(),
        profile_id: SOURCE_REVIEW_PROFILE_ID.to_string(),
        subject: artifact_ref(RefField {
            profile: RELEASE_SOURCE_PROFILE,
            digest_hex: &attachment.subject.source_archive_digest_blake3,
        }),
        parents: vec![
            artifact_ref(RefField {
                profile: CLAIM_ROOT_PROFILE,
                digest_hex: &attachment.subject.claim_root_blake3,
            }),
            artifact_ref(RefField {
                profile: REVIEW_POLICY_PROFILE,
                digest_hex: &attachment.subject.review_policy_digest_blake3,
            }),
        ],
        verifier_context: artifact_ref(RefField {
            profile: VALENCE_EVIDENCE_PROFILE,
            digest_hex: &attachment.valence_evidence_blake3,
        }),
    };
    let trusted_keys = input
        .policy
        .reviewers
        .iter()
        .map(|reviewer| trusted_key_observation(reviewer, policy_blake3))
        .collect::<Vec<_>>();
    let auth_policy = artifact_auth_core::AuthenticationPolicy {
        schema: artifact_auth_core::POLICY_SCHEMA_V1.to_string(),
        profile_id: SOURCE_REVIEW_PROFILE_ID.to_string(),
        threshold: threshold_for_mode(input.mode, input.policy.required_distinct_reviewers),
        trusted_keys,
    };
    debug_assert!(!evidence.is_empty());
    artifact_auth_core::evaluate_authentication(&auth_policy, &scope, evidence)
}

fn trusted_key_observation(
    reviewer: &TrustedReviewerKey,
    policy_blake3: &str,
) -> artifact_auth_core::TrustedKeyObservation {
    artifact_auth_core::TrustedKeyObservation {
        producer_id: reviewer.producer_id.clone(),
        key_id: reviewer.label.clone(),
        key_identity: artifact_auth_core::ArtifactRef {
            profile: artifact_auth_core::ED25519_PUBLIC_KEY_PROFILE_V1.to_string(),
            algorithm: artifact_auth_core::ALGORITHM_BLAKE3.to_string(),
            digest_hex: blake3_hex_of(&decode_hex(&reviewer.public_key_hex).unwrap_or_default()),
        },
        allowed_purposes: vec![SOURCE_REVIEW_STATEMENT_PURPOSE.to_string()],
        generation: REVIEWER_KEY_GENERATION,
        currentness: reviewer.currentness.to_artifact_auth(),
        currentness_ref: artifact_ref(RefField {
            profile: POLICY_CURRENTNESS_PROFILE,
            digest_hex: policy_blake3,
        }),
    }
}

fn threshold_for_mode(mode: SourceReviewMode, policy_threshold: u16) -> u16 {
    if mode.is_required() {
        policy_threshold
    } else {
        let _ = policy_threshold;
        1
    }
}

fn classify_auth_issues(issues: &[artifact_auth_core::AuthenticationIssue]) -> &'static str {
    const ISSUE_CLASSES: [(&str, [&str; 2]); 4] = [
        ("signature-invalid", ["signature_unverified", "cryptographic"]),
        ("unknown-reviewer-key", ["key_untrusted", "key_untrusted"]),
        ("reviewer-key-not-current", ["key_not_current", "key_not_current"]),
        ("insufficient-approvals", ["threshold", "threshold"]),
    ];
    for (class, fragments) in ISSUE_CLASSES {
        if has_issue_fragment(issues, &fragments) {
            return class;
        }
    }
    "signature-invalid"
}

fn has_issue_fragment(issues: &[artifact_auth_core::AuthenticationIssue], fragments: &[&str]) -> bool {
    debug_assert!(!fragments.is_empty());
    issues.iter().any(|issue| issue_has_fragment(issue, fragments))
}

fn issue_has_fragment(issue: &artifact_auth_core::AuthenticationIssue, fragments: &[&str]) -> bool {
    fragments.iter().any(|fragment| issue.code.contains(fragment))
}

fn diagnostics_with(diagnostics: &mut Vec<String>, diagnostic: String) -> Vec<String> {
    push_diagnostic(diagnostics, diagnostic);
    diagnostics.clone()
}

fn push_diagnostic(diagnostics: &mut Vec<String>, diagnostic: String) {
    let Ok(count) = u32::try_from(diagnostics.len()) else {
        return;
    };
    if count < MAX_SOURCE_REVIEW_DIAGNOSTICS_COUNT {
        diagnostics.push(diagnostic);
    }
}

fn failed_verification(
    mode: SourceReviewMode,
    policy_blake3: String,
    attachment_blake3: String,
    reason_code: &str,
    diagnostics: Vec<String>,
) -> SourceReviewVerification {
    let attachment = (!attachment_blake3.is_empty()).then_some(attachment_blake3);
    SourceReviewVerification {
        required: mode.is_required(),
        valid: false,
        status: SOURCE_REVIEW_STATUS_FAILED.to_string(),
        reason_code: Some(reason_code.to_string()),
        attachment_blake3: attachment,
        policy_blake3,
        counted_reviewer_key_blake3: Vec::new(),
        boundary: SOURCE_REVIEW_BOUNDARY.to_string(),
        non_claims: required_non_claims_sorted(),
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const CLAIM_ROOT_A: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const VALENCE_A: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const KEY_A_HEX: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const KEY_B_HEX: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const KEY_C_HEX: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    fn sig_hex(byte: char) -> String {
        alloc::string::ToString::to_string(&byte).repeat(128)
    }
    const PRODUCER_ID: &str = "cairn-verification-obligation-1";

    fn policy_two_reviewers() -> ReviewedSourcePolicy {
        ReviewedSourcePolicy {
            schema: REVIEWED_SOURCE_POLICY_SCHEMA.to_string(),
            preset: Some(REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER.to_string()),
            required_distinct_reviewers: STAGEX_TWO_REVIEWER_THRESHOLD,
            reviewers: vec![
                TrustedReviewerKey {
                    producer_id: PRODUCER_ID.to_string(),
                    label: "reviewer-a".to_string(),
                    public_key_hex: KEY_A_HEX.to_string(),
                    currentness: ReviewerCurrentness::Current,
                },
                TrustedReviewerKey {
                    producer_id: PRODUCER_ID.to_string(),
                    label: "reviewer-b".to_string(),
                    public_key_hex: KEY_B_HEX.to_string(),
                    currentness: ReviewerCurrentness::Current,
                },
            ],
            excluded_author_public_key_hex: None,
        }
    }

    fn approval(label: &str, key: &str, signature: &str) -> SourceReviewApprovalStatement {
        SourceReviewApprovalStatement {
            statement_domain: SOURCE_REVIEW_STATEMENT_DOMAIN.to_string(),
            reviewer_label: label.to_string(),
            public_key_hex: key.to_string(),
            signature_hex: signature.to_string(),
            disposition: SOURCE_REVIEW_DISPOSITION_APPROVED.to_string(),
            generation: REVIEWER_KEY_GENERATION,
        }
    }

    fn attachment(approvals: Vec<SourceReviewApprovalStatement>, policy_digest: &str) -> SourceReviewAttachment {
        let attachment = SourceReviewAttachment {
            schema: SOURCE_REVIEW_ATTACHMENT_SCHEMA.to_string(),
            subject: SourceReviewSubjectLink {
                source_archive_digest_blake3: SOURCE_DIGEST_A.to_string(),
                source_revision: None,
                claim_root_blake3: CLAIM_ROOT_A.to_string(),
                review_policy_digest_blake3: policy_digest.to_string(),
            },
            producer_obligation_id: PRODUCER_ID.to_string(),
            producer_disposition: "approved".to_string(),
            valence_evidence_blake3: VALENCE_A.to_string(),
            approvals,
            non_claims: required_non_claims_sorted(),
            attachment_blake3: String::new(),
        };
        let identity = attachment_identity_excluding_field(&attachment);
        SourceReviewAttachment {
            attachment_blake3: identity,
            ..attachment
        }
    }

    fn verified_observation(key_hex: &str) -> artifact_auth_core::CryptographicObservation {
        artifact_auth_core::CryptographicObservation {
            algorithm: artifact_auth_core::ALGORITHM_ED25519.to_string(),
            key_identity: artifact_auth_core::ArtifactRef {
                profile: artifact_auth_core::ED25519_PUBLIC_KEY_PROFILE_V1.to_string(),
                algorithm: artifact_auth_core::ALGORITHM_BLAKE3.to_string(),
                digest_hex: blake3_hex_of(&decode_hex(key_hex).unwrap()),
            },
            verified: true,
            failure_code: None,
        }
    }

    fn valid_input<'a>(
        mode: SourceReviewMode,
        policy: &'a ReviewedSourcePolicy,
        attachment_value: &'a SourceReviewAttachment,
        observations: Vec<artifact_auth_core::CryptographicObservation>,
    ) -> SourceReviewVerificationInput<'a> {
        SourceReviewVerificationInput {
            mode,
            policy,
            expected_source_archive_digest_blake3: SOURCE_DIGEST_A.to_string(),
            expected_claim_root_blake3: CLAIM_ROOT_A.to_string(),
            expected_source_revision: None,
            attachment: Some(attachment_value),
            cryptographic_observations: observations,
        }
    }

    fn two_valid_observations() -> Vec<artifact_auth_core::CryptographicObservation> {
        vec![verified_observation(KEY_A_HEX), verified_observation(KEY_B_HEX)]
    }

    #[test]
    fn optional_mode_without_attachment_is_not_required() {
        let policy = policy_two_reviewers();
        let input = SourceReviewVerificationInput {
            mode: SourceReviewMode::Optional,
            policy: &policy,
            expected_source_archive_digest_blake3: SOURCE_DIGEST_A.to_string(),
            expected_claim_root_blake3: CLAIM_ROOT_A.to_string(),
            expected_source_revision: None,
            attachment: None,
            cryptographic_observations: Vec::new(),
        };
        let result = evaluate_source_review_release_evidence(&input);
        assert!(result.valid);
        assert_eq!(result.status, SOURCE_REVIEW_STATUS_NOT_REQUIRED);
        assert!(result.reason_code.is_none());
        assert!(result.counted_reviewer_key_blake3.is_empty());
    }

    #[test]
    fn required_mode_with_two_distinct_reviewers_is_satisfied() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(
            vec![
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
                approval("reviewer-b", KEY_B_HEX, &sig_hex('b')),
            ],
            &policy_digest,
        );
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, two_valid_observations());
        let result = evaluate_source_review_release_evidence(&input);
        assert!(result.valid);
        assert_eq!(result.status, SOURCE_REVIEW_STATUS_SATISFIED);
        assert_eq!(result.counted_reviewer_key_blake3.len(), 2);
        assert_eq!(result.attachment_blake3.as_deref(), Some(value.attachment_blake3.as_str()));
    }

    #[test]
    fn required_mode_without_attachment_fails_missing() {
        let policy = policy_two_reviewers();
        let input = SourceReviewVerificationInput {
            mode: SourceReviewMode::RequiredReviewedSource,
            policy: &policy,
            expected_source_archive_digest_blake3: SOURCE_DIGEST_A.to_string(),
            expected_claim_root_blake3: CLAIM_ROOT_A.to_string(),
            expected_source_revision: None,
            attachment: None,
            cryptographic_observations: Vec::new(),
        };
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("missing-attachment"));
        assert!(!result.valid);
    }

    fn evaluate_source_verification_fail(input: &SourceReviewVerificationInput<'_>) -> SourceReviewVerification {
        let result = evaluate_source_review_release_evidence(input);
        assert!(!result.valid);
        assert_eq!(result.status, SOURCE_REVIEW_STATUS_FAILED);
        result
    }

    #[test]
    fn optional_mode_with_valid_attachment_reports_verified_optional() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        let input = valid_input(SourceReviewMode::Optional, &policy, &value, vec![verified_observation(KEY_A_HEX)]);
        let result = evaluate_source_review_release_evidence(&input);
        assert!(result.valid);
        assert_eq!(result.status, SOURCE_REVIEW_STATUS_VERIFIED_OPTIONAL);
        assert!(!result.required);
    }

    #[test]
    fn wrong_source_digest_fails_stale_subject() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        let input = SourceReviewVerificationInput {
            mode: SourceReviewMode::RequiredReviewedSource,
            policy: &policy,
            expected_source_archive_digest_blake3: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                .to_string(),
            expected_claim_root_blake3: CLAIM_ROOT_A.to_string(),
            expected_source_revision: None,
            attachment: Some(&value),
            cryptographic_observations: vec![verified_observation(KEY_A_HEX)],
        };
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("stale-source-subject"));
    }

    #[test]
    fn stale_claim_root_fails() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        let mut input =
            valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
                KEY_A_HEX,
            )]);
        input.expected_claim_root_blake3 =
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string();
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("stale-claim-root"));
    }

    #[test]
    fn policy_digest_mismatch_fails() {
        let policy = policy_two_reviewers();
        let value = attachment(
            vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))],
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
            KEY_A_HEX,
        )]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("policy-mismatch"));
    }

    #[test]
    fn unknown_reviewer_key_fails() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(
            vec![
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
                approval("reviewer-c", KEY_C_HEX, &sig_hex('b')),
            ],
            &policy_digest,
        );
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![
            verified_observation(KEY_A_HEX),
            verified_observation(KEY_C_HEX),
        ]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("unknown-reviewer-key"));
    }

    #[test]
    fn revoked_reviewer_key_fails() {
        let mut policy = policy_two_reviewers();
        policy.reviewers[1].currentness = ReviewerCurrentness::Revoked;
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(
            vec![
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
                approval("reviewer-b", KEY_B_HEX, &sig_hex('b')),
            ],
            &policy_digest,
        );
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, two_valid_observations());
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("reviewer-key-not-current"));
    }

    #[test]
    fn excluded_author_approval_does_not_count() {
        let mut policy = policy_two_reviewers();
        policy.excluded_author_public_key_hex = Some(KEY_A_HEX.to_string());
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(
            vec![
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
                approval("reviewer-b", KEY_B_HEX, &sig_hex('b')),
            ],
            &policy_digest,
        );
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, two_valid_observations());
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("insufficient-approvals"));
    }

    #[test]
    fn duplicate_key_under_two_labels_does_not_inflate() {
        let mut policy = policy_two_reviewers();
        policy.preset = None;
        policy.reviewers.push(TrustedReviewerKey {
            producer_id: PRODUCER_ID.to_string(),
            label: "reviewer-a-copy".to_string(),
            public_key_hex: KEY_A_HEX.to_string(),
            currentness: ReviewerCurrentness::Current,
        });
        let policy_digest = reviewed_source_policy_identity(&policy);
        assert!(policy_digest.is_err(), "duplicate policy keys must fail validation");
    }

    #[test]
    fn duplicate_signature_for_one_key_counts_once() {
        let mut policy = policy_two_reviewers();
        policy.preset = None;
        policy.required_distinct_reviewers = 2;
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(
            vec![
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
                approval("reviewer-b", KEY_B_HEX, &sig_hex('b')),
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
            ],
            &policy_digest,
        );
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![
            verified_observation(KEY_A_HEX),
            verified_observation(KEY_B_HEX),
            verified_observation(KEY_A_HEX),
        ]);
        let result = evaluate_source_review_release_evidence(&input);
        assert!(result.valid);
        assert_eq!(result.counted_reviewer_key_blake3.len(), 2);
    }

    #[test]
    fn needs_revision_disposition_fails_selected_policy() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let mut revision = approval("reviewer-b", KEY_B_HEX, &sig_hex('b'));
        revision.disposition = SOURCE_REVIEW_DISPOSITION_NEEDS_REVISION.to_string();
        let value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a')), revision], &policy_digest);
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, two_valid_observations());
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("needs-revision"));
    }

    #[test]
    fn invalid_signature_observation_fails() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(
            vec![
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
                approval("reviewer-b", KEY_B_HEX, &sig_hex('b')),
            ],
            &policy_digest,
        );
        let mut invalid = verified_observation(KEY_B_HEX);
        invalid.verified = false;
        invalid.failure_code = Some("signature_invalid".to_string());
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![
            verified_observation(KEY_A_HEX),
            invalid,
        ]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("signature-invalid"));
    }

    #[test]
    fn build_witness_domain_cannot_satisfy_review_policy() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let mut witness = approval("reviewer-b", KEY_B_HEX, &sig_hex('b'));
        witness.statement_domain = BUILD_WITNESS_STATEMENT_DOMAIN.to_string();
        let value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a')), witness], &policy_digest);
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, two_valid_observations());
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("role-confusion"));
    }

    #[test]
    fn insufficient_distinct_approvals_fail() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
            KEY_A_HEX,
        )]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("insufficient-approvals"));
    }

    #[test]
    fn tampered_attachment_identity_fails() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let mut value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        value.attachment_blake3 = "0".repeat(64);
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
            KEY_A_HEX,
        )]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("tampered-attachment-identity"));
    }

    #[test]
    fn unsupported_schema_fails() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let mut value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        value.schema = "mantle.source-review.attachment.v0".to_string();
        value.attachment_blake3 = attachment_identity_excluding_field(&value);
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
            KEY_A_HEX,
        )]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("unsupported-schema"));
    }

    #[test]
    fn source_revision_mismatch_fails() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let mut value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        value.subject.source_revision = Some("revision-old".to_string());
        value.attachment_blake3 = attachment_identity_excluding_field(&value);
        let mut input =
            valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
                KEY_A_HEX,
            )]);
        input.expected_source_revision = Some("revision-new".to_string());
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("stale-source-subject"));
    }

    #[test]
    fn source_revision_presence_must_match_release_subject_exactly() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let missing = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        let mut missing_input =
            valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &missing, vec![verified_observation(
                KEY_A_HEX,
            )]);
        missing_input.expected_source_revision = Some("revision-required".to_string());
        let missing_result = evaluate_source_verification_fail(&missing_input);

        let mut unexpected = missing.clone();
        unexpected.subject.source_revision = Some("revision-unexpected".to_string());
        unexpected.attachment_blake3 = attachment_identity_excluding_field(&unexpected);
        let unexpected_input =
            valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &unexpected, vec![verified_observation(
                KEY_A_HEX,
            )]);
        let unexpected_result = evaluate_source_verification_fail(&unexpected_input);
        assert_eq!(missing_result.reason_code.as_deref(), Some("stale-source-subject"));
        assert_eq!(unexpected_result.reason_code.as_deref(), Some("stale-source-subject"));
    }

    #[test]
    fn missing_required_non_claim_fails_validation() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let mut value = attachment(vec![approval("reviewer-a", KEY_A_HEX, &sig_hex('a'))], &policy_digest);
        value.non_claims.retain(|claim| claim != "not reviewer competence");
        value.attachment_blake3 = attachment_identity_excluding_field(&value);
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
            KEY_A_HEX,
        )]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("malformed-attachment"));
    }

    #[test]
    fn policy_with_duplicate_reviewer_keys_is_rejected() {
        let mut policy = policy_two_reviewers();
        policy.reviewers.push(TrustedReviewerKey {
            producer_id: PRODUCER_ID.to_string(),
            label: "reviewer-a-again".to_string(),
            public_key_hex: KEY_A_HEX.to_string(),
            currentness: ReviewerCurrentness::Current,
        });
        assert!(reviewed_source_policy_identity(&policy).is_err());
    }

    #[test]
    fn policy_threshold_cannot_exceed_reviewer_count() {
        let mut policy = policy_two_reviewers();
        policy.preset = None;
        policy.required_distinct_reviewers = 3;
        assert!(reviewed_source_policy_identity(&policy).is_err());
    }

    #[test]
    fn policy_without_reviewers_is_valid_but_unsatisfiable() {
        let mut policy = policy_two_reviewers();
        policy.preset = None;
        policy.required_distinct_reviewers = 1;
        policy.reviewers.clear();
        assert!(reviewed_source_policy_identity(&policy).is_ok());
    }

    #[test]
    fn unknown_preset_is_rejected() {
        let mut policy = policy_two_reviewers();
        policy.preset = Some("one-reviewer".to_string());
        assert!(reviewed_source_policy_identity(&policy).is_err());
    }

    #[test]
    fn observation_count_must_match_approvals() {
        let policy = policy_two_reviewers();
        let policy_digest = reviewed_source_policy_identity(&policy).unwrap();
        let value = attachment(
            vec![
                approval("reviewer-a", KEY_A_HEX, &sig_hex('a')),
                approval("reviewer-b", KEY_B_HEX, &sig_hex('b')),
            ],
            &policy_digest,
        );
        let input = valid_input(SourceReviewMode::RequiredReviewedSource, &policy, &value, vec![verified_observation(
            KEY_A_HEX,
        )]);
        let result = evaluate_source_verification_fail(&input);
        assert_eq!(result.reason_code.as_deref(), Some("malformed-attachment"));
    }

    #[test]
    fn mode_parsing_rejects_unknown_values() {
        assert!(SourceReviewMode::parse("sometimes").is_err());
        assert_eq!(SourceReviewMode::parse(SOURCE_REVIEW_MODE_OPTIONAL).unwrap(), SourceReviewMode::Optional);
        assert_eq!(
            SourceReviewMode::parse(SOURCE_REVIEW_MODE_REQUIRED).unwrap(),
            SourceReviewMode::RequiredReviewedSource
        );
    }
}
