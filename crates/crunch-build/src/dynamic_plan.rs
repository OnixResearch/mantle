//! Native dynamic build-plan ABI.
//!
//! This module is the pure Rust core for `mantle-plan-v1` decoding,
//! canonicalization, BLAKE3 digesting, and scalar grammar checks. It
//! deliberately performs no store I/O, no worker mutation, and no scheduler
//! registration.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

const BYTES_PER_KIB: u64 = 1024;
const KIB_PER_MIB: u64 = 1024;
pub const MAX_DYNAMIC_PLAN_MIB: u64 = 4;
pub const MAX_DYNAMIC_PLAN_BYTES: u64 = MAX_DYNAMIC_PLAN_MIB.saturating_mul(KIB_PER_MIB).saturating_mul(BYTES_PER_KIB);
pub const MAX_DYNAMIC_PLAN_UNITS: u32 = 4096;
pub const MAX_DYNAMIC_PLAN_DEPENDENCIES_PER_UNIT: u32 = 256;
pub const MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT: u32 = 16;
pub const MAX_DYNAMIC_PLAN_ENV_ENTRIES_PER_UNIT: u32 = 512;
pub const MAX_DYNAMIC_PLAN_STRING_BYTES: u32 = 16_384;
pub const MAX_DYNAMIC_PLAN_NESTING_DEPTH: u32 = 16;
pub const MAX_DYNAMIC_PLAN_ID_BYTES: u32 = 128;
pub const MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES: u32 = 64;
pub const BLAKE3_HEX_BYTES: usize = 64;
pub const MANTLE_PLAN_V1_SCHEMA: &str = "mantle-plan-v1";
pub const STORE_PATH_KIND: &str = "store_path";
pub const SOURCE_KIND: &str = "source";
pub const UNIT_OUTPUT_KIND: &str = "unit_output";
pub const INHERIT_POLICY_VALUE: &str = "inherit";
pub const NO_HOST_PATHS_POLICY_VALUE: &str = "none";
pub const NATIVE_SANDBOX_VALUE: &str = "native";
const JSON_ROOT_DEPTH: u32 = 1;
const JSON_CHILD_DEPTH_INCREMENT: u32 = 1;
const DYNAMIC_PLACEHOLDER_START: &str = "{{mantle-";
const DYNAMIC_PLACEHOLDER_END: &str = "}}";
const DYNAMIC_SOURCE_PLACEHOLDER_PREFIX: &str = "source:";
const DYNAMIC_UNIT_OUTPUT_PLACEHOLDER_PREFIX: &str = "unit-output:";
const MAX_DYNAMIC_PLACEHOLDERS_PER_STRING: u32 = 1024;

pub type UnitId = String;
pub type SourceId = String;
pub type StorePathString = String;
pub type Blake3Hex = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynamicPlaceholder {
    Source { source: SourceId },
    UnitOutput { unit: UnitId, output: String },
}

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

    #[error("{field} exceeds limit: {actual_count} > {max_count}")]
    LimitExceeded {
        field: &'static str,
        actual_count: u64,
        max_count: u64,
    },

    #[error("policy widening rejected for {field}: `{value}` must be `{expected}`")]
    PolicyWidening {
        field: &'static str,
        value: String,
        expected: &'static str,
    },

    #[error("serializing canonical mantle-plan-v1 JSON: {message}")]
    CanonicalJson { message: String },

    #[error("dynamic plan arithmetic overflow: {field}")]
    ArithmeticOverflow { field: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDynamicPlanV1 {
    pub plan: DynamicPlanV1,
    pub bytes: Vec<u8>,
    pub digest: Blake3Hex,
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
    let actual_bytes = len_as_u64("plan bytes", bytes.len())?;
    if actual_bytes > MAX_DYNAMIC_PLAN_BYTES {
        return Err(DynamicPlanError::PlanTooLarge {
            actual_bytes,
            max_bytes: MAX_DYNAMIC_PLAN_BYTES,
        });
    }

    let value = decode_plan_json_value(bytes)?;
    validate_json_nesting_depth(&value)?;
    require_nullable_fields_present(&value)?;
    serde_json::from_value(value).map_err(|err| DynamicPlanError::JsonDecode {
        message: err.to_string(),
    })
}

pub fn decode_validated_plan_v1(bytes: &[u8], store_prefix: &str) -> Result<CanonicalDynamicPlanV1, DynamicPlanError> {
    let plan = decode_plan_v1(bytes)?;
    validate_plan_v1(&plan, store_prefix)?;
    let canonical = canonicalize_plan_v1(&plan);
    let canonical_bytes = canonical_plan_v1_bytes(&canonical)?;
    let digest = blake3_hex_digest(&canonical_bytes);

    assert!(!canonical_bytes.is_empty());
    assert_eq!(digest.len(), BLAKE3_HEX_BYTES);

    Ok(CanonicalDynamicPlanV1 {
        plan: canonical,
        bytes: canonical_bytes,
        digest,
    })
}

pub fn validate_plan_v1(plan: &DynamicPlanV1, store_prefix: &str) -> Result<(), DynamicPlanError> {
    validate_store_prefix(store_prefix)?;
    validate_plan_schema(plan)?;
    validate_plan_limits(plan)?;
    validate_producer(&plan.producer, store_prefix)?;
    validate_sources(&plan.sources, store_prefix)?;
    validate_units(&plan.units, store_prefix)?;
    validate_roots(&plan.roots)?;
    validate_provenance(&plan.provenance, store_prefix)?;
    validate_plan_graph(plan)?;
    canonical_plan_v1_bytes(plan)?;
    Ok(())
}

pub fn decode_canonical_plan_v1(bytes: &[u8]) -> Result<CanonicalDynamicPlanV1, DynamicPlanError> {
    let plan = decode_plan_v1(bytes)?;
    let canonical = canonicalize_plan_v1(&plan);
    let canonical_bytes = canonical_plan_v1_bytes(&canonical)?;
    let digest = blake3_hex_digest(&canonical_bytes);

    assert!(!canonical_bytes.is_empty());
    assert_eq!(digest.len(), BLAKE3_HEX_BYTES);

    Ok(CanonicalDynamicPlanV1 {
        plan: canonical,
        bytes: canonical_bytes,
        digest,
    })
}

pub fn canonical_plan_v1_bytes(plan: &DynamicPlanV1) -> Result<Vec<u8>, DynamicPlanError> {
    let canonical = canonicalize_plan_v1(plan);
    let bytes = serde_json::to_vec(&canonical).map_err(|err| DynamicPlanError::CanonicalJson {
        message: err.to_string(),
    })?;

    let actual_bytes = len_as_u64("canonical plan bytes", bytes.len())?;
    if actual_bytes > MAX_DYNAMIC_PLAN_BYTES {
        return Err(DynamicPlanError::PlanTooLarge {
            actual_bytes,
            max_bytes: MAX_DYNAMIC_PLAN_BYTES,
        });
    }

    assert!(!bytes.is_empty());

    Ok(bytes)
}

pub fn canonical_plan_v1_digest(plan: &DynamicPlanV1) -> Result<Blake3Hex, DynamicPlanError> {
    let bytes = canonical_plan_v1_bytes(plan)?;
    let digest = blake3_hex_digest(&bytes);

    assert_eq!(digest.len(), BLAKE3_HEX_BYTES);
    assert!(validate_blake3_hex(&digest).is_ok());

    Ok(digest)
}

pub fn canonicalize_plan_v1(plan: &DynamicPlanV1) -> DynamicPlanV1 {
    let source_count = plan.sources.len();
    let unit_count = plan.units.len();
    let root_count = plan.roots.len();

    let mut canonical = plan.clone();
    canonical.sources.sort_by(|left, right| left.id.cmp(&right.id));
    canonical.units.sort_by(|left, right| left.id.cmp(&right.id));
    canonical.roots.sort();
    canonical.units = canonical.units.into_iter().map(canonicalize_unit).collect();

    assert_eq!(canonical.sources.len(), source_count);
    assert_eq!(canonical.units.len(), unit_count);
    assert_eq!(canonical.roots.len(), root_count);

    canonical
}

fn decode_plan_json_value(bytes: &[u8]) -> Result<Value, DynamicPlanError> {
    serde_json::from_slice(bytes).map_err(|err| DynamicPlanError::JsonDecode {
        message: err.to_string(),
    })
}

fn validate_json_nesting_depth(value: &Value) -> Result<(), DynamicPlanError> {
    assert!(JSON_ROOT_DEPTH > 0, "JSON root depth must be positive");
    assert!(JSON_CHILD_DEPTH_INCREMENT > 0, "JSON child depth increment must be positive");
    let mut stack = vec![(value, JSON_ROOT_DEPTH)];
    while let Some((current, depth)) = stack.pop() {
        if depth > MAX_DYNAMIC_PLAN_NESTING_DEPTH {
            return limit_exceeded("json nesting depth", u64::from(depth), MAX_DYNAMIC_PLAN_NESTING_DEPTH);
        }
        let child_depth =
            depth.checked_add(JSON_CHILD_DEPTH_INCREMENT).ok_or(DynamicPlanError::ArithmeticOverflow {
                field: "json nesting depth",
            })?;
        match current {
            Value::Array(items) => {
                for item in items {
                    stack.push((item, child_depth));
                }
            }
            Value::Object(items) => {
                for item in items.values() {
                    stack.push((item, child_depth));
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
    Ok(())
}

fn validate_plan_schema(plan: &DynamicPlanV1) -> Result<(), DynamicPlanError> {
    validate_required_string("schema", &plan.schema, "")?;
    if plan.schema != MANTLE_PLAN_V1_SCHEMA {
        return invalid_scalar("schema", &plan.schema, "must be mantle-plan-v1");
    }
    Ok(())
}

fn validate_plan_limits(plan: &DynamicPlanV1) -> Result<(), DynamicPlanError> {
    validate_len_limit("units", plan.units.len(), MAX_DYNAMIC_PLAN_UNITS)?;
    for unit in &plan.units {
        validate_len_limit("unit dependencies", unit.derivation.inputs.len(), MAX_DYNAMIC_PLAN_DEPENDENCIES_PER_UNIT)?;
        validate_len_limit("unit outputs", unit.derivation.outputs.len(), MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT)?;
        validate_len_limit("requested outputs", unit.requested_outputs.len(), MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT)?;
        validate_len_limit(
            "dynamic plan outputs",
            unit.derivation.dynamic_plan_outputs.len(),
            MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT,
        )?;
        validate_len_limit("unit environment", unit.derivation.env.len(), MAX_DYNAMIC_PLAN_ENV_ENTRIES_PER_UNIT)?;
    }
    Ok(())
}

fn validate_producer(producer: &PlanProducer, store_prefix: &str) -> Result<(), DynamicPlanError> {
    validate_required_string("producer logical name", &producer.logical_name, store_prefix)?;
    if let Some(goal_hint) = &producer.goal_hint {
        validate_required_string("producer goal hint", goal_hint, store_prefix)?;
    }
    Ok(())
}

fn validate_sources(sources: &[DeclaredSourceInput], store_prefix: &str) -> Result<(), DynamicPlanError> {
    for source in sources {
        validate_source_id(&source.id)?;
        validate_store_path_string(&source.path, store_prefix)?;
        if let Some(digest) = &source.nar_blake3 {
            validate_blake3_hex(digest)?;
        }
    }
    Ok(())
}

fn validate_units(units: &[DynamicUnit], store_prefix: &str) -> Result<(), DynamicPlanError> {
    for unit in units {
        validate_unit_id(&unit.id)?;
        validate_dynamic_derivation(&unit.derivation, store_prefix)?;
        validate_output_names("requested outputs", &unit.requested_outputs)?;
        validate_unit_policy(&unit.policy)?;
    }
    Ok(())
}

fn validate_dynamic_derivation(derivation: &DynamicDerivation, store_prefix: &str) -> Result<(), DynamicPlanError> {
    validate_required_string("derivation name", &derivation.name, store_prefix)?;
    validate_store_path_string(&derivation.builder, store_prefix)?;
    validate_required_string("system", &derivation.system, store_prefix)?;
    validate_argument_strings(&derivation.args, store_prefix)?;
    validate_output_names("unit outputs", &derivation.outputs)?;
    validate_environment(&derivation.env, store_prefix)?;
    validate_inputs(&derivation.inputs, store_prefix)?;
    validate_dynamic_placeholders(derivation)?;
    validate_fixed_output(derivation.fixed_output.as_ref(), store_prefix)?;
    validate_sandbox_mode(&derivation.sandbox)?;
    validate_output_names("dynamic plan outputs", &derivation.dynamic_plan_outputs)?;
    Ok(())
}

fn validate_argument_strings(args: &[String], store_prefix: &str) -> Result<(), DynamicPlanError> {
    for arg in args {
        validate_bounded_string("argument", arg)?;
        validate_no_absolute_host_path("argument", arg, store_prefix)?;
    }
    Ok(())
}

fn validate_environment(env: &BTreeMap<String, String>, store_prefix: &str) -> Result<(), DynamicPlanError> {
    for (key, value) in env {
        validate_required_string("environment key", key, "")?;
        validate_bounded_string("environment value", value)?;
        validate_no_absolute_host_path("environment value", value, store_prefix)?;
    }
    Ok(())
}

fn validate_inputs(inputs: &[DynamicInput], store_prefix: &str) -> Result<(), DynamicPlanError> {
    for input in inputs {
        match input {
            DynamicInput::StorePath { path } => validate_store_path_string(path, store_prefix)?,
            DynamicInput::Source { source } => validate_source_id(source)?,
            DynamicInput::UnitOutput { unit, output } => {
                validate_unit_id(unit)?;
                validate_output_name(output)?;
            }
        }
    }
    Ok(())
}

fn validate_dynamic_placeholders(derivation: &DynamicDerivation) -> Result<(), DynamicPlanError> {
    let declared_sources = derivation
        .inputs
        .iter()
        .filter_map(|input| match input {
            DynamicInput::Source { source } => Some(source.as_str()),
            DynamicInput::StorePath { .. } | DynamicInput::UnitOutput { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let declared_outputs = derivation
        .inputs
        .iter()
        .filter_map(|input| match input {
            DynamicInput::UnitOutput { unit, output } => Some((unit.as_str(), output.as_str())),
            DynamicInput::StorePath { .. } | DynamicInput::Source { .. } => None,
        })
        .collect::<BTreeSet<_>>();

    for value in derivation.args.iter().chain(derivation.env.values()) {
        for placeholder in parse_dynamic_placeholders(value)? {
            match placeholder {
                DynamicPlaceholder::Source { source } if declared_sources.contains(source.as_str()) => {}
                DynamicPlaceholder::UnitOutput { unit, output }
                    if declared_outputs.contains(&(unit.as_str(), output.as_str())) => {}
                DynamicPlaceholder::Source { .. } | DynamicPlaceholder::UnitOutput { .. } => {
                    return invalid_scalar(
                        "dynamic placeholder",
                        value,
                        "references an input not declared by the unit",
                    );
                }
            }
        }
    }

    assert!(declared_sources.len() <= derivation.inputs.len());
    assert!(declared_outputs.len() <= derivation.inputs.len());
    Ok(())
}

pub fn parse_dynamic_placeholders(value: &str) -> Result<Vec<DynamicPlaceholder>, DynamicPlanError> {
    assert!(!DYNAMIC_PLACEHOLDER_START.is_empty(), "placeholder start delimiter must not be empty");
    assert!(!DYNAMIC_PLACEHOLDER_END.is_empty(), "placeholder end delimiter must not be empty");
    let placeholder_slots =
        usize::try_from(MAX_DYNAMIC_PLACEHOLDERS_PER_STRING).map_err(|_| DynamicPlanError::ArithmeticOverflow {
            field: "dynamic placeholder capacity",
        })?;
    let mut placeholders = Vec::with_capacity(placeholder_slots);
    let mut remaining = value;
    for _ in 0..MAX_DYNAMIC_PLACEHOLDERS_PER_STRING {
        let Some(start) = remaining.find(DYNAMIC_PLACEHOLDER_START) else {
            assert!(placeholders.len() <= placeholder_slots, "placeholder count must stay within capacity");
            return Ok(placeholders);
        };
        let body_start = start.saturating_add(DYNAMIC_PLACEHOLDER_START.len());
        let after_start = &remaining[body_start..];
        let Some(end) = after_start.find(DYNAMIC_PLACEHOLDER_END) else {
            return invalid_scalar("dynamic placeholder", value, "is missing a closing delimiter");
        };
        let body = &after_start[..end];
        placeholders.push(parse_dynamic_placeholder_body(body, value)?);
        let consumed = body_start.saturating_add(end).saturating_add(DYNAMIC_PLACEHOLDER_END.len());
        remaining = &remaining[consumed..];
    }
    limit_exceeded(
        "dynamic placeholders per string",
        u64::from(MAX_DYNAMIC_PLACEHOLDERS_PER_STRING).saturating_add(1),
        MAX_DYNAMIC_PLACEHOLDERS_PER_STRING,
    )
}

pub fn resolve_dynamic_placeholders(
    value: &str,
    source_paths: &BTreeMap<SourceId, StorePathString>,
    unit_output_paths: &BTreeMap<(UnitId, String), StorePathString>,
) -> Result<String, DynamicPlanError> {
    let placeholders = parse_dynamic_placeholders(value)?;
    let mut resolved = value.to_string();
    for placeholder in &placeholders {
        let (token, path) = match placeholder {
            DynamicPlaceholder::Source { source } => {
                let path = source_paths.get(source).ok_or_else(|| DynamicPlanError::InvalidScalar {
                    field: "dynamic placeholder",
                    value: value.to_string(),
                    reason: "source path binding is unavailable",
                })?;
                (format!("{{{{mantle-source:{source}}}}}"), path)
            }
            DynamicPlaceholder::UnitOutput { unit, output } => {
                let path = unit_output_paths.get(&(unit.clone(), output.clone())).ok_or_else(|| {
                    DynamicPlanError::InvalidScalar {
                        field: "dynamic placeholder",
                        value: value.to_string(),
                        reason: "unit output path binding is unavailable",
                    }
                })?;
                (format!("{{{{mantle-unit-output:{unit}:{output}}}}}"), path)
            }
        };
        resolved = resolved.replace(&token, path);
    }
    validate_bounded_string("resolved dynamic value", &resolved)?;
    if resolved.contains(DYNAMIC_PLACEHOLDER_START) {
        return invalid_scalar("dynamic placeholder", value, "was not fully resolved");
    }
    assert_eq!(parse_dynamic_placeholders(&resolved)?.len(), 0);
    assert!(
        len_as_u64("resolved dynamic value", resolved.len())? <= u64::from(MAX_DYNAMIC_PLAN_STRING_BYTES),
        "resolved dynamic value must stay within the validated byte limit"
    );
    Ok(resolved)
}

fn parse_dynamic_placeholder_body(
    body: &str,
    original: impl Into<String>,
) -> Result<DynamicPlaceholder, DynamicPlanError> {
    assert!(!DYNAMIC_SOURCE_PLACEHOLDER_PREFIX.is_empty(), "source placeholder prefix must not be empty");
    assert!(
        !DYNAMIC_UNIT_OUTPUT_PLACEHOLDER_PREFIX.is_empty(),
        "unit-output placeholder prefix must not be empty"
    );
    let original = original.into();
    if let Some(source) = body.strip_prefix(DYNAMIC_SOURCE_PLACEHOLDER_PREFIX) {
        validate_source_id(source)?;
        return Ok(DynamicPlaceholder::Source {
            source: source.to_string(),
        });
    }
    if let Some(unit_output) = body.strip_prefix(DYNAMIC_UNIT_OUTPUT_PLACEHOLDER_PREFIX) {
        let Some((unit, output)) = unit_output.rsplit_once(':') else {
            return invalid_scalar("dynamic placeholder", original, "unit output token is missing its output name");
        };
        validate_unit_id(unit)?;
        validate_output_name(output)?;
        return Ok(DynamicPlaceholder::UnitOutput {
            unit: unit.to_string(),
            output: output.to_string(),
        });
    }
    invalid_scalar("dynamic placeholder", original, "uses an unsupported token kind")
}

fn validate_fixed_output(fixed_output: Option<&FixedOutputSpec>, store_prefix: &str) -> Result<(), DynamicPlanError> {
    let Some(fixed_output) = fixed_output else {
        return Ok(());
    };
    validate_required_string("fixed output hash", &fixed_output.hash, store_prefix)?;
    if fixed_output.algo == FixedOutputHashAlgo::Blake3 {
        validate_blake3_hex(&fixed_output.hash)?;
    }
    Ok(())
}

fn validate_sandbox_mode(sandbox: &SandboxMode) -> Result<(), DynamicPlanError> {
    match sandbox {
        SandboxMode::Native => Ok(()),
    }
}

fn validate_output_names(field: &'static str, outputs: &[String]) -> Result<(), DynamicPlanError> {
    let mut seen = BTreeSet::new();
    for output in outputs {
        validate_output_name(output)?;
        if !seen.insert(output.as_str()) {
            return invalid_scalar(field, output, "contains duplicate output name");
        }
    }
    Ok(())
}

fn validate_unit_policy(policy: &DynamicUnitPolicy) -> Result<(), DynamicPlanError> {
    validate_policy_literal("policy sandbox", &policy.sandbox, INHERIT_POLICY_VALUE)?;
    validate_policy_literal("policy substitutions", &policy.substitutions, INHERIT_POLICY_VALUE)?;
    validate_policy_literal("policy store prefix", &policy.store_prefix, INHERIT_POLICY_VALUE)?;
    validate_policy_literal("policy host paths", &policy.host_paths, NO_HOST_PATHS_POLICY_VALUE)?;
    Ok(())
}

fn validate_roots(roots: &[UnitId]) -> Result<(), DynamicPlanError> {
    for root in roots {
        validate_unit_id(root)?;
    }
    Ok(())
}

fn validate_provenance(provenance: &BTreeMap<String, String>, store_prefix: &str) -> Result<(), DynamicPlanError> {
    for (key, value) in provenance {
        validate_required_string("provenance key", key, "")?;
        validate_bounded_string("provenance value", value)?;
        validate_no_absolute_host_path("provenance value", value, store_prefix)?;
    }
    Ok(())
}

fn validate_plan_graph(plan: &DynamicPlanV1) -> Result<(), DynamicPlanError> {
    let source_ids = collect_source_ids(&plan.sources)?;
    let unit_outputs = collect_unit_outputs(&plan.units)?;

    validate_root_graph(&plan.roots, &unit_outputs)?;
    validate_input_graph(&plan.units, &source_ids, &unit_outputs)?;
    validate_unit_dependency_cycles(&plan.units)?;

    assert_eq!(unit_outputs.len(), plan.units.len());
    if !plan.roots.is_empty() {
        assert!(!unit_outputs.is_empty(), "non-empty roots require registered unit outputs");
    }

    Ok(())
}

fn collect_source_ids(sources: &[DeclaredSourceInput]) -> Result<BTreeSet<&str>, DynamicPlanError> {
    let mut source_ids = BTreeSet::new();
    for source in sources {
        if !source_ids.insert(source.id.as_str()) {
            return invalid_scalar("source id", &source.id, "contains duplicate source id");
        }
    }
    assert_eq!(source_ids.len(), sources.len());
    Ok(source_ids)
}

fn collect_unit_outputs(units: &[DynamicUnit]) -> Result<BTreeMap<&str, BTreeSet<&str>>, DynamicPlanError> {
    let mut unit_outputs = BTreeMap::new();
    for unit in units {
        let output_names = unit.derivation.outputs.iter().map(String::as_str).collect::<BTreeSet<_>>();
        if unit_outputs.insert(unit.id.as_str(), output_names).is_some() {
            return invalid_scalar("unit id", &unit.id, "contains duplicate unit id");
        }
    }
    assert_eq!(unit_outputs.len(), units.len());
    Ok(unit_outputs)
}

fn validate_root_graph(
    roots: &[UnitId],
    unit_outputs: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<(), DynamicPlanError> {
    if roots.is_empty() {
        return invalid_scalar("roots", "", "must not be empty");
    }

    let mut seen_roots = BTreeSet::new();
    for root in roots {
        if !seen_roots.insert(root.as_str()) {
            return invalid_scalar("roots", root, "contains duplicate root");
        }
        if !unit_outputs.contains_key(root.as_str()) {
            return invalid_scalar("roots", root, "references unknown unit");
        }
    }
    assert_eq!(seen_roots.len(), roots.len());
    Ok(())
}

fn validate_input_graph(
    units: &[DynamicUnit],
    source_ids: &BTreeSet<&str>,
    unit_outputs: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<(), DynamicPlanError> {
    assert_eq!(unit_outputs.len(), units.len(), "every unit must have an output-set entry");
    assert!(units.iter().all(|unit| !unit.id.is_empty()), "validated unit IDs must not be empty");
    for unit in units {
        for input in &unit.derivation.inputs {
            match input {
                DynamicInput::StorePath { .. } => {}
                DynamicInput::Source { source } => {
                    if !source_ids.contains(source.as_str()) {
                        return invalid_scalar("source dependency", source, "references undeclared source input");
                    }
                }
                DynamicInput::UnitOutput { unit, output } => {
                    let Some(outputs) = unit_outputs.get(unit.as_str()) else {
                        return invalid_scalar("unit output dependency", unit, "references unknown unit");
                    };
                    if !outputs.contains(output.as_str()) {
                        return invalid_scalar("unit output dependency", output, "references unknown output");
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_unit_dependency_cycles(units: &[DynamicUnit]) -> Result<(), DynamicPlanError> {
    let mut remaining_dependencies = build_unit_dependency_sets(units);
    assert_eq!(remaining_dependencies.len(), units.len(), "dependency map must cover every unit");
    let dependents = build_dependents_by_dependency(&remaining_dependencies);
    let mut ready = remaining_dependencies
        .iter()
        .filter_map(|(unit, dependencies)| dependencies.is_empty().then_some(*unit))
        .collect::<BTreeSet<_>>();
    let mut processed_count = 0usize;

    while let Some(unit) = ready.pop_first() {
        processed_count = processed_count.checked_add(1).ok_or(DynamicPlanError::ArithmeticOverflow {
            field: "processed unit count",
        })?;
        if let Some(unit_dependents) = dependents.get(unit) {
            for dependent in unit_dependents {
                let Some(dependencies) = remaining_dependencies.get_mut(dependent) else {
                    continue;
                };
                dependencies.remove(unit);
                if dependencies.is_empty() {
                    ready.insert(*dependent);
                }
            }
        }
    }

    if processed_count != units.len() {
        let cycle_member = remaining_dependencies
            .iter()
            .find_map(|(unit, dependencies)| (!dependencies.is_empty()).then_some(*unit))
            .unwrap_or("<unknown>");
        return invalid_scalar("unit dependency graph", cycle_member, "contains dependency cycle");
    }
    assert_eq!(processed_count, units.len());
    Ok(())
}

fn build_unit_dependency_sets(units: &[DynamicUnit]) -> BTreeMap<&str, BTreeSet<&str>> {
    let dependencies_by_unit = units
        .iter()
        .map(|unit| {
            let dependencies = unit
                .derivation
                .inputs
                .iter()
                .filter_map(|input| match input {
                    DynamicInput::UnitOutput { unit: dependency, .. } => Some(dependency.as_str()),
                    DynamicInput::StorePath { .. } | DynamicInput::Source { .. } => None,
                })
                .collect::<BTreeSet<_>>();
            (unit.id.as_str(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(dependencies_by_unit.len(), units.len());
    dependencies_by_unit
}

fn build_dependents_by_dependency<'a>(
    dependencies_by_unit: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    let mut dependents = BTreeMap::new();
    for (unit, dependencies) in dependencies_by_unit {
        for dependency in dependencies {
            dependents.entry(*dependency).or_insert_with(BTreeSet::new).insert(*unit);
        }
    }
    dependents
}

fn validate_policy_literal(
    field: &'static str,
    value: impl AsRef<str>,
    expected: &'static str,
) -> Result<(), DynamicPlanError> {
    let value = value.as_ref();
    validate_bounded_string(field, value)?;
    if value == expected {
        Ok(())
    } else {
        Err(DynamicPlanError::PolicyWidening {
            field,
            value: value.to_string(),
            expected,
        })
    }
}

fn validate_required_string(
    field: &'static str,
    value: impl AsRef<str>,
    store_prefix: &str,
) -> Result<(), DynamicPlanError> {
    let value = value.as_ref();
    if value.is_empty() {
        return invalid_scalar(field, value, "must not be empty");
    }
    validate_bounded_string(field, value)?;
    validate_no_absolute_host_path(field, value, store_prefix)
}

fn validate_bounded_string(field: &'static str, value: impl AsRef<str>) -> Result<(), DynamicPlanError> {
    let value = value.as_ref();
    let byte_len = len_as_u64(field, value.len())?;
    if byte_len > u64::from(MAX_DYNAMIC_PLAN_STRING_BYTES) {
        return limit_exceeded(field, byte_len, MAX_DYNAMIC_PLAN_STRING_BYTES);
    }
    Ok(())
}

fn validate_no_absolute_host_path(
    field: &'static str,
    value: impl AsRef<str>,
    store_prefix: &str,
) -> Result<(), DynamicPlanError> {
    let value = value.as_ref();
    if store_prefix.is_empty() {
        return Ok(());
    }
    if !value.starts_with('/') {
        return Ok(());
    }
    if validate_store_path_string(value, store_prefix).is_ok() {
        return Ok(());
    }
    invalid_scalar(field, value, "absolute non-store host paths are forbidden")
}

fn validate_len_limit(field: &'static str, len: usize, max: u32) -> Result<(), DynamicPlanError> {
    let actual_count = len_as_u64(field, len)?;
    let max_count = u64::from(max);
    if actual_count > max_count {
        limit_exceeded(field, actual_count, max)
    } else {
        Ok(())
    }
}

fn limit_exceeded<T>(field: &'static str, actual_count: u64, max_count: u32) -> Result<T, DynamicPlanError> {
    Err(DynamicPlanError::LimitExceeded {
        field,
        actual_count,
        max_count: u64::from(max_count),
    })
}

fn canonicalize_unit(mut unit: DynamicUnit) -> DynamicUnit {
    let requested_output_count = unit.requested_outputs.len();
    let input_count = unit.derivation.inputs.len();

    unit.requested_outputs.sort();
    unit.derivation.outputs.sort();
    unit.derivation.dynamic_plan_outputs.sort();
    unit.derivation
        .inputs
        .sort_by(|left, right| dynamic_input_sort_key(left).cmp(&dynamic_input_sort_key(right)));

    assert_eq!(unit.requested_outputs.len(), requested_output_count);
    assert_eq!(unit.derivation.inputs.len(), input_count);

    unit
}

fn dynamic_input_sort_key(input: &DynamicInput) -> (&'static str, &str, &str) {
    match input {
        DynamicInput::StorePath { path } => (STORE_PATH_KIND, path.as_str(), ""),
        DynamicInput::Source { source } => (SOURCE_KIND, source.as_str(), ""),
        DynamicInput::UnitOutput { unit, output } => (UNIT_OUTPUT_KIND, unit.as_str(), output.as_str()),
    }
}

fn blake3_hex_digest(bytes: &[u8]) -> Blake3Hex {
    let digest = blake3::hash(bytes).to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_BYTES);
    digest
}

fn require_nullable_fields_present(value: &Value) -> Result<(), DynamicPlanError> {
    assert!(!MANTLE_PLAN_V1_SCHEMA.is_empty(), "dynamic plan schema name must not be empty");
    assert!(
        MAX_DYNAMIC_PLAN_NESTING_DEPTH > JSON_ROOT_DEPTH,
        "dynamic plan nesting bound must exceed root depth"
    );
    if let Some(producer) = value.get("producer").and_then(Value::as_object) {
        require_json_key(producer, RequiredJsonKey {
            key: "goal_hint",
            field: "producer.goal_hint",
        })?;
    }
    if let Some(sources) = value.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(source) = source.as_object() {
                require_json_key(source, RequiredJsonKey {
                    key: "nar_blake3",
                    field: "sources[].nar_blake3",
                })?;
            }
        }
    }
    if let Some(units) = value.get("units").and_then(Value::as_array) {
        for unit in units {
            let Some(derivation) = unit.get("derivation").and_then(Value::as_object) else {
                continue;
            };
            require_json_key(derivation, RequiredJsonKey {
                key: "fixed_output",
                field: "units[].derivation.fixed_output",
            })?;
        }
    }
    Ok(())
}

struct RequiredJsonKey {
    key: &'static str,
    field: &'static str,
}

fn require_json_key(
    object: &serde_json::Map<String, Value>,
    required: RequiredJsonKey,
) -> Result<(), DynamicPlanError> {
    if object.contains_key(required.key) {
        Ok(())
    } else {
        Err(DynamicPlanError::MissingNullableField { field: required.field })
    }
}

pub fn validate_unit_id(value: &str) -> Result<(), DynamicPlanError> {
    validate_id_like("unit id", value, MAX_DYNAMIC_PLAN_ID_BYTES)
}

pub fn validate_source_id(value: &str) -> Result<(), DynamicPlanError> {
    validate_id_like("source id", value, MAX_DYNAMIC_PLAN_ID_BYTES)
}

pub fn validate_output_name(value: &str) -> Result<(), DynamicPlanError> {
    assert!(MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES > 0, "output name byte bound must be positive");
    assert!(
        MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES <= MAX_DYNAMIC_PLAN_STRING_BYTES,
        "output names must fit plan strings"
    );
    let byte_len = len_as_u64("output name", value.len())?;
    if value.is_empty() {
        return invalid_scalar("output name", value, "must not be empty");
    }
    if byte_len > u64::from(MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES) {
        return invalid_scalar("output name", value, "exceeds byte limit");
    }

    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return invalid_scalar("output name", value, "must not be empty");
    };
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
    let byte_len = len_as_u64("blake3 hex", value.len())?;
    if byte_len != len_as_u64("blake3 hex bound", BLAKE3_HEX_BYTES)? {
        return invalid_scalar("blake3 hex", value, "must be exactly 64 lowercase hex bytes");
    }
    if !value.chars().all(is_lower_hex_char) {
        return invalid_scalar("blake3 hex", value, "must contain only lowercase hex characters");
    }
    Ok(())
}

pub fn validate_store_path_string(value: &str, store_prefix: impl AsRef<str>) -> Result<(), DynamicPlanError> {
    let store_prefix = store_prefix.as_ref();
    validate_store_prefix(store_prefix)?;

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
    assert!(!first_component.is_empty(), "validated store path component must not be empty");
    assert!(value.starts_with(store_prefix), "validated store path must retain its prefix");
    Ok(())
}

fn validate_store_prefix(store_prefix: &str) -> Result<(), DynamicPlanError> {
    if store_prefix.is_empty() {
        return invalid_scalar("store prefix", store_prefix, "must not be empty");
    }
    if !store_prefix.starts_with('/') {
        return invalid_scalar("store prefix", store_prefix, "must be absolute");
    }
    if store_prefix.ends_with('/') {
        return invalid_scalar("store prefix", store_prefix, "must not end with slash");
    }
    Ok(())
}

fn validate_id_like(field: &'static str, value: impl AsRef<str>, max_bytes: u32) -> Result<(), DynamicPlanError> {
    assert!(max_bytes > 0, "identifier byte bound must be positive");
    assert!(max_bytes <= MAX_DYNAMIC_PLAN_STRING_BYTES, "identifiers must fit plan strings");
    let value = value.as_ref();
    let byte_len = len_as_u64(field, value.len())?;
    if value.is_empty() {
        return invalid_scalar(field, value, "must not be empty");
    }
    if byte_len > u64::from(max_bytes) {
        return invalid_scalar(field, value, "exceeds byte limit");
    }

    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return invalid_scalar(field, value, "must not be empty");
    };
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

fn invalid_scalar<T>(
    field: &'static str,
    value: impl Into<String>,
    reason: &'static str,
) -> Result<T, DynamicPlanError> {
    Err(DynamicPlanError::InvalidScalar {
        field,
        value: value.into(),
        reason,
    })
}

fn is_lower_hex_char(ch: char) -> bool {
    ch.is_ascii_digit() || ('a'..='f').contains(&ch)
}

fn len_as_u64(field: &'static str, len: usize) -> Result<u64, DynamicPlanError> {
    u64::try_from(len).map_err(|_| DynamicPlanError::ArithmeticOverflow { field })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STORE_PREFIX: &str = "/mantle/store";
    const TEST_STORE_COMPONENT: &str = "00000000000000000000000000000000-dynplan";
    const TEST_STORE_PATH: &str = "/mantle/store/00000000000000000000000000000000-dynplan";
    const TEST_BLAKE3_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const OVER_LIMIT_EXTRA: usize = 1;

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

    fn valid_plan_key_order_variant_json() -> String {
        format!(
            r#"{{
  "provenance": {{ "generator": "test" }},
  "roots": ["unit.main"],
  "units": [{{
    "policy": {{
      "host_paths": "none",
      "store_prefix": "inherit",
      "substitutions": "inherit",
      "sandbox": "inherit"
    }},
    "requested_outputs": ["out"],
    "derivation": {{
      "dynamic_plan_outputs": ["plan"],
      "sandbox": "native",
      "addressing_mode": "content-addressed",
      "fixed_output": null,
      "inputs": [
        {{ "path": "{TEST_STORE_PATH}", "kind": "store_path" }},
        {{ "source": "src.main", "kind": "source" }}
      ],
      "env": {{ "KEY": "VALUE" }},
      "outputs": ["out"],
      "args": ["--build"],
      "system": "x86_64-linux",
      "builder": "{TEST_STORE_PATH}/bin/builder",
      "name": "unit-main"
    }},
    "id": "unit.main"
  }}],
  "sources": [{{ "nar_blake3": "{TEST_BLAKE3_HEX}", "path": "{TEST_STORE_PATH}", "id": "src.main" }}],
  "producer": {{ "goal_hint": "goal-a", "logical_name": "resolver" }},
  "schema": "mantle-plan-v1"
}}"#,
        )
    }

    fn valid_plan() -> DynamicPlanV1 {
        decode_plan_v1(valid_plan_json().as_bytes()).unwrap()
    }

    fn plan_with_extra_collections() -> DynamicPlanV1 {
        let mut plan = valid_plan();
        plan.sources.push(DeclaredSourceInput {
            id: "src.extra".to_string(),
            path: TEST_STORE_PATH.to_string(),
            nar_blake3: None,
        });
        plan.roots.push("unit.extra".to_string());
        plan.provenance.insert("zeta".to_string(), "last".to_string());
        plan.provenance.insert("alpha".to_string(), "first".to_string());
        plan.units.push(extra_unit());
        plan
    }

    fn extra_unit() -> DynamicUnit {
        DynamicUnit {
            id: "unit.extra".to_string(),
            derivation: DynamicDerivation {
                name: "unit-extra".to_string(),
                builder: TEST_STORE_PATH.to_string(),
                system: "x86_64-linux".to_string(),
                args: vec!["--extra".to_string()],
                outputs: vec!["out".to_string(), "dev".to_string()],
                env: std::collections::BTreeMap::from([
                    ("ZED".to_string(), "last".to_string()),
                    ("ALPHA".to_string(), "first".to_string()),
                ]),
                inputs: vec![
                    DynamicInput::UnitOutput {
                        unit: "unit.main".to_string(),
                        output: "out".to_string(),
                    },
                    DynamicInput::StorePath {
                        path: TEST_STORE_PATH.to_string(),
                    },
                    DynamicInput::Source {
                        source: "src.extra".to_string(),
                    },
                ],
                fixed_output: None,
                addressing_mode: AddressingMode::ContentAddressed,
                sandbox: SandboxMode::Native,
                dynamic_plan_outputs: vec!["plan_b".to_string(), "plan_a".to_string()],
            },
            requested_outputs: vec!["out".to_string(), "dev".to_string()],
            policy: DynamicUnitPolicy {
                sandbox: INHERIT_POLICY_VALUE.to_string(),
                substitutions: INHERIT_POLICY_VALUE.to_string(),
                store_prefix: INHERIT_POLICY_VALUE.to_string(),
                host_paths: NO_HOST_PATHS_POLICY_VALUE.to_string(),
            },
        }
    }

    fn over_limit(limit: u32) -> usize {
        usize::try_from(limit).unwrap() + OVER_LIMIT_EXTRA
    }

    fn over_limit_string() -> String {
        "x".repeat(over_limit(MAX_DYNAMIC_PLAN_STRING_BYTES))
    }

    fn validate_err(plan: &DynamicPlanV1) -> DynamicPlanError {
        validate_plan_v1(plan, TEST_STORE_PREFIX).unwrap_err()
    }

    fn expect_limit(err: DynamicPlanError, expected_field: &'static str) {
        assert!(matches!(
            err,
            DynamicPlanError::LimitExceeded { field, .. } if field == expected_field
        ));
    }

    fn expect_invalid_scalar(err: DynamicPlanError, expected_field: &'static str) {
        assert!(matches!(
            err,
            DynamicPlanError::InvalidScalar { field, .. } if field == expected_field
        ));
    }

    fn store_path_input() -> DynamicInput {
        DynamicInput::StorePath {
            path: TEST_STORE_PATH.to_string(),
        }
    }

    #[test]
    fn decode_accepts_valid_plan_shape() {
        let plan = valid_plan();

        assert_eq!(plan.schema, MANTLE_PLAN_V1_SCHEMA);
        assert_eq!(plan.producer.logical_name, "resolver");
        assert_eq!(plan.sources.len(), 1);
        assert_eq!(plan.units.len(), 1);
        assert_eq!(plan.roots, vec!["unit.main"]);
        assert_eq!(plan.units[0].derivation.sandbox, SandboxMode::Native);
    }

    #[test]
    fn canonical_bytes_are_compact_json() {
        let plan = valid_plan();
        let bytes = canonical_plan_v1_bytes(&plan).unwrap();
        let text = String::from_utf8(bytes.clone()).unwrap();
        let decoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(decoded["schema"], MANTLE_PLAN_V1_SCHEMA);
        assert!(!text.contains('\n'));
        assert!(!text.contains(": "));
        assert!(!text.contains(", "));
    }

    #[test]
    fn canonical_digest_ignores_formatting_and_object_key_order() {
        let left = valid_plan();
        let right = decode_plan_v1(valid_plan_key_order_variant_json().as_bytes()).unwrap();

        let left_bytes = canonical_plan_v1_bytes(&left).unwrap();
        let right_bytes = canonical_plan_v1_bytes(&right).unwrap();
        let left_digest = canonical_plan_v1_digest(&left).unwrap();
        let right_digest = canonical_plan_v1_digest(&right).unwrap();
        let direct_digest = blake3::hash(&left_bytes).to_hex().to_string();

        assert_eq!(left_bytes, right_bytes);
        assert_eq!(left_digest, right_digest);
        assert_eq!(left_digest, direct_digest);
        assert_eq!(left_digest.len(), BLAKE3_HEX_BYTES);
        assert!(validate_blake3_hex(&left_digest).is_ok());
    }

    #[test]
    fn canonicalization_sorts_set_like_collections() {
        let left = plan_with_extra_collections();
        let mut right = left.clone();
        right.sources.reverse();
        right.units.reverse();
        right.roots.reverse();
        for unit in &mut right.units {
            unit.requested_outputs.reverse();
            unit.derivation.outputs.reverse();
            unit.derivation.dynamic_plan_outputs.reverse();
            unit.derivation.inputs.reverse();
        }

        let raw_left = serde_json::to_vec(&left).unwrap();
        let raw_right = serde_json::to_vec(&right).unwrap();
        let canonical_left = canonical_plan_v1_bytes(&left).unwrap();
        let canonical_right = canonical_plan_v1_bytes(&right).unwrap();
        let canonical = canonicalize_plan_v1(&right);

        assert_ne!(raw_left, raw_right);
        assert_eq!(canonical_left, canonical_right);
        assert_eq!(canonical_plan_v1_digest(&left).unwrap(), canonical_plan_v1_digest(&right).unwrap());
        assert_eq!(canonical.sources[0].id, "src.extra");
        assert_eq!(canonical.units[0].id, "unit.extra");
        assert_eq!(canonical.roots, vec!["unit.extra", "unit.main"]);
        assert_eq!(canonical.units[0].requested_outputs, vec!["dev", "out"]);
        assert_eq!(canonical.units[0].derivation.outputs, vec!["dev", "out"]);
        assert_eq!(canonical.units[0].derivation.dynamic_plan_outputs, vec!["plan_a", "plan_b"]);
        assert_eq!(canonical.units[0].derivation.inputs[0], DynamicInput::Source {
            source: "src.extra".to_string()
        });
    }

    #[test]
    fn decode_canonical_plan_returns_canonical_plan_bytes_and_digest() {
        let decoded = decode_canonical_plan_v1(valid_plan_key_order_variant_json().as_bytes()).unwrap();
        let canonical_bytes = canonical_plan_v1_bytes(&decoded.plan).unwrap();
        let canonical_digest = canonical_plan_v1_digest(&decoded.plan).unwrap();

        assert_eq!(decoded.bytes, canonical_bytes);
        assert_eq!(decoded.digest, canonical_digest);
        assert_eq!(decoded.plan.units[0].derivation.inputs[0], DynamicInput::Source {
            source: "src.main".to_string()
        });
    }

    #[test]
    fn validate_accepts_valid_plan_and_decode_validated_returns_canonical_digest() {
        let plan = valid_plan();
        let decoded =
            decode_validated_plan_v1(valid_plan_key_order_variant_json().as_bytes(), TEST_STORE_PREFIX).unwrap();

        assert!(validate_plan_v1(&plan, TEST_STORE_PREFIX).is_ok());
        assert_eq!(decoded.digest, canonical_plan_v1_digest(&decoded.plan).unwrap());
        assert_eq!(decoded.bytes, canonical_plan_v1_bytes(&decoded.plan).unwrap());
    }

    #[test]
    fn validate_rejects_unknown_schema_version() {
        let mut plan = valid_plan();
        plan.schema = "mantle-plan-v2".to_string();

        let err = validate_err(&plan);
        expect_invalid_scalar(err, "schema");
    }

    #[test]
    fn validate_rejects_over_limit_units_dependencies_outputs_and_env() {
        let mut too_many_units = valid_plan();
        too_many_units.units = vec![too_many_units.units[0].clone(); over_limit(MAX_DYNAMIC_PLAN_UNITS)];
        expect_limit(validate_err(&too_many_units), "units");

        let mut too_many_dependencies = valid_plan();
        too_many_dependencies.units[0].derivation.inputs =
            vec![store_path_input(); over_limit(MAX_DYNAMIC_PLAN_DEPENDENCIES_PER_UNIT)];
        expect_limit(validate_err(&too_many_dependencies), "unit dependencies");

        let mut too_many_outputs = valid_plan();
        too_many_outputs.units[0].derivation.outputs =
            vec!["out".to_string(); over_limit(MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT)];
        expect_limit(validate_err(&too_many_outputs), "unit outputs");

        let mut too_many_env = valid_plan();
        too_many_env.units[0].derivation.env = (0..over_limit(MAX_DYNAMIC_PLAN_ENV_ENTRIES_PER_UNIT))
            .map(|index| (format!("KEY_{index}"), "value".to_string()))
            .collect();
        expect_limit(validate_err(&too_many_env), "unit environment");
    }

    #[test]
    fn validate_rejects_over_limit_string_field() {
        let mut plan = valid_plan();
        plan.producer.logical_name = over_limit_string();

        let err = validate_err(&plan);
        expect_limit(err, "producer logical name");
    }

    #[test]
    fn decode_rejects_over_limit_json_nesting_depth() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        let mut nested = serde_json::json!("goal-a");
        for _ in 0..MAX_DYNAMIC_PLAN_NESTING_DEPTH {
            nested = serde_json::json!([nested]);
        }
        value.get_mut("producer").unwrap()["goal_hint"] = nested;

        let err = decode_plan_v1(value.to_string().as_bytes()).unwrap_err();
        expect_limit(err, "json nesting depth");
    }

    #[test]
    fn validate_rejects_invalid_output_name_and_store_prefix_reference() {
        let mut invalid_output = valid_plan();
        invalid_output.units[0].derivation.outputs[0] = "Out".to_string();
        expect_invalid_scalar(validate_err(&invalid_output), "output name");

        let mut invalid_builder = valid_plan();
        invalid_builder.units[0].derivation.builder = "/tmp/builder".to_string();
        expect_invalid_scalar(validate_err(&invalid_builder), "store path");
    }

    #[test]
    fn validate_rejects_malformed_source_digest_and_absolute_host_path() {
        let mut invalid_digest = valid_plan();
        invalid_digest.sources[0].nar_blake3 = Some(TEST_BLAKE3_HEX.to_ascii_uppercase());
        expect_invalid_scalar(validate_err(&invalid_digest), "blake3 hex");

        let mut host_path_arg = valid_plan();
        host_path_arg.units[0].derivation.args.push("/tmp/host-tool".to_string());
        expect_invalid_scalar(validate_err(&host_path_arg), "argument");
    }

    #[test]
    fn validate_rejects_policy_widening() {
        let mut plan = valid_plan();
        plan.units[0].policy.host_paths = "/tmp".to_string();

        let err = validate_err(&plan);
        assert!(matches!(err, DynamicPlanError::PolicyWidening {
            field: "policy host paths",
            ..
        }));
    }

    #[test]
    fn decode_rejects_non_native_derivation_sandbox() {
        let mut value: serde_json::Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value["units"][0]["derivation"]["sandbox"] = serde_json::json!("host");

        let err = decode_validated_plan_v1(value.to_string().as_bytes(), TEST_STORE_PREFIX).unwrap_err();
        assert!(matches!(err, DynamicPlanError::JsonDecode { .. }));
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn validate_rejects_duplicate_output_names() {
        let mut duplicate_unit_output = valid_plan();
        duplicate_unit_output.units[0].derivation.outputs.push("out".to_string());
        expect_invalid_scalar(validate_err(&duplicate_unit_output), "unit outputs");

        let mut duplicate_requested_output = valid_plan();
        duplicate_requested_output.units[0].requested_outputs.push("out".to_string());
        expect_invalid_scalar(validate_err(&duplicate_requested_output), "requested outputs");

        let mut duplicate_plan_output = valid_plan();
        duplicate_plan_output.units[0].derivation.dynamic_plan_outputs.push("plan".to_string());
        expect_invalid_scalar(validate_err(&duplicate_plan_output), "dynamic plan outputs");
    }

    #[test]
    fn validate_rejects_empty_duplicate_and_unknown_roots() {
        let mut empty_roots = valid_plan();
        empty_roots.roots.clear();
        expect_invalid_scalar(validate_err(&empty_roots), "roots");

        let mut duplicate_roots = valid_plan();
        duplicate_roots.roots.push("unit.main".to_string());
        expect_invalid_scalar(validate_err(&duplicate_roots), "roots");

        let mut unknown_root = valid_plan();
        unknown_root.roots[0] = "unit.missing".to_string();
        expect_invalid_scalar(validate_err(&unknown_root), "roots");
    }

    #[test]
    fn validate_rejects_duplicate_unit_and_source_ids() {
        let mut duplicate_units = valid_plan();
        duplicate_units.units.push(duplicate_units.units[0].clone());
        expect_invalid_scalar(validate_err(&duplicate_units), "unit id");

        let mut duplicate_sources = valid_plan();
        duplicate_sources.sources.push(duplicate_sources.sources[0].clone());
        expect_invalid_scalar(validate_err(&duplicate_sources), "source id");
    }

    #[test]
    fn validate_rejects_undeclared_source_and_unit_output_refs() {
        let mut undeclared_source = valid_plan();
        undeclared_source.units[0].derivation.inputs[0] = DynamicInput::Source {
            source: "src.missing".to_string(),
        };
        expect_invalid_scalar(validate_err(&undeclared_source), "source dependency");

        let mut unknown_unit = valid_plan();
        unknown_unit.units[0].derivation.inputs.push(DynamicInput::UnitOutput {
            unit: "unit.missing".to_string(),
            output: "out".to_string(),
        });
        expect_invalid_scalar(validate_err(&unknown_unit), "unit output dependency");

        let mut unknown_output = plan_with_extra_collections();
        unknown_output.units[1].derivation.inputs[0] = DynamicInput::UnitOutput {
            unit: "unit.main".to_string(),
            output: "missing".to_string(),
        };
        expect_invalid_scalar(validate_err(&unknown_output), "unit output dependency");
    }

    #[test]
    fn validate_rejects_unit_output_dependency_cycles() {
        let mut plan = plan_with_extra_collections();
        plan.units[0].derivation.inputs.push(DynamicInput::UnitOutput {
            unit: "unit.extra".to_string(),
            output: "out".to_string(),
        });

        let err = validate_err(&plan);
        expect_invalid_scalar(err, "unit dependency graph");
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

    #[test]
    fn dynamic_placeholders_resolve_declared_source_and_unit_output_paths() {
        let source_token = "{{mantle-source:src.main}}";
        let output_token = "{{mantle-unit-output:unit.main:out}}";
        let source_paths = BTreeMap::from([("src.main".to_string(), TEST_STORE_PATH.to_string())]);
        let output_path = format!("{TEST_STORE_PREFIX}/11111111111111111111111111111111-output");
        let output_paths = BTreeMap::from([(("unit.main".to_string(), "out".to_string()), output_path.clone())]);

        let resolved = resolve_dynamic_placeholders(
            &format!("source={source_token};output={output_token}"),
            &source_paths,
            &output_paths,
        )
        .unwrap();

        assert_eq!(resolved, format!("source={TEST_STORE_PATH};output={output_path}"));
        assert!(parse_dynamic_placeholders(&resolved).unwrap().is_empty());
    }

    #[test]
    fn dynamic_placeholder_validation_rejects_malformed_unknown_and_undeclared_tokens() {
        assert!(parse_dynamic_placeholders("{{mantle-source:src.main}").is_err());
        assert!(parse_dynamic_placeholders("{{mantle-secret:value}}").is_err());

        let mut undeclared = valid_plan();
        undeclared.units[0]
            .derivation
            .env
            .insert("SOURCE".to_string(), "{{mantle-source:src.missing}}".to_string());
        expect_invalid_scalar(validate_err(&undeclared), "dynamic placeholder");

        let missing_bindings =
            resolve_dynamic_placeholders("{{mantle-source:src.main}}", &BTreeMap::new(), &BTreeMap::new()).unwrap_err();
        expect_invalid_scalar(missing_bindings, "dynamic placeholder");
    }
}
