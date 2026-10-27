//! Bounded asynchronous-job API. This module owns observations, never execution
//! authority. A host must authenticate each call and implement `Services` using
//! the existing fenced attempt, quota, log-redaction and result-verification
//! services. There is deliberately no compatibility-ticket or UCAN constructor.
//!
//! The API is callable independently of a transport. Decode HTTP/RPC envelopes
//! with a bounded decoder before constructing `Request`; never deserialize
//! `Authority` from a caller. Public deployment remains an operator concern.

use std::path::Path;

use crunch_nix_gateway_core::Authority;
use crunch_nix_gateway_core::{self as core};
use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

#[path = "api/crypto.rs"]
mod crypto;
#[path = "api/journal.rs"]
mod journal;

pub use crypto::Completion;
pub use crypto::verify_completion;
use journal::Journal;
use journal::Record;

pub const API_VERSION: u32 = 1;
pub const MAX_REQUEST_BYTES: usize = 4096;
pub const MAX_ATTEMPTS: usize = 1024;
pub const MAX_EVENTS_PER_ATTEMPT: usize = 64;
pub const MAX_JOURNAL_BYTES: u64 = 16 * 1024 * 1024;
pub const CURSOR_TTL_SECS: u64 = 900;
pub const MAX_USAGE_ITEMS: u32 = core::MAX_EVENT_PAGE_ITEMS;

/// Closed redacted errors. Host errors must be mapped here, never formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Rejected(core::Reject),
    Unknown,
    Cursor,
    Storage,
    Busy,
    Host,
    Integrity,
    Conflict,
}

impl Error {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Rejected(reason) => reason.code(),
            Self::Unknown => "gateway-attempt-unknown",
            Self::Cursor => "gateway-cursor-invalid",
            Self::Storage => "gateway-journal-unavailable",
            Self::Busy => "gateway-api-busy",
            Self::Host => "gateway-host-unavailable",
            Self::Integrity => "gateway-evidence-invalid",
            Self::Conflict => "gateway-transition-conflict",
        }
    }

    /// Decode only the stable, credential-free wire vocabulary.
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "gateway-version-unsupported" => Self::Rejected(core::Reject::Version),
            "gateway-operation-unsupported" => Self::Rejected(core::Reject::Operation),
            "gateway-limit-exceeded" => Self::Rejected(core::Reject::Bound),
            "gateway-identity-invalid" => Self::Rejected(core::Reject::Identity),
            "gateway-authority-expired" => Self::Rejected(core::Reject::Expired),
            "gateway-audience-mismatch" => Self::Rejected(core::Reject::Audience),
            "gateway-scope-mismatch" => Self::Rejected(core::Reject::Scope),
            "gateway-authority-denied" => Self::Rejected(core::Reject::Authority),
            "gateway-ticket-admin-denied" => Self::Rejected(core::Reject::TicketAdministration),
            "gateway-attempt-owner-mismatch" => Self::Rejected(core::Reject::NotOwner),
            "gateway-idempotency-conflict" => Self::Rejected(core::Reject::Conflict),
            "gateway-fence-stale" => Self::Rejected(core::Reject::StaleFence),
            "gateway-attempt-unknown" => Self::Unknown,
            "gateway-cursor-invalid" => Self::Cursor,
            "gateway-journal-unavailable" => Self::Storage,
            "gateway-api-busy" => Self::Busy,
            "gateway-host-unavailable" => Self::Host,
            "gateway-evidence-invalid" => Self::Integrity,
            "gateway-transition-conflict" => Self::Conflict,
            _ => return None,
        })
    }
}

impl From<core::Reject> for Error {
    fn from(value: core::Reject) -> Self {
        Self::Rejected(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl Phase {
    pub const fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

/// Only public, content-addressed evidence references cross this boundary.
/// A completion signature does not establish action-result usability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub phase: Phase,
    pub fence: u64,
    pub result_evidence: Option<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub sequence: u64,
    pub phase: Phase,
    pub completion: Option<Completion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Status {
    pub attempt_id: String,
    pub request_digest: [u8; 32],
    pub observation: Observation,
    pub cancel_requested: bool,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Page<T> {
    pub values: Vec<T>,
    pub next_cursor: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogWindow {
    pub bytes: Vec<u8>,
    pub next_cursor: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Usage {
    pub admitted: u64,
    pub active: u64,
    pub completed: u64,
    pub cpu_millis: u64,
    pub output_bytes: u64,
    pub accounting: Option<UsageAccounting>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsageAccounting {
    pub schedule_version: u32,
    pub cpu_ms_per_unit: u64,
    pub memory_byte_ms_per_unit: u64,
    pub executor_output_bytes_per_unit: u64,
    pub project_limit_units: u64,
    pub account_limit_units: u64,
    pub project_reason_counts: UsageReasonCounts,
    pub account_reason_counts: UsageReasonCounts,
    pub unit: String,
    pub window_id: String,
    pub window_start_s: u64,
    pub window_end_s: u64,
    pub project_reserved_units: u64,
    pub project_charged_units: u64,
    pub account_reserved_units: u64,
    pub account_charged_units: u64,
    pub memory_measurement: String,
    pub output_measurement: String,
}

/// Fixed public reason codes; counts cover the accounting window, not the
/// caller's bounded attempt sample. Pending reasons retain their reservation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct UsageReasonCounts {
    pub reserved: u64,
    pub settled: u64,
    pub cancelled_without_final_usage: u64,
    pub worker_lost: u64,
    pub partial_observation: u64,
    pub untrusted_observation: u64,
    pub usage_exceeds_reservation: u64,
}

#[derive(Clone, Copy)]
pub enum Request<'a> {
    Submit {
        drv_path: &'a str,
        idempotency_key: &'a str,
    },
    Status {
        attempt_id: &'a str,
    },
    Logs {
        attempt_id: &'a str,
        cursor: Option<&'a str>,
        bytes: u32,
    },
    Events {
        attempt_id: &'a str,
        cursor: Option<&'a str>,
        items: u32,
    },
    Cancel {
        attempt_id: &'a str,
        fence: u64,
    },
    Result {
        attempt_id: &'a str,
    },
    Usage {
        items: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Response {
    Submitted { attempt_id: String, recovered: bool },
    Status(Status),
    Logs(LogWindow),
    Events(Page<Event>),
    CancelRequested { attempt_id: String, terminal: bool },
    Result(Option<Completion>),
    Usage(Usage),
}

/// A verified invocation retained for the entire admitted operation. Implementors
/// keep the actual nonclone live capability here, not just copied claims.
pub trait Authorization: Sync {
    fn facts(&self) -> Authority<'_>;
}

/// Host integration contract; there are no accepting default methods.
///
/// * `submit` MUST atomically reserve existing resource quotas and create or recover the existing
///   durable/fenced attempt using `public_attempt_id` as the stable operation identity. Repeated
///   calls after crash must not execute twice or charge quota twice. A journal intent alone is NOT
///   admission.
/// * `observe` MUST check live authority/capability and return authoritative attempt state/fence,
///   not infer completion from process disappearance.
/// * `cancel` MUST be idempotent by the same public ID, enforce the supplied live fence, and settle
///   quotas only after actual teardown.
/// * `logs` MUST return already secret-scrubbed public bytes, at most `maximum`, starting exactly
///   at `offset`, without reading an unbounded host log.
/// * `verify_result` MUST apply current policy, platform, output identity, signed-result and CAS
///   availability checks; it grants no publication.
/// * `usage` MUST query only the authenticated subject/scope and bounded items.
///
/// All methods receive server-created verified facts, not bearer material.
/// The host remains responsible for revalidation/revocation on each dispatch.
pub trait Services: Sync {
    type Session: Sync;
    type Authorization: Authorization;
    /// Revalidate a transport-authenticated, host-owned session on EVERY call.
    /// Never accept permissions or credential kind supplied in an API request.
    fn authenticate<'a>(
        &'a self,
        session: &'a Self::Session,
        now_unix_s: u64,
        request: Request<'a>,
    ) -> impl Future<Output = Result<Self::Authorization, Error>> + Send + 'a;
    fn submit(
        &self,
        session: &Self::Session,
        authority: &Self::Authorization,
        public_attempt_id: &str,
        drv_path: &str,
        request_digest: [u8; 32],
    ) -> impl Future<Output = Result<Observation, Error>> + Send;
    fn observe(
        &self,
        authority: &Self::Authorization,
        public_attempt_id: &str,
    ) -> impl Future<Output = Result<Observation, Error>> + Send;
    fn cancel(
        &self,
        authority: &Self::Authorization,
        public_attempt_id: &str,
        fence: u64,
    ) -> impl Future<Output = Result<(), Error>> + Send;
    fn logs(
        &self,
        authority: &Self::Authorization,
        public_attempt_id: &str,
        offset: u64,
        maximum: u32,
    ) -> impl Future<Output = Result<Vec<u8>, Error>> + Send;
    fn verify_result(
        &self,
        session: &Self::Session,
        authority: &Self::Authorization,
        public_attempt_id: &str,
        evidence: [u8; 32],
    ) -> impl Future<Output = Result<(), Error>> + Send;
    fn usage(
        &self,
        session: &Self::Session,
        authority: &Self::Authorization,
        items: u32,
    ) -> impl Future<Output = Result<Usage, Error>> + Send;
}

pub struct Api<S> {
    journal: Journal,
    services: S,
    audience: String,
    policy: [u8; 32],
    signer: SigningKey,
    operations: std::sync::atomic::AtomicU32,
}

impl<S: Services> Api<S> {
    /// `directory` must already be private (0700), owner controlled, and
    /// persistent. Supply a persistent random secret distinct from credentials;
    /// changing it fails journal verification and invalidates old cursors.
    /// Audience, policy or signing-key rotation requires a new journal namespace
    /// and retained old verifier keys. Never silently reset damaged state.
    pub fn open(
        directory: &Path,
        secret: [u8; 32],
        signer: SigningKey,
        audience: &str,
        policy: [u8; 32],
        services: S,
    ) -> Result<Self, Error> {
        if !identity(audience) {
            return Err(core::Reject::Identity.into());
        }
        let namespace_secret = crypto::slot(&secret, &["gateway-api-namespace-v1", audience, &hex(&policy)]);
        let journal = Journal::open(directory, namespace_secret, signer.verifying_key())?;
        Ok(Self {
            journal,
            services,
            audience: audience.into(),
            policy,
            signer,
            operations: std::sync::atomic::AtomicU32::new(0),
        })
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signer.verifying_key()
    }

    // r[impl remote_builds.versioned_build_api]
    // r[impl remote_builds.gateway_bounds_and_recovery]
    pub async fn handle(
        &self,
        version: u32,
        session: &S::Session,
        now_unix_s: u64,
        request: Request<'_>,
    ) -> Result<Response, Error> {
        if version != API_VERSION {
            return Err(core::Reject::Version.into());
        }
        let _operation = Operation::enter(&self.operations)?;
        let authority = self.services.authenticate(session, now_unix_s, request).await?;
        match request {
            Request::Submit {
                drv_path,
                idempotency_key,
            } => self.submit(session, &authority, now_unix_s, drv_path, idempotency_key).await,
            Request::Usage { items } => {
                if items == 0 || items > MAX_USAGE_ITEMS {
                    return Err(core::Reject::Bound.into());
                }
                self.admit(core::Request::Usage { items }, authority.facts(), now_unix_s)?;
                Ok(Response::Usage(self.services.usage(session, &authority, items).await?))
            }
            request => self.attempt_request(session, &authority, now_unix_s, request).await,
        }
    }

    fn admit(&self, request: core::Request<'_>, authority: Authority<'_>, now: u64) -> Result<(), Error> {
        core::admit(request, authority, now, &self.audience, authority.scope)?;
        Ok(())
    }

    async fn submit(
        &self,
        session: &S::Session,
        authorization: &S::Authorization,
        now: u64,
        drv: &str,
        key: &str,
    ) -> Result<Response, Error> {
        let authority = authorization.facts();
        if drv.len().saturating_add(key.len()) > MAX_REQUEST_BYTES {
            return Err(core::Reject::Bound.into());
        }
        let digest = *blake3::hash(drv.as_bytes()).as_bytes();
        let digest_text = hex(&digest);
        self.admit(
            core::Request::Build {
                drv_path: drv,
                key,
                digest: &digest_text,
                existing: None,
            },
            authority,
            now,
        )?;
        let slot = self.journal.submission_slot(authority.subject, authority.scope, key);
        let (record, recovered) = self.journal.update(|state| {
            if let Some(record) = state.records.get(&slot) {
                if record.digest != digest {
                    return Err(core::Reject::Conflict.into());
                }
                return Ok((record.clone(), true));
            }
            if state.records.len() >= MAX_ATTEMPTS {
                return Err(core::Reject::Bound.into());
            }
            let mut random = [0_u8; 32];
            getrandom::fill(&mut random).map_err(|_| Error::Storage)?;
            let record = Record::pending(hex(&random), authority, digest, self.policy);
            state.records.insert(slot.clone(), record.clone());
            Ok((record, false))
        })?;
        // Intent and admission are durable, but a host may have stopped before
        // dispatch. Every retry reaches the REQUIRED idempotent host operation
        // under current per-call authority and the same opaque public ID.
        let observation = self.services.submit(session, authorization, &record.id, drv, digest).await?;
        self.record_observation(&record.id, observation)?;
        Ok(Response::Submitted {
            attempt_id: record.id,
            recovered,
        })
    }

    async fn attempt_request(
        &self,
        session: &S::Session,
        authorization: &S::Authorization,
        now: u64,
        request: Request<'_>,
    ) -> Result<Response, Error> {
        let authority = authorization.facts();
        let id = match request {
            Request::Status { attempt_id }
            | Request::Logs { attempt_id, .. }
            | Request::Events { attempt_id, .. }
            | Request::Cancel { attempt_id, .. }
            | Request::Result { attempt_id } => attempt_id,
            _ => return Err(core::Reject::Operation.into()),
        };
        if !identity(id) {
            return Err(core::Reject::Identity.into());
        }
        let record = self.journal.find(id)?;
        if !record.admitted {
            return Err(Error::Busy);
        }
        let attempt = core::Attempt {
            id,
            subject: &record.subject,
            scope: &record.scope,
            fence: record.observation.fence,
        };
        let command = match request {
            Request::Status { .. } => core::Request::Status { attempt },
            Request::Logs { bytes, .. } => core::Request::Log { attempt, bytes },
            Request::Events { items, .. } => core::Request::EventPage { attempt, items },
            Request::Cancel { fence, .. } => core::Request::Cancel { attempt, fence },
            Request::Result { .. } => core::Request::SignedResult { attempt },
            _ => return Err(core::Reject::Operation.into()),
        };
        self.admit(command, authority, now)?;
        match request {
            Request::Status { .. } => {
                self.refresh(authorization, id).await?;
                let current = self.journal.find(id)?;
                Ok(Response::Status(current.status()))
            }
            Request::Logs { cursor, bytes, .. } => self.logs(authorization, now, id, cursor, bytes).await,
            Request::Events { cursor, items, .. } => {
                let offset = self.journal.cursor_read(cursor, authority, id, b'e', now)?;
                self.refresh(authorization, id).await?;
                let current = self.journal.find(id)?;
                if offset > current.events.len() as u64 {
                    return Err(Error::Cursor);
                }
                let events: Vec<_> =
                    current.events.iter().skip(offset as usize).take(items as usize).cloned().collect();
                let next = offset.checked_add(events.len() as u64).ok_or(Error::Cursor)?;
                Ok(Response::Events(Page {
                    values: events,
                    next_cursor: self.journal.cursor_write(authority, id, b'e', next, now)?,
                }))
            }
            Request::Cancel { fence, .. } => self.cancel(authorization, id, fence).await,
            Request::Result { .. } => {
                self.refresh(authorization, id).await?;
                let current = self.journal.find(id)?;
                let completion = current.events.last().and_then(|event| event.completion.clone());
                if let Some(evidence) = completion.as_ref().and_then(|event| event.result_evidence) {
                    self.services.verify_result(session, authorization, id, evidence).await?;
                }
                Ok(Response::Result(completion))
            }
            _ => Err(core::Reject::Operation.into()),
        }
    }

    async fn logs(
        &self,
        authorization: &S::Authorization,
        now: u64,
        id: &str,
        cursor: Option<&str>,
        maximum: u32,
    ) -> Result<Response, Error> {
        let authority = authorization.facts();
        if maximum == 0 {
            return Err(core::Reject::Bound.into());
        }
        let offset = self.journal.cursor_read(cursor, authority, id, b'l', now)?;
        let bytes = self.services.logs(authorization, id, offset, maximum).await?;
        if bytes.len() > maximum as usize {
            return Err(Error::Integrity);
        }
        let next = offset.checked_add(bytes.len() as u64).ok_or(Error::Cursor)?;
        Ok(Response::Logs(LogWindow {
            bytes,
            next_cursor: self.journal.cursor_write(authority, id, b'l', next, now)?,
        }))
    }

    async fn cancel(&self, authorization: &S::Authorization, id: &str, fence: u64) -> Result<Response, Error> {
        let terminal = self.journal.update(|state| {
            let record = state.by_id_mut(id)?;
            if record.observation.fence != fence {
                return Err(core::Reject::StaleFence.into());
            }
            if record.observation.phase.terminal() {
                return Ok(true);
            }
            record.cancel_requested = true;
            Ok(false)
        })?;
        if !terminal {
            self.services.cancel(authorization, id, fence).await?;
        }
        Ok(Response::CancelRequested {
            attempt_id: id.into(),
            terminal,
        })
    }

    async fn refresh(&self, authorization: &S::Authorization, id: &str) -> Result<(), Error> {
        let observation = self.services.observe(authorization, id).await?;
        self.record_observation(id, observation)
    }

    fn record_observation(&self, id: &str, observation: Observation) -> Result<(), Error> {
        self.journal.update(|state| {
            let record = state.by_id_mut(id)?;
            record.observe(observation, &self.signer)
        })
    }
}

fn identity(value: &str) -> bool {
    !value.is_empty() && value.len() <= core::MAX_IDENTITY_BYTES && value.bytes().all(|b| b.is_ascii_graphic())
}

fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut result = String::with_capacity(64);
    for byte in bytes {
        write!(result, "{byte:02x}").expect("String write");
    }
    result
}

struct Operation<'a>(&'a std::sync::atomic::AtomicU32);
impl<'a> Operation<'a> {
    fn enter(counter: &'a std::sync::atomic::AtomicU32) -> Result<Self, Error> {
        use std::sync::atomic::Ordering::AcqRel;
        use std::sync::atomic::Ordering::Acquire;
        counter
            .fetch_update(AcqRel, Acquire, |value| (value < core::MAX_CONCURRENT_OPERATIONS).then(|| value + 1))
            .map_err(|_| Error::Busy)?;
        Ok(Self(counter))
    }
}
impl Drop for Operation<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}

#[cfg(test)]
#[path = "api/tests.rs"]
mod tests;
