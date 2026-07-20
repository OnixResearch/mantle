//! Exact standalone artifact-auth signing and verification for Mantle action results.
//!
//! The pure statement mapping remains in `crunch-action-result-core`. This
//! module composes that mapping with Mantle's existing Nix-compatible Ed25519
//! key adapter without changing action-result or release authority.

use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use artifact_auth_core::KeyCurrentness;
use crunch_action_result_core::CandidateDecision;
use crunch_action_result_core::SignedActionResultRecord;
use crunch_action_result_core::artifact_auth::MantleArtifactAuthObservation;
use crunch_action_result_core::artifact_auth::MantleArtifactAuthReport;
use crunch_action_result_core::artifact_auth::MantleArtifactAuthStatementInput;
use crunch_action_result_core::artifact_auth::MantleSignerObservation;
use crunch_action_result_core::artifact_auth::evaluate_mantle_artifact_auth;
use crunch_action_result_core::artifact_auth::map_mantle_artifact_auth_statement;
use data_encoding::BASE64;
use nix_compat::narinfo::Signature;
use nix_compat::narinfo::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use crate::KeyPair;

const BLAKE3_REF_PREFIX: &str = "blake3:";
const BLAKE3_HEX_CHARS: usize = 64;
const HEX_DIGIT_COUNT: usize = 16;
const HEX_DIGITS: &[u8; HEX_DIGIT_COUNT] = b"0123456789abcdef";
const BITS_PER_NIBBLE: u32 = 4;
const LOW_NIBBLE_MASK: u8 = 0x0f;
const HEX_CHARS_PER_BYTE: usize = 2;
const PUBLIC_KEY_DECODE_BUFFER_BYTES: usize = artifact_auth_ed25519::ED25519_PUBLIC_KEY_BYTES.saturating_add(1);
const STANDALONE_THRESHOLD: u16 = 1;
const OPERATIONAL_RECEIPT_SCHEMA: &str = "mantle-artifact-auth-operational-receipt-v1";
const OPERATIONAL_RECEIPT_CONSUMER: &str = "mantle";
const OPERATIONAL_RECEIPT_DIR: &str = "artifact-auth";
const OPERATIONAL_RECEIPT_EXTENSION: &str = "json";
const OPERATIONAL_RECEIPT_BYTES_MAX: u64 = 1_048_576;
const NIX_KEY_GENERATION_SEPARATOR: char = '-';
const OPERATIONAL_RECEIPT_NON_CLAIM: &str = "persisted replay proves exact standalone bytes under one explicit Mantle trust snapshot only; it does not provide remote trust discovery, revocation freshness, cache or build admission, registry publication, release eligibility, or standalone authority";

#[derive(Debug, Clone, Copy)]
pub struct MantleArtifactAuthShellInput<'a> {
    pub statement: MantleArtifactAuthStatementInput<'a>,
    pub signed_record: &'a SignedActionResultRecord,
    pub legacy: &'a CandidateDecision,
    pub generation: u64,
    pub currentness: KeyCurrentness,
    pub currentness_blake3: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedMantleArtifactAuthStatement {
    pub statement_ref: String,
    pub public_key: String,
    pub public_key_ref: String,
    pub signature_ref: String,
    pub signature_hex: String,
    pub signing_authorization_ref: String,
    pub signature_bytes: Vec<u8>,
}

#[cfg(test)]
impl SignedMantleArtifactAuthStatement {
    fn replace_signature_bytes_for_test(&mut self, signature_bytes: Vec<u8>) {
        self.signature_ref = content_ref(&signature_bytes);
        self.signature_hex = bytes_to_lower_hex(&signature_bytes);
        self.signature_bytes = signature_bytes;
    }

    fn replace_public_key_for_test(&mut self, public_key: String) {
        let (_, public_key_bytes) = decode_public_key(&public_key).expect("test public key");
        self.public_key_ref = content_ref(&public_key_bytes);
        self.public_key = public_key;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MantleArtifactAuthShellReport {
    pub statement_ref: String,
    pub public_key_ref: String,
    pub signature_ref: String,
    pub signature_hex: String,
    pub signing_authorization_observation_ref: String,
    pub legacy_signature_authorized: bool,
    pub currentness_ref: String,
    pub cryptographic_failure_code: Option<String>,
    pub dual_run: MantleArtifactAuthReport,
}

#[derive(Debug, Clone, Copy)]
pub struct MantleArtifactAuthTrustSnapshot<'a> {
    pub policy_hash: &'a str,
    pub expected_policy_hash: &'a str,
    pub valid_after_unix_s: u64,
    pub valid_before_unix_s: u64,
    pub valid_at_unix_s: u64,
    pub revocation_ref: Option<&'a str>,
    pub expected_revocation_ref: Option<&'a str>,
    pub trusted_public_keys: &'a [VerifyingKey],
    pub revoked_public_key_digests: &'a BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MantleArtifactAuthTrustObservation {
    pub generation: u64,
    pub currentness: KeyCurrentness,
    pub currentness_blake3: String,
    pub key_token_blake3: String,
}

#[derive(Debug, Clone, Copy)]
pub struct MantleArtifactAuthOperationalInput<'a> {
    pub statement: MantleArtifactAuthStatementInput<'a>,
    pub signed_record: &'a SignedActionResultRecord,
    pub legacy: &'a CandidateDecision,
    pub trust: MantleArtifactAuthTrustSnapshot<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MantleArtifactAuthOperationalReceipt {
    pub schema: String,
    pub consumer: String,
    pub statement_ref: String,
    pub public_key_ref: String,
    pub signature_ref: String,
    pub signing_authorization_ref: String,
    pub trust_context_ref: String,
    pub key_token_blake3: String,
    pub key_generation: u64,
    pub key_currentness: String,
    pub signed: SignedMantleArtifactAuthStatement,
    pub standalone_passed: bool,
    pub compatibility_case_explained: bool,
    pub legacy_authoritative: bool,
    pub standalone_authority_admitted: bool,
    pub rollback_available: bool,
    pub non_claim: String,
    pub receipt_blake3: String,
}

#[derive(Serialize)]
struct MantleArtifactAuthTrustMaterial<'a> {
    policy_hash: &'a str,
    expected_policy_hash: &'a str,
    valid_after_unix_s: u64,
    valid_before_unix_s: u64,
    valid_at_unix_s: u64,
    revocation_ref: Option<&'a str>,
    expected_revocation_ref: Option<&'a str>,
    trusted_public_keys: Vec<String>,
    revoked_public_key_digests: Vec<&'a str>,
    selected_key_token_blake3: &'a str,
    generation: u64,
    currentness: &'a str,
}

#[derive(Serialize)]
struct MantleArtifactAuthReceiptMaterial<'a> {
    schema: &'a str,
    consumer: &'a str,
    statement_ref: &'a str,
    public_key_ref: &'a str,
    signature_ref: &'a str,
    signing_authorization_ref: &'a str,
    trust_context_ref: &'a str,
    key_token_blake3: &'a str,
    key_generation: u64,
    key_currentness: &'a str,
    signed: &'a SignedMantleArtifactAuthStatement,
    standalone_passed: bool,
    compatibility_case_explained: bool,
    legacy_authoritative: bool,
    standalone_authority_admitted: bool,
    rollback_available: bool,
    non_claim: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MantleArtifactAuthShellError {
    #[error("standalone statement mapping rejected: {0}")]
    StatementMapping(String),
    #[error("standalone statement is not canonicalizable")]
    StatementInvalid,
    #[error("legacy action-result decision is not admitted")]
    LegacyDecisionRejected,
    #[error("legacy action-result decision does not match the signed record")]
    LegacyDecisionMismatch,
    #[error("standalone statement record does not match the signed action-result record")]
    SignedRecordMismatch,
    #[error("legacy action-result signer is not present in the admitted trust basis")]
    LegacyTrustBasisMissing,
    #[error("legacy action-result signature is missing or ambiguous")]
    LegacySignatureMissingOrAmbiguous,
    #[error("legacy action-result signature is invalid")]
    LegacySignatureInvalid,
    #[error("standalone signer does not match the action-result producer and key")]
    SignerMismatch,
    #[error("standalone signing requires a current key")]
    CurrentnessNotCurrent,
    #[error("standalone currentness identity is malformed")]
    CurrentnessMalformed,
    #[error("standalone public key carrier is malformed")]
    PublicKeyMalformed,
    #[error("standalone public key label does not match the statement key")]
    PublicKeyLabelMismatch,
    #[error("standalone full public-key identity does not match the statement")]
    PublicKeyIdentityMismatch,
    #[error("artifact-auth carrier {0} identity mismatch")]
    CarrierIdentityMismatch(&'static str),
    #[error("standalone trust snapshot is invalid: {0}")]
    TrustSnapshotInvalid(&'static str),
    #[error("artifact-auth operational receipt is invalid: {0}")]
    OperationalReceiptInvalid(&'static str),
    #[error("artifact-auth operational receipt I/O failed: {0}")]
    OperationalIo(String),
}

// r[impl mantle.artifact_auth_shell.exact_verification]
// r[impl mantle.artifact_auth_shell.authorization]
// r[impl mantle.artifact_auth_shell.evidence]
pub fn sign_mantle_artifact_auth_statement(
    keypair: &KeyPair,
    input: &MantleArtifactAuthShellInput<'_>,
) -> Result<SignedMantleArtifactAuthStatement, MantleArtifactAuthShellError> {
    let authorization = admit_signing_input(keypair, input)?;
    let statement = map_statement(&input.statement)?;
    let statement_bytes = artifact_auth_core::canonical_statement_bytes(&statement)
        .map_err(|_| MantleArtifactAuthShellError::StatementInvalid)?;
    let signature = keypair.signing_key.sign(&statement_bytes).to_owned();
    let signature_bytes = signature.bytes().to_vec();
    let public_key = keypair.verifying_key.to_string();
    let (_, public_key_bytes) = decode_public_key(&public_key)?;
    let signed = SignedMantleArtifactAuthStatement {
        statement_ref: content_ref(&statement_bytes),
        public_key,
        public_key_ref: content_ref(&public_key_bytes),
        signature_ref: content_ref(&signature_bytes),
        signature_hex: bytes_to_lower_hex(&signature_bytes),
        signing_authorization_ref: authorization,
        signature_bytes,
    };
    debug_assert_eq!(signed.signature_bytes.len(), artifact_auth_ed25519::ED25519_SIGNATURE_BYTES);
    debug_assert_eq!(signed.public_key_ref, format!("{BLAKE3_REF_PREFIX}{}", statement.key_identity.digest_hex));
    Ok(signed)
}

// r[impl mantle.artifact_auth_shell.exact_verification]
// r[impl mantle.artifact_auth_shell.evidence]
// r[impl mantle.artifact_auth_shell.authority]
pub fn evaluate_mantle_artifact_auth_shell(
    input: &MantleArtifactAuthShellInput<'_>,
    signed: &SignedMantleArtifactAuthStatement,
) -> Result<MantleArtifactAuthShellReport, MantleArtifactAuthShellError> {
    validate_common_input(input)?;
    let statement = map_statement(&input.statement)?;
    let statement_bytes = artifact_auth_core::canonical_statement_bytes(&statement)
        .map_err(|_| MantleArtifactAuthShellError::StatementInvalid)?;
    let carrier = validate_carrier(input, signed, &statement, &statement_bytes)?;
    if !carrier.is_legacy_signature_authorized {
        return Err(MantleArtifactAuthShellError::LegacySignatureInvalid);
    }
    let cryptographic =
        artifact_auth_ed25519::verify_statement(&statement, &carrier.public_key_bytes, &signed.signature_bytes);
    let cryptographic_failure_code = cryptographic.failure_code.clone();
    let signers = vec![MantleSignerObservation {
        producer_id: input.statement.producer_id.to_string(),
        key_id: input.statement.key_id.to_string(),
        key_identity_blake3: input.statement.key_identity_blake3.to_string(),
        generation: input.generation,
        currentness: input.currentness,
        currentness_blake3: input.currentness_blake3.to_string(),
        standalone_cryptographic: cryptographic,
    }];
    let required_signer_labels = vec![input.statement.key_id.to_string()];
    let dual_run = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
        profile_id: input.statement.profile_id,
        record: input.statement.record,
        legacy: input.legacy,
        signers: &signers,
        required_signer_labels: &required_signer_labels,
        threshold: STANDALONE_THRESHOLD,
        oci_manifest_sha256: input.statement.oci_manifest_sha256,
        metadata_manifest_sha256: input.statement.metadata_manifest_sha256,
    });
    let evidence = MantleArtifactAuthShellReport {
        statement_ref: signed.statement_ref.clone(),
        public_key_ref: signed.public_key_ref.clone(),
        signature_ref: signed.signature_ref.clone(),
        signature_hex: signed.signature_hex.clone(),
        signing_authorization_observation_ref: signed.signing_authorization_ref.clone(),
        legacy_signature_authorized: carrier.is_legacy_signature_authorized,
        currentness_ref: carrier.currentness_ref,
        cryptographic_failure_code,
        dual_run,
    };
    debug_assert!(evidence.dual_run.compatibility.legacy_authoritative);
    debug_assert!(!evidence.dual_run.compatibility.standalone_authority_admitted);
    Ok(evidence)
}

// r[impl mantle.artifact_auth_operational_receipt.trust]
pub fn derive_mantle_artifact_auth_trust(
    keypair: &KeyPair,
    snapshot: &MantleArtifactAuthTrustSnapshot<'_>,
) -> Result<MantleArtifactAuthTrustObservation, MantleArtifactAuthShellError> {
    validate_trust_snapshot(snapshot)?;
    let key_token = keypair.verifying_key.to_string();
    let generation = nix_key_generation(&key_token)?;
    let key_token_blake3 = blake3::hash(key_token.as_bytes()).to_hex().to_string();
    let currentness = if snapshot.revoked_public_key_digests.contains(&key_token_blake3) {
        KeyCurrentness::Revoked
    } else if snapshot.trusted_public_keys.iter().any(|trusted| trusted.to_string() == key_token) {
        KeyCurrentness::Current
    } else {
        KeyCurrentness::Unknown
    };
    let trusted_public_keys = snapshot.trusted_public_keys.iter().map(ToString::to_string).collect::<Vec<_>>();
    let revoked_public_key_digests = snapshot.revoked_public_key_digests.iter().map(String::as_str).collect::<Vec<_>>();
    let material = MantleArtifactAuthTrustMaterial {
        policy_hash: snapshot.policy_hash,
        expected_policy_hash: snapshot.expected_policy_hash,
        valid_after_unix_s: snapshot.valid_after_unix_s,
        valid_before_unix_s: snapshot.valid_before_unix_s,
        valid_at_unix_s: snapshot.valid_at_unix_s,
        revocation_ref: snapshot.revocation_ref,
        expected_revocation_ref: snapshot.expected_revocation_ref,
        trusted_public_keys,
        revoked_public_key_digests,
        selected_key_token_blake3: &key_token_blake3,
        generation,
        currentness: key_currentness_label(currentness),
    };
    let bytes = serde_json::to_vec(&material)
        .map_err(|error| MantleArtifactAuthShellError::OperationalIo(error.to_string()))?;
    let currentness_blake3 = blake3::hash(&bytes).to_hex().to_string();
    debug_assert!(valid_digest(&key_token_blake3));
    debug_assert!(valid_digest(&currentness_blake3));
    Ok(MantleArtifactAuthTrustObservation {
        generation,
        currentness,
        currentness_blake3,
        key_token_blake3,
    })
}

// r[impl mantle.artifact_auth_operational_receipt.persistence]
pub fn capture_mantle_artifact_auth_operational_receipt(
    state_dir: &Path,
    keypair: &KeyPair,
    input: &MantleArtifactAuthOperationalInput<'_>,
) -> Result<MantleArtifactAuthOperationalReceipt, MantleArtifactAuthShellError> {
    let trust = derive_mantle_artifact_auth_trust(keypair, &input.trust)?;
    if trust.currentness != KeyCurrentness::Current {
        return Err(MantleArtifactAuthShellError::CurrentnessNotCurrent);
    }
    let shell_input = operational_shell_input(input, &trust);
    let signed = sign_mantle_artifact_auth_statement(keypair, &shell_input)?;
    let shell_evidence = evaluate_mantle_artifact_auth_shell(&shell_input, &signed)?;
    let receipt = build_mantle_artifact_auth_operational_receipt(&shell_input, &trust, &signed, &shell_evidence)?;
    write_mantle_artifact_auth_operational_receipt(state_dir, &receipt)?;
    Ok(receipt)
}

// r[impl mantle.artifact_auth_operational_receipt.replay]
pub fn replay_mantle_artifact_auth_operational_receipt(
    state_dir: &Path,
    keypair: &KeyPair,
    input: &MantleArtifactAuthOperationalInput<'_>,
    statement_ref: &str,
) -> Result<MantleArtifactAuthShellReport, MantleArtifactAuthShellError> {
    let receipt = read_mantle_artifact_auth_operational_receipt(state_dir, statement_ref)?;
    let trust = derive_mantle_artifact_auth_trust(keypair, &input.trust)?;
    if trust.currentness != KeyCurrentness::Current {
        return Err(MantleArtifactAuthShellError::CurrentnessNotCurrent);
    }
    let shell_input = operational_shell_input(input, &trust);
    let shell_evidence = evaluate_mantle_artifact_auth_shell(&shell_input, &receipt.signed)?;
    let expected =
        build_mantle_artifact_auth_operational_receipt(&shell_input, &trust, &receipt.signed, &shell_evidence)?;
    if expected != receipt {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("fresh replay drift"));
    }
    debug_assert!(shell_evidence.dual_run.compatibility.legacy_authoritative);
    debug_assert!(!shell_evidence.dual_run.compatibility.standalone_authority_admitted);
    Ok(shell_evidence)
}

fn operational_shell_input<'a>(
    input: &'a MantleArtifactAuthOperationalInput<'a>,
    trust: &'a MantleArtifactAuthTrustObservation,
) -> MantleArtifactAuthShellInput<'a> {
    MantleArtifactAuthShellInput {
        statement: input.statement,
        signed_record: input.signed_record,
        legacy: input.legacy,
        generation: trust.generation,
        currentness: trust.currentness,
        currentness_blake3: &trust.currentness_blake3,
    }
}

// r[impl mantle.artifact_auth_operational_receipt.trust]
fn validate_trust_snapshot(snapshot: &MantleArtifactAuthTrustSnapshot<'_>) -> Result<(), MantleArtifactAuthShellError> {
    if snapshot.policy_hash.is_empty() || snapshot.policy_hash != snapshot.expected_policy_hash {
        return Err(MantleArtifactAuthShellError::TrustSnapshotInvalid("policy identity mismatch"));
    }
    if snapshot.valid_after_unix_s >= snapshot.valid_before_unix_s
        || snapshot.valid_at_unix_s < snapshot.valid_after_unix_s
        || snapshot.valid_at_unix_s >= snapshot.valid_before_unix_s
    {
        return Err(MantleArtifactAuthShellError::TrustSnapshotInvalid("validity window mismatch"));
    }
    if snapshot.expected_revocation_ref.is_some() && snapshot.revocation_ref != snapshot.expected_revocation_ref {
        return Err(MantleArtifactAuthShellError::TrustSnapshotInvalid("revocation identity mismatch"));
    }
    if snapshot.revoked_public_key_digests.iter().any(|digest| !valid_digest(digest)) {
        return Err(MantleArtifactAuthShellError::TrustSnapshotInvalid("revoked key digest malformed"));
    }
    if snapshot.trusted_public_keys.is_empty() {
        return Err(MantleArtifactAuthShellError::TrustSnapshotInvalid("trusted full keys missing"));
    }
    debug_assert_eq!(snapshot.policy_hash, snapshot.expected_policy_hash);
    debug_assert!(snapshot.valid_after_unix_s < snapshot.valid_before_unix_s);
    Ok(())
}

fn nix_key_generation(key_token: &str) -> Result<u64, MantleArtifactAuthShellError> {
    let (key_name, _) = key_token
        .split_once(':')
        .ok_or(MantleArtifactAuthShellError::TrustSnapshotInvalid("verifying key token malformed"))?;
    let (_, generation) = key_name
        .rsplit_once(NIX_KEY_GENERATION_SEPARATOR)
        .ok_or(MantleArtifactAuthShellError::TrustSnapshotInvalid("verifying key generation missing"))?;
    let generation = generation
        .parse::<u64>()
        .map_err(|_| MantleArtifactAuthShellError::TrustSnapshotInvalid("verifying key generation malformed"))?;
    if generation == 0 {
        return Err(MantleArtifactAuthShellError::TrustSnapshotInvalid("verifying key generation is zero"));
    }
    Ok(generation)
}

// r[impl mantle.artifact_auth_operational_receipt.persistence]
fn build_mantle_artifact_auth_operational_receipt(
    input: &MantleArtifactAuthShellInput<'_>,
    trust: &MantleArtifactAuthTrustObservation,
    signed: &SignedMantleArtifactAuthStatement,
    report: &MantleArtifactAuthShellReport,
) -> Result<MantleArtifactAuthOperationalReceipt, MantleArtifactAuthShellError> {
    let is_standalone_passed = report.dual_run.standalone.as_ref().is_some_and(|decision| decision.passed);
    let compatibility = &report.dual_run.compatibility;
    if report.cryptographic_failure_code.is_some() || !is_standalone_passed || !compatibility.case_explained {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid(
            "passing explained standalone evidence required",
        ));
    }
    validate_operational_shell_carrier(report, signed, trust)?;
    let mut receipt = MantleArtifactAuthOperationalReceipt {
        schema: OPERATIONAL_RECEIPT_SCHEMA.to_string(),
        consumer: OPERATIONAL_RECEIPT_CONSUMER.to_string(),
        statement_ref: signed.statement_ref.clone(),
        public_key_ref: signed.public_key_ref.clone(),
        signature_ref: signed.signature_ref.clone(),
        signing_authorization_ref: signed.signing_authorization_ref.clone(),
        trust_context_ref: currentness_ref(&trust.currentness_blake3)?,
        key_token_blake3: trust.key_token_blake3.clone(),
        key_generation: trust.generation,
        key_currentness: key_currentness_label(trust.currentness).to_string(),
        signed: signed.clone(),
        standalone_passed: is_standalone_passed,
        compatibility_case_explained: compatibility.case_explained,
        legacy_authoritative: compatibility.legacy_authoritative,
        standalone_authority_admitted: compatibility.standalone_authority_admitted,
        rollback_available: compatibility.rollback_available,
        non_claim: OPERATIONAL_RECEIPT_NON_CLAIM.to_string(),
        receipt_blake3: String::new(),
    };
    receipt.receipt_blake3 = operational_receipt_identity(&receipt)?;
    validate_mantle_artifact_auth_operational_receipt(&receipt)?;
    debug_assert_eq!(input.generation, receipt.key_generation);
    Ok(receipt)
}

// r[impl mantle.artifact_auth_operational_receipt.persistence]
pub fn validate_mantle_artifact_auth_operational_receipt(
    receipt: &MantleArtifactAuthOperationalReceipt,
) -> Result<(), MantleArtifactAuthShellError> {
    if receipt.schema != OPERATIONAL_RECEIPT_SCHEMA || receipt.consumer != OPERATIONAL_RECEIPT_CONSUMER {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("schema or consumer mismatch"));
    }
    validate_operational_receipt_carrier_refs(receipt)?;
    let (_, public_key_bytes) = decode_public_key(&receipt.signed.public_key)?;
    require_carrier_identity(CarrierIdentityInput {
        label: "public key",
        observed: &receipt.public_key_ref,
        expected: &content_ref(&public_key_bytes),
    })?;
    require_carrier_identity(CarrierIdentityInput {
        label: "signature",
        observed: &receipt.signature_ref,
        expected: &content_ref(&receipt.signed.signature_bytes),
    })?;
    require_carrier_identity(CarrierIdentityInput {
        label: "signature hex",
        observed: &receipt.signed.signature_hex,
        expected: &bytes_to_lower_hex(&receipt.signed.signature_bytes),
    })?;
    validate_operational_receipt_authority(receipt)?;
    require_carrier_identity(CarrierIdentityInput {
        label: "receipt",
        observed: &receipt.receipt_blake3,
        expected: &operational_receipt_identity(receipt)?,
    })?;
    debug_assert_eq!(receipt.schema, OPERATIONAL_RECEIPT_SCHEMA);
    debug_assert_eq!(receipt.consumer, OPERATIONAL_RECEIPT_CONSUMER);
    Ok(())
}

fn validate_operational_shell_carrier(
    shell_report: &MantleArtifactAuthShellReport,
    signed: &SignedMantleArtifactAuthStatement,
    trust: &MantleArtifactAuthTrustObservation,
) -> Result<(), MantleArtifactAuthShellError> {
    if shell_report.statement_ref != signed.statement_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("shell statement drift"));
    }
    if shell_report.public_key_ref != signed.public_key_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("shell public key drift"));
    }
    if shell_report.signature_ref != signed.signature_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("shell signature drift"));
    }
    if shell_report.signing_authorization_observation_ref != signed.signing_authorization_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("shell authorization drift"));
    }
    let expected_currentness_ref = currentness_ref(&trust.currentness_blake3)?;
    if shell_report.currentness_ref != expected_currentness_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("shell currentness drift"));
    }
    debug_assert_eq!(shell_report.statement_ref, signed.statement_ref);
    debug_assert_eq!(shell_report.currentness_ref, expected_currentness_ref);
    Ok(())
}

fn validate_operational_receipt_carrier_refs(
    receipt: &MantleArtifactAuthOperationalReceipt,
) -> Result<(), MantleArtifactAuthShellError> {
    if receipt.statement_ref != receipt.signed.statement_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("statement ref drifted"));
    }
    if receipt.public_key_ref != receipt.signed.public_key_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("public key ref drifted"));
    }
    if receipt.signature_ref != receipt.signed.signature_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("signature ref drifted"));
    }
    if receipt.signing_authorization_ref != receipt.signed.signing_authorization_ref {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("authorization ref drifted"));
    }
    debug_assert_eq!(receipt.statement_ref, receipt.signed.statement_ref);
    debug_assert_eq!(receipt.signature_ref, receipt.signed.signature_ref);
    Ok(())
}

fn validate_operational_receipt_authority(
    receipt: &MantleArtifactAuthOperationalReceipt,
) -> Result<(), MantleArtifactAuthShellError> {
    if receipt.key_generation == 0 {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("key generation is zero"));
    }
    if receipt.key_currentness != key_currentness_label(KeyCurrentness::Current) {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("key is not current"));
    }
    if !valid_digest(&receipt.key_token_blake3) {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("key identity is malformed"));
    }
    if !receipt.standalone_passed || !receipt.compatibility_case_explained {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("standalone evidence did not pass"));
    }
    if !receipt.legacy_authoritative || receipt.standalone_authority_admitted {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("authority boundary drifted"));
    }
    if !receipt.rollback_available || receipt.non_claim != OPERATIONAL_RECEIPT_NON_CLAIM {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("rollback or non-claim drifted"));
    }
    debug_assert!(receipt.legacy_authoritative);
    debug_assert!(!receipt.standalone_authority_admitted);
    Ok(())
}

fn operational_receipt_identity(
    receipt: &MantleArtifactAuthOperationalReceipt,
) -> Result<String, MantleArtifactAuthShellError> {
    let material = MantleArtifactAuthReceiptMaterial {
        schema: &receipt.schema,
        consumer: &receipt.consumer,
        statement_ref: &receipt.statement_ref,
        public_key_ref: &receipt.public_key_ref,
        signature_ref: &receipt.signature_ref,
        signing_authorization_ref: &receipt.signing_authorization_ref,
        trust_context_ref: &receipt.trust_context_ref,
        key_token_blake3: &receipt.key_token_blake3,
        key_generation: receipt.key_generation,
        key_currentness: &receipt.key_currentness,
        signed: &receipt.signed,
        standalone_passed: receipt.standalone_passed,
        compatibility_case_explained: receipt.compatibility_case_explained,
        legacy_authoritative: receipt.legacy_authoritative,
        standalone_authority_admitted: receipt.standalone_authority_admitted,
        rollback_available: receipt.rollback_available,
        non_claim: &receipt.non_claim,
    };
    let bytes = serde_json::to_vec(&material)
        .map_err(|error| MantleArtifactAuthShellError::OperationalIo(error.to_string()))?;
    let receipt_ref = content_ref(&bytes);
    debug_assert!(receipt_ref.starts_with(BLAKE3_REF_PREFIX));
    debug_assert_eq!(receipt_ref.len(), BLAKE3_REF_PREFIX.len().saturating_add(BLAKE3_HEX_CHARS));
    Ok(receipt_ref)
}

fn operational_receipt_path(state_dir: &Path, statement_ref: &str) -> Result<PathBuf, MantleArtifactAuthShellError> {
    let digest = statement_ref
        .strip_prefix(BLAKE3_REF_PREFIX)
        .filter(|digest| valid_digest(digest))
        .ok_or(MantleArtifactAuthShellError::OperationalReceiptInvalid("statement ref malformed"))?;
    Ok(state_dir
        .join("action-results")
        .join("v1")
        .join(OPERATIONAL_RECEIPT_DIR)
        .join(format!("{digest}.{OPERATIONAL_RECEIPT_EXTENSION}")))
}

fn write_mantle_artifact_auth_operational_receipt(
    state_dir: &Path,
    receipt: &MantleArtifactAuthOperationalReceipt,
) -> Result<(), MantleArtifactAuthShellError> {
    validate_mantle_artifact_auth_operational_receipt(receipt)?;
    debug_assert!(receipt.legacy_authoritative);
    debug_assert!(!receipt.standalone_authority_admitted);
    let path = operational_receipt_path(state_dir, &receipt.statement_ref)?;
    let parent = path
        .parent()
        .ok_or(MantleArtifactAuthShellError::OperationalReceiptInvalid("receipt parent missing"))?;
    reject_symlink_components(state_dir, parent)?;
    std::fs::create_dir_all(parent).map_err(operational_io)?;
    if path.exists() {
        let existing = read_mantle_artifact_auth_operational_receipt(state_dir, &receipt.statement_ref)?;
        if existing == *receipt {
            return Ok(());
        }
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("immutable replacement denied"));
    }
    let bytes =
        serde_json::to_vec(receipt).map_err(|error| MantleArtifactAuthShellError::OperationalIo(error.to_string()))?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(&path).map_err(operational_io)?;
    file.write_all(&bytes).map_err(operational_io)?;
    file.sync_all().map_err(operational_io)?;
    drop(file);
    let reopened = read_mantle_artifact_auth_operational_receipt(state_dir, &receipt.statement_ref)?;
    if reopened != *receipt {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("write verification failed"));
    }
    Ok(())
}

fn read_mantle_artifact_auth_operational_receipt(
    state_dir: &Path,
    statement_ref: &str,
) -> Result<MantleArtifactAuthOperationalReceipt, MantleArtifactAuthShellError> {
    let path = operational_receipt_path(state_dir, statement_ref)?;
    reject_symlink_path(&path)?;
    let mut file = File::open(&path).map_err(operational_io)?;
    let size_bytes = file.metadata().map_err(operational_io)?.len();
    if size_bytes > OPERATIONAL_RECEIPT_BYTES_MAX {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("receipt exceeds byte bound"));
    }
    let capacity_bytes = usize::try_from(size_bytes)
        .map_err(|_| MantleArtifactAuthShellError::OperationalReceiptInvalid("receipt size conversion failed"))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    file.read_to_end(&mut bytes).map_err(operational_io)?;
    let receipt = serde_json::from_slice::<MantleArtifactAuthOperationalReceipt>(&bytes)
        .map_err(|error| MantleArtifactAuthShellError::OperationalIo(error.to_string()))?;
    validate_mantle_artifact_auth_operational_receipt(&receipt)?;
    Ok(receipt)
}

fn reject_symlink_components(state_dir: &Path, target_parent: &Path) -> Result<(), MantleArtifactAuthShellError> {
    let mut current = state_dir.to_path_buf();
    reject_symlink_path_if_present(&current)?;
    let relative = target_parent
        .strip_prefix(state_dir)
        .map_err(|_| MantleArtifactAuthShellError::OperationalReceiptInvalid("receipt path escaped state root"))?;
    for component in relative.components() {
        current.push(component);
        reject_symlink_path_if_present(&current)?;
    }
    Ok(())
}

fn reject_symlink_path(path: &Path) -> Result<(), MantleArtifactAuthShellError> {
    let metadata = std::fs::symlink_metadata(path).map_err(operational_io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("receipt leaf is not a regular file"));
    }
    Ok(())
}

fn reject_symlink_path_if_present(path: &Path) -> Result<(), MantleArtifactAuthShellError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(MantleArtifactAuthShellError::OperationalReceiptInvalid("receipt path contains symlink"))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(operational_io(error)),
    }
}

fn operational_io(error: std::io::Error) -> MantleArtifactAuthShellError {
    MantleArtifactAuthShellError::OperationalIo(error.to_string())
}

const fn key_currentness_label(currentness: KeyCurrentness) -> &'static str {
    match currentness {
        KeyCurrentness::Current => "current",
        KeyCurrentness::VerificationOverlap => "verification_overlap",
        KeyCurrentness::Superseded => "superseded",
        KeyCurrentness::Revoked => "revoked",
        KeyCurrentness::Unknown => "unknown",
    }
}

struct ValidatedCarrier {
    public_key_bytes: Vec<u8>,
    currentness_ref: String,
    is_legacy_signature_authorized: bool,
}

fn validate_carrier(
    input: &MantleArtifactAuthShellInput<'_>,
    signed: &SignedMantleArtifactAuthStatement,
    statement: &artifact_auth_core::ArtifactStatement,
    statement_bytes: &[u8],
) -> Result<ValidatedCarrier, MantleArtifactAuthShellError> {
    require_carrier_identity(CarrierIdentityInput {
        label: "statement",
        observed: &signed.statement_ref,
        expected: &content_ref(statement_bytes),
    })?;
    let (public_key_name, public_key_bytes) = decode_public_key(&signed.public_key)?;
    if public_key_name != statement.key_id {
        return Err(MantleArtifactAuthShellError::PublicKeyLabelMismatch);
    }
    require_carrier_identity(CarrierIdentityInput {
        label: "public key",
        observed: &signed.public_key_ref,
        expected: &content_ref(&public_key_bytes),
    })?;
    require_carrier_identity(CarrierIdentityInput {
        label: "signature",
        observed: &signed.signature_ref,
        expected: &content_ref(&signed.signature_bytes),
    })?;
    require_carrier_identity(CarrierIdentityInput {
        label: "signature hex",
        observed: &signed.signature_hex,
        expected: &bytes_to_lower_hex(&signed.signature_bytes),
    })?;
    let legacy_signature = legacy_signature(input)?;
    require_carrier_identity(CarrierIdentityInput {
        label: "signing authorization",
        observed: &signed.signing_authorization_ref,
        expected: &content_ref(legacy_signature.signature.as_bytes()),
    })?;
    let currentness_ref = currentness_ref(input.currentness_blake3)?;
    let is_legacy_signature_authorized =
        verify_legacy_signature(legacy_signature, input.signed_record, &signed.public_key);
    debug_assert_eq!(public_key_name, statement.key_id);
    debug_assert_eq!(public_key_bytes.len(), artifact_auth_ed25519::ED25519_PUBLIC_KEY_BYTES);
    debug_assert!(currentness_ref.starts_with(BLAKE3_REF_PREFIX));
    Ok(ValidatedCarrier {
        public_key_bytes,
        currentness_ref,
        is_legacy_signature_authorized,
    })
}

fn admit_signing_input(
    keypair: &KeyPair,
    input: &MantleArtifactAuthShellInput<'_>,
) -> Result<String, MantleArtifactAuthShellError> {
    validate_common_input(input)?;
    if !input.legacy.admitted {
        return Err(MantleArtifactAuthShellError::LegacyDecisionRejected);
    }
    if !input.legacy.trust_basis.iter().any(|basis| basis == input.statement.key_id) {
        return Err(MantleArtifactAuthShellError::LegacyTrustBasisMissing);
    }
    if input.currentness != KeyCurrentness::Current {
        return Err(MantleArtifactAuthShellError::CurrentnessNotCurrent);
    }
    let legacy_signature = legacy_signature(input)?;
    let public_key = keypair.verifying_key.to_string();
    if keypair.verifying_key.name() != input.statement.key_id
        || input.statement.producer_id != input.statement.record.producer_identity
        || input.statement.producer_id != keypair.verifying_key.name()
    {
        return Err(MantleArtifactAuthShellError::SignerMismatch);
    }
    if !verify_legacy_signature(legacy_signature, input.signed_record, &public_key) {
        return Err(MantleArtifactAuthShellError::LegacySignatureInvalid);
    }
    let (_, public_key_bytes) = decode_public_key(&public_key)?;
    let key_identity = artifact_auth_ed25519::public_key_identity(&public_key_bytes);
    if key_identity.digest_hex != input.statement.key_identity_blake3 {
        return Err(MantleArtifactAuthShellError::PublicKeyIdentityMismatch);
    }
    debug_assert!(input.legacy.admitted);
    debug_assert_eq!(input.statement.producer_id, keypair.verifying_key.name());
    Ok(content_ref(legacy_signature.signature.as_bytes()))
}

fn validate_common_input(input: &MantleArtifactAuthShellInput<'_>) -> Result<(), MantleArtifactAuthShellError> {
    if input.legacy.result_ref != input.signed_record.record.result_ref {
        return Err(MantleArtifactAuthShellError::LegacyDecisionMismatch);
    }
    if input.statement.record != &input.signed_record.record {
        return Err(MantleArtifactAuthShellError::SignedRecordMismatch);
    }
    if input.statement.producer_id != input.statement.record.producer_identity
        || input.statement.key_id != input.statement.record.producer_identity
    {
        return Err(MantleArtifactAuthShellError::SignerMismatch);
    }
    if input.legacy.admitted && !input.legacy.trust_basis.iter().any(|basis| basis == input.statement.key_id) {
        return Err(MantleArtifactAuthShellError::LegacyTrustBasisMissing);
    }
    currentness_ref(input.currentness_blake3)?;
    map_statement(&input.statement)?;
    Ok(())
}

fn legacy_signature<'a>(
    input: &'a MantleArtifactAuthShellInput<'_>,
) -> Result<&'a crunch_action_result_core::DetachedRecordSignature, MantleArtifactAuthShellError> {
    let mut matching = input
        .signed_record
        .record_signatures
        .iter()
        .filter(|signature| signature.key_name == input.statement.key_id);
    let Some(signature) = matching.next() else {
        return Err(MantleArtifactAuthShellError::LegacySignatureMissingOrAmbiguous);
    };
    if matching.next().is_some() {
        return Err(MantleArtifactAuthShellError::LegacySignatureMissingOrAmbiguous);
    }
    Ok(signature)
}

fn verify_legacy_signature(
    detached: &crunch_action_result_core::DetachedRecordSignature,
    signed_record: &SignedActionResultRecord,
    public_key: &str,
) -> bool {
    let Ok(signature) = Signature::<String>::parse(&detached.signature) else {
        return false;
    };
    if signature.name().as_str() != detached.key_name {
        return false;
    }
    let Ok(verifying_key) = VerifyingKey::parse(public_key) else {
        return false;
    };
    verifying_key.verify(&signed_record.record.result_ref, &signature.as_ref())
}

fn map_statement(
    input: &MantleArtifactAuthStatementInput<'_>,
) -> Result<artifact_auth_core::ArtifactStatement, MantleArtifactAuthShellError> {
    map_mantle_artifact_auth_statement(input)
        .map_err(|issues| MantleArtifactAuthShellError::StatementMapping(issues.join(",")))
}

fn decode_public_key(encoded: &str) -> Result<(String, Vec<u8>), MantleArtifactAuthShellError> {
    VerifyingKey::parse(encoded).map_err(|_| MantleArtifactAuthShellError::PublicKeyMalformed)?;
    let (name, encoded_bytes) = encoded.split_once(':').ok_or(MantleArtifactAuthShellError::PublicKeyMalformed)?;
    let mut buffer = [0_u8; PUBLIC_KEY_DECODE_BUFFER_BYTES];
    let decoded = BASE64
        .decode_mut(encoded_bytes.as_bytes(), &mut buffer)
        .map_err(|_| MantleArtifactAuthShellError::PublicKeyMalformed)?;
    if decoded != artifact_auth_ed25519::ED25519_PUBLIC_KEY_BYTES {
        return Err(MantleArtifactAuthShellError::PublicKeyMalformed);
    }
    Ok((name.to_string(), buffer[..decoded].to_vec()))
}

fn currentness_ref(digest_hex: &str) -> Result<String, MantleArtifactAuthShellError> {
    if !valid_digest(digest_hex) {
        return Err(MantleArtifactAuthShellError::CurrentnessMalformed);
    }
    Ok(format!("{BLAKE3_REF_PREFIX}{digest_hex}"))
}

fn valid_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

struct CarrierIdentityInput<'a> {
    label: &'static str,
    observed: &'a str,
    expected: &'a str,
}

fn require_carrier_identity(input: CarrierIdentityInput<'_>) -> Result<(), MantleArtifactAuthShellError> {
    if input.observed != input.expected {
        return Err(MantleArtifactAuthShellError::CarrierIdentityMismatch(input.label));
    }
    Ok(())
}

fn content_ref(bytes: &[u8]) -> String {
    format!("{BLAKE3_REF_PREFIX}{}", blake3::hash(bytes).to_hex())
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(HEX_CHARS_PER_BYTE));
    for byte in bytes {
        let high = usize::from(byte >> BITS_PER_NIBBLE);
        let low = usize::from(byte & LOW_NIBBLE_MASK);
        encoded.push(char::from(HEX_DIGITS[high]));
        encoded.push(char::from(HEX_DIGITS[low]));
    }
    debug_assert_eq!(encoded.len(), bytes.len().saturating_mul(HEX_CHARS_PER_BYTE));
    encoded
}

#[cfg(test)]
mod tests {
    use artifact_auth_core::KeyCurrentness;
    use crunch_action_result_core::ACTION_RECEIPT_REF_PREFIX;
    use crunch_action_result_core::ACTION_REF_PREFIX;
    use crunch_action_result_core::ActionResultOutput;
    use crunch_action_result_core::ActionResultRecord;
    use crunch_action_result_core::ActionResultRecordInput;
    use crunch_action_result_core::DetachedRecordSignature;
    use crunch_action_result_core::NETWORK_POLICY_REF_PREFIX;
    use crunch_action_result_core::OBJECT_REF_PREFIX;
    use crunch_action_result_core::PATH_INFO_REF_PREFIX;
    use crunch_action_result_core::PRODUCER_POLICY_REF_PREFIX;
    use crunch_action_result_core::PUBLICATION_POLICY_REF_PREFIX;
    use crunch_action_result_core::REFERENCE_SCAN_REF_PREFIX;
    use crunch_action_result_core::SANDBOX_POLICY_REF_PREFIX;
    use crunch_action_result_core::SIGNATURE_REF_PREFIX;
    use crunch_action_result_core::SignedActionResultRecord;
    use crunch_action_result_core::canonical_action_result;
    use ed25519_dalek::SigningKey as DalekSigningKey;
    use nix_compat::narinfo::SigningKey;
    use pretty_assertions::assert_eq;

    use super::*;

    const PROFILE_ID: &str = "mantle-action-result-artifact-auth-v1";
    const KEY_NAME: &str = "builder-key-1";
    const GENERATION_ONE: u64 = 1;
    const TEST_KEY_SEED: u8 = 11;
    const WRONG_KEY_SEED: u8 = 13;
    const MALFORMED_SIGNATURE_BYTES: usize = artifact_auth_ed25519::ED25519_SIGNATURE_BYTES - 1;

    struct Fixture {
        record: ActionResultRecord,
        signed_record: SignedActionResultRecord,
        legacy: CandidateDecision,
        keypair: KeyPair,
        key_identity_blake3: String,
        currentness_blake3: String,
    }

    impl Fixture {
        fn input(&self) -> MantleArtifactAuthShellInput<'_> {
            MantleArtifactAuthShellInput {
                statement: MantleArtifactAuthStatementInput {
                    profile_id: PROFILE_ID,
                    record: &self.record,
                    producer_id: KEY_NAME,
                    key_id: KEY_NAME,
                    key_identity_blake3: &self.key_identity_blake3,
                    oci_manifest_sha256: None,
                    metadata_manifest_sha256: None,
                },
                signed_record: &self.signed_record,
                legacy: &self.legacy,
                generation: GENERATION_ONE,
                currentness: KeyCurrentness::Current,
                currentness_blake3: &self.currentness_blake3,
            }
        }
    }

    fn digest(character: char) -> String {
        character.to_string().repeat(BLAKE3_HEX_CHARS)
    }

    fn typed(prefix: &str, character: char) -> String {
        format!("{prefix}{}", digest(character))
    }

    fn keypair(seed: u8) -> KeyPair {
        let raw = DalekSigningKey::from_bytes(&[seed; artifact_auth_ed25519::ED25519_PUBLIC_KEY_BYTES]);
        KeyPair {
            signing_key: SigningKey::new(KEY_NAME.to_string(), raw.clone()),
            verifying_key: VerifyingKey::new(KEY_NAME.to_string(), raw.verifying_key()),
        }
    }

    fn record(output_character: char) -> ActionResultRecord {
        canonical_action_result(ActionResultRecordInput {
            action_ref: typed(ACTION_REF_PREFIX, '1'),
            outputs: vec![ActionResultOutput {
                name: "out".to_string(),
                object_ref: typed(OBJECT_REF_PREFIX, output_character),
                store_path: "/mantle/store/result".to_string(),
                path_info_ref: typed(PATH_INFO_REF_PREFIX, '3'),
            }],
            action_receipt_ref: typed(ACTION_RECEIPT_REF_PREFIX, '4'),
            reference_scan_refs: vec![typed(REFERENCE_SCAN_REF_PREFIX, '5')],
            sandbox_policy_ref: typed(SANDBOX_POLICY_REF_PREFIX, '6'),
            network_policy_ref: typed(NETWORK_POLICY_REF_PREFIX, '7'),
            producer_identity: KEY_NAME.to_string(),
            producer_policy_ref: typed(PRODUCER_POLICY_REF_PREFIX, '8'),
            signature_refs: vec![typed(SIGNATURE_REF_PREFIX, '9')],
            publication_policy_ref: typed(PUBLICATION_POLICY_REF_PREFIX, 'a'),
            non_claims: action_result_non_claims(),
        })
        .expect("valid action result")
    }

    fn action_result_non_claims() -> Vec<String> {
        vec![
            "ca-mapping-presence-is-not-output-trust".to_string(),
            "executor-correctness".to_string(),
            "index-presence-is-not-output-trust".to_string(),
        ]
    }

    fn signed_record(record: &ActionResultRecord, keypair: &KeyPair) -> SignedActionResultRecord {
        let signature = keypair.signing_key.sign(record.result_ref.as_bytes()).to_owned();
        SignedActionResultRecord {
            record: record.clone(),
            record_signatures: vec![DetachedRecordSignature {
                key_name: KEY_NAME.to_string(),
                signature: signature.to_string(),
            }],
        }
    }

    fn legacy(record: &ActionResultRecord, admitted: bool, diagnostics: Vec<String>) -> CandidateDecision {
        CandidateDecision {
            result_ref: record.result_ref.clone(),
            source_id: "local-source".to_string(),
            source_class: "configured-source".to_string(),
            admitted,
            diagnostics,
            trust_basis: vec![KEY_NAME.to_string()],
            output_set_digest_blake3: admitted.then(|| digest('e')),
        }
    }

    fn fixture() -> Fixture {
        let keypair = keypair(TEST_KEY_SEED);
        let record = record('2');
        let signed_record = signed_record(&record, &keypair);
        let (_, public_key_bytes) = decode_public_key(&keypair.verifying_key.to_string()).expect("public key");
        Fixture {
            legacy: legacy(&record, true, Vec::new()),
            record,
            signed_record,
            key_identity_blake3: artifact_auth_ed25519::public_key_identity(&public_key_bytes).digest_hex,
            currentness_blake3: digest('c'),
            keypair,
        }
    }

    // r[verify mantle.artifact_auth_shell.exact_verification]
    // r[verify mantle.artifact_auth_shell.evidence]
    // r[verify mantle.artifact_auth_shell.authority]
    #[test]
    fn exact_statement_round_trip_preserves_product_authority() {
        let fixture = fixture();
        let input = fixture.input();
        let signed = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        let report = evaluate_mantle_artifact_auth_shell(&input, &signed).expect("evaluate");

        assert!(report.legacy_signature_authorized);
        assert!(report.cryptographic_failure_code.is_none());
        assert!(report.dual_run.standalone.as_ref().is_some_and(|decision| decision.passed));
        assert!(report.dual_run.compatibility.case_explained);
        assert!(report.dual_run.compatibility.legacy_authoritative);
        assert!(!report.dual_run.compatibility.standalone_authority_admitted);
        assert!(report.dual_run.compatibility.rollback_available);
        assert!(report.dual_run.repository_authority_retained);
        assert!(report.dual_run.build_authority_retained);
        assert!(report.dual_run.release_authority_retained);
        assert_eq!(report.statement_ref, signed.statement_ref);
        assert_eq!(report.currentness_ref, format!("{BLAKE3_REF_PREFIX}{}", fixture.currentness_blake3));
    }

    // r[verify mantle.artifact_auth_shell.authorization]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn signing_rejects_legacy_authorization_and_currentness_failures() {
        let mut fixture = fixture();
        fixture.legacy.admitted = false;
        let rejected = sign_mantle_artifact_auth_statement(&fixture.keypair, &fixture.input());
        assert_eq!(rejected, Err(MantleArtifactAuthShellError::LegacyDecisionRejected));

        fixture.legacy.admitted = true;
        fixture.legacy.trust_basis.clear();
        let rejected = sign_mantle_artifact_auth_statement(&fixture.keypair, &fixture.input());
        assert_eq!(rejected, Err(MantleArtifactAuthShellError::LegacyTrustBasisMissing));

        fixture.legacy.trust_basis.push(KEY_NAME.to_string());
        let mut input = fixture.input();
        input.currentness = KeyCurrentness::Unknown;
        let rejected = sign_mantle_artifact_auth_statement(&fixture.keypair, &input);
        assert_eq!(rejected, Err(MantleArtifactAuthShellError::CurrentnessNotCurrent));
    }

    // r[verify mantle.artifact_auth_shell.authorization]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn signing_rejects_tampered_legacy_signature_and_wrong_full_key() {
        let mut tampered_fixture = fixture();
        tampered_fixture.signed_record.record_signatures[0].signature.push('x');
        let rejected = sign_mantle_artifact_auth_statement(&tampered_fixture.keypair, &tampered_fixture.input());
        assert_eq!(rejected, Err(MantleArtifactAuthShellError::LegacySignatureInvalid));

        let fixture = fixture();
        let wrong_keypair = keypair(WRONG_KEY_SEED);
        let rejected = sign_mantle_artifact_auth_statement(&wrong_keypair, &fixture.input());
        assert_eq!(rejected, Err(MantleArtifactAuthShellError::LegacySignatureInvalid));
    }

    // r[verify mantle.artifact_auth_shell.exact_verification]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn legacy_result_signature_cannot_verify_as_standalone_statement() {
        let fixture = fixture();
        let input = fixture.input();
        let mut signed = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        let legacy = Signature::<String>::parse(&fixture.signed_record.record_signatures[0].signature)
            .expect("legacy signature");
        signed.replace_signature_bytes_for_test(legacy.bytes().to_vec());
        let report = evaluate_mantle_artifact_auth_shell(&input, &signed).expect("evaluate");

        assert_eq!(
            report.cryptographic_failure_code.as_deref(),
            Some(artifact_auth_ed25519::Ed25519Failure::SignatureInvalid.code())
        );
        assert!(report.dual_run.compatibility.decision_drift);
        assert!(!report.dual_run.compatibility.case_explained);
    }

    // r[verify mantle.artifact_auth_shell.exact_verification]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn changed_statement_preimage_is_rejected_by_carrier_identity() {
        let fixture = fixture();
        let input = fixture.input();
        let signed = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        let changed_record = record('d');
        let changed_signed_record = signed_record(&changed_record, &fixture.keypair);
        let changed_legacy = legacy(&changed_record, true, Vec::new());
        let changed_input = MantleArtifactAuthShellInput {
            statement: MantleArtifactAuthStatementInput {
                record: &changed_record,
                ..input.statement
            },
            signed_record: &changed_signed_record,
            legacy: &changed_legacy,
            ..input
        };

        let result = evaluate_mantle_artifact_auth_shell(&changed_input, &signed);
        assert_eq!(result, Err(MantleArtifactAuthShellError::CarrierIdentityMismatch("statement")));
    }

    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn wrong_public_key_is_rejected_before_malformed_signature_is_classified() {
        let fixture = fixture();
        let input = fixture.input();
        let mut wrong_key = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        wrong_key.replace_public_key_for_test(keypair(WRONG_KEY_SEED).verifying_key.to_string());
        assert_eq!(
            evaluate_mantle_artifact_auth_shell(&input, &wrong_key),
            Err(MantleArtifactAuthShellError::LegacySignatureInvalid)
        );

        let mut malformed = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        malformed.replace_signature_bytes_for_test(vec![0_u8; MALFORMED_SIGNATURE_BYTES]);
        let malformed_report = evaluate_mantle_artifact_auth_shell(&input, &malformed).expect("evaluate");
        assert_eq!(
            malformed_report.cryptographic_failure_code.as_deref(),
            Some(artifact_auth_ed25519::Ed25519Failure::SignatureLength.code())
        );
    }

    // r[verify mantle.artifact_auth_shell.authorization]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn ambiguous_legacy_signature_and_producer_substitution_are_rejected() {
        let mut ambiguous_fixture = fixture();
        ambiguous_fixture
            .signed_record
            .record_signatures
            .push(ambiguous_fixture.signed_record.record_signatures[0].clone());
        assert_eq!(
            sign_mantle_artifact_auth_statement(&ambiguous_fixture.keypair, &ambiguous_fixture.input()),
            Err(MantleArtifactAuthShellError::LegacySignatureMissingOrAmbiguous)
        );

        let fixture = fixture();
        let input = fixture.input();
        let substituted = MantleArtifactAuthShellInput {
            statement: MantleArtifactAuthStatementInput {
                producer_id: "other-producer",
                ..input.statement
            },
            ..input
        };
        assert_eq!(
            sign_mantle_artifact_auth_statement(&fixture.keypair, &substituted),
            Err(MantleArtifactAuthShellError::SignerMismatch)
        );
    }

    // r[verify mantle.artifact_auth_shell.evidence]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn malformed_and_relabeled_public_key_carriers_are_rejected() {
        let fixture = fixture();
        let input = fixture.input();
        let signed = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");

        let mut malformed = signed.clone();
        malformed.public_key = "not-a-public-key".to_string();
        assert_eq!(
            evaluate_mantle_artifact_auth_shell(&input, &malformed),
            Err(MantleArtifactAuthShellError::PublicKeyMalformed)
        );

        let mut relabeled = signed;
        let encoded_key = relabeled.public_key.split_once(':').expect("public key token").1.to_string();
        relabeled.public_key = format!("other-key-1:{encoded_key}");
        assert_eq!(
            evaluate_mantle_artifact_auth_shell(&input, &relabeled),
            Err(MantleArtifactAuthShellError::PublicKeyLabelMismatch)
        );
    }

    // r[verify mantle.artifact_auth_shell.evidence]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn carrier_statement_signature_encoding_and_authorization_drift_fail_closed() {
        let fixture = fixture();
        let input = fixture.input();
        let signed = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        for (label, mutate) in [
            ("statement", 0_u8),
            ("signature hex", 1_u8),
            ("signing authorization", 2_u8),
        ] {
            let mut changed = signed.clone();
            match mutate {
                0 => changed.statement_ref.push('0'),
                1 => changed.signature_hex.push('0'),
                _ => changed.signing_authorization_ref.push('0'),
            }
            assert_eq!(
                evaluate_mantle_artifact_auth_shell(&input, &changed),
                Err(MantleArtifactAuthShellError::CarrierIdentityMismatch(label))
            );
        }
    }

    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn unknown_and_revoked_currentness_remain_explicit_standalone_failures() {
        let fixture = fixture();
        let signed = sign_mantle_artifact_auth_statement(&fixture.keypair, &fixture.input()).expect("sign");
        for currentness in [KeyCurrentness::Unknown, KeyCurrentness::Revoked] {
            let mut input = fixture.input();
            input.currentness = currentness;
            let report = evaluate_mantle_artifact_auth_shell(&input, &signed).expect("evaluate");
            assert!(!report.dual_run.standalone.as_ref().expect("standalone").passed);
            assert!(report.dual_run.compatibility.decision_drift);
            assert!(report.dual_run.compatibility.standalone_issue_causes.iter().any(|cause| cause == "currentness"));
        }
    }

    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn protocol_pass_standalone_fail_and_reverse_drift_are_blocked() {
        let fixture = fixture();
        let input = fixture.input();
        let mut standalone_failure = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        standalone_failure.signature_bytes[0] ^= 1;
        standalone_failure.signature_ref = content_ref(&standalone_failure.signature_bytes);
        standalone_failure.signature_hex = bytes_to_lower_hex(&standalone_failure.signature_bytes);
        let report = evaluate_mantle_artifact_auth_shell(&input, &standalone_failure).expect("evaluate");
        assert!(report.dual_run.compatibility.decision_drift);

        let mut rejected_legacy = legacy(&fixture.record, false, vec!["publication-policy-rejected".to_string()]);
        rejected_legacy.trust_basis = vec![KEY_NAME.to_string()];
        let reverse_input = MantleArtifactAuthShellInput {
            legacy: &rejected_legacy,
            ..input
        };
        let valid = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        let reverse = evaluate_mantle_artifact_auth_shell(&reverse_input, &valid).expect("evaluate");
        assert!(reverse.dual_run.compatibility.decision_drift);
        assert!(!reverse.dual_run.compatibility.case_explained);
    }

    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn unrelated_dual_rejections_are_not_false_parity() {
        let fixture = fixture();
        let input = fixture.input();
        let mut signed = sign_mantle_artifact_auth_statement(&fixture.keypair, &input).expect("sign");
        signed.signature_bytes[0] ^= 1;
        signed.signature_ref = content_ref(&signed.signature_bytes);
        signed.signature_hex = bytes_to_lower_hex(&signed.signature_bytes);
        let rejected_legacy = legacy(&fixture.record, false, vec!["publication-policy-rejected".to_string()]);
        let rejected_input = MantleArtifactAuthShellInput {
            legacy: &rejected_legacy,
            ..input
        };
        let report = evaluate_mantle_artifact_auth_shell(&rejected_input, &signed).expect("evaluate");

        assert!(!report.dual_run.standalone.as_ref().expect("standalone").passed);
        assert!(!report.dual_run.compatibility.decision_drift);
        assert!(report.dual_run.compatibility.blockers.iter().any(|blocker| blocker == "unrelated-rejection-causes"));
        assert!(!report.dual_run.compatibility.case_explained);
    }

    // r[verify mantle.artifact_auth_operational_receipt.trust]
    // r[verify mantle.artifact_auth_operational_receipt.persistence]
    // r[verify mantle.artifact_auth_operational_receipt.replay]
    // r[verify mantle.artifact_auth_operational_receipt.authority]
    #[test]
    fn operational_receipt_persists_replays_and_rejects_trust_drift() {
        const VALID_AFTER_UNIX_S: u64 = 10;
        const VALID_AT_UNIX_S: u64 = 15;
        const VALID_BEFORE_UNIX_S: u64 = 20;

        let fixture = fixture();
        let state = tempfile::tempdir().expect("state root");
        let trusted = vec![fixture.keypair.verifying_key.clone()];
        let revoked = BTreeSet::new();
        let trust = MantleArtifactAuthTrustSnapshot {
            policy_hash: "portable-policy-v1",
            expected_policy_hash: "portable-policy-v1",
            valid_after_unix_s: VALID_AFTER_UNIX_S,
            valid_before_unix_s: VALID_BEFORE_UNIX_S,
            valid_at_unix_s: VALID_AT_UNIX_S,
            revocation_ref: Some("revocations:v1"),
            expected_revocation_ref: Some("revocations:v1"),
            trusted_public_keys: &trusted,
            revoked_public_key_digests: &revoked,
        };
        let input = operational_input(&fixture, trust);
        let receipt = capture_mantle_artifact_auth_operational_receipt(state.path(), &fixture.keypair, &input)
            .expect("capture operational receipt");
        let replay = replay_mantle_artifact_auth_operational_receipt(
            state.path(),
            &fixture.keypair,
            &input,
            &receipt.statement_ref,
        )
        .expect("replay persisted receipt");

        assert!(replay.dual_run.standalone.as_ref().is_some_and(|decision| decision.passed));
        assert!(receipt.legacy_authoritative);
        assert!(!receipt.standalone_authority_admitted);
        assert!(receipt.rollback_available);
        assert_eq!(receipt.key_generation, GENERATION_ONE);
        assert_eq!(receipt.key_currentness, "current");

        let changed_trust = MantleArtifactAuthTrustSnapshot {
            policy_hash: "portable-policy-v2",
            expected_policy_hash: "portable-policy-v2",
            ..trust
        };
        let changed_input = operational_input(&fixture, changed_trust);
        let drift = capture_mantle_artifact_auth_operational_receipt(state.path(), &fixture.keypair, &changed_input)
            .expect_err("immutable receipt rejects changed trust snapshot");
        assert_eq!(drift, MantleArtifactAuthShellError::OperationalReceiptInvalid("immutable replacement denied"));

        let path = operational_receipt_path(state.path(), &receipt.statement_ref).expect("receipt path");
        std::fs::write(path, b"{\"schema\":\"tampered\"}").expect("tamper receipt");
        let tampered = replay_mantle_artifact_auth_operational_receipt(
            state.path(),
            &fixture.keypair,
            &input,
            &receipt.statement_ref,
        )
        .expect_err("tampered receipt rejected");
        assert!(matches!(tampered, MantleArtifactAuthShellError::OperationalIo(_)));
    }

    // r[verify mantle.artifact_auth_operational_receipt.trust]
    // r[verify mantle.artifact_auth_operational_receipt.replay]
    #[test]
    fn operational_receipt_denies_unknown_revoked_stale_and_symlinked_trust() {
        const VALID_AFTER_UNIX_S: u64 = 10;
        const VALID_AT_UNIX_S: u64 = 15;
        const VALID_BEFORE_UNIX_S: u64 = 20;

        let fixture = fixture();
        let state = tempfile::tempdir().expect("state root");
        let wrong_trusted = vec![keypair(WRONG_KEY_SEED).verifying_key];
        let no_revocations = BTreeSet::new();
        let unknown_trust = MantleArtifactAuthTrustSnapshot {
            policy_hash: "portable-policy-v1",
            expected_policy_hash: "portable-policy-v1",
            valid_after_unix_s: VALID_AFTER_UNIX_S,
            valid_before_unix_s: VALID_BEFORE_UNIX_S,
            valid_at_unix_s: VALID_AT_UNIX_S,
            revocation_ref: None,
            expected_revocation_ref: None,
            trusted_public_keys: &wrong_trusted,
            revoked_public_key_digests: &no_revocations,
        };
        assert_eq!(
            capture_mantle_artifact_auth_operational_receipt(
                state.path(),
                &fixture.keypair,
                &operational_input(&fixture, unknown_trust),
            ),
            Err(MantleArtifactAuthShellError::CurrentnessNotCurrent)
        );

        let trusted = vec![fixture.keypair.verifying_key.clone()];
        let key_token_blake3 = blake3::hash(fixture.keypair.verifying_key.to_string().as_bytes()).to_hex().to_string();
        let revoked = BTreeSet::from([key_token_blake3]);
        let revoked_trust = MantleArtifactAuthTrustSnapshot {
            trusted_public_keys: &trusted,
            revoked_public_key_digests: &revoked,
            ..unknown_trust
        };
        assert_eq!(
            capture_mantle_artifact_auth_operational_receipt(
                state.path(),
                &fixture.keypair,
                &operational_input(&fixture, revoked_trust),
            ),
            Err(MantleArtifactAuthShellError::CurrentnessNotCurrent)
        );

        let stale_trust = MantleArtifactAuthTrustSnapshot {
            valid_at_unix_s: VALID_BEFORE_UNIX_S,
            trusted_public_keys: &trusted,
            revoked_public_key_digests: &no_revocations,
            ..unknown_trust
        };
        assert_eq!(
            derive_mantle_artifact_auth_trust(&fixture.keypair, &stale_trust),
            Err(MantleArtifactAuthShellError::TrustSnapshotInvalid("validity window mismatch"))
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let symlink_state = tempfile::tempdir().expect("symlink state root");
            let outside = tempfile::tempdir().expect("outside root");
            symlink(outside.path(), symlink_state.path().join("action-results")).expect("create path substitution");
            let current_trust = MantleArtifactAuthTrustSnapshot {
                trusted_public_keys: &trusted,
                revoked_public_key_digests: &no_revocations,
                ..unknown_trust
            };
            let denied = capture_mantle_artifact_auth_operational_receipt(
                symlink_state.path(),
                &fixture.keypair,
                &operational_input(&fixture, current_trust),
            )
            .expect_err("symlinked state path rejected");
            assert_eq!(
                denied,
                MantleArtifactAuthShellError::OperationalReceiptInvalid("receipt path contains symlink")
            );
        }
    }

    fn operational_input<'a>(
        fixture: &'a Fixture,
        trust: MantleArtifactAuthTrustSnapshot<'a>,
    ) -> MantleArtifactAuthOperationalInput<'a> {
        MantleArtifactAuthOperationalInput {
            statement: MantleArtifactAuthStatementInput {
                profile_id: PROFILE_ID,
                record: &fixture.record,
                producer_id: KEY_NAME,
                key_id: KEY_NAME,
                key_identity_blake3: &fixture.key_identity_blake3,
                oci_manifest_sha256: None,
                metadata_manifest_sha256: None,
            },
            signed_record: &fixture.signed_record,
            legacy: &fixture.legacy,
            trust,
        }
    }
}
