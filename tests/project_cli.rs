//! CLI smoke tests for project management commands.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

const SOUNDNESS_SCHEMA: &str = "mantle-project-soundness-v1";
const STATIC_SOUNDNESS_MODE: &str = "static";
const GENERATED_INPUT_STALE_CLASS: &str = "generated-input-stale";
const PROBE_TRUST_SOUNDNESS_MODE: &str = "static-with-probes-and-trust-requested";
const RETENTION_STATE_FILE: &str = ".mantle/retention.json";
const RETENTION_STATE_TMP_FILE: &str = ".mantle/retention.json.tmp";
const RETENTION_ROOTS_DIR: &str = ".mantle/retention-roots";

fn mantle() -> Command {
    Command::cargo_bin("mantle").unwrap()
}

fn write_file_input_manifest(dir: &TempDir, retention: Option<&str>) {
    let source = dir.path().join("pkg.txt");
    std::fs::write(&source, "payload\n").unwrap();
    let url = format!("file://{}", source.display());
    let retention_field = retention.map(|value| format!("  retention = {value},\n")).unwrap_or_default();
    let manifest = format!(
        r#"{{
  version = "1.0.0",
{retention_field}  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
    }},
  ],
  patches = [],
}}
"#,
        serde_json::to_string(&url).unwrap()
    );
    std::fs::write(dir.path().join("mantle-project.ncl"), manifest).unwrap();
}

fn read_retention_state(dir: &TempDir) -> crunch_project::ProjectRetentionState {
    let text = std::fs::read_to_string(dir.path().join(RETENTION_STATE_FILE)).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn persisted_project_files(dir: &TempDir) -> Vec<Vec<u8>> {
    ["mantle.lock", ".mantle/inputs.ncl", RETENTION_STATE_FILE]
        .iter()
        .map(|name| std::fs::read(dir.path().join(name)).unwrap())
        .collect()
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
    assert!(dir.path().join(RETENTION_STATE_FILE).exists());

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
        freshness: None,
        trust: None,
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
    let before = persisted_project_files(&dir);

    mantle()
        .arg("upgrade")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("already at current version"));
    assert_eq!(persisted_project_files(&dir), before);
}

#[test]
fn upgrade_observes_persisted_legacy_lock_and_generated_inputs() {
    let dir = TempDir::new().unwrap();
    mantle().arg("init").current_dir(dir.path()).assert().success();
    let mut legacy = crunch_project::Lockfile::new();
    legacy.version = crunch_project::SchemaVersion::new(0, 9, 0);
    std::fs::write(dir.path().join("mantle.lock"), legacy.to_json().unwrap()).unwrap();
    std::fs::write(dir.path().join(".mantle/inputs.ncl"), "stale generated bindings\n").unwrap();
    std::fs::write(dir.path().join(RETENTION_STATE_FILE), r#"{"schema":"legacy","records":[]}"#).unwrap();

    let output = mantle().arg("upgrade").current_dir(dir.path()).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("upgrade complete"));
    let upgraded =
        crunch_project::Lockfile::from_json(std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap()).unwrap();
    assert_eq!(upgraded.version, crunch_project::SchemaVersion::CURRENT);
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".mantle/inputs.ncl")).unwrap(),
        crunch_project::generate_inputs_ncl(upgraded),
    );
    assert!(read_retention_state(&dir).validate().is_empty());
}

#[test]
fn refresh_on_empty_project() {
    let dir = TempDir::new().unwrap();

    mantle().arg("init").current_dir(dir.path()).assert().success();
    let before = persisted_project_files(&dir);

    mantle()
        .arg("refresh")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("all inputs up to date"));
    assert_eq!(persisted_project_files(&dir), before);
}

#[test]
fn overbound_project_resolution_blocks_without_mutating_project_files() {
    let dir = TempDir::new().unwrap();
    mantle().arg("init").current_dir(dir.path()).assert().success();
    let marker = dir.path().join("resolver-launched");
    let probe = dir.path().join("freshness-probe.sh");
    std::fs::write(&probe, format!("#!/bin/sh\nprintf launched > '{}'\nprintf revision-v1\n", marker.display()))
        .unwrap();
    let probe_arg = serde_json::to_string(&probe.display().to_string()).unwrap();
    let inputs = (0_u32..257).map(|index| {
        let freshness = if index == 0 {
            format!(
                r#", freshness = {{ type = "command", requires_network = false, command = {{ argv = ["/bin/sh", {probe_arg}], cwd = ".", env = [], timeout_ms = 1000, output_limit_bytes = 4096, success_statuses = [0], utf8_required = true }} }}"#
            )
        } else {
            String::new()
        };
        format!(
            r#"{{ name = "pkg{index}", kind = {{ type = "file", url = "file:///unavailable/pkg{index}" }}{freshness} }}"#
        )
    }).collect::<Vec<_>>().join(",\n");
    std::fs::write(
        dir.path().join("mantle-project.ncl"),
        format!("{{ version = \"1.0.0\", inputs = [{inputs}], patches = [] }}\n",),
    )
    .unwrap();
    let before = persisted_project_files(&dir);
    for command in ["refresh", "list-stale"] {
        let output = mantle().arg(command).current_dir(dir.path()).output().unwrap();
        assert!(!output.status.success(), "{command} must enforce the resolver bound");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("257 inputs beyond the 256-input limit"), "{stderr}");
        assert!(!marker.exists(), "{command} launched the resolver before admitting the bound");
        assert_eq!(persisted_project_files(&dir), before);
    }
}

#[test]
fn refresh_inputs_write_fault_retains_partial_lock_without_claiming_completion() {
    let dir = TempDir::new().unwrap();
    mantle().arg("init").current_dir(dir.path()).assert().success();
    write_file_input_manifest(&dir, Some(r#"{ mode = "current" }"#));
    let old_inputs = std::fs::read(dir.path().join(".mantle/inputs.ncl")).unwrap();
    let old_retention = std::fs::read(dir.path().join(RETENTION_STATE_FILE)).unwrap();
    std::fs::create_dir(dir.path().join(".mantle/inputs.ncl.tmp")).unwrap();

    let output = mantle().arg("refresh").current_dir(dir.path()).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("writing temporary .mantle/inputs.ncl"), "{stderr}");
    assert!(!stderr.contains("input(s) updated"), "{stderr}");
    let lock =
        crunch_project::Lockfile::from_json(std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap()).unwrap();
    assert!(lock.inputs.contains_key("pkg"), "lock write completed before generated input fault");
    assert_eq!(std::fs::read(dir.path().join(".mantle/inputs.ncl")).unwrap(), old_inputs);
    assert_eq!(std::fs::read(dir.path().join(RETENTION_STATE_FILE)).unwrap(), old_retention);
}

#[test]
fn refresh_current_retention_writes_root_and_show_reports_pinned() {
    let dir = TempDir::new().unwrap();
    mantle().arg("init").current_dir(dir.path()).assert().success();
    write_file_input_manifest(&dir, Some(r#"{ mode = "current" }"#));

    mantle().arg("refresh").current_dir(dir.path()).assert().success();

    let state = read_retention_state(&dir);
    assert_eq!(state.records.len(), 1);
    let root_path = dir.path().join(RETENTION_ROOTS_DIR).join(format!("{}.json", state.records[0].root_id));
    assert!(root_path.is_file());
    mantle()
        .arg("show")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("retention: pinned"));
    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("project check passed"));
}

#[test]
fn interrupted_retention_state_is_not_reported_as_pinned() {
    let dir = TempDir::new().unwrap();
    mantle().arg("init").current_dir(dir.path()).assert().success();
    write_file_input_manifest(&dir, Some(r#"{ mode = "current" }"#));
    mantle().arg("refresh").current_dir(dir.path()).assert().success();
    std::fs::rename(dir.path().join(RETENTION_STATE_FILE), dir.path().join(RETENTION_STATE_TMP_FILE)).unwrap();

    mantle()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("interrupted retention root"))
        .stderr(predicate::str::contains("project check passed"));
    mantle()
        .arg("show")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("retention: missing-root"));
}

#[test]
fn untracked_input_refresh_remains_gc_eligible_without_roots() {
    let dir = TempDir::new().unwrap();
    mantle().arg("init").current_dir(dir.path()).assert().success();
    write_file_input_manifest(&dir, None);

    mantle().arg("refresh").current_dir(dir.path()).assert().success();

    let state = read_retention_state(&dir);
    assert!(state.records.is_empty());
    assert!(!dir.path().join(RETENTION_ROOTS_DIR).exists());
    mantle()
        .arg("show")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("retention: gc-eligible"));
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
