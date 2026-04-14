//! Rust types that mirror the Nickel Derivation contract.
//! Deserialized from Nickel `Expr` via `to_serde()` (direct) or
//! via JSON export. Fields that can be Nickel enum tags use
//! `NickelString` deserialization to accept both strings and tags.

use crunch_attestation::Claims;
use serde::Deserialize;
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::de::value::MapAccessDeserializer;

use crate::nickel_string::NickelString;

/// Deserialize a field that may be a Nickel enum tag or a plain string.
fn deserialize_nickel_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    NickelString::deserialize(d).map(|s| s.0)
}

/// A crunch derivation, as described in Nickel.
///
/// This struct is the serde target for Nickel's `Derivation` contract.
/// After deserialization, use `convert()` to turn it into a
/// `nix_compat::Derivation` with computed store paths.
#[derive(Debug, Clone, Deserialize)]
pub struct CrunchDerivation {
    pub name: String,
    pub builder: String,
    #[serde(default = "default_system", deserialize_with = "deserialize_nickel_string")]
    pub system: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_outputs")]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub inputs: Vec<Input>,
    #[serde(default)]
    pub fixed_output: Option<FixedOutput>,
    #[serde(default = "default_addressing_mode", deserialize_with = "deserialize_nickel_string")]
    pub addressing_mode: String,
    #[serde(default)]
    pub provenance: Option<Claims>,
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

/// An input is either a derivation to be built, a pre-existing store path,
/// or a selected output of a multi-output derivation.
///
/// JSON and Nickel direct deserialization both support three shapes:
/// a string → `Source`, a record with `drv` + `output` → `OutputSelection`,
/// and a record with `name` + `builder` → `Derivation`.
#[derive(Debug, Clone)]
pub enum Input {
    /// A pre-existing store path (e.g., from the seed toolchain).
    Source(String),
    /// A specific output of a multi-output derivation.
    OutputSelection(Box<OutputRef>),
    /// A derivation that must be built first (all outputs).
    Derivation(Box<CrunchDerivation>),
}

#[derive(Debug, Clone, Deserialize)]
struct RawInputRecord {
    #[serde(default)]
    drv: Option<CrunchDerivation>,
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    builder: Option<String>,
    #[serde(default = "default_system", deserialize_with = "deserialize_nickel_string")]
    system: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default = "default_outputs")]
    outputs: Vec<String>,
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
    #[serde(default)]
    inputs: Vec<Input>,
    #[serde(default)]
    fixed_output: Option<FixedOutput>,
    #[serde(default = "default_addressing_mode", deserialize_with = "deserialize_nickel_string")]
    addressing_mode: String,
    #[serde(default)]
    provenance: Option<Claims>,
}

impl RawInputRecord {
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
            env: self.env,
            inputs: self.inputs,
            fixed_output: self.fixed_output,
            addressing_mode: self.addressing_mode,
            provenance: self.provenance,
        })))
    }

    fn into_input<E: de::Error>(self) -> Result<Input, E> {
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
        formatter.write_str("string source path, output selection record, or derivation record")
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

/// Reference to a specific output of a derivation.
///
/// Used in the `inputs` array as `{ drv = some_pkg, output = "dev" }`.
/// The `drv` field is the full derivation record; `output` is the single
/// output name to depend on.
#[derive(Debug, Clone, Deserialize)]
pub struct OutputRef {
    pub drv: CrunchDerivation,
    pub output: String,
}

/// Fixed-output derivation parameters.
#[derive(Debug, Clone, Deserialize)]
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
