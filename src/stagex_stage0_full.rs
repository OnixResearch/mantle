use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
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
use crate::stagex_stage0::Stage0MiniInventoryError;
use crate::stagex_stage0::Stage0MiniInventoryReport;
use crate::stagex_stage0::Stage0MiniInventoryRequest;
use crate::stagex_stage0::absolutize_current_dir_paths;
use crate::stagex_stage0::derive_stage0_mini_inventory;

const STAGE0_FULL_ROOT_SCRIPT_SOURCE: &str = "AMD64/kaem.run";
const STAGE0_FULL_NESTED_SCRIPT_SOURCE: &str = "AMD64/mescc-tools-full-kaem.kaem";
const STAGE0_FULL_ROOT_SCRIPT_OUTPUT: &str = "stage0-full-absolute.kaem";
const STAGE0_FULL_NESTED_SCRIPT_OUTPUT: &str = "AMD64/mescc-tools-full-absolute.kaem";
const STAGE0_FULL_AFTER_SCRIPT_OUTPUT: &str = "after.kaem";
const STAGE0_FULL_ANSWERS_OUTPUT: &str = "amd64.answers";
const STAGE0_FULL_STDERR_OUTPUT: &str = "stage0-full.stderr.txt";
const STAGE0_FULL_REPORT_OUTPUT: &str = "stage0-full-inventory.json";
const STAGE0_FULL_REPORT_FORMAT: &str = "mantle-stagex-stage0-full-inventory-v1";
const STAGE0_FULL_NON_CLAIM: &str = "this host-orchestrated inventory derives full Stage0 executable identities only; it is not protected-exec or normalized-provider evidence";
pub(crate) const STAGE0_ANSWERS_BLAKE3: &str = "0ddbb5dad8868a17ddcb7e36bcd981e822f8f374c057ad7d40b5ea5fdf160765";
const STAGE0_FULL_SCRIPT_BYTES_MAX: u64 = 64 * 1024;
const STAGE0_FULL_EXECUTABLE_BYTES_MAX: u64 = 8 * 1024 * 1024;
const STAGE0_FULL_STDERR_BYTES_MAX: u64 = 64 * 1024;
const STAGE0_FULL_EXECUTABLE_COUNT_MAX: u32 = 64;
const STAGE0_FULL_TIMEOUT_MS: u64 = 120_000;
const STAGE0_FULL_POLL_INTERVAL_MS: u64 = 10;
const EXECUTABLE_PERMISSION_MASK: u32 = 0o111;
const STAGE0_FULL_KAEM_RELATIVE_PATH: &str = "AMD64/bin/kaem";
const STAGE0_FULL_BIN_RELATIVE_PATH: &str = "AMD64/bin";
const BINDIR_ASSIGNMENT: &str = "BINDIR=\"../${ARCH_DIR}/bin\"";
const TOOLS_ASSIGNMENT: &str = "TOOLS=\"../${ARCH_DIR}/bin\"";
const FULL_SCRIPT_REFERENCE: &str = "${ARCH_DIR}/mescc-tools-full-kaem.kaem";
const STAGE0_FULL_AUDIT_FLUSH_WAIT_MS: u64 = 50;
const STAGE0_FULL_EXPECTED_EXECUTABLE_COUNT: usize = 20;
const STAGE0_FULL_PLANNED_EXECUTABLE_COUNT: usize = 5;
pub(crate) const STAGE0_FULL_EXTRA_COMMAND_COUNT: u32 = 13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stage0FullExpectedExecutable {
    pub artifact_id: &'static str,
    pub relative_path: &'static str,
    pub bytes_len: u64,
    pub digest_blake3: &'static str,
}

pub(crate) const STAGE0_FULL_EXPECTED_EXECUTABLES: [Stage0FullExpectedExecutable;
    STAGE0_FULL_EXPECTED_EXECUTABLE_COUNT] = [
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-m1",
        relative_path: "AMD64/bin/M1",
        bytes_len: 101_228,
        digest_blake3: "b58f328ad2106ed666e68c485b1e1504ef893e09aed44fbe9805f2a534543642",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-m2-mesoplanet",
        relative_path: "AMD64/bin/M2-Mesoplanet",
        bytes_len: 185_277,
        digest_blake3: "e473c5670f6a2cfc0af270d75c72405930c24d741febbe4577d28f4cbf90cd36",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-m2-planet",
        relative_path: "AMD64/bin/M2-Planet",
        bytes_len: 458_747,
        digest_blake3: "9133f106b326b9badb5945fa50a7a5f36cffa2bdea76ad0568328cb06685f166",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-blood-elf",
        relative_path: "AMD64/bin/blood-elf",
        bytes_len: 76_419,
        digest_blake3: "23154d5fdfc6b669714a5551587feb34d4964759729dd28bc51581960fd19c25",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-catm",
        relative_path: "AMD64/bin/catm",
        bytes_len: 42_650,
        digest_blake3: "2ecde44089def1bb3496368f90d17fbba30d8938cf4109383375e3845ce52be3",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-chmod",
        relative_path: "AMD64/bin/chmod",
        bytes_len: 61_587,
        digest_blake3: "1ce14e51cd2020400ecaa356a8d67cb8fa6d53357a63e236c45901ee9759ac63",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-cp",
        relative_path: "AMD64/bin/cp",
        bytes_len: 70_003,
        digest_blake3: "636b9ee7cc548bfb59fd3425d9fd7cdfe9340766f2ea23e85c98f76f5276c23b",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-get-machine",
        relative_path: "AMD64/bin/get_machine",
        bytes_len: 57_101,
        digest_blake3: "94eb62461a1e94ca7fef4c5c90fa017505696dc22e3235df6b9cc341cf5234e3",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-hex2",
        relative_path: "AMD64/bin/hex2",
        bytes_len: 100_049,
        digest_blake3: "2954bef3c2eeae38f0de55556d920c51491f9dd4e74ed482fecbc43c83ef3c1a",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-kaem",
        relative_path: "AMD64/bin/kaem",
        bytes_len: 114_375,
        digest_blake3: "c6c2f9136a41ec16b049e2af0a017d87804d69f4cad882b20eefbbc6725ea854",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-match",
        relative_path: "AMD64/bin/match",
        bytes_len: 58_439,
        digest_blake3: "b6862c346146a63d7e334c3e057e54eb77ce841e788583f0d0a824879cff0c97",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-mkdir",
        relative_path: "AMD64/bin/mkdir",
        bytes_len: 61_309,
        digest_blake3: "a023e38258367002b7f9dde5fcb49cec445c34cce08a8b63c8933fa3fab5cf62",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-replace",
        relative_path: "AMD64/bin/replace",
        bytes_len: 65_367,
        digest_blake3: "4e67d5271e052629f8d92a11979ec8a93e7d8c416070544b71266d189f37dbbd",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-rm",
        relative_path: "AMD64/bin/rm",
        bytes_len: 58_410,
        digest_blake3: "e7a77376f5a41340b42929b679da8af84ff508ceda13b0f077e6f77e06423e20",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-sha256sum",
        relative_path: "AMD64/bin/sha256sum",
        bytes_len: 78_520,
        digest_blake3: "c507561a47ca67648247eaa06d054193d9aac21160708a7671987dc2f1c48a61",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-unbz2",
        relative_path: "AMD64/bin/unbz2",
        bytes_len: 92_005,
        digest_blake3: "51319fe1892cd3b43fee2751ef3d00ec4ae1ff2a70a6fa72c0764cf9d9e75010",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-ungz",
        relative_path: "AMD64/bin/ungz",
        bytes_len: 93_266,
        digest_blake3: "d7e6dae760f6635196ebd80f203aada32ad83eb8fb0d013b7b09f879dc45290d",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-untar",
        relative_path: "AMD64/bin/untar",
        bytes_len: 74_706,
        digest_blake3: "794461d61868aa601955182f5abc1fbc6f840a1402ece32855c41b59efb2bdf0",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-unxz",
        relative_path: "AMD64/bin/unxz",
        bytes_len: 153_444,
        digest_blake3: "8a1543cab9d6290e9c9306e16efc83ac0d230307c9c394b048e2d0b41b719771",
    },
    Stage0FullExpectedExecutable {
        artifact_id: "stage0-full-wrap",
        relative_path: "AMD64/bin/wrap",
        bytes_len: 67_744,
        digest_blake3: "1d08486cfb920032a821c7603f56410af6c0372c0a4b78546d6c8d8bb1c0f9e0",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Stage0FullPlannedExecutable {
    pub authorization_id: String,
    pub source_stage_id: String,
    pub path: PathBuf,
    pub digest_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Stage0FullInventoryRequest<'a> {
    pub mini: Stage0MiniInventoryRequest<'a>,
    pub answers_path: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Stage0FullExecutableInventoryEntry {
    pub name: String,
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Stage0FullInventoryReport {
    pub format: &'static str,
    pub mini: Stage0MiniInventoryReport,
    pub answers_digest_blake3: String,
    pub root_script_digest_blake3: String,
    pub nested_script_digest_blake3: String,
    pub root_command_count: u32,
    pub nested_command_count: u32,
    pub extra_command_count: u32,
    pub executable_count: u32,
    pub executables: Vec<Stage0FullExecutableInventoryEntry>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) enum Stage0FullInventoryError {
    Mini(Stage0MiniInventoryError),
    InvalidInput(String),
    Io { action: String, source: io::Error },
    ProcessFailure { exit_code: Option<i32>, stderr: String },
    ProcessTimeout { timeout_ms: u64 },
}

impl std::fmt::Display for Stage0FullInventoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mini(error) => write!(formatter, "materializing Stage0 mini predecessor: {error}"),
            Self::InvalidInput(message) => write!(formatter, "invalid full Stage0 inventory input: {message}"),
            Self::Io { action, source } => write!(formatter, "{action}: {source}"),
            Self::ProcessFailure { exit_code, stderr } => {
                write!(formatter, "full Stage0 chain failed with status {exit_code:?}: {stderr}")
            }
            Self::ProcessTimeout { timeout_ms } => write!(formatter, "full Stage0 chain exceeded {timeout_ms} ms"),
        }
    }
}

impl std::error::Error for Stage0FullInventoryError {}

impl From<Stage0MiniInventoryError> for Stage0FullInventoryError {
    fn from(error: Stage0MiniInventoryError) -> Self {
        Self::Mini(error)
    }
}

pub(crate) fn planned_stage0_full_executables(root: &Path) -> Vec<Stage0FullPlannedExecutable> {
    const PLANNED: [(&str, &str); STAGE0_FULL_PLANNED_EXECUTABLE_COUNT] = [
        ("stage0-kaem", "stage0-kaem"),
        ("stage0-full-m2-mesoplanet", "stage0-full-m2-mesoplanet"),
        ("stage0-full-m2-planet", "stage0-full-m2-planet"),
        ("stage0-full-blood-elf", "stage0-full-blood-elf"),
        ("stage0-full-sha256sum", "stage0-full-extras"),
    ];
    let planned = PLANNED
        .iter()
        .map(|(artifact_id, source_stage_id)| {
            let expected = STAGE0_FULL_EXPECTED_EXECUTABLES
                .iter()
                .find(|expected| expected.artifact_id == *artifact_id)
                .expect("planned full Stage0 executable has expected identity");
            Stage0FullPlannedExecutable {
                authorization_id: format!("exec:{artifact_id}"),
                source_stage_id: (*source_stage_id).to_string(),
                path: root.join(expected.relative_path),
                digest_blake3: expected.digest_blake3.to_string(),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(planned.len(), STAGE0_FULL_PLANNED_EXECUTABLE_COUNT);
    assert!(planned.iter().all(|entry| entry.path.is_absolute()));
    planned
}

pub(crate) fn derive_stage0_full_inventory(
    request: Stage0FullInventoryRequest<'_>,
) -> Result<Stage0FullInventoryReport, Stage0FullInventoryError> {
    let mini = derive_stage0_mini_inventory(request.mini.clone())?;
    derive_stage0_full_inventory_from_mini(mini, request.answers_path, None)
}

pub(crate) fn derive_protected_stage0_full_inventory(
    mini: Stage0MiniInventoryReport,
    answers_path: &Path,
    supervisor: &ProtectedSeccompSupervisor,
) -> Result<Stage0FullInventoryReport, Stage0FullInventoryError> {
    derive_stage0_full_inventory_from_mini(mini, answers_path, Some(supervisor))
}

fn derive_stage0_full_inventory_from_mini(
    mini: Stage0MiniInventoryReport,
    answers_path: &Path,
    supervisor: Option<&ProtectedSeccompSupervisor>,
) -> Result<Stage0FullInventoryReport, Stage0FullInventoryError> {
    let answers = read_bound_answers(answers_path)?;
    let root = mini.sources_root().to_path_buf();
    let prepared = prepare_full_stage0(&root, &answers)?;
    let audit_events_before = supervisor.map_or(0, |installed| installed.audit_events().len());
    run_full_stage0(&root, &prepared)?;
    require_protected_audit_growth(supervisor, audit_events_before)?;
    let executables = collect_full_stage0_executables(&root)?;
    validate_expected_full_stage0_executables(&root, &executables)?;
    let report = assemble_full_report(mini, prepared, executables, supervisor.is_some())?;
    write_report(&root, &report)?;
    assert!(!report.executables.is_empty());
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn require_protected_audit_growth(
    supervisor: Option<&ProtectedSeccompSupervisor>,
    events_before: usize,
) -> Result<(), Stage0FullInventoryError> {
    let Some(supervisor) = supervisor else {
        return Ok(());
    };
    thread::sleep(Duration::from_millis(STAGE0_FULL_AUDIT_FLUSH_WAIT_MS));
    let events_after = supervisor.audit_events().len();
    if events_after <= events_before {
        return Err(Stage0FullInventoryError::InvalidInput(
            "protected full Stage0 run produced no new exec audit events".to_string(),
        ));
    }
    assert!(events_after > events_before);
    assert!(events_before < events_after);
    Ok(())
}

struct PreparedFullStage0 {
    root_script: PathBuf,
    stderr: PathBuf,
    root_script_digest_blake3: String,
    nested_script_digest_blake3: String,
    root_command_count: u32,
    nested_command_count: u32,
}

fn read_bound_answers(path: &Path) -> Result<Vec<u8>, Stage0FullInventoryError> {
    let bytes = read_bounded_file(path, STAGE0_FULL_SCRIPT_BYTES_MAX, "Stage0 SHA-256 answers")?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != STAGE0_ANSWERS_BLAKE3 {
        return Err(Stage0FullInventoryError::InvalidInput(format!(
            "Stage0 answers BLAKE3 mismatch: expected {STAGE0_ANSWERS_BLAKE3}, observed {observed}"
        )));
    }
    assert!(!bytes.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(bytes)
}

fn prepare_full_stage0(root: &Path, answers: &[u8]) -> Result<PreparedFullStage0, Stage0FullInventoryError> {
    let nested_source = root.join(STAGE0_FULL_NESTED_SCRIPT_SOURCE);
    let nested_output = root.join(STAGE0_FULL_NESTED_SCRIPT_OUTPUT);
    let nested_base = root.join("AMD64");
    let (nested_bytes, nested_command_count) = rewrite_direct_commands(&nested_source, &nested_base)?;
    write_create_new(&nested_output, &nested_bytes)?;
    let root_source = root.join(STAGE0_FULL_ROOT_SCRIPT_SOURCE);
    let root_output = root.join(STAGE0_FULL_ROOT_SCRIPT_OUTPUT);
    let (root_bytes, root_command_count) = rewrite_root_script(&root_source, root, &nested_output)?;
    write_create_new(&root_output, &root_bytes)?;
    write_create_new(&root.join(STAGE0_FULL_ANSWERS_OUTPUT), answers)?;
    write_create_new(&root.join(STAGE0_FULL_AFTER_SCRIPT_OUTPUT), &[])?;
    assert!(root_output.is_file());
    assert!(nested_output.is_file());
    Ok(PreparedFullStage0 {
        root_script: root_output,
        stderr: root.join(STAGE0_FULL_STDERR_OUTPUT),
        root_script_digest_blake3: blake3::hash(&root_bytes).to_hex().to_string(),
        nested_script_digest_blake3: blake3::hash(&nested_bytes).to_hex().to_string(),
        root_command_count,
        nested_command_count,
    })
}

fn rewrite_direct_commands(source: &Path, command_root: &Path) -> Result<(Vec<u8>, u32), Stage0FullInventoryError> {
    let source_bytes = read_bounded_file(source, STAGE0_FULL_SCRIPT_BYTES_MAX, "full Stage0 nested script")?;
    let source_text = std::str::from_utf8(&source_bytes).map_err(|error| {
        Stage0FullInventoryError::InvalidInput(format!("nested Stage0 script is not UTF-8: {error}"))
    })?;
    let root_text = absolute_utf8_path(command_root, "nested Stage0 command root")?;
    let prefix = format!("{root_text}/");
    let mut rewritten = String::with_capacity(source_text.len().saturating_add(prefix.len()));
    let mut command_count = 0u32;
    for line in source_text.lines() {
        if line.starts_with("./") {
            command_count = command_count
                .checked_add(1)
                .ok_or_else(|| Stage0FullInventoryError::InvalidInput("nested command count overflow".to_string()))?;
        }
        rewritten.push_str(&absolutize_current_dir_paths(line, &prefix));
        rewritten.push('\n');
    }
    if command_count == 0 {
        return Err(Stage0FullInventoryError::InvalidInput(
            "full Stage0 nested script has no direct commands".to_string(),
        ));
    }
    assert!(!rewritten.lines().any(|line| line.starts_with("./")));
    assert!(rewritten.lines().any(|line| line.starts_with(root_text)));
    Ok((rewritten.into_bytes(), command_count))
}

fn rewrite_root_script(
    source: &Path,
    root: &Path,
    nested_script: &Path,
) -> Result<(Vec<u8>, u32), Stage0FullInventoryError> {
    let source_bytes = read_bounded_file(source, STAGE0_FULL_SCRIPT_BYTES_MAX, "full Stage0 root script")?;
    let source_text = std::str::from_utf8(&source_bytes)
        .map_err(|error| Stage0FullInventoryError::InvalidInput(format!("root Stage0 script is not UTF-8: {error}")))?;
    require_root_script_seams(source_text)?;
    let root_text = absolute_utf8_path(root, "full Stage0 root")?;
    let nested_text = absolute_utf8_path(nested_script, "full Stage0 nested script")?;
    let bin_assignment = format!("{}=\"{root_text}/${{ARCH_DIR}}/bin\"", "BINDIR");
    let tools_assignment = format!("{}=\"{root_text}/${{ARCH_DIR}}/bin\"", "TOOLS");
    let mut rewritten = source_text.replace(BINDIR_ASSIGNMENT, &bin_assignment);
    rewritten = rewritten.replace(TOOLS_ASSIGNMENT, &tools_assignment);
    rewritten = rewritten.replace(FULL_SCRIPT_REFERENCE, nested_text);
    rewritten = absolutize_current_dir_paths(&rewritten, &format!("{root_text}/"));
    let command_count = rewritten
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with(root_text) || trimmed.starts_with("${BINDIR}/") || trimmed.starts_with("exec ")
        })
        .count();
    let command_count = u32::try_from(command_count)
        .map_err(|_| Stage0FullInventoryError::InvalidInput("root command count does not fit u32".to_string()))?;
    if command_count == 0 {
        return Err(Stage0FullInventoryError::InvalidInput(
            "full Stage0 root script has no executable commands".to_string(),
        ));
    }
    assert!(!rewritten.contains(BINDIR_ASSIGNMENT));
    assert!(!rewritten.contains(FULL_SCRIPT_REFERENCE));
    Ok((rewritten.into_bytes(), command_count))
}

fn require_root_script_seams(source: &str) -> Result<(), Stage0FullInventoryError> {
    for required in [
        BINDIR_ASSIGNMENT,
        TOOLS_ASSIGNMENT,
        FULL_SCRIPT_REFERENCE,
        "exec ./${ARCH_DIR}/bin/kaem",
    ] {
        if !source.contains(required) {
            return Err(Stage0FullInventoryError::InvalidInput(format!(
                "full Stage0 root script is missing required seam '{required}'"
            )));
        }
    }
    assert!(source.contains("sha256sum -c"));
    assert!(source.contains("mescc-tools-extra.kaem"));
    Ok(())
}

fn run_full_stage0(root: &Path, prepared: &PreparedFullStage0) -> Result<(), Stage0FullInventoryError> {
    let kaem = root.join(STAGE0_FULL_KAEM_RELATIVE_PATH);
    let stderr_file =
        File::create(&prepared.stderr).map_err(|source| io_error("creating full Stage0 stderr", source))?;
    let mut child = Command::new(&kaem)
        .args(["--verbose", "--strict", "--file"])
        .arg(&prepared.root_script)
        .current_dir(root)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|source| io_error("spawning full source-built Stage0 kaem", source))?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|source| io_error("waiting for full Stage0 kaem", source))? {
            return classify_status(status, &prepared.stderr);
        }
        if started.elapsed() >= Duration::from_millis(STAGE0_FULL_TIMEOUT_MS) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Stage0FullInventoryError::ProcessTimeout {
                timeout_ms: STAGE0_FULL_TIMEOUT_MS,
            });
        }
        thread::sleep(Duration::from_millis(STAGE0_FULL_POLL_INTERVAL_MS));
    }
}

fn classify_status(status: ExitStatus, stderr_path: &Path) -> Result<(), Stage0FullInventoryError> {
    let stderr = read_bounded_file(stderr_path, STAGE0_FULL_STDERR_BYTES_MAX, "full Stage0 stderr")?;
    if !status.success() {
        return Err(Stage0FullInventoryError::ProcessFailure {
            exit_code: status.code(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        });
    }
    assert!(status.code().is_none() || status.code() == Some(0));
    assert!(u64::try_from(stderr.len()).unwrap_or(u64::MAX) <= STAGE0_FULL_STDERR_BYTES_MAX);
    Ok(())
}

fn collect_full_stage0_executables(
    root: &Path,
) -> Result<Vec<Stage0FullExecutableInventoryEntry>, Stage0FullInventoryError> {
    let bin_dir = root.join(STAGE0_FULL_BIN_RELATIVE_PATH);
    let mut executables = Vec::new();
    for entry in fs::read_dir(&bin_dir).map_err(|source| io_error("reading full Stage0 bin directory", source))? {
        let entry = entry.map_err(|source| io_error("reading full Stage0 bin entry", source))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| io_error("reading full Stage0 output", source))?;
        if !metadata.is_file() || metadata.permissions().mode() & EXECUTABLE_PERMISSION_MASK == 0 {
            continue;
        }
        if executables.len() >= usize::try_from(STAGE0_FULL_EXECUTABLE_COUNT_MAX).expect("count fits usize") {
            return Err(Stage0FullInventoryError::InvalidInput(format!(
                "full Stage0 exceeds {STAGE0_FULL_EXECUTABLE_COUNT_MAX} executables"
            )));
        }
        executables.push(full_executable_entry(root, path)?);
    }
    executables.sort_by(|left, right| left.name.cmp(&right.name));
    if executables.is_empty() {
        return Err(Stage0FullInventoryError::InvalidInput("full Stage0 produced no executable outputs".to_string()));
    }
    assert!(executables.len() <= usize::try_from(STAGE0_FULL_EXECUTABLE_COUNT_MAX).expect("count fits usize"));
    assert!(executables.iter().all(|entry| entry.absolute_path.starts_with(root)));
    Ok(executables)
}

fn full_executable_entry(
    root: &Path,
    path: PathBuf,
) -> Result<Stage0FullExecutableInventoryEntry, Stage0FullInventoryError> {
    let bytes = read_bounded_file(&path, STAGE0_FULL_EXECUTABLE_BYTES_MAX, "full Stage0 executable")?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Stage0FullInventoryError::InvalidInput("full Stage0 output name is not UTF-8".to_string()))?
        .to_string();
    let relative_path = path
        .strip_prefix(root)
        .map_err(|error| Stage0FullInventoryError::InvalidInput(format!("full Stage0 output escapes root: {error}")))?
        .to_str()
        .ok_or_else(|| Stage0FullInventoryError::InvalidInput("full Stage0 path is not UTF-8".to_string()))?
        .to_string();
    Ok(Stage0FullExecutableInventoryEntry {
        name,
        relative_path,
        absolute_path: path,
        bytes_len: u64::try_from(bytes.len()).map_err(|_| {
            Stage0FullInventoryError::InvalidInput("full Stage0 output size does not fit u64".to_string())
        })?,
        digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn validate_expected_full_stage0_executables(
    root: &Path,
    executables: &[Stage0FullExecutableInventoryEntry],
) -> Result<(), Stage0FullInventoryError> {
    if executables.len() != STAGE0_FULL_EXPECTED_EXECUTABLE_COUNT {
        return Err(Stage0FullInventoryError::InvalidInput(format!(
            "full Stage0 produced {} executables; expected {STAGE0_FULL_EXPECTED_EXECUTABLE_COUNT}",
            executables.len()
        )));
    }
    for (observed, expected) in executables.iter().zip(STAGE0_FULL_EXPECTED_EXECUTABLES) {
        let expected_path = root.join(expected.relative_path);
        if observed.relative_path != expected.relative_path
            || observed.absolute_path != expected_path
            || observed.bytes_len != expected.bytes_len
            || observed.digest_blake3 != expected.digest_blake3
        {
            return Err(Stage0FullInventoryError::InvalidInput(format!(
                "full Stage0 executable {} does not match expected path, size, and BLAKE3",
                expected.artifact_id
            )));
        }
    }
    assert_eq!(executables.len(), STAGE0_FULL_EXPECTED_EXECUTABLES.len());
    assert!(executables.iter().all(|entry| entry.absolute_path.starts_with(root)));
    Ok(())
}

fn assemble_full_report(
    mini: Stage0MiniInventoryReport,
    prepared: PreparedFullStage0,
    executables: Vec<Stage0FullExecutableInventoryEntry>,
    protected_exec_enforced: bool,
) -> Result<Stage0FullInventoryReport, Stage0FullInventoryError> {
    let executable_count = u32::try_from(executables.len())
        .map_err(|_| Stage0FullInventoryError::InvalidInput("full Stage0 count does not fit u32".to_string()))?;
    assert!(prepared.root_command_count > 0);
    assert!(prepared.nested_command_count > 0);
    Ok(Stage0FullInventoryReport {
        format: STAGE0_FULL_REPORT_FORMAT,
        mini,
        answers_digest_blake3: STAGE0_ANSWERS_BLAKE3.to_string(),
        root_script_digest_blake3: prepared.root_script_digest_blake3,
        nested_script_digest_blake3: prepared.nested_script_digest_blake3,
        root_command_count: prepared.root_command_count,
        nested_command_count: prepared.nested_command_count,
        extra_command_count: STAGE0_FULL_EXTRA_COMMAND_COUNT,
        executable_count,
        executables,
        protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: STAGE0_FULL_NON_CLAIM,
    })
}

fn write_report(root: &Path, report: &Stage0FullInventoryReport) -> Result<(), Stage0FullInventoryError> {
    let path = root.join(STAGE0_FULL_REPORT_OUTPUT);
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| Stage0FullInventoryError::InvalidInput(format!("serializing full Stage0 report: {error}")))?;
    write_create_new(&path, &bytes)?;
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn absolute_utf8_path<'a>(path: &'a Path, role: &str) -> Result<&'a str, Stage0FullInventoryError> {
    if !path.is_absolute() {
        return Err(Stage0FullInventoryError::InvalidInput(format!("{role} is not absolute: {}", path.display())));
    }
    let text = path
        .to_str()
        .ok_or_else(|| Stage0FullInventoryError::InvalidInput(format!("{role} is not UTF-8")))?;
    assert!(text.starts_with('/'));
    assert!(!text.is_empty());
    Ok(text)
}

fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), Stage0FullInventoryError> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path).map_err(|source| io_error("creating full Stage0 output", source))?;
    file.write_all(bytes).map_err(|source| io_error("writing full Stage0 output", source))?;
    file.sync_all().map_err(|source| io_error("syncing full Stage0 output", source))?;
    assert!(path.is_file());
    assert_eq!(
        fs::metadata(path).map_err(|source| io_error("reading full Stage0 output", source))?.len(),
        u64::try_from(bytes.len()).expect("full Stage0 output size fits u64")
    );
    Ok(())
}

fn read_bounded_file(path: &Path, bytes_max: u64, role: &str) -> Result<Vec<u8>, Stage0FullInventoryError> {
    let metadata = fs::metadata(path).map_err(|source| io_error(format!("reading {role} metadata"), source))?;
    if !metadata.is_file() || metadata.len() > bytes_max {
        return Err(Stage0FullInventoryError::InvalidInput(format!(
            "{role} at {} is not a regular file within {bytes_max} bytes",
            path.display()
        )));
    }
    let capacity = usize::try_from(metadata.len())
        .map_err(|_| Stage0FullInventoryError::InvalidInput(format!("{role} size does not fit usize")))?;
    let mut bytes = Vec::with_capacity(capacity);
    File::open(path)
        .map_err(|source| io_error(format!("opening {role}"), source))?
        .take(bytes_max.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| io_error(format!("reading {role}"), source))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > bytes_max {
        return Err(Stage0FullInventoryError::InvalidInput(format!("{role} exceeds {bytes_max} bytes")));
    }
    assert_eq!(bytes.len(), capacity);
    assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= bytes_max);
    Ok(bytes)
}

fn io_error(action: impl Into<String>, source: io::Error) -> Stage0FullInventoryError {
    Stage0FullInventoryError::Io {
        action: action.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROOT: &str = "/tmp/mantle-stagex-full-test";
    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const TRANSITION_DIR_ENV: &str = "MANTLE_STAGE_X_TRANSITION_DIR";
    const FULL_SCRATCH_ENV: &str = "MANTLE_STAGE_X_STAGE0_FULL_SCRATCH";
    const FULL_SOURCE_CLOSURE_BLAKE3: &str = "7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45";

    #[test]
    fn rewrites_nested_full_stage0_executables_to_absolute_paths() {
        let temp = tempfile::tempdir().unwrap();
        let script = temp.path().join("full.kaem");
        fs::write(&script, "cd AMD64\n./artifact/M2 input\n\t-f ../input\n").unwrap();

        let (rewritten, count) = rewrite_direct_commands(&script, Path::new(TEST_ROOT)).unwrap();
        let text = String::from_utf8(rewritten).unwrap();

        assert_eq!(count, 1);
        assert!(text.contains("/tmp/mantle-stagex-full-test/artifact/M2"));
        assert!(text.contains("-f ../input"));
        assert!(!text.contains("./artifact/M2"));
    }

    #[test]
    fn rejects_root_script_without_bounded_rewrite_seams() {
        let error = require_root_script_seams("./tool input\n").unwrap_err();

        assert!(error.to_string().contains(BINDIR_ASSIGNMENT));
        assert!(!error.to_string().contains("SHA-256 answers"));
    }

    #[test]
    #[ignore = "derives retained full Stage0 inventory; not protected-transition evidence"]
    fn derives_retained_full_stage0_inventory() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source_bundle = PathBuf::from(std::env::var(SOURCE_BUNDLE_ENV).unwrap());
        let transition_dir = PathBuf::from(std::env::var(TRANSITION_DIR_ENV).unwrap());
        let scratch = PathBuf::from(std::env::var(FULL_SCRATCH_ENV).unwrap());

        let report = derive_stage0_full_inventory(Stage0FullInventoryRequest {
            mini: Stage0MiniInventoryRequest {
                source_bundle_path: &source_bundle,
                expected_source_bundle_blake3: FULL_SOURCE_CLOSURE_BLAKE3,
                reproduced_hex0_path: &transition_dir.join("hex0-reproduced"),
                kaem_path: &transition_dir.join("kaem-0"),
                scratch_dir: &scratch,
            },
            answers_path: &repo.join("bootstrap/stage0-amd64.answers"),
        })
        .unwrap();

        assert!(report.executable_count > 1);
        assert!(!report.protected_exec_enforced);
        assert!(report.fallback_events.is_empty());
        assert!(scratch.join(STAGE0_FULL_REPORT_OUTPUT).is_file());
    }
}
