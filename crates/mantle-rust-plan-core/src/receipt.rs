//! Canonical serialization, validation, and hashing for accepted Rust-plan receipts.

use alloc::string::String;
use alloc::string::ToString;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::digest::is_lowercase_blake3_hex;

/// Accepted Rust unit replay-evidence schema version.
pub const RUST_UNIT_REPLAY_EVIDENCE_SCHEMA_VERSION: u32 = 1;
/// Maximum boundary items accepted per replay-evidence count field.
pub const RUST_UNIT_REPLAY_EVIDENCE_MAX_BOUNDARY_ITEMS: u32 = 4096;
/// Maximum JSON byte size accepted for replay evidence.
pub const RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES: u32 = 16 * 1024;

const _: () = {
    assert!(RUST_UNIT_REPLAY_EVIDENCE_SCHEMA_VERSION > 0);
    assert!(RUST_UNIT_REPLAY_EVIDENCE_MAX_BOUNDARY_ITEMS > 0);
    assert!(RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES > 0);
};

/// Failure to serialize the accepted receipt preimage in canonical field order.
#[derive(Debug)]
pub enum ReceiptError {
    Canonicalization(serde_json::Error),
}

impl core::fmt::Display for ReceiptError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Canonicalization(error) => write!(formatter, "canonicalizing Rust plan receipt: {error}"),
        }
    }
}

/// Typed validation and decoding failures for Rust unit replay evidence.
#[derive(Debug)]
pub enum ReplayEvidenceError {
    JsonTooLarge { max_bytes: u32 },
    MalformedJson(serde_json::Error),
    UnsupportedSchemaVersion,
    EmptyIdentity,
    InvalidBlake3Field { field: &'static str },
    CountExceedsLimit { field: &'static str },
}

impl core::fmt::Display for ReplayEvidenceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::JsonTooLarge { max_bytes } => {
                write!(formatter, "rust unit replay evidence JSON exceeds {max_bytes} byte limit")
            }
            Self::MalformedJson(error) => write!(formatter, "rust unit replay evidence JSON is malformed: {error}"),
            Self::UnsupportedSchemaVersion => {
                formatter.write_str("rust unit replay evidence schema version is unsupported")
            }
            Self::EmptyIdentity => formatter.write_str("rust unit replay evidence identity fields must be non-empty"),
            Self::InvalidBlake3Field { field } => {
                write!(formatter, "rust unit replay evidence field {field} is not a lowercase BLAKE3 digest")
            }
            Self::CountExceedsLimit { field } => {
                write!(formatter, "rust unit replay evidence field {field} exceeds bounded item limit")
            }
        }
    }
}

/// Borrowed validation view of `RustUnitReplayEvidence` preserving exact
/// canonical serialization in the adapter without `usize` in the core API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayEvidenceFacts<'a> {
    pub schema_version: u32,
    pub unit_id: &'a str,
    pub package_id: &'a str,
    pub blocker_class: &'a str,
    pub rustc_args_digest_blake3: &'a str,
    pub environment_digest_blake3: &'a str,
    pub toolchain_policy_digest_blake3: &'a str,
    pub artifact_identity_digest_blake3: &'a str,
    pub declared_outputs_digest_blake3: &'a str,
    pub dependency_artifacts_digest_blake3: &'a str,
    pub host_artifacts_digest_blake3: &'a str,
    pub output_artifacts_digest_blake3: &'a str,
    pub consumed_artifact_roles_digest_blake3: &'a str,
    pub declared_output_count: u32,
    pub dependency_artifact_count: u32,
    pub host_artifact_count: u32,
    pub output_artifact_count: u32,
    pub consumed_artifact_role_count: u32,
}

/// Validate replay-evidence schema version, identity fields, BLAKE3 digests, and bounds.
pub fn validate_rust_unit_replay_evidence(evidence: &ReplayEvidenceFacts<'_>) -> Result<(), ReplayEvidenceError> {
    if evidence.schema_version != RUST_UNIT_REPLAY_EVIDENCE_SCHEMA_VERSION {
        return Err(ReplayEvidenceError::UnsupportedSchemaVersion);
    }
    let is_missing_identity =
        evidence.unit_id.is_empty() || evidence.package_id.is_empty() || evidence.blocker_class.is_empty();
    if is_missing_identity {
        return Err(ReplayEvidenceError::EmptyIdentity);
    }
    for (field, value) in [
        ("rustc_args_digest_blake3", evidence.rustc_args_digest_blake3),
        ("environment_digest_blake3", evidence.environment_digest_blake3),
        ("toolchain_policy_digest_blake3", evidence.toolchain_policy_digest_blake3),
        ("artifact_identity_digest_blake3", evidence.artifact_identity_digest_blake3),
        ("declared_outputs_digest_blake3", evidence.declared_outputs_digest_blake3),
        ("dependency_artifacts_digest_blake3", evidence.dependency_artifacts_digest_blake3),
        ("host_artifacts_digest_blake3", evidence.host_artifacts_digest_blake3),
        ("output_artifacts_digest_blake3", evidence.output_artifacts_digest_blake3),
        ("consumed_artifact_roles_digest_blake3", evidence.consumed_artifact_roles_digest_blake3),
    ] {
        if !is_lowercase_blake3_hex(value) {
            return Err(ReplayEvidenceError::InvalidBlake3Field { field });
        }
    }
    for (field, count) in [
        ("declared_output_count", evidence.declared_output_count),
        ("dependency_artifact_count", evidence.dependency_artifact_count),
        ("host_artifact_count", evidence.host_artifact_count),
        ("output_artifact_count", evidence.output_artifact_count),
        ("consumed_artifact_role_count", evidence.consumed_artifact_role_count),
    ] {
        if count > RUST_UNIT_REPLAY_EVIDENCE_MAX_BOUNDARY_ITEMS {
            return Err(ReplayEvidenceError::CountExceedsLimit { field });
        }
    }
    debug_assert_eq!(evidence.schema_version, RUST_UNIT_REPLAY_EVIDENCE_SCHEMA_VERSION);
    debug_assert!(!evidence.unit_id.is_empty());
    Ok(())
}

/// Parse bounded replay-evidence JSON and validate its extracted facts.
pub fn parse_rust_unit_replay_evidence_json<T: DeserializeOwned>(
    bytes: &[u8],
    to_facts: impl FnOnce(&T) -> ReplayEvidenceFacts<'_>,
) -> Result<T, ReplayEvidenceError> {
    let Ok(byte_len) = u32::try_from(bytes.len()) else {
        return Err(ReplayEvidenceError::JsonTooLarge {
            max_bytes: RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES,
        });
    };
    if byte_len > RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES {
        return Err(ReplayEvidenceError::JsonTooLarge {
            max_bytes: RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES,
        });
    }
    let evidence: T = serde_json::from_slice(bytes).map_err(ReplayEvidenceError::MalformedJson)?;
    let facts = to_facts(&evidence);
    validate_rust_unit_replay_evidence(&facts)?;
    debug_assert!(byte_len <= RUST_UNIT_REPLAY_EVIDENCE_MAX_JSON_BYTES);
    debug_assert_eq!(facts.schema_version, RUST_UNIT_REPLAY_EVIDENCE_SCHEMA_VERSION);
    Ok(evidence)
}

/// Serialize the accepted receipt preimage in field order and hash its
/// canonical JSON bytes. The adapter supplies its existing typed receipt with
/// `receipt_hash` cleared, never a reordered JSON object or debug rendering.
pub fn hash_legacy_receipt_preimage<T: Serialize>(preimage: &T) -> Result<String, ReceiptError> {
    let canonical_bytes = serde_json::to_vec(preimage).map_err(ReceiptError::Canonicalization)?;
    Ok(blake3::hash(&canonical_bytes).to_hex().to_string())
}
