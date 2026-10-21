//! Source-only optional child-boundary adaptations. The enclosing original
//! configure/Make node must really compile and install each adapted program.
use std::collections::BTreeMap;

use super::TREE_BYTES_MAX;
use super::definition;
use super::error;
use super::input;
use super::logs;
use super::output;
use super::produced;
use super::require_produced;
use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::RECORD_IDENTITY;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;

pub(crate) const BISON_PHASE_MEMBER: &str = "source-built/native/final-gnu-bison-phase.inc";
pub(crate) const BISON_PHASE_SOURCE: &str = include_str!("source_built_native_programs_final_gnu_bison_phase.inc");

pub(crate) struct GeneratorRuntime {
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub environment: BTreeMap<String, RecipeWord>,
    pub runtime_mounts: BTreeMap<String, String>,
}

/// This helper only translates original source; it is never an original node's
/// installed executable, output tree, or completed generator capability.
pub(crate) fn append_source_projector(
    request: &crate::source_built_native_programs::NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
) -> Result<ArtifactRef, RunError> {
    const MEMBER: &str = "source-built/native/final-gnu-source-patch.c";
    const SOURCE: &str = include_str!("source_built_native_programs_final_gnu_source_patch.c");
    let mut recipe = crate::source_built_recipe::SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: crate::source_built_native_programs::STAGE.into(),
        preparation: crate::source_built_recipe::RecipePreparation {
            tool: "original GNU source adaptation".into(),
            provenance: "authenticated original release sources and unchanged default tool algorithms".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    let id = format!("{}.final-source-projector", request.id);
    if program.steps.iter().any(|step| step.id == id) {
        return Err(error("original source projector identity collides"));
    }
    let executable =
        crate::source_built_native_programs::append_stagex_c_helper(request.request, &mut recipe, &id, MEMBER)?;
    program.files.insert(MEMBER.into(), SOURCE.into());
    program.steps.extend(recipe.steps);
    program.timeout_groups.extend(recipe.timeout_groups);
    Ok(executable)
}

/// The original Flex filter graph selects the actual m4 child. Original tee,
/// skeleton processing, header output, and line-directive behavior remain intact.
pub(crate) fn append_flex_source_patch(
    program: &mut NativeNodeProgram,
    id: String,
    source_editor: ArtifactRef,
    source: ArtifactRef,
    source_path: &str,
    runtime: &GeneratorRuntime,
) -> Result<ArtifactRef, RunError> {
    use crate::source_built_stagex_programs::generators as shared;
    require_produced(&source_editor)?;
    require_produced(&source)?;
    crate::source_built_recipe::relative(&id, false)?;
    crate::source_built_recipe::relative(source_path, true)?;
    let mut inputs = runtime.inputs.clone();
    if inputs.insert("generator-source".into(), source).is_some() {
        return Err(error("reserved generator source input"));
    }
    for (name, member, text) in [
        (
            "generator-capture",
            "source-built/native/final-gnu-flex-capture.inc",
            shared::flex_capture_branch_source(),
        ),
        (
            "generator-tee",
            "source-built/native/final-gnu-flex-tee.inc",
            shared::flex_tee_streams_source().into(),
        ),
        ("generator-main", "source-built/native/final-gnu-flex-main.inc", shared::flex_native_main_source()),
        (
            "generator-child",
            "source-built/native/final-gnu-flex-child.inc",
            include_str!("source_built_native_programs_final_gnu_flex_child.inc").into(),
        ),
    ] {
        if program.files.get(member).is_some_and(|prior| prior != &text) {
            return Err(error("native Flex phase source collides"));
        }
        program.files.insert(member.into(), text);
        let reference = ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::NativeSourceBundle,
            record_identity: RECORD_IDENTITY.into(),
            member: member.into(),
        };
        if inputs.insert(name.into(), reference).is_some() {
            return Err(error("reserved generator phase input"));
        }
    }
    let seams_id = format!("{id}.seams");
    let mut seams = String::new();
    for text in [
        shared::FLEX_TEE_PIPE_START,
        shared::FLEX_TEE_PIPE_END,
        shared::FLEX_TEE_WAIT,
    ] {
        seams.push_str(&text.len().to_string());
        seams.push(':');
        seams.push_str(text);
        seams.push(',');
    }
    if program.steps.iter().any(|prior| prior.id == id || prior.id == seams_id) {
        return Err(error("Flex source patch identity collides"));
    }
    program.steps.push(RecipeStep {
        id: seams_id.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("text".into(), definition(ArtifactKind::Text, super::TEXT_BYTES_MAX))]),
        operation: RecipeOperation::WriteText {
            output: "text".into(),
            text: seams,
        },
    });
    if inputs.insert("generator-seams".into(), produced(&seams_id, "text")).is_some()
        || inputs.insert("generator-tool".into(), source_editor).is_some()
    {
        return Err(error("reserved generator source adapter inputs"));
    }
    let mut outputs = logs();
    outputs.insert("state".into(), definition(ArtifactKind::Tree, TREE_BYTES_MAX));
    program.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs,
        operation: RecipeOperation::Exec {
            executable_input: "generator-tool".into(),
            arguments: vec![
                input("generator-source", source_path),
                output("state", ""),
                input("generator-capture", ""),
                input("generator-tee", ""),
                input("generator-main", ""),
                input("generator-seams", ""),
                input("generator-child", ""),
            ],
            environment: runtime.environment.clone(),
            runtime_mounts: runtime.runtime_mounts.clone(),
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            guard: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            merge_stderr: false,
            invocation_input: None,
        },
    });
    Ok(produced(&id, "state"))
}
