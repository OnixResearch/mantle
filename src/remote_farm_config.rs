//! Typed operator configuration for remote build farms.
//!
//! Loads the `remote-builders.ncl` contract and produces provider-neutral
//! capability facts for route planning and coordinator matching.
//! Provider-specific transport parameters remain inside adapters.
//!
//! r[impl remote_builds.production_operator_configuration]

use crunch_build::distributed::EXTERNAL_BATCH_PROTOCOL_SCHEMA;
use crunch_build::distributed::ExternalBatchOperationKind;
use crunch_build::distributed::RemoteAttemptRetryPolicy;
use crunch_build::distributed::RemoteFailureDebugPolicy;
use crunch_build::distributed::RemoteTransferPolicy;
use serde::Deserialize;
use serde::Serialize;

use crate::remote_telemetry_export::RemoteTelemetryExportConfig;
use crate::remote_telemetry_export::validate_remote_telemetry_export_config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub struct RemoteTraceContextConfig {
    #[serde(default)]
    pub enabled: bool,
}

const DEFAULT_REMOTE_SYSTEM: &str = "x86_64-linux";
const DEFAULT_REMOTE_MAX_CONCURRENCY: u32 = 1;
const DEFAULT_REMOTE_MAX_UPLOAD_BYTES: u64 = 1_073_741_824;
const DEFAULT_REMOTE_MAX_BUILD_TIME_SECS: u64 = 3_600;
const MAX_REMOTE_BATCH_DISPATCHERS: usize = 128;
const MAX_REMOTE_BATCH_COMMAND_ARGS: usize = 128;
const MAX_REMOTE_BATCH_ARGUMENT_BYTES: usize = 4_096;
const MAX_REMOTE_BATCH_OUTPUT_BYTES: u64 = 1_048_576;
const MAX_REMOTE_BATCH_TIMEOUT_SECS: u64 = 604_800;
const MAX_REMOTE_BATCH_RECONCILE_ATTEMPTS: u32 = 1_024;
const MAX_REMOTE_BATCH_ENVIRONMENT_HANDLES: usize = 32;
const REMOTE_BATCH_ENVIRONMENT_HANDLE_PREFIX: &str = "MANTLE_BATCH_HANDLE_";
const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const DEFAULT_REMOTE_RETRY_ATTEMPTS: u32 = 3;
const DEFAULT_REMOTE_RETRY_DELAY_SECS: u64 = 5;
const DEFAULT_REMOTE_ATTEMPT_TIMEOUT_SECS: u64 = 3_600;
const FORBIDDEN_REMOTE_BATCH_SECRET_MARKERS: [&str; 6] =
    ["credential", "password", "private-key", "secret", "ticket", "token="];
const FORBIDDEN_REMOTE_BATCH_SHELL_FRAGMENTS: [&str; 7] = ["$(", "`", ";", "&&", "||", ">", "<"];

/// Transport binding for a remote builder endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransportMode {
    #[default]
    Stdio,
    SshStdio,
    P2p,
}

/// Sandbox enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteSandboxMode {
    #[default]
    Practical,
    Strict,
}

/// Network access policy during remote builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteNetworkMode {
    #[default]
    None,
    Local,
    Full,
}

/// Fallback policy when a remote build fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFallbackPolicy {
    #[default]
    Never,
    Always,
    TrustedOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteWorkspaceMode {
    #[default]
    None,
    ImmutableSnapshot,
    MutableSession,
}

impl RemoteWorkspaceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ImmutableSnapshot => "immutable-snapshot",
            Self::MutableSession => "mutable-session",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWorkspacePolicy {
    #[serde(default = "default_workspace_modes")]
    pub modes: Vec<RemoteWorkspaceMode>,
    #[serde(default = "default_workspace_authority")]
    pub authority_class: String,
    #[serde(default = "default_workspace_guest_paths")]
    pub guest_paths: Vec<String>,
    #[serde(default = "default_workspace_count")]
    pub workspace_count_max: u32,
    #[serde(default = "default_workspace_bytes")]
    pub bytes_max: u64,
    #[serde(default = "default_workspace_files")]
    pub files_max: u32,
    #[serde(default = "default_workspace_snapshots")]
    pub snapshots_max: u32,
    #[serde(default = "default_workspace_idle")]
    pub idle_generations_max: u64,
    #[serde(default = "default_workspace_age")]
    pub age_generations_max: u64,
    #[serde(default = "default_workspace_quarantine")]
    pub quarantine_count_max: u32,
}

impl Default for RemoteWorkspacePolicy {
    fn default() -> Self {
        Self {
            modes: default_workspace_modes(),
            authority_class: default_workspace_authority(),
            guest_paths: default_workspace_guest_paths(),
            workspace_count_max: default_workspace_count(),
            bytes_max: default_workspace_bytes(),
            files_max: default_workspace_files(),
            snapshots_max: default_workspace_snapshots(),
            idle_generations_max: default_workspace_idle(),
            age_generations_max: default_workspace_age(),
            quarantine_count_max: default_workspace_quarantine(),
        }
    }
}

fn default_workspace_modes() -> Vec<RemoteWorkspaceMode> {
    vec![RemoteWorkspaceMode::None]
}
fn default_workspace_authority() -> String {
    "local-default".to_string()
}
fn default_workspace_guest_paths() -> Vec<String> {
    vec![crunch_build::DEFAULT_WORKSPACE_GUEST_PATH.to_string()]
}
fn default_workspace_count() -> u32 {
    crunch_build::DEFAULT_WORKSPACE_COUNT_MAX
}
fn default_workspace_bytes() -> u64 {
    crunch_build::DEFAULT_WORKSPACE_BYTES_MAX
}
fn default_workspace_files() -> u32 {
    crunch_build::DEFAULT_WORKSPACE_FILES_MAX
}
fn default_workspace_snapshots() -> u32 {
    crunch_build::DEFAULT_WORKSPACE_SNAPSHOTS_MAX
}
fn default_workspace_idle() -> u64 {
    crunch_build::DEFAULT_WORKSPACE_IDLE_GENERATIONS_MAX
}
fn default_workspace_age() -> u64 {
    crunch_build::DEFAULT_WORKSPACE_AGE_GENERATIONS_MAX
}
fn default_workspace_quarantine() -> u32 {
    crunch_build::DEFAULT_WORKSPACE_QUARANTINE_COUNT_MAX
}

/// Capability profile for a remote builder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RemoteCapabilityProfile {
    #[serde(default = "default_system")]
    pub system: String,
    #[serde(default = "default_sandbox_mode")]
    pub sandbox_mode: RemoteSandboxMode,
    #[serde(default = "default_network_mode")]
    pub network_mode: RemoteNetworkMode,
    #[serde(default = "empty_strings")]
    pub features: Vec<String>,
    #[serde(default = "default_worker_generation")]
    pub worker_generation: u64,
    #[serde(default = "default_concurrency")]
    pub max_concurrency: u32,
    #[serde(default = "absent_resource_inventory")]
    pub resource_inventory: Option<crunch_build::distributed::RemoteWorkerResourceInventory>,
    #[serde(default = "default_upload_bytes")]
    pub max_upload_bytes: u64,
    #[serde(default = "default_build_time_secs")]
    pub max_build_time_secs: u64,
    #[serde(default = "default_retry_policy")]
    pub retry_policy: RemoteAttemptRetryPolicy,
    #[serde(default = "default_transfer_policy")]
    pub transfer_policy: RemoteTransferPolicy,
    #[serde(default)]
    pub workspace_policy: RemoteWorkspacePolicy,
}

fn default_sandbox_mode() -> RemoteSandboxMode {
    RemoteSandboxMode::Practical
}
fn default_system() -> String {
    DEFAULT_REMOTE_SYSTEM.to_string()
}
fn default_network_mode() -> RemoteNetworkMode {
    RemoteNetworkMode::None
}
fn default_worker_generation() -> u64 {
    1
}
fn default_concurrency() -> u32 {
    DEFAULT_REMOTE_MAX_CONCURRENCY
}
fn default_upload_bytes() -> u64 {
    DEFAULT_REMOTE_MAX_UPLOAD_BYTES
}
fn default_build_time_secs() -> u64 {
    DEFAULT_REMOTE_MAX_BUILD_TIME_SECS
}

fn empty_strings() -> Vec<String> {
    Vec::new()
}

fn empty_trust_roots() -> Vec<TrustRoot> {
    Vec::new()
}

fn empty_remote_endpoints() -> Vec<RemoteEndpoint> {
    Vec::new()
}

fn empty_publisher_profiles() -> Vec<PublisherProfile> {
    Vec::new()
}

fn empty_remote_builder_pools() -> Vec<RemoteBuilderPool> {
    Vec::new()
}

fn empty_remote_batch_dispatchers() -> Vec<RemoteBatchDispatcherProfile> {
    Vec::new()
}

fn empty_external_batch_operations() -> Vec<ExternalBatchOperationKind> {
    Vec::new()
}

fn empty_environment_handles() -> Vec<RemoteBatchEnvironmentHandle> {
    Vec::new()
}

fn absent_resource_inventory() -> Option<crunch_build::distributed::RemoteWorkerResourceInventory> {
    None
}

fn empty_string() -> String {
    String::new()
}

fn default_retry_policy() -> RemoteAttemptRetryPolicy {
    RemoteAttemptRetryPolicy {
        max_attempts: DEFAULT_REMOTE_RETRY_ATTEMPTS,
        retry_delay_secs: DEFAULT_REMOTE_RETRY_DELAY_SECS,
        attempt_timeout_secs: DEFAULT_REMOTE_ATTEMPT_TIMEOUT_SECS,
    }
}

fn default_transfer_policy() -> RemoteTransferPolicy {
    RemoteTransferPolicy {
        chunk_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CHUNK_BYTES,
        in_flight_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_BYTES,
        in_flight_chunks_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_CHUNKS,
        buffered_chunks_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_BUFFERED_CHUNKS,
        artifact_count_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_ARTIFACTS,
        chunk_count_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CHUNKS,
        total_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_TOTAL_BYTES,
        checkpoint_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CHECKPOINT_BYTES,
        idle_progress_steps_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS,
        replay_rounds_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_REPLAY_ROUNDS,
        control_bytes_max: crunch_build::distributed::DEFAULT_REMOTE_TRANSFER_CONTROL_BYTES,
    }
}

fn default_failure_debug_policy() -> RemoteFailureDebugPolicy {
    RemoteFailureDebugPolicy {
        policy_name: crunch_build::distributed::REMOTE_FAILURE_DEBUG_POLICY_NAME.to_string(),
        metadata_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_METADATA_BYTES,
        object_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_OBJECT_BYTES,
        retention_secs: crunch_build::distributed::DEFAULT_REMOTE_FAILURE_RETENTION_SECS,
        replay_enabled: false,
        capture: crunch_build::distributed::RemoteFailureCapturePolicy {
            enabled: false,
            allowed_relative_paths: Vec::new(),
            sensitivity: crunch_build::distributed::RemoteFailureCaptureSensitivity::RestrictedDiagnostic,
            file_count_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_FILES,
            total_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_TOTAL_BYTES,
            file_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_FILE_BYTES,
            depth_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_DEPTH,
            failure_mode: crunch_build::distributed::RemoteFailureCaptureFailureMode::DiagnosticOnly,
        },
    }
}

impl Default for RemoteCapabilityProfile {
    fn default() -> Self {
        Self {
            system: DEFAULT_REMOTE_SYSTEM.to_string(),
            sandbox_mode: RemoteSandboxMode::Practical,
            network_mode: RemoteNetworkMode::None,
            features: Vec::new(),
            worker_generation: default_worker_generation(),
            max_concurrency: DEFAULT_REMOTE_MAX_CONCURRENCY,
            resource_inventory: None,
            max_upload_bytes: DEFAULT_REMOTE_MAX_UPLOAD_BYTES,
            max_build_time_secs: DEFAULT_REMOTE_MAX_BUILD_TIME_SECS,
            retry_policy: default_retry_policy(),
            transfer_policy: default_transfer_policy(),
            workspace_policy: RemoteWorkspacePolicy::default(),
        }
    }
}

/// A remote builder endpoint with transport and capability facts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct RemoteEndpoint {
    pub endpoint_id: String,
    #[serde(default = "default_transport")]
    pub transport: RemoteTransportMode,
    #[serde(default)]
    pub profile: RemoteCapabilityProfile,
}

fn default_transport() -> RemoteTransportMode {
    RemoteTransportMode::Stdio
}

/// A cryptographic trust root for verifying remote outputs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct TrustRoot {
    pub public_key_name: String,
    pub public_key_base64: String,
}

/// Publication target configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum PublisherMode {
    #[default]
    NixCache,
    LocalArchive,
    S3,
}

fn default_publisher_mode() -> PublisherMode {
    PublisherMode::NixCache
}

/// Publisher profile configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct PublisherProfile {
    #[serde(default = "default_publisher_mode")]
    pub mode: PublisherMode,
    #[serde(default = "empty_string")]
    pub target_url: String,
    #[serde(default = "empty_trust_roots")]
    pub trusted_public_keys: Vec<TrustRoot>,
}

/// A configured remote builder pool with typed capability and trust facts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct RemoteBuilderPool {
    pub pool_id: String,
    #[serde(default = "empty_remote_endpoints")]
    pub endpoints: Vec<RemoteEndpoint>,
    #[serde(default = "default_fallback_policy")]
    pub fallback_policy: RemoteFallbackPolicy,
    #[serde(default = "empty_trust_roots")]
    pub output_trust_roots: Vec<TrustRoot>,
    #[serde(default = "empty_publisher_profiles")]
    pub publishers: Vec<PublisherProfile>,
}

fn default_fallback_policy() -> RemoteFallbackPolicy {
    RemoteFallbackPolicy::TrustedOnly
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteBatchDispatcherAdapter {
    DirectProcessV1,
    SlurmCliV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteBatchBootstrapPolicy {
    DirectWorkerExecV1,
    FixedTemplateV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteBatchEnvironmentHandle {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteBatchDispatcherCommandProfile {
    pub program: std::path::PathBuf,
    pub expected_digest_blake3: String,
    #[serde(default = "empty_strings")]
    pub args: Vec<String>,
    pub timeout_secs: u64,
    pub stdout_limit_bytes: u64,
    pub stderr_limit_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteBatchDispatcherProfile {
    pub instance_id: String,
    pub generation: u64,
    pub adapter: RemoteBatchDispatcherAdapter,
    pub protocol_schema: String,
    #[serde(default = "empty_external_batch_operations")]
    pub allowed_operations: Vec<ExternalBatchOperationKind>,
    pub provider_class: String,
    pub submit: RemoteBatchDispatcherCommandProfile,
    pub observe: RemoteBatchDispatcherCommandProfile,
    pub cancel: RemoteBatchDispatcherCommandProfile,
    pub reconcile: RemoteBatchDispatcherCommandProfile,
    pub worker_program: std::path::PathBuf,
    pub worker_program_digest_blake3: String,
    #[serde(default = "empty_strings")]
    pub worker_args: Vec<String>,
    #[serde(default = "empty_environment_handles")]
    pub environment_handles: Vec<RemoteBatchEnvironmentHandle>,
    pub bootstrap_policy: RemoteBatchBootstrapPolicy,
    pub redact_provider_output: bool,
    pub startup_timeout_secs: u64,
    pub terminal_timeout_secs: u64,
    pub max_reconcile_attempts: u32,
}

#[derive(Serialize)]
struct RemoteBatchCommandIdentity<'a> {
    expected_digest_blake3: &'a str,
    args: &'a [String],
    timeout_secs: u64,
    stdout_limit_bytes: u64,
    stderr_limit_bytes: u64,
}

#[derive(Serialize)]
struct RemoteBatchProfileIdentity<'a> {
    instance_id: &'a str,
    generation: u64,
    adapter: RemoteBatchDispatcherAdapter,
    protocol_schema: &'a str,
    allowed_operations: &'a [ExternalBatchOperationKind],
    provider_class: &'a str,
    submit: RemoteBatchCommandIdentity<'a>,
    observe: RemoteBatchCommandIdentity<'a>,
    cancel: RemoteBatchCommandIdentity<'a>,
    reconcile: RemoteBatchCommandIdentity<'a>,
    worker_program_digest_blake3: &'a str,
    worker_args: &'a [String],
    environment_handles: &'a [RemoteBatchEnvironmentHandle],
    bootstrap_policy: RemoteBatchBootstrapPolicy,
    redact_provider_output: bool,
    startup_timeout_secs: u64,
    terminal_timeout_secs: u64,
    max_reconcile_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct RemoteBuildFarmConfig {
    #[serde(default = "empty_remote_builder_pools")]
    pub pools: Vec<RemoteBuilderPool>,
    #[serde(default = "empty_remote_batch_dispatchers")]
    pub dispatchers: Vec<RemoteBatchDispatcherProfile>,
    #[serde(default)]
    pub telemetry: RemoteTelemetryExportConfig,
    #[serde(default)]
    pub trace_context: RemoteTraceContextConfig,
    #[serde(default = "default_failure_debug_policy")]
    pub failure_debug: RemoteFailureDebugPolicy,
}

impl RemoteBuildFarmConfig {
    /// Maximum number of configured pools.
    pub const MAX_POOLS: usize = 32;

    /// Maximum number of endpoints per pool.
    pub const MAX_ENDPOINTS_PER_POOL: usize = 64;

    /// Validate the configuration against bounded limits.
    pub fn validate(&self) -> Result<(), String> {
        if self.pools.len() > Self::MAX_POOLS {
            return Err(format!("too many remote builder pools: {} > {}", self.pools.len(), Self::MAX_POOLS));
        }
        if self.dispatchers.len() > MAX_REMOTE_BATCH_DISPATCHERS {
            return Err(format!(
                "too many remote batch dispatchers: {} > {MAX_REMOTE_BATCH_DISPATCHERS}",
                self.dispatchers.len()
            ));
        }
        validate_remote_batch_dispatchers(&self.dispatchers)?;
        for pool in &self.pools {
            if pool.endpoints.len() > Self::MAX_ENDPOINTS_PER_POOL {
                return Err(format!(
                    "pool '{}' has too many endpoints: {} > {}",
                    pool.pool_id,
                    pool.endpoints.len(),
                    Self::MAX_ENDPOINTS_PER_POOL
                ));
            }
            if pool.pool_id.is_empty() {
                return Err("pool_id must not be empty".to_string());
            }
            for endpoint in &pool.endpoints {
                if endpoint.endpoint_id.is_empty() {
                    return Err("endpoint_id must not be empty".to_string());
                }
                if endpoint.profile.worker_generation == 0 {
                    return Err(format!("endpoint '{}' worker generation must be positive", endpoint.endpoint_id));
                }
                endpoint.profile.retry_policy.validate().map_err(|reason| {
                    format!("endpoint '{}' retry policy: {}", endpoint.endpoint_id, reason.as_str())
                })?;
                if let Some(inventory) = &endpoint.profile.resource_inventory {
                    crunch_build::distributed::canonical_remote_worker_resource_inventory(inventory).map_err(
                        |reason| format!("endpoint '{}' resource inventory: {}", endpoint.endpoint_id, reason.as_str()),
                    )?;
                }
                endpoint.profile.transfer_policy.validate().map_err(|reason| {
                    format!("endpoint '{}' transfer policy: {}", endpoint.endpoint_id, reason.as_str())
                })?;
                validate_remote_workspace_policy(&endpoint.profile.workspace_policy)
                    .map_err(|reason| format!("endpoint '{}' workspace policy: {reason}", endpoint.endpoint_id))?;
            }
            // Check for duplicate endpoint IDs within a pool.
            let mut seen = std::collections::BTreeSet::new();
            for endpoint in &pool.endpoints {
                if !seen.insert(&endpoint.endpoint_id) {
                    return Err(format!("duplicate endpoint_id '{}' in pool '{}'", endpoint.endpoint_id, pool.pool_id));
                }
            }
        }
        validate_remote_telemetry_export_config(&self.telemetry)
            .map_err(|reason| format!("remote telemetry configuration: {reason}"))?;
        self.failure_debug
            .validate()
            .map_err(|reason| format!("remote failure debug configuration: {}", reason.as_str()))?;
        // Check for duplicate pool IDs.
        let mut seen_pools = std::collections::BTreeSet::new();
        for pool in &self.pools {
            if !seen_pools.insert(&pool.pool_id) {
                return Err(format!("duplicate pool_id '{}'", pool.pool_id));
            }
        }
        Ok(())
    }
}

pub(crate) const fn remote_batch_command_arg_limit() -> usize {
    MAX_REMOTE_BATCH_COMMAND_ARGS
}

fn validate_remote_batch_dispatchers(dispatchers: &[RemoteBatchDispatcherProfile]) -> Result<(), String> {
    let mut instance_ids = std::collections::BTreeSet::new();
    for dispatcher in dispatchers {
        validate_remote_batch_identifier(ConfigField::new("instance", &dispatcher.instance_id))?;
        if !instance_ids.insert(dispatcher.instance_id.clone()) {
            return Err(format!("duplicate remote batch dispatcher instance id {:?}", dispatcher.instance_id));
        }
        validate_remote_batch_dispatcher_profile(dispatcher)?;
    }
    Ok(())
}

// r[impl external_batch_dispatchers.typed_configuration]
// r[impl external_batch_dispatchers.canonical_identity]
pub(crate) fn validate_remote_batch_dispatcher_profile(
    dispatcher: &RemoteBatchDispatcherProfile,
) -> Result<(), String> {
    validate_remote_batch_identifier(ConfigField::new("instance", &dispatcher.instance_id))?;
    validate_remote_batch_identifier(ConfigField::new("provider class", &dispatcher.provider_class))?;
    if dispatcher.generation == 0 {
        return Err(format!("remote batch dispatcher {:?} generation must be positive", dispatcher.instance_id));
    }
    if dispatcher.protocol_schema != EXTERNAL_BATCH_PROTOCOL_SCHEMA {
        return Err(format!("remote batch dispatcher {:?} protocol schema is unsupported", dispatcher.instance_id));
    }
    debug_assert!(!dispatcher.instance_id.is_empty());
    debug_assert!(!dispatcher.provider_class.is_empty());
    validate_remote_batch_allowed_operations(dispatcher)?;
    validate_remote_batch_command(RemoteBatchCommandValidation::new(dispatcher, "submit", &dispatcher.submit))?;
    validate_remote_batch_command(RemoteBatchCommandValidation::new(dispatcher, "observe", &dispatcher.observe))?;
    validate_remote_batch_command(RemoteBatchCommandValidation::new(dispatcher, "cancel", &dispatcher.cancel))?;
    validate_remote_batch_command(RemoteBatchCommandValidation::new(dispatcher, "reconcile", &dispatcher.reconcile))?;
    let worker_context = RemoteBatchOperationContext::new(&dispatcher.instance_id, "worker");
    validate_absolute_program(worker_context, &dispatcher.worker_program)?;
    validate_blake3_digest(ConfigField::new(
        &format!("remote batch dispatcher {:?} worker executable", dispatcher.instance_id),
        &dispatcher.worker_program_digest_blake3,
    ))?;
    validate_remote_batch_args(worker_context, &dispatcher.worker_args)?;
    validate_remote_batch_secret_free_args(worker_context, &dispatcher.worker_args)?;
    validate_remote_batch_environment_handles(dispatcher)?;
    if !dispatcher.redact_provider_output {
        return Err(format!(
            "remote batch dispatcher {:?} provider output redaction must be enabled",
            dispatcher.instance_id
        ));
    }
    validate_remote_batch_limits(dispatcher)
}

fn validate_remote_batch_allowed_operations(dispatcher: &RemoteBatchDispatcherProfile) -> Result<(), String> {
    if dispatcher.allowed_operations.is_empty() {
        return Err(format!("remote batch dispatcher {:?} allowed operations are empty", dispatcher.instance_id));
    }
    let unique = dispatcher.allowed_operations.iter().copied().collect::<std::collections::BTreeSet<_>>();
    if unique.len() != dispatcher.allowed_operations.len() {
        return Err(format!(
            "remote batch dispatcher {:?} allowed operations contain duplicates",
            dispatcher.instance_id
        ));
    }
    Ok(())
}

fn validate_remote_batch_environment_handles(dispatcher: &RemoteBatchDispatcherProfile) -> Result<(), String> {
    if dispatcher.environment_handles.len() > MAX_REMOTE_BATCH_ENVIRONMENT_HANDLES {
        return Err(format!(
            "remote batch dispatcher {:?} environment handle count is exceeded",
            dispatcher.instance_id
        ));
    }
    debug_assert!(dispatcher.environment_handles.len() <= MAX_REMOTE_BATCH_ENVIRONMENT_HANDLES);
    let mut names = std::collections::BTreeSet::new();
    for handle in &dispatcher.environment_handles {
        let is_valid = handle.name.starts_with(REMOTE_BATCH_ENVIRONMENT_HANDLE_PREFIX)
            && handle.name.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_');
        if !is_valid {
            return Err(format!(
                "remote batch dispatcher {:?} environment handle name is invalid",
                dispatcher.instance_id
            ));
        }
        if !names.insert(handle.name.as_str()) {
            return Err(format!(
                "remote batch dispatcher {:?} environment handle is duplicated",
                dispatcher.instance_id
            ));
        }
    }
    debug_assert_eq!(names.len(), dispatcher.environment_handles.len());
    Ok(())
}

pub(crate) fn remote_batch_dispatcher_profile_ref(dispatcher: &RemoteBatchDispatcherProfile) -> Result<String, String> {
    validate_remote_batch_dispatcher_profile(dispatcher)?;
    debug_assert!(dispatcher.generation > 0);
    debug_assert!(dispatcher.redact_provider_output);
    let identity = RemoteBatchProfileIdentity {
        instance_id: &dispatcher.instance_id,
        generation: dispatcher.generation,
        adapter: dispatcher.adapter,
        protocol_schema: &dispatcher.protocol_schema,
        allowed_operations: &dispatcher.allowed_operations,
        provider_class: &dispatcher.provider_class,
        submit: remote_batch_command_identity(&dispatcher.submit),
        observe: remote_batch_command_identity(&dispatcher.observe),
        cancel: remote_batch_command_identity(&dispatcher.cancel),
        reconcile: remote_batch_command_identity(&dispatcher.reconcile),
        worker_program_digest_blake3: &dispatcher.worker_program_digest_blake3,
        worker_args: &dispatcher.worker_args,
        environment_handles: &dispatcher.environment_handles,
        bootstrap_policy: dispatcher.bootstrap_policy,
        redact_provider_output: dispatcher.redact_provider_output,
        startup_timeout_secs: dispatcher.startup_timeout_secs,
        terminal_timeout_secs: dispatcher.terminal_timeout_secs,
        max_reconcile_attempts: dispatcher.max_reconcile_attempts,
    };
    let bytes = serde_json::to_vec(&identity)
        .map_err(|error| format!("serializing remote batch dispatcher profile identity: {error}"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn remote_batch_command_identity(command: &RemoteBatchDispatcherCommandProfile) -> RemoteBatchCommandIdentity<'_> {
    RemoteBatchCommandIdentity {
        expected_digest_blake3: &command.expected_digest_blake3,
        args: &command.args,
        timeout_secs: command.timeout_secs,
        stdout_limit_bytes: command.stdout_limit_bytes,
        stderr_limit_bytes: command.stderr_limit_bytes,
    }
}

pub(crate) fn remote_batch_worker_bootstrap_ref(dispatcher: &RemoteBatchDispatcherProfile) -> Result<String, String> {
    validate_remote_batch_dispatcher_profile(dispatcher)?;
    let bootstrap = (&dispatcher.worker_program_digest_blake3, &dispatcher.worker_args, dispatcher.bootstrap_policy);
    let bytes = serde_json::to_vec(&bootstrap)
        .map_err(|error| format!("serializing remote batch worker bootstrap: {error}"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

#[derive(Clone, Copy)]
struct RemoteBatchOperationContext<'a> {
    instance_id: &'a str,
    operation: &'a str,
}

impl<'a> RemoteBatchOperationContext<'a> {
    fn new(instance_id: &'a str, operation: &'a str) -> Self {
        Self { instance_id, operation }
    }
}

struct RemoteBatchCommandValidation<'a> {
    context: RemoteBatchOperationContext<'a>,
    command: &'a RemoteBatchDispatcherCommandProfile,
}

impl<'a> RemoteBatchCommandValidation<'a> {
    fn new(
        dispatcher: &'a RemoteBatchDispatcherProfile,
        operation: &'a str,
        command: &'a RemoteBatchDispatcherCommandProfile,
    ) -> Self {
        Self {
            context: RemoteBatchOperationContext::new(&dispatcher.instance_id, operation),
            command,
        }
    }
}

#[derive(Clone, Copy)]
struct ConfigField<'a> {
    label: &'a str,
    value: &'a str,
}

impl<'a> ConfigField<'a> {
    fn new(label: &'a str, value: &'a str) -> Self {
        Self { label, value }
    }
}

fn validate_remote_batch_command(request: RemoteBatchCommandValidation<'_>) -> Result<(), String> {
    let context = request.context;
    let command = request.command;
    validate_absolute_program(context, &command.program)?;
    validate_blake3_digest(ConfigField::new(
        &format!("remote batch dispatcher {:?} {} executable", context.instance_id, context.operation),
        &command.expected_digest_blake3,
    ))?;
    validate_remote_batch_args(context, &command.args)?;
    validate_remote_batch_secret_free_args(context, &command.args)?;
    debug_assert!(command.program.is_absolute());
    debug_assert!(!command.expected_digest_blake3.is_empty());
    if command.timeout_secs == 0 || command.timeout_secs > MAX_REMOTE_BATCH_TIMEOUT_SECS {
        return Err(format!(
            "remote batch dispatcher {:?} {} timeout is invalid",
            context.instance_id, context.operation
        ));
    }
    if command.stdout_limit_bytes == 0 || command.stdout_limit_bytes > MAX_REMOTE_BATCH_OUTPUT_BYTES {
        return Err(format!(
            "remote batch dispatcher {:?} {} stdout limit is invalid",
            context.instance_id, context.operation
        ));
    }
    if command.stderr_limit_bytes == 0 || command.stderr_limit_bytes > MAX_REMOTE_BATCH_OUTPUT_BYTES {
        return Err(format!(
            "remote batch dispatcher {:?} {} stderr limit is invalid",
            context.instance_id, context.operation
        ));
    }
    Ok(())
}

fn validate_absolute_program(
    context: RemoteBatchOperationContext<'_>,
    program: &std::path::Path,
) -> Result<(), String> {
    if !program.is_absolute() {
        return Err(format!(
            "remote batch dispatcher {:?} {} program must be absolute",
            context.instance_id, context.operation
        ));
    }
    if program.as_os_str().is_empty() {
        return Err(format!(
            "remote batch dispatcher {:?} {} program is empty",
            context.instance_id, context.operation
        ));
    }
    Ok(())
}

fn validate_remote_batch_args(context: RemoteBatchOperationContext<'_>, args: &[String]) -> Result<(), String> {
    if args.len() > MAX_REMOTE_BATCH_COMMAND_ARGS {
        return Err(format!(
            "remote batch dispatcher {:?} {} argument count exceeds {MAX_REMOTE_BATCH_COMMAND_ARGS}",
            context.instance_id, context.operation
        ));
    }
    for arg in args {
        if arg.len() > MAX_REMOTE_BATCH_ARGUMENT_BYTES || arg.chars().any(char::is_control) {
            return Err(format!(
                "remote batch dispatcher {:?} {} argument is invalid",
                context.instance_id, context.operation
            ));
        }
        if FORBIDDEN_REMOTE_BATCH_SHELL_FRAGMENTS.iter().any(|fragment| arg.contains(fragment)) {
            return Err(format!(
                "remote batch dispatcher {:?} {} argument contains shell syntax",
                context.instance_id, context.operation
            ));
        }
    }
    debug_assert!(args.len() <= MAX_REMOTE_BATCH_COMMAND_ARGS);
    debug_assert!(args.iter().all(|arg| arg.len() <= MAX_REMOTE_BATCH_ARGUMENT_BYTES));
    Ok(())
}

fn validate_remote_batch_secret_free_args(
    context: RemoteBatchOperationContext<'_>,
    args: &[String],
) -> Result<(), String> {
    for arg in args {
        let normalized = arg.to_ascii_lowercase();
        if FORBIDDEN_REMOTE_BATCH_SECRET_MARKERS.iter().any(|marker| normalized.contains(marker)) {
            return Err(format!(
                "remote batch dispatcher {:?} {} arguments may not embed secrets",
                context.instance_id, context.operation
            ));
        }
    }
    Ok(())
}

fn validate_remote_batch_limits(dispatcher: &RemoteBatchDispatcherProfile) -> Result<(), String> {
    if dispatcher.startup_timeout_secs == 0 || dispatcher.startup_timeout_secs > MAX_REMOTE_BATCH_TIMEOUT_SECS {
        return Err(format!("remote batch dispatcher {:?} startup timeout is invalid", dispatcher.instance_id));
    }
    if dispatcher.terminal_timeout_secs == 0 || dispatcher.terminal_timeout_secs > MAX_REMOTE_BATCH_TIMEOUT_SECS {
        return Err(format!("remote batch dispatcher {:?} terminal timeout is invalid", dispatcher.instance_id));
    }
    if dispatcher.max_reconcile_attempts == 0 || dispatcher.max_reconcile_attempts > MAX_REMOTE_BATCH_RECONCILE_ATTEMPTS
    {
        return Err(format!("remote batch dispatcher {:?} reconcile attempt limit is invalid", dispatcher.instance_id));
    }
    Ok(())
}

fn validate_remote_batch_identifier(field: ConfigField<'_>) -> Result<(), String> {
    if field.value.is_empty()
        || field.value.len() > MAX_REMOTE_BATCH_ARGUMENT_BYTES
        || field.value.chars().any(char::is_control)
    {
        return Err(format!("remote batch dispatcher {} is invalid", field.label));
    }
    Ok(())
}

fn validate_blake3_digest(field: ConfigField<'_>) -> Result<(), String> {
    let is_valid = field.value.len() == BLAKE3_HEX_LENGTH_CHARS
        && field.value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if !is_valid {
        return Err(format!("{} digest is not lowercase BLAKE3 hex", field.label));
    }
    Ok(())
}

pub(crate) fn validate_remote_workspace_policy(policy: &RemoteWorkspacePolicy) -> Result<(), String> {
    if policy.modes.is_empty() || policy.authority_class.is_empty() || policy.guest_paths.is_empty() {
        return Err("modes, authority_class, and guest_paths must be non-empty".to_string());
    }
    debug_assert!(!policy.modes.is_empty());
    debug_assert!(!policy.guest_paths.is_empty());
    let mode_count = policy.modes.iter().copied().collect::<std::collections::BTreeSet<_>>().len();
    let guest_count = policy.guest_paths.iter().collect::<std::collections::BTreeSet<_>>().len();
    if mode_count != policy.modes.len() || guest_count != policy.guest_paths.len() {
        return Err("workspace modes and guest paths must be unique".to_string());
    }
    if policy
        .guest_paths
        .iter()
        .any(|path| !path.starts_with("/build/") || path.contains("/../") || path.ends_with("/.."))
    {
        return Err("workspace guest paths must be clean paths beneath /build".to_string());
    }
    let is_nonzero = policy.workspace_count_max > 0
        && policy.bytes_max > 0
        && policy.files_max > 0
        && policy.snapshots_max > 0
        && policy.idle_generations_max > 0
        && policy.age_generations_max > 0
        && policy.quarantine_count_max > 0;
    if !is_nonzero {
        return Err("workspace quotas and retention limits must be positive".to_string());
    }
    Ok(())
}

pub fn load_remote_build_farm_config(path: &std::path::Path) -> Result<RemoteBuildFarmConfig, String> {
    let nickel_search_dirs = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
    let config = crunch_eval::evaluate_and_deserialize::<RemoteBuildFarmConfig>(path, &nickel_search_dirs)
        .map_err(|error| format!("evaluating remote build farm configuration: {error}"))?;
    config.validate()?;
    debug_assert!(config.pools.len() <= RemoteBuildFarmConfig::MAX_POOLS);
    debug_assert!(
        config
            .pools
            .iter()
            .all(|pool| pool.endpoints.len() <= RemoteBuildFarmConfig::MAX_ENDPOINTS_PER_POOL)
    );
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MAX_CONCURRENCY: u32 = 4;
    const SAMPLE_MAX_BUILD_TIME_SECS: u64 = 7_200;
    const SAMPLE_MAX_ATTEMPTS: u32 = 4;
    const SAMPLE_RETRY_DELAY_SECS: u64 = 10;

    fn sample_dispatcher(adapter: RemoteBatchDispatcherAdapter) -> RemoteBatchDispatcherProfile {
        let command = RemoteBatchDispatcherCommandProfile {
            program: std::path::PathBuf::from("/bin/true"),
            expected_digest_blake3: blake3::hash(b"fixture-executable").to_hex().to_string(),
            args: vec!["--fixture".to_string()],
            timeout_secs: SAMPLE_RETRY_DELAY_SECS,
            stdout_limit_bytes: MAX_REMOTE_BATCH_OUTPUT_BYTES,
            stderr_limit_bytes: MAX_REMOTE_BATCH_OUTPUT_BYTES,
        };
        RemoteBatchDispatcherProfile {
            instance_id: "dispatcher-1".to_string(),
            generation: 1,
            adapter,
            protocol_schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
            allowed_operations: vec![
                ExternalBatchOperationKind::Submit,
                ExternalBatchOperationKind::Observe,
                ExternalBatchOperationKind::Cancel,
                ExternalBatchOperationKind::Reconcile,
            ],
            provider_class: "test-provider".to_string(),
            submit: command.clone(),
            observe: command.clone(),
            cancel: command.clone(),
            reconcile: command,
            worker_program: std::path::PathBuf::from("/bin/true"),
            worker_program_digest_blake3: blake3::hash(b"fixture-worker").to_hex().to_string(),
            worker_args: vec!["remote".to_string(), "serve".to_string()],
            environment_handles: Vec::new(),
            bootstrap_policy: RemoteBatchBootstrapPolicy::DirectWorkerExecV1,
            redact_provider_output: true,
            startup_timeout_secs: SAMPLE_RETRY_DELAY_SECS,
            terminal_timeout_secs: SAMPLE_MAX_BUILD_TIME_SECS,
            max_reconcile_attempts: SAMPLE_MAX_ATTEMPTS,
        }
    }

    fn sample_config() -> RemoteBuildFarmConfig {
        RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "build-farm-1".to_string(),
                endpoints: vec![RemoteEndpoint {
                    endpoint_id: "builder-01".to_string(),
                    transport: RemoteTransportMode::SshStdio,
                    profile: RemoteCapabilityProfile {
                        system: "x86_64-linux".to_string(),
                        sandbox_mode: RemoteSandboxMode::Strict,
                        network_mode: RemoteNetworkMode::None,
                        features: vec!["tigerstyle".to_string()],
                        worker_generation: default_worker_generation(),
                        max_concurrency: SAMPLE_MAX_CONCURRENCY,
                        resource_inventory: None,
                        max_upload_bytes: DEFAULT_REMOTE_MAX_UPLOAD_BYTES,
                        max_build_time_secs: SAMPLE_MAX_BUILD_TIME_SECS,
                        retry_policy: RemoteAttemptRetryPolicy {
                            max_attempts: SAMPLE_MAX_ATTEMPTS,
                            retry_delay_secs: SAMPLE_RETRY_DELAY_SECS,
                            attempt_timeout_secs: SAMPLE_MAX_BUILD_TIME_SECS,
                        },
                        transfer_policy: RemoteTransferPolicy::default(),
                        workspace_policy: RemoteWorkspacePolicy::default(),
                    },
                }],
                fallback_policy: RemoteFallbackPolicy::Always,
                output_trust_roots: vec![TrustRoot {
                    public_key_name: "builder-01".to_string(),
                    public_key_base64: "dGVzdC1rZXk=".to_string(),
                }],
                publishers: vec![PublisherProfile {
                    mode: PublisherMode::NixCache,
                    target_url: "https://cache.example.com".to_string(),
                    trusted_public_keys: Vec::new(),
                }],
            }],
            dispatchers: vec![sample_dispatcher(RemoteBatchDispatcherAdapter::DirectProcessV1)],
            telemetry: RemoteTelemetryExportConfig::default(),
            trace_context: RemoteTraceContextConfig::default(),
            failure_debug: RemoteFailureDebugPolicy::default(),
        }
    }

    #[test]
    fn valid_config_passes_validation() {
        let config = sample_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn batch_dispatcher_profile_identity_ignores_host_paths_but_binds_digests() {
        let first = sample_dispatcher(RemoteBatchDispatcherAdapter::DirectProcessV1);
        let mut relocated = first.clone();
        relocated.submit.program = std::path::PathBuf::from("/relocated/direct-adapter");
        relocated.worker_program = std::path::PathBuf::from("/relocated/worker");
        let first_ref = remote_batch_dispatcher_profile_ref(&first).expect("first profile ref derives");
        let relocated_ref = remote_batch_dispatcher_profile_ref(&relocated).expect("relocated profile ref derives");
        relocated.submit.expected_digest_blake3 = blake3::hash(b"changed-adapter").to_hex().to_string();
        let changed_ref = remote_batch_dispatcher_profile_ref(&relocated).expect("changed profile ref derives");

        assert_eq!(first_ref, relocated_ref);
        assert_ne!(first_ref, changed_ref);
    }

    #[test]
    fn batch_dispatcher_profiles_accept_supported_adapters() {
        let mut config = sample_config();
        config.dispatchers.push(sample_dispatcher(RemoteBatchDispatcherAdapter::SlurmCliV1));
        config.dispatchers[1].instance_id = "slurm-1".to_string();

        assert!(config.validate().is_ok());
        assert_eq!(config.dispatchers.len(), 2);
    }

    #[test]
    fn batch_dispatcher_profiles_reject_secret_args_and_bad_digests() {
        let mut secret = sample_config();
        secret.dispatchers[0].worker_args.push("--ticket=do-not-store".to_string());
        let secret_error = secret.validate().expect_err("secret-bearing worker args rejected");
        let mut digest = sample_config();
        digest.dispatchers[0].submit.expected_digest_blake3 = "sha256-is-not-accepted".to_string();
        let digest_error = digest.validate().expect_err("non-BLAKE3 executable identity rejected");

        assert!(secret_error.contains("may not embed secrets"));
        assert!(digest_error.contains("not lowercase BLAKE3 hex"));
    }

    #[test]
    fn batch_dispatcher_profiles_reject_shell_syntax_and_unapproved_environment_names() {
        let mut shell = sample_config();
        shell.dispatchers[0].submit.args.push("$(ambient-tool)".to_string());
        let shell_error = shell.validate().expect_err("configured shell syntax rejected");
        let mut environment = sample_config();
        environment.dispatchers[0].environment_handles.push(RemoteBatchEnvironmentHandle {
            name: "HOME".to_string(),
        });
        let environment_error = environment.validate().expect_err("ambient environment name rejected");

        assert!(shell_error.contains("contains shell syntax"));
        assert!(environment_error.contains("environment handle name is invalid"));
    }

    #[test]
    fn batch_dispatcher_profiles_reject_relative_programs_and_duplicate_instances() {
        let mut relative = sample_config();
        relative.dispatchers[0].observe.program = std::path::PathBuf::from("squeue");
        let relative_error = relative.validate().expect_err("relative dispatcher path rejected");
        let mut duplicate = sample_config();
        duplicate.dispatchers.push(duplicate.dispatchers[0].clone());
        let duplicate_error = duplicate.validate().expect_err("duplicate dispatcher instance rejected");

        assert!(relative_error.contains("program must be absolute"));
        assert!(duplicate_error.contains("duplicate remote batch dispatcher"));
    }

    #[test]
    fn too_many_pools_rejected() {
        let pools = (0..RemoteBuildFarmConfig::MAX_POOLS + 1)
            .map(|i| RemoteBuilderPool {
                pool_id: format!("pool-{i}"),
                endpoints: Vec::new(),
                fallback_policy: RemoteFallbackPolicy::Never,
                output_trust_roots: Vec::new(),
                publishers: Vec::new(),
            })
            .collect();
        let config = RemoteBuildFarmConfig {
            pools,
            ..RemoteBuildFarmConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn duplicate_pool_id_rejected() {
        let config = RemoteBuildFarmConfig {
            pools: vec![
                RemoteBuilderPool {
                    pool_id: "dup".to_string(),
                    ..Default::default()
                },
                RemoteBuilderPool {
                    pool_id: "dup".to_string(),
                    ..Default::default()
                },
            ],
            ..RemoteBuildFarmConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn duplicate_endpoint_id_rejected_in_pool() {
        let config = RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "pool-1".to_string(),
                endpoints: vec![
                    RemoteEndpoint {
                        endpoint_id: "builder-a".to_string(),
                        ..Default::default()
                    },
                    RemoteEndpoint {
                        endpoint_id: "builder-a".to_string(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..RemoteBuildFarmConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_endpoint_id_rejected() {
        let config = RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "pool-1".to_string(),
                endpoints: vec![RemoteEndpoint {
                    endpoint_id: "".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..RemoteBuildFarmConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn empty_pool_id_rejected() {
        let config = RemoteBuildFarmConfig {
            pools: vec![RemoteBuilderPool {
                pool_id: "".to_string(),
                ..Default::default()
            }],
            ..RemoteBuildFarmConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn default_config_valid() {
        let config = RemoteBuildFarmConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_deserializes_from_json() {
        let json = r#"{
            "pools": [{
                "pool_id": "test-pool",
                "endpoints": [{
                    "endpoint_id": "builder-01",
                    "transport": "ssh-stdio",
                    "profile": {
                        "system": "aarch64-linux",
                        "sandbox_mode": "strict",
                        "max_concurrency": 8
                    }
                }],
                "output_trust_roots": [{
                    "public_key_name": "builder-01",
                    "public_key_base64": "dGVzdA=="
                }]
            }]
        }"#;
        let config: RemoteBuildFarmConfig = serde_json::from_str(json).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.pools.len(), 1);
        assert_eq!(config.pools[0].endpoints[0].profile.system, "aarch64-linux");
        assert_eq!(config.pools[0].endpoints[0].profile.max_concurrency, 8);
    }

    #[test]
    fn nickel_batch_dispatcher_contract_round_trips_typed_profile() {
        let temp = tempfile::tempdir().expect("temporary Nickel dispatcher config dir");
        let config_path = temp.path().join("remote-dispatcher.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
let command = {
  program = "/bin/true",
  expected_digest_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  args = ["--fixture"],
  timeout_secs = 10,
  stdout_limit_bytes = 4096,
  stderr_limit_bytes = 4096,
} in
{
  pools = [],
  dispatchers = [({
    instance_id = "slurm-typed",
    generation = 2,
    adapter = 'slurm-cli-v1,
    provider_class = "slurm",
    submit = command,
    observe = command,
    cancel = command,
    reconcile = command,
    worker_program = "/bin/true",
    worker_program_digest_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    worker_args = ["remote", "serve"],
    startup_timeout_secs = 30,
    terminal_timeout_secs = 300,
    max_reconcile_attempts = 4,
  } | remote.BatchDispatcher)],
}
"#,
        )
        .expect("Nickel dispatcher config writes");
        let config = load_remote_build_farm_config(&config_path).expect("typed dispatcher config evaluates");

        assert_eq!(config.dispatchers.len(), 1);
        assert_eq!(config.dispatchers[0].adapter, RemoteBatchDispatcherAdapter::SlurmCliV1);
        assert_eq!(config.dispatchers[0].generation, 2);
    }

    #[test]
    fn nickel_batch_dispatcher_contract_rejects_type_mismatch() {
        let temp = tempfile::tempdir().expect("temporary invalid dispatcher config dir");
        let config_path = temp.path().join("invalid-remote-dispatcher.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
let command = {
  program = "/bin/true",
  expected_digest_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  timeout_secs = 10,
  stdout_limit_bytes = 4096,
  stderr_limit_bytes = 4096,
} in
let invalid_command = {
  program = "/bin/true",
  expected_digest_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  timeout_secs = "forever",
  stdout_limit_bytes = 4096,
  stderr_limit_bytes = 4096,
} in
({
  instance_id = "invalid",
  generation = 1,
  adapter = 'direct-process-v1,
  provider_class = "direct",
  submit = invalid_command,
  observe = command,
  cancel = command,
  reconcile = command,
  worker_program = "/bin/true",
  worker_program_digest_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  startup_timeout_secs = 30,
  terminal_timeout_secs = 300,
  max_reconcile_attempts = 4,
} | remote.BatchDispatcher)
"#,
        )
        .expect("invalid Nickel dispatcher config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let error = crunch_eval::evaluate_and_deserialize::<serde_json::Value>(&config_path, &import_paths)
            .expect_err("typed dispatcher contract rejects invalid timeout type");

        assert!(error.to_string().contains("contract"));
        assert!(error.to_string().contains("timeout_secs"));
    }

    #[test]
    fn nickel_retry_policy_contract_round_trips_typed_values() {
        let temp = tempfile::tempdir().expect("temporary Nickel config dir");
        let config_path = temp.path().join("remote-farm.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
let configured_attempts = 4 in
let configured_retry_delay_secs = 10 in
let configured_attempt_timeout_secs = 7200 in
{
  pools = [
    ({
      pool_id = "typed-pool",
      endpoints = [{
        endpoint_id = "typed-worker",
        profile.retry_policy = {
          max_attempts = configured_attempts,
          retry_delay_secs = configured_retry_delay_secs,
          attempt_timeout_secs = configured_attempt_timeout_secs,
        },
      }],
    } | remote.RemoteBuilderPool),
  ],
}
"#,
        )
        .expect("Nickel config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let config: RemoteBuildFarmConfig = crunch_eval::evaluate_and_deserialize(&config_path, &import_paths)
            .expect("typed Nickel remote policy evaluates");

        assert!(config.validate().is_ok());
        assert_eq!(config.pools[0].endpoints[0].profile.retry_policy.max_attempts, SAMPLE_MAX_ATTEMPTS);
        assert_eq!(config.pools[0].endpoints[0].profile.retry_policy.attempt_timeout_secs, SAMPLE_MAX_BUILD_TIME_SECS);
        assert_eq!(config.pools[0].endpoints[0].profile.transfer_policy, RemoteTransferPolicy::default(),);
    }

    #[test]
    fn nickel_telemetry_contract_defaults_disabled_and_round_trips_bounds() {
        let temp = tempfile::tempdir().expect("temporary Nickel telemetry config dir");
        let config_path = temp.path().join("remote-telemetry.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
{
  pools = [],
  telemetry = ({
    telemetry.event_capacity = 64,
    telemetry.batch_size = 16,
  } | remote.RemoteTelemetry),
}
"#,
        )
        .expect("Nickel telemetry config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let config: RemoteBuildFarmConfig = crunch_eval::evaluate_and_deserialize(&config_path, &import_paths)
            .expect("typed Nickel telemetry policy evaluates");

        assert!(config.validate().is_ok());
        assert_eq!(config.telemetry.telemetry.event_capacity, 64);
        assert_eq!(config.telemetry.telemetry.batch_size, 16);
        assert!(!config.telemetry.prometheus.enabled);
        assert!(!config.telemetry.otlp.enabled);
    }

    #[test]
    fn nickel_telemetry_contract_rejects_type_mismatch() {
        let temp = tempfile::tempdir().expect("temporary invalid telemetry config dir");
        let config_path = temp.path().join("invalid-remote-telemetry.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
{
  pools = [],
  telemetry = ({
    telemetry.event_capacity = "unbounded",
  } | remote.RemoteTelemetry),
}
"#,
        )
        .expect("invalid Nickel telemetry config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let error = crunch_eval::evaluate_and_deserialize::<RemoteBuildFarmConfig>(&config_path, &import_paths)
            .expect_err("typed Nickel telemetry policy rejects string capacity");

        assert!(error.to_string().contains("contract"));
        assert!(error.to_string().contains("event_capacity"));
    }

    #[test]
    fn nickel_retry_policy_contract_rejects_type_mismatch() {
        let temp = tempfile::tempdir().expect("temporary Nickel config dir");
        let config_path = temp.path().join("invalid-remote-farm.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
{
  pools = [
    ({
      pool_id = "invalid-pool",
      endpoints = [{
        endpoint_id = "invalid-worker",
        profile.retry_policy.max_attempts = "unbounded",
      }],
    } | remote.RemoteBuilderPool),
  ],
}
"#,
        )
        .expect("invalid Nickel config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let error = crunch_eval::evaluate_and_deserialize::<RemoteBuildFarmConfig>(&config_path, &import_paths)
            .expect_err("typed Nickel retry policy rejects a string attempt budget");

        assert!(error.to_string().contains("contract"));
        assert!(error.to_string().contains("max_attempts"));
    }

    #[test]
    fn nickel_transfer_policy_contract_rejects_type_mismatch() {
        let temp = tempfile::tempdir().expect("temporary Nickel config dir");
        let config_path = temp.path().join("invalid-transfer-policy.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
{
  pools = [({
    pool_id = "invalid-transfer-pool",
    endpoints = [{
      endpoint_id = "invalid-transfer-worker",
      profile.transfer_policy.chunk_bytes_max = "unbounded",
    }],
  } | remote.RemoteBuilderPool)],
}
"#,
        )
        .expect("invalid Nickel transfer config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let error = crunch_eval::evaluate_and_deserialize::<RemoteBuildFarmConfig>(&config_path, &import_paths)
            .expect_err("typed Nickel transfer policy rejects a string chunk budget");

        assert!(error.to_string().contains("contract"));
        assert!(error.to_string().contains("chunk_bytes_max"));
    }

    #[test]
    fn nickel_failure_debug_contract_round_trips_explicit_bounded_policy() {
        let temp = tempfile::tempdir().expect("temporary failure debug config dir");
        let config_path = temp.path().join("remote-failure-debug.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
{
  pools = [],
  failure_debug = ({
    replay_enabled = true,
    retention_secs = 3600,
    capture = {
      enabled = true,
      allowed_relative_paths = ["build/trace.json"],
      sensitivity = 'restricted-diagnostic,
      file_count_max = 2,
      total_bytes_max = 4096,
      file_bytes_max = 2048,
      depth_max = 4,
      failure_mode = 'diagnostic-only,
    },
  } | remote.RemoteFailureDebug),
}
"#,
        )
        .expect("Nickel failure debug config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let config: RemoteBuildFarmConfig = crunch_eval::evaluate_and_deserialize(&config_path, &import_paths)
            .expect("typed Nickel failure debug policy evaluates");

        assert!(config.validate().is_ok());
        assert!(config.failure_debug.replay_enabled);
        assert!(config.failure_debug.capture.enabled);
        assert_eq!(config.failure_debug.capture.allowed_relative_paths, vec!["build/trace.json"]);
        assert_eq!(config.failure_debug.capture.file_count_max, 2);
    }

    #[test]
    fn nickel_failure_debug_contract_rejects_type_mismatch() {
        let temp = tempfile::tempdir().expect("temporary invalid failure debug config dir");
        let config_path = temp.path().join("invalid-remote-failure-debug.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
{
  pools = [],
  failure_debug = ({ capture.total_bytes_max = "unbounded" } | remote.RemoteFailureDebug),
}
"#,
        )
        .expect("invalid Nickel failure debug config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let error = crunch_eval::evaluate_and_deserialize::<RemoteBuildFarmConfig>(&config_path, &import_paths)
            .expect_err("typed Nickel failure debug policy rejects string byte budget");

        assert!(error.to_string().contains("contract"));
        assert!(error.to_string().contains("total_bytes_max"));
    }

    #[test]
    fn config_defaults_are_safe() {
        // A pool with minimal fields should get safe defaults.
        let json = r#"{
            "pools": [{
                "pool_id": "minimal-pool",
                "endpoints": [{
                    "endpoint_id": "minimal-builder"
                }]
            }]
        }"#;
        let config: RemoteBuildFarmConfig = serde_json::from_str(json).unwrap();
        assert!(config.validate().is_ok());
        let endpoint = &config.pools[0].endpoints[0];
        assert_eq!(endpoint.transport, RemoteTransportMode::Stdio);
        assert_eq!(endpoint.profile.system, "x86_64-linux");
        assert_eq!(endpoint.profile.sandbox_mode, RemoteSandboxMode::Practical);
        assert_eq!(endpoint.profile.network_mode, RemoteNetworkMode::None);
        assert_eq!(endpoint.profile.max_concurrency, DEFAULT_REMOTE_MAX_CONCURRENCY);
        assert_eq!(endpoint.profile.retry_policy, RemoteAttemptRetryPolicy::default());
        assert_eq!(endpoint.profile.transfer_policy, RemoteTransferPolicy::default());
        assert!(!config.failure_debug.capture.enabled);
        assert!(!config.failure_debug.replay_enabled);
    }

    #[test]
    fn nickel_resource_inventory_round_trips_typed_bounded_values() {
        let temp = tempfile::tempdir().expect("temporary Nickel resource config dir");
        let config_path = temp.path().join("remote-resources.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in {
  pools = [({
    pool_id = "resource-pool",
    endpoints = [{
      endpoint_id = "resource-worker",
      profile.resource_inventory = ({
        total = {
          cpu_units = 16,
          memory_bytes = 68719476736,
          scratch_bytes = 1099511627776,
          accelerators = [{ name = "nvidia-sm90", quantity = 2 }],
          named_tokens = [{ name = "linker-seat", quantity = 4 }],
        },
      } | remote.ResourceInventory),
    }],
  } | remote.RemoteBuilderPool)],
}
"#,
        )
        .expect("Nickel resource config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let config: RemoteBuildFarmConfig = crunch_eval::evaluate_and_deserialize(&config_path, &import_paths)
            .expect("typed Nickel resource inventory evaluates");
        let inventory = config.pools[0].endpoints[0]
            .profile
            .resource_inventory
            .as_ref()
            .expect("resource inventory is present");

        assert!(config.validate().is_ok());
        assert_eq!(inventory.total.cpu_units, 16);
        assert_eq!(inventory.total.accelerators[0].name, "nvidia-sm90");
    }

    #[test]
    fn nickel_resource_requirements_and_locality_policy_match_rust_validation() {
        let temp = tempfile::tempdir().expect("temporary Nickel requirements dir");
        let requirements_path = temp.path().join("remote-requirements.ncl");
        std::fs::write(
            &requirements_path,
            r#"
let remote = import "remote-builders.ncl" in
({
  quantities = {
    cpu_units = 8,
    memory_bytes = 16384,
    scratch_bytes = 32768,
    accelerators = [{ name = "nvidia-sm90", quantity = 1 }],
    named_tokens = [{ name = "linker-seat", quantity = 2 }],
  },
  semantic_accelerator_classes = ["nvidia-sm90"],
} | remote.ResourceRequirements)
"#,
        )
        .expect("Nickel requirements write");
        let locality_path = temp.path().join("remote-locality-policy.ncl");
        std::fs::write(&locality_path, r#"let remote = import "remote-builders.ncl" in ({} | remote.LocalityPolicy)"#)
            .expect("Nickel locality policy writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let requirements: crunch_build::distributed::RemoteResourceRequirements =
            crunch_eval::evaluate_and_deserialize(&requirements_path, &import_paths)
                .expect("typed Nickel requirements evaluate");
        let locality: serde_json::Value = crunch_eval::evaluate_and_deserialize(&locality_path, &import_paths)
            .expect("typed Nickel locality policy evaluates");
        let canonical = crunch_build::distributed::canonical_remote_resource_requirements(&requirements)
            .expect("Rust requirement validation agrees");

        assert_eq!(canonical, requirements);
        assert_eq!(locality["basis"], "receiver-verified");
        assert_eq!(locality["stale"], "reject");
        assert_eq!(locality["unverified"], "no-verified-content");
    }

    #[test]
    fn nickel_resource_inventory_rejects_type_mismatch() {
        let temp = tempfile::tempdir().expect("temporary invalid resource config dir");
        let config_path = temp.path().join("invalid-remote-resources.ncl");
        std::fs::write(
            &config_path,
            r#"
let remote = import "remote-builders.ncl" in
({ total = { cpu_units = "many", memory_bytes = 1, scratch_bytes = 1 } }
  | remote.ResourceInventory)
"#,
        )
        .expect("invalid Nickel resource config writes");
        let import_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let result = crunch_eval::evaluate_and_deserialize::<serde_json::Value>(&config_path, &import_paths);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cpu_units"));
    }

    #[test]
    fn invalid_retry_policy_is_rejected() {
        let mut config = sample_config();
        config.pools[0].endpoints[0].profile.retry_policy.max_attempts = 0;
        let error = config.validate().expect_err("zero retry attempts fail closed");
        assert!(error.contains("retry-policy-invalid"));
        assert!(error.contains("builder-01"));
    }

    #[test]
    fn invalid_resource_inventory_is_rejected() {
        let mut config = sample_config();
        config.pools[0].endpoints[0].profile.resource_inventory =
            Some(crunch_build::distributed::RemoteWorkerResourceInventory {
                total: crunch_build::distributed::RemoteResourceVector {
                    cpu_units: 0,
                    memory_bytes: 1,
                    scratch_bytes: 1,
                    accelerators: Vec::new(),
                    named_tokens: Vec::new(),
                },
            });
        let error = config.validate().expect_err("zero CPU inventory fails closed");

        assert!(error.contains("resource-quantity-zero"));
        assert!(error.contains("builder-01"));
    }

    #[test]
    fn nickel_workspace_registration_round_trips_and_defaults_are_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("workspace-remote.ncl");
        std::fs::write(
            &path,
            r#"
let remote = import "remote-builders.ncl" in
{
  pools = [({
    pool_id = "workspace-pool",
    endpoints = [{
      endpoint_id = "workspace-worker",
      profile.workspace_policy = {
        modes = ['none, 'immutable-snapshot, 'mutable-session],
        authority_class = "tenant-a",
        guest_paths = ["/build/.mantle-workspace"],
      },
    }],
  } | remote.RemoteBuilderPool)],
}
"#,
        )
        .unwrap();
        let imports = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
        let config: RemoteBuildFarmConfig = crunch_eval::evaluate_and_deserialize(&path, &imports).unwrap();
        let policy = &config.pools[0].endpoints[0].profile.workspace_policy;
        assert!(policy.modes.contains(&RemoteWorkspaceMode::MutableSession));
        assert_eq!(policy.authority_class, "tenant-a");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn workspace_registration_rejects_duplicate_modes_and_zero_quota() {
        let mut config = sample_config();
        let policy = &mut config.pools[0].endpoints[0].profile.workspace_policy;
        policy.modes = vec![RemoteWorkspaceMode::MutableSession, RemoteWorkspaceMode::MutableSession];
        policy.bytes_max = 0;
        let error = config.validate().unwrap_err();
        assert!(error.contains("workspace policy"));
        assert!(error.contains("unique") || error.contains("positive"));
    }

    #[test]
    fn invalid_transfer_policy_is_rejected() {
        let mut config = sample_config();
        config.pools[0].endpoints[0].profile.transfer_policy.chunk_bytes_max = 0;
        let error = config.validate().expect_err("zero chunk budget fails closed");
        assert!(error.contains("transfer-policy-invalid"));
        assert!(error.contains("builder-01"));
    }
}
