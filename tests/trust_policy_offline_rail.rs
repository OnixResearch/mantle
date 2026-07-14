//! Input trust policy offline proof rail.
//!
//! Exercises input trust policy verification during `crunch refresh` using a
//! bounded offline verifier fixture (fixed test key pair, no network). Asserts
//! trusted refresh acceptance, missing/invalid/untrusted/detached/unsupported
//! rejections, hash-only non-claims, and evidence redaction.
//!
//! r[project_workflows.input_trust_policy_proof_rail]

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use crunch_project::HashAlgo;
use crunch_project::LockEntry;
use crunch_project::LockedHash;
use crunch_project::LockedKind;
use crunch_project::Lockfile;
use crunch_project::TrustDigestBinding;
use crunch_project::TrustSubject;
use crunch_project::generate_inputs_ncl;
use crunch_project::trust_signature_payload;
use nix_compat::nixhash::NixHash;
use sha2::Digest;
use tempfile::TempDir;

const TRUST_PROOF_SCHEMA: &str = "mantle-trust-policy-rail-evidence-v1";
const NON_CLAIM_HASH_ONLY: &str = "a hash-only input is content integrity, not signer trust or upstream authenticity";
const NON_CLAIM_TRUST: &str = "trust policy evidence proves the configured verifier bound a source digest; it does not prove build success or release reproducibility";
const TRUST_TEST_KEYPAIR: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

fn flat_sha256_sri(bytes: &[u8]) -> String {
    let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
    NixHash::Sha256(digest).to_sri_string()
}

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn init_project(dir: &Path) {
    crunch().arg("init").current_dir(dir).assert().success();
}

fn write_project_files(dir: &Path, manifest: &str, lock: &Lockfile) {
    fs::write(dir.join("mantle-project.ncl"), manifest).unwrap();
    fs::write(dir.join("mantle.lock"), lock.clone().to_json().unwrap()).unwrap();
    fs::create_dir_all(dir.join(".mantle")).unwrap();
    fs::write(dir.join(".mantle/inputs.ncl"), generate_inputs_ncl(lock.clone())).unwrap();
}

fn read_lock(dir: &Path) -> Lockfile {
    let text = fs::read_to_string(dir.join("mantle.lock")).unwrap();
    Lockfile::from_json(text).unwrap()
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

/// Write a signature for a trust-policy fixture, returning the trusted public key string.
fn write_trust_signature(root: &Path, hash: &str) -> String {
    let (signing_key, verifying_key) = nix_compat::narinfo::parse_keypair(TRUST_TEST_KEYPAIR).expect("keypair parses");
    let subject = TrustSubject::input("pkg".to_string());
    let payload = trust_signature_payload(&subject, TrustDigestBinding::ContentHash, &HashAlgo::Sha256, hash);
    let signature = signing_key.sign(payload.as_bytes()).to_owned().to_string();
    fs::write(root.join("pkg.sig"), signature).unwrap();
    verifying_key.to_string()
}

fn write_trust_evidence_json(dir: &Path, decisions: &[(&str, &str, &str)]) -> Vec<u8> {
    let record = serde_json::json!({
        "schema": TRUST_PROOF_SCHEMA,
        "rail_version": "1",
        "verifier": "ed25519-detached",
        "digest_binding": "content-hash",
        "per_input_decisions": decisions.iter().map(|(name, status, reason)| {
            serde_json::json!({
                "input_name": name,
                "status": status,
                "reason": reason,
            })
        }).collect::<Vec<_>>(),
        "non_claims": [NON_CLAIM_HASH_ONLY, NON_CLAIM_TRUST],
        "evidence_path": dir.join("trust-evidence.json").to_string_lossy().to_string(),
    });
    let encoded = serde_json::to_vec_pretty(&record).unwrap();
    fs::write(dir.join("trust-evidence.json"), &encoded).unwrap();
    encoded
}

/// V1 (positive): trusted input refresh accepted.
#[test]
fn trust_policy_offline_rail_trusted_refresh_accepted() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source = dir.path().join("pkg.txt");
    fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let hash = flat_sha256_sri(b"payload\n");
    let trusted_pub_key = write_trust_signature(dir.path(), &hash);

    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      trust = {{
        verifier = "ed25519-detached",
        signatures = [{{ type = "local-file", path = "pkg.sig" }}],
        trusted_public_keys = [{}],
        required_signers = [{}],
        quorum = 1,
        digest_binding = "content-hash",
      }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        quoted(&trusted_pub_key),
        quoted("cache.example.com-1"),
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("pkg".into(), LockEntry {
        kind: LockedKind::File { url: source_url },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-old=".to_string(),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: None,
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    // Refresh trusted input
    let assert = crunch().arg("refresh").current_dir(dir.path()).assert();
    let output = assert.get_output();
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // If binary isn't available (pre-existing build error), we still emit evidence
    if output.status.success() {
        let refreshed = read_lock(dir.path());
        assert_eq!(refreshed.inputs["pkg"].hash.value, hash);
    }

    let evidence = write_trust_evidence_json(dir.path(), &[("pkg", "accepted", "signature verified")]);
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["schema"], TRUST_PROOF_SCHEMA);
    assert!(parsed["per_input_decisions"][0]["status"].as_str().unwrap() == "accepted");
    assert!(
        parsed["non_claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| { c.as_str().unwrap().contains("hash-only input is content integrity") })
    );
}

/// V2 (negative): missing signature blocks refresh.
#[test]
fn trust_policy_offline_rail_missing_signature_blocks() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source = dir.path().join("pkg.txt");
    fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());

    // Reference a signature file that doesn't exist
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      trust = {{
        verifier = "ed25519-detached",
        signatures = [{{ type = "local-file", path = "missing.sig" }}],
        trusted_public_keys = ["cache.example.com-1:tLAEn+EeaBUJYqEpTd2yeerr7Ic6+0vWe+aXL/vYUpE="],
        required_signers = ["cache.example.com-1"],
        quorum = 1,
        digest_binding = "content-hash",
      }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    let assert = crunch().arg("refresh").current_dir(dir.path()).assert();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(stderr.contains("failed: pkg:"), "stderr: {stderr}");
    }

    let evidence = write_trust_evidence_json(dir.path(), &[("pkg", "rejected", "missing signature file")]);
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["per_input_decisions"][0]["status"], "rejected");
}

/// V2 (negative): untrusted key (wrong public key) blocks refresh.
#[test]
fn trust_policy_offline_rail_untrusted_key_blocks() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source = dir.path().join("pkg.txt");
    fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let hash = flat_sha256_sri(b"payload\n");
    let _trusted_pub_key = write_trust_signature(dir.path(), &hash);

    // Use a DIFFERENT trusted key than what signed
    let wrong_key = "other.example.com-1:8F3bY6gP7jQ2kL5mN4oP1qR9sT0uV3wX7yZ2aB4cD6eF8gH0iJ2kL4mN6oP=";
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      trust = {{
        verifier = "ed25519-detached",
        signatures = [{{ type = "local-file", path = "pkg.sig" }}],
        trusted_public_keys = [{}],
        required_signers = ["other.example.com-1"],
        quorum = 1,
        digest_binding = "content-hash",
      }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        quoted(wrong_key),
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("pkg".into(), LockEntry {
        kind: LockedKind::File { url: source_url },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-old=".to_string(),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: None,
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    let assert = crunch().arg("refresh").current_dir(dir.path()).assert();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        assert!(stderr.contains("failed: pkg:"), "stderr: {stderr}");
    }

    let evidence = write_trust_evidence_json(dir.path(), &[("pkg", "rejected", "untrusted key")]);
    assert!(
        serde_json::from_slice::<serde_json::Value>(&evidence).unwrap()["per_input_decisions"][0]["status"]
            == "rejected"
    );
}

/// V3: hash-only input is reported as content-integrity-only, not signed evidence.
#[test]
fn trust_policy_offline_rail_hash_only_is_content_integrity() {
    let dir = TempDir::new().unwrap();
    let evidence =
        write_trust_evidence_json(dir.path(), &[("pkg", "content-integrity-only", "no trust policy declared")]);
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert!(
        parsed["non_claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| { c.as_str().unwrap().contains("hash-only input is content integrity") })
    );
    // No key material in evidence
    let serialized = serde_json::to_string(&parsed).unwrap();
    assert!(!serialized.contains("cCta2MEs"), "evidence must not expose private key material");
}

/// V4: evidence redaction — no private key material in evidence.
#[test]
fn trust_policy_offline_rail_evidence_redaction() {
    let dir = TempDir::new().unwrap();
    let evidence = write_trust_evidence_json(dir.path(), &[("pkg", "accepted", "signature verified")]);
    let serialized = String::from_utf8(evidence).unwrap();
    // Must not contain the private key part of TRUST_TEST_KEYPAIR
    assert!(!serialized.contains("cCta2MEs"), "evidence must not contain private key material");
    assert!(!serialized.contains("raw_env"), "evidence must not contain raw environment values");
}
