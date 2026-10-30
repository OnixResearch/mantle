//! Consumer-visible refusal tests for `release gauntlet`.
//!
//! Each gauntlet action plans one release effect, reads and validates its inputs, and classifies
//! the observed output before it reports. A request whose input is missing, malformed or the wrong
//! shape must exit with the typed error class, print no report on stdout and publish no output
//! file. The success paths are exercised by the in-crate gauntlet tests; these tests pin the
//! failure boundary. r[application_architecture.effect_observation_boundary]
//! r[application_architecture.typed_error_ownership]

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

const EXIT_REFUSED: i32 = 3;
const EXIT_USAGE: i32 = 2;

fn gauntlet(work: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("mantle")
        .unwrap()
        .arg("--state-dir")
        .arg(work.join("state"))
        .args(["release", "gauntlet"])
        .args(args)
        .current_dir(work)
        .output()
        .unwrap()
}

fn assert_refused(output: &std::process::Output, code: i32) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stdout.is_empty(), "a refused gauntlet action must not print a report: {output:?}");
}

fn entries(work: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(work)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn canonicalize_refuses_malformed_and_wrong_shaped_reports_without_publishing() {
    let work = TempDir::new().unwrap();
    fs::write(work.path().join("malformed.json"), b"{not json").unwrap();
    fs::write(work.path().join("empty.json"), b"{}").unwrap();
    let before = entries(work.path());
    for input in ["malformed.json", "empty.json", "missing.json"] {
        for kind in ["continuous", "bootstrap-pressure", "release-repeatability"] {
            assert_refused(&gauntlet(work.path(), &["canonicalize", "--kind", kind, input]), EXIT_REFUSED);
            let with_output = gauntlet(work.path(), &["canonicalize", "--kind", kind, input, "--output", "out.json"]);
            assert_refused(&with_output, EXIT_REFUSED);
        }
    }
    assert_eq!(entries(work.path()), before, "a refused canonicalization must not publish an output file");
}

#[test]
fn canonicalize_rejects_an_unknown_report_kind_as_a_usage_error() {
    let work = TempDir::new().unwrap();
    fs::write(work.path().join("report.json"), b"{}").unwrap();
    assert_refused(&gauntlet(work.path(), &["canonicalize", "--kind", "not-a-kind", "report.json"]), EXIT_USAGE);
    assert_refused(&gauntlet(work.path(), &["canonicalize", "report.json"]), EXIT_USAGE);
}

#[test]
fn aggregate_actions_refuse_missing_evidence_without_writing_their_report() {
    let work = TempDir::new().unwrap();
    let before = entries(work.path());
    let continuous = gauntlet(work.path(), &[
        "continuous",
        "--context",
        "absent-context.json",
        "--track",
        "absent-track.json",
        "--report-path",
        "report.json",
    ]);
    assert_refused(&continuous, EXIT_REFUSED);
    let strict = gauntlet(work.path(), &[
        "strict-hermeticity-regression",
        "--run-id",
        "run-1",
        "--plan",
        "absent-plan.json",
        "--evidence",
        "absent-evidence.json",
        "--report-path",
        "report.json",
    ]);
    assert_refused(&strict, EXIT_REFUSED);
    assert_eq!(entries(work.path()), before, "no report may be written when its evidence cannot be read");
}

#[test]
fn aggregate_actions_require_at_least_one_evidence_input() {
    let work = TempDir::new().unwrap();
    fs::write(work.path().join("context.json"), b"{}").unwrap();
    assert_refused(&gauntlet(work.path(), &["continuous", "--context", "context.json"]), EXIT_USAGE);
    assert_refused(
        &gauntlet(work.path(), &[
            "strict-hermeticity-regression",
            "--run-id",
            "run-1",
            "--plan",
            "context.json",
        ]),
        EXIT_USAGE,
    );
}
