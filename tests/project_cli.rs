//! CLI smoke tests for project management commands.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

const SOUNDNESS_SCHEMA: &str = "mantle-project-soundness-v1";
const STATIC_SOUNDNESS_MODE: &str = "static";
const GENERATED_INPUT_STALE_CLASS: &str = "generated-input-stale";
const PROBE_TRUST_SOUNDNESS_MODE: &str = "static-with-probes-and-trust-requested";

fn mantle() -> Command {
    Command::cargo_bin("mantle").unwrap()
}

#[test]
fn init_creates_project_files() {
    let dir = TempDir::new().unwrap();

    mantle()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Initialized Mantle project"));

    assert!(dir.path().join("mantle-project.ncl").exists());
    assert!(dir.path().join("mantle.lock").exists());
    assert!(dir.path().join(".mantle/inputs.ncl").exists());

    // .gitignore should have .mantle/
    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".mantle/"));
}

#[test]
fn init_fails_if_already_initialized() {
    let dir = TempDir::new().unwrap();

    // First init succeeds
    mantle().arg("init").current_dir(dir.path()).assert().success();

    // Second init fails
    mantle()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn check_passes_on_fresh_project() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("project check passed"));
}

#[test]
fn check_json_passes_on_fresh_project_with_bounded_non_claims() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    let assert = mantle().arg("--json").arg("check").current_dir(dir.path()).assert().success();
    let output = assert.get_output();
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(output.stderr.is_empty(), "stderr should stay empty in successful JSON mode");
    assert_eq!(value["schema"], SOUNDNESS_SCHEMA);
    assert_eq!(value["mode"]["name"], STATIC_SOUNDNESS_MODE);
    assert_eq!(value["mode"]["network_behavior"], "no-network");
    assert_eq!(value["valid"], true);
    assert_eq!(value["issue_count"], 0);
    assert!(
        value["non_claims"]
            .as_array()
            .unwrap()
            .iter()
            .any(|claim| { claim.as_str().unwrap().contains("does not prove build success") })
    );
}

#[test]
fn check_static_accepts_build_fetch_policy_without_fetching_remote_url() {
    let dir = TempDir::new().unwrap();
    mantle().arg("init").current_dir(dir.path()).assert().success();

    let remote_url = "https://example.invalid/source.tar.gz";
    let expected_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "remote-src",
      kind = {{ type = "tarball", url = {} }},
      hash = {{ algo = 'sha256, expected = {} }},
      fetch_policy = "build-fetch-action",
    }},
  ],
  patches = [],
}}
"#,
        serde_json::to_string(remote_url).unwrap(),
        serde_json::to_string(expected_hash).unwrap(),
    );
    let mut lock = crunch_project::Lockfile::new();
    lock.inputs.insert("remote-src".into(), crunch_project::LockEntry {
        kind: crunch_project::LockedKind::Tarball {
            url: remote_url.to_string(),
        },
        hash: crunch_project::LockedHash {
            algo: crunch_project::HashAlgo::Sha256,
            value: expected_hash.to_string(),
        },
        patches: Vec::new(),
        mirrors: Vec::new(),
        fetch_policy: crunch_project::InputFetchPolicy::BuildFetchAction,
    });
    std::fs::write(dir.path().join("mantle-project.ncl"), manifest).unwrap();
    std::fs::write(dir.path().join("mantle.lock"), lock.clone().to_json().unwrap()).unwrap();
    std::fs::write(dir.path().join(".mantle/inputs.ncl"), crunch_project::generate_inputs_ncl(lock)).unwrap();

    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("project check passed"));
}

#[test]
fn check_json_explicit_probe_and_trust_mode_labels_behavior() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    let assert = mantle()
        .arg("--json")
        .arg("check")
        .arg("--probes")
        .arg("--trust")
        .current_dir(dir.path())
        .assert()
        .success();
    let output = assert.get_output();
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(value["mode"]["name"], PROBE_TRUST_SOUNDNESS_MODE);
    assert!(value["mode"]["network_behavior"].as_str().unwrap().contains("may-contact-network"));
    assert!(value["mode"]["process_behavior"].as_str().unwrap().contains("may-run-commands"));
    assert_eq!(value["valid"], true);
}

#[test]
fn check_json_failure_stdout_is_parseable_report() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();
    std::fs::write(dir.path().join(".mantle/inputs.ncl"), "stale").unwrap();

    let assert = mantle().arg("--json").arg("check").current_dir(dir.path()).assert().failure();
    let output = assert.get_output();
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(output.stderr.is_empty(), "reported JSON failure should not add human stderr");
    assert_eq!(value["schema"], SOUNDNESS_SCHEMA);
    assert_eq!(value["valid"], false);
    assert_eq!(value["issues"][0]["class"], GENERATED_INPUT_STALE_CLASS);
}

#[test]
fn check_fails_without_init() {
    let dir = TempDir::new().unwrap();

    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn check_fails_on_conflicting_legacy_project_files() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();
    std::fs::write(dir.path().join("crunch-project.ncl"), "{}\n").unwrap();
    std::fs::write(dir.path().join("crunch.lock"), "{}\n").unwrap();
    std::fs::create_dir_all(dir.path().join(".crunch")).unwrap();

    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("conflicting legacy Crunch project files"))
        .stderr(predicate::str::contains("crunch-project.ncl conflicts with mantle-project.ncl"))
        .stderr(predicate::str::contains("crunch.lock conflicts with mantle.lock"))
        .stderr(predicate::str::contains(".crunch conflicts with .mantle"));
}

#[test]
fn show_on_empty_project() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    mantle()
        .arg("show")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No locked inputs"));
}

#[test]
fn list_stale_on_empty_project() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    mantle()
        .arg("list-stale")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("all inputs up to date"));
}

#[test]
fn upgrade_on_current_version() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    mantle()
        .arg("upgrade")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("already at current version"));
}

#[test]
fn refresh_on_empty_project() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    mantle()
        .arg("refresh")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("all inputs up to date"));
}

#[test]
fn check_detects_drift() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    // Corrupt the generated file to create drift
    std::fs::write(dir.path().join(".mantle/inputs.ncl"), "stale").unwrap();

    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("drift"));
}

#[test]
fn refresh_hashes_local_patch_relative_to_project_root() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    std::fs::create_dir_all(dir.path().join("patches")).unwrap();
    std::fs::write(dir.path().join("patches/fix.patch"), "diff --git a/x b/x\n").unwrap();
    let source = dir.path().join("pkg.txt");
    std::fs::write(&source, "hello\n").unwrap();
    let url = format!("file://{}", source.display());

    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = "{}" }},
      patches = ["mypatch"],
    }},
  ],
  patches = [
    {{
      name = "mypatch",
      source = {{ type = "local", path = "patches/fix.patch" }},
    }},
  ],
}}
"#,
        url
    );
    std::fs::write(dir.path().join("mantle-project.ncl"), manifest).unwrap();

    mantle()
        .arg("refresh")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("patch lock data updated"));

    let lock_text = std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap();
    let lock = crunch_project::Lockfile::from_json(lock_text).unwrap();
    assert_eq!(lock.inputs["pkg"].patches, vec!["mypatch"]);
    assert_eq!(lock.patches["mypatch"].source, crunch_project::LockedPatchSource::Local {
        path: "patches/fix.patch".into(),
    });
    assert!(lock.patches["mypatch"].hash.value.starts_with("sha256-"));
}

#[test]
fn check_detects_missing_inputs_file() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();

    // Delete the generated file
    std::fs::remove_file(dir.path().join(".mantle/inputs.ncl")).unwrap();

    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}
