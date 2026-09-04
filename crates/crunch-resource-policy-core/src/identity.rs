use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Serialize;

use crate::ACTION_FAMILY_DOMAIN;
use crate::ACTION_FAMILY_SCHEMA;
use crate::ActionFamilyIdentity;
use crate::ActionFamilyInput;
use crate::BLAKE3_HEX_CHARS;
use crate::DOMAIN_SEPARATOR;
use crate::MAX_FEATURES;
use crate::MAX_ID_BYTES;
use crate::ResourcePolicyError;

pub fn derive_action_family_identity(
    mut input: ActionFamilyInput,
) -> Result<ActionFamilyIdentity, ResourcePolicyError> {
    validate_id(&input.request_kind)?;
    validate_id(&input.system)?;
    validate_id(&input.builder_class)?;
    validate_id(&input.sandbox_mode)?;
    validate_id(&input.network_mode)?;
    input.required_features = canonical_strings(input.required_features, MAX_FEATURES)?;
    input.semantic_accelerator_classes = canonical_strings(input.semantic_accelerator_classes, MAX_FEATURES)?;
    let identity_blake3 = canonical_blake3(ACTION_FAMILY_DOMAIN, &input)?;
    debug_assert!(valid_blake3(&identity_blake3));
    debug_assert!(input.required_features.windows(crate::PAIR_WINDOW_SIZE).all(|pair| pair[0] < pair[1]));
    Ok(ActionFamilyIdentity {
        schema: ACTION_FAMILY_SCHEMA.to_string(),
        identity_blake3,
        request_kind: input.request_kind,
        system: input.system,
        builder_class: input.builder_class,
        sandbox_mode: input.sandbox_mode,
        network_mode: input.network_mode,
        required_features: input.required_features,
        semantic_accelerator_classes: input.semantic_accelerator_classes,
    })
}

pub fn validate_action_family_identity(
    identity: ActionFamilyIdentity,
) -> Result<ActionFamilyIdentity, ResourcePolicyError> {
    if identity.schema != ACTION_FAMILY_SCHEMA {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    let rebuilt = derive_action_family_identity(ActionFamilyInput {
        request_kind: identity.request_kind.clone(),
        system: identity.system.clone(),
        builder_class: identity.builder_class.clone(),
        sandbox_mode: identity.sandbox_mode.clone(),
        network_mode: identity.network_mode.clone(),
        required_features: identity.required_features.clone(),
        semantic_accelerator_classes: identity.semantic_accelerator_classes.clone(),
    })?;
    if identity != rebuilt {
        return Err(ResourcePolicyError::InvalidDigest);
    }
    debug_assert!(valid_blake3(&identity.identity_blake3));
    debug_assert_eq!(identity.schema, ACTION_FAMILY_SCHEMA);
    Ok(identity)
}

pub fn canonical_selection_replay_bytes<T: Serialize>(value: T) -> Result<Vec<u8>, ResourcePolicyError> {
    let bytes = serde_json::to_vec(&value).map_err(|_| ResourcePolicyError::Canonicalization)?;
    if bytes.is_empty() {
        return Err(ResourcePolicyError::Canonicalization);
    }
    debug_assert!(!bytes.is_empty());
    debug_assert!(serde_json::from_slice::<serde_json::Value>(&bytes).is_ok());
    Ok(bytes)
}

pub(crate) fn canonical_blake3<T: Serialize>(domain: &[u8], value: &T) -> Result<String, ResourcePolicyError> {
    if domain.is_empty() {
        return Err(ResourcePolicyError::Canonicalization);
    }
    let bytes = serde_json::to_vec(value).map_err(|_| ResourcePolicyError::Canonicalization)?;
    let domain_len = u64::try_from(domain.len()).map_err(|_| ResourcePolicyError::ArithmeticOverflow)?;
    let value_len = u64::try_from(bytes.len()).map_err(|_| ResourcePolicyError::ArithmeticOverflow)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&domain_len.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(domain);
    hasher.update(&value_len.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(valid_blake3(&digest));
    debug_assert!(!domain.is_empty());
    Ok(digest)
}

pub(crate) fn validate_id(value: &str) -> Result<(), ResourcePolicyError> {
    if value.is_empty() || value.len() > MAX_ID_BYTES || value.chars().any(char::is_control) {
        return Err(ResourcePolicyError::InvalidIdentity);
    }
    debug_assert!(!value.is_empty());
    debug_assert!(value.len() <= MAX_ID_BYTES);
    Ok(())
}

pub(crate) fn validate_digest(value: &str) -> Result<(), ResourcePolicyError> {
    if !valid_blake3(value) {
        return Err(ResourcePolicyError::InvalidDigest);
    }
    debug_assert_eq!(value.len(), BLAKE3_HEX_CHARS);
    debug_assert!(value.bytes().all(|byte| byte.is_ascii_hexdigit()));
    Ok(())
}

pub(crate) fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn canonical_strings(mut values: Vec<String>, limit: u32) -> Result<Vec<String>, ResourcePolicyError> {
    let count = u32::try_from(values.len()).map_err(|_| ResourcePolicyError::InvalidBounds)?;
    if count > limit {
        return Err(ResourcePolicyError::TooManyFeatures);
    }
    for value in &values {
        validate_id(value)?;
    }
    values.sort();
    if values.windows(crate::PAIR_WINDOW_SIZE).any(|pair| pair[0] == pair[1]) {
        return Err(ResourcePolicyError::DuplicateIdentity);
    }
    debug_assert!(u32::try_from(values.len()).is_ok_and(|count| count <= limit));
    debug_assert!(values.windows(crate::PAIR_WINDOW_SIZE).all(|pair| pair[0] < pair[1]));
    Ok(values)
}

pub(crate) fn bounded_count(count: usize, limit: u32, error: ResourcePolicyError) -> Result<u32, ResourcePolicyError> {
    let count = u32::try_from(count).map_err(|_| error.clone())?;
    if count > limit {
        return Err(error);
    }
    debug_assert!(count <= limit);
    debug_assert!(limit > 0);
    Ok(count)
}
