use std::collections::BTreeSet;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use artifact_auth_ed25519::ED25519_PUBLIC_KEY_BYTES;
use artifact_auth_ed25519::public_key_identity;
use crunch_action_result_core::CandidateDecision;
use crunch_action_result_core::SignedActionResultRecord;
use crunch_action_result_core::artifact_auth::MantleArtifactAuthStatementInput;
use crunch_build::KeyPair;
use crunch_build::artifact_auth::MantleArtifactAuthOperationalInput;
use crunch_build::artifact_auth::MantleArtifactAuthTrustSnapshot;
use crunch_build::artifact_auth::capture_mantle_artifact_auth_operational_receipt;
use crunch_build::artifact_auth::replay_mantle_artifact_auth_operational_receipt;
use crunch_build::load_keypair;
use data_encoding::BASE64;
use nix_compat::narinfo::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

const PROFILE_ID: &str = "mantle-action-result-artifact-auth-canary-v1";
const VALIDITY_SKEW_SECONDS: u64 = 60;
const VALIDITY_WINDOW_SECONDS: u64 = 3_600;
const RECORD_DIRECTORY: &str = "action-results/v1/records";
const SIGNING_KEY_FILE: &str = "signing-key";
const REVOCATION_FILE: &str = "artifact-auth-canary-revocations.json";
const CONTEXT_FILE: &str = "artifact-auth-canary-context.json";
const RECEIPT_FILE: &str = "operational-receipt.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CanaryContext {
    valid_after_unix_s: u64,
    valid_before_unix_s: u64,
    valid_at_unix_s: u64,
    statement_ref: Option<String>,
}

struct Material {
    keypair: KeyPair,
    signed_record: SignedActionResultRecord,
    legacy: CandidateDecision,
    key_identity_blake3: String,
    policy_hash: String,
    revocation_ref: String,
    revoked_public_key_digests: BTreeSet<String>,
    trusted_public_keys: Vec<VerifyingKey>,
    context: CanaryContext,
}

impl Material {
    fn input(&self) -> MantleArtifactAuthOperationalInput<'_> {
        let key_name = self.keypair.verifying_key.name();
        MantleArtifactAuthOperationalInput {
            statement: MantleArtifactAuthStatementInput {
                profile_id: PROFILE_ID,
                record: &self.signed_record.record,
                producer_id: key_name,
                key_id: key_name,
                key_identity_blake3: &self.key_identity_blake3,
                oci_manifest_sha256: None,
                metadata_manifest_sha256: None,
            },
            signed_record: &self.signed_record,
            legacy: &self.legacy,
            trust: MantleArtifactAuthTrustSnapshot {
                policy_hash: &self.policy_hash,
                expected_policy_hash: &self.policy_hash,
                valid_after_unix_s: self.context.valid_after_unix_s,
                valid_before_unix_s: self.context.valid_before_unix_s,
                valid_at_unix_s: self.context.valid_at_unix_s,
                revocation_ref: Some(&self.revocation_ref),
                expected_revocation_ref: Some(&self.revocation_ref),
                trusted_public_keys: &self.trusted_public_keys,
                revoked_public_key_digests: &self.revoked_public_key_digests,
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.len() != 4 {
        return Err("usage: artifact_auth_canary <capture|replay|revoke> <state-dir> <evidence-dir>".into());
    }
    let mode = arguments[1].as_str();
    let state_dir = PathBuf::from(&arguments[2]);
    let evidence_dir = PathBuf::from(&arguments[3]);
    std::fs::create_dir_all(&evidence_dir)?;
    match mode {
        "capture" => capture(&state_dir, &evidence_dir),
        "replay" => replay(&state_dir, &evidence_dir),
        "revoke" => revoke(&state_dir, &evidence_dir),
        _ => Err(format!("unknown canary mode: {mode}").into()),
    }
}

fn capture(state_dir: &Path, evidence_dir: &Path) -> Result<(), Box<dyn Error>> {
    initialize_revocations(state_dir)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let context = CanaryContext {
        valid_after_unix_s: now.saturating_sub(VALIDITY_SKEW_SECONDS),
        valid_before_unix_s: now.saturating_add(VALIDITY_WINDOW_SECONDS),
        valid_at_unix_s: now,
        statement_ref: None,
    };
    write_json(&state_dir.join(CONTEXT_FILE), &context)?;
    let material = load_material(state_dir, evidence_dir)?;
    let receipt = capture_mantle_artifact_auth_operational_receipt(state_dir, &material.keypair, &material.input())?;
    let mut persisted_context = material.context.clone();
    persisted_context.statement_ref = Some(receipt.statement_ref.clone());
    write_json(&state_dir.join(CONTEXT_FILE), &persisted_context)?;
    write_json(&evidence_dir.join(RECEIPT_FILE), &receipt)?;
    write_json(
        &evidence_dir.join("capture-summary.json"),
        &serde_json::json!({
            "schema": "mantle-artifact-auth-canary-summary-v1",
            "phase": "capture",
            "result": "pass",
            "source_result_ref": material.signed_record.record.result_ref,
            "statement_ref": receipt.statement_ref,
            "receipt_blake3": receipt.receipt_blake3,
            "legacy_authoritative": receipt.legacy_authoritative,
            "standalone_authority_admitted": receipt.standalone_authority_admitted,
            "rollback_available": receipt.rollback_available,
            "non_claim": "non-production local canary evidence does not grant cache, build, registry, or release authority"
        }),
    )?;
    Ok(())
}

fn replay(state_dir: &Path, evidence_dir: &Path) -> Result<(), Box<dyn Error>> {
    let material = load_material(state_dir, evidence_dir)?;
    let statement_ref =
        material.context.statement_ref.as_deref().ok_or("capture context has no statement reference")?;
    let report = replay_mantle_artifact_auth_operational_receipt(
        state_dir,
        &material.keypair,
        &material.input(),
        statement_ref,
    )?;
    write_json(
        &evidence_dir.join("replay-summary.json"),
        &serde_json::json!({
            "schema": "mantle-artifact-auth-canary-summary-v1",
            "phase": "restart-replay",
            "result": "pass",
            "statement_ref": report.statement_ref,
            "standalone_passed": report.dual_run.standalone.as_ref().is_some_and(|decision| decision.passed),
            "legacy_authoritative": report.dual_run.compatibility.legacy_authoritative,
            "standalone_authority_admitted": report.dual_run.compatibility.standalone_authority_admitted,
            "rollback_available": report.dual_run.compatibility.rollback_available
        }),
    )?;
    Ok(())
}

fn revoke(state_dir: &Path, evidence_dir: &Path) -> Result<(), Box<dyn Error>> {
    let keypair = read_keypair(state_dir)?;
    let key_token_blake3 = blake3::hash(keypair.verifying_key.to_string().as_bytes()).to_hex().to_string();
    write_json(&state_dir.join(REVOCATION_FILE), &vec![key_token_blake3])?;
    let material = load_material(state_dir, evidence_dir)?;
    let statement_ref =
        material.context.statement_ref.as_deref().ok_or("capture context has no statement reference")?;
    let error =
        replay_mantle_artifact_auth_operational_receipt(state_dir, &material.keypair, &material.input(), statement_ref)
            .expect_err("revoked product trust must deny replay");
    write_json(
        &evidence_dir.join("revocation-summary.json"),
        &serde_json::json!({
            "schema": "mantle-artifact-auth-canary-summary-v1",
            "phase": "revocation",
            "result": "expected-denial",
            "error": error.to_string(),
            "standalone_authority_admitted": false,
            "rollback_available": true
        }),
    )?;
    Ok(())
}

fn load_material(state_dir: &Path, evidence_dir: &Path) -> Result<Material, Box<dyn Error>> {
    let keypair = read_keypair(state_dir)?;
    let signed_record = read_single_signed_record(state_dir)?;
    let key_name = keypair.verifying_key.name().to_owned();
    let legacy = candidate_for(&signed_record, &key_name)?;
    let key_identity_blake3 = key_identity(&keypair.verifying_key.to_string())?;
    let build_report = std::fs::read(evidence_dir.join("build.json"))?;
    let policy_hash = blake3::hash(&build_report).to_hex().to_string();
    let revocation_bytes = std::fs::read(state_dir.join(REVOCATION_FILE))?;
    let revocation_ref = format!("blake3:{}", blake3::hash(&revocation_bytes).to_hex());
    let revoked_public_key_digests = serde_json::from_slice(&revocation_bytes)?;
    let context = serde_json::from_slice(&std::fs::read(state_dir.join(CONTEXT_FILE))?)?;
    Ok(Material {
        trusted_public_keys: vec![keypair.verifying_key.clone()],
        keypair,
        signed_record,
        legacy,
        key_identity_blake3,
        policy_hash,
        revocation_ref,
        revoked_public_key_digests,
        context,
    })
}

fn candidate_for(record: &SignedActionResultRecord, key_name: &str) -> Result<CandidateDecision, Box<dyn Error>> {
    let outputs = serde_json::to_vec(&record.record.outputs)?;
    Ok(CandidateDecision {
        result_ref: record.record.result_ref.clone(),
        source_id: "local-canary-build".to_owned(),
        source_class: "product-action-result-state".to_owned(),
        admitted: true,
        diagnostics: vec![format!("record-signature-verified:{key_name}")],
        trust_basis: vec![key_name.to_owned()],
        output_set_digest_blake3: Some(blake3::hash(&outputs).to_hex().to_string()),
    })
}

fn key_identity(token: &str) -> Result<String, Box<dyn Error>> {
    let (_, encoded) = token.split_once(':').ok_or("public key token has no separator")?;
    let bytes = BASE64.decode(encoded.as_bytes())?;
    let public_key: [u8; ED25519_PUBLIC_KEY_BYTES] = bytes.try_into().map_err(|_| "unexpected public key length")?;
    Ok(public_key_identity(&public_key).digest_hex)
}

fn read_keypair(state_dir: &Path) -> Result<KeyPair, Box<dyn Error>> {
    let encoded = std::fs::read_to_string(state_dir.join(SIGNING_KEY_FILE))?;
    Ok(load_keypair(encoded.trim())?)
}

fn read_single_signed_record(state_dir: &Path) -> Result<SignedActionResultRecord, Box<dyn Error>> {
    let directory = state_dir.join(RECORD_DIRECTORY);
    let mut records = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
        .collect::<Vec<_>>();
    records.sort();
    if records.len() != 1 {
        return Err(format!("expected exactly one action-result record, found {}", records.len()).into());
    }
    Ok(serde_json::from_slice(&std::fs::read(&records[0])?)?)
}

fn initialize_revocations(state_dir: &Path) -> Result<(), Box<dyn Error>> {
    let path = state_dir.join(REVOCATION_FILE);
    if path.exists() {
        return Err("canary revocation state already exists".into());
    }
    write_json(&path, &Vec::<String>::new())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn Error>> {
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
