use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

const PIN_IMPORT_SCHEMA: &str = "mantle-project-pin-import-plan-v1";
const HASH_SHA256: &str = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
const HASH_BLAKE3: &str = "blake3-0000000000000000000000000000000000000000000000000000000000000000";
const FIXTURE_FILE: &str = "nixtamal-pins.json";

fn mantle() -> Command {
    Command::cargo_bin("mantle").unwrap()
}

fn write_supported_nixtamal_fixture(dir: &TempDir) {
    let fixture = format!(
        r#"{{
  "inputs": [
    {{
      "name": "tool",
      "kind": "file",
      "url": "https://example.test/tool",
      "hash_algo": "blake3",
      "hash": "{HASH_BLAKE3}",
      "frozen": true,
      "mirrors": ["https://mirror.example.test/tool"],
      "freshness": "locked",
      "fetch_policy": "refresh",
      "trust_policy": "content-hash",
      "lock_identity": "nixtamal:tool@1"
    }},
    {{
      "name": "archive",
      "kind": "tarball",
      "url": "https://example.test/archive.tar.gz",
      "hash_algo": "sha256",
      "hash": "{HASH_SHA256}",
      "patches": ["fix"]
    }},
    {{
      "name": "repo",
      "kind": "git",
      "repository": "https://example.test/repo.git",
      "reference": "main",
      "rev": "0123456789abcdef",
      "hash_algo": "sha256",
      "hash": "{HASH_SHA256}",
      "mirrors": ["https://mirror.example.test/repo.git"]
    }}
  ],
  "patches": [
    {{
      "name": "fix",
      "kind": "remote",
      "url": "https://example.test/fix.patch",
      "hash_algo": "blake3",
      "hash": "{HASH_BLAKE3}"
    }}
  ]
}}
"#
    );
    std::fs::write(dir.path().join(FIXTURE_FILE), fixture).unwrap();
}

fn write_unsupported_nixtamal_fixture(dir: &TempDir) {
    let fixture = format!(
        r#"{{
  "inputs": [
    {{
      "name": "darcs-input",
      "kind": "darcs",
      "url": "https://example.test/repo",
      "hash_algo": "sha1",
      "hash": "sha1-deadbeef",
      "composition_semantics": ["recursive follows rewriting"]
    }},
    {{
      "name": "missing-patch",
      "kind": "file",
      "url": "https://example.test/file",
      "hash_algo": "sha256",
      "hash": "{HASH_SHA256}",
      "patches": ["unknown"]
    }}
  ],
  "patches": [],
  "unsupported_semantics": ["command freshness probe cannot be represented yet"]
}}
"#
    );
    std::fs::write(dir.path().join(FIXTURE_FILE), fixture).unwrap();
}

// r[verify project_workflows.project_lock_importers]
#[test]
fn pins_import_plan_is_no_mutate_and_reviewable() {
    let dir = TempDir::new().unwrap();
    write_supported_nixtamal_fixture(&dir);

    let assert = mantle()
        .arg("--json")
        .arg("import")
        .arg("pins")
        .arg("plan")
        .current_dir(dir.path())
        .assert()
        .success();
    let value: Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();

    assert_eq!(value["schema"], PIN_IMPORT_SCHEMA);
    assert_eq!(value["blockers"].as_array().unwrap().len(), 0);
    assert_eq!(value["mapped_inputs"].as_array().unwrap().len(), 3);
    assert_eq!(value["mapped_patches"].as_array().unwrap().len(), 1);
    assert!(!dir.path().join("mantle-project.ncl").exists());
    assert!(!dir.path().join("mantle.lock").exists());
    assert!(!dir.path().join(".mantle/inputs.ncl").exists());
}

// r[verify project_workflows.project_lock_importers]
#[test]
fn pins_import_apply_writes_only_planned_mantle_files() {
    let dir = TempDir::new().unwrap();
    write_supported_nixtamal_fixture(&dir);

    mantle()
        .arg("import")
        .arg("pins")
        .arg("apply")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pin import applied"));

    assert!(dir.path().join("mantle-project.ncl").exists());
    assert!(dir.path().join("mantle.lock").exists());
    assert!(dir.path().join(".mantle/inputs.ncl").exists());
    assert!(dir.path().join(FIXTURE_FILE).exists());
    assert!(!dir.path().join("crunch.lock").exists());
    assert!(!dir.path().join("unexpected.txt").exists());

    let lock_text = std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap();
    let lock: crunch_project::Lockfile = crunch_project::Lockfile::from_json(lock_text).unwrap();
    assert_eq!(lock.inputs.len(), 3);
    assert!(lock.patches.contains_key("fix"));

    mantle().arg("check").current_dir(dir.path()).assert().success();
}

#[test]
fn pins_import_apply_matches_reviewed_plan_bytes_and_is_repeatable() {
    let dir = TempDir::new().unwrap();
    write_supported_nixtamal_fixture(&dir);
    let plan = mantle()
        .arg("--json")
        .arg("import")
        .arg("pins")
        .arg("plan")
        .current_dir(dir.path())
        .assert()
        .success();
    let plan: Value = serde_json::from_slice(&plan.get_output().stdout).unwrap();

    for _ in 0..2 {
        mantle().arg("import").arg("pins").arg("apply").current_dir(dir.path()).assert().success();
        for operation in plan["file_operations"].as_array().unwrap() {
            let path = operation["path"].as_str().unwrap();
            let expected = operation["content"].as_str().unwrap();
            assert_eq!(std::fs::read(dir.path().join(path)).unwrap(), expected.as_bytes(), "{path}");
        }
    }
}

// r[verify project_workflows.nixtamal_importer]
#[test]
fn pins_import_apply_fails_before_writing_on_blockers_or_conflicts() {
    let dir = TempDir::new().unwrap();
    write_unsupported_nixtamal_fixture(&dir);
    std::fs::write(dir.path().join("mantle.lock"), "user lock\n").unwrap();

    mantle()
        .arg("import")
        .arg("pins")
        .arg("apply")
        .current_dir(dir.path())
        .assert()
        .code(3)
        .stdout(predicate::str::contains("unsupported-source-kind"))
        .stdout(predicate::str::contains("unsupported-hash-algorithm"))
        .stdout(predicate::str::contains("unknown-patch"))
        .stderr(predicate::str::is_empty());

    let lock_text = std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap();
    assert_eq!(lock_text, "user lock\n");
    assert!(!dir.path().join("mantle-project.ncl").exists());
    assert!(!dir.path().join(".mantle/inputs.ncl").exists());
}

#[cfg(unix)]
#[test]
fn pins_import_rejects_symlink_output_parent_before_any_write() {
    let dir = TempDir::new().unwrap();
    write_supported_nixtamal_fixture(&dir);
    let external = TempDir::new().unwrap();
    std::os::unix::fs::symlink(external.path(), dir.path().join(".mantle")).unwrap();

    mantle()
        .arg("import")
        .arg("pins")
        .arg("apply")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("symlink import output"));

    assert!(!dir.path().join("mantle-project.ncl").exists());
    assert!(!dir.path().join("mantle.lock").exists());
    assert!(!external.path().join("inputs.ncl").exists());
}

#[test]
fn pins_import_rejects_overlapping_output_targets_before_writing() {
    let dir = TempDir::new().unwrap();
    write_supported_nixtamal_fixture(&dir);

    mantle()
        .arg("import")
        .arg("pins")
        .arg("apply")
        .args(["--project-file", ".mantle"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("output targets overlap"));

    assert!(!dir.path().join("mantle.lock").exists());
    assert!(!dir.path().join(".mantle").exists());
}
