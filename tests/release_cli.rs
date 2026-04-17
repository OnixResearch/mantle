use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::process::Output;

use assert_cmd::Command;
use predicates::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use tempfile::TempDir;

const RELEASE_EVIDENCE_SCHEMA: &str = "crunch-release-evidence-v1";
const BLAKE3_HEX_LEN: usize = 64;

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
