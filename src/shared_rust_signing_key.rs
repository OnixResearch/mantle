//! Adapter: load one shared Rust signing key from the host filesystem.
//!
//! The loader refuses symlinks, non-regular files, loose permissions, and any
//! file whose identity or contents change between the check and the read, then
//! returns the key name and its Ed25519 signing key.

use std::path::Path;

use crate::RunError;

/// Largest admitted shared Rust signing key file.
const SHARED_RUST_SIGNING_KEY_MAX_BYTES: u64 = 4_096;

/// Permission bits a shared Rust signing key must not carry.
const SHARED_RUST_SIGNING_KEY_FORBIDDEN_MODE_BITS: u32 = 0o077;

/// Secret half of an Ed25519 keypair.
const ED25519_SECRET_KEY_BYTES: usize = 32;

/// Public half of an Ed25519 keypair.
const ED25519_PUBLIC_KEY_BYTES: usize = 32;

/// Both halves of an Ed25519 keypair, secret first.
const ED25519_KEYPAIR_BYTES: usize = ED25519_SECRET_KEY_BYTES + ED25519_PUBLIC_KEY_BYTES;

pub(crate) fn load_shared_rust_signing_key(path: &Path) -> Result<(String, ed25519_dalek::SigningKey), RunError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| RunError::Internal(format!("reading shared Rust signing key metadata: {error}")))?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > SHARED_RUST_SIGNING_KEY_MAX_BYTES
    {
        return Err(RunError::Internal("shared Rust signing key must be a bounded regular file".to_string()));
    }
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    };
    if mode & SHARED_RUST_SIGNING_KEY_FORBIDDEN_MODE_BITS != 0 {
        return Err(RunError::Internal("shared Rust signing key permissions are not private".to_string()));
    }
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(|error| RunError::Internal(format!("opening shared Rust signing key: {error}")))?
    };
    let opened_metadata = file
        .metadata()
        .map_err(|error| RunError::Internal(format!("checking shared Rust signing key: {error}")))?;
    let same_file = {
        use std::os::unix::fs::MetadataExt;
        metadata.dev() == opened_metadata.dev()
            && metadata.ino() == opened_metadata.ino()
            && metadata.len() == opened_metadata.len()
    };
    if !same_file {
        return Err(RunError::Internal("shared Rust signing key changed before reading".to_string()));
    }
    let capacity = usize::try_from(opened_metadata.len())
        .map_err(|_| RunError::Internal("shared Rust signing key is too large".to_string()))?;
    let mut bytes = zeroize::Zeroizing::new(Vec::with_capacity(capacity));
    std::io::Read::read_to_end(&mut file, &mut bytes)
        .map_err(|error| RunError::Internal(format!("reading shared Rust signing key: {error}")))?;
    if bytes.len() != capacity {
        return Err(RunError::Internal("shared Rust signing key size changed while reading".to_string()));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| RunError::Internal("shared Rust signing key is not UTF-8".to_string()))?
        .trim();
    let (name, encoded) = text
        .split_once(':')
        .ok_or_else(|| RunError::Internal("shared Rust signing key is malformed".to_string()))?;
    let decoded = zeroize::Zeroizing::new(
        data_encoding::BASE64
            .decode(encoded.as_bytes())
            .map_err(|_| RunError::Internal("shared Rust signing key is malformed".to_string()))?,
    );
    if decoded.len() != ED25519_KEYPAIR_BYTES {
        return Err(RunError::Internal("shared Rust signing key has the wrong size".to_string()));
    }
    let mut secret = zeroize::Zeroizing::new([0_u8; ED25519_SECRET_KEY_BYTES]);
    secret.copy_from_slice(&decoded[..ED25519_SECRET_KEY_BYTES]);
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret);
    if signing_key.verifying_key().as_bytes() != &decoded[ED25519_SECRET_KEY_BYTES..] {
        return Err(RunError::Internal("shared Rust signing key public material does not match".to_string()));
    }
    assert!(!name.is_empty());
    assert_eq!(decoded.len(), ED25519_KEYPAIR_BYTES);
    Ok((name.to_string(), signing_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY_MODE: u32 = 0o600;
    const PUBLIC_KEY_MODE: u32 = 0o644;
    const TEST_SECRET_BYTE: u8 = 7;

    fn signing_key() -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&[TEST_SECRET_BYTE; ED25519_SECRET_KEY_BYTES])
    }

    fn key_text(signing: &ed25519_dalek::SigningKey) -> String {
        let mut combined = zeroize::Zeroizing::new([0_u8; ED25519_KEYPAIR_BYTES]);
        combined[..ED25519_SECRET_KEY_BYTES].copy_from_slice(signing.as_bytes());
        combined[ED25519_SECRET_KEY_BYTES..].copy_from_slice(signing.verifying_key().as_bytes());
        format!("shared-test:{}", data_encoding::BASE64.encode(&*combined))
    }

    fn write_key(path: &Path, text: &str, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, text).expect("write key");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("set mode");
    }

    #[test]
    fn a_private_matching_keypair_loads() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("shared-signing-key");
        let signing = signing_key();
        write_key(&path, &key_text(&signing), PRIVATE_KEY_MODE);

        let (name, loaded) = load_shared_rust_signing_key(&path).expect("private key loads");
        assert_eq!(name, "shared-test");
        assert_eq!(loaded.verifying_key(), signing.verifying_key());
    }

    #[test]
    fn a_loose_mode_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("shared-signing-key");
        write_key(&path, &key_text(&signing_key()), PUBLIC_KEY_MODE);
        let error = load_shared_rust_signing_key(&path).expect_err("loose mode is rejected");
        assert!(format!("{error}").contains("permissions are not private"));
    }

    #[test]
    fn a_symlink_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target-key");
        write_key(&target, &key_text(&signing_key()), PRIVATE_KEY_MODE);
        let link = root.path().join("linked-key");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        let error = load_shared_rust_signing_key(&link).expect_err("symlink is rejected");
        assert!(format!("{error}").contains("bounded regular file"));
    }

    #[test]
    fn an_oversized_file_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("shared-signing-key");
        write_key(
            &path,
            &"a".repeat(usize::try_from(SHARED_RUST_SIGNING_KEY_MAX_BYTES).unwrap_or(0) + 1),
            PRIVATE_KEY_MODE,
        );
        let error = load_shared_rust_signing_key(&path).expect_err("oversized key is rejected");
        assert!(format!("{error}").contains("bounded regular file"));
    }

    #[test]
    fn a_missing_file_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("absent-key");
        let error = load_shared_rust_signing_key(&path).expect_err("missing key is rejected");
        assert!(format!("{error}").contains("reading shared Rust signing key metadata"));
    }

    #[test]
    fn malformed_text_and_wrong_sizes_are_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("shared-signing-key");
        write_key(&path, "no-separator", PRIVATE_KEY_MODE);
        assert!(format!("{}", load_shared_rust_signing_key(&path).expect_err("malformed")).contains("malformed"));

        write_key(&path, "name:AAAA", PRIVATE_KEY_MODE);
        assert!(format!("{}", load_shared_rust_signing_key(&path).expect_err("short")).contains("wrong size"));

        write_key(&path, "name:%%%", PRIVATE_KEY_MODE);
        assert!(format!("{}", load_shared_rust_signing_key(&path).expect_err("bad base64")).contains("malformed"));
    }

    #[test]
    fn mismatched_public_material_is_rejected() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("shared-signing-key");
        let signing = signing_key();
        let mut combined = zeroize::Zeroizing::new([0_u8; ED25519_KEYPAIR_BYTES]);
        combined[..ED25519_SECRET_KEY_BYTES].copy_from_slice(signing.as_bytes());
        combined[ED25519_SECRET_KEY_BYTES..].copy_from_slice(
            ed25519_dalek::SigningKey::from_bytes(&[9_u8; ED25519_SECRET_KEY_BYTES]).verifying_key().as_bytes(),
        );
        write_key(&path, &format!("shared-test:{}", data_encoding::BASE64.encode(&*combined)), PRIVATE_KEY_MODE);
        let error = load_shared_rust_signing_key(&path).expect_err("mismatched material is rejected");
        assert!(format!("{error}").contains("public material does not match"));
    }
}
