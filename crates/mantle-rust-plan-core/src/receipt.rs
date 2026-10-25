//! Canonical serialization and hashing for accepted Rust-plan receipts.

use alloc::string::String;
use alloc::string::ToString;

use serde::Serialize;

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

/// Serialize the accepted receipt preimage in field order and hash its
/// canonical JSON bytes. The adapter supplies its existing typed receipt with
/// `receipt_hash` cleared, never a reordered JSON object or debug rendering.
pub fn hash_legacy_receipt_preimage<T: Serialize>(preimage: &T) -> Result<String, ReceiptError> {
    let canonical_bytes = serde_json::to_vec(preimage).map_err(ReceiptError::Canonicalization)?;
    Ok(blake3::hash(&canonical_bytes).to_hex().to_string())
}
