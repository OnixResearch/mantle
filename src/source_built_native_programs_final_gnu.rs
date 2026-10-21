//! Complete original final GNU source nodes. The shared shell/Make controller
//! interprets the original recipes, and genuine Perl/Bison/Flex programs expose
//! their actual child boundaries as independently Produced computations.
use std::collections::BTreeMap;

use crate::errors::RunError;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::STAGE;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::WordPart;

#[path = "source_built_native_programs_final_gnu_generators.rs"]
pub(crate) mod generators;
#[path = "source_built_native_programs_final_gnu_nodes.rs"]
mod nodes;

pub(crate) use nodes::is_regenerated_source_node;

const TEXT_BYTES_MAX: u64 = 16 * 1024 * 1024;
const TREE_BYTES_MAX: u64 = 4 * 1024 * 1024 * 1024;

pub(crate) fn perl_transport_source() -> String {
    include_str!("source_built_native_programs_final_gnu_perl_transport.c").into()
}
pub(crate) fn perl_controller_source() -> String {
    concat!(
        include_str!("source_built_native_programs_final_gnu_perl_transport.c"),
        "\n",
        include_str!("source_built_native_programs_final_gnu_perl_controller.c")
    )
    .into()
}
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub(crate) fn perl_controller_hooks(source: &str) -> crate::source_built_native_programs::host::ControllerHooks<'_> {
    crate::source_built_native_programs::host::ControllerHooks {
        source,
        external: "mantle_perl_controller_external",
        prepare: "mantle_perl_controller_prepare",
        environment: "mantle_perl_controller_environment",
    }
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("original native GNU source expansion: {}", message.into()))
}
fn produced(id: &str, output: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{STAGE}/{id}"),
        output_id: output.into(),
    }
}
fn require_produced(reference: &ArtifactRef) -> Result<(), RunError> {
    if matches!(reference, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("original native programs and source states must be Produced"))
    }
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
fn output(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: path.into(),
        }],
    }
}
fn definition(kind: ArtifactKind, bytes_max: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max }
}
fn logs() -> BTreeMap<String, RecipeOutput> {
    BTreeMap::from([
        ("stdout".into(), definition(ArtifactKind::Text, TEXT_BYTES_MAX)),
        ("stderr".into(), definition(ArtifactKind::Text, TEXT_BYTES_MAX)),
        ("exit-status".into(), definition(ArtifactKind::Text, 4)),
    ])
}
fn empty_program() -> NativeNodeProgram {
    NativeNodeProgram {
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        outputs: BTreeMap::new(),
        members: BTreeMap::new(),
        files: BTreeMap::new(),
        phase_tools: BTreeMap::new(),
    }
}

/// The catalog binds source-phase behavior to the exact installed compiled
/// members. Executable authority remains with the common protected leaf matcher.
pub(crate) fn controller_source(request: &NativeNodeRequest<'_, '_>) -> Result<String, RunError> {
    let mut images = BTreeMap::<String, i32>::new();
    for (drv, program) in request.dependencies {
        for (capability, kind) in [("bison", 1), ("flex", 2), ("m4", 3)] {
            if let Some(image) = program.phase_tools.get(capability) {
                require_produced(image)?;
                let mut found = false;
                for (out, members) in &program.members {
                    let prefix = request
                        .logical_outputs
                        .get(&format!("{drv}:{out}"))
                        .ok_or_else(|| error("missing exact original generator output path"))?;
                    for (path, member) in members {
                        if member == image {
                            images.insert(format!("{prefix}/{path}"), kind);
                            found = true;
                        }
                    }
                }
                if !found {
                    return Err(error(format!(
                        "original {capability} phase is not an authentic installed compiled member"
                    )));
                }
            }
        }
    }
    let name = request
        .node
        .derivation
        .environment
        .get("name")
        .and_then(|value| std::str::from_utf8(value.as_ref()).ok())
        .ok_or_else(|| error("original GNU node lacks its evaluated name"))?;
    let current = if name.starts_with("bison-2.3-") || name.starts_with("bison-3.4.1-") {
        Some(("bin/bison", 1))
    } else if name.starts_with("flex-2.5.36-") || name.starts_with("flex-2.6.4-") {
        Some(("bin/flex", 2))
    } else if name == "m4-1.4.7-gcc-v5" {
        Some(("bin/m4", 3))
    } else {
        None
    };
    if let Some((path, kind)) = current {
        let drv = request.node.drv_path.to_absolute_path_with_prefix(&request.request.proof.logical_store_prefix);
        let prefix = request
            .logical_outputs
            .get(&format!("{drv}:out"))
            .ok_or_else(|| error("original generator output path is absent"))?;
        images.insert(format!("{prefix}/{path}"), kind);
    }
    let mut source = String::from("#define FINAL_GNU_CAPABILITIES");
    for (image, kind) in images {
        let quoted = serde_json::to_string(&image).map_err(|cause| error(cause.to_string()))?;
        source.push_str(&format!(" \\\n    {{{quoted},{kind}}},"));
    }
    source.push('\n');
    source.push_str(include_str!("source_built_native_programs_final_gnu_gcc_generators.c"));
    source.push_str(include_str!("source_built_native_programs_final_gnu_controller.c"));
    Ok(source)
}
pub(crate) fn controller_hooks(source: &str) -> crate::source_built_native_programs::host::ControllerHooks<'_> {
    crate::source_built_native_programs::host::ControllerHooks {
        source,
        external: "final_gnu_external",
        prepare: "final_gnu_prepare",
        environment: "final_gnu_environment",
    }
}
pub(crate) fn lower(request: &NativeNodeRequest<'_, '_>) -> Result<Option<NativeNodeProgram>, RunError> {
    nodes::lower(request)
}
