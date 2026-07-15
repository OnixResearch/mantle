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

#[derive(Clone, Copy)]
pub struct NamedValue<'a> {
    pub value: &'a str,
    pub field: &'a str,
}

#[derive(Clone, Copy)]
pub struct TypedRefValidation<'a> {
    pub value: &'a str,
    pub prefix: &'a str,
    pub field: &'a str,
}

#[derive(Clone, Copy)]
pub struct BoundedValue<'a> {
    pub value: &'a str,
    pub field: &'a str,
    pub maximum_bytes: u32,
}

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

pub fn validate_blake3(input: NamedValue<'_>) -> Result<(), String> {
    if input.value.len() != BLAKE3_HEX_LENGTH {
        return Err(format!("{}-blake3-length-invalid", input.field));
    }
    if !is_lower_hex(input.value) {
        return Err(format!("{}-blake3-encoding-invalid", input.field));
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

pub fn validate_typed_ref(input: TypedRefValidation<'_>) -> Result<(), String> {
    let Some(digest) = input.value.strip_prefix(input.prefix) else {
        return Err(format!("{}-prefix-invalid", input.field));
    };
    validate_blake3(NamedValue {
        value: digest,
        field: input.field,
    })
}

pub fn validate_relative_path(input: BoundedValue<'_>) -> Result<(), String> {
    if input.value.is_empty() {
        return Err(format!("{}-empty", input.field));
    }
    validate_byte_limit(input)?;
    if input.value.starts_with('/') || input.value.ends_with('/') {
        return Err(format!("{}-not-relative-normal", input.field));
    }
    for component in input.value.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            return Err(format!("{}-not-relative-normal", input.field));
        }
    }
    debug_assert!(!input.value.starts_with('/'));
    debug_assert!(!input.value.split('/').any(|component| component == ".."));
    Ok(())
}

pub fn validate_identifier(input: BoundedValue<'_>) -> Result<(), String> {
    if input.value.is_empty() {
        return Err(format!("{}-empty", input.field));
    }
    validate_byte_limit(input)?;
    let mut characters = input.value.chars();
    let Some(first) = characters.next() else {
        return Err(format!("{}-empty", input.field));
    };
    if !first.is_ascii_lowercase() {
        return Err(format!("{}-grammar-invalid", input.field));
    }
    if characters.any(|character| {
        !(character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == '-'
            || character == '_'
            || character == '.')
    }) {
        return Err(format!("{}-grammar-invalid", input.field));
    }
    debug_assert!(u32::try_from(input.value.len()).is_ok_and(|value_bytes| value_bytes <= input.maximum_bytes));
    debug_assert!(!input.value.chars().any(char::is_control));
    Ok(())
}

pub fn validate_store_path(input: NamedValue<'_>) -> Result<(), String> {
    const NIX_STORE_PREFIX: &str = "/nix/store/";
    const NIX_STORE_HASH_LENGTH: usize = 32;

    let Some(component) = input.value.strip_prefix(NIX_STORE_PREFIX) else {
        return Err(format!("{}-not-nix-store", input.field));
    };
    let root = component.split('/').next().unwrap_or_default();
    let Some((hash, name)) = root.split_once('-') else {
        return Err(format!("{}-store-component-invalid", input.field));
    };
    if hash.len() != NIX_STORE_HASH_LENGTH || name.is_empty() {
        return Err(format!("{}-store-component-invalid", input.field));
    }
    if input.value.split('/').any(|part| part == "." || part == "..") {
        return Err(format!("{}-store-path-traversal", input.field));
    }
    debug_assert!(input.value.starts_with(NIX_STORE_PREFIX));
    debug_assert!(!name.is_empty());
    Ok(())
}

fn validate_byte_limit(input: BoundedValue<'_>) -> Result<(), String> {
    let value_bytes = u32::try_from(input.value.len()).map_err(|_| format!("{}-too-large", input.field))?;
    if value_bytes > input.maximum_bytes {
        return Err(format!("{}-too-large", input.field));
    }
    debug_assert!(value_bytes <= input.maximum_bytes);
    debug_assert!(!input.field.is_empty());
    Ok(())
}

fn is_lower_hex(value: &str) -> bool {
    value.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
}
