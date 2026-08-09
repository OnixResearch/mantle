use std::fs;
use std::path::Path;
use std::process::Command;

use assert_cmd::cargo::cargo_bin;
use serde_json::Value;

const FIXTURE_ROOT: &str = "fixtures/evaluation-budget";
const STRICT_POLICY: &str = "config/evaluation/default-policy.json";
const POSITIVE_SOURCE: &str = "fixtures/evaluation-budget/positive-evaluation.ncl";

fn mantle() -> Command {
    Command::new(cargo_bin("mantle"))
}

fn report(path: &Path) -> Value {
    let bytes = fs::read(path).expect("report must exist");
    serde_json::from_slice(&bytes).expect("report must be valid JSON")
}

#[test]
fn strict_worker_matches_existing_eval_and_reports_supported_resources() {
    let temporary = tempfile::tempdir().unwrap();
    let report_path = temporary.path().join("strict-report.json");
    let legacy = mantle().args(["eval", POSITIVE_SOURCE]).output().unwrap();
    let strict = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            STRICT_POLICY,
            "--budget-report",
            report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(legacy.status.success(), "legacy stderr: {}", String::from_utf8_lossy(&legacy.stderr));
    assert!(strict.status.success(), "strict stderr: {}", String::from_utf8_lossy(&strict.stderr));
    let legacy_json: Value = serde_json::from_slice(&legacy.stdout).unwrap();
    let strict_json: Value = serde_json::from_slice(&strict.stdout).unwrap();
    assert_eq!(strict_json, legacy_json);

    let report = report(&report_path);
    assert_eq!(report["terminal_disposition"], "success");
    assert_eq!(report["teardown"]["reaped"], true);
    assert_eq!(report["teardown"]["response_present"], true);
    let metrics = report["metrics"].as_array().unwrap();
    assert!(metrics.iter().any(|metric| metric["name"] == "peak_rss" && metric["status"] == "observed"));
    assert!(metrics.iter().any(|metric| metric["name"] == "cpu_time" && metric["status"] == "observed"));
    assert!(metrics.iter().any(|metric| {
        metric["name"] == "explicit_top_level_root_force_count"
            && metric["role"] == "public-api-request-not-nickel-thunk-count"
    }));
}

#[test]
fn observe_only_keeps_in_process_path_and_marks_operation_scoped_metrics_unavailable() {
    let temporary = tempfile::tempdir().unwrap();
    let report_path = temporary.path().join("observe-report.json");
    let output = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-observe-only.json"),
            "--budget-report",
            report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let report = report(&report_path);
    assert_eq!(report["mode"], "observe-only");
    assert_eq!(report["terminal_disposition"], "success");
    let metrics = report["metrics"].as_array().unwrap();
    assert!(metrics.iter().any(|metric| metric["name"] == "peak_rss" && metric["status"] == "unavailable"));
    assert!(metrics.iter().any(|metric| metric["name"] == "cpu_time" && metric["status"] == "unavailable"));
}

#[test]
fn worker_preserves_declared_imports_and_bounded_evaluator_errors() {
    let temporary = tempfile::tempdir().unwrap();
    let import_report_path = temporary.path().join("import-report.json");
    let imported = mantle()
        .args([
            "eval",
            &format!("{FIXTURE_ROOT}/positive-with-import.ncl"),
            "--budget-policy",
            STRICT_POLICY,
            "--budget-report",
            import_report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(imported.status.success(), "stderr: {}", String::from_utf8_lossy(&imported.stderr));
    assert_eq!(serde_json::from_slice::<Value>(&imported.stdout).unwrap()["imported"], "bounded-import");

    let error_report_path = temporary.path().join("error-report.json");
    let invalid = mantle()
        .args([
            "eval",
            &format!("{FIXTURE_ROOT}/negative-evaluator-error.ncl"),
            "--budget-policy",
            STRICT_POLICY,
            "--budget-report",
            error_report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    let error_report = report(&error_report_path);
    assert_eq!(error_report["terminal_disposition"], "evaluation-error");
    assert_eq!(error_report["error_class"], "nickel-evaluation-error");
    assert!(error_report["evaluator"]["diagnostic_count"].as_u64().unwrap() > 0);

    let observe_report_path = temporary.path().join("observe-error-report.json");
    let observe_invalid = mantle()
        .args([
            "eval",
            &format!("{FIXTURE_ROOT}/negative-evaluator-error.ncl"),
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-observe-only.json"),
            "--budget-report",
            observe_report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!observe_invalid.status.success());
    let observe_report = report(&observe_report_path);
    assert_eq!(observe_report["terminal_disposition"], error_report["terminal_disposition"]);
    assert_eq!(observe_report["error_class"], error_report["error_class"]);
    assert_eq!(observe_report["evaluator"]["diagnostic_count"], error_report["evaluator"]["diagnostic_count"]);
}

#[test]
fn strict_worker_confines_imports_to_admitted_roots() {
    let temporary = tempfile::tempdir().unwrap();
    let source_dir = temporary.path().join("source");
    let outside_dir = temporary.path().join("outside");
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();
    let outside = outside_dir.join("outside.ncl");
    fs::write(&outside, "{ escaped = true }\n").unwrap();
    let source = source_dir.join("source.ncl");
    fs::write(&source, format!("import {:?}\n", outside.display().to_string())).unwrap();

    let legacy = mantle().args(["eval", source.to_str().unwrap()]).output().unwrap();
    assert!(legacy.status.success(), "legacy stderr: {}", String::from_utf8_lossy(&legacy.stderr));

    let report_path = temporary.path().join("confined-report.json");
    let strict = mantle()
        .args([
            "eval",
            source.to_str().unwrap(),
            "--budget-policy",
            STRICT_POLICY,
            "--budget-report",
            report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!strict.status.success(), "strict evaluation read an undeclared import");
    let report = report(&report_path);
    assert_eq!(report["terminal_disposition"], "evaluation-error");
    assert!(
        report["metrics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|metric| { metric["name"] == "import_filesystem_confinement" && metric["status"] == "enforced" })
    );
}

#[test]
fn deadline_is_terminal_and_worker_is_reaped() {
    let temporary = tempfile::tempdir().unwrap();
    let report_path = temporary.path().join("timeout-report.json");
    let output = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-timeout.json"),
            "--budget-report",
            report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report = report(&report_path);
    assert_eq!(report["terminal_disposition"], "timeout");
    assert_eq!(report["teardown"]["deadline_exceeded"], true);
    assert_eq!(report["teardown"]["reaped"], true);
    assert_eq!(report["teardown"]["response_present"], false);
}

#[test]
fn invalid_policy_fixtures_fail_before_report_publication() {
    let cases = [
        ("policy-zero.json", "evaluation-budget-invalid-limit:wall-time-ms"),
        ("policy-overflow.json", "evaluation-budget-limit-too-large:source-bytes"),
        (
            "policy-contradiction.json",
            "evaluation-budget-contradictory-limit:selected-roots-vs-discovered-roots",
        ),
        ("policy-unknown-field.json", "unknown field"),
        ("policy-unsupported-mode.json", "unknown variant"),
        ("policy-exceeded-source.json", "evaluation-budget-limit-exceeded:source-bytes"),
        ("policy-exceeded-import-entries.json", "evaluation-budget-limit-exceeded:import-entries"),
    ];
    for (fixture, expected) in cases {
        let temporary = tempfile::tempdir().unwrap();
        let report_path = temporary.path().join("must-not-exist.json");
        let output = mantle()
            .args([
                "eval",
                POSITIVE_SOURCE,
                "--budget-policy",
                &format!("{FIXTURE_ROOT}/{fixture}"),
                "--budget-report",
                report_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "fixture {fixture} unexpectedly succeeded");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(expected), "fixture {fixture} stderr was: {stderr}");
        assert!(!report_path.exists(), "fixture {fixture} published a report");
    }
}

#[test]
fn selected_and_all_root_requests_report_honest_force_counts() {
    let temporary = tempfile::tempdir().unwrap();
    let selected_report_path = temporary.path().join("selected-report.json");
    let selected = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            STRICT_POLICY,
            "--budget-report",
            selected_report_path.to_str().unwrap(),
            "--root",
            "answer",
        ])
        .output()
        .unwrap();
    assert!(selected.status.success(), "stderr: {}", String::from_utf8_lossy(&selected.stderr));
    let selected_value: Value = serde_json::from_slice(&selected.stdout).unwrap();
    assert_eq!(selected_value["answer"].as_f64(), Some(42.0));
    assert!(selected_value.get("message").is_none());
    let selected_observe_report = temporary.path().join("selected-observe-report.json");
    let selected_observe = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-observe-only.json"),
            "--budget-report",
            selected_observe_report.to_str().unwrap(),
            "--root",
            "answer",
        ])
        .output()
        .unwrap();
    assert!(selected_observe.status.success());
    assert_eq!(serde_json::from_slice::<Value>(&selected_observe.stdout).unwrap(), selected_value);
    let selected_report = report(&selected_report_path);
    assert_eq!(selected_report["evaluator"]["discovered_root_count"], 2);
    assert_eq!(selected_report["evaluator"]["selected_root_count"], 1);
    assert_eq!(selected_report["evaluator"]["explicit_top_level_root_force_count"], 1);
    assert!(selected_report["evaluator"]["actual_nonselected_evaluation_count"].is_null());

    let all_report_path = temporary.path().join("all-report.json");
    let all = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            STRICT_POLICY,
            "--budget-report",
            all_report_path.to_str().unwrap(),
            "--all-roots",
        ])
        .output()
        .unwrap();
    assert!(all.status.success(), "stderr: {}", String::from_utf8_lossy(&all.stderr));
    let all_value: Value = serde_json::from_slice(&all.stdout).unwrap();
    assert_eq!(all_value["answer"].as_f64(), Some(42.0));
    assert_eq!(all_value["message"], "bounded");
    let all_observe_report = temporary.path().join("all-observe-report.json");
    let all_observe = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-observe-only.json"),
            "--budget-report",
            all_observe_report.to_str().unwrap(),
            "--all-roots",
        ])
        .output()
        .unwrap();
    assert!(all_observe.status.success());
    assert_eq!(serde_json::from_slice::<Value>(&all_observe.stdout).unwrap(), all_value);
    let all_report = report(&all_report_path);
    assert_eq!(all_report["evaluator"]["discovered_root_count"], 2);
    assert_eq!(all_report["evaluator"]["selected_root_count"], 2);
    assert_eq!(all_report["evaluator"]["explicit_top_level_root_force_count"], 2);
}

#[test]
fn discovered_root_limit_fails_with_its_stable_class() {
    let temporary = tempfile::tempdir().unwrap();
    let report_path = temporary.path().join("root-limit-report.json");
    let output = mantle()
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-discovered-root-limit.json"),
            "--budget-report",
            report_path.to_str().unwrap(),
            "--all-roots",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report = report(&report_path);
    assert_eq!(report["terminal_disposition"], "evaluation-error");
    assert_eq!(report["error_class"], "evaluation-budget-limit-exceeded:discovered-roots");
}

#[test]
fn process_fixtures_classify_crash_signal_protocol_overflow_and_memory() {
    let cases = [
        ("panic", "worker-crash"),
        ("signal", "worker-signal"),
        ("malformed-frame", "protocol-error"),
        ("response-flood", "response-overflow"),
        ("memory-limit", "memory-limit"),
        ("memory-exhaustion", "memory-limit"),
    ];
    for (behavior, disposition) in cases {
        let temporary = tempfile::tempdir().unwrap();
        let report_path = temporary.path().join(format!("{behavior}-report.json"));
        let output = mantle()
            .env("MANTLE_TEST_EVALUATOR_WORKER_FIXTURE", behavior)
            .args([
                "eval",
                POSITIVE_SOURCE,
                "--budget-policy",
                &format!("{FIXTURE_ROOT}/policy-process-fixture.json"),
                "--budget-report",
                report_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "fixture {behavior} unexpectedly succeeded");
        let report = report(&report_path);
        assert_eq!(report["terminal_disposition"], disposition, "fixture {behavior}");
        assert_eq!(report["teardown"]["reaped"], true, "fixture {behavior}");
    }
}

#[test]
fn cpu_exhaustion_reaches_the_enforced_cpu_terminal_class() {
    let temporary = tempfile::tempdir().unwrap();
    let report_path = temporary.path().join("cpu-report.json");
    let output = mantle()
        .env("MANTLE_TEST_EVALUATOR_WORKER_FIXTURE", "cpu-exhaustion")
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-cpu-limit.json"),
            "--budget-report",
            report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report = report(&report_path);
    assert_eq!(report["terminal_disposition"], "cpu-limit");
    assert_eq!(report["teardown"]["reaped"], true);
    assert_eq!(report["teardown"]["deadline_exceeded"], false);
}

#[test]
fn process_fixtures_bound_stderr_and_make_late_results_terminal() {
    let temporary = tempfile::tempdir().unwrap();
    let stderr_report_path = temporary.path().join("stderr-report.json");
    let stderr_output = mantle()
        .env("MANTLE_TEST_EVALUATOR_WORKER_FIXTURE", "stderr-flood")
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-process-fixture.json"),
            "--budget-report",
            stderr_report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(stderr_output.status.success(), "stderr: {}", String::from_utf8_lossy(&stderr_output.stderr));
    let stderr_report = report(&stderr_report_path);
    assert_eq!(stderr_report["terminal_disposition"], "success");
    assert_eq!(stderr_report["teardown"]["stderr_truncated"], true);
    assert_eq!(stderr_report["teardown"]["stderr_bytes"], 4096);
    assert_eq!(stderr_report["bounded_stderr"].as_str().unwrap().len(), 4096);

    let redaction_report_path = temporary.path().join("redaction-report.json");
    let redaction_output = mantle()
        .env("MANTLE_TEST_EVALUATOR_WORKER_FIXTURE", "stderr-path")
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-process-fixture.json"),
            "--budget-report",
            redaction_report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(redaction_output.status.success());
    let redaction_report = report(&redaction_report_path);
    assert_eq!(redaction_report["bounded_stderr"], "<redacted-import-root>");
    assert!(!redaction_report["bounded_stderr"].as_str().unwrap().contains(env!("CARGO_MANIFEST_DIR")));

    for (behavior, kill_expected) in [("late-success", false), ("ignore-terminate", true)] {
        let report_path = temporary.path().join(format!("{behavior}-report.json"));
        let output = mantle()
            .env("MANTLE_TEST_EVALUATOR_WORKER_FIXTURE", behavior)
            .args([
                "eval",
                POSITIVE_SOURCE,
                "--budget-policy",
                &format!("{FIXTURE_ROOT}/policy-process-fixture.json"),
                "--budget-report",
                report_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        let report = report(&report_path);
        assert_eq!(report["terminal_disposition"], "timeout");
        assert_eq!(report["teardown"]["response_present"], false);
        assert_eq!(report["teardown"]["reaped"], true);
        assert_eq!(report["teardown"]["kill_sent"], kill_expected);
    }
}

#[test]
fn simulated_reap_failure_blocks_a_clean_terminal_claim() {
    let temporary = tempfile::tempdir().unwrap();
    let report_path = temporary.path().join("reap-failure-report.json");
    let output = mantle()
        .env("MANTLE_TEST_EVALUATOR_FORCE_REAP_FAILURE", "1")
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            STRICT_POLICY,
            "--budget-report",
            report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report = report(&report_path);
    assert_eq!(report["terminal_disposition"], "incomplete-teardown");
    assert_eq!(report["teardown"]["reaped"], false);
    assert_eq!(report["teardown"]["response_present"], true);
}

#[test]
fn cancellation_is_terminal_and_reaps_a_late_worker() {
    let temporary = tempfile::tempdir().unwrap();
    let report_path = temporary.path().join("cancel-report.json");
    let output = mantle()
        .env("MANTLE_TEST_EVALUATOR_WORKER_FIXTURE", "late-success")
        .env("MANTLE_TEST_EVALUATOR_CANCEL_AFTER_MS", "10")
        .args([
            "eval",
            POSITIVE_SOURCE,
            "--budget-policy",
            &format!("{FIXTURE_ROOT}/policy-process-fixture.json"),
            "--budget-report",
            report_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let report = report(&report_path);
    assert_eq!(report["terminal_disposition"], "cancelled");
    assert_eq!(report["teardown"]["cancellation_requested"], true);
    assert_eq!(report["teardown"]["reaped"], true);
    assert_eq!(report["teardown"]["response_present"], false);
}

#[test]
fn hidden_worker_is_not_listed_in_public_help() {
    let output = mantle().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("evaluator-worker"));
    assert!(stdout.contains("eval"));
}
