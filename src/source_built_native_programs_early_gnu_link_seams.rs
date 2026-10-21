//! Actual query-to-query expansion of the original GCC4 assembler/linker seams.
//! It never stands in for a linker invocation or the owning original build.
use std::collections::BTreeMap;

use super::FILE_MAX;
use super::bounded;
use super::c_helper;
use super::error;
use super::input;
use super::literal;
use super::output;
use super::produced;
use crate::errors::RunError;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeGuard;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;

// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
pub(crate) enum Gcc4LinkSeam {
    /// gcc-4.0-native: indexed libc, dropped CRT/libgcc artifacts, real TCC
    /// runtime supplement and the original compiled compatibility object.
    Native,
    /// gcc-4.0-musl-pass3's seed compiler, before its own libgcc exists.
    SeedPass3,
    /// Installed pass3/pass5/pass6/C++ and the pass4/5/6/C++ build compiler.
    Standard,
    /// Installed pass4 resolves other -l arguments against earlier -L options.
    InstalledPass4,
    /// Generator bundle additionally rebases absolute libgcc and CRT paths.
    GeneratorBase,
    /// The initial GCC4 assembler wrapper drops exactly the original -Qy.
    NativeAssembler,
}
impl Gcc4LinkSeam {
    fn mode(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::SeedPass3 => "seed-pass3",
            Self::Standard => "standard",
            Self::InstalledPass4 => "installed-pass4",
            Self::GeneratorBase => "generator-base",
            Self::NativeAssembler => "native-as",
        }
    }
}

pub(crate) struct Gcc4LinkSeamInvocation {
    pub id: String,
    /// Actual single phase query, including image and final newline, emitted
    /// by the original collect2 source (or selected original assembler phase).
    pub query: ArtifactRef,
    pub seam: Gcc4LinkSeam,
    pub inputs: BTreeMap<String, ArtifactRef>,
    /// Original selected image, checked before any argument transformation.
    pub expected_image: RecipeWord,
    /// Exact binutils image selected by the original shell seam's exec. This
    /// is distinct from GCC's selected assembler/linker wrapper image.
    pub linker_image: RecipeWord,
    pub crt_directory: RecipeWord,
    pub libc: RecipeWord,
    pub libm: RecipeWord,
    pub libgcc: RecipeWord,
    /// Used only by Native; all other seams ignore this operand.
    pub runtime_supplement: RecipeWord,
    /// Used by Native and SeedPass3, not a fixture compatibility object.
    pub compatibility_object: RecipeWord,
    pub runtime_mounts: BTreeMap<String, String>,
    /// Retain the specs/cc1/as/collect2 success predicates for a real probe.
    pub guard: Option<RecipeGuard>,
    /// Configure tests accept rejection; required links require exact success.
    pub exit_policy: RecipeExitPolicy,
}

pub(crate) struct Gcc4LinkSeamExpansion {
    /// Guarded: consumers must retain the invocation guard, then require the
    /// actual adaptation status zero before reading this query or invoking ld.
    pub query: ArtifactRef,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub status: ArtifactRef,
    /// Always available, including when the preceding compiler probe skipped.
    pub outcome: ArtifactRef,
}

pub(crate) fn append_gcc4_link_seam(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    invocation: Gcc4LinkSeamInvocation,
) -> Result<Gcc4LinkSeamExpansion, RunError> {
    const KEY: &str = "gcc4-link-seam";
    const MEMBER: &str = "source-built/native/early-gnu-link-seams.c";
    let helper = if let Some(reference) = program.phase_tools.get(KEY) {
        reference.clone()
    } else {
        let reference = c_helper(
            request,
            program,
            MEMBER,
            include_str!("source_built_native_programs_early_gnu_link_seams.c"),
            "gcc4-link-seam-tool",
        )?;
        program.phase_tools.insert(KEY.into(), reference.clone());
        reference
    };
    if !matches!(invocation.query, ArtifactRef::Produced { .. }) {
        return Err(error("GCC4 seam query must come from its actual original source phase"));
    }
    let mut inputs = invocation.inputs;
    if inputs.insert("query".into(), invocation.query).is_some() || inputs.insert("seam-tool".into(), helper).is_some()
    {
        return Err(error("reserved GCC4 source seam input"));
    }
    let id = invocation.id;
    let outputs = BTreeMap::from([
        ("result".into(), bounded(ArtifactKind::Text, FILE_MAX)),
        ("stdout".into(), bounded(ArtifactKind::Text, FILE_MAX)),
        ("stderr".into(), bounded(ArtifactKind::Text, FILE_MAX)),
        ("exit-status".into(), bounded(ArtifactKind::Text, 4)),
    ]);
    // A missing original -l archive is an actual failed linker request, not an
    // authoring-time answer. Preserve the generated query file even on reject.
    program.steps.push(RecipeStep {
        id: id.clone(),
        inputs,
        outputs,
        operation: RecipeOperation::Exec {
            executable_input: "seam-tool".into(),
            arguments: vec![
                literal(invocation.seam.mode()),
                input("query", ""),
                invocation.expected_image,
                invocation.crt_directory,
                invocation.libc,
                invocation.libm,
                invocation.libgcc,
                invocation.runtime_supplement,
                invocation.compatibility_object,
                invocation.linker_image,
                output("result"),
            ],
            environment: BTreeMap::new(),
            invocation_input: None,
            runtime_mounts: invocation.runtime_mounts,
            cwd_input: None,
            cwd_output: None,
            stdin_input: None,
            merge_stderr: false,
            exit_policy: invocation.exit_policy,
            guard: invocation.guard,
            output_seeds: BTreeMap::new(),
            command_state: None,
        },
    });
    let observation = format!("{id}.observation");
    program.steps.push(RecipeStep {
        id: observation.clone(),
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([(
            "outcome".into(),
            bounded(ArtifactKind::Tree, request.request.proof.resource_bounds.disk_bytes_max),
        )]),
        operation: RecipeOperation::ObservePhase {
            phase: id.clone(),
            output: "outcome".into(),
            state: None,
        },
    });
    Ok(Gcc4LinkSeamExpansion {
        query: produced(&id, "result"),
        status: produced(&id, "exit-status"),
        outcome: produced(&observation, "outcome"),
    })
}
