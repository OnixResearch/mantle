use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

const BLAKE3_HEX_LEN: u32 = 64;
const SCHEMA_VERSION_V1: &str = "host-tool-free-stage0-v1";
const DIGEST_ALGORITHM_BLAKE3: &str = "blake3";
const ROLE_STAGE0_CRUNCH: &str = "stage0-crunch";
const ROLE_SANDBOX_ENTRY: &str = "sandbox-entry";
const ROLE_SANDBOX_SHELL: &str = "sandbox-shell";
const ROLE_BOOTSTRAP_TOOLCHAIN_TOOL: &str = "bootstrap-toolchain-tool";
const ROLE_BOOTSTRAP_BUILD_TOOL: &str = "bootstrap-build-tool";
const PROVENANCE_OPERATOR_SOURCE_BUILD: &str = "operator-supplied-source-build";
const PROVENANCE_OPERATOR_BOOTSTRAP_SEED: &str = "operator-supplied-bootstrap-seed";
const PROVENANCE_TEST_FIXTURE: &str = "test-fixture";
const PROVENANCE_HOST_PATH_DISCOVERY: &str = "host-path-discovery";
const PROVENANCE_NIX_STORE_DISCOVERY: &str = "nix-store-discovery";
const PHASE_PROTECTED: &str = "protected";
const PATH_SEPARATOR: char = '/';
const HASH_BUFFER_BYTES: usize = 64 * 1024;
const MAX_SEED_EXECUTABLES: u32 = 4096;
const MAX_SEED_WALK_DEPTH: u32 = 16;
const ENV_STAGE0_SEED_SANDBOX_ENTRY: &str = "CRUNCH_STAGE0_SEED_SANDBOX_ENTRY";
const ENV_STAGE0_SEED_SANDBOX_SHELL: &str = "CRUNCH_STAGE0_SEED_SANDBOX_SHELL";
const ENV_STAGE0_SEED_TOOLCHAIN_ROOT: &str = "CRUNCH_STAGE0_SEED_TOOLCHAIN_ROOT";
const ENV_STAGE0_SEED_BUILD_TOOLS: &str = "CRUNCH_STAGE0_SEED_BUILD_TOOLS";
const GENERATED_INVENTORY_HEADER: &str = "# Generated host-tool-free stage0 inventory.\n# Do not edit by hand; regenerate from explicit CRUNCH_STAGE0_SEED_* inputs.\n\n";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DigestSpec {
    pub algorithm: String,
    pub hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interoperability_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutableSeedEntry {
    pub schema_version: String,
    pub id: String,
    pub role: String,
    pub phase: String,
    pub executable_path: PathBuf,
    pub digest: DigestSpec,
    pub provenance_category: String,
    pub provenance: String,
    pub allowed_reason: String,
    pub owner: String,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceSeedEntry {
    pub schema_version: String,
    pub id: String,
    pub role: String,
    pub phase: String,
    pub urls: Vec<String>,
    pub extraction_rules: Vec<String>,
    pub digest: DigestSpec,
    pub provenance_category: String,
    pub provenance: String,
    pub allowed_reason: String,
    pub owner: String,
    pub required: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Stage0Inventory {
    pub executable_entries: Vec<ExecutableSeedEntry>,
    pub source_entries: Vec<SourceSeedEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecRequest {
    pub path: PathBuf,
    pub digest_hex: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecDecision {
    pub allowed: bool,
    pub entry_id: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedSourceFetchPlan {
    pub entry_id: String,
    pub url: String,
    pub digest: DigestSpec,
    pub extraction_rules: Vec<String>,
    pub allowed_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedExecPolicy {
    executables_by_path: BTreeMap<PathBuf, ExecutableSeedEntry>,
    sources_by_url: BTreeMap<String, SourceSeedEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectedExecError {
    DuplicateExecutablePath {
        path: PathBuf,
    },
    DuplicateSourceUrl {
        url: String,
    },
    EmptyField {
        entry_id: String,
        field: &'static str,
    },
    UnsupportedSchemaVersion {
        entry_id: String,
        actual: String,
    },
    RelativeExecutablePath {
        entry_id: String,
        path: PathBuf,
    },
    DisallowedRole {
        entry_id: String,
        role: String,
    },
    DisallowedNixExecutable {
        entry_id: String,
        path: PathBuf,
    },
    DisallowedProvenance {
        entry_id: String,
        provenance_category: String,
    },
    MissingInteroperabilityReason {
        entry_id: String,
        algorithm: String,
    },
    InvalidBlake3Digest {
        entry_id: String,
        digest_hex: String,
    },
    MissingRequiredSeedRole {
        role: &'static str,
    },
    UndeclaredExecutable {
        path: PathBuf,
    },
    DigestMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    UndeclaredSourceUrl {
        url: String,
    },
    SourceDigestMismatch {
        url: String,
        expected: String,
        actual: String,
    },
}

impl fmt::Display for ProtectedExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateExecutablePath { path } => write!(f, "duplicate executable path: {}", path.display()),
            Self::DuplicateSourceUrl { url } => write!(f, "duplicate source url: {url}"),
            Self::EmptyField { entry_id, field } => write!(f, "entry {entry_id} has empty field {field}"),
            Self::UnsupportedSchemaVersion { entry_id, actual } => {
                write!(f, "entry {entry_id} has unsupported schema version {actual}")
            }
            Self::RelativeExecutablePath { entry_id, path } => {
                write!(f, "entry {entry_id} executable path is not absolute: {}", path.display())
            }
            Self::DisallowedRole { entry_id, role } => write!(f, "entry {entry_id} uses disallowed role {role}"),
            Self::DisallowedNixExecutable { entry_id, path } => {
                write!(f, "entry {entry_id} declares forbidden nix executable {}", path.display())
            }
            Self::DisallowedProvenance {
                entry_id,
                provenance_category,
            } => write!(f, "entry {entry_id} has disallowed provenance {provenance_category}"),
            Self::MissingInteroperabilityReason { entry_id, algorithm } => {
                write!(f, "entry {entry_id} uses non-blake3 digest {algorithm} without interoperability reason")
            }
            Self::InvalidBlake3Digest { entry_id, digest_hex } => {
                write!(f, "entry {entry_id} has invalid blake3 digest {digest_hex}")
            }
            Self::MissingRequiredSeedRole { role } => write!(f, "missing required seed role {role}"),
            Self::UndeclaredExecutable { path } => write!(f, "undeclared executable path: {}", path.display()),
            Self::DigestMismatch { path, expected, actual } => {
                write!(f, "digest mismatch for {}: expected {expected}, got {actual}", path.display())
            }
            Self::UndeclaredSourceUrl { url } => write!(f, "undeclared protected source url: {url}"),
            Self::SourceDigestMismatch { url, expected, actual } => {
                write!(f, "source digest mismatch for {url}: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for ProtectedExecError {}

impl ProtectedExecPolicy {
    pub fn from_inventory(inventory: Stage0Inventory) -> Result<Self, ProtectedExecError> {
        validate_required_roles(&inventory.executable_entries)?;
        let mut executables_by_path = BTreeMap::new();
        for entry in inventory.executable_entries {
            validate_executable_entry(&entry)?;
            let previous = executables_by_path.insert(entry.executable_path.clone(), entry);
            if previous.is_some() {
                let path = executables_by_path.keys().next_back().cloned().unwrap_or_default();
                return Err(ProtectedExecError::DuplicateExecutablePath { path });
            }
        }

        let mut sources_by_url = BTreeMap::new();
        for entry in inventory.source_entries {
            validate_source_entry(&entry)?;
            for url in &entry.urls {
                let previous = sources_by_url.insert(url.clone(), entry.clone());
                if previous.is_some() {
                    return Err(ProtectedExecError::DuplicateSourceUrl { url: url.clone() });
                }
            }
        }

        assert!(!executables_by_path.is_empty(), "policy must contain required executables");
        Ok(Self {
            executables_by_path,
            sources_by_url,
        })
    }

    pub fn decide_exec(&self, request: &ExecRequest) -> Result<ExecDecision, ProtectedExecError> {
        assert!(request.path.is_absolute(), "exec request path must be absolute");
        assert!(!request.digest_hex.is_empty(), "exec request digest must be present");
        let entry =
            self.executables_by_path
                .get(&request.path)
                .ok_or_else(|| ProtectedExecError::UndeclaredExecutable {
                    path: request.path.clone(),
                })?;
        if entry.digest.hex != request.digest_hex {
            return Err(ProtectedExecError::DigestMismatch {
                path: request.path.clone(),
                expected: entry.digest.hex.clone(),
                actual: request.digest_hex.clone(),
            });
        }
        Ok(ExecDecision {
            allowed: true,
            entry_id: Some(entry.id.clone()),
            reason: entry.allowed_reason.clone(),
        })
    }

    pub fn source_fetch_plan(&self, url: &str) -> Result<ProtectedSourceFetchPlan, ProtectedExecError> {
        assert!(!url.is_empty(), "source url check must not be empty");
        let entry = self
            .sources_by_url
            .get(url)
            .ok_or_else(|| ProtectedExecError::UndeclaredSourceUrl { url: url.to_string() })?;
        Ok(ProtectedSourceFetchPlan {
            entry_id: entry.id.clone(),
            url: url.to_string(),
            digest: entry.digest.clone(),
            extraction_rules: entry.extraction_rules.clone(),
            allowed_reason: entry.allowed_reason.clone(),
        })
    }

    pub fn verify_source_digest(&self, url: &str, actual_digest_hex: &str) -> Result<(), ProtectedExecError> {
        assert!(!actual_digest_hex.is_empty(), "source digest check must not be empty");
        let plan = self.source_fetch_plan(url)?;
        if plan.digest.hex == actual_digest_hex {
            return Ok(());
        }
        Err(ProtectedExecError::SourceDigestMismatch {
            url: url.to_string(),
            expected: plan.digest.hex,
            actual: actual_digest_hex.to_string(),
        })
    }

    pub fn ensure_source_url_allowed(&self, url: &str) -> Result<(), ProtectedExecError> {
        self.source_fetch_plan(url).map(|_| ())
    }
}

fn validate_required_roles(entries: &[ExecutableSeedEntry]) -> Result<(), ProtectedExecError> {
    assert!(!entries.is_empty(), "inventory must include executable entries");
    let roles: BTreeSet<&str> =
        entries.iter().filter(|entry| entry.required).map(|entry| entry.role.as_str()).collect();
    for role in [ROLE_SANDBOX_ENTRY, ROLE_SANDBOX_SHELL] {
        if !roles.contains(role) {
            return Err(ProtectedExecError::MissingRequiredSeedRole { role });
        }
    }
    Ok(())
}

fn validate_executable_entry(entry: &ExecutableSeedEntry) -> Result<(), ProtectedExecError> {
    validate_common_fields(
        &entry.schema_version,
        &entry.id,
        &entry.role,
        &entry.phase,
        &entry.digest,
        &entry.provenance_category,
        &entry.provenance,
        &entry.allowed_reason,
        &entry.owner,
    )?;
    if !entry.executable_path.is_absolute() {
        return Err(ProtectedExecError::RelativeExecutablePath {
            entry_id: entry.id.clone(),
            path: entry.executable_path.clone(),
        });
    }
    if !is_allowed_executable_role(&entry.role) {
        return Err(ProtectedExecError::DisallowedRole {
            entry_id: entry.id.clone(),
            role: entry.role.clone(),
        });
    }
    if is_forbidden_nix_executable(&entry.executable_path) {
        return Err(ProtectedExecError::DisallowedNixExecutable {
            entry_id: entry.id.clone(),
            path: entry.executable_path.clone(),
        });
    }
    Ok(())
}

fn validate_source_entry(entry: &SourceSeedEntry) -> Result<(), ProtectedExecError> {
    validate_common_fields(
        &entry.schema_version,
        &entry.id,
        &entry.role,
        &entry.phase,
        &entry.digest,
        &entry.provenance_category,
        &entry.provenance,
        &entry.allowed_reason,
        &entry.owner,
    )?;
    validate_non_empty(entry.id.as_str(), "urls", entry.urls.first().map(String::as_str).unwrap_or_default())?;
    for url in &entry.urls {
        validate_non_empty(entry.id.as_str(), "urls[]", url)?;
    }
    validate_non_empty(
        entry.id.as_str(),
        "extraction_rules",
        entry.extraction_rules.first().map(String::as_str).unwrap_or_default(),
    )?;
    for rule in &entry.extraction_rules {
        validate_non_empty(entry.id.as_str(), "extraction_rules[]", rule)?;
    }
    Ok(())
}

fn validate_common_fields(
    schema_version: &str,
    entry_id: &str,
    role: &str,
    phase: &str,
    digest: &DigestSpec,
    provenance_category: &str,
    provenance: &str,
    allowed_reason: &str,
    owner: &str,
) -> Result<(), ProtectedExecError> {
    validate_non_empty(entry_id, "id", entry_id)?;
    validate_non_empty(entry_id, "role", role)?;
    validate_non_empty(entry_id, "phase", phase)?;
    validate_non_empty(entry_id, "digest.algorithm", digest.algorithm.as_str())?;
    validate_non_empty(entry_id, "digest.hex", digest.hex.as_str())?;
    validate_non_empty(entry_id, "provenance_category", provenance_category)?;
    validate_non_empty(entry_id, "provenance", provenance)?;
    validate_non_empty(entry_id, "allowed_reason", allowed_reason)?;
    validate_non_empty(entry_id, "owner", owner)?;
    if schema_version != SCHEMA_VERSION_V1 {
        return Err(ProtectedExecError::UnsupportedSchemaVersion {
            entry_id: entry_id.to_string(),
            actual: schema_version.to_string(),
        });
    }
    if phase != PHASE_PROTECTED {
        return Err(ProtectedExecError::DisallowedRole {
            entry_id: entry_id.to_string(),
            role: phase.to_string(),
        });
    }
    validate_digest(entry_id, digest)?;
    validate_provenance(entry_id, provenance_category)?;
    Ok(())
}

fn validate_non_empty(entry_id: &str, field: &'static str, value: &str) -> Result<(), ProtectedExecError> {
    if value.is_empty() {
        return Err(ProtectedExecError::EmptyField {
            entry_id: entry_id.to_string(),
            field,
        });
    }
    Ok(())
}

fn validate_digest(entry_id: &str, digest: &DigestSpec) -> Result<(), ProtectedExecError> {
    if digest.algorithm == DIGEST_ALGORITHM_BLAKE3 {
        if is_valid_hex_digest(&digest.hex, BLAKE3_HEX_LEN) {
            return Ok(());
        }
        return Err(ProtectedExecError::InvalidBlake3Digest {
            entry_id: entry_id.to_string(),
            digest_hex: digest.hex.clone(),
        });
    }
    match &digest.interoperability_reason {
        Some(reason) if !reason.is_empty() => Ok(()),
        _ => Err(ProtectedExecError::MissingInteroperabilityReason {
            entry_id: entry_id.to_string(),
            algorithm: digest.algorithm.clone(),
        }),
    }
}

fn validate_provenance(entry_id: &str, provenance_category: &str) -> Result<(), ProtectedExecError> {
    if provenance_category == PROVENANCE_HOST_PATH_DISCOVERY || provenance_category == PROVENANCE_NIX_STORE_DISCOVERY {
        return Err(ProtectedExecError::DisallowedProvenance {
            entry_id: entry_id.to_string(),
            provenance_category: provenance_category.to_string(),
        });
    }
    if matches!(
        provenance_category,
        PROVENANCE_OPERATOR_SOURCE_BUILD | PROVENANCE_OPERATOR_BOOTSTRAP_SEED | PROVENANCE_TEST_FIXTURE
    ) {
        return Ok(());
    }
    Err(ProtectedExecError::DisallowedProvenance {
        entry_id: entry_id.to_string(),
        provenance_category: provenance_category.to_string(),
    })
}

fn is_allowed_executable_role(role: &str) -> bool {
    matches!(
        role,
        ROLE_STAGE0_CRUNCH
            | ROLE_SANDBOX_ENTRY
            | ROLE_SANDBOX_SHELL
            | ROLE_BOOTSTRAP_TOOLCHAIN_TOOL
            | ROLE_BOOTSTRAP_BUILD_TOOL
    )
}

fn is_forbidden_nix_executable(path: &Path) -> bool {
    match path.file_name().and_then(|name| name.to_str()) {
        Some("nix") | Some("nix-build") | Some("nix-store") | Some("nix-shell") | Some("nix-develop") => true,
        Some(name) if name.starts_with("nix") && name.contains("develop") => true,
        _ => false,
    }
}

fn is_valid_hex_digest(value: &str, expected_len: u32) -> bool {
    let Ok(actual_len) = u32::try_from(value.len()) else {
        return false;
    };
    if actual_len != expected_len {
        return false;
    }
    value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn normalized_path_id(path: &Path) -> String {
    let body = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .filter(|component| *component != "" && *component != "/")
        .collect::<Vec<_>>()
        .join(PATH_SEPARATOR.encode_utf8(&mut [0; 4]));
    if path.is_absolute() {
        return format!("/{body}");
    }
    body
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedLaunchAuditEvent {
    pub executable_path: PathBuf,
    pub digest_hex: String,
    pub reason: String,
    pub phase: String,
    pub inventory_entry_id: String,
    pub policy_decision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedLaunchPlan {
    pub executable_path: PathBuf,
    pub digest_hex: String,
    pub inventory_entry_id: String,
    pub reason: String,
    pub phase: String,
}

pub struct ProtectedProcessLauncher<'policy> {
    policy: &'policy ProtectedExecPolicy,
    audit_events: Vec<ProtectedLaunchAuditEvent>,
}

impl<'policy> ProtectedProcessLauncher<'policy> {
    pub fn new(policy: &'policy ProtectedExecPolicy) -> Self {
        Self {
            policy,
            audit_events: Vec::new(),
        }
    }

    pub fn prepare_command(
        &mut self,
        executable_path: &Path,
    ) -> Result<std::process::Command, Stage0InventoryGenerationError> {
        let plan = plan_protected_launch(self.policy, executable_path)?;
        self.audit_events.push(plan.audit_event());
        Ok(std::process::Command::new(executable_path))
    }

    pub fn audit_events(&self) -> &[ProtectedLaunchAuditEvent] {
        &self.audit_events
    }

    pub fn take_audit_events(&mut self) -> Vec<ProtectedLaunchAuditEvent> {
        std::mem::take(&mut self.audit_events)
    }
}

impl ProtectedLaunchPlan {
    pub fn audit_event(&self) -> ProtectedLaunchAuditEvent {
        ProtectedLaunchAuditEvent {
            executable_path: self.executable_path.clone(),
            digest_hex: self.digest_hex.clone(),
            reason: self.reason.clone(),
            phase: self.phase.clone(),
            inventory_entry_id: self.inventory_entry_id.clone(),
            policy_decision: "allowed".to_string(),
        }
    }
}

pub fn plan_protected_launch(
    policy: &ProtectedExecPolicy,
    executable_path: &Path,
) -> Result<ProtectedLaunchPlan, Stage0InventoryGenerationError> {
    assert!(executable_path.is_absolute(), "protected launch path must be absolute");
    let digest_hex = blake3_file_hex(executable_path)?;
    let decision = policy.decide_exec(&ExecRequest {
        path: executable_path.to_path_buf(),
        digest_hex: digest_hex.clone(),
    })?;
    let entry_id = decision.entry_id.unwrap_or_default();
    assert!(!entry_id.is_empty(), "allowed protected launch must name inventory entry");
    Ok(ProtectedLaunchPlan {
        executable_path: executable_path.to_path_buf(),
        digest_hex,
        inventory_entry_id: entry_id,
        reason: decision.reason,
        phase: PHASE_PROTECTED.to_string(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage0InventorySeedConfig {
    pub sandbox_entry: PathBuf,
    pub sandbox_shell: PathBuf,
    pub toolchain_root: PathBuf,
    pub build_tool_inputs: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stage0InventoryGenerationError {
    MissingEnv { var: &'static str },
    EmptyEnv { var: &'static str },
    NonUtf8Path { path: PathBuf },
    RelativeInputPath { path: PathBuf },
    MissingInputPath { path: PathBuf },
    NotExecutable { path: PathBuf },
    TooManySeedExecutables { limit: u32, actual: u32 },
    WalkDepthExceeded { path: PathBuf, limit: u32 },
    Io { path: PathBuf, message: String },
    Policy(ProtectedExecError),
}

impl fmt::Display for Stage0InventoryGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv { var } => write!(f, "missing required environment variable {var}"),
            Self::EmptyEnv { var } => write!(f, "environment variable {var} must not be empty"),
            Self::NonUtf8Path { path } => write!(f, "path is not valid UTF-8: {}", path.display()),
            Self::RelativeInputPath { path } => write!(f, "seed input path must be absolute: {}", path.display()),
            Self::MissingInputPath { path } => write!(f, "seed input path does not exist: {}", path.display()),
            Self::NotExecutable { path } => write!(f, "seed input path is not executable: {}", path.display()),
            Self::TooManySeedExecutables { limit, actual } => {
                write!(f, "too many seed executables: {actual} > {limit}")
            }
            Self::WalkDepthExceeded { path, limit } => {
                write!(f, "seed executable walk exceeded depth {limit} at {}", path.display())
            }
            Self::Io { path, message } => write!(f, "I/O error at {}: {message}", path.display()),
            Self::Policy(err) => write!(f, "stage0 inventory policy error: {err}"),
        }
    }
}

impl std::error::Error for Stage0InventoryGenerationError {}

impl From<ProtectedExecError> for Stage0InventoryGenerationError {
    fn from(value: ProtectedExecError) -> Self {
        Self::Policy(value)
    }
}

pub fn stage0_seed_config_from_process_env() -> Result<Stage0InventorySeedConfig, Stage0InventoryGenerationError> {
    let env: BTreeMap<OsString, OsString> = std::env::vars_os().collect();
    stage0_seed_config_from_env_map(&env)
}

pub fn stage0_seed_config_from_env_map(
    env: &BTreeMap<OsString, OsString>,
) -> Result<Stage0InventorySeedConfig, Stage0InventoryGenerationError> {
    let sandbox_entry = required_env_path(env, ENV_STAGE0_SEED_SANDBOX_ENTRY)?;
    let sandbox_shell = required_env_path(env, ENV_STAGE0_SEED_SANDBOX_SHELL)?;
    let toolchain_root = required_env_path(env, ENV_STAGE0_SEED_TOOLCHAIN_ROOT)?;
    let build_tool_inputs = required_env_path_list(env, ENV_STAGE0_SEED_BUILD_TOOLS)?;
    Ok(Stage0InventorySeedConfig {
        sandbox_entry,
        sandbox_shell,
        toolchain_root,
        build_tool_inputs,
    })
}

pub fn build_stage0_inventory_from_seed_config(
    config: &Stage0InventorySeedConfig,
) -> Result<Stage0Inventory, Stage0InventoryGenerationError> {
    validate_absolute_existing_path(&config.sandbox_entry)?;
    validate_absolute_existing_path(&config.sandbox_shell)?;
    validate_absolute_existing_path(&config.toolchain_root)?;
    let mut executable_entries = Vec::new();
    executable_entries.push(seed_executable_entry(
        "sandbox-entry",
        ROLE_SANDBOX_ENTRY,
        &config.sandbox_entry,
        true,
        "operator supplied protected sandbox entry",
    )?);
    executable_entries.push(seed_executable_entry(
        "sandbox-shell",
        ROLE_SANDBOX_SHELL,
        &config.sandbox_shell,
        true,
        "operator supplied protected sandbox shell",
    )?);

    let toolchain_executables = collect_seed_executables(&config.toolchain_root)?;
    append_seed_executable_entries(
        &mut executable_entries,
        ROLE_BOOTSTRAP_TOOLCHAIN_TOOL,
        "toolchain",
        &config.toolchain_root,
        &toolchain_executables,
    )?;

    for input in &config.build_tool_inputs {
        validate_absolute_existing_path(input)?;
        let build_tools = collect_seed_executables(input)?;
        append_seed_executable_entries(
            &mut executable_entries,
            ROLE_BOOTSTRAP_BUILD_TOOL,
            "build-tool",
            input,
            &build_tools,
        )?;
    }

    executable_entries.sort_by(|left, right| left.id.cmp(&right.id));
    let inventory = Stage0Inventory {
        executable_entries,
        source_entries: Vec::new(),
    };
    ProtectedExecPolicy::from_inventory(inventory.clone())?;
    Ok(inventory)
}

pub fn write_generated_stage0_inventory(
    config: &Stage0InventorySeedConfig,
    output_path: &Path,
) -> Result<Stage0Inventory, Stage0InventoryGenerationError> {
    let inventory = build_stage0_inventory_from_seed_config(config)?;
    let rendered = render_stage0_inventory_nickel(&inventory)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
    }
    fs::write(output_path, rendered).map_err(|err| io_error(output_path, err))?;
    Ok(inventory)
}

pub fn write_generated_stage0_inventory_from_env(
    output_path: &Path,
) -> Result<Stage0Inventory, Stage0InventoryGenerationError> {
    let config = stage0_seed_config_from_process_env()?;
    write_generated_stage0_inventory(&config, output_path)
}

pub fn render_stage0_inventory_nickel(inventory: &Stage0Inventory) -> Result<String, Stage0InventoryGenerationError> {
    let mut out = String::from(GENERATED_INVENTORY_HEADER);
    out.push_str("{\n");
    out.push_str("  executable_entries = [\n");
    for entry in &inventory.executable_entries {
        push_executable_entry_nickel(&mut out, entry)?;
    }
    out.push_str("  ],\n");
    out.push_str("  source_entries = [\n");
    for entry in &inventory.source_entries {
        push_source_entry_nickel(&mut out, entry);
    }
    out.push_str("  ],\n");
    out.push_str("}\n");
    Ok(out)
}

fn required_env_path(
    env: &BTreeMap<OsString, OsString>,
    var: &'static str,
) -> Result<PathBuf, Stage0InventoryGenerationError> {
    let value = env.get(OsStr::new(var)).ok_or(Stage0InventoryGenerationError::MissingEnv { var })?;
    if value.is_empty() {
        return Err(Stage0InventoryGenerationError::EmptyEnv { var });
    }
    let path = PathBuf::from(value);
    validate_absolute_path_only(&path)?;
    Ok(path)
}

fn required_env_path_list(
    env: &BTreeMap<OsString, OsString>,
    var: &'static str,
) -> Result<Vec<PathBuf>, Stage0InventoryGenerationError> {
    let value = env.get(OsStr::new(var)).ok_or(Stage0InventoryGenerationError::MissingEnv { var })?;
    if value.is_empty() {
        return Err(Stage0InventoryGenerationError::EmptyEnv { var });
    }
    let paths: Vec<PathBuf> = std::env::split_paths(value).collect();
    if paths.is_empty() {
        return Err(Stage0InventoryGenerationError::EmptyEnv { var });
    }
    for path in &paths {
        validate_absolute_path_only(path)?;
    }
    Ok(paths)
}

fn validate_absolute_path_only(path: &Path) -> Result<(), Stage0InventoryGenerationError> {
    if path.is_absolute() {
        return Ok(());
    }
    Err(Stage0InventoryGenerationError::RelativeInputPath {
        path: path.to_path_buf(),
    })
}

fn validate_absolute_existing_path(path: &Path) -> Result<(), Stage0InventoryGenerationError> {
    validate_absolute_path_only(path)?;
    if path.exists() {
        return Ok(());
    }
    Err(Stage0InventoryGenerationError::MissingInputPath {
        path: path.to_path_buf(),
    })
}

fn validate_executable_file(path: &Path) -> Result<(), Stage0InventoryGenerationError> {
    validate_absolute_existing_path(path)?;
    let metadata = fs::metadata(path).map_err(|err| io_error(path, err))?;
    if !metadata.is_file() {
        return Err(Stage0InventoryGenerationError::NotExecutable {
            path: path.to_path_buf(),
        });
    }
    if !is_executable_metadata(&metadata) {
        return Err(Stage0InventoryGenerationError::NotExecutable {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn append_seed_executable_entries(
    entries: &mut Vec<ExecutableSeedEntry>,
    role: &str,
    prefix: &str,
    root: &Path,
    executable_paths: &[PathBuf],
) -> Result<(), Stage0InventoryGenerationError> {
    assert!(!role.is_empty(), "seed role must not be empty");
    assert!(!prefix.is_empty(), "seed id prefix must not be empty");
    for path in executable_paths {
        let id = seed_id_for_path(prefix, root, path);
        entries.push(seed_executable_entry(&id, role, path, true, "operator supplied protected bootstrap seed")?);
    }
    Ok(())
}

fn seed_executable_entry(
    id: &str,
    role: &str,
    path: &Path,
    required: bool,
    allowed_reason: &str,
) -> Result<ExecutableSeedEntry, Stage0InventoryGenerationError> {
    assert!(!id.is_empty(), "seed id must not be empty");
    assert!(!role.is_empty(), "seed role must not be empty");
    validate_executable_file(path)?;
    Ok(ExecutableSeedEntry {
        schema_version: SCHEMA_VERSION_V1.to_string(),
        id: id.to_string(),
        role: role.to_string(),
        phase: PHASE_PROTECTED.to_string(),
        executable_path: path.to_path_buf(),
        digest: DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            hex: blake3_file_hex(path)?,
            interoperability_reason: None,
        },
        provenance_category: PROVENANCE_OPERATOR_BOOTSTRAP_SEED.to_string(),
        provenance: "explicit operator-supplied seed path".to_string(),
        allowed_reason: allowed_reason.to_string(),
        owner: "bootstrap".to_string(),
        required,
    })
}

fn collect_seed_executables(root: &Path) -> Result<Vec<PathBuf>, Stage0InventoryGenerationError> {
    validate_absolute_existing_path(root)?;
    let mut paths = Vec::new();
    collect_seed_executables_inner(root, 0, &mut paths)?;
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return Err(Stage0InventoryGenerationError::NotExecutable {
            path: root.to_path_buf(),
        });
    }
    Ok(paths)
}

fn collect_seed_executables_inner(
    path: &Path,
    depth: u32,
    paths: &mut Vec<PathBuf>,
) -> Result<(), Stage0InventoryGenerationError> {
    if depth > MAX_SEED_WALK_DEPTH {
        return Err(Stage0InventoryGenerationError::WalkDepthExceeded {
            path: path.to_path_buf(),
            limit: MAX_SEED_WALK_DEPTH,
        });
    }
    let metadata = fs::metadata(path).map_err(|err| io_error(path, err))?;
    if metadata.is_file() {
        if is_executable_metadata(&metadata) {
            push_bounded_seed_path(paths, path.to_path_buf())?;
            return Ok(());
        }
        return Err(Stage0InventoryGenerationError::NotExecutable {
            path: path.to_path_buf(),
        });
    }
    if metadata.is_dir() {
        let entries = fs::read_dir(path).map_err(|err| io_error(path, err))?;
        for entry in entries {
            let entry = entry.map_err(|err| io_error(path, err))?;
            collect_seed_executables_inner(&entry.path(), depth.saturating_add(1), paths)?;
        }
    }
    Ok(())
}

fn push_bounded_seed_path(paths: &mut Vec<PathBuf>, path: PathBuf) -> Result<(), Stage0InventoryGenerationError> {
    let next_len = paths.len().saturating_add(1);
    let actual = match u32::try_from(next_len) {
        Ok(value) => value,
        Err(_) => MAX_SEED_EXECUTABLES.saturating_add(1),
    };
    if actual > MAX_SEED_EXECUTABLES {
        return Err(Stage0InventoryGenerationError::TooManySeedExecutables {
            limit: MAX_SEED_EXECUTABLES,
            actual,
        });
    }
    paths.push(path);
    Ok(())
}

#[cfg(unix)]
fn is_executable_metadata(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    const EXECUTABLE_BITS: u32 = 0o111;
    metadata.permissions().mode() & EXECUTABLE_BITS != 0
}

#[cfg(not(unix))]
fn is_executable_metadata(_metadata: &fs::Metadata) -> bool {
    false
}

fn blake3_file_hex(path: &Path) -> Result<String, Stage0InventoryGenerationError> {
    let mut file = fs::File::open(path).map_err(|err| io_error(path, err))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer).map_err(|err| io_error(path, err))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn seed_id_for_path(prefix: &str, root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let normalized = normalized_path_id(relative);
    let safe = normalized
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if safe.is_empty() {
        return prefix.to_string();
    }
    format!("{prefix}-{safe}")
}

fn push_executable_entry_nickel(
    out: &mut String,
    entry: &ExecutableSeedEntry,
) -> Result<(), Stage0InventoryGenerationError> {
    out.push_str("    {\n");
    push_nickel_field(out, "schema_version", &entry.schema_version);
    push_nickel_field(out, "id", &entry.id);
    push_nickel_field(out, "role", &entry.role);
    push_nickel_field(out, "phase", &entry.phase);
    push_nickel_field(out, "executable_path", path_to_str(&entry.executable_path)?);
    push_digest_nickel(out, &entry.digest);
    push_nickel_field(out, "provenance_category", &entry.provenance_category);
    push_nickel_field(out, "provenance", &entry.provenance);
    push_nickel_field(out, "allowed_reason", &entry.allowed_reason);
    push_nickel_field(out, "owner", &entry.owner);
    push_nickel_bool_field(out, "required", entry.required);
    out.push_str("    },\n");
    Ok(())
}

fn push_source_entry_nickel(out: &mut String, entry: &SourceSeedEntry) {
    out.push_str("    {\n");
    push_nickel_field(out, "schema_version", &entry.schema_version);
    push_nickel_field(out, "id", &entry.id);
    push_nickel_field(out, "role", &entry.role);
    push_nickel_field(out, "phase", &entry.phase);
    push_nickel_string_array(out, "urls", &entry.urls);
    push_nickel_string_array(out, "extraction_rules", &entry.extraction_rules);
    push_digest_nickel(out, &entry.digest);
    push_nickel_field(out, "provenance_category", &entry.provenance_category);
    push_nickel_field(out, "provenance", &entry.provenance);
    push_nickel_field(out, "allowed_reason", &entry.allowed_reason);
    push_nickel_field(out, "owner", &entry.owner);
    push_nickel_bool_field(out, "required", entry.required);
    out.push_str("    },\n");
}

fn push_digest_nickel(out: &mut String, digest: &DigestSpec) {
    out.push_str("      digest = {\n");
    push_nickel_field(out, "algorithm", &digest.algorithm);
    push_nickel_field(out, "hex", &digest.hex);
    if let Some(reason) = &digest.interoperability_reason {
        push_nickel_field(out, "interoperability_reason", reason);
    } else {
        out.push_str("        interoperability_reason = null,\n");
    }
    out.push_str("      },\n");
}

fn push_nickel_field(out: &mut String, name: &str, value: &str) {
    out.push_str("      ");
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(&nickel_string(value));
    out.push_str(",\n");
}

fn push_nickel_bool_field(out: &mut String, name: &str, value: bool) {
    out.push_str("      ");
    out.push_str(name);
    out.push_str(" = ");
    out.push_str(if value { "true" } else { "false" });
    out.push_str(",\n");
}

fn push_nickel_string_array(out: &mut String, name: &str, values: &[String]) {
    out.push_str("      ");
    out.push_str(name);
    out.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&nickel_string(value));
    }
    out.push_str("],\n");
}

fn nickel_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            value => out.push(value),
        }
    }
    out.push('"');
    out
}

fn path_to_str(path: &Path) -> Result<&str, Stage0InventoryGenerationError> {
    path.to_str().ok_or_else(|| Stage0InventoryGenerationError::NonUtf8Path {
        path: path.to_path_buf(),
    })
}

fn io_error(path: &Path, err: io::Error) -> Stage0InventoryGenerationError {
    Stage0InventoryGenerationError::Io {
        path: path.to_path_buf(),
        message: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn digest(hex: &str) -> DigestSpec {
        DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            hex: hex.to_string(),
            interoperability_reason: None,
        }
    }

    fn executable(id: &str, role: &str, path: &str, hex: &str) -> ExecutableSeedEntry {
        ExecutableSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: id.to_string(),
            role: role.to_string(),
            phase: PHASE_PROTECTED.to_string(),
            executable_path: PathBuf::from(path),
            digest: digest(hex),
            provenance_category: PROVENANCE_TEST_FIXTURE.to_string(),
            provenance: "fixture-built seed".to_string(),
            allowed_reason: "needed for protected sandbox".to_string(),
            owner: "bootstrap".to_string(),
            required: true,
        }
    }

    fn source(id: &str, url: &str) -> SourceSeedEntry {
        SourceSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: id.to_string(),
            role: "bootstrap-source".to_string(),
            phase: PHASE_PROTECTED.to_string(),
            urls: vec![url.to_string()],
            extraction_rules: vec!["strip-components=1".to_string()],
            digest: digest(DIGEST_A),
            provenance_category: PROVENANCE_TEST_FIXTURE.to_string(),
            provenance: "fixture source".to_string(),
            allowed_reason: "bootstrap source".to_string(),
            owner: "bootstrap".to_string(),
            required: true,
        }
    }

    fn inventory() -> Stage0Inventory {
        Stage0Inventory {
            executable_entries: vec![
                executable("sandbox", ROLE_SANDBOX_ENTRY, "/seed/bin/bwrap", DIGEST_A),
                executable("shell", ROLE_SANDBOX_SHELL, "/seed/bin/busybox", DIGEST_B),
            ],
            source_entries: vec![source("musl", "https://example.invalid/musl.tar.xz")],
        }
    }

    fn make_executable(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
    }

    fn env_with_seed_paths(
        sandbox_entry: &Path,
        sandbox_shell: &Path,
        toolchain_root: &Path,
        build_tools: &[PathBuf],
    ) -> BTreeMap<OsString, OsString> {
        let mut env = BTreeMap::new();
        env.insert(OsString::from(ENV_STAGE0_SEED_SANDBOX_ENTRY), sandbox_entry.as_os_str().to_os_string());
        env.insert(OsString::from(ENV_STAGE0_SEED_SANDBOX_SHELL), sandbox_shell.as_os_str().to_os_string());
        env.insert(OsString::from(ENV_STAGE0_SEED_TOOLCHAIN_ROOT), toolchain_root.as_os_str().to_os_string());
        env.insert(OsString::from(ENV_STAGE0_SEED_BUILD_TOOLS), std::env::join_paths(build_tools).unwrap());
        env
    }

    fn inventory_with_current_exe(current_exe: &Path, digest_hex: String) -> Stage0Inventory {
        let mut inv = inventory();
        inv.executable_entries.push(ExecutableSeedEntry {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            id: "stage0-crunch".to_string(),
            role: ROLE_STAGE0_CRUNCH.to_string(),
            phase: PHASE_PROTECTED.to_string(),
            executable_path: current_exe.to_path_buf(),
            digest: DigestSpec {
                algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
                hex: digest_hex,
                interoperability_reason: None,
            },
            provenance_category: PROVENANCE_TEST_FIXTURE.to_string(),
            provenance: "current test binary".to_string(),
            allowed_reason: "stage0 crunch may launch itself".to_string(),
            owner: "bootstrap".to_string(),
            required: false,
        });
        inv
    }

    #[test]
    fn generated_inventory_ignores_fake_path_host_helper_families() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap-seed");
        let sandbox_shell = temp.path().join("seed/bin/busybox-seed");
        let toolchain_cc = temp.path().join("toolchain/bin/cc");
        let build_make = temp.path().join("build-tools/make");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");

        let fake_path_dir = temp.path().join("fake-path");
        for helper in [
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
            make_executable(&fake_path_dir.join(helper), helper.as_bytes());
        }
        let mut env = env_with_seed_paths(&sandbox_entry, &sandbox_shell, &temp.path().join("toolchain"), &[temp
            .path()
            .join("build-tools")]);
        env.insert(OsString::from("PATH"), fake_path_dir.as_os_str().to_os_string());
        let config = stage0_seed_config_from_env_map(&env).unwrap();
        let inventory = build_stage0_inventory_from_seed_config(&config).unwrap();
        let policy = ProtectedExecPolicy::from_inventory(inventory).unwrap();

        for helper in [
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
            let path = fake_path_dir.join(helper);
            let digest_hex = blake3_file_hex(&path).unwrap();
            let err = policy
                .decide_exec(&ExecRequest {
                    path: path.clone(),
                    digest_hex,
                })
                .unwrap_err();
            assert_eq!(err, ProtectedExecError::UndeclaredExecutable { path });
        }
    }

    #[test]
    fn generated_inventory_missing_required_seed_path_fails_closed() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_shell = temp.path().join("seed/bin/busybox-seed");
        let toolchain_cc = temp.path().join("toolchain/bin/cc");
        let build_make = temp.path().join("build-tools/make");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");
        let config = Stage0InventorySeedConfig {
            sandbox_entry: temp.path().join("missing/bwrap-seed"),
            sandbox_shell,
            toolchain_root: temp.path().join("toolchain"),
            build_tool_inputs: vec![temp.path().join("build-tools")],
        };

        let err = build_stage0_inventory_from_seed_config(&config).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::MissingInputPath {
            path: temp.path().join("missing/bwrap-seed")
        });
    }

    #[test]
    fn protected_launcher_records_allowed_non_shell_command() {
        let current_exe = std::env::current_exe().unwrap();
        let digest_hex = blake3_file_hex(&current_exe).unwrap();
        let inv = inventory_with_current_exe(&current_exe, digest_hex.clone());
        let policy = ProtectedExecPolicy::from_inventory(inv).unwrap();
        let mut launcher = ProtectedProcessLauncher::new(&policy);

        let mut cmd = launcher.prepare_command(&current_exe).unwrap();
        let output = cmd.arg("--help").output().unwrap();
        let events = launcher.audit_events();

        assert!(output.status.success());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].executable_path, current_exe);
        assert_eq!(events[0].digest_hex, digest_hex);
        assert_eq!(events[0].inventory_entry_id, "stage0-crunch");
        assert_eq!(events[0].policy_decision, "allowed");
        assert_eq!(events[0].phase, PHASE_PROTECTED);
    }

    #[test]
    fn protected_launcher_rejects_before_command_is_returned() {
        let current_exe = std::env::current_exe().unwrap();
        let inv = inventory_with_current_exe(&current_exe, DIGEST_A.to_string());
        let policy = ProtectedExecPolicy::from_inventory(inv).unwrap();
        let mut launcher = ProtectedProcessLauncher::new(&policy);

        let err = launcher.prepare_command(&current_exe).unwrap_err();
        assert!(matches!(err, Stage0InventoryGenerationError::Policy(ProtectedExecError::DigestMismatch { .. })));
        assert!(launcher.audit_events().is_empty());
    }

    #[test]
    fn protected_launcher_take_audit_events_drains_events() {
        let current_exe = std::env::current_exe().unwrap();
        let digest_hex = blake3_file_hex(&current_exe).unwrap();
        let inv = inventory_with_current_exe(&current_exe, digest_hex);
        let policy = ProtectedExecPolicy::from_inventory(inv).unwrap();
        let mut launcher = ProtectedProcessLauncher::new(&policy);

        let _cmd = launcher.prepare_command(&current_exe).unwrap();
        let events = launcher.take_audit_events();

        assert_eq!(events.len(), 1);
        assert!(launcher.audit_events().is_empty());
    }

    #[test]
    fn seed_config_requires_explicit_environment_inputs() {
        let env = BTreeMap::new();
        let err = stage0_seed_config_from_env_map(&env).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::MissingEnv {
            var: ENV_STAGE0_SEED_SANDBOX_ENTRY
        });

        let mut partial = BTreeMap::new();
        partial.insert(OsString::from(ENV_STAGE0_SEED_SANDBOX_ENTRY), OsString::from("/seed/bin/bwrap"));
        let err = stage0_seed_config_from_env_map(&partial).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::MissingEnv {
            var: ENV_STAGE0_SEED_SANDBOX_SHELL
        });
    }

    #[test]
    fn generated_inventory_expands_explicit_seed_tools_and_ignores_path() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap");
        let sandbox_shell = temp.path().join("seed/bin/busybox");
        let toolchain_cc = temp.path().join("toolchain/bin/x86_64-linux-musl-gcc");
        let build_make = temp.path().join("build-tools/make");
        let fake_path_tool = temp.path().join("fake-path/git");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");
        make_executable(&fake_path_tool, b"must-not-appear");

        let mut env = env_with_seed_paths(&sandbox_entry, &sandbox_shell, &temp.path().join("toolchain"), &[temp
            .path()
            .join("build-tools")]);
        env.insert(OsString::from("PATH"), temp.path().join("fake-path").as_os_str().to_os_string());
        env.insert(OsString::from("NIX_STORE"), OsString::from("/nix/store"));

        let config = stage0_seed_config_from_env_map(&env).unwrap();
        let inventory = build_stage0_inventory_from_seed_config(&config).unwrap();
        let paths: BTreeSet<PathBuf> =
            inventory.executable_entries.iter().map(|entry| entry.executable_path.clone()).collect();

        assert!(paths.contains(&sandbox_entry));
        assert!(paths.contains(&sandbox_shell));
        assert!(paths.contains(&toolchain_cc));
        assert!(paths.contains(&build_make));
        assert!(!paths.contains(&fake_path_tool));
        assert!(inventory.executable_entries.iter().any(|entry| entry.role == ROLE_BOOTSTRAP_TOOLCHAIN_TOOL));
        assert!(inventory.executable_entries.iter().any(|entry| entry.role == ROLE_BOOTSTRAP_BUILD_TOOL));
        assert_eq!(
            inventory
                .executable_entries
                .iter()
                .find(|entry| entry.executable_path == sandbox_entry)
                .unwrap()
                .digest
                .hex,
            blake3::hash(b"sandbox-entry").to_hex().to_string()
        );
    }

    #[test]
    fn generated_inventory_file_is_nickel_and_policy_valid() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap");
        let sandbox_shell = temp.path().join("seed/bin/busybox");
        let toolchain_cc = temp.path().join("toolchain/bin/cc");
        let build_make = temp.path().join("build-tools/make");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        make_executable(&toolchain_cc, b"toolchain-cc");
        make_executable(&build_make, b"build-make");
        let config = Stage0InventorySeedConfig {
            sandbox_entry: sandbox_entry.clone(),
            sandbox_shell,
            toolchain_root: temp.path().join("toolchain"),
            build_tool_inputs: vec![temp.path().join("build-tools")],
        };
        let output_path = temp.path().join("target/host-tool-free-stage0/stage0-inventory.ncl");

        let inventory = write_generated_stage0_inventory(&config, &output_path).unwrap();
        let text = fs::read_to_string(&output_path).unwrap();
        let policy = ProtectedExecPolicy::from_inventory(inventory).unwrap();

        assert!(text.contains("executable_entries"));
        assert!(text.contains("sandbox-entry"));
        assert!(text.contains("bootstrap-toolchain-tool"));
        assert!(text.contains("bootstrap-build-tool"));
        assert!(
            policy
                .decide_exec(&ExecRequest {
                    path: sandbox_entry,
                    digest_hex: blake3::hash(b"sandbox-entry").to_hex().to_string(),
                })
                .unwrap()
                .allowed
        );
    }

    #[test]
    fn generated_inventory_rejects_empty_or_non_executable_tool_roots() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox_entry = temp.path().join("seed/bin/bwrap");
        let sandbox_shell = temp.path().join("seed/bin/busybox");
        make_executable(&sandbox_entry, b"sandbox-entry");
        make_executable(&sandbox_shell, b"sandbox-shell");
        fs::create_dir_all(temp.path().join("empty-toolchain")).unwrap();
        let config = Stage0InventorySeedConfig {
            sandbox_entry,
            sandbox_shell,
            toolchain_root: temp.path().join("empty-toolchain"),
            build_tool_inputs: vec![temp.path().join("empty-toolchain")],
        };

        let err = build_stage0_inventory_from_seed_config(&config).unwrap_err();
        assert_eq!(err, Stage0InventoryGenerationError::NotExecutable {
            path: temp.path().join("empty-toolchain")
        });
    }

    #[test]
    fn policy_allows_declared_seed_with_matching_digest() {
        let policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let decision = policy
            .decide_exec(&ExecRequest {
                path: PathBuf::from("/seed/bin/bwrap"),
                digest_hex: DIGEST_A.to_string(),
            })
            .unwrap();

        assert!(decision.allowed);
        assert_eq!(decision.entry_id.as_deref(), Some("sandbox"));
        assert_eq!(decision.reason, "needed for protected sandbox");
    }

    #[test]
    fn policy_rejects_undeclared_and_mismatched_executables() {
        let policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let undeclared = policy
            .decide_exec(&ExecRequest {
                path: PathBuf::from("/usr/bin/tar"),
                digest_hex: DIGEST_A.to_string(),
            })
            .unwrap_err();
        assert_eq!(undeclared, ProtectedExecError::UndeclaredExecutable {
            path: PathBuf::from("/usr/bin/tar")
        });

        let mismatch = policy
            .decide_exec(&ExecRequest {
                path: PathBuf::from("/seed/bin/bwrap"),
                digest_hex: DIGEST_B.to_string(),
            })
            .unwrap_err();
        assert!(matches!(mismatch, ProtectedExecError::DigestMismatch { .. }));
    }

    #[test]
    fn inventory_rejects_forbidden_nix_even_with_digest() {
        let mut inv = inventory();
        inv.executable_entries.push(executable("nix", ROLE_BOOTSTRAP_BUILD_TOOL, "/seed/bin/nix", DIGEST_A));
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert!(matches!(err, ProtectedExecError::DisallowedNixExecutable { .. }));
    }

    #[test]
    fn inventory_rejects_bad_provenance_and_relative_paths() {
        let mut bad_provenance = inventory();
        bad_provenance.executable_entries[0].provenance_category = PROVENANCE_NIX_STORE_DISCOVERY.to_string();
        let err = ProtectedExecPolicy::from_inventory(bad_provenance).unwrap_err();
        assert!(matches!(err, ProtectedExecError::DisallowedProvenance { .. }));

        let mut relative = inventory();
        relative.executable_entries[0].executable_path = PathBuf::from("bin/bwrap");
        let err = ProtectedExecPolicy::from_inventory(relative).unwrap_err();
        assert!(matches!(err, ProtectedExecError::RelativeExecutablePath { .. }));
    }

    #[test]
    fn inventory_requires_interoperability_reason_for_non_blake3_digest() {
        let mut inv = inventory();
        inv.executable_entries[0].digest.algorithm = "sha256".to_string();
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert!(matches!(err, ProtectedExecError::MissingInteroperabilityReason { .. }));

        let mut allowed = inventory();
        allowed.executable_entries[0].digest.algorithm = "sha256".to_string();
        allowed.executable_entries[0].digest.interoperability_reason = Some("Cargo package checksum".to_string());
        let policy = ProtectedExecPolicy::from_inventory(allowed).unwrap();
        assert!(policy.executables_by_path.contains_key(Path::new("/seed/bin/bwrap")));
    }

    #[test]
    fn policy_allows_only_declared_source_urls() {
        let policy = ProtectedExecPolicy::from_inventory(inventory()).unwrap();
        let plan = policy.source_fetch_plan("https://example.invalid/musl.tar.xz").unwrap();
        assert_eq!(plan.entry_id, "musl");
        assert_eq!(plan.digest.hex, DIGEST_A);
        assert_eq!(plan.extraction_rules, vec!["strip-components=1".to_string()]);
        assert!(policy.verify_source_digest("https://example.invalid/musl.tar.xz", DIGEST_A).is_ok());

        let mismatch = policy.verify_source_digest("https://example.invalid/musl.tar.xz", DIGEST_B).unwrap_err();
        assert!(matches!(mismatch, ProtectedExecError::SourceDigestMismatch { .. }));
        let err = policy.ensure_source_url_allowed("https://example.invalid/other.tar.xz").unwrap_err();
        assert_eq!(err, ProtectedExecError::UndeclaredSourceUrl {
            url: "https://example.invalid/other.tar.xz".to_string()
        });
    }

    #[test]
    fn inventory_requires_sandbox_entry_and_shell() {
        let inv = Stage0Inventory {
            executable_entries: vec![executable("sandbox", ROLE_SANDBOX_ENTRY, "/seed/bin/bwrap", DIGEST_A)],
            source_entries: Vec::new(),
        };
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert_eq!(err, ProtectedExecError::MissingRequiredSeedRole {
            role: ROLE_SANDBOX_SHELL
        });
    }

    #[test]
    fn inventory_rejects_malformed_source_entries_before_fetch() {
        let mut inv = inventory();
        inv.source_entries[0].urls = vec![String::new()];
        let err = ProtectedExecPolicy::from_inventory(inv).unwrap_err();
        assert_eq!(err, ProtectedExecError::EmptyField {
            entry_id: "musl".to_string(),
            field: "urls"
        });

        let mut missing_rule = inventory();
        missing_rule.source_entries[0].extraction_rules = Vec::new();
        let err = ProtectedExecPolicy::from_inventory(missing_rule).unwrap_err();
        assert_eq!(err, ProtectedExecError::EmptyField {
            entry_id: "musl".to_string(),
            field: "extraction_rules"
        });
    }

    #[test]
    fn normalized_path_id_is_deterministic() {
        let id = normalized_path_id(Path::new("/seed/bin/bwrap"));
        assert_eq!(id, "/seed/bin/bwrap");
        assert_ne!(id, "/seed/bin/busybox");
    }
}
