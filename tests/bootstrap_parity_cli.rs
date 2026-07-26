use std::fs;
use std::path::Path;

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

const BINUTILS_ROW_FIXTURE_PATHS: &[&str] = &[
    "bootstrap/binutils-tcc.ncl",
    "bootstrap/bison-2.3-musl.ncl",
    "bootstrap/diffutils-2.7-musl.ncl",
    "bootstrap/flex-2.6.4-musl.ncl",
    "bootstrap/gawk-3.0.4-musl.ncl",
    "bootstrap/m4-1.4.7-musl.ncl",
    "bootstrap/musl-1.1.24-native.ncl",
    "bootstrap/tcc-musl-native.ncl",
    "bootstrap/tcc-musl-v2.ncl",
    "bootstrap/evidence/early-native-binutils-row-v2.json",
    "bootstrap/evidence/early-native-binutils-artifact.json",
    "bootstrap/evidence/early-native-binutils-acceptance-v1.json",
    "bootstrap/evidence/early-native-predecessors/bison-2.3.json",
    "bootstrap/evidence/early-native-predecessors/flex-2.6.4.json",
    "bootstrap/evidence/early-native-predecessors/gawk-3.0.4.json",
    "bootstrap/evidence/early-native-predecessors/m4-1.4.7.json",
    "bootstrap/evidence/early-native-predecessors/native-musl.json",
    "bootstrap/evidence/early-native-predecessors/tcc-v2.json",
];
const BINUTILS_SOURCE_PATH: &str = "bootstrap/binutils-tcc.ncl";
const BINUTILS_RECEIPT_PATH: &str = "bootstrap/evidence/early-native-binutils-row-v2.json";
const BINUTILS_ARTIFACT_PATH: &str = "bootstrap/evidence/early-native-binutils-artifact.json";
const BINUTILS_ACCEPTANCE_PATH: &str = "bootstrap/evidence/early-native-binutils-acceptance-v1.json";
const BISON_PREDECESSOR_PATH: &str = "bootstrap/evidence/early-native-predecessors/bison-2.3.json";

fn copy_binutils_row_fixture() -> TempDir {
    let fixture = TempDir::new().unwrap();
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in BINUTILS_ROW_FIXTURE_PATHS {
        let source = repository.join(relative);
        let destination = fixture.path().join(relative);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(&source, &destination).unwrap();
        assert!(destination.is_file());
        assert!(fs::metadata(&destination).unwrap().len() > 0);
    }
    fixture
}

fn parity_report(root: &Path) -> Value {
    let output = crunch()
        .arg("--json")
        .arg("bootstrap")
        .arg("parity-report")
        .current_dir(root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["schema"], "crunch-bootstrap-parity-gap-report-v1");
    report
}

fn mutate_json(path: &Path, mutation: impl FnOnce(&mut Value)) {
    let bytes = fs::read(path).unwrap();
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    mutation(&mut value);
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    assert!(path.is_file());
    assert!(fs::metadata(path).unwrap().len() > 0);
}

fn refresh_binutils_source_digest(root: &Path) {
    let source = fs::read(root.join(BINUTILS_SOURCE_PATH)).unwrap();
    let digest = blake3::hash(&source).to_hex().to_string();
    mutate_json(&root.join(BINUTILS_RECEIPT_PATH), |receipt| {
        let records = receipt["source_records"].as_array_mut().unwrap();
        let record = records.iter_mut().find(|record| record["path"] == BINUTILS_SOURCE_PATH).unwrap();
        record["blake3"] = Value::String(digest);
    });
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
fn bootstrap_parity_report_accepts_independently_receipted_early_native_rows() {
    let report = parity_report(Path::new(env!("CARGO_MANIFEST_DIR")));
    let binutils = row_by_id(&report, "binutils.tcc");
    let gcc40 = row_by_id(&report, "gcc.4.0");

    assert_eq!(binutils["status"], "complete");
    assert_eq!(gcc40["status"], "complete");
    assert!(!binutils["notes"].as_str().unwrap().contains("evidence check failed"));
    assert!(!gcc40["notes"].as_str().unwrap().contains("evidence check failed"));
}

#[test]
fn bootstrap_parity_report_rejects_malformed_row_receipt() {
    let fixture = copy_binutils_row_fixture();
    fs::write(fixture.path().join(BINUTILS_RECEIPT_PATH), "{").unwrap();

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("parse early-native row receipt"));
}

#[test]
fn bootstrap_parity_report_rejects_stale_row_source_digest() {
    let fixture = copy_binutils_row_fixture();
    let source_path = fixture.path().join(BINUTILS_SOURCE_PATH);
    let mut source = fs::read_to_string(&source_path).unwrap();
    source.push_str("\n# stale fixture mutation\n");
    fs::write(&source_path, source).unwrap();

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("source record BLAKE3"));
}

#[test]
fn bootstrap_parity_report_rejects_cross_row_receipt_substitution() {
    let fixture = copy_binutils_row_fixture();
    mutate_json(&fixture.path().join(BINUTILS_RECEIPT_PATH), |receipt| {
        receipt["row_id"] = Value::String("gcc.4.0".to_string());
    });

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("row_id"));
}

#[test]
fn bootstrap_parity_report_rejects_corrupt_predecessor_attestation() {
    let fixture = copy_binutils_row_fixture();
    mutate_json(&fixture.path().join(BISON_PREDECESSOR_PATH), |evidence| {
        evidence["digest"] =
            Value::String("0000000000000000000000000000000000000000000000000000000000000000".to_string());
    });

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("artifact evidence digest mismatch"));
}

#[test]
fn bootstrap_parity_report_rejects_missing_output_artifact_evidence() {
    let fixture = copy_binutils_row_fixture();
    fs::remove_file(fixture.path().join(BINUTILS_ARTIFACT_PATH)).unwrap();

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("read early-native artifact evidence"));
}

#[test]
fn bootstrap_parity_report_rejects_forbidden_host_tool_discovery() {
    let fixture = copy_binutils_row_fixture();
    let source_path = fixture.path().join(BINUTILS_SOURCE_PATH);
    let mut source = fs::read_to_string(&source_path).unwrap();
    source.push_str("\n# forbidden command -v host fallback\n");
    fs::write(&source_path, source).unwrap();
    refresh_binutils_source_digest(fixture.path());

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("forbidden policy marker `command -v `"));
}

#[test]
fn bootstrap_parity_report_rejects_wrapper_delegation_marker() {
    let fixture = copy_binutils_row_fixture();
    let source_path = fixture.path().join(BINUTILS_SOURCE_PATH);
    let mut source = fs::read_to_string(&source_path).unwrap();
    source.push_str("\n# forbidden exec \"$TCC/bin/tcc\" delegation\n");
    fs::write(&source_path, source).unwrap();
    refresh_binutils_source_digest(fixture.path());

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("forbidden policy marker"));
}

#[test]
fn bootstrap_parity_report_rejects_untrusted_acceptance_fingerprint() {
    let fixture = copy_binutils_row_fixture();
    mutate_json(&fixture.path().join(BINUTILS_ACCEPTANCE_PATH), |acceptance| {
        acceptance["trust"]["trust_unsigned"] = Value::Bool(true);
    });

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("runtime trust fingerprint"));
}

#[test]
fn bootstrap_parity_report_rejects_stale_generated_artifact_status() {
    let fixture = copy_binutils_row_fixture();
    mutate_json(&fixture.path().join(BINUTILS_RECEIPT_PATH), |receipt| {
        receipt["generated_artifacts"][0]["status"] = Value::String("release-generated".to_string());
    });

    let report = parity_report(fixture.path());
    let row = row_by_id(&report, "binutils.tcc");
    assert_eq!(row["status"], "partial");
    assert!(row["notes"].as_str().unwrap().contains("generated_artifacts.status"));
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
fn bootstrap_parity_report_requires_binutils_row_receipt_for_live_bootstrap_and_guix() {
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
            stdout.contains("assembler/linker/archive/ranlib/nm/objcopy/object-format/relocation"),
            "stdout missing semantic evidence: {stdout}"
        );
        assert!(
            stdout.contains("evidence check failed: read early-native row receipt"),
            "stdout missing receipt read failure: {stdout}"
        );
        assert!(
            stdout.contains("bootstrap/evidence/early-native-binutils-row-v2.json"),
            "stdout missing receipt path: {stdout}"
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
