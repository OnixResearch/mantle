//! Pure immutable remote-attempt-log decisions and bounded payload redaction.
//! Wire encoding, hashing, durable storage, and DTO construction belong to the host.

use crate::attempt::RemoteAttemptPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptLogError {
    ScopeMismatch,
    StaleFenceRejected,
    UnknownFenceRejected,
    AttemptIdentityMismatch,
    AttemptPhaseMismatch,
    EventDigestConflict,
    EventIdentityCapacityExceeded,
    RecordPositionMismatch,
    PreviousRecordMismatch,
    SegmentChainMismatch,
    SegmentBoundsExceeded,
    ManifestSummaryMismatch,
    ArithmeticOverflow,
    CursorAfterHead,
    ReplayRequestInvalid,
    ReplayLimitTooSmall,
    PolicyInvalid,
    PolicyIdentityMismatch,
    RecordPayloadTooLarge,
    ScopeIdentityInvalid,
    RecordPayloadMetadataMismatch,
    ManifestBoundsExceeded,
    RetentionAnchorInvalid,
    RecordSchemaUnsupported,
    SegmentSchemaUnsupported,
    ManifestSchemaUnsupported,
    RecordDigestMismatch,
    SegmentDigestMismatch,
    ManifestDigestMismatch,
    DigestInvalid,
    RetentionWouldDropAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scope<'a> {
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub fence_generation: u64,
}

pub fn check_scope(expected: Scope<'_>, observed: Scope<'_>) -> Result<(), AttemptLogError> {
    if expected.job_id != observed.job_id {
        return Err(AttemptLogError::ScopeMismatch);
    }
    if observed.fence_generation < expected.fence_generation {
        return Err(AttemptLogError::StaleFenceRejected);
    }
    if observed.fence_generation > expected.fence_generation {
        return Err(AttemptLogError::UnknownFenceRejected);
    }
    if expected.attempt_id != observed.attempt_id {
        return Err(AttemptLogError::AttemptIdentityMismatch);
    }
    Ok(())
}

pub fn check_phase(expected: RemoteAttemptPhase, observed: RemoteAttemptPhase) -> Result<(), AttemptLogError> {
    if expected != observed {
        return Err(AttemptLogError::AttemptPhaseMismatch);
    }
    Ok(())
}

pub fn classify_existing_record(existing: &str, candidate: &str) -> Result<(), AttemptLogError> {
    if existing != candidate {
        return Err(AttemptLogError::EventDigestConflict);
    }
    Ok(())
}

pub fn check_event_capacity(count: usize, maximum: usize) -> Result<(), AttemptLogError> {
    if count >= maximum {
        return Err(AttemptLogError::EventIdentityCapacityExceeded);
    }
    Ok(())
}

pub fn check_append_position(
    expected_cursor: u64,
    expected_previous: Option<&str>,
    sequence: u64,
    cursor: u64,
    previous: Option<&str>,
) -> Result<(), AttemptLogError> {
    if sequence != expected_cursor || cursor != expected_cursor {
        return Err(AttemptLogError::RecordPositionMismatch);
    }
    if previous != expected_previous {
        return Err(AttemptLogError::PreviousRecordMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainLink<'a> {
    pub scope: Scope<'a>,
    pub sequence: u64,
    pub cursor: u64,
    pub previous_record: Option<&'a str>,
    pub previous_segment: Option<&'a str>,
}

pub fn check_chain_link(expected: ChainLink<'_>, observed: ChainLink<'_>) -> Result<(), AttemptLogError> {
    if expected.scope != observed.scope
        || expected.sequence != observed.sequence
        || expected.cursor != observed.cursor
        || expected.previous_record != observed.previous_record
        || expected.previous_segment != observed.previous_segment
    {
        return Err(AttemptLogError::SegmentChainMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefLink<'a> {
    pub index: Option<u64>,
    pub first_cursor: u64,
    pub first_sequence: u64,
    pub previous_record: Option<&'a str>,
    pub previous_segment: Option<&'a str>,
}

pub fn check_ref_link(expected: RefLink<'_>, observed: RefLink<'_>) -> Result<(), AttemptLogError> {
    if expected.index != observed.index
        || expected.first_cursor != observed.first_cursor
        || expected.first_sequence != observed.first_sequence
        || expected.previous_segment != observed.previous_segment
        || expected.previous_record != observed.previous_record
    {
        return Err(AttemptLogError::SegmentChainMismatch);
    }
    Ok(())
}

pub fn check_segment_ref_shape(
    first_sequence: u64,
    first_cursor: u64,
    next_cursor: u64,
    last_sequence: u64,
    record_count: u32,
) -> Result<(), AttemptLogError> {
    if record_count == 0 || first_sequence != first_cursor {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    let expected_next = first_cursor.checked_add(u64::from(record_count)).ok_or(AttemptLogError::ArithmeticOverflow)?;
    let expected_last = expected_next.checked_sub(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
    if next_cursor != expected_next || last_sequence != expected_last {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDisposition {
    Retained,
    AtHead,
    Truncated,
}

pub fn classify_cursor(retained_start: u64, next: u64, requested: u64) -> Result<CursorDisposition, AttemptLogError> {
    if retained_start > next {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if requested > next {
        return Err(AttemptLogError::CursorAfterHead);
    }
    if requested < retained_start {
        return Ok(CursorDisposition::Truncated);
    }
    if requested == next {
        return Ok(CursorDisposition::AtHead);
    }
    Ok(CursorDisposition::Retained)
}

pub fn validate_replay_request(count: u32, bytes: u64, max_count: u32, max_bytes: u64) -> Result<(), AttemptLogError> {
    if count == 0 || bytes == 0 || count > max_count || bytes > max_bytes {
        return Err(AttemptLogError::ReplayRequestInvalid);
    }
    Ok(())
}

/// Returns the new payload total when this record fits, or `None` when the page ends.
pub fn admit_replay_record(
    record_count: usize,
    record_count_max: usize,
    payload_bytes: u64,
    record_bytes: u32,
    payload_bytes_max: u64,
) -> Result<Option<u64>, AttemptLogError> {
    let projected = payload_bytes.checked_add(u64::from(record_bytes)).ok_or(AttemptLogError::ArithmeticOverflow)?;
    if record_count >= record_count_max || projected > payload_bytes_max {
        return Ok(None);
    }
    Ok(Some(projected))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionUsage {
    pub segment_count: u32,
    pub record_count: u32,
    pub payload_bytes: u64,
}

fn retention_exceeded(usage: RetentionUsage, limits: RetentionUsage) -> bool {
    usage.segment_count > limits.segment_count
        || usage.record_count > limits.record_count
        || usage.payload_bytes > limits.payload_bytes
}

/// Counts oldest segments to discard, leaving the last segment when possible.
/// The caller must first validate the manifest and its segment references.
pub fn retention_drop_count(
    mut usage: RetentionUsage,
    limits: RetentionUsage,
    segments: impl IntoIterator<Item = RetentionUsage>,
) -> Result<usize, AttemptLogError> {
    let mut segments = segments.into_iter();
    let mut drop_count = 0_usize;
    while retention_exceeded(usage, limits) {
        let item = segments.next().ok_or(AttemptLogError::RetentionWouldDropAll)?;
        usage.segment_count = usage.segment_count.checked_sub(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
        usage.record_count =
            usage.record_count.checked_sub(item.record_count).ok_or(AttemptLogError::ArithmeticOverflow)?;
        usage.payload_bytes =
            usage.payload_bytes.checked_sub(item.payload_bytes).ok_or(AttemptLogError::ArithmeticOverflow)?;
        drop_count = drop_count.checked_add(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
    }
    Ok(drop_count)
}

pub const MAX_POLICY_NAME_BYTES: usize = 128;
pub const MAX_RECORD_PAYLOAD_BYTES: u32 = 1_048_576;
pub const MAX_SEGMENT_RECORDS: u32 = 1_024;
pub const MAX_SEGMENT_PAYLOAD_BYTES: u64 = 8_388_608;
pub const MAX_MANIFEST_SEGMENTS: u32 = 4_096;
pub const MAX_RETAINED_RECORDS: u32 = 65_536;
pub const MAX_RETAINED_PAYLOAD_BYTES: u64 = 67_108_864;
pub const MAX_REPLAY_RECORDS: u32 = 4_096;
pub const MAX_REPLAY_PAYLOAD_BYTES: u64 = 8_388_608;
pub const MAX_EVENT_IDENTITIES: u32 = 65_536;
pub const REDACTED_PAYLOAD: &[u8] = b"[REDACTED]";
const ESCAPE_BYTES: usize = 4;
const SECRET_MARKERS: [&[u8]; 6] = [
    b"authorization",
    b"bearer ",
    b"token=",
    b"password",
    b"secret",
    b"credential",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
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

pub fn validate_policy(policy: Policy) -> Result<(), AttemptLogError> {
    if policy.record_payload_bytes_max < REDACTED_PAYLOAD.len() as u32
        || policy.segment_record_count_max == 0
        || policy.segment_payload_bytes_max == 0
        || policy.manifest_segment_count_max == 0
        || policy.retained_segment_count_max == 0
        || policy.retained_record_count_max == 0
        || policy.retained_payload_bytes_max == 0
        || policy.replay_record_count_max == 0
        || policy.replay_payload_bytes_max == 0
        || policy.event_identity_count_max == 0
        || policy.record_payload_bytes_max > MAX_RECORD_PAYLOAD_BYTES
        || policy.segment_record_count_max > MAX_SEGMENT_RECORDS
        || policy.segment_payload_bytes_max > MAX_SEGMENT_PAYLOAD_BYTES
        || policy.manifest_segment_count_max > MAX_MANIFEST_SEGMENTS
        || policy.retained_record_count_max > MAX_RETAINED_RECORDS
        || policy.retained_payload_bytes_max > MAX_RETAINED_PAYLOAD_BYTES
        || policy.replay_record_count_max > MAX_REPLAY_RECORDS
        || policy.replay_payload_bytes_max > MAX_REPLAY_PAYLOAD_BYTES
        || policy.event_identity_count_max > MAX_EVENT_IDENTITIES
        || policy.retained_segment_count_max > policy.manifest_segment_count_max
        || u64::from(policy.record_payload_bytes_max) > policy.segment_payload_bytes_max
        || policy.segment_record_count_max > policy.retained_record_count_max
        || policy.segment_payload_bytes_max > policy.retained_payload_bytes_max
        || policy.replay_record_count_max > policy.retained_record_count_max
        || policy.replay_payload_bytes_max > policy.retained_payload_bytes_max
    {
        return Err(AttemptLogError::PolicyInvalid);
    }
    Ok(())
}

pub fn validate_policy_name(name: &str) -> Result<(), AttemptLogError> {
    if name.is_empty() || name.len() > MAX_POLICY_NAME_BYTES || name.chars().any(char::is_control) {
        return Err(AttemptLogError::PolicyInvalid);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedPayload {
    pub payload: alloc::vec::Vec<u8>,
    pub secret_redacted: bool,
    pub control_escaped: bool,
    pub payload_truncated: bool,
}

pub fn redact_payload(payload: &[u8], payload_bytes_max: u32) -> Result<RedactedPayload, AttemptLogError> {
    if payload.len() > MAX_RECORD_PAYLOAD_BYTES as usize {
        return Err(AttemptLogError::RecordPayloadTooLarge);
    }
    if SECRET_MARKERS.iter().any(|marker| {
        payload
            .windows(marker.len())
            .any(|window| window.iter().zip(marker.iter()).all(|(left, right)| left.eq_ignore_ascii_case(right)))
    }) {
        return Ok(RedactedPayload {
            payload: REDACTED_PAYLOAD.to_vec(),
            secret_redacted: true,
            control_escaped: false,
            payload_truncated: false,
        });
    }
    let max = payload_bytes_max as usize;
    let mut rendered = alloc::vec::Vec::with_capacity(payload.len().min(max));
    let mut control_escaped = false;
    let mut payload_truncated = false;
    for byte in payload {
        if byte.is_ascii_control() {
            control_escaped = true;
            const HEX: &[u8; 16] = b"0123456789abcdef";
            if rendered.len().saturating_add(ESCAPE_BYTES) > max {
                payload_truncated = true;
                break;
            }
            rendered.extend_from_slice(&[b'\\', b'x', HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 0x0f)]]);
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
    Ok(RedactedPayload {
        payload: rendered,
        secret_redacted: false,
        control_escaped,
        payload_truncated,
    })
}

pub fn validate_identity(value: &str) -> Result<(), AttemptLogError> {
    if value.is_empty()
        || value.len() > crate::attempt::MAX_REMOTE_ATTEMPT_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(AttemptLogError::ScopeIdentityInvalid);
    }
    Ok(())
}

pub fn validate_scope_identity(scope: Scope<'_>) -> Result<(), AttemptLogError> {
    validate_identity(scope.job_id)?;
    validate_identity(scope.attempt_id)?;
    if scope.fence_generation == 0 {
        return Err(AttemptLogError::ScopeIdentityInvalid);
    }
    Ok(())
}

pub fn check_record_payload_metadata(
    actual_length: usize,
    declared_length: u32,
    maximum: u32,
    hash_matches: impl FnOnce() -> bool,
    secret_redacted: bool,
    actual_redacted: impl FnOnce() -> bool,
) -> Result<(), AttemptLogError> {
    let length = u32::try_from(actual_length).map_err(|_| AttemptLogError::ArithmeticOverflow)?;
    if length > maximum {
        return Err(AttemptLogError::RecordPayloadTooLarge);
    }
    if length != declared_length || !hash_matches() {
        return Err(AttemptLogError::RecordPayloadMetadataMismatch);
    }
    if secret_redacted && !actual_redacted() {
        return Err(AttemptLogError::RecordPayloadMetadataMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentFacts<'a> {
    pub scope: Scope<'a>,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub first_cursor: u64,
    pub next_cursor: u64,
    pub record_count: u32,
    pub payload_bytes: u64,
}

pub fn check_segment_summary(expected: SegmentFacts<'_>, observed: SegmentFacts<'_>) -> Result<(), AttemptLogError> {
    if expected != observed {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    Ok(())
}

pub fn check_segment_record_count(count: usize, max: u32) -> Result<u32, AttemptLogError> {
    let count = u32::try_from(count).map_err(|_| AttemptLogError::ArithmeticOverflow)?;
    if count > max {
        return Err(AttemptLogError::SegmentBoundsExceeded);
    }
    Ok(count)
}

pub struct SegmentAccumulator<'a> {
    pub scope: Scope<'a>,
    pub expected_sequence: u64,
    pub expected_cursor: u64,
    pub previous_record: Option<&'a str>,
    pub payload_bytes: u64,
}

impl<'a> SegmentAccumulator<'a> {
    pub fn accept(
        &mut self,
        scope: Scope<'a>,
        sequence: u64,
        cursor: u64,
        previous_record: Option<&str>,
        record_digest: &'a str,
        record_bytes: u32,
        maximum_payload_bytes: u64,
    ) -> Result<(), AttemptLogError> {
        if scope != self.scope || sequence != self.expected_sequence || cursor != self.expected_cursor {
            return Err(AttemptLogError::SegmentChainMismatch);
        }
        if previous_record != self.previous_record {
            return Err(AttemptLogError::PreviousRecordMismatch);
        }
        self.payload_bytes =
            self.payload_bytes.checked_add(u64::from(record_bytes)).ok_or(AttemptLogError::ArithmeticOverflow)?;
        if self.payload_bytes > maximum_payload_bytes {
            return Err(AttemptLogError::SegmentBoundsExceeded);
        }
        self.expected_sequence = self.expected_sequence.checked_add(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
        self.expected_cursor = self.expected_cursor.checked_add(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
        self.previous_record = Some(record_digest);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestBounds {
    pub segment_count: usize,
    pub event_count: usize,
    pub retained_records: u32,
    pub retained_bytes: u64,
    pub retained_start: u64,
    pub next_cursor: u64,
    pub head_record_present: bool,
    pub head_segment_present: bool,
    pub anchor_present: bool,
}

pub fn check_manifest_bounds(facts: ManifestBounds, policy: Policy) -> Result<(), AttemptLogError> {
    if facts.segment_count > policy.manifest_segment_count_max as usize
        || facts.event_count > policy.event_identity_count_max as usize
        || facts.retained_records > MAX_RETAINED_RECORDS
        || facts.retained_bytes > MAX_RETAINED_PAYLOAD_BYTES
        || facts.retained_start > facts.next_cursor
    {
        return Err(AttemptLogError::ManifestBoundsExceeded);
    }
    if facts.segment_count == 0
        && (facts.retained_records != 0
            || facts.retained_bytes != 0
            || facts.head_record_present
            || facts.head_segment_present
            || facts.retained_start != facts.next_cursor
            || facts.anchor_present)
    {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    Ok(())
}

pub fn check_manifest_event_count(count: usize, maximum: u32) -> Result<(), AttemptLogError> {
    if count > maximum as usize {
        return Err(AttemptLogError::EventIdentityCapacityExceeded);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestTotals<'a> {
    pub next_cursor: u64,
    pub segment_count: u32,
    pub record_count: u32,
    pub payload_bytes: u64,
    pub head_record: Option<&'a str>,
    pub head_segment: Option<&'a str>,
}

pub fn check_manifest_ref_totals(
    expected: ManifestTotals<'_>,
    actual: ManifestTotals<'_>,
) -> Result<(), AttemptLogError> {
    if actual.record_count != expected.record_count
        || actual.payload_bytes != expected.payload_bytes
        || actual.next_cursor != expected.next_cursor
        || actual.head_segment != expected.head_segment
        || actual.head_record != expected.head_record
    {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    Ok(())
}

pub fn check_chain_totals(expected: ManifestTotals<'_>, actual: ManifestTotals<'_>) -> Result<(), AttemptLogError> {
    if actual.next_cursor != expected.next_cursor
        || actual.segment_count != expected.segment_count
        || actual.record_count != expected.record_count
        || actual.payload_bytes != expected.payload_bytes
        || actual.head_record != expected.head_record
        || actual.head_segment != expected.head_segment
    {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchorFacts {
    pub schema_matches: bool,
    pub dropped_start: u64,
    pub dropped_end: u64,
    pub retained_start: u64,
    pub dropped_chunks: u32,
    pub dropped_records: u32,
}

pub fn check_anchor(
    facts: AnchorFacts,
    scope_matches: impl FnOnce() -> bool,
    policy_matches: impl FnOnce() -> bool,
) -> Result<(), AttemptLogError> {
    if !facts.schema_matches
        || !scope_matches()
        || !policy_matches()
        || facts.dropped_start >= facts.dropped_end
        || facts.dropped_end != facts.retained_start
        || facts.dropped_chunks == 0
        || facts.dropped_records == 0
    {
        return Err(AttemptLogError::RetentionAnchorInvalid);
    }
    Ok(())
}

pub fn digest_is_valid(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub fn dropped_usage(
    segments: impl IntoIterator<Item = (u32, u64)>,
    count: usize,
) -> Result<(u32, u32, u64), AttemptLogError> {
    let mut records = 0_u32;
    let mut bytes = 0_u64;
    for (segment_records, segment_bytes) in segments {
        records = records.checked_add(segment_records).ok_or(AttemptLogError::ArithmeticOverflow)?;
        bytes = bytes.checked_add(segment_bytes).ok_or(AttemptLogError::ArithmeticOverflow)?;
    }
    let chunks = u32::try_from(count).map_err(|_| AttemptLogError::ArithmeticOverflow)?;
    Ok((chunks, records, bytes))
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectKind {
    Record,
    Segment,
    Manifest,
}

pub fn check_schema(actual: &str, expected: &str, kind: ObjectKind) -> Result<(), AttemptLogError> {
    if actual != expected {
        return Err(match kind {
            ObjectKind::Record => AttemptLogError::RecordSchemaUnsupported,
            ObjectKind::Segment => AttemptLogError::SegmentSchemaUnsupported,
            ObjectKind::Manifest => AttemptLogError::ManifestSchemaUnsupported,
        });
    }
    Ok(())
}

pub fn check_object_digest(actual: &str, expected: &str, kind: ObjectKind) -> Result<(), AttemptLogError> {
    if actual != expected {
        return Err(match kind {
            ObjectKind::Record => AttemptLogError::RecordDigestMismatch,
            ObjectKind::Segment => AttemptLogError::SegmentDigestMismatch,
            ObjectKind::Manifest => AttemptLogError::ManifestDigestMismatch,
        });
    }
    Ok(())
}

pub fn check_record_position(sequence: u64, cursor: u64) -> Result<(), AttemptLogError> {
    if sequence != cursor {
        return Err(AttemptLogError::RecordPositionMismatch);
    }
    Ok(())
}

pub fn check_segment_capacity(count: usize, maximum: u32) -> Result<(), AttemptLogError> {
    if count >= maximum as usize {
        return Err(AttemptLogError::ManifestBoundsExceeded);
    }
    Ok(())
}

pub fn check_chain_length(actual: usize, expected: usize) -> Result<(), AttemptLogError> {
    if actual != expected {
        return Err(AttemptLogError::SegmentChainMismatch);
    }
    Ok(())
}

pub fn check_event_binding(
    first_occurrence: bool,
    matches_manifest: impl FnOnce() -> bool,
) -> Result<(), AttemptLogError> {
    if !first_occurrence || !matches_manifest() {
        return Err(AttemptLogError::EventDigestConflict);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentRefFacts<'a> {
    pub index: u64,
    pub segment_digest: &'a str,
    pub previous_segment: Option<&'a str>,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub first_cursor: u64,
    pub next_cursor: u64,
    pub first_previous_record: Option<&'a str>,
    pub head_record: &'a str,
    pub record_count: u32,
    pub payload_bytes: u64,
}

pub fn check_segment_ref(expected: SegmentRefFacts<'_>, actual: SegmentRefFacts<'_>) -> Result<(), AttemptLogError> {
    if expected != actual {
        return Err(AttemptLogError::SegmentChainMismatch);
    }
    Ok(())
}

pub fn checked_add_u32(left: u32, right: u32) -> Result<u32, AttemptLogError> {
    left.checked_add(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn checked_add_u64(left: u64, right: u64) -> Result<u64, AttemptLogError> {
    left.checked_add(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn checked_sub_u32(left: u32, right: u32) -> Result<u32, AttemptLogError> {
    left.checked_sub(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn checked_sub_u64(left: u64, right: u64) -> Result<u64, AttemptLogError> {
    left.checked_sub(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn next_segment_index(previous: Option<u64>) -> Result<u64, AttemptLogError> {
    match previous {
        Some(index) => index.checked_add(1).ok_or(AttemptLogError::ArithmeticOverflow),
        None => Ok(0),
    }
}

pub fn check_policy_identity(expected_digest: &str, actual_digest: &str) -> Result<(), AttemptLogError> {
    if expected_digest != actual_digest {
        return Err(AttemptLogError::PolicyIdentityMismatch);
    }
    Ok(())
}

pub fn check_anchor_digest(actual: &str, expected: &str) -> Result<(), AttemptLogError> {
    if actual != expected {
        return Err(AttemptLogError::RetentionAnchorInvalid);
    }
    Ok(())
}

pub fn validate_digest(value: &str) -> Result<(), AttemptLogError> {
    if !digest_is_valid(value) {
        return Err(AttemptLogError::DigestInvalid);
    }
    Ok(())
}

pub fn check_new_event_identity(inserted_previous: bool) -> Result<(), AttemptLogError> {
    if inserted_previous {
        return Err(AttemptLogError::EventDigestConflict);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionDecision {
    Unchanged,
    Truncate,
}

pub fn classify_retention(drop_count: usize, segment_count: usize) -> Result<RetentionDecision, AttemptLogError> {
    if drop_count == 0 {
        return Ok(RetentionDecision::Unchanged);
    }
    if drop_count >= segment_count {
        return Err(AttemptLogError::RetentionWouldDropAll);
    }
    Ok(RetentionDecision::Truncate)
}

pub fn usize_to_u32(value: usize) -> Result<u32, AttemptLogError> {
    u32::try_from(value).map_err(|_| AttemptLogError::ArithmeticOverflow)
}

pub fn u32_to_usize(value: u32) -> Result<usize, AttemptLogError> {
    usize::try_from(value).map_err(|_| AttemptLogError::ArithmeticOverflow)
}

pub fn replay_record_is_before_cursor(record_cursor: u64, effective_cursor: u64) -> bool {
    record_cursor < effective_cursor
}

pub fn replay_has_more(next_cursor: u64, head_cursor: u64) -> bool {
    next_cursor < head_cursor
}

pub fn next_replay_cursor(last_record_cursor: Option<u64>) -> Result<u64, AttemptLogError> {
    last_record_cursor
        .ok_or(AttemptLogError::ReplayLimitTooSmall)?
        .checked_add(1)
        .ok_or(AttemptLogError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_redaction_and_metadata_fail_closed_at_boundaries() {
        let policy = Policy {
            record_payload_bytes_max: 16,
            segment_record_count_max: 2,
            segment_payload_bytes_max: 32,
            manifest_segment_count_max: 4,
            retained_segment_count_max: 2,
            retained_record_count_max: 4,
            retained_payload_bytes_max: 64,
            replay_record_count_max: 2,
            replay_payload_bytes_max: 32,
            event_identity_count_max: 4,
        };
        assert_eq!(validate_policy(policy), Ok(()));
        assert_eq!(
            validate_policy(Policy {
                segment_payload_bytes_max: 15,
                ..policy
            }),
            Err(AttemptLogError::PolicyInvalid)
        );
        assert_eq!(validate_policy_name("good-policy"), Ok(()));
        assert_eq!(validate_policy_name("bad\npolicy"), Err(AttemptLogError::PolicyInvalid));
        let secret = redact_payload(b"BeArEr secret", policy.record_payload_bytes_max).unwrap();
        assert_eq!(secret.payload.as_slice(), REDACTED_PAYLOAD);
        assert!(secret.secret_redacted);
        let escaped = redact_payload(b"hi\n", policy.record_payload_bytes_max).unwrap();
        assert_eq!(escaped.payload.as_slice(), b"hi\\x0a");
        assert!(escaped.control_escaped);
        assert_eq!(check_record_payload_metadata(6, 6, 16, || true, false, || false), Ok(()));
        assert_eq!(
            check_record_payload_metadata(6, 6, 5, || panic!("hash checked before limit"), false, || false),
            Err(AttemptLogError::RecordPayloadTooLarge)
        );
        assert_eq!(
            check_record_payload_metadata(6, 5, 16, || panic!("hash checked before length"), false, || false),
            Err(AttemptLogError::RecordPayloadMetadataMismatch)
        );
        assert_eq!(
            check_record_payload_metadata(6, 6, 16, || true, true, || false),
            Err(AttemptLogError::RecordPayloadMetadataMismatch)
        );
    }

    #[test]
    fn fence_and_chain_precedence() {
        let expected = Scope {
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 7,
        };
        assert_eq!(
            check_scope(expected, Scope {
                job_id: "other",
                attempt_id: "other",
                fence_generation: 6
            }),
            Err(AttemptLogError::ScopeMismatch)
        );
        assert_eq!(
            check_scope(expected, Scope {
                fence_generation: 6,
                ..expected
            }),
            Err(AttemptLogError::StaleFenceRejected)
        );
        assert_eq!(
            check_scope(expected, Scope {
                fence_generation: 8,
                ..expected
            }),
            Err(AttemptLogError::UnknownFenceRejected)
        );
        assert_eq!(
            check_scope(expected, Scope {
                attempt_id: "other",
                ..expected
            }),
            Err(AttemptLogError::AttemptIdentityMismatch)
        );
        assert_eq!(check_scope(expected, expected), Ok(()));
        let link = ChainLink {
            scope: expected,
            sequence: 4,
            cursor: 4,
            previous_record: Some("record"),
            previous_segment: Some("segment"),
        };
        assert_eq!(check_chain_link(link, link), Ok(()));
        assert_eq!(
            check_chain_link(link, ChainLink {
                previous_segment: None,
                ..link
            }),
            Err(AttemptLogError::SegmentChainMismatch)
        );
    }

    #[test]
    fn cursor_and_page_boundaries() {
        assert_eq!(classify_cursor(3, 7, 2), Ok(CursorDisposition::Truncated));
        assert_eq!(classify_cursor(3, 7, 3), Ok(CursorDisposition::Retained));
        assert_eq!(classify_cursor(3, 7, 7), Ok(CursorDisposition::AtHead));
        assert_eq!(classify_cursor(3, 7, 8), Err(AttemptLogError::CursorAfterHead));
        assert_eq!(classify_cursor(8, 7, 9), Err(AttemptLogError::ManifestSummaryMismatch));
        assert_eq!(admit_replay_record(0, 2, 4, 5, 9), Ok(Some(9)));
        assert_eq!(admit_replay_record(1, 2, 4, 6, 9), Ok(None));
        assert_eq!(admit_replay_record(2, 2, 4, 1, 9), Ok(None));
        assert_eq!(admit_replay_record(2, 2, u64::MAX, 1, u64::MAX), Err(AttemptLogError::ArithmeticOverflow));
    }

    #[test]
    fn append_and_manifest_ref_checks_reject_conflicts_without_reordering() {
        assert_eq!(classify_existing_record("hash", "hash"), Ok(()));
        assert_eq!(classify_existing_record("hash", "different"), Err(AttemptLogError::EventDigestConflict));
        assert_eq!(check_event_capacity(1, 2), Ok(()));
        assert_eq!(check_event_capacity(2, 2), Err(AttemptLogError::EventIdentityCapacityExceeded));
        assert_eq!(check_append_position(8, Some("prev"), 8, 8, Some("prev")), Ok(()));
        assert_eq!(check_append_position(8, Some("prev"), 7, 8, None), Err(AttemptLogError::RecordPositionMismatch));
        assert_eq!(check_append_position(8, Some("prev"), 8, 8, None), Err(AttemptLogError::PreviousRecordMismatch));
        let link = RefLink {
            index: Some(3),
            first_cursor: 8,
            first_sequence: 8,
            previous_record: Some("record"),
            previous_segment: Some("segment"),
        };
        assert_eq!(check_ref_link(link, link), Ok(()));
        assert_eq!(
            check_ref_link(link, RefLink {
                first_sequence: 9,
                ..link
            }),
            Err(AttemptLogError::SegmentChainMismatch)
        );
        assert_eq!(
            check_ref_link(link, RefLink {
                previous_record: None,
                ..link
            }),
            Err(AttemptLogError::SegmentChainMismatch)
        );
        assert_eq!(validate_replay_request(1, 8, 2, 8), Ok(()));
        assert_eq!(validate_replay_request(1, 9, 2, 8), Err(AttemptLogError::ReplayRequestInvalid));
    }

    #[test]
    fn segment_contiguity_and_manifest_anchor_invariants() {
        let scope = Scope {
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 2,
        };
        let mut state = SegmentAccumulator {
            scope,
            expected_sequence: 3,
            expected_cursor: 3,
            previous_record: Some("prev"),
            payload_bytes: 0,
        };
        assert_eq!(state.accept(scope, 3, 3, Some("prev"), "first", 4, 8), Ok(()));
        assert_eq!(state.expected_cursor, 4);
        assert_eq!(
            state.accept(scope, 4, 4, Some("wrong"), "second", 1, 8),
            Err(AttemptLogError::PreviousRecordMismatch)
        );
        assert_eq!(
            state.accept(scope, 5, 4, Some("first"), "second", 1, 8),
            Err(AttemptLogError::SegmentChainMismatch)
        );
        assert_eq!(
            state.accept(scope, 4, 4, Some("first"), "second", 5, 8),
            Err(AttemptLogError::SegmentBoundsExceeded)
        );
        let bounds = ManifestBounds {
            segment_count: 0,
            event_count: 0,
            retained_records: 0,
            retained_bytes: 0,
            retained_start: 0,
            next_cursor: 0,
            head_record_present: false,
            head_segment_present: false,
            anchor_present: false,
        };
        let policy = Policy {
            record_payload_bytes_max: 16,
            segment_record_count_max: 2,
            segment_payload_bytes_max: 32,
            manifest_segment_count_max: 4,
            retained_segment_count_max: 2,
            retained_record_count_max: 4,
            retained_payload_bytes_max: 64,
            replay_record_count_max: 2,
            replay_payload_bytes_max: 32,
            event_identity_count_max: 4,
        };
        assert_eq!(check_manifest_bounds(bounds, policy), Ok(()));
        assert_eq!(
            check_manifest_bounds(
                ManifestBounds {
                    next_cursor: 1,
                    ..bounds
                },
                policy
            ),
            Err(AttemptLogError::ManifestSummaryMismatch)
        );
        assert_eq!(
            check_manifest_bounds(
                ManifestBounds {
                    retained_start: 1,
                    ..bounds
                },
                policy
            ),
            Err(AttemptLogError::ManifestBoundsExceeded)
        );
        let anchor = AnchorFacts {
            schema_matches: true,
            dropped_start: 0,
            dropped_end: 2,
            retained_start: 2,
            dropped_chunks: 2,
            dropped_records: 2,
        };
        assert_eq!(check_anchor(anchor, || true, || true), Ok(()));
        assert_eq!(
            check_anchor(
                AnchorFacts {
                    retained_start: 3,
                    ..anchor
                },
                || true,
                || true
            ),
            Err(AttemptLogError::RetentionAnchorInvalid)
        );
        assert_eq!(
            check_anchor(
                AnchorFacts {
                    schema_matches: false,
                    ..anchor
                },
                || panic!("scope checked before schema"),
                || true
            ),
            Err(AttemptLogError::RetentionAnchorInvalid)
        );
        assert_eq!(dropped_usage([(1, 4), (1, 5)], 2), Ok((2, 2, 9)));
        assert_eq!(dropped_usage([(u32::MAX, 4), (1, 5)], 2), Err(AttemptLogError::ArithmeticOverflow));
    }

    #[test]
    fn retention_accounts_for_oldest_segments_and_rejects_impossible_limits() {
        let limits = RetentionUsage {
            segment_count: 2,
            record_count: 3,
            payload_bytes: 12,
        };
        let segments = [
            RetentionUsage {
                segment_count: 1,
                record_count: 2,
                payload_bytes: 8,
            },
            RetentionUsage {
                segment_count: 1,
                record_count: 1,
                payload_bytes: 5,
            },
            RetentionUsage {
                segment_count: 1,
                record_count: 2,
                payload_bytes: 7,
            },
        ];
        assert_eq!(
            retention_drop_count(
                RetentionUsage {
                    segment_count: 3,
                    record_count: 5,
                    payload_bytes: 20
                },
                limits,
                segments
            ),
            Ok(1)
        );
        let impossible_limits = RetentionUsage {
            segment_count: 0,
            record_count: 0,
            payload_bytes: 0,
        };
        let single_segment = RetentionUsage {
            segment_count: 1,
            record_count: 2,
            payload_bytes: 8,
        };
        let drop_all = retention_drop_count(single_segment, impossible_limits, segments[..1].iter().copied()).unwrap();
        assert_eq!(drop_all, 1);
        assert_eq!(classify_retention(drop_all, 1), Err(AttemptLogError::RetentionWouldDropAll));
        assert_eq!(
            retention_drop_count(
                RetentionUsage {
                    payload_bytes: 7,
                    ..single_segment
                },
                impossible_limits,
                segments[..1].iter().copied(),
            ),
            Err(AttemptLogError::ArithmeticOverflow),
        );
        assert_eq!(check_segment_ref_shape(10, 10, 12, 11, 2), Ok(()));
        assert_eq!(check_segment_ref_shape(10, 10, 12, 12, 2), Err(AttemptLogError::ManifestSummaryMismatch));
        assert_eq!(check_segment_ref_shape(10, u64::MAX, 0, 0, 2), Err(AttemptLogError::ManifestSummaryMismatch));
    }
}
