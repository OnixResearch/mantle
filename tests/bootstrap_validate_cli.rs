use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap").arg("--version").output().is_ok_and(|o| o.status.success())
}

fn write_minimal_derivation(path: &Path) {
    let lib_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib").join("lib.ncl");
    fs::write(
        path,
        format!(
            r#"let crunch = import "{}" in
{{
  name = "bootstrap-validate-test",
  builder = "/bin/sh",
  args = ["-c", "echo hi > $out"],
  addressing_mode = 'input-addressed,
}} | crunch.Derivation"#,
            lib_path.display()
        ),
    )
    .unwrap();
}

fn write_success_fixture(
    root: &Path,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let state_dir = root.join("state");
    let store_dir = root.join("store");
    let evidence_dir = root.join("evidence");
    let target = root.join("success.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    write_minimal_derivation(&target);
    (state_dir, store_dir, evidence_dir, target)
}

#[test]
fn bootstrap_validate_preflight_failure_writes_evidence_bundle() {
    let root = TempDir::new().unwrap();
    let state_dir = root.path().join("state");
    let store_file = root.path().join("store-file");
    let evidence_dir = root.path().join("evidence");
    let target = root.path().join("make-tcc.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(&store_file, "not a directory").unwrap();
    write_minimal_derivation(&target);

    crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_file)
        .arg("bootstrap")
        .arg("validate")
        .arg(&target)
        .arg("--evidence-dir")
        .arg(&evidence_dir)
        .current_dir(root.path())
        .assert()
        .code(3)
        .stdout(predicate::str::contains("crunch-bootstrap-validation-v1"))
        .stdout(predicate::str::contains("preflight-failed"));

    let summary_path = evidence_dir.join("validation-summary.json");
    let doctor_path = evidence_dir.join("doctor.json");
    let summary: Value = serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    assert_eq!(summary["schema"], "crunch-bootstrap-validation-v1");
    assert_eq!(summary["status"], "preflight-failed");
    assert_eq!(summary["doctor_ok"], false);
    assert_eq!(summary["build_attempted"], false);
    assert!(doctor_path.exists());
    assert!(!evidence_dir.join("build.stdout.log").exists());
    assert!(evidence_dir.join("validation-summary.md").exists());
}

#[test]
fn bootstrap_validate_success_writes_logs_and_summary() {
    if !can_build() {
        eprintln!("skipping bootstrap validate success test: bwrap or /nix/store not available");
        return;
    }

    let root = TempDir::new().unwrap();
    let (state_dir, store_dir, evidence_dir, target) = write_success_fixture(root.path());

    crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("bootstrap")
        .arg("validate")
        .arg(&target)
        .arg("--evidence-dir")
        .arg(&evidence_dir)
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("crunch-bootstrap-validation-v1"))
        .stdout(predicate::str::contains("passed"));

    let summary_path = evidence_dir.join("validation-summary.json");
    let summary: Value = serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
    assert_eq!(summary["schema"], "crunch-bootstrap-validation-v1");
    assert_eq!(summary["status"], "passed");
    assert_eq!(summary["doctor_ok"], true);
    assert_eq!(summary["build_attempted"], true);
    assert_eq!(summary["build_exit_code"], 0);
    assert_eq!(summary["failure_class"], Value::Null);

    let doctor_path = evidence_dir.join("doctor.json");
    let stdout_path = evidence_dir.join("build.stdout.log");
    let stderr_path = evidence_dir.join("build.stderr.log");
    let markdown_path = evidence_dir.join("validation-summary.md");
    assert!(doctor_path.exists());
    assert!(stdout_path.exists());
    assert!(stderr_path.exists());
    assert!(markdown_path.exists());
    assert_eq!(summary["evidence"]["doctor_json"], doctor_path.display().to_string());
    assert_eq!(summary["evidence"]["build_stdout"], stdout_path.display().to_string());
    assert_eq!(summary["evidence"]["build_stderr"], stderr_path.display().to_string());
    assert_eq!(summary["evidence"]["summary_md"], markdown_path.display().to_string());

    let markdown = fs::read_to_string(markdown_path).unwrap();
    assert!(markdown.contains("- Status: `Passed`"));
    assert!(markdown.contains("- Build attempted: `true`"));
}
