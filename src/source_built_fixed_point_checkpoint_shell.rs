//! Imperative storage shell for promoted source-built provider checkpoints.
//!
//! This module observes, publishes, loads, admits, and restores checkpoint
//! payloads. The pure checkpoint core owns every authority decision.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read as _;
use std::io::Write as _;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt as _;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt as _;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_checkpoint::CheckpointPayloadIdentity;
use crate::source_built_fixed_point_checkpoint::CheckpointPayloadKind;
use crate::source_built_fixed_point_checkpoint::ProofCheckpointOrigin;
use crate::source_built_fixed_point_checkpoint::ProviderCheckpointAdmission;
use crate::source_built_fixed_point_checkpoint::ProviderCheckpointManifest;
use crate::source_built_fixed_point_checkpoint::ProviderCheckpointStageObservation;
use crate::source_built_fixed_point_checkpoint::admit_promoted_provider_checkpoint;
use crate::source_built_fixed_point_checkpoint::build_provider_checkpoint_manifest;
use crate::source_built_fixed_point_checkpoint::provider_checkpoint_lookup_key;
use crate::source_built_fixed_point_checkpoint::provider_checkpoint_manifest_digest;

pub(crate) const PROVIDER_CHECKPOINTS_SUBDIR: &str = "promoted-provider-checkpoints";
pub(crate) const PROVIDER_CHECKPOINT_MANIFEST_FILE: &str = "manifest.json";
const CHECKPOINT_PAYLOAD_ROOT: &str = "payload";
const CHECKPOINT_TEMP_PREFIX: &str = ".checkpoint-tmp-";
const COPY_BUFFER_BYTES: usize = 64 * 1_024;
const SINGLE_FILE_ENTRY_COUNT: u32 = 1;
const CHECKPOINT_CANDIDATES_MAX: usize = 16;
const PRESERVED_CHECKPOINT_ENTRIES_MAX: u32 = 2_000_000;
#[cfg(unix)]
const PERMISSION_BITS_MASK: u32 = 0o7_777;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProviderCheckpointLimits {
    pub(crate) preserved_tree: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
}

#[derive(Debug, Clone)]
pub(crate) struct CheckpointPayloadSource {
    pub(crate) payload_id: String,
    pub(crate) source_path: PathBuf,
    pub(crate) relative_path: PathBuf,
    pub(crate) kind: CheckpointPayloadKind,
}

#[derive(Debug, Clone)]
pub(crate) struct CheckpointPayloadRestore {
    pub(crate) payload_id: String,
    pub(crate) destination_path: PathBuf,
    pub(crate) kind: CheckpointPayloadKind,
}

#[derive(Debug, Clone)]
pub(crate) struct PublishedProviderCheckpoint {
    pub(crate) checkpoint_root: PathBuf,
    pub(crate) manifest_path: PathBuf,
    pub(crate) checkpoint_digest_blake3: String,
    pub(crate) lookup_key_blake3: String,
    pub(crate) disposition: CheckpointPublicationDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckpointPublicationDisposition {
    Published,
    ExistingIdentical,
}

#[derive(Debug, Clone)]
pub(crate) struct AdmittedProviderCheckpoint {
    pub(crate) checkpoint_root: PathBuf,
    pub(crate) manifest: ProviderCheckpointManifest,
    pub(crate) admission: ProviderCheckpointAdmission,
}

#[derive(Debug, Clone)]
pub(crate) struct RestoredProviderCheckpoint {
    pub(crate) checkpoint_root: PathBuf,
    pub(crate) manifest: ProviderCheckpointManifest,
    pub(crate) admission: ProviderCheckpointAdmission,
}

pub(crate) fn provider_checkpoint_limits(total_file_bytes_max: u64) -> ProviderCheckpointLimits {
    let release = crunch_release_core::TreeCopyLimits::RELEASE_BUNDLE;
    ProviderCheckpointLimits {
        preserved_tree: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits {
            entries_count_max: PRESERVED_CHECKPOINT_ENTRIES_MAX,
            depth_count_max: release.depth_count_max,
            path_bytes_max: release.path_bytes_max,
            symlink_target_bytes_max: release.path_bytes_max,
            total_file_bytes_max,
        },
    }
}

// r[impl bootstrap_inventory.source_built_mantle_checkpoint_reuse]
pub(crate) fn publish_provider_checkpoint(
    checkpoint_store: &Path,
    plan: &SourceBuiltFixedPointPlan,
    stage_observations: Vec<ProviderCheckpointStageObservation>,
    payload_sources: &[CheckpointPayloadSource],
    expected_stagex_provider_digest_blake3: &str,
    limits: ProviderCheckpointLimits,
) -> Result<PublishedProviderCheckpoint, RunError> {
    require_absolute_directory_or_create(checkpoint_store)?;
    let payloads = observe_payload_sources(payload_sources, limits)?;
    let manifest = build_provider_checkpoint_manifest(
        plan,
        ProofCheckpointOrigin::PromotedExecution,
        stage_observations,
        payloads,
    )
    .map_err(checkpoint_policy_error)?;
    let lookup_key_blake3 = provider_checkpoint_lookup_key(plan).map_err(checkpoint_policy_error)?;
    let checkpoint_digest_blake3 = provider_checkpoint_manifest_digest(&manifest).map_err(checkpoint_policy_error)?;
    let checkpoint_root = checkpoint_store
        .join(PROVIDER_CHECKPOINTS_SUBDIR)
        .join(&lookup_key_blake3)
        .join(&checkpoint_digest_blake3);
    if checkpoint_root.exists() {
        return validate_existing_checkpoint(
            checkpoint_root,
            plan,
            &manifest,
            expected_stagex_provider_digest_blake3,
            limits,
        );
    }
    publish_checkpoint_create_new(
        checkpoint_root,
        plan,
        &manifest,
        payload_sources,
        expected_stagex_provider_digest_blake3,
        limits,
    )
}

// r[impl bootstrap_inventory.source_built_mantle_checkpoint_reuse]
pub(crate) fn admit_provider_checkpoint_store(
    checkpoint_store: &Path,
    plan: &SourceBuiltFixedPointPlan,
    expected_stagex_provider_digest_blake3: &str,
    limits: ProviderCheckpointLimits,
) -> Result<Option<AdmittedProviderCheckpoint>, RunError> {
    if !checkpoint_store.is_absolute() {
        return Err(checkpoint_error(format!("checkpoint store must be absolute: {}", checkpoint_store.display())));
    }
    let lookup_key = provider_checkpoint_lookup_key(plan).map_err(checkpoint_policy_error)?;
    let lookup_root = checkpoint_store.join(PROVIDER_CHECKPOINTS_SUBDIR).join(lookup_key);
    let selected = select_admitted_checkpoint(&lookup_root, plan, expected_stagex_provider_digest_blake3, limits)?;
    Ok(selected.map(|(checkpoint_root, manifest, admission)| AdmittedProviderCheckpoint {
        checkpoint_root,
        manifest,
        admission,
    }))
}

// r[impl bootstrap_inventory.source_built_mantle_checkpoint_reuse]
pub(crate) fn restore_provider_checkpoint(
    admitted: AdmittedProviderCheckpoint,
    restores: &[CheckpointPayloadRestore],
    limits: ProviderCheckpointLimits,
) -> Result<RestoredProviderCheckpoint, RunError> {
    validate_restore_requests(restores, &admitted.manifest)?;
    let mut created = Vec::new();
    for request in restores {
        if let Err(error) = restore_one_payload(&admitted, request, limits) {
            return Err(add_cleanup_result(error, cleanup_restores(&created)));
        }
        created.push(request.destination_path.to_path_buf());
    }
    assert_eq!(created.len(), restores.len());
    Ok(RestoredProviderCheckpoint {
        checkpoint_root: admitted.checkpoint_root,
        manifest: admitted.manifest,
        admission: admitted.admission,
    })
}

fn restore_one_payload(
    admitted: &AdmittedProviderCheckpoint,
    request: &CheckpointPayloadRestore,
    limits: ProviderCheckpointLimits,
) -> Result<(), RunError> {
    let payload = payload_by_id(&admitted.manifest, &request.payload_id)?;
    let source = admitted.checkpoint_root.join(&payload.relative_path);
    if request.destination_path.exists() {
        return Err(checkpoint_error(format!(
            "checkpoint restore destination exists: {}",
            request.destination_path.display()
        )));
    }
    copy_payload(&source, &request.destination_path, request.kind, limits)?;
    let observed = observe_payload(
        &request.destination_path,
        payload.payload_id.as_str(),
        Path::new(&payload.relative_path),
        request.kind,
        limits,
    )?;
    if &observed != payload {
        remove_tree(&request.destination_path)?;
        return Err(checkpoint_error(format!(
            "restored checkpoint payload identity mismatch for {}",
            request.payload_id
        )));
    }
    Ok(())
}

fn select_admitted_checkpoint(
    lookup_root: &Path,
    plan: &SourceBuiltFixedPointPlan,
    expected_stagex_provider_digest_blake3: &str,
    limits: ProviderCheckpointLimits,
) -> Result<Option<(PathBuf, ProviderCheckpointManifest, ProviderCheckpointAdmission)>, RunError> {
    if !lookup_root.exists() {
        return Ok(None);
    }
    if !lookup_root.is_dir() {
        return Err(checkpoint_error(format!("checkpoint lookup root is not a directory: {}", lookup_root.display())));
    }
    let mut candidates = fs::read_dir(lookup_root)
        .map_err(|error| checkpoint_error(format!("reading checkpoint candidates {}: {error}", lookup_root.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| checkpoint_error(format!("reading checkpoint candidate: {error}")))?;
    candidates.sort_by_key(fs::DirEntry::file_name);
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() > CHECKPOINT_CANDIDATES_MAX {
        return Err(checkpoint_error(format!("checkpoint candidate count exceeds {CHECKPOINT_CANDIDATES_MAX}")));
    }
    let mut selected = None;
    let mut semantic_outputs = None;
    for candidate in candidates {
        let path = candidate.path();
        let name = candidate
            .file_name()
            .into_string()
            .map_err(|_| checkpoint_error("checkpoint candidate name is not UTF-8".to_string()))?;
        if !path.is_dir() || !is_lower_blake3(&name) {
            return Err(checkpoint_error(format!("partial or unknown checkpoint candidate: {}", path.display())));
        }
        let (manifest, admission) =
            load_and_admit_checkpoint(&path, plan, expected_stagex_provider_digest_blake3, limits)?;
        if admission.checkpoint_digest_blake3 != name {
            return Err(checkpoint_error(format!("checkpoint candidate path digest mismatch: {}", path.display())));
        }
        let current_outputs = manifest
            .stages
            .iter()
            .map(|stage| (stage.output_role, stage.semantic_output_digest_blake3.clone()))
            .collect::<Vec<_>>();
        if semantic_outputs.as_ref().is_some_and(|expected| expected != &current_outputs) {
            return Err(checkpoint_error(format!(
                "conflicting promoted provider checkpoints for {}",
                lookup_root.display()
            )));
        }
        semantic_outputs = Some(current_outputs);
        if selected.is_none() {
            selected = Some((path, manifest, admission));
        }
    }
    assert!(selected.is_some());
    Ok(selected)
}

pub(crate) fn load_and_admit_checkpoint(
    checkpoint_root: &Path,
    plan: &SourceBuiltFixedPointPlan,
    expected_stagex_provider_digest_blake3: &str,
    limits: ProviderCheckpointLimits,
) -> Result<(ProviderCheckpointManifest, ProviderCheckpointAdmission), RunError> {
    let manifest = read_checkpoint_manifest(checkpoint_root)?;
    let observed = observe_manifest_payloads(checkpoint_root, &manifest, limits)?;
    let admission =
        admit_promoted_provider_checkpoint(plan, &manifest, &observed, expected_stagex_provider_digest_blake3)
            .map_err(checkpoint_policy_error)?;
    assert_eq!(manifest.lookup_key_blake3, admission.lookup_key_blake3);
    Ok((manifest, admission))
}

fn publish_checkpoint_create_new(
    checkpoint_root: PathBuf,
    plan: &SourceBuiltFixedPointPlan,
    manifest: &ProviderCheckpointManifest,
    payload_sources: &[CheckpointPayloadSource],
    expected_stagex_provider_digest_blake3: &str,
    limits: ProviderCheckpointLimits,
) -> Result<PublishedProviderCheckpoint, RunError> {
    let parent = checkpoint_root
        .parent()
        .ok_or_else(|| checkpoint_error("checkpoint root has no parent".to_string()))?;
    fs::create_dir_all(parent)
        .map_err(|error| checkpoint_error(format!("creating checkpoint parent {}: {error}", parent.display())))?;
    let temporary =
        parent.join(format!("{CHECKPOINT_TEMP_PREFIX}{}-{}", manifest.lookup_key_blake3, std::process::id()));
    if temporary.exists() {
        return Err(checkpoint_error(format!("checkpoint temporary path exists: {}", temporary.display())));
    }
    fs::create_dir(&temporary).map_err(|error| {
        checkpoint_error(format!("creating checkpoint temporary root {}: {error}", temporary.display()))
    })?;
    let result = materialize_checkpoint_candidate(
        &temporary,
        plan,
        manifest,
        payload_sources,
        expected_stagex_provider_digest_blake3,
        limits,
    );
    if let Err(error) = result {
        return Err(add_cleanup_result(error, remove_tree(&temporary)));
    }
    if let Err(error) = crate::linux_rename::rename_path_no_replace(&temporary, &checkpoint_root) {
        if let Err(cleanup_error) = remove_tree(&temporary) {
            return Err(add_cleanup_result(checkpoint_error(error.to_string()), Err(cleanup_error)));
        }
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            return validate_existing_checkpoint(
                checkpoint_root,
                plan,
                manifest,
                expected_stagex_provider_digest_blake3,
                limits,
            );
        }
        return Err(checkpoint_error(format!("publishing checkpoint {}: {error}", checkpoint_root.display())));
    }
    sync_directory(parent)?;
    let checkpoint_digest_blake3 = provider_checkpoint_manifest_digest(manifest).map_err(checkpoint_policy_error)?;
    assert!(checkpoint_root.is_dir());
    Ok(PublishedProviderCheckpoint {
        manifest_path: checkpoint_root.join(PROVIDER_CHECKPOINT_MANIFEST_FILE),
        checkpoint_root,
        checkpoint_digest_blake3,
        lookup_key_blake3: manifest.lookup_key_blake3.clone(),
        disposition: CheckpointPublicationDisposition::Published,
    })
}

fn materialize_checkpoint_candidate(
    temporary: &Path,
    plan: &SourceBuiltFixedPointPlan,
    manifest: &ProviderCheckpointManifest,
    payload_sources: &[CheckpointPayloadSource],
    expected_stagex_provider_digest_blake3: &str,
    limits: ProviderCheckpointLimits,
) -> Result<(), RunError> {
    for source in payload_sources {
        let destination = temporary.join(&source.relative_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                checkpoint_error(format!("creating checkpoint payload parent {}: {error}", parent.display()))
            })?;
        }
        copy_payload(&source.source_path, &destination, source.kind, limits)?;
    }
    write_manifest_create_new(&temporary.join(PROVIDER_CHECKPOINT_MANIFEST_FILE), manifest)?;
    let (observed_manifest, admission) =
        load_and_admit_checkpoint(temporary, plan, expected_stagex_provider_digest_blake3, limits)?;
    if observed_manifest != *manifest {
        return Err(checkpoint_error("materialized checkpoint manifest changed before publication".to_string()));
    }
    assert_eq!(admission.lookup_key_blake3, manifest.lookup_key_blake3);
    Ok(())
}

fn validate_existing_checkpoint(
    checkpoint_root: PathBuf,
    plan: &SourceBuiltFixedPointPlan,
    expected_manifest: &ProviderCheckpointManifest,
    expected_stagex_provider_digest_blake3: &str,
    limits: ProviderCheckpointLimits,
) -> Result<PublishedProviderCheckpoint, RunError> {
    let (observed_manifest, admission) =
        load_and_admit_checkpoint(&checkpoint_root, plan, expected_stagex_provider_digest_blake3, limits)?;
    let expected_digest = provider_checkpoint_manifest_digest(expected_manifest).map_err(checkpoint_policy_error)?;
    if admission.checkpoint_digest_blake3 != expected_digest || observed_manifest != *expected_manifest {
        return Err(checkpoint_error(format!(
            "conflicting promoted provider checkpoint for lookup key {}",
            expected_manifest.lookup_key_blake3
        )));
    }
    Ok(PublishedProviderCheckpoint {
        manifest_path: checkpoint_root.join(PROVIDER_CHECKPOINT_MANIFEST_FILE),
        checkpoint_root,
        checkpoint_digest_blake3: admission.checkpoint_digest_blake3,
        lookup_key_blake3: admission.lookup_key_blake3,
        disposition: CheckpointPublicationDisposition::ExistingIdentical,
    })
}

fn observe_payload_sources(
    sources: &[CheckpointPayloadSource],
    limits: ProviderCheckpointLimits,
) -> Result<Vec<CheckpointPayloadIdentity>, RunError> {
    let mut payloads = sources
        .iter()
        .map(|source| {
            observe_payload(&source.source_path, &source.payload_id, &source.relative_path, source.kind, limits)
        })
        .collect::<Result<Vec<_>, _>>()?;
    payloads.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    Ok(payloads)
}

fn observe_manifest_payloads(
    checkpoint_root: &Path,
    manifest: &ProviderCheckpointManifest,
    limits: ProviderCheckpointLimits,
) -> Result<Vec<CheckpointPayloadIdentity>, RunError> {
    let mut payloads = manifest
        .payloads
        .iter()
        .map(|payload| {
            let relative = Path::new(&payload.relative_path);
            validate_relative_path(relative)?;
            observe_payload(&checkpoint_root.join(relative), &payload.payload_id, relative, payload.kind, limits)
        })
        .collect::<Result<Vec<_>, _>>()?;
    payloads.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    Ok(payloads)
}

fn observe_payload(
    path: &Path,
    payload_id: &str,
    relative_path: &Path,
    kind: CheckpointPayloadKind,
    limits: ProviderCheckpointLimits,
) -> Result<CheckpointPayloadIdentity, RunError> {
    validate_relative_path(relative_path)?;
    let (total_file_bytes, digest_blake3, entry_count) = match kind {
        CheckpointPayloadKind::PreservedTree => {
            let identity = crate::preserved_evidence_tree::hash_preserved_evidence_tree(path, limits.preserved_tree)?;
            (identity.total_file_bytes, identity.digest_blake3, identity.entry_count)
        }
        CheckpointPayloadKind::Directory => {
            let prepared = crate::release_tree_copy::prepare_tree_copy(path)?;
            let entry_count = prepared.entry_count()?;
            let (total_file_bytes, digest_blake3) = prepared.artifact_identity()?;
            (total_file_bytes, digest_blake3, entry_count)
        }
        CheckpointPayloadKind::RegularFile => {
            let (total_file_bytes, digest_blake3) = crate::release_tree_copy::hash_file_nofollow(path)?;
            (total_file_bytes, digest_blake3, SINGLE_FILE_ENTRY_COUNT)
        }
    };
    let relative = relative_path.to_str().ok_or_else(|| {
        checkpoint_error(format!("checkpoint payload path is not UTF-8: {}", relative_path.display()))
    })?;
    Ok(CheckpointPayloadIdentity {
        payload_id: payload_id.to_string(),
        relative_path: relative.to_string(),
        kind,
        digest_blake3,
        total_file_bytes,
        entry_count,
    })
}

fn copy_payload(
    source: &Path,
    destination: &Path,
    kind: CheckpointPayloadKind,
    limits: ProviderCheckpointLimits,
) -> Result<(), RunError> {
    match kind {
        CheckpointPayloadKind::PreservedTree => {
            crate::preserved_evidence_tree::copy_preserved_evidence_tree(source, destination, limits.preserved_tree)?;
        }
        CheckpointPayloadKind::Directory => {
            crate::release_tree_copy::copy_directory_tree(source, destination)?;
        }
        CheckpointPayloadKind::RegularFile => copy_regular_file_nofollow(source, destination)?,
    }
    assert!(destination.exists() || fs::symlink_metadata(destination).is_ok());
    Ok(())
}

#[cfg(unix)]
fn copy_regular_file_nofollow(source: &Path, destination: &Path) -> Result<(), RunError> {
    let observed = fs::symlink_metadata(source)
        .map_err(|error| checkpoint_error(format!("reading checkpoint file {}: {error}", source.display())))?;
    if !observed.is_file() {
        return Err(checkpoint_error(format!("checkpoint payload is not a regular file: {}", source.display())));
    }
    let mut input = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(source)
        .map_err(|error| checkpoint_error(format!("opening checkpoint file {}: {error}", source.display())))?;
    let opened = input
        .metadata()
        .map_err(|error| checkpoint_error(format!("reading opened checkpoint file {}: {error}", source.display())))?;
    if opened.dev() != observed.dev() || opened.ino() != observed.ino() || opened.len() != observed.len() {
        return Err(checkpoint_error(format!("checkpoint file changed before copy: {}", source.display())));
    }
    let mut output =
        fs::OpenOptions::new().write(true).create_new(true).open(destination).map_err(|error| {
            checkpoint_error(format!("creating checkpoint file {}: {error}", destination.display()))
        })?;
    copy_bounded_file(&mut input, &mut output, opened.len(), source)?;
    fs::set_permissions(destination, fs::Permissions::from_mode(observed.mode() & PERMISSION_BITS_MASK)).map_err(
        |error| checkpoint_error(format!("setting checkpoint file mode {}: {error}", destination.display())),
    )?;
    Ok(())
}

#[cfg(not(unix))]
fn copy_regular_file_nofollow(_source: &Path, _destination: &Path) -> Result<(), RunError> {
    Err(checkpoint_error("promoted checkpoint file copy requires Unix".to_string()))
}

fn copy_bounded_file(
    input: &mut fs::File,
    output: &mut fs::File,
    expected_bytes: u64,
    source: &Path,
) -> Result<(), RunError> {
    let mut remaining = expected_bytes;
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    while remaining > 0 {
        let buffer_bytes = u64::try_from(buffer.len())
            .map_err(|_| checkpoint_error("checkpoint copy buffer length does not fit u64".to_string()))?;
        let count_max = usize::try_from(remaining.min(buffer_bytes))
            .map_err(|_| checkpoint_error("checkpoint remaining byte count does not fit usize".to_string()))?;
        let count = input
            .read(&mut buffer[..count_max])
            .map_err(|error| checkpoint_error(format!("reading checkpoint file {}: {error}", source.display())))?;
        if count == 0 {
            return Err(checkpoint_error(format!("checkpoint file shrank during copy: {}", source.display())));
        }
        output
            .write_all(&buffer[..count])
            .map_err(|error| checkpoint_error(format!("writing checkpoint file {}: {error}", source.display())))?;
        remaining = remaining
            .checked_sub(
                u64::try_from(count)
                    .map_err(|_| checkpoint_error("checkpoint copy byte count does not fit u64".to_string()))?,
            )
            .ok_or_else(|| checkpoint_error("checkpoint copy byte count underflowed".to_string()))?;
    }
    let extra = input
        .read(&mut buffer[..1])
        .map_err(|error| checkpoint_error(format!("checking checkpoint file growth {}: {error}", source.display())))?;
    if extra != 0 {
        return Err(checkpoint_error(format!("checkpoint file grew during copy: {}", source.display())));
    }
    output
        .sync_all()
        .map_err(|error| checkpoint_error(format!("synchronizing checkpoint file {}: {error}", source.display())))?;
    assert_eq!(remaining, 0);
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), RunError> {
    let directory = fs::File::open(path)
        .map_err(|error| checkpoint_error(format!("opening checkpoint directory {}: {error}", path.display())))?;
    directory
        .sync_all()
        .map_err(|error| checkpoint_error(format!("synchronizing checkpoint directory {}: {error}", path.display())))
}

fn read_checkpoint_manifest(checkpoint_root: &Path) -> Result<ProviderCheckpointManifest, RunError> {
    let path = checkpoint_root.join(PROVIDER_CHECKPOINT_MANIFEST_FILE);
    let bytes = fs::read(&path)
        .map_err(|error| checkpoint_error(format!("reading checkpoint manifest {}: {error}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| checkpoint_error(format!("parsing checkpoint manifest {}: {error}", path.display())))
}

fn write_manifest_create_new(path: &Path, manifest: &ProviderCheckpointManifest) -> Result<(), RunError> {
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| checkpoint_error(format!("serializing checkpoint manifest: {error}")))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| checkpoint_error(format!("creating checkpoint manifest {}: {error}", path.display())))?;
    file.write_all(&bytes)
        .map_err(|error| checkpoint_error(format!("writing checkpoint manifest {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| checkpoint_error(format!("synchronizing checkpoint manifest {}: {error}", path.display())))?;
    Ok(())
}

fn validate_restore_requests(
    restores: &[CheckpointPayloadRestore],
    manifest: &ProviderCheckpointManifest,
) -> Result<(), RunError> {
    if restores.len() != manifest.payloads.len() {
        return Err(checkpoint_error("checkpoint restore request count does not match manifest".to_string()));
    }
    let mut requests = BTreeMap::new();
    for request in restores {
        if requests.insert(request.payload_id.as_str(), request.kind).is_some() {
            return Err(checkpoint_error(format!("duplicate checkpoint restore request {}", request.payload_id)));
        }
        let payload = payload_by_id(manifest, &request.payload_id)?;
        if payload.kind != request.kind {
            return Err(checkpoint_error(format!("checkpoint restore kind mismatch for {}", request.payload_id)));
        }
    }
    assert_eq!(requests.len(), manifest.payloads.len());
    Ok(())
}

fn payload_by_id<'a>(
    manifest: &'a ProviderCheckpointManifest,
    payload_id: &str,
) -> Result<&'a CheckpointPayloadIdentity, RunError> {
    let mut matches = manifest.payloads.iter().filter(|payload| payload.payload_id == payload_id);
    let payload = matches
        .next()
        .ok_or_else(|| checkpoint_error(format!("checkpoint payload is missing: {payload_id}")))?;
    if matches.next().is_some() {
        return Err(checkpoint_error(format!("checkpoint payload is duplicated: {payload_id}")));
    }
    Ok(payload)
}

fn validate_relative_path(path: &Path) -> Result<(), RunError> {
    let valid = !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| matches!(component, Component::Normal(_)));
    if !valid {
        return Err(checkpoint_error(format!(
            "checkpoint payload path is not a safe relative path: {}",
            path.display()
        )));
    }
    let first = path.components().next();
    if first != Some(Component::Normal(CHECKPOINT_PAYLOAD_ROOT.as_ref())) {
        return Err(checkpoint_error(format!(
            "checkpoint payload path must start with {CHECKPOINT_PAYLOAD_ROOT}: {}",
            path.display()
        )));
    }
    Ok(())
}

fn require_absolute_directory_or_create(path: &Path) -> Result<(), RunError> {
    if !path.is_absolute() {
        return Err(checkpoint_error(format!("checkpoint store must be absolute: {}", path.display())));
    }
    fs::create_dir_all(path)
        .map_err(|error| checkpoint_error(format!("creating checkpoint store {}: {error}", path.display())))?;
    if !path.is_dir() {
        return Err(checkpoint_error(format!("checkpoint store is not a directory: {}", path.display())));
    }
    Ok(())
}

fn cleanup_restores(paths: &[PathBuf]) -> Result<(), RunError> {
    for path in paths.iter().rev() {
        remove_tree(path)?;
    }
    Ok(())
}

fn remove_tree(path: &Path) -> Result<(), RunError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(checkpoint_error(format!("reading cleanup path {}: {error}", path.display())));
        }
    };
    if metadata.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| checkpoint_error(format!("removing checkpoint directory {}: {error}", path.display())))
    } else {
        fs::remove_file(path)
            .map_err(|error| checkpoint_error(format!("removing checkpoint path {}: {error}", path.display())))
    }
}

fn add_cleanup_result(primary: RunError, cleanup: Result<(), RunError>) -> RunError {
    match cleanup {
        Ok(()) => primary,
        Err(cleanup_error) => checkpoint_error(format!("{primary}; cleanup failed: {cleanup_error}")),
    }
}

fn is_lower_blake3(value: &str) -> bool {
    value.len() == blake3::OUT_LEN.saturating_mul(2)
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn checkpoint_policy_error(error: crate::source_built_fixed_point_checkpoint::CheckpointError) -> RunError {
    checkpoint_error(error.to_string())
}

fn checkpoint_error(message: impl Into<String>) -> RunError {
    RunError::Internal(format!("source-built provider checkpoint: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_built_fixed_point::ProofOutputRole;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_PLAN;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ACTION_RECONCILIATION;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_ADMISSION;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_PROVIDER;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_NATIVE_TRANSCRIPT;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_RUST_PROVIDER;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_PROVIDER;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_TRANSITION;
    use crate::source_built_fixed_point_checkpoint::PAYLOAD_TOOLCHAIN_CLOSURE;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TEST_TOTAL_BYTES_MAX: u64 = 1_048_576;

    #[test]
    #[cfg(unix)]
    fn promoted_checkpoint_publishes_idempotently_and_restores_into_fresh_paths() {
        let fixture = CheckpointFixture::new();
        let first = fixture.publish();
        let second = fixture.publish();
        let admitted = admit_provider_checkpoint_store(&fixture.store, &fixture.plan, DIGEST_B, fixture.limits)
            .unwrap()
            .expect("published checkpoint");
        let restored = restore_provider_checkpoint(admitted, &fixture.restore_requests(), fixture.limits).unwrap();

        assert_eq!(first.disposition, CheckpointPublicationDisposition::Published);
        assert_eq!(second.disposition, CheckpointPublicationDisposition::ExistingIdentical);
        assert_eq!(first.checkpoint_digest_blake3, second.checkpoint_digest_blake3);
        assert_eq!(restored.admission.checkpoint_digest_blake3, first.checkpoint_digest_blake3);
        assert_eq!(fs::read(fixture.restore_root.join("rust-provider/rustc")).unwrap(), b"rust-provider");
        assert_eq!(
            fs::read_link(fixture.restore_root.join("stagex-transition/opaque-link")).unwrap(),
            Path::new("../negative-fixture")
        );
    }

    #[test]
    fn promoted_checkpoint_rejects_modified_payload_and_partial_candidate() {
        let fixture = CheckpointFixture::new();
        let published = fixture.publish();
        let transcript = published.checkpoint_root.join("payload/evidence/native-transcript.json");
        fs::write(&transcript, b"modified").unwrap();
        let modified =
            admit_provider_checkpoint_store(&fixture.store, &fixture.plan, DIGEST_B, fixture.limits).unwrap_err();
        fs::write(&transcript, b"native-transcript").unwrap();
        let lookup_root = published.checkpoint_root.parent().unwrap();
        fs::create_dir(lookup_root.join("partial-candidate")).unwrap();
        let partial =
            admit_provider_checkpoint_store(&fixture.store, &fixture.plan, DIGEST_B, fixture.limits).unwrap_err();

        assert!(modified.to_string().contains("payload observations"));
        assert!(partial.to_string().contains("partial or unknown"));
    }

    #[test]
    fn promoted_checkpoint_rejects_semantically_conflicting_candidates() {
        let fixture = CheckpointFixture::new();
        let published = fixture.publish();
        let lookup_root = published.checkpoint_root.parent().unwrap();
        let candidate_temp = fixture.store.parent().unwrap().join("conflicting-candidate");
        crate::preserved_evidence_tree::copy_preserved_evidence_tree(
            &published.checkpoint_root,
            &candidate_temp,
            fixture.limits.preserved_tree,
        )
        .unwrap();
        let mut manifest = read_checkpoint_manifest(&candidate_temp).unwrap();
        let rust_payload =
            manifest.payloads.iter_mut().find(|payload| payload.payload_id == PAYLOAD_RUST_PROVIDER).unwrap();
        let rust_relative = PathBuf::from(&rust_payload.relative_path);
        fs::write(candidate_temp.join(&rust_relative).join("rustc"), b"conflicting-rust-provider").unwrap();
        let changed = observe_payload(
            &candidate_temp.join(&rust_relative),
            PAYLOAD_RUST_PROVIDER,
            &rust_relative,
            CheckpointPayloadKind::Directory,
            fixture.limits,
        )
        .unwrap();
        *rust_payload = changed.clone();
        let rust_stage = manifest
            .stages
            .iter_mut()
            .find(|stage| stage.output_role == ProofOutputRole::FullSourceRustProvider)
            .unwrap();
        rust_stage.output_digest_blake3 = changed.digest_blake3.clone();
        rust_stage.semantic_output_digest_blake3 = changed.digest_blake3.clone();
        rust_stage.payload_digest_blake3 = changed.digest_blake3;
        fs::remove_file(candidate_temp.join(PROVIDER_CHECKPOINT_MANIFEST_FILE)).unwrap();
        write_manifest_create_new(&candidate_temp.join(PROVIDER_CHECKPOINT_MANIFEST_FILE), &manifest).unwrap();
        let conflicting_digest = provider_checkpoint_manifest_digest(&manifest).unwrap();
        fs::rename(&candidate_temp, lookup_root.join(conflicting_digest)).unwrap();

        let error =
            admit_provider_checkpoint_store(&fixture.store, &fixture.plan, DIGEST_B, fixture.limits).unwrap_err();

        assert!(error.to_string().contains("conflicting promoted provider checkpoints"));
        assert!(!error.to_string().contains("payload observations"));
    }

    struct CheckpointFixture {
        _temp: tempfile::TempDir,
        store: PathBuf,
        source_root: PathBuf,
        restore_root: PathBuf,
        plan: SourceBuiltFixedPointPlan,
        limits: ProviderCheckpointLimits,
    }

    impl CheckpointFixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let source_root = temp.path().join("source");
            let restore_root = temp.path().join("restore");
            fs::create_dir_all(source_root.join("stagex-transition")).unwrap();
            fs::write(source_root.join("stagex-transition/report"), b"transition").unwrap();
            #[cfg(unix)]
            std::os::unix::fs::symlink("../negative-fixture", source_root.join("stagex-transition/opaque-link"))
                .unwrap();
            for (directory, file, bytes) in [
                ("stagex-provider", "provider", b"stagex-provider".as_slice()),
                ("native-provider", "cc", b"native-provider".as_slice()),
                ("rust-provider", "rustc", b"rust-provider".as_slice()),
            ] {
                fs::create_dir_all(source_root.join(directory)).unwrap();
                fs::write(source_root.join(directory).join(file), bytes).unwrap();
            }
            fs::create_dir_all(source_root.join("evidence")).unwrap();
            fs::write(source_root.join("evidence/native-admission.json"), b"native-admission").unwrap();
            fs::write(source_root.join("evidence/native-transcript.json"), b"native-transcript").unwrap();
            fs::write(source_root.join("evidence/toolchain.json"), b"toolchain").unwrap();
            fs::write(source_root.join("evidence/native-action-plan.json"), b"native-action-plan").unwrap();
            fs::write(source_root.join("evidence/native-action-reconciliation.json"), b"native-action-reconciliation")
                .unwrap();
            fs::create_dir_all(restore_root.join("native-store")).unwrap();
            fs::create_dir_all(restore_root.join("evidence")).unwrap();
            Self {
                store: temp.path().join("checkpoint-store"),
                source_root,
                restore_root,
                plan: crate::source_built_fixed_point_checkpoint::checkpoint_test_plan(DIGEST_A, DIGEST_B),
                limits: provider_checkpoint_limits(TEST_TOTAL_BYTES_MAX),
                _temp: temp,
            }
        }

        fn publish(&self) -> PublishedProviderCheckpoint {
            publish_provider_checkpoint(
                &self.store,
                &self.plan,
                self.stage_observations(),
                &self.payload_sources(),
                DIGEST_B,
                self.limits,
            )
            .unwrap()
        }

        fn stage_observations(&self) -> Vec<ProviderCheckpointStageObservation> {
            let transition = crate::preserved_evidence_tree::hash_preserved_evidence_tree(
                &self.source_root.join("stagex-transition"),
                self.limits.preserved_tree,
            )
            .unwrap()
            .digest_blake3;
            let stagex_payload =
                crate::release_tree_copy::hash_directory_tree(&self.source_root.join("stagex-provider")).unwrap().1;
            let native_payload =
                crate::release_tree_copy::hash_directory_tree(&self.source_root.join("native-provider")).unwrap().1;
            let rust_payload =
                crate::release_tree_copy::hash_directory_tree(&self.source_root.join("rust-provider")).unwrap().1;
            vec![
                observation(ProofOutputRole::StagexTransition, &transition, &transition, &transition),
                observation(ProofOutputRole::StagexProvider, DIGEST_B, DIGEST_B, &stagex_payload),
                observation(ProofOutputRole::FullSourceNativeProvider, DIGEST_B, DIGEST_B, &native_payload),
                observation(ProofOutputRole::FullSourceRustProvider, &rust_payload, &rust_payload, &rust_payload),
            ]
        }

        fn payload_sources(
            &self,
        ) -> [CheckpointPayloadSource; crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_PAYLOAD_COUNT]
        {
            [
                source(
                    PAYLOAD_STAGEX_TRANSITION,
                    &self.source_root.join("stagex-transition"),
                    "payload/stagex-transition",
                    CheckpointPayloadKind::PreservedTree,
                ),
                source(
                    PAYLOAD_STAGEX_PROVIDER,
                    &self.source_root.join("stagex-provider"),
                    "payload/stagex-provider",
                    CheckpointPayloadKind::Directory,
                ),
                source(
                    PAYLOAD_NATIVE_PROVIDER,
                    &self.source_root.join("native-provider"),
                    "payload/native-provider",
                    CheckpointPayloadKind::Directory,
                ),
                source(
                    PAYLOAD_RUST_PROVIDER,
                    &self.source_root.join("rust-provider"),
                    "payload/rust-provider",
                    CheckpointPayloadKind::Directory,
                ),
                source(
                    PAYLOAD_NATIVE_ADMISSION,
                    &self.source_root.join("evidence/native-admission.json"),
                    "payload/evidence/native-admission.json",
                    CheckpointPayloadKind::RegularFile,
                ),
                source(
                    PAYLOAD_NATIVE_TRANSCRIPT,
                    &self.source_root.join("evidence/native-transcript.json"),
                    "payload/evidence/native-transcript.json",
                    CheckpointPayloadKind::RegularFile,
                ),
                source(
                    PAYLOAD_TOOLCHAIN_CLOSURE,
                    &self.source_root.join("evidence/toolchain.json"),
                    "payload/evidence/toolchain.json",
                    CheckpointPayloadKind::RegularFile,
                ),
                source(
                    PAYLOAD_NATIVE_ACTION_PLAN,
                    &self.source_root.join("evidence/native-action-plan.json"),
                    "payload/evidence/native-action-plan.json",
                    CheckpointPayloadKind::RegularFile,
                ),
                source(
                    PAYLOAD_NATIVE_ACTION_RECONCILIATION,
                    &self.source_root.join("evidence/native-action-reconciliation.json"),
                    "payload/evidence/native-action-reconciliation.json",
                    CheckpointPayloadKind::RegularFile,
                ),
            ]
        }

        fn restore_requests(
            &self,
        ) -> [CheckpointPayloadRestore; crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_PAYLOAD_COUNT]
        {
            [
                restore(
                    PAYLOAD_STAGEX_TRANSITION,
                    &self.restore_root.join("stagex-transition"),
                    CheckpointPayloadKind::PreservedTree,
                ),
                restore(
                    PAYLOAD_STAGEX_PROVIDER,
                    &self.restore_root.join("native-store/stagex-provider"),
                    CheckpointPayloadKind::Directory,
                ),
                restore(
                    PAYLOAD_NATIVE_PROVIDER,
                    &self.restore_root.join("native-store/native-provider"),
                    CheckpointPayloadKind::Directory,
                ),
                restore(
                    PAYLOAD_RUST_PROVIDER,
                    &self.restore_root.join("rust-provider"),
                    CheckpointPayloadKind::Directory,
                ),
                restore(
                    PAYLOAD_NATIVE_ADMISSION,
                    &self.restore_root.join("evidence/native-admission.json"),
                    CheckpointPayloadKind::RegularFile,
                ),
                restore(
                    PAYLOAD_NATIVE_TRANSCRIPT,
                    &self.restore_root.join("evidence/native-transcript.json"),
                    CheckpointPayloadKind::RegularFile,
                ),
                restore(
                    PAYLOAD_TOOLCHAIN_CLOSURE,
                    &self.restore_root.join("evidence/toolchain.json"),
                    CheckpointPayloadKind::RegularFile,
                ),
                restore(
                    PAYLOAD_NATIVE_ACTION_PLAN,
                    &self.restore_root.join("evidence/native-action-plan.json"),
                    CheckpointPayloadKind::RegularFile,
                ),
                restore(
                    PAYLOAD_NATIVE_ACTION_RECONCILIATION,
                    &self.restore_root.join("evidence/native-action-reconciliation.json"),
                    CheckpointPayloadKind::RegularFile,
                ),
            ]
        }
    }

    fn source(
        payload_id: &str,
        source_path: &Path,
        relative_path: &str,
        kind: CheckpointPayloadKind,
    ) -> CheckpointPayloadSource {
        CheckpointPayloadSource {
            payload_id: payload_id.to_string(),
            source_path: source_path.to_path_buf(),
            relative_path: PathBuf::from(relative_path),
            kind,
        }
    }

    fn restore(payload_id: &str, destination_path: &Path, kind: CheckpointPayloadKind) -> CheckpointPayloadRestore {
        CheckpointPayloadRestore {
            payload_id: payload_id.to_string(),
            destination_path: destination_path.to_path_buf(),
            kind,
        }
    }

    fn observation(
        output_role: ProofOutputRole,
        output_digest_blake3: &str,
        semantic_output_digest_blake3: &str,
        payload_digest_blake3: &str,
    ) -> ProviderCheckpointStageObservation {
        ProviderCheckpointStageObservation {
            output_role,
            output_digest_blake3: output_digest_blake3.to_string(),
            semantic_output_digest_blake3: semantic_output_digest_blake3.to_string(),
            payload_digest_blake3: payload_digest_blake3.to_string(),
            execution_evidence_digest_blake3: DIGEST_A.to_string(),
            producer_executable_digest_blake3: DIGEST_A.to_string(),
        }
    }
}
