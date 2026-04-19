//! Signing and verification of PathInfo.
//!
//! Pure functions for signing a PathInfo with an ed25519 key
//! and verifying signatures against a set of trusted public keys.
//! No I/O, no async — called from the imperative shell in
//! `persist_and_export_output` and `check_cache`.

use nix_compat::narinfo::Signature;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint;
use nix_compat::store_path::StorePathRef;
use snix_store::path_info::PathInfo;

// ── Signing ─────────────────────────────────────────────────

/// Sign a PathInfo, appending exactly one signature.
///
/// If the PathInfo already has a signature from the same key name,
/// it is replaced rather than duplicated.
///
/// Returns the signature string for logging.
pub fn sign_pathinfo(path_info: &mut PathInfo, signing_key: &SigningKey<ed25519_dalek::SigningKey>) -> String {
    let fp = compute_fingerprint(path_info);

    let sig_ref = signing_key.sign(fp.as_bytes());
    let sig_owned: Signature<String> = sig_ref.to_owned();
    let sig_display = sig_owned.to_string();

    // Replace existing sig from same key name, or append.
    let key_name = signing_key.name();
    if let Some(pos) = path_info.signatures.iter().position(|s| s.name().as_str() == key_name) {
        path_info.signatures[pos] = sig_owned;
    } else {
        path_info.signatures.push(sig_owned);
    }

    debug_assert!(!path_info.signatures.is_empty(), "sign_pathinfo must produce at least one signature");
    debug_assert!(
        path_info.signatures.iter().filter(|s| s.name().as_str() == key_name).count() == 1,
        "must have exactly one signature per key name"
    );

    sig_display
}

// ── Verification ────────────────────────────────────────────

/// Result of verifying a PathInfo's signatures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyResult {
    /// How many signatures matched a trusted key.
    pub trusted_count: u32,
    /// Key names that signed but are NOT in the trusted set.
    pub untrusted_names: Vec<String>,
    /// Total signature count on the PathInfo.
    pub total_sigs: u32,
}

impl VerifyResult {
    /// At least one signature was verified by a trusted key.
    pub fn is_trusted(&self) -> bool {
        self.trusted_count > 0
    }
}

/// Verify the signatures on a PathInfo against a set of trusted keys.
///
/// Returns a [VerifyResult] summarising how many signatures matched.
/// The caller decides policy (e.g., require `is_trusted()` for cache hits).
pub fn verify_pathinfo_signatures(path_info: &PathInfo, trusted_keys: &[VerifyingKey]) -> VerifyResult {
    debug_assert!(!trusted_keys.is_empty(), "verify_pathinfo_signatures called with zero trusted keys");

    if path_info.signatures.is_empty() {
        return VerifyResult {
            trusted_count: 0,
            untrusted_names: vec![],
            total_sigs: 0,
        };
    }

    let fp = compute_fingerprint(path_info);

    let mut trusted_count: u32 = 0;
    let mut untrusted_names: Vec<String> = Vec::with_capacity(path_info.signatures.len());

    for sig in &path_info.signatures {
        let sig_ref = sig.as_ref();
        let is_trusted = trusted_keys.iter().any(|k| k.verify(&fp, &sig_ref));

        if is_trusted {
            trusted_count = trusted_count.saturating_add(1);
        } else {
            untrusted_names.push(sig.name().clone());
        }
    }

    let total_sigs = u32::try_from(path_info.signatures.len()).unwrap_or(u32::MAX);

    VerifyResult {
        trusted_count,
        untrusted_names,
        total_sigs,
    }
}

// ── Key management ──────────────────────────────────────────

/// The cache.nixos.org-1 public key, always included in the default
/// trusted key set.
pub const CACHE_NIXOS_ORG_PUBKEY: &str = "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=";

/// A paired signing key + verifying key, ready to use.
///
/// nix_compat's `SigningKey` doesn't expose its inner ed25519 key,
/// so we keep the VerifyingKey from parse_keypair alongside it.
#[derive(Clone)]
pub struct KeyPair {
    pub signing_key: SigningKey<ed25519_dalek::SigningKey>,
    pub verifying_key: VerifyingKey,
}

/// Build the default trusted key set from a KeyPair and optional
/// user-provided trusted keys.
///
/// If `user_keys` is `Some(...)`, it completely replaces the defaults
/// (except the local key is always prepended). If `None`, the default
/// set (local + cache.nixos.org-1) is used.
pub fn build_trusted_keys(local: &KeyPair, user_keys: Option<&[VerifyingKey]>) -> Vec<VerifyingKey> {
    match user_keys {
        Some(explicit) => {
            let mut keys = Vec::with_capacity(explicit.len().saturating_add(1));
            keys.push(local.verifying_key.clone());
            for k in explicit {
                // Skip only exact duplicates. Different key material may
                // legitimately reuse the same display name across stores.
                if !keys.iter().any(|existing| existing == k) {
                    keys.push(k.clone());
                }
            }
            keys
        }
        None => {
            let mut keys = Vec::with_capacity(2);
            keys.push(local.verifying_key.clone());
            if let Ok(k) = VerifyingKey::parse(CACHE_NIXOS_ORG_PUBKEY)
                && k.name() != local.verifying_key.name()
            {
                keys.push(k);
            }
            keys
        }
    }
}

/// Generate a new ed25519 keypair and return it as a KeyPair along with
/// the Nix-format string for persisting to disk.
///
/// The key name is `crunch-<hostname>-1`.
pub fn generate_keypair() -> (KeyPair, String) {
    use data_encoding::BASE64;
    use ed25519_dalek::PUBLIC_KEY_LENGTH;
    use ed25519_dalek::SECRET_KEY_LENGTH;
    use rand::RngCore;

    let hostname = gethostname::gethostname().to_string_lossy().into_owned();
    let name = format!("crunch-{hostname}-1");

    // Generate 32 random bytes and construct the signing key.
    let mut secret_bytes = [0u8; SECRET_KEY_LENGTH];
    rand::thread_rng().fill_bytes(&mut secret_bytes);
    let dalek_signing = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
    let dalek_verifying = dalek_signing.verifying_key();

    // Nix format: name:base64(secret_32 ++ public_32)
    // SECRET_KEY_LENGTH (32) + PUBLIC_KEY_LENGTH (32)
    let mut combined = [0u8; 64];
    combined[..SECRET_KEY_LENGTH].copy_from_slice(dalek_signing.as_bytes());
    combined[SECRET_KEY_LENGTH..].copy_from_slice(dalek_verifying.as_bytes());
    let line = format!("{}:{}", name, BASE64.encode(&combined));

    let signing_key = SigningKey::new(name.clone(), dalek_signing);
    let verifying_key = VerifyingKey::new(name, dalek_verifying);

    (
        KeyPair {
            signing_key,
            verifying_key,
        },
        line,
    )
}

/// Load a keypair from a Nix-format string (e.g. from a file).
pub fn load_keypair(contents: &str) -> Result<KeyPair, String> {
    let line = contents.trim();
    let (signing_key, verifying_key) =
        nix_compat::narinfo::parse_keypair(line).map_err(|e| format!("invalid signing key: {e}"))?;
    Ok(KeyPair {
        signing_key,
        verifying_key,
    })
}

// ── Internal helpers ────────────────────────────────────────

/// Compute the narinfo fingerprint for a PathInfo.
fn compute_fingerprint(path_info: &PathInfo) -> String {
    let sp_ref: StorePathRef = path_info.store_path.as_ref();
    let refs: Vec<StorePathRef> = path_info.references.iter().map(|r| r.as_ref()).collect();

    fingerprint(&sp_ref, &path_info.nar_sha256, path_info.nar_size, refs.iter())
}

#[cfg(test)]
mod tests {
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;

    use super::*;

    const DUMMY_KEYPAIR_STR: &str =
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

    fn test_keypair() -> KeyPair {
        load_keypair(DUMMY_KEYPAIR_STR).unwrap()
    }

    fn dummy_pathinfo() -> PathInfo {
        let sp = StorePath::from_bytes(b"00000000000000000000000000000000-test-1.0").unwrap();
        PathInfo {
            store_path: sp,
            node: Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("somewhere").unwrap(),
            },
            references: vec![],
            nar_size: 1024,
            nar_sha256: [0xAB; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    // ── sign_pathinfo tests ─────────────────────────────────

    #[test]
    fn sign_produces_one_signature() {
        let kp = test_keypair();
        let mut pi = dummy_pathinfo();
        assert!(pi.signatures.is_empty());

        sign_pathinfo(&mut pi, &kp.signing_key);
        assert_eq!(pi.signatures.len(), 1);
        assert_eq!(pi.signatures[0].name().as_str(), "cache.example.com-1");
    }

    #[test]
    fn sign_round_trip_verifies() {
        let kp = test_keypair();
        let mut pi = dummy_pathinfo();

        sign_pathinfo(&mut pi, &kp.signing_key);
        let result = verify_pathinfo_signatures(&pi, std::slice::from_ref(&kp.verifying_key));
        assert!(result.is_trusted());
        assert_eq!(result.trusted_count, 1);
        assert!(result.untrusted_names.is_empty());
    }

    #[test]
    fn sign_replaces_same_key_name() {
        let kp = test_keypair();
        let mut pi = dummy_pathinfo();

        sign_pathinfo(&mut pi, &kp.signing_key);
        assert_eq!(pi.signatures.len(), 1);

        // Sign again with the same key — should replace, not duplicate.
        sign_pathinfo(&mut pi, &kp.signing_key);
        assert_eq!(pi.signatures.len(), 1);
    }

    #[test]
    fn fingerprint_matches_nix_compat() {
        let pi = dummy_pathinfo();
        let fp = compute_fingerprint(&pi);

        // Must start with "1;" (version) and contain the store path.
        assert!(fp.starts_with("1;"), "fingerprint: {fp}");
        assert!(fp.contains("/nix/store/"), "fingerprint: {fp}");
    }

    // ── verify_pathinfo_signatures tests ────────────────────

    #[test]
    fn verify_no_signatures_is_untrusted() {
        let kp = test_keypair();
        let pi = dummy_pathinfo();
        let result = verify_pathinfo_signatures(&pi, std::slice::from_ref(&kp.verifying_key));
        assert!(!result.is_trusted());
        assert_eq!(result.total_sigs, 0);
    }

    #[test]
    fn verify_wrong_key_is_untrusted() {
        let kp1 = test_keypair();
        let (kp2, _) = generate_keypair();

        let mut pi = dummy_pathinfo();
        sign_pathinfo(&mut pi, &kp1.signing_key);

        // Verify with a different key — should fail.
        let result = verify_pathinfo_signatures(&pi, std::slice::from_ref(&kp2.verifying_key));
        assert!(!result.is_trusted());
        assert_eq!(result.trusted_count, 0);
        assert_eq!(result.untrusted_names.len(), 1);
        assert_eq!(result.untrusted_names[0], "cache.example.com-1");
    }

    #[test]
    fn verify_with_references() {
        let kp = test_keypair();
        let ref_sp = StorePath::from_bytes(b"11111111111111111111111111111111-dep-1.0").unwrap();
        let mut pi = dummy_pathinfo();
        pi.references.push(ref_sp);

        sign_pathinfo(&mut pi, &kp.signing_key);
        let result = verify_pathinfo_signatures(&pi, std::slice::from_ref(&kp.verifying_key));
        assert!(result.is_trusted());
    }

    // ── key management tests ────────────────────────────────

    #[test]
    fn generate_keypair_produces_valid_pair() {
        let (kp, line) = generate_keypair();
        assert!(kp.signing_key.name().starts_with("crunch-"));
        assert!(kp.verifying_key.name().starts_with("crunch-"));
        assert!(line.contains(':'));

        // Round-trip: parse the line back.
        let kp2 = load_keypair(&line).unwrap();
        assert_eq!(kp2.verifying_key.name(), kp.verifying_key.name());
    }

    #[test]
    fn generate_keypair_sign_and_verify() {
        let (kp, _) = generate_keypair();
        let mut pi = dummy_pathinfo();

        sign_pathinfo(&mut pi, &kp.signing_key);
        let result = verify_pathinfo_signatures(&pi, std::slice::from_ref(&kp.verifying_key));
        assert!(result.is_trusted());
    }

    #[test]
    fn load_keypair_rejects_garbage() {
        assert!(load_keypair("not-a-key").is_err());
        assert!(load_keypair("").is_err());
        assert!(load_keypair("name:").is_err());
    }

    // ── build_trusted_keys tests ────────────────────────────

    #[test]
    fn default_trusted_includes_local_and_nixos_cache() {
        let kp = test_keypair();
        let keys = build_trusted_keys(&kp, None);
        assert!(keys.len() >= 2, "expected local + cache.nixos.org-1");
        assert!(keys.iter().any(|k| k.name() == "cache.example.com-1"));
        assert!(keys.iter().any(|k| k.name() == "cache.nixos.org-1"));
    }

    #[test]
    fn user_keys_override_defaults() {
        let kp = test_keypair();
        let custom = VerifyingKey::parse("my-cache-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=").unwrap();

        let keys = build_trusted_keys(&kp, Some(&[custom]));
        // Should have local key + my-cache-1, but NOT cache.nixos.org-1.
        assert!(keys.iter().any(|k| k.name() == "cache.example.com-1"));
        assert!(keys.iter().any(|k| k.name() == "my-cache-1"));
        assert!(!keys.iter().any(|k| k.name() == "cache.nixos.org-1"));
    }

    #[test]
    fn user_keys_dedup_local() {
        let kp = test_keypair();
        // User lists the local key explicitly — should not duplicate.
        let keys = build_trusted_keys(&kp, Some(std::slice::from_ref(&kp.verifying_key)));
        let local_count = keys.iter().filter(|k| k.name() == "cache.example.com-1").count();
        assert_eq!(local_count, 1, "local key must appear exactly once");
    }

    #[test]
    fn user_keys_keep_same_name_with_different_material() {
        let kp = test_keypair();
        let distinct_same_name =
            VerifyingKey::parse("cache.example.com-1:tLAEn+EeaBUJYqEpTd2yeerr7Ic6+0vWe+aXL/vYUpE=").unwrap();

        let keys = build_trusted_keys(&kp, Some(std::slice::from_ref(&distinct_same_name)));
        let local_count = keys.iter().filter(|k| k.name() == "cache.example.com-1").count();
        assert_eq!(local_count, 2, "same-name keys with different bytes must both be kept");
        assert!(keys.iter().any(|k| k == &kp.verifying_key));
        assert!(keys.iter().any(|k| k == &distinct_same_name));
    }
}
