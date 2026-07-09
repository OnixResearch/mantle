use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use crate::errors::RunError;

const MAX_VENDOR_ROWS: usize = 128;
const MAX_VENDOR_FILES: usize = 100_000;
const VENDOR_NON_CLAIM_FRAGMENT: &str = "identity/freshness only";
const BLAKE3_HEX_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VendorFileIdentity {
    pub path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VendorManifestRow {
    pub upstream_repo: String,
    pub revision: String,
    pub filter: String,
    pub selected_paths: Vec<String>,
    pub expected_files: Vec<VendorFileIdentity>,
    pub local_edits: Vec<String>,
    pub refresh_command: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MeasuredVendorFile {
    pub path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VendorManifestValidationInput {
    pub rows: Vec<VendorManifestRow>,
    pub measured_files: Vec<MeasuredVendorFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VendorManifestValidationReport {
    pub valid: bool,
    pub diagnostics: Vec<String>,
}

// r[impl mantle.source_transports.vendor_source_manifests.validation]
pub(crate) fn validate_vendor_manifest(input: &VendorManifestValidationInput) -> VendorManifestValidationReport {
    let mut diagnostics = Vec::new();
    validate_count("rows", input.rows.len(), MAX_VENDOR_ROWS, &mut diagnostics);
    validate_count("measured_files", input.measured_files.len(), MAX_VENDOR_FILES, &mut diagnostics);
    let measured = measured_file_map(&input.measured_files, &mut diagnostics);
    for row in &input.rows {
        validate_vendor_row(row, &measured, &mut diagnostics);
    }
    VendorManifestValidationReport {
        valid: diagnostics.is_empty(),
        diagnostics,
    }
}

// r[impl mantle.source_transports.vendor_source_manifests.validation]
pub(crate) fn measure_vendor_files(
    root: &Path,
    relative_paths: &[String],
) -> Result<Vec<MeasuredVendorFile>, RunError> {
    let mut measured = Vec::with_capacity(relative_paths.len());
    for relative_path in relative_paths {
        if !is_safe_relative_path(relative_path) {
            return Err(RunError::Internal(format!("vendor path is unsafe: {relative_path}")));
        }
        let absolute_path = root.join(relative_path);
        let bytes = std::fs::read(&absolute_path)
            .map_err(|error| RunError::Internal(format!("reading {}: {error}", absolute_path.display())))?;
        measured.push(MeasuredVendorFile {
            path: relative_path.clone(),
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    Ok(measured)
}

fn validate_count(field_name: &str, count: usize, max_count: usize, diagnostics: &mut Vec<String>) {
    if count > max_count {
        diagnostics.push(format!("vendor manifest {field_name} exceeds maximum count {max_count}"));
    }
}

fn measured_file_map(measured_files: &[MeasuredVendorFile], diagnostics: &mut Vec<String>) -> BTreeMap<String, String> {
    let mut measured = BTreeMap::new();
    for file in measured_files {
        if !is_safe_relative_path(&file.path) {
            diagnostics.push(format!("measured file path is unsafe: {}", file.path));
        }
        if !is_blake3_hex(&file.digest_blake3) {
            diagnostics.push(format!("measured file digest is not BLAKE3 hex: {}", file.path));
        }
        if measured.insert(file.path.clone(), file.digest_blake3.clone()).is_some() {
            diagnostics.push(format!("measured file path is duplicated: {}", file.path));
        }
    }
    measured
}

fn validate_vendor_row(row: &VendorManifestRow, measured: &BTreeMap<String, String>, diagnostics: &mut Vec<String>) {
    push_nonempty(&row.upstream_repo, "row.upstream_repo", diagnostics);
    push_nonempty(&row.revision, "row.revision", diagnostics);
    push_nonempty(&row.filter, "row.filter", diagnostics);
    push_nonempty(&row.refresh_command, "row.refresh_command", diagnostics);
    validate_non_claims(&row.non_claims, diagnostics);
    validate_selected_paths(&row.selected_paths, diagnostics);
    validate_local_edits(&row.local_edits, diagnostics);
    validate_expected_files(row, measured, diagnostics);
    validate_undeclared_measured_files(row, measured, diagnostics);
}

fn validate_non_claims(non_claims: &[String], diagnostics: &mut Vec<String>) {
    if !non_claims.iter().any(|claim| claim.contains(VENDOR_NON_CLAIM_FRAGMENT)) {
        diagnostics.push(format!("row.non_claims missing {VENDOR_NON_CLAIM_FRAGMENT:?}"));
    }
}

fn validate_selected_paths(paths: &[String], diagnostics: &mut Vec<String>) {
    if paths.is_empty() {
        diagnostics.push("row.selected_paths must not be empty".to_string());
    }
    for path in paths {
        if !is_safe_relative_path(path) {
            diagnostics.push(format!("row.selected_paths contains unsafe path: {path}"));
        }
    }
}

fn validate_local_edits(local_edits: &[String], diagnostics: &mut Vec<String>) {
    for path in local_edits {
        if !is_safe_relative_path(path) {
            diagnostics.push(format!("row.local_edits contains unsafe path: {path}"));
        }
    }
}

fn validate_expected_files(
    row: &VendorManifestRow,
    measured: &BTreeMap<String, String>,
    diagnostics: &mut Vec<String>,
) {
    if row.expected_files.is_empty() {
        diagnostics.push("row.expected_files must not be empty".to_string());
    }
    for expected in &row.expected_files {
        validate_expected_file_path(row, expected, diagnostics);
        if !is_blake3_hex(&expected.digest_blake3) {
            diagnostics.push(format!("expected file digest is not BLAKE3 hex: {}", expected.path));
        }
        match measured.get(&expected.path) {
            Some(actual) if actual == &expected.digest_blake3 => {}
            Some(_) => diagnostics.push(format!("stale digest for vendored file: {}", expected.path)),
            None => diagnostics.push(format!("missing measured vendored file: {}", expected.path)),
        }
    }
}

fn validate_expected_file_path(row: &VendorManifestRow, expected: &VendorFileIdentity, diagnostics: &mut Vec<String>) {
    if !is_safe_relative_path(&expected.path) {
        diagnostics.push(format!("expected file path is unsafe: {}", expected.path));
        return;
    }
    if !row.selected_paths.iter().any(|selected| path_is_under(&expected.path, selected)) {
        diagnostics.push(format!("expected file is outside selected paths: {}", expected.path));
    }
}

fn validate_undeclared_measured_files(
    row: &VendorManifestRow,
    measured: &BTreeMap<String, String>,
    diagnostics: &mut Vec<String>,
) {
    for path in measured.keys() {
        let declared_expected = row.expected_files.iter().any(|expected| expected.path == *path);
        let declared_local_edit = row.local_edits.iter().any(|edit| edit == path);
        let selected = row.selected_paths.iter().any(|selected| path_is_under(path, selected));
        if selected && !declared_expected && !declared_local_edit {
            diagnostics.push(format!("undeclared local edit in vendored source: {path}"));
        }
    }
}

fn push_nonempty(value: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("vendor manifest {field_name} must not be empty"));
    }
}

fn is_safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() {
        return false;
    }
    path.components().all(|component| matches!(component, Component::Normal(_)))
}

fn path_is_under(path: &str, selected: &str) -> bool {
    path == selected || path.strip_prefix(selected).is_some_and(|suffix| suffix.starts_with('/'))
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_BYTES: &[u8] = b"vendored-source";

    #[test]
    fn validates_fresh_vendor_manifest_and_measured_file() {
        let temp = tempfile::tempdir().unwrap();
        let relative = "vendor/example/lib.rs".to_string();
        let absolute = temp.path().join(&relative);
        std::fs::create_dir_all(absolute.parent().unwrap()).unwrap();
        std::fs::write(&absolute, FIXTURE_BYTES).unwrap();
        let measured = measure_vendor_files(temp.path(), core::slice::from_ref(&relative)).unwrap();
        let input = VendorManifestValidationInput {
            rows: vec![valid_row(relative.clone(), measured[0].digest_blake3.clone())],
            measured_files: measured,
        };

        let report = validate_vendor_manifest(&input);

        assert!(report.valid);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn rejects_invalid_vendor_manifest_fixture_matrix() {
        let digest = blake3::hash(FIXTURE_BYTES).to_hex().to_string();
        let cases: [(&str, fn(&mut VendorManifestValidationInput), &str); 5] = [
            ("stale digest", stale_digest, "stale digest"),
            ("missing revision", missing_revision, "row.revision"),
            ("unsafe path", unsafe_path, "unsafe path"),
            ("undeclared local edit", undeclared_local_edit, "undeclared local edit"),
            ("missing non-claim", missing_non_claim, "identity/freshness only"),
        ];

        for (name, mutate, expected) in cases {
            let mut input = valid_input(digest.clone());
            mutate(&mut input);

            let report = validate_vendor_manifest(&input);

            assert!(!report.valid, "{name} should be rejected");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{name} diagnostics should mention {expected:?}: {:?}",
                report.diagnostics
            );
        }
    }

    #[test]
    fn measuring_rejects_path_escape() {
        let temp = tempfile::tempdir().unwrap();
        let result = measure_vendor_files(temp.path(), &["../escape.rs".to_string()]);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsafe"));
    }

    fn valid_input(digest: String) -> VendorManifestValidationInput {
        let path = "vendor/example/lib.rs".to_string();
        VendorManifestValidationInput {
            rows: vec![valid_row(path.clone(), digest.clone())],
            measured_files: vec![MeasuredVendorFile {
                path,
                digest_blake3: digest,
            }],
        }
    }

    fn valid_row(path: String, digest: String) -> VendorManifestRow {
        VendorManifestRow {
            upstream_repo: "https://example.invalid/upstream.git".to_string(),
            revision: "0123456789abcdef".to_string(),
            filter: "vendor/example/**".to_string(),
            selected_paths: vec!["vendor/example".to_string()],
            expected_files: vec![VendorFileIdentity {
                path,
                digest_blake3: digest,
            }],
            local_edits: vec![],
            refresh_command: "mantle vendor refresh example".to_string(),
            non_claims: vec!["vendored-source identity/freshness only".to_string()],
        }
    }

    fn stale_digest(input: &mut VendorManifestValidationInput) {
        input.rows[0].expected_files[0].digest_blake3 = "f".repeat(BLAKE3_HEX_LENGTH);
    }

    fn missing_revision(input: &mut VendorManifestValidationInput) {
        input.rows[0].revision.clear();
    }

    fn unsafe_path(input: &mut VendorManifestValidationInput) {
        input.rows[0].selected_paths = vec!["../vendor".to_string()];
    }

    fn undeclared_local_edit(input: &mut VendorManifestValidationInput) {
        input.measured_files.push(MeasuredVendorFile {
            path: "vendor/example/local.patch".to_string(),
            digest_blake3: input.measured_files[0].digest_blake3.clone(),
        });
    }

    fn missing_non_claim(input: &mut VendorManifestValidationInput) {
        input.rows[0].non_claims.clear();
    }
}
