use super::model::TrellisNormalizedOutcome;
use super::model::TrellisOutcomeDisposition;
use super::model::TrellisProjectionError;
use super::model::TrellisRejectClass;
use super::model::TrellisRemoteAttemptCase;
use super::model::TrellisRemoteAttemptProjection;
use super::model::TrellisResultEffect;
use super::model::TrellisResultLinkageClass;
use super::projection::project_phase;
use crate::distributed::RemoteAttemptApplyDisposition;
use crate::distributed::RemoteAttemptApplyPlan;
use crate::distributed::RemoteAttemptReasonCode;
use crate::distributed::RemoteAttemptState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeltaValues<T> {
    before: T,
    after: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutcomeDeltas {
    disposition: TrellisOutcomeDisposition,
    event: u8,
    progress: u8,
}

// r[impl remote_builds.trellis_admission_projection]
pub fn normalize_mantle_trellis_outcome(
    current: &RemoteAttemptState,
    projection: TrellisRemoteAttemptProjection,
    plan: &RemoteAttemptApplyPlan,
) -> Result<TrellisNormalizedOutcome, TrellisProjectionError> {
    validate_projection_anchor(current, projection)?;
    debug_assert_eq!(projection.progress_before, current.progress_events_applied);
    debug_assert_eq!(projection.result_present_before, current.result_digest_blake3.is_some());
    let disposition = project_disposition(plan.disposition);
    let is_state_preserved = plan.next_state == *current;
    if disposition != TrellisOutcomeDisposition::Applied && !is_state_preserved {
        return Err(TrellisProjectionError::OutcomeMutation);
    }
    let event_delta = collection_delta(DeltaValues {
        before: current.applied_events.len(),
        after: plan.next_state.applied_events.len(),
    })?;
    let progress_delta = value_delta(DeltaValues {
        before: current.progress_events_applied,
        after: plan.next_state.progress_events_applied,
    })?;
    validate_outcome_deltas(OutcomeDeltas {
        disposition,
        event: event_delta,
        progress: progress_delta,
    })?;
    Ok(TrellisNormalizedOutcome {
        disposition,
        reject_class: project_reject_class(plan, projection.case)?,
        next_phase: project_phase(plan.next_state.phase),
        progress_delta,
        event_delta,
        result_effect: project_result_effect(current, &plan.next_state),
        state_preserved: is_state_preserved,
    })
}

fn validate_projection_anchor(
    current: &RemoteAttemptState,
    projection: TrellisRemoteAttemptProjection,
) -> Result<(), TrellisProjectionError> {
    let event_count =
        u32::try_from(current.applied_events.len()).map_err(|_| TrellisProjectionError::OutcomeMutation)?;
    let is_current = projection.progress_before == current.progress_events_applied
        && projection.event_count_before == event_count
        && projection.result_present_before == current.result_digest_blake3.is_some()
        && projection.case.phase == project_phase(current.phase);
    if is_current {
        Ok(())
    } else {
        Err(TrellisProjectionError::OutcomeMutation)
    }
}

const fn project_disposition(disposition: RemoteAttemptApplyDisposition) -> TrellisOutcomeDisposition {
    match disposition {
        RemoteAttemptApplyDisposition::Applied => TrellisOutcomeDisposition::Applied,
        RemoteAttemptApplyDisposition::AlreadyApplied => TrellisOutcomeDisposition::Replayed,
        RemoteAttemptApplyDisposition::Rejected => TrellisOutcomeDisposition::Rejected,
    }
}

fn collection_delta(values: DeltaValues<usize>) -> Result<u8, TrellisProjectionError> {
    let delta = values.after.checked_sub(values.before).ok_or(TrellisProjectionError::OutcomeMutation)?;
    u8::try_from(delta).map_err(|_| TrellisProjectionError::OutcomeMutation)
}

fn value_delta(values: DeltaValues<u32>) -> Result<u8, TrellisProjectionError> {
    let delta = values.after.checked_sub(values.before).ok_or(TrellisProjectionError::OutcomeMutation)?;
    u8::try_from(delta).map_err(|_| TrellisProjectionError::OutcomeMutation)
}

fn validate_outcome_deltas(deltas: OutcomeDeltas) -> Result<(), TrellisProjectionError> {
    let is_valid = match deltas.disposition {
        TrellisOutcomeDisposition::Applied => deltas.event == 1 && deltas.progress <= 1,
        TrellisOutcomeDisposition::Replayed | TrellisOutcomeDisposition::Rejected => {
            deltas.event == 0 && deltas.progress == 0
        }
    };
    if is_valid {
        Ok(())
    } else {
        Err(TrellisProjectionError::OutcomeMutation)
    }
}

fn project_reject_class(
    plan: &RemoteAttemptApplyPlan,
    case: TrellisRemoteAttemptCase,
) -> Result<Option<TrellisRejectClass>, TrellisProjectionError> {
    if plan.disposition != RemoteAttemptApplyDisposition::Rejected {
        return Ok(None);
    }
    debug_assert_eq!(plan.disposition, RemoteAttemptApplyDisposition::Rejected);
    debug_assert_eq!(project_phase(plan.next_state.phase), case.phase);
    let class = match plan.reason_code {
        RemoteAttemptReasonCode::StaleReportRejected => TrellisRejectClass::StaleEpoch,
        RemoteAttemptReasonCode::UnknownFenceRejected => TrellisRejectClass::FutureEpoch,
        RemoteAttemptReasonCode::JobIdentityMismatch => TrellisRejectClass::WrongJob,
        RemoteAttemptReasonCode::AttemptIdentityMismatch => TrellisRejectClass::WrongRun,
        RemoteAttemptReasonCode::EventDigestConflict => TrellisRejectClass::EventConflict,
        RemoteAttemptReasonCode::TerminalAttempt => TrellisRejectClass::TerminalPhase,
        RemoteAttemptReasonCode::TransitionRejected => TrellisRejectClass::InvalidTransition,
        RemoteAttemptReasonCode::WorkerUnauthorized => TrellisRejectClass::WorkerDenied,
        RemoteAttemptReasonCode::OutputAdmissionUnauthorized => TrellisRejectClass::OutputDenied,
        RemoteAttemptReasonCode::ResultDigestMismatch => result_reject_class(case),
        RemoteAttemptReasonCode::EventRetentionExhausted => TrellisRejectClass::HistoryFull,
        _ => return Err(TrellisProjectionError::ReasonClassUnmapped),
    };
    Ok(Some(class))
}

const fn result_reject_class(case: TrellisRemoteAttemptCase) -> TrellisRejectClass {
    match case.result_linkage {
        TrellisResultLinkageClass::MissingRetained => TrellisRejectClass::MissingResult,
        TrellisResultLinkageClass::Mismatched => TrellisRejectClass::ResultMismatch,
        TrellisResultLinkageClass::NotApplicable | TrellisResultLinkageClass::Matching => {
            TrellisRejectClass::ResultMismatch
        }
    }
}

fn project_result_effect(before: &RemoteAttemptState, after: &RemoteAttemptState) -> TrellisResultEffect {
    if before.result_digest_blake3 == after.result_digest_blake3 {
        TrellisResultEffect::Preserved
    } else if before.result_digest_blake3.is_some() && after.result_digest_blake3.is_none() {
        TrellisResultEffect::Cleared
    } else {
        TrellisResultEffect::SetFromReport
    }
}
