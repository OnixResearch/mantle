//! This module provides tooling to parse private key (pairs) produced by Nix
//! and its
//! `nix-store --generate-binary-cache-key name path.secret path.pub` command.
//! It produces `ed25519_dalek` keys, but the `NarInfo::add_signature` function
//! is generic, allowing other signers.

use data_encoding::BASE64;
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use ed25519_dalek::SECRET_KEY_LENGTH;

use super::SignatureRef;
use super::VerifyingKey;

pub struct SigningKey<S> {
    name: String,
    signing_key: S,
}

impl<S: Clone> Clone for SigningKey<S> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            signing_key: self.signing_key.clone(),
        }
    }
}

impl<S> SigningKey<S>
where S: ed25519::signature::Signer<ed25519::Signature>
{
    /// Constructs a signing key, using a name and a signing key.
    pub fn new(name: String, signing_key: S) -> Self {
        Self { name, signing_key }
    }

    /// Signs a fingerprint using the internal signing key, returns the [SignatureRef]
    pub fn sign<'a>(&'a self, fp: &[u8]) -> SignatureRef<'a> {
        SignatureRef::new(&self.name, self.signing_key.sign(fp).to_bytes())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Parses a SigningKey / VerifyingKey from a byte slice in the format that Nix uses.
pub fn parse_keypair(input: &str) -> Result<(SigningKey<ed25519_dalek::SigningKey>, VerifyingKey), Error> {
    let (name, bytes64) = input.split_once(':').ok_or(Error::MissingSeparator)?;

    if name.is_empty() || !name.chars().all(|c| char::is_alphanumeric(c) || c == '-' || c == '.') {
        return Err(Error::InvalidName(name.to_string()));
    }

    const DECODED_BYTES_LEN: usize = SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH;
    if bytes64.len() != BASE64.encode_len(DECODED_BYTES_LEN) {
        return Err(Error::InvalidSigningKeyLen(bytes64.len()));
    }

    let mut buf = [0; DECODED_BYTES_LEN + 2]; // 64 bytes + 2 bytes padding
    let mut bytes = [0; DECODED_BYTES_LEN];
    match BASE64.decode_mut(bytes64.as_bytes(), &mut buf) {
        Ok(len) if len == DECODED_BYTES_LEN => {
            bytes.copy_from_slice(&buf[..DECODED_BYTES_LEN]);
        }
        Ok(_) => unreachable!(),
        // keeping DecodePartial gets annoying lifetime-wise
        Err(_) => {
            // Never retain or render the encoded input: it contains the secret
            // half of the keypair.
            buf.fill(0);
            bytes.fill(0);
            return Err(Error::DecodeError);
        }
    }
    buf.fill(0);

    let mut bytes_signing_key: [u8; SECRET_KEY_LENGTH] = [0u8; SECRET_KEY_LENGTH];
    bytes_signing_key.copy_from_slice(&bytes[0..SECRET_KEY_LENGTH]);
    let mut bytes_verifying_key: [u8; PUBLIC_KEY_LENGTH] = [0u8; PUBLIC_KEY_LENGTH];
    bytes_verifying_key.copy_from_slice(&bytes[SECRET_KEY_LENGTH..]);
    bytes.fill(0);

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&bytes_signing_key);
    bytes_signing_key.fill(0);
    // A keypair whose declared public half does not match the secret half is
    // malformed and must not be admitted.
    if signing_key.verifying_key().as_bytes() != &bytes_verifying_key {
        bytes_verifying_key.fill(0);
        return Err(Error::PublicKeyMismatch);
    }

    let signing_key = SigningKey::new(name.to_string(), signing_key);

    let dalek_verifying = ed25519_dalek::VerifyingKey::from_bytes(&bytes_verifying_key)
        .map_err(Error::InvalidVerifyingKey)?;
    bytes_verifying_key.fill(0);
    let verifying_key = VerifyingKey::new(name.to_string(), dalek_verifying);

    Ok((signing_key, verifying_key))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid name: {0}")]
    InvalidName(String),
    #[error("Missing separator")]
    MissingSeparator,
    #[error("Invalid signing key len: {0}")]
    InvalidSigningKeyLen(usize),
    #[error("Unable to base64-decode signing key")]
    DecodeError,
    #[error("VerifyingKey error: {0}")]
    InvalidVerifyingKey(ed25519_dalek::SignatureError),
    #[error("Public key does not match secret key")]
    PublicKeyMismatch,
}

#[cfg(test)]
mod test {
    use data_encoding::BASE64;

    use crate::narinfo::DUMMY_KEYPAIR;

    fn parse_error(input: &str) -> super::Error {
        match super::parse_keypair(input) {
            Ok(_) => panic!("malformed keypair unexpectedly parsed"),
            Err(error) => error,
        }
    }
    #[test]
    fn parse() {
        let (_signing_key, _verifying_key) = super::parse_keypair(DUMMY_KEYPAIR).expect("must succeed");
    }

    #[test]
    fn parse_fail() {
        assert!(super::parse_keypair("cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JE").is_err());
        assert!(super::parse_keypair("cache.example.com-1cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JE").is_err());
    }

    #[test]
    fn decode_error_does_not_echo_key_material() {
        let (name, encoded) = DUMMY_KEYPAIR.split_once(':').expect("dummy keypair has a separator");
        let mut chars = encoded.chars().collect::<Vec<_>>();
        let index = chars.len() / 2;
        chars[index] = '!';
        let malformed = format!("{name}:{}", chars.into_iter().collect::<String>());

        let error = parse_error(&malformed);
        assert!(matches!(error, super::Error::DecodeError));
        let rendered = format!("{error}");
        assert!(!rendered.contains(encoded), "parser error must not echo the encoded keypair: {rendered}");
        assert!(!rendered.contains(&malformed), "parser error must not echo the input: {rendered}");
    }

    #[test]
    fn public_half_must_match_secret_half() {
        let (name, encoded) = DUMMY_KEYPAIR.split_once(':').expect("dummy keypair has a separator");
        let mut decoded = BASE64.decode(encoded.as_bytes()).expect("dummy keypair decodes");
        decoded[0] ^= 1;
        let mismatched = format!("{name}:{}", BASE64.encode(&decoded));

        let error = parse_error(&mismatched);
        assert!(matches!(error, super::Error::PublicKeyMismatch));
        assert!(!format!("{error}").contains(&BASE64.encode(&decoded)));
    }
}
