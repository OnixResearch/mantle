use super::*;
use crate::source_built_recipe::ArtifactBinding;
use crate::source_built_recipe::decode_invocation;

const BYTES_MAX: u64 = 1024 * 1024;

fn recipe() -> SourceRecipe {
    SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "captured workspace fixture".into(),
            provenance: "original typed compiler environment".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    }
}

fn compiler(out_dir: RecipeWord) -> RecipeStep {
    RecipeStep {
        id: "library.translate".into(),
        inputs: BTreeMap::from([
            ("compiler".into(), product(STAGE, "producer", "compiler")),
            ("package".into(), product(STAGE, "producer", "package")),
        ]),
        outputs: outputs("translation", ArtifactKind::Tree),
        operation: RecipeOperation::Exec {
            executable_input: "compiler".into(),
            arguments: vec![input("package", "src/lib.rs")],
            environment: BTreeMap::from([("OUT_DIR".into(), out_dir)]),
            runtime_mounts: BTreeMap::new(),
            cwd_input: Some("package".into()),
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            merge_stderr: false,
            invocation_input: None,
        },
    }
}

fn captured_invocation(mount: &str, path: &str) -> Result<RecipeInvocation, RunError> {
    let mut program = recipe();
    let original = compiler(RecipeWord {
        parts: vec![WordPart::Workspace {
            mount: mount.into(),
            path: path.into(),
        }],
    });
    let step = bind_tmp_step(&mut program, original, product(STAGE, "captured", "tree"), BYTES_MAX)?;
    let RecipeOperation::WriteText { text, .. } = &program.steps[0].operation else {
        panic!("captured workspace did not emit its canonical invocation");
    };
    let bindings = step
        .inputs
        .iter()
        .map(|(name, reference)| {
            let kind = match name.as_str() {
                "compiler" => ArtifactKind::Executable,
                "workspace-invocation" => ArtifactKind::Text,
                _ => ArtifactKind::Tree,
            };
            (reference.clone(), ArtifactBinding {
                reference: reference.clone(),
                path: std::path::PathBuf::from(name),
                kind,
                bytes_max: BYTES_MAX,
            })
        })
        .collect::<BTreeMap<_, _>>();
    decode_invocation(&step, text.as_bytes(), &bindings)
}

#[test]
fn captured_tmp_request_preserves_descendant_out_dir_and_rejects_unbound_workspace() {
    let invocation = captured_invocation("/tmp/mantle-rust/original-script/out", "generated").unwrap();
    assert_eq!(invocation.environment["OUT_DIR"].parts, vec![WordPart::Workspace {
        mount: "/tmp".into(),
        path: "mantle-rust/original-script/out/generated".into(),
    }]);
    assert!(captured_invocation("/var/tmp/original-script/out", "").is_err());
}

#[test]
fn captured_tmp_request_rejects_traversal_in_workspace_root_or_member() {
    for (mount, path) in [("/tmp/../escape", ""), ("/tmp/original-script/out", "../escape")] {
        assert!(captured_invocation(mount, path).is_err());
    }
}
