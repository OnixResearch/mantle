//! Adapter: admit trust material from operator input.
//!
//! Two trust surfaces are admitted here: Nix narinfo verifying keys for cache
//! and store trust, and shared Rust cache result keys with the identity of the
//! trust policy they form. Malformed or wrongly sized material fails closed
//! with the offending input named.

use crate::RunError;

/// Public half of an Ed25519 keypair.
const ED25519_PUBLIC_KEY_BYTES: usize = 32;

pub(crate) fn parse_shared_rust_result_keys(
    values: &[String],
) -> Result<Vec<crunch_rust_cache_core::shared::TrustedRustResultKey>, RunError> {
    let mut keys = Vec::with_capacity(values.len());
    for value in values {
        let (name, encoded) = value
            .split_once(':')
            .ok_or_else(|| RunError::Internal("shared Rust trusted key is malformed".to_string()))?;
        let bytes = data_encoding::BASE64
            .decode(encoded.as_bytes())
            .map_err(|_| RunError::Internal("shared Rust trusted key is malformed".to_string()))?;
        if bytes.len() != ED25519_PUBLIC_KEY_BYTES {
            return Err(RunError::Internal("shared Rust trusted key has the wrong size".to_string()));
        }
        keys.push(crunch_rust_cache_core::shared::TrustedRustResultKey {
            signer_name: name.to_string(),
            verifier_key_hex: data_encoding::HEXLOWER.encode(&bytes),
        });
    }
    keys.sort();
    keys.dedup();
    assert!(keys.len() <= values.len());
    assert!(keys.iter().all(|key| !key.signer_name.is_empty()));
    Ok(keys)
}

pub(crate) fn shared_rust_trust_policy_id(
    producer_policies: &[String],
    keys: &[crunch_rust_cache_core::shared::TrustedRustResultKey],
) -> Result<String, RunError> {
    let canonical = serde_json::to_vec(&(producer_policies, keys))
        .map_err(|error| RunError::Internal(format!("encoding shared Rust trust policy: {error}")))?;
    let digest = blake3::hash(&canonical).to_hex();
    let policy_id = format!("mantle-shared-rust-trust-{digest}");
    assert!(!canonical.is_empty());
    assert_eq!(digest.len(), crunch_rust_cache_core::BLAKE3_HEX_CHARS);
    Ok(policy_id)
}

pub(crate) fn parse_trusted_public_keys(
    keys: &[String],
) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if keys.is_empty() {
        return Ok(None);
    }
    let mut parsed = Vec::with_capacity(keys.len());
    for key_str in keys {
        parsed.push(
            nix_compat::narinfo::VerifyingKey::parse(key_str)
                .map_err(|e| RunError::Internal(format!("invalid trusted public key '{key_str}': {e}")))?,
        );
    }
    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn narinfo_key(name: &str) -> String {
        let signing = ed25519_dalek::SigningKey::from_bytes(&[7_u8; ED25519_PUBLIC_KEY_BYTES]);
        format!("{name}:{}", data_encoding::BASE64.encode(signing.verifying_key().as_bytes()))
    }

    fn shared_key(name: &str, byte: u8) -> String {
        format!("{name}:{}", data_encoding::BASE64.encode(&[byte; ED25519_PUBLIC_KEY_BYTES]))
    }

    #[test]
    fn an_empty_public_key_list_is_absent() {
        assert!(parse_trusted_public_keys(&[]).expect("empty list").is_none());
        let parsed = parse_trusted_public_keys(&[narinfo_key("cache-1")]).expect("valid key");
        assert_eq!(parsed.map(|keys| keys.len()), Some(1));
    }

    #[test]
    fn a_malformed_public_key_names_the_key() {
        let error = parse_trusted_public_keys(&[String::from("not-a-key")]).expect_err("malformed key is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("invalid trusted public key"));
        assert!(rendered.contains("not-a-key"));
    }

    #[test]
    fn shared_result_keys_are_parsed_sorted_and_deduped() {
        let values = vec![shared_key("zeta", 2), shared_key("alpha", 1), shared_key("alpha", 1)];
        let keys = parse_shared_rust_result_keys(&values).expect("shared keys parse");
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].signer_name, "alpha");
        assert_eq!(keys[0].verifier_key_hex, data_encoding::HEXLOWER.encode(&[1_u8; ED25519_PUBLIC_KEY_BYTES]));
        assert_eq!(keys[1].signer_name, "zeta");
    }

    #[test]
    fn malformed_shared_keys_are_rejected() {
        let no_separator = parse_shared_rust_result_keys(&[String::from("missing-colon")])
            .expect_err("a key without a separator is rejected");
        assert!(format!("{no_separator}").contains("malformed"));

        let wrong_size = parse_shared_rust_result_keys(&[format!("name:{}", data_encoding::BASE64.encode(&[1_u8; 4]))])
            .expect_err("a short key is rejected");
        assert!(format!("{wrong_size}").contains("wrong size"));

        let bad_base64 =
            parse_shared_rust_result_keys(&[String::from("name:%%%")]).expect_err("bad base64 is rejected");
        assert!(format!("{bad_base64}").contains("malformed"));
    }

    #[test]
    fn the_trust_policy_identity_is_deterministic_and_key_sensitive() {
        let keys = parse_shared_rust_result_keys(&[shared_key("alpha", 1)]).expect("shared keys parse");
        let first = shared_rust_trust_policy_id(&[String::from("policy-a")], &keys).expect("policy id");
        let second = shared_rust_trust_policy_id(&[String::from("policy-a")], &keys).expect("policy id");
        assert_eq!(first, second);
        assert!(first.starts_with("mantle-shared-rust-trust-"));
        let other_keys = parse_shared_rust_result_keys(&[shared_key("alpha", 2)]).expect("shared keys parse");
        let changed = shared_rust_trust_policy_id(&[String::from("policy-a")], &other_keys).expect("policy id");
        assert_ne!(first, changed);
    }
}
