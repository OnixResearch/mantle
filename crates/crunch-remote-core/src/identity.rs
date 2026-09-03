use alloc::format;
use alloc::string::String;
use alloc::string::ToString;

use crate::BLAKE3_HEX_LENGTH;
use crate::RealizationKey;
use crate::RealizationKeyError;
use crate::RealizationKeyRequest;
use crate::RemoteBuildIdentityInput;
use crate::RemoteBuildIdentitySource;
use crate::RemoteCommand;
use crate::RemoteCoreError;
use crate::RemoteEffect;
use crate::RemoteOutputExpectation;
use crate::RemoteReceiptPreimage;

const REALIZATION_KEY_SCHEMA: &str = "crunch-realization-key-v1";
const REMOTE_COMMAND_IDENTITY_CONTEXT: &str = "mantle.remote.command.v1";
const REMOTE_EFFECT_IDENTITY_CONTEXT: &str = "mantle.remote.effect.v1";
const REMOTE_RECEIPT_IDENTITY_CONTEXT: &str = "mantle.remote.receipt-preimage.v1";
const REMOTE_COORDINATOR_BUILD_KEY_LABEL: &str = "remote-coordinator-build-key";
const BITS_PER_NIBBLE: u8 = 4;
const LOW_NIBBLE_MASK: u8 = 0x0f;
const HEX_DIGIT_COUNT: usize = 16;
const HEX_DIGITS: &[u8; HEX_DIGIT_COUNT] = b"0123456789abcdef";

#[must_use]
pub fn is_blake3_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH
        && value.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
}

pub fn remote_command_identity(command: RemoteCommand) -> Result<String, RemoteCoreError> {
    canonical_json_identity(REMOTE_COMMAND_IDENTITY_CONTEXT, &command)
}

pub fn remote_effect_identity(mut effect: RemoteEffect) -> Result<String, RemoteCoreError> {
    effect.effect_id_blake3.clear();
    canonical_json_identity(REMOTE_EFFECT_IDENTITY_CONTEXT, &effect)
}

pub fn remote_receipt_preimage_identity(preimage: RemoteReceiptPreimage) -> Result<String, RemoteCoreError> {
    canonical_json_identity(REMOTE_RECEIPT_IDENTITY_CONTEXT, &preimage)
}

fn canonical_json_identity<T: serde::Serialize>(context: &str, value: &T) -> Result<String, RemoteCoreError> {
    let bytes = serde_json::to_vec(value).map_err(|error| RemoteCoreError::IdentitySerialization {
        reason: error.to_string(),
    })?;
    let mut hasher = blake3::Hasher::new_derive_key(context);
    frame(&mut hasher, &bytes)?;
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(is_blake3_hex_digest(&digest));
    Ok(digest)
}

fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), RemoteCoreError> {
    let length_bytes =
        u64::try_from(bytes.len()).map_err(|_| RemoteCoreError::invalid("remote-identity-length-overflow"))?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(bytes);
    debug_assert_eq!(length_bytes == 0, bytes.is_empty());
    debug_assert_eq!(usize::try_from(length_bytes).ok(), Some(bytes.len()));
    Ok(())
}

pub fn normalized_remote_build_key(input: RemoteBuildIdentityInput) -> Result<String, RemoteCoreError> {
    validate_build_identity(&input)?;
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_COORDINATOR_BUILD_KEY_LABEL);
    hash_labeled_str(&mut hasher, "store-prefix", &input.store_prefix);
    hash_build_source(&mut hasher, &input.source);
    hash_labeled_str(&mut hasher, "system", &input.system);
    hash_ordered_values(&mut hasher, "command-arg", &input.command_args);
    hash_ordered_map(&mut hasher, "env", &input.command_env);
    hash_ordered_values(&mut hasher, "input-ref", &input.input_refs);
    hash_ordered_values(&mut hasher, "source-input-ref", &input.source_input_refs);
    hash_expected_outputs(&mut hasher, &input.expected_outputs);
    hash_optional(&mut hasher, "failure-replay-source-bundle", input.failure_replay_source_bundle_blake3.as_deref());
    hash_optional(&mut hasher, "failure-replay-execution", input.failure_replay_execution_blake3.as_deref());
    hash_labeled_str(&mut hasher, "required-system", &input.required_system);
    hash_ordered_values(&mut hasher, "required-feature", &input.required_features);
    hash_labeled_str(&mut hasher, "required-sandbox", &input.required_sandbox_mode);
    hash_labeled_str(&mut hasher, "required-network", &input.required_network_mode);
    hash_ordered_values(&mut hasher, "semantic-accelerator-class", &input.semantic_accelerator_classes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(is_blake3_hex_digest(&digest));
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn validate_build_identity(input: &RemoteBuildIdentityInput) -> Result<(), RemoteCoreError> {
    if input.store_prefix.is_empty() || !input.store_prefix.starts_with('/') {
        return Err(RemoteCoreError::invalid("remote-build-key-store-prefix-invalid"));
    }
    if input.system.is_empty() || input.required_system.is_empty() {
        return Err(RemoteCoreError::invalid("remote-build-key-system-empty"));
    }
    if input.required_sandbox_mode.is_empty() || input.required_network_mode.is_empty() {
        return Err(RemoteCoreError::invalid("remote-build-key-policy-empty"));
    }
    debug_assert!(input.store_prefix.starts_with('/'));
    debug_assert!(!input.required_system.is_empty());
    Ok(())
}

fn hash_build_source(hasher: &mut blake3::Hasher, source: &RemoteBuildIdentitySource) {
    match source {
        RemoteBuildIdentitySource::Action {
            action_id,
            schema,
            spec_digest_blake3,
        } => {
            hash_labeled_str(hasher, "source-kind", "action");
            hash_labeled_str(hasher, "action-id", action_id);
            hash_labeled_str(hasher, "schema", schema);
            hash_labeled_str(hasher, "spec-digest", spec_digest_blake3);
        }
        RemoteBuildIdentitySource::Derivation {
            declared_drv_path,
            computed_drv_path,
            drv_name,
            drv_digest_blake3,
        } => {
            hash_labeled_str(hasher, "source-kind", "derivation");
            hash_labeled_str(hasher, "declared-drv-path", declared_drv_path);
            hash_labeled_str(hasher, "computed-drv-path", computed_drv_path);
            hash_labeled_str(hasher, "drv-name", drv_name);
            hash_labeled_str(hasher, "drv-digest", drv_digest_blake3);
        }
    }
}

fn hash_ordered_values(hasher: &mut blake3::Hasher, label: &str, values: &[String]) {
    for value in values {
        hash_labeled_str(hasher, label, value);
    }
}

fn hash_ordered_map(hasher: &mut blake3::Hasher, label: &str, values: &alloc::collections::BTreeMap<String, String>) {
    for (name, value) in values {
        hash_labeled_str(hasher, &format!("{label}-name"), name);
        hash_labeled_str(hasher, &format!("{label}-value"), value);
    }
}

fn hash_expected_outputs(hasher: &mut blake3::Hasher, outputs: &[RemoteOutputExpectation]) {
    for output in outputs {
        hash_labeled_str(hasher, "expected-output-name", &output.name);
        let logical_path = output.logical_path.as_deref().unwrap_or("<content-addressed>");
        hash_labeled_str(hasher, "expected-output-path", logical_path);
    }
}

fn hash_optional(hasher: &mut blake3::Hasher, label: &str, value: Option<&str>) {
    if let Some(value) = value {
        hash_labeled_str(hasher, label, value);
    }
}

fn hash_labeled_str(hasher: &mut blake3::Hasher, label: &str, value: impl AsRef<str>) {
    let value = value.as_ref();
    hasher.update(label.as_bytes());
    hasher.update(b":");
    hasher.update(value.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(value.as_bytes());
}

pub fn derive_realization_key(request: RealizationKeyRequest) -> Result<RealizationKey, RealizationKeyError> {
    validate_realization_request(&request)?;
    let bytes = serde_json::to_vec(&request).map_err(|error| RealizationKeyError::Serialize {
        source: error.to_string(),
    })?;
    let digest = blake3::hash(&bytes);
    Ok(RealizationKey(format!("{REALIZATION_KEY_SCHEMA}:{}", hex_lower(digest.as_bytes()))))
}

fn validate_realization_request(request: &RealizationKeyRequest) -> Result<(), RealizationKeyError> {
    validate_non_empty(("derivation.identity", &request.derivation.identity))?;
    validate_non_empty(("derivation.builder", &request.derivation.builder))?;
    validate_non_empty(("platform.system", &request.platform.system))?;
    validate_non_empty(("store.logical_prefix", &request.store.logical_prefix))?;
    validate_non_empty(("store.output_prefix", &request.store.output_prefix))?;
    validate_non_empty(("sandbox.hermeticity", &request.sandbox.hermeticity))?;
    validate_non_empty(("realizer_profile.name", &request.realizer_profile.name))?;
    validate_sorted_unique(
        request.input_closure.iter().map(|fact| fact.store_path.as_str()),
        "input_closure.store_path",
    )?;
    validate_sorted_unique(request.toolchains.iter().map(|fact| fact.name.as_str()), "toolchains.name")?;
    validate_sorted_unique(
        request.realizer_profile.capabilities.iter().map(String::as_str),
        "realizer_profile.capabilities",
    )?;
    Ok(())
}

fn validate_non_empty(input: (&'static str, &str)) -> Result<(), RealizationKeyError> {
    let (field, value) = input;
    if value.is_empty() {
        return Err(RealizationKeyError::EmptyField { field });
    }
    Ok(())
}

fn validate_sorted_unique<'a>(
    values: impl Iterator<Item = &'a str>,
    field: &'static str,
) -> Result<(), RealizationKeyError> {
    let mut previous: Option<&str> = None;
    for value in values {
        if value.is_empty() || previous.is_some_and(|prior| prior >= value) {
            return Err(RealizationKeyError::NotSortedUnique { field });
        }
        previous = Some(value);
    }
    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::new();
    for byte in bytes {
        let high = usize::from(byte >> BITS_PER_NIBBLE);
        let low = usize::from(byte & LOW_NIBBLE_MASK);
        debug_assert!(high < HEX_DIGITS.len());
        debug_assert!(low < HEX_DIGITS.len());
        output.push(char::from(HEX_DIGITS[high]));
        output.push(char::from(HEX_DIGITS[low]));
    }
    output
}
