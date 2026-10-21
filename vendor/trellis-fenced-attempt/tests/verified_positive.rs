use super::*;

#[test]
fn accepted_transitions_retain_exact_result_linkage() {
    let mut state = AdmissionState {
        job: 1, attempt: 2, fence: 1, phase: QUEUED, event_count: 0,
        result_present: false, result_id: 0,
    };
    let mut report = AdmissionReport {
        job: 1, attempt: 2, fence: 1, event_id: 1, digest_id: 10,
        event_relation: NEW_EVENT, kind: START, worker_authorized: true,
        output_authorized: true, result_id: 0,
    };
    for (kind, phase) in [
        (START, RUNNING),
        (HEARTBEAT, RUNNING),
        (LOG_APPEND, RUNNING),
        (CHECKPOINT, TRANSFERRING),
        (HEARTBEAT, TRANSFERRING),
        (LOG_APPEND, TRANSFERRING),
        (CHECKPOINT, TRANSFERRING),
        (READY, RESULT_READY),
        (COMPLETION, COMPLETED),
    ] {
        report.kind = kind;
        report.event_id += 1;
        report.digest_id += 1;
        report.result_id = 44;
        let decision = admit(state, report, 1024);
        assert_eq!(decision.disposition, APPLIED);
        assert_eq!(decision.next.phase, phase);
        assert_eq!(decision.next.event_count, state.event_count + 1);
        assert_eq!(decision.output_admission_allowed, kind == READY || kind == COMPLETION);
        state = decision.next;
    }
    assert!(state.result_present);
    assert_eq!(state.result_id, 44);
}

#[test]
fn every_phase_kind_pair_has_the_named_transition() {
    let phases = [QUEUED, RUNNING, TRANSFERRING, RESULT_READY, COMPLETED, FAILED, SUPERSEDED];
    let kinds = [START, HEARTBEAT, LOG_APPEND, CHECKPOINT, READY, FAILURE, COMPLETION];
    let expected = [
        [Some(RUNNING), None, None, None, None, Some(FAILED), None],
        [None, Some(RUNNING), Some(RUNNING), Some(TRANSFERRING), Some(RESULT_READY), Some(FAILED), None],
        [None, Some(TRANSFERRING), Some(TRANSFERRING), Some(TRANSFERRING), Some(RESULT_READY), Some(FAILED), None],
        [None, None, None, None, None, Some(FAILED), Some(COMPLETED)],
        [None; 7],
        [None; 7],
        [None; 7],
    ];
    for (phase_index, phase) in phases.into_iter().enumerate() {
        for (kind_index, kind) in kinds.into_iter().enumerate() {
            assert_eq!(transition(phase, kind), expected[phase_index][kind_index], "{phase} + {kind}");
        }
    }
}

#[test]
fn queued_and_ready_failure_close_without_output_authority() {
    for phase in [QUEUED, RESULT_READY] {
        let state = AdmissionState {
            job: 1, attempt: 2, fence: 8, phase, event_count: 1,
            result_present: phase == RESULT_READY, result_id: 17,
        };
        let report = AdmissionReport {
            job: 1, attempt: 2, fence: 8, event_id: 9, digest_id: 12,
            event_relation: NEW_EVENT, kind: FAILURE, worker_authorized: true,
            output_authorized: false, result_id: 0,
        };
        let decision = admit(state, report, 1024);
        assert_eq!(decision.disposition, APPLIED);
        assert_eq!(decision.next.phase, FAILED);
        assert!(!decision.next.result_present);
        assert_eq!(decision.next.result_id, 0);
        assert!(!decision.output_admission_allowed);
    }
}
