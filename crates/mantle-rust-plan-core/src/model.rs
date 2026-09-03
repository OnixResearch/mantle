use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StructuralWorkspaceFacts {
    pub schema: String,
    pub workspace_identity: String,
    pub packages: Vec<PackageFact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct PackageFact {
    pub package_id: String,
    pub name: String,
    pub version: String,
    pub manifest_identity: String,
    pub source_identity: String,
    pub workspace_member: bool,
    pub targets: Vec<TargetFact>,
    pub dependencies: Vec<DependencyFact>,
    pub features: Vec<FeatureFact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct TargetFact {
    pub name: String,
    pub crate_name: String,
    pub kind: TargetKind,
    pub crate_types: Vec<String>,
    pub source_identity: String,
    pub edition: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Library,
    Binary,
    Test,
    BuildScript,
    ProcMacro,
}

impl TargetKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Library => "lib",
            Self::Binary => "bin",
            Self::Test => "test",
            Self::BuildScript => "custom-build",
            Self::ProcMacro => "proc-macro",
        }
    }

    #[must_use]
    pub const fn execution_kind(self) -> ExecutionKind {
        match self {
            Self::BuildScript | Self::ProcMacro => ExecutionKind::Host,
            Self::Library | Self::Binary | Self::Test => ExecutionKind::Target,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct DependencyFact {
    pub package_id: String,
    pub role: DependencyRole,
    pub optional: bool,
    pub activates_on: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyRole {
    Normal,
    Build,
    Development,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct FeatureFact {
    pub name: String,
    pub activations: Vec<FeatureActivation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FeatureActivation {
    Feature { name: String },
    Dependency { package_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolchainFact {
    pub compiler_identity: String,
    pub compiler_version_identity: String,
    pub host_triple: String,
    pub target_triple: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CargoOracleFacts {
    pub metadata_identity: String,
    pub unit_graph_identity: String,
    pub package_ids: Vec<String>,
    pub unit_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustPlanningInput {
    pub workspace: StructuralWorkspaceFacts,
    pub toolchain: ToolchainFact,
    pub oracle: Option<CargoOracleFacts>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustPlanRequest {
    pub workspace_members: Vec<String>,
    pub features: Vec<FeatureRequest>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct FeatureRequest {
    pub package_id: String,
    pub feature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmittedWorkspace {
    pub workspace_identity: String,
    pub packages: Vec<PackageFact>,
    pub selected_package_ids: Vec<String>,
    pub selected_features: BTreeMap<String, Vec<String>>,
    pub toolchain: ToolchainFact,
    pub oracle: Option<CargoOracleFacts>,
    pub profile: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionKind {
    Host,
    Target,
}

impl ExecutionKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Target => "target",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustUnitPlan {
    pub sequence: u32,
    pub unit_id: String,
    pub package_id: String,
    pub target_name: String,
    pub target_kind: TargetKind,
    pub crate_name: String,
    pub execution_kind: ExecutionKind,
    pub selected_triple: String,
    pub selected_features: Vec<String>,
    pub dependency_unit_ids: Vec<String>,
    pub effect: RustUnitEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustUnitEdge {
    pub producer_unit_id: String,
    pub consumer_unit_id: String,
    pub role: DependencyRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustUnitEffect {
    pub effect_id: String,
    pub unit_id: String,
    pub compiler_identity: String,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub input_identities: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub limits: RustUnitLimits,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustUnitLimits {
    pub arguments_max: u32,
    pub environment_entries_max: u32,
    pub input_identities_max: u32,
    pub output_identities_max: u32,
    pub stdout_bytes_max: u64,
    pub stderr_bytes_max: u64,
}

impl Default for RustUnitLimits {
    fn default() -> Self {
        Self {
            arguments_max: crate::MAX_UNIT_ARGUMENTS,
            environment_entries_max: crate::MAX_UNIT_ENVIRONMENT_ENTRIES,
            input_identities_max: crate::MAX_UNIT_INPUTS,
            output_identities_max: crate::MAX_UNIT_OUTPUTS,
            stdout_bytes_max: crate::MAX_UNIT_STDOUT_BYTES,
            stderr_bytes_max: crate::MAX_UNIT_STDERR_BYTES,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustPlanBlocker {
    pub class: String,
    pub subject: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompatibilitySummary {
    pub compatibility_class: String,
    pub status: String,
    pub surface_ids: Vec<String>,
    pub blocker_classes: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustPlanReceiptPreimage {
    pub schema: String,
    pub workspace_identity: String,
    pub compiler_identity: String,
    pub compiler_version_identity: String,
    pub host_triple: String,
    pub target_triple: String,
    pub profile: String,
    pub selected_package_ids: Vec<String>,
    pub selected_features: BTreeMap<String, Vec<String>>,
    pub units: Vec<RustUnitPlan>,
    pub edges: Vec<RustUnitEdge>,
    pub blockers: Vec<RustPlanBlocker>,
    pub compatibility: CompatibilitySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustPlan {
    pub receipt_preimage: RustPlanReceiptPreimage,
    pub receipt_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ObservedArtifact {
    pub name: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustUnitObservation {
    pub effect_id: String,
    pub status_code: i32,
    pub stdout_blake3: String,
    pub stderr_blake3: String,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub artifacts: Vec<ObservedArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CacheObservation {
    pub effect_id: String,
    pub status: CacheObservationStatus,
    pub artifacts: Vec<ObservedArtifact>,
    pub failure_code: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CacheObservationStatus {
    Hit,
    Miss,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "decision", rename_all = "kebab-case")]
pub enum CacheDecision {
    Reuse { artifacts: Vec<ObservedArtifact> },
    Execute,
    Block { blocker: RustPlanBlocker },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustUnitOutcome {
    pub unit_id: String,
    pub effect_id: String,
    pub status: String,
    pub artifacts: Vec<ObservedArtifact>,
    pub blocker: Option<RustPlanBlocker>,
    pub receipt_blake3: String,
}
