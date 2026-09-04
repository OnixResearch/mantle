use super::super::*;
use crate::distributed::MAX_REMOTE_ATTEMPT_EVENTS;
use crate::distributed::RemoteAssignmentNonce;
use crate::distributed::RemoteAttemptAuthorizationFacts;
use crate::distributed::RemoteAttemptFailureClass;
use crate::distributed::RemoteAttemptId;
use crate::distributed::RemoteAttemptPhase;
use crate::distributed::RemoteAttemptReasonCode;
use crate::distributed::RemoteAttemptReport;
use crate::distributed::RemoteAttemptReportPayload;
use crate::distributed::RemoteAttemptRetryPolicy;
use crate::distributed::RemoteAttemptState;
use crate::distributed::RemoteAttemptTimeFacts;
use crate::distributed::RemoteEventId;
use crate::distributed::RemoteFenceGeneration;
use crate::distributed::RemoteJobId;
use crate::distributed::RemotePayloadDigest;
use crate::distributed::plan_remote_attempt_assignment;

pub(super) const TEST_OUTPUT_PRIMARY: &str = "1111111111111111111111111111111111111111111111111111111111111111";
pub(super) const TEST_CONFLICT_DIGEST: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const TEST_OUTPUT_ALTERNATE: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const TEST_ASSIGNMENT_NONCE_FIRST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_ASSIGNMENT_NONCE_SECOND: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const TEST_RETAINED_DIGEST: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const TEST_NOW_FIRST_UNIX_S: u64 = 10;
const TEST_NOW_SECOND_UNIX_S: u64 = 20;
const TEST_LAST_HEARTBEAT_UNIX_S: u64 = 30;
const TEST_HEARTBEAT_UNIX_S: u64 = 31;
const TEST_DEADLINE_UNIX_S: u64 = 1_000;
const TEST_PROGRESS_EVENTS: u32 = 3;
const TEST_AVAILABLE_EVENT_COUNT: usize = 3;
const TEST_CHECKPOINT_CURRENT: u64 = 3;
const TEST_CHECKPOINT_NEXT: u64 = 4;
const TEST_TRANSFERRED_BYTES_CURRENT: u64 = 30;
const TEST_TRANSFERRED_BYTES_NEXT: u64 = 40;

#[derive(Debug, Clone, Copy)]
pub(super) enum MatrixReportVariant {
    Start,
    Heartbeat,
    LogAppend,
    Checkpoint,
    ResultReady,
    Failure,
    CompletionPrimary,
    CompletionAlternate,
}

impl MatrixReportVariant {
    fn payload(self) -> RemoteAttemptReportPayload {
        match self {
            Self::Start => RemoteAttemptReportPayload::Start,
            Self::Heartbeat => RemoteAttemptReportPayload::Heartbeat {
                observed_unix_s: TEST_HEARTBEAT_UNIX_S,
            },
            Self::LogAppend => RemoteAttemptReportPayload::LogAppend {
                cursor: 1,
                bytes: "x".to_string(),
            },
            Self::Checkpoint => RemoteAttemptReportPayload::TransferCheckpoint {
                checkpoint: TEST_CHECKPOINT_NEXT,
                transferred_bytes: TEST_TRANSFERRED_BYTES_NEXT,
            },
            Self::ResultReady => RemoteAttemptReportPayload::ResultReady {
                output_digest_blake3: TEST_OUTPUT_PRIMARY.to_string(),
            },
            Self::Failure => RemoteAttemptReportPayload::Failure {
                failure_class: RemoteAttemptFailureClass::Retryable,
                reason_code: RemoteAttemptReasonCode::RetryAllowed,
            },
            Self::CompletionPrimary => RemoteAttemptReportPayload::Completion {
                output_digest_blake3: TEST_OUTPUT_PRIMARY.to_string(),
            },
            Self::CompletionAlternate => RemoteAttemptReportPayload::Completion {
                output_digest_blake3: TEST_OUTPUT_ALTERNATE.to_string(),
            },
        }
    }
}

pub(super) fn for_each_matrix_case(
    mut visit: impl FnMut(
        RemoteAttemptPhase,
        MatrixReportVariant,
        TrellisIdentityClass,
        TrellisEventClass,
        TrellisHistoryClass,
        TrellisScopeClass,
        TrellisScopeClass,
    ),
) {
    for phase in phases() {
        for variant in report_variants() {
            for identity in identity_classes() {
                for event in event_classes() {
                    for history in history_classes() {
                        for worker in scopes() {
                            for output in scopes() {
                                visit(phase, variant, identity, event, history, worker, output);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn matrix_case(
    phase: RemoteAttemptPhase,
    variant: MatrixReportVariant,
    identity: TrellisIdentityClass,
    event: TrellisEventClass,
    history: TrellisHistoryClass,
    worker: TrellisScopeClass,
    output: TrellisScopeClass,
) -> (RemoteAttemptState, RemoteAttemptReport, RemoteAttemptAuthorizationFacts) {
    let mut state = second_attempt();
    configure_state_phase(&mut state, phase);
    let mut report = RemoteAttemptReport::new(
        state.job_id.clone(),
        state.attempt_id.clone(),
        state.fence_generation,
        RemoteEventId::new("matrix-incoming").unwrap(),
        variant.payload(),
    )
    .unwrap();
    configure_report_identity(&state, &mut report, identity);
    configure_event_history(&mut state, &report, event, history);
    let authorization = RemoteAttemptAuthorizationFacts {
        worker_authorized: worker == TrellisScopeClass::Granted,
        output_admission_authorized: output == TrellisScopeClass::Granted,
    };
    (state, report, authorization)
}

fn second_attempt() -> RemoteAttemptState {
    let job_id = RemoteJobId::new("matrix-job").unwrap();
    let policy = RemoteAttemptRetryPolicy::default();
    let first = plan_remote_attempt_assignment(
        &job_id,
        "matrix-worker-first",
        RemoteAssignmentNonce::new(TEST_ASSIGNMENT_NONCE_FIRST).unwrap(),
        None,
        policy,
        time(TEST_NOW_FIRST_UNIX_S),
    )
    .unwrap();
    plan_remote_attempt_assignment(
        &job_id,
        "matrix-worker-second",
        RemoteAssignmentNonce::new(TEST_ASSIGNMENT_NONCE_SECOND).unwrap(),
        Some(&first),
        policy,
        time(TEST_NOW_SECOND_UNIX_S),
    )
    .unwrap()
}

fn time(now_unix_s: u64) -> RemoteAttemptTimeFacts {
    RemoteAttemptTimeFacts {
        now_unix_s,
        failure_observed_unix_s: now_unix_s,
        overall_deadline_unix_s: TEST_DEADLINE_UNIX_S,
    }
}

fn configure_state_phase(state: &mut RemoteAttemptState, phase: RemoteAttemptPhase) {
    state.phase = phase;
    state.progress_events_applied = TEST_PROGRESS_EVENTS;
    state.last_heartbeat_unix_s = Some(TEST_LAST_HEARTBEAT_UNIX_S);
    state.transfer_checkpoint =
        matches!(phase, RemoteAttemptPhase::Transferring | RemoteAttemptPhase::FinishedUndelivered)
            .then_some(TEST_CHECKPOINT_CURRENT);
    state.transferred_bytes = if state.transfer_checkpoint.is_some() {
        TEST_TRANSFERRED_BYTES_CURRENT
    } else {
        0
    };
    state.result_digest_blake3 =
        matches!(phase, RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed)
            .then(|| TEST_OUTPUT_PRIMARY.to_string());
}

fn configure_report_identity(
    state: &RemoteAttemptState,
    report: &mut RemoteAttemptReport,
    identity: TrellisIdentityClass,
) {
    match identity {
        TrellisIdentityClass::Current => {}
        TrellisIdentityClass::Stale => {
            report.identity.fence_generation = RemoteFenceGeneration::new(state.fence_generation.get() - 1).unwrap();
        }
        TrellisIdentityClass::Future => {
            report.identity.fence_generation = state.fence_generation.advance().unwrap();
        }
        TrellisIdentityClass::WrongJob => {
            report.identity.job_id = RemoteJobId::new("matrix-other-job").unwrap();
        }
        TrellisIdentityClass::WrongRun => {
            report.identity.attempt_id = RemoteAttemptId::new("matrix-other-run").unwrap();
        }
    }
}

fn configure_event_history(
    state: &mut RemoteAttemptState,
    report: &RemoteAttemptReport,
    event: TrellisEventClass,
    history: TrellisHistoryClass,
) {
    state.applied_events.clear();
    if event != TrellisEventClass::New {
        let digest = if event == TrellisEventClass::Replayed {
            report.identity.payload_digest.clone()
        } else {
            RemotePayloadDigest::new(TEST_CONFLICT_DIGEST).unwrap()
        };
        state.applied_events.insert(report.identity.event_id.clone(), digest);
    }
    let target = if history == TrellisHistoryClass::Full {
        MAX_REMOTE_ATTEMPT_EVENTS
    } else {
        TEST_AVAILABLE_EVENT_COUNT
    };
    while state.applied_events.len() < target {
        let index = state.applied_events.len();
        state.applied_events.insert(
            RemoteEventId::new(format!("matrix-retained-{index:04}")).unwrap(),
            RemotePayloadDigest::new(TEST_RETAINED_DIGEST).unwrap(),
        );
    }
    assert_eq!(state.progress_events_applied, TEST_PROGRESS_EVENTS);
    assert!(state.progress_events_applied <= u32::try_from(state.applied_events.len()).unwrap());
}

pub(super) fn unchecked_projection(
    state: &RemoteAttemptState,
    report: &RemoteAttemptReport,
    authorization: RemoteAttemptAuthorizationFacts,
) -> TrellisRemoteAttemptProjection {
    super::super::projection::classify_remote_attempt_for_trellis(state, report, authorization).unwrap()
}

fn phases() -> [RemoteAttemptPhase; 7] {
    [
        RemoteAttemptPhase::Queued,
        RemoteAttemptPhase::Running,
        RemoteAttemptPhase::Transferring,
        RemoteAttemptPhase::FinishedUndelivered,
        RemoteAttemptPhase::Completed,
        RemoteAttemptPhase::Failed,
        RemoteAttemptPhase::Superseded,
    ]
}

fn report_variants() -> [MatrixReportVariant; 8] {
    [
        MatrixReportVariant::Start,
        MatrixReportVariant::Heartbeat,
        MatrixReportVariant::LogAppend,
        MatrixReportVariant::Checkpoint,
        MatrixReportVariant::ResultReady,
        MatrixReportVariant::Failure,
        MatrixReportVariant::CompletionPrimary,
        MatrixReportVariant::CompletionAlternate,
    ]
}

fn identity_classes() -> [TrellisIdentityClass; 5] {
    [
        TrellisIdentityClass::Current,
        TrellisIdentityClass::Stale,
        TrellisIdentityClass::Future,
        TrellisIdentityClass::WrongJob,
        TrellisIdentityClass::WrongRun,
    ]
}

fn event_classes() -> [TrellisEventClass; 3] {
    [
        TrellisEventClass::New,
        TrellisEventClass::Replayed,
        TrellisEventClass::Conflict,
    ]
}

fn history_classes() -> [TrellisHistoryClass; 2] {
    [TrellisHistoryClass::Available, TrellisHistoryClass::Full]
}

fn scopes() -> [TrellisScopeClass; 2] {
    [TrellisScopeClass::Granted, TrellisScopeClass::Denied]
}
