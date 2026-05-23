use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::errors::RunError;

const RECEIPT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_CARGO_PROFILE: &str = "dev";

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
    pub(crate) unit_graph: UnitGraphSummary,
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
pub(crate) struct UnitGraphSummary {
    pub(crate) unit_count: usize,
    pub(crate) root_count: usize,
    pub(crate) digest_blake3: String,
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

#[derive(Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: String,
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
        packages: summarize_packages(metadata.packages, &metadata.workspace_members),
        unit_graph: summarize_unit_graph(unit_graph_value)?,
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
    let workspace_member_set: std::collections::BTreeSet<&str> = workspace_members.iter().map(String::as_str).collect();
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
            unit_graph: ok_output(
                r#"{"roots":[0],"units":[{"pkg_id":"path+file://root#demo@0.1.0","target":{"name":"demo"}}]}"#,
            ),
        };

        let receipt = capture_rust_plan_with_oracle(&options(dir.path()), &oracle).unwrap();

        assert_eq!(receipt.schema_version, RECEIPT_SCHEMA_VERSION);
        assert_eq!(receipt.invocation.features, vec!["a", "b"]);
        assert!(receipt.lockfile.blake3.len() >= 32);
        assert_eq!(receipt.package_count, 1);
        assert_eq!(receipt.packages[0].name, "demo");
        assert_eq!(receipt.unit_graph.unit_count, 1);
        assert_eq!(receipt.unit_graph.root_count, 1);
        assert_eq!(receipt.receipt_hash.len(), 64);
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
