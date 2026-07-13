use std::collections::BTreeMap;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crunch_wasm_component_core::Blake3Identity;
use serde::Serialize;

use crate::Error;
use crate::model::ToolExecutionReceipt;
use crate::model::VerifiedToolchain;
use crate::toolchain::hash_file_bounded;

const TOOL_RECEIPT_SCHEMA: &str = "mantle-wasm-component-tool-execution-receipt-v1";
const MAX_TOOL_ARGS: usize = 256;
const MAX_TOOL_ARG_BYTES: usize = 16 * 1024;
const MAX_TOOL_ARG_TOTAL_BYTES: usize = 1024 * 1024;
const MAX_TOOL_ENV_VARS: usize = 64;
const MAX_TOOL_ENV_KEY_BYTES: usize = 128;
const MAX_TOOL_ENV_VALUE_BYTES: usize = 64 * 1024;
const MAX_TOOL_ENV_TOTAL_BYTES: usize = 1024 * 1024;
const MAX_TOOL_READ_ONLY_INPUTS: usize = 128;
const MAX_TOOL_READ_ONLY_PATH_BYTES: usize = 16 * 1024;
const DEFAULT_TOOL_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOOL_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;
const DEFAULT_TOOL_TIMEOUT_MS: u64 = 300 * 1000;
const MAX_TOOL_TIMEOUT_MS: u64 = 30 * 60 * 1000;
const TOOL_POLL_INTERVAL_MS: u64 = 20;
const DRAIN_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolLimits {
    pub timeout_ms: u64,
    pub output_bytes: u64,
}

impl Default for ToolLimits {
    fn default() -> Self {
        Self {
            timeout_ms: DEFAULT_TOOL_TIMEOUT_MS,
            output_bytes: DEFAULT_TOOL_OUTPUT_BYTES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolInvocation {
    pub stage_key: String,
    pub tool_name: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub work_root: PathBuf,
    pub env: BTreeMap<String, String>,
    pub read_only_inputs: Vec<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub limits: ToolLimits,
}

#[derive(Debug, Clone)]
pub struct ToolRun {
    pub receipt: ToolExecutionReceipt,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub success: bool,
}

#[derive(Serialize)]
struct ToolReceiptIdentityInput {
    schema: String,
    stage_key: String,
    program: String,
    program_blake3: Blake3Identity,
    args: Vec<String>,
    read_only_inputs: Vec<String>,
    network_admitted: bool,
    status: String,
    stdout_blake3: Blake3Identity,
    stderr_blake3: Blake3Identity,
    output_blake3: Option<Blake3Identity>,
}

struct ValidatedInvocationPaths {
    work_root: PathBuf,
    cwd: PathBuf,
    read_only_inputs: Vec<PathBuf>,
    output_path: Option<PathBuf>,
}

pub(crate) struct WaitOutcome {
    pub(crate) status: ExitStatus,
    pub(crate) failure: Option<&'static str>,
}

pub fn run_offline_tool(toolchain: &VerifiedToolchain, invocation: ToolInvocation) -> Result<ToolRun, Error> {
    let paths = validate_invocation(&invocation)?;
    let program = toolchain.tool_path(&invocation.tool_name)?;
    let program_blake3 = toolchain.tool_digest(&invocation.tool_name)?;
    let bwrap = toolchain.tool_path("bwrap")?;
    let mut command = sandbox_command(&bwrap, &program, toolchain, &invocation, &paths)?;
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    configure_process_group(&mut command)?;
    let mut child = command.spawn().map_err(|error| Error::io("spawning sandboxed tool", &program, error))?;
    let stdout = child.stdout.take().ok_or_else(|| Error::Tool("tool stdout pipe was unavailable".to_string()))?;
    let stderr = child.stderr.take().ok_or_else(|| Error::Tool("tool stderr pipe was unavailable".to_string()))?;
    let output_limit_hit = Arc::new(AtomicBool::new(false));
    let output_bytes_observed = Arc::new(AtomicU64::new(0));
    let stdout_thread = spawn_drain(
        stdout,
        invocation.limits.output_bytes,
        Arc::clone(&output_limit_hit),
        Arc::clone(&output_bytes_observed),
    );
    let stderr_thread = spawn_drain(
        stderr,
        invocation.limits.output_bytes,
        Arc::clone(&output_limit_hit),
        Arc::clone(&output_bytes_observed),
    );
    let mut outcome = wait_bounded(&mut child, &invocation.stage_key, invocation.limits.timeout_ms, &output_limit_hit)?;
    let stdout = join_drain(stdout_thread, "stdout")?;
    let mut stderr = join_drain(stderr_thread, "stderr")?;
    if output_limit_hit.load(Ordering::Acquire) {
        outcome.failure = Some("output-limit-exceeded");
    }
    if let Some(failure) = outcome.failure {
        stderr.extend_from_slice(format!("\nmantle-process-boundary: {failure}\n").as_bytes());
    }
    let output_blake3 = paths
        .output_path
        .as_deref()
        .filter(|path| std::fs::symlink_metadata(path).is_ok())
        .map(hash_file_bounded)
        .transpose()?;
    let status_label = outcome.failure.unwrap_or_else(|| {
        if outcome.status.success() {
            "succeeded"
        } else {
            "failed"
        }
    });
    let receipt = build_receipt(&invocation, program, program_blake3, status_label, &stdout, &stderr, output_blake3)?;
    let success = outcome.failure.is_none() && outcome.status.success();
    debug_assert_eq!(success, receipt.status == "succeeded");
    debug_assert!(stdout.len() <= usize::try_from(invocation.limits.output_bytes).unwrap_or(usize::MAX));
    Ok(ToolRun {
        receipt,
        stdout,
        stderr,
        success,
    })
}

fn validate_invocation(invocation: &ToolInvocation) -> Result<ValidatedInvocationPaths, Error> {
    if invocation.stage_key.is_empty() || invocation.tool_name.is_empty() {
        return Err(Error::Invalid("tool invocation requires stage and tool names".to_string()));
    }
    validate_arguments(&invocation.args)?;
    validate_environment(&invocation.env)?;
    validate_limits(invocation.limits)?;
    let work_root = canonical_no_symlink_root(&invocation.work_root)?;
    let cwd = canonical_confined_existing(&work_root, &invocation.cwd, "tool cwd")?;
    let read_only_inputs = validate_read_only_inputs(&work_root, &invocation.read_only_inputs)?;
    let output_path =
        invocation.output_path.as_ref().map(|path| validate_confined_output(&work_root, path)).transpose()?;
    debug_assert!(cwd.starts_with(&work_root));
    debug_assert!(output_path.as_ref().is_none_or(|path| path.starts_with(&work_root)));
    Ok(ValidatedInvocationPaths {
        work_root,
        cwd,
        read_only_inputs,
        output_path,
    })
}

fn validate_arguments(args: &[String]) -> Result<(), Error> {
    if args.len() > MAX_TOOL_ARGS {
        return Err(Error::Invalid(format!("tool invocation exceeds {MAX_TOOL_ARGS} arguments")));
    }
    let mut total = 0_usize;
    for arg in args {
        if arg.len() > MAX_TOOL_ARG_BYTES || arg.as_bytes().contains(&0) {
            return Err(Error::Invalid(format!("tool argument exceeds {MAX_TOOL_ARG_BYTES} bytes or contains NUL")));
        }
        total = total
            .checked_add(arg.len())
            .ok_or_else(|| Error::Invalid("tool argument bytes overflowed".to_string()))?;
        if total > MAX_TOOL_ARG_TOTAL_BYTES {
            return Err(Error::Invalid(format!("tool arguments exceed {MAX_TOOL_ARG_TOTAL_BYTES} total bytes")));
        }
    }
    debug_assert!(total <= MAX_TOOL_ARG_TOTAL_BYTES);
    debug_assert!(args.len() <= MAX_TOOL_ARGS);
    Ok(())
}

fn validate_environment(env: &BTreeMap<String, String>) -> Result<(), Error> {
    if env.len() > MAX_TOOL_ENV_VARS {
        return Err(Error::Invalid(format!("tool invocation exceeds {MAX_TOOL_ENV_VARS} environment variables")));
    }
    let mut total = 0_usize;
    for (key, value) in env {
        validate_env_key(key)?;
        if key.len() > MAX_TOOL_ENV_KEY_BYTES || value.len() > MAX_TOOL_ENV_VALUE_BYTES || value.as_bytes().contains(&0)
        {
            return Err(Error::Invalid("tool environment key/value exceeds a fixed bound or contains NUL".to_string()));
        }
        total = total
            .checked_add(key.len())
            .and_then(|bytes| bytes.checked_add(value.len()))
            .ok_or_else(|| Error::Invalid("tool environment bytes overflowed".to_string()))?;
        if total > MAX_TOOL_ENV_TOTAL_BYTES {
            return Err(Error::Invalid(format!("tool environment exceeds {MAX_TOOL_ENV_TOTAL_BYTES} total bytes")));
        }
    }
    debug_assert!(total <= MAX_TOOL_ENV_TOTAL_BYTES);
    debug_assert!(env.len() <= MAX_TOOL_ENV_VARS);
    Ok(())
}

fn validate_limits(limits: ToolLimits) -> Result<(), Error> {
    if limits.timeout_ms == 0 || limits.timeout_ms > MAX_TOOL_TIMEOUT_MS {
        return Err(Error::Invalid(format!("tool timeout must be within 1..={MAX_TOOL_TIMEOUT_MS} ms")));
    }
    if limits.output_bytes == 0 || limits.output_bytes > MAX_TOOL_OUTPUT_BYTES {
        return Err(Error::Invalid(format!("tool output bound must be within 1..={MAX_TOOL_OUTPUT_BYTES} bytes")));
    }
    Ok(())
}

fn canonical_no_symlink_root(path: &Path) -> Result<PathBuf, Error> {
    if !path.is_absolute() {
        return Err(Error::Invalid("tool work root must be absolute".to_string()));
    }
    reject_symlink_components(path)?;
    let canonical =
        std::fs::canonicalize(path).map_err(|error| Error::io("canonicalizing tool work root", path, error))?;
    if canonical != path {
        return Err(Error::Invalid(format!("tool work root is not canonical: {}", path.display())));
    }
    Ok(canonical)
}

fn canonical_confined_existing(root: &Path, path: &Path, label: &str) -> Result<PathBuf, Error> {
    reject_symlink_components(path)?;
    let canonical =
        std::fs::canonicalize(path).map_err(|error| Error::io(&format!("canonicalizing {label}"), path, error))?;
    if !canonical.starts_with(root) || !canonical.is_dir() {
        return Err(Error::Invalid(format!("{label} escapes the canonical work root")));
    }
    Ok(canonical)
}

fn validate_read_only_inputs(root: &Path, inputs: &[PathBuf]) -> Result<Vec<PathBuf>, Error> {
    if inputs.len() > MAX_TOOL_READ_ONLY_INPUTS {
        return Err(Error::Invalid(format!("tool invocation exceeds {MAX_TOOL_READ_ONLY_INPUTS} read-only inputs")));
    }
    let mut canonical_inputs = Vec::with_capacity(inputs.len());
    for input in inputs {
        if !input.is_absolute() || input.as_os_str().len() > MAX_TOOL_READ_ONLY_PATH_BYTES {
            return Err(Error::Invalid("read-only input path is not absolute or exceeds its byte bound".to_string()));
        }
        reject_symlink_components(input)?;
        let canonical = std::fs::canonicalize(input)
            .map_err(|error| Error::io("canonicalizing read-only tool input", input, error))?;
        if canonical.starts_with(root) || root.starts_with(&canonical) {
            return Err(Error::Invalid("read-only tool input overlaps the writable work root".to_string()));
        }
        if canonical_inputs.contains(&canonical) {
            return Err(Error::Invalid(format!("duplicate read-only tool input: {}", canonical.display())));
        }
        canonical_inputs.push(canonical);
    }
    debug_assert!(canonical_inputs.len() <= MAX_TOOL_READ_ONLY_INPUTS);
    debug_assert!(canonical_inputs.iter().all(|input| !input.starts_with(root)));
    Ok(canonical_inputs)
}

fn validate_confined_output(root: &Path, path: &Path) -> Result<PathBuf, Error> {
    if !path.is_absolute() {
        return Err(Error::Invalid("tool output path must be absolute".to_string()));
    }
    let parent = path.parent().ok_or_else(|| Error::Invalid("tool output path has no parent".to_string()))?;
    reject_symlink_components(parent)?;
    let canonical_parent =
        std::fs::canonicalize(parent).map_err(|error| Error::io("canonicalizing tool output parent", parent, error))?;
    if !canonical_parent.starts_with(root) {
        return Err(Error::Invalid("tool output path escapes the canonical work root".to_string()));
    }
    if path.exists() || std::fs::symlink_metadata(path).is_ok() {
        return Err(Error::Invalid(format!("tool output must be a new path: {}", path.display())));
    }
    let name = path.file_name().ok_or_else(|| Error::Invalid("tool output path has no file name".to_string()))?;
    let canonical = canonical_parent.join(name);
    debug_assert!(canonical.starts_with(root));
    debug_assert!(!canonical.exists());
    Ok(canonical)
}

fn reject_symlink_components(path: &Path) -> Result<(), Error> {
    let mut cursor = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => cursor.push(component.as_os_str()),
            Component::Normal(part) => {
                cursor.push(part);
                if std::fs::symlink_metadata(&cursor).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
                    return Err(Error::Invalid(format!(
                        "symlink path component is not admitted: {}",
                        cursor.display()
                    )));
                }
            }
            Component::CurDir | Component::ParentDir => {
                return Err(Error::Invalid(format!(
                    "non-canonical path component is not admitted: {}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

fn sandbox_command(
    bwrap: &Path,
    program: &Path,
    toolchain: &VerifiedToolchain,
    invocation: &ToolInvocation,
    paths: &ValidatedInvocationPaths,
) -> Result<Command, Error> {
    let mut command = Command::new(bwrap);
    command.env_clear();
    command.args([
        "--die-with-parent",
        "--unshare-net",
        "--unshare-pid",
        "--new-session",
        "--dir",
        "/nix",
        "--ro-bind",
        "/nix/store",
        "/nix/store",
        "--tmpfs",
        "/tmp",
    ]);
    append_sandbox_parent_dirs(&mut command, &paths.work_root, &paths.read_only_inputs);
    for input in &paths.read_only_inputs {
        command.arg("--ro-bind").arg(input).arg(input);
    }
    command.arg("--bind").arg(&paths.work_root).arg(&paths.work_root);
    command.args(["--proc", "/proc", "--dev", "/dev", "--chdir"]).arg(&paths.cwd);
    command.arg("--clearenv").args(["--setenv", "PATH"]).arg(toolchain.root.join("bin"));
    for (key, value) in &invocation.env {
        command.args(["--setenv", key, value]);
    }
    command.arg("--").arg(program).args(&invocation.args);
    debug_assert!(program.is_absolute());
    debug_assert!(bwrap.is_absolute());
    Ok(command)
}

fn append_sandbox_parent_dirs(command: &mut Command, work_root: &Path, read_only_inputs: &[PathBuf]) {
    let mut directories = std::collections::BTreeSet::new();
    for path in std::iter::once(work_root).chain(read_only_inputs.iter().map(PathBuf::as_path)) {
        let parent = path.parent();
        for ancestor in parent.into_iter().flat_map(Path::ancestors) {
            if ancestor == Path::new("/") || ancestor == Path::new("/tmp") || ancestor == Path::new("/nix") {
                continue;
            }
            directories.insert(ancestor.to_path_buf());
        }
    }
    let mut directories: Vec<PathBuf> = directories.into_iter().collect();
    directories.sort_by_key(|path| path.components().count());
    for directory in directories {
        command.arg("--dir").arg(directory);
    }
}

fn validate_env_key(key: &str) -> Result<(), Error> {
    let valid =
        !key.is_empty() && key.bytes().all(|byte| byte == b'_' || byte.is_ascii_uppercase() || byte.is_ascii_digit());
    if !valid || key.contains("TOKEN") || key.contains("SECRET") || key.contains("PASSWORD") {
        return Err(Error::Invalid(format!("tool environment key `{key}` is not admitted")));
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn configure_process_group(command: &mut Command) -> Result<(), Error> {
    // SAFETY: this closure only invokes async-signal-safe setpgid before exec.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn configure_process_group(_command: &mut Command) -> Result<(), Error> {
    Err(Error::Blocked(
        "Wasm component process supervision requires a Unix process-group implementation".to_string(),
    ))
}

pub(crate) fn spawn_drain<R: Read + Send + 'static>(
    reader: R,
    output_bytes: u64,
    limit_hit: Arc<AtomicBool>,
    total_observed: Arc<AtomicU64>,
) -> thread::JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || drain_bounded(reader, output_bytes, &limit_hit, &total_observed))
}

fn drain_bounded<R: Read>(
    mut reader: R,
    output_bytes: u64,
    limit_hit: &AtomicBool,
    total_observed: &AtomicU64,
) -> std::io::Result<Vec<u8>> {
    let capacity = usize::try_from(output_bytes).unwrap_or(usize::MAX).min(DRAIN_BUFFER_BYTES);
    let mut retained = Vec::with_capacity(capacity);
    let mut buffer = vec![0_u8; DRAIN_BUFFER_BYTES];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let count_u64 = u64::try_from(count).unwrap_or(u64::MAX);
        let prior = total_observed.fetch_add(count_u64, Ordering::AcqRel);
        let observed = prior.saturating_add(count_u64);
        if observed > output_bytes {
            limit_hit.store(true, Ordering::Release);
        }
        let remaining = usize::try_from(output_bytes.saturating_sub(prior)).unwrap_or(usize::MAX);
        let retained_count = count.min(remaining);
        retained.extend_from_slice(&buffer[..retained_count]);
    }
    debug_assert!(retained.len() <= usize::try_from(output_bytes).unwrap_or(usize::MAX));
    debug_assert!(total_observed.load(Ordering::Acquire) >= u64::try_from(retained.len()).unwrap_or(u64::MAX));
    Ok(retained)
}

pub(crate) fn wait_bounded(
    child: &mut std::process::Child,
    stage_key: &str,
    timeout_ms: u64,
    output_limit_hit: &AtomicBool,
) -> Result<WaitOutcome, Error> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if output_limit_hit.load(Ordering::Acquire) {
            let status = terminate_process_tree(child, stage_key)?;
            return Ok(WaitOutcome {
                status,
                failure: Some("output-limit-exceeded"),
            });
        }
        if let Some(status) =
            child.try_wait().map_err(|error| Error::Tool(format!("waiting for stage `{stage_key}`: {error}")))?
        {
            return Ok(WaitOutcome { status, failure: None });
        }
        if Instant::now() >= deadline {
            let status = terminate_process_tree(child, stage_key)?;
            return Ok(WaitOutcome {
                status,
                failure: Some("timeout-exceeded"),
            });
        }
        thread::sleep(Duration::from_millis(TOOL_POLL_INTERVAL_MS));
    }
}

#[cfg(unix)]
fn terminate_process_tree(child: &mut std::process::Child, stage_key: &str) -> Result<ExitStatus, Error> {
    let process_group =
        i32::try_from(child.id()).map_err(|_| Error::Tool(format!("stage `{stage_key}` process id exceeded i32")))?;
    // SAFETY: negative pid addresses the process group created in pre_exec.
    unsafe {
        libc::kill(-process_group, libc::SIGKILL);
    }
    let _ = child.kill();
    child
        .wait()
        .map_err(|error| Error::Tool(format!("reaping terminated stage `{stage_key}`: {error}")))
}

#[cfg(not(unix))]
fn terminate_process_tree(_child: &mut std::process::Child, stage_key: &str) -> Result<ExitStatus, Error> {
    Err(Error::Blocked(format!("cannot terminate stage `{stage_key}` descendants on a non-Unix host")))
}

pub(crate) fn join_drain(handle: thread::JoinHandle<std::io::Result<Vec<u8>>>, stream: &str) -> Result<Vec<u8>, Error> {
    handle
        .join()
        .map_err(|_| Error::Tool(format!("{stream} drain thread panicked")))?
        .map_err(|error| Error::Tool(format!("draining tool {stream}: {error}")))
}

fn build_receipt(
    invocation: &ToolInvocation,
    program: PathBuf,
    program_blake3: Blake3Identity,
    status: &str,
    stdout: &[u8],
    stderr: &[u8],
    output_blake3: Option<Blake3Identity>,
) -> Result<ToolExecutionReceipt, Error> {
    let input = ToolReceiptIdentityInput {
        schema: TOOL_RECEIPT_SCHEMA.to_string(),
        stage_key: invocation.stage_key.clone(),
        program: program.display().to_string(),
        program_blake3: program_blake3.clone(),
        args: invocation.args.clone(),
        read_only_inputs: invocation.read_only_inputs.iter().map(|path| path.display().to_string()).collect(),
        network_admitted: false,
        status: status.to_string(),
        stdout_blake3: Blake3Identity::from_slice(stdout),
        stderr_blake3: Blake3Identity::from_slice(stderr),
        output_blake3: output_blake3.clone(),
    };
    let canonical = serde_json::to_vec(&input)
        .map_err(|error| Error::Invalid(format!("serializing tool receipt identity input: {error}")))?;
    let receipt_blake3 = Blake3Identity::from_slice(&canonical);
    debug_assert!(!canonical.is_empty());
    debug_assert!(!receipt_blake3.clone().into_hex().is_empty());
    Ok(ToolExecutionReceipt {
        schema: TOOL_RECEIPT_SCHEMA.to_string(),
        stage_key: invocation.stage_key.clone(),
        program: program.display().to_string(),
        program_blake3,
        args: invocation.args.clone(),
        read_only_inputs: invocation.read_only_inputs.iter().map(|path| path.display().to_string()).collect(),
        network_admitted: false,
        status: status.to_string(),
        stdout_blake3: Blake3Identity::from_slice(stdout),
        stderr_blake3: Blake3Identity::from_slice(stderr),
        output_blake3,
        receipt_blake3,
    })
}
