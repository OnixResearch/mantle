//! Host DTOs, canonical v1 digests, and immutable-log plans.
//!
//! Borrowed validation and bounded decisions live in `crunch_remote_core::attempt_log`.
//! This adapter preserves the existing serde wire format and BLAKE3/JSON preimages;
//! callers persist immutable objects before atomically advancing the manifest.
//!
//! r[impl remote_builds.immutable_attempt_log_segments]
//! r[impl remote_builds.pure_log_cursor_kernel]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteAttemptPhase;
use crunch_remote_core::attempt::RemoteEventId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use crunch_remote_core::attempt_log as kernel;
use serde::Deserialize;
use serde::Serialize;

pub const REMOTE_ATTEMPT_LOG_RECORD_SCHEMA: &str = "mantle-remote-attempt-log-record-v1";
pub const REMOTE_ATTEMPT_LOG_SEGMENT_SCHEMA: &str = "mantle-remote-attempt-log-segment-v1";
pub const REMOTE_ATTEMPT_LOG_MANIFEST_SCHEMA: &str = "mantle-remote-attempt-log-manifest-v1";
pub const REMOTE_ATTEMPT_LOG_TRUNCATION_ANCHOR_SCHEMA: &str = "mantle-remote-attempt-log-truncation-anchor-v1";

pub const MAX_REMOTE_ATTEMPT_LOG_POLICY_NAME_BYTES: usize = kernel::MAX_POLICY_NAME_BYTES;
pub const MAX_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES_HARD: u32 = kernel::MAX_RECORD_PAYLOAD_BYTES;
pub const MAX_REMOTE_ATTEMPT_LOG_SEGMENT_RECORDS_HARD: u32 = kernel::MAX_SEGMENT_RECORDS;
pub const MAX_REMOTE_ATTEMPT_LOG_SEGMENT_PAYLOAD_BYTES_HARD: u64 = kernel::MAX_SEGMENT_PAYLOAD_BYTES;
pub const MAX_REMOTE_ATTEMPT_LOG_MANIFEST_SEGMENTS_HARD: u32 = kernel::MAX_MANIFEST_SEGMENTS;
pub const MAX_REMOTE_ATTEMPT_LOG_RETAINED_RECORDS_HARD: u32 = kernel::MAX_RETAINED_RECORDS;
pub const MAX_REMOTE_ATTEMPT_LOG_RETAINED_PAYLOAD_BYTES_HARD: u64 = kernel::MAX_RETAINED_PAYLOAD_BYTES;
pub const MAX_REMOTE_ATTEMPT_LOG_REPLAY_RECORDS_HARD: u32 = kernel::MAX_REPLAY_RECORDS;
pub const MAX_REMOTE_ATTEMPT_LOG_REPLAY_PAYLOAD_BYTES_HARD: u64 = kernel::MAX_REPLAY_PAYLOAD_BYTES;
pub const MAX_REMOTE_ATTEMPT_LOG_EVENT_IDENTITIES_HARD: u32 = kernel::MAX_EVENT_IDENTITIES;

pub const DEFAULT_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES: u32 = 65_536;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_SEGMENT_RECORDS: u32 = 256;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_SEGMENT_PAYLOAD_BYTES: u64 = 1_048_576;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_MANIFEST_SEGMENTS: u32 = 1_024;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_RETAINED_SEGMENTS: u32 = 256;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_RETAINED_RECORDS: u32 = 1_024;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_RETAINED_PAYLOAD_BYTES: u64 = 1_048_576;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_REPLAY_RECORDS: u32 = 256;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_REPLAY_PAYLOAD_BYTES: u64 = 262_144;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_EVENT_IDENTITIES: u32 = 1_024;
pub const DEFAULT_REMOTE_ATTEMPT_LOG_POLICY_NAME: &str = "mantle-default-remote-attempt-log-v1";

const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const INITIAL_LOG_POSITION: u64 = 0;
#[cfg(test)]
const INITIAL_SEGMENT_INDEX: u64 = 0;
const RECORD_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-record-v1";
const SEGMENT_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-segment-v1";
const MANIFEST_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-manifest-v1";
const TRUNCATION_ANCHOR_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-truncation-anchor-v1";
const POLICY_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-policy-v1";
const REDACTED_PAYLOAD: &[u8] = kernel::REDACTED_PAYLOAD;
#[cfg(test)]
const MIN_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES: u32 = REDACTED_PAYLOAD.len() as u32;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemoteAttemptLogDigest(String);

impl RemoteAttemptLogDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteAttemptLogReasonCode> {
        let value = value.into();
        if !is_blake3_hex_digest(&value) {
            return Err(RemoteAttemptLogReasonCode::DigestInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogScope {
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAttemptLogStream {
    Stdout,
    Stderr,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAttemptLogRecordKind {
    Output,
    PhaseTransition,
    Diagnostic,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogRecordFlags {
    pub secret_redacted: bool,
    pub control_escaped: bool,
    pub payload_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogRedactionPlan {
    pub payload: Vec<u8>,
    pub flags: RemoteAttemptLogRecordFlags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogRecordInput {
    pub scope: RemoteAttemptLogScope,
    pub event_id: RemoteEventId,
    pub sequence: u64,
    pub cursor: u64,
    pub phase: RemoteAttemptPhase,
    pub stream: RemoteAttemptLogStream,
    pub kind: RemoteAttemptLogRecordKind,
    pub payload: Vec<u8>,
    pub previous_record_blake3: Option<RemoteAttemptLogDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogRecord {
    pub schema: String,
    pub scope: RemoteAttemptLogScope,
    pub event_id: RemoteEventId,
    pub sequence: u64,
    pub cursor: u64,
    pub phase: RemoteAttemptPhase,
    pub stream: RemoteAttemptLogStream,
    pub kind: RemoteAttemptLogRecordKind,
    pub payload_length_bytes: u32,
    pub payload_blake3: RemoteAttemptLogDigest,
    pub previous_record_blake3: Option<RemoteAttemptLogDigest>,
    pub flags: RemoteAttemptLogRecordFlags,
    pub payload: Vec<u8>,
    pub record_blake3: RemoteAttemptLogDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogSegment {
    pub schema: String,
    pub scope: RemoteAttemptLogScope,
    pub segment_index: u64,
    pub previous_segment_blake3: Option<RemoteAttemptLogDigest>,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub first_cursor: u64,
    pub next_cursor: u64,
    pub record_count: u32,
    pub payload_bytes: u64,
    pub records: Vec<RemoteAttemptLogRecord>,
    pub segment_blake3: RemoteAttemptLogDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogSegmentRef {
    pub segment_index: u64,
    pub segment_blake3: RemoteAttemptLogDigest,
    pub previous_segment_blake3: Option<RemoteAttemptLogDigest>,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub first_cursor: u64,
    pub next_cursor: u64,
    pub first_previous_record_blake3: Option<RemoteAttemptLogDigest>,
    pub head_record_blake3: RemoteAttemptLogDigest,
    pub record_count: u32,
    pub payload_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogPolicy {
    pub record_payload_bytes_max: u32,
    pub segment_record_count_max: u32,
    pub segment_payload_bytes_max: u64,
    pub manifest_segment_count_max: u32,
    pub retained_segment_count_max: u32,
    pub retained_record_count_max: u32,
    pub retained_payload_bytes_max: u64,
    pub replay_record_count_max: u32,
    pub replay_payload_bytes_max: u64,
    pub event_identity_count_max: u32,
}

impl Default for RemoteAttemptLogPolicy {
    fn default() -> Self {
        Self {
            record_payload_bytes_max: DEFAULT_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES,
            segment_record_count_max: DEFAULT_REMOTE_ATTEMPT_LOG_SEGMENT_RECORDS,
            segment_payload_bytes_max: DEFAULT_REMOTE_ATTEMPT_LOG_SEGMENT_PAYLOAD_BYTES,
            manifest_segment_count_max: DEFAULT_REMOTE_ATTEMPT_LOG_MANIFEST_SEGMENTS,
            retained_segment_count_max: DEFAULT_REMOTE_ATTEMPT_LOG_RETAINED_SEGMENTS,
            retained_record_count_max: DEFAULT_REMOTE_ATTEMPT_LOG_RETAINED_RECORDS,
            retained_payload_bytes_max: DEFAULT_REMOTE_ATTEMPT_LOG_RETAINED_PAYLOAD_BYTES,
            replay_record_count_max: DEFAULT_REMOTE_ATTEMPT_LOG_REPLAY_RECORDS,
            replay_payload_bytes_max: DEFAULT_REMOTE_ATTEMPT_LOG_REPLAY_PAYLOAD_BYTES,
            event_identity_count_max: DEFAULT_REMOTE_ATTEMPT_LOG_EVENT_IDENTITIES,
        }
    }
}

impl RemoteAttemptLogPolicy {
    pub fn validate(self) -> Result<(), RemoteAttemptLogReasonCode> {
        kernel::validate_policy(kernel_policy(self)).map_err(RemoteAttemptLogReasonCode::from)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogPolicyIdentity {
    pub name: String,
    pub digest_blake3: RemoteAttemptLogDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogTruncationAnchorRecord {
    pub schema: String,
    pub scope: RemoteAttemptLogScope,
    pub dropped_start_cursor: u64,
    pub dropped_end_cursor_exclusive: u64,
    pub new_retained_start_cursor: u64,
    pub dropped_chunk_count: u32,
    pub dropped_record_count: u32,
    pub dropped_payload_bytes: u64,
    pub dropped_tail_record_blake3: RemoteAttemptLogDigest,
    pub dropped_tail_segment_blake3: RemoteAttemptLogDigest,
    pub prior_head_record_blake3: RemoteAttemptLogDigest,
    pub prior_head_segment_blake3: RemoteAttemptLogDigest,
    pub policy: RemoteAttemptLogPolicyIdentity,
    pub previous_anchor_blake3: Option<RemoteAttemptLogDigest>,
    pub anchor_blake3: RemoteAttemptLogDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogManifest {
    pub schema: String,
    pub scope: RemoteAttemptLogScope,
    pub policy: RemoteAttemptLogPolicyIdentity,
    pub retained_start_cursor: u64,
    pub next_cursor: u64,
    pub head_record_blake3: Option<RemoteAttemptLogDigest>,
    pub head_segment_blake3: Option<RemoteAttemptLogDigest>,
    pub retained_record_count: u32,
    pub retained_payload_bytes: u64,
    pub segments: Vec<RemoteAttemptLogSegmentRef>,
    pub event_records: BTreeMap<RemoteEventId, RemoteAttemptLogDigest>,
    pub truncation_anchor: Option<RemoteAttemptLogTruncationAnchorRecord>,
    pub manifest_blake3: RemoteAttemptLogDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogCurrentAttemptFacts {
    pub scope: RemoteAttemptLogScope,
    pub phase: RemoteAttemptPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAttemptLogAppendDisposition {
    Append,
    AlreadyApplied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogAppendPlan {
    pub disposition: RemoteAttemptLogAppendDisposition,
    pub reason_code: RemoteAttemptLogReasonCode,
    pub segment: Option<RemoteAttemptLogSegment>,
    pub next_manifest: RemoteAttemptLogManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogChainSummary {
    pub retained_start_cursor: u64,
    pub next_cursor: u64,
    pub segment_count: u32,
    pub record_count: u32,
    pub payload_bytes: u64,
    pub head_record_blake3: Option<RemoteAttemptLogDigest>,
    pub head_segment_blake3: Option<RemoteAttemptLogDigest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAttemptLogReplayRequest {
    pub from_cursor: u64,
    pub record_count_max: u32,
    pub payload_bytes_max: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAttemptLogCursorDisposition {
    Retained,
    AtHead,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogCursorDecision {
    pub disposition: RemoteAttemptLogCursorDisposition,
    pub requested_cursor: u64,
    pub effective_cursor: u64,
    pub reason_code: RemoteAttemptLogReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogReplayPlan {
    pub cursor: RemoteAttemptLogCursorDecision,
    pub records: Vec<RemoteAttemptLogRecord>,
    pub next_cursor: u64,
    pub has_more: bool,
    pub truncation_anchor: Option<RemoteAttemptLogTruncationAnchorRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAttemptLogRetentionDisposition {
    Unchanged,
    Truncate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptLogRetentionPlan {
    pub disposition: RemoteAttemptLogRetentionDisposition,
    pub reason_code: RemoteAttemptLogReasonCode,
    pub anchor: Option<RemoteAttemptLogTruncationAnchorRecord>,
    pub delete_segment_blake3: Vec<RemoteAttemptLogDigest>,
    pub retained_segments: Vec<RemoteAttemptLogSegment>,
    pub next_manifest: RemoteAttemptLogManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteAttemptLogReasonCode {
    AppendAccepted,
    AlreadyApplied,
    EventDigestConflict,
    EventIdentityCapacityExceeded,
    RecordCanonical,
    RecordSchemaUnsupported,
    RecordPayloadTooLarge,
    RecordPayloadMetadataMismatch,
    RecordDigestMismatch,
    RecordPositionMismatch,
    PreviousRecordMismatch,
    SegmentCanonical,
    SegmentSchemaUnsupported,
    SegmentEmpty,
    SegmentBoundsExceeded,
    SegmentDigestMismatch,
    SegmentChainMismatch,
    ManifestCanonical,
    ManifestSchemaUnsupported,
    ManifestBoundsExceeded,
    ManifestDigestMismatch,
    ManifestSummaryMismatch,
    PolicyInvalid,
    PolicyIdentityMismatch,
    PolicySerializationFailed,
    ScopeIdentityInvalid,
    ScopeMismatch,
    StaleFenceRejected,
    UnknownFenceRejected,
    AttemptIdentityMismatch,
    AttemptPhaseMismatch,
    DigestInvalid,
    SerializationFailed,
    ArithmeticOverflow,
    CursorRetained,
    CursorAtHead,
    CursorTruncated,
    CursorAfterHead,
    ReplayRequestInvalid,
    ReplayLimitTooSmall,
    RetentionUnchanged,
    RetentionPlanned,
    RetentionAnchorInvalid,
    RetentionWouldDropAll,
}

impl RemoteAttemptLogReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppendAccepted => "attempt-log-append-accepted",
            Self::AlreadyApplied => "attempt-log-already-applied",
            Self::EventDigestConflict => "attempt-log-event-digest-conflict",
            Self::EventIdentityCapacityExceeded => "attempt-log-event-identity-capacity-exceeded",
            Self::RecordCanonical => "attempt-log-record-canonical",
            Self::RecordSchemaUnsupported => "attempt-log-record-schema-unsupported",
            Self::RecordPayloadTooLarge => "attempt-log-record-payload-too-large",
            Self::RecordPayloadMetadataMismatch => "attempt-log-record-payload-metadata-mismatch",
            Self::RecordDigestMismatch => "attempt-log-record-digest-mismatch",
            Self::RecordPositionMismatch => "attempt-log-record-position-mismatch",
            Self::PreviousRecordMismatch => "attempt-log-previous-record-mismatch",
            Self::SegmentCanonical => "attempt-log-segment-canonical",
            Self::SegmentSchemaUnsupported => "attempt-log-segment-schema-unsupported",
            Self::SegmentEmpty => "attempt-log-segment-empty",
            Self::SegmentBoundsExceeded => "attempt-log-segment-bounds-exceeded",
            Self::SegmentDigestMismatch => "attempt-log-segment-digest-mismatch",
            Self::SegmentChainMismatch => "attempt-log-segment-chain-mismatch",
            Self::ManifestCanonical => "attempt-log-manifest-canonical",
            Self::ManifestSchemaUnsupported => "attempt-log-manifest-schema-unsupported",
            Self::ManifestBoundsExceeded => "attempt-log-manifest-bounds-exceeded",
            Self::ManifestDigestMismatch => "attempt-log-manifest-digest-mismatch",
            Self::ManifestSummaryMismatch => "attempt-log-manifest-summary-mismatch",
            Self::PolicyInvalid => "attempt-log-policy-invalid",
            Self::PolicyIdentityMismatch => "attempt-log-policy-identity-mismatch",
            Self::PolicySerializationFailed => "attempt-log-policy-serialization-failed",
            Self::ScopeIdentityInvalid => "attempt-log-scope-identity-invalid",
            Self::ScopeMismatch => "attempt-log-scope-mismatch",
            Self::StaleFenceRejected => "attempt-log-stale-fence-rejected",
            Self::UnknownFenceRejected => "attempt-log-unknown-fence-rejected",
            Self::AttemptIdentityMismatch => "attempt-log-attempt-identity-mismatch",
            Self::AttemptPhaseMismatch => "attempt-log-attempt-phase-mismatch",
            Self::DigestInvalid => "attempt-log-digest-invalid",
            Self::SerializationFailed => "attempt-log-serialization-failed",
            Self::ArithmeticOverflow => "attempt-log-arithmetic-overflow",
            Self::CursorRetained => "attempt-log-cursor-retained",
            Self::CursorAtHead => "attempt-log-cursor-at-head",
            Self::CursorTruncated => "attempt-log-cursor-truncated",
            Self::CursorAfterHead => "attempt-log-cursor-after-head",
            Self::ReplayRequestInvalid => "attempt-log-replay-request-invalid",
            Self::ReplayLimitTooSmall => "attempt-log-replay-limit-too-small",
            Self::RetentionUnchanged => "attempt-log-retention-unchanged",
            Self::RetentionPlanned => "attempt-log-retention-planned",
            Self::RetentionAnchorInvalid => "attempt-log-retention-anchor-invalid",
            Self::RetentionWouldDropAll => "attempt-log-retention-would-drop-all",
        }
    }
}

impl From<kernel::AttemptLogError> for RemoteAttemptLogReasonCode {
    fn from(reason: kernel::AttemptLogError) -> Self {
        match reason {
            kernel::AttemptLogError::ScopeMismatch => Self::ScopeMismatch,
            kernel::AttemptLogError::StaleFenceRejected => Self::StaleFenceRejected,
            kernel::AttemptLogError::UnknownFenceRejected => Self::UnknownFenceRejected,
            kernel::AttemptLogError::AttemptIdentityMismatch => Self::AttemptIdentityMismatch,
            kernel::AttemptLogError::AttemptPhaseMismatch => Self::AttemptPhaseMismatch,
            kernel::AttemptLogError::EventDigestConflict => Self::EventDigestConflict,
            kernel::AttemptLogError::EventIdentityCapacityExceeded => Self::EventIdentityCapacityExceeded,
            kernel::AttemptLogError::RecordPositionMismatch => Self::RecordPositionMismatch,
            kernel::AttemptLogError::PreviousRecordMismatch => Self::PreviousRecordMismatch,
            kernel::AttemptLogError::SegmentChainMismatch => Self::SegmentChainMismatch,
            kernel::AttemptLogError::SegmentBoundsExceeded => Self::SegmentBoundsExceeded,
            kernel::AttemptLogError::ManifestSummaryMismatch => Self::ManifestSummaryMismatch,
            kernel::AttemptLogError::ManifestBoundsExceeded => Self::ManifestBoundsExceeded,
            kernel::AttemptLogError::RetentionAnchorInvalid => Self::RetentionAnchorInvalid,
            kernel::AttemptLogError::RecordSchemaUnsupported => Self::RecordSchemaUnsupported,
            kernel::AttemptLogError::SegmentSchemaUnsupported => Self::SegmentSchemaUnsupported,
            kernel::AttemptLogError::ManifestSchemaUnsupported => Self::ManifestSchemaUnsupported,
            kernel::AttemptLogError::RecordDigestMismatch => Self::RecordDigestMismatch,
            kernel::AttemptLogError::SegmentDigestMismatch => Self::SegmentDigestMismatch,
            kernel::AttemptLogError::ManifestDigestMismatch => Self::ManifestDigestMismatch,
            kernel::AttemptLogError::DigestInvalid => Self::DigestInvalid,
            kernel::AttemptLogError::ArithmeticOverflow => Self::ArithmeticOverflow,
            kernel::AttemptLogError::CursorAfterHead => Self::CursorAfterHead,
            kernel::AttemptLogError::PolicyInvalid => Self::PolicyInvalid,
            kernel::AttemptLogError::PolicyIdentityMismatch => Self::PolicyIdentityMismatch,
            kernel::AttemptLogError::RecordPayloadTooLarge => Self::RecordPayloadTooLarge,
            kernel::AttemptLogError::ScopeIdentityInvalid => Self::ScopeIdentityInvalid,
            kernel::AttemptLogError::RecordPayloadMetadataMismatch => Self::RecordPayloadMetadataMismatch,
            kernel::AttemptLogError::ReplayRequestInvalid => Self::ReplayRequestInvalid,
            kernel::AttemptLogError::ReplayLimitTooSmall => Self::ReplayLimitTooSmall,
            kernel::AttemptLogError::RetentionWouldDropAll => Self::RetentionWouldDropAll,
        }
    }
}

fn borrowed_scope(scope: &RemoteAttemptLogScope) -> kernel::Scope<'_> {
    kernel::Scope {
        job_id: scope.job_id.as_str(),
        attempt_id: scope.attempt_id.as_str(),
        fence_generation: scope.fence_generation.get(),
    }
}

fn kernel_policy(policy: RemoteAttemptLogPolicy) -> kernel::Policy {
    kernel::Policy {
        record_payload_bytes_max: policy.record_payload_bytes_max,
        segment_record_count_max: policy.segment_record_count_max,
        segment_payload_bytes_max: policy.segment_payload_bytes_max,
        manifest_segment_count_max: policy.manifest_segment_count_max,
        retained_segment_count_max: policy.retained_segment_count_max,
        retained_record_count_max: policy.retained_record_count_max,
        retained_payload_bytes_max: policy.retained_payload_bytes_max,
        replay_record_count_max: policy.replay_record_count_max,
        replay_payload_bytes_max: policy.replay_payload_bytes_max,
        event_identity_count_max: policy.event_identity_count_max,
    }
}

pub fn canonical_remote_attempt_log_policy_identity(
    name: impl Into<String>,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogPolicyIdentity, RemoteAttemptLogReasonCode> {
    policy.validate()?;
    let name = name.into();
    kernel::validate_policy_name(&name).map_err(RemoteAttemptLogReasonCode::from)?;
    let digest_blake3 = policy_digest(policy)?;
    debug_assert!(!name.is_empty());
    debug_assert!(is_blake3_hex_digest(digest_blake3.as_str()));
    Ok(RemoteAttemptLogPolicyIdentity { name, digest_blake3 })
}

fn policy_digest(policy: RemoteAttemptLogPolicy) -> Result<RemoteAttemptLogDigest, RemoteAttemptLogReasonCode> {
    let bytes = serde_json::to_vec(&policy).map_err(|_| RemoteAttemptLogReasonCode::PolicySerializationFailed)?;
    domain_hash(POLICY_DIGEST_DOMAIN, &bytes)
}

pub fn redact_remote_attempt_log_payload(
    payload: &[u8],
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogRedactionPlan, RemoteAttemptLogReasonCode> {
    policy.validate()?;
    let redacted =
        kernel::redact_payload(payload, policy.record_payload_bytes_max).map_err(RemoteAttemptLogReasonCode::from)?;
    Ok(RemoteAttemptLogRedactionPlan {
        payload: redacted.payload,
        flags: RemoteAttemptLogRecordFlags {
            secret_redacted: redacted.secret_redacted,
            control_escaped: redacted.control_escaped,
            payload_truncated: redacted.payload_truncated,
        },
    })
}

pub fn seal_remote_attempt_log_record(
    input: RemoteAttemptLogRecordInput,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogRecord, RemoteAttemptLogReasonCode> {
    policy.validate()?;
    validate_scope(&input.scope)?;
    validate_event_id(&input.event_id)?;
    kernel::check_record_position(input.sequence, input.cursor).map_err(RemoteAttemptLogReasonCode::from)?;
    let redaction = redact_remote_attempt_log_payload(&input.payload, policy)?;
    let payload_length_bytes = usize_to_u32(redaction.payload.len())?;
    let payload_blake3 = content_hash(&redaction.payload);
    let mut record = RemoteAttemptLogRecord {
        schema: REMOTE_ATTEMPT_LOG_RECORD_SCHEMA.to_string(),
        scope: input.scope,
        event_id: input.event_id,
        sequence: input.sequence,
        cursor: input.cursor,
        phase: input.phase,
        stream: input.stream,
        kind: input.kind,
        payload_length_bytes,
        payload_blake3,
        previous_record_blake3: input.previous_record_blake3,
        flags: redaction.flags,
        payload: redaction.payload,
        record_blake3: zero_digest(),
    };
    record.record_blake3 = record_payload_digest(&record)?;
    validate_remote_attempt_log_record(&record, policy)?;
    debug_assert_eq!(record.sequence, record.cursor);
    debug_assert_eq!(record.payload_length_bytes, payload_length_bytes);
    Ok(record)
}

pub fn validate_remote_attempt_log_record(
    record: &RemoteAttemptLogRecord,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    policy.validate()?;
    kernel::check_schema(&record.schema, REMOTE_ATTEMPT_LOG_RECORD_SCHEMA, kernel::ObjectKind::Record)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    validate_scope(&record.scope)?;
    validate_event_id(&record.event_id)?;
    kernel::check_record_position(record.sequence, record.cursor).map_err(RemoteAttemptLogReasonCode::from)?;
    validate_record_payload_metadata(record, policy)?;
    validate_optional_digest(&record.previous_record_blake3)?;
    kernel::check_object_digest(
        record.record_blake3.as_str(),
        record_payload_digest(record)?.as_str(),
        kernel::ObjectKind::Record,
    )
    .map_err(RemoteAttemptLogReasonCode::from)?;
    debug_assert!(record.payload.len() <= u32_to_usize(policy.record_payload_bytes_max)?);
    debug_assert!(is_blake3_hex_digest(record.record_blake3.as_str()));
    Ok(())
}

pub fn seal_remote_attempt_log_segment(
    records: Vec<RemoteAttemptLogRecord>,
    segment_index: u64,
    previous_segment_blake3: Option<RemoteAttemptLogDigest>,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogSegment, RemoteAttemptLogReasonCode> {
    policy.validate()?;
    let summary = summarize_segment_records(&records, policy)?;
    validate_optional_digest(&previous_segment_blake3)?;
    let mut segment = RemoteAttemptLogSegment {
        schema: REMOTE_ATTEMPT_LOG_SEGMENT_SCHEMA.to_string(),
        scope: summary.scope,
        segment_index,
        previous_segment_blake3,
        first_sequence: summary.first_sequence,
        last_sequence: summary.last_sequence,
        first_cursor: summary.first_cursor,
        next_cursor: summary.next_cursor,
        record_count: summary.record_count,
        payload_bytes: summary.payload_bytes,
        records,
        segment_blake3: zero_digest(),
    };
    segment.segment_blake3 = segment_payload_digest(&segment)?;
    validate_remote_attempt_log_segment(&segment, policy)?;
    debug_assert!(segment.record_count > 0);
    debug_assert!(segment.next_cursor > segment.first_cursor);
    Ok(segment)
}

pub fn validate_remote_attempt_log_segment(
    segment: &RemoteAttemptLogSegment,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    policy.validate()?;
    kernel::check_schema(&segment.schema, REMOTE_ATTEMPT_LOG_SEGMENT_SCHEMA, kernel::ObjectKind::Segment)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    validate_scope(&segment.scope)?;
    validate_optional_digest(&segment.previous_segment_blake3)?;
    let summary = summarize_segment_records(&segment.records, policy)?;
    validate_segment_summary(segment, &summary)?;
    kernel::check_object_digest(
        segment.segment_blake3.as_str(),
        segment_payload_digest(segment)?.as_str(),
        kernel::ObjectKind::Segment,
    )
    .map_err(RemoteAttemptLogReasonCode::from)?;
    debug_assert_eq!(segment.scope, summary.scope);
    debug_assert_eq!(segment.record_count, summary.record_count);
    Ok(())
}

pub fn empty_remote_attempt_log_manifest(
    scope: RemoteAttemptLogScope,
    policy_name: impl Into<String>,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogManifest, RemoteAttemptLogReasonCode> {
    validate_scope(&scope)?;
    let policy_identity = canonical_remote_attempt_log_policy_identity(policy_name, policy)?;
    let manifest = RemoteAttemptLogManifest {
        schema: REMOTE_ATTEMPT_LOG_MANIFEST_SCHEMA.to_string(),
        scope,
        policy: policy_identity,
        retained_start_cursor: INITIAL_LOG_POSITION,
        next_cursor: INITIAL_LOG_POSITION,
        head_record_blake3: None,
        head_segment_blake3: None,
        retained_record_count: 0,
        retained_payload_bytes: 0,
        segments: Vec::new(),
        event_records: BTreeMap::new(),
        truncation_anchor: None,
        manifest_blake3: zero_digest(),
    };
    let sealed = seal_manifest(manifest, policy)?;
    debug_assert!(sealed.segments.is_empty());
    debug_assert_eq!(sealed.retained_start_cursor, sealed.next_cursor);
    Ok(sealed)
}

pub fn validate_remote_attempt_log_manifest(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    validate_manifest_shape(manifest, policy)?;
    kernel::check_object_digest(
        manifest.manifest_blake3.as_str(),
        manifest_payload_digest(manifest)?.as_str(),
        kernel::ObjectKind::Manifest,
    )
    .map_err(RemoteAttemptLogReasonCode::from)?;
    debug_assert!(manifest.segments.len() <= u32_to_usize(policy.manifest_segment_count_max)?);
    debug_assert!(manifest.retained_start_cursor <= manifest.next_cursor);
    Ok(())
}

pub fn plan_remote_attempt_log_append(
    manifest: &RemoteAttemptLogManifest,
    current: &RemoteAttemptLogCurrentAttemptFacts,
    record: &RemoteAttemptLogRecord,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogAppendPlan, RemoteAttemptLogReasonCode> {
    validate_remote_attempt_log_manifest(manifest, policy)?;
    validate_remote_attempt_log_record(record, policy)?;
    validate_current_scope(&manifest.scope, &current.scope)?;
    validate_current_scope(&current.scope, &record.scope)?;
    kernel::check_phase(current.phase, record.phase).map_err(RemoteAttemptLogReasonCode::from)?;
    if let Some(existing) = manifest.event_records.get(&record.event_id) {
        return classify_existing_record(manifest, existing, record);
    }
    kernel::check_event_capacity(manifest.event_records.len(), u32_to_usize(policy.event_identity_count_max)?)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    validate_append_position(manifest, record)?;
    kernel::check_segment_capacity(manifest.segments.len(), policy.manifest_segment_count_max)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    let segment_index = next_segment_index(manifest)?;
    let segment = seal_remote_attempt_log_segment(
        vec![record.clone()],
        segment_index,
        manifest.head_segment_blake3.clone(),
        policy,
    )?;
    let next_manifest = append_segment_to_manifest(manifest, &segment, policy)?;
    let expected_next_cursor = kernel::checked_add_u64(record.cursor, 1).map_err(RemoteAttemptLogReasonCode::from)?;
    debug_assert_eq!(segment.record_count, 1);
    debug_assert_eq!(next_manifest.next_cursor, expected_next_cursor);
    Ok(RemoteAttemptLogAppendPlan {
        disposition: RemoteAttemptLogAppendDisposition::Append,
        reason_code: RemoteAttemptLogReasonCode::AppendAccepted,
        segment: Some(segment),
        next_manifest,
    })
}

pub fn validate_remote_attempt_log_chain(
    manifest: &RemoteAttemptLogManifest,
    segments: &[RemoteAttemptLogSegment],
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogChainSummary, RemoteAttemptLogReasonCode> {
    validate_remote_attempt_log_manifest(manifest, policy)?;
    kernel::check_chain_length(segments.len(), manifest.segments.len()).map_err(RemoteAttemptLogReasonCode::from)?;
    let mut expected_cursor = manifest.retained_start_cursor;
    let mut expected_sequence = manifest.retained_start_cursor;
    let mut previous_record = manifest.truncation_anchor.as_ref().map(|anchor| &anchor.dropped_tail_record_blake3);
    let mut previous_segment = manifest.truncation_anchor.as_ref().map(|anchor| &anchor.dropped_tail_segment_blake3);
    let mut record_count = 0_u32;
    let mut payload_bytes = 0_u64;
    let mut retained_events = BTreeSet::new();
    for (segment, expected_ref) in segments.iter().zip(&manifest.segments) {
        validate_remote_attempt_log_segment(segment, policy)?;
        validate_segment_against_ref(segment, expected_ref)?;
        validate_chain_link(segment, ChainLinkExpectation {
            scope: &manifest.scope,
            expected_sequence,
            expected_cursor,
            previous_record_blake3: previous_record,
            previous_segment_blake3: previous_segment,
        })?;
        validate_retained_event_records(segment, manifest, &mut retained_events)?;
        record_count =
            kernel::checked_add_u32(record_count, segment.record_count).map_err(RemoteAttemptLogReasonCode::from)?;
        payload_bytes =
            kernel::checked_add_u64(payload_bytes, segment.payload_bytes).map_err(RemoteAttemptLogReasonCode::from)?;
        expected_sequence = segment.next_cursor;
        expected_cursor = segment.next_cursor;
        let head_record = segment.records.last().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
        previous_record = Some(&head_record.record_blake3);
        previous_segment = Some(&segment.segment_blake3);
    }
    let summary = RemoteAttemptLogChainSummary {
        retained_start_cursor: manifest.retained_start_cursor,
        next_cursor: expected_cursor,
        segment_count: usize_to_u32(segments.len())?,
        record_count,
        payload_bytes,
        head_record_blake3: previous_record.cloned(),
        head_segment_blake3: previous_segment.cloned(),
    };
    validate_chain_summary(manifest, &summary)?;
    debug_assert_eq!(summary.record_count, manifest.retained_record_count);
    debug_assert_eq!(summary.payload_bytes, manifest.retained_payload_bytes);
    Ok(summary)
}

pub fn decide_remote_attempt_log_cursor(
    manifest: &RemoteAttemptLogManifest,
    requested_cursor: u64,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogCursorDecision, RemoteAttemptLogReasonCode> {
    validate_remote_attempt_log_manifest(manifest, policy)?;
    let disposition =
        match kernel::classify_cursor(manifest.retained_start_cursor, manifest.next_cursor, requested_cursor)
            .map_err(RemoteAttemptLogReasonCode::from)?
        {
            kernel::CursorDisposition::Retained => RemoteAttemptLogCursorDisposition::Retained,
            kernel::CursorDisposition::AtHead => RemoteAttemptLogCursorDisposition::AtHead,
            kernel::CursorDisposition::Truncated => RemoteAttemptLogCursorDisposition::Truncated,
        };
    let (effective_cursor, reason_code) = match disposition {
        RemoteAttemptLogCursorDisposition::Retained => (requested_cursor, RemoteAttemptLogReasonCode::CursorRetained),
        RemoteAttemptLogCursorDisposition::AtHead => (requested_cursor, RemoteAttemptLogReasonCode::CursorAtHead),
        RemoteAttemptLogCursorDisposition::Truncated => {
            (manifest.retained_start_cursor, RemoteAttemptLogReasonCode::CursorTruncated)
        }
    };
    debug_assert!(effective_cursor <= manifest.next_cursor);
    debug_assert!(effective_cursor >= manifest.retained_start_cursor);
    Ok(RemoteAttemptLogCursorDecision {
        disposition,
        requested_cursor,
        effective_cursor,
        reason_code,
    })
}

pub fn plan_remote_attempt_log_replay(
    manifest: &RemoteAttemptLogManifest,
    segments: &[RemoteAttemptLogSegment],
    request: RemoteAttemptLogReplayRequest,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogReplayPlan, RemoteAttemptLogReasonCode> {
    let chain = validate_remote_attempt_log_chain(manifest, segments, policy)?;
    validate_replay_request(request, policy)?;
    let cursor = decide_remote_attempt_log_cursor(manifest, request.from_cursor, policy)?;
    debug_assert_eq!(chain.next_cursor, manifest.next_cursor);
    debug_assert!(request.record_count_max <= policy.replay_record_count_max);
    if cursor.disposition == RemoteAttemptLogCursorDisposition::Truncated {
        return Ok(RemoteAttemptLogReplayPlan {
            next_cursor: cursor.effective_cursor,
            cursor,
            records: Vec::new(),
            has_more: kernel::replay_has_more(manifest.retained_start_cursor, manifest.next_cursor),
            truncation_anchor: manifest.truncation_anchor.clone(),
        });
    }
    if cursor.disposition == RemoteAttemptLogCursorDisposition::AtHead {
        return Ok(RemoteAttemptLogReplayPlan {
            next_cursor: manifest.next_cursor,
            cursor,
            records: Vec::new(),
            has_more: false,
            truncation_anchor: None,
        });
    }
    collect_replay_records(manifest, segments, request, cursor)
}

pub fn plan_remote_attempt_log_retention(
    manifest: &RemoteAttemptLogManifest,
    segments: &[RemoteAttemptLogSegment],
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogRetentionPlan, RemoteAttemptLogReasonCode> {
    validate_remote_attempt_log_chain(manifest, segments, policy)?;
    let drop_count = retention_drop_count(manifest, policy)?;
    if kernel::classify_retention(drop_count, segments.len()).map_err(RemoteAttemptLogReasonCode::from)?
        == kernel::RetentionDecision::Unchanged
    {
        return Ok(RemoteAttemptLogRetentionPlan {
            disposition: RemoteAttemptLogRetentionDisposition::Unchanged,
            reason_code: RemoteAttemptLogReasonCode::RetentionUnchanged,
            anchor: None,
            delete_segment_blake3: Vec::new(),
            retained_segments: segments.to_vec(),
            next_manifest: manifest.clone(),
        });
    }
    build_retention_plan(manifest, segments, drop_count, policy)
}

fn validate_scope(scope: &RemoteAttemptLogScope) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::validate_scope_identity(borrowed_scope(scope)).map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_event_id(event_id: &RemoteEventId) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::validate_identity(event_id.as_str()).map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_current_scope(
    expected: &RemoteAttemptLogScope,
    observed: &RemoteAttemptLogScope,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_scope(borrowed_scope(expected), borrowed_scope(observed)).map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_record_payload_metadata(
    record: &RemoteAttemptLogRecord,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_record_payload_metadata(
        record.payload.len(),
        record.payload_length_bytes,
        policy.record_payload_bytes_max,
        || record.payload_blake3 == content_hash(&record.payload),
        record.flags.secret_redacted,
        || record.payload == REDACTED_PAYLOAD,
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

#[derive(Serialize)]
struct RemoteAttemptLogRecordPayload<'a> {
    schema: &'a str,
    scope: &'a RemoteAttemptLogScope,
    event_id: &'a RemoteEventId,
    sequence: u64,
    cursor: u64,
    phase: RemoteAttemptPhase,
    stream: RemoteAttemptLogStream,
    kind: RemoteAttemptLogRecordKind,
    payload_length_bytes: u32,
    payload_blake3: &'a RemoteAttemptLogDigest,
    previous_record_blake3: &'a Option<RemoteAttemptLogDigest>,
    flags: RemoteAttemptLogRecordFlags,
    payload: &'a [u8],
}

fn record_payload_digest(
    record: &RemoteAttemptLogRecord,
) -> Result<RemoteAttemptLogDigest, RemoteAttemptLogReasonCode> {
    let payload = RemoteAttemptLogRecordPayload {
        schema: &record.schema,
        scope: &record.scope,
        event_id: &record.event_id,
        sequence: record.sequence,
        cursor: record.cursor,
        phase: record.phase,
        stream: record.stream,
        kind: record.kind,
        payload_length_bytes: record.payload_length_bytes,
        payload_blake3: &record.payload_blake3,
        previous_record_blake3: &record.previous_record_blake3,
        flags: record.flags,
        payload: &record.payload,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|_| RemoteAttemptLogReasonCode::SerializationFailed)?;
    domain_hash(RECORD_DIGEST_DOMAIN, &bytes)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SegmentRecordSummary {
    scope: RemoteAttemptLogScope,
    first_sequence: u64,
    last_sequence: u64,
    first_cursor: u64,
    next_cursor: u64,
    record_count: u32,
    payload_bytes: u64,
}

fn summarize_segment_records(
    records: &[RemoteAttemptLogRecord],
    policy: RemoteAttemptLogPolicy,
) -> Result<SegmentRecordSummary, RemoteAttemptLogReasonCode> {
    let first = records.first().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    let last = records.last().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    let record_count = kernel::check_segment_record_count(records.len(), policy.segment_record_count_max)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    let mut summary = kernel::SegmentAccumulator {
        scope: borrowed_scope(&first.scope),
        expected_sequence: first.sequence,
        expected_cursor: first.cursor,
        previous_record: first.previous_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
        payload_bytes: 0,
    };
    for record in records {
        validate_remote_attempt_log_record(record, policy)?;
        summary
            .accept(
                borrowed_scope(&record.scope),
                record.sequence,
                record.cursor,
                record.previous_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
                record.record_blake3.as_str(),
                record.payload_length_bytes,
                policy.segment_payload_bytes_max,
            )
            .map_err(RemoteAttemptLogReasonCode::from)?;
    }
    Ok(SegmentRecordSummary {
        scope: first.scope.clone(),
        first_sequence: first.sequence,
        last_sequence: last.sequence,
        first_cursor: first.cursor,
        next_cursor: summary.expected_cursor,
        record_count,
        payload_bytes: summary.payload_bytes,
    })
}

fn validate_segment_summary(
    segment: &RemoteAttemptLogSegment,
    summary: &SegmentRecordSummary,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_segment_summary(
        kernel::SegmentFacts {
            scope: borrowed_scope(&summary.scope),
            first_sequence: summary.first_sequence,
            last_sequence: summary.last_sequence,
            first_cursor: summary.first_cursor,
            next_cursor: summary.next_cursor,
            record_count: summary.record_count,
            payload_bytes: summary.payload_bytes,
        },
        kernel::SegmentFacts {
            scope: borrowed_scope(&segment.scope),
            first_sequence: segment.first_sequence,
            last_sequence: segment.last_sequence,
            first_cursor: segment.first_cursor,
            next_cursor: segment.next_cursor,
            record_count: segment.record_count,
            payload_bytes: segment.payload_bytes,
        },
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

#[derive(Serialize)]
struct RemoteAttemptLogSegmentPayload<'a> {
    schema: &'a str,
    scope: &'a RemoteAttemptLogScope,
    segment_index: u64,
    previous_segment_blake3: &'a Option<RemoteAttemptLogDigest>,
    first_sequence: u64,
    last_sequence: u64,
    first_cursor: u64,
    next_cursor: u64,
    record_count: u32,
    payload_bytes: u64,
    records: &'a [RemoteAttemptLogRecord],
}

fn segment_payload_digest(
    segment: &RemoteAttemptLogSegment,
) -> Result<RemoteAttemptLogDigest, RemoteAttemptLogReasonCode> {
    let payload = RemoteAttemptLogSegmentPayload {
        schema: &segment.schema,
        scope: &segment.scope,
        segment_index: segment.segment_index,
        previous_segment_blake3: &segment.previous_segment_blake3,
        first_sequence: segment.first_sequence,
        last_sequence: segment.last_sequence,
        first_cursor: segment.first_cursor,
        next_cursor: segment.next_cursor,
        record_count: segment.record_count,
        payload_bytes: segment.payload_bytes,
        records: &segment.records,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|_| RemoteAttemptLogReasonCode::SerializationFailed)?;
    domain_hash(SEGMENT_DIGEST_DOMAIN, &bytes)
}

fn segment_ref(segment: &RemoteAttemptLogSegment) -> Result<RemoteAttemptLogSegmentRef, RemoteAttemptLogReasonCode> {
    let first = segment.records.first().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    let last = segment.records.last().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    debug_assert!(!segment.records.is_empty());
    debug_assert!(segment.record_count > 0);
    Ok(RemoteAttemptLogSegmentRef {
        segment_index: segment.segment_index,
        segment_blake3: segment.segment_blake3.clone(),
        previous_segment_blake3: segment.previous_segment_blake3.clone(),
        first_sequence: segment.first_sequence,
        last_sequence: segment.last_sequence,
        first_cursor: segment.first_cursor,
        next_cursor: segment.next_cursor,
        first_previous_record_blake3: first.previous_record_blake3.clone(),
        head_record_blake3: last.record_blake3.clone(),
        record_count: segment.record_count,
        payload_bytes: segment.payload_bytes,
    })
}

fn validate_manifest_shape(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    policy.validate()?;
    kernel::check_schema(&manifest.schema, REMOTE_ATTEMPT_LOG_MANIFEST_SCHEMA, kernel::ObjectKind::Manifest)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    validate_scope(&manifest.scope)?;
    validate_policy_identity(&manifest.policy, policy)?;
    validate_manifest_bounds(manifest, policy)?;
    validate_optional_digest(&manifest.head_record_blake3)?;
    validate_optional_digest(&manifest.head_segment_blake3)?;
    if let Some(anchor) = &manifest.truncation_anchor {
        validate_truncation_anchor(anchor, &manifest.scope, &manifest.policy)?;
    }
    validate_manifest_segment_refs(manifest)?;
    validate_manifest_events(manifest, policy)?;
    Ok(())
}

fn validate_policy_identity(
    identity: &RemoteAttemptLogPolicyIdentity,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    policy.validate()?;
    kernel::validate_policy_name(&identity.name).map_err(RemoteAttemptLogReasonCode::from)?;
    kernel::check_policy_identity(policy_digest(policy)?.as_str(), identity.digest_blake3.as_str())
        .map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_manifest_bounds(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_manifest_bounds(
        kernel::ManifestBounds {
            segment_count: manifest.segments.len(),
            event_count: manifest.event_records.len(),
            retained_records: manifest.retained_record_count,
            retained_bytes: manifest.retained_payload_bytes,
            retained_start: manifest.retained_start_cursor,
            next_cursor: manifest.next_cursor,
            head_record_present: manifest.head_record_blake3.is_some(),
            head_segment_present: manifest.head_segment_blake3.is_some(),
            anchor_present: manifest.truncation_anchor.is_some(),
        },
        kernel_policy(policy),
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_manifest_segment_refs(manifest: &RemoteAttemptLogManifest) -> Result<(), RemoteAttemptLogReasonCode> {
    let mut expected_cursor = manifest.retained_start_cursor;
    let mut expected_index = manifest.segments.first().map(|item| item.segment_index);
    let mut previous_segment = manifest.truncation_anchor.as_ref().map(|anchor| &anchor.dropped_tail_segment_blake3);
    let mut previous_record = manifest.truncation_anchor.as_ref().map(|anchor| &anchor.dropped_tail_record_blake3);
    let mut record_count = 0_u32;
    let mut payload_bytes = 0_u64;
    for item in &manifest.segments {
        validate_segment_ref(item)?;
        kernel::check_ref_link(
            kernel::RefLink {
                index: expected_index,
                first_cursor: expected_cursor,
                first_sequence: expected_cursor,
                previous_record: previous_record.map(RemoteAttemptLogDigest::as_str),
                previous_segment: previous_segment.map(RemoteAttemptLogDigest::as_str),
            },
            kernel::RefLink {
                index: Some(item.segment_index),
                first_cursor: item.first_cursor,
                first_sequence: item.first_sequence,
                previous_record: item.first_previous_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
                previous_segment: item.previous_segment_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            },
        )
        .map_err(RemoteAttemptLogReasonCode::from)?;
        record_count =
            kernel::checked_add_u32(record_count, item.record_count).map_err(RemoteAttemptLogReasonCode::from)?;
        payload_bytes =
            kernel::checked_add_u64(payload_bytes, item.payload_bytes).map_err(RemoteAttemptLogReasonCode::from)?;
        expected_cursor = item.next_cursor;
        expected_index =
            Some(kernel::next_segment_index(Some(item.segment_index)).map_err(RemoteAttemptLogReasonCode::from)?);
        previous_segment = Some(&item.segment_blake3);
        previous_record = Some(&item.head_record_blake3);
    }
    kernel::check_manifest_ref_totals(
        kernel::ManifestTotals {
            next_cursor: manifest.next_cursor,
            segment_count: 0,
            record_count: manifest.retained_record_count,
            payload_bytes: manifest.retained_payload_bytes,
            head_record: manifest.head_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            head_segment: manifest.head_segment_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
        },
        kernel::ManifestTotals {
            next_cursor: expected_cursor,
            segment_count: 0,
            record_count,
            payload_bytes,
            head_record: previous_record.map(RemoteAttemptLogDigest::as_str),
            head_segment: previous_segment.map(RemoteAttemptLogDigest::as_str),
        },
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_segment_ref(item: &RemoteAttemptLogSegmentRef) -> Result<(), RemoteAttemptLogReasonCode> {
    validate_optional_digest(&item.previous_segment_blake3)?;
    validate_optional_digest(&item.first_previous_record_blake3)?;
    kernel::validate_digest(item.segment_blake3.as_str()).map_err(RemoteAttemptLogReasonCode::from)?;
    kernel::validate_digest(item.head_record_blake3.as_str()).map_err(RemoteAttemptLogReasonCode::from)?;
    kernel::check_segment_ref_shape(
        item.first_sequence,
        item.first_cursor,
        item.next_cursor,
        item.last_sequence,
        item.record_count,
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_manifest_events(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_manifest_event_count(manifest.event_records.len(), policy.event_identity_count_max)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    for (event_id, digest) in &manifest.event_records {
        validate_event_id(event_id)?;
        kernel::validate_digest(digest.as_str()).map_err(RemoteAttemptLogReasonCode::from)?;
    }
    Ok(())
}

fn seal_manifest(
    mut manifest: RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogManifest, RemoteAttemptLogReasonCode> {
    manifest.manifest_blake3 = zero_digest();
    validate_manifest_shape(&manifest, policy)?;
    manifest.manifest_blake3 = manifest_payload_digest(&manifest)?;
    validate_remote_attempt_log_manifest(&manifest, policy)?;
    Ok(manifest)
}

#[derive(Serialize)]
struct RemoteAttemptLogManifestPayload<'a> {
    schema: &'a str,
    scope: &'a RemoteAttemptLogScope,
    policy: &'a RemoteAttemptLogPolicyIdentity,
    retained_start_cursor: u64,
    next_cursor: u64,
    head_record_blake3: &'a Option<RemoteAttemptLogDigest>,
    head_segment_blake3: &'a Option<RemoteAttemptLogDigest>,
    retained_record_count: u32,
    retained_payload_bytes: u64,
    segments: &'a [RemoteAttemptLogSegmentRef],
    event_records: &'a BTreeMap<RemoteEventId, RemoteAttemptLogDigest>,
    truncation_anchor: &'a Option<RemoteAttemptLogTruncationAnchorRecord>,
}

fn manifest_payload_digest(
    manifest: &RemoteAttemptLogManifest,
) -> Result<RemoteAttemptLogDigest, RemoteAttemptLogReasonCode> {
    let payload = RemoteAttemptLogManifestPayload {
        schema: &manifest.schema,
        scope: &manifest.scope,
        policy: &manifest.policy,
        retained_start_cursor: manifest.retained_start_cursor,
        next_cursor: manifest.next_cursor,
        head_record_blake3: &manifest.head_record_blake3,
        head_segment_blake3: &manifest.head_segment_blake3,
        retained_record_count: manifest.retained_record_count,
        retained_payload_bytes: manifest.retained_payload_bytes,
        segments: &manifest.segments,
        event_records: &manifest.event_records,
        truncation_anchor: &manifest.truncation_anchor,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|_| RemoteAttemptLogReasonCode::SerializationFailed)?;
    domain_hash(MANIFEST_DIGEST_DOMAIN, &bytes)
}

fn classify_existing_record(
    manifest: &RemoteAttemptLogManifest,
    existing: &RemoteAttemptLogDigest,
    record: &RemoteAttemptLogRecord,
) -> Result<RemoteAttemptLogAppendPlan, RemoteAttemptLogReasonCode> {
    kernel::classify_existing_record(existing.as_str(), record.record_blake3.as_str())
        .map_err(RemoteAttemptLogReasonCode::from)?;
    Ok(RemoteAttemptLogAppendPlan {
        disposition: RemoteAttemptLogAppendDisposition::AlreadyApplied,
        reason_code: RemoteAttemptLogReasonCode::AlreadyApplied,
        segment: None,
        next_manifest: manifest.clone(),
    })
}

fn validate_append_position(
    manifest: &RemoteAttemptLogManifest,
    record: &RemoteAttemptLogRecord,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_append_position(
        manifest.next_cursor,
        manifest.head_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
        record.sequence,
        record.cursor,
        record.previous_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn next_segment_index(manifest: &RemoteAttemptLogManifest) -> Result<u64, RemoteAttemptLogReasonCode> {
    kernel::next_segment_index(manifest.segments.last().map(|previous| previous.segment_index))
        .map_err(RemoteAttemptLogReasonCode::from)
}

fn append_segment_to_manifest(
    manifest: &RemoteAttemptLogManifest,
    segment: &RemoteAttemptLogSegment,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogManifest, RemoteAttemptLogReasonCode> {
    let mut next = manifest.clone();
    let head_record = segment.records.last().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    next.segments.push(segment_ref(segment)?);
    next.next_cursor = segment.next_cursor;
    next.head_record_blake3 = Some(head_record.record_blake3.clone());
    next.head_segment_blake3 = Some(segment.segment_blake3.clone());
    next.retained_record_count = kernel::checked_add_u32(next.retained_record_count, segment.record_count)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    next.retained_payload_bytes = kernel::checked_add_u64(next.retained_payload_bytes, segment.payload_bytes)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    for record in &segment.records {
        kernel::check_new_event_identity(
            next.event_records.insert(record.event_id.clone(), record.record_blake3.clone()).is_some(),
        )
        .map_err(RemoteAttemptLogReasonCode::from)?;
    }
    debug_assert_eq!(next.next_cursor, segment.next_cursor);
    debug_assert_eq!(next.head_record_blake3.as_ref(), Some(&head_record.record_blake3));
    seal_manifest(next, policy)
}

fn validate_segment_against_ref(
    segment: &RemoteAttemptLogSegment,
    expected: &RemoteAttemptLogSegmentRef,
) -> Result<(), RemoteAttemptLogReasonCode> {
    let first = segment.records.first().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    let last = segment.records.last().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    kernel::check_segment_ref(
        kernel::SegmentRefFacts {
            index: expected.segment_index,
            segment_digest: expected.segment_blake3.as_str(),
            previous_segment: expected.previous_segment_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            first_sequence: expected.first_sequence,
            last_sequence: expected.last_sequence,
            first_cursor: expected.first_cursor,
            next_cursor: expected.next_cursor,
            first_previous_record: expected.first_previous_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            head_record: expected.head_record_blake3.as_str(),
            record_count: expected.record_count,
            payload_bytes: expected.payload_bytes,
        },
        kernel::SegmentRefFacts {
            index: segment.segment_index,
            segment_digest: segment.segment_blake3.as_str(),
            previous_segment: segment.previous_segment_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            first_sequence: segment.first_sequence,
            last_sequence: segment.last_sequence,
            first_cursor: segment.first_cursor,
            next_cursor: segment.next_cursor,
            first_previous_record: first.previous_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            head_record: last.record_blake3.as_str(),
            record_count: segment.record_count,
            payload_bytes: segment.payload_bytes,
        },
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

#[derive(Debug, Clone, Copy)]
struct ChainLinkExpectation<'a> {
    scope: &'a RemoteAttemptLogScope,
    expected_sequence: u64,
    expected_cursor: u64,
    previous_record_blake3: Option<&'a RemoteAttemptLogDigest>,
    previous_segment_blake3: Option<&'a RemoteAttemptLogDigest>,
}

fn validate_chain_link(
    segment: &RemoteAttemptLogSegment,
    expected: ChainLinkExpectation<'_>,
) -> Result<(), RemoteAttemptLogReasonCode> {
    let first = segment.records.first().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    kernel::check_chain_link(
        kernel::ChainLink {
            scope: borrowed_scope(expected.scope),
            sequence: expected.expected_sequence,
            cursor: expected.expected_cursor,
            previous_record: expected.previous_record_blake3.map(RemoteAttemptLogDigest::as_str),
            previous_segment: expected.previous_segment_blake3.map(RemoteAttemptLogDigest::as_str),
        },
        kernel::ChainLink {
            scope: borrowed_scope(&segment.scope),
            sequence: segment.first_sequence,
            cursor: segment.first_cursor,
            previous_record: first.previous_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            previous_segment: segment.previous_segment_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
        },
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_retained_event_records(
    segment: &RemoteAttemptLogSegment,
    manifest: &RemoteAttemptLogManifest,
    retained_events: &mut BTreeSet<RemoteEventId>,
) -> Result<(), RemoteAttemptLogReasonCode> {
    for record in &segment.records {
        kernel::check_event_binding(retained_events.insert(record.event_id.clone()), || {
            manifest.event_records.get(&record.event_id) == Some(&record.record_blake3)
        })
        .map_err(RemoteAttemptLogReasonCode::from)?;
    }
    Ok(())
}

fn validate_chain_summary(
    manifest: &RemoteAttemptLogManifest,
    summary: &RemoteAttemptLogChainSummary,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_chain_totals(
        kernel::ManifestTotals {
            next_cursor: manifest.next_cursor,
            segment_count: usize_to_u32(manifest.segments.len())?,
            record_count: manifest.retained_record_count,
            payload_bytes: manifest.retained_payload_bytes,
            head_record: manifest.head_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            head_segment: manifest.head_segment_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
        },
        kernel::ManifestTotals {
            next_cursor: summary.next_cursor,
            segment_count: summary.segment_count,
            record_count: summary.record_count,
            payload_bytes: summary.payload_bytes,
            head_record: summary.head_record_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
            head_segment: summary.head_segment_blake3.as_ref().map(RemoteAttemptLogDigest::as_str),
        },
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn validate_replay_request(
    request: RemoteAttemptLogReplayRequest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::validate_replay_request(
        request.record_count_max,
        request.payload_bytes_max,
        policy.replay_record_count_max,
        policy.replay_payload_bytes_max,
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn collect_replay_records(
    manifest: &RemoteAttemptLogManifest,
    segments: &[RemoteAttemptLogSegment],
    request: RemoteAttemptLogReplayRequest,
    cursor: RemoteAttemptLogCursorDecision,
) -> Result<RemoteAttemptLogReplayPlan, RemoteAttemptLogReasonCode> {
    let record_count_max = u32_to_usize(request.record_count_max)?;
    let mut records = Vec::with_capacity(record_count_max);
    let mut payload_bytes = 0_u64;
    for record in segments.iter().flat_map(|segment| &segment.records) {
        if kernel::replay_record_is_before_cursor(record.cursor, cursor.effective_cursor) {
            continue;
        }
        let Some(projected) = kernel::admit_replay_record(
            records.len(),
            record_count_max,
            payload_bytes,
            record.payload_length_bytes,
            request.payload_bytes_max,
        )
        .map_err(RemoteAttemptLogReasonCode::from)?
        else {
            break;
        };
        payload_bytes = projected;
        records.push(record.clone());
    }
    let next_cursor = kernel::next_replay_cursor(records.last().map(|record| record.cursor))
        .map_err(RemoteAttemptLogReasonCode::from)?;
    debug_assert!(payload_bytes <= request.payload_bytes_max);
    debug_assert!(records.len() <= record_count_max);
    Ok(RemoteAttemptLogReplayPlan {
        cursor,
        records,
        next_cursor,
        has_more: kernel::replay_has_more(next_cursor, manifest.next_cursor),
        truncation_anchor: None,
    })
}

fn retention_drop_count(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<usize, RemoteAttemptLogReasonCode> {
    let usage = kernel::RetentionUsage {
        segment_count: usize_to_u32(manifest.segments.len())?,
        record_count: manifest.retained_record_count,
        payload_bytes: manifest.retained_payload_bytes,
    };
    let limits = kernel::RetentionUsage {
        segment_count: policy.retained_segment_count_max,
        record_count: policy.retained_record_count_max,
        payload_bytes: policy.retained_payload_bytes_max,
    };
    kernel::retention_drop_count(
        usage,
        limits,
        manifest.segments.iter().map(|item| kernel::RetentionUsage {
            segment_count: 1,
            record_count: item.record_count,
            payload_bytes: item.payload_bytes,
        }),
    )
    .map_err(RemoteAttemptLogReasonCode::from)
}

fn build_retention_plan(
    manifest: &RemoteAttemptLogManifest,
    segments: &[RemoteAttemptLogSegment],
    drop_count: usize,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogRetentionPlan, RemoteAttemptLogReasonCode> {
    let dropped = &segments[..drop_count];
    let retained = &segments[drop_count..];
    let dropped_summary = summarize_dropped_segments(dropped)?;
    let retained_start = retained.first().ok_or(RemoteAttemptLogReasonCode::RetentionWouldDropAll)?.first_cursor;
    let anchor = seal_truncation_anchor(manifest, retained_start, dropped_summary.clone(), policy)?;
    let mut next = manifest.clone();
    next.segments.drain(..drop_count);
    next.retained_start_cursor = retained_start;
    next.retained_record_count = kernel::checked_sub_u32(next.retained_record_count, dropped_summary.record_count)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    next.retained_payload_bytes = kernel::checked_sub_u64(next.retained_payload_bytes, dropped_summary.payload_bytes)
        .map_err(RemoteAttemptLogReasonCode::from)?;
    next.truncation_anchor = Some(anchor.clone());
    let next_manifest = seal_manifest(next, policy)?;
    validate_remote_attempt_log_chain(&next_manifest, retained, policy)?;
    let delete_segment_blake3 = dropped.iter().map(|segment| segment.segment_blake3.clone()).collect();
    debug_assert_eq!(next_manifest.retained_start_cursor, retained_start);
    debug_assert_eq!(next_manifest.next_cursor, manifest.next_cursor);
    Ok(RemoteAttemptLogRetentionPlan {
        disposition: RemoteAttemptLogRetentionDisposition::Truncate,
        reason_code: RemoteAttemptLogReasonCode::RetentionPlanned,
        anchor: Some(anchor),
        delete_segment_blake3,
        retained_segments: retained.to_vec(),
        next_manifest,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DroppedSegmentSummary {
    start_cursor: u64,
    end_cursor: u64,
    chunk_count: u32,
    record_count: u32,
    payload_bytes: u64,
    tail_record_blake3: RemoteAttemptLogDigest,
    tail_segment_blake3: RemoteAttemptLogDigest,
}

fn summarize_dropped_segments(
    dropped: &[RemoteAttemptLogSegment],
) -> Result<DroppedSegmentSummary, RemoteAttemptLogReasonCode> {
    let first = dropped.first().ok_or(RemoteAttemptLogReasonCode::RetentionAnchorInvalid)?;
    let last = dropped.last().ok_or(RemoteAttemptLogReasonCode::RetentionAnchorInvalid)?;
    let (chunk_count, record_count, payload_bytes) = kernel::dropped_usage(
        dropped.iter().map(|segment| (segment.record_count, segment.payload_bytes)),
        dropped.len(),
    )
    .map_err(RemoteAttemptLogReasonCode::from)?;
    let tail_record = last.records.last().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
    debug_assert!(!dropped.is_empty());
    debug_assert!(first.first_cursor < last.next_cursor);
    Ok(DroppedSegmentSummary {
        start_cursor: first.first_cursor,
        end_cursor: last.next_cursor,
        chunk_count,
        record_count,
        payload_bytes,
        tail_record_blake3: tail_record.record_blake3.clone(),
        tail_segment_blake3: last.segment_blake3.clone(),
    })
}

fn seal_truncation_anchor(
    manifest: &RemoteAttemptLogManifest,
    retained_start: u64,
    dropped: DroppedSegmentSummary,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogTruncationAnchorRecord, RemoteAttemptLogReasonCode> {
    let prior_head_record_blake3 =
        manifest.head_record_blake3.clone().ok_or(RemoteAttemptLogReasonCode::RetentionAnchorInvalid)?;
    let prior_head_segment_blake3 =
        manifest.head_segment_blake3.clone().ok_or(RemoteAttemptLogReasonCode::RetentionAnchorInvalid)?;
    let mut anchor = RemoteAttemptLogTruncationAnchorRecord {
        schema: REMOTE_ATTEMPT_LOG_TRUNCATION_ANCHOR_SCHEMA.to_string(),
        scope: manifest.scope.clone(),
        dropped_start_cursor: dropped.start_cursor,
        dropped_end_cursor_exclusive: dropped.end_cursor,
        new_retained_start_cursor: retained_start,
        dropped_chunk_count: dropped.chunk_count,
        dropped_record_count: dropped.record_count,
        dropped_payload_bytes: dropped.payload_bytes,
        dropped_tail_record_blake3: dropped.tail_record_blake3,
        dropped_tail_segment_blake3: dropped.tail_segment_blake3,
        prior_head_record_blake3,
        prior_head_segment_blake3,
        policy: canonical_remote_attempt_log_policy_identity(manifest.policy.name.clone(), policy)?,
        previous_anchor_blake3: manifest.truncation_anchor.as_ref().map(|item| item.anchor_blake3.clone()),
        anchor_blake3: zero_digest(),
    };
    anchor.anchor_blake3 = truncation_anchor_payload_digest(&anchor)?;
    validate_truncation_anchor(&anchor, &manifest.scope, &manifest.policy)?;
    debug_assert_eq!(anchor.new_retained_start_cursor, retained_start);
    debug_assert!(anchor.dropped_chunk_count > 0);
    Ok(anchor)
}

fn validate_truncation_anchor(
    anchor: &RemoteAttemptLogTruncationAnchorRecord,
    scope: &RemoteAttemptLogScope,
    policy_identity: &RemoteAttemptLogPolicyIdentity,
) -> Result<(), RemoteAttemptLogReasonCode> {
    kernel::check_anchor(
        kernel::AnchorFacts {
            schema_matches: anchor.schema == REMOTE_ATTEMPT_LOG_TRUNCATION_ANCHOR_SCHEMA,
            dropped_start: anchor.dropped_start_cursor,
            dropped_end: anchor.dropped_end_cursor_exclusive,
            retained_start: anchor.new_retained_start_cursor,
            dropped_chunks: anchor.dropped_chunk_count,
            dropped_records: anchor.dropped_record_count,
        },
        || &anchor.scope == scope,
        || &anchor.policy == policy_identity,
    )
    .map_err(RemoteAttemptLogReasonCode::from)?;
    validate_optional_digest(&anchor.previous_anchor_blake3)?;
    for digest in [
        &anchor.dropped_tail_record_blake3,
        &anchor.dropped_tail_segment_blake3,
        &anchor.prior_head_record_blake3,
        &anchor.prior_head_segment_blake3,
    ] {
        kernel::validate_digest(digest.as_str()).map_err(RemoteAttemptLogReasonCode::from)?;
    }
    kernel::check_anchor_digest(anchor.anchor_blake3.as_str(), truncation_anchor_payload_digest(anchor)?.as_str())
        .map_err(RemoteAttemptLogReasonCode::from)?;
    debug_assert_eq!(&anchor.scope, scope);
    debug_assert_eq!(&anchor.policy, policy_identity);
    Ok(())
}

#[derive(Serialize)]
struct RemoteAttemptLogTruncationAnchorPayload<'a> {
    schema: &'a str,
    scope: &'a RemoteAttemptLogScope,
    dropped_start_cursor: u64,
    dropped_end_cursor_exclusive: u64,
    new_retained_start_cursor: u64,
    dropped_chunk_count: u32,
    dropped_record_count: u32,
    dropped_payload_bytes: u64,
    dropped_tail_record_blake3: &'a RemoteAttemptLogDigest,
    dropped_tail_segment_blake3: &'a RemoteAttemptLogDigest,
    prior_head_record_blake3: &'a RemoteAttemptLogDigest,
    prior_head_segment_blake3: &'a RemoteAttemptLogDigest,
    policy: &'a RemoteAttemptLogPolicyIdentity,
    previous_anchor_blake3: &'a Option<RemoteAttemptLogDigest>,
}

fn truncation_anchor_payload_digest(
    anchor: &RemoteAttemptLogTruncationAnchorRecord,
) -> Result<RemoteAttemptLogDigest, RemoteAttemptLogReasonCode> {
    let payload = RemoteAttemptLogTruncationAnchorPayload {
        schema: &anchor.schema,
        scope: &anchor.scope,
        dropped_start_cursor: anchor.dropped_start_cursor,
        dropped_end_cursor_exclusive: anchor.dropped_end_cursor_exclusive,
        new_retained_start_cursor: anchor.new_retained_start_cursor,
        dropped_chunk_count: anchor.dropped_chunk_count,
        dropped_record_count: anchor.dropped_record_count,
        dropped_payload_bytes: anchor.dropped_payload_bytes,
        dropped_tail_record_blake3: &anchor.dropped_tail_record_blake3,
        dropped_tail_segment_blake3: &anchor.dropped_tail_segment_blake3,
        prior_head_record_blake3: &anchor.prior_head_record_blake3,
        prior_head_segment_blake3: &anchor.prior_head_segment_blake3,
        policy: &anchor.policy,
        previous_anchor_blake3: &anchor.previous_anchor_blake3,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|_| RemoteAttemptLogReasonCode::SerializationFailed)?;
    let digest_blake3 = domain_hash(TRUNCATION_ANCHOR_DIGEST_DOMAIN, &bytes)?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(is_blake3_hex_digest(digest_blake3.as_str()));
    Ok(digest_blake3)
}

fn validate_optional_digest(digest: &Option<RemoteAttemptLogDigest>) -> Result<(), RemoteAttemptLogReasonCode> {
    if let Some(digest) = digest {
        kernel::validate_digest(digest.as_str()).map_err(RemoteAttemptLogReasonCode::from)?;
    }
    Ok(())
}

fn content_hash(bytes: &[u8]) -> RemoteAttemptLogDigest {
    RemoteAttemptLogDigest(blake3::hash(bytes).to_hex().to_string())
}

fn domain_hash(domain: &str, bytes: &[u8]) -> Result<RemoteAttemptLogDigest, RemoteAttemptLogReasonCode> {
    let mut hasher = blake3::Hasher::new();
    let domain_length_bytes =
        u64::try_from(domain.len()).map_err(|_| RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
    hasher.update(&domain_length_bytes.to_le_bytes());
    hasher.update(domain.as_bytes());
    hasher.update(bytes);
    let digest_blake3 = RemoteAttemptLogDigest(hasher.finalize().to_hex().to_string());
    debug_assert!(!domain.is_empty());
    debug_assert!(is_blake3_hex_digest(digest_blake3.as_str()));
    Ok(digest_blake3)
}

fn zero_digest() -> RemoteAttemptLogDigest {
    RemoteAttemptLogDigest("0".repeat(BLAKE3_HEX_LENGTH_CHARS))
}

fn is_blake3_hex_digest(value: &str) -> bool {
    kernel::digest_is_valid(value)
}

fn usize_to_u32(value: usize) -> Result<u32, RemoteAttemptLogReasonCode> {
    kernel::usize_to_u32(value).map_err(RemoteAttemptLogReasonCode::from)
}

fn u32_to_usize(value: u32) -> Result<usize, RemoteAttemptLogReasonCode> {
    kernel::u32_to_usize(value).map_err(RemoteAttemptLogReasonCode::from)
}

#[cfg(test)]
mod tests {
    // r[verify remote_builds.immutable_attempt_log_segments]
    // r[verify remote_builds.pure_log_cursor_kernel]
    use super::*;

    const TEST_FENCE_GENERATION: u64 = 7;
    const TEST_RECORD_COUNT: u64 = 4;
    const TEST_RETAINED_SEGMENTS: u32 = 2;
    const TEST_REPLAY_RECORDS: u32 = 2;
    const TEST_MULTI_RECORD_SEGMENT_COUNT: u32 = 2;
    const TEST_REPLAY_BYTES: u64 = 128;
    const TEST_SMALL_PAYLOAD_MAX: u32 = 16;
    const TEST_SECRET: &str = "Authorization: Bearer SHOULD_NOT_LEAK";

    fn scope() -> RemoteAttemptLogScope {
        RemoteAttemptLogScope {
            job_id: RemoteJobId::new("job-attempt-log").unwrap(),
            attempt_id: RemoteAttemptId::new("attempt-attempt-log").unwrap(),
            fence_generation: RemoteFenceGeneration::new(TEST_FENCE_GENERATION).unwrap(),
        }
    }

    fn policy() -> RemoteAttemptLogPolicy {
        RemoteAttemptLogPolicy::default()
    }

    fn retention_policy() -> RemoteAttemptLogPolicy {
        RemoteAttemptLogPolicy {
            segment_record_count_max: 1,
            retained_segment_count_max: TEST_RETAINED_SEGMENTS,
            retained_record_count_max: TEST_RETAINED_SEGMENTS,
            replay_record_count_max: TEST_RETAINED_SEGMENTS,
            ..policy()
        }
    }

    fn empty(policy: RemoteAttemptLogPolicy) -> RemoteAttemptLogManifest {
        empty_remote_attempt_log_manifest(scope(), DEFAULT_REMOTE_ATTEMPT_LOG_POLICY_NAME, policy).unwrap()
    }

    fn record(
        manifest: &RemoteAttemptLogManifest,
        event: &str,
        payload: &[u8],
        policy: RemoteAttemptLogPolicy,
    ) -> RemoteAttemptLogRecord {
        seal_remote_attempt_log_record(
            RemoteAttemptLogRecordInput {
                scope: manifest.scope.clone(),
                event_id: RemoteEventId::new(event).unwrap(),
                sequence: manifest.next_cursor,
                cursor: manifest.next_cursor,
                phase: RemoteAttemptPhase::Running,
                stream: RemoteAttemptLogStream::Stdout,
                kind: RemoteAttemptLogRecordKind::Output,
                payload: payload.to_vec(),
                previous_record_blake3: manifest.head_record_blake3.clone(),
            },
            policy,
        )
        .unwrap()
    }

    fn current(manifest: &RemoteAttemptLogManifest) -> RemoteAttemptLogCurrentAttemptFacts {
        RemoteAttemptLogCurrentAttemptFacts {
            scope: manifest.scope.clone(),
            phase: RemoteAttemptPhase::Running,
        }
    }

    fn append(
        manifest: &RemoteAttemptLogManifest,
        event: &str,
        payload: &[u8],
        policy: RemoteAttemptLogPolicy,
    ) -> (RemoteAttemptLogManifest, RemoteAttemptLogSegment, RemoteAttemptLogRecord) {
        let record = record(manifest, event, payload, policy);
        let plan = plan_remote_attempt_log_append(manifest, &current(manifest), &record, policy).unwrap();
        (plan.next_manifest, plan.segment.expect("accepted append has immutable segment"), record)
    }

    fn chain(
        record_count: u64,
        policy: RemoteAttemptLogPolicy,
    ) -> (RemoteAttemptLogManifest, Vec<RemoteAttemptLogSegment>) {
        let mut manifest = empty(policy);
        let mut segments = Vec::new();
        for index in 0..record_count {
            let payload = format!("payload-{index}");
            let (next, segment, _) = append(&manifest, &format!("event-{index}"), payload.as_bytes(), policy);
            manifest = next;
            segments.push(segment);
        }
        (manifest, segments)
    }

    #[test]
    fn changing_payload_changes_record_and_segment_identities() {
        let manifest = empty(policy());
        let original = record(&manifest, "event-canonical", b"same payload", policy());
        let changed = record(&manifest, "event-canonical", b"changed payload", policy());
        let original_segment =
            seal_remote_attempt_log_segment(vec![original.clone()], INITIAL_SEGMENT_INDEX, None, policy()).unwrap();
        let changed_segment =
            seal_remote_attempt_log_segment(vec![changed.clone()], INITIAL_SEGMENT_INDEX, None, policy()).unwrap();

        assert_ne!(original.record_blake3, changed.record_blake3);
        assert_ne!(original_segment.segment_blake3, changed_segment.segment_blake3);
    }

    #[test]
    fn bounded_segment_accepts_multiple_contiguous_records() {
        let manifest = empty(policy());
        let first = record(&manifest, "event-segment-first", b"first", policy());
        let second = seal_remote_attempt_log_record(
            RemoteAttemptLogRecordInput {
                scope: manifest.scope.clone(),
                event_id: RemoteEventId::new("event-segment-second").unwrap(),
                sequence: first.sequence + 1,
                cursor: first.cursor + 1,
                phase: RemoteAttemptPhase::Running,
                stream: RemoteAttemptLogStream::Stderr,
                kind: RemoteAttemptLogRecordKind::Diagnostic,
                payload: b"second".to_vec(),
                previous_record_blake3: Some(first.record_blake3.clone()),
            },
            policy(),
        )
        .unwrap();
        let segment =
            seal_remote_attempt_log_segment(vec![first.clone(), second.clone()], INITIAL_SEGMENT_INDEX, None, policy())
                .unwrap();

        assert_eq!(segment.record_count, TEST_MULTI_RECORD_SEGMENT_COUNT);
        assert_eq!(segment.first_sequence, first.sequence);
        assert_eq!(segment.last_sequence, second.sequence);
        assert_eq!(segment.next_cursor, second.cursor + 1);
    }

    #[test]
    fn append_is_idempotent_and_changed_payload_under_event_id_conflicts() {
        let manifest = empty(policy());
        let original = record(&manifest, "event-idempotent", b"first", policy());
        let first = plan_remote_attempt_log_append(&manifest, &current(&manifest), &original, policy()).unwrap();
        let duplicate =
            plan_remote_attempt_log_append(&first.next_manifest, &current(&manifest), &original, policy()).unwrap();
        let mut conflict = record(&manifest, "event-idempotent", b"second", policy());
        conflict.sequence = original.sequence;
        conflict.cursor = original.cursor;
        let error =
            plan_remote_attempt_log_append(&first.next_manifest, &current(&manifest), &conflict, policy()).unwrap_err();

        assert_eq!(first.disposition, RemoteAttemptLogAppendDisposition::Append);
        assert_eq!(duplicate.disposition, RemoteAttemptLogAppendDisposition::AlreadyApplied);
        assert_eq!(duplicate.next_manifest, first.next_manifest);
        assert_eq!(error, RemoteAttemptLogReasonCode::EventDigestConflict);
    }

    #[test]
    fn stale_future_and_wrong_attempt_scopes_fail_before_manifest_change() {
        let manifest = empty(policy());
        let current = current(&manifest);
        let candidate = record(&manifest, "event-scope", b"scope", policy());
        let mut stale = current.clone();
        stale.scope.fence_generation = RemoteFenceGeneration::new(TEST_FENCE_GENERATION - 1).unwrap();
        let mut future = current.clone();
        future.scope.fence_generation = current.scope.fence_generation.advance().unwrap();
        let mut wrong_attempt = current.clone();
        wrong_attempt.scope.attempt_id = RemoteAttemptId::new("different-attempt").unwrap();
        let mut wrong_phase = current.clone();
        wrong_phase.phase = RemoteAttemptPhase::Transferring;

        assert_eq!(
            plan_remote_attempt_log_append(&manifest, &stale, &candidate, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::StaleFenceRejected
        );
        assert_eq!(
            plan_remote_attempt_log_append(&manifest, &future, &candidate, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::UnknownFenceRejected
        );
        assert_eq!(
            plan_remote_attempt_log_append(&manifest, &wrong_attempt, &candidate, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::AttemptIdentityMismatch
        );
        assert_eq!(
            plan_remote_attempt_log_append(&manifest, &wrong_phase, &candidate, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::AttemptPhaseMismatch
        );
        assert_eq!(manifest.next_cursor, INITIAL_LOG_POSITION);
    }

    #[test]
    fn chain_validation_rejects_payload_previous_digest_and_segment_tamper() {
        let (manifest, segments) = chain(TEST_RECORD_COUNT, policy());
        let summary = validate_remote_attempt_log_chain(&manifest, &segments, policy()).unwrap();
        let mut payload_tamper = segments.clone();
        payload_tamper[0].records[0].payload = b"tampered".to_vec();
        let mut previous_tamper = segments.clone();
        previous_tamper[1].records[0].previous_record_blake3 = None;
        let mut segment_tamper = segments.clone();
        segment_tamper[0].segment_index = segment_tamper[0].segment_index.checked_add(1).unwrap();

        assert_eq!(summary.record_count, u32::try_from(TEST_RECORD_COUNT).unwrap());
        assert_eq!(summary.next_cursor, TEST_RECORD_COUNT);
        assert_eq!(
            validate_remote_attempt_log_chain(&manifest, &payload_tamper, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::RecordPayloadMetadataMismatch
        );
        assert_eq!(
            validate_remote_attempt_log_chain(&manifest, &previous_tamper, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::RecordDigestMismatch
        );
        assert_eq!(
            validate_remote_attempt_log_chain(&manifest, &segment_tamper, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::SegmentDigestMismatch
        );
    }

    #[test]
    fn replay_is_bounded_and_rejects_after_head_or_too_small_limit() {
        let (manifest, segments) = chain(TEST_RECORD_COUNT, policy());
        let replay = plan_remote_attempt_log_replay(
            &manifest,
            &segments,
            RemoteAttemptLogReplayRequest {
                from_cursor: INITIAL_LOG_POSITION,
                record_count_max: TEST_REPLAY_RECORDS,
                payload_bytes_max: TEST_REPLAY_BYTES,
            },
            policy(),
        )
        .unwrap();
        let after_head = decide_remote_attempt_log_cursor(&manifest, manifest.next_cursor + 1, policy()).unwrap_err();
        let too_small = plan_remote_attempt_log_replay(
            &manifest,
            &segments,
            RemoteAttemptLogReplayRequest {
                from_cursor: INITIAL_LOG_POSITION,
                record_count_max: TEST_REPLAY_RECORDS,
                payload_bytes_max: 1,
            },
            policy(),
        )
        .unwrap_err();

        assert_eq!(replay.records.len(), usize::try_from(TEST_REPLAY_RECORDS).unwrap());
        assert_eq!(replay.next_cursor, u64::from(TEST_REPLAY_RECORDS));
        assert!(replay.has_more);
        assert_eq!(after_head, RemoteAttemptLogReasonCode::CursorAfterHead);
        assert_eq!(too_small, RemoteAttemptLogReasonCode::ReplayLimitTooSmall);
    }

    #[test]
    fn cursor_decision_table_distinguishes_retained_head_and_after_head() {
        let (manifest, _) = chain(TEST_RECORD_COUNT, policy());
        let cases = [
            (INITIAL_LOG_POSITION, Ok(RemoteAttemptLogCursorDisposition::Retained)),
            (manifest.next_cursor - 1, Ok(RemoteAttemptLogCursorDisposition::Retained)),
            (manifest.next_cursor, Ok(RemoteAttemptLogCursorDisposition::AtHead)),
            (manifest.next_cursor + 1, Err(RemoteAttemptLogReasonCode::CursorAfterHead)),
        ];

        for (requested, expected) in cases {
            let observed =
                decide_remote_attempt_log_cursor(&manifest, requested, policy()).map(|decision| decision.disposition);
            assert_eq!(observed, expected, "requested_cursor={requested}");
            assert_eq!(observed.is_ok(), expected.is_ok());
        }
    }

    #[test]
    fn retention_creates_verifiable_anchor_and_distinguishes_truncated_cursor() {
        let policy = retention_policy();
        let (manifest, segments) = chain(TEST_RECORD_COUNT, policy);
        let plan = plan_remote_attempt_log_retention(&manifest, &segments, policy).unwrap();
        let anchor = plan.anchor.as_ref().expect("retention plan has anchor");
        let replay = plan_remote_attempt_log_replay(
            &plan.next_manifest,
            &plan.retained_segments,
            RemoteAttemptLogReplayRequest {
                from_cursor: INITIAL_LOG_POSITION,
                record_count_max: TEST_REPLAY_RECORDS,
                payload_bytes_max: TEST_REPLAY_BYTES,
            },
            policy,
        )
        .unwrap();

        assert_eq!(plan.disposition, RemoteAttemptLogRetentionDisposition::Truncate);
        assert_eq!(anchor.dropped_chunk_count, u32::try_from(TEST_RECORD_COUNT).unwrap() - TEST_RETAINED_SEGMENTS);
        assert_eq!(anchor.new_retained_start_cursor, TEST_RECORD_COUNT - u64::from(TEST_RETAINED_SEGMENTS));
        assert_eq!(replay.cursor.disposition, RemoteAttemptLogCursorDisposition::Truncated);
        assert!(replay.records.is_empty());
        assert_eq!(replay.truncation_anchor.as_ref(), plan.anchor.as_ref());
    }

    #[test]
    fn tampered_truncation_anchor_fails_closed() {
        let policy = retention_policy();
        let (manifest, segments) = chain(TEST_RECORD_COUNT, policy);
        let plan = plan_remote_attempt_log_retention(&manifest, &segments, policy).unwrap();
        let mut tampered = plan.next_manifest.clone();
        tampered.truncation_anchor.as_mut().unwrap().dropped_payload_bytes =
            tampered.truncation_anchor.as_ref().unwrap().dropped_payload_bytes.checked_add(1).unwrap();

        assert_eq!(
            validate_remote_attempt_log_manifest(&tampered, policy).unwrap_err(),
            RemoteAttemptLogReasonCode::RetentionAnchorInvalid
        );
        assert_eq!(plan.next_manifest.next_cursor, manifest.next_cursor);
    }

    #[test]
    fn redaction_escapes_control_data_redacts_secrets_and_truncates_deterministically() {
        let secret = redact_remote_attempt_log_payload(TEST_SECRET.as_bytes(), policy()).unwrap();
        let control = redact_remote_attempt_log_payload(b"frame\x1b[2J", policy()).unwrap();
        let small_policy = RemoteAttemptLogPolicy {
            record_payload_bytes_max: TEST_SMALL_PAYLOAD_MAX,
            ..policy()
        };
        let oversized = redact_remote_attempt_log_payload(b"abcdefghijklmnopqrstuvwxyz", small_policy).unwrap();
        let repeated = redact_remote_attempt_log_payload(b"abcdefghijklmnopqrstuvwxyz", small_policy).unwrap();

        assert_eq!(secret.payload, REDACTED_PAYLOAD);
        assert!(secret.flags.secret_redacted);
        assert_eq!(control.payload, b"frame\\x1b[2J");
        assert!(control.flags.control_escaped);
        assert_eq!(oversized, repeated);
        assert!(oversized.flags.payload_truncated);
        assert_eq!(oversized.payload.len(), usize::try_from(TEST_SMALL_PAYLOAD_MAX).unwrap());
    }

    #[test]
    fn malformed_policy_position_and_overflow_arithmetic_fail_closed() {
        let malformed = RemoteAttemptLogPolicy {
            record_payload_bytes_max: MIN_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES - 1,
            ..policy()
        };
        let manifest = empty(policy());
        let forged_position = seal_remote_attempt_log_record(
            RemoteAttemptLogRecordInput {
                scope: manifest.scope.clone(),
                event_id: RemoteEventId::new("event-forged-position").unwrap(),
                sequence: 1,
                cursor: 1,
                phase: RemoteAttemptPhase::Running,
                stream: RemoteAttemptLogStream::Event,
                kind: RemoteAttemptLogRecordKind::Diagnostic,
                payload: b"position".to_vec(),
                previous_record_blake3: None,
            },
            policy(),
        )
        .unwrap();
        let overflow_record = seal_remote_attempt_log_record(
            RemoteAttemptLogRecordInput {
                scope: manifest.scope.clone(),
                event_id: RemoteEventId::new("event-overflow-position").unwrap(),
                sequence: u64::MAX,
                cursor: u64::MAX,
                phase: RemoteAttemptPhase::Running,
                stream: RemoteAttemptLogStream::Event,
                kind: RemoteAttemptLogRecordKind::Diagnostic,
                payload: b"overflow".to_vec(),
                previous_record_blake3: None,
            },
            policy(),
        )
        .unwrap();

        assert_eq!(malformed.validate(), Err(RemoteAttemptLogReasonCode::PolicyInvalid));
        assert_eq!(
            plan_remote_attempt_log_append(&manifest, &current(&manifest), &forged_position, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::RecordPositionMismatch
        );
        assert_eq!(
            seal_remote_attempt_log_segment(vec![overflow_record], INITIAL_SEGMENT_INDEX, None, policy()).unwrap_err(),
            RemoteAttemptLogReasonCode::ArithmeticOverflow
        );
        assert_eq!(manifest.next_cursor, INITIAL_LOG_POSITION);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn checked_accounting_never_wraps() {
        let left: u64 = kani::any();
        let right: u64 = kani::any();
        match kernel::checked_add_u64(left, right) {
            Ok(sum) => {
                assert!(sum >= left);
                assert!(sum >= right);
            }
            Err(reason) => assert_eq!(reason, kernel::AttemptLogError::ArithmeticOverflow),
        }
    }

    #[kani::proof]
    fn cursor_classification_respects_retained_and_head_bounds() {
        let retained_start_cursor: u64 = kani::any();
        let next_cursor: u64 = kani::any();
        let requested_cursor: u64 = kani::any();
        kani::assume(retained_start_cursor <= next_cursor);
        let decision = classify_cursor_position(RemoteAttemptLogCursorBounds {
            retained_start_cursor,
            next_cursor,
            requested_cursor,
        });
        if requested_cursor > next_cursor {
            assert_eq!(decision, Err(RemoteAttemptLogReasonCode::CursorAfterHead));
        } else if requested_cursor < retained_start_cursor {
            assert_eq!(decision, Ok(RemoteAttemptLogCursorDisposition::Truncated));
        } else if requested_cursor == next_cursor {
            assert_eq!(decision, Ok(RemoteAttemptLogCursorDisposition::AtHead));
        } else {
            assert_eq!(decision, Ok(RemoteAttemptLogCursorDisposition::Retained));
        }
    }
}
