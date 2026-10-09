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

#[derive(Debug, Clone, Copy)]
pub struct DigestComparison<'a> {
    pub actual: &'a str,
    pub expected: &'a str,
}

pub fn classify_existing_record(digests: DigestComparison<'_>) -> Result<(), AttemptLogError> {
    if digests.expected != digests.actual {
        return Err(AttemptLogError::EventDigestConflict);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct CountLimit {
    pub count: u32,
    pub maximum: u32,
}

pub fn check_event_capacity(limit: CountLimit) -> Result<(), AttemptLogError> {
    if limit.count >= limit.maximum {
        return Err(AttemptLogError::EventIdentityCapacityExceeded);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct AppendPosition<'a> {
    pub expected_cursor: u64,
    pub expected_previous: Option<&'a str>,
    pub sequence: u64,
    pub cursor: u64,
    pub previous: Option<&'a str>,
}

pub fn check_append_position(position: AppendPosition<'_>) -> Result<(), AttemptLogError> {
    if position.sequence != position.expected_cursor || position.cursor != position.expected_cursor {
        return Err(AttemptLogError::RecordPositionMismatch);
    }
    if position.previous != position.expected_previous {
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
    if expected != observed {
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
    if expected != observed {
        return Err(AttemptLogError::SegmentChainMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct SegmentRefShape {
    pub first_sequence: u64,
    pub first_cursor: u64,
    pub next_cursor: u64,
    pub last_sequence: u64,
    pub record_count: u32,
}

pub fn check_segment_ref_shape(shape: SegmentRefShape) -> Result<(), AttemptLogError> {
    if shape.record_count == 0 || shape.first_sequence != shape.first_cursor {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    let expected_next = shape
        .first_cursor
        .checked_add(u64::from(shape.record_count))
        .ok_or(AttemptLogError::ArithmeticOverflow)?;
    let expected_last = expected_next.checked_sub(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
    if shape.next_cursor != expected_next || shape.last_sequence != expected_last {
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

#[derive(Debug, Clone, Copy)]
pub struct CursorRange {
    pub retained_start: u64,
    pub next: u64,
}

pub fn classify_cursor(range: CursorRange, requested: u64) -> Result<CursorDisposition, AttemptLogError> {
    if range.retained_start > range.next {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if requested > range.next {
        return Err(AttemptLogError::CursorAfterHead);
    }
    if requested < range.retained_start {
        return Ok(CursorDisposition::Truncated);
    }
    if requested == range.next {
        return Ok(CursorDisposition::AtHead);
    }
    Ok(CursorDisposition::Retained)
}

pub fn validate_replay_request(count: u32, bytes: u64, max_count: u32, max_bytes: u64) -> Result<(), AttemptLogError> {
    if count == 0 || bytes == 0 {
        return Err(AttemptLogError::ReplayRequestInvalid);
    }
    if count > max_count || bytes > max_bytes {
        return Err(AttemptLogError::ReplayRequestInvalid);
    }
    Ok(())
}

/// Returns the new payload total when this record fits, or `None` when the page ends.
#[derive(Debug, Clone, Copy)]
pub struct ReplayAdmission {
    pub record_count: u32,
    pub record_count_max: u32,
    pub payload_bytes: u64,
    pub record_bytes: u32,
    pub payload_bytes_max: u64,
}

pub fn admit_replay_record(admission: ReplayAdmission) -> Result<Option<u64>, AttemptLogError> {
    let projected = admission
        .payload_bytes
        .checked_add(u64::from(admission.record_bytes))
        .ok_or(AttemptLogError::ArithmeticOverflow)?;
    if admission.record_count >= admission.record_count_max || projected > admission.payload_bytes_max {
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
) -> Result<u32, AttemptLogError> {
    let mut segments = segments.into_iter();
    let mut drop_count = 0_u32;
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
    check_policy_minimums(policy)?;
    check_policy_maximums(policy)?;
    check_policy_relationships(policy)?;
    Ok(())
}

fn check_policy_minimums(policy: Policy) -> Result<(), AttemptLogError> {
    if policy.record_payload_bytes_max < REDACTED_PAYLOAD.len() as u32 || policy.segment_record_count_max == 0 {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.segment_payload_bytes_max == 0 || policy.manifest_segment_count_max == 0 {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.retained_segment_count_max == 0 || policy.retained_record_count_max == 0 {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.retained_payload_bytes_max == 0 || policy.replay_record_count_max == 0 {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.replay_payload_bytes_max == 0 || policy.event_identity_count_max == 0 {
        return Err(AttemptLogError::PolicyInvalid);
    }
    Ok(())
}

fn check_policy_maximums(policy: Policy) -> Result<(), AttemptLogError> {
    if policy.record_payload_bytes_max > MAX_RECORD_PAYLOAD_BYTES
        || policy.segment_record_count_max > MAX_SEGMENT_RECORDS
    {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.segment_payload_bytes_max > MAX_SEGMENT_PAYLOAD_BYTES
        || policy.manifest_segment_count_max > MAX_MANIFEST_SEGMENTS
    {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.retained_record_count_max > MAX_RETAINED_RECORDS
        || policy.retained_payload_bytes_max > MAX_RETAINED_PAYLOAD_BYTES
    {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.replay_record_count_max > MAX_REPLAY_RECORDS || policy.replay_payload_bytes_max > MAX_REPLAY_PAYLOAD_BYTES
    {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.event_identity_count_max > MAX_EVENT_IDENTITIES {
        return Err(AttemptLogError::PolicyInvalid);
    }
    debug_assert!(policy.record_payload_bytes_max <= MAX_RECORD_PAYLOAD_BYTES);
    debug_assert!(policy.event_identity_count_max <= MAX_EVENT_IDENTITIES);
    Ok(())
}

fn check_policy_relationships(policy: Policy) -> Result<(), AttemptLogError> {
    if policy.retained_segment_count_max > policy.manifest_segment_count_max
        || u64::from(policy.record_payload_bytes_max) > policy.segment_payload_bytes_max
    {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.segment_record_count_max > policy.retained_record_count_max
        || policy.segment_payload_bytes_max > policy.retained_payload_bytes_max
    {
        return Err(AttemptLogError::PolicyInvalid);
    }
    if policy.replay_record_count_max > policy.retained_record_count_max
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
    if payload.len() > usize::try_from(MAX_RECORD_PAYLOAD_BYTES).map_err(|_| AttemptLogError::ArithmeticOverflow)? {
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
    let max = usize::try_from(payload_bytes_max).map_err(|_| AttemptLogError::ArithmeticOverflow)?;
    let mut rendered = alloc::vec::Vec::with_capacity(payload.len().min(max));
    let mut is_control_escaped = false;
    let mut is_payload_truncated = false;
    for byte in payload {
        if byte.is_ascii_control() {
            is_control_escaped = true;
            const HEX: &[u8; 16] = b"0123456789abcdef";
            if rendered.len().saturating_add(ESCAPE_BYTES) > max {
                is_payload_truncated = true;
                break;
            }
            rendered.extend_from_slice(&[b'\\', b'x', HEX[usize::from(byte >> 4)], HEX[usize::from(byte & 0x0f)]]);
            continue;
        }
        if rendered.len() >= max {
            is_payload_truncated = true;
            break;
        }
        rendered.push(*byte);
    }
    if rendered.len() < payload.len() && !is_control_escaped {
        is_payload_truncated = true;
    }
    debug_assert!(rendered.len() <= max);
    debug_assert!(!is_control_escaped || is_payload_truncated || rendered.contains(&b'\\'));
    Ok(RedactedPayload {
        payload: rendered,
        secret_redacted: false,
        control_escaped: is_control_escaped,
        payload_truncated: is_payload_truncated,
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

#[derive(Debug, Clone, Copy)]
pub struct PayloadMetadata {
    pub actual_length_bytes: u32,
    pub declared_length_bytes: u32,
    pub maximum_bytes: u32,
    pub secret_redacted: bool,
}

pub fn check_record_payload_metadata(
    metadata: PayloadMetadata,
    hash_matches: impl FnOnce() -> bool,
    actual_redacted: impl FnOnce() -> bool,
) -> Result<(), AttemptLogError> {
    if metadata.actual_length_bytes > metadata.maximum_bytes {
        return Err(AttemptLogError::RecordPayloadTooLarge);
    }
    if metadata.actual_length_bytes != metadata.declared_length_bytes || !hash_matches() {
        return Err(AttemptLogError::RecordPayloadMetadataMismatch);
    }
    if metadata.secret_redacted && !actual_redacted() {
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

pub fn check_segment_record_count(limit: CountLimit) -> Result<u32, AttemptLogError> {
    if limit.count > limit.maximum {
        return Err(AttemptLogError::SegmentBoundsExceeded);
    }
    Ok(limit.count)
}

pub struct SegmentAccumulator<'a> {
    pub scope: Scope<'a>,
    pub expected_sequence: u64,
    pub expected_cursor: u64,
    pub previous_record: Option<&'a str>,
    pub payload_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct AccumulatedRecord<'a> {
    pub scope: Scope<'a>,
    pub sequence: u64,
    pub cursor: u64,
    pub previous_record: Option<&'a str>,
    pub record_digest: &'a str,
    pub record_bytes: u32,
}

impl<'a> SegmentAccumulator<'a> {
    pub fn accept(&mut self, record: AccumulatedRecord<'a>, maximum_payload_bytes: u64) -> Result<(), AttemptLogError> {
        if record.scope != self.scope
            || record.sequence != self.expected_sequence
            || record.cursor != self.expected_cursor
        {
            return Err(AttemptLogError::SegmentChainMismatch);
        }
        if record.previous_record != self.previous_record {
            return Err(AttemptLogError::PreviousRecordMismatch);
        }
        let payload_bytes = self
            .payload_bytes
            .checked_add(u64::from(record.record_bytes))
            .ok_or(AttemptLogError::ArithmeticOverflow)?;
        if payload_bytes > maximum_payload_bytes {
            return Err(AttemptLogError::SegmentBoundsExceeded);
        }
        let next_sequence = self.expected_sequence.checked_add(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
        let next_cursor = self.expected_cursor.checked_add(1).ok_or(AttemptLogError::ArithmeticOverflow)?;
        self.payload_bytes = payload_bytes;
        self.expected_sequence = next_sequence;
        self.expected_cursor = next_cursor;
        self.previous_record = Some(record.record_digest);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestBounds {
    pub segment_count: u32,
    pub event_count: u32,
    pub retained_records: u32,
    pub retained_bytes: u64,
    pub retained_start: u64,
    pub next_cursor: u64,
    pub head_record_present: bool,
    pub head_segment_present: bool,
    pub anchor_present: bool,
}

pub fn check_manifest_bounds(facts: ManifestBounds, policy: Policy) -> Result<(), AttemptLogError> {
    if facts.segment_count > policy.manifest_segment_count_max || facts.event_count > policy.event_identity_count_max {
        return Err(AttemptLogError::ManifestBoundsExceeded);
    }
    if facts.retained_records > MAX_RETAINED_RECORDS || facts.retained_bytes > MAX_RETAINED_PAYLOAD_BYTES {
        return Err(AttemptLogError::ManifestBoundsExceeded);
    }
    if facts.retained_start > facts.next_cursor {
        return Err(AttemptLogError::ManifestBoundsExceeded);
    }
    if facts.segment_count == 0 && (facts.retained_records != 0 || facts.retained_bytes != 0) {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if facts.segment_count == 0 && (facts.head_record_present || facts.head_segment_present) {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if facts.segment_count == 0 && (facts.retained_start != facts.next_cursor || facts.anchor_present) {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    debug_assert!(facts.retained_start <= facts.next_cursor);
    debug_assert!(facts.segment_count <= policy.manifest_segment_count_max);
    Ok(())
}

pub fn check_manifest_event_count(limit: CountLimit) -> Result<(), AttemptLogError> {
    if limit.count > limit.maximum {
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
    if actual.record_count != expected.record_count || actual.payload_bytes != expected.payload_bytes {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if actual.next_cursor != expected.next_cursor || actual.head_segment != expected.head_segment {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if actual.head_record != expected.head_record {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    Ok(())
}

pub fn check_chain_totals(expected: ManifestTotals<'_>, actual: ManifestTotals<'_>) -> Result<(), AttemptLogError> {
    if actual.next_cursor != expected.next_cursor || actual.segment_count != expected.segment_count {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if actual.record_count != expected.record_count || actual.payload_bytes != expected.payload_bytes {
        return Err(AttemptLogError::ManifestSummaryMismatch);
    }
    if actual.head_record != expected.head_record || actual.head_segment != expected.head_segment {
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
    if !facts.schema_matches || !scope_matches() {
        return Err(AttemptLogError::RetentionAnchorInvalid);
    }
    if !policy_matches() || facts.dropped_start >= facts.dropped_end {
        return Err(AttemptLogError::RetentionAnchorInvalid);
    }
    if facts.dropped_end != facts.retained_start || facts.dropped_chunks == 0 {
        return Err(AttemptLogError::RetentionAnchorInvalid);
    }
    if facts.dropped_records == 0 {
        return Err(AttemptLogError::RetentionAnchorInvalid);
    }
    Ok(())
}

pub fn digest_is_valid(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub fn dropped_usage(
    segments: impl IntoIterator<Item = (u32, u64)>,
    count: u32,
) -> Result<(u32, u32, u64), AttemptLogError> {
    let mut records = 0_u32;
    let mut bytes = 0_u64;
    for (segment_records, segment_bytes) in segments {
        records = records.checked_add(segment_records).ok_or(AttemptLogError::ArithmeticOverflow)?;
        bytes = bytes.checked_add(segment_bytes).ok_or(AttemptLogError::ArithmeticOverflow)?;
    }
    Ok((count, records, bytes))
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectKind {
    Record,
    Segment,
    Manifest,
}

pub fn check_schema(digests: DigestComparison<'_>, kind: ObjectKind) -> Result<(), AttemptLogError> {
    if digests.actual != digests.expected {
        return Err(match kind {
            ObjectKind::Record => AttemptLogError::RecordSchemaUnsupported,
            ObjectKind::Segment => AttemptLogError::SegmentSchemaUnsupported,
            ObjectKind::Manifest => AttemptLogError::ManifestSchemaUnsupported,
        });
    }
    Ok(())
}

pub fn check_object_digest(digests: DigestComparison<'_>, kind: ObjectKind) -> Result<(), AttemptLogError> {
    if digests.actual != digests.expected {
        return Err(match kind {
            ObjectKind::Record => AttemptLogError::RecordDigestMismatch,
            ObjectKind::Segment => AttemptLogError::SegmentDigestMismatch,
            ObjectKind::Manifest => AttemptLogError::ManifestDigestMismatch,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct RecordPosition {
    pub sequence: u64,
    pub cursor: u64,
}

pub fn check_record_position(position: RecordPosition) -> Result<(), AttemptLogError> {
    if position.sequence != position.cursor {
        return Err(AttemptLogError::RecordPositionMismatch);
    }
    Ok(())
}

pub fn check_segment_capacity(limit: CountLimit) -> Result<(), AttemptLogError> {
    if limit.count >= limit.maximum {
        return Err(AttemptLogError::ManifestBoundsExceeded);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct CountComparison {
    pub actual: u32,
    pub expected: u32,
}

pub fn check_chain_length(lengths: CountComparison) -> Result<(), AttemptLogError> {
    if lengths.actual != lengths.expected {
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

pub fn checked_add_u32((left, right): (u32, u32)) -> Result<u32, AttemptLogError> {
    left.checked_add(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn checked_add_u64((left, right): (u64, u64)) -> Result<u64, AttemptLogError> {
    left.checked_add(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn checked_sub_u32((left, right): (u32, u32)) -> Result<u32, AttemptLogError> {
    left.checked_sub(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn checked_sub_u64((left, right): (u64, u64)) -> Result<u64, AttemptLogError> {
    left.checked_sub(right).ok_or(AttemptLogError::ArithmeticOverflow)
}

pub fn next_segment_index(previous: Option<u64>) -> Result<u64, AttemptLogError> {
    match previous {
        Some(index) => index.checked_add(1).ok_or(AttemptLogError::ArithmeticOverflow),
        None => Ok(0),
    }
}

pub fn check_policy_identity(digests: DigestComparison<'_>) -> Result<(), AttemptLogError> {
    if digests.expected != digests.actual {
        return Err(AttemptLogError::PolicyIdentityMismatch);
    }
    Ok(())
}

pub fn check_anchor_digest(digests: DigestComparison<'_>) -> Result<(), AttemptLogError> {
    if digests.actual != digests.expected {
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

pub fn classify_retention(limit: CountLimit) -> Result<RetentionDecision, AttemptLogError> {
    if limit.count == 0 {
        return Ok(RetentionDecision::Unchanged);
    }
    if limit.count >= limit.maximum {
        return Err(AttemptLogError::RetentionWouldDropAll);
    }
    Ok(RetentionDecision::Truncate)
}

#[derive(Debug, Clone, Copy)]
pub struct ReplayPosition {
    pub record_cursor: u64,
    pub effective_cursor: u64,
}

pub fn replay_record_is_before_cursor(position: ReplayPosition) -> bool {
    position.record_cursor < position.effective_cursor
}

#[derive(Debug, Clone, Copy)]
pub struct ReplayHead {
    pub next_cursor: u64,
    pub head_cursor: u64,
}

pub fn replay_has_more(head: ReplayHead) -> bool {
    head.next_cursor < head.head_cursor
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
        let metadata = PayloadMetadata {
            actual_length_bytes: 6,
            declared_length_bytes: 6,
            maximum_bytes: 16,
            secret_redacted: false,
        };
        assert_eq!(check_record_payload_metadata(metadata, || true, || false), Ok(()));
        assert_eq!(
            check_record_payload_metadata(
                PayloadMetadata {
                    maximum_bytes: 5,
                    ..metadata
                },
                || panic!("hash checked before limit"),
                || false,
            ),
            Err(AttemptLogError::RecordPayloadTooLarge)
        );
        assert_eq!(
            check_record_payload_metadata(
                PayloadMetadata {
                    declared_length_bytes: 5,
                    ..metadata
                },
                || panic!("hash checked before length"),
                || false,
            ),
            Err(AttemptLogError::RecordPayloadMetadataMismatch)
        );
        assert_eq!(
            check_record_payload_metadata(
                PayloadMetadata {
                    secret_redacted: true,
                    ..metadata
                },
                || true,
                || false
            ),
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
        let range = CursorRange {
            retained_start: 3,
            next: 7,
        };
        assert_eq!(classify_cursor(range, 2), Ok(CursorDisposition::Truncated));
        assert_eq!(classify_cursor(range, 3), Ok(CursorDisposition::Retained));
        assert_eq!(classify_cursor(range, 7), Ok(CursorDisposition::AtHead));
        assert_eq!(classify_cursor(range, 8), Err(AttemptLogError::CursorAfterHead));
        assert_eq!(
            classify_cursor(
                CursorRange {
                    retained_start: 8,
                    next: 7
                },
                9
            ),
            Err(AttemptLogError::ManifestSummaryMismatch)
        );
        let admission = ReplayAdmission {
            record_count: 0,
            record_count_max: 2,
            payload_bytes: 4,
            record_bytes: 5,
            payload_bytes_max: 9,
        };
        assert_eq!(admit_replay_record(admission), Ok(Some(9)));
        assert_eq!(
            admit_replay_record(ReplayAdmission {
                record_count: 1,
                record_bytes: 6,
                ..admission
            }),
            Ok(None)
        );
        assert_eq!(
            admit_replay_record(ReplayAdmission {
                record_count: 2,
                record_bytes: 1,
                ..admission
            }),
            Ok(None)
        );
        assert_eq!(
            admit_replay_record(ReplayAdmission {
                record_count: 2,
                payload_bytes: u64::MAX,
                record_bytes: 1,
                payload_bytes_max: u64::MAX,
                ..admission
            }),
            Err(AttemptLogError::ArithmeticOverflow)
        );
    }

    #[test]
    fn append_and_manifest_ref_checks_reject_conflicts_without_reordering() {
        let matching = DigestComparison {
            actual: "hash",
            expected: "hash",
        };
        assert_eq!(classify_existing_record(matching), Ok(()));
        assert_eq!(
            classify_existing_record(DigestComparison {
                actual: "different",
                ..matching
            }),
            Err(AttemptLogError::EventDigestConflict)
        );
        assert_eq!(check_event_capacity(CountLimit { count: 1, maximum: 2 }), Ok(()));
        assert_eq!(
            check_event_capacity(CountLimit { count: 2, maximum: 2 }),
            Err(AttemptLogError::EventIdentityCapacityExceeded)
        );
        let position = AppendPosition {
            expected_cursor: 8,
            expected_previous: Some("prev"),
            sequence: 8,
            cursor: 8,
            previous: Some("prev"),
        };
        assert_eq!(check_append_position(position), Ok(()));
        assert_eq!(
            check_append_position(AppendPosition {
                sequence: 7,
                previous: None,
                ..position
            }),
            Err(AttemptLogError::RecordPositionMismatch)
        );
        assert_eq!(
            check_append_position(AppendPosition {
                previous: None,
                ..position
            }),
            Err(AttemptLogError::PreviousRecordMismatch)
        );
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
        let first = AccumulatedRecord {
            scope,
            sequence: 3,
            cursor: 3,
            previous_record: Some("prev"),
            record_digest: "first",
            record_bytes: 4,
        };
        assert_eq!(state.accept(first, 8), Ok(()));
        assert_eq!(state.expected_cursor, 4);
        let second = AccumulatedRecord {
            sequence: 4,
            cursor: 4,
            previous_record: Some("first"),
            record_digest: "second",
            record_bytes: 1,
            ..first
        };
        assert_eq!(
            state.accept(
                AccumulatedRecord {
                    previous_record: Some("wrong"),
                    ..second
                },
                8
            ),
            Err(AttemptLogError::PreviousRecordMismatch)
        );
        assert_eq!(
            state.accept(AccumulatedRecord { sequence: 5, ..second }, 8),
            Err(AttemptLogError::SegmentChainMismatch)
        );
        assert_eq!(
            state.accept(
                AccumulatedRecord {
                    record_bytes: 5,
                    ..second
                },
                8
            ),
            Err(AttemptLogError::SegmentBoundsExceeded)
        );
        assert_eq!(state.payload_bytes, 4);
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
        assert_eq!(
            classify_retention(CountLimit {
                count: drop_all,
                maximum: 1
            }),
            Err(AttemptLogError::RetentionWouldDropAll)
        );
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
        let shape = SegmentRefShape {
            first_sequence: 10,
            first_cursor: 10,
            next_cursor: 12,
            last_sequence: 11,
            record_count: 2,
        };
        assert_eq!(check_segment_ref_shape(shape), Ok(()));
        assert_eq!(
            check_segment_ref_shape(SegmentRefShape {
                last_sequence: 12,
                ..shape
            }),
            Err(AttemptLogError::ManifestSummaryMismatch)
        );
        assert_eq!(
            check_segment_ref_shape(SegmentRefShape {
                first_cursor: u64::MAX,
                next_cursor: 0,
                last_sequence: 0,
                ..shape
            }),
            Err(AttemptLogError::ManifestSummaryMismatch)
        );
    }
}
