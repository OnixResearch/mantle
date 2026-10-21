//! A closed, finite decision kernel over caller-admitted remote-attempt facts.
//!
//! The event relation is an explicit input fact: the caller must determine it
//! from its complete retained history before invoking this kernel. This proof
//! does not verify history lookup, digest computation, or consumer projection.

use vstd::prelude::*;

verus! {

pub const QUEUED: u8 = 0;
pub const RUNNING: u8 = 1;
pub const TRANSFERRING: u8 = 2;
pub const RESULT_READY: u8 = 3;
pub const COMPLETED: u8 = 4;
pub const FAILED: u8 = 5;
pub const SUPERSEDED: u8 = 6;

pub const START: u8 = 0;
pub const HEARTBEAT: u8 = 1;
pub const LOG_APPEND: u8 = 2;
pub const CHECKPOINT: u8 = 3;
pub const READY: u8 = 4;
pub const FAILURE: u8 = 5;
pub const COMPLETION: u8 = 6;

pub const NEW_EVENT: u8 = 0;
pub const SAME_EVENT: u8 = 1;
pub const CONFLICTING_EVENT: u8 = 2;
pub const APPLIED: u8 = 0;
pub const REPLAYED: u8 = 1;
pub const REJECTED: u8 = 2;

pub const WRONG_JOB: u8 = 1;
pub const STALE_FENCE: u8 = 2;
pub const FUTURE_FENCE: u8 = 3;
pub const WRONG_ATTEMPT: u8 = 4;
pub const WORKER_DENIED: u8 = 5;
pub const OUTPUT_DENIED: u8 = 6;
pub const EVENT_CONFLICT: u8 = 7;
pub const HISTORY_FULL: u8 = 8;
pub const TERMINAL: u8 = 9;
pub const INVALID_TRANSITION: u8 = 10;
pub const RESULT_MISMATCH: u8 = 11;
pub const UNSUPPORTED: u8 = 12;

/// Complete *abstract* state; event bytes and unrelated progress stay in the
/// consumer state, and are not covered by this kernel's state-preservation proof.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AdmissionState {
    pub job: u64,
    pub attempt: u64,
    pub fence: u64,
    pub phase: u8,
    pub event_count: u32,
    pub result_present: bool,
    pub result_id: u64,
}

/// Caller-supplied facts; `event_relation` is classified by event ID and digest
/// against the retained history, not by this abstract decision kernel.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AdmissionReport {
    pub job: u64,
    pub attempt: u64,
    pub fence: u64,
    pub event_id: u64,
    pub digest_id: u64,
    pub event_relation: u8,
    pub kind: u8,
    pub worker_authorized: bool,
    pub output_authorized: bool,
    pub result_id: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AdmissionDecision {
    pub disposition: u8,
    pub reason: u8,
    pub next: AdmissionState,
    pub output_admission_allowed: bool,
}

// r[impl fenced-attempt-admission.rejection-preservation]
pub open spec fn rejected(state: AdmissionState, reason: u8) -> AdmissionDecision {
    AdmissionDecision { disposition: REJECTED, reason, next: state, output_admission_allowed: false }
}

// r[impl fenced-attempt-admission.event-idempotence]
pub open spec fn replayed(state: AdmissionState) -> AdmissionDecision {
    AdmissionDecision { disposition: REPLAYED, reason: 0, next: state, output_admission_allowed: false }
}

// r[impl fenced-attempt-admission.refinement]
pub open spec fn transition_spec(phase: u8, kind: u8) -> Option<u8> {
    if phase == QUEUED && kind == START { Some(RUNNING) }
    else if phase == QUEUED && kind == FAILURE { Some(FAILED) }
    else if (phase == RUNNING || phase == TRANSFERRING) && (kind == HEARTBEAT || kind == LOG_APPEND) { Some(phase) }
    else if (phase == RUNNING || phase == TRANSFERRING) && kind == CHECKPOINT { Some(TRANSFERRING) }
    else if (phase == RUNNING || phase == TRANSFERRING) && kind == READY { Some(RESULT_READY) }
    else if (phase == RUNNING || phase == TRANSFERRING || phase == RESULT_READY) && kind == FAILURE { Some(FAILED) }
    else if phase == RESULT_READY && kind == COMPLETION { Some(COMPLETED) }
    else { None }
}

// r[impl fenced-attempt-admission.refinement]
pub open spec fn admit_spec(state: AdmissionState, report: AdmissionReport, max_events: u32) -> AdmissionDecision {
    if state.fence == 0 || report.fence == 0 || state.phase > SUPERSEDED || report.kind > COMPLETION
       || report.event_relation > CONFLICTING_EVENT || max_events == 0 {
        rejected(state, UNSUPPORTED)
    } else if report.job != state.job { rejected(state, WRONG_JOB) }
    else if report.fence < state.fence { rejected(state, STALE_FENCE) }
    else if report.fence > state.fence { rejected(state, FUTURE_FENCE) }
    else if report.attempt != state.attempt { rejected(state, WRONG_ATTEMPT) }
    else if !report.worker_authorized { rejected(state, WORKER_DENIED) }
    else if (report.kind == READY || report.kind == COMPLETION) && !report.output_authorized { rejected(state, OUTPUT_DENIED) }
    else if report.event_relation == SAME_EVENT { replayed(state) }
    else if report.event_relation == CONFLICTING_EVENT { rejected(state, EVENT_CONFLICT) }
    else if state.event_count >= max_events { rejected(state, HISTORY_FULL) }
    else if state.phase == COMPLETED || state.phase == FAILED || state.phase == SUPERSEDED { rejected(state, TERMINAL) }
    else if transition_spec(state.phase, report.kind) is None { rejected(state, INVALID_TRANSITION) }
    else if report.kind == COMPLETION && (!state.result_present || state.result_id != report.result_id) {
        rejected(state, RESULT_MISMATCH)
    } else {
        let next_phase = transition_spec(state.phase, report.kind)->Some_0;
        let next_result_present = if report.kind == FAILURE { false }
            else if report.kind == READY { true } else { state.result_present };
        let next_result_id = if report.kind == FAILURE { 0 }
            else if report.kind == READY { report.result_id } else { state.result_id };
        AdmissionDecision {
            disposition: APPLIED, reason: 0,
            next: AdmissionState {
                job: state.job, attempt: state.attempt, fence: state.fence,
                phase: next_phase, event_count: (state.event_count + 1) as u32,
                result_present: next_result_present, result_id: next_result_id,
            },
            output_admission_allowed: report.kind == READY || report.kind == COMPLETION,
        }
    }
}

// r[impl fenced-attempt-admission.refinement]
pub fn transition(phase: u8, kind: u8) -> (next: Option<u8>)
    ensures next == transition_spec(phase, kind),
{
    if phase == QUEUED && kind == START { Some(RUNNING) }
    else if phase == QUEUED && kind == FAILURE { Some(FAILED) }
    else if (phase == RUNNING || phase == TRANSFERRING) && (kind == HEARTBEAT || kind == LOG_APPEND) { Some(phase) }
    else if (phase == RUNNING || phase == TRANSFERRING) && kind == CHECKPOINT { Some(TRANSFERRING) }
    else if (phase == RUNNING || phase == TRANSFERRING) && kind == READY { Some(RESULT_READY) }
    else if (phase == RUNNING || phase == TRANSFERRING || phase == RESULT_READY) && kind == FAILURE { Some(FAILED) }
    else if phase == RESULT_READY && kind == COMPLETION { Some(COMPLETED) }
    else { None }
}

fn reject_exec(state: AdmissionState, reason: u8) -> (decision: AdmissionDecision)
    ensures decision == rejected(state, reason),
{
    AdmissionDecision { disposition: REJECTED, reason, next: state, output_admission_allowed: false }
}

// r[impl fenced-attempt-admission.refinement]
pub fn admit(state: AdmissionState, report: AdmissionReport, max_events: u32) -> (decision: AdmissionDecision)
    ensures decision == admit_spec(state, report, max_events),
{
    if state.fence == 0 || report.fence == 0 || state.phase > SUPERSEDED || report.kind > COMPLETION
       || report.event_relation > CONFLICTING_EVENT || max_events == 0 {
        return reject_exec(state, UNSUPPORTED);
    }
    if report.job != state.job { return reject_exec(state, WRONG_JOB); }
    if report.fence < state.fence { return reject_exec(state, STALE_FENCE); }
    if report.fence > state.fence { return reject_exec(state, FUTURE_FENCE); }
    if report.attempt != state.attempt { return reject_exec(state, WRONG_ATTEMPT); }
    if !report.worker_authorized { return reject_exec(state, WORKER_DENIED); }
    if (report.kind == READY || report.kind == COMPLETION) && !report.output_authorized {
        return reject_exec(state, OUTPUT_DENIED);
    }
    if report.event_relation == SAME_EVENT {
        return AdmissionDecision { disposition: REPLAYED, reason: 0, next: state, output_admission_allowed: false };
    }
    if report.event_relation == CONFLICTING_EVENT { return reject_exec(state, EVENT_CONFLICT); }
    if state.event_count >= max_events { return reject_exec(state, HISTORY_FULL); }
    if state.phase == COMPLETED || state.phase == FAILED || state.phase == SUPERSEDED {
        return reject_exec(state, TERMINAL);
    }
    let next_phase = match transition(state.phase, report.kind) {
        Some(phase) => phase,
        None => return reject_exec(state, INVALID_TRANSITION),
    };
    if report.kind == COMPLETION && (!state.result_present || state.result_id != report.result_id) {
        return reject_exec(state, RESULT_MISMATCH);
    }
    let next_result_present = if report.kind == FAILURE { false }
        else if report.kind == READY { true } else { state.result_present };
    let next_result_id = if report.kind == FAILURE { 0 }
        else if report.kind == READY { report.result_id } else { state.result_id };
    AdmissionDecision {
        disposition: APPLIED, reason: 0,
        next: AdmissionState {
            job: state.job, attempt: state.attempt, fence: state.fence,
            phase: next_phase, event_count: state.event_count + 1,
            result_present: next_result_present, result_id: next_result_id,
        },
        output_admission_allowed: report.kind == READY || report.kind == COMPLETION,
    }
}

// r[verify fenced-attempt-admission.fence-safety]
pub proof fn rejected_fence_preserves(state: AdmissionState, report: AdmissionReport, max_events: u32)
    requires state.fence != 0, report.fence != 0, state.phase <= SUPERSEDED,
        report.kind <= COMPLETION, report.event_relation <= CONFLICTING_EVENT, max_events > 0,
        report.job == state.job,
        report.fence != state.fence,
    ensures admit_spec(state, report, max_events).disposition == REJECTED,
        admit_spec(state, report, max_events).next == state,
{}

// r[verify fenced-attempt-admission.terminal-closure]
pub proof fn terminal_new_event_preserves(state: AdmissionState, report: AdmissionReport, max_events: u32)
    requires state.phase == COMPLETED || state.phase == FAILED || state.phase == SUPERSEDED,
        state.fence != 0, report.fence == state.fence, report.job == state.job,
        report.attempt == state.attempt, report.worker_authorized,
        report.output_authorized, report.event_relation == NEW_EVENT,
        report.kind <= COMPLETION, state.event_count < max_events,
    ensures admit_spec(state, report, max_events).disposition == REJECTED,
        admit_spec(state, report, max_events).next == state,
{}

// r[verify fenced-attempt-admission.event-idempotence]
pub proof fn duplicate_and_conflict_preserve(state: AdmissionState, report: AdmissionReport, max_events: u32)
    requires state.fence != 0, report.fence == state.fence,
        state.phase <= SUPERSEDED, report.kind <= COMPLETION,
        report.job == state.job, report.attempt == state.attempt,
        report.worker_authorized, report.output_authorized,
        report.event_relation == SAME_EVENT || report.event_relation == CONFLICTING_EVENT,
        max_events > 0,
    ensures admit_spec(state, report, max_events).next == state,
        report.event_relation == SAME_EVENT ==> admit_spec(state, report, max_events).disposition == REPLAYED,
        report.event_relation == CONFLICTING_EVENT ==> admit_spec(state, report, max_events).disposition == REJECTED,
{}

// r[verify fenced-attempt-admission.completion-linkage]
pub proof fn applied_completion_requires_linkage(state: AdmissionState, report: AdmissionReport, max_events: u32)
    requires report.kind == COMPLETION,
        admit_spec(state, report, max_events).disposition == APPLIED,
    ensures state.phase == RESULT_READY, state.result_present,
        state.result_id == report.result_id, report.worker_authorized, report.output_authorized,
        admit_spec(state, report, max_events).next.phase == COMPLETED,
{}

// r[verify fenced-attempt-admission.rejection-preservation]
pub proof fn non_applied_preserves(state: AdmissionState, report: AdmissionReport, max_events: u32)
    ensures admit_spec(state, report, max_events).disposition != APPLIED ==>
        admit_spec(state, report, max_events).next == state,
{}

// r[verify fenced-attempt-admission.fence-safety]
pub proof fn representable_fence_advances(fence: u64)
    requires 0 < fence && fence < u64::MAX,
    ensures 0 < fence + 1 && fence + 1 > fence,
{}

} // verus!

#[cfg(test)]
#[path = "tests/verified_positive.rs"]
mod verified_positive;
#[cfg(test)]
#[path = "tests/verified_negative.rs"]
mod verified_negative;

