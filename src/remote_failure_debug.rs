//! Imperative shell for portable remote-failure debug bundles.
//!
//! Pure validation, canonicalization, replay, comparison, and retention
//! decisions live in `crunch_build::distributed::remote_failure_debug`.
//!
//! r[impl operator_diagnostics.remote_failure_debug_bundle]
//! r[impl operator_diagnostics.remote_failure_replay]
//! r[impl remote_builds.failure_debug_capture]

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
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crunch_build::distributed::RemoteFailureCaptureObservation;
use crunch_build::distributed::RemoteFailureCaptureObservedKind;
use crunch_build::distributed::RemoteFailureCapturedArtifact;
use crunch_build::distributed::RemoteFailureCapturedArtifactManifest;
use crunch_build::distributed::RemoteFailureDebugBundle;
use crunch_build::distributed::RemoteFailureDebugBundleFacts;
use crunch_build::distributed::RemoteFailureDebugDigest;
use crunch_build::distributed::RemoteFailureDebugPhase;
use crunch_build::distributed::RemoteFailureDebugPolicy;
use crunch_build::distributed::RemoteFailureDebugReasonCode;
use crunch_build::distributed::RemoteFailureDebugRef;
use crunch_build::distributed::RemoteFailureImmutableLogRef;
use crunch_build::distributed::RemoteFailureInspectSummary;
use crunch_build::distributed::RemoteFailureReplayAvailability;
use crunch_build::distributed::RemoteFailureReplayPlan;
use crunch_build::distributed::RemoteFailureRetentionRecord;
use crunch_build::distributed::RemoteFailureWorkspaceMode;
use crunch_build::distributed::admit_remote_failure_capture_observations;
use crunch_build::distributed::inspect_remote_failure_debug_bundle;
use crunch_build::distributed::plan_remote_failure_capture;
use crunch_build::distributed::plan_remote_failure_replay;
use crunch_build::distributed::plan_remote_failure_retention;
use crunch_build::distributed::remote_failure_capture_object_digest;
use crunch_build::distributed::remote_failure_debug_ref;
use crunch_build::distributed::seal_remote_failure_captured_artifact_manifest;
use crunch_build::distributed::seal_remote_failure_debug_bundle;
use crunch_build::distributed::validate_remote_failure_debug_bundle;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::Serialize;

use crate::remote_build::ConcreteBuildRequest;
use crate::remote_build::RemoteAttemptLogControlSummary;
use crate::remote_build::RemoteProductionAttemptBinding;

pub const REMOTE_FAILURE_DEBUG_STORE_DIR: &str = "remote-failure-debug";
const BUNDLES_DIR: &str = "bundles";
const OBJECTS_DIR: &str = "objects";
const CAPTURE_CAS_DIR: &str = "capture-cas";
const LEASES_DIR: &str = "leases";
const MANIFEST_FILE: &str = "manifest.json";
const POLICY_FILE: &str = "policy.json";
const OBJECT_EXTENSION: &str = "json";
const CAPTURE_EXTENSION: &str = "bin";
const OBJECT_ENVELOPE_SCHEMA: &str = "mantle-remote-failure-debug-object-v1";
const LEASE_SCHEMA: &str = "mantle-remote-failure-debug-lease-v1";
const STAGE_PREFIX: &str = ".mantle-remote-failure-stage-";
const MAX_BUNDLE_DIRECTORY_ENTRIES: usize = 256;
const MAX_OBJECT_FILE_BYTES: u64 = crunch_build::distributed::MAX_REMOTE_FAILURE_OBJECT_BYTES;
const MAX_MANIFEST_FILE_BYTES: u64 = crunch_build::distributed::MAX_REMOTE_FAILURE_METADATA_BYTES;
const MAX_POLICY_FILE_BYTES: u64 = 131_072;
const MAX_LEASE_FILE_BYTES: u64 = 4_096;
const MAX_LEASES_PER_BUNDLE: usize = 64;
const RANDOM_BYTES: usize = 16;
const TEMP_CREATE_ATTEMPTS_MAX: u32 = 16;
const BOUNDED_READ_PROBE_BYTES: u64 = 1;
#[cfg(unix)]
const PRIVATE_FILE_MODE: u32 = 0o600;
#[cfg(unix)]
const PRIVATE_DIR_MODE: u32 = 0o700;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureDebugSourceFacts {
    pub request: ConcreteBuildRequest,
    pub attempt: RemoteProductionAttemptBinding,
    pub immutable_log: Option<RemoteAttemptLogControlSummary>,
    pub route_class: String,
    pub worker_capability_classes: Vec<String>,
    pub sandbox_policy_class: String,
    pub network_policy_class: String,
    pub transfer_status_class: String,
    pub admission_status_class: String,
    pub workspace_mode: RemoteFailureWorkspaceMode,
    pub failure_phase: RemoteFailureDebugPhase,
    pub failure_reason_code: String,
    pub cleanup_status_code: String,
    pub created_unix_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureDebugPublishRequest<'a> {
    pub state_dir: &'a Path,
    pub capture_root: Option<&'a Path>,
    pub facts: RemoteFailureDebugSourceFacts,
    pub policy: RemoteFailureDebugPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureDebugPublishOutcome {
    pub bundle_ref: String,
    pub bundle_blake3: RemoteFailureDebugDigest,
    pub capture_outcome_code: String,
    pub cleanup_status_code: String,
    pub immutable_log_available: bool,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteFailureDebugObjectEnvelope {
    schema: String,
    kind: String,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteFailureRouteObject {
    route_class: String,
    job_id: String,
    attempt_id: String,
    fence_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteFailureWorkerObject {
    capability_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteFailureInputObject {
    input_refs: Vec<String>,
    source_input_refs: Vec<String>,
    upload_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteFailureClassObject {
    class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteFailureLeaseRecord {
    schema: String,
    bundle_blake3: RemoteFailureDebugDigest,
    expires_unix_s: u64,
}

#[derive(Debug)]
pub struct RemoteFailureDebugLease {
    path: PathBuf,
}

impl Drop for RemoteFailureDebugLease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureDebugRetentionReport {
    pub preserved_count: u32,
    pub deleted_count: u32,
    pub failed_deletions: Vec<String>,
}

#[derive(Debug)]
struct BundleStage {
    stage_path: PathBuf,
    final_path: PathBuf,
}

pub fn publish_remote_failure_debug_bundle(
    request: RemoteFailureDebugPublishRequest<'_>,
) -> Result<RemoteFailureDebugPublishOutcome, String> {
    request.policy.validate().map_err(reason)?;
    let bundles_root = request.state_dir.join(REMOTE_FAILURE_DEBUG_STORE_DIR).join(BUNDLES_DIR);
    ensure_private_directory(&bundles_root)?;
    let stage = create_bundle_stage(&bundles_root)?;
    let result = assemble_bundle_stage(&stage, &request);
    match result {
        Ok((bundle, capture_outcome_code)) => commit_bundle_stage(stage, bundle, capture_outcome_code),
        Err(error) => {
            let cleanup = fs::remove_dir_all(&stage.stage_path);
            if let Err(cleanup_error) = cleanup {
                return Err(format!("{error}; remote-failure-stage-cleanup-failed: {cleanup_error}"));
            }
            Err(error)
        }
    }
}

pub fn load_and_validate_remote_failure_debug_bundle(
    bundle_dir: &Path,
) -> Result<(RemoteFailureDebugBundle, RemoteFailureDebugPolicy), String> {
    require_real_directory(bundle_dir, "bundle")?;
    let policy: RemoteFailureDebugPolicy =
        read_bounded_json(&bundle_dir.join(POLICY_FILE), MAX_POLICY_FILE_BYTES, "policy")?;
    policy.validate().map_err(reason)?;
    let bundle: RemoteFailureDebugBundle =
        read_bounded_json(&bundle_dir.join(MANIFEST_FILE), MAX_MANIFEST_FILE_BYTES, "manifest")?;
    validate_remote_failure_debug_bundle(&bundle, &policy).map_err(reason)?;
    validate_bundle_object_refs(bundle_dir, &bundle, &policy)?;
    validate_capture_manifest_and_objects(bundle_dir, &bundle, &policy)?;
    Ok((bundle, policy))
}

pub fn inspect_remote_failure_debug_bundle_from_disk(bundle_dir: &Path) -> Result<RemoteFailureInspectSummary, String> {
    let (bundle, policy) = load_and_validate_remote_failure_debug_bundle(bundle_dir)?;
    inspect_remote_failure_debug_bundle(&bundle, &policy).map_err(reason)
}

pub fn plan_remote_failure_replay_from_disk(bundle_dir: &Path) -> Result<RemoteFailureReplayPlan, String> {
    let (bundle, policy) = load_and_validate_remote_failure_debug_bundle(bundle_dir)?;
    let available_refs = collect_available_object_refs(bundle_dir, &bundle)?;
    plan_remote_failure_replay(&bundle, &policy, &RemoteFailureReplayAvailability {
        available_refs,
        current_policy_allows_replay: policy.replay_enabled,
    })
    .map_err(reason)
}

pub fn load_remote_failure_replay_request(bundle_dir: &Path) -> Result<ConcreteBuildRequest, String> {
    let (bundle, policy) = load_and_validate_remote_failure_debug_bundle(bundle_dir)?;
    let plan = plan_remote_failure_replay_from_disk(bundle_dir)?;
    if !plan.executable {
        return Err(format!("remote-failure-replay-blocked:{}", plan.blockers.join(",")));
    }
    let envelope = read_object_envelope(bundle_dir, &bundle.action_ref, &policy)?;
    if envelope.kind != "action-request" {
        return Err("remote-failure-action-object-kind-mismatch".to_string());
    }
    let mut replay: ConcreteBuildRequest = serde_json::from_value(envelope.payload)
        .map_err(|error| format!("remote-failure-action-object-invalid: {error}"))?;
    replay.production_attempt = None;
    replay.transfer_policy = None;
    validate_replay_request_matches_bundle(&replay, &bundle)?;
    Ok(replay)
}

pub fn acquire_remote_failure_debug_lease(
    bundle_dir: &Path,
    now_unix_s: u64,
    lease_duration_secs: u64,
) -> Result<RemoteFailureDebugLease, String> {
    let (bundle, _) = load_and_validate_remote_failure_debug_bundle(bundle_dir)?;
    if lease_duration_secs == 0 {
        return Err("remote-failure-lease-duration-zero".to_string());
    }
    let expires_unix_s = now_unix_s
        .checked_add(lease_duration_secs)
        .ok_or_else(|| "remote-failure-lease-expiry-overflow".to_string())?;
    let leases = bundle_dir.join(LEASES_DIR);
    ensure_private_directory(&leases)?;
    if count_directory_entries_bounded(&leases, MAX_LEASES_PER_BUNDLE)? >= MAX_LEASES_PER_BUNDLE {
        return Err("remote-failure-lease-limit-exceeded".to_string());
    }
    let path = leases.join(format!("{}.json", random_hex()?));
    let record = RemoteFailureLeaseRecord {
        schema: LEASE_SCHEMA.to_string(),
        bundle_blake3: bundle.bundle_blake3,
        expires_unix_s,
    };
    write_new_private_file(&path, &serde_json::to_vec(&record).map_err(|error| error.to_string())?)?;
    Ok(RemoteFailureDebugLease { path })
}

pub fn retain_remote_failure_debug_bundles(
    state_dir: &Path,
    now_unix_s: u64,
) -> Result<RemoteFailureDebugRetentionReport, String> {
    let bundles_root = state_dir.join(REMOTE_FAILURE_DEBUG_STORE_DIR).join(BUNDLES_DIR);
    if !bundles_root.exists() {
        return Ok(RemoteFailureDebugRetentionReport {
            preserved_count: 0,
            deleted_count: 0,
            failed_deletions: Vec::new(),
        });
    }
    require_real_directory(&bundles_root, "bundles-root")?;
    let entries = collect_bundle_directories(&bundles_root)?;
    let mut records = Vec::with_capacity(entries.len());
    for entry in &entries {
        let (bundle, _) = load_and_validate_remote_failure_debug_bundle(entry)?;
        records.push(RemoteFailureRetentionRecord {
            bundle_blake3: bundle.bundle_blake3,
            expires_unix_s: bundle.expires_unix_s,
            active_lease: bundle_has_active_lease(entry, now_unix_s)?,
            ordinary_build_output: false,
        });
    }
    let plan = plan_remote_failure_retention(records, now_unix_s).map_err(reason)?;
    let mut failed_deletions = Vec::new();
    let mut deleted_count = 0_u32;
    for digest in &plan.delete {
        let path = bundles_root.join(digest.as_str());
        match fs::remove_dir_all(&path) {
            Ok(()) => deleted_count = deleted_count.saturating_add(1),
            Err(error) => failed_deletions.push(format!("delete-failed:{error}")),
        }
    }
    let preserved_count = u32::try_from(plan.preserve.len()).map_err(|_| "remote-failure-preserve-count-overflow")?;
    Ok(RemoteFailureDebugRetentionReport {
        preserved_count,
        deleted_count,
        failed_deletions,
    })
}

fn assemble_bundle_stage(
    stage: &BundleStage,
    request: &RemoteFailureDebugPublishRequest<'_>,
) -> Result<(RemoteFailureDebugBundle, String), String> {
    ensure_private_directory(&stage.stage_path.join(OBJECTS_DIR))?;
    ensure_private_directory(&stage.stage_path.join(CAPTURE_CAS_DIR))?;
    let action_ref = write_json_object(&stage.stage_path, "action-request", &request.facts.request, &request.policy)?;
    let route_ref = write_json_object(
        &stage.stage_path,
        "route-assignment",
        &RemoteFailureRouteObject {
            route_class: request.facts.route_class.clone(),
            job_id: request.facts.attempt.job_id.as_str().to_string(),
            attempt_id: request.facts.attempt.attempt_id.as_str().to_string(),
            fence_generation: request.facts.attempt.fence_generation.get(),
        },
        &request.policy,
    )?;
    let mut capabilities = request.facts.worker_capability_classes.clone();
    capabilities.sort();
    capabilities.dedup();
    let worker_capability_ref = write_json_object(
        &stage.stage_path,
        "worker-capabilities",
        &RemoteFailureWorkerObject {
            capability_classes: capabilities,
        },
        &request.policy,
    )?;
    let input_manifest_ref = write_json_object(
        &stage.stage_path,
        "declared-inputs",
        &RemoteFailureInputObject {
            input_refs: sorted_unique(request.facts.request.input_refs.clone()),
            source_input_refs: sorted_unique(request.facts.request.source_input_refs.clone()),
            upload_bytes: request.facts.request.upload_bytes,
        },
        &request.policy,
    )?;
    let sandbox_policy_ref =
        write_class_object(&stage.stage_path, "sandbox-policy", &request.facts.sandbox_policy_class, &request.policy)?;
    let network_policy_ref =
        write_class_object(&stage.stage_path, "network-policy", &request.facts.network_policy_class, &request.policy)?;
    let transfer_ref = optional_class_object(
        &stage.stage_path,
        "transfer-state",
        &request.facts.transfer_status_class,
        &request.policy,
    )?;
    let admission_ref = optional_class_object(
        &stage.stage_path,
        "admission-state",
        &request.facts.admission_status_class,
        &request.policy,
    )?;
    let (captured_artifact_manifest_ref, capture_outcome_code) = capture_failed_artifacts(stage, request)?;
    let immutable_log = request.facts.immutable_log.as_ref().map(immutable_log_ref_from_summary).transpose()?;
    let expires_unix_s = request
        .facts
        .created_unix_s
        .checked_add(request.policy.retention_secs)
        .ok_or_else(|| "remote-failure-expiry-overflow".to_string())?;
    let facts = RemoteFailureDebugBundleFacts {
        action_ref,
        route_ref,
        worker_capability_ref,
        input_manifest_ref,
        sandbox_policy_ref,
        network_policy_ref,
        immutable_log,
        transfer_ref,
        admission_ref,
        captured_artifact_manifest_ref,
        original_job_id: request.facts.attempt.job_id.clone(),
        original_attempt_id: request.facts.attempt.attempt_id.clone(),
        original_fence_generation: request.facts.attempt.fence_generation,
        workspace_mode: request.facts.workspace_mode,
        failure_phase: request.facts.failure_phase,
        failure_reason_code: request.facts.failure_reason_code.clone(),
        capture_outcome_code: capture_outcome_code.clone(),
        cleanup_status_code: request.facts.cleanup_status_code.clone(),
        created_unix_s: request.facts.created_unix_s,
        expires_unix_s,
        non_claims: Vec::new(),
    };
    let bundle = seal_remote_failure_debug_bundle(facts, &request.policy).map_err(reason)?;
    write_new_private_file(
        &stage.stage_path.join(POLICY_FILE),
        &serde_json::to_vec(&request.policy)
            .map_err(|error| format!("remote-failure-policy-serialize-failed:{error}"))?,
    )?;
    write_new_private_file(
        &stage.stage_path.join(MANIFEST_FILE),
        &serde_json::to_vec(&bundle).map_err(|error| format!("remote-failure-manifest-serialize-failed:{error}"))?,
    )?;
    sync_directory(&stage.stage_path)?;
    Ok((bundle, capture_outcome_code))
}

fn commit_bundle_stage(
    mut stage: BundleStage,
    bundle: RemoteFailureDebugBundle,
    capture_outcome_code: String,
) -> Result<RemoteFailureDebugPublishOutcome, String> {
    stage.final_path = stage
        .final_path
        .parent()
        .ok_or_else(|| "remote-failure-final-parent-missing".to_string())?
        .join(bundle.bundle_blake3.as_str());
    match rename_no_replace(&stage.stage_path, &stage.final_path) {
        Ok(()) => sync_directory(stage.final_path.parent().expect("bundle final has parent"))?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            fs::remove_dir_all(&stage.stage_path)
                .map_err(|cleanup| format!("remote-failure-idempotent-stage-cleanup-failed:{cleanup}"))?;
            let (existing, _) = load_and_validate_remote_failure_debug_bundle(&stage.final_path)?;
            if existing != bundle {
                return Err("remote-failure-no-clobber-conflict".to_string());
            }
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&stage.stage_path);
            return Err(format!("remote-failure-bundle-publish-failed:{error}"));
        }
    }
    Ok(RemoteFailureDebugPublishOutcome {
        bundle_ref: format!("remote-failure-debug:{}", bundle.bundle_blake3.as_str()),
        bundle_blake3: bundle.bundle_blake3,
        capture_outcome_code,
        cleanup_status_code: bundle.cleanup_status_code,
        immutable_log_available: bundle.immutable_log.is_some(),
        non_claim: crunch_build::distributed::REMOTE_FAILURE_DEBUG_NON_CLAIM.to_string(),
    })
}

fn capture_failed_artifacts(
    stage: &BundleStage,
    request: &RemoteFailureDebugPublishRequest<'_>,
) -> Result<(Option<RemoteFailureDebugRef>, String), String> {
    let capture_requests = plan_remote_failure_capture(&request.policy.capture).map_err(reason)?;
    if capture_requests.is_empty() {
        return Ok((None, "metadata-only".to_string()));
    }
    let Some(capture_root) = request.capture_root else {
        return Ok((None, "capture-root-unavailable".to_string()));
    };
    require_real_directory(capture_root, "capture-root")?;
    let observations = observe_capture_requests(capture_root, &capture_requests)?;
    let admission = admit_remote_failure_capture_observations(&capture_requests, observations, &request.policy.capture)
        .map_err(reason)?;
    if admission.accepted_paths.is_empty() {
        return Ok((None, "capture-rejected".to_string()));
    }
    let mut artifacts = Vec::with_capacity(admission.accepted_paths.len());
    for relative_path in &admission.accepted_paths {
        let bytes = read_capture_file_nofollow(capture_root, relative_path, request.policy.capture.file_bytes_max)?;
        let object_blake3 = remote_failure_capture_object_digest(&bytes);
        let object_path = stage
            .stage_path
            .join(CAPTURE_CAS_DIR)
            .join(object_blake3.as_str())
            .with_extension(CAPTURE_EXTENSION);
        write_new_private_file(&object_path, &bytes)?;
        artifacts.push(RemoteFailureCapturedArtifact {
            relative_path: relative_path.clone(),
            sensitivity: request.policy.capture.sensitivity,
            byte_count: u64::try_from(bytes.len()).map_err(|_| "remote-failure-capture-size-overflow".to_string())?,
            object_blake3,
        });
    }
    let manifest =
        seal_remote_failure_captured_artifact_manifest(artifacts, &request.policy.capture).map_err(reason)?;
    let manifest_ref = write_json_object(&stage.stage_path, "captured-artifact-manifest", &manifest, &request.policy)?;
    Ok((Some(manifest_ref), "captured".to_string()))
}

fn observe_capture_requests(
    root: &Path,
    requests: &[crunch_build::distributed::RemoteFailureCaptureRequest],
) -> Result<Vec<RemoteFailureCaptureObservation>, String> {
    let mut observations = Vec::with_capacity(requests.len());
    for request in requests {
        let path = confined_relative_path(root, &request.relative_path)?;
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| format!("remote-failure-capture-metadata-failed:{error}"))?;
        observations.push(RemoteFailureCaptureObservation {
            relative_path: request.relative_path.clone(),
            kind: observed_kind(&metadata),
            byte_count: metadata.len(),
        });
    }
    Ok(observations)
}

fn read_capture_file_nofollow(root: &Path, relative_path: &str, bytes_max: u64) -> Result<Vec<u8>, String> {
    let path = confined_relative_path(root, relative_path)?;
    read_bounded_bytes(&path, bytes_max, "capture")
}

fn confined_relative_path(root: &Path, relative_path: &str) -> Result<PathBuf, String> {
    let relative = Path::new(relative_path);
    if relative.is_absolute() || relative.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err("remote-failure-capture-path-invalid".to_string());
    }
    let mut current = root.to_path_buf();
    let components = relative.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("remote-failure-capture-path-invalid".to_string());
        };
        current.push(name);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("remote-failure-capture-component-metadata-failed:{error}"))?;
        if metadata.file_type().is_symlink() {
            return Err("remote-failure-capture-symlink-rejected".to_string());
        }
        if index + 1 < components.len() && !metadata.is_dir() {
            return Err("remote-failure-capture-parent-not-directory".to_string());
        }
    }
    Ok(current)
}

fn observed_kind(metadata: &fs::Metadata) -> RemoteFailureCaptureObservedKind {
    let file_type = metadata.file_type();
    if file_type.is_file() {
        return RemoteFailureCaptureObservedKind::RegularFile;
    }
    if file_type.is_dir() {
        return RemoteFailureCaptureObservedKind::Directory;
    }
    if file_type.is_symlink() {
        return RemoteFailureCaptureObservedKind::Symlink;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if file_type.is_socket() {
            return RemoteFailureCaptureObservedKind::Socket;
        }
        if file_type.is_fifo() {
            return RemoteFailureCaptureObservedKind::Fifo;
        }
        if file_type.is_block_device() || file_type.is_char_device() {
            return RemoteFailureCaptureObservedKind::Device;
        }
    }
    RemoteFailureCaptureObservedKind::Unsupported
}

fn immutable_log_ref_from_summary(
    summary: &RemoteAttemptLogControlSummary,
) -> Result<RemoteFailureImmutableLogRef, String> {
    Ok(RemoteFailureImmutableLogRef {
        scope: summary.scope.clone(),
        manifest_blake3: summary.manifest_blake3.clone(),
        retained_start_cursor: summary.retained_start_cursor,
        next_cursor: summary.next_cursor,
        head_record_blake3: summary.head_record_blake3.clone(),
        head_segment_blake3: summary.head_segment_blake3.clone(),
        truncated: summary.truncated,
    })
}

fn write_class_object(
    stage_path: &Path,
    kind: &str,
    class: &str,
    policy: &RemoteFailureDebugPolicy,
) -> Result<RemoteFailureDebugRef, String> {
    write_json_object(
        stage_path,
        kind,
        &RemoteFailureClassObject {
            class: class.to_string(),
        },
        policy,
    )
}

fn optional_class_object(
    stage_path: &Path,
    kind: &str,
    class: &str,
    policy: &RemoteFailureDebugPolicy,
) -> Result<Option<RemoteFailureDebugRef>, String> {
    if class == "not-available" {
        return Ok(None);
    }
    write_class_object(stage_path, kind, class, policy).map(Some)
}

fn write_json_object(
    stage_path: &Path,
    kind: &str,
    payload: &impl Serialize,
    policy: &RemoteFailureDebugPolicy,
) -> Result<RemoteFailureDebugRef, String> {
    let payload =
        serde_json::to_value(payload).map_err(|error| format!("remote-failure-object-value-failed:{error}"))?;
    let envelope = RemoteFailureDebugObjectEnvelope {
        schema: OBJECT_ENVELOPE_SCHEMA.to_string(),
        kind: kind.to_string(),
        payload,
    };
    let bytes =
        serde_json::to_vec(&envelope).map_err(|error| format!("remote-failure-object-serialize-failed:{error}"))?;
    let reference = remote_failure_debug_ref(kind, &bytes, policy).map_err(reason)?;
    let path = stage_path.join(OBJECTS_DIR).join(reference.digest_blake3.as_str()).with_extension(OBJECT_EXTENSION);
    write_new_private_file(&path, &bytes)?;
    Ok(reference)
}

fn read_object_envelope(
    bundle_dir: &Path,
    reference: &RemoteFailureDebugRef,
    policy: &RemoteFailureDebugPolicy,
) -> Result<RemoteFailureDebugObjectEnvelope, String> {
    let path = object_path(bundle_dir, reference);
    let bytes = read_bounded_bytes(&path, policy.object_bytes_max, "object")?;
    let observed = remote_failure_debug_ref(reference.kind.clone(), &bytes, policy).map_err(reason)?;
    if observed != *reference {
        return Err("remote-failure-object-identity-mismatch".to_string());
    }
    let envelope: RemoteFailureDebugObjectEnvelope =
        serde_json::from_slice(&bytes).map_err(|error| format!("remote-failure-object-parse-failed:{error}"))?;
    if envelope.schema != OBJECT_ENVELOPE_SCHEMA || envelope.kind != reference.kind {
        return Err("remote-failure-object-envelope-mismatch".to_string());
    }
    Ok(envelope)
}

fn validate_bundle_object_refs(
    bundle_dir: &Path,
    bundle: &RemoteFailureDebugBundle,
    policy: &RemoteFailureDebugPolicy,
) -> Result<(), String> {
    for reference in required_and_optional_refs(bundle) {
        read_object_envelope(bundle_dir, reference, policy)?;
    }
    Ok(())
}

fn validate_capture_manifest_and_objects(
    bundle_dir: &Path,
    bundle: &RemoteFailureDebugBundle,
    policy: &RemoteFailureDebugPolicy,
) -> Result<(), String> {
    let Some(reference) = &bundle.captured_artifact_manifest_ref else {
        return Ok(());
    };
    let envelope = read_object_envelope(bundle_dir, reference, policy)?;
    let manifest: RemoteFailureCapturedArtifactManifest = serde_json::from_value(envelope.payload)
        .map_err(|error| format!("remote-failure-capture-manifest-invalid:{error}"))?;
    let resealed =
        seal_remote_failure_captured_artifact_manifest(manifest.artifacts.clone(), &policy.capture).map_err(reason)?;
    if resealed != manifest {
        return Err("remote-failure-capture-manifest-identity-mismatch".to_string());
    }
    for artifact in &manifest.artifacts {
        let path = bundle_dir
            .join(CAPTURE_CAS_DIR)
            .join(artifact.object_blake3.as_str())
            .with_extension(CAPTURE_EXTENSION);
        let bytes = read_bounded_bytes(&path, policy.capture.file_bytes_max, "capture-object")?;
        if remote_failure_capture_object_digest(&bytes) != artifact.object_blake3 {
            return Err("remote-failure-capture-object-identity-mismatch".to_string());
        }
    }
    Ok(())
}

fn collect_available_object_refs(
    bundle_dir: &Path,
    bundle: &RemoteFailureDebugBundle,
) -> Result<std::collections::BTreeSet<RemoteFailureDebugDigest>, String> {
    let mut available = std::collections::BTreeSet::new();
    for reference in required_and_optional_refs(bundle) {
        if object_path(bundle_dir, reference).is_file() {
            available.insert(reference.digest_blake3.clone());
        }
    }
    Ok(available)
}

fn required_and_optional_refs(bundle: &RemoteFailureDebugBundle) -> Vec<&RemoteFailureDebugRef> {
    let mut refs = vec![
        &bundle.action_ref,
        &bundle.route_ref,
        &bundle.worker_capability_ref,
        &bundle.input_manifest_ref,
        &bundle.sandbox_policy_ref,
        &bundle.network_policy_ref,
    ];
    refs.extend(bundle.transfer_ref.iter());
    refs.extend(bundle.admission_ref.iter());
    refs.extend(bundle.captured_artifact_manifest_ref.iter());
    refs
}

fn object_path(bundle_dir: &Path, reference: &RemoteFailureDebugRef) -> PathBuf {
    bundle_dir.join(OBJECTS_DIR).join(reference.digest_blake3.as_str()).with_extension(OBJECT_EXTENSION)
}

fn validate_replay_request_matches_bundle(
    request: &ConcreteBuildRequest,
    bundle: &RemoteFailureDebugBundle,
) -> Result<(), String> {
    if request.production_attempt.is_some() || request.transfer_policy.is_some() {
        return Err("remote-failure-replay-stale-authority-present".to_string());
    }
    if request.request_id.is_empty() || bundle.original_fence_generation.get() == 0 {
        return Err("remote-failure-replay-request-invalid".to_string());
    }
    Ok(())
}

fn create_bundle_stage(bundles_root: &Path) -> Result<BundleStage, String> {
    for _ in 0..TEMP_CREATE_ATTEMPTS_MAX {
        let stage_path = bundles_root.join(format!("{STAGE_PREFIX}{}", random_hex()?));
        match fs::create_dir(&stage_path) {
            Ok(()) => {
                set_private_directory_mode(&stage_path)?;
                return Ok(BundleStage {
                    stage_path,
                    final_path: bundles_root.join("pending"),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("remote-failure-stage-create-failed:{error}")),
        }
    }
    Err("remote-failure-stage-create-attempts-exhausted".to_string())
}

fn collect_bundle_directories(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| format!("remote-failure-bundles-read-failed:{error}"))? {
        let entry = entry.map_err(|error| format!("remote-failure-bundle-entry-failed:{error}"))?;
        if paths.len() >= crunch_build::distributed::MAX_REMOTE_FAILURE_RETENTION_RECORDS {
            return Err("remote-failure-bundle-count-limit-exceeded".to_string());
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with(STAGE_PREFIX) {
            continue;
        }
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| format!("remote-failure-bundle-metadata-failed:{error}"))?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn bundle_has_active_lease(bundle_dir: &Path, now_unix_s: u64) -> Result<bool, String> {
    let leases = bundle_dir.join(LEASES_DIR);
    if !leases.exists() {
        return Ok(false);
    }
    require_real_directory(&leases, "leases")?;
    let entries = fs::read_dir(&leases).map_err(|error| format!("remote-failure-leases-read-failed:{error}"))?;
    for (index, entry) in entries.enumerate() {
        if index >= MAX_LEASES_PER_BUNDLE {
            return Err("remote-failure-lease-limit-exceeded".to_string());
        }
        let path = entry.map_err(|error| format!("remote-failure-lease-entry-failed:{error}"))?.path();
        let lease: RemoteFailureLeaseRecord = read_bounded_json(&path, MAX_LEASE_FILE_BYTES, "lease")?;
        if lease.schema == LEASE_SCHEMA && lease.expires_unix_s > now_unix_s {
            return Ok(true);
        }
    }
    Ok(false)
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn count_directory_entries_bounded(path: &Path, limit: usize) -> Result<usize, String> {
    let mut count = 0_usize;
    for entry in fs::read_dir(path).map_err(|error| format!("remote-failure-directory-read-failed:{error}"))? {
        entry.map_err(|error| format!("remote-failure-directory-entry-failed:{error}"))?;
        count = count.saturating_add(1);
        if count > limit {
            return Err("remote-failure-directory-entry-limit-exceeded".to_string());
        }
    }
    Ok(count)
}

fn require_real_directory(path: &Path, kind: &str) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("remote-failure-{kind}-metadata-failed:{error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("remote-failure-{kind}-not-real-directory"));
    }
    if count_directory_entries_bounded(path, MAX_BUNDLE_DIRECTORY_ENTRIES).is_err() && kind == "bundle" {
        return Err("remote-failure-bundle-entry-limit-exceeded".to_string());
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|error| format!("remote-failure-directory-create-failed:{error}"))?;
    require_real_directory(path, "directory")?;
    set_private_directory_mode(path)?;
    sync_directory(path)
}

#[cfg(unix)]
fn set_private_directory_mode(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIR_MODE))
        .map_err(|error| format!("remote-failure-directory-mode-failed:{error}"))
}

#[cfg(not(unix))]
fn set_private_directory_mode(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn write_new_private_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if u64::try_from(bytes.len()).map_err(|_| "remote-failure-write-size-overflow")? > MAX_OBJECT_FILE_BYTES {
        return Err("remote-failure-write-size-limit-exceeded".to_string());
    }
    let mut file = create_new_private_file(path)?;
    file.write_all(bytes).map_err(|error| format!("remote-failure-file-write-failed:{error}"))?;
    file.sync_all().map_err(|error| format!("remote-failure-file-sync-failed:{error}"))?;
    sync_directory(path.parent().ok_or_else(|| "remote-failure-file-parent-missing".to_string())?)
}

#[cfg(unix)]
fn create_new_private_file(path: &Path) -> Result<File, String> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| format!("remote-failure-file-create-failed:{error}"))
}

#[cfg(not(unix))]
fn create_new_private_file(_path: &Path) -> Result<File, String> {
    Err("remote-failure-secure-file-create-unsupported".to_string())
}

fn read_bounded_json<T: serde::de::DeserializeOwned>(path: &Path, bytes_max: u64, kind: &str) -> Result<T, String> {
    let bytes = read_bounded_bytes(path, bytes_max, kind)?;
    serde_json::from_slice(&bytes).map_err(|error| format!("remote-failure-{kind}-parse-failed:{error}"))
}

fn read_bounded_bytes(path: &Path, bytes_max: u64, kind: &str) -> Result<Vec<u8>, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("remote-failure-{kind}-metadata-failed:{error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("remote-failure-{kind}-not-regular-file"));
    }
    if metadata.len() > bytes_max {
        return Err(format!("remote-failure-{kind}-too-large"));
    }
    let mut file = open_regular_no_follow(path, kind)?;
    let opened = file.metadata().map_err(|error| format!("remote-failure-{kind}-opened-metadata-failed:{error}"))?;
    if !opened.is_file() || opened.len() != metadata.len() {
        return Err(format!("remote-failure-{kind}-type-or-size-drift"));
    }
    let read_limit = bytes_max
        .checked_add(BOUNDED_READ_PROBE_BYTES)
        .ok_or_else(|| format!("remote-failure-{kind}-read-limit-overflow"))?;
    let capacity = usize::try_from(opened.len()).map_err(|_| format!("remote-failure-{kind}-capacity-overflow"))?;
    let mut bytes = Vec::with_capacity(capacity);
    Read::by_ref(&mut file)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("remote-failure-{kind}-read-failed:{error}"))?;
    if u64::try_from(bytes.len()).map_err(|_| format!("remote-failure-{kind}-size-overflow"))? > bytes_max {
        return Err(format!("remote-failure-{kind}-too-large"));
    }
    Ok(bytes)
}

#[cfg(unix)]
fn open_regular_no_follow(path: &Path, kind: &str) -> Result<File, String> {
    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|error| format!("remote-failure-{kind}-open-no-follow-failed:{error}"))
}

#[cfg(not(unix))]
fn open_regular_no_follow(_path: &Path, kind: &str) -> Result<File, String> {
    Err(format!("remote-failure-{kind}-secure-read-unsupported"))
}

#[cfg(target_os = "linux")]
fn rename_no_replace(source: &Path, destination: &Path) -> std::io::Result<()> {
    let source = CString::new(source.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "remote failure source contains NUL"))?;
    let destination = CString::new(destination.as_os_str().as_bytes()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "remote failure destination contains NUL")
    })?;
    // SAFETY: both pointers are live C strings and RENAME_NOREPLACE performs
    // one race-free no-clobber publication within the same parent filesystem.
    let result = unsafe {
        libc::renameat2(libc::AT_FDCWD, source.as_ptr(), libc::AT_FDCWD, destination.as_ptr(), libc::RENAME_NOREPLACE)
    };
    if result == 0 {
        return Ok(());
    }
    Err(std::io::Error::last_os_error())
}

#[cfg(not(target_os = "linux"))]
fn rename_no_replace(_source: &Path, _destination: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic no-replace remote failure publication is unsupported",
    ))
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    let directory = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| format!("remote-failure-directory-open-failed:{error}"))?;
    directory.sync_all().map_err(|error| format!("remote-failure-directory-sync-failed:{error}"))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn random_hex() -> Result<String, String> {
    let mut bytes = [0_u8; RANDOM_BYTES];
    OsRng.try_fill_bytes(&mut bytes).map_err(|error| format!("remote-failure-random-failed:{error}"))?;
    let mut rendered = String::with_capacity(RANDOM_BYTES.saturating_mul(2));
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut rendered, "{byte:02x}").map_err(|_| "remote-failure-random-format-failed".to_string())?;
    }
    Ok(rendered)
}

fn reason(reason: RemoteFailureDebugReasonCode) -> String {
    reason.as_str().to_string()
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use crunch_build::distributed::REMOTE_FAILURE_DEBUG_NON_CLAIM;
    use crunch_build::distributed::RemoteAttemptId;
    use crunch_build::distributed::RemoteAttemptLogDigest;
    use crunch_build::distributed::RemoteAttemptLogScope;
    use crunch_build::distributed::RemoteFailureCapturePolicy;
    use crunch_build::distributed::RemoteFailureCaptureSensitivity;
    use crunch_build::distributed::RemoteFenceGeneration;
    use crunch_build::distributed::RemoteJobId;

    use super::*;

    const NOW_UNIX_S: u64 = 10_000;
    const LEASE_DURATION_SECS: u64 = 300;
    const TEST_FENCE: u64 = 2;

    fn request() -> ConcreteBuildRequest {
        ConcreteBuildRequest {
            request_id: "request-debug".to_string(),
            store_prefix: "/mantle/store".to_string(),
            input_refs: vec!["/mantle/store/input".to_string()],
            source_input_refs: Vec::new(),
            upload_bytes: 0,
            build_time_limit_secs: 60,
            contains_raw_frontend_eval: false,
            payload: crate::remote_build::RemoteConcreteBuildPayload::Action {
                action_id: "debug-action".to_string(),
                spec_json: r#"{"schema":"mantle-remote-action-v1","action_id":"debug-action","builder":"/bin/false","outputs":["out"]}"#.to_string(),
            },
            expected_outputs: vec![crate::remote_build::RemoteExpectedOutput {
                name: "out".to_string(),
                logical_path: None,
            }],
            production_attempt: None,
            transfer_policy: None,
        }
    }

    fn attempt() -> RemoteProductionAttemptBinding {
        RemoteProductionAttemptBinding {
            job_id: RemoteJobId::new("debug-job").unwrap(),
            attempt_id: RemoteAttemptId::new("debug-attempt").unwrap(),
            fence_generation: RemoteFenceGeneration::new(TEST_FENCE).unwrap(),
        }
    }

    fn log_summary() -> RemoteAttemptLogControlSummary {
        let attempt = attempt();
        RemoteAttemptLogControlSummary {
            scope: RemoteAttemptLogScope {
                job_id: attempt.job_id,
                attempt_id: attempt.attempt_id,
                fence_generation: attempt.fence_generation,
            },
            retention_policy: crate::remote_build::RemoteLogRetentionPolicy::default(),
            retained_start_cursor: 0,
            next_cursor: 1,
            retained_record_count: 1,
            retained_payload_bytes: 1,
            head_record_blake3: Some(RemoteAttemptLogDigest::new("a".repeat(64)).unwrap()),
            head_segment_blake3: Some(RemoteAttemptLogDigest::new("b".repeat(64)).unwrap()),
            manifest_blake3: RemoteAttemptLogDigest::new("c".repeat(64)).unwrap(),
            truncation_anchor_blake3: None,
            truncated: false,
        }
    }

    fn facts() -> RemoteFailureDebugSourceFacts {
        RemoteFailureDebugSourceFacts {
            request: request(),
            attempt: attempt(),
            immutable_log: Some(log_summary()),
            route_class: "remote-stdio".to_string(),
            worker_capability_classes: vec!["exact".to_string()],
            sandbox_policy_class: "native".to_string(),
            network_policy_class: "none".to_string(),
            transfer_status_class: "not-available".to_string(),
            admission_status_class: "not-admitted".to_string(),
            workspace_mode: RemoteFailureWorkspaceMode::Ephemeral,
            failure_phase: RemoteFailureDebugPhase::Execution,
            failure_reason_code: "sandbox-exit-nonzero".to_string(),
            cleanup_status_code: "cleanup-complete".to_string(),
            created_unix_s: NOW_UNIX_S,
        }
    }

    fn bundle_dir(state: &Path, outcome: &RemoteFailureDebugPublishOutcome) -> PathBuf {
        state.join(REMOTE_FAILURE_DEBUG_STORE_DIR).join(BUNDLES_DIR).join(outcome.bundle_blake3.as_str())
    }

    #[test]
    fn metadata_only_bundle_publishes_no_clobber_and_inspects_without_sandbox() {
        let state = tempfile::tempdir().unwrap();
        let policy = RemoteFailureDebugPolicy {
            replay_enabled: true,
            ..RemoteFailureDebugPolicy::default()
        };
        let first = publish_remote_failure_debug_bundle(RemoteFailureDebugPublishRequest {
            state_dir: state.path(),
            capture_root: None,
            facts: facts(),
            policy: policy.clone(),
        })
        .unwrap();
        let second = publish_remote_failure_debug_bundle(RemoteFailureDebugPublishRequest {
            state_dir: state.path(),
            capture_root: None,
            facts: facts(),
            policy,
        })
        .unwrap();
        let path = bundle_dir(state.path(), &first);
        let inspect = inspect_remote_failure_debug_bundle_from_disk(&path).unwrap();
        let replay = plan_remote_failure_replay_from_disk(&path).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.capture_outcome_code, "metadata-only");
        assert!(inspect.immutable_log_available);
        assert!(replay.executable);
        assert!(replay.new_authority_required);
        assert!(!path.join(CAPTURE_CAS_DIR).read_dir().unwrap().any(|_| true));
    }

    #[test]
    fn allowlisted_regular_file_is_captured_and_symlink_is_rejected_before_publication() {
        let state = tempfile::tempdir().unwrap();
        let sandbox = tempfile::tempdir().unwrap();
        fs::create_dir(sandbox.path().join("build")).unwrap();
        fs::write(sandbox.path().join("build/trace.json"), b"safe diagnostic").unwrap();
        let victim = state.path().join("host-secret");
        fs::write(&victim, b"SHOULD_NOT_LEAK").unwrap();
        symlink(&victim, sandbox.path().join("build/link")).unwrap();
        let policy = RemoteFailureDebugPolicy {
            replay_enabled: true,
            capture: RemoteFailureCapturePolicy {
                enabled: true,
                allowed_relative_paths: vec!["build/trace.json".to_string()],
                sensitivity: RemoteFailureCaptureSensitivity::RestrictedDiagnostic,
                ..RemoteFailureCapturePolicy::default()
            },
            ..RemoteFailureDebugPolicy::default()
        };
        let outcome = publish_remote_failure_debug_bundle(RemoteFailureDebugPublishRequest {
            state_dir: state.path(),
            capture_root: Some(sandbox.path()),
            facts: facts(),
            policy,
        })
        .unwrap();
        let path = bundle_dir(state.path(), &outcome);
        let rendered = fs::read(path.join(MANIFEST_FILE)).unwrap();

        assert_eq!(outcome.capture_outcome_code, "captured");
        assert!(!String::from_utf8_lossy(&rendered).contains("SHOULD_NOT_LEAK"));
        assert!(load_and_validate_remote_failure_debug_bundle(&path).is_ok());
        assert!(path.join(CAPTURE_CAS_DIR).read_dir().unwrap().any(|_| true));
    }

    #[test]
    fn tampered_manifest_object_and_capture_bytes_fail_closed() {
        let state = tempfile::tempdir().unwrap();
        let policy = RemoteFailureDebugPolicy {
            replay_enabled: true,
            ..RemoteFailureDebugPolicy::default()
        };
        let outcome = publish_remote_failure_debug_bundle(RemoteFailureDebugPublishRequest {
            state_dir: state.path(),
            capture_root: None,
            facts: facts(),
            policy,
        })
        .unwrap();
        let path = bundle_dir(state.path(), &outcome);
        let (bundle, _) = load_and_validate_remote_failure_debug_bundle(&path).unwrap();
        fs::write(object_path(&path, &bundle.action_ref), b"{}").unwrap();

        assert!(load_and_validate_remote_failure_debug_bundle(&path).is_err());
        assert!(plan_remote_failure_replay_from_disk(&path).is_err());
    }

    #[test]
    fn active_replay_lease_preserves_expired_bundle_then_drop_allows_deletion() {
        let state = tempfile::tempdir().unwrap();
        let policy = RemoteFailureDebugPolicy {
            replay_enabled: true,
            retention_secs: 1,
            ..RemoteFailureDebugPolicy::default()
        };
        let outcome = publish_remote_failure_debug_bundle(RemoteFailureDebugPublishRequest {
            state_dir: state.path(),
            capture_root: None,
            facts: facts(),
            policy,
        })
        .unwrap();
        let path = bundle_dir(state.path(), &outcome);
        let lease = acquire_remote_failure_debug_lease(&path, NOW_UNIX_S, LEASE_DURATION_SECS).unwrap();
        let preserved = retain_remote_failure_debug_bundles(state.path(), NOW_UNIX_S + 2).unwrap();
        drop(lease);
        let deleted = retain_remote_failure_debug_bundles(state.path(), NOW_UNIX_S + 2).unwrap();

        assert_eq!(preserved.preserved_count, 1);
        assert_eq!(preserved.deleted_count, 0);
        assert_eq!(deleted.deleted_count, 1);
        assert!(!path.exists());
    }

    #[test]
    fn missing_immutable_log_is_honest_and_replay_action_has_no_stale_authority() {
        let state = tempfile::tempdir().unwrap();
        let mut facts = facts();
        facts.immutable_log = None;
        let policy = RemoteFailureDebugPolicy {
            replay_enabled: true,
            ..RemoteFailureDebugPolicy::default()
        };
        let outcome = publish_remote_failure_debug_bundle(RemoteFailureDebugPublishRequest {
            state_dir: state.path(),
            capture_root: None,
            facts,
            policy,
        })
        .unwrap();
        let path = bundle_dir(state.path(), &outcome);
        let inspect = inspect_remote_failure_debug_bundle_from_disk(&path).unwrap();
        let replay_request = load_remote_failure_replay_request(&path).unwrap();

        assert!(!inspect.immutable_log_available);
        assert!(replay_request.production_attempt.is_none());
        assert!(replay_request.transfer_policy.is_none());
        assert_eq!(outcome.non_claim, REMOTE_FAILURE_DEBUG_NON_CLAIM);
    }
}
