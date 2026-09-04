use super::model::TrellisEventClass;
use super::model::TrellisHistoryClass;
use super::model::TrellisIdentityClass;
use super::model::TrellisProjectionError;
use super::model::TrellisProjectionPhase;
use super::model::TrellisProjectionReportKind;
use super::model::TrellisRemoteAttemptCase;
use super::model::TrellisRemoteAttemptProjection;
use super::model::TrellisResultLinkageClass;
use super::model::TrellisScopeClass;
use super::support::validate_supported_trellis_case;
use crate::distributed::MAX_REMOTE_ATTEMPT_EVENTS;
use crate::distributed::RemoteAttemptAuthorizationFacts;
use crate::distributed::RemoteAttemptPhase;
use crate::distributed::RemoteAttemptReasonCode;
use crate::distributed::RemoteAttemptReport;
use crate::distributed::RemoteAttemptReportKind;
use crate::distributed::RemoteAttemptReportPayload;
use crate::distributed::RemoteAttemptState;
use crate::distributed::RemoteEventDisposition;
use crate::distributed::RemoteFenceDisposition;
use crate::distributed::classify_remote_attempt_event;
use crate::distributed::remote_attempt::validate_payload_linkage;
use crate::distributed::remote_attempt::validate_report_identity;
use crate::distributed::remote_attempt::validate_report_payload;
use crate::distributed::validate_remote_attempt_fence;

const BLAKE3_HEX_LENGTH_CHARS: usize = 64;

// r[impl remote_builds.trellis_admission_projection]
pub fn project_remote_attempt_to_trellis(
    current: &RemoteAttemptState,
    report: &RemoteAttemptReport,
    authorization: RemoteAttemptAuthorizationFacts,
) -> Result<TrellisRemoteAttemptProjection, TrellisProjectionError> {
    let projection = classify_remote_attempt_for_trellis(current, report, authorization)?;
    validate_supported_trellis_case(&projection.case)?;
    Ok(projection)
}

pub(super) fn classify_remote_attempt_for_trellis(
    current: &RemoteAttemptState,
    report: &RemoteAttemptReport,
    authorization: RemoteAttemptAuthorizationFacts,
) -> Result<TrellisRemoteAttemptProjection, TrellisProjectionError> {
    validate_projection_state(current)?;
    validate_projection_report(current, report)?;
    debug_assert!(current.applied_events.len() <= MAX_REMOTE_ATTEMPT_EVENTS);
    debug_assert!(state_result_shape_is_admitted(current));
    let event_count_before =
        u32::try_from(current.applied_events.len()).map_err(|_| TrellisProjectionError::StateNotAdmitted)?;
    Ok(TrellisRemoteAttemptProjection {
        case: TrellisRemoteAttemptCase {
            phase: project_phase(current.phase),
            report_kind: project_report_kind(report.payload.kind()),
            identity: project_identity(current, report),
            event: project_event(current, report),
            history: project_history(current),
            worker: project_scope(authorization.worker_authorized),
            output: project_scope(authorization.output_admission_authorized),
            result_linkage: project_result_linkage(current, &report.payload),
        },
        progress_before: current.progress_events_applied,
        event_count_before,
        result_present_before: current.result_digest_blake3.is_some(),
    })
}

fn validate_projection_state(current: &RemoteAttemptState) -> Result<(), TrellisProjectionError> {
    let event_count =
        u32::try_from(current.applied_events.len()).map_err(|_| TrellisProjectionError::StateNotAdmitted)?;
    if current.applied_events.len() > MAX_REMOTE_ATTEMPT_EVENTS {
        return Err(TrellisProjectionError::StateNotAdmitted);
    }
    if current.progress_events_applied > event_count {
        return Err(TrellisProjectionError::StateNotAdmitted);
    }
    if !state_result_shape_is_admitted(current) {
        return Err(TrellisProjectionError::StateNotAdmitted);
    }
    if current.result_digest_blake3.as_deref().is_some_and(|digest| !is_blake3_hex_digest(digest)) {
        return Err(TrellisProjectionError::StateNotAdmitted);
    }
    Ok(())
}

fn state_result_shape_is_admitted(current: &RemoteAttemptState) -> bool {
    match current.phase {
        RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed => {
            current.result_digest_blake3.is_some()
        }
        RemoteAttemptPhase::Queued
        | RemoteAttemptPhase::Running
        | RemoteAttemptPhase::Transferring
        | RemoteAttemptPhase::Failed
        | RemoteAttemptPhase::Superseded => current.result_digest_blake3.is_none(),
    }
}

fn validate_projection_report(
    current: &RemoteAttemptState,
    report: &RemoteAttemptReport,
) -> Result<(), TrellisProjectionError> {
    if validate_report_identity(report).is_some() {
        return Err(TrellisProjectionError::ReportNotAdmitted);
    }
    if validate_report_payload(report).is_some() {
        return Err(TrellisProjectionError::ReportNotAdmitted);
    }
    if validate_payload_linkage(current, &report.payload) == Some(RemoteAttemptReasonCode::TransitionRejected) {
        return Err(TrellisProjectionError::ReportNotAdmitted);
    }
    Ok(())
}

pub(super) fn project_phase(phase: RemoteAttemptPhase) -> TrellisProjectionPhase {
    match phase {
        RemoteAttemptPhase::Queued => TrellisProjectionPhase::Queued,
        RemoteAttemptPhase::Running => TrellisProjectionPhase::Running,
        RemoteAttemptPhase::Transferring => TrellisProjectionPhase::Transferring,
        RemoteAttemptPhase::FinishedUndelivered => TrellisProjectionPhase::ResultReady,
        RemoteAttemptPhase::Completed => TrellisProjectionPhase::Completed,
        RemoteAttemptPhase::Failed => TrellisProjectionPhase::Failed,
        RemoteAttemptPhase::Superseded => TrellisProjectionPhase::Superseded,
    }
}

pub(super) fn project_report_kind(kind: RemoteAttemptReportKind) -> TrellisProjectionReportKind {
    match kind {
        RemoteAttemptReportKind::Start => TrellisProjectionReportKind::Start,
        RemoteAttemptReportKind::Heartbeat => TrellisProjectionReportKind::Heartbeat,
        RemoteAttemptReportKind::LogAppend => TrellisProjectionReportKind::LogAppend,
        RemoteAttemptReportKind::TransferCheckpoint => TrellisProjectionReportKind::Checkpoint,
        RemoteAttemptReportKind::ResultReady => TrellisProjectionReportKind::ResultReady,
        RemoteAttemptReportKind::Failure => TrellisProjectionReportKind::Failure,
        RemoteAttemptReportKind::Completion => TrellisProjectionReportKind::Completion,
    }
}

pub(super) fn project_identity(current: &RemoteAttemptState, report: &RemoteAttemptReport) -> TrellisIdentityClass {
    match validate_remote_attempt_fence(current, &report.identity) {
        RemoteFenceDisposition::Current => TrellisIdentityClass::Current,
        RemoteFenceDisposition::Stale => TrellisIdentityClass::Stale,
        RemoteFenceDisposition::UnknownFuture => TrellisIdentityClass::Future,
        RemoteFenceDisposition::JobMismatch => TrellisIdentityClass::WrongJob,
        RemoteFenceDisposition::AttemptMismatch => TrellisIdentityClass::WrongRun,
    }
}

pub(super) fn project_event(current: &RemoteAttemptState, report: &RemoteAttemptReport) -> TrellisEventClass {
    match classify_remote_attempt_event(&current.applied_events, &report.identity) {
        RemoteEventDisposition::New => TrellisEventClass::New,
        RemoteEventDisposition::AlreadyApplied => TrellisEventClass::Replayed,
        RemoteEventDisposition::Conflict => TrellisEventClass::Conflict,
    }
}

pub(super) fn project_history(current: &RemoteAttemptState) -> TrellisHistoryClass {
    if current.applied_events.len() == MAX_REMOTE_ATTEMPT_EVENTS {
        TrellisHistoryClass::Full
    } else {
        TrellisHistoryClass::Available
    }
}

pub(super) const fn project_scope(granted: bool) -> TrellisScopeClass {
    if granted {
        TrellisScopeClass::Granted
    } else {
        TrellisScopeClass::Denied
    }
}

pub(super) fn project_result_linkage(
    current: &RemoteAttemptState,
    payload: &RemoteAttemptReportPayload,
) -> TrellisResultLinkageClass {
    match payload {
        RemoteAttemptReportPayload::ResultReady { .. } => TrellisResultLinkageClass::MissingRetained,
        RemoteAttemptReportPayload::Completion { output_digest_blake3 } => {
            match current.result_digest_blake3.as_deref() {
                None => TrellisResultLinkageClass::MissingRetained,
                Some(retained) if retained == output_digest_blake3 => TrellisResultLinkageClass::Matching,
                Some(_) => TrellisResultLinkageClass::Mismatched,
            }
        }
        _ => TrellisResultLinkageClass::NotApplicable,
    }
}

fn is_blake3_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
