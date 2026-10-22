//! Finite, equality-preserving projection for the Trellis fenced-attempt kernel.
//!
//! This is a consumer mapping, not a second admission model. The host parity
//! harness calls the exact pinned Trellis kernel source imported for tests.
//! A projection is not a Valence acceptance receipt or runtime/output authority.
//!
//! r[impl remote_builds.trellis_admission_model]
//! r[impl remote_builds.trellis_admission_projection]
//! r[impl remote_builds.trellis_admission_claim_boundary]

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;

use crate::attempt::MAX_REMOTE_ATTEMPT_EVENTS;
use crate::attempt::MAX_REMOTE_ATTEMPT_ID_BYTES;
use crate::attempt::RemoteAttemptAuthorizationFacts;
use crate::attempt::RemoteAttemptPhase;
use crate::attempt::RemoteAttemptReport;
use crate::attempt::RemoteAttemptReportKind;
use crate::attempt::RemoteAttemptReportPayload;
use crate::attempt::RemoteAttemptState;
use crate::attempt::canonical_remote_attempt_payload_digest;

/// Maximum distinct values in each identity domain of one bounded trace.
pub const MAX_TRELLIS_PROJECTION_IDENTITIES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionError {
    InvalidIdentity,
    InvalidDigest,
    InvalidFence,
    HistoryOutsideBound,
    IdentityCapacity,
    /// Payload validation, heartbeat time and checkpoint progress are not
    /// represented by the upstream kernel. They must not acquire proof cover.
    UnmodeledPayload,
}

/// Consumer fields corresponding to the producer's complete abstract state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedState {
    pub job: u64,
    pub attempt: u64,
    pub fence: u64,
    pub phase: u8,
    pub event_count: u32,
    pub result_present: bool,
    pub result_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedReport {
    pub job: u64,
    pub attempt: u64,
    pub fence: u64,
    pub event_id: u64,
    pub digest_id: u64,
    pub event_relation: u8,
    pub kind: u8,
    pub worker_authorized: bool,
    pub output_authorized: bool,
    pub result_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedInput {
    pub state: ProjectedState,
    pub report: ProjectedReport,
}

/// Full strings are keys; ordinal representatives never truncate a hash.
/// Keep this context for the entire bounded trace, including next-state
/// projection. Rebuilding it between steps would change identity numbering.
#[derive(Default)]
pub struct FiniteProjection {
    jobs: IdentityDomain,
    attempts: IdentityDomain,
    results: IdentityDomain,
    events: IdentityDomain,
    digests: IdentityDomain,
}

#[derive(Default)]
struct IdentityDomain(BTreeMap<String, u64>);

impl IdentityDomain {
    fn intern(&mut self, value: &str) -> Result<u64, ProjectionError> {
        if let Some(id) = self.0.get(value) {
            return Ok(*id);
        }
        if self.0.len() >= MAX_TRELLIS_PROJECTION_IDENTITIES {
            return Err(ProjectionError::IdentityCapacity);
        }
        // Zero is reserved for an absent result. The fixed bound makes this
        // checked conversion and addition representable on every target.
        let id = u64::try_from(self.0.len())
            .ok()
            .and_then(|length| length.checked_add(1))
            .ok_or(ProjectionError::IdentityCapacity)?;
        self.0.insert(value.to_string(), id);
        Ok(id)
    }
}

impl FiniteProjection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only admitted, modeled payload facts receive a representation. Complete
    /// retained history is consulted directly, never a prefix or digest cache.
    pub fn project(
        &mut self,
        current: &RemoteAttemptState,
        report: &RemoteAttemptReport,
        authorization: RemoteAttemptAuthorizationFacts,
    ) -> Result<ProjectedInput, ProjectionError> {
        validate_report(current, report)?;
        let state = self.project_state(current)?;
        let event_relation = match current.applied_events.get(&report.identity.event_id) {
            None => 0,
            Some(digest) if digest == &report.identity.payload_digest => 1,
            Some(_) => 2,
        };
        let projected = ProjectedReport {
            job: self.jobs.intern(report.identity.job_id.as_str())?,
            attempt: self.attempts.intern(report.identity.attempt_id.as_str())?,
            fence: report.identity.fence_generation.get(),
            event_id: self.events.intern(report.identity.event_id.as_str())?,
            digest_id: self.digests.intern(report.identity.payload_digest.as_str())?,
            event_relation,
            kind: project_kind(report.payload.kind()),
            worker_authorized: authorization.worker_authorized,
            output_authorized: authorization.output_admission_authorized,
            result_id: match &report.payload {
                RemoteAttemptReportPayload::ResultReady { output_digest_blake3 }
                | RemoteAttemptReportPayload::Completion { output_digest_blake3 } => {
                    self.results.intern(output_digest_blake3)?
                }
                _ => 0,
            },
        };
        Ok(ProjectedInput {
            state,
            report: projected,
        })
    }

    /// Use the same context for current and next states so result/identity
    /// changes cannot disappear through renumbering.
    pub fn project_state(&mut self, state: &RemoteAttemptState) -> Result<ProjectedState, ProjectionError> {
        validate_state(state)?;
        Ok(ProjectedState {
            job: self.jobs.intern(state.job_id.as_str())?,
            attempt: self.attempts.intern(state.attempt_id.as_str())?,
            fence: state.fence_generation.get(),
            phase: project_phase(state.phase),
            event_count: u32::try_from(state.applied_events.len()).map_err(|_| ProjectionError::HistoryOutsideBound)?,
            result_present: state.result_digest_blake3.is_some(),
            result_id: match &state.result_digest_blake3 {
                Some(digest) => self.results.intern(digest)?,
                None => 0,
            },
        })
    }
}

/// Exhaustive matches make a new consumer enum variant a compile-time failure.
pub const fn project_phase(phase: RemoteAttemptPhase) -> u8 {
    match phase {
        RemoteAttemptPhase::Queued => 0,
        RemoteAttemptPhase::Running => 1,
        RemoteAttemptPhase::Transferring => 2,
        RemoteAttemptPhase::FinishedUndelivered => 3,
        RemoteAttemptPhase::Completed => 4,
        RemoteAttemptPhase::Failed => 5,
        RemoteAttemptPhase::Superseded => 6,
    }
}

pub const fn project_kind(kind: RemoteAttemptReportKind) -> u8 {
    match kind {
        RemoteAttemptReportKind::Start => 0,
        RemoteAttemptReportKind::Heartbeat => 1,
        RemoteAttemptReportKind::LogAppend => 2,
        RemoteAttemptReportKind::TransferCheckpoint => 3,
        RemoteAttemptReportKind::ResultReady => 4,
        RemoteAttemptReportKind::Failure => 5,
        RemoteAttemptReportKind::Completion => 6,
    }
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_REMOTE_ATTEMPT_ID_BYTES && !value.chars().any(char::is_control)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_state(state: &RemoteAttemptState) -> Result<(), ProjectionError> {
    if !valid_identity(state.job_id.as_str()) || !valid_identity(state.attempt_id.as_str()) {
        return Err(ProjectionError::InvalidIdentity);
    }
    if state.fence_generation.get() == 0 {
        return Err(ProjectionError::InvalidFence);
    }
    if state.applied_events.len() > MAX_REMOTE_ATTEMPT_EVENTS {
        return Err(ProjectionError::HistoryOutsideBound);
    }
    if state.result_digest_blake3.as_deref().is_some_and(|digest| !valid_digest(digest)) {
        return Err(ProjectionError::InvalidDigest);
    }
    for (event, digest) in &state.applied_events {
        if !valid_identity(event.as_str()) {
            return Err(ProjectionError::InvalidIdentity);
        }
        if !valid_digest(digest.as_str()) {
            return Err(ProjectionError::InvalidDigest);
        }
    }
    debug_assert!(state.fence_generation.get() > 0);
    debug_assert!(state.applied_events.len() <= MAX_REMOTE_ATTEMPT_EVENTS);
    debug_assert!(state.applied_events.values().all(|digest| valid_digest(digest.as_str())));
    Ok(())
}

fn validate_report(current: &RemoteAttemptState, report: &RemoteAttemptReport) -> Result<(), ProjectionError> {
    if !valid_identity(report.identity.job_id.as_str())
        || !valid_identity(report.identity.attempt_id.as_str())
        || !valid_identity(report.identity.event_id.as_str())
    {
        return Err(ProjectionError::InvalidIdentity);
    }
    if report.identity.fence_generation.get() == 0 {
        return Err(ProjectionError::InvalidFence);
    }
    let digest =
        canonical_remote_attempt_payload_digest(&report.payload).map_err(|_| ProjectionError::UnmodeledPayload)?;
    if digest != report.identity.payload_digest {
        return Err(ProjectionError::InvalidDigest);
    }
    validate_modeled_payload(current, &report.payload)
}

fn payload_is_current(current: &RemoteAttemptState, payload: &RemoteAttemptReportPayload) -> bool {
    match payload {
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s } => {
            *observed_unix_s >= current.started_unix_s && *observed_unix_s <= current.deadline_unix_s
        }
        RemoteAttemptReportPayload::TransferCheckpoint { transferred_bytes, .. } => {
            *transferred_bytes >= current.transferred_bytes
        }
        RemoteAttemptReportPayload::ResultReady { output_digest_blake3 }
        | RemoteAttemptReportPayload::Completion { output_digest_blake3 } => valid_digest(output_digest_blake3),
        RemoteAttemptReportPayload::Start
        | RemoteAttemptReportPayload::LogAppend { .. }
        | RemoteAttemptReportPayload::Failure { .. } => true,
    }
}
fn payload_is_advanced(current: &RemoteAttemptState, payload: &RemoteAttemptReportPayload) -> bool {
    match payload {
        RemoteAttemptReportPayload::Heartbeat { observed_unix_s } => {
            current.last_heartbeat_unix_s.is_none_or(|previous| *observed_unix_s > previous)
        }
        RemoteAttemptReportPayload::TransferCheckpoint { checkpoint, .. } => {
            current.transfer_checkpoint.is_none_or(|previous| *checkpoint > previous)
        }
        RemoteAttemptReportPayload::ResultReady { .. }
        | RemoteAttemptReportPayload::Completion { .. }
        | RemoteAttemptReportPayload::Start
        | RemoteAttemptReportPayload::LogAppend { .. }
        | RemoteAttemptReportPayload::Failure { .. } => true,
    }
}
fn validate_modeled_payload(
    current: &RemoteAttemptState,
    payload: &RemoteAttemptReportPayload,
) -> Result<(), ProjectionError> {
    let is_current = payload_is_current(current, payload);
    let is_advanced = payload_is_advanced(current, payload);
    if !is_current || !is_advanced {
        return Err(ProjectionError::UnmodeledPayload);
    }
    debug_assert!(is_current);
    debug_assert!(is_advanced);
    Ok(())
}

#[cfg(test)]
#[path = "trellis_projection_tests.rs"]
mod tests;
