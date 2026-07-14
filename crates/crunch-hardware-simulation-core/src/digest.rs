use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use serde::Serialize;

pub const BLAKE3_HEX_LENGTH: usize = 64;
pub const GIT_REVISION_HEX_LENGTH: usize = 40;
pub const ACTION_REF_PREFIX: &str = "mantle-action://blake3/";
pub const OBJECT_REF_PREFIX: &str = "mantle-object://blake3/";
pub const PROFILE_REF_PREFIX: &str = "mantle-hardware-profile://blake3/";
pub const COHORT_REF_PREFIX: &str = "mantle-hardware-cohort://blake3/";
pub const LOG_REF_PREFIX: &str = "mantle-hardware-log://blake3/";
pub const SIMULATOR_REF_PREFIX: &str = "mantle-hardware-simulator://blake3/";
pub const EVIDENCE_REF_PREFIX: &str = "mantle-hardware-evidence://blake3/";

const DOMAIN_SEPARATOR: u8 = 0;

pub fn domain_digest<T: Serialize>(domain: &[u8], value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("canonical-json:{error}"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(is_lower_hex(&digest));
    Ok(digest)
}

pub fn digest_ref<T: Serialize>(prefix: &str, domain: &[u8], value: &T) -> Result<String, String> {
    let digest = domain_digest(domain, value)?;
    let reference = format!("{prefix}{digest}");
    debug_assert!(reference.starts_with(prefix));
    debug_assert_eq!(reference.len(), prefix.len().saturating_add(BLAKE3_HEX_LENGTH));
    Ok(reference)
}

pub fn validate_blake3(value: &str, field: &str) -> Result<(), String> {
    if value.len() != BLAKE3_HEX_LENGTH {
        return Err(format!("{field}-blake3-length-invalid"));
    }
    if !is_lower_hex(value) {
        return Err(format!("{field}-blake3-encoding-invalid"));
    }
    Ok(())
}

pub fn validate_git_revision(value: &str) -> Result<(), String> {
    if value.len() != GIT_REVISION_HEX_LENGTH {
        return Err(String::from("source-revision-length-invalid"));
    }
    if !is_lower_hex(value) {
        return Err(String::from("source-revision-encoding-invalid"));
    }
    Ok(())
}

pub fn validate_typed_ref(value: &str, prefix: &str, field: &str) -> Result<(), String> {
    let Some(digest) = value.strip_prefix(prefix) else {
        return Err(format!("{field}-prefix-invalid"));
    };
    validate_blake3(digest, field)
}

pub fn validate_relative_path(value: &str, field: &str, maximum_bytes: u32) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field}-empty"));
    }
    if value.len() > usize::try_from(maximum_bytes).unwrap_or(usize::MAX) {
        return Err(format!("{field}-too-large"));
    }
    if value.starts_with('/') || value.ends_with('/') {
        return Err(format!("{field}-not-relative-normal"));
    }
    for component in value.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            return Err(format!("{field}-not-relative-normal"));
        }
    }
    debug_assert!(!value.starts_with('/'));
    debug_assert!(!value.split('/').any(|component| component == ".."));
    Ok(())
}

pub fn validate_identifier(value: &str, field: &str, maximum_bytes: u32) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field}-empty"));
    }
    if value.len() > usize::try_from(maximum_bytes).unwrap_or(usize::MAX) {
        return Err(format!("{field}-too-large"));
    }
    let mut characters = value.chars();
    let first = characters.next().expect("non-empty identifier has first character");
    if !first.is_ascii_lowercase() {
        return Err(format!("{field}-grammar-invalid"));
    }
    if characters.any(|character| {
        !(character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '-'
            || character == '_'
            || character == '.')
    }) {
        return Err(format!("{field}-grammar-invalid"));
    }
    debug_assert!(value.len() <= usize::try_from(maximum_bytes).unwrap_or(usize::MAX));
    debug_assert!(!value.chars().any(char::is_control));
    Ok(())
}

pub fn validate_store_path(value: &str, field: &str) -> Result<(), String> {
    const NIX_STORE_PREFIX: &str = "/nix/store/";
    const NIX_STORE_HASH_LENGTH: usize = 32;

    let Some(component) = value.strip_prefix(NIX_STORE_PREFIX) else {
        return Err(format!("{field}-not-nix-store"));
    };
    let root = component.split('/').next().unwrap_or_default();
    let Some((hash, name)) = root.split_once('-') else {
        return Err(format!("{field}-store-component-invalid"));
    };
    if hash.len() != NIX_STORE_HASH_LENGTH || name.is_empty() {
        return Err(format!("{field}-store-component-invalid"));
    }
    if value.split('/').any(|part| part == "." || part == "..") {
        return Err(format!("{field}-store-path-traversal"));
    }
    debug_assert!(value.starts_with(NIX_STORE_PREFIX));
    debug_assert!(!name.is_empty());
    Ok(())
}

fn is_lower_hex(value: &str) -> bool {
    value.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
}
