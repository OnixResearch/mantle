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
pub const OCTET_SOURCE_REVISION: &str = "86ee46b3b9257b145d2dbeb6ce9d9897607db99c";
pub const OCTET_PACKAGE_NAME: &str = "cargo-octet";
pub const OCTET_PACKAGE_VERSION: &str = "0.1.0";
pub const OCTET_PROFILE_ID: &str = "portable-component-baseline";
pub const OCTET_CONFIG_BLAKE3: &str = "f58715eb73f91d8a6fffd565cc3b030e7f1f8296ec8e91851155d1bb5e52bafd";
pub const OCTET_PROFILE_IDENTITY_BLAKE3: &str = "e8fbf118b283d63a3662af4853a2ce5f4a1777c18fe37e0be0dc1e235fd707bd";
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
    pub composition_dependencies: Vec<CompositionDependency>,
    pub package_resolution_network: bool,
    pub runtime_invoke: Option<String>,
    pub expected_runtime_stdout: String,
    pub aot_target: String,
    pub aot_cpu_features: Vec<String>,
    pub aot_configuration_args: Vec<String>,
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
pub struct ToolExecutionReceipt {
    pub schema: String,
    pub stage_key: String,
    pub program: String,
    pub program_blake3: Blake3Identity,
    pub args: Vec<String>,
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
