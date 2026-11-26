#![no_std]
//! The admitted application owns the effect sequence, never a second scheduler.
extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crunch_test_core::Attempt;
use crunch_test_core::CollectionFailure;
use crunch_test_core::CoverageReport;
use crunch_test_core::Invocation;
use crunch_test_core::Outcome;
use crunch_test_core::REPORT_SCHEMA;
use crunch_test_core::Report;
use crunch_test_core::SelectedRequest;
use crunch_test_core::Target;
use crunch_test_core::aggregate;
use crunch_test_core::classify_capture;
type Result<T, E = String> = core::result::Result<T, E>;

pub struct Capture {
    pub stdout: String,
    pub exit_code: Option<i32>,
    pub interrupted: Option<Outcome>,
}
pub trait TestPort {
    /// Verify exact executable/tool members and prepare isolated runfiles.
    fn prepare(&mut self, target: &Target, invocation: &Invocation) -> Result<()>;
    /// Execute one exact libtest name with a real deadline and owned teardown.
    fn capture(&mut self, target: &Target, invocation: &Invocation, name: &str, repeat: u32) -> Result<Capture>;
    fn coverage(&mut self, target: &Target, invocation: &Invocation) -> Result<CoverageReport>;
    fn interruption(&self) -> Option<Outcome>;
}
/// The diagnostic recorded for one classified outcome.
fn outcome_reason(outcome: Outcome) -> String {
    let reason = match outcome {
        Outcome::Pass => "complete",
        Outcome::AssertionFailure => "assertion",
        Outcome::Timeout => "deadline",
        Outcome::Cancelled => "cancelled",
        Outcome::InfrastructureFailure => "incomplete-crashed-or-invalid-harness",
    };
    debug_assert!(!reason.is_empty());
    reason.to_string()
}
/// The collection failure recorded when the run could not be prepared.
fn prepare_collection(port: &mut impl TestPort, target: &Target, invocation: &Invocation) -> Option<CollectionFailure> {
    let prepared = port.prepare(target, invocation);
    let is_prepare_failed = prepared.is_err();
    let failure = prepared.err().map(|reason| CollectionFailure {
        outcome: port.interruption().unwrap_or(Outcome::InfrastructureFailure),
        reason,
    });
    debug_assert!(!is_prepare_failed || failure.is_some());
    failure
}
/// Capture one exact libtest name and classify its transcript into an attempt.
fn attempt_from_capture(
    port: &mut impl TestPort,
    target: &Target,
    invocation: &Invocation,
    name: &str,
    repeat: u32,
) -> Attempt {
    let name = name.to_string();
    match port.capture(target, invocation, &name, repeat) {
        Ok(capture) => {
            let interrupted = capture.interrupted;
            let classified = match interrupted {
                Some(outcome) => crunch_test_core::ClassifiedCapture { name, outcome },
                None => classify_capture(name, capture.stdout, capture.exit_code),
            };
            let outcome = classified.outcome;
            let reason = outcome_reason(outcome);
            debug_assert!(!reason.is_empty());
            debug_assert!(interrupted.is_none_or(|value| value == outcome));
            Attempt {
                repeat,
                test: classified.name,
                outcome,
                reason,
            }
        }
        Err(reason) => Attempt {
            repeat,
            test: name,
            outcome: port.interruption().unwrap_or(Outcome::InfrastructureFailure),
            reason,
        },
    }
}
/// Run every selected test once per repeat, stopping when the run is cancelled.
fn collect_attempts(
    port: &mut impl TestPort,
    target: &Target,
    invocation: &Invocation,
    selected: &[String],
) -> Vec<Attempt> {
    let mut attempts = Vec::with_capacity(selected.len());
    for repeat in 0..invocation.repeat {
        // One attempt per selected test per repeat; keep the reservation exact.
        attempts.reserve(selected.len());
        let attempts_before = attempts.len();
        for name in selected {
            let attempt = attempt_from_capture(port, target, invocation, name, repeat);
            let is_cancelled = attempt.outcome == Outcome::Cancelled;
            attempts.push(attempt);
            if is_cancelled {
                break;
            }
        }
        debug_assert!(attempts.len().saturating_sub(attempts_before) <= selected.len());
        if attempts.last().is_some_and(|a| a.outcome == Outcome::Cancelled) {
            break;
        }
    }
    let has_cancelled = attempts.iter().any(|a| a.outcome == Outcome::Cancelled);
    debug_assert!(!has_cancelled || attempts.last().is_some_and(|a| a.outcome == Outcome::Cancelled));
    attempts
}
/// Collect coverage only for a clean run that asked for coverage.
fn collect_coverage(
    port: &mut impl TestPort,
    target: &Target,
    invocation: &Invocation,
    attempts: &[Attempt],
    collection_failure: &mut Option<CollectionFailure>,
) -> Option<CoverageReport> {
    let is_collection_failed = collection_failure.is_some();
    let is_coverage_requested = invocation.coverage;
    let has_blocking_outcome = attempts
        .iter()
        .any(|a| matches!(a.outcome, Outcome::InfrastructureFailure | Outcome::Timeout | Outcome::Cancelled));
    if is_collection_failed || !is_coverage_requested || has_blocking_outcome {
        return None;
    }
    debug_assert!(!is_collection_failed);
    debug_assert!(!has_blocking_outcome);
    match port.coverage(target, invocation) {
        Ok(coverage) => Some(coverage),
        Err(reason) => {
            *collection_failure = Some(CollectionFailure {
                outcome: port.interruption().unwrap_or(Outcome::InfrastructureFailure),
                reason,
            });
            debug_assert!(collection_failure.is_some());
            None
        }
    }
}
pub fn execute(port: &mut impl TestPort, request: SelectedRequest) -> Result<Report> {
    let SelectedRequest {
        target,
        invocation,
        selected,
        identity,
    } = request;
    let mut collection_failure = prepare_collection(port, &target, &invocation);
    let attempts = if collection_failure.is_none() {
        collect_attempts(port, &target, &invocation, &selected)
    } else {
        Vec::new()
    };
    let coverage = collect_coverage(port, &target, &invocation, &attempts, &mut collection_failure);
    let attempt_total = attempts.len();
    let aggregated = aggregate(attempts);
    let outcome = collection_failure.as_ref().map_or(aggregated.outcome, |failure| failure.outcome);
    debug_assert_eq!(attempt_total, aggregated.attempts.len());
    debug_assert!(coverage.is_none() || collection_failure.is_none());
    Ok(Report {
        schema: REPORT_SCHEMA.into(),
        identity,
        target: target.name,
        selected,
        outcome,
        attempts: aggregated.attempts,
        coverage,
        collection_failure,
    })
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::collections::VecDeque;
    use alloc::format;
    use alloc::vec;

    use crunch_test_core::COHORT;
    use crunch_test_core::Coverage;
    use crunch_test_core::Resources;
    use crunch_test_core::TARGET_SCHEMA;
    use crunch_test_core::Tool;
    use crunch_test_core::select;

    use super::*;

    struct MockPort {
        prepare_result: Result<()>,
        captures: VecDeque<Result<Capture>>,
        coverage_result: Option<Result<CoverageReport>>,
        interruption_outcome: Option<Outcome>,
        capture_calls: Vec<(String, u32)>,
        coverage_calls: u32,
    }

    impl MockPort {
        fn new() -> Self {
            Self {
                prepare_result: Ok(()),
                captures: VecDeque::new(),
                coverage_result: None,
                interruption_outcome: None,
                capture_calls: Vec::new(),
                coverage_calls: 0,
            }
        }
    }

    impl TestPort for MockPort {
        fn prepare(&mut self, _target: &Target, _invocation: &Invocation) -> Result<()> {
            self.prepare_result.clone()
        }

        fn capture(&mut self, _target: &Target, _invocation: &Invocation, name: &str, repeat: u32) -> Result<Capture> {
            self.capture_calls.push((name.to_string(), repeat));
            self.captures.pop_front().unwrap_or_else(|| Err("unexpected-capture-call".into()))
        }

        fn coverage(&mut self, _target: &Target, _invocation: &Invocation) -> Result<CoverageReport> {
            self.coverage_calls += 1;
            self.coverage_result.take().unwrap_or_else(|| Err("unexpected-coverage-call".into()))
        }

        fn interruption(&self) -> Option<Outcome> {
            self.interruption_outcome
        }
    }

    fn hex(byte: char) -> String {
        core::iter::repeat_n(byte, 64).collect()
    }

    fn tool(root: &str, member: &str, byte: char) -> Tool {
        Tool {
            root: root.into(),
            member: member.into(),
            blake3: hex(byte),
        }
    }

    fn sample_target(with_coverage: bool) -> Target {
        let bin_root = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bin";
        let src_root = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-src";
        let mut inputs = BTreeMap::from([(bin_root.into(), hex('1'))]);
        let coverage = if with_coverage {
            inputs.insert(src_root.into(), hex('2'));
            Some(Coverage {
                profdata: tool(bin_root, "bin/llvm-profdata", '3'),
                cov: tool(bin_root, "bin/llvm-cov", '4'),
                source_root: src_root.into(),
                source_blake3: hex('2'),
                max_profile_bytes: 1_048_576,
            })
        } else {
            None
        };
        Target {
            schema: TARGET_SCHEMA.into(),
            name: "sample".into(),
            cohort: COHORT.into(),
            rustc_verbose_version: "rustc 1.90.0 (1159e78c4 2025-09-14)\nLLVM version: 20.1.8\n".into(),
            store_prefix: "/mantle/store".into(),
            platform: "x86_64-linux".into(),
            executable: tool(bin_root, "bin/test", '5'),
            runner: tool(bin_root, "bin/runner", '6'),
            inputs,
            runfiles: BTreeMap::new(),
            environment: BTreeMap::new(),
            tests: vec!["tests::alpha".into(), "tests::beta".into()],
            resources: Resources {
                memory_bytes: 67_108_864,
                output_bytes: 65_536,
                timeout_ms: 5_000,
            },
            deterministic: true,
            external_services: false,
            coverage,
        }
    }

    fn libtest_json(name: &str, event: &str, passed: u64, failed: u64) -> String {
        format!(
            concat!(
                "{{\"type\":\"suite\",\"event\":\"started\",\"test_count\":1}}\n",
                "{{\"type\":\"test\",\"event\":\"started\",\"name\":\"{name}\"}}\n",
                "{{\"type\":\"test\",\"name\":\"{name}\",\"event\":\"{event}\"}}\n",
                "{{\"type\":\"suite\",\"event\":\"{suite}\",\"passed\":{passed},\"failed\":{failed},\"ignored\":0,\"measured\":0}}\n"
            ),
            name = name,
            event = event,
            suite = if failed == 0 { "ok" } else { "failed" },
            passed = passed,
            failed = failed,
        )
    }

    fn sample_coverage_report() -> CoverageReport {
        CoverageReport {
            cohort: COHORT.into(),
            source_root: "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-src".into(),
            source_blake3: hex('2'),
            profdata_blake3: hex('3'),
            cov_blake3: hex('4'),
            export_blake3: hex('7'),
            covered_lines: 8,
            total_lines: 10,
        }
    }

    #[test]
    fn execute_records_prepare_failure_without_running_captures_or_coverage() {
        let invocation = Invocation {
            coverage: true,
            ..Invocation::default()
        };
        let request = select(sample_target(true), invocation).unwrap();
        let mut port = MockPort::new();
        port.prepare_result = Err("test-tool-digest-mismatch".into());
        port.interruption_outcome = Some(Outcome::Timeout);

        let report = execute(&mut port, request).unwrap();
        assert_eq!(report.outcome, Outcome::Timeout);
        assert!(report.attempts.is_empty());
        assert!(report.coverage.is_none());
        assert_eq!(
            report.collection_failure,
            Some(CollectionFailure {
                outcome: Outcome::Timeout,
                reason: "test-tool-digest-mismatch".into(),
            })
        );
        assert!(port.capture_calls.is_empty());
        assert_eq!(port.coverage_calls, 0);
    }

    #[test]
    fn execute_stops_immediately_on_cancelled_attempt_and_skips_coverage() {
        let invocation = Invocation {
            repeat: 2,
            coverage: true,
            ..Invocation::default()
        };
        let request = select(sample_target(true), invocation).unwrap();
        let mut port = MockPort::new();
        port.captures.push_back(Ok(Capture {
            stdout: libtest_json("tests::alpha", "ok", 1, 0),
            exit_code: Some(0),
            interrupted: None,
        }));
        port.captures.push_back(Ok(Capture {
            stdout: String::new(),
            exit_code: None,
            interrupted: Some(Outcome::Cancelled),
        }));

        let report = execute(&mut port, request).unwrap();
        assert_eq!(report.outcome, Outcome::Cancelled);
        assert!(report.collection_failure.is_none());
        assert!(report.coverage.is_none());
        assert_eq!(port.coverage_calls, 0);
        assert_eq!(port.capture_calls, vec![("tests::alpha".into(), 0), ("tests::beta".into(), 0)]);
        assert_eq!(report.attempts.len(), 2);
        assert_eq!(report.attempts[0].outcome, Outcome::Pass);
        assert_eq!(report.attempts[1].outcome, Outcome::Cancelled);
        assert_eq!(report.attempts[1].reason, "cancelled");
    }

    #[test]
    fn execute_skips_coverage_on_timeout_and_infrastructure_failure() {
        for (interrupted, expected_outcome) in [
            (Some(Outcome::Timeout), Outcome::Timeout),
            (Some(Outcome::InfrastructureFailure), Outcome::InfrastructureFailure),
        ] {
            let invocation = Invocation {
                coverage: true,
                ..Invocation::default()
            };
            let request = select(sample_target(true), invocation).unwrap();
            let mut port = MockPort::new();
            port.captures.push_back(Ok(Capture {
                stdout: libtest_json("tests::alpha", "ok", 1, 0),
                exit_code: Some(0),
                interrupted: None,
            }));
            port.captures.push_back(Ok(Capture {
                stdout: String::new(),
                exit_code: None,
                interrupted,
            }));

            let report = execute(&mut port, request).unwrap();
            assert_eq!(report.outcome, expected_outcome);
            assert!(report.coverage.is_none());
            assert_eq!(port.coverage_calls, 0);
        }
    }

    #[test]
    fn execute_runs_coverage_on_assertion_failure() {
        let invocation = Invocation {
            filter: Some("tests::alpha".into()),
            exact: true,
            coverage: true,
            ..Invocation::default()
        };
        let request = select(sample_target(true), invocation).unwrap();
        let mut port = MockPort::new();
        port.captures.push_back(Ok(Capture {
            stdout: libtest_json("tests::alpha", "failed", 0, 1),
            exit_code: Some(101),
            interrupted: None,
        }));
        let expected_cov = sample_coverage_report();
        port.coverage_result = Some(Ok(expected_cov.clone()));

        let report = execute(&mut port, request).unwrap();
        assert_eq!(report.outcome, Outcome::AssertionFailure);
        assert!(report.collection_failure.is_none());
        assert_eq!(report.coverage, Some(expected_cov));
        assert_eq!(port.coverage_calls, 1);
    }

    #[test]
    fn execute_preserves_attempts_when_coverage_fails() {
        let invocation = Invocation {
            coverage: true,
            ..Invocation::default()
        };
        let request = select(sample_target(true), invocation).unwrap();
        let mut port = MockPort::new();
        port.captures.push_back(Ok(Capture {
            stdout: libtest_json("tests::alpha", "ok", 1, 0),
            exit_code: Some(0),
            interrupted: None,
        }));
        port.captures.push_back(Ok(Capture {
            stdout: libtest_json("tests::beta", "ok", 1, 0),
            exit_code: Some(0),
            interrupted: None,
        }));
        port.coverage_result = Some(Err("test-coverage-merge-failed".into()));

        let report = execute(&mut port, request).unwrap();
        assert_eq!(report.outcome, Outcome::InfrastructureFailure);
        assert_eq!(report.attempts.len(), 2);
        assert!(report.coverage.is_none());
        assert_eq!(
            report.collection_failure,
            Some(CollectionFailure {
                outcome: Outcome::InfrastructureFailure,
                reason: "test-coverage-merge-failed".into(),
            })
        );
        assert_eq!(port.coverage_calls, 1);
    }

    #[test]
    fn execute_classifies_capture_error_with_interruption_or_infrastructure_failure() {
        for (interruption, expected_outcome) in [
            (Some(Outcome::Cancelled), Outcome::Cancelled),
            (None, Outcome::InfrastructureFailure),
        ] {
            let invocation = Invocation {
                filter: Some("tests::alpha".into()),
                exact: true,
                ..Invocation::default()
            };
            let request = select(sample_target(false), invocation).unwrap();
            let mut port = MockPort::new();
            port.interruption_outcome = interruption;
            port.captures.push_back(Err("pipe-read-failed".into()));

            let report = execute(&mut port, request).unwrap();
            assert_eq!(report.outcome, expected_outcome);
            assert_eq!(report.attempts.len(), 1);
            assert_eq!(report.attempts[0].outcome, expected_outcome);
            assert_eq!(report.attempts[0].reason, "pipe-read-failed");
        }
    }
}
