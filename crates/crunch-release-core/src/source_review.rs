//! Independent release association for Cairn reviewer-signed source evidence.
//! External schemas are owned by Cairn/Valence; review never counts as a release
//! signature, build witness, verified property, or source-correctness proof.
// r[impl verification_evidence.release_source_review_evidence]
use alloc::collections::BTreeSet;
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

use crate::ReleaseEvidenceEvaluation;
use crate::ReleaseEvidenceManifest;

mod error;
pub use error::SourceReviewError;

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

fn failure(is_required: bool, status: SourceReviewStatus, reason: &str) -> SourceReviewVerification {
    SourceReviewVerification {
        status,
        valid: false,
        required: is_required,
        approved_key_blake3: Vec::new(),
        signed_statement_blake3: Vec::new(),
        attachment_identity: None,
        policy_identity: None,
        valence_receipt_identity: None,
        diagnostics: vec![reason.to_string()],
        non_claims: SOURCE_REVIEW_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    }
}

/// Inputs for one independent release source-review verification.
///
/// Each field carries exactly one domain fact so a caller cannot swap
/// artifacts by position, and the trusted policy stays an explicit operator
/// input next to the policy bytes it is compared against. Like the other
/// release evaluators, the request owns its inputs: payloads move in without
/// copying, and the manifest returns with the verification.
#[derive(Debug, Clone)]
pub struct SourceReviewVerificationRequest {
    pub manifest: ReleaseEvidenceManifest,
    pub attachment_bytes: Option<Vec<u8>>,
    pub policy_bytes: Option<Vec<u8>>,
    pub valence_bytes: Option<Vec<u8>>,
    pub trusted_policy_bytes: Option<Vec<u8>>,
    pub require_review: bool,
}

/// Independently verifies real detached statements and exact Valence linkage.
/// The trusted policy is an explicit operator input, never inferred from a
/// bundled producer policy. Absence is optional only when no policy is selected.
pub fn verify_release_source_review(
    request: SourceReviewVerificationRequest,
) -> ReleaseEvidenceEvaluation<SourceReviewVerification> {
    let verification = verify_release_source_review_borrowed(&request);
    ReleaseEvidenceEvaluation {
        manifest: request.manifest,
        verification,
    }
}

fn verify_release_source_review_borrowed(request: &SourceReviewVerificationRequest) -> SourceReviewVerification {
    let is_required = request.require_review || request.trusted_policy_bytes.is_some();
    let has_artifact =
        request.attachment_bytes.is_some() || request.policy_bytes.is_some() || request.valence_bytes.is_some();
    if !has_artifact && !is_required {
        let mut result = failure(false, SourceReviewStatus::NotRequired, "");
        result.valid = true;
        result.diagnostics.clear();
        return result;
    }
    if is_required && request.trusted_policy_bytes.is_none() {
        return failure(is_required, SourceReviewStatus::Missing, "source-review-trusted-policy-missing");
    }
    debug_assert!(!is_required || request.trusted_policy_bytes.is_some());
    let (Some(attachment), Some(policy), Some(valence)) = (
        request.attachment_bytes.as_deref(),
        request.policy_bytes.as_deref(),
        request.valence_bytes.as_deref(),
    ) else {
        return failure(is_required, SourceReviewStatus::Missing, "source-review-artifact-missing");
    };
    if [attachment.len(), policy.len(), valence.len()].into_iter().any(|len| len > SOURCE_REVIEW_BYTES_MAX) {
        return failure(is_required, SourceReviewStatus::Invalid, "source-review-artifact-oversized");
    }
    debug_assert!(attachment.len() <= SOURCE_REVIEW_BYTES_MAX);
    if request.trusted_policy_bytes.as_deref().is_some_and(|trusted| trusted != policy) {
        return failure(is_required, SourceReviewStatus::Stale, "source-review-selected-policy-mismatch");
    }
    match verify_present(&request.manifest, attachment, policy, valence, is_required) {
        Ok(result) => result,
        Err(reason) => failure(is_required, reason.status(), &reason.to_string()),
    }
}

/// Canonical identities and reviewer key sets established by verified review rows.
struct VerifiedReviewEvidence {
    approved_keys: Vec<String>,
    statement_ids: Vec<String>,
    claim_ids: Vec<String>,
}

/// One review row after its statement, scope, and claim identity were verified.
struct VerifiedReviewRow {
    key_blake3: String,
    is_approved: bool,
    statement_identity: String,
    claim_identity: String,
}

fn verify_present(
    manifest: &ReleaseEvidenceManifest,
    attachment_bytes: &[u8],
    policy_bytes: &[u8],
    valence_bytes: &[u8],
    is_required: bool,
) -> Result<SourceReviewVerification, SourceReviewError> {
    let mut attachment: SourceReviewAttachment =
        serde_json::from_slice(attachment_bytes).map_err(|_| SourceReviewError::AttachmentMalformed)?;
    let mut policy: SourceReviewVerificationPolicy =
        serde_json::from_slice(policy_bytes).map_err(|_| SourceReviewError::PolicyMalformed)?;
    validate_profile(manifest, &attachment, &policy)?;
    canonicalize_review_order(&mut attachment, &mut policy);
    let evidence = verify_review_rows(&attachment.reviews, &policy)?;
    if evidence.approved_keys.len() < usize::from(policy.required_approvals) {
        return Err(SourceReviewError::InsufficientApprovals);
    }
    debug_assert!(evidence.approved_keys.len() >= usize::from(policy.required_approvals));
    debug_assert_eq!(evidence.statement_ids.len(), attachment.reviews.len());
    let authentication_policy_id = artifact_auth_core::policy_identity(&policy.authentication_policy)
        .map_err(|_| SourceReviewError::AuthenticationPolicyInvalid)?;
    let VerifiedReviewEvidence {
        approved_keys,
        mut statement_ids,
        mut claim_ids,
    } = evidence;
    statement_ids.sort();
    claim_ids.sort();
    claim_ids.dedup();
    let attachment_id = source_review_identity(b"cairn.source-review-attachment.v1\0", &attachment)?;
    let policy_id = source_review_identity(b"cairn.source-review-verification-policy.v1\0", &policy)?;
    let receipt_id = validate_valence_receipt(&ValenceReceiptExpectation {
        receipt_bytes: valence_bytes,
        attachment_bytes,
        policy_bytes,
        policy: &policy,
        attachment_id: &attachment_id,
        policy_id: &policy_id,
        authentication_policy_id: &authentication_policy_id,
        approved_keys: &approved_keys,
        statement_ids: &statement_ids,
        claim_ids: &claim_ids,
    })?;
    Ok(SourceReviewVerification {
        status: if is_required {
            SourceReviewStatus::Satisfied
        } else {
            SourceReviewStatus::Recorded
        },
        valid: true,
        required: is_required,
        approved_key_blake3: approved_keys,
        signed_statement_blake3: statement_ids,
        attachment_identity: Some(attachment_id),
        policy_identity: Some(policy_id),
        valence_receipt_identity: Some(receipt_id),
        diagnostics: Vec::new(),
        non_claims: attachment.non_claims,
    })
}

/// Orders every review collection so identities do not depend on input order.
fn canonicalize_review_order(attachment: &mut SourceReviewAttachment, policy: &mut SourceReviewVerificationPolicy) {
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
}

/// Verifies every review row and collects the identities it establishes.
fn verify_review_rows(
    reviews: &[SignedSourceReview],
    policy: &SourceReviewVerificationPolicy,
) -> Result<VerifiedReviewEvidence, SourceReviewError> {
    let mut seen_keys = BTreeSet::new();
    let mut approved_keys = BTreeSet::new();
    let mut statement_ids = Vec::with_capacity(reviews.len());
    let mut claim_ids = Vec::with_capacity(reviews.len());
    let mut row_policy = policy.authentication_policy.clone();
    row_policy.threshold = 1;
    for review in reviews {
        let scope = verify_review_claim(review, policy)?;
        let key = review.statement.key_identity.digest_hex.clone();
        if !seen_keys.insert(key.clone()) {
            return Err(SourceReviewError::DuplicateFullKey);
        }
        if policy.exclude_authors && policy.author_key_blake3.iter().any(|author| author.as_str() == key) {
            return Err(SourceReviewError::AuthorExcluded);
        }
        let row = verify_review_signature(review, &row_policy, &scope)?;
        if row.is_approved {
            approved_keys.insert(row.key_blake3);
        }
        statement_ids.push(row.statement_identity);
        claim_ids.push(row.claim_identity);
    }
    debug_assert_eq!(statement_ids.len(), reviews.len());
    debug_assert_eq!(claim_ids.len(), reviews.len());
    Ok(VerifiedReviewEvidence {
        approved_keys: approved_keys.into_iter().collect(),
        statement_ids,
        claim_ids,
    })
}

/// Checks the review claim and returns the scope its statement must carry.
fn verify_review_claim(
    review: &SignedSourceReview,
    policy: &SourceReviewVerificationPolicy,
) -> Result<AuthenticationScope, SourceReviewError> {
    validate_review(review, policy)?;
    expected_review_scope(&review.claim, &policy.verifier_context)
}

/// Authenticates one review statement and derives its canonical identities.
fn verify_review_signature(
    review: &SignedSourceReview,
    row_policy: &AuthenticationPolicy,
    scope: &AuthenticationScope,
) -> Result<VerifiedReviewRow, SourceReviewError> {
    let public_key = decode_hex::<32>(&review.public_key_hex)?;
    let signature = decode_hex::<64>(&review.signature_hex)?;
    debug_assert_eq!(row_policy.threshold, 1);
    debug_assert!(matches!(review.claim.disposition.as_str(), "approved" | "needs_revision"));
    let cryptographic = artifact_auth_ed25519::verify_statement(&review.statement, &public_key, &signature);
    let authentication = artifact_auth_core::evaluate_authentication(row_policy, scope, &[SignatureEvidence {
        statement: review.statement.clone(),
        generation: review.generation,
        cryptographic,
    }]);
    if !authentication.passed {
        let codes = authentication.issues.iter().map(|issue| issue.code.as_str()).collect::<Vec<_>>().join(",");
        return Err(SourceReviewError::AuthenticationRejected(codes));
    }
    let statement_identity =
        artifact_auth_core::statement_identity(&review.statement).map_err(|_| SourceReviewError::StatementInvalid)?;
    let claim_identity = source_review_identity(b"cairn.source-review-claim.v1\0", &review.claim)?;
    Ok(VerifiedReviewRow {
        key_blake3: review.statement.key_identity.digest_hex.clone(),
        is_approved: review.claim.disposition == "approved",
        statement_identity,
        claim_identity,
    })
}

fn validate_profile(
    manifest: &ReleaseEvidenceManifest,
    attachment: &SourceReviewAttachment,
    policy: &SourceReviewVerificationPolicy,
) -> Result<(), SourceReviewError> {
    crate::manifest::validate_source_acquisition(manifest)
        .map_err(|_| SourceReviewError::ReleaseAcquisitionOrObservationInvalid)?;
    validate_profile_schemas(attachment, policy)?;
    validate_profile_bounds(attachment, policy)?;
    validate_non_claims(&attachment.non_claims)?;
    validate_non_claims(&policy.non_claims)?;
    validate_profile_subject(policy)?;
    validate_profile_revision(policy)?;
    validate_profile_acquisition(manifest, policy)?;
    validate_profile_reviewer_keys(policy)?;
    Ok(())
}

/// Requires the fixed attachment and policy schema names.
fn validate_profile_schemas(
    attachment: &SourceReviewAttachment,
    policy: &SourceReviewVerificationPolicy,
) -> Result<(), SourceReviewError> {
    if attachment.schema != SOURCE_REVIEW_ATTACHMENT_SCHEMA || policy.schema != SOURCE_REVIEW_POLICY_SCHEMA {
        return Err(SourceReviewError::SchemaUnsupported);
    }
    Ok(())
}

/// Bounds every reviewer collection and pins the authentication profile.
fn validate_profile_bounds(
    attachment: &SourceReviewAttachment,
    policy: &SourceReviewVerificationPolicy,
) -> Result<(), SourceReviewError> {
    if attachment.reviews.len() > SOURCE_REVIEW_ROWS_MAX || policy.author_key_blake3.len() > SOURCE_REVIEW_ROWS_MAX {
        return Err(SourceReviewError::PolicyBoundsInvalid);
    }
    let trusted_keys = &policy.authentication_policy.trusted_keys;
    if trusted_keys.is_empty() || trusted_keys.len() > SOURCE_REVIEW_ROWS_MAX {
        return Err(SourceReviewError::PolicyBoundsInvalid);
    }
    debug_assert!(trusted_keys.len() <= SOURCE_REVIEW_ROWS_MAX);
    if policy.required_approvals == 0 || usize::from(policy.required_approvals) > SOURCE_REVIEW_ROWS_MAX {
        return Err(SourceReviewError::PolicyBoundsInvalid);
    }
    debug_assert!(policy.required_approvals > 0);
    if policy.authentication_policy.profile_id != "cairn.source-review.v1" {
        return Err(SourceReviewError::PolicyBoundsInvalid);
    }
    if policy.authentication_policy.threshold != policy.required_approvals {
        return Err(SourceReviewError::PolicyBoundsInvalid);
    }
    Ok(())
}

/// Requires the reviewed subject to describe the released source projection.
fn validate_profile_subject(policy: &SourceReviewVerificationPolicy) -> Result<(), SourceReviewError> {
    let subject = &policy.subject;
    if subject.schema != "cairn.source-review-subject.v1" {
        return Err(SourceReviewError::SubjectProfileUnsupported);
    }
    if subject.snapshot_profile != "mantle-release-source" {
        return Err(SourceReviewError::SubjectProfileUnsupported);
    }
    if subject.snapshot_profile_version != 1 {
        return Err(SourceReviewError::SubjectProfileUnsupported);
    }
    if subject.projection != "source" {
        return Err(SourceReviewError::SubjectProfileUnsupported);
    }
    Ok(())
}

/// Bounds the reviewed source revision text.
fn validate_profile_revision(policy: &SourceReviewVerificationPolicy) -> Result<(), SourceReviewError> {
    if policy
        .subject
        .source_revision
        .as_ref()
        .is_some_and(|revision| revision.is_empty() || revision.len() > 256 || revision.chars().any(char::is_control))
    {
        return Err(SourceReviewError::SourceRevisionInvalid);
    }
    Ok(())
}

/// Binds the reviewed subject to the released source acquisition facts.
fn validate_profile_acquisition(
    manifest: &ReleaseEvidenceManifest,
    policy: &SourceReviewVerificationPolicy,
) -> Result<(), SourceReviewError> {
    let subject = &policy.subject;
    if subject.source_archive_blake3.as_str() != manifest.source_archive.digest_blake3 {
        return Err(SourceReviewError::ReleaseSourceMismatch);
    }
    debug_assert_eq!(subject.source_archive_blake3.as_str(), manifest.source_archive.digest_blake3);
    let Some(acquisition) = &manifest.source_acquisition else {
        if subject.source_revision.is_some() {
            return Err(SourceReviewError::ReleaseRevisionUnavailable);
        }
        return Ok(());
    };
    if acquisition.commit != subject.source_revision {
        return Err(SourceReviewError::ReleaseRevisionMismatch);
    }
    debug_assert_eq!(acquisition.commit, subject.source_revision);
    let Some(observation) = &acquisition.source_observation else {
        return Ok(());
    };
    let is_mismatch = observation.payload_blake3 != subject.source_archive_blake3.as_str()
        || observation.snapshot_profile_name != subject.snapshot_profile
        || observation.snapshot_profile_version != subject.snapshot_profile_version;
    if is_mismatch {
        return Err(SourceReviewError::ObservationSubjectMismatch);
    }
    Ok(())
}

/// Requires every trusted reviewer key to carry the reviewer approval role.
fn validate_profile_reviewer_keys(policy: &SourceReviewVerificationPolicy) -> Result<(), SourceReviewError> {
    let mut keys = BTreeSet::new();
    for key in &policy.authentication_policy.trusted_keys {
        if !keys.insert(key.key_identity.digest_hex.as_str())
            || key.allowed_purposes != ["source-review-approval"]
            || key.currentness_ref != policy.verifier_context
        {
            return Err(SourceReviewError::ReviewerKeyRoleInvalid);
        }
    }
    let mut authors = BTreeSet::new();
    if policy.author_key_blake3.iter().any(|author| !authors.insert(author.as_str())) {
        return Err(SourceReviewError::DuplicateAuthorKey);
    }
    Ok(())
}

fn validate_review(
    review: &SignedSourceReview,
    policy: &SourceReviewVerificationPolicy,
) -> Result<(), SourceReviewError> {
    let claim = &review.claim;
    if claim.schema != "cairn.source-review-claim.v1"
        || !matches!(claim.disposition.as_str(), "approved" | "needs_revision")
    {
        return Err(SourceReviewError::DispositionUnsupported);
    }
    validate_non_claims(&claim.non_claims)?;
    if claim.subject != policy.subject
        || claim.claim_root != policy.claim_root
        || claim.review_policy_blake3 != policy.review_policy_blake3
    {
        return Err(SourceReviewError::StaleSubjectRootOrPolicy);
    }
    let expected = expected_review_scope(claim, &policy.verifier_context)?;
    if review.statement.scope != expected {
        return Err(SourceReviewError::SignedScopeMismatch);
    }
    debug_assert!(matches!(claim.disposition.as_str(), "approved" | "needs_revision"));
    debug_assert_eq!(claim.subject, policy.subject);
    Ok(())
}

fn expected_review_scope(
    claim: &SourceReviewClaim,
    context: &ArtifactRef,
) -> Result<AuthenticationScope, SourceReviewError> {
    let mut parents = vec![
        reference(ProfileDigest {
            profile: "mantle-release-source.v1",
            digest: claim.subject.source_archive_blake3.as_str(),
        }),
        reference(ProfileDigest {
            profile: "cairn.source-review-policy.v1",
            digest: claim.review_policy_blake3.as_str(),
        }),
    ];
    parents.sort();
    debug_assert_eq!(parents.len(), 2);
    debug_assert!(parents.windows(2).all(|pair| pair[0] <= pair[1]));
    Ok(AuthenticationScope {
        domain: "cairn.source-review.v1".to_string(),
        purpose: "source-review-approval".to_string(),
        profile_id: "cairn.source-review.v1".to_string(),
        subject: reference(ProfileDigest {
            profile: "cairn.source-review-claim.v1",
            digest: &source_review_identity(b"cairn.source-review-claim.v1\0", claim)?,
        }),
        parents,
        verifier_context: context.clone(),
    })
}

/// Binds a profile name to the digest it references so the pair cannot be swapped.
struct ProfileDigest<'a> {
    profile: &'a str,
    digest: &'a str,
}

fn reference(named: ProfileDigest<'_>) -> ArtifactRef {
    ArtifactRef {
        profile: named.profile.to_string(),
        algorithm: "blake3".to_string(),
        digest_hex: named.digest.to_string(),
    }
}

fn validate_non_claims(values: &[String]) -> Result<(), SourceReviewError> {
    if values.iter().map(String::as_str).ne(SOURCE_REVIEW_NON_CLAIMS.iter().copied()) {
        return Err(SourceReviewError::NonClaimsInvalid);
    }
    Ok(())
}

fn decode_hex<const N: usize>(text: &str) -> Result<[u8; N], SourceReviewError> {
    let expected_len = N.checked_mul(2).ok_or(SourceReviewError::KeyOrSignatureMalformed)?;
    if text.len() != expected_len || !text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(SourceReviewError::KeyOrSignatureMalformed);
    }
    let mut output = [0; N];
    for (index, pair) in text.as_bytes().chunks_exact(2).enumerate() {
        let high = hex_digit(pair[0])?;
        let low = hex_digit(pair[1])?;
        output[index] = (high << 4) | low;
    }
    Ok(output)
}

/// Decodes one lowercase hexadecimal digit without unchecked arithmetic.
fn hex_digit(byte: u8) -> Result<u8, SourceReviewError> {
    let value = if byte <= b'9' {
        byte.checked_sub(b'0')
    } else {
        byte.checked_sub(b'a').and_then(|digit| digit.checked_add(10))
    };
    value.ok_or(SourceReviewError::KeyOrSignatureMalformed)
}

/// Canonical external payload identity, distinct from exact artifact-byte hashes.
fn source_review_identity<T: Serialize>(domain: &[u8], value: &T) -> Result<String, SourceReviewError> {
    let value = serde_json::to_value(value).map_err(|_| SourceReviewError::CanonicalizationFailed)?;
    let bytes = serde_json::to_vec(&CanonicalJson(&value)).map_err(|_| SourceReviewError::CanonicalizationFailed)?;
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

/// Facts a Valence receipt must agree with the independently verified evidence.
struct ValenceReceiptExpectation<'a> {
    receipt_bytes: &'a [u8],
    attachment_bytes: &'a [u8],
    policy_bytes: &'a [u8],
    policy: &'a SourceReviewVerificationPolicy,
    attachment_id: &'a str,
    policy_id: &'a str,
    authentication_policy_id: &'a str,
    approved_keys: &'a [String],
    statement_ids: &'a [String],
    claim_ids: &'a [String],
}

/// Number of scalar linkage fields a Valence receipt must carry.
const VALENCE_SCALAR_FIELD_COUNT: usize = 11;

/// Number of structured linkage fields a Valence receipt must carry.
const VALENCE_LINKAGE_FIELD_COUNT: usize = 7;

fn validate_valence_receipt(expectation: &ValenceReceiptExpectation<'_>) -> Result<String, SourceReviewError> {
    let parsed: ValenceSourceReviewAcceptance =
        serde_json::from_slice(expectation.receipt_bytes).map_err(|_| SourceReviewError::ValenceMalformed)?;
    let mut receipt = serde_json::to_value(parsed).map_err(|_| SourceReviewError::ValenceCanonicalizationFailed)?;
    let object = receipt.as_object_mut().ok_or(SourceReviewError::ValenceMalformed)?;
    let receipt_id = object
        .remove("receipt_blake3")
        .and_then(|value| value.as_str().map(ToString::to_string))
        .ok_or(SourceReviewError::ValenceReceiptIdentityMissing)?;
    if source_review_identity(b"valence.source-review-acceptance.v1\0", &receipt)? != receipt_id {
        return Err(SourceReviewError::ValenceReceiptIdentityMismatch);
    }
    validate_valence_scalar_fields(&receipt, expectation)?;
    validate_valence_linkage_fields(&receipt, expectation)?;
    Ok(receipt_id)
}

/// Requires every scalar linkage field of the receipt to match the evidence.
fn validate_valence_scalar_fields(
    receipt: &Value,
    expectation: &ValenceReceiptExpectation<'_>,
) -> Result<(), SourceReviewError> {
    let expected = [
        ("schema", SOURCE_REVIEW_VALENCE_SCHEMA.to_string()),
        ("outcome", "accepted_manual_review".to_string()),
        ("verification_role", "manual_review".to_string()),
        ("attachment_identity", expectation.attachment_id.to_string()),
        ("verification_policy_identity", expectation.policy_id.to_string()),
        ("authentication_policy_identity", expectation.authentication_policy_id.to_string()),
        ("review_policy_blake3", expectation.policy.review_policy_blake3.as_str().to_string()),
        ("attachment_artifact_blake3", blake3::hash(expectation.attachment_bytes).to_hex().to_string()),
        ("policy_artifact_blake3", blake3::hash(expectation.policy_bytes).to_hex().to_string()),
        (
            "subject_identity",
            source_review_identity(b"cairn.source-review-subject.v1\0", &expectation.policy.subject)?,
        ),
        (
            "claim_root_identity",
            source_review_identity(b"cairn.claim-root.v1\0", &expectation.policy.claim_root)?,
        ),
    ];
    debug_assert_eq!(expected.len(), VALENCE_SCALAR_FIELD_COUNT);
    for (field, value) in expected {
        if receipt.get(field).and_then(Value::as_str) != Some(value.as_str()) {
            return Err(SourceReviewError::ValenceLinkageMismatch(field));
        }
    }
    Ok(())
}

/// Requires every structured linkage field of the receipt to match the evidence.
fn validate_valence_linkage_fields(
    receipt: &Value,
    expectation: &ValenceReceiptExpectation<'_>,
) -> Result<(), SourceReviewError> {
    let mut expected = Vec::with_capacity(VALENCE_LINKAGE_FIELD_COUNT);
    expected.push(("subject", canonical_value(&expectation.policy.subject)?));
    expected.push(("claim_root", canonical_value(&expectation.policy.claim_root)?));
    expected.push(("approved_key_blake3", canonical_value(expectation.approved_keys)?));
    expected.push(("signed_statement_blake3", canonical_value(expectation.statement_ids)?));
    expected.push(("claim_identity", canonical_value(expectation.claim_ids)?));
    expected.push(("non_claims", canonical_value(&expectation.policy.non_claims)?));
    expected.push(("issues", Value::Array(Vec::new())));
    debug_assert_eq!(expected.len(), VALENCE_LINKAGE_FIELD_COUNT);
    for (field, value) in expected {
        if receipt.get(field) != Some(&value) {
            return Err(SourceReviewError::ValenceLinkageMismatch(field));
        }
    }
    Ok(())
}

/// Canonicalizes one linkage value for exact comparison with the receipt.
fn canonical_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, SourceReviewError> {
    serde_json::to_value(value).map_err(|_| SourceReviewError::ValenceCanonicalizationFailed)
}

#[cfg(test)]
mod tests;
