//! Bounded W3C trace-context validation and diagnostic health facts.
//!
//! Trace context is transported separately from build requests and is never an
//! input to identity, authorization, scheduling, fencing, cache, or trust logic.
//!
//! r[impl remote_builds.diagnostic_trace_context]

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub const REMOTE_TRACE_CONTEXT_CAPABILITY: &str = "w3c-trace-context-v1";
pub const REMOTE_TRACE_CONTEXT_NON_CLAIM: &str = "trace context is diagnostic only and does not affect request identity, authorization, scheduling, fencing, cache identity, output trust, or build results";
pub const W3C_TRACEPARENT_BYTES: usize = 55;
pub const W3C_TRACESTATE_BYTES_MAX: usize = 512;
pub const W3C_TRACESTATE_MEMBERS_MAX: u32 = 32;
pub const W3C_TRACESTATE_MEMBER_BYTES_MAX: usize = 256;
const W3C_TRACESTATE_TENANT_BYTES_MAX: usize = 241;
const W3C_TRACESTATE_SYSTEM_BYTES_MAX: usize = 14;
const ASCII_VISIBLE_MIN: u8 = 0x20;
const ASCII_VISIBLE_MAX: u8 = 0x7e;
const TRACE_ID_START: usize = 3;
const TRACE_ID_END: usize = 35;
const PARENT_ID_START: usize = 36;
const PARENT_ID_END: usize = 52;
const TRACE_FLAGS_START: usize = 53;
const TRACE_FLAGS_END: usize = W3C_TRACEPARENT_BYTES;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTraceContext {
    pub traceparent: String,
    #[serde(default = "no_trace_context_value", skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTraceContextStatus {
    Disabled,
    Accepted,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTraceContextHealth {
    pub status: RemoteTraceContextStatus,
    pub reason_code: String,
    #[serde(default = "no_trace_context_value", skip_serializing_if = "Option::is_none")]
    pub context_digest_blake3: Option<String>,
    pub non_claim: String,
}

fn no_trace_context_value() -> Option<String> {
    None
}

pub fn accept_remote_trace_context(
    enabled: bool,
    traceparent: Option<&str>,
    tracestate: Option<&str>,
) -> (Option<RemoteTraceContext>, RemoteTraceContextHealth) {
    if !enabled {
        return (None, trace_health(RemoteTraceContextStatus::Disabled, "remote-trace-context-disabled", None));
    }
    let Some(traceparent) = traceparent else {
        return (None, trace_health(RemoteTraceContextStatus::Dropped, "remote-traceparent-missing", None));
    };
    let context = RemoteTraceContext {
        traceparent: traceparent.to_string(),
        tracestate: tracestate.map(str::to_string),
    };
    if let Err(reason) = validate_remote_trace_context(&context) {
        return (None, trace_health(RemoteTraceContextStatus::Dropped, reason, None));
    }
    let digest = match remote_trace_context_digest(&context) {
        Ok(digest) => digest,
        Err(reason) => return (None, trace_health(RemoteTraceContextStatus::Dropped, reason, None)),
    };
    debug_assert_eq!(digest.len(), blake3::OUT_LEN.saturating_mul(2));
    debug_assert!(!context.traceparent.is_empty());
    (
        Some(context),
        trace_health(RemoteTraceContextStatus::Accepted, "remote-trace-context-accepted", Some(digest)),
    )
}

pub fn validate_remote_trace_context(context: &RemoteTraceContext) -> Result<(), &'static str> {
    validate_traceparent(&context.traceparent)?;
    if let Some(tracestate) = context.tracestate.as_deref() {
        validate_tracestate(tracestate)?;
    }
    debug_assert_eq!(context.traceparent.len(), W3C_TRACEPARENT_BYTES);
    debug_assert!(context.tracestate.as_ref().is_none_or(|value| value.len() <= W3C_TRACESTATE_BYTES_MAX));
    Ok(())
}

pub fn remote_trace_context_digest(context: &RemoteTraceContext) -> Result<String, &'static str> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-remote-trace-context-v1\0");
    hash_component(&mut hasher, context.traceparent.as_bytes())?;
    hash_component(&mut hasher, context.tracestate.as_deref().unwrap_or_default().as_bytes())?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn validate_traceparent(value: &str) -> Result<(), &'static str> {
    debug_assert!(TRACE_ID_START < TRACE_ID_END);
    debug_assert!(PARENT_ID_START < PARENT_ID_END);
    if value.len() != W3C_TRACEPARENT_BYTES {
        return Err("remote-traceparent-length-invalid");
    }
    let bytes = value.as_bytes();
    if bytes.get(2) != Some(&b'-') || bytes.get(35) != Some(&b'-') || bytes.get(52) != Some(&b'-') {
        return Err("remote-traceparent-layout-invalid");
    }
    if &bytes[..2] != b"00" {
        return Err("remote-traceparent-version-unsupported");
    }
    let trace_id = &bytes[TRACE_ID_START..TRACE_ID_END];
    let parent_id = &bytes[PARENT_ID_START..PARENT_ID_END];
    let trace_flags = &bytes[TRACE_FLAGS_START..TRACE_FLAGS_END];
    if !lower_hex(trace_id) || !lower_hex(parent_id) || !lower_hex(trace_flags) {
        return Err("remote-traceparent-hex-invalid");
    }
    if all_zero(trace_id) || all_zero(parent_id) {
        return Err("remote-traceparent-zero-identity");
    }
    Ok(())
}

fn validate_tracestate(value: &str) -> Result<(), &'static str> {
    if value.is_empty() || value.len() > W3C_TRACESTATE_BYTES_MAX || !value.is_ascii() {
        return Err("remote-tracestate-bounds-invalid");
    }
    let members = value.split(',').collect::<Vec<_>>();
    let member_count = u32::try_from(members.len()).map_err(|_| "remote-tracestate-member-count-invalid")?;
    if member_count == 0 || member_count > W3C_TRACESTATE_MEMBERS_MAX {
        return Err("remote-tracestate-member-count-invalid");
    }
    let mut keys = BTreeSet::new();
    for member in members {
        let key = validate_tracestate_member(member)?;
        if !keys.insert(key) {
            return Err("remote-tracestate-duplicate-key");
        }
    }
    let member_count_usize = usize::try_from(member_count).map_err(|_| "remote-tracestate-member-count-invalid")?;
    debug_assert_eq!(keys.len(), member_count_usize);
    debug_assert!(member_count <= W3C_TRACESTATE_MEMBERS_MAX);
    Ok(())
}

fn validate_tracestate_member(member: &str) -> Result<&str, &'static str> {
    if member.is_empty() || member.len() > W3C_TRACESTATE_MEMBER_BYTES_MAX || member.trim() != member {
        return Err("remote-tracestate-member-invalid");
    }
    let Some((key, value)) = member.split_once('=') else {
        return Err("remote-tracestate-member-invalid");
    };
    if value.is_empty() || value.contains('=') || !valid_tracestate_key(key) {
        return Err("remote-tracestate-member-invalid");
    }
    if value
        .bytes()
        .any(|byte| !(ASCII_VISIBLE_MIN..=ASCII_VISIBLE_MAX).contains(&byte) || byte == b',' || byte == b'=')
    {
        return Err("remote-tracestate-member-invalid");
    }
    Ok(key)
}

fn valid_tracestate_key(key: &str) -> bool {
    if key.is_empty() || key.len() > W3C_TRACESTATE_MEMBER_BYTES_MAX {
        return false;
    }
    let Some((tenant, system)) = key.split_once('@') else {
        return starts_lowercase(key) && key.bytes().all(valid_tracestate_key_tail_byte);
    };
    if system.contains('@') || tenant.len() > W3C_TRACESTATE_TENANT_BYTES_MAX {
        return false;
    }
    if system.len() > W3C_TRACESTATE_SYSTEM_BYTES_MAX || !starts_lowercase(system) {
        return false;
    }
    let is_tenant_start_valid =
        tenant.as_bytes().first().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    is_tenant_start_valid
        && tenant.bytes().all(valid_tracestate_key_tail_byte)
        && system.bytes().all(valid_tracestate_key_tail_byte)
}

fn starts_lowercase(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
}

fn valid_tracestate_key_tail_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'*' | b'/')
}

fn lower_hex(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn all_zero(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes.iter().all(|byte| *byte == b'0')
}

fn hash_component(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), &'static str> {
    let length_bytes = u64::try_from(bytes.len()).map_err(|_| "remote-trace-context-length-invalid")?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn trace_health(
    status: RemoteTraceContextStatus,
    reason_code: &str,
    context_digest_blake3: Option<String>,
) -> RemoteTraceContextHealth {
    RemoteTraceContextHealth {
        status,
        reason_code: reason_code.to_string(),
        context_digest_blake3,
        non_claim: REMOTE_TRACE_CONTEXT_NON_CLAIM.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    #[test]
    fn valid_w3c_context_is_accepted_and_digest_stable() {
        let (first, health) = accept_remote_trace_context(true, Some(VALID_TRACEPARENT), Some("vendor=value"));
        let (second, _) = accept_remote_trace_context(true, Some(VALID_TRACEPARENT), Some("vendor=value"));

        assert_eq!(first, second);
        assert_eq!(health.status, RemoteTraceContextStatus::Accepted);
        assert_eq!(health.context_digest_blake3.as_deref().map(str::len), Some(blake3::OUT_LEN * 2));
        assert!(!serde_json::to_string(&health).unwrap().contains(VALID_TRACEPARENT));
    }

    #[test]
    fn malformed_oversized_zero_and_non_w3c_contexts_are_dropped_without_values_in_health() {
        let oversized = "x".repeat(W3C_TRACESTATE_BYTES_MAX.saturating_add(1));
        let cases = [
            (Some("malformed"), None),
            (Some("00-00000000000000000000000000000000-00f067aa0ba902b7-01"), None),
            (Some(VALID_TRACEPARENT), Some(oversized.as_str())),
            (Some(VALID_TRACEPARENT), Some("1simple=value")),
            (Some(VALID_TRACEPARENT), Some("tenant@@system=value")),
            (Some(VALID_TRACEPARENT), Some("vendor=first,vendor=second")),
            (None, Some("vendor=value")),
        ];
        for (traceparent, tracestate) in cases {
            let (context, health) = accept_remote_trace_context(true, traceparent, tracestate);
            assert!(context.is_none());
            assert_eq!(health.status, RemoteTraceContextStatus::Dropped);
            assert!(health.context_digest_blake3.is_none());
            assert!(!health.reason_code.contains(VALID_TRACEPARENT));
        }
    }

    #[test]
    fn disabled_context_ignores_even_valid_ambient_values() {
        let (context, health) = accept_remote_trace_context(false, Some(VALID_TRACEPARENT), Some("vendor=value"));

        assert!(context.is_none());
        assert_eq!(health.status, RemoteTraceContextStatus::Disabled);
        assert_eq!(health.reason_code, "remote-trace-context-disabled");
    }
}
