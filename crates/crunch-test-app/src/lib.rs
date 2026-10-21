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
pub fn execute(port: &mut impl TestPort, request: SelectedRequest) -> Result<Report> {
    let SelectedRequest {
        target,
        invocation,
        selected,
        identity,
    } = request;
    let prepared = port.prepare(&target, &invocation);
    let mut collection_failure = prepared.err().map(|reason| CollectionFailure {
        outcome: port.interruption().unwrap_or(Outcome::InfrastructureFailure),
        reason,
    });
    let mut attempts = Vec::new();
    for repeat in 0..invocation.repeat {
        if collection_failure.is_some() {
            break;
        }
        for name in &selected {
            let name = name.clone();
            let (name, outcome, reason) = match port.capture(&target, &invocation, &name, repeat) {
                Ok(capture) => {
                    let classified = match capture.interrupted {
                        Some(outcome) => crunch_test_core::ClassifiedCapture { name, outcome },
                        None => classify_capture(name, capture.stdout, capture.exit_code),
                    };
                    let outcome = classified.outcome;
                    (
                        classified.name,
                        outcome,
                        match outcome {
                            Outcome::Pass => "complete",
                            Outcome::AssertionFailure => "assertion",
                            Outcome::Timeout => "deadline",
                            Outcome::Cancelled => "cancelled",
                            Outcome::InfrastructureFailure => "incomplete-crashed-or-invalid-harness",
                        }
                        .to_string(),
                    )
                }
                Err(reason) => (name, port.interruption().unwrap_or(Outcome::InfrastructureFailure), reason),
            };
            attempts.push(Attempt {
                repeat,
                test: name,
                outcome,
                reason,
            });
            if outcome == Outcome::Cancelled {
                break;
            }
        }
        if attempts.last().is_some_and(|a| a.outcome == Outcome::Cancelled) {
            break;
        }
    }
    let coverage = if collection_failure.is_none()
        && invocation.coverage
        && !attempts
            .iter()
            .any(|a| matches!(a.outcome, Outcome::InfrastructureFailure | Outcome::Timeout | Outcome::Cancelled))
    {
        match port.coverage(&target, &invocation) {
            Ok(coverage) => Some(coverage),
            Err(reason) => {
                collection_failure = Some(CollectionFailure {
                    outcome: port.interruption().unwrap_or(Outcome::InfrastructureFailure),
                    reason,
                });
                None
            }
        }
    } else {
        None
    };
    let aggregated = aggregate(attempts);
    let outcome = collection_failure.as_ref().map_or(aggregated.outcome, |failure| failure.outcome);
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
