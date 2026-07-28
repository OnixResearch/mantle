use std::collections::VecDeque;
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::{self};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde::Serialize;

use crate::protected_exec_seccomp::ProtectedSeccompSupervisor;
use crate::stagex_sources::StagexSourceMaterializationReport;
use crate::stagex_sources::materialize_authenticated_stagex_sources;

const STAGE0_MINI_SCRIPT_RELATIVE_PATH: &str = "AMD64/mescc-tools-mini-kaem.kaem";
const STAGE0_ABSOLUTE_SCRIPT_RELATIVE_PATH: &str = "AMD64/mescc-tools-mini-absolute.kaem";
const STAGE0_ARTIFACT_DIR_RELATIVE_PATH: &str = "AMD64/artifact";
const STAGE0_BIN_DIR_RELATIVE_PATH: &str = "AMD64/bin";
const STAGE0_HEX0_RELATIVE_PATH: &str = "AMD64/artifact/hex0";
const STAGE0_STDERR_RELATIVE_PATH: &str = "stage0-mini.stderr.txt";
const STAGE0_INVENTORY_REPORT_RELATIVE_PATH: &str = "stage0-mini-inventory.json";
const STAGE0_INVENTORY_FORMAT: &str = "mantle-stagex-stage0-mini-inventory-v1";
const STAGE0_INVENTORY_NON_CLAIM: &str = "this host-orchestrated inventory derives expected executable identities only; it is not protected-exec or provider-admission evidence";
const STAGE0_HEX0_BLAKE3: &str = "cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80";
const STAGE0_KAEM_BLAKE3: &str = "edc664d028b349824cc8c66030a8883b81bdc03d07bc871e44e60ecb4311ab2a";
const STAGE0_SCRIPT_BYTES_MAX: u64 = 64 * 1024;
const STAGE0_EXECUTABLE_BYTES_MAX: u64 = 8 * 1024 * 1024;
const STAGE0_STDERR_BYTES_MAX: u64 = 64 * 1024;
const STAGE0_DIRECTORY_ENTRIES_MAX: u32 = 1024;
const STAGE0_EXECUTABLE_COUNT_MAX: u32 = 128;
const STAGE0_TIMEOUT_MS: u64 = 120_000;
const STAGE0_POLL_INTERVAL_MS: u64 = 10;
const STAGE0_AUDIT_FLUSH_WAIT_MS: u64 = 50;
const EXECUTABLE_PERMISSION_MASK: u32 = 0o111;
const OWNER_EXECUTABLE_MODE: u32 = 0o700;
const ALLOWED_PREEXISTING_OUTPUT_ENTRIES: [&str; 2] = ["README", "placeholder"];
const CURRENT_DIR_PREFIX_CHAR_COUNT: usize = 2;
const STAGE0_EXPECTED_EXECUTABLE_COUNT: usize = 13;
const STAGE0_PLANNED_EXECUTABLE_COUNT: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stage0ExpectedExecutable {
    pub artifact_id: &'static str,
    pub source_stage_id: &'static str,
    pub relative_path: &'static str,
    pub bytes_len: u64,
    pub digest_blake3: &'static str,
}

pub(crate) const STAGE0_EXPECTED_EXECUTABLES: [Stage0ExpectedExecutable; STAGE0_EXPECTED_EXECUTABLE_COUNT] = [
    Stage0ExpectedExecutable {
        artifact_id: "stage0-m0",
        source_stage_id: "stage0-m0",
        relative_path: "AMD64/artifact/M0",
        bytes_len: 1_684,
        digest_blake3: "080eca455a46f2cab998f0ff78b401077294ad612d829397309df7be23fa5a8e",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-m1-bootstrap",
        source_stage_id: "stage0-m1-bootstrap",
        relative_path: "AMD64/artifact/M1-0",
        bytes_len: 54_959,
        digest_blake3: "78be6fd6f49cee594969f3f82c8677a7b771e2907c023380c637628eff92c760",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-m2",
        source_stage_id: "stage0-m2",
        relative_path: "AMD64/artifact/M2",
        bytes_len: 194_297,
        digest_blake3: "2d583c15ad3fa677883b769e7d8e91c7847ea30cff93013e1f0307b5451d70ed",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-blood-elf-bootstrap",
        source_stage_id: "stage0-blood-elf-bootstrap",
        relative_path: "AMD64/artifact/blood-elf-0",
        bytes_len: 23_184,
        digest_blake3: "2f4f5d787f886552ef4600ab8b7e31c612e78b59a341e79ed2911558c9180eda",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-catm",
        source_stage_id: "stage0-catm",
        relative_path: "AMD64/artifact/catm",
        bytes_len: 299,
        digest_blake3: "0d2c2c14eaf77c130f5ffaaea777e7b6f7eab1408acf57c8b516b5f56c451899",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-cc-amd64",
        source_stage_id: "stage0-cc-amd64",
        relative_path: "AMD64/artifact/cc_amd64",
        bytes_len: 17_309,
        digest_blake3: "e91376dee1ac7aa4ef6dc5b86c3e73f08b71fc78b5e069f5e80035955eb971e4",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-hex0-copy",
        source_stage_id: "hex0-reproduction",
        relative_path: "AMD64/artifact/hex0",
        bytes_len: 229,
        digest_blake3: STAGE0_HEX0_BLAKE3,
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-hex1",
        source_stage_id: "stage0-hex1",
        relative_path: "AMD64/artifact/hex1",
        bytes_len: 622,
        digest_blake3: "580b8b771a6dc2b91feb34775ea7152623cc096b37013382f166f2fa9c6d1e40",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-hex2-bootstrap",
        source_stage_id: "stage0-hex2-bootstrap",
        relative_path: "AMD64/artifact/hex2-0",
        bytes_len: 1_519,
        digest_blake3: "84c05689d80bacb397a41eb22c02b3e8e827c648b3b1b5f941d4545decbae0b3",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-hex2-intermediate",
        source_stage_id: "stage0-hex2-intermediate",
        relative_path: "AMD64/artifact/hex2-1",
        bytes_len: 100_049,
        digest_blake3: "2954bef3c2eeae38f0de55556d920c51491f9dd4e74ed482fecbc43c83ef3c1a",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-m1",
        source_stage_id: "stage0-m1",
        relative_path: "AMD64/bin/M1",
        bytes_len: 101_228,
        digest_blake3: "b58f328ad2106ed666e68c485b1e1504ef893e09aed44fbe9805f2a534543642",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-hex2",
        source_stage_id: "stage0-hex2",
        relative_path: "AMD64/bin/hex2",
        bytes_len: 100_049,
        digest_blake3: "2954bef3c2eeae38f0de55556d920c51491f9dd4e74ed482fecbc43c83ef3c1a",
    },
    Stage0ExpectedExecutable {
        artifact_id: "stage0-kaem",
        source_stage_id: "stage0-kaem",
        relative_path: "AMD64/bin/kaem",
        bytes_len: 114_375,
        digest_blake3: "c6c2f9136a41ec16b049e2af0a017d87804d69f4cad882b20eefbbc6725ea854",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Stage0PlannedExecutable {
    pub authorization_id: String,
    pub source_stage_id: String,
    pub path: PathBuf,
    pub digest_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Stage0MiniInventoryRequest<'a> {
    pub source_bundle_path: &'a Path,
    pub expected_source_bundle_blake3: &'a str,
    pub reproduced_hex0_path: &'a Path,
    pub kaem_path: &'a Path,
    pub scratch_dir: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Stage0ExecutableInventoryEntry {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Stage0MiniInventoryReport {
    pub format: &'static str,
    pub source_materialization: StagexSourceMaterializationReport,
    pub script_relative_path: &'static str,
    pub script_digest_blake3: String,
    pub command_count: u32,
    pub executable_count: u32,
    pub executables: Vec<Stage0ExecutableInventoryEntry>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

impl Stage0MiniInventoryReport {
    pub(crate) fn sources_root(&self) -> &Path {
        let first_source = self
            .source_materialization
            .sources
            .first()
            .expect("Stage0 materialization report has required sources");
        let root = first_source.output_path.parent().expect("materialized Stage0 source has a parent root");
        assert!(self.source_materialization.sources.iter().all(|source| source.output_path.parent() == Some(root)));
        assert!(root.is_absolute());
        root
    }
}

#[derive(Debug)]
pub(crate) enum Stage0MiniInventoryError {
    Source(crate::stagex_sources::StagexSourceError),
    InvalidInput(String),
    Io { action: String, source: io::Error },
    ProcessFailure { exit_code: Option<i32>, stderr: String },
    ProcessTimeout { timeout_ms: u64 },
}

impl std::fmt::Display for Stage0MiniInventoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "preparing authenticated Stage0 sources: {error}"),
            Self::InvalidInput(message) => write!(formatter, "invalid Stage0 mini inventory input: {message}"),
            Self::Io { action, source } => write!(formatter, "{action}: {source}"),
            Self::ProcessFailure { exit_code, stderr } => {
                write!(formatter, "Stage0 mini chain failed with status {exit_code:?}: {stderr}")
            }
            Self::ProcessTimeout { timeout_ms } => write!(formatter, "Stage0 mini chain exceeded {timeout_ms} ms"),
        }
    }
}

impl std::error::Error for Stage0MiniInventoryError {}

impl From<crate::stagex_sources::StagexSourceError> for Stage0MiniInventoryError {
    fn from(error: crate::stagex_sources::StagexSourceError) -> Self {
        Self::Source(error)
    }
}

pub(crate) fn planned_stage0_executables(root: &Path) -> Vec<Stage0PlannedExecutable> {
    assert!(root.is_absolute());
    assert_eq!(STAGE0_EXPECTED_EXECUTABLES.len(), STAGE0_EXPECTED_EXECUTABLE_COUNT);
    let planned = STAGE0_EXPECTED_EXECUTABLES
        .iter()
        .filter(|expected| expected.artifact_id != "stage0-kaem")
        .map(|expected| Stage0PlannedExecutable {
            authorization_id: format!("exec:{}", expected.artifact_id),
            source_stage_id: expected.source_stage_id.to_string(),
            path: root.join(expected.relative_path),
            digest_blake3: expected.digest_blake3.to_string(),
        })
        .collect::<Vec<_>>();
    assert_eq!(planned.len(), STAGE0_PLANNED_EXECUTABLE_COUNT);
    assert!(planned.iter().all(|entry| entry.path.is_absolute()));
    planned
}

pub(crate) fn derive_stage0_mini_inventory(
    request: Stage0MiniInventoryRequest<'_>,
) -> Result<Stage0MiniInventoryReport, Stage0MiniInventoryError> {
    derive_stage0_mini_inventory_with_supervisor(request, None)
}

pub(crate) fn derive_protected_stage0_mini_inventory(
    request: Stage0MiniInventoryRequest<'_>,
    supervisor: &ProtectedSeccompSupervisor,
) -> Result<Stage0MiniInventoryReport, Stage0MiniInventoryError> {
    derive_stage0_mini_inventory_with_supervisor(request, Some(supervisor))
}

fn derive_stage0_mini_inventory_with_supervisor(
    request: Stage0MiniInventoryRequest<'_>,
    supervisor: Option<&ProtectedSeccompSupervisor>,
) -> Result<Stage0MiniInventoryReport, Stage0MiniInventoryError> {
    validate_bootstrap_executable(&request, request.reproduced_hex0_path, "reproduced hex0", STAGE0_HEX0_BLAKE3)?;
    validate_bootstrap_executable(&request, request.kaem_path, "kaem-0", STAGE0_KAEM_BLAKE3)?;
    let source_materialization = materialize_authenticated_stagex_sources(
        request.source_bundle_path,
        request.expected_source_bundle_blake3,
        request.scratch_dir,
    )?;
    let prepared = prepare_stage0_mini_workspace(&request)?;
    let audit_events_before = supervisor.map_or(0, |installed| installed.audit_events().len());
    run_stage0_mini_chain(&prepared)?;
    require_protected_audit_growth(supervisor, audit_events_before)?;
    let executables = collect_stage0_executables(request.scratch_dir)?;
    validate_expected_stage0_executables(request.scratch_dir, &executables)?;
    let report = assemble_inventory_report(source_materialization, prepared, executables, supervisor.is_some())?;
    write_inventory_report(request.scratch_dir, &report)?;
    assert!(!report.executables.is_empty());
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn require_protected_audit_growth(
    supervisor: Option<&ProtectedSeccompSupervisor>,
    events_before: usize,
) -> Result<(), Stage0MiniInventoryError> {
    let Some(supervisor) = supervisor else {
        return Ok(());
    };
    thread::sleep(Duration::from_millis(STAGE0_AUDIT_FLUSH_WAIT_MS));
    let events_after = supervisor.audit_events().len();
    if events_after <= events_before {
        return Err(Stage0MiniInventoryError::InvalidInput(
            "protected Stage0 mini run produced no new exec audit events".to_string(),
        ));
    }
    assert!(events_after > events_before);
    assert!(events_before < events_after);
    Ok(())
}

struct PreparedStage0Mini {
    kaem: PathBuf,
    script: PathBuf,
    stderr: PathBuf,
    script_digest_blake3: String,
    command_count: u32,
    current_dir: PathBuf,
}

fn validate_bootstrap_executable(
    request: &Stage0MiniInventoryRequest<'_>,
    path: &Path,
    role: &str,
    expected_blake3: &str,
) -> Result<(), Stage0MiniInventoryError> {
    if path.starts_with(request.scratch_dir) {
        return Err(Stage0MiniInventoryError::InvalidInput(format!(
            "{role} input must be outside create-new scratch {}",
            request.scratch_dir.display()
        )));
    }
    let bytes = read_bounded_file(path, STAGE0_EXECUTABLE_BYTES_MAX, role)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected_blake3 {
        return Err(Stage0MiniInventoryError::InvalidInput(format!(
            "{role} BLAKE3 mismatch: expected {expected_blake3}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn prepare_stage0_mini_workspace(
    request: &Stage0MiniInventoryRequest<'_>,
) -> Result<PreparedStage0Mini, Stage0MiniInventoryError> {
    let artifact_dir = request.scratch_dir.join(STAGE0_ARTIFACT_DIR_RELATIVE_PATH);
    let bin_dir = request.scratch_dir.join(STAGE0_BIN_DIR_RELATIVE_PATH);
    validate_empty_output_directory(&artifact_dir, "Stage0 artifact directory")?;
    validate_empty_output_directory(&bin_dir, "Stage0 binary directory")?;
    let hex0 = request.scratch_dir.join(STAGE0_HEX0_RELATIVE_PATH);
    copy_executable_create_new(request.reproduced_hex0_path, &hex0)?;
    let source_script = request.scratch_dir.join(STAGE0_MINI_SCRIPT_RELATIVE_PATH);
    let script = request.scratch_dir.join(STAGE0_ABSOLUTE_SCRIPT_RELATIVE_PATH);
    let (script_bytes, command_count) = rewrite_stage0_mini_script(&source_script, request.scratch_dir)?;
    write_create_new(&script, &script_bytes)?;
    let script_digest_blake3 = blake3::hash(&script_bytes).to_hex().to_string();
    assert!(command_count > 0);
    assert!(script.is_absolute());
    Ok(PreparedStage0Mini {
        kaem: request.kaem_path.to_path_buf(),
        script,
        stderr: request.scratch_dir.join(STAGE0_STDERR_RELATIVE_PATH),
        script_digest_blake3,
        command_count,
        current_dir: request.scratch_dir.to_path_buf(),
    })
}

fn validate_empty_output_directory(directory: &Path, role: &str) -> Result<(), Stage0MiniInventoryError> {
    if !directory.is_dir() {
        return Err(Stage0MiniInventoryError::InvalidInput(format!("{role} is missing: {}", directory.display())));
    }
    let mut entries_seen = 0u32;
    for entry in fs::read_dir(directory).map_err(|source| io_error(format!("reading {role}"), source))? {
        let entry = entry.map_err(|source| io_error(format!("reading {role} entry"), source))?;
        entries_seen = entries_seen
            .checked_add(1)
            .ok_or_else(|| Stage0MiniInventoryError::InvalidInput(format!("{role} entry count overflow")))?;
        let file_name = entry.file_name();
        let file_name_text = file_name
            .to_str()
            .ok_or_else(|| Stage0MiniInventoryError::InvalidInput(format!("{role} contains a non-UTF-8 entry")))?;
        if !ALLOWED_PREEXISTING_OUTPUT_ENTRIES.contains(&file_name_text) || !entry.path().is_file() {
            return Err(Stage0MiniInventoryError::InvalidInput(format!(
                "{role} contains unexpected preexisting entry {}",
                entry.path().display()
            )));
        }
    }
    assert!(entries_seen <= u32::try_from(ALLOWED_PREEXISTING_OUTPUT_ENTRIES.len()).expect("entry count fits u32"));
    assert!(directory.is_dir());
    Ok(())
}

fn rewrite_stage0_mini_script(
    source_script: &Path,
    absolute_root: &Path,
) -> Result<(Vec<u8>, u32), Stage0MiniInventoryError> {
    if !absolute_root.is_absolute() {
        return Err(Stage0MiniInventoryError::InvalidInput(format!(
            "Stage0 source root is not absolute: {}",
            absolute_root.display()
        )));
    }
    let source = read_bounded_file(source_script, STAGE0_SCRIPT_BYTES_MAX, "Stage0 mini script")?;
    let source_text = std::str::from_utf8(&source)
        .map_err(|error| Stage0MiniInventoryError::InvalidInput(format!("Stage0 mini script is not UTF-8: {error}")))?;
    validate_script_without_shell_features(source_text)?;
    let root_text = absolute_root.to_str().ok_or_else(|| {
        Stage0MiniInventoryError::InvalidInput(format!("Stage0 source root is not UTF-8: {}", absolute_root.display()))
    })?;
    let absolute_prefix = format!("{root_text}/");
    let mut command_count = 0u32;
    let mut rewritten = String::with_capacity(source_text.len().saturating_add(absolute_prefix.len()));
    for line in source_text.lines() {
        if line.starts_with("./") {
            command_count = command_count
                .checked_add(1)
                .ok_or_else(|| Stage0MiniInventoryError::InvalidInput("Stage0 command count overflow".to_string()))?;
        }
        rewritten.push_str(&absolutize_current_dir_paths(line, &absolute_prefix));
        rewritten.push('\n');
    }
    if command_count == 0 {
        return Err(Stage0MiniInventoryError::InvalidInput(
            "Stage0 mini script contains no executable commands".to_string(),
        ));
    }
    assert!(!rewritten.lines().any(|line| line.starts_with("./")));
    assert!(rewritten.lines().filter(|line| line.starts_with(root_text)).count() > 0);
    Ok((rewritten.into_bytes(), command_count))
}

pub(crate) fn absolutize_current_dir_paths(source: &str, absolute_prefix: &str) -> String {
    assert!(absolute_prefix.starts_with('/'));
    assert!(absolute_prefix.ends_with('/'));
    let characters = source.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(source.len().saturating_add(absolute_prefix.len()));
    let mut index = 0usize;
    while index < characters.len() {
        let is_current_dir_prefix = characters[index] == '.'
            && characters.get(index.saturating_add(1)) == Some(&'/')
            && (index == 0 || characters[index - 1] != '.');
        if is_current_dir_prefix {
            output.push_str(absolute_prefix);
            index = index.saturating_add(CURRENT_DIR_PREFIX_CHAR_COUNT);
        } else {
            output.push(characters[index]);
            index = index.saturating_add(1);
        }
    }
    assert_eq!(output.matches("../").count(), source.matches("../").count());
    assert!(output.len() >= source.len().saturating_sub(source.matches("./").count()));
    output
}

fn validate_script_without_shell_features(source: &str) -> Result<(), Stage0MiniInventoryError> {
    const FORBIDDEN_FEATURES: [&str; 6] = ["|", ">", "<", "`", "$(", ";"];
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        for feature in FORBIDDEN_FEATURES {
            if trimmed.contains(feature) {
                return Err(Stage0MiniInventoryError::InvalidInput(format!(
                    "Stage0 mini script uses forbidden shell feature '{feature}'"
                )));
            }
        }
    }
    assert!(source.contains("./AMD64/artifact/hex0"));
    assert!(!source.is_empty());
    Ok(())
}

fn run_stage0_mini_chain(prepared: &PreparedStage0Mini) -> Result<(), Stage0MiniInventoryError> {
    let stderr_file =
        File::create(&prepared.stderr).map_err(|source| io_error("creating Stage0 stderr log", source))?;
    let mut child = Command::new(&prepared.kaem)
        .arg(&prepared.script)
        .current_dir(&prepared.current_dir)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|source| io_error("spawning source-built kaem-0", source))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|source| io_error("waiting for source-built kaem-0", source))? {
            return classify_stage0_status(status, &prepared.stderr);
        }
        if started.elapsed() >= Duration::from_millis(STAGE0_TIMEOUT_MS) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Stage0MiniInventoryError::ProcessTimeout {
                timeout_ms: STAGE0_TIMEOUT_MS,
            });
        }
        thread::sleep(Duration::from_millis(STAGE0_POLL_INTERVAL_MS));
    }
}

fn classify_stage0_status(status: ExitStatus, stderr_path: &Path) -> Result<(), Stage0MiniInventoryError> {
    let stderr = read_bounded_file(stderr_path, STAGE0_STDERR_BYTES_MAX, "Stage0 stderr")?;
    let stderr_text = String::from_utf8_lossy(&stderr).into_owned();
    if !status.success() {
        return Err(Stage0MiniInventoryError::ProcessFailure {
            exit_code: status.code(),
            stderr: stderr_text,
        });
    }
    assert!(status.code().is_none() || status.code() == Some(0));
    assert!(stderr.len() <= usize::try_from(STAGE0_STDERR_BYTES_MAX).expect("stderr limit fits usize"));
    Ok(())
}

fn collect_stage0_executables(root: &Path) -> Result<Vec<Stage0ExecutableInventoryEntry>, Stage0MiniInventoryError> {
    let mut queue = VecDeque::from([
        root.join(STAGE0_ARTIFACT_DIR_RELATIVE_PATH),
        root.join(STAGE0_BIN_DIR_RELATIVE_PATH),
    ]);
    let mut entries_seen = 0u32;
    let mut executables = Vec::new();
    while let Some(directory) = queue.pop_front() {
        collect_directory_entries(root, &directory, &mut queue, &mut entries_seen, &mut executables)?;
    }
    executables.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    if executables.is_empty() {
        return Err(Stage0MiniInventoryError::InvalidInput(
            "Stage0 mini chain produced no executable files".to_string(),
        ));
    }
    assert!(entries_seen <= STAGE0_DIRECTORY_ENTRIES_MAX);
    assert!(executables.len() <= usize::try_from(STAGE0_EXECUTABLE_COUNT_MAX).expect("count fits usize"));
    Ok(executables)
}

fn collect_directory_entries(
    root: &Path,
    directory: &Path,
    queue: &mut VecDeque<PathBuf>,
    entries_seen: &mut u32,
    executables: &mut Vec<Stage0ExecutableInventoryEntry>,
) -> Result<(), Stage0MiniInventoryError> {
    let entries = fs::read_dir(directory).map_err(|source| io_error("reading Stage0 output directory", source))?;
    for entry in entries {
        *entries_seen = entries_seen.checked_add(1).ok_or_else(|| {
            Stage0MiniInventoryError::InvalidInput("Stage0 directory entry count overflow".to_string())
        })?;
        if *entries_seen > STAGE0_DIRECTORY_ENTRIES_MAX {
            return Err(Stage0MiniInventoryError::InvalidInput(format!(
                "Stage0 output exceeds {STAGE0_DIRECTORY_ENTRIES_MAX} directory entries"
            )));
        }
        let entry = entry.map_err(|source| io_error("reading Stage0 output entry", source))?;
        classify_output_entry(root, entry.path(), queue, executables)?;
    }
    assert!(*entries_seen <= STAGE0_DIRECTORY_ENTRIES_MAX);
    assert!(executables.len() <= usize::try_from(STAGE0_EXECUTABLE_COUNT_MAX).expect("count fits usize"));
    Ok(())
}

fn classify_output_entry(
    root: &Path,
    path: PathBuf,
    queue: &mut VecDeque<PathBuf>,
    executables: &mut Vec<Stage0ExecutableInventoryEntry>,
) -> Result<(), Stage0MiniInventoryError> {
    let metadata = fs::symlink_metadata(&path).map_err(|source| io_error("reading Stage0 output metadata", source))?;
    if metadata.is_dir() {
        queue.push_back(path);
        return Ok(());
    }
    if !metadata.is_file() || metadata.permissions().mode() & EXECUTABLE_PERMISSION_MASK == 0 {
        return Ok(());
    }
    if executables.len() >= usize::try_from(STAGE0_EXECUTABLE_COUNT_MAX).expect("count fits usize") {
        return Err(Stage0MiniInventoryError::InvalidInput(format!(
            "Stage0 output exceeds {STAGE0_EXECUTABLE_COUNT_MAX} executable files"
        )));
    }
    let bytes = read_bounded_file(&path, STAGE0_EXECUTABLE_BYTES_MAX, "Stage0 executable output")?;
    let relative_path = path
        .strip_prefix(root)
        .map_err(|error| Stage0MiniInventoryError::InvalidInput(format!("Stage0 output escapes scratch: {error}")))?
        .to_str()
        .ok_or_else(|| Stage0MiniInventoryError::InvalidInput("Stage0 output path is not UTF-8".to_string()))?
        .to_string();
    executables.push(Stage0ExecutableInventoryEntry {
        relative_path,
        absolute_path: path,
        bytes_len: u64::try_from(bytes.len())
            .map_err(|_| Stage0MiniInventoryError::InvalidInput("Stage0 output size does not fit u64".to_string()))?,
        digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
    });
    Ok(())
}

fn validate_expected_stage0_executables(
    root: &Path,
    executables: &[Stage0ExecutableInventoryEntry],
) -> Result<(), Stage0MiniInventoryError> {
    if executables.len() != STAGE0_EXPECTED_EXECUTABLE_COUNT {
        return Err(Stage0MiniInventoryError::InvalidInput(format!(
            "Stage0 mini chain produced {} executables; expected {STAGE0_EXPECTED_EXECUTABLE_COUNT}",
            executables.len()
        )));
    }
    for (observed, expected) in executables.iter().zip(STAGE0_EXPECTED_EXECUTABLES) {
        let expected_path = root.join(expected.relative_path);
        if observed.relative_path != expected.relative_path
            || observed.absolute_path != expected_path
            || observed.bytes_len != expected.bytes_len
            || observed.digest_blake3 != expected.digest_blake3
        {
            return Err(Stage0MiniInventoryError::InvalidInput(format!(
                "Stage0 executable {} does not match its expected path, size, and BLAKE3",
                expected.artifact_id
            )));
        }
    }
    assert_eq!(executables.len(), STAGE0_EXPECTED_EXECUTABLES.len());
    assert!(executables.iter().all(|entry| entry.absolute_path.starts_with(root)));
    Ok(())
}

fn assemble_inventory_report(
    source_materialization: StagexSourceMaterializationReport,
    prepared: PreparedStage0Mini,
    executables: Vec<Stage0ExecutableInventoryEntry>,
    protected_exec_enforced: bool,
) -> Result<Stage0MiniInventoryReport, Stage0MiniInventoryError> {
    let executable_count = u32::try_from(executables.len())
        .map_err(|_| Stage0MiniInventoryError::InvalidInput("Stage0 executable count does not fit u32".to_string()))?;
    if executable_count > STAGE0_EXECUTABLE_COUNT_MAX {
        return Err(Stage0MiniInventoryError::InvalidInput(format!(
            "Stage0 executable count {executable_count} exceeds {STAGE0_EXECUTABLE_COUNT_MAX}"
        )));
    }
    assert!(prepared.command_count > 0);
    assert!(!prepared.script_digest_blake3.is_empty());
    Ok(Stage0MiniInventoryReport {
        format: STAGE0_INVENTORY_FORMAT,
        source_materialization,
        script_relative_path: STAGE0_ABSOLUTE_SCRIPT_RELATIVE_PATH,
        script_digest_blake3: prepared.script_digest_blake3,
        command_count: prepared.command_count,
        executable_count,
        executables,
        protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: STAGE0_INVENTORY_NON_CLAIM,
    })
}

fn write_inventory_report(root: &Path, report: &Stage0MiniInventoryReport) -> Result<(), Stage0MiniInventoryError> {
    let report_path = root.join(STAGE0_INVENTORY_REPORT_RELATIVE_PATH);
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| Stage0MiniInventoryError::InvalidInput(format!("serializing Stage0 inventory: {error}")))?;
    write_create_new(&report_path, &bytes)?;
    assert!(report_path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn copy_executable_create_new(source: &Path, destination: &Path) -> Result<(), Stage0MiniInventoryError> {
    let bytes = read_bounded_file(source, STAGE0_EXECUTABLE_BYTES_MAX, "bootstrap executable")?;
    write_create_new(destination, &bytes)?;
    fs::set_permissions(destination, fs::Permissions::from_mode(OWNER_EXECUTABLE_MODE))
        .map_err(|source| io_error("setting bootstrap executable permissions", source))?;
    assert!(destination.is_file());
    assert_eq!(
        fs::metadata(destination).map_err(|source| io_error("reading copied executable", source))?.len(),
        u64::try_from(bytes.len()).expect("copied executable size fits u64")
    );
    Ok(())
}

fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), Stage0MiniInventoryError> {
    use std::io::Write;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path).map_err(|source| io_error("creating Stage0 output", source))?;
    file.write_all(bytes).map_err(|source| io_error("writing Stage0 output", source))?;
    file.sync_all().map_err(|source| io_error("syncing Stage0 output", source))?;
    assert!(path.is_file());
    assert_eq!(
        fs::metadata(path).map_err(|source| io_error("reading Stage0 output", source))?.len(),
        u64::try_from(bytes.len()).expect("Stage0 output size fits u64")
    );
    Ok(())
}

fn read_bounded_file(path: &Path, bytes_max: u64, role: &str) -> Result<Vec<u8>, Stage0MiniInventoryError> {
    let metadata = fs::metadata(path).map_err(|source| io_error(format!("reading {role} metadata"), source))?;
    if !metadata.is_file() || metadata.len() > bytes_max {
        return Err(Stage0MiniInventoryError::InvalidInput(format!(
            "{role} at {} is not a regular file within {bytes_max} bytes",
            path.display()
        )));
    }
    let capacity = usize::try_from(metadata.len())
        .map_err(|_| Stage0MiniInventoryError::InvalidInput(format!("{role} size does not fit usize")))?;
    let mut bytes = Vec::with_capacity(capacity);
    File::open(path)
        .map_err(|source| io_error(format!("opening {role}"), source))?
        .take(bytes_max.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(format!("reading {role}"), source))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > bytes_max {
        return Err(Stage0MiniInventoryError::InvalidInput(format!("{role} exceeds {bytes_max} bytes")));
    }
    assert_eq!(bytes.len(), capacity);
    assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= bytes_max);
    Ok(bytes)
}

fn io_error(action: impl Into<String>, source: io::Error) -> Stage0MiniInventoryError {
    Stage0MiniInventoryError::Io {
        action: action.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROOT: &str = "/tmp/mantle-stagex-test-root";
    const FULL_SOURCE_CLOSURE_BLAKE3: &str = "7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45";
    const RETAINED_SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const RETAINED_TRANSITION_DIR_ENV: &str = "MANTLE_STAGE_X_TRANSITION_DIR";
    const RETAINED_STAGE0_SCRATCH_ENV: &str = "MANTLE_STAGE_X_STAGE0_SCRATCH";

    #[test]
    fn rewrites_every_mini_chain_command_to_an_absolute_executable() {
        let temp = tempfile::tempdir().unwrap();
        let script = temp.path().join("mini.kaem");
        fs::write(&script, "# comment\n./AMD64/artifact/hex0 ./source ./output\n\t-f ./input\n").unwrap();

        let (rewritten, command_count) = rewrite_stage0_mini_script(&script, Path::new(TEST_ROOT)).unwrap();
        let text = String::from_utf8(rewritten).unwrap();

        assert_eq!(command_count, 1);
        assert!(text.contains("/tmp/mantle-stagex-test-root/AMD64/artifact/hex0"));
        assert!(!text.contains("./AMD64/artifact/hex0"));
    }

    #[test]
    fn rejects_shell_feature_in_mini_chain_script() {
        let temp = tempfile::tempdir().unwrap();
        let script = temp.path().join("mini.kaem");
        fs::write(&script, "./tool input > output\n").unwrap();

        let error = rewrite_stage0_mini_script(&script, Path::new(TEST_ROOT)).unwrap_err();

        assert!(error.to_string().contains("forbidden shell feature '>'"));
        assert!(!error.to_string().contains("contains no executable commands"));
    }

    #[test]
    #[ignore = "derives retained host-orchestrated inventory; not protected-transition evidence"]
    fn derives_retained_stage0_mini_executable_inventory() {
        let bundle = PathBuf::from(std::env::var(RETAINED_SOURCE_BUNDLE_ENV).unwrap());
        let transition_dir = PathBuf::from(std::env::var(RETAINED_TRANSITION_DIR_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(RETAINED_STAGE0_SCRATCH_ENV).unwrap());

        let report = derive_stage0_mini_inventory(Stage0MiniInventoryRequest {
            source_bundle_path: &bundle,
            expected_source_bundle_blake3: FULL_SOURCE_CLOSURE_BLAKE3,
            reproduced_hex0_path: &transition_dir.join("hex0-reproduced"),
            kaem_path: &transition_dir.join("kaem-0"),
            scratch_dir: &scratch,
        })
        .unwrap();

        assert!(report.executable_count > 1);
        assert!(!report.protected_exec_enforced);
        assert!(report.fallback_events.is_empty());
        assert!(scratch.join(STAGE0_INVENTORY_REPORT_RELATIVE_PATH).is_file());
    }
}
