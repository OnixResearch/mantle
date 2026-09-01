use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;

use crate::source_built_parity_promotion::PromotionEvidence;
use crate::source_built_parity_promotion::PromotionSummary;
use crate::source_built_parity_promotion::validate_promotion;

const BOUND_EVIDENCE_FIELD: &str = "bound_evidence";
const DETERMINISTIC_KEY: &str = "deterministic_proof";
const ACTION_PLAN_KEY: &str = "action_plan";
const ACTION_RECONCILIATION_KEY: &str = "action_reconciliation";
const TRUST_REPORT_KEY: &str = "trust_report";
const PATH_BYTES_MAX: usize = 1_024;
const BYTES_PER_KIBIBYTE: u64 = 1_024;
const KIBIBYTES_PER_MEBIBYTE: u64 = 1_024;
const EVIDENCE_MEBIBYTES_MAX: u64 = 32;
const EVIDENCE_BYTES_MAX: u64 = EVIDENCE_MEBIBYTES_MAX * KIBIBYTES_PER_MEBIBYTE * BYTES_PER_KIBIBYTE;
const BLAKE3_HEX_LENGTH: usize = 64;

struct LoadedEvidence {
    digest_blake3: String,
    value: Value,
}

pub(crate) fn validate_bound_promotion(project_root: &Path, descriptor: &Value) -> Result<PromotionSummary, String> {
    let bindings = descriptor
        .get(BOUND_EVIDENCE_FIELD)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing object field {BOUND_EVIDENCE_FIELD}"))?;
    let deterministic = load_binding(project_root, bindings, DETERMINISTIC_KEY)?;
    let action_plan = load_binding(project_root, bindings, ACTION_PLAN_KEY)?;
    let reconciliation = load_binding(project_root, bindings, ACTION_RECONCILIATION_KEY)?;
    let trust_report = load_binding(project_root, bindings, TRUST_REPORT_KEY)?;
    validate_promotion(PromotionEvidence {
        deterministic_proof: &deterministic.value,
        deterministic_file_blake3: &deterministic.digest_blake3,
        action_plan: &action_plan.value,
        action_plan_file_blake3: &action_plan.digest_blake3,
        action_reconciliation: &reconciliation.value,
        action_reconciliation_file_blake3: &reconciliation.digest_blake3,
        trust_report: &trust_report.value,
        trust_report_file_blake3: &trust_report.digest_blake3,
    })
}

fn load_binding(
    project_root: &Path,
    bindings: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<LoadedEvidence, String> {
    let binding = bindings
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("bound evidence {key} is not an object"))?;
    let relative = binding
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("bound evidence {key}.path is not a string"))?;
    let expected_digest = binding
        .get("blake3")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("bound evidence {key}.blake3 is not a string"))?;
    validate_relative_path(relative)?;
    validate_blake3(expected_digest, key)?;
    let path = project_root.join(relative);
    let bytes = read_bounded_regular_file(&path)?;
    let observed_digest = blake3::hash(&bytes).to_hex().to_string();
    if observed_digest != expected_digest {
        return Err(format!(
            "bound evidence {key} digest mismatch: expected {expected_digest}, observed {observed_digest}"
        ));
    }
    let value =
        serde_json::from_slice(&bytes).map_err(|error| format!("parse bound evidence {}: {error}", path.display()))?;
    Ok(LoadedEvidence {
        digest_blake3: observed_digest,
        value,
    })
}

fn validate_relative_path(value: &str) -> Result<PathBuf, String> {
    if value.is_empty() || value.len() > PATH_BYTES_MAX {
        return Err(format!("bound evidence path is empty or too long: {value:?}"));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!("bound evidence path is absolute: {value}"));
    }
    if !path.components().all(|component| matches!(component, Component::Normal(_))) {
        return Err(format!("bound evidence path has a forbidden component: {value}"));
    }
    Ok(path.to_path_buf())
}

fn validate_blake3(value: &str, key: &str) -> Result<(), String> {
    let valid = value.len() == BLAKE3_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid {
        Ok(())
    } else {
        Err(format!("bound evidence {key}.blake3 is not lowercase BLAKE3"))
    }
}

fn read_bounded_regular_file(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("metadata for bound evidence {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(format!("bound evidence is not a regular no-follow file: {}", path.display()));
    }
    if metadata.len() > EVIDENCE_BYTES_MAX {
        return Err(format!("bound evidence exceeds {EVIDENCE_BYTES_MAX} bytes: {}", path.display()));
    }
    fs::read(path).map_err(|error| format!("read bound evidence {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bound_evidence_paths_accept_safe_relative_files() {
        let path = validate_relative_path(".cairn/archive/proof/evidence.json").unwrap();
        assert_eq!(path, PathBuf::from(".cairn/archive/proof/evidence.json"));
        assert!(path.is_relative());
    }

    #[test]
    fn bound_evidence_paths_reject_absolute_and_parent_components() {
        let absolute = validate_relative_path("/tmp/evidence.json").unwrap_err();
        let parent = validate_relative_path(".cairn/../evidence.json").unwrap_err();
        assert!(absolute.contains("absolute"));
        assert!(parent.contains("forbidden component"));
    }
}
