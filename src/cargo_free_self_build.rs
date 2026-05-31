use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
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
const FIXED_POINT_SCHEMA: &str = "mantle-cargo-free-fixed-point-proof-v1";
const RECEIPT_FILE: &str = "receipt.json";
const STDERR_FILE: &str = "stderr.txt";
const STATUS_FILE: &str = "status.txt";
const META_FILE: &str = "meta.json";
const PRE_FLIGHT_FILE: &str = "preflight.json";
const NON_CLAIMS_FILE: &str = "non-claims.txt";
const SMOKE_STDOUT_FILE: &str = "smoke-stdout.txt";
const SMOKE_STDERR_FILE: &str = "smoke-stderr.txt";
const EXECUTION_DIR: &str = "execution";
const CARGO_SHIM_FILE: &str = "cargo-forbidden";
const CARGO_SHIM_DIR: &str = "cargo-guard-bin";
const CARGO_SHIM_NAME: &str = "cargo";
const CARGO_MARKER_FILE: &str = "cargo-was-invoked";
const C_COMPILER_ALIAS: &str = "cc";
const PKG_CONFIG_ALIAS: &str = "pkg-config";
const PRODUCED_MANTLE_FILE: &str = "mantle";
const MANTLE_TARGET_NAME: &str = "mantle";
const MANTLE_TARGET_KIND: &str = "bin";
const SUCCESS_STATUS: &str = "success";
const BLOCKED_STATUS: &str = "blocked";
const MISMATCH_STATUS: &str = "mismatch";
const STAGE1_DIR: &str = "stage1";
const STAGE2_DIR: &str = "stage2";
const JSON_FLAG: &str = "--json";
const RUST_PLAN_COMMAND: &str = "rust-plan";
const ROOT_FLAG: &str = "--root";
const CARGO_FLAG: &str = "--cargo";
const RUSTC_FLAG: &str = "--rustc";
const NO_CARGO_ORACLE_FLAG: &str = "--no-cargo-oracle";
const EXECUTE_TOPOLOGY_FLAG: &str = "--execute-topology";
const EXECUTION_OUTPUT_ROOT_FLAG: &str = "--execution-output-root";
const HELP_FLAG: &str = "--help";
const TOOLCHAIN_DIR: &str = "toolchain";
const RUSTC_WRAPPER_FILE: &str = "rustc-normalized";
const COMPATIBILITY_FILE: &str = "compatibility.json";
const RUSTC_PROBE_DIR: &str = "rustc-probe";
const RUSTC_PROBE_SOURCE_FILE: &str = "probe.rs";
const RUSTC_PROBE_SOURCE: &str = "fn main() {}\n";
const LINK_SELF_CONTAINED_PROBE_ARG: &str = "link-self-contained=no";
const LINK_SELF_CONTAINED_JOINED_ARG: &str = "-Clink-self-contained=no";
const RUSTC_BOOTSTRAP_ENV: &str = "RUSTC_BOOTSTRAP";
const REAL_RUSTC_ENV: &str = "MANTLE_REAL_RUSTC";
const NORMALIZATION_NONE: &str = "none";
const NORMALIZATION_STRIP_LINK_SELF_CONTAINED: &str = "strip-link-self-contained-no";
const SIGNAL_STATUS_TEXT: &str = "signal";
const BLOCKED_SMOKE_STDOUT: &str = "not run: blocked before binary\n";
const BLOCKED_SMOKE_STDERR_PREFIX: &str = "not run: blocked before binary";
const SOURCE_DIGEST_FIELD: &str = "source_digest";
const SOURCE_DIGEST_ALGORITHM_FIELD: &str = "algorithm";
const RUSTC_SYSROOT_PRINT_ARG: &str = "sysroot";
const SOURCE_DIGEST_VALUE_FIELD: &str = "value";
const SUCCESS_EXIT_CODE: i32 = 0;
const FALLBACK_ERROR_EXIT_CODE: i32 = 1;
const EXPECTED_MANTLE_UNIT_COUNT: usize = 1;
const FIXED_POINT_STAGE1_INDEX: usize = 0;
const FIXED_POINT_STAGE2_INDEX: usize = 1;
const FIXED_POINT_STAGE_COUNT: usize = 2;
#[cfg(unix)]
const CARGO_SHIM_PERMISSIONS: u32 = 0o755;
#[cfg(unix)]
const UNIX_EXECUTE_BITS: u32 = 0o111;

pub(crate) struct CargoFreeSelfBuildOptions<'a> {
    pub(crate) root: &'a Path,
    pub(crate) out_dir: &'a Path,
    pub(crate) rustc: &'a Path,
    pub(crate) toolchain_closure: Option<&'a Path>,
    pub(crate) json: bool,
}

#[derive(Clone, Debug)]
struct LoadedToolchainClosure {
    status: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    manifest_path: Option<PathBuf>,
    manifest: Option<crate::source_toolchain_closure::ToolchainClosureManifest>,
}

#[derive(Clone, Debug)]
struct ExecutionToolchain {
    rustc: PathBuf,
    path_env: OsString,
    status: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FixedPointPlan {
    pub(crate) schema: &'static str,
    pub(crate) root: PathBuf,
    pub(crate) bundle_dir: PathBuf,
    pub(crate) shared_execution_dir: PathBuf,
    pub(crate) preflight_path: PathBuf,
    pub(crate) meta_path: PathBuf,
    pub(crate) non_claims_path: PathBuf,
    pub(crate) stages: [FixedPointStagePlan; FIXED_POINT_STAGE_COUNT],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FixedPointStagePlan {
    pub(crate) name: &'static str,
    pub(crate) stage_dir: PathBuf,
    pub(crate) execution_dir: PathBuf,
    pub(crate) receipt_path: PathBuf,
    pub(crate) stderr_path: PathBuf,
    pub(crate) status_path: PathBuf,
    pub(crate) cargo_marker_path: PathBuf,
    pub(crate) explicit_cargo_shim: PathBuf,
    pub(crate) path_cargo_shim: PathBuf,
    pub(crate) guard_path_dir: PathBuf,
    pub(crate) binary_path: PathBuf,
    pub(crate) smoke_stdout_path: PathBuf,
    pub(crate) smoke_stderr_path: PathBuf,
    pub(crate) command: FixedPointStageCommandPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FixedPointStageCommandPlan {
    pub(crate) mantle_binary: FixedPointMantleBinary,
    pub(crate) args: Vec<OsString>,
    pub(crate) current_dir: PathBuf,
    pub(crate) cargo_env_value: PathBuf,
    pub(crate) path_guard_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FixedPointMantleBinary {
    Host,
    StageOutput { stage_name: &'static str, path: PathBuf },
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
struct RustcCompatibilitySummary {
    requested_rustc: PathBuf,
    stage_rustc: PathBuf,
    normalization: &'static str,
    wrapper: Option<PathBuf>,
    wrapper_blake3: Option<String>,
}

#[derive(Debug)]
struct RustcCompatibility {
    summary: RustcCompatibilitySummary,
}

#[derive(Debug)]
struct FixedPointStageRun {
    name: &'static str,
    dir: PathBuf,
    execution_dir: PathBuf,
    receipt_path: PathBuf,
    stderr_path: PathBuf,
    status_path: PathBuf,
    status_code: Option<i32>,
    execution_status: String,
    cargo_marker_absent: bool,
    success: bool,
    unit_count: u64,
    failed_unit_count: u64,
    binary: Option<PathBuf>,
    binary_blake3: Option<String>,
    smoke_status_code: Option<i32>,
    blocker: Option<String>,
}

#[derive(Debug)]
struct FixedPointStageArtifact {
    binary: PathBuf,
    digest: String,
    smoke_status_code: i32,
}

#[derive(Debug, Serialize)]
struct FixedPointSummary {
    schema: &'static str,
    status: String,
    root: PathBuf,
    bundle_dir: PathBuf,
    fixed_point: bool,
    stage1: FixedPointStageSummary,
    stage2: Option<FixedPointStageSummary>,
    rustc_compatibility: RustcCompatibilitySummary,
    source_built_toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    blocker: Option<String>,
    non_claims: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct FixedPointStageSummary {
    name: &'static str,
    dir: PathBuf,
    execution_dir: PathBuf,
    receipt: PathBuf,
    stderr: PathBuf,
    status: PathBuf,
    status_code: Option<i32>,
    execution_status: String,
    cargo_marker_absent: bool,
    success: bool,
    unit_count: u64,
    failed_unit_count: u64,
    binary: Option<PathBuf>,
    binary_blake3: Option<String>,
    smoke_status_code: Option<i32>,
    blocker: Option<String>,
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
    source_built_toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    non_claims: Vec<&'static str>,
}

pub(crate) fn cmd_cargo_free_self_build(options: CargoFreeSelfBuildOptions<'_>) -> Result<(), RunError> {
    let paths = prepare_paths(options.root, options.out_dir)?;
    prepare_output_dir(&paths)?;
    write_non_claims(&paths.out_dir)?;
    write_cargo_shim(&paths.explicit_cargo_shim, &paths.marker_path)?;
    write_cargo_shim(&paths.path_cargo_shim, &paths.marker_path)?;
    let loaded_toolchain_closure = load_source_built_toolchain_closure(options.toolchain_closure)?;
    let execution_toolchain =
        prepare_execution_toolchain(&paths.guard_path_dir, options.rustc, &loaded_toolchain_closure)?;

    let mut child = run_rust_plan_child(&paths, &execution_toolchain.rustc, &execution_toolchain.path_env)?;
    let produced = if child.blocker.is_none() {
        materialize_or_block(&paths, &mut child)?
    } else {
        None
    };
    if child.blocker.is_some() && produced.is_none() {
        write_blocked_smoke_outputs(&paths, child.blocker.as_deref())?;
    }
    let summary = summarize(&paths, &child, produced.as_ref(), execution_toolchain.status);
    write_summary(&paths.meta_path, &summary)?;
    print_summary(&summary, options.json)?;

    if let Some(blocker) = child.blocker {
        return Err(RunError::Build(format!("Cargo-free self-build blocked: {blocker}")));
    }
    Ok(())
}

pub(crate) fn cmd_cargo_free_fixed_point_self_build(options: CargoFreeSelfBuildOptions<'_>) -> Result<(), RunError> {
    let root = canonicalize_root(options.root)?;
    let bundle_dir = absolutize(&root, options.out_dir);
    ensure_outside_root(&bundle_dir, &root)?;
    prepare_fixed_point_output_dir(&bundle_dir)?;
    let loaded_toolchain_closure = load_source_built_toolchain_closure(options.toolchain_closure)?;
    let compatibility_rustc = prepare_rustc_for_compatibility(options.rustc, &loaded_toolchain_closure)?;
    let compatibility = prepare_rustc_compatibility(&bundle_dir, &compatibility_rustc)?;
    let plan = plan_fixed_point_paths(&root, &bundle_dir, &compatibility.summary.stage_rustc)?;
    let toolchain_status =
        enforce_fixed_point_toolchain(&compatibility.summary.stage_rustc, &loaded_toolchain_closure)?;
    write_fixed_point_non_claims(&plan.bundle_dir)?;
    write_fixed_point_preflight(&plan, &compatibility.summary, &toolchain_status)?;

    let host_mantle = env::current_exe().map_err(|err| internal(format!("resolve current executable: {err}")))?;
    let stage1 =
        execute_fixed_point_stage(&plan.stages[FIXED_POINT_STAGE1_INDEX], &host_mantle, &loaded_toolchain_closure)?;
    if !stage1.success {
        return finish_fixed_point(
            options.json,
            &plan,
            &compatibility.summary,
            &toolchain_status,
            stage1,
            None,
            BLOCKED_STATUS,
        );
    }
    let Some(stage1_binary) = stage1.binary.as_deref() else {
        let stage1 = blocked_fixed_point_stage(stage1, "stage1 succeeded without produced binary path".to_string());
        return finish_fixed_point(
            options.json,
            &plan,
            &compatibility.summary,
            &toolchain_status,
            stage1,
            None,
            BLOCKED_STATUS,
        );
    };
    let stage2 =
        execute_fixed_point_stage(&plan.stages[FIXED_POINT_STAGE2_INDEX], stage1_binary, &loaded_toolchain_closure)?;
    if !stage2.success {
        return finish_fixed_point(
            options.json,
            &plan,
            &compatibility.summary,
            &toolchain_status,
            stage1,
            Some(stage2),
            BLOCKED_STATUS,
        );
    }
    let status = fixed_point_status(&stage1, &stage2)?;
    finish_fixed_point(options.json, &plan, &compatibility.summary, &toolchain_status, stage1, Some(stage2), status)
}

fn prepare_paths(root: &Path, out_dir: &Path) -> Result<BuildPaths, RunError> {
    let root = canonicalize_root(root)?;
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

fn canonicalize_root(root: &Path) -> Result<PathBuf, RunError> {
    fs::canonicalize(root).map_err(|err| internal(format!("canonicalize root {}: {err}", root.display())))
}

pub(crate) fn plan_fixed_point_paths(root: &Path, out_dir: &Path, rustc: &Path) -> Result<FixedPointPlan, RunError> {
    if !root.is_absolute() {
        return Err(RunError::Build(format!(
            "fixed-point planner requires an absolute source root, got {}",
            root.display()
        )));
    }
    if rustc.as_os_str().is_empty() {
        return Err(RunError::Build("fixed-point planner requires a non-empty rustc path".to_string()));
    }
    let bundle_dir = absolutize(root, out_dir);
    ensure_outside_root(&bundle_dir, root)?;
    let shared_execution_dir = bundle_dir.join(EXECUTION_DIR);
    let stage1_binary_path = bundle_dir.join(STAGE1_DIR).join(PRODUCED_MANTLE_FILE);
    let stages = [
        fixed_point_stage_plan(
            STAGE1_DIR,
            root,
            &bundle_dir,
            &shared_execution_dir,
            rustc,
            FixedPointMantleBinary::Host,
        ),
        fixed_point_stage_plan(
            STAGE2_DIR,
            root,
            &bundle_dir,
            &shared_execution_dir,
            rustc,
            FixedPointMantleBinary::StageOutput {
                stage_name: STAGE1_DIR,
                path: stage1_binary_path,
            },
        ),
    ];
    debug_assert_eq!(stages.len(), FIXED_POINT_STAGE_COUNT);
    debug_assert_eq!(stages[FIXED_POINT_STAGE1_INDEX].name, STAGE1_DIR);
    debug_assert_eq!(stages[FIXED_POINT_STAGE2_INDEX].name, STAGE2_DIR);
    Ok(FixedPointPlan {
        schema: FIXED_POINT_SCHEMA,
        root: root.to_path_buf(),
        bundle_dir: bundle_dir.clone(),
        shared_execution_dir,
        preflight_path: bundle_dir.join(PRE_FLIGHT_FILE),
        meta_path: bundle_dir.join(META_FILE),
        non_claims_path: bundle_dir.join(NON_CLAIMS_FILE),
        stages,
    })
}

fn fixed_point_stage_plan(
    name: &'static str,
    root: &Path,
    bundle_dir: &Path,
    execution_dir: &Path,
    rustc: &Path,
    mantle_binary: FixedPointMantleBinary,
) -> FixedPointStagePlan {
    debug_assert!(!name.is_empty());
    debug_assert!(root.is_absolute());
    debug_assert!(bundle_dir.is_absolute());
    let stage_dir = bundle_dir.join(name);
    let explicit_cargo_shim = stage_dir.join(CARGO_SHIM_FILE);
    let guard_path_dir = stage_dir.join(CARGO_SHIM_DIR);
    let path_cargo_shim = guard_path_dir.join(CARGO_SHIM_NAME);
    let command = FixedPointStageCommandPlan {
        mantle_binary,
        args: rust_plan_args(root, &explicit_cargo_shim, rustc, execution_dir),
        current_dir: root.to_path_buf(),
        cargo_env_value: path_cargo_shim.clone(),
        path_guard_dir: guard_path_dir.clone(),
    };
    FixedPointStagePlan {
        name,
        stage_dir: stage_dir.clone(),
        execution_dir: execution_dir.to_path_buf(),
        receipt_path: stage_dir.join(RECEIPT_FILE),
        stderr_path: stage_dir.join(STDERR_FILE),
        status_path: stage_dir.join(STATUS_FILE),
        cargo_marker_path: stage_dir.join(CARGO_MARKER_FILE),
        explicit_cargo_shim,
        path_cargo_shim,
        guard_path_dir,
        binary_path: stage_dir.join(PRODUCED_MANTLE_FILE),
        smoke_stdout_path: stage_dir.join(SMOKE_STDOUT_FILE),
        smoke_stderr_path: stage_dir.join(SMOKE_STDERR_FILE),
        command,
    }
}

fn rust_plan_args(root: &Path, cargo_shim: &Path, rustc: &Path, execution_dir: &Path) -> Vec<OsString> {
    let args = [
        OsString::from(JSON_FLAG),
        OsString::from(RUST_PLAN_COMMAND),
        OsString::from(ROOT_FLAG),
        root.as_os_str().to_os_string(),
        OsString::from(CARGO_FLAG),
        cargo_shim.as_os_str().to_os_string(),
        OsString::from(RUSTC_FLAG),
        rustc.as_os_str().to_os_string(),
        OsString::from(NO_CARGO_ORACLE_FLAG),
        OsString::from(EXECUTE_TOPOLOGY_FLAG),
        OsString::from(EXECUTION_OUTPUT_ROOT_FLAG),
        execution_dir.as_os_str().to_os_string(),
    ];
    debug_assert_eq!(args.first(), Some(&OsString::from(JSON_FLAG)));
    debug_assert_eq!(args.last(), Some(&execution_dir.as_os_str().to_os_string()));
    args.into_iter().collect()
}

fn prepare_fixed_point_output_dir(bundle_dir: &Path) -> Result<(), RunError> {
    fs::create_dir_all(bundle_dir).map_err(|err| internal(format!("create {}: {err}", bundle_dir.display())))?;
    let owned_paths = [
        bundle_dir.join(STAGE1_DIR),
        bundle_dir.join(STAGE2_DIR),
        bundle_dir.join(EXECUTION_DIR),
        bundle_dir.join(TOOLCHAIN_DIR),
        bundle_dir.join(PRE_FLIGHT_FILE),
        bundle_dir.join(META_FILE),
        bundle_dir.join(NON_CLAIMS_FILE),
    ];
    for path in &owned_paths {
        remove_owned_path(path)?;
    }
    fs::create_dir_all(bundle_dir).map_err(|err| internal(format!("create {}: {err}", bundle_dir.display())))
}

fn prepare_rustc_compatibility(bundle_dir: &Path, requested: &Path) -> Result<RustcCompatibility, RunError> {
    let requested_rustc = resolve_executable(requested, "rustc")?;
    let toolchain_dir = bundle_dir.join(TOOLCHAIN_DIR);
    fs::create_dir_all(&toolchain_dir)
        .map_err(|err| internal(format!("create toolchain dir {}: {err}", toolchain_dir.display())))?;
    if rustc_accepts_link_self_contained_no(&requested_rustc, &toolchain_dir) {
        let summary = RustcCompatibilitySummary {
            requested_rustc: requested_rustc.clone(),
            stage_rustc: requested_rustc,
            normalization: NORMALIZATION_NONE,
            wrapper: None,
            wrapper_blake3: None,
        };
        write_rustc_compatibility(&toolchain_dir.join(COMPATIBILITY_FILE), &summary)?;
        return Ok(RustcCompatibility { summary });
    }
    let wrapper = toolchain_dir.join(RUSTC_WRAPPER_FILE);
    write_rustc_wrapper(&wrapper, &requested_rustc)?;
    let wrapper_blake3 = blake3_file(&wrapper)?;
    let summary = RustcCompatibilitySummary {
        requested_rustc,
        stage_rustc: wrapper.clone(),
        normalization: NORMALIZATION_STRIP_LINK_SELF_CONTAINED,
        wrapper: Some(wrapper),
        wrapper_blake3: Some(wrapper_blake3),
    };
    write_rustc_compatibility(&toolchain_dir.join(COMPATIBILITY_FILE), &summary)?;
    Ok(RustcCompatibility { summary })
}

fn resolve_executable(path: &Path, name: &str) -> Result<PathBuf, RunError> {
    debug_assert!(!name.is_empty());
    if path.components().count() == 1 && !path.is_absolute() {
        return resolve_executable_on_path(path, name);
    }
    let resolved =
        fs::canonicalize(path).map_err(|err| internal(format!("canonicalize {name} {}: {err}", path.display())))?;
    require_executable(&resolved)?;
    Ok(resolved)
}

fn resolve_executable_on_path(path: &Path, name: &str) -> Result<PathBuf, RunError> {
    let path_var = env::var_os("PATH").ok_or_else(|| internal(format!("PATH is unset; cannot find {name}")))?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(path);
        if candidate.is_file() {
            let resolved = fs::canonicalize(&candidate)
                .map_err(|err| internal(format!("canonicalize {}: {err}", candidate.display())))?;
            require_executable(&resolved)?;
            return Ok(resolved);
        }
    }
    Err(internal(format!("required tool not found on PATH: {name}")))
}

fn rustc_accepts_link_self_contained_no(rustc: &Path, toolchain_dir: &Path) -> bool {
    let probe_dir = toolchain_dir.join(RUSTC_PROBE_DIR);
    let probe_source = probe_dir.join(RUSTC_PROBE_SOURCE_FILE);
    if remove_owned_path(&probe_dir).is_err() {
        return false;
    }
    if fs::create_dir_all(&probe_dir).is_err() {
        return false;
    }
    if fs::write(&probe_source, RUSTC_PROBE_SOURCE).is_err() {
        let _ = remove_owned_path(&probe_dir);
        return false;
    }
    let success = Command::new(rustc)
        .arg("--crate-type")
        .arg("bin")
        .arg("-C")
        .arg(LINK_SELF_CONTAINED_PROBE_ARG)
        .arg(&probe_source)
        .arg("--out-dir")
        .arg(&probe_dir)
        .output()
        .is_ok_and(|output| output.status.success());
    let _ = remove_owned_path(&probe_dir);
    success
}

fn write_rustc_wrapper(wrapper: &Path, real_rustc: &Path) -> Result<(), RunError> {
    let script = rustc_wrapper_script(real_rustc);
    write_text(wrapper, &script)?;
    set_executable(wrapper)
}

fn rustc_wrapper_script(real_rustc: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nexport {RUSTC_BOOTSTRAP_ENV}=1\nexport {REAL_RUSTC_ENV}={}\nargs=()\nwhile (($#)); do\n  arg=\"$1\"\n  shift\n  if [[ \"$arg\" == \"-C\" && \"${{1-}}\" == \"{LINK_SELF_CONTAINED_PROBE_ARG}\" ]]; then\n    shift\n    continue\n  fi\n  if [[ \"$arg\" == \"{LINK_SELF_CONTAINED_JOINED_ARG}\" ]]; then\n    continue\n  fi\n  args+=(\"$arg\")\ndone\nexec \"${REAL_RUSTC_ENV}\" \"${{args[@]}}\"\n",
        shell_quote(real_rustc)
    )
}

fn write_rustc_compatibility(path: &Path, summary: &RustcCompatibilitySummary) -> Result<(), RunError> {
    let value = json!(summary);
    let bytes = serde_json::to_vec_pretty(&value).map_err(|err| internal(format!("serialize compatibility: {err}")))?;
    write_bytes(path, &bytes)
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

fn run_rust_plan_child(paths: &BuildPaths, rustc: &Path, path_env: &OsStr) -> Result<ChildRun, RunError> {
    let current_exe = env::current_exe().map_err(|err| internal(format!("resolve current executable: {err}")))?;
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
        .env("PATH", path_env)
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

fn execute_fixed_point_stage(
    stage: &FixedPointStagePlan,
    mantle_bin: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<FixedPointStageRun, RunError> {
    prepare_fixed_point_stage(stage)?;
    let path_env = execution_path_env(&stage.guard_path_dir, toolchain_closure)?;
    let output = Command::new(mantle_bin)
        .args(&stage.command.args)
        .current_dir(&stage.command.current_dir)
        .env("CARGO", &stage.command.cargo_env_value)
        .env("PATH", path_env)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(err) => return blocked_fixed_point_launch(stage, mantle_bin, err),
    };
    let mut blocker = None;
    record_file_write(&stage.receipt_path, &output.stdout, &mut blocker);
    record_file_write(&stage.stderr_path, &output.stderr, &mut blocker);
    record_file_write(&stage.status_path, status_text(output.status.code()).as_bytes(), &mut blocker);
    fixed_point_stage_from_output(stage, output.status.code(), blocker)
}

fn prepare_fixed_point_stage(stage: &FixedPointStagePlan) -> Result<(), RunError> {
    fs::create_dir_all(&stage.stage_dir)
        .map_err(|err| internal(format!("create stage dir {}: {err}", stage.stage_dir.display())))?;
    remove_owned_path(&stage.execution_dir)?;
    fs::create_dir_all(&stage.execution_dir)
        .map_err(|err| internal(format!("create execution dir {}: {err}", stage.execution_dir.display())))?;
    write_cargo_shim(&stage.explicit_cargo_shim, &stage.cargo_marker_path)?;
    write_cargo_shim(&stage.path_cargo_shim, &stage.cargo_marker_path)
}

fn blocked_fixed_point_launch(
    stage: &FixedPointStagePlan,
    mantle_bin: &Path,
    err: std::io::Error,
) -> Result<FixedPointStageRun, RunError> {
    let blocker = format!("launch {} for {}: {err}", mantle_bin.display(), stage.name);
    write_text(&stage.stderr_path, &format!("{blocker}\n"))?;
    write_text(&stage.status_path, "launch-failed\n")?;
    write_blocked_fixed_point_smoke_outputs(stage, &blocker)?;
    Ok(blocked_fixed_point_stage_from_plan(stage, "launch-failed", None, blocker))
}

fn fixed_point_stage_from_output(
    stage: &FixedPointStagePlan,
    status_code: Option<i32>,
    mut blocker: Option<String>,
) -> Result<FixedPointStageRun, RunError> {
    let receipt = parse_receipt(&stage.receipt_path, status_code == Some(SUCCESS_EXIT_CODE))?;
    let execution_status = receipt_execution_status(receipt.as_ref());
    let cargo_marker_absent = !stage.cargo_marker_path.exists();
    if blocker.is_none() {
        blocker = child_blocker(status_code, &execution_status, cargo_marker_absent);
    }
    let produced = if blocker.is_none() {
        materialize_fixed_point_stage_artifact(stage, receipt.as_ref())?
    } else {
        None
    };
    fixed_point_stage_with_artifact(
        stage,
        status_code,
        receipt.as_ref(),
        execution_status,
        cargo_marker_absent,
        blocker,
        produced,
    )
}

fn fixed_point_stage_with_artifact(
    stage: &FixedPointStagePlan,
    status_code: Option<i32>,
    receipt: Option<&Value>,
    execution_status: String,
    cargo_marker_absent: bool,
    mut blocker: Option<String>,
    produced: Option<FixedPointStageArtifact>,
) -> Result<FixedPointStageRun, RunError> {
    if let Some(produced) = produced.as_ref() {
        if produced.smoke_status_code != SUCCESS_EXIT_CODE {
            blocker = Some(format!("smoke check exited with {}", produced.smoke_status_code));
        }
    }
    if blocker.is_some() && produced.is_none() {
        write_blocked_fixed_point_smoke_outputs(stage, blocker.as_deref().unwrap_or("blocked before binary"))?;
    }
    Ok(FixedPointStageRun {
        name: stage.name,
        dir: stage.stage_dir.clone(),
        execution_dir: stage.execution_dir.clone(),
        receipt_path: stage.receipt_path.clone(),
        stderr_path: stage.stderr_path.clone(),
        status_path: stage.status_path.clone(),
        status_code,
        execution_status,
        cargo_marker_absent,
        success: blocker.is_none(),
        unit_count: receipt.map(unit_count).unwrap_or_default(),
        failed_unit_count: receipt.map(failed_unit_count).unwrap_or_default(),
        binary: produced.as_ref().map(|value| value.binary.clone()),
        binary_blake3: produced.as_ref().map(|value| value.digest.clone()),
        smoke_status_code: produced.as_ref().map(|value| value.smoke_status_code),
        blocker,
    })
}

fn materialize_fixed_point_stage_artifact(
    stage: &FixedPointStagePlan,
    receipt: Option<&Value>,
) -> Result<Option<FixedPointStageArtifact>, RunError> {
    let Some(receipt) = receipt else {
        return Ok(None);
    };
    let (unit_id, _) = mantle_unit_from_receipt(receipt)?;
    let source_binary = stage.execution_dir.join(safe_path_component(&unit_id)).join(MANTLE_TARGET_NAME);
    require_executable(&source_binary)?;
    fs::copy(&source_binary, &stage.binary_path).map_err(|err| {
        internal(format!(
            "copy produced binary {} to {}: {err}",
            source_binary.display(),
            stage.binary_path.display()
        ))
    })?;
    require_executable(&stage.binary_path)?;
    let digest = blake3_file(&stage.binary_path)?;
    let smoke_status_code = run_smoke_to_paths(&stage.binary_path, &stage.smoke_stdout_path, &stage.smoke_stderr_path)?;
    Ok(Some(FixedPointStageArtifact {
        binary: stage.binary_path.clone(),
        digest,
        smoke_status_code,
    }))
}

fn blocked_fixed_point_stage(mut stage: FixedPointStageRun, blocker: String) -> FixedPointStageRun {
    stage.success = false;
    stage.blocker = Some(blocker);
    stage
}

fn blocked_fixed_point_stage_from_plan(
    stage: &FixedPointStagePlan,
    execution_status: &str,
    status_code: Option<i32>,
    blocker: String,
) -> FixedPointStageRun {
    FixedPointStageRun {
        name: stage.name,
        dir: stage.stage_dir.clone(),
        execution_dir: stage.execution_dir.clone(),
        receipt_path: stage.receipt_path.clone(),
        stderr_path: stage.stderr_path.clone(),
        status_path: stage.status_path.clone(),
        status_code,
        execution_status: execution_status.to_string(),
        cargo_marker_absent: true,
        success: false,
        unit_count: 0,
        failed_unit_count: 0,
        binary: None,
        binary_blake3: None,
        smoke_status_code: None,
        blocker: Some(blocker),
    }
}

fn fixed_point_status(stage1: &FixedPointStageRun, stage2: &FixedPointStageRun) -> Result<&'static str, RunError> {
    let stage1_digest = stage1
        .binary_blake3
        .as_deref()
        .ok_or_else(|| internal("stage1 succeeded without binary digest".to_string()))?;
    let stage2_digest = stage2
        .binary_blake3
        .as_deref()
        .ok_or_else(|| internal("stage2 succeeded without binary digest".to_string()))?;
    if stage1_digest == stage2_digest {
        Ok(SUCCESS_STATUS)
    } else {
        Ok(MISMATCH_STATUS)
    }
}

fn finish_fixed_point(
    json_mode: bool,
    plan: &FixedPointPlan,
    compatibility: &RustcCompatibilitySummary,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    stage1: FixedPointStageRun,
    stage2: Option<FixedPointStageRun>,
    status: &str,
) -> Result<(), RunError> {
    let summary = fixed_point_summary(plan, compatibility, toolchain_closure, &stage1, stage2.as_ref(), status);
    write_summary(&plan.meta_path, &summary)?;
    print_fixed_point_summary(&summary, json_mode)?;
    if status == SUCCESS_STATUS {
        return Ok(());
    }
    Err(RunError::Build(fixed_point_error_message(&summary)))
}

fn fixed_point_summary(
    plan: &FixedPointPlan,
    compatibility: &RustcCompatibilitySummary,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    stage1: &FixedPointStageRun,
    stage2: Option<&FixedPointStageRun>,
    status: &str,
) -> FixedPointSummary {
    FixedPointSummary {
        schema: FIXED_POINT_SCHEMA,
        status: status.to_string(),
        root: plan.root.clone(),
        bundle_dir: plan.bundle_dir.clone(),
        fixed_point: status == SUCCESS_STATUS,
        stage1: stage_summary(stage1),
        stage2: stage2.map(stage_summary),
        rustc_compatibility: RustcCompatibilitySummary {
            requested_rustc: compatibility.requested_rustc.clone(),
            stage_rustc: compatibility.stage_rustc.clone(),
            normalization: compatibility.normalization,
            wrapper: compatibility.wrapper.clone(),
            wrapper_blake3: compatibility.wrapper_blake3.clone(),
        },
        source_built_toolchain_closure: toolchain_closure.clone(),
        blocker: fixed_point_blocker(stage1, stage2, status),
        non_claims: fixed_point_non_claims(),
    }
}

fn stage_summary(stage: &FixedPointStageRun) -> FixedPointStageSummary {
    FixedPointStageSummary {
        name: stage.name,
        dir: stage.dir.clone(),
        execution_dir: stage.execution_dir.clone(),
        receipt: stage.receipt_path.clone(),
        stderr: stage.stderr_path.clone(),
        status: stage.status_path.clone(),
        status_code: stage.status_code,
        execution_status: stage.execution_status.clone(),
        cargo_marker_absent: stage.cargo_marker_absent,
        success: stage.success,
        unit_count: stage.unit_count,
        failed_unit_count: stage.failed_unit_count,
        binary: stage.binary.clone(),
        binary_blake3: stage.binary_blake3.clone(),
        smoke_status_code: stage.smoke_status_code,
        blocker: stage.blocker.clone(),
    }
}

fn fixed_point_blocker(
    stage1: &FixedPointStageRun,
    stage2: Option<&FixedPointStageRun>,
    status: &str,
) -> Option<String> {
    if let Some(blocker) = &stage1.blocker {
        return Some(format!("stage1 blocked: {blocker}"));
    }
    if let Some(blocker) = stage2.and_then(|stage| stage.blocker.as_ref()) {
        return Some(format!("stage2 blocked: {blocker}"));
    }
    if status == MISMATCH_STATUS {
        return Some("stage1/stage2 Mantle binary digests differ".to_string());
    }
    None
}

fn fixed_point_error_message(summary: &FixedPointSummary) -> String {
    summary
        .blocker
        .clone()
        .unwrap_or_else(|| format!("Cargo-free fixed-point proof ended with status {}", summary.status))
}

fn print_fixed_point_summary(summary: &FixedPointSummary, json_mode: bool) -> Result<(), RunError> {
    if json_mode {
        let rendered = serde_json::to_string(summary).map_err(|err| internal(format!("render summary: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("Cargo-free fixed-point: {}", summary.status);
    println!("bundle: {}", summary.bundle_dir.display());
    if let Some(digest) = &summary.stage1.binary_blake3 {
        println!("stage1_binary_blake3: {digest}");
    }
    if let Some(stage2) = &summary.stage2 {
        if let Some(digest) = &stage2.binary_blake3 {
            println!("stage2_binary_blake3: {digest}");
        }
    }
    Ok(())
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

fn summarize(
    paths: &BuildPaths,
    child: &ChildRun,
    produced: Option<&ProducedBinary>,
    toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> SelfBuildSummary {
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
        source_built_toolchain_closure: toolchain_closure,
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

fn record_file_write(path: &Path, bytes: &[u8], blocker: &mut Option<String>) {
    if blocker.is_some() {
        let _ = fs::write(path, bytes);
        return;
    }
    if let Err(err) = fs::write(path, bytes) {
        *blocker = Some(format!("write {}: {err}", path.display()));
    }
}

fn receipt_execution_status(receipt: Option<&Value>) -> String {
    receipt
        .and_then(|value| value.pointer("/topology_execution/execution_status"))
        .and_then(Value::as_str)
        .unwrap_or("missing")
        .to_string()
}

fn run_smoke(binary: &Path, out_dir: &Path) -> Result<i32, RunError> {
    run_smoke_to_paths(binary, &out_dir.join(SMOKE_STDOUT_FILE), &out_dir.join(SMOKE_STDERR_FILE))
}

fn run_smoke_to_paths(binary: &Path, stdout_path: &Path, stderr_path: &Path) -> Result<i32, RunError> {
    let output = Command::new(binary)
        .arg(HELP_FLAG)
        .output()
        .map_err(|err| internal(format!("run smoke {}: {err}", binary.display())))?;
    write_bytes(stdout_path, &output.stdout)?;
    write_bytes(stderr_path, &output.stderr)?;
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

fn write_blocked_fixed_point_smoke_outputs(stage: &FixedPointStagePlan, blocker: &str) -> Result<(), RunError> {
    write_text(&stage.smoke_stdout_path, BLOCKED_SMOKE_STDOUT)?;
    write_text(&stage.smoke_stderr_path, &format!("{BLOCKED_SMOKE_STDERR_PREFIX}: {blocker}\n"))
}

fn prepare_execution_toolchain(
    guard_path_dir: &Path,
    requested_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<ExecutionToolchain, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(ExecutionToolchain {
            rustc: requested_rustc.to_path_buf(),
            path_env: guarded_path(guard_path_dir)?,
            status: toolchain_closure.status.clone(),
        });
    };
    let rustc = resolve_executable(requested_rustc, "rustc")?;
    let status = enforce_receipt_bound_toolchain(&rustc, toolchain_closure, manifest)?;
    Ok(ExecutionToolchain {
        rustc,
        path_env: execution_path_env(guard_path_dir, toolchain_closure)?,
        status,
    })
}

fn prepare_rustc_for_compatibility(
    requested_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<PathBuf, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(requested_rustc.to_path_buf());
    };
    let rustc = resolve_executable(requested_rustc, "rustc")?;
    enforce_observed_toolchain_subset(manifest, &[observed_file_tool(
        crate::source_toolchain_closure::ToolchainRole::Rustc,
        &rustc,
    )?])?;
    Ok(rustc)
}

fn enforce_fixed_point_toolchain(
    stage_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(toolchain_closure.status.clone());
    };
    enforce_receipt_bound_toolchain(stage_rustc, toolchain_closure, manifest)
}

fn enforce_receipt_bound_toolchain(
    rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus, RunError> {
    let manifest_path = toolchain_closure
        .manifest_path
        .clone()
        .ok_or_else(|| internal("toolchain closure manifest path missing during enforcement".to_string()))?;
    let observed = observed_toolchain_inputs(rustc, manifest)?;
    let validation = enforce_observed_toolchain_subset(manifest, &observed)?;
    Ok(crate::source_toolchain_closure::enforced_source_built_toolchain_closure(manifest_path, &validation))
}

fn enforce_observed_toolchain_subset(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    observed: &[crate::source_toolchain_closure::ToolchainObservedInput],
) -> Result<crate::source_toolchain_closure::ToolchainClosureValidation, RunError> {
    crate::source_toolchain_closure::enforce_observed_toolchain_inputs(manifest, observed)
        .map_err(|err| RunError::Build(format!("source-built toolchain closure blocked: {}", err.message())))
}

fn observed_toolchain_inputs(
    rustc: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<Vec<crate::source_toolchain_closure::ToolchainObservedInput>, RunError> {
    use crate::source_toolchain_closure::ToolchainRole;
    let mut observed = Vec::new();
    observed.push(observed_file_tool(ToolchainRole::Rustc, rustc)?);
    observed.push(observed_sysroot_tool(rustc)?);
    for role in [ToolchainRole::Linker, ToolchainRole::CCompiler] {
        let member = single_member_for_role(manifest, role)?;
        observed.push(observed_member_tool(member)?);
    }
    for member in optional_tool_members(manifest) {
        observed.push(observed_member_tool(member)?);
    }
    Ok(observed)
}

fn observed_file_tool(
    role: crate::source_toolchain_closure::ToolchainRole,
    path: &Path,
) -> Result<crate::source_toolchain_closure::ToolchainObservedInput, RunError> {
    let path = canonicalize_toolchain_path(path, role)?;
    require_executable(&path)?;
    Ok(crate::source_toolchain_closure::ToolchainObservedInput {
        role,
        execution_path: path_to_string(&path)?,
        content_digest_blake3: Some(blake3_file(&path)?),
    })
}

fn observed_member_tool(
    member: &crate::source_toolchain_closure::ToolchainClosureMember,
) -> Result<crate::source_toolchain_closure::ToolchainObservedInput, RunError> {
    observed_file_tool(member.role, Path::new(&member.execution_path))
}

fn observed_sysroot_tool(rustc: &Path) -> Result<crate::source_toolchain_closure::ToolchainObservedInput, RunError> {
    let sysroot = rustc_reported_sysroot(rustc)?;
    Ok(crate::source_toolchain_closure::ToolchainObservedInput {
        role: crate::source_toolchain_closure::ToolchainRole::Sysroot,
        execution_path: path_to_string(&sysroot)?,
        content_digest_blake3: None,
    })
}

fn rustc_reported_sysroot(rustc: &Path) -> Result<PathBuf, RunError> {
    let output = Command::new(rustc).arg("--print").arg(RUSTC_SYSROOT_PRINT_ARG).output().map_err(|err| {
        RunError::Build(format!("source-built toolchain closure blocked: rustc --print sysroot failed: {err}"))
    })?;
    if !output.status.success() {
        return Err(RunError::Build(format!(
            "source-built toolchain closure blocked: rustc --print sysroot exited with {}",
            status_text(output.status.code()).trim_end()
        )));
    }
    let text = String::from_utf8(output.stdout).map_err(|err| {
        RunError::Build(format!("source-built toolchain closure blocked: rustc sysroot was not UTF-8: {err}"))
    })?;
    let sysroot = PathBuf::from(text.trim());
    fs::canonicalize(&sysroot).map_err(|err| {
        RunError::Build(format!(
            "source-built toolchain closure blocked: canonicalize rustc sysroot {}: {err}",
            sysroot.display()
        ))
    })
}

fn single_member_for_role(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    role: crate::source_toolchain_closure::ToolchainRole,
) -> Result<&crate::source_toolchain_closure::ToolchainClosureMember, RunError> {
    let members = manifest.members.iter().filter(|member| member.role == role).collect::<Vec<_>>();
    match members.as_slice() {
        [member] => Ok(member),
        [] => Err(RunError::Build(format!("source-built toolchain closure blocked: missing {role:?} member"))),
        _ => Err(RunError::Build(format!("source-built toolchain closure blocked: multiple {role:?} members"))),
    }
}

fn optional_tool_members(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Vec<&crate::source_toolchain_closure::ToolchainClosureMember> {
    use crate::source_toolchain_closure::ToolchainRole;
    manifest
        .members
        .iter()
        .filter(|member| matches!(member.role, ToolchainRole::PkgConfig | ToolchainRole::NativeHelper))
        .collect()
}

fn execution_path_env(cargo_path_dir: &Path, toolchain_closure: &LoadedToolchainClosure) -> Result<OsString, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return guarded_path(cargo_path_dir);
    };
    write_toolchain_path_aliases(cargo_path_dir, manifest)?;
    env::join_paths([cargo_path_dir]).map_err(|err| internal(format!("construct receipt-bound PATH: {err}")))
}

fn write_toolchain_path_aliases(
    guard_path_dir: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<(), RunError> {
    let aliases = toolchain_path_aliases(manifest)?;
    for (alias, target) in aliases {
        let link = guard_path_dir.join(alias);
        if link.file_name() == Some(OsStr::new(CARGO_SHIM_NAME)) {
            return Err(RunError::Build("source-built toolchain closure blocked: Cargo must stay guarded".to_string()));
        }
        remove_owned_path(&link)?;
        symlink_toolchain_alias(&target, &link)?;
    }
    Ok(())
}

fn toolchain_path_aliases(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<BTreeMap<String, PathBuf>, RunError> {
    let mut aliases = BTreeMap::new();
    for member in executable_path_members(manifest) {
        let target = canonicalize_toolchain_path(Path::new(&member.execution_path), member.role)?;
        require_executable(&target)?;
        add_toolchain_alias(&mut aliases, path_file_name(&target)?, &target)?;
        add_role_aliases(&mut aliases, member.role, &target)?;
    }
    Ok(aliases)
}

fn executable_path_members(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Vec<&crate::source_toolchain_closure::ToolchainClosureMember> {
    use crate::source_toolchain_closure::ToolchainRole;
    manifest
        .members
        .iter()
        .filter(|member| {
            matches!(
                member.role,
                ToolchainRole::Rustc
                    | ToolchainRole::Linker
                    | ToolchainRole::CCompiler
                    | ToolchainRole::CxxCompiler
                    | ToolchainRole::PkgConfig
                    | ToolchainRole::NativeHelper
            )
        })
        .collect()
}

fn add_role_aliases(
    aliases: &mut BTreeMap<String, PathBuf>,
    role: crate::source_toolchain_closure::ToolchainRole,
    target: &Path,
) -> Result<(), RunError> {
    use crate::source_toolchain_closure::ToolchainRole;
    match role {
        ToolchainRole::CCompiler => add_toolchain_alias(aliases, C_COMPILER_ALIAS.to_string(), target),
        ToolchainRole::PkgConfig => add_toolchain_alias(aliases, PKG_CONFIG_ALIAS.to_string(), target),
        _ => Ok(()),
    }
}

fn add_toolchain_alias(aliases: &mut BTreeMap<String, PathBuf>, alias: String, target: &Path) -> Result<(), RunError> {
    if let Some(existing) = aliases.get(&alias) {
        if existing != target {
            return Err(RunError::Build(format!(
                "source-built toolchain closure blocked: PATH alias {alias} has conflicting targets"
            )));
        }
        return Ok(());
    }
    aliases.insert(alias, target.to_path_buf());
    Ok(())
}

fn canonicalize_toolchain_path(
    path: &Path,
    role: crate::source_toolchain_closure::ToolchainRole,
) -> Result<PathBuf, RunError> {
    fs::canonicalize(path).map_err(|err| {
        RunError::Build(format!(
            "source-built toolchain closure blocked: canonicalize {role:?} {}: {err}",
            path.display()
        ))
    })
}

fn path_file_name(path: &Path) -> Result<String, RunError> {
    let name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
        RunError::Build(format!("source-built toolchain closure blocked: invalid tool path {}", path.display()))
    })?;
    Ok(name.to_string())
}

fn path_to_string(path: &Path) -> Result<String, RunError> {
    path.to_str().map(ToOwned::to_owned).ok_or_else(|| {
        RunError::Build(format!("source-built toolchain closure blocked: non-UTF-8 path {}", path.display()))
    })
}

#[cfg(unix)]
fn symlink_toolchain_alias(target: &Path, link: &Path) -> Result<(), RunError> {
    std::os::unix::fs::symlink(target, link)
        .map_err(|err| internal(format!("symlink {} -> {}: {err}", link.display(), target.display())))
}

#[cfg(not(unix))]
fn symlink_toolchain_alias(target: &Path, link: &Path) -> Result<(), RunError> {
    fs::copy(target, link)
        .map_err(|err| internal(format!("copy {} -> {}: {err}", target.display(), link.display())))?;
    set_executable(link)
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

fn write_fixed_point_non_claims(bundle_dir: &Path) -> Result<(), RunError> {
    let text = fixed_point_non_claims_text().join("\n");
    write_text(&bundle_dir.join(NON_CLAIMS_FILE), &format!("{text}\n"))
}

fn fixed_point_non_claims_text() -> Vec<&'static str> {
    vec![
        "This proof claims only a bounded Mantle stage1/stage2 fixed point through native Rust topology execution.",
        "This proof does not claim Crunch bootstrap or release reproducibility.",
        "This proof does not claim source-built compiler/toolchain closure provenance.",
        "This proof does not claim full Cargo compatibility, tests, doctests, examples, or general resolver parity.",
        "This proof still depends on the recorded host rustc/linker/tool environment.",
    ]
}

fn fixed_point_non_claims() -> Vec<&'static str> {
    vec![
        "not-crunch-bootstrap",
        "not-release-reproducibility",
        "not-source-built-toolchain-closure",
        "not-full-cargo-compatibility",
    ]
}

fn load_source_built_toolchain_closure(manifest_path: Option<&Path>) -> Result<LoadedToolchainClosure, RunError> {
    let Some(manifest_path) = manifest_path else {
        return Ok(LoadedToolchainClosure {
            status: crate::source_toolchain_closure::absent_source_built_toolchain_closure(),
            manifest_path: None,
            manifest: None,
        });
    };
    let manifest_bytes = fs::read(manifest_path)
        .map_err(|err| RunError::Build(format!("read --toolchain-closure {}: {err}", manifest_path.display())))?;
    let manifest = serde_json::from_slice::<crate::source_toolchain_closure::ToolchainClosureManifest>(&manifest_bytes)
        .map_err(|err| RunError::Build(format!("parse --toolchain-closure {}: {err}", manifest_path.display())))?;
    let validation =
        crate::source_toolchain_closure::validate_toolchain_closure_manifest(&manifest).map_err(|err| {
            RunError::Build(format!("invalid --toolchain-closure {}: {}", manifest_path.display(), err.message()))
        })?;
    Ok(LoadedToolchainClosure {
        status: crate::source_toolchain_closure::validated_source_built_toolchain_closure(
            manifest_path.to_path_buf(),
            &validation,
        ),
        manifest_path: Some(manifest_path.to_path_buf()),
        manifest: Some(manifest),
    })
}

fn write_fixed_point_preflight(
    plan: &FixedPointPlan,
    compatibility: &RustcCompatibilitySummary,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Result<(), RunError> {
    let value = json!({
        "schema": plan.schema,
        "root": plan.root,
        "bundle_dir": plan.bundle_dir,
        "shared_execution_dir": plan.shared_execution_dir,
        "rustc_compatibility": compatibility,
        "source_built_toolchain_closure": toolchain_closure,
    });
    let bytes = serde_json::to_vec_pretty(&value).map_err(|err| internal(format!("serialize preflight: {err}")))?;
    write_bytes(&plan.preflight_path, &bytes)
}

fn write_summary<T: Serialize>(path: &Path, summary: &T) -> Result<(), RunError> {
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

    #[test]
    fn fixed_point_plan_describes_stage_paths_and_commands() {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/tmp/mantle-fixed-point");
        let rustc = Path::new("/toolchain/bin/rustc");

        let plan = plan_fixed_point_paths(root, out_dir, rustc).unwrap();
        let stage1 = &plan.stages[FIXED_POINT_STAGE1_INDEX];
        let stage2 = &plan.stages[FIXED_POINT_STAGE2_INDEX];

        assert_eq!(plan.schema, FIXED_POINT_SCHEMA);
        assert_eq!(plan.root, root);
        assert_eq!(plan.bundle_dir, out_dir);
        assert_eq!(plan.shared_execution_dir, out_dir.join(EXECUTION_DIR));
        assert_eq!(plan.preflight_path, out_dir.join(PRE_FLIGHT_FILE));
        assert_eq!(plan.meta_path, out_dir.join(META_FILE));
        assert_eq!(plan.non_claims_path, out_dir.join(NON_CLAIMS_FILE));
        assert_eq!(plan.stages.len(), FIXED_POINT_STAGE_COUNT);
        assert_eq!(stage1.name, STAGE1_DIR);
        assert_eq!(stage2.name, STAGE2_DIR);
        assert_ne!(stage1.stage_dir, stage2.stage_dir);
        assert_eq!(stage1.execution_dir, plan.shared_execution_dir);
        assert_eq!(stage2.execution_dir, plan.shared_execution_dir);
        assert_eq!(stage1.binary_path, out_dir.join(STAGE1_DIR).join(PRODUCED_MANTLE_FILE));
        assert_eq!(stage2.binary_path, out_dir.join(STAGE2_DIR).join(PRODUCED_MANTLE_FILE));
        assert_eq!(stage1.command.mantle_binary, FixedPointMantleBinary::Host);
        assert_eq!(stage2.command.mantle_binary, FixedPointMantleBinary::StageOutput {
            stage_name: STAGE1_DIR,
            path: stage1.binary_path.clone(),
        });
        assert_eq!(stage1.command.current_dir, root);
        assert_eq!(stage1.command.cargo_env_value, stage1.path_cargo_shim);
        assert_eq!(stage1.command.path_guard_dir, stage1.guard_path_dir);
        assert_eq!(
            stage1.command.args,
            rust_plan_args(root, &stage1.explicit_cargo_shim, rustc, &plan.shared_execution_dir)
        );
    }

    #[test]
    fn fixed_point_plan_rejects_bundle_inside_source_root() {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/repo/mantle/target/proof");
        let rustc = Path::new("/toolchain/bin/rustc");

        let err = plan_fixed_point_paths(root, out_dir, rustc).unwrap_err();
        assert!(err.message().contains("inside source root"));
        assert!(err.message().contains("choose /tmp"));
    }

    #[test]
    fn rustc_wrapper_script_strips_link_self_contained_runtime_args() {
        let script = rustc_wrapper_script(Path::new("/toolchain/bin/rustc"));

        assert!(script.contains(LINK_SELF_CONTAINED_PROBE_ARG));
        assert!(script.contains(LINK_SELF_CONTAINED_JOINED_ARG));
        assert!(script.contains(RUSTC_BOOTSTRAP_ENV));
        assert!(script.contains(REAL_RUSTC_ENV));
        assert!(script.contains("exec"));
    }

    #[test]
    fn fixed_point_plan_rejects_relative_source_root() {
        let err = plan_fixed_point_paths(Path::new("repo"), Path::new("/tmp/proof"), Path::new("rustc")).unwrap_err();
        assert!(err.message().contains("absolute source root"));
        assert!(err.message().contains("repo"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_env_omits_ambient_path_entries() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest);
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();

        let path_env = execution_path_env(&guard_dir, &closure).unwrap();
        let path_entries = env::split_paths(&path_env).collect::<Vec<_>>();

        assert_eq!(path_entries, vec![guard_dir.clone()]);
        assert!(guard_dir.join(C_COMPILER_ALIAS).exists());
        assert!(!path_env.to_string_lossy().contains("/nix/var/nix/profiles"));
        assert!(!path_env.to_string_lossy().contains("/run/current-system/sw"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_sysroot_leakage() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let leaked_sysroot = dir.path().join("leaked-sysroot");
        fs::create_dir_all(&leaked_sysroot).unwrap();
        let rustc_script = rustc_sysroot_script(&leaked_sysroot);
        write_fake_executable(&tools.rustc, &rustc_script);
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("Sysroot"));
        assert!(err.message().contains(path_to_string(&leaked_sysroot).unwrap().as_str()));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_linker_digest_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, Some(("ld", fake_digest())));
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("Linker"));
        assert!(err.message().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_c_compiler_digest_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, Some(("c-compiler", fake_digest())));
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("CCompiler"));
        assert!(err.message().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_pkg_config_digest_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, true);
        let manifest = fake_toolchain_manifest(&tools, Some((PKG_CONFIG_ALIAS, fake_digest())));
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("PkgConfig"));
        assert!(err.message().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_expose_declared_pkg_config_only() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, true);
        let manifest = fake_toolchain_manifest(&tools, None);
        let aliases = toolchain_path_aliases(&manifest).unwrap();

        let pkg_config = tools.pkg_config.as_ref().unwrap();
        assert!(aliases.contains_key(PKG_CONFIG_ALIAS));
        assert_eq!(aliases.get(PKG_CONFIG_ALIAS), Some(pkg_config));
        assert!(!aliases.contains_key("nix"));
        assert!(!aliases.contains_key("nix-store"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_reject_undeclared_nix_profile_tools() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let profile_dir = dir.path().join("nix-profile-bin");
        fs::create_dir_all(&profile_dir).unwrap();
        write_fake_executable(&profile_dir.join("nix"), "#!/bin/sh\nexit 99\n");
        write_fake_executable(&profile_dir.join("nix-store"), "#!/bin/sh\nexit 99\n");
        let manifest = fake_toolchain_manifest(&tools, None);
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();

        write_toolchain_path_aliases(&guard_dir, &manifest).unwrap();
        let aliases = toolchain_path_aliases(&manifest).unwrap();

        assert!(profile_dir.join("nix").is_file());
        assert!(!aliases.contains_key("nix"));
        assert!(!aliases.contains_key("nix-store"));
        assert!(!guard_dir.join("nix").exists());
        assert!(!guard_dir.join("nix-store").exists());
    }

    #[cfg(unix)]
    #[derive(Debug)]
    struct FakeToolchain {
        rustc: PathBuf,
        linker: PathBuf,
        c_compiler: PathBuf,
        sysroot: PathBuf,
        pkg_config: Option<PathBuf>,
    }

    #[cfg(unix)]
    fn fake_toolchain(dir: &tempfile::TempDir, include_pkg_config: bool) -> FakeToolchain {
        let tool_dir = dir.path().join("toolchain").join("bin");
        let sysroot = dir.path().join("toolchain").join("sysroot");
        fs::create_dir_all(&tool_dir).unwrap();
        fs::create_dir_all(&sysroot).unwrap();
        let rustc = tool_dir.join("rustc");
        let linker = tool_dir.join("ld");
        let c_compiler = tool_dir.join("cc");
        write_fake_executable(&rustc, &rustc_sysroot_script(&sysroot));
        write_fake_executable(&linker, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&c_compiler, "#!/bin/sh\nexit 0\n");
        let pkg_config = include_pkg_config.then(|| {
            let path = tool_dir.join(PKG_CONFIG_ALIAS);
            write_fake_executable(&path, "#!/bin/sh\nexit 0\n");
            path
        });
        FakeToolchain {
            rustc,
            linker,
            c_compiler,
            sysroot,
            pkg_config,
        }
    }

    #[cfg(unix)]
    fn fake_toolchain_manifest(
        tools: &FakeToolchain,
        override_digest: Option<(&str, String)>,
    ) -> crate::source_toolchain_closure::ToolchainClosureManifest {
        use crate::source_toolchain_closure::*;
        let mut members = vec![
            fake_member(ToolchainRole::Rustc, "rustc", &tools.rustc, override_digest.as_ref()),
            fake_member(ToolchainRole::Linker, "ld", &tools.linker, override_digest.as_ref()),
            fake_member(ToolchainRole::CCompiler, "c-compiler", &tools.c_compiler, override_digest.as_ref()),
            fake_member(ToolchainRole::Sysroot, "sysroot", &tools.sysroot, override_digest.as_ref()),
        ];
        if let Some(pkg_config) = &tools.pkg_config {
            members.push(fake_member(ToolchainRole::PkgConfig, PKG_CONFIG_ALIAS, pkg_config, override_digest.as_ref()));
        }
        ToolchainClosureManifest {
            schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA.to_string(),
            members,
            seed_exceptions: Vec::new(),
        }
    }

    #[cfg(unix)]
    fn fake_member(
        role: crate::source_toolchain_closure::ToolchainRole,
        name: &str,
        path: &Path,
        override_digest: Option<&(&str, String)>,
    ) -> crate::source_toolchain_closure::ToolchainClosureMember {
        use crate::source_toolchain_closure::*;
        let content_digest_blake3 = override_digest
            .filter(|(target_name, _digest)| *target_name == name)
            .map(|(_target_name, digest)| digest.clone())
            .unwrap_or_else(|| {
                if path.is_file() {
                    blake3_file(path).unwrap()
                } else {
                    fake_digest()
                }
            });
        ToolchainClosureMember {
            role,
            name: name.to_string(),
            execution_path: path_to_string(path).unwrap(),
            content_digest_blake3,
            trust: ToolchainTrust::SourceBuilt,
            source: Some(ToolchainSourceIdentity {
                kind: ToolchainSourceKind::Generated,
                name: format!("{name}-source"),
                digest_blake3: fake_digest(),
            }),
            build_receipt: Some(ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::MantleRustTopology,
                name: format!("{name}-receipt"),
                digest_blake3: fake_digest(),
            }),
        }
    }

    #[cfg(unix)]
    fn loaded_toolchain_closure(
        manifest_path: PathBuf,
        manifest: crate::source_toolchain_closure::ToolchainClosureManifest,
    ) -> LoadedToolchainClosure {
        let validation = crate::source_toolchain_closure::validate_toolchain_closure_manifest(&manifest).unwrap();
        LoadedToolchainClosure {
            status: crate::source_toolchain_closure::validated_source_built_toolchain_closure(
                manifest_path.clone(),
                &validation,
            ),
            manifest_path: Some(manifest_path),
            manifest: Some(manifest),
        }
    }

    #[cfg(unix)]
    fn write_fake_executable(path: &Path, contents: &str) {
        write_text(path, contents).unwrap();
        set_executable(path).unwrap();
    }

    #[cfg(unix)]
    fn rustc_sysroot_script(sysroot: &Path) -> String {
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--print\" ] && [ \"$2\" = \"{RUSTC_SYSROOT_PRINT_ARG}\" ]; then\n  printf '%s\\n' {}\n  exit 0\nfi\nexit 0\n",
            shell_quote(sysroot)
        )
    }

    #[cfg(unix)]
    fn fake_digest() -> String {
        "abababababababababababababababababababababababababababababababab".to_string()
    }
}
