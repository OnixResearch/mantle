//! Source-only optional process boundary for the genuine original Perl C core.
use std::collections::BTreeMap;

use super::c_helper;
use super::error;
use super::exec;
use super::input;
use super::output;
use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::RECORD_IDENTITY;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;

/// Install checked original selector/popen/pclose seams into an authenticated
/// Perl source tree. `transport_source` is the shared real native request/reply
/// implementation, not an executable or a replacement Perl interpreter.
/// The original recipe must still compile/link and exercise this source tree.
pub(crate) fn append_perl_process_source(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    original: ArtifactRef,
    root: &str,
    transport_source: &str,
) -> Result<ArtifactRef, RunError> {
    crate::source_built_recipe::relative(root, true)?;
    if !matches!(original, ArtifactRef::Produced { .. }) {
        return Err(error("Perl process source requires its actual original source producer"));
    }
    const PATCH: &str = "source-built/native/early-gnu-perl-process-patch.c";
    const HEADER: &str = "source-built/native/early-gnu-perl-process.h";
    const BRIDGE: &str = "source-built/native/early-gnu-perl-process.c";
    program
        .files
        .insert(HEADER.into(), include_str!("source_built_native_programs_early_gnu_perl_child.h").into());
    program.files.insert(
        BRIDGE.into(),
        format!("{transport_source}\n{}\n", include_str!("source_built_native_programs_early_gnu_perl_child.inc")),
    );
    let helper = c_helper(
        request,
        program,
        PATCH,
        include_str!("source_built_native_programs_early_gnu_perl_patch.c"),
        "perl-process-source-helper",
    )?;
    let manifest = |member: &str| ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::NativeSourceBundle,
        record_identity: RECORD_IDENTITY.into(),
        member: member.into(),
    };
    let mut state = original;
    for (mode, file) in [("doio", "doio.c"), ("ppsys", "pp_sys.c"), ("util", "util.c")] {
        let path = if root.is_empty() {
            file.into()
        } else {
            format!("{root}/{file}")
        };
        let transformed = exec(
            program,
            format!("{}.early-perl-process-{mode}", request.id),
            helper.clone(),
            BTreeMap::from([
                ("original".into(), state.clone()),
                ("header".into(), manifest(HEADER)),
                ("bridge".into(), manifest(BRIDGE)),
            ]),
            vec![
                super::literal(mode),
                input("original", &path),
                input("header", ""),
                input("bridge", ""),
                output("result"),
            ],
            ArtifactKind::Object,
        );
        state = super::originals::install_source(
            request,
            program,
            state,
            transformed,
            &path,
            &format!("perl-process-{mode}"),
        )?;
    }
    Ok(state)
}
