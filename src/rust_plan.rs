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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LockPackageIdentity {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
    pub(crate) checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub(crate) blocker: Option<RustUnitExecutionBlocker>,
    pub(crate) receipt_hash: String,
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
    let unit_derivation_graph = summarize_unit_derivation_graph(&unit_graph_value, &source_closure, options)?;
    let packages = summarize_packages(metadata.packages.clone(), &metadata.workspace_members);

    let mut receipt = RustPlanReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        workspace_root: normalize_path_string(Path::new(&metadata.workspace_root)),
        cargo_version: cargo_version.trim().to_string(),
        rustc_version_verbose: rustc_version_verbose.trim().to_string(),
        lockfile: lockfile_identity(&options.root)?,
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

fn summarize_unit_derivation_graph(
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
    for crate_type in normalized_crate_types(&crate_types, &target_kind) {
        args.push("--crate-type".to_string());
        args.push(crate_type);
    }
    if is_host_target_kind(&target_kind) {
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

pub(crate) fn execute_rust_host_artifact_topology(
    graph: &UnitDerivationGraphSummary,
    options: &RustUnitExecutionOptions,
) -> Result<RustUnitHostArtifactTopologyExecutionReceipt, RunError> {
    if !graph.ready {
        return host_artifact_topology_receipt(
            "blocked",
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
            return host_artifact_topology_receipt("blocked", Vec::new(), Some(blocker));
        }
    }

    let mut executions = Vec::new();
    let mut produced_host_artifacts = BTreeMap::<String, PathBuf>::new();
    for index in host_indices {
        let unit = &graph.derivations[index];
        let receipt = execute_rust_unit(unit, options)?;
        if receipt.execution_status != "success" {
            let blocker = receipt.blocker.clone().unwrap_or_else(|| RustUnitExecutionBlocker {
                class: "host-unit-failed".to_string(),
                message: format!("host unit {} did not execute successfully", unit.unit_id),
            });
            executions.push(receipt);
            return host_artifact_topology_receipt("blocked", executions, Some(blocker));
        }
        match produced_host_artifact_path(unit, options)? {
            Ok(path) => {
                produced_host_artifacts.insert(unit.package_id.clone(), path);
            }
            Err(blocker) => {
                executions.push(receipt);
                return host_artifact_topology_receipt("blocked", executions, Some(blocker));
            }
        }
        executions.push(receipt);
    }

    let mut produced_target_artifacts = BTreeMap::<String, PathBuf>::new();
    for index in ordered_target_indices {
        let unit = &graph.derivations[index];
        let mut executable_unit = bind_all_host_artifacts(unit, &produced_host_artifacts)?;
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
            return host_artifact_topology_receipt("blocked", executions, Some(blocker));
        }
        if unit.target_kind == "lib" {
            match produced_library_artifact_path(&executable_unit, options)? {
                Ok(path) => {
                    produced_target_artifacts.insert(unit.package_id.clone(), path);
                }
                Err(blocker) => {
                    executions.push(receipt);
                    return host_artifact_topology_receipt("blocked", executions, Some(blocker));
                }
            }
        }
        executions.push(receipt);
    }

    host_artifact_topology_receipt("success", executions, None)
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
        let matching_dependencies = bound
            .dependency_artifacts
            .iter()
            .filter(|dependency| dependency.package_id == host_artifact.package_id)
            .cloned()
            .collect::<Vec<_>>();
        for dependency in matching_dependencies {
            bound = bind_dependency_artifact(&bound, &dependency, produced_artifact)?;
        }
    }
    Ok(bound)
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
            dependency_artifact_digests.unwrap(),
            host_artifact_digests.unwrap(),
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
    finalized_execution_receipt(
        unit,
        "success",
        "rebuilt-explicit-unit",
        toolchain,
        dependency_artifact_digests.unwrap(),
        host_artifact_digests.unwrap(),
        output_artifact_digests,
        None,
    )
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
    blocker: Option<RustUnitExecutionBlocker>,
) -> Result<RustUnitHostArtifactTopologyExecutionReceipt, RunError> {
    let mut receipt = RustUnitHostArtifactTopologyExecutionReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION,
        execution_status: execution_status.to_string(),
        claim: "bounded Rust host-artifact topology; not full Cargo compatibility, native-link probing, or a general scheduler"
            .to_string(),
        unit_executions,
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
            },
            CargoPackage {
                id: "path+file:///path-crate#path-crate@0.1.0".to_string(),
                name: "path-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: path_manifest.display().to_string(),
            },
            CargoPackage {
                id: "registry+https://github.com/rust-lang/crates.io-index#registry-crate@1.2.3".to_string(),
                name: "registry-crate".to_string(),
                version: "1.2.3".to_string(),
                source: Some("registry+https://github.com/rust-lang/crates.io-index".to_string()),
                manifest_path: dir.path().join("registry-crate/Cargo.toml").display().to_string(),
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
        assert_eq!(repeated.rebuild_reason, "rebuilt-explicit-unit");
        assert_eq!(repeated.declared_outputs, receipt.declared_outputs);
        assert_eq!(repeated.toolchain, receipt.toolchain);
        assert_eq!(repeated.output_artifact_digests, receipt.output_artifact_digests);
        assert_eq!(repeated.receipt_hash, receipt.receipt_hash);
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
            },
            CargoPackage {
                id: "path+file://dep-crate#dep-crate@0.1.0".to_string(),
                name: "dep-crate".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: dep_manifest.display().to_string(),
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
            },
            CargoPackage {
                id: "path+file://mac#mac@0.1.0".to_string(),
                name: "mac".to_string(),
                version: "0.1.0".to_string(),
                source: None,
                manifest_path: macro_manifest.display().to_string(),
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
