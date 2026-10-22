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
