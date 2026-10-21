//! Actual bounded source continuation through the common admitted step runner.
use super::*;
use crate::source_built_fixed_point_action_trust::ActionControllerOutcome;
use crate::source_built_fixed_point_action_trust::ActionControllerPosition;
use crate::source_built_fixed_point_action_trust::ActionControllerRole;
use crate::source_built_fixed_point_action_trust::ActionControllerSpec;
use crate::source_built_fixed_point_action_trust::resolve_controller_action;
use crate::source_built_recipe::controller::StepTemplate;
use crate::source_built_recipe::controller::instantiate;
use crate::source_built_recipe::controller::step_templates;
use crate::source_built_stagex_programs::controller::ControllerSpec;

fn execute(
    execution: &mut RecipeExecution<'_>,
    controller: &ActionControllerSpec,
    template: &StepTemplate,
    turn: u32,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<String, RunError> {
    let position = ActionControllerPosition {
        turn,
        slot: template.position.slot.clone(),
        role: template.position.role,
        sandbox: false,
    };
    let action = resolve_controller_action(controller, &position).map_err(error)?;
    let step = instantiate(template, controller, &position)?;
    for (output_id, output) in &step.outputs {
        let reference = ArtifactRef::Produced {
            action_id: action.id.clone(),
            output_id: output_id.clone(),
        };
        let binding = ArtifactBinding {
            reference: reference.clone(),
            path: execution.prepared.runtime_root.join(&step.id).join("outputs").join(output_id),
            kind: output.kind,
            bytes_max: output.bytes_max,
        };
        if execution.outputs.contains_key(&reference) || execution.dynamic_bindings.contains_key(&reference) {
            return Err(error("controller output identity repeated"));
        }
        if execution.prepared.bindings.get(&reference).is_some_and(|old| old != &binding) {
            return Err(error("controller output binding differs from immutable root"));
        }
        execution.dynamic_bindings.insert(reference, binding);
    }
    let sandbox = if matches!(step.operation, RecipeOperation::Exec { .. }) {
        Some(
            resolve_controller_action(controller, &ActionControllerPosition {
                sandbox: true,
                ..position
            })
            .map_err(error)?,
        )
    } else {
        None
    };
    let mut actions = BTreeMap::from([(action.id.as_str(), &action)]);
    if let Some(sandbox) = &sandbox {
        actions.insert(sandbox.id.as_str(), sandbox);
    }
    execution.step(&step, &actions, emit)?;
    Ok(action.id)
}
fn status(execution: &RecipeExecution<'_>, id: &str) -> Result<i32, RunError> {
    let completion = execution
        .completions
        .last()
        .filter(|completion| completion.action_id == id)
        .ok_or_else(|| error("source control lacks actual normal completion"))?;
    if ![0, 1].contains(&completion.exit_code) {
        return Err(error("invalid actual controller pause/done status"));
    }
    Ok(completion.exit_code)
}
fn outcome(
    execution: &mut RecipeExecution<'_>,
    controller: &ActionControllerSpec,
    id: String,
    turns: u32,
) -> Result<(), RunError> {
    if status(execution, &id)? != 1 {
        return Err(error("source controller finish attempted without genuine done"));
    }
    let Some(PhaseOutcome::Executed { output_digests }) = execution.phase_outcomes.get(&id) else {
        return Err(error("controller done lacks measured outputs"));
    };
    let value = ActionControllerOutcome {
        controller_id: controller.id.clone(),
        completed_turns: turns,
        producer_action: id,
        output_digests: output_digests.clone(),
    };
    if execution.controller_outcomes.insert(controller.id.clone(), value).is_some() {
        return Err(error("controller done outcome repeated"));
    }
    execution.source_scopes.finish();
    Ok(())
}
pub(super) fn run(
    execution: &mut RecipeExecution<'_>,
    step: &RecipeStep,
    s: &ControllerSpec,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<(), RunError> {
    use ActionControllerRole::*;
    let prepared = execution.prepared;
    let stage = &prepared.recipe.stage_id;
    let root = format!("{stage}/{}", s.id);
    let controller = prepared
        .adapter
        .controllers
        .iter()
        .find(|controller| controller.id == root)
        .ok_or_else(|| error("missing admitted compact source controller"))?;
    let bindings = BindingOverlay {
        base: &prepared.bindings,
        dynamic: Some(&execution.dynamic_bindings),
    };
    if let Some((outcome, codes, inputs, hashes)) = guard_decision(&bindings, step, &execution.outputs)? {
        let action =
            crate::source_built_fixed_point_action_trust::controller_guard_action(controller).map_err(error)?;
        let mut decision = event(
            &action,
            ExecutionEventKind::GuardDecision {
                outcome,
                exit_codes: codes.clone(),
            },
            String::new(),
            hashes,
            Vec::new(),
            false,
            false,
            0,
            0,
        );
        decision.executable_producer = None;
        decision.inputs = inputs;
        journal(&mut execution.raw, &decision)?;
        emit(decision)?;
        if outcome == GuardOutcome::Skipped {
            execution.phase_outcomes.insert(root, PhaseOutcome::Skipped { guard_codes: codes });
            return Ok(());
        }
    }
    let templates = step_templates(stage, s)?;
    let start = templates
        .iter()
        .find(|template| template.position.role == Start)
        .ok_or_else(|| error("missing controller entry template"))?;
    let started = execute(execution, controller, start, 0, emit)?;
    let mut done = status(execution, &started)? == 1;
    let mut turns = 0u32;
    let mut producer = started;
    while !done {
        if turns >= controller.commands_max {
            return Err(error("original source controller command bound exhausted before done"));
        }
        let mut selected = 0u32;
        for template in templates.iter().filter(|template| !matches!(template.position.role, Start | Finish)) {
            if template.position.role == Resume && selected != 1 {
                return Err(error("active source controller turn has no unique selected slot"));
            }
            let id = execute(execution, controller, template, turns, emit)?;
            if template.position.role == Prepare && status(execution, &id)? == 0 {
                selected = selected.checked_add(1).ok_or_else(|| error("controller selection overflow"))?;
                if selected > 1 {
                    return Err(error("source controller selected multiple immutable slots"));
                }
            }
            if template.position.role == Resume {
                done = status(execution, &id)? == 1;
                producer = id;
            }
        }
        turns = turns.checked_add(1).ok_or_else(|| error("source controller ordinal overflow"))?;
    }
    outcome(execution, controller, producer, turns)?;
    let finish = templates
        .iter()
        .find(|template| template.position.role == Finish)
        .ok_or_else(|| error("missing controller finish template"))?;
    execute(execution, controller, finish, turns, emit)?;
    Ok(())
}
