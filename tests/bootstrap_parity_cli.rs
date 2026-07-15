use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn row_by_id<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["rows"].as_array().unwrap().iter().find(|row| row["id"] == id).unwrap()
}

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

#[test]
fn bootstrap_parity_report_rejects_legacy_self_build_proof_without_unblocking_axes() {
    let repo = env!("CARGO_MANIFEST_DIR");

    let output = crunch()
        .arg("--json")
        .arg("bootstrap")
        .arg("parity-report")
        .current_dir(repo)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    let row = row_by_id(&report, "crunch.self-build");
    assert_eq!(row["status"], "blocked");
    assert_eq!(row["provider_kind"], "unknown");
    assert!(row["proof_details"].is_null());
    assert!(row["notes"].as_str().unwrap().contains("mantle-deterministic-proof-receipt-v2"));

    let guix = report["axes"].as_array().unwrap().iter().find(|axis| axis["axis"] == "guix").unwrap();
    let stagex = report["axes"].as_array().unwrap().iter().find(|axis| axis["axis"] == "stagex").unwrap();
    assert_eq!(guix["complete"], false);
    assert_eq!(stagex["complete"], false);
    assert!(guix["blocking_rows"].as_array().unwrap().contains(&Value::String("crunch.self-build".to_string())));
    assert!(
        stagex["blocking_rows"]
            .as_array()
            .unwrap()
            .contains(&Value::String("crunch.self-build".to_string()))
    );
}

#[test]
fn bootstrap_parity_report_exposes_full_musl_binutils_contract_without_unblocking_axes() {
    let repo = env!("CARGO_MANIFEST_DIR");

    let output = crunch()
        .arg("--json")
        .arg("bootstrap")
        .arg("parity-report")
        .current_dir(repo)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    let row = row_by_id(&report, "full-musl-binutils");
    assert_eq!(row["status"], "partial");
    assert_eq!(row["provider_kind"], "unknown");
    assert!(row["notes"].as_str().unwrap().contains("checked receipt"));
    assert!(!row["notes"].as_str().unwrap().contains("evidence check failed"));

    let live = report["axes"].as_array().unwrap().iter().find(|axis| axis["axis"] == "live-bootstrap").unwrap();
    let guix = report["axes"].as_array().unwrap().iter().find(|axis| axis["axis"] == "guix").unwrap();
    assert_eq!(live["complete"], false);
    assert_eq!(guix["complete"], false);
    assert!(live["blocking_rows"].as_array().unwrap().contains(&Value::String("full-musl-binutils".to_string())));
    assert!(guix["blocking_rows"].as_array().unwrap().contains(&Value::String("full-musl-binutils".to_string())));
}
