//! Original-generator leaves and source-produced compiler metadata consumption.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::errors::RunError;
use crate::rust_plan::RustUnitDerivationSummary;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeInvocation;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::error;

#[path = "source_built_mantle_programs_metadata_helper.rs"]
mod helper;

/// Compile this source through the same source-produced compiler/native linker
/// route as any other helper. Its image, never a host interpreter, is an input.
pub(crate) fn metadata_helper_source_with_native(cc_process_source: &str, native_request_source: &str) -> String {
    helper::source(cc_process_source, Some(native_request_source))
}

/// Shared pure-I/O request/reply and absent-program module for library overlays.
pub(crate) fn process_replay_source(cc_process_source: &str) -> String {
    helper::process_source(cc_process_source)
}
pub(crate) const METADATA_HELPER_PATH: &str = "source_built_mantle_metadata_helper.rs";
pub(crate) const COMPILE_ENVIRONMENT_VARIABLE: &str = "MANTLE_CAIRN_COMPILE_ENV";
pub(crate) const SCRIPT_ENVIRONMENT_VARIABLE: &str = "MANTLE_CAIRN_SCRIPT_ENV";
pub(crate) const SOURCE_OVERLAY_VARIABLE: &str = "MANTLE_CAIRN_SOURCE_OVERLAY";

fn literal(text: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    }
}
fn input(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn output(name: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: String::new(),
        }],
    }
}
fn produced(stage: &str, id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{stage}/{id}"),
        output_id: name.into(),
    }
}
fn spec(kind: ArtifactKind, bytes_max: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max }
}
fn require_produced(reference: &ArtifactRef, what: &str) -> Result<(), RunError> {
    if !matches!(reference, ArtifactRef::Produced { .. }) {
        return Err(error(format!("{what} must have compiled source ancestry")));
    }
    Ok(())
}
fn insert(inputs: &mut BTreeMap<String, ArtifactRef>, name: &str, reference: ArtifactRef) -> Result<(), RunError> {
    if inputs.contains_key(name) {
        return Err(error(format!("metadata input name collision: {name}")));
    }
    inputs.insert(name.into(), reference);
    Ok(())
}

pub(crate) struct ScriptRun {
    pub(crate) step: RecipeStep,
    pub(crate) stdout: ArtifactRef,
    pub(crate) out_dir: ArtifactRef,
}

/// Exactly the topology's original build-script process. The caller must bind
/// the original compiled script, not a metadata simulator or launcher. Native
/// compiler/probe child paths must have been split into real source-owned
/// phases before this leaf can execute under the no-child policy.
///
/// Compile the original script with the combined source overlay before execution.
/// Its entry hook creates Cargo-equivalent OUT_DIR; dynamic environment values
/// are supplied by append_metadata_invocation, never by a hidden frame loader.
pub(crate) fn original_script_run(
    stage: &str,
    id: &str,
    executable: ArtifactRef,
    package_tree: ArtifactRef,
    mut environment: BTreeMap<String, RecipeWord>,
    mut extra_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<ScriptRun, RunError> {
    require_produced(&executable, "original build-script executable")?;
    insert(&mut extra_inputs, "script", executable)?;
    insert(&mut extra_inputs, "package", package_tree)?;
    environment.insert("OUT_DIR".into(), output("out-dir"));
    environment.insert("CARGO_MANIFEST_DIR".into(), input("package", ""));
    Ok(ScriptRun {
        stdout: produced(stage, id, "stdout"),
        out_dir: produced(stage, id, "out-dir"),
        step: RecipeStep {
            id: id.into(),
            inputs: extra_inputs,
            outputs: BTreeMap::from([
                ("out-dir".into(), spec(ArtifactKind::Tree, 2 * 1024 * 1024 * 1024)),
                ("stdout".into(), spec(ArtifactKind::Text, 16 * 1024 * 1024)),
                ("stderr".into(), spec(ArtifactKind::Text, 16 * 1024 * 1024)),
                ("exit-status".into(), spec(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "script".into(),
                invocation_input: None,
                merge_stderr: false,
                arguments: Vec::new(),
                environment,
                runtime_mounts,
                output_seeds: BTreeMap::new(),
                guard: None,
                cwd_input: Some("package".into()),
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        },
    })
}

/// Derive the same package, selected-feature, target-cfg and build-override profile environment as
/// rust_plan::build_script_child_env, without inspecting host rustc or Cargo.
/// Path-remapped C compiler environment is supplied separately by native leaves.
pub(crate) fn build_script_environment(
    unit: &RustUnitDerivationSummary,
    host: &str,
    rustc: RecipeWord,
) -> Result<BTreeMap<String, RecipeWord>, RunError> {
    let mut environment = BTreeMap::new();
    for (key, value) in &unit.derivation.env {
        if key.starts_with("DEP_") || key.starts_with("CARGO_PKG_") || key == "CARGO_MANIFEST_LINKS" {
            environment.insert(key.clone(), literal(value));
        }
    }
    for (key, value) in crate::rust_plan::build_script_feature_environment(unit) {
        environment.insert(key, literal(value));
    }
    environment
        .entry("CARGO_PKG_NAME".into())
        .or_insert_with(|| literal(unit.target_name.replace('-', "_")));
    environment.insert("RUSTC".into(), rustc);
    environment.insert("HOST".into(), literal(host));
    let target = unit.derivation.env.get("TARGET").map(String::as_str).unwrap_or(host);
    environment.insert("TARGET".into(), literal(target));
    environment.insert("PROFILE".into(), literal(&unit.profile));
    let target_cfg = mantle_rust_plan_core::classify_native_target_triple(target);
    for (key, value) in [
        ("TARGET_ABI", target_cfg.abi),
        ("TARGET_ARCH", target_cfg.arch),
        ("TARGET_ENDIAN", target_cfg.endian),
        ("TARGET_ENV", target_cfg.env),
        ("TARGET_FAMILY", target_cfg.family),
        ("TARGET_FEATURE", target_cfg.features),
        ("TARGET_OS", target_cfg.os),
        ("TARGET_POINTER_WIDTH", target_cfg.pointer_width),
        ("TARGET_VENDOR", target_cfg.vendor),
    ] {
        environment.insert(format!("CARGO_CFG_{key}"), literal(value));
    }
    if target_cfg.is_unix {
        environment.insert("CARGO_CFG_UNIX".into(), literal(""));
    }
    if target_cfg.os == "windows" {
        environment.insert("CARGO_CFG_WINDOWS".into(), literal(""));
    }
    let recorded = unit
        .derivation
        .env
        .get("MANTLE_CARGO_PROFILE_POLICY")
        .and_then(|source| serde_json::from_str::<crate::cargo_profile::CargoProfilePolicy>(source).ok())
        .and_then(|policy| policy.build_override);
    let profile = match recorded {
        Some(policy) => policy,
        None => crate::cargo_profile::resolve_build_override_policy(&unit.profile, false)
            .map_err(|e| error(format!("build-script profile for {}: {e:?}", unit.unit_id)))?,
    };
    for (key, value) in profile.environment {
        environment.insert(key, literal(value));
    }
    Ok(environment)
}

/// An exact original generator and its consumer-side tree input name. A links
/// value selects DEP_<links>_<key> propagation rather than own cfg/env/link flags.
pub(crate) struct MetadataInput {
    pub(crate) stdout: ArtifactRef,
    pub(crate) out_dir: ArtifactRef,
    pub(crate) consumer_out_dir: String,
    pub(crate) source_package: ArtifactRef,
    pub(crate) consumer_package: String,
    pub(crate) links: Option<String>,
}

pub(crate) struct MetadataProducts {
    pub(crate) step: RecipeStep,
    pub(crate) metadata: ArtifactRef,
    /// Install these exact input aliases on the compiler (or instrumented
    /// original script). They are the paths encoded in the produced response.
    pub(crate) generated_inputs: BTreeMap<String, ArtifactRef>,
}

/// Produce actual compiler.rsp and compiler.env by reading actual script stdout.
/// Preserve the topology's own-script order and linked-metadata order in inputs.
/// Consumers bind `metadata` for its typed response-file operand, then lower the
/// actual environment frame through append_metadata_invocation. That canonical
/// invocation genuinely consumes every selected generated-path alias.
pub(crate) fn metadata_response(
    stage: &str,
    id: &str,
    executable: ArtifactRef,
    metadata_inputs: &[MetadataInput],
    runtime_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<MetadataProducts, RunError> {
    require_produced(&executable, "metadata phase helper")?;
    let mut inputs = runtime_inputs;
    insert(&mut inputs, "metadata-helper", executable)?;
    let mut generated_inputs = BTreeMap::new();
    let mut arguments = vec![output("metadata")];
    for (index, metadata) in metadata_inputs.iter().enumerate() {
        require_produced(&metadata.stdout, "original generator stdout")?;
        require_produced(&metadata.out_dir, "original generator OUT_DIR")?;
        let name = &metadata.consumer_out_dir;
        let package = &metadata.consumer_package;
        for alias in [name, package] {
            if alias.is_empty()
                || alias.len() > 256
                || matches!(alias.as_str(), "." | ".." | "metadata" | "metadata-helper")
                || !alias.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':' | b'.'))
            {
                return Err(error("invalid metadata source/output input alias"));
            }
        }
        if !matches!(&metadata.source_package, ArtifactRef::Source { .. } | ArtifactRef::ManifestSource { .. }) {
            return Err(error("metadata source package must be original admitted source"));
        }
        let stdout_name = format!("metadata-stdout-{index}");
        insert(&mut inputs, &stdout_name, metadata.stdout.clone())?;
        insert(&mut inputs, name, metadata.out_dir.clone())?;
        insert(&mut generated_inputs, name, metadata.out_dir.clone())?;
        insert(&mut inputs, package, metadata.source_package.clone())?;
        insert(&mut generated_inputs, package, metadata.source_package.clone())?;
        match &metadata.links {
            Some(links) => {
                arguments.push(literal("--dependency"));
                arguments.push(literal(links));
            }
            None => arguments.push(literal("--own")),
        }
        arguments.push(input(&stdout_name, ""));
        arguments.push(input(name, ""));
        arguments.push(input(package, ""));
    }
    Ok(MetadataProducts {
        metadata: produced(stage, id, "metadata"),
        generated_inputs,
        step: RecipeStep {
            id: id.into(),
            inputs,
            outputs: BTreeMap::from([
                ("metadata".into(), spec(ArtifactKind::Tree, 32 * 1024 * 1024)),
                ("stdout".into(), spec(ArtifactKind::Text, 1024 * 1024)),
                ("stderr".into(), spec(ArtifactKind::Text, 1024 * 1024)),
                ("exit-status".into(), spec(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "metadata-helper".into(),
                invocation_input: None,
                merge_stderr: false,
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts,
                output_seeds: BTreeMap::new(),
                guard: None,
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        },
    })
}

pub(crate) struct ScriptOverlay {
    pub(crate) preparation: Vec<RecipeStep>,
    pub(crate) step: RecipeStep,
    pub(crate) tree: ArtifactRef,
}

#[derive(Clone, Copy)]
pub(crate) enum ScriptProcessKind {
    Output,
    Status,
    AbsentSpawn,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    AbsentOutput,
}
#[derive(Clone)]
pub(crate) struct ScriptProcessBoundary {
    /// Exact original receiver expression, excluding .output()/.status()/.spawn().
    pub(crate) receiver: String,
    pub(crate) kind: ScriptProcessKind,
}

pub(crate) struct RustcRequest {
    pub(crate) preparation: Vec<RecipeStep>,
    pub(crate) step: RecipeStep,
    pub(crate) metadata: ArtifactRef,
}

/// Convert an actual original script request into compiler.rsp/compiler.env.
/// Executable, cwd and every argv word must equal the immutable source-selected
/// schedule. Only explicitly admitted prefix mappings may relocate paths;
/// neither this phase nor its compiler consumer selects or launches a tool.
pub(crate) fn rustc_request(
    stage: &str,
    id: &str,
    helper_executable: ArtifactRef,
    original_request: ArtifactRef,
    compiler: ArtifactRef,
    expected_cwd: RecipeWord,
    expected_arguments: &[RecipeWord],
    path_mappings: &[(RecipeWord, RecipeWord)],
    mut runtime_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<RustcRequest, RunError> {
    require_produced(&helper_executable, "compiler request helper")?;
    require_produced(&original_request, "original script compiler request")?;
    require_produced(&compiler, "source compiler query executable")?;
    insert(&mut runtime_inputs, "metadata-helper", helper_executable)?;
    insert(&mut runtime_inputs, "request", original_request)?;
    insert(&mut runtime_inputs, "rustc", compiler)?;
    let mut arguments = vec![
        literal("--rustc-request"),
        input("request", ""),
        input("rustc", ""),
        expected_cwd,
        output("metadata"),
    ];
    for (original, relocated) in path_mappings {
        arguments.push(literal("--map"));
        arguments.push(original.clone());
        arguments.push(relocated.clone());
    }
    arguments.push(literal("--"));
    arguments.extend_from_slice(expected_arguments);
    let preparation = lower_source_data(stage, id, &mut runtime_inputs, &mut arguments)?;
    Ok(RustcRequest {
        preparation,
        metadata: produced(stage, id, "metadata"),
        step: RecipeStep {
            id: id.into(),
            inputs: runtime_inputs,
            outputs: BTreeMap::from([
                ("metadata".into(), spec(ArtifactKind::Tree, 32 * 1024 * 1024)),
                ("stdout".into(), spec(ArtifactKind::Text, 1024 * 1024)),
                ("stderr".into(), spec(ArtifactKind::Text, 1024 * 1024)),
                ("exit-status".into(), spec(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "metadata-helper".into(),
                invocation_input: None,
                merge_stderr: false,
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts,
                output_seeds: BTreeMap::new(),
                guard: None,
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        },
    })
}

pub(crate) struct LibraryOverlaySource {
    pub(crate) original: ArtifactRef,
    pub(crate) compiler_path: String,
    pub(crate) replacements: Vec<(String, String)>,
    pub(crate) append: Vec<String>,
}

/// Preserve pkg-config's original fallback decisions by performing the actual
/// executable lookup in the isolated namespace. A present executable requires
/// a genuine process phase and fails loudly; no failed status or stdout is
/// manufactured. The original WrappedCommand argv/environment remain intact.
pub(crate) fn pkg_config_library_overlay(
    source_package: &ArtifactRef,
    compiler_package_path: &str,
) -> Result<LibraryOverlaySource, RunError> {
    let member = |root: &str| {
        if root.is_empty() {
            "src/lib.rs".to_string()
        } else {
            format!("{root}/src/lib.rs")
        }
    };
    let original = match source_package {
        ArtifactRef::Source { role, path } => ArtifactRef::Source {
            role: *role,
            path: member(path),
        },
        ArtifactRef::ManifestSource {
            role,
            record_identity,
            member: root,
        } => ArtifactRef::ManifestSource {
            role: *role,
            record_identity: record_identity.clone(),
            member: member(root),
        },
        ArtifactRef::Produced { .. } => return Err(error("pkg-config overlay requires the original source package")),
    };
    Ok(LibraryOverlaySource {
        original,
        compiler_path: format!("{}/src/lib.rs", compiler_package_path.trim_end_matches('/')),
        replacements: vec![(
            "self.inner.output()".into(),
            "crate::mantle_cairn_script_process::output_if_absent(&mut self.inner)".into(),
        )],
        append: vec![process_replay_source("")],
    })
}

/// Preserve all original library argv/module identities while replacing
/// inspected process boundaries. No main instrumentation is added. One frame
/// covers every original module and its separately produced replacement.
fn unprepared_library_overlays(
    stage: &str,
    id: &str,
    executable: ArtifactRef,
    sources: &[LibraryOverlaySource],
    mut runtime_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<ScriptOverlay, RunError> {
    require_produced(&executable, "library overlay helper")?;
    if sources.is_empty() {
        return Err(error("library overlay requires original sources"));
    }
    insert(&mut runtime_inputs, "metadata-helper", executable)?;
    let mut arguments = vec![literal("--library-overlays"), output("source-overlay")];
    for (index, source) in sources.iter().enumerate() {
        if !matches!(&source.original, ArtifactRef::Source { .. } | ArtifactRef::ManifestSource { .. }) {
            return Err(error("library overlay must read original admitted source"));
        }
        let name = format!("original-{index}");
        insert(&mut runtime_inputs, &name, source.original.clone())?;
        arguments.push(literal("--source"));
        arguments.push(input(&name, ""));
        arguments.push(literal(&source.compiler_path));
        for (before, after) in &source.replacements {
            arguments.push(literal("--replace"));
            arguments.push(literal(before));
            arguments.push(literal(after));
        }
        for addition in &source.append {
            arguments.push(literal("--append"));
            arguments.push(literal(addition));
        }
    }
    Ok(ScriptOverlay {
        preparation: Vec::new(),
        tree: produced(stage, id, "source-overlay"),
        step: RecipeStep {
            id: id.into(),
            inputs: runtime_inputs,
            outputs: BTreeMap::from([
                ("source-overlay".into(), spec(ArtifactKind::Tree, 32 * 1024 * 1024)),
                ("stdout".into(), spec(ArtifactKind::Text, 1024 * 1024)),
                ("stderr".into(), spec(ArtifactKind::Text, 1024 * 1024)),
                ("exit-status".into(), spec(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "metadata-helper".into(),
                invocation_input: None,
                merge_stderr: false,
                arguments,
                environment: BTreeMap::new(),
                runtime_mounts,
                output_seeds: BTreeMap::new(),
                guard: None,
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
            },
        },
    })
}

/// Admit helper source-program data without granting a literal filesystem path.
/// Input/Output/Runtime operands remain typed and unchanged; only literal data
/// carried to this in-process metadata helper receives a Produced Text input.
pub(super) fn lower_source_data(
    stage: &str,
    id: &str,
    inputs: &mut BTreeMap<String, ArtifactRef>,
    arguments: &mut [RecipeWord],
) -> Result<Vec<RecipeStep>, RunError> {
    let mut preparation = Vec::new();
    for word in arguments {
        for part in &mut word.parts {
            let WordPart::Literal { text } = part else {
                continue;
            };
            if !text.contains(['/', '\\']) {
                continue;
            }
            let name = format!("{id}-source-data-{:05}", preparation.len());
            let text = std::mem::take(text);
            let bytes_max = u64::try_from(text.len()).map_err(|_| error("metadata source data exceeds byte bound"))?;
            insert(inputs, &name, produced(stage, &name, "text"))?;
            preparation.push(RecipeStep {
                id: name.clone(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("text".into(), spec(ArtifactKind::Text, bytes_max))]),
                operation: RecipeOperation::WriteText {
                    output: "text".into(),
                    text,
                },
            });
            *part = WordPart::TextInput { name };
        }
    }
    Ok(preparation)
}

fn prepare_overlay(stage: &str, id: &str, overlay: &mut ScriptOverlay) -> Result<(), RunError> {
    let RecipeOperation::Exec { arguments, .. } = &mut overlay.step.operation else {
        unreachable!()
    };
    overlay.preparation = lower_source_data(stage, id, &mut overlay.step.inputs, arguments)?;
    Ok(())
}

/// Preserve original library module identities and exact source transforms.
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
#[allow(dead_code)]
pub(crate) fn library_overlays(
    stage: &str,
    id: &str,
    executable: ArtifactRef,
    sources: &[LibraryOverlaySource],
    runtime_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<ScriptOverlay, RunError> {
    let mut overlay = unprepared_library_overlays(stage, id, executable, sources, runtime_inputs, runtime_mounts)?;
    prepare_overlay(stage, id, &mut overlay)?;
    Ok(overlay)
}

/// Compose source-exact module repairs and the original script entry hook in a
/// single compiler frame. Every module is still read from admitted original
/// source by the produced helper; no preparation-time transformed source wins.
pub(crate) fn script_and_library_overlays(
    stage: &str,
    id: &str,
    executable: ArtifactRef,
    sources: &[LibraryOverlaySource],
    entry_path: Option<&str>,
    boundaries: &BTreeMap<String, Vec<ScriptProcessBoundary>>,
    runtime_inputs: BTreeMap<String, ArtifactRef>,
    runtime_mounts: BTreeMap<String, String>,
) -> Result<ScriptOverlay, RunError> {
    let mut overlay = unprepared_library_overlays(stage, id, executable, sources, runtime_inputs, runtime_mounts)?;
    let RecipeOperation::Exec { arguments, .. } = &mut overlay.step.operation else {
        unreachable!()
    };
    let mut combined = Vec::with_capacity(arguments.len());
    let mut original = std::mem::take(arguments).into_iter();
    combined.extend(original.by_ref().take(2));
    for source in sources {
        let count = 3 + 3 * source.replacements.len() + 2 * source.append.len();
        combined.extend(original.by_ref().take(count));
        let path = source.compiler_path.as_str();
        if entry_path == Some(path) {
            combined.push(literal("--entry"));
        }
        if let Some(boundaries) = boundaries.get(path) {
            for boundary in boundaries {
                combined.push(literal(match boundary.kind {
                    ScriptProcessKind::Output => "--output",
                    ScriptProcessKind::Status => "--status",
                    ScriptProcessKind::AbsentSpawn => "--absent-spawn",
                    ScriptProcessKind::AbsentOutput => "--absent-output",
                }));
                combined.push(literal(&boundary.receiver));
            }
            if !boundaries.is_empty() {
                combined.push(literal("--process-module"));
            }
        }
    }
    if original.next().is_some() {
        return Err(error("source overlay argument inventory drift"));
    }
    *arguments = combined;
    prepare_overlay(stage, id, &mut overlay)?;
    Ok(overlay)
}

/// Validate a source-fixed autocfg probe name against the original generator's
/// produced UUID instead of inventing a deterministic replacement for it.
pub(crate) fn add_autocfg_name(
    request: &mut RustcRequest,
    uuid: ArtifactRef,
    member: &str,
    probe_index: u64,
) -> Result<(), RunError> {
    require_produced(&uuid, "original autocfg UUID")?;
    crate::source_built_recipe::relative(member, true)?;
    insert(&mut request.step.inputs, "autocfg-uuid", uuid)?;
    let RecipeOperation::Exec { arguments, .. } = &mut request.step.operation else {
        unreachable!()
    };
    arguments.splice(5..5, [
        literal("--autocfg-name"),
        input("autocfg-uuid", member),
        literal(probe_index.to_string()),
    ]);
    Ok(())
}

/// Merge the actual original metadata environment with a source-authored typed
/// invocation. `generated_inputs` is the exact original-source-selected resolver,
/// not a list of speculative dependencies. Actual frame paths must resolve to
/// these aliases (or existing typed command inputs/outputs/runtime mounts).
/// No compiler/script entry hook loads an opaque environment frame afterward.
pub(crate) fn append_metadata_invocation(
    program: &mut SourceRecipe,
    step: &mut RecipeStep,
    executable: ArtifactRef,
    metadata: ArtifactRef,
    generated_inputs: &BTreeMap<String, ArtifactRef>,
) -> Result<(), RunError> {
    require_produced(&executable, "invocation metadata helper")?;
    require_produced(&metadata, "actual original metadata environment")?;
    for (name, reference) in generated_inputs {
        match step.inputs.get(name) {
            Some(previous) if previous != reference => return Err(error("metadata invocation input alias collision")),
            Some(_) => (),
            None => {
                step.inputs.insert(name.clone(), reference.clone());
            }
        }
    }
    let RecipeOperation::Exec {
        arguments,
        environment,
        runtime_mounts,
        executable_input,
        cwd_input,
        stdin_input,
        output_seeds,
        guard,
        invocation_input,
        ..
    } = &mut step.operation
    else {
        return Err(error("metadata invocation requires a command"));
    };
    if invocation_input.is_some() {
        return Err(error("metadata invocation is already authored"));
    }
    environment.remove(COMPILE_ENVIRONMENT_VARIABLE);
    environment.remove(SCRIPT_ENVIRONMENT_VARIABLE);
    let invocation = RecipeInvocation {
        readonly_input_roots: Vec::new(),
        arguments: std::mem::take(arguments),
        environment: std::mem::take(environment),
        cwd: None,
        argv0: None,
        executable_path: None,
        workspace_mounts: BTreeMap::new(),
        process_attributes: None,
        descriptors: None,
    };
    let mut used = BTreeSet::new();
    used.insert(executable_input.as_str());
    used.extend(runtime_mounts.values().map(String::as_str));
    used.extend(cwd_input.iter().chain(stdin_input.iter()).map(String::as_str));
    used.extend(output_seeds.values().map(String::as_str));
    if let Some(guard) = guard {
        used.extend(guard.conditions.iter().map(|condition| condition.status_input.as_str()));
    }
    for word in invocation.arguments.iter().chain(invocation.environment.values()) {
        for part in &word.parts {
            match part {
                WordPart::Input { name, .. } | WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                    used.insert(name.as_str());
                }
                WordPart::Runtime { mount, .. } => {
                    let name = runtime_mounts
                        .get(mount)
                        .ok_or_else(|| error("metadata template has undeclared runtime mount"))?;
                    used.insert(name.as_str());
                }
                _ => (),
            }
        }
    }
    used.extend(generated_inputs.keys().map(String::as_str));
    step.inputs.retain(|name, _| used.contains(name.as_str()));
    let id = format!("{}-invocation", step.id);
    let template_id = format!("{id}-template");
    let data_name = format!("{id}-data");
    #[derive(serde::Serialize)]
    struct Template<'a> {
        invocation: &'a RecipeInvocation,
        inputs: Vec<&'a str>,
        outputs: Vec<&'a str>,
        runtime: Vec<&'a str>,
        data_name: &'a str,
    }
    let template = serde_json::to_string(&Template {
        invocation: &invocation,
        inputs: step.inputs.keys().map(String::as_str).collect(),
        outputs: step.outputs.keys().map(String::as_str).collect(),
        runtime: runtime_mounts.keys().map(String::as_str).collect(),
        data_name: &data_name,
    })
    .map_err(|failure| error(format!("serializing source invocation template: {failure}")))?;
    program.steps.push(RecipeStep {
        id: template_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("template".into(), spec(ArtifactKind::Text, template.len() as u64))]),
        operation: RecipeOperation::WriteText {
            output: "template".into(),
            text: template,
        },
    });
    let mut inputs = BTreeMap::from([
        ("metadata-helper".into(), executable),
        ("actual-metadata".into(), metadata),
        ("template".into(), produced(&program.stage_id, &template_id, "template")),
    ]);
    for name in runtime_mounts.values() {
        let reference = step.inputs.get(name).ok_or_else(|| error("metadata invocation runtime input absent"))?;
        if let Some(previous) = inputs.insert(name.clone(), reference.clone())
            && previous != *reference
        {
            return Err(error("invocation helper runtime alias collision"));
        }
    }
    program.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([
            ("invocation".into(), spec(ArtifactKind::Text, 32 * 1024 * 1024)),
            ("environment-data".into(), spec(ArtifactKind::Text, 32 * 1024 * 1024)),
            ("stdout".into(), spec(ArtifactKind::Text, 1024 * 1024)),
            ("stderr".into(), spec(ArtifactKind::Text, 1024 * 1024)),
            ("exit-status".into(), spec(ArtifactKind::Text, 4)),
        ]),
        operation: RecipeOperation::Exec {
            command_state: None,
            executable_input: "metadata-helper".into(),
            arguments: vec![
                literal("--compiler-invocation"),
                input("template", ""),
                input("actual-metadata", "compiler.env"),
                output("invocation"),
                output("environment-data"),
            ],
            environment: BTreeMap::new(),
            runtime_mounts: runtime_mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
            output_seeds: BTreeMap::new(),
            merge_stderr: false,
            invocation_input: None,
        },
    });
    insert(&mut step.inputs, &data_name, produced(&program.stage_id, &id, "environment-data"))?;
    insert(&mut step.inputs, &id, produced(&program.stage_id, &id, "invocation"))?;
    *invocation_input = Some(id);
    Ok(())
}
