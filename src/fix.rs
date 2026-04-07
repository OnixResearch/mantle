use std::path::Path;

use crunch_pipeline::FodMismatch;
use nix_compat::store_path::StorePath;

use crate::errors::RunError;
use crate::build_cmd::write_log;

pub fn handle_fod_mismatch(
    mismatch: &FodMismatch,
    drv_path: &StorePath<String>,
    label: &str,
    log_dir: &Path,
    source_file: &Path,
    fix: bool,
) -> Result<(), RunError> {
    let msg = format!(
        "hash mismatch for '{}':\n expected: {}\n got:      {}",
        mismatch.name,
        mismatch.expected_sri,
        mismatch.actual_sri,
    );

    if fix {
        match auto_fix_hash(source_file, &mismatch.expected_sri, &mismatch.actual_sri) {
            Ok(()) => {
                eprintln!("{msg}");
                eprintln!("  fixed: updated {} with correct hash", source_file.display());
                write_log(log_dir, drv_path, label, false, &msg);
                return Err(RunError::Build(format!(
                    "{msg}\n  fixed: re-run to build with the corrected hash"
                )));
            }
            Err(fix_err) => {
                eprintln!("{msg}");
                eprintln!("  --fix failed: {fix_err}");
            }
        }
    } else {
        eprintln!("{msg}");
        eprintln!(
            "  update {}: hash = \"{}\"",
            source_file.display(),
            mismatch.actual_sri,
        );
    }

    write_log(log_dir, drv_path, label, false, &msg);
    Err(RunError::Build(msg))
}

pub fn auto_fix_hash(file: &Path, old_hash: &str, new_hash: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| format!("reading {}: {e}", file.display()))?;

    let count = content.matches(old_hash).count();
    if count == 0 {
        return Err(format!("hash '{}' not found in {}", old_hash, file.display()));
    }
    if count > 1 {
        return Err(format!(
            "hash '{}' appears {} times in {} -- ambiguous, not fixing",
            old_hash,
            count,
            file.display(),
        ));
    }

    let fixed = content.replacen(old_hash, new_hash, 1);
    std::fs::write(file, &fixed)
        .map_err(|e| format!("writing {}: {e}", file.display()))?;
    Ok(())
}
