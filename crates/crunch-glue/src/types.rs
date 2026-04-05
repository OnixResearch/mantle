//! Rust types that mirror the Nickel Derivation contract.
//! Deserialized from Nickel `Expr` via `to_serde()` (direct) or
//! via JSON export. Fields that can be Nickel enum tags use
//! `NickelString` deserialization to accept both strings and tags.

use serde::Deserialize;
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

/// An input is either a derivation to be built or a pre-existing store path.
///
/// Serde untagged: a JSON string → `Source`, a JSON object → `Derivation`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Input {
    /// A pre-existing store path (e.g., from the seed toolchain).
    Source(String),
    /// A derivation that must be built first.
    Derivation(Box<CrunchDerivation>),
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
