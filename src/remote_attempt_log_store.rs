//! Imperative filesystem shell for immutable remote-attempt logs.
//!
//! The pure record, append, replay, chain, and retention decisions live in
//! `crunch_build::distributed::remote_attempt_log`. This module only applies
//! accepted plans with crash-ordered filesystem operations.
//!
//! r[impl remote_builds.immutable_attempt_log_segments]

use std::fs;
use std::io::Write;
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

const REMOTE_ATTEMPT_LOG_STORE_DIR: &str = "remote-attempt-logs";
const REMOTE_ATTEMPT_LOG_SEGMENTS_DIR: &str = "segments";
const REMOTE_ATTEMPT_LOG_ANCHORS_DIR: &str = "anchors";
const REMOTE_ATTEMPT_LOG_MANIFEST_FILE: &str = "manifest.json";
const JSON_FILE_EXTENSION: &str = "json";
const TEMP_FILE_EXTENSION: &str = "tmp";
const REMOTE_ATTEMPT_LOG_SCOPE_DOMAIN: &str = "mantle-remote-attempt-log-scope-v1";
const MAX_REMOTE_ATTEMPT_LOG_MANIFEST_FILE_BYTES: u64 = 2_097_152;
const MAX_REMOTE_ATTEMPT_LOG_SEGMENT_FILE_BYTES: u64 = 8_388_608;
const MAX_REMOTE_ATTEMPT_LOG_ANCHOR_FILE_BYTES: u64 = 131_072;
const DURABLE_DIRECTORY_ANCESTOR_SYNC_COUNT: usize = 2;

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

pub fn append_remote_attempt_log(
    state_dir: &Path,
    current: &RemoteAttemptLogCurrentAttemptFacts,
    event_id: RemoteEventId,
    cursor: u64,
    payload: &[u8],
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogStoreAppendResult, String> {
    policy.validate().map_err(reason)?;
    let loaded = load_remote_attempt_log(state_dir, &current.scope, policy)?;
    let previous_record_blake3 = previous_record_for_append(&loaded, &event_id, cursor)?;
    let record = seal_remote_attempt_log_record(
        RemoteAttemptLogRecordInput {
            scope: current.scope.clone(),
            event_id,
            sequence: cursor,
            cursor,
            phase: current.phase,
            stream: RemoteAttemptLogStream::Stdout,
            kind: RemoteAttemptLogRecordKind::Output,
            payload: payload.to_vec(),
            previous_record_blake3,
        },
        policy,
    )
    .map_err(reason)?;
    apply_append_plan(loaded, current, &record, policy)
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
    let paths = remote_attempt_log_paths(state_dir, scope);
    if !paths.manifest.exists() {
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
    validate_remote_attempt_log_manifest(manifest, policy).map_err(reason)?;
    let bytes = canonical_json_bytes(manifest, "manifest")?;
    if u64::try_from(bytes.len()).map_err(|_| "attempt-log-manifest-size-overflow".to_string())?
        > MAX_REMOTE_ATTEMPT_LOG_MANIFEST_FILE_BYTES
    {
        return Err("attempt-log-manifest-file-too-large".to_string());
    }
    ensure_durable_dir(&paths.root)?;
    let temp = manifest_temp_path(&paths.manifest, &manifest.manifest_blake3);
    write_synced_file(&temp, &bytes, false)?;
    fs::rename(&temp, &paths.manifest).map_err(|error| format!("attempt-log-manifest-commit-failed: {error}"))?;
    sync_directory(&paths.root)
}

fn publish_immutable_file(path: &Path, bytes: &[u8], bytes_max: u64) -> Result<(), String> {
    let byte_count = u64::try_from(bytes.len()).map_err(|_| "attempt-log-object-size-overflow".to_string())?;
    if byte_count > bytes_max {
        return Err("attempt-log-object-file-too-large".to_string());
    }
    let parent = path.parent().ok_or_else(|| "attempt-log-object-parent-missing".to_string())?;
    ensure_durable_dir(parent)?;
    if path.exists() {
        let existing = read_bounded_bytes(path, bytes_max, "immutable-object")?;
        if existing == bytes {
            return Ok(());
        }
        return Err("attempt-log-immutable-object-conflict".to_string());
    }
    let temp = path.with_extension(TEMP_FILE_EXTENSION);
    if temp.exists() {
        let staged = read_bounded_bytes(&temp, bytes_max, "immutable-staged-object")?;
        if staged != bytes {
            return Err("attempt-log-immutable-staged-object-conflict".to_string());
        }
    } else {
        write_synced_file(&temp, bytes, true)?;
    }
    match fs::rename(&temp, path) {
        Ok(()) => sync_directory(parent),
        Err(error) if path.exists() => {
            let _ = fs::remove_file(&temp);
            let existing = read_bounded_bytes(path, bytes_max, "immutable-object")?;
            if existing == bytes {
                return Ok(());
            }
            Err(format!("attempt-log-immutable-object-conflict: {error}"))
        }
        Err(error) => Err(format!("attempt-log-immutable-object-commit-failed: {error}")),
    }
}

fn write_synced_file(path: &Path, bytes: &[u8], create_new: bool) -> Result<(), String> {
    let mut options = fs::OpenOptions::new();
    options.write(true);
    if create_new {
        options.create_new(true);
    } else {
        options.create(true).truncate(true);
    }
    let mut file = options.open(path).map_err(|error| format!("attempt-log-file-open-failed: {error}"))?;
    file.write_all(bytes).map_err(|error| format!("attempt-log-file-write-failed: {error}"))?;
    file.sync_all().map_err(|error| format!("attempt-log-file-sync-failed: {error}"))
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
    let metadata = fs::metadata(path).map_err(|error| format!("attempt-log-{kind}-metadata-failed: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("attempt-log-{kind}-not-regular-file"));
    }
    if metadata.len() > bytes_max {
        return Err(format!("attempt-log-{kind}-file-too-large"));
    }
    let bytes = fs::read(path).map_err(|error| format!("attempt-log-{kind}-read-failed: {error}"))?;
    let observed = u64::try_from(bytes.len()).map_err(|_| format!("attempt-log-{kind}-size-overflow"))?;
    if observed > bytes_max {
        return Err(format!("attempt-log-{kind}-file-too-large"));
    }
    Ok(bytes)
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
    let length = u64::try_from(value.len()).expect("bounded attempt-log scope length fits u64");
    hasher.update(&length.to_le_bytes());
    hasher.update(value.as_bytes());
}

fn immutable_json_path(directory: &Path, digest: &RemoteAttemptLogDigest) -> PathBuf {
    directory.join(digest.as_str()).with_extension(JSON_FILE_EXTENSION)
}

fn manifest_temp_path(manifest_path: &Path, digest: &RemoteAttemptLogDigest) -> PathBuf {
    manifest_path.with_extension(format!("{}.{}", digest.as_str(), TEMP_FILE_EXTENSION))
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
    let directory = fs::File::open(path).map_err(|error| format!("attempt-log-directory-open-failed: {error}"))?;
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

    use super::*;

    const TEST_FENCE: u64 = 3;
    const TEST_RETAINED_SEGMENTS: u32 = 2;
    const TEST_RECORD_COUNT: u64 = 4;
    const TEST_REPLAY_BYTES: u64 = 1_024;

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
