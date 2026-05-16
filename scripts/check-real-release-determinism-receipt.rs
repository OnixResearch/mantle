#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "1.8.2"
serde_json = "1"
---

//! Validate real release-specific deterministic proof rail outputs.
//!
//! This checks the files produced by `scripts/prove-real-release-determinism.sh`:
//! the release evidence bundle, deterministic proof receipt, sandbox isolation
//! evidence, and `mantle release verify --require-deterministic-release` JSON.
//! It is intentionally bounded to the packaged artifact proof and does not claim
//! full bootstrap reproducibility.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde_json::Value;
use serde_json::json;

const RELEASE_SCHEMA: &str = "mantle-release-evidence-v1";
const RELEASE_CLAIM_SCOPE: &str = "packaged-integrity-evidence";
const PROOF_SCHEMA: &str = "mantle-deterministic-proof-receipt-v1";
const EFFECT_POLICY_VERSION: &str = "mantle-build-effects-v1";
const PROOF_VERDICT: &str = "self-rebuild-match";
const VERIFY_KIND: &str = "mantle-release-verify-v1";
const SANDBOX_SCHEMA: &str = "mantle-deterministic-sandbox-isolation-evidence-v1";
const SANDBOX_PROFILE_FAMILY: &str = "mantle-proof-sandbox-v1";
const SANDBOX_PROFILE_PREFIX: &str = "mantle-proof-sandbox-v1:";
const SELF_HOSTING_PROOF_SCHEMA: &str = "mantle-self-hosting-proof-v2";
const EXPECTED_PROOF_RUNS: usize = 2;
const HEX64_LEN: usize = 64;
const REQUIRED_SANDBOX_CHECKS: &[&str] = &[
    "denies-host-network-by-default",
    "denies-main-output-and-proof-store-reuse",
    "denies-undeclared-host-access",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args().skip(1))?;
    if args.help {
        print_usage();
        return Ok(());
    }
    if args.self_test {
        return run_self_test();
    }

    let release_bundle =
        args.release_bundle.as_ref().ok_or("release bundle path is required unless --self-test is used")?;
    let paths = ProofPaths::resolve(release_bundle, args.proof_dir.as_deref(), args.verify_receipt.as_deref())?;
    let summary = validate_proof_paths(&paths)?;
    println!("real release determinism proof receipt valid");
    println!("  release: {}", summary.release_id);
    println!("  provider: {}", summary.provider_kind);
    println!("  source BLAKE3: {}", summary.source_blake3);
    println!("  proof bundle BLAKE3: {}", summary.vendor_blake3);
    println!("  artifact digests: {}", summary.artifact_digest_set);
    println!("  proof: {}", paths.proof.display());
    println!("  sandbox evidence: {}", paths.sandbox.display());
    println!("  verify: {}", paths.verify.display());
    println!(
        "  bounded claim: packaged release artifacts rebuilt twice from recorded inputs under recorded {SANDBOX_PROFILE_PREFIX} profiles with matching BLAKE3 digest sets; this is not a full-bootstrap reproducibility claim"
    );
    Ok(())
}

#[derive(Debug)]
struct Args {
    release_bundle: Option<PathBuf>,
    proof_dir: Option<PathBuf>,
    verify_receipt: Option<PathBuf>,
    self_test: bool,
    help: bool,
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where I: Iterator<Item = String> {
        let mut parsed = Args {
            release_bundle: None,
            proof_dir: None,
            verify_receipt: None,
            self_test: false,
            help: false,
        };
        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--proof-dir" => {
                    let value = args.next().ok_or("--proof-dir requires a value")?;
                    parsed.proof_dir = Some(PathBuf::from(value));
                }
                "--verify-receipt" => {
                    let value = args.next().ok_or("--verify-receipt requires a value")?;
                    parsed.verify_receipt = Some(PathBuf::from(value));
                }
                "--self-test" => parsed.self_test = true,
                "-h" | "--help" => parsed.help = true,
                other if other.starts_with('-') => return Err(format!("unknown argument: {other}")),
                path => {
                    if parsed.release_bundle.is_some() {
                        return Err(format!("unexpected extra argument: {path}"));
                    }
                    parsed.release_bundle = Some(PathBuf::from(path));
                }
            }
        }
        Ok(parsed)
    }
}

fn print_usage() {
    println!(
        "Usage: cargo -Zscript scripts/check-real-release-determinism-receipt.rs [--proof-dir DIR] [--verify-receipt JSON] RELEASE_BUNDLE"
    );
    println!("       cargo -Zscript scripts/check-real-release-determinism-receipt.rs --self-test");
}

#[derive(Debug)]
struct ProofPaths {
    manifest: PathBuf,
    proof: PathBuf,
    sandbox: PathBuf,
    verify: PathBuf,
}

impl ProofPaths {
    fn resolve(release_bundle: &Path, proof_dir: Option<&Path>, verify_receipt: Option<&Path>) -> Result<Self, String> {
        let release_bundle = release_bundle.to_path_buf();
        let release_id = release_bundle
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("release bundle has no usable directory name: {}", release_bundle.display()))?;
        let parent = release_bundle.parent().unwrap_or_else(|| Path::new("."));
        let proof_dir = proof_dir.map(Path::to_path_buf).unwrap_or_else(|| parent.join(format!("{release_id}-proof")));
        let verify = verify_receipt
            .map(Path::to_path_buf)
            .unwrap_or_else(|| parent.join(format!("verify-{release_id}.json")));
        Ok(Self {
            manifest: release_bundle.join("manifest.json"),
            proof: proof_dir.join("deterministic-build-proof.json"),
            sandbox: proof_dir.join("deterministic-sandbox-isolation-evidence.json"),
            verify,
        })
    }
}

#[derive(Debug)]
struct ValidationSummary {
    release_id: String,
    provider_kind: String,
    source_blake3: String,
    vendor_blake3: String,
    artifact_digest_set: String,
}

fn validate_proof_paths(paths: &ProofPaths) -> Result<ValidationSummary, String> {
    let manifest = read_json(&paths.manifest)?;
    let proof = read_json(&paths.proof)?;
    let sandbox = read_json(&paths.sandbox)?;
    let verify = read_json(&paths.verify)?;

    validate_release_manifest(&manifest)?;
    validate_deterministic_proof(&manifest, &proof)?;
    validate_sandbox_evidence(&sandbox)?;
    validate_verify_receipt(paths, &manifest, &proof, &sandbox, &verify)?;

    Ok(ValidationSummary {
        release_id: field_str(&manifest, "release_id")?.to_string(),
        provider_kind: field_str(&proof, "selected_provider_kind")?.to_string(),
        source_blake3: field_str(&proof, "source_blake3")?.to_string(),
        vendor_blake3: field_str(&proof, "vendor_blake3")?.to_string(),
        artifact_digest_set: artifact_digest_set_label(runs(&proof)?),
    })
}

fn validate_release_manifest(manifest: &Value) -> Result<(), String> {
    require_str(manifest, "schema", RELEASE_SCHEMA)?;
    require_str(manifest, "claim_scope", RELEASE_CLAIM_SCOPE)?;
    require_non_empty_str(manifest, "release_id")?;
    let workflow = object_field(manifest, "workflow")?;
    require_str(workflow, "version", SELF_HOSTING_PROOF_SCHEMA)?;
    require_digest(
        field_str(object_field(manifest, "source_archive")?, "digest_blake3")?,
        "source_archive.digest_blake3",
    )?;
    require_digest(field_str(object_field(manifest, "proof_bundle")?, "digest_blake3")?, "proof_bundle.digest_blake3")?;
    let linkage = object_field(manifest, "proof_linkage")?;
    require_str(linkage, "proof_bundle_schema", SELF_HOSTING_PROOF_SCHEMA)?;
    require_non_empty_str(linkage, "selected_provider_kind")?;
    let binaries = array_field(manifest, "binaries")?;
    if binaries.is_empty() {
        return Err("manifest.binaries must not be empty".to_string());
    }
    for (idx, binary) in binaries.iter().enumerate() {
        require_non_empty_str(binary, "relative_path").map_err(|err| format!("binaries[{idx}].{err}"))?;
        require_digest(field_str(binary, "digest_blake3")?, &format!("binaries[{idx}].digest_blake3"))?;
    }
    Ok(())
}

fn validate_deterministic_proof(manifest: &Value, proof: &Value) -> Result<(), String> {
    require_str(proof, "schema", PROOF_SCHEMA)?;
    require_str(proof, "workflow_version", PROOF_SCHEMA)?;
    require_str(proof, "effect_policy_version", EFFECT_POLICY_VERSION)?;
    require_effect_set(proof, "declared_effects", &["environment", "read-store", "write-output"])?;
    require_str(proof, "verdict", PROOF_VERDICT)?;
    require_str(proof, "hermeticity_mode", "strict")?;
    require_str(proof, "physical_store_isolation", "fresh-store-per-run")?;
    require_empty_array(proof, "blocking_reasons")?;

    let release_id = field_str(manifest, "release_id")?;
    let proof_unit = object_field(proof, "proof_unit")?;
    require_str(proof_unit, "target_artifact_identity", &format!("release:{release_id}"))?;
    require_str(proof, "derivation_identity", &format!("release:{release_id}"))?;

    let provider = field_str(proof, "selected_provider_kind")?;
    let linkage = object_field(manifest, "proof_linkage")?;
    require_str(linkage, "selected_provider_kind", provider)?;
    require_str(linkage, "source_archive_digest_blake3", field_str(proof, "source_blake3")?)?;
    require_str(linkage, "stage2_binary_digest_blake3", first_manifest_binary_digest(manifest)?)?;
    require_str(object_field(manifest, "source_archive")?, "digest_blake3", field_str(proof, "source_blake3")?)?;
    require_str(object_field(manifest, "proof_bundle")?, "digest_blake3", field_str(proof, "vendor_blake3")?)?;
    require_digest(field_str(proof, "source_blake3")?, "proof.source_blake3")?;
    require_digest(field_str(proof, "vendor_blake3")?, "proof.vendor_blake3")?;

    let run_values = runs(proof)?;
    if run_values.len() != EXPECTED_PROOF_RUNS {
        return Err(format!("expected {EXPECTED_PROOF_RUNS} proof runs, got {}", run_values.len()));
    }
    require_distinct_run_roots(run_values)?;
    require_profiles(proof, run_values)?;
    require_artifact_digest_sets(manifest, proof_unit, run_values)?;
    Ok(())
}

fn validate_sandbox_evidence(sandbox: &Value) -> Result<(), String> {
    require_str(sandbox, "schema", SANDBOX_SCHEMA)?;
    require_str(sandbox, "profile_family", SANDBOX_PROFILE_FAMILY)?;
    require_str(sandbox, "status", "passed")?;
    require_digest(field_str(sandbox, "evidence_digest_blake3")?, "sandbox.evidence_digest_blake3")?;
    let checks = array_field(sandbox, "checks")?;
    for required in REQUIRED_SANDBOX_CHECKS {
        if !checks.iter().any(|check| check.as_str() == Some(required)) {
            return Err(format!("sandbox checks missing required entry {required:?}"));
        }
    }
    Ok(())
}

fn validate_verify_receipt(
    paths: &ProofPaths,
    manifest: &Value,
    proof: &Value,
    sandbox: &Value,
    verify: &Value,
) -> Result<(), String> {
    require_str(verify, "kind", VERIFY_KIND)?;
    require_str(verify, "release_id", field_str(manifest, "release_id")?)?;
    let deterministic = object_field(verify, "deterministic_release")?;
    require_bool(deterministic, "eligible", true)?;
    require_str(deterministic, "status", "eligible")?;
    require_empty_array(deterministic, "blockers")?;
    require_path_field(deterministic, "proof_path", &paths.proof)?;
    require_path_field(deterministic, "sandbox_isolation_evidence_path", &paths.sandbox)?;
    require_str(deterministic, "proof_digest_blake3", &blake3_file(&paths.proof)?)?;
    require_str(deterministic, "sandbox_isolation_evidence_digest_blake3", &blake3_file(&paths.sandbox)?)?;

    // Verify embeds the manifest. Re-check the provider/source/vendor linkage from
    // that copy as well so a mismatched release bundle cannot pass by path alone.
    let verify_manifest = object_field(verify, "manifest")?;
    require_str(verify_manifest, "release_id", field_str(manifest, "release_id")?)?;
    require_str(
        object_field(verify_manifest, "proof_linkage")?,
        "selected_provider_kind",
        field_str(proof, "selected_provider_kind")?,
    )?;
    require_str(
        object_field(verify_manifest, "source_archive")?,
        "digest_blake3",
        field_str(proof, "source_blake3")?,
    )?;
    require_str(object_field(verify_manifest, "proof_bundle")?, "digest_blake3", field_str(proof, "vendor_blake3")?)?;
    let _ = sandbox; // Keep signature explicit: all four output documents are required.
    Ok(())
}

fn require_profiles(proof: &Value, runs: &[Value]) -> Result<(), String> {
    let profiles = array_field(proof, "sandbox_profile_identities")?;
    if profiles.len() != EXPECTED_PROOF_RUNS {
        return Err(format!("expected {EXPECTED_PROOF_RUNS} sandbox profile identities, got {}", profiles.len()));
    }
    for (idx, profile) in profiles.iter().enumerate() {
        let profile = profile.as_str().ok_or_else(|| format!("sandbox_profile_identities[{idx}] is not a string"))?;
        require_sandbox_profile(profile, &format!("sandbox_profile_identities[{idx}]"))?;
    }
    for (idx, run) in runs.iter().enumerate() {
        let profile = field_str(run, "sandbox_profile_identity")?;
        require_sandbox_profile(profile, &format!("runs[{idx}].sandbox_profile_identity"))?;
        if !profiles.iter().any(|known| known.as_str() == Some(profile)) {
            return Err(format!("runs[{idx}].sandbox_profile_identity not present in top-level identities"));
        }
    }
    Ok(())
}

fn require_artifact_digest_sets(manifest: &Value, proof_unit: &Value, runs: &[Value]) -> Result<(), String> {
    let expected_outputs = strings_array(proof_unit, "output_identities")?;
    let manifest_binaries = array_field(manifest, "binaries")?;
    let mut manifest_pairs = Vec::new();
    for binary in manifest_binaries {
        manifest_pairs
            .push((field_str(binary, "relative_path")?.to_string(), field_str(binary, "digest_blake3")?.to_string()));
    }
    let expected_pairs: Vec<(String, String)> = expected_outputs
        .iter()
        .map(|name| {
            manifest_pairs
                .iter()
                .find(|(path, _)| path == name)
                .cloned()
                .ok_or_else(|| format!("proof output identity {name:?} not present in manifest binaries"))
        })
        .collect::<Result<_, _>>()?;

    let first = run_digest_pairs(&runs[0])?;
    if first != expected_pairs {
        return Err(format!(
            "first proof run artifact digest set does not match manifest: expected {expected_pairs:?}, got {first:?}"
        ));
    }
    for (idx, run) in runs.iter().enumerate().skip(1) {
        let actual = run_digest_pairs(run)?;
        if actual != first {
            return Err(format!("proof run {idx} artifact digest set mismatch: expected {first:?}, got {actual:?}"));
        }
    }
    Ok(())
}

fn require_distinct_run_roots(runs: &[Value]) -> Result<(), String> {
    let mut roots = Vec::new();
    let mut stores = Vec::new();
    for (idx, run) in runs.iter().enumerate() {
        require_effect_set(run, "observed_effects", &["environment", "read-store", "write-output"])?;
        let root = field_str(run, "output_root_identity")?;
        if roots.iter().any(|known| known == root) {
            return Err(format!("reused output root identity in proof run {idx}: {root}"));
        }
        roots.push(root.to_string());
        let output_stores = strings_array(run, "output_store_paths")?;
        if output_stores.is_empty() {
            return Err(format!("proof run {idx} missing output_store_paths"));
        }
        for store in output_stores {
            if stores.iter().any(|known| known == &store) {
                return Err(format!("reused proof store path in proof run {idx}: {store}"));
            }
            stores.push(store);
        }
    }
    Ok(())
}

fn run_digest_pairs(run: &Value) -> Result<Vec<(String, String)>, String> {
    let digests = array_field(run, "output_digests")?;
    if digests.is_empty() {
        return Err("proof run output_digests must not be empty".to_string());
    }
    let mut pairs = Vec::new();
    for (idx, digest) in digests.iter().enumerate() {
        let name = field_str(digest, "name").map_err(|err| format!("output_digests[{idx}].{err}"))?;
        let blake3 = field_str(digest, "digest_blake3").map_err(|err| format!("output_digests[{idx}].{err}"))?;
        require_digest(blake3, &format!("output_digests[{idx}].digest_blake3"))?;
        pairs.push((name.to_string(), blake3.to_string()));
    }
    Ok(pairs)
}

fn artifact_digest_set_label(runs: &[Value]) -> String {
    match run_digest_pairs(&runs[0]) {
        Ok(pairs) => format!("{pairs:?}"),
        Err(_) => "<invalid>".to_string(),
    }
}

fn runs(proof: &Value) -> Result<&[Value], String> {
    array_field(proof, "runs")
}

fn first_manifest_binary_digest(manifest: &Value) -> Result<&str, String> {
    let binary = array_field(manifest, "binaries")?
        .first()
        .ok_or_else(|| "manifest.binaries must not be empty".to_string())?;
    field_str(binary, "digest_blake3")
}

fn require_sandbox_profile(profile: &str, field: &str) -> Result<(), String> {
    let digest = profile
        .strip_prefix(SANDBOX_PROFILE_PREFIX)
        .ok_or_else(|| format!("{field} must start with {SANDBOX_PROFILE_PREFIX:?}"))?;
    require_digest(digest, field)
}

fn require_path_field(value: &Value, field: &str, expected: &Path) -> Result<(), String> {
    let actual = PathBuf::from(field_str(value, field)?);
    let expected_abs = expected.canonicalize().unwrap_or_else(|_| expected.to_path_buf());
    let actual_abs = actual.canonicalize().unwrap_or(actual);
    if actual_abs == expected_abs {
        Ok(())
    } else {
        Err(format!("{field} mismatch: expected {}, got {}", expected_abs.display(), actual_abs.display()))
    }
}

fn blake3_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} as JSON: {err}", path.display()))
}

fn object_field<'a>(value: &'a Value, field: &str) -> Result<&'a Value, String> {
    value
        .get(field)
        .filter(|value| value.is_object())
        .ok_or_else(|| format!("missing object field {field}"))
}

fn array_field<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing array field {field}"))
}

fn strings_array(value: &Value, field: &str) -> Result<Vec<String>, String> {
    array_field(value, field)?
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            item.as_str().map(ToString::to_string).ok_or_else(|| format!("{field}[{idx}] is not a string"))
        })
        .collect()
}

fn field_str<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("missing string field {field}"))
}

fn require_str(value: &Value, field: &str, expected: &str) -> Result<(), String> {
    let actual = field_str(value, field)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} mismatch: expected {expected:?}, got {actual:?}"))
    }
}

fn require_bool(value: &Value, field: &str, expected: bool) -> Result<(), String> {
    let actual = value.get(field).and_then(Value::as_bool).ok_or_else(|| format!("missing bool field {field}"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} mismatch: expected {expected}, got {actual}"))
    }
}

fn require_non_empty_str(value: &Value, field: &str) -> Result<(), String> {
    let actual = field_str(value, field)?;
    if actual.trim().is_empty() {
        Err(format!("{field} must not be empty"))
    } else {
        Ok(())
    }
}

fn require_empty_array(value: &Value, field: &str) -> Result<(), String> {
    let array = array_field(value, field)?;
    if array.is_empty() {
        Ok(())
    } else {
        Err(format!("{field} must be empty, got {array:?}"))
    }
}

fn require_effect_set(value: &Value, field: &str, expected: &[&str]) -> Result<(), String> {
    let mut actual = strings_array(value, field)?;
    actual.sort();
    let expected = expected.iter().map(|value| value.to_string()).collect::<Vec<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} mismatch: expected {expected:?}, got {actual:?}"))
    }
}

fn require_digest(digest: &str, field: &str) -> Result<(), String> {
    if digest.len() == HEX64_LEN && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!("{field} must be 64 hex characters"))
    }
}

fn run_self_test() -> Result<(), String> {
    let dir = env::temp_dir().join(format!("mantle-real-release-proof-check-{}-{}", std::process::id(), unix_ms()));
    let release_bundle = dir.join("demo-release");
    let proof_dir = dir.join("demo-release-proof");
    fs::create_dir_all(&release_bundle).map_err(|err| format!("create {}: {err}", release_bundle.display()))?;
    fs::create_dir_all(&proof_dir).map_err(|err| format!("create {}: {err}", proof_dir.display()))?;

    let fixture = Fixture::new(&release_bundle, &proof_dir);
    fixture.write_all()?;
    validate_proof_paths(&fixture.paths)?;

    fixture.write_proof(mutate(&fixture.proof, "selected_provider_kind", json!("source-root")))?;
    assert_rejected("provider kind mismatch", &fixture.paths)?;
    fixture.write_proof(fixture.proof.clone())?;

    let mut mismatched = fixture.proof.clone();
    mismatched["runs"][1]["output_digests"][0]["digest_blake3"] = json!("0".repeat(HEX64_LEN));
    fixture.write_proof(mismatched)?;
    fixture.write_verify_for_current_proof()?;
    assert_rejected("mismatched digest set", &fixture.paths)?;
    fixture.write_proof(fixture.proof.clone())?;
    fixture.write_verify_for_current_proof()?;

    let mut missing_sandbox = fixture.proof.clone();
    missing_sandbox["sandbox_profile_identities"] = json!([]);
    fixture.write_proof(missing_sandbox)?;
    fixture.write_verify_for_current_proof()?;
    assert_rejected("missing sandbox evidence", &fixture.paths)?;
    fixture.write_proof(fixture.proof.clone())?;
    fixture.write_verify_for_current_proof()?;

    fixture.write_proof(mutate(&fixture.proof, "workflow_version", json!("mantle-deterministic-proof-receipt-v2")))?;
    fixture.write_verify_for_current_proof()?;
    assert_rejected("unsupported workflow version", &fixture.paths)?;
    fixture.write_proof(fixture.proof.clone())?;
    fixture.write_verify_for_current_proof()?;

    fixture.write_verify(mutate(
        &fixture.verify()?,
        "deterministic_release",
        json!({"eligible": false, "status": "missing-evidence", "blockers": ["nope"]}),
    ))?;
    assert_rejected("ineligible verify receipt", &fixture.paths)?;

    fs::remove_dir_all(&dir).map_err(|err| format!("remove {}: {err}", dir.display()))?;
    println!("real release determinism receipt checker self-test passed");
    Ok(())
}

struct Fixture {
    paths: ProofPaths,
    manifest: Value,
    proof: Value,
    sandbox: Value,
}

impl Fixture {
    fn new(release_bundle: &Path, proof_dir: &Path) -> Self {
        let source_digest = "a".repeat(HEX64_LEN);
        let vendor_digest = "b".repeat(HEX64_LEN);
        let binary_digest = "c".repeat(HEX64_LEN);
        let prereq_digest = "d".repeat(HEX64_LEN);
        let sandbox_a = format!("{SANDBOX_PROFILE_PREFIX}{}", "1".repeat(HEX64_LEN));
        let sandbox_b = format!("{SANDBOX_PROFILE_PREFIX}{}", "2".repeat(HEX64_LEN));
        let paths = ProofPaths::resolve(
            release_bundle,
            Some(proof_dir),
            Some(&release_bundle.parent().unwrap().join("verify-demo-release.json")),
        )
        .unwrap();
        let manifest = json!({
            "schema": RELEASE_SCHEMA,
            "release_id": "demo-release",
            "claim_scope": RELEASE_CLAIM_SCOPE,
            "workflow": {"command": "./scripts/prove-self-hosting.sh", "version": SELF_HOSTING_PROOF_SCHEMA},
            "source_archive": {"kind": "file", "relative_path": "source/src.tar", "size_bytes": 1, "digest_blake3": source_digest},
            "binaries": [{"kind": "file", "relative_path": "binaries/01-stage2-mantle", "size_bytes": 1, "digest_blake3": binary_digest}],
            "proof_bundle": {"kind": "directory", "relative_path": "proof/self-hosting", "size_bytes": 1, "digest_blake3": vendor_digest},
            "prerequisite_inventory": {"kind": "file", "relative_path": "proof/inventory.md", "size_bytes": 1, "digest_blake3": prereq_digest},
            "proof_linkage": {
                "release_id": "demo-release",
                "source_archive_digest_blake3": source_digest,
                "proof_bundle_schema": SELF_HOSTING_PROOF_SCHEMA,
                "proof_mode": "fixed-point",
                "selected_provider_kind": "legacy-fetch",
                "stage2_binary_digest_blake3": binary_digest,
                "prerequisite_inventory_digest_blake3": prereq_digest,
                "proof_manifest_digest_blake3": "e".repeat(HEX64_LEN),
                "staged_source": "/tmp/staged-source"
            }
        });
        let run = |run_id: &str, profile: &str, root: &str, store: &str| {
            json!({
                "run_id": run_id,
                "perturbation_case": "baseline-clean-env",
                "output_store_paths": [store],
                "output_root_identity": root,
                "sandbox_profile_identity": profile,
                "output_digests": [{"name": "binaries/01-stage2-mantle", "digest_blake3": binary_digest}],
                "substituted_dependency_identities": [],
                "hermeticity_audit_events": [],
                "observed_effects": ["environment", "read-store", "write-output"]
            })
        };
        let proof = json!({
            "schema": PROOF_SCHEMA,
            "proof_unit": {"target_artifact_identity": "release:demo-release", "output_identities": ["binaries/01-stage2-mantle"]},
            "derivation_identity": "release:demo-release",
            "hermeticity_mode": "strict",
            "workflow_version": PROOF_SCHEMA,
            "selected_provider_kind": "legacy-fetch",
            "source_blake3": source_digest,
            "vendor_blake3": vendor_digest,
            "toolchain_provider_identity": "provider-kind=legacy-fetch;command=busybox sh helper",
            "toolchain_stage_roots": [format!("prerequisite-inventory={prereq_digest}"), format!("stage2-binary={binary_digest}"), "staged-source=/tmp/staged-source"],
            "logical_store_prefix": "/mantle/store",
            "physical_store_isolation": "fresh-store-per-run",
            "effect_policy_version": EFFECT_POLICY_VERSION,
            "declared_effects": ["environment", "read-store", "write-output"],
            "normalized_execution_envelope": ["sandbox=bwrap", format!("sandbox-profile={sandbox_a}"), format!("sandbox-profile={sandbox_b}")],
            "ambient_host_perturbations": ["HOME", "PATH"],
            "sandbox_profile_identities": [sandbox_a, sandbox_b],
            "runs": [run("run-000", &sandbox_a, "/tmp/run-000/out", "/tmp/run-000/store"), run("run-001", &sandbox_b, "/tmp/run-001/out", "/tmp/run-001/store")],
            "verdict": PROOF_VERDICT,
            "blocking_reasons": []
        });
        let sandbox = json!({
            "schema": SANDBOX_SCHEMA,
            "profile_family": SANDBOX_PROFILE_FAMILY,
            "evidence_version": "mantle-release-reproducibility-v1",
            "status": "passed",
            "checks": REQUIRED_SANDBOX_CHECKS,
            "evidence_digest_blake3": "f".repeat(HEX64_LEN)
        });
        Self {
            paths,
            manifest,
            proof,
            sandbox,
        }
    }

    fn write_all(&self) -> Result<(), String> {
        write_json(&self.paths.manifest, &self.manifest)?;
        self.write_proof(self.proof.clone())?;
        write_json(&self.paths.sandbox, &self.sandbox)?;
        self.write_verify_for_current_proof()
    }

    fn write_proof(&self, value: Value) -> Result<(), String> {
        write_json(&self.paths.proof, &value)
    }

    fn write_verify(&self, value: Value) -> Result<(), String> {
        write_json(&self.paths.verify, &value)
    }

    fn write_verify_for_current_proof(&self) -> Result<(), String> {
        self.write_verify(self.verify()?)
    }

    fn verify(&self) -> Result<Value, String> {
        let proof_digest = blake3_file(&self.paths.proof)?;
        let sandbox_digest = blake3_file(&self.paths.sandbox)?;
        Ok(json!({
            "kind": VERIFY_KIND,
            "release_id": "demo-release",
            "manifest": self.manifest,
            "reproducibility_status": "absent",
            "deterministic_release": {
                "eligible": true,
                "status": "eligible",
                "blockers": [],
                "proof_path": self.paths.proof,
                "proof_digest_blake3": proof_digest,
                "sandbox_isolation_evidence_path": self.paths.sandbox,
                "sandbox_isolation_evidence_digest_blake3": sandbox_digest
            }
        }))
    }
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|err| format!("serialize JSON: {err}"))?;
    fs::write(path, format!("{text}\n")).map_err(|err| format!("write {}: {err}", path.display()))
}

fn mutate(value: &Value, field: &str, replacement: Value) -> Value {
    let mut copy = value.clone();
    copy.as_object_mut().expect("self-test fixture is object").insert(field.to_string(), replacement);
    copy
}

fn assert_rejected(label: &str, paths: &ProofPaths) -> Result<(), String> {
    if validate_proof_paths(paths).is_ok() {
        Err(format!("self-test expected rejection for {label}"))
    } else {
        Ok(())
    }
}

fn unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}
