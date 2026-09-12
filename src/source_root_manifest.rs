//! Adapter: read and validate a source-root manifest from the host filesystem.
//!
//! The decision lives in `bootstrap_source_root` (parse, validate, digest) and
//! the typed policy lives in the application contract. This adapter owns the
//! file read and the error mapping, so a command path no longer reaches the
//! filesystem for this input directly.

use std::fs;
use std::path::Path;

use crate::RunError;
use crate::bootstrap_source_root;

/// A validated source-root manifest with the facts derived from its bytes.
#[derive(Debug)]
pub(crate) struct SourceRootManifestCheck {
    pub(crate) manifest: bootstrap_source_root::SourceRootManifest,
    pub(crate) manifest_bytes: Vec<u8>,
    pub(crate) manifest_digest: String,
    pub(crate) expected_output_role_count: u32,
}

pub(crate) fn read_source_root_manifest(manifest_path: &Path) -> Result<SourceRootManifestCheck, RunError> {
    debug_assert!(!manifest_path.as_os_str().is_empty());
    let manifest_bytes = fs::read(manifest_path).map_err(|err| {
        RunError::Internal(format!("reading source-root manifest {}: {err}", manifest_path.display()))
    })?;
    if manifest_bytes.is_empty() {
        return Err(RunError::Internal(format!("source-root manifest {} is empty", manifest_path.display())));
    }
    let manifest =
        bootstrap_source_root::parse_source_root_manifest_bytes(&manifest_bytes).map_err(RunError::Internal)?;
    let validation = bootstrap_source_root::validate_source_root_manifest(&manifest)
        .map_err(|errors| RunError::Internal(bootstrap_source_root::format_diagnostics(&errors)))?;
    let expected_output_role_count = u32::try_from(validation.expected_output_roles.len()).map_err(|_| {
        RunError::Internal("source-root manifest expected output role count overflowed u32".to_string())
    })?;
    let manifest_digest = bootstrap_source_root::source_root_manifest_digest(&manifest_bytes);
    Ok(SourceRootManifestCheck {
        manifest,
        manifest_bytes,
        manifest_digest,
        expected_output_role_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest_bytes() -> Vec<u8> {
        let manifest = bootstrap_source_root::sample_source_root_manifest_for_tests();
        serde_json::to_vec(&manifest).expect("sample manifest serializes")
    }

    #[test]
    fn a_valid_manifest_file_is_read_parsed_validated_and_digested() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("source-root.json");
        let bytes = sample_manifest_bytes();
        fs::write(&path, &bytes).expect("write manifest");

        let checked = read_source_root_manifest(&path).expect("manifest is accepted");
        assert_eq!(checked.manifest_bytes, bytes);
        assert_eq!(checked.manifest_digest, bootstrap_source_root::source_root_manifest_digest(&bytes));
        assert!(checked.expected_output_role_count > 0);
    }

    #[test]
    fn a_missing_manifest_file_is_rejected() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("absent.json");
        let error = read_source_root_manifest(&path).expect_err("missing file is rejected");
        assert!(format!("{error}").contains("reading source-root manifest"));
    }

    #[test]
    fn an_empty_manifest_file_is_rejected() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("empty.json");
        fs::write(&path, b"").expect("write empty manifest");
        let error = read_source_root_manifest(&path).expect_err("empty file is rejected");
        assert!(format!("{error}").contains("is empty"));
    }

    #[test]
    fn a_malformed_manifest_file_is_rejected() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("malformed.json");
        fs::write(&path, b"{\"provider_name\":").expect("write malformed manifest");
        let error = read_source_root_manifest(&path).expect_err("malformed file is rejected");
        assert!(format!("{error}").contains("parsing source-root manifest JSON"));
    }

    #[test]
    fn an_incomplete_manifest_file_is_rejected() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("incomplete.json");
        fs::write(&path, b"{}").expect("write incomplete manifest");
        let error = read_source_root_manifest(&path).expect_err("incomplete file is rejected");
        let rendered = format!("{error}");
        assert!(!rendered.is_empty(), "an incomplete manifest must report diagnostics");
        assert!(
            rendered.contains("provider") || rendered.contains("manifest"),
            "diagnostics must name the missing manifest facts: {rendered}"
        );
    }
}
