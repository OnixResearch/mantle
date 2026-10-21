//! Protected source-compiler continuations at original proc-macro boundaries.
//! Only actual compiler requests select plugin leaves; observed stdout remains
//! binary and the original compiler validates the replayed protocol and inputs.
use std::collections::BTreeMap;

use super::artifact;
use super::error;
use super::input;
use super::lit;
use super::logs;
use super::output;
use super::outputs;
use super::produced;
use super::product;
use crate::errors::RunError;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeGuard;
use crate::source_built_recipe::RecipeGuardCondition;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;

fn guard(status: &str, code: i32) -> RecipeGuard {
    RecipeGuard {
        conditions: vec![RecipeGuardCondition {
            status_input: status.into(),
            accepted_codes: vec![code],
        }],
    }
}
fn command(
    id: &str,
    helper: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    declared: BTreeMap<String, crate::source_built_recipe::RecipeOutput>,
    conditional: Option<RecipeGuard>,
    policy: RecipeExitPolicy,
) -> RecipeStep {
    inputs.insert("helper".into(), helper);
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
            guard: conditional,
            stdin_input: None,
            merge_stderr: false,
            exit_policy: policy,
        },
    }
}
fn observe(program: &mut SourceRecipe, id: &str, phase: &str) -> ArtifactRef {
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("observation".into(), artifact(ArtifactKind::Tree))]),
        operation: RecipeOperation::ObservePhase {
            phase: phase.into(),
            output: "observation".into(),
            state: None,
        },
    });
    product(id, "observation")
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
    if plugins.is_empty() || slots == 0 {
        return Err(error(
            "macro continuations require a nonempty original plugin inventory and an explicit resource reservation",
        ));
    }
    let package = compiler
        .inputs
        .get("package")
        .ok_or_else(|| error("macro compiler inputs require the real source/dependency world under package/"))?
        .clone();
    let runtime = compiler
        .inputs
        .get("rust-runtime")
        .ok_or_else(|| error("macro compiler lacks the original target runtime"))?
        .clone();
    let empty = format!("{id}.macro-empty");
    program.steps.push(command(
        &empty,
        helper.clone(),
        BTreeMap::new(),
        vec![lit("--empty"), output("tree", "")],
        outputs("tree", ArtifactKind::Tree),
        None,
        RecipeExitPolicy::Exact { code: 0 },
    ));
    let mut state = product(&empty, "tree");
    let mut history = state.clone();
    let mut tmp = captured_tmp.unwrap_or_else(|| state.clone());
    let mut resume = None;
    for slot in 0..=slots {
        let prefix = format!("{id}.macro-{slot}");
        let mut step = compiler.clone();
        step.id = format!("{prefix}.compiler");
        step.inputs.insert("macro-history".into(), history.clone());
        let RecipeOperation::Exec {
            environment,
            guard: conditional,
            exit_policy,
            ..
        } = &mut step.operation
        else {
            return Err(error("macro continuation template is not a compiler command"));
        };
        environment.insert("MRUSTC_CAIRN_MACRO_HISTORY".into(), input("macro-history", ""));
        environment.insert("MRUSTC_CAIRN_MACRO_WORK".into(), output("translation", ""));
        if let Some(status) = resume.take() {
            step.inputs.insert("resume".into(), status);
            *conditional = Some(guard("resume", 1));
        } else {
            step.inputs.insert("initial-status".into(), product(&empty, "exit-status"));
            *conditional = Some(guard("initial-status", 0));
        }
        *exit_policy = RecipeExitPolicy::OneOf { codes: vec![0, 75] };
        let compiler_id = step.id.clone();
        let step = super::bind_tmp_step(program, step, tmp.clone(), disk)?;
        program.steps.push(step);
        let observed = observe(program, &format!("{prefix}.observe-compiler"), &compiler_id);
        let classify = format!("{prefix}.classify");
        let mut inputs = BTreeMap::from([
            ("phase".into(), observed),
            ("prior".into(), state),
            ("prior-tmp".into(), tmp),
        ]);
        let mut args = vec![
            lit("--macro-classify"),
            input("phase", ""),
            input("prior", ""),
            output("state", ""),
            input("prior-tmp", ""),
            output("tmp", ""),
        ];
        for (index, (name, executable)) in plugins.iter().enumerate() {
            produced(executable)?;
            if name.is_empty() || name.contains('/') {
                return Err(error("macro inventory keys must retain exact original executable basenames"));
            }
            let alias = format!("plugin-{index}");
            inputs.insert(alias.clone(), executable.clone());
            args.extend([lit(name), input(&alias, "")]);
        }
        // package supplies the exact compiler-visible request executable path.
        inputs.insert("package".into(), package.clone());
        args.push(input("package", ""));
        let mut declared = outputs("state", ArtifactKind::Tree);
        declared.insert("tmp".into(), crate::source_built_recipe::RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: disk,
        });
        program
            .steps
            .push(command(&classify, helper.clone(), inputs, args, declared, None, RecipeExitPolicy::OneOf {
                codes: vec![0, 1],
            }));
        state = product(&classify, "state");
        tmp = product(&classify, "tmp");
        if slot == slots {
            break;
        }
        let decode = format!("{prefix}.decode");
        let mut declared = logs();
        for (name, kind) in [
            ("executable", ArtifactKind::Executable),
            ("invocation", ArtifactKind::Text),
            ("environment-data", ArtifactKind::Text),
            ("stdin", ArtifactKind::Object),
        ] {
            declared.insert(name.into(), artifact(kind));
        }
        let mut decode_inputs = BTreeMap::from([
            ("selection".into(), product(&classify, "exit-status")),
            ("state".into(), state.clone()),
            ("package".into(), package.clone()),
        ]);
        let mut decode_arguments = vec![
            lit("--macro-invocation"),
            input("state", ""),
            output("executable", ""),
            output("invocation", ""),
            output("environment-data", ""),
            output("stdin", ""),
        ];
        for (index, (name, executable)) in plugins.iter().enumerate() {
            let alias = format!("plugin-{index}");
            decode_inputs.insert(alias.clone(), executable.clone());
            decode_arguments.extend([lit(name), input(&alias, "")]);
        }
        program.steps.push(command(
            &decode,
            helper.clone(),
            decode_inputs,
            decode_arguments,
            declared,
            Some(guard("selection", 1)),
            RecipeExitPolicy::Exact { code: 0 },
        ));
        let run = format!("{prefix}.execute-macro");
        let mut declared = logs();
        declared.insert("stdout".into(), artifact(ArtifactKind::Object));
        declared.insert("generated-tmp".into(), crate::source_built_recipe::RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: disk,
        });
        program.steps.push(RecipeStep {
            id: run.clone(),
            inputs: BTreeMap::from([
                ("executable".into(), product(&decode, "executable")),
                ("invocation".into(), product(&decode, "invocation")),
                ("environment-data".into(), product(&decode, "environment-data")),
                ("stdin".into(), product(&decode, "stdin")),
                ("generated-tmp-seed".into(), tmp.clone()),
                ("selection".into(), product(&classify, "exit-status")),
                ("package".into(), package.clone()),
                ("rust-runtime".into(), runtime.clone()),
            ]),
            outputs: declared,
            operation: RecipeOperation::Exec {
                executable_input: "executable".into(),
                invocation_input: Some("invocation".into()),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
                cwd_input: None,
                cwd_output: None,
                output_seeds: BTreeMap::from([("generated-tmp".into(), "generated-tmp-seed".into())]),
                command_state: None,
                guard: Some(guard("selection", 1)),
                stdin_input: Some("stdin".into()),
                merge_stderr: false,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        });
        let observed_plugins = vec![observe(program, &format!("{prefix}.observe-macro"), &run)];
        let reply = format!("{prefix}.reply");
        let mut inputs = BTreeMap::from([
            ("state".into(), state.clone()),
            ("prior".into(), history),
            ("prior-tmp".into(), tmp),
        ]);
        let mut args = vec![
            lit("--macro-reply"),
            input("state", ""),
            input("prior", ""),
            output("history", ""),
            input("prior-tmp", ""),
            output("tmp", ""),
        ];
        for (index, phase) in observed_plugins.into_iter().enumerate() {
            let alias = format!("phase-{index}");
            inputs.insert(alias.clone(), phase);
            args.push(input(&alias, ""));
        }
        let mut declared = outputs("history", ArtifactKind::Tree);
        declared.insert("tmp".into(), crate::source_built_recipe::RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: disk,
        });
        program
            .steps
            .push(command(&reply, helper.clone(), inputs, args, declared, None, RecipeExitPolicy::OneOf {
                codes: vec![0, 1],
            }));
        history = product(&reply, "history");
        tmp = product(&reply, "tmp");
        resume = Some(product(&reply, "exit-status"));
    }
    let finish = format!("{id}.macro-finish");
    program.steps.push(command(
        &finish,
        helper,
        BTreeMap::from([("state".into(), state)]),
        vec![lit("--macro-finish"), input("state", ""), output("translation", "")],
        outputs("translation", ArtifactKind::Tree),
        None,
        RecipeExitPolicy::Exact { code: 0 },
    ));
    Ok((product(&finish, "translation"), tmp))
}
