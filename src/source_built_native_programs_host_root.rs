//! Evaluate the selected original shell node with explicit source command leaves.
use std::collections::BTreeMap;

use super::fail;
use super::input;
use super::lit;
use super::output;
use super::produced;
use crate::errors::RunError;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::STAGE;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeTimeoutGroup;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_stagex_programs::controller::ControllerExecutable;
use crate::source_built_stagex_programs::controller::ControllerSlot;
use crate::source_built_stagex_programs::controller::ControllerSpec;
use crate::source_built_stagex_programs::controller::append_controller;

type Files = BTreeMap<String, ArtifactRef>;
/// All tools here are already real, individually Produced original images.
/// Source patches are preparation steps, never a completed original node.
pub(crate) struct OriginalShellNode {
    pub namespace: String,
    pub script: Option<String>,
    pub controller: ArtifactRef,
    pub projector: ArtifactRef,
    pub tools: Vec<(String, ArtifactRef)>,
    pub command_inputs: Files,
    pub runtime_mounts: BTreeMap<String, String>,
    pub source_overrides: BTreeMap<String, ArtifactRef>,
    pub members: BTreeMap<String, BTreeMap<String, ArtifactKind>>,
    /// Capability name -> (original selected output, authentic installed member).
    pub phase_members: BTreeMap<String, (String, String)>,
    pub steps: Vec<RecipeStep>,
    pub timeout_groups: Vec<RecipeTimeoutGroup>,
    pub files: BTreeMap<String, String>,
    pub commands_max: usize,
}
fn netstring(out: &mut String, value: &str) {
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push(',');
}
fn text(recipe: &mut SourceRecipe, id: String, value: String) -> ArtifactRef {
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs: Files::new(),
        outputs: BTreeMap::from([("result".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 64 * 1024 * 1024,
        })]),
        operation: RecipeOperation::WriteText {
            output: "result".into(),
            text: value,
        },
    });
    produced(&id, "result")
}
fn tree(recipe: &mut SourceRecipe, id: String, entries: Files, bytes_max: u64) -> ArtifactRef {
    let mut inputs = Files::new();
    let mut names = BTreeMap::new();
    for (index, (path, reference)) in entries.into_iter().enumerate() {
        let name = format!("member-{index}");
        inputs.insert(name.clone(), reference);
        names.insert(path, name);
    }
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([("result".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max,
        })]),
        operation: RecipeOperation::AssembleTree {
            output: "result".into(),
            entries: names,
        },
    });
    produced(&id, "result")
}
fn project(
    recipe: &mut SourceRecipe,
    id: String,
    projector: &ArtifactRef,
    from: ArtifactRef,
    path: &str,
    kind: ArtifactKind,
    bytes_max: u64,
) -> ArtifactRef {
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs: Files::from([("tool".into(), projector.clone()), ("tree".into(), from)]),
        outputs: BTreeMap::from([("result".into(), RecipeOutput { kind, bytes_max })]),
        operation: RecipeOperation::Exec {
            executable_input: "tool".into(),
            arguments: vec![lit("copy"), input("tree", path), output("result")],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    produced(&id, "result")
}
fn project_executable(
    recipe: &mut SourceRecipe,
    id: String,
    controller: &ArtifactRef,
    from: ArtifactRef,
    catalog: &ArtifactRef,
    path: &str,
    bytes_max: u64,
) -> ArtifactRef {
    recipe.steps.push(RecipeStep {
        id: id.clone(),
        inputs: Files::from([
            ("tool".into(), controller.clone()),
            ("tree".into(), from),
            ("catalog".into(), catalog.clone()),
        ]),
        outputs: BTreeMap::from([("result".into(), RecipeOutput {
            kind: ArtifactKind::Executable,
            bytes_max,
        })]),
        operation: RecipeOperation::Exec {
            executable_input: "tool".into(),
            arguments: vec![
                lit("--mantle-native-installed"),
                input("tree", path),
                input("catalog", ""),
                output("result"),
            ],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            merge_stderr: false,
            invocation_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
        },
    });
    produced(&id, "result")
}
/// Preserve the evaluated original script, environment, dependency roots and
/// complete installed trees. Every compiler/utility/generator is selected from
/// a finite inventory or earlier isolated actual compiler/link output.
pub(crate) fn lower_original_shell_node(
    request: &NativeNodeRequest<'_, '_>,
    mut plan: OriginalShellNode,
) -> Result<NativeNodeProgram, RunError> {
    if request.node.derivation.builder != "/bin/sh" {
        return Err(fail("original shell author received a non-shell builder"));
    }
    let arguments = &request.node.derivation.arguments;
    if arguments.len() < 2 || arguments[0] != "-c" {
        return Err(fail("original shell builder does not expose its script argument"));
    }
    if plan.tools.is_empty() || plan.commands_max == 0 {
        return Err(fail("original shell node lacks actual source tools or request bound"));
    }
    let disk = request.request.proof.resource_bounds.disk_bytes_max;
    let base = format!("{}.host.original{}", request.id, plan.namespace);
    let mut recipe = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "original native source shell author".into(),
            provenance: "evaluated original derivation arguments/environment and exact source/dependency trees".into(),
        },
        steps: std::mem::take(&mut plan.steps),
        timeout_groups: std::mem::take(&mut plan.timeout_groups),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    let script =
        text(&mut recipe, format!("{base}.script"), plan.script.take().unwrap_or_else(|| arguments[1].clone()));
    let scripts = tree(&mut recipe, format!("{base}.scripts"), Files::from([("recipe.sh".into(), script)]), disk);
    let empty = tree(&mut recipe, format!("{base}.empty-directory"), Files::new(), disk);
    let mut initial_entries = Files::from([("tmp".into(), empty.clone())]);
    for name in request.node.derivation.outputs.keys() {
        initial_entries.insert(format!("outputs/{name}"), empty.clone());
    }
    let initial = tree(&mut recipe, format!("{base}.initial"), initial_entries, disk);
    let initial_cwd = text(&mut recipe, format!("{base}.initial-cwd"), "/tmp".into());
    let original_zero = text(
        &mut recipe,
        format!("{base}.original-zero"),
        arguments.get(2).cloned().unwrap_or_else(|| "/bin/sh".into()),
    );
    let mut environment = String::new();
    for (key, value) in &request.node.derivation.environment {
        if request.node.derivation.outputs.contains_key(key) {
            continue;
        }
        let value = std::str::from_utf8(value.as_ref())
            .map_err(|_| fail(format!("non-UTF8 original shell environment {key}")))?;
        netstring(&mut environment, key);
        netstring(&mut environment, value);
    }
    if !request.node.derivation.environment.contains_key("NIX_STORE") {
        netstring(&mut environment, "NIX_STORE");
        netstring(&mut environment, &request.request.proof.logical_store_prefix);
    }
    // These are interpreter-entry metadata, not replacement configure answers.
    netstring(&mut environment, "MANTLE_CONTROLLER_ROOTS");
    netstring(&mut environment, "source-owned-manifest");
    let mut roots = String::new();
    netstring(&mut roots, "MANTLE-CONTROLLER-ROOTS-1");
    for value in ["mutable", "/tmp", "tmp"] {
        netstring(&mut roots, value);
    }
    let mut output_paths = BTreeMap::new();
    let drv = request.node.drv_path.to_absolute_path_with_prefix(&request.request.proof.logical_store_prefix);
    for name in request.node.derivation.outputs.keys() {
        let logical = request
            .logical_outputs
            .get(&format!("{drv}:{name}"))
            .ok_or_else(|| fail(format!("missing exact original output {drv}:{name}")))?;
        netstring(&mut environment, name);
        netstring(&mut environment, logical);
        let relative = format!("outputs/{name}");
        for value in ["mutable", logical.as_str(), relative.as_str()] {
            netstring(&mut roots, value);
        }
        output_paths.insert(name.clone(), relative);
    }
    let mut runtime_mounts = std::mem::take(&mut plan.runtime_mounts);
    let mut command_inputs = std::mem::take(&mut plan.command_inputs);
    let mut seen = BTreeMap::<String, ArtifactRef>::new();
    for (dependency, program) in request.dependencies {
        for (name, reference) in &program.outputs {
            let logical = request
                .logical_outputs
                .get(&format!("{dependency}:{name}"))
                .ok_or_else(|| fail(format!("missing exact original dependency output {dependency}:{name}")))?;
            let reference = plan.source_overrides.get(logical).unwrap_or(reference).clone();
            if let Some(old) = seen.insert(logical.clone(), reference.clone())
                && old != reference
            {
                return Err(fail("original runtime root has conflicting source producers"));
            }
            let key = format!("native-runtime-{}", seen.len());
            if command_inputs.insert(key.clone(), reference).is_some() {
                return Err(fail("native command input collides with original dependency"));
            }
            if runtime_mounts.insert(logical.clone(), key).is_some() {
                return Err(fail("native original runtime root declared twice"));
            }
        }
    }
    for logical in plan.source_overrides.keys() {
        if !seen.contains_key(logical) {
            return Err(fail(format!("source override is not an original declared dependency {logical}")));
        }
    }
    for root in runtime_mounts.keys() {
        for value in ["runtime", root.as_str(), root.as_str()] {
            netstring(&mut roots, value);
        }
    }
    let roots = text(&mut recipe, format!("{base}.roots"), roots);
    if command_inputs.insert("command-roots".into(), roots.clone()).is_some() {
        return Err(fail("reserved command-roots binding collision"));
    }
    let environment = text(&mut recipe, format!("{base}.environment"), environment);
    let mut inputs = Files::from([
        ("scripts".into(), scripts),
        ("initial-environment".into(), environment),
        ("initial-workspace".into(), initial),
        ("controller-roots".into(), roots),
        ("initial-cwd".into(), initial_cwd),
        ("original-zero".into(), original_zero),
    ]);
    let mut start_arguments = vec![
        lit("--mantle-source-shell"),
        lit("run"),
        input("scripts", ""),
        lit("recipe.sh"),
        input("initial-environment", ""),
        lit("-"),
        lit("-"),
        input("initial-workspace", ""),
        output("checkpoint"),
    ];
    for (index, arg) in arguments.iter().skip(3).enumerate() {
        let name = format!("original-argument-{index}");
        let value = text(&mut recipe, format!("{base}.{name}"), arg.clone());
        inputs.insert(name.clone(), value);
        start_arguments.push(crate::source_built_recipe::RecipeWord {
            parts: vec![crate::source_built_recipe::WordPart::TextInput { name }],
        });
    }
    let inventory = "native-tools".to_string();
    let slots = vec![
        ControllerSlot {
            id: inventory.clone(),
            executable: ControllerExecutable::Inventory(plan.tools),
            inputs: command_inputs.clone(),
            runtime_mounts: runtime_mounts.clone(),
            exit_policy: RecipeExitPolicy::OneOf {
                codes: (0..=255).collect(),
            },
            merge_stderr: false,
            generated_outputs: true,
        },
        ControllerSlot {
            id: "native-generated".into(),
            executable: ControllerExecutable::RequestOutput {
                producer_slots: vec![inventory],
            },
            inputs: command_inputs,
            runtime_mounts,
            exit_policy: RecipeExitPolicy::OneOf {
                codes: (0..=255).collect(),
            },
            merge_stderr: false,
            generated_outputs: true,
        },
    ];
    let controller_environment = BTreeMap::from([
        ("MANTLE_CONTROLLER_ROOTS".into(), input("controller-roots", "")),
        ("MANTLE_CONTROLLER_INITIAL_CWD".into(), input("initial-cwd", "")),
        ("MANTLE_SHELL_ORIGINAL_ZERO".into(), input("original-zero", "")),
        ("MANTLE_NATIVE_SCOPE_ROOT".into(), lit(base.clone())),
    ]);
    let result = append_controller(&mut recipe, ControllerSpec {
        id: format!("{base}.controller"),
        controller: plan.controller.clone(),
        inputs,
        start_arguments,
        environment: controller_environment,
        controller_runtime_mounts: BTreeMap::new(),
        restart_indices: [5, 6, 8],
        slots,
        commands_max: plan.commands_max,
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        file_bytes_max: disk.min(64 * 1024 * 1024),
        tree_bytes_max: disk,
    })?;
    let mut outputs = BTreeMap::new();
    let mut members = BTreeMap::new();
    for (name, path) in output_paths {
        let installed = project(
            &mut recipe,
            format!("{base}.install-{name}"),
            &plan.projector,
            result.workspace.clone(),
            &path,
            ArtifactKind::Tree,
            disk,
        );
        let mut selected = Files::new();
        if let Some(requested) = plan.members.remove(&name) {
            for (index, (path, kind)) in requested.into_iter().enumerate() {
                let id = format!("{base}.member-{name}-{index}");
                let reference = if matches!(kind, ArtifactKind::Executable) {
                    project_executable(
                        &mut recipe,
                        id,
                        &plan.controller,
                        installed.clone(),
                        &result.generated,
                        &path,
                        disk,
                    )
                } else {
                    project(&mut recipe, id, &plan.projector, installed.clone(), &path, kind, disk)
                };
                selected.insert(path, reference);
            }
        }
        outputs.insert(name.clone(), installed);
        members.insert(name, selected);
    }
    if !plan.members.is_empty() {
        return Err(fail("member projection names an absent original output"));
    }
    let mut phase_tools = Files::new();
    for (capability, (name, path)) in plan.phase_members {
        let installed = outputs
            .get(&name)
            .ok_or_else(|| fail(format!("phase capability names absent original output {name}")))?
            .clone();
        let reference = if path.is_empty() {
            installed
        } else if let Some(reference) = members.get(&name).and_then(|files| files.get(&path)) {
            reference.clone()
        } else {
            let reference = project_executable(
                &mut recipe,
                format!("{base}.capability-{capability}"),
                &plan.controller,
                installed,
                &result.generated,
                &path,
                disk,
            );
            members.get_mut(&name).expect("known original output").insert(path, reference.clone());
            reference
        };
        phase_tools.insert(capability, reference);
    }
    Ok(NativeNodeProgram {
        steps: recipe.steps,
        timeout_groups: recipe.timeout_groups,
        outputs,
        members,
        phase_tools,
        files: plan.files,
    })
}
