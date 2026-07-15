// machine-artifact-public: release.witness-handoff
use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::AttestationDigest;
use crunch_attestation::Canonicalize;
use crunch_attestation::WitnessAttestation;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::release_attestation::RELEASE_ATTESTATION_FILE_NAME;
use crate::release_attestation::RELEASE_ATTESTATION_SIG_FILE_NAME;
use crate::release_attestation::WITNESSES_DIR_NAME;
use crate::release_attestation::load_release_attestation_document;
use crate::release_attestation::prepare_witness_dir;
use crate::release_attestation::validate_witness_identity;
use crate::release_evidence::ReleaseEvidenceManifest;
use crate::release_evidence::copy_directory_tree;

pub(crate) const WITNESS_REQUEST_SCHEMA: &str = "mantle-witness-request-v1";
pub(crate) const WITNESS_REQUEST_FILE_NAME: &str = "request.json";
const REQUEST_LAYOUT_VERSION_V1: u32 = 1;
const REQUEST_BUNDLE_DIR_NAME: &str = "release-evidence";
const REQUEST_VERIFICATION_DIR_NAME: &str = "release-verification";
const REQUEST_TARGET_DIR_NAME: &str = "release-witness-requests";
const JSON_EXTENSION: &str = "json";
const SIGNATURE_SUFFIX: &str = ".sig";
const MAX_IMPORT_CANDIDATES: u32 = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WitnessRequestLayout {
    pub request_file_relative_path: PathBuf,
    pub release_bundle_relative_path: PathBuf,
    pub verification_seed_relative_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WitnessRequestDocument {
    pub schema: String,
    pub request_layout_version: u32,
    pub release_id: String,
    pub release_bundle_relative_path: String,
    pub verification_seed_relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreatedWitnessRequest {
    pub request_dir: PathBuf,
    pub request_path: PathBuf,
    pub release_id: String,
    pub layout_version: u32,
    pub release_bundle_path: PathBuf,
    pub verification_seed_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedWitnessMaterial {
    pub verification_dir: PathBuf,
    pub imported_witness_identities: Vec<String>,
    pub skipped_duplicate_identities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WitnessImportCandidate {
    identity: String,
    attestation: WitnessAttestation,
    attestation_bytes: Vec<u8>,
    signature_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WitnessImportDecision {
    WriteNew,
    SkipExactDuplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WitnessImportPlanEntry {
    identity: String,
    decision: WitnessImportDecision,
    attestation_bytes: Vec<u8>,
    signature_bytes: Vec<u8>,
}

pub(crate) fn default_witness_request_dir(current_dir: &Path, release_id: &str) -> PathBuf {
    current_dir.join("target").join(REQUEST_TARGET_DIR_NAME).join(release_id)
}

pub(crate) fn plan_witness_request_layout(release_id: &str) -> Result<WitnessRequestLayout, RunError> {
    validate_release_id(release_id)?;
    Ok(WitnessRequestLayout {
        request_file_relative_path: PathBuf::from(WITNESS_REQUEST_FILE_NAME),
        release_bundle_relative_path: PathBuf::from(REQUEST_BUNDLE_DIR_NAME).join(release_id),
        verification_seed_relative_path: PathBuf::from(REQUEST_VERIFICATION_DIR_NAME).join(release_id),
    })
}

pub(crate) fn create_witness_request_directory(
    verified_manifest: &ReleaseEvidenceManifest,
    bundle_dir: &Path,
    verification_dir: &Path,
    request_dir: &Path,
) -> Result<CreatedWitnessRequest, RunError> {
    debug_assert!(!WITNESS_REQUEST_SCHEMA.is_empty());
    debug_assert!(!WITNESS_REQUEST_FILE_NAME.is_empty());
    let (release_attestation, _attestation_path) = load_release_attestation_document(verification_dir)?;
    validate_matching_release_ids(ReleaseIdPair {
        bundle: &verified_manifest.release_id,
        attestation: &release_attestation.release_id,
    })?;
    let layout = plan_witness_request_layout(&verified_manifest.release_id)?;
    let request_document = build_witness_request_document(&layout, &verified_manifest.release_id)?;
    prepare_empty_directory(request_dir, "witness request directory")?;
    let release_bundle_dest = request_dir.join(&layout.release_bundle_relative_path);
    copy_directory_into_destination(bundle_dir, &release_bundle_dest)?;
    let verification_seed_dest = request_dir.join(&layout.verification_seed_relative_path);
    copy_release_attestation_seed(verification_dir, &verification_seed_dest)?;
    let request_path = request_dir.join(&layout.request_file_relative_path);
    write_compact_json(&request_path, &request_document)?;
    Ok(CreatedWitnessRequest {
        request_dir: request_dir.to_path_buf(),
        request_path,
        release_id: verified_manifest.release_id.clone(),
        layout_version: REQUEST_LAYOUT_VERSION_V1,
        release_bundle_path: release_bundle_dest,
        verification_seed_path: verification_seed_dest,
    })
}

pub(crate) fn import_witness_material(
    verification_dir: &Path,
    source: &Path,
) -> Result<ImportedWitnessMaterial, RunError> {
    let destination_release_digest = load_destination_release_digest(verification_dir)?;
    let candidates = load_witness_import_candidates(source)?;
    let witness_dir = prepare_witness_dir(verification_dir)?;
    let plan = build_witness_import_plan(&destination_release_digest, &witness_dir, &candidates)?;
    let applied = apply_witness_import_plan(&witness_dir, &plan)?;
    Ok(ImportedWitnessMaterial {
        verification_dir: verification_dir.to_path_buf(),
        imported_witness_identities: applied.imported_witness_identities,
        skipped_duplicate_identities: applied.skipped_duplicate_identities,
    })
}

fn validate_release_id(release_id: &str) -> Result<(), RunError> {
    let trimmed_release_id = release_id.trim();
    if trimmed_release_id.is_empty() {
        return Err(RunError::Internal("release id must not be empty".to_string()));
    }
    if trimmed_release_id.contains('/') || trimmed_release_id.contains('\\') {
        return Err(RunError::Internal(format!("release id must not contain path separators: {}", trimmed_release_id)));
    }
    Ok(())
}

struct ReleaseIdPair<'a> {
    bundle: &'a str,
    attestation: &'a str,
}

fn validate_matching_release_ids(release_ids: ReleaseIdPair<'_>) -> Result<(), RunError> {
    if release_ids.bundle == release_ids.attestation {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "release id mismatch between verified bundle '{}' and release attestation '{}'",
        release_ids.bundle, release_ids.attestation
    )))
}

fn build_witness_request_document(
    layout: &WitnessRequestLayout,
    release_id: &str,
) -> Result<WitnessRequestDocument, RunError> {
    Ok(WitnessRequestDocument {
        schema: WITNESS_REQUEST_SCHEMA.to_string(),
        request_layout_version: REQUEST_LAYOUT_VERSION_V1,
        release_id: release_id.to_string(),
        release_bundle_relative_path: path_to_forward_slash_string(&layout.release_bundle_relative_path)?,
        verification_seed_relative_path: path_to_forward_slash_string(&layout.verification_seed_relative_path)?,
    })
}

fn path_to_forward_slash_string(path: &Path) -> Result<String, RunError> {
    let rendered = path
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("path is not valid UTF-8: {}", path.display())))?;
    Ok(rendered.replace('\\', "/"))
}

fn prepare_empty_directory(path: &Path, label: &str) -> Result<(), RunError> {
    if path.exists() {
        if !path.is_dir() {
            return Err(RunError::Internal(format!("{} is not a directory: {}", label, path.display())));
        }
        let mut entries =
            std::fs::read_dir(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
        if entries.next().is_some() {
            return Err(RunError::Internal(format!("{} must be empty: {}", label, path.display())));
        }
        return Ok(());
    }
    std::fs::create_dir_all(path).map_err(|err| RunError::Internal(format!("creating {}: {err}", path.display())))
}

fn copy_directory_into_destination(source_dir: &Path, dest_dir: &Path) -> Result<(), RunError> {
    if !source_dir.is_dir() {
        return Err(RunError::Internal(format!("witness-request source directory missing: {}", source_dir.display())));
    }
    std::fs::create_dir_all(dest_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", dest_dir.display())))?;
    copy_directory_tree(source_dir, dest_dir)
}

fn copy_release_attestation_seed(source_verification_dir: &Path, dest_verification_dir: &Path) -> Result<(), RunError> {
    std::fs::create_dir_all(dest_verification_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", dest_verification_dir.display())))?;
    copy_required_file(
        &source_verification_dir.join(RELEASE_ATTESTATION_FILE_NAME),
        &dest_verification_dir.join(RELEASE_ATTESTATION_FILE_NAME),
        "release attestation",
    )?;
    copy_required_file(
        &source_verification_dir.join(RELEASE_ATTESTATION_SIG_FILE_NAME),
        &dest_verification_dir.join(RELEASE_ATTESTATION_SIG_FILE_NAME),
        "release attestation signature",
    )
}

fn copy_required_file(source_path: &Path, dest_path: &Path, label: &str) -> Result<(), RunError> {
    if !source_path.is_file() {
        return Err(RunError::Internal(format!("{} missing: {}", label, source_path.display())));
    }
    std::fs::copy(source_path, dest_path).map_err(|err| {
        RunError::Internal(format!("copying {} to {}: {err}", source_path.display(), dest_path.display()))
    })?;
    Ok(())
}

fn write_compact_json<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|err| RunError::Internal(format!("serializing {}: {err}", path.display())))?;
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn load_destination_release_digest(verification_dir: &Path) -> Result<AttestationDigest, RunError> {
    let (release_attestation, _attestation_path) = load_release_attestation_document(verification_dir)?;
    release_attestation
        .canonical_digest()
        .map_err(|err| RunError::Internal(format!("release attestation digest: {err}")))
}

fn load_witness_import_candidates(source: &Path) -> Result<Vec<WitnessImportCandidate>, RunError> {
    if source.is_file() {
        return Ok(vec![load_single_witness_import_candidate(source)?]);
    }
    let witness_dir = resolve_witness_source_directory(source)?;
    let json_paths = collect_witness_json_paths(&witness_dir)?;
    let mut candidates = Vec::with_capacity(json_paths.len());
    for json_path in &json_paths {
        candidates.push(load_single_witness_import_candidate(json_path)?);
    }
    Ok(candidates)
}

fn resolve_witness_source_directory(source: &Path) -> Result<PathBuf, RunError> {
    if !source.is_dir() {
        return Err(RunError::Internal(format!("witness import source does not exist: {}", source.display())));
    }
    let nested_witness_dir = source.join(WITNESSES_DIR_NAME);
    if nested_witness_dir.is_dir() {
        return Ok(nested_witness_dir);
    }
    Ok(source.to_path_buf())
}

fn collect_witness_json_paths(witness_dir: &Path) -> Result<Vec<PathBuf>, RunError> {
    let maximum_candidate_count = usize::try_from(MAX_IMPORT_CANDIDATES)
        .map_err(|_| RunError::Internal("witness import candidate limit does not fit usize".to_string()))?;
    debug_assert!(maximum_candidate_count > 0);
    debug_assert!(!JSON_EXTENSION.is_empty());
    let mut json_paths = Vec::with_capacity(maximum_candidate_count);
    let entries = std::fs::read_dir(witness_dir)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", witness_dir.display())))?;
    for entry_result in entries {
        let entry = entry_result
            .map_err(|err| RunError::Internal(format!("reading {} entry: {err}", witness_dir.display())))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some(JSON_EXTENSION) {
            if json_paths.len() >= maximum_candidate_count {
                return Err(RunError::Internal(format!(
                    "witness import candidate count exceeds limit {MAX_IMPORT_CANDIDATES}"
                )));
            }
            json_paths.push(path);
        }
    }
    json_paths.sort();
    let candidate_count_u32 = u32::try_from(json_paths.len())
        .map_err(|_| RunError::Internal("witness import candidate count overflowed u32".to_string()))?;
    if candidate_count_u32 == 0 {
        return Err(RunError::Internal(format!(
            "no witness attestation json files found in {}",
            witness_dir.display()
        )));
    }
    if candidate_count_u32 > MAX_IMPORT_CANDIDATES {
        return Err(RunError::Internal(format!(
            "witness import candidate count {} exceeds limit {}",
            candidate_count_u32, MAX_IMPORT_CANDIDATES
        )));
    }
    Ok(json_paths)
}

fn load_single_witness_import_candidate(attestation_path: &Path) -> Result<WitnessImportCandidate, RunError> {
    let attestation_bytes = std::fs::read(attestation_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", attestation_path.display())))?;
    let attestation: WitnessAttestation = serde_json::from_slice(&attestation_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", attestation_path.display())))?;
    validate_witness_identity(&attestation.witness_identity)?;
    let signature_path = signature_path_for_attestation(attestation_path)?;
    let signature_bytes = std::fs::read(&signature_path).map_err(|err| {
        RunError::Internal(format!("missing witness signature sidecar {}: {err}", signature_path.display()))
    })?;
    Ok(WitnessImportCandidate {
        identity: attestation.witness_identity.clone(),
        attestation,
        attestation_bytes,
        signature_bytes,
    })
}

fn signature_path_for_attestation(attestation_path: &Path) -> Result<PathBuf, RunError> {
    let file_name = attestation_path.file_name().and_then(|value| value.to_str()).ok_or_else(|| {
        RunError::Internal(format!("witness attestation path has no UTF-8 file name: {}", attestation_path.display()))
    })?;
    Ok(attestation_path.with_file_name(format!("{file_name}{SIGNATURE_SUFFIX}")))
}

fn build_witness_import_plan(
    destination_release_digest: &AttestationDigest,
    witness_dir: &Path,
    candidates: &[WitnessImportCandidate],
) -> Result<Vec<WitnessImportPlanEntry>, RunError> {
    let maximum_candidate_count = usize::try_from(MAX_IMPORT_CANDIDATES)
        .map_err(|_| RunError::Internal("witness import candidate limit does not fit usize".to_string()))?;
    if candidates.len() > maximum_candidate_count {
        return Err(RunError::Internal(format!(
            "witness import candidate count exceeds limit {MAX_IMPORT_CANDIDATES}"
        )));
    }
    let mut plan = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        validate_release_digest_match(destination_release_digest, candidate)?;
        let destination_paths = destination_witness_paths(witness_dir, &candidate.identity);
        let existing_attestation = read_optional_bytes(&destination_paths.0)?;
        let existing_signature = read_optional_bytes(&destination_paths.1)?;
        let decision = classify_witness_import(
            existing_attestation.as_deref(),
            existing_signature.as_deref(),
            &candidate.attestation_bytes,
            &candidate.signature_bytes,
            &candidate.identity,
        )?;
        plan.push(WitnessImportPlanEntry {
            identity: candidate.identity.clone(),
            decision,
            attestation_bytes: candidate.attestation_bytes.clone(),
            signature_bytes: candidate.signature_bytes.clone(),
        });
    }
    debug_assert_eq!(plan.len(), candidates.len());
    debug_assert!(plan.len() <= maximum_candidate_count);
    Ok(plan)
}

fn validate_release_digest_match(
    destination_release_digest: &AttestationDigest,
    candidate: &WitnessImportCandidate,
) -> Result<(), RunError> {
    if &candidate.attestation.release_attestation_digest_blake3 == destination_release_digest {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "witness '{}' references release attestation digest {} but destination verification directory expects {}",
        candidate.identity,
        candidate.attestation.release_attestation_digest_blake3.to_hex(),
        destination_release_digest.to_hex()
    )))
}

fn destination_witness_paths(witness_dir: &Path, identity: &str) -> (PathBuf, PathBuf) {
    let attestation_path = witness_dir.join(format!("{identity}.json"));
    let signature_path = witness_dir.join(format!("{identity}.json.sig"));
    (attestation_path, signature_path)
}

fn read_optional_bytes(path: &Path) -> Result<Option<Vec<u8>>, RunError> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    Ok(Some(bytes))
}

fn classify_witness_import(
    existing_attestation: Option<&[u8]>,
    existing_signature: Option<&[u8]>,
    imported_attestation: &[u8],
    imported_signature: &[u8],
    identity: &str,
) -> Result<WitnessImportDecision, RunError> {
    match (existing_attestation, existing_signature) {
        (None, None) => Ok(WitnessImportDecision::WriteNew),
        (Some(existing_attestation), Some(existing_signature)) => {
            if existing_attestation == imported_attestation && existing_signature == imported_signature {
                return Ok(WitnessImportDecision::SkipExactDuplicate);
            }
            Err(RunError::Internal(format!(
                "conflicting witness identity '{}' already exists in destination verification directory",
                identity
            )))
        }
        _ => Err(RunError::Internal(format!(
            "conflicting witness identity '{}' already exists in destination verification directory",
            identity
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppliedWitnessImportPlan {
    imported_witness_identities: Vec<String>,
    skipped_duplicate_identities: Vec<String>,
}

fn apply_witness_import_plan(
    witness_dir: &Path,
    plan: &[WitnessImportPlanEntry],
) -> Result<AppliedWitnessImportPlan, RunError> {
    let mut written_witness_identities = Vec::with_capacity(plan.len());
    let mut skipped_duplicate_identities = Vec::with_capacity(plan.len());
    for entry in plan {
        let destination_paths = destination_witness_paths(witness_dir, &entry.identity);
        match entry.decision {
            WitnessImportDecision::WriteNew => {
                std::fs::write(&destination_paths.0, &entry.attestation_bytes)
                    .map_err(|err| RunError::Internal(format!("writing {}: {err}", destination_paths.0.display())))?;
                std::fs::write(&destination_paths.1, &entry.signature_bytes)
                    .map_err(|err| RunError::Internal(format!("writing {}: {err}", destination_paths.1.display())))?;
                written_witness_identities.push(entry.identity.clone());
            }
            WitnessImportDecision::SkipExactDuplicate => skipped_duplicate_identities.push(entry.identity.clone()),
        }
    }
    debug_assert_eq!(written_witness_identities.len().saturating_add(skipped_duplicate_identities.len()), plan.len());
    debug_assert!(written_witness_identities.len() <= plan.len());
    Ok(AppliedWitnessImportPlan {
        imported_witness_identities: written_witness_identities,
        skipped_duplicate_identities,
    })
}
