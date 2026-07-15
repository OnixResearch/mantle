use std::path::Path;

use crunch_pipeline::FodMismatch;
use nix_compat::store_path::StorePath;

use crate::errors::RunError;

#[expect(
    tigerstyle::too_many_parameters,
    tigerstyle::ambiguous_boolean_parameters,
    reason = "the protected build command caller retains this compatibility boundary"
)]
pub fn handle_fod_mismatch(
    mismatch: &FodMismatch,
    _drv_path: &StorePath<String>,
    _label: &str,
    source_file: &Path,
    fix: bool,
    emit_human: bool,
) -> Result<(), RunError> {
    debug_assert!(!_drv_path.name().is_empty());
    debug_assert_ne!(mismatch.expected_sri, mismatch.actual_sri);
    let msg = format!(
        "hash mismatch for '{}':\n expected: {}\n got:      {}",
        mismatch.name, mismatch.expected_sri, mismatch.actual_sri,
    );

    if fix {
        match auto_fix_hash(HashReplacement {
            file: source_file,
            old_hash: &mismatch.expected_sri,
            new_hash: &mismatch.actual_sri,
        }) {
            Ok(()) => {
                if emit_human {
                    eprintln!("{msg}");
                    eprintln!("  fixed: updated {} with correct hash", source_file.display());
                }
                return Err(RunError::Build(format!("{msg}\n  fixed: re-run to build with the corrected hash")));
            }
            Err(fix_err) => {
                if emit_human {
                    eprintln!("{msg}");
                    eprintln!("  --fix failed: {fix_err}");
                }
                return Err(RunError::Build(format!("{msg}\n  --fix failed: {fix_err}")));
            }
        }
    }

    if emit_human {
        eprintln!("{msg}");
        eprintln!("  update {}: hash = \"{}\"", source_file.display(), mismatch.actual_sri,);
    }
    Err(RunError::Build(msg))
}

pub struct HashReplacement<'a> {
    pub file: &'a Path,
    pub old_hash: &'a str,
    pub new_hash: &'a str,
}

pub fn auto_fix_hash(replacement: HashReplacement<'_>) -> Result<(), String> {
    let content = std::fs::read_to_string(replacement.file)
        .map_err(|error| format!("reading {}: {error}", replacement.file.display()))?;

    let match_count = content.matches(replacement.old_hash).count();
    if match_count == 0 {
        return Err(format!("hash '{}' not found in {}", replacement.old_hash, replacement.file.display()));
    }
    if match_count > 1 {
        return Err(format!(
            "hash '{}' appears {} times in {} -- ambiguous, not fixing",
            replacement.old_hash,
            match_count,
            replacement.file.display(),
        ));
    }

    let fixed = content.replacen(replacement.old_hash, replacement.new_hash, 1);
    debug_assert_eq!(match_count, 1);
    debug_assert!(fixed.contains(replacement.new_hash));
    std::fs::write(replacement.file, &fixed)
        .map_err(|error| format!("writing {}: {error}", replacement.file.display()))?;
    Ok(())
}
