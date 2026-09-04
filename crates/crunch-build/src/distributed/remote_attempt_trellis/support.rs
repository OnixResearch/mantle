use super::model::TrellisEventClass;
use super::model::TrellisHistoryClass;
use super::model::TrellisIdentityClass;
use super::model::TrellisModelReportKind;
use super::model::TrellisProjectionError;
use super::model::TrellisProjectionPhase;
use super::model::TrellisProjectionReportKind;
use super::model::TrellisRemoteAttemptCase;
use super::model::TrellisResultLinkageClass;
use crate::distributed::RemoteAttemptPhase;
use crate::distributed::RemoteAttemptReportKind;
use crate::distributed::decide_remote_attempt_transition;

// r[impl remote_builds.trellis_admission_projection]
pub fn validate_supported_trellis_case(case: &TrellisRemoteAttemptCase) -> Result<(), TrellisProjectionError> {
    if case.identity != TrellisIdentityClass::Current {
        return Ok(());
    }
    if case.event != TrellisEventClass::New {
        return validate_replayed_or_conflicting_case(case);
    }
    validate_new_event_case(case)
}

fn validate_replayed_or_conflicting_case(case: &TrellisRemoteAttemptCase) -> Result<(), TrellisProjectionError> {
    if mantle_authority_admitted(case) {
        return Ok(());
    }
    Err(TrellisProjectionError::AuthorityOrderMismatch)
}

fn validate_new_event_case(case: &TrellisRemoteAttemptCase) -> Result<(), TrellisProjectionError> {
    let transition = transition_correspondence(case);
    if !case.worker.is_granted() {
        return validate_worker_denial(case, transition);
    }
    if case.report_kind.requires_output_in_mantle() && !case.output.is_granted() {
        return validate_output_denial(case, transition);
    }
    let next_phase = transition?;
    if case.history == TrellisHistoryClass::Full {
        validate_history_precedence(case, next_phase)?;
        return Ok(());
    }
    if is_result_retention_mismatch(case, next_phase) {
        return Err(TrellisProjectionError::ResultRetentionMismatch);
    }
    Ok(())
}

fn validate_worker_denial(
    case: &TrellisRemoteAttemptCase,
    transition: Result<Option<TrellisProjectionPhase>, TrellisProjectionError>,
) -> Result<(), TrellisProjectionError> {
    if !case.report_kind.requires_worker_in_trellis() {
        return Err(TrellisProjectionError::AuthorityMeaningMismatch);
    }
    if transition?.is_none() {
        return Err(TrellisProjectionError::AuthorityOrderMismatch);
    }
    Ok(())
}

fn validate_output_denial(
    case: &TrellisRemoteAttemptCase,
    transition: Result<Option<TrellisProjectionPhase>, TrellisProjectionError>,
) -> Result<(), TrellisProjectionError> {
    if case.report_kind == TrellisProjectionReportKind::ResultReady {
        return Err(TrellisProjectionError::AuthorityMeaningMismatch);
    }
    if transition?.is_none() {
        return Err(TrellisProjectionError::AuthorityOrderMismatch);
    }
    Ok(())
}

fn validate_history_precedence(
    case: &TrellisRemoteAttemptCase,
    next_phase: Option<TrellisProjectionPhase>,
) -> Result<(), TrellisProjectionError> {
    if next_phase.is_none() {
        return Err(TrellisProjectionError::HistoryOrderMismatch);
    }
    if completion_linkage_rejects(case) {
        return Err(TrellisProjectionError::HistoryOrderMismatch);
    }
    Ok(())
}

fn transition_correspondence(
    case: &TrellisRemoteAttemptCase,
) -> Result<Option<TrellisProjectionPhase>, TrellisProjectionError> {
    let mantle = mantle_transition(case);
    let trellis = trellis_transition(case.phase, case.report_kind.trellis_kind());
    if mantle != trellis {
        return Err(TrellisProjectionError::PhaseReportMismatch);
    }
    Ok(mantle)
}

fn mantle_transition(case: &TrellisRemoteAttemptCase) -> Option<TrellisProjectionPhase> {
    let phase = unproject_phase(case.phase);
    let kind = unproject_report_kind(case.report_kind);
    decide_remote_attempt_transition(phase, kind).next_phase.map(super::projection::project_phase)
}

const fn trellis_transition(
    phase: TrellisProjectionPhase,
    kind: TrellisModelReportKind,
) -> Option<TrellisProjectionPhase> {
    match (phase, kind) {
        (TrellisProjectionPhase::Queued, TrellisModelReportKind::Start)
        | (TrellisProjectionPhase::Running, TrellisModelReportKind::Progress) => Some(TrellisProjectionPhase::Running),
        (
            TrellisProjectionPhase::Running | TrellisProjectionPhase::Transferring,
            TrellisModelReportKind::Checkpoint,
        ) => Some(TrellisProjectionPhase::Transferring),
        (
            TrellisProjectionPhase::Running | TrellisProjectionPhase::Transferring,
            TrellisModelReportKind::ResultReady,
        ) => Some(TrellisProjectionPhase::ResultReady),
        (
            TrellisProjectionPhase::Running
            | TrellisProjectionPhase::Transferring
            | TrellisProjectionPhase::ResultReady,
            TrellisModelReportKind::Failure,
        ) => Some(TrellisProjectionPhase::Failed),
        (TrellisProjectionPhase::ResultReady, TrellisModelReportKind::Completion) => {
            Some(TrellisProjectionPhase::Completed)
        }
        _ => None,
    }
}

const fn unproject_phase(phase: TrellisProjectionPhase) -> RemoteAttemptPhase {
    match phase {
        TrellisProjectionPhase::Queued => RemoteAttemptPhase::Queued,
        TrellisProjectionPhase::Running => RemoteAttemptPhase::Running,
        TrellisProjectionPhase::Transferring => RemoteAttemptPhase::Transferring,
        TrellisProjectionPhase::ResultReady => RemoteAttemptPhase::FinishedUndelivered,
        TrellisProjectionPhase::Completed => RemoteAttemptPhase::Completed,
        TrellisProjectionPhase::Failed => RemoteAttemptPhase::Failed,
        TrellisProjectionPhase::Superseded => RemoteAttemptPhase::Superseded,
    }
}

const fn unproject_report_kind(kind: TrellisProjectionReportKind) -> RemoteAttemptReportKind {
    match kind {
        TrellisProjectionReportKind::Start => RemoteAttemptReportKind::Start,
        TrellisProjectionReportKind::Heartbeat => RemoteAttemptReportKind::Heartbeat,
        TrellisProjectionReportKind::LogAppend => RemoteAttemptReportKind::LogAppend,
        TrellisProjectionReportKind::Checkpoint => RemoteAttemptReportKind::TransferCheckpoint,
        TrellisProjectionReportKind::ResultReady => RemoteAttemptReportKind::ResultReady,
        TrellisProjectionReportKind::Failure => RemoteAttemptReportKind::Failure,
        TrellisProjectionReportKind::Completion => RemoteAttemptReportKind::Completion,
    }
}

fn mantle_authority_admitted(case: &TrellisRemoteAttemptCase) -> bool {
    if !case.worker.is_granted() {
        return false;
    }
    if case.report_kind.requires_output_in_mantle() && !case.output.is_granted() {
        return false;
    }
    true
}

fn completion_linkage_rejects(case: &TrellisRemoteAttemptCase) -> bool {
    case.report_kind == TrellisProjectionReportKind::Completion
        && case.result_linkage != TrellisResultLinkageClass::Matching
}

fn is_result_retention_mismatch(case: &TrellisRemoteAttemptCase, next_phase: Option<TrellisProjectionPhase>) -> bool {
    case.phase == TrellisProjectionPhase::ResultReady
        && case.report_kind == TrellisProjectionReportKind::Failure
        && next_phase == Some(TrellisProjectionPhase::Failed)
}
