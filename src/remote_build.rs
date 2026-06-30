use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;

pub const REMOTE_PROTOCOL_ALPN: &str = "mantle-remote-build/1";
pub const REMOTE_PROTOCOL_VERSION: u32 = 1;
pub const MAX_REMOTE_CAPABILITIES: usize = 32;
pub const MAX_REMOTE_INPUT_REFS: usize = 1_000_000;
pub const MAX_REMOTE_UPLOAD_BYTES: u64 = 1_099_511_627_776;
pub const MAX_REMOTE_BUILD_TIME_SECS: u64 = 86_400;
pub const DEFAULT_TICKET_TTL_SECS: u64 = 3_600;
pub const DEFAULT_TICKET_USES: u32 = 1;
pub const DEFAULT_TICKET_BUILD_TIME_SECS: u64 = 3_600;
pub const DEFAULT_TICKET_UPLOAD_BYTES: u64 = 1_073_741_824;
pub const MAX_TICKET_DISPLAY_NAME_BYTES: usize = 128;
const TICKET_STATE_DIR: &str = "remote-builders";
const TICKET_STATE_FILE: &str = "tickets.json";
const SECRET_REDACTION: &str = "<redacted>";
const TEMP_FILE_EXTENSION: &str = "tmp";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteHello {
    pub alpn: String,
    pub version: u32,
    pub endpoint_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedHello {
    pub endpoint_id: String,
    pub accepted_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTicket {
    pub id: String,
    pub display_name: String,
    pub secret: String,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub uses_remaining: u32,
    pub max_build_time_secs: u64,
    pub max_upload_bytes: u64,
    pub bound_client_endpoint: Option<String>,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteTicketView {
    pub id: String,
    pub display_name: String,
    pub secret: String,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub uses_remaining: u32,
    pub max_build_time_secs: u64,
    pub max_upload_bytes: u64,
    pub bound_client_endpoint: Option<String>,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RemoteTicketState {
    pub tickets: BTreeMap<String, RemoteTicket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketAuthRequest {
    pub ticket_id: String,
    pub secret: String,
    pub client_endpoint: Option<String>,
    pub now_unix_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcreteBuildRequest {
    pub request_id: String,
    pub store_prefix: String,
    pub input_refs: Vec<String>,
    pub upload_bytes: u64,
    pub build_time_limit_secs: u64,
    pub contains_raw_frontend_eval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketDecision {
    Authorized,
    Reject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolDecision {
    Proceed(AcceptedHello),
    Reject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputTrustDecision {
    Accept { key_id: String },
    Reject(String),
}

pub fn validate_hello(
    hello: &RemoteHello,
    expected_endpoint_id: &str,
    supported_capabilities: &[String],
) -> ProtocolDecision {
    if hello.alpn != REMOTE_PROTOCOL_ALPN {
        return ProtocolDecision::Reject(format!("unsupported ALPN {}", hello.alpn));
    }
    if hello.version != REMOTE_PROTOCOL_VERSION {
        return ProtocolDecision::Reject(format!("unsupported protocol version {}", hello.version));
    }
    if hello.endpoint_id != expected_endpoint_id {
        return ProtocolDecision::Reject("endpoint identity mismatch".to_string());
    }
    if hello.capabilities.len() > MAX_REMOTE_CAPABILITIES {
        return ProtocolDecision::Reject(format!("capability count exceeds {MAX_REMOTE_CAPABILITIES}"));
    }
    let accepted_capabilities = hello
        .capabilities
        .iter()
        .filter(|capability| supported_capabilities.contains(capability))
        .cloned()
        .collect::<Vec<_>>();
    ProtocolDecision::Proceed(AcceptedHello {
        endpoint_id: hello.endpoint_id.clone(),
        accepted_capabilities,
    })
}

pub fn authorize_ticket(ticket: &RemoteTicket, request: &TicketAuthRequest) -> TicketDecision {
    if ticket.revoked {
        return TicketDecision::Reject("ticket-revoked".to_string());
    }
    if request.now_unix_s >= ticket.expires_unix_s {
        return TicketDecision::Reject("ticket-expired".to_string());
    }
    if ticket.uses_remaining == 0 {
        return TicketDecision::Reject("ticket-exhausted".to_string());
    }
    if request.secret != ticket.secret {
        return TicketDecision::Reject("ticket-secret-mismatch".to_string());
    }
    if let Some(bound) = &ticket.bound_client_endpoint
        && request.client_endpoint.as_deref() != Some(bound.as_str())
    {
        return TicketDecision::Reject("ticket-client-endpoint-mismatch".to_string());
    }
    TicketDecision::Authorized
}

pub fn validate_concrete_request(request: &ConcreteBuildRequest, ticket: &RemoteTicket) -> Result<(), String> {
    if request.contains_raw_frontend_eval {
        return Err("raw-frontend-evaluation-rejected".to_string());
    }
    if !request.store_prefix.starts_with('/') {
        return Err("store-prefix-not-absolute".to_string());
    }
    if request.input_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"));
    }
    if request.upload_bytes > ticket.max_upload_bytes || request.upload_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err("upload-byte-limit-exceeded".to_string());
    }
    if request.build_time_limit_secs > ticket.max_build_time_secs
        || request.build_time_limit_secs > MAX_REMOTE_BUILD_TIME_SECS
    {
        return Err("build-time-limit-exceeded".to_string());
    }
    Ok(())
}

pub fn redeem_after_queue(ticket: &mut RemoteTicket, request_validated: bool) -> Result<(), String> {
    if !request_validated {
        return Ok(());
    }
    if ticket.uses_remaining == 0 {
        return Err("ticket-exhausted".to_string());
    }
    ticket.uses_remaining = ticket.uses_remaining.saturating_sub(1);
    Ok(())
}

pub fn derive_missing_inputs(declared_refs: &[String], present_refs: &[String]) -> Result<Vec<String>, String> {
    if declared_refs.len() > MAX_REMOTE_INPUT_REFS || present_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"));
    }
    let present = present_refs.iter().collect::<std::collections::BTreeSet<_>>();
    Ok(declared_refs.iter().filter(|reference| !present.contains(reference)).cloned().collect::<Vec<_>>())
}

pub fn decide_output_trust(
    signing_key_id: &str,
    trusted_key_ids: &[String],
    store_prefix_matches: bool,
) -> OutputTrustDecision {
    if !store_prefix_matches {
        return OutputTrustDecision::Reject("store-prefix-mismatch".to_string());
    }
    if trusted_key_ids.iter().any(|trusted| trusted == signing_key_id) {
        return OutputTrustDecision::Accept {
            key_id: signing_key_id.to_string(),
        };
    }
    OutputTrustDecision::Reject("untrusted-output-key".to_string())
}

pub fn redacted_ticket_view(ticket: &RemoteTicket) -> RemoteTicketView {
    RemoteTicketView {
        id: ticket.id.clone(),
        display_name: ticket.display_name.clone(),
        secret: SECRET_REDACTION.to_string(),
        created_unix_s: ticket.created_unix_s,
        expires_unix_s: ticket.expires_unix_s,
        uses_remaining: ticket.uses_remaining,
        max_build_time_secs: ticket.max_build_time_secs,
        max_upload_bytes: ticket.max_upload_bytes,
        bound_client_endpoint: ticket.bound_client_endpoint.clone(),
        revoked: ticket.revoked,
    }
}

pub fn revealed_ticket_view(ticket: &RemoteTicket) -> RemoteTicketView {
    let mut view = redacted_ticket_view(ticket);
    view.secret = ticket.secret.clone();
    view
}

pub fn create_ticket(
    state: &mut RemoteTicketState,
    display_name: String,
    now_unix_s: u64,
    ttl_secs: u64,
    uses: u32,
    max_build_time_secs: u64,
    max_upload_bytes: u64,
    bound_client_endpoint: Option<String>,
) -> Result<RemoteTicketView, RunError> {
    validate_ticket_limits(&display_name, ttl_secs, uses, max_build_time_secs, max_upload_bytes)?;
    let seed = format!("{display_name}:{now_unix_s}:{ttl_secs}:{uses}:{}", state.tickets.len());
    let secret_hash = blake3::hash(seed.as_bytes());
    let secret = secret_hash.to_hex().to_string();
    let id = blake3::hash(secret.as_bytes()).to_hex()[..16].to_string();
    let ticket = RemoteTicket {
        id: id.clone(),
        display_name,
        secret,
        created_unix_s: now_unix_s,
        expires_unix_s: now_unix_s.saturating_add(ttl_secs),
        uses_remaining: uses,
        max_build_time_secs,
        max_upload_bytes,
        bound_client_endpoint,
        revoked: false,
    };
    state.tickets.insert(id, ticket.clone());
    Ok(redacted_ticket_view(&ticket))
}

pub fn revoke_ticket(state: &mut RemoteTicketState, id: &str) -> Result<RemoteTicketView, RunError> {
    let ticket = state.tickets.get_mut(id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
    ticket.revoked = true;
    Ok(redacted_ticket_view(ticket))
}

pub fn load_ticket_state(state_dir: &Path) -> Result<RemoteTicketState, RunError> {
    let path = ticket_state_path(state_dir);
    if !path.exists() {
        return Ok(RemoteTicketState::default());
    }
    let bytes = fs::read(&path)
        .map_err(|err| RunError::Internal(format!("reading remote ticket state {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing remote ticket state {}: {err}", path.display())))
}

pub fn save_ticket_state(state_dir: &Path, state: &RemoteTicketState) -> Result<(), RunError> {
    let path = ticket_state_path(state_dir);
    let parent = path.parent().expect("ticket state path has parent");
    fs::create_dir_all(parent)
        .map_err(|err| RunError::Internal(format!("creating remote ticket state dir {}: {err}", parent.display())))?;
    let rendered = serde_json::to_string_pretty(state)
        .map_err(|err| RunError::Internal(format!("serializing remote ticket state: {err}")))?;
    let tmp = path.with_extension(TEMP_FILE_EXTENSION);
    fs::write(&tmp, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing remote ticket state temp {}: {err}", tmp.display())))?;
    fs::rename(&tmp, &path)
        .map_err(|err| RunError::Internal(format!("committing remote ticket state {}: {err}", path.display())))
}

fn validate_ticket_limits(
    display_name: &str,
    ttl_secs: u64,
    uses: u32,
    max_build_time_secs: u64,
    max_upload_bytes: u64,
) -> Result<(), RunError> {
    if display_name.is_empty() || display_name.len() > MAX_TICKET_DISPLAY_NAME_BYTES {
        return Err(RunError::Internal(format!("ticket display name exceeds {MAX_TICKET_DISPLAY_NAME_BYTES} bytes")));
    }
    if ttl_secs == 0 || uses == 0 {
        return Err(RunError::Internal("ticket ttl and uses must be non-zero".to_string()));
    }
    if max_build_time_secs == 0 || max_build_time_secs > MAX_REMOTE_BUILD_TIME_SECS {
        return Err(RunError::Internal(format!("ticket build time limit must be 1..={MAX_REMOTE_BUILD_TIME_SECS}")));
    }
    if max_upload_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err(RunError::Internal(format!("ticket upload limit exceeds {MAX_REMOTE_UPLOAD_BYTES}")));
    }
    Ok(())
}

fn ticket_state_path(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join(TICKET_STATE_DIR).join(TICKET_STATE_FILE)
}

pub fn cmd_remote(action: crate::RemoteAction, state_dir: &Path, json_output: bool) -> Result<(), RunError> {
    match action {
        crate::RemoteAction::Ticket { action } => cmd_remote_ticket(action, state_dir, json_output),
        crate::RemoteAction::Serve { endpoint_id } => {
            let rendered = serde_json::json!({
                "protocol": REMOTE_PROTOCOL_ALPN,
                "endpoint_id": endpoint_id,
                "status": "transport-not-started",
                "diagnostic": "remote serve requires a concrete P2P transport binding before accepting builds"
            });
            print_json_or_human(&rendered, json_output)
        }
    }
}

fn cmd_remote_ticket(action: crate::RemoteTicketAction, state_dir: &Path, json_output: bool) -> Result<(), RunError> {
    let mut state = load_ticket_state(state_dir)?;
    match action {
        crate::RemoteTicketAction::Create {
            display_name,
            now_unix_s,
            ttl_secs,
            uses,
            max_build_time_secs,
            max_upload_bytes,
            bound_client_endpoint,
        } => {
            let view = create_ticket(
                &mut state,
                display_name,
                now_unix_s,
                ttl_secs,
                uses,
                max_build_time_secs,
                max_upload_bytes,
                bound_client_endpoint,
            )?;
            save_ticket_state(state_dir, &state)?;
            print_json_or_human(&view, json_output)
        }
        crate::RemoteTicketAction::List => {
            let views = state.tickets.values().map(redacted_ticket_view).collect::<Vec<_>>();
            print_json_or_human(&views, json_output)
        }
        crate::RemoteTicketAction::Inspect { id } => {
            let ticket =
                state.tickets.get(&id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
            print_json_or_human(&redacted_ticket_view(ticket), json_output)
        }
        crate::RemoteTicketAction::Reveal { id } => {
            let ticket =
                state.tickets.get(&id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
            print_json_or_human(&revealed_ticket_view(ticket), json_output)
        }
        crate::RemoteTicketAction::Revoke { id } => {
            let view = revoke_ticket(&mut state, &id)?;
            save_ticket_state(state_dir, &state)?;
            print_json_or_human(&view, json_output)
        }
    }
}

fn print_json_or_human(value: &impl Serialize, json_output: bool) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|err| RunError::Internal(format!("serializing remote output: {err}")))?;
    if json_output {
        println!("{rendered}");
        return Ok(());
    }
    println!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_hello_reaches_authorization() {
        let hello = RemoteHello {
            alpn: REMOTE_PROTOCOL_ALPN.to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: "builder-1".to_string(),
            capabilities: vec!["delta".to_string(), "full".to_string()],
        };
        let decision = validate_hello(&hello, "builder-1", &["delta".to_string()]);
        assert!(matches!(decision, ProtocolDecision::Proceed(_)));
    }

    #[test]
    fn protocol_mismatch_fails_closed() {
        let hello = RemoteHello {
            alpn: "wrong".to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: "builder-1".to_string(),
            capabilities: Vec::new(),
        };
        let decision = validate_hello(&hello, "builder-1", &[]);
        assert!(matches!(decision, ProtocolDecision::Reject(reason) if reason.contains("unsupported ALPN")));
    }

    #[test]
    fn malformed_request_does_not_consume_ticket() {
        let mut ticket = fixture_ticket();
        let request = ConcreteBuildRequest {
            request_id: "r1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            input_refs: Vec::new(),
            upload_bytes: 0,
            build_time_limit_secs: 1,
            contains_raw_frontend_eval: true,
        };
        assert!(validate_concrete_request(&request, &ticket).is_err());
        redeem_after_queue(&mut ticket, false).unwrap();
        assert_eq!(ticket.uses_remaining, 1);
    }

    #[test]
    fn valid_request_redeems_ticket_once() {
        let mut ticket = fixture_ticket();
        let auth = TicketAuthRequest {
            ticket_id: ticket.id.clone(),
            secret: ticket.secret.clone(),
            client_endpoint: Some("client-a".to_string()),
            now_unix_s: 2,
        };
        assert_eq!(authorize_ticket(&ticket, &auth), TicketDecision::Authorized);
        let request = ConcreteBuildRequest {
            request_id: "r1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            input_refs: vec!["input-a".to_string()],
            upload_bytes: 1,
            build_time_limit_secs: 1,
            contains_raw_frontend_eval: false,
        };
        validate_concrete_request(&request, &ticket).unwrap();
        redeem_after_queue(&mut ticket, true).unwrap();
        assert_eq!(ticket.uses_remaining, 0);
    }

    #[test]
    fn status_view_redacts_ticket_secret() {
        let ticket = fixture_ticket();
        let view = redacted_ticket_view(&ticket);
        assert_eq!(view.secret, SECRET_REDACTION);
        let revealed = revealed_ticket_view(&ticket);
        assert_eq!(revealed.secret, ticket.secret);
    }

    #[test]
    fn missing_inputs_are_derived_from_declared_refs() {
        let missing =
            derive_missing_inputs(&["a".to_string(), "b".to_string(), "c".to_string()], &["b".to_string()]).unwrap();
        assert_eq!(missing, vec!["a".to_string(), "c".to_string()]);
    }

    #[test]
    fn output_trust_requires_trusted_key_and_prefix() {
        let trusted = vec!["builder-key".to_string()];
        assert!(matches!(decide_output_trust("builder-key", &trusted, true), OutputTrustDecision::Accept { .. }));
        assert!(matches!(
            decide_output_trust("other", &trusted, true),
            OutputTrustDecision::Reject(reason) if reason == "untrusted-output-key"
        ));
        assert!(matches!(
            decide_output_trust("builder-key", &trusted, false),
            OutputTrustDecision::Reject(reason) if reason == "store-prefix-mismatch"
        ));
    }

    fn fixture_ticket() -> RemoteTicket {
        RemoteTicket {
            id: "ticket-1".to_string(),
            display_name: "test".to_string(),
            secret: "secret".to_string(),
            created_unix_s: 1,
            expires_unix_s: 10,
            uses_remaining: 1,
            max_build_time_secs: 10,
            max_upload_bytes: 10,
            bound_client_endpoint: Some("client-a".to_string()),
            revoked: false,
        }
    }
}
