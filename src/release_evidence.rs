use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[cfg(test)]
pub(crate) use crunch_release_core::BLAKE3_HEX_LENGTH_CHARS as BLAKE3_HEX_LEN;
pub(crate) use crunch_release_core::BundledArtifact;
pub(crate) use crunch_release_core::BundledArtifactKind;
pub(crate) use crunch_release_core::CLAIM_SCOPE_PACKAGED_INTEGRITY;
pub(crate) use crunch_release_core::DEFAULT_PROOF_WORKFLOW_COMMAND;
pub(crate) use crunch_release_core::DEFAULT_PROOF_WORKFLOW_VERSION;
#[cfg(test)]
pub(crate) use crunch_release_core::FULL_SELF_HOSTING_PROOF_SCHEMA;
pub(crate) use crunch_release_core::RELEASE_EVIDENCE_SCHEMA;
use crunch_release_core::ReleaseEvidenceError;
pub(crate) use crunch_release_core::ReleaseEvidenceManifest;
pub(crate) use crunch_release_core::ReleaseProofLinkage;
use crunch_release_core::ReleaseReproducibilityReport;
use crunch_release_core::ReleaseReproducibilityReportLinkage;
pub(crate) use crunch_release_core::ReleaseWorkflowIdentity;
use crunch_release_core::canonical_release_evidence_manifest;
use crunch_release_core::extract_full_self_hosting_proof_identity_fields;
use crunch_release_core::release_reproducibility_report_canonical_bytes;
use crunch_release_core::validate_bundled_artifact_record;
use crunch_release_core::validate_release_reproducibility_report_artifact_names;
use crunch_release_core::validate_release_reproducibility_report_linkage;

use crate::errors::RunError;

const PROOF_INVENTORY_RELATIVE_PATH: &str = "stage0-prerequisites/inventory.md";
const MAX_BINARY_ARTIFACTS: u32 = 16;
const MAX_BUNDLE_TREE_ENTRIES: u32 = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FullSelfHostingProofIdentity {
    pub schema: String,
    pub proof_mode: String,
    pub selected_provider_kind: String,
    pub staged_source: String,
    pub stage2_binary_digest_blake3: String,
    pub prerequisite_inventory_digest_blake3: String,
    pub proof_manifest_digest_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ReleaseBundleCreateRequest {
    pub release_id: String,
    pub bundle_dir: PathBuf,
    pub source_archive_path: PathBuf,
    pub binary_paths: Vec<PathBuf>,
    pub proof_bundle_dir: PathBuf,
    pub workflow_command: String,
    pub workflow_version: String,
    pub reproducibility_report_path: Option<PathBuf>,
}

impl ReleaseBundleCreateRequest {
    #[cfg(test)]
    pub(crate) fn with_defaults(
        release_id: String,
        bundle_dir: PathBuf,
        source_archive_path: PathBuf,
        binary_paths: Vec<PathBuf>,
        proof_bundle_dir: PathBuf,
    ) -> Self {
        Self {
            release_id,
            bundle_dir,
            source_archive_path,
            binary_paths,
            proof_bundle_dir,
            workflow_command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
            workflow_version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            reproducibility_report_path: None,
        }
    }
}

fn core_error_to_run_error(err: ReleaseEvidenceError) -> RunError {
    RunError::Internal(err.to_string())
}

pub(crate) fn create_release_evidence_bundle(
    request: &ReleaseBundleCreateRequest,
) -> Result<ReleaseEvidenceManifest, RunError> {
    validate_create_request(request)?;
    prepare_output_bundle_dir(&request.bundle_dir)?;

    let proof_identity = load_full_self_hosting_proof_identity(&request.proof_bundle_dir)?;
    let source_archive = copy_file_into_bundle(
        &request.source_archive_path,
        &request.bundle_dir,
        &bundle_file_destination("source", &request.source_archive_path, 0)?,
    )?;
    let binaries = copy_binary_set_into_bundle(&request.binary_paths, &request.bundle_dir)?;
    let proof_bundle =
        copy_directory_into_bundle(&request.proof_bundle_dir, &request.bundle_dir, Path::new("proof/self-hosting"))?;
    let inventory_path = request.proof_bundle_dir.join(PROOF_INVENTORY_RELATIVE_PATH);
    let prerequisite_inventory =
        copy_file_into_bundle(&inventory_path, &request.bundle_dir, Path::new("proof/inventory.md"))?;
    let reproducibility_report =
        copy_optional_reproducibility_report(request, &source_archive, &binaries, &proof_bundle)?;
    let source_archive_digest_blake3 = source_archive.digest_blake3.clone();

    let manifest = ReleaseEvidenceManifest {
        schema: RELEASE_EVIDENCE_SCHEMA.to_string(),
        release_id: request.release_id.clone(),
        claim_scope: CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
        workflow: ReleaseWorkflowIdentity {
            command: request.workflow_command.clone(),
            version: request.workflow_version.clone(),
        },
        source_archive,
        binaries,
        proof_bundle,
        prerequisite_inventory: prerequisite_inventory.clone(),
        reproducibility_report,
        independent_agreement_report: None,
        proof_linkage: ReleaseProofLinkage {
            release_id: request.release_id.clone(),
            source_archive_digest_blake3,
            proof_bundle_schema: proof_identity.schema,
            proof_mode: proof_identity.proof_mode,
            selected_provider_kind: proof_identity.selected_provider_kind,
            staged_source: proof_identity.staged_source,
            stage2_binary_digest_blake3: proof_identity.stage2_binary_digest_blake3,
            prerequisite_inventory_digest_blake3: proof_identity.prerequisite_inventory_digest_blake3,
            proof_manifest_digest_blake3: proof_identity.proof_manifest_digest_blake3,
        },
    };
    write_manifest_file(&request.bundle_dir, &manifest)?;
    Ok(manifest)
}

pub(crate) fn verify_release_evidence_bundle(bundle_dir: &Path) -> Result<ReleaseEvidenceManifest, RunError> {
    let manifest_path = bundle_dir.join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", manifest_path.display())))?;
    let manifest: ReleaseEvidenceManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", manifest_path.display())))?;
    let expected_canonical = canonical_release_evidence_manifest(manifest.clone()).map_err(core_error_to_run_error)?;
    if manifest_bytes != expected_canonical {
        return Err(RunError::Internal("release evidence manifest.json is not canonical compact JSON".to_string()));
    }

    verify_manifest_artifacts(&manifest, bundle_dir)?;
    verify_manifest_proof_linkage(&manifest, bundle_dir)?;
    Ok(manifest)
}

pub(crate) fn load_full_self_hosting_proof_identity(
    bundle_dir: &Path,
) -> Result<FullSelfHostingProofIdentity, RunError> {
    if !bundle_dir.exists() {
        return Err(RunError::Internal(format!("proof bundle directory does not exist: {}", bundle_dir.display())));
    }
    if !bundle_dir.is_dir() {
        return Err(RunError::Internal(format!("proof bundle path is not a directory: {}", bundle_dir.display())));
    }

    let manifest_path = bundle_dir.join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", manifest_path.display())))?;
    let proof_manifest_digest_blake3 = blake3::hash(&manifest_bytes).to_hex().to_string();
    let manifest = extract_full_self_hosting_proof_identity_fields(manifest_bytes).map_err(core_error_to_run_error)?;
    Ok(FullSelfHostingProofIdentity {
        schema: manifest.schema,
        proof_mode: manifest.proof_mode,
        selected_provider_kind: manifest.selected_provider_kind,
        staged_source: manifest.staged_source,
        stage2_binary_digest_blake3: manifest.stage2_binary_digest_blake3,
        prerequisite_inventory_digest_blake3: manifest.prerequisite_inventory_digest_blake3,
        proof_manifest_digest_blake3,
    })
}

fn validate_create_request(request: &ReleaseBundleCreateRequest) -> Result<(), RunError> {
    if request.release_id.trim().is_empty() {
        return Err(RunError::Internal("release evidence release_id must not be empty".to_string()));
    }
    if request.workflow_command.trim().is_empty() {
        return Err(RunError::Internal("release evidence workflow command must not be empty".to_string()));
    }
    if request.workflow_version.trim().is_empty() {
        return Err(RunError::Internal("release evidence workflow version must not be empty".to_string()));
    }
    if !request.source_archive_path.is_file() {
        return Err(RunError::Internal(format!(
            "release evidence source archive is missing: {}",
            request.source_archive_path.display()
        )));
    }
    let binary_count: u32 = request
        .binary_paths
        .len()
        .try_into()
        .map_err(|_| RunError::Internal("release evidence binary count overflowed u32".to_string()))?;
    if binary_count == 0 {
        return Err(RunError::Internal("release evidence requires at least one --binary input".to_string()));
    }
    if binary_count > MAX_BINARY_ARTIFACTS {
        return Err(RunError::Internal(format!(
            "release evidence received {binary_count} binaries, limit is {MAX_BINARY_ARTIFACTS}"
        )));
    }
    for binary_path in &request.binary_paths {
        if !binary_path.is_file() {
            return Err(RunError::Internal(format!("release evidence binary is missing: {}", binary_path.display())));
        }
    }
    if let Some(report_path) = &request.reproducibility_report_path {
        if !report_path.is_file() {
            return Err(RunError::Internal(format!(
                "release evidence reproducibility report is missing: {}",
                report_path.display()
            )));
        }
    }
    Ok(())
}

fn prepare_output_bundle_dir(bundle_dir: &Path) -> Result<(), RunError> {
    if bundle_dir.exists() {
        if !bundle_dir.is_dir() {
            return Err(RunError::Internal(format!(
                "release evidence bundle path is not a directory: {}",
                bundle_dir.display()
            )));
        }
        let mut existing_entries = std::fs::read_dir(bundle_dir)
            .map_err(|err| RunError::Internal(format!("reading {}: {err}", bundle_dir.display())))?;
        if existing_entries.next().is_some() {
            return Err(RunError::Internal(format!(
                "release evidence bundle directory must be empty: {}",
                bundle_dir.display()
            )));
        }
        return Ok(());
    }
    std::fs::create_dir_all(bundle_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", bundle_dir.display())))
}

fn bundle_file_destination(prefix: &str, source_path: &Path, index_u32: u32) -> Result<PathBuf, RunError> {
    let file_name = source_path.file_name().ok_or_else(|| {
        RunError::Internal(format!("release evidence input has no file name: {}", source_path.display()))
    })?;
    let mut relative = PathBuf::from(prefix);
    if index_u32 == 0 {
        relative.push(file_name);
    } else {
        relative.push(format!("{index_u32:02}-{}", file_name.to_string_lossy()));
    }
    Ok(relative)
}

fn copy_binary_set_into_bundle(binary_paths: &[PathBuf], bundle_dir: &Path) -> Result<Vec<BundledArtifact>, RunError> {
    let mut bundled = Vec::with_capacity(binary_paths.len());
    for (index_usize, binary_path) in binary_paths.iter().enumerate() {
        let index_u32: u32 = index_usize
            .try_into()
            .map_err(|_| RunError::Internal("release evidence binary index overflowed u32".to_string()))?;
        let relative = bundle_file_destination("binaries", binary_path, index_u32.saturating_add(1))?;
        bundled.push(copy_file_into_bundle(binary_path, bundle_dir, &relative)?);
    }
    assert!(!bundled.is_empty(), "binary bundle copy must emit at least one artifact");
    Ok(bundled)
}

fn copy_optional_reproducibility_report(
    request: &ReleaseBundleCreateRequest,
    source_archive: &BundledArtifact,
    binaries: &[BundledArtifact],
    proof_bundle: &BundledArtifact,
) -> Result<Option<BundledArtifact>, RunError> {
    let Some(report_path) = &request.reproducibility_report_path else {
        return Ok(None);
    };
    validate_reproducibility_report_for_bundle(request, report_path, source_archive, binaries, proof_bundle)?;
    let relative = Path::new("reproducibility").join("reproducibility-report.json");
    copy_file_into_bundle(report_path, &request.bundle_dir, &relative).map(Some)
}

fn validate_reproducibility_report_for_bundle(
    request: &ReleaseBundleCreateRequest,
    report_path: &Path,
    source_archive: &BundledArtifact,
    binaries: &[BundledArtifact],
    proof_bundle: &BundledArtifact,
) -> Result<(), RunError> {
    let report_bytes = std::fs::read(report_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", report_path.display())))?;
    let report: ReleaseReproducibilityReport = serde_json::from_slice(&report_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", report_path.display())))?;
    let canonical_bytes =
        release_reproducibility_report_canonical_bytes(report.clone()).map_err(core_error_to_run_error)?;
    if report_bytes != canonical_bytes {
        return Err(RunError::Internal(
            "release evidence reproducibility report is not canonical compact JSON".to_string(),
        ));
    }
    let expected = ReleaseReproducibilityReportLinkage {
        release_id: request.release_id.clone(),
        source_archive_digest_blake3: source_archive.digest_blake3.clone(),
        proof_bundle_digest_blake3: proof_bundle.digest_blake3.clone(),
    };
    let report = validate_release_reproducibility_report_linkage(report, expected).map_err(core_error_to_run_error)?;
    let expected_names = binaries.iter().map(|artifact| artifact.relative_path.clone()).collect::<Vec<_>>();
    validate_release_reproducibility_report_artifact_names(report, expected_names).map_err(core_error_to_run_error)?;
    Ok(())
}

fn copy_file_into_bundle(
    source_path: &Path,
    bundle_dir: &Path,
    relative_path: &Path,
) -> Result<BundledArtifact, RunError> {
    if !source_path.is_file() {
        return Err(RunError::Internal(format!("bundle input file missing: {}", source_path.display())));
    }
    let dest_path = bundle_dir.join(relative_path);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::copy(source_path, &dest_path).map_err(|err| {
        RunError::Internal(format!("copying {} to {}: {err}", source_path.display(), dest_path.display()))
    })?;
    build_artifact_record(&dest_path, relative_path, BundledArtifactKind::File)
}

fn copy_directory_into_bundle(
    source_dir: &Path,
    bundle_dir: &Path,
    relative_path: &Path,
) -> Result<BundledArtifact, RunError> {
    if !source_dir.is_dir() {
        return Err(RunError::Internal(format!("bundle input directory missing: {}", source_dir.display())));
    }
    let dest_dir = bundle_dir.join(relative_path);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", dest_dir.display())))?;
    copy_directory_tree(source_dir, &dest_dir)?;
    build_artifact_record(&dest_dir, relative_path, BundledArtifactKind::Directory)
}

pub(crate) fn copy_directory_tree(source_dir: &Path, dest_dir: &Path) -> Result<(), RunError> {
    let mut entries = Vec::new();
    collect_paths_sorted(source_dir, &mut entries)?;
    assert!(
        entries.len() <= usize::try_from(MAX_BUNDLE_TREE_ENTRIES).unwrap(),
        "bundle tree copy entry count exceeded limit"
    );
    for source_entry in &entries {
        let relative = source_entry.strip_prefix(source_dir).map_err(|err| {
            RunError::Internal(format!(
                "bundle tree strip_prefix {} from {}: {err}",
                source_entry.display(),
                source_dir.display()
            ))
        })?;
        let dest_entry = dest_dir.join(relative);
        copy_tree_entry(source_entry, &dest_entry)?;
    }
    Ok(())
}

fn copy_tree_entry(source_path: &Path, dest_path: &Path) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(source_path)
        .map_err(|err| RunError::Internal(format!("symlink_metadata {}: {err}", source_path.display())))?;
    if metadata.is_dir() {
        std::fs::create_dir_all(dest_path)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", dest_path.display())))?;
        return Ok(());
    }
    if metadata.is_file() {
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
        }
        std::fs::copy(source_path, dest_path).map_err(|err| {
            RunError::Internal(format!("copying {} to {}: {err}", source_path.display(), dest_path.display()))
        })?;
        return Ok(());
    }
    if metadata.file_type().is_symlink() {
        return copy_symlink_entry(source_path, dest_path);
    }
    Err(RunError::Internal(format!("unsupported bundle tree entry type: {}", source_path.display())))
}

#[cfg(unix)]
fn copy_symlink_entry(source_path: &Path, dest_path: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::symlink;

    let target = std::fs::read_link(source_path)
        .map_err(|err| RunError::Internal(format!("read_link {}: {err}", source_path.display())))?;
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    symlink(&target, dest_path).map_err(|err| {
        RunError::Internal(format!("creating symlink {} -> {}: {err}", dest_path.display(), target.display()))
    })
}

#[cfg(not(unix))]
fn copy_symlink_entry(source_path: &Path, _dest_path: &Path) -> Result<(), RunError> {
    Err(RunError::Internal(format!(
        "symlink bundle copy is only supported on Unix: {}",
        source_path.display()
    )))
}

fn build_artifact_record(
    path: &Path,
    relative_path: &Path,
    kind: BundledArtifactKind,
) -> Result<BundledArtifact, RunError> {
    let relative_path_string = path_to_forward_slash_string(relative_path)?;
    let (size_bytes, digest_blake3) = match kind {
        BundledArtifactKind::File => hash_file(path)?,
        BundledArtifactKind::Directory => hash_directory(path)?,
    };
    let artifact = BundledArtifact {
        kind,
        relative_path: relative_path_string,
        size_bytes,
        digest_blake3,
    };
    validate_bundled_artifact_record(artifact, "artifact".to_string()).map_err(core_error_to_run_error)
}

pub(crate) fn compute_path_blake3_digest(path: &Path) -> Result<String, RunError> {
    if path.is_file() {
        return hash_file(path).map(|(_size_bytes, digest_blake3)| digest_blake3);
    }
    if path.is_dir() {
        return hash_directory(path).map(|(_size_bytes, digest_blake3)| digest_blake3);
    }
    Err(RunError::Internal(format!("expected file or directory artifact: {}", path.display())))
}

fn hash_file(path: &Path) -> Result<(u64, String), RunError> {
    let metadata =
        std::fs::metadata(path).map_err(|err| RunError::Internal(format!("metadata {}: {err}", path.display())))?;
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("expected file artifact: {}", path.display())));
    }
    let mut file = File::open(path).map_err(|err| RunError::Internal(format!("open {}: {err}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| RunError::Internal(format!("read {}: {err}", path.display())))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok((metadata.len(), hasher.finalize().to_hex().to_string()))
}

fn hash_directory(path: &Path) -> Result<(u64, String), RunError> {
    if !path.is_dir() {
        return Err(RunError::Internal(format!("expected directory artifact: {}", path.display())));
    }
    let mut entries = Vec::new();
    collect_paths_sorted(path, &mut entries)?;
    let mut hasher = blake3::Hasher::new();
    let mut total_file_bytes: u64 = 0;
    for entry in &entries {
        total_file_bytes = total_file_bytes.saturating_add(hash_tree_entry(path, entry, &mut hasher)?);
    }
    if total_file_bytes == 0 {
        total_file_bytes = 1;
    }
    Ok((total_file_bytes, hasher.finalize().to_hex().to_string()))
}

fn collect_paths_sorted(root: &Path, entries: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let mut children = Vec::new();
    for child_result in
        std::fs::read_dir(root).map_err(|err| RunError::Internal(format!("read_dir {}: {err}", root.display())))?
    {
        let child =
            child_result.map_err(|err| RunError::Internal(format!("read_dir entry {}: {err}", root.display())))?;
        children.push(child.path());
    }
    children.sort();
    for child in children {
        let entry_count_u32: u32 = entries
            .len()
            .try_into()
            .map_err(|_| RunError::Internal("bundle tree entry count overflowed u32".to_string()))?;
        if entry_count_u32 >= MAX_BUNDLE_TREE_ENTRIES {
            return Err(RunError::Internal(format!("bundle tree exceeds {MAX_BUNDLE_TREE_ENTRIES} entries")));
        }
        entries.push(child.clone());
        if child.is_dir() {
            collect_paths_sorted(&child, entries)?;
        }
    }
    Ok(())
}

fn hash_tree_entry(root: &Path, entry: &Path, hasher: &mut blake3::Hasher) -> Result<u64, RunError> {
    let relative = entry.strip_prefix(root).map_err(|err| {
        RunError::Internal(format!("tree hash strip_prefix {} from {}: {err}", entry.display(), root.display()))
    })?;
    let metadata = std::fs::symlink_metadata(entry)
        .map_err(|err| RunError::Internal(format!("symlink_metadata {}: {err}", entry.display())))?;
    let relative_bytes = relative.as_os_str().as_encoded_bytes();
    hasher.update(&(relative_bytes.len() as u64).to_le_bytes());
    hasher.update(relative_bytes);
    hasher.update(&entry_mode_bits(&metadata).to_le_bytes());

    if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(entry)
            .map_err(|err| RunError::Internal(format!("read_link {}: {err}", entry.display())))?;
        let target_bytes = target.as_os_str().as_encoded_bytes();
        hasher.update(b"symlink\0");
        hasher.update(&(target_bytes.len() as u64).to_le_bytes());
        hasher.update(target_bytes);
        return Ok(0);
    }
    if metadata.is_dir() {
        hasher.update(b"dir\0");
        return Ok(0);
    }
    if metadata.is_file() {
        hasher.update(b"file\0");
        hasher.update(&metadata.len().to_le_bytes());
        let mut file =
            File::open(entry).map_err(|err| RunError::Internal(format!("open {}: {err}", entry.display())))?;
        let mut buffer = [0_u8; 8192];
        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|err| RunError::Internal(format!("read {}: {err}", entry.display())))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        return Ok(metadata.len());
    }
    Err(RunError::Internal(format!("unsupported tree hash entry type: {}", entry.display())))
}

#[cfg(unix)]
fn entry_mode_bits(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::MetadataExt;
    metadata.mode()
}

#[cfg(not(unix))]
fn entry_mode_bits(_metadata: &std::fs::Metadata) -> u32 {
    0
}

fn write_manifest_file(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest) -> Result<(), RunError> {
    let manifest_bytes = canonical_release_evidence_manifest(manifest.clone()).map_err(core_error_to_run_error)?;
    let manifest_path = bundle_dir.join("manifest.json");
    std::fs::write(&manifest_path, manifest_bytes)
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", manifest_path.display())))
}

fn verify_manifest_artifacts(manifest: &ReleaseEvidenceManifest, bundle_dir: &Path) -> Result<(), RunError> {
    verify_artifact_matches_bundle(&manifest.source_archive, bundle_dir, "source_archive")?;
    verify_artifact_matches_bundle(&manifest.proof_bundle, bundle_dir, "proof_bundle")?;
    verify_artifact_matches_bundle(&manifest.prerequisite_inventory, bundle_dir, "prerequisite_inventory")?;
    for (index_usize, artifact) in manifest.binaries.iter().enumerate() {
        let index_u32: u32 = index_usize
            .try_into()
            .map_err(|_| RunError::Internal("release evidence verify binary index overflowed u32".to_string()))?;
        verify_artifact_matches_bundle(artifact, bundle_dir, &format!("binaries[{index_u32}]"))?;
    }
    if let Some(report) = &manifest.reproducibility_report {
        verify_artifact_matches_bundle(report, bundle_dir, "reproducibility_report")?;
    }
    if let Some(report) = &manifest.independent_agreement_report {
        verify_artifact_matches_bundle(report, bundle_dir, "independent_agreement_report")?;
    }
    Ok(())
}

fn verify_artifact_matches_bundle(
    artifact: &BundledArtifact,
    bundle_dir: &Path,
    field_name: &str,
) -> Result<(), RunError> {
    let artifact_path = bundle_dir.join(&artifact.relative_path);
    let actual = build_artifact_record(&artifact_path, Path::new(&artifact.relative_path), artifact.kind)
        .map_err(|err| RunError::Internal(format!("verifying {field_name}: {err}")))?;
    if artifact != &actual {
        return Err(RunError::Internal(format!(
            "release evidence {field_name} does not match manifest: expected {} {} got {} {}",
            artifact.size_bytes, artifact.digest_blake3, actual.size_bytes, actual.digest_blake3,
        )));
    }
    Ok(())
}

fn verify_manifest_proof_linkage(manifest: &ReleaseEvidenceManifest, bundle_dir: &Path) -> Result<(), RunError> {
    let proof_bundle_dir = bundle_dir.join(&manifest.proof_bundle.relative_path);
    let proof_identity = load_full_self_hosting_proof_identity(&proof_bundle_dir)?;
    if manifest.proof_linkage.release_id != manifest.release_id {
        return Err(RunError::Internal("release evidence proof linkage release_id mismatch".to_string()));
    }
    if manifest.proof_linkage.source_archive_digest_blake3 != manifest.source_archive.digest_blake3 {
        return Err(RunError::Internal("release evidence proof linkage source archive digest mismatch".to_string()));
    }
    if proof_identity.schema != manifest.proof_linkage.proof_bundle_schema {
        return Err(RunError::Internal("release evidence proof linkage schema mismatch".to_string()));
    }
    if proof_identity.proof_mode != manifest.proof_linkage.proof_mode {
        return Err(RunError::Internal("release evidence proof linkage proof_mode mismatch".to_string()));
    }
    if proof_identity.selected_provider_kind != manifest.proof_linkage.selected_provider_kind {
        return Err(RunError::Internal("release evidence proof linkage selected_provider_kind mismatch".to_string()));
    }
    if proof_identity.staged_source != manifest.proof_linkage.staged_source {
        return Err(RunError::Internal("release evidence proof linkage staged_source mismatch".to_string()));
    }
    if proof_identity.stage2_binary_digest_blake3 != manifest.proof_linkage.stage2_binary_digest_blake3 {
        return Err(RunError::Internal("release evidence proof linkage stage2 digest mismatch".to_string()));
    }
    if proof_identity.prerequisite_inventory_digest_blake3
        != manifest.proof_linkage.prerequisite_inventory_digest_blake3
    {
        return Err(RunError::Internal(
            "release evidence proof linkage prerequisite inventory digest mismatch".to_string(),
        ));
    }
    if proof_identity.proof_manifest_digest_blake3 != manifest.proof_linkage.proof_manifest_digest_blake3 {
        return Err(RunError::Internal("release evidence proof linkage proof manifest digest mismatch".to_string()));
    }
    Ok(())
}

fn path_to_forward_slash_string(path: &Path) -> Result<String, RunError> {
    let raw = path
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("non-utf8 bundle path: {}", path.display())))?;
    Ok(raw.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn sample_digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % 16);
        byte.repeat(BLAKE3_HEX_LEN)
    }

    fn write_file(path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn sample_artifact(kind: BundledArtifactKind, relative_path: &str, seed: u8) -> BundledArtifact {
        BundledArtifact {
            kind,
            relative_path: relative_path.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
        }
    }

    fn sample_manifest() -> ReleaseEvidenceManifest {
        let stage2_binary = sample_artifact(BundledArtifactKind::File, "binaries/01-crunch", 3);
        let inventory = sample_artifact(BundledArtifactKind::File, "proof/inventory.md", 5);
        ReleaseEvidenceManifest {
            schema: RELEASE_EVIDENCE_SCHEMA.to_string(),
            release_id: "crunch-0.1.0-rc1".to_string(),
            claim_scope: CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
            workflow: ReleaseWorkflowIdentity {
                command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
                version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            },
            source_archive: sample_artifact(BundledArtifactKind::File, "source/crunch-src.tar", 1),
            binaries: vec![stage2_binary.clone()],
            proof_bundle: sample_artifact(BundledArtifactKind::Directory, "proof/self-hosting", 7),
            prerequisite_inventory: inventory.clone(),
            reproducibility_report: None,
            independent_agreement_report: None,
            proof_linkage: ReleaseProofLinkage {
                release_id: "crunch-0.1.0-rc1".to_string(),
                source_archive_digest_blake3: sample_digest(1),
                proof_bundle_schema: FULL_SELF_HOSTING_PROOF_SCHEMA.to_string(),
                proof_mode: "fixed-point".to_string(),
                selected_provider_kind: "source-root".to_string(),
                staged_source: "/tmp/proof-store/abcd-crunch-src".to_string(),
                stage2_binary_digest_blake3: stage2_binary.digest_blake3,
                prerequisite_inventory_digest_blake3: inventory.digest_blake3,
                proof_manifest_digest_blake3: sample_digest(9),
            },
        }
    }

    fn write_full_proof_manifest(bundle_dir: &Path, inventory_digest: &str, stage2_digest: &str) {
        let manifest = json!({
            "schema": FULL_SELF_HOSTING_PROOF_SCHEMA,
            "staged_source": "/tmp/proof-store/abcd-crunch-src",
            "prerequisites": {
                "mode": "fixed-point",
                "provider_kind": "source-root",
                "inventory_doc": {
                    "path": "/tmp/proof-bundle/stage0-prerequisites/inventory.md",
                    "size_bytes": 9,
                    "digest_blake3": inventory_digest
                }
            },
            "binaries": {
                "stage1": {
                    "path": "/tmp/proof-store/stage1-crunch/bin/crunch",
                    "size_bytes": 20,
                    "digest_blake3": sample_digest(10)
                },
                "stage2": {
                    "path": "/tmp/proof-store/stage2-crunch/bin/crunch",
                    "size_bytes": 13,
                    "digest_blake3": stage2_digest
                }
            },
            "tools": {
                "stage0_bwrap": {
                    "path": "/tmp/proof-store/stage0-bwrap/bin/bwrap",
                    "size_bytes": 22,
                    "digest_blake3": sample_digest(12)
                },
                "stage0_busybox": {
                    "path": "/tmp/proof-store/stage0-busybox/bin/busybox",
                    "size_bytes": 23,
                    "digest_blake3": sample_digest(13)
                },
                "stage2_bwrap": {
                    "path": "/tmp/proof-store/stage2-bwrap/bin/bwrap",
                    "size_bytes": 24,
                    "digest_blake3": sample_digest(14)
                },
                "stage2_busybox": {
                    "path": "/tmp/proof-store/stage2-busybox/bin/busybox",
                    "size_bytes": 25,
                    "digest_blake3": sample_digest(15)
                }
            },
            "fixed_point": {
                "stage1_equals_stage2": true,
                "stage0_bwrap_equals_stage2_bwrap": true,
                "stage0_busybox_equals_stage2_busybox": true
            },
            "stage0": {
                "report": {
                    "staged_source": "/tmp/proof-store/abcd-crunch-src",
                    "output_binary": "/tmp/proof-store/stage1-crunch/bin/crunch",
                    "busybox_path": "/tmp/proof-store/stage0-busybox/bin/busybox"
                }
            },
            "stage2": {
                "report": {
                    "staged_source": "/tmp/proof-store/abcd-crunch-src",
                    "output_binary": "/tmp/proof-store/stage2-crunch/bin/crunch",
                    "busybox_path": "/tmp/proof-store/stage2-busybox/bin/busybox"
                }
            }
        });
        std::fs::create_dir_all(bundle_dir).unwrap();
        write_file(&bundle_dir.join("manifest.json"), &serde_json::to_vec(&manifest).unwrap());
        write_file(&bundle_dir.join(PROOF_INVENTORY_RELATIVE_PATH), b"inventory");
        write_file(&bundle_dir.join("summary.txt"), b"summary");
        write_file(&bundle_dir.join("stage0/stdout.txt"), b"stage0 stdout");
    }

    #[test]
    fn canonical_bytes_are_stable_and_compact() {
        let manifest = sample_manifest();
        let first = canonical_release_evidence_manifest(manifest.clone()).unwrap();
        let second = canonical_release_evidence_manifest(manifest.clone()).unwrap();
        assert_eq!(first, second);
        assert!(!first.contains(&b'\n'));
    }

    #[test]
    fn validate_rejects_absolute_member_path() {
        let mut manifest = sample_manifest();
        manifest.source_archive.relative_path = "/tmp/source.tar".to_string();
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn validate_rejects_prerequisite_inventory_linkage_mismatch() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.prerequisite_inventory_digest_blake3 = sample_digest(8);
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("prerequisite inventory digest does not match"));
    }

    #[test]
    fn validate_rejects_stage2_digest_not_present_in_binaries() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.stage2_binary_digest_blake3 = sample_digest(4);
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("does not match any bundled binary artifact"));
    }

    #[test]
    fn load_full_self_hosting_proof_identity_accepts_full_proof_bundle() {
        let dir = tempfile::tempdir().unwrap();
        write_full_proof_manifest(dir.path(), &sample_digest(9), &sample_digest(11));
        let identity = load_full_self_hosting_proof_identity(dir.path()).unwrap();
        assert_eq!(identity.schema, FULL_SELF_HOSTING_PROOF_SCHEMA);
        assert_eq!(identity.proof_mode, "fixed-point");
        assert_eq!(identity.staged_source, "/tmp/proof-store/abcd-crunch-src");
        assert_eq!(identity.stage2_binary_digest_blake3, sample_digest(11));
        assert_eq!(identity.prerequisite_inventory_digest_blake3, sample_digest(9));
        assert_eq!(identity.proof_manifest_digest_blake3.len(), BLAKE3_HEX_LEN);
    }

    #[test]
    fn load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("manifest.json"), br#"{"schema":"fake-proof"}"#);
        let err = load_full_self_hosting_proof_identity(dir.path()).unwrap_err();
        assert!(err.to_string().contains("full proof artifact required"));
    }

    #[test]
    fn create_and_verify_release_bundle_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("crunch-src.tar");
        let binary_path = temp.path().join("crunch");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"crunch-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "crunch-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive.clone(),
            vec![binary_path.clone()],
            proof_bundle_dir.clone(),
        );
        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();

        assert_eq!(created.release_id, "crunch-0.1.0-rc1");
        assert_eq!(created.proof_linkage.selected_provider_kind, "source-root");
        assert_eq!(created, verified);
        assert!(output_bundle_dir.join("proof/self-hosting/manifest.json").exists());
        assert!(output_bundle_dir.join("proof/inventory.md").exists());
        assert!(output_bundle_dir.join("manifest.json").exists());
    }

    #[test]
    fn verify_rejects_provider_kind_linkage_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("crunch-src.tar");
        let binary_path = temp.path().join("crunch");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source");
        write_file(&binary_path, b"crunch-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "crunch-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        let mut created = create_release_evidence_bundle(&request).unwrap();
        created.proof_linkage.selected_provider_kind = "stagex-lineage".to_string();
        write_manifest_file(&output_bundle_dir, &created).unwrap();

        let err = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(err.to_string().contains("selected_provider_kind mismatch"));
    }

    #[test]
    fn verify_rejects_non_canonical_manifest_json() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("crunch-src.tar");
        let binary_path = temp.path().join("crunch");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"crunch-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "crunch-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        let created = create_release_evidence_bundle(&request).unwrap();
        let pretty_json = serde_json::to_string_pretty(&created).unwrap();
        write_file(&output_bundle_dir.join("manifest.json"), pretty_json.as_bytes());

        let err = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(err.to_string().contains("not canonical compact JSON"));
    }

    #[test]
    fn verify_rejects_tampered_binary_artifact() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("crunch-src.tar");
        let binary_path = temp.path().join("crunch");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"crunch-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"crunch-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "crunch-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        create_release_evidence_bundle(&request).unwrap();
        write_file(&output_bundle_dir.join("binaries/01-crunch"), b"tampered-binary");

        let err = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(err.to_string().contains("binaries[0] does not match manifest"));
    }
}
