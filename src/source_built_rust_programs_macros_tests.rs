//! Behavioral coverage of compact authoring, without executing source programs.
use super::*;
use crate::source_built_recipe::RecipePreparation;

const DISK: u64 = 64 * 1024 * 1024;

fn reference(name: &str) -> ArtifactRef {
    product("stagex-transition", name, "artifact")
}

fn recipe() -> SourceRecipe {
    SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: super::super::STAGE.into(),
        preparation: RecipePreparation {
            tool: "macro-authoring-test".into(),
            provenance: "typed source compiler fixture".into(),
        },
        steps: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
        timeout_groups: Vec::new(),
    }
}

fn compiler() -> RecipeStep {
    RecipeStep {
        id: "crate.translate".into(),
        inputs: BTreeMap::from([
            ("mrustc".into(), reference("mrustc")),
            ("package".into(), reference("package")),
            ("rust-runtime".into(), reference("runtime")),
            ("metadata".into(), reference("metadata")),
        ]),
        outputs: outputs("translation", ArtifactKind::Tree),
        operation: RecipeOperation::Exec {
            executable_input: "mrustc".into(),
            invocation_input: None,
            arguments: vec![
                input("package", "src/lib.rs"),
                lit("-o"),
                output("translation", "libcrate.rlib"),
            ],
            environment: BTreeMap::from([("MRUSTC_CAIRN_ARGUMENTS".into(), RecipeWord {
                parts: vec![WordPart::TextInput {
                    name: "metadata".into(),
                }],
            })]),
            runtime_mounts: BTreeMap::from([("/mantle-rust-runtime".into(), "rust-runtime".into())]),
            cwd_input: Some("package".into()),
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

fn append_fixture(
    recipe: &mut SourceRecipe,
    compiler: RecipeStep,
    slots: usize,
    tmp: Option<ArtifactRef>,
) -> Result<(ArtifactRef, ArtifactRef), RunError> {
    append(
        recipe,
        "crate",
        compiler,
        reference("helper"),
        &BTreeMap::from([("derive-original".into(), reference("plugin"))]),
        slots,
        tmp,
        DISK,
    )
}

fn specification(recipe: &SourceRecipe) -> &ControllerSpec {
    recipe
        .steps
        .iter()
        .find_map(|step| match &step.operation {
            RecipeOperation::Controller { specification, .. } => Some(specification),
            _ => None,
        })
        .expect("compact controller declaration")
}

fn config(recipe: &SourceRecipe) -> RecipeInvocation {
    let text = recipe
        .steps
        .iter()
        .find_map(|step| match &step.operation {
            RecipeOperation::WriteText { text, .. } => Some(text),
            _ => None,
        })
        .expect("normalized compiler invocation");
    serde_json::from_str(text.split_once('\n').unwrap().0).unwrap()
}

#[test]
fn macro_5612_slots_use_constant_declarations_and_exact_images() {
    let mut program = recipe();
    let (translation, tmp) = append_fixture(&mut program, compiler(), 5612, None).unwrap();
    assert_eq!(program.steps.len(), DECLARATION_STEPS_MAX);
    let spec = specification(&program);
    assert_eq!(spec.commands_max, 11225);
    assert_eq!(spec.restart_indices, [1, 2, 3]);
    assert_eq!(spec.start_arguments, vec![
        lit("--macro-control"),
        lit("-"),
        lit("-"),
        output("checkpoint", ""),
        input("compiler-config", ""),
        input("initial-tmp", ""),
        lit("5612"),
        lit("derive-original"),
        input("plugin-0", ""),
    ]);
    assert_eq!(spec.slots.iter().map(|slot| slot.id.as_str()).collect::<Vec<_>>(), ["compiler", "macro"]);
    assert_eq!(spec.slots[0].executable, ControllerExecutable::Fixed(reference("mrustc")));
    assert_eq!(
        spec.slots[1].executable,
        ControllerExecutable::Inventory(vec![("derive-original".into(), reference("plugin"))])
    );
    assert_eq!(spec.slots[0].exit_policy, RecipeExitPolicy::OneOf { codes: vec![0, 75] });
    assert_eq!(spec.slots[1].exit_policy, RecipeExitPolicy::Exact { code: 0 });
    assert!(spec.slots.iter().all(|slot| !slot.generated_outputs));
    assert!(!spec.slots[0].inputs.contains_key("mrustc"));
    assert_eq!(spec.slots[0].inputs["metadata"], reference("metadata"));
    assert!(!spec.slots[1].inputs.contains_key("metadata"));
    assert_eq!(spec.slots[1].inputs["package"], reference("package"));
    assert_eq!(spec.slots[1].runtime_mounts, spec.slots[0].runtime_mounts);
    assert!(spec.slots.iter().all(|slot| slot.inputs.keys().all(|name| !spec.inputs.contains_key(name))));
    assert_eq!(translation, product(&program.stage_id, "crate.macro-translation", "tree"));
    assert_eq!(tmp, product(&program.stage_id, "crate.macro-tmp", "tree"));
    for (step, member) in program.steps.iter().rev().take(2).zip(["tmp", "translation"]) {
        let RecipeOperation::Exec { arguments, .. } = &step.operation else {
            panic!("normal extraction leaf")
        };
        assert_eq!(arguments, &vec![lit("--select"), input("workspace", member), output("tree", "")]);
    }
    let templates =
        crate::source_built_stagex_programs::controller::template_recipe("controller-validation", spec, None).unwrap();
    assert_eq!(templates.steps.len(), 10);
    let mut small = recipe();
    append_fixture(&mut small, compiler(), 1, None).unwrap();
    assert_eq!(small.steps.len(), program.steps.len());
}

#[test]
fn macro_reservations_obey_command_ceiling_not_expansion_size() {
    let max = super::super::macro_reservation_slots_max(DECLARATION_STEPS_MAX).unwrap();
    assert!(max >= 5612);
    assert_eq!(super::super::macro_reservation_slots_max(crate::source_built_recipe::STEP_COUNT_MAX).unwrap(), max);
    assert!(super::super::macro_reservation_slots_max(DECLARATION_STEPS_MAX - 1).is_err());
    assert!(commands_max(max as usize).is_ok());
    for slots in [0, max as usize + 1, usize::MAX] {
        let mut program = recipe();
        assert!(append_fixture(&mut program, compiler(), slots, None).is_err());
        assert!(program.steps.is_empty());
    }
    let mut program = recipe();
    append_fixture(&mut program, compiler(), max as usize, Some(reference("prior-tmp"))).unwrap();
    assert_eq!(program.steps.len(), DECLARATION_STEPS_MAX - 1);
    assert_eq!(specification(&program).inputs["initial-tmp"], reference("prior-tmp"));
}

#[test]
fn macro_graphs_admit_actual_stage_roots_and_reject_cross_stage_roots() {
    for stage in ["full-source-rust-provider", "mantle-stage1", "mantle-stage2"] {
        let mut program = recipe();
        program.stage_id = stage.into();
        let (translation, tmp) = append_fixture(&mut program, compiler(), 1, None).unwrap();
        program.roots = vec![translation, tmp];
        crate::source_built_recipe::validate_source_recipe(&program).unwrap();
        program.roots[0] = product("stagex-transition", "crate.macro-translation", "tree");
        assert!(crate::source_built_recipe::validate_source_recipe(&program).is_err());
    }
}

#[test]
fn macro_compiler_config_preserves_typed_operands_environment_cwd_and_metadata() {
    let mut original = compiler();
    let RecipeOperation::Exec {
        arguments, environment, ..
    } = &mut original.operation
    else {
        unreachable!()
    };
    let user_data = RecipeWord {
        parts: vec![WordPart::LiteralData {
            text: "translation/file /outputs/translation -- untouched".into(),
        }],
    };
    let attached = RecipeWord {
        parts: vec![
            WordPart::Literal {
                text: "emit-build-command=".into(),
            },
            WordPart::Output {
                name: "translation".into(),
                path: "nested/original-codegen.sh".into(),
            },
        ],
    };
    arguments.extend([attached, user_data.clone(), RecipeWord {
        parts: vec![WordPart::TextSlice {
            name: "metadata".into(),
            offset_bytes: 7,
            length_bytes: 12,
        }],
    }]);
    environment.insert("USER_DATA".into(), user_data.clone());
    environment.insert("OUTPUT_DIR".into(), output("translation", "nested"));
    environment.insert("RUNTIME".into(), RecipeWord {
        parts: vec![WordPart::Runtime {
            mount: "/mantle-rust-runtime".into(),
            path: "lib/libstd.hir".into(),
        }],
    });
    let mut program = recipe();
    append_fixture(&mut program, original, 2, Some(reference("prior-tmp"))).unwrap();
    let invocation = config(&program);
    assert_eq!(invocation.arguments[0], input("package", "src/lib.rs"));
    assert_eq!(invocation.arguments[2], output("workspace", "translation/libcrate.rlib"));
    assert_eq!(invocation.arguments[3].parts, vec![
        WordPart::Literal {
            text: "emit-build-command=".into()
        },
        WordPart::Output {
            name: "workspace".into(),
            path: "translation/nested/original-codegen.sh".into()
        },
    ]);
    assert_eq!(invocation.arguments[4], user_data);
    assert_eq!(invocation.arguments[5].parts, vec![WordPart::TextSlice {
        name: "metadata".into(),
        offset_bytes: 7,
        length_bytes: 12
    }]);
    assert_eq!(invocation.environment["USER_DATA"], user_data);
    assert_eq!(invocation.environment["OUTPUT_DIR"], output("workspace", "translation/nested"));
    assert_eq!(invocation.environment["MRUSTC_CAIRN_ARGUMENTS"].parts, vec![WordPart::TextInput {
        name: "metadata".into()
    }]);
    assert_eq!(invocation.environment["MRUSTC_CAIRN_MACRO_HISTORY"], input("workspace-seed", "history"));
    assert_eq!(invocation.environment["MRUSTC_CAIRN_MACRO_WORK"], output("workspace", "translation"));
    assert_eq!(invocation.cwd, Some(input("package", "")));
    assert_eq!(
        invocation.workspace_mounts,
        BTreeMap::from([("/tmp".into(), RecipeWorkspaceMount {
            output: "workspace".into(),
            path: "tmp".into()
        })])
    );
    assert_eq!(invocation.readonly_input_roots, ["package", "rust-runtime"]);
}

#[test]
fn macro_compiler_output_cwd_and_referenced_executable_alias_remain_typed() {
    let mut original = compiler();
    let RecipeOperation::Exec {
        arguments,
        cwd_input,
        cwd_output,
        ..
    } = &mut original.operation
    else {
        unreachable!()
    };
    arguments.push(input("mrustc", ""));
    *cwd_input = None;
    *cwd_output = Some("translation".into());
    let (slot, invocation, _) = compiler_slot(original).unwrap();
    assert_eq!(invocation.cwd, Some(output("workspace", "translation")));
    assert_eq!(slot.inputs["mrustc"], reference("mrustc"));
    assert_eq!(invocation.arguments.last(), Some(&input("mrustc", "")));
}

#[test]
fn macro_compiler_rejects_conflicting_continuation_state() {
    for mode in [
        "invocation",
        "guard",
        "seeds",
        "state",
        "stdin",
        "history",
        "work",
        "cwd",
        "tmp",
        "output",
    ] {
        let mut original = compiler();
        let RecipeOperation::Exec {
            invocation_input,
            guard,
            output_seeds,
            command_state,
            stdin_input,
            environment,
            cwd_output,
            runtime_mounts,
            arguments,
            ..
        } = &mut original.operation
        else {
            unreachable!()
        };
        match mode {
            "invocation" => *invocation_input = Some("metadata".into()),
            "guard" => *guard = Some(crate::source_built_recipe::RecipeGuard { conditions: Vec::new() }),
            "seeds" => {
                output_seeds.insert("translation".into(), "package".into());
            }
            "state" => {
                *command_state = Some(crate::source_built_recipe::RecipeCommandState {
                    workspace_output: "translation".into(),
                    prior_times_input: None,
                    times_output: "times".into(),
                    descriptors_output: "descriptors".into(),
                    descriptor_files_output: None,
                })
            }
            "stdin" => *stdin_input = Some("metadata".into()),
            "history" => {
                environment.insert("MRUSTC_CAIRN_MACRO_HISTORY".into(), lit("spoofed"));
            }
            "work" => {
                environment.insert("MRUSTC_CAIRN_MACRO_WORK".into(), lit("spoofed"));
            }
            "cwd" => *cwd_output = Some("translation".into()),
            "tmp" => {
                runtime_mounts.insert("/tmp".into(), "package".into());
            }
            "output" => arguments.push(output("other", "file")),
            _ => unreachable!(),
        }
        let mut program = recipe();
        assert!(append_fixture(&mut program, original, 1, None).is_err(), "{mode}");
        assert!(program.steps.is_empty(), "{mode}");
    }
}

#[test]
fn macro_inventory_rejects_non_basenames_and_non_produced_images() {
    let source = ArtifactRef::Source {
        role: crate::source_built_fixed_point::SourceAuthorityRole::StagexSourceBundle,
        path: "plugin".into(),
    };
    for (name, image) in [
        ("", reference("plugin")),
        ("dir/plugin", reference("plugin")),
        ("..", reference("plugin")),
        ("plugin", source),
    ] {
        let mut program = recipe();
        assert!(
            append(
                &mut program,
                "crate",
                compiler(),
                reference("helper"),
                &BTreeMap::from([(name.into(), image)]),
                1,
                None,
                DISK
            )
            .is_err()
        );
        assert!(program.steps.is_empty());
    }
}
