//! Explicit source-owned StageX runtime variants for original early consumers.
use std::collections::BTreeMap;

use super::c_helper;
use super::error;
use super::exec;
use super::input;
use super::literal;
use super::output;
use super::predecessor;
use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::RECORD_IDENTITY;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;

/// Preserve the complete shared source-owned phase runtime. Its publication
/// already binds the real phase executable bytes; the native TCC rebuild adds
/// only checked original source-member changes. Neither variant is claimed
/// byte-equal to the raw provider; bind it at the explicit original literal root.
pub(super) fn literal_overrides(
    request: &NativeNodeRequest<'_, '_>,
    program: &mut NativeNodeProgram,
    native_tcc: bool,
) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
    let mut roots = BTreeMap::new();
    for logical in &request.node.derivation.input_sources {
        let logical = logical.to_absolute_path_with_prefix(&request.request.proof.logical_store_prefix);
        if !logical.ends_with("-mantle-stagex-intermediate-provider") {
            continue;
        }
        if !roots.is_empty() {
            return Err(error("ambiguous original literal StageX provider root"));
        }
        let mut tree = predecessor(request, "stagex-provider-publication:phase-runtime")?;
        if native_tcc {
            const PATCH: &str = "source-built/native/early-gnu-tcc-source-patch.c";
            const PUBLISHER: &str = "source-built/native/early-gnu-tcc-generated-writer.c";
            program
                .files
                .insert(PUBLISHER.into(), include_str!("source_built_native_programs_generated.c").into());
            let helper = c_helper(
                request,
                program,
                PATCH,
                include_str!("source_built_native_programs_early_gnu_tcc_patch.c"),
                "tcc-source-helper",
            )?;
            let publisher = ArtifactRef::ManifestSource {
                role: SourceAuthorityRole::NativeSourceBundle,
                record_identity: RECORD_IDENTITY.into(),
                member: PUBLISHER.into(),
            };
            let path = "share/mantle-stagex/tcc-source-patched/tccelf.c";
            let data = exec(
                program,
                format!("{}.early-tcc-writer-source", request.id),
                helper,
                BTreeMap::from([("original".into(), tree.clone()), ("publisher".into(), publisher)]),
                vec![
                    literal("writer"),
                    input("original", path),
                    input("publisher", ""),
                    output("result"),
                ],
                ArtifactKind::Object,
            );
            tree = super::originals::install_source(request, program, tree, data, path, "tcc-writer")?;
        }
        roots.insert(logical, tree);
    }
    Ok(roots)
}
