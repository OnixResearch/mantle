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

#[derive(Clone, PartialEq, Eq)]
pub struct TicketVerifierKey {
    id: String,
    bytes: [u8; TICKET_VERIFIER_KEY_BYTES],
}

impl TicketVerifierKey {
    pub fn parse(value: &str) -> Result<Self, String> {
        let (id, encoded) = value
            .split_once(':')
            .ok_or_else(|| "remote-ticket-verifier-key-format-invalid".to_string())?;
        validate_key_id(id)?;
        let bytes = decode_canonical_base64url::<TICKET_VERIFIER_KEY_BYTES>(
            encoded,
            "remote-ticket-verifier-key-encoding-invalid",
        )?;
        if bytes.iter().all(|byte| *byte == ALL_ZERO_ENTROPY_BYTE) {
            return Err("remote-ticket-verifier-key-all-zero".to_string());
        }
        Ok(Self {
            id: id.to_string(),
            bytes,
        })
    }

    pub fn id(&self) -> &str {
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
            .field("id", &self.id)
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

impl fmt::Debug for TicketVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TicketVerifier(<redacted>)")
    }
}

impl Serialize for TicketVerifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&URL_SAFE_NO_PAD.encode(self.0))
    }
}

impl<'de> Deserialize<'de> for TicketVerifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        decode_canonical_base64url::<TICKET_VERIFIER_BYTES>(
            &encoded,
            "remote-ticket-verifier-encoding-invalid",
        )
        .map(Self)
        .map_err(serde::de::Error::custom)
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
    #[serde(default)]
    pub invalidated_legacy_ticket_ids: BTreeSet<String>,
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
            validate_ticket_record(map_id, ticket)?;
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

pub struct TicketBearerCredential {
    value: String,
}

impl TicketBearerCredential {
    pub fn as_str(&self) -> &str {
        &self.value
    }

    #[cfg(test)]
    fn token(&self) -> &str {
        self.value
            .split_once(':')
            .expect("issued credential always has an id separator")
            .1
    }
}

impl fmt::Debug for TicketBearerCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TicketBearerCredential(<redacted>)")
    }
}

impl Drop for TicketBearerCredential {
    fn drop(&mut self) {
        self.value.zeroize();
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

pub struct TicketIssuePlan {
    ticket: RemoteTicket,
    bearer_credential: TicketBearerCredential,
    next_ticket_sequence: u64,
}

impl TicketIssuePlan {
    #[cfg(test)]
    pub fn ticket(&self) -> &RemoteTicket {
        &self.ticket
    }

    pub fn bearer_credential(&self) -> &TicketBearerCredential {
        &self.bearer_credential
    }
}

impl fmt::Debug for TicketIssuePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TicketIssuePlan")
            .field("ticket", &self.ticket)
            .field("bearer_credential", &self.bearer_credential)
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
    validate_ticket_issue_input(&input)?;
    validate_ticket_entropy(entropy)?;
    let expires_unix_s = input
        .now_unix_s
        .checked_add(input.ttl_secs)
        .ok_or_else(|| "remote-ticket-expiry-overflow".to_string())?;
    let next_ticket_sequence = state
        .next_ticket_sequence
        .checked_add(1)
        .ok_or_else(|| "remote-ticket-sequence-overflow".to_string())?;
    let token = Zeroizing::new(URL_SAFE_NO_PAD.encode(entropy));
    let token_bytes = Zeroizing::new(decode_ticket_token(token.as_str())?);
    let verifier = keyed_ticket_verifier(verifier_key, &token_bytes);
    if state
        .tickets
        .values()
        .any(|ticket| ticket.verifier.constant_time_eq(&verifier))
    {
        return Err("remote-ticket-entropy-reused".to_string());
    }
    let id = ticket_identity(&input, state.next_ticket_sequence);
    if state.tickets.contains_key(&id) || state.invalidated_legacy_ticket_ids.contains(&id) {
        return Err("remote-ticket-identity-collision".to_string());
    }
    let bearer_credential = TicketBearerCredential {
        value: format!("{id}:{}", token.as_str()),
    };
    let ticket = RemoteTicket {
        id,
        display_name: input.display_name,
        verifier_key_id: verifier_key.id().to_string(),
        verifier,
        created_unix_s: input.now_unix_s,
        expires_unix_s,
        uses_remaining: input.uses,
        max_build_time_secs: input.max_build_time_secs,
        max_upload_bytes: input.max_upload_bytes,
        bound_client_endpoint: input.bound_client_endpoint,
        revoked: false,
        runtime_verifier_key: Some(Arc::clone(verifier_key)),
    };
    Ok(TicketIssuePlan {
        ticket,
        bearer_credential,
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
    state
        .tickets
        .insert(plan.ticket.id.clone(), plan.ticket.clone());
    state.validate()?;
    Ok(redacted_ticket_view(&plan.ticket))
}

pub fn validate_presented_ticket_token(token: &str) -> Result<(), String> {
    decode_ticket_token(token).map(|_bytes| ())
}

pub fn authorize_ticket(
    ticket: &RemoteTicket,
    presented_token: &str,
    facts: &TicketPolicyFacts<'_>,
) -> TicketAuthorization {
    let Some(key) = ticket.active_verifier_key() else {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    };
    if key.id() != ticket.verifier_key_id {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    }
    let Ok(token_bytes) = decode_ticket_token(presented_token) else {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    };
    let token_bytes = Zeroizing::new(token_bytes);
    let presented_verifier = keyed_ticket_verifier(key, &token_bytes);
    if !ticket.verifier.constant_time_eq(&presented_verifier) {
        return TicketAuthorization::Rejected("ticket-authentication-failed");
    }
    apply_ticket_policy(ticket, facts)
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
    validate_key_id(retiring_key_id)?;
    let mut invalidated_ids = Vec::new();
    for ticket in state.tickets.values_mut() {
        if ticket.verifier_key_id == retiring_key_id && !ticket.revoked {
            ticket.revoked = true;
            ticket.clear_active_verifier_key();
            invalidated_ids.push(ticket.id.clone());
        }
    }
    Ok(invalidated_ids)
}

fn apply_ticket_policy(ticket: &RemoteTicket, facts: &TicketPolicyFacts<'_>) -> TicketAuthorization {
    if facts.ticket_id != ticket.id {
        return TicketAuthorization::Rejected("ticket-id-mismatch");
    }
    if facts.now_unix_s < ticket.created_unix_s {
        return TicketAuthorization::Rejected("ticket-clock-before-issuance");
    }
    if ticket.revoked {
        return TicketAuthorization::Rejected("ticket-revoked");
    }
    if facts.now_unix_s >= ticket.expires_unix_s {
        return TicketAuthorization::Rejected("ticket-expired");
    }
    if ticket.uses_remaining == 0 {
        return TicketAuthorization::Rejected("ticket-exhausted");
    }
    if let Some(bound) = ticket.bound_client_endpoint.as_deref()
        && facts.client_endpoint != Some(bound)
    {
        return TicketAuthorization::Rejected("ticket-client-endpoint-mismatch");
    }
    TicketAuthorization::Authorized
}

fn validate_ticket_issue_input(input: &TicketIssueInput) -> Result<(), String> {
    if input.display_name.is_empty() || input.display_name.len() > MAX_TICKET_DISPLAY_NAME_BYTES {
        return Err("remote-ticket-display-name-invalid".to_string());
    }
    if input.ttl_secs == 0 || input.uses == 0 {
        return Err("remote-ticket-ttl-or-uses-zero".to_string());
    }
    if input.ttl_secs > TICKET_TTL_SECS_MAX {
        return Err("remote-ticket-ttl-limit-exceeded".to_string());
    }
    if input.uses > TICKET_USES_MAX {
        return Err("remote-ticket-use-limit-exceeded".to_string());
    }
    if input.max_build_time_secs == 0 {
        return Err("remote-ticket-build-time-zero".to_string());
    }
    if input.max_build_time_secs > TICKET_BUILD_TIME_SECS_MAX {
        return Err("remote-ticket-build-time-limit-exceeded".to_string());
    }
    if input.max_upload_bytes > TICKET_UPLOAD_BYTES_MAX {
        return Err("remote-ticket-upload-limit-exceeded".to_string());
    }
    validate_bound_client_endpoint(input.bound_client_endpoint.as_deref())?;
    Ok(())
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

fn decode_ticket_token(token: &str) -> Result<[u8; TICKET_ENTROPY_BYTES], String> {
    if token.len() != TICKET_TOKEN_BASE64URL_BYTES {
        return Err("remote-ticket-token-encoding-invalid".to_string());
    }
    decode_canonical_base64url::<TICKET_ENTROPY_BYTES>(token, "remote-ticket-token-encoding-invalid")
}

fn decode_canonical_base64url<const OUTPUT_BYTES: usize>(
    encoded: &str,
    error: &'static str,
) -> Result<[u8; OUTPUT_BYTES], String> {
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
    Ok(bytes)
}

fn keyed_ticket_verifier(
    key: &TicketVerifierKey,
    token_bytes: &[u8; TICKET_ENTROPY_BYTES],
) -> TicketVerifier {
    let mut hasher = blake3::Hasher::new_keyed(key.bytes());
    hasher.update(TICKET_VERIFIER_DOMAIN);
    hasher.update(token_bytes);
    TicketVerifier(*hasher.finalize().as_bytes())
}

fn ticket_identity(input: &TicketIssueInput, sequence: u64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TICKET_IDENTITY_DOMAIN);
    hash_labeled_bytes(&mut hasher, b"display-name", input.display_name.as_bytes());
    hash_labeled_bytes(&mut hasher, b"created-unix-s", &input.now_unix_s.to_be_bytes());
    hash_labeled_bytes(&mut hasher, b"ttl-secs", &input.ttl_secs.to_be_bytes());
    hash_labeled_bytes(&mut hasher, b"uses", &input.uses.to_be_bytes());
    hash_labeled_bytes(&mut hasher, b"sequence", &sequence.to_be_bytes());
    hasher.finalize().to_hex()[..TICKET_ID_HEX_CHARS].to_string()
}

fn hash_labeled_bytes(hasher: &mut blake3::Hasher, label: &[u8], value: &[u8]) {
    let label_len = u64::try_from(label.len()).expect("fixed ticket hash label length fits u64");
    let value_len = u64::try_from(value.len()).expect("bounded ticket hash value length fits u64");
    hasher.update(&label_len.to_be_bytes()[..HASH_LENGTH_PREFIX_BYTES]);
    hasher.update(label);
    hasher.update(&value_len.to_be_bytes()[..HASH_LENGTH_PREFIX_BYTES]);
    hasher.update(value);
}

fn validate_ticket_record(map_id: &str, ticket: &RemoteTicket) -> Result<(), String> {
    validate_generated_ticket_id(map_id)?;
    if map_id != ticket.id {
        return Err("remote-ticket-state-map-id-mismatch".to_string());
    }
    if ticket.display_name.is_empty() || ticket.display_name.len() > MAX_TICKET_DISPLAY_NAME_BYTES {
        return Err("remote-ticket-state-display-name-invalid".to_string());
    }
    validate_key_id(&ticket.verifier_key_id)?;
    let ttl_span = ticket
        .expires_unix_s
        .checked_sub(ticket.created_unix_s)
        .ok_or_else(|| "remote-ticket-state-expiry-invalid".to_string())?;
    if ttl_span == 0 || ttl_span > TICKET_TTL_SECS_MAX {
        return Err("remote-ticket-state-ttl-limit-exceeded".to_string());
    }
    if ticket.uses_remaining > TICKET_USES_MAX {
        return Err("remote-ticket-state-use-limit-exceeded".to_string());
    }
    if ticket.max_build_time_secs == 0 || ticket.max_build_time_secs > TICKET_BUILD_TIME_SECS_MAX {
        return Err("remote-ticket-state-build-time-limit-invalid".to_string());
    }
    if ticket.max_upload_bytes > TICKET_UPLOAD_BYTES_MAX {
        return Err("remote-ticket-state-upload-limit-exceeded".to_string());
    }
    validate_bound_client_endpoint(ticket.bound_client_endpoint.as_deref())?;
    Ok(())
}

fn validate_generated_ticket_id(id: &str) -> Result<(), String> {
    if id.len() != TICKET_ID_HEX_CHARS || !id.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
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
        let token = plan.bearer_credential().token().to_string();
        let ticket_id = plan.ticket().id.clone();
        let view = apply_ticket_issue(&mut state, &plan).unwrap();
        let decision = authorize_ticket(
            state.tickets.get(&ticket_id).unwrap(),
            &token,
            &TicketPolicyFacts {
                ticket_id: &ticket_id,
                client_endpoint: Some("client-a"),
                now_unix_s: TEST_NOW_UNIX_S,
            },
        );

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
        excessive_ttl.tickets.get_mut(&id).unwrap().expires_unix_s = TEST_NOW_UNIX_S
            .checked_add(TICKET_TTL_SECS_MAX)
            .and_then(|value| value.checked_add(1))
            .unwrap();
        assert_eq!(excessive_ttl.validate().unwrap_err(), "remote-ticket-state-ttl-limit-exceeded");

        let mut excessive_uses = baseline.clone();
        excessive_uses.tickets.get_mut(&id).unwrap().uses_remaining = TICKET_USES_MAX.checked_add(1).unwrap();
        assert_eq!(excessive_uses.validate().unwrap_err(), "remote-ticket-state-use-limit-exceeded");

        let mut excessive_build = baseline.clone();
        excessive_build.tickets.get_mut(&id).unwrap().max_build_time_secs =
            TICKET_BUILD_TIME_SECS_MAX.checked_add(1).unwrap();
        assert_eq!(excessive_build.validate().unwrap_err(), "remote-ticket-state-build-time-limit-invalid");

        let mut excessive_upload = baseline.clone();
        excessive_upload.tickets.get_mut(&id).unwrap().max_upload_bytes = TICKET_UPLOAD_BYTES_MAX.checked_add(1).unwrap();
        assert_eq!(excessive_upload.validate().unwrap_err(), "remote-ticket-state-upload-limit-exceeded");

        for endpoint in [String::new(), "x".repeat(TICKET_BOUND_ENDPOINT_BYTES_MAX.checked_add(1).unwrap())] {
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
        assert_eq!(
            overlapping.validate().unwrap_err(),
            "remote-ticket-state-active-invalidated-id-overlap"
        );
    }

    #[test]
    fn reviewed_ttl_and_use_boundaries_are_enforced() {
        let key = key(TEST_KEY_ID, TEST_KEY_BYTE);
        let state = RemoteTicketState::default();
        let mut boundary = input();
        boundary.ttl_secs = TICKET_TTL_SECS_MAX;
        boundary.uses = TICKET_USES_MAX;
        assert!(
            plan_ticket_issue(
                &state,
                boundary,
                &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES],
                &key,
            )
            .is_ok()
        );

        let mut excessive_ttl = input();
        excessive_ttl.ttl_secs = TICKET_TTL_SECS_MAX.checked_add(1).unwrap();
        assert_eq!(
            plan_ticket_issue(
                &state,
                excessive_ttl,
                &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES],
                &key,
            )
            .unwrap_err(),
            "remote-ticket-ttl-limit-exceeded"
        );
        let mut excessive_uses = input();
        excessive_uses.uses = TICKET_USES_MAX.checked_add(1).unwrap();
        assert_eq!(
            plan_ticket_issue(
                &state,
                excessive_uses,
                &[TEST_ENTROPY_BYTE; TICKET_ENTROPY_BYTES],
                &key,
            )
            .unwrap_err(),
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
        let token = plan.bearer_credential().token().to_string();
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
        let token = plan.bearer_credential().token().to_string();
        let id = plan.ticket().id.clone();
        apply_ticket_issue(&mut state, &plan).unwrap();
        let ticket = state.tickets.get_mut(&id).unwrap();
        ticket.bind_active_verifier_key(&key);

        assert_eq!(
            authorize_ticket(
                ticket,
                &token,
                &TicketPolicyFacts {
                    ticket_id: &id,
                    client_endpoint: Some("client-a"),
                    now_unix_s: CLOCK_ROLLBACK_NOW_UNIX_S,
                },
            ),
            TicketAuthorization::Rejected("ticket-clock-before-issuance")
        );

        ticket.revoked = true;
        assert_eq!(
            authorize_ticket(
                ticket,
                &token,
                &TicketPolicyFacts {
                    ticket_id: &id,
                    client_endpoint: Some("client-a"),
                    now_unix_s: TEST_NOW_UNIX_S,
                },
            ),
            TicketAuthorization::Rejected("ticket-revoked")
        );
        ticket.revoked = false;
        assert_eq!(
            authorize_ticket(
                ticket,
                &token,
                &TicketPolicyFacts {
                    ticket_id: &id,
                    client_endpoint: Some("client-a"),
                    now_unix_s: TEST_NOW_UNIX_S + TEST_TTL_SECS,
                },
            ),
            TicketAuthorization::Rejected("ticket-expired")
        );
        ticket.uses_remaining = 0;
        assert_eq!(
            authorize_ticket(
                ticket,
                &token,
                &TicketPolicyFacts {
                    ticket_id: &id,
                    client_endpoint: Some("client-a"),
                    now_unix_s: TEST_NOW_UNIX_S,
                },
            ),
            TicketAuthorization::Rejected("ticket-exhausted")
        );
        ticket.uses_remaining = TEST_USES;
        assert_eq!(
            authorize_ticket(
                ticket,
                &token,
                &TicketPolicyFacts {
                    ticket_id: &id,
                    client_endpoint: Some("client-b"),
                    now_unix_s: TEST_NOW_UNIX_S,
                },
            ),
            TicketAuthorization::Rejected("ticket-client-endpoint-mismatch")
        );
    }

    #[test]
    fn state_serialization_contains_verifier_but_not_bearer_material() {
        let (mut state, plan, _key) = issued();
        let credential = plan.bearer_credential().as_str().to_string();
        let token = plan.bearer_credential().token().to_string();
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
        let legacy_seed = format!(
            "{}:{}:{}:{}:{}",
            input().display_name,
            TEST_NOW_UNIX_S,
            TEST_TTL_SECS,
            TEST_USES,
            0
        );
        let predicted_legacy_secret = blake3::hash(legacy_seed.as_bytes()).to_hex().to_string();
        let (_state, plan, _key) = issued();

        assert_eq!(predicted_legacy_secret.len(), 64);
        assert_ne!(plan.bearer_credential().token(), predicted_legacy_secret);
        assert_eq!(plan.bearer_credential().token().len(), TICKET_TOKEN_BASE64URL_BYTES);
    }

    #[test]
    fn verifier_key_parser_rejects_malformed_and_zero_keys() {
        assert_eq!(
            TicketVerifierKey::parse("missing-separator").unwrap_err(),
            "remote-ticket-verifier-key-format-invalid"
        );
        assert_eq!(
            TicketVerifierKey::parse("bad key:AAAA").unwrap_err(),
            "remote-ticket-verifier-key-id-invalid"
        );
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
}
