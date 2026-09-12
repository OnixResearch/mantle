//! Adapter: run one selected artifact as a child process.
//!
//! The composition root selects the binary and decides what the CLI does with
//! the result; this adapter owns process creation, the child's inherited
//! standard streams, and the exit-code mapping.

use std::path::Path;
use std::process::ExitStatus;

use crate::RunError;

/// Status used when a child terminated without an exit code, such as from a
/// signal.
pub(crate) const CHILD_NO_EXIT_CODE_STATUS: i32 = 1;

/// Largest status a process exit code can carry.
pub(crate) const CHILD_EXIT_CODE_MAX: i32 = 255;

/// Run one child executable with inherited standard streams.
pub(crate) fn run_child(exe_path: &Path, args: &[String]) -> Result<ExitStatus, RunError> {
    if exe_path.as_os_str().is_empty() {
        return Err(RunError::Internal("child executable path is empty".to_string()));
    }
    debug_assert!(!exe_path.as_os_str().is_empty());
    let status = std::process::Command::new(exe_path)
        .args(args)
        .status()
        .map_err(|error| RunError::Internal(format!("exec {}: {error}", exe_path.display())))?;
    debug_assert_eq!(status.success(), status.code() == Some(0));
    Ok(status)
}

/// Process exit code for a finished child.
///
/// A child that terminated without an exit code, such as from a signal, maps to
/// [`CHILD_NO_EXIT_CODE_STATUS`].
pub(crate) fn child_exit_code(status: ExitStatus) -> i32 {
    let code = status.code().unwrap_or(CHILD_NO_EXIT_CODE_STATUS);
    debug_assert!(status.code().is_some() || code == CHILD_NO_EXIT_CODE_STATUS);
    debug_assert!(code >= 0 && code <= CHILD_EXIT_CODE_MAX);
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_exe() -> std::path::PathBuf {
        std::env::current_exe().expect("test binary path")
    }

    #[test]
    fn a_successful_child_reports_success_and_code_zero() {
        let status = run_child(&current_exe(), &[String::from("--list")]).expect("child runs");
        assert!(status.success());
        assert_eq!(child_exit_code(status), 0);
        assert_eq!(status.code(), Some(0));
    }

    #[test]
    fn a_failing_child_reports_its_code() {
        let shell = Path::new("/bin/sh");
        assert!(shell.exists(), "this environment provides /bin/sh");
        let status = run_child(shell, &[String::from("-c"), String::from("exit 7")]).expect("child runs");
        assert!(!status.success());
        assert_eq!(child_exit_code(status), 7);
        assert_eq!(status.code(), Some(7));
    }

    #[test]
    fn a_missing_executable_is_rejected() {
        let directory = tempfile::tempdir().expect("tempdir");
        let missing = directory.path().join("absent-binary");
        let error = run_child(&missing, &[]).expect_err("missing executable is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("exec "));
        assert!(rendered.contains("absent-binary"));
    }

    #[test]
    fn an_empty_executable_path_is_rejected() {
        let error = run_child(Path::new(""), &[]).expect_err("empty path is rejected");
        assert!(format!("{error}").contains("empty"));
        assert_eq!(CHILD_EXIT_CODE_MAX, 255);
    }
}
