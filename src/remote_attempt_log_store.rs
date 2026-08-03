//! Imperative filesystem shell for immutable remote-attempt logs.
//!
//! The pure record, append, replay, chain, and retention decisions live in
//! `crunch_build::distributed::remote_attempt_log`. This module only applies
//! accepted plans with crash-ordered filesystem operations.
//!
//! r[impl remote_builds.immutable_attempt_log_segments]

#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;

use crunch_build::distributed::DEFAULT_REMOTE_ATTEMPT_LOG_POLICY_NAME;
use crunch_build::distributed::RemoteAttemptLogAppendDisposition;
use crunch_build::distributed::RemoteAttemptLogCurrentAttemptFacts;
use crunch_build::distributed::RemoteAttemptLogDigest;
use crunch_build::distributed::RemoteAttemptLogManifest;
use crunch_build::distributed::RemoteAttemptLogPolicy;
use crunch_build::distributed::RemoteAttemptLogReasonCode;
use crunch_build::distributed::RemoteAttemptLogRecord;
use crunch_build::distributed::RemoteAttemptLogRecordInput;
use crunch_build::distributed::RemoteAttemptLogRecordKind;
use crunch_build::distributed::RemoteAttemptLogReplayPlan;
use crunch_build::distributed::RemoteAttemptLogReplayRequest;
use crunch_build::distributed::RemoteAttemptLogRetentionDisposition;
use crunch_build::distributed::RemoteAttemptLogScope;
use crunch_build::distributed::RemoteAttemptLogSegment;
use crunch_build::distributed::RemoteAttemptLogStream;
use crunch_build::distributed::RemoteEventId;
use crunch_build::distributed::empty_remote_attempt_log_manifest;
use crunch_build::distributed::plan_remote_attempt_log_append;
use crunch_build::distributed::plan_remote_attempt_log_replay;
use crunch_build::distributed::plan_remote_attempt_log_retention;
use crunch_build::distributed::seal_remote_attempt_log_record;
use crunch_build::distributed::validate_remote_attempt_log_chain;
use crunch_build::distributed::validate_remote_attempt_log_manifest;
use durable_file_publication::core::CleanupDisposition;
use durable_file_publication::core::DurabilityMode;
use durable_file_publication::core::PrimaryFailure;
use durable_file_publication::core::PublicationDisposition;
use durable_file_publication::core::PublicationRequest;
use durable_file_publication::core::ReplacementMode;
#[cfg(target_os = "linux")]
use durable_file_publication::shell::LinuxDirectory;
use durable_file_publication::shell::StageNameSource;
#[cfg(target_os = "linux")]
use durable_file_publication::shell::publish_one_file;
use rand::RngCore;
use rand::rngs::OsRng;

const REMOTE_ATTEMPT_LOG_STORE_DIR: &str = "remote-attempt-logs";
const REMOTE_ATTEMPT_LOG_SEGMENTS_DIR: &str = "segments";
const REMOTE_ATTEMPT_LOG_ANCHORS_DIR: &str = "anchors";
const REMOTE_ATTEMPT_LOG_MANIFEST_FILE: &str = "manifest.json";
const JSON_FILE_EXTENSION: &str = "json";
const REMOTE_ATTEMPT_LOG_SCOPE_DOMAIN: &str = "mantle-remote-attempt-log-scope-v1";
const MAX_REMOTE_ATTEMPT_LOG_MANIFEST_FILE_BYTES: u64 = 2_097_152;
const MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES: u64 = 8_388_608;
const MAX_REMOTE_ATTEMPT_LOG_ANCHOR_FILE_BYTES: u64 = 131_072;
const DURABLE_DIRECTORY_ANCESTOR_SYNC_COUNT: usize = 2;
const IDENTITY_TEMP_RANDOM_BYTES: usize = 16;
const HEX_CHARS_PER_BYTE: usize = 2;
const IDENTITY_TEMP_CREATE_ATTEMPTS_MAX: u32 = 8;
const BOUNDED_READ_PROBE_BYTES: u64 = 1;
const PRIVATE_TEMP_FILE_MODE: u32 = 0o600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImmutablePublicationBackend {
    Shared,
    Legacy,
}

const PRODUCTION_IMMUTABLE_PUBLICATION_BACKEND: ImmutablePublicationBackend = ImmutablePublicationBackend::Shared;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MantleImmutablePublicationDisposition {
    Created,
    Existing {
        cleanup: CleanupDisposition,
    },
    NotCommitted {
        failure: PrimaryFailure,
        cleanup: CleanupDisposition,
    },
    CommittedDurabilityUnknown,
    UnexpectedVisibilityOnly,
}

struct PrecomputedStageNames {
    names: Vec<String>,
    next_index: usize,
}

impl PrecomputedStageNames {
    fn new(names: Vec<String>) -> Self {
        assert!(!names.is_empty(), "precomputed publication stage names must not be empty");
        assert!(names.iter().all(|name| !name.is_empty()));
        Self { names, next_index: 0 }
    }
}

impl StageNameSource for PrecomputedStageNames {
    fn next_stage_leaf(&mut self) -> String {
        assert!(self.next_index < self.names.len(), "publication stage-name source exhausted unexpectedly");
        let name = self.names[self.next_index].clone();
        self.next_index = self.next_index.saturating_add(1);
        name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteAttemptLogDurabilityStep {
    SegmentDurable,
    AppendManifestDurable,
    AnchorDurable,
    RetentionManifestDurable,
    ExpiredSegmentsDeleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogStoreAppendResult {
    pub manifest: RemoteAttemptLogManifest,
    pub disposition: RemoteAttemptLogAppendDisposition,
    pub durability_steps: Vec<RemoteAttemptLogDurabilityStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedRemoteAttemptLog {
    paths: RemoteAttemptLogPaths,
    manifest: RemoteAttemptLogManifest,
    segments: Vec<RemoteAttemptLogSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteAttemptLogPaths {
    root: PathBuf,
    segments: PathBuf,
    anchors: PathBuf,
    manifest: PathBuf,
}

pub struct RemoteAttemptLogAppendRequest<'a> {
    pub state_dir: &'a Path,
    pub current: &'a RemoteAttemptLogCurrentAttemptFacts,
    pub event_id: RemoteEventId,
    pub cursor: u64,
    pub payload: &'a [u8],
    pub policy: RemoteAttemptLogPolicy,
}

type AppendRemoteAttemptLogFn = fn(
    &Path,
    &RemoteAttemptLogCurrentAttemptFacts,
    RemoteEventId,
    u64,
    &[u8],
    RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogStoreAppendResult, String>;

pub const APPEND_REMOTE_ATTEMPT_LOG: AppendRemoteAttemptLogFn =
    |state_dir, current, event_id, cursor, payload, policy| {
        append_remote_attempt_log_request(RemoteAttemptLogAppendRequest {
            state_dir,
            current,
            event_id,
            cursor,
            payload,
            policy,
        })
    };
pub use APPEND_REMOTE_ATTEMPT_LOG as append_remote_attempt_log;

fn append_remote_attempt_log_request(
    request: RemoteAttemptLogAppendRequest<'_>,
) -> Result<RemoteAttemptLogStoreAppendResult, String> {
    request.policy.validate().map_err(reason)?;
    assert!(!request.current.scope.job_id.as_str().is_empty(), "attempt-log job identity must not be empty");
    assert!(
        !request.current.scope.attempt_id.as_str().is_empty(),
        "attempt-log attempt identity must not be empty"
    );
    let loaded = load_remote_attempt_log(request.state_dir, &request.current.scope, request.policy)?;
    let previous_record_blake3 = previous_record_for_append(&loaded, &request.event_id, request.cursor)?;
    let record = seal_remote_attempt_log_record(
        RemoteAttemptLogRecordInput {
            scope: request.current.scope.clone(),
            event_id: request.event_id,
            sequence: request.cursor,
            cursor: request.cursor,
            phase: request.current.phase,
            stream: RemoteAttemptLogStream::Stdout,
            kind: RemoteAttemptLogRecordKind::Output,
            payload: request.payload.to_vec(),
            previous_record_blake3,
        },
        request.policy,
    )
    .map_err(reason)?;
    apply_append_plan(loaded, request.current, &record, request.policy)
}

pub fn replay_remote_attempt_log(
    state_dir: &Path,
    scope: &RemoteAttemptLogScope,
    request: RemoteAttemptLogReplayRequest,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogReplayPlan, String> {
    policy.validate().map_err(reason)?;
    let loaded = load_remote_attempt_log(state_dir, scope, policy)?;
    plan_remote_attempt_log_replay(&loaded.manifest, &loaded.segments, request, policy).map_err(reason)
}

pub fn load_remote_attempt_log_manifest(
    state_dir: &Path,
    scope: &RemoteAttemptLogScope,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogManifest, String> {
    policy.validate().map_err(reason)?;
    load_remote_attempt_log(state_dir, scope, policy).map(|loaded| loaded.manifest)
}

fn apply_append_plan(
    mut loaded: LoadedRemoteAttemptLog,
    current: &RemoteAttemptLogCurrentAttemptFacts,
    record: &RemoteAttemptLogRecord,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogStoreAppendResult, String> {
    let append = plan_remote_attempt_log_append(&loaded.manifest, current, record, policy).map_err(reason)?;
    if append.disposition == RemoteAttemptLogAppendDisposition::AlreadyApplied {
        return Ok(RemoteAttemptLogStoreAppendResult {
            manifest: append.next_manifest,
            disposition: append.disposition,
            durability_steps: Vec::new(),
        });
    }
    let segment = append.segment.as_ref().ok_or_else(|| "attempt-log-append-segment-missing".to_string())?;
    let mut durability_steps = Vec::new();
    persist_immutable_segment(&loaded.paths, segment)?;
    durability_steps.push(RemoteAttemptLogDurabilityStep::SegmentDurable);
    commit_manifest(&loaded.paths, &append.next_manifest, policy)?;
    durability_steps.push(RemoteAttemptLogDurabilityStep::AppendManifestDurable);
    loaded.segments.push(segment.clone());
    loaded.manifest = append.next_manifest;
    apply_retention(loaded, append.disposition, durability_steps, policy)
}

fn apply_retention(
    loaded: LoadedRemoteAttemptLog,
    disposition: RemoteAttemptLogAppendDisposition,
    mut durability_steps: Vec<RemoteAttemptLogDurabilityStep>,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogStoreAppendResult, String> {
    assert_eq!(
        loaded.manifest.segments.len(),
        loaded.segments.len(),
        "loaded attempt-log segments must match manifest references"
    );
    assert!(durability_steps.len() <= 2, "append durability prefix must stay bounded");
    let retention = plan_remote_attempt_log_retention(&loaded.manifest, &loaded.segments, policy).map_err(reason)?;
    if retention.disposition == RemoteAttemptLogRetentionDisposition::Unchanged {
        return Ok(RemoteAttemptLogStoreAppendResult {
            manifest: retention.next_manifest,
            disposition,
            durability_steps,
        });
    }
    let anchor = retention.anchor.as_ref().ok_or_else(|| "attempt-log-retention-anchor-missing".to_string())?;
    persist_immutable_anchor(&loaded.paths, anchor)?;
    durability_steps.push(RemoteAttemptLogDurabilityStep::AnchorDurable);
    commit_manifest(&loaded.paths, &retention.next_manifest, policy)?;
    durability_steps.push(RemoteAttemptLogDurabilityStep::RetentionManifestDurable);
    delete_expired_segments(&loaded.paths, &retention.delete_segment_blake3)?;
    durability_steps.push(RemoteAttemptLogDurabilityStep::ExpiredSegmentsDeleted);
    prune_superseded_anchors(&loaded.paths, &anchor.anchor_blake3)?;
    Ok(RemoteAttemptLogStoreAppendResult {
        manifest: retention.next_manifest,
        disposition,
        durability_steps,
    })
}

fn load_remote_attempt_log(
    state_dir: &Path,
    scope: &RemoteAttemptLogScope,
    policy: RemoteAttemptLogPolicy,
) -> Result<LoadedRemoteAttemptLog, String> {
    assert!(!scope.job_id.as_str().is_empty(), "attempt-log job identity must not be empty");
    assert!(!scope.attempt_id.as_str().is_empty(), "attempt-log attempt identity must not be empty");
    let paths = remote_attempt_log_paths(state_dir, scope);
    if path_is_absent_no_follow(&paths.manifest, "manifest")? {
        let manifest = empty_remote_attempt_log_manifest(scope.clone(), DEFAULT_REMOTE_ATTEMPT_LOG_POLICY_NAME, policy)
            .map_err(reason)?;
        return Ok(LoadedRemoteAttemptLog {
            paths,
            manifest,
            segments: Vec::new(),
        });
    }
    let manifest: RemoteAttemptLogManifest =
        read_bounded_json(&paths.manifest, MAX_REMOTE_ATTEMPT_LOG_MANIFEST_FILE_BYTES, "manifest")?;
    validate_remote_attempt_log_manifest(&manifest, policy).map_err(reason)?;
    if manifest.scope != *scope {
        return Err(RemoteAttemptLogReasonCode::ScopeMismatch.as_str().to_string());
    }
    let mut segments = Vec::with_capacity(manifest.segments.len());
    for item in &manifest.segments {
        let path = immutable_json_path(&paths.segments, &item.segment_blake3);
        let segment = read_bounded_json(&path, MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES, "segment")?;
        segments.push(segment);
    }
    validate_remote_attempt_log_chain(&manifest, &segments, policy).map_err(reason)?;
    validate_current_anchor_file(&paths, &manifest)?;
    Ok(LoadedRemoteAttemptLog {
        paths,
        manifest,
        segments,
    })
}

fn previous_record_for_append(
    loaded: &LoadedRemoteAttemptLog,
    event_id: &RemoteEventId,
    cursor: u64,
) -> Result<Option<RemoteAttemptLogDigest>, String> {
    if cursor == loaded.manifest.next_cursor {
        return Ok(loaded.manifest.head_record_blake3.clone());
    }
    if loaded.manifest.event_records.contains_key(event_id) {
        let existing = loaded
            .segments
            .iter()
            .flat_map(|segment| &segment.records)
            .find(|record| &record.event_id == event_id)
            .ok_or_else(|| "attempt-log-idempotent-record-not-retained".to_string())?;
        if existing.cursor != cursor {
            return Err(RemoteAttemptLogReasonCode::EventDigestConflict.as_str().to_string());
        }
        return Ok(existing.previous_record_blake3.clone());
    }
    Ok(loaded.manifest.head_record_blake3.clone())
}

fn persist_immutable_segment(paths: &RemoteAttemptLogPaths, segment: &RemoteAttemptLogSegment) -> Result<(), String> {
    let bytes = canonical_json_bytes(segment, "segment")?;
    let path = immutable_json_path(&paths.segments, &segment.segment_blake3);
    publish_immutable_file(&path, &bytes, MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES)
}

fn persist_immutable_anchor(
    paths: &RemoteAttemptLogPaths,
    anchor: &crunch_build::distributed::RemoteAttemptLogTruncationAnchorRecord,
) -> Result<(), String> {
    let bytes = canonical_json_bytes(anchor, "anchor")?;
    let path = immutable_json_path(&paths.anchors, &anchor.anchor_blake3);
    publish_immutable_file(&path, &bytes, MAX_REMOTE_ATTEMPT_LOG_ANCHOR_FILE_BYTES)
}

fn commit_manifest(
    paths: &RemoteAttemptLogPaths,
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), String> {
    commit_manifest_with_hook(paths, manifest, policy, &mut no_publication_hook)
}

fn commit_manifest_with_hook(
    paths: &RemoteAttemptLogPaths,
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
    hook: &mut impl FnMut(PublicationHookPoint, &Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
    validate_remote_attempt_log_manifest(manifest, policy).map_err(reason)?;
    let bytes = canonical_json_bytes(manifest, "manifest")?;
    validate_serialized_size(&bytes, MAX_REMOTE_ATTEMPT_LOG_MANIFEST_FILE_BYTES, "manifest")?;
    ensure_durable_dir(&paths.root)?;
    let temp = write_identity_owned_temp(&paths.manifest, manifest.manifest_blake3.as_str(), &bytes, hook)?;
    if let Err(error) = hook(PublicationHookPoint::BeforePublish, &temp, &paths.manifest) {
        return Err(error_after_temp_cleanup(&temp, error));
    }
    match fs::rename(&temp, &paths.manifest) {
        Ok(()) => sync_directory(&paths.root),
        Err(error) => Err(error_after_temp_cleanup(&temp, format!("attempt-log-manifest-commit-failed: {error}"))),
    }
}

fn publish_immutable_file(path: &Path, bytes: &[u8], bytes_max: u64) -> Result<(), String> {
    publish_immutable_file_with_backend(path, bytes, bytes_max, PRODUCTION_IMMUTABLE_PUBLICATION_BACKEND)
}

fn publish_immutable_file_with_backend(
    path: &Path,
    bytes: &[u8],
    bytes_max: u64,
    backend: ImmutablePublicationBackend,
) -> Result<(), String> {
    match backend {
        ImmutablePublicationBackend::Shared => publish_immutable_file_shared(path, bytes, bytes_max),
        ImmutablePublicationBackend::Legacy => {
            publish_immutable_file_with_hook(path, bytes, bytes_max, &mut no_publication_hook)
        }
    }
}

fn map_shared_publication_disposition(disposition: &PublicationDisposition) -> MantleImmutablePublicationDisposition {
    match disposition {
        PublicationDisposition::NotCommitted { failure, cleanup } => {
            MantleImmutablePublicationDisposition::NotCommitted {
                failure: *failure,
                cleanup: *cleanup,
            }
        }
        PublicationDisposition::DestinationExists { cleanup } => {
            MantleImmutablePublicationDisposition::Existing { cleanup: *cleanup }
        }
        PublicationDisposition::CommittedVisibilityOnly => {
            MantleImmutablePublicationDisposition::UnexpectedVisibilityOnly
        }
        PublicationDisposition::CommittedAndParentSynchronized => MantleImmutablePublicationDisposition::Created,
        PublicationDisposition::CommittedDurabilityUnknown => {
            MantleImmutablePublicationDisposition::CommittedDurabilityUnknown
        }
    }
}

fn shared_publication_request(
    destination_leaf: String,
    payload_bytes: usize,
    bytes_max: u64,
) -> Result<PublicationRequest, String> {
    let maximum_payload_bytes =
        usize::try_from(bytes_max).map_err(|_| "attempt-log-object-size-limit-overflow".to_string())?;
    let collision_attempt_limit = usize::try_from(IDENTITY_TEMP_CREATE_ATTEMPTS_MAX)
        .map_err(|_| "attempt-log-temp-attempt-limit-overflow".to_string())?;
    Ok(PublicationRequest {
        destination_leaf,
        payload_bytes,
        maximum_payload_bytes,
        collision_attempt_limit,
        final_mode: PRIVATE_TEMP_FILE_MODE,
        replacement: ReplacementMode::NoReplace,
        durability: DurabilityMode::DurabilityRequired,
    })
}

fn precompute_stage_names(destination: &Path, identity: &str, count: usize) -> Result<PrecomputedStageNames, String> {
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        let path = identity_owned_temp_path(destination, identity)?;
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "attempt-log-temp-name-invalid".to_string())?;
        names.push(name.to_string());
    }
    if names.len() != count {
        return Err("attempt-log-temp-name-count-invalid".to_string());
    }
    Ok(PrecomputedStageNames::new(names))
}

#[cfg(target_os = "linux")]
fn publish_immutable_file_shared(path: &Path, bytes: &[u8], bytes_max: u64) -> Result<(), String> {
    validate_serialized_size(bytes, bytes_max, "object")?;
    let parent = path.parent().ok_or_else(|| "attempt-log-object-parent-missing".to_string())?;
    let destination_leaf = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "attempt-log-object-destination-name-invalid".to_string())?;
    let identity = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "attempt-log-object-identity-invalid".to_string())?;
    let request = shared_publication_request(destination_leaf.to_string(), bytes.len(), bytes_max)?;
    let mut names = precompute_stage_names(path, identity, request.collision_attempt_limit)?;
    ensure_durable_dir(parent)?;
    let directory = open_publication_parent(parent)?;
    let mut io = LinuxDirectory::from_open_directory(directory)
        .map_err(|error| format!("attempt-log-object-parent-capability-invalid: {error}"))?;
    let classification = publish_one_file(&mut io, &mut names, &request, bytes)
        .map_err(|error| format!("attempt-log-object-classification-invalid: {error:?}"))?;
    interpret_shared_publication(path, bytes, bytes_max, &classification.disposition)
}

#[cfg(not(target_os = "linux"))]
fn publish_immutable_file_shared(_path: &Path, _bytes: &[u8], _bytes_max: u64) -> Result<(), String> {
    Err("attempt-log-immutable-object-shared-publication-unsupported".to_string())
}

#[cfg(target_os = "linux")]
fn open_publication_parent(parent: &Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt as _;

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent)
        .map_err(|error| format!("attempt-log-object-parent-open-no-follow-failed: {error}"))
}

fn interpret_shared_publication(
    path: &Path,
    bytes: &[u8],
    bytes_max: u64,
    disposition: &PublicationDisposition,
) -> Result<(), String> {
    match map_shared_publication_disposition(disposition) {
        MantleImmutablePublicationDisposition::Created => Ok(()),
        MantleImmutablePublicationDisposition::Existing {
            cleanup: CleanupDisposition::Removed,
        } => compare_existing_immutable_file(path, bytes, bytes_max),
        MantleImmutablePublicationDisposition::Existing { cleanup } => {
            Err(format!("attempt-log-immutable-object-existing-cleanup-{}", cleanup_code(cleanup)))
        }
        MantleImmutablePublicationDisposition::NotCommitted { failure, cleanup } => Err(format!(
            "attempt-log-immutable-object-not-committed-{}-cleanup-{}",
            primary_failure_code(failure),
            cleanup_code(cleanup)
        )),
        MantleImmutablePublicationDisposition::CommittedDurabilityUnknown => {
            Err("attempt-log-immutable-object-committed-durability-unknown".to_string())
        }
        MantleImmutablePublicationDisposition::UnexpectedVisibilityOnly => {
            Err("attempt-log-immutable-object-unexpected-visibility-only".to_string())
        }
    }
}

fn compare_existing_immutable_file(path: &Path, bytes: &[u8], bytes_max: u64) -> Result<(), String> {
    let existing = read_bounded_bytes(path, bytes_max, "immutable-object")?;
    if existing == bytes {
        return Ok(());
    }
    Err("attempt-log-immutable-object-conflict".to_string())
}

fn primary_failure_code(failure: PrimaryFailure) -> &'static str {
    match failure {
        PrimaryFailure::NoReplaceUnsupported => "no-replace-unsupported",
        PrimaryFailure::ParentSyncUnsupported => "parent-sync-unsupported",
        PrimaryFailure::StageCreate => "stage-create",
        PrimaryFailure::CollisionExhausted => "collision-exhausted",
        PrimaryFailure::PayloadWrite => "payload-write",
        PrimaryFailure::PermissionApply => "permission-apply",
        PrimaryFailure::PayloadSync => "payload-sync",
        PrimaryFailure::Rename => "rename",
    }
}

fn cleanup_code(cleanup: CleanupDisposition) -> &'static str {
    match cleanup {
        CleanupDisposition::NotNeeded => "not-needed",
        CleanupDisposition::Removed => "removed",
        CleanupDisposition::Failed => "failed",
    }
}

fn publish_immutable_file_with_hook(
    path: &Path,
    bytes: &[u8],
    bytes_max: u64,
    hook: &mut impl FnMut(PublicationHookPoint, &Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
    validate_serialized_size(bytes, bytes_max, "object")?;
    assert!(!path.as_os_str().is_empty(), "immutable attempt-log path must not be empty");
    assert!(u64::try_from(bytes.len()).is_ok_and(|byte_count| byte_count <= bytes_max));
    let parent = path.parent().ok_or_else(|| "attempt-log-object-parent-missing".to_string())?;
    ensure_durable_dir(parent)?;
    let identity = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "attempt-log-object-identity-invalid".to_string())?;
    let temp = write_identity_owned_temp(path, identity, bytes, hook)?;
    if let Err(error) = hook(PublicationHookPoint::BeforePublish, &temp, path) {
        return Err(error_after_temp_cleanup(&temp, error));
    }
    match rename_no_replace(&temp, path) {
        Ok(()) => sync_directory(parent),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            remove_private_temp(&temp)?;
            let existing = read_bounded_bytes(path, bytes_max, "immutable-object")?;
            if existing == bytes {
                return Ok(());
            }
            Err("attempt-log-immutable-object-conflict".to_string())
        }
        Err(error) => {
            Err(error_after_temp_cleanup(&temp, format!("attempt-log-immutable-object-commit-failed: {error}")))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicationHookPoint {
    BeforeTempCreate,
    BeforePublish,
}

fn no_publication_hook(_point: PublicationHookPoint, _temp: &Path, _destination: &Path) -> Result<(), String> {
    Ok(())
}

fn write_identity_owned_temp(
    destination: &Path,
    identity: &str,
    bytes: &[u8],
    hook: &mut impl FnMut(PublicationHookPoint, &Path, &Path) -> Result<(), String>,
) -> Result<PathBuf, String> {
    assert!(!identity.is_empty(), "attempt-log temp identity must not be empty");
    assert!(!destination.as_os_str().is_empty(), "attempt-log temp destination must not be empty");
    let parent = destination.parent().ok_or_else(|| "attempt-log-temp-parent-missing".to_string())?;
    for _ in 0..IDENTITY_TEMP_CREATE_ATTEMPTS_MAX {
        let temp = identity_owned_temp_path(destination, identity)?;
        hook(PublicationHookPoint::BeforeTempCreate, &temp, destination)?;
        match create_new_private_file(&temp) {
            Ok(mut file) => {
                let write_result = file
                    .write_all(bytes)
                    .map_err(|error| format!("attempt-log-temp-write-failed: {error}"))
                    .and_then(|()| file.sync_all().map_err(|error| format!("attempt-log-temp-sync-failed: {error}")));
                drop(file);
                if let Err(error) = write_result {
                    return Err(error_after_temp_cleanup(&temp, error));
                }
                if let Err(error) = sync_directory(parent) {
                    return Err(error_after_temp_cleanup(&temp, error));
                }
                return Ok(temp);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("attempt-log-temp-create-failed: {error}")),
        }
    }
    Err("attempt-log-temp-create-attempts-exhausted".to_string())
}

fn identity_owned_temp_path(destination: &Path, identity: &str) -> Result<PathBuf, String> {
    assert!(!destination.as_os_str().is_empty(), "attempt-log temp destination must not be empty");
    assert!(!identity.is_empty(), "attempt-log temp identity must not be empty");
    let file_name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "attempt-log-temp-destination-name-invalid".to_string())?;
    let mut random = [0_u8; IDENTITY_TEMP_RANDOM_BYTES];
    OsRng
        .try_fill_bytes(&mut random)
        .map_err(|error| format!("attempt-log-temp-random-failed: {error}"))?;
    let random_hex_size_bytes = IDENTITY_TEMP_RANDOM_BYTES
        .checked_mul(HEX_CHARS_PER_BYTE)
        .ok_or_else(|| "attempt-log-temp-random-capacity-overflow".to_string())?;
    let mut random_hex = String::with_capacity(random_hex_size_bytes);
    for byte in random {
        use std::fmt::Write as _;
        write!(&mut random_hex, "{byte:02x}").map_err(|_| "attempt-log-temp-random-format-failed".to_string())?;
    }
    if random_hex.len() != random_hex_size_bytes {
        return Err("attempt-log-temp-random-length-invalid".to_string());
    }
    Ok(destination.with_file_name(format!(".{file_name}.{identity}.{random_hex}.tmp")))
}

#[cfg(unix)]
fn create_new_private_file(path: &Path) -> std::io::Result<File> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_TEMP_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
}

#[cfg(not(unix))]
fn create_new_private_file(_path: &Path) -> std::io::Result<File> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "secure create-new attempt-log temps are unsupported on this platform",
    ))
}

#[cfg(target_os = "linux")]
fn rename_no_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
    let source = CString::new(source.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "attempt-log source contains NUL"))?;
    let destination = CString::new(destination.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "attempt-log destination contains NUL"))?;
    crate::linux_rename::rename_no_replace(libc::AT_FDCWD, &source, libc::AT_FDCWD, &destination)
}

#[cfg(not(target_os = "linux"))]
fn rename_no_replace(_source: &Path, _destination: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic no-replace attempt-log publication is unsupported on this platform",
    ))
}

fn remove_private_temp(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("attempt-log-private-temp-cleanup-failed: {error}")),
    }
}

fn error_after_temp_cleanup(path: &Path, primary: String) -> String {
    match remove_private_temp(path) {
        Ok(()) => primary,
        Err(cleanup) => format!("{primary}; {cleanup}"),
    }
}

fn validate_serialized_size(bytes: &[u8], bytes_max: u64, kind: &str) -> Result<(), String> {
    let byte_count = u64::try_from(bytes.len()).map_err(|_| format!("attempt-log-{kind}-size-overflow"))?;
    if byte_count > bytes_max {
        return Err(format!("attempt-log-{kind}-file-too-large"));
    }
    Ok(())
}

fn delete_expired_segments(paths: &RemoteAttemptLogPaths, digests: &[RemoteAttemptLogDigest]) -> Result<(), String> {
    for digest in digests {
        let path = immutable_json_path(&paths.segments, digest);
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("attempt-log-expired-segment-delete-failed: {error}")),
        }
    }
    sync_directory(&paths.segments)
}

fn prune_superseded_anchors(paths: &RemoteAttemptLogPaths, retained: &RemoteAttemptLogDigest) -> Result<(), String> {
    let retained_path = immutable_json_path(&paths.anchors, retained);
    let entries =
        fs::read_dir(&paths.anchors).map_err(|error| format!("attempt-log-anchor-directory-read-failed: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("attempt-log-anchor-entry-read-failed: {error}"))?;
        let path = entry.path();
        if path == retained_path || path.extension().and_then(|value| value.to_str()) != Some(JSON_FILE_EXTENSION) {
            continue;
        }
        fs::remove_file(&path).map_err(|error| format!("attempt-log-superseded-anchor-delete-failed: {error}"))?;
    }
    sync_directory(&paths.anchors)
}

fn validate_current_anchor_file(
    paths: &RemoteAttemptLogPaths,
    manifest: &RemoteAttemptLogManifest,
) -> Result<(), String> {
    let Some(expected) = manifest.truncation_anchor.as_ref() else {
        return Ok(());
    };
    let path = immutable_json_path(&paths.anchors, &expected.anchor_blake3);
    let observed: crunch_build::distributed::RemoteAttemptLogTruncationAnchorRecord =
        read_bounded_json(&path, MAX_REMOTE_ATTEMPT_LOG_ANCHOR_FILE_BYTES, "anchor")?;
    if &observed != expected {
        return Err("attempt-log-anchor-content-mismatch".to_string());
    }
    Ok(())
}

fn read_bounded_json<T: serde::de::DeserializeOwned>(path: &Path, bytes_max: u64, kind: &str) -> Result<T, String> {
    let bytes = read_bounded_bytes(path, bytes_max, kind)?;
    serde_json::from_slice(&bytes).map_err(|error| format!("attempt-log-{kind}-parse-failed: {error}"))
}

fn read_bounded_bytes(path: &Path, bytes_max: u64, kind: &str) -> Result<Vec<u8>, String> {
    assert!(!path.as_os_str().is_empty(), "bounded attempt-log path must not be empty");
    assert!(bytes_max > 0, "bounded attempt-log read limit must be positive");
    require_regular_path_no_follow(path, kind)?;
    let mut file = open_regular_no_follow(path, kind)?;
    let metadata = file.metadata().map_err(|error| format!("attempt-log-{kind}-opened-metadata-failed: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("attempt-log-{kind}-not-regular-file"));
    }
    if metadata.len() > bytes_max {
        return Err(format!("attempt-log-{kind}-file-too-large"));
    }
    let read_limit_bytes = bytes_max
        .checked_add(BOUNDED_READ_PROBE_BYTES)
        .ok_or_else(|| format!("attempt-log-{kind}-read-limit-overflow"))?;
    let file_size_bytes =
        usize::try_from(metadata.len()).map_err(|_| format!("attempt-log-{kind}-capacity-overflow"))?;
    let mut bytes = Vec::with_capacity(file_size_bytes);
    Read::by_ref(&mut file)
        .take(read_limit_bytes)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("attempt-log-{kind}-read-failed: {error}"))?;
    validate_serialized_size(&bytes, bytes_max, kind)?;
    Ok(bytes)
}

fn path_is_absent_no_follow(path: &Path, kind: &str) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!("attempt-log-{kind}-metadata-failed: {error}")),
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!("attempt-log-{kind}-symlink-rejected")),
        Ok(metadata) if !metadata.is_file() => Err(format!("attempt-log-{kind}-not-regular-file")),
        Ok(_) => Ok(false),
    }
}

fn require_regular_path_no_follow(path: &Path, kind: &str) -> Result<(), String> {
    if path_is_absent_no_follow(path, kind)? {
        return Err(format!("attempt-log-{kind}-missing"));
    }
    Ok(())
}

#[cfg(unix)]
fn open_regular_no_follow(path: &Path, kind: &str) -> Result<File, String> {
    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| format!("attempt-log-{kind}-open-no-follow-failed: {error}"))
}

#[cfg(not(unix))]
fn open_regular_no_follow(_path: &Path, kind: &str) -> Result<File, String> {
    Err(format!("attempt-log-{kind}-secure-read-unsupported"))
}

fn canonical_json_bytes(value: &impl serde::Serialize, kind: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| format!("attempt-log-{kind}-serialize-failed: {error}"))
}

pub(crate) fn remote_attempt_log_manifest_path(state_dir: &Path, scope: &RemoteAttemptLogScope) -> PathBuf {
    remote_attempt_log_paths(state_dir, scope).manifest
}

pub(crate) fn remote_attempt_log_segment_path(
    state_dir: &Path,
    scope: &RemoteAttemptLogScope,
    digest: &RemoteAttemptLogDigest,
) -> PathBuf {
    immutable_json_path(&remote_attempt_log_paths(state_dir, scope).segments, digest)
}

pub(crate) fn remote_attempt_log_anchor_path(
    state_dir: &Path,
    scope: &RemoteAttemptLogScope,
    digest: &RemoteAttemptLogDigest,
) -> PathBuf {
    immutable_json_path(&remote_attempt_log_paths(state_dir, scope).anchors, digest)
}

fn remote_attempt_log_paths(state_dir: &Path, scope: &RemoteAttemptLogScope) -> RemoteAttemptLogPaths {
    let scope_key = remote_attempt_log_scope_key(scope);
    let root = state_dir.join(REMOTE_ATTEMPT_LOG_STORE_DIR).join(scope_key);
    RemoteAttemptLogPaths {
        segments: root.join(REMOTE_ATTEMPT_LOG_SEGMENTS_DIR),
        anchors: root.join(REMOTE_ATTEMPT_LOG_ANCHORS_DIR),
        manifest: root.join(REMOTE_ATTEMPT_LOG_MANIFEST_FILE),
        root,
    }
}

fn remote_attempt_log_scope_key(scope: &RemoteAttemptLogScope) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_scope_part(&mut hasher, REMOTE_ATTEMPT_LOG_SCOPE_DOMAIN);
    hash_scope_part(&mut hasher, scope.job_id.as_str());
    hash_scope_part(&mut hasher, scope.attempt_id.as_str());
    hasher.update(&scope.fence_generation.get().to_le_bytes());
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), blake3::OUT_LEN.saturating_mul(2));
    debug_assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    digest
}

fn hash_scope_part(hasher: &mut blake3::Hasher, value: &str) {
    let length_bytes = match u64::try_from(value.len()) {
        Ok(length_bytes) => length_bytes,
        Err(_) => {
            hasher.update(b"attempt-log-scope-length-overflow");
            return;
        }
    };
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(value.as_bytes());
}

fn immutable_json_path(directory: &Path, digest: &RemoteAttemptLogDigest) -> PathBuf {
    directory.join(digest.as_str()).with_extension(JSON_FILE_EXTENSION)
}

fn ensure_durable_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| format!("attempt-log-directory-create-failed: {error}"))?;
    sync_directory(path)?;
    let mut ancestor = path.parent();
    for _ in 0..DURABLE_DIRECTORY_ANCESTOR_SYNC_COUNT {
        let Some(directory) = ancestor else {
            break;
        };
        sync_directory(directory)?;
        ancestor = directory.parent();
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    let directory = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| format!("attempt-log-directory-open-no-follow-failed: {error}"))?;
    let metadata = directory.metadata().map_err(|error| format!("attempt-log-directory-metadata-failed: {error}"))?;
    if !metadata.is_dir() {
        return Err("attempt-log-directory-not-directory".to_string());
    }
    directory.sync_all().map_err(|error| format!("attempt-log-directory-sync-failed: {error}"))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn reason(reason: RemoteAttemptLogReasonCode) -> String {
    reason.as_str().to_string()
}

#[cfg(test)]
mod tests {
    use crunch_build::distributed::RemoteAttemptId;
    use crunch_build::distributed::RemoteAttemptPhase;
    use crunch_build::distributed::RemoteFenceGeneration;
    use crunch_build::distributed::RemoteJobId;
    use durable_file_publication::core::Classification;
    use durable_file_publication::core::ClassificationError;
    use durable_file_publication::corpus::CONFORMANCE_CORPUS;
    use durable_file_publication::corpus::CorpusCase;
    use durable_file_publication::corpus::ExpectedDisposition;
    use durable_file_publication::corpus::FaultPoint;
    use durable_file_publication::shell::MechanicalIoFailure;
    use durable_file_publication::shell::PublicationIo;
    use durable_file_publication::shell::RenameDisposition;
    use durable_file_publication::shell::StageCreateDisposition;

    use super::*;

    const TEST_FENCE: u64 = 3;
    const TEST_RETAINED_SEGMENTS: u32 = 2;
    const TEST_RECORD_COUNT: u64 = 4;
    const TEST_REPLAY_BYTES: u64 = 1_024;
    const CORPUS_COLLISION_ATTEMPT_LIMIT: usize = 3;
    const EXPECTED_SHARED_CORPUS_CASES: usize = 17;
    const RACE_WRITER_COUNT: usize = 2;
    const CORPUS_PAYLOAD: &[u8] = b"mantle-corpus-payload";

    struct CorpusIo {
        case: CorpusCase,
        collision_count: usize,
        stage_created: bool,
    }

    impl CorpusIo {
        fn new(case: CorpusCase) -> Self {
            Self {
                case,
                collision_count: 0,
                stage_created: false,
            }
        }
    }

    impl PublicationIo for CorpusIo {
        fn supports_no_replace(&self) -> bool {
            self.case.no_replace_supported
        }

        fn supports_parent_sync(&self) -> bool {
            self.case.parent_sync_supported
        }

        fn create_stage(
            &mut self,
            _leaf: &str,
            _private_mode: u32,
        ) -> Result<StageCreateDisposition, MechanicalIoFailure> {
            if self.case.fault == FaultPoint::StageCreate {
                return Err(MechanicalIoFailure);
            }
            if self.collision_count < self.case.collisions_before_success {
                self.collision_count = self.collision_count.saturating_add(1);
                return Ok(StageCreateDisposition::AlreadyExists);
            }
            self.stage_created = true;
            Ok(StageCreateDisposition::Created)
        }

        fn write_all(&mut self, _bytes: &[u8]) -> Result<(), MechanicalIoFailure> {
            if self.case.fault == FaultPoint::PayloadWrite {
                return Err(MechanicalIoFailure);
            }
            Ok(())
        }

        fn apply_permissions(&mut self, _mode: u32) -> Result<(), MechanicalIoFailure> {
            if self.case.fault == FaultPoint::PermissionApply {
                return Err(MechanicalIoFailure);
            }
            Ok(())
        }

        fn sync_payload(&mut self) -> Result<(), MechanicalIoFailure> {
            if self.case.fault == FaultPoint::PayloadSync {
                return Err(MechanicalIoFailure);
            }
            Ok(())
        }

        fn rename_stage(
            &mut self,
            _stage_leaf: &str,
            _destination_leaf: &str,
            replacement: ReplacementMode,
        ) -> Result<RenameDisposition, MechanicalIoFailure> {
            if self.case.fault == FaultPoint::Rename {
                return Err(MechanicalIoFailure);
            }
            if self.case.destination_exists && replacement == ReplacementMode::NoReplace {
                return Ok(RenameDisposition::DestinationExists);
            }
            Ok(RenameDisposition::Committed)
        }

        fn sync_parent(&mut self) -> Result<(), MechanicalIoFailure> {
            if self.case.fault == FaultPoint::ParentSync {
                return Err(MechanicalIoFailure);
            }
            Ok(())
        }

        fn cleanup_stage(&mut self, _stage_leaf: &str) -> Result<(), MechanicalIoFailure> {
            self.stage_created = false;
            if self.case.cleanup_fails {
                return Err(MechanicalIoFailure);
            }
            Ok(())
        }
    }

    struct CorpusStageNames {
        unsafe_name: bool,
        next_index: usize,
    }

    impl StageNameSource for CorpusStageNames {
        fn next_stage_leaf(&mut self) -> String {
            if self.unsafe_name {
                return "../unsafe-stage".to_string();
            }
            let leaf = format!(".mantle-corpus-stage-{}", self.next_index);
            self.next_index = self.next_index.saturating_add(1);
            leaf
        }
    }

    fn corpus_request(case: CorpusCase) -> PublicationRequest {
        let maximum_payload_bytes = if case.payload_over_bound {
            CORPUS_PAYLOAD.len().saturating_sub(1)
        } else {
            CORPUS_PAYLOAD.len()
        };
        PublicationRequest {
            destination_leaf: "object.json".to_string(),
            payload_bytes: CORPUS_PAYLOAD.len(),
            maximum_payload_bytes,
            collision_attempt_limit: CORPUS_COLLISION_ATTEMPT_LIMIT,
            final_mode: PRIVATE_TEMP_FILE_MODE,
            replacement: case.replacement,
            durability: case.durability,
        }
    }

    fn expected_cleanup(case: CorpusCase, failure: PrimaryFailure) -> CleanupDisposition {
        if matches!(
            failure,
            PrimaryFailure::NoReplaceUnsupported
                | PrimaryFailure::ParentSyncUnsupported
                | PrimaryFailure::StageCreate
                | PrimaryFailure::CollisionExhausted
        ) {
            return CleanupDisposition::NotNeeded;
        }
        if case.cleanup_fails {
            CleanupDisposition::Failed
        } else {
            CleanupDisposition::Removed
        }
    }

    fn assert_corpus_result(case: CorpusCase, result: Result<Classification, ClassificationError>) {
        match case.expected {
            ExpectedDisposition::InvalidRequest => {
                assert!(matches!(result, Err(ClassificationError::InvalidRequest(_))), "{}", case.id);
            }
            ExpectedDisposition::InvalidStageName => {
                assert_eq!(result, Err(ClassificationError::InvalidStageLeaf), "{}", case.id);
            }
            ExpectedDisposition::Visible => {
                let classification = result.expect(case.id);
                assert_eq!(classification.disposition, PublicationDisposition::CommittedVisibilityOnly);
                assert_eq!(
                    map_shared_publication_disposition(&classification.disposition),
                    MantleImmutablePublicationDisposition::UnexpectedVisibilityOnly
                );
            }
            ExpectedDisposition::Durable => {
                let classification = result.expect(case.id);
                assert_eq!(classification.disposition, PublicationDisposition::CommittedAndParentSynchronized);
                assert_eq!(
                    map_shared_publication_disposition(&classification.disposition),
                    MantleImmutablePublicationDisposition::Created
                );
            }
            ExpectedDisposition::DestinationExists => {
                let classification = result.expect(case.id);
                let expected = PublicationDisposition::DestinationExists {
                    cleanup: if case.cleanup_fails {
                        CleanupDisposition::Failed
                    } else {
                        CleanupDisposition::Removed
                    },
                };
                assert_eq!(classification.disposition, expected);
                assert_eq!(
                    map_shared_publication_disposition(&classification.disposition),
                    MantleImmutablePublicationDisposition::Existing {
                        cleanup: if case.cleanup_fails {
                            CleanupDisposition::Failed
                        } else {
                            CleanupDisposition::Removed
                        }
                    }
                );
            }
            ExpectedDisposition::NotCommitted(failure) => {
                let classification = result.expect(case.id);
                let cleanup = expected_cleanup(case, failure);
                let expected = PublicationDisposition::NotCommitted { failure, cleanup };
                assert_eq!(classification.disposition, expected);
                assert_eq!(
                    map_shared_publication_disposition(&classification.disposition),
                    MantleImmutablePublicationDisposition::NotCommitted { failure, cleanup }
                );
            }
            ExpectedDisposition::CommittedDurabilityUnknown => {
                let classification = result.expect(case.id);
                assert_eq!(classification.disposition, PublicationDisposition::CommittedDurabilityUnknown);
                assert_eq!(
                    map_shared_publication_disposition(&classification.disposition),
                    MantleImmutablePublicationDisposition::CommittedDurabilityUnknown
                );
            }
        }
    }

    fn scope() -> RemoteAttemptLogScope {
        RemoteAttemptLogScope {
            job_id: RemoteJobId::new("shell-job").unwrap(),
            attempt_id: RemoteAttemptId::new("shell-attempt").unwrap(),
            fence_generation: RemoteFenceGeneration::new(TEST_FENCE).unwrap(),
        }
    }

    fn current() -> RemoteAttemptLogCurrentAttemptFacts {
        RemoteAttemptLogCurrentAttemptFacts {
            scope: scope(),
            phase: RemoteAttemptPhase::Running,
        }
    }

    fn policy() -> RemoteAttemptLogPolicy {
        RemoteAttemptLogPolicy {
            segment_record_count_max: 1,
            retained_segment_count_max: TEST_RETAINED_SEGMENTS,
            retained_record_count_max: TEST_RETAINED_SEGMENTS,
            replay_record_count_max: TEST_RETAINED_SEGMENTS,
            ..RemoteAttemptLogPolicy::default()
        }
    }

    // r[verify mantle.durable_file_publication.validation]
    #[test]
    fn shared_corpus_preserves_all_mechanical_and_mapping_dispositions() {
        assert_eq!(CONFORMANCE_CORPUS.len(), EXPECTED_SHARED_CORPUS_CASES);
        assert!(CONFORMANCE_CORPUS.iter().any(|case| case.cleanup_fails));
        for case in CONFORMANCE_CORPUS {
            let mut io = CorpusIo::new(*case);
            let mut names = CorpusStageNames {
                unsafe_name: case.unsafe_stage_name,
                next_index: 0,
            };
            let request = corpus_request(*case);
            let result =
                durable_file_publication::shell::publish_one_file(&mut io, &mut names, &request, CORPUS_PAYLOAD);
            assert_corpus_result(*case, result);
        }
    }

    #[cfg(target_os = "linux")]
    // r[verify mantle.durable_file_publication.adapter]
    // r[verify mantle.durable_file_publication.mapping]
    #[test]
    fn shared_backend_accepts_only_identical_existing_content() {
        let state = tempfile::tempdir().unwrap();
        let destination = state.path().join("immutable").join("object.json");
        let original = b"original-object";
        let conflict = b"conflicting-object";

        publish_immutable_file_with_backend(
            &destination,
            original,
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            ImmutablePublicationBackend::Shared,
        )
        .unwrap();
        publish_immutable_file_with_backend(
            &destination,
            original,
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            ImmutablePublicationBackend::Shared,
        )
        .unwrap();
        let error = publish_immutable_file_with_backend(
            &destination,
            conflict,
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            ImmutablePublicationBackend::Shared,
        )
        .unwrap_err();

        assert_eq!(error, "attempt-log-immutable-object-conflict");
        assert_eq!(fs::read(&destination).unwrap(), original);
        assert!(fs::symlink_metadata(&destination).unwrap().is_file());
    }

    #[cfg(target_os = "linux")]
    // r[verify mantle.durable_file_publication.validation]
    #[test]
    fn shared_backend_destination_race_keeps_one_immutable_winner() {
        use std::sync::Arc;
        use std::sync::Barrier;

        let state = tempfile::tempdir().unwrap();
        let destination = state.path().join("immutable-race").join("object.json");
        let barrier = Arc::new(Barrier::new(RACE_WRITER_COUNT));
        let payloads = [b"first-race-payload".to_vec(), b"second-race-payload".to_vec()];
        let mut workers = Vec::with_capacity(RACE_WRITER_COUNT);
        for payload in &payloads {
            let worker_destination = destination.clone();
            let worker_barrier = Arc::clone(&barrier);
            let worker_payload = payload.clone();
            workers.push(std::thread::spawn(move || {
                worker_barrier.wait();
                let result = publish_immutable_file_with_backend(
                    &worker_destination,
                    &worker_payload,
                    MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
                    ImmutablePublicationBackend::Shared,
                );
                (worker_payload, result)
            }));
        }
        let outcomes = workers.into_iter().map(|worker| worker.join().unwrap()).collect::<Vec<_>>();
        let success_count = outcomes.iter().filter(|(_, result)| result.is_ok()).count();
        let conflict_count = outcomes
            .iter()
            .filter(|(_, result)| result.as_ref().is_err_and(|error| error == "attempt-log-immutable-object-conflict"))
            .count();
        let final_bytes = fs::read(&destination).unwrap();

        assert_eq!(success_count, 1);
        assert_eq!(conflict_count, 1);
        assert!(payloads.iter().any(|payload| payload == &final_bytes));
    }

    #[cfg(target_os = "linux")]
    // r[verify mantle.durable_file_publication.validation]
    #[test]
    fn shared_backend_rejects_symlink_destination_and_parent() {
        use std::os::unix::fs::symlink;

        let state = tempfile::tempdir().unwrap();
        let immutable_parent = state.path().join("immutable");
        fs::create_dir(&immutable_parent).unwrap();
        let destination = immutable_parent.join("object.json");
        let victim = state.path().join("victim.txt");
        fs::write(&victim, b"unchanged").unwrap();
        symlink(&victim, &destination).unwrap();
        let destination_error = publish_immutable_file_with_backend(
            &destination,
            b"must-not-reach-victim",
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            ImmutablePublicationBackend::Shared,
        )
        .unwrap_err();

        let actual_parent = state.path().join("actual-parent");
        fs::create_dir(&actual_parent).unwrap();
        let linked_parent = state.path().join("linked-parent");
        symlink(&actual_parent, &linked_parent).unwrap();
        let parent_error = publish_immutable_file_with_backend(
            &linked_parent.join("object.json"),
            b"must-not-reach-parent",
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            ImmutablePublicationBackend::Shared,
        )
        .unwrap_err();

        assert!(destination_error.contains("immutable-object-symlink-rejected"));
        assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
        assert!(parent_error.contains("directory-open-no-follow-failed"));
        assert!(fs::symlink_metadata(actual_parent.join("object.json")).is_err());
    }

    #[cfg(target_os = "linux")]
    // r[verify mantle.durable_file_publication.authority]
    #[test]
    fn legacy_backend_remains_explicit_without_automatic_fallback() {
        let state = tempfile::tempdir().unwrap();
        let destination = state.path().join("legacy").join("object.json");

        publish_immutable_file_with_backend(
            &destination,
            b"legacy-rollback-object",
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            ImmutablePublicationBackend::Legacy,
        )
        .unwrap();

        assert_eq!(PRODUCTION_IMMUTABLE_PUBLICATION_BACKEND, ImmutablePublicationBackend::Shared);
        assert_eq!(fs::read(&destination).unwrap(), b"legacy-rollback-object");
    }

    #[test]
    fn committed_durability_unknown_is_neither_failure_nor_success() {
        let mapped = map_shared_publication_disposition(&PublicationDisposition::CommittedDurabilityUnknown);
        let error = interpret_shared_publication(
            Path::new("unused-object.json"),
            b"unused",
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            &PublicationDisposition::CommittedDurabilityUnknown,
        )
        .unwrap_err();

        assert_eq!(mapped, MantleImmutablePublicationDisposition::CommittedDurabilityUnknown);
        assert_eq!(error, "attempt-log-immutable-object-committed-durability-unknown");
    }

    #[test]
    fn append_restart_replay_and_retention_use_durable_immutable_files() {
        let state = tempfile::tempdir().unwrap();
        let mut last = None;
        for cursor in 0..TEST_RECORD_COUNT {
            let result = append_remote_attempt_log(
                state.path(),
                &current(),
                RemoteEventId::new(format!("event-{cursor}")).unwrap(),
                cursor,
                format!("payload-{cursor}").as_bytes(),
                policy(),
            )
            .unwrap();
            last = Some(result);
        }
        let result = last.unwrap();
        let manifest = load_remote_attempt_log_manifest(state.path(), &scope(), policy()).unwrap();
        let replay = replay_remote_attempt_log(
            state.path(),
            &scope(),
            RemoteAttemptLogReplayRequest {
                from_cursor: manifest.retained_start_cursor,
                record_count_max: TEST_RETAINED_SEGMENTS,
                payload_bytes_max: TEST_REPLAY_BYTES,
            },
            policy(),
        )
        .unwrap();
        let paths = remote_attempt_log_paths(state.path(), &scope());
        let segment_files = fs::read_dir(&paths.segments).unwrap().count();
        let anchor_files = fs::read_dir(&paths.anchors).unwrap().count();

        assert_eq!(result.manifest, manifest);
        assert_eq!(manifest.retained_start_cursor, TEST_RECORD_COUNT - u64::from(TEST_RETAINED_SEGMENTS));
        assert_eq!(replay.records.len(), usize::try_from(TEST_RETAINED_SEGMENTS).unwrap());
        assert_eq!(segment_files, usize::try_from(TEST_RETAINED_SEGMENTS).unwrap());
        assert_eq!(anchor_files, 1);
        assert!(result.durability_steps.windows(2).all(|pair| pair[0] != pair[1]));
        assert_eq!(result.durability_steps, vec![
            RemoteAttemptLogDurabilityStep::SegmentDurable,
            RemoteAttemptLogDurabilityStep::AppendManifestDurable,
            RemoteAttemptLogDurabilityStep::AnchorDurable,
            RemoteAttemptLogDurabilityStep::RetentionManifestDurable,
            RemoteAttemptLogDurabilityStep::ExpiredSegmentsDeleted,
        ]);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_segment_is_rejected_without_reading_target_bytes() {
        use std::os::unix::fs::symlink;

        let state = tempfile::tempdir().unwrap();
        append_remote_attempt_log(
            state.path(),
            &current(),
            RemoteEventId::new("symlink-segment").unwrap(),
            0,
            b"payload",
            policy(),
        )
        .unwrap();
        let manifest = load_remote_attempt_log_manifest(state.path(), &scope(), policy()).unwrap();
        let paths = remote_attempt_log_paths(state.path(), &scope());
        let segment = manifest.segments.first().unwrap();
        let segment_path = immutable_json_path(&paths.segments, &segment.segment_blake3);
        let target = state.path().join("attacker-controlled-segment.json");
        let original = fs::read(&segment_path).unwrap();
        fs::write(&target, original).unwrap();
        fs::remove_file(&segment_path).unwrap();
        symlink(&target, &segment_path).unwrap();

        let error = load_remote_attempt_log_manifest(state.path(), &scope(), policy()).unwrap_err();
        assert!(error.contains("segment-symlink-rejected") || error.contains("segment-open-no-follow-failed"));
        assert!(fs::symlink_metadata(&segment_path).unwrap().file_type().is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn immutable_temp_symlinks_are_never_followed() {
        use std::os::unix::fs::symlink;

        let state = tempfile::tempdir().unwrap();
        let destination = state.path().join("immutable").join("object.json");
        let victim = state.path().join("victim.txt");
        fs::write(&victim, b"unchanged").unwrap();
        let mut hook = |point: PublicationHookPoint, temp: &Path, _destination: &Path| {
            if point == PublicationHookPoint::BeforeTempCreate {
                symlink(&victim, temp).map_err(|error| error.to_string())?;
            }
            Ok(())
        };
        let error = publish_immutable_file_with_hook(
            &destination,
            b"must-not-reach-victim",
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            &mut hook,
        )
        .unwrap_err();

        assert_eq!(error, "attempt-log-temp-create-attempts-exhausted");
        assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
        assert!(fs::symlink_metadata(&destination).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn manifest_temp_symlinks_are_never_followed() {
        use std::os::unix::fs::symlink;

        let state = tempfile::tempdir().unwrap();
        let paths = remote_attempt_log_paths(state.path(), &scope());
        let manifest =
            empty_remote_attempt_log_manifest(scope(), DEFAULT_REMOTE_ATTEMPT_LOG_POLICY_NAME, policy()).unwrap();
        let victim = state.path().join("manifest-victim.txt");
        fs::write(&victim, b"unchanged").unwrap();
        let mut hook = |point: PublicationHookPoint, temp: &Path, _destination: &Path| {
            if point == PublicationHookPoint::BeforeTempCreate {
                symlink(&victim, temp).map_err(|error| error.to_string())?;
            }
            Ok(())
        };
        let error = commit_manifest_with_hook(&paths, &manifest, policy(), &mut hook).unwrap_err();

        assert_eq!(error, "attempt-log-temp-create-attempts-exhausted");
        assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
        assert!(fs::symlink_metadata(&paths.manifest).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn destination_creation_race_cannot_replace_immutable_winner() {
        let state = tempfile::tempdir().unwrap();
        let destination = state.path().join("immutable").join("object.json");
        let mut hook = |point: PublicationHookPoint, _temp: &Path, destination: &Path| {
            if point == PublicationHookPoint::BeforePublish {
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(destination)
                    .and_then(|mut file| file.write_all(b"concurrent-winner"))
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        };
        let error = publish_immutable_file_with_hook(
            &destination,
            b"losing-attempt",
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            &mut hook,
        )
        .unwrap_err();

        assert_eq!(error, "attempt-log-immutable-object-conflict");
        assert_eq!(fs::read(&destination).unwrap(), b"concurrent-winner");
        assert!(fs::symlink_metadata(&destination).unwrap().is_file());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn destination_symlink_race_cannot_redirect_immutable_publish() {
        use std::os::unix::fs::symlink;

        let state = tempfile::tempdir().unwrap();
        let destination = state.path().join("immutable").join("object.json");
        let victim = state.path().join("destination-victim.txt");
        fs::write(&victim, b"unchanged").unwrap();
        let mut hook = |point: PublicationHookPoint, _temp: &Path, destination: &Path| {
            if point == PublicationHookPoint::BeforePublish {
                symlink(&victim, destination).map_err(|error| error.to_string())?;
            }
            Ok(())
        };
        let error = publish_immutable_file_with_hook(
            &destination,
            b"must-not-reach-victim",
            MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES,
            &mut hook,
        )
        .unwrap_err();

        assert!(error.contains("immutable-object-symlink-rejected"));
        assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
        assert!(fs::symlink_metadata(&destination).unwrap().file_type().is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn manifest_commit_replaces_destination_symlink_without_following_it() {
        use std::os::unix::fs::symlink;

        let state = tempfile::tempdir().unwrap();
        let paths = remote_attempt_log_paths(state.path(), &scope());
        ensure_durable_dir(&paths.root).unwrap();
        let victim = state.path().join("manifest-destination-victim.txt");
        fs::write(&victim, b"unchanged").unwrap();
        symlink(&victim, &paths.manifest).unwrap();
        let manifest =
            empty_remote_attempt_log_manifest(scope(), DEFAULT_REMOTE_ATTEMPT_LOG_POLICY_NAME, policy()).unwrap();
        commit_manifest(&paths, &manifest, policy()).unwrap();
        let observed = load_remote_attempt_log_manifest(state.path(), &scope(), policy()).unwrap();

        assert_eq!(observed, manifest);
        assert_eq!(fs::read(&victim).unwrap(), b"unchanged");
        assert!(fs::symlink_metadata(&paths.manifest).unwrap().is_file());
    }

    #[test]
    fn tampered_segment_and_anchor_fail_closed() {
        let state = tempfile::tempdir().unwrap();
        for cursor in 0..TEST_RECORD_COUNT {
            append_remote_attempt_log(
                state.path(),
                &current(),
                RemoteEventId::new(format!("tamper-{cursor}")).unwrap(),
                cursor,
                b"payload",
                policy(),
            )
            .unwrap();
        }
        let manifest = load_remote_attempt_log_manifest(state.path(), &scope(), policy()).unwrap();
        let paths = remote_attempt_log_paths(state.path(), &scope());
        let retained = manifest.segments.first().unwrap();
        let segment_path = immutable_json_path(&paths.segments, &retained.segment_blake3);
        fs::write(&segment_path, b"{}").unwrap();
        let segment_error = replay_remote_attempt_log(
            state.path(),
            &scope(),
            RemoteAttemptLogReplayRequest {
                from_cursor: manifest.retained_start_cursor,
                record_count_max: TEST_RETAINED_SEGMENTS,
                payload_bytes_max: TEST_REPLAY_BYTES,
            },
            policy(),
        )
        .unwrap_err();
        assert!(segment_error.contains("segment-parse-failed") || segment_error.contains("segment-schema-unsupported"));

        let state = tempfile::tempdir().unwrap();
        for cursor in 0..TEST_RECORD_COUNT {
            append_remote_attempt_log(
                state.path(),
                &current(),
                RemoteEventId::new(format!("anchor-tamper-{cursor}")).unwrap(),
                cursor,
                b"payload",
                policy(),
            )
            .unwrap();
        }
        let manifest = load_remote_attempt_log_manifest(state.path(), &scope(), policy()).unwrap();
        let paths = remote_attempt_log_paths(state.path(), &scope());
        let anchor = manifest.truncation_anchor.as_ref().unwrap();
        fs::write(immutable_json_path(&paths.anchors, &anchor.anchor_blake3), b"{}").unwrap();
        let anchor_error = load_remote_attempt_log_manifest(state.path(), &scope(), policy()).unwrap_err();
        assert!(anchor_error.contains("anchor-parse-failed") || anchor_error.contains("anchor-content-mismatch"));
    }
}
