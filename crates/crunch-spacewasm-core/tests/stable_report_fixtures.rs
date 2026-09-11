//! Fixtures for stable SpaceWasm report admission.
//!
//! Positive fixtures prove that presentation differences (test completion
//! order, durations, progress lines) never change the stable identity while
//! any changed fact does. Negative fixtures cover changed outcomes, changed
//! test sets, duplicates, malformed and truncated captures, unknown
//! grammar, contradictory summaries, and missing required facts.

use crunch_spacewasm_core::HarnessLine;
use crunch_spacewasm_core::StableReportRequest;
use crunch_spacewasm_core::StableTestStatus;
use crunch_spacewasm_core::admit_stable_report;
use crunch_spacewasm_core::parse_libtest_events;

const SUITE: &str = "upstream-unit-tests";
const COMMAND: &str = "cargo test --locked --offline --no-default-features --lib -- --format json -Z unstable-options";
const EXPECTED: [&str; 3] = [
    "spacewasm::tests::alpha",
    "spacewasm::tests::bravo",
    "spacewasm::tests::charlie",
];

/// Harness capture where `alpha` completes last (the rebuild presentation).
fn capture_original() -> String {
    String::from(
        r#"{"type":"suite","event":"started","test_count":3}
{"type":"test","name":"spacewasm::tests::alpha","event":"ok"}
{"type":"test","name":"spacewasm::tests::bravo","event":"ok"}
{"type":"test","name":"spacewasm::tests::charlie","event":"ok"}
{"type":"suite","event":"ok"}
"#,
    )
}

/// Same suite, different completion order; same admitted facts.
fn capture_reordered() -> String {
    String::from(
        r#"{"type":"suite","event":"started","test_count":3}
{"type":"test","name":"spacewasm::tests::charlie","event":"ok"}
{"type":"test","name":"spacewasm::tests::alpha","event":"ok"}
{"type":"test","name":"spacewasm::tests::bravo","event":"ok"}
{"type":"suite","event":"ok"}
"#,
    )
}

fn request_for(capture: &str) -> StableReportRequest {
    let lines = parse_libtest_events(capture).expect("fixture capture parses");
    StableReportRequest {
        suite: String::from(SUITE),
        command: String::from(COMMAND),
        expected_tests: EXPECTED.iter().map(|name| String::from(*name)).collect(),
        lines,
    }
}

#[test]
fn equivalent_presentations_admit_to_one_identity() {
    let first = admit_stable_report(&request_for(&capture_original()));
    let second = admit_stable_report(&request_for(&capture_reordered()));
    let first = first.report.expect("original presentation admits");
    let second = second.report.expect("reordered presentation admits");
    assert_eq!(first.stable_identity_blake3, second.stable_identity_blake3);
    assert_eq!(first.tests.len(), EXPECTED.len());
    assert_eq!(first.tests, second.tests);
}

#[test]
fn changed_outcome_changes_the_identity() {
    let mut drifted = capture_original();
    drifted = drifted.replace(
        r#"{"type":"test","name":"spacewasm::tests::bravo","event":"ok"}"#,
        r#"{"type":"test","name":"spacewasm::tests::bravo","event":"failed"}"#,
    );
    drifted = drifted.replace("{\"type\":\"suite\",\"event\":\"ok\"}", "{\"type\":\"suite\",\"event\":\"failed\"}");
    let result = admit_stable_report(&request_for(&drifted));
    let report = result.report.expect("a failed member still admits");
    assert_ne!(
        report.stable_identity_blake3,
        admit_stable_report(&request_for(&capture_original()))
            .report
            .expect("original admits")
            .stable_identity_blake3
    );
    assert!(
        report
            .tests
            .iter()
            .any(|record| record.name == "spacewasm::tests::bravo" && record.status == StableTestStatus::Failed)
    );
}

#[test]
fn changed_test_inventory_changes_the_identity() {
    let mut capture = capture_original();
    capture = capture.replace(
        r#"{"type":"test","name":"spacewasm::tests::charlie","event":"ok"}"#,
        r#"{"type":"test","name":"spacewasm::tests::charlie-delta","event":"ok"}"#,
    );
    let result = admit_stable_report(&request_for(&capture));
    let diagnostics = result.diagnostics;
    assert!(result.report.is_none());
    assert!(diagnostics.iter().any(|d| d.code == "stable-report-missing-expected-test"));
}

#[test]
fn duplicate_test_records_reject() {
    let mut capture = capture_original();
    capture.push_str(r#"{"type":"test","name":"spacewasm::tests::alpha","event":"ok"}"#);
    capture.push('\n');
    let result = admit_stable_report(&request_for(&capture));
    assert!(result.report.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "stable-report-duplicate-test"));
}

#[test]
fn malformed_and_unknown_grammar_lines_reject() {
    for capture in [
        "{\"type\":\"test\",\"name\":\"spacewasm::tests::alpha\"\n",
        "{\"type\":\"benchmark\",\"name\":\"x\"}\n",
        "{\"type\":\"test\",\"name\":\"spacewasm::tests::alpha\",\"event\":\"timeout\"}\n",
        "{\"type\":\"test\",\"name\":\"spacewasm::tests::alpha\",\"event\":\"ok\",\"extra\":1}\n",
        "running 3 tests\n",
        "{\"type\":\"suite\",\"event\":\"started\",\"test_count\":3}\n",
    ] {
        let outcome = parse_libtest_events(capture);
        if let Ok(lines) = outcome {
            let request = StableReportRequest {
                suite: String::from(SUITE),
                command: String::from(COMMAND),
                expected_tests: Vec::new(),
                lines,
            };
            let result = admit_stable_report(&request);
            assert!(result.report.is_none(), "capture must not admit a report: {capture}");
        }
    }
}

#[test]
fn truncated_capture_rejects() {
    let truncated = "{\"type\":\"suite\",\"event\":\"started\",\"test_count\":3}\n{\"type\":\"test\",\"nam";
    assert!(parse_libtest_events(truncated).is_err());
}

#[test]
fn contradictory_suite_summary_rejects() {
    let capture = String::from(
        r#"{"type":"suite","event":"started","test_count":3}
{"type":"test","name":"spacewasm::tests::alpha","event":"failed"}
{"type":"test","name":"spacewasm::tests::bravo","event":"ok"}
{"type":"test","name":"spacewasm::tests::charlie","event":"ok"}
{"type":"suite","event":"ok"}
"#,
    );
    let result = admit_stable_report(&request_for(&capture));
    assert!(result.report.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "stable-report-contradictory-summary"));
}

#[test]
fn failed_and_skipped_facts_stay_visible() {
    let capture = String::from(
        r#"{"type":"suite","event":"started","test_count":3}
{"type":"test","name":"spacewasm::tests::alpha","event":"ok"}
{"type":"test","name":"spacewasm::tests::bravo","event":"failed"}
{"type":"test","name":"spacewasm::tests::charlie","event":"ignored"}
{"type":"suite","event":"failed"}
"#,
    );
    let result = admit_stable_report(&request_for(&capture));
    let report = result.report.expect("failed suite admits with visible facts");
    assert!(
        report
            .tests
            .iter()
            .any(|record| record.name == "spacewasm::tests::charlie" && record.status == StableTestStatus::Skipped)
    );
    assert!(
        report
            .tests
            .iter()
            .any(|record| record.name == "spacewasm::tests::bravo" && record.status == StableTestStatus::Failed)
    );
}

#[test]
fn empty_and_headerless_captures_reject() {
    assert!(parse_libtest_events("").is_err());
    assert!(parse_libtest_events("\n\n").is_err());
}

#[test]
fn empty_suite_or_command_reject() {
    let mut request = request_for(&capture_original());
    request.suite = String::new();
    let result = admit_stable_report(&request);
    assert!(result.report.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "stable-report-empty-suite"));

    let mut request = request_for(&capture_original());
    request.command = String::new();
    let result = admit_stable_report(&request);
    assert!(result.report.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "stable-report-empty-command"));
}

#[test]
fn harness_line_enum_round_trips_through_the_grammar() {
    let lines = parse_libtest_events(&capture_original()).expect("parses");
    assert_eq!(lines.len(), 5);
    assert!(lines.iter().any(|line| matches!(line, HarnessLine::Suite { event } if event == "started")));
    assert!(lines.iter().any(|line| matches!(line, HarnessLine::Suite { event } if event == "ok")));
    assert_eq!(lines.iter().filter(|line| matches!(line, HarnessLine::Test { .. })).count(), 3);
}
