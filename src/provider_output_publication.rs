//! Adapter: publish one materialized provider output into the store directory.
//!
//! Publication is idempotent: when the destination directory already exists the
//! freshly materialized tree is discarded instead of replacing store content,
//! and any other existing destination shape fails closed.

use std::fs;
use std::path::Path;

use crate::RunError;

/// Publish one materialized output path at its final location.
///
/// An existing directory destination keeps its content and the materialized
/// path is removed; an absent destination receives the materialized path by
/// rename; any other existing shape is rejected.
pub(crate) fn publish_provider_output(materialized: &Path, final_output: &Path) -> Result<(), RunError> {
    debug_assert!(!materialized.as_os_str().is_empty());
    debug_assert!(!final_output.as_os_str().is_empty());
    if final_output.exists() {
        if !final_output.is_dir() {
            return Err(RunError::Build(format!(
                "source-root provider output path exists but is not a directory: {}",
                final_output.display()
            )));
        }
        fs::remove_dir_all(materialized).map_err(|err| {
            RunError::Internal(format!("removing duplicate source-root output {}: {err}", materialized.display()))
        })?;
        debug_assert!(final_output.is_dir());
        debug_assert!(!materialized.exists());
        return Ok(());
    }
    fs::rename(materialized, final_output).map_err(|err| {
        RunError::Internal(format!(
            "moving source-root provider {} to {}: {err}",
            materialized.display(),
            final_output.display()
        ))
    })?;
    debug_assert!(final_output.is_dir());
    debug_assert!(!materialized.exists());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn materialized_tree(root: &Path) -> std::path::PathBuf {
        let path = root.join("materialized");
        fs::create_dir(&path).expect("materialized dir");
        fs::write(path.join("provider.json"), b"{}").expect("provider file");
        path
    }

    #[test]
    fn an_absent_destination_receives_the_materialized_tree() {
        let root = tempfile::tempdir().expect("tempdir");
        let materialized = materialized_tree(root.path());
        let destination = root.path().join("store-output");

        publish_provider_output(&materialized, &destination).expect("publishes");
        assert!(destination.is_dir());
        assert!(destination.join("provider.json").is_file());
        assert!(!materialized.exists());
    }

    #[test]
    fn an_existing_directory_keeps_its_content_and_discards_the_fresh_tree() {
        let root = tempfile::tempdir().expect("tempdir");
        let materialized = materialized_tree(root.path());
        let destination = root.path().join("store-output");
        fs::create_dir(&destination).expect("destination dir");
        fs::write(destination.join("provider.json"), b"existing").expect("existing file");

        publish_provider_output(&materialized, &destination).expect("publishes idempotently");
        assert_eq!(fs::read(destination.join("provider.json")).expect("existing content"), b"existing");
        assert!(!materialized.exists());
    }

    #[test]
    fn an_existing_file_destination_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let materialized = materialized_tree(root.path());
        let destination = root.path().join("store-output");
        fs::write(&destination, b"not a directory").expect("existing file");

        let error = publish_provider_output(&materialized, &destination).expect_err("file destination is rejected");
        assert!(format!("{error}").contains("exists but is not a directory"));
        assert!(materialized.is_dir(), "the fresh tree survives a rejected publication");
    }

    #[test]
    fn a_missing_materialized_tree_is_reported_with_both_paths() {
        let root = tempfile::tempdir().expect("tempdir");
        let materialized = root.path().join("absent");
        let destination = root.path().join("store-output");
        let error = publish_provider_output(&materialized, &destination).expect_err("missing source is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("moving source-root provider"));
        assert!(rendered.contains("store-output"));
    }
}
