//! Adapter: write generated text to an output path.
//!
//! Generated files are written whole and replaced whole, so a failed write
//! names the path and never leaves a partial file behind a successful exit.

use std::fs;
use std::path::Path;

use crate::RunError;

/// Write generated text to one output path, replacing existing content.
pub(crate) fn write_generated_text(path: &Path, text: &str) -> Result<(), RunError> {
    if path.as_os_str().is_empty() {
        return Err(RunError::Internal("generated output path must not be empty".to_string()));
    }
    debug_assert!(!path.as_os_str().is_empty());
    fs::write(path, text).map_err(|error| RunError::Internal(format!("writing {}: {error}", path.display())))?;
    debug_assert!(path.is_file());
    debug_assert_eq!(
        fs::metadata(path).map(|meta| meta.len()).unwrap_or(u64::MAX),
        u64::try_from(text.len()).unwrap_or(u64::MAX)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_text_is_written_whole() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("generated.ncl");
        let text = "let value = 1;\n";
        write_generated_text(&path, text).expect("writes");
        assert_eq!(fs::read_to_string(&path).expect("read back"), text);
    }

    #[test]
    fn existing_content_is_replaced() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("generated.ncl");
        fs::write(&path, "stale content that is longer\n").expect("seed file");
        write_generated_text(&path, "short\n").expect("writes");
        assert_eq!(fs::read_to_string(&path).expect("read back"), "short\n");
    }

    #[test]
    fn a_missing_parent_directory_is_reported_with_the_path() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("absent").join("generated.ncl");
        let error = write_generated_text(&path, "let value = 1;\n").expect_err("missing parent is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("writing "));
        assert!(rendered.contains("generated.ncl"));
    }

    #[test]
    fn an_empty_output_path_is_rejected_before_io() {
        let error = write_generated_text(Path::new(""), "text").expect_err("empty path is rejected");
        assert!(format!("{error}").contains("must not be empty"));
    }
}
