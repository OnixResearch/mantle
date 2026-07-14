use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

pub const BLAKE3_HEX_LENGTH: usize = 64;
pub const MAX_CANONICAL_IDENTITY_BYTES: u32 = 4_194_304;
const CANONICAL_IDENTITY_OVERFLOW_BYTES: u32 = MAX_CANONICAL_IDENTITY_BYTES.saturating_add(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigestError {
    InvalidBlake3,
    CanonicalValueTooLarge { actual_bytes: u32, maximum_bytes: u32 },
    CanonicalSerialization,
}

impl fmt::Display for DigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBlake3 => formatter.write_str("expected a 64-character lowercase BLAKE3 digest"),
            Self::CanonicalValueTooLarge {
                actual_bytes,
                maximum_bytes,
            } => write!(formatter, "canonical identity input exceeds bound: {actual_bytes} > {maximum_bytes} bytes"),
            Self::CanonicalSerialization => formatter.write_str("canonical JSON serialization failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blake3Digest(String);

impl Blake3Digest {
    pub fn parse(value: String) -> Result<Self, DigestError> {
        if !is_lower_hex(&value, BLAKE3_HEX_LENGTH) {
            return Err(DigestError::InvalidBlake3);
        }
        debug_assert_eq!(value.len(), BLAKE3_HEX_LENGTH);
        debug_assert!(value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
        Ok(Self(value))
    }

    pub fn from_slice(bytes: &[u8]) -> Self {
        let value = blake3::hash(bytes).to_hex().to_string();
        debug_assert_eq!(value.len(), BLAKE3_HEX_LENGTH);
        debug_assert!(is_lower_hex(&value, BLAKE3_HEX_LENGTH));
        Self(value)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::from_slice(&bytes)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Blake3Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for Blake3Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Blake3Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

pub(crate) fn canonical_identity<T>(value: T) -> Result<Blake3Digest, DigestError>
where T: Serialize {
    let bytes = serde_json::to_vec(&value).map_err(|_| DigestError::CanonicalSerialization)?;
    let byte_count = u32::try_from(bytes.len()).unwrap_or(CANONICAL_IDENTITY_OVERFLOW_BYTES);
    if byte_count > MAX_CANONICAL_IDENTITY_BYTES {
        return Err(DigestError::CanonicalValueTooLarge {
            actual_bytes: byte_count,
            maximum_bytes: MAX_CANONICAL_IDENTITY_BYTES,
        });
    }
    debug_assert!(!bytes.is_empty());
    debug_assert!(byte_count <= MAX_CANONICAL_IDENTITY_BYTES);
    Ok(Blake3Digest::from_bytes(bytes))
}

fn is_lower_hex(value: &str, expected_length: usize) -> bool {
    value.len() == expected_length && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub(crate) fn count_exceeds(count: usize, maximum: u32) -> bool {
    usize::try_from(maximum).is_ok_and(|maximum| count > maximum)
}
