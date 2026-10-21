use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::Signature;
use ed25519_dalek::Signer as _;
use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use super::Authority;
use super::CURSOR_TTL_SECS;
use super::Error;
use super::Phase;
use super::core;

const COMPLETION_DOMAIN: &[u8] = b"mantle-gateway-completion-v1\0";
const CURSOR_BYTES: usize = 82;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    pub version: u32,
    pub event_id: [u8; 32],
    pub attempt_id: String,
    pub request_digest: [u8; 32],
    pub terminal: Phase,
    pub result_evidence: Option<[u8; 32]>,
    pub policy_version: [u8; 32],
    pub sequence: u64,
    pub producer: [u8; 32],
    pub signature: Vec<u8>,
}

impl Completion {
    pub(super) fn create(record: &super::Record, sequence: u64, signer: &SigningKey) -> Result<Self, Error> {
        let mut result = Self {
            version: super::API_VERSION,
            event_id: [0; 32],
            attempt_id: record.id.clone(),
            request_digest: record.digest,
            terminal: record.observation.phase,
            result_evidence: record.observation.result_evidence,
            policy_version: record.policy,
            sequence,
            producer: signer.verifying_key().to_bytes(),
            signature: Vec::new(),
        };
        result.event_id = result.identity()?;
        let mut message = COMPLETION_DOMAIN.to_vec();
        message.extend_from_slice(&result.event_id);
        result.signature = signer.sign(&message).to_bytes().to_vec();
        Ok(result)
    }

    fn identity(&self) -> Result<[u8; 32], Error> {
        // Fixed field order and domain separation are part of wire version 1.
        // Signature and event_id are excluded from the signed identity.
        let bytes = serde_json::to_vec(&(
            self.version,
            &self.attempt_id,
            self.request_digest,
            self.terminal,
            self.result_evidence,
            self.policy_version,
            self.sequence,
            self.producer,
        ))
        .map_err(|_| Error::Integrity)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(COMPLETION_DOMAIN);
        hasher.update(&bytes);
        Ok(*hasher.finalize().as_bytes())
    }
}

/// A trusted producer key MUST come from operator configuration, not the event.
/// This verifies linkage only; it is NOT a release or reusable-output grant.
pub fn verify_completion(event: &Completion, trusted: &VerifyingKey) -> Result<(), Error> {
    if event.version != super::API_VERSION
        || !event.terminal.terminal()
        || event.sequence == 0
        || !super::identity(&event.attempt_id)
        || event.producer != trusted.to_bytes()
        || event.event_id != event.identity()?
    {
        return Err(Error::Integrity);
    }
    let signature = Signature::from_slice(&event.signature).map_err(|_| Error::Integrity)?;
    let mut message = COMPLETION_DOMAIN.to_vec();
    message.extend_from_slice(&event.event_id);
    trusted.verify_strict(&message, &signature).map_err(|_| Error::Integrity)
}

pub(super) fn slot(secret: &[u8; 32], fields: &[&str]) -> [u8; 32] {
    let mut hash = blake3::Hasher::new_keyed(secret);
    for field in fields {
        hash.update(&(field.len() as u64).to_le_bytes());
        hash.update(field.as_bytes());
    }
    *hash.finalize().as_bytes()
}

fn binding(secret: &[u8; 32], authority: Authority<'_>, id: &str) -> [u8; 32] {
    slot(secret, &[
        "gateway-cursor-v1",
        authority.subject,
        authority.scope,
        authority.audience,
        id,
    ])
}

pub(super) fn cursor_write(
    secret: &[u8; 32],
    authority: Authority<'_>,
    id: &str,
    kind: u8,
    offset: u64,
    now: u64,
) -> Result<String, Error> {
    let expires = now.checked_add(CURSOR_TTL_SECS).ok_or(Error::Cursor)?;
    let mut bytes = Vec::with_capacity(CURSOR_BYTES);
    bytes.extend_from_slice(&[1, kind]);
    bytes.extend_from_slice(&offset.to_le_bytes());
    bytes.extend_from_slice(&expires.to_le_bytes());
    bytes.extend_from_slice(&binding(secret, authority, id));
    let mac = blake3::keyed_hash(secret, &bytes);
    bytes.extend_from_slice(mac.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub(super) fn cursor_read(
    secret: &[u8; 32],
    cursor: Option<&str>,
    authority: Authority<'_>,
    id: &str,
    kind: u8,
    now: u64,
) -> Result<u64, Error> {
    let Some(cursor) = cursor else { return Ok(0) };
    if cursor.len() > core::MAX_CURSOR_BYTES as usize {
        return Err(Error::Cursor);
    }
    let bytes = URL_SAFE_NO_PAD.decode(cursor).map_err(|_| Error::Cursor)?;
    if bytes.len() != CURSOR_BYTES || bytes[0] != 1 || bytes[1] != kind {
        return Err(Error::Cursor);
    }
    let tag: [u8; 32] = bytes[50..].try_into().map_err(|_| Error::Cursor)?;
    // blake3::Hash equality uses constant-time comparison.
    if blake3::keyed_hash(secret, &bytes[..50]) != blake3::Hash::from(tag)
        || bytes[18..50] != binding(secret, authority, id)
    {
        return Err(Error::Cursor);
    }
    let expires = u64::from_le_bytes(bytes[10..18].try_into().map_err(|_| Error::Cursor)?);
    if now >= expires || expires.saturating_sub(now) > CURSOR_TTL_SECS {
        return Err(Error::Cursor);
    }
    Ok(u64::from_le_bytes(bytes[2..10].try_into().map_err(|_| Error::Cursor)?))
}
