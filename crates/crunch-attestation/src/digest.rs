use data_encoding::HEXLOWER;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::Error;

const DIGEST_BYTES_LEN: usize = 32;
const DIGEST_HEX_LEN: usize = DIGEST_BYTES_LEN * 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AttestationDigest([u8; DIGEST_BYTES_LEN]);

impl AttestationDigest {
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        let digest = blake3::hash(bytes);
        Self(*digest.as_bytes())
    }

    pub fn parse_hex(value: &str) -> Result<Self, Error> {
        validate_digest_hex(value)?;
        let decoded = HEXLOWER.decode(value.as_bytes()).map_err(|_| Error::InvalidDigestHex {
            value: value.to_string(),
        })?;
        let digest_bytes: [u8; DIGEST_BYTES_LEN] = decoded.try_into().map_err(|_| Error::InvalidDigestHex {
            value: value.to_string(),
        })?;
        Ok(Self(digest_bytes))
    }

    pub fn to_hex(self) -> String {
        HEXLOWER.encode(&self.0)
    }

    pub fn as_bytes(&self) -> &[u8; DIGEST_BYTES_LEN] {
        &self.0
    }
}

impl Serialize for AttestationDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for AttestationDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        Self::parse_hex(&value).map_err(serde::de::Error::custom)
    }
}

fn validate_digest_hex(value: &str) -> Result<(), Error> {
    if value.len() != DIGEST_HEX_LEN {
        return Err(Error::InvalidDigestHex {
            value: value.to_string(),
        });
    }

    if !value.bytes().all(is_lower_hex_byte) {
        return Err(Error::InvalidDigestHex {
            value: value.to_string(),
        });
    }

    Ok(())
}

fn is_lower_hex_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || (byte.is_ascii_lowercase() && byte <= b'f')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_round_trip_hex() {
        let digest = AttestationDigest::from_canonical_bytes(b"hello");
        let encoded = digest.to_hex();
        let decoded = AttestationDigest::parse_hex(&encoded).unwrap();
        assert_eq!(decoded, digest);
        assert_eq!(encoded.len(), DIGEST_HEX_LEN);
    }

    #[test]
    fn digest_rejects_uppercase_hex() {
        let err = AttestationDigest::parse_hex("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
            .unwrap_err();
        assert!(matches!(err, Error::InvalidDigestHex { .. }));
    }
}
