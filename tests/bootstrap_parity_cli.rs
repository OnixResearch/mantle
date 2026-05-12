use std::fs;

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

#[test]
fn bootstrap_parity_report_stagex_requires_lineage_seed_provider() {
    let root = TempDir::new().unwrap();

    let output = crunch()
        .arg("bootstrap")
        .arg("parity-report")
        .arg("--require")
        .arg("stagex")
        .current_dir(root.path())
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("stagex: incomplete"));
    assert!(stdout.contains("seed-full.stagex-lineage"));
}

#[test]
fn bootstrap_parity_report_requires_binutils_tcc_transcript_for_live_bootstrap_and_guix() {
    let root = TempDir::new().unwrap();
    let bootstrap = root.path().join("bootstrap");
    fs::create_dir_all(&bootstrap).unwrap();
    fs::write(bootstrap.join("binutils-tcc.ncl"), "# concrete binutils tcc derivation body\n").unwrap();

    for axis in ["live-bootstrap", "guix"] {
        let output = crunch()
            .arg("bootstrap")
            .arg("parity-report")
            .arg("--require")
            .arg(axis)
            .current_dir(root.path())
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8(output).unwrap();
        assert!(stdout.contains(&format!("{axis}: incomplete")), "stdout missing axis {axis}: {stdout}");
        assert!(stdout.contains("binutils.tcc [partial]"), "stdout missing binutils row: {stdout}");
        assert!(
            stdout.contains("checked binutils-tcc tool transcript required"),
            "stdout missing semantic evidence: {stdout}"
        );
        assert!(
            stdout.contains("evidence check failed: binutils-tcc tool transcript missing"),
            "stdout missing missing-transcript note: {stdout}"
        );
        assert!(
            stdout.contains("bootstrap/evidence/binutils-tcc-tool-smoke.json"),
            "stdout missing transcript path: {stdout}"
        );
    }
}
