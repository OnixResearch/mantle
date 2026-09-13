//! Debug-only remote transfer interruption scaffold.
//!
//! The scaffold binds one count from the environment onto a prepared remote
//! command. The variable names, the debug-build gate, and the count admission
//! live here, so the composition root does not read them.

use std::ffi::OsString;

use crate::RunError;
use crate::remote_build;

/// Variable that interrupts a remote input stream after the bound chunk count.
pub(crate) const AFTER_INPUT_CHUNKS_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_INPUT_CHUNKS";
/// Variable that interrupts a remote output stream after the bound chunk count.
pub(crate) const AFTER_OUTPUT_CHUNKS_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS";

/// Apply the requested interruption count when this is a debug build.
///
/// `environment_variable` names the bound variable; `is_input` selects the input
/// or output half. A release build ignores the request, an absent variable makes
/// no change, and a value that is not a UTF-8 `u32` fails closed.
pub(crate) fn configure_interruption(
    command: &mut remote_build::RemoteStdioCommand,
    environment_variable: &str,
    is_input: bool,
) -> Result<(), RunError> {
    debug_assert!(!environment_variable.is_empty());
    if !cfg!(debug_assertions) {
        return Ok(());
    }
    let Some(value) = read_bound_value(environment_variable) else {
        return Ok(());
    };
    let chunk_count = parse_chunk_count(&value)?;
    if is_input {
        remote_build::set_remote_production_interrupt_after_input_chunks(command, chunk_count)
    } else {
        remote_build::set_remote_production_interrupt_after_output_chunks(command, chunk_count)
    }
    .map_err(RunError::Internal)
}

/// Read the bound variable, if it is present.
fn read_bound_value(environment_variable: &str) -> Option<OsString> {
    debug_assert!(!environment_variable.is_empty());
    std::env::var_os(environment_variable)
}

/// Convert bound text into a chunk count.
///
/// The count is admitted through a wider integer so a value above the `u32`
/// chunk-count bound fails closed instead of wrapping.
fn parse_chunk_count(value: &OsString) -> Result<u32, RunError> {
    let text = value
        .to_str()
        .ok_or_else(|| RunError::Internal("remote test interruption count is not UTF-8".to_string()))?;
    let wide = text
        .parse::<u64>()
        .map_err(|err| RunError::Internal(format!("remote test interruption count is invalid: {err}")))?;
    let count = u32::try_from(wide).map_err(|_| {
        RunError::Internal(format!("remote test interruption count {wide} exceeds the chunk-count bound"))
    })?;
    debug_assert_eq!(u64::from(count), wide);
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_variable_names_are_namespaced() {
        assert!(AFTER_INPUT_CHUNKS_ENV.starts_with("MANTLE_TEST_REMOTE_INTERRUPT_"));
        assert!(AFTER_OUTPUT_CHUNKS_ENV.starts_with("MANTLE_TEST_REMOTE_INTERRUPT_"));
        assert_ne!(AFTER_INPUT_CHUNKS_ENV, AFTER_OUTPUT_CHUNKS_ENV);
        assert!(AFTER_INPUT_CHUNKS_ENV.contains("INPUT"));
        assert!(AFTER_OUTPUT_CHUNKS_ENV.contains("OUTPUT"));
    }

    #[test]
    fn an_absent_variable_reads_as_nothing() {
        assert!(read_bound_value("MANTLE_TEST_VARIABLE_THAT_IS_NOT_SET").is_none());
    }

    #[test]
    fn a_bound_count_parses_and_a_bad_one_fails_closed() {
        assert_eq!(parse_chunk_count(&OsString::from("7")).expect("count"), 7);
        assert_eq!(parse_chunk_count(&OsString::from("0")).expect("count"), 0);
        assert!(parse_chunk_count(&OsString::from("not-a-count")).is_err());
        assert!(parse_chunk_count(&OsString::from("")).is_err());
        assert!(parse_chunk_count(&OsString::from("-1")).is_err());
        assert!(parse_chunk_count(&OsString::from("4294967296")).is_err());
        assert_eq!(parse_chunk_count(&OsString::from("4294967295")).expect("count"), u32::MAX);
    }
}
