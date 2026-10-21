//! Original early GNU source phases and explicitly bound StageX adapters.
//!
//! Original configure/build scripts expand through the shared source controller;
//! GCC4 specs and collect2 select individual real phase commands. StageX adapters
//! verify raw installed members; Make additionally binds its declared real
//! phase-runtime variant instead of claiming byte identity with raw Make.
use std::collections::BTreeMap;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::RECORD_IDENTITY;
use crate::source_built_native_programs::STAGE;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::WordPart;

#[path = "source_built_native_programs_early_gnu_link_seams.rs"]
mod link_seams;
pub(crate) use link_seams::Gcc4LinkSeam;
pub(crate) use link_seams::Gcc4LinkSeamInvocation;
pub(crate) use link_seams::append_gcc4_link_seam;
#[path = "source_built_native_programs_early_gnu_generator_base.rs"]
mod generator_base;
#[path = "source_built_native_programs_early_gnu_originals.rs"]
mod originals;
#[path = "source_built_native_programs_early_gnu_perl_child.rs"]
mod perl_child;
pub(crate) use perl_child::append_perl_process_source;
#[path = "source_built_native_programs_early_gnu_tcc.rs"]
mod tcc;

const MEMBER_PREFIX: &str = "stagex-transition:member:";
const ADAPTER_MEMBER: &str = "source-built/native/early-gnu-adapter.c";
const FILE_MAX: u64 = 64 * 1024 * 1024;

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("original early native GNU lowering: {}", message.into()))
}
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
fn flag(prefix: &str, name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: prefix.into() }, WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn bounded(kind: ArtifactKind, bytes_max: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max }
}
fn produced(id: &str, name: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: name.into(),
    }
}
fn predecessor(request: &NativeNodeRequest<'_, '_>, key: &str) -> Result<ArtifactRef, RunError> {
    match request.request.predecessor_exports.get(key) {
        Some(reference @ ArtifactRef::Produced { .. }) => Ok(reference.clone()),
        Some(_) => Err(error(format!("predecessor {key} is not an individual Produced artifact"))),
        None => Err(error(format!("missing exact predecessor export {key}"))),
    }
}
fn exec(
    program: &mut NativeNodeProgram,
    id: String,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
    kind: ArtifactKind,
) -> ArtifactRef {
    inputs.insert("executable".into(), executable);
    let result = produced(&id, "result");
    program.steps.push(RecipeStep {
        id,
        inputs,
        outputs: BTreeMap::from([
            ("result".into(), bounded(kind, FILE_MAX)),
            ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
            ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
        ]),
        operation: RecipeOperation::Exec {
            executable_input: "executable".into(),
            arguments,
            environment: BTreeMap::new(),
            invocation_input: None,
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            merge_stderr: false,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
        },
    });
    result
}
fn c_helper(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    member: &str,
    source: &str,
    suffix: &str,
) -> Result<ArtifactRef, RunError> {
    program.files.insert(member.into(), source.into());
    let source = ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: RECORD_IDENTITY.into(),
        member: member.into(),
    };
    let inputs = BTreeMap::from([
        ("source".into(), source),
        ("headers".into(), predecessor(request, "stagex-provider-publication:musl.headers")?),
        ("libraries".into(), predecessor(request, "stagex-provider-publication:musl.libraries")?),
        ("runtime".into(), predecessor(request, "stagex-provider-publication:tcc.runtime")?),
    ]);
    let args = vec![
        flag("-B", "runtime", ""),
        literal("-nostdinc"),
        flag("-I", "runtime", "include"),
        flag("-I", "headers", ""),
        literal("-nostdlib"),
        literal("-static"),
        input("libraries", "crt1.o"),
        input("libraries", "crti.o"),
        input("source", ""),
        input("libraries", "libc.a"),
        input("runtime", "libtcc1.a"),
        input("libraries", "crtn.o"),
        literal("-o"),
        output("result"),
    ];
    Ok(exec(
        program,
        format!("{}.{suffix}", request.id),
        predecessor(request, "stagex-provider-publication:tcc")?,
        inputs,
        args,
        ArtifactKind::Executable,
    ))
}

/// Transform the authenticated original collect2 source. This is only source
/// authoring: the caller must still compile the real collect2 translation unit,
/// link its original dependencies and use its query output for the linker phase.
pub(crate) fn append_collect2_phase_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    phase: &str,
    original: ArtifactRef,
    member: &str,
) -> Result<ArtifactRef, RunError> {
    crate::source_built_recipe::relative(phase, false)?;
    crate::source_built_recipe::relative(member, true)?;
    if !matches!(original, ArtifactRef::Produced { .. }) {
        return Err(error("original GCC4 collect2 must come from its authenticated source-materialization producer"));
    }
    const PATCH_MEMBER: &str = "source-built/native/early-gnu-collect2-patch.c";
    const HEADER_MEMBER: &str = "source-built/native/early-gnu-collect2-phase.inc";
    program.files.insert(
        HEADER_MEMBER.into(),
        include_str!("source_built_native_programs_early_gnu_collect2_phase.inc").into(),
    );
    let manifest = |member: &str| ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: RECORD_IDENTITY.into(),
        member: member.into(),
    };
    const KEY: &str = "gcc4-collect2-source-tool";
    let helper = if let Some(reference) = program.phase_tools.get(KEY) {
        reference.clone()
    } else {
        let helper = c_helper(
            request,
            program,
            PATCH_MEMBER,
            include_str!("source_built_native_programs_early_gnu_collect2_patch.c"),
            "gcc4-collect2-source-tool",
        )?;
        program.phase_tools.insert(KEY.into(), helper.clone());
        helper
    };
    Ok(exec(
        program,
        format!("{}.gcc4-{phase}-collect2-source", request.id),
        helper,
        BTreeMap::from([
            ("original".into(), original),
            ("phase-source".into(), manifest(HEADER_MEMBER)),
        ]),
        vec![input("original", member), input("phase-source", ""), output("result")],
        ArtifactKind::Text,
    ))
}

/// GCC4 does not print its derived child environment under -###. Observe the
/// actual original build_search_list/set_collect_gcc_options results only in
/// that existing no-child mode, so the common phase scheduler can transfer
/// ordered metadata rather than invent compiler search paths or option flags.
pub(crate) fn append_gcc4_query_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    phase: &str,
    original: ArtifactRef,
    member: &str,
) -> Result<ArtifactRef, RunError> {
    crate::source_built_recipe::relative(phase, false)?;
    crate::source_built_recipe::relative(member, true)?;
    if !matches!(original, ArtifactRef::Produced { .. }) {
        return Err(error("original GCC4 query driver must come from its actual source producer"));
    }
    const KEY: &str = "gcc4-query-source-tool";
    const MEMBER: &str = "source-built/native/early-gnu-gcc4-query-patch.c";
    let helper = if let Some(reference) = program.phase_tools.get(KEY) {
        reference.clone()
    } else {
        let helper = c_helper(
            request,
            program,
            MEMBER,
            include_str!("source_built_native_programs_early_gnu_gcc4_query_patch.c"),
            "gcc4-query-source-tool",
        )?;
        program.phase_tools.insert(KEY.into(), helper.clone());
        helper
    };
    Ok(exec(
        program,
        format!("{}.gcc4-{phase}-query-source", request.id),
        helper,
        BTreeMap::from([("original".into(), original)]),
        vec![input("original", member), output("result")],
        ArtifactKind::Text,
    ))
}

/// Retain the complete authenticated original source tree while installing
/// both query-only source seams. Original configure/Make/recipe preparation
/// still runs over this tree; no generated provider output is substituted.
pub(crate) fn append_gcc4_phase_ready_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    phase: &str,
    original: ArtifactRef,
    root: &str,
) -> Result<ArtifactRef, RunError> {
    crate::source_built_recipe::relative(root, true)?;
    let member = |path: &str| {
        if root.is_empty() {
            path.into()
        } else {
            format!("{root}/{path}")
        }
    };
    let driver_member = member("gcc/gcc.c");
    let collect_member = member("gcc/collect2.c");
    let driver = append_gcc4_query_source(request, program, phase, original.clone(), &driver_member)?;
    let collect = append_collect2_phase_source(request, program, phase, original.clone(), &collect_member)?;
    let cp = predecessor(request, "stagex-transition:member:coreutils-stage/runtime/output/bin/cp")?;
    let mut state = original;
    for (name, data, path) in [("driver", driver, driver_member), ("collect2", collect, collect_member)] {
        let id = format!("{}.gcc4-{phase}-install-{name}-query-source", request.id);
        program.steps.push(RecipeStep {
            id: id.clone(),
            inputs: BTreeMap::from([
                ("cp".into(), cp.clone()),
                ("source".into(), data),
                ("prior".into(), state),
            ]),
            outputs: BTreeMap::from([
                ("result".into(), bounded(ArtifactKind::Tree, request.request.proof.resource_bounds.disk_bytes_max)),
                ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
                ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
            ]),
            operation: RecipeOperation::Exec {
                executable_input: "cp".into(),
                arguments: vec![input("source", ""), RecipeWord {
                    parts: vec![WordPart::Output {
                        name: "result".into(),
                        path,
                    }],
                }],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: Some("result".into()),
                output_seeds: BTreeMap::from([("result".into(), "prior".into())]),
                guard: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                command_state: None,
            },
        });
        state = produced(&id, "result");
    }
    Ok(state)
}

/// Expand original early GNU configure/build nodes, the generator bundle and
/// POSIX/Make adapters. Selection uses the evaluated original derivation name.
pub(crate) fn lower(request: &NativeNodeRequest<'_, '_>) -> Result<Option<NativeNodeProgram>, RunError> {
    if let Some(program) = originals::lower(request)? {
        return Ok(Some(program));
    }
    let name: Option<&[u8]> = request.node.derivation.environment.get("name").map(|value| value.as_ref());
    if name == Some(b"gcc-generator-base-v4".as_slice()) {
        return generator_base::lower(request).map(Some);
    }
    let stage0 = match name {
        Some(b"stage0-posix") => true,
        Some(b"make-3.82-tcc") => false,
        _ => return Ok(None),
    };
    // A literal StorePath imports an Input::Source, not a native derivation.
    // Bind the freshly executed transition export only after matching that
    // original source edge; the logical path itself authorizes no bytes.
    let mut originals = include_str!("../bootstrap/stagex-transition-proof-input.ncl")
        .lines()
        .filter(|line| line.starts_with('"'));
    let original = originals
        .next()
        .and_then(|line| line.split('"').nth(1))
        .ok_or_else(|| error("original transition proof input has no literal StorePath"))?;
    if originals.next().is_some() {
        return Err(error("original transition proof input has ambiguous literal StorePaths"));
    }
    let basename = original.rsplit('/').next().ok_or_else(|| error("invalid original transition StorePath"))?;
    let expected = format!("{}/{basename}", request.request.proof.logical_store_prefix.trim_end_matches('/'));
    if !request.dependencies.is_empty()
        || !request.node.derivation.input_derivations.is_empty()
        || request.node.derivation.input_sources.len() != 1
        || request
            .node
            .derivation
            .input_sources
            .iter()
            .next()
            .map(|path| path.to_absolute_path_with_prefix(&request.request.proof.logical_store_prefix))
            .as_deref()
            != Some(expected.as_str())
    {
        return Err(error("StageX compatibility adapter must retain exactly its original transition source edge"));
    }
    let transition = predecessor(request, "stagex-transition:stagex-transition")?;
    let mut program = NativeNodeProgram {
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        outputs: BTreeMap::new(),
        members: BTreeMap::new(),
        phase_tools: BTreeMap::new(),
        files: BTreeMap::new(),
    };
    if !stage0 {
        program
            .phase_tools
            .insert("make".into(), predecessor(request, "stagex-transition:phase-tool:make-3.82")?);
    }
    let helper = c_helper(
        request,
        &mut program,
        ADAPTER_MEMBER,
        include_str!("source_built_native_programs_early_gnu_adapter.c"),
        "adapter-build",
    )?;
    let runtime_prefix = if stage0 {
        "coreutils-stage/runtime/output/bin/"
    } else {
        "make-stage/runtime/output/"
    };
    let mut selected = BTreeMap::<String, (String, ArtifactRef)>::new();
    for (key, reference) in request.request.predecessor_exports {
        let Some(path) = key.strip_prefix(MEMBER_PREFIX) else {
            continue;
        };
        let Some(relative) = path.strip_prefix(runtime_prefix) else {
            continue;
        };
        crate::source_built_recipe::relative(relative, false)?;
        if stage0 && relative.contains('/') {
            return Err(error("unexpected nested executable in original coreutils bin"));
        }
        if !matches!(reference, ArtifactRef::Produced { .. }) {
            return Err(error(format!("non-Produced installed transition member {path}")));
        }
        let installed = if stage0 {
            format!("bin/{relative}")
        } else {
            relative.into()
        };
        selected.insert(installed, (path.into(), reference.clone()));
    }
    if stage0 {
        let path = "bash-full-stage/runtime/output/bin/bash-full";
        let bash = predecessor(request, &format!("{MEMBER_PREFIX}{path}"))?;
        selected.insert("bin/sh".into(), (path.into(), bash.clone()));
        selected.insert("bin/bash".into(), (path.into(), bash));
    }
    let required = if stage0 { "bin/cp" } else { "bin/make" };
    if !selected.contains_key(required) {
        return Err(error(format!("missing original installed member {required}")));
    }
    let mut installed = BTreeMap::new();
    for (ordinal, (destination, (source_path, reference))) in selected.into_iter().enumerate() {
        let executable = stage0 || destination.starts_with("bin/");
        let (inputs, args) = if !stage0 && destination == "bin/make" {
            let inputs = BTreeMap::from([
                ("member".into(), program.phase_tools["make"].clone()),
                ("phase-runtime".into(), predecessor(request, "stagex-provider-publication:phase-runtime")?),
                ("raw-member".into(), reference),
                ("transition".into(), transition.clone()),
            ]);
            let args = vec![
                input("member", ""),
                input("phase-runtime", "make-stage/runtime/output/bin/make"),
                output("result"),
                literal("phase-executable"),
                input("raw-member", ""),
                input("transition", &source_path),
            ];
            (inputs, args)
        } else {
            (BTreeMap::from([("member".into(), reference), ("transition".into(), transition.clone())]), vec![
                input("member", ""),
                input("transition", &source_path),
                output("result"),
                literal(if executable { "executable" } else { "data" }),
            ])
        };
        let result = exec(
            &mut program,
            format!("{}.install-{ordinal}", request.id),
            helper.clone(),
            inputs,
            args,
            if executable {
                ArtifactKind::Executable
            } else {
                ArtifactKind::Object
            },
        );
        installed.insert(destination, result);
    }
    let id = format!("{}.installed", request.id);
    let mut inputs = BTreeMap::new();
    let mut entries = BTreeMap::new();
    for (ordinal, (path, reference)) in installed.iter().enumerate() {
        let key = format!("member-{ordinal}");
        inputs.insert(key.clone(), reference.clone());
        entries.insert(path.clone(), key);
    }
    program.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs: BTreeMap::from([(
            "out".into(),
            bounded(ArtifactKind::Tree, request.request.proof.resource_bounds.disk_bytes_max),
        )]),
        operation: RecipeOperation::AssembleTree {
            output: "out".into(),
            entries,
        },
    });
    program.outputs.insert("out".into(), produced(&id, "out"));
    program.members.insert("out".into(), installed);
    Ok(Some(program))
}
