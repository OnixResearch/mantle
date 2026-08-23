use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

const PLAN_FILE: &str = "source-built-fixed-point-plan.json";
const STAGE_EVIDENCE_FILE: &str = "source-built-stage-evidence.json";
const RECEIPT_FILE: &str = "deterministic-build-proof.json";
const EXTENSION_FIELD: &str = "source_built_fixed_point";
const EXECUTED_STAGE_COUNT: u64 = 2;
const RESTORED_STAGE_COUNT: u64 = 4;
const HEX_DIGITS_PER_BYTE: usize = 2;
const BUNDLE_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-proof-bundle-v2\0";
const V48_EVIDENCE: &str =
    "cairn/changes/prove-source-built-mantle-fixed-point/evidence/v48-promoted-fixed-point-success-2026-08-23";

fn mantle() -> Command {
    Command::cargo_bin("mantle").unwrap()
}

#[test]
fn bootstrap_trust_report_renders_verified_fixed_point_gap() {
    let fixture = v48_fixture();

    let output = mantle()
        .arg("--json")
        .arg("bootstrap")
        .arg("trust-report")
        .arg("--proof-root")
        .arg(fixture.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["schema"], "mantle-bootstrap-trust-report-v1");
    assert_eq!(report["status"], "fixed-point-only");
    assert_eq!(report["fixed_point_verified"], true);
    assert_eq!(report["root_action_trust_complete"], false);
    assert_eq!(report["stages"]["executed"], EXECUTED_STAGE_COUNT);
    assert_eq!(report["stages"]["restored_checkpoint"], RESTORED_STAGE_COUNT);
    assert!(report["blockers"].as_array().unwrap().iter().any(|value| value == "action-trust-plan-not-bound"));
}

#[test]
fn bootstrap_trust_report_rejects_tampered_linked_evidence() {
    let fixture = v48_fixture();
    fs::write(fixture.path().join(STAGE_EVIDENCE_FILE), "[]\n").unwrap();

    mantle()
        .arg("--json")
        .arg("bootstrap")
        .arg("trust-report")
        .arg("--proof-root")
        .arg(fixture.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("source-built stage evidence digest mismatch"));

    assert!(fixture.path().join(RECEIPT_FILE).is_file());
}

#[test]
fn bootstrap_trust_report_requires_a_proof_root() {
    mantle()
        .arg("bootstrap")
        .arg("trust-report")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--proof-root <PATH>"));
}

fn v48_fixture() -> TempDir {
    let fixture = TempDir::new().unwrap();
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let evidence = repository.join(V48_EVIDENCE);
    let plan = fs::read(evidence.join(PLAN_FILE)).unwrap();
    let stages = fs::read(evidence.join(STAGE_EVIDENCE_FILE)).unwrap();
    fs::write(fixture.path().join(PLAN_FILE), &plan).unwrap();
    fs::write(fixture.path().join(STAGE_EVIDENCE_FILE), &stages).unwrap();
    let bundle_digest = fixture_bundle_digest(&[(PLAN_FILE, plan), (STAGE_EVIDENCE_FILE, stages)]);
    let mut receipt: Value = serde_json::from_slice(&fs::read(evidence.join(RECEIPT_FILE)).unwrap()).unwrap();
    receipt[EXTENSION_FIELD]["final_proof_bundle_digest_blake3"] = Value::String(bundle_digest);
    fs::write(fixture.path().join(RECEIPT_FILE), serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    assert!(fixture.path().join(PLAN_FILE).is_file());
    assert!(fixture.path().join(STAGE_EVIDENCE_FILE).is_file());
    fixture
}

fn fixture_bundle_digest(entries: &[(&str, Vec<u8>)]) -> String {
    let mut entries = entries.to_vec();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    let mut hasher = blake3::Hasher::new();
    hasher.update(BUNDLE_DIGEST_DOMAIN);
    for (relative, bytes) in entries {
        hasher.update(relative.as_bytes());
        hasher.update(&[0]);
        hasher.update(b"file\0");
        hasher.update(blake3::hash(&bytes).to_hex().as_bytes());
        hasher.update(&[0]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN * HEX_DIGITS_PER_BYTE);
    assert!(!digest.is_empty());
    digest
}
