use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
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
const MAX_TOOL_ENV_VARS: usize = 64;
const MAX_TOOL_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;
const TOOL_TIMEOUT_SECS: u64 = 300;
const TOOL_POLL_INTERVAL_MS: u64 = 20;

#[derive(Debug, Clone)]
pub struct ToolInvocation {
    pub stage_key: String,
    pub tool_name: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub work_root: PathBuf,
    pub env: BTreeMap<String, String>,
    pub output_path: Option<PathBuf>,
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
    network_admitted: bool,
    status: String,
    stdout_blake3: Blake3Identity,
    stderr_blake3: Blake3Identity,
    output_blake3: Option<Blake3Identity>,
}

pub fn run_offline_tool(toolchain: &VerifiedToolchain, invocation: ToolInvocation) -> Result<ToolRun, Error> {
    validate_invocation(&invocation)?;
    let program = toolchain.tool_path(&invocation.tool_name)?;
    let program_blake3 = toolchain.tool_digest(&invocation.tool_name)?;
    let bwrap = toolchain.tool_path("bwrap")?;
    let log_dir = invocation.work_root.join(".mantle-tool-logs");
    fs::create_dir_all(&log_dir).map_err(|error| Error::io("creating tool log directory", &log_dir, error))?;
    let stdout_path = log_dir.join(format!("{}.stdout", invocation.stage_key));
    let stderr_path = log_dir.join(format!("{}.stderr", invocation.stage_key));
    let stdout_file =
        File::create(&stdout_path).map_err(|error| Error::io("creating tool stdout", &stdout_path, error))?;
    let stderr_file =
        File::create(&stderr_path).map_err(|error| Error::io("creating tool stderr", &stderr_path, error))?;
    let mut command = sandbox_command(&bwrap, &program, toolchain, &invocation)?;
    command.stdout(stdout_file);
    command.stderr(stderr_file);
    let mut child = command.spawn().map_err(|error| Error::io("spawning sandboxed tool", &program, error))?;
    let status = wait_bounded(&mut child, &invocation.stage_key)?;
    let stdout = read_output_bounded(&stdout_path)?;
    let stderr = read_output_bounded(&stderr_path)?;
    let output_blake3 = invocation.output_path.as_deref().map(hash_file_bounded).transpose()?;
    let receipt = build_receipt(&invocation, program, program_blake3, status, &stdout, &stderr, output_blake3)?;
    let success = status.success();
    debug_assert_eq!(success, receipt.status == "succeeded");
    debug_assert!(!receipt.receipt_blake3.clone().into_hex().is_empty());
    Ok(ToolRun {
        receipt,
        stdout,
        stderr,
        success,
    })
}

fn validate_invocation(invocation: &ToolInvocation) -> Result<(), Error> {
    if invocation.stage_key.is_empty() || invocation.tool_name.is_empty() {
        return Err(Error::Invalid("tool invocation requires stage and tool names".to_string()));
    }
    if invocation.args.len() > MAX_TOOL_ARGS || invocation.env.len() > MAX_TOOL_ENV_VARS {
        return Err(Error::Invalid("tool invocation exceeds argument or environment bounds".to_string()));
    }
    if !invocation.work_root.is_absolute() || !invocation.cwd.starts_with(&invocation.work_root) {
        return Err(Error::Invalid("tool cwd must stay beneath the absolute scratch root".to_string()));
    }
    if invocation
        .output_path
        .as_ref()
        .is_some_and(|path| !path.is_absolute() || !path.starts_with(&invocation.work_root))
    {
        return Err(Error::Invalid("tool output path must stay beneath the scratch root".to_string()));
    }
    debug_assert!(invocation.args.len() <= MAX_TOOL_ARGS);
    debug_assert!(invocation.env.len() <= MAX_TOOL_ENV_VARS);
    Ok(())
}

fn sandbox_command(
    bwrap: &Path,
    program: &Path,
    toolchain: &VerifiedToolchain,
    invocation: &ToolInvocation,
) -> Result<Command, Error> {
    let mut command = Command::new(bwrap);
    command.env_clear();
    command.args([
        "--die-with-parent",
        "--unshare-net",
        "--new-session",
        "--ro-bind",
        "/nix/store",
        "/nix/store",
        "--bind",
    ]);
    command.arg(&invocation.work_root);
    command.arg(&invocation.work_root);
    command.args(["--proc", "/proc", "--dev", "/dev", "--tmpfs", "/tmp", "--chdir"]);
    command.arg(&invocation.cwd);
    command.arg("--clearenv");
    command.args(["--setenv", "PATH"]);
    command.arg(toolchain.root.join("bin"));
    for (key, value) in &invocation.env {
        validate_env_key(key)?;
        command.args(["--setenv", key, value]);
    }
    command.arg("--");
    command.arg(program);
    command.args(&invocation.args);
    debug_assert!(program.is_absolute());
    debug_assert!(bwrap.is_absolute());
    Ok(command)
}

fn validate_env_key(key: &str) -> Result<(), Error> {
    let valid =
        !key.is_empty() && key.bytes().all(|byte| byte == b'_' || byte.is_ascii_uppercase() || byte.is_ascii_digit());
    if !valid || key.contains("TOKEN") || key.contains("SECRET") || key.contains("PASSWORD") {
        return Err(Error::Invalid(format!("tool environment key `{key}` is not admitted")));
    }
    Ok(())
}

fn wait_bounded(child: &mut std::process::Child, stage_key: &str) -> Result<ExitStatus, Error> {
    let deadline = Instant::now() + Duration::from_secs(TOOL_TIMEOUT_SECS);
    loop {
        if let Some(status) =
            child.try_wait().map_err(|error| Error::Tool(format!("waiting for stage `{stage_key}`: {error}")))?
        {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            child
                .kill()
                .map_err(|error| Error::Tool(format!("killing timed-out stage `{stage_key}`: {error}")))?;
            let _ = child.wait();
            return Err(Error::Tool(format!("stage `{stage_key}` exceeded {TOOL_TIMEOUT_SECS} seconds")));
        }
        thread::sleep(Duration::from_millis(TOOL_POLL_INTERVAL_MS));
    }
}

fn read_output_bounded(path: &Path) -> Result<Vec<u8>, Error> {
    let metadata = fs::metadata(path).map_err(|error| Error::io("reading tool output metadata", path, error))?;
    if metadata.len() > MAX_TOOL_OUTPUT_BYTES {
        return Err(Error::Tool(format!("tool output {} exceeded {} bytes", path.display(), MAX_TOOL_OUTPUT_BYTES)));
    }
    let bytes = fs::read(path).map_err(|error| Error::io("reading tool output", path, error))?;
    debug_assert_eq!(u64::try_from(bytes.len()).ok(), Some(metadata.len()));
    debug_assert!(metadata.len() <= MAX_TOOL_OUTPUT_BYTES);
    Ok(bytes)
}

fn build_receipt(
    invocation: &ToolInvocation,
    program: PathBuf,
    program_blake3: Blake3Identity,
    status: ExitStatus,
    stdout: &[u8],
    stderr: &[u8],
    output_blake3: Option<Blake3Identity>,
) -> Result<ToolExecutionReceipt, Error> {
    let status_label = if status.success() { "succeeded" } else { "failed" };
    let input = ToolReceiptIdentityInput {
        schema: String::from(TOOL_RECEIPT_SCHEMA),
        stage_key: invocation.stage_key.clone(),
        program: program.display().to_string(),
        program_blake3: program_blake3.clone(),
        args: invocation.args.clone(),
        network_admitted: false,
        status: String::from(status_label),
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
        schema: String::from(TOOL_RECEIPT_SCHEMA),
        stage_key: invocation.stage_key.clone(),
        program: program.display().to_string(),
        program_blake3,
        args: invocation.args.clone(),
        network_admitted: false,
        status: String::from(status_label),
        stdout_blake3: Blake3Identity::from_slice(stdout),
        stderr_blake3: Blake3Identity::from_slice(stderr),
        output_blake3,
        receipt_blake3,
    })
}
