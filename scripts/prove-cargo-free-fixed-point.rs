#!/usr/bin/env -S nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.2"
serde_json = "1"
---

//! Prove a bounded Mantle Cargo-free fixed point: host Mantle builds stage1,
//! stage1 builds stage2, and the two produced Mantle binaries have identical
//! BLAKE3 digests. This is still not Mantle bootstrap or release evidence.

use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde_json::Value;
use serde_json::json;

const SCHEMA: &str = "mantle-cargo-free-fixed-point-proof-v1";
const DEFAULT_BUNDLE_ROOT: &str = "mantle-cargo-free-fixed-point-proof";
const CARGO_MARKER_FILE: &str = "cargo-was-invoked";
const CARGO_SHIM_FILE: &str = "cargo-forbidden";
const CARGO_SHIM_DIR: &str = "cargo-guard-bin";
const CARGO_SHIM_NAME: &str = "cargo";
const RECEIPT_FILE: &str = "receipt.json";
const STDERR_FILE: &str = "stderr.txt";
const STATUS_FILE: &str = "status.txt";
const SMOKE_STDOUT_FILE: &str = "smoke-stdout.txt";
const SMOKE_STDERR_FILE: &str = "smoke-stderr.txt";
const EXECUTION_DIR: &str = "execution";
const PRODUCED_MANTLE_FILE: &str = "mantle";
const MANTLE_TARGET_NAME: &str = "mantle";
const MANTLE_TARGET_KIND: &str = "bin";
const HELP_FLAG: &str = "--help";
const FIXED_POINT_SUCCESS: &str = "success";
const FIXED_POINT_MISMATCH: &str = "mismatch";
const FIXED_POINT_BLOCKED: &str = "blocked";
const NON_CLAIMS_FILE: &str = "non-claims.txt";
const META_FILE: &str = "meta.json";
const PRE_FLIGHT_FILE: &str = "preflight.json";
const BUNDLE_PREFIX: &str = "run";
const FAKE_CARGO_PERMISSIONS: u32 = 0o755;
const HEX_DIGEST_LEN: usize = 64;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args_os().skip(1))?;
    if args.help {
        print_usage();
        return Ok(());
    }

    let root = canonicalize_existing(&args.root, "root")?;
    let bundle_dir = absolutize(&root, &args.bundle_dir.unwrap_or_else(default_bundle_dir));
    ensure_bundle_outside_root(&bundle_dir, &root)?;
    let host_mantle = resolve_mantle_bin(args.mantle_bin.as_deref())?;
    let rustc = resolve_tool(args.rustc.as_deref(), "rustc")?;
    let jq = resolve_tool(None, "jq").ok();

    if args.check {
        println!("schema: {SCHEMA}");
        println!("root: {}", root.display());
        println!("bundle: {}", bundle_dir.display());
        println!("mantle: {}", host_mantle.display());
        println!("rustc: {}", rustc.display());
        return Ok(());
    }

    fs::create_dir_all(&bundle_dir).map_err(|err| format!("create {}: {err}", bundle_dir.display()))?;
    write_non_claims(&bundle_dir)?;
    write_preflight(&bundle_dir, &root, &host_mantle, &rustc, jq.as_deref())?;

    let shared_execution_dir = bundle_dir.join(EXECUTION_DIR);
    let stage1 = match execute_stage(StageName::Stage1, &bundle_dir, &shared_execution_dir, &root, &host_mantle, &rustc)
    {
        Ok(stage) => stage,
        Err(err) => {
            let stage = setup_blocked_stage(StageName::Stage1, &bundle_dir, &shared_execution_dir, err);
            write_meta(&bundle_dir, &root, &stage, None, FIXED_POINT_BLOCKED)?;
            return Err(format!("stage1 blocked; see {}", stage.dir.display()));
        }
    };
    if !stage1.success {
        write_meta(&bundle_dir, &root, &stage1, None, FIXED_POINT_BLOCKED)?;
        return Err(format!("stage1 blocked; see {}", stage1.dir.display()));
    }
    let Some(stage1_binary) = stage1.binary.as_deref() else {
        let mut blocked = stage1;
        blocked.success = false;
        blocked.blocker = Some("stage1 succeeded without produced binary path".to_string());
        write_meta(&bundle_dir, &root, &blocked, None, FIXED_POINT_BLOCKED)?;
        return Err(format!("stage1 blocked; see {}", blocked.dir.display()));
    };

    let stage2 =
        match execute_stage(StageName::Stage2, &bundle_dir, &shared_execution_dir, &root, stage1_binary, &rustc) {
            Ok(stage) => stage,
            Err(err) => {
                let stage = setup_blocked_stage(StageName::Stage2, &bundle_dir, &shared_execution_dir, err);
                write_meta(&bundle_dir, &root, &stage1, Some(&stage), FIXED_POINT_BLOCKED)?;
                return Err(format!("stage2 blocked; see {}", stage.dir.display()));
            }
        };
    if !stage2.success {
        write_meta(&bundle_dir, &root, &stage1, Some(&stage2), FIXED_POINT_BLOCKED)?;
        return Err(format!("stage2 blocked; see {}", stage2.dir.display()));
    }

    let stage1_digest = stage1.binary_digest.as_deref().ok_or("stage1 succeeded without binary digest")?;
    let stage2_digest = stage2.binary_digest.as_deref().ok_or("stage2 succeeded without binary digest")?;
    debug_assert_eq!(stage1_digest.len(), HEX_DIGEST_LEN);
    debug_assert_eq!(stage2_digest.len(), HEX_DIGEST_LEN);
    let status = if stage1_digest == stage2_digest {
        FIXED_POINT_SUCCESS
    } else {
        FIXED_POINT_MISMATCH
    };
    write_meta(&bundle_dir, &root, &stage1, Some(&stage2), status)?;

    println!("fixed-point proof bundle: {}", bundle_dir.display());
    println!("stage1: {stage1_digest}");
    println!("stage2: {stage2_digest}");
    println!("status: {status}");

    if status != FIXED_POINT_SUCCESS {
        return Err("stage1/stage2 Mantle binary digests differ".to_string());
    }
    Ok(())
}

#[derive(Debug)]
struct Args {
    check: bool,
    help: bool,
    bundle_dir: Option<PathBuf>,
    mantle_bin: Option<PathBuf>,
    root: PathBuf,
    rustc: Option<PathBuf>,
}

impl Args {
    fn parse<I>(mut args: I) -> Result<Self, String>
    where I: Iterator<Item = OsString> {
        let mut parsed = Args {
            check: false,
            help: false,
            bundle_dir: None,
            mantle_bin: env::var_os("MANTLE_BIN").map(PathBuf::from),
            root: env::var_os("MANTLE_PROOF_ROOT").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")),
            rustc: env::var_os("RUSTC").map(PathBuf::from),
        };
        while let Some(arg) = args.next() {
            match arg.to_string_lossy().as_ref() {
                "--check" => parsed.check = true,
                "--help" | "-h" => parsed.help = true,
                "--bundle-dir" => parsed.bundle_dir = Some(next_path(&mut args, "--bundle-dir")?),
                "--mantle-bin" => parsed.mantle_bin = Some(next_path(&mut args, "--mantle-bin")?),
                "--root" => parsed.root = next_path(&mut args, "--root")?,
                "--rustc" => parsed.rustc = Some(next_path(&mut args, "--rustc")?),
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn next_path<I>(args: &mut I, flag: &str) -> Result<PathBuf, String>
where I: Iterator<Item = OsString> {
    args.next().map(PathBuf::from).ok_or_else(|| format!("{flag} requires a path"))
}

#[derive(Clone, Copy, Debug)]
enum StageName {
    Stage1,
    Stage2,
}

impl StageName {
    fn as_str(self) -> &'static str {
        match self {
            StageName::Stage1 => "stage1",
            StageName::Stage2 => "stage2",
        }
    }
}

#[derive(Debug)]
struct StageResult {
    name: &'static str,
    dir: PathBuf,
    execution_dir: PathBuf,
    receipt_path: PathBuf,
    stderr_path: PathBuf,
    status_code: Option<i32>,
    execution_status: String,
    cargo_marker_absent: bool,
    success: bool,
    unit_count: u64,
    failed_unit_count: u64,
    binary: Option<PathBuf>,
    binary_digest: Option<String>,
    smoke_status_code: Option<i32>,
    blocker: Option<String>,
}

#[derive(Debug)]
struct StageArtifact {
    binary: PathBuf,
    digest: String,
    smoke_status_code: i32,
}

fn execute_stage(
    stage: StageName,
    bundle_dir: &Path,
    execution_dir: &Path,
    root: &Path,
    mantle_bin: &Path,
    rustc: &Path,
) -> Result<StageResult, String> {
    debug_assert!(root.is_absolute());
    debug_assert!(bundle_dir.is_absolute() || bundle_dir.components().count() > 0);
    debug_assert!(execution_dir.is_absolute() || execution_dir.components().count() > 0);
    let stage_dir = bundle_dir.join(stage.as_str());
    fs::create_dir_all(&stage_dir).map_err(|err| format!("create {}: {err}", stage_dir.display()))?;
    reset_dir(execution_dir)?;

    let receipt_path = stage_dir.join(RECEIPT_FILE);
    let stderr_path = stage_dir.join(STDERR_FILE);
    let cargo_marker = stage_dir.join(CARGO_MARKER_FILE);
    let explicit_cargo_shim = stage_dir.join(CARGO_SHIM_FILE);
    let cargo_path_dir = stage_dir.join(CARGO_SHIM_DIR);
    let path_cargo_shim = cargo_path_dir.join(CARGO_SHIM_NAME);
    write_cargo_shim(&explicit_cargo_shim, &cargo_marker)?;
    write_cargo_shim(&path_cargo_shim, &cargo_marker)?;
    let guarded_path = guarded_path(&cargo_path_dir)?;

    let output = Command::new(mantle_bin)
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(root)
        .arg("--cargo")
        .arg(&explicit_cargo_shim)
        .arg("--rustc")
        .arg(rustc)
        .arg("--no-cargo-oracle")
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(execution_dir)
        .current_dir(root)
        .env("CARGO", &path_cargo_shim)
        .env("PATH", guarded_path)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(err) => {
            let blocker = format!("launch {} for {}: {err}", mantle_bin.display(), stage.as_str());
            let _ = fs::write(&stderr_path, format!("{blocker}\n"));
            let _ = fs::write(stage_dir.join(STATUS_FILE), "launch-failed\n");
            return Ok(blocked_stage(
                stage,
                stage_dir,
                execution_dir,
                receipt_path,
                stderr_path,
                "launch-failed",
                None,
                blocker,
            ));
        }
    };

    let mut blocker = None;
    record_file_write(&receipt_path, &output.stdout, &mut blocker);
    record_file_write(&stderr_path, &output.stderr, &mut blocker);
    record_file_write(&stage_dir.join(STATUS_FILE), status_text(output.status.code()).as_bytes(), &mut blocker);

    let receipt = parse_stage_receipt(&receipt_path, output.status.success(), &mut blocker);
    let execution_status = receipt_execution_status(receipt.as_ref());
    let unit_count = receipt.as_ref().map(unit_count).unwrap_or_default();
    let failed_unit_count = receipt.as_ref().map(failed_unit_count).unwrap_or_default();
    let cargo_marker_absent = !cargo_marker.exists();
    if !cargo_marker_absent {
        blocker = stage_command_blocker(output.status.code(), &execution_status, cargo_marker_absent);
    } else if blocker.is_none() {
        blocker = stage_command_blocker(output.status.code(), &execution_status, cargo_marker_absent);
    }

    let mut binary = None;
    let mut binary_digest = None;
    let mut smoke_status_code = None;
    if blocker.is_none() {
        match materialize_stage_artifact(receipt.as_ref(), execution_dir, &stage_dir) {
            Ok(artifact) => {
                if artifact.smoke_status_code != 0 {
                    blocker = Some(format!("smoke check exited with {}", artifact.smoke_status_code));
                }
                smoke_status_code = Some(artifact.smoke_status_code);
                binary_digest = Some(artifact.digest);
                binary = Some(artifact.binary);
            }
            Err(err) => blocker = Some(err),
        }
    }

    Ok(StageResult {
        name: stage.as_str(),
        dir: stage_dir,
        execution_dir: execution_dir.to_path_buf(),
        receipt_path,
        stderr_path,
        status_code: output.status.code(),
        execution_status,
        cargo_marker_absent,
        success: blocker.is_none(),
        unit_count,
        failed_unit_count,
        binary,
        binary_digest,
        smoke_status_code,
        blocker,
    })
}

fn blocked_stage(
    stage: StageName,
    stage_dir: PathBuf,
    execution_dir: &Path,
    receipt_path: PathBuf,
    stderr_path: PathBuf,
    execution_status: &str,
    status_code: Option<i32>,
    blocker: String,
) -> StageResult {
    StageResult {
        name: stage.as_str(),
        dir: stage_dir,
        execution_dir: execution_dir.to_path_buf(),
        receipt_path,
        stderr_path,
        status_code,
        execution_status: execution_status.to_string(),
        cargo_marker_absent: true,
        success: false,
        unit_count: 0,
        failed_unit_count: 0,
        binary: None,
        binary_digest: None,
        smoke_status_code: None,
        blocker: Some(blocker),
    }
}

fn setup_blocked_stage(stage: StageName, bundle_dir: &Path, execution_dir: &Path, blocker: String) -> StageResult {
    let stage_dir = bundle_dir.join(stage.as_str());
    let receipt_path = stage_dir.join(RECEIPT_FILE);
    let stderr_path = stage_dir.join(STDERR_FILE);
    blocked_stage(stage, stage_dir, execution_dir, receipt_path, stderr_path, "setup-failed", None, blocker)
}

fn print_usage() {
    println!(
        "usage: scripts/prove-cargo-free-fixed-point.rs [--check] [--bundle-dir DIR] [--mantle-bin PATH] [--root DIR] [--rustc PATH]\n\nBuild stage1 Mantle with host Mantle, build stage2 Mantle with stage1, compare BLAKE3 digests."
    );
}

fn default_bundle_dir() -> PathBuf {
    let seconds = unix_seconds();
    env::temp_dir().join(DEFAULT_BUNDLE_ROOT).join(format!("{BUNDLE_PREFIX}-{seconds}"))
}

fn unix_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn canonicalize_existing(path: &Path, label: &str) -> Result<PathBuf, String> {
    debug_assert!(!label.is_empty());
    fs::canonicalize(path).map_err(|err| format!("canonicalize {label} {}: {err}", path.display()))
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    debug_assert!(root.is_absolute());
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn ensure_bundle_outside_root(bundle_dir: &Path, root: &Path) -> Result<(), String> {
    debug_assert!(root.is_absolute());
    debug_assert!(bundle_dir.is_absolute());
    if bundle_dir.starts_with(root) {
        return Err(format!(
            "bundle dir {} is inside source root {}; use /tmp or another outside path so proof artifacts do not change native source digests",
            bundle_dir.display(),
            root.display()
        ));
    }
    Ok(())
}

fn reset_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|err| format!("remove {}: {err}", path.display()))?;
    }
    fs::create_dir_all(path).map_err(|err| format!("create {}: {err}", path.display()))
}

fn guarded_path(cargo_path_dir: &Path) -> Result<OsString, String> {
    debug_assert!(cargo_path_dir.components().count() > 0);
    let mut paths = vec![cargo_path_dir.to_path_buf()];
    if let Some(path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&path));
    }
    env::join_paths(paths).map_err(|err| format!("construct guarded PATH: {err}"))
}

fn record_file_write(path: &Path, bytes: &[u8], blocker: &mut Option<String>) {
    if blocker.is_some() {
        let _ = fs::write(path, bytes);
        return;
    }
    if let Err(err) = fs::write(path, bytes) {
        *blocker = Some(format!("write {}: {err}", path.display()));
    }
}

fn parse_stage_receipt(path: &Path, command_succeeded: bool, blocker: &mut Option<String>) -> Option<Value> {
    match parse_receipt(path) {
        Ok(value) => Some(value),
        Err(err) => {
            if command_succeeded && blocker.is_none() {
                *blocker = Some(err);
            }
            None
        }
    }
}

fn receipt_execution_status(receipt: Option<&Value>) -> String {
    receipt
        .and_then(|value| value.pointer("/topology_execution/execution_status"))
        .and_then(Value::as_str)
        .unwrap_or("missing")
        .to_string()
}

fn stage_command_blocker(
    status_code: Option<i32>,
    execution_status: &str,
    cargo_marker_absent: bool,
) -> Option<String> {
    if !cargo_marker_absent {
        return Some("cargo guard was invoked".to_string());
    }
    if status_code != Some(0) {
        return Some(format!("stage command exited with {}", status_text(status_code).trim_end()));
    }
    if execution_status != "success" {
        return Some(format!("topology execution status was {execution_status}"));
    }
    None
}

fn materialize_stage_artifact(
    receipt: Option<&Value>,
    execution_dir: &Path,
    stage_dir: &Path,
) -> Result<StageArtifact, String> {
    let value = receipt.ok_or("stage succeeded without JSON receipt")?;
    let found = mantle_binary_from_receipt(value, execution_dir)?;
    let produced = stage_dir.join(PRODUCED_MANTLE_FILE);
    fs::copy(&found, &produced).map_err(|err| format!("copy {} to {}: {err}", found.display(), produced.display()))?;
    require_executable(&produced)?;
    let digest = blake3_file(&produced)?;
    let smoke_status_code = run_smoke(&produced, stage_dir)?;
    Ok(StageArtifact {
        binary: produced,
        digest,
        smoke_status_code,
    })
}

fn resolve_mantle_bin(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        let resolved = canonicalize_existing(path, "mantle binary")?;
        require_executable(&resolved)?;
        return Ok(resolved);
    }
    if let Ok(path) = resolve_tool(None, "mantle") {
        return Ok(path);
    }
    let fallback = PathBuf::from("/home/brittonr/.cargo-target/debug/mantle");
    let resolved = canonicalize_existing(&fallback, "fallback mantle binary")?;
    require_executable(&resolved)?;
    Ok(resolved)
}

fn resolve_tool(explicit: Option<&Path>, name: &str) -> Result<PathBuf, String> {
    debug_assert!(!name.is_empty());
    if let Some(path) = explicit {
        let resolved = canonicalize_existing(path, name)?;
        require_executable(&resolved)?;
        return Ok(resolved);
    }
    let path_var = env::var_os("PATH").ok_or_else(|| format!("PATH is unset; cannot find {name}"))?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            require_executable(&candidate)?;
            return fs::canonicalize(&candidate).map_err(|err| format!("canonicalize {}: {err}", candidate.display()));
        }
    }
    Err(format!("required tool not found on PATH: {name}"))
}

fn require_executable(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|err| format!("stat {}: {err}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(format!("{} is not executable", path.display()));
    }
    Ok(())
}

fn write_cargo_shim(path: &Path, marker: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("{} has no parent", path.display()))?;
    fs::create_dir_all(parent).map_err(|err| format!("create {}: {err}", parent.display()))?;
    let mut file = fs::File::create(path).map_err(|err| format!("create {}: {err}", path.display()))?;
    writeln!(file, "#!/bin/sh").map_err(|err| format!("write {}: {err}", path.display()))?;
    writeln!(file, "printf invoked > '{}'", marker.display())
        .map_err(|err| format!("write {}: {err}", path.display()))?;
    writeln!(file, "exit 99").map_err(|err| format!("write {}: {err}", path.display()))?;
    let mut permissions = fs::metadata(path).map_err(|err| format!("stat {}: {err}", path.display()))?.permissions();
    permissions.set_mode(FAKE_CARGO_PERMISSIONS);
    fs::set_permissions(path, permissions).map_err(|err| format!("chmod {}: {err}", path.display()))?;
    Ok(())
}

fn parse_receipt(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    if bytes.is_empty() {
        return Err(format!("{} is empty", path.display()));
    }
    serde_json::from_slice(&bytes).map_err(|err| format!("parse {}: {err}", path.display()))
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
                .filter(|item| item.get("execution_status").and_then(Value::as_str) != Some("success"))
                .count() as u64
        })
        .unwrap_or_default()
}

fn mantle_binary_from_receipt(receipt: &Value, execution_dir: &Path) -> Result<PathBuf, String> {
    let units = receipt
        .pointer("/topology_execution/unit_executions")
        .and_then(Value::as_array)
        .ok_or("receipt has no topology_execution.unit_executions")?;
    let matches = units
        .iter()
        .filter(|unit| unit.get("target_name").and_then(Value::as_str) == Some(MANTLE_TARGET_NAME))
        .filter(|unit| unit.get("target_kind").and_then(Value::as_str) == Some(MANTLE_TARGET_KIND))
        .filter(|unit| unit.get("execution_status").and_then(Value::as_str) == Some("success"))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!("expected one successful mantle bin unit, found {}", matches.len()));
    }
    let unit_id = matches[0].get("unit_id").and_then(Value::as_str).ok_or("mantle bin unit lacks unit_id")?;
    let path = execution_dir.join(safe_path_component(unit_id)).join(MANTLE_TARGET_NAME);
    require_executable(&path)?;
    Ok(path)
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

fn run_smoke(binary: &Path, stage_dir: &Path) -> Result<i32, String> {
    let output = Command::new(binary)
        .arg(HELP_FLAG)
        .output()
        .map_err(|err| format!("run smoke {}: {err}", binary.display()))?;
    fs::write(stage_dir.join(SMOKE_STDOUT_FILE), &output.stdout)
        .map_err(|err| format!("write smoke stdout for {}: {err}", binary.display()))?;
    fs::write(stage_dir.join(SMOKE_STDERR_FILE), &output.stderr)
        .map_err(|err| format!("write smoke stderr for {}: {err}", binary.display()))?;
    Ok(output.status.code().unwrap_or(1))
}

fn blake3_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    debug_assert_eq!(digest.len(), HEX_DIGEST_LEN);
    Ok(digest)
}

fn write_preflight(
    bundle_dir: &Path,
    root: &Path,
    mantle: &Path,
    rustc: &Path,
    jq: Option<&Path>,
) -> Result<(), String> {
    let mut env_summary = BTreeMap::new();
    for key in ["SNIX_BUILD_SANDBOX_SHELL", "PKG_CONFIG_PATH"] {
        env_summary.insert(key.to_string(), env::var(key).unwrap_or_default());
    }
    let value = json!({
        "schema": SCHEMA,
        "root": root,
        "tools": {
            "host_mantle": mantle,
            "rustc": rustc,
            "jq": jq,
        },
        "environment": env_summary,
    });
    write_json(&bundle_dir.join(PRE_FLIGHT_FILE), &value)
}

fn write_non_claims(bundle_dir: &Path) -> Result<(), String> {
    let text = [
        "This proof claims only a bounded Mantle stage1/stage2 fixed point through native Rust topology execution.",
        "This proof does not claim Mantle bootstrap or release reproducibility.",
        "This proof does not claim source-built compiler/toolchain closure provenance.",
        "This proof does not claim full Cargo compatibility, tests, doctests, examples, or general resolver parity.",
        "This proof still depends on the recorded host rustc/linker/tool environment.",
    ]
    .join("\n");
    fs::write(bundle_dir.join(NON_CLAIMS_FILE), format!("{text}\n")).map_err(|err| format!("write non-claims: {err}"))
}

fn write_meta(
    bundle_dir: &Path,
    root: &Path,
    stage1: &StageResult,
    stage2: Option<&StageResult>,
    status: &str,
) -> Result<(), String> {
    let fixed_point = stage2
        .and_then(|stage| stage.binary_digest.as_ref())
        .zip(stage1.binary_digest.as_ref())
        .map(|(right, left)| left == right)
        .unwrap_or(false);
    let value = json!({
        "schema": SCHEMA,
        "status": status,
        "root": root,
        "fixed_point": fixed_point,
        "stage1": stage_json(stage1),
        "stage2": stage2.map(stage_json),
        "non_claims": [
            "not-crunch-bootstrap",
            "not-release-reproducibility",
            "not-source-built-toolchain-closure",
            "not-full-cargo-compatibility",
        ],
    });
    write_json(&bundle_dir.join(META_FILE), &value)
}

fn stage_json(stage: &StageResult) -> Value {
    json!({
        "name": stage.name,
        "dir": stage.dir,
        "execution_dir": stage.execution_dir,
        "receipt": stage.receipt_path,
        "stderr": stage.stderr_path,
        "status_code": stage.status_code,
        "execution_status": stage.execution_status,
        "cargo_marker_absent": stage.cargo_marker_absent,
        "success": stage.success,
        "unit_count": stage.unit_count,
        "failed_unit_count": stage.failed_unit_count,
        "binary": stage.binary,
        "binary_blake3": stage.binary_digest,
        "smoke_status_code": stage.smoke_status_code,
        "blocker": stage.blocker,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| format!("serialize {}: {err}", path.display()))?;
    fs::write(path, bytes).map_err(|err| format!("write {}: {err}", path.display()))
}

fn status_text(code: Option<i32>) -> String {
    match code {
        Some(code) => format!("{code}\n"),
        None => "signal\n".to_string(),
    }
}
