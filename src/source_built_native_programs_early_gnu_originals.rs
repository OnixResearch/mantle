//! Original selected early GNU nodes. Source overrides add only no-child entry
//! seams; configure, regeneration, make rules, probes and checks remain the
//! evaluated original scripts interpreted by the common source controller.
use std::collections::BTreeMap;

use super::FILE_MAX;
use super::bounded;
use super::c_helper;
use super::error;
use super::exec;
use super::input;
use super::predecessor;
use super::produced;
use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::NativeSource;
use crate::source_built_native_programs::RECORD_IDENTITY;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::WordPart;

type Files = BTreeMap<String, ArtifactRef>;
pub(crate) struct EarlyOriginal {
    pub preparation: NativeNodeProgram,
    pub source_overrides: Files,
    pub literal_overrides: Files,
    pub members: BTreeMap<String, BTreeMap<String, ArtifactKind>>,
    pub phase_members: BTreeMap<String, (String, String)>,
}
fn source<'a>(
    request: &'a NativeNodeRequest<'_, '_>,
    suffix: &str,
    package: &str,
    required: &str,
) -> Result<(&'a NativeSource, String), RunError> {
    let mut candidates = request.sources.values().filter(|source| {
        source.original_output.ends_with(suffix)
            && request
                .dependencies
                .values()
                .any(|program| program.outputs.values().any(|reference| reference == &source.artifact))
    });
    let source = candidates
        .next()
        .ok_or_else(|| error(format!("missing original direct source dependency {suffix}:{required}")))?;
    if candidates.next().is_some() {
        return Err(error(format!("ambiguous original direct source dependency {suffix}")));
    }
    let root = ["", package]
        .into_iter()
        .find(|root| source.preparation_path.join(root).join(required).is_file())
        .ok_or_else(|| error(format!("missing original source member {}:{required}", source.record_identity)))?;
    Ok((source, root.into()))
}
fn member(root: &str, path: &str) -> String {
    if root.is_empty() {
        path.into()
    } else {
        format!("{root}/{path}")
    }
}
pub(super) fn install_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    original: ArtifactRef,
    data: ArtifactRef,
    path: &str,
    suffix: &str,
) -> Result<ArtifactRef, RunError> {
    let id = format!("{}.early-{suffix}-source-tree", request.id);
    program.steps.push(RecipeStep {
        id: id.clone(),
        inputs: Files::from([
            (
                "copy".into(),
                predecessor(request, "stagex-transition:member:coreutils-stage/runtime/output/bin/cp")?,
            ),
            ("prior".into(), original),
            ("source".into(), data),
        ]),
        outputs: BTreeMap::from([
            ("result".into(), bounded(ArtifactKind::Tree, request.request.proof.resource_bounds.disk_bytes_max)),
            ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
            ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
        ]),
        operation: RecipeOperation::Exec {
            executable_input: "copy".into(),
            arguments: vec![input("source", ""), RecipeWord {
                parts: vec![WordPart::Output {
                    name: "result".into(),
                    path: path.into(),
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
    Ok(produced(&id, "result"))
}
fn manifest(member: &str) -> ArtifactRef {
    ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: RECORD_IDENTITY.into(),
        member: member.into(),
    }
}
fn bison_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    original: ArtifactRef,
    root: &str,
) -> Result<ArtifactRef, RunError> {
    const PATCH: &str = "source-built/native/early-gnu-bison23-patch.c";
    const PHASE: &str = "source-built/native/early-gnu-bison23-phase.inc";
    program
        .files
        .insert(PHASE.into(), include_str!("source_built_native_programs_early_gnu_bison23_phase.inc").into());
    let helper = c_helper(
        request,
        program,
        PATCH,
        include_str!("source_built_native_programs_early_gnu_bison23_patch.c"),
        "bison23-source-helper",
    )?;
    let path = member(root, "src/output.c");
    let transformed = exec(
        program,
        format!("{}.early-bison23-source", request.id),
        helper,
        Files::from([("original".into(), original.clone()), ("phase".into(), manifest(PHASE))]),
        vec![input("original", &path), input("phase", ""), super::output("result")],
        ArtifactKind::Text,
    );
    install_source(request, program, original, transformed, &path, "bison23")
}
fn m4_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    original: ArtifactRef,
    root: &str,
) -> Result<ArtifactRef, RunError> {
    const PATCH: &str = "source-built/native/early-gnu-m4-patch.c";
    const ENTRY: &str = "source-built/native/early-gnu-m4-response-entry.c";
    const CHILD: &str = "source-built/native/early-gnu-m4-child.c";
    let entry = include_str!("source_built_native_programs_argv_main.c");
    let declaration = "extern int mantle_phase_original_main(int, char **);";
    let invocation = "return mantle_phase_original_main((int)count, arguments);";
    if entry.matches(declaration).count() != 1 || entry.matches(invocation).count() != 1 {
        return Err(error("shared response entry source ABI changed"));
    }
    let entry = entry
        .replace(
            declaration,
            "extern char **environ;\nextern int mantle_m4_original_main(int, char *const *, char *const *);",
        )
        .replace(invocation, "return mantle_m4_original_main((int)count, arguments, environ);");
    program.files.insert(ENTRY.into(), entry);
    program.files.insert(
        CHILD.into(),
        format!(
            "{}\n{}\n",
            crate::source_built_native_programs::final_gnu::perl_transport_source(),
            include_str!("source_built_native_programs_early_gnu_m4_child.inc")
        ),
    );
    let helper = c_helper(
        request,
        program,
        PATCH,
        include_str!("source_built_native_programs_early_gnu_m4_patch.c"),
        "m4-source-helper",
    )?;
    let path = member(root, "src/m4.c");
    let transformed = exec(
        program,
        format!("{}.early-m4-source", request.id),
        helper.clone(),
        Files::from([("original".into(), original.clone()), ("entry".into(), manifest(ENTRY))]),
        vec![
            super::literal("main"),
            input("original", &path),
            input("entry", ""),
            super::output("result"),
        ],
        ArtifactKind::Object,
    );
    let state = install_source(request, program, original, transformed, &path, "m4")?;
    let path = member(root, "src/builtin.c");
    let transformed = exec(
        program,
        format!("{}.early-m4-child-source", request.id),
        helper,
        Files::from([("original".into(), state.clone()), ("entry".into(), manifest(CHILD))]),
        vec![
            super::literal("builtin"),
            input("original", &path),
            input("entry", ""),
            super::output("result"),
        ],
        ArtifactKind::Object,
    );
    install_source(request, program, state, transformed, &path, "m4-child")
}

/// Complete member contracts are required only where the original node itself
/// requires those installed files. In particular initial GCC4 may finish with
/// its original pass2 frontier record and no driver; do not eagerly extract it.
pub(crate) fn prepare(request: &NativeNodeRequest<'_, '_>) -> Result<Option<EarlyOriginal>, RunError> {
    let name = request
        .node
        .derivation
        .environment
        .get("name")
        .map(|value| std::str::from_utf8(value.as_ref()))
        .transpose()
        .map_err(|_| error("non-UTF8 original early GNU node name"))?
        .unwrap_or("");
    let gcc = matches!(
        name,
        "gcc-4.0.4-native-gas-v45"
            | "gcc-4.0.4-musl-pass3-v23"
            | "gcc-4.0.4-musl-pass4-v7"
            | "gcc-4.0.4-musl-pass5-v2"
            | "gcc-4.0.4-musl-pass6-v1"
            | "gcc-4.0.4-musl-cxx-v8"
    );
    let musl = matches!(
        name,
        "musl-1.1.24-native-candidate"
            | "musl-1.1.24-gcc-pass3-v5"
            | "musl-1.1.24-gcc-pass4-v2"
            | "musl-1.1.24-gcc-pass5-v1"
    );
    let perl = matches!(
        name,
        "perl-5.000-gcc-v7"
            | "perl-5.003-gcc-v6"
            | "perl-5.004_05-gcc-v5"
            | "perl-5.005_03-gcc-v7"
            | "perl-5.6.2-gcc-v19"
            | "perl-5.6.2-configured-gcc-v16"
    );
    let other = matches!(
        name,
        "tcc-0.9.27-musl-native-runtime"
            | "binutils-2.30-gas-source-v41"
            | "binutils-2.30-tcc-source-v1"
            | "m4-1.4.7-gcc-v5"
            | "gawk-3.0.4-gcc-v4"
            | "grep-2.4-gcc-v10"
            | "heirloom-devtools-070527-gcc-v6"
            | "bison-2.3-gcc-v9"
    );
    if !gcc && !musl && !perl && !other {
        return Ok(None);
    }
    let mut plan = EarlyOriginal {
        preparation: NativeNodeProgram {
            steps: Vec::new(),
            timeout_groups: Vec::new(),
            outputs: Files::new(),
            members: BTreeMap::new(),
            phase_tools: Files::new(),
            files: BTreeMap::new(),
        },
        source_overrides: Files::new(),
        literal_overrides: Files::new(),
        members: BTreeMap::new(),
        phase_members: BTreeMap::new(),
    };
    plan.literal_overrides =
        super::tcc::literal_overrides(request, &mut plan.preparation, name == "tcc-0.9.27-musl-native-runtime")?;
    let mut installed = BTreeMap::new();
    if gcc {
        let (original, root) = source(request, "-gcc-4.0.4-native-src", "gcc-4.0.4", "gcc/gcc.c")?;
        let ready = super::append_gcc4_phase_ready_source(
            request,
            &mut plan.preparation,
            "original",
            original.artifact.clone(),
            &root,
        )?;
        plan.source_overrides.insert(original.original_output.clone(), ready);
        if name != "gcc-4.0.4-native-gas-v45" {
            for (capability, path) in [
                ("gcc", "bin/gcc"),
                ("cc1", "libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1"),
                ("collect2", "libexec/gcc/x86_64-unknown-linux-musl/4.0.4/collect2"),
            ] {
                installed.insert(path.into(), ArtifactKind::Executable);
                plan.phase_members.insert(capability.into(), ("out".into(), path.into()));
            }
            installed.insert("lib/gcc/x86_64-unknown-linux-musl/4.0.4/libgcc.a".into(), ArtifactKind::Archive);
            if name == "gcc-4.0.4-musl-cxx-v8" {
                for path in [
                    "bin/g++",
                    "bin/c++",
                    "libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1plus",
                ] {
                    installed.insert(path.into(), ArtifactKind::Executable);
                }
            }
        }
    } else if musl {
        for path in ["lib/libc.a", "lib/libm.a"] {
            installed.insert(path.into(), ArtifactKind::Archive);
        }
        for path in ["lib/crt1.o", "lib/crti.o", "lib/crtn.o"] {
            installed.insert(path.into(), ArtifactKind::Object);
        }
        installed.insert("include".into(), ArtifactKind::Tree);
    } else if perl {
        installed.insert("bin/perl".into(), ArtifactKind::Executable);
        if name != "perl-5.6.2-configured-gcc-v16" {
            installed.insert("bin/miniperl".into(), ArtifactKind::Executable);
        }
        installed.insert("lib".into(), ArtifactKind::Tree);
        if matches!(name, "perl-5.6.2-gcc-v19" | "perl-5.6.2-configured-gcc-v16") {
            let (original, root) = source(request, "-perl-5.6.2-src", "perl-5.6.2", "util.c")?;
            let transport = crate::source_built_native_programs::final_gnu::perl_transport_source();
            let ready = super::append_perl_process_source(
                request,
                &mut plan.preparation,
                original.artifact.clone(),
                &root,
                &transport,
            )?;
            plan.source_overrides.insert(original.original_output.clone(), ready);
            plan.phase_members.insert("perl".into(), ("out".into(), "bin/perl".into()));
        }
    } else {
        match name {
            "m4-1.4.7-gcc-v5" => {
                let (original, root) = source(request, "-m4-1.4.7-src", "m4-1.4.7", "src/m4.c")?;
                let ready = m4_source(request, &mut plan.preparation, original.artifact.clone(), &root)?;
                plan.source_overrides.insert(original.original_output.clone(), ready);
                installed.insert("bin/m4".into(), ArtifactKind::Executable);
                plan.phase_members.insert("m4".into(), ("out".into(), "bin/m4".into()));
            }
            "bison-2.3-gcc-v9" => {
                let (original, root) = source(request, "-bison-2.3-src", "bison-2.3", "src/output.c")?;
                let ready = bison_source(request, &mut plan.preparation, original.artifact.clone(), &root)?;
                plan.source_overrides.insert(original.original_output.clone(), ready);
                installed.insert("bin/bison".into(), ArtifactKind::Executable);
                installed.insert("share/bison".into(), ArtifactKind::Tree);
                plan.phase_members.insert("bison".into(), ("out".into(), "bin/bison".into()));
            }
            "gawk-3.0.4-gcc-v4" => {
                for path in ["bin/gawk", "bin/awk"] {
                    installed.insert(path.into(), ArtifactKind::Executable);
                }
            }
            "grep-2.4-gcc-v10" => {
                installed.insert("bin/grep".into(), ArtifactKind::Executable);
            }
            "heirloom-devtools-070527-gcc-v6" => {
                for path in ["bin/yacc", "bin/lex"] {
                    installed.insert(path.into(), ArtifactKind::Executable);
                }
                for path in ["lib/yaccpar", "lib/lex/ncform", "lib/lex/nceucform", "lib/lex/nrform"] {
                    installed.insert(path.into(), ArtifactKind::Text);
                }
            }
            "tcc-0.9.27-musl-native-runtime" => {
                installed.insert("bin/tcc".into(), ArtifactKind::Executable);
                installed.insert("lib/tcc".into(), ArtifactKind::Tree);
                plan.phase_members.insert("tcc".into(), ("out".into(), "bin/tcc".into()));
            }
            "binutils-2.30-gas-source-v41" => {
                installed.insert("bin/as".into(), ArtifactKind::Executable);
            }
            "binutils-2.30-tcc-source-v1" => {
                let (original, root) = source(request, "-binutils-2.30-src", "binutils-2.30", "ld/ldmain.c")?;
                let editor = crate::source_built_native_programs::final_gnu::generators::append_source_projector(
                    request,
                    &mut plan.preparation,
                )?;
                const PUBLISHER: &str = "source-built/native/generated.c";
                plan.preparation
                    .files
                    .insert(PUBLISHER.into(), include_str!("source_built_native_programs_generated.c").into());
                let ready = exec(
                    &mut plan.preparation,
                    format!("{}.early-ld-writer-source", request.id),
                    editor,
                    Files::from([
                        ("original".into(), original.artifact.clone()),
                        ("publisher".into(), manifest(PUBLISHER)),
                    ]),
                    vec![
                        super::literal("ld-source"),
                        input("original", ""),
                        super::output("result"),
                        input("publisher", ""),
                        super::literal(&root),
                    ],
                    ArtifactKind::Tree,
                );
                plan.source_overrides.insert(original.original_output.clone(), ready);
                for tool in [
                    "as", "ld", "ar", "ranlib", "nm", "objcopy", "objdump", "readelf", "size", "strings", "strip",
                ] {
                    installed.insert(format!("bin/{tool}"), ArtifactKind::Executable);
                }
                plan.phase_members.insert("ld".into(), ("out".into(), "bin/ld".into()));
            }
            _ => unreachable!("known original early GNU package"),
        }
    }
    plan.members.insert("out".into(), installed);
    Ok(Some(plan))
}

pub(crate) fn lower(request: &NativeNodeRequest<'_, '_>) -> Result<Option<NativeNodeProgram>, RunError> {
    let Some(plan) = prepare(request)? else {
        return Ok(None);
    };
    let mut additional = plan.literal_overrides;
    let original = include_str!("../bootstrap/stagex-transition-proof-input.ncl")
        .lines()
        .find(|line| line.starts_with('"'))
        .and_then(|line| line.split('"').nth(1))
        .ok_or_else(|| error("original transition proof input lacks its literal StorePath"))?;
    let basename = original.rsplit('/').next().ok_or_else(|| error("invalid original transition StorePath"))?;
    let transition = format!("/{basename}");
    for path in &request.node.derivation.input_sources {
        let path = path.to_absolute_path_with_prefix(&request.request.proof.logical_store_prefix);
        if path.ends_with(&transition) {
            additional.insert(path, predecessor(request, "stagex-transition:stagex-transition")?);
        }
    }
    let callbacks = crate::source_built_native_programs::final_gnu::controller_source(request)?;
    let program = crate::source_built_native_programs::host::original_node_prepared(
        request,
        plan.source_overrides,
        additional,
        plan.members,
        plan.phase_members,
        &[crate::source_built_native_programs::final_gnu::controller_hooks(
            &callbacks,
        )],
        plan.preparation.steps,
        plan.preparation.timeout_groups,
        plan.preparation.files,
    )?;
    Ok(Some(program))
}
