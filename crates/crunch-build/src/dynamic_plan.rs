//! Native dynamic build-plan ABI.
//!
//! Pure decoding, canonicalization, and scalar validation for versioned plans.
//! No store I/O, worker mutation, or scheduler registration lives here.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::marker::PhantomData;

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

/// A checked dynamic-plan unit identifier.
///
/// ```
/// # use crunch_build::dynamic_plan::UnitId;
/// let unit = UnitId::new("unit.main")?;
/// assert_eq!(unit.as_str(), "unit.main");
/// # Ok::<(), crunch_build::dynamic_plan::DynamicPlanError>(())
/// ```
///
/// ```compile_fail
/// # use crunch_build::dynamic_plan::{SourceId, UnitId};
/// fn requires_unit(_: UnitId) {}
/// let source = SourceId::new("src.main").unwrap();
/// requires_unit(source);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitId(String);

impl UnitId {
    pub fn new(value: impl Into<String>) -> Result<Self, DynamicPlanError> {
        let value = value.into();
        validate_unit_id(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UnitId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A checked dynamic-plan source identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, DynamicPlanError> {
        let value = value.into();
        validate_source_id(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A checked logical store path under one admitted store prefix.
///
/// This type does not prove that the path exists or that its contents are trusted.
///
/// ```compile_fail
/// # use crunch_build::dynamic_plan::{SourceId, StorePathString};
/// fn requires_path(_: StorePathString) {}
/// let source = SourceId::new("src.main").unwrap();
/// requires_path(source);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorePathString(String);

impl StorePathString {
    pub fn new(value: impl Into<String>, store_prefix: &str) -> Result<Self, DynamicPlanError> {
        let value = value.into();
        validate_store_path_string(&value, store_prefix)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StorePathString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The only non-store builder admitted by a native dynamic plan is the
/// fixed-output fetch service selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicBuilder {
    StorePath(StorePathString),
    FetchUrl,
}

impl DynamicBuilder {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::StorePath(path) => path.as_str(),
            Self::FetchUrl => crate::fetch_build_service::FETCH_BUILDER,
        }
    }
}

impl From<StorePathString> for DynamicBuilder {
    fn from(path: StorePathString) -> Self {
        Self::StorePath(path)
    }
}

/// A checked derivation output name.
///
/// ```compile_fail
/// # use crunch_build::dynamic_plan::{OutputName, UnitId};
/// fn requires_output(_: OutputName) {}
/// let unit = UnitId::new("unit.main").unwrap();
/// requires_output(unit);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputName(String);

impl OutputName {
    pub fn new(value: impl Into<String>) -> Result<Self, DynamicPlanError> {
        let value = value.into();
        validate_output_name(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OutputName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

mod nominal_sealed {
    pub trait Sealed {}
}

/// Marker implemented by each closed dynamic-plan BLAKE3 role.
pub trait Blake3Role: nominal_sealed::Sealed + Clone + Copy + std::fmt::Debug + Eq + Ord + 'static {}

/// Marker for canonical versioned dynamic-plan identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanDigestRole;
impl nominal_sealed::Sealed for PlanDigestRole {}
impl Blake3Role for PlanDigestRole {}

/// Marker for declared source NAR identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NarDigestRole;
impl nominal_sealed::Sealed for NarDigestRole {}
impl Blake3Role for NarDigestRole {}

/// Checked lowercase BLAKE3 text for one compile-time role.
///
/// ```compile_fail
/// # use crunch_build::dynamic_plan::{NarDigest, PlanDigest};
/// fn requires_plan(_: PlanDigest) {}
/// let nar = NarDigest::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").unwrap();
/// requires_plan(nar);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blake3Hex<Role: Blake3Role> {
    hex: String,
    role: PhantomData<fn() -> Role>,
}

impl<Role: Blake3Role> Blake3Hex<Role> {
    pub fn new(value: impl Into<String>) -> Result<Self, DynamicPlanError> {
        let value = value.into();
        validate_blake3_hex(&value)?;
        Ok(Self {
            hex: value,
            role: PhantomData,
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.hex
    }
}

impl Blake3Hex<PlanDigestRole> {
    #[must_use]
    pub fn from_canonical_bytes(bytes: &[u8]) -> Self {
        let hex = blake3::hash(bytes).to_hex().to_string();
        assert_eq!(hex.len(), BLAKE3_HEX_BYTES);
        assert!(validate_blake3_hex(&hex).is_ok());
        Self { hex, role: PhantomData }
    }
}

impl<Role: Blake3Role> std::fmt::Display for Blake3Hex<Role> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub type PlanDigest = Blake3Hex<PlanDigestRole>;
pub type NarDigest = Blake3Hex<NarDigestRole>;

#[path = "dynamic_plan/wire.rs"]
mod wire;
pub use wire::admit_plan_v1;

#[path = "dynamic_plan/slices.rs"]
mod slices;
pub use slices::PlannedSlice;
pub use slices::SliceNodeKind;
pub use slices::SliceRejection;
pub use slices::SliceRejectionKind;
pub use slices::SliceTreeFact;
pub use slices::plan_slices;
#[path = "dynamic_plan/v2.rs"]
mod v2;
pub use v2::CanonicalDynamicPlanV2;
pub use v2::DynamicPlanV2;
pub use v2::MANTLE_PLAN_V2_SCHEMA;
pub use v2::MAX_PLAN_SLICES;
pub use v2::MAX_SLICE_ADMITTED_BYTES;
pub use v2::MAX_SLICE_SUBPATH_BYTES;
pub use v2::MAX_SLICE_SUBPATH_DEPTH;
pub use v2::SliceSource;
pub use v2::SourceV2;
pub use v2::WireDynamicPlanV2;
pub use v2::WireSliceSource;
pub use v2::WireSourceV2;
pub use v2::admit_plan_v2;
pub use v2::canonical_plan_v2_bytes;
pub use v2::decode_plan_v2;
pub use v2::decode_validated_plan_v2;
pub use v2::validate_slice_subpath;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynamicPlaceholder {
    Source { source: SourceId },
    UnitOutput { unit: UnitId, output: OutputName },
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
    pub digest: PlanDigest,
}

/// Admitted dynamic plan. Semantic scalar fields use checked nominal types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicPlanV1 {
    pub schema: String,
    pub producer: PlanProducer,
    pub sources: Vec<DeclaredSourceInput>,
    pub units: Vec<DynamicUnit>,
    pub roots: Vec<UnitId>,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanProducer {
    pub logical_name: String,
    pub goal_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicUnit {
    pub id: UnitId,
    pub derivation: DynamicDerivation,
    pub requested_outputs: Vec<OutputName>,
    pub policy: DynamicUnitPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicUnitPolicy {
    pub sandbox: String,
    pub substitutions: String,
    pub store_prefix: String,
    pub host_paths: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicDerivation {
    pub name: String,
    pub builder: DynamicBuilder,
    pub system: String,
    pub args: Vec<String>,
    pub outputs: Vec<OutputName>,
    pub env: BTreeMap<String, String>,
    pub inputs: Vec<DynamicInput>,
    pub fixed_output: Option<FixedOutputSpec>,
    pub addressing_mode: AddressingMode,
    pub sandbox: SandboxMode,
    pub dynamic_plan_outputs: Vec<OutputName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredSourceInput {
    pub id: SourceId,
    pub path: StorePathString,
    pub nar_blake3: Option<NarDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedOutputSpec {
    pub mode: FixedOutputMode,
    pub algo: FixedOutputHashAlgo,
    pub hash: String,
}

/// Current-shape `mantle-plan-v1` wire record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDynamicPlanV1 {
    pub schema: String,
    pub producer: WirePlanProducer,
    pub sources: Vec<WireDeclaredSourceInput>,
    pub units: Vec<WireDynamicUnit>,
    pub roots: Vec<String>,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WirePlanProducer {
    pub logical_name: String,
    pub goal_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDynamicUnit {
    pub id: String,
    pub derivation: WireDynamicDerivation,
    pub requested_outputs: Vec<String>,
    pub policy: WireDynamicUnitPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDynamicUnitPolicy {
    pub sandbox: String,
    pub substitutions: String,
    pub store_prefix: String,
    pub host_paths: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDynamicDerivation {
    pub name: String,
    pub builder: String,
    pub system: String,
    pub args: Vec<String>,
    pub outputs: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub inputs: Vec<WireDynamicInput>,
    pub fixed_output: Option<WireFixedOutputSpec>,
    pub addressing_mode: AddressingMode,
    pub sandbox: SandboxMode,
    pub dynamic_plan_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireDeclaredSourceInput {
    pub id: String,
    pub path: String,
    pub nar_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireFixedOutputSpec {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicInput {
    StorePath { path: StorePathString },
    Source { source: SourceId },
    UnitOutput { unit: UnitId, output: OutputName },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WireDynamicInput {
    StorePath { path: String },
    Source { source: String },
    UnitOutput { unit: String, output: String },
}

/// Decode the bounded `mantle-plan-v1` wire shape without admitting semantic values.
pub fn decode_plan_v1(bytes: &[u8]) -> Result<WireDynamicPlanV1, DynamicPlanError> {
    decode_wire_plan_v1(bytes)
}

/// Decode the bounded current-shape wire DTO.
pub fn decode_wire_plan_v1(bytes: &[u8]) -> Result<WireDynamicPlanV1, DynamicPlanError> {
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
    let wire = decode_wire_plan_v1(bytes)?;
    let plan = admit_plan_v1(wire, store_prefix)?;
    let canonical = canonicalize_plan_v1(&plan);
    let canonical_bytes = canonical_plan_v1_bytes(&canonical)?;
    let digest = PlanDigest::from_canonical_bytes(&canonical_bytes);

    assert!(!canonical_bytes.is_empty());
    assert_eq!(digest.as_str().len(), BLAKE3_HEX_BYTES);

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

pub fn decode_canonical_plan_v1(bytes: &[u8], store_prefix: &str) -> Result<CanonicalDynamicPlanV1, DynamicPlanError> {
    decode_validated_plan_v1(bytes, store_prefix)
}

pub fn canonical_plan_v1_bytes(plan: &DynamicPlanV1) -> Result<Vec<u8>, DynamicPlanError> {
    let canonical = canonicalize_plan_v1(plan);
    let wire = WireDynamicPlanV1::from(&canonical);
    let bytes = serde_json::to_vec(&wire).map_err(|err| DynamicPlanError::CanonicalJson {
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

pub fn canonical_plan_v1_digest(plan: &DynamicPlanV1) -> Result<PlanDigest, DynamicPlanError> {
    let bytes = canonical_plan_v1_bytes(plan)?;
    let digest = PlanDigest::from_canonical_bytes(&bytes);

    assert_eq!(digest.as_str().len(), BLAKE3_HEX_BYTES);
    assert!(validate_blake3_hex(digest.as_str()).is_ok());

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
    let mut stack = vec![(value, JSON_ROOT_DEPTH)];
    assert_eq!(stack.last().map(|(_, depth)| *depth), Some(JSON_ROOT_DEPTH));
    assert!(stack.capacity() >= stack.len());
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
        validate_source_id(source.id.as_str())?;
        validate_store_path_string(source.path.as_str(), store_prefix)?;
        if let Some(digest) = &source.nar_blake3 {
            validate_blake3_hex(digest.as_str())?;
        }
    }
    Ok(())
}

fn validate_units(units: &[DynamicUnit], store_prefix: &str) -> Result<(), DynamicPlanError> {
    for unit in units {
        validate_unit_id(unit.id.as_str())?;
        validate_dynamic_derivation(&unit.derivation, store_prefix)?;
        validate_output_names("requested outputs", &unit.requested_outputs)?;
        validate_unit_policy(&unit.policy)?;
    }
    Ok(())
}

fn validate_dynamic_derivation(derivation: &DynamicDerivation, store_prefix: &str) -> Result<(), DynamicPlanError> {
    validate_required_string("derivation name", &derivation.name, store_prefix)?;
    match &derivation.builder {
        DynamicBuilder::StorePath(path) => validate_store_path_string(path.as_str(), store_prefix)?,
        DynamicBuilder::FetchUrl => validate_fetch_builder(derivation)?,
    }
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

fn validate_fetch_builder(derivation: &DynamicDerivation) -> Result<(), DynamicPlanError> {
    let reject = |field, value: &str, reason| invalid_scalar(field, value, reason);
    if !matches!(derivation.system.as_str(), "builtin" | "x86_64-linux") {
        return reject("fetch system", &derivation.system, "unsupported fetcher system");
    }
    if !derivation.args.is_empty() {
        return reject("fetch arguments", &derivation.args[0], "builtin fetcher takes no arguments");
    }
    if !derivation.inputs.is_empty() {
        return reject("fetch inputs", "nonempty", "builtin fetcher takes no store inputs");
    }
    if derivation.outputs.len() != 1 || derivation.outputs[0].as_str() != "out" {
        return reject("fetch outputs", "not exactly out", "builtin fetcher requires only out");
    }
    if !derivation.dynamic_plan_outputs.is_empty() {
        return reject("fetch dynamic plan outputs", "nonempty", "fetch result cannot produce dynamic plans");
    }
    if derivation.addressing_mode != AddressingMode::InputAddressed {
        return reject("fetch addressing mode", "content-addressed", "fixed-output fetch uses input-addressed mode");
    }
    let Some(spec) = derivation.fixed_output.as_ref() else {
        return reject("fetch fixed output", "missing", "builtin fetcher requires a sha256 fixed output");
    };
    if spec.algo != FixedOutputHashAlgo::Sha256 {
        return reject("fetch hash algorithm", "not sha256", "builtin fetcher requires sha256");
    }
    let hash = if spec.hash.starts_with("sha256-") {
        nix_compat::nixhash::NixHash::from_sri(&spec.hash).ok().filter(|hash| {
            let canonical = format!("sha256-{}", data_encoding::BASE64.encode(hash.digest_as_bytes()));
            spec.hash == canonical || spec.hash == canonical.trim_end_matches('=')
        })
    } else {
        data_encoding::HEXLOWER.decode(spec.hash.as_bytes()).ok().and_then(|digest| {
            nix_compat::nixhash::NixHash::from_algo_and_digest(nix_compat::nixhash::HashAlgo::Sha256, &digest).ok()
        })
    };
    if !matches!(hash, Some(nix_compat::nixhash::NixHash::Sha256(_))) {
        return reject("fetch hash", &spec.hash, "expected a valid sha256 SRI or lowercase 64-digit hex digest");
    }
    let Some(url) = derivation.env.get("url") else {
        return reject("fetch URL", "missing", "builtin fetcher requires url");
    };
    if url.chars().any(|ch| ch.is_whitespace() || ch.is_control()) || url.contains(&['\\', '{', '}'][..]) {
        return reject("fetch URL", url, "unsafe URL characters");
    }
    let parsed = url::Url::parse(url).ok();
    if !parsed.as_ref().is_some_and(|parsed| {
        matches!(parsed.scheme(), "https" | "http")
            && parsed.host_str().is_some()
            && parsed.username().is_empty()
            && parsed.password().is_none()
            && parsed.fragment().is_none()
    }) {
        return reject("fetch URL", url, "expected an HTTP(S) URL without credentials or fragment");
    }
    let git = derivation.env.get("type").is_some_and(|value| value == "git");
    let allowed: &[&str] = if git { &["url", "type", "rev"] } else { &["url"] };
    for (key, value) in &derivation.env {
        if !allowed.contains(&key.as_str()) {
            return reject("fetch environment key", key, "unsupported fetcher parameter");
        }
        if value.contains(DYNAMIC_PLACEHOLDER_START) {
            return reject("fetch environment value", value, "fetch parameters must be literal");
        }
    }
    if git {
        if spec.mode != FixedOutputMode::Recursive {
            return reject("fetch fixed output mode", "flat", "git tree requires recursive NAR hash");
        }
        let Some(rev) = derivation.env.get("rev") else {
            return reject("fetch revision", "missing", "git fetch requires pinned revision");
        };
        if rev.len() != 40 || !rev.bytes().all(|byte| is_lower_hex_char(char::from(byte))) {
            return reject("fetch revision", rev, "expected a pinned 40-digit lowercase git commit id");
        }
    } else if spec.mode != FixedOutputMode::Flat {
        return reject("fetch fixed output mode", "recursive", "URL archive fetch requires flat hash");
    }
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
            DynamicInput::StorePath { path } => validate_store_path_string(path.as_str(), store_prefix)?,
            DynamicInput::Source { source } => validate_source_id(source.as_str())?,
            DynamicInput::UnitOutput { unit, output } => {
                validate_unit_id(unit.as_str())?;
                validate_output_name(output.as_str())?;
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
            DynamicInput::Source { source } => Some(source.clone()),
            DynamicInput::StorePath { .. } | DynamicInput::UnitOutput { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let declared_outputs = derivation
        .inputs
        .iter()
        .filter_map(|input| match input {
            DynamicInput::UnitOutput { unit, output } => Some((unit.clone(), output.clone())),
            DynamicInput::StorePath { .. } | DynamicInput::Source { .. } => None,
        })
        .collect::<BTreeSet<_>>();

    for value in derivation.args.iter().chain(derivation.env.values()) {
        for placeholder in parse_dynamic_placeholders(value)? {
            match placeholder {
                DynamicPlaceholder::Source { source } if declared_sources.contains(&source) => {}
                DynamicPlaceholder::UnitOutput { unit, output }
                    if declared_outputs.contains(&(unit.clone(), output.clone())) => {}
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
    unit_output_paths: &BTreeMap<(UnitId, OutputName), StorePathString>,
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
        resolved = resolved.replace(&token, path.as_str());
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
        return Ok(DynamicPlaceholder::Source {
            source: SourceId::new(source)?,
        });
    }
    if let Some(unit_output) = body.strip_prefix(DYNAMIC_UNIT_OUTPUT_PLACEHOLDER_PREFIX) {
        let Some((unit, output)) = unit_output.rsplit_once(':') else {
            return invalid_scalar("dynamic placeholder", original, "unit output token is missing its output name");
        };
        return Ok(DynamicPlaceholder::UnitOutput {
            unit: UnitId::new(unit)?,
            output: OutputName::new(output)?,
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

fn validate_output_names(field: &'static str, outputs: &[OutputName]) -> Result<(), DynamicPlanError> {
    let mut seen = BTreeSet::new();
    for output in outputs {
        validate_output_name(output.as_str())?;
        if !seen.insert(output) {
            return invalid_scalar(field, output.as_str(), "contains duplicate output name");
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
        validate_unit_id(root.as_str())?;
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

fn collect_source_ids(sources: &[DeclaredSourceInput]) -> Result<BTreeSet<&SourceId>, DynamicPlanError> {
    let mut source_ids = BTreeSet::new();
    for source in sources {
        if !source_ids.insert(&source.id) {
            return invalid_scalar("source id", source.id.as_str(), "contains duplicate source id");
        }
    }
    assert_eq!(source_ids.len(), sources.len());
    Ok(source_ids)
}

fn collect_unit_outputs(units: &[DynamicUnit]) -> Result<BTreeMap<&UnitId, BTreeSet<&OutputName>>, DynamicPlanError> {
    let mut unit_outputs = BTreeMap::new();
    for unit in units {
        let output_names = unit.derivation.outputs.iter().collect::<BTreeSet<_>>();
        if unit_outputs.insert(&unit.id, output_names).is_some() {
            return invalid_scalar("unit id", unit.id.as_str(), "contains duplicate unit id");
        }
    }
    assert_eq!(unit_outputs.len(), units.len());
    Ok(unit_outputs)
}

fn validate_root_graph(
    roots: &[UnitId],
    unit_outputs: &BTreeMap<&UnitId, BTreeSet<&OutputName>>,
) -> Result<(), DynamicPlanError> {
    if roots.is_empty() {
        return invalid_scalar("roots", "", "must not be empty");
    }

    let mut seen_roots = BTreeSet::new();
    for root in roots {
        if !seen_roots.insert(root) {
            return invalid_scalar("roots", root.as_str(), "contains duplicate root");
        }
        if !unit_outputs.contains_key(root) {
            return invalid_scalar("roots", root.as_str(), "references unknown unit");
        }
    }
    assert_eq!(seen_roots.len(), roots.len());
    Ok(())
}

fn validate_input_graph(
    units: &[DynamicUnit],
    source_ids: &BTreeSet<&SourceId>,
    unit_outputs: &BTreeMap<&UnitId, BTreeSet<&OutputName>>,
) -> Result<(), DynamicPlanError> {
    assert_eq!(unit_outputs.len(), units.len(), "every unit must have an output-set entry");
    assert!(units.iter().all(|unit| !unit.id.as_str().is_empty()), "validated unit IDs must not be empty");
    for unit in units {
        for input in &unit.derivation.inputs {
            match input {
                DynamicInput::StorePath { .. } => {}
                DynamicInput::Source { source } => {
                    if !source_ids.contains(source) {
                        return invalid_scalar(
                            "source dependency",
                            source.as_str(),
                            "references undeclared source input",
                        );
                    }
                }
                DynamicInput::UnitOutput { unit, output } => {
                    let Some(outputs) = unit_outputs.get(unit) else {
                        return invalid_scalar("unit output dependency", unit.as_str(), "references unknown unit");
                    };
                    if !outputs.contains(output) {
                        return invalid_scalar("unit output dependency", output.as_str(), "references unknown output");
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
            .find_map(|(unit, dependencies)| (!dependencies.is_empty()).then_some(unit.as_str()))
            .unwrap_or("<unknown>");
        return invalid_scalar("unit dependency graph", cycle_member, "contains dependency cycle");
    }
    assert_eq!(processed_count, units.len());
    Ok(())
}

fn build_unit_dependency_sets(units: &[DynamicUnit]) -> BTreeMap<&UnitId, BTreeSet<&UnitId>> {
    let dependencies_by_unit = units
        .iter()
        .map(|unit| {
            let dependencies = unit
                .derivation
                .inputs
                .iter()
                .filter_map(|input| match input {
                    DynamicInput::UnitOutput { unit: dependency, .. } => Some(dependency),
                    DynamicInput::StorePath { .. } | DynamicInput::Source { .. } => None,
                })
                .collect::<BTreeSet<_>>();
            (&unit.id, dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(dependencies_by_unit.len(), units.len());
    dependencies_by_unit
}

fn build_dependents_by_dependency<'a>(
    dependencies_by_unit: &BTreeMap<&'a UnitId, BTreeSet<&'a UnitId>>,
) -> BTreeMap<&'a UnitId, BTreeSet<&'a UnitId>> {
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

fn require_nullable_fields_present(value: &Value) -> Result<(), DynamicPlanError> {
    let mut source_entry_count = 0usize;
    let mut source_field_count = 0usize;
    let mut unit_entry_count = 0usize;
    let mut unit_field_count = 0usize;
    if let Some(producer) = value.get("producer").and_then(Value::as_object) {
        require_json_key(producer, RequiredJsonKey {
            key: "goal_hint",
            field: "producer.goal_hint",
        })?;
    }
    if let Some(sources) = value.get("sources").and_then(Value::as_array) {
        source_entry_count = sources.len();
        for source in sources {
            if let Some(source) = source.as_object() {
                require_json_key(source, RequiredJsonKey {
                    key: "nar_blake3",
                    field: "sources[].nar_blake3",
                })?;
                source_field_count = source_field_count.checked_add(1).ok_or(DynamicPlanError::ArithmeticOverflow {
                    field: "required nullable source field count",
                })?;
            }
        }
    }
    if let Some(units) = value.get("units").and_then(Value::as_array) {
        unit_entry_count = units.len();
        for unit in units {
            let Some(derivation) = unit.get("derivation").and_then(Value::as_object) else {
                continue;
            };
            require_json_key(derivation, RequiredJsonKey {
                key: "fixed_output",
                field: "units[].derivation.fixed_output",
            })?;
            unit_field_count = unit_field_count.checked_add(1).ok_or(DynamicPlanError::ArithmeticOverflow {
                field: "required nullable unit field count",
            })?;
        }
    }
    assert!(source_field_count <= source_entry_count);
    assert!(unit_field_count <= unit_entry_count);
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
    let byte_len = len_as_u64("output name", value.len())?;
    if value.is_empty() {
        return invalid_scalar("output name", value, "must not be empty");
    }
    if byte_len > u64::from(MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES) {
        return invalid_scalar("output name", value, "exceeds byte limit");
    }
    assert!(!value.is_empty());
    assert!(byte_len <= u64::from(MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES));

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
    const TEST_CANONICAL_PLAN_BLAKE3: &str = "dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750";
    const TEST_CANONICAL_PLAN_JSON: &[u8] = include_bytes!("../testdata/dynamic-plan-v1-canonical.json");
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

    fn unit_id(value: &str) -> UnitId {
        UnitId::new(value).unwrap()
    }

    fn source_id(value: &str) -> SourceId {
        SourceId::new(value).unwrap()
    }

    fn output_name(value: &str) -> OutputName {
        OutputName::new(value).unwrap()
    }

    fn store_path(value: &str) -> StorePathString {
        StorePathString::new(value, TEST_STORE_PREFIX).unwrap()
    }

    fn valid_wire_plan() -> WireDynamicPlanV1 {
        decode_plan_v1(valid_plan_json().as_bytes()).unwrap()
    }

    fn valid_plan() -> DynamicPlanV1 {
        admit_plan_v1(valid_wire_plan(), TEST_STORE_PREFIX).unwrap()
    }

    fn plan_with_extra_collections() -> DynamicPlanV1 {
        let mut plan = valid_plan();
        plan.sources.push(DeclaredSourceInput {
            id: source_id("src.extra"),
            path: store_path(TEST_STORE_PATH),
            nar_blake3: None,
        });
        plan.roots.push(unit_id("unit.extra"));
        plan.provenance.insert("zeta".to_string(), "last".to_string());
        plan.provenance.insert("alpha".to_string(), "first".to_string());
        plan.units.push(extra_unit());
        plan
    }

    fn extra_unit() -> DynamicUnit {
        DynamicUnit {
            id: unit_id("unit.extra"),
            derivation: DynamicDerivation {
                name: "unit-extra".to_string(),
                builder: store_path(TEST_STORE_PATH).into(),
                system: "x86_64-linux".to_string(),
                args: vec!["--extra".to_string()],
                outputs: vec![output_name("out"), output_name("dev")],
                env: std::collections::BTreeMap::from([
                    ("ZED".to_string(), "last".to_string()),
                    ("ALPHA".to_string(), "first".to_string()),
                ]),
                inputs: vec![
                    DynamicInput::UnitOutput {
                        unit: unit_id("unit.main"),
                        output: output_name("out"),
                    },
                    DynamicInput::StorePath {
                        path: store_path(TEST_STORE_PATH),
                    },
                    DynamicInput::Source {
                        source: source_id("src.extra"),
                    },
                ],
                fixed_output: None,
                addressing_mode: AddressingMode::ContentAddressed,
                sandbox: SandboxMode::Native,
                dynamic_plan_outputs: vec![output_name("plan_b"), output_name("plan_a")],
            },
            requested_outputs: vec![output_name("out"), output_name("dev")],
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
            path: store_path(TEST_STORE_PATH),
        }
    }

    #[test]
    fn decode_accepts_valid_plan_shape() {
        let plan = valid_wire_plan();

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
    fn wire_projection_preserves_frozen_canonical_bytes_and_plan_digest() {
        let plan = valid_plan();
        let canonical_bytes = canonical_plan_v1_bytes(&plan).unwrap();
        let digest = canonical_plan_v1_digest(&plan).unwrap();
        let projected = WireDynamicPlanV1::from(&plan);
        let projected_bytes = serde_json::to_vec(&projected).unwrap();

        assert_eq!(canonical_bytes, TEST_CANONICAL_PLAN_JSON);
        assert_eq!(projected_bytes, TEST_CANONICAL_PLAN_JSON);
        assert_eq!(digest.as_str(), TEST_CANONICAL_PLAN_BLAKE3);
    }

    #[test]
    fn canonical_digest_ignores_formatting_and_object_key_order() {
        let left = valid_plan();
        let right =
            admit_plan_v1(decode_plan_v1(valid_plan_key_order_variant_json().as_bytes()).unwrap(), TEST_STORE_PREFIX)
                .unwrap();

        let left_bytes = canonical_plan_v1_bytes(&left).unwrap();
        let right_bytes = canonical_plan_v1_bytes(&right).unwrap();
        let left_digest = canonical_plan_v1_digest(&left).unwrap();
        let right_digest = canonical_plan_v1_digest(&right).unwrap();
        let direct_digest = blake3::hash(&left_bytes).to_hex().to_string();

        assert_eq!(left_bytes, right_bytes);
        assert_eq!(left_digest, right_digest);
        assert_eq!(left_digest.as_str(), direct_digest);
        assert_eq!(left_digest.as_str().len(), BLAKE3_HEX_BYTES);
        assert!(validate_blake3_hex(left_digest.as_str()).is_ok());
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

        let raw_left = serde_json::to_vec(&WireDynamicPlanV1::from(&left)).unwrap();
        let raw_right = serde_json::to_vec(&WireDynamicPlanV1::from(&right)).unwrap();
        let canonical_left = canonical_plan_v1_bytes(&left).unwrap();
        let canonical_right = canonical_plan_v1_bytes(&right).unwrap();
        let canonical = canonicalize_plan_v1(&right);

        assert_ne!(raw_left, raw_right);
        assert_eq!(canonical_left, canonical_right);
        assert_eq!(canonical_plan_v1_digest(&left).unwrap(), canonical_plan_v1_digest(&right).unwrap());
        assert_eq!(canonical.sources[0].id.as_str(), "src.extra");
        assert_eq!(canonical.units[0].id.as_str(), "unit.extra");
        assert_eq!(canonical.roots.iter().map(UnitId::as_str).collect::<Vec<_>>(), vec!["unit.extra", "unit.main"]);
        assert_eq!(canonical.units[0].requested_outputs.iter().map(OutputName::as_str).collect::<Vec<_>>(), vec![
            "dev", "out"
        ]);
        assert_eq!(canonical.units[0].derivation.outputs.iter().map(OutputName::as_str).collect::<Vec<_>>(), vec![
            "dev", "out"
        ]);
        assert_eq!(
            canonical.units[0]
                .derivation
                .dynamic_plan_outputs
                .iter()
                .map(OutputName::as_str)
                .collect::<Vec<_>>(),
            vec!["plan_a", "plan_b"]
        );
        assert_eq!(canonical.units[0].derivation.inputs[0], DynamicInput::Source {
            source: source_id("src.extra")
        });
    }

    #[test]
    fn decode_canonical_plan_returns_canonical_plan_bytes_and_digest() {
        let decoded =
            decode_canonical_plan_v1(valid_plan_key_order_variant_json().as_bytes(), TEST_STORE_PREFIX).unwrap();
        let canonical_bytes = canonical_plan_v1_bytes(&decoded.plan).unwrap();
        let canonical_digest = canonical_plan_v1_digest(&decoded.plan).unwrap();

        assert_eq!(decoded.bytes, canonical_bytes);
        assert_eq!(decoded.digest, canonical_digest);
        assert_eq!(decoded.plan.units[0].derivation.inputs[0], DynamicInput::Source {
            source: source_id("src.main")
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
            vec![output_name("out"); over_limit(MAX_DYNAMIC_PLAN_OUTPUTS_PER_UNIT)];
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
    fn admission_rejects_invalid_output_name_and_store_prefix_reference() {
        let mut invalid_output: Value = serde_json::from_str(&valid_plan_json()).unwrap();
        invalid_output["units"][0]["derivation"]["outputs"][0] = serde_json::json!("Out");
        let output_error =
            decode_validated_plan_v1(invalid_output.to_string().as_bytes(), TEST_STORE_PREFIX).unwrap_err();
        expect_invalid_scalar(output_error, "output name");

        let mut invalid_builder: Value = serde_json::from_str(&valid_plan_json()).unwrap();
        invalid_builder["units"][0]["derivation"]["builder"] = serde_json::json!("/tmp/builder");
        let builder_error =
            decode_validated_plan_v1(invalid_builder.to_string().as_bytes(), TEST_STORE_PREFIX).unwrap_err();
        expect_invalid_scalar(builder_error, "store path");
    }

    #[test]
    fn admission_rejects_malformed_source_digest_and_absolute_host_path() {
        let mut invalid_digest: Value = serde_json::from_str(&valid_plan_json()).unwrap();
        invalid_digest["sources"][0]["nar_blake3"] = serde_json::json!(TEST_BLAKE3_HEX.to_ascii_uppercase());
        let digest_error =
            decode_validated_plan_v1(invalid_digest.to_string().as_bytes(), TEST_STORE_PREFIX).unwrap_err();
        expect_invalid_scalar(digest_error, "blake3 hex");

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
        duplicate_unit_output.units[0].derivation.outputs.push(output_name("out"));
        expect_invalid_scalar(validate_err(&duplicate_unit_output), "unit outputs");

        let mut duplicate_requested_output = valid_plan();
        duplicate_requested_output.units[0].requested_outputs.push(output_name("out"));
        expect_invalid_scalar(validate_err(&duplicate_requested_output), "requested outputs");

        let mut duplicate_plan_output = valid_plan();
        duplicate_plan_output.units[0].derivation.dynamic_plan_outputs.push(output_name("plan"));
        expect_invalid_scalar(validate_err(&duplicate_plan_output), "dynamic plan outputs");
    }

    #[test]
    fn validate_rejects_empty_duplicate_and_unknown_roots() {
        let mut empty_roots = valid_plan();
        empty_roots.roots.clear();
        expect_invalid_scalar(validate_err(&empty_roots), "roots");

        let mut duplicate_roots = valid_plan();
        duplicate_roots.roots.push(unit_id("unit.main"));
        expect_invalid_scalar(validate_err(&duplicate_roots), "roots");

        let mut unknown_root = valid_plan();
        unknown_root.roots[0] = unit_id("unit.missing");
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
            source: source_id("src.missing"),
        };
        expect_invalid_scalar(validate_err(&undeclared_source), "source dependency");

        let mut unknown_unit = valid_plan();
        unknown_unit.units[0].derivation.inputs.push(DynamicInput::UnitOutput {
            unit: unit_id("unit.missing"),
            output: output_name("out"),
        });
        expect_invalid_scalar(validate_err(&unknown_unit), "unit output dependency");

        let mut unknown_output = plan_with_extra_collections();
        unknown_output.units[1].derivation.inputs[0] = DynamicInput::UnitOutput {
            unit: unit_id("unit.main"),
            output: output_name("missing"),
        };
        expect_invalid_scalar(validate_err(&unknown_output), "unit output dependency");
    }

    #[test]
    fn validate_rejects_unit_output_dependency_cycles() {
        let mut plan = plan_with_extra_collections();
        plan.units[0].derivation.inputs.push(DynamicInput::UnitOutput {
            unit: unit_id("unit.extra"),
            output: output_name("out"),
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
    fn nominal_constructors_admit_valid_distinct_values() {
        let unit = UnitId::new("unit.main").unwrap();
        let source = SourceId::new("src.main").unwrap();
        let output = OutputName::new("out").unwrap();
        let path = StorePathString::new(TEST_STORE_PATH, TEST_STORE_PREFIX).unwrap();
        let nar = NarDigest::new(TEST_BLAKE3_HEX).unwrap();
        let plan = PlanDigest::new(TEST_BLAKE3_HEX).unwrap();

        assert_eq!(unit.as_str(), "unit.main");
        assert_eq!(source.as_str(), "src.main");
        assert_eq!(output.as_str(), "out");
        assert_eq!(path.as_str(), TEST_STORE_PATH);
        assert_eq!(nar.as_str(), TEST_BLAKE3_HEX);
        assert_eq!(plan.as_str(), TEST_BLAKE3_HEX);
    }

    #[test]
    fn nominal_constructors_reject_empty_oversized_control_and_malformed_values() {
        let oversized_id = "a".repeat(over_limit(MAX_DYNAMIC_PLAN_ID_BYTES));
        let oversized_output = "a".repeat(over_limit(MAX_DYNAMIC_PLAN_OUTPUT_NAME_BYTES));

        let empty_error = UnitId::new("").unwrap_err();
        let path_error = StorePathString::new(TEST_STORE_PATH, "/other/store").unwrap_err();
        let digest_error = NarDigest::new("0").unwrap_err();

        assert_eq!(empty_error.to_string(), "invalid unit id ``: must not be empty");
        assert_eq!(
            path_error.to_string(),
            format!("invalid store path `{TEST_STORE_PATH}`: does not start with active store prefix")
        );
        assert_eq!(digest_error.to_string(), "invalid blake3 hex `0`: must be exactly 64 lowercase hex bytes");
        assert!(UnitId::new(oversized_id).is_err());
        assert!(SourceId::new("src\nmain").is_err());
        assert!(OutputName::new(oversized_output).is_err());
        assert!(PlanDigest::new(TEST_BLAKE3_HEX.to_ascii_uppercase()).is_err());
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
        let source_paths = BTreeMap::from([(source_id("src.main"), store_path(TEST_STORE_PATH))]);
        let output_path = format!("{TEST_STORE_PREFIX}/11111111111111111111111111111111-output");
        let output_paths = BTreeMap::from([((unit_id("unit.main"), output_name("out")), store_path(&output_path))]);

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
    fn dynamic_placeholder_validation_rejects_wrong_role_tokens() {
        let mut source_as_unit = valid_plan();
        source_as_unit.units[0].derivation.args.push("{{mantle-unit-output:src.main:out}}".to_string());
        expect_invalid_scalar(validate_err(&source_as_unit), "dynamic placeholder");

        let mut unit_as_source = valid_plan();
        unit_as_source.units[0].derivation.args.push("{{mantle-source:unit.main}}".to_string());
        expect_invalid_scalar(validate_err(&unit_as_source), "dynamic placeholder");
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

    fn slice_wire_plan() -> v2::WireDynamicPlanV2 {
        let mut value: Value = serde_json::from_str(&valid_plan_json()).unwrap();
        value["schema"] = Value::String(v2::MANTLE_PLAN_V2_SCHEMA.to_owned());
        value["sources"] = serde_json::json!([
            {"id":"src.z","producer_output":"sources","subpath":"crate/z","store_name":"crate-same","nar_blake3":TEST_BLAKE3_HEX},
            {"id":"src.main","producer_output":"sources","subpath":"crate/a","store_name":"crate-same","nar_blake3":TEST_BLAKE3_HEX}
        ]);
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn v2_slice_canonical_order_and_digest_bind_expected_content() {
        let wire = slice_wire_plan();
        let raw = serde_json::to_vec(&wire).unwrap();
        let accepted = v2::decode_validated_plan_v2(&raw, TEST_STORE_PREFIX).unwrap();
        let ids = accepted.plan.sources.iter().map(|source| source.id().as_str()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["src.main", "src.z"]);
        let canonical: Value = serde_json::from_slice(&accepted.bytes).unwrap();
        assert_eq!(canonical["sources"][0]["id"], "src.main");
        let mut reordered = wire.clone();
        reordered.sources.reverse();
        let reordered =
            v2::decode_validated_plan_v2(&serde_json::to_vec(&reordered).unwrap(), TEST_STORE_PREFIX).unwrap();
        assert_eq!(accepted.bytes, reordered.bytes);
        assert_eq!(accepted.digest, reordered.digest);
        let mut changed = wire;
        let v2::WireSourceV2::Slice(slice) = &mut changed.sources[0] else {
            panic!("slice fixture")
        };
        slice.nar_blake3 = "f".repeat(BLAKE3_HEX_BYTES);
        let changed = v2::decode_validated_plan_v2(&serde_json::to_vec(&changed).unwrap(), TEST_STORE_PREFIX).unwrap();
        assert_ne!(accepted.digest, changed.digest);
    }

    #[test]
    fn v2_rejects_bad_subpaths_undeclared_output_and_source_conflict() {
        for path in ["/absolute", "../up", "a//b", "a/./b", "a/../b"] {
            assert!(v2::validate_slice_subpath(path).is_err(), "{path}");
        }
        assert!(v2::validate_slice_subpath(&"a/".repeat(33)).is_err());
        assert!(v2::validate_slice_subpath(&"a".repeat(v2::MAX_SLICE_SUBPATH_BYTES as usize + 1)).is_err());
        let mut wire = slice_wire_plan();
        wire.sources.push(wire.sources[0].clone());
        assert_eq!(v2::admit_plan_v2(wire.clone(), TEST_STORE_PREFIX).unwrap().sources.len(), 2);
        let v2::WireSourceV2::Slice(slice) = &mut wire.sources[2] else {
            panic!("slice fixture")
        };
        slice.nar_blake3 = "f".repeat(BLAKE3_HEX_BYTES);
        let err = v2::admit_plan_v2(wire, TEST_STORE_PREFIX).unwrap_err();
        assert!(matches!(err, DynamicPlanError::InvalidScalar {
            reason: "slice-conflict",
            ..
        }));
        let mut v1: Value = serde_json::from_str(&valid_plan_json()).unwrap();
        v1["sources"][0] = serde_json::to_value(slice_wire_plan().sources[0].clone()).unwrap();
        assert!(decode_validated_plan_v1(&serde_json::to_vec(&v1).unwrap(), TEST_STORE_PREFIX).is_err());
        let mut invalid_name = slice_wire_plan();
        let v2::WireSourceV2::Slice(slice) = &mut invalid_name.sources[0] else {
            panic!("slice fixture")
        };
        slice.store_name = "bad/name".to_owned();
        assert!(matches!(
            v2::admit_plan_v2(invalid_name, TEST_STORE_PREFIX),
            Err(DynamicPlanError::InvalidScalar {
                field: "slice store name",
                ..
            })
        ));
        assert!(v2::decode_validated_plan_v2(valid_plan_json().as_bytes(), TEST_STORE_PREFIX).is_err());
    }

    #[test]
    fn v2_enforces_slice_count_depth_and_canonical_byte_ceiling() {
        assert!(v2::validate_slice_subpath(&vec!["a"; v2::MAX_SLICE_SUBPATH_DEPTH as usize].join("/")).is_ok());
        let deep = vec!["a"; v2::MAX_SLICE_SUBPATH_DEPTH as usize + 1].join("/");
        assert!(matches!(
            v2::validate_slice_subpath(&deep),
            Err(DynamicPlanError::LimitExceeded {
                field: "slice subpath depth",
                ..
            })
        ));
        let mut wire = slice_wire_plan();
        let original = wire.sources[0].clone();
        wire.sources.clear();
        for index in 0..=v2::MAX_PLAN_SLICES {
            let v2::WireSourceV2::Slice(mut slice) = original.clone() else {
                panic!("slice fixture")
            };
            slice.id = format!("slice.{index}");
            wire.sources.push(v2::WireSourceV2::Slice(slice));
        }
        assert!(matches!(
            v2::admit_plan_v2(wire, TEST_STORE_PREFIX),
            Err(DynamicPlanError::LimitExceeded {
                field: "slice count",
                ..
            })
        ));
        let mut admitted = v2::admit_plan_v2(slice_wire_plan(), TEST_STORE_PREFIX).unwrap();
        admitted.provenance.insert("oversize".to_owned(), "x".repeat(MAX_DYNAMIC_PLAN_BYTES as usize));
        assert!(matches!(v2::canonical_plan_v2_bytes(&admitted), Err(DynamicPlanError::PlanTooLarge { .. })));
    }

    #[test]
    fn slice_planner_accepts_two_and_rejects_without_partial_plan() {
        use slices::SliceNodeKind;
        use slices::SliceRejectionKind;
        use slices::SliceTreeFact;
        use slices::plan_slices;
        let accepted = v2::admit_plan_v2(slice_wire_plan(), TEST_STORE_PREFIX).unwrap();
        let slices = accepted
            .sources
            .iter()
            .filter_map(|source| match source {
                v2::SourceV2::Slice(slice) => Some(slice.clone()),
                v2::SourceV2::StorePath(_) => None,
            })
            .collect::<Vec<_>>();
        let facts = slices
            .iter()
            .map(|slice| SliceTreeFact {
                source_id: slice.id.clone(),
                producer_output: slice.producer_output.clone(),
                subpath: slice.subpath.clone(),
                kind: SliceNodeKind::Directory,
                traversed_symlink: false,
                observed_nar_blake3: Some(slice.nar_blake3.clone()),
                nar_bytes: 40,
            })
            .collect::<Vec<_>>();
        let outputs = BTreeSet::from([output_name("sources")]);
        let planned = plan_slices(&slices, &outputs, &facts).unwrap();
        assert_eq!(planned.iter().map(|slice| slice.source_id.as_str()).collect::<Vec<_>>(), vec!["src.main", "src.z"]);
        assert_eq!(planned[0].declared_nar_blake3, planned[1].declared_nar_blake3);
        assert_eq!(planned[0].store_name, planned[1].store_name);
        assert_ne!(planned[0].subpath, planned[1].subpath);
        assert_eq!(planned[0].publication_source_id, planned[0].source_id);
        assert_eq!(planned[1].publication_source_id, planned[0].source_id);
        let distinct = slices.iter().map(|slice| slice.id.clone()).collect::<BTreeSet<_>>();
        assert_eq!(distinct.len(), 2);
        assert_eq!(planned.iter().map(|slice| &slice.publication_source_id).collect::<BTreeSet<_>>().len(), 1);
        let mut renamed = slices.clone();
        renamed[0].store_name = "another-name".to_owned();
        let distinct_names = plan_slices(&renamed, &outputs, &facts).unwrap();
        assert_eq!(distinct_names[0].publication_source_id, distinct_names[0].source_id);
        assert_eq!(distinct_names[1].publication_source_id, distinct_names[1].source_id);
        let mut wrong_root = facts.clone();
        wrong_root[0].subpath = "another/root".to_owned();
        assert_eq!(plan_slices(&slices, &outputs, &wrong_root).unwrap_err().kind, SliceRejectionKind::Conflict);
        let mut missing = facts.clone();
        missing[0].kind = SliceNodeKind::Absent;
        assert_eq!(plan_slices(&slices, &outputs, &missing).unwrap_err().kind, SliceRejectionKind::Absent);
        let mut symlink = facts.clone();
        symlink[0].traversed_symlink = true;
        assert_eq!(plan_slices(&slices, &outputs, &symlink).unwrap_err().kind, SliceRejectionKind::SymlinkTraversal);
        let mut mismatch = facts.clone();
        mismatch[0].observed_nar_blake3 = Some(NarDigest::new("f".repeat(BLAKE3_HEX_BYTES)).unwrap());
        assert_eq!(plan_slices(&slices, &outputs, &mismatch).unwrap_err().kind, SliceRejectionKind::DigestMismatch);
        let mut excess = facts;
        excess[0].nar_bytes = v2::MAX_SLICE_ADMITTED_BYTES;
        assert_eq!(plan_slices(&slices, &outputs, &excess).unwrap_err().kind, SliceRejectionKind::Limit);
        assert_eq!(
            plan_slices(&slices, &BTreeSet::new(), &excess).unwrap_err().kind,
            SliceRejectionKind::OutputUndeclared
        );
        let too_many_facts = vec![excess[0].clone(); v2::MAX_PLAN_SLICES as usize + 1];
        assert_eq!(plan_slices(&slices, &outputs, &too_many_facts).unwrap_err().detail, "tree fact count");
    }

    fn pinned_fetch_wire() -> WireDynamicPlanV1 {
        let mut wire = valid_wire_plan();
        let drv = &mut wire.units[0].derivation;
        drv.builder = crate::fetch_build_service::FETCH_BUILDER.to_string();
        drv.system = "x86_64-linux".to_string();
        drv.args.clear();
        drv.inputs.clear();
        drv.addressing_mode = AddressingMode::InputAddressed;
        drv.dynamic_plan_outputs.clear();
        drv.env = BTreeMap::from([("url".to_string(), "https://static.crates.io/crates/memchr/memchr-2.7.6.crate".to_string())]);
        drv.fixed_output = Some(WireFixedOutputSpec {
            mode: FixedOutputMode::Flat,
            algo: FixedOutputHashAlgo::Sha256,
            hash: "0123456789abcdef".repeat(4),
        });
        wire
    }

    #[test]
    fn native_fetch_admits_pinned_archive_and_recursive_git_tree() {
        let flat = admit_plan_v1(pinned_fetch_wire(), TEST_STORE_PREFIX).unwrap();
        assert_eq!(flat.units[0].derivation.builder, DynamicBuilder::FetchUrl);
        assert_eq!(flat.units[0].derivation.fixed_output.as_ref().unwrap().mode, FixedOutputMode::Flat);

        let mut git = pinned_fetch_wire();
        let drv = &mut git.units[0].derivation;
        drv.env.insert("url".to_string(), "https://github.com/example/repo.git".to_string());
        drv.env.insert("type".to_string(), "git".to_string());
        drv.env.insert("rev".to_string(), "0123456789abcdef0123456789abcdef01234567".to_string());
        let spec = drv.fixed_output.as_mut().unwrap();
        spec.mode = FixedOutputMode::Recursive;
        spec.hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string();
        let admitted = admit_plan_v1(git, TEST_STORE_PREFIX).unwrap();
        assert_eq!(admitted.units[0].derivation.builder, DynamicBuilder::FetchUrl);
        assert_eq!(admitted.units[0].derivation.fixed_output.as_ref().unwrap().mode, FixedOutputMode::Recursive);
    }

    #[test]
    fn native_fetch_denies_unpinned_or_unsafe_artifact_requests() {
        for (key, value, field) in [
            ("url", "file:///etc/passwd", "fetch URL"),
            ("url", "https://user:secret@example.org/archive", "fetch URL"),
            ("url", "https://example.org/archive#mutable", "fetch URL"),
            ("type", "archive", "fetch environment key"),
            ("rev", "HEAD", "fetch environment key"),
        ] {
            let mut wire = pinned_fetch_wire();
            wire.units[0].derivation.env.insert(key.to_string(), value.to_string());
            expect_invalid_scalar(admit_plan_v1(wire, TEST_STORE_PREFIX).unwrap_err(), field);
        }
        let mut missing_hash = pinned_fetch_wire();
        missing_hash.units[0].derivation.fixed_output = None;
        expect_invalid_scalar(admit_plan_v1(missing_hash, TEST_STORE_PREFIX).unwrap_err(), "fetch fixed output");

        let mut extra_inputs = pinned_fetch_wire();
        extra_inputs.units[0].derivation.inputs.push(WireDynamicInput::StorePath { path: TEST_STORE_PATH.to_string() });
        expect_invalid_scalar(admit_plan_v1(extra_inputs, TEST_STORE_PREFIX).unwrap_err(), "fetch inputs");

        let mut unpinned_git = pinned_fetch_wire();
        unpinned_git.units[0].derivation.env.insert("type".to_string(), "git".to_string());
        unpinned_git.units[0].derivation.fixed_output.as_mut().unwrap().mode = FixedOutputMode::Recursive;
        expect_invalid_scalar(admit_plan_v1(unpinned_git.clone(), TEST_STORE_PREFIX).unwrap_err(), "fetch revision");
        unpinned_git.units[0].derivation.env.insert("rev".to_string(), "main".to_string());
        expect_invalid_scalar(admit_plan_v1(unpinned_git, TEST_STORE_PREFIX).unwrap_err(), "fetch revision");
    }
}
