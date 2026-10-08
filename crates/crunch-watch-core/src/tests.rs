use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

fn admitted(ids: &[&str]) -> GoalSet {
    GoalSet::admit(ids.iter().map(|identity| (*identity).to_string()).collect()).unwrap()
}

fn classified(diff: &GoalDiff) -> Vec<(WatchEventKind, &str)> {
    diff.events.iter().map(|event| (event.kind, event.goal_identity.as_deref().unwrap())).collect()
}

#[test]
fn editing_one_root_retracts_changed_identity_and_retains_unrelated_root() {
    let previous = admitted(&["/store/b.drv", "/store/a-v1.drv"]);
    let current = admitted(&["/store/b.drv", "/store/a-v2.drv"]);
    let transition = diff(&previous, &current, "watch-run-one", 3).unwrap();
    assert_eq!(transition.run_identity, "watch-run-one");
    assert_eq!(classified(&transition), vec![
        (WatchEventKind::Retracted, "/store/a-v1.drv"),
        (WatchEventKind::Added, "/store/a-v2.drv"),
        (WatchEventKind::Retained, "/store/b.drv"),
    ]);
}

#[test]
fn adding_root_emits_one_added_event_and_retains_previous_identity() {
    let previous = admitted(&["/store/old.drv"]);
    let current = admitted(&["/store/new.drv", "/store/old.drv"]);
    let transition = diff(&previous, &current, "same-run", 2).unwrap();
    assert_eq!(classified(&transition), vec![
        (WatchEventKind::Added, "/store/new.drv"),
        (WatchEventKind::Retained, "/store/old.drv"),
    ]);
    assert!(current.contains("/store/old.drv"));
    assert!(!previous.contains("/store/new.drv"));
}

#[test]
fn retraction_requires_build_cancellation_and_disallows_late_success() {
    let previous = admitted(&["/store/deleted.drv"]);
    let transition = diff(&previous, &admitted(&[]), "run", 1).unwrap();
    assert_eq!(classified(&transition), vec![(WatchEventKind::Retracted, "/store/deleted.drv")]);
    assert_eq!(retract(GoalExecutionState::Building), RetractionDecision {
        cancel_in_flight: true,
        release_reservation: false,
        admit_late_success: false
    });
    for state in [
        GoalExecutionState::Queued,
        GoalExecutionState::Stopped,
        GoalExecutionState::Complete,
    ] {
        let decision = retract(state);
        assert!(!decision.cancel_in_flight);
        assert!(decision.release_reservation);
        assert!(!decision.admit_late_success);
    }
}

#[test]
fn rejected_generation_has_no_retractions_and_preserves_admitted_set() {
    let previous = admitted(&["/store/running.drv", "/store/complete.drv"]);
    let rejection = reject_generation("run", "Nickel syntax error".to_string(), 1).unwrap();
    assert_eq!(rejection.events[0].kind, WatchEventKind::Rejected);
    assert_eq!(rejection.events[0].goal_identity, None);
    assert_eq!(rejection.events[0].diagnostic.as_deref(), Some("Nickel syntax error"));
    assert_eq!(previous, admitted(&["/store/complete.drv", "/store/running.drv"]));
    assert!(!rejection.events.iter().any(|event| event.kind == WatchEventKind::Retracted));
}

#[test]
fn coalesced_edits_never_commit_or_publish_the_superseded_goal() {
    let mut loop_state = WatchLoop::new("watch-process".to_string(), admitted(&["old", "retained"])).unwrap();
    let first = loop_state.offer(admitted(&["edit-one", "retained"]), 3).unwrap();
    assert_eq!(loop_state.pending().unwrap().0, first);
    let second = loop_state.offer(admitted(&["edit-two", "retained"]), 3).unwrap();
    assert_eq!(loop_state.commit(first), Err(WatchError::StaleGeneration));
    assert_eq!(classified(loop_state.pending().unwrap().1), vec![
        (WatchEventKind::Added, "edit-two"),
        (WatchEventKind::Retracted, "old"),
        (WatchEventKind::Retained, "retained"),
    ]);
    assert_eq!(loop_state.pending().unwrap().1.run_identity, "watch-process");
    assert_eq!(loop_state.admitted(), &admitted(&["old", "retained"]));
    loop_state.commit(second).unwrap();
    assert_eq!(loop_state.admitted(), &admitted(&["edit-two", "retained"]));
    assert!(loop_state.pending().is_none());
}

#[test]
fn invalid_generation_discards_pending_edit_without_retracting_admitted_goals() {
    let mut loop_state = WatchLoop::new("run".to_string(), admitted(&["running"])).unwrap();
    let pending = loop_state.offer(admitted(&["broken"]), 2).unwrap();
    let rejected = loop_state.reject("syntax error".to_string(), 1).unwrap();
    assert_eq!(rejected.events[0].kind, WatchEventKind::Rejected);
    assert_eq!(loop_state.commit(pending), Err(WatchError::StaleGeneration));
    assert_eq!(loop_state.admitted(), &admitted(&["running"]));
    let recovery = loop_state.offer(admitted(&["recovered"]), 2).unwrap();
    assert_eq!(classified(loop_state.pending().unwrap().1), vec![
        (WatchEventKind::Added, "recovered"),
        (WatchEventKind::Retracted, "running")
    ]);
    loop_state.commit(recovery).unwrap();
    assert_eq!(loop_state.admitted(), &admitted(&["recovered"]));
}

#[test]
fn overflowing_event_budget_cannot_promote_prior_pending_generation() {
    let mut loop_state = WatchLoop::new("run".to_string(), admitted(&["a", "b"])).unwrap();
    let prior = loop_state.offer(admitted(&["a", "b"]), 2).unwrap();
    assert_eq!(loop_state.offer(admitted(&["c", "d"]), 3), Err(WatchError::EventBudget));
    assert!(loop_state.pending().is_none());
    assert_eq!(loop_state.commit(prior), Err(WatchError::StaleGeneration));
    assert_eq!(loop_state.admitted(), &admitted(&["a", "b"]));
}

#[test]
fn exhausted_generation_counter_never_commits_an_old_offer() {
    let mut loop_state = WatchLoop::new("run".to_string(), admitted(&["running"])).unwrap();
    let old = loop_state.offer(admitted(&["candidate"]), 2).unwrap();
    loop_state.last_generation = u64::MAX;
    assert_eq!(loop_state.offer(admitted(&["new-candidate"]), 2), Err(WatchError::GenerationOverflow));
    assert!(loop_state.pending().is_none());
    assert_eq!(loop_state.commit(old), Err(WatchError::StaleGeneration));
    assert_eq!(loop_state.admitted(), &admitted(&["running"]));
}

#[test]
fn diff_from_last_admission_ignores_unadmitted_intermediate_and_duplicates() {
    let previous = admitted(&["/store/retained.drv"]);
    let _unadmitted_intermediate = admitted(&["/store/old-edit.drv", "/store/retained.drv"]);
    let latest = admitted(&["/store/new-edit.drv", "/store/new-edit.drv", "/store/retained.drv"]);
    let transition = diff(&previous, &latest, "run", 2).unwrap();
    assert_eq!(classified(&transition), vec![
        (WatchEventKind::Added, "/store/new-edit.drv"),
        (WatchEventKind::Retained, "/store/retained.drv"),
    ]);
    assert!(!transition.events.iter().any(|event| event.goal_identity.as_deref() == Some("/store/old-edit.drv")));
}

#[test]
fn input_order_and_duplicate_requests_do_not_change_transition() {
    let left = admitted(&["z", "a", "z", "b"]);
    let right = admitted(&["a", "b", "z"]);
    assert_eq!(left, right);
    assert_eq!(left.identities(), &["a", "b", "z"]);
    let transition = diff(&left, &right, "run", 3).unwrap();
    assert_eq!(classified(&transition).len(), 3);
    assert!(transition.events.iter().all(|event| event.kind == WatchEventKind::Retained));
}

#[test]
fn event_budget_fails_closed_even_for_complete_set_replacement() {
    let previous = admitted(&["a", "b"]);
    let next = admitted(&["c", "d"]);
    assert_eq!(diff(&previous, &next, "run", 3), Err(WatchError::EventBudget));
    assert_eq!(diff(&previous, &previous, "run", 2).unwrap().events.len(), 2);
    assert_eq!(diff(&previous, &next, "run", MAX_TRANSITION_EVENTS + 1), Err(WatchError::EventBudget));
    assert_eq!(reject_generation("run", "bad".to_string(), 0), Err(WatchError::EventBudget));
}

#[test]
fn largest_disjoint_sets_emit_exactly_the_declared_budget() {
    let previous = GoalSet::admit((0..MAX_LIVE_GOALS).map(|index| format!("old-{index:05}")).collect()).unwrap();
    let next = GoalSet::admit((0..MAX_LIVE_GOALS).map(|index| format!("new-{index:05}")).collect()).unwrap();
    assert_eq!(diff(&previous, &next, "run", MAX_TRANSITION_EVENTS - 1), Err(WatchError::EventBudget));
    let result = diff(&previous, &next, "run", MAX_TRANSITION_EVENTS).unwrap();
    assert_eq!(result.events.len(), MAX_TRANSITION_EVENTS as usize);
    assert_eq!(result.events.first().unwrap().kind, WatchEventKind::Added);
    assert_eq!(result.events.last().unwrap().kind, WatchEventKind::Retracted);
}

#[test]
fn goal_count_identity_and_diagnostic_bounds_reject_before_admission() {
    let many = (0..=MAX_LIVE_GOALS).map(|_| "duplicate".to_string()).collect();
    assert_eq!(GoalSet::admit(many), Err(WatchError::GoalLimit));
    assert_eq!(GoalSet::admit(vec![String::new()]), Err(WatchError::EmptyGoalIdentity));
    assert_eq!(
        GoalSet::admit(vec!["x".repeat(MAX_GOAL_IDENTITY_BYTES as usize + 1)]),
        Err(WatchError::GoalIdentityTooLong)
    );
    assert_eq!(diff(&admitted(&[]), &admitted(&[]), "", 0), Err(WatchError::EmptyRunIdentity));
    assert_eq!(
        reject_generation("run", "x".repeat(MAX_DIAGNOSTIC_BYTES as usize + 1), 1),
        Err(WatchError::DiagnosticTooLong)
    );
    assert_eq!(reject_generation("run", String::new(), 1), Err(WatchError::EmptyDiagnostic));
    assert_eq!(
        diff(&admitted(&[]), &admitted(&[]), &"x".repeat(MAX_RUN_IDENTITY_BYTES as usize + 1), 0),
        Err(WatchError::RunIdentityTooLong)
    );
}

#[test]
fn reevaluation_window_limit_rejects_excess_attempts() {
    assert_eq!(admit_evaluation(0), Ok(()));
    assert_eq!(admit_evaluation(MAX_EVALUATIONS_PER_WINDOW - 1), Ok(()));
    assert_eq!(admit_evaluation(MAX_EVALUATIONS_PER_WINDOW), Err(WatchError::EvaluationRate));
}
