use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::AttestationDigest;
use crate::Error;
use crate::canonical::to_canonical_bytes;

pub const RELEASE_ATTESTATION_SCHEMA: &str = "mantle-release-attestation-v1";
pub const WITNESS_ATTESTATION_SCHEMA: &str = "mantle-witness-attestation-v1";

const MAX_BINARY_DIGEST_COUNT: u32 = 256;
const MAX_ENV_FIELD_LEN: u32 = 256;
pub const WITNESS_SOURCE_ACQUISITION_MODE_UNSPECIFIED: &str = "unspecified";
pub const WITNESS_SOURCE_ACQUISITION_MODE_NOT_RECORDED: &str = "not-recorded";
const ED25519_SIGNATURE_BYTES: usize = 64;

const _: () = assert!(MAX_BINARY_DIGEST_COUNT >= 1, "binary digest limit must be positive");

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_effect_claims: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_effect_facts: Option<Vec<String>>,
    pub workflow: Workflow,
    pub binary_digests: Vec<BinaryDigest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAttestationInit {
    pub release_id: String,
    pub release_evidence_manifest_digest_blake3: AttestationDigest,
    pub proof_bundle_digest_blake3: AttestationDigest,
    pub proof_mode: String,
    pub declared_effect_claims: Option<Vec<String>>,
    pub observed_effect_facts: Option<Vec<String>>,
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
            declared_effect_claims: init.declared_effect_claims,
            observed_effect_facts: init.observed_effect_facts,
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
    let declared_effect_claims = normalize_optional_string_set(value.declared_effect_claims, "declared_effect_claims")?;
    let observed_effect_facts = normalize_optional_string_set(value.observed_effect_facts, "observed_effect_facts")?;

    Ok(ReleaseAttestation {
        schema: RELEASE_ATTESTATION_SCHEMA.to_string(),
        release_id: value.release_id,
        release_evidence_manifest_digest_blake3: value.release_evidence_manifest_digest_blake3,
        proof_bundle_digest_blake3: value.proof_bundle_digest_blake3,
        proof_mode: value.proof_mode,
        declared_effect_claims,
        observed_effect_facts,
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
    Ok(AttestationDigest::from_canonical_bytes(bytes))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WitnessAttestation {
    pub schema: String,
    pub release_attestation_digest_blake3: AttestationDigest,
    pub witness_identity: String,
    pub signature_suite: SignatureSuite,
    pub rebuilt_digests: Vec<BinaryDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_acquisition_mode: Option<String>,
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
            source_acquisition_mode: Some(WITNESS_SOURCE_ACQUISITION_MODE_UNSPECIFIED.to_string()),
            rebuild_environment_summary,
        }
    }

    pub fn with_source_acquisition_mode(mut self, mode: String) -> Self {
        self.source_acquisition_mode = Some(mode);
        self
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
    let source_acquisition_mode = normalize_optional_string(value.source_acquisition_mode, "source_acquisition_mode")?;

    Ok(WitnessAttestation {
        schema: WITNESS_ATTESTATION_SCHEMA.to_string(),
        release_attestation_digest_blake3: value.release_attestation_digest_blake3,
        witness_identity: value.witness_identity,
        signature_suite: value.signature_suite,
        rebuilt_digests,
        source_acquisition_mode,
        rebuild_environment_summary: value.rebuild_environment_summary,
    })
}

pub fn witness_attestation_canonical_bytes(value: WitnessAttestation) -> Result<Vec<u8>, Error> {
    let canonical = canonical_witness_attestation(value)?;
    to_canonical_bytes(&canonical)
}

pub fn witness_attestation_canonical_digest(value: WitnessAttestation) -> Result<AttestationDigest, Error> {
    let bytes = witness_attestation_canonical_bytes(value)?;
    Ok(AttestationDigest::from_canonical_bytes(bytes))
}

pub const INDEPENDENT_AGREEMENT_REPORT_SCHEMA: &str = "mantle-independent-agreement-report-v1";
const MAX_AGREEMENT_WITNESS_COUNT: u32 = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndependentAgreementStatus {
    Satisfied,
    Unsatisfied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WitnessClassificationReason {
    Counted,
    UnknownKey,
    InvalidSignature,
    Revoked,
    DigestMismatch,
    DuplicateIndependenceDomain,
    MissingIndependenceEvidence,
    MalformedEnvironmentEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgreementWitnessClassification {
    pub witness_identity: String,
    pub signer_key_name: String,
    pub witness_digest_blake3: AttestationDigest,
    pub release_attestation_digest_blake3: AttestationDigest,
    pub signature_valid: bool,
    pub digest_match: bool,
    pub independence_domain: String,
    pub source_acquisition_mode: String,
    pub policy_counted: bool,
    pub classification_reason: WitnessClassificationReason,
    pub rebuilt_output_digests: Vec<BinaryDigest>,
    pub environment_summary: RebuildEnvironmentSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndependentAgreementReport {
    pub schema: String,
    pub release_attestation_digest_blake3: AttestationDigest,
    pub policy_digest_blake3: AttestationDigest,
    pub independence_selector: String,
    pub required_witness_count: u32,
    pub counted_witness_count: u32,
    pub skipped_witness_count: u32,
    pub failed_witness_count: u32,
    pub witnesses: Vec<AgreementWitnessClassification>,
    pub artifact_digest_sets: Vec<BinaryDigest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndependentAgreementReportInit {
    pub release_attestation_digest_blake3: AttestationDigest,
    pub policy_digest_blake3: AttestationDigest,
    pub independence_selector: String,
    pub required_witness_count: u32,
    pub witnesses: Vec<AgreementWitnessClassification>,
    pub artifact_digest_sets: Vec<BinaryDigest>,
}

impl IndependentAgreementReport {
    pub fn new(init: IndependentAgreementReportInit) -> Result<Self, Error> {
        let witnesses = normalize_agreement_witnesses(init.witnesses)?;
        let counted_witness_count = count_witnesses(&witnesses, WitnessCountKind::Counted);
        let skipped_witness_count = count_witnesses(&witnesses, WitnessCountKind::Skipped);
        let failed_witness_count = count_witnesses(&witnesses, WitnessCountKind::Failed);
        let artifact_digest_sets = normalize_binary_digests(init.artifact_digest_sets)?;
        validate_non_empty(NamedField {
            name: "independence_selector",
            value: &init.independence_selector,
        })?;
        Ok(Self {
            schema: INDEPENDENT_AGREEMENT_REPORT_SCHEMA.to_string(),
            release_attestation_digest_blake3: init.release_attestation_digest_blake3,
            policy_digest_blake3: init.policy_digest_blake3,
            independence_selector: init.independence_selector,
            required_witness_count: init.required_witness_count,
            counted_witness_count,
            skipped_witness_count,
            failed_witness_count,
            witnesses,
            artifact_digest_sets,
        })
    }

    pub fn status(&self) -> IndependentAgreementStatus {
        if self.counted_witness_count >= self.required_witness_count {
            IndependentAgreementStatus::Satisfied
        } else {
            IndependentAgreementStatus::Unsatisfied
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WitnessCountKind {
    Counted,
    Skipped,
    Failed,
}

pub fn independent_agreement_report_canonical_bytes(value: IndependentAgreementReport) -> Result<Vec<u8>, Error> {
    let canonical = canonical_independent_agreement_report(value)?;
    to_canonical_bytes(&canonical)
}

pub fn independent_agreement_report_canonical_digest(
    value: IndependentAgreementReport,
) -> Result<AttestationDigest, Error> {
    let bytes = independent_agreement_report_canonical_bytes(value)?;
    Ok(AttestationDigest::from_canonical_bytes(bytes))
}

pub fn canonical_independent_agreement_report(
    value: IndependentAgreementReport,
) -> Result<IndependentAgreementReport, Error> {
    validate_schema_tag(SchemaTag {
        actual: &value.schema,
        expected: INDEPENDENT_AGREEMENT_REPORT_SCHEMA,
    })?;
    IndependentAgreementReport::new(IndependentAgreementReportInit {
        release_attestation_digest_blake3: value.release_attestation_digest_blake3,
        policy_digest_blake3: value.policy_digest_blake3,
        independence_selector: value.independence_selector,
        required_witness_count: value.required_witness_count,
        witnesses: value.witnesses,
        artifact_digest_sets: value.artifact_digest_sets,
    })
}

fn normalize_agreement_witnesses(
    witnesses: Vec<AgreementWitnessClassification>,
) -> Result<Vec<AgreementWitnessClassification>, Error> {
    let actual = count_with_overflow_marker(witnesses.len(), MAX_AGREEMENT_WITNESS_COUNT);
    if actual > MAX_AGREEMENT_WITNESS_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_AGREEMENT_WITNESS_COUNT,
            actual,
        });
    }
    let mut normalized = Vec::with_capacity(witnesses.len());
    for witness in witnesses {
        normalized.push(normalize_agreement_witness(witness)?);
    }
    normalized.sort_by(|left, right| {
        (
            &left.witness_identity,
            &left.signer_key_name,
            &left.independence_domain,
            &left.witness_digest_blake3,
        )
            .cmp(&(
                &right.witness_identity,
                &right.signer_key_name,
                &right.independence_domain,
                &right.witness_digest_blake3,
            ))
    });
    Ok(normalized)
}

fn normalize_agreement_witness(
    mut witness: AgreementWitnessClassification,
) -> Result<AgreementWitnessClassification, Error> {
    validate_non_empty(NamedField {
        name: "witness_identity",
        value: &witness.witness_identity,
    })?;
    validate_non_empty(NamedField {
        name: "signer_key_name",
        value: &witness.signer_key_name,
    })?;
    if witness.classification_reason == WitnessClassificationReason::Counted {
        validate_non_empty(NamedField {
            name: "independence_domain",
            value: &witness.independence_domain,
        })?;
        validate_env_field(NamedField {
            name: "source_acquisition_mode",
            value: &witness.source_acquisition_mode,
        })?;
    }
    if witness.classification_reason != WitnessClassificationReason::MalformedEnvironmentEvidence {
        validate_env_field(NamedField {
            name: "system",
            value: &witness.environment_summary.system,
        })?;
        validate_env_field(NamedField {
            name: "toolchain",
            value: &witness.environment_summary.toolchain,
        })?;
        validate_env_field(NamedField {
            name: "host_class",
            value: &witness.environment_summary.host_class,
        })?;
    }
    witness.rebuilt_output_digests = normalize_binary_digests(witness.rebuilt_output_digests)?;
    Ok(witness)
}

fn count_witnesses(witnesses: &[AgreementWitnessClassification], kind: WitnessCountKind) -> u32 {
    let count = witnesses.iter().filter(|witness| witness_matches_count_kind(witness, kind)).count();
    count_with_overflow_marker(count, MAX_AGREEMENT_WITNESS_COUNT)
}

fn witness_matches_count_kind(witness: &AgreementWitnessClassification, kind: WitnessCountKind) -> bool {
    match kind {
        WitnessCountKind::Counted => witness.policy_counted,
        WitnessCountKind::Skipped => {
            !witness.policy_counted
                && matches!(
                    witness.classification_reason,
                    WitnessClassificationReason::UnknownKey
                        | WitnessClassificationReason::InvalidSignature
                        | WitnessClassificationReason::Revoked
                        | WitnessClassificationReason::DuplicateIndependenceDomain
                        | WitnessClassificationReason::MissingIndependenceEvidence
                )
        }
        WitnessCountKind::Failed => {
            !witness.policy_counted
                && matches!(
                    witness.classification_reason,
                    WitnessClassificationReason::DigestMismatch
                        | WitnessClassificationReason::MalformedEnvironmentEvidence
                )
        }
    }
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

fn normalize_optional_string_set(
    values: Option<Vec<String>>,
    field: &'static str,
) -> Result<Option<Vec<String>>, Error> {
    let Some(mut values) = values else {
        return Ok(None);
    };
    for value in &values {
        validate_non_empty(NamedField { name: field, value })?;
    }
    values.sort();
    values.dedup();
    Ok(Some(values))
}

fn normalize_optional_string(value: Option<String>, field: &'static str) -> Result<Option<String>, Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    validate_env_field(NamedField {
        name: field,
        value: &value,
    })?;
    Ok(Some(value))
}

fn normalize_binary_digests(digests: Vec<BinaryDigest>) -> Result<Vec<BinaryDigest>, Error> {
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
        expected: tag.expected.to_string(),
        actual: tag.actual.to_string(),
    })
}

fn validate_non_empty(field: NamedField<'_>) -> Result<(), Error> {
    if !field.value.is_empty() {
        return Ok(());
    }
    Err(Error::EmptyField {
        field: field.name.to_string(),
    })
}

fn validate_env_field(field: NamedField<'_>) -> Result<(), Error> {
    validate_non_empty(NamedField {
        name: field.name,
        value: field.value,
    })?;
    let actual = count_with_overflow_marker(field.value.len(), MAX_ENV_FIELD_LEN);
    if actual > MAX_ENV_FIELD_LEN {
        return Err(Error::FieldTooLong {
            field: field.name.to_string(),
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
    fn release_attestation_effect_claims_and_facts_affect_digest() {
        let mut with_effects = sample_release();
        with_effects.declared_effect_claims = Some(vec!["write-output".to_string(), "read-store".to_string()]);
        with_effects.observed_effect_facts = Some(vec!["read-store".to_string(), "write-output".to_string()]);

        let canonical = canonical_release_attestation(with_effects.clone()).unwrap();
        assert_eq!(canonical.declared_effect_claims, Some(vec!["read-store".to_string(), "write-output".to_string()]));
        assert_ne!(
            release_attestation_canonical_digest(sample_release()).unwrap(),
            release_attestation_canonical_digest(with_effects).unwrap()
        );
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
        let manifest_digest = AttestationDigest::from_canonical_bytes(b"manifest-content".to_vec());
        let proof_digest = AttestationDigest::from_canonical_bytes(b"proof-bundle".to_vec());
        let attestation = ReleaseAttestation::new(ReleaseAttestationInit {
            release_id: "mantle-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: manifest_digest,
            proof_bundle_digest_blake3: proof_digest,
            proof_mode: "fixed-point".to_string(),
            declared_effect_claims: None,
            observed_effect_facts: None,
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
            expected: RELEASE_ATTESTATION_SCHEMA.to_string(),
            actual: "wrong-schema".to_string(),
        });
    }

    #[test]
    fn witness_rejects_wrong_schema_tag() {
        let mut attestation = sample_witness();
        attestation.schema = "wrong-schema".to_string();

        let err = witness_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::SchemaTagMismatch {
            expected: WITNESS_ATTESTATION_SCHEMA.to_string(),
            actual: "wrong-schema".to_string(),
        });
    }

    #[test]
    fn release_rejects_empty_release_id() {
        let mut attestation = sample_release();
        attestation.release_id.clear();

        let err = release_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::EmptyField {
            field: "release_id".to_string(),
        });
    }

    #[test]
    fn witness_rejects_empty_witness_identity() {
        let mut attestation = sample_witness();
        attestation.witness_identity.clear();

        let err = witness_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::EmptyField {
            field: "witness_identity".to_string()
        });
    }

    #[test]
    fn release_rejects_empty_binary_digest_name() {
        let mut attestation = sample_release();
        attestation.binary_digests[0].name.clear();

        let err = release_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::EmptyField {
            field: "binary_digest.name".to_string()
        });
    }

    #[test]
    fn witness_rejects_oversized_env_field() {
        let mut attestation = sample_witness();
        attestation.rebuild_environment_summary.system = "x".repeat(257);

        let err = witness_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::FieldTooLong {
            field: "system".to_string(),
            limit: 256,
            actual: 257,
        });
    }

    #[test]
    fn witness_accepts_legacy_payload_without_source_mode() {
        let mut attestation = sample_witness();
        attestation.source_acquisition_mode = None;

        let bytes = witness_attestation_canonical_bytes(attestation).unwrap();
        let json = core::str::from_utf8(&bytes).unwrap();

        assert!(!json.contains("source_acquisition_mode"));
        assert!(json.contains("rebuild_environment_summary"));
    }

    #[test]
    fn witness_rejects_oversized_source_acquisition_mode() {
        let mut attestation = sample_witness();
        attestation.source_acquisition_mode = Some("x".repeat(257));

        let err = witness_attestation_canonical_bytes(attestation).unwrap_err();
        assert_eq!(err, Error::FieldTooLong {
            field: "source_acquisition_mode".to_string(),
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
        let source_mode_pos = json.find("\"source_acquisition_mode\"").unwrap();
        let env_pos = json.find("\"rebuild_environment_summary\"").unwrap();

        assert!(schema_pos < release_ref_pos);
        assert!(release_ref_pos < identity_pos);
        assert!(identity_pos < suite_pos);
        assert!(suite_pos < rebuilt_pos);
        assert!(rebuilt_pos < source_mode_pos);
        assert!(source_mode_pos < env_pos);
    }

    #[test]
    fn independent_agreement_report_digest_is_stable_across_witness_order() {
        let report_a = sample_agreement_report(vec![
            sample_agreement_witness("witness-b", "key-b", "host-b", WitnessClassificationReason::Counted, true),
            sample_agreement_witness("witness-a", "key-a", "host-a", WitnessClassificationReason::Counted, true),
        ]);
        let report_b = sample_agreement_report(vec![
            sample_agreement_witness("witness-a", "key-a", "host-a", WitnessClassificationReason::Counted, true),
            sample_agreement_witness("witness-b", "key-b", "host-b", WitnessClassificationReason::Counted, true),
        ]);

        let bytes_a = independent_agreement_report_canonical_bytes(report_a.clone()).unwrap();
        let bytes_b = independent_agreement_report_canonical_bytes(report_b.clone()).unwrap();
        assert_eq!(bytes_a, bytes_b);

        let digest_a = independent_agreement_report_canonical_digest(report_a).unwrap();
        let digest_b = independent_agreement_report_canonical_digest(report_b).unwrap();
        assert_eq!(digest_a, digest_b);
    }

    #[test]
    fn independent_agreement_report_counts_skipped_and_failed_witnesses() {
        let report = sample_agreement_report(vec![
            sample_agreement_witness("witness-a", "key-a", "host-a", WitnessClassificationReason::Counted, true),
            sample_agreement_witness("witness-b", "key-b", "host-b", WitnessClassificationReason::UnknownKey, false),
            sample_agreement_witness(
                "witness-c",
                "key-c",
                "host-c",
                WitnessClassificationReason::DigestMismatch,
                false,
            ),
        ]);

        assert_eq!(report.counted_witness_count, 1);
        assert_eq!(report.skipped_witness_count, 1);
        assert_eq!(report.failed_witness_count, 1);
        assert_eq!(report.status(), IndependentAgreementStatus::Unsatisfied);
    }

    #[test]
    fn independent_agreement_report_classifies_all_skip_reasons() {
        let report = sample_agreement_report(vec![
            sample_agreement_witness(
                "witness-a",
                "key-a",
                "host-a",
                WitnessClassificationReason::InvalidSignature,
                false,
            ),
            sample_agreement_witness("witness-b", "key-b", "host-b", WitnessClassificationReason::Revoked, false),
            sample_agreement_witness(
                "witness-c",
                "key-c",
                "host-c",
                WitnessClassificationReason::DuplicateIndependenceDomain,
                false,
            ),
            sample_agreement_witness(
                "witness-d",
                "key-d",
                "host-d",
                WitnessClassificationReason::MissingIndependenceEvidence,
                false,
            ),
            sample_agreement_witness(
                "witness-e",
                "key-e",
                "",
                WitnessClassificationReason::MalformedEnvironmentEvidence,
                false,
            ),
        ]);

        assert_eq!(report.counted_witness_count, 0);
        assert_eq!(report.skipped_witness_count, 4);
        assert_eq!(report.failed_witness_count, 1);
        assert_eq!(report.status(), IndependentAgreementStatus::Unsatisfied);
    }

    #[test]
    fn independent_agreement_report_requires_counted_domain() {
        let err = IndependentAgreementReport::new(IndependentAgreementReportInit {
            release_attestation_digest_blake3: release_attestation_canonical_digest(sample_release()).unwrap(),
            policy_digest_blake3: AttestationDigest::from_canonical_bytes(b"policy".to_vec()),
            independence_selector: "witness_identity".to_string(),
            required_witness_count: 1,
            witnesses: vec![sample_agreement_witness(
                "witness-a",
                "key-a",
                "",
                WitnessClassificationReason::Counted,
                true,
            )],
            artifact_digest_sets: sample_release().binary_digests,
        })
        .unwrap_err();
        assert_eq!(err, Error::EmptyField {
            field: "independence_domain".to_string()
        });
    }

    fn sample_agreement_report(witnesses: Vec<AgreementWitnessClassification>) -> IndependentAgreementReport {
        IndependentAgreementReport::new(IndependentAgreementReportInit {
            release_attestation_digest_blake3: release_attestation_canonical_digest(sample_release()).unwrap(),
            policy_digest_blake3: AttestationDigest::from_canonical_bytes(b"policy".to_vec()),
            independence_selector: "witness_identity".to_string(),
            required_witness_count: 2,
            witnesses,
            artifact_digest_sets: sample_release().binary_digests,
        })
        .unwrap()
    }

    fn sample_agreement_witness(
        witness_identity: &str,
        signer_key_name: &str,
        independence_domain: &str,
        classification_reason: WitnessClassificationReason,
        policy_counted: bool,
    ) -> AgreementWitnessClassification {
        AgreementWitnessClassification {
            witness_identity: witness_identity.to_string(),
            signer_key_name: signer_key_name.to_string(),
            witness_digest_blake3: AttestationDigest::from_canonical_bytes(witness_identity.as_bytes().to_vec()),
            release_attestation_digest_blake3: release_attestation_canonical_digest(sample_release()).unwrap(),
            signature_valid: classification_reason != WitnessClassificationReason::InvalidSignature,
            digest_match: classification_reason != WitnessClassificationReason::DigestMismatch,
            independence_domain: independence_domain.to_string(),
            source_acquisition_mode: WITNESS_SOURCE_ACQUISITION_MODE_UNSPECIFIED.to_string(),
            policy_counted,
            classification_reason,
            rebuilt_output_digests: sample_release().binary_digests,
            environment_summary: RebuildEnvironmentSummary {
                system: "x86_64-linux".to_string(),
                toolchain: "rust-1.91.1".to_string(),
                host_class: independence_domain.to_string(),
            },
        }
    }

    fn sample_release() -> ReleaseAttestation {
        ReleaseAttestation::new(ReleaseAttestationInit {
            release_id: "mantle-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: AttestationDigest::from_canonical_bytes(b"manifest".to_vec()),
            proof_bundle_digest_blake3: AttestationDigest::from_canonical_bytes(b"proof".to_vec()),
            proof_mode: "fixed-point".to_string(),
            declared_effect_claims: None,
            observed_effect_facts: None,
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
            release_id: "mantle-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: AttestationDigest::from_canonical_bytes(b"manifest".to_vec()),
            proof_bundle_digest_blake3: AttestationDigest::from_canonical_bytes(b"proof".to_vec()),
            proof_mode: "fixed-point".to_string(),
            declared_effect_claims: None,
            observed_effect_facts: None,
            workflow: Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            binary_digests: digests,
        })
    }
}
