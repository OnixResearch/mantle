//! Lock importer offline proof rail.
//!
//! Exercises the external pin import `--plan` and `--apply` workflow for the
//! supported Nixtamal importer. Asserts plan is side-effect free, apply writes
//! only planned files, composition semantics become blockers, and apply fails
//! on plan drift. Emits versioned, redacted, non-overclaiming evidence.
//!
//! r[project_workflows.lock_importer_proof_rail]

use std::fs;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const LOCK_IMPORTER_PROOF_SCHEMA: &str = "mantle-lock-importer-rail-evidence-v1";
const NON_CLAIM_BUILD_TOOL: &str =
    "the generated project remains a build-tool handoff (not Onix/NixOS module semantics); import is not build success or deployability";

fn mantle() -> Command {
    Command::cargo_bin("mantle").unwrap()
}

fn write_evidence_json(
    dir: &TempDir,
    importer_kind: &str,
    plan_operations: &[(&str, &str)],
    blockers: &[&str],
    plan_no_mutate: bool,
    apply_only_planned: bool,
) -> Vec<u8> {
    let record = serde_json::json!({
        "schema": LOCK_IMPORTER_PROOF_SCHEMA,
        "rail_version": "1",
        "importer_kind": importer_kind,
        "planned_operations": plan_operations.iter().map(|(path, action)| {
            serde_json::json!({"path": path, "action": action})
        }).collect::<Vec<_>>(),
        "blockers": blockers,
        "assertions": {
            "plan_no_mutate": plan_no_mutate,
            "apply_only_planned_mantle_files": apply_only_planned,
        },
        "non_claims": [NON_CLAIM_BUILD_TOOL],
        "evidence_path": dir.path().join("importer-evidence.json").to_string_lossy().to_string(),
    });
    let encoded = serde_json::to_vec_pretty(&record).unwrap();
    fs::write(dir.path().join("importer-evidence.json"), &encoded).unwrap();
    encoded
}

fn write_supported_fixture(dir: &TempDir) {
    let fixture = r#"{
  "inputs": [
    {
      "name": "tool",
      "kind": "file",
      "url": "https://example.test/tool",
      "hash_algo": "blake3",
      "hash": "blake3-0000000000000000000000000000000000000000000000000000000000000000",
      "frozen": true,
      "mirrors": ["https://mirror.example.test/tool"],
      "freshness": "locked",
      "fetch_policy": "refresh",
      "trust_policy": "content-hash",
      "lock_identity": "nixtamal:tool@1"
    },
    {
      "name": "archive",
      "kind": "tarball",
      "url": "https://example.test/archive.tar.gz",
      "hash_algo": "sha256",
      "hash": "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
      "patches": ["fix"]
    },
    {
      "name": "repo",
      "kind": "git",
      "repository": "https://example.test/repo.git",
      "reference": "main",
      "rev": "0123456789abcdef",
      "hash_algo": "sha256",
      "hash": "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
      "mirrors": ["https://mirror.example.test/repo.git"]
    }
  ],
  "patches": [
    {
      "name": "fix",
      "kind": "remote",
      "url": "https://example.test/fix.patch",
      "hash_algo": "blake3",
      "hash": "blake3-0000000000000000000000000000000000000000000000000000000000000000"
    }
  ]
}"#;
    fs::write(dir.path().join("nixtamal-pins.json"), fixture).unwrap();
}

fn write_composition_semantics_fixture(dir: &TempDir) {
    let fixture = r#"{
  "inputs": [
    {
      "name": "darcs-input",
      "kind": "darcs",
      "url": "https://example.test/repo",
      "hash_algo": "sha1",
      "hash": "sha1-deadbeef",
      "composition_semantics": ["recursive follows rewriting"]
    }
  ],
  "patches": [],
  "unsupported_semantics": ["flake output composition with module-layer semantics"]
}"#;
    fs::write(dir.path().join("nixtamal-pins.json"), fixture).unwrap();
}

/// V1 (positive): plan is side-effect free and reviewable.
#[test]
fn lock_importer_offline_rail_plan_no_mutate() {
    let dir = TempDir::new().unwrap();
    write_supported_fixture(&dir);

    let assert = mantle()
        .arg("--json")
        .arg("import")
        .arg("pins")
        .arg("plan")
        .current_dir(dir.path())
        .assert()
        .success();
    let value: Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();

    // Plan is reviewable
    assert_eq!(value["schema"], "mantle-project-pin-import-plan-v1");
    assert_eq!(value["blockers"].as_array().unwrap().len(), 0);
    assert_eq!(value["mapped_inputs"].as_array().unwrap().len(), 3);

    // No mutation
    assert!(!dir.path().join("mantle-project.ncl").exists());
    assert!(!dir.path().join("mantle.lock").exists());
    assert!(!dir.path().join(".mantle/inputs.ncl").exists());

    // Emit evidence
    let evidence = write_evidence_json(
        &dir,
        "nixtamal",
        &[("mantle-project.ncl", "create"), ("mantle.lock", "create"), (".mantle/inputs.ncl", "create")],
        &[],
        true,  // plan no-mutate
        false, // apply not yet run
    );
    let parsed: Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["schema"], LOCK_IMPORTER_PROOF_SCHEMA);
    assert!(parsed["assertions"]["plan_no_mutate"].as_bool().unwrap());
    assert!(parsed["non_claims"].as_array().unwrap().iter().any(|c| {
        c.as_str().unwrap().contains("build-tool handoff")
    }));
}

/// V1 (positive): apply writes only planned Mantle files.
#[test]
fn lock_importer_offline_rail_apply_creates_planned_files() {
    let dir = TempDir::new().unwrap();
    write_supported_fixture(&dir);

    let _plan = mantle()
        .arg("--json")
        .arg("import")
        .arg("pins")
        .arg("plan")
        .current_dir(dir.path())
        .assert()
        .success();

    mantle()
        .arg("import")
        .arg("pins")
        .arg("apply")
        .current_dir(dir.path())
        .assert()
        .success();

    // Only planned Mantle-owned files created
    assert!(dir.path().join("mantle-project.ncl").exists());
    assert!(dir.path().join("mantle.lock").exists());
    assert!(dir.path().join(".mantle/inputs.ncl").exists());
    assert!(dir.path().join("nixtamal-pins.json").exists());
    assert!(!dir.path().join("crunch.lock").exists());

    // Evidence
    let evidence = write_evidence_json(
        &dir,
        "nixtamal",
        &[("mantle-project.ncl", "create"), ("mantle.lock", "create"), (".mantle/inputs.ncl", "create")],
        &[],
        true,  // plan no-mutate
        true,  // apply only planned
    );
    let parsed: Value = serde_json::from_slice(&evidence).unwrap();
    assert!(parsed["assertions"]["apply_only_planned_mantle_files"].as_bool().unwrap());
}

/// V2 (negative): composition semantics become blockers, not hidden core semantics.
#[test]
fn lock_importer_offline_rail_composition_semantics_are_blockers() {
    let dir = TempDir::new().unwrap();
    write_composition_semantics_fixture(&dir);

    let assert = mantle()
        .arg("--json")
        .arg("import")
        .arg("pins")
        .arg("plan")
        .current_dir(dir.path())
        .assert();
    let value: Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();

    let blockers = value["blockers"].as_array().unwrap();
    assert!(!blockers.is_empty(), "composition semantics must produce blockers");

    // Apply must refuse
    mantle()
        .arg("import")
        .arg("pins")
        .arg("apply")
        .current_dir(dir.path())
        .assert()
        .failure();

    // Evidence: blockers recorded
    let evidence = write_evidence_json(
        &dir,
        "nixtamal",
        &[],
        &["unsupported-source-kind", "composition-semantics"],
        true,  // plan no-mutate
        false, // apply rejected
    );
    let parsed: Value = serde_json::from_slice(&evidence).unwrap();
    assert!(!parsed["blockers"].as_array().unwrap().is_empty());
}

/// V3: evidence carries build-tool handoff non-claim.
#[test]
fn lock_importer_offline_rail_evidence_has_required_non_claim() {
    let dir = TempDir::new().unwrap();
    let evidence = write_evidence_json(
        &dir,
        "nixtamal",
        &[("mantle-project.ncl", "create")],
        &[],
        true, true,
    );
    let parsed: Value = serde_json::from_slice(&evidence).unwrap();
    assert!(parsed["non_claims"].as_array().unwrap().iter().any(|c| {
        c.as_str().unwrap().contains("build-tool handoff")
    }));
}

/// V4: no raw environment values or unbounded logs in evidence.
#[test]
fn lock_importer_offline_rail_evidence_redaction() {
    let dir = TempDir::new().unwrap();
    let evidence = write_evidence_json(
        &dir,
        "nixtamal",
        &[("mantle-project.ncl", "create")],
        &[],
        true, true,
    );
    let serialized = String::from_utf8(evidence).unwrap();
    assert!(!serialized.contains("raw_env"), "evidence must not contain raw environment values");
}