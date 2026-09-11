//! Domain-separated BLAKE3 identities for Rust-plan facts.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// Exact lowercase hexadecimal identity length.
pub const BLAKE3_HEX_LENGTH: usize = 64;

/// Lowercase hexadecimal digits.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// A 64-character lowercase hexadecimal BLAKE3 identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blake3Digest(String);

impl Blake3Digest {
    /// Parse an exact lowercase 64-character hex identity.
    pub fn parse(value: String) -> Result<Self, DigestError> {
        if !is_lower_hex(&value) {
            return Err(DigestError::InvalidBlake3);
        }
        debug_assert_eq!(value.len(), BLAKE3_HEX_LENGTH);
        Ok(Self(value))
    }

    /// Hash bytes into an identity.
    pub fn from_slice(bytes: &[u8]) -> Self {
        let value = hex_of_hash(blake3::hash(bytes));
        debug_assert_eq!(value.len(), BLAKE3_HEX_LENGTH);
        Self(value)
    }

    /// The identity as lowercase hex.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for Blake3Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Blake3Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Blake3Digest::parse(value).map_err(serde::de::Error::custom)
    }
}

/// Digest construction failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestError {
    /// The value is not exact lowercase 64-character hex.
    InvalidBlake3,
}

impl core::fmt::Display for DigestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DigestError::InvalidBlake3 => formatter.write_str("invalid blake3 identity"),
        }
    }
}

/// Hash a canonical serialization under one explicit domain tag.
pub fn domain_digest<T: Serialize>(domain: &[u8], value: &T) -> Result<Blake3Digest, DigestError> {
    let bytes = match serde_json::to_vec(value) {
        Ok(bytes) => bytes,
        Err(_) => return Err(DigestError::InvalidBlake3),
    };
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&bytes);
    let identity = hex_of_hash(hasher.finalize());
    debug_assert_eq!(identity.len(), BLAKE3_HEX_LENGTH);
    Ok(Blake3Digest(identity))
}

fn hex_of_hash(hash: blake3::Hash) -> String {
    let mut output: Vec<u8> = Vec::with_capacity(BLAKE3_HEX_LENGTH);
    for byte in hash.as_bytes() {
        output.push(HEX_DIGITS[usize::from(byte >> 4)]);
        output.push(HEX_DIGITS[usize::from(byte & 0x0f)]);
    }
    debug_assert_eq!(output.len(), BLAKE3_HEX_LENGTH);
    match String::from_utf8(output) {
        Ok(text) => {
            debug_assert!(is_lower_hex(&text));
            text
        }
        Err(_) => String::new(),
    }
}

fn is_lower_hex(value: &str) -> bool {
    let is_lower_hexadecimal = value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    debug_assert!(is_lower_hexadecimal || !value.is_empty() || value.is_empty());
    value.len() == BLAKE3_HEX_LENGTH && is_lower_hexadecimal
}
