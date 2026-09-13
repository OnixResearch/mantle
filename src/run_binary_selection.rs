//! Adapter: select the executable to run from a built output tree.
//!
//! Selection reads only the candidate tree: the name policy comes from the
//! application contract, and executability is decided from file metadata.

use std::path::Path;
use std::path::PathBuf;

use crate::RunError;

/// Permission bits that mark a file executable.
const UNIX_EXECUTE_BITS: u32 = 0o111;

#[cfg(unix)]
fn metadata_has_execute_bit(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & UNIX_EXECUTE_BITS != 0
}

#[cfg(not(unix))]
fn metadata_has_execute_bit(_metadata: &std::fs::Metadata) -> bool {
    true
}

/// Whether one path is an executable regular file or a symlink to one.
fn is_executable_file_or_symlink(path: &Path) -> bool {
    let Ok(symlink_metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if symlink_metadata.is_dir() {
        return false;
    }
    if !symlink_metadata.is_file() && !symlink_metadata.file_type().is_symlink() {
        return false;
    }
    let Ok(target_metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !target_metadata.is_file() {
        return false;
    }
    metadata_has_execute_bit(&target_metadata)
}

/// Select the executable to run from one built output tree.
///
/// An explicit `bin` name must be a single admissible entry and must resolve to
/// an executable; without one, the first executable entry in name order wins.
pub(crate) fn select_run_binary(out_path: &Path, bin: Option<&str>) -> Result<PathBuf, RunError> {
    // An empty output path is a caller defect, so it fails closed rather than
    // resolving to a relative "bin" directory.
    if out_path.as_os_str().is_empty() {
        return Err(RunError::Internal("run output path must not be empty".to_string()));
    }
    let bin_dir = out_path.join("bin");
    if !bin_dir.is_dir() {
        return Err(RunError::Internal(format!("no bin/ directory in {}", out_path.display())));
    }

    if let Some(name) = bin {
        if !mantle_application_contract::is_run_bin_name_admissible(name) {
            return Err(RunError::Internal(format!("invalid --bin value '{name}': expected a single bin/ entry name")));
        }
        let exe_path = bin_dir.join(name);
        if !is_executable_file_or_symlink(&exe_path) {
            return Err(RunError::Internal(format!("selected binary is not executable: {}", exe_path.display())));
        }
        return Ok(exe_path);
    }

    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&bin_dir)
        .map_err(|e| RunError::Internal(format!("reading {}: {e}", bin_dir.display())))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_executable_file_or_symlink(path))
        .collect();
    candidates.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));

    candidates
        .first()
        .cloned()
        .ok_or_else(|| RunError::Internal(format!("no executables in {}", bin_dir.display())))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    const TEST_READ_MODE: u32 = 0o644;
    const TEST_EXEC_MODE: u32 = 0o755;

    fn write_file_with_mode(path: &Path, contents: &str, mode: u32) {
        std::fs::write(path, contents).expect("write file");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("set mode");
    }

    fn output_tree() -> tempfile::TempDir {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(temp.path().join("bin")).expect("bin dir");
        temp
    }

    #[test]
    fn run_bin_name_rejects_path_like_values() {
        assert!(mantle_application_contract::is_run_bin_name_admissible("tool"));
        assert!(!mantle_application_contract::is_run_bin_name_admissible(""));
        assert!(!mantle_application_contract::is_run_bin_name_admissible("."));
        assert!(!mantle_application_contract::is_run_bin_name_admissible(".."));
        assert!(!mantle_application_contract::is_run_bin_name_admissible("/tool"));
        assert!(!mantle_application_contract::is_run_bin_name_admissible("../tool"));
        assert!(!mantle_application_contract::is_run_bin_name_admissible("dir/tool"));
    }

    #[test]
    fn run_binary_fallback_skips_invalid_entries() {
        let temp = output_tree();
        let bin = temp.path().join("bin");
        write_file_with_mode(&bin.join("aaa"), "no execute", TEST_READ_MODE);
        std::os::unix::fs::symlink(bin.join("missing"), bin.join("aab")).expect("dangling symlink");
        write_file_with_mode(&bin.join("bbb"), "#!/bin/sh\n", TEST_EXEC_MODE);

        let selected = select_run_binary(temp.path(), None).expect("selects a candidate");
        assert_eq!(selected.file_name().unwrap(), "bbb");
        assert!(selected.ends_with("bbb"));
    }

    #[test]
    fn run_binary_explicit_symlink_selects_executable_target() {
        let temp = output_tree();
        let bin = temp.path().join("bin");
        write_file_with_mode(&bin.join("real"), "#!/bin/sh\n", TEST_EXEC_MODE);
        std::os::unix::fs::symlink(bin.join("real"), bin.join("alias")).expect("symlink");

        let selected = select_run_binary(temp.path(), Some("alias")).expect("symlink selects");
        assert_eq!(selected.file_name().unwrap(), "alias");
        assert!(selected.ends_with("alias"));
    }

    #[test]
    fn run_binary_explicit_rejects_non_executable_target() {
        let temp = output_tree();
        write_file_with_mode(&temp.path().join("bin/tool"), "no execute", TEST_READ_MODE);
        let err = select_run_binary(temp.path(), Some("tool")).expect_err("not executable").to_string();
        assert!(err.contains("not executable"));
        assert!(err.contains("tool"));
    }

    #[test]
    fn a_missing_or_empty_bin_directory_is_rejected() {
        let temp = tempfile::tempdir().expect("tempdir");
        let missing = select_run_binary(temp.path(), None).expect_err("missing bin dir").to_string();
        assert!(missing.contains("no bin/ directory"));
        std::fs::create_dir(temp.path().join("bin")).expect("bin dir");
        let empty = select_run_binary(temp.path(), None).expect_err("empty bin dir").to_string();
        assert!(empty.contains("no executables in"));
    }

    #[test]
    fn an_invalid_explicit_name_is_rejected_before_lookup() {
        let temp = output_tree();
        let err = select_run_binary(temp.path(), Some("dir/tool")).expect_err("invalid name").to_string();
        assert!(err.contains("invalid --bin value"));
        assert!(err.contains("dir/tool"));
    }

    #[test]
    fn an_empty_output_path_is_rejected_without_resolving_a_relative_bin_directory() {
        let err = select_run_binary(Path::new(""), None).expect_err("empty path is rejected").to_string();
        assert!(err.contains("must not be empty"));
        assert!(!err.contains("bin/ directory"));
    }
}
