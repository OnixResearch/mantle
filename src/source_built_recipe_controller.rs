//! Compact immutable source-controller declaration and exact template binding.
use super::*;
use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_EXEC_EVENT_COUNT_MAX;
use crate::source_built_fixed_point_action_trust::ActionControllerPosition;
use crate::source_built_fixed_point_action_trust::ActionControllerRef;
use crate::source_built_fixed_point_action_trust::ActionControllerRole;
use crate::source_built_fixed_point_action_trust::ActionControllerSpec;
use crate::source_built_fixed_point_action_trust::ActionControllerTemplate;
use crate::source_built_stagex_programs::controller::ControllerExecutable;
use crate::source_built_stagex_programs::controller::ControllerResult;
use crate::source_built_stagex_programs::controller::ControllerSpec;
use crate::source_built_stagex_programs::controller::template_recipe;

pub(crate) const CONTROLLER_COMMANDS_MAX: u32 = SOURCE_BUILT_FIXED_POINT_EXEC_EVENT_COUNT_MAX;

pub(crate) fn reserve_native_commands(
    policy: &crate::source_built_program_authoring::ContinuationReservationPolicy,
    phase_id: &str,
    source_files: u32,
    remaining_steps: usize,
) -> Result<usize, RunError> {
    if remaining_steps == 0 {
        return Err(error("compact source controller has no remaining declaration slot"));
    }
    policy.native_slots(phase_id, source_files, CONTROLLER_COMMANDS_MAX)
}

fn outputs(s: &ControllerSpec) -> BTreeMap<String, RecipeOutput> {
    [
        ("checkpoint", ArtifactKind::Tree, s.tree_bytes_max),
        ("workspace", ArtifactKind::Tree, s.tree_bytes_max),
        ("times", ArtifactKind::Text, s.file_bytes_max),
        ("generated", ArtifactKind::Tree, s.tree_bytes_max),
        ("stdout", ArtifactKind::Object, s.file_bytes_max),
        ("stderr", ArtifactKind::Text, s.file_bytes_max),
        ("exit-status", ArtifactKind::Text, 4),
    ]
    .into_iter()
    .map(|(name, kind, bytes_max)| (name.into(), RecipeOutput { kind, bytes_max }))
    .collect()
}
fn inputs(s: &ControllerSpec) -> BTreeMap<String, ArtifactRef> {
    let mut references = BTreeSet::new();
    references.insert(s.controller.clone());
    references.extend(s.inputs.values().cloned());
    for slot in &s.slots {
        references.extend(slot.inputs.values().cloned());
        match &slot.executable {
            ControllerExecutable::Fixed(image) => {
                references.insert(image.clone());
            }
            ControllerExecutable::Adapted {
                executable,
                original_image,
            } => {
                references.insert(executable.clone());
                references.insert(original_image.clone());
            }
            ControllerExecutable::Inventory(images) => references.extend(images.iter().map(|(_, image)| image.clone())),
            ControllerExecutable::RequestOutput { .. } => {}
        }
    }
    references
        .into_iter()
        .enumerate()
        .map(|(i, reference)| (format!("controller-input-{i}"), reference))
        .collect()
}

pub(crate) fn validate(step: &RecipeStep, s: &ControllerSpec) -> Result<(), RunError> {
    label(&s.id)?;
    let mut expected_inputs = inputs(s);
    if let Some(guard) = operation_guard(&step.operation) {
        lower_guard(step)?;
        for condition in &guard.conditions {
            let reference = step
                .inputs
                .get(&condition.status_input)
                .ok_or_else(|| error("controller root guard lacks actual status input"))?;
            if expected_inputs.insert(condition.status_input.clone(), reference.clone()).is_some() {
                return Err(error("controller guard input collides with immutable catalog input"));
            }
        }
    }
    if step.id != s.id
        || step.inputs != expected_inputs
        || step.outputs != outputs(s)
        || s.commands_max == 0
        || s.commands_max > CONTROLLER_COMMANDS_MAX as usize
    {
        return Err(error("invalid compact immutable source controller declaration"));
    }
    s.exit_policy.validate()?;
    let templates = template_recipe("controller-validation", s, None)?;
    for template in &templates.steps {
        validate_operation(template)?;
    }
    Ok(())
}

pub(crate) fn append(recipe: &mut SourceRecipe, s: ControllerSpec) -> Result<ControllerResult, RunError> {
    append_guarded(recipe, s, Vec::new())
}
pub(crate) fn append_guarded(
    recipe: &mut SourceRecipe,
    s: ControllerSpec,
    predicates: Vec<(ArtifactRef, Vec<i32>)>,
) -> Result<ControllerResult, RunError> {
    let mut inputs = inputs(&s);
    let mut conditions = Vec::with_capacity(predicates.len());
    for (index, (reference, accepted_codes)) in predicates.into_iter().enumerate() {
        let status_input = format!("controller-guard-{index}");
        inputs.insert(status_input.clone(), reference);
        conditions.push(RecipeGuardCondition {
            status_input,
            accepted_codes,
        });
    }
    let guard = if conditions.is_empty() {
        None
    } else {
        Some(RecipeGuard { conditions })
    };
    let step = RecipeStep {
        id: s.id.clone(),
        inputs,
        outputs: outputs(&s),
        operation: RecipeOperation::Controller {
            specification: s,
            guard,
        },
    };
    let RecipeOperation::Controller { specification, .. } = &step.operation else {
        unreachable!()
    };
    validate(&step, specification)?;
    if recipe.steps.len() >= STEP_COUNT_MAX || recipe.steps.iter().any(|old| old.id == step.id) {
        return Err(error("duplicate or excessive compact source controller"));
    }
    let reference = |output: &str| ArtifactRef::Produced {
        action_id: action_id(&recipe.stage_id, &step.id),
        output_id: output.into(),
    };
    let result = ControllerResult {
        checkpoint: reference("checkpoint"),
        workspace: reference("workspace"),
        times: reference("times"),
        generated: reference("generated"),
        status: reference("exit-status"),
    };
    recipe.steps.push(step);
    Ok(result)
}

pub(crate) struct StepTemplate {
    pub position: ActionControllerPosition,
    pub step: RecipeStep,
    pub references: BTreeMap<String, ActionControllerRef>,
}

pub(crate) fn step_templates(stage: &str, s: &ControllerSpec) -> Result<Vec<StepTemplate>, RunError> {
    use ActionControllerRole::*;
    // '@' is forbidden in admitted source step labels, so no immutable source
    // producer can collide with these structural template reference tokens.
    let template_id = format!("@controller-{}", s.id);
    let recipe = template_recipe(stage, s, Some(&template_id))?;
    let root = action_id(stage, &template_id);
    let mut roles = BTreeMap::new();
    roles.insert(format!("{root}-start"), (Start, None));
    roles.insert(format!("{root}-000000-resume"), (Resume, None));
    roles.insert(format!("{root}-finish"), (Finish, None));
    for slot in &s.slots {
        for (suffix, role) in [
            ("prepare", Prepare),
            ("executable", Executable),
            ("command", Command),
            ("observation", Observation),
        ] {
            roles.insert(format!("{root}-000000-{}-{suffix}", slot.id), (role, Some(slot.id.clone())));
        }
    }
    let mut result = Vec::with_capacity(recipe.steps.len());
    for step in recipe.steps {
        let (role, slot) = roles
            .get(&action_id(stage, &step.id))
            .cloned()
            .ok_or_else(|| error("unknown source controller template role"))?;
        let mut references = BTreeMap::new();
        for (token, (target_role, target_slot)) in &roles {
            let target = if *target_role == Start {
                ActionControllerRef::PreviousOrEntry
            } else {
                ActionControllerRef::Current {
                    slot: target_slot.clone(),
                    role: *target_role,
                    sandbox: false,
                }
            };
            references.insert(token.clone(), target);
        }
        for reference in step.inputs.values() {
            if let ArtifactRef::Produced { action_id, .. } = reference {
                references.entry(action_id.clone()).or_insert_with(|| ActionControllerRef::Static {
                    action_id: action_id.clone(),
                });
            }
        }
        result.push(StepTemplate {
            position: ActionControllerPosition {
                turn: 0,
                slot,
                role,
                sandbox: false,
            },
            step,
            references,
        });
    }
    Ok(result)
}

pub(crate) fn instantiate(
    template: &StepTemplate,
    controller: &ActionControllerSpec,
    position: &ActionControllerPosition,
) -> Result<RecipeStep, RunError> {
    let resolved =
        crate::source_built_fixed_point_action_trust::resolve_controller_action(controller, position).map_err(error)?;
    let mut step = template.step.clone();
    step.id = resolved
        .id
        .strip_prefix(&format!("{}/", resolved.stage_id))
        .ok_or_else(|| error("cross-stage controller action"))?
        .into();
    let resolve = |token: &str| -> Result<String, RunError> {
        let Some(reference) = template.references.get(token) else {
            return Ok(token.into());
        };
        crate::source_built_fixed_point_action_trust::resolve_controller_reference(controller, position, reference)
            .map_err(error)
    };
    for reference in step.inputs.values_mut() {
        if let ArtifactRef::Produced { action_id, .. } = reference {
            *action_id = resolve(action_id)?;
        }
    }
    if let RecipeOperation::ObservePhase { phase, .. } = &mut step.operation {
        *phase = resolve(&action_id(&resolved.stage_id, phase))?
            .strip_prefix(&format!("{}/", resolved.stage_id))
            .ok_or_else(|| error("cross-stage observed controller phase"))?
            .into();
    }
    Ok(step)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower(
    proof: &SourceBuiltFixedPointPlan,
    stage: &str,
    stage_inputs: &[StageAuthorityInput],
    prior: &[&PreparedRecipe],
    paths: &RecipePaths<'_>,
    cache: &Path,
    host: &str,
    sandbox: &str,
    source_inputs: &[ActionInput],
    stage_outputs: &[ArtifactRef],
    s: &ControllerSpec,
    bindings: &mut BTreeMap<ArtifactRef, ArtifactBinding>,
) -> Result<ActionControllerSpec, RunError> {
    let templates = step_templates(stage, s)?;
    let mut local = MutableBindingOverlay {
        base: bindings,
        dynamic: BTreeMap::new(),
    };
    let mut actions = Vec::new();
    for template in &templates {
        let lowered = lower_step(
            proof,
            stage,
            stage_inputs,
            prior,
            paths,
            cache,
            host,
            sandbox,
            source_inputs,
            stage_outputs,
            &template.step,
            &mut local,
        )?;
        for mut action in lowered {
            let sandbox = action.id.ends_with(".sandbox");
            if sandbox {
                action.output_ids = vec!["sandbox-stderr".into(), "sandbox-stdout".into()];
            }
            if template.position.role == ActionControllerRole::Finish {
                action.inputs.push(ActionInput::ControllerOutcome {
                    controller_id: action_id(stage, &s.id),
                });
            }
            let mut references = template.references.clone();
            for id in action.producer_actions.iter().chain(action.inputs.iter().filter_map(|input| match input {
                ActionInput::Produced { action_id, .. } | ActionInput::PhaseOutcome { action_id } => Some(action_id),
                _ => None,
            })) {
                references
                    .entry(id.clone())
                    .or_insert_with(|| ActionControllerRef::Static { action_id: id.clone() });
            }
            actions.push(ActionControllerTemplate {
                role: template.position.role,
                slot: template.position.slot.clone(),
                sandbox,
                action,
                references,
            });
        }
    }
    // Only immutable imported/source bindings survive preparation; turn outputs
    // are allocated individually at their actual resolved positions by runtime.
    let producers = templates.iter().map(|template| action_id(stage, &template.step.id)).collect::<BTreeSet<_>>();
    for (reference, binding) in local.dynamic {
        let template_output = matches!(&reference,ArtifactRef::Produced{action_id,..} if producers.contains(action_id));
        if !template_output && bindings.insert(reference, binding).is_some() {
            return Err(error("controller source hydration overwrote an immutable binding"));
        }
    }
    Ok(ActionControllerSpec {
        id: action_id(stage, &s.id),
        guard: None,
        commands_max: u32::try_from(s.commands_max).map_err(|_| error("controller command bound overflow"))?,
        slots: s.slots.iter().map(|slot| slot.id.clone()).collect(),
        templates: actions,
    })
}
