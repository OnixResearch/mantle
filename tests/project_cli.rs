//! CLI smoke tests for project management commands.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

#[test]
fn init_creates_project_files() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Initialized crunch project"));

    assert!(dir.path().join("crunch-project.ncl").exists());
    assert!(dir.path().join("crunch.lock").exists());
    assert!(dir.path().join(".crunch/inputs.ncl").exists());

    // .gitignore should have .crunch/
    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".crunch/"));
}

#[test]
fn init_fails_if_already_initialized() {
    let dir = TempDir::new().unwrap();

    // First init succeeds
    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Second init fails
    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn check_passes_on_fresh_project() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    crunch()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("project check passed"));
}

#[test]
fn check_fails_without_init() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn show_on_empty_project() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    crunch()
        .arg("show")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No locked inputs"));
}

#[test]
fn list_stale_on_empty_project() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    crunch()
        .arg("list-stale")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("all inputs up to date"));
}

#[test]
fn upgrade_on_current_version() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    crunch()
        .arg("upgrade")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("already at current version"));
}

#[test]
fn refresh_on_empty_project() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    crunch()
        .arg("refresh")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("all inputs up to date"));
}

#[test]
fn check_detects_drift() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Corrupt the generated file to create drift
    std::fs::write(dir.path().join(".crunch/inputs.ncl"), "stale").unwrap();

    crunch()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("drift"));
}

#[test]
fn check_detects_missing_inputs_file() {
    let dir = TempDir::new().unwrap();

    crunch()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Delete the generated file
    std::fs::remove_file(dir.path().join(".crunch/inputs.ncl")).unwrap();

    crunch()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}
