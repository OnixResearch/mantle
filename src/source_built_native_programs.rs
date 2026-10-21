//! Preparation-time expansion of the exact original native derivation union.
//! All executables in node programs are individual Produced leaves. Logical
//! store paths identify original edges; they never authorize a directory prefix.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_program_authoring::ContinuationReservationPolicy;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeStep;

#[path = "source_built_native_programs_early_gnu.rs"]
mod early_gnu;
#[path = "source_built_native_programs_final_gnu.rs"]
mod final_gnu;
#[path = "source_built_native_programs_host.rs"]
pub(crate) mod host;
#[path = "source_built_native_programs_phases.rs"]
pub(crate) mod phases;

pub(crate) const STAGE: &str = "full-source-native-provider";
pub(crate) const RECORD_IDENTITY: &str = "fixed-point-native-source-program";
pub(crate) const RECIPE_MEMBER: &str = "source-built/native/program.json";
pub(crate) const INDEX_MEMBER: &str = "source-built/native/program.expansion.json";

/// One authenticated original fetch source, available for preparation-time
/// inspection only. Its execution-time materialization remains a real node.
pub(crate) struct NativeSource {
    pub record_identity: String,
    pub original_output: String,
    pub preparation_path: PathBuf,
    pub artifact: ArtifactRef,
    /// Unique regular native source members in this authenticated record.
    /// This sizes an admitted reservation; it is not a future command count.
    pub native_source_files: u32,
}

/// Source-owned expansion of one original derivation. Exact final outputs and
/// individual installed members retain their real compiler/generator ancestry.
pub(crate) struct NativeNodeProgram {
    pub steps: Vec<RecipeStep>,
    pub timeout_groups: Vec<crate::source_built_recipe::RecipeTimeoutGroup>,
    pub outputs: BTreeMap<String, ArtifactRef>,
    pub members: BTreeMap<String, BTreeMap<String, ArtifactRef>>,
    /// Exact source-built auxiliary entries, separate from original installed
    /// members. They never stand in for completion of the original outputs.
    pub phase_tools: BTreeMap<String, ArtifactRef>,
    pub files: BTreeMap<String, String>,
}

pub(crate) struct NativeNodeRequest<'a, 'b> {
    pub request: &'a AuthoringRequest<'b>,
    pub node: &'a crunch_pipeline::native_preflight::NativePlannedDerivation,
    /// Stable source identity derived from the exact HDM, not observed bytes.
    pub id: &'a str,
    /// Exact derivation path -> already expanded selected outputs/members.
    pub dependencies: &'a BTreeMap<String, &'a NativeNodeProgram>,
    /// Exact original logical source output path -> authenticated source.
    pub sources: &'a BTreeMap<String, NativeSource>,
    pub gcc_splitter: &'a ArtifactRef,
    pub logical_outputs: &'a BTreeMap<String, String>,
    pub reservations: &'a ContinuationReservationPolicy,
    pub native_source_files: u32,
}

pub(crate) struct NativeLogicalPrefixes {
    pub provider: String,
    pub gcc: String,
    pub musl: String,
    pub binutils: String,
    /// Exact `<original-drv>:<selected-output>` -> original logical path.
    pub derivation_outputs: BTreeMap<String, String>,
}

/// Original compiler search/runtime prefixes come from the SAME seven-root
/// evaluator as admission. No path is learned from execution or host ELF data.
pub(crate) fn logical_prefixes(
    request: &AuthoringRequest<'_>,
) -> Result<NativeLogicalPrefixes, crate::errors::RunError> {
    use crate::source_built_fixed_point::SourceAuthorityRole;
    let error = |message: &str| crate::errors::RunError::Build(format!("native source authoring: {message}"));
    let source = request
        .source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("missing exact Mantle recipe source authority"))?;
    let prefix = &request.proof.logical_store_prefix;
    let graph = crate::source_built_fixed_point_native_actions::closure(source, prefix)?;
    let mut outputs = BTreeMap::new();
    for node in &graph.derivations {
        let drv = node.drv_path.to_absolute_path_with_prefix(prefix);
        for (name, output) in &node.derivation.outputs {
            let path = output.path.as_ref().ok_or_else(|| error("original static native output path is missing"))?;
            outputs.insert(format!("{drv}:{name}"), path.to_absolute_path_with_prefix(prefix));
        }
    }
    let selected = |suffix: &str| -> Result<String, crate::errors::RunError> {
        let mut matches = graph.derivations.iter().filter(|node| node.drv_path.to_string().ends_with(suffix));
        let node = matches.next().ok_or_else(|| error("original native compiler dependency is absent"))?;
        if matches.next().is_some() {
            return Err(error("ambiguous original native compiler dependency"));
        }
        outputs
            .get(&format!("{}:out", node.drv_path.to_absolute_path_with_prefix(prefix)))
            .cloned()
            .ok_or_else(|| error("original native compiler output is absent"))
    };
    let provider_root = graph.roots.first().ok_or_else(|| error("seven-root native closure is empty"))?;
    let provider = outputs
        .get(&format!("{}:out", provider_root.drv_path.to_absolute_path_with_prefix(prefix)))
        .cloned()
        .ok_or_else(|| error("original selected provider output is absent"))?;
    Ok(NativeLogicalPrefixes {
        provider,
        gcc: selected("-gcc-10.5.0-musl-final-v1.drv")?,
        musl: selected("-musl-1.2.5-gcc10-v1.drv")?,
        binutils: selected("-binutils-2.41-gcc10-v1.drv")?,
        derivation_outputs: outputs,
    })
}

/// Compile source-authenticated phase code with the real predecessor TCC.
/// This is the original StageX static-link shape, not an ambient C compiler.
pub(crate) fn append_stagex_c_helper(
    request: &AuthoringRequest<'_>,
    program: &mut crate::source_built_recipe::SourceRecipe,
    id: &str,
    member: &str,
) -> Result<ArtifactRef, crate::errors::RunError> {
    use crate::source_built_fixed_point::SourceAuthorityRole;
    use crate::source_built_recipe::ArtifactKind;
    use crate::source_built_recipe::RecipeExitPolicy;
    use crate::source_built_recipe::RecipeOperation;
    use crate::source_built_recipe::RecipeOutput;
    use crate::source_built_recipe::RecipeWord;
    use crate::source_built_recipe::WordPart;
    let failure = |message: String| crate::errors::RunError::Build(format!("native phase bootstrap: {message}"));
    let predecessor = |role: &str| -> Result<ArtifactRef, crate::errors::RunError> {
        let key = format!("stagex-provider-publication:{role}");
        match request.predecessor_exports.get(&key) {
            Some(reference @ ArtifactRef::Produced { .. }) => Ok(reference.clone()),
            _ => Err(failure(format!("missing exact Produced predecessor {key}"))),
        }
    };
    crate::source_built_recipe::relative(member, false)?;
    if program.steps.iter().any(|step| step.id == id) {
        return Err(failure(format!("duplicate helper producer {id}")));
    }
    let literal = |text: &str| RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    };
    let input = |name: &str, path: &str| RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    };
    let flag_input = |flag: &str, name: &str, path: &str| RecipeWord {
        parts: vec![WordPart::Literal { text: flag.into() }, WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    };
    let inputs = BTreeMap::from([
        ("tcc".into(), predecessor("tcc")?),
        ("headers".into(), predecessor("musl.headers")?),
        ("libraries".into(), predecessor("musl.libraries")?),
        ("runtime".into(), predecessor("tcc.runtime")?),
        ("source".into(), ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::NativeSourceBundle,
            record_identity: RECORD_IDENTITY.into(),
            member: member.into(),
        }),
    ]);
    let output = ArtifactRef::Produced {
        action_id: format!("{}/{id}", program.stage_id),
        output_id: "tool".into(),
    };
    program.steps.push(RecipeStep {
        id: id.into(),
        inputs,
        outputs: BTreeMap::from([("tool".into(), RecipeOutput {
            kind: ArtifactKind::Executable,
            bytes_max: request.proof.resource_bounds.disk_bytes_max.min(64 * 1024 * 1024),
        })]),
        operation: RecipeOperation::Exec {
            executable_input: "tcc".into(),
            arguments: vec![
                flag_input("-B", "runtime", ""),
                literal("-nostdinc"),
                flag_input("-I", "runtime", "include"),
                flag_input("-I", "headers", ""),
                literal("-nostdlib"),
                literal("-static"),
                input("libraries", "crt1.o"),
                input("libraries", "crti.o"),
                input("source", ""),
                input("libraries", "libc.a"),
                input("runtime", "libtcc1.a"),
                input("libraries", "crtn.o"),
                literal("-o"),
                RecipeWord {
                    parts: vec![WordPart::Output {
                        name: "tool".into(),
                        path: String::new(),
                    }],
                },
            ],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            merge_stderr: false,
            invocation_input: None,
        },
    });
    Ok(output)
}

pub(crate) fn phase_sources() -> BTreeMap<String, String> {
    [
        ("gcc-phases.c", phases::GCC_PHASE_HELPER_SOURCE),
        ("argv-main.c", include_str!("source_built_native_programs_argv_main.c")),
        ("collect2-patch.c", include_str!("source_built_native_programs_collect2_patch.c")),
        ("collect2-phase.inc", include_str!("source_built_native_programs_collect2_phase.inc")),
        (
            "collect2-driver-metadata.inc",
            include_str!("source_built_native_programs_collect2_driver_metadata.inc"),
        ),
        ("controller-gcc.c", include_str!("source_built_native_programs_controller_gcc.c")),
        ("generated.c", include_str!("source_built_native_programs_generated.c")),
    ]
    .into_iter()
    .map(|(member, source)| (format!("source-built/native/{member}"), source.to_string()))
    .collect()
}

/// Embed the source phase seam into the real original collect2 translation unit.
/// This is source preparation only; consumers still build and execute collect2.
pub(crate) fn append_gcc10_collect2_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    phase: &str,
    original: ArtifactRef,
    member: &str,
) -> Result<ArtifactRef, crate::errors::RunError> {
    use crate::source_built_fixed_point::SourceAuthorityRole;
    use crate::source_built_recipe::ArtifactKind;
    use crate::source_built_recipe::RecipeExitPolicy;
    use crate::source_built_recipe::RecipeOperation;
    use crate::source_built_recipe::RecipeOutput;
    use crate::source_built_recipe::RecipePreparation;
    use crate::source_built_recipe::RecipeWord;
    use crate::source_built_recipe::SourceRecipe;
    use crate::source_built_recipe::WordPart;
    crate::source_built_recipe::relative(phase, false)?;
    crate::source_built_recipe::relative(member, false)?;
    if !matches!(original, ArtifactRef::Produced { .. }) {
        return Err(author_error("GCC10 collect2 source must retain its original producer"));
    }
    const KEY: &str = "gcc10-collect2-source-helper";
    let tool = if let Some(tool) = program.phase_tools.get(KEY) {
        tool.clone()
    } else {
        let mut fragment = SourceRecipe {
            schema: crate::source_built_recipe::SCHEMA.into(),
            stage_id: STAGE.into(),
            preparation: RecipePreparation {
                tool: "GCC10 source phase adaptation".into(),
                provenance: "authenticated original collect2 source and source-owned exact first-link seam".into(),
            },
            steps: Vec::new(),
            timeout_groups: Vec::new(),
            roots: Vec::new(),
            exports: BTreeMap::new(),
            semantic_roots: BTreeMap::new(),
        };
        let tool = append_stagex_c_helper(
            request.request,
            &mut fragment,
            &format!("{}.gcc10-collect2-source-helper", request.id),
            "source-built/native/collect2-patch.c",
        )?;
        program.steps.extend(fragment.steps);
        program.timeout_groups.extend(fragment.timeout_groups);
        program.phase_tools.insert(KEY.into(), tool.clone());
        tool
    };
    let source = |member: &str| ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: RECORD_IDENTITY.into(),
        member: format!("source-built/native/{member}"),
    };
    let input = |name: &str, path: &str| RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    };
    let id = format!("{}.gcc10-{phase}-collect2-source", request.id);
    let result = ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: "source".into(),
    };
    program.steps.push(RecipeStep {
        id,
        inputs: BTreeMap::from([
            ("tool".into(), tool),
            ("original".into(), original),
            ("phase".into(), source("collect2-phase.inc")),
            ("metadata".into(), source("collect2-driver-metadata.inc")),
        ]),
        outputs: BTreeMap::from([("source".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 8 * 1024 * 1024,
        })]),
        operation: RecipeOperation::Exec {
            executable_input: "tool".into(),
            arguments: vec![
                input("original", member),
                input("phase", ""),
                input("metadata", ""),
                RecipeWord {
                    parts: vec![WordPart::Output {
                        name: "source".into(),
                        path: String::new(),
                    }],
                },
            ],
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            merge_stderr: false,
            invocation_input: None,
        },
    });
    Ok(result)
}

/// Append after shellvm::library_source(). The wire protocol and path mapper
/// remain shared; only source GCC child selection and native operand grammar
/// are added. The quote parser is the same source used by static GCC phases.
pub(crate) fn controller_gcc_source() -> Result<String, crate::errors::RunError> {
    let splitter = include_str!("source_built_native_programs_gcc_phases.c");
    let begin = splitter
        .find("/* GCC's -### print_argv")
        .ok_or_else(|| author_error("GCC phase quote parser boundary drift"))?;
    let end = splitter[begin..]
        .find("static void append(")
        .map(|n| begin + n)
        .ok_or_else(|| author_error("GCC phase metadata parser boundary drift"))?;
    let mut source = String::from(
        "\n#define ARGS_MAX 16384u\n#define WORD_MAX 65536u\nstatic char *native_gcc_phase_arguments[ARGS_MAX];\nstatic void native_gcc_phase_fail(const char *s){die(\"native GCC: %s\",s);}\n#define split native_gcc_phase_split\n#define metadata native_gcc_phase_metadata\n#define arguments native_gcc_phase_arguments\n#define fail native_gcc_phase_fail\n",
    );
    source.push_str(&splitter[begin..end]);
    source
        .push_str("\n#undef fail\n#undef arguments\n#undef metadata\n#undef split\n#undef WORD_MAX\n#undef ARGS_MAX\n");
    source.push_str(include_str!("source_built_native_programs_controller_gcc.c"));
    Ok(source)
}

fn author_error(message: impl Into<String>) -> crate::errors::RunError {
    crate::errors::RunError::Build(format!("native source authoring: {}", message.into()))
}

fn environment<'a>(node: &'a crunch_pipeline::native_preflight::NativePlannedDerivation, key: &str) -> Option<&'a str> {
    node.derivation.environment.get(key).and_then(|value| std::str::from_utf8(value.as_ref()).ok())
}

/// Match all original fetch semantics, including the fixed-output pin. A URL,
/// basename or available tree alone is not source authority for a derivation.
fn source_record_matches(
    node: &crunch_pipeline::native_preflight::NativePlannedDerivation,
    record: &crate::source_bundle::SourceRecord,
) -> Result<bool, crate::errors::RunError> {
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::HashAlgo;
    use nix_compat::nixhash::NixHash;
    if !crate::source_bundle::source_record_is_fetcher_input(record) {
        return Ok(false);
    }
    for key in ["name", "url", "unpack", "executable", "type", "rev", "fetch_policy"] {
        if environment(node, key) != record.metadata.get(key).map(String::as_str) {
            return Ok(false);
        }
    }
    let Some(hash) = record.metadata.get("hash") else {
        return Ok(false);
    };
    let Some(algo) = record.metadata.get("hash_algo") else {
        return Ok(false);
    };
    let algo = HashAlgo::try_from(algo.as_str()).map_err(|error| author_error(error.to_string()))?;
    let pin = if hash.contains('-') {
        NixHash::from_sri(hash).map_err(|error| author_error(error.to_string()))?
    } else {
        let bytes = data_encoding::HEXLOWER.decode(hash.as_bytes()).map_err(|error| author_error(error.to_string()))?;
        NixHash::from_algo_and_digest(algo, &bytes).map_err(|error| author_error(error.to_string()))?
    };
    let pin = match record.metadata.get("hash_mode").map(String::as_str) {
        Some("flat") => CAHash::Flat(pin),
        Some("recursive") => CAHash::Nar(pin),
        _ => return Ok(false),
    };
    Ok(node.derivation.outputs.len() == 1
        && node.derivation.outputs.values().all(|output| output.ca_hash.as_ref() == Some(&pin)))
}

fn original_source_node(
    request: &AuthoringRequest<'_>,
    node: &crunch_pipeline::native_preflight::NativePlannedDerivation,
    id: &str,
    manifest: &crate::source_bundle::SourceBundleManifest,
    scratch: &std::path::Path,
    inspected: &mut BTreeMap<String, PathBuf>,
    sources: &mut BTreeMap<String, NativeSource>,
) -> Result<Option<NativeNodeProgram>, crate::errors::RunError> {
    use crate::source_built_fixed_point::SourceAuthorityRole;
    use crate::source_built_recipe::ArtifactKind;
    use crate::source_built_recipe::RecipeOperation;
    use crate::source_built_recipe::RecipeOutput;
    if node.derivation.builder != "builtin:fetchurl" {
        return Ok(None);
    }
    if !node.derivation.input_derivations.is_empty() || !node.derivation.input_sources.is_empty() {
        return Err(author_error("original fetcher unexpectedly has derivation/source dependencies"));
    }
    let mut matching = Vec::new();
    for record in &manifest.records {
        if source_record_matches(node, record)? {
            matching.push(record);
        }
    }
    if matching.len() != 1 {
        return Err(author_error(format!(
            "original pinned fetcher {} has {} matching admitted source records",
            node.drv_path,
            matching.len()
        )));
    }
    let record = matching[0];
    let preparation_path = if let Some(path) = inspected.get(&record.identity) {
        path.clone()
    } else {
        let path = scratch.join(format!("original-{}", inspected.len()));
        crate::source_bundle::materialize_source_record_for_offline_use(record, &path)?;
        inspected.insert(record.identity.clone(), path.clone());
        path
    };
    // Source records may hold archive chunks, not member paths. Inspect only
    // the already-authenticated offline materialization, without following
    // source symlinks into other records or host directories.
    let mut native_source_files = 0u32;
    let mut pending_members = vec![preparation_path.clone()];
    while let Some(member) = pending_members.pop() {
        let metadata = std::fs::symlink_metadata(&member)
            .map_err(|cause| author_error(format!("original source member metadata: {cause}")))?;
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&member)
                .map_err(|cause| author_error(format!("original source member inventory: {cause}")))?
            {
                pending_members.push(entry.map_err(|cause| author_error(cause.to_string()))?.path());
            }
        } else if metadata.is_file()
            && matches!(
                member.extension().and_then(|extension| extension.to_str()),
                Some("c" | "cc" | "cpp" | "cxx" | "c++" | "C" | "s" | "S" | "asm")
            )
        {
            native_source_files = native_source_files
                .checked_add(1)
                .ok_or_else(|| author_error("original native source-file reservation hint exceeds u32"))?;
        }
    }
    let (output_name, output) =
        node.derivation.outputs.iter().next().ok_or_else(|| author_error("empty original fetch output"))?;
    let original_output = output
        .path
        .as_ref()
        .ok_or_else(|| author_error("original source has no static output path"))?
        .to_absolute_path_with_prefix(&request.proof.logical_store_prefix);
    let kind = if record.metadata.get("unpack").map(String::as_str) == Some("1")
        || record.kind == crate::source_bundle::SourceRecordKind::VcsSnapshot
    {
        ArtifactKind::Tree
    } else if record.metadata.get("executable").map(String::as_str) == Some("1") {
        // A fetched executable is never native compiler/provider authority.
        return Err(author_error("native original fetcher cannot import executable provider authority"));
    } else {
        ArtifactKind::Text
    };
    let reference = ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: output_name.clone(),
    };
    if sources
        .insert(original_output.clone(), NativeSource {
            record_identity: record.identity.clone(),
            original_output,
            preparation_path,
            artifact: reference.clone(),
            native_source_files,
        })
        .is_some()
    {
        return Err(author_error("duplicate original source output identity"));
    }
    Ok(Some(NativeNodeProgram {
        steps: vec![RecipeStep {
            id: id.into(),
            inputs: BTreeMap::from([("original".into(), ArtifactRef::ManifestSource {
                role: SourceAuthorityRole::NativeSourceBundle,
                record_identity: record.identity.clone(),
                member: String::new(),
            })]),
            outputs: BTreeMap::from([(output_name.clone(), RecipeOutput {
                kind,
                bytes_max: request.proof.resource_bounds.disk_bytes_max,
            })]),
            operation: RecipeOperation::MaterializeFetcher {
                role: SourceAuthorityRole::NativeSourceBundle,
                record_identity: record.identity.clone(),
                output: output_name.clone(),
            },
        }],
        outputs: BTreeMap::from([(output_name.clone(), reference)]),
        members: BTreeMap::new(),
        phase_tools: BTreeMap::new(),
        files: BTreeMap::new(),
        timeout_groups: Vec::new(),
    }))
}

fn validate_node_program(
    node: &crunch_pipeline::native_preflight::NativePlannedDerivation,
    program: &NativeNodeProgram,
    dependencies: &BTreeMap<String, &NativeNodeProgram>,
    prefix: &str,
) -> Result<(), crate::errors::RunError> {
    if program.steps.is_empty() || program.outputs.keys().ne(node.derivation.outputs.keys()) {
        return Err(author_error(format!("incomplete original output expansion {}", node.drv_path)));
    }
    let steps = program.steps.iter().map(|step| (step.id.as_str(), step)).collect::<BTreeMap<_, _>>();
    if steps.len() != program.steps.len() {
        return Err(author_error("duplicate local original phase identity"));
    }
    for reference in program.outputs.values() {
        let ArtifactRef::Produced { action_id, output_id } = reference else {
            return Err(author_error("original output is not locally produced"));
        };
        let step = action_id
            .strip_prefix(&format!("{STAGE}/"))
            .and_then(|id| steps.get(id))
            .ok_or_else(|| author_error("original output producer is not owned by its derivation"))?;
        if !step.outputs.contains_key(output_id) {
            return Err(author_error("original output producer has no such output"));
        }
    }
    for (dependency, selected) in &node.derivation.input_derivations {
        let key = dependency.to_absolute_path_with_prefix(prefix);
        let dependency =
            dependencies.get(&key).ok_or_else(|| author_error("missing original direct dependency expansion"))?;
        for output in selected {
            let reference = dependency
                .outputs
                .get(output)
                .ok_or_else(|| author_error("missing selected original dependency output"))?;
            if !program.steps.iter().any(|step| step.inputs.values().any(|input| input == reference)) {
                return Err(author_error(format!("original commands omit selected dependency {key}:{output}")));
            }
        }
    }
    Ok(())
}

/// Expand the independently evaluated seven-root union in dependency order.
/// No partial program is returned: every original source/build node must have a
/// genuine lowerer before any recipe/index can be admitted or executed.
pub(crate) fn author(
    request: &AuthoringRequest<'_>,
) -> Result<crate::source_built_program_authoring::AuthoredPrograms, crate::errors::RunError> {
    use crate::source_built_fixed_point::SourceAuthorityRole;
    use crate::source_built_fixed_point_native_actions::NativeExpandedDerivation;
    use crate::source_built_fixed_point_native_actions::NativeExpansionIndex;
    use crate::source_built_recipe::RecipePreparation;
    use crate::source_built_recipe::SourceRecipe;
    use crate::source_built_recipe::{self as recipe};
    let root = request
        .source_roots
        .get(&SourceAuthorityRole::MantleSource)
        .ok_or_else(|| author_error("missing authenticated original Mantle source"))?;
    let bundle = request
        .source_roots
        .get(&SourceAuthorityRole::NativeSourceBundle)
        .ok_or_else(|| author_error("missing authenticated original native source bundle"))?;
    for (role, path) in [
        (SourceAuthorityRole::MantleSource, root),
        (SourceAuthorityRole::NativeSourceBundle, bundle),
    ] {
        let source = request
            .proof
            .source_inputs
            .iter()
            .find(|source| source.role == role)
            .ok_or_else(|| author_error("missing original source authority"))?;
        if recipe::digest(path)?.1 != source.digest_blake3 {
            return Err(author_error("authenticated native source identity drift"));
        }
    }
    let manifest = crate::source_bundle::read_source_bundle(bundle)?;
    let graph = crate::source_built_fixed_point_native_actions::closure(root, &request.proof.logical_store_prefix)?;
    let reservations = crate::source_built_program_authoring::read_continuation_policy(root)?;
    let logical_outputs = logical_prefixes(request)?.derivation_outputs;
    let scratch = tempfile::Builder::new()
        .prefix("native-author-original-")
        .tempdir_in(request.scratch_root)
        .map_err(|error| author_error(error.to_string()))?;
    let mut recipe = SourceRecipe {
        schema: recipe::SCHEMA.into(), stage_id: STAGE.into(),
        preparation: RecipePreparation { tool: "mantle-original-native-source-author".into(), provenance: "complete evaluated seven-root closure, authenticated original fixed-output pins and source-owned individual phases".into() },
        steps: Vec::new(), timeout_groups: Vec::new(), roots: Vec::new(), exports: BTreeMap::new(), semantic_roots: BTreeMap::new(),
    };
    let mut bootstrap = recipe.clone();
    let splitter = append_stagex_c_helper(
        request,
        &mut bootstrap,
        "native-gcc-phase-splitter",
        "source-built/native/gcc-phases.c",
    )?;
    let mut bootstrap_steps = Some(bootstrap.steps);
    let mut sources = BTreeMap::<String, NativeSource>::new();
    let mut inspected = BTreeMap::new();
    let mut programs = BTreeMap::<String, NativeNodeProgram>::new();
    let mut expanded = BTreeMap::new();
    // Only genuine source outputs carry source lineage. Installed compiler,
    // library and tool outputs do not re-reserve their already-built sources.
    let mut source_lineage = BTreeMap::<String, BTreeSet<String>>::new();
    let mut reservation_report = Vec::new();
    let mut files = phase_sources();
    let prefix = &request.proof.logical_store_prefix;
    let mut pending = graph
        .derivations
        .iter()
        .map(|node| (node.drv_path.to_absolute_path_with_prefix(prefix), node))
        .collect::<BTreeMap<_, _>>();
    while !pending.is_empty() {
        let ready = pending
            .iter()
            .filter(|(_, node)| {
                node.derivation
                    .input_derivations
                    .keys()
                    .all(|path| programs.contains_key(&path.to_absolute_path_with_prefix(prefix)))
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        if ready.is_empty() {
            return Err(author_error("original native dependency graph has no producer-first expansion"));
        }
        for key in ready {
            let node = pending.remove(&key).expect("ready original node");
            let hdm = data_encoding::HEXLOWER.encode(&node.hash_derivation_modulo);
            let id = format!("original-{hdm}");
            let dependencies = node
                .derivation
                .input_derivations
                .keys()
                .map(|path| {
                    let key = path.to_absolute_path_with_prefix(prefix);
                    let program = programs.get(&key).expect("ready original dependency");
                    (key, program)
                })
                .collect::<BTreeMap<_, _>>();
            let mut selected_sources = BTreeSet::new();
            for (dependency, selected) in &node.derivation.input_derivations {
                let dependency = dependency.to_absolute_path_with_prefix(prefix);
                for output in selected {
                    let logical = logical_outputs
                        .get(&format!("{dependency}:{output}"))
                        .ok_or_else(|| author_error("selected native dependency lacks its original logical output"))?;
                    if let Some(origins) = source_lineage.get(logical) {
                        selected_sources.extend(origins.iter().cloned());
                    }
                }
            }
            let mut counted_records = BTreeSet::new();
            let native_source_files = selected_sources.iter().try_fold(0u32, |count, logical| {
                let source = sources
                    .get(logical)
                    .ok_or_else(|| author_error("original source lineage lacks its authenticated record"))?;
                if !counted_records.insert(source.record_identity.as_str()) {
                    return Ok(count);
                }
                count
                    .checked_add(source.native_source_files)
                    .ok_or_else(|| author_error("native source reservation count overflow"))
            })?;
            let mut program = if let Some(program) =
                original_source_node(request, node, &id, &manifest, scratch.path(), &mut inspected, &mut sources)?
            {
                for source in
                    sources.values().filter(|source| program.outputs.values().any(|output| output == &source.artifact))
                {
                    source_lineage
                        .insert(source.original_output.clone(), BTreeSet::from([source.original_output.clone()]));
                }
                program
            } else {
                let node_request = NativeNodeRequest {
                    request,
                    node,
                    id: &id,
                    dependencies: &dependencies,
                    sources: &sources,
                    gcc_splitter: &splitter,
                    logical_outputs: &logical_outputs,
                    reservations: &reservations,
                    native_source_files,
                };
                if let Some(program) = early_gnu::lower(&node_request)? {
                    program
                } else if let Some(program) = final_gnu::lower(&node_request)? {
                    program
                } else if let Some(program) = host::lower(&node_request)? {
                    program
                } else {
                    return Err(author_error(format!(
                        "original native derivation has no complete command/source lowering: {key}"
                    )));
                }
            };
            if node.derivation.builder != "builtin:fetchurl" && final_gnu::is_regenerated_source_node(node)? {
                for output in program.outputs.keys() {
                    let logical = logical_outputs
                        .get(&format!("{key}:{output}"))
                        .ok_or_else(|| author_error("regenerated source lacks its exact original logical output"))?;
                    source_lineage.insert(logical.clone(), selected_sources.clone());
                }
            }
            for step in &program.steps {
                if let RecipeStep {
                    operation: crate::source_built_recipe::RecipeOperation::Controller { specification, guard },
                    ..
                } = step
                {
                    let slots = u64::try_from(specification.slots.len())
                        .map_err(|_| author_error("native controller slot count overflow"))?;
                    let copies = specification.slots.iter().filter(|slot| matches!(slot.executable,
                        crate::source_built_stagex_programs::controller::ControllerExecutable::Inventory(_)
                            | crate::source_built_stagex_programs::controller::ControllerExecutable::RequestOutput { .. })).count() as u64;
                    // Each normal leaf has two ProtectedExec observations and
                    // two completions. Guarded leaves add two guard events;
                    // every slot has one pure phase observation each turn.
                    let copy_events = copies
                        .checked_mul(2)
                        .ok_or_else(|| author_error("native copy reservation event arithmetic overflow"))?;
                    let events_per_turn = slots
                        .checked_mul(7)
                        .and_then(|events| events.checked_add(copy_events))
                        .and_then(|events| events.checked_add(8 + if copies > 0 { 4 } else { 0 }))
                        .ok_or_else(|| author_error("native per-turn reservation event arithmetic overflow"))?;
                    let events = u64::try_from(specification.commands_max)
                        .ok()
                        .and_then(|commands| commands.checked_mul(events_per_turn))
                        .and_then(|events| events.checked_add(8 + u64::from(guard.is_some())))
                        .ok_or_else(|| author_error("native controller reservation event arithmetic overflow"))?;
                    reservation_report.push(serde_json::json!({"derivation": key, "controller": specification.id, "node_native_source_files": native_source_files,
                        "commands_reserved": specification.commands_max, "events_per_turn_upper_bound": events_per_turn, "controller_event_upper_bound": events,
                        "source_outputs": selected_sources}));
                }
            }
            if program.steps.iter().any(|step| step.inputs.values().any(|input| input == &splitter))
                && let Some(mut steps) = bootstrap_steps.take()
            {
                steps.append(&mut program.steps);
                program.steps = steps;
            }
            validate_node_program(node, &program, &dependencies, prefix)?;
            expanded.insert(key.clone(), NativeExpandedDerivation {
                hash_derivation_modulo: hdm,
                input_derivations: node
                    .derivation
                    .input_derivations
                    .iter()
                    .map(|(path, outputs)| (path.to_absolute_path_with_prefix(prefix), outputs.clone()))
                    .collect(),
                input_sources: node
                    .derivation
                    .input_sources
                    .iter()
                    .map(|path| path.to_absolute_path_with_prefix(prefix))
                    .collect(),
                step_ids: program.steps.iter().map(|step| step.id.clone()).collect(),
                outputs: program.outputs.clone(),
            });
            for (path, text) in std::mem::take(&mut program.files) {
                recipe::relative(&path, false)?;
                recipe::validate_text(&text)?;
                if let Some(old) = files.get(&path) {
                    if old != &text {
                        return Err(author_error(format!("conflicting original helper source {path}")));
                    }
                } else {
                    files.insert(path, text);
                }
            }
            recipe.steps.append(&mut program.steps);
            recipe.timeout_groups.append(&mut program.timeout_groups);
            programs.insert(key, program);
        }
    }
    export_original_products(request, &graph, &programs, &sources, &splitter, &mut recipe)?;
    files.insert(
        "source-built/native/continuation-reservations.json".into(),
        serde_json::to_string_pretty(&reservation_report)
            .map_err(|cause| author_error(format!("native source reservation report: {cause}")))?,
    );
    recipe::validate_source_recipe(&recipe)?;
    let index = NativeExpansionIndex {
        schema: "mantle-native-source-expansion-v1".into(),
        recipe_member: RECIPE_MEMBER.into(),
        roots: graph.roots.iter().map(|root| root.drv_path.to_absolute_path_with_prefix(prefix)).collect(),
        derivations: expanded,
    };
    files.insert(
        RECIPE_MEMBER.into(),
        serde_json::to_string_pretty(&recipe).map_err(|error| author_error(error.to_string()))?,
    );
    files.insert(
        INDEX_MEMBER.into(),
        serde_json::to_string_pretty(&index).map_err(|error| author_error(error.to_string()))?,
    );
    let exports = recipe.exports.into_iter().map(|(name, reference)| (format!("{STAGE}:{name}"), reference)).collect();
    Ok(crate::source_built_program_authoring::AuthoredPrograms { files, exports })
}

fn export_original_products(
    request: &AuthoringRequest<'_>,
    graph: &crunch_pipeline::native_preflight::NativeDerivationClosure,
    programs: &BTreeMap<String, NativeNodeProgram>,
    sources: &BTreeMap<String, NativeSource>,
    splitter: &ArtifactRef,
    recipe: &mut crate::source_built_recipe::SourceRecipe,
) -> Result<(), crate::errors::RunError> {
    let prefix = &request.proof.logical_store_prefix;
    let provider_root = graph.roots.first().ok_or_else(|| author_error("empty original native root union"))?;
    let provider = programs
        .get(&provider_root.drv_path.to_absolute_path_with_prefix(prefix))
        .ok_or_else(|| author_error("original provider was not expanded"))?;
    let provider_tree = provider
        .outputs
        .get("out")
        .ok_or_else(|| author_error("original normalized provider out is missing"))?;
    if !matches!(provider_tree, ArtifactRef::Produced { output_id, .. } if output_id == STAGE) {
        return Err(author_error(
            "original normalized provider must itself produce canonical full-source-native-provider output",
        ));
    }
    let members = provider
        .members
        .get("out")
        .ok_or_else(|| author_error("original normalized provider has no exact installed member ancestry"))?;
    for (role, path) in [
        ("native.cc1", "libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1"),
        ("native.cc1plus", "libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1plus"),
        ("native.gcc", "bin/gcc.real"),
        ("native.gxx", "bin/g++.real"),
        ("native.collect2-query", "libexec/gcc/x86_64-unknown-linux-musl/10.5.0/collect2"),
        ("native.as", "bin/as"),
        ("native.ld", "bin/ld"),
        ("native.ar", "bin/ar"),
        ("native.nm", "bin/nm"),
        ("native.ranlib", "bin/ranlib"),
        ("native.objcopy", "bin/objcopy"),
        ("native.strip", "bin/strip"),
        ("native.readelf", "bin/readelf"),
        ("native.loader", "x86_64-linux-musl/lib/ld-musl-x86_64.so.1"),
        ("native.libc", "x86_64-linux-musl/lib/libc.a"),
        ("native.libc-shared", "x86_64-linux-musl/lib/libc.so"),
        ("native.libgcc", "x86_64-linux-musl/lib/libgcc.a"),
        ("native.libgcc-eh", "x86_64-linux-musl/lib/libgcc_eh.a"),
        ("native.libgcc-s", "x86_64-linux-musl/lib/libgcc_s.so.1"),
        ("native.libstdcxx", "x86_64-linux-musl/lib/libstdc++.a"),
    ] {
        let reference = members
            .get(path)
            .ok_or_else(|| author_error(format!("original normalized provider lacks exact member {path}")))?;
        if !matches!(reference, ArtifactRef::Produced { .. }) {
            return Err(author_error("native installed member cannot be imported authority"));
        }
        recipe.exports.insert(role.into(), reference.clone());
    }
    recipe.exports.insert("native.gcc-splitter".into(), splitter.clone());
    for role in ["native.provider", "native.sysroot", "native.runtime"] {
        // The data-tree layout stays original: target includes/libraries live
        // under x86_64-linux-musl/. Executable authority remains per member.
        recipe.exports.insert(role.into(), provider_tree.clone());
    }
    for (path, reference) in members {
        recipe.exports.insert(format!("native.member:{path}"), reference.clone());
    }
    for (role, suffix) in [
        ("native.gcc-tree", "-gcc-10.5.0-musl-final-v1.drv"),
        ("native.musl-tree", "-musl-1.2.5-gcc10-v1.drv"),
        ("native.binutils-tree", "-binutils-2.41-gcc10-v1.drv"),
    ] {
        let mut candidates = programs.iter().filter(|(path, _)| path.ends_with(suffix));
        let (_, node) = candidates
            .next()
            .ok_or_else(|| author_error(format!("missing original compiler output {suffix}")))?;
        if candidates.next().is_some() {
            return Err(author_error("ambiguous original compiler output"));
        }
        recipe.exports.insert(
            role.into(),
            node.outputs.get("out").ok_or_else(|| author_error("original compiler out is missing"))?.clone(),
        );
    }
    for (member, role, executable) in [
        ("bootstrap/make-4.4.1-gcc10.ncl", "host.make", "bin/make"),
        ("bootstrap/linux-headers-6.6-gcc10.ncl", "linux.headers", ""),
        ("bootstrap/busybox-1.37.0-gcc10.ncl", "host.busybox", "bin/busybox"),
        ("bootstrap/cmake-3.31.8-gcc10.ncl", "host.cmake", "bin/cmake"),
        ("bootstrap/python-3.13.5-gcc10.ncl", "host.python", "bin/python3"),
        ("bootstrap/perl-5.10.1-gcc10.ncl", "host.perl", "bin/perl"),
    ] {
        let mut roots = graph.roots.iter().filter(|root| root.recipe.ends_with(member));
        let root = roots.next().ok_or_else(|| author_error(format!("missing selected original root {member}")))?;
        if roots.next().is_some() {
            return Err(author_error("ambiguous selected original host root"));
        }
        let node = &programs[&root.drv_path.to_absolute_path_with_prefix(prefix)];
        let tree = node.outputs.get("out").ok_or_else(|| author_error("selected original host out is missing"))?;
        let reference = if executable.is_empty() {
            tree
        } else {
            node.members
                .get("out")
                .and_then(|members| members.get(executable))
                .ok_or_else(|| author_error(format!("selected original host lacks member {executable}")))?
        };
        recipe.exports.insert(role.into(), reference.clone());
        if let Some(host) = role.strip_prefix("host.") {
            recipe.exports.insert(format!("host.{host}-root"), tree.clone());
        }
        if let Some(host) = role.strip_prefix("host.")
            && let Some(phase) = node.phase_tools.get(host)
        {
            if !matches!(phase, ArtifactRef::Produced { .. }) {
                return Err(author_error("host phase entry is not actually source-produced"));
            }
            recipe.exports.insert(format!("host.{host}-phase"), phase.clone());
            recipe.exports.insert(format!("host.{host}-phase-root"), tree.clone());
        }
    }
    for node in &graph.derivations {
        if node.derivation.builder != "builtin:fetchurl" {
            continue;
        }
        let name = environment(node, "name").ok_or_else(|| author_error("original fetcher name is missing"))?;
        let output = node
            .derivation
            .outputs
            .values()
            .next()
            .and_then(|output| output.path.as_ref())
            .ok_or_else(|| author_error("original fetcher path is missing"))?
            .to_absolute_path_with_prefix(prefix);
        let original =
            sources.get(&output).ok_or_else(|| author_error("original source materialization is missing"))?;
        if recipe.exports.insert(format!("source:{name}"), original.artifact.clone()).is_some() {
            return Err(author_error("ambiguous original source export name"));
        }
    }
    // Preserve the selected original union as products, never merely semantic
    // labels over a reduced compiler or source-only helper closure.
    recipe.roots = programs
        .values()
        .flat_map(|program| program.outputs.values().cloned())
        .chain(recipe.exports.values().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(())
}
