//! Adapter: compare one generated file with its expected content.
//!
//! The comparison is the drift guard for files an operator contract generates,
//! so a mismatch names the file and never rewrites it.

use std::fs;
use std::path::Path;

use crate::RunError;

/// Compare one generated file with the content the contract expects.
pub(crate) fn check_generated_file(path: &Path, expected: &str) -> Result<(), RunError> {
    if path.as_os_str().is_empty() {
        return Err(RunError::Internal("generated file path must not be empty".to_string()));
    }
    if expected.is_empty() {
        return Err(RunError::Internal("generated file content must not be empty".to_string()));
    }
    debug_assert!(!expected.is_empty());
    let actual = fs::read_to_string(path)
        .map_err(|error| RunError::Internal(format!("reading generated operator file {}: {error}", path.display())))?;
    if actual != expected {
        return Err(RunError::Internal(format!("stale generated operator file: {}", path.display())));
    }
    debug_assert_eq!(actual, expected);
    debug_assert!(!actual.is_empty());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED: &str = "generated body\n";

    fn write_generated(root: &Path, contents: &str) -> std::path::PathBuf {
        let path = root.join("generated.json");
        fs::write(&path, contents).expect("write generated file");
        path
    }

    #[test]
    fn matching_content_is_accepted() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = write_generated(root.path(), EXPECTED);
        check_generated_file(&path, EXPECTED).expect("matching content is accepted");
        assert!(path.is_file());
    }

    #[test]
    fn differing_content_is_reported_as_stale() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = write_generated(root.path(), "drifted body\n");
        let error = check_generated_file(&path, EXPECTED).expect_err("drift is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("stale generated operator file"));
        assert!(rendered.contains("generated.json"));
        assert_eq!(fs::read_to_string(&path).expect("content survives"), "drifted body\n");
    }

    #[test]
    fn a_missing_file_is_reported_with_its_name() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("absent.json");
        let error = check_generated_file(&path, EXPECTED).expect_err("missing file is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("reading generated operator file"));
        assert!(rendered.contains("absent.json"));
    }

    #[test]
    fn empty_inputs_are_rejected_before_touching_the_filesystem() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = write_generated(root.path(), EXPECTED);
        let empty_path = check_generated_file(Path::new(""), EXPECTED).expect_err("empty path is rejected");
        assert!(format!("{empty_path}").contains("path must not be empty"));
        let empty_expected = check_generated_file(&path, "").expect_err("empty expectation is rejected");
        assert!(format!("{empty_expected}").contains("content must not be empty"));
    }
}
