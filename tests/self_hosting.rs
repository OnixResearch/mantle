//! Self-hosting proof: stage0 -> stage1 -> stage2.
//!
//! This test is expensive (~30 min) and requires:
//!   - bwrap on PATH
//!   - git, cargo, tar, xz, cp on PATH
//!   - ~4 GiB free disk in the proof scratch filesystem (`target/self-hosting-proof/work/` by
//!     default, or `CRUNCH_PROOF_SCRATCH_DIR`)
//!   - Internet access (for initial bootstrap fetch)
//!
//! Run with:
//!   ./scripts/prove-self-hosting.sh
//!
//! The helper prepares PATH, compiler/linker lookup, pkg-config, openssl,
//! and sandbox-shell discovery before invoking the ignored proof test.
//!
//! The test:
//! 1. Runs `crunch self-build` (stage0) using the checkout binary
//! 2. Finds the stage1 binary in the output store
//! 3. Invalidates the prior `*-crunch` output
//! 4. Runs `stage1/bin/crunch self-build` (stage2) with a fresh state dir
//! 5. Verifies stage2 produced a working binary
//! 6. Checks that stage2 used crunch-built bwrap (not host fallback)

mod audit_support;

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;
use std::process::Stdio;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::OnceLock;
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use assert_cmd::cargo::cargo_bin;
use audit_support::AuditArtifact;
use audit_support::write_command_audit;
use crunch::protected_exec::DigestSpec;
use crunch::protected_exec::ExecutableSeedEntry;
use crunch::protected_exec::ProtectedSeccompAuditEvent;
use crunch::protected_exec::Stage0Inventory;
use crunch::protected_exec::render_stage0_inventory_nickel;
use serde::Serialize;

const MAX_DIAGNOSTIC_LINES: u32 = 60;
const MAX_DIAGNOSTIC_ENTRIES: u32 = 64;
const MAX_PROOF_STORE_ENTRIES: usize = 32;
const MAX_EMBEDDED_STORE_PATHS: usize = 32;
const MAX_RECORDED_PROOF_TOOLS: usize = 16;
#[cfg(unix)]
const PROOF_REMOVABLE_DIR_MODE: u32 = 0o755;
#[cfg(all(test, unix))]
const PROOF_READ_ONLY_DIR_MODE: u32 = 0o555;
#[cfg(test)]
const FAKE_STORE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CONTROLLED_FAILURE_ENV: &str = "CRUNCH_SELF_HOSTING_CONTROLLED_FAILURE";
const PROOF_BUNDLE_ENV: &str = "CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR";
const PROOF_SCRATCH_ENV: &str = "CRUNCH_PROOF_SCRATCH_DIR";
const PROOF_COMMAND_SENTINEL_ENV: &str = "CRUNCH_TEST_PROOF_COMMAND_SENTINEL";
const PROOF_MODE_ENV: &str = "CRUNCH_SELF_HOSTING_PROOF_MODE";
const PROOF_STAGE0_INVENTORY_DOC_ENV: &str = "CRUNCH_SELF_HOSTING_STAGE0_INVENTORY_DOC";
const PROOF_NO_HOST_TOOLS_ENV: &str = "CRUNCH_SELF_HOSTING_NO_HOST_TOOLS";
const PROOF_STAGE0_INVENTORY_ENV: &str = "CRUNCH_SELF_HOSTING_STAGE0_INVENTORY";
const PROOF_LATER_STAGE_HERMETICITY_ENV: &str = "CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE";
const PROOF_BUNDLE_SCHEMA: &str = "crunch-self-hosting-proof-v2";
const PROOF_STAGE1_BINARY_RELATIVE_PATH: &str = "binaries/stage1-crunch";
const PROOF_STAGE2_BINARY_RELATIVE_PATH: &str = "binaries/stage2-crunch";
const PROOF_STAGE0_INVENTORY_RELATIVE_PATH: &str = "stage0-prerequisites/stage0-inventory.ncl";
const PROOF_MODE_FIXED_POINT: &str = "fixed-point";
const PROOF_MODE_NON_NIX_HOST: &str = "non-nix-host";
const DEFAULT_PROOF_SCRATCH_SOURCE: &str = "default repo-local policy";
const HELPER_PROOF_TOOL_NAMES: [&str; 13] = [
    "cargo",
    "rustc",
    "clang",
    "mold",
    "pkg-config",
    "bwrap",
    "git",
    "stat",
    "tar",
    "xz",
    "cp",
    "chmod",
    "bash",
];
const STAGE0_PROOF_TOOL_NAMES: [&str; 8] = ["bwrap", "git", "cargo", "tar", "xz", "cp", "chmod", "bash"];
const BLOCKED_NIX_BINARIES: [&str; 4] = ["nix-build", "nix-store", "nix-shell", "nix"];
const BLOCKED_HOST_TOOL_BINARIES: [&str; 9] = [
    "git",
    "tar",
    "cp",
    "sh",
    "cargo",
    "nix-build",
    "nix-store",
    "nix-shell",
    "nix",
];

struct StageEvidence {
    stage_name: String,
    output: Output,
    stderr: String,
    audit_dir: PathBuf,
    diagnostics_file: PathBuf,
    stdout_file: PathBuf,
    stderr_file: PathBuf,
}

struct CapturedStageOutput {
    output: Output,
    stdout_file: PathBuf,
    stderr_file: PathBuf,
}

#[derive(Debug, Serialize)]
struct ProofBundleManifest {
    schema: &'static str,
    generated_unix_s: u64,
    repo_root: String,
    bundle_dir: String,
    store_dir: String,
    staged_source: String,
    state_dirs: ProofStateDirs,
    prerequisites: ProofPrerequisiteSet,
    store_inventory: ProofStoreInventory,
    binaries: ProofBinarySet,
    tools: ProofToolSet,
    fixed_point: ProofFixedPointAnalysis,
    stage0: ProofStageManifest,
    stage2: ProofStageManifest,
}

#[derive(Debug, Serialize)]
struct ProofStateDirs {
    stage0: String,
    stage2: String,
}

#[derive(Debug, Serialize)]
struct ProofBinarySet {
    checkout: ProofHashedPath,
    stage1: ProofHashedPath,
    stage2: ProofHashedPath,
}

#[derive(Debug, Serialize)]
struct ProofToolSet {
    stage0_bwrap: ProofHashedPath,
    stage0_busybox: ProofHashedPath,
    stage2_bwrap: ProofHashedPath,
    stage2_busybox: ProofHashedPath,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum ProofMode {
    FixedPoint,
    NonNixHost,
}

#[derive(Debug, Serialize)]
struct ProofPrerequisiteSet {
    mode: ProofMode,
    inventory_doc: ProofHashedPath,
    sandbox_shell: ProofHashedPath,
    helper_tools: Vec<ProofResolvedTool>,
    stage0_tools: Vec<ProofResolvedTool>,
    stage0_path_strategy: String,
    stage0_path_dir: Option<String>,
    stage0_nix_binaries_absent: Vec<String>,
    stage0_host_binaries_absent: Vec<String>,
    cc: Option<String>,
    pkg_config_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProofResolvedTool {
    name: String,
    executable: ProofHashedPath,
}

#[derive(Debug, Serialize)]
struct ProofStoreInventory {
    crunch_entries: Vec<String>,
    bwrap_entries: Vec<String>,
    busybox_entries: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProofFixedPointAnalysis {
    stage1_equals_stage2: bool,
    stage1_vs_stage2: ProofBinaryDiff,
    stage0_bwrap_equals_stage2_bwrap: bool,
    stage0_busybox_equals_stage2_busybox: bool,
    stage1_embedded_store_paths: Vec<String>,
    stage2_embedded_store_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProofBinaryDiff {
    same_size: bool,
    size_delta_bytes: i64,
    first_diff_offset: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ProofStageManifest {
    name: String,
    original_audit_dir: String,
    report: ProofReportManifest,
    files: ProofStageFiles,
}

#[derive(Debug, Serialize)]
struct ProofStageFiles {
    audit_meta: ProofHashedPath,
    stdout: ProofHashedPath,
    stderr: ProofHashedPath,
    diagnostics: ProofHashedPath,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ProofProtectedTransition {
    bwrap_path: String,
    bwrap_digest: String,
    bwrap_store_name: String,
    busybox_path: String,
    busybox_digest: String,
    busybox_store_name: String,
}

#[derive(Debug, Serialize)]
struct ProofProtectedExecAudit {
    schema: &'static str,
    no_host_tools: bool,
    stage0_inventory: Option<ProofHashedPath>,
    blocked_host_commands: Vec<&'static str>,
    declared_seed_artifacts: Vec<ProofSeedArtifactRecord>,
    stage0_seccomp_events: Vec<ProtectedSeccompAuditEvent>,
    stage2_seccomp_events: Vec<ProtectedSeccompAuditEvent>,
    stage0_fallback_events: Vec<String>,
    stage2_fallback_events: Vec<String>,
    stage0_transition: Option<ProofProtectedTransition>,
    stage2_transition: Option<ProofProtectedTransition>,
    result: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ProofSeedArtifactRecord {
    id: String,
    role: String,
    phase: String,
    path: String,
    digest_algorithm: String,
    digest_hex: String,
    provenance_category: String,
    provenance: String,
    owner: String,
    required: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ProofReportManifest {
    hermeticity_mode: String,
    invoking_binary: String,
    staged_source: String,
    bwrap_source: String,
    fallback_events: Vec<String>,
    protected_transition: Option<ProofProtectedTransition>,
    protected_seccomp_events: Vec<ProtectedSeccompAuditEvent>,
    busybox_path: Option<String>,
    output_binary: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProofHashedPath {
    path: String,
    size_bytes: u64,
    digest_blake3: String,
}

fn stage_stream_file(proof_dir: &Path, stage_name: &str, stream_name: &str) -> PathBuf {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(matches!(stream_name, "stdout" | "stderr"), "unexpected stream name: {stream_name}");

    proof_dir.join(format!("{stage_name}-{stream_name}.txt"))
}

fn write_stage_stream_files(proof_dir: &Path, stage_name: &str, output: &Output) -> (PathBuf, PathBuf) {
    let stdout_file = stage_stream_file(proof_dir, stage_name, "stdout");
    let stderr_file = stage_stream_file(proof_dir, stage_name, "stderr");
    std::fs::write(&stdout_file, &output.stdout).unwrap_or_else(|err| {
        panic!(
            "{stage_name} stdout file write failed: {err}\n\
             path: {}\nproof_dir: {}",
            stdout_file.display(),
            proof_dir.display(),
        )
    });
    std::fs::write(&stderr_file, &output.stderr).unwrap_or_else(|err| {
        panic!(
            "{stage_name} stderr file write failed: {err}\n\
             path: {}\nproof_dir: {}",
            stderr_file.display(),
            proof_dir.display(),
        )
    });
    (stdout_file, stderr_file)
}

fn copy_stream_to_parent_and_file<R: Read>(mut reader: R, mut file: File, is_stderr: bool) -> io::Result<Vec<u8>> {
    let mut captured = Vec::new();
    let mut buffer = [0_u8; 4096];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        file.write_all(chunk)?;
        if is_stderr {
            let mut sink = io::stderr();
            sink.write_all(chunk)?;
            sink.flush()?;
        } else {
            let mut sink = io::stdout();
            sink.write_all(chunk)?;
            sink.flush()?;
        }
        captured.extend_from_slice(chunk);
    }

    file.flush()?;
    Ok(captured)
}

fn spawn_stream_capture<R: Read + Send + 'static>(
    reader: R,
    file: File,
    is_stderr: bool,
) -> JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || copy_stream_to_parent_and_file(reader, file, is_stderr))
}

fn join_stream_capture(
    handle: JoinHandle<io::Result<Vec<u8>>>,
    stage_name: &str,
    stream_name: &str,
    proof_dir: &Path,
) -> Vec<u8> {
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(matches!(stream_name, "stdout" | "stderr"), "unexpected stream name: {stream_name}");

    match handle.join() {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(err)) => panic!(
            "{stage_name} {stream_name} capture failed: {err}\n\
             proof_dir: {}\n\
             Check {stage_name}-{stream_name}.txt in proof_dir for partial output.",
            proof_dir.display(),
        ),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn run_command_live(
    proof_dir: &Path,
    stage_name: &str,
    command: &mut std::process::Command,
) -> io::Result<CapturedStageOutput> {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");

    let stdout_file = stage_stream_file(proof_dir, stage_name, "stdout");
    let stderr_file = stage_stream_file(proof_dir, stage_name, "stderr");
    let stdout_writer = File::create(&stdout_file)?;
    let stderr_writer = File::create(&stderr_file)?;

    eprintln!("{stage_name} stdout: {}", stdout_file.display());
    eprintln!("{stage_name} stderr: {}", stderr_file.display());

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout_reader = child.stdout.take().ok_or_else(|| io::Error::other("child stdout must be piped"))?;
    let stderr_reader = child.stderr.take().ok_or_else(|| io::Error::other("child stderr must be piped"))?;

    let stdout_handle = spawn_stream_capture(stdout_reader, stdout_writer, false);
    let stderr_handle = spawn_stream_capture(stderr_reader, stderr_writer, true);
    let status = child.wait()?;

    let stdout = join_stream_capture(stdout_handle, stage_name, "stdout", proof_dir);
    let stderr = join_stream_capture(stderr_handle, stage_name, "stderr", proof_dir);
    Ok(CapturedStageOutput {
        output: Output { status, stdout, stderr },
        stdout_file,
        stderr_file,
    })
}

fn pre_stage_context(stage_name: &str, command: &[String], store_dir: &Path, state_dir: &Path) -> String {
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(!command.is_empty(), "stage command must not be empty");

    let logs_dir = state_dir.join("logs");
    let mut out = String::with_capacity(4096);
    out.push_str(&format!("stage: {stage_name}\n"));
    out.push_str(&format!("command: {}\n", command.join(" ")));
    out.push_str(&format!("store_dir: {}\n", store_dir.display()));
    out.push_str(&format!("state_dir: {}\n", state_dir.display()));
    out.push_str(&format!("logs_dir: {}\n", logs_dir.display()));
    append_section(&mut out, "store entries", &list_dir_entries(store_dir, MAX_DIAGNOSTIC_ENTRIES));
    append_section(&mut out, "log entries", &list_dir_entries(&logs_dir, MAX_DIAGNOSTIC_ENTRIES));
    out
}

/// Check prerequisites for the proof.
fn self_build_prereq_error() -> Option<String> {
    // Need the helper's core runtime tools on PATH.
    let tools = ["bwrap", "git", "cargo", "tar", "xz", "cp", "chmod"];
    for tool in tools {
        if std::process::Command::new(tool).arg("--version").output().is_err() {
            return Some(format!(
                "{tool} not on PATH. Run ./scripts/prove-self-hosting.sh --check for the full environment probe."
            ));
        }
    }
    // Need the crunch source tree (Cargo.toml + bootstrap/ in cwd or parents).
    let cwd = std::env::current_dir().unwrap();
    if !cwd.join("Cargo.toml").exists() || !cwd.join("bootstrap").exists() {
        return Some("not in crunch source tree".to_string());
    }
    None
}

/// Find the crunch binary in an output store (`*-crunch/bin/crunch`).
fn find_crunch_binary(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            let binary = entry.path().join("bin").join("crunch");
            if binary.exists() {
                return Some(binary);
            }
        }
    }
    None
}

#[cfg(unix)]
fn make_tree_removable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let child = entry?.path();
        make_tree_removable(&child)?;
    }

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(PROOF_REMOVABLE_DIR_MODE))
}

#[cfg(not(unix))]
fn make_tree_removable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// Remove all `*-crunch` output directories from a store.
fn remove_crunch_outputs(store: &Path) -> u32 {
    let mut removed: u32 = 0;
    let entries = match std::fs::read_dir(store) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") && make_tree_removable(&path).is_ok() && std::fs::remove_dir_all(path).is_ok()
        {
            removed += 1;
        }
    }
    removed
}

/// Scan store for `*-bwrap/bin/bwrap` and return the path if found.
fn find_bwrap_on_disk(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-bwrap") {
            let bin = entry.path().join("bin").join("bwrap");
            if bin.exists() {
                return Some(bin);
            }
        }
    }
    None
}

/// Scan store for `*-busybox/bin/busybox` and return the path if found.
fn find_busybox_on_disk(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-busybox") {
            let bin = entry.path().join("bin").join("busybox");
            if bin.exists() {
                return Some(bin);
            }
        }
    }
    None
}

/// Check a file is executable (unix).
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

/// Parse proof lines from stderr.
fn extract_proof_field<'a>(stderr: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("self-build-proof: {key}=");
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return Some(rest);
        }
    }
    None
}

fn extract_proof_fields<'a>(stderr: &'a str, key: &str) -> Vec<&'a str> {
    let prefix = format!("self-build-proof: {key}=");
    let mut values = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            values.push(rest);
        }
    }
    values
}

fn extract_optional_path_field(stderr: &str, key: &str) -> Option<PathBuf> {
    let value = extract_proof_field(stderr, key)?;
    if value.is_empty() {
        return None;
    }
    if value == "none" {
        return None;
    }
    Some(PathBuf::from(value))
}

fn extract_bwrap_binary_path(stderr: &str) -> Option<PathBuf> {
    let value = extract_proof_field(stderr, "bwrap-source")?;
    let (_, path) = value.split_once(':')?;
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(path))
}

fn extract_store_entry_name(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_string_lossy();
    if file_name == "bin" {
        return path.parent()?.file_name().map(|name| name.to_string_lossy().into_owned());
    }

    let parent = path.parent()?;
    if parent.file_name()?.to_string_lossy() == "bin" {
        return parent.parent()?.file_name().map(|name| name.to_string_lossy().into_owned());
    }

    parent.file_name().map(|name| name.to_string_lossy().into_owned())
}

fn expected_store_prefix(command: &[String]) -> String {
    assert!(!command.is_empty(), "command must not be empty");

    let mut store_prefix = "/crunch/store".to_string();
    let mut nix_compat = false;
    let mut index: usize = 0;
    while index < command.len() {
        let arg = &command[index];
        if arg == "--nix-compat" {
            nix_compat = true;
            index = index.saturating_add(1);
            continue;
        }
        if arg == "--store-prefix" {
            let value = command
                .get(index.saturating_add(1))
                .unwrap_or_else(|| panic!("--store-prefix must be followed by a value: {}", command.join(" ")));
            assert!(!value.is_empty(), "--store-prefix value must not be empty");
            store_prefix = value.clone();
            index = index.saturating_add(2);
            continue;
        }
        index = index.saturating_add(1);
    }

    if nix_compat {
        "/nix/store".to_string()
    } else {
        store_prefix
    }
}

fn append_section(out: &mut String, title: &str, lines: &[String]) {
    assert!(!title.is_empty(), "section title must not be empty");
    assert!(!lines.is_empty(), "section {title} must have at least one line");

    out.push_str(title);
    out.push_str(":\n");
    for line in lines {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
}

fn tail_lines(text: &str, max_lines: u32) -> Vec<String> {
    assert!(max_lines > 0, "max_lines must be positive");

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return vec!["<none>".to_string()];
    }

    let line_count = u32::try_from(lines.len()).unwrap_or(u32::MAX);
    let start = line_count.saturating_sub(max_lines);
    let mut tail = Vec::new();
    if start > 0 {
        tail.push(format!("... {start} earlier lines omitted ..."));
    }
    let start_index = usize::try_from(start).unwrap_or(usize::MAX);
    for line in lines.into_iter().skip(start_index) {
        tail.push(line.to_string());
    }
    tail
}

fn list_dir_entries(dir: &Path, max_entries: u32) -> Vec<String> {
    assert!(max_entries > 0, "max_entries must be positive");

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => return vec![format!("<unavailable: {err}>")],
    };

    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    if names.is_empty() {
        return vec!["<empty>".to_string()];
    }

    let name_count = u32::try_from(names.len()).unwrap_or(u32::MAX);
    if name_count > max_entries {
        let keep = usize::try_from(max_entries).unwrap_or(usize::MAX);
        let omitted = name_count.saturating_sub(max_entries);
        names.truncate(keep);
        names.push(format!("... {omitted} more entries omitted ..."));
    }
    names
}

fn proof_lines(stderr: &str, max_lines: u32) -> Vec<String> {
    assert!(max_lines > 0, "max_lines must be positive");

    let mut lines: Vec<String> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("self-build-proof:"))
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        return vec!["<none>".to_string()];
    }

    let line_count = u32::try_from(lines.len()).unwrap_or(u32::MAX);
    if line_count > max_lines {
        let keep = usize::try_from(max_lines).unwrap_or(usize::MAX);
        let omitted = line_count.saturating_sub(max_lines);
        lines.truncate(keep);
        lines.push(format!("... {omitted} more proof lines omitted ..."));
    }
    lines
}

fn render_stage_diagnostics(
    stage_name: &str,
    command: &[String],
    output: &Output,
    store_dir: &Path,
    state_dir: &Path,
) -> String {
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(!command.is_empty(), "stage command must not be empty");

    let logs_dir = state_dir.join("logs");
    let exit_code = output.status.code().map(|code| code.to_string()).unwrap_or_else(|| "signal".to_string());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut out = String::with_capacity(8192);
    out.push_str(&format!("stage: {stage_name}\n"));
    out.push_str(&format!("exit_code: {exit_code}\n"));
    out.push_str(&format!("command: {}\n", command.join(" ")));
    out.push_str(&format!("store_dir: {}\n", store_dir.display()));
    out.push_str(&format!("state_dir: {}\n", state_dir.display()));
    out.push_str(&format!("logs_dir: {}\n", logs_dir.display()));
    append_section(&mut out, "proof lines", &proof_lines(&stderr, MAX_DIAGNOSTIC_LINES));
    append_section(&mut out, "store entries", &list_dir_entries(store_dir, MAX_DIAGNOSTIC_ENTRIES));
    append_section(&mut out, "log entries", &list_dir_entries(&logs_dir, MAX_DIAGNOSTIC_ENTRIES));
    append_section(&mut out, "stderr tail", &tail_lines(&stderr, MAX_DIAGNOSTIC_LINES));
    append_section(&mut out, "stdout tail", &tail_lines(&stdout, MAX_DIAGNOSTIC_LINES / 2));
    out
}

fn write_stage_diagnostics_file(proof_dir: &Path, stage_name: &str, diagnostics: &str) -> PathBuf {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");

    let path = proof_dir.join(format!("{stage_name}-diagnostics.txt"));
    std::fs::write(&path, diagnostics).unwrap_or_else(|err| {
        panic!(
            "{stage_name} diagnostics write failed: {err}\n\
             path: {}\nproof_dir: {}",
            path.display(),
            proof_dir.display(),
        )
    });
    path
}

fn record_stage_evidence(
    proof_dir: &Path,
    stage_name: &str,
    command: Vec<String>,
    output: Output,
    store_dir: &Path,
    state_dir: &Path,
) -> StageEvidence {
    assert!(proof_dir.exists(), "proof dir must exist");
    assert!(!stage_name.is_empty(), "stage name must not be empty");
    assert!(!command.is_empty(), "stage command must not be empty");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let (stdout_file, stderr_file) = write_stage_stream_files(proof_dir, stage_name, &output);
    let diagnostics = render_stage_diagnostics(stage_name, &command, &output, store_dir, state_dir);
    let diagnostics_file = write_stage_diagnostics_file(proof_dir, stage_name, &diagnostics);
    let logs_dir = state_dir.join("logs");
    let pathinfo_db = state_dir.join("pathinfo.redb");
    let output_binary = extract_optional_path_field(&stderr, "output-binary");
    let busybox_binary = extract_optional_path_field(&stderr, "busybox-path");
    let bwrap_binary = extract_bwrap_binary_path(&stderr);

    let mut audit_artifacts = vec![
        AuditArtifact {
            label: "logs-dir",
            path: &logs_dir,
        },
        AuditArtifact {
            label: "pathinfo-db",
            path: &pathinfo_db,
        },
        AuditArtifact {
            label: "diagnostics",
            path: &diagnostics_file,
        },
        AuditArtifact {
            label: "stdout-capture",
            path: &stdout_file,
        },
        AuditArtifact {
            label: "stderr-capture",
            path: &stderr_file,
        },
    ];
    if let Some(path) = output_binary.as_ref() {
        audit_artifacts.push(AuditArtifact {
            label: "output-binary",
            path,
        });
    }
    if let Some(path) = busybox_binary.as_ref() {
        audit_artifacts.push(AuditArtifact {
            label: "busybox-binary",
            path,
        });
    }
    if let Some(path) = bwrap_binary.as_ref() {
        audit_artifacts.push(AuditArtifact {
            label: "bwrap-binary",
            path,
        });
    }

    let cwd = std::env::current_dir().unwrap_or_else(|err| {
        panic!(
            "{stage_name} cwd lookup failed: {err}\n{}",
            pre_stage_context(stage_name, &command, store_dir, state_dir),
        )
    });
    let audit_dir = write_command_audit("self-hosting", stage_name, &cwd, &command, &output, &audit_artifacts, &[
        ("CRUNCH_STATE_DIR", state_dir.display().to_string()),
        ("CRUNCH_STORE_DIR", store_dir.display().to_string()),
    ])
    .unwrap_or_else(|err| {
        panic!(
            "{stage_name} audit bundle write failed: {err}\n\
             diagnostics: {}\n{}",
            diagnostics_file.display(),
            pre_stage_context(stage_name, &command, store_dir, state_dir),
        )
    });

    StageEvidence {
        stage_name: stage_name.to_string(),
        output,
        stderr,
        audit_dir,
        diagnostics_file,
        stdout_file,
        stderr_file,
    }
}

fn stage_context(stage: &StageEvidence) -> String {
    let mut out = String::with_capacity(10240);
    out.push_str(&format!("audit bundle: {}\n", stage.audit_dir.display()));
    out.push_str(&format!("diagnostics: {}\n", stage.diagnostics_file.display()));
    out.push_str(&format!("stdout: {}\n", stage.stdout_file.display()));
    out.push_str(&format!("stderr: {}\n", stage.stderr_file.display()));
    match std::fs::read_to_string(&stage.diagnostics_file) {
        Ok(snapshot) => {
            out.push_str("saved diagnostics snapshot:\n");
            out.push_str(&snapshot);
            if !snapshot.ends_with('\n') {
                out.push('\n');
            }
        }
        Err(err) => {
            out.push_str("saved diagnostics snapshot:\n");
            out.push_str(&format!("<unavailable: {err}>\n"));
        }
    }
    out
}

fn assert_stage_success(stage: &StageEvidence) {
    let exit_code = stage.output.status.code().unwrap_or(-1);
    assert!(
        stage.output.status.success(),
        "{} failed (exit {}).\n{}",
        stage.stage_name,
        exit_code,
        stage_context(stage),
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn proof_bundle_root() -> PathBuf {
    repo_root().join("target/self-hosting-proof")
}

fn now_unix_s() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0)
}

fn now_unix_ns() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or(0)
}

fn default_proof_bundle_dir() -> PathBuf {
    let unique = format!("run-{}-{}", std::process::id(), now_unix_ns());
    proof_bundle_root().join(unique)
}

fn normalize_proof_bundle_dir(path: PathBuf) -> PathBuf {
    assert!(!path.as_os_str().is_empty(), "proof bundle path must not be empty");

    if path.is_absolute() {
        return path;
    }

    repo_root().join(path)
}

fn resolve_proof_bundle_dir() -> PathBuf {
    match std::env::var_os(PROOF_BUNDLE_ENV) {
        Some(value) => {
            let path = PathBuf::from(value);
            normalize_proof_bundle_dir(path)
        }
        None => default_proof_bundle_dir(),
    }
}

impl ProofMode {
    fn current() -> Self {
        match std::env::var(PROOF_MODE_ENV).ok().as_deref() {
            Some(PROOF_MODE_NON_NIX_HOST) => Self::NonNixHost,
            Some(PROOF_MODE_FIXED_POINT) | None => Self::FixedPoint,
            Some(other) => panic!("unexpected {PROOF_MODE_ENV} value: {other}"),
        }
    }

    fn stage0_path_is_scrubbed(self) -> bool {
        matches!(self, Self::NonNixHost)
    }

    fn stage0_path_strategy(self) -> &'static str {
        if self.stage0_path_is_scrubbed() {
            "scrubbed-non-nix-host"
        } else {
            "inherited"
        }
    }
}

fn proof_no_host_tools_enabled() -> bool {
    match std::env::var(PROOF_NO_HOST_TOOLS_ENV).ok().as_deref() {
        Some("1") => true,
        Some("0") | None => false,
        Some(other) => panic!("unexpected {PROOF_NO_HOST_TOOLS_ENV} value: {other}"),
    }
}

fn proof_stage0_inventory() -> Option<PathBuf> {
    if !proof_no_host_tools_enabled() {
        return None;
    }
    let path = std::env::var_os(PROOF_STAGE0_INVENTORY_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{PROOF_STAGE0_INVENTORY_ENV} must be set when {PROOF_NO_HOST_TOOLS_ENV}=1"));
    assert!(path.exists(), "stage0 inventory must exist: {}", path.display());
    Some(path)
}

fn blocked_host_tool_name(tool: &str) -> bool {
    assert!(!tool.is_empty(), "tool name must not be empty");
    BLOCKED_HOST_TOOL_BINARIES.iter().any(|blocked| blocked == &tool)
}

fn filtered_proof_tool_names(tool_names: &[&'static str], no_host_tools: bool) -> Vec<&'static str> {
    assert!(!tool_names.is_empty(), "tool_names must not be empty");
    let filtered: Vec<&'static str> =
        tool_names.iter().copied().filter(|tool| !no_host_tools || !blocked_host_tool_name(tool)).collect();
    assert!(!filtered.is_empty(), "filtered proof tool set must not be empty");
    filtered
}

fn append_no_host_tools_stage0_args(stage0_command: &mut Vec<String>, inventory: Option<&Path>) {
    assert!(!stage0_command.is_empty(), "stage0 command must not be empty");
    if let Some(inventory) = inventory {
        assert!(inventory.exists(), "stage0 inventory must exist: {}", inventory.display());
        stage0_command.push("--no-host-tools".to_string());
        stage0_command.push("--stage0-inventory".to_string());
        stage0_command.push(inventory.display().to_string());
    }
}

fn proof_later_stage_hermeticity_mode() -> crunch_pipeline::HermeticityMode {
    match std::env::var(PROOF_LATER_STAGE_HERMETICITY_ENV).ok().as_deref() {
        Some("practical") => crunch_pipeline::HermeticityMode::Practical,
        Some("strict") | None => crunch_pipeline::HermeticityMode::Strict,
        Some(other) => panic!("unexpected {PROOF_LATER_STAGE_HERMETICITY_ENV} value: {other}"),
    }
}

fn proof_inventory_doc_source() -> PathBuf {
    let source = std::env::var_os(PROOF_STAGE0_INVENTORY_DOC_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("docs/bootstrap-stage0-inventory.md"));
    assert!(source.exists(), "stage0 inventory doc must exist: {}", source.display());
    source
}

fn canonicalize_or_self(path: &Path) -> PathBuf {
    assert!(path.exists(), "path must exist before canonicalize: {}", path.display());
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn path_entries(path_var: &OsStr) -> Vec<PathBuf> {
    std::env::split_paths(path_var).collect()
}

fn find_executable_in_entries(tool: &str, entries: &[PathBuf]) -> Option<PathBuf> {
    assert!(!tool.is_empty(), "tool name must not be empty");
    assert!(!entries.is_empty(), "search PATH entries must not be empty");

    for entry in entries {
        let candidate = entry.join(tool);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn find_executable_in_path_var(tool: &str, path_var: &OsStr) -> Option<PathBuf> {
    let entries = path_entries(path_var);
    find_executable_in_entries(tool, &entries)
}

#[cfg(unix)]
fn create_stage0_scrubbed_path(proof_dir: &Path) -> PathBuf {
    use std::os::unix::fs::symlink;

    assert!(proof_dir.exists(), "proof dir must exist: {}", proof_dir.display());
    let current_path = std::env::var_os("PATH").expect("PATH must be set before scrubbing stage0");
    let current_entries = path_entries(&current_path);
    let stage0_bin_dir = proof_dir.join("stage0-non-nix-path").join("bin");
    std::fs::create_dir_all(&stage0_bin_dir).unwrap_or_else(|err| panic!("mkdir {}: {err}", stage0_bin_dir.display()));

    for tool in STAGE0_PROOF_TOOL_NAMES {
        let source = find_executable_in_entries(tool, &current_entries)
            .unwrap_or_else(|| panic!("required stage0 tool not found on PATH while scrubbing: {tool}"));
        let target = stage0_bin_dir.join(tool);
        if target.exists() {
            std::fs::remove_file(&target).unwrap_or_else(|err| panic!("remove old {}: {err}", target.display()));
        }
        symlink(&source, &target)
            .unwrap_or_else(|err| panic!("symlink {} -> {}: {err}", source.display(), target.display()));
    }

    let stage0_path = std::env::join_paths([stage0_bin_dir.clone()]).expect("join scrubbed PATH");
    for blocked in BLOCKED_NIX_BINARIES {
        assert!(
            find_executable_in_path_var(blocked, stage0_path.as_os_str()).is_none(),
            "scrubbed stage0 PATH must block {blocked}",
        );
    }

    stage0_bin_dir
}

fn hash_executable_record(path: &Path) -> ProofHashedPath {
    let resolved = canonicalize_or_self(path);
    hash_file_record(&resolved, resolved.display().to_string())
}

fn collect_resolved_tools(tool_names: &[&str], path_var: &OsStr) -> Vec<ProofResolvedTool> {
    assert!(!tool_names.is_empty(), "tool_names must not be empty");

    let entries = path_entries(path_var);
    let mut tools = Vec::with_capacity(tool_names.len());
    for tool in tool_names {
        let resolved = find_executable_in_entries(tool, &entries)
            .unwrap_or_else(|| panic!("required proof tool missing from PATH: {tool}"));
        tools.push(ProofResolvedTool {
            name: (*tool).to_string(),
            executable: hash_executable_record(&resolved),
        });
    }
    assert!(tools.len() <= MAX_RECORDED_PROOF_TOOLS, "recorded tool count exceeded limit");
    tools
}

fn collect_absent_binaries(path_var: &OsStr, blocked_binaries: &[&str]) -> Vec<String> {
    assert!(!blocked_binaries.is_empty(), "blocked_binaries must not be empty");

    let entries = path_entries(path_var);
    let mut absent = Vec::new();
    for blocked in blocked_binaries {
        if find_executable_in_entries(blocked, &entries).is_none() {
            absent.push((*blocked).to_string());
        }
    }
    absent
}

fn collect_prerequisites(
    bundle_dir: &Path,
    proof_mode: ProofMode,
    stage0_path_dir: Option<&Path>,
) -> ProofPrerequisiteSet {
    assert!(!bundle_dir.as_os_str().is_empty(), "bundle dir must not be empty");

    let inventory_doc_source = proof_inventory_doc_source();
    let inventory_doc = copy_bundle_file(&inventory_doc_source, bundle_dir, "stage0-prerequisites/inventory.md");
    let sandbox_shell = std::env::var_os("SNIX_BUILD_SANDBOX_SHELL")
        .map(PathBuf::from)
        .map(|path| hash_executable_record(&path))
        .unwrap_or_else(|| panic!("SNIX_BUILD_SANDBOX_SHELL must be set for proof bundle"));
    let helper_path = std::env::var_os("PATH").expect("PATH must be set for proof bundle");
    let stage0_path = stage0_path_dir
        .map(|dir| std::env::join_paths([dir.to_path_buf()]).expect("join stage0 PATH"))
        .unwrap_or_else(|| helper_path.clone());
    let no_host_tools = proof_no_host_tools_enabled();
    let stage0_nix_binaries_absent = collect_absent_binaries(stage0_path.as_os_str(), &BLOCKED_NIX_BINARIES);
    let stage0_host_binaries_absent = collect_absent_binaries(stage0_path.as_os_str(), &BLOCKED_HOST_TOOL_BINARIES);
    if proof_mode.stage0_path_is_scrubbed() {
        assert_eq!(
            stage0_nix_binaries_absent.len(),
            BLOCKED_NIX_BINARIES.len(),
            "non-nix-host proof must block every Nix binary from stage0 PATH",
        );
    }
    if no_host_tools {
        assert_eq!(
            stage0_host_binaries_absent.len(),
            BLOCKED_HOST_TOOL_BINARIES.len(),
            "no-host-tools proof must block every common host helper from stage0 PATH",
        );
    }

    let helper_tool_names = filtered_proof_tool_names(&HELPER_PROOF_TOOL_NAMES, no_host_tools);
    let stage0_tool_names = filtered_proof_tool_names(&STAGE0_PROOF_TOOL_NAMES, no_host_tools);

    ProofPrerequisiteSet {
        mode: proof_mode,
        inventory_doc,
        sandbox_shell,
        helper_tools: collect_resolved_tools(&helper_tool_names, &helper_path),
        stage0_tools: collect_resolved_tools(&stage0_tool_names, stage0_path.as_os_str()),
        stage0_path_strategy: proof_mode.stage0_path_strategy().to_string(),
        stage0_path_dir: stage0_path_dir.map(|dir| dir.display().to_string()),
        stage0_nix_binaries_absent,
        stage0_host_binaries_absent,
        cc: std::env::var("CC").ok(),
        pkg_config_path: std::env::var("PKG_CONFIG_PATH").ok(),
    }
}

fn parse_protected_transition(stage: &StageEvidence) -> Option<ProofProtectedTransition> {
    let marker = extract_proof_field(&stage.stderr, "protected-transition")?;
    if marker == "none" {
        return None;
    }
    assert_eq!(marker, "bootstrap-tools-selected", "unexpected protected transition marker");
    Some(ProofProtectedTransition {
        bwrap_path: extract_proof_field(&stage.stderr, "protected-transition-bwrap-path")
            .unwrap_or_else(|| panic!("{} missing transition bwrap path", stage.stage_name))
            .to_string(),
        bwrap_digest: extract_proof_field(&stage.stderr, "protected-transition-bwrap-digest")
            .unwrap_or_else(|| panic!("{} missing transition bwrap digest", stage.stage_name))
            .to_string(),
        bwrap_store_name: extract_proof_field(&stage.stderr, "protected-transition-bwrap-store-name")
            .unwrap_or_else(|| panic!("{} missing transition bwrap store name", stage.stage_name))
            .to_string(),
        busybox_path: extract_proof_field(&stage.stderr, "protected-transition-busybox-path")
            .unwrap_or_else(|| panic!("{} missing transition busybox path", stage.stage_name))
            .to_string(),
        busybox_digest: extract_proof_field(&stage.stderr, "protected-transition-busybox-digest")
            .unwrap_or_else(|| panic!("{} missing transition busybox digest", stage.stage_name))
            .to_string(),
        busybox_store_name: extract_proof_field(&stage.stderr, "protected-transition-busybox-store-name")
            .unwrap_or_else(|| panic!("{} missing transition busybox store name", stage.stage_name))
            .to_string(),
    })
}

fn parse_protected_seccomp_events(stage: &StageEvidence) -> Vec<ProtectedSeccompAuditEvent> {
    extract_proof_fields(&stage.stderr, "protected-seccomp-event")
        .into_iter()
        .filter(|value| *value != "none")
        .map(|value| {
            serde_json::from_str(value).unwrap_or_else(|err| {
                panic!("{} has invalid protected seccomp event JSON: {err}\n{value}", stage.stage_name)
            })
        })
        .collect()
}

fn parse_stage_report(stage: &StageEvidence) -> ProofReportManifest {
    assert!(!stage.stage_name.is_empty(), "stage name must not be empty");
    assert!(!stage.stderr.is_empty(), "stage stderr must not be empty");

    let hermeticity_mode = extract_proof_field(&stage.stderr, "hermeticity-mode").unwrap_or_else(|| {
        panic!("{} missing hermeticity-mode proof line.\n{}", stage.stage_name, stage_context(stage),)
    });
    let invoking_binary = extract_proof_field(&stage.stderr, "invoking-binary").unwrap_or_else(|| {
        panic!("{} missing invoking-binary proof line.\n{}", stage.stage_name, stage_context(stage),)
    });
    let staged_source = extract_proof_field(&stage.stderr, "staged-source")
        .unwrap_or_else(|| panic!("{} missing staged-source proof line.\n{}", stage.stage_name, stage_context(stage),));
    let bwrap_source = extract_proof_field(&stage.stderr, "bwrap-source")
        .unwrap_or_else(|| panic!("{} missing bwrap-source proof line.\n{}", stage.stage_name, stage_context(stage),));
    let output_binary = extract_proof_field(&stage.stderr, "output-binary")
        .unwrap_or_else(|| panic!("{} missing output-binary proof line.\n{}", stage.stage_name, stage_context(stage),));
    let fallback_events_raw = extract_proof_fields(&stage.stderr, "fallback-event");
    assert!(
        !fallback_events_raw.is_empty(),
        "{} missing fallback-event proof lines.\n{}",
        stage.stage_name,
        stage_context(stage),
    );
    let fallback_events = if fallback_events_raw.len() == 1 && fallback_events_raw[0] == "none" {
        Vec::new()
    } else {
        fallback_events_raw.into_iter().map(str::to_string).collect()
    };

    ProofReportManifest {
        hermeticity_mode: hermeticity_mode.to_string(),
        invoking_binary: invoking_binary.to_string(),
        staged_source: staged_source.to_string(),
        bwrap_source: bwrap_source.to_string(),
        fallback_events,
        protected_transition: parse_protected_transition(stage),
        protected_seccomp_events: parse_protected_seccomp_events(stage),
        busybox_path: extract_optional_path_field(&stage.stderr, "busybox-path").map(|path| path.display().to_string()),
        output_binary: output_binary.to_string(),
    }
}

fn hash_file_record(path: &Path, display_path: String) -> ProofHashedPath {
    assert!(path.exists(), "path to hash must exist: {}", path.display());
    assert!(path.is_file(), "path to hash must be a file: {}", path.display());

    let bytes = std::fs::read(path).unwrap_or_else(|err| panic!("read {} for digest: {err}", path.display()));
    let size_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    let digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    ProofHashedPath {
        path: display_path,
        size_bytes,
        digest_blake3,
    }
}

fn hashed_paths_match(left: &ProofHashedPath, right: &ProofHashedPath) -> bool {
    left.size_bytes == right.size_bytes && left.digest_blake3 == right.digest_blake3
}

fn diff_binary_bytes(left: &Path, right: &Path) -> ProofBinaryDiff {
    let left_bytes = std::fs::read(left).unwrap_or_else(|err| panic!("read {} for diff: {err}", left.display()));
    let right_bytes = std::fs::read(right).unwrap_or_else(|err| panic!("read {} for diff: {err}", right.display()));
    let same_size = left_bytes.len() == right_bytes.len();
    let size_delta_bytes =
        i64::try_from(right_bytes.len()).unwrap_or(i64::MAX) - i64::try_from(left_bytes.len()).unwrap_or(i64::MAX);
    let first_diff_offset = left_bytes
        .iter()
        .zip(right_bytes.iter())
        .position(|(left_byte, right_byte)| left_byte != right_byte)
        .map(|offset| u64::try_from(offset).unwrap_or(u64::MAX))
        .or_else(|| {
            if same_size {
                None
            } else {
                Some(u64::try_from(left_bytes.len().min(right_bytes.len())).unwrap_or(u64::MAX))
            }
        });
    ProofBinaryDiff {
        same_size,
        size_delta_bytes,
        first_diff_offset,
    }
}

fn find_embedded_store_path_end(candidate: &str) -> Option<usize> {
    for marker in ["-busybox/bin/busybox", "-bwrap/bin/bwrap", "-crunch-src"] {
        if let Some(offset) = candidate.find(marker) {
            return Some(offset + marker.len());
        }
    }
    None
}

fn collect_embedded_store_paths(binary_path: &Path) -> Vec<String> {
    let bytes = std::fs::read(binary_path)
        .unwrap_or_else(|err| panic!("read {} for embedded store path scan: {err}", binary_path.display()));
    let needle = b"/nix/store/";
    let mut paths = std::collections::BTreeSet::new();
    let mut offset: usize = 0;

    while offset + needle.len() <= bytes.len() {
        if &bytes[offset..offset + needle.len()] != needle {
            offset = offset.saturating_add(1);
            continue;
        }

        let remaining = bytes.len().saturating_sub(offset);
        let window_len = remaining.min(256);
        let window = &bytes[offset..offset + window_len];
        let candidate_len = window.iter().position(|byte| *byte == 0).unwrap_or(window_len);
        let candidate = String::from_utf8_lossy(&window[..candidate_len]);
        let Some(relative_end) = find_embedded_store_path_end(&candidate) else {
            offset = offset.saturating_add(needle.len());
            continue;
        };
        paths.insert(candidate[..relative_end].to_string());
        if paths.len() >= MAX_EMBEDDED_STORE_PATHS {
            break;
        }
        offset = offset.saturating_add(relative_end);
    }

    paths.into_iter().collect()
}

fn collect_store_entries(store_dir: &Path, suffix: &str) -> Vec<String> {
    let entries =
        std::fs::read_dir(store_dir).unwrap_or_else(|err| panic!("read store dir {}: {err}", store_dir.display()));
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read store entry in {}: {err}", store_dir.display()));
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if name.ends_with(suffix) {
            names.push(name);
            if names.len() >= MAX_PROOF_STORE_ENTRIES {
                break;
            }
        }
    }
    names.sort();
    names
}

fn collect_store_inventory(store_dir: &Path) -> ProofStoreInventory {
    ProofStoreInventory {
        crunch_entries: collect_store_entries(store_dir, "-crunch"),
        bwrap_entries: collect_store_entries(store_dir, "-bwrap"),
        busybox_entries: collect_store_entries(store_dir, "-busybox"),
    }
}

fn copy_bundle_file(src: &Path, bundle_dir: &Path, relative_path: &str) -> ProofHashedPath {
    assert!(src.exists(), "bundle source must exist: {}", src.display());
    assert!(!relative_path.is_empty(), "bundle relative path must not be empty");

    let dest = bundle_dir.join(relative_path);
    let parent = dest.parent().unwrap_or_else(|| panic!("bundle file has no parent: {}", dest.display()));
    std::fs::create_dir_all(parent).unwrap_or_else(|err| panic!("create bundle parent {}: {err}", parent.display()));
    std::fs::copy(src, &dest).unwrap_or_else(|err| panic!("copy {} -> {}: {err}", src.display(), dest.display()));
    hash_file_record(&dest, relative_path.to_string())
}

fn normalize_bwrap_binary_path(bwrap_report_path: &Path) -> PathBuf {
    assert!(bwrap_report_path.exists(), "reported bwrap path must exist: {}", bwrap_report_path.display(),);

    if bwrap_report_path.file_name().map(|name| name == "bin").unwrap_or(false) {
        return bwrap_report_path.join("bwrap");
    }
    bwrap_report_path.to_path_buf()
}

fn copy_stage_bundle_files(bundle_dir: &Path, stage: &StageEvidence) -> ProofStageFiles {
    let stage_dir = bundle_dir.join(&stage.stage_name);
    std::fs::create_dir_all(&stage_dir)
        .unwrap_or_else(|err| panic!("create stage bundle dir {}: {err}", stage_dir.display()));

    ProofStageFiles {
        audit_meta: copy_bundle_file(
            &stage.audit_dir.join("meta.json"),
            bundle_dir,
            &format!("{}/meta.json", stage.stage_name),
        ),
        stdout: copy_bundle_file(&stage.stdout_file, bundle_dir, &format!("{}/stdout.txt", stage.stage_name)),
        stderr: copy_bundle_file(&stage.stderr_file, bundle_dir, &format!("{}/stderr.txt", stage.stage_name)),
        diagnostics: copy_bundle_file(
            &stage.diagnostics_file,
            bundle_dir,
            &format!("{}/diagnostics.txt", stage.stage_name),
        ),
    }
}

fn write_protected_exec_audit(bundle_dir: &Path, manifest: &ProofBundleManifest) -> ProofHashedPath {
    assert!(bundle_dir.exists(), "bundle dir must exist before protected audit write");
    let no_host_tools = proof_no_host_tools_enabled();
    let (stage0_inventory, declared_seed_artifacts) = copy_stage0_inventory_into_bundle(bundle_dir)
        .map(|(record, artifacts)| (Some(record), artifacts))
        .unwrap_or_else(|| (None, Vec::new()));
    let audit = ProofProtectedExecAudit {
        schema: "crunch-protected-exec-audit-v1",
        no_host_tools,
        stage0_inventory,
        blocked_host_commands: BLOCKED_HOST_TOOL_BINARIES.to_vec(),
        declared_seed_artifacts,
        stage0_seccomp_events: manifest.stage0.report.protected_seccomp_events.clone(),
        stage2_seccomp_events: manifest.stage2.report.protected_seccomp_events.clone(),
        stage0_fallback_events: manifest.stage0.report.fallback_events.clone(),
        stage2_fallback_events: manifest.stage2.report.fallback_events.clone(),
        stage0_transition: manifest.stage0.report.protected_transition.clone(),
        stage2_transition: manifest.stage2.report.protected_transition.clone(),
        result: derived_proof_result(manifest),
    };
    let path = bundle_dir.join("protected-exec-audit.json");
    let json = serde_json::to_vec_pretty(&audit).unwrap_or_else(|err| panic!("serialize protected exec audit: {err}"));
    std::fs::write(&path, json).unwrap_or_else(|err| panic!("write protected exec audit {}: {err}", path.display()));
    hash_file_record(&path, "protected-exec-audit.json".to_string())
}

fn copy_stage0_inventory_into_bundle(bundle_dir: &Path) -> Option<(ProofHashedPath, Vec<ProofSeedArtifactRecord>)> {
    let source = proof_stage0_inventory()?;
    let record = copy_bundle_file(&source, bundle_dir, PROOF_STAGE0_INVENTORY_RELATIVE_PATH);
    let copied = bundle_dir.join(PROOF_STAGE0_INVENTORY_RELATIVE_PATH);
    let inventory = load_stage0_inventory_for_proof(&copied);
    let artifacts = seed_artifacts_from_inventory(&inventory);
    Some((record, artifacts))
}

fn load_stage0_inventory_for_proof(path: &Path) -> Stage0Inventory {
    let import_paths: Vec<std::ffi::OsString> = Vec::new();
    crunch_eval::evaluate_and_deserialize(path, &import_paths)
        .unwrap_or_else(|err| panic!("load bundled stage0 inventory {}: {err}", path.display()))
}

fn seed_artifacts_from_inventory(inventory: &Stage0Inventory) -> Vec<ProofSeedArtifactRecord> {
    inventory
        .executable_entries
        .iter()
        .map(|entry| ProofSeedArtifactRecord {
            id: entry.id.clone(),
            role: entry.role.clone(),
            phase: entry.phase.clone(),
            path: entry.executable_path.display().to_string(),
            digest_algorithm: entry.digest.algorithm.clone(),
            digest_hex: entry.digest.hex.clone(),
            provenance_category: entry.provenance_category.clone(),
            provenance: entry.provenance.clone(),
            owner: entry.owner.clone(),
            required: entry.required,
        })
        .collect()
}

fn bundled_stage0_inventory_record(manifest: &ProofBundleManifest) -> Option<ProofHashedPath> {
    let path = PathBuf::from(&manifest.bundle_dir).join(PROOF_STAGE0_INVENTORY_RELATIVE_PATH);
    path.exists().then(|| hash_file_record(&path, PROOF_STAGE0_INVENTORY_RELATIVE_PATH.to_string()))
}

fn declared_seed_artifacts_for_summary(manifest: &ProofBundleManifest) -> Vec<ProofSeedArtifactRecord> {
    let path = PathBuf::from(&manifest.bundle_dir).join(PROOF_STAGE0_INVENTORY_RELATIVE_PATH);
    if !path.exists() {
        return Vec::new();
    }
    seed_artifacts_from_inventory(&load_stage0_inventory_for_proof(&path))
}

fn derived_proof_result(manifest: &ProofBundleManifest) -> &'static str {
    if manifest.fixed_point.stage1_equals_stage2
        && manifest.fixed_point.stage0_bwrap_equals_stage2_bwrap
        && manifest.fixed_point.stage0_busybox_equals_stage2_busybox
    {
        return "success";
    }
    "fixed-point-mismatch"
}

fn render_proof_bundle_summary(manifest: &ProofBundleManifest, protected_audit: &ProofHashedPath) -> String {
    assert_eq!(manifest.schema, PROOF_BUNDLE_SCHEMA, "unexpected proof bundle schema");
    assert!(!manifest.bundle_dir.is_empty(), "bundle dir must not be empty");
    let bundled_inventory = bundled_stage0_inventory_record(manifest);
    let declared_seed_artifacts = declared_seed_artifacts_for_summary(manifest);

    let mut out = String::with_capacity(6144);
    out.push_str(&format!("schema: {}\n", manifest.schema));
    out.push_str(&format!("generated_unix_s: {}\n", manifest.generated_unix_s));
    out.push_str(&format!("repo_root: {}\n", manifest.repo_root));
    out.push_str(&format!("bundle_dir: {}\n", manifest.bundle_dir));
    out.push_str(&format!("store_dir: {}\n", manifest.store_dir));
    out.push_str(&format!("staged_source: {}\n", manifest.staged_source));
    out.push_str(&format!("proof_mode: {:?}\n", manifest.prerequisites.mode));
    out.push_str(&format!("protected_exec_result: {}\n", derived_proof_result(manifest)));
    out.push_str(&format!("protected_exec_audit: {} {}\n", protected_audit.digest_blake3, protected_audit.path));
    match bundled_inventory {
        Some(record) => out.push_str(&format!("stage0_inventory_copy: {} {}\n", record.digest_blake3, record.path)),
        None => out.push_str("stage0_inventory_copy: none\n"),
    }
    out.push_str(&format!("declared_seed_artifacts: {}\n", declared_seed_artifacts.len()));
    for artifact in &declared_seed_artifacts {
        out.push_str(&format!(
            "declared_seed_artifact: id={} role={} phase={} path={} digest={}:{} provenance_category={} provenance={} owner={} required={}\n",
            artifact.id,
            artifact.role,
            artifact.phase,
            artifact.path,
            artifact.digest_algorithm,
            artifact.digest_hex,
            artifact.provenance_category,
            artifact.provenance,
            artifact.owner,
            artifact.required,
        ));
    }
    out.push_str(&format!("stage0_path_strategy: {}\n", manifest.prerequisites.stage0_path_strategy));
    out.push_str(&format!(
        "stage0_path_dir: {}\n",
        manifest.prerequisites.stage0_path_dir.as_deref().unwrap_or("<inherited>")
    ));
    out.push_str(&format!("stage0_nix_binaries_absent: {:?}\n", manifest.prerequisites.stage0_nix_binaries_absent));
    out.push_str(&format!("stage0_host_binaries_absent: {:?}\n", manifest.prerequisites.stage0_host_binaries_absent));
    out.push_str(&format!(
        "stage0_inventory_doc: {} {}\n",
        manifest.prerequisites.inventory_doc.digest_blake3, manifest.prerequisites.inventory_doc.path
    ));
    out.push_str(&format!(
        "sandbox_shell: {} {}\n",
        manifest.prerequisites.sandbox_shell.digest_blake3, manifest.prerequisites.sandbox_shell.path
    ));
    out.push_str(&format!("stage0_state: {}\n", manifest.state_dirs.stage0));
    out.push_str(&format!("stage2_state: {}\n", manifest.state_dirs.stage2));
    out.push_str(&format!("store_crunch_entries: {:?}\n", manifest.store_inventory.crunch_entries));
    out.push_str(&format!("store_bwrap_entries: {:?}\n", manifest.store_inventory.bwrap_entries));
    out.push_str(&format!("store_busybox_entries: {:?}\n", manifest.store_inventory.busybox_entries));
    out.push_str(&format!(
        "checkout_binary: {} {}\n",
        manifest.binaries.checkout.digest_blake3, manifest.binaries.checkout.path
    ));
    out.push_str(&format!(
        "stage1_binary: {} {}\n",
        manifest.binaries.stage1.digest_blake3, manifest.binaries.stage1.path
    ));
    out.push_str(&format!(
        "stage2_binary: {} {}\n",
        manifest.binaries.stage2.digest_blake3, manifest.binaries.stage2.path
    ));
    out.push_str(&format!(
        "stage0_bwrap: {} {}\n",
        manifest.tools.stage0_bwrap.digest_blake3, manifest.tools.stage0_bwrap.path
    ));
    out.push_str(&format!(
        "stage0_busybox: {} {}\n",
        manifest.tools.stage0_busybox.digest_blake3, manifest.tools.stage0_busybox.path
    ));
    out.push_str(&format!(
        "stage2_bwrap: {} {}\n",
        manifest.tools.stage2_bwrap.digest_blake3, manifest.tools.stage2_bwrap.path
    ));
    out.push_str(&format!(
        "stage2_busybox: {} {}\n",
        manifest.tools.stage2_busybox.digest_blake3, manifest.tools.stage2_busybox.path
    ));
    out.push_str(&format!("stage1_equals_stage2: {}\n", manifest.fixed_point.stage1_equals_stage2));
    out.push_str(&format!(
        "stage1_vs_stage2: same_size={} size_delta_bytes={} first_diff_offset={:?}\n",
        manifest.fixed_point.stage1_vs_stage2.same_size,
        manifest.fixed_point.stage1_vs_stage2.size_delta_bytes,
        manifest.fixed_point.stage1_vs_stage2.first_diff_offset,
    ));
    out.push_str(&format!(
        "stage0_bwrap_equals_stage2_bwrap: {}\n",
        manifest.fixed_point.stage0_bwrap_equals_stage2_bwrap
    ));
    out.push_str(&format!(
        "stage0_busybox_equals_stage2_busybox: {}\n",
        manifest.fixed_point.stage0_busybox_equals_stage2_busybox
    ));
    out.push_str(&format!("stage1_embedded_store_paths: {:?}\n", manifest.fixed_point.stage1_embedded_store_paths));
    out.push_str(&format!("stage2_embedded_store_paths: {:?}\n", manifest.fixed_point.stage2_embedded_store_paths));
    out.push_str(&format!("stage0_hermeticity_mode: {}\n", manifest.stage0.report.hermeticity_mode));
    out.push_str(&format!("stage0_fallback_events: {:?}\n", manifest.stage0.report.fallback_events));
    out.push_str(&format!("stage0_report: {}\n", manifest.stage0.report.bwrap_source));
    out.push_str(&format!("stage0_protected_transition: {:?}\n", manifest.stage0.report.protected_transition));
    out.push_str(&format!("stage2_hermeticity_mode: {}\n", manifest.stage2.report.hermeticity_mode));
    out.push_str(&format!("stage2_fallback_events: {:?}\n", manifest.stage2.report.fallback_events));
    out.push_str(&format!("stage2_report: {}\n", manifest.stage2.report.bwrap_source));
    out.push_str(&format!("stage2_protected_transition: {:?}\n", manifest.stage2.report.protected_transition));
    out
}

fn write_proof_bundle(
    bundle_dir: &Path,
    stage0: &StageEvidence,
    stage2: &StageEvidence,
    store_dir: &Path,
    stage0_state_dir: &Path,
    stage2_state_dir: &Path,
    stage1_binary: &Path,
    stage2_binary: &Path,
    proof_mode: ProofMode,
    stage0_path_dir: Option<&Path>,
) -> PathBuf {
    assert!(!bundle_dir.as_os_str().is_empty(), "bundle dir must not be empty");
    assert!(store_dir.exists(), "store dir must exist: {}", store_dir.display());
    assert!(stage0_state_dir.exists(), "stage0 state dir must exist: {}", stage0_state_dir.display());
    assert!(stage2_state_dir.exists(), "stage2 state dir must exist: {}", stage2_state_dir.display());
    assert!(stage1_binary.exists(), "stage1 binary must exist: {}", stage1_binary.display());
    assert!(stage2_binary.exists(), "stage2 binary must exist: {}", stage2_binary.display());

    std::fs::create_dir_all(bundle_dir)
        .unwrap_or_else(|err| panic!("create proof bundle dir {}: {err}", bundle_dir.display()));

    let mut stage0_report = parse_stage_report(stage0);
    let mut stage2_report = parse_stage_report(stage2);
    assert_eq!(
        stage0_report.staged_source, stage2_report.staged_source,
        "proof bundle expects stage0 and stage2 to use the same staged source",
    );

    let durable_stage1_binary = copy_bundle_file(stage1_binary, bundle_dir, PROOF_STAGE1_BINARY_RELATIVE_PATH);
    let durable_stage2_binary = copy_bundle_file(stage2_binary, bundle_dir, PROOF_STAGE2_BINARY_RELATIVE_PATH);
    stage0_report.output_binary = durable_stage1_binary.path.clone();
    stage2_report.output_binary = durable_stage2_binary.path.clone();

    let checkout_binary_path = PathBuf::from(&stage0_report.invoking_binary);
    let stage0_bwrap_report_path = extract_bwrap_binary_path(&stage0.stderr)
        .unwrap_or_else(|| panic!("stage0 missing bwrap report path.\n{}", stage_context(stage0),));
    let stage0_bwrap_binary = normalize_bwrap_binary_path(&stage0_bwrap_report_path);
    let stage0_busybox_binary = extract_optional_path_field(&stage0.stderr, "busybox-path")
        .unwrap_or_else(|| panic!("stage0 missing busybox-path proof line.\n{}", stage_context(stage0),));
    let stage2_bwrap_report_path = extract_bwrap_binary_path(&stage2.stderr)
        .unwrap_or_else(|| panic!("stage2 missing bwrap report path.\n{}", stage_context(stage2),));
    let stage2_bwrap_binary = normalize_bwrap_binary_path(&stage2_bwrap_report_path);
    let stage2_busybox_binary = extract_optional_path_field(&stage2.stderr, "busybox-path")
        .unwrap_or_else(|| panic!("stage2 missing busybox-path proof line.\n{}", stage_context(stage2),));

    let binaries = ProofBinarySet {
        checkout: hash_file_record(&checkout_binary_path, checkout_binary_path.display().to_string()),
        stage1: durable_stage1_binary,
        stage2: durable_stage2_binary,
    };
    let tools = ProofToolSet {
        stage0_bwrap: hash_file_record(&stage0_bwrap_binary, stage0_bwrap_binary.display().to_string()),
        stage0_busybox: hash_file_record(&stage0_busybox_binary, stage0_busybox_binary.display().to_string()),
        stage2_bwrap: hash_file_record(&stage2_bwrap_binary, stage2_bwrap_binary.display().to_string()),
        stage2_busybox: hash_file_record(&stage2_busybox_binary, stage2_busybox_binary.display().to_string()),
    };
    let fixed_point = ProofFixedPointAnalysis {
        stage1_equals_stage2: hashed_paths_match(&binaries.stage1, &binaries.stage2),
        stage1_vs_stage2: diff_binary_bytes(stage1_binary, stage2_binary),
        stage0_bwrap_equals_stage2_bwrap: hashed_paths_match(&tools.stage0_bwrap, &tools.stage2_bwrap),
        stage0_busybox_equals_stage2_busybox: hashed_paths_match(&tools.stage0_busybox, &tools.stage2_busybox),
        stage1_embedded_store_paths: collect_embedded_store_paths(stage1_binary),
        stage2_embedded_store_paths: collect_embedded_store_paths(stage2_binary),
    };
    let prerequisites = collect_prerequisites(bundle_dir, proof_mode, stage0_path_dir);

    let manifest = ProofBundleManifest {
        schema: PROOF_BUNDLE_SCHEMA,
        generated_unix_s: now_unix_s(),
        repo_root: env!("CARGO_MANIFEST_DIR").to_string(),
        bundle_dir: bundle_dir.display().to_string(),
        store_dir: store_dir.display().to_string(),
        staged_source: stage2_report.staged_source.clone(),
        state_dirs: ProofStateDirs {
            stage0: stage0_state_dir.display().to_string(),
            stage2: stage2_state_dir.display().to_string(),
        },
        prerequisites,
        store_inventory: collect_store_inventory(store_dir),
        binaries,
        tools,
        fixed_point,
        stage0: ProofStageManifest {
            name: stage0.stage_name.clone(),
            original_audit_dir: stage0.audit_dir.display().to_string(),
            report: stage0_report,
            files: copy_stage_bundle_files(bundle_dir, stage0),
        },
        stage2: ProofStageManifest {
            name: stage2.stage_name.clone(),
            original_audit_dir: stage2.audit_dir.display().to_string(),
            report: stage2_report,
            files: copy_stage_bundle_files(bundle_dir, stage2),
        },
    };

    let manifest_path = bundle_dir.join("manifest.json");
    let summary_path = bundle_dir.join("summary.txt");
    let protected_audit = write_protected_exec_audit(bundle_dir, &manifest);
    let manifest_json =
        serde_json::to_vec_pretty(&manifest).unwrap_or_else(|err| panic!("serialize proof manifest: {err}"));
    std::fs::write(&manifest_path, manifest_json)
        .unwrap_or_else(|err| panic!("write proof manifest {}: {err}", manifest_path.display()));
    let summary = render_proof_bundle_summary(&manifest, &protected_audit);
    std::fs::write(&summary_path, summary)
        .unwrap_or_else(|err| panic!("write proof summary {}: {err}", summary_path.display()));
    manifest_path
}

fn sample_proof_seccomp_event() -> ProtectedSeccompAuditEvent {
    ProtectedSeccompAuditEvent {
        pid: 7,
        syscall: "execve".to_string(),
        executable_path: PathBuf::from("/seed/bin/bwrap"),
        tracee_path: PathBuf::from("/bin/bwrap"),
        resolved_host_path: PathBuf::from("/seed/bin/bwrap"),
        digest_hex: "d".repeat(64),
        reason: "declared sandbox entry".to_string(),
        phase: "protected".to_string(),
        inventory_entry_id: Some("sandbox-entry".to_string()),
        policy_decision: "allowed".to_string(),
    }
}

fn write_sample_stage0_inventory(path: &Path, sandbox_entry: &Path, sandbox_shell: &Path) {
    let inventory = Stage0Inventory {
        executable_entries: vec![
            sample_inventory_entry("sandbox-entry", "sandbox-entry", sandbox_entry),
            sample_inventory_entry("sandbox-shell", "sandbox-shell", sandbox_shell),
        ],
        source_entries: Vec::new(),
    };
    let rendered = render_stage0_inventory_nickel(&inventory).unwrap();
    std::fs::write(path, rendered).unwrap_or_else(|err| panic!("write sample inventory {}: {err}", path.display()));
}

fn sample_inventory_entry(id: &str, role: &str, path: &Path) -> ExecutableSeedEntry {
    let digest_hex = blake3::hash(&std::fs::read(path).unwrap()).to_hex().to_string();
    ExecutableSeedEntry {
        schema_version: "host-tool-free-stage0-v1".to_string(),
        id: id.to_string(),
        role: role.to_string(),
        phase: "protected".to_string(),
        executable_path: path.to_path_buf(),
        digest: DigestSpec {
            algorithm: "blake3".to_string(),
            hex: digest_hex,
            interoperability_reason: None,
        },
        provenance_category: "test-fixture".to_string(),
        provenance: "proof bundle fixture".to_string(),
        allowed_reason: format!("allow {id}"),
        owner: "bootstrap".to_string(),
        required: true,
    }
}

#[test]
fn diff_binary_bytes_reports_first_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    let left = tmp.path().join("stage1");
    let right = tmp.path().join("stage2");
    std::fs::write(&left, b"abc123").unwrap();
    std::fs::write(&right, b"abcXYZ").unwrap();

    let diff = diff_binary_bytes(&left, &right);
    assert!(diff.same_size);
    assert_eq!(diff.size_delta_bytes, 0);
    assert_eq!(diff.first_diff_offset, Some(3));
}

#[test]
fn collect_embedded_store_paths_filters_relevant_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let binary = tmp.path().join("binary");
    let payload = b"prefix /nix/store/aaa-stage0-busybox/bin/busybox\0 \
                    /nix/store/bbb-stage0-bwrap/bin/bwrap\0 \
                    /nix/store/ccc-rust/bin/rustc\0 \
                    /nix/store/ddd-stage0-crunch-src/Cargo.toml\0";
    std::fs::write(&binary, payload).unwrap();

    let paths = collect_embedded_store_paths(&binary);
    assert_eq!(paths, vec![
        "/nix/store/aaa-stage0-busybox/bin/busybox".to_string(),
        "/nix/store/bbb-stage0-bwrap/bin/bwrap".to_string(),
        "/nix/store/ddd-stage0-crunch-src".to_string(),
    ]);
}

#[test]
fn render_stage_diagnostics_includes_proof_lines_and_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tmp.path().join("store");
    let state = tmp.path().join("state");
    let logs = state.join("logs");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&logs).unwrap();
    std::fs::write(logs.join("stage.log"), "boom").unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'stdout-line\n'; printf 'self-build-proof: key=value\nstderr-line\n' >&2; exit 7")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));

    let text =
        render_stage_diagnostics("stage-x", &["crunch".to_string(), "self-build".to_string()], &output, &store, &state);

    assert!(text.contains("stage: stage-x"));
    assert!(text.contains("command: crunch self-build"));
    assert!(text.contains("self-build-proof: key=value"));
    assert!(text.contains("store entries:"));
    assert!(text.contains("log entries:"));
    assert!(text.contains("stderr tail:"));
}

#[test]
fn run_command_live_writes_stage_stream_files() {
    let proof_dir = tempfile::tempdir().unwrap();
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("printf 'stdout-1\n'; printf 'stderr-1\n' >&2; printf 'stdout-2\n'; printf 'stderr-2\n' >&2");

    let captured = run_command_live(proof_dir.path(), "live-stage", &mut command).unwrap();
    let stdout = String::from_utf8_lossy(&captured.output.stdout);
    let stderr = String::from_utf8_lossy(&captured.output.stderr);

    assert!(captured.output.status.success(), "live command should succeed");
    assert_eq!(stdout, "stdout-1\nstdout-2\n");
    assert_eq!(stderr, "stderr-1\nstderr-2\n");
    assert_eq!(std::fs::read(&captured.stdout_file).unwrap(), captured.output.stdout);
    assert_eq!(std::fs::read(&captured.stderr_file).unwrap(), captured.output.stderr);
}

#[test]
fn record_stage_evidence_writes_audit_and_diagnostics_paths() {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let output_binary = proof_dir.path().join("out");
    std::fs::write(&output_binary, b"binary").unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("printf 'self-build-proof: output-binary={}\\n' >&2", output_binary.display()))
        .output()
        .unwrap();
    assert!(output.status.success());

    let evidence = record_stage_evidence(
        proof_dir.path(),
        "stage-y",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );
    let context = stage_context(&evidence);
    let meta_json = std::fs::read_to_string(evidence.audit_dir.join("meta.json")).unwrap();

    assert!(evidence.audit_dir.exists(), "audit dir should exist on disk");
    assert!(evidence.diagnostics_file.exists(), "diagnostics file should exist on disk");
    assert!(evidence.stdout_file.exists(), "stdout file should exist on disk");
    assert!(evidence.stderr_file.exists(), "stderr file should exist on disk");
    assert!(context.contains(&evidence.audit_dir.display().to_string()));
    assert!(context.contains(&evidence.diagnostics_file.display().to_string()));
    assert!(context.contains(&evidence.stdout_file.display().to_string()));
    assert!(context.contains(&evidence.stderr_file.display().to_string()));
    assert!(context.contains("saved diagnostics snapshot:"));
    assert!(meta_json.contains("\"output-binary\""));
    assert!(meta_json.contains("\"stdout-capture\""));
    assert!(meta_json.contains("\"stderr-capture\""));
    assert!(!meta_json.contains("\"store-dir\""));
    assert!(!meta_json.contains("\"state-dir\""));
}

/// Regression: a populated store directory must NOT be hashed as an
/// audit artifact. The prior bug hashed the entire store/ tree,
/// which exceeded `MAX_AUDIT_ENTRIES` (100,000) on real self-build
/// stores and panicked between stage0 and stage2.
///
/// This test creates a store with enough structure to be representative
/// and verifies that `record_stage_evidence` succeeds without including
/// store-dir or state-dir as audit artifacts.
#[test]
fn record_stage_evidence_with_populated_store_avoids_audit_limit() {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    let logs = state.join("logs");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&logs).unwrap();

    // Simulate a self-build store: 200 hash-prefixed directories, each
    // with a bin/ subdir and a fake binary. Total entries: 200 * 3 = 600
    // (dir + bin/ + bin/file). A real store has 10k+ entries.
    let entry_count: u32 = 200;
    for i in 0..entry_count {
        let name = format!("abcdef1234567890-bootstrap-tool-{i}");
        let bin_dir = store.join(&name).join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("tool"), format!("fake-{i}")).unwrap();
    }
    // Also populate logs.
    for i in 0..50_u32 {
        std::fs::write(logs.join(format!("build-{i}.log")), format!("log-{i}")).unwrap();
    }
    // Write a pathinfo.redb stand-in.
    std::fs::write(state.join("pathinfo.redb"), b"fake-redb").unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'self-build-proof: output-binary=/tmp/fake\n' >&2")
        .output()
        .unwrap();
    assert!(output.status.success());

    // This must NOT panic. If store/ were an audit artifact, the
    // recursive hash would process 600+ entries (and on a real store,
    // 100k+, triggering the MAX_AUDIT_ENTRIES assert).
    let evidence = record_stage_evidence(
        proof_dir.path(),
        "populated-store",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );

    let meta_json = std::fs::read_to_string(evidence.audit_dir.join("meta.json")).unwrap();
    // Store and state dirs must NOT appear as artifact labels.
    // (The store path may appear in the env section — that's fine.)
    assert!(!meta_json.contains("\"store-dir\""));
    assert!(!meta_json.contains("\"state-dir\""));
    // But selected artifacts (logs-dir, pathinfo-db, diagnostics, captures) must appear.
    assert!(meta_json.contains("\"logs-dir\""));
    assert!(meta_json.contains("\"pathinfo-db\""));
    assert!(meta_json.contains("\"diagnostics\""));
    assert!(meta_json.contains("\"stdout-capture\""));
    assert!(meta_json.contains("\"stderr-capture\""));
}

#[test]
fn write_proof_bundle_copies_stage_artifacts_and_manifest() {
    let _lock = lock_proof_env();
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let stage0_state = proof_dir.path().join("state0");
    let stage2_state = proof_dir.path().join("state2");
    let staged_source = store.join("abc-crunch-src");
    let checkout_binary = proof_dir.path().join("checkout-crunch");
    let stage1_binary = proof_dir.path().join("stage1-crunch");
    let stage2_binary = proof_dir.path().join("stage2-crunch");
    let bwrap_bin_dir = store.join("abc-bwrap").join("bin");
    let busybox_bin = store.join("xyz-busybox").join("bin").join("busybox");
    let bwrap_bin = bwrap_bin_dir.join("bwrap");

    std::fs::create_dir_all(&staged_source).unwrap();
    std::fs::create_dir_all(stage0_state.join("logs")).unwrap();
    std::fs::create_dir_all(stage2_state.join("logs")).unwrap();
    std::fs::create_dir_all(&bwrap_bin_dir).unwrap();
    std::fs::create_dir_all(busybox_bin.parent().unwrap()).unwrap();
    std::fs::write(&checkout_binary, b"checkout-binary").unwrap();
    std::fs::write(&stage1_binary, b"stage1-binary").unwrap();
    std::fs::write(&stage2_binary, b"stage2-binary").unwrap();
    std::fs::write(&bwrap_bin, b"bwrap-binary").unwrap();
    std::fs::write(&busybox_bin, b"busybox-binary").unwrap();
    #[cfg(unix)]
    {
        chmod_executable(&bwrap_bin);
        chmod_executable(&busybox_bin);
    }
    std::fs::write(staged_source.join("Cargo.toml"), b"[package]\nname='proof'\n").unwrap();
    let inventory_doc = proof_dir.path().join("bootstrap-stage0-inventory.md");
    std::fs::write(&inventory_doc, b"# inventory\n").unwrap();
    let sandbox_shell = proof_dir.path().join("static-busybox");
    std::fs::write(&sandbox_shell, b"busybox-static").unwrap();
    #[cfg(unix)]
    chmod_executable(&sandbox_shell);
    let tool_dir = proof_dir.path().join("tools");
    std::fs::create_dir_all(&tool_dir).unwrap();
    for tool in HELPER_PROOF_TOOL_NAMES {
        write_executable_script(&tool_dir.join(tool), "#!/bin/sh\nset -eu\nexit 0\n");
    }
    let stage0_clean_dir = proof_dir.path().join("stage0-clean-tools");
    std::fs::create_dir_all(&stage0_clean_dir).unwrap();
    for tool in ["xz", "chmod", "bash"] {
        write_executable_script(&stage0_clean_dir.join(tool), "#!/bin/sh\nset -eu\nexit 0\n");
    }
    let stage0_inventory = proof_dir.path().join("stage0-inventory.ncl");
    write_sample_stage0_inventory(&stage0_inventory, &bwrap_bin, &busybox_bin);
    let _inventory_guard = EnvVarGuard::set(PROOF_STAGE0_INVENTORY_DOC_ENV, &inventory_doc.display().to_string());
    let _no_host_guard = EnvVarGuard::set(PROOF_NO_HOST_TOOLS_ENV, "1");
    let _stage0_inventory_guard = EnvVarGuard::set(PROOF_STAGE0_INVENTORY_ENV, &stage0_inventory.display().to_string());
    let _mode_guard = EnvVarGuard::set(PROOF_MODE_ENV, PROOF_MODE_FIXED_POINT);
    let _shell_guard = EnvVarGuard::set("SNIX_BUILD_SANDBOX_SHELL", &sandbox_shell.display().to_string());
    let _path_guard = EnvVarGuard::set("PATH", &tool_dir.display().to_string());
    let stage0_seccomp_event = serde_json::to_string(&sample_proof_seccomp_event()).unwrap();

    let stage0_output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!(
            "printf 'self-build-proof: hermeticity-mode=practical\\n' >&2; \
             printf 'self-build-proof: invoking-binary={}\\n' >&2; \
             printf 'self-build-proof: staged-source={}\\n' >&2; \
             printf 'self-build-proof: bwrap-source=crunch-built:{}\\n' >&2; \
             printf 'self-build-proof: fallback-event=bwrap-host-fallback:/run/wrappers/bin/bwrap\\n' >&2; \
             printf 'self-build-proof: fallback-event=source-host-discovery:/work/crunch\\n' >&2; \
             printf 'self-build-proof: protected-seccomp-event={}\\n' >&2; \
             printf 'self-build-proof: busybox-path={}\\n' >&2; \
             printf 'self-build-proof: output-binary={}\\n' >&2",
            checkout_binary.display(),
            staged_source.display(),
            bwrap_bin_dir.display(),
            stage0_seccomp_event,
            busybox_bin.display(),
            stage1_binary.display(),
        ))
        .output()
        .unwrap();
    let stage2_output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!(
            "printf 'self-build-proof: hermeticity-mode=strict\\n' >&2; \
             printf 'self-build-proof: invoking-binary={}\\n' >&2; \
             printf 'self-build-proof: staged-source={}\\n' >&2; \
             printf 'self-build-proof: bwrap-source=crunch-built:{}\\n' >&2; \
             printf 'self-build-proof: fallback-event=none\\n' >&2; \
             printf 'self-build-proof: busybox-path={}\\n' >&2; \
             printf 'self-build-proof: output-binary={}\\n' >&2",
            stage1_binary.display(),
            staged_source.display(),
            bwrap_bin_dir.display(),
            busybox_bin.display(),
            stage2_binary.display(),
        ))
        .output()
        .unwrap();
    assert!(stage0_output.status.success());
    assert!(stage2_output.status.success());

    let stage0_evidence = record_stage_evidence(
        proof_dir.path(),
        "stage0",
        vec!["crunch".to_string(), "self-build".to_string()],
        stage0_output,
        &store,
        &stage0_state,
    );
    let stage2_evidence = record_stage_evidence(
        proof_dir.path(),
        "stage2",
        vec![stage1_binary.display().to_string(), "self-build".to_string()],
        stage2_output,
        &store,
        &stage2_state,
    );

    let bundle_dir = proof_dir.path().join("proof-bundle");
    let manifest_path = write_proof_bundle(
        &bundle_dir,
        &stage0_evidence,
        &stage2_evidence,
        &store,
        &stage0_state,
        &stage2_state,
        &stage1_binary,
        &stage2_binary,
        ProofMode::FixedPoint,
        Some(&stage0_clean_dir),
    );
    let summary_path = bundle_dir.join("summary.txt");
    let protected_audit_path = bundle_dir.join("protected-exec-audit.json");
    let manifest_json = std::fs::read_to_string(&manifest_path).unwrap();
    let protected_audit_json = std::fs::read_to_string(&protected_audit_path).unwrap();
    let summary = std::fs::read_to_string(&summary_path).unwrap();

    assert!(manifest_path.exists(), "proof manifest should exist");
    assert!(summary_path.exists(), "proof summary should exist");
    assert!(protected_audit_path.exists(), "protected exec audit should exist");
    assert!(bundle_dir.join("stage0/meta.json").exists(), "stage0 meta should be copied");
    assert!(bundle_dir.join("stage0/stderr.txt").exists(), "stage0 stderr should be copied");
    assert!(bundle_dir.join("stage2/stdout.txt").exists(), "stage2 stdout should be copied");
    assert!(bundle_dir.join("stage2/diagnostics.txt").exists(), "stage2 diagnostics should be copied");
    let bundled_stage1_binary = bundle_dir.join(PROOF_STAGE1_BINARY_RELATIVE_PATH);
    let bundled_stage2_binary = bundle_dir.join(PROOF_STAGE2_BINARY_RELATIVE_PATH);
    assert!(bundled_stage1_binary.exists(), "stage1 binary should be durably copied");
    assert!(bundled_stage2_binary.exists(), "stage2 binary should be durably copied");
    assert_eq!(std::fs::read(&bundled_stage1_binary).unwrap(), b"stage1-binary");
    assert_eq!(std::fs::read(&bundled_stage2_binary).unwrap(), b"stage2-binary");
    assert!(manifest_json.contains(PROOF_BUNDLE_SCHEMA));
    assert!(manifest_json.contains(PROOF_STAGE1_BINARY_RELATIVE_PATH));
    assert!(manifest_json.contains(PROOF_STAGE2_BINARY_RELATIVE_PATH));
    assert!(manifest_json.contains(&busybox_bin.display().to_string()));
    assert!(manifest_json.contains(&bwrap_bin.display().to_string()));
    assert!(manifest_json.contains("\"fixed_point\""));
    assert!(manifest_json.contains("\"store_inventory\""));
    assert!(manifest_json.contains("\"prerequisites\""));
    assert!(manifest_json.contains("\"mode\": "));
    assert!(manifest_json.contains("\"hermeticity_mode\""));
    assert!(manifest_json.contains("\"fallback_events\""));
    assert!(protected_audit_json.contains("crunch-protected-exec-audit-v1"));
    assert!(protected_audit_json.contains("\"blocked_host_commands\""));
    assert!(protected_audit_json.contains("\"stage0_inventory\""));
    assert!(protected_audit_json.contains(PROOF_STAGE0_INVENTORY_RELATIVE_PATH));
    assert!(protected_audit_json.contains("\"declared_seed_artifacts\""));
    assert!(protected_audit_json.contains("\"id\": \"sandbox-entry\""));
    assert!(protected_audit_json.contains("\"role\": \"sandbox-shell\""));
    assert!(protected_audit_json.contains("\"required\": true"));
    assert!(protected_audit_json.contains("\"stage0_seccomp_events\""));
    assert!(protected_audit_json.contains("\"tracee_path\""));
    assert!(protected_audit_json.contains("/bin/bwrap"));
    assert!(protected_audit_json.contains("bwrap-host-fallback"));
    assert!(protected_audit_json.contains("fixed-point-mismatch"));
    assert!(bundle_dir.join("stage0-prerequisites/inventory.md").exists());
    assert!(summary.contains("proof_mode:"));
    assert!(summary.contains("protected_exec_result: fixed-point-mismatch"));
    assert!(summary.contains("protected_exec_audit:"));
    assert!(summary.contains("stage0_inventory_copy:"));
    assert!(summary.contains("declared_seed_artifact: id=sandbox-entry role=sandbox-entry"));
    assert!(summary.contains("declared_seed_artifact: id=sandbox-shell role=sandbox-shell"));
    assert!(summary.contains("stage0_bwrap:"));
    assert!(summary.contains("stage2_bwrap:"));
    assert!(summary.contains("stage1_equals_stage2:"));
    assert!(summary.contains("stage1_embedded_store_paths:"));
    assert!(summary.contains("stage0_hermeticity_mode: practical"));
    assert!(summary.contains("stage2_hermeticity_mode: strict"));
    assert!(summary.contains("stage0_fallback_events:"));
    assert!(summary.contains("stage0_protected_transition:"));
    assert!(summary.contains("stage2_fallback_events: []"));
    assert!(summary.contains("stage2_protected_transition:"));
    assert!(summary.contains(PROOF_STAGE2_BINARY_RELATIVE_PATH));
}

struct EnvVarGuard {
    key: &'static str,
    old_value: Option<std::ffi::OsString>,
}

fn proof_env_mutex() -> &'static Mutex<()> {
    static PROOF_ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    PROOF_ENV_MUTEX.get_or_init(|| Mutex::new(()))
}

fn lock_proof_env() -> MutexGuard<'static, ()> {
    proof_env_mutex().lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let old_value = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, old_value }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.old_value.as_ref() {
            Some(value) => unsafe {
                std::env::set_var(self.key, value);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

#[cfg(unix)]
fn chmod_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).unwrap_or_else(|err| panic!("stat {}: {err}", path.display()));
    let mut permissions = metadata.permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions).unwrap_or_else(|err| panic!("chmod {}: {err}", path.display()));
}

#[cfg(unix)]
fn chmod_executable(path: &Path) {
    chmod_mode(path, 0o755);
}

#[cfg(unix)]
fn write_executable_script(path: &Path, body: &str) {
    let parent = path.parent().unwrap_or_else(|| panic!("script path has no parent: {}", path.display()));
    std::fs::create_dir_all(parent).unwrap_or_else(|err| panic!("mkdir {}: {err}", parent.display()));
    std::fs::write(path, body).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    chmod_executable(path);
}

#[cfg(unix)]
struct ProofScriptFixture {
    _temp: tempfile::TempDir,
    repo_dir: PathBuf,
    tool_dir: PathBuf,
}

#[cfg(unix)]
impl ProofScriptFixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo_dir = temp.path().join("repo");
        let tool_dir = temp.path().join("tools");
        let script_dir = repo_dir.join("scripts");
        let docs_dir = repo_dir.join("docs");
        std::fs::create_dir_all(&script_dir).unwrap();
        std::fs::create_dir_all(&docs_dir).unwrap();
        std::fs::create_dir_all(repo_dir.join("bootstrap")).unwrap();
        std::fs::write(repo_dir.join("Cargo.toml"), "[package]\nname='proof-fixture'\nversion='0.0.0'\n").unwrap();
        std::fs::write(docs_dir.join("bootstrap-stage0-inventory.md"), "# fixture inventory\n").unwrap();

        let src_script = repo_root().join("scripts/prove-self-hosting.sh");
        let dst_script = script_dir.join("prove-self-hosting.sh");
        std::fs::copy(&src_script, &dst_script).unwrap();
        chmod_executable(&dst_script);

        let cargo_body = r#"#!/bin/sh
set -eu
if [ "${1:-}" = "run" ]; then
  out=""
  while [ "$#" -gt 0 ]; do
    if [ "$1" = "--output" ]; then
      shift
      out="${1:?}"
      break
    fi
    shift
  done
  [ -n "$out" ] || { printf 'missing stage0 inventory output\n' >&2; exit 1; }
  mkdir -p "${out%/*}"
  printf '{ executable_entries = [], source_entries = [] }\n' > "$out"
  printf 'stage0 inventory written: %s\n' "$out"
  printf 'seed closure risk: fixture\n'
  exit 0
fi
: "${CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR:?}"
if [ -n "${CRUNCH_TEST_PROOF_COMMAND_SENTINEL:-}" ]; then
  printf 'launched\n' > "$CRUNCH_TEST_PROOF_COMMAND_SENTINEL"
fi
mkdir -p "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR"
printf '{"schema":"fake-proof"}\n' > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/manifest.json"
printf 'summary\n' > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/summary.txt"
printf '%s\n' "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/env-path.txt"
printf '%s\n' "$PWD" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/cwd.txt"
printf '%s\n' "$*" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/argv.txt"
printf '%s\n' "${CRUNCH_SELF_HOSTING_PROOF_MODE:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/proof-mode.txt"
printf '%s\n' "${CRUNCH_SELF_HOSTING_STAGE0_INVENTORY_DOC:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/inventory-doc.txt"
printf '%s\n' "${CRUNCH_SELF_HOSTING_NO_HOST_TOOLS:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/no-host-tools.txt"
printf '%s\n' "${CRUNCH_SELF_HOSTING_STAGE0_INVENTORY:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/stage0-inventory.txt"
printf '%s\n' "${CRUNCH_SELF_HOSTING_BLOCKED_HOST_TOOLS:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/blocked-host-tools.txt"
printf '%s\n' "${CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/later-stage-hermeticity.txt"
printf '%s\n' "${SNIX_BUILD_SANDBOX_SHELL:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/sandbox-shell.txt"
printf '%s\n' "${TMPDIR:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/tmpdir.txt"
printf '%s\n' "${CARGO_TARGET_DIR:-}" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/cargo-target-dir.txt"
printf '%s\n' "$PATH" > "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/path.txt"
for blocked in nix-build nix-store nix-shell nix; do
  if command -v "$blocked" >/dev/null 2>&1; then
    printf '%s\n' "$blocked" >> "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/blocked-tools-found.txt"
  fi
done
for blocked in ${CRUNCH_SELF_HOSTING_BLOCKED_HOST_TOOLS:-}; do
  if command -v "$blocked" >/dev/null 2>&1; then
    printf '%s\n' "$blocked" >> "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR/blocked-host-tools-found.txt"
  fi
done
"#;
        write_executable_script(&tool_dir.join("cargo"), cargo_body);
        write_executable_script(
            &tool_dir.join("rustc"),
            "#!/bin/sh\nset -eu\nif [ \"${1:-}\" = \"--version\" ]; then\n  printf 'rustc 1.99.0-nightly (fake)\\n'\n  exit 0\nfi\nexit 0\n",
        );
        let rustup_body = format!(
            "#!/bin/sh\nset -eu\nif [ \"${{1:-}}\" = \"which\" ] && [ \"${{2:-}}\" = \"--toolchain\" ] && [ \"${{3:-}}\" = \"nightly\" ]; then\n  case \"${{4:-}}\" in\n    cargo) printf '%s\\n' \"{}\" ;;\n    rustc) printf '%s\\n' \"{}\" ;;\n    *) exit 1 ;;\n  esac\n  exit 0\nfi\nexit 1\n",
            tool_dir.join("cargo").display(),
            tool_dir.join("rustc").display(),
        );
        write_executable_script(&tool_dir.join("rustup"), &rustup_body);
        write_executable_script(
            &tool_dir.join("pkg-config"),
            "#!/bin/sh\nset -eu\nif [ \"${1:-}\" = \"--exists\" ] && [ \"${2:-}\" = \"openssl\" ]; then\n  exit 0\nfi\nif [ \"${1:-}\" = \"--modversion\" ] && [ \"${2:-}\" = \"openssl\" ]; then\n  printf '3.6.1\\n'\n  exit 0\nfi\nexit 1\n",
        );
        write_executable_script(&tool_dir.join("stat"), "#!/bin/sh\nset -eu\nprintf '5000000 1024\\n'\n");
        for tool in ["clang", "mold", "git", "tar", "xz", "cp", "bwrap"] {
            write_executable_script(&tool_dir.join(tool), "#!/bin/sh\nset -eu\nexit 0\n");
        }
        for blocked in BLOCKED_NIX_BINARIES {
            write_executable_script(&tool_dir.join(blocked), "#!/bin/sh\nset -eu\nexit 0\n");
        }
        write_executable_script(&tool_dir.join("bash"), "#!/bin/sh\nexec /bin/sh \"$@\"\n");
        write_executable_script(&tool_dir.join("static-sh"), "#!/bin/sh\nset -eu\nexit 0\n");

        Self {
            _temp: temp,
            repo_dir,
            tool_dir,
        }
    }

    fn script_path(&self) -> PathBuf {
        self.repo_dir.join("scripts/prove-self-hosting.sh")
    }

    fn ambient_tmpdir(&self) -> PathBuf {
        self.repo_dir.join("tmp")
    }

    fn ambient_cargo_target_dir(&self) -> PathBuf {
        self.repo_dir.join("ambient-cargo-target")
    }

    fn default_scratch_root(&self) -> PathBuf {
        self.repo_dir.join("target/self-hosting-proof/work")
    }

    fn set_stat_output(&self, free_blocks: u64, block_size: u64) {
        let body = format!("#!/bin/sh\nset -eu\nprintf '{free_blocks} {block_size}\\n'\n");
        write_executable_script(&self.tool_dir.join("stat"), &body);
    }

    fn read_bundle_text(&self, bundle_dir: &Path, file_name: &str) -> String {
        std::fs::read_to_string(bundle_dir.join(file_name)).unwrap().trim().to_string()
    }

    fn proof_launch_sentinel(&self, name: &str) -> PathBuf {
        self.repo_dir.join(format!("{name}-proof-command-launched.txt"))
    }

    fn base_command_in_cwd(&self, cwd: &Path) -> std::process::Command {
        let host_path = std::env::var("PATH").unwrap_or_default();
        let fake_path = if host_path.is_empty() {
            self.tool_dir.display().to_string()
        } else {
            format!("{}:{host_path}", self.tool_dir.display())
        };
        let mut command = std::process::Command::new(self.script_path());
        command
            .current_dir(cwd)
            .env("PATH", fake_path)
            .env("HOME", self.repo_dir.join("home"))
            .env("TMPDIR", self.ambient_tmpdir())
            .env("CARGO_TARGET_DIR", self.ambient_cargo_target_dir())
            .env("SNIX_BUILD_SANDBOX_SHELL", self.tool_dir.join("static-sh"));
        command
    }

    fn run_args(&self, args: &[&str]) -> std::process::Output {
        self.run_args_with_envs(args, &[])
    }

    fn run_args_with_envs(&self, args: &[&str], extra_envs: &[(&str, String)]) -> std::process::Output {
        self.run_args_from_cwd_with_envs(&self.repo_dir, args, extra_envs)
    }

    fn run_args_from_cwd_with_envs(
        &self,
        cwd: &Path,
        args: &[&str],
        extra_envs: &[(&str, String)],
    ) -> std::process::Output {
        let _lock = lock_proof_env();
        let mut command = self.base_command_in_cwd(cwd);
        for (key, value) in extra_envs {
            command.env(key, value);
        }
        for arg in args {
            command.arg(arg);
        }
        command.output().unwrap()
    }

    fn run(&self, bundle_arg: &Path) -> std::process::Output {
        let bundle_arg_owned = bundle_arg.to_string_lossy().into_owned();
        self.run_args(&["--bundle-dir", &bundle_arg_owned])
    }
}

#[test]
fn resolve_proof_bundle_dir_anchors_relative_env_to_repo_root() {
    let _lock = lock_proof_env();
    let _guard = EnvVarGuard::set(PROOF_BUNDLE_ENV, "target/custom-proof");
    let resolved = resolve_proof_bundle_dir();

    assert_eq!(resolved, repo_root().join("target/custom-proof"));
}

#[test]
fn resolve_proof_bundle_dir_preserves_absolute_env_path() {
    let _lock = lock_proof_env();
    let tmp = tempfile::tempdir().unwrap();
    let absolute = tmp.path().join("proof-bundle");
    let _guard = EnvVarGuard::set(PROOF_BUNDLE_ENV, &absolute.display().to_string());
    let resolved = resolve_proof_bundle_dir();

    assert_eq!(resolved, absolute);
}

#[test]
fn proof_mode_defaults_to_fixed_point() {
    let _lock = lock_proof_env();
    let _guard = EnvVarGuard::set(PROOF_MODE_ENV, PROOF_MODE_FIXED_POINT);
    assert_eq!(ProofMode::current(), ProofMode::FixedPoint);
    assert!(!ProofMode::current().stage0_path_is_scrubbed());
}

#[test]
fn append_no_host_tools_stage0_args_preserves_base_command_and_adds_inventory() {
    let temp = tempfile::tempdir().unwrap();
    let inventory = temp.path().join("stage0-inventory.ncl");
    std::fs::write(&inventory, "# inventory\n").unwrap();
    let mut command = vec!["crunch".to_string(), "self-build".to_string()];

    append_no_host_tools_stage0_args(&mut command, Some(&inventory));

    assert_eq!(command[0], "crunch");
    assert_eq!(command[1], "self-build");
    assert_eq!(command[2], "--no-host-tools");
    assert_eq!(command[3], "--stage0-inventory");
    assert_eq!(command[4], inventory.display().to_string());
}

#[test]
fn filtered_proof_tool_names_removes_blocked_host_tools_for_no_host_mode() {
    let filtered = filtered_proof_tool_names(&STAGE0_PROOF_TOOL_NAMES, true);

    assert!(filtered.contains(&"xz"));
    assert!(filtered.contains(&"chmod"));
    assert!(filtered.contains(&"bash"));
    for blocked in ["bwrap", "git", "cargo", "tar", "cp"] {
        assert!(!filtered.contains(&blocked), "no-host-tools stage0 tool record must omit {blocked}");
    }
}

#[cfg(unix)]
#[test]
fn create_stage0_scrubbed_path_blocks_nix_binaries() {
    let _lock = lock_proof_env();
    let temp = tempfile::tempdir().unwrap();
    let tool_dir = temp.path().join("tools");
    let nix_dir = temp.path().join("nix-tools");
    std::fs::create_dir_all(&tool_dir).unwrap();
    std::fs::create_dir_all(&nix_dir).unwrap();
    for tool in STAGE0_PROOF_TOOL_NAMES {
        write_executable_script(&tool_dir.join(tool), "#!/bin/sh\nset -eu\nexit 0\n");
    }
    for blocked in BLOCKED_NIX_BINARIES {
        write_executable_script(&nix_dir.join(blocked), "#!/bin/sh\nset -eu\nexit 0\n");
    }
    let joined = std::env::join_paths([tool_dir.clone(), nix_dir.clone()]).unwrap();
    let _path_guard = EnvVarGuard::set("PATH", &joined.to_string_lossy());

    let scrubbed_dir = create_stage0_scrubbed_path(temp.path());
    let scrubbed_path = std::env::join_paths([scrubbed_dir.clone()]).unwrap();
    for blocked in BLOCKED_NIX_BINARIES {
        assert!(find_executable_in_path_var(blocked, scrubbed_path.as_os_str()).is_none());
    }
    for tool in STAGE0_PROOF_TOOL_NAMES {
        assert!(find_executable_in_path_var(tool, scrubbed_path.as_os_str()).is_some());
    }
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_reports_default_scratch_policy_in_check_mode() {
    let fixture = ProofScriptFixture::new();
    let scratch_root = fixture.default_scratch_root();
    let scratch_tmp = scratch_root.join("tmp");
    let scratch_cargo_target = scratch_root.join("cargo-target");
    assert!(!scratch_root.exists(), "default scratch root should start absent in the fixture");

    let output = fixture.run_args(&["--check"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(stderr.contains(&format!("proof scratch root: {}", scratch_root.display())));
    assert!(stderr.contains(&format!("proof scratch source: {DEFAULT_PROOF_SCRATCH_SOURCE}")));
    assert!(stderr.contains(&format!("proof TMPDIR: {}", scratch_tmp.display())));
    assert!(stderr.contains(&format!("proof CARGO_TARGET_DIR: {}", scratch_cargo_target.display())));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_creates_selected_scratch_root_when_missing() {
    let fixture = ProofScriptFixture::new();
    let override_root = fixture.repo_dir.join("new-proof-scratch");
    assert!(!override_root.exists(), "override scratch root should start absent in the fixture");

    let output = fixture.run_args_with_envs(&["--check"], &[(PROOF_SCRATCH_ENV, "new-proof-scratch".to_string())]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(override_root.is_dir(), "helper must create the selected scratch root");
    assert!(override_root.join("tmp").is_dir(), "helper must create the selected TMPDIR subdir");
    assert!(override_root.join("cargo-target").is_dir(), "helper must create the selected cargo-target subdir");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_routes_spawned_tmpdir_and_cargo_target_under_default_scratch_root() {
    let fixture = ProofScriptFixture::new();
    let bundle_arg = Path::new("target/default-scratch-bundle");
    let bundle_dir = fixture.repo_dir.join(bundle_arg);
    let output = fixture.run(bundle_arg);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let scratch_root = fixture.default_scratch_root();
    let scratch_tmp = scratch_root.join("tmp");
    let scratch_cargo_target = scratch_root.join("cargo-target");

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert_eq!(fixture.read_bundle_text(&bundle_dir, "tmpdir.txt"), scratch_tmp.display().to_string());
    assert_eq!(
        fixture.read_bundle_text(&bundle_dir, "cargo-target-dir.txt"),
        scratch_cargo_target.display().to_string()
    );
    assert_ne!(fixture.read_bundle_text(&bundle_dir, "tmpdir.txt"), fixture.ambient_tmpdir().display().to_string());
    assert_ne!(
        fixture.read_bundle_text(&bundle_dir, "cargo-target-dir.txt"),
        fixture.ambient_cargo_target_dir().display().to_string()
    );
    assert!(stderr.contains(&format!("proof scratch source: {DEFAULT_PROOF_SCRATCH_SOURCE}")));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_reports_scratch_override_in_check_mode() {
    let fixture = ProofScriptFixture::new();
    let override_root = fixture.repo_dir.join("custom-proof-scratch");
    let output = fixture.run_args_with_envs(&["--check"], &[(PROOF_SCRATCH_ENV, "custom-proof-scratch".to_string())]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(stderr.contains(&format!("proof scratch root: {}", override_root.display())));
    assert!(stderr.contains(&format!("proof scratch source: {PROOF_SCRATCH_ENV}")));
    assert!(stderr.contains(&format!("proof TMPDIR: {}", override_root.join("tmp").display())));
    assert!(stderr.contains(&format!("proof CARGO_TARGET_DIR: {}", override_root.join("cargo-target").display())));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_preserves_absolute_scratch_override_path() {
    let fixture = ProofScriptFixture::new();
    let override_root = fixture._temp.path().join("absolute-proof-scratch");
    let output = fixture.run_args_with_envs(&["--check"], &[(PROOF_SCRATCH_ENV, override_root.display().to_string())]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(stderr.contains(&format!("proof scratch root: {}", override_root.display())));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_anchors_relative_scratch_override_to_repo_root_from_non_repo_cwd() {
    let fixture = ProofScriptFixture::new();
    let outside_cwd = fixture._temp.path().join("outside-cwd");
    std::fs::create_dir_all(&outside_cwd).unwrap();
    let override_root = fixture.repo_dir.join("anchored-proof-scratch");

    let output = fixture.run_args_from_cwd_with_envs(&outside_cwd, &["--check"], &[(
        PROOF_SCRATCH_ENV,
        "anchored-proof-scratch".to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(stderr.contains(&format!("proof scratch root: {}", override_root.display())));
    assert!(
        !stderr.contains(&format!("proof scratch root: {}", outside_cwd.join("anchored-proof-scratch").display())),
        "relative scratch override must anchor to repo root, not caller cwd"
    );
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_anchors_default_scratch_root_to_repo_root_from_non_repo_cwd() {
    let fixture = ProofScriptFixture::new();
    let outside_cwd = fixture._temp.path().join("outside-cwd-default");
    std::fs::create_dir_all(&outside_cwd).unwrap();
    let default_root = fixture.default_scratch_root();

    let output = fixture.run_args_from_cwd_with_envs(&outside_cwd, &["--check"], &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(stderr.contains(&format!("proof scratch root: {}", default_root.display())));
    assert!(
        !stderr
            .contains(&format!("proof scratch root: {}", outside_cwd.join("target/self-hosting-proof/work").display())),
        "default scratch root must anchor to repo root, not caller cwd"
    );
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_create_failure_scratch_override_without_fallback() {
    let fixture = ProofScriptFixture::new();
    let blocked_parent = fixture.repo_dir.join("blocked-parent");
    let bundle_dir = fixture.repo_dir.join("target/create-failure-override-bundle");
    let launch_sentinel = fixture.proof_launch_sentinel("create-failure-override");
    std::fs::create_dir_all(&blocked_parent).unwrap();
    chmod_mode(&blocked_parent, 0o555);

    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/create-failure-override-bundle"], &[
        (PROOF_SCRATCH_ENV, "blocked-parent/child-scratch".to_string()),
        (PROOF_COMMAND_SENTINEL_ENV, launch_sentinel.display().to_string()),
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    chmod_mode(&blocked_parent, 0o755);

    assert!(!output.status.success(), "script should fail when override scratch root cannot be created");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&fixture.repo_dir.join("blocked-parent/child-scratch").display().to_string()));
    assert!(
        !fixture.default_scratch_root().exists(),
        "default scratch root must not be created when explicit override cannot be created"
    );
    assert!(!launch_sentinel.exists(), "proof command must not launch on override create failure");
    assert!(!bundle_dir.exists(), "bundle dir must stay absent on override create failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_unusable_scratch_override_without_fallback() {
    let fixture = ProofScriptFixture::new();
    let blocked_path = fixture.repo_dir.join("blocked-scratch");
    let bundle_dir = fixture.repo_dir.join("target/non-directory-override-bundle");
    let launch_sentinel = fixture.proof_launch_sentinel("non-directory-override");
    std::fs::write(&blocked_path, "blocked\n").unwrap();
    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/non-directory-override-bundle"], &[
        (PROOF_SCRATCH_ENV, "blocked-scratch".to_string()),
        (PROOF_COMMAND_SENTINEL_ENV, launch_sentinel.display().to_string()),
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should fail for unusable override");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&blocked_path.display().to_string()));
    assert!(
        !fixture.default_scratch_root().exists(),
        "default scratch root must not be created when explicit override fails"
    );
    assert!(!launch_sentinel.exists(), "proof command must not launch on override non-directory failure");
    assert!(!bundle_dir.exists(), "bundle dir must stay absent on override non-directory failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_unwritable_scratch_override_without_fallback() {
    let fixture = ProofScriptFixture::new();
    let unwritable_root = fixture.repo_dir.join("unwritable-override-scratch");
    let bundle_dir = fixture.repo_dir.join("target/unwritable-override-bundle");
    let launch_sentinel = fixture.proof_launch_sentinel("unwritable-override");
    std::fs::create_dir_all(&unwritable_root).unwrap();
    chmod_mode(&unwritable_root, 0o555);

    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/unwritable-override-bundle"], &[
        (PROOF_SCRATCH_ENV, "unwritable-override-scratch".to_string()),
        (PROOF_COMMAND_SENTINEL_ENV, launch_sentinel.display().to_string()),
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    chmod_mode(&unwritable_root, 0o755);

    assert!(!output.status.success(), "script should fail for unwritable override");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&unwritable_root.display().to_string()));
    assert!(
        !fixture.default_scratch_root().exists(),
        "default scratch root must not be created when unwritable override fails"
    );
    assert!(!launch_sentinel.exists(), "proof command must not launch on override unwritable failure");
    assert!(!bundle_dir.exists(), "bundle dir must stay absent on override unwritable failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_create_failure_default_scratch_before_proof_work() {
    let fixture = ProofScriptFixture::new();
    let bundle_dir = fixture.repo_dir.join("target/create-failure-default-bundle");
    let launch_sentinel = fixture.proof_launch_sentinel("create-failure-default");
    std::fs::create_dir_all(fixture.repo_dir.join("target")).unwrap();
    std::fs::write(fixture.repo_dir.join("target/self-hosting-proof"), "blocked-parent\n").unwrap();
    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/create-failure-default-bundle"], &[(
        PROOF_COMMAND_SENTINEL_ENV,
        launch_sentinel.display().to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should fail when default scratch root cannot be created");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&fixture.default_scratch_root().display().to_string()));
    assert!(!launch_sentinel.exists(), "proof command must not launch on default create failure");
    assert!(!bundle_dir.exists(), "bundle dir must stay absent on default create failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_unusable_default_scratch_before_proof_work() {
    let fixture = ProofScriptFixture::new();
    let default_root = fixture.default_scratch_root();
    let bundle_dir = fixture.repo_dir.join("target/non-directory-default-bundle");
    let launch_sentinel = fixture.proof_launch_sentinel("non-directory-default");
    std::fs::create_dir_all(default_root.parent().unwrap()).unwrap();
    std::fs::write(&default_root, "blocked\n").unwrap();
    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/non-directory-default-bundle"], &[(
        PROOF_COMMAND_SENTINEL_ENV,
        launch_sentinel.display().to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should fail for unusable default scratch root");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&default_root.display().to_string()));
    assert!(!launch_sentinel.exists(), "proof command must not launch on default non-directory failure");
    assert!(!bundle_dir.exists(), "bundle dir must stay absent on default non-directory failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_unwritable_default_scratch_before_proof_work() {
    let fixture = ProofScriptFixture::new();
    let default_root = fixture.default_scratch_root();
    let bundle_dir = fixture.repo_dir.join("target/unwritable-default-bundle");
    let launch_sentinel = fixture.proof_launch_sentinel("unwritable-default");
    std::fs::create_dir_all(&default_root).unwrap();
    chmod_mode(&default_root, 0o555);

    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/unwritable-default-bundle"], &[(
        PROOF_COMMAND_SENTINEL_ENV,
        launch_sentinel.display().to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    chmod_mode(&default_root, 0o755);

    assert!(!output.status.success(), "script should fail for unwritable default scratch root");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&default_root.display().to_string()));
    assert!(!launch_sentinel.exists(), "proof command must not launch on default unwritable failure");
    assert!(!bundle_dir.exists(), "bundle dir must stay absent on default unwritable failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_below_threshold_scratch_before_proof_work() {
    let fixture = ProofScriptFixture::new();
    let launch_sentinel = fixture.proof_launch_sentinel("below-threshold-default");
    fixture.set_stat_output(4_194_303, 1024);
    let bundle_arg = Path::new("target/too-small-scratch-bundle");
    let bundle_dir = fixture.repo_dir.join(bundle_arg);
    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/too-small-scratch-bundle"], &[(
        PROOF_COMMAND_SENTINEL_ENV,
        launch_sentinel.display().to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should fail for undersized proof scratch root");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&fixture.default_scratch_root().display().to_string()));
    assert!(
        !bundle_dir.exists(),
        "proof bundle dir must stay absent when preflight blocks the run before proof work"
    );
    assert!(!launch_sentinel.exists(), "proof command must not launch on default low-space failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_below_threshold_override_scratch_before_proof_work() {
    let fixture = ProofScriptFixture::new();
    let override_root = fixture.repo_dir.join("too-small-override-scratch");
    let launch_sentinel = fixture.proof_launch_sentinel("below-threshold-override");
    let bundle_arg = Path::new("target/too-small-override-scratch-bundle");
    let bundle_dir = fixture.repo_dir.join(bundle_arg);
    fixture.set_stat_output(4_194_303, 1024);
    let bundle_arg_owned = bundle_arg.to_string_lossy().into_owned();
    let output = fixture.run_args_with_envs(&["--bundle-dir", &bundle_arg_owned], &[
        (PROOF_SCRATCH_ENV, "too-small-override-scratch".to_string()),
        (PROOF_COMMAND_SENTINEL_ENV, launch_sentinel.display().to_string()),
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should fail for undersized override scratch root");
    assert!(stderr.contains(PROOF_SCRATCH_ENV));
    assert!(stderr.contains(&override_root.display().to_string()));
    assert!(
        !bundle_dir.exists(),
        "proof bundle dir must stay absent when override preflight blocks the run before proof work"
    );
    assert!(!launch_sentinel.exists(), "proof command must not launch on override low-space failure");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_accepts_exact_scratch_threshold() {
    let fixture = ProofScriptFixture::new();
    fixture.set_stat_output(4_194_304, 1024);
    let output = fixture.run_args(&["--check"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should accept exact threshold, stderr:\n{stderr}");
    assert!(stderr.contains("proof scratch free: 4096 MiB"));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_anchors_relative_bundle_dir_and_updates_latest() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let bundle_arg = Path::new("target/custom-relative-bundle");
    let expected_bundle = fixture.repo_dir.join(bundle_arg);
    let output = fixture.run(bundle_arg);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let latest_link = fixture.repo_dir.join("target/self-hosting-proof/latest");

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(expected_bundle.join("manifest.json").exists(), "manifest should exist in anchored bundle dir");
    assert!(expected_bundle.join("summary.txt").exists(), "summary should exist in anchored bundle dir");
    assert_eq!(std::fs::read_link(&latest_link).unwrap(), expected_bundle);
    assert_eq!(
        std::fs::read_to_string(expected_bundle.join("env-path.txt")).unwrap().trim(),
        expected_bundle.display().to_string()
    );
    assert_eq!(
        std::fs::read_to_string(expected_bundle.join("cwd.txt")).unwrap().trim(),
        fixture.repo_dir.display().to_string()
    );
    assert!(stderr.contains(&format!("proof bundle: {}", expected_bundle.display())));
    assert!(
        stderr.contains(&format!("latest bundle: {}/target/self-hosting-proof/latest", fixture.repo_dir.display()))
    );
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_preserves_absolute_bundle_dir_and_updates_latest() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let absolute_bundle = fixture.repo_dir.join("outside-bundle");
    let output = fixture.run(&absolute_bundle);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let latest_link = fixture.repo_dir.join("target/self-hosting-proof/latest");

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(absolute_bundle.join("manifest.json").exists(), "manifest should exist in absolute bundle dir");
    assert!(absolute_bundle.join("summary.txt").exists(), "summary should exist in absolute bundle dir");
    assert_eq!(std::fs::read_link(&latest_link).unwrap(), absolute_bundle);
    assert_eq!(
        std::fs::read_to_string(absolute_bundle.join("env-path.txt")).unwrap().trim(),
        absolute_bundle.display().to_string()
    );
    assert!(stderr.contains(&format!("proof bundle: {}", absolute_bundle.display())));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_uses_bundle_dir_env_when_cli_arg_absent() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let bundle_arg = "target/env-selected-bundle";
    let expected_bundle = fixture.repo_dir.join(bundle_arg);
    let output = fixture.run_args_with_envs(&[], &[(PROOF_BUNDLE_ENV, bundle_arg.to_string())]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let latest_link = fixture.repo_dir.join("target/self-hosting-proof/latest");

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(expected_bundle.join("manifest.json").exists(), "env-selected bundle must receive manifest");
    assert!(expected_bundle.join("summary.txt").exists(), "env-selected bundle must receive summary");
    assert_eq!(std::fs::read_link(&latest_link).unwrap(), expected_bundle);
    assert_eq!(
        std::fs::read_to_string(expected_bundle.join("env-path.txt")).unwrap().trim(),
        expected_bundle.display().to_string()
    );
    assert!(stderr.contains(&format!("proof bundle: {}", expected_bundle.display())));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_exports_non_nix_host_mode_and_inventory_doc() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let bundle_arg = Path::new("target/non-nix-proof");
    let bundle_dir = fixture.repo_dir.join(bundle_arg);
    let output = fixture.run_args(&["--non-nix-host", "--bundle-dir", "target/non-nix-proof"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert_eq!(std::fs::read_to_string(bundle_dir.join("proof-mode.txt")).unwrap().trim(), PROOF_MODE_NON_NIX_HOST);
    assert_eq!(
        std::fs::read_to_string(bundle_dir.join("inventory-doc.txt")).unwrap().trim(),
        fixture.repo_dir.join("docs/bootstrap-stage0-inventory.md").display().to_string()
    );
    assert_eq!(std::fs::read_to_string(bundle_dir.join("later-stage-hermeticity.txt")).unwrap().trim(), "strict");
    assert!(!bundle_dir.join("blocked-tools-found.txt").exists(), "blocked nix tools must stay off helper PATH");
    assert!(stderr.contains("proof mode: non-nix-host"));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_exports_no_host_tools_inventory_and_blocks_host_tools() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let inventory = fixture.repo_dir.join("bootstrap/stage0-inventory.ncl");
    std::fs::write(&inventory, "# fixture inventory\n").unwrap();
    let bundle_dir = fixture.repo_dir.join("target/no-host-tools-proof");

    let output = fixture.run_args(&[
        "--no-host-tools",
        "--stage0-inventory",
        "bootstrap/stage0-inventory.ncl",
        "--bundle-dir",
        "target/no-host-tools-proof",
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert_eq!(std::fs::read_to_string(bundle_dir.join("no-host-tools.txt")).unwrap().trim(), "1");
    assert_eq!(
        std::fs::read_to_string(bundle_dir.join("stage0-inventory.txt")).unwrap().trim(),
        inventory.display().to_string()
    );
    let blocked = std::fs::read_to_string(bundle_dir.join("blocked-host-tools.txt")).unwrap();
    for tool in [
        "git",
        "tar",
        "cp",
        "sh",
        "cargo",
        "bwrap",
        "nix",
        "nix-build",
        "nix-store",
        "nix-shell",
    ] {
        assert!(blocked.split_whitespace().any(|found| found == tool), "blocked set must contain {tool}");
    }
    assert!(
        !bundle_dir.join("blocked-host-tools-found.txt").exists(),
        "blocked host tools must stay off helper PATH"
    );
    assert!(stderr.contains("no-host-tools stage0 inventory"), "summary must name inventory, stderr:\n{stderr}");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_generates_no_host_tools_inventory_from_explicit_seeds() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let inventory = fixture.repo_dir.join("target/generated-stage0-inventory.ncl");
    let bundle_dir = fixture.repo_dir.join("target/generated-inventory-proof");

    let output = fixture.run_args_with_envs(
        &[
            "--generate-stage0-inventory",
            "target/generated-stage0-inventory.ncl",
            "--bundle-dir",
            "target/generated-inventory-proof",
        ],
        &[
            ("CRUNCH_STAGE0_SEED_SANDBOX_ENTRY", fixture.tool_dir.join("bwrap").display().to_string()),
            ("CRUNCH_STAGE0_SEED_SANDBOX_SHELL", fixture.tool_dir.join("static-sh").display().to_string()),
            ("CRUNCH_STAGE0_SEED_TOOLCHAIN_ROOT", fixture.tool_dir.display().to_string()),
            ("CRUNCH_STAGE0_SEED_BUILD_TOOLS", fixture.tool_dir.display().to_string()),
        ],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert!(inventory.exists(), "generated inventory should exist: {}", inventory.display());
    assert_eq!(std::fs::read_to_string(bundle_dir.join("no-host-tools.txt")).unwrap().trim(), "1");
    assert_eq!(
        std::fs::read_to_string(bundle_dir.join("stage0-inventory.txt")).unwrap().trim(),
        inventory.display().to_string()
    );
    assert!(stderr.contains("stage0 inventory seed policy: explicit CRUNCH_STAGE0_SEED_* paths only"));
    assert!(stderr.contains("no PATH or /nix/store discovery"));
    assert!(stderr.contains(&format!("no-host-tools stage0 inventory: {}", inventory.display())));
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_no_host_tools_requires_inventory_before_launch() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let launch_sentinel = fixture.proof_launch_sentinel("no-host-tools-missing-inventory");

    let output = fixture.run_args_with_envs(&["--no-host-tools", "--bundle-dir", "target/no-host-tools-missing"], &[(
        PROOF_COMMAND_SENTINEL_ENV,
        launch_sentinel.display().to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should fail without inventory");
    assert!(stderr.contains("--no-host-tools requires --stage0-inventory FILE"), "stderr:\n{stderr}");
    assert!(!launch_sentinel.exists(), "proof command must not launch before inventory validation");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let bundle_arg = Path::new("target/default-proof");
    let bundle_dir = fixture.repo_dir.join(bundle_arg);

    let output = fixture.run(bundle_arg);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert_eq!(std::fs::read_to_string(bundle_dir.join("later-stage-hermeticity.txt")).unwrap().trim(), "strict");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_discovers_repo_local_default_sandbox_shell_when_env_is_bin_sh() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let busybox_root = fixture.repo_dir.join("target/proof-busybox-static/bin");
    std::fs::create_dir_all(&busybox_root).unwrap();
    write_executable_script(&busybox_root.join("busybox"), "#!/bin/sh\nset -eu\nexit 0\n");
    let nix_build_sentinel = fixture.proof_launch_sentinel("nix-build-default-shell");
    let nix_build_body =
        format!("#!/bin/sh\nset -eu\nprintf 'called\\n' > \"{}\"\nexit 99\n", nix_build_sentinel.display());
    write_executable_script(&fixture.tool_dir.join("nix-build"), &nix_build_body);
    let bundle_dir = fixture.repo_dir.join("target/default-discovered-shell-proof");

    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/default-discovered-shell-proof"], &[(
        "SNIX_BUILD_SANDBOX_SHELL",
        "/bin/sh".to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert_eq!(
        std::fs::read_to_string(bundle_dir.join("sandbox-shell.txt")).unwrap().trim(),
        fixture.repo_dir.join("target/proof-busybox-static/bin/busybox").display().to_string()
    );
    assert!(!nix_build_sentinel.exists(), "default sandbox-shell discovery must not invoke nix-build");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_fails_fast_when_default_sandbox_shell_missing_without_nix_build_fallback() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let nix_build_sentinel = fixture.proof_launch_sentinel("nix-build-missing-shell");
    let nix_build_body =
        format!("#!/bin/sh\nset -eu\nprintf 'called\\n' > \"{}\"\nexit 99\n", nix_build_sentinel.display());
    write_executable_script(&fixture.tool_dir.join("nix-build"), &nix_build_body);

    let output = fixture.run_args_with_envs(&["--check"], &[
        ("SNIX_BUILD_SANDBOX_SHELL", "/bin/sh".to_string()),
        ("CRUNCH_PROOF_STATIC_BUSYBOX_CANDIDATE", "target/definitely-missing-busybox".to_string()),
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should fail when no static busybox is discoverable");
    assert!(stderr.contains("static busybox shell not found"), "failure should be explicit, stderr:\n{stderr}");
    assert!(!nix_build_sentinel.exists(), "missing-shell path must not invoke nix-build");
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_anchors_relative_sandbox_shell_to_repo_root() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let busybox_root = fixture.repo_dir.join("target/proof-busybox-static/bin");
    std::fs::create_dir_all(&busybox_root).unwrap();
    write_executable_script(&busybox_root.join("busybox"), "#!/bin/sh\nset -eu\nexit 0\n");
    let bundle_arg = Path::new("target/relative-shell-proof");
    let bundle_dir = fixture.repo_dir.join(bundle_arg);

    let output = fixture.run_args_with_envs(&["--bundle-dir", "target/relative-shell-proof"], &[(
        "SNIX_BUILD_SANDBOX_SHELL",
        "target/proof-busybox-static/bin/busybox".to_string(),
    )]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "script should succeed, stderr:\n{stderr}");
    assert_eq!(
        std::fs::read_to_string(bundle_dir.join("sandbox-shell.txt")).unwrap().trim(),
        fixture.repo_dir.join("target/proof-busybox-static/bin/busybox").display().to_string()
    );
}

#[cfg(unix)]
#[test]
fn prove_self_hosting_script_rejects_option_like_bundle_dir_value() {
    let fixture = ProofScriptFixture::new();
    std::fs::create_dir_all(fixture.repo_dir.join("tmp")).unwrap();
    let output = fixture.run_args(&["--bundle-dir", "--check"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "script should reject malformed bundle-dir usage");
    assert!(stderr.contains("--bundle-dir requires a directory, got option-like value: --check"));
}

/// Regression: audit bundle hashing enforces MAX_AUDIT_DEPTH (64).
/// A directory tree deeper than the limit must trigger a panic, not
/// silently skip or corrupt the hash.
#[test]
fn audit_hashing_rejects_excessively_deep_trees() {
    let tmp = tempfile::tempdir().unwrap();
    // Build a path 70 levels deep (exceeds MAX_AUDIT_DEPTH of 64).
    let depth: u32 = 70;
    let mut deep_path = tmp.path().to_path_buf();
    for i in 0..depth {
        deep_path = deep_path.join(format!("d{i}"));
    }
    std::fs::create_dir_all(&deep_path).unwrap();
    std::fs::write(deep_path.join("leaf.txt"), b"deep").unwrap();

    let dummy_output = std::process::Command::new("/bin/sh").arg("-c").arg("true").output().unwrap();

    let result = std::panic::catch_unwind(|| {
        write_command_audit(
            "depth-limit",
            "deep",
            tmp.path(),
            &["test".to_string()],
            &dummy_output,
            &[AuditArtifact {
                label: "deep-tree",
                path: tmp.path(),
            }],
            &[],
        )
    });

    assert!(result.is_err(), "audit hashing must panic on trees deeper than MAX_AUDIT_DEPTH");
}

#[test]
fn remove_crunch_outputs_ignores_non_crunch_entries() {
    let tempdir = tempfile::tempdir().unwrap();
    let store = tempdir.path();
    let kept = store.join(format!("{FAKE_STORE_HASH}-kept"));
    std::fs::create_dir_all(&kept).unwrap();

    assert_eq!(remove_crunch_outputs(store), 0);
    assert!(kept.exists());
}

#[cfg(unix)]
#[test]
fn remove_crunch_outputs_removes_read_only_crunch_output() {
    use std::os::unix::fs::PermissionsExt;

    let tempdir = tempfile::tempdir().unwrap();
    let store = tempdir.path();
    let output = store.join(format!("{FAKE_STORE_HASH}-crunch"));
    let bin = output.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("crunch"), b"binary").unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(PROOF_READ_ONLY_DIR_MODE)).unwrap();
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(PROOF_READ_ONLY_DIR_MODE)).unwrap();

    assert_eq!(remove_crunch_outputs(store), 1);
    assert!(!output.exists());
}

#[test]
fn stage_context_reads_saved_snapshot_from_disk() {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'self-build-proof: output-binary=/tmp/out\n' >&2")
        .output()
        .unwrap();
    assert!(output.status.success());

    let evidence = record_stage_evidence(
        proof_dir.path(),
        "stage-z",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );
    std::fs::write(&evidence.diagnostics_file, "saved snapshot marker\n").unwrap();

    let context = stage_context(&evidence);
    assert!(context.contains("saved snapshot marker"));
    assert!(
        !context.contains("stage: stage-z"),
        "context should come from saved file, not recompute diagnostics"
    );
}

fn panic_with_controlled_breadcrumbs() -> ! {
    let proof_dir = tempfile::tempdir().unwrap();
    let store = proof_dir.path().join("store");
    let state = proof_dir.path().join("state");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("printf 'self-build-proof: output-binary=/tmp/fake-out\ncontrolled stderr\n' >&2")
        .output()
        .unwrap();
    assert!(output.status.success());

    let evidence = record_stage_evidence(
        proof_dir.path(),
        "controlled-failure",
        vec!["crunch".to_string(), "self-build".to_string()],
        output,
        &store,
        &state,
    );

    panic!("controlled failure for breadcrumb verification\n{}", stage_context(&evidence));
}

#[test]
fn expected_store_prefix_defaults_to_crunch_store() {
    let command = vec!["crunch".to_string(), "self-build".to_string()];
    assert_eq!(expected_store_prefix(&command), "/crunch/store");
}

#[test]
fn expected_store_prefix_matches_nix_compat_precedence() {
    let command = vec![
        "crunch".to_string(),
        "--store-prefix".to_string(),
        "/tmp/custom-store".to_string(),
        "--nix-compat".to_string(),
        "self-build".to_string(),
    ];
    assert_eq!(expected_store_prefix(&command), "/nix/store");
}

#[test]
fn self_hosting_controlled_failure_reports_breadcrumbs_trigger() {
    if std::env::var_os(CONTROLLED_FAILURE_ENV).is_none() {
        return;
    }

    panic_with_controlled_breadcrumbs();
}

#[test]
fn self_hosting_controlled_failure_reports_breadcrumbs() {
    let current_exe = std::env::current_exe().unwrap();
    let output = std::process::Command::new(&current_exe)
        .env(CONTROLLED_FAILURE_ENV, "1")
        .arg("self_hosting_controlled_failure_reports_breadcrumbs_trigger")
        .arg("--exact")
        .arg("--nocapture")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    assert!(!output.status.success(), "controlled child run should fail to expose breadcrumbs");
    assert!(combined.contains("controlled failure for breadcrumb verification"));
    assert!(combined.contains("audit bundle:"));
    assert!(combined.contains("diagnostics:"));
    assert!(combined.contains("saved diagnostics snapshot:"));
    assert!(combined.contains("self-build-proof: output-binary=/tmp/fake-out"));
}

#[test]
#[ignore]
fn self_hosting_stage0_stage1_stage2() {
    if let Some(err) = self_build_prereq_error() {
        panic!("self-hosting proof prerequisites not met: {err}");
    }

    let proof_mode = ProofMode::current();
    let no_host_tools_inventory = proof_stage0_inventory();
    let later_stage_hermeticity = proof_later_stage_hermeticity_mode();
    let proof_dir = tempfile::tempdir().unwrap_or_else(|err| panic!("tempdir for proof: {err}"));
    eprintln!("proof dir: {}", proof_dir.path().display());
    eprintln!("proof mode: {:?}", proof_mode);
    eprintln!("later-stage hermeticity: {}", later_stage_hermeticity.as_str());
    let store = proof_dir.path().join("store");
    std::fs::create_dir_all(&store).unwrap();

    // The store starts empty. Verify no stale outputs exist.
    assert!(find_bwrap_on_disk(&store).is_none(), "fresh store must not contain bwrap",);
    assert!(find_busybox_on_disk(&store).is_none(), "fresh store must not contain busybox",);
    assert!(find_crunch_binary(&store).is_none(), "fresh store must not contain crunch",);

    // ── Stage 0: checkout binary builds stage1 ──────────────────

    eprintln!("\n=== PROOF: Stage 0 (checkout -> stage1) ===\n");

    let stage0_state = proof_dir.path().join("state0");
    std::fs::create_dir_all(&stage0_state).unwrap();

    let mut stage0_command = vec![
        "crunch".to_string(),
        "--verbose".to_string(),
        "--log-level".to_string(),
        "info".to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--state-dir".to_string(),
        stage0_state.display().to_string(),
        "--nix-compat".to_string(),
        "self-build".to_string(),
        "--no-substitute".to_string(),
        "-j".to_string(),
        "4".to_string(),
    ];
    append_no_host_tools_stage0_args(&mut stage0_command, no_host_tools_inventory.as_deref());
    if proof_mode.stage0_path_is_scrubbed() || no_host_tools_inventory.is_some() {
        let helper_path = std::env::var_os("PATH").expect("proof PATH must be set");
        if proof_mode.stage0_path_is_scrubbed() {
            for blocked in BLOCKED_NIX_BINARIES {
                assert!(
                    find_executable_in_path_var(blocked, helper_path.as_os_str()).is_none(),
                    "non-nix-host proof PATH must block {blocked}",
                );
            }
        }
        if no_host_tools_inventory.is_some() {
            for blocked in BLOCKED_HOST_TOOL_BINARIES {
                assert!(
                    find_executable_in_path_var(blocked, helper_path.as_os_str()).is_none(),
                    "no-host-tools proof PATH must block {blocked}",
                );
            }
        }
    }
    eprintln!("stage0 store: {}", store.display());
    eprintln!("stage0 state: {}", stage0_state.display());
    let crunch_bin = cargo_bin("crunch");
    let mut stage0_process = std::process::Command::new(&crunch_bin);
    stage0_process.args(&stage0_command[1..]);
    let stage0 = run_command_live(proof_dir.path(), "stage0", &mut stage0_process).unwrap_or_else(|err| {
        panic!(
            "stage0 should execute: {err}\n{}",
            pre_stage_context("stage0", &stage0_command, &store, &stage0_state),
        )
    });
    assert_eq!(stage0.stdout_file, stage_stream_file(proof_dir.path(), "stage0", "stdout"));
    assert_eq!(stage0.stderr_file, stage_stream_file(proof_dir.path(), "stage0", "stderr"));
    let stage0_evidence =
        record_stage_evidence(proof_dir.path(), "stage0", stage0_command, stage0.output, &store, &stage0_state);

    eprintln!("stage0 audit: {}", stage0_evidence.audit_dir.display());
    eprintln!("stage0 diagnostics: {}", stage0_evidence.diagnostics_file.display());
    eprintln!("stage0 stdout: {}", stage0_evidence.stdout_file.display());
    eprintln!("stage0 stderr: {}", stage0_evidence.stderr_file.display());
    assert_stage_success(&stage0_evidence);

    let s0_mode = extract_proof_field(&stage0_evidence.stderr, "hermeticity-mode");
    assert_eq!(
        s0_mode,
        Some("practical"),
        "stage0 should report practical hermeticity.\n{}",
        stage_context(&stage0_evidence),
    );
    let s0_fallbacks = extract_proof_fields(&stage0_evidence.stderr, "fallback-event");
    assert!(
        s0_fallbacks.iter().any(|value| value.starts_with("bwrap-host-fallback:")),
        "stage0 should report initial host bwrap fallback.\n{}",
        stage_context(&stage0_evidence),
    );
    assert!(
        s0_fallbacks.iter().any(|value| value.starts_with("source-host-discovery:")),
        "stage0 should report checkout source discovery fallback.\n{}",
        stage_context(&stage0_evidence),
    );

    // Find the stage1 binary.
    let stage1_binary = find_crunch_binary(&store);
    assert!(
        stage1_binary.is_some(),
        "stage0 should produce *-crunch/bin/crunch.\n{}",
        stage_context(&stage0_evidence),
    );
    let stage1_binary = stage1_binary.unwrap();
    eprintln!("stage1 binary: {}", stage1_binary.display());

    // Verify stage1 runs.
    let stage1_help = std::process::Command::new(&stage1_binary)
        .arg("--help")
        .output()
        .unwrap_or_else(|err| panic!("stage1 binary should run: {err}\n{}", stage_context(&stage0_evidence)));
    let stage1_help_stderr = String::from_utf8_lossy(&stage1_help.stderr);
    assert!(
        stage1_help.status.success(),
        "stage1 --help failed.\n{}\nhelp stderr:\n{}",
        stage_context(&stage0_evidence),
        stage1_help_stderr,
    );

    // ── Verify bootstrap tools on disk (spec requirement) ─────

    let bwrap_bin = find_bwrap_on_disk(&store);
    assert!(
        bwrap_bin.is_some(),
        "*-bwrap/bin/bwrap must exist on disk after stage0.\n{}",
        stage_context(&stage0_evidence),
    );
    let bwrap_bin = bwrap_bin.unwrap();
    #[cfg(unix)]
    assert!(
        is_executable(&bwrap_bin),
        "bwrap binary must be executable: {}\n{}",
        bwrap_bin.display(),
        stage_context(&stage0_evidence),
    );
    eprintln!("bwrap on disk: {}", bwrap_bin.display());

    let busybox_bin = find_busybox_on_disk(&store);
    assert!(
        busybox_bin.is_some(),
        "*-busybox/bin/busybox must exist on disk after stage0.\n{}",
        stage_context(&stage0_evidence),
    );
    let busybox_bin = busybox_bin.unwrap();
    #[cfg(unix)]
    assert!(
        is_executable(&busybox_bin),
        "busybox binary must be executable: {}\n{}",
        busybox_bin.display(),
        stage_context(&stage0_evidence),
    );
    eprintln!("busybox on disk: {}", busybox_bin.display());

    // ── Prepare for Stage 2 ─────────────────────────────────────

    // The store has the root output (*-crunch) on disk.
    // Intermediate bootstrap deps (bwrap, busybox, gcc, etc.) are in
    // castore/PathInfo but NOT exported to the --store directory
    // (only root outputs get exported to disk).
    //
    // Copy the stage1 binary out before invalidation, since it lives
    // inside the *-crunch directory we're about to remove.

    let stage1_copy = proof_dir.path().join("stage1-crunch");
    std::fs::copy(&stage1_binary, &stage1_copy)
        .unwrap_or_else(|err| panic!("copy stage1 binary out of store: {err}\n{}", stage_context(&stage0_evidence)));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stage1_copy, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let stage1_binary = stage1_copy;

    let staged_source = extract_proof_field(&stage0_evidence.stderr, "staged-source");
    assert!(
        staged_source.is_some(),
        "stage0 should emit staged-source proof line.\n{}",
        stage_context(&stage0_evidence),
    );
    let staged_source = staged_source.unwrap();
    let staged_source_path = PathBuf::from(&staged_source);
    assert!(
        staged_source_path.exists(),
        "stage0 staged source must exist on disk: {}\n{}",
        staged_source_path.display(),
        stage_context(&stage0_evidence),
    );

    // Remove *-crunch so stage2 must rebuild the final binary.
    // Stage2 reuses the castore/PathInfo cache for intermediate deps.
    let removed = remove_crunch_outputs(&store);
    assert!(removed >= 1, "should have removed at least 1 *-crunch dir");
    assert!(find_crunch_binary(&store).is_none(), "crunch output should be gone after invalidation",);

    let stale_bwrap_bin = store.join("00000000000000000000000000000000-stale-bwrap").join("bin").join("bwrap");
    write_executable_script(&stale_bwrap_bin, "#!/bin/sh\nset -eu\nexit 97\n");
    let stale_busybox_bin = store.join("00000000000000000000000000000000-stale-busybox").join("bin").join("busybox");
    write_executable_script(&stale_busybox_bin, "#!/bin/sh\nset -eu\nexit 98\n");
    assert_ne!(stale_bwrap_bin, bwrap_bin, "stale bwrap sibling must differ from stage0 bwrap");
    assert_ne!(stale_busybox_bin, busybox_bin, "stale busybox sibling must differ from stage0 busybox");

    // Fresh state dir so pathinfo.redb doesn't give a false cache hit
    // on the final crunch output.
    let stage2_state = proof_dir.path().join("state2");
    std::fs::create_dir_all(&stage2_state).unwrap();

    // ── Stage 2: stage1 binary rebuilds crunch ──────────────────

    eprintln!("\n=== PROOF: Stage 2 (stage1 -> stage2) ===\n");

    let mut stage2_command = vec![
        stage1_binary.display().to_string(),
        "--verbose".to_string(),
        "--log-level".to_string(),
        "info".to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--state-dir".to_string(),
        stage2_state.display().to_string(),
        "--nix-compat".to_string(),
        "self-build".to_string(),
        "--source-store-path".to_string(),
        staged_source.to_string(),
        "--bootstrap-bwrap-path".to_string(),
        bwrap_bin.display().to_string(),
        "--bootstrap-busybox-path".to_string(),
        busybox_bin.display().to_string(),
        "--no-substitute".to_string(),
        "-j".to_string(),
        "4".to_string(),
    ];
    if later_stage_hermeticity.is_strict() {
        stage2_command.push("--strict-hermetic".to_string());
    }
    eprintln!("stage2 store: {}", store.display());
    eprintln!("stage2 state: {}", stage2_state.display());
    let stage2_store_prefix = expected_store_prefix(&stage2_command);
    let mut stage2_process = std::process::Command::new(&stage1_binary);
    stage2_process.current_dir(proof_dir.path());
    stage2_process.env("PATH", "");
    stage2_process.args(&stage2_command[1..]);
    let stage2 = run_command_live(proof_dir.path(), "stage2", &mut stage2_process).unwrap_or_else(|err| {
        panic!(
            "stage2 should execute: {err}\n{}",
            pre_stage_context("stage2", &stage2_command, &store, &stage2_state),
        )
    });
    assert_eq!(stage2.stdout_file, stage_stream_file(proof_dir.path(), "stage2", "stdout"));
    assert_eq!(stage2.stderr_file, stage_stream_file(proof_dir.path(), "stage2", "stderr"));
    let stage2_evidence =
        record_stage_evidence(proof_dir.path(), "stage2", stage2_command, stage2.output, &store, &stage2_state);

    eprintln!("stage2 audit: {}", stage2_evidence.audit_dir.display());
    eprintln!("stage2 diagnostics: {}", stage2_evidence.diagnostics_file.display());
    eprintln!("stage2 stdout: {}", stage2_evidence.stdout_file.display());
    eprintln!("stage2 stderr: {}", stage2_evidence.stderr_file.display());
    assert_stage_success(&stage2_evidence);

    // ── Verify stage2 output ────────────────────────────────────

    let stage2_binary = find_crunch_binary(&store);
    assert!(
        stage2_binary.is_some(),
        "stage2 should produce *-crunch/bin/crunch.\n{}",
        stage_context(&stage2_evidence),
    );
    let stage2_binary = stage2_binary.unwrap();
    eprintln!("stage2 binary: {}", stage2_binary.display());

    // Stage2 binary should run.
    let stage2_help = std::process::Command::new(&stage2_binary)
        .arg("--help")
        .output()
        .unwrap_or_else(|err| panic!("stage2 binary should run: {err}\n{}", stage_context(&stage2_evidence)));
    let stage2_help_stderr = String::from_utf8_lossy(&stage2_help.stderr);
    assert!(
        stage2_help.status.success(),
        "stage2 --help failed.\n{}\nhelp stderr:\n{}",
        stage_context(&stage2_evidence),
        stage2_help_stderr,
    );
    let stage2_stdout = String::from_utf8_lossy(&stage2_help.stdout);
    assert!(
        stage2_stdout.contains("crunch"),
        "stage2 --help should mention crunch.\n{}",
        stage_context(&stage2_evidence),
    );

    // ── Verify proof markers ────────────────────────────────────

    // Stage2 was driven by stage1 binary, not the checkout binary.
    let s2_invoking = extract_proof_field(&stage2_evidence.stderr, "invoking-binary");
    assert!(
        s2_invoking.is_some(),
        "stage2 should emit invoking-binary proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    let s2_invoking_path = PathBuf::from(s2_invoking.unwrap());
    // The invoking binary MUST be the stage1 binary we found earlier.
    // current_exe() may resolve symlinks or return a different
    // representation, so canonicalize both before comparing.
    let stage1_canonical = std::fs::canonicalize(&stage1_binary).unwrap_or_else(|_| stage1_binary.clone());
    let invoking_canonical = std::fs::canonicalize(&s2_invoking_path).unwrap_or_else(|_| s2_invoking_path.clone());
    assert_eq!(
        invoking_canonical,
        stage1_canonical,
        "stage2 invoking binary must be the stage1 binary.\n\
         invoking: {}\n\
         stage1:   {}\n{}",
        invoking_canonical.display(),
        stage1_canonical.display(),
        stage_context(&stage2_evidence),
    );

    // Stage2 MUST use crunch-built bwrap. The self-build pipeline
    // now exports bwrap and busybox as separate root builds (step 2/4)
    // so they land on disk in the --store directory.
    let s2_source = extract_proof_field(&stage2_evidence.stderr, "staged-source");
    assert!(
        s2_source.is_some(),
        "stage2 should emit staged-source proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    assert_eq!(
        PathBuf::from(s2_source.unwrap()),
        staged_source_path,
        "stage2 must reuse the exact staged source from stage0.\n{}",
        stage_context(&stage2_evidence),
    );

    let s2_mode = extract_proof_field(&stage2_evidence.stderr, "hermeticity-mode");
    assert!(
        s2_mode.is_some(),
        "stage2 should emit hermeticity-mode proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    assert_eq!(
        s2_mode.unwrap(),
        later_stage_hermeticity.as_str(),
        "stage2 must report the selected later-stage hermeticity mode.\n{}",
        stage_context(&stage2_evidence),
    );

    let s2_bwrap = extract_proof_field(&stage2_evidence.stderr, "bwrap-source");
    assert!(
        s2_bwrap.is_some(),
        "stage2 should emit bwrap-source proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    let bwrap_val = s2_bwrap.unwrap();
    assert!(
        bwrap_val.starts_with("crunch-built:"),
        "stage2 bwrap must be crunch-built, got: {bwrap_val}.\n{}",
        stage_context(&stage2_evidence),
    );
    let bwrap_report_path = extract_bwrap_binary_path(&stage2_evidence.stderr).expect("checked proof line above");
    let normalized_bwrap_report_path = normalize_bwrap_binary_path(&bwrap_report_path);
    assert_eq!(
        normalized_bwrap_report_path,
        bwrap_bin,
        "stage2 must reuse the exact stage0 bwrap root even when stale siblings exist.\n{}",
        stage_context(&stage2_evidence),
    );
    let bwrap_store_name =
        extract_store_entry_name(&bwrap_report_path).expect("bwrap report path must include store entry");
    let expected_bwrap_log = format!("Using crunch-built bwrap: {stage2_store_prefix}/{bwrap_store_name}/bin");
    assert!(
        stage2_evidence.stderr.contains(&expected_bwrap_log),
        "stage2 build log must use the exact reported crunch-built bwrap.\nexpected: {expected_bwrap_log}\n{}",
        stage_context(&stage2_evidence),
    );

    // Stage2 MUST find crunch-built busybox on disk and use the exact same one inside the crunch build.
    let s2_fallbacks = extract_proof_fields(&stage2_evidence.stderr, "fallback-event");
    assert_eq!(
        s2_fallbacks,
        vec!["none"],
        "stage2 strict proof path must report zero fallback events.\n{}",
        stage_context(&stage2_evidence),
    );

    let s2_busybox = extract_proof_field(&stage2_evidence.stderr, "busybox-path");
    assert!(
        s2_busybox.is_some(),
        "stage2 should emit busybox-path proof line.\n{}",
        stage_context(&stage2_evidence),
    );
    let busybox_val = s2_busybox.unwrap();
    assert_ne!(
        busybox_val,
        "none",
        "stage2 must have a crunch-built busybox, not none.\n{}",
        stage_context(&stage2_evidence),
    );
    let busybox_report_path = PathBuf::from(busybox_val);
    assert_eq!(
        busybox_report_path,
        busybox_bin,
        "stage2 must reuse the exact stage0 busybox root even when stale siblings exist.\n{}",
        stage_context(&stage2_evidence),
    );
    let busybox_store_name =
        extract_store_entry_name(&busybox_report_path).expect("busybox report path must include store entry");
    let expected_busybox_log =
        format!("Using crunch-built busybox: {stage2_store_prefix}/{busybox_store_name}/bin/busybox");
    assert!(
        stage2_evidence.stderr.contains(&expected_busybox_log),
        "stage2 build log must use the exact reported crunch-built busybox.\nexpected: {expected_busybox_log}\n{}",
        stage_context(&stage2_evidence),
    );

    // Output binary recorded.
    let s2_output = extract_proof_field(&stage2_evidence.stderr, "output-binary");
    assert!(
        s2_output.is_some(),
        "stage2 should emit output-binary proof line.\n{}",
        stage_context(&stage2_evidence),
    );

    let stage1_hash = hash_file_record(&stage1_binary, stage1_binary.display().to_string());
    let stage2_hash = hash_file_record(&stage2_binary, stage2_binary.display().to_string());
    assert_eq!(
        stage1_hash.digest_blake3,
        stage2_hash.digest_blake3,
        "stage1 and stage2 binaries must match byte-for-byte.\n{}\n{}",
        stage_context(&stage0_evidence),
        stage_context(&stage2_evidence),
    );
    let stage0_bwrap_hash = hash_file_record(&bwrap_bin, bwrap_bin.display().to_string());
    let stage2_bwrap_binary = normalize_bwrap_binary_path(&bwrap_report_path);
    let stage2_bwrap_hash = hash_file_record(&stage2_bwrap_binary, stage2_bwrap_binary.display().to_string());
    assert_eq!(
        stage0_bwrap_hash.digest_blake3,
        stage2_bwrap_hash.digest_blake3,
        "stage0 and stage2 bwrap bootstrap outputs must match.\n{}\n{}",
        stage_context(&stage0_evidence),
        stage_context(&stage2_evidence),
    );
    let stage0_busybox_hash = hash_file_record(&busybox_bin, busybox_bin.display().to_string());
    let stage2_busybox_hash = hash_file_record(&busybox_report_path, busybox_report_path.display().to_string());
    assert_eq!(
        stage0_busybox_hash.digest_blake3,
        stage2_busybox_hash.digest_blake3,
        "stage0 and stage2 busybox bootstrap outputs must match.\n{}\n{}",
        stage_context(&stage0_evidence),
        stage_context(&stage2_evidence),
    );

    let proof_bundle_dir = resolve_proof_bundle_dir();
    let proof_manifest = write_proof_bundle(
        &proof_bundle_dir,
        &stage0_evidence,
        &stage2_evidence,
        &store,
        &stage0_state,
        &stage2_state,
        &stage1_binary,
        &stage2_binary,
        proof_mode,
        None,
    );
    let proof_summary = proof_bundle_dir.join("summary.txt");
    assert!(proof_manifest.exists(), "proof manifest must exist: {}", proof_manifest.display());
    assert!(proof_summary.exists(), "proof summary must exist: {}", proof_summary.display());

    eprintln!("\n=== PROOF PASSED ===");
    eprintln!("stage1: {}", stage1_binary.display());
    eprintln!("stage2: {}", stage2_binary.display());
    eprintln!("bwrap:  {bwrap_val}");
    eprintln!("busybox: {}", busybox_val);
    eprintln!("proof bundle: {}", proof_bundle_dir.display());
    eprintln!("proof manifest: {}", proof_manifest.display());
    eprintln!("proof summary: {}", proof_summary.display());
}
