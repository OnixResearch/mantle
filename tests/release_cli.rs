use std::fs::File;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::process::Output;

use assert_cmd::Command;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use crunch_attestation::AttestationDigest;
use crunch_attestation::Canonicalize;
use crunch_attestation::DetachedSignature;
use crunch_attestation::RebuildEnvironmentSummary;
use crunch_attestation::ReleaseAttestation;
use crunch_attestation::ReleasePolicy;
use crunch_attestation::ReleaseRevocations;
use crunch_attestation::WitnessAttestation;
use crunch_attestation::encode_detached_signature;
use crunch_build::KeyPair;
use crunch_build::load_keypair;
use predicates::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use tempfile::TempDir;

const RELEASE_EVIDENCE_SCHEMA: &str = "crunch-release-evidence-v1";
const BLAKE3_HEX_LEN: usize = 64;
const RELEASE_SIGNING_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

fn run_git(repo_root: &Path, args: &[&str]) -> Output {
    ProcessCommand::new("git").args(args).current_dir(repo_root).output().unwrap()
}

fn assert_git_ok(repo_root: &Path, args: &[&str]) {
    let output = run_git(repo_root, args);
    assert!(
        output.status.success(),
        "git {:?} failed: stdout={} stderr={}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn create_minimal_release_repo(repo_root: &Path) {
    write_file(&repo_root.join(".cargo/vendor-config.toml"), b"directory = \"vendor-deps\"\n");
    write_file(&repo_root.join("vendor-deps/dep/Cargo.toml"), b"[package]\nname=\"dep\"\nversion=\"0.1.0\"\n");
    write_file(&repo_root.join("Cargo.toml"), b"[package]\nname=\"demo\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    write_file(&repo_root.join("Cargo.lock"), b"# lock\n");
    write_file(&repo_root.join("bootstrap/seed.ncl"), b"{}\n");
    write_file(&repo_root.join("builders/default.ncl"), b"{}\n");
    write_file(&repo_root.join("crates/demo/src/lib.rs"), b"pub fn demo() {}\n");
    write_file(&repo_root.join("lib/lib.ncl"), b"{}\n");
    write_file(&repo_root.join("rust-toolchain.toml"), b"[toolchain]\nchannel=\"nightly\"\n");
    write_file(&repo_root.join("src/main.rs"), b"fn main() { println!(\"tracked\"); }\n");
    write_file(&repo_root.join("vendor/README"), b"vendor\n");

    assert_git_ok(repo_root, &["init"]);
    assert_git_ok(repo_root, &["config", "user.email", "pi@example.com"]);
    assert_git_ok(repo_root, &["config", "user.name", "Pi"]);
    assert_git_ok(repo_root, &["add", "."]);
    assert_git_ok(repo_root, &["commit", "-m", "initial"]);
}

fn write_fake_proof_bundle(proof_dir: &Path) {
    write_file(
        proof_dir.join("manifest.json").as_path(),
        serde_json::to_vec(&serde_json::json!({ "schema": "fake-proof" })).unwrap().as_slice(),
    );
}

fn write_full_proof_bundle(proof_dir: &Path, stage2_digest_blake3: &str, inventory_digest_blake3: &str) {
    let manifest = serde_json::json!({
        "schema": "crunch-self-hosting-proof-v2",
        "staged_source": "/tmp/proof-store/abcd-crunch-src",
        "prerequisites": {
            "mode": "fixed-point",
            "inventory_doc": {
                "path": "/tmp/proof-bundle/stage0-prerequisites/inventory.md",
                "size_bytes": 9,
                "digest_blake3": inventory_digest_blake3
            }
        },
        "binaries": {
            "stage1": {
                "path": "/tmp/proof-store/stage1-crunch/bin/crunch",
                "size_bytes": 20,
                "digest_blake3": sample_digest(1)
            },
            "stage2": {
                "path": "/tmp/proof-store/stage2-crunch/bin/crunch",
                "size_bytes": 13,
                "digest_blake3": stage2_digest_blake3
            }
        },
        "tools": {
            "stage0_bwrap": {
                "path": "/tmp/proof-store/stage0-bwrap/bin/bwrap",
                "size_bytes": 22,
                "digest_blake3": sample_digest(2)
            },
            "stage0_busybox": {
                "path": "/tmp/proof-store/stage0-busybox/bin/busybox",
                "size_bytes": 23,
                "digest_blake3": sample_digest(3)
            },
            "stage2_bwrap": {
                "path": "/tmp/proof-store/stage2-bwrap/bin/bwrap",
                "size_bytes": 24,
                "digest_blake3": sample_digest(4)
            },
            "stage2_busybox": {
                "path": "/tmp/proof-store/stage2-busybox/bin/busybox",
                "size_bytes": 25,
                "digest_blake3": sample_digest(5)
            }
        },
        "fixed_point": {
            "stage1_equals_stage2": true,
            "stage0_bwrap_equals_stage2_bwrap": true,
            "stage0_busybox_equals_stage2_busybox": true
        },
        "stage0": {
            "report": {
                "staged_source": "/tmp/proof-store/abcd-crunch-src",
                "output_binary": "/tmp/proof-store/stage1-crunch/bin/crunch",
                "busybox_path": "/tmp/proof-store/stage0-busybox/bin/busybox"
            }
        },
        "stage2": {
            "report": {
                "staged_source": "/tmp/proof-store/abcd-crunch-src",
                "output_binary": "/tmp/proof-store/stage2-crunch/bin/crunch",
                "busybox_path": "/tmp/proof-store/stage2-busybox/bin/busybox"
            }
        }
    });
    write_file(proof_dir.join("manifest.json").as_path(), serde_json::to_vec(&manifest).unwrap().as_slice());
    write_file(proof_dir.join("stage0-prerequisites/inventory.md").as_path(), b"inventory");
    write_file(proof_dir.join("summary.txt").as_path(), b"summary\n");
    write_file(proof_dir.join("stage0/stdout.txt").as_path(), b"stage0 stdout\n");
}

fn sample_digest(seed: u8) -> String {
    let nibble = format!("{:x}", seed % 16);
    nibble.repeat(BLAKE3_HEX_LEN)
}

fn make_valid_bundle() -> (TempDir, PathBuf, ReleaseEvidenceManifest) {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());

    let binary_path = temp.path().join("crunch-bin");
    write_file(&binary_path, b"crunch-binary");
    let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
    let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &stage2_digest, &inventory_digest);

    let bundle_dir = temp.path().join("bundle");
    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("crunch-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("release id: crunch-0.1.0-rc1"));

    let manifest: ReleaseEvidenceManifest =
        serde_json::from_slice(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).unwrap();
    (temp, bundle_dir, manifest)
}

fn release_keypair() -> KeyPair {
    load_keypair(RELEASE_SIGNING_KEY).unwrap()
}

fn write_release_signing_key(path: &Path) {
    write_file(path, format!("{RELEASE_SIGNING_KEY}\n").as_bytes());
}

fn write_generated_signing_key(path: &Path) -> KeyPair {
    let (keypair, key_line) = crunch_build::generate_keypair();
    write_file(path, format!("{key_line}\n").as_bytes());
    keypair
}

fn trusted_public_key_from_default_config(current_dir: &Path, config_dir: &Path) -> String {
    let output = crunch()
        .current_dir(current_dir)
        .env("CRUNCH_CONFIG_DIR", config_dir)
        .arg("attest")
        .arg("key-show")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn trusted_public_key_key_name(trusted_public_key: &str) -> &str {
    trusted_public_key
        .split_once(':')
        .map(|(key_name, _key_material)| key_name)
        .unwrap_or_else(|| panic!("trusted public key must contain a ':' separator: {trusted_public_key}"))
}

fn read_release_attestation(verification_dir: &Path) -> ReleaseAttestation {
    serde_json::from_slice(&std::fs::read(verification_dir.join("release-attestation.json")).unwrap()).unwrap()
}

fn read_policy(verification_dir: &Path) -> ReleasePolicy {
    serde_json::from_slice(&std::fs::read(verification_dir.join("policy.json")).unwrap()).unwrap()
}

fn read_revocations(verification_dir: &Path) -> ReleaseRevocations {
    serde_json::from_slice(&std::fs::read(verification_dir.join("revocations.json")).unwrap()).unwrap()
}

fn write_policy(
    verification_dir: &Path,
    release_signer_name: &str,
    trusted_witness_identities: Vec<String>,
    min_matching_witnesses: u32,
) {
    let policy = ReleasePolicy::new(
        min_matching_witnesses,
        "witness_identity".to_string(),
        vec![release_signer_name.to_string()],
        trusted_witness_identities,
    );
    write_file(&verification_dir.join("policy.json"), serde_json::to_vec_pretty(&policy).unwrap().as_slice());
}

fn write_empty_revocations(verification_dir: &Path) {
    let revocations = ReleaseRevocations::empty();
    write_file(
        &verification_dir.join("revocations.json"),
        serde_json::to_vec_pretty(&revocations).unwrap().as_slice(),
    );
}

fn write_witness_material(
    verification_dir: &Path,
    release_attestation: &ReleaseAttestation,
    witness_keypair: &KeyPair,
    identity: &str,
) {
    write_witness_material_with_release_digest(
        verification_dir,
        release_attestation,
        release_attestation.canonical_digest().unwrap(),
        witness_keypair,
        identity,
    );
}

fn write_witness_material_with_release_digest(
    verification_dir: &Path,
    release_attestation: &ReleaseAttestation,
    release_attestation_digest: AttestationDigest,
    witness_keypair: &KeyPair,
    identity: &str,
) {
    let witness = WitnessAttestation::new(
        release_attestation_digest,
        identity.to_string(),
        release_attestation.binary_digests.clone(),
        RebuildEnvironmentSummary {
            system: "x86_64-linux".to_string(),
            toolchain: "rust-1.91.1".to_string(),
            host_class: "nixos-25.05".to_string(),
        },
    );
    let witness_bytes = witness.canonical_bytes().unwrap();
    write_file(&verification_dir.join("witnesses").join(format!("{identity}.json")), witness_bytes.as_slice());

    let signature = witness_keypair.signing_key.sign(&witness_bytes);
    let detached_signature = DetachedSignature {
        key_name: signature.name().to_string(),
        signature_bytes: *signature.bytes(),
    };
    write_file(
        &verification_dir.join("witnesses").join(format!("{identity}.json.sig")),
        format!("{}\n", encode_detached_signature(&detached_signature)).as_bytes(),
    );
}

fn invalid_signature_line(key_name: &str) -> String {
    format!("{}:{}", key_name, BASE64_STANDARD.encode([0_u8; 64]))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WitnessRequestJson {
    schema: String,
    request_layout_version: u32,
    release_id: String,
    release_bundle_relative_path: String,
    verification_seed_relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WitnessRebuildAuditJson {
    schema: String,
    release_id: String,
    started_unix_ms: u64,
    finished_unix_ms: u64,
    status: String,
    rebuilt_outputs: Vec<WitnessRebuildAuditOutputJson>,
    witness_attestation_path: Option<String>,
    witness_signature_path: Option<String>,
    failure_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WitnessRebuildAuditOutputJson {
    published_name: String,
    path: String,
    digest_blake3: String,
}

fn read_witness_request(request_dir: &Path) -> WitnessRequestJson {
    serde_json::from_slice(&std::fs::read(request_dir.join("request.json")).unwrap()).unwrap()
}

fn read_witness_rebuild_audit(audit_path: &Path) -> WitnessRebuildAuditJson {
    serde_json::from_slice(&std::fs::read(audit_path).unwrap()).unwrap()
}

fn write_fake_witness_rebuild_driver(path: &Path) {
    let script = r#"#!/usr/bin/env bash
set -euo pipefail
readonly DRIVER_SLEEP_SECONDS=0.1
: "${CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR:?}"
: "${CRUNCH_WITNESS_RELEASE_BUNDLE_DIR:?}"
mode="${CRUNCH_TEST_WITNESS_DRIVER_MODE:-match}"
sentinel="${CRUNCH_TEST_WITNESS_DRIVER_SENTINEL:-}"
if [[ -n "$sentinel" ]]; then
  printf 'launched\n' > "$sentinel"
fi
mkdir -p "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR"
printf 'driver mode: %s\n' "$mode"
sleep "$DRIVER_SLEEP_SECONDS"
output_path="$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/stage2-crunch"
case "$mode" in
  match)
    cp "$CRUNCH_WITNESS_RELEASE_BUNDLE_DIR/binaries/01-crunch-bin" "$output_path"
    ;;
  mismatch)
    printf 'wrong-binary\n' > "$output_path"
    ;;
  fail)
    printf 'driver failure\n' >&2
    exit 7
    ;;
  *)
    printf 'unknown driver mode: %s\n' "$mode" >&2
    exit 9
    ;;
esac
printf '{"stage2":{"report":{"output_binary":"%s"}}}\n' "$output_path" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/manifest.json"
"#;
    write_file(path, script.as_bytes());
    #[cfg(unix)]
    {
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

fn rewrite_request_bundle_manifest(request_dir: &Path, rewrite: impl FnOnce(&mut ReleaseEvidenceManifest)) {
    let request = read_witness_request(request_dir);
    let manifest_path = request_dir.join(&request.release_bundle_relative_path).join("manifest.json");
    let mut manifest: ReleaseEvidenceManifest =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    rewrite(&mut manifest);
    write_file(&manifest_path, &serde_json::to_vec(&manifest).unwrap());
}

fn rewrite_request_release_attestation(request_dir: &Path, rewrite: impl FnOnce(&mut ReleaseAttestation)) {
    let request = read_witness_request(request_dir);
    let attestation_path = request_dir.join(&request.verification_seed_relative_path).join("release-attestation.json");
    let mut attestation: ReleaseAttestation =
        serde_json::from_slice(&std::fs::read(&attestation_path).unwrap()).unwrap();
    rewrite(&mut attestation);
    write_file(&attestation_path, &serde_json::to_vec(&attestation).unwrap());
}

fn witness_rebuild_helper_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/rebuild-witness-request.sh")
}

fn copy_directory_for_test(source_dir: &Path, dest_dir: &Path) {
    std::fs::create_dir_all(dest_dir).unwrap();
    copy_directory_entries_for_test(source_dir, dest_dir);
}

fn copy_directory_entries_for_test(source_dir: &Path, dest_dir: &Path) {
    let mut entries = std::fs::read_dir(source_dir).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
    entries.sort();
    for source_path in entries {
        let relative = source_path.strip_prefix(source_dir).unwrap();
        let dest_path = dest_dir.join(relative);
        if source_path.is_dir() {
            std::fs::create_dir_all(&dest_path).unwrap();
            copy_directory_entries_for_test(&source_path, &dest_path);
            continue;
        }
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::copy(&source_path, &dest_path).unwrap();
    }
}

#[test]
fn release_create_fails_when_proof_bundle_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());

    let binary_path = temp.path().join("crunch-bin");
    write_file(&binary_path, b"crunch-binary");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("crunch-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(temp.path().join("bundle"))
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(temp.path().join("missing-proof"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("proof bundle directory does not exist"));
}

#[test]
fn release_create_rejects_prerequisite_only_proof_bundle() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());

    let binary_path = temp.path().join("crunch-bin");
    write_file(&binary_path, b"crunch-binary");
    let proof_dir = temp.path().join("fake-proof");
    write_fake_proof_bundle(&proof_dir);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("crunch-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(temp.path().join("bundle"))
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("full proof artifact required"));
}

#[test]
fn release_verify_fails_when_prerequisite_inventory_is_missing() {
    let (_temp, bundle_dir, manifest) = make_valid_bundle();
    std::fs::remove_file(bundle_dir.join(&manifest.prerequisite_inventory.relative_path)).unwrap();

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("prerequisite_inventory"))
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn release_verify_fails_when_source_archive_is_missing() {
    let (_temp, bundle_dir, manifest) = make_valid_bundle();
    std::fs::remove_file(bundle_dir.join(&manifest.source_archive.relative_path)).unwrap();

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("source_archive"))
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn release_verify_fails_when_binary_artifact_is_missing() {
    let (_temp, bundle_dir, manifest) = make_valid_bundle();
    std::fs::remove_file(bundle_dir.join(&manifest.binaries[0].relative_path)).unwrap();

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("binaries[0]"))
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn release_verify_fails_when_proof_bundle_directory_is_missing() {
    let (_temp, bundle_dir, manifest) = make_valid_bundle();
    std::fs::remove_dir_all(bundle_dir.join(&manifest.proof_bundle.relative_path)).unwrap();

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("proof_bundle"))
        .stderr(predicate::str::contains("expected directory artifact"));
}

#[test]
fn release_verify_rejects_fake_proof_bundle_even_if_manifest_digest_matches() {
    let (_temp, bundle_dir, mut manifest) = make_valid_bundle();
    let fake_proof_manifest = serde_json::json!({ "schema": "fake-proof" });
    let proof_manifest_path = bundle_dir.join("proof/self-hosting/manifest.json");
    write_file(&proof_manifest_path, serde_json::to_vec(&fake_proof_manifest).unwrap().as_slice());

    let proof_dir = bundle_dir.join(&manifest.proof_bundle.relative_path);
    let (proof_size_bytes, proof_digest_blake3) = hash_directory(&proof_dir).unwrap();
    manifest.proof_bundle.size_bytes = proof_size_bytes;
    manifest.proof_bundle.digest_blake3 = proof_digest_blake3;
    manifest.proof_linkage.proof_manifest_digest_blake3 =
        blake3::hash(&std::fs::read(&proof_manifest_path).unwrap()).to_hex().to_string();
    write_canonical_manifest(&bundle_dir, &manifest);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("full proof artifact required"));
}

#[test]
fn release_verify_succeeds_using_bundle_local_contents_only() {
    let (_temp, bundle_dir, manifest) = make_valid_bundle();

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("release id: {}", manifest.release_id)))
        .stdout(predicate::str::contains(format!("binaries: {}", manifest.binaries.len())))
        .stdout(predicate::str::contains(format!("source digest: {}", manifest.source_archive.digest_blake3)))
        .stdout(predicate::str::contains(format!(
            "stage2 digest: {}",
            manifest.proof_linkage.stage2_binary_digest_blake3
        )));
}

#[test]
fn release_verify_rejects_manifest_schema_mismatch() {
    let (_temp, bundle_dir, mut manifest) = make_valid_bundle();
    manifest.schema = "crunch-release-evidence-v999".to_string();
    write_canonical_manifest(&bundle_dir, &manifest);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("release evidence schema must be"));
}

#[test]
fn release_verify_rejects_missing_workflow_provenance() {
    let (_temp, bundle_dir, mut manifest) = make_valid_bundle();
    manifest.workflow.command.clear();
    write_canonical_manifest(&bundle_dir, &manifest);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("workflow.command must not be empty"));
}

#[test]
fn release_verify_rejects_claim_boundary_violation() {
    let (_temp, bundle_dir, mut manifest) = make_valid_bundle();
    manifest.claim_scope = "full-source-bootstrap-proof".to_string();
    write_canonical_manifest(&bundle_dir, &manifest);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("claim_scope must be packaged-integrity-evidence"));
}

#[test]
fn release_verify_rejects_proof_linkage_source_digest_mismatch() {
    let (_temp, bundle_dir, mut manifest) = make_valid_bundle();
    manifest.proof_linkage.source_archive_digest_blake3 = sample_digest(7);
    write_canonical_manifest(&bundle_dir, &manifest);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("source archive digest does not match bundled source archive"));
}

#[test]
fn release_attest_writes_signed_release_attestation_and_default_dir() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let expected_verification_dir = temp.path().join("target/release-verification").join(&manifest.release_id);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "release attestation: {}",
            expected_verification_dir.join("release-attestation.json").display()
        )))
        .stdout(predicate::str::contains(format!("release id: {}", manifest.release_id)));

    let attestation = read_release_attestation(&expected_verification_dir);
    let manifest_digest = blake3::hash(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).to_hex().to_string();

    assert_eq!(attestation.release_id, manifest.release_id);
    assert_eq!(attestation.release_evidence_manifest_digest_blake3.to_hex(), manifest_digest,);
    assert_eq!(attestation.proof_bundle_digest_blake3.to_hex(), manifest.proof_bundle.digest_blake3);
    assert_eq!(attestation.binary_digests.len(), 1);
    assert_eq!(attestation.binary_digests[0].name, manifest.binaries[0].relative_path);
    assert!(expected_verification_dir.join("release-attestation.json.sig").exists());
}

#[test]
fn release_attest_honors_json_output() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification-json");

    let output = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(json["release_id"], "crunch-0.1.0-rc1");
    assert_eq!(json["attestation_path"], verification_dir.join("release-attestation.json").display().to_string());
    assert_eq!(json["signature_path"], verification_dir.join("release-attestation.json.sig").display().to_string());
}

#[test]
fn attest_key_show_explicit_signing_key_prints_trusted_public_key() {
    let temp = tempfile::tempdir().unwrap();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);

    let output = crunch().arg("attest").arg("key-show").arg("--signing-key").arg(&signing_key_path).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), release_keypair().verifying_key.to_string());
}

#[test]
fn attest_key_show_uses_default_config_signing_key() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let signing_key_path = config_dir.join("signing-key");
    write_release_signing_key(&signing_key_path);

    let output = crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &config_dir)
        .arg("--json")
        .arg("attest")
        .arg("key-show")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(json["kind"], "crunch-trusted-public-key");
    assert_eq!(json["trusted_public_key"], release_keypair().verifying_key.to_string());
    assert_eq!(json["key_name"], release_keypair().verifying_key.name());
    assert_eq!(json["source_path"], signing_key_path.display().to_string());
}

#[test]
fn attest_key_show_missing_signing_key_fails_without_generation() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let signing_key_path = config_dir.join("signing-key");

    crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &config_dir)
        .arg("attest")
        .arg("key-show")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no signing key found"));

    assert!(!signing_key_path.exists());
}

#[test]
fn attest_policy_init_self_proof_only_writes_policy_files() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let output = crunch()
        .arg("--json")
        .arg("attest")
        .arg("policy-init")
        .arg(&verification_dir)
        .arg("--profile")
        .arg("self-proof-only")
        .arg("--trusted-release-signer")
        .arg(release_keypair().verifying_key.name())
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let created_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(created_json["kind"], "crunch-release-policy-init");
    assert_eq!(created_json["profile"], "self-proof-only");
    assert_eq!(created_json["policy_path"], verification_dir.join("policy.json").display().to_string());
    assert_eq!(created_json["revocations_path"], verification_dir.join("revocations.json").display().to_string());

    let policy = read_policy(&verification_dir);
    let revocations = read_revocations(&verification_dir);
    assert_eq!(policy.min_matching_witnesses, 0);
    assert_eq!(policy.independence_field, "witness_identity");
    assert_eq!(policy.trusted_release_signers, vec![release_keypair().verifying_key.name().to_string()]);
    assert!(policy.trusted_witness_signers.is_empty());
    assert_eq!(revocations, ReleaseRevocations::empty());
}

#[test]
fn attest_policy_init_single_witness_writes_policy_files() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .arg("attest")
        .arg("policy-init")
        .arg(&verification_dir)
        .arg("--profile")
        .arg("single-witness")
        .arg("--trusted-release-signer")
        .arg(release_keypair().verifying_key.name())
        .arg("--trusted-witness-identity")
        .arg("witness-a")
        .assert()
        .success()
        .stdout(predicate::str::contains("profile: single-witness"))
        .stdout(predicate::str::contains("trusted witness identities: witness-a"));

    let policy = read_policy(&verification_dir);
    let revocations = read_revocations(&verification_dir);
    assert_eq!(policy.min_matching_witnesses, 1);
    assert_eq!(policy.independence_field, "witness_identity");
    assert_eq!(policy.trusted_release_signers, vec![release_keypair().verifying_key.name().to_string()]);
    assert_eq!(policy.trusted_witness_signers, vec!["witness-a".to_string()]);
    assert_eq!(revocations, ReleaseRevocations::empty());
}

#[test]
fn attest_policy_init_rejects_existing_policy_without_force() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .arg("attest")
        .arg("policy-init")
        .arg(&verification_dir)
        .arg("--profile")
        .arg("self-proof-only")
        .arg("--trusted-release-signer")
        .arg(release_keypair().verifying_key.name())
        .assert()
        .success();

    let policy_before = std::fs::read(verification_dir.join("policy.json")).unwrap();
    let revocations_before = std::fs::read(verification_dir.join("revocations.json")).unwrap();

    crunch()
        .arg("attest")
        .arg("policy-init")
        .arg(&verification_dir)
        .arg("--profile")
        .arg("single-witness")
        .arg("--trusted-release-signer")
        .arg(release_keypair().verifying_key.name())
        .arg("--trusted-witness-identity")
        .arg("witness-a")
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite existing"));

    let policy_after = std::fs::read(verification_dir.join("policy.json")).unwrap();
    let revocations_after = std::fs::read(verification_dir.join("revocations.json")).unwrap();
    assert_eq!(policy_before, policy_after);
    assert_eq!(revocations_before, revocations_after);
}

#[test]
fn witness_export_writes_request_directory() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let export_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let request = read_witness_request(&request_dir);

    assert_eq!(export_json["kind"], "crunch-witness-request");
    assert_eq!(export_json["release_id"], manifest.release_id);
    assert_eq!(request.schema, "crunch-witness-request-v1");
    assert_eq!(request.request_layout_version, 1);
    assert_eq!(request.release_id, manifest.release_id);
    assert_eq!(request.release_bundle_relative_path, format!("release-evidence/{}", manifest.release_id));
    assert_eq!(request.verification_seed_relative_path, format!("release-verification/{}", manifest.release_id));
    assert!(request_dir.join(&request.release_bundle_relative_path).join("manifest.json").exists());
    assert!(request_dir.join(&request.verification_seed_relative_path).join("release-attestation.json").exists());
    assert!(
        request_dir
            .join(&request.verification_seed_relative_path)
            .join("release-attestation.json.sig")
            .exists()
    );
    assert!(!request_dir.join(&request.verification_seed_relative_path).join("policy.json").exists());
    assert!(!request_dir.join(&request.verification_seed_relative_path).join("revocations.json").exists());
    assert!(!request_dir.join("signing-key").exists());

    crunch()
        .arg("release")
        .arg("verify")
        .arg(request_dir.join(&request.release_bundle_relative_path))
        .assert()
        .success();
}

#[test]
fn witness_export_rejects_release_id_mismatch() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let mut attestation = read_release_attestation(&verification_dir);
    attestation.release_id = "wrong-release".to_string();
    write_file(
        &verification_dir.join("release-attestation.json"),
        serde_json::to_vec(&attestation).unwrap().as_slice(),
    );

    crunch()
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("release id mismatch"));
}

#[test]
fn witness_import_accepts_directory_source_and_skips_exact_duplicates() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let publisher_verification_dir = temp.path().join("publisher-verification");
    let source_verification_dir = temp.path().join("returned-verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&publisher_verification_dir);
    let witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&source_verification_dir, &release_attestation, &witness_keypair, "witness-a");

    crunch()
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(&source_verification_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("imported witness identities: witness-a"));
    assert!(publisher_verification_dir.join("witnesses/witness-a.json").exists());
    assert!(publisher_verification_dir.join("witnesses/witness-a.json.sig").exists());

    let duplicate_output = crunch()
        .arg("--json")
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(source_verification_dir.join("witnesses/witness-a.json"))
        .output()
        .unwrap();
    assert!(duplicate_output.status.success(), "{}", String::from_utf8_lossy(&duplicate_output.stderr));
    let duplicate_json: serde_json::Value = serde_json::from_slice(&duplicate_output.stdout).unwrap();

    assert_eq!(duplicate_json["kind"], "crunch-witness-import");
    assert_eq!(duplicate_json["imported_witness_identities"], serde_json::json!([]));
    assert_eq!(duplicate_json["skipped_duplicate_identities"], serde_json::json!(["witness-a"]));
}

#[test]
fn witness_import_rejects_missing_signature_sidecar() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let publisher_verification_dir = temp.path().join("publisher-verification");
    let source_verification_dir = temp.path().join("returned-verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&publisher_verification_dir);
    let witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&source_verification_dir, &release_attestation, &witness_keypair, "witness-a");
    std::fs::remove_file(source_verification_dir.join("witnesses/witness-a.json.sig")).unwrap();

    crunch()
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(&source_verification_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing witness signature sidecar"));
}

#[test]
fn witness_import_rejects_wrong_release_attestation_digest() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let publisher_verification_dir = temp.path().join("publisher-verification");
    let source_verification_dir = temp.path().join("returned-verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&publisher_verification_dir);
    let witness_keypair = crunch_build::generate_keypair().0;
    let wrong_release_digest = AttestationDigest::from_canonical_bytes(b"wrong-release".to_vec());
    write_witness_material_with_release_digest(
        &source_verification_dir,
        &release_attestation,
        wrong_release_digest,
        &witness_keypair,
        "witness-a",
    );

    crunch()
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(&source_verification_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("references release attestation digest"));
}

#[test]
fn witness_import_rejects_conflicting_duplicate_identity() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let publisher_verification_dir = temp.path().join("publisher-verification");
    let first_source_dir = temp.path().join("returned-first");
    let second_source_dir = temp.path().join("returned-second");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&publisher_verification_dir);
    let first_witness_keypair = crunch_build::generate_keypair().0;
    let second_witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&first_source_dir, &release_attestation, &first_witness_keypair, "witness-a");
    write_witness_material(&second_source_dir, &release_attestation, &second_witness_keypair, "witness-a");

    crunch()
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(&first_source_dir)
        .assert()
        .success();

    crunch()
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(&second_source_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("conflicting witness identity"));
}

#[test]
fn witness_rebuild_cli_check_is_preflight_only() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--scratch-dir")
        .arg(&scratch_dir)
        .arg("--check")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let rebuild_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(rebuild_json["kind"], "crunch-witness-rebuild");
    assert_eq!(rebuild_json["check_only"], true);
    assert_eq!(rebuild_json["release_id"], manifest.release_id);
    assert!(!scratch_dir.exists(), "check mode must not create scratch root");
    let request = read_witness_request(&request_dir);
    assert!(!request_dir.join(&request.verification_seed_relative_path).join("witnesses").exists());
}

#[test]
fn witness_rebuild_helper_check_is_preflight_only() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    let output = ProcessCommand::new(witness_rebuild_helper_path())
        .current_dir(temp.path())
        .arg("--check")
        .arg("--json")
        .arg("--scratch-dir")
        .arg(&scratch_dir)
        .arg(&request_dir)
        .env("CRUNCH_WITNESS_REBUILD_CLI_BIN", env!("CARGO_BIN_EXE_crunch"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let rebuild_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(rebuild_json["kind"], "crunch-witness-rebuild");
    assert_eq!(rebuild_json["check_only"], true);
    assert_eq!(rebuild_json["release_id"], manifest.release_id);
    assert!(!scratch_dir.exists(), "helper check mode must stay preflight-only");
    let request = read_witness_request(&request_dir);
    assert!(!request_dir.join(&request.verification_seed_relative_path).join("witnesses").exists());
}

#[test]
fn witness_rebuild_helper_happy_path_writes_sidecars_and_audit() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    let witness_config_dir = temp.path().join("witness-config");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    write_fake_witness_rebuild_driver(&driver_path);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    let output = ProcessCommand::new(witness_rebuild_helper_path())
        .current_dir(temp.path())
        .arg("--json")
        .arg("--scratch-dir")
        .arg(&scratch_dir)
        .arg(&request_dir)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .env("CRUNCH_CONFIG_DIR", &witness_config_dir)
        .env("CRUNCH_WITNESS_REBUILD_CLI_BIN", env!("CARGO_BIN_EXE_crunch"))
        .env("CRUNCH_WITNESS_REBUILD_DRIVER", &driver_path)
        .env("CRUNCH_TEST_WITNESS_DRIVER_MODE", "match")
        .env("SNIX_BUILD_SANDBOX_SHELL", &driver_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let rebuild_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let audit_path = PathBuf::from(rebuild_json["audit_meta_path"].as_str().unwrap());

    assert_eq!(rebuild_json["kind"], "crunch-witness-rebuild");
    assert_eq!(rebuild_json["check_only"], false);
    assert_eq!(rebuild_json["release_id"], manifest.release_id);
    assert!(Path::new(rebuild_json["attestation_path"].as_str().unwrap()).exists());
    assert!(Path::new(rebuild_json["signature_path"].as_str().unwrap()).exists());
    assert!(audit_path.exists());
    let audit = read_witness_rebuild_audit(&audit_path);
    assert_eq!(audit.status, "success");
    assert_eq!(audit.rebuilt_outputs[0].digest_blake3, manifest.binaries[0].digest_blake3);
}

#[cfg(unix)]
#[test]
fn witness_rebuild_cli_rejects_symlinked_scratch_root_before_launching_driver() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_target = temp.path().join("scratch-target");
    let scratch_link = temp.path().join("scratch-link");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    let sentinel_path = temp.path().join("driver-sentinel");
    write_fake_witness_rebuild_driver(&driver_path);
    std::fs::create_dir_all(&scratch_target).unwrap();
    symlink(&scratch_target, &scratch_link).unwrap();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .env("CRUNCH_WITNESS_REBUILD_DRIVER", &driver_path)
        .env("CRUNCH_TEST_WITNESS_DRIVER_SENTINEL", &sentinel_path)
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--scratch-dir")
        .arg(&scratch_link)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must not be a symlink"));
    assert!(!sentinel_path.exists(), "symlinked scratch root must fail before launching the driver");
}

#[test]
fn witness_rebuild_cli_rejects_request_paths_that_escape_request_dir() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    let mut request = read_witness_request(&request_dir);
    request.release_bundle_relative_path = "../escape".to_string();
    write_file(&request_dir.join("request.json"), serde_json::to_vec(&request).unwrap().as_slice());

    crunch()
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must stay inside the request directory"));
}

#[test]
fn witness_rebuild_cli_rejects_unsupported_workflow_before_running_driver() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    let sentinel_path = temp.path().join("driver-sentinel");
    write_fake_witness_rebuild_driver(&driver_path);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    rewrite_request_bundle_manifest(&request_dir, |manifest| {
        manifest.workflow.command = "./scripts/other-proof.sh".to_string();
    });

    crunch()
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--check")
        .env("CRUNCH_WITNESS_REBUILD_DRIVER", &driver_path)
        .env("CRUNCH_TEST_WITNESS_DRIVER_SENTINEL", &sentinel_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported witness rebuild workflow"));
    assert!(!sentinel_path.exists(), "unsupported workflow must fail before launching the driver");
}

#[test]
fn witness_rebuild_cli_rejects_published_output_name_mismatch_before_running_driver() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    let sentinel_path = temp.path().join("driver-sentinel");
    write_fake_witness_rebuild_driver(&driver_path);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    rewrite_request_release_attestation(&request_dir, |attestation| {
        attestation.binary_digests[0].name = "binaries/99-unexpected".to_string();
    });

    crunch()
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--check")
        .env("CRUNCH_WITNESS_REBUILD_DRIVER", &driver_path)
        .env("CRUNCH_TEST_WITNESS_DRIVER_SENTINEL", &sentinel_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("published output name mismatch"));
    assert!(!sentinel_path.exists(), "output-name mismatch must fail before launching the driver");
}

#[test]
fn witness_rebuild_cli_happy_path_writes_sidecars_and_audit() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    let witness_config_dir = temp.path().join("witness-config");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    write_fake_witness_rebuild_driver(&driver_path);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    let output = crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &witness_config_dir)
        .env("CRUNCH_WITNESS_REBUILD_DRIVER", &driver_path)
        .env("CRUNCH_TEST_WITNESS_DRIVER_MODE", "match")
        .arg("--json")
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--scratch-dir")
        .arg(&scratch_dir)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let rebuild_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let audit_path = PathBuf::from(rebuild_json["audit_meta_path"].as_str().unwrap());
    let returned_verification_dir = PathBuf::from(rebuild_json["verification_dir"].as_str().unwrap());
    let audit = read_witness_rebuild_audit(&audit_path);

    assert_eq!(rebuild_json["kind"], "crunch-witness-rebuild");
    assert_eq!(rebuild_json["check_only"], false);
    assert_eq!(rebuild_json["release_id"], manifest.release_id);
    assert!(Path::new(rebuild_json["attestation_path"].as_str().unwrap()).exists());
    assert!(Path::new(rebuild_json["signature_path"].as_str().unwrap()).exists());
    assert_eq!(audit.schema, "crunch-witness-rebuild-audit-v1");
    assert_eq!(audit.status, "success");
    assert!(
        audit.finished_unix_ms > audit.started_unix_ms,
        "audit timestamps must bracket the rebuild subprocess"
    );
    assert_eq!(audit.rebuilt_outputs.len(), 1);
    assert_eq!(audit.rebuilt_outputs[0].published_name, "binaries/01-crunch-bin");
    assert_eq!(audit.rebuilt_outputs[0].digest_blake3, manifest.binaries[0].digest_blake3);
    assert_eq!(audit.witness_attestation_path.as_deref(), rebuild_json["attestation_path"].as_str());
    assert_eq!(audit.witness_signature_path.as_deref(), rebuild_json["signature_path"].as_str());
    assert!(returned_verification_dir.join("witnesses/witness-a.json").exists());
    let request = read_witness_request(&request_dir);
    assert!(request_dir.join(&request.verification_seed_relative_path).join("release-attestation.json").exists());
    assert!(!request_dir.join(&request.verification_seed_relative_path).join("witnesses").exists());
}

#[test]
fn witness_rebuild_cli_rejects_rebuilt_output_digest_mismatch() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    write_fake_witness_rebuild_driver(&driver_path);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .env("CRUNCH_WITNESS_REBUILD_DRIVER", &driver_path)
        .env("CRUNCH_TEST_WITNESS_DRIVER_MODE", "mismatch")
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--scratch-dir")
        .arg(&scratch_dir)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .assert()
        .failure()
        .stderr(predicate::str::contains("rebuilt output digest mismatch"));

    let audit = read_witness_rebuild_audit(&scratch_dir.join("witness-rebuild-audit/meta.json"));
    assert_eq!(audit.status, "failed");
    assert!(audit.failure_message.unwrap().contains("rebuilt output digest mismatch"));
    assert!(!scratch_dir.join("release-verification").join("crunch-0.1.0-rc1").join("witnesses").exists());
}

#[test]
fn witnessed_self_hosting_rebuild_workflow_reports_quorum_satisfied() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let publisher_verification_dir = temp.path().join("publisher-verification");
    let publisher_config_dir = temp.path().join("publisher-config");
    let witness_config_dir = temp.path().join("witness-config");
    let request_dir = temp.path().join("publisher-request");
    let scratch_dir = temp.path().join("witness-scratch");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    write_fake_witness_rebuild_driver(&driver_path);

    crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &publisher_config_dir)
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .assert()
        .success();

    let release_trusted_key = trusted_public_key_from_default_config(temp.path(), &publisher_config_dir);
    let release_signer_name = trusted_public_key_key_name(&release_trusted_key).to_string();

    crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("policy-init")
        .arg(&publisher_verification_dir)
        .arg("--profile")
        .arg("single-witness")
        .arg("--trusted-release-signer")
        .arg(&release_signer_name)
        .arg("--trusted-witness-identity")
        .arg("witness-a")
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    let rebuild_output = crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &witness_config_dir)
        .env("CRUNCH_WITNESS_REBUILD_DRIVER", &driver_path)
        .env("CRUNCH_TEST_WITNESS_DRIVER_MODE", "match")
        .arg("--json")
        .arg("release")
        .arg("witness-rebuild")
        .arg(&request_dir)
        .arg("--scratch-dir")
        .arg(&scratch_dir)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .output()
        .unwrap();
    assert!(rebuild_output.status.success(), "{}", String::from_utf8_lossy(&rebuild_output.stderr));
    let rebuild_json: serde_json::Value = serde_json::from_slice(&rebuild_output.stdout).unwrap();
    let witness_verification_dir = PathBuf::from(rebuild_json["verification_dir"].as_str().unwrap());
    let witness_trusted_key = trusted_public_key_from_default_config(temp.path(), &witness_config_dir);

    crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(&witness_verification_dir)
        .assert()
        .success();

    let verify_output = crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("release-verify")
        .arg(&publisher_verification_dir)
        .arg("--trusted-public-key")
        .arg(&release_trusted_key)
        .arg("--trusted-public-key")
        .arg(&witness_trusted_key)
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["technical_class"], "external-witness-match");
    assert_eq!(verify_json["policy_status"], "satisfied");
    assert_eq!(verify_json["final_class"], "quorum-satisfied");
}

#[test]
fn cross_machine_witness_handoff_reports_quorum_satisfied() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let publisher_verification_dir = temp.path().join("publisher-verification");
    let publisher_config_dir = temp.path().join("publisher-config");
    let witness_config_dir = temp.path().join("witness-config");
    let request_dir = temp.path().join("publisher-request");
    let witness_workspace = temp.path().join("witness-workspace");
    let witness_request_dir = witness_workspace.join("request");

    crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &publisher_config_dir)
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .assert()
        .success();
    assert!(publisher_config_dir.join("signing-key").exists());

    let release_trusted_key = trusted_public_key_from_default_config(temp.path(), &publisher_config_dir);
    let release_signer_name = trusted_public_key_key_name(&release_trusted_key).to_string();

    crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("policy-init")
        .arg(&publisher_verification_dir)
        .arg("--profile")
        .arg("single-witness")
        .arg("--trusted-release-signer")
        .arg(&release_signer_name)
        .arg("--trusted-witness-identity")
        .arg("witness-a")
        .assert()
        .success();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("witness-export")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&publisher_verification_dir)
        .arg("--request-dir")
        .arg(&request_dir)
        .assert()
        .success();

    copy_directory_for_test(&request_dir, &witness_request_dir);
    let witness_request = read_witness_request(&witness_request_dir);
    assert_eq!(witness_request.release_id, manifest.release_id);

    let witness_bundle_dir = witness_request_dir.join(&witness_request.release_bundle_relative_path);
    let witness_verification_dir = witness_request_dir.join(&witness_request.verification_seed_relative_path);

    crunch()
        .current_dir(&witness_workspace)
        .arg("release")
        .arg("verify")
        .arg(&witness_bundle_dir)
        .assert()
        .success();

    let rebuilt_binary_path = witness_bundle_dir.join("binaries/01-crunch-bin");
    crunch()
        .current_dir(&witness_workspace)
        .env("CRUNCH_CONFIG_DIR", &witness_config_dir)
        .arg("attest")
        .arg("witness-create")
        .arg(&witness_verification_dir)
        .arg("--rebuilt-binary")
        .arg(&rebuilt_binary_path)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .assert()
        .success();
    assert!(witness_config_dir.join("signing-key").exists());

    let witness_trusted_key = trusted_public_key_from_default_config(&witness_workspace, &witness_config_dir);

    crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("witness-import")
        .arg(&publisher_verification_dir)
        .arg(&witness_verification_dir)
        .assert()
        .success();

    let verify_output = crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("release-verify")
        .arg(&publisher_verification_dir)
        .arg("--trusted-public-key")
        .arg(&release_trusted_key)
        .arg("--trusted-public-key")
        .arg(&witness_trusted_key)
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["release_signer_key_name"], release_signer_name);
    assert_eq!(verify_json["technical_class"], "external-witness-match");
    assert_eq!(verify_json["policy_status"], "satisfied");
    assert_eq!(verify_json["final_class"], "quorum-satisfied");
    assert_eq!(verify_json["matching_witness_count"], 1);
}

#[test]
fn witnessed_self_hosting_release_workflow_with_generated_keys_and_key_show_reports_quorum_satisfied() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let verification_dir = temp.path().join("verification");
    let publisher_config_dir = temp.path().join("publisher-config");
    let witness_config_dir = temp.path().join("witness-config");

    crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &publisher_config_dir)
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .assert()
        .success();
    assert!(publisher_config_dir.join("signing-key").exists());

    let release_trusted_key = trusted_public_key_from_default_config(temp.path(), &publisher_config_dir);
    let release_signer_name = trusted_public_key_key_name(&release_trusted_key).to_string();

    crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("policy-init")
        .arg(&verification_dir)
        .arg("--profile")
        .arg("single-witness")
        .arg("--trusted-release-signer")
        .arg(&release_signer_name)
        .arg("--trusted-witness-identity")
        .arg("witness-a")
        .assert()
        .success();

    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-crunch-bin");
    crunch()
        .current_dir(temp.path())
        .env("CRUNCH_CONFIG_DIR", &witness_config_dir)
        .arg("attest")
        .arg("witness-create")
        .arg(&verification_dir)
        .arg("--rebuilt-binary")
        .arg(&rebuilt_binary_path)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .assert()
        .success();
    assert!(witness_config_dir.join("signing-key").exists());

    let witness_trusted_key = trusted_public_key_from_default_config(temp.path(), &witness_config_dir);
    let verify_output = crunch()
        .current_dir(temp.path())
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(&release_trusted_key)
        .arg("--trusted-public-key")
        .arg(&witness_trusted_key)
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["release_signer_key_name"], release_signer_name);
    assert_eq!(verify_json["technical_class"], "external-witness-match");
    assert_eq!(verify_json["policy_status"], "satisfied");
    assert_eq!(verify_json["final_class"], "quorum-satisfied");
    assert_eq!(verify_json["matching_witness_count"], 1);
}

#[test]
fn witnessed_self_hosting_release_workflow_reports_quorum_satisfied() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    crunch()
        .arg("attest")
        .arg("policy-init")
        .arg(&verification_dir)
        .arg("--profile")
        .arg("single-witness")
        .arg("--trusted-release-signer")
        .arg(release_keypair().verifying_key.name())
        .arg("--trusted-witness-identity")
        .arg("witness-a")
        .assert()
        .success();

    let witness_signing_key_path = temp.path().join("witness.key");
    let witness_keypair = write_generated_signing_key(&witness_signing_key_path);
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-crunch-bin");

    crunch()
        .arg("attest")
        .arg("witness-create")
        .arg(&verification_dir)
        .arg("--rebuilt-binary")
        .arg(&rebuilt_binary_path)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .arg("--signing-key")
        .arg(&witness_signing_key_path)
        .assert()
        .success();

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair().verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["technical_class"], "external-witness-match");
    assert_eq!(verify_json["policy_status"], "satisfied");
    assert_eq!(verify_json["final_class"], "quorum-satisfied");
    assert_eq!(verify_json["matching_witness_count"], 1);
}

#[test]
fn attest_witness_create_prints_human_summary_without_json() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let witness_signing_key_path = temp.path().join("witness.key");
    let witness_keypair = write_generated_signing_key(&witness_signing_key_path);
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-crunch-bin");

    crunch()
        .arg("attest")
        .arg("witness-create")
        .arg(&verification_dir)
        .arg("--rebuilt-binary")
        .arg(&rebuilt_binary_path)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .arg("--signing-key")
        .arg(&witness_signing_key_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("witness attestation: "))
        .stdout(predicate::str::contains("witness identity: witness-a"))
        .stdout(predicate::str::contains(format!("signer: {}", witness_keypair.verifying_key.name())));
}

#[test]
fn attest_witness_create_writes_signed_witness_attestation() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let witness_signing_key_path = temp.path().join("witness.key");
    let witness_keypair = write_generated_signing_key(&witness_signing_key_path);
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-crunch-bin");

    let create_output = crunch()
        .arg("--json")
        .arg("attest")
        .arg("witness-create")
        .arg(&verification_dir)
        .arg("--rebuilt-binary")
        .arg(&rebuilt_binary_path)
        .arg("--identity")
        .arg("witness-a")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .arg("--signing-key")
        .arg(&witness_signing_key_path)
        .output()
        .unwrap();
    assert!(create_output.status.success(), "{}", String::from_utf8_lossy(&create_output.stderr));
    let created_json: serde_json::Value = serde_json::from_slice(&create_output.stdout).unwrap();

    assert_eq!(created_json["kind"], "crunch-witness-attestation");
    assert_eq!(created_json["signer"], witness_keypair.verifying_key.name());
    assert_eq!(created_json["stored_path"], verification_dir.join("witnesses/witness-a.json").display().to_string());
    assert_eq!(
        created_json["signature_path"],
        verification_dir.join("witnesses/witness-a.json.sig").display().to_string()
    );

    write_policy(&verification_dir, release_keypair().verifying_key.name(), vec!["witness-a".to_string()], 1);
    write_empty_revocations(&verification_dir);

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair().verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["matching_witness_count"], 1);
    assert_eq!(verify_json["final_class"], "quorum-satisfied");
}

#[test]
fn attest_witness_create_rejects_too_long_identity() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let witness_signing_key_path = temp.path().join("witness.key");
    let _witness_keypair = write_generated_signing_key(&witness_signing_key_path);
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-crunch-bin");
    let long_identity = "w".repeat(129);

    crunch()
        .arg("attest")
        .arg("witness-create")
        .arg(&verification_dir)
        .arg("--rebuilt-binary")
        .arg(&rebuilt_binary_path)
        .arg("--identity")
        .arg(&long_identity)
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .arg("--signing-key")
        .arg(&witness_signing_key_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("witness identity exceeds 128 bytes"));
}

#[test]
fn attest_witness_create_rejects_rebuilt_binary_count_mismatch() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let release_signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&release_signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&release_signing_key_path)
        .assert()
        .success();

    let witness_signing_key_path = temp.path().join("witness.key");
    let _witness_keypair = write_generated_signing_key(&witness_signing_key_path);
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-crunch-bin");
    let extra_binary_path = temp.path().join("extra-bin");
    write_file(&extra_binary_path, b"extra-binary");

    crunch()
        .arg("attest")
        .arg("witness-create")
        .arg(&verification_dir)
        .arg("--rebuilt-binary")
        .arg(&rebuilt_binary_path)
        .arg("--rebuilt-binary")
        .arg(&extra_binary_path)
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .arg("--signing-key")
        .arg(&witness_signing_key_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("rebuilt binary count mismatch"));
}

#[test]
fn attest_release_show_prints_release_attestation_envelope() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let output = crunch().arg("attest").arg("release-show").arg(&verification_dir).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let show_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(show_json["kind"], "crunch-release-attestation");
    assert_eq!(show_json["stored_path"], verification_dir.join("release-attestation.json").display().to_string());
    assert_eq!(show_json["attestation"]["release_id"], "crunch-0.1.0-rc1");
}

#[test]
fn attest_witness_show_and_release_verify_report_quorum_satisfied() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&verification_dir);
    let release_keypair = release_keypair();
    let witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&verification_dir, &release_attestation, &witness_keypair, "witness-a");
    write_policy(&verification_dir, release_keypair.verifying_key.name(), vec!["witness-a".to_string()], 1);
    write_empty_revocations(&verification_dir);

    let show_output =
        crunch().arg("attest").arg("witness-show").arg(&verification_dir).arg("witness-a").output().unwrap();
    assert!(show_output.status.success(), "{}", String::from_utf8_lossy(&show_output.stderr));
    let show_json: serde_json::Value = serde_json::from_slice(&show_output.stdout).unwrap();
    assert_eq!(show_json["kind"], "crunch-witness-attestation");
    assert_eq!(show_json["attestation"]["witness_identity"], "witness-a");

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["release_signer_key_name"], release_keypair.verifying_key.name());
    assert_eq!(verify_json["discovered_witness_count"], 1);
    assert_eq!(verify_json["considered_witness_count"], 1);
    assert_eq!(verify_json["technical_class"], "external-witness-match");
    assert_eq!(verify_json["policy_status"], "satisfied");
    assert_eq!(verify_json["final_class"], "quorum-satisfied");
    assert_eq!(verify_json["matching_witness_count"], 1);
}

#[test]
fn attest_release_verify_skips_witness_signed_by_unknown_key_when_quorum_still_holds() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&verification_dir);
    let release_keypair = release_keypair();
    let trusted_witness_keypair = crunch_build::generate_keypair().0;
    let unknown_witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&verification_dir, &release_attestation, &trusted_witness_keypair, "witness-good");
    write_witness_material(&verification_dir, &release_attestation, &unknown_witness_keypair, "witness-unknown");
    write_policy(
        &verification_dir,
        release_keypair.verifying_key.name(),
        vec!["witness-good".to_string(), "witness-unknown".to_string()],
        1,
    );
    write_empty_revocations(&verification_dir);

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(trusted_witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["discovered_witness_count"], 2);
    assert_eq!(verify_json["considered_witness_count"], 1);
    assert_eq!(verify_json["matching_witness_count"], 1);
    assert_eq!(verify_json["policy_status"], "satisfied");
    assert_eq!(verify_json["final_class"], "quorum-satisfied");
}

#[test]
fn attest_release_verify_rejects_invalid_release_signature() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let release_keypair = release_keypair();
    write_policy(&verification_dir, release_keypair.verifying_key.name(), Vec::new(), 0);
    write_empty_revocations(&verification_dir);
    write_file(
        &verification_dir.join("release-attestation.json.sig"),
        format!("{}\n", invalid_signature_line(release_keypair.verifying_key.name())).as_bytes(),
    );

    crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .assert()
        .failure()
        .stderr(predicate::str::contains("release attestation signature failed verification"));
}

#[test]
fn attest_release_verify_reports_policy_insufficient_for_wrong_release_digest_witness() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&verification_dir);
    let release_keypair = release_keypair();
    let witness_keypair = crunch_build::generate_keypair().0;
    let wrong_release_digest = AttestationDigest::from_canonical_bytes(b"wrong-release".to_vec());
    write_witness_material_with_release_digest(
        &verification_dir,
        &release_attestation,
        wrong_release_digest,
        &witness_keypair,
        "witness-a",
    );
    write_policy(&verification_dir, release_keypair.verifying_key.name(), vec!["witness-a".to_string()], 1);
    write_empty_revocations(&verification_dir);

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["discovered_witness_count"], 1);
    assert_eq!(verify_json["considered_witness_count"], 1);
    assert_eq!(verify_json["matching_witness_count"], 0);
    assert_eq!(verify_json["technical_class"], "self-proof-valid");
    assert_eq!(verify_json["policy_status"], "insufficient");
    assert_eq!(verify_json["final_class"], "self-proof-valid");
}

#[test]
fn attest_release_verify_reports_policy_insufficient_for_untrusted_witness_identity() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("attest")
        .arg(&bundle_dir)
        .arg("--verification-dir")
        .arg(&verification_dir)
        .arg("--signing-key")
        .arg(&signing_key_path)
        .assert()
        .success();

    let release_attestation = read_release_attestation(&verification_dir);
    let release_keypair = release_keypair();
    let witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&verification_dir, &release_attestation, &witness_keypair, "witness-a");
    write_policy(&verification_dir, release_keypair.verifying_key.name(), vec!["witness-b".to_string()], 1);
    write_empty_revocations(&verification_dir);

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["discovered_witness_count"], 1);
    assert_eq!(verify_json["considered_witness_count"], 0);
    assert_eq!(verify_json["technical_class"], "self-proof-valid");
    assert_eq!(verify_json["policy_status"], "insufficient");
    assert_eq!(verify_json["final_class"], "self-proof-valid");
    assert_eq!(verify_json["matching_witness_count"], 0);
}

fn write_canonical_manifest(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest) {
    let bytes = serde_json::to_vec(manifest).unwrap();
    std::fs::write(bundle_dir.join("manifest.json"), bytes).unwrap();
}

fn hash_directory(path: &Path) -> Result<(u64, String), String> {
    if !path.is_dir() {
        return Err(format!("expected directory artifact: {}", path.display()));
    }
    let mut entries = Vec::new();
    collect_paths_sorted(path, &mut entries)?;
    let mut hasher = blake3::Hasher::new();
    let mut total_file_bytes: u64 = 0;
    for entry in &entries {
        total_file_bytes = total_file_bytes.saturating_add(hash_tree_entry(path, entry, &mut hasher)?);
    }
    if total_file_bytes == 0 {
        total_file_bytes = 1;
    }
    Ok((total_file_bytes, hasher.finalize().to_hex().to_string()))
}

fn collect_paths_sorted(root: &Path, entries: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut children = Vec::new();
    for child_result in std::fs::read_dir(root).map_err(|err| format!("read_dir {}: {err}", root.display()))? {
        let child = child_result.map_err(|err| format!("read_dir entry {}: {err}", root.display()))?;
        children.push(child.path());
    }
    children.sort();
    for child in children {
        entries.push(child.clone());
        if child.is_dir() {
            collect_paths_sorted(&child, entries)?;
        }
    }
    Ok(())
}

fn hash_tree_entry(root: &Path, entry: &Path, hasher: &mut blake3::Hasher) -> Result<u64, String> {
    let relative = entry
        .strip_prefix(root)
        .map_err(|err| format!("tree hash strip_prefix {} from {}: {err}", entry.display(), root.display()))?;
    let metadata =
        std::fs::symlink_metadata(entry).map_err(|err| format!("symlink_metadata {}: {err}", entry.display()))?;
    let relative_bytes = relative.as_os_str().as_encoded_bytes();
    hasher.update(&(relative_bytes.len() as u64).to_le_bytes());
    hasher.update(relative_bytes);
    hasher.update(&entry_mode_bits(&metadata).to_le_bytes());

    if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(entry).map_err(|err| format!("read_link {}: {err}", entry.display()))?;
        let target_bytes = target.as_os_str().as_encoded_bytes();
        hasher.update(b"symlink\0");
        hasher.update(&(target_bytes.len() as u64).to_le_bytes());
        hasher.update(target_bytes);
        return Ok(0);
    }
    if metadata.is_dir() {
        hasher.update(b"dir\0");
        return Ok(0);
    }
    if metadata.is_file() {
        hasher.update(b"file\0");
        hasher.update(&metadata.len().to_le_bytes());
        let mut file = File::open(entry).map_err(|err| format!("open {}: {err}", entry.display()))?;
        let mut buffer = [0_u8; 8192];
        loop {
            let bytes_read = file.read(&mut buffer).map_err(|err| format!("read {}: {err}", entry.display()))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        return Ok(metadata.len());
    }
    Err(format!("unsupported tree hash entry type: {}", entry.display()))
}

#[cfg(unix)]
fn entry_mode_bits(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn entry_mode_bits(_metadata: &std::fs::Metadata) -> u32 {
    0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BundledArtifactKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BundledArtifact {
    kind: BundledArtifactKind,
    relative_path: String,
    size_bytes: u64,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReleaseWorkflowIdentity {
    command: String,
    version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReleaseProofLinkage {
    release_id: String,
    source_archive_digest_blake3: String,
    proof_bundle_schema: String,
    proof_mode: String,
    staged_source: String,
    stage2_binary_digest_blake3: String,
    prerequisite_inventory_digest_blake3: String,
    proof_manifest_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReleaseEvidenceManifest {
    schema: String,
    release_id: String,
    claim_scope: String,
    workflow: ReleaseWorkflowIdentity,
    source_archive: BundledArtifact,
    binaries: Vec<BundledArtifact>,
    proof_bundle: BundledArtifact,
    prerequisite_inventory: BundledArtifact,
    proof_linkage: ReleaseProofLinkage,
}

#[test]
fn release_manifest_schema_constant_matches_fixture_expectation() {
    assert_eq!(RELEASE_EVIDENCE_SCHEMA, "crunch-release-evidence-v1");
}
