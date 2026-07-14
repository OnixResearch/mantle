use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const HARDWARE_PROFILE_SCHEMA: &str = "mantle-hardware-profile-v1";
pub const TOOL_COHORT_SCHEMA: &str = "mantle-hardware-tool-cohort-v1";
pub const MANTLE_PLAN_SCHEMA: &str = "mantle-plan-v1";
pub const SMOKE_RESULT_SCHEMA: &str = "mantle-hardware-smoke-result-v1";
pub const EVIDENCE_BUNDLE_SCHEMA: &str = "mantle-hardware-build-evidence-v1";
pub const NIX_SEED_BOUNDARY: &str = "nix-produced-open-source-tool-cohort";
pub const STRICT_SANDBOX_POLICY: &str = "strict-bwrap-no-network";
pub const NO_HOST_PATHS_POLICY: &str = "none";
pub const INHERIT_POLICY: &str = "inherit";
pub const SYSTEM_X86_64_LINUX: &str = "x86_64-linux";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareProfile {
    pub schema: String,
    pub profile_id: String,
    pub selected_target: String,
    pub source_packages: Vec<SourcePackage>,
    pub targets: Vec<HardwareTarget>,
    pub tool_cohort: ToolCohort,
    pub generation: GenerationOptions,
    pub compile_flags: Vec<String>,
    pub link_flags: Vec<String>,
    pub smoke_cases: Vec<SmokeCase>,
    pub expected_outputs: Vec<ExpectedOutput>,
    pub bounds: HardwareBounds,
    pub support_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePackage {
    pub id: String,
    pub locator: String,
    pub revision: String,
    pub recursive_blake3: String,
    pub object_ref: String,
    pub sentinel_blake3: String,
    pub dependencies: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareTarget {
    pub id: String,
    pub top_module: String,
    pub source_packages: Vec<String>,
    pub source_files: Vec<String>,
    pub reference_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolCohort {
    pub schema: String,
    pub identity_blake3: String,
    pub seed_boundary: String,
    pub system: String,
    pub members: Vec<ToolMember>,
    pub closure_paths_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolRole {
    Verilator,
    CxxCompiler,
    Linker,
    RuntimeSupport,
    Shell,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolMember {
    pub role: ToolRole,
    pub package: String,
    pub version: String,
    pub store_path: String,
    pub executable: String,
    pub binary_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationOptions {
    pub language: String,
    pub trace: bool,
    pub optimization: String,
    pub output_prefix: String,
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SmokeCase {
    pub id: String,
    pub input: String,
    pub expected: String,
    pub max_log_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedOutput {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareBounds {
    pub max_source_packages: u32,
    pub max_source_files: u32,
    pub max_generated_units: u32,
    pub max_options: u32,
    pub max_smoke_cases: u32,
    pub max_outputs: u32,
    pub max_text_bytes: u32,
    pub max_plan_bytes: u32,
    pub max_log_bytes: u32,
    pub max_actions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileValidation {
    pub normalized: HardwareProfile,
    pub profile_ref: String,
    pub selected_source_ids: Vec<String>,
    pub selected_source_refs: Vec<String>,
    pub cohort_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObservation {
    pub id: String,
    pub locator: String,
    pub revision: String,
    pub recursive_blake3: String,
    pub sentinel_blake3: String,
    pub declared_edges: Vec<String>,
    pub acquired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceClosureDecision {
    pub selected_source_ids: Vec<String>,
    pub selected_source_refs: Vec<String>,
    pub rejected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedTranslationUnit {
    pub id: String,
    pub relative_path: String,
    pub source_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwarePlanRequest {
    pub profile_ref: String,
    pub cohort_ref: String,
    pub source_refs: Vec<String>,
    pub generated_source_path: String,
    pub generated_source_blake3: String,
    pub shell_builder: String,
    pub cxx_executable: String,
    pub linker_executable: String,
    pub runtime_support_path: String,
    pub tool_closure_paths: Vec<String>,
    pub compile_flags: Vec<String>,
    pub link_flags: Vec<String>,
    pub generated_units: Vec<GeneratedTranslationUnit>,
    pub smoke_cases: Vec<SmokeCase>,
    pub bounds: HardwareBounds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MantlePlanV1 {
    pub schema: String,
    pub producer: PlanProducer,
    pub sources: Vec<DeclaredSourceInput>,
    pub units: Vec<DynamicUnit>,
    pub roots: Vec<String>,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanProducer {
    pub logical_name: String,
    pub goal_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredSourceInput {
    pub id: String,
    pub path: String,
    pub nar_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicUnit {
    pub id: String,
    pub derivation: DynamicDerivation,
    pub requested_outputs: Vec<String>,
    pub policy: DynamicUnitPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicDerivation {
    pub name: String,
    pub builder: String,
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
pub struct DynamicUnitPolicy {
    pub sandbox: String,
    pub substitutions: String,
    pub store_prefix: String,
    pub host_paths: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DynamicInput {
    StorePath { path: String },
    Source { source: String },
    UnitOutput { unit: String, output: String },
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
#[serde(deny_unknown_fields)]
pub struct FixedOutputSpec {
    pub mode: String,
    pub algo: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwarePlan {
    pub plan: MantlePlanV1,
    pub canonical_bytes: Vec<u8>,
    pub plan_blake3: String,
    pub action_graph: Vec<ActionNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionStage {
    Generation,
    Compile,
    Link,
    Smoke,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionNode {
    pub action_ref: String,
    pub unit_id: String,
    pub stage: ActionStage,
    pub direct_input_refs: Vec<String>,
    pub dependency_action_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmokeVerdict {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundedLogRef {
    pub log_ref: String,
    pub byte_count: u32,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareSmokeResult {
    pub schema: String,
    pub case_id: String,
    pub input: String,
    pub expected: String,
    pub observed: String,
    pub verdict: SmokeVerdict,
    pub process_exit_code: i32,
    pub simulator_ref: String,
    pub action_ref: String,
    pub profile_ref: String,
    pub cohort_ref: String,
    pub source_refs: Vec<String>,
    pub stdout: BoundedLogRef,
    pub stderr: BoundedLogRef,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SmokeValidation {
    pub valid: bool,
    pub publishable: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunClass {
    Fresh,
    SelectedSourceChange,
    UnrelatedSourceChange,
    FullSharedHit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageCounts {
    pub requested: u32,
    pub executed: u32,
    pub reused: u32,
    pub invalidated: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunEvidence {
    pub run_class: RunClass,
    pub environment_class: String,
    pub profile_ref: String,
    pub cohort_ref: String,
    pub source_refs: Vec<String>,
    pub action_refs: Vec<String>,
    pub counts: BTreeMap<ActionStage, StageCounts>,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub invalidated_action_refs: Vec<String>,
    pub elapsed_diagnostic_ns: Option<u64>,
    pub elapsed_is_gating: bool,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareEvidenceBundle {
    pub schema: String,
    pub evidence_ref: String,
    pub fresh: RunEvidence,
    pub selected_source_change: RunEvidence,
    pub unrelated_source_change: RunEvidence,
    pub full_shared_hit: RunEvidence,
    pub non_claims: Vec<String>,
}
