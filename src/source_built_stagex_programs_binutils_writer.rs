//! Rebuild the original linker with its successful-close product publisher.
//! The canonical installed image is already immutable before this phase starts.
use std::collections::BTreeMap;

use super::Author;
use super::FILE_BYTES_MAX;
use super::Files;
use super::NativeHelpers;
use super::TREE_BYTES_MAX;
use super::error;
use crate::errors::RunError;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_stagex_programs::controller::ControllerSlot;
use crate::source_built_stagex_programs::controller::ControllerSpec;
use crate::source_built_stagex_programs::input;
use crate::source_built_stagex_programs::literal;
use crate::source_built_stagex_programs::output;
use crate::stagex_transition::BINUTILS_INSTALL_STAGE_ID as INSTALL;

const RUNTIME: &str = "/mantle/stagex/transition/binutils-stage/runtime";
const SOURCE_MEMBER: &str = "stagex/transition/binutils-stage/runtime/source";
const LD_MEMBER: &str = "stagex/transition/binutils-stage/runtime/source/ld/ld-new";
const TOOLS: &str = "/mantle/stagex/transition";
const PREFIX: &str = "/mantle/stagex/binutils-probe-output";

fn authored(author: &mut Author<'_>, name: &str, text: String) -> Result<ArtifactRef, RunError> {
    author.builder.authored(&format!("helpers/binutils-ld-writer/{name}"), text)
}

// A controller's workspace and times are outputs of the same observed finish.
// Check the exact producer contract rather than constructing an unbound input.
fn workspace_times(author: &Author<'_>, workspace: &ArtifactRef) -> Result<ArtifactRef, RunError> {
    let ArtifactRef::Produced { action_id, output_id } = workspace else {
        return Err(error("linker writer requires the completed original controller workspace"));
    };
    if output_id != "workspace" {
        return Err(error("linker writer input is not the original workspace output"));
    }
    let producer_id = action_id
        .strip_prefix(author.builder.recipe.stage_id.as_str())
        .and_then(|suffix| suffix.strip_prefix('/'))
        .ok_or_else(|| error("linker writer workspace leaves its original recipe"))?;
    let producer = author
        .builder
        .recipe
        .steps
        .iter()
        .find(|step| step.id == producer_id)
        .ok_or_else(|| error("linker writer workspace has no registered original producer"))?;
    if producer.outputs.get("times").is_none_or(|value| value.kind != ArtifactKind::Text) {
        return Err(error("linker writer workspace producer lacks observed original times"));
    }
    Ok(ArtifactRef::Produced {
        action_id: action_id.clone(),
        output_id: "times".into(),
    })
}

pub(super) fn ld_writer_variant(
    author: &mut Author<'_>,
    native: &NativeHelpers,
    workspace: ArtifactRef,
    controller: ArtifactRef,
    slots: Vec<ControllerSlot>,
    roots: ArtifactRef,
    environment: ArtifactRef,
) -> Result<ArtifactRef, RunError> {
    let times = workspace_times(author, &workspace)?;
    // Reuse the Native source adapter's exact unique success-close and argv0
    // seams. It copies the completed workspace first; the provider stays intact.
    let adapter_source = authored(
        author,
        "source-patch.c",
        include_str!("source_built_native_programs_final_gnu_source_patch.c").into(),
    )?;
    let adapter = author.compile(INSTALL, "binutils-ld-writer-source-patch-compile", adapter_source, "")?;
    let adapter = author.link(
        INSTALL,
        "binutils-ld-writer-source-patch-link",
        &[adapter, native.runtime.clone(), native.assembly.clone()],
        false,
    )?;
    let publisher = authored(author, "generated.c", include_str!("source_built_native_programs_generated.c").into())?;
    let adapted = author.exec(
        INSTALL,
        "binutils-ld-writer-adapt-source",
        adapter,
        Files::from([("original".into(), workspace), ("publisher".into(), publisher)]),
        vec![
            literal("ld-source"),
            input("original", ""),
            output("result", ""),
            input("publisher", ""),
            literal(SOURCE_MEMBER),
        ],
        ArtifactKind::Tree,
    )?;

    // Restoring the genuine observed times retains the completed Make state.
    // Remove only the altered translation unit and final image, so the original
    // Makefile executes its actual compile and link recipes unconditionally.
    let script = format!(
        "set -eu\ncd {RUNTIME}/source\nunset LD MANTLE_BINUTILS_CONFIGURE_PROBE_DIR MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS\n\
         export BISON={TOOLS}/bison-stage/runtime/output/bin/bison\n\
         export FLEX={TOOLS}/flex-stage/runtime/output/bin/flex\n\
         export BISON_PKGDATADIR={TOOLS}/bison-stage/runtime/output/share/bison\n\
         {TOOLS}/coreutils-stage/runtime/output/bin/rm -f ld/ldmain.o ld/ld-new\n\
         {TOOLS}/make-stage/runtime/output/bin/make -j1 -C {RUNTIME}/source/ld ld-new tooldir={PREFIX} \"CC=$CC\" \"AR=$AR\" \"SHELL=$SHELL\" RANLIB=true MAKEINFO=true CPPFLAGS=-DPLUGIN_LITTLE_ENDIAN \"CFLAGS=$CFLAGS\" > {RUNTIME}/make-ld-writer-build.stdout.txt 2> {RUNTIME}/make-ld-writer-build.stderr.txt\n"
    );
    let script = authored(author, "recipe.sh", script)?;
    let scripts =
        author.tree(INSTALL, "binutils-ld-writer-driver-source", Files::from([("recipe.sh".into(), script)]))?;
    let cwd = authored(author, "cwd.txt", format!("{RUNTIME}\n"))?;
    let zero = authored(author, "zero.txt", format!("{TOOLS}/bash-full-stage/runtime/output/bin/bash-full\n"))?;
    let inputs = Files::from([
        ("scripts".into(), scripts),
        ("initial-environment".into(), environment),
        ("initial-workspace".into(), adapted),
        ("initial-times".into(), times),
        ("controller-roots".into(), roots),
        ("initial-cwd".into(), cwd),
        ("original-zero".into(), zero),
    ]);
    let result = author.builder.register_controller(INSTALL, ControllerSpec {
        id: "binutils-original-ld-writer".into(),
        controller: controller.clone(),
        inputs,
        controller_runtime_mounts: BTreeMap::new(),
        start_arguments: vec![
            literal("--mantle-source-shell"),
            literal("run"),
            input("scripts", ""),
            literal("recipe.sh"),
            input("initial-environment", ""),
            literal("-"),
            literal("-"),
            input("initial-workspace", ""),
            output("checkpoint", ""),
        ],
        environment: BTreeMap::from([
            ("MANTLE_CONTROLLER_ROOTS".into(), input("controller-roots", "")),
            ("MANTLE_CONTROLLER_INITIAL_CWD".into(), input("initial-cwd", "")),
            ("MANTLE_CONTROLLER_INITIAL_TIMES".into(), input("initial-times", "")),
            ("MANTLE_SHELL_ORIGINAL_ZERO".into(), input("original-zero", "")),
        ]),
        restart_indices: [5, 6, 8],
        slots,
        commands_max: crate::stagex_binutils::BINUTILS_PROTECTED_EXEC_EVENT_COUNT_BOUNDS[1],
        exit_policy: RecipeExitPolicy::Exact { code: 0 },
        file_bytes_max: FILE_BYTES_MAX,
        tree_bytes_max: TREE_BYTES_MAX,
    })?;
    // The shared copier requires byte identity with a genuine isolated compiler
    // or canonicalizer product. No executable is promoted from a workspace walk.
    author.exec(
        INSTALL,
        "binutils-ld-writer-publish",
        controller,
        Files::from([
            ("workspace".into(), result.workspace),
            ("catalog".into(), result.generated),
        ]),
        vec![
            literal("--mantle-native-installed"),
            input("workspace", LD_MEMBER),
            input("catalog", ""),
            output("result", ""),
        ],
        ArtifactKind::Executable,
    )
}
