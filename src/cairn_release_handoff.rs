use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::CAIRN_HANDOFF_INPUT_SCHEMA;
use crunch_release_core::CairnHandoffAuthenticationDependency;
use crunch_release_core::CairnMeasuredArtifact;
use crunch_release_core::CairnReleaseEvidenceHandoff;
use crunch_release_core::CairnReleaseEvidenceRow;
use crunch_release_core::CairnReleaseEvidenceValidationReceipt;
use crunch_release_core::MAX_CAIRN_HANDOFF_ROWS_COUNT;
use crunch_release_core::MAX_CAIRN_HANDOFF_TEXT_BYTES_COUNT;
use serde::Deserialize;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::ValidatedReleasePath;

const MAX_CAIRN_HANDOFF_DESCRIPTOR_BYTES: u64 = 1_048_576;
const MAX_CAIRN_HANDOFF_ARTIFACT_BYTES: u64 = 67_108_864;
const CAIRN_HANDOFF_BUNDLE_DIRECTORY: &str = "cairn-handoff";
const CAIRN_HANDOFF_ARTIFACT_FILE: &str = "artifact";
const CAIRN_HANDOFF_POLICY_FILE: &str = "policy";
const CAIRN_HANDOFF_AUTHENTICATION_DIRECTORY: &str = "authentication";
const CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_FILE: &str = "archive-receipt.json";
const CAIRN_HANDOFF_ARTIFACTS_PER_ROW: usize = 2;
const CAIRN_HANDOFF_AUTHENTICATION_ARTIFACTS_COUNT: usize = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CairnHandoffDescriptor {
    schema: String,
    authentication: CairnHandoffDescriptorAuthentication,
    rows: Vec<CairnHandoffDescriptorRow>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CairnHandoffDescriptorAuthentication {
    schema: String,
    change_name: String,
    cairn_revision: String,
    archive_manifest_blake3: String,
    archive_mutation_receipt_blake3: String,
    archive_receipt_path: PathBuf,
    archive_receipt_digest_blake3: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CairnHandoffDescriptorRow {
    artifact_id: String,
    role: String,
    schema_id: String,
    artifact_path: PathBuf,
    artifact_digest_blake3: String,
    cairn_policy_path: PathBuf,
    cairn_policy_digest_blake3: String,
    release_readiness_id: String,
    covers: Vec<String>,
    non_claims: Vec<String>,
}

// r[impl mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
// r[impl mantle.release_provenance.cairn_evidence_handoff.production_wiring]
pub fn prepare_cairn_handoff_for_bundle(
    descriptor_path: &Path,
    bundle_dir: &Path,
) -> Result<CairnReleaseEvidenceHandoff, RunError> {
    let descriptor_bytes = read_bounded_file(
        descriptor_path,
        MAX_CAIRN_HANDOFF_DESCRIPTOR_BYTES,
        "Cairn handoff descriptor",
        ReleaseRootKind::BuildArtifact,
    )?;
    let descriptor: CairnHandoffDescriptor = serde_json::from_slice(&descriptor_bytes).map_err(|error| {
        RunError::Internal(format!("parsing Cairn handoff descriptor {}: {error}", descriptor_path.display()))
    })?;
    validate_descriptor_header(&descriptor)?;
    let descriptor_parent = descriptor_path.parent().unwrap_or_else(|| Path::new("."));
    let preflight = preflight_handoff(&descriptor, descriptor_parent, bundle_dir)?;
    debug_assert_eq!(preflight.rows.len(), descriptor.rows.len());
    let authentication = prepare_authentication(descriptor.authentication, descriptor_parent, bundle_dir)?;
    let mut rows = Vec::with_capacity(descriptor.rows.len());
    for (index, row) in descriptor.rows.into_iter().enumerate() {
        let index_u32 = u32::try_from(index)
            .map_err(|_| RunError::Internal("Cairn handoff row index overflowed u32".to_string()))?;
        rows.push(prepare_row(row, index_u32, descriptor_parent, bundle_dir)?);
    }
    let handoff = CairnReleaseEvidenceHandoff { authentication, rows };
    let validation_result = crunch_release_core::validate_cairn_release_evidence_handoff(&handoff);
    if !validation_result.valid {
        return Err(RunError::Internal(format!(
            "Cairn handoff descriptor failed measured validation: {}",
            validation_result.diagnostics.join("; ")
        )));
    }
    let maximum_rows_count = maximum_handoff_rows_count()?;
    debug_assert!(!handoff.rows.is_empty());
    debug_assert!(handoff.rows.len() <= maximum_rows_count);
    Ok(handoff)
}

pub fn plan_cairn_handoff_artifacts(
    descriptor_path: &Path,
    bundle_dir: &Path,
) -> Result<Vec<CairnMeasuredArtifact>, RunError> {
    let descriptor_bytes = read_bounded_file(
        descriptor_path,
        MAX_CAIRN_HANDOFF_DESCRIPTOR_BYTES,
        "Cairn handoff descriptor",
        ReleaseRootKind::BuildArtifact,
    )?;
    let descriptor: CairnHandoffDescriptor = serde_json::from_slice(&descriptor_bytes).map_err(|error| {
        RunError::Internal(format!("parsing Cairn handoff descriptor {}: {error}", descriptor_path.display()))
    })?;
    validate_descriptor_header(&descriptor)?;
    let descriptor_parent = descriptor_path.parent().unwrap_or_else(|| Path::new("."));
    let handoff = preflight_handoff(&descriptor, descriptor_parent, bundle_dir)?;
    let expected_row_artifacts = handoff.rows.len().saturating_mul(CAIRN_HANDOFF_ARTIFACTS_PER_ROW);
    let mut artifacts =
        Vec::with_capacity(expected_row_artifacts.saturating_add(CAIRN_HANDOFF_AUTHENTICATION_ARTIFACTS_COUNT));
    artifacts.push(handoff.authentication.archive_receipt);
    for row in handoff.rows {
        artifacts.push(row.artifact);
        artifacts.push(row.cairn_policy);
    }
    debug_assert!(!artifacts.is_empty());
    debug_assert_eq!(
        artifacts.len(),
        expected_row_artifacts.saturating_add(CAIRN_HANDOFF_AUTHENTICATION_ARTIFACTS_COUNT)
    );
    Ok(artifacts)
}

pub fn remeasure_cairn_handoff_from_bundle(
    bundle_dir: &Path,
    receipt: &CairnReleaseEvidenceValidationReceipt,
) -> Result<CairnReleaseEvidenceHandoff, RunError> {
    let mut handoff = receipt.handoff.clone();
    handoff.authentication.archive_receipt = remeasure_artifact(bundle_dir, &handoff.authentication.archive_receipt)?;
    for row in &mut handoff.rows {
        row.artifact = remeasure_artifact(bundle_dir, &row.artifact)?;
        row.cairn_policy = remeasure_artifact(bundle_dir, &row.cairn_policy)?;
    }
    let validation_result = crunch_release_core::validate_cairn_release_evidence_handoff(&handoff);
    if !validation_result.valid {
        return Err(RunError::Internal(format!(
            "bundle-local Cairn handoff bytes failed validation: {}",
            validation_result.diagnostics.join("; ")
        )));
    }
    debug_assert!(!handoff.rows.is_empty());
    debug_assert_eq!(handoff.rows.len(), receipt.handoff.rows.len());
    Ok(handoff)
}

fn preflight_handoff(
    descriptor: &CairnHandoffDescriptor,
    descriptor_parent: &Path,
    bundle_dir: &Path,
) -> Result<CairnReleaseEvidenceHandoff, RunError> {
    debug_assert!(!CAIRN_HANDOFF_BUNDLE_DIRECTORY.is_empty());
    debug_assert!(!CAIRN_HANDOFF_AUTHENTICATION_DIRECTORY.is_empty());
    let authentication_source =
        resolve_descriptor_path(descriptor_parent, &descriptor.authentication.archive_receipt_path);
    reject_bundle_local_source(&authentication_source, bundle_dir)?;
    let authentication_relative_path = PathBuf::from(CAIRN_HANDOFF_BUNDLE_DIRECTORY)
        .join(CAIRN_HANDOFF_AUTHENTICATION_DIRECTORY)
        .join(CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_FILE);
    let authentication = CairnHandoffAuthenticationDependency {
        schema: descriptor.authentication.schema.clone(),
        change_name: descriptor.authentication.change_name.clone(),
        cairn_revision: descriptor.authentication.cairn_revision.clone(),
        archive_manifest_blake3: descriptor.authentication.archive_manifest_blake3.clone(),
        archive_mutation_receipt_blake3: descriptor.authentication.archive_mutation_receipt_blake3.clone(),
        archive_receipt: measure_planned_artifact(
            &authentication_source,
            &authentication_relative_path,
            descriptor.authentication.archive_receipt_digest_blake3.clone(),
        )?,
    };
    let mut rows = Vec::with_capacity(descriptor.rows.len());
    for (index, row) in descriptor.rows.iter().enumerate() {
        let index_u32 = u32::try_from(index)
            .map_err(|_| RunError::Internal("Cairn handoff row index overflowed u32".to_string()))?;
        let artifact_source = resolve_descriptor_path(descriptor_parent, &row.artifact_path);
        let policy_source = resolve_descriptor_path(descriptor_parent, &row.cairn_policy_path);
        reject_bundle_local_source(&artifact_source, bundle_dir)?;
        reject_bundle_local_source(&policy_source, bundle_dir)?;
        let row_directory = PathBuf::from(CAIRN_HANDOFF_BUNDLE_DIRECTORY).join(index_u32.to_string());
        rows.push(CairnReleaseEvidenceRow {
            artifact_id: row.artifact_id.clone(),
            role: row.role.clone(),
            schema_id: row.schema_id.clone(),
            artifact: measure_planned_artifact(
                &artifact_source,
                &row_directory.join(CAIRN_HANDOFF_ARTIFACT_FILE),
                row.artifact_digest_blake3.clone(),
            )?,
            cairn_policy: measure_planned_artifact(
                &policy_source,
                &row_directory.join(CAIRN_HANDOFF_POLICY_FILE),
                row.cairn_policy_digest_blake3.clone(),
            )?,
            release_readiness_id: row.release_readiness_id.clone(),
            covers: row.covers.clone(),
            non_claims: row.non_claims.clone(),
        });
    }
    let handoff = CairnReleaseEvidenceHandoff { authentication, rows };
    let validation_result = crunch_release_core::validate_cairn_release_evidence_handoff(&handoff);
    if !validation_result.valid {
        return Err(RunError::Internal(format!(
            "Cairn handoff descriptor failed preflight validation: {}",
            validation_result.diagnostics.join("; ")
        )));
    }
    debug_assert_eq!(handoff.rows.len(), descriptor.rows.len());
    debug_assert!(validation_result.valid);
    Ok(handoff)
}

fn prepare_authentication(
    authentication: CairnHandoffDescriptorAuthentication,
    descriptor_parent: &Path,
    bundle_dir: &Path,
) -> Result<CairnHandoffAuthenticationDependency, RunError> {
    debug_assert!(!CAIRN_HANDOFF_AUTHENTICATION_DIRECTORY.is_empty());
    debug_assert!(!CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_FILE.is_empty());
    let receipt_source = resolve_descriptor_path(descriptor_parent, &authentication.archive_receipt_path);
    reject_bundle_local_source(&receipt_source, bundle_dir)?;
    let receipt_relative_path = PathBuf::from(CAIRN_HANDOFF_BUNDLE_DIRECTORY)
        .join(CAIRN_HANDOFF_AUTHENTICATION_DIRECTORY)
        .join(CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_FILE);
    let archive_receipt = copy_and_measure(
        &receipt_source,
        bundle_dir,
        &receipt_relative_path,
        authentication.archive_receipt_digest_blake3,
    )?;
    Ok(CairnHandoffAuthenticationDependency {
        schema: authentication.schema,
        change_name: authentication.change_name,
        cairn_revision: authentication.cairn_revision,
        archive_manifest_blake3: authentication.archive_manifest_blake3,
        archive_mutation_receipt_blake3: authentication.archive_mutation_receipt_blake3,
        archive_receipt,
    })
}

fn prepare_row(
    row: CairnHandoffDescriptorRow,
    index: u32,
    descriptor_parent: &Path,
    bundle_dir: &Path,
) -> Result<CairnReleaseEvidenceRow, RunError> {
    debug_assert!(!CAIRN_HANDOFF_ARTIFACT_FILE.is_empty());
    debug_assert!(!CAIRN_HANDOFF_POLICY_FILE.is_empty());
    let artifact_source = resolve_descriptor_path(descriptor_parent, &row.artifact_path);
    let policy_source = resolve_descriptor_path(descriptor_parent, &row.cairn_policy_path);
    reject_bundle_local_source(&artifact_source, bundle_dir)?;
    reject_bundle_local_source(&policy_source, bundle_dir)?;
    let row_directory = PathBuf::from(CAIRN_HANDOFF_BUNDLE_DIRECTORY).join(index.to_string());
    let artifact = copy_and_measure(
        &artifact_source,
        bundle_dir,
        &row_directory.join(CAIRN_HANDOFF_ARTIFACT_FILE),
        row.artifact_digest_blake3,
    )?;
    let cairn_policy = copy_and_measure(
        &policy_source,
        bundle_dir,
        &row_directory.join(CAIRN_HANDOFF_POLICY_FILE),
        row.cairn_policy_digest_blake3,
    )?;
    Ok(CairnReleaseEvidenceRow {
        artifact_id: row.artifact_id,
        role: row.role,
        schema_id: row.schema_id,
        artifact,
        cairn_policy,
        release_readiness_id: row.release_readiness_id,
        covers: row.covers,
        non_claims: row.non_claims,
    })
}

fn copy_and_measure(
    source: &Path,
    bundle_dir: &Path,
    relative_path: &Path,
    declared_digest_blake3: String,
) -> Result<CairnMeasuredArtifact, RunError> {
    let bytes = read_bounded_file(
        source,
        MAX_CAIRN_HANDOFF_ARTIFACT_BYTES,
        "Cairn handoff artifact",
        ReleaseRootKind::BuildArtifact,
    )?;
    if bytes.is_empty() {
        return Err(RunError::Internal(format!("Cairn handoff artifact must be non-empty: {}", source.display())));
    }
    let measured_digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    let size_bytes = u64::try_from(bytes.len())
        .map_err(|_| RunError::Internal("Cairn handoff artifact length overflowed u64".to_string()))?;
    let bundled_relative_path = write_measured_bundle_bytes(bundle_dir, relative_path, &bytes)?;
    debug_assert!(size_bytes <= MAX_CAIRN_HANDOFF_ARTIFACT_BYTES);
    debug_assert_eq!(measured_digest_blake3, blake3::hash(&bytes).to_hex().to_string());
    Ok(CairnMeasuredArtifact {
        relative_path: bundled_relative_path,
        size_bytes,
        declared_digest_blake3,
        measured_digest_blake3,
    })
}

fn measure_planned_artifact(
    source: &Path,
    relative_path: &Path,
    declared_digest_blake3: String,
) -> Result<CairnMeasuredArtifact, RunError> {
    let bytes = read_bounded_file(
        source,
        MAX_CAIRN_HANDOFF_ARTIFACT_BYTES,
        "Cairn handoff artifact",
        ReleaseRootKind::BuildArtifact,
    )?;
    if bytes.is_empty() {
        return Err(RunError::Internal(format!("Cairn handoff artifact must be non-empty: {}", source.display())));
    }
    let relative_path = relative_path
        .to_str()
        .ok_or_else(|| RunError::Internal("Cairn handoff bundle path is not UTF-8".to_string()))?
        .replace(std::path::MAIN_SEPARATOR, "/");
    let size_bytes = u64::try_from(bytes.len())
        .map_err(|_| RunError::Internal("Cairn handoff artifact length overflowed u64".to_string()))?;
    let measured_digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    if declared_digest_blake3 != measured_digest_blake3 {
        return Err(RunError::Internal(format!(
            "Cairn handoff artifact declared digest does not match measured bytes: {}",
            source.display()
        )));
    }
    debug_assert!(!relative_path.is_empty());
    debug_assert!(size_bytes <= MAX_CAIRN_HANDOFF_ARTIFACT_BYTES);
    Ok(CairnMeasuredArtifact {
        relative_path,
        size_bytes,
        declared_digest_blake3,
        measured_digest_blake3,
    })
}

fn write_measured_bundle_bytes(bundle_dir: &Path, relative_path: &Path, bytes: &[u8]) -> Result<String, RunError> {
    let destination = bundle_dir.join(relative_path);
    let parent = destination.parent().ok_or_else(|| {
        RunError::Internal(format!("Cairn handoff bundle path has no parent: {}", destination.display()))
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        RunError::Internal(format!("creating Cairn handoff bundle directory {}: {error}", parent.display()))
    })?;
    fs::write(&destination, bytes).map_err(|error| {
        RunError::Internal(format!("writing measured Cairn handoff bytes {}: {error}", destination.display()))
    })?;
    let written = read_bounded_file(
        &destination,
        MAX_CAIRN_HANDOFF_ARTIFACT_BYTES,
        "bundled Cairn handoff artifact",
        ReleaseRootKind::ReleaseEvidence,
    )?;
    if written != bytes {
        return Err(RunError::Internal(format!(
            "bundled Cairn handoff bytes changed while being written: {}",
            destination.display()
        )));
    }
    let relative_path = relative_path
        .to_str()
        .ok_or_else(|| RunError::Internal("Cairn handoff bundle path is not UTF-8".to_string()))?
        .replace(std::path::MAIN_SEPARATOR, "/");
    debug_assert!(!relative_path.is_empty());
    debug_assert!(!Path::new(&relative_path).is_absolute());
    Ok(relative_path)
}

fn remeasure_artifact(bundle_dir: &Path, artifact: &CairnMeasuredArtifact) -> Result<CairnMeasuredArtifact, RunError> {
    let path = bundle_dir.join(&artifact.relative_path);
    let bytes = read_bounded_file(
        &path,
        MAX_CAIRN_HANDOFF_ARTIFACT_BYTES,
        "bundled Cairn handoff artifact",
        ReleaseRootKind::ReleaseEvidence,
    )?;
    let size_bytes = u64::try_from(bytes.len())
        .map_err(|_| RunError::Internal("bundled Cairn artifact length overflowed u64".to_string()))?;
    debug_assert!(size_bytes <= MAX_CAIRN_HANDOFF_ARTIFACT_BYTES);
    debug_assert!(!artifact.relative_path.is_empty());
    Ok(CairnMeasuredArtifact {
        relative_path: artifact.relative_path.clone(),
        size_bytes,
        declared_digest_blake3: artifact.declared_digest_blake3.clone(),
        measured_digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn read_bounded_file(
    path: &Path,
    maximum_bytes: u64,
    label: &str,
    root_kind: ReleaseRootKind,
) -> Result<Vec<u8>, RunError> {
    let absolute = std::path::absolute(path)
        .map_err(|error| RunError::Internal(format!("resolving {label} path {}: {error}", path.display())))?;
    let parent = absolute
        .parent()
        .ok_or_else(|| RunError::Internal(format!("{label} path has no parent: {}", absolute.display())))?;
    let file_name = absolute
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| RunError::Internal(format!("{label} file name is not UTF-8: {}", absolute.display())))?;
    let relative_path = ValidatedReleasePath::new(file_name).map_err(|error| {
        RunError::Internal(format!("validating {label} file name {}: {error:?}", absolute.display()))
    })?;
    let root = ReleaseCapabilityRoot::open_ambient_nofollow(root_kind, parent).map_err(|error| {
        RunError::Internal(format!("opening no-follow {label} parent {}: {error}", parent.display()))
    })?;
    let file = root
        .open_file_read_nofollow(&relative_path)
        .map_err(|error| RunError::Internal(format!("opening no-follow {label} {}: {error}", absolute.display())))?;
    let metadata = file.metadata().map_err(|error| {
        RunError::Internal(format!("reading opened {label} metadata {}: {error}", absolute.display()))
    })?;
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("{label} must be a regular non-symlink file: {}", absolute.display())));
    }
    if metadata.len() > maximum_bytes {
        return Err(RunError::Internal(format!(
            "{label} {} is {} bytes, limit is {maximum_bytes}",
            absolute.display(),
            metadata.len()
        )));
    }
    let capacity_bytes = usize::try_from(metadata.len())
        .map_err(|_| RunError::Internal(format!("{label} byte length does not fit usize: {}", absolute.display())))?;
    let read_limit_bytes = maximum_bytes
        .checked_add(1)
        .ok_or_else(|| RunError::Internal(format!("{label} read limit overflowed u64")))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    file.take(read_limit_bytes)
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading bounded {label} {}: {error}", absolute.display())))?;
    let observed_bytes = u64::try_from(bytes.len())
        .map_err(|_| RunError::Internal(format!("{label} observed byte length overflowed u64")))?;
    if observed_bytes > maximum_bytes {
        return Err(RunError::Internal(format!(
            "{label} {} grew beyond the {maximum_bytes}-byte limit while being read",
            absolute.display()
        )));
    }
    debug_assert!(observed_bytes <= maximum_bytes);
    debug_assert!(bytes.capacity() >= bytes.len());
    Ok(bytes)
}

fn validate_descriptor_header(descriptor: &CairnHandoffDescriptor) -> Result<(), RunError> {
    let maximum_rows_count = maximum_handoff_rows_count()?;
    let maximum_text_size_bytes = usize::try_from(MAX_CAIRN_HANDOFF_TEXT_BYTES_COUNT)
        .map_err(|_| RunError::Internal("Cairn handoff text limit does not fit usize".to_string()))?;
    if descriptor.schema != CAIRN_HANDOFF_INPUT_SCHEMA {
        return Err(RunError::Internal(format!("unsupported Cairn handoff descriptor schema: {}", descriptor.schema)));
    }
    if descriptor.rows.is_empty() || descriptor.rows.len() > maximum_rows_count {
        return Err(RunError::Internal(format!(
            "Cairn handoff descriptor row count must be between 1 and {MAX_CAIRN_HANDOFF_ROWS_COUNT}"
        )));
    }
    for row in &descriptor.rows {
        if row.artifact_id.len() > maximum_text_size_bytes {
            return Err(RunError::Internal("Cairn handoff artifact_id is oversized".to_string()));
        }
    }
    Ok(())
}

fn maximum_handoff_rows_count() -> Result<usize, RunError> {
    usize::try_from(MAX_CAIRN_HANDOFF_ROWS_COUNT)
        .map_err(|_| RunError::Internal("Cairn handoff row limit does not fit usize".to_string()))
}

fn resolve_descriptor_path(parent: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        parent.join(path)
    }
}

fn reject_bundle_local_source(source: &Path, bundle_dir: &Path) -> Result<(), RunError> {
    let bundle_dir = bundle_dir.canonicalize().unwrap_or_else(|_| bundle_dir.to_path_buf());
    let source = source
        .canonicalize()
        .map_err(|error| RunError::Internal(format!("resolving Cairn handoff source {}: {error}", source.display())))?;
    if source.starts_with(&bundle_dir) {
        return Err(RunError::Internal(format!(
            "Cairn handoff source cannot point into the bundle being assembled: {}",
            source.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACT_BYTES: &[u8] = b"receipt";
    const POLICY_BYTES: &[u8] = b"policy";
    const AUTHENTICATION_RECEIPT_BYTES: &[u8] =
        include_bytes!("../cairn-policy/evidence/cairn-authenticated-inputs-archive-receipt.json");

    fn write_descriptor(root: &Path, declared_artifact_digest: &str) -> PathBuf {
        fs::write(root.join("receipt.json"), ARTIFACT_BYTES).unwrap();
        fs::write(root.join("policy.ncl"), POLICY_BYTES).unwrap();
        fs::write(root.join("cairn-authentication.json"), AUTHENTICATION_RECEIPT_BYTES).unwrap();
        let policy_digest = blake3::hash(POLICY_BYTES).to_hex().to_string();
        let descriptor = serde_json::json!({
            "schema": CAIRN_HANDOFF_INPUT_SCHEMA,
            "authentication": {
                "schema": crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_SCHEMA,
                "change_name": crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_CHANGE,
                "cairn_revision": crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_CAIRN_REVISION,
                "archive_manifest_blake3": crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MANIFEST_BLAKE3,
                "archive_mutation_receipt_blake3": crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MUTATION_RECEIPT_BLAKE3,
                "archive_receipt_path": "cairn-authentication.json",
                "archive_receipt_digest_blake3": crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_BLAKE3
            },
            "rows": [{
                "artifact_id": "readiness",
                "role": "cairn-release-readiness-receipt",
                "schema_id": "cairn.release-readiness.v1",
                "artifact_path": "receipt.json",
                "artifact_digest_blake3": declared_artifact_digest,
                "cairn_policy_path": "policy.ncl",
                "cairn_policy_digest_blake3": policy_digest,
                "release_readiness_id": "release-1",
                "covers": ["mantle.release"],
                "non_claims": ["not release correctness"]
            }]
        });
        let path = root.join("handoff.json");
        fs::write(&path, serde_json::to_vec(&descriptor).unwrap()).unwrap();
        path
    }

    #[test]
    fn shell_measures_and_bundles_declared_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = temp.path().join("bundle");
        fs::create_dir(&bundle).unwrap();
        let digest = blake3::hash(ARTIFACT_BYTES).to_hex().to_string();
        let descriptor = write_descriptor(temp.path(), &digest);
        let handoff = prepare_cairn_handoff_for_bundle(&descriptor, &bundle).unwrap();
        assert_eq!(handoff.rows.len(), 1);
        assert_eq!(handoff.rows[0].artifact.measured_digest_blake3, digest);
    }

    #[test]
    fn shell_rejects_tampered_authentication_dependency() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = temp.path().join("bundle");
        fs::create_dir(&bundle).unwrap();
        let digest = blake3::hash(ARTIFACT_BYTES).to_hex().to_string();
        let descriptor = write_descriptor(temp.path(), &digest);
        fs::write(temp.path().join("cairn-authentication.json"), b"tampered").unwrap();
        let error = prepare_cairn_handoff_for_bundle(&descriptor, &bundle).unwrap_err();
        assert!(error.to_string().contains("declared digest"));
    }

    #[test]
    fn shell_rejects_fabricated_declared_digest() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = temp.path().join("bundle");
        fs::create_dir(&bundle).unwrap();
        let fabricated = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let descriptor = write_descriptor(temp.path(), fabricated);
        let error = prepare_cairn_handoff_for_bundle(&descriptor, &bundle).unwrap_err();
        assert!(error.to_string().contains("declared digest"));
        assert!(!error.to_string().contains("release correctness proven"));
    }

    #[cfg(unix)]
    #[test]
    fn shell_rejects_symlinked_descriptor_and_artifact_inputs() {
        use std::os::unix::fs::symlink;

        let descriptor_case = tempfile::tempdir().unwrap();
        let descriptor_bundle = descriptor_case.path().join("bundle");
        fs::create_dir(&descriptor_bundle).unwrap();
        let digest = blake3::hash(ARTIFACT_BYTES).to_hex().to_string();
        let descriptor_target = write_descriptor(descriptor_case.path(), &digest);
        let descriptor_link = descriptor_case.path().join("handoff-link.json");
        symlink(&descriptor_target, &descriptor_link).unwrap();
        let descriptor_error = prepare_cairn_handoff_for_bundle(&descriptor_link, &descriptor_bundle).unwrap_err();

        let artifact_case = tempfile::tempdir().unwrap();
        let artifact_bundle = artifact_case.path().join("bundle");
        fs::create_dir(&artifact_bundle).unwrap();
        let descriptor = write_descriptor(artifact_case.path(), &digest);
        let artifact_path = artifact_case.path().join("receipt.json");
        let artifact_target = artifact_case.path().join("receipt-target.json");
        fs::rename(&artifact_path, &artifact_target).unwrap();
        symlink(&artifact_target, &artifact_path).unwrap();
        let artifact_error = prepare_cairn_handoff_for_bundle(&descriptor, &artifact_bundle).unwrap_err();

        assert!(descriptor_error.to_string().contains("no-follow"));
        assert!(artifact_error.to_string().contains("no-follow"));
    }

    #[cfg(unix)]
    #[test]
    fn shell_rejects_symlinked_input_parent() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let actual_parent = temp.path().join("actual");
        let linked_parent = temp.path().join("linked");
        let bundle = temp.path().join("bundle");
        fs::create_dir(&actual_parent).unwrap();
        fs::create_dir(&bundle).unwrap();
        let digest = blake3::hash(ARTIFACT_BYTES).to_hex().to_string();
        write_descriptor(&actual_parent, &digest);
        symlink(&actual_parent, &linked_parent).unwrap();

        let error = prepare_cairn_handoff_for_bundle(&linked_parent.join("handoff.json"), &bundle).unwrap_err();
        assert!(error.to_string().contains("no-follow"));
        assert!(!bundle.join(CAIRN_HANDOFF_BUNDLE_DIRECTORY).exists());
    }

    #[test]
    fn shell_rejects_empty_artifact() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = temp.path().join("bundle");
        fs::create_dir(&bundle).unwrap();
        let empty_digest = blake3::hash(b"").to_hex().to_string();
        let descriptor = write_descriptor(temp.path(), &empty_digest);
        fs::write(temp.path().join("receipt.json"), b"").unwrap();

        let error = prepare_cairn_handoff_for_bundle(&descriptor, &bundle).unwrap_err();
        assert!(error.to_string().contains("non-empty"));
        assert!(!bundle.join(CAIRN_HANDOFF_BUNDLE_DIRECTORY).exists());
    }

    #[test]
    fn shell_rejects_descriptor_over_byte_limit() {
        let temp = tempfile::tempdir().unwrap();
        let bundle = temp.path().join("bundle");
        let descriptor = temp.path().join("oversized.json");
        fs::create_dir(&bundle).unwrap();
        let file = fs::File::create(&descriptor).unwrap();
        file.set_len(MAX_CAIRN_HANDOFF_DESCRIPTOR_BYTES.checked_add(1).unwrap()).unwrap();

        let error = prepare_cairn_handoff_for_bundle(&descriptor, &bundle).unwrap_err();
        assert!(error.to_string().contains("limit"));
        assert!(!bundle.join(CAIRN_HANDOFF_BUNDLE_DIRECTORY).exists());
    }

    #[test]
    fn checked_fixtures_cover_positive_and_negative_handoffs() {
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cairn-handoff");
        let positive_bundle = tempfile::tempdir().unwrap();
        let handoff =
            prepare_cairn_handoff_for_bundle(&fixture_root.join("positive/handoff.json"), positive_bundle.path())
                .unwrap();
        assert_eq!(handoff.rows.len(), 1);

        for fixture in ["fabricated-digest.json", "wrong-role-schema.json", "empty-bypass.json"] {
            let bundle = tempfile::tempdir().unwrap();
            let error = prepare_cairn_handoff_for_bundle(&fixture_root.join("negative").join(fixture), bundle.path())
                .unwrap_err();
            assert!(!error.to_string().is_empty(), "fixture {fixture} must fail closed");
        }
    }
}
