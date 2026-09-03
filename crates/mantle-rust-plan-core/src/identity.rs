use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Serialize;

const DOMAIN_SEPARATOR: u8 = 0;
const PLAN_DOMAIN: &[u8] = b"mantle.rust-plan.core.plan.v1";
const EFFECT_DOMAIN: &[u8] = b"mantle.rust-plan.core.effect.v1";
const OUTCOME_DOMAIN: &[u8] = b"mantle.rust-plan.core.outcome.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TripleSelectionInput<'a> {
    pub execution_kind: &'a str,
    pub host_triple: &'a str,
    pub target_triple: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeUnitIdentityInput {
    pub package_id: String,
    pub target_name: String,
    pub target_kind: String,
    pub mode: String,
    pub profile: String,
    pub source_algorithm: String,
    pub source_value: String,
    pub features: Vec<String>,
    pub dependencies: Vec<(String, String)>,
}

#[must_use]
pub fn classify_target_kind(kinds: &[String], crate_types: &[String]) -> Option<&'static str> {
    let is_proc_macro_type = crate_types.iter().any(|crate_type| crate_type == "proc-macro");
    let is_library_kind = kinds.iter().any(|kind| kind == "lib" || kind == "rlib");
    let classified = [
        (kinds.iter().any(|kind| kind == "custom-build"), "custom-build"),
        (
            kinds.iter().any(|kind| kind == "proc-macro") || (is_proc_macro_type && is_library_kind),
            "proc-macro",
        ),
        (is_library_kind, "lib"),
        (kinds.iter().any(|kind| kind == "bin"), "bin"),
    ]
    .into_iter()
    .find_map(|(matches, label)| matches.then_some(label));
    debug_assert!(!is_proc_macro_type || !is_library_kind || classified == Some("proc-macro"));
    debug_assert!(classified.is_none_or(|label| !label.is_empty()));
    classified
}

#[must_use]
pub fn target_kind_uses_host(kind: &str) -> bool {
    debug_assert!(!kind.is_empty());
    let is_host_kind = matches!(kind, "custom-build" | "proc-macro");
    debug_assert!(!is_host_kind || kind != "lib");
    is_host_kind
}

#[must_use]
pub fn selected_triple(input: TripleSelectionInput<'_>) -> String {
    debug_assert!(!input.execution_kind.is_empty());
    debug_assert!(!input.host_triple.is_empty());
    if input.execution_kind == "target" {
        debug_assert!(!input.target_triple.is_empty());
        return input.target_triple.to_string();
    }
    input.host_triple.to_string()
}

#[must_use]
pub fn rust_crate_name(name: &str) -> String {
    debug_assert!(!name.is_empty());
    let normalized = name.replace('-', "_");
    debug_assert!(!normalized.is_empty());
    normalized
}

#[must_use]
pub fn native_unit_identity(input: &NativeUnitIdentityInput) -> String {
    let features = input.features.join(",");
    let dependencies = input
        .dependencies
        .iter()
        .map(|(package_id, name)| format!("{package_id}:{name}"))
        .collect::<Vec<_>>()
        .join(",");
    let material = [
        input.package_id.as_str(),
        input.target_name.as_str(),
        input.target_kind.as_str(),
        input.mode.as_str(),
        input.profile.as_str(),
        input.source_algorithm.as_str(),
        input.source_value.as_str(),
        features.as_str(),
        dependencies.as_str(),
    ]
    .join("\0");
    let digest = blake3::hash(material.as_bytes()).to_hex().to_string();
    debug_assert_eq!(digest.len(), crate::BLAKE3_HEX_CHARS);
    debug_assert!(!input.package_id.is_empty());
    format!("native:{digest}:{}:{}:{}:{}", input.package_id, input.target_name, input.target_kind, input.mode)
}

pub fn receipt_preimage_identity(value: &crate::RustPlanReceiptPreimage) -> Result<String, crate::RustPlanCoreError> {
    canonical_digest(PLAN_DOMAIN, value)
}

pub(crate) fn plan_digest<T: Serialize>(value: &T) -> Result<String, crate::RustPlanCoreError> {
    canonical_digest(PLAN_DOMAIN, value)
}

pub(crate) fn effect_digest<T: Serialize>(value: &T) -> Result<String, crate::RustPlanCoreError> {
    canonical_digest(EFFECT_DOMAIN, value)
}

pub(crate) fn outcome_digest<T: Serialize>(value: &T) -> Result<String, crate::RustPlanCoreError> {
    canonical_digest(OUTCOME_DOMAIN, value)
}

fn canonical_digest<T: Serialize>(domain: &[u8], value: &T) -> Result<String, crate::RustPlanCoreError> {
    let bytes = serde_json::to_vec(value).map_err(|_| crate::RustPlanCoreError::Serialization)?;
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, domain)?;
    update_frame(&mut hasher, &bytes)?;
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), crate::BLAKE3_HEX_CHARS);
    debug_assert!(!domain.is_empty());
    Ok(digest)
}

fn update_frame(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), crate::RustPlanCoreError> {
    let length_bytes = u64::try_from(bytes.len()).map_err(|_| crate::RustPlanCoreError::Serialization)?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
    debug_assert_eq!(usize::try_from(length_bytes).ok(), Some(bytes.len()));
    debug_assert_eq!(core::mem::size_of_val(&length_bytes), core::mem::size_of::<u64>());
    Ok(())
}

pub(crate) fn lowercase_blake3(value: &str) -> bool {
    value.len() == crate::BLAKE3_HEX_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
