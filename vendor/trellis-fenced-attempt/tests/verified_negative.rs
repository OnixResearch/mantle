use super::*;

#[test]
fn fences_authority_events_history_and_linkage_reject_without_output_authority() {
    let state = AdmissionState {
        job: 9, attempt: 3, fence: 8, phase: RESULT_READY, event_count: 2,
        result_present: true, result_id: 17,
    };
    let report = AdmissionReport {
        job: 9, attempt: 3, fence: 8, event_id: 13, digest_id: 21,
        event_relation: NEW_EVENT, kind: COMPLETION, worker_authorized: true,
        output_authorized: true, result_id: 17,
    };
    for (changed, limit, expected) in [
        (AdmissionReport { job: 10, ..report }, 1024, WRONG_JOB),
        (AdmissionReport { fence: 7, ..report }, 1024, STALE_FENCE),
        (AdmissionReport { fence: 9, ..report }, 1024, FUTURE_FENCE),
        (AdmissionReport { attempt: 4, ..report }, 1024, WRONG_ATTEMPT),
        (AdmissionReport { worker_authorized: false, ..report }, 1024, WORKER_DENIED),
        (AdmissionReport { output_authorized: false, ..report }, 1024, OUTPUT_DENIED),
        (AdmissionReport { result_id: 18, ..report }, 1024, RESULT_MISMATCH),
        (AdmissionReport { event_relation: CONFLICTING_EVENT, ..report }, 1024, EVENT_CONFLICT),
        (report, state.event_count, HISTORY_FULL),
    ] {
        let decision = admit(state, changed, limit);
        assert_eq!(decision.disposition, REJECTED);
        assert_eq!(decision.reason, expected);
        assert!(decision.next == state);
        assert!(!decision.output_admission_allowed);
    }
    let matching = admit(state, report, 1024);
    assert_eq!(matching.next.phase, COMPLETED);
    assert!(matching.output_admission_allowed);
    assert_eq!(matching.next.result_id, state.result_id);
}

#[test]
fn duplicate_and_conflicting_events_preserve_terminal_state() {
    let state = AdmissionState {
        job: 1, attempt: 2, fence: 8, phase: COMPLETED, event_count: 4,
        result_present: true, result_id: 44,
    };
    let report = AdmissionReport {
        job: 1, attempt: 2, fence: 8, event_id: 7, digest_id: 9,
        event_relation: SAME_EVENT, kind: COMPLETION, worker_authorized: true,
        output_authorized: true, result_id: 44,
    };
    assert_eq!(admit(state, report, 1024).disposition, REPLAYED);
    assert!(admit(state, report, 1024).next == state);
    let conflicting = AdmissionReport { digest_id: 10, event_relation: CONFLICTING_EVENT, ..report };
    assert_eq!(admit(state, conflicting, 1024).reason, EVENT_CONFLICT);
    assert!(admit(state, conflicting, 1024).next == state);
    for kind in [START, HEARTBEAT, LOG_APPEND, CHECKPOINT, READY, FAILURE, COMPLETION] {
        let changed = AdmissionReport { kind, event_relation: NEW_EVENT, ..report };
        let decision = admit(state, changed, 1024);
        assert_eq!(decision.reason, TERMINAL);
        assert!(decision.next == state);
    }
}

#[test]
fn invalid_status_tags_reject_before_any_transition() {
    let state = AdmissionState {
        job: 1, attempt: 2, fence: 8, phase: QUEUED, event_count: 0,
        result_present: false, result_id: 0,
    };
    let report = AdmissionReport {
        job: 1, attempt: 2, fence: 8, event_id: 7, digest_id: 9,
        event_relation: NEW_EVENT, kind: START, worker_authorized: true,
        output_authorized: true, result_id: 0,
    };
    for (state, report, limit) in [
        (AdmissionState { phase: 7, ..state }, report, 1024),
        (state, AdmissionReport { kind: 7, ..report }, 1024),
        (state, AdmissionReport { event_relation: 3, ..report }, 1024),
        (state, report, 0),
        (AdmissionState { fence: 0, ..state }, report, 1024),
    ] {
        let decision = admit(state, report, limit);
        assert_eq!(decision.disposition, REJECTED);
        assert_eq!(decision.reason, UNSUPPORTED);
        assert!(decision.next == state);
        assert!(!decision.output_admission_allowed);
    }
}

#[test]
fn early_or_unlinked_completion_never_exposes_a_result() {
    let running = AdmissionState {
        job: 1, attempt: 2, fence: 8, phase: RUNNING, event_count: 1,
        result_present: false, result_id: 0,
    };
    let report = AdmissionReport {
        job: 1, attempt: 2, fence: 8, event_id: 7, digest_id: 9,
        event_relation: NEW_EVENT, kind: COMPLETION, worker_authorized: true,
        output_authorized: true, result_id: 44,
    };
    let early = admit(running, report, 1024);
    assert_eq!(early.reason, INVALID_TRANSITION);
    assert!(early.next == running);
    let ready_without_link = AdmissionState { phase: RESULT_READY, ..running };
    let unlinked = admit(ready_without_link, report, 1024);
    assert_eq!(unlinked.reason, RESULT_MISMATCH);
    assert!(unlinked.next == ready_without_link);
    assert!(!unlinked.output_admission_allowed);
}
