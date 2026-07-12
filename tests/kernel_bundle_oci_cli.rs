use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

#[test]
fn oci_projection_commands_are_public_and_document_the_local_boundary() {
    mantle_cmd()
        .args(["artifact", "oci-export", "--help"])
        .assert()
        .success()
        .stdout(contains("atomic local OCI image layout"))
        .stdout(contains("--spec-material"))
        .stdout(contains("--source-admissions"))
        .stdout(contains("--projection"));

    mantle_cmd()
        .args(["artifact", "oci-import", "--help"])
        .assert()
        .success()
        .stdout(contains("local OCI image layout"))
        .stdout(contains("--report-out"));
}

#[test]
fn oci_export_rejects_unsealed_unknown_projection_before_object_reads() {
    let temporary = TempDir::new().expect("temporary CLI fixture should exist");
    let state = temporary.path().join("state");
    let projection = temporary.path().join("projection.json");
    let spec = temporary.path().join("spec.md");
    let source_admissions = temporary.path().join("source-admissions.json");
    let output = temporary.path().join("layout");
    std::fs::write(&projection, b"{\"schema\":\"unknown\"}\n").expect("projection fixture should write");
    std::fs::write(&spec, b"bounded spec material\n").expect("spec fixture should write");
    std::fs::write(&source_admissions, b"{}\n").expect("source admission fixture should write");

    mantle_cmd()
        .arg("--state-dir")
        .arg(&state)
        .args(["artifact", "oci-export", "--projection"])
        .arg(&projection)
        .arg("--spec-material")
        .arg(&spec)
        .arg("--source-admissions")
        .arg(&source_admissions)
        .arg("--out")
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            contains("parsing OCI projection")
                .or(contains("exporting OCI layout"))
                .and(contains("missing field")),
        );

    assert!(!output.exists(), "failed preflight must not publish a layout");
    assert!(!state.exists(), "failed preflight must not read or mutate CAS state");
}
