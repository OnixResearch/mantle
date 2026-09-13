//! Adapter: prepare the scratch directory for the Rust source provider.
//!
//! A requested path is created once and canonicalized so later comparisons use
//! one spelling; without a request the adapter creates an ephemeral directory
//! that is removed when the scratch value drops.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::RunError;

/// Prefix used for ephemeral provider scratch directories.
pub(crate) const EPHEMERAL_SCRATCH_PREFIX: &str = "mantle-rust-source-provider-";

/// Provider scratch that is either created by the adapter or requested by the operator.
#[derive(Debug)]
pub(crate) enum RustSourceProviderScratch {
    Ephemeral(tempfile::TempDir),
    Persistent(PathBuf),
}

impl RustSourceProviderScratch {
    /// Path of the scratch directory.
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Ephemeral(scratch) => scratch.path(),
            Self::Persistent(path) => path,
        }
    }

    /// Take ownership of the scratch directory so it outlives the process.
    pub(crate) fn preserve(self) -> PathBuf {
        match self {
            Self::Ephemeral(scratch) => scratch.keep(),
            Self::Persistent(path) => path,
        }
    }
}

pub(crate) fn prepare_rust_source_provider_scratch(
    requested_path: Option<&Path>,
) -> Result<RustSourceProviderScratch, RunError> {
    if let Some(path) = requested_path {
        fs::create_dir(path).map_err(|error| {
            RunError::Internal(format!("creating persistent Rust provider scratch {}: {error}", path.display()))
        })?;
        let canonical = fs::canonicalize(path).map_err(|error| {
            RunError::Internal(format!("canonicalizing persistent Rust provider scratch {}: {error}", path.display()))
        })?;
        return Ok(RustSourceProviderScratch::Persistent(canonical));
    }
    let scratch = tempfile::Builder::new()
        .prefix(EPHEMERAL_SCRATCH_PREFIX)
        .tempdir()
        .map_err(|error| RunError::Internal(format!("creating Rust provider scratch: {error}")))?;
    Ok(RustSourceProviderScratch::Ephemeral(scratch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_requested_path_is_created_and_canonicalized() {
        let root = tempfile::tempdir().expect("tempdir");
        let requested = root.path().join("scratch");
        let scratch = prepare_rust_source_provider_scratch(Some(&requested)).expect("scratch is created");
        assert!(matches!(&scratch, RustSourceProviderScratch::Persistent(_)));
        assert!(scratch.path().is_dir());
        assert_eq!(scratch.path(), fs::canonicalize(&requested).expect("canonical path"));
    }

    #[test]
    fn a_requested_path_that_already_exists_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let requested = root.path().join("scratch");
        fs::create_dir(&requested).expect("existing scratch");
        let error = prepare_rust_source_provider_scratch(Some(&requested)).expect_err("duplicate is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("creating persistent Rust provider scratch"));
        assert!(rendered.contains("scratch"));
    }

    #[test]
    fn a_requested_path_with_a_missing_parent_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let requested = root.path().join("absent").join("scratch");
        let error = prepare_rust_source_provider_scratch(Some(&requested)).expect_err("missing parent is rejected");
        assert!(format!("{error}").contains("creating persistent Rust provider scratch"));
    }

    #[test]
    fn an_absent_request_creates_a_prefixed_ephemeral_directory_removed_on_drop() {
        let scratch = prepare_rust_source_provider_scratch(None).expect("ephemeral scratch");
        assert!(matches!(&scratch, RustSourceProviderScratch::Ephemeral(_)));
        let path = scratch.path().to_path_buf();
        assert!(path.is_dir());
        assert!(
            path.file_name()
                .map(|name| name.to_string_lossy().starts_with(EPHEMERAL_SCRATCH_PREFIX))
                .unwrap_or(false)
        );
        drop(scratch);
        assert!(!path.exists(), "ephemeral scratch is removed on drop");
    }

    #[test]
    fn preserving_an_ephemeral_scratch_keeps_its_directory() {
        let scratch = prepare_rust_source_provider_scratch(None).expect("ephemeral scratch");
        let path = scratch.preserve();
        assert!(path.is_dir());
        assert!(
            path.file_name()
                .map(|name| name.to_string_lossy().starts_with(EPHEMERAL_SCRATCH_PREFIX))
                .unwrap_or(false)
        );
        fs::remove_dir_all(&path).expect("clean up preserved scratch");
    }
}
