//! Shell adapter for release source-review attachment preparation and verification.
//!
//! The pure evaluation lives in `crunch_release_core::source_review`. This
//! module owns file reads, bundle placement, Ed25519 signature verification
//! through the admitted Artifact Auth boundary, and operator policy parsing.
//! Producer status fields never authorize a decision.
//!
//! r[impl verification_evidence.release_source_review_evidence]

use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::RELEASED_SOURCE_REVIEW_ATTACHMENT_RELATIVE_PATH;
use crunch_release_core::REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER;
use crunch_release_core::ReviewedSourcePolicy;
use crunch_release_core::SOURCE_REVIEW_MODE_OPTIONAL;
use crunch_release_core::STAGEX_TWO_REVIEWER_THRESHOLD;
use crunch_release_core::SourceReviewAttachment;
use crunch_release_core::SourceReviewMode;
use crunch_release_core::SourceReviewVerification;
use crunch_release_core::SourceReviewVerificationInput;
use crunch_release_core::TrustedReviewerKey;

use crate::cairn_release_handoff::read_bounded_file;
use crate::errors::RunError;
use crate::release_capability::ReleaseRootKind;

/// Maximum accepted source-review attachment size.
pub const MAX_SOURCE_REVIEW_ATTACHMENT_BYTES: u64 = 1 << 20;
/// Default producer identity for review statements.
pub const DEFAULT_REVIEW_PRODUCER_ID: &str = "cairn-verification-obligation";
/// Separates the reviewer label from the base64 key in CLI arguments.
const REVIEWER_ARG_SEPARATOR: char = ':';
/// Base64 decoder output capacity for one Ed25519 public key.
const ED25519_PUBLIC_KEY_BYTES: usize = 32;

/// One parsed reviewer-key CLI argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewerKeyArg {
    pub label: String,
    pub public_key_base64: String,
}

/// Operator inputs for one release verification's source-review policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReviewVerifyInputs {
    pub mode: String,
    pub preset: Option<String>,
    pub threshold: Option<u16>,
    pub trusted_reviewers: Vec<ReviewerKeyArg>,
    pub revoked_reviewers: Vec<ReviewerKeyArg>,
    pub author_key_base64: Option<String>,
    pub claim_root_blake3: Option<String>,
    pub producer_id: Option<String>,
    pub attachment_override: Option<PathBuf>,
}

impl Default for SourceReviewVerifyInputs {
    fn default() -> Self {
        Self {
            mode: SOURCE_REVIEW_MODE_OPTIONAL.to_string(),
            preset: None,
            threshold: None,
            trusted_reviewers: Vec::new(),
            revoked_reviewers: Vec::new(),
            author_key_base64: None,
            claim_root_blake3: None,
            producer_id: None,
            attachment_override: None,
        }
    }
}

/// Parse one `label:base64` reviewer argument.
pub fn parse_reviewer_arg(value: &str) -> Result<ReviewerKeyArg, RunError> {
    let (label, key) = value.split_once(REVIEWER_ARG_SEPARATOR).ok_or_else(|| {
        RunError::Internal(format!("reviewer argument '{value}' must use label:base64-public-key form"))
    })?;
    if label.is_empty() || key.is_empty() {
        return Err(RunError::Internal(format!(
            "reviewer argument '{value}' must provide a non-empty label and base64 public key"
        )));
    }
    Ok(ReviewerKeyArg {
        label: label.to_string(),
        public_key_base64: key.to_string(),
    })
}

fn decode_base64(value: &str, label: &str) -> Result<Vec<u8>, RunError> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|error| RunError::Internal(format!("decoding {label} base64 key: {error}")))
}

fn decode_hex(value: &str, label: &str) -> Result<Vec<u8>, RunError> {
    if !value.len().is_multiple_of(2) {
        return Err(RunError::Internal(format!("{label} must have an even hex length")));
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let chars = value.as_bytes();
    let mut index = 0;
    while index < chars.len() {
        let high = hex_digit(chars[index], label)?;
        let low = hex_digit(chars[index + 1], label)?;
        bytes.push((high << 4) | low);
        index += 2;
    }
    Ok(bytes)
}

fn hex_digit(byte: u8, label: &str) -> Result<u8, RunError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(RunError::Internal(format!("{label} contains a non-hex character"))),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(hex_char(byte >> 4));
        output.push(hex_char(byte & 0x0f));
    }
    output
}

const fn hex_char(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'a' + nibble - 10) as char,
    }
}

fn reviewer_public_key_hex(arg: &ReviewerKeyArg) -> Result<String, RunError> {
    let bytes = decode_base64(&arg.public_key_base64, "reviewer")?;
    if bytes.len() != ED25519_PUBLIC_KEY_BYTES {
        return Err(RunError::Internal(format!(
            "reviewer public key for '{}' must decode to {ED25519_PUBLIC_KEY_BYTES} bytes, got {}",
            arg.label,
            bytes.len()
        )));
    }
    Ok(encode_hex(&bytes))
}

/// Build the typed reviewed-source policy from operator inputs.
pub fn build_reviewed_source_policy(inputs: &SourceReviewVerifyInputs) -> Result<ReviewedSourcePolicy, RunError> {
    let mode = SourceReviewMode::parse(&inputs.mode)
        .map_err(|error| RunError::Internal(format!("invalid --reviewed-source mode: {error}")))?;
    let preset = inputs.preset.as_deref();
    let threshold = match preset {
        Some(name) => {
            if name != REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER {
                return Err(RunError::Internal(format!(
                    "unknown reviewed-source preset '{name}', expected '{REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER}'"
                )));
            }
            STAGEX_TWO_REVIEWER_THRESHOLD
        }
        None => match (inputs.threshold, mode.is_required()) {
            (Some(explicit), _) => explicit,
            (None, false) => 1,
            (None, true) => {
                return Err(RunError::Internal(
                    "reviewed-source policy requires --review-preset stagex-two-reviewer or an explicit --review-threshold"
                        .to_string(),
                ));
            }
        },
    };
    if threshold == 0 {
        return Err(RunError::Internal("reviewed-source threshold must be at least one distinct reviewer".to_string()));
    }
    let producer_id = inputs.producer_id.clone().unwrap_or_else(|| DEFAULT_REVIEW_PRODUCER_ID.to_string());
    let mut reviewers = Vec::with_capacity(inputs.trusted_reviewers.len() + inputs.revoked_reviewers.len());
    for arg in &inputs.trusted_reviewers {
        reviewers.push(TrustedReviewerKey {
            producer_id: producer_id.clone(),
            label: arg.label.clone(),
            public_key_hex: reviewer_public_key_hex(arg)?,
            currentness: crunch_release_core::ReviewerCurrentness::Current,
        });
    }
    for arg in &inputs.revoked_reviewers {
        reviewers.push(TrustedReviewerKey {
            producer_id: producer_id.clone(),
            label: arg.label.clone(),
            public_key_hex: reviewer_public_key_hex(arg)?,
            currentness: crunch_release_core::ReviewerCurrentness::Revoked,
        });
    }
    if mode.is_required() && reviewers.is_empty() {
        return Err(RunError::Internal(
            "reviewed-source policy requires at least one --trusted-reviewer or --revoked-reviewer key".to_string(),
        ));
    }
    let excluded_author_public_key_hex = match &inputs.author_key_base64 {
        Some(value) => {
            let bytes = decode_base64(value, "review author")?;
            if bytes.len() != ED25519_PUBLIC_KEY_BYTES {
                return Err(RunError::Internal(format!(
                    "review author key must decode to {ED25519_PUBLIC_KEY_BYTES} bytes, got {}",
                    bytes.len()
                )));
            }
            Some(encode_hex(&bytes))
        }
        None => None,
    };
    let policy = ReviewedSourcePolicy {
        schema: crunch_release_core::REVIEWED_SOURCE_POLICY_SCHEMA.to_string(),
        preset: inputs.preset.clone(),
        required_distinct_reviewers: threshold,
        reviewers,
        excluded_author_public_key_hex,
    };
    crunch_release_core::validate_reviewed_source_policy(&policy)
        .map_err(|error| RunError::Internal(format!("invalid reviewed-source policy: {error}")))?;
    Ok(policy)
}

/// Read and structurally validate a source-review attachment file.
pub fn read_source_review_attachment(path: &Path) -> Result<SourceReviewAttachment, RunError> {
    let attachment = read_source_review_attachment_unvalidated(path)?;
    crunch_release_core::validate_source_review_attachment_structure(&attachment)
        .map_err(|error| RunError::Internal(format!("invalid source review attachment: {error}")))?;
    debug_assert!(!attachment.attachment_blake3.is_empty());
    Ok(attachment)
}

/// Read and parse a source-review attachment without structural validation.
///
/// Verification-time structural failures are classified by the pure core so
/// the release decision report still carries the exact reason code.
fn read_source_review_attachment_unvalidated(path: &Path) -> Result<SourceReviewAttachment, RunError> {
    let bytes = read_bounded_file(
        path,
        MAX_SOURCE_REVIEW_ATTACHMENT_BYTES,
        "Source review attachment",
        ReleaseRootKind::BuildArtifact,
    )?;
    crunch_release_core::parse_source_review_attachment(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing source review attachment: {error}")))
}

/// Verify every approval signature through the Artifact Auth Ed25519 boundary.
///
/// Decode or mapping failures become deterministic failed observations so the
/// pure core can classify them instead of aborting the decision report.
fn verify_attachment_signatures(
    attachment: &SourceReviewAttachment,
) -> Vec<artifact_auth_core::CryptographicObservation> {
    let mut observations = Vec::with_capacity(attachment.approvals.len());
    for approval in &attachment.approvals {
        let statement = crunch_release_core::map_source_review_statement(attachment, approval).ok();
        let public_key_bytes = decode_hex(&approval.public_key_hex, "reviewer public key").ok();
        let signature_bytes = decode_hex(&approval.signature_hex, "review signature").ok();
        let observation = match (statement, public_key_bytes, signature_bytes) {
            (Some(statement), Some(public_key_bytes), Some(signature_bytes)) => {
                artifact_auth_ed25519::verify_statement(&statement, &public_key_bytes, &signature_bytes)
            }
            _ => artifact_auth_core::CryptographicObservation {
                algorithm: artifact_auth_core::ALGORITHM_ED25519.to_string(),
                key_identity: artifact_auth_core::ArtifactRef {
                    profile: artifact_auth_core::ED25519_PUBLIC_KEY_PROFILE_V1.to_string(),
                    algorithm: artifact_auth_core::ALGORITHM_BLAKE3.to_string(),
                    digest_hex: blake3::hash(
                        &decode_hex(&approval.public_key_hex, "reviewer public key").unwrap_or_default(),
                    )
                    .to_hex()
                    .to_string(),
                },
                verified: false,
                failure_code: Some("review_statement_decode_failed".to_string()),
            },
        };
        observations.push(observation);
    }
    debug_assert_eq!(observations.len(), attachment.approvals.len());
    observations
}

/// Read a bundled source-review attachment, remeasure its digest, and validate it.
pub fn remeasure_bundled_source_review_attachment(
    bundle_dir: &Path,
    record: &crunch_release_core::BundledArtifact,
) -> Result<SourceReviewAttachment, RunError> {
    let member_path = bundle_dir.join(&record.relative_path);
    let bytes = read_bounded_file(
        &member_path,
        MAX_SOURCE_REVIEW_ATTACHMENT_BYTES,
        "Bundled source review attachment",
        ReleaseRootKind::ReleaseEvidence,
    )?;
    let measured = blake3::hash(&bytes).to_hex().to_string();
    if measured != record.digest_blake3 {
        return Err(RunError::Internal(format!(
            "bundled source review attachment digest mismatch: manifest {}, measured {measured}",
            record.digest_blake3
        )));
    }
    let attachment = crunch_release_core::parse_source_review_attachment(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing bundled source review attachment: {error}")))?;
    crunch_release_core::validate_source_review_attachment_structure(&attachment)
        .map_err(|error| RunError::Internal(format!("invalid bundled source review attachment: {error}")))?;
    debug_assert!(!attachment.attachment_blake3.is_empty());
    Ok(attachment)
}

/// Resolve, remeasure, and evaluate source-review evidence for one release verification.
pub fn evaluate_source_review_release(
    bundle_dir: &Path,
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    inputs: &SourceReviewVerifyInputs,
    policy: &ReviewedSourcePolicy,
) -> Result<SourceReviewVerification, RunError> {
    let mode = SourceReviewMode::parse(&inputs.mode)
        .map_err(|error| RunError::Internal(format!("invalid --reviewed-source mode: {error}")))?;
    let attachment = match &inputs.attachment_override {
        Some(path) => Some(read_source_review_attachment_unvalidated(path)?),
        None => manifest
            .source_review_attachment
            .as_ref()
            .map(|record| remeasure_bundled_source_review_attachment(bundle_dir, &record.attachment))
            .transpose()?,
    };
    if !mode.is_required() && attachment.is_none() {
        let input = SourceReviewVerificationInput {
            mode,
            policy,
            expected_source_archive_digest_blake3: manifest.source_archive.digest_blake3.clone(),
            expected_claim_root_blake3: String::new(),
            expected_source_revision: None,
            attachment: None,
            cryptographic_observations: Vec::new(),
        };
        return Ok(crunch_release_core::evaluate_source_review_release_evidence(&input));
    }
    if policy.reviewers.is_empty() {
        return Err(RunError::Internal(
            "the release carries source-review evidence; pass --trusted-reviewer keys (and --review-claim-root) to verify it, or select --reviewed-source required with a reviewed-source policy"
                .to_string(),
        ));
    }
    let attachment_value = attachment.as_ref().expect("attachment was checked above");
    let claim_root = inputs.claim_root_blake3.as_deref().ok_or_else(|| {
        RunError::Internal(
            "evaluating source review evidence requires --review-claim-root with the current claim root BLAKE3"
                .to_string(),
        )
    })?;
    let expected_revision = manifest.source_acquisition.as_ref().and_then(|acquisition| acquisition.commit.clone());
    let observations = verify_attachment_signatures(attachment_value);
    let input = SourceReviewVerificationInput {
        mode,
        policy,
        expected_source_archive_digest_blake3: manifest.source_archive.digest_blake3.clone(),
        expected_claim_root_blake3: claim_root.to_string(),
        expected_source_revision: expected_revision,
        attachment: Some(attachment_value),
        cryptographic_observations: observations,
    };
    Ok(crunch_release_core::evaluate_source_review_release_evidence(&input))
}

/// Bundle-relative destination for a source-review attachment inside a release bundle.
#[must_use]
pub fn source_review_bundle_relative_path() -> PathBuf {
    PathBuf::from(RELEASED_SOURCE_REVIEW_ATTACHMENT_RELATIVE_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reviewer_arg(label: &str, key: &str) -> ReviewerKeyArg {
        ReviewerKeyArg {
            label: label.to_string(),
            public_key_base64: key.to_string(),
        }
    }

    #[test]
    fn verification_inputs_default_to_optional_review() {
        let inputs = SourceReviewVerifyInputs::default();
        assert_eq!(inputs.mode, SOURCE_REVIEW_MODE_OPTIONAL);
        assert!(inputs.preset.is_none());
    }

    #[test]
    fn reviewer_arg_requires_label_and_key() {
        assert!(parse_reviewer_arg("nolabelkey").is_err());
        assert!(parse_reviewer_arg(":key").is_err());
        assert!(parse_reviewer_arg("label:").is_err());
        let parsed = parse_reviewer_arg("alice:key").expect("valid arg");
        assert_eq!(parsed.label, "alice");
        assert_eq!(parsed.public_key_base64, "key");
    }

    #[test]
    fn hex_round_trip_is_exact() {
        let bytes = vec![0x00, 0x0f, 0xff, 0xa5];
        let encoded = encode_hex(&bytes);
        assert_eq!(encoded, "000fffa5");
        let decoded = decode_hex(&encoded, "test").expect("valid hex");
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn hex_decode_rejects_non_hex() {
        assert!(decode_hex("zz", "test").is_err());
        assert!(decode_hex("abc", "test").is_err());
    }

    #[test]
    fn preset_policy_uses_two_reviewer_threshold() {
        let first_key = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let second_key = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=";
        let inputs = SourceReviewVerifyInputs {
            mode: "required".to_string(),
            preset: Some(REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER.to_string()),
            threshold: None,
            trusted_reviewers: vec![reviewer_arg("a", first_key), reviewer_arg("b", second_key)],
            ..SourceReviewVerifyInputs::default()
        };
        let policy = build_reviewed_source_policy(&inputs).expect("valid policy");
        assert_eq!(policy.required_distinct_reviewers, STAGEX_TWO_REVIEWER_THRESHOLD);
        assert!(policy.preset.is_some());
    }

    #[test]
    fn preset_policy_rejects_duplicate_reviewer_key() {
        let key = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let inputs = SourceReviewVerifyInputs {
            mode: "required".to_string(),
            preset: Some(REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER.to_string()),
            trusted_reviewers: vec![reviewer_arg("a", key), reviewer_arg("b", key)],
            ..SourceReviewVerifyInputs::default()
        };
        assert!(build_reviewed_source_policy(&inputs).is_err());
        assert_eq!(inputs.trusted_reviewers.len(), usize::from(STAGEX_TWO_REVIEWER_THRESHOLD));
    }

    #[test]
    fn unknown_preset_is_rejected() {
        let inputs = SourceReviewVerifyInputs {
            mode: "required".to_string(),
            preset: Some("one-reviewer".to_string()),
            ..SourceReviewVerifyInputs::default()
        };
        assert!(build_reviewed_source_policy(&inputs).is_err());
    }

    #[test]
    fn required_mode_without_preset_or_threshold_is_rejected() {
        let inputs = SourceReviewVerifyInputs {
            mode: "required".to_string(),
            ..SourceReviewVerifyInputs::default()
        };
        assert!(build_reviewed_source_policy(&inputs).is_err());
    }

    #[test]
    fn wrong_length_reviewer_key_is_rejected() {
        let short_key = "AAAA";
        let inputs = SourceReviewVerifyInputs {
            mode: "required".to_string(),
            preset: Some(REVIEWED_SOURCE_PRESET_STAGEX_TWO_REVIEWER.to_string()),
            trusted_reviewers: vec![reviewer_arg("a", short_key)],
            ..SourceReviewVerifyInputs::default()
        };
        assert!(build_reviewed_source_policy(&inputs).is_err());
    }
}
