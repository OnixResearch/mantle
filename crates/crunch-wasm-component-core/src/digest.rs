use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

pub const BLAKE3_HEX_LENGTH: usize = 64;
pub const SHA256_HEX_LENGTH: usize = 64;
pub const OCI_SHA256_PREFIX: &str = "sha256:";
pub const MAX_CANONICAL_IDENTITY_BYTES: u32 = 4_194_304;
const CANONICAL_IDENTITY_OVERFLOW_BYTES: u32 = MAX_CANONICAL_IDENTITY_BYTES.saturating_add(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigestError {
    InvalidBlake3,
    InvalidOciSha256,
    CanonicalValueTooLarge { actual_bytes: u32, maximum_bytes: u32 },
    CanonicalSerialization,
}

impl fmt::Display for DigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBlake3 => formatter.write_str("expected a 64-character lowercase BLAKE3 hex identity"),
            Self::InvalidOciSha256 => formatter.write_str("expected an OCI sha256:<64 lowercase hex> digest"),
            Self::CanonicalValueTooLarge {
                actual_bytes,
                maximum_bytes,
            } => write!(formatter, "canonical identity input exceeds bound: {actual_bytes} > {maximum_bytes} bytes"),
            Self::CanonicalSerialization => formatter.write_str("canonical JSON serialization failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blake3Identity(String);

impl Blake3Identity {
    pub fn parse(hex: String) -> Result<Self, DigestError> {
        if !is_lower_hex(&hex, BLAKE3_HEX_LENGTH) {
            return Err(DigestError::InvalidBlake3);
        }
        debug_assert_eq!(hex.len(), BLAKE3_HEX_LENGTH);
        debug_assert!(hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
        Ok(Self(hex))
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::from_slice(&bytes)
    }

    pub fn from_slice(bytes: &[u8]) -> Self {
        let hex = blake3::hash(bytes).to_hex().to_string();
        debug_assert_eq!(hex.len(), BLAKE3_HEX_LENGTH);
        debug_assert!(is_lower_hex(&hex, BLAKE3_HEX_LENGTH));
        Self(hex)
    }

    pub fn into_hex(self) -> String {
        self.0
    }
}

impl fmt::Display for Blake3Identity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for Blake3Identity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Blake3Identity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OciSha256Digest(String);

impl OciSha256Digest {
    pub fn parse(value: String) -> Result<Self, DigestError> {
        let Some(hex) = value.strip_prefix(OCI_SHA256_PREFIX) else {
            return Err(DigestError::InvalidOciSha256);
        };
        if !is_lower_hex(hex, SHA256_HEX_LENGTH) {
            return Err(DigestError::InvalidOciSha256);
        }
        debug_assert_eq!(value.len(), OCI_SHA256_PREFIX.len().saturating_add(SHA256_HEX_LENGTH));
        debug_assert!(value.starts_with(OCI_SHA256_PREFIX));
        Ok(Self(value))
    }

    pub fn into_value(self) -> String {
        self.0
    }
}

impl fmt::Display for OciSha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for OciSha256Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for OciSha256Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DigestRole {
    MantleBlake3,
    OciSha256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoleDigest {
    Mantle(Blake3Identity),
    ExternalOci(OciSha256Digest),
}

pub fn parse_role_digest(role: DigestRole, value: String) -> Result<RoleDigest, DigestError> {
    let parsed = match role {
        DigestRole::MantleBlake3 => RoleDigest::Mantle(Blake3Identity::parse(value)?),
        DigestRole::OciSha256 => RoleDigest::ExternalOci(OciSha256Digest::parse(value)?),
    };
    debug_assert!(matches!(
        (&role, &parsed),
        (DigestRole::MantleBlake3, RoleDigest::Mantle(_)) | (DigestRole::OciSha256, RoleDigest::ExternalOci(_))
    ));
    debug_assert!(!matches!(
        (&role, &parsed),
        (DigestRole::MantleBlake3, RoleDigest::ExternalOci(_)) | (DigestRole::OciSha256, RoleDigest::Mantle(_))
    ));
    Ok(parsed)
}

pub(crate) fn canonical_identity<T>(value: T) -> Result<Blake3Identity, DigestError>
where T: Serialize {
    let bytes = serde_json::to_vec(&value).map_err(|_| DigestError::CanonicalSerialization)?;
    let byte_count = match u32::try_from(bytes.len()) {
        Ok(count) => count,
        Err(_) => {
            return Err(DigestError::CanonicalValueTooLarge {
                actual_bytes: CANONICAL_IDENTITY_OVERFLOW_BYTES,
                maximum_bytes: MAX_CANONICAL_IDENTITY_BYTES,
            });
        }
    };
    if byte_count > MAX_CANONICAL_IDENTITY_BYTES {
        return Err(DigestError::CanonicalValueTooLarge {
            actual_bytes: byte_count,
            maximum_bytes: MAX_CANONICAL_IDENTITY_BYTES,
        });
    }
    debug_assert!(!bytes.is_empty());
    debug_assert!(byte_count <= MAX_CANONICAL_IDENTITY_BYTES);
    Ok(Blake3Identity::from_bytes(bytes))
}

fn is_lower_hex(value: &str, expected_length: usize) -> bool {
    value.len() == expected_length && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub(crate) fn is_count_above_bound(count: usize, maximum: u32) -> bool {
    match usize::try_from(maximum) {
        Ok(maximum_count) => count > maximum_count,
        Err(_) => false,
    }
}

pub(crate) fn is_count_within_bound(count: usize, maximum: u32) -> bool {
    !is_count_above_bound(count, maximum)
}
