//! Rust types that mirror the Nickel Derivation contract.
//! Deserialized from Nickel `Expr` via `to_serde()` (direct) or
//! via JSON export. Fields that can be Nickel enum tags use
//! `NickelString` deserialization to accept both strings and tags.

use std::collections::HashMap;

use crunch_attestation::Claims;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::de::value::MapAccessDeserializer;

use crate::nickel_string::NickelString;

pub const WORKSPACE_POLICY_ENV: &str = "__MANTLE_STATEFUL_WORKSPACE_POLICY";
pub const FINISH_GATES_POLICY_ENV: &str = "__MANTLE_DERIVATION_FINISH_GATES";
pub const PLAN_OUTPUT_BINDINGS_ENV_KEY: &str = "__MANTLE_PLAN_OUTPUT_BINDINGS";
pub const MAX_PLAN_OUTPUT_REFERENCES: usize = 16;

fn valid_id(value: &str, max: usize) -> bool {
    value.len() <= max
        && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value.as_bytes()[1..]
            .iter()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, b'_' | b'.' | b'-'))
}

fn valid_output_name(value: &str) -> bool {
    value.len() <= 64
        && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value.as_bytes()[1..]
            .iter()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, b'_' | b'+' | b'-'))
}

pub fn validate_plan_output_ref(reference: &PlanOutputRef) -> Result<(), String> {
    if !valid_id(&reference.name, 64) {
        return Err(format!("invalid plan-output reference name '{}'", reference.name));
    }
    if !valid_id(&reference.root, 128) {
        return Err(format!("invalid plan-output root '{}'", reference.root));
    }
    for output in [&reference.plan_output, &reference.unit_output] {
        if !valid_output_name(output) {
            return Err(format!("invalid plan-output output name '{output}'"));
        }
    }
    validate_dynamic_plan_outputs(&reference.producer.outputs, &reference.producer.dynamic_plan_outputs)?;
    if !reference.producer.dynamic_plan_outputs.contains(&reference.plan_output) {
        return Err(format!(
            "plan-output '{}' is not a declared dynamic_plan_outputs entry of producer '{}'",
            reference.plan_output, reference.producer.name
        ));
    }
    Ok(())
}

pub fn validate_plan_output_inputs(inputs: &[Input]) -> Result<(), String> {
    let mut names = std::collections::BTreeSet::new();
    for input in inputs {
        if let Input::PlanOutput(reference) = input {
            validate_plan_output_ref(reference)?;
            if !names.insert(reference.name.as_str()) {
                return Err(format!("duplicate plan-output reference name '{}'", reference.name));
            }
            if names.len() > MAX_PLAN_OUTPUT_REFERENCES {
                return Err(format!("plan-output references exceed limit of {MAX_PLAN_OUTPUT_REFERENCES}"));
            }
        }
    }
    Ok(())
}

/// Deserialize a field that may be a Nickel enum tag or a plain string.
fn deserialize_nickel_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    NickelString::deserialize(d).map(|s| s.0)
}

/// A crunch derivation, as described in Nickel.
///
/// This struct is the serde target for Nickel's `Derivation` contract.
/// After deserialization, use `convert()` to turn it into a
/// `nix_compat::Derivation` with computed store paths.
#[derive(Debug, Clone, Serialize)]
pub struct CrunchDerivation {
    pub name: String,
    pub builder: String,
    pub system: String,
    pub args: Vec<String>,
    pub outputs: Vec<String>,
    pub dynamic_plan_outputs: Vec<String>,
    pub env: HashMap<String, String>,
    pub inputs: Vec<Input>,
    pub fixed_output: Option<FixedOutput>,
    pub addressing_mode: String,
    pub provenance: Option<Claims>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub schema: String,
    #[serde(deserialize_with = "deserialize_nickel_string")]
    pub mode: String,
    #[serde(deserialize_with = "deserialize_nickel_string")]
    pub fallback: String,
    pub workspace_id: Option<String>,
    pub snapshot_ref: Option<String>,
    pub guest_path: String,
    pub compatibility: WorkspaceCompatibilityConfig,
    pub quota: WorkspaceQuotaConfig,
    pub retention: WorkspaceRetentionConfig,
    pub scrub: WorkspaceScrubConfig,
    pub snapshot: WorkspaceSnapshotConfig,
    pub clean_rebuild: WorkspaceCleanRebuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCompatibilityConfig {
    pub authority_class: String,
    pub action_class: String,
    pub toolchain_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceQuotaConfig {
    pub bytes_max: u64,
    pub files_max: u32,
    pub snapshots_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRetentionConfig {
    pub workspace_count_max: u32,
    pub idle_generations_max: u64,
    pub age_generations_max: u64,
    pub quarantine_count_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceScrubConfig {
    pub sensitive_paths: Vec<String>,
    pub secret_markers: Vec<String>,
    pub scan_depth_max: u32,
    pub path_bytes_max: u32,
    pub reject_host_paths: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshotConfig {
    pub enabled: bool,
    pub require_clean_scrub: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCleanRebuildConfig {
    pub enabled: bool,
    pub require_declared_inputs: bool,
}

#[derive(Deserialize, Serialize)]
struct FinishGates {
    schema: String,
    version: FinishVersion,
    reference_leak: FinishReferenceLeak,
    relocation: FinishToggle,
    dlopen: FinishDlopen,
}

#[derive(Deserialize, Serialize)]
struct FinishVersion {
    enabled: bool,
    command: Option<Vec<String>>,
    expected: Option<String>,
    environment: std::collections::BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
struct FinishReferenceLeak {
    enabled: bool,
    #[serde(deserialize_with = "deserialize_nickel_string")]
    native: String,
    build_platform_paths: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct FinishToggle {
    enabled: bool,
}

#[derive(Deserialize, Serialize)]
struct FinishDlopen {
    enabled: bool,
    optional_sonames: Vec<String>,
}

#[derive(Deserialize)]
struct RawCrunchDerivation {
    name: String,
    builder: String,
    system: Option<NickelString>,
    args: Option<Vec<String>>,
    outputs: Option<Vec<String>>,
    dynamic_plan_outputs: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    inputs: Option<Vec<Input>>,
    fixed_output: Option<FixedOutput>,
    addressing_mode: Option<NickelString>,
    provenance: Option<Claims>,
    workspace: Option<WorkspaceConfig>,
    finish_gates: Option<FinishGates>,
}

impl<'de> Deserialize<'de> for CrunchDerivation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawCrunchDerivation::deserialize(deserializer)?;
        let outputs = raw.outputs.unwrap_or_else(default_outputs);
        let dynamic_plan_outputs = raw.dynamic_plan_outputs.unwrap_or_else(Vec::new);
        validate_dynamic_plan_outputs(&outputs, &dynamic_plan_outputs).map_err(de::Error::custom)?;
        validate_plan_output_inputs(raw.inputs.as_deref().unwrap_or(&[])).map_err(de::Error::custom)?;
        let mut env = raw.env.unwrap_or_else(HashMap::new);
        if env.contains_key(PLAN_OUTPUT_BINDINGS_ENV_KEY) {
            return Err(de::Error::custom("reserved plan-output bindings environment key is set"));
        }
        if env.contains_key(FINISH_GATES_POLICY_ENV) {
            return Err(de::Error::custom("reserved finish gates environment key is set"));
        }
        if let Some(workspace) = raw.workspace {
            let encoded = serde_json::to_string(&workspace).map_err(de::Error::custom)?;
            if env.insert(WORKSPACE_POLICY_ENV.to_string(), encoded).is_some() {
                return Err(de::Error::custom("reserved workspace policy environment key is set"));
            }
        }
        if let Some(finish_gates) = raw.finish_gates {
            let encoded = serde_json::to_string(&finish_gates).map_err(de::Error::custom)?;
            if env.insert(FINISH_GATES_POLICY_ENV.to_string(), encoded).is_some() {
                return Err(de::Error::custom("reserved finish gates environment key is set"));
            }
        }
        Ok(Self {
            name: raw.name,
            builder: raw.builder,
            system: raw.system.map(|value| value.0).unwrap_or_else(default_system),
            args: raw.args.unwrap_or_else(Vec::new),
            outputs,
            dynamic_plan_outputs,
            env,
            inputs: raw.inputs.unwrap_or_else(Vec::new),
            fixed_output: raw.fixed_output,
            addressing_mode: raw.addressing_mode.map(|value| value.0).unwrap_or_else(default_addressing_mode),
            provenance: raw.provenance,
        })
    }
}

fn default_addressing_mode() -> String {
    "content-addressed".to_string()
}

fn default_system() -> String {
    "x86_64-linux".to_string()
}

fn default_outputs() -> Vec<String> {
    vec!["out".to_string()]
}

pub fn validate_dynamic_plan_outputs(outputs: &[String], dynamic_plan_outputs: &[String]) -> Result<(), String> {
    let output_set = outputs.iter().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
    let mut seen = std::collections::BTreeSet::new();
    for output in dynamic_plan_outputs {
        if !seen.insert(output.as_str()) {
            return Err(format!("duplicate dynamic_plan_outputs entry '{output}'"));
        }
        if !output_set.contains(output.as_str()) {
            return Err(format!(
                "dynamic_plan_outputs entry '{output}' is not declared in outputs [{}]",
                outputs.join(", ")
            ));
        }
    }
    Ok(())
}

/// An input is a derivation, a lazy derivation-file reference, a pre-existing
/// store path, a selected output, or a late-bound dynamic-plan root.
#[derive(Debug, Clone)]
pub enum Input {
    /// A pre-existing store path (e.g., from the seed toolchain).
    Source(String),
    /// A relative Nickel file resolved by the pipeline before conversion.
    DerivationFile(DerivationFileRef),
    /// Pipeline-internal preconverted derivation edge.
    ResolvedDerivation(ResolvedDerivationRef),
    /// A specific output of a multi-output derivation.
    OutputSelection(Box<OutputRef>),
    /// A named root of a producer's declared dynamic plan output.
    PlanOutput(Box<PlanOutputRef>),
    /// A derivation that must be built first (all outputs).
    Derivation(Box<CrunchDerivation>),
}

#[derive(Debug, Clone)]
struct RawInputRecord {
    producer: Option<CrunchDerivation>,
    plan_output: Option<String>,
    root: Option<String>,
    unit_output: Option<String>,
    forbidden_plan_fields: bool,
    derivation_file: Option<String>,
    drv: Option<CrunchDerivation>,
    output: Option<String>,
    name: Option<String>,
    builder: Option<String>,
    system: String,
    args: Vec<String>,
    outputs: Vec<String>,
    dynamic_plan_outputs: Vec<String>,
    env: HashMap<String, String>,
    inputs: Vec<Input>,
    fixed_output: Option<FixedOutput>,
    addressing_mode: String,
    provenance: Option<Claims>,
}

#[derive(Default)]
struct RawInputRecordFields {
    producer: Option<CrunchDerivation>,
    plan_output: Option<String>,
    root: Option<String>,
    unit_output: Option<String>,
    unknown_fields: bool,
    derivation_file: Option<String>,
    drv: Option<CrunchDerivation>,
    output: Option<String>,
    name: Option<String>,
    builder: Option<String>,
    system: Option<NickelString>,
    args: Option<Vec<String>>,
    outputs: Option<Vec<String>>,
    dynamic_plan_outputs: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    inputs: Option<Vec<Input>>,
    fixed_output: Option<FixedOutput>,
    addressing_mode: Option<NickelString>,
    provenance: Option<Claims>,
}

// Keep Nickel enum-tag values streaming. Serde's `flatten` buffers the whole
// record and cannot replay enum inputs in legacy nested derivations.
#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum InputField {
    Producer,
    PlanOutput,
    Root,
    UnitOutput,
    DerivationFile,
    Drv,
    Output,
    Name,
    Builder,
    System,
    Args,
    Outputs,
    DynamicPlanOutputs,
    Env,
    Inputs,
    FixedOutput,
    AddressingMode,
    Provenance,
    #[serde(other)]
    Unknown,
}

impl<'de> Deserialize<'de> for RawInputRecordFields {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct FieldsVisitor;

        impl<'de> Visitor<'de> for FieldsVisitor {
            type Value = RawInputRecordFields;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an input reference or derivation record")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut fields = RawInputRecordFields::default();
                let mut seen = 0_u32;
                macro_rules! read_field {
                    ($field:ident, $index:expr) => {{
                        let bit = 1_u32 << $index;
                        if seen & bit != 0 {
                            return Err(de::Error::duplicate_field(stringify!($field)));
                        }
                        seen |= bit;
                        fields.$field = map.next_value()?;
                    }};
                }
                while let Some(key) = map.next_key::<InputField>()? {
                    match key {
                        InputField::Producer => read_field!(producer, 0),
                        InputField::PlanOutput => read_field!(plan_output, 1),
                        InputField::Root => read_field!(root, 2),
                        InputField::UnitOutput => read_field!(unit_output, 3),
                        InputField::DerivationFile => read_field!(derivation_file, 4),
                        InputField::Drv => read_field!(drv, 5),
                        InputField::Output => read_field!(output, 6),
                        InputField::Name => read_field!(name, 7),
                        InputField::Builder => read_field!(builder, 8),
                        InputField::System => read_field!(system, 9),
                        InputField::Args => read_field!(args, 10),
                        InputField::Outputs => read_field!(outputs, 11),
                        InputField::DynamicPlanOutputs => read_field!(dynamic_plan_outputs, 12),
                        InputField::Env => read_field!(env, 13),
                        InputField::Inputs => read_field!(inputs, 14),
                        InputField::FixedOutput => read_field!(fixed_output, 15),
                        InputField::AddressingMode => read_field!(addressing_mode, 16),
                        InputField::Provenance => read_field!(provenance, 17),
                        InputField::Unknown => {
                            fields.unknown_fields = true;
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(fields)
            }
        }

        deserializer.deserialize_map(FieldsVisitor)
    }
}

impl<'de> Deserialize<'de> for RawInputRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawInputRecordFields::deserialize(deserializer)?;
        let forbidden_plan_fields = raw.derivation_file.is_some()
            || raw.drv.is_some()
            || raw.output.is_some()
            || raw.builder.is_some()
            || raw.system.is_some()
            || raw.args.is_some()
            || raw.outputs.is_some()
            || raw.dynamic_plan_outputs.is_some()
            || raw.env.is_some()
            || raw.inputs.is_some()
            || raw.fixed_output.is_some()
            || raw.addressing_mode.is_some()
            || raw.provenance.is_some()
            || raw.unknown_fields;
        let outputs = raw.outputs.unwrap_or_else(default_outputs);
        let dynamic_plan_outputs = raw.dynamic_plan_outputs.unwrap_or_else(Vec::new);
        validate_dynamic_plan_outputs(&outputs, &dynamic_plan_outputs).map_err(de::Error::custom)?;
        Ok(Self {
            producer: raw.producer,
            plan_output: raw.plan_output,
            root: raw.root,
            unit_output: raw.unit_output,
            forbidden_plan_fields,
            derivation_file: raw.derivation_file,
            drv: raw.drv,
            output: raw.output,
            name: raw.name,
            builder: raw.builder,
            system: raw.system.map(|value| value.0).unwrap_or_else(default_system),
            args: raw.args.unwrap_or_else(Vec::new),
            outputs,
            dynamic_plan_outputs,
            env: raw.env.unwrap_or_else(HashMap::new),
            inputs: raw.inputs.unwrap_or_else(Vec::new),
            fixed_output: raw.fixed_output,
            addressing_mode: raw.addressing_mode.map(|value| value.0).unwrap_or_else(default_addressing_mode),
            provenance: raw.provenance,
        })
    }
}

impl RawInputRecord {
    fn into_plan_output<E: de::Error>(self) -> Result<Input, E> {
        // Unlike ordinary derivations, a binding is a closed record: rejecting
        // even harmless extra fields keeps its identity unambiguous.
        let forbidden = self.forbidden_plan_fields;
        if forbidden {
            return Err(E::custom(
                "plan-output input contains fields outside name, producer, plan_output, root, unit_output",
            ));
        }
        let reference = PlanOutputRef {
            name: self.name.ok_or_else(|| E::custom("plan-output input is missing 'name'"))?,
            producer: self.producer.ok_or_else(|| E::custom("plan-output input is missing 'producer'"))?,
            plan_output: self.plan_output.ok_or_else(|| E::custom("plan-output input is missing 'plan_output'"))?,
            root: self.root.ok_or_else(|| E::custom("plan-output input is missing 'root'"))?,
            unit_output: self.unit_output.ok_or_else(|| E::custom("plan-output input is missing 'unit_output'"))?,
        };
        validate_plan_output_ref(&reference).map_err(E::custom)?;
        Ok(Input::PlanOutput(Box::new(reference)))
    }

    fn into_output_selection<E: de::Error>(self) -> Result<Input, E> {
        let drv = self.drv.ok_or_else(|| E::custom("input output selection is missing 'drv'"))?;
        let output = self.output.ok_or_else(|| E::custom("input output selection is missing 'output'"))?;
        Ok(Input::OutputSelection(Box::new(OutputRef { drv, output })))
    }

    fn into_derivation(self) -> Result<Input, &'static str> {
        let Some(name) = self.name else {
            return Err("input derivation is missing 'name'");
        };
        let Some(builder) = self.builder else {
            return Err("input derivation is missing 'builder'");
        };

        Ok(Input::Derivation(Box::new(CrunchDerivation {
            name,
            builder,
            system: self.system,
            args: self.args,
            outputs: self.outputs,
            dynamic_plan_outputs: self.dynamic_plan_outputs,
            env: self.env,
            inputs: self.inputs,
            fixed_output: self.fixed_output,
            addressing_mode: self.addressing_mode,
            provenance: self.provenance,
        })))
    }

    fn into_input<E: de::Error>(self) -> Result<Input, E> {
        if self.producer.is_some() || self.plan_output.is_some() || self.root.is_some() || self.unit_output.is_some() {
            return self.into_plan_output();
        }
        if let Some(path) = self.derivation_file.as_ref() {
            if self.drv.is_some() || self.name.is_some() || self.builder.is_some() {
                return Err(E::custom("derivation-file input cannot mix derivation_file with derivation fields"));
            }
            return Ok(Input::DerivationFile(DerivationFileRef {
                path: path.clone(),
                output: self.output,
            }));
        }
        if self.drv.is_some() || self.output.is_some() {
            return self.into_output_selection();
        }

        self.into_derivation().map_err(E::custom)
    }
}

struct InputVisitor;

impl<'de> Visitor<'de> for InputVisitor {
    type Value = Input;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("string source path, derivation-file record, output selection record, plan-output record, or derivation record")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Input, E> {
        Ok(Input::Source(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Input, E> {
        Ok(Input::Source(value))
    }

    fn visit_map<A>(self, map: A) -> Result<Input, A::Error>
    where A: MapAccess<'de> {
        let record = RawInputRecord::deserialize(MapAccessDeserializer::new(map))?;
        record.into_input()
    }
}

impl<'de> Deserialize<'de> for Input {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        deserializer.deserialize_any(InputVisitor)
    }
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        match self {
            Self::Source(path) => serializer.serialize_str(path),
            Self::DerivationFile(reference) => reference.serialize(serializer),
            Self::ResolvedDerivation(reference) => reference.serialize(serializer),
            Self::OutputSelection(output_ref) => output_ref.serialize(serializer),
            Self::PlanOutput(reference) => reference.serialize(serializer),
            Self::Derivation(derivation) => derivation.serialize(serializer),
        }
    }
}

/// A late-bound root of a producer's declared dynamic-plan output.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlanOutputRef {
    pub name: String,
    pub producer: CrunchDerivation,
    pub plan_output: String,
    pub root: String,
    pub unit_output: String,
}

/// Relative Nickel file reference resolved by the imperative pipeline shell.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DerivationFileRef {
    #[serde(rename = "derivation_file")]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Pipeline-internal edge to a derivation already converted into the shared cache.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedDerivationRef {
    pub drv_path: String,
    pub outputs: Vec<String>,
}

/// Reference to a specific output of a derivation.
///
/// Used in the `inputs` array as `{ drv = some_pkg, output = "dev" }`.
/// The `drv` field is the full derivation record; `output` is the single
/// output name to depend on.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputRef {
    pub drv: CrunchDerivation,
    pub output: String,
}

/// Fixed-output derivation parameters.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FixedOutput {
    /// Hash in SRI format ("sha256-...") or hex.
    pub hash: String,
    /// Hash algorithm name.
    #[serde(deserialize_with = "deserialize_nickel_string")]
    pub algo: String,
    /// Hashing mode: "flat" or "recursive".
    #[serde(default = "default_hash_mode", deserialize_with = "deserialize_nickel_string")]
    pub mode: String,
}

fn default_hash_mode() -> String {
    "flat".to_string()
}
