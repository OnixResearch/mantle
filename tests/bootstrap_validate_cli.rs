use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
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

#[test]
fn bootstrap_validate_preflight_failure_writes_evidence_bundle() {
    let root = TempDir::new().unwrap();
    let state_dir = root.path().join("state");
    let missing_store = root.path().join("missing-store");
    let evidence_dir = root.path().join("evidence");
    let target = root.path().join("make-tcc.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    write_minimal_derivation(&target);

    crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&missing_store)
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
