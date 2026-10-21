//! Independent release association for Cairn reviewer-signed source evidence.
//! External schemas are owned by Cairn/Valence; review never counts as a release
//! signature, build witness, verified property, or source-correctness proof.
// r[impl verification_evidence.release_source_review_evidence]
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use artifact_auth_core::ArtifactRef;
use artifact_auth_core::ArtifactStatement;
use artifact_auth_core::AuthenticationPolicy;
use artifact_auth_core::AuthenticationScope;
use artifact_auth_core::SignatureEvidence;
use crunch_source_core::Blake3Digest;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::ReleaseEvidenceManifest;

pub const SOURCE_REVIEW_ATTACHMENT_ROLE: &str = "cairn-source-review-attachment";
pub const SOURCE_REVIEW_POLICY_ROLE: &str = "cairn-source-review-policy";
pub const SOURCE_REVIEW_VALENCE_ROLE: &str = "valence-source-review-acceptance";
pub const SOURCE_REVIEW_CLAIM_SCOPE: &str = "source-review-identity-linkage-only";
pub const SOURCE_REVIEW_ATTACHMENT_SCHEMA: &str = "cairn.source-review-attachment.v1";
pub const SOURCE_REVIEW_POLICY_SCHEMA: &str = "cairn.source-review-verification-policy.v1";
pub const SOURCE_REVIEW_VALENCE_SCHEMA: &str = "valence.source-review-acceptance.v1";
pub const SOURCE_REVIEW_BYTES_MAX: usize = 1024 * 1024;
pub const SOURCE_REVIEW_ROWS_MAX: usize = 64;
pub const SOURCE_REVIEW_NON_CLAIMS: &[&str] = &[
    "source-review-does-not-prove-build-correctness",
    "source-review-does-not-prove-deployment-safety",
    "source-review-does-not-prove-release-eligibility",
    "source-review-does-not-prove-reproducibility",
    "source-review-does-not-prove-review-quality",
    "source-review-does-not-prove-source-correctness",
    "source-review-does-not-prove-witness-independence",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewSubject {
    pub schema: String,
    pub source_archive_blake3: Blake3Digest,
    pub source_revision: Option<String>,
    pub source_tree_blake3: Option<Blake3Digest>,
    pub snapshot_profile: String,
    pub snapshot_profile_version: u32,
    pub projection: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewClaimRoot {
    pub claim_identity: Blake3Digest,
    pub subject_root: Blake3Digest,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewClaim {
    pub schema: String,
    pub subject: SourceReviewSubject,
    pub claim_root: SourceReviewClaimRoot,
    pub review_policy_blake3: Blake3Digest,
    pub disposition: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedSourceReview {
    pub claim: SourceReviewClaim,
    pub statement: ArtifactStatement,
    pub generation: u64,
    pub public_key_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewAttachment {
    pub schema: String,
    pub reviews: Vec<SignedSourceReview>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewVerificationPolicy {
    pub schema: String,
    pub subject: SourceReviewSubject,
    pub claim_root: SourceReviewClaimRoot,
    pub review_policy_blake3: Blake3Digest,
    pub authentication_policy: AuthenticationPolicy,
    pub verifier_context: ArtifactRef,
    pub author_key_blake3: Vec<Blake3Digest>,
    pub exclude_authors: bool,
    pub required_approvals: u16,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceReviewStatus {
    NotRequired,
    Recorded,
    Satisfied,
    Missing,
    Insufficient,
    Invalid,
    Stale,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceReviewVerification {
    pub status: SourceReviewStatus,
    pub valid: bool,
    pub required: bool,
    pub approved_key_blake3: Vec<String>,
    pub signed_statement_blake3: Vec<String>,
    pub attachment_identity: Option<String>,
    pub policy_identity: Option<String>,
    pub valence_receipt_identity: Option<String>,
    pub diagnostics: Vec<String>,
    pub non_claims: Vec<String>,
}

fn failure(required: bool, status: SourceReviewStatus, reason: &str) -> SourceReviewVerification {
    SourceReviewVerification {
        status,
        valid: false,
        required,
        approved_key_blake3: Vec::new(),
        signed_statement_blake3: Vec::new(),
        attachment_identity: None,
        policy_identity: None,
        valence_receipt_identity: None,
        diagnostics: vec![reason.to_string()],
        non_claims: SOURCE_REVIEW_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    }
}

/// Independently verifies real detached statements and exact Valence linkage.
/// The trusted policy is an explicit operator input, never inferred from a
/// bundled producer policy. Absence is optional only when no policy is selected.
pub fn verify_release_source_review(
    manifest: &ReleaseEvidenceManifest,
    attachment_bytes: Option<&[u8]>,
    policy_bytes: Option<&[u8]>,
    valence_bytes: Option<&[u8]>,
    trusted_policy_bytes: Option<&[u8]>,
    require_review: bool,
) -> SourceReviewVerification {
    let required = require_review || trusted_policy_bytes.is_some();
    if attachment_bytes.is_none() && policy_bytes.is_none() && valence_bytes.is_none() && !required {
        let mut result = failure(false, SourceReviewStatus::NotRequired, "");
        result.valid = true;
        result.diagnostics.clear();
        return result;
    }
    if required && trusted_policy_bytes.is_none() {
        return failure(required, SourceReviewStatus::Missing, "source-review-trusted-policy-missing");
    }
    let (Some(attachment), Some(policy), Some(valence)) = (attachment_bytes, policy_bytes, valence_bytes) else {
        return failure(required, SourceReviewStatus::Missing, "source-review-artifact-missing");
    };
    if [attachment.len(), policy.len(), valence.len()].into_iter().any(|len| len > SOURCE_REVIEW_BYTES_MAX) {
        return failure(required, SourceReviewStatus::Invalid, "source-review-artifact-oversized");
    }
    if trusted_policy_bytes.is_some_and(|trusted| trusted != policy) {
        return failure(required, SourceReviewStatus::Stale, "source-review-selected-policy-mismatch");
    }
    match verify_present(manifest, attachment, policy, valence, required) {
        Ok(result) => result,
        Err(reason) => {
            let status = if reason == "source-review-insufficient-approvals" {
                SourceReviewStatus::Insufficient
            } else if reason.ends_with("unsupported") {
                SourceReviewStatus::Unsupported
            } else if reason.starts_with("source-review-stale-")
                || reason.starts_with("source-review-release-")
                || reason == "source-review-observation-subject-mismatch"
            {
                SourceReviewStatus::Stale
            } else {
                SourceReviewStatus::Invalid
            };
            failure(required, status, &reason)
        }
    }
}

fn verify_present(
    manifest: &ReleaseEvidenceManifest,
    attachment_bytes: &[u8],
    policy_bytes: &[u8],
    valence_bytes: &[u8],
    required: bool,
) -> Result<SourceReviewVerification, String> {
    let mut attachment: SourceReviewAttachment =
        serde_json::from_slice(attachment_bytes).map_err(|_| "source-review-attachment-malformed")?;
    let mut policy: SourceReviewVerificationPolicy =
        serde_json::from_slice(policy_bytes).map_err(|_| "source-review-policy-malformed")?;
    validate_profile(manifest, &attachment, &policy)?;
    attachment
        .reviews
        .sort_by(|left, right| left.statement.key_identity.digest_hex.cmp(&right.statement.key_identity.digest_hex));
    policy.author_key_blake3.sort();
    policy.authentication_policy.trusted_keys.sort_by(|left, right| {
        left.key_identity
            .cmp(&right.key_identity)
            .then(left.producer_id.cmp(&right.producer_id))
            .then(left.key_id.cmp(&right.key_id))
    });
    for key in &mut policy.authentication_policy.trusted_keys {
        key.allowed_purposes.sort();
    }
    let mut statement_ids = Vec::with_capacity(attachment.reviews.len());
    let mut claim_ids = Vec::with_capacity(attachment.reviews.len());
    let mut seen_keys = BTreeSet::new();
    let mut approved_keys = BTreeSet::new();
    let mut row_policy = policy.authentication_policy.clone();
    row_policy.threshold = 1;
    for review in &attachment.reviews {
        validate_review(review, &policy)?;
        let scope = expected_review_scope(&review.claim, &policy.verifier_context)?;
        let key = review.statement.key_identity.digest_hex.clone();
        if !seen_keys.insert(key.clone()) {
            return Err("source-review-duplicate-full-key".to_string());
        }
        if policy.exclude_authors && policy.author_key_blake3.iter().any(|author| author.as_str() == key) {
            return Err("source-review-author-excluded".to_string());
        }
        let public_key = decode_hex::<32>(&review.public_key_hex)?;
        let signature = decode_hex::<64>(&review.signature_hex)?;
        let cryptographic = artifact_auth_ed25519::verify_statement(&review.statement, &public_key, &signature);
        let authentication = artifact_auth_core::evaluate_authentication(&row_policy, &scope, &[SignatureEvidence {
            statement: review.statement.clone(),
            generation: review.generation,
            cryptographic,
        }]);
        if !authentication.passed {
            let codes = authentication.issues.iter().map(|issue| issue.code.as_str()).collect::<Vec<_>>().join(",");
            return Err(format!("source-review-authentication-rejected:{codes}"));
        }
        if review.claim.disposition == "approved" {
            approved_keys.insert(key);
        }
        statement_ids.push(
            artifact_auth_core::statement_identity(&review.statement).map_err(|_| "source-review-statement-invalid")?,
        );
        claim_ids.push(source_review_identity(b"cairn.source-review-claim.v1\0", &review.claim)?);
    }
    if approved_keys.len() < usize::from(policy.required_approvals) {
        return Err("source-review-insufficient-approvals".to_string());
    }
    let authentication_policy_id = artifact_auth_core::policy_identity(&policy.authentication_policy)
        .map_err(|_| "source-review-authentication-policy-invalid")?;
    let approved_keys = approved_keys.into_iter().collect::<Vec<_>>();
    statement_ids.sort();
    claim_ids.sort();
    claim_ids.dedup();
    let attachment_id = source_review_identity(b"cairn.source-review-attachment.v1\0", &attachment)?;
    let policy_id = source_review_identity(b"cairn.source-review-verification-policy.v1\0", &policy)?;
    let receipt_id = validate_valence_receipt(
        valence_bytes,
        attachment_bytes,
        policy_bytes,
        &policy,
        &attachment_id,
        &policy_id,
        &authentication_policy_id,
        &approved_keys,
        &statement_ids,
        &claim_ids,
    )?;
    Ok(SourceReviewVerification {
        status: if required {
            SourceReviewStatus::Satisfied
        } else {
            SourceReviewStatus::Recorded
        },
        valid: true,
        required,
        approved_key_blake3: approved_keys,
        signed_statement_blake3: statement_ids,
        attachment_identity: Some(attachment_id),
        policy_identity: Some(policy_id),
        valence_receipt_identity: Some(receipt_id),
        diagnostics: Vec::new(),
        non_claims: attachment.non_claims,
    })
}

fn validate_profile(
    manifest: &ReleaseEvidenceManifest,
    attachment: &SourceReviewAttachment,
    policy: &SourceReviewVerificationPolicy,
) -> Result<(), String> {
    crate::manifest::validate_source_acquisition(manifest)
        .map_err(|_| "source-review-release-acquisition-or-observation-invalid")?;
    if attachment.schema != SOURCE_REVIEW_ATTACHMENT_SCHEMA || policy.schema != SOURCE_REVIEW_POLICY_SCHEMA {
        return Err("source-review-schema-unsupported".to_string());
    }
    if attachment.reviews.len() > SOURCE_REVIEW_ROWS_MAX
        || policy.author_key_blake3.len() > SOURCE_REVIEW_ROWS_MAX
        || policy.authentication_policy.trusted_keys.is_empty()
        || policy.authentication_policy.trusted_keys.len() > SOURCE_REVIEW_ROWS_MAX
        || policy.required_approvals == 0
        || usize::from(policy.required_approvals) > SOURCE_REVIEW_ROWS_MAX
        || policy.authentication_policy.profile_id != "cairn.source-review.v1"
        || policy.authentication_policy.threshold != policy.required_approvals
    {
        return Err("source-review-policy-bounds-invalid".to_string());
    }
    validate_non_claims(&attachment.non_claims)?;
    validate_non_claims(&policy.non_claims)?;
    let subject = &policy.subject;
    if subject.schema != "cairn.source-review-subject.v1"
        || subject.snapshot_profile != "mantle-release-source"
        || subject.snapshot_profile_version != 1
        || subject.projection != "source"
    {
        return Err("source-review-subject-profile-unsupported".to_string());
    }
    if subject
        .source_revision
        .as_ref()
        .is_some_and(|revision| revision.is_empty() || revision.len() > 256 || revision.chars().any(char::is_control))
    {
        return Err("source-review-source-revision-invalid".to_string());
    }
    if subject.source_archive_blake3.as_str() != manifest.source_archive.digest_blake3 {
        return Err("source-review-release-source-mismatch".to_string());
    }
    if let Some(acquisition) = &manifest.source_acquisition {
        if acquisition.commit != subject.source_revision {
            return Err("source-review-release-revision-mismatch".to_string());
        }
        if let Some(observation) = &acquisition.source_observation {
            let mismatch = observation.payload_blake3 != subject.source_archive_blake3.as_str()
                || observation.snapshot_profile_name != subject.snapshot_profile
                || observation.snapshot_profile_version != subject.snapshot_profile_version;
            if mismatch {
                return Err("source-review-observation-subject-mismatch".to_string());
            }
        }
    } else if subject.source_revision.is_some() {
        return Err("source-review-release-revision-unavailable".to_string());
    }
    let mut keys = BTreeSet::new();
    for key in &policy.authentication_policy.trusted_keys {
        if !keys.insert(key.key_identity.digest_hex.as_str())
            || key.allowed_purposes != ["source-review-approval"]
            || key.currentness_ref != policy.verifier_context
        {
            return Err("source-review-reviewer-key-role-invalid".to_string());
        }
    }
    let mut authors = BTreeSet::new();
    if policy.author_key_blake3.iter().any(|author| !authors.insert(author.as_str())) {
        return Err("source-review-duplicate-author-key".to_string());
    }
    Ok(())
}

fn validate_review(review: &SignedSourceReview, policy: &SourceReviewVerificationPolicy) -> Result<(), String> {
    let claim = &review.claim;
    if claim.schema != "cairn.source-review-claim.v1"
        || !matches!(claim.disposition.as_str(), "approved" | "needs_revision")
    {
        return Err("source-review-disposition-unsupported".to_string());
    }
    validate_non_claims(&claim.non_claims)?;
    if claim.subject != policy.subject
        || claim.claim_root != policy.claim_root
        || claim.review_policy_blake3 != policy.review_policy_blake3
    {
        return Err("source-review-stale-subject-root-or-policy".to_string());
    }
    let expected = expected_review_scope(claim, &policy.verifier_context)?;
    if review.statement.scope != expected {
        return Err("source-review-signed-scope-mismatch".to_string());
    }
    Ok(())
}

fn expected_review_scope(claim: &SourceReviewClaim, context: &ArtifactRef) -> Result<AuthenticationScope, String> {
    let mut parents = vec![
        reference("mantle-release-source.v1", claim.subject.source_archive_blake3.as_str()),
        reference("cairn.source-review-policy.v1", claim.review_policy_blake3.as_str()),
    ];
    parents.sort();
    Ok(AuthenticationScope {
        domain: "cairn.source-review.v1".to_string(),
        purpose: "source-review-approval".to_string(),
        profile_id: "cairn.source-review.v1".to_string(),
        subject: reference(
            "cairn.source-review-claim.v1",
            &source_review_identity(b"cairn.source-review-claim.v1\0", claim)?,
        ),
        parents,
        verifier_context: context.clone(),
    })
}

fn reference(profile: &str, digest: &str) -> ArtifactRef {
    ArtifactRef {
        profile: profile.to_string(),
        algorithm: "blake3".to_string(),
        digest_hex: digest.to_string(),
    }
}

fn validate_non_claims(values: &[String]) -> Result<(), String> {
    if values.iter().map(String::as_str).ne(SOURCE_REVIEW_NON_CLAIMS.iter().copied()) {
        return Err("source-review-non-claims-invalid".to_string());
    }
    Ok(())
}

fn decode_hex<const N: usize>(text: &str) -> Result<[u8; N], String> {
    if text.len() != N * 2 || !text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err("source-review-key-or-signature-malformed".to_string());
    }
    let mut output = [0; N];
    for (index, pair) in text.as_bytes().chunks_exact(2).enumerate() {
        let digit = |byte: u8| if byte <= b'9' { byte - b'0' } else { byte - b'a' + 10 };
        output[index] = (digit(pair[0]) << 4) | digit(pair[1]);
    }
    Ok(output)
}

/// Canonical external payload identity, distinct from exact artifact-byte hashes.
pub fn source_review_identity<T: Serialize>(domain: &[u8], value: &T) -> Result<String, String> {
    let value = serde_json::to_value(value).map_err(|_| "source-review-canonicalization-failed")?;
    let bytes = serde_json::to_vec(&CanonicalJson(&value)).map_err(|_| "source-review-canonicalization-failed")?;
    let mut hash = blake3::Hasher::new();
    hash.update(domain);
    hash.update(&bytes);
    Ok(hash.finalize().to_hex().to_string())
}

struct CanonicalJson<'a>(&'a Value);

impl Serialize for CanonicalJson<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        use serde::ser::SerializeSeq;
        match self.0 {
            Value::Object(fields) => {
                let mut fields = fields.iter().collect::<Vec<_>>();
                fields.sort_unstable_by(|left, right| left.0.cmp(right.0));
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (key, value) in fields {
                    map.serialize_entry(key, &CanonicalJson(value))?;
                }
                map.end()
            }
            Value::Array(values) => {
                let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    sequence.serialize_element(&CanonicalJson(value))?;
                }
                sequence.end()
            }
            value => value.serialize(serializer),
        }
    }
}

// Closed independent mirror of Valence's public receipt. Direct typed decoding
// rejects duplicate fields before canonical identity calculation loses them.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValenceSourceReviewAcceptance {
    schema: String,
    outcome: String,
    verification_role: String,
    subject: Option<SourceReviewSubject>,
    subject_identity: Option<String>,
    claim_root: Option<SourceReviewClaimRoot>,
    claim_root_identity: Option<String>,
    review_policy_blake3: Option<String>,
    verification_policy_identity: Option<String>,
    authentication_policy_identity: Option<String>,
    approved_key_blake3: Vec<String>,
    signed_statement_blake3: Vec<String>,
    claim_identity: Vec<String>,
    attachment_identity: Option<String>,
    attachment_artifact_blake3: String,
    policy_artifact_blake3: String,
    receipt_blake3: String,
    non_claims: Vec<String>,
    issues: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
fn validate_valence_receipt(
    bytes: &[u8],
    attachment_bytes: &[u8],
    policy_bytes: &[u8],
    policy: &SourceReviewVerificationPolicy,
    attachment_id: &str,
    policy_id: &str,
    authentication_policy_id: &str,
    keys: &[String],
    statements: &[String],
    claims: &[String],
) -> Result<String, String> {
    let parsed: ValenceSourceReviewAcceptance =
        serde_json::from_slice(bytes).map_err(|_| "source-review-valence-malformed")?;
    let mut receipt = serde_json::to_value(parsed).map_err(|_| "source-review-valence-canonicalization-failed")?;
    let object = receipt.as_object_mut().ok_or("source-review-valence-malformed")?;
    let receipt_id = object
        .remove("receipt_blake3")
        .and_then(|value| value.as_str().map(ToString::to_string))
        .ok_or("source-review-valence-receipt-identity-missing")?;
    if source_review_identity(b"valence.source-review-acceptance.v1\0", &receipt)? != receipt_id {
        return Err("source-review-valence-receipt-identity-mismatch".to_string());
    }
    let expected = [
        ("schema", SOURCE_REVIEW_VALENCE_SCHEMA.to_string()),
        ("outcome", "accepted_manual_review".to_string()),
        ("verification_role", "manual_review".to_string()),
        ("attachment_identity", attachment_id.to_string()),
        ("verification_policy_identity", policy_id.to_string()),
        ("authentication_policy_identity", authentication_policy_id.to_string()),
        ("review_policy_blake3", policy.review_policy_blake3.as_str().to_string()),
        ("attachment_artifact_blake3", blake3::hash(attachment_bytes).to_hex().to_string()),
        ("policy_artifact_blake3", blake3::hash(policy_bytes).to_hex().to_string()),
        ("subject_identity", source_review_identity(b"cairn.source-review-subject.v1\0", &policy.subject)?),
        ("claim_root_identity", source_review_identity(b"cairn.claim-root.v1\0", &policy.claim_root)?),
    ];
    for (field, value) in expected {
        if receipt.get(field).and_then(Value::as_str) != Some(value.as_str()) {
            return Err(format!("source-review-valence-linkage-mismatch:{field}"));
        }
    }
    for (field, value) in [
        ("subject", serde_json::to_value(&policy.subject)),
        ("claim_root", serde_json::to_value(&policy.claim_root)),
        ("approved_key_blake3", serde_json::to_value(keys)),
        ("signed_statement_blake3", serde_json::to_value(statements)),
        ("claim_identity", serde_json::to_value(claims)),
        ("non_claims", serde_json::to_value(&policy.non_claims)),
        ("issues", Ok(Value::Array(Vec::new()))),
    ] {
        if receipt.get(field) != Some(&value.map_err(|_| "source-review-valence-canonicalization-failed")?) {
            return Err(format!("source-review-valence-linkage-mismatch:{field}"));
        }
    }
    Ok(receipt_id)
}

#[cfg(test)]
mod tests;
