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
use crate::source_built_recipe::RecipeGuard;
use crate::source_built_recipe::RecipeGuardCondition;
use crate::source_built_recipe::RecipeInvocation;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePhaseState;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::RecipeWorkspaceMount;
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
    let slots = remaining_steps
        .checked_sub(9)
        .ok_or_else(|| error("no remaining source continuation layout capacity"))?
        / 11;
    u32::try_from(slots).map_err(|_| error("native continuation capacity exceeds u32"))
}
fn guarded(status: &str, codes: Vec<i32>) -> RecipeGuard {
    RecipeGuard {
        conditions: vec![RecipeGuardCondition {
            status_input: status.into(),
            accepted_codes: codes,
        }],
    }
}
fn exec(
    id: &str,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    environment: BTreeMap<String, RecipeWord>,
    declared: BTreeMap<String, RecipeOutput>,
    guard: Option<RecipeGuard>,
    exit_policy: RecipeExitPolicy,
    runtime: BTreeMap<String, String>,
) -> RecipeStep {
    inputs.insert("executable".into(), executable);
    RecipeStep {
        id: id.into(),
        inputs,
        outputs: declared,
        operation: RecipeOperation::Exec {
            executable_input: "executable".into(),
            arguments,
            environment,
            runtime_mounts: runtime,
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard,
            exit_policy,
            merge_stderr: false,
            invocation_input: None,
        },
    }
}
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
    exec(
        id,
        helper,
        inputs,
        arguments,
        BTreeMap::from([("LD_LIBRARY_PATH".into(), RecipeWord {
            parts: vec![WordPart::Runtime {
                mount: "/mantle-rust-runtime".into(),
                path: String::new(),
            }],
        })]),
        declared,
        None,
        exit_policy,
        BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
    )
}
fn observe(steps: &mut Vec<RecipeStep>, id: &str, phase: &str, state: ArtifactRef) -> ArtifactRef {
    steps.push(RecipeStep {
        id: id.into(),
        inputs: BTreeMap::from([("previous-state".into(), state)]),
        outputs: BTreeMap::from([("observation".into(), super::spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::ObservePhase {
            phase: phase.into(),
            output: "observation".into(),
            state: Some(RecipePhaseState {
                output: "out-dir".into(),
                skipped_input: "previous-state".into(),
                skipped_path: String::new(),
            }),
        },
    });
    product(id, "observation")
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
    let controller_helper = inputs
        .remove("llvm-native-helper")
        .ok_or_else(|| error("native source controller lacks its exact LLVM-produced I/O helper"))?;
    let world = super::super::llvm::native_command_world(request)?;
    let mut runtime = world.runtime_mounts.clone();
    runtime.insert("/mantle-rust-runtime".into(), "rust-runtime".into());
    let runtime_tree = inputs
        .get("rust-runtime")
        .or_else(|| inputs.get("native-runtime"))
        .cloned()
        .ok_or_else(|| error("native continuation requires the exact source-produced runtime tree"))?;
    inputs.remove("native-runtime");
    let llvm_config = inputs.remove("llvm-config");
    inputs.insert("rust-runtime".into(), runtime_tree.clone());
    inputs.insert("package".into(), package);
    inputs.extend(world.inputs.clone());
    let logical_out = out_directory(id);
    let relative_out = logical_out.strip_prefix("/tmp/").expect("fixed logical output root");
    environment.insert("OUT_DIR".into(), RecipeWord {
        parts: vec![WordPart::Workspace {
            mount: "/tmp".into(),
            path: relative_out.into(),
        }],
    });
    environment.insert("MANTLE_CC_REQUEST".into(), RecipeWord {
        parts: vec![WordPart::Workspace {
            mount: "/tmp".into(),
            path: format!("{relative_out}/.mantle-successor-request"),
        }],
    });
    let policy_root = request
        .source_roots
        .get(&crate::source_built_fixed_point::SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("native continuation policy source is absent"))?;
    let policy = crate::source_built_program_authoring::read_continuation_policy(policy_root)?;
    let commands_max = crate::source_built_recipe::controller::reserve_native_commands(
        &policy,
        &format!("{id}.native"),
        u32::try_from(slots).map_err(|_| error("native source reservation hint exceeds u32"))?,
        crate::source_built_recipe::STEP_COUNT_MAX,
    )?;
    let mut recipe=SourceRecipe{schema:crate::source_built_recipe::SCHEMA.into(),stage_id:STAGE.into(),preparation:RecipePreparation{tool:"original-rust-standard-process".into(),provenance:"Original std Command output buffers use declared measured capture-file ports; inherited descriptors retain genuine file/null/OFD state. Native descendants use original source controller algorithms.".into()},steps:Vec::new(),timeout_groups:Vec::new(),roots:Vec::new(),exports:BTreeMap::new(),semantic_roots:BTreeMap::new()};
    let initial = format!("{id}.empty");
    recipe.steps.push(helper_exec(
        &initial,
        helper.clone(),
        runtime_tree.clone(),
        BTreeMap::new(),
        vec![lit("--empty"), output("tree")],
        outputs("tree", ArtifactKind::Tree),
        RecipeExitPolicy::Exact { code: 0 },
    ));
    let logical_id = format!("{id}.logical-out");
    recipe.steps.push(RecipeStep {
        id: logical_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("root".into(), super::spec(ArtifactKind::Text))]),
        operation: RecipeOperation::WriteText {
            output: "root".into(),
            text: logical_out.clone(),
        },
    });
    inputs.insert("logical-out".into(), product(&logical_id, "root"));
    let seed = format!("{id}.initial-tmp");
    let mut seed_inputs = BTreeMap::from([("own".into(), product(&initial, "tree"))]);
    let mut entries = BTreeMap::from([(relative_out.to_string(), "own".into())]);
    for (index, (root, tree)) in generated.iter().enumerate() {
        require(tree)?;
        let path = root
            .strip_prefix("/tmp/")
            .ok_or_else(|| error("captured source output is outside original tmp workspace"))?;
        crate::source_built_recipe::relative(path, false)?;
        let alias = format!("root-{index}");
        seed_inputs.insert(alias.clone(), tree.clone());
        entries.insert(path.into(), alias);
    }
    recipe.steps.push(RecipeStep {
        id: seed.clone(),
        inputs: seed_inputs,
        outputs: BTreeMap::from([("tree".into(), super::spec(ArtifactKind::Tree))]),
        operation: RecipeOperation::AssembleTree {
            output: "tree".into(),
            entries,
        },
    });
    inputs.insert("initial-tmp".into(), product(&seed, "tree"));
    let mut state = product(&initial, "tree");
    let mut replies = state.clone();
    let mut previous = state.clone();
    let mut pending: Option<ArtifactRef> = None;
    for slot in 0..=slots {
        if recipe.steps.len() > crate::source_built_recipe::STEP_COUNT_MAX {
            return Err(error("original generator continuation exceeds recipe action ceiling"));
        }
        let prefix = format!("{id}.slot-{slot:04}");
        let run = format!("{prefix}.script");
        let mut run_inputs = inputs.clone();
        run_inputs.insert("cc-replies".into(), replies.clone());
        run_inputs.insert("cc-state".into(), previous.clone());
        let mut env = environment.clone();
        env.insert("MANTLE_CC_REPLIES".into(), input("cc-replies", ""));
        env.insert("MANTLE_CC_PHASE_STATE".into(), input("cc-state", relative_out));
        let guard = if let Some(status) = &pending {
            run_inputs.insert("pending".into(), status.clone());
            guarded("pending", vec![1])
        } else {
            run_inputs.insert("initial-status".into(), product(&initial, "exit-status"));
            guarded("initial-status", vec![0])
        };
        let invocation = RecipeInvocation {
            arguments: Vec::new(),
            environment: env,
            cwd: environment.get("CARGO_MANIFEST_DIR").cloned(),
            argv0: None,
            executable_path: None,
            workspace_mounts: BTreeMap::from([("/tmp".into(), RecipeWorkspaceMount {
                output: "out-dir".into(),
                path: String::new(),
            })]),
            readonly_input_roots: vec!["package".into()],
            process_attributes: None,
            descriptors: None,
        };
        let text = serde_json::to_string(&invocation)
            .map_err(|reason| error(format!("original script invocation: {reason}")))?;
        let frame = format!("{run}.invocation");
        recipe.steps.push(RecipeStep {
            id: frame.clone(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("invocation".into(), super::spec(ArtifactKind::Text))]),
            operation: RecipeOperation::WriteText {
                output: "invocation".into(),
                text,
            },
        });
        run_inputs.insert("invocation".into(), product(&frame, "invocation"));
        let mut step = exec(
            &run,
            executable.clone(),
            run_inputs,
            Vec::new(),
            BTreeMap::new(),
            outputs("out-dir", ArtifactKind::Tree),
            Some(guard),
            RecipeExitPolicy::OneOf { codes: vec![0, 75] },
            runtime.clone(),
        );
        if let RecipeOperation::Exec {
            invocation_input,
            output_seeds,
            ..
        } = &mut step.operation
        {
            *invocation_input = Some("invocation".into());
            output_seeds.insert("out-dir".into(), "initial-tmp".into());
        }
        recipe.steps.push(step);
        let observed = observe(&mut recipe.steps, &format!("{prefix}.script-observe"), &run, previous.clone());
        let classify = format!("{prefix}.classify");
        recipe.steps.push(helper_exec(
            &classify,
            helper.clone(),
            runtime_tree.clone(),
            BTreeMap::from([
                ("script-phase".into(), observed),
                ("previous".into(), state.clone()),
                ("logical-out".into(), product(&logical_id, "root")),
            ]),
            vec![
                lit("--classify"),
                input("script-phase", ""),
                output("state"),
                input("previous", ""),
                input("logical-out", ""),
            ],
            outputs("state", ArtifactKind::Tree),
            RecipeExitPolicy::OneOf { codes: vec![0, 1] },
        ));
        state = product(&classify, "state");
        let selected = format!("{prefix}.out-dir");
        recipe.steps.push(helper_exec(
            &selected,
            helper.clone(),
            runtime_tree.clone(),
            BTreeMap::from([("state".into(), state.clone())]),
            vec![lit("--select"), input("state", "out-dir"), output("selected")],
            outputs("selected", ArtifactKind::Tree),
            RecipeExitPolicy::Exact { code: 0 },
        ));
        previous = product(&selected, "selected");
        if slot == slots {
            break;
        }
        let mut command_inputs = world.inputs.clone();
        for name in ["package", "rust-runtime"] {
            command_inputs.insert(name.into(), inputs[name].clone());
        }
        let readonly_inputs = command_inputs.keys().cloned().collect::<BTreeSet<_>>();
        let mut inventory = world.inventory.clone();
        inventory.push(("rustc".into(), compiler.clone()));
        if let Some(config) = &llvm_config {
            inventory.push(("llvm-config".into(), config.clone()));
        }
        let outcome = super::super::llvm::append_native_command(
            request,
            &mut recipe,
            controller.clone(),
            controller_helper.clone(),
            super::super::llvm::NativeCommandSpec {
                id: format!("{prefix}.native"),
                captured_request: state.clone(),
                captured_request_path: "request".into(),
                whole_tmp: previous.clone(),
                inputs: command_inputs,
                readonly_inputs,
                runtime_mounts: runtime.clone(),
                inventory,
                predicates: vec![(product(&classify, "exit-status"), vec![1])],
                commands_max,
            },
        )?;
        let merge = format!("{prefix}.reply");
        recipe.steps.push(helper_exec(
            &merge,
            helper.clone(),
            runtime_tree.clone(),
            BTreeMap::from([
                ("state".into(), state.clone()),
                ("prior-replies".into(), replies),
                ("outcome".into(), outcome),
            ]),
            vec![
                lit("--merge-reply"),
                input("state", ""),
                input("prior-replies", ""),
                output("replies"),
                lit(slot.to_string()),
                lit("shared-result"),
                input("outcome", ""),
            ],
            outputs("replies", ArtifactKind::Tree),
            RecipeExitPolicy::OneOf { codes: vec![0, 1] },
        ));
        replies = product(&merge, "replies");
        pending = Some(product(&merge, "exit-status"));
    }
    let finish = format!("{id}.complete");
    let mut declared = outputs("out-dir", ArtifactKind::Tree);
    declared.insert("generated-tmp".into(), super::spec(ArtifactKind::Tree));
    recipe.steps.push(helper_exec(
        &finish,
        helper,
        runtime_tree,
        BTreeMap::from([("state".into(), state)]),
        vec![
            lit("--finish-script"),
            input("state", ""),
            output("out-dir"),
            output("generated-tmp"),
        ],
        declared,
        RecipeExitPolicy::Exact { code: 0 },
    ));
    Ok(ScriptRun {
        steps: recipe.steps,
        stdout: product(&finish, "stdout"),
        out_dir: product(&finish, "out-dir"),
        logical_out,
        generated_tmp: product(&finish, "generated-tmp"),
    })
}
