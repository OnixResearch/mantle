use std::path::PathBuf;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::ComponentManifest;
use crunch_wasm_component_core::GeneratedInputCandidate;
use crunch_wasm_component_core::PackageMaterialization;
use serde::Deserialize;
use serde::Serialize;

pub const PIPELINE_REQUEST_SCHEMA: &str = "mantle-wasm-component-pipeline-request-v1";
pub const PIPELINE_EXECUTION_REPORT_SCHEMA: &str = "mantle-wasm-component-pipeline-execution-report-v1";
pub const TOOLCHAIN_MANIFEST_SCHEMA: &str = "mantle-wasm-component-toolchain-v1";
pub const OCTET_AUTHORITY_BLOCKER: &str = "octet-wasm-artifact-rail-unavailable";

pub const REQUIRED_TOOL_NAMES: [&str; 11] = [
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
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPipelineRequest {
    pub schema: String,
    pub manifest: ComponentManifest,
    pub generated_inputs: Vec<GeneratedInputCandidate>,
    pub toolchain_manifest: String,
    pub source_root: String,
    pub package_materializations: Vec<PackageMaterialization>,
    pub cargo_component_relative_path: String,
    pub composition_dependencies: Vec<CompositionDependency>,
    pub package_resolution_network: bool,
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
    pub tools: Vec<ToolRecord>,
    pub cohort_identity_blake3: Blake3Identity,
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
    pub final_status: String,
    pub report_blake3: Blake3Identity,
    pub non_claims: Vec<String>,
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
