pub mod contracts;
pub mod error;
pub mod eval_trait;
pub mod inventory;
pub mod loader;
pub mod threading;

pub use crunch_eval::Expr as NickelValue;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentSource {
    Module {
        module_name: String,
        role_name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedModule {
    pub module_name: String,
    pub role_names: Vec<String>,
    pub inputs: Vec<String>,
    pub consumes_providers: Vec<String>,
    pub produces_providers: Vec<String>,
    pub priority: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluatedFragment {
    pub machine_name: String,
    pub module_name: String,
    pub source: FragmentSource,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergedConfig {
    pub machine_name: String,
    pub data: serde_json::Value,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum MachineOutcome {
    Fragments { merged_config: MergedConfig },
    Derivations { derivations: Vec<crunch_glue::CrunchDerivation> },
    Build { reports: Vec<serde_json::Value> },
    Failed,
}

#[derive(Debug, Clone)]
pub struct SystemPipelineResult {
    pub machines: BTreeMap<String, MachineOutcome>,
    pub errors: Vec<error::SystemConfigError>,
    pub warnings: Vec<error::SystemConfigWarning>,
}
