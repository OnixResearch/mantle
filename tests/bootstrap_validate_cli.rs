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

#[test]
fn bootstrap_validate_build_child_uses_selected_casita_backend() {
    if !can_build() {
        eprintln!("skipping selected-backend bootstrap child test: bwrap or /nix/store not available");
        return;
    }

    let root = TempDir::new().unwrap();
    let (state_dir, store_dir, evidence_dir, target) = write_success_fixture(root.path());
    crunch()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .args(["store", "list"])
        .assert()
        .success();
    let identity_path = state_dir.join("store-identity.json");
    let identity_before = fs::read(&identity_path).unwrap();

    let output = crunch()
        .args(["--json", "--store-backend", "casita", "--state-dir"])
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .args(["bootstrap", "validate"])
        .arg(&target)
        .arg("--evidence-dir")
        .arg(&evidence_dir)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let summary: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(summary["status"], "passed");
    assert_eq!(summary["doctor_ok"], true);
    assert_eq!(summary["build_attempted"], true);
    assert_eq!(summary["build_exit_code"], 0);
    assert_eq!(fs::read(&identity_path).unwrap(), identity_before);
    assert!(state_dir.join("casita").is_dir());
    assert!(!state_dir.join("pathinfo.redb").exists());
}

#[test]
fn native_closure_existing_output_is_not_overwritten_by_failed_input_read() {
    let root = TempDir::new().unwrap();
    let output = root.path().join("native-closure.json");
    fs::write(&output, b"operator-owned").unwrap();

    crunch()
        .args(["bootstrap", "native-toolchain-closure"])
        .arg("--rust-source-provider")
        .arg(root.path().join("missing-rust"))
        .arg("--host-root")
        .arg(root.path().join("missing-host"))
        .arg("--target-root")
        .arg(root.path().join("missing-target"))
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    assert_eq!(fs::read(&output).unwrap(), b"operator-owned");
}

#[test]
fn provider_admission_rejects_invalid_digest_before_creating_report() {
    let root = TempDir::new().unwrap();
    let report = root.path().join("admission.json");
    crunch()
        .args(["bootstrap", "full-source-provider-admit"])
        .arg("--provider-dir")
        .arg(root.path().join("missing-provider"))
        .arg("--expected-output-blake3")
        .arg("not-a-digest")
        .arg("--source-closure")
        .arg(root.path().join("missing-closure.json"))
        .arg("--expected-source-closure-blake3")
        .arg("0".repeat(64))
        .arg("--report")
        .arg(&report)
        .assert()
        .failure()
        .stderr(predicate::str::contains("lowercase hexadecimal characters"));

    assert!(!report.exists(), "invalid admission must not publish a report");
}
