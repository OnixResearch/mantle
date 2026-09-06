use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

const DOC_PATHS: &[&str] = &[
    "README.md",
    "docs/operator-workflows.md",
    "docs/operator-proof-guide.md",
    "examples/README.md",
];
const REQUIRED_RUNBOOK_NEEDLES: &[&str] = &[
    "mantle source bundle export",
    "mantle source bundle import --from",
    "--pin",
    "mantle source bundle verify",
    "mantle source bundle preflight",
    "mantle build --offline-source-preflight --no-substitute",
    "network_policy_reports[]",
    "cargo_build_evidence[]",
    "cargo_build_evidence_diagnostics[]",
    "next_actions[]",
    "source_state_blake3",
    "ready_class",
    "source bundle evidence proves declared source/input availability and identity only",
    "final outputs need separate build/cache and attestation evidence",
    "Do not report source readiness, source import, route eligibility, source-bundle input realization, or offline Cargo evidence as build success, output trust, Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.",
];
const FORBIDDEN_OVERCLAIMS: &[&str] = &[
    "source-bundle readiness proves build success",
    "source readiness proves build success",
    "route eligibility proves output trust",
    "offline Cargo evidence proves Cargo-free execution",
    "offline Cargo evidence proves full Cargo compatibility",
    "source import proves release reproducibility",
];

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn read_docs() -> String {
    DOC_PATHS
        .iter()
        .map(|path| std::fs::read_to_string(repo_root().join(path)).expect("read runbook doc"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn validate_offline_build_runbook_docs(docs: &str) -> Vec<String> {
    let docs = docs.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut errors = Vec::new();
    for needle in REQUIRED_RUNBOOK_NEEDLES {
        if !docs.contains(needle) {
            errors.push(format!("offline build runbook docs missing `{needle}`"));
        }
    }
    for forbidden in FORBIDDEN_OVERCLAIMS {
        if docs.contains(forbidden) {
            errors.push(format!("offline build runbook docs overclaim with `{forbidden}`"));
        }
    }
    errors
}

#[test]
fn offline_build_runbook_docs_cover_commands_evidence_and_non_claims() {
    let docs = read_docs();
    let errors = validate_offline_build_runbook_docs(&docs);

    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn offline_build_runbook_validator_rejects_missing_commands_and_overclaims() {
    let bad_docs = "source-bundle readiness proves build success\n";
    let errors = validate_offline_build_runbook_docs(bad_docs);

    assert!(errors.iter().any(|error| error.contains("mantle source bundle export")));
    assert!(errors.iter().any(|error| error.contains("source-bundle readiness proves build success")));
}

#[test]
fn runbook_line_wrapping_cannot_hide_missing_non_claims() {
    // r[verify native_package_parity.inventories]
    let valid = REQUIRED_RUNBOOK_NEEDLES.join("\n");
    assert!(validate_offline_build_runbook_docs(&valid.replace(' ', "\n")).is_empty());
    let missing = valid.replace("final outputs need separate build/cache and attestation evidence", "");
    assert!(!validate_offline_build_runbook_docs(&missing).is_empty());
    for forbidden in FORBIDDEN_OVERCLAIMS {
        let poisoned = format!("{valid}\n{}", forbidden.replace(' ', "\n"));
        assert!(validate_offline_build_runbook_docs(&poisoned).iter().any(|error| error.contains("overclaim")));
    }
}

#[test]
fn cli_help_exposes_source_bundle_and_offline_source_preflight() {
    mantle_cmd().arg("--help").assert().success().stdout(predicate::str::contains("source"));
    mantle_cmd()
        .args(["source", "bundle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("preflight"));
    mantle_cmd()
        .args(["build", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--offline-source-preflight"));
}
