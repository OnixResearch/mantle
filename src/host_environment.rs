//! Adapter for the two host-environment actions the root still needs.
//!
//! The state-directory variable name, its export, and the current-executable
//! lookup live here so the root keeps no ambient-state mutation of its own.

use std::path::Path;
use std::path::PathBuf;

use crate::RunError;

/// Variable a child process reads to learn the resolved state directory.
pub(crate) const STATE_DIR_ENV: &str = "CRUNCH_STATE_DIR";

/// Export the resolved state directory for child processes.
///
/// The write is the ambient-state boundary: the shell exports one resolved value
/// so a child binary observes the same state directory as its parent.
pub(crate) fn export_state_dir(state_dir: &Path) {
    debug_assert!(!state_dir.as_os_str().is_empty());
    // SAFETY: the CLI exports one state directory before it spawns any child, so
    // no other thread in this process reads the environment concurrently.
    unsafe { std::env::set_var(STATE_DIR_ENV, state_dir) };
    debug_assert!(state_dir.is_absolute() || !state_dir.as_os_str().is_empty());
}

/// Resolve the running executable.
pub(crate) fn current_executable() -> Result<PathBuf, RunError> {
    let executable =
        std::env::current_exe().map_err(|err| RunError::Internal(format!("resolving current executable: {err}")))?;
    debug_assert!(executable.is_absolute());
    Ok(executable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_state_directory_variable_is_namespaced() {
        assert_eq!(STATE_DIR_ENV, "CRUNCH_STATE_DIR");
        assert!(STATE_DIR_ENV.chars().all(|ch| ch.is_ascii_uppercase() || ch == '_'));
    }

    #[test]
    fn the_current_executable_is_absolute_and_exists() {
        let executable = current_executable().expect("current executable");
        assert!(executable.is_absolute(), "{}", executable.display());
        assert!(executable.exists(), "{}", executable.display());
    }
}
