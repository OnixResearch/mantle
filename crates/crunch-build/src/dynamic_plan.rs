//! Native dynamic build-plan ABI.
//!
//! This module is the pure Rust core for `mantle-plan-v1` decoding and
//! scalar grammar checks. It deliberately performs no store I/O, no worker
//! mutation, and no scheduler registration.

use std::collections::BTreeMap;

use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

const BYTES_PER_KIB: u64 = 1024;
const KIB_PER_MIB: u64 = 1024;
pub const MAX_DYNAMIC_PLAN_MIB: u64 = 4;
pub const MAX_DYNAMIC_PLAN_BYTES: u64 = MAX_DYNAMIC_PLAN_MIB * KIB_PER_MIB * BYTES_PER_KIB;
pub const MAX_DYNAMIC_PLAN_UNITS: u32 = 4096;
pub const MAX_DYNAMIC_PLAN_DEPENDENCIES_PER_UNIT: u32 = 256;
pub const MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT: u32 = 16;
pub const MAX_DYNAMIC_PLAN_ENV_ENTRIES_PER_UNIT: u32 = 512;
pub const MAX_DYNAMIC_PLAN_STRING_BYTES: u32 = 16_384;
pub const MAX_DYNAMIC_PLAN_NESTING_DEPTH: u32 = 16;
pub const MAX_DYNAMIC_PLAN_ID_BYTES: u32 = 128;
pub const MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES: u32 = 64;
pub const BLAKE3_HEX_BYTES: u32 = 64;
pub const MANTLE_PLAN_V1_SCHEMA: &str = "mantle-plan-v1";
pub const STORE_PATH_KIND: &str = "store_path";
pub const SOURCE_KIND: &str = "source";
pub const UNIT_OUTPUT_KIND: &str = "unit_output";
pub const INHERIT_POLICY_VALUE: &str = "inherit";
pub const NO_HOST_PATHS_POLICY_VALUE: &str = "none";
pub const NATIVE_SANDBOX_VALUE: &str = "native";

pub type UnitId = String;
pub type SourceId = String;
pub type StorePathString = String;
pub type Blake3Hex = String;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DynamicPlanError {
    #[error("dynamic plan exceeds byte limit: {actual_bytes} bytes > {max_bytes} bytes")]
    PlanTooLarge { actual_bytes: u64, max_bytes: u64 },

    #[error("decoding mantle-plan-v1 JSON: {message}")]
    JsonDecode { message: String },

    #[error("required nullable field missing: {field}")]
    MissingNullableField { field: &'static str },

    #[error("invalid {field} `{value}`: {reason}")]
    InvalidScalar {
        field: &'static str,
        value: String,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicPlanV1 {
    pub schema: String,
    pub producer: PlanProducer,
    pub sources: Vec<DeclaredSourceInput>,
    pub units: Vec<DynamicUnit>,
    pub roots: Vec<UnitId>,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanProducer {
    pub logical_name: String,
    pub goal_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicUnit {
    pub id: UnitId,
    pub derivation: DynamicDerivation,
    pub requested_outputs: Vec<String>,
    pub policy: DynamicUnitPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicUnitPolicy {
    pub sandbox: String,
    pub substitutions: String,
    pub store_prefix: String,
    pub host_paths: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicDerivation {
    pub name: String,
    pub builder: StorePathString,
    pub system: String,
    pub args: Vec<String>,
    pub outputs: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub inputs: Vec<DynamicInput>,
    pub fixed_output: Option<FixedOutputSpec>,
    pub addressing_mode: AddressingMode,
    pub sandbox: SandboxMode,
    pub dynamic_plan_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredSourceInput {
    pub id: SourceId,
    pub path: StorePathString,
    pub nar_blake3: Option<Blake3Hex>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedOutputSpec {
    pub mode: FixedOutputMode,
    pub algo: FixedOutputHashAlgo,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddressingMode {
    ContentAddressed,
    InputAddressed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxMode {
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixedOutputMode {
    Flat,
    Recursive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixedOutputHashAlgo {
    Sha256,
    Sha512,
    Sha1,
    Md5,
    Blake3,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DynamicInput {
    StorePath { path: StorePathString },
    Source { source: SourceId },
    UnitOutput { unit: UnitId, output: String },
}

pub fn decode_plan_v1(bytes: &[u8]) -> Result<DynamicPlanV1, DynamicPlanError> {
    let actual_bytes = len_as_u64(bytes.len());
    if actual_bytes > MAX_DYNAMIC_PLAN_BYTES {
        return Err(DynamicPlanError::PlanTooLarge {
            actual_bytes,
            max_bytes: MAX_DYNAMIC_PLAN_BYTES,
        });
    }

    let value = decode_plan_json_value(bytes)?;
    require_nullable_fields_present(&value)?;
    serde_json::from_value(value).map_err(|err| DynamicPlanError::JsonDecode {
        message: err.to_string(),
    })
}

fn decode_plan_json_value(bytes: &[u8]) -> Result<Value, DynamicPlanError> {
    serde_json::from_slice(bytes).map_err(|err| DynamicPlanError::JsonDecode {
        message: err.to_string(),
    })
}

fn require_nullable_fields_present(value: &Value) -> Result<(), DynamicPlanError> {
    if let Some(producer) = value.get("producer").and_then(Value::as_object) {
        require_json_key(producer, "goal_hint", "producer.goal_hint")?;
    }
    if let Some(sources) = value.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(source) = source.as_object() {
                require_json_key(source, "nar_blake3", "sources[].nar_blake3")?;
            }
        }
    }
    if let Some(units) = value.get("units").and_then(Value::as_array) {
        for unit in units {
            let Some(derivation) = unit.get("derivation").and_then(Value::as_object) else {
                continue;
            };
            require_json_key(derivation, "fixed_output", "units[].derivation.fixed_output")?;
        }
    }
    Ok(())
}

fn require_json_key(
    object: &serde_json::Map<String, Value>,
    key: &'static str,
    field: &'static str,
) -> Result<(), DynamicPlanError> {
    if object.contains_key(key) {
        Ok(())
    } else {
        Err(DynamicPlanError::MissingNullableField { field })
    }
}

pub fn validate_unit_id(value: &str) -> Result<(), DynamicPlanError> {
    validate_id_like("unit id", value, MAX_DYNAMIC_PLAN_ID_BYTES)
}

pub fn validate_source_id(value: &str) -> Result<(), DynamicPlanError> {
    validate_id_like("source id", value, MAX_DYNAMIC_PLAN_ID_BYTES)
}

pub fn validate_output_name(value: &str) -> Result<(), DynamicPlanError> {
    let byte_len = len_as_u64(value.len());
    if value.is_empty() {
        return invalid_scalar("output name", value, "must not be empty");
    }
    if byte_len > u64::from(MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES) {
        return invalid_scalar("output name", value, "exceeds byte limit");
    }

    let mut chars = value.chars();
    let first = chars.next().expect("non-empty output name has first char");
    if !first.is_ascii_lowercase() {
        return invalid_scalar("output name", value, "must start with lowercase ASCII letter");
    }
    for ch in chars {
        if !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '+' || ch == '-') {
            return invalid_scalar("output name", value, "contains invalid character");
        }
    }
    Ok(())
}

pub fn validate_blake3_hex(value: &str) -> Result<(), DynamicPlanError> {
    let byte_len = len_as_u64(value.len());
    if byte_len != u64::from(BLAKE3_HEX_BYTES) {
        return invalid_scalar("blake3 hex", value, "must be exactly 64 lowercase hex bytes");
    }
    if !value.chars().all(is_lower_hex_char) {
        return invalid_scalar("blake3 hex", value, "must contain only lowercase hex characters");
    }
    Ok(())
}

pub fn validate_store_path_string(value: &str, store_prefix: &str) -> Result<(), DynamicPlanError> {
    if store_prefix.is_empty() {
        return invalid_scalar("store prefix", store_prefix, "must not be empty");
    }
    if !store_prefix.starts_with('/') {
        return invalid_scalar("store prefix", store_prefix, "must be absolute");
    }
    if store_prefix.ends_with('/') {
        return invalid_scalar("store prefix", store_prefix, "must not end with slash");
    }

    let Some(rest) = value.strip_prefix(store_prefix) else {
        return invalid_scalar("store path", value, "does not start with active store prefix");
    };
    let Some(rest) = rest.strip_prefix('/') else {
        return invalid_scalar("store path", value, "missing slash after store prefix");
    };
    let mut components = rest.split('/');
    let first_component = components.next().unwrap_or_default();
    if first_component.is_empty() {
        return invalid_scalar("store path", value, "missing store path component");
    }
    StorePath::<String>::from_bytes(first_component.as_bytes()).map_err(|_| DynamicPlanError::InvalidScalar {
        field: "store path",
        value: value.to_string(),
        reason: "invalid store path component",
    })?;
    for component in components {
        if component.is_empty() {
            return invalid_scalar("store path", value, "contains empty suffix component");
        }
        if component == "." || component == ".." {
            return invalid_scalar("store path", value, "contains non-normal suffix component");
        }
    }
    Ok(())
}

fn validate_id_like(field: &'static str, value: &str, max_bytes: u32) -> Result<(), DynamicPlanError> {
    let byte_len = len_as_u64(value.len());
    if value.is_empty() {
        return invalid_scalar(field, value, "must not be empty");
    }
    if byte_len > u64::from(max_bytes) {
        return invalid_scalar(field, value, "exceeds byte limit");
    }

    let mut chars = value.chars();
    let first = chars.next().expect("non-empty identifier has first char");
    if !first.is_ascii_lowercase() {
        return invalid_scalar(field, value, "must start with lowercase ASCII letter");
    }
    for ch in chars {
        if !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '.' || ch == '-') {
            return invalid_scalar(field, value, "contains invalid character");
        }
    }
    Ok(())
}

fn invalid_scalar<T>(field: &'static str, value: &str, reason: &'static str) -> Result<T, DynamicPlanError> {
    Err(DynamicPlanError::InvalidScalar {
        field,
        value: value.to_string(),
        reason,
    })
}

fn is_lower_hex_char(ch: char) -> bool {
    ch.is_ascii_digit() || ('a'..='f').contains(&ch)
}

fn len_as_u64(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STORE_PREFIX: &str = "/mantle/store";
    const TEST_STORE_COMPONENT: &str = "00000000000000000000000000000000-dynplan";
    const TEST_STORE_PATH: &str = "/mantle/store/00000000000000000000000000000000-dynplan";
    const TEST_BLAKE3_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn valid_plan_json() -> String {
        format!(
            r#"{{
  "schema": "mantle-plan-v1",
  "producer": {{ "logical_name": "resolver", "goal_hint": "goal-a" }},
  "sources": [{{ "id": "src.main", "path": "{TEST_STORE_PATH}", "nar_blake3": "{TEST_BLAKE3_HEX}" }}],
  "units": [{{
    "id": "unit.main",
    "derivation": {{
      "name": "unit-main",
      "builder": "{TEST_STORE_PATH}/bin/builder",
      "system": "x86_64-linux",
      "args": ["--build"],
      "outputs": ["out"],
      "env": {{ "KEY": "VALUE" }},
      "inputs": [
        {{ "kind": "source", "source": "src.main" }},
        {{ "kind": "store_path", "path": "{TEST_STORE_PATH}" }}
      ],
      "fixed_output": null,
      "addressing_mode": "content-addressed",
      "sandbox": "native",
      "dynamic_plan_outputs": ["plan"]
    }},
    "requested_outputs": ["out"],
    "policy": {{
      "sandbox": "inherit",
      "substitutions": "inherit",
      "store_prefix": "inherit",
      "host_paths": "none"
    }}
  }}],
  "roots": ["unit.main"],
  "provenance": {{ "generator": "test" }}
}}"#,
        )
    }

    #[test]
    fn decode_accepts_valid_plan_shape() {
        let plan = decode_plan_v1(valid_plan_json().as_bytes()).unwrap();

        assert_eq!(plan.schema, MANTLE_PLAN_V1_SCHEMA);
        assert_eq!(plan.producer.logical_name, "resolver");
        assert_eq!(plan.sources.len(), 1);
        assert_eq!(plan.units.len(), 1);
        assert_eq!(plan.roots, vec!["unit.main"]);
        assert_eq!(plan.units[0].derivation.sandbox, SandboxMode::Native);
    }

    #[test]
    fn decode_rejects_unknown_top_level_field() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value.as_object_mut().unwrap().insert("surprise".to_string(), serde_json::json!(true));

        let err = decode_plan_v1(value.to_string().as_bytes()).unwrap_err();
        assert!(matches!(err, DynamicPlanError::JsonDecode { .. }));
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn decode_rejects_unknown_nested_field() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        let producer = value.get_mut("producer").unwrap().as_object_mut().unwrap();
        producer.insert("surprise".to_string(), serde_json::json!(true));

        let err = decode_plan_v1(value.to_string().as_bytes()).unwrap_err();
        assert!(matches!(err, DynamicPlanError::JsonDecode { .. }));
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn decode_rejects_missing_required_field() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value.as_object_mut().unwrap().remove("schema");

        let err = decode_plan_v1(value.to_string().as_bytes()).unwrap_err();
        assert!(matches!(err, DynamicPlanError::JsonDecode { .. }));
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn decode_requires_nullable_goal_hint_field() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value.get_mut("producer").unwrap().as_object_mut().unwrap().remove("goal_hint");

        let err = decode_plan_v1(value.to_string().as_bytes()).unwrap_err();
        assert!(matches!(err, DynamicPlanError::MissingNullableField {
            field: "producer.goal_hint"
        }));
    }

    #[test]
    fn decode_requires_nullable_source_digest_field() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value.get_mut("sources").unwrap().as_array_mut().unwrap()[0]
            .as_object_mut()
            .unwrap()
            .remove("nar_blake3");

        let err = decode_plan_v1(value.to_string().as_bytes()).unwrap_err();
        assert!(matches!(err, DynamicPlanError::MissingNullableField {
            field: "sources[].nar_blake3"
        }));
    }

    #[test]
    fn decode_requires_nullable_fixed_output_field() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value.get_mut("units").unwrap().as_array_mut().unwrap()[0]
            .get_mut("derivation")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("fixed_output");

        let err = decode_plan_v1(value.to_string().as_bytes()).unwrap_err();
        assert!(matches!(err, DynamicPlanError::MissingNullableField {
            field: "units[].derivation.fixed_output"
        }));
    }

    #[test]
    fn decode_accepts_present_null_nullable_fields() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value.get_mut("producer").unwrap()["goal_hint"] = serde_json::Value::Null;
        value.get_mut("sources").unwrap().as_array_mut().unwrap()[0]["nar_blake3"] = serde_json::Value::Null;
        value.get_mut("units").unwrap().as_array_mut().unwrap()[0]["derivation"]["fixed_output"] =
            serde_json::Value::Null;

        let plan = decode_plan_v1(value.to_string().as_bytes()).unwrap();
        assert_eq!(plan.producer.goal_hint, None);
        assert_eq!(plan.sources[0].nar_blake3, None);
        assert_eq!(plan.units[0].derivation.fixed_output, None);
    }

    #[test]
    fn decode_rejects_oversized_plan_before_json_parse() {
        let oversized_len = usize::try_from(MAX_DYNAMIC_PLAN_BYTES).unwrap() + 1;
        let bytes = vec![b' '; oversized_len];

        let err = decode_plan_v1(&bytes).unwrap_err();
        assert!(matches!(err, DynamicPlanError::PlanTooLarge { .. }));
        assert!(err.to_string().contains("exceeds byte limit"));
    }

    #[test]
    fn unit_id_accepts_expected_grammar() {
        assert!(validate_unit_id("unit.main-1_ok").is_ok());
        assert!(validate_unit_id("a").is_ok());
    }

    #[test]
    fn unit_id_rejects_invalid_grammar() {
        assert!(validate_unit_id("").is_err());
        assert!(validate_unit_id("Unit").is_err());
        assert!(validate_unit_id("1unit").is_err());
        assert!(validate_unit_id("unit/main").is_err());
    }

    #[test]
    fn source_id_uses_unit_id_grammar() {
        assert!(validate_source_id("src.main").is_ok());
        assert!(validate_source_id("Src").is_err());
    }

    #[test]
    fn output_name_accepts_expected_grammar() {
        assert!(validate_output_name("out").is_ok());
        assert!(validate_output_name("lib+static_1").is_ok());
    }

    #[test]
    fn output_name_rejects_invalid_grammar() {
        assert!(validate_output_name("").is_err());
        assert!(validate_output_name("Out").is_err());
        assert!(validate_output_name("1out").is_err());
        assert!(validate_output_name("out.dev").is_err());
    }

    #[test]
    fn blake3_hex_accepts_lowercase_hex() {
        assert!(validate_blake3_hex(TEST_BLAKE3_HEX).is_ok());
    }

    #[test]
    fn blake3_hex_rejects_wrong_length_or_uppercase() {
        assert!(validate_blake3_hex("0").is_err());
        let uppercase = TEST_BLAKE3_HEX.to_ascii_uppercase();
        assert!(validate_blake3_hex(&uppercase).is_err());
        let bad_char = "g000000000000000000000000000000000000000000000000000000000000000";
        assert!(validate_blake3_hex(bad_char).is_err());
    }

    #[test]
    fn store_path_string_accepts_store_path_and_suffix() {
        assert!(validate_store_path_string(TEST_STORE_PATH, TEST_STORE_PREFIX).is_ok());
        let nested = format!("{TEST_STORE_PATH}/bin/tool");
        assert!(validate_store_path_string(&nested, TEST_STORE_PREFIX).is_ok());
        assert!(
            validate_store_path_string(&format!("{TEST_STORE_PREFIX}/{TEST_STORE_COMPONENT}"), TEST_STORE_PREFIX)
                .is_ok()
        );
    }

    #[test]
    fn store_path_string_rejects_wrong_prefix_and_bad_component() {
        assert!(validate_store_path_string(TEST_STORE_PATH, "/other/store").is_err());
        assert!(validate_store_path_string("/mantle/store/not-a-store-path", TEST_STORE_PREFIX).is_err());
        assert!(validate_store_path_string("/mantle/store", TEST_STORE_PREFIX).is_err());
    }

    #[test]
    fn store_path_string_rejects_non_normal_suffix() {
        assert!(validate_store_path_string(&format!("{TEST_STORE_PATH}//bin"), TEST_STORE_PREFIX).is_err());
        assert!(validate_store_path_string(&format!("{TEST_STORE_PATH}/./bin"), TEST_STORE_PREFIX).is_err());
        assert!(validate_store_path_string(&format!("{TEST_STORE_PATH}/../bin"), TEST_STORE_PREFIX).is_err());
    }

    #[test]
    fn store_prefix_must_be_absolute_and_normalized() {
        assert!(validate_store_path_string(TEST_STORE_PATH, "").is_err());
        assert!(validate_store_path_string(TEST_STORE_PATH, "mantle/store").is_err());
        assert!(validate_store_path_string(TEST_STORE_PATH, "/mantle/store/").is_err());
    }
}
