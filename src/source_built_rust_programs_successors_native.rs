//! Genuine original standard-library process requests are lowered by the same
//! source controller used by LLVM. Every requested native/generator command has
//! its own protected Produced image and authentic normal status.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::STAGE;
use super::error;
use super::input;
use super::lit;
use super::output;
use super::outputs;
use super::product;
use super::require;
use crate::errors::RunError;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;

pub(crate) struct ScriptRun {
    pub steps: Vec<RecipeStep>,
    pub stdout: ArtifactRef,
    pub out_dir: ArtifactRef,
    pub logical_out: String,
    pub generated_tmp: ArtifactRef,
}
pub(crate) fn out_directory(id: &str) -> String {
    format!("/tmp/mantle-rust/{id}/out")
}
/// Source-authored reservations bound suspended requests, never predict exits.
pub(crate) fn reservation_slots_max(remaining_steps: usize) -> Result<u32, RunError> {
    if remaining_steps < DECLARATION_STEPS_MAX {
        return Err(error("compact native script needs eleven fixed declarations"));
    }
    let commands = crate::source_built_recipe::controller::CONTROLLER_COMMANDS_MAX;
    debug_assert!(commands > 1);
    Ok((commands - 1) / 2)
}
const DECLARATION_STEPS_MAX: usize = 11;
fn helper_exec(
    id: &str,
    helper: ArtifactRef,
    runtime_tree: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    declared: BTreeMap<String, RecipeOutput>,
    exit_policy: RecipeExitPolicy,
) -> RecipeStep {
    inputs.insert("rust-runtime".into(), runtime_tree);
    inputs.insert("executable".into(), helper);
    RecipeStep {
        id: id.into(),
        inputs,
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "executable".into(),
            arguments,
            environment: BTreeMap::from([("LD_LIBRARY_PATH".into(), RecipeWord {
                parts: vec![WordPart::Runtime {
                    mount: "/mantle-rust-runtime".into(),
                    path: String::new(),
                }],
            })]),
            runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            exit_policy,
            merge_stderr: false,
            invocation_input: None,
        },
    }
}
/// Install complete source-built host runtime roots before source PATH lookup.
pub(crate) fn extend_world(
    request: &AuthoringRequest<'_>,
    members: &mut BTreeMap<String, ArtifactRef>,
) -> Result<(), RunError> {
    for (name, root) in super::super::llvm::native_command_world(request)?.inputs {
        require(&root)?;
        if let Some(previous) = members.insert(name.clone(), root.clone())
            && previous != root
        {
            return Err(error(format!("native source world root collision {name}")));
        }
    }
    Ok(())
}
pub(crate) fn extend_environment(
    request: &AuthoringRequest<'_>,
    environment: &mut BTreeMap<String, RecipeWord>,
) -> Result<(), RunError> {
    let world = super::super::llvm::native_command_world(request)?;
    let word = |value: &str| -> Result<RecipeWord, RunError> {
        let mut parts = Vec::new();
        for (index, path) in value.split(':').enumerate() {
            if index != 0 {
                parts.push(WordPart::Literal { text: ":".into() });
            }
            let (root, name) = world
                .runtime_mounts
                .iter()
                .find(|(root, _)| {
                    path == root.as_str() || path.strip_prefix(root.as_str()).is_some_and(|tail| tail.starts_with('/'))
                })
                .ok_or_else(|| error(format!("source host policy path lacks an admitted runtime root: {path}")))?;
            let tail = path.strip_prefix(root).expect("matched source root").trim_start_matches('/');
            parts.push(WordPart::Input {
                name: "package".into(),
                path: if tail.is_empty() {
                    name.clone()
                } else {
                    format!("{name}/{tail}")
                },
            });
        }
        Ok(RecipeWord { parts })
    };
    let host = word(&world.path_prefix)?;
    let mut path = input("package", "target-wrappers");
    path.parts.push(WordPart::Literal { text: ":".into() });
    path.parts
        .extend(environment.remove("PATH").unwrap_or_else(|| input("package", "phase-tools")).parts);
    path.parts.push(WordPart::Literal { text: ":".into() });
    path.parts.extend(host.parts);
    environment.insert("PATH".into(), path);
    for (key, value) in &world.environment {
        environment.entry(key.clone()).or_insert(word(value)?);
    }
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn schedule(
    request: &AuthoringRequest<'_>,
    id: &str,
    executable: ArtifactRef,
    package: ArtifactRef,
    compiler: ArtifactRef,
    helper: ArtifactRef,
    mut environment: BTreeMap<String, RecipeWord>,
    mut inputs: BTreeMap<String, ArtifactRef>,
    slots: usize,
    generated: &BTreeMap<String, ArtifactRef>,
) -> Result<ScriptRun, RunError> {
    for reference in [&executable, &package, &compiler, &helper] {
        require(reference)?;
    }
    let controller = inputs
        .remove("llvm-native-controller")
        .ok_or_else(|| error("native source controller lacks its exact LLVM-produced image"))?;
    let runtime_tree = inputs
        .get("rust-runtime")
        .or_else(|| inputs.get("native-runtime"))
        .cloned()
        .ok_or_else(|| error("native continuation requires the exact source-produced runtime tree"))?;
    inputs.remove("native-runtime");
    let llvm_config = inputs.remove("llvm-config");
    inputs.insert("rust-runtime".into(), runtime_tree.clone());
    inputs.insert("package".into(), package);
    inputs.insert("script".into(), executable.clone());
    let world = super::super::llvm::native_command_world(request)?;
    inputs.extend(world.inputs);
    let mut runtime = world.runtime_mounts;
    runtime.insert("/mantle-rust-runtime".into(), "rust-runtime".into());
    let logical_out = out_directory(id);
    let policy_root = request
        .source_roots
        .get(&crate::source_built_fixed_point::SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("native continuation policy source is absent"))?;
    let policy = crate::source_built_program_authoring::read_continuation_policy(policy_root)?;
    let per_request = crate::source_built_recipe::controller::reserve_native_commands(
        &policy,
        &format!("{id}.native"),
        u32::try_from(slots).map_err(|_| error("native reservation exceeds u32"))?,
        crate::source_built_recipe::STEP_COUNT_MAX,
    )?;
    let commands_max = command_budget(slots, per_request)?;
    let mut recipe = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(), stage_id: STAGE.into(),
        preparation: RecipePreparation { tool: "original-rust-standard-process".into(),
            provenance: "Shared source controller with original std Command requests, exact Produced native images, descriptor observations and bounded replay.".into() },
        steps: Vec::new(), timeout_groups: Vec::new(), roots: Vec::new(), exports: BTreeMap::new(), semantic_roots: BTreeMap::new(),
    };
    let initial = initial_workspace(&mut recipe, id, helper.clone(), runtime_tree.clone(), &logical_out, generated)?;
    let (frame, logical) = script_frame(
        &mut recipe,
        id,
        helper.clone(),
        runtime_tree.clone(),
        &mut inputs,
        &mut environment,
        &logical_out,
        slots,
        per_request,
    )?;
    inputs.insert("script-frame".into(), frame.clone());
    let readonly = inputs.keys().cloned().collect();
    let mut inventory = world.inventory;
    inventory.extend([("script".into(), executable), ("rustc".into(), compiler)]);
    if let Some(config) = llvm_config {
        inventory.push(("llvm-config".into(), config));
    }
    let result = super::super::llvm::append_native_source_script(
        request,
        &mut recipe,
        controller,
        super::super::llvm::NativeSourceScript {
            id: id.into(),
            script: ":".into(),
            initial_workspace: initial,
            initial_cwd: String::new(),
            initial_cwd_input: Some((frame.clone(), "cwd".into())),
            original_zero: "/inputs/script-frame/script".into(),
            environment: BTreeMap::new(),
            environment_input: Some((frame, "environment".into())),
            inputs,
            readonly_inputs: readonly,
            runtime_mounts: runtime,
            inventory,
            commands_max,
        },
    )?;
    let finish = format!("{id}.complete");
    let mut declared = outputs("out-dir", ArtifactKind::Tree);
    declared.insert("generated-tmp".into(), super::spec(ArtifactKind::Tree));
    recipe.steps.push(helper_exec(
        &finish,
        helper,
        runtime_tree,
        BTreeMap::from([
            ("checkpoint".into(), result.checkpoint),
            ("logical-out".into(), logical),
        ]),
        vec![
            lit("--finish-compact-script"),
            input("checkpoint", ""),
            input("logical-out", ""),
            output("out-dir"),
            output("generated-tmp"),
        ],
        declared,
        RecipeExitPolicy::Exact { code: 0 },
    ));
    debug_assert!(recipe.steps.len() <= DECLARATION_STEPS_MAX);
    debug_assert!(matches!(result.workspace, ArtifactRef::Produced { .. }));
    Ok(ScriptRun {
        steps: recipe.steps,
        stdout: product(&finish, "stdout"),
        out_dir: product(&finish, "out-dir"),
        logical_out,
        generated_tmp: product(&finish, "generated-tmp"),
    })
}

fn command_budget(slots: usize, per_request: usize) -> Result<usize, RunError> {
    let source_bound = slots
        .checked_mul(per_request)
        .and_then(|commands| commands.checked_add(slots))
        .and_then(|commands| commands.checked_add(1))
        .ok_or_else(|| error("native script command reservation overflow"))?;
    let ceiling = crate::source_built_recipe::controller::CONTROLLER_COMMANDS_MAX as usize;
    if per_request == 0 || slots > (ceiling - 1) / 2 {
        return Err(error("native script reservation exceeds command ceiling"));
    }
    // The source request and per-request bounds remain intact. The pre-existing
    // global execution ceiling is an independent intersection, never an override.
    debug_assert!(source_bound > 0);
    debug_assert!(ceiling > 1);
    Ok(source_bound.min(ceiling))
}

fn initial_workspace(
    recipe: &mut SourceRecipe,
    id: &str,
    helper: ArtifactRef,
    runtime: ArtifactRef,
    logical_out: &str,
    generated: &BTreeMap<String, ArtifactRef>,
) -> Result<ArtifactRef, RunError> {
    let empty = format!("{id}.empty");
    recipe.steps.push(helper_exec(
        &empty,
        helper,
        runtime,
        BTreeMap::new(),
        vec![lit("--empty"), output("tree")],
        outputs("tree", ArtifactKind::Tree),
        RecipeExitPolicy::Exact { code: 0 },
    ));
    let relative = logical_out.strip_prefix("/tmp/").ok_or_else(|| error("script output must be in tmp"))?;
    let mut inputs = BTreeMap::from([("own".into(), product(&empty, "tree"))]);
    let mut entries = BTreeMap::from([(relative.into(), "own".into())]);
    for (index, (path, tree)) in generated.iter().enumerate() {
        require(tree)?;
        let path = path.strip_prefix("/tmp/").ok_or_else(|| error("generated script state is outside tmp"))?;
        crate::source_built_recipe::relative(path, false)?;
        let alias = format!("root-{index}");
        inputs.insert(alias.clone(), tree.clone());
        entries.insert(path.into(), alias);
    }
    let seed = format!("{id}.initial-tmp");
    recipe.steps.push(RecipeStep {
        id: seed.clone(),
        inputs,
        outputs: BTreeMap::from([("tree".into(), super::spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries,
        },
    });
    let root = format!("{id}.initial-workspace");
    recipe.steps.push(RecipeStep {
        id: root.clone(),
        inputs: BTreeMap::from([("tmp".into(), product(&seed, "tree"))]),
        outputs: BTreeMap::from([("tree".into(), super::spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries: BTreeMap::from([("tmp".into(), "tmp".into())]),
        },
    });
    debug_assert!(recipe.steps.len() >= 3);
    Ok(product(&root, "tree"))
}

#[allow(clippy::too_many_arguments)]
fn script_frame(
    recipe: &mut SourceRecipe,
    id: &str,
    helper: ArtifactRef,
    runtime: ArtifactRef,
    inputs: &mut BTreeMap<String, ArtifactRef>,
    environment: &mut BTreeMap<String, RecipeWord>,
    logical_out: &str,
    slots: usize,
    per_request: usize,
) -> Result<(ArtifactRef, ArtifactRef), RunError> {
    let data = |text: String| RecipeWord {
        parts: vec![WordPart::LiteralData { text }],
    };
    for (key, word) in [
        ("OUT_DIR", data(logical_out.into())),
        ("MANTLE_CC_REQUEST", data(format!("{logical_out}/.mantle-successor-request"))),
        ("MANTLE_RUST_SCRIPT_IMAGE", data("/inputs/script-frame/script".into())),
        ("MANTLE_RUST_SCRIPT_SLOTS", lit(slots.to_string())),
        ("MANTLE_RUST_NATIVE_COMMANDS_MAX", lit(per_request.to_string())),
    ] {
        environment.insert(key.into(), word);
    }
    let logical = format!("{id}.logical-out");
    recipe.steps.push(RecipeStep {
        id: logical.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("root".into(), super::spec(ArtifactKind::Text))]),
        operation: RecipeOperation::WriteText {
            output: "root".into(),
            text: logical_out.into(),
        },
    });
    let frame = format!("{id}.script-frame");
    let mut args = vec![lit("--script-frame"), output("frame"), input("script", "")];
    let mut referenced = BTreeSet::from(["script".to_owned()]);
    let mut direct = BTreeSet::new();
    for (key, value) in std::mem::take(environment) {
        for part in &value.parts {
            match part {
                WordPart::Input { name, .. } => {
                    direct.insert(name.clone());
                    referenced.insert(name.clone());
                }
                WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                    referenced.insert(name.clone());
                }
                WordPart::Literal { .. } | WordPart::LiteralData { .. } | WordPart::Runtime { .. } => {}
                _ => return Err(error("script environment must not refer to an unbound mutable workspace")),
            }
        }
        args.extend([lit(key), value]);
    }
    let frame_inputs = referenced
        .iter()
        .map(|name| {
            let value = inputs.get(name).ok_or_else(|| error(format!("script environment input is absent: {name}")))?;
            Ok((name.clone(), value.clone()))
        })
        .collect::<Result<BTreeMap<_, _>, RunError>>()?;
    for name in referenced.difference(&direct) {
        inputs.remove(name);
    }
    recipe.steps.push(helper_exec(
        &frame,
        helper,
        runtime,
        frame_inputs,
        args,
        outputs("frame", ArtifactKind::Tree),
        RecipeExitPolicy::Exact { code: 0 },
    ));
    debug_assert!(recipe.steps.len() >= 2);
    Ok((product(&frame, "frame"), product(&logical, "root")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_reservations_use_runtime_ceiling_not_expanded_declarations() {
        let max = reservation_slots_max(DECLARATION_STEPS_MAX).unwrap();
        assert_eq!(reservation_slots_max(crate::source_built_recipe::STEP_COUNT_MAX).unwrap(), max);
        assert!(reservation_slots_max(DECLARATION_STEPS_MAX - 1).is_err());
        assert_eq!(command_budget(0, 8).unwrap(), 1);
        assert_eq!(command_budget(12, 56).unwrap(), 685);
        assert_eq!(
            command_budget(max as usize, 8).unwrap(),
            crate::source_built_recipe::controller::CONTROLLER_COMMANDS_MAX as usize
        );
        assert!(command_budget(max as usize + 1, 8).is_err());
        assert!(command_budget(1, 0).is_err());
        assert!(command_budget(usize::MAX, usize::MAX).is_err());
    }
}
