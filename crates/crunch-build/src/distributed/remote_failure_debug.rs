//! Pure bounded remote-failure debug bundle, capture, replay, comparison, and retention decisions.
//!
//! Filesystem access, CAS writes, clocks, process execution, and rendering belong
//! in the root-package shell. This module only validates normalized facts and
//! returns deterministic plans.
//!
//! r[impl operator_diagnostics.remote_failure_debug_bundle]
//! r[impl operator_diagnostics.remote_failure_replay]
//! r[impl remote_builds.failure_debug_capture]

use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use super::RemoteAttemptId;
use super::RemoteAttemptLogDigest;
use super::RemoteAttemptLogScope;
use super::RemoteFenceGeneration;
use super::RemoteJobId;

pub const REMOTE_FAILURE_DEBUG_BUNDLE_SCHEMA: &str = "mantle-remote-failure-debug-bundle-v1";
pub const REMOTE_FAILURE_CAPTURE_MANIFEST_SCHEMA: &str = "mantle-remote-failure-captured-artifacts-v1";
pub const REMOTE_FAILURE_INSPECT_SCHEMA: &str = "mantle-remote-failure-debug-inspect-v1";
pub const REMOTE_FAILURE_REPLAY_PLAN_SCHEMA: &str = "mantle-remote-failure-replay-plan-v1";
pub const REMOTE_FAILURE_REPLAY_COMPARISON_SCHEMA: &str = "mantle-remote-failure-replay-comparison-v1";
pub const REMOTE_FAILURE_DEBUG_POLICY_NAME: &str = "mantle-remote-failure-debug-default-v1";
pub const REMOTE_FAILURE_DEBUG_NON_CLAIM: &str = "remote failure debug evidence is diagnostic only; it does not authorize execution, admit outputs, prove determinism, reproduce a sandbox byte-for-byte, or rewrite the original execution result";
pub const MAX_REMOTE_FAILURE_METADATA_BYTES: u64 = 1_048_576;
pub const MAX_REMOTE_FAILURE_OBJECT_BYTES: u64 = 67_108_864;
pub const MAX_REMOTE_FAILURE_CAPTURE_FILES: u32 = 32;
pub const MAX_REMOTE_FAILURE_CAPTURE_TOTAL_BYTES: u64 = 16_777_216;
pub const MAX_REMOTE_FAILURE_CAPTURE_FILE_BYTES: u64 = 8_388_608;
pub const MAX_REMOTE_FAILURE_CAPTURE_DEPTH: u32 = 16;
pub const MAX_REMOTE_FAILURE_CAPTURE_PATH_BYTES: usize = 1_024;
pub const MAX_REMOTE_FAILURE_REASON_BYTES: usize = 256;
pub const MAX_REMOTE_FAILURE_NON_CLAIMS: usize = 16;
pub const MAX_REMOTE_FAILURE_REFS: usize = 16;
pub const MAX_REMOTE_FAILURE_RETENTION_RECORDS: usize = 4_096;
pub const MAX_REMOTE_FAILURE_RETENTION_SECS: u64 = 2_592_000;
pub const DEFAULT_REMOTE_FAILURE_RETENTION_SECS: u64 = 604_800;
const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const BUNDLE_IDENTITY_DOMAIN: &str = "mantle-remote-failure-debug-bundle-identity-v1";
const POLICY_IDENTITY_DOMAIN: &str = "mantle-remote-failure-debug-policy-identity-v1";
const CAPTURE_MANIFEST_IDENTITY_DOMAIN: &str = "mantle-remote-failure-capture-manifest-identity-v1";
const CAPTURE_OBJECT_IDENTITY_DOMAIN: &str = "mantle-remote-failure-capture-object-identity-v1";
const INSPECT_IDENTITY_DOMAIN: &str = "mantle-remote-failure-inspect-identity-v1";
const REPLAY_PLAN_IDENTITY_DOMAIN: &str = "mantle-remote-failure-replay-plan-identity-v1";
const REPLAY_COMPARISON_IDENTITY_DOMAIN: &str = "mantle-remote-failure-replay-comparison-identity-v1";
const SECRET_REDACTION: &str = "<redacted>";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemoteFailureDebugDigest(String);

impl RemoteFailureDebugDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteFailureDebugReasonCode> {
        let value = value.into();
        if !is_blake3_hex_digest(&value) {
            return Err(RemoteFailureDebugReasonCode::DigestInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureDebugRef {
    pub kind: String,
    pub digest_blake3: RemoteFailureDebugDigest,
    pub byte_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureDebugPhase {
    Route,
    Assignment,
    InputTransfer,
    Execution,
    OutputTransfer,
    OutputAdmission,
    Cleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureWorkspaceMode {
    Ephemeral,
    Stateful,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureCaptureSensitivity {
    PublicDiagnostic,
    RestrictedDiagnostic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureCaptureFailureMode {
    DiagnosticOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureCapturePolicy {
    pub enabled: bool,
    pub allowed_relative_paths: Vec<String>,
    pub sensitivity: RemoteFailureCaptureSensitivity,
    pub file_count_max: u32,
    pub total_bytes_max: u64,
    pub file_bytes_max: u64,
    pub depth_max: u32,
    pub failure_mode: RemoteFailureCaptureFailureMode,
}

impl Default for RemoteFailureCapturePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_relative_paths: Vec::new(),
            sensitivity: RemoteFailureCaptureSensitivity::RestrictedDiagnostic,
            file_count_max: MAX_REMOTE_FAILURE_CAPTURE_FILES,
            total_bytes_max: MAX_REMOTE_FAILURE_CAPTURE_TOTAL_BYTES,
            file_bytes_max: MAX_REMOTE_FAILURE_CAPTURE_FILE_BYTES,
            depth_max: MAX_REMOTE_FAILURE_CAPTURE_DEPTH,
            failure_mode: RemoteFailureCaptureFailureMode::DiagnosticOnly,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureDebugPolicy {
    pub policy_name: String,
    pub metadata_bytes_max: u64,
    pub object_bytes_max: u64,
    pub retention_secs: u64,
    pub replay_enabled: bool,
    pub capture: RemoteFailureCapturePolicy,
}

impl Default for RemoteFailureDebugPolicy {
    fn default() -> Self {
        Self {
            policy_name: REMOTE_FAILURE_DEBUG_POLICY_NAME.to_string(),
            metadata_bytes_max: MAX_REMOTE_FAILURE_METADATA_BYTES,
            object_bytes_max: MAX_REMOTE_FAILURE_OBJECT_BYTES,
            retention_secs: DEFAULT_REMOTE_FAILURE_RETENTION_SECS,
            replay_enabled: false,
            capture: RemoteFailureCapturePolicy::default(),
        }
    }
}

impl RemoteFailureDebugPolicy {
    pub fn validate(&self) -> Result<(), RemoteFailureDebugReasonCode> {
        validate_token(&self.policy_name)?;
        if self.metadata_bytes_max == 0 || self.metadata_bytes_max > MAX_REMOTE_FAILURE_METADATA_BYTES {
            return Err(RemoteFailureDebugReasonCode::MetadataLimitInvalid);
        }
        if self.object_bytes_max == 0 || self.object_bytes_max > MAX_REMOTE_FAILURE_OBJECT_BYTES {
            return Err(RemoteFailureDebugReasonCode::ObjectLimitInvalid);
        }
        if self.retention_secs == 0 || self.retention_secs > MAX_REMOTE_FAILURE_RETENTION_SECS {
            return Err(RemoteFailureDebugReasonCode::RetentionInvalid);
        }
        validate_capture_policy(&self.capture)?;
        debug_assert!(self.metadata_bytes_max <= MAX_REMOTE_FAILURE_METADATA_BYTES);
        debug_assert!(self.object_bytes_max <= MAX_REMOTE_FAILURE_OBJECT_BYTES);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureImmutableLogRef {
    pub scope: RemoteAttemptLogScope,
    pub manifest_blake3: RemoteAttemptLogDigest,
    pub retained_start_cursor: u64,
    pub next_cursor: u64,
    pub head_record_blake3: Option<RemoteAttemptLogDigest>,
    pub head_segment_blake3: Option<RemoteAttemptLogDigest>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureDebugBundleFacts {
    pub action_ref: RemoteFailureDebugRef,
    pub route_ref: RemoteFailureDebugRef,
    pub worker_capability_ref: RemoteFailureDebugRef,
    pub input_manifest_ref: RemoteFailureDebugRef,
    pub sandbox_policy_ref: RemoteFailureDebugRef,
    pub network_policy_ref: RemoteFailureDebugRef,
    pub immutable_log: Option<RemoteFailureImmutableLogRef>,
    pub transfer_ref: Option<RemoteFailureDebugRef>,
    pub admission_ref: Option<RemoteFailureDebugRef>,
    pub captured_artifact_manifest_ref: Option<RemoteFailureDebugRef>,
    pub original_job_id: RemoteJobId,
    pub original_attempt_id: RemoteAttemptId,
    pub original_fence_generation: RemoteFenceGeneration,
    pub workspace_mode: RemoteFailureWorkspaceMode,
    pub failure_phase: RemoteFailureDebugPhase,
    pub failure_reason_code: String,
    pub capture_outcome_code: String,
    pub cleanup_status_code: String,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureDebugBundle {
    pub schema: String,
    pub bundle_blake3: RemoteFailureDebugDigest,
    pub policy_blake3: RemoteFailureDebugDigest,
    pub action_ref: RemoteFailureDebugRef,
    pub route_ref: RemoteFailureDebugRef,
    pub worker_capability_ref: RemoteFailureDebugRef,
    pub input_manifest_ref: RemoteFailureDebugRef,
    pub sandbox_policy_ref: RemoteFailureDebugRef,
    pub network_policy_ref: RemoteFailureDebugRef,
    pub immutable_log: Option<RemoteFailureImmutableLogRef>,
    pub transfer_ref: Option<RemoteFailureDebugRef>,
    pub admission_ref: Option<RemoteFailureDebugRef>,
    pub captured_artifact_manifest_ref: Option<RemoteFailureDebugRef>,
    pub original_job_id: RemoteJobId,
    pub original_attempt_id: RemoteAttemptId,
    pub original_fence_generation: RemoteFenceGeneration,
    pub workspace_mode: RemoteFailureWorkspaceMode,
    pub failure_phase: RemoteFailureDebugPhase,
    pub failure_reason_code: String,
    pub capture_outcome_code: String,
    pub cleanup_status_code: String,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureCaptureRequest {
    pub relative_path: String,
    pub sensitivity: RemoteFailureCaptureSensitivity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureCaptureObservedKind {
    RegularFile,
    Directory,
    Symlink,
    Socket,
    Fifo,
    Device,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureCaptureObservation {
    pub relative_path: String,
    pub kind: RemoteFailureCaptureObservedKind,
    pub byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureCaptureAdmissionPlan {
    pub accepted_paths: Vec<String>,
    pub rejected: Vec<RemoteFailureCaptureRejection>,
    pub accepted_total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureCaptureRejection {
    pub relative_path: String,
    pub reason_code: RemoteFailureDebugReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureCapturedArtifact {
    pub relative_path: String,
    pub sensitivity: RemoteFailureCaptureSensitivity,
    pub byte_count: u64,
    pub object_blake3: RemoteFailureDebugDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureCapturedArtifactManifest {
    pub schema: String,
    pub manifest_blake3: RemoteFailureDebugDigest,
    pub artifacts: Vec<RemoteFailureCapturedArtifact>,
    pub total_bytes: u64,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureInspectSummary {
    pub schema: String,
    pub inspect_blake3: RemoteFailureDebugDigest,
    pub bundle_blake3: RemoteFailureDebugDigest,
    pub failure_phase: RemoteFailureDebugPhase,
    pub failure_reason_code: String,
    pub workspace_mode: RemoteFailureWorkspaceMode,
    pub immutable_log_available: bool,
    pub captured_artifact_count: u32,
    pub replay_allowed_by_policy: bool,
    pub expires_unix_s: u64,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureReplayPlan {
    pub schema: String,
    pub replay_plan_blake3: RemoteFailureDebugDigest,
    pub source_bundle_blake3: RemoteFailureDebugDigest,
    pub action_ref: RemoteFailureDebugRef,
    pub input_manifest_ref: RemoteFailureDebugRef,
    pub sandbox_policy_ref: RemoteFailureDebugRef,
    pub network_policy_ref: RemoteFailureDebugRef,
    pub blockers: Vec<String>,
    pub new_authority_required: bool,
    pub ordinary_route_required: bool,
    pub ordinary_output_admission_required: bool,
    pub executable: bool,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureReplayAvailability {
    pub available_refs: BTreeSet<RemoteFailureDebugDigest>,
    pub current_policy_allows_replay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureExecutionSummary {
    pub failure_phase: Option<RemoteFailureDebugPhase>,
    pub failure_reason_code: Option<String>,
    pub exit_status_class: String,
    pub admitted_output_digests: Vec<RemoteFailureDebugDigest>,
    pub log_head_blake3: Option<RemoteFailureDebugDigest>,
    pub captured_manifest_blake3: Option<RemoteFailureDebugDigest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureReplayComparisonClass {
    Match,
    Diverged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureReplayComparison {
    pub schema: String,
    pub comparison_blake3: RemoteFailureDebugDigest,
    pub class: RemoteFailureReplayComparisonClass,
    pub matching_fact_classes: Vec<String>,
    pub divergent_fact_classes: Vec<String>,
    pub original_result_immutable: bool,
    pub replay_output_requires_ordinary_admission: bool,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureRetentionRecord {
    pub bundle_blake3: RemoteFailureDebugDigest,
    pub expires_unix_s: u64,
    pub active_lease: bool,
    pub ordinary_build_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFailureRetentionPlan {
    pub preserve: Vec<RemoteFailureDebugDigest>,
    pub delete: Vec<RemoteFailureDebugDigest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureDebugReasonCode {
    SchemaUnsupported,
    DigestInvalid,
    RefKindInvalid,
    RefTooLarge,
    MetadataLimitInvalid,
    ObjectLimitInvalid,
    RetentionInvalid,
    CaptureLimitInvalid,
    CaptureDisabled,
    CapturePathInvalid,
    CapturePathDuplicate,
    CapturePathNotAllowed,
    CaptureFileTooLarge,
    CaptureCountExceeded,
    CaptureTotalBytesExceeded,
    CaptureArithmeticOverflow,
    CaptureUnsupportedKind,
    AttemptBindingMismatch,
    ImmutableLogRangeInvalid,
    FailureReasonInvalid,
    CleanupStatusInvalid,
    NonClaimInvalid,
    TimestampInvalid,
    SerializedMetadataTooLarge,
    ManifestIdentityMismatch,
    RequiredRefMissing,
    ReplayPolicyDenied,
    RetentionRecordLimitExceeded,
    OrdinaryOutputDeletionForbidden,
}

impl RemoteFailureDebugReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SchemaUnsupported => "schema-unsupported",
            Self::DigestInvalid => "digest-invalid",
            Self::RefKindInvalid => "ref-kind-invalid",
            Self::RefTooLarge => "ref-too-large",
            Self::MetadataLimitInvalid => "metadata-limit-invalid",
            Self::ObjectLimitInvalid => "object-limit-invalid",
            Self::RetentionInvalid => "retention-invalid",
            Self::CaptureLimitInvalid => "capture-limit-invalid",
            Self::CaptureDisabled => "capture-disabled",
            Self::CapturePathInvalid => "capture-path-invalid",
            Self::CapturePathDuplicate => "capture-path-duplicate",
            Self::CapturePathNotAllowed => "capture-path-not-allowed",
            Self::CaptureFileTooLarge => "capture-file-too-large",
            Self::CaptureCountExceeded => "capture-count-exceeded",
            Self::CaptureTotalBytesExceeded => "capture-total-bytes-exceeded",
            Self::CaptureArithmeticOverflow => "capture-arithmetic-overflow",
            Self::CaptureUnsupportedKind => "capture-unsupported-kind",
            Self::AttemptBindingMismatch => "attempt-binding-mismatch",
            Self::ImmutableLogRangeInvalid => "immutable-log-range-invalid",
            Self::FailureReasonInvalid => "failure-reason-invalid",
            Self::CleanupStatusInvalid => "cleanup-status-invalid",
            Self::NonClaimInvalid => "non-claim-invalid",
            Self::TimestampInvalid => "timestamp-invalid",
            Self::SerializedMetadataTooLarge => "serialized-metadata-too-large",
            Self::ManifestIdentityMismatch => "manifest-identity-mismatch",
            Self::RequiredRefMissing => "required-ref-missing",
            Self::ReplayPolicyDenied => "replay-policy-denied",
            Self::RetentionRecordLimitExceeded => "retention-record-limit-exceeded",
            Self::OrdinaryOutputDeletionForbidden => "ordinary-output-deletion-forbidden",
        }
    }
}

pub fn remote_failure_debug_ref(
    kind: impl Into<String>,
    bytes: &[u8],
    policy: &RemoteFailureDebugPolicy,
) -> Result<RemoteFailureDebugRef, RemoteFailureDebugReasonCode> {
    policy.validate()?;
    let kind = kind.into();
    validate_token(&kind).map_err(|_| RemoteFailureDebugReasonCode::RefKindInvalid)?;
    let byte_count = u64::try_from(bytes.len()).map_err(|_| RemoteFailureDebugReasonCode::RefTooLarge)?;
    if byte_count > policy.object_bytes_max {
        return Err(RemoteFailureDebugReasonCode::RefTooLarge);
    }
    let digest_blake3 = domain_digest(&format!("mantle-remote-failure-ref-{kind}-v1"), bytes);
    debug_assert_eq!(digest_blake3.as_str().len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert!(byte_count <= policy.object_bytes_max);
    Ok(RemoteFailureDebugRef {
        kind,
        digest_blake3,
        byte_count,
    })
}

pub fn remote_failure_capture_object_digest(bytes: &[u8]) -> RemoteFailureDebugDigest {
    domain_digest(CAPTURE_OBJECT_IDENTITY_DOMAIN, bytes)
}

pub fn seal_remote_failure_debug_bundle(
    facts: RemoteFailureDebugBundleFacts,
    policy: &RemoteFailureDebugPolicy,
) -> Result<RemoteFailureDebugBundle, RemoteFailureDebugReasonCode> {
    policy.validate()?;
    validate_bundle_facts(&facts, policy)?;
    let policy_blake3 = digest_serialized(POLICY_IDENTITY_DOMAIN, policy)?;
    let non_claims = canonical_non_claims(facts.non_claims)?;
    let mut bundle = RemoteFailureDebugBundle {
        schema: REMOTE_FAILURE_DEBUG_BUNDLE_SCHEMA.to_string(),
        bundle_blake3: zero_digest(),
        policy_blake3,
        action_ref: facts.action_ref,
        route_ref: facts.route_ref,
        worker_capability_ref: facts.worker_capability_ref,
        input_manifest_ref: facts.input_manifest_ref,
        sandbox_policy_ref: facts.sandbox_policy_ref,
        network_policy_ref: facts.network_policy_ref,
        immutable_log: facts.immutable_log,
        transfer_ref: facts.transfer_ref,
        admission_ref: facts.admission_ref,
        captured_artifact_manifest_ref: facts.captured_artifact_manifest_ref,
        original_job_id: facts.original_job_id,
        original_attempt_id: facts.original_attempt_id,
        original_fence_generation: facts.original_fence_generation,
        workspace_mode: facts.workspace_mode,
        failure_phase: facts.failure_phase,
        failure_reason_code: facts.failure_reason_code,
        capture_outcome_code: facts.capture_outcome_code,
        cleanup_status_code: facts.cleanup_status_code,
        created_unix_s: facts.created_unix_s,
        expires_unix_s: facts.expires_unix_s,
        non_claims,
    };
    bundle.bundle_blake3 = bundle_identity(&bundle)?;
    validate_remote_failure_debug_bundle(&bundle, policy)?;
    Ok(bundle)
}

pub fn validate_remote_failure_debug_bundle(
    bundle: &RemoteFailureDebugBundle,
    policy: &RemoteFailureDebugPolicy,
) -> Result<(), RemoteFailureDebugReasonCode> {
    policy.validate()?;
    if bundle.schema != REMOTE_FAILURE_DEBUG_BUNDLE_SCHEMA {
        return Err(RemoteFailureDebugReasonCode::SchemaUnsupported);
    }
    validate_ref(&bundle.action_ref, policy)?;
    validate_ref(&bundle.route_ref, policy)?;
    validate_ref(&bundle.worker_capability_ref, policy)?;
    validate_ref(&bundle.input_manifest_ref, policy)?;
    validate_ref(&bundle.sandbox_policy_ref, policy)?;
    validate_ref(&bundle.network_policy_ref, policy)?;
    validate_optional_ref(bundle.transfer_ref.as_ref(), policy)?;
    validate_optional_ref(bundle.admission_ref.as_ref(), policy)?;
    validate_optional_ref(bundle.captured_artifact_manifest_ref.as_ref(), policy)?;
    validate_attempt_and_log_binding(bundle)?;
    validate_failure_codes(bundle)?;
    validate_timestamps(bundle.created_unix_s, bundle.expires_unix_s, policy.retention_secs)?;
    if canonical_non_claims(bundle.non_claims.clone())? != bundle.non_claims {
        return Err(RemoteFailureDebugReasonCode::NonClaimInvalid);
    }
    let expected_policy = digest_serialized(POLICY_IDENTITY_DOMAIN, policy)?;
    if expected_policy != bundle.policy_blake3 {
        return Err(RemoteFailureDebugReasonCode::ManifestIdentityMismatch);
    }
    let expected_bundle = bundle_identity(bundle)?;
    if expected_bundle != bundle.bundle_blake3 {
        return Err(RemoteFailureDebugReasonCode::ManifestIdentityMismatch);
    }
    let bytes = serde_json::to_vec(bundle).map_err(|_| RemoteFailureDebugReasonCode::SerializedMetadataTooLarge)?;
    let byte_count =
        u64::try_from(bytes.len()).map_err(|_| RemoteFailureDebugReasonCode::SerializedMetadataTooLarge)?;
    if byte_count > policy.metadata_bytes_max {
        return Err(RemoteFailureDebugReasonCode::SerializedMetadataTooLarge);
    }
    debug_assert_eq!(expected_bundle, bundle.bundle_blake3);
    debug_assert!(byte_count <= policy.metadata_bytes_max);
    Ok(())
}

pub fn plan_remote_failure_capture(
    policy: &RemoteFailureCapturePolicy,
) -> Result<Vec<RemoteFailureCaptureRequest>, RemoteFailureDebugReasonCode> {
    validate_capture_policy(policy)?;
    if !policy.enabled {
        if policy.allowed_relative_paths.is_empty() {
            return Ok(Vec::new());
        }
        return Err(RemoteFailureDebugReasonCode::CaptureDisabled);
    }
    let mut paths = policy.allowed_relative_paths.clone();
    paths.sort();
    let mut seen = BTreeSet::new();
    let mut requests = Vec::with_capacity(paths.len());
    for path in paths {
        validate_capture_relative_path(&path, policy.depth_max)?;
        if !seen.insert(path.clone()) {
            return Err(RemoteFailureDebugReasonCode::CapturePathDuplicate);
        }
        requests.push(RemoteFailureCaptureRequest {
            relative_path: path,
            sensitivity: policy.sensitivity,
        });
    }
    debug_assert!(requests.len() <= usize::try_from(policy.file_count_max).unwrap_or(usize::MAX));
    debug_assert!(requests.windows(2).all(|window| window[0].relative_path < window[1].relative_path));
    Ok(requests)
}

pub fn admit_remote_failure_capture_observations(
    requests: &[RemoteFailureCaptureRequest],
    observations: Vec<RemoteFailureCaptureObservation>,
    policy: &RemoteFailureCapturePolicy,
) -> Result<RemoteFailureCaptureAdmissionPlan, RemoteFailureDebugReasonCode> {
    validate_capture_policy(policy)?;
    if observations.len() > usize::try_from(policy.file_count_max).unwrap_or(usize::MAX) {
        return Err(RemoteFailureDebugReasonCode::CaptureCountExceeded);
    }
    let allowed = requests.iter().map(|request| request.relative_path.as_str()).collect::<BTreeSet<_>>();
    let mut ordered = observations;
    ordered.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let mut accepted_paths = Vec::new();
    let mut rejected = Vec::new();
    let mut total_bytes = 0_u64;
    let mut seen = BTreeSet::new();
    for observation in ordered {
        let reason = capture_observation_rejection(&observation, &allowed, &mut seen, policy, total_bytes)?;
        if let Some(reason_code) = reason {
            rejected.push(RemoteFailureCaptureRejection {
                relative_path: observation.relative_path,
                reason_code,
            });
            continue;
        }
        total_bytes = total_bytes
            .checked_add(observation.byte_count)
            .ok_or(RemoteFailureDebugReasonCode::CaptureArithmeticOverflow)?;
        accepted_paths.push(observation.relative_path);
    }
    debug_assert!(total_bytes <= policy.total_bytes_max);
    debug_assert!(accepted_paths.len() <= usize::try_from(policy.file_count_max).unwrap_or(usize::MAX));
    Ok(RemoteFailureCaptureAdmissionPlan {
        accepted_paths,
        rejected,
        accepted_total_bytes: total_bytes,
    })
}

pub fn seal_remote_failure_captured_artifact_manifest(
    mut artifacts: Vec<RemoteFailureCapturedArtifact>,
    policy: &RemoteFailureCapturePolicy,
) -> Result<RemoteFailureCapturedArtifactManifest, RemoteFailureDebugReasonCode> {
    validate_capture_policy(policy)?;
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let mut total_bytes = 0_u64;
    let mut seen = BTreeSet::new();
    for artifact in &artifacts {
        validate_capture_relative_path(&artifact.relative_path, policy.depth_max)?;
        if !seen.insert(&artifact.relative_path) {
            return Err(RemoteFailureDebugReasonCode::CapturePathDuplicate);
        }
        if artifact.byte_count > policy.file_bytes_max {
            return Err(RemoteFailureDebugReasonCode::CaptureFileTooLarge);
        }
        total_bytes = total_bytes
            .checked_add(artifact.byte_count)
            .ok_or(RemoteFailureDebugReasonCode::CaptureArithmeticOverflow)?;
        if total_bytes > policy.total_bytes_max {
            return Err(RemoteFailureDebugReasonCode::CaptureTotalBytesExceeded);
        }
    }
    if artifacts.len() > usize::try_from(policy.file_count_max).unwrap_or(usize::MAX) {
        return Err(RemoteFailureDebugReasonCode::CaptureCountExceeded);
    }
    let mut manifest = RemoteFailureCapturedArtifactManifest {
        schema: REMOTE_FAILURE_CAPTURE_MANIFEST_SCHEMA.to_string(),
        manifest_blake3: zero_digest(),
        artifacts,
        total_bytes,
        non_claim: REMOTE_FAILURE_DEBUG_NON_CLAIM.to_string(),
    };
    manifest.manifest_blake3 = capture_manifest_identity(&manifest)?;
    debug_assert!(manifest.total_bytes <= policy.total_bytes_max);
    debug_assert!(manifest.artifacts.len() <= usize::try_from(policy.file_count_max).unwrap_or(usize::MAX));
    Ok(manifest)
}

pub fn inspect_remote_failure_debug_bundle(
    bundle: &RemoteFailureDebugBundle,
    policy: &RemoteFailureDebugPolicy,
) -> Result<RemoteFailureInspectSummary, RemoteFailureDebugReasonCode> {
    validate_remote_failure_debug_bundle(bundle, policy)?;
    let captured_artifact_count = u32::from(bundle.captured_artifact_manifest_ref.is_some());
    let mut summary = RemoteFailureInspectSummary {
        schema: REMOTE_FAILURE_INSPECT_SCHEMA.to_string(),
        inspect_blake3: zero_digest(),
        bundle_blake3: bundle.bundle_blake3.clone(),
        failure_phase: bundle.failure_phase,
        failure_reason_code: bundle.failure_reason_code.clone(),
        workspace_mode: bundle.workspace_mode,
        immutable_log_available: bundle.immutable_log.is_some(),
        captured_artifact_count,
        replay_allowed_by_policy: policy.replay_enabled,
        expires_unix_s: bundle.expires_unix_s,
        non_claims: bundle.non_claims.clone(),
    };
    summary.inspect_blake3 = digest_serialized(INSPECT_IDENTITY_DOMAIN, &summary_without_digest(&summary))?;
    debug_assert_eq!(summary.bundle_blake3, bundle.bundle_blake3);
    debug_assert!(summary.captured_artifact_count <= 1);
    Ok(summary)
}

pub fn plan_remote_failure_replay(
    bundle: &RemoteFailureDebugBundle,
    policy: &RemoteFailureDebugPolicy,
    availability: &RemoteFailureReplayAvailability,
) -> Result<RemoteFailureReplayPlan, RemoteFailureDebugReasonCode> {
    validate_remote_failure_debug_bundle(bundle, policy)?;
    let required = [
        &bundle.action_ref,
        &bundle.input_manifest_ref,
        &bundle.sandbox_policy_ref,
        &bundle.network_policy_ref,
    ];
    let mut blockers = Vec::new();
    if !policy.replay_enabled || !availability.current_policy_allows_replay {
        blockers.push(RemoteFailureDebugReasonCode::ReplayPolicyDenied.as_str().to_string());
    }
    for reference in required {
        if !availability.available_refs.contains(&reference.digest_blake3) {
            blockers.push(format!("{}:{}", RemoteFailureDebugReasonCode::RequiredRefMissing.as_str(), reference.kind));
        }
    }
    blockers.sort();
    blockers.dedup();
    let mut plan = RemoteFailureReplayPlan {
        schema: REMOTE_FAILURE_REPLAY_PLAN_SCHEMA.to_string(),
        replay_plan_blake3: zero_digest(),
        source_bundle_blake3: bundle.bundle_blake3.clone(),
        action_ref: bundle.action_ref.clone(),
        input_manifest_ref: bundle.input_manifest_ref.clone(),
        sandbox_policy_ref: bundle.sandbox_policy_ref.clone(),
        network_policy_ref: bundle.network_policy_ref.clone(),
        executable: blockers.is_empty(),
        blockers,
        new_authority_required: true,
        ordinary_route_required: true,
        ordinary_output_admission_required: true,
        non_claims: bundle.non_claims.clone(),
    };
    plan.replay_plan_blake3 = digest_serialized(REPLAY_PLAN_IDENTITY_DOMAIN, &replay_plan_without_digest(&plan))?;
    debug_assert!(plan.new_authority_required);
    debug_assert!(plan.ordinary_output_admission_required);
    Ok(plan)
}

pub fn compare_remote_failure_replay(
    original: &RemoteFailureExecutionSummary,
    replay: &RemoteFailureExecutionSummary,
) -> Result<RemoteFailureReplayComparison, RemoteFailureDebugReasonCode> {
    let mut matching = Vec::new();
    let mut divergent = Vec::new();
    compare_fact("failure-phase", &original.failure_phase, &replay.failure_phase, &mut matching, &mut divergent);
    compare_fact(
        "failure-reason-code",
        &original.failure_reason_code,
        &replay.failure_reason_code,
        &mut matching,
        &mut divergent,
    );
    compare_fact(
        "exit-status-class",
        &original.exit_status_class,
        &replay.exit_status_class,
        &mut matching,
        &mut divergent,
    );
    compare_fact(
        "admitted-output-digests",
        &original.admitted_output_digests,
        &replay.admitted_output_digests,
        &mut matching,
        &mut divergent,
    );
    compare_fact("log-head", &original.log_head_blake3, &replay.log_head_blake3, &mut matching, &mut divergent);
    compare_fact(
        "captured-artifact-manifest",
        &original.captured_manifest_blake3,
        &replay.captured_manifest_blake3,
        &mut matching,
        &mut divergent,
    );
    let mut comparison = RemoteFailureReplayComparison {
        schema: REMOTE_FAILURE_REPLAY_COMPARISON_SCHEMA.to_string(),
        comparison_blake3: zero_digest(),
        class: if divergent.is_empty() {
            RemoteFailureReplayComparisonClass::Match
        } else {
            RemoteFailureReplayComparisonClass::Diverged
        },
        matching_fact_classes: matching,
        divergent_fact_classes: divergent,
        original_result_immutable: true,
        replay_output_requires_ordinary_admission: true,
        non_claim: REMOTE_FAILURE_DEBUG_NON_CLAIM.to_string(),
    };
    comparison.comparison_blake3 =
        digest_serialized(REPLAY_COMPARISON_IDENTITY_DOMAIN, &replay_comparison_without_digest(&comparison))?;
    debug_assert!(comparison.original_result_immutable);
    debug_assert!(comparison.replay_output_requires_ordinary_admission);
    Ok(comparison)
}

pub fn plan_remote_failure_retention(
    records: Vec<RemoteFailureRetentionRecord>,
    now_unix_s: u64,
) -> Result<RemoteFailureRetentionPlan, RemoteFailureDebugReasonCode> {
    if records.len() > MAX_REMOTE_FAILURE_RETENTION_RECORDS {
        return Err(RemoteFailureDebugReasonCode::RetentionRecordLimitExceeded);
    }
    let mut ordered = records;
    ordered.sort_by(|left, right| left.bundle_blake3.cmp(&right.bundle_blake3));
    let mut seen = BTreeSet::new();
    let mut preserve = Vec::new();
    let mut delete = Vec::new();
    for record in ordered {
        if !seen.insert(record.bundle_blake3.clone()) {
            return Err(RemoteFailureDebugReasonCode::ManifestIdentityMismatch);
        }
        if record.ordinary_build_output {
            return Err(RemoteFailureDebugReasonCode::OrdinaryOutputDeletionForbidden);
        }
        if record.active_lease || record.expires_unix_s > now_unix_s {
            preserve.push(record.bundle_blake3);
        } else {
            delete.push(record.bundle_blake3);
        }
    }
    debug_assert_eq!(preserve.len().saturating_add(delete.len()), seen.len());
    debug_assert!(preserve.iter().all(|digest| !delete.contains(digest)));
    Ok(RemoteFailureRetentionPlan { preserve, delete })
}

pub fn redact_remote_failure_diagnostic(value: &str) -> Result<String, RemoteFailureDebugReasonCode> {
    if value.len() > MAX_REMOTE_FAILURE_REASON_BYTES {
        return Err(RemoteFailureDebugReasonCode::FailureReasonInvalid);
    }
    if text_looks_secret_bearing(value) || text_looks_host_path(value) {
        return Ok(SECRET_REDACTION.to_string());
    }
    let rendered: String =
        value.chars().map(|character| if character.is_control() { '�' } else { character }).collect();
    debug_assert!(!rendered.contains('\0'));
    debug_assert!(rendered.len() <= MAX_REMOTE_FAILURE_REASON_BYTES.saturating_mul(char::MAX_LEN_UTF8));
    Ok(rendered)
}

fn validate_bundle_facts(
    facts: &RemoteFailureDebugBundleFacts,
    policy: &RemoteFailureDebugPolicy,
) -> Result<(), RemoteFailureDebugReasonCode> {
    for reference in [
        &facts.action_ref,
        &facts.route_ref,
        &facts.worker_capability_ref,
        &facts.input_manifest_ref,
        &facts.sandbox_policy_ref,
        &facts.network_policy_ref,
    ] {
        validate_ref(reference, policy)?;
    }
    validate_optional_ref(facts.transfer_ref.as_ref(), policy)?;
    validate_optional_ref(facts.admission_ref.as_ref(), policy)?;
    validate_optional_ref(facts.captured_artifact_manifest_ref.as_ref(), policy)?;
    validate_token(&facts.failure_reason_code).map_err(|_| RemoteFailureDebugReasonCode::FailureReasonInvalid)?;
    validate_token(&facts.capture_outcome_code).map_err(|_| RemoteFailureDebugReasonCode::FailureReasonInvalid)?;
    validate_token(&facts.cleanup_status_code).map_err(|_| RemoteFailureDebugReasonCode::CleanupStatusInvalid)?;
    validate_timestamps(facts.created_unix_s, facts.expires_unix_s, policy.retention_secs)?;
    if let Some(log) = &facts.immutable_log {
        validate_immutable_log(
            log,
            &facts.original_job_id,
            &facts.original_attempt_id,
            facts.original_fence_generation,
        )?;
    }
    Ok(())
}

fn validate_ref(
    reference: &RemoteFailureDebugRef,
    policy: &RemoteFailureDebugPolicy,
) -> Result<(), RemoteFailureDebugReasonCode> {
    validate_token(&reference.kind).map_err(|_| RemoteFailureDebugReasonCode::RefKindInvalid)?;
    if reference.byte_count > policy.object_bytes_max {
        return Err(RemoteFailureDebugReasonCode::RefTooLarge);
    }
    if !is_blake3_hex_digest(reference.digest_blake3.as_str()) {
        return Err(RemoteFailureDebugReasonCode::DigestInvalid);
    }
    Ok(())
}

fn validate_optional_ref(
    reference: Option<&RemoteFailureDebugRef>,
    policy: &RemoteFailureDebugPolicy,
) -> Result<(), RemoteFailureDebugReasonCode> {
    if let Some(reference) = reference {
        validate_ref(reference, policy)?;
    }
    Ok(())
}

fn validate_attempt_and_log_binding(bundle: &RemoteFailureDebugBundle) -> Result<(), RemoteFailureDebugReasonCode> {
    if let Some(log) = &bundle.immutable_log {
        validate_immutable_log(
            log,
            &bundle.original_job_id,
            &bundle.original_attempt_id,
            bundle.original_fence_generation,
        )?;
    }
    Ok(())
}

fn validate_immutable_log(
    log: &RemoteFailureImmutableLogRef,
    job_id: &RemoteJobId,
    attempt_id: &RemoteAttemptId,
    fence_generation: RemoteFenceGeneration,
) -> Result<(), RemoteFailureDebugReasonCode> {
    if &log.scope.job_id != job_id
        || &log.scope.attempt_id != attempt_id
        || log.scope.fence_generation != fence_generation
    {
        return Err(RemoteFailureDebugReasonCode::AttemptBindingMismatch);
    }
    if log.retained_start_cursor > log.next_cursor {
        return Err(RemoteFailureDebugReasonCode::ImmutableLogRangeInvalid);
    }
    Ok(())
}

fn validate_failure_codes(bundle: &RemoteFailureDebugBundle) -> Result<(), RemoteFailureDebugReasonCode> {
    validate_token(&bundle.failure_reason_code).map_err(|_| RemoteFailureDebugReasonCode::FailureReasonInvalid)?;
    validate_token(&bundle.capture_outcome_code).map_err(|_| RemoteFailureDebugReasonCode::FailureReasonInvalid)?;
    validate_token(&bundle.cleanup_status_code).map_err(|_| RemoteFailureDebugReasonCode::CleanupStatusInvalid)?;
    Ok(())
}

fn validate_timestamps(
    created_unix_s: u64,
    expires_unix_s: u64,
    retention_secs: u64,
) -> Result<(), RemoteFailureDebugReasonCode> {
    let expected = created_unix_s.checked_add(retention_secs).ok_or(RemoteFailureDebugReasonCode::TimestampInvalid)?;
    if expires_unix_s != expected {
        return Err(RemoteFailureDebugReasonCode::TimestampInvalid);
    }
    Ok(())
}

fn validate_capture_policy(policy: &RemoteFailureCapturePolicy) -> Result<(), RemoteFailureDebugReasonCode> {
    if policy.file_count_max == 0 || policy.file_count_max > MAX_REMOTE_FAILURE_CAPTURE_FILES {
        return Err(RemoteFailureDebugReasonCode::CaptureLimitInvalid);
    }
    if policy.total_bytes_max == 0 || policy.total_bytes_max > MAX_REMOTE_FAILURE_CAPTURE_TOTAL_BYTES {
        return Err(RemoteFailureDebugReasonCode::CaptureLimitInvalid);
    }
    if policy.file_bytes_max == 0 || policy.file_bytes_max > MAX_REMOTE_FAILURE_CAPTURE_FILE_BYTES {
        return Err(RemoteFailureDebugReasonCode::CaptureLimitInvalid);
    }
    if policy.file_bytes_max > policy.total_bytes_max {
        return Err(RemoteFailureDebugReasonCode::CaptureLimitInvalid);
    }
    if policy.depth_max == 0 || policy.depth_max > MAX_REMOTE_FAILURE_CAPTURE_DEPTH {
        return Err(RemoteFailureDebugReasonCode::CaptureLimitInvalid);
    }
    if policy.allowed_relative_paths.len() > usize::try_from(policy.file_count_max).unwrap_or(usize::MAX) {
        return Err(RemoteFailureDebugReasonCode::CaptureCountExceeded);
    }
    if !policy.enabled && !policy.allowed_relative_paths.is_empty() {
        return Err(RemoteFailureDebugReasonCode::CaptureDisabled);
    }
    Ok(())
}

fn validate_capture_relative_path(path: &str, depth_max: u32) -> Result<(), RemoteFailureDebugReasonCode> {
    if path.is_empty() || path.len() > MAX_REMOTE_FAILURE_CAPTURE_PATH_BYTES || path.chars().any(char::is_control) {
        return Err(RemoteFailureDebugReasonCode::CapturePathInvalid);
    }
    let parsed = Path::new(path);
    if parsed.is_absolute() || text_looks_host_path(path) || text_looks_secret_bearing(path) {
        return Err(RemoteFailureDebugReasonCode::CapturePathInvalid);
    }
    let mut depth = 0_u32;
    for component in parsed.components() {
        match component {
            Component::Normal(_) => {
                depth = depth.checked_add(1).ok_or(RemoteFailureDebugReasonCode::CaptureArithmeticOverflow)?;
            }
            Component::CurDir | Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(RemoteFailureDebugReasonCode::CapturePathInvalid);
            }
        }
    }
    if depth == 0 || depth > depth_max {
        return Err(RemoteFailureDebugReasonCode::CapturePathInvalid);
    }
    Ok(())
}

fn capture_observation_rejection(
    observation: &RemoteFailureCaptureObservation,
    allowed: &BTreeSet<&str>,
    seen: &mut BTreeSet<String>,
    policy: &RemoteFailureCapturePolicy,
    accepted_bytes: u64,
) -> Result<Option<RemoteFailureDebugReasonCode>, RemoteFailureDebugReasonCode> {
    validate_capture_relative_path(&observation.relative_path, policy.depth_max)?;
    if !seen.insert(observation.relative_path.clone()) {
        return Ok(Some(RemoteFailureDebugReasonCode::CapturePathDuplicate));
    }
    if !allowed.contains(observation.relative_path.as_str()) {
        return Ok(Some(RemoteFailureDebugReasonCode::CapturePathNotAllowed));
    }
    if observation.kind != RemoteFailureCaptureObservedKind::RegularFile {
        return Ok(Some(RemoteFailureDebugReasonCode::CaptureUnsupportedKind));
    }
    if observation.byte_count > policy.file_bytes_max {
        return Ok(Some(RemoteFailureDebugReasonCode::CaptureFileTooLarge));
    }
    let next_total = accepted_bytes
        .checked_add(observation.byte_count)
        .ok_or(RemoteFailureDebugReasonCode::CaptureArithmeticOverflow)?;
    if next_total > policy.total_bytes_max {
        return Ok(Some(RemoteFailureDebugReasonCode::CaptureTotalBytesExceeded));
    }
    Ok(None)
}

fn canonical_non_claims(mut non_claims: Vec<String>) -> Result<Vec<String>, RemoteFailureDebugReasonCode> {
    non_claims.push(REMOTE_FAILURE_DEBUG_NON_CLAIM.to_string());
    non_claims.sort();
    non_claims.dedup();
    if non_claims.len() > MAX_REMOTE_FAILURE_NON_CLAIMS {
        return Err(RemoteFailureDebugReasonCode::NonClaimInvalid);
    }
    for non_claim in &non_claims {
        if non_claim.is_empty()
            || non_claim.len() > MAX_REMOTE_FAILURE_REASON_BYTES
            || non_claim.chars().any(char::is_control)
            || text_looks_secret_bearing(non_claim)
            || text_looks_host_path(non_claim)
        {
            return Err(RemoteFailureDebugReasonCode::NonClaimInvalid);
        }
    }
    Ok(non_claims)
}

fn validate_token(value: &str) -> Result<(), RemoteFailureDebugReasonCode> {
    if value.is_empty() || value.len() > MAX_REMOTE_FAILURE_REASON_BYTES {
        return Err(RemoteFailureDebugReasonCode::FailureReasonInvalid);
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.' | b'_' | b':'))
    {
        return Err(RemoteFailureDebugReasonCode::FailureReasonInvalid);
    }
    Ok(())
}

fn bundle_identity(
    bundle: &RemoteFailureDebugBundle,
) -> Result<RemoteFailureDebugDigest, RemoteFailureDebugReasonCode> {
    let mut identity = bundle.clone();
    identity.bundle_blake3 = zero_digest();
    digest_serialized(BUNDLE_IDENTITY_DOMAIN, &identity)
}

fn capture_manifest_identity(
    manifest: &RemoteFailureCapturedArtifactManifest,
) -> Result<RemoteFailureDebugDigest, RemoteFailureDebugReasonCode> {
    let mut identity = manifest.clone();
    identity.manifest_blake3 = zero_digest();
    digest_serialized(CAPTURE_MANIFEST_IDENTITY_DOMAIN, &identity)
}

fn summary_without_digest(summary: &RemoteFailureInspectSummary) -> RemoteFailureInspectSummary {
    let mut identity = summary.clone();
    identity.inspect_blake3 = zero_digest();
    identity
}

fn replay_plan_without_digest(plan: &RemoteFailureReplayPlan) -> RemoteFailureReplayPlan {
    let mut identity = plan.clone();
    identity.replay_plan_blake3 = zero_digest();
    identity
}

fn replay_comparison_without_digest(comparison: &RemoteFailureReplayComparison) -> RemoteFailureReplayComparison {
    let mut identity = comparison.clone();
    identity.comparison_blake3 = zero_digest();
    identity
}

fn digest_serialized(
    domain: &str,
    value: &impl Serialize,
) -> Result<RemoteFailureDebugDigest, RemoteFailureDebugReasonCode> {
    let bytes = serde_json::to_vec(value).map_err(|_| RemoteFailureDebugReasonCode::SerializedMetadataTooLarge)?;
    Ok(domain_digest(domain, &bytes))
}

fn domain_digest(domain: &str, bytes: &[u8]) -> RemoteFailureDebugDigest {
    let mut hasher = blake3::Hasher::new();
    hash_length_delimited(&mut hasher, domain.as_bytes());
    hash_length_delimited(&mut hasher, bytes);
    RemoteFailureDebugDigest(hasher.finalize().to_hex().to_string())
}

fn hash_length_delimited(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).expect("bounded debug input length fits u64");
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

fn zero_digest() -> RemoteFailureDebugDigest {
    RemoteFailureDebugDigest("0".repeat(BLAKE3_HEX_LENGTH_CHARS))
}

fn is_blake3_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn text_looks_secret_bearing(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("bearer ")
        || lower.contains("token=")
        || lower.contains("secret")
        || lower.contains("private-key")
        || lower.contains("private_key")
        || lower.contains("credential")
}

fn text_looks_host_path(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with('/')
        || lower.contains("/home/")
        || lower.contains("/tmp/")
        || lower.contains("/nix/store/")
        || lower.contains("/mantle/store/")
        || lower.contains("\\users\\")
}

fn compare_fact<T: PartialEq>(
    label: &str,
    original: &T,
    replay: &T,
    matching: &mut Vec<String>,
    divergent: &mut Vec<String>,
) {
    if original == replay {
        matching.push(label.to_string());
    } else {
        divergent.push(label.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CREATED_UNIX_S: u64 = 1_000;
    const FENCE_GENERATION: u64 = 3;
    const SMALL_CAPTURE_FILE_BYTES: u64 = 4;

    fn policy() -> RemoteFailureDebugPolicy {
        RemoteFailureDebugPolicy {
            replay_enabled: true,
            ..RemoteFailureDebugPolicy::default()
        }
    }

    fn reference(kind: &str) -> RemoteFailureDebugRef {
        remote_failure_debug_ref(kind, kind.as_bytes(), &policy()).unwrap()
    }

    fn scope() -> RemoteAttemptLogScope {
        RemoteAttemptLogScope {
            job_id: RemoteJobId::new("job-debug").unwrap(),
            attempt_id: RemoteAttemptId::new("attempt-debug").unwrap(),
            fence_generation: RemoteFenceGeneration::new(FENCE_GENERATION).unwrap(),
        }
    }

    fn facts(non_claims: Vec<String>) -> RemoteFailureDebugBundleFacts {
        let scope = scope();
        RemoteFailureDebugBundleFacts {
            action_ref: reference("action"),
            route_ref: reference("route"),
            worker_capability_ref: reference("worker"),
            input_manifest_ref: reference("inputs"),
            sandbox_policy_ref: reference("sandbox"),
            network_policy_ref: reference("network"),
            immutable_log: Some(RemoteFailureImmutableLogRef {
                scope: scope.clone(),
                manifest_blake3: RemoteAttemptLogDigest::new("a".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap(),
                retained_start_cursor: 0,
                next_cursor: 1,
                head_record_blake3: Some(RemoteAttemptLogDigest::new("b".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap()),
                head_segment_blake3: Some(RemoteAttemptLogDigest::new("c".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap()),
                truncated: false,
            }),
            transfer_ref: Some(reference("transfer")),
            admission_ref: Some(reference("admission")),
            captured_artifact_manifest_ref: None,
            original_job_id: scope.job_id,
            original_attempt_id: scope.attempt_id,
            original_fence_generation: scope.fence_generation,
            workspace_mode: RemoteFailureWorkspaceMode::Ephemeral,
            failure_phase: RemoteFailureDebugPhase::Execution,
            failure_reason_code: "sandbox-exit-nonzero".to_string(),
            capture_outcome_code: "metadata-only".to_string(),
            cleanup_status_code: "cleanup-complete".to_string(),
            created_unix_s: CREATED_UNIX_S,
            expires_unix_s: CREATED_UNIX_S + DEFAULT_REMOTE_FAILURE_RETENTION_SECS,
            non_claims,
        }
    }

    #[test]
    fn permutation_equivalent_bundle_facts_have_one_identity() {
        let left = seal_remote_failure_debug_bundle(
            facts(vec![
                "does-not-prove-reproducibility".to_string(),
                "does-not-admit-output".to_string(),
            ]),
            &policy(),
        )
        .unwrap();
        let right = seal_remote_failure_debug_bundle(
            facts(vec![
                "does-not-admit-output".to_string(),
                "does-not-prove-reproducibility".to_string(),
            ]),
            &policy(),
        )
        .unwrap();

        assert_eq!(left, right);
        assert_eq!(left.bundle_blake3, right.bundle_blake3);
    }

    #[test]
    fn metadata_only_bundle_validates_inspects_and_plans_new_authority_replay() {
        let bundle = seal_remote_failure_debug_bundle(facts(Vec::new()), &policy()).unwrap();
        let inspect = inspect_remote_failure_debug_bundle(&bundle, &policy()).unwrap();
        let available_refs = [
            bundle.action_ref.digest_blake3.clone(),
            bundle.input_manifest_ref.digest_blake3.clone(),
            bundle.sandbox_policy_ref.digest_blake3.clone(),
            bundle.network_policy_ref.digest_blake3.clone(),
        ]
        .into_iter()
        .collect();
        let replay = plan_remote_failure_replay(&bundle, &policy(), &RemoteFailureReplayAvailability {
            available_refs,
            current_policy_allows_replay: true,
        })
        .unwrap();

        assert!(inspect.immutable_log_available);
        assert!(replay.executable);
        assert!(replay.new_authority_required);
        assert!(replay.ordinary_output_admission_required);
        assert_ne!(inspect.inspect_blake3, replay.replay_plan_blake3);
    }

    #[test]
    fn malformed_and_mismatched_bundle_facts_fail_closed() {
        let mut wrong_attempt = facts(Vec::new());
        wrong_attempt.original_attempt_id = RemoteAttemptId::new("different-attempt").unwrap();
        assert_eq!(
            seal_remote_failure_debug_bundle(wrong_attempt, &policy()).unwrap_err(),
            RemoteFailureDebugReasonCode::AttemptBindingMismatch
        );

        let mut tampered = seal_remote_failure_debug_bundle(facts(Vec::new()), &policy()).unwrap();
        tampered.failure_reason_code = "other-reason".to_string();
        assert_eq!(
            validate_remote_failure_debug_bundle(&tampered, &policy()).unwrap_err(),
            RemoteFailureDebugReasonCode::ManifestIdentityMismatch
        );
    }

    #[test]
    fn capture_plan_rejects_traversal_secrets_host_paths_and_duplicates() {
        for rejected in ["../escape", "/tmp/host", "private-key", "nested/../escape", "token=raw"] {
            let capture = RemoteFailureCapturePolicy {
                enabled: true,
                allowed_relative_paths: vec![rejected.to_string()],
                ..RemoteFailureCapturePolicy::default()
            };
            assert_eq!(
                plan_remote_failure_capture(&capture).unwrap_err(),
                RemoteFailureDebugReasonCode::CapturePathInvalid,
                "{rejected}"
            );
        }
        let duplicate = RemoteFailureCapturePolicy {
            enabled: true,
            allowed_relative_paths: vec!["build/trace.json".to_string(), "build/trace.json".to_string()],
            ..RemoteFailureCapturePolicy::default()
        };
        assert_eq!(
            plan_remote_failure_capture(&duplicate).unwrap_err(),
            RemoteFailureDebugReasonCode::CapturePathDuplicate
        );
    }

    #[test]
    fn capture_observation_admission_is_bounded_and_regular_file_only() {
        let capture = RemoteFailureCapturePolicy {
            enabled: true,
            allowed_relative_paths: vec!["build/trace.json".to_string(), "build/socket".to_string()],
            total_bytes_max: SMALL_CAPTURE_FILE_BYTES,
            file_bytes_max: SMALL_CAPTURE_FILE_BYTES,
            ..RemoteFailureCapturePolicy::default()
        };
        let requests = plan_remote_failure_capture(&capture).unwrap();
        let plan = admit_remote_failure_capture_observations(
            &requests,
            vec![
                RemoteFailureCaptureObservation {
                    relative_path: "build/socket".to_string(),
                    kind: RemoteFailureCaptureObservedKind::Socket,
                    byte_count: 0,
                },
                RemoteFailureCaptureObservation {
                    relative_path: "build/trace.json".to_string(),
                    kind: RemoteFailureCaptureObservedKind::RegularFile,
                    byte_count: SMALL_CAPTURE_FILE_BYTES,
                },
            ],
            &capture,
        )
        .unwrap();

        assert_eq!(plan.accepted_paths, vec!["build/trace.json"]);
        assert_eq!(plan.accepted_total_bytes, SMALL_CAPTURE_FILE_BYTES);
        assert_eq!(plan.rejected[0].reason_code, RemoteFailureDebugReasonCode::CaptureUnsupportedKind);
    }

    #[test]
    fn capture_accounting_detects_overflow_before_publication() {
        let capture = RemoteFailureCapturePolicy {
            enabled: true,
            allowed_relative_paths: vec!["a".to_string(), "b".to_string()],
            total_bytes_max: u64::MAX,
            file_bytes_max: u64::MAX,
            ..RemoteFailureCapturePolicy::default()
        };
        assert_eq!(validate_capture_policy(&capture).unwrap_err(), RemoteFailureDebugReasonCode::CaptureLimitInvalid);
    }

    #[test]
    fn redaction_never_returns_raw_secret_host_path_or_control_text() {
        assert_eq!(redact_remote_failure_diagnostic("bearer SHOULD_NOT_LEAK").unwrap(), SECRET_REDACTION);
        assert_eq!(redact_remote_failure_diagnostic("/home/operator/private").unwrap(), SECRET_REDACTION);
        let escaped = redact_remote_failure_diagnostic("failed\u{0007}now").unwrap();
        assert!(!escaped.contains('\u{0007}'));
        assert!(escaped.contains('�'));
    }

    #[test]
    fn replay_comparison_reports_match_and_divergence_without_rewriting_truth() {
        let original = RemoteFailureExecutionSummary {
            failure_phase: Some(RemoteFailureDebugPhase::Execution),
            failure_reason_code: Some("sandbox-exit-nonzero".to_string()),
            exit_status_class: "nonzero".to_string(),
            admitted_output_digests: Vec::new(),
            log_head_blake3: None,
            captured_manifest_blake3: None,
        };
        let matching = compare_remote_failure_replay(&original, &original).unwrap();
        let mut replay = original.clone();
        replay.failure_phase = None;
        replay.failure_reason_code = None;
        replay.exit_status_class = "zero".to_string();
        replay
            .admitted_output_digests
            .push(RemoteFailureDebugDigest::new("d".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap());
        let comparison = compare_remote_failure_replay(&original, &replay).unwrap();

        assert_eq!(matching.class, RemoteFailureReplayComparisonClass::Match);
        assert!(matching.divergent_fact_classes.is_empty());
        assert_eq!(comparison.class, RemoteFailureReplayComparisonClass::Diverged);
        assert!(comparison.original_result_immutable);
        assert!(comparison.replay_output_requires_ordinary_admission);
        assert!(comparison.divergent_fact_classes.contains(&"admitted-output-digests".to_string()));
    }

    #[test]
    fn retention_preserves_active_lease_and_deletes_only_expired_debug_roots() {
        let leased = RemoteFailureDebugDigest::new("a".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap();
        let expired = RemoteFailureDebugDigest::new("b".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap();
        let live = RemoteFailureDebugDigest::new("c".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap();
        let plan = plan_remote_failure_retention(
            vec![
                RemoteFailureRetentionRecord {
                    bundle_blake3: expired.clone(),
                    expires_unix_s: CREATED_UNIX_S,
                    active_lease: false,
                    ordinary_build_output: false,
                },
                RemoteFailureRetentionRecord {
                    bundle_blake3: leased.clone(),
                    expires_unix_s: CREATED_UNIX_S,
                    active_lease: true,
                    ordinary_build_output: false,
                },
                RemoteFailureRetentionRecord {
                    bundle_blake3: live.clone(),
                    expires_unix_s: CREATED_UNIX_S + DEFAULT_REMOTE_FAILURE_RETENTION_SECS,
                    active_lease: false,
                    ordinary_build_output: false,
                },
            ],
            CREATED_UNIX_S,
        )
        .unwrap();

        assert_eq!(plan.delete, vec![expired]);
        assert_eq!(plan.preserve, vec![leased, live]);
    }

    #[test]
    fn retention_rejects_ordinary_build_output_deletion_authority() {
        let result = plan_remote_failure_retention(
            vec![RemoteFailureRetentionRecord {
                bundle_blake3: RemoteFailureDebugDigest::new("e".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap(),
                expires_unix_s: 0,
                active_lease: false,
                ordinary_build_output: true,
            }],
            CREATED_UNIX_S,
        );
        assert_eq!(result.unwrap_err(), RemoteFailureDebugReasonCode::OrdinaryOutputDeletionForbidden);
    }
}
