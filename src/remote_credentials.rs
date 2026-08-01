use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use subtle::ConstantTimeEq as _;
use zeroize::Zeroize as _;
use zeroize::Zeroizing;

pub const REMOTE_TICKET_STATE_SCHEMA_VERSION: u32 = 2;
pub const TICKET_ENTROPY_BYTES: usize = 32;
pub const TICKET_VERIFIER_KEY_BYTES: usize = 32;
pub const TICKET_VERIFIER_BYTES: usize = 32;
pub const TICKET_TOKEN_BASE64URL_BYTES: usize = 43;
pub const TICKET_TTL_SECS_MAX: u64 = 2_592_000;
pub const TICKET_USES_MAX: u32 = 100_000;
pub const TICKET_BUILD_TIME_SECS_MAX: u64 = 86_400;
pub const TICKET_UPLOAD_BYTES_MAX: u64 = 1_099_511_627_776;
pub const TICKET_BOUND_ENDPOINT_BYTES_MAX: usize = 256;
pub const MAX_TICKET_DISPLAY_NAME_BYTES: usize = 128;
pub const MAX_TICKET_KEY_ID_BYTES: usize = 128;
pub const MAX_REMOTE_TICKETS: usize = 4_096;
pub const MAX_INVALIDATED_LEGACY_TICKET_IDS: usize = MAX_REMOTE_TICKETS;

const TICKET_IDENTITY_DOMAIN: &[u8] = b"mantle-remote-ticket-identity-v2\0";
const TICKET_VERIFIER_DOMAIN: &[u8] = b"mantle-remote-ticket-verifier-v2\0";
const TICKET_ID_HEX_CHARS: usize = 32;
const HASH_LENGTH_PREFIX_BYTES: usize = std::mem::size_of::<u64>();
const ALL_ZERO_ENTROPY_BYTE: u8 = 0;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TicketId(String);

impl TicketId {
    pub fn parse_compatible(value: &str) -> Result<Self, String> {
        validate_legacy_ticket_id(value)?;
        debug_assert!(!value.is_empty());
        debug_assert!(value.len() <= TICKET_ID_HEX_CHARS);
        Ok(Self(value.to_string()))
    }

    fn parse_generated(value: &str) -> Result<Self, String> {
        validate_generated_ticket_id(value)?;
        debug_assert_eq!(value.len(), TICKET_ID_HEX_CHARS);
        debug_assert!(value.bytes().all(|byte| !byte.is_ascii_uppercase()));
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TicketVerifierKeyId(String);

impl TicketVerifierKeyId {
    pub fn parse(value: &str) -> Result<Self, String> {
        validate_key_id(value)?;
        debug_assert!(!value.is_empty());
        debug_assert!(value.len() <= MAX_TICKET_KEY_ID_BYTES);
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketTtl(u64);

impl TicketTtl {
    pub fn new(ttl_secs: u64) -> Result<Self, String> {
        if ttl_secs == 0 {
            return Err("remote-ticket-ttl-or-uses-zero".to_string());
        }
        if ttl_secs > TICKET_TTL_SECS_MAX {
            return Err("remote-ticket-ttl-limit-exceeded".to_string());
        }
        debug_assert!(ttl_secs > 0);
        debug_assert!(ttl_secs <= TICKET_TTL_SECS_MAX);
        Ok(Self(ttl_secs))
    }

    pub fn seconds(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketValidityWindow {
    created_unix_s: u64,
    expires_unix_s: u64,
}

impl TicketValidityWindow {
    pub fn from_ttl(created_unix_s: u64, ttl: TicketTtl) -> Result<Self, String> {
        let expires_unix_s = created_unix_s
            .checked_add(ttl.seconds())
            .ok_or_else(|| "remote-ticket-expiry-overflow".to_string())?;
        Self::from_bounds(created_unix_s, expires_unix_s)
    }

    pub fn from_bounds(created_unix_s: u64, expires_unix_s: u64) -> Result<Self, String> {
        let span_secs = expires_unix_s
            .checked_sub(created_unix_s)
            .ok_or_else(|| "remote-ticket-validity-window-invalid".to_string())?;
        TicketTtl::new(span_secs).map_err(|_| "remote-ticket-validity-window-invalid".to_string())?;
        debug_assert!(expires_unix_s > created_unix_s);
        debug_assert!(span_secs <= TICKET_TTL_SECS_MAX);
        Ok(Self {
            created_unix_s,
            expires_unix_s,
        })
    }

    pub fn created_unix_s(self) -> u64 {
        self.created_unix_s
    }

    pub fn expires_unix_s(self) -> u64 {
        self.expires_unix_s
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketUseLimit(u32);

impl TicketUseLimit {
    pub fn new(uses: u32) -> Result<Self, String> {
        if uses == 0 {
            return Err("remote-ticket-ttl-or-uses-zero".to_string());
        }
        if uses > TICKET_USES_MAX {
            return Err("remote-ticket-use-limit-exceeded".to_string());
        }
        debug_assert!(uses > 0);
        debug_assert!(uses <= TICKET_USES_MAX);
        Ok(Self(uses))
    }

    pub fn count(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketUsesRemaining(u32);

impl TicketUsesRemaining {
    pub fn new(uses: u32) -> Result<Self, String> {
        if uses > TICKET_USES_MAX {
            return Err("remote-ticket-use-limit-exceeded".to_string());
        }
        debug_assert!(uses <= TICKET_USES_MAX);
        debug_assert_eq!(uses.min(TICKET_USES_MAX), uses);
        Ok(Self(uses))
    }

    pub fn count(self) -> u32 {
        self.0
    }

    pub fn redeem(self) -> Result<Self, String> {
        let next = self.0.checked_sub(1).ok_or_else(|| "ticket-exhausted".to_string())?;
        let remaining = Self::new(next)?;
        debug_assert!(remaining.count() < self.count());
        debug_assert_eq!(remaining.count().checked_add(1), Some(self.count()));
        Ok(remaining)
    }
}

pub fn redeem_ticket_use(uses_remaining: u32) -> Result<u32, String> {
    let current =
        TicketUsesRemaining::new(uses_remaining).map_err(|_| "remote-ticket-state-use-limit-exceeded".to_string())?;
    let next = current.redeem()?;
    debug_assert!(next.count() < uses_remaining);
    debug_assert_eq!(next.count().checked_add(1), Some(uses_remaining));
    Ok(next.count())
}

/// A checked build-time role cannot be replaced with an upload-byte role.
///
/// ```compile_fail
/// use mantle::remote_credentials::{BuildTimeLimit, UploadByteLimit};
/// fn accepts_build_time(_: BuildTimeLimit) {}
/// let upload = UploadByteLimit::new(1).unwrap();
/// accepts_build_time(upload);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildTimeLimit(u64);

impl BuildTimeLimit {
    pub fn new(seconds: u64) -> Result<Self, String> {
        if seconds == 0 {
            return Err("remote-ticket-build-time-zero".to_string());
        }
        if seconds > TICKET_BUILD_TIME_SECS_MAX {
            return Err("remote-ticket-build-time-limit-exceeded".to_string());
        }
        debug_assert!(seconds > 0);
        debug_assert!(seconds <= TICKET_BUILD_TIME_SECS_MAX);
        Ok(Self(seconds))
    }

    pub fn seconds(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UploadByteLimit(u64);

impl UploadByteLimit {
    pub fn new(bytes: u64) -> Result<Self, String> {
        if bytes > TICKET_UPLOAD_BYTES_MAX {
            return Err("remote-ticket-upload-limit-exceeded".to_string());
        }
        debug_assert!(bytes <= TICKET_UPLOAD_BYTES_MAX);
        debug_assert_eq!(bytes.min(TICKET_UPLOAD_BYTES_MAX), bytes);
        Ok(Self(bytes))
    }

    pub fn bytes(self) -> u64 {
        self.0
    }
}

/// Issued bearer material has no ordinary display or Serde path.
///
/// ```compile_fail
/// use mantle::remote_credentials::IssuedBearerToken;
/// let _: IssuedBearerToken = serde_json::from_str("\"secret\"").unwrap();
/// ```
///
/// ```compile_fail
/// use mantle::remote_credentials::IssuedBearerToken;
/// fn render(token: IssuedBearerToken) { println!("{token}"); }
/// ```
pub struct IssuedBearerToken(String);

impl IssuedBearerToken {
    fn from_encoded(value: String) -> Self {
        debug_assert_eq!(value.len(), TICKET_TOKEN_BASE64URL_BYTES);
        debug_assert!(!value.is_empty());
        Self(value)
    }

    fn expose_to_sink(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for IssuedBearerToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IssuedBearerToken(<redacted>)")
    }
}

impl Drop for IssuedBearerToken {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Presented bearer material has no direct deserialization path.
///
/// ```compile_fail
/// use mantle::remote_credentials::PresentedBearerToken;
/// let _: PresentedBearerToken = serde_json::from_str("\"secret\"").unwrap();
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct PresentedBearerToken([u8; TICKET_ENTROPY_BYTES]);

impl PresentedBearerToken {
    pub fn parse(value: &str) -> Result<Self, String> {
        let mut decoded = decode_ticket_token(value)?;
        let bytes = std::mem::take(&mut *decoded);
        debug_assert_eq!(bytes.len(), TICKET_ENTROPY_BYTES);
        debug_assert!(value.len() == TICKET_TOKEN_BASE64URL_BYTES);
        Ok(Self(bytes))
    }

    fn bytes(&self) -> &[u8; TICKET_ENTROPY_BYTES] {
        &self.0
    }
}

impl fmt::Debug for PresentedBearerToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PresentedBearerToken(<redacted>)")
    }
}

impl Drop for PresentedBearerToken {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug)]
pub struct AdmittedTicketPolicy<'a> {
    ticket_id: TicketId,
    verifier_key_id: TicketVerifierKeyId,
    verifier: &'a TicketVerifier,
    validity: TicketValidityWindow,
    uses_remaining: TicketUsesRemaining,
    build_time_limit: BuildTimeLimit,
    upload_byte_limit: UploadByteLimit,
    bound_client_endpoint: Option<String>,
    revoked: bool,
}

impl AdmittedTicketPolicy<'_> {
    pub fn build_time_limit(&self) -> BuildTimeLimit {
        self.build_time_limit
    }

    pub fn upload_byte_limit(&self) -> UploadByteLimit {
        self.upload_byte_limit
    }
}

struct AdmittedTicketPresentation {
    ticket_id: TicketId,
    bearer: PresentedBearerToken,
    client_endpoint: Option<String>,
    now_unix_s: u64,
}

struct AdmittedTicketCredential<'a> {
    policy: AdmittedTicketPolicy<'a>,
    presentation: AdmittedTicketPresentation,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TicketVerifierKey {
    id: TicketVerifierKeyId,
    bytes: [u8; TICKET_VERIFIER_KEY_BYTES],
}

impl TicketVerifierKey {
    pub fn parse(value: &str) -> Result<Self, String> {
        let (id, encoded) =
            value.split_once(':').ok_or_else(|| "remote-ticket-verifier-key-format-invalid".to_string())?;
        let id = TicketVerifierKeyId::parse(id)?;
        let mut bytes = decode_canonical_base64url::<TICKET_VERIFIER_KEY_BYTES>(CanonicalBase64DecodeInput {
            encoded,
            error: "remote-ticket-verifier-key-encoding-invalid",
        })?;
        if bytes.iter().all(|byte| *byte == ALL_ZERO_ENTROPY_BYTE) {
            return Err("remote-ticket-verifier-key-all-zero".to_string());
        }
        debug_assert!(bytes.iter().any(|byte| *byte != ALL_ZERO_ENTROPY_BYTE));
        debug_assert_eq!(bytes.len(), TICKET_VERIFIER_KEY_BYTES);
        Ok(Self {
            id,
            bytes: std::mem::take(&mut *bytes),
        })
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    fn key_id(&self) -> &TicketVerifierKeyId {
        &self.id
    }

    fn bytes(&self) -> &[u8; TICKET_VERIFIER_KEY_BYTES] {
        &self.bytes
    }
}

impl fmt::Debug for TicketVerifierKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TicketVerifierKey")
            .field("id", &self.id.as_str())
            .field("bytes", &"<redacted>")
            .finish()
    }
}

impl Drop for TicketVerifierKey {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TicketVerifier([u8; TICKET_VERIFIER_BYTES]);

impl TicketVerifier {
    fn constant_time_eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }

    #[cfg(test)]
    fn encoded(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }
}

impl Drop for TicketVerifier {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for TicketVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TicketVerifier(<redacted>)")
    }
}

impl Serialize for TicketVerifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(self.0));
        serializer.serialize_str(&encoded)
    }
}

impl<'de> Deserialize<'de> for TicketVerifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let encoded = Zeroizing::new(String::deserialize(deserializer)?);
        let mut decoded = decode_canonical_base64url::<TICKET_VERIFIER_BYTES>(CanonicalBase64DecodeInput {
            encoded: &encoded,
            error: "remote-ticket-verifier-encoding-invalid",
        })
        .map_err(serde::de::Error::custom)?;
        Ok(Self(std::mem::take(&mut *decoded)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteTicket {
    pub id: String,
    pub display_name: String,
    pub verifier_key_id: String,
    pub verifier: TicketVerifier,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub uses_remaining: u32,
    pub max_build_time_secs: u64,
    pub max_upload_bytes: u64,
    pub bound_client_endpoint: Option<String>,
    pub revoked: bool,
    #[serde(skip)]
    runtime_verifier_key: Option<Arc<TicketVerifierKey>>,
}

impl PartialEq for RemoteTicket {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.display_name == other.display_name
            && self.verifier_key_id == other.verifier_key_id
            && self.verifier == other.verifier
            && self.created_unix_s == other.created_unix_s
            && self.expires_unix_s == other.expires_unix_s
            && self.uses_remaining == other.uses_remaining
            && self.max_build_time_secs == other.max_build_time_secs
            && self.max_upload_bytes == other.max_upload_bytes
            && self.bound_client_endpoint == other.bound_client_endpoint
            && self.revoked == other.revoked
    }
}

impl Eq for RemoteTicket {}

impl RemoteTicket {
    pub fn bind_active_verifier_key(&mut self, key: &Arc<TicketVerifierKey>) {
        if self.verifier_key_id == key.id() {
            self.runtime_verifier_key = Some(Arc::clone(key));
        } else {
            self.runtime_verifier_key = None;
        }
    }

    pub fn clear_active_verifier_key(&mut self) {
        self.runtime_verifier_key = None;
    }

    fn active_verifier_key(&self) -> Option<&TicketVerifierKey> {
        self.runtime_verifier_key.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn fixture(
        id: &str,
        display_name: &str,
        token: &str,
        key: &Arc<TicketVerifierKey>,
        created_unix_s: u64,
        expires_unix_s: u64,
        uses_remaining: u32,
        max_build_time_secs: u64,
        max_upload_bytes: u64,
        bound_client_endpoint: Option<String>,
    ) -> Self {
        let token_bytes = decode_ticket_token(token).expect("fixture token must be canonical");
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            verifier_key_id: key.id().to_string(),
            verifier: keyed_ticket_verifier(key, &token_bytes),
            created_unix_s,
            expires_unix_s,
            uses_remaining,
            max_build_time_secs,
            max_upload_bytes,
            bound_client_endpoint,
            revoked: false,
            runtime_verifier_key: Some(Arc::clone(key)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteTicketView {
    pub id: String,
    pub display_name: String,
    pub verifier_key_id: String,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub uses_remaining: u32,
    pub max_build_time_secs: u64,
    pub max_upload_bytes: u64,
    pub bound_client_endpoint: Option<String>,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteTicketState {
    pub schema_version: u32,
    pub next_ticket_sequence: u64,
    pub tickets: BTreeMap<String, RemoteTicket>,
    #[serde(default = "empty_invalidated_legacy_ticket_ids")]
    pub invalidated_legacy_ticket_ids: BTreeSet<String>,
}

fn empty_invalidated_legacy_ticket_ids() -> BTreeSet<String> {
    let ids = BTreeSet::new();
    debug_assert!(ids.is_empty());
    debug_assert_eq!(ids.len(), 0);
    ids
}

impl Default for RemoteTicketState {
    fn default() -> Self {
        Self {
            schema_version: REMOTE_TICKET_STATE_SCHEMA_VERSION,
            next_ticket_sequence: 0,
            tickets: BTreeMap::new(),
            invalidated_legacy_ticket_ids: BTreeSet::new(),
        }
    }
}

impl RemoteTicketState {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != REMOTE_TICKET_STATE_SCHEMA_VERSION {
            return Err("remote-ticket-state-schema-version-unsupported".to_string());
        }
        if self.tickets.len() > MAX_REMOTE_TICKETS {
            return Err("remote-ticket-state-ticket-count-exceeded".to_string());
        }
        if self.invalidated_legacy_ticket_ids.len() > MAX_INVALIDATED_LEGACY_TICKET_IDS {
            return Err("remote-ticket-state-invalidated-id-count-exceeded".to_string());
        }
        for (map_id, ticket) in &self.tickets {
            admit_ticket_record(map_id, ticket, TicketIdAdmission::GeneratedState).map(drop)?;
            if self.invalidated_legacy_ticket_ids.contains(map_id) {
                return Err("remote-ticket-state-active-invalidated-id-overlap".to_string());
            }
        }
        for id in &self.invalidated_legacy_ticket_ids {
            validate_legacy_ticket_id(id)?;
        }
        Ok(())
    }

    pub fn bind_active_verifier_key(&mut self, key: &Arc<TicketVerifierKey>) {
        for ticket in self.tickets.values_mut() {
            ticket.bind_active_verifier_key(key);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketIssueInput {
    pub display_name: String,
    pub now_unix_s: u64,
    pub ttl_secs: u64,
    pub uses: u32,
    pub max_build_time_secs: u64,
    pub max_upload_bytes: u64,
    pub bound_client_endpoint: Option<String>,
}

#[derive(Debug)]
struct AdmittedTicketIssueInput {
    display_name: String,
    ttl: TicketTtl,
    validity: TicketValidityWindow,
    use_limit: TicketUseLimit,
    build_time_limit: BuildTimeLimit,
    upload_byte_limit: UploadByteLimit,
    bound_client_endpoint: Option<String>,
}

pub struct TicketIssuePlan {
    ticket: RemoteTicket,
    issued_bearer: IssuedBearerToken,
    next_ticket_sequence: u64,
}

impl TicketIssuePlan {
    #[cfg(test)]
    pub fn ticket(&self) -> &RemoteTicket {
        &self.ticket
    }

    pub fn with_bearer_credential<R>(&self, sink: impl FnOnce(&str) -> R) -> R {
        let credential = Zeroizing::new(format!("{}:{}", self.ticket.id, self.issued_bearer.expose_to_sink()));
        debug_assert!(credential.starts_with(&self.ticket.id));
        debug_assert!(credential.len() > self.ticket.id.len());
        sink(&credential)
    }

    #[cfg(test)]
    fn issued_bearer_for_test(&self) -> &str {
        self.issued_bearer.expose_to_sink()
    }
}

impl fmt::Debug for TicketIssuePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TicketIssuePlan")
            .field("ticket", &self.ticket)
            .field("issued_bearer", &self.issued_bearer)
            .field("next_ticket_sequence", &self.next_ticket_sequence)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketPolicyFacts<'a> {
    pub ticket_id: &'a str,
    pub client_endpoint: Option<&'a str>,
    pub now_unix_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketAuthorization {
    Authorized,
    Rejected(&'static str),
}

pub fn plan_ticket_issue(
    state: &RemoteTicketState,
    input: TicketIssueInput,
    entropy: &[u8],
    verifier_key: &Arc<TicketVerifierKey>,
) -> Result<TicketIssuePlan, String> {
    state.validate()?;
    let admitted = admit_ticket_issue_input(input)?;
    validate_ticket_entropy(entropy)?;
    let next_ticket_sequence = state
        .next_ticket_sequence
        .checked_add(1)
        .ok_or_else(|| "remote-ticket-sequence-overflow".to_string())?;
    let issued_bearer = IssuedBearerToken::from_encoded(URL_SAFE_NO_PAD.encode(entropy));
    let presented_bearer = PresentedBearerToken::parse(issued_bearer.expose_to_sink())?;
    let verifier = keyed_ticket_verifier(verifier_key, presented_bearer.bytes());
    if state.tickets.values().any(|ticket| ticket.verifier.constant_time_eq(&verifier)) {
        return Err("remote-ticket-entropy-reused".to_string());
    }
    let id = ticket_identity(&admitted, state.next_ticket_sequence)?;
    let ticket_id = TicketId::parse_generated(&id)?;
    if state.tickets.contains_key(ticket_id.as_str())
        || state.invalidated_legacy_ticket_ids.contains(ticket_id.as_str())
    {
        return Err("remote-ticket-identity-collision".to_string());
    }
    debug_assert_eq!(ticket_id.as_str(), id);
    debug_assert_eq!(Some(next_ticket_sequence), state.next_ticket_sequence.checked_add(1));
    let ticket = RemoteTicket {
        id,
        display_name: admitted.display_name,
        verifier_key_id: verifier_key.id().to_string(),
        verifier,
        created_unix_s: admitted.validity.created_unix_s(),
        expires_unix_s: admitted.validity.expires_unix_s(),
        uses_remaining: admitted.use_limit.count(),
        max_build_time_secs: admitted.build_time_limit.seconds(),
        max_upload_bytes: admitted.upload_byte_limit.bytes(),
        bound_client_endpoint: admitted.bound_client_endpoint,
        revoked: false,
        runtime_verifier_key: Some(Arc::clone(verifier_key)),
    };
    Ok(TicketIssuePlan {
        ticket,
        issued_bearer,
        next_ticket_sequence,
    })
}

pub fn apply_ticket_issue(state: &mut RemoteTicketState, plan: &TicketIssuePlan) -> Result<RemoteTicketView, String> {
    state.validate()?;
    if state.next_ticket_sequence.checked_add(1) != Some(plan.next_ticket_sequence) {
        return Err("remote-ticket-issue-plan-stale".to_string());
    }
    if state.tickets.contains_key(&plan.ticket.id) {
        return Err("remote-ticket-identity-collision".to_string());
    }
    state.next_ticket_sequence = plan.next_ticket_sequence;
    state.tickets.insert(plan.ticket.id.clone(), plan.ticket.clone());
    state.validate()?;
    Ok(redacted_ticket_view(&plan.ticket))
}

pub fn validate_presented_ticket_token(token: &str) -> Result<(), String> {
    PresentedBearerToken::parse(token).map(drop)
}

pub fn admit_ticket_policy(ticket: &RemoteTicket) -> Result<AdmittedTicketPolicy<'_>, String> {
    admit_ticket_record(&ticket.id, ticket, TicketIdAdmission::CompatibleProtocol)
}

fn admit_ticket_credential<'a>(
    ticket: &'a RemoteTicket,
    presented_token: &str,
    facts: &TicketPolicyFacts<'_>,
) -> Result<AdmittedTicketCredential<'a>, String> {
    let policy = admit_ticket_policy(ticket)?;
    let ticket_id = TicketId::parse_compatible(facts.ticket_id)?;
    let bearer = PresentedBearerToken::parse(presented_token)?;
    validate_bound_client_endpoint(facts.client_endpoint)?;
    let presentation = AdmittedTicketPresentation {
        ticket_id,
        bearer,
        client_endpoint: facts.client_endpoint.map(str::to_string),
        now_unix_s: facts.now_unix_s,
    };
    debug_assert!(!presentation.ticket_id.as_str().is_empty());
    debug_assert_eq!(presentation.bearer.bytes().len(), TICKET_ENTROPY_BYTES);
    Ok(AdmittedTicketCredential { policy, presentation })
}

pub fn authorize_ticket(
    ticket: &RemoteTicket,
    presented_token: &str,
    facts: &TicketPolicyFacts<'_>,
) -> TicketAuthorization {
    let Ok(admitted) = admit_ticket_credential(ticket, presented_token, facts) else {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    };
    let Some(key) = ticket.active_verifier_key() else {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    };
    if key.key_id() != &admitted.policy.verifier_key_id {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    }
    let presented_verifier = keyed_ticket_verifier(key, admitted.presentation.bearer.bytes());
    if !admitted.policy.verifier.constant_time_eq(&presented_verifier) {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    }
    apply_ticket_policy(&admitted)
}

pub fn redacted_ticket_view(ticket: &RemoteTicket) -> RemoteTicketView {
    RemoteTicketView {
        id: ticket.id.clone(),
        display_name: ticket.display_name.clone(),
        verifier_key_id: ticket.verifier_key_id.clone(),
        created_unix_s: ticket.created_unix_s,
        expires_unix_s: ticket.expires_unix_s,
        uses_remaining: ticket.uses_remaining,
        max_build_time_secs: ticket.max_build_time_secs,
        max_upload_bytes: ticket.max_upload_bytes,
        bound_client_endpoint: ticket.bound_client_endpoint.clone(),
        revoked: ticket.revoked,
    }
}

pub fn invalidate_tickets_for_key(state: &mut RemoteTicketState, retiring_key_id: &str) -> Result<Vec<String>, String> {
    state.validate()?;
    validate_key_id(retiring_key_id)?;
    let mut invalidated_ids = Vec::with_capacity(state.tickets.len());
    for ticket in state.tickets.values_mut() {
        if ticket.verifier_key_id == retiring_key_id && !ticket.revoked {
            ticket.revoked = true;
            ticket.clear_active_verifier_key();
            invalidated_ids.push(ticket.id.clone());
        }
    }
    debug_assert!(invalidated_ids.len() <= state.tickets.len());
    debug_assert!(invalidated_ids.len() <= MAX_REMOTE_TICKETS);
    Ok(invalidated_ids)
}

fn apply_ticket_policy(admitted: &AdmittedTicketCredential<'_>) -> TicketAuthorization {
    let policy = &admitted.policy;
    let presentation = &admitted.presentation;
    debug_assert!(policy.validity.expires_unix_s() > policy.validity.created_unix_s());
    debug_assert!(policy.uses_remaining.count() <= TICKET_USES_MAX);
    if presentation.ticket_id != policy.ticket_id {
        return TicketAuthorization::Rejected("ticket-id-mismatch");
    }
    if presentation.now_unix_s < policy.validity.created_unix_s() {
        return TicketAuthorization::Rejected("ticket-clock-before-issuance");
    }
    if policy.revoked {
        return TicketAuthorization::Rejected("ticket-revoked");
    }
    if presentation.now_unix_s >= policy.validity.expires_unix_s() {
        return TicketAuthorization::Rejected("ticket-expired");
    }
    if policy.uses_remaining.count() == 0 {
        return TicketAuthorization::Rejected("ticket-exhausted");
    }
    if let Some(bound) = policy.bound_client_endpoint.as_deref()
        && presentation.client_endpoint.as_deref() != Some(bound)
    {
        return TicketAuthorization::Rejected("ticket-client-endpoint-mismatch");
    }
    TicketAuthorization::Authorized
}

fn admit_ticket_issue_input(input: TicketIssueInput) -> Result<AdmittedTicketIssueInput, String> {
    if input.display_name.is_empty() || input.display_name.len() > MAX_TICKET_DISPLAY_NAME_BYTES {
        return Err("remote-ticket-display-name-invalid".to_string());
    }
    let ttl = TicketTtl::new(input.ttl_secs)?;
    let validity = TicketValidityWindow::from_ttl(input.now_unix_s, ttl)?;
    let build_time_limit_secs = BuildTimeLimit::new(input.max_build_time_secs)?;
    let upload_byte_limit_bytes = UploadByteLimit::new(input.max_upload_bytes)?;
    validate_bound_client_endpoint(input.bound_client_endpoint.as_deref())?;
    let admitted = AdmittedTicketIssueInput {
        display_name: input.display_name,
        ttl,
        validity,
        use_limit: TicketUseLimit::new(input.uses)?,
        build_time_limit: build_time_limit_secs,
        upload_byte_limit: upload_byte_limit_bytes,
        bound_client_endpoint: input.bound_client_endpoint,
    };
    debug_assert!(admitted.validity.expires_unix_s() > admitted.validity.created_unix_s());
    debug_assert!(admitted.use_limit.count() > 0);
    Ok(admitted)
}

fn validate_ticket_entropy(entropy: &[u8]) -> Result<(), String> {
    if entropy.len() != TICKET_ENTROPY_BYTES {
        return Err("remote-ticket-entropy-length-invalid".to_string());
    }
    if entropy.iter().all(|byte| *byte == ALL_ZERO_ENTROPY_BYTE) {
        return Err("remote-ticket-entropy-all-zero".to_string());
    }
    Ok(())
}

fn decode_ticket_token(token: &str) -> Result<Zeroizing<[u8; TICKET_ENTROPY_BYTES]>, String> {
    if token.len() != TICKET_TOKEN_BASE64URL_BYTES {
        return Err("remote-ticket-token-encoding-invalid".to_string());
    }
    decode_canonical_base64url::<TICKET_ENTROPY_BYTES>(CanonicalBase64DecodeInput {
        encoded: token,
        error: "remote-ticket-token-encoding-invalid",
    })
}

struct CanonicalBase64DecodeInput<'a> {
    encoded: &'a str,
    error: &'static str,
}

fn decode_canonical_base64url<const OUTPUT_BYTES: usize>(
    input: CanonicalBase64DecodeInput<'_>,
) -> Result<Zeroizing<[u8; OUTPUT_BYTES]>, String> {
    let encoded = input.encoded;
    let error = input.error;
    let decoded = Zeroizing::new(URL_SAFE_NO_PAD.decode(encoded).map_err(|_| error.to_string())?);
    if decoded.len() != OUTPUT_BYTES {
        return Err(error.to_string());
    }
    let mut bytes = [0_u8; OUTPUT_BYTES];
    bytes.copy_from_slice(&decoded);
    let canonical = Zeroizing::new(URL_SAFE_NO_PAD.encode(bytes.as_slice()));
    if canonical.as_str() != encoded {
        bytes.zeroize();
        return Err(error.to_string());
    }
    Ok(Zeroizing::new(bytes))
}

fn keyed_ticket_verifier(key: &TicketVerifierKey, token_bytes: &[u8; TICKET_ENTROPY_BYTES]) -> TicketVerifier {
    let mut hasher = blake3::Hasher::new_keyed(key.bytes());
    hasher.update(TICKET_VERIFIER_DOMAIN);
    hasher.update(token_bytes);
    TicketVerifier(*hasher.finalize().as_bytes())
}

fn ticket_identity(input: &AdmittedTicketIssueInput, sequence: u64) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TICKET_IDENTITY_DOMAIN);
    hash_labeled_bytes(&mut hasher, b"display-name", input.display_name.as_bytes())?;
    hash_labeled_bytes(&mut hasher, b"created-unix-s", &input.validity.created_unix_s().to_be_bytes())?;
    hash_labeled_bytes(&mut hasher, b"ttl-secs", &input.ttl.seconds().to_be_bytes())?;
    hash_labeled_bytes(&mut hasher, b"uses", &input.use_limit.count().to_be_bytes())?;
    hash_labeled_bytes(&mut hasher, b"sequence", &sequence.to_be_bytes())?;
    let id = hasher.finalize().to_hex()[..TICKET_ID_HEX_CHARS].to_string();
    debug_assert_eq!(id.len(), TICKET_ID_HEX_CHARS);
    debug_assert!(id.bytes().all(|byte| !byte.is_ascii_uppercase()));
    Ok(id)
}

fn hash_labeled_bytes(hasher: &mut blake3::Hasher, label: &[u8], value: &[u8]) -> Result<(), String> {
    let label_len =
        u64::try_from(label.len()).map_err(|_| "remote-ticket-identity-label-length-invalid".to_string())?;
    let value_len =
        u64::try_from(value.len()).map_err(|_| "remote-ticket-identity-value-length-invalid".to_string())?;
    hasher.update(&label_len.to_be_bytes()[..HASH_LENGTH_PREFIX_BYTES]);
    hasher.update(label);
    hasher.update(&value_len.to_be_bytes()[..HASH_LENGTH_PREFIX_BYTES]);
    hasher.update(value);
    debug_assert_eq!(usize::try_from(label_len), Ok(label.len()));
    debug_assert_eq!(usize::try_from(value_len), Ok(value.len()));
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TicketIdAdmission {
    GeneratedState,
    CompatibleProtocol,
}

fn admit_ticket_record<'a>(
    map_id: &str,
    ticket: &'a RemoteTicket,
    id_admission: TicketIdAdmission,
) -> Result<AdmittedTicketPolicy<'a>, String> {
    let ticket_id = match id_admission {
        TicketIdAdmission::GeneratedState => TicketId::parse_generated(map_id)?,
        TicketIdAdmission::CompatibleProtocol => TicketId::parse_compatible(map_id)?,
    };
    if map_id != ticket.id {
        return Err("remote-ticket-state-map-id-mismatch".to_string());
    }
    if ticket.display_name.is_empty() || ticket.display_name.len() > MAX_TICKET_DISPLAY_NAME_BYTES {
        return Err("remote-ticket-state-display-name-invalid".to_string());
    }
    let verifier_key_id = TicketVerifierKeyId::parse(&ticket.verifier_key_id)?;
    let ttl_span = ticket
        .expires_unix_s
        .checked_sub(ticket.created_unix_s)
        .ok_or_else(|| "remote-ticket-state-expiry-invalid".to_string())?;
    if ttl_span == 0 || ttl_span > TICKET_TTL_SECS_MAX {
        return Err("remote-ticket-state-ttl-limit-exceeded".to_string());
    }
    let validity = TicketValidityWindow {
        created_unix_s: ticket.created_unix_s,
        expires_unix_s: ticket.expires_unix_s,
    };
    let uses_remaining = TicketUsesRemaining::new(ticket.uses_remaining)
        .map_err(|_| "remote-ticket-state-use-limit-exceeded".to_string())?;
    let build_time_limit_secs = BuildTimeLimit::new(ticket.max_build_time_secs)
        .map_err(|_| "remote-ticket-state-build-time-limit-invalid".to_string())?;
    let upload_byte_limit_bytes = UploadByteLimit::new(ticket.max_upload_bytes)
        .map_err(|_| "remote-ticket-state-upload-limit-exceeded".to_string())?;
    validate_bound_client_endpoint(ticket.bound_client_endpoint.as_deref())?;
    debug_assert_eq!(ticket_id.as_str(), ticket.id);
    debug_assert!(validity.expires_unix_s() > validity.created_unix_s());
    Ok(AdmittedTicketPolicy {
        ticket_id,
        verifier_key_id,
        verifier: &ticket.verifier,
        validity,
        uses_remaining,
        build_time_limit: build_time_limit_secs,
        upload_byte_limit: upload_byte_limit_bytes,
        bound_client_endpoint: ticket.bound_client_endpoint.clone(),
        revoked: ticket.revoked,
    })
}

fn validate_generated_ticket_id(id: &str) -> Result<(), String> {
    if id.len() != TICKET_ID_HEX_CHARS || !id.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("remote-ticket-generated-id-invalid".to_string());
    }
    Ok(())
}

fn validate_legacy_ticket_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > TICKET_ID_HEX_CHARS {
        return Err("remote-ticket-id-invalid".to_string());
    }
    if !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')) {
        return Err("remote-ticket-id-invalid".to_string());
    }
    Ok(())
}

fn validate_bound_client_endpoint(endpoint: Option<&str>) -> Result<(), String> {
    let Some(endpoint) = endpoint else {
        return Ok(());
    };
    if endpoint.is_empty() || endpoint.len() > TICKET_BOUND_ENDPOINT_BYTES_MAX {
        return Err("remote-ticket-bound-endpoint-invalid".to_string());
    }
    Ok(())
}

fn validate_key_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > MAX_TICKET_KEY_ID_BYTES {
        return Err("remote-ticket-verifier-key-id-invalid".to_string());
    }
    if !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')) {
        return Err("remote-ticket-verifier-key-id-invalid".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY_ID: &str = "ticket-key-1";
    const TEST_KEY_BYTE: u8 = 0x41;
    const OTHER_KEY_BYTE: u8 = 0x52;
    const TEST_ENTROPY_BYTE: u8 = 0x23;
    const OTHER_ENTROPY_BYTE: u8 = 0x34;
    const TEST_NOW_UNIX_S: u64 = 100;
    const TEST_TTL_SECS: u64 = 300;
    const TEST_MAX_BUILD_TIME_SECS: u64 = 60;
    const TEST_MAX_UPLOAD_BYTES: u64 = 1_024;
    const TEST_USES: u32 = 2;
    const CLOCK_ROLLBACK_NOW_UNIX_S: u64 = TEST_NOW_UNIX_S - 1;

    fn key(id: &str, byte: u8) -> Arc<TicketVerifierKey> {
        let encoded = URL_SAFE_NO_PAD.encode([byte; TICKET_VERIFIER_KEY_BYTES]);
        Arc::new(TicketVerifierKey::parse(&format!("{id}:{encoded}")).unwrap())
    }

    fn input() -> TicketIssueInput {
        TicketIssueInput {
            display_name: "test ticket".to_string(),
            now_unix_s: TEST_NOW_UNIX_S,
            ttl_secs: TEST_TTL_SECS,
            uses: TEST_USES,
            max_build_time_secs: TEST_MAX_BUILD_TIME_SECS,
            max_upload_bytes: TEST_MAX_UPLOAD_BYTES,
            bound_client_endpoint: Some("client-a".to_string()),
        }
    }

    fn issued() -> (RemoteTicketState, TicketIssuePlan, Arc<TicketVerifierKey>) {
        let state = RemoteTicketState::default();
        let key = key(TEST_KEY_ID, TEST_KEY_BYTE);
        let plan = plan_ticket_issue(&state, input(), &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES], &key).unwrap();
        (state, plan, key)
    }

    #[test]
    fn valid_entropy_issues_and_verifies_ticket() {
        let (mut state, plan, _key) = issued();
        let token = plan.issued_bearer_for_test().to_string();
        let ticket_id = plan.ticket().id.clone();
        let view = apply_ticket_issue(&mut state, &plan).unwrap();
        let decision = authorize_ticket(state.tickets.get(&ticket_id).unwrap(), &token, &TicketPolicyFacts {
            ticket_id: &ticket_id,
            client_endpoint: Some("client-a"),
            now_unix_s: TEST_NOW_UNIX_S,
        });

        assert_eq!(decision, TicketAuthorization::Authorized);
        assert_eq!(view.verifier_key_id, TEST_KEY_ID);
        assert_eq!(state.schema_version, REMOTE_TICKET_STATE_SCHEMA_VERSION);
        assert_eq!(state.next_ticket_sequence, 1);
    }

    #[test]
    fn persisted_state_rejects_policy_limit_and_identity_violations() {
        let (mut baseline, plan, _) = issued();
        apply_ticket_issue(&mut baseline, &plan).unwrap();
        assert!(baseline.validate().is_ok());
        let id = baseline.tickets.keys().next().unwrap().clone();

        let mut excessive_ttl = baseline.clone();
        excessive_ttl.tickets.get_mut(&id).unwrap().expires_unix_s =
            TEST_NOW_UNIX_S.checked_add(TICKET_TTL_SECS_MAX).and_then(|value| value.checked_add(1)).unwrap();
        assert_eq!(excessive_ttl.validate().unwrap_err(), "remote-ticket-state-ttl-limit-exceeded");

        let mut excessive_uses = baseline.clone();
        excessive_uses.tickets.get_mut(&id).unwrap().uses_remaining = TICKET_USES_MAX.checked_add(1).unwrap();
        assert_eq!(excessive_uses.validate().unwrap_err(), "remote-ticket-state-use-limit-exceeded");

        let mut excessive_build = baseline.clone();
        excessive_build.tickets.get_mut(&id).unwrap().max_build_time_secs =
            TICKET_BUILD_TIME_SECS_MAX.checked_add(1).unwrap();
        assert_eq!(excessive_build.validate().unwrap_err(), "remote-ticket-state-build-time-limit-invalid");

        let mut excessive_upload = baseline.clone();
        excessive_upload.tickets.get_mut(&id).unwrap().max_upload_bytes =
            TICKET_UPLOAD_BYTES_MAX.checked_add(1).unwrap();
        assert_eq!(excessive_upload.validate().unwrap_err(), "remote-ticket-state-upload-limit-exceeded");

        for endpoint in [
            String::new(),
            "x".repeat(TICKET_BOUND_ENDPOINT_BYTES_MAX.checked_add(1).unwrap()),
        ] {
            let mut invalid_endpoint = baseline.clone();
            invalid_endpoint.tickets.get_mut(&id).unwrap().bound_client_endpoint = Some(endpoint);
            assert_eq!(invalid_endpoint.validate().unwrap_err(), "remote-ticket-bound-endpoint-invalid");
        }

        let mut malformed_id = baseline.clone();
        let mut ticket = malformed_id.tickets.remove(&id).unwrap();
        ticket.id = "ticket-legacy-shape".to_string();
        malformed_id.tickets.insert(ticket.id.clone(), ticket);
        assert_eq!(malformed_id.validate().unwrap_err(), "remote-ticket-generated-id-invalid");

        let mut overlapping = baseline;
        overlapping.invalidated_legacy_ticket_ids.insert(id);
        assert_eq!(overlapping.validate().unwrap_err(), "remote-ticket-state-active-invalidated-id-overlap");
    }

    #[test]
    fn reviewed_ttl_and_use_boundaries_are_enforced() {
        let key = key(TEST_KEY_ID, TEST_KEY_BYTE);
        let state = RemoteTicketState::default();
        let mut boundary = input();
        boundary.ttl_secs = TICKET_TTL_SECS_MAX;
        boundary.uses = TICKET_USES_MAX;
        assert!(plan_ticket_issue(&state, boundary, &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES], &key,).is_ok());

        let mut excessive_ttl = input();
        excessive_ttl.ttl_secs = TICKET_TTL_SECS_MAX.checked_add(1).unwrap();
        assert_eq!(
            plan_ticket_issue(&state, excessive_ttl, &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES], &key,).unwrap_err(),
            "remote-ticket-ttl-limit-exceeded"
        );
        let mut excessive_uses = input();
        excessive_uses.uses = TICKET_USES_MAX.checked_add(1).unwrap();
        assert_eq!(
            plan_ticket_issue(&state, excessive_uses, &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES], &key,).unwrap_err(),
            "remote-ticket-use-limit-exceeded"
        );
    }

    #[test]
    fn invalid_entropy_and_repeated_entropy_fail_closed() {
        let key = key(TEST_KEY_ID, TEST_KEY_BYTE);
        let state = RemoteTicketState::default();
        let short = [TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES - 1];
        assert_eq!(
            plan_ticket_issue(&state, input(), &short, &key).unwrap_err(),
            "remote-ticket-entropy-length-invalid"
        );
        assert_eq!(
            plan_ticket_issue(&state, input(), &[0_u8; TICKET_ENTROPY_BYTES], &key).unwrap_err(),
            "remote-ticket-entropy-all-zero"
        );

        let first = plan_ticket_issue(&state, input(), &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES], &key).unwrap();
        let mut committed = state;
        apply_ticket_issue(&mut committed, &first).unwrap();
        assert_eq!(
            plan_ticket_issue(&committed, input(), &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES], &key).unwrap_err(),
            "remote-ticket-entropy-reused"
        );
    }

    #[test]
    fn malformed_tokens_and_wrong_keys_fail_authentication() {
        let (mut state, plan, _key) = issued();
        let token = plan.issued_bearer_for_test().to_string();
        let id = plan.ticket().id.clone();
        apply_ticket_issue(&mut state, &plan).unwrap();
        let facts = TicketPolicyFacts {
            ticket_id: &id,
            client_endpoint: Some("client-a"),
            now_unix_s: TEST_NOW_UNIX_S,
        };
        let ticket = state.tickets.get_mut(&id).unwrap();

        for malformed in ["", "not-base64", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="] {
            assert_eq!(
                authorize_ticket(ticket, malformed, &facts),
                TicketAuthorization::Rejected("ticket-authentication-failed")
            );
        }
        let wrong_token = URL_SAFE_NO_PAD.encode([OTHER_ENTROPY_BYTE; TICKET_ENTROPY_BYTES]);
        assert_eq!(
            authorize_ticket(ticket, &wrong_token, &facts),
            TicketAuthorization::Rejected("ticket-authentication-failed")
        );
        let wrong_key = key("ticket-key-2", OTHER_KEY_BYTE);
        ticket.bind_active_verifier_key(&wrong_key);
        assert_eq!(
            authorize_ticket(ticket, &token, &facts),
            TicketAuthorization::Rejected("ticket-authentication-failed")
        );
    }

    #[test]
    fn policy_checks_follow_successful_verifier_comparison() {
        let (mut state, plan, key) = issued();
        let token = plan.issued_bearer_for_test().to_string();
        let id = plan.ticket().id.clone();
        apply_ticket_issue(&mut state, &plan).unwrap();
        let ticket = state.tickets.get_mut(&id).unwrap();
        ticket.bind_active_verifier_key(&key);

        assert_eq!(
            authorize_ticket(ticket, &token, &TicketPolicyFacts {
                ticket_id: &id,
                client_endpoint: Some("client-a"),
                now_unix_s: CLOCK_ROLLBACK_NOW_UNIX_S,
            },),
            TicketAuthorization::Rejected("ticket-clock-before-issuance")
        );

        ticket.revoked = true;
        assert_eq!(
            authorize_ticket(ticket, &token, &TicketPolicyFacts {
                ticket_id: &id,
                client_endpoint: Some("client-a"),
                now_unix_s: TEST_NOW_UNIX_S,
            },),
            TicketAuthorization::Rejected("ticket-revoked")
        );
        ticket.revoked = false;
        assert_eq!(
            authorize_ticket(ticket, &token, &TicketPolicyFacts {
                ticket_id: &id,
                client_endpoint: Some("client-a"),
                now_unix_s: TEST_NOW_UNIX_S + TEST_TTL_SECS,
            },),
            TicketAuthorization::Rejected("ticket-expired")
        );
        ticket.uses_remaining = 0;
        assert_eq!(
            authorize_ticket(ticket, &token, &TicketPolicyFacts {
                ticket_id: &id,
                client_endpoint: Some("client-a"),
                now_unix_s: TEST_NOW_UNIX_S,
            },),
            TicketAuthorization::Rejected("ticket-exhausted")
        );
        ticket.uses_remaining = TEST_USES;
        assert_eq!(
            authorize_ticket(ticket, &token, &TicketPolicyFacts {
                ticket_id: &id,
                client_endpoint: Some("client-b"),
                now_unix_s: TEST_NOW_UNIX_S,
            },),
            TicketAuthorization::Rejected("ticket-client-endpoint-mismatch")
        );
    }

    #[test]
    fn state_serialization_contains_verifier_but_not_bearer_material() {
        let (mut state, plan, _key) = issued();
        let credential = plan.with_bearer_credential(str::to_string);
        let token = plan.issued_bearer_for_test().to_string();
        apply_ticket_issue(&mut state, &plan).unwrap();
        let rendered = serde_json::to_string(&state).unwrap();

        assert!(rendered.contains("verifier_key_id"));
        assert!(rendered.contains("verifier"));
        assert!(!rendered.contains(&credential));
        assert!(!rendered.contains(&token));
        assert!(!rendered.contains("\"secret\""));
    }

    #[test]
    fn legacy_public_seed_prediction_does_not_match_new_token() {
        let legacy_seed = format!("{}:{}:{}:{}:{}", input().display_name, TEST_NOW_UNIX_S, TEST_TTL_SECS, TEST_USES, 0);
        let predicted_legacy_secret = blake3::hash(legacy_seed.as_bytes()).to_hex().to_string();
        let (_state, plan, _key) = issued();

        assert_eq!(predicted_legacy_secret.len(), 64);
        assert_ne!(plan.issued_bearer_for_test(), predicted_legacy_secret);
        assert_eq!(plan.issued_bearer_for_test().len(), TICKET_TOKEN_BASE64URL_BYTES);
    }

    #[test]
    fn verifier_key_parser_rejects_malformed_and_zero_keys() {
        assert_eq!(
            TicketVerifierKey::parse("missing-separator").unwrap_err(),
            "remote-ticket-verifier-key-format-invalid"
        );
        assert_eq!(TicketVerifierKey::parse("bad key:AAAA").unwrap_err(), "remote-ticket-verifier-key-id-invalid");
        let zero = URL_SAFE_NO_PAD.encode([0_u8; TICKET_VERIFIER_KEY_BYTES]);
        assert_eq!(
            TicketVerifierKey::parse(&format!("ticket-key-1:{zero}")).unwrap_err(),
            "remote-ticket-verifier-key-all-zero"
        );
    }

    #[test]
    fn verifier_serialization_is_strict_and_debug_is_redacted() {
        let (_state, plan, _key) = issued();
        let encoded = plan.ticket().verifier.encoded();
        let decoded: TicketVerifier = serde_json::from_str(&format!("\"{encoded}\"")).unwrap();

        assert_eq!(decoded, plan.ticket().verifier);
        assert_eq!(format!("{:?}", plan.ticket().verifier), "TicketVerifier(<redacted>)");
        assert!(serde_json::from_str::<TicketVerifier>("\"AAAA\"").is_err());
    }

    #[test]
    fn nominal_roles_accept_valid_boundaries() {
        let ticket_id = TicketId::parse_compatible("ticket-1").unwrap();
        let key_id = TicketVerifierKeyId::parse(TEST_KEY_ID).unwrap();
        let ttl = TicketTtl::new(TICKET_TTL_SECS_MAX).unwrap();
        let validity = TicketValidityWindow::from_ttl(TEST_NOW_UNIX_S, ttl).unwrap();
        let use_limit = TicketUseLimit::new(TICKET_USES_MAX).unwrap();
        let uses_remaining = TicketUsesRemaining::new(0).unwrap();
        let redeemed = TicketUsesRemaining::new(1).unwrap().redeem().unwrap();
        let build_time = BuildTimeLimit::new(TICKET_BUILD_TIME_SECS_MAX).unwrap();
        let upload_bytes = UploadByteLimit::new(TICKET_UPLOAD_BYTES_MAX).unwrap();

        assert_eq!(ticket_id.as_str(), "ticket-1");
        assert_eq!(key_id.as_str(), TEST_KEY_ID);
        assert_eq!(validity.expires_unix_s(), TEST_NOW_UNIX_S.checked_add(TICKET_TTL_SECS_MAX).unwrap());
        assert_eq!(use_limit.count(), TICKET_USES_MAX);
        assert_eq!(uses_remaining.count(), 0);
        assert_eq!(redeemed.count(), 0);
        assert_eq!(build_time.seconds(), TICKET_BUILD_TIME_SECS_MAX);
        assert_eq!(upload_bytes.bytes(), TICKET_UPLOAD_BYTES_MAX);
    }

    #[test]
    fn nominal_roles_reject_invalid_boundaries_and_overflow() {
        let oversized_id = "x".repeat(TICKET_ID_HEX_CHARS.checked_add(1).unwrap());
        assert!(TicketId::parse_compatible("").is_err());
        assert!(TicketId::parse_compatible("bad id").is_err());
        assert!(TicketId::parse_compatible(&oversized_id).is_err());
        assert!(TicketVerifierKeyId::parse("").is_err());
        assert!(TicketTtl::new(0).is_err());
        assert!(TicketTtl::new(TICKET_TTL_SECS_MAX.checked_add(1).unwrap()).is_err());
        assert!(TicketValidityWindow::from_ttl(u64::MAX, TicketTtl::new(1).unwrap()).is_err());
        assert!(TicketValidityWindow::from_bounds(TEST_NOW_UNIX_S, TEST_NOW_UNIX_S).is_err());
        assert!(TicketValidityWindow::from_bounds(TEST_NOW_UNIX_S, CLOCK_ROLLBACK_NOW_UNIX_S).is_err());
        assert!(TicketUseLimit::new(0).is_err());
        assert!(TicketUseLimit::new(TICKET_USES_MAX.checked_add(1).unwrap()).is_err());
        assert!(TicketUsesRemaining::new(TICKET_USES_MAX.checked_add(1).unwrap()).is_err());
        assert!(TicketUsesRemaining::new(0).unwrap().redeem().is_err());
        assert!(BuildTimeLimit::new(0).is_err());
        assert!(BuildTimeLimit::new(TICKET_BUILD_TIME_SECS_MAX.checked_add(1).unwrap()).is_err());
        assert!(UploadByteLimit::new(TICKET_UPLOAD_BYTES_MAX.checked_add(1).unwrap()).is_err());
        assert!(PresentedBearerToken::parse("not-a-token").is_err());
    }

    #[test]
    fn secret_nominal_types_format_only_as_redacted() {
        let encoded = URL_SAFE_NO_PAD.encode([TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES]);
        let issued = IssuedBearerToken::from_encoded(encoded.clone());
        let presented = PresentedBearerToken::parse(&encoded).unwrap();
        let issued_debug = format!("{issued:?}");
        let presented_debug = format!("{presented:?}");

        assert_eq!(issued_debug, "IssuedBearerToken(<redacted>)");
        assert_eq!(presented_debug, "PresentedBearerToken(<redacted>)");
        assert!(!issued_debug.contains(&encoded));
        assert!(!presented_debug.contains(&encoded));
    }

    #[test]
    fn malformed_structural_state_cannot_reach_authentication_policy() {
        let (mut state, plan, key) = issued();
        let token = plan.issued_bearer_for_test().to_string();
        let id = plan.ticket().id.clone();
        apply_ticket_issue(&mut state, &plan).unwrap();
        let ticket = state.tickets.get_mut(&id).unwrap();
        ticket.bind_active_verifier_key(&key);
        ticket.expires_unix_s = ticket.created_unix_s;
        let facts = TicketPolicyFacts {
            ticket_id: &id,
            client_endpoint: Some("client-a"),
            now_unix_s: TEST_NOW_UNIX_S,
        };

        assert_eq!(admit_ticket_policy(ticket).unwrap_err(), "remote-ticket-state-ttl-limit-exceeded");
        assert_eq!(
            authorize_ticket(ticket, &token, &facts),
            TicketAuthorization::Rejected("ticket-authentication-failed")
        );
    }
}
