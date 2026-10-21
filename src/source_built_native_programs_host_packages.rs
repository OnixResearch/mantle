//! Complete original selected host-package shell source expansions.
use std::collections::BTreeMap;

use super::archive;
use super::archive_root;
use super::original_node_prepared;
use super::original_node_with;
use crate::errors::RunError;
use crate::source_built_native_programs::NativeNodeProgram;
use crate::source_built_native_programs::NativeNodeRequest;
use crate::source_built_native_programs::STAGE;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::SourceRecipe;
fn members(entries: &[(&str, ArtifactKind)]) -> BTreeMap<String, BTreeMap<String, ArtifactKind>> {
    BTreeMap::from([("out".into(), entries.iter().map(|(path, kind)| ((*path).into(), *kind)).collect())])
}
fn capability(name: &str, path: &str) -> BTreeMap<String, (String, String)> {
    BTreeMap::from([(name.into(), ("out".into(), path.into()))])
}
fn busybox(request: &NativeNodeRequest<'_, '_>) -> Result<NativeNodeProgram, RunError> {
    let source = archive(request, "-busybox-1.37.0-full-source-src")?;
    archive_root(source, "busybox-1.37.0", "Makefile")?;
    original_node_with(
        request,
        BTreeMap::new(),
        BTreeMap::new(),
        members(&[("bin/busybox", ArtifactKind::Executable)]),
        capability("busybox", "bin/busybox"),
        &[],
    )
}
fn cmake(request: &NativeNodeRequest<'_, '_>) -> Result<NativeNodeProgram, RunError> {
    let source = archive(request, "-cmake-3.31.8-src")?;
    let (_, member) = archive_root(source, "cmake-3.31.8", "bootstrap")?;
    let mut preparation = SourceRecipe {
        schema: crate::source_built_recipe::SCHEMA.into(),
        stage_id: STAGE.into(),
        preparation: RecipePreparation {
            tool: "original CMake source phase adaptation".into(),
            provenance: "checked original CMake launch seam before its original bootstrap and full source build".into(),
        },
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    };
    let patched = crate::source_built_rust_programs::llvm::append_cmake_source(
        request.request,
        &mut preparation,
        &format!("{}.host.cmake-source", request.id),
        source.artifact.clone(),
        &member,
    )?;
    original_node_prepared(
        request,
        BTreeMap::from([(source.original_output.clone(), patched)]),
        BTreeMap::new(),
        members(&[
            ("bin/cmake", ArtifactKind::Executable),
            ("bin/ctest", ArtifactKind::Executable),
            ("bin/cpack", ArtifactKind::Executable),
            ("share/cmake-3.31/Modules", ArtifactKind::Tree),
            ("lib/libz.a", ArtifactKind::Archive),
            ("include/zlib.h", ArtifactKind::Text),
            ("include/zconf.h", ArtifactKind::Text),
            ("include/cm_zlib_mangle.h", ArtifactKind::Text),
        ]),
        capability("cmake", "bin/cmake"),
        &[],
        preparation.steps,
        preparation.timeout_groups,
        crate::source_built_rust_programs::llvm::cmake_source_files(),
    )
}
fn python(request: &NativeNodeRequest<'_, '_>) -> Result<NativeNodeProgram, RunError> {
    let source = archive(request, "-python-3.13.5-src")?;
    archive_root(source, "Python-3.13.5", "configure")?;
    original_node_with(
        request,
        BTreeMap::new(),
        BTreeMap::new(),
        members(&[
            ("bin/python3", ArtifactKind::Executable),
            ("bin/python3.13", ArtifactKind::Executable),
            ("lib/python3.13", ArtifactKind::Tree),
            ("include/python3.13", ArtifactKind::Tree),
            ("lib/libpython3.13.a", ArtifactKind::Archive),
        ]),
        capability("python", "bin/python3"),
        &[],
    )
}
fn perl(request: &NativeNodeRequest<'_, '_>, version: &str) -> Result<NativeNodeProgram, RunError> {
    let source = archive(request, &format!("-perl-{version}-src"))?;
    let (_, root) = archive_root(source, &format!("perl-{version}"), "Configure")?;
    let mut preparation = NativeNodeProgram {
        steps: Vec::new(),
        timeout_groups: Vec::new(),
        outputs: BTreeMap::new(),
        members: BTreeMap::new(),
        phase_tools: BTreeMap::new(),
        files: BTreeMap::new(),
    };
    let transport = crate::source_built_native_programs::final_gnu::perl_transport_source();
    let patched = crate::source_built_native_programs::early_gnu::append_perl_process_source(
        request,
        &mut preparation,
        source.artifact.clone(),
        &root,
        &transport,
    )?;
    original_node_prepared(
        request,
        BTreeMap::from([(source.original_output.clone(), patched)]),
        BTreeMap::new(),
        members(&[
            ("bin/perl", ArtifactKind::Executable),
            (&format!("lib/{version}"), ArtifactKind::Tree),
        ]),
        capability("perl", "bin/perl"),
        &[],
        preparation.steps,
        preparation.timeout_groups,
        preparation.files,
    )
}
pub(crate) fn lower(request: &NativeNodeRequest<'_, '_>) -> Result<Option<NativeNodeProgram>, RunError> {
    let path = request.node.drv_path.to_string();
    let program = if path.ends_with("-busybox-1.37.0-full-source-gcc10-v1.drv") {
        busybox(request)?
    } else if path.ends_with("-cmake-3.31.8-full-source-gcc10-v1.drv") {
        cmake(request)?
    } else if path.ends_with("-python-3.13.5-full-source-gcc10-v1.drv") {
        python(request)?
    } else if path.ends_with("-perl-5.6.2-full-source-gcc10-v1.drv") {
        perl(request, "5.6.2")?
    } else if path.ends_with("-perl-5.10.1-full-source-gcc10-v5.drv") {
        perl(request, "5.10.1")?
    } else {
        return Ok(None);
    };
    Ok(Some(program))
}
