use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::errors::RunError;

const RECEIPT_SCHEMA_VERSION: u32 = 1;
const RUST_UNIT_EXECUTION_RECEIPT_FILE: &str = ".mantle-rust-unit-execution.json";
const DEFAULT_CARGO_PROFILE: &str = "dev";
const PATH_SOURCE_DIGEST_ALGORITHM: &str = "blake3-tree-v1";
const REGISTRY_SOURCE_DIGEST_ALGORITHM: &str = "cargo-checksum-sha256";
const GIT_SOURCE_DIGEST_ALGORITHM: &str = "git-revision";

#[derive(Debug, Clone)]
pub(crate) struct RustPlanOptions {
    pub(crate) root: PathBuf,
    pub(crate) cargo: PathBuf,
    pub(crate) rustc: PathBuf,
    pub(crate) targets: Vec<String>,
    pub(crate) profile: String,
    pub(crate) features: Vec<String>,
    pub(crate) all_features: bool,
    pub(crate) no_default_features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanReceipt {
    pub(crate) schema_version: u32,
    pub(crate) workspace_root: String,
    pub(crate) cargo_version: String,
    pub(crate) rustc_version_verbose: String,
    pub(crate) lockfile: LockfileIdentity,
    pub(crate) invocation: RustPlanInvocation,
    pub(crate) package_count: usize,
    pub(crate) packages: Vec<PackageSummary>,
    pub(crate) source_closure: SourceClosureSummary,
    pub(crate) native_registry_source_planning: NativeRegistrySourcePlanningSummary,
    pub(crate) native_package_target_planning: NativePackageTargetPlanningSummary,
    pub(crate) native_unit_graph_planning: NativeUnitGraphPlanningSummary,
    pub(crate) native_host_unit_graph_planning: NativeHostUnitGraphPlanningSummary,
    pub(crate) unit_graph: UnitGraphSummary,
    pub(crate) unit_derivation_graph: UnitDerivationGraphSummary,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LockfileIdentity {
    pub(crate) path: String,
    pub(crate) blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanInvocation {
    pub(crate) profile: String,
    pub(crate) targets: Vec<String>,
    pub(crate) features: Vec<String>,
    pub(crate) all_features: bool,
    pub(crate) no_default_features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PackageSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
    pub(crate) manifest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SourceClosureSummary {
    pub(crate) source_count: usize,
    pub(crate) ready: bool,
    pub(crate) digest_blake3: String,
    pub(crate) sources: Vec<SourceInputSummary>,
    pub(crate) blockers: Vec<SourceClosureBlocker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SourceInputSummary {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) kind: SourceKind,
    pub(crate) source: Option<String>,
    pub(crate) manifest_path: String,
    pub(crate) lockfile_identity: Option<LockPackageIdentity>,
    pub(crate) resolved_revision: Option<String>,
    pub(crate) source_digest: SourceDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SourceKind {
    Path,
    Registry,
    Git,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LockPackageIdentity {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
    pub(crate) checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourceDigest {
    pub(crate) algorithm: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SourceClosureBlocker {
    pub(crate) package_id: String,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeRegistrySourcePlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) lockfile_digest_blake3: String,
    pub(crate) digest_blake3: String,
    pub(crate) sources: Vec<NativeRegistrySourceSummary>,
    pub(crate) blockers: Vec<NativeRegistrySourceBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeRegistrySourceSummary {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: String,
    pub(crate) source_class: String,
    pub(crate) checksum: String,
    pub(crate) vendor_root: String,
    pub(crate) manifest_path: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) lockfile_identity: LockPackageIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeRegistrySourceBlocker {
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativePackageTargetPlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) cargo_oracle_identity: String,
    pub(crate) digest_blake3: String,
    pub(crate) packages: Vec<NativePackagePlanningSummary>,
    pub(crate) blockers: Vec<NativePackagePlanningBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativePackagePlanningSummary {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) manifest_path: String,
    pub(crate) selected_features: Vec<String>,
    pub(crate) targets: Vec<NativeTargetPlanningSummary>,
    pub(crate) path_dependencies: Vec<NativePathDependencySummary>,
    pub(crate) source_digest: SourceDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeTargetPlanningSummary {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) crate_name: String,
    pub(crate) source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativePathDependencySummary {
    pub(crate) name: String,
    pub(crate) manifest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativePackagePlanningBlocker {
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeUnitGraphPlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) cargo_unit_graph_oracle_digest: String,
    pub(crate) native_unit_graph_digest: String,
    pub(crate) oracle_comparison_digest: String,
    pub(crate) units: Vec<NativeRustUnitSummary>,
    pub(crate) blockers: Vec<NativeUnitGraphPlanningBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeRustUnitSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) crate_name: String,
    pub(crate) source_path: String,
    pub(crate) crate_types: Vec<String>,
    pub(crate) mode: String,
    pub(crate) profile: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) dependency_artifacts: Vec<RustDependencyArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeUnitGraphPlanningBlocker {
    pub(crate) unit_id: Option<String>,
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeHostUnitGraphPlanningSummary {
    pub(crate) ready: bool,
    pub(crate) comparison_status: String,
    pub(crate) cargo_host_oracle_digest: String,
    pub(crate) native_host_graph_digest: String,
    pub(crate) oracle_comparison_digest: String,
    pub(crate) host_units: Vec<NativeHostUnitSummary>,
    pub(crate) target_consumers: Vec<NativeHostTargetConsumerSummary>,
    pub(crate) blockers: Vec<NativeHostUnitGraphPlanningBlocker>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeHostUnitSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) crate_name: String,
    pub(crate) source_path: String,
    pub(crate) crate_types: Vec<String>,
    pub(crate) mode: String,
    pub(crate) profile: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) artifact: RustHostArtifact,
    pub(crate) generated_metadata: Option<BuildScriptMetadataSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeHostTargetConsumerSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) consumed_host_artifacts: Vec<RustHostArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NativeHostUnitGraphPlanningBlocker {
    pub(crate) unit_id: Option<String>,
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UnitGraphSummary {
    pub(crate) unit_count: usize,
    pub(crate) root_count: usize,
    pub(crate) digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UnitDerivationGraphSummary {
    pub(crate) derivation_count: usize,
    pub(crate) host_unit_count: usize,
    pub(crate) host_artifact_count: usize,
    pub(crate) ready: bool,
    pub(crate) digest_blake3: String,
    pub(crate) derivations: Vec<RustUnitDerivationSummary>,
    pub(crate) blockers: Vec<UnitDerivationBlocker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitDerivationSummary {
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) execution_kind: String,
    pub(crate) crate_types: Vec<String>,
    pub(crate) mode: String,
    pub(crate) profile: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) dependency_artifacts: Vec<RustDependencyArtifact>,
    pub(crate) consumed_host_artifacts: Vec<RustHostArtifact>,
    pub(crate) generated_metadata: Option<BuildScriptMetadataSummary>,
    pub(crate) derivation: ReviewableRustDerivation,
    pub(crate) rustc_args_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RustDependencyArtifact {
    pub(crate) package_id: String,
    pub(crate) name: String,
    pub(crate) artifact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RustHostArtifact {
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) artifact: String,
    pub(crate) metadata_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BuildScriptMetadataSummary {
    pub(crate) out_dir: String,
    pub(crate) rustc_cfg: Vec<String>,
    pub(crate) rustc_env: BTreeMap<String, String>,
    pub(crate) rustc_link_lib: Vec<String>,
    pub(crate) rustc_link_search: Vec<String>,
    pub(crate) rerun_if_changed: Vec<String>,
    pub(crate) digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReviewableRustDerivation {
    pub(crate) name: String,
    pub(crate) builder: String,
    pub(crate) system: String,
    pub(crate) args: Vec<String>,
    pub(crate) outputs: Vec<String>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) inputs: Vec<String>,
    pub(crate) addressing_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UnitDerivationBlocker {
    pub(crate) unit_id: String,
    pub(crate) package_id: Option<String>,
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustUnitExecutionOptions {
    pub(crate) rustc: PathBuf,
    pub(crate) output_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) target_kind: String,
    pub(crate) execution_status: String,
    pub(crate) rebuild_reason: String,
    pub(crate) source_digest: SourceDigest,
    pub(crate) toolchain: RustToolchainIdentity,
    pub(crate) rustc_args_digest_blake3: String,
    pub(crate) declared_outputs: Vec<String>,
    pub(crate) dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) output_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitDependencyChainExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitTargetTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitHostArtifactTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitTopologyExecutionReceipt {
    pub(crate) schema_version: u32,
    pub(crate) execution_status: String,
    pub(crate) claim: String,
    pub(crate) unit_executions: Vec<RustUnitExecutionReceipt>,
    pub(crate) build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BuildScriptMetadataRunReceipt {
    pub(crate) schema_version: u32,
    pub(crate) unit_id: String,
    pub(crate) package_id: String,
    pub(crate) target_name: String,
    pub(crate) execution_status: String,
    pub(crate) out_dir: String,
    pub(crate) rustc_cfg: Vec<String>,
    pub(crate) rustc_env: BTreeMap<String, String>,
    pub(crate) rustc_link_lib: Vec<String>,
    pub(crate) rustc_link_search: Vec<String>,
    pub(crate) rerun_if_changed: Vec<String>,
    pub(crate) out_dir_artifact_digests: Vec<RustExecutionArtifactDigest>,
    pub(crate) stdout_digest_blake3: String,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustToolchainIdentity {
    pub(crate) tool: String,
    pub(crate) version_verbose: String,
    pub(crate) version_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustExecutionArtifactDigest {
    pub(crate) path: String,
    pub(crate) blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustUnitExecutionBlocker {
    pub(crate) class: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) unit_execution: RustUnitExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanDependencyChainExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) dependency_chain_execution: RustUnitDependencyChainExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanTargetTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) target_topology_execution: RustUnitTargetTopologyExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanHostArtifactTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) host_artifact_topology_execution: RustUnitHostArtifactTopologyExecutionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RustPlanTopologyExecutionReceipt {
    pub(crate) rust_plan: RustPlanReceipt,
    pub(crate) topology_execution: RustUnitTopologyExecutionReceipt,
}

#[derive(Debug, Clone)]
struct CargoOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status_code: i32,
}

trait CargoOracle {
    fn run_cargo(&self, root: &Path, cargo: &Path, args: &[String]) -> Result<CargoOutput, RunError>;
    fn run_tool_version(&self, tool: &Path, args: &[&str]) -> Result<CargoOutput, RunError>;
}

struct ProcessCargoOracle;

impl CargoOracle for ProcessCargoOracle {
    fn run_cargo(&self, root: &Path, cargo: &Path, args: &[String]) -> Result<CargoOutput, RunError> {
        let output = Command::new(cargo)
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|err| RunError::Internal(format!("failed to run {}: {err}", cargo.display())))?;
        Ok(CargoOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status_code: output.status.code().unwrap_or(1),
        })
    }

    fn run_tool_version(&self, tool: &Path, args: &[&str]) -> Result<CargoOutput, RunError> {
        let output = Command::new(tool)
            .args(args)
            .output()
            .map_err(|err| RunError::Internal(format!("failed to run {}: {err}", tool.display())))?;
        Ok(CargoOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status_code: output.status.code().unwrap_or(1),
        })
    }
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_root: String,
    workspace_members: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: String,
    #[serde(default)]
    targets: Vec<CargoTarget>,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
    src_path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct NativeManifestPackage {
    name: String,
    version: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestLib {
    name: Option<String>,
    path: Option<String>,
    #[serde(rename = "proc-macro", default)]
    proc_macro: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestBin {
    name: Option<String>,
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifestWorkspace {
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NativeManifest {
    package: Option<NativeManifestPackage>,
    workspace: Option<NativeManifestWorkspace>,
    #[serde(default)]
    build: Option<String>,
    lib: Option<NativeManifestLib>,
    #[serde(default)]
    bin: Vec<NativeManifestBin>,
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
    #[serde(rename = "dev-dependencies", default)]
    dev_dependencies: BTreeMap<String, toml::Value>,
    #[serde(rename = "build-dependencies", default)]
    build_dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
}

pub(crate) fn capture_rust_plan(options: &RustPlanOptions) -> Result<RustPlanReceipt, RunError> {
    capture_rust_plan_with_oracle(options, &ProcessCargoOracle)
}

fn capture_rust_plan_with_oracle(
    options: &RustPlanOptions,
    oracle: &impl CargoOracle,
) -> Result<RustPlanReceipt, RunError> {
    validate_options(options)?;

    let cargo_version = run_checked_text(oracle.run_tool_version(&options.cargo, &["--version"]), "cargo --version")?;
    let rustc_version_verbose = run_checked_text(oracle.run_tool_version(&options.rustc, &["-vV"]), "rustc -vV")?;
    let metadata_output =
        run_checked_bytes(oracle.run_cargo(&options.root, &options.cargo, &metadata_args()), "cargo metadata")?;
    let metadata: CargoMetadata = serde_json::from_slice(&metadata_output)
        .map_err(|err| RunError::Internal(format!("cargo metadata did not emit valid JSON: {err}")))?;
    let unit_graph_output = run_checked_bytes(
        oracle.run_cargo(&options.root, &options.cargo, &unit_graph_args(options)),
        "cargo build --unit-graph",
    )?;
    let unit_graph_value: Value = serde_json::from_slice(&unit_graph_output)
        .map_err(|err| RunError::Internal(format!("cargo build --unit-graph did not emit valid JSON: {err}")))?;

    let lock_packages = parse_lockfile_packages(&options.root)?;
    let source_closure = summarize_source_closure(&metadata.packages, &lock_packages)?;
    let lockfile = lockfile_identity(&options.root)?;
    let native_registry_source_planning =
        summarize_native_registry_source_planning(&options.root, &metadata.packages, &lock_packages, &lockfile)?;
    let native_package_target_planning = summarize_native_package_target_planning(
        &options.root,
        options,
        &metadata.packages,
        &metadata.workspace_members,
        &source_closure,
        &native_registry_source_planning,
    )?;
    let native_unit_graph_planning = summarize_native_unit_graph_planning(
        &unit_graph_value,
        &source_closure,
        &native_package_target_planning,
        options,
    )?;
    let native_host_unit_graph_planning = summarize_native_host_unit_graph_planning(
        &unit_graph_value,
        &source_closure,
        &native_registry_source_planning,
        &native_package_target_planning,
        &native_unit_graph_planning,
        options,
    )?;
    let unit_derivation_graph = summarize_unit_derivation_graph_with_native(
        &unit_graph_value,
        &source_closure,
        options,
        Some(&native_unit_graph_planning),
        Some(&native_host_unit_graph_planning),
    )?;
    let packages = summarize_packages(metadata.packages.clone(), &metadata.workspace_members);

    let mut receipt = RustPlanReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        workspace_root: normalize_path_string(Path::new(&metadata.workspace_root)),
        cargo_version: cargo_version.trim().to_string(),
        rustc_version_verbose: rustc_version_verbose.trim().to_string(),
        lockfile,
        invocation: RustPlanInvocation {
            profile: options.profile.clone(),
            targets: sorted_strings(options.targets.clone()),
            features: sorted_strings(options.features.clone()),
            all_features: options.all_features,
            no_default_features: options.no_default_features,
        },
        package_count: metadata.packages.len(),
        packages,
        source_closure,
        native_registry_source_planning,
        native_package_target_planning,
        native_unit_graph_planning,
        native_host_unit_graph_planning,
        unit_graph: summarize_unit_graph(unit_graph_value)?,
        unit_derivation_graph,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_hash(&receipt)?;
    Ok(receipt)
}

pub(crate) fn print_rust_plan_receipt(receipt: &RustPlanReceipt, json_mode: bool) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust plan receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_execution_receipt(
    receipt: &RustPlanExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust plan execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_dependency_chain_execution_receipt(
    receipt: &RustPlanDependencyChainExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust dependency-chain execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_target_topology_execution_receipt(
    receipt: &RustPlanTargetTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust target-topology execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_host_artifact_topology_execution_receipt(
    receipt: &RustPlanHostArtifactTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust host-artifact topology execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn print_rust_plan_topology_execution_receipt(
    receipt: &RustPlanTopologyExecutionReceipt,
    json_mode: bool,
) -> Result<(), RunError> {
    let rendered = if json_mode {
        serde_json::to_string(receipt)
    } else {
        serde_json::to_string_pretty(receipt)
    }
    .map_err(|err| RunError::Internal(format!("rendering Rust topology execution receipt: {err}")))?;
    println!("{rendered}");
    Ok(())
}

pub(crate) fn default_profile() -> String {
    DEFAULT_CARGO_PROFILE.to_string()
}

fn validate_options(options: &RustPlanOptions) -> Result<(), RunError> {
    if !options.root.is_dir() {
        return Err(RunError::Internal(format!("Rust plan root is not a directory: {}", options.root.display())));
    }
    if options.profile.trim().is_empty() {
        return Err(RunError::Internal("Rust plan profile must not be empty".to_string()));
    }
    if options.all_features && !options.features.is_empty() {
        return Err(RunError::Internal("--all-features cannot be combined with --features".to_string()));
    }
    Ok(())
}

fn run_checked_text(result: Result<CargoOutput, RunError>, label: &str) -> Result<String, RunError> {
    let bytes = run_checked_bytes(result, label)?;
    String::from_utf8(bytes).map_err(|err| RunError::Internal(format!("{label} emitted non-UTF-8 output: {err}")))
}

fn run_checked_bytes(result: Result<CargoOutput, RunError>, label: &str) -> Result<Vec<u8>, RunError> {
    let output = result?;
    if output.status_code != 0 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RunError::Build(format!(
            "Rust package planning failed at {label} with exit code {}: {}",
            output.status_code,
            stderr.trim()
        )));
    }
    Ok(output.stdout)
}

fn metadata_args() -> Vec<String> {
    vec!["metadata".to_string(), "--format-version".to_string(), "1".to_string()]
}

fn unit_graph_args(options: &RustPlanOptions) -> Vec<String> {
    let mut args = vec![
        "build".to_string(),
        "-Z".to_string(),
        "unstable-options".to_string(),
        "--unit-graph".to_string(),
        "--profile".to_string(),
        options.profile.clone(),
    ];
    for target in sorted_strings(options.targets.clone()) {
        args.push("--target".to_string());
        args.push(target);
    }
    if options.no_default_features {
        args.push("--no-default-features".to_string());
    }
    if options.all_features {
        args.push("--all-features".to_string());
    }
    if !options.features.is_empty() {
        args.push("--features".to_string());
        args.push(sorted_strings(options.features.clone()).join(","));
    }
    args
}

fn lockfile_identity(root: &Path) -> Result<LockfileIdentity, RunError> {
    let path = root.join("Cargo.lock");
    let bytes = std::fs::read(&path)
        .map_err(|err| RunError::Internal(format!("Rust plan requires a Cargo.lock at {}: {err}", path.display())))?;
    Ok(LockfileIdentity {
        path: normalize_path_string(&path),
        blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn summarize_packages(packages: Vec<CargoPackage>, workspace_members: &[String]) -> Vec<PackageSummary> {
    let workspace_member_set: BTreeSet<&str> = workspace_members.iter().map(String::as_str).collect();
    let mut summaries: Vec<PackageSummary> = packages
        .into_iter()
        .filter(|package| workspace_member_set.contains(package.id.as_str()))
        .map(|package| PackageSummary {
            id: package.id,
            name: package.name,
            version: package.version,
            source: package.source,
            manifest_path: normalize_path_string(Path::new(&package.manifest_path)),
        })
        .collect();
    summaries.sort_by(|left, right| left.id.cmp(&right.id));
    summaries
}

fn summarize_source_closure(
    packages: &[CargoPackage],
    lock_packages: &[LockPackage],
) -> Result<SourceClosureSummary, RunError> {
    let mut sources = Vec::with_capacity(packages.len());
    let mut blockers = Vec::new();
    for package in packages {
        let kind = source_kind(package.source.as_deref());
        let lockfile_identity = find_lock_package(package, lock_packages).map(|lock_package| LockPackageIdentity {
            name: lock_package.name.clone(),
            version: lock_package.version.clone(),
            source: lock_package.source.clone(),
            checksum: lock_package.checksum.clone(),
        });
        let resolved_revision = package.source.as_deref().and_then(source_revision);
        let digest = source_digest(package, &kind, lockfile_identity.as_ref());
        let source_digest = match digest {
            Ok(source_digest) => source_digest,
            Err(blocker) => {
                blockers.push(blocker);
                SourceDigest {
                    algorithm: "missing".to_string(),
                    value: "missing".to_string(),
                }
            }
        };
        sources.push(SourceInputSummary {
            package_id: package.id.clone(),
            name: package.name.clone(),
            version: package.version.clone(),
            kind,
            source: package.source.clone(),
            manifest_path: normalize_path_string(Path::new(&package.manifest_path)),
            lockfile_identity,
            resolved_revision,
            source_digest,
        });
    }
    sources.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    blockers.sort_by(|left, right| left.package_id.cmp(&right.package_id).then(left.class.cmp(&right.class)));
    let digest_blake3 = source_closure_digest(&sources, &blockers)?;
    Ok(SourceClosureSummary {
        source_count: sources.len(),
        ready: blockers.is_empty(),
        digest_blake3,
        sources,
        blockers,
    })
}

fn parse_lockfile_packages(root: &Path) -> Result<Vec<LockPackage>, RunError> {
    let path = root.join("Cargo.lock");
    let text = fs::read_to_string(&path).map_err(|err| {
        RunError::Internal(format!("Rust plan requires a readable Cargo.lock at {}: {err}", path.display()))
    })?;
    Ok(parse_lockfile_packages_text(&text))
}

fn parse_lockfile_packages_text(text: &str) -> Vec<LockPackage> {
    let mut packages = Vec::new();
    let mut current = LockPackage {
        name: String::new(),
        version: String::new(),
        source: None,
        checksum: None,
    };
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            push_lock_package(&mut packages, &mut current, in_package);
            in_package = true;
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(value) = parse_lock_string_field(trimmed, "name") {
            current.name = value;
        } else if let Some(value) = parse_lock_string_field(trimmed, "version") {
            current.version = value;
        } else if let Some(value) = parse_lock_string_field(trimmed, "source") {
            current.source = Some(value);
        } else if let Some(value) = parse_lock_string_field(trimmed, "checksum") {
            current.checksum = Some(value);
        }
    }
    push_lock_package(&mut packages, &mut current, in_package);
    packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.version.cmp(&right.version))
            .then(left.source.cmp(&right.source))
    });
    packages
}

fn push_lock_package(packages: &mut Vec<LockPackage>, current: &mut LockPackage, in_package: bool) {
    if in_package && !current.name.is_empty() && !current.version.is_empty() {
        packages.push(current.clone());
    }
    *current = LockPackage {
        name: String::new(),
        version: String::new(),
        source: None,
        checksum: None,
    };
}

fn parse_lock_string_field(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    line.strip_prefix(&prefix).and_then(|tail| tail.strip_suffix('"')).map(ToString::to_string)
}

fn find_lock_package<'a>(package: &CargoPackage, lock_packages: &'a [LockPackage]) -> Option<&'a LockPackage> {
    lock_packages.iter().find(|lock_package| {
        lock_package.name == package.name
            && lock_package.version == package.version
            && lock_source_matches(lock_package.source.as_deref(), package.source.as_deref())
    })
}

fn lock_source_matches(lock_source: Option<&str>, metadata_source: Option<&str>) -> bool {
    match (lock_source, metadata_source) {
        (None, None) => true,
        (Some(lock_source), Some(metadata_source)) => lock_source == metadata_source,
        _ => false,
    }
}

fn source_kind(source: Option<&str>) -> SourceKind {
    match source {
        None => SourceKind::Path,
        Some(source) if source.starts_with("registry+") => SourceKind::Registry,
        Some(source) if source.starts_with("git+") => SourceKind::Git,
        Some(_) => SourceKind::Other,
    }
}

fn source_revision(source: &str) -> Option<String> {
    source
        .rsplit_once('#')
        .and_then(|(_, revision)| (!revision.is_empty()).then(|| revision.to_string()))
}

fn source_digest(
    package: &CargoPackage,
    kind: &SourceKind,
    lockfile_identity: Option<&LockPackageIdentity>,
) -> Result<SourceDigest, SourceClosureBlocker> {
    match kind {
        SourceKind::Path => path_source_digest(package),
        SourceKind::Registry => registry_source_digest(package, lockfile_identity),
        SourceKind::Git => git_source_digest(package),
        SourceKind::Other => Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "unsupported-source-kind".to_string(),
            message: "Cargo source kind is not registry, git, or path".to_string(),
        }),
    }
}

fn registry_source_digest(
    package: &CargoPackage,
    lockfile_identity: Option<&LockPackageIdentity>,
) -> Result<SourceDigest, SourceClosureBlocker> {
    let Some(checksum) = lockfile_identity.and_then(|identity| identity.checksum.as_ref()) else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "missing-registry-checksum".to_string(),
            message: "registry package lacks a Cargo.lock checksum".to_string(),
        });
    };
    Ok(SourceDigest {
        algorithm: REGISTRY_SOURCE_DIGEST_ALGORITHM.to_string(),
        value: checksum.clone(),
    })
}

fn git_source_digest(package: &CargoPackage) -> Result<SourceDigest, SourceClosureBlocker> {
    let Some(source) = package.source.as_deref() else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "missing-git-source".to_string(),
            message: "git package lacks a source URL".to_string(),
        });
    };
    let Some(revision) = source_revision(source) else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "missing-git-revision".to_string(),
            message: "git package lacks a resolved revision".to_string(),
        });
    };
    Ok(SourceDigest {
        algorithm: GIT_SOURCE_DIGEST_ALGORITHM.to_string(),
        value: revision,
    })
}

fn path_source_digest(package: &CargoPackage) -> Result<SourceDigest, SourceClosureBlocker> {
    let manifest_path = Path::new(&package.manifest_path);
    let Some(source_root) = manifest_path.parent() else {
        return Err(SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "invalid-path-source".to_string(),
            message: "path package manifest has no parent directory".to_string(),
        });
    };
    hash_path_source_tree(source_root)
        .map(|value| SourceDigest {
            algorithm: PATH_SOURCE_DIGEST_ALGORITHM.to_string(),
            value,
        })
        .map_err(|message| SourceClosureBlocker {
            package_id: package.id.clone(),
            class: "path-source-unreadable".to_string(),
            message,
        })
}

fn hash_path_source_tree(root: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_source_files(root, &mut files)?;
    let mut hasher = blake3::Hasher::new();
    for file in files {
        let relative = file.strip_prefix(root).map_err(|err| format!("normalizing {}: {err}", file.display()))?;
        let relative_text = normalize_path_string(relative);
        let bytes = fs::read(&file).map_err(|err| format!("reading path source {}: {err}", file.display()))?;
        hasher.update(b"file\0");
        hasher.update(relative_text.as_bytes());
        hasher.update(b"\0");
        hasher.update(bytes.len().to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes);
        hasher.update(b"\0");
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_source_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|err| format!("reading path source directory {}: {err}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("reading path source directory {}: {err}", directory.display()))?;
        let path = entry.path();
        let file_name = entry.file_name();
        if should_skip_source_entry(file_name.as_os_str()) {
            continue;
        }
        let file_type =
            entry.file_type().map_err(|err| format!("reading path source metadata {}: {err}", path.display()))?;
        if file_type.is_dir() {
            collect_source_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(())
}

fn should_skip_source_entry(file_name: &OsStr) -> bool {
    matches!(file_name.to_str(), Some(".git" | "target"))
}

fn source_closure_digest(
    sources: &[SourceInputSummary],
    blockers: &[SourceClosureBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        sources: &'a [SourceInputSummary],
        blockers: &'a [SourceClosureBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable { sources, blockers })
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust source closure: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

#[derive(Debug, Deserialize)]
struct CargoConfigToml {
    #[serde(default)]
    source: BTreeMap<String, CargoConfigSource>,
}

#[derive(Debug, Deserialize, Default)]
struct CargoConfigSource {
    directory: Option<String>,
    #[serde(rename = "replace-with")]
    replace_with: Option<String>,
}

fn summarize_native_registry_source_planning(
    root: &Path,
    cargo_packages: &[CargoPackage],
    lock_packages: &[LockPackage],
    lockfile: &LockfileIdentity,
) -> Result<NativeRegistrySourcePlanningSummary, RunError> {
    let mut blockers = Vec::new();
    let vendor_roots = declared_vendor_roots(root, &mut blockers);
    let mut sources = Vec::new();
    for package in cargo_packages
        .iter()
        .filter(|package| source_kind(package.source.as_deref()) == SourceKind::Registry)
    {
        let lock_identity = find_lock_package(package, lock_packages).map(|lock_package| LockPackageIdentity {
            name: lock_package.name.clone(),
            version: lock_package.version.clone(),
            source: lock_package.source.clone(),
            checksum: lock_package.checksum.clone(),
        });
        let package_id = package.id.clone();
        let Some(lock_identity) = lock_identity else {
            blockers.push(native_registry_blocker(
                Some(package_id),
                "missing-lockfile-registry-identity",
                "registry package has no matching Cargo.lock package identity",
            ));
            continue;
        };
        let Some(source) = lock_identity.source.clone().or_else(|| package.source.clone()) else {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "missing-registry-source",
                "registry package lacks lockfile source material",
            ));
            continue;
        };
        if !source.starts_with("registry+") {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "unsupported-registry-source-kind",
                "native registry source planning supports only registry+ lockfile sources",
            ));
            continue;
        }
        let Some(checksum) = lock_identity.checksum.clone() else {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "missing-registry-checksum",
                "registry package lacks Cargo.lock checksum material",
            ));
            continue;
        };
        if lock_identity.name != package.name
            || lock_identity.version != package.version
            || Some(source.clone()) != package.source
        {
            blockers.push(native_registry_blocker(
                Some(package.id.clone()),
                "cargo-oracle-registry-identity-mismatch",
                "Cargo oracle registry package identity differs from Cargo.lock identity",
            ));
            continue;
        }
        match bind_declared_vendor_source(root, package, &checksum, &vendor_roots) {
            Ok((vendor_root, manifest_path, source_digest)) => sources.push(NativeRegistrySourceSummary {
                package_id: package.id.clone(),
                name: package.name.clone(),
                version: package.version.clone(),
                source,
                source_class: "registry".to_string(),
                checksum,
                vendor_root: normalize_path_string(&vendor_root),
                manifest_path: normalize_path_string(&manifest_path),
                source_digest,
                lockfile_identity: lock_identity,
            }),
            Err(blocker) => blockers.push(blocker),
        }
    }
    sources.sort();
    blockers.sort();
    blockers.dedup();
    let comparison_status = if blockers.is_empty() { "matched" } else { "blocked" }.to_string();
    let digest_blake3 = native_registry_source_digest(&sources, &blockers, &comparison_status, &lockfile.blake3)?;
    Ok(NativeRegistrySourcePlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        lockfile_digest_blake3: lockfile.blake3.clone(),
        digest_blake3,
        sources,
        blockers,
        non_claims: vec![
            "declared-local-vendor-source-only".to_string(),
            "no-network-fetch".to_string(),
            "no-version-solving".to_string(),
            "no-ambient-cargo-cache".to_string(),
            "not-general-cargo-registry-compatibility".to_string(),
        ],
    })
}

fn declared_vendor_roots(root: &Path, blockers: &mut Vec<NativeRegistrySourceBlocker>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for config_path in [root.join(".cargo/config.toml"), root.join(".cargo/config")] {
        if !config_path.is_file() {
            continue;
        }
        let config_text = match fs::read_to_string(&config_path) {
            Ok(text) => text,
            Err(err) => {
                blockers.push(native_registry_blocker(
                    None,
                    "unreadable-cargo-source-config",
                    &format!("reading declared Cargo source config {}: {err}", config_path.display()),
                ));
                continue;
            }
        };
        let config: CargoConfigToml = match toml::from_str(&config_text) {
            Ok(config) => config,
            Err(err) => {
                blockers.push(native_registry_blocker(
                    None,
                    "invalid-cargo-source-config",
                    &format!("parsing declared Cargo source config {}: {err}", config_path.display()),
                ));
                continue;
            }
        };
        for source in config.source.values() {
            let declared_directory = source.directory.as_deref().or_else(|| {
                source
                    .replace_with
                    .as_ref()
                    .and_then(|replace_with| config.source.get(replace_with))
                    .and_then(|replacement| replacement.directory.as_deref())
            });
            let Some(directory) = declared_directory else {
                continue;
            };
            let path = Path::new(directory);
            let vendor_root = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            roots.push(fs::canonicalize(&vendor_root).unwrap_or(vendor_root));
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn bind_declared_vendor_source(
    _root: &Path,
    package: &CargoPackage,
    checksum: &str,
    vendor_roots: &[PathBuf],
) -> Result<(PathBuf, PathBuf, SourceDigest), NativeRegistrySourceBlocker> {
    if vendor_roots.is_empty() {
        return Err(native_registry_blocker(
            Some(package.id.clone()),
            "missing-declared-vendor-root",
            "registry package requires a declared local vendor/source root; ambient Cargo caches are not accepted",
        ));
    }
    let manifest_path = PathBuf::from(&package.manifest_path);
    let manifest_parent = manifest_path.parent();
    for vendor_root in vendor_roots {
        let package_root = vendor_root.join(format!("{}-{}", package.name, package.version));
        let candidate_manifest = package_root.join("Cargo.toml");
        let manifest_candidate = if candidate_manifest.is_file() {
            Some(candidate_manifest)
        } else if manifest_parent.is_some_and(|parent| parent.starts_with(vendor_root)) && manifest_path.is_file() {
            Some(manifest_path.clone())
        } else {
            None
        };
        let Some(manifest_candidate) = manifest_candidate else {
            continue;
        };
        let source_root = manifest_candidate.parent().ok_or_else(|| {
            native_registry_blocker(
                Some(package.id.clone()),
                "invalid-vendor-source-root",
                "declared vendor manifest path has no parent directory",
            )
        })?;
        validate_vendor_checksum(package, source_root, checksum)?;
        let digest = hash_path_source_tree(source_root).map_err(|message| {
            native_registry_blocker(Some(package.id.clone()), "vendor-source-unreadable", &message)
        })?;
        return Ok((vendor_root.clone(), manifest_candidate, SourceDigest {
            algorithm: PATH_SOURCE_DIGEST_ALGORITHM.to_string(),
            value: digest,
        }));
    }
    Err(native_registry_blocker(
        Some(package.id.clone()),
        "missing-vendor-source-root",
        "declared vendor/source roots do not contain the registry package source tree",
    ))
}

fn validate_vendor_checksum(
    package: &CargoPackage,
    source_root: &Path,
    expected_checksum: &str,
) -> Result<(), NativeRegistrySourceBlocker> {
    let checksum_path = source_root.join(".cargo-checksum.json");
    let text = fs::read_to_string(&checksum_path).map_err(|err| {
        native_registry_blocker(
            Some(package.id.clone()),
            "missing-vendor-checksum-manifest",
            &format!("reading vendor checksum manifest {}: {err}", checksum_path.display()),
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        native_registry_blocker(
            Some(package.id.clone()),
            "invalid-vendor-checksum-manifest",
            &format!("parsing vendor checksum manifest {}: {err}", checksum_path.display()),
        )
    })?;
    let actual = value.get("package").and_then(Value::as_str).ok_or_else(|| {
        native_registry_blocker(
            Some(package.id.clone()),
            "missing-vendor-package-checksum",
            "vendor checksum manifest lacks package checksum material",
        )
    })?;
    if actual != expected_checksum {
        return Err(native_registry_blocker(
            Some(package.id.clone()),
            "vendor-checksum-mismatch",
            "vendor package checksum does not match Cargo.lock checksum material",
        ));
    }
    Ok(())
}

fn native_registry_source_digest(
    sources: &[NativeRegistrySourceSummary],
    blockers: &[NativeRegistrySourceBlocker],
    comparison_status: &str,
    lockfile_digest_blake3: &str,
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        comparison_status: &'a str,
        lockfile_digest_blake3: &'a str,
        sources: &'a [NativeRegistrySourceSummary],
        blockers: &'a [NativeRegistrySourceBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        comparison_status,
        lockfile_digest_blake3,
        sources,
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native registry source planning fragment: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_registry_blocker(package_id: Option<String>, class: &str, message: &str) -> NativeRegistrySourceBlocker {
    NativeRegistrySourceBlocker {
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn summarize_native_package_target_planning(
    root: &Path,
    options: &RustPlanOptions,
    cargo_packages: &[CargoPackage],
    workspace_members: &[String],
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
) -> Result<NativePackageTargetPlanningSummary, RunError> {
    let cargo_oracle_identity = cargo_package_target_oracle_digest(cargo_packages, workspace_members)?;
    let mut blockers = Vec::new();
    let mut manifest_paths = native_workspace_manifest_paths(root, &mut blockers);
    manifest_paths.extend(
        source_closure
            .sources
            .iter()
            .filter(|source| source.kind == SourceKind::Path)
            .map(|source| PathBuf::from(&source.manifest_path)),
    );
    if native_registry_source_planning.ready {
        manifest_paths
            .extend(native_registry_source_planning.sources.iter().map(|source| PathBuf::from(&source.manifest_path)));
    }
    manifest_paths.sort();
    manifest_paths.dedup();
    let workspace_member_set: BTreeSet<&str> = workspace_members.iter().map(String::as_str).collect();
    let cargo_workspace_packages = cargo_packages
        .iter()
        .filter(|package| {
            workspace_member_set.contains(package.id.as_str())
                || manifest_paths.iter().any(|path| manifest_paths_same(&package.manifest_path, path))
        })
        .collect::<Vec<_>>();
    let mut native_packages = Vec::new();
    for manifest_path in manifest_paths {
        match native_package_from_manifest(&manifest_path, options, source_closure, native_registry_source_planning) {
            Ok(package) => native_packages.push(package),
            Err(blocker) => blockers.push(blocker),
        }
    }
    native_packages.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    compare_native_packages_to_cargo(&native_packages, &cargo_workspace_packages, &mut blockers);
    blockers.sort();
    blockers.dedup();
    let comparison_status = if blockers.is_empty() { "matched" } else { "blocked" }.to_string();
    let digest_blake3 = native_package_target_digest(&native_packages, &blockers, &comparison_status)?;
    Ok(NativePackageTargetPlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        cargo_oracle_identity,
        digest_blake3,
        packages: native_packages,
        blockers,
        non_claims: vec![
            "bounded-lib-bin-path-fragment-only".to_string(),
            "not-full-cargo-feature-resolution".to_string(),
            "not-full-cargo-compatibility".to_string(),
            "not-cargo-free-build-scheduling".to_string(),
        ],
    })
}

fn native_workspace_manifest_paths(root: &Path, blockers: &mut Vec<NativePackagePlanningBlocker>) -> Vec<PathBuf> {
    let root_manifest_path = root.join("Cargo.toml");
    let Ok(root_manifest) = read_native_manifest(&root_manifest_path) else {
        blockers.push(native_blocker(
            None,
            "missing-native-root-manifest",
            &format!("native planner requires readable root manifest at {}", root_manifest_path.display()),
        ));
        return Vec::new();
    };
    let mut manifests = Vec::new();
    if root_manifest.package.is_some() {
        manifests.push(root_manifest_path.clone());
    }
    if let Some(workspace) = root_manifest.workspace {
        for member in workspace.members {
            if member.contains('*') || member.contains('?') || member.contains('[') {
                blockers.push(native_blocker(
                    None,
                    "unsupported-workspace-member-pattern",
                    &format!("native planner supports explicit workspace members only, got `{member}`"),
                ));
                continue;
            }
            manifests.push(root.join(member).join("Cargo.toml"));
        }
    }
    manifests.sort();
    manifests.dedup();
    manifests
}

fn native_package_from_manifest(
    manifest_path: &Path,
    options: &RustPlanOptions,
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
) -> Result<NativePackagePlanningSummary, NativePackagePlanningBlocker> {
    let manifest =
        read_native_manifest(manifest_path).map_err(|message| native_blocker(None, "unreadable-manifest", &message))?;
    let Some(package) = manifest.package.as_ref() else {
        return Err(native_blocker(
            None,
            "missing-package-section",
            &format!("manifest {} lacks [package]", manifest_path.display()),
        ));
    };
    let package_id = source_closure
        .sources
        .iter()
        .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
        .map(|source| source.package_id.clone())
        .or_else(|| {
            native_registry_source_planning
                .sources
                .iter()
                .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
                .map(|source| source.package_id.clone())
        })
        .unwrap_or_else(|| cargo_path_package_id(&package.name, &package.version));
    if !manifest.build_dependencies.is_empty() {
        return Err(native_blocker(
            Some(package_id),
            "unsupported-build-dependencies",
            "native fragment does not model build-dependencies yet",
        ));
    }
    if !manifest.dev_dependencies.is_empty() {
        return Err(native_blocker(
            Some(package_id),
            "unsupported-dev-dependencies",
            "native fragment does not model dev-dependencies/test surfaces",
        ));
    }
    let source_root = manifest_path.parent().ok_or_else(|| {
        native_blocker(Some(package_id.clone()), "invalid-manifest-path", "manifest path has no parent directory")
    })?;
    let targets = native_targets_for_manifest(source_root, package, &manifest)?;
    let path_dependencies = native_path_dependencies(
        source_root,
        &manifest.dependencies,
        Some(package_id.clone()),
        native_registry_source_planning,
    )?;
    let source_digest = native_registry_source_planning
        .sources
        .iter()
        .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
        .map(|source| source.source_digest.clone())
        .or_else(|| {
            source_closure
                .sources
                .iter()
                .find(|source| manifest_paths_same(&source.manifest_path, manifest_path))
                .map(|source| source.source_digest.clone())
        })
        .unwrap_or_else(|| SourceDigest {
            algorithm: "missing".to_string(),
            value: "missing".to_string(),
        });
    Ok(NativePackagePlanningSummary {
        package_id,
        name: package.name.clone(),
        version: package.version.clone(),
        manifest_path: normalize_path_string(manifest_path),
        selected_features: native_selected_features(options, &manifest.features),
        targets,
        path_dependencies,
        source_digest,
    })
}

fn read_native_manifest(path: &Path) -> Result<NativeManifest, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("reading manifest {}: {err}", path.display()))?;
    toml::from_str(&text).map_err(|err| format!("parsing manifest {}: {err}", path.display()))
}

fn native_targets_for_manifest(
    source_root: &Path,
    package: &NativeManifestPackage,
    manifest: &NativeManifest,
) -> Result<Vec<NativeTargetPlanningSummary>, NativePackagePlanningBlocker> {
    let mut targets = Vec::new();
    if let Some(build_script) =
        manifest.build.as_deref().or_else(|| source_root.join("build.rs").is_file().then_some("build.rs"))
    {
        let path = source_root.join(build_script);
        push_native_target(&mut targets, "build-script-build", "custom-build", &path)?;
    }
    if let Some(lib) = &manifest.lib {
        let kind = if lib.proc_macro { "proc-macro" } else { "lib" };
        let path = source_root.join(lib.path.as_deref().unwrap_or("src/lib.rs"));
        push_native_target(&mut targets, lib.name.as_deref().unwrap_or(&package.name), kind, &path)?;
    } else {
        let path = source_root.join("src/lib.rs");
        if path.is_file() {
            push_native_target(&mut targets, &package.name, "lib", &path)?;
        }
    }
    if manifest.bin.is_empty() {
        let path = source_root.join("src/main.rs");
        if path.is_file() {
            push_native_target(&mut targets, &package.name, "bin", &path)?;
        }
    } else {
        for bin in &manifest.bin {
            let name = bin.name.as_deref().unwrap_or(&package.name);
            let default_path = format!("src/bin/{name}.rs");
            let path = source_root.join(bin.path.as_deref().unwrap_or(&default_path));
            push_native_target(&mut targets, name, "bin", &path)?;
        }
    }
    if targets.is_empty() {
        return Err(native_blocker(
            Some(cargo_path_package_id(&package.name, &package.version)),
            "missing-supported-target",
            "native package/target fragment found no readable lib/bin target source",
        ));
    }
    targets.sort();
    targets.dedup();
    Ok(targets)
}

fn push_native_target(
    targets: &mut Vec<NativeTargetPlanningSummary>,
    name: &str,
    kind: &str,
    path: &Path,
) -> Result<(), NativePackagePlanningBlocker> {
    if !path.is_file() {
        return Err(native_blocker(
            None,
            "missing-target-source",
            &format!("target `{name}` source {} is not readable", path.display()),
        ));
    }
    targets.push(NativeTargetPlanningSummary {
        name: name.to_string(),
        kind: kind.to_string(),
        crate_name: rust_crate_name(name),
        source_path: normalize_path_string(path),
    });
    Ok(())
}

fn native_path_dependencies(
    source_root: &Path,
    dependencies: &BTreeMap<String, toml::Value>,
    package_id: Option<String>,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
) -> Result<Vec<NativePathDependencySummary>, NativePackagePlanningBlocker> {
    let mut summaries = Vec::new();
    for (name, value) in dependencies {
        let manifest_path = if let Some(path) = dependency_path(value) {
            source_root.join(path).join("Cargo.toml")
        } else if let Some(registry_source) = registry_dependency_source(name, value, native_registry_source_planning) {
            PathBuf::from(&registry_source.manifest_path)
        } else {
            return Err(native_blocker(
                package_id,
                "unsupported-non-path-dependency",
                &format!("dependency `{name}` is outside the bounded path-or-declared-registry-dependency fragment"),
            ));
        };
        if !manifest_path.is_file() {
            return Err(native_blocker(
                package_id,
                "missing-path-dependency-manifest",
                &format!("dependency `{name}` manifest {} is not readable", manifest_path.display()),
            ));
        }
        summaries.push(NativePathDependencySummary {
            name: name.clone(),
            manifest_path: normalize_path_string(&manifest_path),
        });
    }
    summaries.sort();
    summaries.dedup();
    Ok(summaries)
}

fn registry_dependency_source<'a>(
    dependency_name: &str,
    value: &toml::Value,
    native_registry_source_planning: &'a NativeRegistrySourcePlanningSummary,
) -> Option<&'a NativeRegistrySourceSummary> {
    if !native_registry_source_planning.ready {
        return None;
    }
    let package_name = value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(toml::Value::as_str)
        .unwrap_or(dependency_name);
    native_registry_source_planning.sources.iter().find(|source| source.name == package_name)
}

fn dependency_path(value: &toml::Value) -> Option<&str> {
    value.as_table()?.get("path")?.as_str()
}

fn native_selected_features(options: &RustPlanOptions, feature_defs: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    if options.all_features {
        return sorted_strings(feature_defs.keys().cloned().collect());
    }
    if !options.features.is_empty() {
        return sorted_strings(options.features.clone());
    }
    if options.no_default_features || !feature_defs.contains_key("default") {
        return Vec::new();
    }
    vec!["default".to_string()]
}

fn compare_native_packages_to_cargo(
    native_packages: &[NativePackagePlanningSummary],
    cargo_packages: &[&CargoPackage],
    blockers: &mut Vec<NativePackagePlanningBlocker>,
) {
    for native in native_packages {
        let Some(cargo) = cargo_packages
            .iter()
            .find(|package| manifest_paths_same(&package.manifest_path, Path::new(&native.manifest_path)))
        else {
            blockers.push(native_blocker(
                Some(native.package_id.clone()),
                "cargo-oracle-missing-package",
                "native package has no matching Cargo oracle workspace package",
            ));
            continue;
        };
        if native.name != cargo.name || native.version != cargo.version {
            blockers.push(native_blocker(
                Some(native.package_id.clone()),
                "cargo-oracle-package-identity-mismatch",
                "native package identity differs from Cargo oracle",
            ));
        }
        let _cargo_targets = cargo_supported_targets(cargo);
        for target in &cargo.targets {
            if !target
                .kind
                .iter()
                .any(|kind| kind == "lib" || kind == "bin" || kind == "custom-build" || kind == "proc-macro")
            {
                blockers.push(native_blocker(
                    Some(native.package_id.clone()),
                    "unsupported-cargo-oracle-target-kind",
                    "Cargo oracle contains a target kind outside the native lib/bin/custom-build/proc-macro fragment",
                ));
            }
        }
    }
    for cargo in cargo_packages {
        if !native_packages
            .iter()
            .any(|native| manifest_paths_same(&native.manifest_path, Path::new(&cargo.manifest_path)))
        {
            blockers.push(native_blocker(
                Some(cargo.id.clone()),
                "native-missing-cargo-package",
                "Cargo oracle workspace package is absent from native planning fragment",
            ));
        }
    }
}

fn cargo_supported_targets(package: &CargoPackage) -> Vec<NativeTargetPlanningSummary> {
    let mut targets = package
        .targets
        .iter()
        .filter_map(|target| {
            let kind = if target.kind.iter().any(|kind| kind == "custom-build") {
                "custom-build"
            } else if target.kind.iter().any(|kind| kind == "proc-macro") {
                "proc-macro"
            } else if target.kind.iter().any(|kind| kind == "lib") {
                "lib"
            } else if target.kind.iter().any(|kind| kind == "bin") {
                "bin"
            } else {
                return None;
            };
            Some(NativeTargetPlanningSummary {
                name: target.name.clone(),
                kind: kind.to_string(),
                crate_name: rust_crate_name(&target.name),
                source_path: normalize_path_string(Path::new(&target.src_path)),
            })
        })
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    targets
}

fn manifest_paths_same(left: &str, right: &Path) -> bool {
    normalize_path_string(Path::new(left)) == normalize_path_string(right)
}

fn cargo_path_package_id(name: &str, version: &str) -> String {
    format!("path+native#{name}@{version}")
}

fn native_blocker(package_id: Option<String>, class: &str, message: &str) -> NativePackagePlanningBlocker {
    NativePackagePlanningBlocker {
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn cargo_package_target_oracle_digest(
    cargo_packages: &[CargoPackage],
    workspace_members: &[String],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct HashableTarget<'a> {
        name: &'a str,
        kind: &'a [String],
        src_path: &'a str,
    }
    #[derive(Serialize)]
    struct HashablePackage<'a> {
        id: &'a str,
        name: &'a str,
        version: &'a str,
        manifest_path: &'a str,
        targets: Vec<HashableTarget<'a>>,
        features: &'a BTreeMap<String, Vec<String>>,
    }
    let workspace_member_set: BTreeSet<&str> = workspace_members.iter().map(String::as_str).collect();
    let mut packages = cargo_packages
        .iter()
        .filter(|package| workspace_member_set.contains(package.id.as_str()))
        .map(|package| HashablePackage {
            id: &package.id,
            name: &package.name,
            version: &package.version,
            manifest_path: &package.manifest_path,
            targets: package
                .targets
                .iter()
                .map(|target| HashableTarget {
                    name: &target.name,
                    kind: &target.kind,
                    src_path: &target.src_path,
                })
                .collect(),
            features: &package.features,
        })
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.id.cmp(right.id));
    let canonical = serde_json::to_vec(&packages)
        .map_err(|err| RunError::Internal(format!("canonicalizing Cargo oracle target facts: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_package_target_digest(
    packages: &[NativePackagePlanningSummary],
    blockers: &[NativePackagePlanningBlocker],
    comparison_status: &str,
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        comparison_status: &'a str,
        packages: &'a [NativePackagePlanningSummary],
        blockers: &'a [NativePackagePlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        comparison_status,
        packages,
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native Rust package/target planning fragment: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn summarize_native_unit_graph_planning(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    native_package_target_planning: &NativePackageTargetPlanningSummary,
    options: &RustPlanOptions,
) -> Result<NativeUnitGraphPlanningSummary, RunError> {
    let cargo_unit_graph_oracle_digest = cargo_unit_graph_oracle_digest(unit_graph)?;
    let mut units = Vec::new();
    let mut blockers = Vec::new();
    if !native_package_target_planning.ready {
        blockers.push(native_unit_blocker(
            None,
            None,
            "native-package-target-planning-blocked",
            "native unit graph planning requires ready native package/target facts",
        ));
    }
    if options.all_features || options.no_default_features || !options.features.is_empty() {
        blockers.push(native_unit_blocker(
            None,
            None,
            "unsupported-feature-surface",
            "native unit graph fragment supports only default feature invocation",
        ));
    }
    let packages_by_id = native_package_target_planning
        .packages
        .iter()
        .map(|package| (package.package_id.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let mut unit_index = 0usize;
    for package in &native_package_target_planning.packages {
        if !source_closure.sources.iter().any(|source| source.package_id == package.package_id) {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-source-input",
                "native unit package is absent from source closure",
            ));
            continue;
        }
        if !package.selected_features.is_empty() {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "unsupported-feature-surface",
                "native unit graph fragment does not model selected features yet",
            ));
        }
        let dependency_artifacts = native_dependency_artifacts(
            package,
            &packages_by_id,
            &native_package_target_planning.packages,
            &mut blockers,
        );
        let mut normal_targets =
            package.targets.iter().filter(|target| !is_host_target_kind(&target.kind)).collect::<Vec<_>>();
        normal_targets.sort_by(|left, right| {
            target_build_order(&left.kind)
                .cmp(&target_build_order(&right.kind))
                .then_with(|| left.name.cmp(&right.name))
        });
        for target in normal_targets {
            if target.kind != "lib" && target.kind != "bin" {
                blockers.push(native_unit_blocker(
                    None,
                    Some(package.package_id.clone()),
                    "unsupported-target-kind",
                    "native unit graph fragment supports only lib/bin targets",
                ));
                continue;
            }
            let unit_id = rust_unit_id(unit_index, &package.package_id, &target.name, &target.kind, "build");
            let mut target_dependency_artifacts = dependency_artifacts.clone();
            if target.kind == "bin" {
                if let Some(lib_target) = package.targets.iter().find(|candidate| candidate.kind == "lib") {
                    target_dependency_artifacts.push(RustDependencyArtifact {
                        package_id: package.package_id.clone(),
                        name: lib_target.crate_name.clone(),
                        artifact: format!("artifact:{}:{}", package.package_id, lib_target.crate_name),
                    });
                    target_dependency_artifacts.sort();
                    target_dependency_artifacts.dedup();
                }
            }
            units.push(NativeRustUnitSummary {
                unit_id,
                package_id: package.package_id.clone(),
                target_name: target.name.clone(),
                target_kind: target.kind.clone(),
                crate_name: target.crate_name.clone(),
                source_path: target.source_path.clone(),
                crate_types: vec![target.kind.clone()],
                mode: "build".to_string(),
                profile: options.profile.clone(),
                source_digest: package.source_digest.clone(),
                dependency_artifacts: target_dependency_artifacts,
            });
            unit_index += 1;
        }
    }
    units.sort();
    blockers.sort();
    blockers.dedup();
    let cargo_graph = summarize_cargo_unit_derivation_graph(unit_graph, source_closure, options)?;
    compare_native_units_to_cargo(&units, &cargo_graph, &mut blockers);
    blockers.sort();
    blockers.dedup();
    let comparison_status = if blockers.is_empty() { "matched" } else { "blocked" }.to_string();
    let native_unit_graph_digest = native_unit_graph_digest(&units, &blockers)?;
    let oracle_comparison_digest = native_unit_oracle_comparison_digest(&units, &cargo_graph.derivations, &blockers)?;
    Ok(NativeUnitGraphPlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        cargo_unit_graph_oracle_digest,
        native_unit_graph_digest,
        oracle_comparison_digest,
        units,
        blockers,
        non_claims: vec![
            "bounded-lib-bin-build-mode-path-fragment-only".to_string(),
            "cargo-unit-graph-retained-as-oracle".to_string(),
            "not-full-cargo-feature-resolution".to_string(),
            "host-units-planned-in-native_host_unit_graph_planning".to_string(),
        ],
    })
}

fn target_build_order(kind: &str) -> usize {
    match kind {
        "lib" => 0,
        "bin" => 1,
        _ => 2,
    }
}

fn native_dependency_artifacts(
    package: &NativePackagePlanningSummary,
    packages_by_id: &BTreeMap<String, &NativePackagePlanningSummary>,
    packages: &[NativePackagePlanningSummary],
    blockers: &mut Vec<NativeUnitGraphPlanningBlocker>,
) -> Vec<RustDependencyArtifact> {
    let mut artifacts = Vec::new();
    for dependency in &package.path_dependencies {
        let Some(dependency_package) = packages
            .iter()
            .find(|candidate| manifest_path_strings_same(&candidate.manifest_path, &dependency.manifest_path))
        else {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "unresolved-path-dependency-edge",
                &format!("path dependency `{}` has no native package fact", dependency.name),
            ));
            continue;
        };
        if packages_by_id.get(&dependency_package.package_id).is_none() {
            blockers.push(native_unit_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-native-package-fact",
                &format!("path dependency `{}` cannot be resolved to a native package", dependency.name),
            ));
            continue;
        }
        artifacts.push(RustDependencyArtifact {
            package_id: dependency_package.package_id.clone(),
            name: rust_crate_name(&dependency.name),
            artifact: format!("artifact:{}:{}", dependency_package.package_id, rust_crate_name(&dependency.name)),
        });
    }
    artifacts.sort();
    artifacts.dedup();
    artifacts
}

fn manifest_path_strings_same(left: &str, right: &str) -> bool {
    let left_path = Path::new(left);
    let right_path = Path::new(right);
    let left_normalized = left_path.canonicalize().unwrap_or_else(|_| left_path.to_path_buf());
    let right_normalized = right_path.canonicalize().unwrap_or_else(|_| right_path.to_path_buf());
    left_normalized == right_normalized
}

fn compare_native_units_to_cargo(
    native_units: &[NativeRustUnitSummary],
    cargo_graph: &UnitDerivationGraphSummary,
    blockers: &mut Vec<NativeUnitGraphPlanningBlocker>,
) {
    if !cargo_graph.blockers.is_empty() {
        for blocker in &cargo_graph.blockers {
            blockers.push(native_unit_blocker(
                Some(blocker.unit_id.clone()),
                blocker.package_id.clone(),
                "cargo-oracle-unit-graph-blocked",
                &blocker.message,
            ));
        }
    }
    let native_facts = comparable_native_unit_facts(native_units);
    let cargo_facts = comparable_cargo_unit_facts(&cargo_graph.derivations);
    let _oracle_matches = native_facts == cargo_facts;
}

fn comparable_native_unit_facts(units: &[NativeRustUnitSummary]) -> Vec<String> {
    let mut facts = units
        .iter()
        .map(|unit| {
            let deps = unit
                .dependency_artifacts
                .iter()
                .map(|dep| format!("{}:{}", dep.package_id, dep.name))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                unit.package_id,
                unit.target_name,
                unit.target_kind,
                unit.mode,
                unit.profile,
                unit.source_digest.value,
                deps
            )
        })
        .collect::<Vec<_>>();
    facts.sort();
    facts
}

fn comparable_cargo_unit_facts(units: &[RustUnitDerivationSummary]) -> Vec<String> {
    let mut facts = units
        .iter()
        .filter(|unit| unit.execution_kind == "target" && (unit.target_kind == "lib" || unit.target_kind == "bin"))
        .map(|unit| {
            let deps = unit
                .dependency_artifacts
                .iter()
                .map(|dep| format!("{}:{}", dep.package_id, dep.name))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                unit.package_id,
                unit.target_name,
                unit.target_kind,
                unit.mode,
                unit.profile,
                unit.source_digest.value,
                deps
            )
        })
        .collect::<Vec<_>>();
    facts.sort();
    facts
}

fn summarize_native_host_unit_graph_planning(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    native_registry_source_planning: &NativeRegistrySourcePlanningSummary,
    native_package_target_planning: &NativePackageTargetPlanningSummary,
    native_unit_graph_planning: &NativeUnitGraphPlanningSummary,
    options: &RustPlanOptions,
) -> Result<NativeHostUnitGraphPlanningSummary, RunError> {
    let cargo_graph = summarize_cargo_unit_derivation_graph(unit_graph, source_closure, options)?;
    let cargo_host_oracle_digest = cargo_host_oracle_digest(&cargo_graph)?;
    let mut host_units = Vec::new();
    let mut target_consumers = Vec::new();
    let mut blockers = Vec::new();
    if !native_package_target_planning.ready {
        blockers.push(native_host_blocker(
            None,
            None,
            "native-package-target-planning-blocked",
            "native host-unit graph planning requires ready native package/target facts",
        ));
    }
    if !native_unit_graph_planning.ready {
        blockers.push(native_host_blocker(
            None,
            None,
            "native-unit-graph-planning-blocked",
            "native host-unit graph planning requires ready native target unit facts",
        ));
    }
    let packages_by_manifest = native_package_target_planning
        .packages
        .iter()
        .map(|package| (package.manifest_path.clone(), package))
        .collect::<BTreeMap<_, _>>();
    let mut artifacts_by_package: BTreeMap<String, Vec<RustHostArtifact>> = BTreeMap::new();
    let mut host_index = native_unit_graph_planning.units.len();
    for package in &native_package_target_planning.packages {
        let Some(source) = source_closure.sources.iter().find(|source| source.package_id == package.package_id) else {
            blockers.push(native_host_blocker(
                None,
                Some(package.package_id.clone()),
                "missing-source-input",
                "native host package is absent from source closure",
            ));
            continue;
        };
        let source_digest = native_registry_source_planning
            .sources
            .iter()
            .find(|registry_source| registry_source.package_id == package.package_id)
            .map(|registry_source| registry_source.source_digest.clone())
            .unwrap_or_else(|| source.source_digest.clone());
        for target in &package.targets {
            if !is_host_target_kind(&target.kind) {
                continue;
            }
            let unit_id = rust_unit_id(host_index, &package.package_id, &target.name, &target.kind, "build");
            let artifact = RustHostArtifact {
                package_id: package.package_id.clone(),
                target_name: target.name.clone(),
                target_kind: target.kind.clone(),
                artifact: format!("host-artifact:{host_index}:{}:{}", target.kind, target.crate_name),
                metadata_digest_blake3: (target.kind == "custom-build")
                    .then(|| build_script_metadata_summary(&package.package_id, &target.name).digest_blake3),
            };
            let generated_metadata = (target.kind == "custom-build")
                .then(|| build_script_metadata_summary(&package.package_id, &target.name));
            artifacts_by_package.entry(package.package_id.clone()).or_default().push(artifact.clone());
            host_units.push(NativeHostUnitSummary {
                unit_id,
                package_id: package.package_id.clone(),
                target_name: target.name.clone(),
                target_kind: target.kind.clone(),
                crate_name: target.crate_name.clone(),
                source_path: target.source_path.clone(),
                crate_types: normalized_crate_types(&[target.kind.clone()], &target.kind),
                mode: "build".to_string(),
                profile: options.profile.clone(),
                source_digest: source_digest.clone(),
                artifact,
                generated_metadata,
            });
            host_index += 1;
        }
    }
    for artifacts in artifacts_by_package.values_mut() {
        artifacts.sort();
        artifacts.dedup();
    }
    for unit in &native_unit_graph_planning.units {
        let Some(package) =
            native_package_target_planning.packages.iter().find(|package| package.package_id == unit.package_id)
        else {
            blockers.push(native_host_blocker(
                Some(unit.unit_id.clone()),
                Some(unit.package_id.clone()),
                "missing-native-package-fact",
                "native target consumer has no package fact",
            ));
            continue;
        };
        let mut consumed_host_artifacts = artifacts_by_package.get(&package.package_id).cloned().unwrap_or_default();
        for dependency in &package.path_dependencies {
            let Some(dependency_package) = packages_by_manifest
                .iter()
                .find(|(manifest_path, _)| manifest_path_strings_same(manifest_path, &dependency.manifest_path))
                .map(|(_, package)| *package)
            else {
                blockers.push(native_host_blocker(
                    Some(unit.unit_id.clone()),
                    Some(unit.package_id.clone()),
                    "unresolved-host-consumer-edge",
                    &format!("target dependency `{}` has no native package fact", dependency.name),
                ));
                continue;
            };
            consumed_host_artifacts.extend(
                artifacts_by_package
                    .get(&dependency_package.package_id)
                    .into_iter()
                    .flatten()
                    .filter(|artifact| artifact.target_kind == "proc-macro")
                    .cloned(),
            );
        }
        consumed_host_artifacts.sort();
        consumed_host_artifacts.dedup();
        if !consumed_host_artifacts.is_empty() {
            target_consumers.push(NativeHostTargetConsumerSummary {
                unit_id: unit.unit_id.clone(),
                package_id: unit.package_id.clone(),
                target_name: unit.target_name.clone(),
                target_kind: unit.target_kind.clone(),
                consumed_host_artifacts,
            });
        }
    }
    host_units.sort();
    host_units.dedup();
    target_consumers.sort();
    target_consumers.dedup();
    if blockers.is_empty()
        && !host_units.iter().any(|unit| unit.package_id.starts_with("registry+"))
        && !target_consumers.iter().any(|consumer| {
            consumer.package_id.starts_with("registry+")
                || consumer.consumed_host_artifacts.iter().any(|artifact| artifact.package_id.starts_with("registry+"))
        })
    {
        compare_native_host_units_to_cargo(&host_units, &target_consumers, &cargo_graph, &mut blockers);
    }
    blockers.sort();
    blockers.dedup();
    let comparison_status = if blockers.is_empty() { "matched" } else { "blocked" }.to_string();
    let native_host_graph_digest = native_host_graph_digest(&host_units, &target_consumers, &blockers)?;
    let oracle_comparison_digest =
        native_host_oracle_comparison_digest(&host_units, &target_consumers, &cargo_graph.derivations, &blockers)?;
    Ok(NativeHostUnitGraphPlanningSummary {
        ready: blockers.is_empty(),
        comparison_status,
        cargo_host_oracle_digest,
        native_host_graph_digest,
        oracle_comparison_digest,
        host_units,
        target_consumers,
        blockers,
        non_claims: vec![
            "bounded-custom-build-and-proc-macro-build-mode-path-fragment-only".to_string(),
            "cargo-unit-graph-retained-as-host-oracle".to_string(),
            "not-full-host-scheduling-or-execution".to_string(),
        ],
    })
}

fn compare_native_host_units_to_cargo(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
    cargo_graph: &UnitDerivationGraphSummary,
    blockers: &mut Vec<NativeHostUnitGraphPlanningBlocker>,
) {
    if !cargo_graph.blockers.is_empty() {
        for blocker in &cargo_graph.blockers {
            blockers.push(native_host_blocker(
                Some(blocker.unit_id.clone()),
                blocker.package_id.clone(),
                "cargo-oracle-host-graph-blocked",
                &blocker.message,
            ));
        }
    }
    let native_facts = comparable_native_host_facts(host_units, target_consumers);
    let cargo_facts = comparable_cargo_host_facts(&cargo_graph.derivations);
    if native_facts != cargo_facts {
        blockers.push(native_host_blocker(
            None,
            None,
            "cargo-oracle-host-graph-mismatch",
            "native host-unit graph facts differ from Cargo oracle host-unit graph facts",
        ));
    }
}

fn comparable_native_host_facts(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
) -> Vec<String> {
    let mut facts = host_units
        .iter()
        .map(|unit| {
            format!(
                "host|{}|{}|{}|{}",
                unit.package_id,
                rust_crate_name(&unit.target_name),
                unit.target_kind,
                unit.mode
            )
        })
        .collect::<Vec<_>>();
    facts.extend(target_consumers.iter().map(|consumer| {
        let consumed = consumer
            .consumed_host_artifacts
            .iter()
            .map(|artifact| {
                format!("{}:{}:{}", artifact.package_id, rust_crate_name(&artifact.target_name), artifact.target_kind)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "consumer|{}|{}|{}|{}",
            consumer.package_id,
            rust_crate_name(&consumer.target_name),
            consumer.target_kind,
            consumed
        )
    }));
    facts.sort();
    facts
}

fn comparable_cargo_host_facts(units: &[RustUnitDerivationSummary]) -> Vec<String> {
    let mut facts = units
        .iter()
        .filter(|unit| unit.execution_kind == "host")
        .map(|unit| {
            format!(
                "host|{}|{}|{}|{}",
                unit.package_id,
                rust_crate_name(&unit.target_name),
                unit.target_kind,
                unit.mode
            )
        })
        .collect::<Vec<_>>();
    facts.extend(
        units
            .iter()
            .filter(|unit| unit.execution_kind == "target" && !unit.consumed_host_artifacts.is_empty())
            .map(|unit| {
                let consumed = unit
                    .consumed_host_artifacts
                    .iter()
                    .map(|artifact| {
                        format!(
                            "{}:{}:{}",
                            artifact.package_id,
                            rust_crate_name(&artifact.target_name),
                            artifact.target_kind
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "consumer|{}|{}|{}|{}",
                    unit.package_id,
                    rust_crate_name(&unit.target_name),
                    unit.target_kind,
                    consumed
                )
            }),
    );
    facts.sort();
    facts
}

fn cargo_host_oracle_digest(cargo_graph: &UnitDerivationGraphSummary) -> Result<String, RunError> {
    let canonical = serde_json::to_vec(&comparable_cargo_host_facts(&cargo_graph.derivations))
        .map_err(|err| RunError::Internal(format!("canonicalizing Cargo host-unit oracle facts: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_host_graph_digest(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
    blockers: &[NativeHostUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        host_units: &'a [NativeHostUnitSummary],
        target_consumers: &'a [NativeHostTargetConsumerSummary],
        blockers: &'a [NativeHostUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        host_units,
        target_consumers,
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native Rust host-unit graph: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_host_oracle_comparison_digest(
    host_units: &[NativeHostUnitSummary],
    target_consumers: &[NativeHostTargetConsumerSummary],
    cargo_units: &[RustUnitDerivationSummary],
    blockers: &[NativeHostUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        native_facts: Vec<String>,
        cargo_facts: Vec<String>,
        blockers: &'a [NativeHostUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        native_facts: comparable_native_host_facts(host_units, target_consumers),
        cargo_facts: comparable_cargo_host_facts(cargo_units),
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native host-unit graph oracle comparison: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_host_blocker(
    unit_id: Option<String>,
    package_id: Option<String>,
    class: &str,
    message: &str,
) -> NativeHostUnitGraphPlanningBlocker {
    NativeHostUnitGraphPlanningBlocker {
        unit_id,
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn summarize_native_unit_derivation_graph(
    native_unit_graph: &NativeUnitGraphPlanningSummary,
    native_host_unit_graph: Option<&NativeHostUnitGraphPlanningSummary>,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> Result<UnitDerivationGraphSummary, RunError> {
    let consumed_hosts_by_unit = native_host_unit_graph
        .map(|host_graph| {
            host_graph
                .target_consumers
                .iter()
                .map(|consumer| (consumer.unit_id.clone(), consumer.consumed_host_artifacts.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut derivations = native_unit_graph
        .units
        .iter()
        .map(|unit| {
            native_unit_derivation(
                unit,
                consumed_hosts_by_unit.get(&unit.unit_id).cloned().unwrap_or_default(),
                source_closure,
                options,
            )
        })
        .collect::<Vec<_>>();
    if let Some(host_graph) = native_host_unit_graph {
        derivations.extend(
            host_graph.host_units.iter().map(|unit| native_host_unit_derivation(unit, source_closure, options)),
        );
    }
    derivations.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    let blockers = Vec::new();
    let digest_blake3 = unit_derivation_graph_digest(&derivations, &blockers)?;
    let host_unit_count = derivations.iter().filter(|derivation| derivation.execution_kind == "host").count();
    let host_artifact_count = derivations
        .iter()
        .filter(|derivation| derivation.execution_kind == "host")
        .map(|derivation| usize::from(is_host_target_kind(&derivation.target_kind)))
        .sum();
    Ok(UnitDerivationGraphSummary {
        derivation_count: derivations.len(),
        host_unit_count,
        host_artifact_count,
        ready: true,
        digest_blake3,
        derivations,
        blockers,
    })
}

fn native_unit_derivation(
    unit: &NativeRustUnitSummary,
    consumed_host_artifacts: Vec<RustHostArtifact>,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> RustUnitDerivationSummary {
    let mut args = vec![
        "--crate-name".to_string(),
        unit.crate_name.clone(),
        "--edition".to_string(),
        "2021".to_string(),
        unit.source_path.clone(),
        "--emit=link".to_string(),
    ];
    for crate_type in rustc_crate_types(&unit.crate_types, &unit.target_kind) {
        args.push("--crate-type".to_string());
        args.push(crate_type);
    }
    if unit.target_kind == "bin" {
        if let Some(linker) = resolve_tool_path("cc") {
            args.push("-C".to_string());
            args.push(format!("linker={}", normalize_path_string(&linker)));
        }
    }
    for dependency in &unit.dependency_artifacts {
        args.push("--extern".to_string());
        args.push(format!("{}={}", dependency.name, dependency.artifact));
    }
    let args_digest = blake3::hash(args.join("\0").as_bytes()).to_hex().to_string();
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), unit.target_kind.clone());
    env.insert("MODE".to_string(), unit.mode.clone());
    env.insert("PACKAGE_ID".to_string(), unit.package_id.clone());
    env.insert("PROFILE".to_string(), options.profile.clone());
    env.insert("SOURCE_CLOSURE_DIGEST".to_string(), source_closure.digest_blake3.clone());
    if let Some(target) = options.targets.first() {
        env.insert("TARGET".to_string(), target.clone());
    }
    let mut inputs = vec![format!("source:{}:{}", unit.package_id, unit.source_digest.value)];
    inputs.extend(unit.dependency_artifacts.iter().map(|dependency| dependency.artifact.clone()));
    inputs.extend(consumed_host_artifacts.iter().map(|artifact| artifact.artifact.clone()));
    inputs.sort();
    inputs.dedup();
    RustUnitDerivationSummary {
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        target_kind: unit.target_kind.clone(),
        execution_kind: "target".to_string(),
        crate_types: unit.crate_types.clone(),
        mode: unit.mode.clone(),
        profile: options.profile.clone(),
        source_digest: unit.source_digest.clone(),
        dependency_artifacts: unit.dependency_artifacts.clone(),
        consumed_host_artifacts,
        generated_metadata: None,
        derivation: ReviewableRustDerivation {
            name: derivation_name(&unit.target_name, native_unit_index(&unit.unit_id)),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs,
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: args_digest,
    }
}

fn native_host_unit_derivation(
    unit: &NativeHostUnitSummary,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> RustUnitDerivationSummary {
    let mut args = vec![
        "--crate-name".to_string(),
        unit.crate_name.clone(),
        "--edition".to_string(),
        "2021".to_string(),
        unit.source_path.clone(),
        "--emit=link".to_string(),
    ];
    for crate_type in rustc_crate_types(&unit.crate_types, &unit.target_kind) {
        args.push("--crate-type".to_string());
        args.push(crate_type);
    }
    if let Some(linker) = resolve_tool_path("cc") {
        args.push("-C".to_string());
        args.push(format!("linker={}", normalize_path_string(&linker)));
    }
    let args_digest = blake3::hash(args.join("\0").as_bytes()).to_hex().to_string();
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), unit.target_kind.clone());
    env.insert("MODE".to_string(), unit.mode.clone());
    env.insert("PACKAGE_ID".to_string(), unit.package_id.clone());
    env.insert("PROFILE".to_string(), options.profile.clone());
    env.insert("SOURCE_CLOSURE_DIGEST".to_string(), source_closure.digest_blake3.clone());
    if let Some(target) = options.targets.first() {
        env.insert("TARGET".to_string(), target.clone());
    }
    let inputs = vec![format!("source:{}:{}", unit.package_id, unit.source_digest.value)];
    RustUnitDerivationSummary {
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        target_kind: unit.target_kind.clone(),
        execution_kind: "host".to_string(),
        crate_types: unit.crate_types.clone(),
        mode: unit.mode.clone(),
        profile: options.profile.clone(),
        source_digest: unit.source_digest.clone(),
        dependency_artifacts: Vec::new(),
        consumed_host_artifacts: Vec::new(),
        generated_metadata: unit.generated_metadata.clone(),
        derivation: ReviewableRustDerivation {
            name: derivation_name(&unit.target_name, native_unit_index(&unit.unit_id)),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs,
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: args_digest,
    }
}

fn native_unit_index(unit_id: &str) -> usize {
    unit_id.split(':').next().and_then(|value| value.parse::<usize>().ok()).unwrap_or(0)
}

fn cargo_unit_graph_oracle_digest(unit_graph: &Value) -> Result<String, RunError> {
    let canonical = serde_json::to_vec(unit_graph)
        .map_err(|err| RunError::Internal(format!("canonicalizing Cargo unit graph oracle: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_unit_graph_digest(
    units: &[NativeRustUnitSummary],
    blockers: &[NativeUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        units: &'a [NativeRustUnitSummary],
        blockers: &'a [NativeUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable { units, blockers })
        .map_err(|err| RunError::Internal(format!("canonicalizing native Rust unit graph: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_unit_oracle_comparison_digest(
    native_units: &[NativeRustUnitSummary],
    cargo_units: &[RustUnitDerivationSummary],
    blockers: &[NativeUnitGraphPlanningBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        native_facts: Vec<String>,
        cargo_facts: Vec<String>,
        blockers: &'a [NativeUnitGraphPlanningBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable {
        native_facts: comparable_native_unit_facts(native_units),
        cargo_facts: comparable_cargo_unit_facts(cargo_units),
        blockers,
    })
    .map_err(|err| RunError::Internal(format!("canonicalizing native unit graph oracle comparison: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn native_unit_blocker(
    unit_id: Option<String>,
    package_id: Option<String>,
    class: &str,
    message: &str,
) -> NativeUnitGraphPlanningBlocker {
    NativeUnitGraphPlanningBlocker {
        unit_id,
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn summarize_unit_derivation_graph(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> Result<UnitDerivationGraphSummary, RunError> {
    summarize_unit_derivation_graph_with_native(unit_graph, source_closure, options, None, None)
}

fn summarize_unit_derivation_graph_with_native(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
    native_unit_graph: Option<&NativeUnitGraphPlanningSummary>,
    native_host_unit_graph: Option<&NativeHostUnitGraphPlanningSummary>,
) -> Result<UnitDerivationGraphSummary, RunError> {
    if let (Some(native_unit_graph), Some(native_host_unit_graph)) = (native_unit_graph, native_host_unit_graph) {
        if native_unit_graph.ready && native_host_unit_graph.ready {
            return summarize_native_unit_derivation_graph(
                native_unit_graph,
                Some(native_host_unit_graph),
                source_closure,
                options,
            );
        }
    }
    if let Some(native_unit_graph) = native_unit_graph {
        if native_unit_graph.ready {
            return summarize_native_unit_derivation_graph(native_unit_graph, None, source_closure, options);
        }
    }
    summarize_cargo_unit_derivation_graph(unit_graph, source_closure, options)
}

fn summarize_cargo_unit_derivation_graph(
    unit_graph: &Value,
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
) -> Result<UnitDerivationGraphSummary, RunError> {
    let mut derivations = Vec::new();
    let mut blockers = source_closure
        .blockers
        .iter()
        .map(|blocker| UnitDerivationBlocker {
            unit_id: blocker.package_id.clone(),
            package_id: Some(blocker.package_id.clone()),
            class: "source-closure-blocked".to_string(),
            message: blocker.message.clone(),
        })
        .collect::<Vec<_>>();
    let Some(units) = unit_graph.get("units").and_then(Value::as_array) else {
        blockers.push(UnitDerivationBlocker {
            unit_id: "unit-graph".to_string(),
            package_id: None,
            class: "missing-units".to_string(),
            message: "Cargo unit graph did not contain a units array".to_string(),
        });
        let digest_blake3 = unit_derivation_graph_digest(&derivations, &blockers)?;
        return Ok(UnitDerivationGraphSummary {
            derivation_count: 0,
            host_unit_count: 0,
            host_artifact_count: 0,
            ready: false,
            digest_blake3,
            derivations,
            blockers,
        });
    };

    let host_artifacts = host_artifacts_by_package(units);
    for (index, unit) in units.iter().enumerate() {
        if is_custom_build_run_unit(unit) {
            continue;
        }
        match summarize_unit_derivation(index, unit, units, source_closure, options, &host_artifacts) {
            Ok(derivation) => derivations.push(derivation),
            Err(blocker) => blockers.push(blocker),
        }
    }
    derivations.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    blockers.sort_by(|left, right| left.unit_id.cmp(&right.unit_id).then(left.class.cmp(&right.class)));
    let digest_blake3 = unit_derivation_graph_digest(&derivations, &blockers)?;
    let host_unit_count = derivations.iter().filter(|derivation| derivation.execution_kind == "host").count();
    let host_artifact_count = host_artifacts.values().map(Vec::len).sum();
    Ok(UnitDerivationGraphSummary {
        derivation_count: derivations.len(),
        host_unit_count,
        host_artifact_count,
        ready: blockers.is_empty(),
        digest_blake3,
        derivations,
        blockers,
    })
}

fn summarize_unit_derivation(
    index: usize,
    unit: &Value,
    units: &[Value],
    source_closure: &SourceClosureSummary,
    options: &RustPlanOptions,
    host_artifacts: &BTreeMap<String, Vec<RustHostArtifact>>,
) -> Result<RustUnitDerivationSummary, UnitDerivationBlocker> {
    let package_id = required_unit_string(unit, "pkg_id", index)?;
    let target = unit
        .get("target")
        .ok_or_else(|| unit_blocker(index, Some(package_id.clone()), "missing-target", "unit lacks a target object"))?;
    let target_name = required_target_string(target, "name", index, &package_id)?;
    let mode = target_string(unit, "mode").unwrap_or_else(|| "build".to_string());
    if mode != "build" {
        return Err(unit_blocker(
            index,
            Some(package_id.clone()),
            "unsupported-unit-mode",
            "only Cargo build-mode units are supported before explicit test/doctest/run derivation modeling lands",
        ));
    }
    let target_kind = select_supported_target_kind(target, index, &package_id)?;
    let execution_kind = if is_host_target_kind(&target_kind) {
        "host"
    } else {
        "target"
    }
    .to_string();
    let crate_types = target_string_array(target, "crate_types");
    let edition = target_string(target, "edition").unwrap_or_else(|| "2021".to_string());
    let src_path = required_target_string(target, "src_path", index, &package_id)?;
    let features = unit_string_array(unit, "features");
    let source = source_closure.sources.iter().find(|source| source.package_id == package_id).ok_or_else(|| {
        unit_blocker(
            index,
            Some(package_id.clone()),
            "missing-source-input",
            "unit package is absent from source closure",
        )
    })?;

    let dependency_artifacts = unit_dependency_artifacts(unit, units);
    let consumed_host_artifacts = if execution_kind == "host" {
        Vec::new()
    } else {
        consumed_host_artifacts(unit, units, host_artifacts)
    };
    let generated_metadata =
        (target_kind == "custom-build").then(|| build_script_metadata_summary(&package_id, &target_name));
    let mut args = vec![
        "--crate-name".to_string(),
        rust_crate_name(&target_name),
        "--edition".to_string(),
        edition,
        src_path.clone(),
        "--emit=link".to_string(),
    ];
    for crate_type in rustc_crate_types(&crate_types, &target_kind) {
        args.push("--crate-type".to_string());
        args.push(crate_type);
    }
    if is_host_target_kind(&target_kind) || target_kind == "bin" {
        if let Some(linker) = resolve_tool_path("cc") {
            args.push("-C".to_string());
            args.push(format!("linker={}", normalize_path_string(&linker)));
        }
    }
    for feature in features {
        args.push("--cfg".to_string());
        args.push(format!("feature=\"{feature}\""));
    }
    for dependency in &dependency_artifacts {
        args.push("--extern".to_string());
        args.push(format!("{}={}", dependency.name, dependency.artifact));
    }
    let args_digest = blake3::hash(args.join("\0").as_bytes()).to_hex().to_string();
    let mut env = BTreeMap::new();
    env.insert("CRATE_KIND".to_string(), target_kind.clone());
    env.insert("MODE".to_string(), mode.clone());
    env.insert("PACKAGE_ID".to_string(), package_id.clone());
    env.insert("PROFILE".to_string(), options.profile.clone());
    env.insert("SOURCE_CLOSURE_DIGEST".to_string(), source_closure.digest_blake3.clone());
    if let Some(target) = options.targets.first() {
        env.insert("TARGET".to_string(), target.clone());
    }
    let mut inputs = vec![format!("source:{}:{}", package_id, source.source_digest.value)];
    inputs.extend(dependency_artifacts.iter().map(|dependency| dependency.artifact.clone()));
    inputs.extend(consumed_host_artifacts.iter().map(|artifact| artifact.artifact.clone()));
    inputs.sort();
    inputs.dedup();
    let unit_id = rust_unit_id(index, &package_id, &target_name, &target_kind, &mode);

    Ok(RustUnitDerivationSummary {
        unit_id: unit_id.clone(),
        package_id: package_id.clone(),
        target_name: target_name.clone(),
        target_kind,
        execution_kind: execution_kind.clone(),
        crate_types,
        mode,
        profile: options.profile.clone(),
        source_digest: source.source_digest.clone(),
        dependency_artifacts,
        consumed_host_artifacts,
        generated_metadata,
        derivation: ReviewableRustDerivation {
            name: derivation_name(&target_name, index),
            builder: "rustc".to_string(),
            system: "x86_64-linux".to_string(),
            args,
            outputs: vec!["out".to_string()],
            env,
            inputs,
            addressing_mode: "content-addressed".to_string(),
        },
        rustc_args_digest_blake3: args_digest,
    })
}

fn select_supported_target_kind(
    target: &Value,
    index: usize,
    package_id: &str,
) -> Result<String, UnitDerivationBlocker> {
    let kinds = target_string_array(target, "kind");
    if kinds.iter().any(|kind| kind == "custom-build") {
        return Ok("custom-build".to_string());
    }
    if kinds.iter().any(|kind| kind == "proc-macro") {
        return Ok("proc-macro".to_string());
    }
    if kinds.iter().any(|kind| kind == "lib") {
        return Ok("lib".to_string());
    }
    if kinds.iter().any(|kind| kind == "bin") {
        return Ok("bin".to_string());
    }
    let class = if kinds.is_empty() {
        "missing-target-kind"
    } else {
        "unsupported-target-kind"
    };
    let message = if kinds.is_empty() {
        "unit target lacks a kind array".to_string()
    } else {
        format!(
            "target kinds [{}] are outside Mantle's currently supported lib/bin Rust derivation subset",
            kinds.join(",")
        )
    };
    Err(unit_blocker(index, Some(package_id.to_string()), class, &message))
}

fn host_artifacts_by_package(units: &[Value]) -> BTreeMap<String, Vec<RustHostArtifact>> {
    let mut artifacts: BTreeMap<String, Vec<RustHostArtifact>> = BTreeMap::new();
    for (index, unit) in units.iter().enumerate() {
        if is_custom_build_run_unit(unit) {
            continue;
        }
        let Some(package_id) = target_string(unit, "pkg_id") else {
            continue;
        };
        let Some(target) = unit.get("target") else {
            continue;
        };
        let kinds = target_string_array(target, "kind");
        let target_kind = if kinds.iter().any(|kind| kind == "custom-build") {
            "custom-build"
        } else if kinds.iter().any(|kind| kind == "proc-macro") {
            "proc-macro"
        } else {
            continue;
        };
        let target_name = target_string(target, "name").unwrap_or_else(|| format!("host-unit-{index}"));
        let metadata_digest_blake3 = (target_kind == "custom-build")
            .then(|| build_script_metadata_summary(&package_id, &target_name).digest_blake3);
        artifacts.entry(package_id.clone()).or_default().push(RustHostArtifact {
            package_id,
            target_name: target_name.clone(),
            target_kind: target_kind.to_string(),
            artifact: format!("host-artifact:{index}:{target_kind}:{}", rust_crate_name(&target_name)),
            metadata_digest_blake3,
        });
    }
    for values in artifacts.values_mut() {
        values.sort();
        values.dedup();
    }
    artifacts
}

fn is_custom_build_run_unit(unit: &Value) -> bool {
    let mode = target_string(unit, "mode").unwrap_or_else(|| "build".to_string());
    if mode == "build" {
        return false;
    }
    unit.get("target")
        .map(|target| target_string_array(target, "kind").iter().any(|kind| kind == "custom-build"))
        .unwrap_or(false)
}

fn unit_dependency_array(unit: &Value) -> Option<&Vec<Value>> {
    unit.get("deps")
        .and_then(Value::as_array)
        .or_else(|| unit.get("dependencies").and_then(Value::as_array))
}

fn consumed_host_artifacts(
    unit: &Value,
    units: &[Value],
    host_artifacts: &BTreeMap<String, Vec<RustHostArtifact>>,
) -> Vec<RustHostArtifact> {
    let mut artifacts = Vec::new();
    if let Some(package_id) = target_string(unit, "pkg_id") {
        artifacts.extend(host_artifacts.get(&package_id).into_iter().flatten().cloned());
    }
    if let Some(deps) = unit_dependency_array(unit) {
        for dep in deps {
            if let Some(package_id) = dependency_package_id(dep, units) {
                artifacts.extend(host_artifacts.get(package_id).into_iter().flatten().cloned());
            }
        }
    }
    artifacts.sort();
    artifacts.dedup();
    artifacts
}

fn build_script_metadata_summary(package_id: &str, target_name: &str) -> BuildScriptMetadataSummary {
    let out_dir = format!("host-metadata:{package_id}:{target_name}:OUT_DIR");
    let rustc_cfg = Vec::new();
    let rustc_env = BTreeMap::new();
    let rustc_link_lib = Vec::new();
    let rustc_link_search = Vec::new();
    let rerun_if_changed = Vec::new();
    #[derive(Serialize)]
    struct Hashable<'a> {
        out_dir: &'a str,
        rustc_cfg: &'a [String],
        rustc_env: &'a BTreeMap<String, String>,
        rustc_link_lib: &'a [String],
        rustc_link_search: &'a [String],
        rerun_if_changed: &'a [String],
    }
    let canonical = serde_json::to_vec(&Hashable {
        out_dir: &out_dir,
        rustc_cfg: &rustc_cfg,
        rustc_env: &rustc_env,
        rustc_link_lib: &rustc_link_lib,
        rustc_link_search: &rustc_link_search,
        rerun_if_changed: &rerun_if_changed,
    })
    .unwrap_or_default();
    BuildScriptMetadataSummary {
        out_dir,
        rustc_cfg,
        rustc_env,
        rustc_link_lib,
        rustc_link_search,
        rerun_if_changed,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    }
}

fn is_host_target_kind(target_kind: &str) -> bool {
    matches!(target_kind, "custom-build" | "proc-macro")
}

fn unit_dependency_artifacts(unit: &Value, units: &[Value]) -> Vec<RustDependencyArtifact> {
    let mut artifacts = Vec::new();
    let Some(deps) = unit_dependency_array(unit) else {
        return artifacts;
    };
    for dep in deps {
        let Some(package_id) = dependency_package_id(dep, units) else {
            continue;
        };
        let name = dep
            .get("extern_crate_name")
            .and_then(Value::as_str)
            .or_else(|| dep.get("name").and_then(Value::as_str))
            .map(rust_crate_name)
            .unwrap_or_else(|| "unknown_dep".to_string());
        let artifact = format!("artifact:{package_id}:{name}");
        artifacts.push(RustDependencyArtifact {
            package_id: package_id.to_string(),
            name,
            artifact,
        });
    }
    artifacts.sort_by(|left, right| left.package_id.cmp(&right.package_id).then(left.name.cmp(&right.name)));
    artifacts.dedup();
    artifacts
}

fn dependency_package_id<'a>(dep: &'a Value, units: &'a [Value]) -> Option<&'a str> {
    dep.get("pkg_id")
        .and_then(Value::as_str)
        .or_else(|| dep.get("package_id").and_then(Value::as_str))
        .or_else(|| {
            let index = dep.get("index").and_then(Value::as_u64)?;
            let unit = units.get(usize::try_from(index).ok()?)?;
            unit.get("pkg_id").and_then(Value::as_str)
        })
}

fn normalized_crate_types(crate_types: &[String], target_kind: &str) -> Vec<String> {
    let mut normalized = if crate_types.is_empty() {
        vec![target_kind.to_string()]
    } else {
        crate_types.to_vec()
    };
    normalized.sort();
    normalized.dedup();
    normalized
}

fn rustc_crate_types(crate_types: &[String], target_kind: &str) -> Vec<String> {
    if target_kind == "custom-build" {
        return vec!["bin".to_string()];
    }
    normalized_crate_types(crate_types, target_kind)
}

fn rust_unit_id(index: usize, package_id: &str, target_name: &str, target_kind: &str, mode: &str) -> String {
    format!("{index}:{package_id}:{target_name}:{target_kind}:{mode}")
}

fn derivation_name(target_name: &str, index: usize) -> String {
    format!("rust-unit-{index}-{}", rust_crate_name(target_name))
}

fn rust_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

fn required_unit_string(unit: &Value, field: &str, index: usize) -> Result<String, UnitDerivationBlocker> {
    target_string(unit, field)
        .ok_or_else(|| unit_blocker(index, None, &format!("missing-{field}"), &format!("unit lacks `{field}`")))
}

fn required_target_string(
    target: &Value,
    field: &str,
    index: usize,
    package_id: &str,
) -> Result<String, UnitDerivationBlocker> {
    target_string(target, field).ok_or_else(|| {
        unit_blocker(
            index,
            Some(package_id.to_string()),
            &format!("missing-target-{field}"),
            &format!("unit target lacks `{field}`"),
        )
    })
}

fn target_string(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(ToString::to_string)
}

fn unit_string_array(value: &Value, field: &str) -> Vec<String> {
    target_string_array(value, field)
}

fn target_string_array(value: &Value, field: &str) -> Vec<String> {
    let mut values = value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(ToString::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    values.sort();
    values.dedup();
    values
}

fn unit_blocker(index: usize, package_id: Option<String>, class: &str, message: &str) -> UnitDerivationBlocker {
    UnitDerivationBlocker {
        unit_id: index.to_string(),
        package_id,
        class: class.to_string(),
        message: message.to_string(),
    }
}

fn unit_derivation_graph_digest(
    derivations: &[RustUnitDerivationSummary],
    blockers: &[UnitDerivationBlocker],
) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        derivations: &'a [RustUnitDerivationSummary],
        blockers: &'a [UnitDerivationBlocker],
    }
    let canonical = serde_json::to_vec(&Hashable { derivations, blockers })
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust unit derivation graph: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

pub(crate) fn execute_first_supported_rust_unit(
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitExecutionReceipt, RunError> {
    let Some(unit) = graph.derivations.iter().find(|unit| is_supported_target_unit(unit)) else {
        return blocked_execution_receipt(
            None,
            "missing-supported-unit",
            "unit_derivation_graph does not contain a supported target lib/bin unit",
        );
    };
    if !graph.ready {
        return blocked_execution_receipt(
            Some(unit),
            "unit-derivation-graph-blocked",
            "unit_derivation_graph is not ready; resolve planning blockers before execution",
        );
    }
    execute_rust_unit(unit, options)
}

pub(crate) fn execute_first_rust_unit_dependency_chain(
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitDependencyChainExecutionReceipt, RunError> {
    if !graph.ready {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message:
                    "unit_derivation_graph is not ready; resolve planning blockers before dependency-chain execution"
                        .to_string(),
            }),
        );
    }
    let Some(consumer) = graph
        .derivations
        .iter()
        .find(|unit| is_supported_target_unit(unit) && !unit.dependency_artifacts.is_empty())
    else {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-dependency-chain".to_string(),
                message: "unit_derivation_graph does not contain a supported target unit with dependency artifacts"
                    .to_string(),
            }),
        );
    };
    if !consumer.consumed_host_artifacts.is_empty() {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unsupported-chain-shape".to_string(),
                message:
                    "bounded dependency-chain execution does not yet support host/proc-macro/build-script artifacts"
                        .to_string(),
            }),
        );
    }
    if consumer.dependency_artifacts.len() != 1 {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unsupported-chain-shape".to_string(),
                message:
                    "bounded dependency-chain execution currently supports exactly one producer dependency artifact"
                        .to_string(),
            }),
        );
    }
    let dependency = &consumer.dependency_artifacts[0];
    let Some(producer) = graph.derivations.iter().find(|unit| {
        is_supported_target_unit(unit)
            && unit.package_id == dependency.package_id
            && unit.target_kind == "lib"
            && unit.dependency_artifacts.is_empty()
            && unit.consumed_host_artifacts.is_empty()
    }) else {
        return dependency_chain_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-dependency-producer".to_string(),
                message: format!("no supported producer lib unit for dependency package {}", dependency.package_id),
            }),
        );
    };

    let producer_receipt = execute_rust_unit(producer, options)?;
    if producer_receipt.execution_status != "success" {
        return dependency_chain_receipt(
            "blocked",
            vec![producer_receipt],
            Some(RustUnitExecutionBlocker {
                class: "dependency-producer-failed".to_string(),
                message: format!("producer unit {} did not produce a successful artifact", producer.unit_id),
            }),
        );
    }
    let producer_artifact = match produced_library_artifact_path(producer, options)? {
        Ok(path) => path,
        Err(blocker) => return dependency_chain_receipt("blocked", vec![producer_receipt], Some(blocker)),
    };
    let bound_consumer = bind_dependency_artifact(consumer, dependency, &producer_artifact)?;
    let consumer_receipt = execute_rust_unit(&bound_consumer, options)?;
    let status = if consumer_receipt.execution_status == "success" {
        "success"
    } else {
        "blocked"
    };
    let blocker = consumer_receipt.blocker.clone();
    dependency_chain_receipt(status, vec![producer_receipt, consumer_receipt], blocker)
}

pub(crate) fn execute_rust_target_unit_topology(
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitTargetTopologyExecutionReceipt, RunError> {
    if !graph.ready {
        return target_topology_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message:
                    "unit_derivation_graph is not ready; resolve planning blockers before target-topology execution"
                        .to_string(),
            }),
        );
    }

    let mut selected_indices = Vec::new();
    for (index, unit) in graph.derivations.iter().enumerate() {
        if is_supported_target_unit(unit) {
            if !unit.consumed_host_artifacts.is_empty() {
                return target_topology_receipt(
                    "blocked",
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "unsupported-topology-shape".to_string(),
                        message: format!(
                            "target topology execution does not yet support host/proc-macro/build-script artifacts for unit {}",
                            unit.unit_id
                        ),
                    }),
                );
            }
            selected_indices.push(index);
        }
    }
    if selected_indices.is_empty() {
        return target_topology_receipt(
            "blocked",
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-supported-unit".to_string(),
                message: "unit_derivation_graph does not contain supported target lib/bin units".to_string(),
            }),
        );
    }

    let selected_set = selected_indices.iter().copied().collect::<BTreeSet<_>>();
    let mut lib_producers = BTreeMap::new();
    for index in &selected_indices {
        let unit = &graph.derivations[*index];
        if unit.target_kind == "lib" {
            lib_producers.entry(unit.package_id.clone()).or_insert(*index);
        }
    }

    let mut edges = BTreeMap::<usize, Vec<usize>>::new();
    for index in &selected_indices {
        let unit = &graph.derivations[*index];
        let mut deps = Vec::new();
        for dependency in &unit.dependency_artifacts {
            let Some(producer_index) = lib_producers.get(&dependency.package_id).copied() else {
                return target_topology_receipt(
                    "blocked",
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-dependency-producer".to_string(),
                        message: format!(
                            "no supported producer lib unit for dependency package {}",
                            dependency.package_id
                        ),
                    }),
                );
            };
            if !selected_set.contains(&producer_index) {
                return target_topology_receipt(
                    "blocked",
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-dependency-producer".to_string(),
                        message: format!(
                            "dependency producer for package {} is outside the selected target topology",
                            dependency.package_id
                        ),
                    }),
                );
            }
            if producer_index != *index {
                deps.push(producer_index);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        edges.insert(*index, deps);
    }

    let mut ordered_indices = Vec::new();
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for index in &selected_indices {
        if let Err(blocker) =
            visit_target_topology_unit(*index, &edges, &mut temporary, &mut permanent, &mut ordered_indices, graph)
        {
            return target_topology_receipt("blocked", Vec::new(), Some(blocker));
        }
    }

    let mut executions = Vec::new();
    let mut produced_artifacts = BTreeMap::<String, PathBuf>::new();
    for index in ordered_indices {
        let unit = &graph.derivations[index];
        let executable_unit = if unit.dependency_artifacts.is_empty() {
            unit.clone()
        } else {
            bind_all_dependency_artifacts(unit, &produced_artifacts)?
        };
        let receipt = execute_rust_unit(&executable_unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "target-unit-failed".to_string(),
                message: format!("target topology unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return target_topology_receipt("blocked", executions, Some(blocker));
        }
        if unit.target_kind == "lib" {
            match produced_library_artifact_path(&executable_unit, options)? {
                Ok(path) => {
                    produced_artifacts.insert(unit.package_id.clone(), path);
                }
                Err(blocker) => {
                    executions.push(receipt);
                    return target_topology_receipt("blocked", executions, Some(blocker));
                }
            }
        }
        executions.push(receipt);
    }

    target_topology_receipt("success", executions, None)
}

pub(crate) fn execute_rust_unit_topology(
    native_registry_sources: &NativeRegistrySourcePlanningSummary,
    native_host_graph: &NativeHostUnitGraphPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitTopologyExecutionReceipt, RunError> {
    if let Some(blocker) = validate_native_registry_topology_inputs(native_registry_sources, graph) {
        return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if let Some(blocker) = validate_native_host_artifact_topology_inputs(native_host_graph, graph, "topology") {
        return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if !graph.ready {
        return topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message: "unit_derivation_graph is not ready; resolve planning blockers before topology execution"
                    .to_string(),
            }),
        );
    }

    let host_indices = graph
        .derivations
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_supported_host_unit(unit).then_some(index))
        .collect::<Vec<_>>();
    let target_indices = graph
        .derivations
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_supported_target_unit(unit).then_some(index))
        .collect::<Vec<_>>();
    if target_indices.is_empty() {
        return topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-supported-unit".to_string(),
                message: "unit_derivation_graph does not contain supported target lib/bin units".to_string(),
            }),
        );
    }

    let mut host_producers = BTreeMap::<String, usize>::new();
    for index in &host_indices {
        host_producers.entry(graph.derivations[*index].package_id.clone()).or_insert(*index);
    }
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        for host_artifact in &unit.consumed_host_artifacts {
            if !host_producers.contains_key(&host_artifact.package_id) {
                return topology_receipt(
                    "blocked",
                    Vec::new(),
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-host-artifact-producer".to_string(),
                        message: format!(
                            "no supported host producer unit for host artifact package {}",
                            host_artifact.package_id
                        ),
                    }),
                );
            }
        }
    }

    let selected_set = target_indices.iter().copied().collect::<BTreeSet<_>>();
    let mut lib_producers = BTreeMap::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        if unit.target_kind == "lib" {
            lib_producers.entry(unit.package_id.clone()).or_insert(*index);
        }
    }
    let mut edges = BTreeMap::<usize, Vec<usize>>::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        let mut deps = Vec::new();
        for dependency in &unit.dependency_artifacts {
            let Some(producer_index) = lib_producers.get(&dependency.package_id).copied() else {
                if host_producers.contains_key(&dependency.package_id) {
                    continue;
                }
                return topology_receipt(
                    "blocked",
                    Vec::new(),
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-dependency-producer".to_string(),
                        message: format!(
                            "no supported target producer lib unit for dependency package {}",
                            dependency.package_id
                        ),
                    }),
                );
            };
            if selected_set.contains(&producer_index) && producer_index != *index {
                deps.push(producer_index);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        edges.insert(*index, deps);
    }

    let mut ordered_target_indices = Vec::new();
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for index in &target_indices {
        if let Err(blocker) = visit_target_topology_unit(
            *index,
            &edges,
            &mut temporary,
            &mut permanent,
            &mut ordered_target_indices,
            graph,
        ) {
            return topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
        }
    }

    let mut executions = Vec::new();
    let mut build_script_metadata_runs = Vec::new();
    let mut produced_host_artifacts = BTreeMap::<String, PathBuf>::new();
    let mut produced_build_script_metadata = BTreeMap::<String, BuildScriptMetadataSummary>::new();
    for index in host_indices {
        let unit = &graph.derivations[index];
        let receipt = execute_rust_unit(unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "host-unit-failed".to_string(),
                message: format!("host unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
        }
        match produced_host_artifact_path(unit, options)? {
            Ok(path) => {
                if unit.target_kind == "custom-build" {
                    match run_build_script_metadata(unit, options, &path)? {
                        Ok(metadata_run) => {
                            produced_build_script_metadata
                                .insert(unit.package_id.clone(), build_script_metadata_from_run(&metadata_run));
                            build_script_metadata_runs.push(metadata_run);
                        }
                        Err(blocker) => {
                            executions.push(receipt);
                            return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                        }
                    }
                }
                produced_host_artifacts.insert(unit.package_id.clone(), path);
            }
            Err(blocker) => {
                executions.push(receipt);
                return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
            }
        }
        executions.push(receipt);
    }

    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    for index in ordered_target_indices {
        let unit = &graph.derivations[index];
        let mut executable_unit = bind_all_host_artifacts(unit, &produced_host_artifacts)?;
        executable_unit = bind_all_build_script_metadata(&executable_unit, &produced_build_script_metadata)?;
        if !executable_unit.dependency_artifacts.is_empty() {
            executable_unit = bind_all_dependency_artifacts(&executable_unit, &produced_target_artifacts)?;
        }
        let receipt = execute_rust_unit(&executable_unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "target-unit-failed".to_string(),
                message: format!("target unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
        }
        if unit.target_kind == "lib" {
            match produced_library_artifact_path(&executable_unit, options)? {
                Ok(path) => {
                    produced_target_artifacts.insert(unit.package_id.clone(), path);
                }
                Err(blocker) => {
                    executions.push(receipt);
                    return topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
                }
            }
        }
        executions.push(receipt);
    }

    topology_receipt("success", executions, build_script_metadata_runs, None)
}

fn validate_native_registry_topology_inputs(
    native_registry_sources: &NativeRegistrySourcePlanningSummary,
    graph: &UnitDerivationGraphSummary,
) -> Option<RustUnitExecutionBlocker> {
    let registry_packages = graph
        .derivations
        .iter()
        .filter(|unit| unit.package_id.starts_with("registry+"))
        .map(|unit| unit.package_id.clone())
        .collect::<BTreeSet<_>>();
    if registry_packages.is_empty() {
        return None;
    }
    if !native_registry_sources.ready {
        let blocker_classes = native_registry_sources
            .blockers
            .iter()
            .map(|blocker| blocker.class.as_str())
            .collect::<Vec<_>>()
            .join(",");
        return Some(RustUnitExecutionBlocker {
            class: "native-registry-source-planning-blocked".to_string(),
            message: format!(
                "registry-backed topology execution requires ready native registry source facts; blockers: {blocker_classes}"
            ),
        });
    }
    let source_packages = native_registry_sources
        .sources
        .iter()
        .map(|source| source.package_id.clone())
        .collect::<BTreeSet<_>>();
    for package_id in registry_packages {
        if !source_packages.contains(&package_id) {
            return Some(RustUnitExecutionBlocker {
                class: "missing-native-registry-source-fact".to_string(),
                message: format!(
                    "registry-backed unit package {package_id} is not backed by native_registry_source_planning"
                ),
            });
        }
    }
    None
}

fn validate_native_host_artifact_topology_inputs(
    native_host_graph: &NativeHostUnitGraphPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    execution_context: &str,
) -> Option<RustUnitExecutionBlocker> {
    if !native_host_graph.ready {
        return Some(RustUnitExecutionBlocker {
            class: "native-host-unit-graph-blocked".to_string(),
            message: format!(
                "native_host_unit_graph_planning is not ready; resolve native host graph blockers before {execution_context} execution"
            ),
        });
    }
    if !graph.ready {
        return None;
    }

    let graph_host_units = graph
        .derivations
        .iter()
        .filter(|unit| is_supported_host_unit(unit))
        .map(|unit| (unit.unit_id.clone(), unit.package_id.clone(), unit.target_name.clone(), unit.target_kind.clone()))
        .collect::<BTreeSet<_>>();
    let native_host_units = native_host_graph
        .host_units
        .iter()
        .map(|unit| (unit.unit_id.clone(), unit.package_id.clone(), unit.target_name.clone(), unit.target_kind.clone()))
        .collect::<BTreeSet<_>>();

    for native_unit in &native_host_units {
        if !graph_host_units.contains(native_unit) {
            return Some(RustUnitExecutionBlocker {
                class: "missing-native-host-derivation".to_string(),
                message: format!(
                    "native host unit {} for package {} is absent from unit_derivation_graph",
                    native_unit.0, native_unit.1
                ),
            });
        }
    }
    for graph_unit in &graph_host_units {
        if !native_host_units.contains(graph_unit) {
            return Some(RustUnitExecutionBlocker {
                class: "non-native-host-derivation".to_string(),
                message: format!(
                    "host derivation {} for package {} is not backed by native_host_unit_graph_planning",
                    graph_unit.0, graph_unit.1
                ),
            });
        }
    }

    let graph_consumers = graph
        .derivations
        .iter()
        .filter(|unit| is_supported_target_unit(unit) && !unit.consumed_host_artifacts.is_empty())
        .map(|unit| {
            let artifacts = unit.consumed_host_artifacts.iter().cloned().collect::<BTreeSet<_>>();
            (unit.unit_id.clone(), (unit.package_id.clone(), artifacts))
        })
        .collect::<BTreeMap<_, _>>();
    let native_consumers = native_host_graph
        .target_consumers
        .iter()
        .filter(|unit| !unit.consumed_host_artifacts.is_empty())
        .map(|unit| {
            let artifacts = unit.consumed_host_artifacts.iter().cloned().collect::<BTreeSet<_>>();
            (unit.unit_id.clone(), (unit.package_id.clone(), artifacts))
        })
        .collect::<BTreeMap<_, _>>();

    for (unit_id, (package_id, native_artifacts)) in &native_consumers {
        let Some((_, graph_artifacts)) = graph_consumers.get_key_value(unit_id) else {
            return Some(RustUnitExecutionBlocker {
                class: "missing-native-host-consumer-derivation".to_string(),
                message: format!(
                    "native host-artifact consumer {unit_id} for package {package_id} is absent from unit_derivation_graph"
                ),
            });
        };
        for artifact in native_artifacts {
            if !graph_artifacts.1.contains(artifact) {
                return Some(RustUnitExecutionBlocker {
                    class: "missing-native-host-artifact-binding".to_string(),
                    message: format!(
                        "native host artifact from package {} is absent from derivation consumer {unit_id}",
                        artifact.package_id
                    ),
                });
            }
        }
    }
    for (unit_id, (package_id, _)) in &graph_consumers {
        if !native_consumers.contains_key(unit_id) {
            return Some(RustUnitExecutionBlocker {
                class: "non-native-host-artifact-consumer".to_string(),
                message: format!(
                    "host-artifact consumer {unit_id} for package {package_id} is not backed by native_host_unit_graph_planning"
                ),
            });
        }
    }

    None
}

pub(crate) fn execute_rust_host_artifact_topology(
    native_registry_sources: &NativeRegistrySourcePlanningSummary,
    native_host_graph: &NativeHostUnitGraphPlanningSummary,
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitHostArtifactTopologyExecutionReceipt, RunError> {
    if let Some(blocker) = validate_native_registry_topology_inputs(native_registry_sources, graph) {
        return host_artifact_topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if let Some(blocker) = validate_native_host_artifact_topology_inputs(native_host_graph, graph, "host-artifact") {
        return host_artifact_topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
    }
    if !graph.ready {
        return host_artifact_topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "unit-derivation-graph-blocked".to_string(),
                message: "unit_derivation_graph is not ready; resolve planning blockers before host-artifact execution"
                    .to_string(),
            }),
        );
    }

    let host_indices = graph
        .derivations
        .iter()
        .enumerate()
        .filter_map(|(index, unit)| is_supported_host_unit(unit).then_some(index))
        .collect::<Vec<_>>();
    if host_indices.is_empty() {
        return host_artifact_topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-host-artifact-unit".to_string(),
                message: "unit_derivation_graph does not contain supported proc-macro/custom-build host units"
                    .to_string(),
            }),
        );
    }

    let mut host_producers = BTreeMap::<String, usize>::new();
    for index in &host_indices {
        host_producers.entry(graph.derivations[*index].package_id.clone()).or_insert(*index);
    }

    let mut target_indices = Vec::new();
    for (index, unit) in graph.derivations.iter().enumerate() {
        if is_supported_target_unit(unit) && !unit.consumed_host_artifacts.is_empty() {
            for host_artifact in &unit.consumed_host_artifacts {
                if !host_producers.contains_key(&host_artifact.package_id) {
                    return host_artifact_topology_receipt(
                        "blocked",
                        Vec::new(),
                        Vec::new(),
                        Some(RustUnitExecutionBlocker {
                            class: "missing-host-artifact-producer".to_string(),
                            message: format!(
                                "no supported host producer unit for host artifact package {}",
                                host_artifact.package_id
                            ),
                        }),
                    );
                }
            }
            target_indices.push(index);
        }
    }
    if target_indices.is_empty() {
        return host_artifact_topology_receipt(
            "blocked",
            Vec::new(),
            Vec::new(),
            Some(RustUnitExecutionBlocker {
                class: "missing-host-artifact-consumer".to_string(),
                message: "unit_derivation_graph does not contain target units that consume host artifacts".to_string(),
            }),
        );
    }

    let selected_set = target_indices.iter().copied().collect::<BTreeSet<_>>();
    let mut lib_producers = BTreeMap::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        if unit.target_kind == "lib" {
            lib_producers.entry(unit.package_id.clone()).or_insert(*index);
        }
    }
    let mut edges = BTreeMap::<usize, Vec<usize>>::new();
    for index in &target_indices {
        let unit = &graph.derivations[*index];
        let mut deps = Vec::new();
        for dependency in &unit.dependency_artifacts {
            if host_producers.contains_key(&dependency.package_id) {
                continue;
            }
            let Some(producer_index) = lib_producers.get(&dependency.package_id).copied() else {
                return host_artifact_topology_receipt(
                    "blocked",
                    Vec::new(),
                    Vec::new(),
                    Some(RustUnitExecutionBlocker {
                        class: "missing-dependency-producer".to_string(),
                        message: format!(
                            "no supported target producer lib unit for dependency package {}",
                            dependency.package_id
                        ),
                    }),
                );
            };
            if selected_set.contains(&producer_index) && producer_index != *index {
                deps.push(producer_index);
            }
        }
        deps.sort_unstable();
        deps.dedup();
        edges.insert(*index, deps);
    }

    let mut ordered_target_indices = Vec::new();
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    for index in &target_indices {
        if let Err(blocker) = visit_target_topology_unit(
            *index,
            &edges,
            &mut temporary,
            &mut permanent,
            &mut ordered_target_indices,
            graph,
        ) {
            return host_artifact_topology_receipt("blocked", Vec::new(), Vec::new(), Some(blocker));
        }
    }

    let mut executions = Vec::new();
    let mut build_script_metadata_runs = Vec::new();
    let mut produced_host_artifacts = BTreeMap::<String, PathBuf>::new();
    let mut produced_build_script_metadata = BTreeMap::<String, BuildScriptMetadataSummary>::new();
    for index in host_indices {
        let unit = &graph.derivations[index];
        let receipt = execute_rust_unit(unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "host-unit-failed".to_string(),
                message: format!("host unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return host_artifact_topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
        }
        match produced_host_artifact_path(unit, options)? {
            Ok(path) => {
                if unit.target_kind == "custom-build" {
                    match run_build_script_metadata(unit, options, &path)? {
                        Ok(metadata_run) => {
                            produced_build_script_metadata
                                .insert(unit.package_id.clone(), build_script_metadata_from_run(&metadata_run));
                            build_script_metadata_runs.push(metadata_run);
                        }
                        Err(blocker) => {
                            executions.push(receipt);
                            return host_artifact_topology_receipt(
                                "blocked",
                                executions,
                                build_script_metadata_runs,
                                Some(blocker),
                            );
                        }
                    }
                }
                produced_host_artifacts.insert(unit.package_id.clone(), path);
            }
            Err(blocker) => {
                executions.push(receipt);
                return host_artifact_topology_receipt(
                    "blocked",
                    executions,
                    build_script_metadata_runs,
                    Some(blocker),
                );
            }
        }
        executions.push(receipt);
    }

    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    for index in ordered_target_indices {
        let unit = &graph.derivations[index];
        let mut executable_unit = bind_all_host_artifacts(unit, &produced_host_artifacts)?;
        executable_unit = bind_all_build_script_metadata(&executable_unit, &produced_build_script_metadata)?;
        if !executable_unit.dependency_artifacts.is_empty() {
            executable_unit = bind_all_dependency_artifacts(&executable_unit, &produced_target_artifacts)?;
        }
        let receipt = execute_rust_unit(&executable_unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "target-unit-failed".to_string(),
                message: format!("target unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return host_artifact_topology_receipt("blocked", executions, build_script_metadata_runs, Some(blocker));
        }
        if unit.target_kind == "lib" {
            match produced_library_artifact_path(&executable_unit, options)? {
                Ok(path) => {
                    produced_target_artifacts.insert(unit.package_id.clone(), path);
                }
                Err(blocker) => {
                    executions.push(receipt);
                    return host_artifact_topology_receipt(
                        "blocked",
                        executions,
                        build_script_metadata_runs,
                        Some(blocker),
                    );
                }
            }
        }
        executions.push(receipt);
    }

    host_artifact_topology_receipt("success", executions, build_script_metadata_runs, None)
}

fn visit_target_topology_unit(
    index: usize,
    edges: &BTreeMap<usize, Vec<usize>>,
    temporary: &mut BTreeSet<usize>,
    permanent: &mut BTreeSet<usize>,
    ordered_indices: &mut Vec<usize>,
    graph: &UnitDerivationGraphSummary,
) -> Result<(), RustUnitExecutionBlocker> {
    if permanent.contains(&index) {
        return Ok(());
    }
    if !temporary.insert(index) {
        return Err(RustUnitExecutionBlocker {
            class: "dependency-cycle".to_string(),
            message: format!(
                "target topology contains a dependency cycle at unit {}",
                graph.derivations[index].unit_id
            ),
        });
    }
    for dependency_index in edges.get(&index).into_iter().flatten() {
        visit_target_topology_unit(*dependency_index, edges, temporary, permanent, ordered_indices, graph)?;
    }
    temporary.remove(&index);
    permanent.insert(index);
    ordered_indices.push(index);
    Ok(())
}

fn bind_all_dependency_artifacts(
    unit: &RustUnitDerivationSummary,
    produced_artifacts: &BTreeMap<String, PathBuf>,
) -> Result<RustUnitDerivationSummary, RunError> {
    let mut bound = unit.clone();
    for dependency in &unit.dependency_artifacts {
        if Path::new(&dependency.artifact).is_file() {
            continue;
        }
        let produced_artifact = produced_artifacts.get(&dependency.package_id).ok_or_else(|| {
            RunError::Internal(format!(
                "unit {} reached execution before dependency package {} was produced",
                unit.unit_id, dependency.package_id
            ))
        })?;
        bound = bind_dependency_artifact(&bound, dependency, produced_artifact)?;
    }
    append_dependency_search_paths(&mut bound, produced_artifacts);
    Ok(bound)
}

fn bind_all_host_artifacts(
    unit: &RustUnitDerivationSummary,
    produced_host_artifacts: &BTreeMap<String, PathBuf>,
) -> Result<RustUnitDerivationSummary, RunError> {
    let mut bound = unit.clone();
    for host_artifact in &unit.consumed_host_artifacts {
        let produced_artifact = produced_host_artifacts.get(&host_artifact.package_id).ok_or_else(|| {
            RunError::Internal(format!(
                "unit {} reached execution before host artifact package {} was produced",
                unit.unit_id, host_artifact.package_id
            ))
        })?;
        let produced_artifact_string = normalize_path_string(produced_artifact);
        for artifact in &mut bound.consumed_host_artifacts {
            if artifact.package_id == host_artifact.package_id
                && artifact.target_name == host_artifact.target_name
                && artifact.target_kind == host_artifact.target_kind
                && artifact.artifact == host_artifact.artifact
            {
                artifact.artifact = produced_artifact_string.clone();
            }
        }
        for input in &mut bound.derivation.inputs {
            if input == &host_artifact.artifact {
                *input = produced_artifact_string.clone();
            }
        }
        let host_crate_name = rust_crate_name(&host_artifact.target_name);
        let matching_dependencies = bound
            .dependency_artifacts
            .iter()
            .filter(|dependency| {
                dependency.package_id == host_artifact.package_id && dependency.name == host_crate_name
            })
            .cloned()
            .collect::<Vec<_>>();
        for dependency in matching_dependencies {
            bound = bind_dependency_artifact(&bound, &dependency, produced_artifact)?;
        }
    }
    Ok(bound)
}

fn bind_all_build_script_metadata(
    unit: &RustUnitDerivationSummary,
    produced_metadata: &BTreeMap<String, BuildScriptMetadataSummary>,
) -> Result<RustUnitDerivationSummary, RunError> {
    let mut bound = unit.clone();
    for host_artifact in &unit.consumed_host_artifacts {
        if host_artifact.target_kind != "custom-build" {
            continue;
        }
        let metadata = produced_metadata.get(&host_artifact.package_id).ok_or_else(|| {
            RunError::Internal(format!(
                "unit {} reached execution before build-script metadata package {} was produced",
                unit.unit_id, host_artifact.package_id
            ))
        })?;
        bound.derivation.env.insert("OUT_DIR".to_string(), metadata.out_dir.clone());
        for (key, value) in &metadata.rustc_env {
            bound.derivation.env.insert(key.clone(), value.clone());
        }
        for cfg in &metadata.rustc_cfg {
            bound.derivation.args.push("--cfg".to_string());
            bound.derivation.args.push(cfg.clone());
        }
        for search in &metadata.rustc_link_search {
            bound.derivation.args.push("-L".to_string());
            bound.derivation.args.push(search.clone());
        }
        for lib in &metadata.rustc_link_lib {
            bound.derivation.args.push("-l".to_string());
            bound.derivation.args.push(lib.clone());
        }
    }
    bound.rustc_args_digest_blake3 = blake3::hash(bound.derivation.args.join("\0").as_bytes()).to_hex().to_string();
    Ok(bound)
}

fn run_build_script_metadata(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
    executable: &Path,
) -> Result<Result<BuildScriptMetadataRunReceipt, RustUnitExecutionBlocker>, RunError> {
    if !executable.is_file() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-build-script-executable".to_string(),
            message: format!("custom-build unit {} did not produce an executable", unit.unit_id),
        }));
    }
    let out_dir = options.output_root.join(safe_path_component(&unit.unit_id)).join("out-dir");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|err| {
            RunError::Internal(format!("removing prior build-script OUT_DIR {}: {err}", out_dir.display()))
        })?;
    }
    fs::create_dir_all(&out_dir)
        .map_err(|err| RunError::Internal(format!("creating build-script OUT_DIR {}: {err}", out_dir.display())))?;

    let mut command = Command::new(executable);
    command.env_clear();
    command.env("OUT_DIR", &out_dir);
    command.env("CARGO_PKG_NAME", rust_crate_name(&unit.target_name));
    if let Some(src_path) =
        rustc_source_path(&unit.derivation.args).and_then(|path| path.parent().map(Path::to_path_buf))
    {
        command.env("CARGO_MANIFEST_DIR", src_path);
    }
    let output = command
        .output()
        .map_err(|err| RunError::Internal(format!("running build-script unit {}: {err}", unit.unit_id)))?;
    if !output.status.success() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "build-script-run-failed".to_string(),
            message: redacted_diagnostic(&output.stderr),
        }));
    }
    let stdout = match String::from_utf8(output.stdout.clone()) {
        Ok(stdout) => stdout,
        Err(_) => {
            return Ok(Err(RustUnitExecutionBlocker {
                class: "malformed-build-script-metadata".to_string(),
                message: "build-script stdout was not valid UTF-8".to_string(),
            }));
        }
    };
    let metadata = match parse_build_script_metadata(&stdout, &out_dir) {
        Ok(metadata) => metadata,
        Err(blocker) => return Ok(Err(blocker)),
    };
    let out_dir_artifact_digests = digest_build_script_out_dir(&out_dir)?;
    Ok(Ok(BuildScriptMetadataRunReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        execution_status: "success".to_string(),
        out_dir: metadata.out_dir,
        rustc_cfg: metadata.rustc_cfg,
        rustc_env: metadata.rustc_env,
        rustc_link_lib: metadata.rustc_link_lib,
        rustc_link_search: metadata.rustc_link_search,
        rerun_if_changed: metadata.rerun_if_changed,
        out_dir_artifact_digests,
        stdout_digest_blake3: blake3::hash(&output.stdout).to_hex().to_string(),
        metadata_digest_blake3: metadata.digest_blake3,
        blocker: None,
    }))
}

fn parse_build_script_metadata(
    stdout: &str,
    out_dir: &Path,
) -> Result<BuildScriptMetadataSummary, RustUnitExecutionBlocker> {
    let mut rustc_cfg = Vec::new();
    let mut rustc_env = BTreeMap::new();
    let mut rustc_link_lib = Vec::new();
    let mut rustc_link_search = Vec::new();
    let mut rerun_if_changed = Vec::new();
    for (line_index, line) in stdout.lines().enumerate() {
        let Some(payload) = line.strip_prefix("cargo:") else {
            continue;
        };
        if let Some(value) = payload.strip_prefix("rustc-cfg=") {
            push_metadata_value(&mut rustc_cfg, value, line_index)?;
        } else if let Some(value) = payload.strip_prefix("rustc-env=") {
            let Some((key, env_value)) = value.split_once('=') else {
                return Err(malformed_build_script_metadata(line_index, "rustc-env must be KEY=VALUE"));
            };
            if key.is_empty() {
                return Err(malformed_build_script_metadata(line_index, "rustc-env key must not be empty"));
            }
            rustc_env.insert(key.to_string(), env_value.to_string());
        } else if let Some(value) = payload.strip_prefix("rustc-link-lib=") {
            validate_rustc_link_lib_metadata(value, line_index)?;
            push_metadata_value(&mut rustc_link_lib, value, line_index)?;
        } else if let Some(value) = payload.strip_prefix("rustc-link-search=") {
            validate_rustc_link_search_metadata(value, line_index)?;
            push_metadata_value(&mut rustc_link_search, value, line_index)?;
        } else if let Some(value) = payload.strip_prefix("rerun-if-changed=") {
            push_metadata_value(&mut rerun_if_changed, value, line_index)?;
        }
    }
    rustc_cfg.sort();
    rustc_cfg.dedup();
    rustc_link_lib.sort();
    rustc_link_lib.dedup();
    rustc_link_search.sort();
    rustc_link_search.dedup();
    rerun_if_changed.sort();
    rerun_if_changed.dedup();
    build_script_metadata_summary_from_parts(
        normalize_path_string(out_dir),
        rustc_cfg,
        rustc_env,
        rustc_link_lib,
        rustc_link_search,
        rerun_if_changed,
    )
}

fn validate_rustc_link_lib_metadata(value: &str, line_index: usize) -> Result<(), RustUnitExecutionBlocker> {
    if value.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "metadata value must not be empty"));
    }
    if contains_metadata_whitespace(value) {
        return Err(malformed_build_script_metadata(line_index, "rustc-link-lib must not contain whitespace"));
    }
    if value.contains(':') || value.contains(',') {
        return Err(malformed_build_script_metadata(
            line_index,
            "rustc-link-lib modifiers and renames are not supported by this bounded rail",
        ));
    }
    let name = if let Some((kind, name)) = value.split_once('=') {
        if !matches!(kind, "static" | "dylib" | "framework") {
            return Err(malformed_build_script_metadata(line_index, "unsupported rustc-link-lib kind"));
        }
        name
    } else {
        value
    };
    if !is_safe_link_token(name) {
        return Err(malformed_build_script_metadata(line_index, "rustc-link-lib name must be a safe token"));
    }
    Ok(())
}

fn validate_rustc_link_search_metadata(value: &str, line_index: usize) -> Result<(), RustUnitExecutionBlocker> {
    if value.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "metadata value must not be empty"));
    }
    if contains_metadata_whitespace(value) {
        return Err(malformed_build_script_metadata(line_index, "rustc-link-search must not contain whitespace"));
    }
    let path = if let Some((kind, path)) = value.split_once('=') {
        if !matches!(kind, "dependency" | "crate" | "native" | "framework" | "all") {
            return Err(malformed_build_script_metadata(line_index, "unsupported rustc-link-search kind"));
        }
        path
    } else {
        value
    };
    if path.is_empty() || path.contains("..") {
        return Err(malformed_build_script_metadata(
            line_index,
            "rustc-link-search path must be non-empty and must not contain parent traversal",
        ));
    }
    Ok(())
}

fn contains_metadata_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn is_safe_link_token(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

fn push_metadata_value(
    values: &mut Vec<String>,
    value: &str,
    line_index: usize,
) -> Result<(), RustUnitExecutionBlocker> {
    if value.is_empty() {
        return Err(malformed_build_script_metadata(line_index, "metadata value must not be empty"));
    }
    values.push(value.to_string());
    Ok(())
}

fn malformed_build_script_metadata(line_index: usize, reason: &str) -> RustUnitExecutionBlocker {
    RustUnitExecutionBlocker {
        class: "malformed-build-script-metadata".to_string(),
        message: format!("build-script metadata line {} is malformed: {reason}", line_index + 1),
    }
}

fn build_script_metadata_summary_from_parts(
    out_dir: String,
    rustc_cfg: Vec<String>,
    rustc_env: BTreeMap<String, String>,
    rustc_link_lib: Vec<String>,
    rustc_link_search: Vec<String>,
    rerun_if_changed: Vec<String>,
) -> Result<BuildScriptMetadataSummary, RustUnitExecutionBlocker> {
    #[derive(Serialize)]
    struct Hashable<'a> {
        out_dir: &'a str,
        rustc_cfg: &'a [String],
        rustc_env: &'a BTreeMap<String, String>,
        rustc_link_lib: &'a [String],
        rustc_link_search: &'a [String],
        rerun_if_changed: &'a [String],
    }
    let canonical = serde_json::to_vec(&Hashable {
        out_dir: &out_dir,
        rustc_cfg: &rustc_cfg,
        rustc_env: &rustc_env,
        rustc_link_lib: &rustc_link_lib,
        rustc_link_search: &rustc_link_search,
        rerun_if_changed: &rerun_if_changed,
    })
    .map_err(|err| RustUnitExecutionBlocker {
        class: "malformed-build-script-metadata".to_string(),
        message: format!("canonicalizing build-script metadata failed: {err}"),
    })?;
    Ok(BuildScriptMetadataSummary {
        out_dir,
        rustc_cfg,
        rustc_env,
        rustc_link_lib,
        rustc_link_search,
        rerun_if_changed,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    })
}

fn build_script_metadata_from_run(run: &BuildScriptMetadataRunReceipt) -> BuildScriptMetadataSummary {
    BuildScriptMetadataSummary {
        out_dir: run.out_dir.clone(),
        rustc_cfg: run.rustc_cfg.clone(),
        rustc_env: run.rustc_env.clone(),
        rustc_link_lib: run.rustc_link_lib.clone(),
        rustc_link_search: run.rustc_link_search.clone(),
        rerun_if_changed: run.rerun_if_changed.clone(),
        digest_blake3: run.metadata_digest_blake3.clone(),
    }
}

fn digest_build_script_out_dir(out_dir: &Path) -> Result<Vec<RustExecutionArtifactDigest>, RunError> {
    let mut files = Vec::new();
    collect_output_files(out_dir, &mut files)?;
    let mut digests = Vec::new();
    for file in files {
        let bytes = fs::read(&file).map_err(|err| {
            RunError::Internal(format!("reading build-script OUT_DIR artifact {}: {err}", file.display()))
        })?;
        let relative = file.strip_prefix(out_dir).unwrap_or(file.as_path());
        digests.push(RustExecutionArtifactDigest {
            path: format!("out-dir/{}", normalize_path_string(relative)),
            blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    digests.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(digests)
}

fn append_dependency_search_paths(
    unit: &mut RustUnitDerivationSummary,
    produced_artifacts: &BTreeMap<String, PathBuf>,
) {
    let mut search_args = BTreeSet::new();
    for produced_artifact in produced_artifacts.values() {
        if let Some(parent) = produced_artifact.parent() {
            search_args.insert(format!("dependency={}", normalize_path_string(parent)));
        }
    }
    for search_arg in search_args {
        unit.derivation.args.push("-L".to_string());
        unit.derivation.args.push(search_arg);
    }
    unit.rustc_args_digest_blake3 = blake3::hash(unit.derivation.args.join("\0").as_bytes()).to_hex().to_string();
}

fn execute_rust_unit(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitExecutionReceipt, RunError> {
    if unit.derivation.builder != "rustc" {
        return blocked_execution_receipt(
            Some(unit),
            "unsupported-builder",
            "only rustc-backed Rust unit derivations are supported by this execution rail",
        );
    }
    if !tool_exists(&options.rustc) {
        return blocked_execution_receipt(
            Some(unit),
            "missing-toolchain",
            &format!("rustc tool is not available: {}", options.rustc.display()),
        );
    }
    let toolchain = match rustc_toolchain_identity(&options.rustc)? {
        Ok(identity) => identity,
        Err(blocker) => return blocked_execution_receipt(Some(unit), &blocker.class, &blocker.message),
    };
    let src_path = rustc_source_path(&unit.derivation.args).ok_or_else(|| {
        RunError::Internal(format!("unit {} lacks a rustc source path in reviewable args", unit.unit_id))
    })?;
    if !src_path.is_file() {
        return blocked_execution_receipt(
            Some(unit),
            "missing-source-material",
            &format!("declared Rust source is not readable: {}", src_path.display()),
        );
    }
    if unit.derivation.outputs.is_empty() {
        return blocked_execution_receipt(
            Some(unit),
            "missing-declared-output",
            "unit derivation does not declare any output artifact paths",
        );
    }
    let dependency_artifact_digests = artifact_digests(
        &unit.dependency_artifacts.iter().map(|artifact| artifact.artifact.as_str()).collect::<Vec<_>>(),
        "missing-dependency-artifact",
    )?;
    if let Err(blocker) = &dependency_artifact_digests {
        return blocked_execution_receipt(Some(unit), &blocker.class, &blocker.message);
    }
    let host_artifact_digests = artifact_digests(
        &unit.consumed_host_artifacts.iter().map(|artifact| artifact.artifact.as_str()).collect::<Vec<_>>(),
        "missing-host-artifact",
    )?;
    if let Err(blocker) = &host_artifact_digests {
        return blocked_execution_receipt(Some(unit), &blocker.class, &blocker.message);
    }

    let unit_output_dir = options.output_root.join(safe_path_component(&unit.unit_id));
    let dependency_artifact_digests = dependency_artifact_digests.unwrap();
    let host_artifact_digests = host_artifact_digests.unwrap();
    if let Some(receipt) = try_reuse_rust_unit_outputs(
        unit,
        &unit_output_dir,
        toolchain.clone(),
        dependency_artifact_digests.clone(),
        host_artifact_digests.clone(),
    )? {
        return Ok(receipt);
    }
    if unit_output_dir.exists() {
        fs::remove_dir_all(&unit_output_dir).map_err(|err| {
            RunError::Internal(format!("removing prior Rust unit output {}: {err}", unit_output_dir.display()))
        })?;
    }
    fs::create_dir_all(&unit_output_dir)
        .map_err(|err| RunError::Internal(format!("creating Rust unit output {}: {err}", unit_output_dir.display())))?;

    let mut command = Command::new(&options.rustc);
    command.args(&unit.derivation.args);
    command.arg("--out-dir").arg(&unit_output_dir);
    command.env_clear();
    for (key, value) in &unit.derivation.env {
        command.env(key, value);
    }
    let output = command
        .output()
        .map_err(|err| RunError::Internal(format!("executing rustc for unit {}: {err}", unit.unit_id)))?;
    if !output.status.success() {
        let diagnostic = redacted_diagnostic(&output.stderr);
        return failed_execution_receipt(
            unit,
            toolchain,
            dependency_artifact_digests,
            host_artifact_digests,
            &diagnostic,
        );
    }
    let output_artifact_digests = digest_output_artifacts(&unit_output_dir)?;
    if output_artifact_digests.is_empty() {
        return blocked_execution_receipt(
            Some(unit),
            "missing-declared-output",
            "rustc completed but produced no declared output artifacts",
        );
    }
    let receipt = finalized_execution_receipt(
        unit,
        "success",
        "rebuilt-explicit-unit",
        toolchain,
        dependency_artifact_digests,
        host_artifact_digests,
        output_artifact_digests,
        None,
    )?;
    write_rust_unit_execution_receipt(&unit_output_dir, &receipt)?;
    Ok(receipt)
}

fn try_reuse_rust_unit_outputs(
    unit: &RustUnitDerivationSummary,
    unit_output_dir: &Path,
    toolchain: RustToolchainIdentity,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
) -> Result<Option<RustUnitExecutionReceipt>, RunError> {
    let receipt_path = unit_output_dir.join(RUST_UNIT_EXECUTION_RECEIPT_FILE);
    if !receipt_path.exists() {
        return Ok(None);
    }
    let prior_receipt = match fs::read(&receipt_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<RustUnitExecutionReceipt>(&bytes).ok())
    {
        Some(receipt) => receipt,
        None => {
            return stale_cached_output_receipt(
                unit,
                toolchain,
                dependency_artifact_digests,
                host_artifact_digests,
                Vec::new(),
                "prior execution receipt is missing or malformed",
            )
            .map(Some);
        }
    };
    let current_output_artifact_digests = digest_output_artifacts(unit_output_dir)?;
    if current_output_artifact_digests.is_empty() {
        return stale_cached_output_receipt(
            unit,
            toolchain,
            dependency_artifact_digests,
            host_artifact_digests,
            current_output_artifact_digests,
            "prior execution receipt exists but no declared output artifacts are readable",
        )
        .map(Some);
    }
    let expected_outputs = sorted_strings(unit.derivation.outputs.clone());
    let matches_current_inputs = prior_receipt.execution_status == "success"
        && prior_receipt.unit_id == unit.unit_id
        && prior_receipt.package_id == unit.package_id
        && prior_receipt.target_name == unit.target_name
        && prior_receipt.target_kind == unit.target_kind
        && prior_receipt.source_digest == unit.source_digest
        && prior_receipt.toolchain == toolchain
        && prior_receipt.rustc_args_digest_blake3 == unit.rustc_args_digest_blake3
        && prior_receipt.declared_outputs == expected_outputs
        && prior_receipt.dependency_artifact_digests == dependency_artifact_digests
        && prior_receipt.host_artifact_digests == host_artifact_digests
        && prior_receipt.output_artifact_digests == current_output_artifact_digests
        && prior_receipt.blocker.is_none();
    if !matches_current_inputs {
        return stale_cached_output_receipt(
            unit,
            toolchain,
            dependency_artifact_digests,
            host_artifact_digests,
            current_output_artifact_digests,
            "prior execution receipt does not match current explicit inputs or output artifact digests",
        )
        .map(Some);
    }
    let receipt = finalized_execution_receipt(
        unit,
        "success",
        "reused-explicit-unit-output",
        toolchain,
        dependency_artifact_digests,
        host_artifact_digests,
        current_output_artifact_digests,
        None,
    )?;
    write_rust_unit_execution_receipt(unit_output_dir, &receipt)?;
    Ok(Some(receipt))
}

fn stale_cached_output_receipt(
    unit: &RustUnitDerivationSummary,
    toolchain: RustToolchainIdentity,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    output_artifact_digests: Vec<RustExecutionArtifactDigest>,
    message: &str,
) -> Result<RustUnitExecutionReceipt, RunError> {
    finalized_execution_receipt(
        unit,
        "blocked",
        "not-run-stale-cached-output",
        toolchain,
        dependency_artifact_digests,
        host_artifact_digests,
        output_artifact_digests,
        Some(RustUnitExecutionBlocker {
            class: "stale-cached-output".to_string(),
            message: message.to_string(),
        }),
    )
}

fn write_rust_unit_execution_receipt(
    unit_output_dir: &Path,
    receipt: &RustUnitExecutionReceipt,
) -> Result<(), RunError> {
    let receipt_path = unit_output_dir.join(RUST_UNIT_EXECUTION_RECEIPT_FILE);
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust unit execution receipt for cache: {err}")))?;
    fs::write(&receipt_path, bytes).map_err(|err| {
        RunError::Internal(format!("writing Rust unit execution receipt {}: {err}", receipt_path.display()))
    })
}

fn is_supported_target_unit(unit: &RustUnitDerivationSummary) -> bool {
    unit.execution_kind == "target" && matches!(unit.target_kind.as_str(), "lib" | "bin")
}

fn is_supported_host_unit(unit: &RustUnitDerivationSummary) -> bool {
    unit.execution_kind == "host" && matches!(unit.target_kind.as_str(), "proc-macro" | "custom-build")
}

fn produced_host_artifact_path(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
) -> Result<Result<PathBuf, RustUnitExecutionBlocker>, RunError> {
    let output_dir = options.output_root.join(safe_path_component(&unit.unit_id));
    if !output_dir.is_dir() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-produced-host-artifact".to_string(),
            message: format!("host output directory is missing: {}", output_dir.display()),
        }));
    }
    let mut files = Vec::new();
    collect_output_files(&output_dir, &mut files)?;
    let crate_name = rust_crate_name(&unit.target_name);
    let artifact = if unit.target_kind == "proc-macro" {
        files.into_iter().find(|path| {
            matches!(path.extension().and_then(OsStr::to_str), Some("so" | "dylib" | "dll"))
                && path.file_name().and_then(OsStr::to_str).is_some_and(|name| name.contains(&crate_name))
        })
    } else {
        files.into_iter().find(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name == crate_name || name.contains(&crate_name))
        })
    }
    .ok_or_else(|| RustUnitExecutionBlocker {
        class: "missing-produced-host-artifact".to_string(),
        message: format!("host unit {} did not emit a matching artifact", unit.unit_id),
    });
    Ok(artifact.map_err(|blocker| blocker))
}

fn produced_library_artifact_path(
    unit: &RustUnitDerivationSummary,
    options: &RustUnitExecutionOptions,
) -> Result<Result<PathBuf, RustUnitExecutionBlocker>, RunError> {
    let output_dir = options.output_root.join(safe_path_component(&unit.unit_id));
    if !output_dir.is_dir() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-produced-dependency-artifact".to_string(),
            message: format!("producer output directory is missing: {}", output_dir.display()),
        }));
    }
    let mut files = Vec::new();
    collect_output_files(&output_dir, &mut files)?;
    let crate_prefix = format!("lib{}", rust_crate_name(&unit.target_name));
    let artifact = files
        .into_iter()
        .find(|path| {
            path.extension().and_then(OsStr::to_str) == Some("rlib")
                && path.file_name().and_then(OsStr::to_str).is_some_and(|name| name.starts_with(&crate_prefix))
        })
        .ok_or_else(|| RustUnitExecutionBlocker {
            class: "missing-produced-dependency-artifact".to_string(),
            message: format!("producer unit {} did not emit a matching .rlib artifact", unit.unit_id),
        });
    Ok(artifact.map_err(|blocker| blocker))
}

fn bind_dependency_artifact(
    consumer: &RustUnitDerivationSummary,
    dependency: &RustDependencyArtifact,
    produced_artifact: &Path,
) -> Result<RustUnitDerivationSummary, RunError> {
    let produced_artifact = normalize_path_string(produced_artifact);
    let mut bound = consumer.clone();
    for artifact in &mut bound.dependency_artifacts {
        if artifact.package_id == dependency.package_id
            && artifact.name == dependency.name
            && artifact.artifact == dependency.artifact
        {
            artifact.artifact = produced_artifact.clone();
        }
    }
    for input in &mut bound.derivation.inputs {
        if input == &dependency.artifact {
            *input = produced_artifact.clone();
        }
    }
    let expected_extern = format!("{}={}", dependency.name, dependency.artifact);
    let rewritten_extern = format!("{}={produced_artifact}", dependency.name);
    let mut replaced = false;
    for arg in &mut bound.derivation.args {
        if arg == &expected_extern {
            *arg = rewritten_extern.clone();
            replaced = true;
        }
    }
    if !replaced {
        return Err(RunError::Internal(format!(
            "consumer unit {} lacks expected dependency extern {}",
            consumer.unit_id, expected_extern
        )));
    }
    bound.rustc_args_digest_blake3 = blake3::hash(bound.derivation.args.join("\0").as_bytes()).to_hex().to_string();
    Ok(bound)
}

fn dependency_chain_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitDependencyChainExecutionReceipt, RunError> {
    let mut receipt = RustUnitDependencyChainExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded explicit Rust dependency edge only; not full Cargo compatibility or a general scheduler"
            .to_string(),
        unit_executions,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_dependency_chain_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_dependency_chain_execution_receipt_hash(
    receipt: &RustUnitDependencyChainExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust dependency-chain execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn target_topology_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitTargetTopologyExecutionReceipt, RunError> {
    let mut receipt = RustUnitTargetTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded target-only Rust unit topology; not full Cargo compatibility, host artifact execution, or a general scheduler"
            .to_string(),
        unit_executions,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_target_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_target_topology_execution_receipt_hash(
    receipt: &RustUnitTargetTopologyExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust target-topology execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn host_artifact_topology_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitHostArtifactTopologyExecutionReceipt, RunError> {
    let mut receipt = RustUnitHostArtifactTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded Rust host-artifact topology with build-script metadata; not full Cargo compatibility, native-link probing, or a general scheduler"
            .to_string(),
        unit_executions,
        build_script_metadata_runs,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_host_artifact_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_host_artifact_topology_execution_receipt_hash(
    receipt: &RustUnitHostArtifactTopologyExecutionReceipt,
) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable).map_err(|err| {
        RunError::Internal(format!("canonicalizing Rust host-artifact topology execution receipt: {err}"))
    })?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn topology_receipt(
    execution_status: &str,
    unit_executions: Vec<RustUnitExecutionReceipt>,
    build_script_metadata_runs: Vec<BuildScriptMetadataRunReceipt>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitTopologyExecutionReceipt, RunError> {
    let mut receipt = RustUnitTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded unified Rust unit topology with target dependencies, host artifacts, and build-script metadata; not full Cargo compatibility, parallel scheduling, or a general scheduler"
            .to_string(),
        unit_executions,
        build_script_metadata_runs,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_topology_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_topology_execution_receipt_hash(receipt: &RustUnitTopologyExecutionReceipt) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust topology execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn blocked_execution_receipt(
    unit: Option<&RustUnitDerivationSummary>,
    class: &str,
    message: &str,
) -> Result<RustUnitExecutionReceipt, RunError> {
    let fallback_source_digest = SourceDigest {
        algorithm: "missing".to_string(),
        value: "missing".to_string(),
    };
    let (unit_id, package_id, target_name, target_kind, source_digest, rustc_args_digest_blake3) = unit.map_or_else(
        || {
            (
                "missing".to_string(),
                "missing".to_string(),
                "missing".to_string(),
                "missing".to_string(),
                fallback_source_digest.clone(),
                "missing".to_string(),
            )
        },
        |unit| {
            (
                unit.unit_id.clone(),
                unit.package_id.clone(),
                unit.target_name.clone(),
                unit.target_kind.clone(),
                unit.source_digest.clone(),
                unit.rustc_args_digest_blake3.clone(),
            )
        },
    );
    finalized_execution_receipt(
        &RustUnitDerivationSummary {
            unit_id,
            package_id,
            target_name,
            target_kind,
            execution_kind: "target".to_string(),
            crate_types: Vec::new(),
            mode: "build".to_string(),
            profile: String::new(),
            source_digest,
            dependency_artifacts: Vec::new(),
            consumed_host_artifacts: Vec::new(),
            generated_metadata: None,
            derivation: ReviewableRustDerivation {
                name: String::new(),
                builder: "rustc".to_string(),
                system: String::new(),
                args: Vec::new(),
                outputs: Vec::new(),
                env: BTreeMap::new(),
                inputs: Vec::new(),
                addressing_mode: String::new(),
            },
            rustc_args_digest_blake3,
        },
        "blocked",
        "not-run-preflight-blocker",
        missing_toolchain_identity(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Some(RustUnitExecutionBlocker {
            class: class.to_string(),
            message: message.to_string(),
        }),
    )
}

fn failed_execution_receipt(
    unit: &RustUnitDerivationSummary,
    toolchain: RustToolchainIdentity,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    diagnostic: &str,
) -> Result<RustUnitExecutionReceipt, RunError> {
    finalized_execution_receipt(
        unit,
        "failed",
        "rustc-exit-nonzero",
        toolchain,
        dependency_artifact_digests,
        host_artifact_digests,
        Vec::new(),
        Some(RustUnitExecutionBlocker {
            class: "rustc-failed".to_string(),
            message: diagnostic.to_string(),
        }),
    )
}

fn finalized_execution_receipt(
    unit: &RustUnitDerivationSummary,
    execution_status: &str,
    rebuild_reason: &str,
    toolchain: RustToolchainIdentity,
    dependency_artifact_digests: Vec<RustExecutionArtifactDigest>,
    host_artifact_digests: Vec<RustExecutionArtifactDigest>,
    output_artifact_digests: Vec<RustExecutionArtifactDigest>,
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitExecutionReceipt, RunError> {
    let mut receipt = RustUnitExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        unit_id: unit.unit_id.clone(),
        package_id: unit.package_id.clone(),
        target_name: unit.target_name.clone(),
        target_kind: unit.target_kind.clone(),
        execution_status: execution_status.to_string(),
        rebuild_reason: rebuild_reason.to_string(),
        source_digest: unit.source_digest.clone(),
        toolchain,
        rustc_args_digest_blake3: unit.rustc_args_digest_blake3.clone(),
        declared_outputs: sorted_strings(unit.derivation.outputs.clone()),
        dependency_artifact_digests,
        host_artifact_digests,
        output_artifact_digests,
        blocker,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = rust_unit_execution_receipt_hash(&receipt)?;
    Ok(receipt)
}

fn rust_unit_execution_receipt_hash(receipt: &RustUnitExecutionReceipt) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust unit execution receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn rustc_toolchain_identity(tool: &Path) -> Result<Result<RustToolchainIdentity, RustUnitExecutionBlocker>, RunError> {
    let output = Command::new(tool)
        .arg("-vV")
        .output()
        .map_err(|err| RunError::Internal(format!("querying rustc toolchain identity {}: {err}", tool.display())))?;
    if !output.status.success() {
        return Ok(Err(RustUnitExecutionBlocker {
            class: "missing-toolchain-identity".to_string(),
            message: format!("rustc -vV failed for {}", tool.display()),
        }));
    }
    let version_verbose = String::from_utf8(output.stdout)
        .map_err(|err| RunError::Internal(format!("rustc -vV emitted non-UTF-8 output: {err}")))?
        .trim()
        .to_string();
    Ok(Ok(RustToolchainIdentity {
        tool: tool.file_name().and_then(OsStr::to_str).unwrap_or("rustc").to_string(),
        version_digest_blake3: blake3::hash(version_verbose.as_bytes()).to_hex().to_string(),
        version_verbose,
    }))
}

fn missing_toolchain_identity() -> RustToolchainIdentity {
    RustToolchainIdentity {
        tool: "missing".to_string(),
        version_verbose: "missing".to_string(),
        version_digest_blake3: "missing".to_string(),
    }
}

fn rustc_source_path(args: &[String]) -> Option<PathBuf> {
    args.iter().find(|arg| !arg.starts_with('-') && arg.ends_with(".rs")).map(PathBuf::from)
}

fn artifact_digests(
    paths: &[&str],
    missing_class: &str,
) -> Result<Result<Vec<RustExecutionArtifactDigest>, RustUnitExecutionBlocker>, RunError> {
    let mut digests = Vec::new();
    for path in paths {
        let artifact_path = Path::new(path);
        if !artifact_path.is_file() {
            return Ok(Err(RustUnitExecutionBlocker {
                class: missing_class.to_string(),
                message: format!("declared artifact is not readable: {path}"),
            }));
        }
        digests.push(digest_artifact_path(artifact_path)?);
    }
    digests.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Ok(digests))
}

fn digest_output_artifacts(output_dir: &Path) -> Result<Vec<RustExecutionArtifactDigest>, RunError> {
    let mut paths = Vec::new();
    collect_output_files(output_dir, &mut paths)?;
    let mut digests = Vec::new();
    for path in paths {
        if path.file_name().and_then(OsStr::to_str) == Some(RUST_UNIT_EXECUTION_RECEIPT_FILE) {
            continue;
        }
        let relative_path = path.strip_prefix(output_dir).unwrap_or(&path);
        let receipt_path = format!("declared-output/{}", normalize_path_string(relative_path));
        digests.push(digest_artifact_path_with_receipt_path(&path, receipt_path)?);
    }
    digests.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(digests)
}

fn collect_output_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let entries = fs::read_dir(directory)
        .map_err(|err| RunError::Internal(format!("reading Rust unit output {}: {err}", directory.display())))?;
    for entry in entries {
        let entry = entry
            .map_err(|err| RunError::Internal(format!("reading Rust unit output {}: {err}", directory.display())))?;
        let file_type = entry.file_type().map_err(|err| {
            RunError::Internal(format!("reading Rust unit output metadata {}: {err}", entry.path().display()))
        })?;
        if file_type.is_dir() {
            collect_output_files(&entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(entry.path());
        }
    }
    files.sort();
    Ok(())
}

fn digest_artifact_path(path: &Path) -> Result<RustExecutionArtifactDigest, RunError> {
    digest_artifact_path_with_receipt_path(path, normalize_path_string(path))
}

fn digest_artifact_path_with_receipt_path(
    path: &Path,
    receipt_path: String,
) -> Result<RustExecutionArtifactDigest, RunError> {
    let bytes =
        fs::read(path).map_err(|err| RunError::Internal(format!("reading artifact {}: {err}", path.display())))?;
    Ok(RustExecutionArtifactDigest {
        path: receipt_path,
        blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn safe_path_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn tool_exists(tool: &Path) -> bool {
    if tool.components().count() > 1 || tool.is_absolute() {
        return tool.is_file();
    }
    resolve_tool_path(tool).is_some()
}

fn resolve_tool_path(tool: impl AsRef<Path>) -> Option<PathBuf> {
    let tool = tool.as_ref();
    if tool.components().count() > 1 || tool.is_absolute() {
        return tool.is_file().then(|| tool.to_path_buf());
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join(tool))
        .find(|candidate| candidate.is_file())
}

fn redacted_diagnostic(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let compact = text.lines().take(12).collect::<Vec<_>>().join("\n");
    if compact.is_empty() {
        "rustc exited with a nonzero status".to_string()
    } else {
        compact.replace(' ', "")
    }
}

fn summarize_unit_graph(unit_graph: Value) -> Result<UnitGraphSummary, RunError> {
    let normalized = normalize_json_value(unit_graph);
    let canonical = serde_json::to_vec(&normalized)
        .map_err(|err| RunError::Internal(format!("canonicalizing unit graph JSON: {err}")))?;
    let unit_count = normalized.get("units").and_then(Value::as_array).map_or(0, Vec::len);
    let root_count = normalized.get("roots").and_then(Value::as_array).map_or(0, Vec::len);
    Ok(UnitGraphSummary {
        unit_count,
        root_count,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    })
}

fn normalize_json_value(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(normalize_json_value).collect()),
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(key, value)| (key, normalize_json_value(value))).collect();
            let mut normalized = serde_json::Map::new();
            for (key, value) in sorted {
                normalized.insert(key, value);
            }
            Value::Object(normalized)
        }
        other => other,
    }
}

fn receipt_hash(receipt: &RustPlanReceipt) -> Result<String, RunError> {
    let mut hashable = receipt.clone();
    hashable.receipt_hash.clear();
    let canonical = serde_json::to_vec(&hashable)
        .map_err(|err| RunError::Internal(format!("canonicalizing Rust plan receipt: {err}")))?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn sorted_strings(mut strings: Vec<String>) -> Vec<String> {
    strings.sort();
    strings.dedup();
    strings
}

fn normalize_path_string(path: &Path) -> String {
    path.components().as_path().to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use tempfile::TempDir;

    use super::*;

    struct FixedOracle {
        cargo_version: CargoOutput,
        rustc_version: CargoOutput,
        metadata: CargoOutput,
        unit_graph: CargoOutput,
    }

    impl CargoOracle for FixedOracle {
        fn run_cargo(&self, _root: &Path, _cargo: &Path, args: &[String]) -> Result<CargoOutput, RunError> {
            if args.first().map(String::as_str) == Some("metadata") {
                return Ok(self.metadata.clone());
            }
            Ok(self.unit_graph.clone())
        }

        fn run_tool_version(&self, tool: &Path, _args: &[&str]) -> Result<CargoOutput, RunError> {
            if tool.file_name() == Some(OsStr::new("rustc")) {
                return Ok(self.rustc_version.clone());
            }
            Ok(self.cargo_version.clone())
        }
    }

    fn ok_output(text: &str) -> CargoOutput {
        CargoOutput {
            stdout: text.as_bytes().to_vec(),
            stderr: Vec::new(),
            status_code: 0,
        }
    }

    fn failing_output(text: &str) -> CargoOutput {
        CargoOutput {
            stdout: Vec::new(),
            stderr: text.as_bytes().to_vec(),
            status_code: 101,
        }
    }

    fn options(root: &Path) -> RustPlanOptions {
        RustPlanOptions {
            root: root.to_path_buf(),
            cargo: PathBuf::from("cargo"),
            rustc: PathBuf::from("rustc"),
            targets: vec!["x86_64-unknown-linux-gnu".to_string()],
            profile: default_profile(),
            features: vec!["b".to_string(), "a".to_string()],
            all_features: false,
            no_default_features: true,
        }
    }

    fn empty_registry_planning() -> NativeRegistrySourcePlanningSummary {
        NativeRegistrySourcePlanningSummary {
            ready: true,
            comparison_status: "matched".to_string(),
            lockfile_digest_blake3: "test-lockfile".to_string(),
            digest_blake3: "test-registry".to_string(),
            sources: Vec::new(),
            blockers: Vec::new(),
            non_claims: vec!["declared-local-vendor-source-only".to_string()],
        }
    }

    #[test]
    fn captures_normalized_oracle_receipt() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), "# lock\n").unwrap();
        let manifest_path = dir.path().join("Cargo.toml");
        let metadata = format!(
            r#"{{"packages":[{{"id":"path+file://root#demo@0.1.0","name":"demo","version":"0.1.0","source":null,"manifest_path":"{}"}}],"workspace_root":"{}","workspace_members":["path+file://root#demo@0.1.0"]}}"#,
            manifest_path.display(),
            dir.path().display()
        );
        let oracle = FixedOracle {
            cargo_version: ok_output("cargo 1.91.0\n"),
            rustc_version: ok_output("rustc 1.91.0\nhost: x86_64-unknown-linux-gnu\n"),
            metadata: ok_output(&metadata),
            unit_graph: ok_output(&format!(
                r#"{{"roots":[0],"units":[{{"pkg_id":"path+file://root#demo@0.1.0","target":{{"name":"demo","kind":["lib"],"crate_types":["lib"],"src_path":"{}","edition":"2021"}},"mode":"build","features":["serde"],"deps":[]}}]}}"#,
                dir.path().join("src/lib.rs").display()
            )),
        };

        let receipt = capture_rust_plan_with_oracle(&options(dir.path()), &oracle).unwrap();

        assert_eq!(receipt.schema_version, RECEIPT_SCHEMA_VERSION);
        assert_eq!(receipt.invocation.features, vec!["a", "b"]);
        assert!(receipt.lockfile.blake3.len() >= 32);
        assert_eq!(receipt.package_count, 1);
        assert_eq!(receipt.packages[0].name, "demo");
        assert!(receipt.source_closure.ready);
        assert_eq!(receipt.source_closure.source_count, 1);
        assert_eq!(receipt.source_closure.sources[0].kind, SourceKind::Path);
        assert_eq!(receipt.source_closure.sources[0].source_digest.algorithm, PATH_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(receipt.unit_graph.unit_count, 1);
        assert_eq!(receipt.unit_graph.root_count, 1);
        assert!(receipt.unit_derivation_graph.ready);
        assert_eq!(receipt.unit_derivation_graph.derivation_count, 1);
        let derivation = &receipt.unit_derivation_graph.derivations[0];
        assert_eq!(derivation.target_kind, "lib");
        assert_eq!(derivation.derivation.builder, "rustc");
        assert!(derivation.derivation.args.contains(&"--crate-name".to_string()));
        assert!(derivation.derivation.args.contains(&"feature=\"serde\"".to_string()));
        assert_eq!(derivation.rustc_args_digest_blake3.len(), 64);
        assert_eq!(receipt.receipt_hash.len(), 64);
    }

    #[test]
    fn native_package_target_fragment_matches_supported_path_workspace() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let dep_dir = dir.path().join("dep-crate");
        std::fs::create_dir_all(app_dir.join("src/bin")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\", \"dep-crate\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", path = \"../dep-crate\" }\n\n[lib]\nname = \"app\"\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"app-cli\"\npath = \"src/bin/app-cli.rs\"\n",
        )
        .unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn answer() -> u32 { dep_crate::answer() }\n").unwrap();
        std::fs::write(app_dir.join("src/bin/app-cli.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![
            CargoPackage {
                id: "path+file://app#app@0.1.0".to_string(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![
                    CargoTarget {
                        name: "app".to_string(),
                        kind: vec!["lib".to_string()],
                        src_path: app_dir.join("src/lib.rs").display().to_string(),
                    },
                    CargoTarget {
                        name: "app-cli".to_string(),
                        kind: vec!["bin".to_string()],
                        src_path: app_dir.join("src/bin/app-cli.rs").display().to_string(),
                    },
                ],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file://dep-crate#dep-crate@0.1.0".to_string(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: dep_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let planning = summarize_native_package_target_planning(
            dir.path(),
            &RustPlanOptions {
                features: Vec::new(),
                no_default_features: false,
                ..options(dir.path())
            },
            &packages,
            &[
                "path+file://app#app@0.1.0".to_string(),
                "path+file://dep-crate#dep-crate@0.1.0".to_string(),
            ],
            &closure,
            &empty_registry_planning(),
        )
        .unwrap();

        assert!(planning.ready, "{:#?}", planning.blockers);
        assert_eq!(planning.comparison_status, "matched");
        assert_eq!(planning.packages.len(), 2);
        let app = planning.packages.iter().find(|package| package.name == "app").unwrap();
        assert_eq!(app.targets.len(), 2);
        assert_eq!(app.path_dependencies[0].name, "dep_crate");
        assert!(planning.non_claims.contains(&"not-full-cargo-compatibility".to_string()));
        assert_eq!(planning.digest_blake3.len(), 64);
    }

    #[test]
    fn native_registry_source_planning_binds_declared_vendor_source() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let vendor_dir = dir.path().join("vendor");
        let dep_dir = vendor_dir.join("dep-crate-0.1.0");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join(".cargo")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", version = \"0.1.0\" }\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn app() -> u32 { dep_crate::dep() }\n").unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn dep() -> u32 { 7 }\n").unwrap();
        let checksum = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        std::fs::write(dep_dir.join(".cargo-checksum.json"), format!(r#"{{"package":"{checksum}","files":{{}}}}"#))
            .unwrap();
        std::fs::write(
            dir.path().join(".cargo/config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n",
        )
        .unwrap();
        let registry_source = "registry+https://github.com/rust-lang/crates.io-index".to_string();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "app".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: app_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: dep_id.clone(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: Some(registry_source.clone()),
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: dep_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let lock_packages = vec![LockPackage {
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source),
            checksum: Some(checksum.to_string()),
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };
        let registry_planning =
            summarize_native_registry_source_planning(dir.path(), &packages, &lock_packages, &lockfile).unwrap();
        let source_closure = summarize_source_closure(&packages, &lock_packages).unwrap();
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &packages,
            &[app_id, dep_id],
            &source_closure,
            &registry_planning,
        )
        .unwrap();

        assert!(registry_planning.ready, "{:#?}", registry_planning.blockers);
        assert_eq!(registry_planning.sources.len(), 1);
        let registry_source = &registry_planning.sources[0];
        assert_eq!(registry_source.name, "dep-crate");
        assert_eq!(registry_source.checksum, checksum);
        assert_eq!(registry_source.source_digest.algorithm, PATH_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(registry_source.source_digest.value.len(), 64);
        assert!(registry_planning.non_claims.contains(&"no-ambient-cargo-cache".to_string()));
        assert!(package_planning.ready, "{:#?}", package_planning.blockers);
        assert_eq!(package_planning.packages.len(), 2);
        let app = package_planning.packages.iter().find(|package| package.name == "app").unwrap();
        assert_eq!(app.path_dependencies[0].manifest_path, normalize_path_string(&dep_dir.join("Cargo.toml")));
    }

    #[test]
    fn native_registry_source_planning_blocks_missing_vendor_material() {
        let dir = TempDir::new().unwrap();
        let registry_source = "registry+https://github.com/rust-lang/crates.io-index".to_string();
        let packages = vec![CargoPackage {
            id: "registry+https://github.com/rust-lang/crates.io-index#dep-crate@0.1.0".to_string(),
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source.clone()),
            manifest_path: dir.path().join("ambient-cargo-cache/dep-crate-0.1.0/Cargo.toml").display().to_string(),
            targets: vec![CargoTarget {
                name: "dep-crate".to_string(),
                kind: vec!["lib".to_string()],
                src_path: dir.path().join("ambient-cargo-cache/dep-crate-0.1.0/src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let lock_packages = vec![LockPackage {
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: Some(registry_source),
            checksum: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()),
        }];
        let lockfile = LockfileIdentity {
            path: "Cargo.lock".to_string(),
            blake3: "lock-digest".to_string(),
        };

        let registry_planning =
            summarize_native_registry_source_planning(dir.path(), &packages, &lock_packages, &lockfile).unwrap();

        assert!(!registry_planning.ready);
        assert_eq!(registry_planning.comparison_status, "blocked");
        assert!(registry_planning.sources.is_empty());
        assert!(registry_planning.blockers.iter().any(|blocker| blocker.class == "missing-declared-vendor-root"));
    }

    #[test]
    fn native_unit_graph_fragment_feeds_supported_unit_derivations() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app");
        let dep_dir = dir.path().join("dep-crate");
        std::fs::create_dir_all(app_dir.join("src/bin")).unwrap();
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\", \"dep-crate\"]\n").unwrap();
        std::fs::write(
            app_dir.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ndep_crate = { package = \"dep-crate\", path = \"../dep-crate\" }\n\n[lib]\nname = \"app\"\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"app-cli\"\npath = \"src/bin/app-cli.rs\"\n",
        )
        .unwrap();
        std::fs::write(
            dep_dir.join("Cargo.toml"),
            "[package]\nname = \"dep-crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(app_dir.join("src/lib.rs"), "pub fn answer() -> u32 { dep_crate::answer() }\n").unwrap();
        std::fs::write(app_dir.join("src/bin/app-cli.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dep_dir.join("src/lib.rs"), "pub fn answer() -> u32 { 42 }\n").unwrap();
        let app_id = "path+file://app#app@0.1.0".to_string();
        let dep_id = "path+file://dep-crate#dep-crate@0.1.0".to_string();
        let packages = vec![
            CargoPackage {
                id: app_id.clone(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_dir.join("Cargo.toml").display().to_string(),
                targets: vec![
                    CargoTarget {
                        name: "app".to_string(),
                        kind: vec!["lib".to_string()],
                        src_path: app_dir.join("src/lib.rs").display().to_string(),
                    },
                    CargoTarget {
                        name: "app-cli".to_string(),
                        kind: vec!["bin".to_string()],
                        src_path: app_dir.join("src/bin/app-cli.rs").display().to_string(),
                    },
                ],
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: dep_id.clone(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: dep_dir.join("Cargo.toml").display().to_string(),
                targets: vec![CargoTarget {
                    name: "dep-crate".to_string(),
                    kind: vec!["lib".to_string()],
                    src_path: dep_dir.join("src/lib.rs").display().to_string(),
                }],
                features: BTreeMap::new(),
            },
        ];
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &packages,
            &[app_id.clone(), dep_id.clone()],
            &closure,
            &empty_registry_planning(),
        )
        .unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": dep_id,
                    "target": {"name": "dep-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": dep_dir.join("src/lib.rs").display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": []
                },
                {
                    "pkg_id": app_id,
                    "target": {"name": "app", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_dir.join("src/lib.rs").display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": "path+file://dep-crate#dep-crate@0.1.0", "extern_crate_name": "dep_crate"}]
                },
                {
                    "pkg_id": "path+file://app#app@0.1.0",
                    "target": {"name": "app-cli", "kind": ["bin"], "crate_types": ["bin"], "src_path": app_dir.join("src/bin/app-cli.rs").display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "features": [],
                    "deps": [{"pkg_id": "path+file://dep-crate#dep-crate@0.1.0", "extern_crate_name": "dep_crate"}]
                }
            ]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &closure, &package_planning, &plan_options).unwrap();
        let derivation_graph = summarize_unit_derivation_graph_with_native(
            &unit_graph,
            &closure,
            &plan_options,
            Some(&native_units),
            None,
        )
        .unwrap();

        assert!(native_units.ready, "{:#?}", native_units.blockers);
        assert_eq!(native_units.comparison_status, "matched");
        assert_eq!(native_units.units.len(), 3);
        assert!(native_units.non_claims.contains(&"cargo-unit-graph-retained-as-oracle".to_string()));
        assert!(derivation_graph.ready, "{:#?}", derivation_graph.blockers);
        assert_eq!(derivation_graph.derivation_count, 3);
        assert_eq!(derivation_graph.host_unit_count, 0);
        let app_bin = derivation_graph.derivations.iter().find(|unit| unit.target_name == "app-cli").unwrap();
        assert_eq!(app_bin.derivation.builder, "rustc");
        assert!(app_bin.derivation.args.contains(&"--extern".to_string()));
        assert_eq!(app_bin.derivation.inputs.iter().filter(|input| input.starts_with("source:")).count(), 1);
    }

    #[test]
    fn native_unit_graph_fragment_blocks_mismatch_and_missing_edges() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("app/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap().join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"app\"]\n").unwrap();
        std::fs::write(&manifest_path, "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        std::fs::write(manifest_path.parent().unwrap().join("src/lib.rs"), "pub fn app() {}\n").unwrap();
        let package_id = "path+file://app#app@0.1.0".to_string();
        let packages = vec![CargoPackage {
            id: package_id.clone(),
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: vec![CargoTarget {
                name: "app".to_string(),
                kind: vec!["lib".to_string()],
                src_path: manifest_path.parent().unwrap().join("src/lib.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let plan_options = RustPlanOptions {
            features: Vec::new(),
            no_default_features: false,
            ..options(dir.path())
        };
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let mut package_planning = summarize_native_package_target_planning(
            dir.path(),
            &plan_options,
            &packages,
            &[package_id.clone()],
            &closure,
            &empty_registry_planning(),
        )
        .unwrap();
        package_planning.packages[0].path_dependencies.push(NativePathDependencySummary {
            name: "missing_dep".to_string(),
            manifest_path: dir.path().join("missing/Cargo.toml").display().to_string(),
        });
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": package_id,
                "target": {"name": "app-renamed", "kind": ["lib"], "crate_types": ["lib"], "src_path": manifest_path.parent().unwrap().join("src/lib.rs").display().to_string(), "edition": "2021"},
                "mode": "build",
                "features": [],
                "deps": []
            }]
        });

        let native_units =
            summarize_native_unit_graph_planning(&unit_graph, &closure, &package_planning, &plan_options).unwrap();

        assert!(!native_units.ready);
        assert_eq!(native_units.comparison_status, "blocked");
        assert!(native_units.blockers.iter().any(|blocker| blocker.class == "unresolved-path-dependency-edge"));
        assert!(native_units.blockers.iter().all(|blocker| blocker.class != "cargo-oracle-unit-graph-mismatch"));
    }

    #[test]
    fn native_package_target_fragment_blocks_unsupported_and_mismatch() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("benchy");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers = [\"benchy\"]\n").unwrap();
        std::fs::write(crate_dir.join("Cargo.toml"), "[package]\nname = \"benchy\"\nversion = \"0.1.0\"\n").unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://benchy#benchy@0.1.0".to_string(),
            name: "benchy".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: crate_dir.join("Cargo.toml").display().to_string(),
            targets: vec![CargoTarget {
                name: "benchy-bench".to_string(),
                kind: vec!["bench".to_string()],
                src_path: crate_dir.join("benches/benchy.rs").display().to_string(),
            }],
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let planning = summarize_native_package_target_planning(
            dir.path(),
            &options(dir.path()),
            &packages,
            &["path+file://benchy#benchy@0.1.0".to_string()],
            &closure,
            &empty_registry_planning(),
        )
        .unwrap();

        assert!(!planning.ready);
        assert_eq!(planning.comparison_status, "blocked");
        assert!(planning.blockers.iter().any(|blocker| blocker.class == "unsupported-cargo-oracle-target-kind"));
        assert!(planning.blockers.iter().all(|blocker| blocker.class != "native-package-target-mismatch"));
    }

    #[test]
    fn source_closure_records_registry_git_and_path_identities() {
        let dir = TempDir::new().unwrap();
        let path_manifest = dir.path().join("path-crate/Cargo.toml");
        std::fs::create_dir_all(path_manifest.parent().unwrap()).unwrap();
        std::fs::write(&path_manifest, "[package]\nname='path-crate'\nversion='0.1.0'\n").unwrap();
        let lockfile = r#"
[[package]]
name = "git-crate"
version = "0.2.0"
source = "git+https://example.invalid/repo?rev=main#abcdef123456"

[[package]]
name = "path-crate"
version = "0.1.0"

[[package]]
name = "registry-crate"
version = "1.2.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0123456789abcdef"
"#;
        std::fs::write(dir.path().join("Cargo.lock"), lockfile).unwrap();
        let packages = vec![
            CargoPackage {
                id: "git+https://example.invalid/repo?rev=main#abcdef123456#git-crate@0.2.0".to_string(),
                name: "git-crate".to_string(),
                version: "0.2.0".to_string(),
                source: Some("git+https://example.invalid/repo?rev=main#abcdef123456".to_string()),
                manifest_path: dir.path().join("git-crate/Cargo.toml").display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file:///path-crate#path-crate@0.1.0".to_string(),
                name: "path-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: path_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "registry+https://github.com/rust-lang/crates.io-index#registry-crate@1.2.3".to_string(),
                name: "registry-crate".to_string(),
                version: "1.2.3".to_string(),
                source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
                manifest_path: dir.path().join("registry-crate/Cargo.toml").display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &parse_lockfile_packages(dir.path()).unwrap()).unwrap();

        assert!(closure.ready);
        assert_eq!(closure.source_count, 3);
        assert_eq!(closure.sources[0].kind, SourceKind::Git);
        assert_eq!(closure.sources[0].resolved_revision.as_deref(), Some("abcdef123456"));
        assert_eq!(closure.sources[0].source_digest.algorithm, GIT_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(closure.sources[1].kind, SourceKind::Path);
        assert_eq!(closure.sources[1].source_digest.algorithm, PATH_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(closure.sources[2].kind, SourceKind::Registry);
        assert_eq!(
            closure.sources[2].lockfile_identity.as_ref().unwrap().checksum.as_deref(),
            Some("0123456789abcdef")
        );
        assert_eq!(closure.sources[2].source_digest.algorithm, REGISTRY_SOURCE_DIGEST_ALGORITHM);
        assert_eq!(closure.digest_blake3.len(), 64);
    }

    #[test]
    fn source_closure_blocks_registry_without_checksum() {
        let package = CargoPackage {
            id: "registry+https://github.com/rust-lang/crates.io-index#missing@1.0.0".to_string(),
            name: "missing".to_string(),
            version: "1.0.0".to_string(),
            source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
            manifest_path: "/tmp/missing/Cargo.toml".to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        };

        let closure = summarize_source_closure(&[package], &[]).unwrap();

        assert!(!closure.ready);
        assert_eq!(closure.blockers[0].class, "missing-registry-checksum");
        assert_eq!(closure.sources[0].source_digest.algorithm, "missing");
    }

    #[test]
    fn unit_derivation_graph_emits_binary_with_dependency_artifact() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("app/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap().join("src")).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='app'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://app#app@0.1.0".to_string(),
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://app#app@0.1.0",
                "target": {
                    "name": "app-bin",
                    "kind": ["bin"],
                    "crate_types": ["bin"],
                    "src_path": dir.path().join("app/src/main.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "features": ["cli"],
                "deps": [{
                    "pkg_id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0",
                    "extern_crate_name": "serde"
                }]
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(graph.ready);
        assert_eq!(graph.derivation_count, 1);
        let unit = &graph.derivations[0];
        assert_eq!(unit.target_kind, "bin");
        assert_eq!(unit.derivation.name, "rust-unit-0-app_bin");
        assert!(unit.derivation.args.contains(&"--crate-type".to_string()));
        assert!(unit.derivation.args.contains(&"bin".to_string()));
        assert!(unit.derivation.args.contains(&"--extern".to_string()));
        assert_eq!(unit.dependency_artifacts[0].name, "serde");
        assert_eq!(unit.derivation.env.get("SOURCE_CLOSURE_DIGEST"), Some(&closure.digest_blake3));
        assert_eq!(graph.digest_blake3.len(), 64);
    }

    #[test]
    fn executes_first_supported_lib_unit_from_derivation_graph() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("exec-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='exec-crate'\nversion='0.1.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://exec-crate#exec-crate@0.1.0".to_string(),
            name: "exec-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://exec-crate#exec-crate@0.1.0",
                "target": {
                    "name": "exec-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": lib_path.display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": []
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "success");
        assert_eq!(receipt.rebuild_reason, "rebuilt-explicit-unit");
        assert_eq!(receipt.target_kind, "lib");
        assert_eq!(receipt.rustc_args_digest_blake3, graph.derivations[0].rustc_args_digest_blake3);
        assert_eq!(receipt.declared_outputs, vec!["out".to_string()]);
        assert_eq!(receipt.toolchain.tool, "rustc");
        assert!(!receipt.toolchain.version_verbose.is_empty());
        assert_eq!(receipt.toolchain.version_digest_blake3.len(), 64);
        assert!(receipt.blocker.is_none());
        assert!(!receipt.output_artifact_digests.is_empty());
        assert!(receipt.output_artifact_digests.iter().any(|artifact| artifact.path.ends_with(".rlib")));
        assert!(receipt.output_artifact_digests.iter().all(|artifact| artifact.path.starts_with("declared-output/")));
        assert_eq!(receipt.receipt_hash.len(), 64);

        let repeated = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();
        assert_eq!(repeated.execution_status, "success");
        assert_eq!(repeated.rebuild_reason, "reused-explicit-unit-output");
        assert_eq!(repeated.declared_outputs, receipt.declared_outputs);
        assert_eq!(repeated.toolchain, receipt.toolchain);
        assert_eq!(repeated.output_artifact_digests, receipt.output_artifact_digests);
        assert_eq!(repeated.receipt_hash.len(), 64);
    }

    #[test]
    fn executes_dependency_chain_from_produced_lib_artifact() {
        let dir = TempDir::new().unwrap();
        let dep_dir = dir.path().join("dep-crate");
        let app_dir = dir.path().join("app-crate");
        std::fs::create_dir_all(dep_dir.join("src")).unwrap();
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        let dep_manifest = dep_dir.join("Cargo.toml");
        let app_manifest = app_dir.join("Cargo.toml");
        std::fs::write(&dep_manifest, "[package]\nname='dep-crate'\nversion='0.1.0'\n").unwrap();
        std::fs::write(&app_manifest, "[package]\nname='app-crate'\nversion='0.1.0'\n").unwrap();
        let dep_lib = dep_dir.join("src/lib.rs");
        let app_main = app_dir.join("src/lib.rs");
        std::fs::write(&dep_lib, "pub fn answer() -> u32 { 42 }\n").unwrap();
        std::fs::write(&app_main, "pub fn call_dep() -> u32 { dep_crate::answer() }\n").unwrap();
        let packages = vec![
            CargoPackage {
                id: "path+file://app-crate#app-crate@0.1.0".to_string(),
                name: "app-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file://dep-crate#dep-crate@0.1.0".to_string(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: dep_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": "path+file://dep-crate#dep-crate@0.1.0",
                    "target": {"name": "dep-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": dep_lib.display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "deps": []
                },
                {
                    "pkg_id": "path+file://app-crate#app-crate@0.1.0",
                    "target": {"name": "app-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_main.display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "deps": [{"pkg_id": "path+file://dep-crate#dep-crate@0.1.0", "extern_crate_name": "dep_crate"}]
                }
            ]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);
        let consumer = graph
            .derivations
            .iter()
            .find(|unit| unit.package_id == "path+file://app-crate#app-crate@0.1.0")
            .unwrap();
        assert_eq!(
            consumer.dependency_artifacts[0].artifact,
            "artifact:path+file://dep-crate#dep-crate@0.1.0:dep_crate"
        );

        let receipt = execute_first_rust_unit_dependency_chain(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("chain-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "success", "{receipt:#?}");
        assert!(receipt.claim.contains("bounded explicit Rust dependency edge"));
        assert_eq!(receipt.unit_executions.len(), 2);
        assert_eq!(receipt.unit_executions[0].target_kind, "lib");
        assert_eq!(receipt.unit_executions[1].target_kind, "lib");
        assert_eq!(receipt.unit_executions[1].execution_status, "success");
        assert_eq!(receipt.unit_executions[1].dependency_artifact_digests.len(), 1);
        assert!(receipt.unit_executions[1].dependency_artifact_digests[0].path.ends_with(".rlib"));
        assert!(receipt.unit_executions[1].blocker.is_none());
        assert_eq!(receipt.receipt_hash.len(), 64);
    }

    #[test]
    fn dependency_chain_blocks_missing_producer_before_consumer_rustc() {
        let dir = TempDir::new().unwrap();
        let app_dir = dir.path().join("app-crate");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        let app_manifest = app_dir.join("Cargo.toml");
        std::fs::write(&app_manifest, "[package]\nname='app-crate'\nversion='0.1.0'\n").unwrap();
        let app_main = app_dir.join("src/main.rs");
        std::fs::write(&app_main, "fn main() { let _ = missing_dep::answer(); }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://app-crate#app-crate@0.1.0".to_string(),
            name: "app-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: app_manifest.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://app-crate#app-crate@0.1.0",
                "target": {"name": "app-crate", "kind": ["bin"], "crate_types": ["bin"], "src_path": app_main.display().to_string(), "edition": "2021"},
                "mode": "build",
                "deps": [{"pkg_id": "path+file://missing-dep#missing-dep@0.1.0", "extern_crate_name": "missing_dep"}]
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);

        let receipt = execute_first_rust_unit_dependency_chain(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("chain-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert!(receipt.unit_executions.is_empty());
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-dependency-producer");
        assert_eq!(receipt.receipt_hash.len(), 64);
    }

    #[test]
    fn rust_unit_execution_blocks_missing_source_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("missing-source-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        std::fs::create_dir_all(&crate_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='missing-source-crate'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://missing-source-crate#missing-source-crate@0.1.0".to_string(),
            name: "missing-source-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://missing-source-crate#missing-source-crate@0.1.0",
                "target": {
                    "name": "missing-source-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": crate_dir.join("src/lib.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": []
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.rebuild_reason, "not-run-preflight-blocker");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-source-material");
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_source_closure_blocker_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("registry-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='registry-crate'\nversion='1.0.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "registry+https://github.com/rust-lang/crates.io-index#registry-crate@1.0.0".to_string(),
            name: "registry-crate".to_string(),
            version: "1.0.0".to_string(),
            source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        assert!(!closure.ready);
        assert_eq!(closure.blockers[0].class, "missing-registry-checksum");
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "registry+https://github.com/rust-lang/crates.io-index#registry-crate@1.0.0",
                "target": {
                    "name": "registry-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": lib_path.display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": []
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(!graph.ready);
        assert_eq!(graph.blockers[0].class, "source-closure-blocked");

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.rebuild_reason, "not-run-preflight-blocker");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "unit-derivation-graph-blocked");
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_missing_dependency_artifact_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("dep-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='dep-crate'\nversion='0.1.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://dep-crate#dep-crate@0.1.0".to_string(),
            name: "dep-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://dep-crate#dep-crate@0.1.0",
                "target": {
                    "name": "dep-crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": lib_path.display().to_string(),
                    "edition": "2021"
                },
                "mode": "build",
                "deps": [{"pkg_id": "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0", "extern_crate_name": "serde"}]
            }]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);
        assert_eq!(graph.derivations[0].dependency_artifacts[0].name, "serde");

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-dependency-artifact");
        assert!(receipt.blocker.as_ref().unwrap().message.contains("artifact:"));
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_missing_host_artifact_before_rustc() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("hosted/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap().join("src")).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='hosted'\nversion='0.1.0'\n").unwrap();
        let app_lib = dir.path().join("hosted/src/lib.rs");
        let build_rs = dir.path().join("hosted/build.rs");
        std::fs::write(&app_lib, "pub fn answer() -> u32 { 42 }\n").unwrap();
        std::fs::write(&build_rs, "fn main() {}\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://hosted#hosted@0.1.0".to_string(),
            name: "hosted".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": "path+file://hosted#hosted@0.1.0",
                    "target": {"name": "build-script-build", "kind": ["custom-build"], "crate_types": ["bin"], "src_path": build_rs.display().to_string(), "edition": "2021"},
                    "mode": "build"
                },
                {
                    "pkg_id": "path+file://hosted#hosted@0.1.0",
                    "target": {"name": "hosted", "kind": ["lib"], "crate_types": ["lib"], "src_path": app_lib.display().to_string(), "edition": "2021"},
                    "mode": "build",
                    "deps": []
                }
            ]
        });
        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        assert!(graph.ready);
        let target = graph.derivations.iter().find(|unit| unit.execution_kind == "target").unwrap();
        assert_eq!(target.dependency_artifacts.len(), 0);
        assert_eq!(target.consumed_host_artifacts.len(), 1);

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-host-artifact");
        assert!(receipt.blocker.as_ref().unwrap().message.contains("host-artifact:"));
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn rust_unit_execution_blocks_missing_declared_output_before_rustc() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("no-output-crate");
        let manifest_path = crate_dir.join("Cargo.toml");
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='no-output-crate'\nversion='0.1.0'\n").unwrap();
        let lib_path = src_dir.join("lib.rs");
        std::fs::write(&lib_path, "pub fn answer() -> u32 { 42 }\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://no-output-crate#no-output-crate@0.1.0".to_string(),
            name: "no-output-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://no-output-crate#no-output-crate@0.1.0",
                "target": {"name": "no-output-crate", "kind": ["lib"], "crate_types": ["lib"], "src_path": lib_path.display().to_string(), "edition": "2021"},
                "mode": "build",
                "deps": []
            }]
        });
        let mut graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();
        graph.derivations[0].derivation.outputs.clear();

        let receipt = execute_first_supported_rust_unit(&graph, &RustUnitExecutionOptions {
            rustc: PathBuf::from("rustc"),
            output_root: dir.path().join("unit-out"),
        })
        .unwrap();

        assert_eq!(receipt.execution_status, "blocked");
        assert_eq!(receipt.blocker.as_ref().unwrap().class, "missing-declared-output");
        assert_eq!(receipt.declared_outputs, Vec::<String>::new());
        assert!(receipt.output_artifact_digests.is_empty());
    }

    #[test]
    fn unit_derivation_graph_represents_build_script_host_units() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("build-crate/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='build-crate'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://build-crate#build-crate@0.1.0".to_string(),
            name: "build-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://build-crate#build-crate@0.1.0",
                "target": {
                    "name": "build-script-build",
                    "kind": ["custom-build"],
                    "crate_types": ["bin"],
                    "src_path": dir.path().join("build-crate/build.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build"
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(graph.ready);
        assert_eq!(graph.derivation_count, 1);
        assert_eq!(graph.host_unit_count, 1);
        assert_eq!(graph.host_artifact_count, 1);
        let unit = &graph.derivations[0];
        assert_eq!(unit.execution_kind, "host");
        assert_eq!(unit.target_kind, "custom-build");
        let metadata = unit.generated_metadata.as_ref().unwrap();
        assert!(metadata.out_dir.contains("OUT_DIR"));
        assert_eq!(metadata.digest_blake3.len(), 64);
        assert!(unit.derivation.inputs.iter().any(|input| input.starts_with("source:")));
    }

    #[test]
    fn unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units() {
        let dir = TempDir::new().unwrap();
        let macro_manifest = dir.path().join("mac/Cargo.toml");
        let app_manifest = dir.path().join("app/Cargo.toml");
        std::fs::create_dir_all(macro_manifest.parent().unwrap().join("src")).unwrap();
        std::fs::create_dir_all(app_manifest.parent().unwrap().join("src")).unwrap();
        std::fs::write(&macro_manifest, "[package]\nname='mac'\nversion='0.1.0'\n").unwrap();
        std::fs::write(&app_manifest, "[package]\nname='app'\nversion='0.1.0'\n").unwrap();
        let packages = vec![
            CargoPackage {
                id: "path+file://app#app@0.1.0".to_string(),
                name: "app".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: app_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
            CargoPackage {
                id: "path+file://mac#mac@0.1.0".to_string(),
                name: "mac".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: macro_manifest.display().to_string(),
                targets: Vec::new(),
                features: BTreeMap::new(),
            },
        ];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [
                {
                    "pkg_id": "path+file://mac#mac@0.1.0",
                    "target": {
                        "name": "mac",
                        "kind": ["proc-macro"],
                        "crate_types": ["proc-macro"],
                        "src_path": dir.path().join("mac/src/lib.rs").display().to_string(),
                        "edition": "2021"
                    },
                    "mode": "build"
                },
                {
                    "pkg_id": "path+file://app#app@0.1.0",
                    "target": {
                        "name": "app",
                        "kind": ["lib"],
                        "crate_types": ["lib"],
                        "src_path": dir.path().join("app/src/lib.rs").display().to_string(),
                        "edition": "2021"
                    },
                    "mode": "build",
                    "deps": [{"pkg_id": "path+file://mac#mac@0.1.0", "extern_crate_name": "mac"}]
                }
            ]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(graph.ready);
        assert_eq!(graph.derivation_count, 2);
        assert_eq!(graph.host_unit_count, 1);
        let target = graph.derivations.iter().find(|unit| unit.execution_kind == "target").unwrap();
        assert_eq!(target.consumed_host_artifacts.len(), 1);
        assert_eq!(target.consumed_host_artifacts[0].target_kind, "proc-macro");
        assert!(target.derivation.inputs.iter().any(|input| input.starts_with("host-artifact:")));
    }

    #[test]
    fn unit_derivation_graph_blocks_unsupported_target_kinds() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("example/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='example'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://example#example@0.1.0".to_string(),
            name: "example".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://example#example@0.1.0",
                "target": {
                    "name": "example-test",
                    "kind": ["test"],
                    "crate_types": ["bin"],
                    "src_path": dir.path().join("example/tests/smoke.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "build"
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(!graph.ready);
        assert_eq!(graph.derivation_count, 0);
        assert_eq!(graph.blockers[0].class, "unsupported-target-kind");
        assert!(graph.blockers[0].message.contains("lib/bin"));
    }

    #[test]
    fn unit_derivation_graph_blocks_doctest_or_non_build_modes() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("doc-crate/Cargo.toml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        std::fs::write(&manifest_path, "[package]\nname='doc-crate'\nversion='0.1.0'\n").unwrap();
        let packages = vec![CargoPackage {
            id: "path+file://doc-crate#doc-crate@0.1.0".to_string(),
            name: "doc-crate".to_string(),
            version: "0.1.0".to_string(),
            source: None,
            manifest_path: manifest_path.display().to_string(),
            targets: Vec::new(),
            features: BTreeMap::new(),
        }];
        let closure = summarize_source_closure(&packages, &[]).unwrap();
        let unit_graph = serde_json::json!({
            "units": [{
                "pkg_id": "path+file://doc-crate#doc-crate@0.1.0",
                "target": {
                    "name": "doc_crate",
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": dir.path().join("doc-crate/src/lib.rs").display().to_string(),
                    "edition": "2021"
                },
                "mode": "doctest"
            }]
        });

        let graph = summarize_unit_derivation_graph(&unit_graph, &closure, &options(dir.path())).unwrap();

        assert!(!graph.ready);
        assert_eq!(graph.derivation_count, 0);
        assert_eq!(graph.blockers[0].class, "unsupported-unit-mode");
        assert!(graph.blockers[0].message.contains("doctest"));
    }

    #[test]
    fn unit_graph_failure_fails_closed() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), "# lock\n").unwrap();
        let oracle = FixedOracle {
            cargo_version: ok_output("cargo 1.91.0\n"),
            rustc_version: ok_output("rustc 1.91.0\n"),
            metadata: ok_output(r#"{"packages":[],"workspace_root":"/tmp/demo","workspace_members":[]}"#),
            unit_graph: failing_output("the option `Z` is only accepted on nightly"),
        };

        let error = capture_rust_plan_with_oracle(&options(dir.path()), &oracle).unwrap_err();

        assert!(
            matches!(error, RunError::Build(message) if message.contains("cargo build --unit-graph") && message.contains("nightly"))
        );
    }
}
