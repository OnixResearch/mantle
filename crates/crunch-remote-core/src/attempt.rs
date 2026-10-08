//! Pure durable remote-attempt identity, fencing, transition, and retry decisions.
//!
//! This module is the functional core. It performs no clock reads, persistence,
//! transport, cancellation, logging, or output admission.
//!
//! r[impl remote_builds.durable_attempt_fencing]
//! r[impl remote_builds.idempotent_attempt_reporting]
//! r[impl remote_builds.pure_attempt_decisions]

use alloc::collections::BTreeMap;
#[cfg(test)]
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

pub const MAX_REMOTE_ATTEMPT_EVENTS: usize = 1_024;
pub const MAX_REMOTE_ATTEMPTS: u32 = 32;
pub const MAX_REMOTE_ATTEMPT_ID_BYTES: usize = 128;
pub const MAX_REMOTE_ATTEMPT_PAYLOAD_BYTES: usize = 1_048_576;
pub const MAX_REMOTE_RETRY_DELAY_SECS: u64 = 3_600;
pub const MAX_REMOTE_ATTEMPT_TIMEOUT_SECS: u64 = 86_400;
const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const INITIAL_REMOTE_FENCE_GENERATION: u64 = 1;
const INITIAL_REMOTE_ATTEMPT_COUNT: u32 = 1;
const DEFAULT_REMOTE_MAX_ATTEMPTS: u32 = 3;
const DEFAULT_REMOTE_RETRY_DELAY_SECS: u64 = 5;
const DEFAULT_REMOTE_ATTEMPT_TIMEOUT_SECS: u64 = 3_600;
const REMOTE_ATTEMPT_ID_DOMAIN: &str = "mantle-remote-attempt-v1";
const REMOTE_ATTEMPT_PAYLOAD_DOMAIN: &str = "mantle-remote-attempt-payload-v1";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct RemoteJobId(String);

impl RemoteJobId {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteAttemptReasonCode> {
        bounded_identity(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct RemoteAttemptId(String);

impl RemoteAttemptId {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteAttemptReasonCode> {
        bounded_identity(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct RemoteAssignmentNonce(String);

impl RemoteAssignmentNonce {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteAttemptReasonCode> {
        let value = value.into();
        if !is_blake3_hex_digest(&value) {
            return Err(RemoteAttemptReasonCode::AssignmentNonceInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "serde")]
fn legacy_missing_assignment_nonce() -> RemoteAssignmentNonce {
    RemoteAssignmentNonce(String::new())
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct RemoteEventId(String);

impl RemoteEventId {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteAttemptReasonCode> {
        bounded_identity(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct RemotePayloadDigest(String);

impl RemotePayloadDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteAttemptReasonCode> {
        let value = value.into();
        if !is_blake3_hex_digest(&value) {
            return Err(RemoteAttemptReasonCode::PayloadDigestInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct RemoteFenceGeneration(u64);

impl RemoteFenceGeneration {
    pub const INITIAL: Self = Self(INITIAL_REMOTE_FENCE_GENERATION);

    pub fn new(value: u64) -> Result<Self, RemoteAttemptReasonCode> {
        if value == 0 {
            return Err(RemoteAttemptReasonCode::FenceInvalid);
        }
        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn advance(self) -> Result<Self, RemoteAttemptReasonCode> {
        let next = self.0.checked_add(1).ok_or(RemoteAttemptReasonCode::FenceExhausted)?;
        debug_assert!(next > self.0);
        debug_assert!(next != 0);
        Ok(Self(next))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum RemoteAttemptPhase {
    Queued,
    Running,
    Transferring,
    FinishedUndelivered,
    Completed,
    Failed,
    Superseded,
}

impl RemoteAttemptPhase {
    pub fn is_live(self) -> bool {
        matches!(self, Self::Queued | Self::Running | Self::Transferring | Self::FinishedUndelivered)
    }

    pub fn is_terminal(self) -> bool {
        !self.is_live()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum RemoteAttemptFailureClass {
    Retryable,
    Terminal,
    PolicyDenied,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum RemoteAttemptReportKind {
    Start,
    Heartbeat,
    LogAppend,
    TransferCheckpoint,
    ResultReady,
    Failure,
    Completion,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "kebab-case"))]
pub enum RemoteAttemptReportPayload {
    Start,
    Heartbeat {
        observed_unix_s: u64,
    },
    LogAppend {
        cursor: u64,
        bytes: String,
    },
    TransferCheckpoint {
        checkpoint: u64,
        transferred_bytes: u64,
    },
    ResultReady {
        output_digest_blake3: String,
    },
    Failure {
        failure_class: RemoteAttemptFailureClass,
        reason_code: RemoteAttemptReasonCode,
    },
    Completion {
        output_digest_blake3: String,
    },
}

impl RemoteAttemptReportPayload {
    pub fn kind(&self) -> RemoteAttemptReportKind {
        match self {
            Self::Start => RemoteAttemptReportKind::Start,
            Self::Heartbeat { .. } => RemoteAttemptReportKind::Heartbeat,
            Self::LogAppend { .. } => RemoteAttemptReportKind::LogAppend,
            Self::TransferCheckpoint { .. } => RemoteAttemptReportKind::TransferCheckpoint,
            Self::ResultReady { .. } => RemoteAttemptReportKind::ResultReady,
            Self::Failure { .. } => RemoteAttemptReportKind::Failure,
            Self::Completion { .. } => RemoteAttemptReportKind::Completion,
        }
    }

    fn output_digest(&self) -> Option<&str> {
        match self {
            Self::ResultReady { output_digest_blake3 } | Self::Completion { output_digest_blake3 } => {
                Some(output_digest_blake3)
            }
            _ => None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptReportIdentity {
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub event_id: RemoteEventId,
    pub payload_digest: RemotePayloadDigest,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptReport {
    pub identity: RemoteAttemptReportIdentity,
    pub payload: RemoteAttemptReportPayload,
}

impl RemoteAttemptReport {
    pub fn new(
        job_id: RemoteJobId,
        attempt_id: RemoteAttemptId,
        fence_generation: RemoteFenceGeneration,
        event_id: RemoteEventId,
        payload: RemoteAttemptReportPayload,
    ) -> Result<Self, RemoteAttemptReasonCode> {
        let payload_digest = canonical_remote_attempt_payload_digest(&payload)?;
        Ok(Self {
            identity: RemoteAttemptReportIdentity {
                job_id,
                attempt_id,
                fence_generation,
                event_id,
                payload_digest,
            },
            payload,
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptState {
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    #[cfg_attr(feature = "serde", serde(default = "legacy_missing_assignment_nonce"))]
    pub assignment_nonce: RemoteAssignmentNonce,
    pub fence_generation: RemoteFenceGeneration,
    pub phase: RemoteAttemptPhase,
    pub attempts_started: u32,
    pub started_unix_s: u64,
    pub deadline_unix_s: u64,
    pub applied_events: BTreeMap<RemoteEventId, RemotePayloadDigest>,
    pub transfer_checkpoint: Option<u64>,
    pub transferred_bytes: u64,
    pub last_heartbeat_unix_s: Option<u64>,
    pub result_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinatorAttemptPhase {
    Queued,
    Running,
    Finished,
    Lost,
}

impl CoordinatorAttemptPhase {
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

pub const fn persisted_missing_coordinator_attempt_is_valid(
    phase: CoordinatorAttemptPhase,
    last_attempt_reason: Option<RemoteAttemptReasonCode>,
) -> bool {
    matches!(phase, CoordinatorAttemptPhase::Lost)
        && matches!(
            last_attempt_reason,
            Some(RemoteAttemptReasonCode::LegacyStateRejected | RemoteAttemptReasonCode::DurableStateInvalid)
        )
}

pub const fn coordinator_phase_for_attempt(phase: RemoteAttemptPhase) -> CoordinatorAttemptPhase {
    match phase {
        RemoteAttemptPhase::Queued => CoordinatorAttemptPhase::Queued,
        RemoteAttemptPhase::Running | RemoteAttemptPhase::Transferring => CoordinatorAttemptPhase::Running,
        RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed => CoordinatorAttemptPhase::Finished,
        RemoteAttemptPhase::Failed | RemoteAttemptPhase::Superseded => CoordinatorAttemptPhase::Lost,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoordinatorAttemptProjectionFacts {
    pub phase: CoordinatorAttemptPhase,
    pub result_available: bool,
    pub output_admission_completed: bool,
    pub transfer_checkpoint: Option<u64>,
    pub transferred_bytes: u64,
}

pub fn persisted_coordinator_attempt_projection_matches(
    facts: CoordinatorAttemptProjectionFacts,
    attempt: &RemoteAttemptState,
) -> bool {
    facts.phase == coordinator_phase_for_attempt(attempt.phase)
        && facts.result_available == (attempt.phase == RemoteAttemptPhase::Completed)
        && facts.output_admission_completed
            == matches!(attempt.phase, RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed)
        && facts.transfer_checkpoint == attempt.transfer_checkpoint
        && facts.transferred_bytes == attempt.transferred_bytes
}

/// Validate a decoded durable attempt before trusting its coordinator projection.
/// Decoding a transparent identifier bypasses its constructor, so every stored
/// identity and the derived worker/fence binding must be checked again.
pub fn persisted_remote_attempt_is_valid(
    expected_job_id: &RemoteJobId,
    worker_endpoint_id: Option<&str>,
    attempt: &RemoteAttemptState,
) -> bool {
    if &attempt.job_id != expected_job_id
        || !persisted_identity_is_valid(attempt.job_id.as_str())
        || !persisted_identity_is_valid(attempt.attempt_id.as_str())
        || !is_blake3_hex_digest(attempt.assignment_nonce.as_str())
        || attempt.fence_generation.get() == 0
        || attempt.attempts_started == 0
        || attempt.attempts_started > MAX_REMOTE_ATTEMPTS
    {
        return false;
    }
    let Some(worker_endpoint_id) = worker_endpoint_id else {
        return false;
    };
    if !derive_remote_attempt_id(
        &attempt.job_id,
        &attempt.assignment_nonce,
        attempt.fence_generation,
        worker_endpoint_id,
    )
    .is_ok_and(|derived| derived == attempt.attempt_id)
    {
        return false;
    }
    let result_is_valid = match attempt.phase {
        RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed => {
            attempt.result_digest_blake3.as_deref().is_some_and(is_blake3_hex_digest)
        }
        RemoteAttemptPhase::Queued
        | RemoteAttemptPhase::Running
        | RemoteAttemptPhase::Transferring
        | RemoteAttemptPhase::Failed
        | RemoteAttemptPhase::Superseded => attempt.result_digest_blake3.is_none(),
    };
    result_is_valid
        && attempt.started_unix_s < attempt.deadline_unix_s
        && attempt.applied_events.len() <= MAX_REMOTE_ATTEMPT_EVENTS
        && attempt
            .last_heartbeat_unix_s
            .is_none_or(|observed| observed >= attempt.started_unix_s && observed <= attempt.deadline_unix_s)
        && attempt
            .applied_events
            .iter()
            .all(|(event, digest)| persisted_identity_is_valid(event.as_str()) && is_blake3_hex_digest(digest.as_str()))
}

fn persisted_identity_is_valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_REMOTE_ATTEMPT_ID_BYTES && !value.chars().any(char::is_control)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAttemptRetryPolicy {
    pub max_attempts: u32,
    pub retry_delay_secs: u64,
    pub attempt_timeout_secs: u64,
}

impl Default for RemoteAttemptRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_REMOTE_MAX_ATTEMPTS,
            retry_delay_secs: DEFAULT_REMOTE_RETRY_DELAY_SECS,
            attempt_timeout_secs: DEFAULT_REMOTE_ATTEMPT_TIMEOUT_SECS,
        }
    }
}

impl RemoteAttemptRetryPolicy {
    pub fn validate(self) -> Result<(), RemoteAttemptReasonCode> {
        if self.max_attempts == 0 || self.max_attempts > MAX_REMOTE_ATTEMPTS {
            return Err(RemoteAttemptReasonCode::RetryPolicyInvalid);
        }
        if self.retry_delay_secs > MAX_REMOTE_RETRY_DELAY_SECS {
            return Err(RemoteAttemptReasonCode::RetryPolicyInvalid);
        }
        if self.attempt_timeout_secs == 0 || self.attempt_timeout_secs > MAX_REMOTE_ATTEMPT_TIMEOUT_SECS {
            return Err(RemoteAttemptReasonCode::RetryPolicyInvalid);
        }
        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAttemptTimeFacts {
    pub now_unix_s: u64,
    pub failure_observed_unix_s: u64,
    pub overall_deadline_unix_s: u64,
}

#[derive(Debug)]
pub struct RemoteAttemptAssignmentInput<'a> {
    pub job_id: &'a RemoteJobId,
    pub worker_endpoint_id: &'a str,
    pub assignment_nonce: RemoteAssignmentNonce,
    pub previous: Option<&'a RemoteAttemptState>,
    pub retry_policy: RemoteAttemptRetryPolicy,
    pub time: RemoteAttemptTimeFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAttemptAuthorizationFacts {
    pub worker_authorized: bool,
    pub output_admission_authorized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFenceDisposition {
    Current,
    Stale,
    UnknownFuture,
    JobMismatch,
    AttemptMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteEventDisposition {
    New,
    AlreadyApplied,
    Conflict,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum RemoteAttemptApplyDisposition {
    Applied,
    AlreadyApplied,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAttemptApplyPlan {
    pub disposition: RemoteAttemptApplyDisposition,
    pub reason_code: RemoteAttemptReasonCode,
    pub next_state: RemoteAttemptState,
    pub output_admission_allowed: bool,
    /// A proposed persistence effect, never evidence that it succeeded.
    pub pending_effect: Option<crate::effect::EffectKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAttemptTransitionDecision {
    pub next_phase: Option<RemoteAttemptPhase>,
    pub reason_code: RemoteAttemptReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteAttemptRetryDecision {
    pub retry_allowed: bool,
    pub not_before_unix_s: Option<u64>,
    pub reason_code: RemoteAttemptReasonCode,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum RemoteAttemptReasonCode {
    CurrentAttemptApplied,
    CurrentAttemptCompleted,
    AlreadyApplied,
    EventDigestConflict,
    EventRetentionExhausted,
    PayloadDigestInvalid,
    PayloadDigestMismatch,
    PayloadSerializationFailed,
    PayloadTooLarge,
    IdentityInvalid,
    AssignmentNonceInvalid,
    FenceInvalid,
    FenceExhausted,
    StaleReportRejected,
    UnknownFenceRejected,
    JobIdentityMismatch,
    AttemptIdentityMismatch,
    WorkerUnauthorized,
    OutputAdmissionUnauthorized,
    TransitionRejected,
    TerminalAttempt,
    ResultDigestMismatch,
    RetryAllowed,
    RetryBackoffRequired,
    RetryBudgetExhausted,
    RetryDeadlineExceeded,
    RetryFailureTerminal,
    RetryPolicyInvalid,
    Superseded,
    LegacyStateRejected,
    DurableStateInvalid,
    PersistenceUnconfigured,
    PersistenceFailed,
    ResumeStateConflict,
}

impl RemoteAttemptReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CurrentAttemptApplied => "current-attempt-applied",
            Self::CurrentAttemptCompleted => "current-attempt-completed",
            Self::AlreadyApplied => "already-applied",
            Self::EventDigestConflict => "event-digest-conflict",
            Self::EventRetentionExhausted => "event-retention-exhausted",
            Self::PayloadDigestInvalid => "payload-digest-invalid",
            Self::PayloadDigestMismatch => "payload-digest-mismatch",
            Self::PayloadSerializationFailed => "payload-serialization-failed",
            Self::PayloadTooLarge => "payload-too-large",
            Self::IdentityInvalid => "identity-invalid",
            Self::AssignmentNonceInvalid => "assignment-nonce-invalid",
            Self::FenceInvalid => "fence-invalid",
            Self::FenceExhausted => "fence-exhausted",
            Self::StaleReportRejected => "stale-report-rejected",
            Self::UnknownFenceRejected => "unknown-fence-rejected",
            Self::JobIdentityMismatch => "job-identity-mismatch",
            Self::AttemptIdentityMismatch => "attempt-identity-mismatch",
            Self::WorkerUnauthorized => "worker-unauthorized",
            Self::OutputAdmissionUnauthorized => "output-admission-unauthorized",
            Self::TransitionRejected => "transition-rejected",
            Self::TerminalAttempt => "terminal-attempt",
            Self::ResultDigestMismatch => "result-digest-mismatch",
            Self::RetryAllowed => "retry-allowed",
            Self::RetryBackoffRequired => "retry-backoff-required",
            Self::RetryBudgetExhausted => "retry-budget-exhausted",
            Self::RetryDeadlineExceeded => "retry-deadline-exceeded",
            Self::RetryFailureTerminal => "retry-failure-terminal",
            Self::RetryPolicyInvalid => "retry-policy-invalid",
            Self::Superseded => "superseded",
            Self::LegacyStateRejected => "legacy-state-rejected",
            Self::DurableStateInvalid => "durable-state-invalid",
            Self::PersistenceUnconfigured => "persistence-unconfigured",
            Self::PersistenceFailed => "persistence-failed",
            Self::ResumeStateConflict => "resume-state-conflict",
        }
    }
}

pub fn canonical_remote_attempt_payload_digest(
    payload: &RemoteAttemptReportPayload,
) -> Result<RemotePayloadDigest, RemoteAttemptReasonCode> {
    let mut hasher = blake3::Hasher::new();
    hash_identity_part(&mut hasher, REMOTE_ATTEMPT_PAYLOAD_DOMAIN)?;
    let mut preimage = PayloadPreimage::new(&mut hasher);
    match payload {
        RemoteAttemptReportPayload::Start => preimage.write(b"{\"kind\":\"start\"}")?,
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s } => {
            preimage.write(b"{\"kind\":\"heartbeat\",\"observed_unix_s\":")?;
            preimage.number(*observed_unix_s)?;
            preimage.write(b"}")?;
        }
        RemoteAttemptReportPayload::LogAppend { cursor, bytes } => {
            preimage.write(b"{\"kind\":\"log-append\",\"cursor\":")?;
            preimage.number(*cursor)?;
            preimage.write(b",\"bytes\":")?;
            preimage.string(bytes)?;
            preimage.write(b"}")?;
        }
        RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint,
            transferred_bytes,
        } => {
            preimage.write(b"{\"kind\":\"transfer-checkpoint\",\"checkpoint\":")?;
            preimage.number(*checkpoint)?;
            preimage.write(b",\"transferred_bytes\":")?;
            preimage.number(*transferred_bytes)?;
            preimage.write(b"}")?;
        }
        RemoteAttemptReportPayload::ResultReady { output_digest_blake3 } => {
            preimage.write(b"{\"kind\":\"result-ready\",\"output_digest_blake3\":")?;
            preimage.string(output_digest_blake3)?;
            preimage.write(b"}")?;
        }
        RemoteAttemptReportPayload::Failure {
            failure_class,
            reason_code,
        } => {
            let class = match failure_class {
                RemoteAttemptFailureClass::Retryable => "retryable",
                RemoteAttemptFailureClass::Terminal => "terminal",
                RemoteAttemptFailureClass::PolicyDenied => "policy-denied",
            };
            preimage.write(b"{\"kind\":\"failure\",\"failure_class\":")?;
            preimage.string(class)?;
            preimage.write(b",\"reason_code\":")?;
            preimage.string(reason_code.as_str())?;
            preimage.write(b"}")?;
        }
        RemoteAttemptReportPayload::Completion { output_digest_blake3 } => {
            preimage.write(b"{\"kind\":\"completion\",\"output_digest_blake3\":")?;
            preimage.string(output_digest_blake3)?;
            preimage.write(b"}")?;
        }
    }
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert!(is_blake3_hex_digest(&digest));
    RemotePayloadDigest::new(digest)
}

/// Stream exactly the version-one tagged JSON preimage, without a JSON
/// renderer, temporary allocation, or ambient authority in the core.
struct PayloadPreimage<'a> {
    hasher: &'a mut blake3::Hasher,
    bytes: usize,
}

impl<'a> PayloadPreimage<'a> {
    fn new(hasher: &'a mut blake3::Hasher) -> Self {
        Self { hasher, bytes: 0 }
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), RemoteAttemptReasonCode> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|length| *length <= MAX_REMOTE_ATTEMPT_PAYLOAD_BYTES)
            .ok_or(RemoteAttemptReasonCode::PayloadTooLarge)?;
        self.hasher.update(bytes);
        Ok(())
    }

    fn number(&mut self, mut value: u64) -> Result<(), RemoteAttemptReasonCode> {
        let mut digits = [0_u8; 20];
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        self.write(&digits[start..])
    }

    fn string(&mut self, value: &str) -> Result<(), RemoteAttemptReasonCode> {
        self.write(b"\"")?;
        let bytes = value.as_bytes();
        let mut start = 0;
        for (index, byte) in bytes.iter().copied().enumerate() {
            let escape: Option<&[u8]> = match byte {
                b'"' => Some(b"\\\""),
                b'\\' => Some(b"\\\\"),
                8 => Some(b"\\b"),
                12 => Some(b"\\f"),
                b'\n' => Some(b"\\n"),
                b'\r' => Some(b"\\r"),
                b'\t' => Some(b"\\t"),
                0..=31 => None,
                _ => continue,
            };
            self.write(&bytes[start..index])?;
            if let Some(escape) = escape {
                self.write(escape)?;
            } else {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                self.write(&[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX[usize::from(byte >> 4)],
                    HEX[usize::from(byte & 15)],
                ])?;
            }
            start = index + 1;
        }
        self.write(&bytes[start..])?;
        self.write(b"\"")
    }
}

pub fn derive_remote_attempt_id(
    job_id: &RemoteJobId,
    assignment_nonce: &RemoteAssignmentNonce,
    fence_generation: RemoteFenceGeneration,
    worker_endpoint_id: &str,
) -> Result<RemoteAttemptId, RemoteAttemptReasonCode> {
    let worker_endpoint_id = bounded_identity(worker_endpoint_id.to_string())?;
    RemoteAssignmentNonce::new(assignment_nonce.as_str().to_string())?;
    let mut hasher = blake3::Hasher::new();
    hash_identity_part(&mut hasher, REMOTE_ATTEMPT_ID_DOMAIN)?;
    hash_identity_part(&mut hasher, job_id.as_str())?;
    hash_identity_part(&mut hasher, assignment_nonce.as_str())?;
    hash_identity_part(&mut hasher, &fence_generation.get().to_string())?;
    hash_identity_part(&mut hasher, &worker_endpoint_id)?;
    let value = hasher.finalize().to_hex().to_string();
    debug_assert!(is_blake3_hex_digest(&value));
    debug_assert_ne!(fence_generation.get(), 0);
    RemoteAttemptId::new(value)
}

pub fn plan_remote_attempt_assignment_from_input(
    input: RemoteAttemptAssignmentInput<'_>,
) -> Result<RemoteAttemptState, RemoteAttemptReasonCode> {
    input.retry_policy.validate()?;
    validate_previous_attempt(input.job_id, input.previous)?;
    let fence_generation = next_remote_fence(input.previous)?;
    let attempts_started = next_attempt_count(input.previous)?;
    if attempts_started > input.retry_policy.max_attempts {
        return Err(RemoteAttemptReasonCode::RetryBudgetExhausted);
    }
    let attempt_deadline_unix_s = input
        .time
        .now_unix_s
        .checked_add(input.retry_policy.attempt_timeout_secs)
        .ok_or(RemoteAttemptReasonCode::RetryDeadlineExceeded)?;
    let deadline_unix_s = attempt_deadline_unix_s.min(input.time.overall_deadline_unix_s);
    if deadline_unix_s <= input.time.now_unix_s {
        return Err(RemoteAttemptReasonCode::RetryDeadlineExceeded);
    }
    let attempt_id =
        derive_remote_attempt_id(input.job_id, &input.assignment_nonce, fence_generation, input.worker_endpoint_id)?;
    let applied_events = input.previous.map(|attempt| attempt.applied_events.clone()).unwrap_or_default();
    debug_assert!(applied_events.len() <= MAX_REMOTE_ATTEMPT_EVENTS);
    debug_assert!(deadline_unix_s > input.time.now_unix_s);
    Ok(RemoteAttemptState {
        job_id: input.job_id.clone(),
        attempt_id,
        assignment_nonce: input.assignment_nonce,
        fence_generation,
        phase: RemoteAttemptPhase::Queued,
        attempts_started,
        started_unix_s: input.time.now_unix_s,
        deadline_unix_s,
        applied_events,
        transfer_checkpoint: None,
        transferred_bytes: 0,
        last_heartbeat_unix_s: None,
        result_digest_blake3: None,
    })
}

pub fn validate_remote_attempt_fence(
    current: &RemoteAttemptState,
    report: &RemoteAttemptReportIdentity,
) -> RemoteFenceDisposition {
    if report.job_id != current.job_id {
        return RemoteFenceDisposition::JobMismatch;
    }
    if report.fence_generation < current.fence_generation {
        return RemoteFenceDisposition::Stale;
    }
    if report.fence_generation > current.fence_generation {
        return RemoteFenceDisposition::UnknownFuture;
    }
    if report.attempt_id != current.attempt_id {
        return RemoteFenceDisposition::AttemptMismatch;
    }
    debug_assert_eq!(report.fence_generation, current.fence_generation);
    debug_assert_eq!(report.job_id, current.job_id);
    RemoteFenceDisposition::Current
}

pub fn classify_remote_attempt_event(
    applied_events: &BTreeMap<RemoteEventId, RemotePayloadDigest>,
    report: &RemoteAttemptReportIdentity,
) -> RemoteEventDisposition {
    match applied_events.get(&report.event_id) {
        None => RemoteEventDisposition::New,
        Some(digest) if digest == &report.payload_digest => RemoteEventDisposition::AlreadyApplied,
        Some(_) => RemoteEventDisposition::Conflict,
    }
}

pub fn decide_remote_attempt_transition(
    phase: RemoteAttemptPhase,
    report_kind: RemoteAttemptReportKind,
) -> RemoteAttemptTransitionDecision {
    if phase.is_terminal() {
        return rejected_transition(RemoteAttemptReasonCode::TerminalAttempt);
    }
    let next_phase = match (phase, report_kind) {
        (RemoteAttemptPhase::Queued, RemoteAttemptReportKind::Start) => RemoteAttemptPhase::Running,
        (RemoteAttemptPhase::Queued, RemoteAttemptReportKind::Failure) => RemoteAttemptPhase::Failed,
        (RemoteAttemptPhase::Running, RemoteAttemptReportKind::Heartbeat | RemoteAttemptReportKind::LogAppend) => {
            RemoteAttemptPhase::Running
        }
        (RemoteAttemptPhase::Running, RemoteAttemptReportKind::TransferCheckpoint) => RemoteAttemptPhase::Transferring,
        (RemoteAttemptPhase::Running | RemoteAttemptPhase::Transferring, RemoteAttemptReportKind::ResultReady) => {
            RemoteAttemptPhase::FinishedUndelivered
        }
        (RemoteAttemptPhase::Running | RemoteAttemptPhase::Transferring, RemoteAttemptReportKind::Failure) => {
            RemoteAttemptPhase::Failed
        }
        (RemoteAttemptPhase::Transferring, RemoteAttemptReportKind::Heartbeat | RemoteAttemptReportKind::LogAppend) => {
            RemoteAttemptPhase::Transferring
        }
        (RemoteAttemptPhase::Transferring, RemoteAttemptReportKind::TransferCheckpoint) => {
            RemoteAttemptPhase::Transferring
        }
        (RemoteAttemptPhase::FinishedUndelivered, RemoteAttemptReportKind::Completion) => RemoteAttemptPhase::Completed,
        (RemoteAttemptPhase::FinishedUndelivered, RemoteAttemptReportKind::Failure) => RemoteAttemptPhase::Failed,
        _ => return rejected_transition(RemoteAttemptReasonCode::TransitionRejected),
    };
    debug_assert!(!phase.is_terminal());
    debug_assert_ne!(next_phase, RemoteAttemptPhase::Superseded);
    let reason_code = if next_phase == RemoteAttemptPhase::Completed {
        RemoteAttemptReasonCode::CurrentAttemptCompleted
    } else {
        RemoteAttemptReasonCode::CurrentAttemptApplied
    };
    RemoteAttemptTransitionDecision {
        next_phase: Some(next_phase),
        reason_code,
    }
}

pub fn decide_remote_attempt_retry(
    state: &RemoteAttemptState,
    failure_class: RemoteAttemptFailureClass,
    retry_policy: RemoteAttemptRetryPolicy,
    time: RemoteAttemptTimeFacts,
) -> RemoteAttemptRetryDecision {
    if retry_policy.validate().is_err() {
        return rejected_retry(RemoteAttemptReasonCode::RetryPolicyInvalid);
    }
    if failure_class != RemoteAttemptFailureClass::Retryable {
        return rejected_retry(RemoteAttemptReasonCode::RetryFailureTerminal);
    }
    if state.attempts_started >= retry_policy.max_attempts {
        return rejected_retry(RemoteAttemptReasonCode::RetryBudgetExhausted);
    }
    let Some(not_before_unix_s) = time.failure_observed_unix_s.checked_add(retry_policy.retry_delay_secs) else {
        return rejected_retry(RemoteAttemptReasonCode::RetryDeadlineExceeded);
    };
    if time.now_unix_s > time.overall_deadline_unix_s || not_before_unix_s > time.overall_deadline_unix_s {
        return rejected_retry(RemoteAttemptReasonCode::RetryDeadlineExceeded);
    }
    if time.now_unix_s < not_before_unix_s {
        return RemoteAttemptRetryDecision {
            retry_allowed: false,
            not_before_unix_s: Some(not_before_unix_s),
            reason_code: RemoteAttemptReasonCode::RetryBackoffRequired,
        };
    }
    debug_assert!(state.attempts_started < retry_policy.max_attempts);
    debug_assert!(not_before_unix_s <= time.overall_deadline_unix_s);
    RemoteAttemptRetryDecision {
        retry_allowed: true,
        not_before_unix_s: Some(not_before_unix_s),
        reason_code: RemoteAttemptReasonCode::RetryAllowed,
    }
}

pub fn plan_remote_attempt_report(
    current: &RemoteAttemptState,
    report: &RemoteAttemptReport,
    authorization: RemoteAttemptAuthorizationFacts,
) -> RemoteAttemptApplyPlan {
    if let Some(reason_code) = validate_report_identity(report) {
        return rejected_plan(current, reason_code);
    }
    if let Some(reason_code) = validate_report_payload(report) {
        return rejected_plan(current, reason_code);
    }
    if let Err(reason_code) = authorize_remote_attempt_report(current, report, authorization) {
        return rejected_plan(current, reason_code);
    }
    match classify_remote_attempt_event(&current.applied_events, &report.identity) {
        RemoteEventDisposition::AlreadyApplied => return already_applied_plan(current),
        RemoteEventDisposition::Conflict => {
            return rejected_plan(current, RemoteAttemptReasonCode::EventDigestConflict);
        }
        RemoteEventDisposition::New => {}
    }
    if current.applied_events.len() >= MAX_REMOTE_ATTEMPT_EVENTS {
        return rejected_plan(current, RemoteAttemptReasonCode::EventRetentionExhausted);
    }
    let transition = decide_remote_attempt_transition(current.phase, report.payload.kind());
    let Some(next_phase) = transition.next_phase else {
        return rejected_plan(current, transition.reason_code);
    };
    if let Some(reason_code) = validate_payload_linkage(current, &report.payload) {
        return rejected_plan(current, reason_code);
    }
    debug_assert_eq!(current.job_id, report.identity.job_id);
    debug_assert_eq!(current.fence_generation, report.identity.fence_generation);
    accepted_plan(current, report, next_phase, transition.reason_code)
}

fn validate_report_identity(report: &RemoteAttemptReport) -> Option<RemoteAttemptReasonCode> {
    if RemoteJobId::new(report.identity.job_id.as_str().to_string()).is_err()
        || RemoteAttemptId::new(report.identity.attempt_id.as_str().to_string()).is_err()
        || RemoteEventId::new(report.identity.event_id.as_str().to_string()).is_err()
    {
        return Some(RemoteAttemptReasonCode::IdentityInvalid);
    }
    if report.identity.fence_generation.get() == 0 {
        return Some(RemoteAttemptReasonCode::FenceInvalid);
    }
    if RemotePayloadDigest::new(report.identity.payload_digest.as_str().to_string()).is_err() {
        return Some(RemoteAttemptReasonCode::PayloadDigestInvalid);
    }
    None
}

pub fn authorize_remote_attempt_report(
    current: &RemoteAttemptState,
    report: &RemoteAttemptReport,
    authorization: RemoteAttemptAuthorizationFacts,
) -> Result<(), RemoteAttemptReasonCode> {
    let fence_reason = match validate_remote_attempt_fence(current, &report.identity) {
        RemoteFenceDisposition::Current => None,
        RemoteFenceDisposition::Stale => Some(RemoteAttemptReasonCode::StaleReportRejected),
        RemoteFenceDisposition::UnknownFuture => Some(RemoteAttemptReasonCode::UnknownFenceRejected),
        RemoteFenceDisposition::JobMismatch => Some(RemoteAttemptReasonCode::JobIdentityMismatch),
        RemoteFenceDisposition::AttemptMismatch => Some(RemoteAttemptReasonCode::AttemptIdentityMismatch),
    };
    if let Some(reason_code) = fence_reason {
        return Err(reason_code);
    }
    debug_assert_eq!(current.job_id, report.identity.job_id);
    debug_assert_eq!(current.attempt_id, report.identity.attempt_id);
    if !authorization.worker_authorized {
        return Err(RemoteAttemptReasonCode::WorkerUnauthorized);
    }
    if report_requires_output_admission(&report.payload) && !authorization.output_admission_authorized {
        return Err(RemoteAttemptReasonCode::OutputAdmissionUnauthorized);
    }
    Ok(())
}

fn validate_report_payload(report: &RemoteAttemptReport) -> Option<RemoteAttemptReasonCode> {
    let digest = match canonical_remote_attempt_payload_digest(&report.payload) {
        Ok(digest) => digest,
        Err(reason_code) => return Some(reason_code),
    };
    if digest != report.identity.payload_digest {
        return Some(RemoteAttemptReasonCode::PayloadDigestMismatch);
    }
    if let Some(output_digest) = report.payload.output_digest()
        && !is_blake3_hex_digest(output_digest)
    {
        return Some(RemoteAttemptReasonCode::PayloadDigestInvalid);
    }
    None
}

fn validate_payload_linkage(
    current: &RemoteAttemptState,
    payload: &RemoteAttemptReportPayload,
) -> Option<RemoteAttemptReasonCode> {
    match payload {
        RemoteAttemptReportPayload::Completion { output_digest_blake3 }
            if current.result_digest_blake3.as_deref() != Some(output_digest_blake3.as_str()) =>
        {
            Some(RemoteAttemptReasonCode::ResultDigestMismatch)
        }
        RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint,
            transferred_bytes,
        } if transfer_checkpoint_regresses(current, TransferCheckpointProgress {
            checkpoint: *checkpoint,
            transferred_bytes: *transferred_bytes,
        }) =>
        {
            Some(RemoteAttemptReasonCode::TransitionRejected)
        }
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s }
            if heartbeat_time_is_invalid(current, *observed_unix_s) =>
        {
            Some(RemoteAttemptReasonCode::TransitionRejected)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TransferCheckpointProgress {
    checkpoint: u64,
    transferred_bytes: u64,
}

fn transfer_checkpoint_regresses(current: &RemoteAttemptState, progress: TransferCheckpointProgress) -> bool {
    if current.transfer_checkpoint.is_some_and(|current| progress.checkpoint <= current) {
        return true;
    }
    if progress.transferred_bytes < current.transferred_bytes {
        return true;
    }
    false
}

fn heartbeat_time_is_invalid(current: &RemoteAttemptState, observed_unix_s: u64) -> bool {
    if observed_unix_s < current.started_unix_s {
        return true;
    }
    if observed_unix_s > current.deadline_unix_s {
        return true;
    }
    current.last_heartbeat_unix_s.is_some_and(|previous| observed_unix_s <= previous)
}

fn accepted_plan(
    current: &RemoteAttemptState,
    report: &RemoteAttemptReport,
    next_phase: RemoteAttemptPhase,
    reason_code: RemoteAttemptReasonCode,
) -> RemoteAttemptApplyPlan {
    let mut next_state = current.clone();
    next_state.phase = next_phase;
    match &report.payload {
        RemoteAttemptReportPayload::ResultReady { output_digest_blake3 } => {
            next_state.result_digest_blake3 = Some(output_digest_blake3.clone());
        }
        RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint,
            transferred_bytes,
        } => {
            next_state.transfer_checkpoint = Some(*checkpoint);
            next_state.transferred_bytes = *transferred_bytes;
        }
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s } => {
            next_state.last_heartbeat_unix_s = Some(*observed_unix_s);
        }
        RemoteAttemptReportPayload::Failure { .. } => {
            next_state.result_digest_blake3 = None;
        }
        _ => {}
    }
    next_state
        .applied_events
        .insert(report.identity.event_id.clone(), report.identity.payload_digest.clone());
    debug_assert!(next_state.applied_events.len() <= MAX_REMOTE_ATTEMPT_EVENTS);
    debug_assert_eq!(next_state.fence_generation, current.fence_generation);
    RemoteAttemptApplyPlan {
        disposition: RemoteAttemptApplyDisposition::Applied,
        reason_code,
        next_state,
        output_admission_allowed: report_requires_output_admission(&report.payload),
        pending_effect: Some(crate::effect::EffectKind::AttemptPersist),
    }
}

fn rejected_plan(current: &RemoteAttemptState, reason_code: RemoteAttemptReasonCode) -> RemoteAttemptApplyPlan {
    RemoteAttemptApplyPlan {
        disposition: RemoteAttemptApplyDisposition::Rejected,
        reason_code,
        next_state: current.clone(),
        output_admission_allowed: false,
        pending_effect: None,
    }
}

fn already_applied_plan(current: &RemoteAttemptState) -> RemoteAttemptApplyPlan {
    RemoteAttemptApplyPlan {
        disposition: RemoteAttemptApplyDisposition::AlreadyApplied,
        reason_code: RemoteAttemptReasonCode::AlreadyApplied,
        next_state: current.clone(),
        output_admission_allowed: false,
        pending_effect: None,
    }
}

fn rejected_transition(reason_code: RemoteAttemptReasonCode) -> RemoteAttemptTransitionDecision {
    RemoteAttemptTransitionDecision {
        next_phase: None,
        reason_code,
    }
}

fn rejected_retry(reason_code: RemoteAttemptReasonCode) -> RemoteAttemptRetryDecision {
    RemoteAttemptRetryDecision {
        retry_allowed: false,
        not_before_unix_s: None,
        reason_code,
    }
}

fn report_requires_output_admission(payload: &RemoteAttemptReportPayload) -> bool {
    matches!(
        payload,
        RemoteAttemptReportPayload::ResultReady { .. } | RemoteAttemptReportPayload::Completion { .. }
    )
}

fn validate_previous_attempt(
    job_id: &RemoteJobId,
    previous: Option<&RemoteAttemptState>,
) -> Result<(), RemoteAttemptReasonCode> {
    if let Some(previous) = previous {
        if &previous.job_id != job_id {
            return Err(RemoteAttemptReasonCode::JobIdentityMismatch);
        }
        if RemoteAssignmentNonce::new(previous.assignment_nonce.as_str().to_string()).is_err() {
            return Err(RemoteAttemptReasonCode::AssignmentNonceInvalid);
        }
        if previous.applied_events.len() > MAX_REMOTE_ATTEMPT_EVENTS {
            return Err(RemoteAttemptReasonCode::EventRetentionExhausted);
        }
    }
    Ok(())
}

fn next_remote_fence(previous: Option<&RemoteAttemptState>) -> Result<RemoteFenceGeneration, RemoteAttemptReasonCode> {
    match previous {
        Some(previous) => previous.fence_generation.advance(),
        None => Ok(RemoteFenceGeneration::INITIAL),
    }
}

fn next_attempt_count(previous: Option<&RemoteAttemptState>) -> Result<u32, RemoteAttemptReasonCode> {
    match previous {
        Some(previous) => previous.attempts_started.checked_add(1).ok_or(RemoteAttemptReasonCode::RetryBudgetExhausted),
        None => Ok(INITIAL_REMOTE_ATTEMPT_COUNT),
    }
}

fn bounded_identity(value: String) -> Result<String, RemoteAttemptReasonCode> {
    if value.is_empty() || value.len() > MAX_REMOTE_ATTEMPT_ID_BYTES {
        return Err(RemoteAttemptReasonCode::IdentityInvalid);
    }
    if value.chars().any(char::is_control) {
        return Err(RemoteAttemptReasonCode::IdentityInvalid);
    }
    Ok(value)
}

fn is_blake3_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hash_identity_part(hasher: &mut blake3::Hasher, value: &str) -> Result<(), RemoteAttemptReasonCode> {
    let length_bytes = u64::try_from(value.len()).map_err(|_| RemoteAttemptReasonCode::IdentityInvalid)?.to_le_bytes();
    debug_assert!(!value.is_empty());
    debug_assert!(value.len() <= MAX_REMOTE_ATTEMPT_ID_BYTES);
    hasher.update(&length_bytes);
    hasher.update(value.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    // r[verify remote_builds.pure_attempt_decisions]
    // r[verify remote_builds.idempotent_attempt_reporting]
    // r[verify remote_builds.durable_attempt_fencing]
    use proptest::prelude::*;

    use super::*;

    const TEST_NOW_UNIX_S: u64 = 10;
    const TEST_SECOND_ATTEMPT_NOW_UNIX_S: u64 = 20;
    const TEST_DEADLINE_UNIX_S: u64 = 10_000;
    const TEST_EXPIRED_DEADLINE_UNIX_S: u64 = TEST_NOW_UNIX_S - INITIAL_REMOTE_FENCE_GENERATION;
    const PROPERTY_EVENT_SUFFIX_MAX_EXCLUSIVE: u64 = 1_000;
    const PROPERTY_REPORT_KIND_COUNT: u8 = 7;
    const TEST_FENCE_CHAIN_ATTEMPTS: u32 = 8;
    const TEST_TRANSFER_CHECKPOINT: u64 = 2;
    const TEST_TRANSFERRED_BYTES: u64 = 64;
    const TEST_OUTPUT_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn job_id() -> RemoteJobId {
        RemoteJobId::new("job-1").unwrap()
    }

    fn assignment_nonce(label: &str) -> RemoteAssignmentNonce {
        RemoteAssignmentNonce::new(blake3::hash(label.as_bytes()).to_hex().to_string()).unwrap()
    }

    fn policy() -> RemoteAttemptRetryPolicy {
        RemoteAttemptRetryPolicy::default()
    }

    fn time_facts(now_unix_s: u64) -> RemoteAttemptTimeFacts {
        RemoteAttemptTimeFacts {
            now_unix_s,
            failure_observed_unix_s: now_unix_s,
            overall_deadline_unix_s: TEST_DEADLINE_UNIX_S,
        }
    }

    fn attempt() -> RemoteAttemptState {
        plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &job_id(),
            worker_endpoint_id: "worker-1",
            assignment_nonce: assignment_nonce("attempt-1"),
            previous: None,
            retry_policy: policy(),
            time: time_facts(TEST_NOW_UNIX_S),
        })
        .unwrap()
    }

    fn report(state: &RemoteAttemptState, event_id: &str, payload: RemoteAttemptReportPayload) -> RemoteAttemptReport {
        RemoteAttemptReport::new(
            state.job_id.clone(),
            state.attempt_id.clone(),
            state.fence_generation,
            RemoteEventId::new(event_id).unwrap(),
            payload,
        )
        .unwrap()
    }

    fn authorized() -> RemoteAttemptAuthorizationFacts {
        RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: true,
        }
    }

    #[test]
    fn durable_attempt_rejects_replayed_worker_identity_missing_nonce_and_forged_result() {
        let mut state = attempt();
        assert!(persisted_remote_attempt_is_valid(&job_id(), Some("worker-1"), &state));
        assert!(!persisted_remote_attempt_is_valid(&job_id(), Some("worker-2"), &state));

        state.phase = RemoteAttemptPhase::Completed;
        state.result_digest_blake3 = Some(TEST_OUTPUT_DIGEST.to_string());
        assert!(persisted_remote_attempt_is_valid(&job_id(), Some("worker-1"), &state));
        state.result_digest_blake3 = Some("not-a-digest".to_string());
        assert!(!persisted_remote_attempt_is_valid(&job_id(), Some("worker-1"), &state));

        let mut state = attempt();
        state.assignment_nonce = RemoteAssignmentNonce(String::new());
        assert!(!persisted_remote_attempt_is_valid(&job_id(), Some("worker-1"), &state));
        let mut state = attempt();
        state
            .applied_events
            .insert(RemoteEventId("event-1".to_string()), RemotePayloadDigest("bad".to_string()));
        assert!(!persisted_remote_attempt_is_valid(&job_id(), Some("worker-1"), &state));
    }

    #[test]
    fn durable_coordinator_projection_rejects_stale_transfer_and_false_result_claims() {
        let mut state = attempt();
        state.phase = RemoteAttemptPhase::FinishedUndelivered;
        state.transfer_checkpoint = Some(4);
        state.transferred_bytes = 24;
        let expected = CoordinatorAttemptProjectionFacts {
            phase: CoordinatorAttemptPhase::Finished,
            result_available: false,
            output_admission_completed: true,
            transfer_checkpoint: Some(4),
            transferred_bytes: 24,
        };
        assert!(persisted_coordinator_attempt_projection_matches(expected, &state));
        assert!(!persisted_coordinator_attempt_projection_matches(
            CoordinatorAttemptProjectionFacts {
                result_available: true,
                ..expected
            },
            &state
        ));
        assert!(!persisted_coordinator_attempt_projection_matches(
            CoordinatorAttemptProjectionFacts {
                transfer_checkpoint: Some(3),
                ..expected
            },
            &state
        ));
        assert!(!persisted_coordinator_attempt_projection_matches(
            CoordinatorAttemptProjectionFacts {
                phase: CoordinatorAttemptPhase::Running,
                ..expected
            },
            &state
        ));
        state.phase = RemoteAttemptPhase::Completed;
        assert!(persisted_coordinator_attempt_projection_matches(
            CoordinatorAttemptProjectionFacts {
                result_available: true,
                ..expected
            },
            &state
        ));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn canonical_payload_digests_match_the_existing_v1_serde_wire_bytes() {
        let payloads = [
            RemoteAttemptReportPayload::Start,
            RemoteAttemptReportPayload::Heartbeat {
                observed_unix_s: u64::MAX,
            },
            RemoteAttemptReportPayload::LogAppend {
                cursor: u64::MAX,
                bytes: String::from("quote \" slash \\ line\nbackspace\u{0008} form\u{000c} nul\u{0000} café 🦀"),
            },
            RemoteAttemptReportPayload::TransferCheckpoint {
                checkpoint: u64::MAX,
                transferred_bytes: 0,
            },
            RemoteAttemptReportPayload::ResultReady {
                output_digest_blake3: String::from("output\n\"é"),
            },
            RemoteAttemptReportPayload::Failure {
                failure_class: RemoteAttemptFailureClass::Retryable,
                reason_code: RemoteAttemptReasonCode::StaleReportRejected,
            },
            RemoteAttemptReportPayload::Failure {
                failure_class: RemoteAttemptFailureClass::Terminal,
                reason_code: RemoteAttemptReasonCode::RetryBudgetExhausted,
            },
            RemoteAttemptReportPayload::Failure {
                failure_class: RemoteAttemptFailureClass::PolicyDenied,
                reason_code: RemoteAttemptReasonCode::OutputAdmissionUnauthorized,
            },
            RemoteAttemptReportPayload::Completion {
                output_digest_blake3: String::from("completed"),
            },
        ];
        for payload in payloads {
            let bytes = serde_json::to_vec(&payload).unwrap();
            let mut reference = blake3::Hasher::new();
            hash_identity_part(&mut reference, REMOTE_ATTEMPT_PAYLOAD_DOMAIN).unwrap();
            reference.update(&bytes);
            assert_eq!(
                canonical_remote_attempt_payload_digest(&payload).unwrap().as_str(),
                reference.finalize().to_hex().as_str(),
                "version-one payload bytes changed: {payload:?}"
            );
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn canonical_payload_byte_limit_includes_json_escaping_and_framing() {
        let empty = RemoteAttemptReportPayload::LogAppend {
            cursor: 0,
            bytes: String::new(),
        };
        let framing = serde_json::to_vec(&empty).unwrap().len();
        let within = RemoteAttemptReportPayload::LogAppend {
            cursor: 0,
            bytes: "a".repeat(MAX_REMOTE_ATTEMPT_PAYLOAD_BYTES - framing),
        };
        assert!(canonical_remote_attempt_payload_digest(&within).is_ok());
        let over = RemoteAttemptReportPayload::LogAppend {
            cursor: 0,
            bytes: "a".repeat(MAX_REMOTE_ATTEMPT_PAYLOAD_BYTES - framing + 1),
        };
        assert_eq!(canonical_remote_attempt_payload_digest(&over), Err(RemoteAttemptReasonCode::PayloadTooLarge));
        let escaped = RemoteAttemptReportPayload::LogAppend {
            cursor: 0,
            bytes: "\n".repeat((MAX_REMOTE_ATTEMPT_PAYLOAD_BYTES - framing) / 2 + 1),
        };
        assert_eq!(canonical_remote_attempt_payload_digest(&escaped), Err(RemoteAttemptReasonCode::PayloadTooLarge));
    }

    #[test]
    fn transition_table_accepts_only_declared_live_edges() {
        let cases = [
            (RemoteAttemptPhase::Queued, RemoteAttemptReportKind::Start, Some(RemoteAttemptPhase::Running)),
            (RemoteAttemptPhase::Running, RemoteAttemptReportKind::Heartbeat, Some(RemoteAttemptPhase::Running)),
            (RemoteAttemptPhase::Running, RemoteAttemptReportKind::LogAppend, Some(RemoteAttemptPhase::Running)),
            (
                RemoteAttemptPhase::Running,
                RemoteAttemptReportKind::TransferCheckpoint,
                Some(RemoteAttemptPhase::Transferring),
            ),
            (
                RemoteAttemptPhase::Transferring,
                RemoteAttemptReportKind::ResultReady,
                Some(RemoteAttemptPhase::FinishedUndelivered),
            ),
            (
                RemoteAttemptPhase::FinishedUndelivered,
                RemoteAttemptReportKind::Completion,
                Some(RemoteAttemptPhase::Completed),
            ),
            (RemoteAttemptPhase::Completed, RemoteAttemptReportKind::Start, None),
            (RemoteAttemptPhase::Failed, RemoteAttemptReportKind::Heartbeat, None),
            (RemoteAttemptPhase::Superseded, RemoteAttemptReportKind::Completion, None),
        ];
        for (phase, kind, expected) in cases {
            let decision = decide_remote_attempt_transition(phase, kind);
            assert_eq!(decision.next_phase, expected, "phase={phase:?} kind={kind:?}");
            assert_eq!(decision.next_phase.is_some(), expected.is_some());
        }
    }

    #[test]
    fn duplicate_report_is_an_idempotent_no_op_and_conflict_is_rejected() {
        let queued = attempt();
        let start = report(&queued, "event-1", RemoteAttemptReportPayload::Start);
        let first = plan_remote_attempt_report(&queued, &start, authorized());
        let duplicate = plan_remote_attempt_report(&first.next_state, &start, authorized());
        let mut conflict = start.clone();
        conflict.payload = RemoteAttemptReportPayload::Failure {
            failure_class: RemoteAttemptFailureClass::Retryable,
            reason_code: RemoteAttemptReasonCode::RetryAllowed,
        };
        conflict.identity.payload_digest = canonical_remote_attempt_payload_digest(&conflict.payload).unwrap();
        let rejected = plan_remote_attempt_report(&first.next_state, &conflict, authorized());

        assert_eq!(first.disposition, RemoteAttemptApplyDisposition::Applied);
        assert_eq!(duplicate.disposition, RemoteAttemptApplyDisposition::AlreadyApplied);
        assert_eq!(duplicate.next_state, first.next_state);
        assert_eq!(rejected.reason_code, RemoteAttemptReasonCode::EventDigestConflict);
        assert_eq!(rejected.next_state, first.next_state);
    }

    #[test]
    fn stale_report_is_rejected_before_output_admission() {
        let first = attempt();
        let second = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &job_id(),
            worker_endpoint_id: "worker-2",
            assignment_nonce: assignment_nonce("attempt-2"),
            previous: Some(&first),
            retry_policy: policy(),
            time: time_facts(TEST_SECOND_ATTEMPT_NOW_UNIX_S),
        })
        .unwrap();
        let stale = report(&first, "event-stale", RemoteAttemptReportPayload::Completion {
            output_digest_blake3: TEST_OUTPUT_DIGEST.to_string(),
        });
        let decision = plan_remote_attempt_report(&second, &stale, authorized());

        assert_eq!(decision.reason_code, RemoteAttemptReasonCode::StaleReportRejected);
        assert_eq!(decision.next_state, second);
        assert!(!decision.output_admission_allowed);
    }

    #[test]
    fn assignment_nonce_prevents_identity_reuse_after_state_reset() {
        let first = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &job_id(),
            worker_endpoint_id: "worker-1",
            assignment_nonce: assignment_nonce("coordinator-incarnation-1"),
            previous: None,
            retry_policy: policy(),
            time: time_facts(TEST_NOW_UNIX_S),
        })
        .unwrap();
        let replacement_after_reset = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &job_id(),
            worker_endpoint_id: "worker-1",
            assignment_nonce: assignment_nonce("coordinator-incarnation-2"),
            previous: None,
            retry_policy: policy(),
            time: time_facts(TEST_NOW_UNIX_S),
        })
        .unwrap();
        let stale = report(&first, "reset-stale", RemoteAttemptReportPayload::Start);
        let decision = plan_remote_attempt_report(&replacement_after_reset, &stale, authorized());

        assert_ne!(first.assignment_nonce, replacement_after_reset.assignment_nonce);
        assert_ne!(first.attempt_id, replacement_after_reset.attempt_id);
        assert_eq!(decision.reason_code, RemoteAttemptReasonCode::AttemptIdentityMismatch);
        assert_eq!(decision.next_state, replacement_after_reset);
    }

    #[test]
    fn malformed_assignment_nonce_fails_before_identity_derivation() {
        let error = RemoteAssignmentNonce::new("not-a-blake3-digest").unwrap_err();

        assert_eq!(error, RemoteAttemptReasonCode::AssignmentNonceInvalid);
        assert_eq!(error.as_str(), "assignment-nonce-invalid");
    }

    #[test]
    fn malformed_identity_and_unknown_future_fence_fail_without_state_change() {
        let state = attempt();
        let valid = report(&state, "identity-event", RemoteAttemptReportPayload::Start);
        let mut malformed = valid.clone();
        malformed.identity.event_id = RemoteEventId(String::new());
        let malformed_decision = plan_remote_attempt_report(&state, &malformed, authorized());
        let mut future = valid;
        future.identity.fence_generation = state.fence_generation.advance().unwrap();
        let future_decision = plan_remote_attempt_report(&state, &future, authorized());

        assert_eq!(malformed_decision.reason_code, RemoteAttemptReasonCode::IdentityInvalid);
        assert_eq!(malformed_decision.next_state, state);
        assert_eq!(future_decision.reason_code, RemoteAttemptReasonCode::UnknownFenceRejected);
        assert_eq!(future_decision.next_state, state);
        assert!(!future_decision.output_admission_allowed);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialized_malformed_identity_is_rejected_by_the_pure_transition() {
        let state = attempt();
        let mut encoded = serde_json::to_value(report(&state, "event", RemoteAttemptReportPayload::Start)).unwrap();
        encoded["identity"]["event_id"] = serde_json::Value::String(String::new());
        let malformed: RemoteAttemptReport = serde_json::from_value(encoded).unwrap();
        let decision = plan_remote_attempt_report(&state, &malformed, authorized());
        assert_eq!(decision.reason_code, RemoteAttemptReasonCode::IdentityInvalid);
        assert_eq!(decision.next_state, state);
    }

    #[test]
    fn result_ready_and_completion_require_matching_output_digest() {
        let queued = attempt();
        let started = plan_remote_attempt_report(
            &queued,
            &report(&queued, "start", RemoteAttemptReportPayload::Start),
            authorized(),
        )
        .next_state;
        let result = report(&started, "result", RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: TEST_OUTPUT_DIGEST.to_string(),
        });
        let ready = plan_remote_attempt_report(&started, &result, authorized());
        let wrong_completion = report(&ready.next_state, "complete", RemoteAttemptReportPayload::Completion {
            output_digest_blake3: "b".repeat(BLAKE3_HEX_LENGTH_CHARS),
        });
        let rejected = plan_remote_attempt_report(&ready.next_state, &wrong_completion, authorized());

        assert!(ready.output_admission_allowed);
        assert_eq!(ready.next_state.phase, RemoteAttemptPhase::FinishedUndelivered);
        assert_eq!(rejected.reason_code, RemoteAttemptReasonCode::ResultDigestMismatch);
        assert!(!rejected.output_admission_allowed);
    }

    #[test]
    fn event_retention_exhaustion_fails_closed_without_evicting_conflict_history() {
        let mut state = attempt();
        let digest = canonical_remote_attempt_payload_digest(&RemoteAttemptReportPayload::Start).unwrap();
        for event_index in 0..MAX_REMOTE_ATTEMPT_EVENTS {
            state
                .applied_events
                .insert(RemoteEventId::new(format!("retained-event-{event_index}")).unwrap(), digest.clone());
        }
        let overflow = report(&state, "overflow-event", RemoteAttemptReportPayload::Start);
        let decision = plan_remote_attempt_report(&state, &overflow, authorized());

        assert_eq!(decision.reason_code, RemoteAttemptReasonCode::EventRetentionExhausted);
        assert_eq!(decision.next_state, state);
        assert_eq!(decision.next_state.applied_events.len(), MAX_REMOTE_ATTEMPT_EVENTS);
        assert!(!decision.output_admission_allowed);
    }

    #[test]
    fn repeated_assignment_advances_every_fence_generation() {
        let chain_policy = RemoteAttemptRetryPolicy {
            max_attempts: TEST_FENCE_CHAIN_ATTEMPTS,
            ..policy()
        };
        let mut current = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &job_id(),
            worker_endpoint_id: "worker-chain",
            assignment_nonce: assignment_nonce("chain-0"),
            previous: None,
            retry_policy: chain_policy,
            time: time_facts(TEST_NOW_UNIX_S),
        })
        .unwrap();
        for attempt_index in INITIAL_REMOTE_ATTEMPT_COUNT..TEST_FENCE_CHAIN_ATTEMPTS {
            let next = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
                job_id: &job_id(),
                worker_endpoint_id: "worker-chain",
                assignment_nonce: assignment_nonce(&format!("chain-{attempt_index}")),
                previous: Some(&current),
                retry_policy: chain_policy,
                time: time_facts(TEST_NOW_UNIX_S + u64::from(attempt_index)),
            })
            .unwrap();
            assert!(next.fence_generation > current.fence_generation);
            assert_eq!(next.attempts_started, current.attempts_started + INITIAL_REMOTE_ATTEMPT_COUNT);
            current = next;
        }
        assert_eq!(current.attempts_started, TEST_FENCE_CHAIN_ATTEMPTS);
        assert_eq!(current.fence_generation.get(), u64::from(TEST_FENCE_CHAIN_ATTEMPTS));
    }

    #[test]
    fn transfer_checkpoints_cannot_regress_with_a_fresh_event_id() {
        let queued = attempt();
        let start = report(&queued, "transfer-start", RemoteAttemptReportPayload::Start);
        let running = plan_remote_attempt_report(&queued, &start, authorized()).next_state;
        let checkpoint = report(&running, "transfer-current", RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint: TEST_TRANSFER_CHECKPOINT,
            transferred_bytes: TEST_TRANSFERRED_BYTES,
        });
        let transferred = plan_remote_attempt_report(&running, &checkpoint, authorized()).next_state;
        let regressed = report(&transferred, "transfer-regressed", RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint: TEST_TRANSFER_CHECKPOINT,
            transferred_bytes: TEST_TRANSFERRED_BYTES,
        });
        let decision = plan_remote_attempt_report(&transferred, &regressed, authorized());

        assert_eq!(decision.reason_code, RemoteAttemptReasonCode::TransitionRejected);
        assert_eq!(decision.next_state, transferred);
        assert_eq!(decision.next_state.transfer_checkpoint, Some(TEST_TRANSFER_CHECKPOINT));
        assert!(!decision.output_admission_allowed);
    }

    #[test]
    fn output_result_requires_separate_output_admission_authorization() {
        let queued = attempt();
        let start = report(&queued, "authorize-start", RemoteAttemptReportPayload::Start);
        let running = plan_remote_attempt_report(&queued, &start, authorized()).next_state;
        let result = report(&running, "unauthorized-result", RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: TEST_OUTPUT_DIGEST.to_string(),
        });
        let decision = plan_remote_attempt_report(&running, &result, RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: false,
        });

        assert_eq!(decision.reason_code, RemoteAttemptReasonCode::OutputAdmissionUnauthorized);
        assert_eq!(decision.next_state, running);
        assert!(!decision.output_admission_allowed);
    }

    #[test]
    fn retry_decision_uses_only_supplied_budget_and_time_facts() {
        let state = attempt();
        let waiting = decide_remote_attempt_retry(
            &state,
            RemoteAttemptFailureClass::Retryable,
            policy(),
            RemoteAttemptTimeFacts {
                now_unix_s: TEST_NOW_UNIX_S,
                failure_observed_unix_s: TEST_NOW_UNIX_S,
                overall_deadline_unix_s: TEST_DEADLINE_UNIX_S,
            },
        );
        let terminal = decide_remote_attempt_retry(
            &state,
            RemoteAttemptFailureClass::Terminal,
            policy(),
            time_facts(TEST_NOW_UNIX_S),
        );
        let mut exhausted_state = state.clone();
        exhausted_state.attempts_started = policy().max_attempts;
        let exhausted = decide_remote_attempt_retry(
            &exhausted_state,
            RemoteAttemptFailureClass::Retryable,
            policy(),
            time_facts(TEST_NOW_UNIX_S),
        );
        let expired = decide_remote_attempt_retry(
            &state,
            RemoteAttemptFailureClass::Retryable,
            policy(),
            RemoteAttemptTimeFacts {
                now_unix_s: TEST_NOW_UNIX_S,
                failure_observed_unix_s: TEST_NOW_UNIX_S,
                overall_deadline_unix_s: TEST_EXPIRED_DEADLINE_UNIX_S,
            },
        );

        assert_eq!(waiting.reason_code, RemoteAttemptReasonCode::RetryBackoffRequired);
        assert_eq!(waiting.not_before_unix_s, Some(TEST_NOW_UNIX_S + policy().retry_delay_secs));
        assert_eq!(terminal.reason_code, RemoteAttemptReasonCode::RetryFailureTerminal);
        assert!(!terminal.retry_allowed);
        assert_eq!(exhausted.reason_code, RemoteAttemptReasonCode::RetryBudgetExhausted);
        assert_eq!(expired.reason_code, RemoteAttemptReasonCode::RetryDeadlineExceeded);
    }

    proptest! {
        #[test]
        fn equivalent_facts_produce_equivalent_decisions(
            event_suffix in INITIAL_REMOTE_FENCE_GENERATION..PROPERTY_EVENT_SUFFIX_MAX_EXCLUSIVE
        ) {
            let state = attempt();
            let event_id = format!("event-{event_suffix}");
            let report = report(&state, &event_id, RemoteAttemptReportPayload::Start);
            let left = plan_remote_attempt_report(&state, &report, authorized());
            let right = plan_remote_attempt_report(&state.clone(), &report.clone(), authorized());
            prop_assert_eq!(&left, &right);
            prop_assert_eq!(left.reason_code.as_str(), right.reason_code.as_str());
        }

        #[test]
        fn advancing_a_valid_fence_never_decreases(value in 1_u64..u64::MAX) {
            let current = RemoteFenceGeneration::new(value).unwrap();
            let next = current.advance().unwrap();
            prop_assert!(next > current);
            prop_assert_ne!(next.get(), 0);
        }

        #[test]
        fn terminal_states_never_transition_to_live_states(kind_index in 0_u8..PROPERTY_REPORT_KIND_COUNT) {
            let kinds = [
                RemoteAttemptReportKind::Start,
                RemoteAttemptReportKind::Heartbeat,
                RemoteAttemptReportKind::LogAppend,
                RemoteAttemptReportKind::TransferCheckpoint,
                RemoteAttemptReportKind::ResultReady,
                RemoteAttemptReportKind::Failure,
                RemoteAttemptReportKind::Completion,
            ];
            let kind = kinds[usize::from(kind_index)];
            for phase in [RemoteAttemptPhase::Completed, RemoteAttemptPhase::Failed, RemoteAttemptPhase::Superseded] {
                let decision = decide_remote_attempt_transition(phase, kind);
                prop_assert!(decision.next_phase.is_none());
                prop_assert_eq!(decision.reason_code, RemoteAttemptReasonCode::TerminalAttempt);
            }
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    const KANI_NOW_UNIX_S: u64 = 1;
    const KANI_DEADLINE_UNIX_S: u64 = 10_000;

    #[kani::proof]
    fn equivalent_facts_produce_equivalent_decisions() {
        let worker_authorized: bool = kani::any();
        let job_id = RemoteJobId::new("kani-job").unwrap();
        let state = plan_remote_attempt_assignment_from_input(RemoteAttemptAssignmentInput {
            job_id: &job_id,
            worker_endpoint_id: "kani-worker",
            assignment_nonce: RemoteAssignmentNonce::new("a".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap(),
            previous: None,
            retry_policy: RemoteAttemptRetryPolicy::default(),
            time: RemoteAttemptTimeFacts {
                now_unix_s: KANI_NOW_UNIX_S,
                failure_observed_unix_s: KANI_NOW_UNIX_S,
                overall_deadline_unix_s: KANI_DEADLINE_UNIX_S,
            },
        })
        .unwrap();
        let report = RemoteAttemptReport::new(
            job_id,
            state.attempt_id.clone(),
            state.fence_generation,
            RemoteEventId::new("kani-event").unwrap(),
            RemoteAttemptReportPayload::Start,
        )
        .unwrap();
        let authorization = RemoteAttemptAuthorizationFacts {
            worker_authorized,
            output_admission_authorized: false,
        };
        let left = plan_remote_attempt_report(&state, &report, authorization);
        let right = plan_remote_attempt_report(&state.clone(), &report.clone(), authorization);
        assert_eq!(left, right);
        assert_eq!(left.reason_code.as_str(), right.reason_code.as_str());
    }

    #[kani::proof]
    fn fence_generations_never_decrease() {
        let value: u64 = kani::any();
        kani::assume(value > 0);
        kani::assume(value < u64::MAX);
        let current = RemoteFenceGeneration::new(value).unwrap();
        let next = current.advance().unwrap();
        assert!(next > current);
        assert!(next.get() != 0);
    }

    #[kani::proof]
    fn terminal_attempts_cannot_return_to_live_states() {
        let choose_failed: bool = kani::any();
        let phase = if choose_failed {
            RemoteAttemptPhase::Failed
        } else {
            RemoteAttemptPhase::Completed
        };
        let decision = decide_remote_attempt_transition(phase, RemoteAttemptReportKind::Start);
        assert!(decision.next_phase.is_none());
        assert_eq!(decision.reason_code, RemoteAttemptReasonCode::TerminalAttempt);
    }
}
