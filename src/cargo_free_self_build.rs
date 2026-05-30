use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::errors::RunError;

const SCHEMA: &str = "mantle-cargo-free-self-build-v1";
const RECEIPT_FILE: &str = "receipt.json";
const STDERR_FILE: &str = "stderr.txt";
const STATUS_FILE: &str = "status.txt";
const META_FILE: &str = "meta.json";
const NON_CLAIMS_FILE: &str = "non-claims.txt";
const SMOKE_STDOUT_FILE: &str = "smoke-stdout.txt";
const SMOKE_STDERR_FILE: &str = "smoke-stderr.txt";
const EXECUTION_DIR: &str = "execution";
const CARGO_SHIM_FILE: &str = "cargo-forbidden";
const CARGO_SHIM_DIR: &str = "cargo-guard-bin";
const CARGO_SHIM_NAME: &str = "cargo";
const CARGO_MARKER_FILE: &str = "cargo-was-invoked";
const PRODUCED_MANTLE_FILE: &str = "mantle";
const MANTLE_TARGET_NAME: &str = "mantle";
const MANTLE_TARGET_KIND: &str = "bin";
const SUCCESS_STATUS: &str = "success";
const BLOCKED_STATUS: &str = "blocked";
const HELP_FLAG: &str = "--help";
const SIGNAL_STATUS_TEXT: &str = "signal";
const BLOCKED_SMOKE_STDOUT: &str = "not run: blocked before binary\n";
const BLOCKED_SMOKE_STDERR_PREFIX: &str = "not run: blocked before binary";
const SOURCE_DIGEST_FIELD: &str = "source_digest";
const SOURCE_DIGEST_ALGORITHM_FIELD: &str = "algorithm";
const SOURCE_DIGEST_VALUE_FIELD: &str = "value";
const SUCCESS_EXIT_CODE: i32 = 0;
const FALLBACK_ERROR_EXIT_CODE: i32 = 1;
const EXPECTED_MANTLE_UNIT_COUNT: usize = 1;
#[cfg(unix)]
const CARGO_SHIM_PERMISSIONS: u32 = 0o755;
#[cfg(unix)]
const UNIX_EXECUTE_BITS: u32 = 0o111;

pub(crate) struct CargoFreeSelfBuildOptions<'a> {
    pub(crate) root: &'a Path,
    pub(crate) out_dir: &'a Path,
    pub(crate) rustc: &'a Path,
    pub(crate) json: bool,
}

#[derive(Debug)]
struct BuildPaths {
    root: PathBuf,
    out_dir: PathBuf,
    execution_dir: PathBuf,
    receipt_path: PathBuf,
    stderr_path: PathBuf,
    status_path: PathBuf,
    marker_path: PathBuf,
    explicit_cargo_shim: PathBuf,
    path_cargo_shim: PathBuf,
    guard_path_dir: PathBuf,
    binary_path: PathBuf,
    meta_path: PathBuf,
}

#[derive(Debug)]
struct ChildRun {
    status_code: Option<i32>,
    receipt: Option<Value>,
    execution_status: String,
    cargo_marker_absent: bool,
    blocker: Option<String>,
}

#[derive(Debug)]
struct ProducedBinary {
    path: PathBuf,
    blake3: String,
    source_digest: Value,
    source_closure_digest_blake3: Option<String>,
    smoke_status_code: Option<i32>,
}

#[derive(Debug, Serialize)]
struct SelfBuildSummary {
    schema: &'static str,
    status: String,
    root: PathBuf,
    out_dir: PathBuf,
    binary: Option<PathBuf>,
    binary_blake3: Option<String>,
    source_digest: Option<Value>,
    source_closure_digest_blake3: Option<String>,
    receipt: PathBuf,
    stderr: PathBuf,
    status_code: Option<i32>,
    execution_status: String,
    cargo_marker_absent: bool,
    unit_count: u64,
    failed_unit_count: u64,
    smoke_status_code: Option<i32>,
    blocker: Option<String>,
    non_claims: Vec<&'static str>,
}

pub(crate) fn cmd_cargo_free_self_build(options: CargoFreeSelfBuildOptions<'_>) -> Result<(), RunError> {
    let paths = prepare_paths(options.root, options.out_dir)?;
    prepare_output_dir(&paths)?;
    write_non_claims(&paths.out_dir)?;
    write_cargo_shim(&paths.explicit_cargo_shim, &paths.marker_path)?;
    write_cargo_shim(&paths.path_cargo_shim, &paths.marker_path)?;

    let mut child = run_rust_plan_child(&paths, options.rustc)?;
    let produced = if child.blocker.is_none() {
        materialize_or_block(&paths, &mut child)?
    } else {
        None
    };
    if child.blocker.is_some() && produced.is_none() {
        write_blocked_smoke_outputs(&paths, child.blocker.as_deref())?;
    }
    let summary = summarize(&paths, &child, produced.as_ref());
    write_summary(&paths.meta_path, &summary)?;
    print_summary(&summary, options.json)?;

    if let Some(blocker) = child.blocker {
        return Err(RunError::Build(format!("Cargo-free self-build blocked: {blocker}")));
    }
    Ok(())
}

fn prepare_paths(root: &Path, out_dir: &Path) -> Result<BuildPaths, RunError> {
    let root =
        fs::canonicalize(root).map_err(|err| internal(format!("canonicalize root {}: {err}", root.display())))?;
    let out_dir = absolutize(&root, out_dir);
    ensure_outside_root(&out_dir, &root)?;
    let execution_dir = out_dir.join(EXECUTION_DIR);
    Ok(BuildPaths {
        root,
        receipt_path: out_dir.join(RECEIPT_FILE),
        stderr_path: out_dir.join(STDERR_FILE),
        status_path: out_dir.join(STATUS_FILE),
        marker_path: out_dir.join(CARGO_MARKER_FILE),
        explicit_cargo_shim: out_dir.join(CARGO_SHIM_FILE),
        path_cargo_shim: out_dir.join(CARGO_SHIM_DIR).join(CARGO_SHIM_NAME),
        guard_path_dir: out_dir.join(CARGO_SHIM_DIR),
        binary_path: out_dir.join(PRODUCED_MANTLE_FILE),
        meta_path: out_dir.join(META_FILE),
        out_dir,
        execution_dir,
    })
}

fn prepare_output_dir(paths: &BuildPaths) -> Result<(), RunError> {
    fs::create_dir_all(&paths.out_dir)
        .map_err(|err| internal(format!("create output dir {}: {err}", paths.out_dir.display())))?;
    remove_owned_path(&paths.execution_dir)?;
    remove_owned_path(&paths.guard_path_dir)?;
    let owned_files = vec![
        paths.receipt_path.clone(),
        paths.stderr_path.clone(),
        paths.status_path.clone(),
        paths.marker_path.clone(),
        paths.explicit_cargo_shim.clone(),
        paths.binary_path.clone(),
        paths.meta_path.clone(),
        paths.out_dir.join(SMOKE_STDOUT_FILE),
        paths.out_dir.join(SMOKE_STDERR_FILE),
    ];
    for path in &owned_files {
        remove_owned_path(path)?;
    }
    fs::create_dir_all(&paths.execution_dir)
        .map_err(|err| internal(format!("create execution dir {}: {err}", paths.execution_dir.display())))
}

fn remove_owned_path(path: &Path) -> Result<(), RunError> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|err| internal(format!("remove dir {}: {err}", path.display())))
    } else {
        fs::remove_file(path).map_err(|err| internal(format!("remove file {}: {err}", path.display())))
    }
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    debug_assert!(root.is_absolute());
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn ensure_outside_root(out_dir: &Path, root: &Path) -> Result<(), RunError> {
    debug_assert!(root.is_absolute());
    debug_assert!(out_dir.is_absolute());
    if out_dir.starts_with(root) {
        return Err(RunError::Build(format!(
            "--out {} is inside source root {}; choose /tmp or another outside path so build evidence does not change native source digests",
            out_dir.display(),
            root.display()
        )));
    }
    Ok(())
}

fn run_rust_plan_child(paths: &BuildPaths, rustc: &Path) -> Result<ChildRun, RunError> {
    let current_exe = env::current_exe().map_err(|err| internal(format!("resolve current executable: {err}")))?;
    let guarded_path = guarded_path(&paths.guard_path_dir)?;
    let output = Command::new(&current_exe)
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&paths.root)
        .arg("--cargo")
        .arg(&paths.explicit_cargo_shim)
        .arg("--rustc")
        .arg(rustc)
        .arg("--no-cargo-oracle")
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(&paths.execution_dir)
        .current_dir(&paths.root)
        .env("CARGO", &paths.path_cargo_shim)
        .env("PATH", guarded_path)
        .output()
        .map_err(|err| internal(format!("launch {} rust-plan: {err}", current_exe.display())))?;

    write_bytes(&paths.receipt_path, &output.stdout)?;
    write_bytes(&paths.stderr_path, &output.stderr)?;
    write_text(&paths.status_path, &status_text(output.status.code()))?;
    let receipt = parse_receipt(&paths.receipt_path, output.status.success())?;
    let execution_status = receipt_execution_status(receipt.as_ref());
    let cargo_marker_absent = !paths.marker_path.exists();
    let blocker = child_blocker(output.status.code(), &execution_status, cargo_marker_absent);
    Ok(ChildRun {
        status_code: output.status.code(),
        receipt,
        execution_status,
        cargo_marker_absent,
        blocker,
    })
}

fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
    if !cargo_marker_absent {
        return Some("cargo guard was invoked".to_string());
    }
    if status_code != Some(SUCCESS_EXIT_CODE) {
        return Some(format!("rust-plan exited with {}", status_text(status_code).trim_end()));
    }
    if execution_status != SUCCESS_STATUS {
        return Some(format!("topology execution status was {execution_status}"));
    }
    None
}

fn materialize_or_block(paths: &BuildPaths, child: &mut ChildRun) -> Result<Option<ProducedBinary>, RunError> {
    match materialize_binary(paths, child.receipt.as_ref()) {
        Ok(produced) => {
            if produced.smoke_status_code != Some(SUCCESS_EXIT_CODE) {
                child.blocker =
                    Some(format!("smoke check exited with {}", status_text(produced.smoke_status_code).trim_end()));
            }
            Ok(Some(produced))
        }
        Err(err) => {
            child.blocker = Some(err.message().to_string());
            Ok(None)
        }
    }
}

fn materialize_binary(paths: &BuildPaths, receipt: Option<&Value>) -> Result<ProducedBinary, RunError> {
    let receipt = receipt.ok_or_else(|| internal("successful rust-plan produced no receipt".to_string()))?;
    let (unit_id, source_digest) = mantle_unit_from_receipt(receipt)?;
    let source_closure_digest_blake3 = receipt
        .pointer("/rust_plan/source_closure/digest_blake3")
        .and_then(Value::as_str)
        .map(str::to_string);
    let source_binary = paths.execution_dir.join(safe_path_component(&unit_id)).join(MANTLE_TARGET_NAME);
    require_executable(&source_binary)?;
    fs::copy(&source_binary, &paths.binary_path).map_err(|err| {
        internal(format!(
            "copy produced binary {} to {}: {err}",
            source_binary.display(),
            paths.binary_path.display()
        ))
    })?;
    require_executable(&paths.binary_path)?;
    let blake3 = blake3_file(&paths.binary_path)?;
    let smoke_status_code = run_smoke(&paths.binary_path, &paths.out_dir)?;
    Ok(ProducedBinary {
        path: paths.binary_path.clone(),
        blake3,
        source_digest,
        source_closure_digest_blake3,
        smoke_status_code: Some(smoke_status_code),
    })
}

fn mantle_unit_from_receipt(receipt: &Value) -> Result<(String, Value), RunError> {
    let units = receipt
        .pointer("/topology_execution/unit_executions")
        .and_then(Value::as_array)
        .ok_or_else(|| internal("receipt has no topology_execution.unit_executions".to_string()))?;
    let matches = units.iter().filter(|unit| is_successful_mantle_unit(unit)).collect::<Vec<_>>();
    if matches.len() != EXPECTED_MANTLE_UNIT_COUNT {
        return Err(RunError::Build(format!("expected one successful mantle bin unit, found {}", matches.len())));
    }
    let unit = matches[0];
    let unit_id = unit
        .get("unit_id")
        .and_then(Value::as_str)
        .ok_or_else(|| internal("mantle bin unit lacks unit_id".to_string()))?;
    let source_digest = source_digest_from_unit(unit)?;
    Ok((unit_id.to_string(), source_digest))
}

fn source_digest_from_unit(unit: &Value) -> Result<Value, RunError> {
    let digest = unit
        .get(SOURCE_DIGEST_FIELD)
        .cloned()
        .ok_or_else(|| RunError::Build("mantle bin unit lacks source_digest".to_string()))?;
    if digest.is_null() {
        return Err(RunError::Build("mantle bin unit lacks source_digest".to_string()));
    }
    if digest.get(SOURCE_DIGEST_ALGORITHM_FIELD).and_then(Value::as_str).is_none() {
        return Err(RunError::Build("mantle bin unit source_digest lacks algorithm".to_string()));
    }
    if digest.get(SOURCE_DIGEST_VALUE_FIELD).and_then(Value::as_str).is_none() {
        return Err(RunError::Build("mantle bin unit source_digest lacks value".to_string()));
    }
    Ok(digest)
}

fn is_successful_mantle_unit(unit: &Value) -> bool {
    unit.get("target_name").and_then(Value::as_str) == Some(MANTLE_TARGET_NAME)
        && unit.get("target_kind").and_then(Value::as_str) == Some(MANTLE_TARGET_KIND)
        && unit.get("execution_status").and_then(Value::as_str) == Some(SUCCESS_STATUS)
}

fn summarize(paths: &BuildPaths, child: &ChildRun, produced: Option<&ProducedBinary>) -> SelfBuildSummary {
    let receipt = child.receipt.as_ref();
    SelfBuildSummary {
        schema: SCHEMA,
        status: if child.blocker.is_none() {
            SUCCESS_STATUS
        } else {
            BLOCKED_STATUS
        }
        .to_string(),
        root: paths.root.clone(),
        out_dir: paths.out_dir.clone(),
        binary: produced.map(|value| value.path.clone()),
        binary_blake3: produced.map(|value| value.blake3.clone()),
        source_digest: produced.map(|value| value.source_digest.clone()),
        source_closure_digest_blake3: produced.and_then(|value| value.source_closure_digest_blake3.clone()),
        receipt: paths.receipt_path.clone(),
        stderr: paths.stderr_path.clone(),
        status_code: child.status_code,
        execution_status: child.execution_status.clone(),
        cargo_marker_absent: child.cargo_marker_absent,
        unit_count: receipt.map(unit_count).unwrap_or_default(),
        failed_unit_count: receipt.map(failed_unit_count).unwrap_or_default(),
        smoke_status_code: produced.and_then(|value| value.smoke_status_code),
        blocker: child.blocker.clone(),
        non_claims: vec![
            "not-crunch-bootstrap",
            "not-release-reproducibility",
            "not-source-built-toolchain-closure",
            "not-full-cargo-compatibility",
        ],
    }
}

fn unit_count(receipt: &Value) -> u64 {
    receipt
        .pointer("/topology_execution/unit_executions")
        .and_then(Value::as_array)
        .map(|items| items.len() as u64)
        .unwrap_or_default()
}

fn failed_unit_count(receipt: &Value) -> u64 {
    receipt
        .pointer("/topology_execution/unit_executions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| item.get("execution_status").and_then(Value::as_str) != Some(SUCCESS_STATUS))
                .count() as u64
        })
        .unwrap_or_default()
}

fn parse_receipt(path: &Path, command_succeeded: bool) -> Result<Option<Value>, RunError> {
    if !command_succeeded {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|err| internal(format!("read receipt {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|err| internal(format!("parse receipt {}: {err}", path.display())))
}

fn receipt_execution_status(receipt: Option<&Value>) -> String {
    receipt
        .and_then(|value| value.pointer("/topology_execution/execution_status"))
        .and_then(Value::as_str)
        .unwrap_or("missing")
        .to_string()
}

fn run_smoke(binary: &Path, out_dir: &Path) -> Result<i32, RunError> {
    let output = Command::new(binary)
        .arg(HELP_FLAG)
        .output()
        .map_err(|err| internal(format!("run smoke {}: {err}", binary.display())))?;
    write_bytes(&out_dir.join(SMOKE_STDOUT_FILE), &output.stdout)?;
    write_bytes(&out_dir.join(SMOKE_STDERR_FILE), &output.stderr)?;
    Ok(output.status.code().unwrap_or(FALLBACK_ERROR_EXIT_CODE))
}

fn write_blocked_smoke_outputs(paths: &BuildPaths, blocker: Option<&str>) -> Result<(), RunError> {
    let stderr = match blocker {
        Some(message) => format!("{BLOCKED_SMOKE_STDERR_PREFIX}: {message}\n"),
        None => format!("{BLOCKED_SMOKE_STDERR_PREFIX}\n"),
    };
    write_text(&paths.out_dir.join(SMOKE_STDOUT_FILE), BLOCKED_SMOKE_STDOUT)?;
    write_text(&paths.out_dir.join(SMOKE_STDERR_FILE), &stderr)
}

fn guarded_path(cargo_path_dir: &Path) -> Result<OsString, RunError> {
    let mut paths = vec![cargo_path_dir.to_path_buf()];
    if let Some(path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&path));
    }
    env::join_paths(paths).map_err(|err| internal(format!("construct guarded PATH: {err}")))
}

fn write_cargo_shim(path: &Path, marker: &Path) -> Result<(), RunError> {
    let parent = path.parent().ok_or_else(|| internal(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent).map_err(|err| internal(format!("create {}: {err}", parent.display())))?;
    let mut file = fs::File::create(path).map_err(|err| internal(format!("create {}: {err}", path.display())))?;
    writeln!(file, "#!/bin/sh").map_err(|err| internal(format!("write {}: {err}", path.display())))?;
    writeln!(file, "printf invoked > {}", shell_quote(marker))
        .map_err(|err| internal(format!("write {}: {err}", path.display())))?;
    writeln!(file, "exit 99").map_err(|err| internal(format!("write {}: {err}", path.display())))?;
    set_executable(path)
}

fn shell_quote(path: &Path) -> String {
    let raw = path.as_os_str().to_string_lossy();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions =
        fs::metadata(path).map_err(|err| internal(format!("stat {}: {err}", path.display())))?.permissions();
    permissions.set_mode(CARGO_SHIM_PERMISSIONS);
    fs::set_permissions(path, permissions).map_err(|err| internal(format!("chmod {}: {err}", path.display())))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), RunError> {
    Ok(())
}

fn require_executable(path: &Path) -> Result<(), RunError> {
    let metadata = fs::metadata(path).map_err(|err| internal(format!("stat {}: {err}", path.display())))?;
    if !metadata.is_file() {
        return Err(internal(format!("{} is not a file", path.display())));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & UNIX_EXECUTE_BITS == 0 {
            return Err(internal(format!("{} is not executable", path.display())));
        }
    }
    Ok(())
}

fn safe_path_component(value: &str) -> String {
    debug_assert!(!value.is_empty());
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

fn blake3_file(path: &Path) -> Result<String, RunError> {
    let bytes = fs::read(path).map_err(|err| internal(format!("read {}: {err}", path.display())))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn write_non_claims(out_dir: &Path) -> Result<(), RunError> {
    let text = [
        "This build claims only bounded Mantle Cargo-free Rust topology execution.",
        "This build does not claim Crunch bootstrap or release reproducibility.",
        "This build does not claim source-built compiler/toolchain closure provenance.",
        "This build does not claim full Cargo compatibility, tests, doctests, examples, or general resolver parity.",
    ]
    .join("\n");
    write_text(&out_dir.join(NON_CLAIMS_FILE), &format!("{text}\n"))
}

fn write_summary(path: &Path, summary: &SelfBuildSummary) -> Result<(), RunError> {
    let value = json!(summary);
    let bytes = serde_json::to_vec_pretty(&value).map_err(|err| internal(format!("serialize summary: {err}")))?;
    write_bytes(path, &bytes)
}

fn print_summary(summary: &SelfBuildSummary, json_mode: bool) -> Result<(), RunError> {
    if json_mode {
        let rendered = serde_json::to_string(summary).map_err(|err| internal(format!("render summary: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("Cargo-free self-build: {}", summary.status);
    if let Some(binary) = &summary.binary {
        println!("binary: {}", binary.display());
    }
    if let Some(digest) = &summary.binary_blake3 {
        println!("binary_blake3: {digest}");
    }
    println!("receipt: {}", summary.receipt.display());
    Ok(())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    fs::write(path, bytes).map_err(|err| internal(format!("write {}: {err}", path.display())))
}

fn write_text(path: &Path, text: &str) -> Result<(), RunError> {
    write_bytes(path, text.as_bytes())
}

fn status_text(code: Option<i32>) -> String {
    match code {
        Some(code) => format!("{code}\n"),
        None => format!("{SIGNAL_STATUS_TEXT}\n"),
    }
}

fn internal(message: String) -> RunError {
    RunError::Internal(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_path_component_replaces_unsafe_path_bytes() {
        assert_eq!(safe_path_component("unit:with/slash"), "unit_with_slash");
        assert_eq!(safe_path_component("unit.with-dash_ok"), "unit.with-dash_ok");
    }

    #[test]
    fn child_blocker_fails_when_cargo_guard_was_invoked() {
        let blocker = child_blocker(Some(SUCCESS_EXIT_CODE), SUCCESS_STATUS, false).unwrap();
        assert_eq!(blocker, "cargo guard was invoked");
    }

    #[test]
    fn child_blocker_accepts_successful_guarded_execution() {
        assert!(child_blocker(Some(SUCCESS_EXIT_CODE), SUCCESS_STATUS, true).is_none());
    }

    #[test]
    fn mantle_unit_from_receipt_requires_source_digest() {
        let receipt = json!({
            "topology_execution": {
                "unit_executions": [{
                    "unit_id": "unit-1",
                    "target_name": MANTLE_TARGET_NAME,
                    "target_kind": MANTLE_TARGET_KIND,
                    "execution_status": SUCCESS_STATUS
                }]
            }
        });

        let err = mantle_unit_from_receipt(&receipt).unwrap_err();
        assert_eq!(err.message(), "mantle bin unit lacks source_digest");
    }

    #[test]
    fn mantle_unit_from_receipt_rejects_null_source_digest() {
        let receipt = json!({
            "topology_execution": {
                "unit_executions": [{
                    "unit_id": "unit-1",
                    "target_name": MANTLE_TARGET_NAME,
                    "target_kind": MANTLE_TARGET_KIND,
                    "execution_status": SUCCESS_STATUS,
                    "source_digest": null
                }]
            }
        });

        let err = mantle_unit_from_receipt(&receipt).unwrap_err();
        assert_eq!(err.message(), "mantle bin unit lacks source_digest");
    }
}
