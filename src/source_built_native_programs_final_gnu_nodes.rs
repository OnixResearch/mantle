//! Complete original final-GNU nodes, executed through the shared source shell
//! and Make continuation. Source adapters below only expose genuine child
//! boundaries; their outputs never stand in for an installed original node.
use std::collections::BTreeMap;
use std::path::Path;

use super::TREE_BYTES_MAX;
use super::definition;
use super::empty_program;
use super::error;
use super::generators::BISON_PHASE_MEMBER;
use super::generators::BISON_PHASE_SOURCE;
use super::generators::GeneratorRuntime;
use super::generators::append_flex_source_patch;
use super::generators::append_source_projector;
use super::input;
use super::literal;
use super::logs;
use super::output;
use super::produced;
use super::require_produced;
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

#[derive(Clone, Copy)]
enum Node {
    Autoconf,
    Automake,
    Bison,
    Flex(bool),
    Gmp,
    Mpfr,
    Mpc,
    Binutils(bool),
    Gcc,
    Musl,
    Regeneration(bool),
}

fn selected(name: &str) -> Option<Node> {
    Some(match name {
        "autoconf-2.13-gcc-v1"
        | "autoconf-2.52-gcc-v6"
        | "autoconf-2.53-gcc-v7"
        | "autoconf-2.54-gcc-v1"
        | "autoconf-2.55-gcc-v2"
        | "autoconf-2.57-gcc-v1"
        | "autoconf-2.59-gcc-v2"
        | "autoconf-2.61-gcc-v1"
        | "autoconf-2.64-gcc-v1"
        | "autoconf-2.69-gcc-v1" => Node::Autoconf,
        "automake-1.6.3-gcc-v4"
        | "automake-1.7-gcc-v4"
        | "automake-1.7.8-gcc-v1"
        | "automake-1.8.5-gcc-v1"
        | "automake-1.9.6-gcc-v1"
        | "automake-1.10.3-gcc-v1"
        | "automake-1.11.2-gcc-v1"
        | "automake-1.15.1-gcc-v1" => Node::Automake,
        "bison-3.4.1-stage1-gcc-v1" | "bison-3.4.1-gcc-v2" => Node::Bison,
        "flex-2.5.11-gcc-v6" => Node::Flex(false),
        "flex-2.5.36-gcc-v4" | "flex-2.6.4-gcc-v1" => Node::Flex(true),
        "gmp-6.2.1-gcc40-v1" => Node::Gmp,
        "mpfr-4.1.0-gcc40-v1" => Node::Mpfr,
        "mpc-1.2.1-gcc40-v1" => Node::Mpc,
        "binutils-2.30-gcc-pass4-v17" => Node::Binutils(false),
        "binutils-2.41-gcc10-v1" => Node::Binutils(true),
        "gcc-10.5.0-musl-gcc47-v2" | "gcc-10.5.0-musl-final-v1" => Node::Gcc,
        "musl-1.2.5-gcc10-v1" => Node::Musl,
        "binutils-2.41-regeneration-ready-source-v1" => Node::Regeneration(false),
        "gcc-10.5.0-regenerated-source-v1" => Node::Regeneration(true),
        _ => return None,
    })
}

fn evaluated_name(node: &crunch_pipeline::native_preflight::NativePlannedDerivation) -> Result<&str, RunError> {
    node.derivation
        .environment
        .get("name")
        .and_then(|value| std::str::from_utf8(value.as_ref()).ok())
        .ok_or_else(|| error("original GNU derivation lacks its evaluated name"))
}

/// Only completed source-data nodes propagate original source-file lineage.
pub(crate) fn is_regenerated_source_node(
    node: &crunch_pipeline::native_preflight::NativePlannedDerivation,
) -> Result<bool, RunError> {
    Ok(matches!(selected(evaluated_name(node)?), Some(Node::Regeneration(_))))
}

fn archive<'a>(request: &'a NativeNodeRequest<'_, '_>, name: &str) -> Result<&'a NativeSource, RunError> {
    let suffix = format!("-{name}");
    let mut sources = request.sources.values().filter(|source| source.original_output.ends_with(&suffix));
    let source = sources.next().ok_or_else(|| error(format!("missing authenticated original {name}")))?;
    if sources.next().is_some() {
        return Err(error(format!("ambiguous authenticated original {name}")));
    }
    require_produced(&source.artifact)?;
    Ok(source)
}

fn source_root(source: &NativeSource, archive: &str, member: &str) -> Result<String, RunError> {
    let root = archive.strip_suffix("-src").ok_or_else(|| error("original source name lacks its release suffix"))?;
    for prefix in ["", root] {
        let path = Path::new(prefix).join(member);
        if source.preparation_path.join(&path).is_file() {
            return Ok(prefix.into());
        }
    }
    Err(error(format!("original {archive} requires source member {member}")))
}

fn dependency_source(request: &NativeNodeRequest<'_, '_>, name: &str) -> Result<(String, ArtifactRef), RunError> {
    let suffix = format!("-{name}.drv");
    let mut dependencies = request.dependencies.iter().filter(|(path, _)| path.ends_with(&suffix));
    let (drv, program) =
        dependencies.next().ok_or_else(|| error(format!("missing completed original source node {name}")))?;
    if dependencies.next().is_some() {
        return Err(error("ambiguous original generated-source node"));
    }
    let logical = request
        .logical_outputs
        .get(&format!("{drv}:out"))
        .ok_or_else(|| error("missing original generated-source logical output"))?
        .clone();
    let reference = program
        .outputs
        .get("out")
        .ok_or_else(|| error("original generated-source node has no out tree"))?
        .clone();
    require_produced(&reference)?;
    Ok((logical, reference))
}

fn source_exec(
    program: &mut NativeNodeProgram,
    id: String,
    executable: ArtifactRef,
    mut inputs: BTreeMap<String, ArtifactRef>,
    arguments: Vec<RecipeWord>,
) -> ArtifactRef {
    inputs.insert("source-tool".into(), executable);
    let mut outputs = logs();
    outputs.insert("state".into(), definition(ArtifactKind::Tree, TREE_BYTES_MAX));
    program.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs,
        operation: RecipeOperation::Exec {
            executable_input: "source-tool".into(),
            arguments,
            environment: BTreeMap::new(),
            runtime_mounts: BTreeMap::new(),
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            guard: None,
            stdin_input: None,
            command_state: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            merge_stderr: false,
            invocation_input: None,
        },
    });
    produced(&id, "state")
}

fn adapt_source(
    request: &NativeNodeRequest<'_, '_>,
    node: Node,
    name: &str,
    program: &mut NativeNodeProgram,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    let mut overrides = BTreeMap::new();
    match node {
        Node::Bison | Node::Flex(true) | Node::Binutils(_) => {
            let source_editor = append_source_projector(request, program)?;
            match node {
                Node::Bison => {
                    let source = archive(request, "bison-3.4.1-src")?;
                    let root = source_root(source, "bison-3.4.1-src", "src/output.c")?;
                    program.files.insert(BISON_PHASE_MEMBER.into(), BISON_PHASE_SOURCE.into());
                    let phase = ArtifactRef::ManifestSource {
                        role: SourceAuthorityRole::NativeSourceBundle,
                        record_identity: RECORD_IDENTITY.into(),
                        member: BISON_PHASE_MEMBER.into(),
                    };
                    let tree = source_exec(
                        program,
                        format!("{}.bison-source-phase", request.id),
                        source_editor,
                        BTreeMap::from([("original".into(), source.artifact.clone()), ("phase".into(), phase)]),
                        vec![
                            literal("bison-source"),
                            input("original", ""),
                            output("state", ""),
                            input("phase", ""),
                            literal(&root),
                        ],
                    );
                    overrides.insert(source.original_output.clone(), tree);
                }
                Node::Flex(true) => {
                    let archive_name = if name.starts_with("flex-2.5.36-") {
                        "flex-2.5.36-src"
                    } else {
                        "flex-2.6.4-src"
                    };
                    let source = archive(request, archive_name)?;
                    let member = if name.starts_with("flex-2.5.36-") {
                        "filter.c"
                    } else {
                        "src/filter.c"
                    };
                    let root = source_root(source, archive_name, member)?;
                    let runtime = GeneratorRuntime {
                        inputs: BTreeMap::new(),
                        environment: BTreeMap::new(),
                        runtime_mounts: BTreeMap::new(),
                    };
                    let tree = append_flex_source_patch(
                        program,
                        format!("{}.flex-source-phase", request.id),
                        source_editor,
                        source.artifact.clone(),
                        &root,
                        &runtime,
                    )?;
                    if !root.is_empty() {
                        return Err(error("original Flex fetch must expose its authenticated release source root"));
                    }
                    overrides.insert(source.original_output.clone(), tree);
                }
                Node::Binutils(final_version) => {
                    let (logical, source, root) = if final_version {
                        let (logical, source) =
                            dependency_source(request, "binutils-2.41-regeneration-ready-source-v1")?;
                        (logical, source, String::new())
                    } else {
                        let source = archive(request, "binutils-2.30-src")?;
                        (
                            source.original_output.clone(),
                            source.artifact.clone(),
                            source_root(source, "binutils-2.30-src", "ld/ldmain.c")?,
                        )
                    };
                    let publisher = ArtifactRef::ManifestSource {
                        role: SourceAuthorityRole::NativeSourceBundle,
                        record_identity: RECORD_IDENTITY.into(),
                        member: "source-built/native/generated.c".into(),
                    };
                    let tree = source_exec(
                        program,
                        format!("{}.ld-writer-source", request.id),
                        source_editor,
                        BTreeMap::from([("original".into(), source), ("publisher".into(), publisher)]),
                        vec![
                            literal("ld-source"),
                            input("original", ""),
                            output("state", ""),
                            input("publisher", ""),
                            literal(&root),
                        ],
                    );
                    overrides.insert(logical, tree);
                }
                _ => unreachable!(),
            }
        }
        Node::Gcc => {
            let (logical, source) = dependency_source(request, "gcc-10.5.0-regenerated-source-v1")?;
            let collect2 = crate::source_built_native_programs::append_gcc10_collect2_source(
                request,
                program,
                "final-native",
                source.clone(),
                "gcc/collect2.c",
            )?;
            let editor = append_source_projector(request, program)?;
            let tree = source_exec(
                program,
                format!("{}.gcc-collect2-source-tree", request.id),
                editor,
                BTreeMap::from([("original".into(), source), ("member".into(), collect2)]),
                vec![
                    literal("gcc-source"),
                    input("original", ""),
                    output("state", ""),
                    input("member", ""),
                ],
            );
            overrides.insert(logical, tree);
        }
        _ => {}
    }
    Ok(overrides)
}

fn installed(node: Node, name: &str) -> BTreeMap<String, ArtifactKind> {
    let mut members = BTreeMap::new();
    let mut add = |path: &str, kind| {
        members.insert(path.into(), kind);
    };
    match node {
        Node::Autoconf => {
            for tool in [
                "autoconf",
                "autoheader",
                "autoreconf",
                "autoscan",
                "autoupdate",
                "ifnames",
            ] {
                add(&format!("bin/{tool}"), ArtifactKind::Text);
            }
            if !name.starts_with("autoconf-2.13-") {
                add("bin/autom4te", ArtifactKind::Text);
            }
            add("share/autoconf", ArtifactKind::Tree);
        }
        Node::Automake => {
            for tool in ["automake", "aclocal"] {
                add(&format!("bin/{tool}"), ArtifactKind::Text);
            }
        }
        Node::Bison => {
            add("bin/bison", ArtifactKind::Executable);
            add("share/bison", ArtifactKind::Tree);
        }
        Node::Flex(_) => {
            add("bin/flex", ArtifactKind::Executable);
            if !name.starts_with("flex-2.5.11-") {
                add("lib/libfl.a", ArtifactKind::Text);
                add("include/FlexLexer.h", ArtifactKind::Text);
            }
        }
        Node::Gmp => {
            for path in ["lib/libgmp.a", "include/gmp.h"] {
                add(path, ArtifactKind::Text);
            }
        }
        Node::Mpfr => {
            for path in ["lib/libmpfr.a", "include/mpfr.h", "include/mpf2mpfr.h"] {
                add(path, ArtifactKind::Text);
            }
        }
        Node::Mpc => {
            for path in ["lib/libmpc.a", "include/mpc.h"] {
                add(path, ArtifactKind::Text);
            }
        }
        Node::Binutils(_) => {
            for tool in [
                "as", "ld", "ar", "ranlib", "nm", "objcopy", "objdump", "readelf", "size", "strings", "strip",
            ] {
                add(&format!("bin/{tool}"), ArtifactKind::Executable);
            }
        }
        Node::Gcc => {
            for tool in ["gcc", "g++", "cpp"] {
                add(&format!("bin/{tool}"), ArtifactKind::Executable);
            }
            for tool in ["cc1", "cc1plus", "collect2"] {
                add(&format!("libexec/gcc/x86_64-unknown-linux-musl/10.5.0/{tool}"), ArtifactKind::Executable);
            }
            for path in [
                "lib/gcc/x86_64-unknown-linux-musl/10.5.0/libgcc.a",
                "lib/gcc/x86_64-unknown-linux-musl/10.5.0/libgcc_eh.a",
            ] {
                add(path, ArtifactKind::Text);
            }
        }
        Node::Musl => {
            for path in [
                "lib/libc.a",
                "lib/libc.so",
                "lib/crt1.o",
                "lib/Scrt1.o",
                "lib/crti.o",
                "lib/crtn.o",
            ] {
                add(path, ArtifactKind::Text);
            }
        }
        Node::Regeneration(gcc) => {
            let directories: &[&str] = if gcc {
                &[
                    "",
                    "fixincludes",
                    "intl",
                    "gcc",
                    "libcpp",
                    "libdecnumber",
                    "libgcc",
                    "libiberty",
                    "libstdc++-v3",
                    "lto-plugin",
                    "zlib",
                ]
            } else {
                &[
                    "",
                    "bfd",
                    "binutils",
                    "gas",
                    "gprof",
                    "intl",
                    "ld",
                    "libctf",
                    "libiberty",
                    "libsframe",
                    "opcodes",
                ]
            };
            for directory in directories {
                add(
                    &if directory.is_empty() {
                        "configure".into()
                    } else {
                        format!("{directory}/configure")
                    },
                    ArtifactKind::Text,
                );
            }
            add(".mantle-regeneration/provenance.txt", ArtifactKind::Text);
            if gcc {
                for path in [
                    "intl/plural.c",
                    "gcc/config.in",
                    "gcc/gengtype-lex.c",
                    ".mantle-regeneration/gcc-10.5.0-files.txt",
                ] {
                    add(path, ArtifactKind::Text);
                }
            } else {
                for path in [
                    ".mantle-regeneration/binutils-2.41-configure-files.txt",
                    ".mantle-regeneration/binutils-2.41-removed-generated-files.txt",
                ] {
                    add(path, ArtifactKind::Text);
                }
            }
        }
    }
    members
}

pub(crate) fn lower(request: &NativeNodeRequest<'_, '_>) -> Result<Option<NativeNodeProgram>, RunError> {
    let name = evaluated_name(request.node)?;
    let Some(node) = selected(name) else {
        return Ok(None);
    };
    let mut preparation = empty_program();
    let overrides = adapt_source(request, node, name, &mut preparation)?;
    let mut phases = BTreeMap::new();
    match node {
        Node::Bison => {
            phases.insert("bison".into(), ("out".into(), "bin/bison".into()));
        }
        Node::Flex(true) => {
            phases.insert("flex".into(), ("out".into(), "bin/flex".into()));
        }
        _ => {}
    }
    let source = super::controller_source(request)?;
    let program = crate::source_built_native_programs::host::original_node_prepared(
        request,
        overrides,
        BTreeMap::new(),
        BTreeMap::from([("out".into(), installed(node, name))]),
        phases,
        &[super::controller_hooks(&source)],
        preparation.steps,
        preparation.timeout_groups,
        preparation.files,
    )?;
    Ok(Some(program))
}
