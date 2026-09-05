//! Dev-only prefix manifests reference shared, remeasured payload objects.

use super::*;
use crate::source_built_fixed_point_checkpoint::DEV_PROVIDER_PREFIX_SCHEMA;
use crate::source_built_fixed_point_checkpoint::build_dev_provider_prefix_manifest;

const PREFIXES_SUBDIR: &str = "dev-provider-prefixes";
const OBJECTS_SUBDIR: &str = "dev-provider-objects";
const OBJECT_VALUE: &str = "value";
const TEMPORARY_PREFIX: &str = ".dev-prefix-tmp-";
const MANIFEST_BYTES_MAX: u64 = 65_536;

pub(crate) fn publish_dev_provider_prefix(
    store: &Path,
    plan: &SourceBuiltFixedPointPlan,
    observations: Vec<ProviderCheckpointStageObservation>,
    sources: &[CheckpointPayloadSource],
    expected_stagex_digest: &str,
    limits: ProviderCheckpointLimits,
) -> Result<PublishedProviderCheckpoint, RunError> {
    let payloads = observe_payload_sources(sources, limits)?;
    let manifest = build_dev_provider_prefix_manifest(plan, observations, payloads).map_err(checkpoint_policy_error)?;
    let digest = provider_checkpoint_manifest_digest(&manifest).map_err(checkpoint_policy_error)?;
    require_absolute_directory_or_create(store)?;
    require_plain_directory(store)?;
    for source in sources {
        let payload = payload_by_id(&manifest, &source.payload_id)?;
        publish_object(store, source, payload, limits)?;
    }
    let parent = prefix_parent(store, plan)?;
    let destination = parent.join(&digest);
    if destination.try_exists().map_err(|error| checkpoint_error(error.to_string()))? {
        return validate_publication(store, plan, &manifest, expected_stagex_digest, limits, false);
    }
    create_plain_parent(&parent)?;
    let temporary = create_temporary(&parent, &digest)?;
    let written = write_manifest_create_new(&temporary.join(PROVIDER_CHECKPOINT_MANIFEST_FILE), &manifest);
    if let Err(error) = written {
        return Err(add_cleanup_result(error, remove_tree(&temporary)));
    }
    let published = commit_directory(&temporary, &destination)?;
    validate_publication(store, plan, &manifest, expected_stagex_digest, limits, published)
}

pub(crate) fn admit_dev_provider_prefix(
    store: &Path,
    plan: &SourceBuiltFixedPointPlan,
    digest: &str,
    expected_stagex_digest: &str,
    limits: ProviderCheckpointLimits,
) -> Result<Option<AdmittedProviderCheckpoint>, RunError> {
    if !is_lower_blake3(digest) {
        return Err(checkpoint_error("dev prefix identity is invalid"));
    }
    let root = prefix_parent(store, plan)?.join(digest);
    match fs::symlink_metadata(&root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(checkpoint_error(format!("reading dev prefix root: {error}"))),
        Ok(_) => require_plain_directory(&root)?,
    }
    require_plain_directory(store)?;
    require_plain_directory(&store.join(PREFIXES_SUBDIR))?;
    require_plain_directory(root.parent().expect("prefix root has a lookup parent"))?;
    let manifest = read_prefix_manifest(&root)?;
    if manifest.schema != DEV_PROVIDER_PREFIX_SCHEMA {
        return Err(checkpoint_error("dev prefix namespace rejects another checkpoint schema"));
    }
    let declared_digest = provider_checkpoint_manifest_digest(&manifest).map_err(checkpoint_policy_error)?;
    if declared_digest != digest {
        return Err(checkpoint_error("dev prefix declared identity does not match its path"));
    }
    let observed = observe_manifest_payloads(&root, &manifest, limits)?;
    let admission = crate::source_built_fixed_point_checkpoint::admit_dev_provider_checkpoint(
        plan,
        &manifest,
        &observed,
        expected_stagex_digest,
    )
    .map_err(checkpoint_policy_error)?;
    if admission.checkpoint_digest_blake3 != digest {
        return Err(checkpoint_error("dev prefix path identity mismatch"));
    }
    Ok(Some(AdmittedProviderCheckpoint {
        checkpoint_root: root,
        manifest,
        admission,
    }))
}

pub(super) fn payload_path(
    checkpoint_root: &Path,
    manifest: &ProviderCheckpointManifest,
    payload: &CheckpointPayloadIdentity,
) -> Result<PathBuf, RunError> {
    let relative = Path::new(&payload.relative_path);
    validate_relative_path(relative)?;
    if manifest.schema != DEV_PROVIDER_PREFIX_SCHEMA {
        return Ok(checkpoint_root.join(relative));
    }
    let lookup = checkpoint_root.parent().ok_or_else(|| checkpoint_error("dev prefix has no lookup parent"))?;
    let namespace = lookup.parent().ok_or_else(|| checkpoint_error("dev prefix has no namespace parent"))?;
    if namespace.file_name() != Some(PREFIXES_SUBDIR.as_ref()) {
        return Err(checkpoint_error("dev prefix object lookup has the wrong namespace"));
    }
    let store = namespace.parent().ok_or_else(|| checkpoint_error("dev prefix has no store parent"))?;
    let object = object_root(store, payload)?;
    require_plain_directory(&store.join(OBJECTS_SUBDIR))?;
    require_plain_directory(object.parent().expect("object root has a kind parent"))?;
    require_plain_directory(&object)?;
    Ok(object.join(OBJECT_VALUE))
}

fn prefix_parent(store: &Path, plan: &SourceBuiltFixedPointPlan) -> Result<PathBuf, RunError> {
    if !store.is_absolute() {
        return Err(checkpoint_error("dev prefix store must be absolute"));
    }
    let lookup = provider_checkpoint_lookup_key(plan).map_err(checkpoint_policy_error)?;
    Ok(store.join(PREFIXES_SUBDIR).join(lookup))
}

fn object_root(store: &Path, payload: &CheckpointPayloadIdentity) -> Result<PathBuf, RunError> {
    if !is_lower_blake3(&payload.digest_blake3) {
        return Err(checkpoint_error("dev object identity is invalid"));
    }
    let kind = match payload.kind {
        CheckpointPayloadKind::PreservedTree => "preserved-tree",
        CheckpointPayloadKind::Directory => "directory",
        CheckpointPayloadKind::RegularFile => "regular-file",
    };
    Ok(store.join(OBJECTS_SUBDIR).join(kind).join(&payload.digest_blake3))
}

fn publish_object(
    store: &Path,
    source: &CheckpointPayloadSource,
    expected: &CheckpointPayloadIdentity,
    limits: ProviderCheckpointLimits,
) -> Result<(), RunError> {
    let destination = object_root(store, expected)?;
    if destination.try_exists().map_err(|error| checkpoint_error(error.to_string()))? {
        return validate_object(&destination, expected, limits);
    }
    let parent = destination.parent().expect("object root has a kind parent");
    create_plain_parent(parent)?;
    let temporary = create_temporary(parent, &expected.digest_blake3)?;
    let copied = copy_payload(&source.source_path, &temporary.join(OBJECT_VALUE), source.kind, limits)
        .and_then(|()| validate_object(&temporary, expected, limits));
    if let Err(error) = copied {
        return Err(add_cleanup_result(error, remove_tree(&temporary)));
    }
    commit_directory(&temporary, &destination)?;
    validate_object(&destination, expected, limits)
}

fn validate_object(
    root: &Path,
    expected: &CheckpointPayloadIdentity,
    limits: ProviderCheckpointLimits,
) -> Result<(), RunError> {
    require_plain_directory(root)?;
    let observed = observe_payload(
        &root.join(OBJECT_VALUE),
        &expected.payload_id,
        Path::new(&expected.relative_path),
        expected.kind,
        limits,
    )?;
    if observed != *expected {
        return Err(checkpoint_error("dev prefix payload object changed"));
    }
    Ok(())
}

fn validate_publication(
    store: &Path,
    plan: &SourceBuiltFixedPointPlan,
    expected: &ProviderCheckpointManifest,
    expected_stagex_digest: &str,
    limits: ProviderCheckpointLimits,
    published: bool,
) -> Result<PublishedProviderCheckpoint, RunError> {
    let digest = provider_checkpoint_manifest_digest(expected).map_err(checkpoint_policy_error)?;
    let admitted = admit_dev_provider_prefix(store, plan, &digest, expected_stagex_digest, limits)?
        .ok_or_else(|| checkpoint_error("published dev prefix is missing"))?;
    if admitted.manifest != *expected {
        return Err(checkpoint_error("published dev prefix manifest changed"));
    }
    Ok(PublishedProviderCheckpoint {
        manifest_path: admitted.checkpoint_root.join(PROVIDER_CHECKPOINT_MANIFEST_FILE),
        checkpoint_root: admitted.checkpoint_root,
        checkpoint_digest_blake3: admitted.admission.checkpoint_digest_blake3,
        lookup_key_blake3: admitted.admission.lookup_key_blake3,
        disposition: if published {
            CheckpointPublicationDisposition::Published
        } else {
            CheckpointPublicationDisposition::ExistingIdentical
        },
    })
}

fn create_plain_parent(path: &Path) -> Result<(), RunError> {
    let namespace = path.parent().ok_or_else(|| checkpoint_error("dev prefix parent has no namespace"))?;
    for directory in [namespace, path] {
        match fs::create_dir(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(checkpoint_error(format!("creating dev prefix parent: {error}"))),
        }
        require_plain_directory(directory)?;
    }
    Ok(())
}

fn require_plain_directory(path: &Path) -> Result<(), RunError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| checkpoint_error(format!("reading dev directory {}: {error}", path.display())))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(checkpoint_error("dev prefix directory must not be a symlink"));
    }
    Ok(())
}

fn create_temporary(parent: &Path, digest: &str) -> Result<PathBuf, RunError> {
    let path = parent.join(format!("{TEMPORARY_PREFIX}{digest}-{}", std::process::id()));
    fs::create_dir(&path)
        .map_err(|error| checkpoint_error(format!("creating dev prefix temporary directory: {error}")))?;
    Ok(path)
}

fn commit_directory(temporary: &Path, destination: &Path) -> Result<bool, RunError> {
    sync_directory(temporary)?;
    match crate::linux_rename::rename_path_no_replace(temporary, destination) {
        Ok(()) => {
            sync_directory(destination.parent().expect("published directory has a parent"))?;
            Ok(true)
        }
        Err(error) => {
            let already_exists = error.kind() == std::io::ErrorKind::AlreadyExists;
            let cleanup = remove_tree(temporary);
            if already_exists {
                cleanup?;
                return Ok(false);
            }
            Err(add_cleanup_result(checkpoint_error(format!("publishing dev prefix directory: {error}")), cleanup))
        }
    }
}

#[cfg(unix)]
fn read_prefix_manifest(root: &Path) -> Result<ProviderCheckpointManifest, RunError> {
    use std::os::unix::fs::OpenOptionsExt;
    let path = root.join(PROVIDER_CHECKPOINT_MANIFEST_FILE);
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
        .map_err(|error| checkpoint_error(format!("opening dev prefix manifest: {error}")))?;
    let metadata = file
        .metadata()
        .map_err(|error| checkpoint_error(format!("observing dev prefix manifest: {error}")))?;
    if !metadata.is_file() || metadata.len() > MANIFEST_BYTES_MAX {
        return Err(checkpoint_error("dev prefix manifest type or size is invalid"));
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len()).map_err(|_| checkpoint_error("dev prefix manifest size does not fit usize"))?,
    );
    file.take(MANIFEST_BYTES_MAX + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| checkpoint_error(format!("reading dev prefix manifest: {error}")))?;
    if u64::try_from(bytes.len()).ok() != Some(metadata.len()) {
        return Err(checkpoint_error("dev prefix manifest changed while reading"));
    }
    serde_json::from_slice(&bytes).map_err(|error| checkpoint_error(format!("parsing dev prefix manifest: {error}")))
}

#[cfg(not(unix))]
fn read_prefix_manifest(_root: &Path) -> Result<ProviderCheckpointManifest, RunError> {
    Err(checkpoint_error("dev prefix admission requires no-follow file reads"))
}

#[cfg(test)]
mod tests;
