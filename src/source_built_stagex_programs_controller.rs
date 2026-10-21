//! Bounded source-controller request slots. A controller computes data; every
//! selected original command remains a separately guarded, observed action.
use std::collections::BTreeMap;

use super::error;
use super::input;
use super::literal;
use super::output;
use crate::errors::RunError;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeGuard;
use crate::source_built_recipe::RecipeGuardCondition;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ControllerExecutable {
    Fixed(ArtifactRef),
    Adapted {
        executable: ArtifactRef,
        original_image: ArtifactRef,
    },
    /// Finite individually Produced images, selected by exact original request
    /// name and bytes. This is not a runtime-directory executable grant.
    Inventory(Vec<(String, ArtifactRef)>),
    /// Only the isolated products of these earlier actual compiler/link slots
    /// may supply a generated executable. Source/workspace ELF is not authority.
    RequestOutput {
        producer_slots: Vec<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControllerSlot {
    pub id: String,
    pub executable: ControllerExecutable,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub exit_policy: RecipeExitPolicy,
    pub merge_stderr: bool,
    /// The source compiler/link entry publishes a dedicated, initially empty
    /// product tree, populated only by its actual original output callbacks.
    pub generated_outputs: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControllerSpec {
    pub id: String,
    pub controller: ArtifactRef,
    pub inputs: BTreeMap<String, ArtifactRef>,
    /// Exact original source/dependency roots needed by the evaluator itself.
    /// Values name this controller's source inputs, never executable grants.
    pub controller_runtime_mounts: BTreeMap<String, String>,
    /// Initial source-controller CLI; its checkpoint output is named checkpoint.
    pub start_arguments: Vec<RecipeWord>,
    pub environment: BTreeMap<String, RecipeWord>,
    /// Indices in start_arguments of previous checkpoint, acknowledged result,
    /// and new checkpoint. These are source-controller CLI positions, not argv
    /// inferred from a tool's command line.
    pub restart_indices: [usize; 3],
    pub slots: Vec<ControllerSlot>,
    pub commands_max: usize,
    pub exit_policy: RecipeExitPolicy,
    pub file_bytes_max: u64,
    pub tree_bytes_max: u64,
}
pub(crate) struct ControllerResult {
    pub checkpoint: ArtifactRef,
    pub workspace: ArtifactRef,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub times: ArtifactRef,
    pub generated: ArtifactRef,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub status: ArtifactRef,
}
fn spec(kind: ArtifactKind, bytes_max: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max }
}
fn push(recipe: &mut SourceRecipe, step: RecipeStep) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    if recipe.steps.len() >= crate::source_built_recipe::STEP_COUNT_MAX {
        return Err(error("source controller template count exceeds recipe declaration bound"));
    }
    let refs = step
        .outputs
        .keys()
        .map(|id| {
            (id.clone(), ArtifactRef::Produced {
                action_id: format!("{}/{}", recipe.stage_id, step.id),
                output_id: id.clone(),
            })
        })
        .collect();
    recipe.steps.push(step);
    Ok(refs)
}
fn streams(outputs: &mut BTreeMap<String, RecipeOutput>, limit: u64) {
    outputs.insert("stdout".into(), spec(ArtifactKind::Object, limit));
    outputs.insert("stderr".into(), spec(ArtifactKind::Text, limit));
    outputs.insert("exit-status".into(), spec(ArtifactKind::Text, 4));
}
struct ControllerRuntime {
    inputs: BTreeMap<String, ArtifactRef>,
    mounts: BTreeMap<String, String>,
}
impl ControllerRuntime {
    fn new(
        source_inputs: &BTreeMap<String, ArtifactRef>,
        source_mounts: &BTreeMap<String, String>,
        slots: &[ControllerSlot],
    ) -> Result<Self, RunError> {
        let mut runtime = Self {
            inputs: BTreeMap::new(),
            mounts: BTreeMap::new(),
        };
        for inputs in std::iter::once(source_inputs).chain(slots.iter().map(|slot| &slot.inputs)) {
            for (name, reference) in inputs {
                if let Some(old) = runtime.inputs.get(name) {
                    if old != reference {
                        return Err(error(format!("controller input root {name} has conflicting original producers")));
                    }
                } else {
                    runtime.inputs.insert(name.clone(), reference.clone());
                }
            }
        }
        for (inputs, mounts) in std::iter::once((source_inputs, source_mounts))
            .chain(slots.iter().map(|slot| (&slot.inputs, &slot.runtime_mounts)))
        {
            for (mount, input_name) in mounts {
                let reference = inputs
                    .get(input_name)
                    .ok_or_else(|| error(format!("controller runtime mount {mount} lacks input {input_name}")))?;
                if let Some(name) = runtime.mounts.get(mount) {
                    if runtime.inputs.get(name) != Some(reference) {
                        return Err(error(format!(
                            "controller runtime root {mount} has conflicting original producers"
                        )));
                    }
                    continue;
                }
                let name = format!("controller-runtime-{}", runtime.inputs.len());
                runtime.inputs.insert(name.clone(), reference.clone());
                runtime.mounts.insert(mount.clone(), name);
            }
        }
        Ok(runtime)
    }
    fn bind(&self, inputs: &mut BTreeMap<String, ArtifactRef>) -> Result<(), RunError> {
        for (name, reference) in &self.inputs {
            if let Some(old) = inputs.get(name) {
                if old != reference {
                    return Err(error(format!("conflicting controller input root {name}")));
                }
            } else {
                inputs.insert(name.clone(), reference.clone());
            }
        }
        Ok(())
    }
}
fn command(
    recipe: &mut SourceRecipe,
    id: String,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    environment: BTreeMap<String, RecipeWord>,
    mut outputs: BTreeMap<String, RecipeOutput>,
    policy: RecipeExitPolicy,
    limit: u64,
    runtime: &ControllerRuntime,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    runtime.bind(&mut inputs)?;
    inputs.insert("executable".into(), executable);
    streams(&mut outputs, limit);
    push(recipe, RecipeStep {
        id,
        inputs,
        outputs,
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "executable".into(),
            arguments,
            environment,
            runtime_mounts: runtime.mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: policy,
            guard: None,
            output_seeds: BTreeMap::new(),
        },
    })
}
/// Construct only one immutable turn. The generic source runtime instantiates
/// exact admitted positions lazily and stops at the first genuine done result.
pub(crate) fn template_recipe(
    stage: &str,
    specification: &ControllerSpec,
    template_id: Option<&str>,
) -> Result<SourceRecipe, RunError> {
    let mut recipe = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: stage.into(),
        preparation: crate::source_built_recipe::RecipePreparation {
            tool: "source-controller".into(),
            provenance: "immutable source controller templates".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    let mut specification = specification.clone();
    if let Some(id) = template_id {
        specification.id = id.into();
    }
    build_templates(&mut recipe, specification)?;
    Ok(recipe)
}
fn build_templates(recipe: &mut SourceRecipe, specification: ControllerSpec) -> Result<ControllerResult, RunError> {
    let s = specification;
    if s.commands_max == 0 || s.file_bytes_max < 4 || s.tree_bytes_max == 0 || s.slots.is_empty() {
        return Err(error("invalid original controller resource bound or empty command catalog"));
    }
    let templates_count = s
        .slots
        .iter()
        .try_fold(3usize, |count, slot| {
            count.checked_add(
                if matches!(
                    slot.executable,
                    ControllerExecutable::Inventory(_) | ControllerExecutable::RequestOutput { .. }
                ) {
                    4
                } else {
                    3
                },
            )
        })
        .ok_or_else(|| error("controller template count overflow"))?;
    if templates_count > crate::source_built_recipe::STEP_COUNT_MAX {
        return Err(error("source controller template count exceeds recipe declaration bound"));
    }
    let slots = s.slots.iter().map(|slot| (slot.id.as_str(), slot)).collect::<BTreeMap<_, _>>();
    if slots.len() != s.slots.len() {
        return Err(error("duplicate original controller command slot"));
    }
    for slot in &s.slots {
        if let ControllerExecutable::RequestOutput { producer_slots } = &slot.executable
            && (producer_slots.is_empty()
                || producer_slots
                    .iter()
                    .any(|key| !slots.get(key.as_str()).is_some_and(|candidate| candidate.generated_outputs)))
        {
            return Err(error("generated command lacks exact original compiler/link producer slots"));
        }
        for reserved in [
            "executable",
            "invocation",
            "workspace-seed",
            "descriptor-files-seed",
            "stdin",
            "times",
            "data",
            "prepare-status",
            "copy-status",
        ] {
            if slot.inputs.contains_key(reserved) {
                return Err(error(format!("reserved controller input {reserved}")));
            }
        }
    }
    if s.restart_indices.iter().any(|index| *index >= s.start_arguments.len())
        || s.restart_indices[0] == s.restart_indices[1]
        || s.restart_indices[0] == s.restart_indices[2]
        || s.restart_indices[1] == s.restart_indices[2]
    {
        return Err(error("invalid source controller restart positions"));
    }
    let runtime = ControllerRuntime::new(&s.inputs, &s.controller_runtime_mounts, &s.slots)?;
    let mut start_arguments = vec![
        literal("--mantle-controller-start"),
        output("checkpoint", ""),
        literal("--"),
    ];
    start_arguments.extend(s.start_arguments.clone());
    let started = command(
        recipe,
        format!("{}-start", s.id),
        s.controller.clone(),
        s.inputs.clone(),
        start_arguments,
        s.environment.clone(),
        BTreeMap::from([("checkpoint".into(), spec(ArtifactKind::Tree, s.tree_bytes_max))]),
        RecipeExitPolicy::OneOf { codes: vec![0, 1] },
        s.file_bytes_max,
        &runtime,
    )?;
    let checkpoint = started["checkpoint"].clone();
    let checkpoint_member = "";
    let ordinal = 0usize;
    let mut observations = Vec::with_capacity(s.slots.len());
    for slot in &s.slots {
        let base = format!("{}-{ordinal:06}-{}", s.id, slot.id);
        let mut inputs = s.inputs.clone();
        for (name, reference) in &slot.inputs {
            if inputs.insert(name.clone(), reference.clone()).is_some() {
                return Err(error(format!("controller source/command input name collision {name}")));
            }
        }
        inputs.insert("checkpoint".into(), checkpoint.clone());
        let mut arguments = vec![
            literal("--mantle-controller-prepare"),
            input("checkpoint", checkpoint_member),
            literal(&slot.id),
            output("invocation", ""),
            output("workspace", ""),
            output("stdin", ""),
            output("times", ""),
            output("data", ""),
            output("descriptor-files", ""),
        ];
        for name in s.inputs.keys() {
            arguments.extend([literal(name), input(name, "")]);
        }
        arguments.push(literal("--command-inputs"));
        for name in slot.inputs.keys() {
            arguments.extend([literal(name), input(name, "")]);
        }
        let runtime_inputs = slot.runtime_mounts.values().collect::<std::collections::BTreeSet<_>>();
        arguments.extend([literal("--runtime-inputs"), literal(&runtime_inputs.len().to_string())]);
        arguments.extend(runtime_inputs.into_iter().map(|name| literal(name)));
        arguments.push(literal("--"));
        match &slot.executable {
            ControllerExecutable::Fixed(image)
            | ControllerExecutable::Adapted {
                original_image: image, ..
            } => {
                let name = "selector-image";
                inputs.insert(name.into(), image.clone());
                arguments.extend([literal("fixed-image"), literal(&slot.id), input(name, "")]);
            }
            ControllerExecutable::Inventory(images) => {
                if images.is_empty() {
                    return Err(error("empty source command image inventory"));
                }
                arguments.extend([literal("inventory"), literal(&images.len().to_string())]);
                for (index, (key, image)) in images.iter().enumerate() {
                    let name = format!("selector-image-{index}");
                    inputs.insert(name.clone(), image.clone());
                    arguments.extend([literal(key), input(&name, "")]);
                }
            }
            ControllerExecutable::RequestOutput { producer_slots } => {
                arguments.extend([literal("generated"), literal(&producer_slots.len().to_string())]);
                arguments.extend(producer_slots.iter().map(|key| literal(key)));
            }
        }
        let prepared = command(
            recipe,
            format!("{base}-prepare"),
            s.controller.clone(),
            inputs,
            arguments,
            s.environment.clone(),
            BTreeMap::from([
                ("invocation".into(), spec(ArtifactKind::Text, s.file_bytes_max)),
                ("workspace".into(), spec(ArtifactKind::Tree, s.tree_bytes_max)),
                ("stdin".into(), spec(ArtifactKind::Object, s.file_bytes_max)),
                ("times".into(), spec(ArtifactKind::Text, s.file_bytes_max)),
                ("data".into(), spec(ArtifactKind::Text, s.file_bytes_max)),
                ("descriptor-files".into(), spec(ArtifactKind::Tree, s.tree_bytes_max)),
            ]),
            RecipeExitPolicy::OneOf { codes: vec![0, 1] },
            s.file_bytes_max,
            &runtime,
        )?;
        let mut inputs = slot.inputs.clone();
        let mut guard = vec![RecipeGuardCondition {
            status_input: "prepare-status".into(),
            accepted_codes: vec![0],
        }];
        inputs.extend([
            ("prepare-status".into(), prepared["exit-status"].clone()),
            ("invocation".into(), prepared["invocation"].clone()),
            ("workspace-seed".into(), prepared["workspace"].clone()),
            ("descriptor-files-seed".into(), prepared["descriptor-files"].clone()),
            ("stdin".into(), prepared["stdin"].clone()),
            ("times".into(), prepared["times"].clone()),
            ("data".into(), prepared["data"].clone()),
        ]);
        let executable = match &slot.executable {
            ControllerExecutable::Fixed(reference) => reference.clone(),
            ControllerExecutable::Adapted { executable, .. } => executable.clone(),
            ControllerExecutable::Inventory(images) => {
                let mut copy_inputs = BTreeMap::from([
                    ("checkpoint".into(), checkpoint.clone()),
                    ("prepare-status".into(), prepared["exit-status"].clone()),
                    ("executable".into(), s.controller.clone()),
                ]);
                let mut args = vec![
                    literal("--mantle-controller-executable"),
                    input("checkpoint", checkpoint_member),
                    literal(&slot.id),
                    output("program", ""),
                    literal("inventory"),
                    literal(&images.len().to_string()),
                ];
                for (index, (key, image)) in images.iter().enumerate() {
                    let name = format!("image-{index}");
                    copy_inputs.insert(name.clone(), image.clone());
                    args.extend([literal(key), input(&name, "")]);
                }
                runtime.bind(&mut copy_inputs)?;
                let mut outputs =
                    BTreeMap::from([("program".into(), spec(ArtifactKind::Executable, s.file_bytes_max))]);
                streams(&mut outputs, s.file_bytes_max);
                let copied = push(recipe, RecipeStep {
                    id: format!("{base}-executable"),
                    inputs: copy_inputs,
                    outputs,
                    operation: RecipeOperation::Exec {
                        command_state: None,
                        executable_input: "executable".into(),
                        arguments: args,
                        environment: BTreeMap::new(),
                        runtime_mounts: runtime.mounts.clone(),
                        cwd_input: None,
                        cwd_output: None,
                        stdin_input: None,
                        merge_stderr: false,
                        invocation_input: None,
                        exit_policy: RecipeExitPolicy::Exact { code: 0 },
                        guard: Some(RecipeGuard {
                            conditions: guard.clone(),
                        }),
                        output_seeds: BTreeMap::new(),
                    },
                })?;
                inputs.insert("copy-status".into(), copied["exit-status"].clone());
                guard.push(RecipeGuardCondition {
                    status_input: "copy-status".into(),
                    accepted_codes: vec![0],
                });
                copied["program"].clone()
            }
            ControllerExecutable::RequestOutput { producer_slots } => {
                let mut copy_inputs = BTreeMap::from([
                    ("checkpoint".into(), checkpoint.clone()),
                    ("prepare-status".into(), prepared["exit-status"].clone()),
                ]);
                let mut args = vec![
                    literal("--mantle-controller-executable"),
                    input("checkpoint", checkpoint_member),
                    literal(&slot.id),
                    output("program", ""),
                    literal("generated"),
                    literal(&producer_slots.len().to_string()),
                ];
                for producer in producer_slots {
                    args.push(literal(producer));
                }
                copy_inputs.insert("executable".into(), s.controller.clone());
                runtime.bind(&mut copy_inputs)?;
                let mut outputs =
                    BTreeMap::from([("program".into(), spec(ArtifactKind::Executable, s.file_bytes_max))]);
                streams(&mut outputs, s.file_bytes_max);
                let copied = push(recipe, RecipeStep {
                    id: format!("{base}-executable"),
                    inputs: copy_inputs,
                    outputs,
                    operation: RecipeOperation::Exec {
                        command_state: None,
                        executable_input: "executable".into(),
                        arguments: args,
                        environment: BTreeMap::new(),
                        runtime_mounts: runtime.mounts.clone(),
                        cwd_input: None,
                        cwd_output: None,
                        stdin_input: None,
                        merge_stderr: false,
                        invocation_input: None,
                        exit_policy: RecipeExitPolicy::Exact { code: 0 },
                        guard: Some(RecipeGuard {
                            conditions: guard.clone(),
                        }),
                        output_seeds: BTreeMap::new(),
                    },
                })?;
                inputs.insert("copy-status".into(), copied["exit-status"].clone());
                guard.push(RecipeGuardCondition {
                    status_input: "copy-status".into(),
                    accepted_codes: vec![0],
                });
                copied["program"].clone()
            }
        };
        inputs.insert("executable".into(), executable);
        let mut outputs = BTreeMap::from([
            ("workspace".into(), spec(ArtifactKind::Tree, s.tree_bytes_max)),
            ("times".into(), spec(ArtifactKind::Text, s.file_bytes_max)),
            ("descriptors".into(), spec(ArtifactKind::Text, s.file_bytes_max)),
        ]);
        outputs.insert("descriptor-files".into(), spec(ArtifactKind::Tree, s.tree_bytes_max));
        if slot.generated_outputs {
            outputs.insert("generated".into(), spec(ArtifactKind::Tree, s.tree_bytes_max));
        }
        streams(&mut outputs, s.file_bytes_max);
        let phase = format!("{base}-command");
        push(recipe, RecipeStep {
            id: phase.clone(),
            inputs,
            outputs,
            operation: RecipeOperation::Exec {
                command_state: Some(crate::source_built_recipe::RecipeCommandState {
                    workspace_output: "workspace".into(),
                    prior_times_input: Some("times".into()),
                    times_output: "times".into(),
                    descriptors_output: "descriptors".into(),
                    descriptor_files_output: Some("descriptor-files".into()),
                }),
                executable_input: "executable".into(),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: slot.runtime_mounts.clone(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: Some("stdin".into()),
                merge_stderr: slot.merge_stderr,
                invocation_input: Some("invocation".into()),
                exit_policy: slot.exit_policy.clone(),
                guard: Some(RecipeGuard { conditions: guard }),
                output_seeds: BTreeMap::from([
                    ("workspace".into(), "workspace-seed".into()),
                    ("descriptor-files".into(), "descriptor-files-seed".into()),
                ]),
            },
        })?;
        let observed = push(recipe, RecipeStep {
            id: format!("{base}-observation"),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("phase".into(), spec(ArtifactKind::Tree, s.tree_bytes_max))]),
            operation: RecipeOperation::ObservePhase {
                phase,
                output: "phase".into(),
                state: None,
            },
        })?;
        observations.push((slot.id.clone(), slot.generated_outputs, observed["phase"].clone()));
    }
    let mut inputs = s.inputs.clone();
    inputs.insert("checkpoint".into(), checkpoint.clone());
    let mut arguments = vec![
        literal("--mantle-controller-resume"),
        input("checkpoint", checkpoint_member),
        output("checkpoint", ""),
        literal(&s.restart_indices[0].to_string()),
        literal(&s.restart_indices[1].to_string()),
        literal(&s.restart_indices[2].to_string()),
        literal(&observations.len().to_string()),
    ];
    for (index, (key, generated, reference)) in observations.into_iter().enumerate() {
        let name = format!("phase-{index}");
        inputs.insert(name.clone(), reference);
        arguments.extend([
            literal(&key),
            literal(if generated { "1" } else { "0" }),
            input(&name, ""),
        ]);
    }
    arguments.push(literal("--"));
    arguments.extend(s.start_arguments.clone());
    let phase = format!("{}-{ordinal:06}-resume", s.id);
    command(
        recipe,
        phase,
        s.controller.clone(),
        inputs,
        arguments,
        s.environment.clone(),
        BTreeMap::from([("checkpoint".into(), spec(ArtifactKind::Tree, s.tree_bytes_max))]),
        RecipeExitPolicy::OneOf { codes: vec![0, 1] },
        s.file_bytes_max,
        &runtime,
    )?;
    let done = command(
        recipe,
        format!("{}-finish", s.id),
        s.controller,
        BTreeMap::from([("checkpoint".into(), checkpoint)]),
        vec![
            literal("--mantle-controller-finish"),
            input("checkpoint", checkpoint_member),
            output("workspace", ""),
            output("times", ""),
            output("generated", ""),
            output("checkpoint", ""),
        ],
        BTreeMap::new(),
        BTreeMap::from([
            ("workspace".into(), spec(ArtifactKind::Tree, s.tree_bytes_max)),
            ("times".into(), spec(ArtifactKind::Text, s.file_bytes_max)),
            ("generated".into(), spec(ArtifactKind::Tree, s.tree_bytes_max)),
            ("checkpoint".into(), spec(ArtifactKind::Tree, s.tree_bytes_max)),
        ]),
        s.exit_policy,
        s.file_bytes_max,
        &runtime,
    )?;
    Ok(ControllerResult {
        checkpoint: done["checkpoint"].clone(),
        workspace: done["workspace"].clone(),
        times: done["times"].clone(),
        generated: done["generated"].clone(),
        status: done["exit-status"].clone(),
    })
}

/// A persisted controller is one declaration, not commands_max × catalog slots.
pub(crate) fn append_controller(
    recipe: &mut SourceRecipe,
    specification: ControllerSpec,
) -> Result<ControllerResult, RunError> {
    crate::source_built_recipe::controller::append(recipe, specification)
}

/// One actual prior-status decision skips the whole controller honestly.
pub(crate) fn append_guarded_controller(
    recipe: &mut SourceRecipe,
    specification: ControllerSpec,
    predicates: Vec<(ArtifactRef, Vec<i32>)>,
) -> Result<ControllerResult, RunError> {
    crate::source_built_recipe::controller::append_guarded(recipe, specification, predicates)
}
