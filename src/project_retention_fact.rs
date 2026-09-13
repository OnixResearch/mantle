//! Adapter: read one bounded project retention fact.
//!
//! The retention identity depends on exact file bytes, so the read is bounded,
//! limited to regular files, and reported with the offending path.

use std::path::Path;

use crate::RunError;

/// Largest admitted project retention fact.
pub(crate) const MAX_PROJECT_RETENTION_FACT_BYTES: u64 = 16_777_216;

/// Read one bounded retention fact from the filesystem.
///
/// The fact must be a regular file within the admitted bound; every rejection
/// names the path so an operator can act on it.
pub(crate) fn read_bounded_project_retention_fact(path: &Path) -> Result<Vec<u8>, RunError> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        RunError::Internal(format!("reading project retention metadata {}: {error}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("project retention fact is not a file: {}", path.display())));
    }
    if metadata.len() > MAX_PROJECT_RETENTION_FACT_BYTES {
        return Err(RunError::Internal(format!(
            "project retention fact exceeds {MAX_PROJECT_RETENTION_FACT_BYTES} bytes: {}",
            path.display()
        )));
    }
    let bytes = std::fs::read(path)
        .map_err(|error| RunError::Internal(format!("reading project retention fact {}: {error}", path.display())))?;
    debug_assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= MAX_PROJECT_RETENTION_FACT_BYTES);
    debug_assert_eq!(bytes.len(), usize::try_from(metadata.len()).unwrap_or(bytes.len()));
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_regular_file_inside_the_bound_is_read() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("retention.json");
        std::fs::write(&path, b"{\"schema\":\"retention\"}").expect("write fact");
        let bytes = read_bounded_project_retention_fact(&path).expect("reads");
        assert_eq!(bytes, b"{\"schema\":\"retention\"}");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn an_empty_file_is_read_as_empty() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("empty.json");
        std::fs::write(&path, b"").expect("write fact");
        let bytes = read_bounded_project_retention_fact(&path).expect("reads");
        assert!(bytes.is_empty());
        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn a_missing_path_is_rejected_with_its_name() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("absent.json");
        let error = read_bounded_project_retention_fact(&path).expect_err("missing is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("reading project retention metadata"));
        assert!(rendered.contains("absent.json"));
    }

    #[test]
    fn a_directory_is_rejected() {
        let directory = tempfile::tempdir().expect("tempdir");
        let error = read_bounded_project_retention_fact(directory.path()).expect_err("directory is rejected");
        assert!(format!("{error}").contains("is not a file"));
    }

    #[test]
    fn a_file_over_the_bound_is_rejected() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("oversized.json");
        let over_bound = usize::try_from(MAX_PROJECT_RETENTION_FACT_BYTES).unwrap_or(0) + 1;
        std::fs::write(&path, vec![b'a'; over_bound]).expect("write oversized fact");
        let error = read_bounded_project_retention_fact(&path).expect_err("oversized is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("exceeds"));
        assert!(rendered.contains(&MAX_PROJECT_RETENTION_FACT_BYTES.to_string()));
    }
}
