//! Host-effects adapter for bounded ELF relocation. Planning is entirely pure.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::elf_dynamic_fixup_core::DependencyRewrite;
use crate::elf_dynamic_fixup_core::Refusal;
use crate::elf_dynamic_fixup_core::apply_plan;
use crate::elf_dynamic_fixup_core::plan_dynamic_relocation;

const FILE_BYTES_MAX: u64 = 128 * 1024 * 1024;

#[derive(Debug)]
pub enum FileFixupError {
    Input(String),
    Refused { file: String, reason: Refusal },
    Publication(String),
}

impl std::fmt::Display for FileFixupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(text) => write!(f, "ELF input: {text}"),
            Self::Refused { file, reason } => write!(f, "ELF fixup refused {file}: {reason}"),
            Self::Publication(text) => write!(f, "ELF publication: {text}"),
        }
    }
}
/// One declaration contains the exact existing string, flat-store member, and
/// member-relative library (NEEDED) or directory (RUNPATH) path, in that order.
pub fn relocate_declared_file(
    path: &Path,
    depth: usize,
    needed: &[String],
    runpath: &[String],
) -> Result<usize, FileFixupError> {
    if !needed.len().is_multiple_of(3) || !runpath.len().is_multiple_of(3) || needed.len() + runpath.len() > 768 {
        return Err(FileFixupError::Input("dependency declarations must be bounded triples".into()));
    }
    let mut rewrites = Vec::with_capacity((needed.len() + runpath.len()) / 3);
    for triple in needed.as_chunks::<3>().0 {
        rewrites.push(DependencyRewrite {
            kind: crate::elf_dynamic_fixup_core::EntryKind::Needed,
            current: &triple[0],
            dependency: &triple[1],
            library_path: &triple[2],
        });
    }
    for triple in runpath.as_chunks::<3>().0 {
        rewrites.push(DependencyRewrite {
            kind: crate::elf_dynamic_fixup_core::EntryKind::Runpath,
            current: &triple[0],
            dependency: &triple[1],
            library_path: &triple[2],
        });
    }
    relocate_dynamic_file(path, depth, &rewrites)
}

/// Plan and publish all rewrites together; failures never publish partial bytes.
/// `depth` counts the file's parent directories beneath its output root.
// r[impl mantle.relocatable_outputs.bounded_fixup_admission]
pub fn relocate_dynamic_file(
    path: &Path,
    depth: usize,
    rewrites: &[DependencyRewrite<'_>],
) -> Result<usize, FileFixupError> {
    use mantle_application_contract::bootstrap_effects::BootstrapEffect::*;
    let mut port =
        crate::bootstrap_effect_port::BootstrapPort::new(&[(ReadInput, 2), (WriteOutput, 2), (VerifyOutput, 1)])
            .map_err(|error| FileFixupError::Input(error.to_string()))?;
    let result = (|| {
        let metadata = port
            .call(ReadInput, || fs::symlink_metadata(path))
            .map_err(|e| FileFixupError::Input(format!("{}: {e}", path.display())))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() == 0
            || metadata.len() > FILE_BYTES_MAX
        {
            return Err(FileFixupError::Input(format!("{}: not a bounded regular file", path.display())));
        }
        let source = port
            .call(ReadInput, || fs::read(path))
            .map_err(|e| FileFixupError::Input(format!("{}: {e}", path.display())))?;
        if source.len() as u64 != metadata.len() {
            return Err(FileFixupError::Input(format!("{}: input changed during read", path.display())));
        }
        let refusal = |reason| FileFixupError::Refused {
            file: path.display().to_string(),
            reason,
        };
        let plan = plan_dynamic_relocation(&source, depth, rewrites).map_err(refusal)?;
        let output = apply_plan(&source, &plan).map_err(refusal)?;
        assert_eq!(source.len(), output.len());
        let staged = path.with_extension(format!("mantle-fixup-{}.tmp", std::process::id()));
        let publication = port.call(WriteOutput, || -> Result<(), FileFixupError> {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staged)
                .map_err(|e| FileFixupError::Publication(format!("{}: {e}", staged.display())))?;
            file.write_all(&output)
                .map_err(|e| FileFixupError::Publication(format!("{}: {e}", staged.display())))?;
            file.set_permissions(metadata.permissions())
                .map_err(|e| FileFixupError::Publication(format!("{}: {e}", staged.display())))?;
            file.sync_all().map_err(|e| FileFixupError::Publication(format!("{}: {e}", staged.display())))?;
            fs::rename(&staged, path).map_err(|e| FileFixupError::Publication(format!("{}: {e}", path.display())))?;
            fs::File::open(path.parent().ok_or_else(|| FileFixupError::Publication("missing parent".into()))?)
                .and_then(|dir| dir.sync_all())
                .map_err(|e| FileFixupError::Publication(format!("{}: {e}", path.display())))?;
            Ok(())
        });
        if publication.is_err() {
            let _ = port.call(WriteOutput, || fs::remove_file(&staged));
        }
        publication?;
        port.call(VerifyOutput, || {
            let actual = fs::read(path)
                .map_err(|error| FileFixupError::Publication(format!("readback {}: {error}", path.display())))?;
            if actual != output {
                return Err(FileFixupError::Publication(format!(
                    "published ELF {} differs from relocation plan",
                    path.display()
                )));
            }
            Ok(())
        })?;
        Ok(plan.patches.len())
    })();
    port.finish_with(result, |outcome| {
        FileFixupError::Publication(format!("ELF effect observations inconsistent: {outcome:?}"))
    })
}
