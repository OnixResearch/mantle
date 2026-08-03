//! Pure authority and identity logic for shared Rust unit results.
//!
//! This module does not perform discovery, network access, object transfer, or
//! filesystem mutation. Shell code supplies signed records and explicit trust
//! policy. The core returns a deterministic authority decision.

use std::collections::BTreeSet;

use data_encoding::HEXLOWER;
use ed25519_dalek::Signer;
use ed25519_dalek::Verifier;
use serde::Deserialize;
use serde::Serialize;

use crate::BLAKE3_HEX_CHARS;
use crate::MAX_RECORD_BYTES;
use crate::MAX_STRING_BYTES;
use crate::RustUnitResult;
use crate::validate_rust_result;

pub const SHARED_RUST_ENVELOPE_SCHEMA: &str = "mantle-shared-rust-unit-result-envelope-v1";
pub const SHARED_RUST_TRUST_POLICY_SCHEMA: &str = "mantle-shared-rust-unit-trust-policy-v1";
pub const SHARED_RUST_CLAIM_CLASS: &str = "admitted-rust-unit-result-v1";
pub const SHARED_RUST_ENVELOPE_REF_PREFIX: &str = "mantle-shared-rust-envelope://blake3/";
pub const SHARED_RUST_OBJECT_REF_PREFIX: &str = "mantle-shared-rust-object://blake3/";
pub const MAX_TRUSTED_RUST_RESULT_KEYS: usize = 64;
pub const MAX_ACCEPTED_PRODUCER_POLICIES: usize = 64;
pub const ED25519_PUBLIC_KEY_BYTES: usize = 32;
pub const ED25519_SIGNATURE_BYTES: usize = 64;
pub const HEX_CHARS_PER_BYTE: usize = 2;
pub const SHARED_ENVELOPE_OVERHEAD_BYTES: usize = 16_384;
pub const ED25519_PUBLIC_KEY_HEX_CHARS: usize = ED25519_PUBLIC_KEY_BYTES.saturating_mul(HEX_CHARS_PER_BYTE);
pub const ED25519_SIGNATURE_HEX_CHARS: usize = ED25519_SIGNATURE_BYTES.saturating_mul(HEX_CHARS_PER_BYTE);
pub const MAX_SHARED_ENVELOPE_BYTES: usize = MAX_RECORD_BYTES.saturating_add(SHARED_ENVELOPE_OVERHEAD_BYTES);

const ENVELOPE_DOMAIN: &[u8] = b"mantle.shared-rust-unit.envelope.v1";
const VERIFIER_DOMAIN: &[u8] = b"mantle.shared-rust-unit.verifier.v1";
const SIGNATURE_DOMAIN: &[u8] = b"mantle.shared-rust-unit.signature.v1";
const DOMAIN_SEPARATOR: u8 = 0;

#[derive(Clone, Copy)]
struct ValidationCode<'a>(&'a str);

#[derive(Clone, Copy)]
struct TypedRefRule<'a> {
    prefix: &'a str,
    code: ValidationCode<'a>,
}

#[derive(Clone, Copy)]
struct EnvelopeSignerFacts<'a> {
    signer_name: &'a str,
    verifier_key_blake3: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustResultProducerIdentity {
    pub producer_id: String,
    pub producer_policy_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustResultObjectIdentity {
    pub object_ref: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustResultEnvelope {
    pub schema: String,
    pub envelope_ref: String,
    pub claim_class: String,
    pub result: RustUnitResult,
    pub object: RustResultObjectIdentity,
    pub producer: RustResultProducerIdentity,
    pub signer_name: String,
    pub verifier_key_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedRustResultEnvelope {
    pub envelope: RustResultEnvelope,
    pub verifier_key_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct TrustedRustResultKey {
    pub signer_name: String,
    pub verifier_key_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustResultTrustPolicy {
    pub schema: String,
    pub policy_id: String,
    pub accepted_producer_policy_ids: Vec<String>,
    pub trusted_keys: Vec<TrustedRustResultKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustResultAuthorityDecision {
    pub admitted: bool,
    pub authority_disposition: String,
    pub reason_codes: Vec<String>,
    pub accepted_verifier_blake3: Option<String>,
    pub trust_policy_id: String,
}

#[derive(Debug, Clone, Copy)]
pub struct ExpectedRustResultRefs<'a> {
    pub action_ref: &'a str,
    pub result_ref: &'a str,
}

#[derive(Serialize)]
struct EnvelopeHashable<'a> {
    schema: &'a str,
    claim_class: &'a str,
    result: &'a RustUnitResult,
    object: &'a RustResultObjectIdentity,
    producer: &'a RustResultProducerIdentity,
    signer_name: &'a str,
    verifier_key_blake3: &'a str,
}

pub fn sign_rust_result_envelope(
    result: RustUnitResult,
    object: RustResultObjectIdentity,
    producer: RustResultProducerIdentity,
    signer_name: String,
    signing_key: &ed25519_dalek::SigningKey,
) -> Result<SignedRustResultEnvelope, String> {
    validate_rust_result(&result)?;
    validate_object_identity(&object)?;
    validate_identifier(&producer.producer_id, ValidationCode("shared-rust-producer-id-invalid"))?;
    validate_identifier(&producer.producer_policy_id, ValidationCode("shared-rust-producer-policy-id-invalid"))?;
    validate_identifier(&signer_name, ValidationCode("shared-rust-signer-name-invalid"))?;
    let verifier_key_hex = HEXLOWER.encode(signing_key.verifying_key().as_bytes());
    let verifier_key_blake3 = verifier_key_digest(&verifier_key_hex)?;
    let envelope_ref = envelope_reference(&result, &object, &producer, EnvelopeSignerFacts {
        signer_name: &signer_name,
        verifier_key_blake3: &verifier_key_blake3,
    })?;
    let envelope = RustResultEnvelope {
        schema: SHARED_RUST_ENVELOPE_SCHEMA.to_string(),
        envelope_ref,
        claim_class: SHARED_RUST_CLAIM_CLASS.to_string(),
        result,
        object,
        producer,
        signer_name,
        verifier_key_blake3,
    };
    validate_rust_result_envelope(&envelope)?;
    let signature_message = signature_message(&envelope)?;
    let signature_hex = HEXLOWER.encode(&signing_key.sign(&signature_message).to_bytes());
    let signed = SignedRustResultEnvelope {
        envelope,
        verifier_key_hex,
        signature_hex,
    };
    validate_signed_rust_result_envelope(&signed)?;
    assert_eq!(signed.signature_hex.len(), ED25519_SIGNATURE_HEX_CHARS);
    assert_eq!(signed.verifier_key_hex.len(), ED25519_PUBLIC_KEY_HEX_CHARS);
    Ok(signed)
}

pub fn validate_rust_result_envelope(envelope: &RustResultEnvelope) -> Result<(), String> {
    if envelope.schema != SHARED_RUST_ENVELOPE_SCHEMA {
        return Err("shared-rust-envelope-schema-unsupported".to_string());
    }
    if envelope.claim_class != SHARED_RUST_CLAIM_CLASS {
        return Err("shared-rust-claim-class-unsupported".to_string());
    }
    validate_rust_result(&envelope.result)?;
    validate_object_identity(&envelope.object)?;
    validate_identifier(&envelope.producer.producer_id, ValidationCode("shared-rust-producer-id-invalid"))?;
    validate_identifier(
        &envelope.producer.producer_policy_id,
        ValidationCode("shared-rust-producer-policy-id-invalid"),
    )?;
    validate_identifier(&envelope.signer_name, ValidationCode("shared-rust-signer-name-invalid"))?;
    validate_blake3(&envelope.verifier_key_blake3, ValidationCode("shared-rust-verifier-key-digest-invalid"))?;
    let expected_ref =
        envelope_reference(&envelope.result, &envelope.object, &envelope.producer, EnvelopeSignerFacts {
            signer_name: &envelope.signer_name,
            verifier_key_blake3: &envelope.verifier_key_blake3,
        })?;
    if envelope.envelope_ref != expected_ref {
        return Err("shared-rust-envelope-ref-mismatch".to_string());
    }
    let bytes = serde_json::to_vec(envelope).map_err(|error| format!("shared-rust-envelope-json:{error}"))?;
    if bytes.len() > MAX_SHARED_ENVELOPE_BYTES {
        return Err("shared-rust-envelope-too-large".to_string());
    }
    assert!(envelope.envelope_ref.starts_with(SHARED_RUST_ENVELOPE_REF_PREFIX));
    assert!(bytes.len() <= MAX_SHARED_ENVELOPE_BYTES);
    Ok(())
}

pub fn validate_signed_rust_result_envelope(signed: &SignedRustResultEnvelope) -> Result<(), String> {
    validate_rust_result_envelope(&signed.envelope)?;
    let verifier_bytes =
        decode_exact_hex(&signed.verifier_key_hex, ED25519_PUBLIC_KEY_BYTES, "shared-rust-verifier-key-invalid")?;
    let signature_bytes =
        decode_exact_hex(&signed.signature_hex, ED25519_SIGNATURE_BYTES, "shared-rust-signature-invalid")?;
    let actual_verifier_digest = domain_digest(VERIFIER_DOMAIN, &verifier_bytes);
    if signed.envelope.verifier_key_blake3 != actual_verifier_digest {
        return Err("shared-rust-verifier-key-digest-mismatch".to_string());
    }
    let bytes = serde_json::to_vec(signed).map_err(|error| format!("shared-rust-signed-envelope-json:{error}"))?;
    if bytes.len() > MAX_SHARED_ENVELOPE_BYTES {
        return Err("shared-rust-signed-envelope-too-large".to_string());
    }
    assert_eq!(verifier_bytes.len(), ED25519_PUBLIC_KEY_BYTES);
    assert_eq!(signature_bytes.len(), ED25519_SIGNATURE_BYTES);
    Ok(())
}

pub fn evaluate_rust_result_authority(
    signed: &SignedRustResultEnvelope,
    policy: &RustResultTrustPolicy,
    expected: ExpectedRustResultRefs<'_>,
) -> RustResultAuthorityDecision {
    let mut reasons = BTreeSet::new();
    if let Err(reason) = validate_trust_policy(policy) {
        reasons.insert(reason);
    }
    if let Err(reason) = validate_signed_rust_result_envelope(signed) {
        reasons.insert(reason);
    }
    if signed.envelope.result.input.action_ref != expected.action_ref {
        reasons.insert("shared-rust-action-ref-mismatch".to_string());
    }
    if signed.envelope.result.result_ref != expected.result_ref {
        reasons.insert("shared-rust-result-ref-mismatch".to_string());
    }
    if !policy.accepted_producer_policy_ids.contains(&signed.envelope.producer.producer_policy_id) {
        reasons.insert("shared-rust-producer-policy-untrusted".to_string());
    }
    let is_trusted = policy
        .trusted_keys
        .iter()
        .any(|key| key.signer_name == signed.envelope.signer_name && key.verifier_key_hex == signed.verifier_key_hex);
    if !is_trusted {
        reasons.insert("shared-rust-verifier-key-untrusted".to_string());
    }
    if reasons.is_empty()
        && let Err(reason) = verify_signature(signed)
    {
        reasons.insert(reason);
    }
    let is_admitted = reasons.is_empty();
    let reason_codes = reasons.into_iter().collect::<Vec<_>>();
    let accepted_verifier_blake3 = is_admitted.then(|| signed.envelope.verifier_key_blake3.clone());
    let authority_disposition = if is_admitted {
        "accepted".to_string()
    } else {
        "rejected".to_string()
    };
    assert_eq!(is_admitted, reason_codes.is_empty());
    assert_eq!(accepted_verifier_blake3.is_some(), is_admitted);
    RustResultAuthorityDecision {
        admitted: is_admitted,
        authority_disposition,
        reason_codes,
        accepted_verifier_blake3,
        trust_policy_id: policy.policy_id.clone(),
    }
}

pub fn validate_trust_policy(policy: &RustResultTrustPolicy) -> Result<(), String> {
    if policy.schema != SHARED_RUST_TRUST_POLICY_SCHEMA {
        return Err("shared-rust-trust-policy-schema-unsupported".to_string());
    }
    validate_identifier(&policy.policy_id, ValidationCode("shared-rust-trust-policy-id-invalid"))?;
    if policy.accepted_producer_policy_ids.is_empty()
        || policy.accepted_producer_policy_ids.len() > MAX_ACCEPTED_PRODUCER_POLICIES
    {
        return Err("shared-rust-producer-policy-count-invalid".to_string());
    }
    if policy.trusted_keys.is_empty() || policy.trusted_keys.len() > MAX_TRUSTED_RUST_RESULT_KEYS {
        return Err("shared-rust-trusted-key-count-invalid".to_string());
    }
    if policy.accepted_producer_policy_ids != sorted_unique(policy.accepted_producer_policy_ids.clone()) {
        return Err("shared-rust-producer-policies-not-canonical".to_string());
    }
    if policy.trusted_keys != sorted_unique(policy.trusted_keys.clone()) {
        return Err("shared-rust-trusted-keys-not-canonical".to_string());
    }
    for producer_policy in &policy.accepted_producer_policy_ids {
        validate_identifier(producer_policy, ValidationCode("shared-rust-producer-policy-id-invalid"))?;
    }
    for trusted_key in &policy.trusted_keys {
        validate_identifier(&trusted_key.signer_name, ValidationCode("shared-rust-trusted-key-name-invalid"))?;
        decode_exact_hex(&trusted_key.verifier_key_hex, ED25519_PUBLIC_KEY_BYTES, "shared-rust-trusted-key-invalid")?;
    }
    assert!(policy.trusted_keys.len() <= MAX_TRUSTED_RUST_RESULT_KEYS);
    assert!(policy.accepted_producer_policy_ids.len() <= MAX_ACCEPTED_PRODUCER_POLICIES);
    Ok(())
}

pub fn verifier_key_digest(verifier_key_hex: &str) -> Result<String, String> {
    let bytes = decode_exact_hex(verifier_key_hex, ED25519_PUBLIC_KEY_BYTES, "shared-rust-verifier-key-invalid")?;
    let digest = domain_digest(VERIFIER_DOMAIN, &bytes);
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert_eq!(bytes.len(), ED25519_PUBLIC_KEY_BYTES);
    Ok(digest)
}

fn envelope_reference(
    result: &RustUnitResult,
    object: &RustResultObjectIdentity,
    producer: &RustResultProducerIdentity,
    signer: EnvelopeSignerFacts<'_>,
) -> Result<String, String> {
    let hashable = EnvelopeHashable {
        schema: SHARED_RUST_ENVELOPE_SCHEMA,
        claim_class: SHARED_RUST_CLAIM_CLASS,
        result,
        object,
        producer,
        signer_name: signer.signer_name,
        verifier_key_blake3: signer.verifier_key_blake3,
    };
    let bytes = serde_json::to_vec(&hashable).map_err(|error| format!("shared-rust-envelope-json:{error}"))?;
    if bytes.len() > MAX_SHARED_ENVELOPE_BYTES {
        return Err("shared-rust-envelope-too-large".to_string());
    }
    let digest = domain_digest(ENVELOPE_DOMAIN, &bytes);
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert!(bytes.len() <= MAX_SHARED_ENVELOPE_BYTES);
    Ok(format!("{SHARED_RUST_ENVELOPE_REF_PREFIX}{digest}"))
}

fn validate_object_identity(object: &RustResultObjectIdentity) -> Result<(), String> {
    validate_typed_ref(&object.object_ref, TypedRefRule {
        prefix: SHARED_RUST_OBJECT_REF_PREFIX,
        code: ValidationCode("shared-rust-object-ref-invalid"),
    })?;
    if object.size_bytes == 0 || object.size_bytes > crate::MAX_TREE_BYTES {
        return Err("shared-rust-object-size-invalid".to_string());
    }
    assert!(object.object_ref.starts_with(SHARED_RUST_OBJECT_REF_PREFIX));
    assert!(object.size_bytes <= crate::MAX_TREE_BYTES);
    Ok(())
}

fn validate_typed_ref(value: &str, rule: TypedRefRule<'_>) -> Result<(), String> {
    let Some(digest) = value.strip_prefix(rule.prefix) else {
        return Err(rule.code.0.to_string());
    };
    validate_blake3(digest, rule.code)?;
    assert_eq!(value.len(), rule.prefix.len().saturating_add(BLAKE3_HEX_CHARS));
    assert!(value.starts_with(rule.prefix));
    Ok(())
}

fn verify_signature(signed: &SignedRustResultEnvelope) -> Result<(), String> {
    let verifier_bytes =
        decode_exact_hex(&signed.verifier_key_hex, ED25519_PUBLIC_KEY_BYTES, "shared-rust-verifier-key-invalid")?;
    let signature_bytes =
        decode_exact_hex(&signed.signature_hex, ED25519_SIGNATURE_BYTES, "shared-rust-signature-invalid")?;
    let verifier_array: [u8; ED25519_PUBLIC_KEY_BYTES] =
        verifier_bytes.try_into().map_err(|_| "shared-rust-verifier-key-invalid".to_string())?;
    let signature_array: [u8; ED25519_SIGNATURE_BYTES] =
        signature_bytes.try_into().map_err(|_| "shared-rust-signature-invalid".to_string())?;
    let verifier = ed25519_dalek::VerifyingKey::from_bytes(&verifier_array)
        .map_err(|_| "shared-rust-verifier-key-invalid".to_string())?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_array);
    let message = signature_message(&signed.envelope)?;
    verifier.verify(&message, &signature).map_err(|_| "shared-rust-signature-invalid".to_string())?;
    assert_eq!(message[0..SIGNATURE_DOMAIN.len()], *SIGNATURE_DOMAIN);
    assert!(!message.is_empty());
    Ok(())
}

fn signature_message(envelope: &RustResultEnvelope) -> Result<Vec<u8>, String> {
    let canonical = serde_json::to_vec(envelope).map_err(|error| format!("shared-rust-envelope-json:{error}"))?;
    if canonical.len() > MAX_SHARED_ENVELOPE_BYTES {
        return Err("shared-rust-envelope-too-large".to_string());
    }
    let mut message = Vec::with_capacity(
        SIGNATURE_DOMAIN
            .len()
            .checked_add(1)
            .and_then(|value| value.checked_add(canonical.len()))
            .ok_or_else(|| "shared-rust-signature-message-overflow".to_string())?,
    );
    message.extend_from_slice(SIGNATURE_DOMAIN);
    message.push(DOMAIN_SEPARATOR);
    message.extend_from_slice(&canonical);
    assert!(message.len() > canonical.len());
    assert_eq!(message.capacity(), message.len());
    Ok(message)
}

fn decode_exact_hex(value: &str, expected_bytes: usize, code: &str) -> Result<Vec<u8>, String> {
    let expected_chars = expected_bytes.checked_mul(2).ok_or_else(|| code.to_string())?;
    if value.len() != expected_chars {
        return Err(code.to_string());
    }
    let decoded = HEXLOWER.decode(value.as_bytes()).map_err(|_| code.to_string())?;
    if decoded.len() != expected_bytes || HEXLOWER.encode(&decoded) != value {
        return Err(code.to_string());
    }
    assert_eq!(decoded.len(), expected_bytes);
    assert_eq!(value.len(), expected_chars);
    Ok(decoded)
}

fn validate_identifier(value: &str, code: ValidationCode<'_>) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_STRING_BYTES {
        return Err(code.0.to_string());
    }
    if value.bytes().any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace()) {
        return Err(code.0.to_string());
    }
    assert!(!value.is_empty());
    assert!(value.len() <= MAX_STRING_BYTES);
    Ok(())
}

fn validate_blake3(value: &str, code: ValidationCode<'_>) -> Result<(), String> {
    if value.len() != BLAKE3_HEX_CHARS
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(code.0.to_string());
    }
    assert_eq!(value.len(), BLAKE3_HEX_CHARS);
    assert!(value.bytes().all(|byte| !byte.is_ascii_uppercase()));
    Ok(())
}

fn domain_digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    digest
}

fn sorted_unique<T: Ord>(values: Vec<T>) -> Vec<T> {
    let set = values.into_iter().collect::<BTreeSet<_>>();
    let values = set.into_iter().collect::<Vec<_>>();
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(values.len() <= values.capacity());
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CastoreNodeIdentity;
    use crate::CastoreNodeKind;
    use crate::RUST_ACTION_REF_PREFIX;
    use crate::RUST_RESULT_REF_PREFIX;
    use crate::RustArtifactKind;
    use crate::RustResultArtifact;
    use crate::RustUnitResultInput;
    use crate::canonical_rust_result;

    const TEST_KEY_BYTE: u8 = 17;
    const WRONG_KEY_BYTE: u8 = 29;
    const ACTION_HEX: char = 'a';
    const OTHER_ACTION_HEX: char = 'b';
    const DIGEST_HEX: char = 'c';
    const RECEIPT_HEX: char = 'd';

    struct Fixture {
        signed: SignedRustResultEnvelope,
        policy: RustResultTrustPolicy,
    }

    #[test]
    fn accepted_full_key_and_policy_admit_envelope() {
        let fixture = fixture();
        let decision = evaluate(&fixture.signed, &fixture.policy);

        assert!(decision.admitted);
        assert_eq!(decision.authority_disposition, "accepted");
        assert!(decision.reason_codes.is_empty());
        assert_eq!(
            decision.accepted_verifier_blake3.as_deref(),
            Some(fixture.signed.envelope.verifier_key_blake3.as_str())
        );
    }

    #[test]
    fn unknown_key_is_rejected() {
        let fixture = fixture();
        let wrong_key = signing_key(WRONG_KEY_BYTE);
        let mut policy = fixture.policy;
        policy.trusted_keys[0].verifier_key_hex = HEXLOWER.encode(wrong_key.verifying_key().as_bytes());

        let decision = evaluate(&fixture.signed, &policy);

        assert!(!decision.admitted);
        assert_eq!(decision.reason_codes, ["shared-rust-verifier-key-untrusted"]);
        assert!(decision.accepted_verifier_blake3.is_none());
    }

    #[test]
    fn duplicate_signer_name_does_not_replace_full_key_match() {
        let fixture = fixture();
        let wrong_key = signing_key(WRONG_KEY_BYTE);
        let mut policy = fixture.policy;
        policy.trusted_keys[0] = TrustedRustResultKey {
            signer_name: fixture.signed.envelope.signer_name.clone(),
            verifier_key_hex: HEXLOWER.encode(wrong_key.verifying_key().as_bytes()),
        };

        let decision = evaluate(&fixture.signed, &policy);

        assert!(!decision.admitted);
        assert_eq!(decision.reason_codes, ["shared-rust-verifier-key-untrusted"]);
        assert_eq!(decision.authority_disposition, "rejected");
    }

    #[test]
    fn modified_record_is_rejected_before_signature_authority() {
        let fixture = fixture();
        let mut signed = fixture.signed;
        signed.envelope.result.input.artifacts[0].digest_blake3 = hex('e');

        let decision = evaluate(&signed, &fixture.policy);

        assert!(!decision.admitted);
        assert!(decision.reason_codes.contains(&"rust-result-ref-mismatch".to_string()));
        assert!(decision.accepted_verifier_blake3.is_none());
    }

    #[test]
    fn wrong_expected_action_reference_is_rejected() {
        let fixture = fixture();
        let expected_action = typed_ref(RUST_ACTION_REF_PREFIX, OTHER_ACTION_HEX);
        let expected_result = fixture.signed.envelope.result.result_ref.clone();
        let decision = evaluate_rust_result_authority(&fixture.signed, &fixture.policy, ExpectedRustResultRefs {
            action_ref: &expected_action,
            result_ref: &expected_result,
        });

        assert!(!decision.admitted);
        assert_eq!(decision.reason_codes, ["shared-rust-action-ref-mismatch"]);
        assert!(decision.accepted_verifier_blake3.is_none());
    }

    #[test]
    fn wrong_expected_result_reference_is_rejected() {
        let fixture = fixture();
        let action_ref = fixture.signed.envelope.result.input.action_ref.clone();
        let result_ref = typed_ref(RUST_RESULT_REF_PREFIX, OTHER_ACTION_HEX);
        let decision = evaluate_rust_result_authority(&fixture.signed, &fixture.policy, ExpectedRustResultRefs {
            action_ref: &action_ref,
            result_ref: &result_ref,
        });

        assert!(!decision.admitted);
        assert_eq!(decision.reason_codes, ["shared-rust-result-ref-mismatch"]);
        assert!(decision.accepted_verifier_blake3.is_none());
    }

    #[test]
    fn modified_signature_is_rejected() {
        let fixture = fixture();
        let mut signed = fixture.signed;
        signed.signature_hex.replace_range(0..2, "00");

        let decision = evaluate(&signed, &fixture.policy);

        assert!(!decision.admitted);
        assert_eq!(decision.reason_codes, ["shared-rust-signature-invalid"]);
        assert!(decision.accepted_verifier_blake3.is_none());
    }

    #[test]
    fn untrusted_producer_policy_is_rejected() {
        let fixture = fixture();
        let mut policy = fixture.policy;
        policy.accepted_producer_policy_ids = vec!["different-producer-policy-v1".to_string()];

        let decision = evaluate(&fixture.signed, &policy);

        assert!(!decision.admitted);
        assert_eq!(decision.reason_codes, ["shared-rust-producer-policy-untrusted"]);
        assert!(decision.accepted_verifier_blake3.is_none());
    }

    fn fixture() -> Fixture {
        let result = canonical_rust_result(RustUnitResultInput {
            action_ref: typed_ref(RUST_ACTION_REF_PREFIX, ACTION_HEX),
            root_node: CastoreNodeIdentity {
                kind: CastoreNodeKind::Directory,
                digest_blake3: hex(DIGEST_HEX),
                size_bytes: 7,
            },
            artifacts: vec![RustResultArtifact {
                relative_path: "libfixture.rlib".to_string(),
                kind: RustArtifactKind::File,
                mode: 0o644,
                size_bytes: 7,
                digest_blake3: hex(DIGEST_HEX),
            }],
            producer_receipt_ref: format!("mantle-rust-receipt://blake3/{}", hex(RECEIPT_HEX)),
        })
        .expect("canonical result");
        let signing_key = signing_key(TEST_KEY_BYTE);
        let signed = sign_rust_result_envelope(
            result,
            RustResultObjectIdentity {
                object_ref: typed_ref(SHARED_RUST_OBJECT_REF_PREFIX, DIGEST_HEX),
                size_bytes: 128,
            },
            RustResultProducerIdentity {
                producer_id: "fixture-builder".to_string(),
                producer_policy_id: "fixture-producer-policy-v1".to_string(),
            },
            "fixture-key".to_string(),
            &signing_key,
        )
        .expect("signed envelope");
        let policy = RustResultTrustPolicy {
            schema: SHARED_RUST_TRUST_POLICY_SCHEMA.to_string(),
            policy_id: "fixture-trust-policy-v1".to_string(),
            accepted_producer_policy_ids: vec!["fixture-producer-policy-v1".to_string()],
            trusted_keys: vec![TrustedRustResultKey {
                signer_name: signed.envelope.signer_name.clone(),
                verifier_key_hex: signed.verifier_key_hex.clone(),
            }],
        };
        validate_trust_policy(&policy).expect("valid trust policy");
        Fixture { signed, policy }
    }

    fn evaluate(signed: &SignedRustResultEnvelope, policy: &RustResultTrustPolicy) -> RustResultAuthorityDecision {
        evaluate_rust_result_authority(signed, policy, ExpectedRustResultRefs {
            action_ref: &signed.envelope.result.input.action_ref,
            result_ref: &signed.envelope.result.result_ref,
        })
    }

    fn signing_key(byte: u8) -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&[byte; ED25519_PUBLIC_KEY_BYTES])
    }

    fn typed_ref(prefix: &str, byte: char) -> String {
        format!("{prefix}{}", hex(byte))
    }

    fn hex(byte: char) -> String {
        std::iter::repeat_n(byte, BLAKE3_HEX_CHARS).collect()
    }
}
