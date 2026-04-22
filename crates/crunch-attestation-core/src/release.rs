use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::AttestationDigest;
use crate::Error;
use crate::canonical::to_canonical_bytes;

pub const RELEASE_ATTESTATION_SCHEMA: &str = "crunch-release-attestation-v1";
pub const WITNESS_ATTESTATION_SCHEMA: &str = "crunch-witness-attestation-v1";

const MAX_BINARY_DIGEST_COUNT: u32 = 256;
const MAX_ENV_FIELD_LEN: u32 = 256;
const ED25519_SIGNATURE_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct BinaryDigest {
    pub name: String,
    pub algorithm: String,
    pub digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BinaryDigestMatchInput {
    pub published: Vec<BinaryDigest>,
    pub rebuilt: Vec<BinaryDigest>,
}

pub fn binary_digests_match(input: BinaryDigestMatchInput) -> bool {
    debug_assert!(is_sorted(&input.published), "published digests must be sorted");
    debug_assert!(is_sorted(&input.rebuilt), "rebuilt digests must be sorted");
    input.published == input.rebuilt
}

fn is_sorted(digests: &[BinaryDigest]) -> bool {
    digests.windows(2).all(|pair| pair[0] <= pair[1])
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    pub command: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RebuildEnvironmentSummary {
    pub system: String,
    pub toolchain: String,
    pub host_class: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignatureSuite {
    #[serde(rename = "ed25519-detached-v1")]
    Ed25519DetachedV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetachedSignature {
    pub key_name: String,
    pub signature_bytes: [u8; ED25519_SIGNATURE_BYTES],
}

pub fn parse_detached_signature(line: String) -> Result<DetachedSignature, Error> {
    let colon_pos = line.find(':').ok_or_else(|| Error::InvalidDetachedSignature {
        message: "missing colon separator".to_string(),
    })?;
    let key_name = &line[..colon_pos];
    let sig_b64 = &line[colon_pos.saturating_add(1)..];

    if key_name.is_empty() {
        return Err(Error::InvalidDetachedSignature {
            message: "empty key name".to_string(),
        });
    }

    if sig_b64.is_empty() {
        return Err(Error::InvalidDetachedSignature {
            message: "empty signature payload".to_string(),
        });
    }

    let decoded = data_encoding::BASE64.decode(sig_b64.as_bytes()).map_err(|err| Error::InvalidDetachedSignature {
        message: alloc::format!("invalid base64: {err}"),
    })?;

    let signature_bytes: [u8; ED25519_SIGNATURE_BYTES] =
        decoded.try_into().map_err(|v: Vec<u8>| Error::InvalidDetachedSignature {
            message: alloc::format!("expected {ED25519_SIGNATURE_BYTES} signature bytes, got {}", v.len()),
        })?;

    Ok(DetachedSignature {
        key_name: key_name.to_string(),
        signature_bytes,
    })
}

pub fn encode_detached_signature(signature: DetachedSignature) -> String {
    let encoded = data_encoding::BASE64.encode(&signature.signature_bytes);
    alloc::format!("{}:{encoded}", signature.key_name)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAttestation {
    pub schema: String,
    pub release_id: String,
    pub release_evidence_manifest_digest_blake3: AttestationDigest,
    pub proof_bundle_digest_blake3: AttestationDigest,
    pub proof_mode: String,
    pub workflow: Workflow,
    pub binary_digests: Vec<BinaryDigest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAttestationInit {
    pub release_id: String,
    pub release_evidence_manifest_digest_blake3: AttestationDigest,
    pub proof_bundle_digest_blake3: AttestationDigest,
    pub proof_mode: String,
    pub workflow: Workflow,
    pub binary_digests: Vec<BinaryDigest>,
}

impl ReleaseAttestation {
    pub fn new(init: ReleaseAttestationInit) -> Self {
        Self {
            schema: RELEASE_ATTESTATION_SCHEMA.to_string(),
            release_id: init.release_id,
            release_evidence_manifest_digest_blake3: init.release_evidence_manifest_digest_blake3,
            proof_bundle_digest_blake3: init.proof_bundle_digest_blake3,
            proof_mode: init.proof_mode,
            workflow: init.workflow,
            binary_digests: init.binary_digests,
        }
    }
}

pub fn canonical_release_attestation(value: ReleaseAttestation) -> Result<ReleaseAttestation, Error> {
    validate_schema_tag(SchemaTag {
        actual: &value.schema,
        expected: RELEASE_ATTESTATION_SCHEMA,
    })?;
    validate_non_empty(NamedField {
        name: "release_id",
        value: &value.release_id,
    })?;
    validate_non_empty(NamedField {
        name: "proof_mode",
        value: &value.proof_mode,
    })?;
    validate_non_empty(NamedField {
        name: "workflow.command",
        value: &value.workflow.command,
    })?;
    validate_non_empty(NamedField {
        name: "workflow.version",
        value: &value.workflow.version,
    })?;
    let binary_digests = normalize_binary_digests(value.binary_digests)?;

    Ok(ReleaseAttestation {
        schema: RELEASE_ATTESTATION_SCHEMA.to_string(),
        release_id: value.release_id,
        release_evidence_manifest_digest_blake3: value.release_evidence_manifest_digest_blake3,
        proof_bundle_digest_blake3: value.proof_bundle_digest_blake3,
        proof_mode: value.proof_mode,
        workflow: value.workflow,
        binary_digests,
    })
}

pub fn release_attestation_canonical_bytes(value: ReleaseAttestation) -> Result<Vec<u8>, Error> {
    let canonical = canonical_release_attestation(value)?;
    to_canonical_bytes(&canonical)
}

pub fn release_attestation_canonical_digest(value: ReleaseAttestation) -> Result<AttestationDigest, Error> {
    let bytes = release_attestation_canonical_bytes(value)?;
    Ok(AttestationDigest::from_canonical_bytes(&bytes))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WitnessAttestation {
    pub schema: String,
    pub release_attestation_digest_blake3: AttestationDigest,
    pub witness_identity: String,
    pub signature_suite: SignatureSuite,
    pub rebuilt_digests: Vec<BinaryDigest>,
    pub rebuild_environment_summary: RebuildEnvironmentSummary,
}

impl WitnessAttestation {
    pub fn new(
        release_attestation_digest_blake3: AttestationDigest,
        witness_identity: String,
        rebuilt_digests: Vec<BinaryDigest>,
        rebuild_environment_summary: RebuildEnvironmentSummary,
    ) -> Self {
        Self {
            schema: WITNESS_ATTESTATION_SCHEMA.to_string(),
            release_attestation_digest_blake3,
            witness_identity,
            signature_suite: SignatureSuite::Ed25519DetachedV1,
            rebuilt_digests,
            rebuild_environment_summary,
        }
    }
}

pub fn canonical_witness_attestation(value: WitnessAttestation) -> Result<WitnessAttestation, Error> {
    validate_schema_tag(SchemaTag {
        actual: &value.schema,
        expected: WITNESS_ATTESTATION_SCHEMA,
    })?;
    validate_non_empty(NamedField {
        name: "witness_identity",
        value: &value.witness_identity,
    })?;
    validate_env_field(NamedField {
        name: "system",
        value: &value.rebuild_environment_summary.system,
    })?;
    validate_env_field(NamedField {
        name: "toolchain",
        value: &value.rebuild_environment_summary.toolchain,
    })?;
    validate_env_field(NamedField {
        name: "host_class",
        value: &value.rebuild_environment_summary.host_class,
    })?;
    let rebuilt_digests = normalize_binary_digests(value.rebuilt_digests)?;

    Ok(WitnessAttestation {
        schema: WITNESS_ATTESTATION_SCHEMA.to_string(),
        release_attestation_digest_blake3: value.release_attestation_digest_blake3,
        witness_identity: value.witness_identity,
        signature_suite: value.signature_suite,
        rebuilt_digests,
        rebuild_environment_summary: value.rebuild_environment_summary,
    })
}

pub fn witness_attestation_canonical_bytes(value: WitnessAttestation) -> Result<Vec<u8>, Error> {
    let canonical = canonical_witness_attestation(value)?;
    to_canonical_bytes(&canonical)
}

pub fn witness_attestation_canonical_digest(value: WitnessAttestation) -> Result<AttestationDigest, Error> {
    let bytes = witness_attestation_canonical_bytes(value)?;
    Ok(AttestationDigest::from_canonical_bytes(&bytes))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TechnicalClass {
    BundleConsistent,
    SelfProofValid,
    ExternalWitnessMatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyStatus {
    NotEvaluated,
    Insufficient,
    Satisfied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinalClass {
    BundleConsistent,
    SelfProofValid,
    ExternalWitnessMatch,
    QuorumSatisfied,
}

impl FinalClass {
    pub fn resolve(technical: TechnicalClass, policy: PolicyStatus) -> Self {
        if policy == PolicyStatus::Satisfied {
            return Self::QuorumSatisfied;
        }
        match technical {
            TechnicalClass::BundleConsistent => Self::BundleConsistent,
            TechnicalClass::SelfProofValid => Self::SelfProofValid,
            TechnicalClass::ExternalWitnessMatch => Self::ExternalWitnessMatch,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrustTier {
    pub technical_class: TechnicalClass,
    pub policy_status: PolicyStatus,
    pub final_class: FinalClass,
}

fn normalize_binary_digests(digests: Vec<BinaryDigest>) -> Result<Vec<BinaryDigest>, Error> {
    assert!(MAX_BINARY_DIGEST_COUNT >= 1, "binary digest limit must be positive");
    let actual = count_with_overflow_marker(digests.len(), MAX_BINARY_DIGEST_COUNT);
    if actual > MAX_BINARY_DIGEST_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_BINARY_DIGEST_COUNT,
            actual,
        });
    }
    for digest in &digests {
        validate_non_empty(NamedField {
            name: "binary_digest.name",
            value: &digest.name,
        })?;
        validate_non_empty(NamedField {
            name: "binary_digest.algorithm",
            value: &digest.algorithm,
        })?;
        validate_non_empty(NamedField {
            name: "binary_digest.digest",
            value: &digest.digest,
        })?;
    }
    let mut sorted = digests;
    sorted.sort();
    Ok(sorted)
}

struct SchemaTag<'a> {
    actual: &'a str,
    expected: &'static str,
}

struct NamedField<'a> {
    name: &'static str,
    value: &'a str,
}

fn validate_schema_tag(tag: SchemaTag<'_>) -> Result<(), Error> {
    if tag.actual == tag.expected {
        return Ok(());
    }
    Err(Error::SchemaTagMismatch {
        expected: tag.expected,
        actual: tag.actual.to_string(),
    })
}

fn validate_non_empty(field: NamedField<'_>) -> Result<(), Error> {
    if !field.value.is_empty() {
        return Ok(());
    }
    Err(Error::EmptyField { field: field.name })
}

fn validate_env_field(field: NamedField<'_>) -> Result<(), Error> {
    validate_non_empty(NamedField {
        name: field.name,
        value: field.value,
    })?;
    let actual = count_with_overflow_marker(field.value.len(), MAX_ENV_FIELD_LEN);
    if actual > MAX_ENV_FIELD_LEN {
        return Err(Error::FieldTooLong {
            field: field.name,
            limit: MAX_ENV_FIELD_LEN,
            actual,
        });
    }
    Ok(())
}

fn count_with_overflow_marker(value_count: usize, overflow_floor: u32) -> u32 {
    match u32::try_from(value_count) {
        Ok(value) => value,
        Err(_) => overflow_floor.saturating_add(1),
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test]
    fn release_attestation_digest_is_stable_across_serializations() {
        let attestation = sample_release();
        let bytes_1 = release_attestation_canonical_bytes(attestation.clone()).unwrap();
        let bytes_2 = release_attestation_canonical_bytes(attestation).unwrap();
        assert_eq!(bytes_1, bytes_2);

        let digest_1 = release_attestation_canonical_digest(sample_release()).unwrap();
        let digest_2 = release_attestation_canonical_digest(sample_release()).unwrap();
        assert_eq!(digest_1, digest_2);
    }

    #[test]
    fn witness_attestation_digest_is_stable_across_serializations() {
        let attestation = sample_witness();
        let bytes_1 = witness_attestation_canonical_bytes(attestation.clone()).unwrap();
        let bytes_2 = witness_attestation_canonical_bytes(attestation).unwrap();
        assert_eq!(bytes_1, bytes_2);

        let digest_1 = witness_attestation_canonical_digest(sample_witness()).unwrap();
        let digest_2 = witness_attestation_canonical_digest(sample_witness()).unwrap();
        assert_eq!(digest_1, digest_2);
    }

    #[test]
    fn release_attestation_binds_manifest_and_binary_digests() {
        let manifest_digest = AttestationDigest::from_canonical_bytes(b"manifest-content");
        let proof_digest = AttestationDigest::from_canonical_bytes(b"proof-bundle");
        let attestation = ReleaseAttestation::new(ReleaseAttestationInit {
            release_id: "crunch-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: manifest_digest,
            proof_bundle_digest_blake3: proof_digest,
            proof_mode: "fixed-point".to_string(),
            workflow: Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            binary_digests: vec![BinaryDigest {
                name: "crunch".to_string(),
                algorithm: "blake3".to_string(),
                digest: manifest_digest.to_hex(),
            }],
        });

        assert_eq!(attestation.release_evidence_manifest_digest_blake3, manifest_digest);
        assert_eq!(attestation.proof_bundle_digest_blake3, proof_digest);
        assert_eq!(attestation.binary_digests.len(), 1);
        assert_eq!(attestation.binary_digests[0].algorithm, "blake3");

        let _digest = release_attestation_canonical_digest(attestation).unwrap();
    }

    #[test]
    fn witness_attestation_binds_to_release_digest() {
        let release = sample_release();
        let release_digest = release_attestation_canonical_digest(release.clone()).unwrap();

        let witness = WitnessAttestation::new(
            release_digest,
            "witness-a".to_string(),
            release.binary_digests,
            RebuildEnvironmentSummary {
                system: "x86_64-linux".to_string(),
                toolchain: "rust-1.91.1".to_string(),
                host_class: "nixos-25.05".to_string(),
            },
        );

        assert_eq!(witness.release_attestation_digest_blake3, release_digest);
        let _digest = witness_attestation_canonical_digest(witness).unwrap();
    }

    #[test]
    fn binary_digests_sorted_regardless_of_insertion_order() {
        let d_a = BinaryDigest {
            name: "alpha".to_string(),
            algorithm: "blake3".to_string(),
            digest: "aa".repeat(32),
        };
        let d_b = BinaryDigest {
            name: "beta".to_string(),
            algorithm: "blake3".to_string(),
            digest: "bb".repeat(32),
        };

        let forward = release_with_digests(vec![d_a.clone(), d_b.clone()]);
        let reversed = release_with_digests(vec![d_b, d_a]);

        let forward_bytes = release_attestation_canonical_bytes(forward).unwrap();
        let reversed_bytes = release_attestation_canonical_bytes(reversed).unwrap();
        assert_eq!(forward_bytes, reversed_bytes);
    }

    #[test]
    fn matching_digest_sets_compare_equal() {
        let digests = vec![BinaryDigest {
            name: "crunch".to_string(),
            algorithm: "blake3".to_string(),
            digest: "aa".repeat(32),
        }];
        assert!(binary_digests_match(BinaryDigestMatchInput {
            published: digests.clone(),
            rebuilt: digests,
        }));
    }

    #[test]
    fn mismatched_digest_sets_compare_unequal() {
        let published = vec![BinaryDigest {
            name: "crunch".to_string(),
            algorithm: "blake3".to_string(),
            digest: "aa".repeat(32),
        }];
        let rebuilt = vec![BinaryDigest {
            name: "crunch".to_string(),
            algorithm: "blake3".to_string(),
            digest: "bb".repeat(32),
        }];
        assert!(!binary_digests_match(BinaryDigestMatchInput { published, rebuilt }));
    }

    #[test]
    fn release_rejects_wrong_schema_tag() {
        let mut attestation = sample_release();
        attestation.schema = "wrong-schema".to_string();

        let err = release_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::SchemaTagMismatch {
            expected: RELEASE_ATTESTATION_SCHEMA,
            actual: "wrong-schema".to_string(),
        });
    }

    #[test]
    fn witness_rejects_wrong_schema_tag() {
        let mut attestation = sample_witness();
        attestation.schema = "wrong-schema".to_string();

        let err = witness_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::SchemaTagMismatch {
            expected: WITNESS_ATTESTATION_SCHEMA,
            actual: "wrong-schema".to_string(),
        });
    }

    #[test]
    fn release_rejects_empty_release_id() {
        let mut attestation = sample_release();
        attestation.release_id.clear();

        let err = release_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::EmptyField { field: "release_id" });
    }

    #[test]
    fn witness_rejects_empty_witness_identity() {
        let mut attestation = sample_witness();
        attestation.witness_identity.clear();

        let err = witness_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::EmptyField {
            field: "witness_identity"
        });
    }

    #[test]
    fn release_rejects_empty_binary_digest_name() {
        let mut attestation = sample_release();
        attestation.binary_digests[0].name.clear();

        let err = release_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::EmptyField {
            field: "binary_digest.name"
        });
    }

    #[test]
    fn witness_rejects_oversized_env_field() {
        let mut attestation = sample_witness();
        attestation.rebuild_environment_summary.system = "x".repeat(257);

        let err = witness_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::FieldTooLong {
            field: "system",
            limit: 256,
            actual: 257,
        });
    }

    #[test]
    fn release_rejects_oversized_binary_digests() {
        let mut attestation = sample_release();
        attestation.binary_digests = (0..257)
            .map(|i| BinaryDigest {
                name: alloc::format!("output-{i}"),
                algorithm: "blake3".to_string(),
                digest: "aa".repeat(32),
            })
            .collect();

        let err = release_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::CollectionTooLarge {
            limit: 256,
            actual: 257,
        });
    }

    #[test]
    fn detached_signature_round_trip() {
        let sig = DetachedSignature {
            key_name: "witness-a".to_string(),
            signature_bytes: [0xab; ED25519_SIGNATURE_BYTES],
        };
        let encoded = encode_detached_signature(sig.clone());
        assert!(encoded.starts_with("witness-a:"));

        let parsed = parse_detached_signature(encoded).unwrap();
        assert_eq!(parsed.key_name, sig.key_name);
        assert_eq!(parsed.signature_bytes, sig.signature_bytes);
    }

    #[test]
    fn detached_signature_rejects_missing_colon() {
        let err = parse_detached_signature("no-colon-here".to_string()).unwrap_err();
        assert!(matches!(err, Error::InvalidDetachedSignature { .. }));
    }

    #[test]
    fn detached_signature_rejects_empty_key_name() {
        let encoded = alloc::format!(":{}", data_encoding::BASE64.encode(&[0u8; 64]));
        let err = parse_detached_signature(encoded).unwrap_err();
        assert!(matches!(err, Error::InvalidDetachedSignature { .. }));
    }

    #[test]
    fn detached_signature_rejects_empty_payload() {
        let err = parse_detached_signature("key:".to_string()).unwrap_err();
        assert!(matches!(err, Error::InvalidDetachedSignature { .. }));
    }

    #[test]
    fn detached_signature_rejects_wrong_length() {
        let short = data_encoding::BASE64.encode(&[0u8; 32]);
        let err = parse_detached_signature(alloc::format!("key:{short}")).unwrap_err();
        assert!(matches!(err, Error::InvalidDetachedSignature { .. }));
    }

    #[test]
    fn technical_class_ordering_matches_spec() {
        assert!(TechnicalClass::BundleConsistent < TechnicalClass::SelfProofValid);
        assert!(TechnicalClass::SelfProofValid < TechnicalClass::ExternalWitnessMatch);
    }

    #[test]
    fn final_class_ordering_matches_spec() {
        assert!(FinalClass::BundleConsistent < FinalClass::SelfProofValid);
        assert!(FinalClass::SelfProofValid < FinalClass::ExternalWitnessMatch);
        assert!(FinalClass::ExternalWitnessMatch < FinalClass::QuorumSatisfied);
    }

    #[test]
    fn satisfied_policy_promotes_to_quorum_satisfied() {
        let result = FinalClass::resolve(TechnicalClass::SelfProofValid, PolicyStatus::Satisfied);
        assert_eq!(result, FinalClass::QuorumSatisfied);
    }

    #[test]
    fn insufficient_policy_mirrors_technical_class() {
        let result = FinalClass::resolve(TechnicalClass::ExternalWitnessMatch, PolicyStatus::Insufficient);
        assert_eq!(result, FinalClass::ExternalWitnessMatch);
    }

    #[test]
    fn not_evaluated_policy_mirrors_technical_class() {
        let result = FinalClass::resolve(TechnicalClass::SelfProofValid, PolicyStatus::NotEvaluated);
        assert_eq!(result, FinalClass::SelfProofValid);
    }

    #[test]
    fn trust_tier_separates_all_three_fields() {
        let tier = TrustTier {
            technical_class: TechnicalClass::ExternalWitnessMatch,
            policy_status: PolicyStatus::Insufficient,
            final_class: FinalClass::resolve(TechnicalClass::ExternalWitnessMatch, PolicyStatus::Insufficient),
        };
        assert_eq!(tier.technical_class, TechnicalClass::ExternalWitnessMatch);
        assert_eq!(tier.policy_status, PolicyStatus::Insufficient);
        assert_eq!(tier.final_class, FinalClass::ExternalWitnessMatch);
    }

    #[test]
    fn signature_suite_serializes_as_kebab() {
        let json = serde_json::to_string(&SignatureSuite::Ed25519DetachedV1).unwrap();
        assert_eq!(json, "\"ed25519-detached-v1\"");

        let parsed: SignatureSuite = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SignatureSuite::Ed25519DetachedV1);
    }

    #[test]
    fn technical_class_serializes_as_kebab() {
        let json = serde_json::to_string(&TechnicalClass::ExternalWitnessMatch).unwrap();
        assert_eq!(json, "\"external-witness-match\"");
    }

    #[test]
    fn final_class_serializes_as_kebab() {
        let json = serde_json::to_string(&FinalClass::QuorumSatisfied).unwrap();
        assert_eq!(json, "\"quorum-satisfied\"");
    }

    #[test]
    fn release_attestation_json_round_trip() {
        let original = sample_release();
        let bytes = release_attestation_canonical_bytes(original).unwrap();
        let parsed: ReleaseAttestation = serde_json::from_slice(&bytes).unwrap();
        let re_bytes = release_attestation_canonical_bytes(parsed).unwrap();
        assert_eq!(bytes, re_bytes);
    }

    #[test]
    fn witness_attestation_json_round_trip() {
        let original = sample_witness();
        let bytes = witness_attestation_canonical_bytes(original).unwrap();
        let parsed: WitnessAttestation = serde_json::from_slice(&bytes).unwrap();
        let re_bytes = witness_attestation_canonical_bytes(parsed).unwrap();
        assert_eq!(bytes, re_bytes);
    }

    #[test]
    fn release_canonical_json_has_correct_key_order() {
        let attestation = sample_release();
        let bytes = release_attestation_canonical_bytes(attestation).unwrap();
        let json = core::str::from_utf8(&bytes).unwrap();

        let schema_pos = json.find("\"schema\"").unwrap();
        let release_id_pos = json.find("\"release_id\"").unwrap();
        let manifest_pos = json.find("\"release_evidence_manifest_digest_blake3\"").unwrap();
        let proof_pos = json.find("\"proof_bundle_digest_blake3\"").unwrap();
        let mode_pos = json.find("\"proof_mode\"").unwrap();
        let workflow_pos = json.find("\"workflow\"").unwrap();
        let digests_pos = json.find("\"binary_digests\"").unwrap();

        assert!(schema_pos < release_id_pos);
        assert!(release_id_pos < manifest_pos);
        assert!(manifest_pos < proof_pos);
        assert!(proof_pos < mode_pos);
        assert!(mode_pos < workflow_pos);
        assert!(workflow_pos < digests_pos);
    }

    #[test]
    fn witness_canonical_json_has_correct_key_order() {
        let attestation = sample_witness();
        let bytes = witness_attestation_canonical_bytes(attestation).unwrap();
        let json = core::str::from_utf8(&bytes).unwrap();

        let schema_pos = json.find("\"schema\"").unwrap();
        let release_ref_pos = json.find("\"release_attestation_digest_blake3\"").unwrap();
        let identity_pos = json.find("\"witness_identity\"").unwrap();
        let suite_pos = json.find("\"signature_suite\"").unwrap();
        let rebuilt_pos = json.find("\"rebuilt_digests\"").unwrap();
        let env_pos = json.find("\"rebuild_environment_summary\"").unwrap();

        assert!(schema_pos < release_ref_pos);
        assert!(release_ref_pos < identity_pos);
        assert!(identity_pos < suite_pos);
        assert!(suite_pos < rebuilt_pos);
        assert!(rebuilt_pos < env_pos);
    }

    fn sample_release() -> ReleaseAttestation {
        ReleaseAttestation::new(ReleaseAttestationInit {
            release_id: "crunch-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: AttestationDigest::from_canonical_bytes(b"manifest"),
            proof_bundle_digest_blake3: AttestationDigest::from_canonical_bytes(b"proof"),
            proof_mode: "fixed-point".to_string(),
            workflow: Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            binary_digests: vec![BinaryDigest {
                name: "crunch".to_string(),
                algorithm: "blake3".to_string(),
                digest: "aa".repeat(32),
            }],
        })
    }

    fn sample_witness() -> WitnessAttestation {
        let release = sample_release();
        let release_digest = release_attestation_canonical_digest(release.clone()).unwrap();
        WitnessAttestation::new(
            release_digest,
            "witness-a".to_string(),
            release.binary_digests,
            RebuildEnvironmentSummary {
                system: "x86_64-linux".to_string(),
                toolchain: "rust-1.91.1".to_string(),
                host_class: "nixos-25.05".to_string(),
            },
        )
    }

    fn release_with_digests(digests: Vec<BinaryDigest>) -> ReleaseAttestation {
        ReleaseAttestation::new(ReleaseAttestationInit {
            release_id: "crunch-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: AttestationDigest::from_canonical_bytes(b"manifest"),
            proof_bundle_digest_blake3: AttestationDigest::from_canonical_bytes(b"proof"),
            proof_mode: "fixed-point".to_string(),
            workflow: Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            binary_digests: digests,
        })
    }
}
