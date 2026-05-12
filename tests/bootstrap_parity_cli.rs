use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

#[test]
fn bootstrap_parity_report_emits_json_gap_report() {
    let root = TempDir::new().unwrap();

    let output = crunch()
        .arg("--json")
        .arg("bootstrap")
        .arg("parity-report")
        .current_dir(root.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["schema"], "crunch-bootstrap-parity-gap-report-v1");
    assert!(report["rows"].as_array().unwrap().len() >= 20);
    assert_eq!(report["axes"][0]["axis"], "live-bootstrap");
    assert_eq!(report["axes"][0]["complete"], false);
}

#[test]
fn bootstrap_parity_report_require_fails_closed_on_gaps() {
    let root = TempDir::new().unwrap();

    crunch()
        .arg("bootstrap")
        .arg("parity-report")
        .arg("--require")
        .arg("live-bootstrap")
        .current_dir(root.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("Bootstrap parity gap report"))
        .stdout(predicate::str::contains("live-bootstrap: incomplete"));
}
