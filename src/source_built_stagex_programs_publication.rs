//! Producer-preserving normalization and the original eight provider launches.
use super::*;
const STAGE: &str = "stagex-provider-publication";
const MEMBER: &str = "stagex-provider-publication.json";
const FILE_MAX: u64 = 64 * 1024 * 1024;
struct Publication {
    recipe: SourceRecipe,
}
impl Publication {
    fn push(&mut self, step: RecipeStep) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
        if self.recipe.steps.iter().any(|s| s.id == step.id) {
            return Err(error("duplicate publication leaf"));
        }
        let refs = step
            .outputs
            .keys()
            .map(|id| {
                (id.clone(), ArtifactRef::Produced {
                    action_id: format!("{STAGE}/{}", step.id),
                    output_id: id.clone(),
                })
            })
            .collect();
        self.recipe.steps.push(step);
        Ok(refs)
    }
    fn exec(
        &mut self,
        id: &str,
        executable: ArtifactRef,
        mut inputs: BTreeMap<String, ArtifactRef>,
        arguments: Vec<RecipeWord>,
        mut outputs: BTreeMap<String, RecipeOutput>,
        status: i32,
    ) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
        inputs.insert("executable".into(), executable);
        outputs.insert("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX));
        outputs.insert("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX));
        outputs.insert("exit-status".into(), bounded(ArtifactKind::Text, 4));
        self.push(RecipeStep {
            id: id.into(),
            inputs,
            outputs,
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: status },
                output_seeds: BTreeMap::new(),
                guard: None,
            },
        })
    }
    fn export(&mut self, name: &str, reference: ArtifactRef) {
        self.recipe.exports.insert(name.into(), reference);
    }
}
fn prefixed(prefix: &str, name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: prefix.into() }, WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn compile_args(source: &str, destination: &str) -> Vec<RecipeWord> {
    vec![
        prefixed("-B", "runtime", ""),
        literal("-nostdinc"),
        prefixed("-I", "runtime", "include"),
        prefixed("-I", "headers", ""),
        literal("-nostdlib"),
        literal("-static"),
        input("libraries", "crt1.o"),
        input("libraries", "crti.o"),
        input(source, ""),
        input("libraries", "libc.a"),
        input("runtime", "libtcc1.a"),
        input("libraries", "crtn.o"),
        literal("-o"),
        output(destination, ""),
    ]
}
fn compiler_inputs(b: &TransitionBuilder) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    Ok(BTreeMap::from([
        ("runtime".into(), b.artifact("publication-tcc-runtime")?),
        ("headers".into(), b.artifact("publication-musl-headers")?),
        ("libraries".into(), b.artifact("publication-musl-libraries")?),
    ]))
}
pub(super) fn extend(
    b: &mut TransitionBuilder,
    request: &AuthoringRequest<'_>,
    exports: &mut BTreeMap<String, ArtifactRef>,
) -> Result<(), RunError> {
    let mut p = Publication { recipe: program(STAGE) };
    let tcc = b.artifact("tcc-musl-v2")?;
    let mut inputs = compiler_inputs(b)?;
    inputs.insert("source.c".into(), b.helper_source());
    let helper = p.exec(
        "build-normalization-helper",
        tcc.clone(),
        inputs,
        compile_args("source.c", "helper"),
        BTreeMap::from([("helper".into(), bounded(ArtifactKind::Executable, FILE_MAX))]),
        0,
    )?["helper"]
        .clone();
    let transition = b
        .recipe
        .exports
        .get(TRANSITION)
        .cloned()
        .ok_or_else(|| error("missing canonical transition export"))?;
    let mut payload = BTreeMap::<String, ArtifactRef>::new();
    let mut tool_exports = BTreeMap::new();
    let mut tools = vec![
        ("tcc".to_string(), "tcc-musl-v2".to_string()),
        ("tcc-selfhost".into(), "tcc-musl-selfhost".into()),
        ("tcc-0.9.26".into(), "tinycc-0.9.26".into()),
    ];
    tools.extend(
        crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS
            .iter()
            .map(|(tool, _)| (tool.to_string(), format!("binutils-installed-{tool}"))),
    );
    for (role, original) in tools {
        let source = b.artifact(&original)?;
        // Compare against the actual canonical transition member before copying:
        // tree ancestry is consumed, not merely declared as an unused label.
        let refs = p.exec(
            &format!("publish-{role}"),
            helper.clone(),
            BTreeMap::from([("source".into(), source), ("transition".into(), transition.clone())]),
            vec![
                literal("copy-checked"),
                input("source", ""),
                output("tool", ""),
                input("transition", &format!("original/{original}")),
            ],
            BTreeMap::from([("tool".into(), bounded(ArtifactKind::Executable, FILE_MAX))]),
            0,
        )?;
        p.export(&role, refs["tool"].clone());
        tool_exports.insert(role.clone(), refs["tool"].clone());
        let suffix = if role == "tcc-0.9.26" { "tcc-0.9.26" } else { &role };
        payload.insert(format!("bin/x86_64-linux-musl-{suffix}"), refs["tool"].clone());
    }
    let trees = [
        ("musl.headers", "publication-musl-headers", "x86_64-linux-musl/include"),
        ("musl.libraries", "publication-musl-libraries", "x86_64-linux-musl/lib"),
        ("tcc.runtime", "publication-tcc-runtime", "lib/tcc"),
        ("tcc.source", "publication-tcc-source", "share/mantle-stagex/tcc-source-patched"),
        ("binutils.headers", "publication-binutils-headers", "x86_64-linux-musl/include-binutils"),
        ("binutils.ldscripts", "publication-binutils-ldscripts", "x86_64-linux-musl/lib/ldscripts"),
    ];
    let mut local_trees = BTreeMap::new();
    for (role, original, path) in trees {
        let refs = p.exec(
            &format!("publish-{}", role.replace('.', "-")),
            helper.clone(),
            BTreeMap::from([("source".into(), b.artifact(original)?)]),
            vec![literal("tree"), input("source", ""), output("tree", "")],
            BTreeMap::from([(
                "tree".into(),
                bounded(ArtifactKind::Tree, request.proof.resource_bounds.disk_bytes_max),
            )]),
            0,
        )?;
        p.export(role, refs["tree"].clone());
        payload.insert(path.into(), refs["tree"].clone());
        local_trees.insert(role.to_string(), refs["tree"].clone());
    }
    let (upstream, bounded_source) = crate::stagex_provider::publication_runtime_source_replacement();
    let original_definition = b.authored("helpers/provider-runtime-upstream.txt", upstream.into())?;
    let replacement_definition = b.authored("helpers/provider-runtime-bounded.txt", bounded_source.into())?;
    let transformed = p.exec(
        "transform-runtime-source",
        helper.clone(),
        BTreeMap::from([
            ("source".into(), b.artifact("publication-tcc-runtime-source")?),
            ("upstream".into(), original_definition),
            ("bounded".into(), replacement_definition),
        ]),
        vec![
            literal("replace-files-absent"),
            input("source", ""),
            output("source", ""),
            input("upstream", ""),
            input("bounded", ""),
            literal("1"),
        ],
        BTreeMap::from([("source".into(), bounded(ArtifactKind::Text, FILE_MAX))]),
        0,
    )?["source"]
        .clone();
    // The original runtime supplement compile remains an independently executed
    // tool command; its object is not a literal or a source bundle member.
    let runtime_object = p.exec(
        "runtime-supplement",
        tool_exports["tcc"].clone(),
        BTreeMap::from([("source.c".into(), transformed)]),
        vec![
            literal("-c"),
            input("source.c", ""),
            literal("-o"),
            output("object", ""),
        ],
        BTreeMap::from([("object".into(), bounded(ArtifactKind::Object, FILE_MAX))]),
        0,
    )?["object"]
        .clone();
    let canonical = p.exec(
        "canonicalize-runtime-supplement",
        b.artifact("elf-local-symbol-canonicalizer-copy")?,
        BTreeMap::from([("object".into(), runtime_object)]),
        vec![input("object", ""), output("object", "")],
        BTreeMap::from([("object".into(), bounded(ArtifactKind::Object, FILE_MAX))]),
        0,
    )?["object"]
        .clone();
    let runtime = p.exec(
        "assemble-complete-tcc-runtime",
        helper.clone(),
        BTreeMap::from([
            ("tree".into(), local_trees["tcc.runtime"].clone()),
            ("supplement".into(), canonical.clone()),
        ]),
        vec![
            literal("overlay"),
            input("tree", ""),
            output("tree", ""),
            literal("libtcc1-stagex-provider.o"),
            input("supplement", ""),
        ],
        BTreeMap::from([("tree".into(), bounded(ArtifactKind::Tree, request.proof.resource_bounds.disk_bytes_max))]),
        0,
    )?["tree"]
        .clone();
    payload.insert("lib/tcc".into(), runtime.clone());
    local_trees.insert("tcc.runtime".into(), runtime.clone());
    p.export("tcc.runtime", runtime);
    p.export("tcc.runtime-supplement", canonical);
    let mut smoke_sources = BTreeMap::new();
    for (name, bytes) in crate::stagex_provider::publication_validation_sources() {
        let text = std::str::from_utf8(bytes).map_err(|e| error(e.to_string()))?.to_string();
        smoke_sources.insert(name.to_string(), b.authored(&format!("validation/{name}"), text)?);
    }
    let inputs = BTreeMap::from([
        ("runtime".into(), local_trees["tcc.runtime"].clone()),
        ("headers".into(), local_trees["musl.headers"].clone()),
        ("libraries".into(), local_trees["musl.libraries"].clone()),
        ("source.c".into(), smoke_sources["positive.c"].clone()),
    ]);
    let positive = p.exec(
        "tcc-positive",
        tool_exports["tcc"].clone(),
        inputs,
        compile_args("source.c", "positive"),
        BTreeMap::from([("positive".into(), bounded(ArtifactKind::Executable, FILE_MAX))]),
        0,
    )?["positive"]
        .clone();
    p.exec("tcc-execution", positive, BTreeMap::new(), Vec::new(), BTreeMap::new(), 42)?;
    // Keep the original missing-file negative case, not a substitute malformed
    // source. /work is fresh and contains no missing-source.c or negative.o.
    p.exec(
        "tcc-negative",
        tool_exports["tcc"].clone(),
        BTreeMap::new(),
        vec![
            literal("-c"),
            literal("missing-source.c"),
            literal("-o"),
            literal("negative.o"),
        ],
        BTreeMap::new(),
        1,
    )?;
    let object = p.exec(
        "as-positive",
        tool_exports["as"].clone(),
        BTreeMap::from([("source".into(), smoke_sources["positive.s"].clone())]),
        vec![input("source", ""), literal("-o"), output("object", "")],
        BTreeMap::from([("object".into(), bounded(ArtifactKind::Object, FILE_MAX))]),
        0,
    )?["object"]
        .clone();
    let binary = p.exec(
        "ld-positive",
        tool_exports["ld"].clone(),
        BTreeMap::from([("object".into(), object)]),
        vec![input("object", ""), literal("-o"), output("positive", "")],
        BTreeMap::from([("positive".into(), bounded(ArtifactKind::Executable, FILE_MAX))]),
        0,
    )?["positive"]
        .clone();
    p.exec(
        "as-negative",
        tool_exports["as"].clone(),
        BTreeMap::from([("source".into(), smoke_sources["negative.s"].clone())]),
        vec![input("source", ""), literal("-o"), literal("negative-as.o")],
        BTreeMap::new(),
        1,
    )?;
    p.exec("positive-execution", binary, BTreeMap::new(), Vec::new(), BTreeMap::new(), 42)?;
    // Source-controller consumers need the genuine writer/Make variants, while
    // the normalized provider above remains the unchanged raw tool authority.
    let mut phase_payload = payload.clone();
    let entries = payload.keys().enumerate().map(|(i, path)| (path.clone(), format!("payload-{i}"))).collect();
    let inputs = payload
        .into_values()
        .enumerate()
        .map(|(i, reference)| (format!("payload-{i}"), reference))
        .collect();
    let refs = p.push(RecipeStep {
        id: "canonical-provider".into(),
        inputs,
        outputs: BTreeMap::from([(
            "stagex-provider".into(),
            bounded(ArtifactKind::Tree, request.proof.resource_bounds.disk_bytes_max),
        )]),
        operation: RecipeOperation::AssembleTree {
            output: "stagex-provider".into(),
            entries,
        },
    })?;
    p.export("stagex-provider", refs["stagex-provider"].clone());
    for (role, original, path) in [
        ("tcc", "phase-tool:tcc-musl-v2", "bin/x86_64-linux-musl-tcc"),
        ("ld", "phase-tool:binutils-ld", "bin/x86_64-linux-musl-ld"),
        ("make", "phase-tool:make-3.82", "make-stage/runtime/output/bin/make"),
    ] {
        let value = p.exec(
            &format!("publish-phase-{role}"),
            helper.clone(),
            BTreeMap::from([
                ("source".into(), b.artifact(original)?),
                ("transition".into(), transition.clone()),
            ]),
            vec![
                literal("copy-checked"),
                input("source", ""),
                output("tool", ""),
                input("transition", &format!("original/{original}")),
            ],
            BTreeMap::from([("tool".into(), bounded(ArtifactKind::Executable, FILE_MAX))]),
            0,
        )?["tool"]
            .clone();
        phase_payload.insert(path.into(), value);
    }
    let entries = phase_payload.keys().enumerate().map(|(i, path)| (path.clone(), format!("phase-{i}"))).collect();
    let inputs = phase_payload
        .into_values()
        .enumerate()
        .map(|(i, reference)| (format!("phase-{i}"), reference))
        .collect();
    let phase = p.push(RecipeStep {
        id: "canonical-phase-runtime".into(),
        inputs,
        outputs: BTreeMap::from([(
            "phase-runtime".into(),
            bounded(ArtifactKind::Tree, request.proof.resource_bounds.disk_bytes_max),
        )]),
        operation: RecipeOperation::AssembleTree {
            output: "phase-runtime".into(),
            entries,
        },
    })?;
    p.export("phase-runtime", phase["phase-runtime"].clone());
    // All eight launches and their actual status/stream artifacts remain roots,
    // including negative commands whose expected behavior creates no object.
    p.recipe.roots = p
        .recipe
        .steps
        .iter()
        .flat_map(|step| {
            step.outputs.keys().map(|output| ArtifactRef::Produced {
                action_id: format!("{STAGE}/{}", step.id),
                output_id: output.clone(),
            })
        })
        .collect();
    recipe::validate_source_recipe(&p.recipe)?;
    let validation_commands = crate::stagex_provider::provider_validation_launches()
        .into_iter()
        .map(|(id, _)| (id, id))
        .collect::<BTreeMap<_, _>>();
    let index =
        serde_json::json!({"schema":INDEX_SCHEMA,"recipe_member":MEMBER,"validation_commands":validation_commands});
    b.files.insert(
        format!("{MEMBER}.expansion.json"),
        serde_json::to_string_pretty(&index).map_err(|e| error(e.to_string()))?,
    );
    b.files
        .insert(MEMBER.into(), serde_json::to_string_pretty(&p.recipe).map_err(|e| error(e.to_string()))?);
    exports.extend(p.recipe.exports.into_iter().map(|(name, reference)| (format!("{STAGE}:{name}"), reference)));
    Ok(())
}
