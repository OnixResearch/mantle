use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::CurrentReleasePointerPlan;
use crunch_release_core::CurrentReleasePointerRequest;
use crunch_release_core::ImmutableReleaseEvidence;
use crunch_release_core::ImmutableReleaseEvidenceInput;
use crunch_release_core::ImmutableReleaseObjectDisposition;
use crunch_release_core::ImmutableReleaseObjectPlan;
use crunch_release_core::ImmutableReleaseObjectRequest;
use crunch_release_core::RELEASE_OBJECT_BYTES_MAX;
use crunch_release_core::ReleaseMetadataField;
use crunch_release_core::build_immutable_release_evidence;
use crunch_release_core::immutable_release_evidence_canonical_bytes;
use crunch_release_core::plan_current_release_pointer;
use crunch_release_core::plan_immutable_release_object;

use crate::errors::RunError;

const CURRENT_POINTER_BYTES_COUNT: usize = 65;
const CURRENT_POINTER_DIGEST_BYTES_COUNT: usize = 64;
const POINTER_TRAILING_NEWLINE: u8 = b'\n';
const READ_OVERFLOW_SENTINEL_BYTES_COUNT: u64 = 1;
const IMMUTABLE_RELEASE_EVIDENCE_BYTES_MAX: u64 = 1_048_576;

#[derive(Debug, Clone)]
pub(crate) struct ImmutableReleasePublishRequest {
    pub object_path: PathBuf,
    pub current_pointer_path: PathBuf,
    pub evidence_path: PathBuf,
    pub object_bytes: Vec<u8>,
    pub declared_identity_blake3: String,
    pub release_metadata: Vec<ReleaseMetadataField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImmutableReleasePublishOutcome {
    pub object_path: PathBuf,
    pub current_pointer_path: PathBuf,
    pub evidence_path: PathBuf,
    pub evidence: ImmutableReleaseEvidence,
}

struct PreparedImmutableRelease {
    object_plan: ImmutableReleaseObjectPlan,
    object_bytes: Vec<u8>,
    release_metadata: Vec<ReleaseMetadataField>,
}

// r[impl mantle.release.object]
// r[impl mantle.release.pointer]
// r[impl mantle.release.evidence]
// r[impl mantle.release.boundary]
pub(crate) fn publish_immutable_release(
    request: ImmutableReleasePublishRequest,
) -> Result<ImmutableReleasePublishOutcome, RunError> {
    let prepared = prepare_immutable_release(&request)?;
    publish_release_object(&request.object_path, &prepared.object_plan, &prepared.object_bytes)?;
    let pointer_plan = switch_current_release_pointer(
        &request.current_pointer_path,
        &request.object_path,
        prepared.object_plan.object_identity_blake3.clone(),
    )?;
    let evidence = build_immutable_release_evidence(ImmutableReleaseEvidenceInput {
        object_plan: prepared.object_plan,
        pointer_plan,
        release_metadata: prepared.release_metadata,
    })
    .map_err(core_error)?;
    publish_release_evidence(&request.evidence_path, evidence.clone())?;
    assert_eq!(evidence.current_pointer_identity_blake3, request.declared_identity_blake3);
    assert!(request.object_path.is_file());
    Ok(ImmutableReleasePublishOutcome {
        object_path: request.object_path,
        current_pointer_path: request.current_pointer_path,
        evidence_path: request.evidence_path,
        evidence,
    })
}

fn prepare_immutable_release(request: &ImmutableReleasePublishRequest) -> Result<PreparedImmutableRelease, RunError> {
    validate_content_addressed_object_path(&request.object_path, &request.declared_identity_blake3)?;
    require_existing_parent(&request.object_path, "release object")?;
    require_existing_parent(&request.current_pointer_path, "current release pointer")?;
    require_existing_parent(&request.evidence_path, "immutable release evidence")?;
    let existing_object_bytes = read_optional_bounded_regular(
        &request.object_path,
        RELEASE_OBJECT_BYTES_MAX,
        "published immutable release object",
    )?;
    let object_plan = plan_immutable_release_object(ImmutableReleaseObjectRequest {
        object_bytes: request.object_bytes.clone(),
        declared_identity_blake3: request.declared_identity_blake3.clone(),
        existing_object_bytes,
    })
    .map_err(core_error)?;
    let current_identity_blake3 = read_current_pointer(&request.current_pointer_path)?;
    let preflight_pointer_plan = plan_current_release_pointer(CurrentReleasePointerRequest {
        target_identity_blake3: object_plan.object_identity_blake3.clone(),
        target_object_exists: true,
        current_identity_blake3,
    })
    .map_err(core_error)?;
    let preflight_evidence = build_immutable_release_evidence(ImmutableReleaseEvidenceInput {
        object_plan: object_plan.clone(),
        pointer_plan: preflight_pointer_plan.clone(),
        release_metadata: request.release_metadata.clone(),
    })
    .map_err(core_error)?;
    preflight_release_evidence_path(&request.evidence_path, preflight_evidence)?;
    assert_eq!(object_plan.object_identity_blake3, request.declared_identity_blake3);
    assert_eq!(preflight_pointer_plan.target_identity_blake3, request.declared_identity_blake3);
    Ok(PreparedImmutableRelease {
        object_plan,
        object_bytes: request.object_bytes.clone(),
        release_metadata: request.release_metadata.clone(),
    })
}

fn validate_content_addressed_object_path(path: &Path, identity_blake3: &str) -> Result<(), RunError> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| shell_error(format!("release object path has no UTF-8 file name: {}", path.display())))?;
    if file_name != identity_blake3 {
        return Err(shell_error(format!(
            "release object path must end with its BLAKE3 identity {identity_blake3}: {}",
            path.display()
        )));
    }
    assert!(!file_name.is_empty());
    assert_eq!(file_name, identity_blake3);
    Ok(())
}

fn require_existing_parent(path: &Path, label: &str) -> Result<(), RunError> {
    let parent = path
        .parent()
        .ok_or_else(|| shell_error(format!("{label} path has no parent: {}", path.display())))?;
    let metadata = std::fs::symlink_metadata(parent)
        .map_err(|error| shell_error(format!("reading {label} parent {}: {error}", parent.display())))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(shell_error(format!("{label} parent must be a non-symlink directory: {}", parent.display())));
    }
    assert!(parent.components().next().is_some());
    assert!(metadata.is_dir());
    Ok(())
}

fn publish_release_object(path: &Path, plan: &ImmutableReleaseObjectPlan, bytes: &[u8]) -> Result<(), RunError> {
    match plan.disposition {
        ImmutableReleaseObjectDisposition::AlreadyPublishedIdentical => return Ok(()),
        ImmutableReleaseObjectDisposition::PublishNew => {}
    }
    match write_new_synced_file(path, bytes) {
        Ok(()) => sync_parent(path, "release object"),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            validate_concurrent_release_object(path, plan, bytes)
        }
        Err(error) => Err(shell_error(format!("publishing immutable release object {}: {error}", path.display()))),
    }
}

fn validate_concurrent_release_object(
    path: &Path,
    plan: &ImmutableReleaseObjectPlan,
    bytes: &[u8],
) -> Result<(), RunError> {
    let existing = read_optional_bounded_regular(path, RELEASE_OBJECT_BYTES_MAX, "concurrent release object")?
        .ok_or_else(|| shell_error("concurrent release object disappeared before validation".to_string()))?;
    let concurrent_plan = plan_immutable_release_object(ImmutableReleaseObjectRequest {
        object_bytes: bytes.to_vec(),
        declared_identity_blake3: plan.object_identity_blake3.clone(),
        existing_object_bytes: Some(existing),
    })
    .map_err(core_error)?;
    if concurrent_plan.disposition != ImmutableReleaseObjectDisposition::AlreadyPublishedIdentical {
        return Err(shell_error("concurrent release object was not identical".to_string()));
    }
    assert_eq!(concurrent_plan.object_identity_blake3, plan.object_identity_blake3);
    assert_eq!(concurrent_plan.object_size_bytes, plan.object_size_bytes);
    Ok(())
}

fn switch_current_release_pointer(
    pointer_path: &Path,
    object_path: &Path,
    target_identity_blake3: String,
) -> Result<CurrentReleasePointerPlan, RunError> {
    let object_bytes =
        read_optional_bounded_regular(object_path, RELEASE_OBJECT_BYTES_MAX, "release pointer target object")?
            .ok_or_else(|| {
                shell_error(format!("current release pointer target object is missing: {}", object_path.display()))
            })?;
    let object_plan = plan_immutable_release_object(ImmutableReleaseObjectRequest {
        object_bytes,
        declared_identity_blake3: target_identity_blake3.clone(),
        existing_object_bytes: None,
    })
    .map_err(core_error)?;
    let current_identity_blake3 = read_current_pointer(pointer_path)?;
    let pointer_plan = plan_current_release_pointer(CurrentReleasePointerRequest {
        target_identity_blake3,
        target_object_exists: true,
        current_identity_blake3,
    })
    .map_err(core_error)?;
    write_current_pointer(pointer_path, &pointer_plan.target_identity_blake3)?;
    let observed = read_current_pointer(pointer_path)?;
    if observed.as_deref() != Some(pointer_plan.target_identity_blake3.as_str()) {
        return Err(shell_error("current release pointer read-back did not match the selected object".to_string()));
    }
    assert_eq!(object_plan.object_identity_blake3, pointer_plan.target_identity_blake3);
    assert_eq!(observed, Some(pointer_plan.target_identity_blake3.clone()));
    Ok(pointer_plan)
}

fn read_current_pointer(path: &Path) -> Result<Option<String>, RunError> {
    let Some(bytes) = read_optional_bounded_regular(
        path,
        u64::try_from(CURRENT_POINTER_BYTES_COUNT).expect("pointer byte count fits u64"),
        "current release pointer",
    )?
    else {
        return Ok(None);
    };
    if bytes.len() != CURRENT_POINTER_BYTES_COUNT
        || bytes[CURRENT_POINTER_DIGEST_BYTES_COUNT] != POINTER_TRAILING_NEWLINE
    {
        return Err(shell_error(format!("current release pointer has invalid encoding: {}", path.display())));
    }
    let digest = std::str::from_utf8(&bytes[..CURRENT_POINTER_DIGEST_BYTES_COUNT])
        .map_err(|error| shell_error(format!("current release pointer is not UTF-8: {error}")))?
        .to_string();
    plan_current_release_pointer(CurrentReleasePointerRequest {
        target_identity_blake3: digest.clone(),
        target_object_exists: true,
        current_identity_blake3: None,
    })
    .map_err(core_error)?;
    assert_eq!(digest.len(), CURRENT_POINTER_DIGEST_BYTES_COUNT);
    assert_eq!(bytes.len(), CURRENT_POINTER_BYTES_COUNT);
    Ok(Some(digest))
}

fn write_current_pointer(path: &Path, identity_blake3: &str) -> Result<(), RunError> {
    reject_non_regular_existing_path(path, "current release pointer")?;
    let parent = path.parent().ok_or_else(|| shell_error("current release pointer has no parent".to_string()))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| shell_error(format!("creating current release pointer stage: {error}")))?;
    let bytes = format!("{identity_blake3}\n").into_bytes();
    temporary
        .write_all(&bytes)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| shell_error(format!("writing current release pointer stage: {error}")))?;
    temporary.persist(path).map_err(|error| {
        shell_error(format!("atomically replacing current release pointer {}: {}", path.display(), error.error))
    })?;
    sync_parent(path, "current release pointer")?;
    assert_eq!(bytes.len(), CURRENT_POINTER_BYTES_COUNT);
    assert_eq!(bytes[CURRENT_POINTER_DIGEST_BYTES_COUNT], POINTER_TRAILING_NEWLINE);
    Ok(())
}

fn preflight_release_evidence_path(path: &Path, evidence: ImmutableReleaseEvidence) -> Result<(), RunError> {
    let bytes = immutable_release_evidence_canonical_bytes(evidence).map_err(core_error)?;
    let byte_limit = usize::try_from(IMMUTABLE_RELEASE_EVIDENCE_BYTES_MAX)
        .expect("immutable release evidence byte limit fits usize");
    if bytes.len() > byte_limit {
        return Err(shell_error(format!(
            "immutable release evidence exceeds {IMMUTABLE_RELEASE_EVIDENCE_BYTES_MAX} bytes"
        )));
    }
    validate_existing_release_evidence(path, &bytes)
}

fn publish_release_evidence(path: &Path, evidence: ImmutableReleaseEvidence) -> Result<(), RunError> {
    let bytes = immutable_release_evidence_canonical_bytes(evidence).map_err(core_error)?;
    match write_new_synced_file(path, &bytes) {
        Ok(()) => sync_parent(path, "immutable release evidence"),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            validate_existing_release_evidence(path, &bytes)
        }
        Err(error) => Err(shell_error(format!("publishing immutable release evidence {}: {error}", path.display()))),
    }
}

fn validate_existing_release_evidence(path: &Path, expected_bytes: &[u8]) -> Result<(), RunError> {
    let Some(existing) =
        read_optional_bounded_regular(path, IMMUTABLE_RELEASE_EVIDENCE_BYTES_MAX, "immutable release evidence")?
    else {
        return Ok(());
    };
    if existing != expected_bytes {
        return Err(shell_error(format!(
            "immutable release evidence already exists with different bytes: {}",
            path.display()
        )));
    }
    assert_eq!(existing, expected_bytes);
    assert!(!existing.is_empty());
    Ok(())
}

fn write_new_synced_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn read_optional_bounded_regular(path: &Path, bytes_max: u64, label: &str) -> Result<Option<Vec<u8>>, RunError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(shell_error(format!("reading {label} metadata {}: {error}", path.display()))),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(shell_error(format!("{label} must be a non-symlink regular file: {}", path.display())));
    }
    if metadata.len() > bytes_max {
        return Err(shell_error(format!("{label} exceeds {bytes_max} bytes: {}", path.display())));
    }
    let read_limit = bytes_max
        .checked_add(READ_OVERFLOW_SENTINEL_BYTES_COUNT)
        .ok_or_else(|| shell_error(format!("{label} read limit overflowed u64")))?;
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(read_limit).read_to_end(&mut bytes))
        .map_err(|error| shell_error(format!("reading {label} {}: {error}", path.display())))?;
    let observed_size = u64::try_from(bytes.len()).map_err(|_| shell_error(format!("{label} size overflowed u64")))?;
    if observed_size != metadata.len() || observed_size > bytes_max {
        return Err(shell_error(format!("{label} changed during bounded read: {}", path.display())));
    }
    assert!(observed_size <= bytes_max);
    assert_eq!(observed_size, metadata.len());
    Ok(Some(bytes))
}

fn reject_non_regular_existing_path(path: &Path, label: &str) -> Result<(), RunError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(shell_error(format!("reading {label} metadata: {error}"))),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(shell_error(format!("{label} must be absent or a non-symlink regular file: {}", path.display())));
    }
    Ok(())
}

fn sync_parent(path: &Path, label: &str) -> Result<(), RunError> {
    let parent = path.parent().ok_or_else(|| shell_error(format!("{label} path has no parent")))?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| shell_error(format!("synchronizing {label} parent {}: {error}", parent.display())))
}

fn core_error(error: crunch_release_core::ReleaseEvidenceError) -> RunError {
    shell_error(error.to_string())
}

fn shell_error(message: String) -> RunError {
    RunError::Internal(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_OBJECT_BYTES: &[u8] = b"release-object-one";
    const SECOND_OBJECT_BYTES: &[u8] = b"release-object-two";
    const RELEASE_ID_FIELD: &str = "release-id";
    const FIRST_RELEASE_ID: &str = "mantle-1";
    const SECOND_RELEASE_ID: &str = "mantle-2";

    struct Fixture {
        _root: tempfile::TempDir,
        objects: PathBuf,
        evidence: PathBuf,
        current: PathBuf,
    }

    fn fixture() -> Fixture {
        let root = tempfile::tempdir().unwrap();
        let objects = root.path().join("objects");
        let evidence = root.path().join("evidence");
        std::fs::create_dir(&objects).unwrap();
        std::fs::create_dir(&evidence).unwrap();
        let current = root.path().join("current");
        Fixture {
            _root: root,
            objects,
            evidence,
            current,
        }
    }

    fn identity(bytes: &[u8]) -> String {
        blake3::hash(bytes).to_hex().to_string()
    }

    fn request(fixture: &Fixture, bytes: &[u8], release_id: &str) -> ImmutableReleasePublishRequest {
        let identity = identity(bytes);
        ImmutableReleasePublishRequest {
            object_path: fixture.objects.join(&identity),
            current_pointer_path: fixture.current.clone(),
            evidence_path: fixture.evidence.join(format!("{identity}.json")),
            object_bytes: bytes.to_vec(),
            declared_identity_blake3: identity,
            release_metadata: vec![ReleaseMetadataField {
                name: RELEASE_ID_FIELD.to_string(),
                value: release_id.to_string(),
            }],
        }
    }

    // r[verify mantle.release.verification]
    #[test]
    fn shell_publishes_immutable_object_pointer_and_evidence() {
        let fixture = fixture();
        let request = request(&fixture, FIRST_OBJECT_BYTES, FIRST_RELEASE_ID);
        let outcome = publish_immutable_release(request.clone()).expect("publish release");

        assert_eq!(std::fs::read(&outcome.object_path).unwrap(), FIRST_OBJECT_BYTES);
        assert_eq!(read_current_pointer(&outcome.current_pointer_path).unwrap(), Some(identity(FIRST_OBJECT_BYTES)));
        assert!(outcome.evidence_path.is_file());
        assert_eq!(outcome.evidence.object_identity_blake3, identity(FIRST_OBJECT_BYTES));
    }

    // r[verify mantle.release.verification]
    #[test]
    fn shell_switches_and_rolls_back_without_mutating_objects() {
        let fixture = fixture();
        let first = request(&fixture, FIRST_OBJECT_BYTES, FIRST_RELEASE_ID);
        let second = request(&fixture, SECOND_OBJECT_BYTES, SECOND_RELEASE_ID);
        let first_outcome = publish_immutable_release(first.clone()).expect("first release");
        let second_outcome = publish_immutable_release(second).expect("second release");
        let mut rollback = first;
        rollback.evidence_path = fixture.evidence.join("rollback.json");
        let rollback_outcome = publish_immutable_release(rollback).expect("rollback release");

        assert_eq!(std::fs::read(&first_outcome.object_path).unwrap(), FIRST_OBJECT_BYTES);
        assert_eq!(std::fs::read(&second_outcome.object_path).unwrap(), SECOND_OBJECT_BYTES);
        assert_eq!(read_current_pointer(&fixture.current).unwrap(), Some(identity(FIRST_OBJECT_BYTES)));
        assert_eq!(rollback_outcome.evidence.previous_release_identity_blake3, Some(identity(SECOND_OBJECT_BYTES)));
    }

    // r[verify mantle.release.verification]
    #[test]
    fn shell_rejects_reused_evidence_path_before_rollback_pointer_change() {
        let fixture = fixture();
        let first = request(&fixture, FIRST_OBJECT_BYTES, FIRST_RELEASE_ID);
        let second = request(&fixture, SECOND_OBJECT_BYTES, SECOND_RELEASE_ID);
        publish_immutable_release(first.clone()).expect("first release");
        publish_immutable_release(second).expect("second release");

        let error = publish_immutable_release(first).expect_err("reused rollback evidence path must fail");

        assert!(error.to_string().contains("different bytes"));
        assert_eq!(read_current_pointer(&fixture.current).unwrap(), Some(identity(SECOND_OBJECT_BYTES)));
        assert_eq!(std::fs::read(fixture.objects.join(identity(FIRST_OBJECT_BYTES))).unwrap(), FIRST_OBJECT_BYTES);
    }

    // r[verify mantle.release.verification]
    #[test]
    fn shell_rejects_overwrite_of_published_object() {
        let fixture = fixture();
        let request = request(&fixture, FIRST_OBJECT_BYTES, FIRST_RELEASE_ID);
        std::fs::write(&request.object_path, SECOND_OBJECT_BYTES).unwrap();
        let before = std::fs::read(&request.object_path).unwrap();

        let error = publish_immutable_release(request).expect_err("object overwrite must fail");

        assert!(error.to_string().contains("differs from the supplied object"));
        assert_eq!(std::fs::read(fixture.objects.join(identity(FIRST_OBJECT_BYTES))).unwrap(), before);
        assert!(!fixture.current.exists());
    }

    // r[verify mantle.release.verification]
    #[test]
    fn shell_rejects_pointer_to_missing_object() {
        let fixture = fixture();
        let missing_path = fixture.objects.join(identity(FIRST_OBJECT_BYTES));
        let error = switch_current_release_pointer(&fixture.current, &missing_path, identity(FIRST_OBJECT_BYTES))
            .expect_err("missing object must fail");

        assert!(error.to_string().contains("target object is missing"));
        assert!(!fixture.current.exists());
    }

    #[cfg(unix)]
    // r[verify mantle.release.verification]
    #[test]
    fn shell_rejects_symlink_pointer_without_changing_target() {
        let fixture = fixture();
        let request = request(&fixture, FIRST_OBJECT_BYTES, FIRST_RELEASE_ID);
        std::fs::write(&request.object_path, FIRST_OBJECT_BYTES).unwrap();
        let external = fixture._root.path().join("external");
        std::fs::write(&external, b"external-value").unwrap();
        std::os::unix::fs::symlink(&external, &fixture.current).unwrap();

        let error = publish_immutable_release(request).expect_err("symlink pointer must fail");

        assert!(error.to_string().contains("non-symlink regular file"));
        assert_eq!(std::fs::read(&external).unwrap(), b"external-value");
        assert!(fixture.current.is_symlink());
    }

    // r[verify mantle.release.verification]
    #[test]
    fn shell_rejects_evidence_overwrite_after_pointer_readback() {
        let fixture = fixture();
        let request = request(&fixture, FIRST_OBJECT_BYTES, FIRST_RELEASE_ID);
        std::fs::write(&request.evidence_path, b"unrelated-evidence").unwrap();

        let error = publish_immutable_release(request).expect_err("evidence overwrite must fail");

        assert!(error.to_string().contains("different bytes"));
        assert_eq!(
            std::fs::read(fixture.evidence.join(format!("{}.json", identity(FIRST_OBJECT_BYTES)))).unwrap(),
            b"unrelated-evidence"
        );
        assert!(!fixture.current.exists());
    }
}
