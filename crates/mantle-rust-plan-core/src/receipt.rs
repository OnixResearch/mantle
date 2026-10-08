//! Canonical serialization and hashing for accepted Rust-plan receipts.

use alloc::string::String;
use alloc::string::ToString;

use serde::Serialize;

/// Serialize the accepted receipt preimage in field order and hash its
/// canonical JSON bytes. The adapter supplies its existing typed receipt with
/// `receipt_hash` cleared, never a reordered JSON object or debug rendering.
pub fn hash_legacy_receipt_preimage<T: Serialize>(preimage: &T) -> Result<String, String> {
    let canonical_bytes =
        serde_json::to_vec(preimage).map_err(|error| alloc::format!("canonicalizing Rust plan receipt: {error}"))?;
    Ok(blake3::hash(&canonical_bytes).to_hex().to_string())
}
