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
use crunch_attestation::AgreementWitnessClassification;
use crunch_attestation::AttestationDigest;
use crunch_attestation::Canonicalize;
use crunch_attestation::DetachedSignature;
use crunch_attestation::IndependentAgreementReport;
use crunch_attestation::IndependentAgreementReportInit;
use crunch_attestation::RebuildEnvironmentSummary;
use crunch_attestation::ReleaseAttestation;
use crunch_attestation::ReleasePolicy;
use crunch_attestation::ReleaseRevocations;
use crunch_attestation::WitnessAttestation;
use crunch_attestation::WitnessClassificationReason;
use crunch_attestation::encode_detached_signature;
use crunch_attestation::independent_agreement_report_canonical_bytes;
use crunch_build::KeyPair;
use crunch_build::load_keypair;
use crunch_release_core::BUILD_EFFECT_POLICY_VERSION;
use crunch_release_core::DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE;
use crunch_release_core::DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA;
use crunch_release_core::DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE;
use crunch_release_core::DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA;
use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicBuildProofReceiptInit;
use crunch_release_core::DeterministicBuildRunReceipt;
use crunch_release_core::DeterministicOutputDigest;
use crunch_release_core::DeterministicProofUnit;
use crunch_release_core::DeterministicSandboxIsolationEvidence;
use crunch_release_core::DeterministicSandboxIsolationEvidenceStatus;
use crunch_release_core::NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS;
use crunch_release_core::NIX_CROSS_BUILDER_WITNESS_RECEIPT_SCHEMA;
use crunch_release_core::PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE;
use crunch_release_core::PURE_LOCAL_BUILD_EFFECTS;
use crunch_release_core::REQUIRED_ISOLATION_CHECKS;
use crunch_release_core::ReleaseReproducibilityReport;
use crunch_release_core::SUPPORTED_SANDBOX_PROFILE_FAMILY;
use crunch_release_core::deterministic_build_proof_receipt_canonical_bytes;
use crunch_release_core::deterministic_sandbox_isolation_evidence_canonical_bytes;
use crunch_release_core::release_reproducibility_report_canonical_bytes;
use predicates::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;
use tempfile::TempDir;

const RELEASE_EVIDENCE_SCHEMA: &str = "mantle-release-evidence-v1";
const BLAKE3_HEX_LEN: usize = 64;
const HEX_CHARS_PER_BYTE: usize = 2;
const FUNCTION_ADDRESS_VALENCE_RECEIPT_HASH_SEED: u8 = 2;
const FUNCTION_ADDRESS_KAMACITE_RECEIPT_HASH_SEED: u8 = 3;
const FUNCTION_ADDRESS_STALE_DIGEST_SEED: u8 = 9;
const TEST_CARGO_SHA256_HEX_LEN: usize = 64;
const RELEASE_SIGNING_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const FAKE_WITNESS_DRIVER_MODE_ENV: &str = "CRUNCH_TEST_WITNESS_DRIVER_MODE";
const FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV: &str = "CRUNCH_TEST_WITNESS_DRIVER_LAUNCH_SIGNAL";
const FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_CONTENT: &str = "launched";
const PROVIDER_FIXED_POINT_SCHEMA: &str = "mantle-cargo-free-fixed-point-proof-v1";
const RUST_SOURCE_PROVIDER_BINDING_SCHEMA: &str = "mantle-cargo-free-rust-source-provider-binding-v1";
const PROVIDER_FIXED_POINT_BINARY_BYTES: &[u8] = b"provider-fixed-point-binary";
const PROVIDER_FIXED_POINT_STAGE_UNIT_COUNT: u32 = 2;
const DEFAULT_PROVIDER_WITNESS_TARGET: &str = "x86_64-unknown-linux-musl";
const TEST_BUNDLED_DETERMINISTIC_PROOF_PATH: &str = "deterministic-release/deterministic-build-proof.json";
const TEST_BUNDLED_DETERMINISTIC_SANDBOX_PATH: &str =
    "deterministic-release/deterministic-sandbox-isolation-evidence.json";
const FUNCTION_ADDRESS_SIDECAR_PATH: &str = "external-evidence/03-function-address-sidecar.json";
const FUNCTION_ADDRESS_VALENCE_RECEIPT_PATH: &str = "external-evidence/04-valence-function-address-receipt.json";
const FUNCTION_ADDRESS_KAMACITE_RECEIPT_PATH: &str = "external-evidence/05-kamacite-function-address-receipt.json";
const FUNCTION_ADDRESS_CLI_RECEIPT_OUT_ENV: &str = "MANTLE_FUNCTION_ADDRESS_CLI_RECEIPT_OUT";
const FUNCTION_ADDRESS_CLI_RECEIPT_FILE: &str = "mantle-binding.json";
const FUNCTION_ADDRESS_NEGATIVE_CASE_COUNT: usize = 6;
const RELEASE_VERIFY_JSON_KIND: &str = "mantle-release-verify-v2";
const RELEASE_VERIFY_DECISION_SCHEMA: &str = "mantle-release-verification-decision-v1";
const RELEASE_VERIFY_SUCCESS_MARKER: &str = "release evidence verified";
const RELEASE_VERIFY_REJECTION_MARKER: &str = "release evidence rejected";
const RELEASE_VERIFY_REJECTED_DISPOSITION: &str = "policy-rejected";
const TEST_WITNESS_SOURCE_ACQUISITION_MODE: &str = "manual-operator-supplied";
const TEST_PROOF_FIXED_UMASK: &str = "0022\n";
const TEST_PROOF_NORMALIZATION_ENV: &str = "1|UTC|C.UTF-8|C.UTF-8|/tmp|/tmp|/tmp|/tmp|nobody|nobody\n";
#[cfg(unix)]
const TEST_SCRIPT_MODE: u32 = 0o755;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn assert_no_release_verify_success_marker(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(RELEASE_VERIFY_SUCCESS_MARKER), "stdout was: {stdout}");
    assert!(!stderr.contains(RELEASE_VERIFY_SUCCESS_MARKER), "stderr was: {stderr}");
}

fn assert_human_release_verify_rejection(output: Output, expected_diagnostic: &str) {
    assert!(!output.status.success());
    assert_no_release_verify_success_marker(&output);
    assert!(output.stdout.is_empty(), "stdout was: {}", String::from_utf8_lossy(&output.stdout));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(RELEASE_VERIFY_REJECTION_MARKER), "stderr was: {stderr}");
    assert!(stderr.contains(expected_diagnostic), "stderr was: {stderr}");
}

fn assert_json_release_verify_rejection(
    output: Output,
    expected_contributor: &str,
    expected_diagnostic: &str,
) -> serde_json::Value {
    assert!(!output.status.success());
    assert_no_release_verify_success_marker(&output);
    assert!(output.stderr.is_empty(), "stderr was: {}", String::from_utf8_lossy(&output.stderr));
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(json["kind"], RELEASE_VERIFY_JSON_KIND);
    assert_eq!(json["decision_schema"], RELEASE_VERIFY_DECISION_SCHEMA);
    assert_eq!(json["valid"], false);
    assert_eq!(json["disposition"], RELEASE_VERIFY_REJECTED_DISPOSITION);
    assert!(
        json["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic.as_str().unwrap().contains(expected_diagnostic)),
        "JSON was: {json}"
    );
    let checks = json["checks"].as_array().unwrap();
    assert!(
        checks
            .iter()
            .any(|check| { check["contributor"] == expected_contributor && check["blocking"] == true }),
        "JSON was: {json}"
    );
    let ordered_check_diagnostics = checks
        .iter()
        .filter(|check| check["blocking"] == true)
        .flat_map(|check| check["diagnostics"].as_array().unwrap().iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(json["diagnostics"].as_array().unwrap(), ordered_check_diagnostics.as_slice());
    json
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

fn git_stdout(repo_root: &Path, args: &[&str]) -> String {
    let output = run_git(repo_root, args);
    assert!(
        output.status.success(),
        "git {:?} failed: stdout={} stderr={}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn cargo_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = <Sha256 as sha2::Digest>::new();
    <Sha256 as sha2::Digest>::update(&mut hasher, bytes);
    let digest = <Sha256 as sha2::Digest>::finalize(hasher);
    let encoded = data_encoding::HEXLOWER.encode(&digest);
    assert_eq!(encoded.len(), TEST_CARGO_SHA256_HEX_LEN);
    encoded
}

fn write_test_vendor_package(repo_root: &Path) {
    let manifest = b"[package]\nname=\"dep\"\nversion=\"0.1.0\"\n";
    let lib = b"pub fn dep() {}\n";
    write_file(&repo_root.join("vendor-deps/dep/Cargo.toml"), manifest);
    write_file(&repo_root.join("vendor-deps/dep/lib.rs"), lib);
    let manifest_digest = cargo_sha256_hex(manifest);
    let lib_digest = cargo_sha256_hex(lib);
    let checksum_manifest =
        format!("{{\"files\":{{\"Cargo.toml\":\"{manifest_digest}\",\"lib.rs\":\"{lib_digest}\"}},\"package\":null}}",);
    write_file(&repo_root.join("vendor-deps/dep/.cargo-checksum.json"), checksum_manifest.as_bytes());
}

fn create_minimal_release_repo(repo_root: &Path) {
    write_file(&repo_root.join(".gitignore"), b"vendor-deps/\n");
    write_file(&repo_root.join(".cargo/vendor-config.toml"), b"directory = \"vendor-deps\"\n");
    write_test_vendor_package(repo_root);
    write_file(&repo_root.join("Cargo.toml"), b"[package]\nname=\"demo\"\nversion=\"0.1.0\"\nedition=\"2024\"\n");
    write_file(
        &repo_root.join("Cargo.lock"),
        b"[[package]]\nname = \"dep\"\nversion = \"0.1.0\"\nsource = \"git+https://example.invalid/dep.git#0123456789abcdef\"\n",
    );
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
        "schema": "mantle-self-hosting-proof-v2",
        "staged_source": "/tmp/proof-store/abcd-mantle-src",
        "prerequisites": {
            "mode": "fixed-point",
            "provider_kind": "source-root",
            "inventory_doc": {
                "path": "/tmp/proof-bundle/stage0-prerequisites/inventory.md",
                "size_bytes": 9,
                "digest_blake3": inventory_digest_blake3
            }
        },
        "binaries": {
            "stage1": {
                "path": "/tmp/proof-store/stage1-mantle/bin/mantle",
                "size_bytes": 20,
                "digest_blake3": sample_digest(1)
            },
            "stage2": {
                "path": "/tmp/proof-store/stage2-mantle/bin/mantle",
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
                "staged_source": "/tmp/proof-store/abcd-mantle-src",
                "output_binary": "/tmp/proof-store/stage1-mantle/bin/mantle",
                "busybox_path": "/tmp/proof-store/stage0-busybox/bin/busybox"
            }
        },
        "stage2": {
            "report": {
                "staged_source": "/tmp/proof-store/abcd-mantle-src",
                "output_binary": "/tmp/proof-store/stage2-mantle/bin/mantle",
                "busybox_path": "/tmp/proof-store/stage2-busybox/bin/busybox"
            }
        }
    });
    write_file(proof_dir.join("manifest.json").as_path(), serde_json::to_vec(&manifest).unwrap().as_slice());
    write_file(proof_dir.join("stage0-prerequisites/inventory.md").as_path(), b"inventory");
    write_file(proof_dir.join("summary.txt").as_path(), b"summary\n");
    write_file(proof_dir.join("stage0/stdout.txt").as_path(), b"stage0 stdout\n");
}

fn write_provider_fixed_point_proof_bundle(proof_dir: &Path, binary_bytes: &[u8]) {
    let binary_digest = blake3::hash(binary_bytes).to_hex().to_string();
    let policy_digest = sample_digest(12);
    write_file(&proof_dir.join("stage1/mantle"), binary_bytes);
    write_file(&proof_dir.join("stage2/mantle"), binary_bytes);
    write_provider_stage_receipt(&proof_dir.join("stage1/receipt.json"));
    write_provider_stage_receipt(&proof_dir.join("stage2/receipt.json"));
    write_file(&proof_dir.join("non-claims.txt"), b"This proof does not claim Mantle bootstrap or release reproducibility.\nThis proof does not claim full Cargo compatibility.\n");
    write_file(
        &proof_dir.join("preflight.json"),
        &serde_json::to_vec(&provider_preflight_json(&policy_digest)).unwrap(),
    );
    write_file(
        &proof_dir.join("meta.json"),
        &serde_json::to_vec(&provider_meta_json(proof_dir, &binary_digest, &policy_digest)).unwrap(),
    );
}

fn write_provider_stage_receipt(path: &Path) {
    write_file(
        path,
        &serde_json::to_vec(&serde_json::json!({
            "topology_execution": {
                "execution_status": "success",
                "unit_executions": [
                    { "unit_id": "unit-a", "execution_status": "success" },
                    { "unit_id": "unit-b", "execution_status": "success" }
                ]
            }
        }))
        .unwrap(),
    );
}

fn provider_stage_json(stage_name: &str, binary_digest: &str, policy_digest: &str) -> serde_json::Value {
    serde_json::json!({
        "name": stage_name,
        "dir": stage_name,
        "execution_dir": "execution",
        "receipt": format!("{stage_name}/receipt.json"),
        "stderr": format!("{stage_name}/stderr.txt"),
        "status": format!("{stage_name}/status.txt"),
        "status_code": 0,
        "execution_status": "success",
        "cargo_marker_absent": true,
        "success": true,
        "unit_count": PROVIDER_FIXED_POINT_STAGE_UNIT_COUNT,
        "failed_unit_count": 0,
        "binary": format!("{stage_name}/mantle"),
        "binary_blake3": binary_digest,
        "smoke_status_code": 0,
        "source_built_toolchain_closure_policy_digest_blake3": policy_digest,
        "blocker": null
    })
}

fn provider_preflight_json(policy_digest: &str) -> serde_json::Value {
    serde_json::json!({
        "schema": PROVIDER_FIXED_POINT_SCHEMA,
        "root": "/repo/mantle",
        "bundle_dir": "/tmp/provider-fixed-point-proof",
        "source_built_toolchain_closure": {
            "status": "enforced-source-built",
            "claim": true,
            "policy_digest_blake3": policy_digest
        }
    })
}

fn provider_meta_json(proof_dir: &Path, binary_digest: &str, policy_digest: &str) -> serde_json::Value {
    serde_json::json!({
        "schema": PROVIDER_FIXED_POINT_SCHEMA,
        "status": "success",
        "root": "/repo/mantle",
        "bundle_dir": proof_dir,
        "fixed_point": true,
        "hermeticity_mode": crunch_release_core::STRICT_HERMETICITY_MODE,
        "stage1": provider_stage_json("stage1", binary_digest, policy_digest),
        "stage2": provider_stage_json("stage2", binary_digest, policy_digest),
        "source_built_toolchain_closure": {
            "schema": "mantle-source-built-toolchain-closure-v1",
            "status": "enforced-source-built",
            "claim": true,
            "non_claim": null,
            "manifest_path": "/tmp/toolchain-closure.json",
            "policy_digest_blake3": policy_digest,
            "member_count": PROVIDER_FIXED_POINT_STAGE_UNIT_COUNT,
            "source_built_member_count": PROVIDER_FIXED_POINT_STAGE_UNIT_COUNT,
            "seed_exception_count": 0
        },
        "rust_source_provider": {
            "schema": RUST_SOURCE_PROVIDER_BINDING_SCHEMA,
            "status": "validated",
            "provider_dir": "/provider",
            "metadata_path": "/provider/share/mantle-rust-provider/provider.json",
            "metadata_digest_blake3": sample_digest(13),
            "policy_digest_blake3": policy_digest,
            "host_triple": "x86_64-unknown-linux-musl",
            "target_triple": "x86_64-unknown-linux-musl",
            "artifact_count": 6,
            "source_count": 2,
            "receipt_count": 1,
            "rustc_path": "/provider/bin/rustc"
        },
        "blocker": null,
        "non_claims": [
            "not-crunch-bootstrap",
            "not-release-reproducibility",
            "not-full-cargo-compatibility"
        ]
    })
}

fn sample_digest(seed: u8) -> String {
    let nibble = format!("{:x}", seed % 16);
    nibble.repeat(BLAKE3_HEX_LEN)
}

fn sample_byte_digest(seed: u8) -> String {
    format!("{seed:02x}").repeat(BLAKE3_HEX_LEN / HEX_CHARS_PER_BYTE)
}

fn make_valid_bundle() -> (TempDir, PathBuf, ReleaseEvidenceManifest) {
    make_release_bundle(false)
}

fn make_valid_bundle_with_provider() -> (TempDir, PathBuf, ReleaseEvidenceManifest) {
    make_release_bundle(true)
}

fn make_release_bundle(include_provider_fixed_point: bool) -> (TempDir, PathBuf, ReleaseEvidenceManifest) {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());

    let binary_path = temp.path().join("mantle-bin");
    let binary_bytes: &[u8] = if include_provider_fixed_point {
        PROVIDER_FIXED_POINT_BINARY_BYTES
    } else {
        b"crunch-binary"
    };
    write_file(&binary_path, binary_bytes);
    let stage2_digest = blake3::hash(binary_bytes).to_hex().to_string();
    let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &stage2_digest, &inventory_digest);
    let provider_proof_dir = temp.path().join("provider-fixed-point-input");
    if include_provider_fixed_point {
        write_provider_fixed_point_proof_bundle(&provider_proof_dir, binary_bytes);
    }

    let bundle_dir = temp.path().join("bundle");
    let mut command = crunch();
    command
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir);
    if include_provider_fixed_point {
        command.arg("--provider-fixed-point-proof").arg(&provider_proof_dir);
    }
    command.assert().success().stdout(predicate::str::contains("release id: mantle-0.1.0-rc1"));

    let manifest: ReleaseEvidenceManifest =
        serde_json::from_slice(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).unwrap();
    (temp, bundle_dir, manifest)
}

#[derive(Debug, Clone)]
struct FunctionAddressFixturePaths {
    sidecar: String,
    valence_receipt: String,
    kamacite_receipt: Option<String>,
    release_binary: String,
}

#[derive(Debug, Deserialize)]
struct FunctionAddressCliNegativeCase {
    name: String,
    mutation: String,
    expected_error: String,
    receipt_written: bool,
}

fn install_function_address_fixture(
    bundle_dir: &Path,
    manifest: &mut ReleaseEvidenceManifest,
    with_kamacite: bool,
) -> FunctionAddressFixturePaths {
    let sidecar_bytes = std::fs::read(function_address_cli_fixture_path("sidecar.valid.json")).unwrap();
    let valence_fixture = if with_kamacite {
        function_address_fixture_path("valence-receipt.valid.json")
    } else {
        function_address_cli_fixture_path("valence-receipt.optional.valid.json")
    };
    let valence_bytes = std::fs::read(valence_fixture).unwrap();
    let sidecar_path = bundle_dir.join(FUNCTION_ADDRESS_SIDECAR_PATH);
    let valence_path = bundle_dir.join(FUNCTION_ADDRESS_VALENCE_RECEIPT_PATH);
    write_file(&sidecar_path, &sidecar_bytes);
    write_file(&valence_path, &valence_bytes);
    manifest
        .external_evidence
        .push(function_address_external_evidence(FunctionAddressExternalEvidenceFixture {
            relative_path: FUNCTION_ADDRESS_SIDECAR_PATH,
            bytes: &sidecar_bytes,
            role: crunch_release_core::FUNCTION_ADDRESS_EVIDENCE_ROLE,
            schema: crunch_release_core::FUNCTION_ADDRESS_EVIDENCE_SCHEMA,
        }));
    manifest
        .external_evidence
        .push(function_address_external_evidence(FunctionAddressExternalEvidenceFixture {
            relative_path: FUNCTION_ADDRESS_VALENCE_RECEIPT_PATH,
            bytes: &valence_bytes,
            role: crunch_release_core::VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE,
            schema: crunch_release_core::VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA,
        }));
    let kamacite_receipt = if with_kamacite {
        let bytes = std::fs::read(function_address_fixture_path("kamacite-receipt.valid.json")).unwrap();
        write_file(&bundle_dir.join(FUNCTION_ADDRESS_KAMACITE_RECEIPT_PATH), &bytes);
        manifest
            .external_evidence
            .push(function_address_external_evidence(FunctionAddressExternalEvidenceFixture {
                relative_path: FUNCTION_ADDRESS_KAMACITE_RECEIPT_PATH,
                bytes: &bytes,
                role: crunch_release_core::KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE,
                schema: crunch_release_core::KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA,
            }));
        Some(FUNCTION_ADDRESS_KAMACITE_RECEIPT_PATH.to_string())
    } else {
        None
    };
    write_canonical_manifest(bundle_dir, manifest);
    FunctionAddressFixturePaths {
        sidecar: FUNCTION_ADDRESS_SIDECAR_PATH.to_string(),
        valence_receipt: FUNCTION_ADDRESS_VALENCE_RECEIPT_PATH.to_string(),
        kamacite_receipt,
        release_binary: manifest.binaries.first().expect("release binary").relative_path.clone(),
    }
}

struct FunctionAddressExternalEvidenceFixture<'a> {
    relative_path: &'a str,
    bytes: &'a [u8],
    role: &'a str,
    schema: &'a str,
}

fn function_address_external_evidence(
    fixture: FunctionAddressExternalEvidenceFixture<'_>,
) -> crunch_release_core::ExternalEvidence {
    crunch_release_core::ExternalEvidence {
        role: fixture.role.to_string(),
        schema: fixture.schema.to_string(),
        relative_path: fixture.relative_path.to_string(),
        digest_blake3: blake3::hash(fixture.bytes).to_hex().to_string(),
        claim_scope: crunch_release_core::FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
        non_claims: vec![crunch_release_core::FUNCTION_ADDRESS_OPAQUE_BOUNDARY.to_string()],
    }
}

fn function_address_fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/function-address-release-binding")
        .join(file_name)
}

fn function_address_cli_fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/function-address-release-binding-cli")
        .join(file_name)
}

fn function_address_bind_command(
    bundle_dir: &Path,
    fixture: &FunctionAddressFixturePaths,
    mode: &str,
    receipt_out: &Path,
) -> Command {
    let mut command = crunch();
    command
        .arg("--json")
        .arg("release")
        .arg("function-address-bind")
        .arg(bundle_dir)
        .arg("--mode")
        .arg(mode)
        .arg("--sidecar")
        .arg(&fixture.sidecar)
        .arg("--valence-receipt")
        .arg(&fixture.valence_receipt)
        .arg("--release-binary")
        .arg(&fixture.release_binary)
        .arg("--receipt-out")
        .arg(receipt_out);
    if let Some(kamacite) = fixture.kamacite_receipt.as_ref() {
        command.arg("--kamacite-receipt").arg(kamacite);
    }
    command
}

fn function_address_cli_receipt_out(temp: &TempDir) -> PathBuf {
    std::env::var_os(FUNCTION_ADDRESS_CLI_RECEIPT_OUT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| temp.path().join(FUNCTION_ADDRESS_CLI_RECEIPT_FILE))
}

fn apply_function_address_negative_mutation(mutation: &str, manifest: &mut ReleaseEvidenceManifest) {
    match mutation {
        "remove-valence-row" => {
            manifest.external_evidence.retain(|row| row.relative_path != FUNCTION_ADDRESS_VALENCE_RECEIPT_PATH);
        }
        "stale-valence-digest" => {
            manifest.external_evidence[1].digest_blake3 = sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED);
        }
        "wrong-sidecar-role" => {
            manifest.external_evidence[0].role = "wrong-function-address-role".to_string();
        }
        "wrong-valence-schema" => {
            manifest.external_evidence[1].schema = "wrong.valence.schema.v1".to_string();
        }
        "wrong-sidecar-scope" => {
            manifest.external_evidence[0].claim_scope = "function-address-semantic-correctness".to_string();
        }
        "overclaim" => {
            manifest.external_evidence[0].non_claims = vec!["Mantle proves function correctness".to_string()];
        }
        other => panic!("unsupported function-address CLI negative mutation: {other}"),
    }
}

fn function_address_negative_cases() -> Vec<FunctionAddressCliNegativeCase> {
    serde_json::from_str(include_str!("fixtures/function-address-release-binding-cli/negative-cases.json"))
        .expect("parse function-address CLI negative cases")
}

// r[verify mantle.release_provenance.function_address_binding_cli.command]
// r[verify mantle.release_provenance.function_address_binding_cli.receipt]
// r[verify mantle.release_provenance.function_address_binding_cli.positive]
#[test]
fn function_address_binding_cli_emits_cairn_ready_required_receipt() {
    // r[verify mantle.release_provenance.function_address_binding_cli.validation]
    let (temp, bundle_dir, mut manifest) = make_valid_bundle();
    let fixture = install_function_address_fixture(&bundle_dir, &mut manifest, true);
    let receipt_out = function_address_cli_receipt_out(&temp);
    let _ = std::fs::remove_file(&receipt_out);

    let mut command = function_address_bind_command(&bundle_dir, &fixture, "required", &receipt_out);
    let assert = command.assert().success();
    let receipt: crunch_release_core::FunctionAddressBindingReceipt =
        serde_json::from_slice(&assert.get_output().stdout).expect("parse CLI binding receipt");
    let written = std::fs::read(&receipt_out).expect("read CLI binding receipt");
    let canonical = crunch_release_core::function_address_binding_receipt_canonical_bytes(receipt.clone())
        .expect("canonical CLI binding receipt");

    assert_eq!(written, canonical);
    assert!(receipt.valid);
    assert_eq!(receipt.verdict, crunch_release_core::FUNCTION_ADDRESS_BINDING_VERDICT_PASS);
    assert_eq!(receipt.valence_receipt_digest, sample_byte_digest(FUNCTION_ADDRESS_VALENCE_RECEIPT_HASH_SEED));
    assert_eq!(
        receipt.kamacite_receipt_digest,
        Some(sample_byte_digest(FUNCTION_ADDRESS_KAMACITE_RECEIPT_HASH_SEED))
    );
    assert_ne!(
        receipt.valence_receipt_digest,
        receipt.verification_summary.valence_receipt_digest_blake3.expect("Valence artifact digest")
    );
    assert_ne!(receipt.kamacite_receipt_digest, receipt.verification_summary.kamacite_receipt_digest_blake3);

    function_address_bind_command(&bundle_dir, &fixture, "required", &receipt_out)
        .assert()
        .failure()
        .stderr(predicate::str::contains("without overwrite"));
}

// r[verify mantle.release_provenance.function_address_binding_cli.positive]
#[test]
fn function_address_binding_cli_accepts_optional_valence_without_kamacite() {
    let (temp, bundle_dir, mut manifest) = make_valid_bundle();
    let fixture = install_function_address_fixture(&bundle_dir, &mut manifest, false);
    let receipt_out = temp.path().join("optional-function-address-binding.json");

    let mut command = function_address_bind_command(&bundle_dir, &fixture, "optional", &receipt_out);
    let assert = command.assert().success();
    let receipt: crunch_release_core::FunctionAddressBindingReceipt =
        serde_json::from_slice(&assert.get_output().stdout).expect("parse optional binding receipt");

    assert!(receipt.valid);
    assert!(!receipt.verification_summary.required);
    assert!(receipt.kamacite_receipt_digest.is_none());
    assert!(receipt.verification_summary.kamacite_receipt_hash_blake3.is_none());
    assert!(receipt_out.is_file());
}

// r[verify mantle.release_provenance.function_address_binding_cli.negative]
#[test]
fn function_address_binding_cli_negative_fixture_matrix_fails_closed() {
    let cases = function_address_negative_cases();
    assert_eq!(cases.len(), FUNCTION_ADDRESS_NEGATIVE_CASE_COUNT);
    assert!(cases.iter().any(|case| case.receipt_written));
    for case in cases {
        let (temp, bundle_dir, mut manifest) = make_valid_bundle();
        let fixture = install_function_address_fixture(&bundle_dir, &mut manifest, true);
        apply_function_address_negative_mutation(&case.mutation, &mut manifest);
        write_canonical_manifest(&bundle_dir, &manifest);
        let receipt_out = temp.path().join(format!("{}.json", case.name));
        let output = function_address_bind_command(&bundle_dir, &fixture, "required", &receipt_out)
            .output()
            .expect("run function-address negative fixture");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(!output.status.success(), "negative fixture unexpectedly passed: {}", case.name);
        assert!(stderr.contains(&case.expected_error), "negative fixture {} stderr was: {stderr}", case.name);
        assert_eq!(receipt_out.exists(), case.receipt_written, "negative fixture: {}", case.name);
    }
}

// r[verify mantle.release_provenance.function_address_binding_cli.shell]
// r[verify mantle.release_provenance.function_address_binding_cli.negative]
#[test]
fn function_address_binding_cli_rejects_stale_valence_bytes_before_output() {
    let (temp, bundle_dir, mut manifest) = make_valid_bundle();
    let fixture = install_function_address_fixture(&bundle_dir, &mut manifest, true);
    let receipt_out = temp.path().join("stale-valence-binding.json");
    write_file(&bundle_dir.join(FUNCTION_ADDRESS_VALENCE_RECEIPT_PATH), br#"{"tampered":true}"#);

    function_address_bind_command(&bundle_dir, &fixture, "required", &receipt_out)
        .assert()
        .failure()
        .stderr(predicate::str::contains("external_evidence[1] does not match manifest"));
    assert!(!receipt_out.exists());
}

// r[verify mantle.release_provenance.function_address_binding_cli.negative]
#[test]
fn function_address_binding_cli_rejects_stale_kamacite_logical_link() {
    let (temp, bundle_dir, mut manifest) = make_valid_bundle();
    let fixture = install_function_address_fixture(&bundle_dir, &mut manifest, true);
    let receipt_out = temp.path().join("stale-kamacite-link-binding.json");
    let valence_path = bundle_dir.join(FUNCTION_ADDRESS_VALENCE_RECEIPT_PATH);
    let mut valence: serde_json::Value = serde_json::from_slice(&std::fs::read(&valence_path).unwrap()).unwrap();
    valence["kamacite_receipt_hash"] = serde_json::Value::String(sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED));
    let bytes = serde_json::to_vec(&valence).unwrap();
    write_file(&valence_path, &bytes);
    manifest.external_evidence[1].digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    write_canonical_manifest(&bundle_dir, &manifest);

    function_address_bind_command(&bundle_dir, &fixture, "required", &receipt_out)
        .assert()
        .failure()
        .stderr(predicate::str::contains("logical receipt identities do not match"));
    assert!(!receipt_out.exists());
}

// r[verify mantle.release_provenance.function_address_binding_cli.negative]
#[test]
fn function_address_binding_cli_preserves_overclaim_rejection_receipt() {
    let (temp, bundle_dir, mut manifest) = make_valid_bundle();
    let fixture = install_function_address_fixture(&bundle_dir, &mut manifest, true);
    let receipt_out = temp.path().join("overclaim-binding.json");
    manifest.external_evidence[0].non_claims = vec!["Mantle proves function correctness".to_string()];
    write_canonical_manifest(&bundle_dir, &manifest);

    function_address_bind_command(&bundle_dir, &fixture, "required", &receipt_out)
        .assert()
        .failure()
        .stderr(predicate::str::contains("function-address binding rejected"));
    let receipt: crunch_release_core::FunctionAddressBindingReceipt =
        serde_json::from_slice(&std::fs::read(&receipt_out).expect("invalid receipt is preserved"))
            .expect("parse invalid binding receipt");
    assert!(!receipt.valid);
    assert_eq!(receipt.verdict, crunch_release_core::FUNCTION_ADDRESS_BINDING_VERDICT_FAIL);
    assert!(receipt.verification_summary.diagnostics.iter().any(|diagnostic| diagnostic.contains("overclaim")));
}

#[test]
fn release_create_and_verify_external_evidence_sidecar() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());
    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");
    let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
    let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &stage2_digest, &inventory_digest);
    let sidecar_path = temp.path().join("stack-provenance.json");
    write_file(&sidecar_path, br#"{"schema":"valence.stack-provenance-adapter.v1"}"#);
    let bundle_dir = temp.path().join("bundle-external-evidence");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .arg("--external-evidence")
        .arg(&sidecar_path)
        .arg("--external-evidence-role")
        .arg("stack-provenance-trace")
        .arg("--external-evidence-schema")
        .arg("valence.stack-provenance-adapter.v1")
        .arg("--external-evidence-claim-scope")
        .arg("identity-linkage-sidecar")
        .arg("--external-evidence-non-claim")
        .arg("not semantic validation by Mantle")
        .assert()
        .success()
        .stdout(predicate::str::contains("external evidence: 1"));

    let manifest: ReleaseEvidenceManifest =
        serde_json::from_slice(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).unwrap();
    let evidence = manifest.external_evidence.first().expect("external evidence should be recorded");
    assert_eq!(evidence.role, "stack-provenance-trace");
    assert_eq!(evidence.schema, "valence.stack-provenance-adapter.v1");
    assert_eq!(evidence.claim_scope, "identity-linkage-sidecar");
    assert_eq!(evidence.non_claims, vec!["not semantic validation by Mantle".to_string()]);
    assert!(bundle_dir.join(&evidence.relative_path).is_file());

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-external-evidence-role")
        .arg("stack-provenance-trace")
        .assert()
        .success()
        .stdout(predicate::str::contains("external evidence: 1"));

    let output = crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-external-evidence-role")
        .arg("missing-role")
        .output()
        .unwrap();
    assert_human_release_verify_rejection(output, "external evidence role required but missing");
}

// r[verify mantle.release_provenance.valence_receipt_binding]
// r[verify mantle.release_provenance.valence_required_policy.required_valid]
#[test]
fn release_create_and_verify_stack_provenance_sidecar() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());
    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");
    let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
    let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &stage2_digest, &inventory_digest);
    let sidecar_path = temp.path().join("stack-provenance-sidecar.json");
    let receipt_path = temp.path().join("valence-stack-provenance-graph-report.json");
    write_file(&sidecar_path, br#"{"schema":"valence.stack-provenance-sidecar.v1"}"#);
    write_file(&receipt_path, br#"{"schema":"valence.stack-provenance-graph-report.v1","valid":true}"#);
    let bundle_dir = temp.path().join("bundle-stack-provenance");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .arg("--stack-provenance-sidecar")
        .arg(&sidecar_path)
        .arg("--stack-provenance-valence-receipt")
        .arg(&receipt_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("external evidence: 2"));

    let manifest_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest_json["stack_provenance"]["sidecar_role"], "stack-provenance-trace");
    assert_eq!(manifest_json["stack_provenance"]["valence_receipt_role"], "valence-stack-provenance-graph-report");
    assert_eq!(manifest_json["stack_provenance"]["sidecar_claim_scope"], "identity-linkage-sidecar");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--stack-provenance")
        .arg("required")
        .assert()
        .success()
        .stdout(predicate::str::contains("stack provenance: present"))
        .stdout(predicate::str::contains("stack provenance mode: required"));
}

// r[verify mantle.release_provenance.valence_required_policy.required_missing]
#[test]
fn release_verify_required_stack_provenance_fails_when_missing() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();

    let output = crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--stack-provenance")
        .arg("required")
        .output()
        .unwrap();
    assert_human_release_verify_rejection(output, "required Valence stack provenance sidecar or receipt is missing");
}

#[test]
fn release_create_records_git_source_metadata() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());
    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");
    let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
    let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &stage2_digest, &inventory_digest);
    let bundle_dir = temp.path().join("bundle-git-source");
    let commit = git_stdout(temp.path(), &["rev-parse", "HEAD"]);
    let branch_ref = format!("refs/heads/{}", git_stdout(temp.path(), &["branch", "--show-current"]));
    let remote_url = format!("file://{}", temp.path().display());

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .arg("--git-source-url")
        .arg(&remote_url)
        .arg("--git-source-commit")
        .arg(&commit)
        .arg("--git-source-ref")
        .arg(&branch_ref)
        .assert()
        .success()
        .stdout(predicate::str::contains("source acquisition commit"));

    let manifest: ReleaseEvidenceManifest =
        serde_json::from_slice(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).unwrap();
    let source = manifest.source_acquisition.expect("Git source acquisition should be recorded");
    assert_eq!(source.kind, "git");
    assert_eq!(source.url, remote_url);
    assert_eq!(source.commit.as_deref(), Some(commit.as_str()));
    assert_eq!(source.reference.as_deref(), Some(branch_ref.as_str()));
    assert_eq!(source.digest_blake3, manifest.source_archive.digest_blake3);
}

#[test]
fn release_create_rejects_git_source_url_without_commit() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());
    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &blake3::hash(b"crunch-binary").to_hex().to_string(), &sample_digest(9));

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(temp.path().join("bundle-missing-commit"))
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .arg("--git-source-url")
        .arg(format!("file://{}", temp.path().display()))
        .assert()
        .failure()
        .stderr(predicate::str::contains("git-source-commit"));
}

#[test]
fn release_create_rejects_conflicting_source_acquisition_flags() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());
    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &blake3::hash(b"crunch-binary").to_hex().to_string(), &sample_digest(9));
    let commit = git_stdout(temp.path(), &["rev-parse", "HEAD"]);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(temp.path().join("bundle-conflict"))
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .arg("--source-acquisition-url")
        .arg("https://example.invalid/source.tar")
        .arg("--git-source-url")
        .arg(format!("file://{}", temp.path().display()))
        .arg("--git-source-commit")
        .arg(commit)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
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
    write_policy_with_independence(
        verification_dir,
        release_signer_name,
        trusted_witness_identities,
        min_matching_witnesses,
        "witness_identity",
    );
}

fn write_policy_with_independence(
    verification_dir: &Path,
    release_signer_name: &str,
    trusted_witness_identities: Vec<String>,
    min_matching_witnesses: u32,
    independence_field: &str,
) {
    let policy = ReleasePolicy::new(
        min_matching_witnesses,
        independence_field.to_string(),
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
    )
    .with_source_acquisition_mode(TEST_WITNESS_SOURCE_ACQUISITION_MODE.to_string());
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

fn write_matching_agreement_report(
    verification_dir: &Path,
    release_attestation: &ReleaseAttestation,
    policy: &ReleasePolicy,
    witness_keypair: &KeyPair,
    witness_identity: &str,
) -> PathBuf {
    let report = build_matching_agreement_report(release_attestation, policy, witness_keypair, witness_identity);
    let bytes = independent_agreement_report_canonical_bytes(report).unwrap();
    let report_path = verification_dir.join("agreement-report.json");
    write_file(&report_path, &bytes);
    report_path
}

fn build_matching_agreement_report(
    release_attestation: &ReleaseAttestation,
    policy: &ReleasePolicy,
    witness_keypair: &KeyPair,
    witness_identity: &str,
) -> IndependentAgreementReport {
    let release_digest = release_attestation.canonical_digest().unwrap();
    let witness = WitnessAttestation::new(
        release_digest,
        witness_identity.to_string(),
        release_attestation.binary_digests.clone(),
        RebuildEnvironmentSummary {
            system: "x86_64-linux".to_string(),
            toolchain: "rust-1.91.1".to_string(),
            host_class: "nixos-25.05".to_string(),
        },
    )
    .with_source_acquisition_mode(TEST_WITNESS_SOURCE_ACQUISITION_MODE.to_string());
    let policy_digest = AttestationDigest::from_canonical_bytes(serde_json::to_vec(policy).unwrap());
    IndependentAgreementReport::new(IndependentAgreementReportInit {
        release_attestation_digest_blake3: release_digest,
        policy_digest_blake3: policy_digest,
        independence_selector: policy.independence_field.clone(),
        required_witness_count: policy.min_matching_witnesses,
        witnesses: vec![AgreementWitnessClassification {
            witness_identity: witness_identity.to_string(),
            signer_key_name: witness_keypair.verifying_key.name().to_string(),
            witness_digest_blake3: witness.canonical_digest().unwrap(),
            release_attestation_digest_blake3: witness.release_attestation_digest_blake3.clone(),
            signature_valid: true,
            digest_match: true,
            independence_domain: witness_identity.to_string(),
            source_acquisition_mode: witness.source_acquisition_mode.clone().unwrap(),
            policy_counted: true,
            classification_reason: WitnessClassificationReason::Counted,
            rebuilt_output_digests: release_attestation.binary_digests.clone(),
            environment_summary: witness.rebuild_environment_summary,
        }],
        artifact_digest_sets: release_attestation.binary_digests.clone(),
    })
    .unwrap()
}

fn write_witness_material_with_host_class(
    verification_dir: &Path,
    release_attestation: &ReleaseAttestation,
    witness_keypair: &KeyPair,
    identity: &str,
    host_class: &str,
) {
    let witness = WitnessAttestation::new(
        release_attestation.canonical_digest().unwrap(),
        identity.to_string(),
        release_attestation.binary_digests.clone(),
        RebuildEnvironmentSummary {
            system: "x86_64-linux".to_string(),
            toolchain: "rust-1.91.1".to_string(),
            host_class: host_class.to_string(),
        },
    )
    .with_source_acquisition_mode(TEST_WITNESS_SOURCE_ACQUISITION_MODE.to_string());
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

fn assert_fake_witness_driver_launch_signal_present(path: &Path) {
    assert!(path.exists(), "fake witness driver must record launch signal: {}", path.display());
    let launch_signal = std::fs::read_to_string(path).unwrap();
    assert_eq!(launch_signal.trim(), FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_CONTENT);
}

fn assert_fake_witness_driver_launch_signal_absent(path: &Path) {
    assert!(!path.exists(), "fake witness driver must not record launch signal: {}", path.display());
}

fn write_fake_provider_cli_capture_shim(path: &Path) {
    let script = r#"#!/usr/bin/env bash
set -euo pipefail
: "${CRUNCH_TEST_REAL_CLI:?}"
: "${CRUNCH_TEST_CAPTURE_ARGS:?}"
if [[ "$#" -ge 2 && "$1" == "release" && "$2" == "witness-rebuild" ]]; then
  exec "$CRUNCH_TEST_REAL_CLI" "$@"
fi
: > "$CRUNCH_TEST_CAPTURE_ARGS"
for arg in "$@"; do
  printf '%s\n' "$arg" >> "$CRUNCH_TEST_CAPTURE_ARGS"
done
"#;
    write_file(path, script.as_bytes());
    #[cfg(unix)]
    {
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(TEST_SCRIPT_MODE);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

fn write_fake_witness_rebuild_driver(path: &Path) {
    let script = r#"#!/usr/bin/env bash
set -euo pipefail
readonly DRIVER_SLEEP_SECONDS=0.1
launch_signal="${CRUNCH_TEST_WITNESS_DRIVER_LAUNCH_SIGNAL:?fake driver requires launch signal path}"
# Write launch signal before later env validation so preflight-failure tests can
# distinguish "driver never launched" from "driver started and failed early".
printf 'launched\n' > "$launch_signal"
: "${CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR:?}"
: "${CRUNCH_WITNESS_RELEASE_BUNDLE_DIR:?}"
mode="${CRUNCH_TEST_WITNESS_DRIVER_MODE:-match}"
mkdir -p "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR"
printf 'driver mode: %s\n' "$mode"
sleep "$DRIVER_SLEEP_SECONDS"
output_path="$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/stage2-mantle"
case "$mode" in
  match)
    cp "$CRUNCH_WITNESS_RELEASE_BUNDLE_DIR/binaries/01-mantle-bin" "$output_path"
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
        permissions.set_mode(TEST_SCRIPT_MODE);
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

fn write_rebuild_copy_script(script_path: &Path, binary_relative_path: &str) {
    write_rebuild_script(
        script_path,
        &format!(
            "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\ncp \"$MANTLE_REPRODUCE_BUNDLE_DIR/{binary_relative_path}\" \"$MANTLE_REPRODUCE_OUTPUT_DIR/{binary_relative_path}\"\n"
        ),
    );
}

fn write_rebuild_script(script_path: &Path, body: &str) {
    let script = format!("#!/bin/sh\nset -eu\n{body}");
    write_file(script_path, script.as_bytes());
    #[cfg(unix)]
    {
        let mut permissions = std::fs::metadata(script_path).unwrap().permissions();
        permissions.set_mode(TEST_SCRIPT_MODE);
        std::fs::set_permissions(script_path, permissions).unwrap();
    }
}

#[cfg(unix)]
fn write_fake_bwrap(script_path: &Path) {
    write_rebuild_script(
        script_path,
        r#"
if [ "${1:-}" = "--version" ]; then
  printf 'bwrap 1.0-test\n'
  exit 0
fi
if [ -n "${MANTLE_FAKE_BWRAP_TRANSCRIPT:-}" ]; then
  {
    printf -- '-- fake-bwrap invocation --\n'
    for arg in "$@"; do printf '%s\n' "$arg"; done
  } >> "$MANTLE_FAKE_BWRAP_TRANSCRIPT"
fi
while [ "$#" -gt 0 ]; do
  case "$1" in
    --share-net)
      printf 'fake bwrap denied host network opt-in\n' >&2
      exit 125
      ;;
    --unshare-all|--die-with-parent|--new-session|--clearenv) shift ;;
    --ro-bind|--bind)
      if [ -n "${MANTLE_FAKE_BWRAP_FORBIDDEN_BIND:-}" ] && { [ "$2" = "$MANTLE_FAKE_BWRAP_FORBIDDEN_BIND" ] || [ "$3" = "$MANTLE_FAKE_BWRAP_FORBIDDEN_BIND" ]; }; then
        printf 'fake bwrap denied forbidden bind: %s\n' "$MANTLE_FAKE_BWRAP_FORBIDDEN_BIND" >&2
        exit 125
      fi
      shift 3
      ;;
    --tmpfs|--dev|--proc|--chdir|--dir)
      if [ "$1" = "--chdir" ]; then cd "$2"; fi
      shift 2 ;;
    --setenv) export "$2=$3"; shift 3 ;;
    *)
      if [ -n "${MANTLE_FAKE_BWRAP_FORBIDDEN_HOST_PATH:-}" ] && [ -f "$1" ] && grep -F -- "$MANTLE_FAKE_BWRAP_FORBIDDEN_HOST_PATH" "$1" >/dev/null 2>&1; then
        printf 'fake bwrap denied undeclared host path: %s\n' "$MANTLE_FAKE_BWRAP_FORBIDDEN_HOST_PATH" >&2
        exit 125
      fi
      if [ -n "${MANTLE_FAKE_BWRAP_HOST_PATH:-}" ]; then export PATH="$MANTLE_FAKE_BWRAP_HOST_PATH"; fi
      exec "$@"
      ;;
  esac
done
"#,
    );
}

#[cfg(unix)]
fn write_matched_default_reproducibility_report(
    temp: &TempDir,
    bundle_dir: &Path,
    manifest: &ReleaseEvidenceManifest,
) -> PathBuf {
    let rebuild_script = temp.path().join("verify-rebuild.sh");
    let rebuild_output_dir = temp.path().join("verify-rebuild-output");
    write_rebuild_copy_script(&rebuild_script, &manifest.binaries[0].relative_path);
    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("reproduce")
        .arg(bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .assert()
        .success();
    bundle_dir.join("reproducibility/reproducibility-report.json")
}

fn read_reproducibility_report(report_path: &Path) -> ReleaseReproducibilityReport {
    serde_json::from_slice(&std::fs::read(report_path).unwrap()).unwrap()
}

fn write_canonical_reproducibility_report(report_path: &Path, report: ReleaseReproducibilityReport) {
    let bytes = release_reproducibility_report_canonical_bytes(report).unwrap();
    write_file(report_path, &bytes);
}

fn write_deterministic_verify_artifacts(
    dir: &Path,
    manifest: &ReleaseEvidenceManifest,
) -> (PathBuf, PathBuf, String, String) {
    let proof_path = dir.join("deterministic-build-proof.json");
    let evidence_path = dir.join("deterministic-sandbox-isolation-evidence.json");
    let profile = format!("{SUPPORTED_SANDBOX_PROFILE_FAMILY}:test-profile");
    let output_digest = manifest.binaries[0].digest_blake3.clone();
    let output_name = manifest.binaries[0].relative_path.clone();
    let run = |run_id: &str, perturbation_case: &str, store: &str| DeterministicBuildRunReceipt {
        run_id: run_id.to_string(),
        perturbation_case: perturbation_case.to_string(),
        output_store_paths: vec![store.to_string()],
        output_root_identity: format!("{store}/outputs"),
        sandbox_profile_identity: profile.clone(),
        output_digests: vec![DeterministicOutputDigest {
            name: output_name.clone(),
            digest_blake3: output_digest.clone(),
        }],
        substituted_dependency_identities: Vec::new(),
        hermeticity_audit_events: Vec::new(),
        observed_effects: Some(PURE_LOCAL_BUILD_EFFECTS.to_vec()),
    };
    let proof = DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
        proof_unit: DeterministicProofUnit {
            target_artifact_identity: format!("release:{}", manifest.release_id),
            output_identities: vec![output_name.clone()],
        },
        derivation_identity: format!("release:{}", manifest.release_id),
        hermeticity_mode: "strict".to_string(),
        workflow_version: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
        selected_provider_kind: manifest.proof_linkage.selected_provider_kind.clone(),
        source_blake3: manifest.source_archive.digest_blake3.clone(),
        vendor_blake3: manifest.proof_bundle.digest_blake3.clone(),
        toolchain_provider_identity: format!(
            "provider-kind={};command=test-provider",
            manifest.proof_linkage.selected_provider_kind
        ),
        toolchain_stage_roots: vec![
            format!("staged-source={}", manifest.proof_linkage.staged_source),
            format!("stage2-binary={}", manifest.proof_linkage.stage2_binary_digest_blake3),
            format!("prerequisite-inventory={}", manifest.proof_linkage.prerequisite_inventory_digest_blake3),
        ],
        logical_store_prefix: "/mantle/store".to_string(),
        physical_store_isolation: "fresh-store-per-run".to_string(),
        effect_policy_version: BUILD_EFFECT_POLICY_VERSION.to_string(),
        declared_effects: PURE_LOCAL_BUILD_EFFECTS.to_vec(),
        observed_effects: None,
        normalized_execution_envelope: vec![
            "bundle-read-only".to_string(),
            "network=none".to_string(),
            "output-dir-empty".to_string(),
            "sandbox=bwrap".to_string(),
            format!("sandbox-profile={profile}"),
            "normalization:time=fixed".to_string(),
            "normalization:timezone=utc".to_string(),
            "normalization:locale=c".to_string(),
            "normalization:temp-roots=isolated".to_string(),
            "normalization:host-user-metadata=fixed".to_string(),
            "normalization:umask=0022".to_string(),
            "normalization:modeled-randomness=seeded".to_string(),
            "normalization:order-sensitive-output-processing=sorted".to_string(),
        ],
        ambient_host_perturbations: vec![
            "HOME".to_string(),
            "PATH".to_string(),
            "USER".to_string(),
            "LOGNAME".to_string(),
            "TZ".to_string(),
            "LANG".to_string(),
            "LC_ALL".to_string(),
            "TMPDIR".to_string(),
            "cwd".to_string(),
            "umask".to_string(),
            "env-noise".to_string(),
        ],
        sandbox_profile_identities: vec![profile.clone()],
        runs: vec![
            run("run-000", "baseline-clean-env", "/tmp/store-a"),
            run("run-001", "host-env-noise", "/tmp/store-b"),
        ],
    });
    let proof_bytes = deterministic_build_proof_receipt_canonical_bytes(proof).unwrap();
    let proof_digest = blake3::hash(&proof_bytes).to_hex().to_string();
    write_file(&proof_path, &proof_bytes);

    let evidence = DeterministicSandboxIsolationEvidence {
        schema: DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA.to_string(),
        profile_family: SUPPORTED_SANDBOX_PROFILE_FAMILY.to_string(),
        evidence_version: "mantle-release-reproducibility-v1".to_string(),
        status: DeterministicSandboxIsolationEvidenceStatus::Passed,
        checks: REQUIRED_ISOLATION_CHECKS.iter().map(|check| (*check).to_string()).collect(),
        evidence_digest_blake3: sample_digest(8),
    };
    let evidence_bytes = deterministic_sandbox_isolation_evidence_canonical_bytes(evidence).unwrap();
    let evidence_digest = blake3::hash(&evidence_bytes).to_hex().to_string();
    write_file(&evidence_path, &evidence_bytes);
    (proof_path, evidence_path, proof_digest, evidence_digest)
}

fn install_bundled_deterministic_artifacts(
    bundle_dir: &Path,
    manifest: &ReleaseEvidenceManifest,
    proof_path: &Path,
    evidence_path: &Path,
) -> ReleaseEvidenceManifest {
    let bundled_proof_path = bundle_dir.join(TEST_BUNDLED_DETERMINISTIC_PROOF_PATH);
    let bundled_evidence_path = bundle_dir.join(TEST_BUNDLED_DETERMINISTIC_SANDBOX_PATH);
    write_file(&bundled_proof_path, &std::fs::read(proof_path).unwrap());
    write_file(&bundled_evidence_path, &std::fs::read(evidence_path).unwrap());

    let mut updated = manifest.clone();
    updated.deterministic_build_proof = Some(role_bounded_fixture_artifact(
        &bundled_proof_path,
        TEST_BUNDLED_DETERMINISTIC_PROOF_PATH,
        DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
    ));
    updated.deterministic_sandbox_isolation_evidence = Some(role_bounded_fixture_artifact(
        &bundled_evidence_path,
        TEST_BUNDLED_DETERMINISTIC_SANDBOX_PATH,
        DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
    ));
    write_canonical_manifest(bundle_dir, &updated);
    updated
}

fn role_bounded_fixture_artifact(
    path: &Path,
    relative_path: &str,
    role: &str,
) -> crunch_release_core::RoleBoundedReleaseArtifact {
    let bytes = std::fs::read(path).unwrap();
    crunch_release_core::RoleBoundedReleaseArtifact {
        kind: crunch_release_core::BundledArtifactKind::File,
        relative_path: relative_path.to_string(),
        size_bytes: bytes.len().try_into().unwrap(),
        digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        evidence_role: role.to_string(),
    }
}

fn run_nix_witness_args(
    bundle_dir: &Path,
    nix_output_dir: &Path,
    deterministic_proof: &Path,
    receipt_path: &Path,
) -> Command {
    let mut cmd = crunch();
    cmd.arg("--json")
        .arg("release")
        .arg("nix-witness")
        .arg(bundle_dir)
        .arg("--nix-output-dir")
        .arg(nix_output_dir)
        .arg("--deterministic-proof")
        .arg(deterministic_proof)
        .arg("--receipt-path")
        .arg(receipt_path)
        .arg("--rust-toolchain-identity")
        .arg("rust-nightly-1.91.1")
        .arg("--target-triple")
        .arg("x86_64-unknown-linux-musl")
        .arg("--build-flag")
        .arg("-Ctarget-feature=+crt-static")
        .arg("--linker-identity")
        .arg("clang+mold")
        .arg("--strip-debug-policy")
        .arg("strip")
        .arg("--source-date-epoch-policy")
        .arg("release-manifest")
        .arg("--nix-derivation-identity")
        .arg("/nix/store/demo-mantle.drv")
        .arg("--nix-output-identity")
        .arg("/nix/store/demo-mantle");
    cmd
}

fn populate_nix_output_from_bundle(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest, nix_output_dir: &Path) {
    for artifact in &manifest.binaries {
        let bytes = std::fs::read(bundle_dir.join(&artifact.relative_path)).unwrap();
        write_file(&nix_output_dir.join(&artifact.relative_path), &bytes);
    }
}

#[test]
fn release_nix_witness_writes_match_receipt_from_located_nix_artifacts() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let (proof_path, _, proof_digest, _) = write_deterministic_verify_artifacts(&temp.path().join("proof"), &manifest);
    let nix_output_dir = temp.path().join("nix-output");
    let receipt_path = temp.path().join("nix-witness.json");
    populate_nix_output_from_bundle(&bundle_dir, &manifest, &nix_output_dir);

    let assert = run_nix_witness_args(&bundle_dir, &nix_output_dir, &proof_path, &receipt_path)
        .current_dir(temp.path())
        .assert()
        .success();
    let stdout = serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap();
    let receipt = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&receipt_path).unwrap()).unwrap();

    assert_eq!(stdout["kind"], "mantle-nix-cross-builder-witness-run-v1");
    assert_eq!(stdout["comparison_verdict"], "nix-witness-match");
    assert_eq!(stdout["proof_class"], NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS);
    assert_eq!(stdout["receipt_path"], receipt_path.display().to_string());
    assert!(stdout["bounded_claim"].as_str().unwrap().contains("does not replace self-rebuild-match"));
    assert_eq!(receipt["schema"], NIX_CROSS_BUILDER_WITNESS_RECEIPT_SCHEMA);
    assert_eq!(receipt["proof_class"], NIX_CROSS_BUILDER_WITNESS_PROOF_CLASS);
    assert_eq!(receipt["comparison_verdict"], "nix-witness-match");
    assert_eq!(receipt["mantle_deterministic_proof_receipt_digest_blake3"], proof_digest);
    assert_eq!(receipt["source_tree_digest_blake3"], manifest.source_archive.digest_blake3);
    assert_eq!(receipt["vendor_input_digest_blake3"], manifest.proof_bundle.digest_blake3);
    assert_eq!(receipt["build_policy"]["build_flags"][0], "-Ctarget-feature=+crt-static");
    assert!(receipt["receipt_blake3"].as_str().unwrap().len() == BLAKE3_HEX_LEN);
}

#[test]
fn release_nix_witness_writes_mismatch_receipt_without_promoting_proof_class() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let (proof_path, _, _, _) = write_deterministic_verify_artifacts(&temp.path().join("proof"), &manifest);
    let nix_output_dir = temp.path().join("nix-output");
    let receipt_path = temp.path().join("nix-witness-mismatch.json");
    populate_nix_output_from_bundle(&bundle_dir, &manifest, &nix_output_dir);
    write_file(&nix_output_dir.join(&manifest.binaries[0].relative_path), b"nix-drift");

    let assert = run_nix_witness_args(&bundle_dir, &nix_output_dir, &proof_path, &receipt_path)
        .current_dir(temp.path())
        .assert()
        .success();
    let stdout = serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap();
    let receipt = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&receipt_path).unwrap()).unwrap();

    assert_eq!(stdout["comparison_verdict"], "cross-builder-mismatch");
    assert_eq!(stdout["proof_class"], serde_json::Value::Null);
    assert_eq!(receipt["comparison_verdict"], "cross-builder-mismatch");
    assert!(receipt.get("proof_class").is_none());
}

#[test]
fn release_nix_witness_require_match_fails_closed_on_digest_mismatch() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let (proof_path, _, _, _) = write_deterministic_verify_artifacts(&temp.path().join("proof"), &manifest);
    let nix_output_dir = temp.path().join("nix-output");
    let receipt_path = temp.path().join("nix-witness-mismatch-required.json");
    populate_nix_output_from_bundle(&bundle_dir, &manifest, &nix_output_dir);
    write_file(&nix_output_dir.join(&manifest.binaries[0].relative_path), b"nix-drift");

    run_nix_witness_args(&bundle_dir, &nix_output_dir, &proof_path, &receipt_path)
        .arg("--require-match")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nix cross-builder witness required but verdict is cross-builder-mismatch"));
    assert!(receipt_path.is_file(), "mismatch receipt should still be written for diagnosis");
}

#[test]
fn release_nix_witness_rejects_deterministic_proof_source_drift() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let (proof_path, _, _, _) = write_deterministic_verify_artifacts(&temp.path().join("proof"), &manifest);
    let nix_output_dir = temp.path().join("nix-output");
    let receipt_path = temp.path().join("nix-witness-source-drift.json");
    populate_nix_output_from_bundle(&bundle_dir, &manifest, &nix_output_dir);
    let mut proof: DeterministicBuildProofReceipt =
        serde_json::from_slice(&std::fs::read(&proof_path).unwrap()).unwrap();
    proof.source_blake3 = sample_digest(41);
    let proof_bytes = deterministic_build_proof_receipt_canonical_bytes(proof).unwrap();
    write_file(&proof_path, &proof_bytes);

    run_nix_witness_args(&bundle_dir, &nix_output_dir, &proof_path, &receipt_path)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "deterministic proof source digest does not match release bundle source digest",
        ));
    assert!(!receipt_path.exists());
}

#[test]
fn release_nix_witness_rejects_missing_mantle_deterministic_proof() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let nix_output_dir = temp.path().join("nix-output");
    let receipt_path = temp.path().join("nix-witness-missing-proof.json");
    populate_nix_output_from_bundle(&bundle_dir, &manifest, &nix_output_dir);

    run_nix_witness_args(&bundle_dir, &nix_output_dir, &temp.path().join("missing-proof.json"), &receipt_path)
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("deterministic proof artifact does not exist"));
    assert!(!receipt_path.exists());
}

#[test]
fn release_create_fails_when_proof_bundle_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());

    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
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

// r[verify mantle.release_provenance.bundle_tree_copy.validation.production]
// r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
#[test]
#[cfg(unix)]
fn release_create_rejects_directory_symlink_escape_without_external_writes() {
    use std::os::unix::fs::symlink;

    const EXTERNAL_SENTINEL_BYTES: &[u8] = b"external-sentinel";
    const ATTACKER_REPLACEMENT_BYTES: &[u8] = b"attacker-replacement";
    const ATTACKER_CREATED_BYTES: &[u8] = b"attacker-created";
    const ESCAPING_TARGET: &str = "../../../outside";

    let source_temp = tempfile::tempdir().unwrap();
    let destination_temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(source_temp.path());
    let binary_path = source_temp.path().join("mantle-bin");
    let proof_dir = source_temp.path().join("a/b/proof-input");
    let source_outside = source_temp.path().join("outside");
    let bundle_dir = destination_temp.path().join("release-bundle");
    let external_dir = destination_temp.path().join("outside");
    let external_sentinel = external_dir.join("sentinel.txt");
    let external_created = external_dir.join("created.txt");

    write_file(&binary_path, b"crunch-binary");
    let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
    let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
    write_full_proof_bundle(&proof_dir, &stage2_digest, &inventory_digest);
    write_file(&source_outside.join("sentinel.txt"), ATTACKER_REPLACEMENT_BYTES);
    write_file(&source_outside.join("created.txt"), ATTACKER_CREATED_BYTES);
    symlink(ESCAPING_TARGET, proof_dir.join("escape")).unwrap();
    write_file(&external_sentinel, EXTERNAL_SENTINEL_BYTES);

    crunch()
        .current_dir(source_temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-tree-copy-security")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("symlink target escapes"));

    assert_eq!(std::fs::read(&external_sentinel).unwrap(), EXTERNAL_SENTINEL_BYTES);
    assert!(!external_created.exists(), "release create must not create an external path");
    assert!(!bundle_dir.exists(), "invalid tree plan must block bundle mutation");
}

#[test]
fn release_create_rejects_prerequisite_only_proof_bundle() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());

    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");
    let proof_dir = temp.path().join("fake-proof");
    write_fake_proof_bundle(&proof_dir);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
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

    let output = crunch().arg("release").arg("verify").arg(&bundle_dir).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "stderr was: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("release id: {}", manifest.release_id)));
    assert!(stdout.contains(&format!("binaries: {}", manifest.binaries.len())));
    assert!(stdout.contains(&format!("source digest: {}", manifest.source_archive.digest_blake3)));
    assert!(stdout.contains(&format!("stage2 digest: {}", manifest.proof_linkage.stage2_binary_digest_blake3)));
    assert_eq!(stdout.matches(RELEASE_VERIFY_SUCCESS_MARKER).count(), 1);
    assert_eq!(
        stdout.matches("verification check:").count(),
        crunch_release_core::RELEASE_VERIFICATION_CONTRIBUTOR_COUNT
    );
    assert!(stdout.rfind("verification check:").unwrap() < stdout.find(RELEASE_VERIFY_SUCCESS_MARKER).unwrap());
    assert!(stdout.trim_end().ends_with(&format!("{}: {}", RELEASE_VERIFY_SUCCESS_MARKER, bundle_dir.display())));
}

#[test]
fn release_verify_reports_required_deterministic_release_from_artifacts() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let proof_dir = temp.path().join("deterministic-proof-artifacts");
    let (proof_path, evidence_path, proof_digest, evidence_digest) =
        write_deterministic_verify_artifacts(&proof_dir, &manifest);

    let assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--deterministic-proof")
        .arg(&proof_path)
        .arg("--deterministic-sandbox-isolation-evidence")
        .arg(&evidence_path)
        .arg("--require-deterministic-release")
        .assert()
        .success();
    let stdout = serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap();
    assert_eq!(stdout["kind"], RELEASE_VERIFY_JSON_KIND);
    assert_eq!(stdout["valid"], true);
    assert_eq!(stdout["disposition"], "accepted");
    assert!(stdout["diagnostics"].as_array().unwrap().is_empty());
    assert_eq!(stdout["deterministic_release"]["status"], "eligible");
    assert_eq!(stdout["deterministic_release"]["eligible"], true);
    assert_eq!(stdout["deterministic_release"]["proof_digest_blake3"], proof_digest);
    assert_eq!(stdout["deterministic_release"]["sandbox_isolation_evidence_digest_blake3"], evidence_digest);
}

#[test]
fn release_verify_require_deterministic_release_rejects_missing_isolation_evidence() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let proof_dir = temp.path().join("deterministic-proof-artifacts");
    let (proof_path, _evidence_path, _proof_digest, _evidence_digest) =
        write_deterministic_verify_artifacts(&proof_dir, &manifest);

    let output = crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--deterministic-proof")
        .arg(&proof_path)
        .arg("--require-deterministic-release")
        .output()
        .unwrap();
    assert_human_release_verify_rejection(output, "missing deterministic sandbox isolation evidence");
}

#[test]
fn release_verify_rejects_noncanonical_deterministic_isolation_evidence() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let proof_dir = temp.path().join("deterministic-proof-artifacts");
    let (proof_path, evidence_path, _proof_digest, _evidence_digest) =
        write_deterministic_verify_artifacts(&proof_dir, &manifest);
    let pretty_evidence: serde_json::Value = serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    write_file(&evidence_path, serde_json::to_string_pretty(&pretty_evidence).unwrap().as_bytes());

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--deterministic-proof")
        .arg(&proof_path)
        .arg("--deterministic-sandbox-isolation-evidence")
        .arg(&evidence_path)
        .arg("--require-deterministic-release")
        .assert()
        .failure()
        .stderr(predicate::str::contains("deterministic sandbox isolation evidence is not canonical compact JSON"));
}

#[test]
fn release_verify_uses_bundle_local_deterministic_release_artifacts() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let proof_dir = temp.path().join("deterministic-proof-artifacts");
    let (proof_path, evidence_path, proof_digest, evidence_digest) =
        write_deterministic_verify_artifacts(&proof_dir, &manifest);
    install_bundled_deterministic_artifacts(&bundle_dir, &manifest, &proof_path, &evidence_path);

    let assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-deterministic-release")
        .assert()
        .success();
    let stdout = serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap();

    assert_eq!(stdout["deterministic_release"]["status"], "eligible");
    assert_eq!(stdout["deterministic_release"]["proof_source"], "bundled");
    assert_eq!(stdout["deterministic_release"]["proof_digest_blake3"], proof_digest);
    assert_eq!(stdout["deterministic_release"]["sandbox_isolation_evidence_digest_blake3"], evidence_digest);
    assert_eq!(
        stdout["manifest"]["deterministic_build_proof"]["evidence_role"],
        DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE
    );
    assert_eq!(
        stdout["manifest"]["deterministic_sandbox_isolation_evidence"]["evidence_role"],
        DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE
    );
}

#[test]
fn release_verify_reports_external_deterministic_override_source() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let bundled_dir = temp.path().join("bundled-proof-artifacts");
    let (bundled_proof_path, bundled_evidence_path, _bundled_proof_digest, _bundled_evidence_digest) =
        write_deterministic_verify_artifacts(&bundled_dir, &manifest);
    install_bundled_deterministic_artifacts(&bundle_dir, &manifest, &bundled_proof_path, &bundled_evidence_path);
    let external_dir = temp.path().join("external-proof-artifacts");
    let (external_proof_path, external_evidence_path, external_proof_digest, _external_evidence_digest) =
        write_deterministic_verify_artifacts(&external_dir, &manifest);

    let assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--deterministic-proof")
        .arg(&external_proof_path)
        .arg("--deterministic-sandbox-isolation-evidence")
        .arg(&external_evidence_path)
        .arg("--require-deterministic-release")
        .assert()
        .success();
    let stdout = serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap();

    assert_eq!(stdout["deterministic_release"]["status"], "eligible");
    assert_eq!(stdout["deterministic_release"]["proof_source"], "external");
    assert_eq!(stdout["deterministic_release"]["proof_digest_blake3"], external_proof_digest);
    assert_eq!(
        stdout["manifest"]["deterministic_build_proof"]["relative_path"],
        TEST_BUNDLED_DETERMINISTIC_PROOF_PATH
    );
}

#[test]
fn release_verify_requires_bundled_deterministic_artifacts_when_no_override_is_supplied() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-deterministic-release")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing bundled deterministic proof artifact"));
}

#[test]
fn release_verify_rejects_missing_bundled_deterministic_proof_file() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let proof_dir = temp.path().join("deterministic-proof-artifacts");
    let (proof_path, evidence_path, _proof_digest, _evidence_digest) =
        write_deterministic_verify_artifacts(&proof_dir, &manifest);
    install_bundled_deterministic_artifacts(&bundle_dir, &manifest, &proof_path, &evidence_path);
    std::fs::remove_file(bundle_dir.join(TEST_BUNDLED_DETERMINISTIC_PROOF_PATH)).unwrap();

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-deterministic-release")
        .assert()
        .failure()
        .stderr(predicate::str::contains("deterministic_build_proof"));
}

#[test]
fn release_verify_rejects_corrupted_bundled_deterministic_proof_file() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let proof_dir = temp.path().join("deterministic-proof-artifacts");
    let (proof_path, evidence_path, _proof_digest, _evidence_digest) =
        write_deterministic_verify_artifacts(&proof_dir, &manifest);
    install_bundled_deterministic_artifacts(&bundle_dir, &manifest, &proof_path, &evidence_path);
    write_file(&bundle_dir.join(TEST_BUNDLED_DETERMINISTIC_PROOF_PATH), b"corrupted-proof");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-deterministic-release")
        .assert()
        .failure()
        .stderr(predicate::str::contains("deterministic_build_proof does not match manifest"));
}

#[test]
fn release_verify_blocks_bundled_deterministic_proof_digest_mismatch() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let mut wrong_manifest = manifest.clone();
    wrong_manifest.binaries[0].digest_blake3 = sample_digest(7);
    let proof_dir = temp.path().join("wrong-digest-proof-artifacts");
    let (proof_path, evidence_path, _proof_digest, _evidence_digest) =
        write_deterministic_verify_artifacts(&proof_dir, &wrong_manifest);
    install_bundled_deterministic_artifacts(&bundle_dir, &manifest, &proof_path, &evidence_path);

    let output = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-deterministic-release")
        .output()
        .unwrap();
    let stdout = assert_json_release_verify_rejection(
        output,
        "deterministic-release",
        "deterministic proof artifacts do not prove",
    );
    assert_eq!(stdout["deterministic_release"]["status"], "blocked");
    assert_eq!(stdout["deterministic_release"]["proof_source"], "bundled");
}

#[cfg(unix)]
#[test]
fn release_reproduce_writes_matched_report_from_isolated_rebuild_output() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("fake-rebuild.sh");
    let rebuild_output_dir = temp.path().join("rebuild-output");
    let report_path = temp.path().join("reproducibility-report.json");
    write_rebuild_copy_script(&rebuild_script, &manifest.binaries[0].relative_path);

    let assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--report-path")
        .arg(&report_path)
        .assert()
        .success();
    let stdout = serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap();
    let report = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&report_path).unwrap()).unwrap();

    assert_eq!(stdout["release_id"], manifest.release_id);
    assert_eq!(stdout["matched_count"], 1);
    assert_eq!(stdout["mismatched_count"], 0);
    assert_eq!(stdout["missing_count"], 0);
    assert_eq!(report["schema"], "crunch-release-reproducibility-report-v1");
    assert_eq!(report["release_id"], manifest.release_id);
    assert_eq!(report["proof_class"], "self-rebuild-match");
    assert_eq!(report["comparison_verdict"], "matched");
    assert_eq!(report["clean_rebuild_store_identities"][0], rebuild_output_dir.display().to_string());
    assert!(report["environment_assumptions"].as_array().unwrap().len() >= 2);
    assert_eq!(report["evidence_artifact_digests_blake3"].as_array().unwrap().len(), 2);
    assert_eq!(report["artifacts"][0]["name"], manifest.binaries[0].relative_path);
    assert_eq!(report["artifacts"][0]["result"], "matched");
    assert!(rebuild_output_dir.join(&manifest.binaries[0].relative_path).is_file());
}

#[cfg(unix)]
#[test]
fn release_reproduce_writes_deterministic_proof_from_repeated_clean_runs() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("fake-deterministic-rebuild.sh");
    let rebuild_output_dir = temp.path().join("deterministic-main-output");
    let fake_bwrap = temp.path().join("fake-bwrap.sh");
    write_fake_bwrap(&fake_bwrap);
    let fake_bwrap_transcript = temp.path().join("fake-bwrap-transcript.txt");
    let proof_dir = temp.path().join("deterministic-proof-work");
    let report_path = temp.path().join("deterministic-report.json");
    let relative_path = &manifest.binaries[0].relative_path;
    write_rebuild_script(
        &rebuild_script,
        &format!(
            "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\ncp \"$MANTLE_REPRODUCE_BUNDLE_DIR/{relative_path}\" \"$MANTLE_REPRODUCE_OUTPUT_DIR/{relative_path}\"\nif [ -n \"${{MANTLE_DETERMINISTIC_PROOF_STORE_DIR:-}}\" ]; then\n  mkdir -p \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR\"\n  printf '%s\\n' \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR\" > \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/store-marker.txt\"\n  umask > \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/umask.txt\"\n  printf '%s|%s|%s|%s|%s|%s|%s|%s|%s|%s\\n' \"$SOURCE_DATE_EPOCH\" \"$TZ\" \"$LANG\" \"$LC_ALL\" \"$TEMP\" \"$TEMPDIR\" \"$TMP\" \"$TMPDIR\" \"$USER\" \"$LOGNAME\" > \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/normalization-env.txt\"\nfi\n"
        ),
    );

    let assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--report-path")
        .arg(&report_path)
        .arg("--deterministic-proof-runs")
        .arg("2")
        .arg("--deterministic-proof-dir")
        .arg(&proof_dir)
        .env("MANTLE_DETERMINISTIC_PROOF_BWRAP", &fake_bwrap)
        .env("MANTLE_FAKE_BWRAP_TRANSCRIPT", &fake_bwrap_transcript)
        .env("MANTLE_FAKE_BWRAP_FORBIDDEN_BIND", &rebuild_output_dir)
        .assert()
        .success();

    let stdout = serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap();
    let proof_path = proof_dir.join("deterministic-build-proof.json");
    let isolation_evidence_path = proof_dir.join("deterministic-sandbox-isolation-evidence.json");
    assert_eq!(stdout["deterministic_proof_path"], proof_path.display().to_string());
    assert_eq!(
        stdout["deterministic_sandbox_isolation_evidence_path"],
        isolation_evidence_path.display().to_string()
    );
    assert!(stdout["deterministic_proof_digest_blake3"].as_str().unwrap().len() == BLAKE3_HEX_LEN);
    assert_eq!(stdout["deterministic_proof_verdict"], "self-rebuild-match");
    assert_eq!(stdout["deterministic_proof_blockers"], serde_json::json!([]));
    assert_eq!(
        stdout["deterministic_proof_unit"]["target_artifact_identity"],
        format!("release:{}", manifest.release_id)
    );
    assert_eq!(stdout["deterministic_proof_run_roots"].as_array().unwrap().len(), 2);
    assert!(
        stdout["deterministic_proof_sandbox_profiles"][0]
            .as_str()
            .unwrap()
            .starts_with("mantle-proof-sandbox-v1:")
    );
    assert!(stdout["deterministic_sandbox_isolation_evidence_digest_blake3"].as_str().unwrap().len() == BLAKE3_HEX_LEN);
    assert!(proof_path.is_file());
    assert!(isolation_evidence_path.is_file());
    let bundled_proof_path = bundle_dir.join(TEST_BUNDLED_DETERMINISTIC_PROOF_PATH);
    let bundled_evidence_path = bundle_dir.join(TEST_BUNDLED_DETERMINISTIC_SANDBOX_PATH);
    assert!(bundled_proof_path.is_file());
    assert!(bundled_evidence_path.is_file());
    let updated_manifest: ReleaseEvidenceManifest =
        serde_json::from_slice(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).unwrap();
    let bundled_proof = updated_manifest.deterministic_build_proof.as_ref().unwrap();
    let bundled_evidence = updated_manifest.deterministic_sandbox_isolation_evidence.as_ref().unwrap();
    assert_eq!(bundled_proof.relative_path, TEST_BUNDLED_DETERMINISTIC_PROOF_PATH);
    assert_eq!(bundled_proof.evidence_role, DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE);
    assert_eq!(bundled_proof.digest_blake3, stdout["deterministic_proof_digest_blake3"].as_str().unwrap());
    assert_eq!(bundled_evidence.relative_path, TEST_BUNDLED_DETERMINISTIC_SANDBOX_PATH);
    assert_eq!(bundled_evidence.evidence_role, DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE);
    assert_eq!(
        bundled_evidence.digest_blake3,
        stdout["deterministic_sandbox_isolation_evidence_digest_blake3"].as_str().unwrap()
    );
    assert!(proof_dir.join("run-000/store/store-marker.txt").is_file());
    assert!(proof_dir.join("run-001/store/store-marker.txt").is_file());
    assert!(rebuild_output_dir.join(relative_path).is_file());
    for run_id in ["run-000", "run-001"] {
        let store_dir = proof_dir.join(run_id).join("store");
        assert_eq!(std::fs::read_to_string(store_dir.join("umask.txt")).unwrap(), TEST_PROOF_FIXED_UMASK);
        assert_eq!(
            std::fs::read_to_string(store_dir.join("normalization-env.txt")).unwrap(),
            TEST_PROOF_NORMALIZATION_ENV
        );
    }

    let proof = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&proof_path).unwrap()).unwrap();
    let isolation_evidence_bytes = std::fs::read(&isolation_evidence_path).unwrap();
    let isolation_evidence = serde_json::from_slice::<serde_json::Value>(&isolation_evidence_bytes).unwrap();
    let isolation_evidence_file_digest = blake3::hash(&isolation_evidence_bytes).to_hex().to_string();
    assert_eq!(stdout["deterministic_sandbox_isolation_evidence_digest_blake3"], isolation_evidence_file_digest);
    assert_eq!(isolation_evidence["schema"], "mantle-deterministic-sandbox-isolation-evidence-v1");
    assert_eq!(isolation_evidence["profile_family"], "mantle-proof-sandbox-v1");
    assert_eq!(isolation_evidence["evidence_version"], "mantle-release-reproducibility-v1");
    assert_eq!(isolation_evidence["status"], "passed");
    assert!(isolation_evidence["evidence_digest_blake3"].as_str().unwrap().len() == BLAKE3_HEX_LEN);
    assert_eq!(
        isolation_evidence["checks"],
        serde_json::json!([
            "denies-host-network-by-default",
            "denies-main-output-and-proof-store-reuse",
            "denies-undeclared-host-access"
        ])
    );
    assert_eq!(proof["schema"], "mantle-deterministic-proof-receipt-v1");
    assert_eq!(proof["workflow_version"], "mantle-deterministic-proof-receipt-v1");
    assert_eq!(proof["proof_unit"]["target_artifact_identity"], format!("release:{}", manifest.release_id));
    assert_eq!(proof["selected_provider_kind"], manifest.proof_linkage.selected_provider_kind);
    assert_eq!(proof["source_blake3"], manifest.source_archive.digest_blake3);
    assert_eq!(proof["vendor_blake3"], manifest.proof_bundle.digest_blake3);
    assert_eq!(proof["derivation_identity"], format!("release:{}", manifest.release_id));
    assert_eq!(proof["hermeticity_mode"], "strict");
    assert_eq!(proof["physical_store_isolation"], "fresh-store-per-run");
    assert!(proof["sandbox_profile_identities"][0].as_str().unwrap().starts_with("mantle-proof-sandbox-v1:"));
    let runs = proof["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0]["run_id"], "run-000");
    assert_eq!(runs[1]["run_id"], "run-001");
    assert_ne!(runs[0]["output_store_paths"][0], runs[1]["output_store_paths"][0]);
    assert_ne!(runs[0]["output_root_identity"], runs[1]["output_root_identity"]);
    assert!(runs[0]["sandbox_profile_identity"].as_str().unwrap().starts_with("mantle-proof-sandbox-v1:"));
    assert!(runs[1]["sandbox_profile_identity"].as_str().unwrap().starts_with("mantle-proof-sandbox-v1:"));
    assert_eq!(runs[0]["output_digests"][0]["name"], *relative_path);
    assert_eq!(runs[0]["output_digests"], runs[1]["output_digests"]);

    let transcript = std::fs::read_to_string(&fake_bwrap_transcript).unwrap();
    assert!(transcript.contains("--unshare-all"));
    assert!(transcript.contains("--clearenv"));
    assert!(!transcript.contains("--share-net"));
    assert!(!transcript.contains(&rebuild_output_dir.display().to_string()));
    assert!(transcript.contains(&proof_dir.join("run-000/output").display().to_string()));
    assert!(transcript.contains(&proof_dir.join("run-000/store").display().to_string()));
    assert!(transcript.contains(&proof_dir.join("run-001/output").display().to_string()));
    assert!(transcript.contains(&proof_dir.join("run-001/store").display().to_string()));
}

#[cfg(unix)]
#[test]
fn release_reproduce_generated_two_clean_store_proof_verifies_deterministic_release() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("fake-e2e-deterministic-rebuild.sh");
    let rebuild_output_dir = temp.path().join("e2e-deterministic-main-output");
    let fake_bwrap = temp.path().join("fake-bwrap.sh");
    write_fake_bwrap(&fake_bwrap);
    let proof_dir = temp.path().join("e2e-deterministic-proof-work");
    let report_path = temp.path().join("e2e-deterministic-report.json");
    let relative_path = &manifest.binaries[0].relative_path;
    write_rebuild_script(
        &rebuild_script,
        &format!(
            "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\ncp \"$MANTLE_REPRODUCE_BUNDLE_DIR/{relative_path}\" \"$MANTLE_REPRODUCE_OUTPUT_DIR/{relative_path}\"\nif [ -n \"${{MANTLE_DETERMINISTIC_PROOF_STORE_DIR:-}}\" ]; then mkdir -p \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR\"; printf '%s\\n' \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR\" > \"$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/store-marker.txt\"; fi\n"
        ),
    );

    let reproduce_assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--report-path")
        .arg(&report_path)
        .arg("--deterministic-proof-runs")
        .arg("2")
        .arg("--deterministic-proof-dir")
        .arg(&proof_dir)
        .env("MANTLE_DETERMINISTIC_PROOF_BWRAP", &fake_bwrap)
        .env("MANTLE_FAKE_BWRAP_FORBIDDEN_BIND", &rebuild_output_dir)
        .assert()
        .success();
    let reproduce_stdout = serde_json::from_slice::<serde_json::Value>(&reproduce_assert.get_output().stdout).unwrap();
    let proof_path = proof_dir.join("deterministic-build-proof.json");
    let isolation_evidence_path = proof_dir.join("deterministic-sandbox-isolation-evidence.json");
    assert_eq!(reproduce_stdout["deterministic_proof_path"], proof_path.display().to_string());
    assert_eq!(
        reproduce_stdout["deterministic_sandbox_isolation_evidence_path"],
        isolation_evidence_path.display().to_string()
    );

    let proof = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&proof_path).unwrap()).unwrap();
    let runs = proof["runs"].as_array().unwrap();
    assert_eq!(proof["schema"], "mantle-deterministic-proof-receipt-v1");
    assert_eq!(proof["verdict"], "self-rebuild-match", "proof was: {proof}");
    assert_eq!(runs.len(), 2);
    assert_ne!(runs[0]["output_store_paths"][0], runs[1]["output_store_paths"][0]);
    assert_ne!(runs[0]["output_root_identity"], runs[1]["output_root_identity"]);
    assert_eq!(runs[0]["output_digests"], runs[1]["output_digests"]);
    assert!(proof_dir.join("run-000/store/store-marker.txt").is_file());
    assert!(proof_dir.join("run-001/store/store-marker.txt").is_file());

    let verify_assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--deterministic-proof")
        .arg(&proof_path)
        .arg("--deterministic-sandbox-isolation-evidence")
        .arg(&isolation_evidence_path)
        .arg("--require-deterministic-release")
        .assert()
        .success();
    let verify_stdout = serde_json::from_slice::<serde_json::Value>(&verify_assert.get_output().stdout).unwrap();
    assert_eq!(verify_stdout["deterministic_release"]["status"], "eligible");
    assert_eq!(verify_stdout["deterministic_release"]["eligible"], true);
    assert_eq!(
        verify_stdout["deterministic_release"]["proof_digest_blake3"],
        reproduce_stdout["deterministic_proof_digest_blake3"]
    );
    assert_eq!(
        verify_stdout["deterministic_release"]["sandbox_isolation_evidence_digest_blake3"],
        reproduce_stdout["deterministic_sandbox_isolation_evidence_digest_blake3"]
    );

    let bundled_verify_assert = crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-deterministic-release")
        .assert()
        .success();
    let bundled_verify_stdout =
        serde_json::from_slice::<serde_json::Value>(&bundled_verify_assert.get_output().stdout).unwrap();
    assert_eq!(bundled_verify_stdout["deterministic_release"]["status"], "eligible");
    assert_eq!(bundled_verify_stdout["deterministic_release"]["proof_source"], "bundled");
    assert_eq!(
        bundled_verify_stdout["deterministic_release"]["proof_digest_blake3"],
        reproduce_stdout["deterministic_proof_digest_blake3"]
    );
    assert_eq!(
        bundled_verify_stdout["deterministic_release"]["sandbox_isolation_evidence_digest_blake3"],
        reproduce_stdout["deterministic_sandbox_isolation_evidence_digest_blake3"]
    );
}

#[cfg(unix)]
#[test]
fn release_reproduce_denies_host_only_deterministic_proof_recipe() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("host-only-deterministic-rebuild.sh");
    let rebuild_output_dir = temp.path().join("deterministic-main-output-host-only");
    let fake_bwrap = temp.path().join("fake-bwrap.sh");
    write_fake_bwrap(&fake_bwrap);
    let proof_dir = temp.path().join("deterministic-proof-host-only");
    let host_secret = temp.path().join("undeclared-host-secret.txt");
    write_file(&host_secret, b"host-only-secret");
    let relative_path = &manifest.binaries[0].relative_path;
    write_rebuild_script(
        &rebuild_script,
        &format!(
            "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\nif [ -n \"${{MANTLE_DETERMINISTIC_PROOF_STORE_DIR:-}}\" ]; then\n  cat {host_secret} > \"$MANTLE_REPRODUCE_OUTPUT_DIR/{relative_path}\"\nelse\n  cp \"$MANTLE_REPRODUCE_BUNDLE_DIR/{relative_path}\" \"$MANTLE_REPRODUCE_OUTPUT_DIR/{relative_path}\"\nfi\n",
            host_secret = host_secret.display(),
        ),
    );

    crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--deterministic-proof-runs")
        .arg("2")
        .arg("--deterministic-proof-dir")
        .arg(&proof_dir)
        .env("MANTLE_DETERMINISTIC_PROOF_BWRAP", &fake_bwrap)
        .env("MANTLE_FAKE_BWRAP_FORBIDDEN_HOST_PATH", &host_secret)
        .assert()
        .failure()
        .stderr(predicate::str::contains("fake bwrap denied undeclared host path"));

    assert!(!proof_dir.join("deterministic-build-proof.json").exists());
}

#[cfg(unix)]
#[test]
fn release_reproduce_fails_closed_without_deterministic_proof_sandbox() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("fake-deterministic-rebuild.sh");
    let rebuild_output_dir = temp.path().join("deterministic-main-output-missing-sandbox");
    let proof_dir = temp.path().join("deterministic-proof-missing-sandbox");
    write_rebuild_copy_script(&rebuild_script, &manifest.binaries[0].relative_path);

    crunch()
        .current_dir(temp.path())
        .arg("--json")
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--deterministic-proof-runs")
        .arg("2")
        .arg("--deterministic-proof-dir")
        .arg(&proof_dir)
        .env("MANTLE_DETERMINISTIC_PROOF_BWRAP", temp.path().join("missing-bwrap"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("deterministic proof sandbox execution is unavailable"));

    assert!(!proof_dir.join("deterministic-build-proof.json").exists());
}

#[cfg(unix)]
#[test]
fn release_reproduce_fails_when_rebuilt_artifact_is_missing() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("missing-rebuild.sh");
    let rebuild_output_dir = temp.path().join("missing-output");
    let report_path = temp.path().join("missing-report.json");
    write_rebuild_script(&rebuild_script, "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\n");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--report-path")
        .arg(&report_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing rebuilt artifact"));

    let report = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&report_path).unwrap()).unwrap();
    assert_eq!(report["proof_class"], "self-proof-valid");
    assert_eq!(report["comparison_verdict"], "failed");
    assert_eq!(report["artifacts"][0]["result"], "missing-rebuilt-artifact");
}

#[cfg(unix)]
#[test]
fn release_reproduce_rejects_unknown_workflow_version() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("fake-rebuild.sh");
    let rebuild_output_dir = temp.path().join("unknown-workflow-output");
    write_rebuild_copy_script(&rebuild_script, &manifest.binaries[0].relative_path);

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--workflow-version")
        .arg("unreviewed-recipe-v99")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported release reproducibility workflow version"));
}

#[cfg(unix)]
#[test]
fn release_reproduce_fails_on_byte_length_drift() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("length-drift.sh");
    let rebuild_output_dir = temp.path().join("length-output");
    let report_path = temp.path().join("length-report.json");
    write_rebuild_script(
        &rebuild_script,
        &format!(
            "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\nprintf 'short' > \"$MANTLE_REPRODUCE_OUTPUT_DIR/{}\"\n",
            manifest.binaries[0].relative_path
        ),
    );

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--report-path")
        .arg(&report_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("byte-length drift"));

    let report = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&report_path).unwrap()).unwrap();
    assert_eq!(report["artifacts"][0]["result"], "mismatched");
}

#[cfg(unix)]
#[test]
fn release_reproduce_fails_on_digest_drift() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("digest-drift.sh");
    let rebuild_output_dir = temp.path().join("digest-output");
    let report_path = temp.path().join("digest-report.json");
    write_rebuild_script(
        &rebuild_script,
        &format!(
            "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\nprintf 'drift-binary!' > \"$MANTLE_REPRODUCE_OUTPUT_DIR/{}\"\n",
            manifest.binaries[0].relative_path
        ),
    );

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--report-path")
        .arg(&report_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("digest drift"));

    let report = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&report_path).unwrap()).unwrap();
    assert_eq!(report["artifacts"][0]["result"], "mismatched");
}

#[cfg(unix)]
#[test]
fn release_reproduce_fails_on_output_name_drift() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("name-drift.sh");
    let rebuild_output_dir = temp.path().join("name-output");
    let report_path = temp.path().join("name-report.json");
    write_rebuild_script(
        &rebuild_script,
        "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\nprintf 'crunch-binary' > \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries/renamed-crunch\"\n",
    );

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .arg("--report-path")
        .arg(&report_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("output-name drift"));

    assert!(report_path.exists());
}

#[test]
fn release_create_can_package_provider_fixed_point_proof() {
    let (_temp, bundle_dir, manifest) = make_valid_bundle_with_provider();
    let proof = manifest.provider_fixed_point_proof.as_ref().unwrap();

    assert_eq!(proof.relative_path, "proof/provider-fixed-point");
    assert_eq!(proof.evidence_role, PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE);
    assert!(bundle_dir.join("proof/provider-fixed-point/meta.json").exists());
}

#[test]
fn release_create_rejects_invalid_provider_fixed_point_proof() {
    let temp = tempfile::tempdir().unwrap();
    create_minimal_release_repo(temp.path());
    let binary_path = temp.path().join("mantle-bin");
    write_file(&binary_path, b"crunch-binary");
    let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
    let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
    let proof_dir = temp.path().join("proof-input");
    write_full_proof_bundle(&proof_dir, &stage2_digest, &inventory_digest);
    let invalid_provider_dir = temp.path().join("invalid-provider-fixed-point");
    write_file(&invalid_provider_dir.join("meta.json"), br#"{}"#);
    let bundle_dir = temp.path().join("bundle");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg("mantle-0.1.0-rc1")
        .arg("--bundle-dir")
        .arg(&bundle_dir)
        .arg("--binary")
        .arg(&binary_path)
        .arg("--proof-bundle")
        .arg(&proof_dir)
        .arg("--provider-fixed-point-proof")
        .arg(&invalid_provider_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("provider fixed-point proof is invalid"));

    assert!(!bundle_dir.join("manifest.json").exists());
}

#[test]
fn release_verify_required_provider_fixed_point_uses_bundled_proof() {
    let (_temp, bundle_dir, manifest) = make_valid_bundle_with_provider();
    let proof = manifest.provider_fixed_point_proof.as_ref().unwrap();

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-provider-fixed-point-proof")
        .assert()
        .success()
        .stdout(predicate::str::contains("provider fixed-point proof: valid"))
        .stdout(predicate::str::contains("provider fixed-point proof source: bundled"))
        .stdout(predicate::str::contains(format!(
            "provider fixed-point proof artifact digest: {}",
            proof.digest_blake3
        )))
        .stdout(predicate::str::contains("provider fixed-point closure policy digest:"))
        .stdout(predicate::str::contains("provider fixed-point stage binary digest:"))
        .stdout(predicate::str::contains("provider fixed-point non-claim: not-release-reproducibility"))
        .stdout(predicate::str::contains("provider fixed-point non-claim: not-full-cargo-compatibility"));
}

#[test]
fn release_verify_required_provider_fixed_point_fails_when_missing() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();

    let output = crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-provider-fixed-point-proof")
        .output()
        .unwrap();
    assert_human_release_verify_rejection(output, "missing provider fixed-point proof bundle");
}

#[test]
fn release_verify_provider_fixed_point_external_override_reports_source() {
    let (temp, bundle_dir, manifest) = make_valid_bundle_with_provider();
    let external_proof_dir = temp.path().join("external-provider-fixed-point");
    write_provider_fixed_point_proof_bundle(&external_proof_dir, PROVIDER_FIXED_POINT_BINARY_BYTES);

    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-provider-fixed-point-proof")
        .arg("--provider-fixed-point-proof")
        .arg(&external_proof_dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();

    assert_eq!(json["provider_fixed_point_proof"]["status"], "valid");
    assert_eq!(json["provider_fixed_point_proof"]["proof_source"], "external");
    assert_eq!(json["provider_fixed_point_proof"]["proof_dir"], external_proof_dir.display().to_string());
    assert_eq!(
        json["manifest"]["provider_fixed_point_proof"]["relative_path"],
        manifest.provider_fixed_point_proof.as_ref().unwrap().relative_path
    );
}

#[cfg(unix)]
#[test]
fn release_create_can_package_optional_reproducibility_report() {
    let (temp, _bundle_dir, manifest) = make_valid_bundle();
    let report_path = write_matched_default_reproducibility_report(&temp, &_bundle_dir, &manifest);
    let packaged_bundle_dir = temp.path().join("bundle-with-report");

    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("create")
        .arg("--release-id")
        .arg(&manifest.release_id)
        .arg("--bundle-dir")
        .arg(&packaged_bundle_dir)
        .arg("--binary")
        .arg(temp.path().join("mantle-bin"))
        .arg("--proof-bundle")
        .arg(temp.path().join("proof-input"))
        .arg("--reproducibility-report")
        .arg(&report_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("reproducibility report: reproducibility/reproducibility-report.json"));

    let packaged_manifest =
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(packaged_bundle_dir.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(packaged_manifest["claim_scope"], "packaged-integrity-evidence");
    assert!(packaged_manifest.get("reproducible_release").is_none());
    assert_eq!(
        packaged_manifest["reproducibility_report"]["relative_path"],
        "reproducibility/reproducibility-report.json"
    );
    assert!(packaged_bundle_dir.join("reproducibility/reproducibility-report.json").is_file());

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&packaged_bundle_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("reproducibility: matched"));
}

#[test]
fn release_create_without_reproducibility_report_stays_ordinary_bundle() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let manifest =
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(bundle_dir.join("manifest.json")).unwrap()).unwrap();

    assert!(manifest.get("reproducibility_report").is_none());
    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("reproducibility: absent"));
}

#[cfg(unix)]
#[test]
fn release_verify_reports_matched_reproducibility_report() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let report_path = write_matched_default_reproducibility_report(&temp, &bundle_dir, &manifest);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("reproducibility: matched"))
        .stdout(predicate::str::contains(report_path.display().to_string()));
}

#[test]
fn release_verify_json_reports_absent_reproducibility() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();

    let output = crunch().arg("--json").arg("release").arg("verify").arg(&bundle_dir).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();

    assert_eq!(json["kind"], RELEASE_VERIFY_JSON_KIND);
    assert_eq!(json["decision_schema"], RELEASE_VERIFY_DECISION_SCHEMA);
    assert_eq!(json["valid"], true);
    assert_eq!(json["disposition"], "accepted");
    assert_eq!(
        json["checks"].as_array().unwrap().len(),
        crunch_release_core::RELEASE_VERIFICATION_CONTRIBUTOR_COUNT
    );
    assert!(json["diagnostics"].as_array().unwrap().is_empty());
    assert_eq!(json["reproducibility_status"], "absent");
    assert!(json["reproducibility_report"].is_null());
}

#[cfg(unix)]
#[test]
fn release_verify_json_reports_matched_reproducibility() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let report_path = write_matched_default_reproducibility_report(&temp, &bundle_dir, &manifest);

    let output = crunch().arg("--json").arg("release").arg("verify").arg(&bundle_dir).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();

    assert_eq!(json["manifest"]["claim_scope"], "packaged-integrity-evidence");
    assert!(json["manifest"].get("reproducible_release").is_none());
    assert_eq!(json["reproducibility_status"], "matched");
    assert_eq!(json["reproducibility_report"]["path"], report_path.display().to_string());
    assert_eq!(json["reproducibility_report"]["digest_blake3"].as_str().unwrap().len(), BLAKE3_HEX_LEN);
}

// r[verify mantle.operator_diagnostics.release_verification.json_negative]
#[test]
fn release_verify_require_reproducible_fails_when_report_absent() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();

    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-reproducible")
        .output()
        .unwrap();
    let json = assert_json_release_verify_rejection(output, "reproducibility", "reproducibility evidence required");

    assert_eq!(json["reproducibility_status"], "absent");
}

// r[verify mantle.operator_diagnostics.release_verification.fixtures.negative]
#[test]
fn release_verify_human_rejects_missing_required_reproducibility_without_success_marker() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();

    let output = crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-reproducible")
        .output()
        .unwrap();

    assert_human_release_verify_rejection(output, "reproducibility evidence required");
}

// r[verify mantle.operator_diagnostics.release_verification.json_negative]
#[test]
fn release_verify_json_rejects_missing_required_deterministic_proof_once() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-deterministic-release")
        .output()
        .unwrap();

    assert_json_release_verify_rejection(
        output,
        "deterministic-release",
        "missing bundled deterministic proof artifact",
    );
}

// r[verify mantle.operator_diagnostics.release_verification.json_negative]
#[test]
fn release_verify_json_rejects_missing_required_provider_proof_once() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-provider-fixed-point-proof")
        .output()
        .unwrap();

    assert_json_release_verify_rejection(
        output,
        "provider-fixed-point-proof",
        "missing provider fixed-point proof bundle",
    );
}

// r[verify mantle.operator_diagnostics.release_verification.json_negative]
#[test]
fn release_verify_json_rejects_missing_required_stack_provenance_once() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--stack-provenance")
        .arg("required")
        .output()
        .unwrap();

    assert_json_release_verify_rejection(
        output,
        "stack-provenance",
        "required Valence stack provenance sidecar or receipt is missing",
    );
}

// r[verify mantle.operator_diagnostics.release_verification.json_negative]
#[test]
fn release_verify_json_rejects_missing_required_external_role_once() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-external-evidence-role")
        .arg("missing-role")
        .output()
        .unwrap();

    assert_json_release_verify_rejection(
        output,
        "external-evidence-roles",
        "external evidence role required but missing: missing-role",
    );
}

// r[verify mantle.operator_diagnostics.release_verification.json_negative]
#[test]
fn release_verify_json_rejects_external_role_count_above_fixed_limit() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let mut command = crunch();
    command.arg("--json").arg("release").arg("verify").arg(&bundle_dir);
    for role_index in 0..=crunch_release_core::MAX_RELEASE_VERIFICATION_REQUIRED_EXTERNAL_ROLES {
        command.arg("--require-external-evidence-role").arg(format!("missing-role-{role_index}"));
    }

    let output = command.output().unwrap();

    assert_json_release_verify_rejection(
        output,
        "external-evidence-roles",
        "required external evidence role count exceeds limit",
    );
}

// r[verify mantle.operator_diagnostics.release_verification.json_negative]
#[test]
fn release_verify_json_rejects_unsatisfied_stagex_policy_once() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let output = crunch()
        .arg("--json")
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-stagex-no-quorum")
        .output()
        .unwrap();

    assert_json_release_verify_rejection(output, "stagex-no-quorum", "StageX no-quorum profile unsatisfied");
}

// r[verify mantle.operator_diagnostics.release_verification.fixtures.negative]
#[test]
fn release_verify_human_rejects_unsatisfied_stagex_without_success_marker() {
    let (_temp, bundle_dir, _manifest) = make_valid_bundle();
    let output = crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-stagex-no-quorum")
        .output()
        .unwrap();

    assert_human_release_verify_rejection(output, "StageX no-quorum profile unsatisfied");
}

#[cfg(unix)]
#[test]
fn release_verify_json_reports_mismatched_reproducibility() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let rebuild_script = temp.path().join("json-mismatch.sh");
    let rebuild_output_dir = temp.path().join("json-mismatch-output");
    write_rebuild_script(
        &rebuild_script,
        &format!(
            "mkdir -p \"$MANTLE_REPRODUCE_OUTPUT_DIR/binaries\"\nprintf 'drift-binary!' > \"$MANTLE_REPRODUCE_OUTPUT_DIR/{}\"\n",
            manifest.binaries[0].relative_path
        ),
    );
    crunch()
        .current_dir(temp.path())
        .arg("release")
        .arg("reproduce")
        .arg(&bundle_dir)
        .arg("--rebuild-output-dir")
        .arg(&rebuild_output_dir)
        .arg("--rebuild-command")
        .arg(&rebuild_script)
        .assert()
        .failure()
        .stderr(predicate::str::contains("digest drift"));

    let output = crunch().arg("--json").arg("release").arg("verify").arg(&bundle_dir).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();

    assert_eq!(json["reproducibility_status"], "mismatched");
}

#[cfg(unix)]
#[test]
fn release_verify_require_reproducible_accepts_matched_report() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    write_matched_default_reproducibility_report(&temp, &bundle_dir, &manifest);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .arg("--require-reproducible")
        .assert()
        .success()
        .stdout(predicate::str::contains("reproducibility: matched"));
}

#[cfg(unix)]
#[test]
fn release_verify_rejects_non_canonical_reproducibility_report() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let report_path = write_matched_default_reproducibility_report(&temp, &bundle_dir, &manifest);
    let report = serde_json::from_slice::<serde_json::Value>(&std::fs::read(&report_path).unwrap()).unwrap();
    write_file(&report_path, serde_json::to_vec_pretty(&report).unwrap().as_slice());

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("reproducibility report is not canonical compact JSON"));
}

#[cfg(unix)]
#[test]
fn release_verify_rejects_reproducibility_report_linkage_mismatch() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let report_path = write_matched_default_reproducibility_report(&temp, &bundle_dir, &manifest);
    let mut report = read_reproducibility_report(&report_path);
    report.source_archive_digest_blake3 = sample_digest(14);
    write_canonical_reproducibility_report(&report_path, report);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("source_archive_digest_blake3 linkage mismatch"));
}

#[cfg(unix)]
#[test]
fn release_verify_rejects_reproducibility_report_artifact_set_mismatch() {
    let (temp, bundle_dir, manifest) = make_valid_bundle();
    let report_path = write_matched_default_reproducibility_report(&temp, &bundle_dir, &manifest);
    let mut report = read_reproducibility_report(&report_path);
    report.artifacts[0].name = "renamed-crunch".to_string();
    write_canonical_reproducibility_report(&report_path, report);

    crunch()
        .arg("release")
        .arg("verify")
        .arg(&bundle_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("artifact set mismatch"));
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

    assert_eq!(json["release_id"], "mantle-0.1.0-rc1");
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

    assert_eq!(created_json["kind"], "mantle-release-policy-init");
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

    assert_eq!(export_json["kind"], "mantle-witness-request");
    assert_eq!(export_json["release_id"], manifest.release_id);
    assert_eq!(request.schema, "mantle-witness-request-v1");
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

    assert_eq!(duplicate_json["kind"], "mantle-witness-import");
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

    assert_eq!(rebuild_json["kind"], "mantle-witness-rebuild");
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

    assert_eq!(rebuild_json["kind"], "mantle-witness-rebuild");
    assert_eq!(rebuild_json["check_only"], true);
    assert_eq!(rebuild_json["release_id"], manifest.release_id);
    assert!(!scratch_dir.exists(), "helper check mode must stay preflight-only");
    let request = read_witness_request(&request_dir);
    assert!(!request_dir.join(&request.verification_seed_relative_path).join("witnesses").exists());
}

#[test]
fn witness_rebuild_helper_check_accepts_provider_bound_inputs() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    let provider_dir = temp.path().join("provider-out");
    let closure_manifest = temp.path().join("native-toolchain-closure.json");
    std::fs::create_dir_all(&provider_dir).unwrap();
    write_file(&closure_manifest, br#"{}"#);
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
        .env("CRUNCH_WITNESS_RUST_SOURCE_PROVIDER", &provider_dir)
        .env("CRUNCH_WITNESS_TOOLCHAIN_CLOSURE", &closure_manifest)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("provider-bound replay inputs: enabled"));
    assert!(!scratch_dir.exists(), "provider check mode must not create scratch root");
}

#[test]
fn witness_rebuild_helper_provider_bound_driver_passes_default_target() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle_with_provider();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    let witness_config_dir = temp.path().join("witness-config");
    let provider_dir = temp.path().join("provider-out");
    let closure_manifest = temp.path().join("native-toolchain-closure.json");
    let cli_shim_path = temp.path().join("provider-cli-shim.sh");
    let captured_args_path = temp.path().join("provider-cli-args.txt");
    std::fs::create_dir_all(&provider_dir).unwrap();
    write_file(&closure_manifest, br#"{}"#);
    write_fake_provider_cli_capture_shim(&cli_shim_path);

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
        .arg("--scratch-dir")
        .arg(&scratch_dir)
        .arg(&request_dir)
        .arg("--identity")
        .arg("provider-target-test")
        .arg("--system")
        .arg("x86_64-linux")
        .arg("--toolchain")
        .arg("rust-1.91.1")
        .arg("--host-class")
        .arg("nixos-25.05")
        .env("CRUNCH_CONFIG_DIR", &witness_config_dir)
        .env("CRUNCH_TEST_REAL_CLI", env!("CARGO_BIN_EXE_crunch"))
        .env("CRUNCH_TEST_CAPTURE_ARGS", &captured_args_path)
        .env("CRUNCH_WITNESS_REBUILD_CLI_BIN", &cli_shim_path)
        .env("CRUNCH_WITNESS_RUST_SOURCE_PROVIDER", &provider_dir)
        .env("CRUNCH_WITNESS_TOOLCHAIN_CLOSURE", &closure_manifest)
        .output()
        .unwrap();

    assert!(!output.status.success(), "helper unexpectedly succeeded");
    assert!(
        captured_args_path.exists(),
        "provider self-build args were not captured; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let captured_args = std::fs::read_to_string(&captured_args_path).unwrap();
    assert!(captured_args.lines().any(|arg| arg == "self-build"));
    assert!(captured_args.lines().any(|arg| arg == "--fixed-point"));
    assert!(captured_args.lines().any(|arg| arg == "--target"));
    assert!(captured_args.lines().any(|arg| arg == DEFAULT_PROVIDER_WITNESS_TARGET));
}

#[test]
fn witness_rebuild_helper_rejects_incomplete_provider_bound_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let provider_dir = temp.path().join("provider-out");
    let request_dir = temp.path().join("request");
    std::fs::create_dir_all(&provider_dir).unwrap();

    let output = ProcessCommand::new(witness_rebuild_helper_path())
        .current_dir(temp.path())
        .arg("--check")
        .arg(&request_dir)
        .env("CRUNCH_WITNESS_REBUILD_CLI_BIN", env!("CARGO_BIN_EXE_crunch"))
        .env("CRUNCH_WITNESS_RUST_SOURCE_PROVIDER", &provider_dir)
        .output()
        .unwrap();

    assert!(!output.status.success(), "helper unexpectedly succeeded");
    assert!(String::from_utf8_lossy(&output.stderr).contains("CRUNCH_WITNESS_TOOLCHAIN_CLOSURE is required"));
}

#[test]
fn witness_rebuild_helper_rejects_empty_provider_target() {
    let temp = tempfile::tempdir().unwrap();
    let provider_dir = temp.path().join("provider-out");
    let closure_manifest = temp.path().join("native-toolchain-closure.json");
    let request_dir = temp.path().join("request");
    std::fs::create_dir_all(&provider_dir).unwrap();
    write_file(&closure_manifest, br#"{}"#);

    let output = ProcessCommand::new(witness_rebuild_helper_path())
        .current_dir(temp.path())
        .arg("--check")
        .arg(&request_dir)
        .env("CRUNCH_WITNESS_REBUILD_CLI_BIN", env!("CARGO_BIN_EXE_crunch"))
        .env("CRUNCH_WITNESS_RUST_SOURCE_PROVIDER", &provider_dir)
        .env("CRUNCH_WITNESS_TOOLCHAIN_CLOSURE", &closure_manifest)
        .env("CRUNCH_WITNESS_PROVIDER_TARGET", "")
        .output()
        .unwrap();

    assert!(!output.status.success(), "helper unexpectedly succeeded");
    assert!(String::from_utf8_lossy(&output.stderr).contains("CRUNCH_WITNESS_PROVIDER_TARGET must not be empty"));
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
    let launch_signal_path = temp.path().join("driver-launch-signal");
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
        .env(FAKE_WITNESS_DRIVER_MODE_ENV, "match")
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
        .env("SNIX_BUILD_SANDBOX_SHELL", &driver_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let rebuild_json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let audit_path = PathBuf::from(rebuild_json["audit_meta_path"].as_str().unwrap());

    assert_eq!(rebuild_json["kind"], "mantle-witness-rebuild");
    assert_eq!(rebuild_json["check_only"], false);
    assert_eq!(rebuild_json["release_id"], manifest.release_id);
    assert!(Path::new(rebuild_json["attestation_path"].as_str().unwrap()).exists());
    assert!(Path::new(rebuild_json["signature_path"].as_str().unwrap()).exists());
    assert_fake_witness_driver_launch_signal_present(&launch_signal_path);
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
    let launch_signal_path = temp.path().join("driver-launch-signal");
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
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
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
    assert_fake_witness_driver_launch_signal_absent(&launch_signal_path);
}

#[cfg(unix)]
#[test]
fn witness_rebuild_cli_rejects_symlinked_helper_owned_scratch_entry_before_launching_driver() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    let helper_target = temp.path().join("helper-tmp-target");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    let launch_signal_path = temp.path().join("driver-launch-signal");
    write_fake_witness_rebuild_driver(&driver_path);
    std::fs::create_dir_all(&scratch_dir).unwrap();
    std::fs::create_dir_all(&helper_target).unwrap();
    symlink(&helper_target, scratch_dir.join("tmp")).unwrap();

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
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
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
        .stderr(predicate::str::contains("helper-owned scratch entry must not be a symlink"));
    assert_fake_witness_driver_launch_signal_absent(&launch_signal_path);
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
    let launch_signal_path = temp.path().join("driver-launch-signal");
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
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported witness rebuild workflow"));
    assert_fake_witness_driver_launch_signal_absent(&launch_signal_path);
}

#[test]
fn witness_rebuild_cli_rejects_published_output_name_mismatch_before_running_driver() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    let launch_signal_path = temp.path().join("driver-launch-signal");
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
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("published output name mismatch"));
    assert_fake_witness_driver_launch_signal_absent(&launch_signal_path);
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
    let launch_signal_path = temp.path().join("driver-launch-signal");
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
        .env(FAKE_WITNESS_DRIVER_MODE_ENV, "match")
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
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

    assert_eq!(rebuild_json["kind"], "mantle-witness-rebuild");
    assert_eq!(rebuild_json["check_only"], false);
    assert_eq!(rebuild_json["release_id"], manifest.release_id);
    assert!(Path::new(rebuild_json["attestation_path"].as_str().unwrap()).exists());
    assert!(Path::new(rebuild_json["signature_path"].as_str().unwrap()).exists());
    assert_fake_witness_driver_launch_signal_present(&launch_signal_path);
    assert_eq!(audit.schema, "mantle-witness-rebuild-audit-v1");
    assert_eq!(audit.status, "success");
    assert!(
        audit.finished_unix_ms > audit.started_unix_ms,
        "audit timestamps must bracket the rebuild subprocess"
    );
    assert_eq!(audit.rebuilt_outputs.len(), 1);
    assert_eq!(audit.rebuilt_outputs[0].published_name, "binaries/01-mantle-bin");
    assert_eq!(audit.rebuilt_outputs[0].digest_blake3, manifest.binaries[0].digest_blake3);
    assert_eq!(audit.witness_attestation_path.as_deref(), rebuild_json["attestation_path"].as_str());
    assert_eq!(audit.witness_signature_path.as_deref(), rebuild_json["signature_path"].as_str());
    assert!(returned_verification_dir.join("witnesses/witness-a.json").exists());
    let request = read_witness_request(&request_dir);
    assert!(request_dir.join(&request.verification_seed_relative_path).join("release-attestation.json").exists());
    assert!(!request_dir.join(&request.verification_seed_relative_path).join("witnesses").exists());
}

#[test]
fn witness_rebuild_cli_rejects_unmatched_rebuilt_output_digest() {
    let (temp, bundle_dir, _manifest) = make_valid_bundle();
    let signing_key_path = temp.path().join("release.key");
    write_release_signing_key(&signing_key_path);
    let verification_dir = temp.path().join("verification");
    let request_dir = temp.path().join("request");
    let scratch_dir = temp.path().join("scratch");
    let driver_path = temp.path().join("fake-witness-driver.sh");
    let launch_signal_path = temp.path().join("driver-launch-signal");
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
        .env(FAKE_WITNESS_DRIVER_MODE_ENV, "mismatch")
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
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
        .stderr(predicate::str::contains("did not produce proof artifact matching"));

    let audit = read_witness_rebuild_audit(&scratch_dir.join("witness-rebuild-audit/meta.json"));
    assert_eq!(audit.status, "failed");
    assert!(audit.failure_message.unwrap().contains("did not produce proof artifact matching"));
    assert!(!scratch_dir.join("release-verification").join("mantle-0.1.0-rc1").join("witnesses").exists());
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
    let launch_signal_path = temp.path().join("driver-launch-signal");
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
        .env(FAKE_WITNESS_DRIVER_MODE_ENV, "match")
        .env(FAKE_WITNESS_DRIVER_LAUNCH_SIGNAL_ENV, &launch_signal_path)
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

    let rebuilt_binary_path = witness_bundle_dir.join("binaries/01-mantle-bin");
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

    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-mantle-bin");
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
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-mantle-bin");

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
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-mantle-bin");

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
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-mantle-bin");

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

    assert_eq!(created_json["kind"], "mantle-witness-attestation");
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
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-mantle-bin");
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
    let rebuilt_binary_path = bundle_dir.join("binaries").join("01-mantle-bin");
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

    assert_eq!(show_json["kind"], "mantle-release-attestation");
    assert_eq!(show_json["stored_path"], verification_dir.join("release-attestation.json").display().to_string());
    assert_eq!(show_json["attestation"]["release_id"], "mantle-0.1.0-rc1");
}

#[test]
fn attest_release_verify_accepts_matching_agreement_attachment() {
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
    let policy = read_policy(&verification_dir);
    write_matching_agreement_report(&verification_dir, &release_attestation, &policy, &witness_keypair, "witness-a");

    crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("independent-rebuild-agreement"));
}

#[test]
fn attest_release_verify_rejects_mismatched_agreement_attachment() {
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
    let mut policy = read_policy(&verification_dir);
    policy.min_matching_witnesses = 2;
    write_matching_agreement_report(&verification_dir, &release_attestation, &policy, &witness_keypair, "witness-a");

    crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .assert()
        .failure()
        .stderr(predicate::str::contains("independent agreement report digest mismatch"));
}

#[test]
fn attest_release_verify_rejects_duplicate_agreement_attachment() {
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
    let policy = read_policy(&verification_dir);
    let report_path = write_matching_agreement_report(
        &verification_dir,
        &release_attestation,
        &policy,
        &witness_keypair,
        "witness-a",
    );
    let duplicate_path = verification_dir.join("nested/agreement-report.json");
    write_file(&duplicate_path, &std::fs::read(report_path).unwrap());

    crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_keypair.verifying_key.to_string())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ambiguous independent agreement report attachments"));
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
    assert_eq!(show_json["kind"], "mantle-witness-attestation");
    assert_eq!(show_json["attestation"]["witness_identity"], "witness-a");
    assert_eq!(show_json["attestation"]["source_acquisition_mode"], "manual-operator-supplied");

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
    assert_eq!(verify_json["policy_independence_field"], "witness_identity");
    assert_eq!(verify_json["policy_required_witness_count"], 1);
    assert_eq!(verify_json["independent_agreement_status"], "satisfied");
    assert_eq!(verify_json["independent_agreement_class"], "independent-rebuild-agreement");
    assert_eq!(verify_json["independent_agreement_report_digest"].as_str().unwrap().len(), 64);
    assert_eq!(verify_json["independent_agreement_counted_witness_count"], 1);
    assert_eq!(verify_json["independent_agreement_skipped_witness_count"], 0);
    assert_eq!(verify_json["independent_agreement_failed_witness_count"], 0);
    assert_eq!(
        verify_json["independent_agreement_witnesses"][0]["signer_key_name"],
        witness_keypair.verifying_key.name()
    );
    assert_eq!(
        verify_json["independent_agreement_witnesses"][0]["release_attestation_digest_blake3"],
        verify_json["release_attestation_digest"]
    );
    assert_eq!(
        verify_json["independent_agreement_witnesses"][0]["source_acquisition_mode"],
        "manual-operator-supplied"
    );
    assert_eq!(verify_json["independent_agreement_witnesses"][0]["classification_reason"], "counted");
}

#[test]
fn attest_release_verify_reports_two_independent_witness_agreement() {
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
    let witness_a_keypair = crunch_build::generate_keypair().0;
    let witness_b_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&verification_dir, &release_attestation, &witness_a_keypair, "witness-a");
    write_witness_material(&verification_dir, &release_attestation, &witness_b_keypair, "witness-b");
    write_policy(
        &verification_dir,
        release_keypair.verifying_key.name(),
        vec!["witness-a".to_string(), "witness-b".to_string()],
        2,
    );
    write_empty_revocations(&verification_dir);

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_a_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_b_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["independent_agreement_status"], "satisfied");
    assert_eq!(verify_json["independent_agreement_class"], "independent-rebuild-agreement");
    assert_eq!(verify_json["independent_agreement_counted_witness_count"], 2);
    assert_eq!(verify_json["independent_agreement_skipped_witness_count"], 0);
}

#[test]
fn attest_release_verify_reports_unsatisfied_for_same_host_class() {
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
    let witness_a_keypair = crunch_build::generate_keypair().0;
    let witness_b_keypair = crunch_build::generate_keypair().0;
    write_witness_material_with_host_class(
        &verification_dir,
        &release_attestation,
        &witness_a_keypair,
        "witness-a",
        "shared-host",
    );
    write_witness_material_with_host_class(
        &verification_dir,
        &release_attestation,
        &witness_b_keypair,
        "witness-b",
        "shared-host",
    );
    write_policy_with_independence(
        &verification_dir,
        release_keypair.verifying_key.name(),
        vec!["witness-a".to_string(), "witness-b".to_string()],
        2,
        "rebuild_environment_summary.host_class",
    );
    write_empty_revocations(&verification_dir);

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_a_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(witness_b_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["independent_agreement_status"], "unsatisfied");
    assert!(verify_json["independent_agreement_class"].is_null());
    assert_eq!(verify_json["independent_agreement_counted_witness_count"], 1);
    assert_eq!(verify_json["independent_agreement_skipped_witness_count"], 1);
    assert_eq!(
        verify_json["independent_agreement_witnesses"][1]["classification_reason"],
        "duplicate-independence-domain"
    );
}

#[test]
fn attest_release_verify_classifies_invalid_signature_witness() {
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
    let invalid_witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&verification_dir, &release_attestation, &trusted_witness_keypair, "witness-good");
    write_witness_material(&verification_dir, &release_attestation, &invalid_witness_keypair, "witness-bad");
    write_file(
        &verification_dir.join("witnesses/witness-bad.json.sig"),
        format!("{}\n", invalid_signature_line(invalid_witness_keypair.verifying_key.name())).as_bytes(),
    );
    write_policy(
        &verification_dir,
        release_keypair.verifying_key.name(),
        vec!["witness-good".to_string(), "witness-bad".to_string()],
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
        .arg("--trusted-public-key")
        .arg(invalid_witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["independent_agreement_status"], "satisfied");
    assert_eq!(verify_json["independent_agreement_skipped_witness_count"], 1);
    assert_eq!(verify_json["independent_agreement_witnesses"][0]["classification_reason"], "invalid-signature");
}

#[test]
fn attest_release_verify_classifies_malformed_witness_environment() {
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
    let malformed_witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&verification_dir, &release_attestation, &trusted_witness_keypair, "witness-good");
    write_witness_material(&verification_dir, &release_attestation, &malformed_witness_keypair, "witness-malformed");
    let malformed_path = verification_dir.join("witnesses/witness-malformed.json");
    let mut malformed_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&malformed_path).unwrap()).unwrap();
    malformed_json["rebuild_environment_summary"]["host_class"] = serde_json::Value::String(String::new());
    write_file(&malformed_path, serde_json::to_vec(&malformed_json).unwrap().as_slice());
    write_policy(
        &verification_dir,
        release_keypair.verifying_key.name(),
        vec!["witness-good".to_string(), "witness-malformed".to_string()],
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
        .arg("--trusted-public-key")
        .arg(malformed_witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["independent_agreement_status"], "satisfied");
    assert_eq!(verify_json["independent_agreement_failed_witness_count"], 1);
    assert_eq!(
        verify_json["independent_agreement_witnesses"][1]["classification_reason"],
        "malformed-environment-evidence"
    );
}

#[test]
fn attest_release_verify_classifies_revoked_witness() {
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
    let revoked_witness_keypair = crunch_build::generate_keypair().0;
    write_witness_material(&verification_dir, &release_attestation, &trusted_witness_keypair, "witness-good");
    write_witness_material(&verification_dir, &release_attestation, &revoked_witness_keypair, "witness-revoked");
    write_policy(
        &verification_dir,
        release_keypair.verifying_key.name(),
        vec!["witness-good".to_string(), "witness-revoked".to_string()],
        1,
    );
    let revoked_witness: WitnessAttestation =
        serde_json::from_slice(&std::fs::read(verification_dir.join("witnesses/witness-revoked.json")).unwrap())
            .unwrap();
    let revocations = ReleaseRevocations::new(Vec::new(), vec![revoked_witness.canonical_digest().unwrap().to_hex()]);
    write_file(
        &verification_dir.join("revocations.json"),
        serde_json::to_vec_pretty(&revocations).unwrap().as_slice(),
    );

    let verify_output = crunch()
        .arg("attest")
        .arg("release-verify")
        .arg(&verification_dir)
        .arg("--trusted-public-key")
        .arg(release_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(trusted_witness_keypair.verifying_key.to_string())
        .arg("--trusted-public-key")
        .arg(revoked_witness_keypair.verifying_key.to_string())
        .output()
        .unwrap();
    assert!(verify_output.status.success(), "{}", String::from_utf8_lossy(&verify_output.stderr));
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();

    assert_eq!(verify_json["independent_agreement_status"], "satisfied");
    assert_eq!(verify_json["independent_agreement_skipped_witness_count"], 1);
    assert_eq!(verify_json["independent_agreement_witnesses"][1]["classification_reason"], "revoked");
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
    write_file(
        &verification_dir.join("witnesses/witness-unknown.json.sig"),
        format!("{}\n", invalid_signature_line("untrusted-witness-key")).as_bytes(),
    );
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
    assert_eq!(verify_json["independent_agreement_skipped_witness_count"], 1);
    assert_eq!(verify_json["independent_agreement_witnesses"][1]["classification_reason"], "unknown-key");
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
    assert_eq!(verify_json["independent_agreement_failed_witness_count"], 1);
    assert_eq!(verify_json["independent_agreement_witnesses"][0]["classification_reason"], "digest-mismatch");
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
    assert_eq!(verify_json["independent_agreement_skipped_witness_count"], 1);
    assert_eq!(
        verify_json["independent_agreement_witnesses"][0]["classification_reason"],
        "missing-independence-evidence"
    );
}

fn write_canonical_manifest(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest) {
    let core_manifest =
        serde_json::from_value::<crunch_release_core::ReleaseEvidenceManifest>(serde_json::to_value(manifest).unwrap())
            .unwrap();
    let bytes = serde_json::to_vec(&core_manifest).unwrap();
    std::fs::write(bundle_dir.join("manifest.json"), bytes).unwrap();
}

fn hash_directory(path: &Path) -> Result<(u64, String), String> {
    if !path.is_dir() {
        return Err(format!("expected directory artifact: {}", path.display()));
    }
    let mut entries = Vec::new();
    collect_paths_nofollow(path, &mut entries)?;
    entries.sort_by(|left, right| {
        let left_relative = left.strip_prefix(path).expect("collected tree entry is below the root");
        let right_relative = right.strip_prefix(path).expect("collected tree entry is below the root");
        left_relative.as_os_str().as_encoded_bytes().cmp(right_relative.as_os_str().as_encoded_bytes())
    });
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

fn collect_paths_nofollow(root: &Path, entries: &mut Vec<PathBuf>) -> Result<(), String> {
    for child_result in std::fs::read_dir(root).map_err(|err| format!("read_dir {}: {err}", root.display()))? {
        let child = child_result.map_err(|err| format!("read_dir entry {}: {err}", root.display()))?;
        let child_path = child.path();
        let metadata = std::fs::symlink_metadata(&child_path)
            .map_err(|err| format!("symlink_metadata {}: {err}", child_path.display()))?;
        entries.push(child_path.clone());
        if metadata.is_dir() {
            collect_paths_nofollow(&child_path, entries)?;
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
    use std::os::unix::fs::MetadataExt;
    metadata.mode()
}

#[cfg(not(unix))]
fn entry_mode_bits(_metadata: &std::fs::Metadata) -> u32 {
    0
}

type ReleaseEvidenceManifest = crunch_release_core::ReleaseEvidenceManifest;

#[test]
fn release_manifest_schema_constant_matches_fixture_expectation() {
    assert_eq!(RELEASE_EVIDENCE_SCHEMA, "mantle-release-evidence-v1");
}
