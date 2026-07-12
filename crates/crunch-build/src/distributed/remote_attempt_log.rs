//! Pure immutable remote-attempt log identities and bounded planning.
//!
//! This functional core performs no filesystem access, transport, clock reads,
//! deletion, rendering, coordinator mutation, or output admission. Shell code
//! may apply accepted append and retention plans only after persisting immutable
//! objects and atomically advancing the manifest.
//!
//! r[impl remote_builds.immutable_attempt_log_segments]
//! r[impl remote_builds.pure_log_cursor_kernel]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use super::remote_attempt::RemoteAttemptId;
use super::remote_attempt::RemoteAttemptPhase;
use super::remote_attempt::RemoteEventId;
use super::remote_attempt::RemoteFenceGeneration;
use super::remote_attempt::RemoteJobId;

pub const REMOTE_ATTEMPT_LOG_RECORD_SCHEMA: &str = "mantle-remote-attempt-log-record-v1";
pub const REMOTE_ATTEMPT_LOG_SEGMENT_SCHEMA: &str = "mantle-remote-attempt-log-segment-v1";
pub const REMOTE_ATTEMPT_LOG_MANIFEST_SCHEMA: &str = "mantle-remote-attempt-log-manifest-v1";
pub const REMOTE_ATTEMPT_LOG_TRUNCATION_ANCHOR_SCHEMA: &str = "mantle-remote-attempt-log-truncation-anchor-v1";

pub const MAX_REMOTE_ATTEMPT_LOG_POLICY_NAME_BYTES: usize = 128;
pub const MAX_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES_HARD: u32 = 1_048_576;
pub const MAX_REMOTE_ATTEMPT_LOG_SEGMENT_RECORDS_HARD: u32 = 1_024;
pub const MAX_REMOTE_ATTEMPT_LOG_SEGMENT_PAYLOAD_BYTES_HARD: u64 = 8_388_608;
pub const MAX_REMOTE_ATTEMPT_LOG_MANIFEST_SEGMENTS_HARD: u32 = 4_096;
pub const MAX_REMOTE_ATTEMPT_LOG_RETAINED_RECORDS_HARD: u32 = 65_536;
pub const MAX_REMOTE_ATTEMPT_LOG_RETAINED_PAYLOAD_BYTES_HARD: u64 = 67_108_864;
pub const MAX_REMOTE_ATTEMPT_LOG_REPLAY_RECORDS_HARD: u32 = 4_096;
pub const MAX_REMOTE_ATTEMPT_LOG_REPLAY_PAYLOAD_BYTES_HARD: u64 = 8_388_608;
pub const MAX_REMOTE_ATTEMPT_LOG_EVENT_IDENTITIES_HARD: u32 = 65_536;

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
const INITIAL_SEGMENT_INDEX: u64 = 0;
const RECORD_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-record-v1";
const SEGMENT_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-segment-v1";
const MANIFEST_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-manifest-v1";
const TRUNCATION_ANCHOR_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-truncation-anchor-v1";
const POLICY_DIGEST_DOMAIN: &str = "mantle-remote-attempt-log-policy-v1";
const REDACTED_PAYLOAD: &[u8] = b"[REDACTED]";
const CONTROL_ESCAPE_BYTES: usize = 4;
const MIN_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES: u32 = REDACTED_PAYLOAD.len() as u32;
const SECRET_MARKERS: [&[u8]; 6] = [
    b"authorization",
    b"bearer ",
    b"token=",
    b"password",
    b"secret",
    b"credential",
];

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
        validate_policy_nonzero(self)?;
        validate_policy_hard_limits(self)?;
        validate_policy_relationships(self)?;
        Ok(())
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

pub fn canonical_remote_attempt_log_policy_identity(
    name: impl Into<String>,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogPolicyIdentity, RemoteAttemptLogReasonCode> {
    policy.validate()?;
    let name = validate_policy_name(name.into())?;
    let bytes = serde_json::to_vec(&policy).map_err(|_| RemoteAttemptLogReasonCode::PolicySerializationFailed)?;
    let digest_blake3 = domain_hash(POLICY_DIGEST_DOMAIN, &bytes)?;
    debug_assert!(!name.is_empty());
    debug_assert!(is_blake3_hex_digest(digest_blake3.as_str()));
    Ok(RemoteAttemptLogPolicyIdentity { name, digest_blake3 })
}

pub fn redact_remote_attempt_log_payload(
    payload: &[u8],
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogRedactionPlan, RemoteAttemptLogReasonCode> {
    policy.validate()?;
    let hard_max = u32_to_usize(MAX_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES_HARD)?;
    if payload.len() > hard_max {
        return Err(RemoteAttemptLogReasonCode::RecordPayloadTooLarge);
    }
    if contains_secret_marker(payload) {
        return Ok(RemoteAttemptLogRedactionPlan {
            payload: REDACTED_PAYLOAD.to_vec(),
            flags: RemoteAttemptLogRecordFlags {
                secret_redacted: true,
                control_escaped: false,
                payload_truncated: false,
            },
        });
    }
    escape_and_bound_payload(payload, policy.record_payload_bytes_max)
}

pub fn seal_remote_attempt_log_record(
    input: RemoteAttemptLogRecordInput,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogRecord, RemoteAttemptLogReasonCode> {
    policy.validate()?;
    validate_scope(&input.scope)?;
    validate_event_id(&input.event_id)?;
    if input.sequence != input.cursor {
        return Err(RemoteAttemptLogReasonCode::RecordPositionMismatch);
    }
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
    if record.schema != REMOTE_ATTEMPT_LOG_RECORD_SCHEMA {
        return Err(RemoteAttemptLogReasonCode::RecordSchemaUnsupported);
    }
    validate_scope(&record.scope)?;
    validate_event_id(&record.event_id)?;
    if record.sequence != record.cursor {
        return Err(RemoteAttemptLogReasonCode::RecordPositionMismatch);
    }
    validate_record_payload_metadata(record, policy)?;
    validate_optional_digest(&record.previous_record_blake3)?;
    if record.record_blake3 != record_payload_digest(record)? {
        return Err(RemoteAttemptLogReasonCode::RecordDigestMismatch);
    }
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
    if segment.schema != REMOTE_ATTEMPT_LOG_SEGMENT_SCHEMA {
        return Err(RemoteAttemptLogReasonCode::SegmentSchemaUnsupported);
    }
    validate_scope(&segment.scope)?;
    validate_optional_digest(&segment.previous_segment_blake3)?;
    let summary = summarize_segment_records(&segment.records, policy)?;
    if !segment_summary_matches(segment, &summary) {
        return Err(RemoteAttemptLogReasonCode::ManifestSummaryMismatch);
    }
    if segment.segment_blake3 != segment_payload_digest(segment)? {
        return Err(RemoteAttemptLogReasonCode::SegmentDigestMismatch);
    }
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
    if manifest.manifest_blake3 != manifest_payload_digest(manifest)? {
        return Err(RemoteAttemptLogReasonCode::ManifestDigestMismatch);
    }
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
    if record.phase != current.phase {
        return Err(RemoteAttemptLogReasonCode::AttemptPhaseMismatch);
    }
    if let Some(existing) = manifest.event_records.get(&record.event_id) {
        return classify_existing_record(manifest, existing, record);
    }
    if manifest.event_records.len() >= u32_to_usize(policy.event_identity_count_max)? {
        return Err(RemoteAttemptLogReasonCode::EventIdentityCapacityExceeded);
    }
    validate_append_position(manifest, record)?;
    if manifest.segments.len() >= u32_to_usize(policy.manifest_segment_count_max)? {
        return Err(RemoteAttemptLogReasonCode::ManifestBoundsExceeded);
    }
    let segment_index = next_segment_index(manifest)?;
    let segment = seal_remote_attempt_log_segment(
        vec![record.clone()],
        segment_index,
        manifest.head_segment_blake3.clone(),
        policy,
    )?;
    let next_manifest = append_segment_to_manifest(manifest, &segment, policy)?;
    let expected_next_cursor = record.cursor.checked_add(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
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
    if segments.len() != manifest.segments.len() {
        return Err(RemoteAttemptLogReasonCode::SegmentChainMismatch);
    }
    let mut expected_cursor = manifest.retained_start_cursor;
    let mut expected_sequence = manifest.retained_start_cursor;
    let mut previous_record =
        manifest.truncation_anchor.as_ref().map(|anchor| anchor.dropped_tail_record_blake3.clone());
    let mut previous_segment =
        manifest.truncation_anchor.as_ref().map(|anchor| anchor.dropped_tail_segment_blake3.clone());
    let mut record_count = 0_u32;
    let mut payload_bytes = 0_u64;
    let mut retained_events = BTreeSet::new();
    for (segment, expected_ref) in segments.iter().zip(&manifest.segments) {
        validate_remote_attempt_log_segment(segment, policy)?;
        validate_segment_against_ref(segment, expected_ref)?;
        validate_chain_link(
            segment,
            &manifest.scope,
            expected_sequence,
            expected_cursor,
            &previous_record,
            &previous_segment,
        )?;
        validate_retained_event_records(segment, manifest, &mut retained_events)?;
        record_count = checked_add_u32(record_count, segment.record_count)?;
        payload_bytes = checked_add_u64(payload_bytes, segment.payload_bytes)?;
        expected_sequence = segment.next_cursor;
        expected_cursor = segment.next_cursor;
        let head_record = segment.records.last().ok_or(RemoteAttemptLogReasonCode::SegmentEmpty)?;
        previous_record = Some(head_record.record_blake3.clone());
        previous_segment = Some(segment.segment_blake3.clone());
    }
    let summary = RemoteAttemptLogChainSummary {
        retained_start_cursor: manifest.retained_start_cursor,
        next_cursor: expected_cursor,
        segment_count: usize_to_u32(segments.len())?,
        record_count,
        payload_bytes,
        head_record_blake3: previous_record,
        head_segment_blake3: previous_segment,
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
    let disposition = classify_cursor_position(RemoteAttemptLogCursorBounds {
        retained_start_cursor: manifest.retained_start_cursor,
        next_cursor: manifest.next_cursor,
        requested_cursor,
    })?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RemoteAttemptLogCursorBounds {
    retained_start_cursor: u64,
    next_cursor: u64,
    requested_cursor: u64,
}

fn classify_cursor_position(
    bounds: RemoteAttemptLogCursorBounds,
) -> Result<RemoteAttemptLogCursorDisposition, RemoteAttemptLogReasonCode> {
    if bounds.retained_start_cursor > bounds.next_cursor {
        return Err(RemoteAttemptLogReasonCode::ManifestSummaryMismatch);
    }
    if bounds.requested_cursor > bounds.next_cursor {
        return Err(RemoteAttemptLogReasonCode::CursorAfterHead);
    }
    if bounds.requested_cursor < bounds.retained_start_cursor {
        return Ok(RemoteAttemptLogCursorDisposition::Truncated);
    }
    if bounds.requested_cursor == bounds.next_cursor {
        return Ok(RemoteAttemptLogCursorDisposition::AtHead);
    }
    Ok(RemoteAttemptLogCursorDisposition::Retained)
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
            has_more: manifest.retained_start_cursor < manifest.next_cursor,
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
    if drop_count == 0 {
        return Ok(RemoteAttemptLogRetentionPlan {
            disposition: RemoteAttemptLogRetentionDisposition::Unchanged,
            reason_code: RemoteAttemptLogReasonCode::RetentionUnchanged,
            anchor: None,
            delete_segment_blake3: Vec::new(),
            retained_segments: segments.to_vec(),
            next_manifest: manifest.clone(),
        });
    }
    if drop_count >= segments.len() {
        return Err(RemoteAttemptLogReasonCode::RetentionWouldDropAll);
    }
    build_retention_plan(manifest, segments, drop_count, policy)
}

fn validate_policy_nonzero(policy: RemoteAttemptLogPolicy) -> Result<(), RemoteAttemptLogReasonCode> {
    if policy.record_payload_bytes_max < MIN_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    if policy.segment_record_count_max == 0
        || policy.segment_payload_bytes_max == 0
        || policy.manifest_segment_count_max == 0
        || policy.retained_segment_count_max == 0
        || policy.retained_record_count_max == 0
        || policy.retained_payload_bytes_max == 0
        || policy.replay_record_count_max == 0
        || policy.replay_payload_bytes_max == 0
        || policy.event_identity_count_max == 0
    {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_policy_hard_limits(policy: RemoteAttemptLogPolicy) -> Result<(), RemoteAttemptLogReasonCode> {
    if policy.record_payload_bytes_max > MAX_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES_HARD
        || policy.segment_record_count_max > MAX_REMOTE_ATTEMPT_LOG_SEGMENT_RECORDS_HARD
        || policy.segment_payload_bytes_max > MAX_REMOTE_ATTEMPT_LOG_SEGMENT_PAYLOAD_BYTES_HARD
        || policy.manifest_segment_count_max > MAX_REMOTE_ATTEMPT_LOG_MANIFEST_SEGMENTS_HARD
        || policy.retained_record_count_max > MAX_REMOTE_ATTEMPT_LOG_RETAINED_RECORDS_HARD
        || policy.retained_payload_bytes_max > MAX_REMOTE_ATTEMPT_LOG_RETAINED_PAYLOAD_BYTES_HARD
        || policy.replay_record_count_max > MAX_REMOTE_ATTEMPT_LOG_REPLAY_RECORDS_HARD
        || policy.replay_payload_bytes_max > MAX_REMOTE_ATTEMPT_LOG_REPLAY_PAYLOAD_BYTES_HARD
        || policy.event_identity_count_max > MAX_REMOTE_ATTEMPT_LOG_EVENT_IDENTITIES_HARD
    {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    if policy.retained_segment_count_max > policy.manifest_segment_count_max {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_policy_relationships(policy: RemoteAttemptLogPolicy) -> Result<(), RemoteAttemptLogReasonCode> {
    if u64::from(policy.record_payload_bytes_max) > policy.segment_payload_bytes_max {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    if policy.segment_record_count_max > policy.retained_record_count_max {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    if policy.segment_payload_bytes_max > policy.retained_payload_bytes_max {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    if policy.replay_record_count_max > policy.retained_record_count_max {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    if policy.replay_payload_bytes_max > policy.retained_payload_bytes_max {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_policy_name(name: String) -> Result<String, RemoteAttemptLogReasonCode> {
    if name.is_empty() || name.len() > MAX_REMOTE_ATTEMPT_LOG_POLICY_NAME_BYTES {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    if name.chars().any(char::is_control) {
        return Err(RemoteAttemptLogReasonCode::PolicyInvalid);
    }
    Ok(name)
}

fn escape_and_bound_payload(
    payload: &[u8],
    payload_bytes_max: u32,
) -> Result<RemoteAttemptLogRedactionPlan, RemoteAttemptLogReasonCode> {
    let max = u32_to_usize(payload_bytes_max)?;
    let mut rendered = Vec::with_capacity(payload.len().min(max));
    let mut control_escaped = false;
    let mut payload_truncated = false;
    for byte in payload {
        if byte.is_ascii_control() {
            control_escaped = true;
            let escaped = control_escape(*byte);
            if rendered.len().saturating_add(CONTROL_ESCAPE_BYTES) > max {
                payload_truncated = true;
                break;
            }
            rendered.extend_from_slice(&escaped);
            continue;
        }
        if rendered.len() >= max {
            payload_truncated = true;
            break;
        }
        rendered.push(*byte);
    }
    if rendered.len() < payload.len() && !control_escaped {
        payload_truncated = true;
    }
    debug_assert!(rendered.len() <= max);
    debug_assert!(!control_escaped || payload.iter().any(u8::is_ascii_control));
    Ok(RemoteAttemptLogRedactionPlan {
        payload: rendered,
        flags: RemoteAttemptLogRecordFlags {
            secret_redacted: false,
            control_escaped,
            payload_truncated,
        },
    })
}

fn control_escape(byte: u8) -> [u8; CONTROL_ESCAPE_BYTES] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let high = usize::from(byte >> 4);
    let low = usize::from(byte & 0x0f);
    [b'\\', b'x', HEX[high], HEX[low]]
}

fn contains_secret_marker(payload: &[u8]) -> bool {
    SECRET_MARKERS.iter().any(|marker| {
        payload
            .windows(marker.len())
            .any(|window| window.iter().zip(marker.iter()).all(|(left, right)| left.eq_ignore_ascii_case(right)))
    })
}

fn validate_scope(scope: &RemoteAttemptLogScope) -> Result<(), RemoteAttemptLogReasonCode> {
    if RemoteJobId::new(scope.job_id.as_str().to_string()).is_err()
        || RemoteAttemptId::new(scope.attempt_id.as_str().to_string()).is_err()
        || scope.fence_generation.get() == 0
    {
        return Err(RemoteAttemptLogReasonCode::ScopeIdentityInvalid);
    }
    Ok(())
}

fn validate_event_id(event_id: &RemoteEventId) -> Result<(), RemoteAttemptLogReasonCode> {
    RemoteEventId::new(event_id.as_str().to_string())
        .map(|_| ())
        .map_err(|_| RemoteAttemptLogReasonCode::ScopeIdentityInvalid)
}

fn validate_current_scope(
    expected: &RemoteAttemptLogScope,
    observed: &RemoteAttemptLogScope,
) -> Result<(), RemoteAttemptLogReasonCode> {
    if expected.job_id != observed.job_id {
        return Err(RemoteAttemptLogReasonCode::ScopeMismatch);
    }
    if observed.fence_generation < expected.fence_generation {
        return Err(RemoteAttemptLogReasonCode::StaleFenceRejected);
    }
    if observed.fence_generation > expected.fence_generation {
        return Err(RemoteAttemptLogReasonCode::UnknownFenceRejected);
    }
    if expected.attempt_id != observed.attempt_id {
        return Err(RemoteAttemptLogReasonCode::AttemptIdentityMismatch);
    }
    Ok(())
}

fn validate_record_payload_metadata(
    record: &RemoteAttemptLogRecord,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    let payload_length = usize_to_u32(record.payload.len())?;
    if payload_length > policy.record_payload_bytes_max {
        return Err(RemoteAttemptLogReasonCode::RecordPayloadTooLarge);
    }
    if payload_length != record.payload_length_bytes || record.payload_blake3 != content_hash(&record.payload) {
        return Err(RemoteAttemptLogReasonCode::RecordPayloadMetadataMismatch);
    }
    if record.flags.secret_redacted && record.payload != REDACTED_PAYLOAD {
        return Err(RemoteAttemptLogReasonCode::RecordPayloadMetadataMismatch);
    }
    Ok(())
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
    if records.is_empty() {
        return Err(RemoteAttemptLogReasonCode::SegmentEmpty);
    }
    let record_count = usize_to_u32(records.len())?;
    if record_count > policy.segment_record_count_max {
        return Err(RemoteAttemptLogReasonCode::SegmentBoundsExceeded);
    }
    let first = records.first().expect("non-empty records");
    let mut expected_sequence = first.sequence;
    let mut expected_cursor = first.cursor;
    let mut previous_record = first.previous_record_blake3.clone();
    let mut payload_bytes = 0_u64;
    for record in records {
        validate_remote_attempt_log_record(record, policy)?;
        if record.scope != first.scope || record.sequence != expected_sequence || record.cursor != expected_cursor {
            return Err(RemoteAttemptLogReasonCode::SegmentChainMismatch);
        }
        if record.previous_record_blake3 != previous_record {
            return Err(RemoteAttemptLogReasonCode::PreviousRecordMismatch);
        }
        payload_bytes = checked_add_u64(payload_bytes, u64::from(record.payload_length_bytes))?;
        if payload_bytes > policy.segment_payload_bytes_max {
            return Err(RemoteAttemptLogReasonCode::SegmentBoundsExceeded);
        }
        expected_sequence = expected_sequence.checked_add(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
        expected_cursor = expected_cursor.checked_add(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
        previous_record = Some(record.record_blake3.clone());
    }
    let last = records.last().expect("non-empty records");
    Ok(SegmentRecordSummary {
        scope: first.scope.clone(),
        first_sequence: first.sequence,
        last_sequence: last.sequence,
        first_cursor: first.cursor,
        next_cursor: expected_cursor,
        record_count,
        payload_bytes,
    })
}

fn segment_summary_matches(segment: &RemoteAttemptLogSegment, summary: &SegmentRecordSummary) -> bool {
    segment.scope == summary.scope
        && segment.first_sequence == summary.first_sequence
        && segment.last_sequence == summary.last_sequence
        && segment.first_cursor == summary.first_cursor
        && segment.next_cursor == summary.next_cursor
        && segment.record_count == summary.record_count
        && segment.payload_bytes == summary.payload_bytes
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

fn segment_ref(segment: &RemoteAttemptLogSegment) -> RemoteAttemptLogSegmentRef {
    let first = segment.records.first().expect("validated non-empty segment");
    let last = segment.records.last().expect("validated non-empty segment");
    RemoteAttemptLogSegmentRef {
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
    }
}

fn validate_manifest_shape(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    policy.validate()?;
    if manifest.schema != REMOTE_ATTEMPT_LOG_MANIFEST_SCHEMA {
        return Err(RemoteAttemptLogReasonCode::ManifestSchemaUnsupported);
    }
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
    let expected = canonical_remote_attempt_log_policy_identity(identity.name.clone(), policy)?;
    if &expected != identity {
        return Err(RemoteAttemptLogReasonCode::PolicyIdentityMismatch);
    }
    Ok(())
}

fn validate_manifest_bounds(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    if manifest.segments.len() > u32_to_usize(policy.manifest_segment_count_max)?
        || manifest.event_records.len() > u32_to_usize(policy.event_identity_count_max)?
        || manifest.retained_record_count > MAX_REMOTE_ATTEMPT_LOG_RETAINED_RECORDS_HARD
        || manifest.retained_payload_bytes > MAX_REMOTE_ATTEMPT_LOG_RETAINED_PAYLOAD_BYTES_HARD
        || manifest.retained_start_cursor > manifest.next_cursor
    {
        return Err(RemoteAttemptLogReasonCode::ManifestBoundsExceeded);
    }
    if manifest.segments.is_empty() && empty_manifest_summary_is_invalid(manifest) {
        return Err(RemoteAttemptLogReasonCode::ManifestSummaryMismatch);
    }
    Ok(())
}

fn empty_manifest_summary_is_invalid(manifest: &RemoteAttemptLogManifest) -> bool {
    manifest.retained_record_count != 0
        || manifest.retained_payload_bytes != 0
        || manifest.head_record_blake3.is_some()
        || manifest.head_segment_blake3.is_some()
        || manifest.retained_start_cursor != manifest.next_cursor
        || manifest.truncation_anchor.is_some()
}

fn validate_manifest_segment_refs(manifest: &RemoteAttemptLogManifest) -> Result<(), RemoteAttemptLogReasonCode> {
    let mut expected_cursor = manifest.retained_start_cursor;
    let mut expected_index = manifest.segments.first().map(|item| item.segment_index);
    let mut previous_segment =
        manifest.truncation_anchor.as_ref().map(|anchor| anchor.dropped_tail_segment_blake3.clone());
    let mut previous_record =
        manifest.truncation_anchor.as_ref().map(|anchor| anchor.dropped_tail_record_blake3.clone());
    let mut record_count = 0_u32;
    let mut payload_bytes = 0_u64;
    for item in &manifest.segments {
        validate_segment_ref(item)?;
        if Some(item.segment_index) != expected_index
            || item.first_cursor != expected_cursor
            || item.first_sequence != expected_cursor
            || item.previous_segment_blake3 != previous_segment
            || item.first_previous_record_blake3 != previous_record
        {
            return Err(RemoteAttemptLogReasonCode::SegmentChainMismatch);
        }
        record_count = checked_add_u32(record_count, item.record_count)?;
        payload_bytes = checked_add_u64(payload_bytes, item.payload_bytes)?;
        expected_cursor = item.next_cursor;
        expected_index = Some(item.segment_index.checked_add(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?);
        previous_segment = Some(item.segment_blake3.clone());
        previous_record = Some(item.head_record_blake3.clone());
    }
    if record_count != manifest.retained_record_count
        || payload_bytes != manifest.retained_payload_bytes
        || expected_cursor != manifest.next_cursor
        || previous_segment != manifest.head_segment_blake3
        || previous_record != manifest.head_record_blake3
    {
        return Err(RemoteAttemptLogReasonCode::ManifestSummaryMismatch);
    }
    Ok(())
}

fn validate_segment_ref(item: &RemoteAttemptLogSegmentRef) -> Result<(), RemoteAttemptLogReasonCode> {
    validate_optional_digest(&item.previous_segment_blake3)?;
    validate_optional_digest(&item.first_previous_record_blake3)?;
    RemoteAttemptLogDigest::new(item.segment_blake3.as_str().to_string())?;
    RemoteAttemptLogDigest::new(item.head_record_blake3.as_str().to_string())?;
    if item.record_count == 0 || item.first_sequence != item.first_cursor {
        return Err(RemoteAttemptLogReasonCode::ManifestSummaryMismatch);
    }
    let expected_next = item
        .first_cursor
        .checked_add(u64::from(item.record_count))
        .ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
    let expected_last = expected_next.checked_sub(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
    if item.next_cursor != expected_next || item.last_sequence != expected_last {
        return Err(RemoteAttemptLogReasonCode::ManifestSummaryMismatch);
    }
    Ok(())
}

fn validate_manifest_events(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    if manifest.event_records.len() > u32_to_usize(policy.event_identity_count_max)? {
        return Err(RemoteAttemptLogReasonCode::EventIdentityCapacityExceeded);
    }
    for (event_id, digest) in &manifest.event_records {
        validate_event_id(event_id)?;
        RemoteAttemptLogDigest::new(digest.as_str().to_string())?;
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
    if existing != &record.record_blake3 {
        return Err(RemoteAttemptLogReasonCode::EventDigestConflict);
    }
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
    if record.sequence != manifest.next_cursor || record.cursor != manifest.next_cursor {
        return Err(RemoteAttemptLogReasonCode::RecordPositionMismatch);
    }
    if record.previous_record_blake3 != manifest.head_record_blake3 {
        return Err(RemoteAttemptLogReasonCode::PreviousRecordMismatch);
    }
    Ok(())
}

fn next_segment_index(manifest: &RemoteAttemptLogManifest) -> Result<u64, RemoteAttemptLogReasonCode> {
    match manifest.segments.last() {
        Some(previous) => previous.segment_index.checked_add(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow),
        None => Ok(INITIAL_SEGMENT_INDEX),
    }
}

fn append_segment_to_manifest(
    manifest: &RemoteAttemptLogManifest,
    segment: &RemoteAttemptLogSegment,
    policy: RemoteAttemptLogPolicy,
) -> Result<RemoteAttemptLogManifest, RemoteAttemptLogReasonCode> {
    let mut next = manifest.clone();
    next.segments.push(segment_ref(segment));
    next.next_cursor = segment.next_cursor;
    next.head_record_blake3 = Some(segment.records.last().expect("validated non-empty segment").record_blake3.clone());
    next.head_segment_blake3 = Some(segment.segment_blake3.clone());
    next.retained_record_count = checked_add_u32(next.retained_record_count, segment.record_count)?;
    next.retained_payload_bytes = checked_add_u64(next.retained_payload_bytes, segment.payload_bytes)?;
    for record in &segment.records {
        if next.event_records.insert(record.event_id.clone(), record.record_blake3.clone()).is_some() {
            return Err(RemoteAttemptLogReasonCode::EventDigestConflict);
        }
    }
    seal_manifest(next, policy)
}

fn validate_segment_against_ref(
    segment: &RemoteAttemptLogSegment,
    expected: &RemoteAttemptLogSegmentRef,
) -> Result<(), RemoteAttemptLogReasonCode> {
    if segment_ref(segment) != *expected {
        return Err(RemoteAttemptLogReasonCode::SegmentChainMismatch);
    }
    Ok(())
}

fn validate_chain_link(
    segment: &RemoteAttemptLogSegment,
    scope: &RemoteAttemptLogScope,
    expected_sequence: u64,
    expected_cursor: u64,
    previous_record: &Option<RemoteAttemptLogDigest>,
    previous_segment: &Option<RemoteAttemptLogDigest>,
) -> Result<(), RemoteAttemptLogReasonCode> {
    let first = segment.records.first().expect("validated non-empty segment");
    if &segment.scope != scope
        || segment.first_sequence != expected_sequence
        || segment.first_cursor != expected_cursor
        || &first.previous_record_blake3 != previous_record
        || &segment.previous_segment_blake3 != previous_segment
    {
        return Err(RemoteAttemptLogReasonCode::SegmentChainMismatch);
    }
    Ok(())
}

fn validate_retained_event_records(
    segment: &RemoteAttemptLogSegment,
    manifest: &RemoteAttemptLogManifest,
    retained_events: &mut BTreeSet<RemoteEventId>,
) -> Result<(), RemoteAttemptLogReasonCode> {
    for record in &segment.records {
        if !retained_events.insert(record.event_id.clone()) {
            return Err(RemoteAttemptLogReasonCode::EventDigestConflict);
        }
        if manifest.event_records.get(&record.event_id) != Some(&record.record_blake3) {
            return Err(RemoteAttemptLogReasonCode::EventDigestConflict);
        }
    }
    Ok(())
}

fn validate_chain_summary(
    manifest: &RemoteAttemptLogManifest,
    summary: &RemoteAttemptLogChainSummary,
) -> Result<(), RemoteAttemptLogReasonCode> {
    if summary.next_cursor != manifest.next_cursor
        || summary.segment_count != usize_to_u32(manifest.segments.len())?
        || summary.record_count != manifest.retained_record_count
        || summary.payload_bytes != manifest.retained_payload_bytes
        || summary.head_record_blake3 != manifest.head_record_blake3
        || summary.head_segment_blake3 != manifest.head_segment_blake3
    {
        return Err(RemoteAttemptLogReasonCode::ManifestSummaryMismatch);
    }
    Ok(())
}

fn validate_replay_request(
    request: RemoteAttemptLogReplayRequest,
    policy: RemoteAttemptLogPolicy,
) -> Result<(), RemoteAttemptLogReasonCode> {
    if request.record_count_max == 0
        || request.payload_bytes_max == 0
        || request.record_count_max > policy.replay_record_count_max
        || request.payload_bytes_max > policy.replay_payload_bytes_max
    {
        return Err(RemoteAttemptLogReasonCode::ReplayRequestInvalid);
    }
    Ok(())
}

fn collect_replay_records(
    manifest: &RemoteAttemptLogManifest,
    segments: &[RemoteAttemptLogSegment],
    request: RemoteAttemptLogReplayRequest,
    cursor: RemoteAttemptLogCursorDecision,
) -> Result<RemoteAttemptLogReplayPlan, RemoteAttemptLogReasonCode> {
    let mut records = Vec::new();
    let mut payload_bytes = 0_u64;
    let record_limit = u32_to_usize(request.record_count_max)?;
    for record in segments.iter().flat_map(|segment| &segment.records) {
        if record.cursor < cursor.effective_cursor {
            continue;
        }
        let projected = checked_add_u64(payload_bytes, u64::from(record.payload_length_bytes))?;
        if records.len() >= record_limit || projected > request.payload_bytes_max {
            break;
        }
        payload_bytes = projected;
        records.push(record.clone());
    }
    if records.is_empty() {
        return Err(RemoteAttemptLogReasonCode::ReplayLimitTooSmall);
    }
    let next_cursor = records
        .last()
        .expect("non-empty replay records")
        .cursor
        .checked_add(1)
        .ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
    debug_assert!(payload_bytes <= request.payload_bytes_max);
    debug_assert!(records.len() <= record_limit);
    Ok(RemoteAttemptLogReplayPlan {
        cursor,
        records,
        next_cursor,
        has_more: next_cursor < manifest.next_cursor,
        truncation_anchor: None,
    })
}

fn retention_drop_count(
    manifest: &RemoteAttemptLogManifest,
    policy: RemoteAttemptLogPolicy,
) -> Result<usize, RemoteAttemptLogReasonCode> {
    let mut remaining_segments = usize_to_u32(manifest.segments.len())?;
    let mut remaining_records = manifest.retained_record_count;
    let mut remaining_bytes = manifest.retained_payload_bytes;
    let mut drop_count = 0_usize;
    while retention_exceeded(remaining_segments, remaining_records, remaining_bytes, policy) {
        let item = manifest.segments.get(drop_count).ok_or(RemoteAttemptLogReasonCode::RetentionWouldDropAll)?;
        remaining_segments = remaining_segments.checked_sub(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
        remaining_records = remaining_records
            .checked_sub(item.record_count)
            .ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
        remaining_bytes = remaining_bytes
            .checked_sub(item.payload_bytes)
            .ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
        drop_count = drop_count.checked_add(1).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
    }
    Ok(drop_count)
}

fn retention_exceeded(
    segment_count: u32,
    record_count: u32,
    payload_bytes: u64,
    policy: RemoteAttemptLogPolicy,
) -> bool {
    segment_count > policy.retained_segment_count_max
        || record_count > policy.retained_record_count_max
        || payload_bytes > policy.retained_payload_bytes_max
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
    next.retained_record_count = next
        .retained_record_count
        .checked_sub(dropped_summary.record_count)
        .ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
    next.retained_payload_bytes = next
        .retained_payload_bytes
        .checked_sub(dropped_summary.payload_bytes)
        .ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
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
    let mut record_count = 0_u32;
    let mut payload_bytes = 0_u64;
    for segment in dropped {
        record_count = checked_add_u32(record_count, segment.record_count)?;
        payload_bytes = checked_add_u64(payload_bytes, segment.payload_bytes)?;
    }
    Ok(DroppedSegmentSummary {
        start_cursor: first.first_cursor,
        end_cursor: last.next_cursor,
        chunk_count: usize_to_u32(dropped.len())?,
        record_count,
        payload_bytes,
        tail_record_blake3: last.records.last().expect("validated non-empty segment").record_blake3.clone(),
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
    Ok(anchor)
}

fn validate_truncation_anchor(
    anchor: &RemoteAttemptLogTruncationAnchorRecord,
    scope: &RemoteAttemptLogScope,
    policy_identity: &RemoteAttemptLogPolicyIdentity,
) -> Result<(), RemoteAttemptLogReasonCode> {
    if anchor.schema != REMOTE_ATTEMPT_LOG_TRUNCATION_ANCHOR_SCHEMA
        || &anchor.scope != scope
        || &anchor.policy != policy_identity
        || anchor.dropped_start_cursor >= anchor.dropped_end_cursor_exclusive
        || anchor.dropped_end_cursor_exclusive != anchor.new_retained_start_cursor
        || anchor.dropped_chunk_count == 0
        || anchor.dropped_record_count == 0
    {
        return Err(RemoteAttemptLogReasonCode::RetentionAnchorInvalid);
    }
    validate_optional_digest(&anchor.previous_anchor_blake3)?;
    for digest in [
        &anchor.dropped_tail_record_blake3,
        &anchor.dropped_tail_segment_blake3,
        &anchor.prior_head_record_blake3,
        &anchor.prior_head_segment_blake3,
    ] {
        RemoteAttemptLogDigest::new(digest.as_str().to_string())?;
    }
    if anchor.anchor_blake3 != truncation_anchor_payload_digest(anchor)? {
        return Err(RemoteAttemptLogReasonCode::RetentionAnchorInvalid);
    }
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
        RemoteAttemptLogDigest::new(digest.as_str().to_string())?;
    }
    Ok(())
}

fn content_hash(bytes: &[u8]) -> RemoteAttemptLogDigest {
    RemoteAttemptLogDigest(blake3::hash(bytes).to_hex().to_string())
}

fn domain_hash(domain: &str, bytes: &[u8]) -> Result<RemoteAttemptLogDigest, RemoteAttemptLogReasonCode> {
    let mut hasher = blake3::Hasher::new();
    let domain_length_bytes = u64::try_from(domain.len()).map_err(|_| RemoteAttemptLogReasonCode::ArithmeticOverflow)?;
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
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn checked_add_u32(left: u32, right: u32) -> Result<u32, RemoteAttemptLogReasonCode> {
    left.checked_add(right).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)
}

fn checked_add_u64(left: u64, right: u64) -> Result<u64, RemoteAttemptLogReasonCode> {
    left.checked_add(right).ok_or(RemoteAttemptLogReasonCode::ArithmeticOverflow)
}

fn usize_to_u32(value: usize) -> Result<u32, RemoteAttemptLogReasonCode> {
    u32::try_from(value).map_err(|_| RemoteAttemptLogReasonCode::ArithmeticOverflow)
}

fn u32_to_usize(value: u32) -> Result<usize, RemoteAttemptLogReasonCode> {
    usize::try_from(value).map_err(|_| RemoteAttemptLogReasonCode::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    // r[verify remote_builds.immutable_attempt_log_segments]
    // r[verify remote_builds.pure_log_cursor_kernel]
    use proptest::prelude::*;

    use super::*;

    const TEST_FENCE_GENERATION: u64 = 7;
    const TEST_RECORD_COUNT: u64 = 4;
    const TEST_RETAINED_SEGMENTS: u32 = 2;
    const TEST_REPLAY_RECORDS: u32 = 2;
    const TEST_MULTI_RECORD_SEGMENT_COUNT: u32 = 2;
    const TEST_REPLAY_BYTES: u64 = 128;
    const TEST_SMALL_PAYLOAD_MAX: u32 = 16;
    const TEST_PROPERTY_CURSOR_MAX_EXCLUSIVE: u64 = 10_000;
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
    fn canonical_record_segment_and_manifest_identities_are_deterministic() {
        let manifest = empty(policy());
        let left = record(&manifest, "event-canonical", b"same payload", policy());
        let right = record(&manifest, "event-canonical", b"same payload", policy());
        let changed = record(&manifest, "event-canonical", b"changed payload", policy());
        let left_segment =
            seal_remote_attempt_log_segment(vec![left.clone()], INITIAL_SEGMENT_INDEX, None, policy()).unwrap();
        let right_segment =
            seal_remote_attempt_log_segment(vec![right.clone()], INITIAL_SEGMENT_INDEX, None, policy()).unwrap();

        assert_eq!(left.record_blake3, right.record_blake3);
        assert_ne!(left.record_blake3, changed.record_blake3);
        assert_eq!(left_segment.segment_blake3, right_segment.segment_blake3);
        assert_eq!(manifest.manifest_blake3, empty(policy()).manifest_blake3);
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
        assert_eq!(checked_add_u64(u64::MAX, 1), Err(RemoteAttemptLogReasonCode::ArithmeticOverflow));
        assert_eq!(checked_add_u32(u32::MAX, 1), Err(RemoteAttemptLogReasonCode::ArithmeticOverflow));
        assert_eq!(manifest.next_cursor, INITIAL_LOG_POSITION);
    }

    proptest! {
        #[test]
        fn equivalent_record_facts_produce_equivalent_identity(
            cursor in INITIAL_LOG_POSITION..TEST_PROPERTY_CURSOR_MAX_EXCLUSIVE,
            payload in proptest::collection::vec(any::<u8>(), 0..256)
        ) {
            let manifest = empty(policy());
            let input = RemoteAttemptLogRecordInput {
                scope: manifest.scope.clone(),
                event_id: RemoteEventId::new(format!("property-event-{cursor}")).unwrap(),
                sequence: cursor,
                cursor,
                phase: RemoteAttemptPhase::Running,
                stream: RemoteAttemptLogStream::Event,
                kind: RemoteAttemptLogRecordKind::Diagnostic,
                payload,
                previous_record_blake3: None,
            };
            let left = seal_remote_attempt_log_record(input.clone(), policy()).unwrap();
            let right = seal_remote_attempt_log_record(input, policy()).unwrap();
            prop_assert_eq!(&left, &right);
            prop_assert_eq!(left.record_blake3, right.record_blake3);
        }

        #[test]
        fn checked_accounting_never_wraps(left in any::<u64>(), right in any::<u64>()) {
            match checked_add_u64(left, right) {
                Ok(sum) => {
                    prop_assert!(sum >= left);
                    prop_assert!(sum >= right);
                }
                Err(reason) => prop_assert_eq!(reason, RemoteAttemptLogReasonCode::ArithmeticOverflow),
            }
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn checked_accounting_never_wraps() {
        let left: u64 = kani::any();
        let right: u64 = kani::any();
        match checked_add_u64(left, right) {
            Ok(sum) => {
                assert!(sum >= left);
                assert!(sum >= right);
            }
            Err(reason) => assert_eq!(reason, RemoteAttemptLogReasonCode::ArithmeticOverflow),
        }
    }

    #[kani::proof]
    fn cursor_classification_respects_retained_and_head_bounds() {
        let retained_start_cursor: u64 = kani::any();
        let next_cursor: u64 = kani::any();
        let requested_cursor: u64 = kani::any();
        kani::assume(retained_start_cursor <= next_cursor);
        let decision = classify_cursor_position(retained_start_cursor, next_cursor, requested_cursor);
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
