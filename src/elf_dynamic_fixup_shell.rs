// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::usize_in_public_api)]

//! Host-effects adapter for bounded ELF relocation. Planning is entirely pure.

use std::fs;
use std::io::Write;
use std::path::Path;

use mantle_application_contract::bootstrap_effects::BootstrapEffect::ReadInput;
use mantle_application_contract::bootstrap_effects::BootstrapEffect::VerifyOutput;
use mantle_application_contract::bootstrap_effects::BootstrapEffect::WriteOutput;

use crate::bootstrap_effect_port::BootstrapPort;
use crate::elf_dynamic_fixup_core::DependencyRewrite;
use crate::elf_dynamic_fixup_core::EntryKind;
use crate::elf_dynamic_fixup_core::Refusal;
use crate::elf_dynamic_fixup_core::apply_plan;
use crate::elf_dynamic_fixup_core::plan_dynamic_relocation;

const FILE_BYTES_MAX: u64 = 134_217_728; // 128 MiB
const DECLARATION_FIELDS: usize = 3;
const DECLARATION_BOUND: usize = 768;

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

/// Collects the declared triples into the rewrites the planner admits.
fn declared_rewrites<'a>(needed: &'a [String], runpath: &'a [String]) -> Vec<DependencyRewrite<'a>> {
    let declaration_total = needed.len().saturating_add(runpath.len());
    let mut rewrites = Vec::with_capacity(declaration_total / DECLARATION_FIELDS);
    for triple in needed.as_chunks::<DECLARATION_FIELDS>().0 {
        rewrites.push(DependencyRewrite {
            kind: EntryKind::Needed,
            current: &triple[0],
            dependency: &triple[1],
            library_path: &triple[2],
        });
    }
    for triple in runpath.as_chunks::<DECLARATION_FIELDS>().0 {
        rewrites.push(DependencyRewrite {
            kind: EntryKind::Runpath,
            current: &triple[0],
            dependency: &triple[1],
            library_path: &triple[2],
        });
    }
    debug_assert_eq!(rewrites.len(), declaration_total / DECLARATION_FIELDS);
    debug_assert!(rewrites.len() <= declaration_total);
    rewrites
}

/// One declaration contains the exact existing string, flat-store member, and
/// member-relative library (NEEDED) or directory (RUNPATH) path, in that order.
pub fn relocate_declared_file(
    path: &Path,
    depth: usize,
    needed: &[String],
    runpath: &[String],
) -> Result<usize, FileFixupError> {
    let declaration_total = needed.len().saturating_add(runpath.len());
    let has_needed_misaligned = !needed.len().is_multiple_of(DECLARATION_FIELDS);
    let has_runpath_misaligned = !runpath.len().is_multiple_of(DECLARATION_FIELDS);
    let is_over_bounded = declaration_total > DECLARATION_BOUND;
    if has_needed_misaligned || has_runpath_misaligned || is_over_bounded {
        return Err(FileFixupError::Input("dependency declarations must be bounded triples".into()));
    }
    let rewrites = declared_rewrites(needed, runpath);
    debug_assert!(needed.len().is_multiple_of(DECLARATION_FIELDS));
    debug_assert!(runpath.len().is_multiple_of(DECLARATION_FIELDS));
    relocate_dynamic_file(path, depth, &rewrites)
}

/// Reads the bounded regular file at `path` together with its metadata.
fn read_bounded_source(effects: &mut BootstrapPort, path: &Path) -> Result<(fs::Metadata, Vec<u8>), FileFixupError> {
    let metadata = effects
        .call(ReadInput, || fs::symlink_metadata(path))
        .map_err(|e| FileFixupError::Input(format!("{}: {e}", path.display())))?;
    let is_regular_file = metadata.is_file() && !metadata.file_type().is_symlink();
    let is_within_bound = metadata.len() > 0 && metadata.len() <= FILE_BYTES_MAX;
    if !is_regular_file || !is_within_bound {
        return Err(FileFixupError::Input(format!("{}: not a bounded regular file", path.display())));
    }
    let source = effects
        .call(ReadInput, || fs::read(path))
        .map_err(|e| FileFixupError::Input(format!("{}: {e}", path.display())))?;
    if source.len() as u64 != metadata.len() {
        return Err(FileFixupError::Input(format!("{}: input changed during read", path.display())));
    }
    debug_assert!(metadata.len() > 0);
    debug_assert!(metadata.len() <= FILE_BYTES_MAX);
    Ok((metadata, source))
}

/// Writes the relocated bytes beside `path` and publishes them atomically.
fn publish_relocated_file(
    effects: &mut BootstrapPort,
    path: &Path,
    metadata: &fs::Metadata,
    output: &[u8],
) -> Result<(), FileFixupError> {
    let staged = path.with_extension(format!("mantle-fixup-{}.tmp", std::process::id()));
    let publication = effects.call(WriteOutput, || -> Result<(), FileFixupError> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(|e| FileFixupError::Publication(format!("{}: {e}", staged.display())))?;
        file.write_all(output)
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
        // Best-effort cleanup: the port ledger records its outcome, and the
        // staged file is already absent when creation failed or the rename
        // consumed it. The publication error is returned either way.
        let removal = effects.call(WriteOutput, || fs::remove_file(&staged));
        drop(removal);
    }
    debug_assert!(!output.is_empty());
    debug_assert_eq!(staged.parent(), path.parent());
    publication?;
    Ok(())
}

/// Reads the published file back and compares it with the planned bytes.
fn verify_published_bytes(effects: &mut BootstrapPort, path: &Path, output: &[u8]) -> Result<(), FileFixupError> {
    effects.call(VerifyOutput, || {
        let actual = fs::read(path)
            .map_err(|error| FileFixupError::Publication(format!("readback {}: {error}", path.display())))?;
        if actual != output {
            return Err(FileFixupError::Publication(format!(
                "published ELF {} differs from relocation plan",
                path.display()
            )));
        }
        Ok(())
    })
}

/// Plan and publish all rewrites together; failures never publish partial bytes.
/// `depth` counts the file's parent directories beneath its output root.
// r[impl mantle.relocatable_outputs.bounded_fixup_admission]
pub fn relocate_dynamic_file(
    path: &Path,
    depth: usize,
    rewrites: &[DependencyRewrite<'_>],
) -> Result<usize, FileFixupError> {
    let mut effects = BootstrapPort::new(&[(ReadInput, 2), (WriteOutput, 2), (VerifyOutput, 1)])
        .map_err(|error| FileFixupError::Input(error.to_string()))?;
    let result = (|| {
        let (metadata, source) = read_bounded_source(&mut effects, path)?;
        let refusal = |reason| FileFixupError::Refused {
            file: path.display().to_string(),
            reason,
        };
        let plan = plan_dynamic_relocation(&source, depth, rewrites).map_err(refusal)?;
        debug_assert_eq!(plan.patches.len(), rewrites.len());
        let output = apply_plan(&source, &plan).map_err(refusal)?;
        assert_eq!(source.len(), output.len());
        publish_relocated_file(&mut effects, path, &metadata, &output)?;
        verify_published_bytes(&mut effects, path, &output)?;
        Ok(plan.patches.len())
    })();
    effects.finish_with(result, |outcome| {
        FileFixupError::Publication(format!("ELF effect observations inconsistent: {outcome:?}"))
    })
}
