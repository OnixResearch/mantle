//! Executable consumer parity against the actual pinned producer, never a
//! reimplementation of its transition or admission algorithm.

extern crate std;

use alloc::format;

use super::*;
use crate::attempt::RemoteAssignmentNonce;
use crate::attempt::RemoteAttemptApplyDisposition;
use crate::attempt::RemoteAttemptApplyPlan;
use crate::attempt::RemoteAttemptFailureClass;
use crate::attempt::RemoteAttemptId;
use crate::attempt::RemoteAttemptReasonCode;
use crate::attempt::RemoteEventId;
use crate::attempt::RemoteFenceGeneration;
use crate::attempt::RemoteJobId;
use crate::attempt::RemotePayloadDigest;
use crate::attempt::plan_remote_attempt_report;
// Exact upstream bytes are refreshed only by the repository-owned importer.
#[rustfmt::skip]
#[path = "../../../vendor/trellis-fenced-attempt/verified.rs"]
mod trellis;

const PHASES: [RemoteAttemptPhase; 7] = [
    RemoteAttemptPhase::Queued,
    RemoteAttemptPhase::Running,
    RemoteAttemptPhase::Transferring,
    RemoteAttemptPhase::FinishedUndelivered,
    RemoteAttemptPhase::Completed,
    RemoteAttemptPhase::Failed,
    RemoteAttemptPhase::Superseded,
];
const KINDS: [RemoteAttemptReportKind; 7] = [
    RemoteAttemptReportKind::Start,
    RemoteAttemptReportKind::Heartbeat,
    RemoteAttemptReportKind::LogAppend,
    RemoteAttemptReportKind::TransferCheckpoint,
    RemoteAttemptReportKind::ResultReady,
    RemoteAttemptReportKind::Failure,
    RemoteAttemptReportKind::Completion,
];
// Identical prefixes AND suffixes: the middle of the full digest matters.
const RESULT: &str = "aaaaaaaaaaaaaaaa11111111111111111111111111111111bbbbbbbbbbbbbbbb";
const OTHER_RESULT: &str = "aaaaaaaaaaaaaaaa22222222222222222222222222222222bbbbbbbbbbbbbbbb";
const RETAINED_EVENT: &str = "zz-last-retained-event";

fn state_with_history(count: usize) -> RemoteAttemptState {
    let mut applied_events = BTreeMap::new();
    for index in 1..count {
        applied_events.insert(
            RemoteEventId::new(format!("event-{index:04}")).unwrap(),
            RemotePayloadDigest::new(OTHER_RESULT).unwrap(),
        );
    }
    if count > 0 {
        applied_events
            .insert(RemoteEventId::new(RETAINED_EVENT).unwrap(), RemotePayloadDigest::new(OTHER_RESULT).unwrap());
    }
    RemoteAttemptState {
        job_id: RemoteJobId::new("job-original").unwrap(),
        attempt_id: RemoteAttemptId::new("attempt-original").unwrap(),
        assignment_nonce: RemoteAssignmentNonce::new(RESULT).unwrap(),
        fence_generation: RemoteFenceGeneration::new(2).unwrap(),
        phase: RemoteAttemptPhase::Queued,
        attempts_started: 1,
        started_unix_s: 100,
        deadline_unix_s: 200,
        applied_events,
        transfer_checkpoint: Some(1),
        transferred_bytes: 10,
        last_heartbeat_unix_s: Some(110),
        result_digest_blake3: None,
    }
}

fn payload(kind: RemoteAttemptReportKind) -> RemoteAttemptReportPayload {
    match kind {
        RemoteAttemptReportKind::Start => RemoteAttemptReportPayload::Start,
        RemoteAttemptReportKind::Heartbeat => RemoteAttemptReportPayload::Heartbeat { observed_unix_s: 120 },
        RemoteAttemptReportKind::LogAppend => RemoteAttemptReportPayload::LogAppend {
            cursor: 5,
            bytes: "log".into(),
        },
        RemoteAttemptReportKind::TransferCheckpoint => RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint: 2,
            transferred_bytes: 20,
        },
        RemoteAttemptReportKind::ResultReady => RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: RESULT.into(),
        },
        RemoteAttemptReportKind::Failure => RemoteAttemptReportPayload::Failure {
            failure_class: RemoteAttemptFailureClass::Retryable,
            reason_code: RemoteAttemptReasonCode::PersistenceFailed,
        },
        RemoteAttemptReportKind::Completion => RemoteAttemptReportPayload::Completion {
            output_digest_blake3: RESULT.into(),
        },
    }
}

fn report(state: &RemoteAttemptState, kind: RemoteAttemptReportKind) -> RemoteAttemptReport {
    RemoteAttemptReport::new(
        state.job_id.clone(),
        state.attempt_id.clone(),
        state.fence_generation,
        RemoteEventId::new("new-event").unwrap(),
        payload(kind),
    )
    .unwrap()
}

fn upstream_state(state: ProjectedState) -> trellis::AdmissionState {
    trellis::AdmissionState {
        job: state.job,
        attempt: state.attempt,
        fence: state.fence,
        phase: state.phase,
        event_count: state.event_count,
        result_present: state.result_present,
        result_id: state.result_id,
    }
}

fn upstream_report(report: ProjectedReport) -> trellis::AdmissionReport {
    trellis::AdmissionReport {
        job: report.job,
        attempt: report.attempt,
        fence: report.fence,
        event_id: report.event_id,
        digest_id: report.digest_id,
        event_relation: report.event_relation,
        kind: report.kind,
        worker_authorized: report.worker_authorized,
        output_authorized: report.output_authorized,
        result_id: report.result_id,
    }
}

fn disposition(disposition: RemoteAttemptApplyDisposition) -> u8 {
    match disposition {
        RemoteAttemptApplyDisposition::Applied => trellis::APPLIED,
        RemoteAttemptApplyDisposition::AlreadyApplied => trellis::REPLAYED,
        RemoteAttemptApplyDisposition::Rejected => trellis::REJECTED,
    }
}

// Deliberately exhaustive: a new consumer reason must not silently acquire
// proof cover. Out-of-model reasons are errors, not a catch-all model reason.
fn reason(reason: RemoteAttemptReasonCode) -> Option<u8> {
    use RemoteAttemptReasonCode::*;
    match reason {
        CurrentAttemptApplied | CurrentAttemptCompleted | AlreadyApplied => Some(0),
        EventDigestConflict => Some(trellis::EVENT_CONFLICT),
        EventRetentionExhausted => Some(trellis::HISTORY_FULL),
        StaleReportRejected => Some(trellis::STALE_FENCE),
        UnknownFenceRejected => Some(trellis::FUTURE_FENCE),
        JobIdentityMismatch => Some(trellis::WRONG_JOB),
        AttemptIdentityMismatch => Some(trellis::WRONG_ATTEMPT),
        WorkerUnauthorized => Some(trellis::WORKER_DENIED),
        OutputAdmissionUnauthorized => Some(trellis::OUTPUT_DENIED),
        TransitionRejected => Some(trellis::INVALID_TRANSITION),
        TerminalAttempt => Some(trellis::TERMINAL),
        ResultDigestMismatch => Some(trellis::RESULT_MISMATCH),
        PayloadDigestInvalid
        | PayloadDigestMismatch
        | PayloadSerializationFailed
        | PayloadTooLarge
        | IdentityInvalid
        | AssignmentNonceInvalid
        | FenceInvalid
        | FenceExhausted
        | RetryAllowed
        | RetryBackoffRequired
        | RetryBudgetExhausted
        | RetryDeadlineExceeded
        | RetryFailureTerminal
        | RetryPolicyInvalid
        | Superseded
        | LegacyStateRejected
        | DurableStateInvalid
        | PersistenceUnconfigured
        | PersistenceFailed
        | ResumeStateConflict => None,
    }
}

fn assert_parity(
    projection: &mut FiniteProjection,
    current: &RemoteAttemptState,
    report: &RemoteAttemptReport,
    authorization: RemoteAttemptAuthorizationFacts,
) -> RemoteAttemptApplyPlan {
    let input = projection.project(current, report, authorization).unwrap();
    let expected = trellis::admit(
        upstream_state(input.state),
        upstream_report(input.report),
        u32::try_from(MAX_REMOTE_ATTEMPT_EVENTS).unwrap(),
    );
    let actual = plan_remote_attempt_report(current, report, authorization);
    assert_eq!(disposition(actual.disposition), expected.disposition, "input={input:?}");
    assert_eq!(reason(actual.reason_code), Some(expected.reason), "input={input:?}");
    let next = projection.project_state(&actual.next_state).unwrap();
    assert!(upstream_state(next) == expected.next, "input={input:?}, next={next:?}");
    assert_eq!(actual.output_admission_allowed, expected.output_admission_allowed, "input={input:?}");
    if actual.disposition != RemoteAttemptApplyDisposition::Applied {
        // Stronger consumer obligation than Trellis's abstract preservation.
        assert_eq!(&actual.next_state, current);
        assert_eq!(actual.pending_effect, None);
    } else {
        assert_eq!(
            actual.next_state.applied_events.get(&report.identity.event_id),
            Some(&report.identity.payload_digest)
        );
    }
    actual
}

#[test]
fn complete_finite_matrix_matches_actual_trellis_admit() {
    let mut cases: u32 = 0;
    for history_count in [0, 1, MAX_REMOTE_ATTEMPT_EVENTS - 1, MAX_REMOTE_ATTEMPT_EVENTS] {
        let baseline = state_with_history(history_count);
        for phase in PHASES {
            for kind in KINDS {
                for identity_class in 0..5 {
                    for event_class in 0..3 {
                        if history_count == 0 && event_class != 0 {
                            // An empty retained history has no duplicate/conflict class.
                            continue;
                        }
                        for authorization_class in 0..4 {
                            for result_class in 0..3 {
                                let mut current = baseline.clone();
                                current.phase = phase;
                                current.result_digest_blake3 = match result_class {
                                    0 => None,
                                    1 => Some(RESULT.into()),
                                    _ => Some(OTHER_RESULT.into()),
                                };
                                let mut report = report(&current, kind);
                                match identity_class {
                                    0 => {}
                                    1 => report.identity.job_id = RemoteJobId::new("job-other").unwrap(),
                                    2 => report.identity.fence_generation = RemoteFenceGeneration::new(1).unwrap(),
                                    3 => report.identity.fence_generation = RemoteFenceGeneration::new(3).unwrap(),
                                    _ => report.identity.attempt_id = RemoteAttemptId::new("attempt-other").unwrap(),
                                }
                                if event_class != 0 {
                                    report.identity.event_id = RemoteEventId::new(RETAINED_EVENT).unwrap();
                                    let digest = if event_class == 1 {
                                        report.identity.payload_digest.clone()
                                    } else {
                                        // Canonical report payload remains intact; only retained history conflicts.
                                        RemotePayloadDigest::new(OTHER_RESULT).unwrap()
                                    };
                                    current.applied_events.insert(report.identity.event_id.clone(), digest);
                                }
                                let authorization = RemoteAttemptAuthorizationFacts {
                                    worker_authorized: authorization_class & 1 != 0,
                                    output_admission_authorized: authorization_class & 2 != 0,
                                };
                                assert_parity(&mut FiniteProjection::new(), &current, &report, authorization);
                                cases += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    std::println!("actual Trellis admit / Mantle plan parity: {cases} finite fixtures");
}

#[test]
fn authority_precedes_replay_and_replay_precedes_capacity_and_terminal() {
    for phase in PHASES {
        for kind in KINDS {
            let mut current = state_with_history(MAX_REMOTE_ATTEMPT_EVENTS);
            current.phase = phase;
            let mut report = report(&current, kind);
            report.identity.event_id = RemoteEventId::new(RETAINED_EVENT).unwrap();
            current
                .applied_events
                .insert(report.identity.event_id.clone(), report.identity.payload_digest.clone());
            for (worker, output, expected) in [
                (false, false, RemoteAttemptReasonCode::WorkerUnauthorized),
                (false, true, RemoteAttemptReasonCode::WorkerUnauthorized),
                (true, true, RemoteAttemptReasonCode::AlreadyApplied),
            ] {
                let actual =
                    assert_parity(&mut FiniteProjection::new(), &current, &report, RemoteAttemptAuthorizationFacts {
                        worker_authorized: worker,
                        output_admission_authorized: output,
                    });
                assert_eq!(actual.reason_code, expected);
            }
            let actual =
                assert_parity(&mut FiniteProjection::new(), &current, &report, RemoteAttemptAuthorizationFacts {
                    worker_authorized: true,
                    output_admission_authorized: false,
                });
            let expected = if matches!(kind, RemoteAttemptReportKind::ResultReady | RemoteAttemptReportKind::Completion)
            {
                RemoteAttemptReasonCode::OutputAdmissionUnauthorized
            } else {
                RemoteAttemptReasonCode::AlreadyApplied
            };
            assert_eq!(actual.reason_code, expected);
        }
    }
}

#[test]
fn full_identity_equality_is_stable_across_a_trace() {
    let mut projection = FiniteProjection::new();
    let mut current = state_with_history(0);
    let authorization = RemoteAttemptAuthorizationFacts {
        worker_authorized: true,
        output_admission_authorized: true,
    };
    for (index, kind) in [
        RemoteAttemptReportKind::Start,
        RemoteAttemptReportKind::ResultReady,
        RemoteAttemptReportKind::Completion,
    ]
    .into_iter()
    .enumerate()
    {
        let mut report = report(&current, kind);
        report.identity.event_id = RemoteEventId::new(format!("trace-{index}")).unwrap();
        let next = assert_parity(&mut projection, &current, &report, authorization);
        assert_eq!(next.disposition, RemoteAttemptApplyDisposition::Applied);
        current = next.next_state;
    }
    assert_eq!(current.phase, RemoteAttemptPhase::Completed);
    let before = projection.project_state(&current).unwrap();
    let mut other = current.clone();
    other.job_id = RemoteJobId::new("job-different").unwrap();
    other.attempt_id = RemoteAttemptId::new("attempt-different").unwrap();
    other.result_digest_blake3 = Some(OTHER_RESULT.into());
    let after = projection.project_state(&other).unwrap();
    assert_ne!(before.job, after.job);
    assert_ne!(before.attempt, after.attempt);
    assert_ne!(before.result_id, after.result_id);
    assert_eq!(projection.project_state(&current).unwrap(), before);
}

#[test]
fn complete_retained_history_classifies_the_last_entry() {
    let mut current = state_with_history(MAX_REMOTE_ATTEMPT_EVENTS);
    current.phase = RemoteAttemptPhase::Completed;
    let mut report = report(&current, RemoteAttemptReportKind::Start);
    report.identity.event_id = RemoteEventId::new(RETAINED_EVENT).unwrap();
    let authorization = RemoteAttemptAuthorizationFacts {
        worker_authorized: true,
        output_admission_authorized: true,
    };
    let mut projection = FiniteProjection::new();
    assert_eq!(
        projection.project(&current, &report, authorization).unwrap().report.event_relation,
        trellis::CONFLICTING_EVENT
    );
    current
        .applied_events
        .insert(report.identity.event_id.clone(), report.identity.payload_digest.clone());
    let input = projection.project(&current, &report, authorization).unwrap();
    assert_eq!(input.report.event_relation, trellis::SAME_EVENT);
    let actual = assert_parity(&mut projection, &current, &report, authorization);
    assert_eq!(actual.disposition, RemoteAttemptApplyDisposition::AlreadyApplied);
}

#[test]
fn out_of_model_progress_and_malformed_payloads_do_not_receive_coverage() {
    let mut current = state_with_history(0);
    current.phase = RemoteAttemptPhase::Running;
    let authorization = RemoteAttemptAuthorizationFacts {
        worker_authorized: true,
        output_admission_authorized: true,
    };
    for payload in [
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s: 99 },
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s: 201 },
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s: 110 },
        RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint: 1,
            transferred_bytes: 20,
        },
        RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint: 2,
            transferred_bytes: 9,
        },
    ] {
        let report = RemoteAttemptReport::new(
            current.job_id.clone(),
            current.attempt_id.clone(),
            current.fence_generation,
            RemoteEventId::new("invalid-progress").unwrap(),
            payload,
        )
        .unwrap();
        assert_eq!(
            FiniteProjection::new().project(&current, &report, authorization),
            Err(ProjectionError::UnmodeledPayload)
        );
        let actual = plan_remote_attempt_report(&current, &report, authorization);
        assert_eq!(actual.reason_code, RemoteAttemptReasonCode::TransitionRejected);
        assert_eq!(actual.next_state, current);
    }
    let mut report = report(&current, RemoteAttemptReportKind::Start);
    report.identity.payload_digest = RemotePayloadDigest::new(OTHER_RESULT).unwrap();
    assert_eq!(
        FiniteProjection::new().project(&current, &report, authorization),
        Err(ProjectionError::InvalidDigest)
    );
    assert_eq!(
        plan_remote_attempt_report(&current, &report, authorization).reason_code,
        RemoteAttemptReasonCode::PayloadDigestMismatch
    );
}

#[test]
fn exhausted_identity_domain_never_aliases_or_forgets_existing_values() {
    let mut domain = IdentityDomain::default();
    for index in 0..MAX_TRELLIS_PROJECTION_IDENTITIES {
        let identity = format!("identity-{index}");
        assert_eq!(domain.intern(&identity).unwrap(), u64::try_from(index + 1).unwrap());
    }
    assert_eq!(domain.intern("one-too-many"), Err(ProjectionError::IdentityCapacity));
    assert_eq!(domain.intern("identity-0"), Ok(1));
    assert_eq!(domain.intern("identity-4095"), Ok(4096));
}

#[test]
fn first_and_final_fences_keep_report_admission_separate_from_reassignment() {
    let authorization = RemoteAttemptAuthorizationFacts {
        worker_authorized: true,
        output_admission_authorized: true,
    };
    for fence in [1, u64::MAX] {
        let mut current = state_with_history(0);
        current.fence_generation = RemoteFenceGeneration::new(fence).unwrap();
        let mut projection = FiniteProjection::new();
        for (index, kind) in [
            RemoteAttemptReportKind::Start,
            RemoteAttemptReportKind::ResultReady,
            RemoteAttemptReportKind::Completion,
        ]
        .into_iter()
        .enumerate()
        {
            let mut report = report(&current, kind);
            report.identity.event_id = RemoteEventId::new(format!("boundary-{index}")).unwrap();
            let plan = assert_parity(&mut projection, &current, &report, authorization);
            assert_eq!(plan.disposition, RemoteAttemptApplyDisposition::Applied);
            assert_eq!(plan.next_state.fence_generation.get(), fence);
            current = plan.next_state;
        }
        assert_eq!(current.phase, RemoteAttemptPhase::Completed);
        let mut wrong_fence = report(&current, RemoteAttemptReportKind::Completion);
        wrong_fence.identity.fence_generation =
            RemoteFenceGeneration::new(if fence == 1 { 2 } else { fence - 1 }).unwrap();
        let rejected = assert_parity(&mut projection, &current, &wrong_fence, authorization);
        assert_eq!(rejected.disposition, RemoteAttemptApplyDisposition::Rejected);
        assert_eq!(
            rejected.reason_code,
            if fence == 1 {
                RemoteAttemptReasonCode::UnknownFenceRejected
            } else {
                RemoteAttemptReasonCode::StaleReportRejected
            }
        );
    }
    // The producer's representable_fence_advances theorem is arithmetic, not
    // an admission transition. The consumer must refuse overflow independently.
    assert_eq!(RemoteFenceGeneration::new(u64::MAX - 1).unwrap().advance().unwrap().get(), u64::MAX);
    assert_eq!(
        RemoteFenceGeneration::new(u64::MAX).unwrap().advance(),
        Err(RemoteAttemptReasonCode::FenceExhausted)
    );
}
