//! Behavior boundaries for compact authority; synthetic facts are not process evidence.
use super::super::tests::fixture;
use super::*;
use crate::source_built_fixed_point_planning::plan_actions;

fn compact_fixture() -> (SourceBuiltFixedPointPlan, ActionTrustPlan, Vec<ActionExecutionEvent>) {
    use ActionControllerRole::*;
    let (proof, mut adapters, events) = fixture();
    let base = adapters[0].actions[0].clone();
    let id = format!("{}/controller", base.stage_id);
    let slots = vec!["chosen".into(), "other".into()];
    let mut c = ActionControllerSpec {
        id: id.clone(),
        commands_max: 1,
        slots: slots.clone(),
        templates: Vec::new(),
        guard: None,
    };
    let phases = vec![
        (Start, None),
        (Prepare, Some(slots[0].clone())),
        (Command, Some(slots[0].clone())),
        (Observation, Some(slots[0].clone())),
        (Prepare, Some(slots[1].clone())),
        (Command, Some(slots[1].clone())),
        (Observation, Some(slots[1].clone())),
        (Resume, None),
        (Finish, None),
    ];
    for (role, slot) in phases {
        let mut action = base.clone();
        action.id = "immutable-template".into();
        action.output_ids = vec!["exit-status".into(), "stderr".into(), "stdout".into()];
        action.accepted_exit_codes = vec![0];
        let mut refs = BTreeMap::new();
        if matches!(role, Prepare | Resume | Finish) {
            action.inputs.push(ActionInput::Produced {
                action_id: "previous".into(),
                output_id: "checkpoint".into(),
            });
            refs.insert("previous".into(), ActionControllerRef::PreviousOrEntry);
        }
        match role {
            Start | Resume => {
                action.output_ids.push("checkpoint".into());
                action.accepted_exit_codes = vec![0, 1];
            }
            Prepare => {
                action.output_ids.push("invocation".into());
                action.accepted_exit_codes = vec![0, 1];
            }
            Command => {
                refs.insert("prepare".into(), ActionControllerRef::Current {
                    slot: slot.clone(),
                    role: Prepare,
                    sandbox: false,
                });
                action.inputs.extend([
                    ActionInput::Produced {
                        action_id: "prepare".into(),
                        output_id: "exit-status".into(),
                    },
                    ActionInput::Produced {
                        action_id: "prepare".into(),
                        output_id: "invocation".into(),
                    },
                ]);
                action.guard = Some(ActionGuard {
                    conditions: vec![ActionGuardCondition {
                        action_id: "prepare".into(),
                        accepted_codes: vec![0],
                    }],
                });
            }
            Observation => {
                refs.insert("command".into(), ActionControllerRef::Current {
                    slot: slot.clone(),
                    role: Command,
                    sandbox: false,
                });
                action.inputs.push(ActionInput::PhaseOutcome {
                    action_id: "command".into(),
                });
                action.output_ids = vec!["phase".into()];
                action.accepted_exit_codes.clear();
                action.protected_events_min = 0;
                action.protected_events_max = 0;
            }
            Finish => {
                action.inputs.push(ActionInput::ControllerOutcome {
                    controller_id: id.clone(),
                });
                action.output_ids.push("controller-result".into());
            }
            _ => {}
        }
        if role == Resume {
            for s in &slots {
                let token = format!("observed-{s}");
                refs.insert(token.clone(), ActionControllerRef::Current {
                    slot: Some(s.clone()),
                    role: Observation,
                    sandbox: false,
                });
                action.inputs.push(ActionInput::Produced {
                    action_id: token,
                    output_id: "phase".into(),
                });
            }
        }
        action.output_ids.sort();
        action.producer_actions = action
            .inputs
            .iter()
            .filter_map(|i| match i {
                ActionInput::Produced { action_id, .. } | ActionInput::PhaseOutcome { action_id } => {
                    Some(action_id.clone())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if role != Observation {
            let mut sandbox = action.clone();
            sandbox.accepted_exit_codes.clear();
            sandbox.output_ids = vec!["sandbox-stderr".into(), "sandbox-stdout".into()];
            sandbox.executable = ActionExecutable::Fixed {
                digest_blake3: "f".repeat(64),
            };
            c.templates.push(ActionControllerTemplate {
                role,
                slot: slot.clone(),
                sandbox: true,
                action: sandbox,
                references: refs.clone(),
            });
        }
        c.templates.push(ActionControllerTemplate {
            role,
            slot,
            sandbox: false,
            action,
            references: refs,
        });
    }
    let input = ActionInput::Produced {
        action_id: id.clone(),
        output_id: "controller-result".into(),
    };
    adapters[0].actions[0].inputs.push(input.clone());
    adapters[0].actions[0].producer_actions.push(id);
    adapters[0].controllers.push(c);
    let mut tail = events;
    for e in &mut tail[..2] {
        e.inputs.push(input.clone());
        e.input_digests_blake3.push("b".repeat(64));
    }
    let plan = plan_actions(&proof, adapters).unwrap();
    (proof, plan, tail)
}
fn measured_event(
    proof: &SourceBuiltFixedPointPlan,
    state: &ActionObservationState,
    action: &ActionSpec,
    kind: ExecutionEventKind,
    code: Option<i32>,
) -> ActionExecutionEvent {
    let executable_digest_blake3 = match &action.executable {
        ActionExecutable::Fixed { digest_blake3 } => digest_blake3.clone(),
        _ => unreachable!(),
    };
    let inputs = action.inputs.clone();
    let input_digests_blake3 = inputs
        .iter()
        .map(|i| match i {
            ActionInput::Stage { authority } => stage_input_digest(proof, *authority).unwrap().into(),
            ActionInput::Produced { action_id, output_id } => state.produced[action_id][output_id].clone(),
            ActionInput::PhaseOutcome { action_id } => state.phase_outcomes[action_id].digest().unwrap(),
            ActionInput::ControllerOutcome { controller_id } => {
                state.controller_outcome(controller_id).unwrap().digest().unwrap()
            }
        })
        .collect();
    let produced_outputs = if kind == ExecutionEventKind::BuildExecution {
        action
            .output_ids
            .iter()
            .map(|id| {
                let digest = if id == "exit-status" {
                    let (bytes, n) = canonical_exit_status(code.unwrap()).unwrap();
                    blake3::hash(&bytes[..n]).to_hex().to_string()
                } else {
                    "b".repeat(64)
                };
                (id.clone(), digest)
            })
            .collect()
    } else {
        Vec::new()
    };
    ActionExecutionEvent {
        action_id: action.id.clone(),
        kind,
        normal_exit_code: code,
        executable_digest_blake3,
        executable_producer: None,
        inputs,
        input_digests_blake3,
        produced_outputs,
        locality: "local".into(),
        completed_locally: true,
        cache_hit: false,
        live_fetch: false,
        cargo_invocation: false,
        fallback: false,
        protected_exec_violation: false,
        elapsed_seconds: 1,
        disk_bytes: 1,
    }
}
fn execute_phase(
    proof: &SourceBuiltFixedPointPlan,
    plan: &ActionTrustPlan,
    state: &mut ActionObservationState,
    events: &mut Vec<ActionExecutionEvent>,
    p: ActionControllerPosition,
    code: i32,
) {
    let c = &plan.adapters[0].controllers[0];
    let action = resolve_controller_action(c, &p).unwrap();
    let mut submit = |state: &mut ActionObservationState, e: ActionExecutionEvent| {
        state.record_planned(proof, plan, &e).unwrap();
        events.push(e);
    };
    if p.role == ActionControllerRole::Observation {
        let e = measured_event(proof, state, &action, ExecutionEventKind::BuildExecution, None);
        submit(state, e);
        return;
    }
    let sandbox = resolve_controller_action(c, &ActionControllerPosition {
        sandbox: true,
        ..p.clone()
    })
    .unwrap();
    if let Some(guard) = &action.guard {
        let producer = &guard.conditions[0].action_id;
        let status = state.normal_exit_codes[producer];
        let active = status == 0;
        for a in [&sandbox, &action] {
            let e = ActionExecutionEvent {
                action_id: a.id.clone(),
                kind: ExecutionEventKind::GuardDecision {
                    outcome: if active {
                        GuardOutcome::Active
                    } else {
                        GuardOutcome::Skipped
                    },
                    exit_codes: vec![status],
                },
                normal_exit_code: None,
                executable_digest_blake3: String::new(),
                executable_producer: None,
                inputs: vec![ActionInput::Produced {
                    action_id: producer.clone(),
                    output_id: "exit-status".into(),
                }],
                input_digests_blake3: vec![state.produced[producer]["exit-status"].clone()],
                produced_outputs: Vec::new(),
                locality: "local".into(),
                completed_locally: false,
                cache_hit: false,
                live_fetch: false,
                cargo_invocation: false,
                fallback: false,
                protected_exec_violation: false,
                elapsed_seconds: 0,
                disk_bytes: 0,
            };
            submit(state, e);
        }
        if !active {
            return;
        }
    }
    for (a, kind, normal) in [
        (&sandbox, ExecutionEventKind::ProtectedExec, None),
        (&action, ExecutionEventKind::ProtectedExec, None),
        (&sandbox, ExecutionEventKind::BuildExecution, None),
        (&action, ExecutionEventKind::BuildExecution, Some(code)),
    ] {
        let e = measured_event(proof, state, a, kind, normal);
        submit(state, e);
    }
}
fn observed_fixture(done_at_start: bool) -> (SourceBuiltFixedPointPlan, ActionTrustPlan, Vec<ActionExecutionEvent>) {
    use ActionControllerRole::*;
    let (proof, plan, mut tail) = compact_fixture();
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    execute_phase(&proof, &plan, &mut state, &mut events, position(0, None, Start, false), i32::from(done_at_start));
    if !done_at_start {
        for slot in ["chosen", "other"] {
            execute_phase(
                &proof,
                &plan,
                &mut state,
                &mut events,
                position(0, Some(slot.into()), Prepare, false),
                i32::from(slot == "other"),
            );
            execute_phase(&proof, &plan, &mut state, &mut events, position(0, Some(slot.into()), Command, false), 0);
            execute_phase(
                &proof,
                &plan,
                &mut state,
                &mut events,
                position(0, Some(slot.into()), Observation, false),
                0,
            );
        }
        execute_phase(&proof, &plan, &mut state, &mut events, position(0, None, Resume, false), 1);
    }
    execute_phase(&proof, &plan, &mut state, &mut events, position(u32::from(!done_at_start), None, Finish, false), 0);
    events.append(&mut tail);
    (proof, plan, events)
}
#[test]
fn compact_journal_covers_actual_turn_and_honest_skip_without_potential_completion() {
    let (proof, plan, events) = observed_fixture(false);
    let result = reconcile_actions(&proof, &plan, &events).unwrap();
    assert!(result.action_coverage.iter().any(|c| c.action_id.ends_with("/other/command")
        && c.guard_outcome == Some(GuardOutcome::Skipped)
        && c.build_observed == 0));
    assert!(result.action_coverage.iter().all(|c| !c.action_id.contains("/turn/1/")));
    verify_action_reconciliation(&proof, &plan, &events, &result).unwrap();
    let (proof, plan, events) = observed_fixture(true);
    let result = reconcile_actions(&proof, &plan, &events).unwrap();
    assert!(result.action_coverage.iter().all(|c| !c.action_id.contains("/turn/")));
}
#[test]
fn compact_journal_rejects_repetition_gaps_premature_finish_and_unfinished_roots() {
    let (proof, plan, events) = observed_fixture(false);
    for variant in 0..5 {
        let mut bad = events.clone();
        match variant {
            0 => {
                bad.insert(0, bad[0].clone());
            }
            1 => {
                bad.remove(3);
            }
            2 => {
                let index = bad.iter().position(|e| e.action_id.ends_with("/turn/0/chosen/prepare.sandbox")).unwrap();
                bad[index].action_id = bad[index].action_id.replace("/turn/0/", "/turn/1/");
            }
            3 => {
                let finish = bad.iter().find(|e| e.action_id == plan.adapters[0].controllers[0].id).unwrap().clone();
                bad.insert(4, finish);
            }
            _ => {
                bad.retain(|e| e.action_id != plan.adapters[0].controllers[0].id);
            }
        }
        let error = reconcile_actions(&proof, &plan, &bad).unwrap_err();
        let mut live = ActionObservationState::default();
        let live_error = bad.iter().find_map(|e| live.record_planned(&proof, &plan, e).err()).unwrap();
        assert_eq!(error, live_error, "variant {variant}");
        assert!(live.record_planned(&proof, &plan, &events[0]).unwrap_err().contains("already failed"));
    }
}
#[test]
fn compact_journal_binds_previous_checkpoint_done_outcome_and_real_image_hashes() {
    let (proof, plan, events) = observed_fixture(false);
    for variant in 0..4 {
        let mut bad = events.clone();
        let event = match variant {
            0 => bad
                .iter_mut()
                .find(|e| e.action_id.ends_with("/chosen/prepare") && e.kind == ExecutionEventKind::ProtectedExec)
                .unwrap(),
            1 => bad
                .iter_mut()
                .find(|e| {
                    e.action_id == plan.adapters[0].controllers[0].id && e.kind == ExecutionEventKind::ProtectedExec
                })
                .unwrap(),
            2 => bad
                .iter_mut()
                .find(|e| e.action_id.ends_with("/chosen/command") && e.kind == ExecutionEventKind::ProtectedExec)
                .unwrap(),
            _ => bad
                .iter_mut()
                .find(|e| e.action_id.ends_with("/resume") && e.kind == ExecutionEventKind::BuildExecution)
                .unwrap(),
        };
        match variant {
            0 => {
                let n = event
                    .inputs
                    .iter()
                    .position(|i| matches!(i,ActionInput::Produced{output_id,..}if output_id=="checkpoint"))
                    .unwrap();
                event.input_digests_blake3[n] = "c".repeat(64);
            }
            1 => {
                let n = event.inputs.iter().position(|i| matches!(i, ActionInput::ControllerOutcome { .. })).unwrap();
                event.input_digests_blake3[n] = "c".repeat(64);
            }
            2 => event.executable_digest_blake3 = "c".repeat(64),
            _ => {
                event.normal_exit_code = Some(0);
                let (bytes, n) = canonical_exit_status(0).unwrap();
                event.produced_outputs.iter_mut().find(|(id, _)| id == "exit-status").unwrap().1 =
                    blake3::hash(&bytes[..n]).to_hex().to_string();
            }
        }
        assert!(reconcile_actions(&proof, &plan, &bad).is_err(), "variant {variant}");
    }
}
#[test]
fn compact_catalog_rejects_static_shadowing_overflow_and_unknown_slot() {
    let (proof, mut plan, _) = compact_fixture();
    let c = &mut plan.adapters[0].controllers[0];
    c.commands_max = u32::MAX;
    assert!(validate_controller_shape(c, &proof.stages[0].stage_id).is_err());
    let (proof, mut plan, _) = compact_fixture();
    let c = &plan.adapters[0].controllers[0];
    assert!(
        resolve_controller_action(c, &position(0, Some("forged".into()), ActionControllerRole::Prepare, false))
            .is_err()
    );
    let id = format!("{}/start", c.id);
    plan.adapters[0].actions[0].id = id.clone();
    plan.adapters[0].reachable_action_ids = vec![id];
    plan.digest_blake3 = action_plan_digest(&plan).unwrap();
    assert!(validate_action_plan(&proof, &plan).is_err());
}

fn guarded_compact_fixture(active: bool) -> (SourceBuiltFixedPointPlan, ActionTrustPlan, Vec<ActionExecutionEvent>) {
    let (proof, mut plan, mut tail) = compact_fixture();
    let root = plan.adapters[0].controllers[0].id.clone();
    let mut status = plan.adapters[0].actions[0].clone();
    status.id = format!("{}/outer-status", status.stage_id);
    status.inputs.retain(|i| !matches!(i,ActionInput::Produced{action_id,..}if action_id==&root));
    status.producer_actions.clear();
    status.output_ids = vec!["exit-status".into(), "stderr".into(), "stdout".into()];
    status.accepted_exit_codes = vec![0, 1];
    let guard = ActionGuard {
        conditions: vec![ActionGuardCondition {
            action_id: status.id.clone(),
            accepted_codes: vec![0],
        }],
    };
    plan.adapters[0].controllers[0].guard = Some(guard);
    let product = &mut plan.adapters[0].actions[0];
    *product.inputs.last_mut().unwrap() = ActionInput::PhaseOutcome {
        action_id: root.clone(),
    };
    plan.adapters[0].reachable_action_ids.push(status.id.clone());
    plan.adapters[0].reachable_action_ids.sort();
    plan.adapters[0].actions.insert(0, status.clone());
    plan = plan_actions(&proof, plan.adapters).unwrap();
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    for (kind, code) in [
        (ExecutionEventKind::ProtectedExec, None),
        (ExecutionEventKind::BuildExecution, Some(i32::from(!active))),
    ] {
        let e = measured_event(&proof, &state, &status, kind, code);
        state.record_planned(&proof, &plan, &e).unwrap();
        events.push(e);
    }
    let control = controller_guard_action(&plan.adapters[0].controllers[0]).unwrap();
    let producer = &status.id;
    let e = ActionExecutionEvent {
        action_id: root.clone(),
        kind: ExecutionEventKind::GuardDecision {
            outcome: if active {
                GuardOutcome::Active
            } else {
                GuardOutcome::Skipped
            },
            exit_codes: vec![i32::from(!active)],
        },
        normal_exit_code: None,
        executable_digest_blake3: String::new(),
        executable_producer: None,
        inputs: control.inputs,
        input_digests_blake3: vec![state.produced[producer]["exit-status"].clone()],
        produced_outputs: Vec::new(),
        locality: "local".into(),
        completed_locally: false,
        cache_hit: false,
        live_fetch: false,
        cargo_invocation: false,
        fallback: false,
        protected_exec_violation: false,
        elapsed_seconds: 0,
        disk_bytes: 0,
    };
    state.record_planned(&proof, &plan, &e).unwrap();
    events.push(e);
    if active {
        execute_phase(&proof, &plan, &mut state, &mut events, position(0, None, ActionControllerRole::Start, false), 1);
        execute_phase(
            &proof,
            &plan,
            &mut state,
            &mut events,
            position(0, None, ActionControllerRole::Finish, false),
            0,
        );
    }
    let digest = state.phase_outcomes[&root].digest().unwrap();
    for e in &mut tail[..2] {
        *e.inputs.last_mut().unwrap() = ActionInput::PhaseOutcome {
            action_id: root.clone(),
        };
        *e.input_digests_blake3.last_mut().unwrap() = digest.clone();
    }
    events.append(&mut tail);
    (proof, plan, events)
}

#[test]
fn outer_controller_guard_skips_without_finish_authority_and_active_runs_real_finish() {
    for active in [false, true] {
        let (proof, plan, events) = guarded_compact_fixture(active);
        let result = reconcile_actions(&proof, &plan, &events).unwrap();
        let root = &plan.adapters[0].controllers[0].id;
        let coverage = result.action_coverage.iter().find(|c| &c.action_id == root).unwrap();
        assert_eq!(
            coverage.guard_outcome,
            Some(if active {
                GuardOutcome::Active
            } else {
                GuardOutcome::Skipped
            })
        );
        assert_eq!(coverage.build_observed, u32::from(active));
        if !active {
            assert!(!events.iter().any(|e| e.action_id.starts_with(&format!("{root}/"))));
        }
        let mut bad = events.clone();
        bad[2].input_digests_blake3[0] = "c".repeat(64);
        assert!(reconcile_actions(&proof, &plan, &bad).is_err());
        let mut missing = events;
        missing.remove(2);
        assert!(reconcile_actions(&proof, &plan, &missing).is_err());
    }
}

fn replace_actual_status(event: &mut ActionExecutionEvent, code: i32) {
    event.normal_exit_code = Some(code);
    let (bytes, n) = canonical_exit_status(code).unwrap();
    event.produced_outputs.iter_mut().find(|(id, _)| id == "exit-status").unwrap().1 =
        blake3::hash(&bytes[..n]).to_hex().to_string();
}
#[test]
fn controller_requires_exactly_one_selected_original_command_per_active_turn() {
    let (proof, plan, events) = observed_fixture(false);
    let mut double = events.clone();
    let event = double
        .iter_mut()
        .find(|e| e.action_id.ends_with("/other/prepare") && e.kind == ExecutionEventKind::BuildExecution)
        .unwrap();
    replace_actual_status(event, 0);
    assert!(reconcile_actions(&proof, &plan, &double).unwrap_err().contains("selected twice"));
    let mut missing = events;
    let (bytes, n) = canonical_exit_status(1).unwrap();
    let hash = blake3::hash(&bytes[..n]).to_hex().to_string();
    for event in &mut missing {
        if event.action_id.ends_with("/chosen/prepare") && event.kind == ExecutionEventKind::BuildExecution {
            replace_actual_status(event, 1);
        }
        let selected =
            event.action_id.ends_with("/chosen/command") || event.action_id.ends_with("/chosen/command.sandbox");
        if selected && let ExecutionEventKind::GuardDecision { outcome, exit_codes } = &mut event.kind {
            *outcome = GuardOutcome::Skipped;
            *exit_codes = vec![1];
            event.input_digests_blake3[0] = hash.clone();
        }
        if event.action_id.ends_with("/chosen/observation") {
            let index = event.inputs.iter().position(|i| matches!(i, ActionInput::PhaseOutcome { .. })).unwrap();
            event.input_digests_blake3[index] = PhaseOutcome::Skipped { guard_codes: vec![1] }.digest().unwrap();
        }
    }
    missing.retain(|e| {
        !(e.action_id.ends_with("/chosen/command") || e.action_id.ends_with("/chosen/command.sandbox"))
            || matches!(e.kind, ExecutionEventKind::GuardDecision { .. })
    });
    assert!(reconcile_actions(&proof, &plan, &missing).unwrap_err().contains("no selected command"));
}
#[test]
fn selected_command_preserves_explicit_normal255_opt_in_and_actual_phase_status() {
    let (proof, mut plan, mut events) = observed_fixture(false);
    for template in &mut plan.adapters[0].controllers[0].templates {
        if template.role == ActionControllerRole::Command
            && template.slot.as_deref() == Some("chosen")
            && !template.sandbox
        {
            template.action.accepted_exit_codes = (0..=255).collect();
        }
    }
    plan.digest_blake3 = action_plan_digest(&plan).unwrap();
    let command = events
        .iter_mut()
        .find(|e| e.action_id.ends_with("/chosen/command") && e.kind == ExecutionEventKind::BuildExecution)
        .unwrap();
    replace_actual_status(command, 255);
    let outcome = PhaseOutcome::Executed {
        output_digests: command.produced_outputs.iter().cloned().collect(),
    }
    .digest()
    .unwrap();
    let observation = events.iter_mut().find(|e| e.action_id.ends_with("/chosen/observation")).unwrap();
    let index = observation.inputs.iter().position(|i| matches!(i, ActionInput::PhaseOutcome { .. })).unwrap();
    observation.input_digests_blake3[index] = outcome;
    reconcile_actions(&proof, &plan, &events).unwrap();
    for template in &mut plan.adapters[0].controllers[0].templates {
        if template.role == ActionControllerRole::Command
            && template.slot.as_deref() == Some("chosen")
            && !template.sandbox
        {
            template.action.accepted_exit_codes = vec![0];
        }
    }
    plan.digest_blake3 = action_plan_digest(&plan).unwrap();
    assert!(reconcile_actions(&proof, &plan, &events).unwrap_err().contains("normal exit"));
}

#[test]
fn actual_turns_have_unique_ids_and_bind_previous_producer_even_for_identical_bytes() {
    use ActionControllerRole::*;
    let (proof, mut plan, mut tail) = compact_fixture();
    let c = &mut plan.adapters[0].controllers[0];
    c.commands_max = 2;
    c.slots.truncate(1);
    c.templates.retain(|t| t.slot.as_deref() != Some("other"));
    for t in &mut c.templates {
        t.action
            .inputs
            .retain(|i| !matches!(i,ActionInput::Produced{action_id,..}if action_id=="observed-other"));
        t.action.producer_actions.retain(|id| id != "observed-other");
    }
    plan = plan_actions(&proof, plan.adapters).unwrap();
    let mut state = ActionObservationState::default();
    let mut events = Vec::new();
    execute_phase(&proof, &plan, &mut state, &mut events, position(0, None, Start, false), 0);
    for turn in 0..2 {
        execute_phase(&proof, &plan, &mut state, &mut events, position(turn, Some("chosen".into()), Prepare, false), 0);
        execute_phase(&proof, &plan, &mut state, &mut events, position(turn, Some("chosen".into()), Command, false), 0);
        execute_phase(
            &proof,
            &plan,
            &mut state,
            &mut events,
            position(turn, Some("chosen".into()), Observation, false),
            0,
        );
        execute_phase(
            &proof,
            &plan,
            &mut state,
            &mut events,
            position(turn, None, Resume, false),
            i32::from(turn == 1),
        );
    }
    assert_eq!(state.controller_outcome(&plan.adapters[0].controllers[0].id).unwrap().completed_turns, 2);
    execute_phase(&proof, &plan, &mut state, &mut events, position(2, None, Finish, false), 0);
    events.append(&mut tail);
    let result = reconcile_actions(&proof, &plan, &events).unwrap();
    let unique = result.action_coverage.iter().map(|row| row.action_id.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), result.action_coverage.len());
    assert!(unique.iter().any(|id| id.contains("/turn/0/")) && unique.iter().any(|id| id.contains("/turn/1/")));
    let event = events
        .iter_mut()
        .find(|e| e.action_id.ends_with("/turn/1/chosen/prepare") && e.kind == ExecutionEventKind::ProtectedExec)
        .unwrap();
    let input = event
        .inputs
        .iter_mut()
        .find(|i| matches!(i,ActionInput::Produced{output_id,..}if output_id=="checkpoint"))
        .unwrap();
    if let ActionInput::Produced { action_id, .. } = input {
        *action_id = format!("{}/start", plan.adapters[0].controllers[0].id);
    }
    assert!(reconcile_actions(&proof, &plan, &events).is_err());
}

#[test]
fn large_conditional_catalog_accepts_actual_done_early_without_reserving_unexecuted_turns() {
    let (proof, mut plan, events) = observed_fixture(true);
    plan.adapters[0].controllers[0].commands_max =
        crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_EXEC_EVENT_COUNT_MAX;
    let potential = validate_controller_shape(&plan.adapters[0].controllers[0], &proof.stages[0].stage_id).unwrap();
    assert!(potential > u64::from(proof.resource_bounds.protected_exec_events_max));
    let plan = plan_actions(&proof, plan.adapters).unwrap();
    let result = reconcile_actions(&proof, &plan, &events).unwrap();
    assert!(result.action_coverage.iter().all(|row| !row.action_id.contains("/turn/")));
}

#[test]
fn controller_potential_events_charge_each_turn_phase_once_per_admitted_turn() {
    let (proof, plan, _) = compact_fixture();
    let stage = &proof.stages[0].stage_id;
    let mut controller = plan.adapters[0].controllers[0].clone();
    let turn_phase_events = controller
        .templates
        .iter()
        .filter(|t| !matches!(t.role, ActionControllerRole::Start | ActionControllerRole::Finish))
        .map(|t| {
            u64::from(t.action.protected_events_max)
                + u64::from(t.action.build_events_max)
                + u64::from(t.action.guard.is_some())
        })
        .sum::<u64>();
    let potential = validate_controller_shape(&controller, stage).unwrap();
    controller.commands_max += 1;
    let grown = validate_controller_shape(&controller, stage).unwrap();
    assert!(turn_phase_events > 0);
    assert_eq!(grown - potential, turn_phase_events);
}

#[test]
fn actual_event_budget_exhaustion_latches_and_matches_portable_replay() {
    use ActionControllerRole::*;
    let (mut proof, mut plan, mut tail) = compact_fixture();
    proof.resource_bounds.protected_exec_events_max = 128;
    proof.plan_digest_blake3 = crate::source_built_fixed_point::plan_digest_blake3(&proof).unwrap();
    let c = &mut plan.adapters[0].controllers[0];
    c.commands_max = crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_EXEC_EVENT_COUNT_MAX;
    c.slots.truncate(1);
    c.templates.retain(|t| t.slot.as_deref() != Some("other"));
    for t in &mut c.templates {
        t.action
            .inputs
            .retain(|i| !matches!(i,ActionInput::Produced{action_id,..}if action_id=="observed-other"));
        t.action.producer_actions.retain(|id| id != "observed-other");
    }
    let plan = plan_actions(&proof, plan.adapters).unwrap();
    let mut generator = ActionObservationState::default();
    let mut events = Vec::new();
    execute_phase(&proof, &plan, &mut generator, &mut events, position(0, None, Start, false), 0);
    for turn in 0..4 {
        execute_phase(
            &proof,
            &plan,
            &mut generator,
            &mut events,
            position(turn, Some("chosen".into()), Prepare, false),
            0,
        );
        execute_phase(
            &proof,
            &plan,
            &mut generator,
            &mut events,
            position(turn, Some("chosen".into()), Command, false),
            0,
        );
        execute_phase(
            &proof,
            &plan,
            &mut generator,
            &mut events,
            position(turn, Some("chosen".into()), Observation, false),
            0,
        );
        execute_phase(
            &proof,
            &plan,
            &mut generator,
            &mut events,
            position(turn, None, Resume, false),
            i32::from(turn == 3),
        );
    }
    execute_phase(&proof, &plan, &mut generator, &mut events, position(4, None, Finish, false), 0);
    events.append(&mut tail);
    reconcile_actions(&proof, &plan, &events).unwrap();
    proof.resource_bounds.protected_exec_events_max = 64;
    proof.plan_digest_blake3 = crate::source_built_fixed_point::plan_digest_blake3(&proof).unwrap();
    let plan = plan_actions(&proof, plan.adapters).unwrap();
    let mut live = ActionObservationState::default();
    for event in &events[..64] {
        live.record_planned(&proof, &plan, event).unwrap();
    }
    let error = live.record_planned(&proof, &plan, &events[64]).unwrap_err();
    assert!(error.contains("execution event count exceeds proof bound"));
    assert!(live.record_planned(&proof, &plan, &events[0]).unwrap_err().contains("already failed"));
    assert_eq!(reconcile_actions(&proof, &plan, &events).unwrap_err(), error);
    assert!(reconcile_actions(&proof, &plan, &events[..64]).unwrap_err().contains("incomplete controller"));
}
