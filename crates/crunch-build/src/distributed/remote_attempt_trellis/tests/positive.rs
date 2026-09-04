use super::super::super::*;
use super::fixtures::MatrixReportVariant;
use super::fixtures::matrix_case;
use crate::distributed::RemoteAttemptApplyDisposition;
use crate::distributed::RemoteAttemptPhase;
use crate::distributed::RemoteAttemptState;
use crate::distributed::plan_remote_attempt_report;

#[test]
// r[verify remote_builds.trellis_admission_safety]
// r[verify remote_builds.trellis_admission_projection]
fn every_supported_applied_transition_projects() {
    let cases = [
        (RemoteAttemptPhase::Queued, MatrixReportVariant::Start),
        (RemoteAttemptPhase::Running, MatrixReportVariant::Heartbeat),
        (RemoteAttemptPhase::Running, MatrixReportVariant::LogAppend),
        (RemoteAttemptPhase::Running, MatrixReportVariant::Checkpoint),
        (RemoteAttemptPhase::Transferring, MatrixReportVariant::Checkpoint),
        (RemoteAttemptPhase::Running, MatrixReportVariant::ResultReady),
        (RemoteAttemptPhase::Transferring, MatrixReportVariant::ResultReady),
        (RemoteAttemptPhase::Running, MatrixReportVariant::Failure),
        (RemoteAttemptPhase::Transferring, MatrixReportVariant::Failure),
        (RemoteAttemptPhase::FinishedUndelivered, MatrixReportVariant::CompletionPrimary),
    ];
    for (phase, variant) in cases {
        let (state, report, authorization) = matrix_case(
            phase,
            variant,
            TrellisIdentityClass::Current,
            TrellisEventClass::New,
            TrellisHistoryClass::Available,
            TrellisScopeClass::Granted,
            TrellisScopeClass::Granted,
        );
        let projection = project_remote_attempt_to_trellis(&state, &report, authorization).unwrap();
        let plan = plan_remote_attempt_report(&state, &report, authorization);
        let normalized = normalize_mantle_trellis_outcome(&state, projection, &plan).unwrap();

        assert_eq!(plan.disposition, RemoteAttemptApplyDisposition::Applied);
        assert_eq!(normalized.disposition, TrellisOutcomeDisposition::Applied);
        assert_eq!(normalized.event_delta, 1);
    }
}

#[test]
// r[verify remote_builds.trellis_admission_projection]
fn legacy_state_defaults_the_progress_counter_to_zero() {
    let (state, _, _) = matrix_case(
        RemoteAttemptPhase::Running,
        MatrixReportVariant::Heartbeat,
        TrellisIdentityClass::Current,
        TrellisEventClass::New,
        TrellisHistoryClass::Available,
        TrellisScopeClass::Granted,
        TrellisScopeClass::Granted,
    );
    let mut encoded = serde_json::to_value(state).unwrap();
    encoded.as_object_mut().unwrap().remove("progress_events_applied");
    let decoded: RemoteAttemptState = serde_json::from_value(encoded).unwrap();

    assert_eq!(decoded.progress_events_applied, 0);
    assert!(!decoded.applied_events.is_empty());
}

#[test]
// r[verify remote_builds.trellis_admission_safety]
fn replay_conflict_and_result_mismatch_project_as_preserving_outcomes() {
    let cases = [
        (
            RemoteAttemptPhase::Completed,
            MatrixReportVariant::Start,
            TrellisEventClass::Replayed,
            TrellisOutcomeDisposition::Replayed,
        ),
        (
            RemoteAttemptPhase::Running,
            MatrixReportVariant::Heartbeat,
            TrellisEventClass::Conflict,
            TrellisOutcomeDisposition::Rejected,
        ),
        (
            RemoteAttemptPhase::FinishedUndelivered,
            MatrixReportVariant::CompletionAlternate,
            TrellisEventClass::New,
            TrellisOutcomeDisposition::Rejected,
        ),
    ];
    for (phase, variant, event, expected) in cases {
        let (state, report, authorization) = matrix_case(
            phase,
            variant,
            TrellisIdentityClass::Current,
            event,
            TrellisHistoryClass::Available,
            TrellisScopeClass::Granted,
            TrellisScopeClass::Granted,
        );
        let projection = project_remote_attempt_to_trellis(&state, &report, authorization).unwrap();
        let plan = plan_remote_attempt_report(&state, &report, authorization);
        let normalized = normalize_mantle_trellis_outcome(&state, projection, &plan).unwrap();

        assert_eq!(normalized.disposition, expected);
        assert!(normalized.state_preserved);
        assert_eq!(normalized.event_delta, 0);
    }
}
