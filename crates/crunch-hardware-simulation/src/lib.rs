use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crunch_hardware_simulation_core::HardwareProfile;
use crunch_hardware_simulation_core::SourceObservation;
use crunch_hardware_simulation_core::SourcePackage;
use crunch_hardware_simulation_core::ToolCohort;
use crunch_hardware_simulation_core::validate_profile;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub const DEFAULT_MAX_PROFILE_BYTES: u64 = 1_048_576;
pub const DEFAULT_MAX_FIXTURE_FILES: u32 = 256;
pub const DEFAULT_MAX_FIXTURE_FILE_BYTES: u64 = 1_048_576;
pub const DEFAULT_MAX_FIXTURE_TOTAL_BYTES: u64 = 16_777_216;
pub const DEFAULT_MAX_LOG_BYTES: u64 = 65_536;
pub const DEFAULT_ACTION_TIMEOUT_MS: u64 = 120_000;

const PROCESS_POLL_INTERVAL_MS: u64 = 10;
const MAX_READ_ONLY_BINDINGS: u32 = 64;
const GIT_AUTHOR_NAME: &str = "Mantle Fixture";
const GIT_AUTHOR_EMAIL: &str = "mantle-fixture@example.invalid";
const GIT_TIMESTAMP: &str = "2000-01-01T00:00:00Z";
const DIRECTORY_HASH_DOMAIN: &[u8] = b"mantle.hardware.fixture-tree.v1";
const CLOSURE_HASH_DOMAIN: &[u8] = b"mantle.hardware.tool-closure.v1";
const DOMAIN_SEPARATOR: u8 = 0;

#[derive(Debug)]
pub enum ShellError {
    Io {
        operation: &'static str,
        path: PathBuf,
        message: String,
    },
    Invalid(String),
    Process {
        program: PathBuf,
        exit_code: Option<i32>,
        stderr: String,
    },
    Timeout {
        program: PathBuf,
        timeout_ms: u64,
    },
    Json(String),
    Core(Vec<String>),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                message,
            } => write!(formatter, "{operation} `{}`: {message}", path.display()),
            Self::Invalid(message) => formatter.write_str(message),
            Self::Process {
                program,
                exit_code,
                stderr,
            } => write!(formatter, "process `{}` exited {exit_code:?}: {stderr}", program.display()),
            Self::Timeout { program, timeout_ms } => {
                write!(formatter, "process `{}` exceeded {timeout_ms} ms", program.display())
            }
            Self::Json(message) => write!(formatter, "hardware simulation JSON: {message}"),
            Self::Core(diagnostics) => {
                write!(formatter, "hardware simulation core rejected facts: {}", diagnostics.join(","))
            }
        }
    }
}

impl std::error::Error for ShellError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryMeasurement {
    pub digest_blake3: String,
    pub sentinel_blake3: String,
    pub file_count: u32,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedGitFixture {
    pub repository_path: PathBuf,
    pub revision: String,
    pub measurement: DirectoryMeasurement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCohortObservation {
    pub closure_paths: Vec<PathBuf>,
    pub closure_paths_blake3: String,
    pub member_binary_blake3: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOnlyBinding {
    pub source: PathBuf,
    pub guest: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictActionRequest {
    pub bwrap: PathBuf,
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub read_only_paths: Vec<PathBuf>,
    pub read_only_bindings: Vec<ReadOnlyBinding>,
    pub writable_root: PathBuf,
    pub timeout_ms: u64,
    pub max_log_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictActionResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub elapsed_diagnostic_ns: u64,
}

pub fn read_json_bounded<T: DeserializeOwned>(path: &Path, maximum_bytes: u64) -> Result<T, ShellError> {
    let bytes = read_regular_file_bounded(path, maximum_bytes)?;
    let value = serde_json::from_slice(&bytes).map_err(|error| ShellError::Json(error.to_string()))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= maximum_bytes);
    Ok(value)
}

pub fn read_and_validate_profile(
    path: &Path,
) -> Result<crunch_hardware_simulation_core::ProfileValidation, ShellError> {
    let profile: HardwareProfile = read_json_bounded(path, DEFAULT_MAX_PROFILE_BYTES)?;
    let validation = validate_profile(profile).map_err(ShellError::Core)?;
    debug_assert!(!validation.profile_ref.is_empty());
    debug_assert!(!validation.selected_source_ids.is_empty());
    Ok(validation)
}

pub fn write_json_new<T: Serialize>(path: &Path, value: &T) -> Result<(), ShellError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| ShellError::Json(error.to_string()))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error("creating JSON output", path, error))?;
    output.write_all(&bytes).map_err(|error| io_error("writing JSON output", path, error))?;
    output.sync_all().map_err(|error| io_error("syncing JSON output", path, error))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(path.exists());
    Ok(())
}

pub fn materialize_git_fixture(
    git_executable: &Path,
    source_root: &Path,
    repository_path: &Path,
    sentinel_relative_path: &Path,
) -> Result<MaterializedGitFixture, ShellError> {
    require_absolute_executable(git_executable)?;
    require_real_directory(source_root)?;
    if repository_path.exists() {
        return Err(ShellError::Invalid(String::from("git-fixture-destination-exists")));
    }
    copy_fixture_tree(source_root, repository_path)?;
    run_git(git_executable, repository_path, &["init", "--quiet"])?;
    run_git(git_executable, repository_path, &["add", "--all"])?;
    run_git(git_executable, repository_path, &["commit", "--quiet", "--message", "pinned fixture"])?;
    let revision = git_stdout(git_executable, repository_path, &["rev-parse", "HEAD"])?;
    let measurement = measure_directory(
        repository_path,
        sentinel_relative_path,
        DEFAULT_MAX_FIXTURE_FILES,
        DEFAULT_MAX_FIXTURE_FILE_BYTES,
        DEFAULT_MAX_FIXTURE_TOTAL_BYTES,
    )?;
    debug_assert_eq!(revision.len(), crunch_hardware_simulation_core::GIT_REVISION_HEX_LENGTH);
    debug_assert!(repository_path.join(".git").is_dir());
    Ok(MaterializedGitFixture {
        repository_path: repository_path.to_path_buf(),
        revision,
        measurement,
    })
}

pub fn source_observation(
    package: &SourcePackage,
    fixture: &MaterializedGitFixture,
    acquired: bool,
) -> SourceObservation {
    let observation = SourceObservation {
        id: package.id.clone(),
        locator: package.locator.clone(),
        revision: fixture.revision.clone(),
        recursive_blake3: fixture.measurement.digest_blake3.clone(),
        sentinel_blake3: fixture.measurement.sentinel_blake3.clone(),
        declared_edges: package.dependencies.clone(),
        acquired,
    };
    debug_assert!(!observation.id.is_empty());
    debug_assert_eq!(observation.revision.len(), crunch_hardware_simulation_core::GIT_REVISION_HEX_LENGTH);
    observation
}

pub fn measure_directory(
    root: &Path,
    sentinel_relative_path: &Path,
    maximum_files: u32,
    maximum_file_bytes: u64,
    maximum_total_bytes: u64,
) -> Result<DirectoryMeasurement, ShellError> {
    require_real_directory(root)?;
    let mut paths = Vec::new();
    collect_regular_files(root, root, maximum_files, &mut paths)?;
    paths.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(DIRECTORY_HASH_DOMAIN);
    hasher.update(&[DOMAIN_SEPARATOR]);
    let mut total_bytes = 0_u64;
    for relative in &paths {
        let absolute = root.join(relative);
        let bytes = read_regular_file_bounded(&absolute, maximum_file_bytes)?;
        total_bytes = total_bytes
            .checked_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX))
            .ok_or_else(|| ShellError::Invalid(String::from("fixture-total-byte-overflow")))?;
        if total_bytes > maximum_total_bytes {
            return Err(ShellError::Invalid(String::from("fixture-total-byte-bound-exceeded")));
        }
        update_tree_hash(&mut hasher, relative, &bytes);
    }
    let sentinel = read_regular_file_bounded(&root.join(sentinel_relative_path), maximum_file_bytes)?;
    let file_count =
        u32::try_from(paths.len()).map_err(|_| ShellError::Invalid(String::from("fixture-file-count-overflow")))?;
    let measurement = DirectoryMeasurement {
        digest_blake3: hasher.finalize().to_hex().to_string(),
        sentinel_blake3: blake3::hash(&sentinel).to_hex().to_string(),
        file_count,
        total_bytes,
    };
    debug_assert!(measurement.file_count > 0);
    debug_assert!(measurement.total_bytes > 0);
    Ok(measurement)
}

pub fn observe_tool_cohort(
    cohort: &ToolCohort,
    closure_paths: Vec<PathBuf>,
) -> Result<ToolCohortObservation, ShellError> {
    if closure_paths.is_empty() {
        return Err(ShellError::Invalid(String::from("tool-closure-empty")));
    }
    let mut member_binary_blake3 = BTreeMap::new();
    for member in &cohort.members {
        let executable = Path::new(&member.store_path).join(&member.executable);
        let digest = hash_regular_file(&executable, DEFAULT_MAX_FIXTURE_TOTAL_BYTES)?;
        member_binary_blake3.insert(format!("{:?}", member.role), digest);
    }
    let closure_paths = canonical_paths(closure_paths)?;
    let closure_paths_blake3 = hash_path_list(CLOSURE_HASH_DOMAIN, &closure_paths);
    debug_assert_eq!(member_binary_blake3.len(), cohort.members.len());
    debug_assert!(!closure_paths_blake3.is_empty());
    Ok(ToolCohortObservation {
        closure_paths,
        closure_paths_blake3,
        member_binary_blake3,
    })
}

pub fn run_strict_action(request: StrictActionRequest) -> Result<StrictActionResult, ShellError> {
    validate_action_request(&request)?;
    fs::create_dir_all(&request.writable_root)
        .map_err(|error| io_error("creating strict action writable root", &request.writable_root, error))?;
    let stdout_path = request.writable_root.join("stdout.log");
    let stderr_path = request.writable_root.join("stderr.log");
    let stdout_file = create_new_file(&stdout_path)?;
    let stderr_file = create_new_file(&stderr_path)?;
    let mut command = strict_bwrap_command(&request)?;
    command.stdout(Stdio::from(stdout_file));
    command.stderr(Stdio::from(stderr_file));
    let started = Instant::now();
    let mut child = command.spawn().map_err(|error| io_error("spawning strict bwrap action", &request.bwrap, error))?;
    let status = wait_bounded(&mut child, &request.executable, request.timeout_ms)?;
    let elapsed_diagnostic_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let stdout = read_regular_file_allow_empty_bounded(&stdout_path, request.max_log_bytes)?;
    let stderr = read_regular_file_allow_empty_bounded(&stderr_path, request.max_log_bytes)?;
    let exit_code = status.code().unwrap_or(i32::MIN);
    debug_assert!(elapsed_diagnostic_ns > 0);
    debug_assert!(u64::try_from(stdout.len()).unwrap_or(u64::MAX) <= request.max_log_bytes);
    Ok(StrictActionResult {
        exit_code,
        stdout,
        stderr,
        elapsed_diagnostic_ns,
    })
}

fn validate_action_request(request: &StrictActionRequest) -> Result<(), ShellError> {
    require_absolute_executable(&request.bwrap)?;
    require_absolute_executable(&request.executable)?;
    if request.timeout_ms == 0 || request.max_log_bytes == 0 {
        return Err(ShellError::Invalid(String::from("strict-action-limit-zero")));
    }
    if request.read_only_paths.is_empty() {
        return Err(ShellError::Invalid(String::from("strict-action-read-closure-empty")));
    }
    validate_read_only_bindings(&request.read_only_bindings)?;
    if !request.writable_root.is_absolute() {
        return Err(ShellError::Invalid(String::from("strict-action-writable-root-not-absolute")));
    }
    debug_assert!(request.timeout_ms > 0);
    debug_assert!(request.max_log_bytes > 0);
    Ok(())
}

fn validate_read_only_bindings(bindings: &[ReadOnlyBinding]) -> Result<(), ShellError> {
    let binding_count = u32::try_from(bindings.len()).unwrap_or(u32::MAX);
    if binding_count > MAX_READ_ONLY_BINDINGS {
        return Err(ShellError::Invalid(String::from("strict-action-binding-count-exceeded")));
    }
    for binding in bindings {
        if !binding.source.is_absolute() || !binding.source.is_file() {
            return Err(ShellError::Invalid(String::from("strict-action-binding-source-invalid")));
        }
        if !binding.guest.is_absolute() || !binding.guest.starts_with("/bin/") {
            return Err(ShellError::Invalid(String::from("strict-action-binding-guest-invalid")));
        }
        if binding.guest.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
            return Err(ShellError::Invalid(String::from("strict-action-binding-guest-invalid")));
        }
    }
    debug_assert!(binding_count <= MAX_READ_ONLY_BINDINGS);
    debug_assert!(bindings.iter().all(|binding| binding.source.is_absolute()));
    Ok(())
}

fn strict_bwrap_command(request: &StrictActionRequest) -> Result<Command, ShellError> {
    let mut command = Command::new(&request.bwrap);
    command.env_clear();
    command.args([
        "--unshare-all",
        "--die-with-parent",
        "--new-session",
        "--clearenv",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--dir",
        "/nix",
        "--dir",
        "/nix/store",
        "--dir",
        "/bin",
        "--dir",
        "/build",
        "--chdir",
        "/build",
    ]);
    for path in canonical_paths(request.read_only_paths.clone())? {
        command.arg("--ro-bind").arg(&path).arg(&path);
    }
    for binding in &request.read_only_bindings {
        let source = fs::canonicalize(&binding.source)
            .map_err(|error| io_error("canonicalizing strict action binding", &binding.source, error))?;
        command.arg("--ro-bind").arg(source).arg(&binding.guest);
    }
    command.arg("--bind").arg(&request.writable_root).arg("/build");
    for (key, value) in &request.env {
        command.arg("--setenv").arg(key).arg(value);
    }
    command.arg("--").arg(&request.executable).args(&request.args);
    debug_assert!(command.get_envs().all(|(_, value)| value.is_none()));
    debug_assert_eq!(command.get_program(), request.bwrap.as_os_str());
    Ok(command)
}

fn wait_bounded(child: &mut std::process::Child, program: &Path, timeout_ms: u64) -> Result<ExitStatus, ShellError> {
    let started = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    let interval = Duration::from_millis(PROCESS_POLL_INTERVAL_MS);
    loop {
        if let Some(status) = child.try_wait().map_err(|error| io_error("polling strict action", program, error))? {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ShellError::Timeout {
                program: program.to_path_buf(),
                timeout_ms,
            });
        }
        thread::sleep(interval);
    }
}

fn run_git(git: &Path, repository: &Path, args: &[&str]) -> Result<(), ShellError> {
    let output = git_command(git, repository, args)
        .output()
        .map_err(|error| io_error("running fixture git", git, error))?;
    if !output.status.success() {
        return Err(ShellError::Process {
            program: git.to_path_buf(),
            exit_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    debug_assert!(output.status.success());
    debug_assert!(output.stderr.is_empty());
    Ok(())
}

fn git_stdout(git: &Path, repository: &Path, args: &[&str]) -> Result<String, ShellError> {
    let output = git_command(git, repository, args)
        .output()
        .map_err(|error| io_error("reading fixture git", git, error))?;
    if !output.status.success() {
        return Err(ShellError::Process {
            program: git.to_path_buf(),
            exit_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    let value = String::from_utf8(output.stdout)
        .map_err(|error| ShellError::Invalid(format!("git-output-not-utf8:{error}")))?
        .trim()
        .to_string();
    if value.is_empty() {
        return Err(ShellError::Invalid(String::from("git-output-empty")));
    }
    debug_assert!(!value.contains('\n'));
    debug_assert!(!value.is_empty());
    Ok(value)
}

fn git_command(git: &Path, repository: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(git);
    command
        .env_clear()
        .env("HOME", repository)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", GIT_AUTHOR_NAME)
        .env("GIT_AUTHOR_EMAIL", GIT_AUTHOR_EMAIL)
        .env("GIT_COMMITTER_NAME", GIT_AUTHOR_NAME)
        .env("GIT_COMMITTER_EMAIL", GIT_AUTHOR_EMAIL)
        .env("GIT_AUTHOR_DATE", GIT_TIMESTAMP)
        .env("GIT_COMMITTER_DATE", GIT_TIMESTAMP)
        .current_dir(repository)
        .args(args);
    command
}

fn copy_fixture_tree(source: &Path, destination: &Path) -> Result<(), ShellError> {
    fs::create_dir(destination).map_err(|error| io_error("creating fixture repository", destination, error))?;
    let mut entries = fs::read_dir(source)
        .map_err(|error| io_error("reading fixture source", source, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error("reading fixture source entry", source, error))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| io_error("inspecting fixture source", &source_path, error))?;
        if metadata.file_type().is_dir() {
            copy_fixture_tree(&source_path, &destination_path)?;
        } else if metadata.file_type().is_file() {
            fs::copy(&source_path, &destination_path)
                .map_err(|error| io_error("copying fixture source", &source_path, error))?;
        } else {
            return Err(ShellError::Invalid(String::from("fixture-source-kind-unsupported")));
        }
    }
    debug_assert!(destination.is_dir());
    debug_assert!(!destination.is_symlink());
    Ok(())
}

fn collect_regular_files(
    root: &Path,
    current: &Path,
    maximum_files: u32,
    paths: &mut Vec<PathBuf>,
) -> Result<(), ShellError> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| io_error("reading fixture directory", current, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error("reading fixture entry", current, error))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        if entry.file_name() == ".git" {
            continue;
        }
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| io_error("inspecting fixture entry", &path, error))?;
        if metadata.file_type().is_dir() {
            collect_regular_files(root, &path, maximum_files, paths)?;
            continue;
        }
        if !metadata.file_type().is_file() {
            return Err(ShellError::Invalid(String::from("fixture-entry-kind-unsupported")));
        }
        paths.push(
            path.strip_prefix(root)
                .map_err(|_| ShellError::Invalid(String::from("fixture-path-escape")))?
                .to_path_buf(),
        );
        if u32::try_from(paths.len()).unwrap_or(u32::MAX) > maximum_files {
            return Err(ShellError::Invalid(String::from("fixture-file-count-bound-exceeded")));
        }
    }
    debug_assert!(u32::try_from(paths.len()).unwrap_or(u32::MAX) <= maximum_files);
    debug_assert!(paths.iter().all(|path| path.is_relative()));
    Ok(())
}

fn update_tree_hash(hasher: &mut blake3::Hasher, relative: &Path, bytes: &[u8]) {
    let path = relative.to_string_lossy();
    hasher.update(path.as_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
}

fn hash_regular_file(path: &Path, maximum_bytes: u64) -> Result<String, ShellError> {
    let bytes = read_regular_file_bounded(path, maximum_bytes)?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    debug_assert!(!bytes.is_empty());
    debug_assert_eq!(digest.len(), crunch_hardware_simulation_core::BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn hash_path_list(domain: &[u8], paths: &[PathBuf]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    for path in paths {
        hasher.update(path.as_os_str().as_encoded_bytes());
        hasher.update(&[DOMAIN_SEPARATOR]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(!paths.is_empty());
    debug_assert_eq!(digest.len(), crunch_hardware_simulation_core::BLAKE3_HEX_LENGTH);
    digest
}

fn canonical_paths(mut paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, ShellError> {
    if paths.iter().any(|path| !path.is_absolute()) {
        return Err(ShellError::Invalid(String::from("declared-path-not-absolute")));
    }
    paths.sort();
    paths.dedup();
    debug_assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(paths.iter().all(|path| path.is_absolute()));
    Ok(paths)
}

fn require_absolute_executable(path: &Path) -> Result<(), ShellError> {
    if !path.is_absolute() {
        return Err(ShellError::Invalid(String::from("executable-not-absolute")));
    }
    let metadata = fs::metadata(path).map_err(|error| io_error("inspecting executable", path, error))?;
    if !metadata.is_file() {
        return Err(ShellError::Invalid(String::from("executable-not-regular-file")));
    }
    debug_assert!(path.is_absolute());
    debug_assert!(metadata.is_file());
    Ok(())
}

fn require_real_directory(path: &Path) -> Result<(), ShellError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| io_error("inspecting directory", path, error))?;
    if !metadata.file_type().is_dir() {
        return Err(ShellError::Invalid(String::from("directory-not-real")));
    }
    debug_assert!(metadata.is_dir());
    debug_assert!(!metadata.file_type().is_symlink());
    Ok(())
}

fn create_new_file(path: &Path) -> Result<File, ShellError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error("creating action log", path, error))
}

fn read_regular_file_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, ShellError> {
    let bytes = read_regular_file_allow_empty_bounded(path, maximum_bytes)?;
    if bytes.is_empty() {
        return Err(ShellError::Invalid(format!("regular-file-empty:{}", path.display())));
    }
    Ok(bytes)
}

fn read_regular_file_allow_empty_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, ShellError> {
    let mut input = File::open(path).map_err(|error| io_error("opening bounded file", path, error))?;
    let metadata = input.metadata().map_err(|error| io_error("inspecting bounded file", path, error))?;
    if !metadata.is_file() || metadata.len() > maximum_bytes {
        return Err(ShellError::Invalid(format!("regular-file-bound-invalid:{}", path.display())));
    }
    let capacity =
        usize::try_from(metadata.len()).map_err(|_| ShellError::Invalid(String::from("file-size-overflow")))?;
    let mut bytes = Vec::with_capacity(capacity);
    input.read_to_end(&mut bytes).map_err(|error| io_error("reading bounded file", path, error))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) != metadata.len() {
        return Err(ShellError::Invalid(format!("regular-file-size-drift:{}", path.display())));
    }
    debug_assert!(u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= maximum_bytes);
    debug_assert_eq!(bytes.len(), capacity);
    Ok(bytes)
}

fn io_error(operation: &'static str, path: &Path, error: std::io::Error) -> ShellError {
    ShellError::Io {
        operation,
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
