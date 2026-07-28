use std::collections::BTreeMap;
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde::Serialize;

use crate::stagex_mes_sources::MesSourceMaterializationReport;
use crate::stagex_mes_sources::materialize_authenticated_mes_sources;

const MES_SOURCE_DIR: &str = "mes-0.27.1";
const NYACC_SOURCE_DIR: &str = "nyacc";
const MES_KAEM_RUN: &str = "kaem.run";
const MES_GENERATED_KAEM: &str = "stagex-mes-m2.kaem";
const MES_STDERR: &str = "mes-m2.stderr.txt";
const MES_SMOKE_STDERR: &str = "mes-m2-smoke.stderr.txt";
const MES_REPORT: &str = "mes-m2-inventory.json";
const MES_REPORT_FORMAT: &str = "mantle-stagex-mes-m2-inventory-v1";
const MES_NON_CLAIM: &str = "this inventory binds source-built mes-m2 only; it does not prove NYACC regeneration, Mes libc, TinyCC, or provider admission";
pub(crate) const MES_M2_BLAKE3: &str = "de4f20cdc2ad232ca79c5aee20c1ef491a71ce95fcfdb28020fc0c9a4c51ea3d";
const MES_CONFIG_BYTES: &[u8] = b"#undef SYSTEM_LIBC\n#define MES_VERSION \"0.27.1\"\n";
const MES_KAEM_COMMAND_MARKER: &str = "mkdir -p m2";
const MES_LOAD_ADDRESS_OLD: &str = "0x1000000";
const MES_LOAD_ADDRESS_NEW: &str = "0x8048000";
const MES_WAIT4_MARKER: &str = "  long long_rusage = cast_voidp_to_long (rusage);";
const MES_WAIT4_INSERT: &str = "  *long_status_ptr = 0;";
const MES_SCRIPT_BYTES_MAX: u64 = 256 * 1024;
const MES_EXECUTABLE_BYTES_MAX: u64 = 8 * 1024 * 1024;
const MES_STDERR_BYTES_MAX: u64 = 64 * 1024;
const MES_TIMEOUT_MS: u64 = 120_000;
const MES_POLL_INTERVAL_MS: u64 = 10;
const GENERATED_LAUNCH_MAX_ATTEMPTS: u32 = 16;
const GENERATED_LAUNCH_RETRY_DELAY_MS: u64 = 20;
const ETXTBSY_EXIT_CODE: i32 = 126;
const ETXTBSY_STDERR_MARKER: &str = "Text file busy";
const MES_ARENA_BYTES: &str = "20000000";
const MES_STACK_BYTES: &str = "6000000";
const MES_OUTPUT_COUNT: usize = 2;

#[derive(Debug, Clone)]
pub(crate) struct MesM2InventoryRequest<'a> {
    pub source_bundle_path: &'a Path,
    pub expected_source_bundle_blake3: &'a str,
    pub full_stage0_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MesM2Output {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MesM2InventoryReport {
    pub format: &'static str,
    pub sources: MesSourceMaterializationReport,
    pub generated_script_path: PathBuf,
    pub generated_script_digest_blake3: String,
    pub command_count: u32,
    pub script_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<MesM2Output>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum MesM2InventoryError {
    Source(crate::stagex_mes_sources::MesSourceError),
    InvalidInput(String),
    Io { action: String, source: io::Error },
    ProcessFailure { exit_code: Option<i32>, stderr: String },
    ProcessTimeout { timeout_ms: u64 },
}

impl std::fmt::Display for MesM2InventoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "materializing Mes sources: {error}"),
            Self::InvalidInput(message) => write!(formatter, "invalid mes-m2 inventory input: {message}"),
            Self::Io { action, source } => write!(formatter, "{action}: {source}"),
            Self::ProcessFailure { exit_code, stderr } => {
                write!(formatter, "source-built mes-m2 stage failed with status {exit_code:?}: {stderr}")
            }
            Self::ProcessTimeout { timeout_ms } => {
                write!(formatter, "source-built mes-m2 stage exceeded {timeout_ms} ms")
            }
        }
    }
}

impl std::error::Error for MesM2InventoryError {}

impl From<crate::stagex_mes_sources::MesSourceError> for MesM2InventoryError {
    fn from(error: crate::stagex_mes_sources::MesSourceError) -> Self {
        Self::Source(error)
    }
}

pub(crate) fn derive_mes_m2_inventory(
    request: MesM2InventoryRequest<'_>,
) -> Result<MesM2InventoryReport, MesM2InventoryError> {
    let tool_paths = validate_stage0_tool_paths(request.full_stage0_root)?;
    let sources = materialize_authenticated_mes_sources(
        request.source_bundle_path,
        request.expected_source_bundle_blake3,
        request.scratch_dir,
    )?;
    let mes_root = request.scratch_dir.join(MES_SOURCE_DIR);
    prepare_mes_sources(&mes_root)?;
    let (script_path, script_digest, command_count) = generate_mes_m2_script(&mes_root, &tool_paths)?;
    run_mes_m2_stage(&mes_root, &tool_paths, &script_path)?;
    run_mes_m2_smoke(&mes_root)?;
    let outputs = collect_mes_outputs(&mes_root)?;
    let report = MesM2InventoryReport {
        format: MES_REPORT_FORMAT,
        sources,
        generated_script_path: script_path,
        generated_script_digest_blake3: script_digest,
        command_count: command_count
            .checked_add(1)
            .ok_or_else(|| MesM2InventoryError::InvalidInput("Mes total command count overflow".to_string()))?,
        script_command_count: command_count,
        smoke_command_count: 1,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: MES_NON_CLAIM,
    };
    write_report(request.scratch_dir, &report)?;
    assert_eq!(report.outputs.len(), MES_OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

#[derive(Debug)]
struct Stage0MesTools {
    kaem: PathBuf,
    tools: BTreeMap<&'static str, PathBuf>,
}

fn validate_stage0_tool_paths(root: &Path) -> Result<Stage0MesTools, MesM2InventoryError> {
    let required = ["kaem", "mkdir", "M2-Planet", "blood-elf", "M1", "hex2", "cp"];
    let expected_by_name = crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
        .iter()
        .filter_map(|expected| {
            Path::new(expected.relative_path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| (name, expected))
        })
        .collect::<BTreeMap<_, _>>();
    let mut tools = BTreeMap::new();
    for name in required {
        let expected = expected_by_name
            .get(name)
            .ok_or_else(|| MesM2InventoryError::InvalidInput(format!("full Stage0 inventory lacks {name}")))?;
        let path = root.join(expected.relative_path);
        validate_file_digest(&path, expected.digest_blake3, name)?;
        tools.insert(name, path);
    }
    let kaem = tools
        .get("kaem")
        .cloned()
        .ok_or_else(|| MesM2InventoryError::InvalidInput("full Stage0 kaem is missing".to_string()))?;
    assert_eq!(tools.len(), required.len());
    assert!(tools.values().all(|path| path.is_absolute()));
    Ok(Stage0MesTools { kaem, tools })
}

fn prepare_mes_sources(mes_root: &Path) -> Result<(), MesM2InventoryError> {
    make_tree_owner_writable(mes_root)?;
    let config_path = mes_root.join("include/mes/config.h");
    fs::write(&config_path, MES_CONFIG_BYTES).map_err(|source| io_error("writing Mes config.h", source))?;
    replace_file_text(&mes_root.join(MES_KAEM_RUN), MES_LOAD_ADDRESS_OLD, MES_LOAD_ADDRESS_NEW, true)?;
    insert_line_after_marker(&mes_root.join("lib/linux/wait4.c"), MES_WAIT4_MARKER, MES_WAIT4_INSERT)?;
    let arch_dir = mes_root.join("include/arch");
    fs::create_dir(&arch_dir).map_err(|source| io_error("creating Mes arch include directory", source))?;
    for file in ["kernel-stat.h", "signal.h", "syscall.h"] {
        fs::copy(mes_root.join("include/linux/x86_64").join(file), arch_dir.join(file))
            .map_err(|source| io_error(format!("copying Mes arch header {file}"), source))?;
    }
    remove_if_present(&mes_root.join("mes/module/mes/psyntax.pp"))?;
    remove_if_present(&mes_root.join("mes/module/mes/psyntax.pp.header"))?;
    copy_if_regular(&mes_root.join("mes/module/srfi/srfi-9-struct.mes"), &mes_root.join("mes/module/srfi/srfi-9.mes"))?;
    copy_if_regular(
        &mes_root.join("mes/module/srfi/srfi-9/gnu-struct.mes"),
        &mes_root.join("mes/module/srfi/srfi-9/gnu.mes"),
    )?;
    assert!(config_path.is_file());
    assert!(arch_dir.join("syscall.h").is_file());
    Ok(())
}

fn generate_mes_m2_script(
    mes_root: &Path,
    stage0: &Stage0MesTools,
) -> Result<(PathBuf, String, u32), MesM2InventoryError> {
    let source = read_bounded_file(&mes_root.join(MES_KAEM_RUN), MES_SCRIPT_BYTES_MAX, "Mes kaem.run")?;
    let source_text = std::str::from_utf8(&source)
        .map_err(|error| MesM2InventoryError::InvalidInput(format!("Mes kaem.run is not UTF-8: {error}")))?;
    let command_body = source_text.find(MES_KAEM_COMMAND_MARKER).ok_or_else(|| {
        MesM2InventoryError::InvalidInput(format!("Mes kaem.run lacks command marker '{MES_KAEM_COMMAND_MARKER}'"))
    })?;
    let mut script = String::from("cc_cpu=x86_64\nmes_cpu=x86_64\nstage0_cpu=amd64\nblood_elf_flag=--64\n");
    script.push_str(&format!("srcdest={}/\n", absolute_utf8_path(mes_root, "Mes source root")?));
    script.push_str(&source_text[command_body..]);
    let (script, command_count) = rewrite_mes_commands(&script, mes_root, stage0)?;
    let script = script.replace(MES_LOAD_ADDRESS_OLD, MES_LOAD_ADDRESS_NEW);
    let bytes = script.into_bytes();
    let output = mes_root.join(MES_GENERATED_KAEM);
    write_create_new(&output, &bytes)?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert!(command_count > 0);
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    Ok((output, digest, command_count))
}

fn rewrite_mes_commands(
    source: &str,
    mes_root: &Path,
    stage0: &Stage0MesTools,
) -> Result<(String, u32), MesM2InventoryError> {
    let mut commands = stage0.tools.clone();
    commands.remove("kaem");
    let mes_m2_path = mes_root.join("bin/mes-m2");
    let mut output = String::with_capacity(source.len());
    let mut command_count = 0u32;
    for line in source.lines() {
        if is_embedded_mes_smoke_line(line) {
            continue;
        }
        let rewritten = rewrite_command_line(line, &commands, &mes_m2_path)?;
        if rewritten != line {
            command_count = command_count
                .checked_add(1)
                .ok_or_else(|| MesM2InventoryError::InvalidInput("Mes command count overflow".to_string()))?;
        }
        output.push_str(&rewritten);
        output.push('\n');
    }
    if command_count == 0 {
        return Err(MesM2InventoryError::InvalidInput("Mes script has no rewritten commands".to_string()));
    }
    assert!(!output.lines().any(|line| line.starts_with("./bin/mes-m2")));
    assert!(!output.contains("Running mes-m2"));
    assert!(!output.contains("GUILE_LOAD_PATH="));
    Ok((output, command_count))
}

fn is_embedded_mes_smoke_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "echo Running mes-m2"
        || trimmed.starts_with("GUILE_LOAD_PATH=")
        || trimmed.starts_with("./bin/mes-m2 -c ")
}

fn rewrite_command_line(
    line: &str,
    commands: &BTreeMap<&str, PathBuf>,
    mes_m2_path: &Path,
) -> Result<String, MesM2InventoryError> {
    if line.starts_with("./bin/mes-m2") {
        return Ok(line.replacen("./bin/mes-m2", absolute_utf8_path(mes_m2_path, "generated mes-m2")?, 1));
    }
    for (name, path) in commands {
        if line == *name || line.starts_with(&format!("{name} ")) {
            return Ok(line.replacen(name, absolute_utf8_path(path, name)?, 1));
        }
    }
    Ok(line.to_string())
}

fn run_mes_m2_stage(mes_root: &Path, stage0: &Stage0MesTools, script_path: &Path) -> Result<(), MesM2InventoryError> {
    let stderr_path = mes_root.parent().expect("Mes root has parent").join(MES_STDERR);
    let stderr_file = File::create(&stderr_path).map_err(|source| io_error("creating mes-m2 stderr", source))?;
    let mut child = Command::new(&stage0.kaem)
        .args(["--verbose", "--strict", "--file"])
        .arg(script_path)
        .current_dir(mes_root)
        .env_clear()
        .env("MES_ARENA", MES_ARENA_BYTES)
        .env("MES_MAX_ARENA", MES_ARENA_BYTES)
        .env("MES_STACK", MES_STACK_BYTES)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|source| io_error("spawning full Stage0 kaem for mes-m2", source))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|source| io_error("waiting for mes-m2 stage", source))? {
            return classify_status(status, &stderr_path);
        }
        if started.elapsed() >= Duration::from_millis(MES_TIMEOUT_MS) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(MesM2InventoryError::ProcessTimeout {
                timeout_ms: MES_TIMEOUT_MS,
            });
        }
        thread::sleep(Duration::from_millis(MES_POLL_INTERVAL_MS));
    }
}

fn run_mes_m2_smoke(mes_root: &Path) -> Result<(), MesM2InventoryError> {
    let executable = mes_root.join("bin/mes-m2");
    let stderr_path = mes_root.parent().expect("Mes root has parent").join(MES_SMOKE_STDERR);
    let load_path = format!(
        "{}/mes/module:{}/module",
        absolute_utf8_path(mes_root, "Mes source root")?,
        absolute_utf8_path(mes_root, "Mes source root")?
    );
    let mut attempt = 0u32;
    while attempt < GENERATED_LAUNCH_MAX_ATTEMPTS {
        attempt = attempt.saturating_add(1);
        let result = run_mes_m2_smoke_once(&executable, mes_root, &stderr_path, &load_path);
        match result {
            Err(MesM2InventoryError::Io { source, .. }) if source.raw_os_error() == Some(libc::ETXTBSY) => {
                thread::sleep(Duration::from_millis(GENERATED_LAUNCH_RETRY_DELAY_MS));
            }
            Err(MesM2InventoryError::ProcessFailure { exit_code, ref stderr })
                if exit_code == Some(ETXTBSY_EXIT_CODE) && stderr.contains(ETXTBSY_STDERR_MARKER) =>
            {
                thread::sleep(Duration::from_millis(GENERATED_LAUNCH_RETRY_DELAY_MS));
            }
            result => return result,
        }
    }
    Err(io_error(
        "spawning generated mes-m2 after bounded ETXTBSY retries",
        io::Error::from_raw_os_error(libc::ETXTBSY),
    ))
}

fn run_mes_m2_smoke_once(
    executable: &Path,
    mes_root: &Path,
    stderr_path: &Path,
    load_path: &str,
) -> Result<(), MesM2InventoryError> {
    let stderr_file = File::create(stderr_path).map_err(|source| io_error("creating mes-m2 smoke stderr", source))?;
    let mut child = Command::new(executable)
        .args(["-c", "(display 'Hello,M2-mes!) (newline)"])
        .current_dir(mes_root)
        .env_clear()
        .env("GUILE_LOAD_PATH", load_path)
        .env("MES_ARENA", MES_ARENA_BYTES)
        .env("MES_MAX_ARENA", MES_ARENA_BYTES)
        .env("MES_STACK", MES_STACK_BYTES)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|source| io_error("spawning generated mes-m2 smoke", source))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|source| io_error("waiting for mes-m2 smoke", source))? {
            return classify_status(status, stderr_path);
        }
        if started.elapsed() >= Duration::from_millis(MES_TIMEOUT_MS) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(MesM2InventoryError::ProcessTimeout {
                timeout_ms: MES_TIMEOUT_MS,
            });
        }
        thread::sleep(Duration::from_millis(MES_POLL_INTERVAL_MS));
    }
}

fn classify_status(status: ExitStatus, stderr_path: &Path) -> Result<(), MesM2InventoryError> {
    let stderr = read_bounded_observation(stderr_path, MES_STDERR_BYTES_MAX, "mes-m2 stderr")?;
    if !status.success() {
        return Err(MesM2InventoryError::ProcessFailure {
            exit_code: status.code(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        });
    }
    assert!(status.code().is_none() || status.code() == Some(0));
    assert!(u64::try_from(stderr.len()).unwrap_or(u64::MAX) <= MES_STDERR_BYTES_MAX);
    Ok(())
}

fn collect_mes_outputs(mes_root: &Path) -> Result<Vec<MesM2Output>, MesM2InventoryError> {
    let specs = [("mes-m2", "bin/mes-m2"), ("mes", "bin/mes")];
    let mut outputs = Vec::with_capacity(MES_OUTPUT_COUNT);
    for (artifact_id, relative_path) in specs {
        let path = mes_root.join(relative_path);
        let bytes = read_bounded_file(&path, MES_EXECUTABLE_BYTES_MAX, artifact_id)?;
        outputs.push(MesM2Output {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len: u64::try_from(bytes.len())
                .map_err(|_| MesM2InventoryError::InvalidInput("Mes output size does not fit u64".to_string()))?,
            digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    assert_eq!(outputs.len(), MES_OUTPUT_COUNT);
    assert_eq!(outputs[0].digest_blake3, outputs[1].digest_blake3);
    Ok(outputs)
}

fn make_tree_owner_writable(root: &Path) -> Result<(), MesM2InventoryError> {
    const WALK_ENTRY_MAX: u32 = 20_000;
    let mut pending = vec![root.to_path_buf()];
    let mut entry_count = 0u32;
    while let Some(path) = pending.pop() {
        entry_count = entry_count
            .checked_add(1)
            .ok_or_else(|| MesM2InventoryError::InvalidInput("Mes source walk count overflow".to_string()))?;
        if entry_count > WALK_ENTRY_MAX {
            return Err(MesM2InventoryError::InvalidInput(format!("Mes source walk exceeds {WALK_ENTRY_MAX} entries")));
        }
        make_path_owner_writable(&path)?;
        if path.is_dir() {
            for entry in fs::read_dir(&path).map_err(|source| io_error("reading Mes source tree", source))? {
                pending.push(entry.map_err(|source| io_error("reading Mes source entry", source))?.path());
            }
        }
    }
    assert!(entry_count > 0);
    assert!(entry_count <= WALK_ENTRY_MAX);
    Ok(())
}

fn make_path_owner_writable(path: &Path) -> Result<(), MesM2InventoryError> {
    use std::os::unix::fs::PermissionsExt;
    const OWNER_WRITE: u32 = 0o200;
    let metadata = fs::symlink_metadata(path).map_err(|source| io_error("reading Mes path metadata", source))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | OWNER_WRITE);
    fs::set_permissions(path, permissions).map_err(|source| io_error("making Mes path owner-writable", source))?;
    assert!(
        fs::metadata(path)
            .map_err(|source| io_error("checking Mes path mode", source))?
            .permissions()
            .mode()
            & OWNER_WRITE
            != 0
    );
    Ok(())
}

fn replace_file_text(path: &Path, old: &str, new: &str, require_old: bool) -> Result<(), MesM2InventoryError> {
    let bytes = read_bounded_file(path, MES_SCRIPT_BYTES_MAX, "Mes patch target")?;
    let text = String::from_utf8(bytes)
        .map_err(|error| MesM2InventoryError::InvalidInput(format!("Mes patch target is not UTF-8: {error}")))?;
    if require_old && !text.contains(old) {
        return Err(MesM2InventoryError::InvalidInput(format!(
            "Mes patch target {} lacks required text '{old}'",
            path.display()
        )));
    }
    let replaced = text.replace(old, new);
    fs::write(path, replaced.as_bytes()).map_err(|source| io_error("writing Mes text patch", source))?;
    assert!(!require_old || replaced.contains(new));
    assert!(!require_old || !replaced.contains(old));
    Ok(())
}

fn insert_line_after_marker(path: &Path, marker: &str, insertion: &str) -> Result<(), MesM2InventoryError> {
    let bytes = read_bounded_file(path, MES_SCRIPT_BYTES_MAX, "Mes line patch target")?;
    let text = String::from_utf8(bytes)
        .map_err(|error| MesM2InventoryError::InvalidInput(format!("Mes line patch target is not UTF-8: {error}")))?;
    if text.contains(insertion) {
        return Ok(());
    }
    let replacement = format!("{marker}\n{insertion}");
    if !text.contains(marker) {
        return Err(MesM2InventoryError::InvalidInput(format!(
            "Mes patch target {} lacks required marker",
            path.display()
        )));
    }
    let patched = text.replacen(marker, &replacement, 1);
    fs::write(path, patched.as_bytes()).map_err(|source| io_error("writing Mes line patch", source))?;
    assert!(patched.contains(marker));
    assert!(patched.contains(insertion));
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<(), MesM2InventoryError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error(format!("removing {}", path.display()), source)),
    }
}

fn copy_if_regular(source: &Path, target: &Path) -> Result<(), MesM2InventoryError> {
    if !source.is_file() {
        return Ok(());
    }
    let source_bytes = read_bounded_file(source, MES_SCRIPT_BYTES_MAX, "Mes compatibility source")?;
    if let Ok(metadata) = fs::symlink_metadata(target) {
        if metadata.file_type().is_symlink() || metadata.is_file() {
            fs::remove_file(target).map_err(|error| io_error(format!("removing {}", target.display()), error))?;
        } else {
            return Err(MesM2InventoryError::InvalidInput(format!(
                "Mes compatibility target is not replaceable: {}",
                target.display()
            )));
        }
    }
    write_create_new(target, &source_bytes)?;
    assert!(target.is_file());
    assert_eq!(fs::symlink_metadata(target).map(|metadata| metadata.file_type().is_symlink()).ok(), Some(false));
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), MesM2InventoryError> {
    let bytes = read_bounded_file(path, MES_EXECUTABLE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(MesM2InventoryError::InvalidInput(format!(
            "full Stage0 {label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn read_bounded_observation(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, MesM2InventoryError> {
    let metadata = fs::metadata(path).map_err(|source| io_error(format!("reading {label} metadata"), source))?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(MesM2InventoryError::InvalidInput(format!(
            "{label} must be a regular file of at most {max_bytes} bytes, observed {} bytes at {}",
            metadata.len(),
            path.display()
        )));
    }
    let bytes = fs::read(path).map_err(|source| io_error(format!("reading {label}"), source))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(MesM2InventoryError::InvalidInput(format!("{label} exceeds {max_bytes} bytes")));
    }
    assert_eq!(u64::try_from(bytes.len()).ok(), Some(metadata.len()));
    assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= max_bytes);
    Ok(bytes)
}

fn read_bounded_file(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, MesM2InventoryError> {
    let metadata = fs::metadata(path).map_err(|source| io_error(format!("reading {label} metadata"), source))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_bytes {
        return Err(MesM2InventoryError::InvalidInput(format!(
            "{label} must be a nonempty regular file of at most {max_bytes} bytes, observed {} bytes at {}",
            metadata.len(),
            path.display()
        )));
    }
    let capacity = usize::try_from(metadata.len())
        .map_err(|_| MesM2InventoryError::InvalidInput(format!("{label} size does not fit usize")))?;
    let mut bytes = Vec::with_capacity(capacity);
    File::open(path)
        .and_then(|file| file.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes))
        .map_err(|source| io_error(format!("reading {label}"), source))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(MesM2InventoryError::InvalidInput(format!("{label} exceeds {max_bytes} bytes")));
    }
    assert_eq!(u64::try_from(bytes.len()).ok(), Some(metadata.len()));
    assert!(!bytes.is_empty());
    Ok(bytes)
}

fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), MesM2InventoryError> {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| io_error(format!("creating {}", path.display()), source))?;
    file.write_all(bytes).map_err(|source| io_error(format!("writing {}", path.display()), source))?;
    file.sync_all().map_err(|source| io_error(format!("syncing {}", path.display()), source))?;
    assert!(path.is_file());
    assert_eq!(fs::metadata(path).map(|metadata| metadata.len()).ok(), u64::try_from(bytes.len()).ok());
    Ok(())
}

fn write_report(scratch_dir: &Path, report: &MesM2InventoryReport) -> Result<(), MesM2InventoryError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| MesM2InventoryError::InvalidInput(format!("serializing mes-m2 report: {error}")))?;
    let path = scratch_dir.join(MES_REPORT);
    write_create_new(&path, &bytes)?;
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn absolute_utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str, MesM2InventoryError> {
    if !path.is_absolute() {
        return Err(MesM2InventoryError::InvalidInput(format!("{label} path is not absolute: {}", path.display())));
    }
    let text = path
        .to_str()
        .ok_or_else(|| MesM2InventoryError::InvalidInput(format!("{label} path is not UTF-8: {}", path.display())))?;
    assert!(text.starts_with('/'));
    assert!(!text.is_empty());
    Ok(text)
}

fn io_error(action: impl Into<String>, source: io::Error) -> MesM2InventoryError {
    MesM2InventoryError::Io {
        action: action.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const MES_SCRATCH_ENV: &str = "MANTLE_STAGE_X_MES_INVENTORY_SCRATCH";
    const STAGE0_ROOT_ENV: &str = "MANTLE_STAGE_X_STAGE0_FULL_ROOT";

    fn test_tools(root: &Path) -> Stage0MesTools {
        let tools = BTreeMap::from([
            ("M2-Planet", root.join("M2-Planet")),
            ("blood-elf", root.join("blood-elf")),
            ("M1", root.join("M1")),
            ("hex2", root.join("hex2")),
            ("mkdir", root.join("mkdir")),
            ("cp", root.join("cp")),
            ("kaem", root.join("kaem")),
        ]);
        Stage0MesTools {
            kaem: root.join("kaem"),
            tools,
        }
    }

    #[test]
    fn rewrites_mes_commands_to_exact_absolute_paths() {
        let root = Path::new("/stage0");
        let mes_root = Path::new("/scratch/mes-0.27.1");
        let source = "mkdir -p m2\nM2-Planet \\\n --debug\nblood-elf --64\nM1 -f in\nhex2 -f in\necho Running mes-m2\nGUILE_LOAD_PATH=/modules\n./bin/mes-m2 -c smoke\nGUILE_LOAD_PATH=/fubar\ncp bin/mes-m2 bin/mes\n";

        let (rewritten, count) = rewrite_mes_commands(source, mes_root, &test_tools(root)).unwrap();

        assert_eq!(count, 6);
        assert!(rewritten.contains("/stage0/M2-Planet"));
        assert!(rewritten.contains("/stage0/cp bin/mes-m2 bin/mes"));
        assert!(!rewritten.contains("Running mes-m2"));
        assert!(!rewritten.contains("GUILE_LOAD_PATH="));
        assert!(!rewritten.contains("./bin/mes-m2"));
    }

    #[test]
    fn rejects_relative_stage0_root_and_missing_command_marker() {
        let relative_error = absolute_utf8_path(Path::new("relative"), "test tool").unwrap_err();
        assert!(relative_error.to_string().contains("path is not absolute"));

        let marker_error = "echo only".find(MES_KAEM_COMMAND_MARKER).ok_or_else(|| {
            MesM2InventoryError::InvalidInput(format!("Mes kaem.run lacks command marker '{MES_KAEM_COMMAND_MARKER}'"))
        });
        assert!(marker_error.unwrap_err().to_string().contains("lacks command marker"));
    }

    #[test]
    #[ignore = "requires explicit authenticated source bundle, full Stage0 tree, and create-new scratch"]
    fn derives_retained_mes_m2_inventory() {
        let bundle = PathBuf::from(std::env::var(SOURCE_BUNDLE_ENV).unwrap());
        let stage0_root = PathBuf::from(std::env::var(STAGE0_ROOT_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(MES_SCRATCH_ENV).unwrap());

        let report = derive_mes_m2_inventory(MesM2InventoryRequest {
            source_bundle_path: &bundle,
            expected_source_bundle_blake3: crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
            full_stage0_root: &stage0_root,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();

        assert_eq!(report.outputs.len(), MES_OUTPUT_COUNT);
        assert_eq!(report.outputs[0].digest_blake3, report.outputs[1].digest_blake3);
        assert!(scratch.join(MES_REPORT).is_file());
    }
}
