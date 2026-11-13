//! Compact protected source-compiler continuations at original proc-macro boundaries.
//! The controller only selects commands; original compiler and plugin images remain
//! separately guarded and observed leaves, including binary protocol and /tmp state.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::error;
use super::input;
use super::lit;
use super::output;
use super::outputs;
use super::produced;
use super::product;
use crate::errors::RunError;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeInvocation;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::RecipeWorkspaceMount;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_stagex_programs::controller::ControllerExecutable;
use crate::source_built_stagex_programs::controller::ControllerSlot;
use crate::source_built_stagex_programs::controller::ControllerSpec;

/// Empty initial tmp, normalized invocation, compact controller, and two extracts.
pub(super) const DECLARATION_STEPS_MAX: usize = 5;

fn commands_max(slots: usize) -> Result<usize, RunError> {
    let commands = slots
        .checked_mul(2)
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| error("macro command reservation overflow"))?;
    if slots == 0 || commands > crate::source_built_recipe::controller::CONTROLLER_COMMANDS_MAX as usize {
        return Err(error("macro command reservation exceeds the protected controller ceiling"));
    }
    Ok(commands)
}

fn command(
    id: &str,
    helper: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    name: &str,
    disk: u64,
) -> RecipeStep {
    inputs.insert("helper".into(), helper);
    let mut declared = outputs(name, ArtifactKind::Tree);
    declared.get_mut(name).expect("declared tree").bytes_max = disk;
    RecipeStep {
        id: id.into(),
        inputs,
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "helper".into(),
            invocation_input: None,
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    }
}

/// Remap only typed output operands, never source text or user literal data.
fn normalize_word(word: &mut RecipeWord) -> Result<(), RunError> {
    for part in &mut word.parts {
        if let WordPart::Output { name, path } = part {
            if name != "translation" {
                return Err(error("macro compiler refers to an output outside its translation tree"));
            }
            crate::source_built_recipe::relative(path, true)?;
            *name = "workspace".into();
            *path = if path.is_empty() {
                "translation".into()
            } else {
                format!("translation/{path}")
            };
        }
    }
    Ok(())
}

fn references_input(invocation: &RecipeInvocation, alias: &str) -> bool {
    invocation
        .arguments
        .iter()
        .chain(invocation.environment.values())
        .chain(invocation.cwd.iter())
        .any(|word| {
            word.parts.iter().any(|part| match part {
                WordPart::Input { name, .. } | WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                    name == alias
                }
                _ => false,
            })
        })
}

fn compiler_slot(compiler: RecipeStep) -> Result<(ControllerSlot, RecipeInvocation, String), RunError> {
    let RecipeOperation::Exec {
        executable_input,
        invocation_input,
        mut arguments,
        mut environment,
        runtime_mounts,
        cwd_input,
        cwd_output,
        output_seeds,
        command_state,
        guard,
        stdin_input,
        merge_stderr,
        exit_policy,
    } = compiler.operation
    else {
        return Err(error("macro continuation template is not an original compiler command"));
    };
    if invocation_input.is_some()
        || guard.is_some()
        || !output_seeds.is_empty()
        || command_state.is_some()
        || stdin_input.is_some()
        || runtime_mounts.contains_key("/tmp")
        || exit_policy != (RecipeExitPolicy::Exact { code: 0 })
    {
        return Err(error("macro continuation requires an unbound original static compiler invocation"));
    }
    if environment.contains_key("MRUSTC_CAIRN_MACRO_HISTORY") || environment.contains_key("MRUSTC_CAIRN_MACRO_WORK") {
        return Err(error("macro compiler shadows continuation environment"));
    }
    if !compiler.outputs.get("translation").is_some_and(|spec| spec.kind == ArtifactKind::Tree)
        || compiler.outputs.keys().any(|name| !matches!(name.as_str(), "translation" | "stdout" | "stderr"))
    {
        return Err(error("macro compiler must declare only its original translation tree and streams"));
    }
    let executable = compiler
        .inputs
        .get(&executable_input)
        .ok_or_else(|| error("macro compiler image is not bound"))?
        .clone();
    produced(&executable)?;
    let mut readonly = BTreeSet::from(["package".into()]);
    readonly.extend(runtime_mounts.values().cloned());
    let mut cwd = match (cwd_input, cwd_output) {
        (Some(name), None) => {
            readonly.insert(name.clone());
            Some(input(&name, ""))
        }
        (None, Some(name)) => Some(output(&name, "")),
        (None, None) => None,
        _ => return Err(error("macro compiler has conflicting cwd roots")),
    };
    for word in arguments.iter_mut().chain(environment.values_mut()).chain(cwd.iter_mut()) {
        normalize_word(word)?;
    }
    environment.insert("MRUSTC_CAIRN_MACRO_HISTORY".into(), input("workspace-seed", "history"));
    environment.insert("MRUSTC_CAIRN_MACRO_WORK".into(), output("workspace", "translation"));
    let argv0 = format!("/inputs/{executable_input}");
    let invocation = RecipeInvocation {
        arguments,
        environment,
        cwd,
        argv0: Some(RecipeWord {
            parts: vec![WordPart::TextSlice {
                name: "data".into(),
                offset_bytes: 0,
                length_bytes: argv0.len() as u64,
            }],
        }),
        executable_path: None,
        workspace_mounts: BTreeMap::from([("/tmp".into(), RecipeWorkspaceMount {
            output: "workspace".into(),
            path: "tmp".into(),
        })]),
        readonly_input_roots: readonly.into_iter().collect(),
        process_attributes: None,
        descriptors: None,
    };
    let mut inputs = compiler.inputs;
    if !references_input(&invocation, &executable_input)
        && !runtime_mounts.values().any(|name| name == &executable_input)
    {
        inputs.remove(&executable_input);
    }
    for name in &invocation.readonly_input_roots {
        let reference =
            inputs.get(name).ok_or_else(|| error(format!("macro compiler lacks readonly input root {name}")))?;
        produced(reference)?;
    }
    Ok((
        ControllerSlot {
            id: "compiler".into(),
            executable: ControllerExecutable::Fixed(executable),
            inputs,
            runtime_mounts,
            exit_policy: RecipeExitPolicy::OneOf { codes: vec![0, 75] },
            merge_stderr,
            generated_outputs: false,
        },
        invocation,
        argv0,
    ))
}

pub(super) fn append(
    program: &mut SourceRecipe,
    id: &str,
    compiler: RecipeStep,
    helper: ArtifactRef,
    plugins: &BTreeMap<String, ArtifactRef>,
    slots: usize,
    captured_tmp: Option<ArtifactRef>,
    disk: u64,
) -> Result<(ArtifactRef, ArtifactRef), RunError> {
    produced(&helper)?;
    let commands_max = commands_max(slots)?;
    if plugins.is_empty() {
        return Err(error("macro continuations require a nonempty exact original plugin inventory"));
    }
    let declarations = DECLARATION_STEPS_MAX - usize::from(captured_tmp.is_some());
    if program
        .steps
        .len()
        .checked_add(declarations)
        .is_none_or(|count| count > crate::source_built_recipe::STEP_COUNT_MAX)
    {
        return Err(error("compact macro controller exceeds remaining recipe declaration capacity"));
    }
    if let Some(tmp) = &captured_tmp {
        produced(tmp)?;
    }
    let (compiler, invocation, argv0) = compiler_slot(compiler)?;
    let mut macro_inputs = BTreeMap::new();
    for name in std::iter::once("package").chain(compiler.runtime_mounts.values().map(String::as_str)) {
        let reference =
            compiler.inputs.get(name).ok_or_else(|| error(format!("macro runtime input {name} is not bound")))?;
        macro_inputs.insert(name.into(), reference.clone());
    }
    // One canonical JSON line followed by the original argv[0] data frame.
    let config = format!(
        "{}\n{argv0}",
        serde_json::to_string(&invocation).map_err(|e| error(format!("macro compiler invocation: {e}")))?
    );
    let mut inputs = BTreeMap::new();
    let mut start_arguments = vec![
        lit("--macro-control"),
        lit("-"),
        lit("-"),
        output("checkpoint", ""),
        input("compiler-config", ""),
        input("initial-tmp", ""),
        lit(slots.to_string()),
    ];
    for (index, (name, image)) in plugins.iter().enumerate() {
        produced(image)?;
        crate::source_built_recipe::relative(name, false)?;
        if name.contains('/') {
            return Err(error("macro inventory keys must retain exact original executable basenames"));
        }
        let alias = format!("plugin-{index}");
        inputs.insert(alias.clone(), image.clone());
        start_arguments.extend([lit(name), input(&alias, "")]);
    }
    for name in inputs.keys().map(String::as_str).chain(["compiler-config", "initial-tmp"]) {
        if compiler.inputs.contains_key(name) || macro_inputs.contains_key(name) {
            return Err(error(format!("macro controller source/command input name collision {name}")));
        }
    }
    let tmp = if let Some(tmp) = captured_tmp {
        tmp
    } else {
        let empty = format!("{id}.macro-empty");
        program.steps.push(command(
            &empty,
            helper.clone(),
            BTreeMap::new(),
            vec![lit("--empty"), output("tree", "")],
            "tree",
            disk,
        ));
        product(&program.stage_id, &empty, "tree")
    };
    let config_id = format!("{id}.macro-compiler-config");
    program.steps.push(RecipeStep {
        id: config_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("invocation".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: config.len() as u64,
        })]),
        operation: RecipeOperation::WriteText {
            output: "invocation".into(),
            text: config,
        },
    });
    inputs.insert("compiler-config".into(), product(&program.stage_id, &config_id, "invocation"));
    inputs.insert("initial-tmp".into(), tmp);
    let macro_slot = ControllerSlot {
        id: "macro".into(),
        executable: ControllerExecutable::Inventory(
            plugins.iter().map(|(name, image)| (name.clone(), image.clone())).collect(),
        ),
        inputs: macro_inputs,
        runtime_mounts: compiler.runtime_mounts.clone(),
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        merge_stderr: false,
        generated_outputs: false,
    };
    let result = crate::source_built_recipe::controller::append(program, ControllerSpec {
        id: format!("{id}.macro-controller"),
        controller: helper.clone(),
        inputs,
        controller_runtime_mounts: BTreeMap::new(),
        start_arguments,
        environment: BTreeMap::new(),
        restart_indices: [1, 2, 3],
        slots: vec![compiler, macro_slot],
        commands_max,
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        file_bytes_max: super::MAX,
        tree_bytes_max: disk,
    })?;
    for member in ["translation", "tmp"] {
        let select = format!("{id}.macro-{member}");
        program.steps.push(command(
            &select,
            helper.clone(),
            BTreeMap::from([("workspace".into(), result.workspace.clone())]),
            vec![lit("--select"), input("workspace", member), output("tree", "")],
            "tree",
            disk,
        ));
    }
    Ok((
        product(&program.stage_id, &format!("{id}.macro-translation"), "tree"),
        product(&program.stage_id, &format!("{id}.macro-tmp"), "tree"),
    ))
}

#[cfg(test)]
#[path = "source_built_rust_programs_macros_tests.rs"]
mod tests;
