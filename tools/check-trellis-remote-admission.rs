#!/usr/bin/env -S nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.2"
serde_json = "1.0"
---

//! Validate the pinned Trellis remote-admission oracle and evidence chain.
// machine-artifact-public: remote.trellis-admission-evidence
// r[verify remote_builds.trellis_admission_projection]
// r[verify remote_builds.trellis_admission_evidence_boundary]
// r[verify remote_builds.trellis_admission_claim_boundary]

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

use serde_json::Value;
use serde_json::json;

const REPORT_PATH: &str = "evidence/trellis/remote-admission-v1/report.json";
const POSITIVE_FIXTURE_PATH: &str = "schemas/machine-contracts/fixtures/trellis-remote-admission-evidence.valid.json";
const EVIDENCE_ROOT: &str = "evidence/trellis/remote-admission-v1";
const ORACLE_PATH: &str = "fixtures/trellis-remote-admission/oracle.v1.bin";
const ORACLE_MANIFEST_PATH: &str = "fixtures/trellis-remote-admission/oracle.v1.json";
const POLICY_PATH: &str = "config/trellis-remote-admission.ncl";
const NEGATIVE_FIXTURE_PATH: &str =
    "schemas/machine-contracts/fixtures/trellis-remote-admission-evidence.negatives.json";
const REPORT_SCHEMA: &str = "mantle-trellis-remote-admission-evidence-v1";
const SURFACE_ID: &str = "remote.trellis-admission-evidence";
const ORACLE_SCHEMA: &str = "mantle-trellis-remote-admission-oracle-v1";
const TRELLIS_REPOSITORY: &str = "github.com/OnixResearch/trellis";
const TRELLIS_REVISION: &str = "8de4b24aa2d66cc2e6ec966d686df023492265d3";
const TRELLIS_TREE_OID: &str = "91dace31060cf5195beb72dba71640d9091e67bf";
const TRELLIS_SOURCE_BLAKE3: &str = "e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c";
const TRELLIS_SOURCE_BYTES: u64 = 61_440;
const KAMACITE_REVISION: &str = "de710a092d351e829abfb288d46124e2db8e5b7f";
const KAMACITE_PROFILE: &str = "kamacite.trellis-proof-evidence-profile.v1";
const VALENCE_REVISION: &str = "27b8b2124e12b80718ded124274fec98bed7a581";
const VALENCE_SCHEMA: &str = "trellis.proof-evidence";
const VALENCE_RECEIPT_DOMAIN: &[u8] = b"valence.trellis-proof-evidence.receipt.v1\0";
const SOURCE_SET_DOMAIN: &[u8] = b"mantle.trellis-remote-admission.source-set.v1\0";
const EXPECTED_CASES: u64 = 6_720;
const EXPECTED_RECORD_BYTES: u64 = 7;
const MAX_ARTIFACT_BYTES: u64 = 8_388_608;
const MAX_SOURCE_FILES: usize = 16;
const EXPECTED_PROPERTY_COUNT: usize = 8;
const EXPECTED_REQUIREMENT_COUNT: usize = 8;
const EXPECTED_NON_CLAIM_COUNT: usize = 8;
const CLAIM: &str =
    "the named abstract fenced-attempt safety properties agree for the recorded supported projection cases";
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "not full implementation equivalence",
    "not persistence atomicity",
    "not transport reliability",
    "not cryptographic correctness",
    "not remote worker correctness",
    "not liveness or availability",
    "not whole-build correctness",
    "not release eligibility",
];
const PURE_PROJECTION_PATHS: &[&str] = &[
    "crates/crunch-build/src/distributed/remote_attempt_trellis.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/boundary.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/model.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/outcome.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/projection.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/support.rs",
];
const FORBIDDEN_PROJECTION_MARKERS: &[&str] = &[
    "std::fs",
    "std::env",
    "std::process",
    "std::time",
    "tokio::",
    "reqwest",
    "ureq",
    "println!",
    "eprintln!",
];
const PROJECTION_SOURCE_PATHS: &[&str] = &[
    "crates/crunch-build/src/distributed.rs",
    "crates/crunch-build/src/distributed/remote_attempt.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/boundary.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/model.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/outcome.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/projection.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/support.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/tests.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/tests/fixtures.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/tests/matrix.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/tests/negative.rs",
    "crates/crunch-build/src/distributed/remote_attempt_trellis/tests/positive.rs",
];

struct Options {
    root: PathBuf,
    trellis_root: Option<PathBuf>,
    write_report: bool,
    write_negative_fixtures: bool,
    self_test: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("trellis remote-admission evidence check failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let options = options()?;
    let expected = build_report(&options.root)?;
    if options.write_report {
        write_json(&options.root.join(REPORT_PATH), &expected)?;
        write_json(&options.root.join(POSITIVE_FIXTURE_PATH), &expected)?;
    }
    if options.write_negative_fixtures {
        write_negative_fixtures(&options.root, &expected)?;
    }
    let actual = read_json(&options.root.join(REPORT_PATH))?;
    if actual != expected {
        return Err("report differs from current bounded artifacts".to_string());
    }
    if let Some(trellis_root) = options.trellis_root.as_deref() {
        validate_trellis_source(trellis_root)?;
    }
    if options.self_test {
        run_self_test(&options.root, &expected)?;
    }
    Ok(format!(
        "trellis remote-admission evidence: PASS (cases={EXPECTED_CASES}, source={}, valence=accepted_formal_proof)",
        TRELLIS_SOURCE_BLAKE3
    ))
}

fn options() -> Result<Options, String> {
    let mut root = env::current_dir().map_err(|error| format!("read current directory: {error}"))?;
    let mut trellis_root = None;
    let mut write_report = false;
    let mut write_negative_fixtures = false;
    let mut self_test = false;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => root = PathBuf::from(next_value(&mut arguments, "--root")?),
            "--trellis-root" => {
                trellis_root = Some(PathBuf::from(next_value(&mut arguments, "--trellis-root")?));
            }
            "--write-report" => write_report = true,
            "--write-negative-fixtures" => write_negative_fixtures = true,
            "--self-test" => self_test = true,
            _ => return Err(format!("unsupported argument: {argument}")),
        }
    }
    Ok(Options {
        root,
        trellis_root,
        write_report,
        write_negative_fixtures,
        self_test,
    })
}

fn next_value(arguments: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    arguments.next().ok_or_else(|| format!("{flag} requires a value"))
}

fn build_report(root: &Path) -> Result<Value, String> {
    validate_runtime_boundary(root)?;
    let oracle = read_bounded(&root.join(ORACLE_PATH))?;
    let oracle_manifest = read_json(&root.join(ORACLE_MANIFEST_PATH))?;
    validate_oracle(&oracle, &oracle_manifest)?;
    let proof_ir = read_json(&root.join(EVIDENCE_ROOT).join("proof-ir.json"))?;
    let verifier = read_json(&root.join(EVIDENCE_ROOT).join("verifier-receipt.json"))?;
    validate_proof_inputs(&proof_ir, &verifier)?;
    let kamacite = kamacite_summary(root)?;
    let valence = valence_summary(root)?;
    let projection_counts = value_at(&oracle_manifest, &["projection_counts"])?.clone();
    let supported = u64_at(&oracle_manifest, &["projection_counts", "supported"])?;
    let source_digest = source_set_digest(root, PROJECTION_SOURCE_PATHS)?;
    Ok(json!({
        "schema": REPORT_SCHEMA,
        "trellis": {
            "repository": TRELLIS_REPOSITORY,
            "revision": TRELLIS_REVISION,
            "model_tree_oid": TRELLIS_TREE_OID,
            "source_archive_blake3": TRELLIS_SOURCE_BLAKE3,
            "source_archive_bytes": TRELLIS_SOURCE_BYTES
        },
        "mantle": {
            "projection_source_blake3": source_digest,
            "remote_attempt_source_blake3": file_blake3(&root.join(PROJECTION_SOURCE_PATHS[1]))?,
            "oracle_manifest_blake3": file_blake3(&root.join(ORACLE_MANIFEST_PATH))?,
            "oracle_matrix_blake3": blake3_hex(&oracle),
            "oracle_case_count": EXPECTED_CASES,
            "supported_case_count": supported,
            "unsupported_case_count": EXPECTED_CASES.checked_sub(supported).ok_or("supported count overflow")?,
            "projection_counts": projection_counts,
            "policy_blake3": file_blake3(&root.join(POLICY_PATH))?
        },
        "proof": {
            "proof_ir_blake3": file_blake3(&root.join(EVIDENCE_ROOT).join("proof-ir.json"))?,
            "verifier_receipt_blake3": file_blake3(&root.join(EVIDENCE_ROOT).join("verifier-receipt.json"))?,
            "properties": value_at(&proof_ir, &["properties"])?.clone(),
            "requirement_ids": value_at(&proof_ir, &["requirement_ids"])?.clone(),
            "verification_status": string_at(&verifier, &["verification_status"])?
        },
        "kamacite": kamacite,
        "valence": valence,
        "assumptions": value_at(&proof_ir, &["assumptions"])?.clone(),
        "claims": [CLAIM],
        "non_claims": REQUIRED_NON_CLAIMS,
        "runtime_authority": "unchanged-plan_remote_attempt_report"
    }))
}

fn validate_oracle(bytes: &[u8], manifest: &Value) -> Result<(), String> {
    require_equal(string_at(manifest, &["schema"])?, ORACLE_SCHEMA, "oracle schema")?;
    require_equal(string_at(manifest, &["trellis_revision"])?, TRELLIS_REVISION, "oracle revision")?;
    require_equal(string_at(manifest, &["trellis_source_archive_blake3"])?, TRELLIS_SOURCE_BLAKE3, "oracle source")?;
    require_u64(u64_at(manifest, &["case_count"])?, EXPECTED_CASES, "oracle cases")?;
    require_u64(u64_at(manifest, &["record_bytes"])?, EXPECTED_RECORD_BYTES, "oracle record bytes")?;
    let expected_bytes = EXPECTED_CASES.checked_mul(EXPECTED_RECORD_BYTES).ok_or("oracle byte count overflow")?;
    require_u64(u64::try_from(bytes.len()).map_err(|_| "oracle length overflow")?, expected_bytes, "oracle bytes")?;
    require_equal(string_at(manifest, &["matrix_blake3"])?, &blake3_hex(bytes), "oracle digest")
}

fn validate_proof_inputs(proof: &Value, verifier: &Value) -> Result<(), String> {
    require_equal(string_at(proof, &["repository_revision"])?, TRELLIS_REVISION, "proof revision")?;
    require_equal(string_at(proof, &["source_archive", "blake3"])?, TRELLIS_SOURCE_BLAKE3, "proof source")?;
    require_len(value_at(proof, &["properties"])?, EXPECTED_PROPERTY_COUNT, "properties")?;
    require_len(value_at(proof, &["requirement_ids"])?, EXPECTED_REQUIREMENT_COUNT, "requirement IDs")?;
    require_len(value_at(proof, &["non_claims"])?, EXPECTED_NON_CLAIM_COUNT, "proof non-claims")?;
    require_equal(string_at(verifier, &["verification_status"])?, "passed", "verifier status")?;
    require_u64(u64_at(verifier, &["focused", "verus_errors"])?, 0, "focused Verus errors")?;
    require_u64(u64_at(verifier, &["full", "verus_errors"])?, 0, "full Verus errors")?;
    require_u64(
        u64_at(verifier, &["proof_gap_inventory", "fenced_attempt_external_body"])?,
        0,
        "fenced external bodies",
    )
}

fn kamacite_summary(root: &Path) -> Result<Value, String> {
    let envelope_path = root.join(EVIDENCE_ROOT).join("kamacite-envelope.json");
    let canonical_path = root.join(EVIDENCE_ROOT).join("kamacite-envelope.preserves");
    let projection_path = root.join(EVIDENCE_ROOT).join("kamacite-envelope.compat.json");
    let envelope = read_json(&envelope_path)?;
    let projection = read_json(&projection_path)?;
    require_equal(string_at(&envelope, &["profile", "schema_version"])?, KAMACITE_PROFILE, "Kamacite profile")?;
    require_equal(string_at(&envelope, &["profile", "role"])?, "formal-proof-candidate", "Kamacite role")?;
    require_equal(
        string_at(&projection, &["profile_identity", "hex"])?,
        string_at(&envelope, &["profile_identity", "hex"])?,
        "Kamacite projection identity",
    )?;
    let canonical = read_bounded(&canonical_path)?;
    Ok(json!({
        "revision": KAMACITE_REVISION,
        "profile": KAMACITE_PROFILE,
        "producer_role": "formal-proof-candidate",
        "canonical_path": relative(root, &canonical_path)?,
        "canonical_blake3": blake3_hex(&canonical),
        "canonical_bytes": canonical.len(),
        "profile_identity_blake3": string_at(&envelope, &["profile_identity", "hex"] )?,
        "projection_path": relative(root, &projection_path)?,
        "projection_blake3": file_blake3(&projection_path)?
    }))
}

fn valence_summary(root: &Path) -> Result<Value, String> {
    let path = root.join(EVIDENCE_ROOT).join("valence-acceptance.json");
    let artifact = read_json(&path)?;
    require_equal(string_at(&artifact, &["body", "schema"])?, VALENCE_SCHEMA, "Valence schema")?;
    require_equal(string_at(&artifact, &["body", "valence_revision"])?, VALENCE_REVISION, "Valence revision")?;
    require_equal(string_at(&artifact, &["body", "input", "verification_role"])?, "property", "Valence role")?;
    require_equal(string_at(&artifact, &["body", "report", "outcome"])?, "accepted_formal_proof", "Valence outcome")?;
    if value_at(&artifact, &["body", "report", "valid"])?.as_bool() != Some(true) {
        return Err("Valence report is not valid".to_string());
    }
    require_len(value_at(&artifact, &["body", "report", "issues"])?, 0, "Valence issues")?;
    let body = value_at(&artifact, &["body"])?;
    let receipt_hash = domain_hash(VALENCE_RECEIPT_DOMAIN, &serde_json::to_vec(body).map_err(json_error)?);
    require_equal(string_at(&artifact, &["receipt_hash_blake3"])?, &receipt_hash, "Valence receipt hash")?;
    Ok(json!({
        "revision": VALENCE_REVISION,
        "schema": VALENCE_SCHEMA,
        "validation_role": "property",
        "outcome": "accepted_formal_proof",
        "valid": true,
        "artifact_path": relative(root, &path)?,
        "artifact_blake3": file_blake3(&path)?,
        "receipt_hash_blake3": receipt_hash
    }))
}

fn validate_runtime_boundary(root: &Path) -> Result<(), String> {
    let workspace = read_text(&root.join("Cargo.toml"))?;
    let crate_manifest = read_text(&root.join("crates/crunch-build/Cargo.toml"))?;
    let runtime = read_text(&root.join("src/remote_build.rs"))?;
    if workspace.contains("verified-logic") || crate_manifest.contains("verified-logic") {
        return Err("Trellis entered the ordinary Cargo dependency graph".to_string());
    }
    if !runtime.contains("plan_remote_attempt_report") {
        return Err("runtime admission no longer uses plan_remote_attempt_report".to_string());
    }
    if runtime.contains("project_remote_attempt_to_trellis") || runtime.contains("trellis-proof-envelope") {
        return Err("proof evidence entered runtime admission authority".to_string());
    }
    for relative_path in PURE_PROJECTION_PATHS {
        let source = read_text(&root.join(relative_path))?;
        if let Some(marker) = FORBIDDEN_PROJECTION_MARKERS.iter().find(|marker| source.contains(**marker)) {
            return Err(format!("pure projection contains forbidden authority {marker}: {relative_path}"));
        }
    }
    Ok(())
}

fn validate_trellis_source(root: &Path) -> Result<(), String> {
    let revision = command_text(Command::new("git").arg("-C").arg(root).args(["rev-parse", "HEAD"]))?;
    require_equal(revision.trim(), TRELLIS_REVISION, "Trellis checkout revision")?;
    let tree = command_text(Command::new("git").arg("-C").arg(root).args([
        "ls-tree",
        TRELLIS_REVISION,
        "src/fenced_attempt",
    ]))?;
    if !tree.contains(TRELLIS_TREE_OID) {
        return Err("Trellis model tree OID mismatch".to_string());
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["archive", "--format=tar", TRELLIS_REVISION, "src/fenced_attempt"])
        .output()
        .map_err(|error| format!("run git archive: {error}"))?;
    if !output.status.success() {
        return Err(format!("git archive failed with {}", output.status));
    }
    require_u64(
        u64::try_from(output.stdout.len()).map_err(|_| "archive length overflow")?,
        TRELLIS_SOURCE_BYTES,
        "Trellis archive bytes",
    )?;
    require_equal(&blake3_hex(&output.stdout), TRELLIS_SOURCE_BLAKE3, "Trellis archive digest")
}

fn run_self_test(root: &Path, expected: &Value) -> Result<(), String> {
    let mutations = [
        (["mantle", "projection_source_blake3"], "transition"),
        (["mantle", "oracle_matrix_blake3"], "mapping"),
        (["proof", "proof_ir_blake3"], "proof-identity"),
        (["valence", "validation_role"], "valence-role"),
    ];
    for (path, label) in mutations {
        let mut mutated = expected.clone();
        set_string(&mut mutated, &path, "stale")?;
        if mutated == build_report(root)? {
            return Err(format!("{label} mutation was accepted"));
        }
    }
    let mut assumption = expected.clone();
    value_at_mut(&mut assumption, &["assumptions"])?
        .as_array_mut()
        .ok_or("assumptions is not an array")?
        .clear();
    if assumption == build_report(root)? {
        return Err("assumption mutation was accepted".to_string());
    }
    let mut claim = expected.clone();
    value_at_mut(&mut claim, &["claims"])?.as_array_mut().ok_or("claims is not an array")?[0] =
        Value::String("proves release eligibility".to_string());
    if claim == build_report(root)? {
        return Err("claim mutation was accepted".to_string());
    }
    Ok(())
}

fn write_negative_fixtures(root: &Path, expected: &Value) -> Result<(), String> {
    let mut missing_schema = expected.clone();
    missing_schema.as_object_mut().ok_or("report is not an object")?.remove("schema");
    let mut unsupported_version = expected.clone();
    set_string(&mut unsupported_version, &["schema"], "mantle-trellis-remote-admission-evidence-v0")?;
    let mut malformed_digest = expected.clone();
    set_string(&mut malformed_digest, &["mantle", "projection_source_blake3"], "not-a-digest")?;
    let mut zero_supported = expected.clone();
    *value_at_mut(&mut zero_supported, &["mantle", "supported_case_count"])? = json!(0);
    let mut wrong_role = expected.clone();
    set_string(&mut wrong_role, &["valence", "validation_role"], "recorded_only")?;
    let mut unknown_field = expected.clone();
    unknown_field
        .as_object_mut()
        .ok_or("report is not an object")?
        .insert("authority".to_string(), Value::String("forbidden".to_string()));
    let fixture = json!({
        "surface_id": SURFACE_ID,
        "cases": [
            negative_case("missing-schema", "schema", "/schema", missing_schema),
            negative_case("unsupported-version", "version", "/schema", unsupported_version),
            negative_case(
                "malformed-digest",
                "digest",
                "/mantle/projection_source_blake3",
                malformed_digest,
            ),
            negative_case(
                "zero-supported-count",
                "bounds",
                "/mantle/supported_case_count",
                zero_supported,
            ),
            negative_case(
                "wrong-valence-role",
                "version",
                "/valence/validation_role",
                wrong_role,
            ),
            negative_case(
                "unknown-authority-field",
                "unknown-field",
                "/authority",
                unknown_field,
            )
        ]
    });
    write_json(&root.join(NEGATIVE_FIXTURE_PATH), &fixture)
}

fn negative_case(id: &str, issue_class: &str, expected_path: &str, artifact: Value) -> Value {
    json!({
        "id": id,
        "issue_class": issue_class,
        "expected_path": expected_path,
        "artifact": artifact
    })
}

fn source_set_digest(root: &Path, paths: &[&str]) -> Result<String, String> {
    if paths.is_empty() || paths.len() > MAX_SOURCE_FILES {
        return Err("projection source set count is outside bounds".to_string());
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_SET_DOMAIN);
    for relative_path in paths {
        let bytes = read_bounded(&root.join(relative_path))?;
        hash_frame(&mut hasher, relative_path.as_bytes())?;
        hash_frame(&mut hasher, &bytes)?;
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_frame(hasher: &mut blake3::Hasher, value: &[u8]) -> Result<(), String> {
    let length = u64::try_from(value.len()).map_err(|_| "hash frame length overflow")?;
    hasher.update(&length.to_le_bytes());
    hasher.update(value);
    Ok(())
}

fn command_text(command: &mut Command) -> Result<String, String> {
    let output = command.output().map_err(|error| format!("run command: {error}"))?;
    if !output.status.success() {
        return Err(format!("command failed with {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("decode command output: {error}"))
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_ARTIFACT_BYTES {
        return Err(format!("artifact shape or size is invalid: {}", path.display()));
    }
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn read_text(path: &Path) -> Result<String, String> {
    String::from_utf8(read_bounded(path)?).map_err(|error| format!("decode {}: {error}", path.display()))
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&read_bounded(path)?).map_err(json_error)
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(json_error)?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn file_blake3(path: &Path) -> Result<String, String> {
    read_bounded(path).map(|bytes| blake3_hex(&bytes))
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn domain_hash(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Result<&'a Value, String> {
    path.iter().try_fold(value, |current, key| {
        current.get(*key).ok_or_else(|| format!("missing JSON field {}", path.join(".")))
    })
}

fn value_at_mut<'a>(value: &'a mut Value, path: &[&str]) -> Result<&'a mut Value, String> {
    let joined = path.join(".");
    let mut current = value;
    for key in path {
        current = current.get_mut(*key).ok_or_else(|| format!("missing JSON field {joined}"))?;
    }
    Ok(current)
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Result<&'a str, String> {
    value_at(value, path)?
        .as_str()
        .ok_or_else(|| format!("JSON field {} is not a string", path.join(".")))
}

fn u64_at(value: &Value, path: &[&str]) -> Result<u64, String> {
    value_at(value, path)?
        .as_u64()
        .ok_or_else(|| format!("JSON field {} is not an unsigned integer", path.join(".")))
}

fn set_string(value: &mut Value, path: &[&str], replacement: &str) -> Result<(), String> {
    *value_at_mut(value, path)? = Value::String(replacement.to_string());
    Ok(())
}

fn require_equal(actual: &str, expected: &str, label: &str) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label} mismatch: expected {expected}, got {actual}"))
    }
}

fn require_u64(actual: u64, expected: u64, label: &str) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label} mismatch: expected {expected}, got {actual}"))
    }
}

fn require_len(value: &Value, expected: usize, label: &str) -> Result<(), String> {
    let actual = value.as_array().ok_or_else(|| format!("{label} is not an array"))?.len();
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label} count mismatch: expected {expected}, got {actual}"))
    }
}

fn relative(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map(|value| value.to_string_lossy().into_owned())
        .map_err(|_| format!("{} is outside {}", path.display(), root.display()))
}

fn json_error(error: serde_json::Error) -> String {
    format!("JSON error: {error}")
}
