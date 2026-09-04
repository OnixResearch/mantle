use super::*;

const DEV_RESUME_OBJECTS_SUBDIR: &str = "dev-resume-objects";
const DEV_RESUME_OBJECT_TEMP_PREFIX: &str = ".dev-resume-object-tmp-";

pub(super) fn publish_tree_object(
    cache: &Path,
    source: &Path,
    payload_id: &str,
    destination_relative_path: &str,
    limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<crunch_dev_resume_core::ResumePayloadBinding, RunError> {
    let binding = observe_tree(source, payload_id, destination_relative_path, limits)?;
    let parent = cache.join(DEV_RESUME_OBJECTS_SUBDIR);
    fs::create_dir_all(&parent)
        .map_err(|error| proof_error(format!("creating dev resume object store {}: {error}", parent.display())))?;
    let destination = parent.join(&binding.digest_blake3);
    if destination.exists() {
        validate_existing_object(&destination, &binding, limits)?;
        return Ok(binding);
    }
    let temporary =
        parent.join(format!("{DEV_RESUME_OBJECT_TEMP_PREFIX}{}-{}", binding.digest_blake3, std::process::id()));
    remove_path(&temporary)?;
    crate::preserved_evidence_tree::copy_preserved_evidence_tree(source, &temporary, limits)?;
    validate_existing_object(&temporary, &binding, limits)?;
    if let Err(error) = crate::linux_rename::rename_path_no_replace(&temporary, &destination) {
        let cleanup = remove_path(&temporary);
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            cleanup?;
            validate_existing_object(&destination, &binding, limits)?;
            return Ok(binding);
        }
        return Err(add_cleanup_error(
            proof_error(format!("publishing dev resume object {}: {error}", destination.display())),
            cleanup,
        ));
    }
    sync_directory(&parent)?;
    debug_assert!(destination.is_dir());
    debug_assert!(!binding.digest_blake3.is_empty());
    Ok(binding)
}

pub(super) fn object_path(cache: &Path, digest_blake3: &str) -> PathBuf {
    cache.join(DEV_RESUME_OBJECTS_SUBDIR).join(digest_blake3)
}

pub(super) fn observe_tree_object(
    cache: &Path,
    declared: &crunch_dev_resume_core::ResumePayloadBinding,
    limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<crunch_dev_resume_core::ResumePayloadBinding, RunError> {
    let path = object_path(cache, &declared.digest_blake3);
    observe_tree(&path, &declared.payload_id, &declared.destination_relative_path, limits)
}

pub(super) fn restore_tree_object(
    cache: &Path,
    binding: &crunch_dev_resume_core::ResumePayloadBinding,
    destination: &Path,
    limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<(), RunError> {
    if destination.exists() {
        return Err(proof_error(format!("dev resume destination exists: {}", destination.display())));
    }
    let source = object_path(cache, &binding.digest_blake3);
    crate::preserved_evidence_tree::copy_preserved_evidence_tree(&source, destination, limits)?;
    let observed = observe_tree(destination, &binding.payload_id, &binding.destination_relative_path, limits)?;
    if observed != *binding {
        remove_path(destination)?;
        return Err(proof_error(format!("restored dev resume payload changed: {}", binding.payload_id)));
    }
    debug_assert!(destination.is_dir());
    debug_assert_eq!(observed.digest_blake3, binding.digest_blake3);
    Ok(())
}

fn observe_tree(
    path: &Path,
    payload_id: &str,
    destination_relative_path: &str,
    limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<crunch_dev_resume_core::ResumePayloadBinding, RunError> {
    let identity = crate::preserved_evidence_tree::hash_preserved_evidence_tree(path, limits)?;
    if identity.total_file_bytes == 0 || identity.entry_count == 0 {
        return Err(proof_error(format!("dev resume payload is empty: {payload_id}")));
    }
    Ok(crunch_dev_resume_core::ResumePayloadBinding {
        payload_id: payload_id.to_string(),
        destination_relative_path: destination_relative_path.to_string(),
        kind: crunch_dev_resume_core::ResumePayloadKind::PreservedTree,
        digest_blake3: identity.digest_blake3,
        total_file_bytes: identity.total_file_bytes,
        entry_count: identity.entry_count,
    })
}

fn validate_existing_object(
    path: &Path,
    expected: &crunch_dev_resume_core::ResumePayloadBinding,
    limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<(), RunError> {
    let observed = observe_tree(path, &expected.payload_id, &expected.destination_relative_path, limits)?;
    if observed != *expected {
        return Err(proof_error(format!("conflicting dev resume object: {}", path.display())));
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), RunError> {
    let directory = fs::File::open(path)
        .map_err(|error| proof_error(format!("opening dev resume directory {}: {error}", path.display())))?;
    directory
        .sync_all()
        .map_err(|error| proof_error(format!("synchronizing dev resume directory {}: {error}", path.display())))
}

pub(super) fn remove_path(path: &Path) -> Result<(), RunError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path)
            .map_err(|error| proof_error(format!("removing dev resume directory {}: {error}", path.display()))),
        Ok(_) => fs::remove_file(path)
            .map_err(|error| proof_error(format!("removing dev resume path {}: {error}", path.display()))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(proof_error(format!("reading dev resume cleanup path {}: {error}", path.display()))),
    }
}

fn add_cleanup_error(primary: RunError, cleanup: Result<(), RunError>) -> RunError {
    match cleanup {
        Ok(()) => primary,
        Err(error) => proof_error(format!("{primary}; cleanup failed: {error}")),
    }
}
