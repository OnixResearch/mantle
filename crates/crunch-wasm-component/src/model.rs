use std::collections::BTreeMap;
use std::path::PathBuf;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::ComponentArtifactAttestation;
use crunch_wasm_component_core::ComponentBuildReport;
use crunch_wasm_component_core::ComponentManifest;
use crunch_wasm_component_core::ComponentReleaseBinding;
use crunch_wasm_component_core::GeneratedInputCandidate;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::PackageMaterialization;
use crunch_wasm_component_core::StoreObject;
use serde::Deserialize;
use serde::Serialize;

pub const PIPELINE_REQUEST_SCHEMA: &str = "mantle-wasm-component-pipeline-request-v1";
pub const PIPELINE_EXECUTION_REPORT_SCHEMA: &str = "mantle-wasm-component-pipeline-execution-report-v1";
pub const TOOLCHAIN_MANIFEST_SCHEMA: &str = "mantle-wasm-component-toolchain-v1";
pub const OCTET_SOURCE_REPOSITORY: &str = "https://github.com/OnixResearch/octet";
pub const OCTET_SOURCE_REVISION: &str = "f43a1d9e81a45b15b830930be9ec5c54bfd6db00";
pub const OCTET_PACKAGE_NAME: &str = "cargo-octet";
pub const OCTET_PACKAGE_VERSION: &str = "0.1.1-rc.1";
pub const OCTET_PROFILE_ID: &str = "portable-component-baseline";
pub const OCTET_MOLTEN_PROFILE_ID: &str = "molten-component-baseline";
pub const OCTET_MOLTEN_EFFECTS_PROFILE_ID: &str = "molten-component-effects-v1";
pub const OCTET_WRPC_REFERENCE_PROFILE_ID: &str = "wrpc-reference";
pub const OCTET_WRPC_REFERENCE_PROFILE_IDENTITY_BLAKE3: &str =
    "1b28d739a99d5fe6de915eb57151fb1c6a71d85d1bd99349ceba514a75184712";
pub const OCTET_WRPC_TELEMETRY_PROFILE_ID: &str = "wrpc-telemetry-reference";
pub const OCTET_WRPC_TELEMETRY_PROFILE_IDENTITY_BLAKE3: &str =
    "e63085dd4f4d006348635ea40589c807282aa8377dd262d244bb6373c507ba20";
pub const OCTET_CONFIG_BLAKE3: &str = "dcc7fec0e9fad86b607c1b3d5ef6ff402b3a38f3f5c5dc543d02f95eec29a396";
pub const OCTET_PROFILE_IDENTITY_BLAKE3: &str = "8c4e752b30431a14a61c25554e101681ce4db73513477c82035f01e8281fab36";
pub const OCTET_MOLTEN_PROFILE_IDENTITY_BLAKE3: &str =
    "c93eb7f4607f0a1223e4eb7e125cb62b562c77af5c7197dda38bfda906933e86";
pub const OCTET_MOLTEN_EFFECTS_PROFILE_IDENTITY_BLAKE3: &str =
    "b2c257ab2acb98f15313488899b092bf6c18efb643b64eb6eb79baa756e7ef08";
pub const OCTET_WASM_TOOLS_COHORT_BLAKE3: &str = "c50e2d7f0e8c49de4a1d44afae196bdf96bb14e67e7de0a153de146a6207449a";

pub const REQUIRED_TOOL_NAMES: [&str; 12] = [
    "cargo",
    "rustc",
    "wasm-component-ld",
    "wkg",
    "wit-bindgen",
    "wasm-tools",
    "wac",
    "wasi-virt",
    "wizer",
    "wasmtime",
    "bwrap",
    "cargo-octet",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPipelineRequest {
    pub schema: String,
    pub manifest: ComponentManifest,
    pub generated_inputs: Vec<GeneratedInputCandidate>,
    pub toolchain_manifest: String,
    pub source_root: String,
    pub wit_relative_path: String,
    pub package_materializations: Vec<PackageMaterialization>,
    pub cargo_component_relative_path: String,
    pub cargo_core_module_relative_path: Option<String>,
    pub composition_dependencies: Vec<CompositionDependency>,
    pub package_resolution_network: bool,
    #[serde(default = "standalone_runtime_validation")]
    pub runtime_validation: crunch_wasm_component_core::RuntimeValidation,
    #[serde(default = "no_runtime_composition")]
    pub runtime_composition: Option<RuntimeComposition>,
    pub runtime_invoke: Option<String>,
    pub expected_runtime_stdout: String,
    pub aot_target: String,
    pub aot_cpu_features: Vec<String>,
    pub aot_configuration_args: Vec<String>,
}

fn standalone_runtime_validation() -> crunch_wasm_component_core::RuntimeValidation {
    crunch_wasm_component_core::RuntimeValidation::StandaloneSmoke
}

fn no_runtime_composition() -> Option<RuntimeComposition> {
    None
}

/// A smoke-only composition; the final portable artifact remains unlinked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeComposition {
    pub source_wac: String,
    pub dependencies: Vec<CompositionDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionDependency {
    pub package: String,
    pub source: CompositionDependencySource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompositionDependencySource {
    CompiledComponent,
    StoreObject {
        path: String,
        digest_blake3: Blake3Identity,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainManifest {
    pub schema: String,
    pub rust_target: String,
    pub octet: OctetRailIdentity,
    pub tools: Vec<ToolRecord>,
    pub cohort_identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OctetRailIdentity {
    pub source_repository: String,
    pub source_revision: String,
    pub package_name: String,
    pub package_version: String,
    pub config_path: String,
    pub config_digest_blake3: Blake3Identity,
    pub profile_id: String,
    pub profile_identity_blake3: Blake3Identity,
    pub wasm_tools_cohort_identity_blake3: Blake3Identity,
    /// Exact selectable profiles in this toolchain cohort, including its default.
    pub profiles: Vec<OctetProfileIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OctetProfileIdentity {
    pub profile_id: String,
    pub profile_identity_blake3: Blake3Identity,
}

// r[impl mantle.wasm_component.profile_selection]
impl OctetRailIdentity {
    /// Resolve one declared profile identity against this cohort without fallback.
    pub fn select_profile(&self, expected: &Blake3Identity) -> Option<&OctetProfileIdentity> {
        self.profiles.iter().find(|profile| &profile.profile_identity_blake3 == expected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolRecord {
    pub name: String,
    pub version: String,
    pub version_output: String,
    pub path: String,
    pub binary_digest_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedToolchain {
    pub root: PathBuf,
    pub manifest: ToolchainManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptToolDependency {
    pub name: String,
    pub path: String,
    pub digest_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolExecutionReceipt {
    pub schema: String,
    pub stage_key: String,
    pub program: String,
    pub program_blake3: Blake3Identity,
    pub args: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub tool_dependencies: Vec<ReceiptToolDependency>,
    pub read_only_inputs: Vec<String>,
    pub network_admitted: bool,
    pub status: String,
    pub stdout_blake3: Blake3Identity,
    pub stderr_blake3: Blake3Identity,
    pub output_blake3: Option<Blake3Identity>,
    pub receipt_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineBlocker {
    pub code: String,
    pub stage_key: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineExecutionReport {
    pub schema: String,
    pub request_blake3: Blake3Identity,
    pub cohort_blake3: Option<Blake3Identity>,
    pub stage_receipts: Vec<ToolExecutionReceipt>,
    pub artifacts: Vec<PipelineArtifact>,
    pub blockers: Vec<PipelineBlocker>,
    pub octet_validations: Vec<OctetValidationEvidence>,
    pub component_report: Option<ComponentBuildReport>,
    pub materialization_bundle: Option<MaterializationBundle>,
    pub component_attestation: Option<ComponentArtifactAttestation>,
    pub release_binding: Option<ComponentReleaseBinding>,
    pub final_status: String,
    pub report_blake3: Blake3Identity,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OctetValidationEvidence {
    pub stage_key: String,
    pub artifact: StoreObject,
    pub receipt: StoreObject,
    pub verification_report: StoreObject,
    pub profile_blake3: Blake3Identity,
    pub cohort_blake3: Blake3Identity,
    pub source_repository: String,
    pub source_revision: String,
    pub package_name: String,
    pub package_version: String,
    pub config_blake3: Blake3Identity,
    pub decision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineArtifact {
    pub role: String,
    pub path: String,
    pub digest_blake3: Blake3Identity,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelinePaths {
    pub evidence_dir: PathBuf,
    pub scratch_parent: PathBuf,
}
