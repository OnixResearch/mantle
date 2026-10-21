//! Source-only transport for the original StageX binutils shell program.
//!
//! The shared controller interprets the original configure/config.status, TinyCC
//! wrapper, archive runner and sed bridge scripts. These hooks add only original
//! leaf operand grammars; executable selection remains the immutable catalog's
//! job. No tool image or provider is imported while composing this source.
use crate::errors::RunError;
use crate::source_built_native_programs::host::ControllerHooks;

/// Exact original wrapper text, before the original path/budget substitutions.
/// Keep one source of truth: a change to the original policy must be reflected
/// in this extraction rather than leaving a second hand-maintained wrapper.
pub(super) fn tcc_wrapper_template() -> Result<&'static str, RunError> {
    let original = include_str!("stagex_binutils.rs");
    let opener = "const TCC_WRAPPER_TEMPLATE: &str = r#\"";
    let closer = "\"#;";
    if original.matches(opener).count() != 1 {
        return Err(super::error("original binutils TinyCC wrapper declaration drift"));
    }
    original
        .split_once(opener)
        .and_then(|(_, tail)| tail.split_once(closer))
        .map(|(template, _)| template)
        .ok_or_else(|| super::error("original binutils TinyCC wrapper terminator drift"))
}

pub(super) fn controller_source(bison_image: &str, flex_image: &str) -> Result<String, RunError> {
    let mut generators = String::from("#define FINAL_GNU_CAPABILITIES");
    for (image, kind) in [
        (bison_image, 1),
        (flex_image, 2),
        ("/mantle/stagex/transition/binutils-stage/runtime/tools/bison", 1),
        ("/mantle/stagex/transition/binutils-stage/runtime/tools/flex", 2),
    ] {
        let image = serde_json::to_string(image).map_err(|cause| super::error(cause.to_string()))?;
        generators.push_str(&format!(" \\\n    {{{image},{kind}}},"));
    }
    generators.push('\n');
    generators.push_str(include_str!("source_built_native_programs_final_gnu_controller.c"));
    let launcher = include_str!("../bootstrap/stagex-sed-bridge-launcher.c");
    let constants = launcher
        .split_once("enum {")
        .and_then(|(_, tail)| tail.split_once("};"))
        .map(|(body, _)| body)
        .ok_or_else(|| super::error("original sed launcher constants boundary drift"))?;
    let validation = launcher
        .split_once("static int mantle_absolute_path(")
        .and_then(|(_, tail)| tail.split_once("static int mantle_error("))
        .map(|(body, _)| body)
        .ok_or_else(|| super::error("original sed launcher validation source boundary drift"))?;
    let dispatch = format!(
        "#include <assert.h>\nenum {{{constants}}};\n#define mantle_absolute_path stagex_binutils_absolute_path\n#define mantle_self_test stagex_binutils_launcher_self_test\nstatic int mantle_absolute_path({validation}\n#undef mantle_absolute_path\n#undef mantle_self_test\n{}",
        include_str!("source_built_stagex_programs_binutils_dispatch.c")
    );
    crate::source_built_native_programs::host::controller_source(&[
        ControllerHooks {
            source: &generators,
            external: "final_gnu_external",
            prepare: "final_gnu_prepare",
            environment: "final_gnu_environment",
        },
        ControllerHooks {
            source: &dispatch,
            external: "stagex_binutils_external",
            prepare: "stagex_binutils_prepare",
            environment: "stagex_binutils_environment",
        },
    ])
}
