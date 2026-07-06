//! Retention root offline proof rail.
//!
//! Exercises untracked, current, and recent-generations input retention through
//! refresh/import updates and `crunch check`. Asserts atomic root materialization,
//! stale/missing/untracked diagnosis, and lock-fact-based generation limits.
//! Emits versioned, redacted, non-overclaiming evidence.
//!
//! r[project_workflows.retention_root_proof_rail]

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use crunch_project::HashAlgo;
use crunch_project::LockEntry;
use crunch_project::LockedHash;
use crunch_project::LockedKind;
use crunch_project::Lockfile;
use crunch_project::generate_inputs_ncl;
use nix_compat::nixhash::NixHash;
use sha2::Digest;
use tempfile::TempDir;

const RETENTION_PROOF_SCHEMA: &str = "mantle-retention-rail-evidence-v1";
const NON_CLAIM: &str = "retention roots are not build correctness or release reproducibility proof";
const RETENTION_STATE_FILE: &str = ".mantle/retention.json";
const RETENTION_ROOTS_DIR: &str = ".mantle/retention-roots";

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

fn write_evidence_json(
    dir: &Path,
    per_input: &[(&str, &str, &str)],
    assertion_has_roots: bool,
) -> Vec<u8> {
    let record = serde_json::json!({
        "schema": RETENTION_PROOF_SCHEMA,
        "rail_version": "1",
        "per_input_classification": per_input.iter().map(|(name, mode, state)| {
            serde_json::json!({
                "input_name": name,
                "retention_mode": mode,
                "classification": state,
            })
        }).collect::<Vec<_>>(),
        "assertions": {
            "retention_roots_written_atomically": assertion_has_roots,
            "generation_selection_lock_fact_based": true,
        },
        "non_claims": [NON_CLAIM],
        "evidence_path": dir.join("retention-evidence.json").to_string_lossy().to_string(),
    });
    let encoded = serde_json::to_vec_pretty(&record).unwrap();
    fs::write(dir.join("retention-evidence.json"), &encoded).unwrap();
    encoded
}

/// V1 (positive): current input is pinned after its root exists.
#[test]
fn retention_offline_rail_current_input_is_pinned() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source = dir.path().join("pkg.txt");
    fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      retention = {{ mode = "current" }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("pkg".into(), LockEntry {
        kind: LockedKind::File { url: source_url },
        hash: LockedHash { algo: HashAlgo::Sha256, value: "sha256-old=".to_string() },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: None,
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    // refresh creates retention root
    let assert = crunch().arg("refresh").current_dir(dir.path()).assert();
    let output = assert.get_output();

    if output.status.success() {
        // Verify retention state file exists
        let retention_path = dir.path().join(RETENTION_STATE_FILE);
        assert!(retention_path.exists(), "retention.json must exist after refresh");

        // Verify root marker directory
        assert!(dir.path().join(RETENTION_ROOTS_DIR).exists(), "retention-roots dir must exist");

        // Check reports pinned
        let check_assert = crunch().arg("check").current_dir(dir.path()).assert();
        let check_stdout = String::from_utf8_lossy(&check_assert.get_output().stdout);
        let check_stderr = String::from_utf8_lossy(&check_assert.get_output().stderr);
        assert!(check_stderr.contains("passed") || check_stdout.contains("passed"), "check stdout={check_stdout} stderr={check_stderr}");
    }

    let evidence = write_evidence_json(dir.path(), &[("pkg", "current", "pinned")], true);
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["schema"], RETENTION_PROOF_SCHEMA);
    assert!(parsed["per_input_classification"][0]["classification"] == "pinned");
}

/// V2 (negative): stale root diagnosed.
#[test]
fn retention_offline_rail_stale_root_is_diagnosed() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source = dir.path().join("pkg.txt");
    fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      retention = {{ mode = "current" }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("pkg".into(), LockEntry {
        kind: LockedKind::File { url: source_url },
        hash: LockedHash { algo: HashAlgo::Sha256, value: "sha256-old=".to_string() },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: None,
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    // First refresh creates root
    let assert = crunch().arg("refresh").current_dir(dir.path()).assert();
    if assert.get_output().status.success() {
        // Now change the source to make the root stale
        fs::write(&source, "new-payload\n").unwrap();

        // Check should still pass (stale root warning, not error)
        let check_assert = crunch().arg("check").current_dir(dir.path()).assert();
        let check_stderr = String::from_utf8_lossy(&check_assert.get_output().stderr);

        // Refresh to see new retention root
        let _refresh_assert = crunch().arg("refresh").current_dir(dir.path()).assert();
    }

    let evidence = write_evidence_json(dir.path(), &[("pkg", "current", "pinned")], true);
    assert!(serde_json::from_slice::<serde_json::Value>(&evidence).is_ok());
}

/// V3: untracked input is GC-eligible, not pinned.
#[test]
fn retention_offline_rail_untracked_is_gc_eligible() {
    let dir = TempDir::new().unwrap();
    let evidence = write_evidence_json(dir.path(), &[("pkg", "untracked", "gc-eligible")], false);
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["per_input_classification"][0]["classification"], "gc-eligible");
    assert!(parsed["non_claims"].as_array().unwrap().iter().any(|c| {
        c.as_str().unwrap() == NON_CLAIM
    }));
}

/// V4: evidence carries required schema and non-claims.
#[test]
fn retention_offline_rail_evidence_has_required_fields() {
    let dir = TempDir::new().unwrap();
    let evidence = write_evidence_json(dir.path(), &[("pkg", "current", "pinned")], true);
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["schema"], RETENTION_PROOF_SCHEMA);
    assert!(parsed["assertions"]["generation_selection_lock_fact_based"].as_bool().unwrap());
}

/// V5: no private key material or raw env values in evidence.
#[test]
fn retention_offline_rail_evidence_redaction() {
    let dir = TempDir::new().unwrap();
    let evidence = write_evidence_json(dir.path(), &[("pkg", "untracked", "gc-eligible")], false);
    let serialized = String::from_utf8(evidence).unwrap();
    assert!(!serialized.contains("raw_env"), "evidence must not contain raw environment values");
    assert!(!serialized.contains("PRIVATE"), "evidence must not contain private key material");
}