// machine-artifact-public: remote.execution-reports
// machine-artifact-public: remote.attempt-future
// machine-artifact-public: remote.observability-future
// machine-artifact-public: remote.transfer-future
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Command;
use std::process::Stdio;
use std::task::Context;
use std::task::Poll;
use std::thread;
use std::time::Duration;

use crunch_build::distributed::CanonicalRemoteTransferManifest;
use crunch_build::distributed::EXTERNAL_BATCH_NON_CLAIM;
pub use crunch_build::distributed::ExternalBatchJobState;
pub use crunch_build::distributed::ExternalBatchLimits;
pub use crunch_build::distributed::ExternalBatchOperation;
pub use crunch_build::distributed::ExternalBatchOperationInput;
pub use crunch_build::distributed::ExternalBatchOperationKind;
pub use crunch_build::distributed::ExternalBatchOperationResponse;
pub use crunch_build::distributed::ExternalBatchReconcileDecision;
use crunch_build::distributed::ExternalBatchReconcileFacts;
use crunch_build::distributed::MAX_REMOTE_LOCALITY_OBSERVATIONS;
use crunch_build::distributed::REMOTE_LOCALITY_SUMMARY_SCHEMA;
pub use crunch_build::distributed::RemoteAssignmentNonce;
pub use crunch_build::distributed::RemoteAttemptApplyDisposition;
pub use crunch_build::distributed::RemoteAttemptAuthorizationFacts;
pub use crunch_build::distributed::RemoteAttemptFailureClass;
pub use crunch_build::distributed::RemoteAttemptId;
pub use crunch_build::distributed::RemoteAttemptLogCurrentAttemptFacts;
pub use crunch_build::distributed::RemoteAttemptLogDigest;
pub use crunch_build::distributed::RemoteAttemptLogManifest;
pub use crunch_build::distributed::RemoteAttemptLogPolicy;
pub use crunch_build::distributed::RemoteAttemptLogReplayPlan;
pub use crunch_build::distributed::RemoteAttemptLogReplayRequest;
pub use crunch_build::distributed::RemoteAttemptLogScope;
pub use crunch_build::distributed::RemoteAttemptPhase;
pub use crunch_build::distributed::RemoteAttemptReasonCode;
pub use crunch_build::distributed::RemoteAttemptReport;
pub use crunch_build::distributed::RemoteAttemptReportPayload;
pub use crunch_build::distributed::RemoteAttemptRetryPolicy;
pub use crunch_build::distributed::RemoteAttemptState;
pub use crunch_build::distributed::RemoteAttemptTimeFacts;
pub use crunch_build::distributed::RemoteEventId;
pub use crunch_build::distributed::RemoteFenceGeneration;
pub use crunch_build::distributed::RemoteJobId;
pub use crunch_build::distributed::RemoteLocalityProbeObservation;
pub use crunch_build::distributed::RemoteLocalityReasonCode;
pub use crunch_build::distributed::RemoteLocalityScope;
pub use crunch_build::distributed::RemotePayloadDigest;
pub use crunch_build::distributed::RemoteResourceAvailability;
pub use crunch_build::distributed::RemoteResourceLease;
pub use crunch_build::distributed::RemoteResourceLeaseScope;
use crunch_build::distributed::RemoteResourceRecoveryDisposition;
pub use crunch_build::distributed::RemoteResourceRequirements;
use crunch_build::distributed::RemoteTelemetryBuffer;
use crunch_build::distributed::RemoteTelemetryCapabilityClass;
use crunch_build::distributed::RemoteTelemetryCategory;
use crunch_build::distributed::RemoteTelemetryEvent;
use crunch_build::distributed::RemoteTelemetryMeasurementKind;
use crunch_build::distributed::RemoteTelemetryPhaseClass;
use crunch_build::distributed::RemoteTelemetryPolicy;
use crunch_build::distributed::RemoteTelemetryReasonClass;
use crunch_build::distributed::RemoteTelemetryReasonCode;
use crunch_build::distributed::RemoteTelemetryResultClass;
use crunch_build::distributed::RemoteTelemetryRetryClass;
use crunch_build::distributed::RemoteTelemetryRouteClass;
use crunch_build::distributed::RemoteTelemetryTransferClass;
use crunch_build::distributed::RemoteTransferAcknowledgement;
use crunch_build::distributed::RemoteTransferArtifactId;
use crunch_build::distributed::RemoteTransferArtifactKind;
use crunch_build::distributed::RemoteTransferChunkDemand;
use crunch_build::distributed::RemoteTransferCreditGrant;
use crunch_build::distributed::RemoteTransferCreditState;
use crunch_build::distributed::RemoteTransferDemand;
use crunch_build::distributed::RemoteTransferDigest;
use crunch_build::distributed::RemoteTransferManifest;
use crunch_build::distributed::RemoteTransferPolicy;
use crunch_build::distributed::RemoteTransferReasonCode;
use crunch_build::distributed::RemoteTransferReceiverFacts;
pub use crunch_build::distributed::RemoteVerifiedLocalitySummary;
use crunch_build::distributed::RemoteWorkerPlacementFacts;
pub use crunch_build::distributed::RemoteWorkerResourceInventory;
use crunch_build::distributed::acknowledge_remote_transfer_chunk;
use crunch_build::distributed::canonical_remote_resource_requirements;
use crunch_build::distributed::canonical_remote_worker_resource_inventory;
use crunch_build::distributed::canonicalize_remote_transfer_manifest;
use crunch_build::distributed::decide_remote_attempt_retry;
use crunch_build::distributed::derive_remote_attempt_id;
use crunch_build::distributed::normalize_remote_verified_locality;
use crunch_build::distributed::plan_external_batch_followup;
use crunch_build::distributed::plan_external_batch_operation;
use crunch_build::distributed::plan_remote_attempt_assignment;
use crunch_build::distributed::plan_remote_attempt_report;
use crunch_build::distributed::plan_remote_resource_availability;
use crunch_build::distributed::plan_remote_resource_lease_recovery;
use crunch_build::distributed::plan_remote_resource_lease_release;
use crunch_build::distributed::plan_remote_resource_reservation;
use crunch_build::distributed::rank_remote_worker_placement_candidates;
use crunch_build::distributed::remote_resource_remaining_capacity;
use crunch_build::distributed::validate_remote_resource_lease_snapshot;
use fs2::FileExt;
use nix_compat::store_path::StorePath;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::Serialize;
use snix_store::path_info::PathInfo;
use tokio::io::AsyncWrite;
use zeroize::Zeroize as _;

use crate::errors::RunError;
#[cfg(test)]
use crate::external_batch_dispatch::ConfiguredExternalBatchDispatcher;
use crate::external_batch_dispatch::ExternalBatchDispatcher;
use crate::remote_credential_state::load_ticket_state;
use crate::remote_credential_state::save_ticket_state;
pub use crate::remote_credentials::RemoteTicket;
pub use crate::remote_credentials::RemoteTicketState;
pub use crate::remote_credentials::RemoteTicketView;
use crate::remote_credentials::TicketAuthorization;
use crate::remote_credentials::TicketIssueInput;
use crate::remote_credentials::TicketPolicyFacts;
use crate::remote_credentials::TicketVerifierKey;
use crate::remote_credentials::redacted_ticket_view;
use crate::remote_telemetry_export::RemoteTelemetryAdapterHealth;
use crate::remote_telemetry_export::RemoteTelemetryAdapterStatus;
use crate::remote_telemetry_export::RemoteTelemetryExportReport;
use crate::remote_trace_context::REMOTE_TRACE_CONTEXT_CAPABILITY;
use crate::remote_trace_context::RemoteTraceContext;
use crate::remote_trace_context::RemoteTraceContextHealth;
use crate::remote_trace_context::accept_remote_trace_context;

pub const REMOTE_PROTOCOL_ALPN: &str = "mantle-remote-build/1";
pub const REMOTE_OBSERVABILITY_NON_CLAIM: &str = "remote observability is diagnostic only and does not prove build success, output validity, reproducibility, release eligibility, or physical-target determinism";
pub const REMOTE_PROTOCOL_VERSION: u32 = 1;
pub const MAX_REMOTE_CAPABILITIES: usize = 32;
pub const MAX_REMOTE_INPUT_REFS: usize = 1_000_000;
pub const MAX_REMOTE_EXPECTED_OUTPUTS: usize = 128;
pub const MAX_REMOTE_BUILD_PAYLOAD_BYTES: usize = 1_048_576;
pub const MAX_REMOTE_FRAME_BYTES: usize = 1_048_576;
const MAX_REMOTE_PATHINFO_READ_BYTES: u64 = 1_048_577;
const REMOTE_JSON_BYTE_ARRAY_MAX_CHARS_PER_BYTE: usize = 4;
const REMOTE_INLINE_NAR_FRAME_METADATA_RESERVE_BYTES: usize = 4_096;
const REMOTE_INLINE_NAR_FRAME_BUDGET_BYTES: usize = MAX_REMOTE_FRAME_BYTES / REMOTE_JSON_BYTE_ARRAY_MAX_CHARS_PER_BYTE;
const _: () = assert!(REMOTE_INLINE_NAR_FRAME_BUDGET_BYTES >= REMOTE_INLINE_NAR_FRAME_METADATA_RESERVE_BYTES);
pub const MAX_REMOTE_INLINE_NAR_PAYLOAD_BYTES: usize =
    REMOTE_INLINE_NAR_FRAME_BUDGET_BYTES.saturating_sub(REMOTE_INLINE_NAR_FRAME_METADATA_RESERVE_BYTES);
pub const MAX_REMOTE_STDIO_STDERR_BYTES: usize = 65_536;
pub const MAX_REMOTE_STDIO_FRAME_COUNT: usize = 4_096;
pub const MAX_REMOTE_STDIO_INPUT_BYTES: usize = 4_194_304;
pub const MAX_REMOTE_STATUS_ITEMS: usize = 4_096;
pub const MAX_REMOTE_EXECUTOR_ARGS: usize = 512;
pub const MAX_REMOTE_EXECUTOR_ENV_VARS: usize = 512;
pub const MAX_REMOTE_EXECUTOR_ENV_VALUE_BYTES: usize = 65_536;
pub const MAX_REMOTE_UPLOAD_BYTES: u64 = crate::remote_credentials::TICKET_UPLOAD_BYTES_MAX;
pub const MAX_REMOTE_TRANSFER_TOTAL_BYTES: u64 = MAX_REMOTE_UPLOAD_BYTES;
pub const MAX_REMOTE_TRANSFER_ARTIFACTS: usize = MAX_REMOTE_STATUS_ITEMS;
pub const MAX_REMOTE_INPUT_UPLOAD_ARTIFACTS: usize = MAX_REMOTE_STATUS_ITEMS;
pub const MAX_REMOTE_BUILD_TIME_SECS: u64 = crate::remote_credentials::TICKET_BUILD_TIME_SECS_MAX;
pub const DEFAULT_TICKET_TTL_SECS: u64 = 3_600;
pub const DEFAULT_TICKET_USES: u32 = 1;
pub const DEFAULT_TICKET_BUILD_TIME_SECS: u64 = 3_600;
pub const DEFAULT_REMOTE_STDIO_TIMEOUT_SECS: u64 = DEFAULT_TICKET_BUILD_TIME_SECS;
pub const DEFAULT_TICKET_UPLOAD_BYTES: u64 = 1_073_741_824;
pub const DEFAULT_REMOTE_CONCURRENCY: u32 = 1;
const COORDINATOR_STATE_FILE: &str = "remote-coordinator-state.json";
const COORDINATOR_MUTATION_LOCK_FILE: &str = "remote-coordinator-mutation.lock";
#[cfg(unix)]
const COORDINATOR_LOCK_FILE_MODE: u32 = 0o600;
const SECRET_REDACTION: &str = "<redacted>";
const TEMP_FILE_EXTENSION: &str = "tmp";
const REMOTE_FRAME_HEADER_BYTES: usize = std::mem::size_of::<u32>();
const REMOTE_TICKET_CREDENTIAL_BYTES_MAX: u64 = 256;
const REMOTE_TICKET_CREDENTIAL_READ_TIMEOUT_MS: u64 = 5_000;
const REMOTE_TICKET_CREDENTIAL_READ_CHUNK_BYTES: usize = 256;
const REMOTE_LOOPBACK_OUTPUT_BYTES: u64 = 64;
const DELTA_REUSE_PERCENT: u64 = 75;
const PERCENT_DENOMINATOR: u64 = 100;
const REMOTE_SESSION_NON_CLAIM: &str = "loopback remote-build session evidence proves protocol control flow only; it does not prove production P2P transport, sandbox execution, or artifact correctness";
const STDERR_TRUNCATION_MARKER: &str = "\n<stderr-truncated>";
const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const STORE_PATH_HASH_CHARS: usize = 32;
const REMOTE_ACTION_SPEC_SCHEMA: &str = "mantle-remote-action-v1";
const DEFAULT_REMOTE_ACTION_SYSTEM: &str = "x86_64-linux";
const REMOTE_LOCAL_BUILD_WORKDIR_NAME: &str = "mantle-remote-builds";
const REMOTE_FAILURE_WORKSPACE_DIR: &str = "remote-failure-workspaces";
const REMOTE_FAILURE_WORKSPACE_SCAN_MAX: usize = 2;
const REMOTE_LOCAL_EXECUTOR_THREADS: usize = 2;
const REMOTE_FAILURE_CLEANUP_ENTRIES_MAX: usize = 4_096;
#[cfg(unix)]
const REMOTE_FAILURE_CLEANUP_DIRECTORY_MODE: u32 = 0o700;
#[cfg(unix)]
const REMOTE_FAILURE_CLEANUP_FILE_MODE: u32 = 0o600;
const REMOTE_CLIENT_REQUEST_ID_LABEL: &str = "remote-client-request";
const REMOTE_CLIENT_SESSION_ID_LABEL: &str = "remote-client-session";
const REMOTE_TRANSFER_ARTIFACTS_PER_PATHINFO_OUTPUT: usize = 2;
const MAX_REMOTE_WORKER_CONCURRENCY: u32 = 1_024;
const MAX_REMOTE_LOG_CHUNKS: usize = 1_024;
const MAX_REMOTE_LOG_BYTES: u64 = 1_048_576;
const MIN_REMOTE_ATTEMPT_LOG_SHELL_PAYLOAD_BYTES: u64 = 10;
const MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES: usize = 512;
const LEGACY_LOG_MIGRATION_CLASSIFICATION: &str = "legacy-unbound-diagnostics-discarded";
const LEGACY_LOG_MIGRATION_COUNT_MAX_U64: u64 = 4_294_967_295;
const LEGACY_LOG_MIGRATION_COUNT_MAX_U32: u32 = 4_294_967_295;
const LEGACY_LOG_MIGRATION_PAYLOAD_BYTES_MAX: u64 = 18_446_744_073_709_551_615;
const LEGACY_LOG_MIGRATION_NON_CLAIM: &str =
    "legacy mutable log vectors are not immutable attempt-log history and were not rehashed or promoted";
const REMOTE_COORDINATOR_BUILD_KEY_LABEL: &str = "remote-coordinator-build-key";
const REMOTE_COORDINATOR_JOB_ID_LABEL: &str = "remote-coordinator-job";
const REMOTE_ASSIGNMENT_NONCE_LABEL: &str = "remote-assignment-nonce";
const EXTERNAL_BATCH_PLAN_REPORT_SCHEMA: &str = "mantle-external-batch-plan-v1";
const EXTERNAL_BATCH_COMPOSITION_EVIDENCE_SCHEMA: &str = "mantle-external-batch-composition-v1";
const REMOTE_RESOURCE_STATUS_NON_CLAIMS: [&str; 5] = [
    "resource leases authorize scheduling capacity only",
    "named tokens do not prove tool identity or license compliance",
    "locality is receiver-verified for one manifest policy and worker generation",
    "no global placement optimality",
    "no output trust or release reproducibility",
];
const REMOTE_ASSIGNMENT_NONCE_BYTES: usize = 32;
const LEGACY_MISSING_ATTEMPT_ID: &str = "legacy-missing-attempt-id";
const REMOTE_CHILD_POLL_INTERVAL_MS: u64 = 10;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const REMOTE_CHILD_TEARDOWN_TIMEOUT_SECS: u64 = 5;
const DEFAULT_REMOTE_ATTEMPT_MAX: u32 = 3;
const DEFAULT_REMOTE_RETRY_DELAY_SECS: u64 = 5;
const DEFAULT_REMOTE_ATTEMPT_TIMEOUT_SECS: u64 = 3_600;
const MAX_REMOTE_STDIO_COMPAT_OUTPUT_BYTES: usize = 67_108_864;
const REMOTE_TRANSFER_LEASE_DURATION_SECS: u64 = 3_600;
const REMOTE_TRANSFER_SPOOL_DIR: &str = "remote-transfer-spool";
const REMOTE_INPUT_NAR_ARTIFACT_DOMAIN: &str = "remote-input-nar-artifact-v1";
const REMOTE_OUTPUT_NAR_ARTIFACT_DOMAIN: &str = "remote-output-nar-artifact-v1";
const REMOTE_OUTPUT_PATHINFO_ARTIFACT_DOMAIN: &str = "remote-output-pathinfo-artifact-v1";
const REMOTE_PRODUCTION_TRANSFER_NON_CLAIM: &str = "bounded transfer completion proves verified transport bytes only; ordinary signed PathInfo, requested identity, builder trust, attestation, and output import remain separate";
const SSH_STDIO_FIXED_ARG_COUNT: usize = 2;
const REMOTE_UPLOAD_PROOF_PREFIX: &str = "proof:";
const REMOTE_UPLOAD_SECRET_DESCRIPTOR_PREFIX: &str = "secret-descriptor:";
const REMOTE_OPERATOR_E2E_RAIL_SCHEMA: &str = "mantle-remote-build-operator-e2e-rail-v1";
const REMOTE_OPERATOR_E2E_RAIL_NON_CLAIM: &str = "remote-build e2e rail proves deterministic fixture composition only; it does not prove production P2P deployment, release reproducibility, or general package-manager compatibility";
const REMOTE_E2E_ROUTE_PLANNED_PHASE: &str = "route-planned";
const REMOTE_E2E_HANDSHAKE_COMPLETE_PHASE: &str = "handshake-complete";
const REMOTE_E2E_INPUT_SYNC_COMPLETE_PHASE: &str = "input-sync-complete";
const REMOTE_E2E_EXECUTION_COMPLETE_PHASE: &str = "remote-execution-complete";
const REMOTE_E2E_OUTPUT_ADMISSION_COMPLETE_PHASE: &str = "output-admission-complete";

fn absent_remote_value<T>() -> Option<T> {
    None
}

fn empty_remote_values<T>() -> Vec<T> {
    Vec::new()
}

fn empty_remote_map<K, V>() -> BTreeMap<K, V> {
    BTreeMap::new()
}

fn remote_false() -> bool {
    false
}

fn remote_zero_u64() -> u64 {
    0
}

fn default_remote_failure_debug_policy() -> crunch_build::distributed::RemoteFailureDebugPolicy {
    use crunch_build::distributed::RemoteFailureCaptureFailureMode;
    use crunch_build::distributed::RemoteFailureCapturePolicy;
    use crunch_build::distributed::RemoteFailureCaptureSensitivity;

    let policy = crunch_build::distributed::RemoteFailureDebugPolicy {
        policy_name: crunch_build::distributed::REMOTE_FAILURE_DEBUG_POLICY_NAME.to_string(),
        metadata_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_METADATA_BYTES,
        object_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_OBJECT_BYTES,
        retention_secs: crunch_build::distributed::DEFAULT_REMOTE_FAILURE_RETENTION_SECS,
        replay_enabled: false,
        capture: RemoteFailureCapturePolicy {
            enabled: false,
            allowed_relative_paths: Vec::new(),
            sensitivity: RemoteFailureCaptureSensitivity::RestrictedDiagnostic,
            file_count_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_FILES,
            total_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_TOTAL_BYTES,
            file_bytes_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_FILE_BYTES,
            depth_max: crunch_build::distributed::MAX_REMOTE_FAILURE_CAPTURE_DEPTH,
            failure_mode: RemoteFailureCaptureFailureMode::DiagnosticOnly,
        },
    };
    debug_assert!(policy.validate().is_ok());
    debug_assert!(!policy.policy_name.is_empty());
    policy
}

fn empty_remote_telemetry_buffer() -> RemoteTelemetryBuffer {
    RemoteTelemetryBuffer {
        events: Vec::new(),
        accepted_events: 0,
        dropped_events: 0,
        last_drop_reason: None,
    }
}

fn default_remote_telemetry_policy() -> RemoteTelemetryPolicy {
    RemoteTelemetryPolicy {
        event_capacity: crunch_build::distributed::DEFAULT_REMOTE_TELEMETRY_EVENT_CAPACITY,
        batch_size: crunch_build::distributed::DEFAULT_REMOTE_TELEMETRY_BATCH_SIZE,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteHello {
    pub alpn: String,
    pub version: u32,
    pub endpoint_id: String,
    pub capabilities: Vec<String>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub workspace_policy: Option<crate::remote_farm_config::RemoteWorkspacePolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedHello {
    pub endpoint_id: String,
    pub accepted_capabilities: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketAuthRequest {
    pub ticket_id: String,
    pub secret: String,
    /// Untrusted compatibility claim. Authorization ignores this value.
    pub client_endpoint: Option<String>,
    /// Untrusted compatibility claim. Authorization ignores this value.
    pub now_unix_s: u64,
}

impl TicketAuthRequest {
    fn zeroize_secret(&mut self) {
        self.secret.zeroize();
    }
}

impl Drop for TicketAuthRequest {
    fn drop(&mut self) {
        self.zeroize_secret();
    }
}

impl std::fmt::Debug for TicketAuthRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TicketAuthRequest")
            .field("ticket_id", &self.ticket_id)
            .field("secret", &SECRET_REDACTION)
            .field("client_endpoint", &self.client_endpoint)
            .field("now_unix_s", &self.now_unix_s)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteProductionAttemptBinding {
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureReplayBinding {
    pub source_bundle_blake3: crunch_build::distributed::RemoteFailureDebugDigest,
    pub execution_blake3: crunch_build::distributed::RemoteFailureDebugDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcreteBuildRequest {
    pub request_id: String,
    pub store_prefix: String,
    pub input_refs: Vec<String>,
    #[serde(default = "empty_remote_values", skip_serializing_if = "Vec::is_empty")]
    pub source_input_refs: Vec<String>,
    pub upload_bytes: u64,
    pub build_time_limit_secs: u64,
    pub contains_raw_frontend_eval: bool,
    pub payload: RemoteConcreteBuildPayload,
    pub expected_outputs: Vec<RemoteExpectedOutput>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub production_attempt: Option<RemoteProductionAttemptBinding>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub transfer_policy: Option<RemoteTransferPolicy>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub resource_requirements: Option<RemoteResourceRequirements>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub locality_scope: Option<RemoteLocalityScope>,
    #[serde(default = "default_remote_failure_debug_policy")]
    pub failure_debug_policy: crunch_build::distributed::RemoteFailureDebugPolicy,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub failure_replay: Option<RemoteFailureReplayBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RemoteConcreteBuildPayload {
    Derivation { drv_path: String, drv_json: String },
    Action { action_id: String, spec_json: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExpectedOutput {
    pub name: String,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub logical_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExecutablePlan {
    pub request_id: String,
    pub store_prefix: String,
    pub source: RemoteExecutablePlanSource,
    pub command_args: Vec<String>,
    pub command_env: BTreeMap<String, String>,
    pub system: String,
    pub expected_outputs: Vec<RemoteExpectedOutput>,
    pub plan_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RemoteExecutablePlanSource {
    Derivation {
        declared_drv_path: String,
        computed_drv_path: String,
        drv_name: String,
        drv_digest_blake3: String,
    },
    Action {
        action_id: String,
        schema: String,
        spec_digest_blake3: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExecutionOutcome {
    pub request_id: String,
    pub plan_digest_blake3: String,
    pub output_digest_blake3: String,
    pub output_size_bytes: u64,
    pub outputs: Vec<RemoteExecutionOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExecutionOutput {
    pub name: String,
    pub logical_path: String,
    pub content_digest_blake3: String,
    pub size_bytes: u64,
    pub artifact_attestation_digest_blake3: String,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub path_info: Option<PathInfo>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub nar_payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteProducedOutput {
    pub name: String,
    pub logical_path: String,
    pub content_digest_blake3: String,
    pub size_bytes: u64,
    pub path_info_signing_key_id: String,
    pub artifact_attestation_digest_blake3: String,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub path_info: Option<PathInfo>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub nar_payload_digest_blake3: Option<String>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub nar_payload_size_bytes: Option<u64>,
}

pub trait RemoteBuildExecutor {
    fn execute(
        &self,
        request: &ConcreteBuildRequest,
        plan: &RemoteExecutablePlan,
        input_upload: &RemoteInputUpload,
    ) -> Result<RemoteExecutionOutcome, String>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RemoteFixtureExecutor;

impl RemoteBuildExecutor for RemoteFixtureExecutor {
    fn execute(
        &self,
        _request: &ConcreteBuildRequest,
        plan: &RemoteExecutablePlan,
        input_upload: &RemoteInputUpload,
    ) -> Result<RemoteExecutionOutcome, String> {
        execute_remote_fixture_plan(plan, &input_upload.refs)
    }
}

#[derive(Clone)]
pub struct RemoteLocalBuildExecutor {
    pub endpoint_id: String,
    pub coordinator_state_dir: PathBuf,
    pub state_dir: PathBuf,
    pub output_dir: PathBuf,
    pub store_prefix: String,
    pub keypair: crunch_build::KeyPair,
    pub trusted_keys: Vec<nix_compat::narinfo::VerifyingKey>,
    pub trust_unsigned: bool,
    pub verbose: bool,
}

impl RemoteLocalBuildExecutor {
    fn signing_key_id(&self) -> String {
        self.keypair.verifying_key.name().to_string()
    }
}

impl RemoteBuildExecutor for RemoteLocalBuildExecutor {
    fn execute(
        &self,
        request: &ConcreteBuildRequest,
        plan: &RemoteExecutablePlan,
        input_upload: &RemoteInputUpload,
    ) -> Result<RemoteExecutionOutcome, String> {
        let created_unix_s = crate::unix_time_now_s().map_err(|error| error.to_string())?;
        execute_remote_local_build(self, RemoteLocalBuildExecutionInput {
            request,
            plan,
            input_upload,
            created_unix_s,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RemoteActionSpec {
    pub schema: String,
    pub action_id: String,
    pub builder: String,
    #[serde(default = "default_remote_action_system")]
    pub system: String,
    #[serde(default = "empty_remote_values")]
    pub args: Vec<String>,
    #[serde(default = "empty_remote_map")]
    pub env: BTreeMap<String, String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransportBinding {
    Loopback,
    Stdio,
    SshStdio,
    P2p,
}

impl RemoteTransportBinding {
    fn as_str(self) -> &'static str {
        match self {
            Self::Loopback => "loopback",
            Self::Stdio => "stdio",
            Self::SshStdio => "ssh-stdio",
            Self::P2p => "p2p",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFrameDirection {
    ClientToBuilder,
    BuilderToClient,
}

impl RemoteFrameDirection {
    fn as_str(self) -> &'static str {
        match self {
            Self::ClientToBuilder => "client-to-builder",
            Self::BuilderToClient => "builder-to-client",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteProtocolPhase {
    Open,
    AwaitAuth,
    AwaitAuthOk,
    AwaitBuildRequest,
    AwaitInputManifest,
    AwaitMissingInputs,
    AwaitInputUpload,
    AwaitQueueAdmission,
    Queued,
    Building,
    AwaitOutputTransfer,
    Done,
    Failed,
}

impl RemoteProtocolPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::AwaitAuth => "await-auth",
            Self::AwaitAuthOk => "await-auth-ok",
            Self::AwaitBuildRequest => "await-build-request",
            Self::AwaitInputManifest => "await-input-manifest",
            Self::AwaitMissingInputs => "await-missing-inputs",
            Self::AwaitInputUpload => "await-input-upload",
            Self::AwaitQueueAdmission => "await-queue-admission",
            Self::Queued => "queued",
            Self::Building => "building",
            Self::AwaitOutputTransfer => "await-output-transfer",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFrameKind {
    Hello,
    TraceContext,
    TraceContextAck,
    AuthTicket,
    AuthOk,
    BuildRequest,
    InputManifest,
    MissingInputs,
    InputUpload,
    TransferManifest,
    TransferDemand,
    TransferCredit,
    TransferAcknowledgement,
    TransferComplete,
    BuildQueued,
    BuildStarted,
    BuildFinished,
    OutputTransferArtifact,
    OutputTransferDone,
    Done,
    Error,
}

impl RemoteFrameKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Hello => "hello",
            Self::TraceContext => "trace-context",
            Self::TraceContextAck => "trace-context-ack",
            Self::AuthTicket => "auth-ticket",
            Self::AuthOk => "auth-ok",
            Self::BuildRequest => "build-request",
            Self::InputManifest => "input-manifest",
            Self::MissingInputs => "missing-inputs",
            Self::InputUpload => "input-upload",
            Self::TransferManifest => "transfer-manifest",
            Self::TransferDemand => "transfer-demand",
            Self::TransferCredit => "transfer-credit",
            Self::TransferAcknowledgement => "transfer-acknowledgement",
            Self::TransferComplete => "transfer-complete",
            Self::BuildQueued => "build-queued",
            Self::BuildStarted => "build-started",
            Self::BuildFinished => "build-finished",
            Self::OutputTransferArtifact => "output-transfer-artifact",
            Self::OutputTransferDone => "output-transfer-done",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAuthOk {
    pub builder_signing_keys: Vec<String>,
    pub accepted_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTraceContextAck {
    pub health: RemoteTraceContextHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteInputManifest {
    pub request_id: String,
    pub store_prefix: String,
    pub input_refs: Vec<String>,
    pub closure_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteInputUpload {
    pub request_id: String,
    pub refs: Vec<String>,
    pub byte_count: u64,
    #[serde(default = "empty_remote_values", skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<RemoteInputUploadArtifact>,
    #[serde(default = "remote_false")]
    pub streamed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteInputUploadArtifactKind {
    Nar,
}

/// Compatibility full-NAR fallback artifact. This whole-payload DTO is never
/// evidence for `RemoteTransferMode::Streaming`; production streaming uses the
/// bounded socket/stdio data plane in `crate::remote_transfer`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteInputUploadArtifact {
    pub request_id: String,
    pub input_ref: String,
    pub artifact_kind: RemoteInputUploadArtifactKind,
    pub digest_blake3: String,
    pub size_bytes: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteInputUploadClass {
    Store,
    Source,
    Proof,
    SecretDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteInputUploadPrivacyPolicy {
    pub allowed_classes: BTreeSet<RemoteInputUploadClass>,
    pub max_objects: usize,
    pub max_bytes: u64,
}

impl Default for RemoteInputUploadPrivacyPolicy {
    fn default() -> Self {
        Self {
            allowed_classes: BTreeSet::from([
                RemoteInputUploadClass::Store,
                RemoteInputUploadClass::Source,
                RemoteInputUploadClass::Proof,
            ]),
            max_objects: MAX_REMOTE_INPUT_REFS,
            max_bytes: MAX_REMOTE_UPLOAD_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteInputUploadPrivacySummary {
    pub store_objects: u32,
    pub source_objects: u32,
    pub proof_objects: u32,
    pub secret_descriptor_objects: u32,
    pub total_objects: u32,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSourceUploadReadiness {
    pub identity: String,
    pub source_kind: String,
    pub ready_class: String,
    pub content_blake3: String,
    pub store_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteBuildFinished {
    pub request_id: String,
    pub output_digest_blake3: String,
    pub builder_signing_key_id: String,
    pub store_prefix: String,
    pub outputs: Vec<RemoteProducedOutput>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteOutputTransferArtifactKind {
    PathInfoJson,
    Nar,
}

/// Compatibility full-NAR/PathInfo fallback artifact. Streaming reports may
/// only be built from a completed `RemoteTransferShellReport`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteOutputTransferArtifact {
    pub request_id: String,
    pub output_name: String,
    pub logical_path: String,
    pub artifact_kind: RemoteOutputTransferArtifactKind,
    pub digest_blake3: String,
    pub size_bytes: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedRemoteOutputTransferArtifact {
    output_name: String,
    logical_path: String,
    artifact_kind: RemoteOutputTransferArtifactKind,
    digest_blake3: String,
    size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteProtocolErrorFrame {
    pub phase: RemoteFailurePhase,
    pub retry_class: RemoteRetryClass,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferManifestFrame {
    pub direction: crate::remote_transfer::RemoteTransferDirection,
    pub manifest: RemoteTransferManifest,
    pub manifest_digest_blake3: RemoteTransferDigest,
    pub actual_mode: RemoteTransferMode,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferDemandFrame {
    pub direction: crate::remote_transfer::RemoteTransferDirection,
    pub demand: RemoteTransferDemand,
    pub sender_state: RemoteTransferCreditState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferCreditFrame {
    pub direction: crate::remote_transfer::RemoteTransferDirection,
    pub missing: RemoteTransferChunkDemand,
    pub grant: RemoteTransferCreditGrant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RemoteFrame {
    Hello {
        hello: RemoteHello,
    },
    TraceContext {
        context: Option<RemoteTraceContext>,
    },
    TraceContextAck {
        acknowledgement: RemoteTraceContextAck,
    },
    AuthTicket {
        auth: TicketAuthRequest,
    },
    AuthOk {
        auth: RemoteAuthOk,
    },
    BuildRequest {
        request: ConcreteBuildRequest,
    },
    InputManifest {
        manifest: RemoteInputManifest,
    },
    MissingInputs {
        request_id: String,
        refs: Vec<String>,
    },
    InputUpload {
        upload: RemoteInputUpload,
    },
    TransferManifest {
        transfer: RemoteTransferManifestFrame,
    },
    TransferDemand {
        transfer: RemoteTransferDemandFrame,
    },
    TransferCredit {
        transfer: RemoteTransferCreditFrame,
    },
    TransferAcknowledgement {
        direction: crate::remote_transfer::RemoteTransferDirection,
        acknowledgement: RemoteTransferAcknowledgement,
    },
    TransferComplete {
        direction: crate::remote_transfer::RemoteTransferDirection,
        report: crate::remote_transfer::RemoteTransferShellReport,
    },
    BuildQueued {
        request_id: String,
        session_id: String,
    },
    BuildStarted {
        request_id: String,
    },
    BuildFinished {
        result: RemoteBuildFinished,
    },
    OutputTransferArtifact {
        artifact: RemoteOutputTransferArtifact,
    },
    OutputTransferDone {
        report: RemoteTransferReport,
    },
    Done {
        request_id: String,
    },
    Error {
        error: RemoteProtocolErrorFrame,
    },
}

impl RemoteFrame {
    pub fn kind(&self) -> RemoteFrameKind {
        match self {
            Self::Hello { .. } => RemoteFrameKind::Hello,
            Self::TraceContext { .. } => RemoteFrameKind::TraceContext,
            Self::TraceContextAck { .. } => RemoteFrameKind::TraceContextAck,
            Self::AuthTicket { .. } => RemoteFrameKind::AuthTicket,
            Self::AuthOk { .. } => RemoteFrameKind::AuthOk,
            Self::BuildRequest { .. } => RemoteFrameKind::BuildRequest,
            Self::InputManifest { .. } => RemoteFrameKind::InputManifest,
            Self::MissingInputs { .. } => RemoteFrameKind::MissingInputs,
            Self::InputUpload { .. } => RemoteFrameKind::InputUpload,
            Self::TransferManifest { .. } => RemoteFrameKind::TransferManifest,
            Self::TransferDemand { .. } => RemoteFrameKind::TransferDemand,
            Self::TransferCredit { .. } => RemoteFrameKind::TransferCredit,
            Self::TransferAcknowledgement { .. } => RemoteFrameKind::TransferAcknowledgement,
            Self::TransferComplete { .. } => RemoteFrameKind::TransferComplete,
            Self::BuildQueued { .. } => RemoteFrameKind::BuildQueued,
            Self::BuildStarted { .. } => RemoteFrameKind::BuildStarted,
            Self::BuildFinished { .. } => RemoteFrameKind::BuildFinished,
            Self::OutputTransferArtifact { .. } => RemoteFrameKind::OutputTransferArtifact,
            Self::OutputTransferDone { .. } => RemoteFrameKind::OutputTransferDone,
            Self::Done { .. } => RemoteFrameKind::Done,
            Self::Error { .. } => RemoteFrameKind::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransferMode {
    Delta,
    Full,
    /// Negotiated streaming capability label. Production support is claimed
    /// only when the bounded data-plane path and current interruption evidence
    /// accompany this label.
    Streaming,
}

impl RemoteTransferMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Delta => "delta",
            Self::Full => "full",
            Self::Streaming => "streaming",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferCapabilities {
    pub delta: bool,
    pub full: bool,
    pub streaming: bool,
    pub simulate_delta_failure: bool,
}

impl RemoteTransferCapabilities {
    pub fn delta_and_full() -> Self {
        Self {
            delta: true,
            full: true,
            streaming: false,
            simulate_delta_failure: false,
        }
    }

    pub fn full_only() -> Self {
        Self {
            delta: false,
            full: true,
            streaming: false,
            simulate_delta_failure: false,
        }
    }

    pub fn with_streaming(mut self) -> Self {
        self.streaming = true;
        self
    }

    fn as_capability_labels(self) -> Vec<String> {
        let mut labels = Vec::new();
        if self.delta {
            labels.push(RemoteTransferMode::Delta.as_str().to_string());
        }
        if self.full {
            labels.push(RemoteTransferMode::Full.as_str().to_string());
        }
        if self.streaming {
            labels.push(RemoteTransferMode::Streaming.as_str().to_string());
        }
        labels
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferReport {
    pub mode: RemoteTransferMode,
    pub mode_label: String,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub fallback_reason: Option<String>,
    pub verified_builder_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailurePhase {
    TransportSetup,
    Authentication,
    RequestValidation,
    InputSync,
    Queue,
    BuildExecution,
    OutputImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteRetryClass {
    Retryable,
    Terminal,
    BuildOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteFailureClassification {
    pub phase: RemoteFailurePhase,
    pub retry_class: RemoteRetryClass,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteLoopbackBuilder {
    pub endpoint_id: String,
    pub store_prefix: String,
    pub supported_capabilities: Vec<String>,
    pub present_input_refs: Vec<String>,
    pub signing_key_id: String,
    pub transfer_capabilities: RemoteTransferCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteLoopbackClient {
    pub session_id: String,
    pub hello: RemoteHello,
    pub auth: TicketAuthRequest,
    pub request: ConcreteBuildRequest,
    pub input_manifest: RemoteInputManifest,
    pub uploaded_input_refs: Vec<String>,
    pub trusted_output_keys: Vec<String>,
    pub transfer_capabilities: RemoteTransferCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSessionLeasePlan {
    pub session_id: String,
    pub leased_refs: Vec<String>,
    pub release_when_done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteLoopbackSessionReport {
    pub schema: String,
    pub binding: RemoteTransportBinding,
    pub session_id: String,
    pub endpoint_id: String,
    pub accepted_capabilities: Vec<String>,
    pub missing_input_refs: Vec<String>,
    pub uploaded_input_refs: Vec<String>,
    pub output_digest_blake3: String,
    pub transfer: RemoteTransferReport,
    pub lease_plan: RemoteSessionLeasePlan,
    pub ticket_uses_remaining: u32,
    pub phases: Vec<RemoteProtocolPhase>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteStdioChildOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status_success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteStreamingOutputReceipt {
    pub manifest: RemoteTransferManifest,
    pub manifest_digest_blake3: RemoteTransferDigest,
    pub receiver_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteStdioTranscript {
    pub binding: RemoteTransportBinding,
    pub frames: Vec<RemoteFrame>,
    pub stderr_summary: String,
    #[serde(default = "empty_remote_telemetry_buffer")]
    pub telemetry: RemoteTelemetryBuffer,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub streaming_output: Option<RemoteStreamingOutputReceipt>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub trace_context_health: Option<RemoteTraceContextHealth>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteBuilderFrameResponse {
    pub response_frames: Vec<RemoteFrame>,
    pub missing_input_refs: Vec<String>,
    pub execution_plan_digest_blake3: String,
    pub output_digest_blake3: String,
    pub outputs: Vec<RemoteProducedOutput>,
    pub transfer_artifacts: Vec<RemoteOutputTransferArtifact>,
    pub transfer: RemoteTransferReport,
    pub ticket_uses_remaining: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteOutputTrustBasis {
    pub key_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_material_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteOutputAdmissionReport {
    pub request_id: String,
    pub output_digest_blake3: String,
    pub builder_signing_key_id: String,
    pub trust_basis: RemoteOutputTrustBasis,
    pub store_prefix: String,
    pub outputs: Vec<RemoteProducedOutput>,
    pub transfer_artifacts: Vec<RemoteOutputTransferArtifact>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub streamed_manifest: Option<RemoteTransferManifest>,
    pub transfer: RemoteTransferReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteOutputImportAction {
    pub output_name: String,
    pub logical_path: String,
    pub store_path: StorePath<String>,
    pub path_info: PathInfo,
    pub final_node: snix_castore::Node,
    pub path_info_signing_key_id: String,
    pub artifact_attestation_digest_blake3: String,
    pub nar_payload: Option<Vec<u8>>,
    pub nar_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteImportedOutput {
    pub name: String,
    pub logical_path: String,
    pub path_info_signing_key_id: String,
    pub artifact_attestation_digest_blake3: String,
    pub artifact_attestation_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteOutputImportReport {
    pub request_id: String,
    pub store_prefix: String,
    pub outputs: Vec<RemoteImportedOutput>,
    pub transfer: RemoteTransferReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteStdioExchangeReport {
    pub binding: RemoteTransportBinding,
    pub frames: Vec<RemoteFrame>,
    pub stderr_summary: String,
    pub admission: RemoteOutputAdmissionReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteProductionTransferClient {
    pub state_dir: PathBuf,
    pub transfer_policy: RemoteTransferPolicy,
    pub telemetry_policy: RemoteTelemetryPolicy,
    pub trusted_output_keys: Vec<String>,
    pub input_transfer: Option<crate::remote_transfer::PreparedRemoteTransfer>,
    pub interrupt_after_input_chunks: Option<u32>,
    pub interrupt_after_output_chunks: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteStdioCommand {
    pub binding: RemoteTransportBinding,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub input_frames: Vec<RemoteFrame>,
    pub timeout_secs: u64,
    pub production_transfer: Option<RemoteProductionTransferClient>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct RemoteTicketCredential {
    pub ticket_id: String,
    pub secret: String,
}

impl RemoteTicketCredential {
    fn zeroize_secret(&mut self) {
        self.secret.zeroize();
    }
}

impl Drop for RemoteTicketCredential {
    fn drop(&mut self) {
        self.zeroize_secret();
    }
}

impl std::fmt::Debug for RemoteTicketCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RemoteTicketCredential")
            .field("ticket_id", &self.ticket_id)
            .field("secret", &SECRET_REDACTION)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteStdioBuilderCommand {
    pub endpoint_id: String,
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSshStdioPlanRequest {
    pub endpoint_id: String,
    pub ssh_program: PathBuf,
    pub destination: String,
    pub remote_program: String,
    pub remote_args: Vec<String>,
}

pub fn plan_ssh_stdio_builder_command(request: RemoteSshStdioPlanRequest) -> Result<RemoteStdioBuilderCommand, String> {
    if request.endpoint_id.is_empty() {
        return Err("ssh-stdio-endpoint-empty".to_string());
    }
    if request.ssh_program.as_os_str().is_empty() {
        return Err("ssh-stdio-program-empty".to_string());
    }
    if request.destination.is_empty() {
        return Err("ssh-stdio-destination-empty".to_string());
    }
    if request.remote_program.is_empty() {
        return Err("ssh-stdio-remote-program-empty".to_string());
    }
    if request.remote_args.len() > MAX_REMOTE_EXECUTOR_ARGS {
        return Err(format!("ssh-stdio-remote-arg-count-exceeds-{MAX_REMOTE_EXECUTOR_ARGS}"));
    }
    let mut args = Vec::with_capacity(request.remote_args.len().saturating_add(SSH_STDIO_FIXED_ARG_COUNT));
    args.push(request.destination);
    args.push(request.remote_program);
    args.extend(request.remote_args);
    let command = RemoteStdioBuilderCommand {
        endpoint_id: request.endpoint_id,
        program: request.ssh_program,
        args,
    };
    debug_assert!(!command.endpoint_id.is_empty());
    debug_assert!(command.args.len() >= SSH_STDIO_FIXED_ARG_COUNT);
    Ok(command)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteClientBuildOptions {
    pub store_prefix: String,
    pub ticket: RemoteTicketCredential,
    pub builder: RemoteStdioBuilderCommand,
    pub trusted_output_keys: Vec<String>,
    pub now_unix_s: u64,
    pub build_time_limit_secs: u64,
    pub client_endpoint: Option<String>,
    pub transfer_capabilities: RemoteTransferCapabilities,
}

#[derive(Debug, Clone)]
pub struct RemoteClientDerivationInput {
    pub label: String,
    pub drv_path: StorePath<String>,
    pub crunch_derivation: crunch_glue::CrunchDerivation,
    pub nix_derivation: nix_compat::derivation::Derivation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRootPriorityCandidate {
    pub input_index: u32,
    pub root_identity_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRootPriorityPlan {
    pub ordered_input_indices: Vec<u32>,
    pub evidence: crunch_build::scheduling::PriorityDecisionEvidence,
}

pub fn remote_root_priority_identity(
    label: &str,
    drv_path: &StorePath<String>,
    store_prefix: &str,
) -> Result<String, String> {
    if label.is_empty() || store_prefix.is_empty() {
        return Err("remote-root-priority-identity-input-empty".to_string());
    }
    let identity = remote_client_request_id(label, drv_path, store_prefix);
    assert!(is_blake3_hex_digest(&identity));
    assert_eq!(identity.len(), BLAKE3_HEX_LENGTH_CHARS);
    Ok(identity)
}

pub fn plan_remote_root_priority(
    policy: &crunch_build::scheduling::SchedulingPolicy,
    candidates: &[RemoteRootPriorityCandidate],
) -> Result<RemoteRootPriorityPlan, String> {
    if candidates.is_empty() {
        return Err("remote-root-priority-candidates-empty".to_string());
    }
    let candidate_count_max = usize::try_from(crunch_build::scheduling::MAX_READY_GOALS)
        .map_err(|_| "remote-root-priority-candidate-limit-overflow".to_string())?;
    if candidates.len() > candidate_count_max {
        return Err("remote-root-priority-candidate-limit-exceeded".to_string());
    }
    let candidate_count =
        u32::try_from(candidates.len()).map_err(|_| "remote-root-priority-candidate-count-overflow".to_string())?;
    let mut input_indices = BTreeSet::new();
    let mut root_identities = BTreeSet::new();
    let mut index_entries = Vec::with_capacity(candidates.len());
    let mut ready_goals = Vec::with_capacity(candidates.len());
    let mut pressure_entries = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !input_indices.insert(candidate.input_index) {
            return Err("remote-root-priority-input-index-duplicate".to_string());
        }
        if !is_blake3_hex_digest(&candidate.root_identity_blake3) {
            return Err("remote-root-priority-identity-invalid".to_string());
        }
        if !root_identities.insert(candidate.root_identity_blake3.clone()) {
            return Err("remote-root-priority-identity-duplicate".to_string());
        }
        index_entries.push((candidate.root_identity_blake3.clone(), candidate.input_index));
        ready_goals.push(crunch_build::scheduling::ReadyGoalFacts::ordinary(candidate.root_identity_blake3.clone(), 0));
        pressure_entries.push((candidate.root_identity_blake3.clone(), crunch_build::scheduling::KnownGraphPressure {
            known_critical_path_nodes: 0,
            known_critical_path_work_units: 0,
            blocked_root_count: 0,
            blocked_root_count_saturated: false,
        }));
    }
    let index_by_identity = index_entries.into_iter().collect::<BTreeMap<_, _>>();
    let pressures = pressure_entries.into_iter().collect::<BTreeMap<_, _>>();
    let ranked = crunch_build::scheduling::rank_ready_goals(policy, 0, &ready_goals, &pressures)
        .map_err(|error| error.to_string())?;
    let evidence = crunch_build::scheduling::priority_decision_evidence(
        policy,
        0,
        &ranked,
        crunch_build::scheduling::HistoryBasis::StructuralFallbackMissing,
        None,
    )
    .map_err(|error| error.to_string())?;
    let ordered_input_indices = ranked
        .iter()
        .map(|ranked_goal| {
            index_by_identity
                .get(&ranked_goal.goal_key)
                .copied()
                .ok_or_else(|| "remote-root-priority-selection-missing".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(ordered_input_indices.len(), candidates.len());
    assert_eq!(evidence.competing_goal_count, candidate_count);
    Ok(RemoteRootPriorityPlan {
        ordered_input_indices,
        evidence,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteClientDispatchPlan {
    pub label: String,
    pub command: RemoteStdioCommand,
    pub client: RemoteLoopbackClient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTelemetryBufferHealth {
    pub accepted_events: u64,
    pub dropped_events: u64,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub last_drop_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptObservabilityHealth {
    pub telemetry: RemoteTelemetryBufferHealth,
    pub exporters: RemoteTelemetryExportReport,
    pub trace_context: RemoteTraceContextHealth,
    pub immutable_log: RemoteTelemetryAdapterHealth,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptObservabilityReport {
    pub events: Vec<RemoteTelemetryEvent>,
    pub health: RemoteAttemptObservabilityHealth,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub immutable_log: Option<RemoteAttemptLogControlSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteClientImportedBuild {
    pub label: String,
    pub request_id: String,
    pub imported: RemoteOutputImportReport,
    pub observability: RemoteAttemptObservabilityReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteClientBuildReport {
    pub schema: String,
    pub builder: String,
    pub store_prefix: String,
    pub priority_decisions: Vec<crunch_build::scheduling::PriorityDecisionEvidence>,
    pub imported: Vec<RemoteClientImportedBuild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketDecision {
    Authorized,
    Reject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolDecision {
    Proceed(AcceptedHello),
    Reject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputTrustDecision {
    Accept {
        key_id: String,
        trust_basis: RemoteOutputTrustBasis,
    },
    Reject(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteCoordinatorJobPhase {
    Queued,
    Running,
    Finished,
    Lost,
}

impl RemoteCoordinatorJobPhase {
    fn is_live(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWorkerResumeSummary {
    pub job_id: RemoteJobId,
    #[serde(default = "legacy_missing_attempt_id")]
    pub attempt_id: RemoteAttemptId,
    #[serde(default = "legacy_missing_fence_generation")]
    pub fence_generation: RemoteFenceGeneration,
    pub normalized_build_key: String,
    pub phase: RemoteCoordinatorJobPhase,
    pub result_available: bool,
    pub log_next_cursor: u64,
}

fn legacy_missing_attempt_id() -> RemoteAttemptId {
    // The external validated newtype has no infallible constructor for this checked constant.
    #[allow(tigerstyle::no_unwrap)]
    RemoteAttemptId::new(LEGACY_MISSING_ATTEMPT_ID).expect("legacy attempt sentinel is a bounded identity")
}

fn legacy_missing_fence_generation() -> RemoteFenceGeneration {
    RemoteFenceGeneration::INITIAL
}

fn legacy_worker_generation() -> u64 {
    RemoteFenceGeneration::INITIAL.get()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWorkerRegistration {
    pub endpoint_id: String,
    pub protocol_version: u32,
    /// Monotonic worker incarnation and locality-fact generation. Workers must
    /// advance it before restart or any destructive content-cache transition.
    #[serde(default = "legacy_worker_generation")]
    pub worker_generation: u64,
    pub systems: Vec<String>,
    pub feature_labels: Vec<String>,
    pub sandbox_modes: Vec<String>,
    pub network_modes: Vec<String>,
    pub logical_store_prefixes: Vec<String>,
    pub concurrency: u32,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub resource_inventory: Option<RemoteWorkerResourceInventory>,
    pub output_signing_key_ids: Vec<String>,
    pub resumable_jobs: Vec<RemoteWorkerResumeSummary>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub workspace_policy: Option<crate::remote_farm_config::RemoteWorkspacePolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorBuildRequest {
    pub request: ConcreteBuildRequest,
    pub required_system: String,
    pub required_features: Vec<String>,
    pub required_sandbox_mode: String,
    pub required_network_mode: String,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub resource_requirements: Option<RemoteResourceRequirements>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub locality_scope: Option<RemoteLocalityScope>,
    pub trusted_output_keys: Vec<String>,
    pub live_output_claims: Vec<String>,
    pub wait_for_worker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteFailureDebugStatus {
    pub bundle_ref: String,
    pub capture_outcome_code: String,
    pub cleanup_status_code: String,
    pub immutable_log_available: bool,
    pub non_claim: String,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub worker_bundle_ref: Option<String>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub replay_attempt_identity: Option<String>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub replay_comparison_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorJobSummary {
    pub job_id: RemoteJobId,
    pub normalized_build_key: String,
    pub assigned_worker_endpoint_id: Option<String>,
    pub phase: RemoteCoordinatorJobPhase,
    pub live_output_claims: Vec<String>,
    pub result_available: bool,
    pub lost_phase: Option<RemoteFailurePhase>,
    #[serde(default = "absent_remote_value")]
    pub immutable_log: Option<RemoteAttemptLogControlSummary>,
    #[serde(default = "absent_remote_value")]
    pub observability_health: Option<RemoteAttemptObservabilityHealth>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub failure_debug: Option<RemoteFailureDebugStatus>,
    pub short_error: Option<String>,
    #[serde(default = "absent_remote_value")]
    pub current_attempt: Option<RemoteAttemptState>,
    #[serde(default = "absent_remote_value")]
    pub transfer_checkpoint: Option<u64>,
    #[serde(default = "remote_zero_u64")]
    pub transferred_bytes: u64,
    #[serde(default = "remote_false")]
    pub output_admission_completed: bool,
    #[serde(default = "absent_remote_value")]
    pub last_attempt_reason_code: Option<RemoteAttemptReasonCode>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub resource_requirements: Option<RemoteResourceRequirements>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub resource_lease_id_blake3: Option<String>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub resource_fit: Option<crunch_build::ResourceFitClass>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub locality: Option<RemoteVerifiedLocalitySummary>,
}

// r[impl external_batch_dispatchers.fenced_lifecycle]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalBatchCoordinatorRecord {
    pub submit_operation: ExternalBatchOperation,
    pub last_operation: ExternalBatchOperation,
    pub last_response: ExternalBatchOperationResponse,
    pub reconcile_attempts: u32,
    pub worker_registered: bool,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub coordinator_job_id: Option<RemoteJobId>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub resource_lease_id_blake3: Option<String>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub output_admission_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExternalBatchAttemptStatus {
    pub dispatch_id_blake3: String,
    pub adapter_instance_id: String,
    pub dispatcher_profile_ref_blake3: String,
    pub provider_class: String,
    pub dispatcher_generation: u64,
    pub allocation_job_id: RemoteJobId,
    pub allocation_attempt_id: RemoteAttemptId,
    pub allocation_fence_generation: RemoteFenceGeneration,
    pub external_job_id: Option<String>,
    pub state: ExternalBatchJobState,
    pub resources: crunch_build::distributed::ExternalBatchResourceProjection,
    pub worker_endpoint_id: String,
    pub worker_registered: bool,
    pub coordinator_job_id: Option<RemoteJobId>,
    pub resource_lease_id_blake3: Option<String>,
    pub output_admission_completed: bool,
    pub reconcile_attempts: u32,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExternalBatchPlanReport {
    pub schema: &'static str,
    pub dispatch_id_blake3: String,
    pub dispatcher_profile_ref_blake3: String,
    pub provider_class: String,
    pub worker_bootstrap_ref_blake3: String,
    pub semantic_capability_class: String,
    pub resources: crunch_build::distributed::ExternalBatchResourceProjection,
    pub startup_timeout_secs: u64,
    pub terminal_timeout_secs: u64,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExternalBatchCompositionEvidence {
    pub schema: &'static str,
    pub dispatch_id_blake3: String,
    pub dispatcher_profile_ref_blake3: String,
    pub provider_class: String,
    pub external_job_id: String,
    pub queue_state: ExternalBatchJobState,
    pub resources: crunch_build::distributed::ExternalBatchResourceProjection,
    pub worker_endpoint_id: String,
    pub worker_registered: bool,
    pub coordinator_job_id: RemoteJobId,
    pub resource_lease_id_blake3: Option<String>,
    pub transfer_mode: RemoteTransferMode,
    pub output_admission_digest_blake3: String,
    pub output_trust_key_id: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorState {
    pub workers: BTreeMap<String, RemoteWorkerRegistration>,
    pub jobs: BTreeMap<RemoteJobId, RemoteCoordinatorJobSummary>,
    pub live_output_claims: BTreeMap<String, String>,
    #[serde(default = "empty_remote_map")]
    pub resource_leases: BTreeMap<String, RemoteResourceLease>,
    #[serde(default = "empty_remote_map")]
    pub verified_locality_observations: BTreeMap<String, RemoteVerifiedLocalitySummary>,
    #[serde(default = "empty_remote_map")]
    pub workspace_registrations: BTreeMap<String, crate::remote_farm_config::RemoteWorkspacePolicy>,
    #[serde(default = "empty_remote_map")]
    pub workspace_leases: BTreeMap<String, crunch_build::WorkspaceLeaseRecord>,
    #[serde(default = "empty_remote_map")]
    pub external_batch_attempts: BTreeMap<String, ExternalBatchCoordinatorRecord>,
    #[serde(default = "absent_remote_value", skip_serializing_if = "Option::is_none")]
    pub legacy_log_migration: Option<RemoteLegacyLogMigrationSummary>,
    /// When set, mutations auto-save to this directory for durability.
    #[serde(skip)]
    pub state_dir: Option<PathBuf>,
    #[serde(skip)]
    allow_volatile_test_state: bool,
}

// The test-only volatile flag intentionally differs from the derived non-test default.
#[allow(clippy::derivable_impls)]
impl Default for RemoteCoordinatorState {
    fn default() -> Self {
        Self {
            workers: BTreeMap::new(),
            jobs: BTreeMap::new(),
            live_output_claims: BTreeMap::new(),
            resource_leases: BTreeMap::new(),
            verified_locality_observations: BTreeMap::new(),
            workspace_registrations: BTreeMap::new(),
            workspace_leases: BTreeMap::new(),
            external_batch_attempts: BTreeMap::new(),
            legacy_log_migration: None,
            state_dir: None,
            allow_volatile_test_state: cfg!(test),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "kebab-case")]
pub enum RemoteCoordinatorDispatchDecision {
    Dispatch {
        worker_endpoint_id: String,
        job_id: RemoteJobId,
        attempt_id: RemoteAttemptId,
        fence_generation: RemoteFenceGeneration,
        normalized_build_key: String,
        resource_fit: crunch_build::ResourceFitClass,
        locality: Option<RemoteVerifiedLocalitySummary>,
    },
    AttachExisting {
        job_id: RemoteJobId,
        normalized_build_key: String,
    },
    RedeliverResult {
        job_id: RemoteJobId,
        normalized_build_key: String,
    },
    Pending {
        reason: String,
    },
    Reject {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorAttemptApplyResult {
    pub disposition: RemoteAttemptApplyDisposition,
    pub reason_code: RemoteAttemptReasonCode,
    pub output_admission_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFencedOutputAdmissionDecision {
    NoAdmission {
        attempt: RemoteCoordinatorAttemptApplyResult,
    },
    Admit {
        attempt: RemoteCoordinatorAttemptApplyResult,
        admission: Box<RemoteOutputAdmissionReport>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAttemptLogControlSummary {
    pub scope: RemoteAttemptLogScope,
    pub retention_policy: RemoteLogRetentionPolicy,
    pub retained_start_cursor: u64,
    pub next_cursor: u64,
    pub retained_record_count: u32,
    pub retained_payload_bytes: u64,
    pub head_record_blake3: Option<RemoteAttemptLogDigest>,
    pub head_segment_blake3: Option<RemoteAttemptLogDigest>,
    pub manifest_blake3: RemoteAttemptLogDigest,
    pub truncation_anchor_blake3: Option<RemoteAttemptLogDigest>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteLegacyLogMigrationSummary {
    pub legacy_job_count: u32,
    pub legacy_chunk_count: u32,
    pub legacy_payload_bytes: u64,
    pub count_saturated: bool,
    pub classification: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteLogRetentionPolicy {
    pub max_chunks: usize,
    pub max_bytes: u64,
}

impl Default for RemoteLogRetentionPolicy {
    fn default() -> Self {
        Self {
            max_chunks: MAX_REMOTE_LOG_CHUNKS,
            max_bytes: MAX_REMOTE_LOG_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorJobStatus {
    pub job_id: RemoteJobId,
    pub phase: RemoteCoordinatorJobPhase,
    pub worker_endpoint_id: Option<String>,
    pub short_error: Option<String>,
    pub immutable_log: Option<RemoteAttemptLogControlSummary>,
    pub observability_health: Option<RemoteAttemptObservabilityHealth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_debug: Option<RemoteFailureDebugStatus>,
    pub attempt_id: Option<RemoteAttemptId>,
    pub fence_generation: Option<RemoteFenceGeneration>,
    pub attempt_phase: Option<RemoteAttemptPhase>,
    pub attempt_reason_code: Option<RemoteAttemptReasonCode>,
    pub resource_requirements: Option<RemoteResourceRequirements>,
    pub resource_lease_id_blake3: Option<String>,
    pub resource_fit: Option<crunch_build::ResourceFitClass>,
    pub locality: Option<RemoteVerifiedLocalitySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteWorkerStatus {
    pub endpoint_id: String,
    pub worker_generation: u64,
    pub systems: Vec<String>,
    pub feature_labels: Vec<String>,
    pub sandbox_modes: Vec<String>,
    pub network_modes: Vec<String>,
    pub logical_store_prefixes: Vec<String>,
    pub concurrency: u32,
    pub resource_inventory: Option<RemoteWorkerResourceInventory>,
    pub resource_remaining: Option<RemoteResourceAvailability>,
    pub output_signing_key_count: u32,
    pub resumable_job_count: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workspace_modes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_authority_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteFailureStatus {
    pub job_id: RemoteJobId,
    pub phase: RemoteCoordinatorJobPhase,
    pub lost_phase: Option<RemoteFailurePhase>,
    pub retry_class: RemoteRetryClass,
    pub short_error: Option<String>,
    pub attempt_reason_code: Option<RemoteAttemptReasonCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_debug: Option<RemoteFailureDebugStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteLogCursorStatus {
    pub job_id: RemoteJobId,
    pub attempt_id: Option<RemoteAttemptId>,
    pub fence_generation: Option<RemoteFenceGeneration>,
    pub retained_start_cursor: u64,
    pub next_cursor: u64,
    pub retained_record_count: u32,
    pub retained_payload_bytes: u64,
    pub head_record_blake3: Option<RemoteAttemptLogDigest>,
    pub head_segment_blake3: Option<RemoteAttemptLogDigest>,
    pub manifest_blake3: Option<RemoteAttemptLogDigest>,
    pub truncation_anchor_blake3: Option<RemoteAttemptLogDigest>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteCoordinatorStatusSnapshot {
    pub endpoint_id: String,
    pub configured_concurrency: u32,
    pub worker_count: u32,
    pub workers: Vec<RemoteWorkerStatus>,
    pub queued_jobs: Vec<RemoteCoordinatorJobStatus>,
    pub active_jobs: Vec<RemoteCoordinatorJobStatus>,
    pub recent_jobs: Vec<RemoteCoordinatorJobStatus>,
    pub recent_failures: Vec<RemoteFailureStatus>,
    pub log_cursors: Vec<RemoteLogCursorStatus>,
    pub attempt_reason_codes: Vec<RemoteAttemptReasonCode>,
    pub resource_leases: Vec<RemoteResourceLease>,
    pub external_batch_attempts: Vec<ExternalBatchAttemptStatus>,
    pub tickets: Vec<RemoteTicketView>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteBuildOutputObservability {
    pub output_name: String,
    pub logical_path: String,
    pub artifact_attestation_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteBuildObservabilityReport {
    pub selected_route: String,
    pub rejected_route_reasons: Vec<String>,
    pub endpoint_id: Option<String>,
    pub attempt_reason_codes: Vec<String>,
    pub upload_summary: RemoteInputUploadPrivacySummary,
    pub resource_lease: Option<RemoteResourceLease>,
    pub locality: Option<RemoteVerifiedLocalitySummary>,
    pub transfer: RemoteTransferReport,
    pub trust_basis: RemoteOutputTrustBasis,
    pub outputs: Vec<RemoteBuildOutputObservability>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteOperatorE2eRailPhases {
    pub route: String,
    pub handshake: String,
    pub input_sync: String,
    pub execution: String,
    pub output_admission: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteOperatorE2eRailReport {
    pub schema: &'static str,
    pub selected_route: String,
    pub selected_reason_code: String,
    pub endpoint_id: Option<String>,
    pub phases: RemoteOperatorE2eRailPhases,
    pub upload_summary: RemoteInputUploadPrivacySummary,
    pub transfer: RemoteTransferReport,
    pub trust_basis: RemoteOutputTrustBasis,
    pub artifact_attestation_paths: Vec<String>,
    pub status: RemoteCoordinatorStatusSnapshot,
    pub build_report: RemoteBuildObservabilityReport,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteReconnectDecision {
    SameSession,
    NewSession,
    BackoffRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteLeaseReleaseDecision {
    Retain,
    Release,
}

pub fn validate_hello(
    hello: &RemoteHello,
    expected_endpoint_id: &str,
    supported_capabilities: &[String],
) -> ProtocolDecision {
    if hello.alpn != REMOTE_PROTOCOL_ALPN {
        return ProtocolDecision::Reject(format!("unsupported ALPN {}", hello.alpn));
    }
    if hello.version != REMOTE_PROTOCOL_VERSION {
        return ProtocolDecision::Reject(format!("unsupported protocol version {}", hello.version));
    }
    if hello.endpoint_id != expected_endpoint_id {
        return ProtocolDecision::Reject("endpoint identity mismatch".to_string());
    }
    if hello.capabilities.len() > MAX_REMOTE_CAPABILITIES {
        return ProtocolDecision::Reject(format!("capability count exceeds {MAX_REMOTE_CAPABILITIES}"));
    }
    if let Some(policy) = hello.workspace_policy.as_ref()
        && let Err(reason) = crate::remote_farm_config::validate_remote_workspace_policy(policy)
    {
        return ProtocolDecision::Reject(format!("invalid workspace registration: {reason}"));
    }
    let accepted_capabilities = hello
        .capabilities
        .iter()
        .filter(|capability| supported_capabilities.contains(capability))
        .cloned()
        .collect::<Vec<_>>();
    debug_assert!(accepted_capabilities.len() <= hello.capabilities.len());
    debug_assert!(accepted_capabilities.iter().all(|capability| supported_capabilities.contains(capability)));
    ProtocolDecision::Proceed(AcceptedHello {
        endpoint_id: hello.endpoint_id.clone(),
        accepted_capabilities,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteTicketAuthFacts<'a> {
    pub server_now_unix_s: u64,
    pub authenticated_client_endpoint: Option<&'a str>,
}

pub fn authorize_ticket(
    ticket: &RemoteTicket,
    request: &TicketAuthRequest,
    service_facts: RemoteTicketAuthFacts<'_>,
) -> TicketDecision {
    let facts = TicketPolicyFacts {
        ticket_id: &request.ticket_id,
        client_endpoint: service_facts.authenticated_client_endpoint,
        now_unix_s: service_facts.server_now_unix_s,
    };
    match crate::remote_credentials::authorize_ticket(ticket, &request.secret, &facts) {
        TicketAuthorization::Authorized => TicketDecision::Authorized,
        TicketAuthorization::Rejected(reason) => TicketDecision::Reject(reason.to_string()),
    }
}

pub fn validate_concrete_request(request: &ConcreteBuildRequest, ticket: &RemoteTicket) -> Result<(), String> {
    let admitted_ticket = crate::remote_credentials::admit_ticket_policy(ticket)?;
    plan_remote_executable_request(request)?;
    // Reject raw frontend eval requests — CI systems must submit
    // concrete build requests, not eval scheduling fields.
    // r[impl remote_builds.production_ci_build_separation]
    if request.contains_raw_frontend_eval {
        return Err("contains-raw-frontend-eval".to_string());
    }
    if request.input_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"));
    }
    validate_source_input_refs(request)?;
    if request.upload_bytes > admitted_ticket.upload_byte_limit().bytes()
        || request.upload_bytes > MAX_REMOTE_UPLOAD_BYTES
    {
        return Err("upload-byte-limit-exceeded".to_string());
    }
    if request.build_time_limit_secs > admitted_ticket.build_time_limit().seconds()
        || request.build_time_limit_secs > MAX_REMOTE_BUILD_TIME_SECS
    {
        return Err("build-time-limit-exceeded".to_string());
    }
    if let Some(requirements) = &request.resource_requirements {
        canonical_remote_resource_requirements(requirements).map_err(|reason| reason.as_str().to_string())?;
    }
    if let Some(scope) = &request.locality_scope
        && (!is_blake3_hex_digest(&scope.manifest_digest_blake3) || !is_blake3_hex_digest(&scope.policy_digest_blake3))
    {
        return Err("remote-locality-scope-invalid".to_string());
    }
    debug_assert!(!request.contains_raw_frontend_eval);
    debug_assert!(request.input_refs.len() <= MAX_REMOTE_INPUT_REFS);
    Ok(())
}

fn validate_source_input_refs(request: &ConcreteBuildRequest) -> Result<(), String> {
    if request.source_input_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(format!("source-input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"));
    }
    let input_refs = request.input_refs.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for input_ref in &request.source_input_refs {
        if input_ref.is_empty() {
            return Err("source-input-ref-empty".to_string());
        }
        if !seen.insert(input_ref) {
            return Err("source-input-ref-duplicate".to_string());
        }
        if !input_refs.contains(input_ref) {
            return Err("source-input-ref-not-declared".to_string());
        }
    }
    Ok(())
}

pub fn plan_remote_executable_request(request: &ConcreteBuildRequest) -> Result<RemoteExecutablePlan, String> {
    if request.contains_raw_frontend_eval {
        return Err("raw-frontend-evaluation-rejected".to_string());
    }
    if !request.store_prefix.starts_with('/') {
        return Err("store-prefix-not-absolute".to_string());
    }
    validate_expected_outputs(&request.expected_outputs, &request.store_prefix)?;
    match &request.payload {
        RemoteConcreteBuildPayload::Action { action_id, spec_json } => {
            plan_remote_action_payload((action_id, spec_json), request)
        }
        RemoteConcreteBuildPayload::Derivation { drv_path, drv_json } => {
            plan_remote_derivation_payload((drv_path, drv_json), request)
        }
    }
}

fn plan_remote_action_payload(
    payload: (&str, &str),
    request: &ConcreteBuildRequest,
) -> Result<RemoteExecutablePlan, String> {
    let (action_id, spec_json) = payload;
    if action_id.is_empty() {
        return Err("remote-action-id-empty".to_string());
    }
    validate_remote_payload_json_size(spec_json)?;
    let spec: RemoteActionSpec =
        serde_json::from_str(spec_json).map_err(|err| format!("remote-action-spec-json-invalid: {err}"))?;
    validate_action_spec(action_id, &spec, &request.expected_outputs)?;
    let command_args = remote_action_command_args(&spec)?;
    validate_command_env(&spec.env)?;
    let source = RemoteExecutablePlanSource::Action {
        action_id: action_id.to_string(),
        schema: spec.schema.clone(),
        spec_digest_blake3: remote_action_spec_digest(&spec)?,
    };
    finish_remote_executable_plan(request, source, command_args, spec.env, spec.system)
}

fn plan_remote_derivation_payload(
    payload: (&str, &str),
    request: &ConcreteBuildRequest,
) -> Result<RemoteExecutablePlan, String> {
    let (drv_path, drv_json) = payload;
    if drv_path.is_empty() || !drv_path.starts_with(&request.store_prefix) {
        return Err("remote-derivation-path-store-prefix-mismatch".to_string());
    }
    validate_remote_payload_json_size(drv_json)?;
    let drv: crunch_glue::CrunchDerivation =
        serde_json::from_str(drv_json).map_err(|err| format!("remote-derivation-json-invalid: {err}"))?;
    let mut known_paths = crunch_glue::ConversionCache::new(&request.store_prefix);
    let (computed_path, nix_drv) = crunch_glue::convert(&drv, &mut known_paths)
        .map_err(|err| format!("remote-derivation-convert-invalid: {err}"))?;
    let computed_drv_path = computed_path.to_absolute_path_with_prefix(&request.store_prefix);
    validate_derivation_plan_identity(
        (drv_path, &computed_drv_path),
        &nix_drv,
        &request.expected_outputs,
        &request.store_prefix,
    )?;
    debug_assert_eq!(drv_path, computed_drv_path);
    debug_assert!(request.expected_outputs.len() <= MAX_REMOTE_EXPECTED_OUTPUTS);
    let source = RemoteExecutablePlanSource::Derivation {
        declared_drv_path: drv_path.to_string(),
        computed_drv_path,
        drv_name: drv.name,
        drv_digest_blake3: remote_derivation_digest(&nix_drv, &request.store_prefix),
    };
    finish_remote_executable_plan(
        request,
        source,
        remote_derivation_command_args(&nix_drv)?,
        remote_derivation_env_utf8(&nix_drv)?,
        nix_drv.system,
    )
}

fn validate_remote_payload_json_size(payload_json: &str) -> Result<(), String> {
    if payload_json.is_empty() {
        return Err("remote-build-payload-empty".to_string());
    }
    if payload_json.len() > MAX_REMOTE_BUILD_PAYLOAD_BYTES {
        return Err(format!("remote-build-payload-exceeds-{MAX_REMOTE_BUILD_PAYLOAD_BYTES}"));
    }
    Ok(())
}

fn validate_expected_outputs(outputs: &[RemoteExpectedOutput], store_prefix: &str) -> Result<(), String> {
    if outputs.is_empty() {
        return Err("remote-expected-outputs-empty".to_string());
    }
    if outputs.len() > MAX_REMOTE_EXPECTED_OUTPUTS {
        return Err(format!("remote-expected-output-count-exceeds-{MAX_REMOTE_EXPECTED_OUTPUTS}"));
    }
    let mut names = BTreeSet::new();
    for output in outputs {
        if output.name.is_empty() {
            return Err("remote-expected-output-name-empty".to_string());
        }
        if !names.insert(output.name.as_str()) {
            return Err("remote-expected-output-name-duplicate".to_string());
        }
        if let Some(logical_path) = &output.logical_path
            && (logical_path.is_empty() || !logical_path.starts_with(store_prefix))
        {
            return Err("remote-expected-output-store-prefix-mismatch".to_string());
        }
    }
    debug_assert_eq!(names.len(), outputs.len());
    debug_assert!(outputs.len() <= MAX_REMOTE_EXPECTED_OUTPUTS);
    Ok(())
}

fn validate_action_spec(
    action_id: &str,
    spec: &RemoteActionSpec,
    expected_outputs: &[RemoteExpectedOutput],
) -> Result<(), String> {
    if spec.schema != REMOTE_ACTION_SPEC_SCHEMA {
        return Err("remote-action-schema-unsupported".to_string());
    }
    if spec.action_id != action_id {
        return Err("remote-action-id-mismatch".to_string());
    }
    if spec.builder.is_empty() || spec.system.is_empty() {
        return Err("remote-action-executor-field-empty".to_string());
    }
    validate_action_outputs(&spec.outputs, expected_outputs)
}

fn validate_action_outputs(outputs: &[String], expected_outputs: &[RemoteExpectedOutput]) -> Result<(), String> {
    if outputs.is_empty() {
        return Err("remote-action-outputs-empty".to_string());
    }
    if outputs.len() > MAX_REMOTE_EXPECTED_OUTPUTS {
        return Err(format!("remote-action-output-count-exceeds-{MAX_REMOTE_EXPECTED_OUTPUTS}"));
    }
    let mut action_names = BTreeSet::new();
    for output in outputs {
        if output.is_empty() {
            return Err("remote-action-output-name-empty".to_string());
        }
        if !action_names.insert(output.clone()) {
            return Err("remote-action-output-name-duplicate".to_string());
        }
    }
    let expected_names = expected_outputs.iter().map(|output| output.name.clone()).collect::<BTreeSet<_>>();
    if action_names != expected_names {
        return Err("remote-action-expected-output-mismatch".to_string());
    }
    debug_assert_eq!(action_names.len(), outputs.len());
    debug_assert_eq!(action_names, expected_names);
    Ok(())
}

fn validate_derivation_plan_identity(
    paths: (&str, &str),
    derivation: &nix_compat::derivation::Derivation,
    expected_outputs: &[RemoteExpectedOutput],
    store_prefix: &str,
) -> Result<(), String> {
    let (declared_drv_path, computed_drv_path) = paths;
    if declared_drv_path != computed_drv_path {
        return Err("remote-derivation-path-mismatch".to_string());
    }
    validate_derivation_expected_outputs(derivation, expected_outputs, store_prefix)
}

fn validate_derivation_expected_outputs(
    derivation: &nix_compat::derivation::Derivation,
    expected_outputs: &[RemoteExpectedOutput],
    store_prefix: &str,
) -> Result<(), String> {
    for expected in expected_outputs {
        let output = derivation
            .outputs
            .get(&expected.name)
            .ok_or_else(|| "remote-derivation-expected-output-unknown".to_string())?;
        match (&output.path, &expected.logical_path) {
            (Some(path), Some(expected_logical_path)) => {
                let logical_path = path.to_absolute_path_with_prefix(store_prefix);
                if &logical_path != expected_logical_path {
                    return Err("remote-derivation-output-path-mismatch".to_string());
                }
            }
            (Some(_), None) => return Err("remote-derivation-output-path-missing".to_string()),
            (None, Some(_)) => return Err("remote-derivation-output-path-unexpected".to_string()),
            (None, None) => {}
        }
    }
    debug_assert!(expected_outputs.len() <= derivation.outputs.len());
    debug_assert!(expected_outputs.iter().all(|expected| derivation.outputs.contains_key(&expected.name)));
    Ok(())
}

fn remote_action_command_args(spec: &RemoteActionSpec) -> Result<Vec<String>, String> {
    let mut command_args = Vec::with_capacity(spec.args.len().saturating_add(1));
    command_args.push(spec.builder.clone());
    command_args.extend(spec.args.clone());
    validate_command_args(&command_args)?;
    Ok(command_args)
}

fn remote_derivation_command_args(derivation: &nix_compat::derivation::Derivation) -> Result<Vec<String>, String> {
    let mut command_args = Vec::with_capacity(derivation.arguments.len().saturating_add(1));
    command_args.push(derivation.builder.clone());
    command_args.extend(derivation.arguments.clone());
    validate_command_args(&command_args)?;
    Ok(command_args)
}

fn validate_command_args(command_args: &[String]) -> Result<(), String> {
    if command_args.len() > MAX_REMOTE_EXECUTOR_ARGS {
        return Err(format!("remote-executor-arg-count-exceeds-{MAX_REMOTE_EXECUTOR_ARGS}"));
    }
    if command_args.first().map(String::is_empty).unwrap_or(true) {
        return Err("remote-executor-builder-empty".to_string());
    }
    Ok(())
}

fn validate_command_env(env: &BTreeMap<String, String>) -> Result<(), String> {
    if env.len() > MAX_REMOTE_EXECUTOR_ENV_VARS {
        return Err(format!("remote-executor-env-count-exceeds-{MAX_REMOTE_EXECUTOR_ENV_VARS}"));
    }
    for (name, value) in env {
        if name.is_empty() {
            return Err("remote-executor-env-name-empty".to_string());
        }
        if value.len() > MAX_REMOTE_EXECUTOR_ENV_VALUE_BYTES {
            return Err(format!("remote-executor-env-value-exceeds-{MAX_REMOTE_EXECUTOR_ENV_VALUE_BYTES}"));
        }
    }
    Ok(())
}

fn remote_derivation_env_utf8(
    derivation: &nix_compat::derivation::Derivation,
) -> Result<BTreeMap<String, String>, String> {
    if derivation.environment.len() > MAX_REMOTE_EXECUTOR_ENV_VARS {
        return Err(format!("remote-executor-env-count-exceeds-{MAX_REMOTE_EXECUTOR_ENV_VARS}"));
    }
    let mut env_entries = Vec::with_capacity(derivation.environment.len());
    for (name, value) in &derivation.environment {
        let value = String::from_utf8(value.to_vec()).map_err(|_| "remote-derivation-env-non-utf8".to_string())?;
        env_entries.push((name.clone(), value));
    }
    let env = env_entries.into_iter().collect::<BTreeMap<_, _>>();
    validate_command_env(&env)?;
    Ok(env)
}

fn remote_action_spec_digest(spec: &RemoteActionSpec) -> Result<String, String> {
    let bytes = serde_json::to_vec(spec).map_err(|err| format!("remote-action-spec-digest-json-invalid: {err}"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn remote_derivation_digest(derivation: &nix_compat::derivation::Derivation, store_prefix: &str) -> String {
    blake3::hash(&derivation.to_aterm_bytes_with_store_dir(store_prefix)).to_hex().to_string()
}

fn finish_remote_executable_plan(
    request: &ConcreteBuildRequest,
    source: RemoteExecutablePlanSource,
    command_args: Vec<String>,
    command_env: BTreeMap<String, String>,
    system: String,
) -> Result<RemoteExecutablePlan, String> {
    validate_command_args(&command_args)?;
    validate_command_env(&command_env)?;
    let plan_digest_blake3 = remote_executable_plan_digest(RemoteExecutablePlanDigestInput {
        request_id: &request.request_id,
        store_prefix: &request.store_prefix,
        source: &source,
        command_args: &command_args,
        command_env: &command_env,
        system: &system,
        expected_outputs: &request.expected_outputs,
    });
    let plan = RemoteExecutablePlan {
        request_id: request.request_id.clone(),
        store_prefix: request.store_prefix.clone(),
        source,
        command_args,
        command_env,
        system,
        expected_outputs: request.expected_outputs.clone(),
        plan_digest_blake3,
    };
    debug_assert!(is_blake3_hex_digest(&plan.plan_digest_blake3));
    debug_assert_eq!(plan.expected_outputs.len(), request.expected_outputs.len());
    Ok(plan)
}

struct RemoteExecutablePlanDigestInput<'a> {
    request_id: &'a str,
    store_prefix: &'a str,
    source: &'a RemoteExecutablePlanSource,
    command_args: &'a [String],
    command_env: &'a BTreeMap<String, String>,
    system: &'a str,
    expected_outputs: &'a [RemoteExpectedOutput],
}

fn remote_executable_plan_digest(input: RemoteExecutablePlanDigestInput<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "request-id", input.request_id);
    hash_labeled_str(&mut hasher, "store-prefix", input.store_prefix);
    hash_executable_plan_source(&mut hasher, input.source);
    hash_labeled_str(&mut hasher, "system", input.system);
    for arg in input.command_args {
        hash_labeled_str(&mut hasher, "command-arg", arg);
    }
    for (name, value) in input.command_env {
        hash_labeled_str(&mut hasher, "env-name", name);
        hash_labeled_str(&mut hasher, "env-value", value);
    }
    for output in input.expected_outputs {
        hash_labeled_str(&mut hasher, "output-name", &output.name);
        match &output.logical_path {
            Some(logical_path) => hash_labeled_str(&mut hasher, "output-path", logical_path),
            None => hash_labeled_str(&mut hasher, "output-path-state", "content-addressed-unknown"),
        }
    }
    let digest_blake3 = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert!(is_blake3_hex_digest(&digest_blake3));
    digest_blake3
}

fn hash_executable_plan_source(hasher: &mut blake3::Hasher, source: &RemoteExecutablePlanSource) {
    match source {
        RemoteExecutablePlanSource::Action {
            action_id,
            schema,
            spec_digest_blake3,
        } => {
            hash_labeled_str(hasher, "source-kind", "action");
            hash_labeled_str(hasher, "action-id", action_id);
            hash_labeled_str(hasher, "schema", schema);
            hash_labeled_str(hasher, "spec-digest", spec_digest_blake3);
        }
        RemoteExecutablePlanSource::Derivation {
            declared_drv_path,
            computed_drv_path,
            drv_name,
            drv_digest_blake3,
        } => {
            hash_labeled_str(hasher, "source-kind", "derivation");
            hash_labeled_str(hasher, "declared-drv-path", declared_drv_path);
            hash_labeled_str(hasher, "computed-drv-path", computed_drv_path);
            hash_labeled_str(hasher, "drv-name", drv_name);
            hash_labeled_str(hasher, "drv-digest", drv_digest_blake3);
        }
    }
}

fn hash_labeled_str(hasher: &mut blake3::Hasher, label: &str, value: impl AsRef<str>) {
    let value = value.as_ref();
    hasher.update(label.as_bytes());
    hasher.update(b":");
    hasher.update(value.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(value.as_bytes());
}

fn default_remote_action_system() -> String {
    DEFAULT_REMOTE_ACTION_SYSTEM.to_string()
}

fn execute_remote_fixture_plan(
    plan: &RemoteExecutablePlan,
    input_refs: &[String],
) -> Result<RemoteExecutionOutcome, String> {
    validate_remote_execution_plan(plan)?;
    let mut outputs = Vec::with_capacity(plan.expected_outputs.len());
    for expected in &plan.expected_outputs {
        outputs.push(fixture_execution_output(plan, input_refs, expected));
    }
    let output_size_bytes = sum_remote_execution_output_sizes(&outputs)?;
    let output_digest_blake3 = remote_execution_outputs_digest(&outputs)?;
    Ok(RemoteExecutionOutcome {
        request_id: plan.request_id.clone(),
        plan_digest_blake3: plan.plan_digest_blake3.clone(),
        output_digest_blake3,
        output_size_bytes,
        outputs,
    })
}

struct RemoteLocalBuildExecutionInput<'a> {
    request: &'a ConcreteBuildRequest,
    plan: &'a RemoteExecutablePlan,
    input_upload: &'a RemoteInputUpload,
    created_unix_s: u64,
}

fn execute_remote_local_build(
    executor: &RemoteLocalBuildExecutor,
    input: RemoteLocalBuildExecutionInput<'_>,
) -> Result<RemoteExecutionOutcome, String> {
    let RemoteLocalBuildExecutionInput {
        request,
        plan,
        input_upload,
        created_unix_s,
    } = input;
    validate_local_executor_input_refs(request, &input_upload.refs)?;
    if executor.store_prefix != request.store_prefix {
        return Err("remote-local-executor-store-prefix-mismatch".to_string());
    }
    validate_remote_execution_plan(plan)?;
    debug_assert_eq!(executor.store_prefix, request.store_prefix);
    debug_assert_eq!(plan.request_id, request.request_id);

    #[cfg(target_os = "linux")]
    {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(REMOTE_LOCAL_EXECUTOR_THREADS)
            .enable_all()
            .build()
            .map_err(|err| format!("remote-local-executor-runtime: {err}"))?;
        rt.block_on(execute_remote_local_build_linux(executor, RemoteLocalBuildExecutionInput {
            request,
            plan,
            input_upload,
            created_unix_s,
        }))
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = executor;
        let _ = request;
        let _ = plan;
        Err("remote-local-executor-unsupported-platform".to_string())
    }
}

#[cfg(target_os = "linux")]
async fn execute_remote_local_build_linux(
    executor: &RemoteLocalBuildExecutor,
    input: RemoteLocalBuildExecutionInput<'_>,
) -> Result<RemoteExecutionOutcome, String> {
    use snix_build::buildservice::BubblewrapBuildService;
    let RemoteLocalBuildExecutionInput {
        request,
        plan,
        input_upload,
        created_unix_s,
    } = input;
    let _mutation_guard = crunch_store::StoreMutationGuard::acquire_wait(&executor.state_dir)
        .map_err(|err| format!("remote-local-executor-mutation-lock: {err}"))?;
    let (drv_path, mut known_paths) = local_derivation_registry(request, plan)?;
    let workspace_lease = acquire_remote_execution_workspace(executor, request, &drv_path, &mut known_paths)?;
    let store = crunch_store::StoreHandle::open(remote_local_store_config(executor))
        .await
        .map_err(|err| format!("remote-local-executor-open-store: {err}"))?;
    materialize_remote_input_upload(&store, request, input_upload).await?;
    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    let pathinfo_service = store.pathinfo_service();
    let workdir = std::env::temp_dir().join(REMOTE_LOCAL_BUILD_WORKDIR_NAME);
    std::fs::create_dir_all(&workdir).map_err(|err| format!("remote-local-executor-workdir: {err}"))?;
    let failure_workspace_root = remote_failure_workspace_root(executor, request)?;
    let mut bwrap_service = BubblewrapBuildService::new(workdir, blob_service.clone(), directory_service.clone());
    if request.failure_debug_policy.capture.enabled {
        bwrap_service = bwrap_service.with_failure_workspace_root(failure_workspace_root.clone());
    }
    let fetch_service = crunch_build::FetchBuildService::new(blob_service.clone(), directory_service.clone());
    let dispatch = crunch_build::DispatchBuildService::new(fetch_service, bwrap_service);
    let workspace_evidence_collector = new_remote_workspace_report_collector();
    let build_service =
        crunch_build::StatefulWorkspaceBuildService::new(dispatch, &executor.state_dir, workspace_evidence_collector);
    let mut builder = crunch_build::Builder::with_state_dir(
        blob_service,
        directory_service,
        build_service,
        pathinfo_service,
        executor.output_dir.clone(),
        Some(store.state_dir().to_path_buf()),
        store.remote_pathinfo(),
        &executor.store_prefix,
        executor.keypair.clone(),
        executor.trusted_keys.clone(),
        executor.trust_unsigned,
        executor.verbose,
    );
    builder.set_root_retention_source(Some(crunch_store::GcRootSource::Build));
    let build_result = builder.build(&drv_path, &mut known_paths).await;
    let outcome = match build_result {
        Ok(outcome) => {
            release_remote_execution_workspace(executor, workspace_lease)?;
            outcome
        }
        Err(error) => {
            let build_error = error.to_string();
            return Err(remote_local_build_failure(executor, RemoteLocalBuildFailureInput {
                request,
                failure_workspace_root: &failure_workspace_root,
                workspace_lease,
                build_error: &build_error,
                created_unix_s,
            }));
        }
    };
    debug_assert_eq!(outcome.outputs.len(), plan.expected_outputs.len());
    debug_assert!(remote_output_names_match_plan(&outcome, plan));
    remote_execution_outcome_from_build_outcome(request, plan, &outcome)
}

#[cfg(target_os = "linux")]
fn remote_output_names_match_plan(outcome: &crunch_build::BuildOutcome, plan: &RemoteExecutablePlan) -> bool {
    outcome
        .outputs
        .keys()
        .all(|name| plan.expected_outputs.iter().any(|expected| &expected.name == name))
}

#[cfg(target_os = "linux")]
fn new_remote_workspace_report_collector() -> crunch_build::WorkspaceReportCollector {
    new_default_remote_value()
}

#[cfg(target_os = "linux")]
fn new_default_remote_value<T: Default>() -> T {
    T::default()
}

#[cfg(target_os = "linux")]
fn remote_local_store_config(executor: &RemoteLocalBuildExecutor) -> crunch_store::StoreConfig {
    let config = crunch_store::StoreConfig {
        state_dir: executor.state_dir.clone(),
        output_dir: executor.output_dir.clone(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: executor.store_prefix.clone(),
        base_state_dirs: Vec::new(),
    };
    debug_assert_eq!(config.store_dir, executor.store_prefix);
    debug_assert_eq!(config.state_dir, executor.state_dir);
    config
}

#[cfg(target_os = "linux")]
struct RemoteLocalBuildFailureInput<'a> {
    request: &'a ConcreteBuildRequest,
    failure_workspace_root: &'a Path,
    workspace_lease: Option<crunch_build::WorkspaceLeaseRequest>,
    build_error: &'a str,
    created_unix_s: u64,
}

#[cfg(target_os = "linux")]
fn remote_local_build_failure(executor: &RemoteLocalBuildExecutor, input: RemoteLocalBuildFailureInput<'_>) -> String {
    let debug_ref = publish_remote_worker_failure_debug(
        executor,
        input.request,
        input.failure_workspace_root,
        input.created_unix_s,
    );
    let cleanup = cleanup_remote_failure_workspace(input.failure_workspace_root);
    let workspace_release = release_remote_execution_workspace(executor, input.workspace_lease);
    let mut message = format!("remote-local-executor-build: {}", input.build_error);
    append_remote_failure_debug_result(&mut message, debug_ref);
    if let Err(cleanup_error) = cleanup
        && cleanup_error.kind() != std::io::ErrorKind::NotFound
    {
        message.push_str("; remote-failure-workspace-cleanup-degraded");
    }
    if let Err(release_error) = workspace_release {
        append_redacted_remote_failure(
            &mut message,
            ("remote-workspace-release-degraded", "workspace-release-failed"),
            &release_error,
        );
    }
    debug_assert!(message.starts_with("remote-local-executor-build:"));
    debug_assert!(!message.contains("token="));
    message
}

#[cfg(target_os = "linux")]
fn append_remote_failure_debug_result(
    message: &mut String,
    debug_ref: Result<Option<crate::remote_failure_debug::RemoteFailureDebugPublishOutcome>, String>,
) {
    match debug_ref {
        Ok(Some(debug)) => {
            message.push_str("; worker_bundle_ref=");
            message.push_str(&debug.bundle_ref);
            message.push_str("; worker_capture_outcome=");
            message.push_str(&debug.capture_outcome_code);
            message.push_str("; worker_cleanup_status=");
            message.push_str(&debug.cleanup_status_code);
        }
        Ok(None) => {}
        Err(capture_error) => append_redacted_remote_failure(
            message,
            ("remote-failure-debug-capture-failed", "capture-failed"),
            &capture_error,
        ),
    }
}

#[cfg(target_os = "linux")]
fn append_redacted_remote_failure(message: &mut String, diagnostic: (&str, &str), raw_error: impl AsRef<str>) {
    let (label, redaction_fallback) = diagnostic;
    let safe_error = crunch_build::distributed::redact_remote_failure_diagnostic(raw_error.as_ref())
        .unwrap_or_else(|_| redaction_fallback.to_string());
    message.push_str("; ");
    message.push_str(label);
    message.push(':');
    message.push_str(&safe_error);
}

fn acquire_remote_execution_workspace(
    executor: &RemoteLocalBuildExecutor,
    request: &ConcreteBuildRequest,
    drv_path: &StorePath<String>,
    registry: &mut crunch_build::DerivationRegistry,
) -> Result<Option<crunch_build::WorkspaceLeaseRequest>, String> {
    let drv_abs = drv_path.to_absolute_path_with_prefix(&request.store_prefix);
    let entry = registry
        .get_by_drv_path_mut(&drv_abs)
        .ok_or_else(|| "remote-workspace-registry-entry-missing".to_string())?;
    let Some(raw_policy) = entry.derivation.environment.get(crunch_build::WORKSPACE_POLICY_ENV) else {
        return Ok(None);
    };
    let policy: crunch_build::WorkspacePolicy = serde_json::from_slice(raw_policy.as_ref())
        .map_err(|error| format!("remote-workspace-policy-invalid: {error}"))?;
    crunch_build::validate_workspace_policy(&policy)
        .map_err(|reason| format!("remote-workspace-policy-invalid: {}", reason.as_str()))?;
    if policy.mode != crunch_build::WorkspaceMode::MutableSession {
        return Ok(None);
    }
    let binding = request
        .production_attempt
        .as_ref()
        .ok_or_else(|| "remote-workspace-production-attempt-missing".to_string())?;
    validate_current_production_attempt(&executor.coordinator_state_dir, binding)?;
    let mut state = load_coordinator_state(&executor.coordinator_state_dir).map_err(|error| error.to_string())?;
    let registration = state
        .workspace_registrations
        .get(&executor.endpoint_id)
        .ok_or_else(|| "remote worker has no workspace registration".to_string())?;
    let owner = crunch_build::WorkspaceLeaseOwner {
        worker_id: executor.endpoint_id.clone(),
        authority_class: registration.authority_class.clone(),
        job_id: binding.job_id.as_str().to_string(),
        attempt_id: binding.attempt_id.as_str().to_string(),
        fence_generation: binding.fence_generation.get(),
    };
    let lease_request = crunch_build::WorkspaceLeaseRequest {
        workspace_id: policy.workspace_id.clone().ok_or_else(|| "remote workspace id missing".to_string())?,
        compatibility_digest_blake3: crunch_build::derive_workspace_compatibility_digest(&policy.compatibility)
            .map_err(|reason| reason.as_str().to_string())?,
        toolchain_refs: policy.compatibility.toolchain_refs.clone(),
        guest_path: policy.guest_path.clone(),
        quota: policy.quota.clone(),
        retention_class: "declared".to_string(),
        generation: binding.fence_generation.get(),
        owner,
        operation: crunch_build::WorkspaceLeaseOperation::Acquire,
    };
    let plan = apply_remote_workspace_lease(&mut state, &executor.endpoint_id, &lease_request)?;
    if plan.disposition != crunch_build::WorkspaceLeaseDisposition::Accepted {
        return Err(format!("remote workspace lease rejected: {}", plan.reason.as_str()));
    }
    let lease_binding = snix_build::buildservice::StatefulWorkspaceLeaseBinding {
        worker_id: lease_request.owner.worker_id.clone(),
        authority_class: lease_request.owner.authority_class.clone(),
        job_id: lease_request.owner.job_id.clone(),
        attempt_id: lease_request.owner.attempt_id.clone(),
        fence_generation: lease_request.owner.fence_generation,
    };
    let derivation = std::sync::Arc::make_mut(&mut entry.derivation);
    derivation.environment.insert(
        crunch_build::WORKSPACE_LEASE_ENV.to_string(),
        serde_json::to_vec(&lease_binding).map_err(|error| error.to_string())?.into(),
    );
    debug_assert_eq!(lease_request.owner.worker_id, executor.endpoint_id);
    debug_assert_eq!(lease_request.owner.fence_generation, binding.fence_generation.get());
    Ok(Some(lease_request))
}

fn release_remote_execution_workspace(
    executor: &RemoteLocalBuildExecutor,
    request: Option<crunch_build::WorkspaceLeaseRequest>,
) -> Result<(), String> {
    let Some(mut request) = request else {
        return Ok(());
    };
    request.operation = crunch_build::WorkspaceLeaseOperation::Release;
    let mut state = load_coordinator_state(&executor.coordinator_state_dir).map_err(|error| error.to_string())?;
    let plan = apply_remote_workspace_lease(&mut state, &executor.endpoint_id, &request)?;
    if plan.disposition != crunch_build::WorkspaceLeaseDisposition::Accepted {
        return Err(format!("remote workspace release rejected: {}", plan.reason.as_str()));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn remote_failure_workspace_root(
    executor: &RemoteLocalBuildExecutor,
    request: &ConcreteBuildRequest,
) -> Result<PathBuf, String> {
    let attempt = request
        .production_attempt
        .as_ref()
        .ok_or_else(|| "remote-failure-workspace-attempt-missing".to_string())?;
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "domain", "mantle-remote-failure-workspace-v1");
    hash_labeled_str(&mut hasher, "request-id", &request.request_id);
    hash_labeled_str(&mut hasher, "attempt-id", attempt.attempt_id.as_str());
    hash_labeled_str(&mut hasher, "fence-generation", attempt.fence_generation.get().to_string());
    Ok(executor.state_dir.join(REMOTE_FAILURE_WORKSPACE_DIR).join(hasher.finalize().to_hex().as_str()))
}

#[cfg(target_os = "linux")]
fn publish_remote_worker_failure_debug(
    executor: &RemoteLocalBuildExecutor,
    request: &ConcreteBuildRequest,
    failure_workspace_root: &Path,
    created_unix_s: u64,
) -> Result<Option<crate::remote_failure_debug::RemoteFailureDebugPublishOutcome>, String> {
    if !request.failure_debug_policy.capture.enabled {
        return Ok(None);
    }
    let attempt = request
        .production_attempt
        .as_ref()
        .ok_or_else(|| "remote-failure-worker-attempt-missing".to_string())?
        .clone();
    let capture_root =
        sole_retained_failure_workspace(failure_workspace_root)?.map(|workspace| workspace.join("scratches"));
    let outcome = crate::remote_failure_debug::publish_remote_failure_debug_bundle(
        crate::remote_failure_debug::RemoteFailureDebugPublishRequest {
            state_dir: &executor.state_dir,
            capture_root: capture_root.as_deref(),
            facts: crate::remote_failure_debug::RemoteFailureDebugSourceFacts {
                request: request.clone(),
                attempt,
                immutable_log: None,
                route_class: "worker-local-execution".to_string(),
                worker_capability_classes: vec!["bubblewrap".to_string(), "pre-cleanup-capture".to_string()],
                sandbox_policy_class: "bubblewrap".to_string(),
                network_policy_class: "request-policy".to_string(),
                transfer_status_class: "input-transfer-complete".to_string(),
                admission_status_class: "not-admitted".to_string(),
                workspace_mode: crunch_build::distributed::RemoteFailureWorkspaceMode::Ephemeral,
                failure_phase: crunch_build::distributed::RemoteFailureDebugPhase::Execution,
                failure_reason_code: "sandbox-build-failed".to_string(),
                cleanup_status_code: "cleanup-attempted-after-capture".to_string(),
                created_unix_s,
            },
            policy: request.failure_debug_policy.clone(),
        },
    )?;
    debug_assert!(!outcome.bundle_ref.is_empty());
    debug_assert!(!outcome.capture_outcome_code.is_empty());
    Ok(Some(outcome))
}

#[cfg(target_os = "linux")]
fn sole_retained_failure_workspace(root: &Path) -> Result<Option<PathBuf>, String> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("remote-failure-workspace-read-failed:{error}")),
    };
    let mut retained = Vec::with_capacity(REMOTE_FAILURE_WORKSPACE_SCAN_MAX);
    for entry in entries {
        if retained.len() >= REMOTE_FAILURE_WORKSPACE_SCAN_MAX {
            return Err("remote-failure-workspace-count-exceeded".to_string());
        }
        let path = entry.map_err(|error| format!("remote-failure-workspace-entry-failed:{error}"))?.path();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("remote-failure-workspace-metadata-failed:{error}"))?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            retained.push(path);
        }
    }
    if retained.len() > 1 {
        return Err("remote-failure-workspace-ambiguous".to_string());
    }
    debug_assert!(retained.len() <= 1);
    debug_assert!(retained.len() <= REMOTE_FAILURE_WORKSPACE_SCAN_MAX);
    Ok(retained.pop())
}

#[cfg(target_os = "linux")]
fn cleanup_remote_failure_workspace(root: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if !root.exists() {
        return Ok(());
    }
    let mut pending = vec![root.to_path_buf()];
    debug_assert_eq!(pending.len(), 1);
    debug_assert_eq!(pending.last().map(PathBuf::as_path), Some(root));
    let mut visited = 0_usize;
    while let Some(path) = pending.pop() {
        if visited >= REMOTE_FAILURE_CLEANUP_ENTRIES_MAX {
            return Err(std::io::Error::other("remote failure cleanup entry limit exceeded"));
        }
        visited = visited.saturating_add(1);
        debug_assert!(visited <= REMOTE_FAILURE_CLEANUP_ENTRIES_MAX);
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(REMOTE_FAILURE_CLEANUP_DIRECTORY_MODE))?;
            pending.extend(std::fs::read_dir(&path)?.filter_map(Result::ok).map(|entry| entry.path()));
        } else if metadata.is_file() {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(REMOTE_FAILURE_CLEANUP_FILE_MODE))?;
        }
    }
    std::fs::remove_dir_all(root)
}

fn local_derivation_registry(
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
) -> Result<(StorePath<String>, crunch_build::DerivationRegistry), String> {
    let RemoteConcreteBuildPayload::Derivation { drv_path, drv_json } = &request.payload else {
        return Err("remote-local-executor-action-unsupported".to_string());
    };
    let declared = StorePath::from_absolute_path_with_prefix(drv_path.as_bytes(), &request.store_prefix)
        .map_err(|_| "remote-local-executor-drv-path-invalid".to_string())?;
    let drv: crunch_glue::CrunchDerivation = serde_json::from_str(drv_json)
        .map_err(|err| format!("remote-local-executor-derivation-json-invalid: {err}"))?;
    let mut conversion_cache = crunch_glue::ConversionCache::new(&request.store_prefix);
    let (computed, _) = crunch_glue::convert(&drv, &mut conversion_cache)
        .map_err(|err| format!("remote-local-executor-derivation-convert-invalid: {err}"))?;
    if declared != computed {
        return Err("remote-local-executor-drv-path-mismatch".to_string());
    }
    if !matches!(
        &plan.source,
        RemoteExecutablePlanSource::Derivation { computed_drv_path, .. }
            if computed_drv_path == &computed.to_absolute_path_with_prefix(&request.store_prefix)
    ) {
        return Err("remote-local-executor-plan-source-mismatch".to_string());
    }
    let mut registry = crunch_build::DerivationRegistry::new(&request.store_prefix);
    crunch_build::populate_registry(&mut registry, conversion_cache.iter_entries());
    let computed_abs = computed.to_absolute_path_with_prefix(&request.store_prefix);
    if registry.get_by_drv_path(&computed_abs).is_none() {
        return Err("remote-local-executor-registry-root-missing".to_string());
    }
    debug_assert_eq!(declared, computed);
    debug_assert!(registry.get_by_drv_path(&computed_abs).is_some());
    Ok((computed, registry))
}

fn remote_execution_outcome_from_build_outcome(
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
    outcome: &crunch_build::BuildOutcome,
) -> Result<RemoteExecutionOutcome, String> {
    let mut outputs = Vec::with_capacity(plan.expected_outputs.len());
    for expected in &plan.expected_outputs {
        let path_info = outcome
            .outputs
            .get(&expected.name)
            .ok_or_else(|| "remote-local-executor-output-missing".to_string())?;
        outputs.push(remote_execution_output_from_pathinfo(&request.store_prefix, expected, path_info, None)?);
    }
    remote_execution_outcome_from_outputs(request, plan, outputs)
}

fn remote_execution_outcome_from_outputs(
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
    outputs: Vec<RemoteExecutionOutput>,
) -> Result<RemoteExecutionOutcome, String> {
    let output_size_bytes = sum_remote_execution_output_sizes(&outputs)?;
    let output_digest_blake3 = remote_execution_outputs_digest(&outputs)?;
    Ok(RemoteExecutionOutcome {
        request_id: request.request_id.clone(),
        plan_digest_blake3: plan.plan_digest_blake3.clone(),
        output_digest_blake3,
        output_size_bytes,
        outputs,
    })
}

fn remote_execution_output_from_pathinfo(
    store_prefix: &str,
    expected: &RemoteExpectedOutput,
    path_info: &PathInfo,
    nar_payload: Option<Vec<u8>>,
) -> Result<RemoteExecutionOutput, String> {
    let logical_path = path_info.store_path.to_absolute_path_with_prefix(store_prefix);
    if let Some(expected_logical_path) = &expected.logical_path
        && &logical_path != expected_logical_path
    {
        return Err("remote-local-executor-output-path-mismatch".to_string());
    }
    let content_digest_blake3 = remote_pathinfo_node_digest_blake3(path_info)?;
    let artifact_attestation_digest_blake3 =
        crunch_store::artifact_attestation_digest_for_pathinfo(store_prefix, path_info, &expected.name, None)
            .map_err(|err| format!("remote-local-executor-artifact-digest: {err}"))?
            .to_hex();
    let output = RemoteExecutionOutput {
        name: expected.name.clone(),
        logical_path,
        content_digest_blake3,
        size_bytes: path_info.nar_size,
        artifact_attestation_digest_blake3,
        path_info: Some(path_info.clone()),
        nar_payload,
    };
    debug_assert_eq!(output.name, expected.name);
    debug_assert_eq!(output.size_bytes, path_info.nar_size);
    Ok(output)
}

fn remote_pathinfo_node_digest_blake3(path_info: &PathInfo) -> Result<String, String> {
    let node_bytes =
        serde_json::to_vec(&path_info.node).map_err(|err| format!("remote-local-executor-node-digest-json: {err}"))?;
    Ok(blake3::hash(&node_bytes).to_hex().to_string())
}

pub fn bind_remote_production_dispatch(
    plan: &mut RemoteClientDispatchPlan,
    attempt: RemoteProductionAttemptBinding,
    transfer_policy: RemoteTransferPolicy,
    failure_debug_policy: crunch_build::distributed::RemoteFailureDebugPolicy,
    state_dir: &Path,
) -> Result<(), String> {
    transfer_policy.validate().map_err(|reason| reason.as_str().to_string())?;
    failure_debug_policy.validate().map_err(|reason| reason.as_str().to_string())?;
    plan.client.request.production_attempt = Some(attempt);
    plan.client.request.transfer_policy = Some(transfer_policy);
    plan.client.request.failure_debug_policy = failure_debug_policy;
    let mut is_request_frame_updated = false;
    for frame in &mut plan.command.input_frames {
        if let RemoteFrame::BuildRequest { request } = frame {
            *request = plan.client.request.clone();
            is_request_frame_updated = true;
        }
    }
    if !is_request_frame_updated {
        return Err("remote-production-build-request-frame-missing".to_string());
    }
    plan.command.production_transfer = Some(RemoteProductionTransferClient {
        state_dir: state_dir.to_path_buf(),
        transfer_policy,
        telemetry_policy: default_remote_telemetry_policy(),
        trusted_output_keys: plan.client.trusted_output_keys.clone(),
        input_transfer: None,
        interrupt_after_input_chunks: None,
        interrupt_after_output_chunks: None,
    });
    assert!(plan.client.request.production_attempt.is_some());
    assert!(plan.client.request.transfer_policy.is_some());
    debug_assert!(plan.client.request.failure_debug_policy.validate().is_ok());
    Ok(())
}

pub fn set_remote_production_interrupt_after_input_chunks(
    command: &mut RemoteStdioCommand,
    chunk_count: u32,
) -> Result<(), String> {
    if chunk_count == 0 {
        return Err("remote-production-interrupt-chunk-count-zero".to_string());
    }
    let production = command
        .production_transfer
        .as_mut()
        .ok_or_else(|| "remote-production-transfer-context-missing".to_string())?;
    production.interrupt_after_input_chunks = Some(chunk_count);
    assert_eq!(production.interrupt_after_input_chunks, Some(chunk_count));
    assert!(chunk_count > 0);
    Ok(())
}

pub fn set_remote_production_interrupt_after_output_chunks(
    command: &mut RemoteStdioCommand,
    chunk_count: u32,
) -> Result<(), String> {
    if chunk_count == 0 {
        return Err("remote-production-interrupt-chunk-count-zero".to_string());
    }
    let production = command
        .production_transfer
        .as_mut()
        .ok_or_else(|| "remote-production-transfer-context-missing".to_string())?;
    production.interrupt_after_output_chunks = Some(chunk_count);
    assert_eq!(production.interrupt_after_output_chunks, Some(chunk_count));
    assert!(chunk_count > 0);
    Ok(())
}

pub fn set_remote_production_telemetry_policy(
    command: &mut RemoteStdioCommand,
    policy: RemoteTelemetryPolicy,
) -> Result<(), String> {
    policy.validate().map_err(|reason| reason.as_str().to_string())?;
    let production = command
        .production_transfer
        .as_mut()
        .ok_or_else(|| "remote-production-transfer-context-missing".to_string())?;
    production.telemetry_policy = policy;
    assert_eq!(production.telemetry_policy, policy);
    assert!(policy.batch_size <= policy.event_capacity);
    Ok(())
}

pub fn set_remote_diagnostic_trace_context(
    command: &mut RemoteStdioCommand,
    context: Option<RemoteTraceContext>,
) -> Result<(), String> {
    command.input_frames.retain(|frame| !matches!(frame, RemoteFrame::TraceContext { .. }));
    let hello_index = command
        .input_frames
        .iter()
        .position(|frame| matches!(frame, RemoteFrame::Hello { .. }))
        .ok_or_else(|| "remote-production-hello-frame-missing".to_string())?;
    let hello = match &mut command.input_frames[hello_index] {
        RemoteFrame::Hello { hello } => hello,
        _ => return Err("remote-production-hello-frame-index-invalid".to_string()),
    };
    if !hello.capabilities.iter().any(|capability| capability == REMOTE_TRACE_CONTEXT_CAPABILITY) {
        hello.capabilities.push(REMOTE_TRACE_CONTEXT_CAPABILITY.to_string());
    }
    if hello.capabilities.len() > MAX_REMOTE_CAPABILITIES {
        return Err(format!("capability count exceeds {MAX_REMOTE_CAPABILITIES}"));
    }
    let trace_index = hello_index.checked_add(1).ok_or_else(|| "remote-trace-frame-index-overflow".to_string())?;
    command.input_frames.insert(trace_index, RemoteFrame::TraceContext { context });
    assert!(matches!(command.input_frames.get(trace_index), Some(RemoteFrame::TraceContext { .. })));
    assert!(hello_index < trace_index);
    Ok(())
}

pub async fn prepare_remote_production_input_transfer(
    store: &crunch_store::StoreHandle,
    command: &mut RemoteStdioCommand,
    request: &ConcreteBuildRequest,
    source_state_dir: &Path,
) -> Result<(), String> {
    let production = command
        .production_transfer
        .as_mut()
        .ok_or_else(|| "remote-production-transfer-context-missing".to_string())?;
    let attempt = request.production_attempt.as_ref().ok_or_else(|| "remote-production-attempt-missing".to_string())?;
    let policy = request.transfer_policy.ok_or_else(|| "remote-production-transfer-policy-missing".to_string())?;
    if policy != production.transfer_policy {
        return Err("remote-production-transfer-policy-mismatch".to_string());
    }
    let spool_dir = production.state_dir.join(REMOTE_TRANSFER_SPOOL_DIR).join(&request.request_id).join("inputs");
    let mut artifacts = Vec::with_capacity(request.input_refs.len());
    for input_ref in &request.input_refs {
        artifacts.push(
            prepare_remote_input_nar_artifact(
                &RemoteInputNarPrepareContext {
                    store,
                    request,
                    source_state_dir,
                    spool_dir: &spool_dir,
                    policy,
                },
                input_ref,
            )
            .await?,
        );
    }
    if artifacts.is_empty() {
        production.input_transfer = None;
        return Ok(());
    }
    let prepared = crate::remote_transfer::prepare_remote_transfer(
        crate::remote_transfer::RemoteTransferBinding {
            job_id: attempt.job_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            fence_generation: attempt.fence_generation,
            store_prefix: request.store_prefix.clone(),
            requested_content_blake3: remote_input_requested_content_digest(request)?,
        },
        policy,
        artifacts,
    )?;
    production.input_transfer = Some(prepared);
    assert!(production.input_transfer.is_some());
    assert!(
        command
            .input_frames
            .iter()
            .all(|frame| { !matches!(frame, RemoteFrame::InputUpload { upload } if !upload.artifacts.is_empty()) })
    );
    Ok(())
}

struct RemoteInputNarPrepareContext<'a> {
    store: &'a crunch_store::StoreHandle,
    request: &'a ConcreteBuildRequest,
    source_state_dir: &'a Path,
    spool_dir: &'a Path,
    policy: RemoteTransferPolicy,
}

async fn prepare_remote_input_nar_artifact(
    context: &RemoteInputNarPrepareContext<'_>,
    input_ref: &str,
) -> Result<crate::remote_transfer::PreparedRemoteTransferArtifact, String> {
    let source_scratch;
    let host_path = match remote_input_ref_host_path(context.store, &context.request.store_prefix, input_ref) {
        Ok(path) => path,
        Err(reason) if reason == "remote-input-source-path-missing" => {
            source_scratch = tempfile::Builder::new()
                .prefix("mantle-remote-streamed-source-input-")
                .tempdir()
                .map_err(|err| format!("remote-input-source-state-scratch-failed: {err}"))?;
            let is_materialized = crate::source_bundle::materialize_imported_source_record_for_store_path(
                context.source_state_dir,
                &context.request.store_prefix,
                input_ref,
                source_scratch.path(),
            )
            .map_err(|err| format!("remote-input-source-state-materialize-failed: {err}"))?;
            if !is_materialized {
                return Err("remote-input-source-path-missing".to_string());
            }
            source_scratch.path().to_path_buf()
        }
        Err(reason) => return Err(reason),
    };
    let node = snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(
        context.store.blob_service(),
        context.store.directory_service(),
        &host_path,
        None,
    )
    .await
    .map_err(|err| format!("remote-input-source-ingest-failed: {err}"))?;
    let artifact = crate::remote_transfer::prepare_nar_node_transfer_artifact(
        context.store,
        &node,
        remote_input_nar_artifact_id(input_ref)?,
        context.spool_dir,
        true,
        context.policy,
    )
    .await?;
    debug_assert!(artifact.source_path.is_file());
    debug_assert!(artifact.descriptor.size_bytes > 0);
    Ok(artifact)
}

fn remote_input_nar_artifact_id(input_ref: &str) -> Result<RemoteTransferArtifactId, String> {
    remote_hashed_transfer_artifact_id(REMOTE_INPUT_NAR_ARTIFACT_DOMAIN, input_ref)
}

fn remote_output_nar_artifact_id(
    output_name: &str,
    logical_path: impl AsRef<str>,
) -> Result<RemoteTransferArtifactId, String> {
    remote_hashed_transfer_artifact_id(
        REMOTE_OUTPUT_NAR_ARTIFACT_DOMAIN,
        format!("{output_name}\0{}", logical_path.as_ref()),
    )
}

fn remote_output_pathinfo_artifact_id(
    output_name: &str,
    logical_path: impl AsRef<str>,
) -> Result<RemoteTransferArtifactId, String> {
    remote_hashed_transfer_artifact_id(
        REMOTE_OUTPUT_PATHINFO_ARTIFACT_DOMAIN,
        format!("{output_name}\0{}", logical_path.as_ref()),
    )
}

fn remote_hashed_transfer_artifact_id(
    domain: &str,
    identity: impl AsRef<str>,
) -> Result<RemoteTransferArtifactId, String> {
    let identity = identity.as_ref();
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "domain", domain);
    hash_labeled_str(&mut hasher, "identity", identity);
    RemoteTransferArtifactId::new(format!("{domain}:{}", hasher.finalize().to_hex()))
        .map_err(|reason| reason.as_str().to_string())
}

fn remote_input_requested_content_digest(request: &ConcreteBuildRequest) -> Result<RemoteTransferDigest, String> {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", "production-input-transfer");
    hash_labeled_str(&mut hasher, "request-id", &request.request_id);
    hash_ordered_values(&mut hasher, "input-ref", &request.input_refs);
    RemoteTransferDigest::new(hasher.finalize().to_hex().to_string()).map_err(|reason| reason.as_str().to_string())
}

pub async fn populate_remote_input_upload_artifacts_from_store(
    store: &crunch_store::StoreHandle,
    command: &mut RemoteStdioCommand,
    request: &ConcreteBuildRequest,
) -> Result<(), String> {
    let artifacts = plan_remote_input_upload_artifacts_from_store(store, request).await?;
    attach_remote_input_upload_artifacts(command, request, artifacts)
}

pub async fn populate_remote_input_upload_artifacts_from_store_or_source_state(
    store: &crunch_store::StoreHandle,
    command: &mut RemoteStdioCommand,
    request: &ConcreteBuildRequest,
    source_state_dir: &Path,
) -> Result<(), String> {
    let artifacts =
        plan_remote_input_upload_artifacts_from_store_or_source_state(store, request, source_state_dir).await?;
    attach_remote_input_upload_artifacts(command, request, artifacts)
}

fn attach_remote_input_upload_artifacts(
    command: &mut RemoteStdioCommand,
    request: &ConcreteBuildRequest,
    artifacts: Vec<RemoteInputUploadArtifact>,
) -> Result<(), String> {
    let artifact_count = artifacts.len();
    for frame in &mut command.input_frames {
        if let RemoteFrame::InputUpload { upload } = frame {
            upload.artifacts = artifacts;
            upload.byte_count = remote_input_upload_byte_count(&upload.refs, &upload.artifacts)?;
            validate_remote_input_upload_artifacts(
                &request.request_id,
                &upload.refs,
                &request.source_input_refs,
                &upload.artifacts,
            )?;
            let _summary = plan_remote_input_upload_privacy_summary(
                &upload.refs,
                &request.source_input_refs,
                &upload.artifacts,
                &RemoteInputUploadPrivacyPolicy::default(),
            )?;
            debug_assert_eq!(upload.artifacts.len(), artifact_count);
            debug_assert_eq!(upload.request_id, request.request_id);
            return Ok(());
        }
    }
    Err("remote-input-upload-frame-missing".to_string())
}

pub async fn plan_remote_input_upload_artifacts_from_store(
    store: &crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
) -> Result<Vec<RemoteInputUploadArtifact>, String> {
    plan_remote_input_upload_artifacts(store, request, None).await
}

pub async fn plan_remote_input_upload_artifacts_from_store_or_source_state(
    store: &crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
    source_state_dir: &Path,
) -> Result<Vec<RemoteInputUploadArtifact>, String> {
    plan_remote_input_upload_artifacts(store, request, Some(source_state_dir)).await
}

async fn plan_remote_input_upload_artifacts(
    store: &crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
    source_state_dir: Option<&Path>,
) -> Result<Vec<RemoteInputUploadArtifact>, String> {
    validate_source_input_refs(request)?;
    let mut artifacts = Vec::with_capacity(request.source_input_refs.len());
    for input_ref in &request.source_input_refs {
        let payload = render_remote_input_nar_payload_for_ref(store, request, input_ref, source_state_dir).await?;
        let size_bytes = remote_payload_size_bytes(payload.len())?;
        artifacts.push(RemoteInputUploadArtifact {
            request_id: request.request_id.clone(),
            input_ref: input_ref.clone(),
            artifact_kind: RemoteInputUploadArtifactKind::Nar,
            digest_blake3: blake3::hash(&payload).to_hex().to_string(),
            size_bytes,
            payload,
        });
    }
    Ok(artifacts)
}

fn remote_input_upload_byte_count(
    uploaded_refs: &[String],
    artifacts: &[RemoteInputUploadArtifact],
) -> Result<u64, String> {
    let mut total = remote_input_ref_upload_bytes(uploaded_refs)?;
    for artifact in artifacts {
        total = total
            .checked_add(artifact.size_bytes)
            .ok_or_else(|| "remote-input-upload-byte-count-overflow".to_string())?;
    }
    Ok(total)
}

pub fn plan_remote_input_upload_privacy_summary(
    uploaded_refs: &[String],
    source_input_refs: &[String],
    artifacts: &[RemoteInputUploadArtifact],
    policy: &RemoteInputUploadPrivacyPolicy,
) -> Result<RemoteInputUploadPrivacySummary, String> {
    if uploaded_refs.len() > policy.max_objects {
        return Err("remote-input-upload-object-count-exceeded".to_string());
    }
    let total_objects = bounded_runtime_count_u32(uploaded_refs.len())?;
    let total_bytes = remote_input_upload_byte_count(uploaded_refs, artifacts)?;
    if total_bytes > policy.max_bytes {
        return Err("remote-input-upload-byte-quota-exceeded".to_string());
    }
    let mut summary = RemoteInputUploadPrivacySummary {
        store_objects: 0,
        source_objects: 0,
        proof_objects: 0,
        secret_descriptor_objects: 0,
        total_objects,
        total_bytes,
    };
    for input_ref in uploaded_refs {
        let upload_class = classify_remote_input_upload_ref(input_ref, source_input_refs);
        if !policy.allowed_classes.contains(&upload_class) {
            return Err("remote-input-upload-class-disallowed".to_string());
        }
        increment_remote_input_upload_class_count(&mut summary, upload_class)?;
    }
    let classified_objects = summary
        .store_objects
        .saturating_add(summary.source_objects)
        .saturating_add(summary.proof_objects)
        .saturating_add(summary.secret_descriptor_objects);
    debug_assert_eq!(classified_objects, summary.total_objects);
    debug_assert!(summary.total_bytes <= policy.max_bytes);
    Ok(summary)
}

fn classify_remote_input_upload_ref(input_ref: &str, source_input_refs: &[String]) -> RemoteInputUploadClass {
    if source_input_refs.iter().any(|source_ref| source_ref == input_ref) {
        return RemoteInputUploadClass::Source;
    }
    if input_ref.starts_with(REMOTE_UPLOAD_PROOF_PREFIX) {
        return RemoteInputUploadClass::Proof;
    }
    if input_ref.starts_with(REMOTE_UPLOAD_SECRET_DESCRIPTOR_PREFIX) {
        return RemoteInputUploadClass::SecretDescriptor;
    }
    RemoteInputUploadClass::Store
}

fn increment_remote_input_upload_class_count(
    summary: &mut RemoteInputUploadPrivacySummary,
    upload_class: RemoteInputUploadClass,
) -> Result<(), String> {
    let count = match upload_class {
        RemoteInputUploadClass::Store => &mut summary.store_objects,
        RemoteInputUploadClass::Source => &mut summary.source_objects,
        RemoteInputUploadClass::Proof => &mut summary.proof_objects,
        RemoteInputUploadClass::SecretDescriptor => &mut summary.secret_descriptor_objects,
    };
    *count = count.checked_add(1).ok_or_else(|| "remote-input-upload-class-count-overflow".to_string())?;
    Ok(())
}

struct RemoteBuildObservabilityInput<'a> {
    selected_route: &'a str,
    rejected_route_reasons: &'a [String],
    endpoint_id: Option<&'a str>,
    attempt_reason_codes: &'a [RemoteAttemptReasonCode],
    upload_summary: RemoteInputUploadPrivacySummary,
    resource_lease: Option<&'a RemoteResourceLease>,
    locality: Option<&'a RemoteVerifiedLocalitySummary>,
    admission: &'a RemoteOutputAdmissionReport,
    import_report: &'a RemoteOutputImportReport,
    non_claims: &'a [String],
}

pub type RemoteBuildObservabilityReportFn = fn(
    &str,
    &[String],
    Option<&str>,
    &[RemoteAttemptReasonCode],
    RemoteInputUploadPrivacySummary,
    Option<&RemoteResourceLease>,
    Option<&RemoteVerifiedLocalitySummary>,
    &RemoteOutputAdmissionReport,
    &RemoteOutputImportReport,
    &[String],
) -> Result<RemoteBuildObservabilityReport, String>;

pub const REMOTE_BUILD_OBSERVABILITY_REPORT: RemoteBuildObservabilityReportFn =
    |selected_route,
     rejected_route_reasons,
     endpoint_id,
     attempt_reason_codes,
     upload_summary,
     resource_lease,
     locality,
     admission,
     import_report,
     non_claims| {
        remote_build_observability_report_core(RemoteBuildObservabilityInput {
            selected_route,
            rejected_route_reasons,
            endpoint_id,
            attempt_reason_codes,
            upload_summary,
            resource_lease,
            locality,
            admission,
            import_report,
            non_claims,
        })
    };
pub use REMOTE_BUILD_OBSERVABILITY_REPORT as remote_build_observability_report;

fn remote_build_observability_report_core(
    input: RemoteBuildObservabilityInput<'_>,
) -> Result<RemoteBuildObservabilityReport, String> {
    let RemoteBuildObservabilityInput {
        selected_route,
        rejected_route_reasons,
        endpoint_id,
        attempt_reason_codes,
        upload_summary,
        resource_lease,
        locality,
        admission,
        import_report,
        non_claims,
    } = input;
    if selected_route.is_empty() {
        return Err("remote-observability-route-empty".to_string());
    }
    if admission.request_id != import_report.request_id {
        return Err("remote-observability-request-id-mismatch".to_string());
    }
    let outputs = import_report
        .outputs
        .iter()
        .map(|output| RemoteBuildOutputObservability {
            output_name: output.name.clone(),
            logical_path: output.logical_path.clone(),
            artifact_attestation_path: Some(output.artifact_attestation_path.clone()),
        })
        .collect::<Vec<_>>();
    if outputs.is_empty() {
        return Err("remote-observability-output-empty".to_string());
    }
    debug_assert_eq!(admission.request_id, import_report.request_id);
    debug_assert_eq!(outputs.len(), import_report.outputs.len());
    Ok(RemoteBuildObservabilityReport {
        selected_route: selected_route.to_string(),
        rejected_route_reasons: rejected_route_reasons.iter().map(|reason| bounded_untrusted_text(reason)).collect(),
        endpoint_id: endpoint_id.map(bounded_untrusted_text),
        attempt_reason_codes: attempt_reason_codes
            .iter()
            .take(MAX_REMOTE_STATUS_ITEMS)
            .map(|reason| reason.as_str().to_string())
            .collect(),
        upload_summary,
        resource_lease: resource_lease.cloned(),
        locality: locality.cloned(),
        transfer: import_report.transfer.clone(),
        trust_basis: admission.trust_basis.clone(),
        outputs,
        non_claims: non_claims.iter().map(|claim| bounded_untrusted_text(claim)).collect(),
    })
}

struct RemoteOperatorE2eRailInput<'a> {
    route_plan: &'a crate::realization_routing::RoutePlanReport,
    transcript: &'a RemoteStdioTranscript,
    upload_summary: RemoteInputUploadPrivacySummary,
    admission: &'a RemoteOutputAdmissionReport,
    import_report: &'a RemoteOutputImportReport,
    status: RemoteCoordinatorStatusSnapshot,
    non_claims: &'a [String],
}

pub type RemoteOperatorE2eRailReportFn = fn(
    &crate::realization_routing::RoutePlanReport,
    &RemoteStdioTranscript,
    RemoteInputUploadPrivacySummary,
    &RemoteOutputAdmissionReport,
    &RemoteOutputImportReport,
    RemoteCoordinatorStatusSnapshot,
    &[String],
) -> Result<RemoteOperatorE2eRailReport, String>;

pub const REMOTE_OPERATOR_E2E_RAIL_REPORT: RemoteOperatorE2eRailReportFn =
    |route_plan, transcript, upload_summary, admission, import_report, status, non_claims| {
        remote_operator_e2e_rail_report_core(RemoteOperatorE2eRailInput {
            route_plan,
            transcript,
            upload_summary,
            admission,
            import_report,
            status,
            non_claims,
        })
    };
pub use REMOTE_OPERATOR_E2E_RAIL_REPORT as remote_operator_e2e_rail_report;

fn remote_operator_e2e_rail_report_core(
    input: RemoteOperatorE2eRailInput<'_>,
) -> Result<RemoteOperatorE2eRailReport, String> {
    let RemoteOperatorE2eRailInput {
        route_plan,
        transcript,
        upload_summary,
        admission,
        import_report,
        status,
        non_claims,
    } = input;
    if route_plan.selected_route != crate::realization_routing::RouteClass::P2pRemoteBuilder {
        return Err("remote-e2e-route-not-remote-builder".to_string());
    }
    validate_remote_operator_e2e_frames(&transcript.frames)?;
    validate_remote_operator_e2e_status(&status)?;
    let selected_route = route_plan.selected_route.as_str().to_string();
    let rejected_route_reasons =
        route_plan.rejected_routes.iter().map(|rejection| rejection.reason_code.clone()).collect::<Vec<_>>();
    let rail_non_claims = remote_operator_e2e_non_claims(route_plan.non_claim, non_claims);
    let resource_lease = status.resource_leases.first();
    let locality = status
        .queued_jobs
        .iter()
        .chain(status.active_jobs.iter())
        .chain(status.recent_jobs.iter())
        .find_map(|job| job.locality.as_ref());
    let build_observability = remote_build_observability_report_core(RemoteBuildObservabilityInput {
        selected_route: &selected_route,
        rejected_route_reasons: &rejected_route_reasons,
        endpoint_id: Some(&status.endpoint_id),
        attempt_reason_codes: &status.attempt_reason_codes,
        upload_summary: upload_summary.clone(),
        resource_lease,
        locality,
        admission,
        import_report,
        non_claims: &rail_non_claims,
    })?;
    let artifact_attestation_paths = import_report
        .outputs
        .iter()
        .map(|output| output.artifact_attestation_path.clone())
        .collect::<Vec<_>>();
    if artifact_attestation_paths.is_empty() {
        return Err("remote-e2e-artifact-attestation-empty".to_string());
    }
    debug_assert_eq!(artifact_attestation_paths.len(), import_report.outputs.len());
    debug_assert_eq!(build_observability.selected_route, selected_route);
    Ok(RemoteOperatorE2eRailReport {
        schema: REMOTE_OPERATOR_E2E_RAIL_SCHEMA,
        selected_route,
        selected_reason_code: route_plan.selected_reason_code.clone(),
        endpoint_id: Some(status.endpoint_id.clone()),
        phases: RemoteOperatorE2eRailPhases {
            route: REMOTE_E2E_ROUTE_PLANNED_PHASE.to_string(),
            handshake: REMOTE_E2E_HANDSHAKE_COMPLETE_PHASE.to_string(),
            input_sync: REMOTE_E2E_INPUT_SYNC_COMPLETE_PHASE.to_string(),
            execution: REMOTE_E2E_EXECUTION_COMPLETE_PHASE.to_string(),
            output_admission: REMOTE_E2E_OUTPUT_ADMISSION_COMPLETE_PHASE.to_string(),
        },
        upload_summary,
        transfer: import_report.transfer.clone(),
        trust_basis: admission.trust_basis.clone(),
        artifact_attestation_paths,
        status,
        build_report: build_observability,
        non_claims: rail_non_claims,
    })
}

fn validate_remote_operator_e2e_frames(frames: &[RemoteFrame]) -> Result<(), String> {
    require_remote_e2e_frame(frames, RemoteFrameKind::AuthOk, "remote-e2e-handshake-frame-missing")?;
    require_remote_e2e_frame(frames, RemoteFrameKind::MissingInputs, "remote-e2e-input-sync-frame-missing")?;
    require_remote_e2e_frame(frames, RemoteFrameKind::BuildStarted, "remote-e2e-build-started-frame-missing")?;
    require_remote_e2e_frame(frames, RemoteFrameKind::BuildFinished, "remote-e2e-build-finished-frame-missing")?;
    require_remote_e2e_frame(frames, RemoteFrameKind::OutputTransferDone, "remote-e2e-output-transfer-frame-missing")?;
    require_remote_e2e_frame(frames, RemoteFrameKind::Done, "remote-e2e-done-frame-missing")?;
    Ok(())
}

fn require_remote_e2e_frame(
    frames: &[RemoteFrame],
    required: RemoteFrameKind,
    error: &'static str,
) -> Result<(), String> {
    if frames.iter().any(|frame| frame.kind() == required) {
        return Ok(());
    }
    Err(error.to_string())
}

fn validate_remote_operator_e2e_status(status: &RemoteCoordinatorStatusSnapshot) -> Result<(), String> {
    if status.endpoint_id.is_empty() {
        return Err("remote-e2e-status-endpoint-empty".to_string());
    }
    if status.worker_count == 0 {
        return Err("remote-e2e-status-worker-empty".to_string());
    }
    if status.tickets.iter().any(|ticket| ticket.verifier_key_id.is_empty()) {
        return Err("remote-e2e-status-ticket-key-id-empty".to_string());
    }
    Ok(())
}

fn remote_operator_e2e_non_claims(route_non_claim: &str, non_claims: &[String]) -> Vec<String> {
    let mut combined = Vec::with_capacity(non_claims.len().saturating_add(2));
    combined.push(bounded_untrusted_text(route_non_claim));
    combined.push(REMOTE_OPERATOR_E2E_RAIL_NON_CLAIM.to_string());
    combined.extend(non_claims.iter().map(|claim| bounded_untrusted_text(claim)));
    combined
}

pub fn validate_remote_source_upload_record(
    record: &crate::source_bundle::SourceRecord,
    store_prefix: &str,
    logical_store_path: impl AsRef<str>,
    declared_content_blake3: Option<&str>,
) -> Result<RemoteSourceUploadReadiness, String> {
    let logical_store_path = logical_store_path.as_ref();
    if record.kind != crate::source_bundle::SourceRecordKind::ToolchainSourceRoot {
        return Err("remote-input-source-kind-unsupported".to_string());
    }
    let Some(record_store_prefix) = record.store_prefix.as_deref() else {
        return Err("remote-input-source-store-prefix-missing".to_string());
    };
    if record_store_prefix != store_prefix {
        return Err("remote-input-source-store-prefix-mismatch".to_string());
    }
    if record.metadata.get("store_path").map(String::as_str) != Some(logical_store_path) {
        return Err("remote-input-source-identity-mismatch".to_string());
    }
    if record.files.is_empty() || record.content_blake3.is_empty() {
        return Err("remote-input-source-readiness-missing".to_string());
    }
    if let Some(expected) = declared_content_blake3
        && expected != record.content_blake3
    {
        return Err("remote-input-source-digest-stale".to_string());
    }
    debug_assert_eq!(record_store_prefix, store_prefix);
    debug_assert_eq!(record.metadata.get("store_path").map(String::as_str), Some(logical_store_path));
    Ok(RemoteSourceUploadReadiness {
        identity: record.identity.clone(),
        source_kind: "toolchain-source-root".to_string(),
        ready_class: "verified-imported-source-state".to_string(),
        content_blake3: record.content_blake3.clone(),
        store_prefix: record_store_prefix.to_string(),
    })
}

fn remote_input_ref_host_path(
    store: &crunch_store::StoreHandle,
    store_prefix: &str,
    input_ref: impl AsRef<str>,
) -> Result<PathBuf, String> {
    let input_ref = input_ref.as_ref();
    let logical_path = PathBuf::from(input_ref);
    if logical_path.exists() {
        return Ok(logical_path);
    }
    let store_path = parse_remote_output_store_path(input_ref, store_prefix)?;
    let store_host_path = PathBuf::from(store_path.to_absolute_path_with_prefix(store.output_dir_str()));
    if store_host_path.exists() {
        return Ok(store_host_path);
    }
    Err("remote-input-source-path-missing".to_string())
}

async fn render_remote_input_nar_payload_for_ref(
    store: &crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
    input_ref: &str,
    source_state_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    match remote_input_ref_host_path(store, &request.store_prefix, input_ref) {
        Ok(host_path) => render_remote_input_nar_payload(store, &host_path).await,
        Err(err) if err == "remote-input-source-path-missing" => {
            render_remote_input_nar_payload_from_source_state(store, request, input_ref, source_state_dir).await
        }
        Err(err) => Err(err),
    }
}

async fn render_remote_input_nar_payload_from_source_state(
    store: &crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
    input_ref: &str,
    source_state_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let source_state_dir = source_state_dir.ok_or_else(|| "remote-input-source-path-missing".to_string())?;
    let scratch = tempfile::Builder::new()
        .prefix("mantle-remote-source-input-")
        .tempdir()
        .map_err(|err| format!("remote-input-source-state-scratch-failed: {err}"))?;
    let is_materialized = crate::source_bundle::materialize_imported_source_record_for_store_path(
        source_state_dir,
        &request.store_prefix,
        input_ref,
        scratch.path(),
    )
    .map_err(|err| format!("remote-input-source-state-materialize-failed: {err}"))?;
    if !is_materialized {
        return Err("remote-input-source-path-missing".to_string());
    }
    render_remote_input_nar_payload(store, scratch.path()).await
}

async fn render_remote_input_nar_payload(
    store: &crunch_store::StoreHandle,
    host_path: &Path,
) -> Result<Vec<u8>, String> {
    let node = snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(
        store.blob_service(),
        store.directory_service(),
        host_path,
        None,
    )
    .await
    .map_err(|err| format!("remote-input-source-ingest-failed: {err}"))?;
    let mut writer = BoundedAsyncVecWriter::new(MAX_REMOTE_INLINE_NAR_PAYLOAD_BYTES);
    store
        .render_nar(&node, &mut writer)
        .await
        .map_err(|err| format!("remote-input-nar-render-failed: {err}"))?;
    Ok(writer.into_inner())
}

async fn materialize_remote_input_upload(
    store: &crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
    upload: &RemoteInputUpload,
) -> Result<(), String> {
    if upload.streamed {
        if !upload.artifacts.is_empty() {
            return Err("remote-streamed-input-inline-artifact-forbidden".to_string());
        }
        for input_ref in &upload.refs {
            remote_input_ref_host_path(store, &request.store_prefix, input_ref)
                .map_err(|_| "remote-streamed-input-materialization-missing".to_string())?;
        }
        assert!(upload.artifacts.is_empty());
        assert!(upload.refs.len() <= request.input_refs.len());
        return Ok(());
    }
    validate_remote_input_upload_artifacts(
        &request.request_id,
        &upload.refs,
        &request.source_input_refs,
        &upload.artifacts,
    )?;
    for artifact in &upload.artifacts {
        materialize_remote_input_artifact(store, &request.store_prefix, artifact).await?;
    }
    Ok(())
}

async fn materialize_remote_input_artifact(
    store: &crunch_store::StoreHandle,
    store_prefix: &str,
    artifact: &RemoteInputUploadArtifact,
) -> Result<(), String> {
    let store_path = parse_remote_output_store_path(&artifact.input_ref, store_prefix)?;
    let mut reader = std::io::Cursor::new(artifact.payload.as_slice());
    let (node, _nar_sha256, nar_size) =
        snix_store::nar::ingest_nar_and_hash(store.blob_service(), store.directory_service(), &mut reader, &None)
            .await
            .map_err(|err| format!("remote-input-nar-ingest-failed: {err}"))?;
    if nar_size != artifact.size_bytes {
        return Err("remote-input-nar-size-mismatch".to_string());
    }
    export_remote_input_node(store, &store_path, &node).await
}

async fn materialize_streamed_remote_inputs(
    store: &crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
    manifest: &crunch_build::distributed::CanonicalRemoteTransferManifest,
    receiver_root: &Path,
) -> Result<(), String> {
    let expected_ids = request
        .input_refs
        .iter()
        .map(|input_ref| remote_input_nar_artifact_id(input_ref))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let actual_ids = manifest
        .manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_id.clone())
        .collect::<BTreeSet<_>>();
    if actual_ids != expected_ids {
        return Err("remote-streamed-input-manifest-set-mismatch".to_string());
    }
    for input_ref in &request.input_refs {
        let artifact_id = remote_input_nar_artifact_id(input_ref)?;
        let descriptor = manifest
            .manifest
            .artifacts
            .iter()
            .find(|artifact| artifact.artifact_id == artifact_id)
            .ok_or_else(|| "remote-streamed-input-artifact-missing".to_string())?;
        if descriptor.artifact_kind != RemoteTransferArtifactKind::Nar {
            return Err("remote-streamed-input-artifact-kind-mismatch".to_string());
        }
        let reader = crate::remote_transfer::open_remote_transfer_received_artifact(receiver_root, &artifact_id)
            .map_err(|err| format!("remote-streamed-input-open-failed: {err}"))?;
        let mut reader = tokio::fs::File::from_std(reader);
        let (node, nar_sha256, nar_size) =
            snix_store::nar::ingest_nar_and_hash(store.blob_service(), store.directory_service(), &mut reader, &None)
                .await
                .map_err(|err| format!("remote-streamed-input-nar-ingest-failed: {err}"))?;
        if nar_size != descriptor.size_bytes {
            return Err("remote-streamed-input-nar-size-mismatch".to_string());
        }
        if descriptor
            .nar_sha256_hex
            .as_deref()
            .is_some_and(|expected| expected != data_encoding::HEXLOWER.encode(&nar_sha256))
        {
            return Err("remote-streamed-input-nar-sha256-mismatch".to_string());
        }
        let store_path = parse_remote_output_store_path(input_ref, &request.store_prefix)?;
        export_remote_input_node(store, &store_path, &node).await?;
    }
    assert_eq!(expected_ids.len(), request.input_refs.len());
    assert!(manifest.total_bytes <= request.transfer_policy.unwrap_or_default().total_bytes_max);
    Ok(())
}

async fn export_remote_input_node(
    store: &crunch_store::StoreHandle,
    store_path: &StorePath<String>,
    node: &snix_castore::Node,
) -> Result<(), String> {
    let host_path = store_path.to_absolute_path_with_prefix(store.output_dir_str());
    if !Path::new(&host_path).exists() {
        crunch_store::export_castore_to_disk(node, &host_path, &store.blob_service(), &store.directory_service())
            .await
            .map_err(|err| format!("remote-input-export-failed: {err}"))?;
    }
    Ok(())
}

struct PreparedRemoteProductionOutput {
    result: RemoteBuildFinished,
    transfer: crate::remote_transfer::PreparedRemoteTransfer,
}

async fn prepare_remote_production_output(
    executor: &RemoteLocalBuildExecutor,
    request: &ConcreteBuildRequest,
    execution: &RemoteExecutionOutcome,
) -> Result<PreparedRemoteProductionOutput, String> {
    let attempt = request.production_attempt.as_ref().ok_or_else(|| "remote-production-attempt-missing".to_string())?;
    let policy = request.transfer_policy.ok_or_else(|| "remote-production-transfer-policy-missing".to_string())?;
    let store = open_remote_local_executor_store(executor).await?;
    let spool_dir = executor.state_dir.join(REMOTE_TRANSFER_SPOOL_DIR).join(&request.request_id).join("outputs");
    let mut outputs = sign_remote_execution_outputs(&execution.outputs, &executor.signing_key_id())?;
    let mut artifacts = Vec::with_capacity(outputs.len().saturating_mul(REMOTE_TRANSFER_ARTIFACTS_PER_PATHINFO_OUTPUT));
    for output in &mut outputs {
        let path_info =
            output.path_info.as_ref().ok_or_else(|| "remote-production-output-pathinfo-missing".to_string())?;
        let pathinfo = crate::remote_transfer::prepare_pathinfo_transfer_artifact_with_id(
            path_info,
            remote_output_pathinfo_artifact_id(&output.name, &output.logical_path)?,
            &spool_dir,
            true,
            policy,
        )?;
        let nar = crate::remote_transfer::prepare_nar_transfer_artifact(
            &store,
            path_info,
            remote_output_nar_artifact_id(&output.name, &output.logical_path)?,
            &spool_dir,
            true,
            policy,
        )
        .await?;
        output.nar_payload_digest_blake3 = Some(nar.descriptor.digest_blake3.as_str().to_string());
        output.nar_payload_size_bytes = Some(nar.descriptor.size_bytes);
        artifacts.push(pathinfo);
        artifacts.push(nar);
    }
    let output_digest_blake3 = remote_produced_outputs_content_digest(&outputs);
    let requested_content_blake3 =
        RemoteTransferDigest::new(output_digest_blake3.clone()).map_err(|reason| reason.as_str().to_string())?;
    let transfer = crate::remote_transfer::prepare_remote_transfer(
        crate::remote_transfer::RemoteTransferBinding {
            job_id: attempt.job_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            fence_generation: attempt.fence_generation,
            store_prefix: request.store_prefix.clone(),
            requested_content_blake3,
        },
        policy,
        artifacts,
    )?;
    let result = RemoteBuildFinished {
        request_id: request.request_id.clone(),
        output_digest_blake3,
        builder_signing_key_id: executor.signing_key_id(),
        store_prefix: request.store_prefix.clone(),
        outputs,
    };
    validate_remote_produced_outputs(request, &result)?;
    assert!(!transfer.manifest.manifest.artifacts.is_empty());
    assert!(transfer.manifest.total_bytes > 0);
    Ok(PreparedRemoteProductionOutput { result, transfer })
}

async fn open_remote_local_executor_store(
    executor: &RemoteLocalBuildExecutor,
) -> Result<crunch_store::StoreHandle, String> {
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: executor.state_dir.clone(),
        output_dir: executor.output_dir.clone(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: executor.store_prefix.clone(),
        base_state_dirs: Vec::new(),
    })
    .await
    .map_err(|err| format!("remote-local-executor-open-store: {err}"))
}

struct BoundedAsyncVecWriter {
    bytes: Vec<u8>,
    max_bytes: usize,
}

impl BoundedAsyncVecWriter {
    fn new(max_bytes: usize) -> Self {
        Self {
            bytes: Vec::new(),
            max_bytes,
        }
    }

    fn into_inner(self) -> Vec<u8> {
        self.bytes
    }
}

impl AsyncWrite for BoundedAsyncVecWriter {
    fn poll_write(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let remaining = self.max_bytes.saturating_sub(self.bytes.len());
        if buf.len() > remaining {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "remote output NAR payload exceeds frame limit",
            )));
        }
        self.bytes.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

fn validate_local_executor_input_refs(request: &ConcreteBuildRequest, input_refs: &[String]) -> Result<(), String> {
    if input_refs.len() > MAX_REMOTE_INPUT_REFS || request.input_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"));
    }
    let requested = request.input_refs.iter().collect::<BTreeSet<_>>();
    let uploaded = input_refs.iter().collect::<BTreeSet<_>>();
    if requested.len() != request.input_refs.len() || uploaded.len() != input_refs.len() {
        return Err("remote-local-executor-input-refs-duplicate".to_string());
    }
    if requested != uploaded {
        return Err("remote-local-executor-input-refs-mismatch".to_string());
    }
    Ok(())
}

fn validate_remote_execution_plan(plan: &RemoteExecutablePlan) -> Result<(), String> {
    if plan.request_id.is_empty() || plan.plan_digest_blake3.is_empty() {
        return Err("remote-executor-plan-identity-empty".to_string());
    }
    if !is_blake3_hex_digest(&plan.plan_digest_blake3) {
        return Err("remote-executor-plan-digest-invalid".to_string());
    }
    validate_command_args(&plan.command_args)?;
    validate_command_env(&plan.command_env)?;
    validate_expected_outputs(&plan.expected_outputs, &plan.store_prefix)
}

fn fixture_execution_output(
    plan: &RemoteExecutablePlan,
    input_refs: &[String],
    expected: &RemoteExpectedOutput,
) -> RemoteExecutionOutput {
    let content_digest_blake3 = remote_fixture_output_content_digest(plan, input_refs, expected);
    let artifact_attestation_digest_blake3 =
        remote_fixture_artifact_attestation_digest(plan, expected, &content_digest_blake3);
    RemoteExecutionOutput {
        name: expected.name.clone(),
        logical_path: remote_fixture_output_logical_path(plan, input_refs, expected, &content_digest_blake3),
        content_digest_blake3,
        size_bytes: REMOTE_LOOPBACK_OUTPUT_BYTES,
        artifact_attestation_digest_blake3,
        path_info: None,
        nar_payload: None,
    }
}

fn remote_fixture_output_content_digest(
    plan: &RemoteExecutablePlan,
    input_refs: &[String],
    expected: &RemoteExpectedOutput,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "fixture-plan", &plan.plan_digest_blake3);
    hash_labeled_str(&mut hasher, "output-name", &expected.name);
    match &expected.logical_path {
        Some(logical_path) => hash_labeled_str(&mut hasher, "output-path", logical_path),
        None => hash_labeled_str(&mut hasher, "output-path-state", "content-addressed-unknown"),
    }
    for input_ref in input_refs {
        hash_labeled_str(&mut hasher, "input-ref", input_ref);
    }
    hasher.finalize().to_hex().to_string()
}

fn remote_fixture_output_logical_path(
    plan: &RemoteExecutablePlan,
    _input_refs: &[String],
    expected: &RemoteExpectedOutput,
    content_digest_blake3: &str,
) -> String {
    match &expected.logical_path {
        Some(logical_path) => logical_path.clone(),
        None => {
            let store_hash = content_digest_blake3.get(..STORE_PATH_HASH_CHARS).unwrap_or(content_digest_blake3);
            format!("{}/{}-{}", plan.store_prefix, store_hash, expected.name)
        }
    }
}

fn remote_fixture_artifact_attestation_digest(
    plan: &RemoteExecutablePlan,
    expected: &RemoteExpectedOutput,
    content_digest_blake3: &str,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "fixture-artifact-plan", &plan.plan_digest_blake3);
    hash_labeled_str(&mut hasher, "output-name", &expected.name);
    hash_labeled_str(&mut hasher, "content-digest", content_digest_blake3);
    hasher.finalize().to_hex().to_string()
}

fn sum_remote_execution_output_sizes(outputs: &[RemoteExecutionOutput]) -> Result<u64, String> {
    let mut total = 0_u64;
    for output in outputs {
        total = total
            .checked_add(output.size_bytes)
            .ok_or_else(|| "remote-execution-output-size-overflow".to_string())?;
    }
    Ok(total)
}

fn remote_execution_outputs_digest(outputs: &[RemoteExecutionOutput]) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    for output in outputs {
        let (nar_payload_digest_blake3, nar_payload_size_bytes) = remote_nar_payload_summary(output)?;
        hash_remote_output_digest_fields(&mut hasher, RemoteOutputDigestFields {
            name: &output.name,
            logical_path: &output.logical_path,
            content_digest_blake3: &output.content_digest_blake3,
            artifact_attestation_digest_blake3: &output.artifact_attestation_digest_blake3,
            size_bytes: output.size_bytes,
            nar_payload_digest_blake3: nar_payload_digest_blake3.as_deref(),
            nar_payload_size_bytes,
        });
    }
    Ok(hasher.finalize().to_hex().to_string())
}

struct RemoteOutputDigestFields<'a> {
    name: &'a str,
    logical_path: &'a str,
    content_digest_blake3: &'a str,
    artifact_attestation_digest_blake3: &'a str,
    size_bytes: u64,
    nar_payload_digest_blake3: Option<&'a str>,
    nar_payload_size_bytes: Option<u64>,
}

fn hash_remote_output_digest_fields(hasher: &mut blake3::Hasher, fields: RemoteOutputDigestFields<'_>) {
    hash_labeled_str(hasher, "output-name", fields.name);
    hash_labeled_str(hasher, "output-path", fields.logical_path);
    hash_labeled_str(hasher, "content-digest", fields.content_digest_blake3);
    hash_labeled_str(hasher, "artifact-digest", fields.artifact_attestation_digest_blake3);
    hash_labeled_str(hasher, "size-bytes", fields.size_bytes.to_string());
    hash_optional_digest_field(hasher, "nar-payload-digest", fields.nar_payload_digest_blake3);
    hash_optional_u64_field(hasher, "nar-payload-size", fields.nar_payload_size_bytes);
}

fn hash_optional_digest_field(hasher: &mut blake3::Hasher, label: &str, value: Option<&str>) {
    match value {
        Some(value) => hash_labeled_str(hasher, label, value),
        None => hash_labeled_str(hasher, label, "<absent>"),
    }
}

fn hash_optional_u64_field(hasher: &mut blake3::Hasher, label: &str, value: Option<u64>) {
    match value {
        Some(value) => hash_labeled_str(hasher, label, value.to_string()),
        None => hash_labeled_str(hasher, label, "<absent>"),
    }
}

fn sign_remote_execution_outputs(
    outputs: &[RemoteExecutionOutput],
    signing_key_id: &str,
) -> Result<Vec<RemoteProducedOutput>, String> {
    outputs
        .iter()
        .map(|output| {
            let (nar_payload_digest_blake3, nar_payload_size_bytes) = remote_nar_payload_summary(output)?;
            Ok(RemoteProducedOutput {
                name: output.name.clone(),
                logical_path: output.logical_path.clone(),
                content_digest_blake3: output.content_digest_blake3.clone(),
                size_bytes: output.size_bytes,
                path_info_signing_key_id: signing_key_id.to_string(),
                artifact_attestation_digest_blake3: output.artifact_attestation_digest_blake3.clone(),
                path_info: output.path_info.clone(),
                nar_payload_digest_blake3,
                nar_payload_size_bytes,
            })
        })
        .collect()
}

fn remote_nar_payload_summary(output: &RemoteExecutionOutput) -> Result<(Option<String>, Option<u64>), String> {
    let Some(payload) = &output.nar_payload else {
        return Ok((None, None));
    };
    let size_bytes = remote_payload_size_bytes(payload.len())?;
    if size_bytes != output.size_bytes {
        return Err("remote-output-nar-summary-size-mismatch".to_string());
    }
    Ok((Some(blake3::hash(payload).to_hex().to_string()), Some(size_bytes)))
}

fn plan_output_transfer_artifacts(
    request_id: &str,
    outputs: &[RemoteProducedOutput],
    execution_outputs: &[RemoteExecutionOutput],
) -> Result<Vec<RemoteOutputTransferArtifact>, String> {
    if request_id.is_empty() {
        return Err("remote-output-transfer-request-id-empty".to_string());
    }
    if outputs.len() > MAX_REMOTE_TRANSFER_ARTIFACTS {
        return Err(format!("remote-output-transfer-artifact-count-exceeds-{MAX_REMOTE_TRANSFER_ARTIFACTS}"));
    }
    let execution_by_name = execution_output_by_name(execution_outputs)?;
    let mut artifacts = Vec::with_capacity(outputs.len().saturating_mul(REMOTE_TRANSFER_ARTIFACTS_PER_PATHINFO_OUTPUT));
    for output in outputs {
        if let Some(path_info) = &output.path_info {
            artifacts.push(pathinfo_json_transfer_artifact(request_id, output, path_info)?);
        }
        if output.nar_payload_digest_blake3.is_some() {
            let execution = execution_by_name
                .get(output.name.as_str())
                .ok_or_else(|| "remote-output-transfer-execution-output-missing".to_string())?;
            artifacts.push(nar_transfer_artifact(request_id, output, execution)?);
        }
    }
    validate_remote_output_transfer_artifacts(request_id, outputs, &artifacts)?;
    debug_assert!(artifacts.len() <= MAX_REMOTE_TRANSFER_ARTIFACTS);
    debug_assert!(artifacts.iter().all(|artifact| artifact.request_id == request_id));
    Ok(artifacts)
}

fn execution_output_by_name(
    outputs: &[RemoteExecutionOutput],
) -> Result<BTreeMap<&str, &RemoteExecutionOutput>, String> {
    let mut by_name = BTreeMap::new();
    for output in outputs {
        if by_name.insert(output.name.as_str(), output).is_some() {
            return Err("remote-output-transfer-execution-output-duplicate".to_string());
        }
    }
    Ok(by_name)
}

fn nar_transfer_artifact(
    request_id: &str,
    output: &RemoteProducedOutput,
    execution: &RemoteExecutionOutput,
) -> Result<RemoteOutputTransferArtifact, String> {
    if execution.name != output.name {
        return Err("remote-output-transfer-nar-output-mismatch".to_string());
    }
    let payload = execution
        .nar_payload
        .clone()
        .ok_or_else(|| "remote-output-transfer-nar-payload-missing".to_string())?;
    let size_bytes = remote_payload_size_bytes(payload.len())?;
    if Some(size_bytes) != output.nar_payload_size_bytes {
        return Err("remote-output-transfer-nar-size-mismatch".to_string());
    }
    let digest_blake3 = blake3::hash(&payload).to_hex().to_string();
    if Some(digest_blake3.as_str()) != output.nar_payload_digest_blake3.as_deref() {
        return Err("remote-output-transfer-nar-digest-mismatch".to_string());
    }
    let artifact = RemoteOutputTransferArtifact {
        request_id: request_id.to_string(),
        output_name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        artifact_kind: RemoteOutputTransferArtifactKind::Nar,
        digest_blake3,
        size_bytes,
        payload,
    };
    debug_assert_eq!(artifact.output_name, execution.name);
    debug_assert_eq!(artifact.size_bytes, execution.size_bytes);
    Ok(artifact)
}

fn pathinfo_json_transfer_artifact(
    request_id: &str,
    output: &RemoteProducedOutput,
    path_info: &PathInfo,
) -> Result<RemoteOutputTransferArtifact, String> {
    let payload = serialize_remote_pathinfo_payload(path_info)?;
    let size_bytes = remote_payload_size_bytes(payload.len())?;
    if size_bytes > MAX_REMOTE_TRANSFER_TOTAL_BYTES {
        return Err("remote-output-transfer-artifact-too-large".to_string());
    }
    Ok(RemoteOutputTransferArtifact {
        request_id: request_id.to_string(),
        output_name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        artifact_kind: RemoteOutputTransferArtifactKind::PathInfoJson,
        digest_blake3: blake3::hash(&payload).to_hex().to_string(),
        size_bytes,
        payload,
    })
}

fn serialize_remote_pathinfo_payload(path_info: &PathInfo) -> Result<Vec<u8>, String> {
    serde_json::to_vec(path_info).map_err(|err| format!("remote-output-pathinfo-json-serialize-failed: {err}"))
}

fn remote_payload_size_bytes(payload_len: usize) -> Result<u64, String> {
    u64::try_from(payload_len).map_err(|_| "remote-output-transfer-payload-size-overflow".to_string())
}

pub fn validate_remote_output_transfer_artifacts(
    request_id: &str,
    outputs: &[RemoteProducedOutput],
    artifacts: &[RemoteOutputTransferArtifact],
) -> Result<(), String> {
    let expected = expected_remote_output_transfer_artifacts(outputs)?;
    validate_remote_output_transfer_artifact_list_shape(request_id, artifacts)?;
    let mut seen = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for artifact in artifacts {
        let key = validate_remote_output_transfer_artifact(request_id, artifact, &expected)?;
        if !seen.insert(key) {
            return Err("remote-output-transfer-artifact-duplicate".to_string());
        }
        total_bytes = total_bytes
            .checked_add(artifact.size_bytes)
            .ok_or_else(|| "remote-output-transfer-total-bytes-overflow".to_string())?;
        if total_bytes > MAX_REMOTE_TRANSFER_TOTAL_BYTES {
            return Err("remote-output-transfer-total-bytes-exceeded".to_string());
        }
    }
    for expected_key in expected.keys() {
        if !seen.contains(expected_key) {
            return Err("remote-output-transfer-artifact-missing".to_string());
        }
    }
    debug_assert_eq!(seen.len(), expected.len());
    debug_assert!(total_bytes <= MAX_REMOTE_TRANSFER_TOTAL_BYTES);
    Ok(())
}

fn validate_remote_output_transfer_artifact_list_shape(
    request_id: &str,
    artifacts: &[RemoteOutputTransferArtifact],
) -> Result<(), String> {
    if request_id.is_empty() {
        return Err("remote-output-transfer-request-id-empty".to_string());
    }
    if artifacts.len() > MAX_REMOTE_TRANSFER_ARTIFACTS {
        return Err(format!("remote-output-transfer-artifact-count-exceeds-{MAX_REMOTE_TRANSFER_ARTIFACTS}"));
    }
    Ok(())
}

fn validate_remote_output_transfer_artifact(
    request_id: &str,
    artifact: &RemoteOutputTransferArtifact,
    expected: &BTreeMap<(String, RemoteOutputTransferArtifactKind), ExpectedRemoteOutputTransferArtifact>,
) -> Result<(String, RemoteOutputTransferArtifactKind), String> {
    if artifact.request_id != request_id {
        return Err("remote-output-transfer-artifact-request-id-mismatch".to_string());
    }
    if artifact.output_name.is_empty() || artifact.logical_path.is_empty() {
        return Err("remote-output-transfer-artifact-identity-empty".to_string());
    }
    if !is_blake3_hex_digest(&artifact.digest_blake3) {
        return Err("remote-output-transfer-artifact-digest-invalid".to_string());
    }
    let payload_size_bytes = remote_payload_size_bytes(artifact.payload.len())?;
    if payload_size_bytes != artifact.size_bytes {
        return Err("remote-output-transfer-artifact-size-mismatch".to_string());
    }
    if blake3::hash(&artifact.payload).to_hex().to_string() != artifact.digest_blake3 {
        return Err("remote-output-transfer-artifact-digest-mismatch".to_string());
    }
    let key = (artifact.output_name.clone(), artifact.artifact_kind);
    let expected_artifact =
        expected.get(&key).ok_or_else(|| "remote-output-transfer-artifact-unexpected".to_string())?;
    if expected_artifact.output_name != artifact.output_name || expected_artifact.logical_path != artifact.logical_path
    {
        return Err("remote-output-transfer-artifact-output-mismatch".to_string());
    }
    if expected_artifact.artifact_kind != artifact.artifact_kind {
        return Err("remote-output-transfer-artifact-kind-mismatch".to_string());
    }
    if expected_artifact.digest_blake3 != artifact.digest_blake3 {
        return Err("remote-output-transfer-artifact-expected-digest-mismatch".to_string());
    }
    if expected_artifact.size_bytes != artifact.size_bytes {
        return Err("remote-output-transfer-artifact-expected-size-mismatch".to_string());
    }
    debug_assert_eq!(expected_artifact.digest_blake3, artifact.digest_blake3);
    debug_assert_eq!(expected_artifact.size_bytes, artifact.size_bytes);
    Ok(key)
}

fn expected_remote_output_transfer_artifacts(
    outputs: &[RemoteProducedOutput],
) -> Result<BTreeMap<(String, RemoteOutputTransferArtifactKind), ExpectedRemoteOutputTransferArtifact>, String> {
    let mut expected = BTreeMap::new();
    for output in outputs {
        if let Some(path_info) = &output.path_info {
            insert_expected_transfer_artifact(&mut expected, expected_pathinfo_transfer_artifact(output, path_info)?)?;
        }
        if output.nar_payload_digest_blake3.is_some() || output.nar_payload_size_bytes.is_some() {
            insert_expected_transfer_artifact(&mut expected, expected_nar_transfer_artifact(output)?)?;
        }
    }
    Ok(expected)
}

fn insert_expected_transfer_artifact(
    expected: &mut BTreeMap<(String, RemoteOutputTransferArtifactKind), ExpectedRemoteOutputTransferArtifact>,
    artifact: ExpectedRemoteOutputTransferArtifact,
) -> Result<(), String> {
    let key = (artifact.output_name.clone(), artifact.artifact_kind);
    if expected.insert(key, artifact).is_some() {
        return Err("remote-output-transfer-expected-artifact-duplicate".to_string());
    }
    Ok(())
}

fn expected_pathinfo_transfer_artifact(
    output: &RemoteProducedOutput,
    path_info: &PathInfo,
) -> Result<ExpectedRemoteOutputTransferArtifact, String> {
    let payload = serialize_remote_pathinfo_payload(path_info)?;
    let size_bytes = remote_payload_size_bytes(payload.len())?;
    Ok(ExpectedRemoteOutputTransferArtifact {
        output_name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        artifact_kind: RemoteOutputTransferArtifactKind::PathInfoJson,
        digest_blake3: blake3::hash(&payload).to_hex().to_string(),
        size_bytes,
    })
}

fn expected_nar_transfer_artifact(
    output: &RemoteProducedOutput,
) -> Result<ExpectedRemoteOutputTransferArtifact, String> {
    let digest_blake3 = output
        .nar_payload_digest_blake3
        .clone()
        .ok_or_else(|| "remote-output-transfer-nar-digest-missing".to_string())?;
    let size_bytes =
        output.nar_payload_size_bytes.ok_or_else(|| "remote-output-transfer-nar-size-missing".to_string())?;
    if output.path_info.is_none() {
        return Err("remote-output-transfer-nar-pathinfo-missing".to_string());
    }
    Ok(ExpectedRemoteOutputTransferArtifact {
        output_name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        artifact_kind: RemoteOutputTransferArtifactKind::Nar,
        digest_blake3,
        size_bytes,
    })
}

fn remote_produced_outputs_content_digest(outputs: &[RemoteProducedOutput]) -> String {
    let mut hasher = blake3::Hasher::new();
    for output in outputs {
        hash_remote_output_digest_fields(&mut hasher, RemoteOutputDigestFields {
            name: &output.name,
            logical_path: &output.logical_path,
            content_digest_blake3: &output.content_digest_blake3,
            artifact_attestation_digest_blake3: &output.artifact_attestation_digest_blake3,
            size_bytes: output.size_bytes,
            nar_payload_digest_blake3: output.nar_payload_digest_blake3.as_deref(),
            nar_payload_size_bytes: output.nar_payload_size_bytes,
        });
    }
    hasher.finalize().to_hex().to_string()
}

fn validate_remote_execution_outcome(
    outcome: &RemoteExecutionOutcome,
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
) -> Result<(), String> {
    if outcome.request_id != request.request_id {
        return Err("remote-execution-request-id-mismatch".to_string());
    }
    if outcome.plan_digest_blake3 != plan.plan_digest_blake3 {
        return Err("remote-execution-plan-digest-mismatch".to_string());
    }
    if !is_blake3_hex_digest(&outcome.output_digest_blake3) {
        return Err("remote-execution-output-digest-invalid".to_string());
    }
    if remote_execution_outputs_digest(&outcome.outputs)? != outcome.output_digest_blake3 {
        return Err("remote-execution-output-digest-mismatch".to_string());
    }
    if sum_remote_execution_output_sizes(&outcome.outputs)? != outcome.output_size_bytes {
        return Err("remote-execution-output-size-mismatch".to_string());
    }
    validate_execution_outputs_match_expected(&outcome.outputs, &request.expected_outputs, &request.store_prefix)
}

fn validate_remote_execution_output_pathinfos(
    outputs: &[RemoteExecutionOutput],
    signing_key_id: &str,
    store_prefix: impl AsRef<str>,
) -> Result<(), String> {
    let store_prefix = store_prefix.as_ref();
    if signing_key_id.is_empty() {
        return Err("remote-execution-builder-signing-key-empty".to_string());
    }
    if !store_prefix.starts_with('/') {
        return Err("remote-execution-store-prefix-not-absolute".to_string());
    }

    for output in outputs {
        if let Some(path_info) = &output.path_info {
            validate_remote_execution_output_pathinfo(output, path_info, signing_key_id, store_prefix)?;
        }
    }
    Ok(())
}

fn validate_remote_execution_output_pathinfo(
    output: &RemoteExecutionOutput,
    path_info: &PathInfo,
    signing_key_id: &str,
    store_prefix: impl AsRef<str>,
) -> Result<(), String> {
    let store_prefix = store_prefix.as_ref();
    let store_path = parse_remote_output_store_path(&output.logical_path, store_prefix)?;
    validate_remote_pathinfo_binding(RemotePathInfoBindingInput {
        output_name: &output.name,
        artifact_attestation_digest_blake3: &output.artifact_attestation_digest_blake3,
        path_info,
        store_path: &store_path,
        store_prefix,
        signing_key_id,
        error_labels: RemotePathInfoErrorLabels {
            store_path_mismatch: "remote-execution-output-pathinfo-store-path-mismatch",
            unsigned: "remote-execution-output-pathinfo-unsigned",
            signing_key_mismatch: "remote-execution-output-pathinfo-signing-key-mismatch",
            artifact_digest_failed: "remote-execution-output-artifact-attestation-digest-failed",
            artifact_digest_mismatch: "remote-execution-output-artifact-attestation-digest-mismatch",
        },
    })
}

fn expected_output_path_map(expected_outputs: &[RemoteExpectedOutput]) -> BTreeMap<&str, Option<&str>> {
    expected_outputs
        .iter()
        .map(|output| (output.name.as_str(), output.logical_path.as_deref()))
        .collect()
}

fn validate_output_path_against_expected(
    output_name: &str,
    logical_path: impl AsRef<str>,
    expected: &BTreeMap<&str, Option<&str>>,
    store_prefix: &str,
    mismatch_label: impl Into<String> + Copy,
) -> Result<(), String> {
    let logical_path = logical_path.as_ref();
    match expected.get(output_name) {
        Some(Some(expected_logical_path)) => {
            if *expected_logical_path != logical_path {
                return Err(mismatch_label.into());
            }
        }
        Some(None) => {
            if logical_path.is_empty() || !logical_path.starts_with(store_prefix) {
                return Err(mismatch_label.into());
            }
        }
        None => return Err(mismatch_label.into()),
    }
    Ok(())
}

fn validate_execution_outputs_match_expected(
    outputs: &[RemoteExecutionOutput],
    expected_outputs: &[RemoteExpectedOutput],
    store_prefix: &str,
) -> Result<(), String> {
    if outputs.len() != expected_outputs.len() {
        return Err("remote-execution-output-count-mismatch".to_string());
    }
    let expected = expected_output_path_map(expected_outputs);
    let mut seen = BTreeSet::new();
    for output in outputs {
        if !seen.insert(output.name.as_str()) {
            return Err("remote-execution-output-name-duplicate".to_string());
        }
        validate_output_path_against_expected(
            output.name.as_str(),
            output.logical_path.as_str(),
            &expected,
            store_prefix,
            "remote-execution-output-identity-mismatch",
        )?;
        if !is_blake3_hex_digest(&output.content_digest_blake3)
            || !is_blake3_hex_digest(&output.artifact_attestation_digest_blake3)
        {
            return Err("remote-execution-output-metadata-digest-invalid".to_string());
        }
        if output.nar_payload.is_some() && output.path_info.is_none() {
            return Err("remote-execution-output-nar-pathinfo-missing".to_string());
        }
        remote_nar_payload_summary(output)?;
    }
    debug_assert_eq!(seen.len(), outputs.len());
    debug_assert_eq!(expected.len(), expected_outputs.len());
    Ok(())
}

pub fn redeem_after_queue(ticket: &mut RemoteTicket, request_validated: bool) -> Result<(), String> {
    if !request_validated {
        return Ok(());
    }
    if ticket.uses_remaining == 0 {
        return Err("ticket-exhausted".to_string());
    }
    ticket.uses_remaining = ticket.uses_remaining.saturating_sub(1);
    Ok(())
}

pub fn derive_missing_inputs(declared_refs: &[String], present_refs: &[String]) -> Result<Vec<String>, String> {
    if declared_refs.len() > MAX_REMOTE_INPUT_REFS || present_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"));
    }
    let present = present_refs.iter().collect::<std::collections::BTreeSet<_>>();
    Ok(declared_refs.iter().filter(|reference| !present.contains(reference)).cloned().collect::<Vec<_>>())
}

pub fn decide_output_trust(
    signing_key_id: &str,
    trusted_key_ids: &[String],
    store_prefix_matches: bool,
) -> OutputTrustDecision {
    if !store_prefix_matches {
        return OutputTrustDecision::Reject("store-prefix-mismatch".to_string());
    }
    let signer = parse_output_key_ref(signing_key_id);
    debug_assert_eq!(signer.name.is_empty(), signing_key_id.is_empty());
    debug_assert!(signer.key_material_digest_blake3.as_ref().is_none_or(|digest| is_blake3_hex_digest(digest)));
    let mut is_same_name_material_required = false;
    let mut is_same_name_different_material = false;
    for trusted_key_id in trusted_key_ids {
        let trusted = parse_output_key_ref(trusted_key_id);
        if trusted.name != signer.name {
            continue;
        }
        match (&signer.key_material_digest_blake3, &trusted.key_material_digest_blake3) {
            (Some(signer_digest), Some(trusted_digest)) if signer_digest == trusted_digest => {
                return OutputTrustDecision::Accept {
                    key_id: signing_key_id.to_string(),
                    trust_basis: signer.trust_basis(signing_key_id),
                };
            }
            (Some(_), Some(_)) => is_same_name_different_material = true,
            (None, Some(_)) => is_same_name_material_required = true,
            (_, None) => {
                return OutputTrustDecision::Accept {
                    key_id: signing_key_id.to_string(),
                    trust_basis: signer.trust_basis(signing_key_id),
                };
            }
        }
    }
    if is_same_name_material_required {
        return OutputTrustDecision::Reject("output-key-material-missing".to_string());
    }
    if is_same_name_different_material {
        return OutputTrustDecision::Reject("same-name-different-output-key".to_string());
    }
    OutputTrustDecision::Reject("untrusted-output-key".to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OutputKeyRef {
    name: String,
    key_material_digest_blake3: Option<String>,
}

impl OutputKeyRef {
    fn trust_basis(&self, key_id: &str) -> RemoteOutputTrustBasis {
        RemoteOutputTrustBasis {
            key_id: key_id.to_string(),
            key_material_digest_blake3: self.key_material_digest_blake3.clone(),
        }
    }
}

fn parse_output_key_ref(value: &str) -> OutputKeyRef {
    if let Some((name, material)) = value.split_once(':')
        && !name.is_empty()
        && !material.is_empty()
    {
        return OutputKeyRef {
            name: name.to_string(),
            key_material_digest_blake3: Some(blake3::hash(material.as_bytes()).to_hex().to_string()),
        };
    }
    OutputKeyRef {
        name: value.to_string(),
        key_material_digest_blake3: None,
    }
}

pub fn validate_worker_registration(registration: &RemoteWorkerRegistration) -> Result<(), String> {
    if registration.endpoint_id.is_empty() {
        return Err("remote-worker-endpoint-empty".to_string());
    }
    if registration.protocol_version != REMOTE_PROTOCOL_VERSION {
        return Err("remote-worker-protocol-version-mismatch".to_string());
    }
    if registration.worker_generation == 0 {
        return Err("remote-worker-generation-zero".to_string());
    }
    if registration.concurrency == 0 || registration.concurrency > MAX_REMOTE_WORKER_CONCURRENCY {
        return Err(format!("remote-worker-concurrency-exceeds-{MAX_REMOTE_WORKER_CONCURRENCY}"));
    }
    validate_non_empty_bounded_unique_strings("remote-worker-system", &registration.systems, MAX_REMOTE_CAPABILITIES)?;
    validate_bounded_unique_strings("remote-worker-feature", &registration.feature_labels, MAX_REMOTE_CAPABILITIES)?;
    validate_non_empty_bounded_unique_strings(
        "remote-worker-sandbox",
        &registration.sandbox_modes,
        MAX_REMOTE_CAPABILITIES,
    )?;
    validate_non_empty_bounded_unique_strings(
        "remote-worker-network",
        &registration.network_modes,
        MAX_REMOTE_CAPABILITIES,
    )?;
    validate_non_empty_bounded_unique_strings(
        "remote-worker-store-prefix",
        &registration.logical_store_prefixes,
        MAX_REMOTE_CAPABILITIES,
    )?;
    validate_non_empty_bounded_unique_strings(
        "remote-worker-signing-key",
        &registration.output_signing_key_ids,
        MAX_REMOTE_CAPABILITIES,
    )?;
    if let Some(inventory) = &registration.resource_inventory {
        canonical_remote_worker_resource_inventory(inventory).map_err(|reason| reason.as_str().to_string())?;
    }
    if registration.resumable_jobs.len() > MAX_REMOTE_STATUS_ITEMS {
        return Err(format!("remote-worker-resume-summary-count-exceeds-{MAX_REMOTE_STATUS_ITEMS}"));
    }
    for summary in &registration.resumable_jobs {
        validate_worker_resume_summary(summary)?;
    }
    debug_assert!(registration.concurrency <= MAX_REMOTE_WORKER_CONCURRENCY);
    debug_assert!(registration.resumable_jobs.len() <= MAX_REMOTE_STATUS_ITEMS);
    Ok(())
}

fn validate_worker_resume_summary(summary: &RemoteWorkerResumeSummary) -> Result<(), String> {
    if summary.job_id.as_str().is_empty() {
        return Err("remote-worker-resume-identity-empty".to_string());
    }
    if summary.attempt_id.as_str().is_empty() {
        return Err("remote-worker-resume-identity-empty".to_string());
    }
    if summary.attempt_id.as_str() == LEGACY_MISSING_ATTEMPT_ID {
        return Err("remote-worker-resume-identity-empty".to_string());
    }
    if summary.normalized_build_key.is_empty() {
        return Err("remote-worker-resume-identity-empty".to_string());
    }
    if summary.fence_generation.get() == 0 {
        return Err(RemoteAttemptReasonCode::FenceInvalid.as_str().to_string());
    }
    if !is_blake3_hex_digest(&summary.normalized_build_key) {
        return Err("remote-worker-resume-key-invalid".to_string());
    }
    debug_assert!(!summary.job_id.as_str().is_empty());
    debug_assert!(summary.fence_generation.get() > 0);
    Ok(())
}

fn validate_bounded_unique_strings(label: &str, values: &[String], max_len: usize) -> Result<(), String> {
    if values.len() > max_len {
        return Err(format!("{label}-count-exceeds-{max_len}"));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if value.is_empty() {
            return Err(format!("{label}-empty"));
        }
        if !seen.insert(value.as_str()) {
            return Err(format!("{label}-duplicate"));
        }
    }
    Ok(())
}

fn validate_non_empty_bounded_unique_strings(label: &str, values: &[String], max_len: usize) -> Result<(), String> {
    if values.is_empty() {
        return Err(format!("{label}-missing"));
    }
    validate_bounded_unique_strings(label, values, max_len)
}

pub fn validate_coordinator_build_request(
    request: &RemoteCoordinatorBuildRequest,
) -> Result<RemoteExecutablePlan, String> {
    if request.required_system.is_empty() {
        return Err("remote-coordinator-required-system-empty".to_string());
    }
    if request.required_sandbox_mode.is_empty() {
        return Err("remote-coordinator-required-sandbox-empty".to_string());
    }
    if request.required_network_mode.is_empty() {
        return Err("remote-coordinator-required-network-empty".to_string());
    }
    if request.trusted_output_keys.is_empty() {
        return Err("remote-coordinator-trusted-output-keys-empty".to_string());
    }
    validate_bounded_unique_strings(
        "remote-coordinator-required-feature",
        &request.required_features,
        MAX_REMOTE_CAPABILITIES,
    )?;
    validate_bounded_unique_strings(
        "remote-coordinator-live-output-claim",
        &request.live_output_claims,
        MAX_REMOTE_EXPECTED_OUTPUTS,
    )?;
    if let Some(requirements) = &request.resource_requirements {
        canonical_remote_resource_requirements(requirements).map_err(|reason| reason.as_str().to_string())?;
    }
    if let Some(scope) = &request.locality_scope
        && (!is_blake3_hex_digest(&scope.manifest_digest_blake3) || !is_blake3_hex_digest(&scope.policy_digest_blake3))
    {
        return Err("remote-coordinator-locality-scope-invalid".to_string());
    }
    if request.request.upload_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err("remote-coordinator-upload-byte-limit-exceeded".to_string());
    }
    if request.request.build_time_limit_secs == 0 || request.request.build_time_limit_secs > MAX_REMOTE_BUILD_TIME_SECS
    {
        return Err("remote-coordinator-build-time-limit-exceeded".to_string());
    }
    let plan = plan_remote_executable_request(&request.request)?;
    debug_assert_eq!(plan.request_id, request.request.request_id);
    debug_assert!(request.request.build_time_limit_secs <= MAX_REMOTE_BUILD_TIME_SECS);
    Ok(plan)
}

pub fn normalized_remote_build_key(request: &RemoteCoordinatorBuildRequest) -> Result<String, String> {
    let plan = validate_coordinator_build_request(request)?;
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_COORDINATOR_BUILD_KEY_LABEL);
    hash_labeled_str(&mut hasher, "store-prefix", &request.request.store_prefix);
    hash_executable_plan_source(&mut hasher, &plan.source);
    hash_labeled_str(&mut hasher, "system", &plan.system);
    hash_ordered_values(&mut hasher, "command-arg", &plan.command_args);
    hash_ordered_map(&mut hasher, "env", &plan.command_env);
    hash_ordered_values(&mut hasher, "input-ref", &request.request.input_refs);
    hash_ordered_values(&mut hasher, "source-input-ref", &request.request.source_input_refs);
    hash_expected_outputs_for_key(&mut hasher, &request.request.expected_outputs);
    if let Some(replay) = &request.request.failure_replay {
        hash_labeled_str(&mut hasher, "failure-replay-source-bundle", replay.source_bundle_blake3.as_str());
        hash_labeled_str(&mut hasher, "failure-replay-execution", replay.execution_blake3.as_str());
    }
    hash_labeled_str(&mut hasher, "required-system", &request.required_system);
    hash_ordered_values(&mut hasher, "required-feature", &request.required_features);
    hash_labeled_str(&mut hasher, "required-sandbox", &request.required_sandbox_mode);
    hash_labeled_str(&mut hasher, "required-network", &request.required_network_mode);
    if let Some(requirements) = &request.resource_requirements {
        let canonical =
            canonical_remote_resource_requirements(requirements).map_err(|reason| reason.as_str().to_string())?;
        hash_ordered_values(&mut hasher, "semantic-accelerator-class", &canonical.semantic_accelerator_classes);
    }
    let normalized_key_blake3 = hasher.finalize().to_hex().to_string();
    debug_assert!(is_blake3_hex_digest(&normalized_key_blake3));
    debug_assert_eq!(normalized_key_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    Ok(normalized_key_blake3)
}

fn hash_ordered_values(hasher: &mut blake3::Hasher, label: &str, values: &[String]) {
    for value in values {
        hash_labeled_str(hasher, label, value);
    }
}

fn hash_ordered_map(hasher: &mut blake3::Hasher, label: &str, values: &BTreeMap<String, String>) {
    for (name, value) in values {
        hash_labeled_str(hasher, &format!("{label}-name"), name);
        hash_labeled_str(hasher, &format!("{label}-value"), value);
    }
}

fn hash_expected_outputs_for_key(hasher: &mut blake3::Hasher, outputs: &[RemoteExpectedOutput]) {
    for output in outputs {
        hash_labeled_str(hasher, "expected-output-name", &output.name);
        match &output.logical_path {
            Some(logical_path) => hash_labeled_str(hasher, "expected-output-path", logical_path),
            None => hash_labeled_str(hasher, "expected-output-path", "<content-addressed>"),
        }
    }
}

pub fn apply_worker_registration(
    state: &mut RemoteCoordinatorState,
    registration: RemoteWorkerRegistration,
) -> Result<Vec<RemoteJobId>, String> {
    validate_worker_registration(&registration)?;
    let adopted = validate_worker_resume_summaries(state, &registration)?;
    let mut candidate = state.clone();
    let endpoint_id = registration.endpoint_id.clone();
    let worker_generation = registration.worker_generation;
    if let Some(policy) = registration.workspace_policy.as_ref() {
        crate::remote_farm_config::validate_remote_workspace_policy(policy)?;
        candidate.workspace_registrations.insert(endpoint_id.clone(), policy.clone());
    } else {
        candidate.workspace_registrations.remove(&endpoint_id);
    }
    candidate.workers.insert(endpoint_id.clone(), registration);
    candidate.verified_locality_observations.retain(|_, summary| {
        summary.worker_endpoint_id != endpoint_id || summary.worker_generation == worker_generation
    });
    validate_coordinator_resource_state(&candidate)?;
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    debug_assert!(state.workers.contains_key(&endpoint_id));
    debug_assert!(state.verified_locality_observations.values().all(|summary| {
        summary.worker_endpoint_id != endpoint_id || summary.worker_generation == worker_generation
    }));
    Ok(adopted)
}

// r[impl external_batch_dispatchers.diagnostics]
pub fn external_batch_plan_report(operation: &ExternalBatchOperation) -> ExternalBatchPlanReport {
    ExternalBatchPlanReport {
        schema: EXTERNAL_BATCH_PLAN_REPORT_SCHEMA,
        dispatch_id_blake3: operation.dispatch_id_blake3.clone(),
        dispatcher_profile_ref_blake3: operation.dispatcher_profile_ref_blake3.clone(),
        provider_class: bounded_untrusted_text(&operation.provider_class),
        worker_bootstrap_ref_blake3: operation.worker_bootstrap_ref_blake3.clone(),
        semantic_capability_class: operation.semantic_capability_class.clone(),
        resources: operation.resources.clone(),
        startup_timeout_secs: operation.limits.startup_timeout_secs,
        terminal_timeout_secs: operation.limits.terminal_timeout_secs,
        non_claims: vec![EXTERNAL_BATCH_NON_CLAIM.to_string()],
    }
}

struct ExternalBatchSubmitInput<'a> {
    request: &'a RemoteCoordinatorBuildRequest,
    profile: &'a crate::remote_farm_config::RemoteBatchDispatcherProfile,
    expected_worker_endpoint_id: &'a str,
    allocation_attempt: u32,
    dispatcher: &'a dyn ExternalBatchDispatcher,
    observed_unix_s: u64,
}

pub type ExternalBatchAllocationSubmitFn = fn(
    &mut RemoteCoordinatorState,
    &RemoteCoordinatorBuildRequest,
    &crate::remote_farm_config::RemoteBatchDispatcherProfile,
    &str,
    u32,
    &dyn ExternalBatchDispatcher,
    u64,
) -> Result<ExternalBatchOperationResponse, String>;

pub const SUBMIT_EXTERNAL_BATCH_ALLOCATION: ExternalBatchAllocationSubmitFn =
    |state, request, profile, expected_worker_endpoint_id, allocation_attempt, dispatcher, observed_unix_s| {
        submit_external_batch_allocation_core(state, ExternalBatchSubmitInput {
            request,
            profile,
            expected_worker_endpoint_id,
            allocation_attempt,
            dispatcher,
            observed_unix_s,
        })
    };
pub use SUBMIT_EXTERNAL_BATCH_ALLOCATION as submit_external_batch_allocation;

fn submit_external_batch_allocation_core(
    state: &mut RemoteCoordinatorState,
    input: ExternalBatchSubmitInput<'_>,
) -> Result<ExternalBatchOperationResponse, String> {
    let operation = plan_external_batch_allocation(
        input.request,
        input.profile,
        input.expected_worker_endpoint_id,
        input.allocation_attempt,
    )?;
    if let Some(existing) = state.external_batch_attempts.get(&operation.dispatch_id_blake3) {
        if existing.submit_operation != operation {
            return Err("external-batch-idempotency-conflict".to_string());
        }
        return Ok(existing.last_response.clone());
    }
    if state.external_batch_attempts.len() >= MAX_REMOTE_STATUS_ITEMS {
        return Err("external-batch-state-attempt-limit-exceeded".to_string());
    }
    let response = input.dispatcher.submit(&operation, input.observed_unix_s)?;
    crunch_build::distributed::validate_external_batch_response(&operation, &response)?;
    let record = ExternalBatchCoordinatorRecord {
        submit_operation: operation.clone(),
        last_operation: operation,
        last_response: response.clone(),
        reconcile_attempts: 0,
        worker_registered: false,
        coordinator_job_id: None,
        resource_lease_id_blake3: None,
        output_admission_digest_blake3: None,
    };
    let mut candidate = state.clone();
    candidate.external_batch_attempts.insert(record.submit_operation.dispatch_id_blake3.clone(), record);
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    assert!(state.external_batch_attempts.contains_key(&response.dispatch_id_blake3));
    assert!(!state.external_batch_attempts[&response.dispatch_id_blake3].worker_registered);
    Ok(response)
}

fn plan_external_batch_allocation(
    request: &RemoteCoordinatorBuildRequest,
    profile: &crate::remote_farm_config::RemoteBatchDispatcherProfile,
    expected_worker_endpoint_id: &str,
    allocation_attempt: u32,
) -> Result<ExternalBatchOperation, String> {
    if allocation_attempt == 0 || allocation_attempt > profile.max_reconcile_attempts {
        return Err("external-batch-allocation-attempt-invalid".to_string());
    }
    let normalized_build_key = normalized_remote_build_key(request)?;
    let identity = derive_external_batch_allocation_identity(ExternalBatchAllocationIdentityInput {
        normalized_build_key: &normalized_build_key,
        adapter_instance_id: &profile.instance_id,
        dispatcher_generation: profile.generation,
        allocation_attempt,
    });
    let job_id =
        RemoteJobId::new(format!("external-batch-{identity}")).map_err(|reason| reason.as_str().to_string())?;
    let attempt_id = RemoteAttemptId::new(format!("external-batch-attempt-{allocation_attempt}-{identity}"))
        .map_err(|reason| reason.as_str().to_string())?;
    let fence_generation =
        RemoteFenceGeneration::new(u64::from(allocation_attempt)).map_err(|reason| reason.as_str().to_string())?;
    let resource_requirements = request
        .resource_requirements
        .clone()
        .ok_or_else(|| "external-batch-resource-requirements-missing".to_string())?;
    let operation = plan_external_batch_operation(ExternalBatchOperationInput {
        kind: ExternalBatchOperationKind::Submit,
        adapter_instance_id: profile.instance_id.clone(),
        dispatcher_profile_ref_blake3: crate::remote_farm_config::remote_batch_dispatcher_profile_ref(profile)?,
        provider_class: profile.provider_class.clone(),
        worker_bootstrap_ref_blake3: crate::remote_farm_config::remote_batch_worker_bootstrap_ref(profile)?,
        semantic_capability_class: external_batch_semantic_capability_class(request),
        dispatcher_generation: profile.generation,
        normalized_build_key,
        job_id,
        attempt_id,
        fence_generation,
        expected_worker_endpoint_id: expected_worker_endpoint_id.to_string(),
        resource_requirements,
        limits: ExternalBatchLimits {
            startup_timeout_secs: profile.startup_timeout_secs,
            terminal_timeout_secs: profile.terminal_timeout_secs,
            max_reconcile_attempts: profile.max_reconcile_attempts,
        },
        external_job_id: None,
    })?;
    debug_assert_eq!(operation.expected_worker_endpoint_id, expected_worker_endpoint_id);
    debug_assert_eq!(operation.fence_generation.get(), u64::from(allocation_attempt));
    Ok(operation)
}

fn external_batch_semantic_capability_class(request: &RemoteCoordinatorBuildRequest) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "system", &request.required_system);
    hash_ordered_values(&mut hasher, "feature", &request.required_features);
    hash_labeled_str(&mut hasher, "sandbox", &request.required_sandbox_mode);
    hash_labeled_str(&mut hasher, "network", &request.required_network_mode);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(is_blake3_hex_digest(&digest));
    debug_assert!(!request.required_system.is_empty());
    digest
}

struct ExternalBatchAllocationIdentityInput<'a> {
    normalized_build_key: &'a str,
    adapter_instance_id: &'a str,
    dispatcher_generation: u64,
    allocation_attempt: u32,
}

fn derive_external_batch_allocation_identity(input: ExternalBatchAllocationIdentityInput<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "domain", "mantle-external-batch-allocation-identity-v1");
    hash_labeled_str(&mut hasher, "normalized-build-key", input.normalized_build_key);
    hash_labeled_str(&mut hasher, "adapter-instance", input.adapter_instance_id);
    hash_labeled_str(&mut hasher, "dispatcher-generation", input.dispatcher_generation.to_string());
    hash_labeled_str(&mut hasher, "allocation-attempt", input.allocation_attempt.to_string());
    let identity = hasher.finalize().to_hex().to_string();
    debug_assert!(is_blake3_hex_digest(&identity));
    debug_assert!(!input.adapter_instance_id.is_empty());
    identity
}

pub fn observe_external_batch_allocation(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    dispatcher: &dyn ExternalBatchDispatcher,
    observed_unix_s: u64,
) -> Result<ExternalBatchOperationResponse, String> {
    run_external_batch_followup(
        state,
        dispatch_id_blake3,
        ExternalBatchOperationKind::Observe,
        dispatcher,
        observed_unix_s,
    )
}

pub fn cancel_external_batch_allocation(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    dispatcher: &dyn ExternalBatchDispatcher,
    observed_unix_s: u64,
) -> Result<ExternalBatchOperationResponse, String> {
    run_external_batch_followup(
        state,
        dispatch_id_blake3,
        ExternalBatchOperationKind::Cancel,
        dispatcher,
        observed_unix_s,
    )
}

pub fn reconcile_external_batch_allocation(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    dispatcher: &dyn ExternalBatchDispatcher,
    observed_unix_s: u64,
    overall_deadline_exceeded: bool,
) -> Result<ExternalBatchReconcileDecision, String> {
    let response = run_external_batch_followup(
        state,
        dispatch_id_blake3,
        ExternalBatchOperationKind::Reconcile,
        dispatcher,
        observed_unix_s,
    )?;
    let record = state
        .external_batch_attempts
        .get(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    let decision = crunch_build::distributed::decide_external_batch_reconciliation(
        &record.last_operation,
        &response,
        ExternalBatchReconcileFacts {
            worker_registered: record.worker_registered,
            reconcile_attempt: record.reconcile_attempts,
            overall_deadline_exceeded,
        },
    )?;
    assert_eq!(record.last_response, response);
    assert!(record.reconcile_attempts > 0);
    Ok(decision)
}

fn run_external_batch_followup(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    kind: ExternalBatchOperationKind,
    dispatcher: &dyn ExternalBatchDispatcher,
    observed_unix_s: u64,
) -> Result<ExternalBatchOperationResponse, String> {
    let record = state
        .external_batch_attempts
        .get(dispatch_id_blake3)
        .cloned()
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    let external_job_id = record
        .last_response
        .external_job_id
        .clone()
        .ok_or_else(|| "external-batch-external-job-id-missing".to_string())?;
    let operation = plan_external_batch_followup(&record.submit_operation, kind, external_job_id)?;
    let response = match kind {
        ExternalBatchOperationKind::Submit => return Err("external-batch-followup-kind-submit".to_string()),
        ExternalBatchOperationKind::Observe => dispatcher.observe(&operation, observed_unix_s)?,
        ExternalBatchOperationKind::Cancel => dispatcher.cancel(&operation, observed_unix_s)?,
        ExternalBatchOperationKind::Reconcile => dispatcher.reconcile(&operation, observed_unix_s)?,
    };
    crunch_build::distributed::validate_external_batch_response(&operation, &response)?;
    crunch_build::distributed::validate_external_batch_state_transition(record.last_response.state, response.state)?;
    let mut candidate = state.clone();
    let candidate_record = candidate
        .external_batch_attempts
        .get_mut(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    candidate_record.last_operation = operation;
    candidate_record.last_response = response.clone();
    if kind == ExternalBatchOperationKind::Reconcile {
        candidate_record.reconcile_attempts = candidate_record
            .reconcile_attempts
            .checked_add(1)
            .ok_or_else(|| "external-batch-reconcile-attempt-overflow".to_string())?;
        if candidate_record.reconcile_attempts > candidate_record.submit_operation.limits.max_reconcile_attempts {
            return Err("external-batch-reconcile-attempt-limit-exceeded".to_string());
        }
    }
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    assert_eq!(state.external_batch_attempts[dispatch_id_blake3].last_response, response);
    assert_eq!(state.external_batch_attempts[dispatch_id_blake3].last_operation.operation, kind);
    Ok(response)
}

// r[impl external_batch_dispatchers.worker_handoff]
pub fn register_external_batch_worker(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    registration: RemoteWorkerRegistration,
) -> Result<Vec<RemoteJobId>, String> {
    let record = state
        .external_batch_attempts
        .get(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    if record.last_response.state.is_terminal() {
        return Err("external-batch-terminal-allocation-worker-rejected".to_string());
    }
    if registration.endpoint_id != record.submit_operation.expected_worker_endpoint_id {
        return Err("external-batch-worker-endpoint-mismatch".to_string());
    }
    if registration.worker_generation != record.submit_operation.dispatcher_generation {
        return Err("external-batch-worker-generation-mismatch".to_string());
    }
    let mut candidate = state.clone();
    candidate
        .external_batch_attempts
        .get_mut(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?
        .worker_registered = true;
    let adopted = apply_worker_registration(&mut candidate, registration)?;
    *state = candidate;
    assert!(state.external_batch_attempts[dispatch_id_blake3].worker_registered);
    assert!(
        state.workers.contains_key(
            &state.external_batch_attempts[dispatch_id_blake3].submit_operation.expected_worker_endpoint_id
        )
    );
    Ok(adopted)
}

struct ExternalBatchTransferAuthorizationInput<'a> {
    dispatch_id_blake3: &'a str,
    worker_endpoint_id: &'a str,
}

pub const AUTHORIZE_EXTERNAL_BATCH_TRANSFER: fn(&RemoteCoordinatorState, &str, &str) -> Result<(), String> =
    |state, dispatch_id_blake3, worker_endpoint_id| {
        authorize_external_batch_transfer_core(state, ExternalBatchTransferAuthorizationInput {
            dispatch_id_blake3,
            worker_endpoint_id,
        })
    };
pub use AUTHORIZE_EXTERNAL_BATCH_TRANSFER as authorize_external_batch_transfer;

fn authorize_external_batch_transfer_core(
    state: &RemoteCoordinatorState,
    input: ExternalBatchTransferAuthorizationInput<'_>,
) -> Result<(), String> {
    let record = state
        .external_batch_attempts
        .get(input.dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    if !record.worker_registered {
        return Err("external-batch-worker-not-registered".to_string());
    }
    if input.worker_endpoint_id != record.submit_operation.expected_worker_endpoint_id {
        return Err("external-batch-worker-endpoint-mismatch".to_string());
    }
    let worker = state
        .workers
        .get(input.worker_endpoint_id)
        .ok_or_else(|| "external-batch-state-registered-worker-missing".to_string())?;
    if worker.worker_generation != record.submit_operation.dispatcher_generation {
        return Err("external-batch-worker-generation-mismatch".to_string());
    }
    debug_assert!(record.worker_registered);
    debug_assert_eq!(worker.endpoint_id, input.worker_endpoint_id);
    Ok(())
}

struct ExternalBatchTerminationInput<'a> {
    dispatch_id_blake3: &'a str,
    binding: &'a RemoteProductionAttemptBinding,
    cause: RemoteCoordinatorTerminationCause,
    dispatcher: &'a dyn ExternalBatchDispatcher,
    observed_unix_s: u64,
}

pub type ExternalBatchAssignedAttemptTerminationFn = fn(
    &mut RemoteCoordinatorState,
    &str,
    &RemoteProductionAttemptBinding,
    RemoteCoordinatorTerminationCause,
    &dyn ExternalBatchDispatcher,
    u64,
) -> Result<ExternalBatchOperationResponse, String>;

pub const TERMINATE_EXTERNAL_BATCH_ASSIGNED_ATTEMPT: ExternalBatchAssignedAttemptTerminationFn =
    |state, dispatch_id_blake3, binding, cause, dispatcher, observed_unix_s| {
        terminate_external_batch_assigned_attempt_core(state, ExternalBatchTerminationInput {
            dispatch_id_blake3,
            binding,
            cause,
            dispatcher,
            observed_unix_s,
        })
    };
pub use TERMINATE_EXTERNAL_BATCH_ASSIGNED_ATTEMPT as terminate_external_batch_assigned_attempt;

fn terminate_external_batch_assigned_attempt_core(
    state: &mut RemoteCoordinatorState,
    input: ExternalBatchTerminationInput<'_>,
) -> Result<ExternalBatchOperationResponse, String> {
    let response =
        cancel_external_batch_allocation(state, input.dispatch_id_blake3, input.dispatcher, input.observed_unix_s)?;
    terminate_coordinator_attempt(state, input.binding, input.cause)?;
    assert_eq!(response.state, ExternalBatchJobState::Cancelled);
    assert_eq!(state.jobs[&input.binding.job_id].phase, RemoteCoordinatorJobPhase::Lost);
    Ok(response)
}

pub fn admit_external_batch_coordinator_dispatch(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    request: &RemoteCoordinatorBuildRequest,
    retry_policy: RemoteAttemptRetryPolicy,
    time: RemoteAttemptTimeFacts,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    let expected_worker_endpoint_id = state
        .external_batch_attempts
        .get(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?
        .submit_operation
        .expected_worker_endpoint_id
        .clone();
    authorize_external_batch_transfer(state, dispatch_id_blake3, &expected_worker_endpoint_id)?;
    let selected =
        select_coordinator_worker(state, request)?.ok_or_else(|| "external-batch-worker-not-eligible".to_string())?;
    if selected.worker_endpoint_id != expected_worker_endpoint_id {
        return Err("external-batch-worker-not-selected".to_string());
    }
    let decision = admit_coordinator_dispatch(state, request, retry_policy, time)?;
    let coordinator_job_id = match &decision {
        RemoteCoordinatorDispatchDecision::Dispatch {
            worker_endpoint_id,
            job_id,
            ..
        } if worker_endpoint_id == &expected_worker_endpoint_id => job_id.clone(),
        RemoteCoordinatorDispatchDecision::AttachExisting { job_id, .. }
        | RemoteCoordinatorDispatchDecision::RedeliverResult { job_id, .. } => job_id.clone(),
        _ => return Err("external-batch-coordinator-assignment-mismatch".to_string()),
    };
    bind_external_batch_coordinator_job(state, dispatch_id_blake3, &coordinator_job_id)?;
    debug_assert_eq!(
        state.external_batch_attempts[dispatch_id_blake3].coordinator_job_id.as_ref(),
        Some(&coordinator_job_id)
    );
    debug_assert!(state.jobs.contains_key(&coordinator_job_id));
    Ok(decision)
}

fn bind_external_batch_coordinator_job(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    coordinator_job_id: &RemoteJobId,
) -> Result<(), String> {
    let job = state
        .jobs
        .get(coordinator_job_id)
        .cloned()
        .ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let mut candidate = state.clone();
    let record = candidate
        .external_batch_attempts
        .get_mut(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    record.coordinator_job_id = Some(coordinator_job_id.clone());
    record.resource_lease_id_blake3 = job.resource_lease_id_blake3;
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    assert_eq!(
        state.external_batch_attempts[dispatch_id_blake3].coordinator_job_id.as_ref(),
        Some(coordinator_job_id)
    );
    assert_eq!(
        state.external_batch_attempts[dispatch_id_blake3].resource_lease_id_blake3,
        state.jobs[coordinator_job_id].resource_lease_id_blake3
    );
    Ok(())
}

pub fn external_batch_composition_evidence(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    binding: &RemoteProductionAttemptBinding,
    admission: &RemoteOutputAdmissionReport,
) -> Result<ExternalBatchCompositionEvidence, String> {
    let record = state
        .external_batch_attempts
        .get(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    let endpoint_id = &record.submit_operation.expected_worker_endpoint_id;
    authorize_external_batch_transfer(state, dispatch_id_blake3, endpoint_id)?;
    let job = state.jobs.get(&binding.job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    if job.normalized_build_key != record.submit_operation.normalized_build_key
        || job.assigned_worker_endpoint_id.as_deref() != Some(endpoint_id)
    {
        return Err("external-batch-composition-assignment-mismatch".to_string());
    }
    let current = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| "external-batch-composition-current-attempt-missing".to_string())?;
    if current.attempt_id != binding.attempt_id || current.fence_generation != binding.fence_generation {
        return Err("external-batch-composition-attempt-fence-mismatch".to_string());
    }
    if !job.output_admission_completed {
        return Err("external-batch-composition-output-not-admitted".to_string());
    }
    if !is_blake3_hex_digest(&admission.output_digest_blake3) {
        return Err("external-batch-composition-output-digest-invalid".to_string());
    }
    let external_job_id = record
        .last_response
        .external_job_id
        .clone()
        .ok_or_else(|| "external-batch-external-job-id-missing".to_string())?;
    let worker_endpoint_id = endpoint_id.clone();
    let dispatcher_profile_ref_blake3 = record.submit_operation.dispatcher_profile_ref_blake3.clone();
    let provider_class = record.submit_operation.provider_class.clone();
    let queue_state = record.last_response.state;
    let resources = record.submit_operation.resources.clone();
    let resource_lease_id_blake3 = record.resource_lease_id_blake3.clone();
    let mut candidate = state.clone();
    candidate
        .external_batch_attempts
        .get_mut(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?
        .output_admission_digest_blake3 = Some(admission.output_digest_blake3.clone());
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    let evidence = ExternalBatchCompositionEvidence {
        schema: EXTERNAL_BATCH_COMPOSITION_EVIDENCE_SCHEMA,
        dispatch_id_blake3: dispatch_id_blake3.to_string(),
        dispatcher_profile_ref_blake3,
        provider_class,
        external_job_id,
        queue_state,
        resources,
        worker_endpoint_id,
        worker_registered: true,
        coordinator_job_id: binding.job_id.clone(),
        resource_lease_id_blake3,
        transfer_mode: admission.transfer.mode,
        output_admission_digest_blake3: admission.output_digest_blake3.clone(),
        output_trust_key_id: admission.trust_basis.key_id.clone(),
        non_claims: vec![
            EXTERNAL_BATCH_NON_CLAIM.to_string(),
            "output admission remains governed by the ordinary signed-output trust path".to_string(),
        ],
    };
    debug_assert_eq!(evidence.output_admission_digest_blake3, admission.output_digest_blake3);
    debug_assert_eq!(evidence.coordinator_job_id, binding.job_id);
    Ok(evidence)
}

struct RemoteWorkerLocalityProbeInput<'a> {
    worker_endpoint_id: &'a str,
    worker_generation: u64,
    receiver_root: &'a Path,
    manifest: &'a CanonicalRemoteTransferManifest,
    admission: crate::remote_transfer::RemoteTransferAdmissionFacts,
}

pub type RemoteWorkerLocalityProbeFn = fn(
    &mut RemoteCoordinatorState,
    &str,
    u64,
    &Path,
    &CanonicalRemoteTransferManifest,
    crate::remote_transfer::RemoteTransferAdmissionFacts,
) -> Result<RemoteVerifiedLocalitySummary, String>;

pub const PROBE_AND_RECORD_REMOTE_WORKER_LOCALITY: RemoteWorkerLocalityProbeFn =
    |state, worker_endpoint_id, worker_generation, receiver_root, manifest, admission| {
        probe_and_record_remote_worker_locality_core(state, RemoteWorkerLocalityProbeInput {
            worker_endpoint_id,
            worker_generation,
            receiver_root,
            manifest,
            admission,
        })
    };
pub use PROBE_AND_RECORD_REMOTE_WORKER_LOCALITY as probe_and_record_remote_worker_locality;
const _: () = {
    let _observability_api = remote_build_observability_report;
    let _operator_rail_api = remote_operator_e2e_rail_report;
    let _batch_submit_api = submit_external_batch_allocation;
    let _batch_termination_api = terminate_external_batch_assigned_attempt;
    let _locality_probe_api = probe_and_record_remote_worker_locality;
};

fn probe_and_record_remote_worker_locality_core(
    state: &mut RemoteCoordinatorState,
    input: RemoteWorkerLocalityProbeInput<'_>,
) -> Result<RemoteVerifiedLocalitySummary, String> {
    let receiver_facts =
        crate::remote_transfer::probe_remote_transfer_receiver(input.receiver_root, input.manifest, input.admission)?;
    record_receiver_verified_worker_locality(
        state,
        input.worker_endpoint_id,
        input.worker_generation,
        input.manifest,
        receiver_facts,
    )
}

fn record_receiver_verified_worker_locality(
    state: &mut RemoteCoordinatorState,
    worker_endpoint_id: &str,
    worker_generation: u64,
    manifest: &CanonicalRemoteTransferManifest,
    receiver_facts: RemoteTransferReceiverFacts,
) -> Result<RemoteVerifiedLocalitySummary, String> {
    let worker = state
        .workers
        .get(worker_endpoint_id)
        .ok_or_else(|| "remote-coordinator-worker-unknown".to_string())?;
    if worker.worker_generation != worker_generation {
        return Err(RemoteLocalityReasonCode::WorkerGenerationStale.as_str().to_string());
    }
    let observation = RemoteLocalityProbeObservation {
        worker_endpoint_id: worker_endpoint_id.to_string(),
        worker_generation,
        scope: RemoteLocalityScope {
            manifest_digest_blake3: manifest.digest_blake3.as_str().to_string(),
            policy_digest_blake3: manifest.manifest.policy_digest_blake3.as_str().to_string(),
        },
        receiver_probe_verified: true,
        receiver_facts,
    };
    let summary = normalize_remote_verified_locality(worker_endpoint_id, worker_generation, manifest, &observation)
        .map_err(|reason| reason.as_str().to_string())?;
    let key = verified_locality_observation_key(&summary);
    let mut candidate = state.clone();
    if !candidate.verified_locality_observations.contains_key(&key)
        && candidate.verified_locality_observations.len() >= MAX_REMOTE_LOCALITY_OBSERVATIONS
    {
        return Err("remote-locality-observation-limit-exceeded".to_string());
    }
    candidate.verified_locality_observations.insert(key, summary.clone());
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    assert_eq!(summary.worker_endpoint_id, worker_endpoint_id);
    assert_eq!(summary.worker_generation, worker_generation);
    Ok(summary)
}

fn verified_locality_observation_key(summary: &RemoteVerifiedLocalitySummary) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "worker", &summary.worker_endpoint_id);
    hash_labeled_str(&mut hasher, "generation", summary.worker_generation.to_string());
    hash_labeled_str(&mut hasher, "manifest", &summary.scope.manifest_digest_blake3);
    hash_labeled_str(&mut hasher, "policy", &summary.scope.policy_digest_blake3);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(is_blake3_hex_digest(&digest));
    debug_assert!(!summary.worker_endpoint_id.is_empty());
    digest
}

struct RemoteProductionDispatchInput<'a> {
    request: &'a ConcreteBuildRequest,
    endpoint_id: &'a str,
    worker_generation: u64,
    worker_concurrency: u32,
    resource_inventory: Option<RemoteWorkerResourceInventory>,
    trusted_output_keys: &'a [String],
    now_unix_s: u64,
}

pub type RemoteProductionDispatchAdmissionFn = fn(
    &mut RemoteCoordinatorState,
    &ConcreteBuildRequest,
    &str,
    u64,
    u32,
    Option<RemoteWorkerResourceInventory>,
    &[String],
    u64,
) -> Result<RemoteProductionAttemptBinding, String>;

pub const ADMIT_REMOTE_PRODUCTION_DISPATCH: RemoteProductionDispatchAdmissionFn =
    |state,
     request,
     endpoint_id,
     worker_generation,
     worker_concurrency,
     resource_inventory,
     trusted_output_keys,
     now_unix_s| {
        admit_remote_production_dispatch_core(state, RemoteProductionDispatchInput {
            request,
            endpoint_id,
            worker_generation,
            worker_concurrency,
            resource_inventory,
            trusted_output_keys,
            now_unix_s,
        })
    };
pub use ADMIT_REMOTE_PRODUCTION_DISPATCH as admit_remote_production_dispatch;

fn admit_remote_production_dispatch_core(
    state: &mut RemoteCoordinatorState,
    input: RemoteProductionDispatchInput<'_>,
) -> Result<RemoteProductionAttemptBinding, String> {
    let executable = plan_remote_executable_request(input.request)?;
    let registration = production_worker_registration(state, &input, &executable.system);
    apply_worker_registration(state, registration)?;
    let coordinator_request = production_coordinator_request(&input, &executable.system);
    let deadline_unix_s = input
        .now_unix_s
        .checked_add(input.request.build_time_limit_secs)
        .ok_or_else(|| "remote-production-attempt-deadline-overflow".to_string())?;
    let decision = admit_coordinator_dispatch(
        state,
        &coordinator_request,
        RemoteAttemptRetryPolicy {
            max_attempts: DEFAULT_REMOTE_ATTEMPT_MAX,
            retry_delay_secs: DEFAULT_REMOTE_RETRY_DELAY_SECS,
            attempt_timeout_secs: DEFAULT_REMOTE_ATTEMPT_TIMEOUT_SECS,
        },
        RemoteAttemptTimeFacts {
            now_unix_s: input.now_unix_s,
            failure_observed_unix_s: input.now_unix_s,
            overall_deadline_unix_s: deadline_unix_s,
        },
    )?;
    let binding = match decision {
        RemoteCoordinatorDispatchDecision::Dispatch {
            job_id,
            attempt_id,
            fence_generation,
            ..
        } => RemoteProductionAttemptBinding {
            job_id,
            attempt_id,
            fence_generation,
        },
        RemoteCoordinatorDispatchDecision::AttachExisting { job_id, .. } => {
            current_production_attempt_binding(state, &job_id, input.endpoint_id)?
        }
        RemoteCoordinatorDispatchDecision::RedeliverResult { .. } => {
            return Err("remote-production-result-redelivery-requires-stored-transcript".to_string());
        }
        RemoteCoordinatorDispatchDecision::Pending { reason }
        | RemoteCoordinatorDispatchDecision::Reject { reason } => return Err(reason),
    };
    start_remote_production_attempt_if_queued(state, &binding, input.now_unix_s)?;
    assert!(binding.fence_generation.get() > 0);
    assert!(!binding.attempt_id.as_str().is_empty());
    Ok(binding)
}

fn production_worker_registration(
    state: &RemoteCoordinatorState,
    input: &RemoteProductionDispatchInput<'_>,
    system: &str,
) -> RemoteWorkerRegistration {
    let registration = RemoteWorkerRegistration {
        endpoint_id: input.endpoint_id.to_string(),
        protocol_version: REMOTE_PROTOCOL_VERSION,
        worker_generation: input.worker_generation,
        systems: vec![system.to_string()],
        feature_labels: Vec::new(),
        sandbox_modes: vec!["native".to_string()],
        network_modes: vec!["none".to_string()],
        logical_store_prefixes: vec![input.request.store_prefix.clone()],
        concurrency: input.worker_concurrency,
        resource_inventory: input.resource_inventory.clone(),
        output_signing_key_ids: input.trusted_output_keys.to_vec(),
        resumable_jobs: Vec::new(),
        workspace_policy: state.workspace_registrations.get(input.endpoint_id).cloned(),
    };
    debug_assert_eq!(registration.endpoint_id, input.endpoint_id);
    debug_assert_eq!(registration.systems.len(), 1);
    registration
}

fn production_coordinator_request(
    input: &RemoteProductionDispatchInput<'_>,
    system: &str,
) -> RemoteCoordinatorBuildRequest {
    let mut live_output_claims = Vec::with_capacity(input.request.expected_outputs.len());
    live_output_claims.extend(input.request.expected_outputs.iter().filter_map(|output| output.logical_path.clone()));
    let request = RemoteCoordinatorBuildRequest {
        request: input.request.clone(),
        required_system: system.to_string(),
        required_features: Vec::new(),
        required_sandbox_mode: "native".to_string(),
        required_network_mode: "none".to_string(),
        resource_requirements: input.request.resource_requirements.clone(),
        locality_scope: input.request.locality_scope.clone(),
        trusted_output_keys: input.trusted_output_keys.to_vec(),
        live_output_claims,
        wait_for_worker: false,
    };
    debug_assert_eq!(request.required_system, system);
    debug_assert!(request.live_output_claims.len() <= input.request.expected_outputs.len());
    request
}

fn current_production_attempt_binding(
    state: &RemoteCoordinatorState,
    job_id: &RemoteJobId,
    endpoint_id: &str,
) -> Result<RemoteProductionAttemptBinding, String> {
    let job = state.jobs.get(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    if job.assigned_worker_endpoint_id.as_deref() != Some(endpoint_id) {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    let attempt = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    Ok(RemoteProductionAttemptBinding {
        job_id: job_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        fence_generation: attempt.fence_generation,
    })
}

pub fn start_remote_production_attempt_if_queued(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    now_unix_s: u64,
) -> Result<(), String> {
    let phase = state
        .jobs
        .get(&binding.job_id)
        .and_then(|job| job.current_attempt.as_ref())
        .map(|attempt| attempt.phase)
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    if phase != RemoteAttemptPhase::Queued {
        return Ok(());
    }
    let attempt_start = production_attempt_report(binding, "start", now_unix_s, RemoteAttemptReportPayload::Start)?;
    let applied = apply_coordinator_attempt_report(
        state,
        &attempt_start,
        RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: false,
        },
        RemoteLogRetentionPolicy::default(),
    )?;
    if applied.disposition != RemoteAttemptApplyDisposition::Applied {
        return Err(applied.reason_code.as_str().to_string());
    }
    debug_assert_eq!(state.jobs[&binding.job_id].phase, RemoteCoordinatorJobPhase::Running);
    debug_assert_eq!(
        state.jobs[&binding.job_id].current_attempt.as_ref().map(|attempt| attempt.phase),
        Some(RemoteAttemptPhase::Running),
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteProductionTelemetryFact {
    PrioritySelected { competing_goal_count: u32 },
    RouteSelected,
    QueueAdmitted,
    WorkerAssigned,
    FenceAccepted,
    StaleFenceRejected,
    ExecutionStarted,
    ExecutionCompleted,
    ExecutionFailed,
    TransferDemand { chunks: u32 },
    TransferCredit { bytes: u64 },
    TransferResumed { bytes: u64 },
    TransferCutoff { accepted: bool },
    TransferCompleted { bytes: u64, mode: RemoteTransferMode },
    TransferFallback { bytes: u64 },
    OutputAdmitted,
    OutputRejected,
}

pub fn remote_production_telemetry_event(
    fact: RemoteProductionTelemetryFact,
) -> Result<RemoteTelemetryEvent, RemoteTelemetryReasonCode> {
    if matches!(fact, RemoteProductionTelemetryFact::PrioritySelected {
        competing_goal_count: 0
    }) {
        return Err(RemoteTelemetryReasonCode::EventValueInvalid);
    }
    let fields = production_telemetry_fields(fact);
    debug_assert!(fields.value > 0);
    debug_assert_eq!(fields.route, RemoteTelemetryRouteClass::RemoteStdio);
    crunch_build::distributed::remote_telemetry_event(
        fields.category,
        fields.phase,
        fields.result,
        fields.route,
        fields.retry,
        fields.transfer,
        fields.reason,
        fields.capability,
        fields.measurement,
        fields.value,
    )
}

#[derive(Debug, Clone, Copy)]
struct ProductionTelemetryFields {
    category: RemoteTelemetryCategory,
    phase: RemoteTelemetryPhaseClass,
    result: RemoteTelemetryResultClass,
    route: RemoteTelemetryRouteClass,
    retry: RemoteTelemetryRetryClass,
    transfer: RemoteTelemetryTransferClass,
    reason: RemoteTelemetryReasonClass,
    capability: RemoteTelemetryCapabilityClass,
    measurement: RemoteTelemetryMeasurementKind,
    value: u64,
}

fn production_telemetry_fields(fact: RemoteProductionTelemetryFact) -> ProductionTelemetryFields {
    let mut fields = ProductionTelemetryFields {
        category: RemoteTelemetryCategory::Execution,
        phase: RemoteTelemetryPhaseClass::Running,
        result: RemoteTelemetryResultClass::Accepted,
        route: RemoteTelemetryRouteClass::RemoteStdio,
        retry: RemoteTelemetryRetryClass::None,
        transfer: RemoteTelemetryTransferClass::None,
        reason: RemoteTelemetryReasonClass::AttemptTransition,
        capability: RemoteTelemetryCapabilityClass::Exact,
        measurement: RemoteTelemetryMeasurementKind::Occurrences,
        value: 1,
    };
    apply_production_telemetry_fact(&mut fields, fact);
    debug_assert!(fields.value > 0);
    debug_assert_eq!(fields.route, RemoteTelemetryRouteClass::RemoteStdio);
    fields
}

fn apply_production_telemetry_fact(fields: &mut ProductionTelemetryFields, fact: RemoteProductionTelemetryFact) {
    match fact {
        RemoteProductionTelemetryFact::PrioritySelected { competing_goal_count } => {
            set_priority_telemetry_fields(fields, competing_goal_count);
        }
        RemoteProductionTelemetryFact::RouteSelected => set_route_telemetry_fields(fields),
        RemoteProductionTelemetryFact::QueueAdmitted => set_queue_telemetry_fields(fields),
        RemoteProductionTelemetryFact::WorkerAssigned => set_assignment_telemetry_fields(fields),
        RemoteProductionTelemetryFact::FenceAccepted => set_fence_telemetry_fields(fields, true),
        RemoteProductionTelemetryFact::StaleFenceRejected => set_fence_telemetry_fields(fields, false),
        RemoteProductionTelemetryFact::ExecutionStarted => {}
        RemoteProductionTelemetryFact::ExecutionCompleted => set_execution_telemetry_fields(fields, true),
        RemoteProductionTelemetryFact::ExecutionFailed => set_execution_telemetry_fields(fields, false),
        RemoteProductionTelemetryFact::TransferDemand { chunks } => {
            set_transfer_fields(fields, RemoteTelemetryReasonClass::TransferDemand, u64::from(chunks));
            fields.measurement = RemoteTelemetryMeasurementKind::Objects;
        }
        RemoteProductionTelemetryFact::TransferCredit { bytes } => {
            set_transfer_byte_fields(fields, RemoteTelemetryReasonClass::TransferCredit, bytes);
        }
        RemoteProductionTelemetryFact::TransferResumed { bytes } => {
            set_transfer_byte_fields(fields, RemoteTelemetryReasonClass::TransferResumed, bytes);
        }
        RemoteProductionTelemetryFact::TransferCutoff { accepted } => set_transfer_cutoff_fields(fields, accepted),
        RemoteProductionTelemetryFact::TransferCompleted { bytes, mode } => {
            set_transfer_completion_fields(fields, bytes, mode);
        }
        RemoteProductionTelemetryFact::TransferFallback { bytes } => set_transfer_fallback_fields(fields, bytes),
        RemoteProductionTelemetryFact::OutputAdmitted => set_output_telemetry_fields(fields, true),
        RemoteProductionTelemetryFact::OutputRejected => set_output_telemetry_fields(fields, false),
    }
}

fn set_priority_telemetry_fields(fields: &mut ProductionTelemetryFields, competing_goal_count: u32) {
    fields.category = RemoteTelemetryCategory::Queue;
    fields.phase = RemoteTelemetryPhaseClass::Planning;
    fields.reason = RemoteTelemetryReasonClass::PrioritySelected;
    fields.measurement = RemoteTelemetryMeasurementKind::CandidateCount;
    fields.value = u64::from(competing_goal_count);
}

fn set_route_telemetry_fields(fields: &mut ProductionTelemetryFields) {
    fields.category = RemoteTelemetryCategory::Route;
    fields.phase = RemoteTelemetryPhaseClass::Planning;
    fields.reason = RemoteTelemetryReasonClass::RouteSelected;
}

fn set_queue_telemetry_fields(fields: &mut ProductionTelemetryFields) {
    fields.category = RemoteTelemetryCategory::Queue;
    fields.phase = RemoteTelemetryPhaseClass::Queued;
    fields.reason = RemoteTelemetryReasonClass::QueueAdmitted;
}

fn set_assignment_telemetry_fields(fields: &mut ProductionTelemetryFields) {
    fields.category = RemoteTelemetryCategory::Assignment;
    fields.phase = RemoteTelemetryPhaseClass::Assigned;
    fields.reason = RemoteTelemetryReasonClass::WorkerAssigned;
}

fn set_fence_telemetry_fields(fields: &mut ProductionTelemetryFields, is_accepted: bool) {
    fields.category = RemoteTelemetryCategory::RetryFencing;
    fields.result = if is_accepted {
        RemoteTelemetryResultClass::Accepted
    } else {
        RemoteTelemetryResultClass::Rejected
    };
    fields.retry = if is_accepted {
        RemoteTelemetryRetryClass::CurrentFence
    } else {
        RemoteTelemetryRetryClass::StaleFence
    };
    fields.reason = if is_accepted {
        RemoteTelemetryReasonClass::FenceAccepted
    } else {
        RemoteTelemetryReasonClass::FenceRejected
    };
}

fn set_execution_telemetry_fields(fields: &mut ProductionTelemetryFields, is_succeeded: bool) {
    if is_succeeded {
        fields.result = RemoteTelemetryResultClass::Succeeded;
        fields.reason = RemoteTelemetryReasonClass::ExecutionCompleted;
        return;
    }
    fields.phase = RemoteTelemetryPhaseClass::Terminal;
    fields.result = RemoteTelemetryResultClass::Failed;
    fields.retry = RemoteTelemetryRetryClass::Terminal;
    fields.reason = RemoteTelemetryReasonClass::ExecutionFailed;
}

fn set_transfer_byte_fields(fields: &mut ProductionTelemetryFields, reason: RemoteTelemetryReasonClass, bytes: u64) {
    set_transfer_fields(fields, reason, bytes);
    fields.measurement = RemoteTelemetryMeasurementKind::Bytes;
}

fn set_transfer_cutoff_fields(fields: &mut ProductionTelemetryFields, is_accepted: bool) {
    set_transfer_fields(fields, RemoteTelemetryReasonClass::TransferCutoff, 1);
    fields.result = if is_accepted {
        RemoteTelemetryResultClass::Accepted
    } else {
        RemoteTelemetryResultClass::Dropped
    };
}

fn set_transfer_completion_fields(fields: &mut ProductionTelemetryFields, bytes: u64, mode: RemoteTransferMode) {
    set_transfer_fields(fields, RemoteTelemetryReasonClass::TransferCompleted, bytes);
    fields.result = RemoteTelemetryResultClass::Succeeded;
    fields.measurement = RemoteTelemetryMeasurementKind::Bytes;
    fields.transfer = telemetry_transfer_class(mode);
}

fn set_transfer_fallback_fields(fields: &mut ProductionTelemetryFields, bytes: u64) {
    set_transfer_fields(fields, RemoteTelemetryReasonClass::TransferFallback, bytes);
    fields.transfer = RemoteTelemetryTransferClass::Fallback;
    fields.capability = RemoteTelemetryCapabilityClass::Degraded;
    fields.measurement = RemoteTelemetryMeasurementKind::Bytes;
}

fn set_output_telemetry_fields(fields: &mut ProductionTelemetryFields, is_admitted: bool) {
    fields.category = RemoteTelemetryCategory::Admission;
    fields.phase = if is_admitted {
        RemoteTelemetryPhaseClass::Admitting
    } else {
        RemoteTelemetryPhaseClass::Terminal
    };
    fields.result = if is_admitted {
        RemoteTelemetryResultClass::Accepted
    } else {
        RemoteTelemetryResultClass::Rejected
    };
    fields.retry = if is_admitted {
        RemoteTelemetryRetryClass::None
    } else {
        RemoteTelemetryRetryClass::Terminal
    };
    fields.reason = if is_admitted {
        RemoteTelemetryReasonClass::OutputAdmitted
    } else {
        RemoteTelemetryReasonClass::OutputRejected
    };
    debug_assert_eq!(fields.category, RemoteTelemetryCategory::Admission);
    debug_assert_eq!(fields.result == RemoteTelemetryResultClass::Accepted, is_admitted);
}

fn set_transfer_fields(fields: &mut ProductionTelemetryFields, reason: RemoteTelemetryReasonClass, value: u64) {
    fields.category = RemoteTelemetryCategory::Transfer;
    fields.phase = RemoteTelemetryPhaseClass::Transferring;
    fields.transfer = RemoteTelemetryTransferClass::Streaming;
    fields.reason = reason;
    fields.value = value.max(1);
}

fn telemetry_transfer_class(mode: RemoteTransferMode) -> RemoteTelemetryTransferClass {
    match mode {
        RemoteTransferMode::Delta => RemoteTelemetryTransferClass::Delta,
        RemoteTransferMode::Full => RemoteTelemetryTransferClass::Full,
        RemoteTransferMode::Streaming => RemoteTelemetryTransferClass::Streaming,
    }
}

pub fn record_remote_production_telemetry(
    buffer: &RemoteTelemetryBuffer,
    fact: RemoteProductionTelemetryFact,
    policy: RemoteTelemetryPolicy,
) -> RemoteTelemetryBuffer {
    match remote_production_telemetry_event(fact) {
        Ok(event) => crunch_build::distributed::record_remote_telemetry(buffer, event, policy),
        Err(_) => buffer.clone(),
    }
}

pub fn remote_telemetry_buffer_health(buffer: &RemoteTelemetryBuffer) -> RemoteTelemetryBufferHealth {
    RemoteTelemetryBufferHealth {
        accepted_events: buffer.accepted_events,
        dropped_events: buffer.dropped_events,
        last_drop_reason: buffer.last_drop_reason.map(|reason| reason.as_str().to_string()),
    }
}

fn production_attempt_report(
    binding: &RemoteProductionAttemptBinding,
    label: &str,
    event_number: u64,
    payload: RemoteAttemptReportPayload,
) -> Result<RemoteAttemptReport, String> {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "job", binding.job_id.as_str());
    hash_labeled_str(&mut hasher, "attempt", binding.attempt_id.as_str());
    hash_labeled_str(&mut hasher, "label", label);
    hash_labeled_str(&mut hasher, "event-number", event_number.to_string());
    let event_id =
        RemoteEventId::new(hasher.finalize().to_hex().to_string()).map_err(|reason| reason.as_str().to_string())?;
    RemoteAttemptReport::new(
        binding.job_id.clone(),
        binding.attempt_id.clone(),
        binding.fence_generation,
        event_id,
        payload,
    )
    .map_err(|reason| reason.as_str().to_string())
}

pub fn append_remote_production_observability_log(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    telemetry: &RemoteTelemetryBuffer,
) -> Result<RemoteAttemptLogControlSummary, String> {
    let job = state.jobs.get(&binding.job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let attempt = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    if attempt.attempt_id != binding.attempt_id || attempt.fence_generation != binding.fence_generation {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    let cursor = job.immutable_log.as_ref().map_or(0, |log| log.next_cursor);
    let payload = remote_observability_log_payload(telemetry)?;
    let attempt_log_append =
        production_attempt_report(binding, "observability-log", cursor, RemoteAttemptReportPayload::LogAppend {
            cursor,
            bytes: payload.clone(),
        })?;
    let mut candidate = state.clone();
    apply_fenced_log_append(
        &mut candidate,
        &attempt_log_append,
        cursor,
        &payload,
        RemoteLogRetentionPolicy::default(),
    )?;
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    let summary = state
        .jobs
        .get(&binding.job_id)
        .and_then(|job| job.immutable_log.clone())
        .ok_or_else(|| "remote-observability-log-summary-missing".to_string())?;
    assert!(summary.next_cursor > cursor);
    assert!(summary.head_record_blake3.is_some());
    Ok(summary)
}

pub fn append_external_batch_attempt_diagnostic(
    state: &mut RemoteCoordinatorState,
    dispatch_id_blake3: &str,
    binding: &RemoteProductionAttemptBinding,
) -> Result<RemoteAttemptLogControlSummary, String> {
    let job = state.jobs.get(&binding.job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let attempt = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    if attempt.attempt_id != binding.attempt_id || attempt.fence_generation != binding.fence_generation {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    let record = state
        .external_batch_attempts
        .get(dispatch_id_blake3)
        .ok_or_else(|| "external-batch-attempt-unknown".to_string())?;
    if record.coordinator_job_id.as_ref() != Some(&binding.job_id) {
        return Err("external-batch-diagnostic-coordinator-job-mismatch".to_string());
    }
    let status = external_batch_attempt_status(record);
    let status_bytes =
        serde_json::to_vec(&status).map_err(|error| format!("external-batch-diagnostic-serialize-failed: {error}"))?;
    let payload = serde_json::to_string(&serde_json::json!({
        "schema": "mantle-external-batch-immutable-diagnostic-v1",
        "dispatch_id_blake3": status.dispatch_id_blake3,
        "status_digest_blake3": blake3::hash(&status_bytes).to_hex().to_string(),
        "state": status.state,
        "worker_registered": status.worker_registered,
        "output_admission_completed": status.output_admission_completed,
        "non_claim": EXTERNAL_BATCH_NON_CLAIM,
    }))
    .map_err(|error| format!("external-batch-diagnostic-serialize-failed: {error}"))?;
    if payload.len() > MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES {
        return Err("external-batch-diagnostic-payload-too-large".to_string());
    }
    let cursor = job.immutable_log.as_ref().map_or(0, |log| log.next_cursor);
    let attempt_log_append = production_attempt_report(
        binding,
        "external-batch-diagnostic",
        cursor,
        RemoteAttemptReportPayload::LogAppend {
            cursor,
            bytes: payload.clone(),
        },
    )?;
    let mut candidate = state.clone();
    apply_fenced_log_append(
        &mut candidate,
        &attempt_log_append,
        cursor,
        &payload,
        RemoteLogRetentionPolicy::default(),
    )?;
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    let summary = state
        .jobs
        .get(&binding.job_id)
        .and_then(|job| job.immutable_log.clone())
        .ok_or_else(|| "external-batch-diagnostic-summary-missing".to_string())?;
    assert!(summary.next_cursor > cursor);
    assert!(summary.head_record_blake3.is_some());
    Ok(summary)
}

pub fn append_remote_rejected_production_observability_log(
    state_dir: &Path,
    binding: &RemoteProductionAttemptBinding,
    telemetry: &RemoteTelemetryBuffer,
) -> Result<RemoteAttemptLogControlSummary, String> {
    let scope = rejected_observability_log_scope(binding)?;
    let policy = immutable_attempt_log_policy(RemoteLogRetentionPolicy::default())?;
    let manifest = crate::remote_attempt_log_store::load_remote_attempt_log_manifest(state_dir, &scope, policy)?;
    let cursor = manifest.next_cursor;
    let payload = remote_observability_log_payload(telemetry)?;
    let attempt_log_append =
        production_attempt_report(binding, "stale-observability-log", cursor, RemoteAttemptReportPayload::LogAppend {
            cursor,
            bytes: payload.clone(),
        })?;
    let current = RemoteAttemptLogCurrentAttemptFacts {
        scope,
        phase: RemoteAttemptPhase::Superseded,
    };
    let stored = crate::remote_attempt_log_store::append_remote_attempt_log(
        state_dir,
        &current,
        attempt_log_append.identity.event_id,
        cursor,
        payload.as_bytes(),
        policy,
    )?;
    let summary = remote_attempt_log_control_summary(&stored.manifest, RemoteLogRetentionPolicy::default());
    assert!(summary.next_cursor > cursor);
    assert_ne!(summary.scope.attempt_id, binding.attempt_id);
    Ok(summary)
}

fn rejected_observability_log_scope(binding: &RemoteProductionAttemptBinding) -> Result<RemoteAttemptLogScope, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-rejected-remote-observability-v1\0");
    hash_labeled_str(&mut hasher, "job", binding.job_id.as_str());
    hash_labeled_str(&mut hasher, "attempt", binding.attempt_id.as_str());
    hash_labeled_str(&mut hasher, "fence", binding.fence_generation.get().to_string());
    let digest = hasher.finalize().to_hex().to_string();
    let job_id = RemoteJobId::new(format!("rejected-job-{digest}")).map_err(|reason| reason.as_str().to_string())?;
    let attempt_id =
        RemoteAttemptId::new(format!("rejected-attempt-{digest}")).map_err(|reason| reason.as_str().to_string())?;
    debug_assert_ne!(attempt_id, binding.attempt_id);
    debug_assert!(binding.fence_generation.get() > 0);
    Ok(RemoteAttemptLogScope {
        job_id,
        attempt_id,
        fence_generation: binding.fence_generation,
    })
}

fn remote_observability_log_payload(telemetry: &RemoteTelemetryBuffer) -> Result<String, String> {
    let payload = serde_json::to_string(&serde_json::json!({
        "schema": "mantle-remote-observability-log-v1",
        "events": telemetry.events,
        "accepted_events": telemetry.accepted_events,
        "dropped_events": telemetry.dropped_events,
        "non_claim": REMOTE_OBSERVABILITY_NON_CLAIM,
    }))
    .map_err(|error| format!("serializing remote observability log summary: {error}"))?;
    debug_assert!(!payload.is_empty());
    debug_assert!(payload.contains("mantle-remote-observability-log-v1"));
    Ok(payload)
}

pub fn update_remote_production_observability_health(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    health: RemoteAttemptObservabilityHealth,
) -> Result<(), String> {
    let mut candidate = state.clone();
    let job = candidate
        .jobs
        .get_mut(&binding.job_id)
        .ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let attempt = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    if attempt.attempt_id != binding.attempt_id || attempt.fence_generation != binding.fence_generation {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    job.observability_health = Some(health);
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    debug_assert!(state.jobs[&binding.job_id].observability_health.is_some());
    debug_assert_eq!(
        state.jobs[&binding.job_id].current_attempt.as_ref().map(|attempt| &attempt.attempt_id),
        Some(&binding.attempt_id)
    );
    Ok(())
}

pub fn remote_observability_adapter_health(
    status: RemoteTelemetryAdapterStatus,
    reason_code: &str,
) -> RemoteTelemetryAdapterHealth {
    RemoteTelemetryAdapterHealth {
        status,
        reason_code: bounded_untrusted_text(reason_code),
    }
}

pub fn validate_current_production_attempt(
    state_dir: &Path,
    expected: &RemoteProductionAttemptBinding,
) -> Result<(), String> {
    let state = load_coordinator_state(state_dir).map_err(|err| err.to_string())?;
    let job = state
        .jobs
        .get(&expected.job_id)
        .ok_or_else(|| RemoteAttemptReasonCode::JobIdentityMismatch.as_str().to_string())?;
    let current = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    if current.attempt_id != expected.attempt_id || current.fence_generation != expected.fence_generation {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    if matches!(current.phase, RemoteAttemptPhase::Failed | RemoteAttemptPhase::Superseded) {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    assert_eq!(current.job_id, expected.job_id);
    assert!(current.fence_generation.get() > 0);
    Ok(())
}

fn validate_worker_resume_summaries(
    state: &RemoteCoordinatorState,
    registration: &RemoteWorkerRegistration,
) -> Result<Vec<RemoteJobId>, String> {
    let mut adopted = Vec::with_capacity(registration.resumable_jobs.len());
    for summary in &registration.resumable_jobs {
        validate_worker_resume_claim(state, registration, summary)?;
        adopted.push(summary.job_id.clone());
    }
    Ok(adopted)
}

fn validate_worker_resume_claim(
    state: &RemoteCoordinatorState,
    registration: &RemoteWorkerRegistration,
    summary: &RemoteWorkerResumeSummary,
) -> Result<(), String> {
    let job = state.jobs.get(&summary.job_id).ok_or_else(|| "remote-worker-resume-unknown-job".to_string())?;
    let attempt = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    if job.normalized_build_key != summary.normalized_build_key {
        return Err("remote-worker-resume-key-conflict".to_string());
    }
    if job.assigned_worker_endpoint_id.as_deref() != Some(registration.endpoint_id.as_str()) {
        return Err(RemoteAttemptReasonCode::ResumeStateConflict.as_str().to_string());
    }
    if attempt.attempt_id != summary.attempt_id || attempt.fence_generation != summary.fence_generation {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    let log_next_cursor = job.immutable_log.as_ref().map_or(0, |log| log.next_cursor);
    if job.phase != summary.phase
        || job.result_available != summary.result_available
        || log_next_cursor != summary.log_next_cursor
    {
        return Err(RemoteAttemptReasonCode::ResumeStateConflict.as_str().to_string());
    }
    debug_assert_eq!(attempt.attempt_id, summary.attempt_id);
    debug_assert_eq!(job.assigned_worker_endpoint_id.as_deref(), Some(registration.endpoint_id.as_str()));
    Ok(())
}

fn coordinator_request_uses_mutable_workspace(request: &RemoteCoordinatorBuildRequest) -> Result<bool, String> {
    match &request.request.payload {
        RemoteConcreteBuildPayload::Derivation { drv_json, .. } => {
            let derivation: crunch_glue::CrunchDerivation = serde_json::from_str(drv_json)
                .map_err(|error| format!("remote-workspace-derivation-invalid: {error}"))?;
            let Some(policy_json) = derivation.env.get(crunch_glue::WORKSPACE_POLICY_ENV) else {
                return Ok(false);
            };
            let policy: crunch_build::WorkspacePolicy = serde_json::from_str(policy_json)
                .map_err(|error| format!("remote-workspace-policy-invalid: {error}"))?;
            crunch_build::validate_workspace_policy(&policy)
                .map_err(|reason| format!("remote-workspace-policy-invalid: {}", reason.as_str()))?;
            Ok(policy.mode == crunch_build::WorkspaceMode::MutableSession)
        }
        RemoteConcreteBuildPayload::Action { spec_json, .. } => {
            let value: serde_json::Value =
                serde_json::from_str(spec_json).map_err(|error| format!("remote-workspace-action-invalid: {error}"))?;
            Ok(value.pointer("/workspace/mode").and_then(serde_json::Value::as_str) == Some("mutable-session"))
        }
    }
}

fn mutable_workspace_job_key(identity: (&str, &str, &RemoteAssignmentNonce)) -> String {
    let (shared_key, request_id, assignment_nonce) = identity;
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "domain", "mantle-mutable-workspace-job-v1");
    hash_labeled_str(&mut hasher, "shared-key-non-claim", shared_key);
    hash_labeled_str(&mut hasher, "request-id", request_id);
    hash_labeled_str(&mut hasher, "assignment-nonce", assignment_nonce.as_str());
    hasher.finalize().to_hex().to_string()
}

fn plan_coordinator_dispatch(
    state: &RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
    assignment_nonce: &RemoteAssignmentNonce,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    debug_assert!(!request.request.request_id.is_empty());
    debug_assert!(!assignment_nonce.as_str().is_empty());
    let shared_key = normalized_remote_build_key(request)?;
    let is_mutable_workspace = coordinator_request_uses_mutable_workspace(request)?;
    let normalized_key = if is_mutable_workspace {
        mutable_workspace_job_key((&shared_key, &request.request.request_id, assignment_nonce))
    } else {
        shared_key
    };
    if !is_mutable_workspace {
        if let Some(reason) = conflicting_live_output_claim(state, request, &normalized_key) {
            return Ok(RemoteCoordinatorDispatchDecision::Reject { reason });
        }
        if let Some(decision) = existing_job_decision(state, &normalized_key) {
            return Ok(decision);
        }
    }
    let Some(worker) = select_coordinator_worker(state, request)? else {
        return Ok(no_worker_dispatch_decision(state, request));
    };
    new_dispatch_decision(&worker, normalized_key, assignment_nonce)
}

fn new_dispatch_decision(
    placement: &CoordinatorWorkerPlacement,
    normalized_build_key: String,
    assignment_nonce: &RemoteAssignmentNonce,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    let job_id = coordinator_job_id(&normalized_build_key, assignment_nonce)?;
    let fence_generation = RemoteFenceGeneration::INITIAL;
    let attempt_id =
        derive_remote_attempt_id(&job_id, assignment_nonce, fence_generation, &placement.worker_endpoint_id)
            .map_err(|reason| reason.as_str().to_string())?;
    Ok(RemoteCoordinatorDispatchDecision::Dispatch {
        worker_endpoint_id: placement.worker_endpoint_id.clone(),
        job_id,
        attempt_id,
        fence_generation,
        normalized_build_key,
        resource_fit: placement.resource_fit,
        locality: placement.locality.clone(),
    })
}

fn no_worker_dispatch_decision(
    state: &RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
) -> RemoteCoordinatorDispatchDecision {
    let reason = no_matching_worker_reason(state, request);
    if request.wait_for_worker {
        return RemoteCoordinatorDispatchDecision::Pending { reason };
    }
    RemoteCoordinatorDispatchDecision::Reject { reason }
}

pub fn admit_coordinator_dispatch(
    state: &mut RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
    retry_policy: RemoteAttemptRetryPolicy,
    time: RemoteAttemptTimeFacts,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    let assignment_nonce = generate_remote_assignment_nonce()?;
    admit_coordinator_dispatch_with_nonce(state, request, retry_policy, time, assignment_nonce)
}

fn admit_coordinator_dispatch_with_nonce(
    state: &mut RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
    retry_policy: RemoteAttemptRetryPolicy,
    time: RemoteAttemptTimeFacts,
    assignment_nonce: RemoteAssignmentNonce,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    debug_assert!(!request.request.request_id.is_empty());
    debug_assert!(!assignment_nonce.as_str().is_empty());
    let decision = plan_coordinator_dispatch(state, request, &assignment_nonce)?;
    let RemoteCoordinatorDispatchDecision::Dispatch {
        worker_endpoint_id,
        job_id,
        attempt_id,
        fence_generation,
        normalized_build_key,
        ..
    } = &decision
    else {
        return Ok(decision);
    };
    let placement = coordinator_worker_placement_candidates(state, request)?
        .into_iter()
        .find(|placement| placement.worker_endpoint_id == *worker_endpoint_id)
        .ok_or_else(|| "remote-worker-placement-selection-missing".to_string())?;
    let mut summary = queued_job_summary(QueuedJobSummaryInput {
        job_id,
        normalized_key: normalized_build_key,
        worker_endpoint_id,
        assignment_nonce,
        request,
        retry_policy,
        time,
    })?;
    if summary.current_attempt.as_ref().map(|attempt| &attempt.attempt_id) != Some(attempt_id)
        || summary.current_attempt.as_ref().map(|attempt| attempt.fence_generation) != Some(*fence_generation)
    {
        return Err("remote-coordinator-attempt-plan-mismatch".to_string());
    }
    let mut candidate = state.clone();
    install_job_resource_reservation(&mut candidate, &mut summary, request, &placement)?;
    for claim in &summary.live_output_claims {
        candidate.live_output_claims.insert(claim.clone(), normalized_build_key.clone());
    }
    candidate.jobs.insert(job_id.clone(), summary);
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    debug_assert!(state.jobs.contains_key(job_id));
    debug_assert_eq!(state.jobs[job_id].assigned_worker_endpoint_id.as_ref(), Some(worker_endpoint_id));
    Ok(decision)
}

struct QueuedJobSummaryInput<'a> {
    job_id: &'a RemoteJobId,
    normalized_key: &'a str,
    worker_endpoint_id: &'a str,
    assignment_nonce: RemoteAssignmentNonce,
    request: &'a RemoteCoordinatorBuildRequest,
    retry_policy: RemoteAttemptRetryPolicy,
    time: RemoteAttemptTimeFacts,
}

fn queued_job_summary(input: QueuedJobSummaryInput<'_>) -> Result<RemoteCoordinatorJobSummary, String> {
    let QueuedJobSummaryInput {
        job_id,
        normalized_key,
        worker_endpoint_id,
        assignment_nonce,
        request,
        retry_policy,
        time,
    } = input;
    let current_attempt =
        plan_remote_attempt_assignment(job_id, worker_endpoint_id, assignment_nonce, None, retry_policy, time)
            .map_err(|reason| reason.as_str().to_string())?;
    let summary = RemoteCoordinatorJobSummary {
        job_id: job_id.clone(),
        normalized_build_key: normalized_key.to_string(),
        assigned_worker_endpoint_id: Some(worker_endpoint_id.to_string()),
        phase: RemoteCoordinatorJobPhase::Queued,
        live_output_claims: if coordinator_request_uses_mutable_workspace(request)? {
            Vec::new()
        } else {
            request.live_output_claims.clone()
        },
        result_available: false,
        lost_phase: None,
        immutable_log: None,
        observability_health: None,
        failure_debug: None,
        short_error: None,
        current_attempt: Some(current_attempt),
        transfer_checkpoint: None,
        transferred_bytes: 0,
        output_admission_completed: false,
        last_attempt_reason_code: None,
        resource_requirements: request.resource_requirements.clone(),
        resource_lease_id_blake3: None,
        resource_fit: None,
        locality: None,
    };
    debug_assert_eq!(summary.job_id, *job_id);
    debug_assert_eq!(summary.assigned_worker_endpoint_id.as_deref(), Some(worker_endpoint_id));
    Ok(summary)
}

fn install_job_resource_reservation(
    state: &mut RemoteCoordinatorState,
    summary: &mut RemoteCoordinatorJobSummary,
    request: &RemoteCoordinatorBuildRequest,
    placement: &CoordinatorWorkerPlacement,
) -> Result<(), String> {
    summary.resource_fit = Some(placement.resource_fit);
    summary.locality = placement.locality.clone();
    let Some(requirements) = &request.resource_requirements else {
        debug_assert_eq!(placement.resource_fit, crunch_build::ResourceFitClass::Unknown);
        debug_assert!(summary.resource_lease_id_blake3.is_none());
        return Ok(());
    };
    let worker = state
        .workers
        .get(&placement.worker_endpoint_id)
        .ok_or_else(|| "remote-coordinator-worker-unknown".to_string())?;
    let inventory = worker
        .resource_inventory
        .as_ref()
        .ok_or_else(|| "remote-worker-resource-inventory-missing".to_string())?;
    let attempt = summary
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    let scope = RemoteResourceLeaseScope {
        worker_endpoint_id: placement.worker_endpoint_id.clone(),
        worker_generation: worker.worker_generation,
        job_id: summary.job_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        fence_generation: attempt.fence_generation,
    };
    let active_leases = state.resource_leases.values().cloned().collect::<Vec<_>>();
    let plan = plan_remote_resource_reservation(scope, inventory, requirements, &active_leases)
        .map_err(|reason| reason.as_str().to_string())?;
    if plan.resource_fit != placement.resource_fit {
        return Err("remote-resource-placement-drift".to_string());
    }
    summary.resource_lease_id_blake3 = Some(plan.lease.lease_id_blake3.clone());
    state.resource_leases.insert(plan.lease.lease_id_blake3.clone(), plan.lease);
    assert!(summary.resource_lease_id_blake3.is_some());
    assert_eq!(summary.resource_fit, Some(placement.resource_fit));
    Ok(())
}

struct ReassignCoordinatorAttemptInput<'a> {
    job_id: &'a RemoteJobId,
    worker_endpoint_id: &'a str,
    failure_class: RemoteAttemptFailureClass,
    retry_policy: RemoteAttemptRetryPolicy,
    time: RemoteAttemptTimeFacts,
    assignment_nonce: RemoteAssignmentNonce,
}

pub type CoordinatorAttemptReassignmentFn = fn(
    &mut RemoteCoordinatorState,
    &RemoteJobId,
    &str,
    RemoteAttemptFailureClass,
    RemoteAttemptRetryPolicy,
    RemoteAttemptTimeFacts,
) -> Result<RemoteCoordinatorDispatchDecision, String>;

pub const REASSIGN_COORDINATOR_ATTEMPT: CoordinatorAttemptReassignmentFn =
    |state, job_id, worker_endpoint_id, failure_class, retry_policy, time| {
        let assignment_nonce = generate_remote_assignment_nonce()?;
        reassign_coordinator_attempt_core(state, ReassignCoordinatorAttemptInput {
            job_id,
            worker_endpoint_id,
            failure_class,
            retry_policy,
            time,
            assignment_nonce,
        })
    };
pub use REASSIGN_COORDINATOR_ATTEMPT as reassign_coordinator_attempt;
const _: CoordinatorAttemptReassignmentFn = reassign_coordinator_attempt;

fn reassign_coordinator_attempt_core(
    state: &mut RemoteCoordinatorState,
    input: ReassignCoordinatorAttemptInput<'_>,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    let ReassignCoordinatorAttemptInput {
        job_id,
        worker_endpoint_id,
        failure_class,
        retry_policy,
        time,
        assignment_nonce,
    } = input;
    let job = state.jobs.get(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let previous = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    let retry = decide_remote_attempt_retry(previous, failure_class, retry_policy, time);
    if !retry.retry_allowed {
        return Ok(RemoteCoordinatorDispatchDecision::Reject {
            reason: retry.reason_code.as_str().to_string(),
        });
    }
    require_registered_worker(state, worker_endpoint_id)?;
    let next = plan_remote_attempt_assignment(
        job_id,
        worker_endpoint_id,
        assignment_nonce,
        Some(previous),
        retry_policy,
        time,
    )
    .map_err(|reason| reason.as_str().to_string())?;
    let normalized_build_key = job.normalized_build_key.clone();
    let mut candidate = state.clone();
    release_current_job_resource_lease(&mut candidate, job_id, previous)?;
    install_reassigned_attempt(&mut candidate, job_id, worker_endpoint_id, next)?;
    install_reassigned_resource_lease(&mut candidate, job_id, worker_endpoint_id)?;
    let updated_job = candidate.jobs.get(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let updated_attempt = updated_job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    let decision = dispatch_decision_from_attempt(
        CoordinatorDispatchIdentity {
            worker_endpoint_id,
            normalized_build_key: &normalized_build_key,
        },
        updated_attempt,
        updated_job.resource_fit.unwrap_or(crunch_build::ResourceFitClass::Unknown),
        updated_job.locality.clone(),
    );
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    debug_assert_eq!(state.jobs[job_id].phase, RemoteCoordinatorJobPhase::Queued);
    debug_assert_eq!(state.jobs[job_id].assigned_worker_endpoint_id.as_deref(), Some(worker_endpoint_id));
    Ok(decision)
}

fn require_registered_worker(state: &RemoteCoordinatorState, worker_endpoint_id: &str) -> Result<(), String> {
    if state.workers.contains_key(worker_endpoint_id) {
        return Ok(());
    }
    Err("remote-coordinator-worker-unknown".to_string())
}

struct CoordinatorDispatchIdentity<'a> {
    worker_endpoint_id: &'a str,
    normalized_build_key: &'a str,
}

fn dispatch_decision_from_attempt(
    identity: CoordinatorDispatchIdentity<'_>,
    attempt: &RemoteAttemptState,
    resource_fit: crunch_build::ResourceFitClass,
    locality: Option<RemoteVerifiedLocalitySummary>,
) -> RemoteCoordinatorDispatchDecision {
    RemoteCoordinatorDispatchDecision::Dispatch {
        worker_endpoint_id: identity.worker_endpoint_id.to_string(),
        job_id: attempt.job_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        fence_generation: attempt.fence_generation,
        normalized_build_key: identity.normalized_build_key.to_string(),
        resource_fit,
        locality,
    }
}

fn install_reassigned_attempt(
    state: &mut RemoteCoordinatorState,
    job_id: &RemoteJobId,
    worker_endpoint_id: &str,
    attempt: RemoteAttemptState,
) -> Result<(), String> {
    let job = state.jobs.get_mut(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let is_worker_changed = job.assigned_worker_endpoint_id.as_deref() != Some(worker_endpoint_id);
    job.assigned_worker_endpoint_id = Some(worker_endpoint_id.to_string());
    job.phase = RemoteCoordinatorJobPhase::Queued;
    job.result_available = false;
    job.lost_phase = None;
    job.immutable_log = None;
    job.short_error = None;
    job.current_attempt = Some(attempt);
    job.transfer_checkpoint = None;
    job.transferred_bytes = 0;
    job.output_admission_completed = false;
    job.last_attempt_reason_code = Some(RemoteAttemptReasonCode::Superseded);
    job.resource_lease_id_blake3 = None;
    job.resource_fit = None;
    if is_worker_changed {
        job.locality = None;
    }
    debug_assert_eq!(job.phase, RemoteCoordinatorJobPhase::Queued);
    debug_assert!(job.current_attempt.is_some());
    Ok(())
}

fn release_current_job_resource_lease(
    state: &mut RemoteCoordinatorState,
    job_id: &RemoteJobId,
    current_attempt: &RemoteAttemptState,
) -> Result<(), String> {
    let job = state.jobs.get(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let Some(lease_id) = job.resource_lease_id_blake3.as_deref() else {
        if job.resource_requirements.is_some() {
            return Err("remote-resource-current-lease-missing".to_string());
        }
        return Ok(());
    };
    let lease = state
        .resource_leases
        .get(lease_id)
        .cloned()
        .ok_or_else(|| "remote-resource-current-lease-missing".to_string())?;
    let worker_endpoint_id = job
        .assigned_worker_endpoint_id
        .clone()
        .ok_or_else(|| "remote-resource-current-worker-missing".to_string())?;
    let worker_generation = state
        .workers
        .get(&worker_endpoint_id)
        .map(|worker| worker.worker_generation)
        .ok_or_else(|| "remote-resource-current-worker-missing".to_string())?;
    let scope = RemoteResourceLeaseScope {
        worker_endpoint_id,
        worker_generation,
        job_id: job_id.clone(),
        attempt_id: current_attempt.attempt_id.clone(),
        fence_generation: current_attempt.fence_generation,
    };
    let release = plan_remote_resource_lease_release(&lease, &scope).map_err(|reason| reason.as_str().to_string())?;
    state.resource_leases.remove(&release.lease_id_blake3);
    debug_assert!(!state.resource_leases.contains_key(&release.lease_id_blake3));
    debug_assert_eq!(release.released, lease.reserved);
    Ok(())
}

fn install_reassigned_resource_lease(
    state: &mut RemoteCoordinatorState,
    job_id: &RemoteJobId,
    worker_endpoint_id: &str,
) -> Result<(), String> {
    let (requirements, attempt) = {
        let job = state.jobs.get(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
        (job.resource_requirements.clone(), job.current_attempt.clone())
    };
    let Some(requirements) = requirements else {
        return Ok(());
    };
    let attempt = attempt.ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    let worker = state
        .workers
        .get(worker_endpoint_id)
        .ok_or_else(|| "remote-coordinator-worker-unknown".to_string())?;
    let inventory = worker
        .resource_inventory
        .as_ref()
        .ok_or_else(|| "remote-worker-resource-inventory-missing".to_string())?;
    let scope = RemoteResourceLeaseScope {
        worker_endpoint_id: worker_endpoint_id.to_string(),
        worker_generation: worker.worker_generation,
        job_id: job_id.clone(),
        attempt_id: attempt.attempt_id,
        fence_generation: attempt.fence_generation,
    };
    let active_leases = state.resource_leases.values().cloned().collect::<Vec<_>>();
    let plan = plan_remote_resource_reservation(scope, inventory, &requirements, &active_leases)
        .map_err(|reason| reason.as_str().to_string())?;
    let job = state.jobs.get_mut(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    job.resource_lease_id_blake3 = Some(plan.lease.lease_id_blake3.clone());
    job.resource_fit = Some(plan.resource_fit);
    state.resource_leases.insert(plan.lease.lease_id_blake3.clone(), plan.lease);
    assert!(job.resource_lease_id_blake3.is_some());
    assert_ne!(job.resource_fit, Some(crunch_build::ResourceFitClass::Unknown));
    Ok(())
}

pub fn apply_coordinator_attempt_report(
    state: &mut RemoteCoordinatorState,
    report: &RemoteAttemptReport,
    authorization: RemoteAttemptAuthorizationFacts,
    log_policy: RemoteLogRetentionPolicy,
) -> Result<RemoteCoordinatorAttemptApplyResult, String> {
    let Some(job) = state.jobs.get(&report.identity.job_id) else {
        return Ok(rejected_attempt_apply_result(RemoteAttemptReasonCode::JobIdentityMismatch));
    };
    let Some(current) = job.current_attempt.as_ref() else {
        return Ok(rejected_attempt_apply_result(RemoteAttemptReasonCode::LegacyStateRejected));
    };
    let plan = plan_remote_attempt_report(current, report, authorization);
    if plan.disposition != RemoteAttemptApplyDisposition::Applied {
        return Ok(RemoteCoordinatorAttemptApplyResult {
            disposition: plan.disposition,
            reason_code: plan.reason_code,
            output_admission_allowed: false,
        });
    }
    let mut candidate = state.clone();
    apply_attempt_plan_to_candidate(&mut candidate, report, &plan.next_state, plan.reason_code, log_policy)?;
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    debug_assert_eq!(state.jobs[&report.identity.job_id].current_attempt.as_ref(), Some(&plan.next_state));
    debug_assert_eq!(plan.disposition, RemoteAttemptApplyDisposition::Applied);
    Ok(RemoteCoordinatorAttemptApplyResult {
        disposition: plan.disposition,
        reason_code: plan.reason_code,
        output_admission_allowed: plan.output_admission_allowed,
    })
}

fn rejected_attempt_apply_result(reason_code: RemoteAttemptReasonCode) -> RemoteCoordinatorAttemptApplyResult {
    RemoteCoordinatorAttemptApplyResult {
        disposition: RemoteAttemptApplyDisposition::Rejected,
        reason_code,
        output_admission_allowed: false,
    }
}

fn apply_attempt_plan_to_candidate(
    state: &mut RemoteCoordinatorState,
    report: &RemoteAttemptReport,
    next_attempt: &RemoteAttemptState,
    reason_code: RemoteAttemptReasonCode,
    log_policy: RemoteLogRetentionPolicy,
) -> Result<(), String> {
    if let RemoteAttemptReportPayload::LogAppend { cursor, bytes } = &report.payload {
        apply_fenced_log_append(state, report, *cursor, bytes, log_policy)?;
    }
    let is_live_output_claim_release_required = matches!(&report.payload, RemoteAttemptReportPayload::Failure { .. });
    let (normalized_build_key, live_output_claims) = {
        let job = state
            .jobs
            .get_mut(&report.identity.job_id)
            .ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
        apply_attempt_payload_to_job(job, &report.payload, next_attempt);
        job.last_attempt_reason_code = Some(reason_code);
        (job.normalized_build_key.clone(), job.live_output_claims.clone())
    };
    if next_attempt.phase.is_terminal() {
        release_current_job_resource_lease(state, &report.identity.job_id, next_attempt)?;
        let job = state
            .jobs
            .get_mut(&report.identity.job_id)
            .ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
        job.resource_lease_id_blake3 = None;
    }
    if is_live_output_claim_release_required {
        for claim in live_output_claims {
            if state.live_output_claims.get(&claim) == Some(&normalized_build_key) {
                state.live_output_claims.remove(&claim);
            }
        }
    }
    debug_assert!(
        !next_attempt.phase.is_terminal() || state.jobs[&report.identity.job_id].resource_lease_id_blake3.is_none()
    );
    debug_assert_eq!(state.jobs[&report.identity.job_id].current_attempt.as_ref(), Some(next_attempt));
    Ok(())
}

fn apply_fenced_log_append(
    state: &mut RemoteCoordinatorState,
    report: &RemoteAttemptReport,
    cursor: u64,
    bytes: &str,
    log_policy: RemoteLogRetentionPolicy,
) -> Result<(), String> {
    let state_dir = state
        .state_dir
        .as_deref()
        .ok_or_else(|| RemoteAttemptReasonCode::PersistenceUnconfigured.as_str().to_string())?;
    let current_attempt = state
        .jobs
        .get(&report.identity.job_id)
        .and_then(|job| job.current_attempt.as_ref())
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    let current = RemoteAttemptLogCurrentAttemptFacts {
        scope: RemoteAttemptLogScope {
            job_id: current_attempt.job_id.clone(),
            attempt_id: current_attempt.attempt_id.clone(),
            fence_generation: current_attempt.fence_generation,
        },
        phase: current_attempt.phase,
    };
    let immutable_policy = immutable_attempt_log_policy(log_policy)?;
    let stored = crate::remote_attempt_log_store::append_remote_attempt_log(
        state_dir,
        &current,
        report.identity.event_id.clone(),
        cursor,
        bytes.as_bytes(),
        immutable_policy,
    )?;
    let summary = remote_attempt_log_control_summary(&stored.manifest, log_policy);
    let job = state
        .jobs
        .get_mut(&report.identity.job_id)
        .ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    job.immutable_log = Some(summary);
    debug_assert!(job.immutable_log.is_some());
    debug_assert_eq!(job.immutable_log.as_ref().map(|log| log.next_cursor), Some(stored.manifest.next_cursor));
    Ok(())
}

fn immutable_attempt_log_policy(log_policy: RemoteLogRetentionPolicy) -> Result<RemoteAttemptLogPolicy, String> {
    validate_log_policy(log_policy)?;
    let retained_count =
        u32::try_from(log_policy.max_chunks).map_err(|_| "remote-log-chunk-limit-overflow".to_string())?;
    let payload_bytes_max = log_policy.max_bytes.max(MIN_REMOTE_ATTEMPT_LOG_SHELL_PAYLOAD_BYTES);
    let record_payload_bytes_max = u32::try_from(
        payload_bytes_max.min(u64::from(crunch_build::distributed::DEFAULT_REMOTE_ATTEMPT_LOG_RECORD_PAYLOAD_BYTES)),
    )
    .map_err(|_| "remote-log-record-payload-limit-overflow".to_string())?;
    let policy = RemoteAttemptLogPolicy {
        record_payload_bytes_max,
        segment_record_count_max: 1,
        segment_payload_bytes_max: payload_bytes_max,
        retained_segment_count_max: retained_count,
        retained_record_count_max: retained_count,
        retained_payload_bytes_max: payload_bytes_max,
        replay_record_count_max: retained_count,
        replay_payload_bytes_max: payload_bytes_max,
        manifest_segment_count_max: crunch_build::distributed::DEFAULT_REMOTE_ATTEMPT_LOG_MANIFEST_SEGMENTS,
        event_identity_count_max: crunch_build::distributed::DEFAULT_REMOTE_ATTEMPT_LOG_EVENT_IDENTITIES,
    };
    policy.validate().map_err(|reason| reason.as_str().to_string())?;
    debug_assert_eq!(policy.segment_record_count_max, 1);
    debug_assert!(policy.retained_segment_count_max <= policy.manifest_segment_count_max);
    Ok(policy)
}

fn remote_attempt_log_control_summary(
    manifest: &RemoteAttemptLogManifest,
    retention_policy: RemoteLogRetentionPolicy,
) -> RemoteAttemptLogControlSummary {
    RemoteAttemptLogControlSummary {
        scope: manifest.scope.clone(),
        retention_policy,
        retained_start_cursor: manifest.retained_start_cursor,
        next_cursor: manifest.next_cursor,
        retained_record_count: manifest.retained_record_count,
        retained_payload_bytes: manifest.retained_payload_bytes,
        head_record_blake3: manifest.head_record_blake3.clone(),
        head_segment_blake3: manifest.head_segment_blake3.clone(),
        manifest_blake3: manifest.manifest_blake3.clone(),
        truncation_anchor_blake3: manifest.truncation_anchor.as_ref().map(|anchor| anchor.anchor_blake3.clone()),
        truncated: manifest.truncation_anchor.is_some(),
    }
}

fn apply_attempt_payload_to_job(
    job: &mut RemoteCoordinatorJobSummary,
    payload: &RemoteAttemptReportPayload,
    next_attempt: &RemoteAttemptState,
) {
    job.phase = coordinator_phase_for_attempt(next_attempt.phase);
    job.current_attempt = Some(next_attempt.clone());
    match payload {
        RemoteAttemptReportPayload::TransferCheckpoint {
            checkpoint,
            transferred_bytes,
        } => {
            job.transfer_checkpoint = Some(*checkpoint);
            job.transferred_bytes = *transferred_bytes;
        }
        RemoteAttemptReportPayload::ResultReady { .. } => {
            job.result_available = false;
            job.output_admission_completed = true;
        }
        RemoteAttemptReportPayload::Completion { .. } => {
            job.result_available = true;
            job.output_admission_completed = true;
        }
        RemoteAttemptReportPayload::Failure { reason_code, .. } => {
            job.result_available = false;
            job.output_admission_completed = false;
            job.short_error = Some(reason_code.as_str().to_string());
        }
        _ => {}
    }
    debug_assert_eq!(job.current_attempt.as_ref(), Some(next_attempt));
    debug_assert_eq!(job.phase, coordinator_phase_for_attempt(next_attempt.phase));
}

fn coordinator_phase_for_attempt(phase: RemoteAttemptPhase) -> RemoteCoordinatorJobPhase {
    match phase {
        RemoteAttemptPhase::Queued => RemoteCoordinatorJobPhase::Queued,
        RemoteAttemptPhase::Running | RemoteAttemptPhase::Transferring => RemoteCoordinatorJobPhase::Running,
        RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed => RemoteCoordinatorJobPhase::Finished,
        RemoteAttemptPhase::Failed | RemoteAttemptPhase::Superseded => RemoteCoordinatorJobPhase::Lost,
    }
}

fn validate_coordinator_resource_state(state: &RemoteCoordinatorState) -> Result<(), String> {
    let inventories = state
        .workers
        .iter()
        .filter_map(|(endpoint_id, worker)| {
            worker.resource_inventory.clone().map(|inventory| (endpoint_id.clone(), inventory))
        })
        .collect::<BTreeMap<_, _>>();
    let leases = state.resource_leases.values().cloned().collect::<Vec<_>>();
    validate_remote_resource_lease_snapshot(&inventories, &leases).map_err(|reason| reason.as_str().to_string())?;
    for job in state.jobs.values() {
        validate_job_resource_linkage(state, job)?;
        if let Some(locality) = &job.locality {
            validate_verified_locality_summary(locality)?;
            validate_locality_worker_binding(state, locality)?;
        }
    }
    for locality in state.verified_locality_observations.values() {
        validate_verified_locality_summary(locality)?;
        validate_locality_worker_binding(state, locality)?;
    }
    debug_assert_eq!(leases.len(), state.resource_leases.len());
    debug_assert!(state.verified_locality_observations.len() <= MAX_REMOTE_LOCALITY_OBSERVATIONS);
    Ok(())
}

fn validate_job_resource_linkage(
    state: &RemoteCoordinatorState,
    job: &RemoteCoordinatorJobSummary,
) -> Result<(), String> {
    let is_live_lease_required = job.current_attempt.as_ref().is_some_and(|attempt| attempt.phase.is_live())
        && job.resource_requirements.is_some();
    if !is_live_lease_required {
        if job.resource_lease_id_blake3.is_some() {
            return Err("remote-resource-terminal-lease-retained".to_string());
        }
        return Ok(());
    }
    let lease_id = job
        .resource_lease_id_blake3
        .as_deref()
        .ok_or_else(|| "remote-resource-current-lease-missing".to_string())?;
    let lease = state
        .resource_leases
        .get(lease_id)
        .ok_or_else(|| "remote-resource-current-lease-missing".to_string())?;
    let attempt = job
        .current_attempt
        .as_ref()
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    let worker = job
        .assigned_worker_endpoint_id
        .as_deref()
        .ok_or_else(|| "remote-resource-current-worker-missing".to_string())?;
    let worker_generation = state
        .workers
        .get(worker)
        .map(|registration| registration.worker_generation)
        .ok_or_else(|| "remote-resource-current-worker-missing".to_string())?;
    if lease.scope.worker_endpoint_id != worker {
        return Err("remote-resource-job-lease-scope-mismatch".to_string());
    }
    if lease.scope.worker_generation != worker_generation {
        return Err("remote-resource-job-lease-scope-mismatch".to_string());
    }
    if lease.scope.job_id != job.job_id {
        return Err("remote-resource-job-lease-scope-mismatch".to_string());
    }
    if lease.scope.attempt_id != attempt.attempt_id {
        return Err("remote-resource-job-lease-scope-mismatch".to_string());
    }
    if lease.scope.fence_generation != attempt.fence_generation {
        return Err("remote-resource-job-lease-scope-mismatch".to_string());
    }
    if job.resource_fit.is_none() {
        return Err("remote-resource-fit-summary-missing".to_string());
    }
    debug_assert_eq!(lease.lease_id_blake3, lease_id);
    debug_assert!(job.resource_requirements.is_some());
    Ok(())
}

fn validate_locality_worker_binding(
    state: &RemoteCoordinatorState,
    summary: &RemoteVerifiedLocalitySummary,
) -> Result<(), String> {
    let current_generation = state
        .workers
        .get(&summary.worker_endpoint_id)
        .map(|worker| worker.worker_generation)
        .ok_or_else(|| RemoteLocalityReasonCode::WorkerGenerationStale.as_str().to_string())?;
    if current_generation != summary.worker_generation {
        return Err(RemoteLocalityReasonCode::WorkerGenerationStale.as_str().to_string());
    }
    debug_assert!(current_generation > 0);
    debug_assert!(!summary.worker_endpoint_id.is_empty());
    Ok(())
}

fn validate_verified_locality_summary(summary: &RemoteVerifiedLocalitySummary) -> Result<(), String> {
    if summary.schema != REMOTE_LOCALITY_SUMMARY_SCHEMA {
        return Err("remote-locality-summary-invalid".to_string());
    }
    if summary.worker_endpoint_id.is_empty() || summary.worker_generation == 0 {
        return Err("remote-locality-summary-invalid".to_string());
    }
    if !is_blake3_hex_digest(&summary.scope.manifest_digest_blake3) {
        return Err("remote-locality-summary-invalid".to_string());
    }
    if !is_blake3_hex_digest(&summary.scope.policy_digest_blake3) {
        return Err("remote-locality-summary-invalid".to_string());
    }
    let objects = summary
        .verified_present_object_count
        .checked_add(summary.missing_object_count)
        .ok_or_else(|| "remote-locality-object-count-overflow".to_string())?;
    let bytes = summary
        .verified_present_bytes
        .checked_add(summary.missing_bytes)
        .ok_or_else(|| "remote-locality-byte-count-overflow".to_string())?;
    if objects != summary.demanded_object_count || bytes != summary.demanded_bytes {
        return Err("remote-locality-summary-accounting-mismatch".to_string());
    }
    if summary.content_locality == crunch_build::ContentLocalityClass::FullyPresent
        && (summary.missing_object_count != 0
            || summary.missing_bytes != 0
            || summary.transfer_cost != crunch_build::TransferCostClass::None)
    {
        return Err("remote-locality-full-summary-invalid".to_string());
    }
    if summary.non_claims.is_empty() || summary.non_claims.len() > MAX_REMOTE_CAPABILITIES {
        return Err("remote-locality-non-claims-invalid".to_string());
    }
    debug_assert_eq!(objects, summary.demanded_object_count);
    debug_assert_eq!(bytes, summary.demanded_bytes);
    Ok(())
}

pub fn register_remote_workspace_policy(
    state: &mut RemoteCoordinatorState,
    endpoint_id: &str,
    policy: crate::remote_farm_config::RemoteWorkspacePolicy,
) -> Result<(), String> {
    if endpoint_id.is_empty() || !state.workers.contains_key(endpoint_id) {
        return Err("workspace registration requires a known worker endpoint".to_string());
    }
    crate::remote_farm_config::validate_remote_workspace_policy(&policy)?;
    let mut candidate = state.clone();
    candidate.workspace_registrations.insert(endpoint_id.to_string(), policy);
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    Ok(())
}

pub fn apply_remote_workspace_lease(
    state: &mut RemoteCoordinatorState,
    endpoint_id: &str,
    request: &crunch_build::WorkspaceLeaseRequest,
) -> Result<crunch_build::WorkspaceLeasePlan, String> {
    let registration = state
        .workspace_registrations
        .get(endpoint_id)
        .ok_or_else(|| "remote worker has no workspace registration".to_string())?;
    validate_registered_workspace_request(endpoint_id, registration, request)?;
    let current = state.workspace_leases.get(&request.workspace_id);
    let plan = crunch_build::plan_workspace_lease(current, request);
    if plan.disposition != crunch_build::WorkspaceLeaseDisposition::Accepted {
        return Ok(plan);
    }
    let mut candidate = state.clone();
    let next_lease = plan.next.clone().ok_or_else(|| "accepted workspace lease has no next state".to_string())?;
    candidate.workspace_leases.insert(request.workspace_id.clone(), next_lease);
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    debug_assert_eq!(plan.disposition, crunch_build::WorkspaceLeaseDisposition::Accepted);
    debug_assert!(state.workspace_leases.contains_key(&request.workspace_id));
    Ok(plan)
}

pub fn bind_and_acquire_remote_workspace(
    state: &mut RemoteCoordinatorState,
    job_id: &RemoteJobId,
    request: &mut snix_build::buildservice::StatefulWorkspaceRequest,
) -> Result<crunch_build::WorkspaceLeasePlan, String> {
    if request.mode != snix_build::buildservice::StatefulWorkspaceMode::MutableSession {
        return Err("remote workspace binding requires mutable-session mode".to_string());
    }
    let job = state.jobs.get(job_id).ok_or_else(|| "remote workspace job is unknown".to_string())?;
    let worker_id = job
        .assigned_worker_endpoint_id
        .clone()
        .ok_or_else(|| "remote workspace job has no assigned worker".to_string())?;
    let attempt = job
        .current_attempt
        .clone()
        .ok_or_else(|| "remote workspace job has no current attempt".to_string())?;
    let authority_class = state
        .workspace_registrations
        .get(&worker_id)
        .ok_or_else(|| "remote worker has no workspace registration".to_string())?
        .authority_class
        .clone();
    request.generation = attempt.fence_generation.get();
    request.lease = Some(snix_build::buildservice::StatefulWorkspaceLeaseBinding {
        worker_id: worker_id.clone(),
        authority_class,
        job_id: job_id.as_str().to_string(),
        attempt_id: attempt.attempt_id.as_str().to_string(),
        fence_generation: attempt.fence_generation.get(),
    });
    let lease = request.lease.as_ref().ok_or_else(|| "lease binding assignment failed".to_string())?;
    let core_request = crunch_build::WorkspaceLeaseRequest {
        workspace_id: request.workspace_id.clone().ok_or_else(|| "workspace id missing".to_string())?,
        compatibility_digest_blake3: request.compatibility_digest_blake3.clone(),
        toolchain_refs: request.toolchain_refs.clone(),
        guest_path: request.guest_path.display().to_string(),
        quota: crunch_build::WorkspaceQuotaPolicy {
            bytes_max: request.quota_bytes_max,
            files_max: request.quota_files_max,
            snapshots_max: request.quota_snapshots_max,
        },
        retention_class: request.retention_class.clone(),
        generation: request.generation,
        owner: crunch_build::WorkspaceLeaseOwner {
            worker_id: lease.worker_id.clone(),
            authority_class: lease.authority_class.clone(),
            job_id: lease.job_id.clone(),
            attempt_id: lease.attempt_id.clone(),
            fence_generation: lease.fence_generation,
        },
        operation: crunch_build::WorkspaceLeaseOperation::Acquire,
    };
    let plan = apply_remote_workspace_lease(state, &worker_id, &core_request)?;
    debug_assert_eq!(core_request.owner.worker_id, worker_id);
    debug_assert_eq!(core_request.owner.fence_generation, request.generation);
    Ok(plan)
}

fn validate_registered_workspace_request(
    endpoint_id: &str,
    policy: &crate::remote_farm_config::RemoteWorkspacePolicy,
    request: &crunch_build::WorkspaceLeaseRequest,
) -> Result<(), String> {
    if !policy.modes.contains(&crate::remote_farm_config::RemoteWorkspaceMode::MutableSession) {
        return Err("remote worker does not admit mutable-session workspaces".to_string());
    }
    if request.owner.worker_id != endpoint_id {
        return Err(crunch_build::WorkspaceReasonCode::WorkerMismatch.as_str().to_string());
    }
    if request.owner.authority_class != policy.authority_class {
        return Err(crunch_build::WorkspaceReasonCode::AuthorityMismatch.as_str().to_string());
    }
    if !policy.guest_paths.contains(&request.guest_path) {
        return Err(crunch_build::WorkspaceReasonCode::GuestPathInvalid.as_str().to_string());
    }
    let is_within_quota = request.quota.bytes_max <= policy.bytes_max
        && request.quota.files_max <= policy.files_max
        && request.quota.snapshots_max <= policy.snapshots_max;
    if !is_within_quota {
        return Err(crunch_build::WorkspaceReasonCode::QuotaExceeded.as_str().to_string());
    }
    debug_assert_eq!(request.owner.worker_id, endpoint_id);
    debug_assert!(policy.guest_paths.contains(&request.guest_path));
    Ok(())
}

fn validate_external_batch_state(state: &RemoteCoordinatorState) -> Result<(), String> {
    if state.external_batch_attempts.len() > MAX_REMOTE_STATUS_ITEMS {
        return Err("external-batch-state-attempt-limit-exceeded".to_string());
    }
    let mut external_jobs = Vec::with_capacity(state.external_batch_attempts.len());
    for (dispatch_id, record) in &state.external_batch_attempts {
        if dispatch_id != &record.submit_operation.dispatch_id_blake3 {
            return Err("external-batch-state-dispatch-key-mismatch".to_string());
        }
        if record.submit_operation.operation != ExternalBatchOperationKind::Submit {
            return Err("external-batch-state-submit-operation-invalid".to_string());
        }
        if record.last_operation.dispatch_id_blake3 != *dispatch_id {
            return Err("external-batch-state-last-operation-mismatch".to_string());
        }
        crunch_build::distributed::validate_external_batch_response(&record.last_operation, &record.last_response)?;
        if record.reconcile_attempts > record.submit_operation.limits.max_reconcile_attempts {
            return Err("external-batch-state-reconcile-attempt-limit-exceeded".to_string());
        }
        if let Some(external_job_id) = record.last_response.external_job_id.as_deref() {
            let external_key = format!(
                "{}:{}:{}",
                record.submit_operation.adapter_instance_id,
                record.submit_operation.dispatcher_generation,
                external_job_id
            );
            let other_dispatch = external_jobs
                .iter()
                .find(|(known_key, _)| known_key == &external_key)
                .map(|(_, known_dispatch)| known_dispatch);
            let is_locator_conflict = other_dispatch.is_some_and(|other| other != dispatch_id);
            if is_locator_conflict {
                return Err("external-batch-state-external-job-id-conflict".to_string());
            }
            external_jobs.push((external_key, dispatch_id.clone()));
        }
        if record.worker_registered {
            let worker = state
                .workers
                .get(&record.submit_operation.expected_worker_endpoint_id)
                .ok_or_else(|| "external-batch-state-registered-worker-missing".to_string())?;
            if worker.worker_generation != record.submit_operation.dispatcher_generation {
                return Err("external-batch-state-worker-generation-mismatch".to_string());
            }
        }
        validate_external_batch_coordinator_link(state, record)?;
    }
    debug_assert_eq!(external_jobs.len(), external_jobs.iter().map(|(key, _)| key).collect::<BTreeSet<_>>().len());
    debug_assert!(state.external_batch_attempts.len() <= MAX_REMOTE_STATUS_ITEMS);
    Ok(())
}

fn validate_external_batch_coordinator_link(
    state: &RemoteCoordinatorState,
    record: &ExternalBatchCoordinatorRecord,
) -> Result<(), String> {
    let Some(job_id) = record.coordinator_job_id.as_ref() else {
        if record.resource_lease_id_blake3.is_some() || record.output_admission_digest_blake3.is_some() {
            return Err("external-batch-state-unbound-coordinator-facts".to_string());
        }
        return Ok(());
    };
    let job = state.jobs.get(job_id).ok_or_else(|| "external-batch-state-coordinator-job-missing".to_string())?;
    if job.normalized_build_key != record.submit_operation.normalized_build_key
        || job.assigned_worker_endpoint_id.as_deref()
            != Some(record.submit_operation.expected_worker_endpoint_id.as_str())
    {
        return Err("external-batch-state-coordinator-job-mismatch".to_string());
    }
    let is_current_attempt_live = job.current_attempt.as_ref().is_some_and(|attempt| attempt.phase.is_live());
    if is_current_attempt_live && record.resource_lease_id_blake3 != job.resource_lease_id_blake3 {
        return Err("external-batch-state-resource-lease-mismatch".to_string());
    }
    if record.output_admission_digest_blake3.is_some() && !job.output_admission_completed {
        return Err("external-batch-state-output-admission-mismatch".to_string());
    }
    debug_assert_eq!(job.normalized_build_key, record.submit_operation.normalized_build_key);
    debug_assert_eq!(
        job.assigned_worker_endpoint_id.as_deref(),
        Some(record.submit_operation.expected_worker_endpoint_id.as_str())
    );
    Ok(())
}

fn persist_coordinator_candidate(state: &RemoteCoordinatorState) -> Result<(), String> {
    validate_external_batch_state(state)?;
    validate_coordinator_resource_state(state)?;
    let Some(state_dir) = state.state_dir.as_deref() else {
        if state.allow_volatile_test_state {
            return Ok(());
        }
        return Err(RemoteAttemptReasonCode::PersistenceUnconfigured.as_str().to_string());
    };
    save_coordinator_state(state_dir, state)
        .map_err(|error| format!("{}: {error}", RemoteAttemptReasonCode::PersistenceFailed.as_str()))
}

fn conflicting_live_output_claim(
    state: &RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
    normalized_key: &str,
) -> Option<String> {
    for claim in &request.live_output_claims {
        if let Some(owner_key) = state.live_output_claims.get(claim)
            && owner_key != normalized_key
        {
            return Some("live-output-claim-conflict".to_string());
        }
    }
    None
}

fn existing_job_decision(
    state: &RemoteCoordinatorState,
    normalized_key: &str,
) -> Option<RemoteCoordinatorDispatchDecision> {
    state
        .jobs
        .values()
        .find(|job| job.normalized_build_key == normalized_key)
        .map(existing_job_to_dispatch_decision)
}

fn existing_job_to_dispatch_decision(job: &RemoteCoordinatorJobSummary) -> RemoteCoordinatorDispatchDecision {
    match job.phase {
        RemoteCoordinatorJobPhase::Finished if job.result_available => {
            RemoteCoordinatorDispatchDecision::RedeliverResult {
                job_id: job.job_id.clone(),
                normalized_build_key: job.normalized_build_key.clone(),
            }
        }
        RemoteCoordinatorJobPhase::Lost => RemoteCoordinatorDispatchDecision::Reject {
            reason: job
                .last_attempt_reason_code
                .map(RemoteAttemptReasonCode::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| {
                    format!(
                        "restart-state-unavailable-phase-{}",
                        job.lost_phase.map(RemoteFailurePhase::as_status_label).unwrap_or("unknown")
                    )
                }),
        },
        _ => RemoteCoordinatorDispatchDecision::AttachExisting {
            job_id: job.job_id.clone(),
            normalized_build_key: job.normalized_build_key.clone(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CoordinatorWorkerPlacement {
    worker_endpoint_id: String,
    resource_fit: crunch_build::ResourceFitClass,
    locality: Option<RemoteVerifiedLocalitySummary>,
}

fn select_coordinator_worker(
    state: &RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
) -> Result<Option<CoordinatorWorkerPlacement>, String> {
    let placements = coordinator_worker_placement_candidates(state, request)?;
    if placements.is_empty() {
        return Ok(None);
    }
    let facts = placements
        .iter()
        .map(|placement| RemoteWorkerPlacementFacts {
            worker_endpoint_id: placement.worker_endpoint_id.clone(),
            resource_fit: placement.resource_fit,
            content_locality: placement
                .locality
                .as_ref()
                .map_or(crunch_build::ContentLocalityClass::Unknown, |summary| summary.content_locality),
            transfer_cost: placement
                .locality
                .as_ref()
                .map_or(crunch_build::TransferCostClass::Unknown, |summary| summary.transfer_cost),
        })
        .collect::<Vec<_>>();
    let scheduling_policy = crunch_build::SchedulingPolicy {
        schema: crunch_build::scheduling::SCHEDULING_POLICY_SCHEMA.to_string(),
        policy_id: crunch_build::scheduling::DEFAULT_SCHEDULING_POLICY_ID.to_string(),
        preference_order: vec![
            crunch_build::scheduling::PreferenceField::KnownGraph,
            crunch_build::scheduling::PreferenceField::ResourceFit,
            crunch_build::scheduling::PreferenceField::LocalityTransfer,
        ],
        aged_after_epochs: crunch_build::scheduling::DEFAULT_AGED_AFTER_EPOCHS,
        protected_after_epochs: crunch_build::scheduling::DEFAULT_PROTECTED_AFTER_EPOCHS,
    };
    let ranked = rank_remote_worker_placement_candidates(&scheduling_policy, &facts)
        .map_err(|reason| reason.as_str().to_string())?;
    let selected = &ranked[0].worker_endpoint_id;
    let placement = placements
        .into_iter()
        .find(|placement| &placement.worker_endpoint_id == selected)
        .ok_or_else(|| "remote-worker-placement-selection-missing".to_string())?;
    debug_assert!(!placement.worker_endpoint_id.is_empty());
    debug_assert_ne!(placement.resource_fit, crunch_build::ResourceFitClass::Constrained);
    Ok(Some(placement))
}

fn coordinator_worker_placement_candidates(
    state: &RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
) -> Result<Vec<CoordinatorWorkerPlacement>, String> {
    let active_leases = state.resource_leases.values().cloned().collect::<Vec<_>>();
    let mut placements = Vec::with_capacity(state.workers.len());
    for worker in state.workers.values() {
        if worker_satisfies_request(worker, request).is_err()
            || active_jobs_for_worker(state, &worker.endpoint_id) >= worker.concurrency
        {
            continue;
        }
        let resource_fit = match (&request.resource_requirements, &worker.resource_inventory) {
            (Some(requirements), Some(inventory)) => {
                match plan_remote_resource_availability(&worker.endpoint_id, inventory, requirements, &active_leases) {
                    Ok(plan) => plan.resource_fit,
                    Err(_) => continue,
                }
            }
            (Some(_), None) => continue,
            (None, _) => crunch_build::ResourceFitClass::Unknown,
        };
        placements.push(CoordinatorWorkerPlacement {
            worker_endpoint_id: worker.endpoint_id.clone(),
            resource_fit,
            locality: matching_verified_locality(state, worker, request),
        });
    }
    debug_assert!(placements.len() <= state.workers.len());
    debug_assert!(placements.iter().all(|placement| !placement.worker_endpoint_id.is_empty()));
    Ok(placements)
}

fn matching_verified_locality(
    state: &RemoteCoordinatorState,
    worker: &RemoteWorkerRegistration,
    request: &RemoteCoordinatorBuildRequest,
) -> Option<RemoteVerifiedLocalitySummary> {
    let scope = request.locality_scope.as_ref()?;
    state
        .verified_locality_observations
        .values()
        .find(|summary| {
            summary.worker_endpoint_id == worker.endpoint_id
                && summary.worker_generation == worker.worker_generation
                && summary.scope == *scope
        })
        .cloned()
}

fn no_matching_worker_reason(state: &RemoteCoordinatorState, request: &RemoteCoordinatorBuildRequest) -> String {
    debug_assert!(state.workers.len() <= MAX_REMOTE_STATUS_ITEMS);
    debug_assert!(!request.required_system.is_empty());
    if state.workers.is_empty() {
        return "no-workers-registered".to_string();
    }
    for worker in state.workers.values() {
        if let Err(reason) = worker_satisfies_request(worker, request) {
            return reason;
        }
        if active_jobs_for_worker(state, &worker.endpoint_id) >= worker.concurrency {
            return "worker-concurrency-limit".to_string();
        }
        if request.resource_requirements.is_some() && worker.resource_inventory.is_none() {
            return "remote-worker-resource-inventory-missing".to_string();
        }
        if let (Some(requirements), Some(inventory)) = (&request.resource_requirements, &worker.resource_inventory) {
            let leases = state.resource_leases.values().cloned().collect::<Vec<_>>();
            if let Err(reason) =
                plan_remote_resource_availability(&worker.endpoint_id, inventory, requirements, &leases)
            {
                return reason.as_str().to_string();
            }
        }
    }
    "capability-mismatch".to_string()
}

fn worker_satisfies_request(
    worker: &RemoteWorkerRegistration,
    request: &RemoteCoordinatorBuildRequest,
) -> Result<(), String> {
    validate_worker_registration(worker)?;
    validate_coordinator_build_request(request)?;
    require_member("system", &worker.systems, &request.required_system)?;
    require_all_members("feature", &worker.feature_labels, &request.required_features)?;
    require_member("sandbox", &worker.sandbox_modes, &request.required_sandbox_mode)?;
    require_member("network", &worker.network_modes, &request.required_network_mode)?;
    require_member("store-prefix", &worker.logical_store_prefixes, &request.request.store_prefix)?;
    if !worker.output_signing_key_ids.iter().any(|key| request.trusted_output_keys.contains(key)) {
        return Err("output-trust-mismatch".to_string());
    }
    Ok(())
}

fn require_member(label: &str, available: &[String], required: &str) -> Result<(), String> {
    if available.iter().any(|value| value == required) {
        return Ok(());
    }
    Err(format!("remote-worker-{label}-mismatch"))
}

fn require_all_members(label: &str, available: &[String], required: &[String]) -> Result<(), String> {
    for value in required {
        require_member(label, available, value)?;
    }
    Ok(())
}

fn active_jobs_for_worker(state: &RemoteCoordinatorState, endpoint_id: &str) -> u32 {
    state
        .jobs
        .values()
        .filter(|job| job.assigned_worker_endpoint_id.as_deref() == Some(endpoint_id))
        .filter(|job| job.phase.is_live())
        .count()
        .try_into()
        .unwrap_or(MAX_REMOTE_WORKER_CONCURRENCY)
}

fn generate_remote_assignment_nonce() -> Result<RemoteAssignmentNonce, String> {
    let mut entropy = [0_u8; REMOTE_ASSIGNMENT_NONCE_BYTES];
    OsRng
        .try_fill_bytes(&mut entropy)
        .map_err(|error| format!("remote-assignment-nonce-generation-failed: {error}"))?;
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_ASSIGNMENT_NONCE_LABEL);
    hasher.update(&entropy);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert!(is_blake3_hex_digest(&digest));
    debug_assert_eq!(entropy.len(), REMOTE_ASSIGNMENT_NONCE_BYTES);
    RemoteAssignmentNonce::new(digest).map_err(|reason| reason.as_str().to_string())
}

fn coordinator_job_id(normalized_key: &str, assignment_nonce: &RemoteAssignmentNonce) -> Result<RemoteJobId, String> {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_COORDINATOR_JOB_ID_LABEL);
    hash_labeled_str(&mut hasher, "normalized-key", normalized_key);
    hash_labeled_str(&mut hasher, "assignment-nonce", assignment_nonce.as_str());
    let digest = hasher.finalize().to_hex().to_string();
    RemoteJobId::new(digest).map_err(|reason| reason.as_str().to_string())
}

pub fn replay_coordinator_attempt_log(
    state: &RemoteCoordinatorState,
    job_id: &RemoteJobId,
    request: RemoteAttemptLogReplayRequest,
) -> Result<RemoteAttemptLogReplayPlan, String> {
    let state_dir = state
        .state_dir
        .as_deref()
        .ok_or_else(|| RemoteAttemptReasonCode::PersistenceUnconfigured.as_str().to_string())?;
    let job = state.jobs.get(job_id).ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let summary = job.immutable_log.as_ref().ok_or_else(|| "attempt-log-not-recorded".to_string())?;
    let policy = immutable_attempt_log_policy(summary.retention_policy)?;
    let replay =
        crate::remote_attempt_log_store::replay_remote_attempt_log(state_dir, &summary.scope, request, policy)?;
    let manifest =
        crate::remote_attempt_log_store::load_remote_attempt_log_manifest(state_dir, &summary.scope, policy)?;
    if remote_attempt_log_control_summary(&manifest, summary.retention_policy) != *summary {
        return Err("attempt-log-coordinator-summary-mismatch".to_string());
    }
    Ok(replay)
}

fn validate_log_policy(policy: RemoteLogRetentionPolicy) -> Result<(), String> {
    if policy.max_chunks == 0 || policy.max_chunks > MAX_REMOTE_LOG_CHUNKS {
        return Err("remote-log-chunk-limit-invalid".to_string());
    }
    if policy.max_bytes == 0 || policy.max_bytes > MAX_REMOTE_LOG_BYTES {
        return Err("remote-log-byte-limit-invalid".to_string());
    }
    Ok(())
}

pub fn coordinator_status_snapshot(
    endpoint_id: &str,
    configured_concurrency: u32,
    state: &RemoteCoordinatorState,
    transient_attempt_reason_codes: &[RemoteAttemptReasonCode],
    tickets: &[RemoteTicket],
) -> Result<RemoteCoordinatorStatusSnapshot, String> {
    if endpoint_id.is_empty() {
        return Err("remote-status-endpoint-empty".to_string());
    }
    if configured_concurrency == 0 || configured_concurrency > MAX_REMOTE_WORKER_CONCURRENCY {
        return Err("remote-status-concurrency-invalid".to_string());
    }
    validate_coordinator_resource_state(state)?;
    if state.jobs.len() > MAX_REMOTE_STATUS_ITEMS
        || state.external_batch_attempts.len() > MAX_REMOTE_STATUS_ITEMS
        || tickets.len() > MAX_REMOTE_STATUS_ITEMS
    {
        return Err(format!("remote-status-item-count-exceeds-{MAX_REMOTE_STATUS_ITEMS}"));
    }
    let mut queued_jobs = Vec::with_capacity(state.jobs.len());
    let mut active_jobs = Vec::with_capacity(state.jobs.len());
    let mut recent_jobs = Vec::with_capacity(state.jobs.len());
    let mut recent_failures = Vec::with_capacity(state.jobs.len());
    let mut log_cursors = Vec::with_capacity(state.jobs.len());
    for job in state.jobs.values() {
        push_status_job(&mut queued_jobs, &mut active_jobs, &mut recent_jobs, state, job)?;
        push_status_failure(&mut recent_failures, job);
        log_cursors.push(coordinator_log_cursor_status(state, job)?);
    }
    let snapshot = RemoteCoordinatorStatusSnapshot {
        endpoint_id: endpoint_id.to_string(),
        configured_concurrency,
        worker_count: bounded_runtime_count_u32(state.workers.len())?,
        workers: state
            .workers
            .values()
            .map(|worker| remote_worker_status(state, worker))
            .collect::<Result<Vec<_>, _>>()?,
        queued_jobs,
        active_jobs,
        recent_jobs,
        recent_failures,
        log_cursors,
        attempt_reason_codes: transient_attempt_reason_codes
            .iter()
            .copied()
            .chain(state.jobs.values().filter_map(|job| job.last_attempt_reason_code))
            .take(MAX_REMOTE_STATUS_ITEMS)
            .collect(),
        resource_leases: state.resource_leases.values().take(MAX_REMOTE_STATUS_ITEMS).cloned().collect(),
        external_batch_attempts: state.external_batch_attempts.values().map(external_batch_attempt_status).collect(),
        tickets: tickets.iter().map(redacted_ticket_view).collect(),
        non_claims: REMOTE_RESOURCE_STATUS_NON_CLAIMS
            .iter()
            .copied()
            .chain(std::iter::once(EXTERNAL_BATCH_NON_CLAIM))
            .map(str::to_string)
            .collect(),
    };
    debug_assert_eq!(snapshot.endpoint_id, endpoint_id);
    debug_assert_eq!(snapshot.log_cursors.len(), state.jobs.len());
    Ok(snapshot)
}

fn external_batch_attempt_status(record: &ExternalBatchCoordinatorRecord) -> ExternalBatchAttemptStatus {
    ExternalBatchAttemptStatus {
        dispatch_id_blake3: record.submit_operation.dispatch_id_blake3.clone(),
        adapter_instance_id: bounded_untrusted_text(&record.submit_operation.adapter_instance_id),
        dispatcher_profile_ref_blake3: record.submit_operation.dispatcher_profile_ref_blake3.clone(),
        provider_class: bounded_untrusted_text(&record.submit_operation.provider_class),
        dispatcher_generation: record.submit_operation.dispatcher_generation,
        allocation_job_id: record.submit_operation.job_id.clone(),
        allocation_attempt_id: record.submit_operation.attempt_id.clone(),
        allocation_fence_generation: record.submit_operation.fence_generation,
        external_job_id: record.last_response.external_job_id.as_deref().map(bounded_untrusted_text),
        state: record.last_response.state,
        resources: record.submit_operation.resources.clone(),
        worker_endpoint_id: bounded_untrusted_text(&record.submit_operation.expected_worker_endpoint_id),
        worker_registered: record.worker_registered,
        coordinator_job_id: record.coordinator_job_id.clone(),
        resource_lease_id_blake3: record.resource_lease_id_blake3.clone(),
        output_admission_completed: record.output_admission_digest_blake3.is_some(),
        reconcile_attempts: record.reconcile_attempts,
        reason_code: bounded_untrusted_text(&record.last_response.reason_code),
    }
}

fn remote_worker_status(
    state: &RemoteCoordinatorState,
    worker: &RemoteWorkerRegistration,
) -> Result<RemoteWorkerStatus, String> {
    let resource_remaining = match &worker.resource_inventory {
        Some(inventory) => Some(
            remote_resource_remaining_capacity(
                &worker.endpoint_id,
                inventory,
                &state.resource_leases.values().cloned().collect::<Vec<_>>(),
            )
            .map_err(|reason| reason.as_str().to_string())?,
        ),
        None => None,
    };
    let status = RemoteWorkerStatus {
        endpoint_id: worker.endpoint_id.clone(),
        worker_generation: worker.worker_generation,
        systems: bounded_string_list(&worker.systems),
        feature_labels: bounded_string_list(&worker.feature_labels),
        sandbox_modes: bounded_string_list(&worker.sandbox_modes),
        network_modes: bounded_string_list(&worker.network_modes),
        logical_store_prefixes: bounded_string_list(&worker.logical_store_prefixes),
        concurrency: worker.concurrency,
        resource_inventory: worker.resource_inventory.clone(),
        resource_remaining,
        output_signing_key_count: bounded_runtime_count_u32(worker.output_signing_key_ids.len())?,
        resumable_job_count: bounded_runtime_count_u32(worker.resumable_jobs.len())?,
        workspace_modes: worker
            .workspace_policy
            .as_ref()
            .map(|policy| policy.modes.iter().map(|mode| mode.as_str().to_string()).collect())
            .unwrap_or_default(),
        workspace_authority_class: worker
            .workspace_policy
            .as_ref()
            .map(|policy| bounded_untrusted_text(&policy.authority_class)),
    };
    debug_assert_eq!(status.endpoint_id, worker.endpoint_id);
    debug_assert_eq!(status.worker_generation, worker.worker_generation);
    Ok(status)
}

fn bounded_string_list(values: &[String]) -> Vec<String> {
    values.iter().take(MAX_REMOTE_STATUS_ITEMS).map(|value| bounded_untrusted_text(value)).collect()
}

fn push_status_failure(recent_failures: &mut Vec<RemoteFailureStatus>, job: &RemoteCoordinatorJobSummary) {
    if job.phase != RemoteCoordinatorJobPhase::Lost && job.short_error.is_none() {
        return;
    }
    recent_failures.push(RemoteFailureStatus {
        job_id: job.job_id.clone(),
        phase: job.phase,
        lost_phase: job.lost_phase,
        retry_class: RemoteRetryClass::Terminal,
        short_error: job.short_error.as_deref().map(bounded_untrusted_text),
        attempt_reason_code: job.last_attempt_reason_code,
        failure_debug: job.failure_debug.clone(),
    });
}

fn coordinator_log_cursor_status(
    _state: &RemoteCoordinatorState,
    job: &RemoteCoordinatorJobSummary,
) -> Result<RemoteLogCursorStatus, String> {
    let summary = job.immutable_log.as_ref();
    Ok(RemoteLogCursorStatus {
        job_id: job.job_id.clone(),
        attempt_id: summary.map(|value| value.scope.attempt_id.clone()),
        fence_generation: summary.map(|value| value.scope.fence_generation),
        retained_start_cursor: summary.map_or(0, |value| value.retained_start_cursor),
        next_cursor: summary.map_or(0, |value| value.next_cursor),
        retained_record_count: summary.map_or(0, |value| value.retained_record_count),
        retained_payload_bytes: summary.map_or(0, |value| value.retained_payload_bytes),
        head_record_blake3: summary.and_then(|value| value.head_record_blake3.clone()),
        head_segment_blake3: summary.and_then(|value| value.head_segment_blake3.clone()),
        manifest_blake3: summary.map(|value| value.manifest_blake3.clone()),
        truncation_anchor_blake3: summary.and_then(|value| value.truncation_anchor_blake3.clone()),
        truncated: summary.is_some_and(|value| value.truncated),
    })
}

fn bounded_untrusted_text(value: &str) -> String {
    if text_looks_secret_bearing(value) {
        return SECRET_REDACTION.to_string();
    }
    let sanitized = value.chars().map(|ch| if ch.is_control() { '�' } else { ch }).collect::<String>();
    if sanitized.len() <= MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES {
        return sanitized;
    }
    let mut bounded = String::new();
    for ch in sanitized.chars() {
        if bounded.len().saturating_add(ch.len_utf8()) > MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES {
            break;
        }
        bounded.push(ch);
    }
    bounded.push_str("<truncated>");
    bounded
}

fn text_looks_secret_bearing(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("bearer ")
        || lower.contains("token=")
        || lower.contains("secret")
        || lower.contains("private-key")
        || lower.contains("/home/")
}

fn push_status_job(
    queued: &mut Vec<RemoteCoordinatorJobStatus>,
    active: &mut Vec<RemoteCoordinatorJobStatus>,
    recent: &mut Vec<RemoteCoordinatorJobStatus>,
    state: &RemoteCoordinatorState,
    job: &RemoteCoordinatorJobSummary,
) -> Result<(), String> {
    let status = coordinator_job_status(state, job)?;
    match job.phase {
        RemoteCoordinatorJobPhase::Queued => queued.push(status),
        RemoteCoordinatorJobPhase::Running => active.push(status),
        RemoteCoordinatorJobPhase::Finished | RemoteCoordinatorJobPhase::Lost => recent.push(status),
    }
    Ok(())
}

fn coordinator_job_status(
    _state: &RemoteCoordinatorState,
    job: &RemoteCoordinatorJobSummary,
) -> Result<RemoteCoordinatorJobStatus, String> {
    Ok(RemoteCoordinatorJobStatus {
        job_id: job.job_id.clone(),
        phase: job.phase,
        worker_endpoint_id: job.assigned_worker_endpoint_id.clone(),
        short_error: job.short_error.as_deref().map(bounded_untrusted_text),
        immutable_log: job.immutable_log.clone(),
        observability_health: job.observability_health.clone(),
        failure_debug: job.failure_debug.clone(),
        attempt_id: job.current_attempt.as_ref().map(|attempt| attempt.attempt_id.clone()),
        fence_generation: job.current_attempt.as_ref().map(|attempt| attempt.fence_generation),
        attempt_phase: job.current_attempt.as_ref().map(|attempt| attempt.phase),
        attempt_reason_code: job.last_attempt_reason_code,
        resource_requirements: job.resource_requirements.clone(),
        resource_lease_id_blake3: job.resource_lease_id_blake3.clone(),
        resource_fit: job.resource_fit,
        locality: job.locality.clone(),
    })
}

fn bounded_runtime_count_u32(count: usize) -> Result<u32, String> {
    u32::try_from(count).map_err(|_| "remote-status-count-overflow".to_string())
}

struct RemoteReconnectInput<'a> {
    active_session_id: &'a str,
    candidate_session_id: &'a str,
    attempts: u32,
    max_attempts: u32,
}

pub const DECIDE_REMOTE_RECONNECT: fn(&str, &str, u32, u32) -> RemoteReconnectDecision =
    |active_session_id, candidate_session_id, attempts, max_attempts| {
        decide_remote_reconnect_core(RemoteReconnectInput {
            active_session_id,
            candidate_session_id,
            attempts,
            max_attempts,
        })
    };
pub use DECIDE_REMOTE_RECONNECT as decide_remote_reconnect;
const _: fn(&str, &str, u32, u32) -> RemoteReconnectDecision = decide_remote_reconnect;

fn decide_remote_reconnect_core(input: RemoteReconnectInput<'_>) -> RemoteReconnectDecision {
    if input.active_session_id == input.candidate_session_id {
        return RemoteReconnectDecision::SameSession;
    }
    if input.attempts >= input.max_attempts {
        return RemoteReconnectDecision::BackoffRequired;
    }
    RemoteReconnectDecision::NewSession
}

pub fn decide_session_lease_release(phase: RemoteCoordinatorJobPhase) -> RemoteLeaseReleaseDecision {
    if phase.is_live() {
        return RemoteLeaseReleaseDecision::Retain;
    }
    RemoteLeaseReleaseDecision::Release
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteCoordinatorTerminationCause {
    Cancellation,
    Timeout,
    WorkerLoss,
}

impl RemoteCoordinatorTerminationCause {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cancellation => "remote-attempt-cancelled",
            Self::Timeout => "remote-attempt-timed-out",
            Self::WorkerLoss => "remote-worker-lost",
        }
    }

    fn failure_phase(self) -> RemoteFailurePhase {
        match self {
            Self::Cancellation => RemoteFailurePhase::Queue,
            Self::Timeout => RemoteFailurePhase::BuildExecution,
            Self::WorkerLoss => RemoteFailurePhase::TransportSetup,
        }
    }
}

pub fn terminate_coordinator_attempt(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    cause: RemoteCoordinatorTerminationCause,
) -> Result<(), String> {
    mark_coordinator_attempt_lost_with_diagnostic(state, binding, cause.failure_phase(), cause.as_str())
}

pub fn mark_coordinator_attempt_lost(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    lost_phase: RemoteFailurePhase,
) -> Result<(), String> {
    mark_coordinator_attempt_lost_with_diagnostic(
        state,
        binding,
        lost_phase,
        RemoteAttemptReasonCode::Superseded.as_str(),
    )
}

fn mark_coordinator_attempt_lost_with_diagnostic(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    lost_phase: RemoteFailurePhase,
    diagnostic: &str,
) -> Result<(), String> {
    let current = state
        .jobs
        .get(&binding.job_id)
        .and_then(|job| job.current_attempt.as_ref())
        .ok_or_else(|| RemoteAttemptReasonCode::LegacyStateRejected.as_str().to_string())?;
    if current.attempt_id != binding.attempt_id || current.fence_generation != binding.fence_generation {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    let current = current.clone();
    let mut candidate = state.clone();
    release_current_job_resource_lease(&mut candidate, &binding.job_id, &current)?;
    let job = candidate
        .jobs
        .get_mut(&binding.job_id)
        .ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let mut superseded = current;
    superseded.phase = RemoteAttemptPhase::Superseded;
    job.current_attempt = Some(superseded);
    job.phase = RemoteCoordinatorJobPhase::Lost;
    job.lost_phase = Some(lost_phase);
    job.result_available = false;
    job.output_admission_completed = false;
    job.resource_lease_id_blake3 = None;
    job.last_attempt_reason_code = Some(RemoteAttemptReasonCode::Superseded);
    job.short_error = Some(diagnostic.to_string());
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    assert_eq!(state.jobs[&binding.job_id].phase, RemoteCoordinatorJobPhase::Lost);
    assert!(state.jobs[&binding.job_id].resource_lease_id_blake3.is_none());
    Ok(())
}

impl RemoteFailurePhase {
    fn as_status_label(self) -> &'static str {
        match self {
            Self::TransportSetup => "transport-setup",
            Self::Authentication => "authentication",
            Self::RequestValidation => "request-validation",
            Self::InputSync => "input-sync",
            Self::Queue => "queue",
            Self::BuildExecution => "build-execution",
            Self::OutputImport => "output-import",
        }
    }
}

/// Encode one frame in a zeroizing caller-owned buffer.
///
/// The returned bytes can contain bearer material and wipe on drop.
pub fn encode_remote_frame(frame: &RemoteFrame) -> Result<zeroize::Zeroizing<Vec<u8>>, String> {
    encode_remote_frame_zeroizing(frame)
}

struct BoundedFrameCountingWriter {
    bytes_written: usize,
    bytes_max: usize,
    is_exceeded: bool,
}

impl Write for BoundedFrameCountingWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let next_len = self
            .bytes_written
            .checked_add(buffer.len())
            .ok_or_else(|| std::io::Error::other("remote frame count overflow"))?;
        if next_len > self.bytes_max {
            self.is_exceeded = true;
            return Err(std::io::Error::other("remote frame count exceeds limit"));
        }
        self.bytes_written = next_len;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct FixedCapacityFrameWriter<'a> {
    output: &'a mut Vec<u8>,
    bytes_max: usize,
}

impl Write for FixedCapacityFrameWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let next_len = self
            .output
            .len()
            .checked_add(buffer.len())
            .ok_or_else(|| std::io::Error::other("remote frame write overflow"))?;
        if next_len > self.bytes_max {
            return Err(std::io::Error::other("remote frame write exceeds counted length"));
        }
        self.output.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn count_remote_frame_payload(frame: &RemoteFrame) -> Result<usize, String> {
    let mut counter = BoundedFrameCountingWriter {
        bytes_written: 0,
        bytes_max: MAX_REMOTE_FRAME_BYTES,
        is_exceeded: false,
    };
    if let Err(error) = serde_json::to_writer(&mut counter, frame) {
        if counter.is_exceeded {
            return Err(format!("remote-frame-payload-exceeds-{MAX_REMOTE_FRAME_BYTES}"));
        }
        return Err(format!("serializing remote frame: {error}"));
    }
    assert!(counter.bytes_written <= MAX_REMOTE_FRAME_BYTES);
    Ok(counter.bytes_written)
}

fn encode_remote_frame_zeroizing(frame: &RemoteFrame) -> Result<zeroize::Zeroizing<Vec<u8>>, String> {
    let counted_len = count_remote_frame_payload(frame)?;
    let mut payload = zeroize::Zeroizing::new(Vec::with_capacity(counted_len));
    let payload_capacity = payload.capacity();
    assert!(payload_capacity >= counted_len);
    serde_json::to_writer(
        &mut FixedCapacityFrameWriter {
            output: &mut payload,
            bytes_max: counted_len,
        },
        frame,
    )
    .map_err(|error| format!("serializing remote frame: {error}"))?;
    assert_eq!(payload.len(), counted_len);
    assert_eq!(payload.capacity(), payload_capacity);
    let payload_len = u32::try_from(counted_len).map_err(|_| "remote-frame-length-overflow".to_string())?;
    let encoded_capacity = REMOTE_FRAME_HEADER_BYTES
        .checked_add(counted_len)
        .ok_or_else(|| "remote-frame-length-overflow".to_string())?;
    let mut encoded = zeroize::Zeroizing::new(Vec::with_capacity(encoded_capacity));
    encoded.extend_from_slice(&payload_len.to_be_bytes());
    encoded.extend_from_slice(&payload);
    assert_eq!(encoded.len(), encoded_capacity);
    Ok(encoded)
}

pub fn decode_remote_frame(encoded: &[u8]) -> Result<RemoteFrame, String> {
    if encoded.len() < REMOTE_FRAME_HEADER_BYTES {
        return Err("remote-frame-header-incomplete".to_string());
    }
    let payload_len = frame_payload_len(encoded)?;
    if payload_len > MAX_REMOTE_FRAME_BYTES {
        return Err(format!("remote-frame-payload-exceeds-{MAX_REMOTE_FRAME_BYTES}"));
    }
    let expected_len = REMOTE_FRAME_HEADER_BYTES.saturating_add(payload_len);
    if encoded.len() != expected_len {
        return Err("remote-frame-length-mismatch-or-unframed-stdout".to_string());
    }
    let payload = &encoded[REMOTE_FRAME_HEADER_BYTES..expected_len];
    serde_json::from_slice(payload).map_err(|err| format!("remote-frame-json-invalid: {err}"))
}

pub fn write_remote_frame(mut writer: impl Write, frame: &RemoteFrame) -> Result<(), String> {
    let encoded = encode_remote_frame_zeroizing(frame)?;
    writer.write_all(&encoded).map_err(|err| format!("writing remote frame: {err}"))
}

pub fn read_remote_frame(mut reader: impl Read) -> Result<RemoteFrame, String> {
    let mut header = [0_u8; REMOTE_FRAME_HEADER_BYTES];
    reader.read_exact(&mut header).map_err(|err| format!("reading remote frame header: {err}"))?;
    let payload_len = frame_payload_len(&header)?;
    if payload_len > MAX_REMOTE_FRAME_BYTES {
        return Err(format!("remote-frame-payload-exceeds-{MAX_REMOTE_FRAME_BYTES}"));
    }
    let mut payload = zeroize::Zeroizing::new(vec![0_u8; payload_len]);
    reader.read_exact(&mut payload).map_err(|err| format!("reading remote frame payload: {err}"))?;
    serde_json::from_slice(&payload).map_err(|err| format!("remote-frame-json-invalid: {err}"))
}

pub fn decode_remote_frame_stream(encoded: &[u8]) -> Result<Vec<RemoteFrame>, String> {
    let mut frames = Vec::with_capacity(MAX_REMOTE_STDIO_FRAME_COUNT.min(encoded.len()));
    let mut offset_bytes = 0_usize;
    while offset_bytes < encoded.len() {
        if frames.len() >= MAX_REMOTE_STDIO_FRAME_COUNT {
            return Err(format!("remote-stdio-frame-count-exceeds-{MAX_REMOTE_STDIO_FRAME_COUNT}"));
        }
        let remaining = encoded.len().saturating_sub(offset_bytes);
        if remaining < REMOTE_FRAME_HEADER_BYTES {
            return Err("remote-frame-header-incomplete-or-unframed-stdout".to_string());
        }
        let payload_len = frame_payload_len(&encoded[offset_bytes..])?;
        if payload_len > MAX_REMOTE_FRAME_BYTES {
            return Err(format!("remote-frame-payload-exceeds-{MAX_REMOTE_FRAME_BYTES}"));
        }
        let frame_len = REMOTE_FRAME_HEADER_BYTES
            .checked_add(payload_len)
            .ok_or_else(|| "remote-frame-length-overflow".to_string())?;
        let end_bytes = offset_bytes
            .checked_add(frame_len)
            .ok_or_else(|| "remote-frame-stream-offset-overflow".to_string())?;
        if end_bytes > encoded.len() {
            return Err("remote-frame-stream-truncated-or-unframed-stdout".to_string());
        }
        let frame = decode_remote_frame(&encoded[offset_bytes..end_bytes])?;
        frames.push(frame);
        offset_bytes = end_bytes;
    }
    debug_assert_eq!(offset_bytes, encoded.len());
    debug_assert!(frames.len() <= MAX_REMOTE_STDIO_FRAME_COUNT);
    Ok(frames)
}

pub fn validate_stdio_child_output(
    output: &RemoteStdioChildOutput,
) -> Result<RemoteStdioTranscript, RemoteFailureClassification> {
    validate_stdio_child_output_for_binding(output, RemoteTransportBinding::Stdio)
}

pub fn validate_stdio_child_output_for_binding(
    output: &RemoteStdioChildOutput,
    binding: RemoteTransportBinding,
) -> Result<RemoteStdioTranscript, RemoteFailureClassification> {
    if !output.status_success {
        return Err(remote_failure(
            RemoteFailurePhase::TransportSetup,
            RemoteRetryClass::Terminal,
            "stdio-child-exit-failed".to_string(),
        ));
    }
    let frames = decode_remote_frame_stream(&output.stdout)
        .map_err(|reason| remote_failure(RemoteFailurePhase::RequestValidation, RemoteRetryClass::Terminal, reason))?;
    let transcript = RemoteStdioTranscript {
        binding,
        frames,
        stderr_summary: bounded_stderr_summary(&output.stderr),
        telemetry: RemoteTelemetryBuffer {
            events: Vec::new(),
            accepted_events: 0,
            dropped_events: 0,
            last_drop_reason: None,
        },
        streaming_output: None,
        trace_context_health: None,
    };
    debug_assert_eq!(transcript.binding, binding);
    debug_assert!(transcript.frames.len() <= MAX_REMOTE_STDIO_FRAME_COUNT);
    Ok(transcript)
}

pub fn validate_stdio_child_exchange_output(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    output: &RemoteStdioChildOutput,
) -> Result<RemoteStdioExchangeReport, RemoteFailureClassification> {
    let transcript = validate_stdio_child_output(output)?;
    let admission = validate_remote_builder_frames_output_import(request, trusted_output_keys, &transcript.frames)
        .map_err(|reason| remote_failure(RemoteFailurePhase::OutputImport, RemoteRetryClass::Terminal, reason))?;
    Ok(RemoteStdioExchangeReport {
        binding: transcript.binding,
        frames: transcript.frames,
        stderr_summary: transcript.stderr_summary,
        admission,
    })
}

fn write_remote_control_frame(writer: &mut impl Write, frame: &RemoteFrame) -> Result<(), String> {
    write_remote_frame(&mut *writer, frame)?;
    writer.flush().map_err(|err| format!("flushing remote control frame: {err}"))
}

fn read_expected_remote_frame(reader: &mut impl Read, expected: RemoteFrameKind) -> Result<RemoteFrame, String> {
    let frame = read_remote_frame(&mut *reader)?;
    if frame.kind() != expected {
        return Err(format!(
            "unexpected-remote-control-frame expected={} actual={}",
            expected.as_str(),
            frame.kind().as_str()
        ));
    }
    Ok(frame)
}

fn command_client_frames(
    command: &RemoteStdioCommand,
) -> Result<
    (
        RemoteHello,
        Option<RemoteTraceContext>,
        TicketAuthRequest,
        ConcreteBuildRequest,
        RemoteInputManifest,
    ),
    String,
> {
    let mut hello = None;
    let mut trace_context = None;
    let mut is_trace_frame_seen = false;
    let mut auth = None;
    let mut request = None;
    let mut input_manifest = None;
    for frame in &command.input_frames {
        match frame {
            RemoteFrame::Hello { hello: value } => hello = Some(value.clone()),
            RemoteFrame::TraceContext { context } => {
                is_trace_frame_seen = true;
                trace_context = context.clone();
            }
            RemoteFrame::AuthTicket { auth: value } => auth = Some(value.clone()),
            RemoteFrame::BuildRequest { request: value } => request = Some(value.clone()),
            RemoteFrame::InputManifest { manifest: value } => input_manifest = Some(value.clone()),
            _ => {}
        }
    }
    let hello = hello.ok_or_else(|| "remote-production-hello-frame-missing".to_string())?;
    let is_trace_negotiated = hello.capabilities.iter().any(|capability| capability == REMOTE_TRACE_CONTEXT_CAPABILITY);
    if is_trace_negotiated != is_trace_frame_seen {
        return Err("remote-production-trace-capability-frame-mismatch".to_string());
    }
    debug_assert_eq!(is_trace_negotiated, is_trace_frame_seen);
    debug_assert!(!hello.endpoint_id.is_empty());
    Ok((
        hello,
        trace_context,
        auth.ok_or_else(|| "remote-production-auth-frame-missing".to_string())?,
        request.ok_or_else(|| "remote-production-build-request-frame-missing".to_string())?,
        input_manifest.ok_or_else(|| "remote-production-input-manifest-frame-missing".to_string())?,
    ))
}

struct RemoteTransferSendInput<'a> {
    prepared: &'a crate::remote_transfer::PreparedRemoteTransfer,
    policy: RemoteTransferPolicy,
    direction: crate::remote_transfer::RemoteTransferDirection,
    demand_frame: RemoteTransferDemandFrame,
    interrupt_after_chunks: Option<u32>,
}

fn send_prepared_remote_transfer(
    reader: &mut impl Read,
    writer: &mut impl Write,
    input: RemoteTransferSendInput<'_>,
) -> Result<crate::remote_transfer::RemoteTransferShellReport, String> {
    send_prepared_remote_transfer_with_fence_validator(reader, writer, input, &mut || Ok(()))
}

fn send_prepared_remote_transfer_with_fence_validator(
    reader: &mut impl Read,
    writer: &mut impl Write,
    input: RemoteTransferSendInput<'_>,
    validate_fence: &mut impl FnMut() -> Result<(), String>,
) -> Result<crate::remote_transfer::RemoteTransferShellReport, String> {
    send_prepared_remote_transfer_with_observer(reader, writer, input, validate_fence, &mut |_| {})
}

fn send_prepared_remote_transfer_with_observer(
    reader: &mut impl Read,
    writer: &mut impl Write,
    input: RemoteTransferSendInput<'_>,
    validate_fence: &mut impl FnMut() -> Result<(), String>,
    observe: &mut impl FnMut(RemoteProductionTelemetryFact),
) -> Result<crate::remote_transfer::RemoteTransferShellReport, String> {
    let RemoteTransferSendInput {
        prepared,
        policy,
        direction,
        demand_frame,
        interrupt_after_chunks,
    } = input;
    if demand_frame.direction != direction
        || demand_frame.demand.scope != crunch_build::distributed::remote_transfer_scope(&prepared.manifest)
    {
        return Err("remote-transfer-demand-scope-mismatch".to_string());
    }
    let initial_transferred_bytes = demand_frame.sender_state.transferred_bytes;
    let demanded_chunks = u32::try_from(demand_frame.demand.missing_chunks.len())
        .map_err(|_| "remote-transfer-demand-count-overflow".to_string())?;
    if demanded_chunks > 0 {
        observe(RemoteProductionTelemetryFact::TransferDemand {
            chunks: demanded_chunks,
        });
    }
    if initial_transferred_bytes > 0 {
        observe(RemoteProductionTelemetryFact::TransferResumed {
            bytes: initial_transferred_bytes,
        });
    }
    let sender_state = send_demanded_remote_chunks(
        reader,
        writer,
        RemoteTransferChunkSendInput {
            prepared,
            policy,
            direction,
            demand: &demand_frame.demand,
            sender_state: demand_frame.sender_state,
            interrupt_after_chunks,
        },
        validate_fence,
        observe,
    )?;
    validate_fence()?;
    let complete = read_expected_remote_frame(reader, RemoteFrameKind::TransferComplete)?;
    let RemoteFrame::TransferComplete {
        direction: complete_direction,
        report,
    } = complete
    else {
        return Err("remote-transfer-complete-frame-invalid".to_string());
    };
    if complete_direction != direction || report.manifest_digest_blake3 != prepared.manifest.digest_blake3.as_str() {
        return Err("remote-transfer-completion-manifest-mismatch".to_string());
    }
    validate_fence()?;
    let completed_bytes = report.transferred_bytes.saturating_add(report.reused_bytes);
    if completed_bytes > 0 {
        observe(RemoteProductionTelemetryFact::TransferCompleted {
            bytes: completed_bytes,
            mode: RemoteTransferMode::Streaming,
        });
    }
    assert!(sender_state.in_flight.is_empty());
    assert!(report.transferred_bytes >= initial_transferred_bytes);
    Ok(report)
}

struct RemoteTransferChunkSendInput<'a> {
    prepared: &'a crate::remote_transfer::PreparedRemoteTransfer,
    policy: RemoteTransferPolicy,
    direction: crate::remote_transfer::RemoteTransferDirection,
    demand: &'a RemoteTransferDemand,
    sender_state: RemoteTransferCreditState,
    interrupt_after_chunks: Option<u32>,
}

fn send_demanded_remote_chunks(
    reader: &mut impl Read,
    writer: &mut impl Write,
    input: RemoteTransferChunkSendInput<'_>,
    validate_fence: &mut impl FnMut() -> Result<(), String>,
    observe: &mut impl FnMut(RemoteProductionTelemetryFact),
) -> Result<RemoteTransferCreditState, String> {
    let mut sender_state = input.sender_state;
    let mut sent_chunks = 0_u32;
    for missing in &input.demand.missing_chunks {
        let frame = read_expected_remote_frame(reader, RemoteFrameKind::TransferCredit)?;
        let RemoteFrame::TransferCredit { transfer } = frame else {
            return Err("remote-transfer-credit-frame-invalid".to_string());
        };
        if transfer.direction != input.direction || transfer.missing != *missing {
            observe(RemoteProductionTelemetryFact::TransferCutoff { accepted: false });
            return Err("remote-transfer-credit-demand-mismatch".to_string());
        }
        validate_fence()?;
        let credit_bytes = transfer.grant.bytes;
        let reserved = match crate::remote_transfer::write_remote_transfer_data_chunk(
            &mut *writer,
            input.prepared,
            input.policy,
            input.demand,
            &sender_state,
            transfer.grant,
            missing,
        ) {
            Ok(reserved) => reserved,
            Err(reason) => {
                observe(RemoteProductionTelemetryFact::TransferCutoff { accepted: false });
                return Err(reason);
            }
        };
        if credit_bytes > 0 {
            observe(RemoteProductionTelemetryFact::TransferCredit { bytes: credit_bytes });
        }
        let acknowledgement_frame = read_expected_remote_frame(reader, RemoteFrameKind::TransferAcknowledgement)?;
        let RemoteFrame::TransferAcknowledgement {
            direction: acknowledgement_direction,
            acknowledgement,
        } = acknowledgement_frame
        else {
            return Err("remote-transfer-acknowledgement-frame-invalid".to_string());
        };
        if acknowledgement_direction != input.direction {
            return Err("remote-transfer-acknowledgement-direction-mismatch".to_string());
        }
        let progress_step = reserved
            .last_progress_step
            .checked_add(1)
            .ok_or_else(|| RemoteTransferReasonCode::ArithmeticOverflow.as_str().to_string())?;
        sender_state = acknowledge_remote_transfer_chunk(
            input.policy,
            &reserved,
            &acknowledgement,
            &missing.chunk.digest_blake3,
            progress_step,
        )
        .map_err(|reason| reason.as_str().to_string())?;
        sent_chunks =
            sent_chunks.checked_add(1).ok_or_else(|| "remote-transfer-sent-chunk-count-overflow".to_string())?;
        if input.interrupt_after_chunks.is_some_and(|limit| sent_chunks >= limit) {
            return Err("remote-production-input-transfer-interrupted-after-checkpoint".to_string());
        }
    }
    debug_assert!(sender_state.in_flight.is_empty());
    if let Ok(demanded_chunk_count) = u32::try_from(input.demand.missing_chunks.len()) {
        debug_assert!(sent_chunks <= demanded_chunk_count);
    }
    Ok(sender_state)
}

struct RemoteTransferReceiveInput<'a> {
    direction: crate::remote_transfer::RemoteTransferDirection,
    state_dir: Option<&'a Path>,
    attempt: Option<&'a RemoteProductionAttemptBinding>,
    interrupt_after_chunks: Option<u32>,
}

fn receive_remote_transfer_interactively(
    reader: &mut impl Read,
    writer: &mut impl Write,
    session: &mut crate::remote_transfer::RemoteTransferReceiveSession,
    input: RemoteTransferReceiveInput<'_>,
) -> Result<Option<crate::remote_transfer::RemoteTransferShellReport>, String> {
    let RemoteTransferReceiveInput {
        direction,
        state_dir,
        attempt,
        interrupt_after_chunks,
    } = input;
    let mut validate_fence = || {
        if let (Some(state_dir), Some(attempt)) = (state_dir, attempt) {
            validate_current_production_attempt(state_dir, attempt)?;
        }
        Ok(())
    };
    let transfer_result = receive_remote_transfer_interactively_with_fence_validator(
        reader,
        writer,
        session,
        RemoteTransferReceiveInput {
            direction,
            state_dir,
            attempt,
            interrupt_after_chunks,
        },
        &mut validate_fence,
    );
    let sender_state = session.sender_credit_state();
    debug_assert!(sender_state.in_flight.is_empty());
    debug_assert_eq!(sender_state.granted_chunks_remaining, 0);
    transfer_result
}

fn receive_remote_transfer_interactively_with_fence_validator(
    reader: &mut impl Read,
    writer: &mut impl Write,
    session: &mut crate::remote_transfer::RemoteTransferReceiveSession,
    input: RemoteTransferReceiveInput<'_>,
    validate_fence: &mut impl FnMut() -> Result<(), String>,
) -> Result<Option<crate::remote_transfer::RemoteTransferShellReport>, String> {
    let demand = RemoteTransferDemandFrame {
        direction: input.direction,
        demand: session.demand().clone(),
        sender_state: session.sender_credit_state(),
    };
    write_remote_control_frame(writer, &RemoteFrame::TransferDemand {
        transfer: demand.clone(),
    })?;
    let missing_chunks = session.demand().missing_chunks.clone();
    let mut received_chunks = 0_u32;
    for missing in &missing_chunks {
        validate_fence()?;
        let grant = session.credit_for_chunk(missing)?;
        write_remote_control_frame(writer, &RemoteFrame::TransferCredit {
            transfer: RemoteTransferCreditFrame {
                direction: input.direction,
                missing: missing.clone(),
                grant,
            },
        })?;
        let acknowledgement = session.receive_chunk(&mut *reader)?;
        if let Err(reason) = validate_fence() {
            session.invalidate_fenced_progress().map_err(|invalidation| {
                format!("{reason}; remote-transfer-fence-invalidation-failed: {invalidation}")
            })?;
            return Err(reason);
        }
        received_chunks = received_chunks
            .checked_add(1)
            .ok_or_else(|| "remote-transfer-received-chunk-count-overflow".to_string())?;
        if input.interrupt_after_chunks.is_some_and(|limit| received_chunks >= limit) {
            return Err("remote-production-transfer-interrupted-after-checkpoint".to_string());
        }
        write_remote_control_frame(writer, &RemoteFrame::TransferAcknowledgement {
            direction: input.direction,
            acknowledgement,
        })?;
    }
    if let Ok(missing_chunk_count) = u32::try_from(missing_chunks.len()) {
        debug_assert!(received_chunks <= missing_chunk_count);
    }
    debug_assert_eq!(session.demand().scope, demand.demand.scope);
    Ok(None)
}

fn record_production_fact(
    telemetry: &mut RemoteTelemetryBuffer,
    policy: RemoteTelemetryPolicy,
    fact: RemoteProductionTelemetryFact,
) {
    *telemetry = record_remote_production_telemetry(telemetry, fact, policy);
    if let Ok(event_capacity) = usize::try_from(policy.event_capacity) {
        debug_assert!(telemetry.events.len() <= event_capacity);
    }
    if let Ok(event_count) = u64::try_from(telemetry.events.len()) {
        debug_assert!(telemetry.accepted_events >= event_count);
    }
}

fn record_transfer_demand_and_resume(
    telemetry: &mut RemoteTelemetryBuffer,
    policy: RemoteTelemetryPolicy,
    demand: &RemoteTransferDemandFrame,
) {
    let Ok(chunks) = u32::try_from(demand.demand.missing_chunks.len()) else {
        debug_assert!(u32::try_from(demand.demand.missing_chunks.len()).is_err());
        return;
    };
    if chunks > 0 {
        record_production_fact(telemetry, policy, RemoteProductionTelemetryFact::TransferDemand { chunks });
        let Some(bytes) = demand
            .demand
            .missing_chunks
            .iter()
            .try_fold(0_u64, |total, chunk| total.checked_add(u64::from(chunk.chunk.size_bytes)))
        else {
            debug_assert!(!demand.demand.missing_chunks.is_empty());
            return;
        };
        if bytes > 0 {
            record_production_fact(telemetry, policy, RemoteProductionTelemetryFact::TransferCredit { bytes });
        }
    }
    if demand.sender_state.transferred_bytes > 0 {
        record_production_fact(telemetry, policy, RemoteProductionTelemetryFact::TransferResumed {
            bytes: demand.sender_state.transferred_bytes,
        });
    }
    if let Ok(chunk_count) = usize::try_from(chunks) {
        debug_assert_eq!(chunk_count, demand.demand.missing_chunks.len());
    }
}

fn record_transfer_demand_and_resume_from_session(
    telemetry: &mut RemoteTelemetryBuffer,
    policy: RemoteTelemetryPolicy,
    session: &crate::remote_transfer::RemoteTransferReceiveSession,
    direction: crate::remote_transfer::RemoteTransferDirection,
) {
    let demand = RemoteTransferDemandFrame {
        direction,
        demand: session.demand().clone(),
        sender_state: session.sender_credit_state(),
    };
    record_transfer_demand_and_resume(telemetry, policy, &demand);
}

fn record_transfer_completion(
    telemetry: &mut RemoteTelemetryBuffer,
    policy: RemoteTelemetryPolicy,
    report: &crate::remote_transfer::RemoteTransferShellReport,
    mode: RemoteTransferMode,
    fallback_reason: Option<&str>,
) {
    let bytes = report.transferred_bytes.saturating_add(report.reused_bytes);
    if report.disposition == crate::remote_transfer::RemoteTransferShellDisposition::AlreadyPresent {
        record_production_fact(telemetry, policy, RemoteProductionTelemetryFact::TransferCutoff { accepted: true });
    }
    if bytes > 0 {
        record_production_fact(telemetry, policy, RemoteProductionTelemetryFact::TransferCompleted { bytes, mode });
        if fallback_reason.is_some() {
            record_production_fact(telemetry, policy, RemoteProductionTelemetryFact::TransferFallback { bytes });
        }
    }
    debug_assert!(fallback_reason.is_none() || mode != RemoteTransferMode::Streaming);
    debug_assert!(report.transferred_bytes <= bytes);
}

struct RemoteProductionClientIo<'a> {
    stdin: &'a mut std::process::ChildStdin,
    stdout: &'a mut std::process::ChildStdout,
}

struct RemoteProductionClientContext<'a> {
    production: &'a RemoteProductionTransferClient,
    request: &'a ConcreteBuildRequest,
    attempt: &'a RemoteProductionAttemptBinding,
    auth: &'a TicketAuthRequest,
    policy: RemoteTransferPolicy,
}

struct RemoteProductionClientProgress {
    frames: Vec<RemoteFrame>,
    telemetry: RemoteTelemetryBuffer,
    trace_context_health: Option<RemoteTraceContextHealth>,
}

fn run_remote_production_client_protocol(
    mut stdin: std::process::ChildStdin,
    mut stdout: std::process::ChildStdout,
    command: RemoteStdioCommand,
    production: RemoteProductionTransferClient,
) -> Result<RemoteStdioTranscript, String> {
    let (hello, trace_context, auth, request, input_manifest) = command_client_frames(&command)?;
    let attempt = request.production_attempt.as_ref().ok_or_else(|| "remote-production-attempt-missing".to_string())?;
    let policy = request.transfer_policy.ok_or_else(|| "remote-production-transfer-policy-missing".to_string())?;
    if policy != production.transfer_policy {
        return Err("remote-production-transfer-policy-mismatch".to_string());
    }
    let context = RemoteProductionClientContext {
        production: &production,
        request: &request,
        attempt,
        auth: &auth,
        policy,
    };
    let (progress, streaming_output) = {
        let mut io = RemoteProductionClientIo {
            stdin: &mut stdin,
            stdout: &mut stdout,
        };
        let mut progress = open_remote_production_client(&mut io, &context, hello, trace_context, input_manifest)?;
        let streaming_output = receive_remote_production_output(&mut io, &context, &mut progress)?;
        (progress, streaming_output)
    };
    drop(stdin);
    let transcript = RemoteStdioTranscript {
        binding: command.binding,
        frames: progress.frames,
        stderr_summary: String::new(),
        telemetry: progress.telemetry,
        streaming_output: Some(streaming_output),
        trace_context_health: progress.trace_context_health,
    };
    debug_assert_eq!(transcript.binding, command.binding);
    debug_assert!(transcript.streaming_output.is_some());
    Ok(transcript)
}

fn open_remote_production_client(
    io: &mut RemoteProductionClientIo<'_>,
    context: &RemoteProductionClientContext<'_>,
    hello: RemoteHello,
    trace_context: Option<RemoteTraceContext>,
    input_manifest: RemoteInputManifest,
) -> Result<RemoteProductionClientProgress, String> {
    validate_current_production_attempt(&context.production.state_dir, context.attempt)?;
    let is_trace_negotiated = hello.capabilities.iter().any(|capability| capability == REMOTE_TRACE_CONTEXT_CAPABILITY);
    write_remote_control_frame(&mut io.stdin, &RemoteFrame::Hello { hello })?;
    if is_trace_negotiated {
        write_remote_control_frame(&mut io.stdin, &RemoteFrame::TraceContext { context: trace_context })?;
    }
    write_remote_control_frame(&mut io.stdin, &RemoteFrame::AuthTicket {
        auth: context.auth.clone(),
    })?;
    write_remote_control_frame(&mut io.stdin, &RemoteFrame::BuildRequest {
        request: context.request.clone(),
    })?;
    write_remote_control_frame(&mut io.stdin, &RemoteFrame::InputManifest {
        manifest: input_manifest,
    })?;
    let auth_ok = read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::AuthOk)?;
    let mut progress = RemoteProductionClientProgress {
        frames: Vec::with_capacity(MAX_REMOTE_STDIO_FRAME_COUNT),
        telemetry: empty_remote_telemetry_buffer(),
        trace_context_health: None,
    };
    progress.frames.push(auth_ok);
    if is_trace_negotiated {
        let acknowledgement_frame = read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::TraceContextAck)?;
        let health = match &acknowledgement_frame {
            RemoteFrame::TraceContextAck { acknowledgement } => acknowledgement.health.clone(),
            _ => return Err("remote-trace-context-ack-frame-invalid".to_string()),
        };
        progress.frames.push(acknowledgement_frame);
        progress.trace_context_health = Some(health);
    }
    progress.frames.push(read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::MissingInputs)?);
    send_remote_production_inputs(io, context, &mut progress)?;
    debug_assert!(progress.frames.len() <= MAX_REMOTE_STDIO_FRAME_COUNT);
    debug_assert_eq!(progress.trace_context_health.is_some(), is_trace_negotiated);
    Ok(progress)
}

fn send_remote_production_inputs(
    io: &mut RemoteProductionClientIo<'_>,
    context: &RemoteProductionClientContext<'_>,
    progress: &mut RemoteProductionClientProgress,
) -> Result<(), String> {
    let Some(input_transfer) = context.production.input_transfer.as_ref() else {
        if context.request.input_refs.is_empty() {
            return Ok(());
        }
        return Err("remote-production-input-transfer-missing".to_string());
    };
    let transfer_frame = RemoteTransferManifestFrame {
        direction: crate::remote_transfer::RemoteTransferDirection::Upload,
        manifest: input_transfer.manifest.manifest.clone(),
        manifest_digest_blake3: input_transfer.manifest.digest_blake3.clone(),
        actual_mode: RemoteTransferMode::Streaming,
        fallback_reason: None,
    };
    write_remote_control_frame(&mut io.stdin, &RemoteFrame::TransferManifest {
        transfer: transfer_frame,
    })?;
    let demand_frame = read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::TransferDemand)?;
    let demand = match demand_frame {
        RemoteFrame::TransferDemand { transfer } => transfer,
        _ => return Err("remote-transfer-demand-frame-invalid".to_string()),
    };
    let mut validate_upload_fence =
        || validate_current_production_attempt(&context.production.state_dir, context.attempt);
    let telemetry_policy = context.production.telemetry_policy;
    let mut observe = |fact| record_production_fact(&mut progress.telemetry, telemetry_policy, fact);
    let complete = send_prepared_remote_transfer_with_observer(
        &mut io.stdout,
        &mut io.stdin,
        RemoteTransferSendInput {
            prepared: input_transfer,
            policy: context.policy,
            direction: crate::remote_transfer::RemoteTransferDirection::Upload,
            demand_frame: demand,
            interrupt_after_chunks: context.production.interrupt_after_input_chunks,
        },
        &mut validate_upload_fence,
        &mut observe,
    )?;
    progress.frames.push(RemoteFrame::TransferComplete {
        direction: crate::remote_transfer::RemoteTransferDirection::Upload,
        report: complete,
    });
    debug_assert!(!progress.frames.is_empty());
    debug_assert!(progress.frames.len() <= MAX_REMOTE_STDIO_FRAME_COUNT);
    Ok(())
}

fn receive_remote_production_output(
    io: &mut RemoteProductionClientIo<'_>,
    context: &RemoteProductionClientContext<'_>,
    progress: &mut RemoteProductionClientProgress,
) -> Result<RemoteStreamingOutputReceipt, String> {
    progress.frames.push(read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::BuildQueued)?);
    progress.frames.push(read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::BuildStarted)?);
    let telemetry_policy = context.production.telemetry_policy;
    record_production_fact(&mut progress.telemetry, telemetry_policy, RemoteProductionTelemetryFact::ExecutionStarted);
    let finished_frame = read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::BuildFinished)?;
    let result = match &finished_frame {
        RemoteFrame::BuildFinished { result } => result.clone(),
        _ => return Err("remote-build-finished-frame-invalid".to_string()),
    };
    record_production_fact(
        &mut progress.telemetry,
        telemetry_policy,
        RemoteProductionTelemetryFact::ExecutionCompleted,
    );
    prevalidate_remote_streaming_output_metadata(context.request, &context.production.trusted_output_keys, &result)?;
    progress.frames.push(finished_frame);
    let manifest_frame = read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::TransferManifest)?;
    let output_transfer = match &manifest_frame {
        RemoteFrame::TransferManifest { transfer } => transfer.clone(),
        _ => return Err("remote-transfer-manifest-frame-invalid".to_string()),
    };
    validate_output_transfer_manifest_frame(context.request, &result, &output_transfer, context.policy)?;
    progress.frames.push(manifest_frame);
    let receipt = receive_remote_production_output_transfer(io, context, progress, &output_transfer)?;
    progress
        .frames
        .push(read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::OutputTransferDone)?);
    progress.frames.push(read_expected_remote_frame(&mut io.stdout, RemoteFrameKind::Done)?);
    debug_assert!(progress.frames.len() <= MAX_REMOTE_STDIO_FRAME_COUNT);
    debug_assert!(is_blake3_hex_digest(receipt.manifest_digest_blake3.as_str()));
    Ok(receipt)
}

fn receive_remote_production_output_transfer(
    io: &mut RemoteProductionClientIo<'_>,
    context: &RemoteProductionClientContext<'_>,
    progress: &mut RemoteProductionClientProgress,
    output_transfer: &RemoteTransferManifestFrame,
) -> Result<RemoteStreamingOutputReceipt, String> {
    validate_current_production_attempt(&context.production.state_dir, context.attempt)?;
    let canonical = canonicalize_remote_transfer_manifest(output_transfer.manifest.clone(), context.policy)
        .map_err(|reason| reason.as_str().to_string())?;
    let receiver_root = crate::remote_transfer::remote_transfer_receiver_root(
        &context.production.state_dir,
        &canonical.manifest.session_id,
    );
    let lease_expires_unix_s = context
        .auth
        .now_unix_s
        .checked_add(REMOTE_TRANSFER_LEASE_DURATION_SECS)
        .ok_or_else(|| "remote-transfer-lease-expiry-overflow".to_string())?;
    let mut session = crate::remote_transfer::begin_remote_transfer_receive(
        output_transfer.manifest.clone(),
        &output_transfer.manifest_digest_blake3,
        context.policy,
        &context.production.state_dir,
        &receiver_root,
        crate::remote_transfer::RemoteTransferRunOptions {
            direction: crate::remote_transfer::RemoteTransferDirection::Download,
            interrupt_after_chunks: None,
            now_unix_s: context.auth.now_unix_s,
            lease_expires_unix_s,
            admission: crate::remote_transfer::RemoteTransferAdmissionFacts {
                required_closure_metadata_verified: true,
                path_info_admitted: true,
            },
        },
    )?;
    let telemetry_policy = context.production.telemetry_policy;
    record_transfer_demand_and_resume_from_session(
        &mut progress.telemetry,
        telemetry_policy,
        &session,
        crate::remote_transfer::RemoteTransferDirection::Download,
    );
    receive_remote_transfer_interactively(&mut io.stdout, &mut io.stdin, &mut session, RemoteTransferReceiveInput {
        direction: crate::remote_transfer::RemoteTransferDirection::Download,
        state_dir: Some(&context.production.state_dir),
        attempt: Some(context.attempt),
        interrupt_after_chunks: context.production.interrupt_after_output_chunks,
    })?;
    finish_remote_production_output_transfer(io, context, progress, output_transfer, session)?;
    let receipt = RemoteStreamingOutputReceipt {
        manifest: output_transfer.manifest.clone(),
        manifest_digest_blake3: output_transfer.manifest_digest_blake3.clone(),
        receiver_root,
    };
    debug_assert!(receipt.receiver_root.is_absolute());
    debug_assert!(is_blake3_hex_digest(receipt.manifest_digest_blake3.as_str()));
    Ok(receipt)
}

fn finish_remote_production_output_transfer(
    io: &mut RemoteProductionClientIo<'_>,
    context: &RemoteProductionClientContext<'_>,
    progress: &mut RemoteProductionClientProgress,
    output_transfer: &RemoteTransferManifestFrame,
    mut session: crate::remote_transfer::RemoteTransferReceiveSession,
) -> Result<(), String> {
    validate_current_production_attempt(&context.production.state_dir, context.attempt)?;
    let transfer_receipt = session.finish()?;
    record_transfer_completion(
        &mut progress.telemetry,
        context.production.telemetry_policy,
        &transfer_receipt,
        output_transfer.actual_mode,
        output_transfer.fallback_reason.as_deref(),
    );
    if let Err(reason) = validate_current_production_attempt(&context.production.state_dir, context.attempt) {
        session
            .invalidate_fenced_progress()
            .map_err(|invalidation| format!("{reason}; remote-transfer-fence-invalidation-failed: {invalidation}"))?;
        return Err(reason);
    }
    validate_current_production_attempt(&context.production.state_dir, context.attempt)?;
    write_remote_control_frame(&mut io.stdin, &RemoteFrame::TransferComplete {
        direction: crate::remote_transfer::RemoteTransferDirection::Download,
        report: transfer_receipt,
    })?;
    debug_assert_eq!(output_transfer.direction, crate::remote_transfer::RemoteTransferDirection::Download);
    debug_assert!(progress.frames.len() <= MAX_REMOTE_STDIO_FRAME_COUNT);
    Ok(())
}

fn drain_bounded_pipe(mut reader: impl Read, retained_bytes_max: usize) -> Result<(Vec<u8>, bool), String> {
    const PIPE_DRAIN_BUFFER_BYTES: usize = 16_384;
    let mut retained = Vec::with_capacity(retained_bytes_max);
    let mut is_exceeded = false;
    let mut is_eof = false;
    let mut buffer = [0_u8; PIPE_DRAIN_BUFFER_BYTES];
    for _ in 0..MAX_REMOTE_STDIO_FRAME_COUNT {
        let read_bytes = reader.read(&mut buffer).map_err(|err| format!("draining child pipe: {err}"))?;
        if read_bytes == 0 {
            is_eof = true;
            break;
        }
        let remaining = retained_bytes_max.saturating_sub(retained.len());
        let retained_now = read_bytes.min(remaining);
        retained.extend_from_slice(&buffer[..retained_now]);
        is_exceeded |= retained_now < read_bytes;
    }
    if !is_eof {
        return Err("draining child pipe exceeded read bound".to_string());
    }
    assert!(retained.len() <= retained_bytes_max);
    if is_exceeded {
        assert_eq!(retained.len(), retained_bytes_max);
    }
    Ok((retained, is_exceeded))
}

#[cfg(unix)]
fn configure_remote_child_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    // SAFETY: setpgid is async-signal-safe and touches only the child process.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == 0 {
                return Ok(());
            }
            Err(std::io::Error::last_os_error())
        });
    }
}

#[cfg(not(unix))]
fn configure_remote_child_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_remote_child_tree(child: &mut std::process::Child) -> Result<(), RunError> {
    let process_group = i32::try_from(child.id())
        .map_err(|_| RunError::Internal("stdio remote child pid does not fit process-group id".to_string()))?;
    let result = unsafe { libc::kill(-process_group, libc::SIGKILL) };
    if result != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(RunError::Internal(format!("killing stdio remote child process group: {error}")));
        }
    }
    child.wait().map_err(|error| RunError::Internal(format!("reaping stdio remote child: {error}")))?;
    assert!(child.try_wait().is_ok_and(|status| status.is_some()));
    assert!(process_group > 0);
    Ok(())
}

#[cfg(not(unix))]
fn terminate_remote_child_tree(child: &mut std::process::Child) -> Result<(), RunError> {
    child.kill().map_err(|error| RunError::Internal(format!("killing stdio remote child: {error}")))?;
    child.wait().map_err(|error| RunError::Internal(format!("reaping stdio remote child: {error}")))?;
    Ok(())
}

fn wait_for_remote_child_teardown(child: &mut std::process::Child) -> Result<(), RunError> {
    let timeout_ms = REMOTE_CHILD_TEARDOWN_TIMEOUT_SECS
        .checked_mul(MILLISECONDS_PER_SECOND)
        .ok_or_else(|| RunError::Internal("stdio remote child teardown timeout overflow".to_string()))?;
    let poll_attempts = timeout_ms
        .checked_div(REMOTE_CHILD_POLL_INTERVAL_MS)
        .and_then(|attempts| attempts.checked_add(1))
        .ok_or_else(|| RunError::Internal("stdio remote child teardown poll bound invalid".to_string()))?;
    for _ in 0..poll_attempts {
        if child
            .try_wait()
            .map_err(|error| RunError::Internal(format!("polling stdio remote child teardown: {error}")))?
            .is_some()
        {
            assert!(poll_attempts > 0);
            assert!(child.id() > 0);
            return Ok(());
        }
        thread::sleep(Duration::from_millis(REMOTE_CHILD_POLL_INTERVAL_MS));
    }
    terminate_remote_child_tree(child)
}

fn run_production_stdio_remote_child(
    mut child: std::process::Child,
    command: &RemoteStdioCommand,
    production: &RemoteProductionTransferClient,
) -> Result<RemoteStdioTranscript, RunError> {
    debug_assert!(command.timeout_secs > 0);
    debug_assert!(command.timeout_secs <= MAX_REMOTE_BUILD_TIME_SECS);
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| RunError::Internal("stdio remote child stdin missing".to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RunError::Internal("stdio remote child stdout missing".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RunError::Internal("stdio remote child stderr missing".to_string()))?;
    let (protocol_tx, protocol_rx) = std::sync::mpsc::sync_channel(1);
    let command_owned = command.clone();
    let production_owned = production.clone();
    thread::spawn(move || {
        let result = run_remote_production_client_protocol(stdin, stdout, command_owned, production_owned);
        let _delivery_result = protocol_tx.send(result);
    });
    let (stderr_tx, stderr_rx) = std::sync::mpsc::sync_channel(1);
    thread::spawn(move || {
        let _delivery_result = stderr_tx.send(drain_bounded_pipe(stderr, MAX_REMOTE_STDIO_STDERR_BYTES));
    });
    let protocol_timeout_secs = Duration::from_secs(command.timeout_secs);
    let protocol_result = match protocol_rx.recv_timeout(protocol_timeout_secs) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err("remote-production-protocol-thread-disconnected".to_string())
        }
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            terminate_remote_child_tree(&mut child)?;
            return Err(RunError::Internal(format!(
                "stdio remote child timed out after {} seconds phase={:?}",
                command.timeout_secs,
                RemoteFailurePhase::TransportSetup
            )));
        }
    };
    if protocol_result.is_err() {
        terminate_remote_child_tree(&mut child)?;
    } else {
        wait_for_remote_child_teardown(&mut child)?;
    }
    let teardown_timeout_secs = Duration::from_secs(REMOTE_CHILD_TEARDOWN_TIMEOUT_SECS);
    let (stderr, stderr_exceeded) = stderr_rx
        .recv_timeout(teardown_timeout_secs)
        .map_err(|_| RunError::Internal("stdio remote child stderr teardown timed out".to_string()))?
        .map_err(RunError::Internal)?;
    finalize_remote_child_transcript(protocol_result, stderr, stderr_exceeded, command.binding)
}

fn finalize_remote_child_transcript(
    protocol_result: Result<RemoteStdioTranscript, String>,
    stderr: Vec<u8>,
    is_stderr_exceeded: bool,
    binding: RemoteTransportBinding,
) -> Result<RemoteStdioTranscript, RunError> {
    let mut transcript = protocol_result.map_err(|reason| {
        RunError::Internal(format!(
            "stdio remote child failed phase={:?}: {reason}; stderr={}",
            RemoteFailurePhase::TransportSetup,
            bounded_stderr_summary(&stderr)
        ))
    })?;
    if is_stderr_exceeded {
        transcript.stderr_summary = format!("{}{}", String::from_utf8_lossy(&stderr), STDERR_TRUNCATION_MARKER);
    } else {
        transcript.stderr_summary = String::from_utf8_lossy(&stderr).to_string();
    }
    let stderr_summary_limit_bytes = MAX_REMOTE_STDIO_STDERR_BYTES
        .checked_add(STDERR_TRUNCATION_MARKER.len())
        .ok_or_else(|| RunError::Internal("stdio stderr summary limit overflow".to_string()))?;
    debug_assert_eq!(transcript.binding, binding);
    debug_assert!(transcript.stderr_summary.len() <= stderr_summary_limit_bytes);
    Ok(transcript)
}

pub fn run_stdio_remote_child(command: &RemoteStdioCommand) -> Result<RemoteStdioTranscript, RunError> {
    if command.timeout_secs == 0 || command.timeout_secs > MAX_REMOTE_BUILD_TIME_SECS {
        return Err(RunError::Internal(format!(
            "stdio remote child timeout out of bounds: {} seconds",
            command.timeout_secs
        )));
    }
    let mut process = Command::new(&command.program);
    process.args(&command.args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    configure_remote_child_process_group(&mut process);
    let mut child = process.spawn().map_err(|err| {
        RunError::Internal(format!("spawning stdio remote child {}: {err}", command.program.display()))
    })?;
    if let Some(production) = command.production_transfer.as_ref() {
        return run_production_stdio_remote_child(child, command, production);
    }
    if let Some(mut stdin) = child.stdin.take() {
        for frame in &command.input_frames {
            write_remote_frame(&mut stdin, frame).map_err(RunError::Internal)?;
        }
    }
    let output = wait_for_stdio_child_output(child, command.timeout_secs, &command.program)?;
    let child_output = RemoteStdioChildOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        status_success: output.status.success(),
    };
    let transcript = validate_stdio_child_output_for_binding(&child_output, command.binding).map_err(|failure| {
        RunError::Internal(format!(
            "stdio remote child failed phase={:?} retry={:?}: {}; stderr={}",
            failure.phase,
            failure.retry_class,
            failure.reason,
            bounded_stderr_summary(&child_output.stderr)
        ))
    })?;
    debug_assert_eq!(transcript.binding, command.binding);
    debug_assert!(transcript.frames.len() <= MAX_REMOTE_STDIO_FRAME_COUNT);
    Ok(transcript)
}

fn wait_for_stdio_child_output(
    mut child: std::process::Child,
    timeout_secs: u64,
    program: &Path,
) -> Result<std::process::Output, RunError> {
    let timeout_ms = timeout_secs
        .checked_mul(MILLISECONDS_PER_SECOND)
        .ok_or_else(|| RunError::Internal("stdio remote child timeout overflow".to_string()))?;
    let poll_attempts = timeout_ms
        .checked_div(REMOTE_CHILD_POLL_INTERVAL_MS)
        .and_then(|attempts| attempts.checked_add(1))
        .ok_or_else(|| RunError::Internal("stdio remote child poll bound invalid".to_string()))?;
    for _ in 0..poll_attempts {
        if child
            .try_wait()
            .map_err(|err| RunError::Internal(format!("polling stdio remote child {}: {err}", program.display())))?
            .is_some()
        {
            debug_assert!(poll_attempts > 0);
            debug_assert!(child.id() > 0);
            return child.wait_with_output().map_err(|err| {
                RunError::Internal(format!("collecting stdio remote child {} output: {err}", program.display()))
            });
        }
        thread::sleep(Duration::from_millis(REMOTE_CHILD_POLL_INTERVAL_MS));
    }
    terminate_remote_child_tree(&mut child)?;
    Err(RunError::Internal(format!(
        "stdio remote child timed out after {timeout_secs} seconds phase={:?}",
        RemoteFailurePhase::TransportSetup
    )))
}

pub fn remote_client_request_frames(client: &RemoteLoopbackClient) -> Vec<RemoteFrame> {
    vec![
        RemoteFrame::Hello {
            hello: client.hello.clone(),
        },
        RemoteFrame::AuthTicket {
            auth: client.auth.clone(),
        },
        RemoteFrame::BuildRequest {
            request: client.request.clone(),
        },
        RemoteFrame::InputManifest {
            manifest: client.input_manifest.clone(),
        },
        RemoteFrame::InputUpload {
            upload: RemoteInputUpload {
                request_id: client.request.request_id.clone(),
                refs: client.uploaded_input_refs.clone(),
                byte_count: client.request.upload_bytes,
                artifacts: Vec::new(),
                streamed: false,
            },
        },
    ]
}

#[cfg(unix)]
pub fn read_remote_ticket_credential_from_owned_fd(fd: i32) -> Result<RemoteTicketCredential, String> {
    read_remote_ticket_credential_from_owned_fd_with_timeout(
        fd,
        std::time::Duration::from_millis(REMOTE_TICKET_CREDENTIAL_READ_TIMEOUT_MS),
    )
}

#[cfg(unix)]
fn read_remote_ticket_credential_from_owned_fd_with_timeout(
    fd: i32,
    timeout: std::time::Duration,
) -> Result<RemoteTicketCredential, String> {
    use std::io::IsTerminal as _;
    use std::os::fd::FromRawFd as _;

    if fd <= libc::STDERR_FILENO {
        return Err("remote-ticket-input-fd-reserved".to_string());
    }
    // SAFETY: The caller explicitly transfers this descriptor to the command.
    let file = unsafe { File::from_raw_fd(fd) };
    if file.is_terminal() {
        return Err("remote-ticket-input-terminal-forbidden".to_string());
    }
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err("remote-ticket-input-nonblocking-setup-failed".to_string());
    }
    let deadline = std::time::Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| "remote-ticket-input-deadline-overflow".to_string())?;
    let credential_capacity = usize::try_from(REMOTE_TICKET_CREDENTIAL_BYTES_MAX)
        .map_err(|_| "remote-ticket-input-size-invalid".to_string())?;
    let mut bytes = zeroize::Zeroizing::new(Vec::with_capacity(credential_capacity));
    let allocation_capacity = bytes.capacity();
    assert!(allocation_capacity >= credential_capacity);
    let mut chunk = zeroize::Zeroizing::new([0_u8; REMOTE_TICKET_CREDENTIAL_READ_CHUNK_BYTES]);
    loop {
        let now = std::time::Instant::now();
        if now >= deadline {
            return Err("remote-ticket-input-read-timeout".to_string());
        }
        let remaining_ms = deadline.saturating_duration_since(now).as_millis().max(1);
        let poll_timeout_ms = i32::try_from(remaining_ms.min(i32::MAX as u128)).unwrap_or(i32::MAX);
        let mut poll_fd = libc::pollfd {
            fd,
            events: libc::POLLIN | libc::POLLHUP,
            revents: 0,
        };
        let poll_result = unsafe { libc::poll(&mut poll_fd, 1, poll_timeout_ms) };
        if poll_result == 0 {
            return Err("remote-ticket-input-read-timeout".to_string());
        }
        if poll_result < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err("remote-ticket-input-poll-failed".to_string());
        }
        if poll_fd.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
            return Err("remote-ticket-input-poll-failed".to_string());
        }
        let read_count = unsafe { libc::read(fd, chunk.as_mut_ptr().cast(), chunk.len()) };
        if read_count == 0 {
            break;
        }
        if read_count < 0 {
            let kind = std::io::Error::last_os_error().kind();
            if matches!(kind, std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock) {
                continue;
            }
            return Err("remote-ticket-input-read-failed".to_string());
        }
        let read_count = usize::try_from(read_count).map_err(|_| "remote-ticket-input-size-invalid".to_string())?;
        let next_len =
            bytes.len().checked_add(read_count).ok_or_else(|| "remote-ticket-input-size-invalid".to_string())?;
        if next_len > credential_capacity {
            return Err("remote-ticket-input-size-limit-exceeded".to_string());
        }
        bytes.extend_from_slice(&chunk[..read_count]);
        assert_eq!(bytes.capacity(), allocation_capacity);
    }
    drop(file);
    let value = std::str::from_utf8(&bytes).map_err(|_| "remote-ticket-input-utf8-invalid".to_string())?;
    let value = value.strip_suffix('\n').unwrap_or(value);
    let value = value.strip_suffix('\r').unwrap_or(value);
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err("remote-ticket-input-format-invalid".to_string());
    }
    parse_remote_ticket_credential(value)
}

#[cfg(not(unix))]
pub fn read_remote_ticket_credential_from_owned_fd(_fd: i32) -> Result<RemoteTicketCredential, String> {
    Err("remote-ticket-input-fd-unsupported".to_string())
}

pub fn parse_remote_ticket_credential(token: &str) -> Result<RemoteTicketCredential, String> {
    let (ticket_id, secret) = token.split_once(':').ok_or_else(|| "remote-ticket-token-missing-colon".to_string())?;
    if ticket_id.is_empty() {
        return Err("remote-ticket-id-empty".to_string());
    }
    if secret.is_empty() {
        return Err("remote-ticket-secret-empty".to_string());
    }
    crate::remote_credentials::validate_presented_ticket_token(secret)?;
    Ok(RemoteTicketCredential {
        ticket_id: ticket_id.to_string(),
        secret: secret.to_string(),
    })
}

pub fn plan_remote_stdio_client_dispatch(
    input: RemoteClientDerivationInput,
    options: &RemoteClientBuildOptions,
) -> Result<RemoteClientDispatchPlan, String> {
    plan_remote_client_dispatch_for_binding(input, options, RemoteTransportBinding::Stdio)
}

pub fn plan_remote_ssh_stdio_client_dispatch(
    input: RemoteClientDerivationInput,
    options: &RemoteClientBuildOptions,
) -> Result<RemoteClientDispatchPlan, String> {
    plan_remote_client_dispatch_for_binding(input, options, RemoteTransportBinding::SshStdio)
}

pub fn plan_remote_stdio_replay_dispatch(
    request: ConcreteBuildRequest,
    options: &RemoteClientBuildOptions,
) -> Result<RemoteClientDispatchPlan, String> {
    validate_remote_client_options(options)?;
    plan_remote_executable_request(&request)?;
    if request.production_attempt.is_some() || request.transfer_policy.is_some() {
        return Err("remote-replay-request-stale-authority".to_string());
    }
    if request.failure_replay.is_none() {
        return Err("remote-replay-binding-missing".to_string());
    }
    if request.store_prefix != options.store_prefix {
        return Err("remote-replay-request-store-prefix-mismatch".to_string());
    }
    let client = remote_loopback_client_for_request(request, options);
    let command = RemoteStdioCommand {
        binding: RemoteTransportBinding::Stdio,
        program: options.builder.program.clone(),
        args: options.builder.args.clone(),
        input_frames: remote_client_request_frames(&client),
        timeout_secs: options.build_time_limit_secs,
        production_transfer: None,
    };
    let dispatch = RemoteClientDispatchPlan {
        label: "remote-failure-replay".to_string(),
        command,
        client,
    };
    debug_assert_eq!(dispatch.command.binding, RemoteTransportBinding::Stdio);
    debug_assert!(dispatch.client.request.failure_replay.is_some());
    Ok(dispatch)
}

fn plan_remote_client_dispatch_for_binding(
    input: RemoteClientDerivationInput,
    options: &RemoteClientBuildOptions,
    binding: RemoteTransportBinding,
) -> Result<RemoteClientDispatchPlan, String> {
    validate_remote_client_options(options)?;
    let request = concrete_remote_derivation_request(&input, options)?;
    let client = remote_loopback_client_for_request(request, options);
    let command = RemoteStdioCommand {
        binding,
        program: options.builder.program.clone(),
        args: options.builder.args.clone(),
        input_frames: remote_client_request_frames(&client),
        timeout_secs: options.build_time_limit_secs,
        production_transfer: None,
    };
    Ok(RemoteClientDispatchPlan {
        label: input.label,
        command,
        client,
    })
}

pub fn concrete_remote_derivation_request(
    input: &RemoteClientDerivationInput,
    options: &RemoteClientBuildOptions,
) -> Result<ConcreteBuildRequest, String> {
    validate_remote_client_options(options)?;
    let drv_json = serde_json::to_string(&input.crunch_derivation)
        .map_err(|err| format!("remote-client-derivation-json-invalid: {err}"))?;
    let expected_outputs = remote_expected_outputs_from_derivation(&input.nix_derivation, &options.store_prefix)?;
    let input_refs = remote_input_refs_from_derivation(&input.nix_derivation, &options.store_prefix);
    let source_input_refs = remote_source_input_refs_from_derivation(&input.nix_derivation, &options.store_prefix);
    let upload_bytes = remote_input_ref_upload_bytes(&input_refs)?;
    let request = ConcreteBuildRequest {
        request_id: remote_client_request_id(&input.label, &input.drv_path, &options.store_prefix),
        store_prefix: options.store_prefix.clone(),
        input_refs,
        source_input_refs,
        upload_bytes,
        build_time_limit_secs: options.build_time_limit_secs,
        contains_raw_frontend_eval: false,
        payload: RemoteConcreteBuildPayload::Derivation {
            drv_path: input.drv_path.to_absolute_path_with_prefix(&options.store_prefix),
            drv_json,
        },
        expected_outputs,
        production_attempt: None,
        transfer_policy: None,
        resource_requirements: None,
        locality_scope: None,
        failure_debug_policy: default_remote_failure_debug_policy(),
        failure_replay: None,
    };
    debug_assert_eq!(request.store_prefix, options.store_prefix);
    debug_assert!(is_blake3_hex_digest(&request.request_id));
    Ok(request)
}

pub fn remote_expected_outputs_from_derivation(
    derivation: &nix_compat::derivation::Derivation,
    store_prefix: &str,
) -> Result<Vec<RemoteExpectedOutput>, String> {
    if !store_prefix.starts_with('/') {
        return Err("remote-client-store-prefix-not-absolute".to_string());
    }
    if derivation.outputs.is_empty() {
        return Err("remote-client-outputs-empty".to_string());
    }
    if derivation.outputs.len() > MAX_REMOTE_EXPECTED_OUTPUTS {
        return Err(format!("remote-client-output-count-exceeds-{MAX_REMOTE_EXPECTED_OUTPUTS}"));
    }
    let mut outputs = Vec::with_capacity(derivation.outputs.len());
    for (name, output) in &derivation.outputs {
        outputs.push(RemoteExpectedOutput {
            name: name.clone(),
            logical_path: output.path.as_ref().map(|path| path.to_absolute_path_with_prefix(store_prefix)),
        });
    }
    Ok(outputs)
}

pub fn remote_input_refs_from_derivation(
    derivation: &nix_compat::derivation::Derivation,
    store_prefix: &str,
) -> Vec<String> {
    let mut refs =
        Vec::with_capacity(derivation.input_derivations.len().saturating_add(derivation.input_sources.len()));
    refs.extend(derivation.input_derivations.keys().map(|path| path.to_absolute_path_with_prefix(store_prefix)));
    refs.extend(derivation.input_sources.iter().map(|path| path.to_absolute_path_with_prefix(store_prefix)));
    refs.sort();
    refs.dedup();
    refs
}

pub fn remote_source_input_refs_from_derivation(
    derivation: &nix_compat::derivation::Derivation,
    store_prefix: &str,
) -> Vec<String> {
    let mut refs = derivation
        .input_sources
        .iter()
        .map(|path| path.to_absolute_path_with_prefix(store_prefix))
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    refs
}

pub fn remote_input_ref_upload_bytes(input_refs: &[String]) -> Result<u64, String> {
    let mut total = 0_u64;
    for input_ref in input_refs {
        let len = u64::try_from(input_ref.len()).map_err(|_| "remote-client-input-ref-size-overflow".to_string())?;
        total = total.checked_add(len).ok_or_else(|| "remote-client-input-upload-bytes-overflow".to_string())?;
    }
    if total > MAX_REMOTE_UPLOAD_BYTES {
        return Err("remote-client-input-upload-bytes-exceeded".to_string());
    }
    Ok(total)
}

fn validate_remote_client_options(options: &RemoteClientBuildOptions) -> Result<(), String> {
    if options.store_prefix.is_empty() || !options.store_prefix.starts_with('/') {
        return Err("remote-client-store-prefix-not-absolute".to_string());
    }
    if options.builder.endpoint_id.is_empty() {
        return Err("remote-client-builder-endpoint-empty".to_string());
    }
    if options.builder.program.as_os_str().is_empty() {
        return Err("remote-client-builder-program-empty".to_string());
    }
    if options.trusted_output_keys.is_empty() {
        return Err("remote-client-trusted-output-keys-empty".to_string());
    }
    if options.build_time_limit_secs == 0 || options.build_time_limit_secs > MAX_REMOTE_BUILD_TIME_SECS {
        return Err("remote-client-build-time-limit-invalid".to_string());
    }
    Ok(())
}

fn remote_loopback_client_for_request(
    request: ConcreteBuildRequest,
    options: &RemoteClientBuildOptions,
) -> RemoteLoopbackClient {
    let input_manifest = RemoteInputManifest {
        request_id: request.request_id.clone(),
        store_prefix: request.store_prefix.clone(),
        input_refs: request.input_refs.clone(),
        closure_refs: request.input_refs.clone(),
    };
    let session_id = remote_client_session_id(&request.request_id);
    let client = RemoteLoopbackClient {
        session_id,
        hello: RemoteHello {
            alpn: REMOTE_PROTOCOL_ALPN.to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: options.builder.endpoint_id.clone(),
            capabilities: options.transfer_capabilities.as_capability_labels(),
            workspace_policy: None,
        },
        auth: TicketAuthRequest {
            ticket_id: options.ticket.ticket_id.clone(),
            secret: options.ticket.secret.clone(),
            client_endpoint: options.client_endpoint.clone(),
            now_unix_s: options.now_unix_s,
        },
        uploaded_input_refs: request.input_refs.clone(),
        request,
        input_manifest,
        trusted_output_keys: options.trusted_output_keys.clone(),
        transfer_capabilities: options.transfer_capabilities,
    };
    debug_assert_eq!(client.hello.endpoint_id, options.builder.endpoint_id);
    debug_assert_eq!(client.input_manifest.request_id, client.request.request_id);
    client
}

fn remote_client_request_id(label: &str, drv_path: &StorePath<String>, store_prefix: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_CLIENT_REQUEST_ID_LABEL);
    hash_labeled_str(&mut hasher, "label", label);
    hash_labeled_str(&mut hasher, "drv-path", drv_path.to_absolute_path_with_prefix(store_prefix));
    hasher.finalize().to_hex().to_string()
}

fn remote_client_session_id(request_id: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_CLIENT_SESSION_ID_LABEL);
    hash_labeled_str(&mut hasher, "request-id", request_id);
    hasher.finalize().to_hex().to_string()
}

pub fn plan_remote_builder_frames(
    builder: &RemoteLoopbackBuilder,
    ticket: &mut RemoteTicket,
    client_frames: &[RemoteFrame],
    client_transfer: RemoteTransferCapabilities,
) -> Result<RemoteBuilderFrameResponse, String> {
    let executor = RemoteFixtureExecutor;
    plan_remote_builder_frames_with_executor(builder, ticket, client_frames, client_transfer, &executor)
}

pub fn plan_remote_builder_frames_with_executor(
    builder: &RemoteLoopbackBuilder,
    ticket: &mut RemoteTicket,
    client_frames: &[RemoteFrame],
    client_transfer: RemoteTransferCapabilities,
    executor: &dyn RemoteBuildExecutor,
) -> Result<RemoteBuilderFrameResponse, String> {
    let authenticated_client_endpoint = ticket.bound_client_endpoint.clone();
    let service_facts = RemoteTicketAuthFacts {
        server_now_unix_s: ticket.created_unix_s.saturating_add(1),
        authenticated_client_endpoint: authenticated_client_endpoint.as_deref(),
    };
    plan_remote_builder_frames_with_executor_and_auth_facts(
        builder,
        ticket,
        client_frames,
        client_transfer,
        executor,
        service_facts,
    )
}

struct RemoteBuilderAdmission<'a> {
    request: &'a ConcreteBuildRequest,
    input_upload: RemoteInputUpload,
    client_transfer: RemoteTransferCapabilities,
    phase: RemoteProtocolPhase,
    response_frames: Vec<RemoteFrame>,
    missing: Vec<String>,
    ticket_uses_remaining: u32,
}

fn admit_remote_builder_frames_with_auth_facts<'a>(
    builder: &RemoteLoopbackBuilder,
    ticket: &mut RemoteTicket,
    client_frames: &'a [RemoteFrame],
    client_transfer: RemoteTransferCapabilities,
    service_facts: RemoteTicketAuthFacts<'_>,
) -> Result<RemoteBuilderAdmission<'a>, String> {
    let mut phase = RemoteProtocolPhase::Open;
    let mut frames = client_frames.iter();
    let hello = take_hello(&mut frames, &mut phase)?;
    let accepted = expect_accepted_hello(hello, builder)?;
    take_auth(&mut frames, &mut phase, ticket, service_facts)?;
    let auth_ok = RemoteFrame::AuthOk {
        auth: RemoteAuthOk {
            builder_signing_keys: vec![builder.signing_key_id.clone()],
            accepted_capabilities: accepted.accepted_capabilities,
        },
    };
    phase = validate_remote_transition(phase, RemoteFrameDirection::BuilderToClient, &auth_ok)?;
    let request = take_build_request(&mut frames, &mut phase, ticket)?;
    let manifest = take_input_manifest(&mut frames, &mut phase, request)?;
    let missing = derive_missing_inputs(&manifest.input_refs, &builder.present_input_refs)?;
    let missing_frame = RemoteFrame::MissingInputs {
        request_id: request.request_id.clone(),
        refs: missing.clone(),
    };
    phase = validate_remote_transition(phase, RemoteFrameDirection::BuilderToClient, &missing_frame)?;
    let input_upload = take_input_upload(&mut frames, &mut phase, request, &missing, ticket)?;
    reject_extra_client_frames(frames.next())?;
    redeem_after_queue(ticket, true)?;
    Ok(RemoteBuilderAdmission {
        request,
        input_upload,
        client_transfer,
        phase,
        response_frames: vec![auth_ok, missing_frame],
        missing,
        ticket_uses_remaining: ticket.uses_remaining,
    })
}

fn execute_remote_builder_admission(
    builder: &RemoteLoopbackBuilder,
    admission: RemoteBuilderAdmission<'_>,
    executor: &dyn RemoteBuildExecutor,
) -> Result<RemoteBuilderFrameResponse, String> {
    let response = build_response_frames(BuildResponseFramesInput {
        builder,
        request: admission.request,
        input_upload: &admission.input_upload,
        client_transfer: admission.client_transfer,
        phase: admission.phase,
        response_frames: admission.response_frames,
        missing: admission.missing,
        ticket_uses_remaining: admission.ticket_uses_remaining,
        executor,
    })?;
    debug_assert_eq!(response.output_digest_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert!(!response.response_frames.is_empty());
    Ok(response)
}

fn plan_remote_builder_frames_with_executor_and_auth_facts(
    builder: &RemoteLoopbackBuilder,
    ticket: &mut RemoteTicket,
    client_frames: &[RemoteFrame],
    client_transfer: RemoteTransferCapabilities,
    executor: &dyn RemoteBuildExecutor,
    service_facts: RemoteTicketAuthFacts<'_>,
) -> Result<RemoteBuilderFrameResponse, String> {
    let admission =
        admit_remote_builder_frames_with_auth_facts(builder, ticket, client_frames, client_transfer, service_facts)?;
    execute_remote_builder_admission(builder, admission, executor)
}

pub fn serve_stdio_remote_once(
    reader: impl Read,
    mut writer: impl Write,
    builder: &RemoteLoopbackBuilder,
    ticket: &mut RemoteTicket,
    client_transfer: RemoteTransferCapabilities,
) -> Result<RemoteBuilderFrameResponse, String> {
    let input = read_bounded_stdio_input(reader)?;
    let client_frames = decode_remote_frame_stream(&input)?;
    let response = plan_remote_builder_frames(builder, ticket, &client_frames, client_transfer)?;
    write_remote_response_frames(&mut writer, &response)?;
    Ok(response)
}

pub fn plan_stdio_remote_once_from_state(
    reader: impl Read,
    builder: &RemoteLoopbackBuilder,
    state: &mut RemoteTicketState,
    client_transfer: RemoteTransferCapabilities,
) -> Result<RemoteBuilderFrameResponse, String> {
    let input = read_bounded_stdio_input(reader)?;
    let client_frames = decode_remote_frame_stream(&input)?;
    plan_remote_builder_frames_from_state(builder, state, &client_frames, client_transfer)
}

pub fn plan_stdio_remote_once_from_state_with_executor(
    reader: impl Read,
    builder: &RemoteLoopbackBuilder,
    state: &mut RemoteTicketState,
    client_transfer: RemoteTransferCapabilities,
    executor: &dyn RemoteBuildExecutor,
) -> Result<RemoteBuilderFrameResponse, String> {
    let input = read_bounded_stdio_input(reader)?;
    let client_frames = decode_remote_frame_stream(&input)?;
    plan_remote_builder_frames_from_state_with_executor(builder, state, &client_frames, client_transfer, executor)
}

pub fn plan_remote_builder_frames_from_state(
    builder: &RemoteLoopbackBuilder,
    state: &mut RemoteTicketState,
    client_frames: &[RemoteFrame],
    client_transfer: RemoteTransferCapabilities,
) -> Result<RemoteBuilderFrameResponse, String> {
    let executor = RemoteFixtureExecutor;
    plan_remote_builder_frames_from_state_with_executor(builder, state, client_frames, client_transfer, &executor)
}

pub fn plan_remote_builder_frames_from_state_with_executor(
    builder: &RemoteLoopbackBuilder,
    state: &mut RemoteTicketState,
    client_frames: &[RemoteFrame],
    client_transfer: RemoteTransferCapabilities,
    executor: &dyn RemoteBuildExecutor,
) -> Result<RemoteBuilderFrameResponse, String> {
    let ticket_id = request_ticket_id_from_frames(client_frames)?;
    let ticket = state.tickets.get_mut(ticket_id).ok_or_else(|| format!("unknown-remote-ticket-{ticket_id}"))?;
    plan_remote_builder_frames_with_executor(builder, ticket, client_frames, client_transfer, executor)
}

pub fn write_remote_response_frames(
    mut writer: impl Write,
    response: &RemoteBuilderFrameResponse,
) -> Result<(), String> {
    for frame in &response.response_frames {
        write_remote_frame(&mut writer, frame)?;
    }
    Ok(())
}

pub fn validate_remote_transition(
    phase: RemoteProtocolPhase,
    direction: RemoteFrameDirection,
    frame: &RemoteFrame,
) -> Result<RemoteProtocolPhase, String> {
    let kind = frame.kind();
    if kind == RemoteFrameKind::Error {
        return Ok(RemoteProtocolPhase::Failed);
    }
    let next_phase = validate_non_error_remote_transition(phase, direction, kind)?;
    debug_assert_ne!(kind, RemoteFrameKind::Error);
    debug_assert_ne!(next_phase, RemoteProtocolPhase::Failed);
    Ok(next_phase)
}

fn validate_non_error_remote_transition(
    phase: RemoteProtocolPhase,
    direction: RemoteFrameDirection,
    kind: RemoteFrameKind,
) -> Result<RemoteProtocolPhase, String> {
    match (phase, direction, kind) {
        (RemoteProtocolPhase::Open, RemoteFrameDirection::ClientToBuilder, RemoteFrameKind::Hello) => {
            Ok(RemoteProtocolPhase::AwaitAuth)
        }
        (RemoteProtocolPhase::AwaitAuth, RemoteFrameDirection::ClientToBuilder, RemoteFrameKind::TraceContext) => {
            Ok(RemoteProtocolPhase::AwaitAuth)
        }
        (RemoteProtocolPhase::AwaitAuth, RemoteFrameDirection::ClientToBuilder, RemoteFrameKind::AuthTicket) => {
            Ok(RemoteProtocolPhase::AwaitAuthOk)
        }
        (RemoteProtocolPhase::AwaitAuthOk, RemoteFrameDirection::BuilderToClient, RemoteFrameKind::AuthOk) => {
            Ok(RemoteProtocolPhase::AwaitBuildRequest)
        }
        (
            RemoteProtocolPhase::AwaitBuildRequest,
            RemoteFrameDirection::BuilderToClient,
            RemoteFrameKind::TraceContextAck,
        ) => Ok(RemoteProtocolPhase::AwaitBuildRequest),
        (
            RemoteProtocolPhase::AwaitBuildRequest,
            RemoteFrameDirection::ClientToBuilder,
            RemoteFrameKind::BuildRequest,
        ) => Ok(RemoteProtocolPhase::AwaitInputManifest),
        (
            RemoteProtocolPhase::AwaitInputManifest,
            RemoteFrameDirection::ClientToBuilder,
            RemoteFrameKind::InputManifest,
        ) => Ok(RemoteProtocolPhase::AwaitMissingInputs),
        (
            RemoteProtocolPhase::AwaitMissingInputs,
            RemoteFrameDirection::BuilderToClient,
            RemoteFrameKind::MissingInputs,
        ) => Ok(RemoteProtocolPhase::AwaitInputUpload),
        (
            RemoteProtocolPhase::AwaitInputUpload,
            RemoteFrameDirection::ClientToBuilder,
            RemoteFrameKind::InputUpload,
        ) => Ok(RemoteProtocolPhase::AwaitQueueAdmission),
        (
            RemoteProtocolPhase::AwaitQueueAdmission,
            RemoteFrameDirection::BuilderToClient,
            RemoteFrameKind::BuildQueued,
        ) => Ok(RemoteProtocolPhase::Queued),
        (RemoteProtocolPhase::Queued, RemoteFrameDirection::BuilderToClient, RemoteFrameKind::BuildStarted) => {
            Ok(RemoteProtocolPhase::Building)
        }
        (RemoteProtocolPhase::Building, RemoteFrameDirection::BuilderToClient, RemoteFrameKind::BuildFinished) => {
            Ok(RemoteProtocolPhase::AwaitOutputTransfer)
        }
        (
            RemoteProtocolPhase::AwaitOutputTransfer,
            RemoteFrameDirection::BuilderToClient,
            RemoteFrameKind::OutputTransferArtifact,
        ) => Ok(RemoteProtocolPhase::AwaitOutputTransfer),
        (
            RemoteProtocolPhase::AwaitOutputTransfer,
            RemoteFrameDirection::BuilderToClient,
            RemoteFrameKind::OutputTransferDone,
        ) => Ok(RemoteProtocolPhase::Done),
        (RemoteProtocolPhase::Done, RemoteFrameDirection::BuilderToClient, RemoteFrameKind::Done) => {
            Ok(RemoteProtocolPhase::Done)
        }
        _ => invalid_remote_transition(phase, direction, kind),
    }
}

fn invalid_remote_transition(
    phase: RemoteProtocolPhase,
    direction: RemoteFrameDirection,
    kind: RemoteFrameKind,
) -> Result<RemoteProtocolPhase, String> {
    Err(format!(
        "unexpected-remote-frame phase={} direction={} frame={}",
        phase.as_str(),
        direction.as_str(),
        kind.as_str()
    ))
}

pub fn validate_input_manifest(manifest: &RemoteInputManifest, request: &ConcreteBuildRequest) -> Result<(), String> {
    if manifest.request_id != request.request_id {
        return Err("input-manifest-request-id-mismatch".to_string());
    }
    if manifest.store_prefix != request.store_prefix {
        return Err("input-manifest-store-prefix-mismatch".to_string());
    }
    if manifest.input_refs.len() > MAX_REMOTE_INPUT_REFS || manifest.closure_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}"));
    }
    if manifest.input_refs != request.input_refs {
        return Err("input-manifest-does-not-match-build-request".to_string());
    }
    Ok(())
}

struct MissingUploadValidationInput<'a> {
    missing_refs: &'a [String],
    uploaded_refs: &'a [String],
    uploaded_bytes: u64,
    max_upload_bytes: u64,
}

pub type MissingUploadsValidationFn = fn(&[String], &[String], u64, u64) -> Result<(), String>;

pub const VALIDATE_MISSING_UPLOADS: MissingUploadsValidationFn =
    |missing_refs, uploaded_refs, uploaded_bytes, max_upload_bytes| {
        validate_missing_uploads_core(MissingUploadValidationInput {
            missing_refs,
            uploaded_refs,
            uploaded_bytes,
            max_upload_bytes,
        })
    };
pub use VALIDATE_MISSING_UPLOADS as validate_missing_uploads;

fn validate_missing_uploads_core(input: MissingUploadValidationInput<'_>) -> Result<(), String> {
    if input.uploaded_bytes > input.max_upload_bytes || input.uploaded_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err("upload-byte-limit-exceeded".to_string());
    }
    let missing = input.missing_refs.iter().collect::<BTreeSet<_>>();
    let uploaded = input.uploaded_refs.iter().collect::<BTreeSet<_>>();
    if missing != uploaded {
        return Err("uploaded-input-set-does-not-match-missing-set".to_string());
    }
    Ok(())
}

pub fn validate_remote_input_upload_artifacts(
    request_id: &str,
    uploaded_refs: &[String],
    source_input_refs: &[String],
    artifacts: &[RemoteInputUploadArtifact],
) -> Result<(), String> {
    if artifacts.len() > MAX_REMOTE_INPUT_UPLOAD_ARTIFACTS {
        return Err(format!("input-upload-artifact-count-exceeds-{MAX_REMOTE_INPUT_UPLOAD_ARTIFACTS}"));
    }
    let expected = expected_input_upload_artifact_refs(uploaded_refs, source_input_refs);
    let mut seen = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for artifact in artifacts {
        validate_remote_input_upload_artifact(request_id, artifact)?;
        if !expected.contains(artifact.input_ref.as_str()) {
            return Err("input-upload-artifact-unexpected".to_string());
        }
        if !seen.insert(artifact.input_ref.as_str()) {
            return Err("input-upload-artifact-duplicate".to_string());
        }
        total_bytes = total_bytes
            .checked_add(artifact.size_bytes)
            .ok_or_else(|| "input-upload-artifact-total-bytes-overflow".to_string())?;
        if total_bytes > MAX_REMOTE_UPLOAD_BYTES {
            return Err("input-upload-artifact-total-bytes-exceeded".to_string());
        }
    }
    for expected_ref in expected {
        if !seen.contains(expected_ref) {
            return Err("input-upload-artifact-missing".to_string());
        }
    }
    debug_assert_eq!(seen.len(), artifacts.len());
    debug_assert!(total_bytes <= MAX_REMOTE_UPLOAD_BYTES);
    Ok(())
}

fn expected_input_upload_artifact_refs<'a>(
    uploaded_refs: &'a [String],
    source_input_refs: &'a [String],
) -> BTreeSet<&'a str> {
    let sources = source_input_refs.iter().map(String::as_str).collect::<BTreeSet<_>>();
    uploaded_refs.iter().map(String::as_str).filter(|input_ref| sources.contains(input_ref)).collect()
}

fn validate_remote_input_upload_artifact(request_id: &str, artifact: &RemoteInputUploadArtifact) -> Result<(), String> {
    if artifact.request_id != request_id {
        return Err("input-upload-artifact-request-id-mismatch".to_string());
    }
    if artifact.input_ref.is_empty() {
        return Err("input-upload-artifact-ref-empty".to_string());
    }
    if !is_blake3_hex_digest(&artifact.digest_blake3) {
        return Err("input-upload-artifact-digest-invalid".to_string());
    }
    let payload_size_bytes = remote_payload_size_bytes(artifact.payload.len())?;
    if payload_size_bytes != artifact.size_bytes {
        return Err("input-upload-artifact-size-mismatch".to_string());
    }
    if blake3::hash(&artifact.payload).to_hex().to_string() != artifact.digest_blake3 {
        return Err("input-upload-artifact-digest-mismatch".to_string());
    }
    Ok(())
}

pub fn plan_output_transfer(
    client: RemoteTransferCapabilities,
    builder: RemoteTransferCapabilities,
    output_size_bytes: u64,
    verified_builder_key: &str,
) -> Result<RemoteTransferReport, String> {
    // A negotiated label is not runtime evidence. Only
    // `streaming_transfer_report_from_runtime` may report streaming mode.
    let is_delta_available = client.delta && builder.delta;
    let is_delta_failure_simulated = client.simulate_delta_failure || builder.simulate_delta_failure;
    if is_delta_available && !is_delta_failure_simulated {
        let delta_result = delta_transfer_report(output_size_bytes, verified_builder_key);
        debug_assert_eq!(delta_result.transferred_bytes.saturating_add(delta_result.reused_bytes), output_size_bytes);
        debug_assert_eq!(delta_result.verified_builder_key, verified_builder_key);
        return Ok(delta_result);
    }
    if is_delta_available && is_delta_failure_simulated {
        if client.full && builder.full {
            return Ok(full_transfer_report(
                output_size_bytes,
                verified_builder_key,
                Some("delta-transfer-failed".to_string()),
            ));
        }
        return Err("delta-transfer-failed-and-full-unavailable".to_string());
    }
    if client.full && builder.full {
        let fallback_reason = if client.streaming && builder.streaming {
            Some("streaming-runtime-not-bound".to_string())
        } else {
            None
        };
        return Ok(full_transfer_report(output_size_bytes, verified_builder_key, fallback_reason));
    }
    if client.streaming && builder.streaming {
        return Err("streaming-runtime-evidence-required".to_string());
    }
    Err("no-compatible-output-transfer-mode".to_string())
}

fn prevalidate_remote_streaming_output_metadata(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    result: &RemoteBuildFinished,
) -> Result<(), String> {
    if result.request_id != request.request_id {
        return Err("remote-output-request-id-mismatch".to_string());
    }
    if result.store_prefix != request.store_prefix {
        return Err("remote-output-store-prefix-mismatch".to_string());
    }
    if !is_blake3_hex_digest(&result.output_digest_blake3) {
        return Err("remote-output-digest-invalid".to_string());
    }
    validate_remote_produced_outputs(request, result)?;
    for output in &result.outputs {
        let path_info = output.path_info.as_ref().ok_or_else(|| "remote-output-pathinfo-missing".to_string())?;
        let store_path = parse_remote_output_store_path(&output.logical_path, &request.store_prefix)?;
        validate_remote_output_pathinfo(output, path_info, &store_path, &request.store_prefix)?;
    }
    match decide_output_trust(&result.builder_signing_key_id, trusted_output_keys, true) {
        OutputTrustDecision::Accept { .. } => {
            debug_assert_eq!(result.request_id, request.request_id);
            debug_assert_eq!(result.store_prefix, request.store_prefix);
            Ok(())
        }
        OutputTrustDecision::Reject(reason) => Err(reason),
    }
}

fn validate_output_transfer_manifest_frame(
    request: &ConcreteBuildRequest,
    result: &RemoteBuildFinished,
    transfer: &RemoteTransferManifestFrame,
    policy: RemoteTransferPolicy,
) -> Result<(), String> {
    if transfer.direction != crate::remote_transfer::RemoteTransferDirection::Download {
        return Err("remote-output-transfer-direction-mismatch".to_string());
    }
    let canonical = canonicalize_remote_transfer_manifest(transfer.manifest.clone(), policy)
        .map_err(|reason| reason.as_str().to_string())?;
    if canonical.digest_blake3 != transfer.manifest_digest_blake3 {
        return Err(RemoteTransferReasonCode::ManifestIdentityMismatch.as_str().to_string());
    }
    let attempt = request.production_attempt.as_ref().ok_or_else(|| "remote-production-attempt-missing".to_string())?;
    if canonical.manifest.job_id != attempt.job_id
        || canonical.manifest.attempt_id != attempt.attempt_id
        || canonical.manifest.fence_generation != attempt.fence_generation
    {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    if canonical.manifest.store_prefix != request.store_prefix
        || canonical.manifest.requested_content_blake3.as_str() != result.output_digest_blake3
    {
        return Err("remote-output-transfer-manifest-binding-mismatch".to_string());
    }
    let expected_artifact_count = result
        .outputs
        .len()
        .checked_mul(REMOTE_TRANSFER_ARTIFACTS_PER_PATHINFO_OUTPUT)
        .ok_or_else(|| "remote-output-transfer-artifact-count-overflow".to_string())?;
    if canonical.manifest.artifacts.len() != expected_artifact_count {
        return Err("remote-output-transfer-manifest-artifact-count-mismatch".to_string());
    }
    for output in &result.outputs {
        validate_output_transfer_manifest_artifacts(output, &canonical.manifest.artifacts)?;
    }
    if transfer.actual_mode == RemoteTransferMode::Delta {
        return Err("remote-output-transfer-full-manifest-labeled-delta".to_string());
    }
    if transfer.actual_mode == RemoteTransferMode::Streaming && transfer.fallback_reason.is_some() {
        return Err("remote-output-transfer-streaming-fallback-inconsistent".to_string());
    }
    let artifact_count = usize::try_from(canonical.artifact_count)
        .map_err(|_| "remote-output-transfer-artifact-count-conversion-failed".to_string())?;
    assert_eq!(artifact_count, expected_artifact_count);
    assert!(canonical.total_bytes > 0);
    Ok(())
}

fn validate_output_transfer_manifest_artifacts(
    output: &RemoteProducedOutput,
    artifacts: &[crunch_build::distributed::RemoteTransferArtifact],
) -> Result<(), String> {
    let nar_id = remote_output_nar_artifact_id(&output.name, &output.logical_path)?;
    let nar = artifacts
        .iter()
        .find(|artifact| artifact.artifact_id == nar_id)
        .ok_or_else(|| "remote-output-transfer-nar-manifest-missing".to_string())?;
    if nar.artifact_kind != RemoteTransferArtifactKind::Nar {
        return Err("remote-output-transfer-nar-manifest-mismatch".to_string());
    }
    if Some(nar.digest_blake3.as_str()) != output.nar_payload_digest_blake3.as_deref() {
        return Err("remote-output-transfer-nar-manifest-mismatch".to_string());
    }
    if Some(nar.size_bytes) != output.nar_payload_size_bytes {
        return Err("remote-output-transfer-nar-manifest-mismatch".to_string());
    }
    let path_info = output.path_info.as_ref().ok_or_else(|| "remote-output-pathinfo-missing".to_string())?;
    let pathinfo_id = remote_output_pathinfo_artifact_id(&output.name, &output.logical_path)?;
    let pathinfo = artifacts
        .iter()
        .find(|artifact| artifact.artifact_id == pathinfo_id)
        .ok_or_else(|| "remote-output-transfer-pathinfo-manifest-missing".to_string())?;
    let pathinfo_bytes = serialize_remote_pathinfo_payload(path_info)?;
    if pathinfo.artifact_kind != RemoteTransferArtifactKind::PathInfo {
        return Err("remote-output-transfer-pathinfo-manifest-mismatch".to_string());
    }
    if pathinfo.digest_blake3.as_str() != blake3::hash(&pathinfo_bytes).to_hex().as_str() {
        return Err("remote-output-transfer-pathinfo-manifest-mismatch".to_string());
    }
    if pathinfo.size_bytes != remote_payload_size_bytes(pathinfo_bytes.len())? {
        return Err("remote-output-transfer-pathinfo-manifest-mismatch".to_string());
    }
    debug_assert_eq!(nar.artifact_kind, RemoteTransferArtifactKind::Nar);
    debug_assert_eq!(pathinfo.artifact_kind, RemoteTransferArtifactKind::PathInfo);
    Ok(())
}

pub fn validate_remote_output_admission(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    result: &RemoteBuildFinished,
    transfer: &RemoteTransferReport,
    transfer_artifacts: &[RemoteOutputTransferArtifact],
) -> Result<RemoteOutputAdmissionReport, String> {
    if result.request_id != request.request_id {
        return Err("remote-output-request-id-mismatch".to_string());
    }
    if result.store_prefix != request.store_prefix {
        return Err("remote-output-store-prefix-mismatch".to_string());
    }
    if !is_blake3_hex_digest(&result.output_digest_blake3) {
        return Err("remote-output-digest-invalid".to_string());
    }
    validate_remote_produced_outputs(request, result)?;
    validate_remote_output_transfer_artifacts(&request.request_id, &result.outputs, transfer_artifacts)?;
    validate_transfer_report(transfer)?;
    if transfer.verified_builder_key != result.builder_signing_key_id {
        return Err("remote-transfer-builder-key-mismatch".to_string());
    }
    match decide_output_trust(&result.builder_signing_key_id, trusted_output_keys, true) {
        OutputTrustDecision::Accept { key_id, trust_basis } => {
            let admission = RemoteOutputAdmissionReport {
                request_id: request.request_id.clone(),
                output_digest_blake3: result.output_digest_blake3.clone(),
                builder_signing_key_id: key_id,
                trust_basis,
                store_prefix: result.store_prefix.clone(),
                outputs: result.outputs.clone(),
                transfer_artifacts: transfer_artifacts.to_vec(),
                streamed_manifest: None,
                transfer: transfer.clone(),
            };
            debug_assert_eq!(admission.output_digest_blake3, result.output_digest_blake3);
            debug_assert_eq!(admission.transfer, *transfer);
            Ok(admission)
        }
        OutputTrustDecision::Reject(reason) => Err(reason),
    }
}

pub fn validate_remote_builder_response_output_import(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    response: &RemoteBuilderFrameResponse,
) -> Result<RemoteOutputAdmissionReport, String> {
    let output_frames = extract_output_import_frames(&response.response_frames, &request.request_id)?;
    if output_frames.transfer != &response.transfer {
        return Err("remote-output-transfer-report-mismatch".to_string());
    }
    if output_frames.result.outputs != response.outputs {
        return Err("remote-output-metadata-report-mismatch".to_string());
    }
    if output_frames.artifacts != response.transfer_artifacts {
        return Err("remote-output-transfer-artifact-report-mismatch".to_string());
    }
    validate_remote_output_admission(
        request,
        trusted_output_keys,
        output_frames.result,
        output_frames.transfer,
        &output_frames.artifacts,
    )
}

struct FencedRemoteStdioOutputInput<'a> {
    binding: &'a RemoteProductionAttemptBinding,
    worker_authorized: bool,
    request: &'a ConcreteBuildRequest,
    trusted_output_keys: &'a [String],
    transcript: &'a RemoteStdioTranscript,
    event_number: u64,
}

pub type FencedRemoteStdioOutputAdmissionFn = fn(
    &mut RemoteCoordinatorState,
    &RemoteProductionAttemptBinding,
    bool,
    &ConcreteBuildRequest,
    &[String],
    &RemoteStdioTranscript,
    u64,
) -> Result<RemoteOutputAdmissionReport, String>;

pub const ADMIT_FENCED_REMOTE_STDIO_OUTPUT: FencedRemoteStdioOutputAdmissionFn =
    |state, binding, worker_authorized, request, trusted_output_keys, transcript, event_number| {
        admit_fenced_remote_stdio_output_core(state, FencedRemoteStdioOutputInput {
            binding,
            worker_authorized,
            request,
            trusted_output_keys,
            transcript,
            event_number,
        })
    };
pub use ADMIT_FENCED_REMOTE_STDIO_OUTPUT as admit_fenced_remote_stdio_output;

fn admit_fenced_remote_stdio_output_core(
    state: &mut RemoteCoordinatorState,
    input: FencedRemoteStdioOutputInput<'_>,
) -> Result<RemoteOutputAdmissionReport, String> {
    let FencedRemoteStdioOutputInput {
        binding,
        worker_authorized,
        request,
        trusted_output_keys,
        transcript,
        event_number,
    } = input;
    let (result, _) = extract_streaming_output_frames(&transcript.frames, &request.request_id)?;
    let result_ready_event =
        production_attempt_report(binding, "result-ready", event_number, RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: result.output_digest_blake3.clone(),
        })?;
    let preflight = preflight_fenced_output_report(state, &result_ready_event, worker_authorized);
    if preflight.disposition == RemoteAttemptApplyDisposition::Rejected {
        return Err(preflight.reason_code.as_str().to_string());
    }
    let admission = validate_remote_stdio_output_import(request, trusted_output_keys, transcript)?;
    if admission.output_digest_blake3 != result.output_digest_blake3 {
        return Err(RemoteAttemptReasonCode::ResultDigestMismatch.as_str().to_string());
    }
    if preflight.disposition == RemoteAttemptApplyDisposition::Applied {
        let applied = apply_coordinator_attempt_report(
            state,
            &result_ready_event,
            RemoteAttemptAuthorizationFacts {
                worker_authorized,
                output_admission_authorized: true,
            },
            RemoteLogRetentionPolicy::default(),
        )?;
        if !applied.output_admission_allowed {
            return Err("remote-fenced-output-admission-invariant".to_string());
        }
    }
    assert_eq!(admission.output_digest_blake3, result.output_digest_blake3);
    assert!(preflight.output_admission_allowed);
    Ok(admission)
}

pub fn complete_remote_production_attempt(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    output_digest_blake3: &str,
    event_number: u64,
) -> Result<(), String> {
    let completion_event =
        production_attempt_report(binding, "completion", event_number, RemoteAttemptReportPayload::Completion {
            output_digest_blake3: output_digest_blake3.to_string(),
        })?;
    let applied = apply_coordinator_attempt_report(
        state,
        &completion_event,
        RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: true,
        },
        RemoteLogRetentionPolicy::default(),
    )?;
    if applied.disposition != RemoteAttemptApplyDisposition::Applied {
        return Err(applied.reason_code.as_str().to_string());
    }
    Ok(())
}

pub fn fail_remote_production_attempt(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    event_number: u64,
) -> Result<(), String> {
    let failure_event =
        production_attempt_report(binding, "failure", event_number, RemoteAttemptReportPayload::Failure {
            failure_class: RemoteAttemptFailureClass::Retryable,
            reason_code: RemoteAttemptReasonCode::RetryAllowed,
        })?;
    let applied = apply_coordinator_attempt_report(
        state,
        &failure_event,
        RemoteAttemptAuthorizationFacts {
            worker_authorized: true,
            output_admission_authorized: false,
        },
        RemoteLogRetentionPolicy::default(),
    )?;
    if applied.disposition != RemoteAttemptApplyDisposition::Applied {
        return Err(applied.reason_code.as_str().to_string());
    }
    debug_assert_eq!(applied.disposition, RemoteAttemptApplyDisposition::Applied);
    debug_assert!(!applied.output_admission_allowed);
    Ok(())
}

pub fn record_remote_failure_debug_status(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    status: RemoteFailureDebugStatus,
) -> Result<(), String> {
    let mut candidate = state.clone();
    let job = candidate
        .jobs
        .get_mut(&binding.job_id)
        .ok_or_else(|| "remote-coordinator-job-unknown".to_string())?;
    let current = job.current_attempt.as_ref().ok_or_else(|| "remote-coordinator-attempt-missing".to_string())?;
    if current.attempt_id != binding.attempt_id || current.fence_generation != binding.fence_generation {
        return Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string());
    }
    job.failure_debug = Some(status);
    persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    Ok(())
}

struct FencedRemoteBuilderResponseInput<'a> {
    report: &'a RemoteAttemptReport,
    worker_authorized: bool,
    log_policy: RemoteLogRetentionPolicy,
    request: &'a ConcreteBuildRequest,
    trusted_output_keys: &'a [String],
    response: &'a RemoteBuilderFrameResponse,
}

pub type FencedRemoteBuilderResponseAdmissionFn = fn(
    &mut RemoteCoordinatorState,
    &RemoteAttemptReport,
    bool,
    RemoteLogRetentionPolicy,
    &ConcreteBuildRequest,
    &[String],
    &RemoteBuilderFrameResponse,
) -> Result<RemoteFencedOutputAdmissionDecision, String>;

pub const ADMIT_FENCED_REMOTE_BUILDER_RESPONSE: FencedRemoteBuilderResponseAdmissionFn =
    |state, report, worker_authorized, log_policy, request, trusted_output_keys, response| {
        admit_fenced_remote_builder_response_core(state, FencedRemoteBuilderResponseInput {
            report,
            worker_authorized,
            log_policy,
            request,
            trusted_output_keys,
            response,
        })
    };
pub use ADMIT_FENCED_REMOTE_BUILDER_RESPONSE as admit_fenced_remote_builder_response;
const _: FencedRemoteBuilderResponseAdmissionFn = admit_fenced_remote_builder_response;

fn admit_fenced_remote_builder_response_core(
    state: &mut RemoteCoordinatorState,
    input: FencedRemoteBuilderResponseInput<'_>,
) -> Result<RemoteFencedOutputAdmissionDecision, String> {
    let FencedRemoteBuilderResponseInput {
        report,
        worker_authorized,
        log_policy,
        request,
        trusted_output_keys,
        response,
    } = input;
    let preflight = preflight_fenced_output_report(state, report, worker_authorized);
    if preflight.disposition == RemoteAttemptApplyDisposition::Rejected {
        return Ok(RemoteFencedOutputAdmissionDecision::NoAdmission { attempt: preflight });
    }
    if preflight.disposition == RemoteAttemptApplyDisposition::AlreadyApplied
        && !already_applied_output_result_is_redeliverable(state, report)
    {
        return Ok(RemoteFencedOutputAdmissionDecision::NoAdmission { attempt: preflight });
    }
    let RemoteAttemptReportPayload::ResultReady { output_digest_blake3 } = &report.payload else {
        return Err("remote-fenced-output-report-kind-invalid".to_string());
    };
    if output_digest_blake3 != &response.output_digest_blake3 {
        return Err(RemoteAttemptReasonCode::ResultDigestMismatch.as_str().to_string());
    }
    let admission = validate_remote_builder_response_output_import(request, trusted_output_keys, response)?;
    let attempt = if preflight.disposition == RemoteAttemptApplyDisposition::Applied {
        let applied = apply_coordinator_attempt_report(
            state,
            report,
            RemoteAttemptAuthorizationFacts {
                worker_authorized,
                output_admission_authorized: true,
            },
            log_policy,
        )?;
        if !applied.output_admission_allowed {
            return Err("remote-fenced-output-admission-invariant".to_string());
        }
        applied
    } else {
        preflight
    };
    debug_assert_eq!(admission.output_digest_blake3, *output_digest_blake3);
    debug_assert!(matches!(
        attempt.disposition,
        RemoteAttemptApplyDisposition::Applied | RemoteAttemptApplyDisposition::AlreadyApplied
    ));
    Ok(RemoteFencedOutputAdmissionDecision::Admit {
        attempt,
        admission: Box::new(admission),
    })
}

fn already_applied_output_result_is_redeliverable(
    state: &RemoteCoordinatorState,
    report: &RemoteAttemptReport,
) -> bool {
    let RemoteAttemptReportPayload::ResultReady { output_digest_blake3 } = &report.payload else {
        return false;
    };
    let Some(job) = state.jobs.get(&report.identity.job_id) else {
        return false;
    };
    let Some(attempt) = job.current_attempt.as_ref() else {
        return false;
    };
    let is_phase_redeliverable =
        matches!(attempt.phase, RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed);
    is_phase_redeliverable
        && job.output_admission_completed
        && attempt.result_digest_blake3.as_deref() == Some(output_digest_blake3.as_str())
}

fn preflight_fenced_output_report(
    state: &RemoteCoordinatorState,
    report: &RemoteAttemptReport,
    worker_authorized: bool,
) -> RemoteCoordinatorAttemptApplyResult {
    let Some(job) = state.jobs.get(&report.identity.job_id) else {
        return rejected_attempt_apply_result(RemoteAttemptReasonCode::JobIdentityMismatch);
    };
    let Some(current) = job.current_attempt.as_ref() else {
        return rejected_attempt_apply_result(RemoteAttemptReasonCode::LegacyStateRejected);
    };
    let plan = plan_remote_attempt_report(current, report, RemoteAttemptAuthorizationFacts {
        worker_authorized,
        output_admission_authorized: true,
    });
    RemoteCoordinatorAttemptApplyResult {
        disposition: plan.disposition,
        reason_code: plan.reason_code,
        output_admission_allowed: plan.output_admission_allowed,
    }
}

pub fn validate_remote_builder_frames_output_import(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    frames: &[RemoteFrame],
) -> Result<RemoteOutputAdmissionReport, String> {
    let output_frames = extract_output_import_frames(frames, &request.request_id)?;
    validate_remote_output_admission(
        request,
        trusted_output_keys,
        output_frames.result,
        output_frames.transfer,
        &output_frames.artifacts,
    )
}

pub fn validate_remote_stdio_output_import(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    transcript: &RemoteStdioTranscript,
) -> Result<RemoteOutputAdmissionReport, String> {
    let Some(streaming) = transcript.streaming_output.as_ref() else {
        return validate_remote_builder_frames_output_import(request, trusted_output_keys, &transcript.frames);
    };
    let (result, transfer) = extract_streaming_output_frames(&transcript.frames, &request.request_id)?;
    prevalidate_remote_streaming_output_metadata(request, trusted_output_keys, result)?;
    let policy = request.transfer_policy.ok_or_else(|| "remote-production-transfer-policy-missing".to_string())?;
    let manifest_frame = RemoteTransferManifestFrame {
        direction: crate::remote_transfer::RemoteTransferDirection::Download,
        manifest: streaming.manifest.clone(),
        manifest_digest_blake3: streaming.manifest_digest_blake3.clone(),
        actual_mode: transfer.mode,
        fallback_reason: transfer.fallback_reason.clone(),
    };
    validate_output_transfer_manifest_frame(request, result, &manifest_frame, policy)?;
    validate_streamed_output_receiver_files(result, streaming, policy)?;
    validate_transfer_report(transfer)?;
    if transfer.verified_builder_key != result.builder_signing_key_id {
        return Err("remote-transfer-builder-key-mismatch".to_string());
    }
    let OutputTrustDecision::Accept { key_id, trust_basis } =
        decide_output_trust(&result.builder_signing_key_id, trusted_output_keys, true)
    else {
        return Err("untrusted-output-key".to_string());
    };
    let admission = RemoteOutputAdmissionReport {
        request_id: request.request_id.clone(),
        output_digest_blake3: result.output_digest_blake3.clone(),
        builder_signing_key_id: key_id,
        trust_basis,
        store_prefix: result.store_prefix.clone(),
        outputs: result.outputs.clone(),
        transfer_artifacts: Vec::new(),
        streamed_manifest: Some(streaming.manifest.clone()),
        transfer: transfer.clone(),
    };
    debug_assert_eq!(admission.output_digest_blake3, result.output_digest_blake3);
    debug_assert_eq!(admission.transfer, *transfer);
    Ok(admission)
}

fn extract_streaming_output_frames<'a>(
    frames: &'a [RemoteFrame],
    request_id: &str,
) -> Result<(&'a RemoteBuildFinished, &'a RemoteTransferReport), String> {
    let mut result = None;
    let mut transfer = None;
    let mut is_done = false;
    for frame in frames {
        match frame {
            RemoteFrame::BuildFinished { result: value } if result.replace(value).is_some() => {
                return Err("duplicate-build-finished-frame".to_string());
            }
            RemoteFrame::OutputTransferDone { report } if transfer.replace(report).is_some() => {
                return Err("duplicate-output-transfer-frame".to_string());
            }
            RemoteFrame::Done { request_id: value } => {
                if value != request_id || is_done {
                    return Err("remote-done-request-id-mismatch".to_string());
                }
                is_done = true;
            }
            RemoteFrame::OutputTransferArtifact { .. } => {
                return Err("remote-production-inline-output-artifact-forbidden".to_string());
            }
            RemoteFrame::Error { .. } => return Err("remote-builder-error-frame".to_string()),
            _ => {}
        }
    }
    if !is_done {
        return Err("missing-done-frame".to_string());
    }
    let result = result.ok_or_else(|| "missing-build-finished-frame".to_string())?;
    let transfer = transfer.ok_or_else(|| "missing-output-transfer-frame".to_string())?;
    debug_assert_eq!(result.request_id, request_id);
    debug_assert!(is_done);
    Ok((result, transfer))
}

fn validate_streamed_output_receiver_files(
    result: &RemoteBuildFinished,
    streaming: &RemoteStreamingOutputReceipt,
    policy: RemoteTransferPolicy,
) -> Result<(), String> {
    let canonical = canonicalize_remote_transfer_manifest(streaming.manifest.clone(), policy)
        .map_err(|reason| reason.as_str().to_string())?;
    let facts = crate::remote_transfer::probe_remote_transfer_receiver(
        &streaming.receiver_root,
        &canonical,
        crate::remote_transfer::RemoteTransferAdmissionFacts {
            required_closure_metadata_verified: true,
            path_info_admitted: true,
        },
    )?;
    if !facts.requested_content_identity_verified
        || facts.complete_artifact_ids.len() != canonical.manifest.artifacts.len()
    {
        return Err("remote-output-transfer-receiver-incomplete".to_string());
    }
    for output in &result.outputs {
        let path_info = output.path_info.as_ref().ok_or_else(|| "remote-output-pathinfo-missing".to_string())?;
        let pathinfo_id = remote_output_pathinfo_artifact_id(&output.name, &output.logical_path)?;
        let pathinfo_path =
            crate::remote_transfer::remote_transfer_received_artifact_path(&streaming.receiver_root, &pathinfo_id);
        let pathinfo_file = crate::remote_transfer::open_remote_transfer_authority_file(&pathinfo_path)
            .map_err(|err| format!("opening streamed PathInfo {}: {err}", pathinfo_path.display()))?;
        let mut observed_pathinfo = Vec::new();
        pathinfo_file
            .take(MAX_REMOTE_PATHINFO_READ_BYTES)
            .read_to_end(&mut observed_pathinfo)
            .map_err(|err| format!("reading streamed PathInfo {}: {err}", pathinfo_path.display()))?;
        if observed_pathinfo.len() > MAX_REMOTE_FRAME_BYTES {
            return Err("remote-output-streamed-pathinfo-too-large".to_string());
        }
        if observed_pathinfo != serialize_remote_pathinfo_payload(path_info)? {
            return Err("remote-output-streamed-pathinfo-mismatch".to_string());
        }
    }
    assert_eq!(facts.complete_artifact_ids.len(), canonical.manifest.artifacts.len());
    assert!(facts.requested_content_identity_verified);
    Ok(())
}

pub fn plan_remote_output_import_actions(
    request: &ConcreteBuildRequest,
    admission: &RemoteOutputAdmissionReport,
) -> Result<Vec<RemoteOutputImportAction>, String> {
    validate_admission_report_for_import(request, admission)?;
    let mut actions = Vec::with_capacity(admission.outputs.len());
    for output in &admission.outputs {
        actions.push(plan_remote_output_import_action(request, output, &admission.transfer_artifacts)?);
    }
    Ok(actions)
}

pub async fn import_admitted_remote_outputs(
    store: &mut crunch_store::StoreHandle,
    request: &ConcreteBuildRequest,
    admission: &RemoteOutputAdmissionReport,
    is_root: bool,
    root_source: Option<crunch_store::GcRootSource>,
) -> Result<RemoteOutputImportReport, String> {
    if admission.streamed_manifest.is_some() {
        return Err("remote-streamed-output-requires-receiver-root".to_string());
    }
    let actions = plan_remote_output_import_actions(request, admission)?;
    persist_remote_output_actions(store, admission, actions, is_root, root_source).await
}

struct AdmittedRemoteStdioOutputImportInput<'a> {
    store: &'a mut crunch_store::StoreHandle,
    request: &'a ConcreteBuildRequest,
    admission: &'a RemoteOutputAdmissionReport,
    transcript: &'a RemoteStdioTranscript,
    is_root: bool,
    root_source: Option<crunch_store::GcRootSource>,
}

type RemoteOutputImportFuture<'a> =
    Pin<Box<dyn std::future::Future<Output = Result<RemoteOutputImportReport, String>> + 'a>>;

pub const IMPORT_ADMITTED_REMOTE_STDIO_OUTPUTS: for<'a> fn(
    &'a mut crunch_store::StoreHandle,
    &'a ConcreteBuildRequest,
    &'a RemoteOutputAdmissionReport,
    &'a RemoteStdioTranscript,
    bool,
    Option<crunch_store::GcRootSource>,
) -> RemoteOutputImportFuture<'a> = |store, request, admission, transcript, is_root, root_source| {
    Box::pin(import_admitted_remote_stdio_outputs_core(AdmittedRemoteStdioOutputImportInput {
        store,
        request,
        admission,
        transcript,
        is_root,
        root_source,
    }))
};
pub use IMPORT_ADMITTED_REMOTE_STDIO_OUTPUTS as import_admitted_remote_stdio_outputs;

async fn import_admitted_remote_stdio_outputs_core(
    input: AdmittedRemoteStdioOutputImportInput<'_>,
) -> Result<RemoteOutputImportReport, String> {
    let Some(streaming) = input.transcript.streaming_output.as_ref() else {
        return import_admitted_remote_outputs(
            input.store,
            input.request,
            input.admission,
            input.is_root,
            input.root_source,
        )
        .await;
    };
    let policy = input
        .request
        .transfer_policy
        .ok_or_else(|| "remote-production-transfer-policy-missing".to_string())?;
    validate_streamed_output_receiver_files(
        &RemoteBuildFinished {
            request_id: input.admission.request_id.clone(),
            output_digest_blake3: input.admission.output_digest_blake3.clone(),
            builder_signing_key_id: input.admission.builder_signing_key_id.clone(),
            store_prefix: input.admission.store_prefix.clone(),
            outputs: input.admission.outputs.clone(),
        },
        streaming,
        policy,
    )?;
    let mut actions = Vec::with_capacity(input.admission.outputs.len());
    for output in &input.admission.outputs {
        let mut action = plan_streamed_remote_output_import_action(input.request, output, streaming)?;
        if action.nar_payload.is_some() {
            return Err("remote-production-inline-output-payload-forbidden".to_string());
        }
        action.nar_payload = None;
        actions.push(action);
    }
    let output_receipt =
        persist_remote_output_actions(input.store, input.admission, actions, input.is_root, input.root_source).await?;
    debug_assert_eq!(output_receipt.request_id, input.admission.request_id);
    debug_assert_eq!(output_receipt.outputs.len(), input.admission.outputs.len());
    Ok(output_receipt)
}

async fn persist_remote_output_actions(
    store: &mut crunch_store::StoreHandle,
    admission: &RemoteOutputAdmissionReport,
    actions: Vec<RemoteOutputImportAction>,
    is_root: bool,
    root_source: Option<crunch_store::GcRootSource>,
) -> Result<RemoteOutputImportReport, String> {
    let store_substitution = remote_transfer_to_store_report(&admission.transfer);
    let mut output_receipts = Vec::with_capacity(actions.len());
    for action in actions {
        let stored = persist_remote_output_action(store, &action, is_root, root_source).await?;
        store.record_verified_output_substitution_report(&action.store_path, store_substitution.clone());
        output_receipts.push(imported_remote_output_report(store, &action, &stored));
    }
    Ok(RemoteOutputImportReport {
        request_id: admission.request_id.clone(),
        store_prefix: admission.store_prefix.clone(),
        outputs: output_receipts,
        transfer: admission.transfer.clone(),
    })
}

fn plan_streamed_remote_output_import_action(
    request: &ConcreteBuildRequest,
    output: &RemoteProducedOutput,
    streaming: &RemoteStreamingOutputReceipt,
) -> Result<RemoteOutputImportAction, String> {
    let path_info = output.path_info.clone().ok_or_else(|| "remote-output-pathinfo-missing".to_string())?;
    let store_path = parse_remote_output_store_path(&output.logical_path, &request.store_prefix)?;
    validate_remote_output_pathinfo(output, &path_info, &store_path, &request.store_prefix)?;
    let nar_id = remote_output_nar_artifact_id(&output.name, &output.logical_path)?;
    let nar_path = crate::remote_transfer::remote_transfer_received_artifact_path(&streaming.receiver_root, &nar_id);
    crate::remote_transfer::open_remote_transfer_authority_file(&nar_path)
        .map_err(|_| "remote-output-streamed-nar-missing".to_string())?;
    let action = RemoteOutputImportAction {
        output_name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        store_path,
        final_node: path_info.node.clone(),
        path_info,
        path_info_signing_key_id: output.path_info_signing_key_id.clone(),
        artifact_attestation_digest_blake3: output.artifact_attestation_digest_blake3.clone(),
        nar_payload: None,
        nar_path: Some(nar_path),
    };
    debug_assert_eq!(action.output_name, output.name);
    debug_assert!(action.nar_path.is_some());
    Ok(action)
}

pub fn classify_remote_failure(phase: RemoteFailurePhase, reason: String) -> RemoteFailureClassification {
    let retry_class = match phase {
        RemoteFailurePhase::TransportSetup => RemoteRetryClass::Retryable,
        RemoteFailurePhase::BuildExecution => RemoteRetryClass::BuildOutcome,
        RemoteFailurePhase::Authentication
        | RemoteFailurePhase::RequestValidation
        | RemoteFailurePhase::InputSync
        | RemoteFailurePhase::Queue
        | RemoteFailurePhase::OutputImport => RemoteRetryClass::Terminal,
    };
    remote_failure(phase, retry_class, reason)
}

fn remote_failure(
    phase: RemoteFailurePhase,
    retry_class: RemoteRetryClass,
    reason: String,
) -> RemoteFailureClassification {
    RemoteFailureClassification {
        phase,
        retry_class,
        reason,
    }
}

fn validate_admission_report_for_import(
    request: &ConcreteBuildRequest,
    admission: &RemoteOutputAdmissionReport,
) -> Result<(), String> {
    let result = RemoteBuildFinished {
        request_id: admission.request_id.clone(),
        output_digest_blake3: admission.output_digest_blake3.clone(),
        builder_signing_key_id: admission.builder_signing_key_id.clone(),
        store_prefix: admission.store_prefix.clone(),
        outputs: admission.outputs.clone(),
    };
    let trusted = vec![admission.builder_signing_key_id.clone()];
    if admission.streamed_manifest.is_some() {
        prevalidate_remote_streaming_output_metadata(request, &trusted, &result)?;
        validate_transfer_report(&admission.transfer)?;
        return Ok(());
    }
    validate_remote_output_admission(request, &trusted, &result, &admission.transfer, &admission.transfer_artifacts)
        .map(|_| ())
}

fn plan_remote_output_import_action(
    request: &ConcreteBuildRequest,
    output: &RemoteProducedOutput,
    transfer_artifacts: &[RemoteOutputTransferArtifact],
) -> Result<RemoteOutputImportAction, String> {
    let path_info = output.path_info.clone().ok_or_else(|| "remote-output-pathinfo-missing".to_string())?;
    let store_path = parse_remote_output_store_path(&output.logical_path, &request.store_prefix)?;
    validate_remote_output_pathinfo(output, &path_info, &store_path, &request.store_prefix)?;
    let nar_payload = remote_output_nar_payload_for_import(output, transfer_artifacts)?;
    Ok(RemoteOutputImportAction {
        output_name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        store_path,
        final_node: path_info.node.clone(),
        path_info,
        path_info_signing_key_id: output.path_info_signing_key_id.clone(),
        artifact_attestation_digest_blake3: output.artifact_attestation_digest_blake3.clone(),
        nar_payload,
        nar_path: None,
    })
}

fn remote_output_nar_payload_for_import(
    output: &RemoteProducedOutput,
    transfer_artifacts: &[RemoteOutputTransferArtifact],
) -> Result<Option<Vec<u8>>, String> {
    if output.nar_payload_digest_blake3.is_none() && output.nar_payload_size_bytes.is_none() {
        return Ok(None);
    }
    let artifact = transfer_artifacts
        .iter()
        .find(|artifact| {
            artifact.output_name == output.name && artifact.artifact_kind == RemoteOutputTransferArtifactKind::Nar
        })
        .ok_or_else(|| "remote-output-nar-transfer-artifact-missing".to_string())?;
    if Some(artifact.digest_blake3.as_str()) != output.nar_payload_digest_blake3.as_deref() {
        return Err("remote-output-nar-transfer-digest-mismatch".to_string());
    }
    if Some(artifact.size_bytes) != output.nar_payload_size_bytes {
        return Err("remote-output-nar-transfer-size-mismatch".to_string());
    }
    Ok(Some(artifact.payload.clone()))
}

struct RemoteStorePathInput<'a> {
    logical_path: &'a str,
    store_prefix: &'a str,
}

const PARSE_REMOTE_OUTPUT_STORE_PATH: fn(&str, &str) -> Result<StorePath<String>, String> =
    |logical_path, store_prefix| {
        parse_remote_output_store_path_core(RemoteStorePathInput {
            logical_path,
            store_prefix,
        })
    };
use PARSE_REMOTE_OUTPUT_STORE_PATH as parse_remote_output_store_path;

fn parse_remote_output_store_path_core(input: RemoteStorePathInput<'_>) -> Result<StorePath<String>, String> {
    StorePath::from_absolute_path_with_prefix(input.logical_path.as_bytes(), input.store_prefix)
        .map_err(|_| "remote-output-store-path-invalid".to_string())
}

fn validate_remote_output_pathinfo(
    output: &RemoteProducedOutput,
    path_info: &PathInfo,
    store_path: &StorePath<String>,
    store_prefix: &str,
) -> Result<(), String> {
    validate_remote_pathinfo_binding(RemotePathInfoBindingInput {
        output_name: &output.name,
        artifact_attestation_digest_blake3: &output.artifact_attestation_digest_blake3,
        path_info,
        store_path,
        store_prefix,
        signing_key_id: &output.path_info_signing_key_id,
        error_labels: RemotePathInfoErrorLabels {
            store_path_mismatch: "remote-output-pathinfo-store-path-mismatch",
            unsigned: "remote-output-pathinfo-unsigned",
            signing_key_mismatch: "remote-output-pathinfo-signing-key-mismatch",
            artifact_digest_failed: "remote-output-artifact-attestation-digest-failed",
            artifact_digest_mismatch: "remote-output-artifact-attestation-digest-mismatch",
        },
    })
}

#[derive(Clone, Copy)]
struct RemotePathInfoErrorLabels {
    store_path_mismatch: &'static str,
    unsigned: &'static str,
    signing_key_mismatch: &'static str,
    artifact_digest_failed: &'static str,
    artifact_digest_mismatch: &'static str,
}

struct RemotePathInfoBindingInput<'a> {
    output_name: &'a str,
    artifact_attestation_digest_blake3: &'a str,
    path_info: &'a PathInfo,
    store_path: &'a StorePath<String>,
    store_prefix: &'a str,
    signing_key_id: &'a str,
    error_labels: RemotePathInfoErrorLabels,
}

fn validate_remote_pathinfo_binding(input: RemotePathInfoBindingInput<'_>) -> Result<(), String> {
    if input.path_info.store_path != *input.store_path {
        return Err(input.error_labels.store_path_mismatch.to_string());
    }
    if input.path_info.signatures.is_empty() {
        return Err(input.error_labels.unsigned.to_string());
    }
    if !pathinfo_has_signature_name(input.path_info, input.signing_key_id) {
        return Err(input.error_labels.signing_key_mismatch.to_string());
    }
    validate_remote_pathinfo_artifact_digest(&input)
}

fn validate_remote_pathinfo_artifact_digest(input: &RemotePathInfoBindingInput<'_>) -> Result<(), String> {
    let digest = crunch_store::artifact_attestation_digest_for_pathinfo(
        input.store_prefix,
        input.path_info,
        input.output_name,
        None,
    )
    .map_err(|err| format!("{}: {err}", input.error_labels.artifact_digest_failed))?
    .to_hex();
    if digest != input.artifact_attestation_digest_blake3 {
        return Err(input.error_labels.artifact_digest_mismatch.to_string());
    }
    Ok(())
}

fn pathinfo_has_signature_name(path_info: &PathInfo, signing_key_id: &str) -> bool {
    path_info.signatures.iter().any(|signature| signature.name().as_str() == signing_key_id)
}

fn remote_transfer_to_store_report(transfer: &RemoteTransferReport) -> crunch_store::OutputSubstitutionReport {
    crunch_store::OutputSubstitutionReport {
        mode: remote_transfer_mode_to_store_mode(transfer.mode),
        transferred_bytes: transfer.transferred_bytes,
        reused_bytes: transfer.reused_bytes,
        metadata_reused: false,
        fallback_reason: transfer.fallback_reason.clone(),
    }
}

fn remote_transfer_mode_to_store_mode(mode: RemoteTransferMode) -> crunch_store::OutputSubstitutionMode {
    match mode {
        RemoteTransferMode::Delta => crunch_store::OutputSubstitutionMode::Delta,
        RemoteTransferMode::Full => crunch_store::OutputSubstitutionMode::Full,
        RemoteTransferMode::Streaming => crunch_store::OutputSubstitutionMode::Full,
    }
}

async fn ingest_remote_output_nar_payload(
    store: &crunch_store::StoreHandle,
    action: &RemoteOutputImportAction,
) -> Result<(), String> {
    if action.nar_payload.is_some() && action.nar_path.is_some() {
        return Err("remote-output-nar-source-ambiguous".to_string());
    }
    let result = if let Some(payload) = &action.nar_payload {
        let mut reader = std::io::Cursor::new(payload.as_slice());
        snix_store::nar::ingest_nar_and_hash(
            store.blob_service(),
            store.directory_service(),
            &mut reader,
            &action.path_info.ca,
        )
        .await
    } else if let Some(path) = &action.nar_path {
        let reader = crate::remote_transfer::open_remote_transfer_authority_file(path)
            .map_err(|err| format!("remote-output-streamed-nar-open-failed: {err}"))?;
        let mut reader = tokio::fs::File::from_std(reader);
        snix_store::nar::ingest_nar_and_hash(
            store.blob_service(),
            store.directory_service(),
            &mut reader,
            &action.path_info.ca,
        )
        .await
    } else {
        return Ok(());
    };
    let (node, nar_sha256, nar_size) = result.map_err(|err| format!("remote-output-nar-ingest-failed: {err}"))?;
    if node != action.final_node {
        return Err("remote-output-nar-node-mismatch".to_string());
    }
    if nar_sha256 != action.path_info.nar_sha256 {
        return Err("remote-output-nar-sha256-mismatch".to_string());
    }
    if nar_size != action.path_info.nar_size {
        return Err("remote-output-nar-size-mismatch".to_string());
    }
    debug_assert_eq!(node, action.final_node);
    debug_assert_eq!(nar_size, action.path_info.nar_size);
    Ok(())
}

async fn persist_remote_output_action(
    store: &mut crunch_store::StoreHandle,
    action: &RemoteOutputImportAction,
    is_root: bool,
    root_source: Option<crunch_store::GcRootSource>,
) -> Result<PathInfo, String> {
    ingest_remote_output_nar_payload(store, action).await?;
    store
        .persist_and_export_signed_output(crunch_store::PersistOutputRequest {
            output_name: &action.output_name,
            output_path: &action.store_path,
            path_info: action.path_info.clone(),
            final_node: action.final_node.clone(),
            provenance: None,
            is_root,
            root_source,
        })
        .await
        .map_err(|err| format!("remote-output-persist-failed: {err}"))
}

fn imported_remote_output_report(
    store: &crunch_store::StoreHandle,
    action: &RemoteOutputImportAction,
    stored: &PathInfo,
) -> RemoteImportedOutput {
    debug_assert_eq!(stored.store_path, action.store_path, "persisted remote output path must match action");
    let artifact_path =
        crunch_store::artifact_attestation_file_path(store.state_dir(), store.store_dir(), &action.store_path);
    RemoteImportedOutput {
        name: action.output_name.clone(),
        logical_path: action.logical_path.clone(),
        path_info_signing_key_id: action.path_info_signing_key_id.clone(),
        artifact_attestation_digest_blake3: action.artifact_attestation_digest_blake3.clone(),
        artifact_attestation_path: artifact_path.display().to_string(),
    }
}

fn bounded_stderr_summary(stderr: &[u8]) -> String {
    let take_len = stderr.len().min(MAX_REMOTE_STDIO_STDERR_BYTES);
    let mut summary = String::from_utf8_lossy(&stderr[..take_len]).to_string();
    if stderr.len() > MAX_REMOTE_STDIO_STDERR_BYTES {
        summary.push_str(STDERR_TRUNCATION_MARKER);
    }
    summary
}

pub fn plan_session_lease(
    session_id: &str,
    uploaded_refs: &[String],
    output_refs: &[String],
) -> Result<RemoteSessionLeasePlan, String> {
    if uploaded_refs.len().saturating_add(output_refs.len()) > MAX_REMOTE_STATUS_ITEMS {
        return Err(format!("session-lease-ref-count-exceeds-{MAX_REMOTE_STATUS_ITEMS}"));
    }
    let mut leased_refs = uploaded_refs.to_vec();
    leased_refs.extend_from_slice(output_refs);
    leased_refs.sort();
    leased_refs.dedup();
    Ok(RemoteSessionLeasePlan {
        session_id: session_id.to_string(),
        leased_refs,
        release_when_done: true,
    })
}

pub fn run_loopback_remote_session(
    builder: &RemoteLoopbackBuilder,
    ticket: &mut RemoteTicket,
    client: &RemoteLoopbackClient,
) -> Result<RemoteLoopbackSessionReport, String> {
    validate_loopback_participants(builder, client)?;
    let accepted = expect_accepted_hello(&client.hello, builder)?;
    expect_authorized_ticket(ticket, &client.auth, fixture_service_auth_facts(ticket))?;
    validate_concrete_request(&client.request, ticket)?;
    validate_input_manifest(&client.input_manifest, &client.request)?;
    let missing = derive_missing_inputs(&client.input_manifest.input_refs, &builder.present_input_refs)?;
    validate_missing_uploads(
        &missing,
        &client.uploaded_input_refs,
        client.request.upload_bytes,
        ticket.max_upload_bytes,
    )?;
    let output_digest = remote_loopback_output_digest(&client.request, &client.input_manifest.input_refs)?;
    let trusted_key = expect_output_trust(builder, client)?;
    redeem_after_queue(ticket, true)?;
    let transfer = plan_output_transfer(
        client.transfer_capabilities,
        builder.transfer_capabilities,
        REMOTE_LOOPBACK_OUTPUT_BYTES,
        &trusted_key,
    )?;
    let lease_plan =
        plan_session_lease(&client.session_id, &client.uploaded_input_refs, std::slice::from_ref(&output_digest))?;
    let session_receipt = RemoteLoopbackSessionReport {
        schema: "mantle-remote-loopback-session-v1".to_string(),
        binding: RemoteTransportBinding::Loopback,
        session_id: client.session_id.clone(),
        endpoint_id: builder.endpoint_id.clone(),
        accepted_capabilities: accepted.accepted_capabilities,
        missing_input_refs: missing,
        uploaded_input_refs: client.uploaded_input_refs.clone(),
        output_digest_blake3: output_digest,
        transfer,
        lease_plan,
        ticket_uses_remaining: ticket.uses_remaining,
        phases: vec![
            RemoteProtocolPhase::AwaitAuth,
            RemoteProtocolPhase::AwaitAuthOk,
            RemoteProtocolPhase::AwaitBuildRequest,
            RemoteProtocolPhase::AwaitInputManifest,
            RemoteProtocolPhase::AwaitMissingInputs,
            RemoteProtocolPhase::AwaitInputUpload,
            RemoteProtocolPhase::AwaitQueueAdmission,
            RemoteProtocolPhase::Queued,
            RemoteProtocolPhase::Building,
            RemoteProtocolPhase::AwaitOutputTransfer,
            RemoteProtocolPhase::Done,
        ],
        non_claims: vec![REMOTE_SESSION_NON_CLAIM.to_string()],
    };
    debug_assert_eq!(session_receipt.endpoint_id, builder.endpoint_id);
    debug_assert_eq!(session_receipt.ticket_uses_remaining, ticket.uses_remaining);
    Ok(session_receipt)
}

fn frame_payload_len(encoded: &[u8]) -> Result<usize, String> {
    if encoded.len() < REMOTE_FRAME_HEADER_BYTES {
        return Err("remote-frame-header-incomplete".to_string());
    }
    let mut header = [0_u8; REMOTE_FRAME_HEADER_BYTES];
    header.copy_from_slice(&encoded[..REMOTE_FRAME_HEADER_BYTES]);
    usize::try_from(u32::from_be_bytes(header)).map_err(|_| "remote-frame-length-conversion-failed".to_string())
}

fn delta_transfer_report(output_size_bytes: u64, verified_builder_key: &str) -> RemoteTransferReport {
    let reused_bytes = output_size_bytes.saturating_mul(DELTA_REUSE_PERCENT) / PERCENT_DENOMINATOR;
    let transferred_bytes = output_size_bytes.saturating_sub(reused_bytes);
    RemoteTransferReport {
        mode: RemoteTransferMode::Delta,
        mode_label: RemoteTransferMode::Delta.as_str().to_string(),
        transferred_bytes,
        reused_bytes,
        fallback_reason: None,
        verified_builder_key: verified_builder_key.to_string(),
    }
}

fn full_transfer_report(
    output_size_bytes: u64,
    verified_builder_key: &str,
    fallback_reason: Option<String>,
) -> RemoteTransferReport {
    RemoteTransferReport {
        mode: RemoteTransferMode::Full,
        mode_label: RemoteTransferMode::Full.as_str().to_string(),
        transferred_bytes: output_size_bytes,
        reused_bytes: 0,
        fallback_reason,
        verified_builder_key: verified_builder_key.to_string(),
    }
}

pub fn streaming_transfer_report_from_runtime(
    runtime: &crate::remote_transfer::RemoteTransferShellReport,
    verified_builder_key: &str,
) -> Result<RemoteTransferReport, String> {
    production_transfer_report_from_runtime(runtime, verified_builder_key, RemoteTransferMode::Streaming, None)
}

fn validate_loopback_participants(
    builder: &RemoteLoopbackBuilder,
    client: &RemoteLoopbackClient,
) -> Result<(), String> {
    if builder.endpoint_id.is_empty() || client.session_id.is_empty() {
        return Err("remote-loopback-identity-empty".to_string());
    }
    if builder.store_prefix != client.request.store_prefix {
        return Err("remote-loopback-store-prefix-mismatch".to_string());
    }
    Ok(())
}

fn expect_accepted_hello(hello: &RemoteHello, builder: &RemoteLoopbackBuilder) -> Result<AcceptedHello, String> {
    match validate_hello(hello, &builder.endpoint_id, &builder.supported_capabilities) {
        ProtocolDecision::Proceed(accepted) => Ok(accepted),
        ProtocolDecision::Reject(reason) => Err(reason),
    }
}

fn expect_authorized_ticket(
    ticket: &RemoteTicket,
    auth: &TicketAuthRequest,
    service_facts: RemoteTicketAuthFacts<'_>,
) -> Result<(), String> {
    match authorize_ticket(ticket, auth, service_facts) {
        TicketDecision::Authorized => Ok(()),
        TicketDecision::Reject(reason) => Err(reason),
    }
}

fn expect_output_trust(builder: &RemoteLoopbackBuilder, client: &RemoteLoopbackClient) -> Result<String, String> {
    match decide_output_trust(
        &builder.signing_key_id,
        &client.trusted_output_keys,
        builder.store_prefix == client.request.store_prefix,
    ) {
        OutputTrustDecision::Accept { key_id, .. } => Ok(key_id),
        OutputTrustDecision::Reject(reason) => Err(reason),
    }
}

fn remote_loopback_output_digest(request: &ConcreteBuildRequest, input_refs: &[String]) -> Result<String, String> {
    let plan = plan_remote_executable_request(request)?;
    let executor = RemoteFixtureExecutor;
    let input_upload = RemoteInputUpload {
        request_id: request.request_id.clone(),
        refs: input_refs.to_vec(),
        byte_count: request.upload_bytes,
        artifacts: Vec::new(),
        streamed: false,
    };
    let execution = executor.execute(request, &plan, &input_upload)?;
    validate_remote_execution_outcome(&execution, request, &plan)?;
    Ok(execution.output_digest_blake3)
}

fn take_hello<'a>(
    frames: &mut std::slice::Iter<'a, RemoteFrame>,
    phase: &mut RemoteProtocolPhase,
) -> Result<&'a RemoteHello, String> {
    let frame = frames.next().ok_or_else(|| "missing-hello-frame".to_string())?;
    *phase = validate_remote_transition(*phase, RemoteFrameDirection::ClientToBuilder, frame)?;
    match frame {
        RemoteFrame::Hello { hello } => Ok(hello),
        _ => Err("expected-hello-frame".to_string()),
    }
}

fn fixture_service_auth_facts(ticket: &RemoteTicket) -> RemoteTicketAuthFacts<'_> {
    RemoteTicketAuthFacts {
        server_now_unix_s: ticket.created_unix_s.saturating_add(1),
        authenticated_client_endpoint: ticket.bound_client_endpoint.as_deref(),
    }
}

fn take_auth(
    frames: &mut std::slice::Iter<'_, RemoteFrame>,
    phase: &mut RemoteProtocolPhase,
    ticket: &RemoteTicket,
    service_facts: RemoteTicketAuthFacts<'_>,
) -> Result<(), String> {
    let frame = frames.next().ok_or_else(|| "missing-auth-ticket-frame".to_string())?;
    *phase = validate_remote_transition(*phase, RemoteFrameDirection::ClientToBuilder, frame)?;
    let RemoteFrame::AuthTicket { auth } = frame else {
        return Err("expected-auth-ticket-frame".to_string());
    };
    expect_authorized_ticket(ticket, auth, service_facts)
}

fn take_build_request<'a>(
    frames: &mut std::slice::Iter<'a, RemoteFrame>,
    phase: &mut RemoteProtocolPhase,
    ticket: &RemoteTicket,
) -> Result<&'a ConcreteBuildRequest, String> {
    let frame = frames.next().ok_or_else(|| "missing-build-request-frame".to_string())?;
    *phase = validate_remote_transition(*phase, RemoteFrameDirection::ClientToBuilder, frame)?;
    let RemoteFrame::BuildRequest { request } = frame else {
        return Err("expected-build-request-frame".to_string());
    };
    validate_concrete_request(request, ticket)?;
    Ok(request)
}

fn take_input_manifest<'a>(
    frames: &mut std::slice::Iter<'a, RemoteFrame>,
    phase: &mut RemoteProtocolPhase,
    request: &ConcreteBuildRequest,
) -> Result<&'a RemoteInputManifest, String> {
    let frame = frames.next().ok_or_else(|| "missing-input-manifest-frame".to_string())?;
    *phase = validate_remote_transition(*phase, RemoteFrameDirection::ClientToBuilder, frame)?;
    let RemoteFrame::InputManifest { manifest } = frame else {
        return Err("expected-input-manifest-frame".to_string());
    };
    validate_input_manifest(manifest, request)?;
    Ok(manifest)
}

fn take_input_upload(
    frames: &mut std::slice::Iter<'_, RemoteFrame>,
    phase: &mut RemoteProtocolPhase,
    request: &ConcreteBuildRequest,
    missing: &[String],
    ticket: &RemoteTicket,
) -> Result<RemoteInputUpload, String> {
    let frame = frames.next().ok_or_else(|| "missing-input-upload-frame".to_string())?;
    *phase = validate_remote_transition(*phase, RemoteFrameDirection::ClientToBuilder, frame)?;
    let RemoteFrame::InputUpload { upload } = frame else {
        return Err("expected-input-upload-frame".to_string());
    };
    if upload.request_id != request.request_id {
        return Err("input-upload-request-id-mismatch".to_string());
    }
    validate_missing_uploads(missing, &upload.refs, upload.byte_count, ticket.max_upload_bytes)?;
    validate_remote_input_upload_artifacts(
        &request.request_id,
        &upload.refs,
        &request.source_input_refs,
        &upload.artifacts,
    )?;
    Ok(upload.clone())
}

fn reject_extra_client_frames(extra: Option<&RemoteFrame>) -> Result<(), String> {
    if extra.is_some() {
        return Err("unexpected-extra-client-frame".to_string());
    }
    Ok(())
}

fn request_ticket_id_from_frames(frames: &[RemoteFrame]) -> Result<&str, String> {
    for frame in frames {
        if let RemoteFrame::AuthTicket { auth } = frame {
            return Ok(&auth.ticket_id);
        }
    }
    Err("missing-auth-ticket-frame".to_string())
}

struct BuildResponseFramesInput<'a> {
    builder: &'a RemoteLoopbackBuilder,
    request: &'a ConcreteBuildRequest,
    input_upload: &'a RemoteInputUpload,
    client_transfer: RemoteTransferCapabilities,
    phase: RemoteProtocolPhase,
    response_frames: Vec<RemoteFrame>,
    missing: Vec<String>,
    ticket_uses_remaining: u32,
    executor: &'a dyn RemoteBuildExecutor,
}

fn build_response_frames(input: BuildResponseFramesInput<'_>) -> Result<RemoteBuilderFrameResponse, String> {
    let BuildResponseFramesInput {
        builder,
        request,
        input_upload,
        client_transfer,
        mut phase,
        mut response_frames,
        missing,
        ticket_uses_remaining,
        executor,
    } = input;
    let plan = plan_remote_executable_request(request)?;
    push_builder_frame(&mut response_frames, &mut phase, RemoteFrame::BuildQueued {
        request_id: request.request_id.clone(),
        session_id: request.request_id.clone(),
    })?;
    push_builder_frame(&mut response_frames, &mut phase, RemoteFrame::BuildStarted {
        request_id: request.request_id.clone(),
    })?;
    let execution = executor.execute(request, &plan, input_upload)?;
    validate_remote_execution_outcome(&execution, request, &plan)?;
    validate_remote_execution_output_pathinfos(&execution.outputs, &builder.signing_key_id, &request.store_prefix)?;
    let outputs = sign_remote_execution_outputs(&execution.outputs, &builder.signing_key_id)?;
    push_builder_frame(&mut response_frames, &mut phase, RemoteFrame::BuildFinished {
        result: RemoteBuildFinished {
            request_id: request.request_id.clone(),
            output_digest_blake3: execution.output_digest_blake3.clone(),
            builder_signing_key_id: builder.signing_key_id.clone(),
            store_prefix: request.store_prefix.clone(),
            outputs: outputs.clone(),
        },
    })?;
    let transfer_artifacts = plan_output_transfer_artifacts(&request.request_id, &outputs, &execution.outputs)?;
    for artifact in &transfer_artifacts {
        push_builder_frame(&mut response_frames, &mut phase, RemoteFrame::OutputTransferArtifact {
            artifact: artifact.clone(),
        })?;
    }
    let transfer = plan_output_transfer(
        client_transfer,
        builder.transfer_capabilities,
        execution.output_size_bytes,
        &builder.signing_key_id,
    )?;
    push_builder_frame(&mut response_frames, &mut phase, RemoteFrame::OutputTransferDone {
        report: transfer.clone(),
    })?;
    push_builder_frame(&mut response_frames, &mut phase, RemoteFrame::Done {
        request_id: request.request_id.clone(),
    })?;
    let response = RemoteBuilderFrameResponse {
        response_frames,
        missing_input_refs: missing,
        execution_plan_digest_blake3: execution.plan_digest_blake3,
        output_digest_blake3: execution.output_digest_blake3,
        outputs,
        transfer_artifacts,
        transfer,
        ticket_uses_remaining,
    };
    debug_assert!(is_blake3_hex_digest(&response.output_digest_blake3));
    debug_assert!(!response.response_frames.is_empty());
    Ok(response)
}

struct RemoteOutputImportFrames<'a> {
    result: &'a RemoteBuildFinished,
    transfer: &'a RemoteTransferReport,
    artifacts: Vec<RemoteOutputTransferArtifact>,
}

fn extract_output_import_frames<'a>(
    frames: &'a [RemoteFrame],
    expected_request_id: &str,
) -> Result<RemoteOutputImportFrames<'a>, String> {
    let mut result = None;
    let mut transfer = None;
    let mut artifacts = Vec::with_capacity(MAX_REMOTE_TRANSFER_ARTIFACTS);
    let mut is_done_seen = false;
    for frame in frames {
        match frame {
            RemoteFrame::BuildFinished { result: finished } if result.replace(finished).is_some() => {
                return Err("duplicate-build-finished-frame".to_string());
            }
            RemoteFrame::OutputTransferArtifact { artifact } => {
                if result.is_none() {
                    return Err("remote-output-transfer-artifact-before-build-result".to_string());
                }
                if transfer.is_some() || is_done_seen {
                    return Err("remote-output-transfer-artifact-after-transfer-done".to_string());
                }
                if artifact.request_id != expected_request_id {
                    return Err("remote-output-transfer-artifact-request-id-mismatch".to_string());
                }
                if artifacts.len() >= MAX_REMOTE_TRANSFER_ARTIFACTS {
                    return Err(format!(
                        "remote-output-transfer-artifact-count-exceeds-{MAX_REMOTE_TRANSFER_ARTIFACTS}"
                    ));
                }
                artifacts.push(artifact.clone());
            }
            RemoteFrame::OutputTransferDone { report } => {
                if result.is_none() {
                    return Err("remote-output-transfer-before-build-result".to_string());
                }
                if transfer.replace(report).is_some() {
                    return Err("duplicate-output-transfer-frame".to_string());
                }
            }
            RemoteFrame::Done { request_id } => {
                if request_id != expected_request_id {
                    return Err("remote-done-request-id-mismatch".to_string());
                }
                if transfer.is_none() {
                    return Err("remote-done-before-output-transfer".to_string());
                }
                if is_done_seen {
                    return Err("duplicate-done-frame".to_string());
                }
                is_done_seen = true;
            }
            RemoteFrame::Error { .. } => return Err("remote-builder-error-frame".to_string()),
            _ => {}
        }
    }
    if !is_done_seen {
        return Err("missing-done-frame".to_string());
    }
    let result = result.ok_or_else(|| "missing-build-finished-frame".to_string())?;
    let transfer = transfer.ok_or_else(|| "missing-output-transfer-frame".to_string())?;
    validate_remote_output_transfer_artifacts(expected_request_id, &result.outputs, &artifacts)?;
    let output_frames = RemoteOutputImportFrames {
        result,
        transfer,
        artifacts,
    };
    debug_assert_eq!(output_frames.result.request_id, expected_request_id);
    debug_assert!(output_frames.artifacts.len() <= MAX_REMOTE_TRANSFER_ARTIFACTS);
    Ok(output_frames)
}

fn validate_remote_produced_outputs(
    request: &ConcreteBuildRequest,
    result: &RemoteBuildFinished,
) -> Result<(), String> {
    if result.outputs.len() != request.expected_outputs.len() {
        return Err("remote-output-metadata-count-mismatch".to_string());
    }
    if remote_produced_outputs_content_digest(&result.outputs) != result.output_digest_blake3 {
        return Err("remote-output-metadata-digest-mismatch".to_string());
    }
    let expected = expected_output_path_map(&request.expected_outputs);
    let mut seen = BTreeSet::new();
    for output in &result.outputs {
        if !seen.insert(output.name.as_str()) {
            return Err("remote-output-metadata-name-duplicate".to_string());
        }
        validate_output_path_against_expected(
            output.name.as_str(),
            output.logical_path.as_str(),
            &expected,
            &request.store_prefix,
            "remote-output-metadata-identity-mismatch",
        )?;
        if output.path_info_signing_key_id != result.builder_signing_key_id {
            return Err("remote-output-pathinfo-key-mismatch".to_string());
        }
        if !is_blake3_hex_digest(&output.content_digest_blake3)
            || !is_blake3_hex_digest(&output.artifact_attestation_digest_blake3)
        {
            return Err("remote-output-metadata-digest-invalid".to_string());
        }
        validate_remote_produced_output_nar_summary(output)?;
    }
    debug_assert_eq!(seen.len(), result.outputs.len());
    debug_assert_eq!(result.outputs.len(), request.expected_outputs.len());
    Ok(())
}

fn validate_remote_produced_output_nar_summary(output: &RemoteProducedOutput) -> Result<(), String> {
    match (&output.nar_payload_digest_blake3, output.nar_payload_size_bytes) {
        (Some(digest), Some(_)) => {
            if !is_blake3_hex_digest(digest) {
                return Err("remote-output-nar-digest-invalid".to_string());
            }
            if output.path_info.is_none() {
                return Err("remote-output-nar-pathinfo-missing".to_string());
            }
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err("remote-output-nar-summary-incomplete".to_string()),
    }
}

fn validate_transfer_report(report: &RemoteTransferReport) -> Result<(), String> {
    if report.verified_builder_key.is_empty() {
        return Err("remote-transfer-builder-key-empty".to_string());
    }
    if report.mode_label != report.mode.as_str() {
        return Err("remote-transfer-mode-label-mismatch".to_string());
    }
    match report.mode {
        RemoteTransferMode::Delta => {
            if report.fallback_reason.is_some() {
                return Err("remote-transfer-delta-has-fallback-reason".to_string());
            }
        }
        RemoteTransferMode::Full => {
            if report.reused_bytes != 0 {
                return Err("remote-transfer-full-reused-bytes-nonzero".to_string());
            }
        }
        RemoteTransferMode::Streaming => {
            if report.fallback_reason.is_some() {
                return Err("remote-transfer-streaming-has-fallback-reason".to_string());
            }
        }
    }
    debug_assert_eq!(report.mode_label, report.mode.as_str());
    debug_assert!(!report.verified_builder_key.is_empty());
    Ok(())
}

fn is_blake3_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn push_builder_frame(
    response_frames: &mut Vec<RemoteFrame>,
    phase: &mut RemoteProtocolPhase,
    frame: RemoteFrame,
) -> Result<(), String> {
    *phase = validate_remote_transition(*phase, RemoteFrameDirection::BuilderToClient, &frame)?;
    response_frames.push(frame);
    Ok(())
}

fn read_bounded_stdio_input(mut reader: impl Read) -> Result<zeroize::Zeroizing<Vec<u8>>, String> {
    let mut input = zeroize::Zeroizing::new(Vec::new());
    let read_limit_bytes =
        u64::try_from(MAX_REMOTE_STDIO_INPUT_BYTES).map_err(|_| "stdio-input-limit-conversion-failed".to_string())?;
    let read_limit_with_probe_bytes =
        read_limit_bytes.checked_add(1).ok_or_else(|| "stdio-input-limit-overflow".to_string())?;
    reader
        .by_ref()
        .take(read_limit_with_probe_bytes)
        .read_to_end(&mut input)
        .map_err(|err| format!("reading stdio remote input: {err}"))?;
    if input.len() > MAX_REMOTE_STDIO_INPUT_BYTES {
        return Err(format!("remote-stdio-input-exceeds-{MAX_REMOTE_STDIO_INPUT_BYTES}"));
    }
    Ok(input)
}

pub fn revoke_ticket(state: &mut RemoteTicketState, id: &str) -> Result<RemoteTicketView, RunError> {
    let ticket = state.tickets.get_mut(id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
    ticket.revoked = true;
    Ok(redacted_ticket_view(ticket))
}

fn coordinator_state_path(state_dir: &Path) -> PathBuf {
    state_dir.join(COORDINATOR_STATE_FILE)
}

#[derive(Debug)]
pub struct RemoteCoordinatorMutationGuard {
    _file: File,
}

pub fn acquire_remote_coordinator_mutation_guard(state_dir: &Path) -> Result<RemoteCoordinatorMutationGuard, String> {
    fs::create_dir_all(state_dir).map_err(|err| format!("creating coordinator state dir: {err}"))?;
    let path = state_dir.join(COORDINATOR_MUTATION_LOCK_FILE);
    let file = open_coordinator_lock_no_follow(&path)?;
    FileExt::try_lock_exclusive(&file).map_err(|err| format!("remote-coordinator-mutation-lock-busy: {err}"))?;
    let metadata = file.metadata().map_err(|err| format!("reading coordinator mutation lock metadata: {err}"))?;
    if !metadata.is_file() {
        return Err("remote-coordinator-mutation-lock-not-regular".to_string());
    }
    assert!(metadata.is_file());
    assert_eq!(path.parent(), Some(state_dir));
    Ok(RemoteCoordinatorMutationGuard { _file: file })
}

#[cfg(unix)]
fn open_coordinator_lock_no_follow(path: &Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .mode(COORDINATOR_LOCK_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|err| format!("opening coordinator mutation lock without symlink following: {err}"))
}

#[cfg(not(unix))]
fn open_coordinator_lock_no_follow(path: &Path) -> Result<File, String> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("remote-coordinator-mutation-lock-symlink-rejected".to_string());
    }
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .map_err(|err| format!("opening coordinator mutation lock: {err}"))
}

pub fn load_coordinator_state(state_dir: &Path) -> Result<RemoteCoordinatorState, RunError> {
    let path = coordinator_state_path(state_dir);
    if !path.exists() {
        let state = RemoteCoordinatorState {
            state_dir: Some(state_dir.to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        debug_assert_eq!(state.state_dir.as_deref(), Some(state_dir));
        debug_assert!(state.jobs.is_empty());
        return Ok(state);
    }
    let bytes = fs::read(&path)
        .map_err(|err| RunError::Internal(format!("reading coordinator state {}: {err}", path.display())))?;
    let legacy_log_migration = legacy_log_migration_summary(&bytes, &path)?;
    let mut state: RemoteCoordinatorState = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing coordinator state {}: {err}", path.display())))?;
    state.state_dir = Some(state_dir.to_path_buf());
    let mut is_migrated = migrate_legacy_coordinator_state(&mut state);
    if let Some(summary) = legacy_log_migration {
        state.legacy_log_migration = Some(summary);
        is_migrated = true;
    }
    if reconcile_coordinator_resource_leases(&mut state)? {
        is_migrated = true;
    }
    if reconcile_coordinator_log_summaries(state_dir, &mut state)? {
        is_migrated = true;
    }
    validate_external_batch_state(&state).map_err(RunError::Internal)?;
    if is_migrated {
        save_coordinator_state(state_dir, &state)?;
    }
    debug_assert_eq!(state.state_dir.as_deref(), Some(state_dir));
    debug_assert!(state.state_dir.is_some());
    Ok(state)
}

fn reconcile_coordinator_resource_leases(state: &mut RemoteCoordinatorState) -> Result<bool, RunError> {
    let mut is_changed = false;
    let lease_ids = state.resource_leases.keys().cloned().collect::<Vec<_>>();
    for lease_id in lease_ids {
        let lease = state
            .resource_leases
            .get(&lease_id)
            .cloned()
            .ok_or_else(|| RunError::Internal("resource lease disappeared during recovery".to_string()))?;
        let current = state.jobs.get(&lease.scope.job_id).and_then(|job| job.current_attempt.as_ref());
        let current_scope = current.and_then(|attempt| {
            state.jobs.get(&lease.scope.job_id).and_then(|job| {
                let worker_endpoint_id = job.assigned_worker_endpoint_id.as_ref()?;
                let worker_generation = state.workers.get(worker_endpoint_id)?.worker_generation;
                Some(RemoteResourceLeaseScope {
                    worker_endpoint_id: worker_endpoint_id.clone(),
                    worker_generation,
                    job_id: job.job_id.clone(),
                    attempt_id: attempt.attempt_id.clone(),
                    fence_generation: attempt.fence_generation,
                })
            })
        });
        let recovery =
            plan_remote_resource_lease_recovery(&lease, current_scope.as_ref(), current.map(|attempt| attempt.phase))
                .map_err(|reason| RunError::Internal(reason.as_str().to_string()))?;
        if recovery.disposition == RemoteResourceRecoveryDisposition::Release {
            state.resource_leases.remove(&lease_id);
            if let Some(job) = state.jobs.get_mut(&lease.scope.job_id)
                && job.resource_lease_id_blake3.as_deref() == Some(lease_id.as_str())
            {
                job.resource_lease_id_blake3 = None;
            }
            is_changed = true;
        }
    }
    let missing_live_leases = state
        .jobs
        .values()
        .filter(|job| job.resource_requirements.is_some())
        .filter(|job| job.current_attempt.as_ref().is_some_and(|attempt| attempt.phase.is_live()))
        .filter(|job| job.resource_lease_id_blake3.is_none())
        .map(|job| job.job_id.clone())
        .collect::<Vec<_>>();
    for job_id in missing_live_leases {
        if let Some(job) = state.jobs.get_mut(&job_id) {
            fail_closed_coordinator_job(job, RemoteAttemptReasonCode::DurableStateInvalid);
            is_changed = true;
        }
    }
    validate_coordinator_resource_state(state).map_err(RunError::Internal)?;
    debug_assert!(missing_live_leases_are_resolved(state));
    debug_assert!(state.resource_leases.len() <= crunch_build::distributed::MAX_REMOTE_RESOURCE_LEASES);
    Ok(is_changed)
}

fn missing_live_leases_are_resolved(state: &RemoteCoordinatorState) -> bool {
    state.jobs.values().all(|job| {
        job.resource_requirements.is_none()
            || !job.current_attempt.as_ref().is_some_and(|attempt| attempt.phase.is_live())
            || job.resource_lease_id_blake3.is_some()
    })
}

fn legacy_log_migration_summary(
    bytes: &[u8],
    path: &Path,
) -> Result<Option<RemoteLegacyLogMigrationSummary>, RunError> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| RunError::Internal(format!("parsing coordinator state {}: {error}", path.display())))?;
    let Some(logs) = value.get("logs").and_then(serde_json::Value::as_object) else {
        return Ok(None);
    };
    if logs.is_empty() {
        return Ok(None);
    }
    let mut chunk_count = 0_u64;
    let mut payload_bytes = 0_u64;
    let mut is_count_saturated = false;
    for chunks in logs.values().filter_map(serde_json::Value::as_array) {
        let observed_chunk_count = match u64::try_from(chunks.len()) {
            Ok(count) => count,
            Err(_) => {
                is_count_saturated = true;
                LEGACY_LOG_MIGRATION_COUNT_MAX_U64
            }
        };
        chunk_count = bounded_legacy_migration_add(
            BoundedLegacyMigrationAddInput {
                current: chunk_count,
                observed: observed_chunk_count,
                maximum: LEGACY_LOG_MIGRATION_COUNT_MAX_U64,
            },
            &mut is_count_saturated,
        );
        for chunk in chunks {
            let observed_bytes = chunk
                .get("bytes")
                .and_then(serde_json::Value::as_str)
                .and_then(|text| u64::try_from(text.len()).ok())
                .unwrap_or(0);
            payload_bytes = bounded_legacy_migration_add(
                BoundedLegacyMigrationAddInput {
                    current: payload_bytes,
                    observed: observed_bytes,
                    maximum: LEGACY_LOG_MIGRATION_PAYLOAD_BYTES_MAX,
                },
                &mut is_count_saturated,
            );
        }
    }
    let legacy_job_count = match u32::try_from(logs.len()) {
        Ok(count) => count,
        Err(_) => {
            is_count_saturated = true;
            LEGACY_LOG_MIGRATION_COUNT_MAX_U32
        }
    };
    let legacy_chunk_count = match u32::try_from(chunk_count) {
        Ok(count) => count,
        Err(_) => {
            is_count_saturated = true;
            LEGACY_LOG_MIGRATION_COUNT_MAX_U32
        }
    };
    let summary = RemoteLegacyLogMigrationSummary {
        legacy_job_count,
        legacy_chunk_count,
        legacy_payload_bytes: payload_bytes,
        count_saturated: is_count_saturated,
        classification: LEGACY_LOG_MIGRATION_CLASSIFICATION.to_string(),
        non_claim: LEGACY_LOG_MIGRATION_NON_CLAIM.to_string(),
    };
    debug_assert!(!summary.classification.is_empty());
    debug_assert!(!summary.non_claim.is_empty());
    Ok(Some(summary))
}

struct BoundedLegacyMigrationAddInput {
    current: u64,
    observed: u64,
    maximum: u64,
}

fn bounded_legacy_migration_add(input: BoundedLegacyMigrationAddInput, is_saturated: &mut bool) -> u64 {
    debug_assert!(input.current <= input.maximum);
    debug_assert!(input.maximum > 0);
    let Some(next) = input.current.checked_add(input.observed) else {
        *is_saturated = true;
        return input.maximum;
    };
    if next > input.maximum {
        *is_saturated = true;
        return input.maximum;
    }
    next
}

fn reconcile_coordinator_log_summaries(state_dir: &Path, state: &mut RemoteCoordinatorState) -> Result<bool, RunError> {
    let mut is_changed = false;
    for job in state.jobs.values_mut() {
        let Some(attempt) = job.current_attempt.as_ref() else {
            continue;
        };
        let retention_policy = job.immutable_log.as_ref().map(|summary| summary.retention_policy).unwrap_or_default();
        let policy = immutable_attempt_log_policy(retention_policy)
            .map_err(|error| RunError::Internal(format!("reconciling attempt-log policy: {error}")))?;
        let scope = RemoteAttemptLogScope {
            job_id: attempt.job_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            fence_generation: attempt.fence_generation,
        };
        let manifest = crate::remote_attempt_log_store::load_remote_attempt_log_manifest(state_dir, &scope, policy)
            .map_err(|error| RunError::Internal(format!("reconciling immutable attempt log: {error}")))?;
        let observed = remote_attempt_log_control_summary(&manifest, retention_policy);
        if job.immutable_log.as_ref() != Some(&observed) {
            job.immutable_log = Some(observed);
            is_changed = true;
        }
    }
    debug_assert!(state.jobs.len() <= MAX_REMOTE_STATUS_ITEMS);
    debug_assert!(state.jobs.values().all(|job| job.current_attempt.is_none() || job.immutable_log.is_some()));
    Ok(is_changed)
}

fn migrate_legacy_coordinator_state(state: &mut RemoteCoordinatorState) -> bool {
    let mut is_migrated = false;
    for job in state.jobs.values_mut() {
        if coordinator_job_attempt_is_valid(job) {
            continue;
        }
        let reason_code = if job.current_attempt.is_some() {
            RemoteAttemptReasonCode::DurableStateInvalid
        } else {
            RemoteAttemptReasonCode::LegacyStateRejected
        };
        fail_closed_coordinator_job(job, reason_code);
        is_migrated = true;
    }
    if is_migrated {
        rebuild_live_output_claims(state);
    }
    debug_assert!(state.jobs.values().all(coordinator_job_attempt_is_valid));
    debug_assert!(state.live_output_claims.len() <= MAX_REMOTE_STATUS_ITEMS);
    is_migrated
}

fn coordinator_job_attempt_is_valid(job: &RemoteCoordinatorJobSummary) -> bool {
    let Some(attempt) = job.current_attempt.as_ref() else {
        return job.phase == RemoteCoordinatorJobPhase::Lost
            && matches!(
                job.last_attempt_reason_code,
                Some(RemoteAttemptReasonCode::LegacyStateRejected | RemoteAttemptReasonCode::DurableStateInvalid)
            );
    };
    coordinator_attempt_identity_is_valid(job, attempt) && coordinator_attempt_projection_matches(job, attempt)
}

fn coordinator_attempt_identity_is_valid(job: &RemoteCoordinatorJobSummary, attempt: &RemoteAttemptState) -> bool {
    if attempt.job_id != job.job_id {
        return false;
    }
    if RemoteJobId::new(attempt.job_id.as_str().to_string()).is_err() {
        return false;
    }
    if RemoteAttemptId::new(attempt.attempt_id.as_str().to_string()).is_err() {
        return false;
    }
    if RemoteAssignmentNonce::new(attempt.assignment_nonce.as_str().to_string()).is_err() {
        return false;
    }
    if attempt.fence_generation.get() == 0 {
        return false;
    }
    if attempt.attempts_started == 0 {
        return false;
    }
    if attempt.attempts_started > crunch_build::distributed::MAX_REMOTE_ATTEMPTS {
        return false;
    }
    if !coordinator_attempt_derivation_is_valid(job, attempt) {
        return false;
    }
    if !coordinator_attempt_result_is_valid(attempt) {
        return false;
    }
    if attempt.started_unix_s >= attempt.deadline_unix_s {
        return false;
    }
    if attempt.applied_events.len() > crunch_build::distributed::MAX_REMOTE_ATTEMPT_EVENTS {
        return false;
    }
    if let Some(heartbeat_unix_s) = attempt.last_heartbeat_unix_s {
        if heartbeat_unix_s < attempt.started_unix_s {
            return false;
        }
        if heartbeat_unix_s > attempt.deadline_unix_s {
            return false;
        }
    }
    for (event_id, digest) in &attempt.applied_events {
        if RemoteEventId::new(event_id.as_str().to_string()).is_err() {
            return false;
        }
        if RemotePayloadDigest::new(digest.as_str().to_string()).is_err() {
            return false;
        }
    }
    debug_assert_eq!(attempt.job_id, job.job_id);
    debug_assert!(attempt.applied_events.len() <= crunch_build::distributed::MAX_REMOTE_ATTEMPT_EVENTS);
    true
}

fn coordinator_attempt_derivation_is_valid(job: &RemoteCoordinatorJobSummary, attempt: &RemoteAttemptState) -> bool {
    let Some(worker_endpoint_id) = job.assigned_worker_endpoint_id.as_deref() else {
        return false;
    };
    let Ok(expected_attempt_id) = derive_remote_attempt_id(
        &attempt.job_id,
        &attempt.assignment_nonce,
        attempt.fence_generation,
        worker_endpoint_id,
    ) else {
        return false;
    };
    expected_attempt_id == attempt.attempt_id
}

fn coordinator_attempt_result_is_valid(attempt: &RemoteAttemptState) -> bool {
    match attempt.phase {
        RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed => {
            attempt.result_digest_blake3.as_deref().is_some_and(is_blake3_hex_digest)
        }
        RemoteAttemptPhase::Queued
        | RemoteAttemptPhase::Running
        | RemoteAttemptPhase::Transferring
        | RemoteAttemptPhase::Failed
        | RemoteAttemptPhase::Superseded => attempt.result_digest_blake3.is_none(),
    }
}

fn coordinator_attempt_projection_matches(job: &RemoteCoordinatorJobSummary, attempt: &RemoteAttemptState) -> bool {
    let is_result_available = attempt.phase == RemoteAttemptPhase::Completed;
    let is_output_admitted =
        matches!(attempt.phase, RemoteAttemptPhase::FinishedUndelivered | RemoteAttemptPhase::Completed);
    if job.phase != coordinator_phase_for_attempt(attempt.phase) {
        return false;
    }
    if job.result_available != is_result_available {
        return false;
    }
    if job.output_admission_completed != is_output_admitted {
        return false;
    }
    if job.transfer_checkpoint != attempt.transfer_checkpoint {
        return false;
    }
    if job.transferred_bytes != attempt.transferred_bytes {
        return false;
    }
    debug_assert_eq!(job.phase, coordinator_phase_for_attempt(attempt.phase));
    debug_assert_eq!(job.output_admission_completed, is_output_admitted);
    true
}

fn fail_closed_coordinator_job(job: &mut RemoteCoordinatorJobSummary, reason_code: RemoteAttemptReasonCode) {
    job.assigned_worker_endpoint_id = None;
    job.phase = RemoteCoordinatorJobPhase::Lost;
    job.result_available = false;
    job.immutable_log = None;
    job.current_attempt = None;
    job.transfer_checkpoint = None;
    job.transferred_bytes = 0;
    job.output_admission_completed = false;
    job.last_attempt_reason_code = Some(reason_code);
    job.short_error = Some(reason_code.as_str().to_string());
    job.resource_lease_id_blake3 = None;
    job.resource_fit = None;
    job.locality = None;
    debug_assert_eq!(job.phase, RemoteCoordinatorJobPhase::Lost);
    debug_assert!(job.current_attempt.is_none());
}

fn rebuild_live_output_claims(state: &mut RemoteCoordinatorState) {
    let live_keys = state
        .jobs
        .values()
        .filter(|job| job.phase.is_live())
        .map(|job| job.normalized_build_key.clone())
        .collect::<BTreeSet<_>>();
    state.live_output_claims.retain(|_, owner_key| live_keys.contains(owner_key));
}

pub fn save_coordinator_state(state_dir: &Path, state: &RemoteCoordinatorState) -> Result<(), RunError> {
    let path = coordinator_state_path(state_dir);
    let parent = path
        .parent()
        .ok_or_else(|| RunError::Internal("coordinator state path has no parent".to_string()))?;
    fs::create_dir_all(parent)
        .map_err(|err| RunError::Internal(format!("creating coordinator state dir {}: {err}", parent.display())))?;
    let rendered = serde_json::to_string_pretty(state)
        .map_err(|err| RunError::Internal(format!("serializing coordinator state: {err}")))?;
    let tmp = path.with_extension(TEMP_FILE_EXTENSION);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&tmp)
        .map_err(|err| RunError::Internal(format!("opening coordinator state temp {}: {err}", tmp.display())))?;
    file.write_all(format!("{rendered}\n").as_bytes())
        .map_err(|err| RunError::Internal(format!("writing coordinator state temp {}: {err}", tmp.display())))?;
    file.sync_all()
        .map_err(|err| RunError::Internal(format!("syncing coordinator state temp {}: {err}", tmp.display())))?;
    drop(file);
    fs::rename(&tmp, &path)
        .map_err(|err| RunError::Internal(format!("committing coordinator state {}: {err}", path.display())))?;
    sync_coordinator_state_parent(parent)?;
    debug_assert!(path.exists());
    debug_assert!(!tmp.exists());
    Ok(())
}

#[cfg(unix)]
fn sync_coordinator_state_parent(parent: &Path) -> Result<(), RunError> {
    let directory = fs::File::open(parent)
        .map_err(|err| RunError::Internal(format!("opening coordinator state dir {}: {err}", parent.display())))?;
    directory
        .sync_all()
        .map_err(|err| RunError::Internal(format!("syncing coordinator state dir {}: {err}", parent.display())))
}

#[cfg(not(unix))]
fn sync_coordinator_state_parent(_parent: &Path) -> Result<(), RunError> {
    Ok(())
}

pub fn cmd_remote(
    action: crate::RemoteAction,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::RemoteAction::Ticket { action } => cmd_remote_ticket(action, state_dir, json_output),
        crate::RemoteAction::Status {
            endpoint_id,
            concurrency,
        } => cmd_remote_status(endpoint_id, concurrency, state_dir, json_output),
        crate::RemoteAction::Debug { action } => {
            crate::remote_failure_debug::cmd_remote_failure_debug(action, state_dir, json_output)
        }
        crate::RemoteAction::Serve {
            endpoint_id,
            binding,
            signing_key_id,
            executor,
            present_input_refs,
            execution_state_dir,
            secret_manifest,
            secret_profile,
            secret_provider,
        } => cmd_remote_serve(RemoteServeCommandInput {
            endpoint_id,
            binding,
            fixture_signing_key_id: signing_key_id,
            executor,
            present_input_refs,
            output_dir,
            state_dir,
            execution_state_dir: execution_state_dir.as_deref().unwrap_or(state_dir),
            store_prefix,
            json_output,
            secret_request: crate::remote_service_secrets::RemoteServiceSecretRequest {
                manifest_path: secret_manifest,
                profile: secret_profile,
                provider: secret_provider,
            },
        }),
    }
}

struct RemoteServeExecutorInput<'a> {
    executor: crate::RemoteServeExecutor,
    endpoint_id: &'a str,
    fixture_signing_key_id: String,
    output_dir: &'a Path,
    coordinator_state_dir: &'a Path,
    state_dir: &'a Path,
    store_prefix: &'a str,
}

fn remote_serve_executor(
    input: RemoteServeExecutorInput<'_>,
    keypair: crunch_build::KeyPair,
) -> Result<(String, Option<RemoteLocalBuildExecutor>), RunError> {
    let signing_key_id = keypair.verifying_key.name().to_string();
    let is_default_fixture_key = input.fixture_signing_key_id == "builder-key";
    if !is_default_fixture_key && input.fixture_signing_key_id != signing_key_id {
        return Err(RunError::Internal("remote-builder-signing-key-override-forbidden".to_string()));
    }
    match input.executor {
        crate::RemoteServeExecutor::Fixture => Ok((signing_key_id, None)),
        crate::RemoteServeExecutor::LocalBuild => {
            let trusted_keys = vec![keypair.verifying_key.clone()];
            let local = RemoteLocalBuildExecutor {
                endpoint_id: input.endpoint_id.to_string(),
                coordinator_state_dir: input.coordinator_state_dir.to_path_buf(),
                state_dir: input.state_dir.to_path_buf(),
                output_dir: input.output_dir.to_path_buf(),
                store_prefix: input.store_prefix.to_string(),
                keypair,
                trusted_keys,
                trust_unsigned: false,
                verbose: false,
            };
            Ok((local.signing_key_id(), Some(local)))
        }
    }
}

fn remote_transfer_capabilities_from_labels(labels: &[String]) -> RemoteTransferCapabilities {
    RemoteTransferCapabilities {
        delta: labels.iter().any(|label| label == "delta"),
        full: labels.iter().any(|label| label == "full"),
        streaming: labels.iter().any(|label| label == "streaming"),
        simulate_delta_failure: false,
    }
}

fn production_output_transfer_selection(
    client: RemoteTransferCapabilities,
    builder: RemoteTransferCapabilities,
) -> Result<(RemoteTransferMode, Option<String>), String> {
    if client.streaming && builder.streaming {
        if client.delta && builder.delta {
            return Ok((
                RemoteTransferMode::Full,
                Some(crate::remote_transfer::RemoteTransferFallbackReason::DeltaUnavailable.as_str().to_string()),
            ));
        }
        return Ok((RemoteTransferMode::Streaming, None));
    }
    if client.full && builder.full {
        return Ok((RemoteTransferMode::Full, None));
    }
    Err("no-compatible-production-output-transfer-mode".to_string())
}

fn production_transfer_report_from_runtime(
    runtime: &crate::remote_transfer::RemoteTransferShellReport,
    verified_builder_key: &str,
    actual_mode: RemoteTransferMode,
    fallback_reason: Option<String>,
) -> Result<RemoteTransferReport, String> {
    use crate::remote_transfer::RemoteTransferShellDisposition;
    if runtime.output_admission_claimed {
        return Err("remote-streaming-runtime-admission-overclaim".to_string());
    }
    if !matches!(
        runtime.disposition,
        RemoteTransferShellDisposition::Completed | RemoteTransferShellDisposition::AlreadyPresent
    ) {
        return Err("remote-streaming-runtime-incomplete".to_string());
    }
    if verified_builder_key.is_empty() {
        return Err("remote-streaming-runtime-builder-key-empty".to_string());
    }
    if actual_mode == RemoteTransferMode::Delta {
        return Err("remote-production-delta-runtime-evidence-missing".to_string());
    }
    if actual_mode == RemoteTransferMode::Streaming && fallback_reason.is_some() {
        return Err("remote-production-streaming-fallback-inconsistent".to_string());
    }
    let transfer_receipt = RemoteTransferReport {
        mode: actual_mode,
        mode_label: actual_mode.as_str().to_string(),
        transferred_bytes: runtime.transferred_bytes,
        reused_bytes: runtime.reused_bytes,
        fallback_reason,
        verified_builder_key: verified_builder_key.to_string(),
    };
    debug_assert_eq!(transfer_receipt.mode_label, actual_mode.as_str());
    debug_assert!(!transfer_receipt.verified_builder_key.is_empty());
    Ok(transfer_receipt)
}

fn validate_input_transfer_manifest_frame(
    request: &ConcreteBuildRequest,
    transfer: &RemoteTransferManifestFrame,
    policy: RemoteTransferPolicy,
) -> Result<crunch_build::distributed::CanonicalRemoteTransferManifest, String> {
    if transfer.direction != crate::remote_transfer::RemoteTransferDirection::Upload
        || transfer.actual_mode != RemoteTransferMode::Streaming
        || transfer.fallback_reason.is_some()
    {
        return Err("remote-input-transfer-manifest-mode-invalid".to_string());
    }
    let canonical = canonicalize_remote_transfer_manifest(transfer.manifest.clone(), policy)
        .map_err(|reason| reason.as_str().to_string())?;
    if canonical.digest_blake3 != transfer.manifest_digest_blake3 {
        return Err(RemoteTransferReasonCode::ManifestIdentityMismatch.as_str().to_string());
    }
    let attempt = request.production_attempt.as_ref().ok_or_else(|| "remote-production-attempt-missing".to_string())?;
    validate_input_transfer_manifest_binding(request, attempt, &canonical)?;
    let expected_ids = request
        .input_refs
        .iter()
        .map(|input_ref| remote_input_nar_artifact_id(input_ref))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let actual_ids = canonical
        .manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_id.clone())
        .collect::<BTreeSet<_>>();
    if actual_ids != expected_ids
        || canonical
            .manifest
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_kind != RemoteTransferArtifactKind::Nar)
    {
        return Err("remote-input-transfer-manifest-artifact-mismatch".to_string());
    }
    let artifact_count = usize::try_from(canonical.artifact_count)
        .map_err(|_| "remote-input-transfer-artifact-count-overflow".to_string())?;
    assert_eq!(artifact_count, request.input_refs.len());
    assert!(canonical.total_bytes <= policy.total_bytes_max);
    Ok(canonical)
}

fn validate_input_transfer_manifest_binding(
    request: &ConcreteBuildRequest,
    attempt: &RemoteProductionAttemptBinding,
    canonical: &CanonicalRemoteTransferManifest,
) -> Result<(), String> {
    if canonical.manifest.job_id != attempt.job_id {
        return Err("remote-input-transfer-manifest-binding-mismatch".to_string());
    }
    if canonical.manifest.attempt_id != attempt.attempt_id {
        return Err("remote-input-transfer-manifest-binding-mismatch".to_string());
    }
    if canonical.manifest.fence_generation != attempt.fence_generation {
        return Err("remote-input-transfer-manifest-binding-mismatch".to_string());
    }
    if canonical.manifest.store_prefix != request.store_prefix {
        return Err("remote-input-transfer-manifest-binding-mismatch".to_string());
    }
    if canonical.manifest.requested_content_blake3 != remote_input_requested_content_digest(request)? {
        return Err("remote-input-transfer-manifest-binding-mismatch".to_string());
    }
    debug_assert_eq!(canonical.manifest.job_id, attempt.job_id);
    debug_assert_eq!(canonical.manifest.store_prefix, request.store_prefix);
    Ok(())
}

struct RemoteProductionServerContext<'a> {
    builder: &'a RemoteLoopbackBuilder,
    verifier_key: &'a std::sync::Arc<TicketVerifierKey>,
    executor: &'a RemoteLocalBuildExecutor,
    credential_state_dir: &'a Path,
    execution_state_dir: &'a Path,
    authenticated_client_endpoint: Option<String>,
}

struct RemoteProductionServerOpening {
    hello: RemoteHello,
    server_now_unix_s: u64,
    max_upload_bytes: u64,
    request: ConcreteBuildRequest,
    policy: RemoteTransferPolicy,
    missing: Vec<String>,
    accepted_capabilities: Vec<String>,
    is_trace_negotiated: bool,
    trace_context_health: RemoteTraceContextHealth,
}

struct CommittedRemoteTicketAdmission {
    server_now_unix_s: u64,
    max_upload_bytes: u64,
}

fn serve_stdio_remote_production_once(
    reader: &mut impl Read,
    writer: &mut impl Write,
    context: RemoteProductionServerContext<'_>,
) -> Result<(), String> {
    let mut commit = |auth: &TicketAuthRequest, request: &ConcreteBuildRequest| {
        commit_remote_ticket_admission(&context, auth, request)
    };
    serve_stdio_remote_production_once_with_commit(reader, writer, &context, &mut commit)
}

fn serve_stdio_remote_production_once_with_commit(
    reader: &mut impl Read,
    writer: &mut impl Write,
    context: &RemoteProductionServerContext<'_>,
    commit: &mut impl FnMut(&TicketAuthRequest, &ConcreteBuildRequest) -> Result<CommittedRemoteTicketAdmission, String>,
) -> Result<(), String> {
    let mut opening = read_remote_production_opening(reader, context.builder)?;
    let committed = commit(&opening.0, &opening.1.request)?;
    opening.1.server_now_unix_s = committed.server_now_unix_s;
    opening.1.max_upload_bytes = committed.max_upload_bytes;
    write_remote_production_admission(writer, context.builder, &opening.1)?;
    let runtime = tokio::runtime::Runtime::new().map_err(|err| format!("remote production runtime: {err}"))?;
    let uploaded_bytes = receive_remote_production_inputs(reader, writer, context, &opening.1, &runtime)?;
    send_remote_production_result(reader, writer, context, RemoteProductionResultInput {
        opening: &opening.1,
        runtime: &runtime,
        uploaded_bytes,
    })?;
    debug_assert!(!opening.1.request.request_id.is_empty());
    debug_assert!(uploaded_bytes <= MAX_REMOTE_UPLOAD_BYTES);
    Ok(())
}

fn read_remote_production_opening(
    reader: &mut impl Read,
    builder: &RemoteLoopbackBuilder,
) -> Result<(TicketAuthRequest, RemoteProductionServerOpening), String> {
    let hello = match read_expected_remote_frame(reader, RemoteFrameKind::Hello)? {
        RemoteFrame::Hello { hello } => hello,
        _ => return Err("remote-hello-frame-invalid".to_string()),
    };
    let accepted = expect_accepted_hello(&hello, builder)?;
    let is_trace_negotiated = accepted
        .accepted_capabilities
        .iter()
        .any(|capability| capability == REMOTE_TRACE_CONTEXT_CAPABILITY);
    let trace_context_health = read_remote_server_trace_context(reader, is_trace_negotiated)?;
    let auth = match read_expected_remote_frame(reader, RemoteFrameKind::AuthTicket)? {
        RemoteFrame::AuthTicket { auth } => auth,
        _ => return Err("remote-auth-ticket-frame-invalid".to_string()),
    };
    let request = match read_expected_remote_frame(reader, RemoteFrameKind::BuildRequest)? {
        RemoteFrame::BuildRequest { request } => request,
        _ => return Err("remote-build-request-frame-invalid".to_string()),
    };
    let policy = request.transfer_policy.ok_or_else(|| "remote-production-transfer-policy-missing".to_string())?;
    policy.validate().map_err(|reason| reason.as_str().to_string())?;
    let input_manifest = match read_expected_remote_frame(reader, RemoteFrameKind::InputManifest)? {
        RemoteFrame::InputManifest { manifest } => manifest,
        _ => return Err("remote-input-manifest-frame-invalid".to_string()),
    };
    validate_input_manifest(&input_manifest, &request)?;
    let missing = derive_missing_inputs(&input_manifest.input_refs, &builder.present_input_refs)?;
    Ok((auth, RemoteProductionServerOpening {
        hello,
        server_now_unix_s: 0,
        max_upload_bytes: 0,
        request,
        policy,
        missing,
        accepted_capabilities: accepted.accepted_capabilities,
        is_trace_negotiated,
        trace_context_health,
    }))
}

fn write_remote_production_admission(
    writer: &mut impl Write,
    builder: &RemoteLoopbackBuilder,
    opening: &RemoteProductionServerOpening,
) -> Result<(), String> {
    write_remote_control_frame(writer, &RemoteFrame::AuthOk {
        auth: RemoteAuthOk {
            builder_signing_keys: vec![builder.signing_key_id.clone()],
            accepted_capabilities: opening.accepted_capabilities.clone(),
        },
    })?;
    if opening.is_trace_negotiated {
        write_remote_control_frame(writer, &RemoteFrame::TraceContextAck {
            acknowledgement: RemoteTraceContextAck {
                health: opening.trace_context_health.clone(),
            },
        })?;
    }
    write_remote_control_frame(writer, &RemoteFrame::MissingInputs {
        request_id: opening.request.request_id.clone(),
        refs: opening.missing.clone(),
    })?;
    debug_assert!(!opening.trace_context_health.reason_code.is_empty());
    debug_assert!(opening.missing.len() <= MAX_REMOTE_INPUT_REFS);
    Ok(())
}

fn commit_remote_ticket_admission(
    context: &RemoteProductionServerContext<'_>,
    auth: &TicketAuthRequest,
    request: &ConcreteBuildRequest,
) -> Result<CommittedRemoteTicketAdmission, String> {
    commit_remote_ticket_admission_for_state(
        context.credential_state_dir,
        context.verifier_key,
        context.authenticated_client_endpoint.as_deref(),
        auth,
        request,
    )
}

fn commit_remote_ticket_admission_for_state(
    credential_state_dir: &Path,
    verifier_key: &std::sync::Arc<TicketVerifierKey>,
    authenticated_client_endpoint: Option<&str>,
    auth: &TicketAuthRequest,
    request: &ConcreteBuildRequest,
) -> Result<CommittedRemoteTicketAdmission, String> {
    let _guard = crate::remote_credential_state::acquire_ticket_state_mutation_guard(credential_state_dir)
        .map_err(|error| error.to_string())?;
    let mut state = load_ticket_state(credential_state_dir).map_err(|error| error.to_string())?;
    state.bind_active_verifier_key(verifier_key);
    let server_now_unix_s = crate::unix_time_now_s().map_err(|error| error.to_string())?;
    let ticket = state
        .tickets
        .get(&auth.ticket_id)
        .ok_or_else(|| format!("unknown-remote-ticket-{}", auth.ticket_id))?;
    expect_authorized_ticket(ticket, auth, RemoteTicketAuthFacts {
        server_now_unix_s,
        authenticated_client_endpoint,
    })?;
    validate_concrete_request(request, ticket)?;
    let max_upload_bytes = ticket.max_upload_bytes;
    redeem_and_commit_ticket_state(&mut state, &auth.ticket_id, &mut |candidate| {
        save_ticket_state(credential_state_dir, candidate).map_err(|_| "remote-ticket-state-commit-failed".to_string())
    })?;
    Ok(CommittedRemoteTicketAdmission {
        server_now_unix_s,
        max_upload_bytes,
    })
}

fn read_remote_server_trace_context(
    reader: &mut impl Read,
    is_trace_negotiated: bool,
) -> Result<RemoteTraceContextHealth, String> {
    if !is_trace_negotiated {
        return Ok(accept_remote_trace_context(false, None, None).1);
    }
    let context = match read_expected_remote_frame(reader, RemoteFrameKind::TraceContext)? {
        RemoteFrame::TraceContext { context } => context,
        _ => return Err("remote-trace-context-frame-invalid".to_string()),
    };
    let health = match context {
        Some(context) => accept_remote_trace_context(true, Some(&context.traceparent), context.tracestate.as_deref()).1,
        None => accept_remote_trace_context(true, None, None).1,
    };
    debug_assert!(!health.reason_code.is_empty());
    debug_assert!(is_trace_negotiated);
    Ok(health)
}

fn receive_remote_production_inputs(
    reader: &mut impl Read,
    writer: &mut impl Write,
    context: &RemoteProductionServerContext<'_>,
    opening: &RemoteProductionServerOpening,
    runtime: &tokio::runtime::Runtime,
) -> Result<u64, String> {
    let max_upload_bytes = opening.max_upload_bytes;
    let uploaded_bytes = if opening.request.input_refs.is_empty() {
        0
    } else {
        receive_remote_production_input_transfer(reader, writer, context, RemoteProductionInputTransferInput {
            opening,
            runtime,
            max_upload_bytes,
        })?
    };
    validate_missing_uploads(&opening.missing, &opening.missing, uploaded_bytes, max_upload_bytes)?;
    debug_assert!(uploaded_bytes <= max_upload_bytes);
    Ok(uploaded_bytes)
}

fn redeem_and_commit_ticket_state(
    state: &mut RemoteTicketState,
    ticket_id: &str,
    commit_ticket_state: &mut impl FnMut(&RemoteTicketState) -> Result<(), String>,
) -> Result<(), String> {
    let ticket = state.tickets.get_mut(ticket_id).ok_or_else(|| "remote-production-ticket-disappeared".to_string())?;
    let uses_before = ticket.uses_remaining;
    redeem_after_queue(ticket, true)?;
    if commit_ticket_state(state).is_err() {
        let ticket =
            state.tickets.get_mut(ticket_id).ok_or_else(|| "remote-production-ticket-disappeared".to_string())?;
        ticket.uses_remaining = uses_before;
        return Err("remote-ticket-state-commit-failed".to_string());
    }
    debug_assert_eq!(
        state.tickets.get(ticket_id).and_then(|ticket| ticket.uses_remaining.checked_add(1)),
        Some(uses_before)
    );
    Ok(())
}

struct RemoteProductionInputTransferInput<'a> {
    opening: &'a RemoteProductionServerOpening,
    runtime: &'a tokio::runtime::Runtime,
    max_upload_bytes: u64,
}

fn receive_remote_production_input_transfer(
    reader: &mut impl Read,
    writer: &mut impl Write,
    context: &RemoteProductionServerContext<'_>,
    input: RemoteProductionInputTransferInput<'_>,
) -> Result<u64, String> {
    let opening = input.opening;
    let transfer = match read_expected_remote_frame(reader, RemoteFrameKind::TransferManifest)? {
        RemoteFrame::TransferManifest { transfer } => transfer,
        _ => return Err("remote-transfer-manifest-frame-invalid".to_string()),
    };
    let canonical = validate_input_transfer_manifest_frame(&opening.request, &transfer, opening.policy)?;
    if canonical.total_bytes > input.max_upload_bytes {
        return Err("upload-byte-limit-exceeded".to_string());
    }
    let receiver_root = crate::remote_transfer::remote_transfer_receiver_root(
        context.execution_state_dir,
        &canonical.manifest.session_id,
    );
    let lease_expires_unix_s = opening
        .server_now_unix_s
        .checked_add(REMOTE_TRANSFER_LEASE_DURATION_SECS)
        .ok_or_else(|| "remote-transfer-lease-expiry-overflow".to_string())?;
    let mut session = crate::remote_transfer::begin_remote_transfer_receive(
        transfer.manifest,
        &transfer.manifest_digest_blake3,
        opening.policy,
        context.execution_state_dir,
        &receiver_root,
        crate::remote_transfer::RemoteTransferRunOptions {
            direction: crate::remote_transfer::RemoteTransferDirection::Upload,
            interrupt_after_chunks: None,
            now_unix_s: opening.server_now_unix_s,
            lease_expires_unix_s,
            admission: crate::remote_transfer::RemoteTransferAdmissionFacts {
                required_closure_metadata_verified: true,
                path_info_admitted: true,
            },
        },
    )?;
    receive_remote_transfer_interactively(reader, writer, &mut session, RemoteTransferReceiveInput {
        direction: crate::remote_transfer::RemoteTransferDirection::Upload,
        state_dir: None,
        attempt: None,
        interrupt_after_chunks: None,
    })?;
    let runtime_transfer_receipt = session.finish()?;
    write_remote_control_frame(writer, &RemoteFrame::TransferComplete {
        direction: crate::remote_transfer::RemoteTransferDirection::Upload,
        report: runtime_transfer_receipt.clone(),
    })?;
    let store = input.runtime.block_on(open_remote_local_executor_store(context.executor))?;
    input
        .runtime
        .block_on(materialize_streamed_remote_inputs(&store, &opening.request, &canonical, &receiver_root))?;
    debug_assert!(runtime_transfer_receipt.transferred_bytes <= input.max_upload_bytes);
    debug_assert!(canonical.total_bytes <= input.max_upload_bytes);
    Ok(runtime_transfer_receipt.transferred_bytes)
}

struct RemoteProductionResultInput<'a> {
    opening: &'a RemoteProductionServerOpening,
    runtime: &'a tokio::runtime::Runtime,
    uploaded_bytes: u64,
}

fn send_remote_production_result(
    reader: &mut impl Read,
    writer: &mut impl Write,
    context: &RemoteProductionServerContext<'_>,
    input: RemoteProductionResultInput<'_>,
) -> Result<(), String> {
    let opening = input.opening;
    let attempt = opening
        .request
        .production_attempt
        .as_ref()
        .ok_or_else(|| "remote-production-attempt-missing".to_string())?;
    write_remote_control_frame(writer, &RemoteFrame::BuildQueued {
        request_id: opening.request.request_id.clone(),
        session_id: attempt.attempt_id.as_str().to_string(),
    })?;
    write_remote_control_frame(writer, &RemoteFrame::BuildStarted {
        request_id: opening.request.request_id.clone(),
    })?;
    let plan = plan_remote_executable_request(&opening.request)?;
    let upload = RemoteInputUpload {
        request_id: opening.request.request_id.clone(),
        refs: opening.missing.clone(),
        byte_count: input.uploaded_bytes,
        artifacts: Vec::new(),
        streamed: true,
    };
    let execution = context.executor.execute(&opening.request, &plan, &upload)?;
    validate_remote_execution_outcome(&execution, &opening.request, &plan)?;
    validate_remote_execution_output_pathinfos(
        &execution.outputs,
        &context.builder.signing_key_id,
        &opening.request.store_prefix,
    )?;
    let prepared =
        input
            .runtime
            .block_on(prepare_remote_production_output(context.executor, &opening.request, &execution))?;
    send_remote_production_output_transfer(reader, writer, context, opening, &prepared)?;
    debug_assert!(is_blake3_hex_digest(&execution.output_digest_blake3));
    debug_assert_eq!(execution.outputs.len(), opening.request.expected_outputs.len());
    Ok(())
}

fn send_remote_production_output_transfer(
    reader: &mut impl Read,
    writer: &mut impl Write,
    context: &RemoteProductionServerContext<'_>,
    opening: &RemoteProductionServerOpening,
    prepared: &PreparedRemoteProductionOutput,
) -> Result<(), String> {
    write_remote_control_frame(writer, &RemoteFrame::BuildFinished {
        result: prepared.result.clone(),
    })?;
    let client_transfer = remote_transfer_capabilities_from_labels(&opening.hello.capabilities);
    let (actual_mode, fallback_reason) =
        production_output_transfer_selection(client_transfer, context.builder.transfer_capabilities)?;
    write_remote_control_frame(writer, &RemoteFrame::TransferManifest {
        transfer: RemoteTransferManifestFrame {
            direction: crate::remote_transfer::RemoteTransferDirection::Download,
            manifest: prepared.transfer.manifest.manifest.clone(),
            manifest_digest_blake3: prepared.transfer.manifest.digest_blake3.clone(),
            actual_mode,
            fallback_reason: fallback_reason.clone(),
        },
    })?;
    let demand = match read_expected_remote_frame(reader, RemoteFrameKind::TransferDemand)? {
        RemoteFrame::TransferDemand { transfer } => transfer,
        _ => return Err("remote-transfer-demand-frame-invalid".to_string()),
    };
    let runtime_transfer_receipt = send_prepared_remote_transfer(reader, writer, RemoteTransferSendInput {
        prepared: &prepared.transfer,
        policy: opening.policy,
        direction: crate::remote_transfer::RemoteTransferDirection::Download,
        demand_frame: demand,
        interrupt_after_chunks: None,
    })?;
    let transfer_receipt = production_transfer_report_from_runtime(
        &runtime_transfer_receipt,
        &context.builder.signing_key_id,
        actual_mode,
        fallback_reason,
    )?;
    write_remote_control_frame(writer, &RemoteFrame::OutputTransferDone {
        report: transfer_receipt,
    })?;
    write_remote_control_frame(writer, &RemoteFrame::Done {
        request_id: opening.request.request_id.clone(),
    })?;
    debug_assert!(is_blake3_hex_digest(&prepared.result.output_digest_blake3));
    debug_assert_eq!(prepared.result.request_id, opening.request.request_id);
    Ok(())
}

struct RemoteServeCommandInput<'a> {
    endpoint_id: String,
    binding: crate::RemoteServeBinding,
    fixture_signing_key_id: String,
    executor: crate::RemoteServeExecutor,
    present_input_refs: Vec<String>,
    output_dir: &'a Path,
    state_dir: &'a Path,
    execution_state_dir: &'a Path,
    store_prefix: &'a str,
    json_output: bool,
    secret_request: crate::remote_service_secrets::RemoteServiceSecretRequest,
}

fn cmd_remote_serve(input: RemoteServeCommandInput<'_>) -> Result<(), RunError> {
    match input.binding {
        crate::RemoteServeBinding::Metadata => {
            print_json_or_human(&remote_serve_metadata_json((&input.endpoint_id, "metadata-only")), input.json_output)
        }
        crate::RemoteServeBinding::StdioOnce => {
            let service_keys =
                crate::remote_service_secrets::resolve_remote_service_keys_bounded(&input.secret_request)?;
            let (builder_signing_key_id, local_executor) = remote_serve_executor(
                RemoteServeExecutorInput {
                    executor: input.executor,
                    endpoint_id: &input.endpoint_id,
                    fixture_signing_key_id: input.fixture_signing_key_id,
                    output_dir: input.output_dir,
                    coordinator_state_dir: input.state_dir,
                    state_dir: input.execution_state_dir,
                    store_prefix: input.store_prefix,
                },
                service_keys.result_signing_key,
            )?;
            let builder = RemoteLoopbackBuilder {
                endpoint_id: input.endpoint_id,
                store_prefix: input.store_prefix.to_string(),
                supported_capabilities: vec![
                    "delta".to_string(),
                    "full".to_string(),
                    "streaming".to_string(),
                    REMOTE_TRACE_CONTEXT_CAPABILITY.to_string(),
                ],
                present_input_refs: input.present_input_refs,
                signing_key_id: builder_signing_key_id,
                transfer_capabilities: RemoteTransferCapabilities::delta_and_full().with_streaming(),
            };
            match local_executor {
                Some(local_executor) => serve_stdio_remote_production_once(
                    &mut std::io::stdin().lock(),
                    &mut std::io::stdout().lock(),
                    RemoteProductionServerContext {
                        builder: &builder,
                        verifier_key: &service_keys.verifier_key,
                        executor: &local_executor,
                        credential_state_dir: input.state_dir,
                        execution_state_dir: input.execution_state_dir,
                        authenticated_client_endpoint: None,
                    },
                )
                .map_err(|err| RunError::Internal(format!("remote production stdio serve once: {err}"))),
                None => serve_stdio_remote_fixture_once(input.state_dir, &service_keys.verifier_key, &builder),
            }
        }
    }
}

fn serve_stdio_remote_fixture_once(
    state_dir: &Path,
    verifier_key: &std::sync::Arc<TicketVerifierKey>,
    builder: &RemoteLoopbackBuilder,
) -> Result<(), RunError> {
    let input = read_bounded_stdio_input(std::io::stdin().lock()).map_err(RunError::Internal)?;
    let client_frames = decode_remote_frame_stream(&input).map_err(RunError::Internal)?;
    let admission = {
        let _guard = crate::remote_credential_state::acquire_ticket_state_mutation_guard(state_dir)?;
        let mut state = load_ticket_state(state_dir)?;
        state.bind_active_verifier_key(verifier_key);
        let server_now_unix_s = crate::unix_time_now_s()?;
        let ticket_id = request_ticket_id_from_frames(&client_frames).map_err(RunError::Internal)?;
        let ticket = state
            .tickets
            .get_mut(ticket_id)
            .ok_or_else(|| RunError::Internal(format!("unknown-remote-ticket-{ticket_id}")))?;
        let admission = admit_remote_builder_frames_with_auth_facts(
            builder,
            ticket,
            &client_frames,
            RemoteTransferCapabilities::delta_and_full(),
            RemoteTicketAuthFacts {
                server_now_unix_s,
                authenticated_client_endpoint: None,
            },
        )
        .map_err(|error| RunError::Internal(format!("remote fixture stdio serve once: {error}")))?;
        save_ticket_state(state_dir, &state)?;
        admission
    };
    let response = execute_remote_builder_admission(builder, admission, &RemoteFixtureExecutor)
        .map_err(|error| RunError::Internal(format!("remote fixture stdio serve once: {error}")))?;
    write_remote_response_frames(std::io::stdout().lock(), &response)
        .map_err(|error| RunError::Internal(format!("remote fixture stdio serve once response: {error}")))
}

fn remote_serve_metadata_json(input: (&str, &str)) -> serde_json::Value {
    let (endpoint_id, status) = input;
    serde_json::json!({
        "protocol": REMOTE_PROTOCOL_ALPN,
        "endpoint_id": endpoint_id,
        "frame_encoding": "u32be-length-prefixed-json",
        "supported_bindings": [
            RemoteTransportBinding::Loopback.as_str(),
            RemoteTransportBinding::Stdio.as_str(),
            RemoteTransportBinding::SshStdio.as_str(),
            RemoteTransportBinding::P2p.as_str(),
        ],
        "status": status,
        "diagnostic": "remote serve metadata is stable; stdio-once is a supported framed stdio binding; ssh-stdio reuses the same frame contract; production P2P listener dispatch remains gated"
    })
}

fn cmd_remote_status(
    endpoint_id: String,
    concurrency: u32,
    state_dir: &Path,
    json_output: bool,
) -> Result<(), RunError> {
    let ticket_state = load_ticket_state(state_dir)?;
    let tickets = ticket_state.tickets.values().cloned().collect::<Vec<_>>();
    let coordinator_state = load_coordinator_state(state_dir)?;
    let snapshot = coordinator_status_snapshot(&endpoint_id, concurrency, &coordinator_state, &[], &tickets)
        .map_err(RunError::Internal)?;
    print_json_or_human(&snapshot, json_output)
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct TicketCreationReport {
    ticket: RemoteTicketView,
    delivery: TicketDeliveryReport,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct TicketDeliveryReport {
    target: &'static str,
    delivered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TicketDeliveryTarget {
    CallerOwnedFileDescriptor(i32),
    InteractiveOperatorTerminal,
}

impl TicketDeliveryTarget {
    fn label(self) -> &'static str {
        match self {
            Self::CallerOwnedFileDescriptor(_) => "caller-owned-fd",
            Self::InteractiveOperatorTerminal => "interactive-operator-terminal",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RemoteKeyRotationReport {
    active_ticket_verifier_key_id: String,
    active_result_signing_key_id: String,
    invalidated_ticket_ids: Vec<String>,
}

fn cmd_remote_ticket(action: crate::RemoteTicketAction, state_dir: &Path, json_output: bool) -> Result<(), RunError> {
    match action {
        crate::RemoteTicketAction::Create {
            display_name,
            ttl_secs,
            uses,
            max_build_time_secs,
            max_upload_bytes,
            bound_client_endpoint,
            ticket_fd,
            interactive_operator_terminal_reveal,
            secret_manifest,
            secret_profile,
            secret_provider,
        } => create_and_deliver_remote_ticket(
            state_dir,
            json_output,
            ticket_delivery_target(ticket_fd, interactive_operator_terminal_reveal)?,
            TicketIssueInput {
                display_name,
                now_unix_s: crate::unix_time_now_s()?,
                ttl_secs,
                uses,
                max_build_time_secs,
                max_upload_bytes,
                bound_client_endpoint,
            },
            crate::remote_service_secrets::RemoteServiceSecretRequest {
                manifest_path: secret_manifest,
                profile: secret_profile,
                provider: secret_provider,
            },
        ),
        crate::RemoteTicketAction::List => {
            let state = load_ticket_state(state_dir)?;
            let views = state.tickets.values().map(redacted_ticket_view).collect::<Vec<_>>();
            print_json_or_human(&views, json_output)
        }
        crate::RemoteTicketAction::Inspect { id } => {
            let state = load_ticket_state(state_dir)?;
            let ticket =
                state.tickets.get(&id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
            print_json_or_human(&redacted_ticket_view(ticket), json_output)
        }
        crate::RemoteTicketAction::Revoke { id } => {
            let _ticket_state_guard = crate::remote_credential_state::acquire_ticket_state_mutation_guard(state_dir)?;
            let mut state = load_ticket_state(state_dir)?;
            let view = revoke_ticket(&mut state, &id)?;
            save_ticket_state(state_dir, &state)?;
            print_json_or_human(&view, json_output)
        }
        crate::RemoteTicketAction::MigrateLegacy {
            invalidate_legacy_tickets,
            dry_run,
        } => {
            let report = crate::remote_credential_state::migrate_legacy_ticket_state(
                state_dir,
                invalidate_legacy_tickets,
                dry_run,
            )?;
            print_json_or_human(&report, json_output)
        }
        crate::RemoteTicketAction::RotateKeys {
            secret_manifest,
            secret_profile,
            secret_provider,
        } => rotate_remote_service_keys(
            state_dir,
            json_output,
            crate::remote_service_secrets::RemoteServiceSecretRequest {
                manifest_path: secret_manifest,
                profile: secret_profile,
                provider: secret_provider,
            },
        ),
    }
}

fn create_and_deliver_remote_ticket(
    state_dir: &Path,
    json_output: bool,
    delivery_target: TicketDeliveryTarget,
    input: TicketIssueInput,
    secret_request: crate::remote_service_secrets::RemoteServiceSecretRequest,
) -> Result<(), RunError> {
    validate_remote_ticket_resource_limits(&input)?;
    let service_keys = crate::remote_service_secrets::resolve_remote_service_keys_bounded(&secret_request)?;
    let _ticket_state_guard = crate::remote_credential_state::acquire_ticket_state_mutation_guard(state_dir)?;
    let mut state = load_ticket_state(state_dir)?;
    state.bind_active_verifier_key(&service_keys.verifier_key);
    let previous_state = state.clone();
    let mut entropy = [0_u8; crate::remote_credentials::TICKET_ENTROPY_BYTES];
    if OsRng.try_fill_bytes(&mut entropy).is_err() {
        entropy.zeroize();
        return Err(RunError::Internal("remote-ticket-os-randomness-failed".to_string()));
    }
    let planned = crate::remote_credentials::plan_ticket_issue(&state, input, &entropy, &service_keys.verifier_key);
    entropy.zeroize();
    let plan = planned.map_err(RunError::Internal)?;
    let view = crate::remote_credentials::apply_ticket_issue(&mut state, &plan).map_err(RunError::Internal)?;
    save_ticket_state(state_dir, &state)?;
    if plan.with_bearer_credential(|credential| deliver_ticket(delivery_target, credential)).is_err() {
        save_ticket_state(state_dir, &previous_state)
            .map_err(|_| RunError::Internal("remote-ticket-delivery-failed-and-state-rollback-failed".to_string()))?;
        return Err(RunError::Internal("remote-ticket-delivery-failed".to_string()));
    }
    let report = TicketCreationReport {
        ticket: view,
        delivery: TicketDeliveryReport {
            target: delivery_target.label(),
            delivered: true,
        },
    };
    print_json_or_human(&report, json_output)
}

fn validate_remote_ticket_resource_limits(input: &TicketIssueInput) -> Result<(), RunError> {
    if input.max_build_time_secs > MAX_REMOTE_BUILD_TIME_SECS {
        return Err(RunError::Internal("remote-ticket-build-time-limit-exceeded".to_string()));
    }
    if input.max_upload_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err(RunError::Internal("remote-ticket-upload-limit-exceeded".to_string()));
    }
    Ok(())
}

fn rotate_remote_service_keys(
    state_dir: &Path,
    json_output: bool,
    secret_request: crate::remote_service_secrets::RemoteServiceSecretRequest,
) -> Result<(), RunError> {
    let service_keys = crate::remote_service_secrets::resolve_remote_service_keys_bounded(&secret_request)?;
    let _ticket_state_guard = crate::remote_credential_state::acquire_ticket_state_mutation_guard(state_dir)?;
    let mut state = load_ticket_state(state_dir)?;
    let active_key_id = service_keys.verifier_key.id().to_string();
    let retiring_key_ids = state.tickets.values().map(|ticket| ticket.verifier_key_id.clone()).collect::<BTreeSet<_>>();
    let mut invalidated_ticket_ids = Vec::new();
    for retiring_key_id in retiring_key_ids {
        invalidated_ticket_ids.extend(
            crate::remote_credentials::invalidate_tickets_for_key(&mut state, &retiring_key_id)
                .map_err(RunError::Internal)?,
        );
    }
    state.bind_active_verifier_key(&service_keys.verifier_key);
    invalidated_ticket_ids.sort();
    save_ticket_state(state_dir, &state)?;
    let report = RemoteKeyRotationReport {
        active_ticket_verifier_key_id: active_key_id,
        active_result_signing_key_id: service_keys.result_signing_key.verifying_key.name().to_string(),
        invalidated_ticket_ids,
    };
    print_json_or_human(&report, json_output)
}

fn ticket_delivery_target(
    ticket_fd: Option<i32>,
    interactive_operator_terminal_reveal: bool,
) -> Result<TicketDeliveryTarget, RunError> {
    match (ticket_fd, interactive_operator_terminal_reveal) {
        (Some(fd), false) => Ok(TicketDeliveryTarget::CallerOwnedFileDescriptor(fd)),
        (None, true) => Ok(TicketDeliveryTarget::InteractiveOperatorTerminal),
        _ => Err(RunError::Internal("remote-ticket-delivery-target-invalid".to_string())),
    }
}

fn deliver_ticket(target: TicketDeliveryTarget, credential: &str) -> Result<(), RunError> {
    match target {
        TicketDeliveryTarget::CallerOwnedFileDescriptor(fd) => deliver_ticket_to_owned_fd(fd, credential),
        TicketDeliveryTarget::InteractiveOperatorTerminal => deliver_ticket_to_operator_terminal(credential),
    }
}

#[cfg(unix)]
fn deliver_ticket_to_owned_fd(fd: i32, credential: &str) -> Result<(), RunError> {
    use std::io::IsTerminal as _;
    use std::os::fd::FromRawFd as _;

    if fd <= libc::STDERR_FILENO {
        return Err(RunError::Internal("remote-ticket-delivery-fd-reserved".to_string()));
    }
    // SAFETY: The caller explicitly transfers this child-process descriptor to the command.
    let mut file = unsafe { File::from_raw_fd(fd) };
    if file.is_terminal() {
        return Err(RunError::Internal("remote-ticket-terminal-delivery-forbidden".to_string()));
    }
    file.write_all(credential.as_bytes())
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.flush())
        .map_err(|_| RunError::Internal("remote-ticket-delivery-write-failed".to_string()))?;
    Ok(())
}

#[cfg(not(unix))]
fn deliver_ticket_to_owned_fd(_fd: i32, _credential: &str) -> Result<(), RunError> {
    Err(RunError::Internal("remote-ticket-delivery-fd-unsupported".to_string()))
}

#[cfg(unix)]
fn deliver_ticket_to_operator_terminal(credential: &str) -> Result<(), RunError> {
    use std::io::IsTerminal as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    let mut terminal = OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open("/dev/tty")
        .map_err(|_| RunError::Internal("remote-ticket-operator-terminal-unavailable".to_string()))?;
    if !terminal.is_terminal() {
        return Err(RunError::Internal("remote-ticket-operator-terminal-invalid".to_string()));
    }
    terminal
        .write_all(credential.as_bytes())
        .and_then(|()| terminal.write_all(b"\n"))
        .and_then(|()| terminal.flush())
        .map_err(|_| RunError::Internal("remote-ticket-operator-terminal-write-failed".to_string()))
}

#[cfg(not(unix))]
fn deliver_ticket_to_operator_terminal(_credential: &str) -> Result<(), RunError> {
    Err(RunError::Internal("remote-ticket-operator-terminal-unsupported".to_string()))
}

fn print_json_or_human(value: &impl Serialize, json_output: bool) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|err| RunError::Internal(format!("serializing remote output: {err}")))?;
    if json_output {
        println!("{rendered}");
        return Ok(());
    }
    println!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::fd::IntoRawFd as _;
    use std::os::unix::fs::PermissionsExt;

    use crunch_build::distributed::RemoteAttemptLogReasonCode;
    use crunch_build::distributed::RemoteNamedResourceQuantity;
    use crunch_build::distributed::RemoteResourceReasonCode;
    use crunch_build::distributed::RemoteResourceVector;

    use super::*;

    mod external_batch_hardware_tests;

    const CUSTOM_REMOTE_OUTPUT_BYTES: u64 = 777;
    const TEST_TICKET_DELIVERY_FD: i32 = 9;
    const FORGED_CLIENT_NOW_UNIX_S: u64 = 2;
    const EXPIRED_TICKET_UNIX_S: u64 = 5;
    const AUTHORITY_NOW_AFTER_EXPIRY_UNIX_S: u64 = 6;
    const TEST_TICKET_FD_TIMEOUT_MS: u64 = 30;
    const REMOTE_CLIENT_DISPATCH_FRAME_COUNT: usize = 5;
    const ED25519_SIGNATURE_BYTES: usize = 64;
    const SHA256_DIGEST_BYTES: usize = 32;
    const BUILDER_SIGNATURE_FILL_BYTE: u8 = 3;
    const OTHER_SIGNATURE_FILL_BYTE: u8 = 4;
    const IMPORT_NAR_SHA256_FILL_BYTE: u8 = 7;
    const IMPORT_NAR_SIZE_BYTES: u64 = 1;
    const CORRUPTED_TRANSFER_PAYLOAD_BYTE: u8 = b'X';
    const OPERATOR_RAIL_LOG_START_CURSOR: u64 = 0;
    const OPERATOR_RAIL_LOG_NEXT_CURSOR: u64 = 1;
    const TEST_ATTEMPT_NOW_UNIX_S: u64 = 100;
    const TEST_ATTEMPT_DEADLINE_UNIX_S: u64 = 10_000;
    const TEST_ATTEMPT_OUTPUT_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const TEST_RETRY_NOW_UNIX_S: u64 = 110;
    const TEST_RETRY_FAILURE_UNIX_S: u64 = 100;
    const TEST_HEARTBEAT_OBSERVED_UNIX_S: u64 = 101;
    const TEST_LOG_CURSOR: u64 = 0;
    const TEST_INTERACTIVE_TRANSFER_BYTES: usize = 32_768;
    const TEST_INTERACTIVE_PATTERN_MODULUS: usize = 251;
    const TEST_INTERACTIVE_NOW_UNIX_S: u64 = 1_000;
    const TEST_INTERACTIVE_LEASE_EXPIRES_UNIX_S: u64 = 2_000;
    const TEST_TRANSFER_CHECKPOINT: u64 = 1;
    const TEST_TRANSFERRED_BYTES: u64 = 64;
    const TEST_RESOURCE_CPU_UNITS: u32 = 8;
    const TEST_RESOURCE_MEMORY_BYTES: u64 = 16_384;
    const TEST_RESOURCE_SCRATCH_BYTES: u64 = 32_768;
    const TEST_RESOURCE_ACCELERATOR_COUNT: u32 = 1;
    const TEST_RESOURCE_TOKEN_COUNT: u32 = 2;
    const TEST_WORKER_CONCURRENCY: u32 = 4;
    const TEST_EXTERNAL_BATCH_TIMEOUT_SECS: u64 = 10;
    const TEST_EXTERNAL_BATCH_MAX_ATTEMPTS: u32 = 3;
    const TEST_EXTERNAL_BATCH_OUTPUT_LIMIT_BYTES: u64 = 512;
    const TEST_LOCALITY_ARTIFACT_BYTES: u64 = 8;
    const TEST_LOCALITY_CHUNK_BYTES: u32 = 8;
    const TRANSACTION_TICKET_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const TRANSACTION_TICKET_TTL_SECS: u64 = 3_600;
    const STALE_TRANSACTION_TICKET_USES: u32 = 2;
    const CONCURRENT_CONTENDER_COUNT: usize = 2;
    const EXTERNAL_LEGACY_TICKET_ID: &str = "legacy-observation";

    fn fixture_log_control_summary(
        retained_start_cursor: u64,
        next_cursor: u64,
        truncated: bool,
    ) -> RemoteAttemptLogControlSummary {
        let digest = RemoteAttemptLogDigest::new("a".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap();
        RemoteAttemptLogControlSummary {
            scope: RemoteAttemptLogScope {
                job_id: RemoteJobId::new("diagnostic-log-job").unwrap(),
                attempt_id: RemoteAttemptId::new("diagnostic-log-attempt").unwrap(),
                fence_generation: RemoteFenceGeneration::INITIAL,
            },
            retention_policy: RemoteLogRetentionPolicy::default(),
            retained_start_cursor,
            next_cursor,
            retained_record_count: 1,
            retained_payload_bytes: 3,
            head_record_blake3: Some(digest.clone()),
            head_segment_blake3: Some(digest.clone()),
            manifest_blake3: digest.clone(),
            truncation_anchor_blake3: truncated.then_some(digest),
            truncated,
        }
    }

    fn fixture_attempt_time(now_unix_s: u64) -> RemoteAttemptTimeFacts {
        RemoteAttemptTimeFacts {
            now_unix_s,
            failure_observed_unix_s: now_unix_s,
            overall_deadline_unix_s: TEST_ATTEMPT_DEADLINE_UNIX_S,
        }
    }

    fn fixture_assignment_nonce(label: &str) -> RemoteAssignmentNonce {
        RemoteAssignmentNonce::new(blake3::hash(label.as_bytes()).to_hex().to_string()).unwrap()
    }

    fn admit_fixture_dispatch(
        state: &mut RemoteCoordinatorState,
        request: &RemoteCoordinatorBuildRequest,
    ) -> Result<RemoteCoordinatorDispatchDecision, String> {
        let nonce_label = format!("fixture-dispatch-{}", state.jobs.len());
        admit_coordinator_dispatch_with_nonce(
            state,
            request,
            RemoteAttemptRetryPolicy::default(),
            fixture_attempt_time(TEST_ATTEMPT_NOW_UNIX_S),
            fixture_assignment_nonce(&nonce_label),
        )
    }

    fn fixture_attempt_report(
        state: &RemoteCoordinatorState,
        job_id: &RemoteJobId,
        event_id: &str,
        payload: RemoteAttemptReportPayload,
    ) -> RemoteAttemptReport {
        let attempt = state.jobs[job_id].current_attempt.as_ref().expect("fixture attempt exists");
        RemoteAttemptReport::new(
            job_id.clone(),
            attempt.attempt_id.clone(),
            attempt.fence_generation,
            RemoteEventId::new(event_id).expect("fixture event id is valid"),
            payload,
        )
        .expect("fixture report is canonical")
    }

    fn apply_fixture_attempt_report(
        state: &mut RemoteCoordinatorState,
        report: &RemoteAttemptReport,
    ) -> RemoteCoordinatorAttemptApplyResult {
        apply_coordinator_attempt_report(
            state,
            report,
            RemoteAttemptAuthorizationFacts {
                worker_authorized: true,
                output_admission_authorized: true,
            },
            RemoteLogRetentionPolicy::default(),
        )
        .expect("fixture report applies")
    }

    fn persisted_log_fixture(
        payloads: &[&str],
        retention: RemoteLogRetentionPolicy,
    ) -> (tempfile::TempDir, RemoteCoordinatorState, RemoteJobId) {
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let dispatch = admit_fixture_dispatch(&mut state, &fixture_coordinator_request()).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let start = fixture_attempt_report(&state, &job_id, "persisted-log-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(&mut state, &start);
        for (cursor, payload) in payloads.iter().enumerate() {
            let cursor = u64::try_from(cursor).expect("bounded fixture cursor fits u64");
            let report = fixture_attempt_report(
                &state,
                &job_id,
                &format!("persisted-log-{cursor}"),
                RemoteAttemptReportPayload::LogAppend {
                    cursor,
                    bytes: (*payload).to_string(),
                },
            );
            apply_coordinator_attempt_report(
                &mut state,
                &report,
                RemoteAttemptAuthorizationFacts {
                    worker_authorized: true,
                    output_admission_authorized: true,
                },
                retention,
            )
            .expect("persisted log append applies");
        }
        (temp, state, job_id)
    }

    fn assert_persisted_segment_tamper_rejected(mutate: impl FnOnce(&mut serde_json::Value)) {
        const REPLAY_BYTES_MAX: u64 = 1_024;
        let (temp, state, job_id) = persisted_log_fixture(&["diagnostic"], RemoteLogRetentionPolicy::default());
        let summary = state.jobs[&job_id].immutable_log.as_ref().expect("log summary exists");
        let manifest = crate::remote_attempt_log_store::load_remote_attempt_log_manifest(
            temp.path(),
            &summary.scope,
            immutable_attempt_log_policy(summary.retention_policy).unwrap(),
        )
        .unwrap();
        let segment = manifest.segments.first().expect("retained segment exists");
        let path = crate::remote_attempt_log_store::remote_attempt_log_segment_path(
            temp.path(),
            &summary.scope,
            &segment.segment_blake3,
        );
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        mutate(&mut value);
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let state_before = state.clone();
        let control_before = fs::read(coordinator_state_path(temp.path())).unwrap();
        let error = replay_coordinator_attempt_log(&state, &job_id, RemoteAttemptLogReplayRequest {
            from_cursor: summary.retained_start_cursor,
            record_count_max: 1,
            payload_bytes_max: REPLAY_BYTES_MAX,
        })
        .expect_err("tampered immutable segment fails closed");

        assert!(error.starts_with("attempt-log-"));
        assert_eq!(state, state_before);
        assert_eq!(fs::read(coordinator_state_path(temp.path())).unwrap(), control_before);
    }

    fn fixture_report_for_attempt(
        attempt: &RemoteAttemptState,
        event_id: &str,
        payload: RemoteAttemptReportPayload,
    ) -> RemoteAttemptReport {
        RemoteAttemptReport::new(
            attempt.job_id.clone(),
            attempt.attempt_id.clone(),
            attempt.fence_generation,
            RemoteEventId::new(event_id).expect("fixture event id is valid"),
            payload,
        )
        .expect("fixture report is canonical")
    }

    fn fixture_retry_time() -> RemoteAttemptTimeFacts {
        RemoteAttemptTimeFacts {
            now_unix_s: TEST_RETRY_NOW_UNIX_S,
            failure_observed_unix_s: TEST_RETRY_FAILURE_UNIX_S,
            overall_deadline_unix_s: TEST_ATTEMPT_DEADLINE_UNIX_S,
        }
    }

    fn register_second_fixture_worker(state: &mut RemoteCoordinatorState) {
        let mut worker = fixture_worker_registration();
        worker.endpoint_id = "builder-2".to_string();
        apply_worker_registration(state, worker).expect("second worker registers");
    }

    fn advance_current_fixture_attempt(
        state: &mut RemoteCoordinatorState,
        job_id: &RemoteJobId,
        target_phase: RemoteAttemptPhase,
    ) {
        assert!(matches!(
            target_phase,
            RemoteAttemptPhase::Queued
                | RemoteAttemptPhase::Running
                | RemoteAttemptPhase::Transferring
                | RemoteAttemptPhase::FinishedUndelivered
        ));
        assert!(!target_phase.is_terminal());
        if target_phase == RemoteAttemptPhase::Queued {
            return;
        }
        let start = fixture_attempt_report(state, job_id, "phase-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(state, &start);
        if target_phase == RemoteAttemptPhase::Running {
            return;
        }
        let transfer =
            fixture_attempt_report(state, job_id, "phase-transfer", RemoteAttemptReportPayload::TransferCheckpoint {
                checkpoint: TEST_TRANSFER_CHECKPOINT,
                transferred_bytes: TEST_TRANSFERRED_BYTES,
            });
        apply_fixture_attempt_report(state, &transfer);
        if target_phase == RemoteAttemptPhase::Transferring {
            return;
        }
        let ready = fixture_attempt_report(state, job_id, "phase-result", RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
        });
        apply_fixture_attempt_report(state, &ready);
    }

    fn complete_current_fixture_attempt(state: &mut RemoteCoordinatorState, job_id: &RemoteJobId) {
        let start = fixture_attempt_report(state, job_id, "current-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(state, &start);
        let ready = fixture_attempt_report(state, job_id, "current-result", RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
        });
        apply_fixture_attempt_report(state, &ready);
        let complete =
            fixture_attempt_report(state, job_id, "current-complete", RemoteAttemptReportPayload::Completion {
                output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
            });
        apply_fixture_attempt_report(state, &complete);
    }

    fn interactive_transfer_fixture(root: &Path) -> crate::remote_transfer::PreparedRemoteTransfer {
        let policy = RemoteTransferPolicy::default();
        let bytes = (0..TEST_INTERACTIVE_TRANSFER_BYTES)
            .map(|index| u8::try_from(index % TEST_INTERACTIVE_PATTERN_MODULUS).unwrap())
            .collect::<Vec<_>>();
        let artifact = crate::remote_transfer::prepare_inline_fixture_or_bootstrap_artifact(
            crate::remote_transfer::RemoteInlinePayloadCapability::Fixture,
            RemoteTransferArtifactId::new("fence-fixture").unwrap(),
            RemoteTransferArtifactKind::SourceBundle,
            &bytes,
            root,
            true,
            policy,
        )
        .unwrap();
        let requested_content_blake3 = artifact.descriptor.digest_blake3.clone();
        crate::remote_transfer::prepare_remote_transfer(
            crate::remote_transfer::RemoteTransferBinding {
                job_id: RemoteJobId::new("fence-job").unwrap(),
                attempt_id: RemoteAttemptId::new("fence-attempt").unwrap(),
                fence_generation: RemoteFenceGeneration::INITIAL,
                store_prefix: "/mantle/store".to_string(),
                requested_content_blake3,
            },
            policy,
            vec![artifact],
        )
        .unwrap()
    }

    fn interactive_receive_session(
        prepared: &crate::remote_transfer::PreparedRemoteTransfer,
        state_dir: &Path,
        receiver_root: &Path,
    ) -> crate::remote_transfer::RemoteTransferReceiveSession {
        crate::remote_transfer::begin_remote_transfer_receive(
            prepared.manifest.manifest.clone(),
            &prepared.manifest.digest_blake3,
            RemoteTransferPolicy::default(),
            state_dir,
            receiver_root,
            crate::remote_transfer::RemoteTransferRunOptions {
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                interrupt_after_chunks: None,
                now_unix_s: TEST_INTERACTIVE_NOW_UNIX_S,
                lease_expires_unix_s: TEST_INTERACTIVE_LEASE_EXPIRES_UNIX_S,
                admission: crate::remote_transfer::RemoteTransferAdmissionFacts {
                    required_closure_metadata_verified: true,
                    path_info_admitted: true,
                },
            },
        )
        .unwrap()
    }

    #[test]
    fn stale_final_chunk_invalidates_checkpoint_before_acknowledgement_or_completion() {
        let root = tempfile::tempdir().unwrap();
        let prepared = interactive_transfer_fixture(&root.path().join("spool"));
        let state_dir = root.path().join("state");
        let receiver_root = root.path().join("receiver");
        let mut session = interactive_receive_session(&prepared, &state_dir, &receiver_root);
        let demand = session.demand().clone();
        let missing = demand.missing_chunks[0].clone();
        let grant = session.credit_for_chunk(&missing).unwrap();
        let mut incoming = Vec::new();
        crate::remote_transfer::write_remote_transfer_data_chunk(
            &mut incoming,
            &prepared,
            RemoteTransferPolicy::default(),
            &demand,
            &session.sender_credit_state(),
            grant,
            &missing,
        )
        .unwrap();
        let mut validator_calls = 0_u32;
        let mut validate = || {
            validator_calls = validator_calls.checked_add(1).unwrap();
            if validator_calls == 1 {
                Ok(())
            } else {
                Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string())
            }
        };
        let mut control = Vec::new();
        let error = receive_remote_transfer_interactively_with_fence_validator(
            &mut std::io::Cursor::new(incoming),
            &mut control,
            &mut session,
            RemoteTransferReceiveInput {
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                state_dir: None,
                attempt: None,
                interrupt_after_chunks: None,
            },
            &mut validate,
        )
        .unwrap_err();
        assert_eq!(error, RemoteAttemptReasonCode::StaleReportRejected.as_str());
        assert!(
            fs::symlink_metadata(crate::remote_transfer::remote_transfer_state_path(
                &state_dir,
                &prepared.manifest.manifest.session_id,
            ))
            .is_err()
        );
        assert!(fs::symlink_metadata(&receiver_root).is_err());
        let mut frames = std::io::Cursor::new(control);
        assert!(matches!(read_remote_frame(&mut frames).unwrap(), RemoteFrame::TransferDemand { .. }));
        assert!(matches!(read_remote_frame(&mut frames).unwrap(), RemoteFrame::TransferCredit { .. }));
        assert!(read_remote_frame(&mut frames).is_err());
        assert!(session.finish().is_err());
    }

    #[test]
    fn stale_input_attempt_stops_before_source_read_or_data_disclosure() {
        let root = tempfile::tempdir().unwrap();
        let prepared = interactive_transfer_fixture(&root.path().join("spool"));
        let receiver =
            interactive_receive_session(&prepared, &root.path().join("state"), &root.path().join("receiver"));
        let demand = RemoteTransferDemandFrame {
            direction: crate::remote_transfer::RemoteTransferDirection::Upload,
            demand: receiver.demand().clone(),
            sender_state: receiver.sender_credit_state(),
        };
        let missing = demand.demand.missing_chunks[0].clone();
        let grant = receiver.credit_for_chunk(&missing).unwrap();
        let mut control = Vec::new();
        write_remote_control_frame(&mut control, &RemoteFrame::TransferCredit {
            transfer: RemoteTransferCreditFrame {
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                missing,
                grant,
            },
        })
        .unwrap();
        let mut disclosed = Vec::new();
        let error = send_prepared_remote_transfer_with_fence_validator(
            &mut std::io::Cursor::new(control),
            &mut disclosed,
            RemoteTransferSendInput {
                prepared: &prepared,
                policy: RemoteTransferPolicy::default(),
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                demand_frame: demand,
                interrupt_after_chunks: None,
            },
            &mut || Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string()),
        )
        .unwrap_err();
        assert_eq!(error, RemoteAttemptReasonCode::StaleReportRejected.as_str());
        assert!(disclosed.is_empty());
        assert!(!receiver.demand().missing_chunks.is_empty());
    }

    #[test]
    fn production_sender_rejects_excess_credit_before_source_disclosure() {
        let root = tempfile::tempdir().unwrap();
        let prepared = interactive_transfer_fixture(&root.path().join("spool"));
        let receiver =
            interactive_receive_session(&prepared, &root.path().join("state"), &root.path().join("receiver"));
        let demand = RemoteTransferDemandFrame {
            direction: crate::remote_transfer::RemoteTransferDirection::Upload,
            demand: receiver.demand().clone(),
            sender_state: receiver.sender_credit_state(),
        };
        let missing = demand.demand.missing_chunks[0].clone();
        let mut control = Vec::new();
        write_remote_control_frame(&mut control, &RemoteFrame::TransferCredit {
            transfer: RemoteTransferCreditFrame {
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                missing: missing.clone(),
                grant: RemoteTransferCreditGrant {
                    bytes: u64::from(missing.chunk.size_bytes).saturating_add(1),
                    chunks: 1,
                },
            },
        })
        .unwrap();
        let mut disclosed = Vec::new();
        let mut observed = Vec::new();
        let error = send_prepared_remote_transfer_with_observer(
            &mut std::io::Cursor::new(control),
            &mut disclosed,
            RemoteTransferSendInput {
                prepared: &prepared,
                policy: RemoteTransferPolicy::default(),
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                demand_frame: demand,
                interrupt_after_chunks: None,
            },
            &mut || Ok(()),
            &mut |fact| observed.push(fact),
        )
        .unwrap_err();
        assert_eq!(error, RemoteTransferReasonCode::CreditGrantInvalid.as_str());
        assert!(disclosed.is_empty());
        assert!(matches!(observed.first(), Some(RemoteProductionTelemetryFact::TransferDemand { .. })));
        assert_eq!(observed.last(), Some(&RemoteProductionTelemetryFact::TransferCutoff { accepted: false }));
        assert!(!observed.iter().any(|fact| matches!(fact, RemoteProductionTelemetryFact::TransferCredit { .. })));
    }

    #[test]
    fn already_present_transfer_emits_accepted_cutoff_before_completion() {
        let report = crate::remote_transfer::RemoteTransferShellReport {
            direction: crate::remote_transfer::RemoteTransferDirection::Download,
            disposition: crate::remote_transfer::RemoteTransferShellDisposition::AlreadyPresent,
            manifest_digest_blake3: blake3::hash(b"already-present").to_hex().to_string(),
            transferred_bytes: 0,
            reused_bytes: 1,
            chunks_sent: 0,
            sent_chunk_digests: Vec::new(),
            checkpoint_path: "diagnostic-checkpoint".to_string(),
            output_admission_claimed: true,
        };
        let mut telemetry = RemoteTelemetryBuffer::default();
        record_transfer_completion(
            &mut telemetry,
            RemoteTelemetryPolicy::default(),
            &report,
            RemoteTransferMode::Streaming,
            None,
        );

        let reasons = telemetry.events.iter().map(|event| event.reason).collect::<Vec<_>>();
        assert_eq!(reasons, vec![
            RemoteTelemetryReasonClass::TransferCutoff,
            RemoteTelemetryReasonClass::TransferCompleted
        ]);
        assert_eq!(telemetry.events[0].result, RemoteTelemetryResultClass::Accepted);
        assert_eq!(telemetry.events[0].value, 1);
    }

    #[test]
    fn remote_root_priority_plan_orders_once_with_zero_pressure_evidence() {
        const CANDIDATE_COUNT: u32 = 2;
        let candidates = vec![
            RemoteRootPriorityCandidate {
                input_index: 0,
                root_identity_blake3: "b".repeat(BLAKE3_HEX_LENGTH_CHARS),
            },
            RemoteRootPriorityCandidate {
                input_index: 1,
                root_identity_blake3: "a".repeat(BLAKE3_HEX_LENGTH_CHARS),
            },
        ];
        let plan =
            plan_remote_root_priority(&crunch_build::scheduling::SchedulingPolicy::default(), &candidates).unwrap();

        assert_eq!(plan.ordered_input_indices, vec![1, 0]);
        assert_eq!(plan.evidence.competing_goal_count, CANDIDATE_COUNT);
        assert_eq!(plan.evidence.known_critical_path_nodes, 0);
        assert_eq!(plan.evidence.known_critical_path_work_units, 0);
        assert_eq!(plan.evidence.blocked_root_count, 0);
        assert_eq!(plan.evidence.history_basis, crunch_build::scheduling::HistoryBasis::StructuralFallbackMissing);
        assert_eq!(plan.evidence.selection_reason, crunch_build::scheduling::PrioritySelectionReason::StableGoalKey);
        assert_eq!(plan.evidence.selected_goal_key_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
        assert_ne!(plan.evidence.selected_goal_key_blake3, candidates[1].root_identity_blake3);
    }

    #[test]
    fn remote_root_priority_plan_rejects_empty_duplicate_and_oversized_snapshots() {
        let policy = crunch_build::scheduling::SchedulingPolicy::default();
        assert_eq!(plan_remote_root_priority(&policy, &[]).unwrap_err(), "remote-root-priority-candidates-empty");
        let identity = blake3::hash(b"duplicate-root").to_hex().to_string();
        let duplicate_identity = vec![
            RemoteRootPriorityCandidate {
                input_index: 0,
                root_identity_blake3: identity.clone(),
            },
            RemoteRootPriorityCandidate {
                input_index: 1,
                root_identity_blake3: identity,
            },
        ];
        assert_eq!(
            plan_remote_root_priority(&policy, &duplicate_identity).unwrap_err(),
            "remote-root-priority-identity-duplicate"
        );
        let duplicate_index = vec![
            RemoteRootPriorityCandidate {
                input_index: 0,
                root_identity_blake3: blake3::hash(b"root-a").to_hex().to_string(),
            },
            RemoteRootPriorityCandidate {
                input_index: 0,
                root_identity_blake3: blake3::hash(b"root-b").to_hex().to_string(),
            },
        ];
        assert_eq!(
            plan_remote_root_priority(&policy, &duplicate_index).unwrap_err(),
            "remote-root-priority-input-index-duplicate"
        );
        let oversized_count =
            usize::try_from(crunch_build::scheduling::MAX_READY_GOALS).unwrap().checked_add(1).unwrap();
        let oversized = vec![duplicate_identity[0].clone(); oversized_count];
        assert_eq!(
            plan_remote_root_priority(&policy, &oversized).unwrap_err(),
            "remote-root-priority-candidate-limit-exceeded"
        );
    }

    #[test]
    fn lifecycle_fact_mapping_distinguishes_success_failure_fence_and_admission() {
        let facts = [
            RemoteProductionTelemetryFact::RouteSelected,
            RemoteProductionTelemetryFact::QueueAdmitted,
            RemoteProductionTelemetryFact::WorkerAssigned,
            RemoteProductionTelemetryFact::FenceAccepted,
            RemoteProductionTelemetryFact::ExecutionStarted,
            RemoteProductionTelemetryFact::ExecutionCompleted,
            RemoteProductionTelemetryFact::ExecutionFailed,
            RemoteProductionTelemetryFact::OutputAdmitted,
            RemoteProductionTelemetryFact::OutputRejected,
        ];
        let events = facts.into_iter().map(|fact| remote_production_telemetry_event(fact).unwrap()).collect::<Vec<_>>();

        assert_eq!(events[3].category, RemoteTelemetryCategory::RetryFencing);
        assert_eq!(events[3].retry, RemoteTelemetryRetryClass::CurrentFence);
        assert_eq!(events[3].reason, RemoteTelemetryReasonClass::FenceAccepted);
        assert_eq!(events[6].result, RemoteTelemetryResultClass::Failed);
        assert_eq!(events[6].reason, RemoteTelemetryReasonClass::ExecutionFailed);
        assert_eq!(events[8].category, RemoteTelemetryCategory::Admission);
        assert_eq!(events[8].result, RemoteTelemetryResultClass::Rejected);
        assert_eq!(events[8].reason, RemoteTelemetryReasonClass::OutputRejected);
    }

    #[test]
    fn immutable_observability_payload_contains_bounded_lifecycle_events() {
        let telemetry = record_remote_production_telemetry(
            &RemoteTelemetryBuffer::default(),
            RemoteProductionTelemetryFact::OutputAdmitted,
            RemoteTelemetryPolicy::default(),
        );
        let payload = remote_observability_log_payload(&telemetry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        let events = parsed["events"].as_array().expect("bounded lifecycle events are persisted");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["reason"], "output-admitted");
        assert_eq!(parsed["accepted_events"], 1);
        assert_eq!(parsed["dropped_events"], 0);
    }

    #[test]
    fn stale_input_final_chunk_is_rejected_before_transfer_completion() {
        let root = tempfile::tempdir().unwrap();
        let prepared = interactive_transfer_fixture(&root.path().join("spool"));
        let mut receiver =
            interactive_receive_session(&prepared, &root.path().join("state"), &root.path().join("receiver"));
        let demand = RemoteTransferDemandFrame {
            direction: crate::remote_transfer::RemoteTransferDirection::Upload,
            demand: receiver.demand().clone(),
            sender_state: receiver.sender_credit_state(),
        };
        let missing = demand.demand.missing_chunks[0].clone();
        let grant = receiver.credit_for_chunk(&missing).unwrap();
        let mut data = Vec::new();
        crate::remote_transfer::write_remote_transfer_data_chunk(
            &mut data,
            &prepared,
            RemoteTransferPolicy::default(),
            &demand.demand,
            &demand.sender_state,
            grant.clone(),
            &missing,
        )
        .unwrap();
        let acknowledgement = receiver.receive_chunk(std::io::Cursor::new(data)).unwrap();
        let report = receiver.finish().unwrap();
        let mut control = Vec::new();
        for frame in [
            RemoteFrame::TransferCredit {
                transfer: RemoteTransferCreditFrame {
                    direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                    missing,
                    grant,
                },
            },
            RemoteFrame::TransferAcknowledgement {
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                acknowledgement,
            },
            RemoteFrame::TransferComplete {
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                report,
            },
        ] {
            write_remote_control_frame(&mut control, &frame).unwrap();
        }
        let control_len = u64::try_from(control.len()).unwrap();
        let mut reader = std::io::Cursor::new(control);
        let mut validator_calls = 0_u32;
        let mut validate = || {
            validator_calls = validator_calls.checked_add(1).unwrap();
            if validator_calls == 1 {
                Ok(())
            } else {
                Err(RemoteAttemptReasonCode::StaleReportRejected.as_str().to_string())
            }
        };
        let error = send_prepared_remote_transfer_with_fence_validator(
            &mut reader,
            &mut Vec::new(),
            RemoteTransferSendInput {
                prepared: &prepared,
                policy: RemoteTransferPolicy::default(),
                direction: crate::remote_transfer::RemoteTransferDirection::Upload,
                demand_frame: demand,
                interrupt_after_chunks: None,
            },
            &mut validate,
        )
        .unwrap_err();
        assert_eq!(error, RemoteAttemptReasonCode::StaleReportRejected.as_str());
        assert!(reader.position() < control_len);
    }

    #[cfg(unix)]
    #[test]
    fn coordinator_mutation_guard_is_exclusive_and_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let guard = acquire_remote_coordinator_mutation_guard(root.path()).unwrap();
        let busy = acquire_remote_coordinator_mutation_guard(root.path()).unwrap_err();
        assert!(busy.contains("remote-coordinator-mutation-lock-busy"));
        drop(guard);
        drop(acquire_remote_coordinator_mutation_guard(root.path()).unwrap());

        let hostile = root.path().join("hostile");
        fs::create_dir_all(&hostile).unwrap();
        let victim = root.path().join("victim");
        fs::write(&victim, b"unchanged").unwrap();
        symlink(&victim, hostile.join(COORDINATOR_MUTATION_LOCK_FILE)).unwrap();
        let rejected = acquire_remote_coordinator_mutation_guard(&hostile).unwrap_err();
        assert!(rejected.contains("without symlink following"));
        assert_eq!(fs::read(victim).unwrap(), b"unchanged");
    }

    #[test]
    fn trace_context_changes_only_diagnostic_health_not_authoritative_decisions() {
        const TRACEPARENT_A: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        const TRACEPARENT_B: &str = "00-4bf92f3577b34da6a3ce929d0e0e4737-00f067aa0ba902b8-00";
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let command = RemoteStdioCommand {
            binding: RemoteTransportBinding::Stdio,
            program: PathBuf::from("unused"),
            args: Vec::new(),
            input_frames: remote_client_request_frames(&client),
            timeout_secs: DEFAULT_REMOTE_STDIO_TIMEOUT_SECS,
            production_transfer: None,
        };
        let (context_a, health_a) = accept_remote_trace_context(true, Some(TRACEPARENT_A), None);
        let (context_b, health_b) = accept_remote_trace_context(true, Some(TRACEPARENT_B), None);
        let mut command_a = command.clone();
        let mut command_b = command;
        set_remote_diagnostic_trace_context(&mut command_a, context_a).unwrap();
        set_remote_diagnostic_trace_context(&mut command_b, context_b).unwrap();
        let (_, _, auth_a, request_a, _) = command_client_frames(&command_a).unwrap();
        let (_, _, auth_b, request_b, _) = command_client_frames(&command_b).unwrap();
        let mut coordinator_request_a = fixture_remote_operator_coordinator_request(&client);
        coordinator_request_a.request = request_a.clone();
        let mut coordinator_request_b = fixture_remote_operator_coordinator_request(&client);
        coordinator_request_b.request = request_b.clone();
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).unwrap();
        let nonce = fixture_assignment_nonce("trace-invariance");
        let dispatch_a = plan_coordinator_dispatch(&state, &coordinator_request_a, &nonce).unwrap();
        let dispatch_b = plan_coordinator_dispatch(&state, &coordinator_request_b, &nonce).unwrap();
        let response = fixture_builder_response(&client);
        let admission_a =
            validate_remote_builder_response_output_import(&request_a, &client.trusted_output_keys, &response).unwrap();
        let admission_b =
            validate_remote_builder_response_output_import(&request_b, &client.trusted_output_keys, &response).unwrap();

        assert_ne!(health_a.context_digest_blake3, health_b.context_digest_blake3);
        assert_ne!(command_a.input_frames, command_b.input_frames);
        assert_eq!(command_a.binding, command_b.binding);
        assert_eq!(command_a.program, command_b.program);
        assert_eq!(command_a.args, command_b.args);
        assert_eq!(command_a.timeout_secs, command_b.timeout_secs);
        assert_eq!(command_a.production_transfer, command_b.production_transfer);
        assert_eq!(request_a, request_b);
        assert_eq!(
            plan_remote_executable_request(&request_a).unwrap().plan_digest_blake3,
            plan_remote_executable_request(&request_b).unwrap().plan_digest_blake3
        );
        assert_eq!(
            normalized_remote_build_key(&coordinator_request_a).unwrap(),
            normalized_remote_build_key(&coordinator_request_b).unwrap()
        );
        assert_eq!(
            authorize_ticket(&fixture_ticket(), &auth_a, fixture_auth_facts(2)),
            authorize_ticket(&fixture_ticket(), &auth_b, fixture_auth_facts(2))
        );
        assert_eq!(dispatch_a, dispatch_b);
        assert_eq!(admission_a.trust_basis, admission_b.trust_basis);
        assert_eq!(admission_a.builder_signing_key_id, admission_b.builder_signing_key_id);
        assert_eq!(admission_a.output_digest_blake3, admission_b.output_digest_blake3);
        assert_eq!(admission_a, admission_b);
    }

    #[test]
    fn compatible_hello_reaches_authorization() {
        let hello = RemoteHello {
            alpn: REMOTE_PROTOCOL_ALPN.to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: "builder-1".to_string(),
            capabilities: vec!["delta".to_string(), "full".to_string()],
            workspace_policy: None,
        };
        let decision = validate_hello(&hello, "builder-1", &["delta".to_string()]);
        assert!(matches!(decision, ProtocolDecision::Proceed(_)));
    }

    #[test]
    fn protocol_mismatch_fails_closed() {
        let hello = RemoteHello {
            alpn: "wrong".to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: "builder-1".to_string(),
            capabilities: Vec::new(),
            workspace_policy: None,
        };
        let decision = validate_hello(&hello, "builder-1", &[]);
        assert!(matches!(decision, ProtocolDecision::Reject(reason) if reason.contains("unsupported ALPN")));
    }

    #[test]
    fn malformed_request_does_not_consume_ticket() {
        let mut ticket = fixture_ticket();
        let request = ConcreteBuildRequest {
            request_id: "r1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            input_refs: Vec::new(),
            source_input_refs: Vec::new(),
            upload_bytes: 0,
            build_time_limit_secs: 1,
            contains_raw_frontend_eval: true,
            payload: fixture_payload(),
            expected_outputs: fixture_expected_outputs(),
            production_attempt: None,
            transfer_policy: None,
            resource_requirements: None,
            locality_scope: None,
            failure_debug_policy: crunch_build::distributed::RemoteFailureDebugPolicy::default(),
            failure_replay: None,
        };
        assert!(validate_concrete_request(&request, &ticket).is_err());
        redeem_after_queue(&mut ticket, false).unwrap();
        assert_eq!(ticket.uses_remaining, 1);
    }

    #[test]
    fn ci_boundary_concrete_request_passes_validation_without_frontend_eval() {
        // V12: a concrete build request from an external CI system passes
        // validation when it contains a valid derivation payload.
        // r[verify remote_builds.production_ci_build_separation]
        let ticket = fixture_ticket();
        let request = ConcreteBuildRequest {
            request_id: "ci-job-42".to_string(),
            contains_raw_frontend_eval: false,
            ..fixture_request()
        };
        validate_concrete_request(&request, &ticket).unwrap();
    }

    #[test]
    fn ci_boundary_rejects_frontend_eval_before_coordinator_admission() {
        // V12: a request with CI-owned frontend eval fields is rejected
        // before it reaches the coordinator.
        // r[verify remote_builds.production_ci_build_separation]
        let ticket = fixture_ticket();
        let request = ConcreteBuildRequest {
            request_id: "eval-request".to_string(),
            contains_raw_frontend_eval: true,
            ..fixture_request()
        };
        let err = validate_concrete_request(&request, &ticket).unwrap_err();
        assert!(
            err.contains("contains-raw-frontend-eval") || err.contains("raw-frontend-evaluation-rejected"),
            "expected frontend eval rejection, got: {err}"
        );
    }

    #[test]
    fn valid_request_redeems_ticket_once() {
        let mut ticket = fixture_ticket();
        let auth = TicketAuthRequest {
            ticket_id: ticket.id.clone(),
            secret: fixture_ticket_token().to_string(),
            client_endpoint: Some("client-a".to_string()),
            now_unix_s: 2,
        };
        assert_eq!(authorize_ticket(&ticket, &auth, fixture_auth_facts(2)), TicketDecision::Authorized);
        let request = ConcreteBuildRequest {
            request_id: "r1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            input_refs: vec!["input-a".to_string()],
            source_input_refs: Vec::new(),
            upload_bytes: 1,
            build_time_limit_secs: 1,
            contains_raw_frontend_eval: false,
            payload: fixture_payload(),
            expected_outputs: fixture_expected_outputs(),
            production_attempt: None,
            transfer_policy: None,
            resource_requirements: None,
            locality_scope: None,
            failure_debug_policy: crunch_build::distributed::RemoteFailureDebugPolicy::default(),
            failure_replay: None,
        };
        validate_concrete_request(&request, &ticket).unwrap();
        redeem_after_queue(&mut ticket, true).unwrap();
        assert_eq!(ticket.uses_remaining, 0);
    }

    #[test]
    fn production_redemption_requires_durable_commit_before_continuation() {
        let mut state = RemoteTicketState::default();
        let ticket_id = fixture_ticket().id.clone();
        state.tickets.insert(ticket_id.clone(), fixture_ticket());
        let mut failed_commit_observed_redeemed_state = false;
        let error = redeem_and_commit_ticket_state(&mut state, &ticket_id, &mut |candidate| {
            failed_commit_observed_redeemed_state = candidate.tickets[&ticket_id].uses_remaining == 0;
            Err("simulated-fsync-failure".to_string())
        })
        .unwrap_err();

        assert_eq!(error, "remote-ticket-state-commit-failed");
        assert!(failed_commit_observed_redeemed_state);
        assert_eq!(state.tickets[&ticket_id].uses_remaining, 1);

        let mut committed_uses = None;
        redeem_and_commit_ticket_state(&mut state, &ticket_id, &mut |candidate| {
            committed_uses = Some(candidate.tickets[&ticket_id].uses_remaining);
            Ok(())
        })
        .unwrap();
        assert_eq!(committed_uses, Some(0));
        assert_eq!(state.tickets[&ticket_id].uses_remaining, 0);
    }

    #[test]
    fn production_persistence_failure_emits_no_success_frame() {
        let temp = tempfile::tempdir().unwrap();
        let builder = fixture_loopback_builder();
        let verifier_key = fixture_ticket_verifier_key();
        let executor = fixture_local_build_executor();
        let mut client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        client.request.transfer_policy = Some(RemoteTransferPolicy::default());
        let frames = vec![
            RemoteFrame::Hello { hello: client.hello },
            RemoteFrame::AuthTicket { auth: client.auth },
            RemoteFrame::BuildRequest {
                request: client.request,
            },
            RemoteFrame::InputManifest {
                manifest: client.input_manifest,
            },
        ];
        let encoded = encode_frame_stream(&frames);
        let mut reader = &encoded[..];
        let mut writer = Vec::new();
        let context = RemoteProductionServerContext {
            builder: &builder,
            verifier_key: &verifier_key,
            executor: &executor,
            credential_state_dir: temp.path(),
            execution_state_dir: temp.path(),
            authenticated_client_endpoint: None,
        };
        let mut is_request_admitted = false;
        let error = serve_stdio_remote_production_once_with_commit(
            &mut reader,
            &mut writer,
            &context,
            &mut |_auth, request| {
                is_request_admitted = !request.request_id.is_empty();
                Err("remote-ticket-state-commit-failed".to_string())
            },
        )
        .unwrap_err();

        assert_eq!(error, "remote-ticket-state-commit-failed");
        assert!(is_request_admitted);
        assert!(writer.is_empty());
    }

    #[test]
    fn production_concurrent_one_use_contenders_commit_exactly_once() {
        let temp = tempfile::tempdir().unwrap();
        let (verifier_key, auth, request) = persist_transaction_ticket(temp.path(), 1);
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(CONCURRENT_CONTENDER_COUNT));
        let contenders = (0..CONCURRENT_CONTENDER_COUNT)
            .map(|_| {
                let state_dir = temp.path().to_path_buf();
                let verifier_key = verifier_key.clone();
                let auth = auth.clone();
                let request = request.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    commit_remote_ticket_admission_for_state(&state_dir, &verifier_key, None, &auth, &request)
                })
            })
            .collect::<Vec<_>>();
        let results = contenders.into_iter().map(|contender| contender.join().unwrap()).collect::<Vec<_>>();
        let committed_count = results.iter().filter(|result| result.is_ok()).count();
        let rejected_count = results.len().saturating_sub(committed_count);
        let state = load_ticket_state(temp.path()).unwrap();

        assert_eq!(committed_count, 1);
        assert_eq!(rejected_count, 1);
        assert_eq!(state.tickets[TRANSACTION_TICKET_ID].uses_remaining, 0);
    }

    #[test]
    fn production_admission_reloads_state_and_preserves_intervening_mutation() {
        let temp = tempfile::tempdir().unwrap();
        let (verifier_key, auth, request) = persist_transaction_ticket(temp.path(), STALE_TRANSACTION_TICKET_USES);
        commit_remote_ticket_admission_for_state(temp.path(), &verifier_key, None, &auth, &request).unwrap();
        {
            let _guard = crate::remote_credential_state::acquire_ticket_state_mutation_guard(temp.path()).unwrap();
            let mut state = load_ticket_state(temp.path()).unwrap();
            state.invalidated_legacy_ticket_ids.insert(EXTERNAL_LEGACY_TICKET_ID.to_string());
            save_ticket_state(temp.path(), &state).unwrap();
        }
        commit_remote_ticket_admission_for_state(temp.path(), &verifier_key, None, &auth, &request).unwrap();
        let state = load_ticket_state(temp.path()).unwrap();

        assert_eq!(state.tickets[TRANSACTION_TICKET_ID].uses_remaining, 0);
        assert!(state.invalidated_legacy_ticket_ids.contains(EXTERNAL_LEGACY_TICKET_ID));
    }

    #[test]
    fn forged_client_time_and_endpoint_cannot_expand_ticket_authority() {
        let mut expired = fixture_ticket();
        expired.expires_unix_s = EXPIRED_TICKET_UNIX_S;
        let forged = TicketAuthRequest {
            ticket_id: expired.id.clone(),
            secret: fixture_ticket_token().to_string(),
            client_endpoint: Some("client-a".to_string()),
            now_unix_s: FORGED_CLIENT_NOW_UNIX_S,
        };

        assert_eq!(
            authorize_ticket(&expired, &forged, RemoteTicketAuthFacts {
                server_now_unix_s: AUTHORITY_NOW_AFTER_EXPIRY_UNIX_S,
                authenticated_client_endpoint: Some("client-a"),
            },),
            TicketDecision::Reject("ticket-expired".to_string())
        );
        let bound = fixture_ticket();
        assert_eq!(
            authorize_ticket(&bound, &forged, RemoteTicketAuthFacts {
                server_now_unix_s: 2,
                authenticated_client_endpoint: None,
            },),
            TicketDecision::Reject("ticket-client-endpoint-mismatch".to_string())
        );
        assert_eq!(
            authorize_ticket(&bound, &forged, RemoteTicketAuthFacts {
                server_now_unix_s: 2,
                authenticated_client_endpoint: Some("attacker-controlled-peer"),
            },),
            TicketDecision::Reject("ticket-client-endpoint-mismatch".to_string())
        );
    }

    #[test]
    fn ticket_authorization_rejects_revoked_and_expired_tickets() {
        let mut revoked = fixture_ticket();
        revoked.revoked = true;
        let mut expired = fixture_ticket();
        expired.expires_unix_s = 2;
        let auth = TicketAuthRequest {
            ticket_id: revoked.id.clone(),
            secret: fixture_ticket_token().to_string(),
            client_endpoint: Some("client-a".to_string()),
            now_unix_s: 2,
        };

        assert_eq!(
            authorize_ticket(&revoked, &auth, fixture_auth_facts(2)),
            TicketDecision::Reject("ticket-revoked".to_string())
        );
        assert_eq!(
            authorize_ticket(&expired, &auth, fixture_auth_facts(2)),
            TicketDecision::Reject("ticket-expired".to_string())
        );
    }

    #[test]
    fn concrete_request_requires_executable_payload_and_expected_outputs() {
        let ticket = fixture_ticket();
        let mut empty_payload = fixture_request();
        empty_payload.payload = RemoteConcreteBuildPayload::Action {
            action_id: "action-1".to_string(),
            spec_json: String::new(),
        };
        let mut empty_outputs = fixture_request();
        empty_outputs.expected_outputs = Vec::new();

        assert_eq!(
            validate_concrete_request(&empty_payload, &ticket).expect_err("empty payload is rejected"),
            "remote-build-payload-empty"
        );
        assert_eq!(
            validate_concrete_request(&empty_outputs, &ticket).expect_err("empty outputs are rejected"),
            "remote-expected-outputs-empty"
        );
    }

    #[test]
    fn remote_output_digest_changes_when_payload_identity_changes() {
        let mut changed = fixture_request();
        changed.payload = RemoteConcreteBuildPayload::Action {
            action_id: "action-2".to_string(),
            spec_json: fixture_action_spec_json("action-2"),
        };
        let original_digest =
            remote_loopback_output_digest(&fixture_request(), &["input-a".to_string()]).expect("original digest plans");
        let changed_digest =
            remote_loopback_output_digest(&changed, &["input-a".to_string()]).expect("changed digest plans");

        assert_ne!(original_digest, changed_digest);
        assert_eq!(original_digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn action_payload_plans_typed_executor_boundary() {
        let plan = plan_remote_executable_request(&fixture_request()).expect("action request plans");
        assert!(matches!(plan.source, RemoteExecutablePlanSource::Action { .. }));
        assert_eq!(plan.command_args, vec!["builtin:fixture".to_string(), "--emit".to_string()]);
        assert_eq!(plan.system, DEFAULT_REMOTE_ACTION_SYSTEM);
        assert_eq!(plan.expected_outputs, fixture_expected_outputs());
        assert_eq!(plan.plan_digest_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn action_payload_rejects_untyped_json_before_queue_admission() {
        let mut request = fixture_request();
        request.payload = RemoteConcreteBuildPayload::Action {
            action_id: "action-1".to_string(),
            spec_json: "{\"builder\":\"builtin:fixture\"}".to_string(),
        };
        let err = plan_remote_executable_request(&request).expect_err("raw action JSON is not typed enough");
        assert!(err.contains("remote-action-spec-json-invalid"));
    }

    #[test]
    fn derivation_payload_plans_executor_command_and_declared_output() {
        let request = fixture_derivation_request();
        let plan = plan_remote_executable_request(&request).expect("derivation request plans");
        assert!(matches!(plan.source, RemoteExecutablePlanSource::Derivation { .. }));
        assert_eq!(plan.command_args, vec!["/bin/sh".to_string(), "-c".to_string(), "echo hi".to_string()]);
        assert_eq!(plan.command_env.get("outputs"), Some(&"out".to_string()));
        assert_eq!(plan.expected_outputs, request.expected_outputs);
    }

    #[test]
    fn derivation_payload_rejects_stale_declared_output_path() {
        let mut request = fixture_derivation_request();
        request.expected_outputs[0].logical_path =
            Some("/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-stale".to_string());
        let err = plan_remote_executable_request(&request).expect_err("stale output path fails");
        assert_eq!(err, "remote-derivation-output-path-mismatch");
    }

    #[test]
    fn ca_derivation_payload_plans_without_predeclared_output_path() {
        let request = fixture_ca_derivation_request();
        let plan = plan_remote_executable_request(&request).expect("CA derivation request plans");

        assert!(matches!(plan.source, RemoteExecutablePlanSource::Derivation { .. }));
        assert_eq!(plan.expected_outputs[0].name, "out");
        assert!(plan.expected_outputs[0].logical_path.is_none());
        assert!(plan.command_env.get("out").is_some_and(|path| path.starts_with("/mantle/store/")));
    }

    #[test]
    fn ca_derivation_payload_rejects_predeclared_final_output_path() {
        let mut request = fixture_ca_derivation_request();
        request.expected_outputs[0].logical_path =
            Some("/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-ca-stale".to_string());
        let err = plan_remote_executable_request(&request).expect_err("CA final output path is unknown pre-build");

        assert_eq!(err, "remote-derivation-output-path-unexpected");
    }

    #[test]
    fn local_derivation_registry_accepts_declared_derivation_payload() {
        let request = fixture_derivation_request();
        let plan = plan_remote_executable_request(&request).expect("derivation request plans");
        let (drv_path, registry) = local_derivation_registry(&request, &plan).expect("registry builds from payload");
        let drv_abs = drv_path.to_absolute_path_with_prefix(&request.store_prefix);

        assert_eq!(drv_abs, fixture_derivation_drv_path(&request));
        assert!(registry.get_by_drv_path(&drv_abs).is_some());
    }

    #[test]
    fn local_derivation_registry_rejects_action_payload() {
        let request = fixture_request();
        let plan = plan_remote_executable_request(&request).expect("action request plans");
        let err = match local_derivation_registry(&request, &plan) {
            Ok(_) => panic!("actions are not local derivation builds"),
            Err(err) => err,
        };

        assert_eq!(err, "remote-local-executor-action-unsupported");
    }

    #[test]
    fn local_executor_accepts_declared_input_refs_after_upload_boundary() {
        let mut request = fixture_derivation_request();
        request.input_refs = vec!["/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-input.drv".to_string()];

        validate_local_executor_input_refs(&request, &request.input_refs).expect("declared uploaded refs are accepted");
    }

    #[test]
    fn local_executor_rejects_input_refs_missing_from_upload_manifest() {
        let mut request = fixture_derivation_request();
        request.input_refs = vec!["/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-input.drv".to_string()];
        let err = validate_local_executor_input_refs(&request, &[]).expect_err("missing uploaded ref is rejected");

        assert_eq!(err, "remote-local-executor-input-refs-mismatch");
    }

    #[test]
    fn build_outcome_maps_pathinfo_to_remote_execution_output() {
        let request = fixture_derivation_request();
        let plan = plan_remote_executable_request(&request).expect("derivation request plans");
        let expected = plan.expected_outputs[0].clone();
        let mut outputs = BTreeMap::new();
        let expected_path = expected.logical_path.as_deref().expect("fixture expected output path is known");
        outputs.insert(expected.name.clone(), pathinfo_for_logical_path(expected_path));
        let outcome = fixture_build_outcome(&request, outputs);
        let remote = remote_execution_outcome_from_build_outcome(&request, &plan, &outcome)
            .expect("build outcome maps to remote outcome");

        assert_eq!(remote.request_id, request.request_id);
        assert_eq!(remote.plan_digest_blake3, plan.plan_digest_blake3);
        assert_eq!(remote.outputs.len(), 1);
        assert_eq!(remote.outputs[0].path_info, Some(pathinfo_for_logical_path(expected_path)));
        assert_eq!(remote.outputs[0].artifact_attestation_digest_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn ca_build_outcome_binds_final_path_after_execution() {
        let request = fixture_ca_derivation_request();
        let plan = plan_remote_executable_request(&request).expect("CA derivation request plans");
        let expected = plan.expected_outputs[0].clone();
        let final_path_info = importable_pathinfo();
        let final_logical_path = final_path_info.store_path.to_absolute_path_with_prefix(&request.store_prefix);
        let mut outputs = BTreeMap::new();
        outputs.insert(expected.name.clone(), final_path_info.clone());
        let outcome = fixture_build_outcome(&request, outputs);
        let remote = remote_execution_outcome_from_build_outcome(&request, &plan, &outcome)
            .expect("CA build outcome maps to final path");

        assert!(expected.logical_path.is_none());
        assert_eq!(remote.outputs.len(), 1);
        assert_eq!(remote.outputs[0].logical_path, final_logical_path);
        assert_eq!(remote.outputs[0].path_info, Some(final_path_info));
    }

    #[test]
    fn build_outcome_rejects_unexpected_output_path() {
        let request = fixture_derivation_request();
        let plan = plan_remote_executable_request(&request).expect("derivation request plans");
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), importable_pathinfo());
        let outcome = fixture_build_outcome(&request, outputs);
        let err = remote_execution_outcome_from_build_outcome(&request, &plan, &outcome)
            .expect_err("wrong path does not satisfy derivation expectation");

        assert_eq!(err, "remote-local-executor-output-path-mismatch");
    }

    #[test]
    fn secret_owners_and_internal_auth_frame_buffers_zeroize() {
        let mut auth = fixture_auth_request();
        let mut credential = RemoteTicketCredential {
            ticket_id: auth.ticket_id.clone(),
            secret: auth.secret.clone(),
        };
        let frame = RemoteFrame::AuthTicket { auth: auth.clone() };
        let mut encoded = encode_remote_frame_zeroizing(&frame).unwrap();
        let encoded_text = String::from_utf8_lossy(&encoded);

        assert!(encoded_text.contains(fixture_ticket_token()));
        assert!(!format!("{auth:?}").contains(fixture_ticket_token()));
        assert!(!format!("{credential:?}").contains(fixture_ticket_token()));
        auth.zeroize_secret();
        credential.zeroize_secret();
        encoded.zeroize();
        assert!(auth.secret.is_empty());
        assert!(credential.secret.is_empty());
        assert!(encoded.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn ticket_credential_fd_reader_accepts_once_and_rejects_invalid_oversized_and_terminal_inputs() {
        use std::io::Seek as _;
        use std::io::SeekFrom;

        let mut valid = tempfile::tempfile().unwrap();
        writeln!(valid, "ticket-1:{}", fixture_ticket_token()).unwrap();
        valid.seek(SeekFrom::Start(0)).unwrap();
        let credential = read_remote_ticket_credential_from_owned_fd(valid.into_raw_fd()).unwrap();
        assert_eq!(credential.ticket_id, "ticket-1");
        assert_eq!(credential.secret, fixture_ticket_token());

        let mut invalid = tempfile::tempfile().unwrap();
        invalid.write_all(b"not-a-ticket").unwrap();
        invalid.seek(SeekFrom::Start(0)).unwrap();
        assert!(read_remote_ticket_credential_from_owned_fd(invalid.into_raw_fd()).is_err());

        let mut oversized = tempfile::tempfile().unwrap();
        let oversized_bytes = usize::try_from(REMOTE_TICKET_CREDENTIAL_BYTES_MAX.checked_add(1).unwrap()).unwrap();
        oversized.write_all(&vec![b'x'; oversized_bytes]).unwrap();
        oversized.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(
            read_remote_ticket_credential_from_owned_fd(oversized.into_raw_fd()).unwrap_err(),
            "remote-ticket-input-size-limit-exceeded"
        );

        let terminal_fd = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
        assert!(terminal_fd > libc::STDERR_FILENO);
        assert_eq!(unsafe { libc::grantpt(terminal_fd) }, 0);
        assert_eq!(unsafe { libc::unlockpt(terminal_fd) }, 0);
        assert_eq!(unsafe { libc::isatty(terminal_fd) }, 1);
        assert_eq!(
            read_remote_ticket_credential_from_owned_fd(terminal_fd).unwrap_err(),
            "remote-ticket-input-terminal-forbidden"
        );

        let mut pipe_fds = [-1_i32; 2];
        assert_eq!(unsafe { libc::pipe(pipe_fds.as_mut_ptr()) }, 0);
        let timeout_error = read_remote_ticket_credential_from_owned_fd_with_timeout(
            pipe_fds[0],
            std::time::Duration::from_millis(TEST_TICKET_FD_TIMEOUT_MS),
        )
        .unwrap_err();
        assert_eq!(unsafe { libc::close(pipe_fds[1]) }, 0);
        assert_eq!(timeout_error, "remote-ticket-input-read-timeout");
    }

    #[test]
    fn remote_ticket_credential_parses_bearer_token() {
        let credential =
            parse_remote_ticket_credential(&format!("ticket-1:{}", fixture_ticket_token())).expect("ticket parses");

        assert_eq!(credential.ticket_id, "ticket-1");
        assert_eq!(credential.secret, fixture_ticket_token());
    }

    #[test]
    fn remote_ticket_credential_rejects_missing_secret() {
        let err = parse_remote_ticket_credential("ticket-1:").expect_err("empty secret rejected");

        assert_eq!(err, "remote-ticket-secret-empty");
    }

    #[test]
    fn remote_stdio_client_dispatch_plans_derivation_payload_frames() {
        let input = fixture_remote_client_derivation_input();
        let options = fixture_remote_client_options();
        let plan = plan_remote_stdio_client_dispatch(input, &options).expect("client dispatch plans");

        assert_eq!(plan.label, "root");
        assert_eq!(plan.command.binding, RemoteTransportBinding::Stdio);
        assert_eq!(plan.command.program, PathBuf::from("/bin/mantle-remote"));
        assert_eq!(plan.command.args, vec!["serve".to_string()]);
        assert_eq!(plan.command.timeout_secs, DEFAULT_TICKET_BUILD_TIME_SECS);
        assert_eq!(plan.command.input_frames.len(), REMOTE_CLIENT_DISPATCH_FRAME_COUNT);
        assert_eq!(plan.client.hello.endpoint_id, "builder-1");
        assert_eq!(plan.client.trusted_output_keys, vec!["builder-key".to_string()]);
        assert!(matches!(plan.client.request.payload, RemoteConcreteBuildPayload::Derivation { .. }));
        assert!(plan.client.request.input_refs.is_empty());
        assert_eq!(plan.client.input_manifest.request_id, plan.client.request.request_id);
    }

    #[test]
    fn remote_stdio_client_dispatch_frames_non_empty_input_refs_for_upload() {
        let input = fixture_remote_client_derivation_input_with_dep();
        let options = fixture_remote_client_options();
        let plan = plan_remote_stdio_client_dispatch(input, &options).expect("client dispatch plans input refs");
        let expected_upload_bytes =
            remote_input_ref_upload_bytes(&plan.client.request.input_refs).expect("fixture input upload bytes fit");

        assert!(!plan.client.request.input_refs.is_empty());
        assert_eq!(plan.client.request.upload_bytes, expected_upload_bytes);
        assert_eq!(plan.client.uploaded_input_refs, plan.client.request.input_refs);
        assert_eq!(plan.client.input_manifest.input_refs, plan.client.request.input_refs);
        assert!(matches!(
            plan.command.input_frames.last(),
            Some(RemoteFrame::InputUpload { upload })
                if upload.refs == plan.client.request.input_refs && upload.byte_count == expected_upload_bytes
        ));
    }

    #[test]
    fn remote_stdio_client_dispatch_rejects_missing_trusted_keys() {
        let input = fixture_remote_client_derivation_input();
        let mut options = fixture_remote_client_options();
        options.trusted_output_keys.clear();
        let err = plan_remote_stdio_client_dispatch(input, &options).expect_err("trusted keys required");

        assert_eq!(err, "remote-client-trusted-output-keys-empty");
    }

    #[test]
    fn ssh_stdio_command_planner_uses_explicit_argv() {
        let builder = plan_ssh_stdio_builder_command(RemoteSshStdioPlanRequest {
            endpoint_id: "builder-ssh".to_string(),
            ssh_program: PathBuf::from("ssh"),
            destination: "builder.example".to_string(),
            remote_program: "/opt/mantle/bin/mantle".to_string(),
            remote_args: vec![
                "remote".to_string(),
                "serve".to_string(),
                "--binding".to_string(),
                "stdio-once".to_string(),
            ],
        })
        .expect("ssh stdio command plans");

        assert_eq!(builder.endpoint_id, "builder-ssh");
        assert_eq!(builder.program, PathBuf::from("ssh"));
        assert_eq!(builder.args[0], "builder.example");
        assert_eq!(builder.args[1], "/opt/mantle/bin/mantle");
        assert!(builder.args.iter().any(|arg| arg == "stdio-once"));
    }

    #[test]
    fn ssh_stdio_client_dispatch_reuses_stdio_frame_state_machine() {
        let input = fixture_remote_client_derivation_input();
        let mut options = fixture_remote_client_options();
        options.builder = plan_ssh_stdio_builder_command(RemoteSshStdioPlanRequest {
            endpoint_id: "builder-ssh".to_string(),
            ssh_program: PathBuf::from("ssh"),
            destination: "builder.example".to_string(),
            remote_program: "/opt/mantle/bin/mantle".to_string(),
            remote_args: vec!["remote".to_string(), "serve".to_string()],
        })
        .expect("ssh stdio command plans");
        let plan = plan_remote_ssh_stdio_client_dispatch(input, &options).expect("ssh stdio dispatch plans");

        assert_eq!(plan.command.binding, RemoteTransportBinding::SshStdio);
        assert_eq!(plan.command.program, PathBuf::from("ssh"));
        assert_eq!(plan.command.input_frames.len(), REMOTE_CLIENT_DISPATCH_FRAME_COUNT);
        assert!(matches!(plan.command.input_frames.first(), Some(RemoteFrame::Hello { .. })));
        assert!(matches!(plan.command.input_frames.last(), Some(RemoteFrame::InputUpload { .. })));
    }

    #[test]
    fn stdio_child_timeout_is_phase_classified() {
        const TIMEOUT_TEST_SECS: u64 = 1;
        const SLEEP_TEST_SECS: u64 = 2;
        let command = RemoteStdioCommand {
            binding: RemoteTransportBinding::Stdio,
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".to_string(), format!("sleep {SLEEP_TEST_SECS}")],
            input_frames: Vec::new(),
            timeout_secs: TIMEOUT_TEST_SECS,
            production_transfer: None,
        };
        let err = run_stdio_remote_child(&command).expect_err("sleeping child times out");
        let rendered = err.to_string();

        assert!(rendered.contains("timed out"));
        assert!(rendered.contains("TransportSetup"));
    }

    #[test]
    fn ticket_delivery_target_requires_one_explicit_operator_action() {
        assert_eq!(
            ticket_delivery_target(Some(TEST_TICKET_DELIVERY_FD), false).unwrap(),
            TicketDeliveryTarget::CallerOwnedFileDescriptor(TEST_TICKET_DELIVERY_FD)
        );
        assert_eq!(ticket_delivery_target(None, true).unwrap(), TicketDeliveryTarget::InteractiveOperatorTerminal);
        assert!(ticket_delivery_target(None, false).is_err());
        assert!(ticket_delivery_target(Some(TEST_TICKET_DELIVERY_FD), true).is_err());
    }

    #[test]
    fn status_view_omits_ticket_secret_and_verifier() {
        let ticket = fixture_ticket();
        let view = redacted_ticket_view(&ticket);
        let rendered = serde_json::to_string(&view).unwrap();

        assert_eq!(view.verifier_key_id, "ticket-key-1");
        assert!(!rendered.contains("secret"));
        assert!(!rendered.contains("verifier\""));
        assert!(!rendered.contains(fixture_ticket_token()));
    }

    #[test]
    fn missing_inputs_are_derived_from_declared_refs() {
        let missing =
            derive_missing_inputs(&["a".to_string(), "b".to_string(), "c".to_string()], &["b".to_string()]).unwrap();
        assert_eq!(missing, vec!["a".to_string(), "c".to_string()]);
    }

    #[test]
    fn output_trust_requires_trusted_key_and_prefix() {
        let trusted = vec!["builder-key".to_string()];
        assert!(matches!(decide_output_trust("builder-key", &trusted, true), OutputTrustDecision::Accept { .. }));
        assert!(matches!(
            decide_output_trust("other", &trusted, true),
            OutputTrustDecision::Reject(reason) if reason == "untrusted-output-key"
        ));
        assert!(matches!(
            decide_output_trust("builder-key", &trusted, false),
            OutputTrustDecision::Reject(reason) if reason == "store-prefix-mismatch"
        ));
    }

    #[test]
    fn output_trust_uses_key_material_digest_when_available() {
        let trusted = vec!["builder-key:public-material-a".to_string()];
        let decision = decide_output_trust("builder-key:public-material-a", &trusted, true);

        let OutputTrustDecision::Accept { trust_basis, .. } = decision else {
            panic!("matching key material should admit output");
        };
        assert_eq!(trust_basis.key_id, "builder-key:public-material-a");
        assert_eq!(trust_basis.key_material_digest_blake3.as_deref().map(str::len), Some(BLAKE3_HEX_LENGTH_CHARS));
    }

    #[test]
    fn output_trust_rejects_same_name_different_key_material() {
        let trusted = vec!["builder-key:public-material-a".to_string()];
        assert!(matches!(
            decide_output_trust("builder-key:public-material-b", &trusted, true),
            OutputTrustDecision::Reject(reason) if reason == "same-name-different-output-key"
        ));
        assert!(matches!(
            decide_output_trust("builder-key", &trusted, true),
            OutputTrustDecision::Reject(reason) if reason == "output-key-material-missing"
        ));
    }

    #[test]
    fn length_prefixed_frame_roundtrip_and_stdio_pollution_rejected() {
        let frame = RemoteFrame::Hello { hello: fixture_hello() };
        let encoded = encode_remote_frame(&frame).expect("frame encodes");
        let decoded = decode_remote_frame(&encoded).expect("frame decodes");
        assert_eq!(decoded, frame);

        let polluted_stdout = b"human log on stdout\n";
        let err = decode_remote_frame(polluted_stdout).expect_err("unframed stdout is rejected");
        assert!(err.contains("remote-frame"));
    }

    #[test]
    fn oversized_frame_length_is_rejected_before_payload_read() {
        let oversized_len = u32::try_from(MAX_REMOTE_FRAME_BYTES.saturating_add(1)).expect("test max fits u32");
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&oversized_len.to_be_bytes());
        let err = decode_remote_frame(&encoded).expect_err("oversized frame is rejected");
        assert!(err.contains("remote-frame-payload-exceeds"));
    }

    #[test]
    fn remote_frame_read_write_helpers_roundtrip_one_frame() {
        let frame = RemoteFrame::Hello { hello: fixture_hello() };
        let mut encoded = Vec::new();
        write_remote_frame(&mut encoded, &frame).expect("frame writes");
        let decoded = read_remote_frame(std::io::Cursor::new(encoded)).expect("frame reads");
        assert_eq!(decoded, frame);
    }

    #[test]
    fn stdio_child_output_decodes_frame_stream_and_keeps_stderr_diagnostic() {
        let hello = RemoteFrame::Hello { hello: fixture_hello() };
        let done = RemoteFrame::Done {
            request_id: "r1".to_string(),
        };
        let mut stdout = encode_remote_frame(&hello).expect("hello encodes");
        let done_encoded = encode_remote_frame(&done).expect("done encodes");
        stdout.extend_from_slice(&done_encoded);
        let output = RemoteStdioChildOutput {
            stdout: stdout.to_vec(),
            stderr: b"diagnostic on stderr\n".to_vec(),
            status_success: true,
        };
        let transcript = validate_stdio_child_output(&output).expect("stdio output validates");
        assert_eq!(transcript.binding, RemoteTransportBinding::Stdio);
        assert_eq!(transcript.frames, vec![hello, done]);
        assert!(transcript.stderr_summary.contains("diagnostic on stderr"));
    }

    #[test]
    fn stdio_child_human_stdout_is_terminal_protocol_corruption() {
        let output = RemoteStdioChildOutput {
            stdout: b"human log on stdout\n".to_vec(),
            stderr: b"human log belongs here\n".to_vec(),
            status_success: true,
        };
        let failure = validate_stdio_child_output(&output).expect_err("human stdout corrupts protocol");
        assert_eq!(failure.phase, RemoteFailurePhase::RequestValidation);
        assert_eq!(failure.retry_class, RemoteRetryClass::Terminal);
        assert!(failure.reason.contains("remote-frame"));
    }

    #[test]
    fn stdio_child_exit_failure_is_terminal_transport_setup() {
        let output = RemoteStdioChildOutput {
            stdout: Vec::new(),
            stderr: b"failed before handshake\n".to_vec(),
            status_success: false,
        };
        let failure = validate_stdio_child_output(&output).expect_err("failed child is rejected");
        assert_eq!(failure.phase, RemoteFailurePhase::TransportSetup);
        assert_eq!(failure.retry_class, RemoteRetryClass::Terminal);
        assert_eq!(failure.reason, "stdio-child-exit-failed");
    }

    #[test]
    fn stdio_stderr_summary_is_bounded() {
        let oversized = vec![b'x'; MAX_REMOTE_STDIO_STDERR_BYTES.saturating_add(1)];
        let summary = bounded_stderr_summary(&oversized);
        assert!(summary.ends_with(STDERR_TRUNCATION_MARKER));
        assert!(summary.len() > MAX_REMOTE_STDIO_STDERR_BYTES);
    }

    #[test]
    fn stdio_server_once_exchanges_framed_request_and_response() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let input = encode_frame_stream(&remote_client_request_frames(&client));
        let mut stdout = Vec::new();
        let response = serve_stdio_remote_once(
            std::io::Cursor::new(input),
            &mut stdout,
            &builder,
            &mut ticket,
            client.transfer_capabilities,
        )
        .expect("stdio server exchange succeeds");
        let server_frames = decode_remote_frame_stream(&stdout).expect("server stdout decodes");

        assert_eq!(server_frames, response.response_frames);
        assert!(matches!(server_frames.first(), Some(RemoteFrame::AuthOk { .. })));
        assert!(matches!(server_frames.last(), Some(RemoteFrame::Done { .. })));
        let admission =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect("response is import-admissible");

        assert_eq!(response.missing_input_refs, vec!["input-a".to_string()]);
        assert_eq!(response.ticket_uses_remaining, 0);
        assert_eq!(ticket.uses_remaining, 0);
        assert_eq!(admission.output_digest_blake3, response.output_digest_blake3);
        assert_eq!(admission.builder_signing_key_id, "builder-key");
        assert_eq!(admission.transfer, response.transfer);
        assert_eq!(admission.outputs, response.outputs);
        assert_eq!(response.outputs.len(), fixture_expected_outputs().len());
        assert_eq!(response.outputs[0].path_info_signing_key_id, "builder-key");
        assert_eq!(
            Some(response.outputs[0].logical_path.as_str()),
            fixture_expected_outputs()[0].logical_path.as_deref()
        );
        assert_eq!(response.outputs[0].artifact_attestation_digest_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn stdio_server_uses_executor_outcome_for_build_finished_metadata() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let executor = FixedRemoteExecutor {
            output_size_bytes: CUSTOM_REMOTE_OUTPUT_BYTES,
            logical_path_override: None,
        };
        let response = plan_remote_builder_frames_with_executor(
            &builder,
            &mut ticket,
            &remote_client_request_frames(&client),
            client.transfer_capabilities,
            &executor,
        )
        .expect("custom executor response plans");
        let admission =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect("custom executor output admits");

        assert_eq!(ticket.uses_remaining, 0);
        assert_eq!(
            response.execution_plan_digest_blake3,
            plan_remote_executable_request(&client.request).unwrap().plan_digest_blake3
        );
        assert_eq!(response.outputs[0].size_bytes, CUSTOM_REMOTE_OUTPUT_BYTES);
        assert_eq!(response.outputs[0].path_info_signing_key_id, "builder-key");
        assert_eq!(admission.output_digest_blake3, response.output_digest_blake3);
    }

    #[test]
    fn executor_output_identity_mismatch_fails_after_queue_admission() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let executor = FixedRemoteExecutor {
            output_size_bytes: CUSTOM_REMOTE_OUTPUT_BYTES,
            logical_path_override: Some("/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-stale".to_string()),
        };
        let err = plan_remote_builder_frames_with_executor(
            &builder,
            &mut ticket,
            &remote_client_request_frames(&client),
            client.transfer_capabilities,
            &executor,
        )
        .expect_err("executor output identity mismatch fails");

        assert_eq!(err, "remote-execution-output-identity-mismatch");
        assert_eq!(ticket.uses_remaining, 0);
    }

    #[test]
    fn stdio_child_exchange_output_validates_import_admission() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let response = fixture_builder_response(&client);
        let output = RemoteStdioChildOutput {
            stdout: encode_frame_stream(&response.response_frames).to_vec(),
            stderr: b"diagnostic\n".to_vec(),
            status_success: true,
        };
        let report = validate_stdio_child_exchange_output(&client.request, &client.trusted_output_keys, &output)
            .expect("stdio child exchange imports");

        assert_eq!(report.binding, RemoteTransportBinding::Stdio);
        assert_eq!(report.frames, response.response_frames);
        assert_eq!(report.stderr_summary, "diagnostic\n");
        assert_eq!(report.admission.output_digest_blake3, response.output_digest_blake3);
    }

    #[test]
    fn stdio_child_exchange_output_classifies_untrusted_builder_key_as_output_import() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let response = fixture_builder_response(&client);
        let output = RemoteStdioChildOutput {
            stdout: encode_frame_stream(&response.response_frames).to_vec(),
            stderr: Vec::new(),
            status_success: true,
        };
        let err = validate_stdio_child_exchange_output(&client.request, &["other-key".to_string()], &output)
            .expect_err("untrusted remote output must not import");

        assert_eq!(err.phase, RemoteFailurePhase::OutputImport);
        assert_eq!(err.retry_class, RemoteRetryClass::Terminal);
        assert_eq!(err.reason, "untrusted-output-key");
    }

    #[test]
    fn remote_output_admission_rejects_untrusted_builder_key() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let response = fixture_builder_response(&client);
        let err =
            validate_remote_builder_response_output_import(&client.request, &["other-key".to_string()], &response)
                .expect_err("untrusted remote output must not import");

        assert_eq!(err, "untrusted-output-key");
    }

    #[test]
    fn remote_output_admission_rejects_transfer_key_mismatch() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut response = fixture_builder_response(&client);
        rewrite_response_transfer_key(&mut response, "other-key");
        let err =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect_err("mismatched transfer key must not import");

        assert_eq!(err, "remote-transfer-builder-key-mismatch");
    }

    #[test]
    fn remote_output_admission_rejects_malformed_digest() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut response = fixture_builder_response(&client);
        rewrite_response_digest(&mut response, "not-a-blake3-digest");
        let err =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect_err("malformed output digest must not import");

        assert_eq!(err, "remote-output-digest-invalid");
    }

    #[tokio::test]
    async fn stdio_executor_pathinfo_frames_import_durably() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = remote_import_store(temp.path()).await;
        let builder = fixture_loopback_builder();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let executor = PathInfoRemoteExecutor {
            path_info: importable_pathinfo(),
            artifact_attestation_digest_blake3: None,
            nar_payload: None,
        };
        let mut state = RemoteTicketState::default();
        state.tickets.insert("ticket-1".to_string(), fixture_ticket());
        let input = encode_frame_stream(&remote_client_request_frames(&client));

        let response = plan_stdio_remote_once_from_state_with_executor(
            std::io::Cursor::new(input),
            &builder,
            &mut state,
            client.transfer_capabilities,
            &executor,
        )
        .expect("executor pathinfo crosses stdio frame boundary");
        let admission =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect("framed pathinfo output admits");
        let report = import_admitted_remote_outputs(
            &mut store,
            &client.request,
            &admission,
            true,
            Some(crunch_store::GcRootSource::Build),
        )
        .await
        .expect("framed pathinfo output imports");

        assert!(response.outputs[0].path_info.is_some());
        assert!(admission.outputs[0].path_info.is_some());
        assert_eq!(response.transfer_artifacts.len(), 1);
        assert_eq!(response.transfer_artifacts[0].artifact_kind, RemoteOutputTransferArtifactKind::PathInfoJson);
        assert_eq!(response.transfer_artifacts[0].output_name, "out");
        assert_eq!(report.outputs.len(), 1);
        assert_eq!(report.outputs[0].path_info_signing_key_id, "builder-key");
        assert!(store.take_output_substitution_report(&importable_store_path()).is_some());
    }

    #[tokio::test]
    async fn source_input_upload_artifact_materializes_remote_input_bytes() {
        let client_temp = tempfile::tempdir().unwrap();
        let remote_temp = tempfile::tempdir().unwrap();
        let client_store = remote_import_store(client_temp.path()).await;
        let remote_store = remote_import_store(remote_temp.path()).await;
        let source_store_path = importable_store_path();
        let source_ref = source_store_path.to_absolute_path_with_prefix("/mantle/store");
        let client_source_path = source_store_path.to_absolute_path_with_prefix(client_store.output_dir_str());
        std::fs::create_dir_all(std::path::Path::new(&client_source_path).parent().unwrap()).unwrap();
        std::fs::write(&client_source_path, b"remote-source-bytes").unwrap();
        let mut client = fixture_loopback_client(vec![source_ref.clone()], vec!["builder-key".to_string()]);
        client.request.input_refs = vec![source_ref.clone()];
        client.request.source_input_refs = vec![source_ref.clone()];
        client.input_manifest.input_refs = vec![source_ref.clone()];
        client.input_manifest.closure_refs = vec![source_ref.clone()];
        let mut command = RemoteStdioCommand {
            binding: RemoteTransportBinding::Stdio,
            program: PathBuf::from("unused"),
            args: Vec::new(),
            input_frames: remote_client_request_frames(&client),
            timeout_secs: DEFAULT_REMOTE_STDIO_TIMEOUT_SECS,
            production_transfer: None,
        };

        populate_remote_input_upload_artifacts_from_store(&client_store, &mut command, &client.request)
            .await
            .expect("source input artifact is attached");
        let upload = command
            .input_frames
            .iter()
            .find_map(|frame| match frame {
                RemoteFrame::InputUpload { upload } => Some(upload.clone()),
                _ => None,
            })
            .expect("input upload frame present");
        materialize_remote_input_upload(&remote_store, &client.request, &upload)
            .await
            .expect("source input artifact materializes");
        let remote_source_path = source_store_path.to_absolute_path_with_prefix(remote_store.output_dir_str());

        assert_eq!(upload.artifacts.len(), 1);
        assert_eq!(upload.artifacts[0].artifact_kind, RemoteInputUploadArtifactKind::Nar);
        assert_eq!(upload.artifacts[0].input_ref, source_ref);
        assert_eq!(std::fs::read(remote_source_path).unwrap(), b"remote-source-bytes");
    }

    #[tokio::test]
    async fn source_input_upload_reuses_imported_source_state_when_store_path_is_absent() {
        let client_temp = tempfile::tempdir().unwrap();
        let remote_temp = tempfile::tempdir().unwrap();
        let state_temp = tempfile::tempdir().unwrap();
        let payload_temp = tempfile::tempdir().unwrap();
        let client_store = remote_import_store(client_temp.path()).await;
        let remote_store = remote_import_store(remote_temp.path()).await;
        let source_store_path = importable_store_path();
        let source_ref = source_store_path.to_absolute_path_with_prefix("/mantle/store");
        let payload_root = payload_temp.path().join("payload");
        std::fs::create_dir_all(payload_root.join("bin")).unwrap();
        std::fs::write(payload_root.join("bin/tool"), b"source-state-tool").unwrap();
        let record = source_state_toolchain_record(&source_ref);
        let materialized_record =
            crate::source_bundle::materialize_source_record_from_path(&record, &payload_root, true)
                .expect("source-state record is materialized");
        write_imported_source_state_record(state_temp.path(), &materialized_record);
        let mut client = fixture_loopback_client(vec![source_ref.clone()], vec!["builder-key".to_string()]);
        client.request.input_refs = vec![source_ref.clone()];
        client.request.source_input_refs = vec![source_ref.clone()];
        client.input_manifest.input_refs = vec![source_ref.clone()];
        client.input_manifest.closure_refs = vec![source_ref.clone()];
        let mut command = RemoteStdioCommand {
            binding: RemoteTransportBinding::Stdio,
            program: PathBuf::from("unused"),
            args: Vec::new(),
            input_frames: remote_client_request_frames(&client),
            timeout_secs: DEFAULT_REMOTE_STDIO_TIMEOUT_SECS,
            production_transfer: None,
        };

        populate_remote_input_upload_artifacts_from_store_or_source_state(
            &client_store,
            &mut command,
            &client.request,
            state_temp.path(),
        )
        .await
        .expect("source-state source input artifact is attached");
        let upload = command
            .input_frames
            .iter()
            .find_map(|frame| match frame {
                RemoteFrame::InputUpload { upload } => Some(upload.clone()),
                _ => None,
            })
            .expect("input upload frame present");
        materialize_remote_input_upload(&remote_store, &client.request, &upload)
            .await
            .expect("source-state source input artifact materializes");
        let remote_source_path = source_store_path.to_absolute_path_with_prefix(remote_store.output_dir_str());

        assert_eq!(upload.artifacts.len(), 1);
        assert_eq!(upload.artifacts[0].artifact_kind, RemoteInputUploadArtifactKind::Nar);
        assert_eq!(upload.artifacts[0].input_ref, source_ref);
        assert_eq!(std::fs::read(Path::new(&remote_source_path).join("bin/tool")).unwrap(), b"source-state-tool");
    }

    fn source_state_toolchain_record(source_ref: &str) -> crate::source_bundle::SourceRecord {
        let mut metadata = BTreeMap::new();
        metadata.insert("source_kind".to_string(), "pre-existing-store-path".to_string());
        metadata.insert("store_path".to_string(), source_ref.to_string());
        crate::source_bundle::SourceRecord {
            kind: crate::source_bundle::SourceRecordKind::ToolchainSourceRoot,
            identity: "store-path-imported-toolchain".to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata,
            payload_bytes: 0,
            content_blake3: String::new(),
            files: Vec::new(),
        }
    }

    fn write_imported_source_state_record(state_dir: &Path, record: &crate::source_bundle::SourceRecord) {
        let records_dir = state_dir.join("source-bundles").join("records");
        std::fs::create_dir_all(&records_dir).unwrap();
        let record_path = records_dir.join(format!("{}.json", record.content_blake3));
        let record_json = serde_json::to_vec_pretty(record).unwrap();
        std::fs::write(record_path, record_json).unwrap();
    }

    fn source_upload_artifact(request_id: &str, input_ref: &str, payload: &[u8]) -> RemoteInputUploadArtifact {
        RemoteInputUploadArtifact {
            request_id: request_id.to_string(),
            input_ref: input_ref.to_string(),
            artifact_kind: RemoteInputUploadArtifactKind::Nar,
            digest_blake3: blake3::hash(payload).to_hex().to_string(),
            size_bytes: u64::try_from(payload.len()).unwrap(),
            payload: payload.to_vec(),
        }
    }

    fn ready_source_record(source_ref: &str) -> crate::source_bundle::SourceRecord {
        let mut record = source_state_toolchain_record(source_ref);
        record.payload_bytes = 1;
        record.content_blake3 = blake3::hash(b"ready-source").to_hex().to_string();
        record.files = vec![crate::source_bundle::SourceFileEntry {
            path: "bin/tool".to_string(),
            file_type: crate::source_bundle::SourceFileType::Regular,
            executable: true,
            size: 1,
            content_hex: None,
            symlink_target: None,
            chunk_index: None,
            chunk_count: None,
            blake3: blake3::hash(b"x").to_hex().to_string(),
        }];
        record
    }

    #[test]
    fn source_upload_privacy_summary_counts_classes_before_transfer() {
        let source_ref = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-source".to_string();
        let store_ref = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-store".to_string();
        let proof_ref = format!("{REMOTE_UPLOAD_PROOF_PREFIX}self-hosting-proof");
        let uploaded_refs = vec![store_ref, source_ref.clone(), proof_ref];
        let artifact = source_upload_artifact("request-1", &source_ref, b"source-payload");
        let summary = plan_remote_input_upload_privacy_summary(
            &uploaded_refs,
            std::slice::from_ref(&source_ref),
            std::slice::from_ref(&artifact),
            &RemoteInputUploadPrivacyPolicy::default(),
        )
        .expect("privacy summary plans");
        let expected_bytes = remote_input_upload_byte_count(&uploaded_refs, std::slice::from_ref(&artifact)).unwrap();

        assert_eq!(summary.store_objects, 1);
        assert_eq!(summary.source_objects, 1);
        assert_eq!(summary.proof_objects, 1);
        assert_eq!(summary.total_objects, u32::try_from(uploaded_refs.len()).unwrap());
        assert_eq!(summary.total_bytes, expected_bytes);
    }

    #[test]
    fn source_upload_privacy_policy_rejects_secret_class_and_quota() {
        let secret_ref = format!("{REMOTE_UPLOAD_SECRET_DESCRIPTOR_PREFIX}signing-key");
        let secret_err = plan_remote_input_upload_privacy_summary(
            std::slice::from_ref(&secret_ref),
            &[],
            &[],
            &RemoteInputUploadPrivacyPolicy::default(),
        )
        .expect_err("secret descriptor class is disallowed by default");
        assert_eq!(secret_err, "remote-input-upload-class-disallowed");

        let source_ref = "/mantle/store/cccccccccccccccccccccccccccccccc-source".to_string();
        let artifact = source_upload_artifact("request-1", &source_ref, b"source-payload");
        let quota_policy = RemoteInputUploadPrivacyPolicy {
            allowed_classes: RemoteInputUploadPrivacyPolicy::default().allowed_classes,
            max_objects: MAX_REMOTE_INPUT_REFS,
            max_bytes: 1,
        };
        let quota_err = plan_remote_input_upload_privacy_summary(
            std::slice::from_ref(&source_ref),
            std::slice::from_ref(&source_ref),
            std::slice::from_ref(&artifact),
            &quota_policy,
        )
        .expect_err("source upload byte quota is enforced before transfer");
        assert_eq!(quota_err, "remote-input-upload-byte-quota-exceeded");
    }

    #[test]
    fn source_upload_readiness_rejects_stale_or_unsupported_records() {
        let source_ref = "/mantle/store/dddddddddddddddddddddddddddddddd-source";
        let record = ready_source_record(source_ref);
        let readiness =
            validate_remote_source_upload_record(&record, "/mantle/store", source_ref, Some(&record.content_blake3))
                .expect("ready imported source is accepted");
        assert_eq!(readiness.ready_class, "verified-imported-source-state");
        assert_eq!(readiness.store_prefix, "/mantle/store");

        let stale = validate_remote_source_upload_record(&record, "/mantle/store", source_ref, Some("stale-digest"))
            .expect_err("stale source digest rejected");
        assert_eq!(stale, "remote-input-source-digest-stale");

        let mut unsupported = record;
        unsupported.kind = crate::source_bundle::SourceRecordKind::LocalPath;
        let unsupported_err = validate_remote_source_upload_record(&unsupported, "/mantle/store", source_ref, None)
            .expect_err("unsupported source kind rejected");
        assert_eq!(unsupported_err, "remote-input-source-kind-unsupported");
    }

    #[tokio::test]
    async fn source_input_upload_without_store_or_source_state_fails_without_network_fetch() {
        let client_temp = tempfile::tempdir().unwrap();
        let client_store = remote_import_store(client_temp.path()).await;
        let source_ref = importable_store_path().to_absolute_path_with_prefix("/mantle/store");
        let mut client = fixture_loopback_client(vec![source_ref.clone()], vec!["builder-key".to_string()]);
        client.request.input_refs = vec![source_ref.clone()];
        client.request.source_input_refs = vec![source_ref];
        let err = plan_remote_input_upload_artifacts_from_store(&client_store, &client.request)
            .await
            .expect_err("missing source state fails closed");

        assert_eq!(err, "remote-input-source-path-missing");
    }

    #[tokio::test]
    async fn source_input_upload_rejects_tampered_artifact_payload() {
        let client_temp = tempfile::tempdir().unwrap();
        let remote_temp = tempfile::tempdir().unwrap();
        let client_store = remote_import_store(client_temp.path()).await;
        let remote_store = remote_import_store(remote_temp.path()).await;
        let source_store_path = importable_store_path();
        let source_ref = source_store_path.to_absolute_path_with_prefix("/mantle/store");
        let client_source_path = source_store_path.to_absolute_path_with_prefix(client_store.output_dir_str());
        std::fs::create_dir_all(std::path::Path::new(&client_source_path).parent().unwrap()).unwrap();
        std::fs::write(&client_source_path, b"remote-source-bytes").unwrap();
        let mut client = fixture_loopback_client(vec![source_ref.clone()], vec!["builder-key".to_string()]);
        client.request.input_refs = vec![source_ref.clone()];
        client.request.source_input_refs = vec![source_ref.clone()];
        let mut command = RemoteStdioCommand {
            binding: RemoteTransportBinding::Stdio,
            program: PathBuf::from("unused"),
            args: Vec::new(),
            input_frames: remote_client_request_frames(&client),
            timeout_secs: DEFAULT_REMOTE_STDIO_TIMEOUT_SECS,
            production_transfer: None,
        };
        populate_remote_input_upload_artifacts_from_store(&client_store, &mut command, &client.request)
            .await
            .expect("source input artifact is attached");
        let mut upload = command
            .input_frames
            .iter()
            .find_map(|frame| match frame {
                RemoteFrame::InputUpload { upload } => Some(upload.clone()),
                _ => None,
            })
            .expect("input upload frame present");
        upload.artifacts[0].payload[0] = CORRUPTED_TRANSFER_PAYLOAD_BYTE;

        let err = materialize_remote_input_upload(&remote_store, &client.request, &upload)
            .await
            .expect_err("tampered source input artifact fails");
        assert_eq!(err, "input-upload-artifact-digest-mismatch");
    }

    #[tokio::test]
    async fn full_nar_transfer_artifact_materializes_remote_output_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = remote_import_store(temp.path()).await;
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let mut client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        client.transfer_capabilities = RemoteTransferCapabilities::full_only();
        let executor = PathInfoRemoteExecutor {
            path_info: importable_nar_pathinfo(),
            artifact_attestation_digest_blake3: None,
            nar_payload: Some(snix_store::fixtures::NAR_CONTENTS_SYMLINK.to_vec()),
        };

        let response = plan_remote_builder_frames_with_executor(
            &builder,
            &mut ticket,
            &remote_client_request_frames(&client),
            client.transfer_capabilities,
            &executor,
        )
        .expect("pathinfo plus NAR response plans");
        let admission =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect("NAR-framed pathinfo output admits");
        let report = import_admitted_remote_outputs(
            &mut store,
            &client.request,
            &admission,
            true,
            Some(crunch_store::GcRootSource::Build),
        )
        .await
        .expect("NAR-framed pathinfo output imports");
        let exported_path = importable_store_path().to_absolute_path_with_prefix(store.output_dir_str());
        let exported_target = std::fs::read_link(exported_path).expect("symlink output materialized");

        assert_eq!(response.transfer.mode, RemoteTransferMode::Full);
        assert!(
            response
                .transfer_artifacts
                .iter()
                .any(|artifact| artifact.artifact_kind == RemoteOutputTransferArtifactKind::Nar)
        );
        assert!(admission.outputs[0].nar_payload_digest_blake3.is_some());
        assert_eq!(report.outputs.len(), 1);
        assert_eq!(exported_target, std::path::PathBuf::from("/nix/store/somewhereelse"));
    }

    #[test]
    fn remote_output_import_rejects_tampered_transfer_artifact_payload() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let executor = PathInfoRemoteExecutor {
            path_info: importable_pathinfo(),
            artifact_attestation_digest_blake3: None,
            nar_payload: None,
        };
        let mut response = plan_remote_builder_frames_with_executor(
            &builder,
            &mut ticket,
            &remote_client_request_frames(&client),
            client.transfer_capabilities,
            &executor,
        )
        .expect("pathinfo response plans");
        corrupt_first_transfer_artifact_payload(&mut response);

        let err =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect_err("tampered transfer artifact payload fails import validation");

        assert_eq!(err, "remote-output-transfer-artifact-digest-mismatch");
    }

    #[test]
    fn remote_output_import_requires_transfer_artifact_for_framed_pathinfo() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let executor = PathInfoRemoteExecutor {
            path_info: importable_pathinfo(),
            artifact_attestation_digest_blake3: None,
            nar_payload: None,
        };
        let mut response = plan_remote_builder_frames_with_executor(
            &builder,
            &mut ticket,
            &remote_client_request_frames(&client),
            client.transfer_capabilities,
            &executor,
        )
        .expect("pathinfo response plans");
        drop_first_transfer_artifact_frame(&mut response);

        let err =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect_err("missing transfer artifact fails import validation");

        assert_eq!(err, "remote-output-transfer-artifact-missing");
    }

    #[test]
    fn builder_rejects_executor_pathinfo_store_path_mismatch() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut path_info = importable_pathinfo();
        path_info.store_path = mismatched_store_path();
        let executor = PathInfoRemoteExecutor {
            path_info,
            artifact_attestation_digest_blake3: None,
            nar_payload: None,
        };

        let err = plan_remote_builder_frames_with_executor(
            &builder,
            &mut ticket,
            &remote_client_request_frames(&client),
            client.transfer_capabilities,
            &executor,
        )
        .expect_err("executor pathinfo store-path mismatch fails before build-finished frame");

        assert_eq!(err, "remote-execution-output-pathinfo-store-path-mismatch");
    }

    #[test]
    fn builder_rejects_executor_pathinfo_without_signature() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut path_info = importable_pathinfo();
        path_info.signatures.clear();
        let executor = PathInfoRemoteExecutor {
            path_info,
            artifact_attestation_digest_blake3: None,
            nar_payload: None,
        };

        let err = plan_remote_builder_frames_with_executor(
            &builder,
            &mut ticket,
            &remote_client_request_frames(&client),
            client.transfer_capabilities,
            &executor,
        )
        .expect_err("executor unsigned pathinfo fails before build-finished frame");

        assert_eq!(err, "remote-execution-output-pathinfo-unsigned");
    }

    #[tokio::test]
    async fn durable_remote_output_import_persists_signed_pathinfo_attestation_and_report() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = remote_import_store(temp.path()).await;
        let request = fixture_request();
        let admission = importable_admission_report();
        let store_path = importable_store_path();

        let report = import_admitted_remote_outputs(
            &mut store,
            &request,
            &admission,
            true,
            Some(crunch_store::GcRootSource::Build),
        )
        .await
        .expect("remote output imports durably");
        let stored_pathinfo = store.pathinfo_service().get(*store_path.digest()).await.unwrap().unwrap();
        let stored_attestation = store.get_artifact_attestation(&store_path).await.unwrap().unwrap();
        let substitution = store.take_output_substitution_report(&store_path).expect("substitution report");
        let exported_path = store_path.to_absolute_path_with_prefix(store.output_dir_str());

        assert_eq!(report.outputs.len(), 1);
        assert_eq!(report.outputs[0].artifact_attestation_digest_blake3, stored_attestation.digest.to_hex());
        assert_eq!(report.outputs[0].path_info_signing_key_id, "builder-key");
        assert_eq!(stored_pathinfo.store_path, store_path);
        assert_eq!(substitution.mode, crunch_store::OutputSubstitutionMode::Full);
        assert_eq!(substitution.transferred_bytes, admission.transfer.transferred_bytes);
        assert!(std::fs::symlink_metadata(exported_path).is_ok());
    }

    #[test]
    fn durable_remote_output_import_rejects_missing_pathinfo_bundle() {
        let request = fixture_request();
        let mut admission = importable_admission_report();
        admission.outputs[0].path_info = None;
        admission.transfer_artifacts.clear();

        let err = plan_remote_output_import_actions(&request, &admission).expect_err("missing pathinfo fails");
        assert_eq!(err, "remote-output-pathinfo-missing");
    }

    #[test]
    fn durable_remote_output_import_rejects_attestation_digest_mismatch() {
        let request = fixture_request();
        let mut admission = importable_admission_report();
        admission.outputs[0].artifact_attestation_digest_blake3 = blake3::hash(b"wrong-artifact").to_hex().to_string();
        admission.output_digest_blake3 = remote_produced_outputs_content_digest(&admission.outputs);

        let err = plan_remote_output_import_actions(&request, &admission).expect_err("wrong attestation digest fails");
        assert_eq!(err, "remote-output-artifact-attestation-digest-mismatch");
    }

    #[test]
    fn durable_remote_output_import_rejects_pathinfo_signature_key_mismatch() {
        let request = fixture_request();
        let mut admission = importable_admission_report();
        let output_name = admission.outputs[0].name.clone();
        let path_info = admission.outputs[0].path_info.as_mut().expect("pathinfo fixture");
        path_info.signatures = vec![nix_compat::narinfo::Signature::new(
            "other-key".to_string(),
            [OTHER_SIGNATURE_FILL_BYTE; ED25519_SIGNATURE_BYTES],
        )];
        let digest =
            crunch_store::artifact_attestation_digest_for_pathinfo("/mantle/store", path_info, &output_name, None)
                .unwrap();
        admission.outputs[0].artifact_attestation_digest_blake3 = digest.to_hex();
        admission.output_digest_blake3 = remote_produced_outputs_content_digest(&admission.outputs);
        admission.transfer_artifacts = vec![
            pathinfo_json_transfer_artifact(
                &admission.request_id,
                &admission.outputs[0],
                admission.outputs[0].path_info.as_ref().expect("pathinfo remains present"),
            )
            .expect("updated pathinfo transfer artifact"),
        ];

        let err = plan_remote_output_import_actions(&request, &admission).expect_err("wrong signature key fails");
        assert_eq!(err, "remote-output-pathinfo-signing-key-mismatch");
    }

    #[test]
    fn stdio_server_rejects_incomplete_client_sequence_without_redeeming_ticket() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut frames = remote_client_request_frames(&client);
        frames.pop();
        let input = encode_frame_stream(&frames);
        let mut stdout = Vec::new();
        let err = serve_stdio_remote_once(
            std::io::Cursor::new(input),
            &mut stdout,
            &builder,
            &mut ticket,
            client.transfer_capabilities,
        )
        .expect_err("missing upload frame fails");

        assert_eq!(err, "missing-input-upload-frame");
        assert!(stdout.is_empty());
        assert_eq!(ticket.uses_remaining, 1);
    }

    #[test]
    fn stdio_server_state_lookup_redeems_matching_ticket() {
        let builder = fixture_loopback_builder();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut state = RemoteTicketState::default();
        state.tickets.insert("ticket-1".to_string(), fixture_ticket());
        let input = encode_frame_stream(&remote_client_request_frames(&client));
        let response = plan_stdio_remote_once_from_state(
            std::io::Cursor::new(input),
            &builder,
            &mut state,
            client.transfer_capabilities,
        )
        .expect("state-backed stdio planning succeeds");
        let ticket = state.tickets.get("ticket-1").expect("ticket remains present");

        assert_eq!(response.ticket_uses_remaining, 0);
        assert_eq!(ticket.uses_remaining, 0);
        assert_eq!(response.missing_input_refs, vec!["input-a".to_string()]);
    }

    #[test]
    fn stdio_server_state_lookup_rejects_unknown_ticket() {
        let builder = fixture_loopback_builder();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut state = RemoteTicketState::default();
        let input = encode_frame_stream(&remote_client_request_frames(&client));
        let err = plan_stdio_remote_once_from_state(
            std::io::Cursor::new(input),
            &builder,
            &mut state,
            client.transfer_capabilities,
        )
        .expect_err("unknown ticket fails");

        assert_eq!(err, "unknown-remote-ticket-ticket-1");
        assert!(state.tickets.is_empty());
    }

    #[test]
    fn protocol_state_machine_rejects_out_of_order_build_request() {
        let request = fixture_request();
        let frame = RemoteFrame::BuildRequest { request };
        let err = validate_remote_transition(RemoteProtocolPhase::Open, RemoteFrameDirection::ClientToBuilder, &frame)
            .expect_err("build request before hello is invalid");
        assert!(err.contains("unexpected-remote-frame"));
    }

    #[test]
    fn protocol_state_machine_accepts_loopback_order() {
        let sequence = [
            (RemoteFrameDirection::ClientToBuilder, RemoteFrame::Hello { hello: fixture_hello() }),
            (RemoteFrameDirection::ClientToBuilder, RemoteFrame::AuthTicket {
                auth: fixture_auth_request(),
            }),
            (RemoteFrameDirection::BuilderToClient, RemoteFrame::AuthOk {
                auth: RemoteAuthOk {
                    builder_signing_keys: vec!["builder-key".to_string()],
                    accepted_capabilities: vec!["full".to_string()],
                },
            }),
            (RemoteFrameDirection::ClientToBuilder, RemoteFrame::BuildRequest {
                request: fixture_request(),
            }),
            (RemoteFrameDirection::ClientToBuilder, RemoteFrame::InputManifest {
                manifest: fixture_manifest(),
            }),
            (RemoteFrameDirection::BuilderToClient, RemoteFrame::MissingInputs {
                request_id: "r1".to_string(),
                refs: vec!["input-a".to_string()],
            }),
            (RemoteFrameDirection::ClientToBuilder, RemoteFrame::InputUpload {
                upload: RemoteInputUpload {
                    request_id: "r1".to_string(),
                    refs: vec!["input-a".to_string()],
                    byte_count: 1,
                    artifacts: Vec::new(),
                    streamed: false,
                },
            }),
            (RemoteFrameDirection::BuilderToClient, RemoteFrame::BuildQueued {
                request_id: "r1".to_string(),
                session_id: "session-1".to_string(),
            }),
            (RemoteFrameDirection::BuilderToClient, RemoteFrame::BuildStarted {
                request_id: "r1".to_string(),
            }),
            (RemoteFrameDirection::BuilderToClient, RemoteFrame::BuildFinished {
                result: RemoteBuildFinished {
                    request_id: "r1".to_string(),
                    output_digest_blake3: "digest".to_string(),
                    builder_signing_key_id: "builder-key".to_string(),
                    store_prefix: "/mantle/store".to_string(),
                    outputs: Vec::new(),
                },
            }),
            (RemoteFrameDirection::BuilderToClient, RemoteFrame::OutputTransferDone {
                report: full_transfer_report(REMOTE_LOOPBACK_OUTPUT_BYTES, "builder-key", None),
            }),
        ];
        let mut phase = RemoteProtocolPhase::Open;
        for (direction, frame) in sequence {
            phase = validate_remote_transition(phase, direction, &frame).expect("transition succeeds");
        }
        assert_eq!(phase, RemoteProtocolPhase::Done);
    }

    #[test]
    fn loopback_session_uploads_missing_input_and_imports_trusted_output() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let report = run_loopback_remote_session(&builder, &mut ticket, &client).expect("loopback session succeeds");

        assert_eq!(report.binding, RemoteTransportBinding::Loopback);
        assert_eq!(report.missing_input_refs, vec!["input-a".to_string()]);
        assert_eq!(report.uploaded_input_refs, vec!["input-a".to_string()]);
        assert_eq!(report.transfer.mode, RemoteTransferMode::Delta);
        assert_eq!(report.ticket_uses_remaining, 0);
        assert_eq!(ticket.uses_remaining, 0);
        assert!(report.non_claims.iter().any(|claim| claim.contains("does not prove production P2P transport")));
    }

    #[test]
    fn untrusted_loopback_output_key_fails_before_ticket_redemption() {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["other-key".to_string()]);
        let err = run_loopback_remote_session(&builder, &mut ticket, &client).expect_err("untrusted output key fails");

        assert_eq!(err, "untrusted-output-key");
        assert_eq!(ticket.uses_remaining, 1);
    }

    #[test]
    fn upload_set_mismatch_fails_before_queue_admission() {
        let missing = vec!["input-a".to_string()];
        let uploaded = vec!["input-b".to_string()];
        let err = validate_missing_uploads(&missing, &uploaded, 1, DEFAULT_TICKET_UPLOAD_BYTES)
            .expect_err("wrong upload set fails");
        assert_eq!(err, "uploaded-input-set-does-not-match-missing-set");
    }

    #[test]
    fn delta_failure_falls_back_to_full_without_claiming_delta_hit() {
        let mut client = RemoteTransferCapabilities::delta_and_full();
        client.simulate_delta_failure = true;
        let report = plan_output_transfer(
            client,
            RemoteTransferCapabilities::delta_and_full(),
            REMOTE_LOOPBACK_OUTPUT_BYTES,
            "builder-key",
        )
        .expect("full fallback succeeds");

        assert_eq!(report.mode, RemoteTransferMode::Full);
        assert_eq!(report.fallback_reason.as_deref(), Some("delta-transfer-failed"));
        assert_eq!(report.reused_bytes, 0);
    }

    #[test]
    fn failure_classification_and_session_lease_are_bounded() {
        let transport = classify_remote_failure(RemoteFailurePhase::TransportSetup, "connection-reset".to_string());
        let output = classify_remote_failure(RemoteFailurePhase::OutputImport, "untrusted-output-key".to_string());
        let lease = plan_session_lease("session-1", &["input-a".to_string()], &["output-a".to_string()])
            .expect("lease plan succeeds");

        assert_eq!(transport.retry_class, RemoteRetryClass::Retryable);
        assert_eq!(output.retry_class, RemoteRetryClass::Terminal);
        assert_eq!(lease.leased_refs, vec!["input-a".to_string(), "output-a".to_string()]);
        assert!(lease.release_when_done);
    }

    #[test]
    fn coordinator_dispatch_registers_worker_and_dedupes_identical_requests() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_coordinator_request();
        let first = admit_fixture_dispatch(&mut state, &request).expect("first request dispatches");
        let second = admit_fixture_dispatch(&mut state, &request).expect("identical request attaches");

        let RemoteCoordinatorDispatchDecision::Dispatch {
            worker_endpoint_id,
            job_id,
            normalized_build_key,
            ..
        } = first
        else {
            panic!("first coordinator request should dispatch");
        };
        assert_eq!(worker_endpoint_id, "builder-1");
        assert_eq!(state.jobs.len(), 1);
        assert_eq!(state.live_output_claims.get("claim-out"), Some(&normalized_build_key));
        assert!(
            matches!(second, RemoteCoordinatorDispatchDecision::AttachExisting { job_id: attached, .. } if attached == job_id)
        );
    }

    #[test]
    fn concurrent_resource_dispatches_commit_once_and_never_overcommit() {
        let temp = tempfile::tempdir().expect("temporary concurrent coordinator state");
        let mut initial = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        let mut worker = fixture_worker_registration();
        worker.concurrency = TEST_WORKER_CONCURRENCY;
        worker.resource_inventory = Some(fixture_resource_inventory());
        apply_worker_registration(&mut initial, worker).expect("quantified worker registers");
        let first_request = fixture_quantified_coordinator_request("resource-action-a", "resource-claim-a");
        let second_request = fixture_quantified_coordinator_request("resource-action-b", "resource-claim-b");
        let state_dir = temp.path().to_path_buf();
        let (committed_tx, committed_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let first_thread = std::thread::spawn(move || {
            let _guard = acquire_remote_coordinator_mutation_guard(&state_dir).expect("first mutation locks");
            let mut state = load_coordinator_state(&state_dir).expect("first mutation loads");
            let decision = admit_fixture_dispatch(&mut state, &first_request).expect("first resource request plans");
            committed_tx.send(decision).expect("commit signal sends");
            release_rx.recv().expect("release signal arrives");
        });
        let first = committed_rx.recv().expect("first resource request commits");
        let busy = acquire_remote_coordinator_mutation_guard(temp.path())
            .expect_err("concurrent mutation fails closed while committed state is locked");
        release_tx.send(()).expect("first mutation releases");
        first_thread.join().expect("first mutation thread joins");
        let _guard = acquire_remote_coordinator_mutation_guard(temp.path()).expect("retry mutation locks");
        let mut restarted = load_coordinator_state(temp.path()).expect("committed lease reloads");
        let second = admit_fixture_dispatch(&mut restarted, &second_request).expect("second resource request plans");
        let status = coordinator_status_snapshot("builder-1", TEST_WORKER_CONCURRENCY, &restarted, &[], &[])
            .expect("resource status renders");
        let before_shrink = restarted.clone();
        let mut shrunken = fixture_worker_registration();
        shrunken.concurrency = TEST_WORKER_CONCURRENCY;
        let mut shrunken_inventory = fixture_resource_inventory();
        shrunken_inventory.total.cpu_units = TEST_RESOURCE_CPU_UNITS.checked_sub(1).expect("fixture CPU shrinks");
        shrunken.resource_inventory = Some(shrunken_inventory);
        let shrink_error = apply_worker_registration(&mut restarted, shrunken)
            .expect_err("registration cannot shrink below current reservations");
        let mut new_generation = fixture_worker_registration();
        new_generation.concurrency = TEST_WORKER_CONCURRENCY;
        new_generation.resource_inventory = Some(fixture_resource_inventory());
        new_generation.worker_generation =
            new_generation.worker_generation.checked_add(1).expect("fixture worker generation advances");
        let generation_error = apply_worker_registration(&mut restarted, new_generation)
            .expect_err("new worker generation cannot inherit an old lease");

        assert!(matches!(first, RemoteCoordinatorDispatchDecision::Dispatch {
            resource_fit: crunch_build::ResourceFitClass::Exact,
            ..
        }));
        assert!(busy.contains("remote-coordinator-mutation-lock-busy"));
        assert!(
            matches!(second, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == RemoteResourceReasonCode::CpuUnavailable.as_str())
        );
        assert_eq!(restarted.resource_leases.len(), 1);
        assert_eq!(restarted.jobs.len(), 1);
        assert_eq!(status.resource_leases.len(), 1);
        assert!(status.queued_jobs[0].resource_requirements.is_some());
        assert_eq!(status.workers[0].resource_remaining.as_ref().expect("remaining capacity reports").cpu_units, 0);
        assert!(status.non_claims.iter().any(|claim| claim.contains("do not prove tool identity")));
        assert_eq!(shrink_error, RemoteResourceReasonCode::LeaseSnapshotOvercommitted.as_str());
        assert_eq!(generation_error, "remote-resource-job-lease-scope-mismatch");
        assert_eq!(restarted, before_shrink);
    }

    #[test]
    fn resource_lease_restart_preserves_current_fence_and_stale_loss_cannot_release_it() {
        let temp = tempfile::tempdir().expect("temporary restart coordinator state");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        let mut worker = fixture_worker_registration();
        worker.concurrency = TEST_WORKER_CONCURRENCY;
        worker.resource_inventory = Some(fixture_resource_inventory());
        apply_worker_registration(&mut state, worker).expect("quantified worker registers");
        let request = fixture_quantified_coordinator_request("resource-action-restart", "resource-claim-restart");
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("resource request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("quantified request must dispatch");
        };
        let stale = current_production_attempt_binding(&state, &job_id, "builder-1").expect("initial binding exists");
        reassign_coordinator_attempt(
            &mut state,
            &job_id,
            "builder-1",
            RemoteAttemptFailureClass::Retryable,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("resource retry commits replacement lease");
        let before_stale = state.clone();
        let stale_error = mark_coordinator_attempt_lost(&mut state, &stale, RemoteFailurePhase::BuildExecution)
            .expect_err("superseded attempt cannot release current capacity");
        assert_eq!(state, before_stale);
        let current = current_production_attempt_binding(&state, &job_id, "builder-1").expect("current binding exists");
        mark_coordinator_attempt_lost(&mut state, &current, RemoteFailurePhase::BuildExecution)
            .expect("current attempt releases capacity");
        let restarted = load_coordinator_state(temp.path()).expect("released state reloads");
        let remaining = remote_resource_remaining_capacity(
            "builder-1",
            restarted.workers["builder-1"].resource_inventory.as_ref().expect("inventory persists"),
            &[],
        )
        .expect("full capacity recomputes");

        assert_eq!(stale_error, RemoteAttemptReasonCode::StaleReportRejected.as_str());
        assert!(restarted.resource_leases.is_empty());
        assert_eq!(remaining.cpu_units, TEST_RESOURCE_CPU_UNITS);
        assert_eq!(restarted.jobs[&job_id].phase, RemoteCoordinatorJobPhase::Lost);
    }

    #[test]
    fn resource_leases_release_on_completion_cancellation_timeout_and_worker_loss() {
        let mut completed = RemoteCoordinatorState::default();
        let mut worker = fixture_worker_registration();
        worker.concurrency = TEST_WORKER_CONCURRENCY;
        worker.resource_inventory = Some(fixture_resource_inventory());
        apply_worker_registration(&mut completed, worker.clone()).expect("completion worker registers");
        let completion_request = fixture_quantified_coordinator_request("resource-complete", "resource-complete-claim");
        let completion_dispatch = admit_fixture_dispatch(&mut completed, &completion_request)
            .expect("completion resource request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = completion_dispatch else {
            panic!("completion resource request must dispatch");
        };
        complete_current_fixture_attempt(&mut completed, &job_id);
        assert!(completed.resource_leases.is_empty());
        assert_eq!(completed.jobs[&job_id].phase, RemoteCoordinatorJobPhase::Finished);

        let causes = [
            RemoteCoordinatorTerminationCause::Cancellation,
            RemoteCoordinatorTerminationCause::Timeout,
            RemoteCoordinatorTerminationCause::WorkerLoss,
        ];
        for cause in causes {
            let mut state = RemoteCoordinatorState::default();
            apply_worker_registration(&mut state, worker.clone()).expect("termination worker registers");
            let label = cause.as_str();
            let request = fixture_quantified_coordinator_request(label, &format!("{label}-claim"));
            let dispatch = admit_fixture_dispatch(&mut state, &request).expect("termination request dispatches");
            let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
                panic!("termination resource request must dispatch");
            };
            let binding =
                current_production_attempt_binding(&state, &job_id, "builder-1").expect("termination binding exists");
            terminate_coordinator_attempt(&mut state, &binding, cause).expect("termination releases capacity");

            assert!(state.resource_leases.is_empty());
            assert_eq!(state.jobs[&job_id].phase, RemoteCoordinatorJobPhase::Lost);
            assert_eq!(state.jobs[&job_id].short_error.as_deref(), Some(cause.as_str()));
        }
    }

    #[test]
    fn scheduling_quantities_do_not_change_action_key_but_semantic_classes_do() {
        let baseline = fixture_quantified_coordinator_request("resource-identity", "resource-identity-claim");
        let mut quantity_only = baseline.clone();
        let reduced_cpu_units = TEST_RESOURCE_CPU_UNITS.checked_sub(1).expect("fixture CPU quantity reduces");
        quantity_only.resource_requirements.as_mut().expect("requirements exist").quantities.cpu_units =
            reduced_cpu_units;
        quantity_only.request.resource_requirements = quantity_only.resource_requirements.clone();
        let mut semantic_change = baseline.clone();
        let semantic_requirements = semantic_change.resource_requirements.as_mut().expect("requirements exist");
        semantic_requirements.quantities.accelerators[0].name = "amd-gfx942".to_string();
        semantic_requirements.semantic_accelerator_classes = vec!["amd-gfx942".to_string()];
        semantic_change.request.resource_requirements = semantic_change.resource_requirements.clone();
        let baseline_key = normalized_remote_build_key(&baseline).expect("baseline key derives");
        let quantity_key = normalized_remote_build_key(&quantity_only).expect("quantity key derives");
        let semantic_key = normalized_remote_build_key(&semantic_change).expect("semantic key derives");

        assert_eq!(baseline_key, quantity_key);
        assert_ne!(baseline_key, semantic_key);
        assert_ne!(baseline.resource_requirements, quantity_only.resource_requirements);
        assert_eq!(baseline.request.payload, semantic_change.request.payload);
    }

    #[test]
    fn receiver_verified_locality_prefers_only_hard_eligible_current_generation() {
        let mut state = RemoteCoordinatorState::default();
        let mut first = fixture_worker_registration();
        first.endpoint_id = "a-worker".to_string();
        let mut local = fixture_worker_registration();
        local.endpoint_id = "b-worker".to_string();
        apply_worker_registration(&mut state, first).expect("first worker registers");
        apply_worker_registration(&mut state, local.clone()).expect("local worker registers");
        let manifest = fixture_locality_manifest();
        let artifact_id = manifest.manifest.artifacts[0].artifact_id.clone();
        let mut complete_artifacts = BTreeSet::new();
        complete_artifacts.insert(artifact_id);
        let summary = record_receiver_verified_worker_locality(
            &mut state,
            "b-worker",
            local.worker_generation,
            &manifest,
            RemoteTransferReceiverFacts {
                complete_artifact_ids: complete_artifacts,
                requested_content_identity_verified: true,
                required_closure_metadata_verified: true,
                path_info_admitted: true,
                ..RemoteTransferReceiverFacts::default()
            },
        )
        .expect("receiver locality verifies");
        let mut request = fixture_coordinator_request();
        request.locality_scope = Some(summary.scope.clone());
        request.request.locality_scope = request.locality_scope.clone();
        let preferred = plan_coordinator_dispatch(&state, &request, &fixture_assignment_nonce("locality-preferred"))
            .expect("locality placement plans");
        local.output_signing_key_ids = vec!["untrusted-builder-key".to_string()];
        apply_worker_registration(&mut state, local.clone()).expect("trust mismatch registration applies");
        let blocked = plan_coordinator_dispatch(&state, &request, &fixture_assignment_nonce("locality-blocked"))
            .expect("hard eligibility placement plans");
        local.output_signing_key_ids = vec!["builder-key".to_string()];
        local.worker_generation = local.worker_generation.checked_add(1).expect("worker generation advances");
        apply_worker_registration(&mut state, local).expect("new worker generation registers");
        let stale = plan_coordinator_dispatch(&state, &request, &fixture_assignment_nonce("locality-stale"))
            .expect("stale locality placement plans");
        assert!(state.verified_locality_observations.is_empty());
        let stale_key = verified_locality_observation_key(&summary);
        state.verified_locality_observations.insert(stale_key, summary.clone());
        let stale_status = coordinator_status_snapshot("a-worker", DEFAULT_REMOTE_CONCURRENCY, &state, &[], &[])
            .expect_err("stale durable locality cannot be reported as current");

        assert_eq!(summary.content_locality, crunch_build::ContentLocalityClass::FullyPresent);
        assert_eq!(summary.transfer_cost, crunch_build::TransferCostClass::None);
        assert!(
            matches!(preferred, RemoteCoordinatorDispatchDecision::Dispatch { worker_endpoint_id, .. } if worker_endpoint_id == "b-worker")
        );
        assert!(
            matches!(blocked, RemoteCoordinatorDispatchDecision::Dispatch { worker_endpoint_id, .. } if worker_endpoint_id == "a-worker")
        );
        assert_eq!(stale_status, RemoteLocalityReasonCode::WorkerGenerationStale.as_str());
        assert!(
            matches!(stale, RemoteCoordinatorDispatchDecision::Dispatch { worker_endpoint_id, .. } if worker_endpoint_id == "a-worker")
        );
    }

    #[test]
    fn replay_execution_identity_forces_a_new_normalized_job_key() {
        let original = fixture_coordinator_request();
        let mut replay_a = original.clone();
        replay_a.request.failure_replay = Some(RemoteFailureReplayBinding {
            source_bundle_blake3: crunch_build::distributed::RemoteFailureDebugDigest::new(
                "a".repeat(BLAKE3_HEX_LENGTH_CHARS),
            )
            .unwrap(),
            execution_blake3: crunch_build::distributed::RemoteFailureDebugDigest::new(
                "b".repeat(BLAKE3_HEX_LENGTH_CHARS),
            )
            .unwrap(),
        });
        let mut replay_b = replay_a.clone();
        replay_b.request.failure_replay.as_mut().unwrap().execution_blake3 =
            crunch_build::distributed::RemoteFailureDebugDigest::new("c".repeat(BLAKE3_HEX_LENGTH_CHARS)).unwrap();
        let original_key = normalized_remote_build_key(&original).unwrap();
        let replay_a_key = normalized_remote_build_key(&replay_a).unwrap();
        let replay_b_key = normalized_remote_build_key(&replay_b).unwrap();

        assert_ne!(original_key, replay_a_key);
        assert_ne!(replay_a_key, replay_b_key);
        assert_ne!(original_key, replay_b_key);
    }

    #[test]
    fn coordinator_state_reset_cannot_reissue_job_or_attempt_identity() {
        let request = fixture_coordinator_request();
        let mut first_state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut first_state, fixture_worker_registration()).unwrap();
        let first = admit_coordinator_dispatch_with_nonce(
            &mut first_state,
            &request,
            RemoteAttemptRetryPolicy::default(),
            fixture_attempt_time(TEST_ATTEMPT_NOW_UNIX_S),
            fixture_assignment_nonce("coordinator-incarnation-1"),
        )
        .unwrap();
        let RemoteCoordinatorDispatchDecision::Dispatch {
            job_id: first_job_id,
            attempt_id: first_attempt_id,
            ..
        } = first
        else {
            panic!("first coordinator incarnation must dispatch");
        };
        let first_attempt = first_state.jobs[&first_job_id].current_attempt.as_ref().unwrap().clone();
        let stale_report =
            fixture_report_for_attempt(&first_attempt, "stale-after-reset", RemoteAttemptReportPayload::Start);

        let mut replacement_state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut replacement_state, fixture_worker_registration()).unwrap();
        let replacement = admit_coordinator_dispatch_with_nonce(
            &mut replacement_state,
            &request,
            RemoteAttemptRetryPolicy::default(),
            fixture_attempt_time(TEST_ATTEMPT_NOW_UNIX_S),
            fixture_assignment_nonce("coordinator-incarnation-2"),
        )
        .unwrap();
        let RemoteCoordinatorDispatchDecision::Dispatch {
            job_id: replacement_job_id,
            attempt_id: replacement_attempt_id,
            ..
        } = replacement
        else {
            panic!("replacement coordinator incarnation must dispatch");
        };
        let before_stale = replacement_state.clone();
        let rejected = apply_fixture_attempt_report(&mut replacement_state, &stale_report);

        assert_ne!(first_job_id, replacement_job_id);
        assert_ne!(first_attempt_id, replacement_attempt_id);
        assert_eq!(rejected.reason_code, RemoteAttemptReasonCode::JobIdentityMismatch);
        assert_eq!(replacement_state, before_stale);
    }

    #[test]
    fn coordinator_resume_summary_redelivers_finished_result() {
        let request = fixture_coordinator_request();
        let normalized_key = normalized_remote_build_key(&request).expect("request key");
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let start = fixture_attempt_report(&state, &job_id, "resume-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(&mut state, &start);
        let ready = fixture_attempt_report(&state, &job_id, "resume-result", RemoteAttemptReportPayload::ResultReady {
            output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
        });
        apply_fixture_attempt_report(&mut state, &ready);
        let complete =
            fixture_attempt_report(&state, &job_id, "resume-complete", RemoteAttemptReportPayload::Completion {
                output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
            });
        apply_fixture_attempt_report(&mut state, &complete);
        let attempt = state.jobs[&job_id].current_attempt.as_ref().expect("finished attempt exists");
        let mut worker = fixture_worker_registration();
        worker.resumable_jobs.push(RemoteWorkerResumeSummary {
            job_id: job_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            fence_generation: attempt.fence_generation,
            normalized_build_key: normalized_key.clone(),
            phase: RemoteCoordinatorJobPhase::Finished,
            result_available: true,
            log_next_cursor: 0,
        });
        let adopted = apply_worker_registration(&mut state, worker).expect("matching resume summary registers");
        let decision = plan_coordinator_dispatch(&state, &request, &fixture_assignment_nonce("resume-plan"))
            .expect("resumed request plans");

        assert_eq!(adopted, vec![job_id]);
        assert!(
            matches!(decision, RemoteCoordinatorDispatchDecision::RedeliverResult { normalized_build_key, .. } if normalized_build_key == normalized_key)
        );
    }

    #[test]
    fn coordinator_resume_claim_cannot_overwrite_reassigned_durable_owner() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        register_second_fixture_worker(&mut state);
        let request = fixture_coordinator_request();
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let superseded = state.jobs[&job_id].current_attempt.as_ref().expect("attempt exists").clone();
        reassign_coordinator_attempt(
            &mut state,
            &job_id,
            "builder-2",
            RemoteAttemptFailureClass::Retryable,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("replacement persists");
        let mut stale_worker = fixture_worker_registration();
        stale_worker.endpoint_id = "builder-2".to_string();
        stale_worker.resumable_jobs.push(RemoteWorkerResumeSummary {
            job_id: job_id.clone(),
            attempt_id: superseded.attempt_id,
            fence_generation: superseded.fence_generation,
            normalized_build_key: state.jobs[&job_id].normalized_build_key.clone(),
            phase: RemoteCoordinatorJobPhase::Running,
            result_available: false,
            log_next_cursor: 0,
        });
        let before = state.clone();
        let error = apply_worker_registration(&mut state, stale_worker)
            .expect_err("superseded resume claim cannot overwrite current owner");

        assert_eq!(error, RemoteAttemptReasonCode::StaleReportRejected.as_str());
        assert_eq!(state, before);
        assert_eq!(state.jobs[&job_id].assigned_worker_endpoint_id.as_deref(), Some("builder-2"));
    }

    #[test]
    fn coordinator_restart_recovers_each_live_attempt_phase_and_retained_result_disposition() {
        let phases = [
            RemoteAttemptPhase::Queued,
            RemoteAttemptPhase::Running,
            RemoteAttemptPhase::Transferring,
            RemoteAttemptPhase::FinishedUndelivered,
        ];
        for phase in phases {
            let temp = tempfile::tempdir().expect("temp state dir");
            let mut state = RemoteCoordinatorState {
                state_dir: Some(temp.path().to_path_buf()),
                ..RemoteCoordinatorState::default()
            };
            apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
            let request = fixture_coordinator_request();
            let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
            let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
                panic!("fixture request must dispatch");
            };
            let expected = state.jobs[&job_id].current_attempt.as_ref().expect("attempt exists").clone();
            advance_current_fixture_attempt(&mut state, &job_id, phase);
            let restarted = load_coordinator_state(temp.path()).expect("attempt reloads");
            let job = &restarted.jobs[&job_id];
            let attempt = job.current_attempt.as_ref().expect("fenced attempt survives restart");

            assert_eq!(attempt.attempt_id, expected.attempt_id);
            assert_eq!(attempt.fence_generation, expected.fence_generation);
            assert_eq!(attempt.phase, phase);
            assert_eq!(job.output_admission_completed, phase == RemoteAttemptPhase::FinishedUndelivered);
            assert!(!job.result_available);
        }
    }

    #[test]
    fn coordinator_duplicate_current_report_is_durable_idempotent_no_op() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_coordinator_request();
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let start = fixture_attempt_report(&state, &job_id, "idempotent-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(&mut state, &start);
        let log = fixture_attempt_report(&state, &job_id, "idempotent-log", RemoteAttemptReportPayload::LogAppend {
            cursor: TEST_LOG_CURSOR,
            bytes: "one durable line".to_string(),
        });
        let first = apply_fixture_attempt_report(&mut state, &log);
        let expected_state = state.clone();
        let durable_before = fs::read(coordinator_state_path(temp.path())).expect("durable state exists");
        let duplicate = apply_fixture_attempt_report(&mut state, &log);
        let durable_after = fs::read(coordinator_state_path(temp.path())).expect("durable state remains");

        assert_eq!(first.disposition, RemoteAttemptApplyDisposition::Applied);
        assert_eq!(duplicate.disposition, RemoteAttemptApplyDisposition::AlreadyApplied);
        assert_eq!(duplicate.reason_code, RemoteAttemptReasonCode::AlreadyApplied);
        assert_eq!(state, expected_state);
        assert_eq!(durable_after, durable_before);
    }

    #[test]
    fn coordinator_conflicting_event_digest_changes_no_mutable_surface() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_coordinator_request();
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let start = fixture_attempt_report(&state, &job_id, "conflict-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(&mut state, &start);
        let log = fixture_attempt_report(&state, &job_id, "shared-event", RemoteAttemptReportPayload::LogAppend {
            cursor: TEST_LOG_CURSOR,
            bytes: "accepted".to_string(),
        });
        apply_fixture_attempt_report(&mut state, &log);
        let before_conflict = state.clone();
        let attempt = state.jobs[&job_id].current_attempt.as_ref().expect("attempt exists").clone();
        let conflict =
            fixture_report_for_attempt(&attempt, "shared-event", RemoteAttemptReportPayload::TransferCheckpoint {
                checkpoint: TEST_TRANSFER_CHECKPOINT,
                transferred_bytes: TEST_TRANSFERRED_BYTES,
            });
        let rejected = apply_fixture_attempt_report(&mut state, &conflict);

        assert_eq!(rejected.disposition, RemoteAttemptApplyDisposition::Rejected);
        assert_eq!(rejected.reason_code, RemoteAttemptReasonCode::EventDigestConflict);
        assert!(!rejected.output_admission_allowed);
        assert_eq!(state, before_conflict);
    }

    #[test]
    fn coordinator_fenced_output_admission_redelivers_after_restart_and_revalidates() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let response = fixture_builder_response(&client);
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_remote_operator_coordinator_request(&client);
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let start = fixture_attempt_report(&state, &job_id, "admission-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(&mut state, &start);
        let result =
            fixture_attempt_report(&state, &job_id, "admission-result", RemoteAttemptReportPayload::ResultReady {
                output_digest_blake3: response.output_digest_blake3.clone(),
            });
        let admitted = admit_fenced_remote_builder_response(
            &mut state,
            &result,
            true,
            RemoteLogRetentionPolicy::default(),
            &client.request,
            &client.trusted_output_keys,
            &response,
        )
        .expect("current fenced output admits");
        assert!(matches!(admitted, RemoteFencedOutputAdmissionDecision::Admit { .. }));
        assert!(state.jobs[&job_id].output_admission_completed);
        assert!(!state.jobs[&job_id].result_available);

        let mut restarted = load_coordinator_state(temp.path()).expect("admitted result reloads");
        let before_redelivery = restarted.clone();
        let redelivered = admit_fenced_remote_builder_response(
            &mut restarted,
            &result,
            true,
            RemoteLogRetentionPolicy::default(),
            &client.request,
            &client.trusted_output_keys,
            &response,
        )
        .expect("valid duplicate reconstructs admission after restart");
        assert!(matches!(redelivered, RemoteFencedOutputAdmissionDecision::Admit {
            attempt: RemoteCoordinatorAttemptApplyResult {
                disposition: RemoteAttemptApplyDisposition::AlreadyApplied,
                ..
            },
            ..
        }));
        assert_eq!(restarted, before_redelivery);

        let mut invalid_duplicate_response = response;
        invalid_duplicate_response.output_digest_blake3 = "invalid".to_string();
        let error = admit_fenced_remote_builder_response(
            &mut restarted,
            &result,
            true,
            RemoteLogRetentionPolicy::default(),
            &client.request,
            &client.trusted_output_keys,
            &invalid_duplicate_response,
        )
        .expect_err("duplicate response is cryptographically revalidated");
        assert_eq!(error, RemoteAttemptReasonCode::ResultDigestMismatch.as_str());
        assert_eq!(restarted, before_redelivery);
    }

    #[test]
    fn coordinator_stale_output_result_is_rejected_before_cryptographic_admission() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let mut response = fixture_builder_response(&client);
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        register_second_fixture_worker(&mut state);
        let request = fixture_remote_operator_coordinator_request(&client);
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let superseded = state.jobs[&job_id].current_attempt.as_ref().expect("attempt exists").clone();
        reassign_coordinator_attempt(
            &mut state,
            &job_id,
            "builder-2",
            RemoteAttemptFailureClass::Retryable,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("replacement persists");
        let stale =
            fixture_report_for_attempt(&superseded, "stale-output-result", RemoteAttemptReportPayload::ResultReady {
                output_digest_blake3: response.output_digest_blake3.clone(),
            });
        response.output_digest_blake3 = "cryptographically-invalid".to_string();
        let before = state.clone();
        let decision = admit_fenced_remote_builder_response(
            &mut state,
            &stale,
            true,
            RemoteLogRetentionPolicy::default(),
            &client.request,
            &[],
            &response,
        )
        .expect("stale fence rejects before output validation");

        assert!(matches!(decision, RemoteFencedOutputAdmissionDecision::NoAdmission {
            attempt: RemoteCoordinatorAttemptApplyResult {
                reason_code: RemoteAttemptReasonCode::StaleReportRejected,
                ..
            }
        }));
        assert_eq!(state, before);
    }

    #[test]
    fn coordinator_restart_reassignment_fences_every_stale_mutation_and_completion_race() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        register_second_fixture_worker(&mut state);
        let request = fixture_coordinator_request();
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let superseded = state.jobs[&job_id].current_attempt.as_ref().expect("attempt exists").clone();
        let replacement = reassign_coordinator_attempt(
            &mut state,
            &job_id,
            "builder-2",
            RemoteAttemptFailureClass::Retryable,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("replacement persists");
        let RemoteCoordinatorDispatchDecision::Dispatch { fence_generation, .. } = replacement else {
            panic!("retry must dispatch");
        };
        assert!(fence_generation > superseded.fence_generation);
        let mut restarted = load_coordinator_state(temp.path()).expect("coordinator restarts");
        assert_eq!(restarted.jobs[&job_id].current_attempt.as_ref().unwrap().fence_generation, fence_generation);
        reject_all_superseded_report_kinds(&mut restarted, &superseded);
        complete_current_fixture_attempt(&mut restarted, &job_id);
        let completed = restarted.clone();
        let late_completion = fixture_report_for_attempt(
            &superseded,
            "late-after-current-completion",
            RemoteAttemptReportPayload::Completion {
                output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
            },
        );
        let rejected = apply_fixture_attempt_report(&mut restarted, &late_completion);
        let status = coordinator_status_snapshot(
            "coordinator-1",
            DEFAULT_REMOTE_CONCURRENCY,
            &restarted,
            &[rejected.reason_code],
            &[],
        )
        .expect("stale rejection renders in status");
        assert_eq!(rejected.reason_code, RemoteAttemptReasonCode::StaleReportRejected);
        assert_eq!(status.attempt_reason_codes[0], RemoteAttemptReasonCode::StaleReportRejected);
        assert_eq!(restarted, completed);
        let durable = load_coordinator_state(temp.path()).expect("completed attempt reloads");
        assert_eq!(durable.jobs[&job_id].current_attempt.as_ref().unwrap().phase, RemoteAttemptPhase::Completed);
        assert_eq!(
            durable.jobs[&job_id].last_attempt_reason_code,
            Some(RemoteAttemptReasonCode::CurrentAttemptCompleted)
        );
    }

    fn reject_all_superseded_report_kinds(state: &mut RemoteCoordinatorState, superseded: &RemoteAttemptState) {
        let payloads = [
            RemoteAttemptReportPayload::Start,
            RemoteAttemptReportPayload::Heartbeat {
                observed_unix_s: TEST_HEARTBEAT_OBSERVED_UNIX_S,
            },
            RemoteAttemptReportPayload::LogAppend {
                cursor: TEST_LOG_CURSOR,
                bytes: "late log".to_string(),
            },
            RemoteAttemptReportPayload::TransferCheckpoint {
                checkpoint: TEST_TRANSFER_CHECKPOINT,
                transferred_bytes: TEST_TRANSFERRED_BYTES,
            },
            RemoteAttemptReportPayload::ResultReady {
                output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
            },
            RemoteAttemptReportPayload::Failure {
                failure_class: RemoteAttemptFailureClass::Retryable,
                reason_code: RemoteAttemptReasonCode::RetryAllowed,
            },
            RemoteAttemptReportPayload::Completion {
                output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
            },
        ];
        for (event_index, payload) in payloads.into_iter().enumerate() {
            let before = state.clone();
            let event_id = format!("superseded-event-{event_index}");
            let report = fixture_report_for_attempt(superseded, &event_id, payload);
            let result = apply_fixture_attempt_report(state, &report);
            assert_eq!(result.reason_code, RemoteAttemptReasonCode::StaleReportRejected);
            assert!(!result.output_admission_allowed);
            assert_eq!(state, &before);
        }
    }

    #[test]
    fn coordinator_unconfigured_persistence_exposes_no_assignment_or_claim() {
        let mut state = RemoteCoordinatorState {
            allow_volatile_test_state: false,
            ..RemoteCoordinatorState::default()
        };
        let worker = fixture_worker_registration();
        state.workers.insert(worker.endpoint_id.clone(), worker);
        let request = fixture_coordinator_request();
        let before = state.clone();
        let error = admit_fixture_dispatch(&mut state, &request).expect_err("missing persistence rejects dispatch");

        assert_eq!(error, RemoteAttemptReasonCode::PersistenceUnconfigured.as_str());
        assert_eq!(state, before);
        assert!(state.jobs.is_empty());
        assert!(state.live_output_claims.is_empty());
    }

    #[test]
    fn coordinator_persistence_failure_exposes_no_assignment_or_claim() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let blocked_state_path = temp.path().join("not-a-directory");
        fs::write(&blocked_state_path, "blocked").expect("blocking file writes");
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("in-memory worker registers");
        state.state_dir = Some(blocked_state_path);
        let request = fixture_coordinator_request();
        let before = state.clone();
        let error = admit_fixture_dispatch(&mut state, &request).expect_err("failed persistence rejects dispatch");

        assert!(error.starts_with(RemoteAttemptReasonCode::PersistenceFailed.as_str()));
        assert_eq!(state, before);
        assert!(state.jobs.is_empty());
        assert!(state.live_output_claims.is_empty());
    }

    #[test]
    fn coordinator_load_migrates_legacy_unfenced_result_fail_closed() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let job_id = "legacy-job";
        let legacy = serde_json::json!({
            "workers": {},
            "jobs": {
                (job_id): {
                    "job_id": job_id,
                    "normalized_build_key": "a".repeat(BLAKE3_HEX_LENGTH_CHARS),
                    "assigned_worker_endpoint_id": "legacy-worker",
                    "phase": "finished",
                    "live_output_claims": ["legacy-output"],
                    "result_available": true,
                    "lost_phase": null,
                    "log_start_cursor": 0,
                    "log_next_cursor": 0,
                    "short_error": null
                }
            },
            "live_output_claims": { "legacy-output": "a".repeat(BLAKE3_HEX_LENGTH_CHARS) },
            "logs": {
                (job_id): [{
                    "cursor": 7,
                    "bytes": "legacy diagnostic bytes",
                    "event_id": null,
                    "payload_digest": null
                }]
            }
        });
        fs::write(
            coordinator_state_path(temp.path()),
            serde_json::to_vec_pretty(&legacy).expect("legacy state renders"),
        )
        .expect("legacy state writes");
        let state = load_coordinator_state(temp.path()).expect("legacy state migrates");
        let job_id = RemoteJobId::new(job_id).expect("legacy job id");
        let job = &state.jobs[&job_id];
        let persisted = fs::read_to_string(coordinator_state_path(temp.path())).expect("migration persists");

        assert_eq!(job.phase, RemoteCoordinatorJobPhase::Lost);
        assert!(!job.result_available);
        assert!(job.current_attempt.is_none());
        assert_eq!(job.last_attempt_reason_code, Some(RemoteAttemptReasonCode::LegacyStateRejected));
        let migration = state.legacy_log_migration.as_ref().expect("legacy logs receive a bounded non-claim summary");
        let persisted_json: serde_json::Value = serde_json::from_str(&persisted).expect("migrated state parses");
        assert!(state.live_output_claims.is_empty());
        assert!(persisted.contains("legacy-state-rejected"));
        assert_eq!(migration.legacy_job_count, 1);
        assert_eq!(migration.legacy_chunk_count, 1);
        assert_eq!(migration.classification, LEGACY_LOG_MIGRATION_CLASSIFICATION);
        assert_eq!(migration.non_claim, LEGACY_LOG_MIGRATION_NON_CLAIM);
        assert!(persisted_json.get("logs").is_none());
        assert!(job.immutable_log.is_none());
    }

    #[test]
    fn coordinator_load_rejects_corrupt_fenced_projection_fail_closed() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_coordinator_request();
        let dispatch = admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let mut encoded = serde_json::to_value(&state).expect("coordinator state serializes");
        let encoded_job = encoded["jobs"]
            .as_object_mut()
            .and_then(|jobs| jobs.values_mut().next())
            .expect("encoded job exists");
        encoded_job["current_attempt"]["fence_generation"] = serde_json::Value::from(0_u64);
        fs::write(
            coordinator_state_path(temp.path()),
            serde_json::to_vec_pretty(&encoded).expect("corrupt state renders"),
        )
        .expect("corrupt state writes");
        let loaded = load_coordinator_state(temp.path()).expect("corrupt state fails closed");
        let job = &loaded.jobs[&job_id];

        assert_eq!(job.phase, RemoteCoordinatorJobPhase::Lost);
        assert!(job.current_attempt.is_none());
        assert_eq!(job.last_attempt_reason_code, Some(RemoteAttemptReasonCode::DurableStateInvalid));
        assert!(!job.result_available);
    }

    #[test]
    fn coordinator_load_rejects_legacy_attempt_without_assignment_nonce() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).unwrap();
        let request = fixture_coordinator_request();
        let dispatch = admit_fixture_dispatch(&mut state, &request).unwrap();
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let mut encoded = serde_json::to_value(&state).unwrap();
        let encoded_job = encoded["jobs"].as_object_mut().unwrap().values_mut().next().unwrap();
        encoded_job["current_attempt"].as_object_mut().unwrap().remove("assignment_nonce");
        fs::write(coordinator_state_path(temp.path()), serde_json::to_vec_pretty(&encoded).unwrap()).unwrap();

        let loaded = load_coordinator_state(temp.path()).expect("legacy attempt fails closed");
        let job = &loaded.jobs[&job_id];

        assert_eq!(job.phase, RemoteCoordinatorJobPhase::Lost);
        assert!(job.current_attempt.is_none());
        assert_eq!(job.last_attempt_reason_code, Some(RemoteAttemptReasonCode::DurableStateInvalid));
    }

    #[test]
    fn coordinator_load_rejects_attempt_id_not_derived_from_durable_nonce() {
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).unwrap();
        let request = fixture_coordinator_request();
        let dispatch = admit_fixture_dispatch(&mut state, &request).unwrap();
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let mut encoded = serde_json::to_value(&state).unwrap();
        let encoded_job = encoded["jobs"].as_object_mut().unwrap().values_mut().next().unwrap();
        encoded_job["current_attempt"]["assignment_nonce"] =
            serde_json::Value::String(fixture_assignment_nonce("tampered-nonce").as_str().to_string());
        fs::write(coordinator_state_path(temp.path()), serde_json::to_vec_pretty(&encoded).unwrap()).unwrap();

        let loaded = load_coordinator_state(temp.path()).expect("mismatched attempt derivation fails closed");
        let job = &loaded.jobs[&job_id];

        assert_eq!(job.phase, RemoteCoordinatorJobPhase::Lost);
        assert!(job.current_attempt.is_none());
        assert_eq!(job.last_attempt_reason_code, Some(RemoteAttemptReasonCode::DurableStateInvalid));
    }

    #[test]
    fn coordinator_rejects_untrusted_worker_key_without_using_coordinator_as_trust_root() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let mut request = fixture_coordinator_request();
        request.trusted_output_keys = vec!["other-key".to_string()];
        let decision = plan_coordinator_dispatch(&state, &request, &fixture_assignment_nonce("untrusted-plan"))
            .expect("untrusted worker plans terminally");

        assert!(
            matches!(decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "output-trust-mismatch")
        );
        assert_eq!(
            worker_satisfies_request(state.workers.get("builder-1").unwrap(), &request),
            Err("output-trust-mismatch".to_string())
        );
    }

    #[test]
    fn coordinator_rejects_conflicting_live_output_claim_instead_of_deduping() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_coordinator_request();
        admit_fixture_dispatch(&mut state, &request).expect("first request dispatches");
        let mut conflict = fixture_coordinator_request();
        conflict.request.payload = RemoteConcreteBuildPayload::Action {
            action_id: "action-2".to_string(),
            spec_json: fixture_action_spec_json("action-2"),
        };
        let decision = plan_coordinator_dispatch(&state, &conflict, &fixture_assignment_nonce("conflict-plan"))
            .expect("conflict plans");

        assert!(
            matches!(decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "live-output-claim-conflict")
        );
    }

    #[test]
    fn coordinator_rejects_capability_store_prefix_resource_and_upload_mismatches() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");

        let mut feature_mismatch = fixture_coordinator_request();
        feature_mismatch.required_features = vec!["gpu".to_string()];
        let feature_decision =
            plan_coordinator_dispatch(&state, &feature_mismatch, &fixture_assignment_nonce("feature-plan"))
                .expect("feature mismatch plans");
        assert!(
            matches!(feature_decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "remote-worker-feature-mismatch")
        );

        let mut prefix_state = RemoteCoordinatorState::default();
        let mut prefix_worker = fixture_worker_registration();
        prefix_worker.logical_store_prefixes = vec!["/nix/store".to_string()];
        apply_worker_registration(&mut prefix_state, prefix_worker).expect("prefix worker registers");
        let store_prefix_mismatch = fixture_coordinator_request();
        let store_decision =
            plan_coordinator_dispatch(&prefix_state, &store_prefix_mismatch, &fixture_assignment_nonce("store-plan"))
                .expect("store mismatch plans");
        assert!(
            matches!(store_decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "remote-worker-store-prefix-mismatch")
        );

        let first = fixture_coordinator_request();
        admit_fixture_dispatch(&mut state, &first).expect("first request dispatches");
        let mut second = fixture_coordinator_request();
        second.request.payload = RemoteConcreteBuildPayload::Action {
            action_id: "action-2".to_string(),
            spec_json: fixture_action_spec_json("action-2"),
        };
        second.live_output_claims = vec!["claim-second".to_string()];
        let resource_decision = plan_coordinator_dispatch(&state, &second, &fixture_assignment_nonce("resource-plan"))
            .expect("resource mismatch plans");
        assert!(
            matches!(resource_decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "worker-concurrency-limit")
        );

        let mut oversized_upload = fixture_coordinator_request();
        oversized_upload.request.upload_bytes = MAX_REMOTE_UPLOAD_BYTES.saturating_add(1);
        let upload_error =
            validate_coordinator_build_request(&oversized_upload).expect_err("oversized upload rejected");
        assert_eq!(upload_error, "remote-coordinator-upload-byte-limit-exceeded");
    }

    #[test]
    fn coordinator_immutable_log_restarts_replays_and_retains_bounded_segments() {
        const APPEND_COUNT: u64 = 4;
        const RETAINED_COUNT: usize = 2;
        const RETAINED_BYTES: u64 = 64;
        let temp = tempfile::tempdir().expect("temp state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let dispatch = admit_fixture_dispatch(&mut state, &fixture_coordinator_request()).expect("request dispatches");
        let RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } = dispatch else {
            panic!("fixture request must dispatch");
        };
        let start = fixture_attempt_report(&state, &job_id, "immutable-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(&mut state, &start);
        let retention = RemoteLogRetentionPolicy {
            max_chunks: RETAINED_COUNT,
            max_bytes: RETAINED_BYTES,
        };
        for cursor in 0..APPEND_COUNT {
            let report = fixture_attempt_report(
                &state,
                &job_id,
                &format!("immutable-log-{cursor}"),
                RemoteAttemptReportPayload::LogAppend {
                    cursor,
                    bytes: format!("line-{cursor}"),
                },
            );
            let applied = apply_coordinator_attempt_report(
                &mut state,
                &report,
                RemoteAttemptAuthorizationFacts {
                    worker_authorized: true,
                    output_admission_authorized: true,
                },
                retention,
            )
            .expect("immutable log report applies");
            assert_eq!(applied.disposition, RemoteAttemptApplyDisposition::Applied);
        }
        let restarted = load_coordinator_state(temp.path()).expect("coordinator and immutable log restart");
        let summary = restarted.jobs[&job_id].immutable_log.as_ref().expect("immutable log summary persists");
        let replay = replay_coordinator_attempt_log(&restarted, &job_id, RemoteAttemptLogReplayRequest {
            from_cursor: summary.retained_start_cursor,
            record_count_max: u32::try_from(RETAINED_COUNT).unwrap(),
            payload_bytes_max: RETAINED_BYTES,
        })
        .expect("retained immutable log replays");

        assert_eq!(summary.next_cursor, APPEND_COUNT);
        assert_eq!(summary.retained_start_cursor, APPEND_COUNT - u64::try_from(RETAINED_COUNT).unwrap());
        assert_eq!(summary.retained_record_count, u32::try_from(RETAINED_COUNT).unwrap());
        assert!(summary.truncated);
        assert!(summary.truncation_anchor_blake3.is_some());
        assert_eq!(replay.records.len(), RETAINED_COUNT);
        assert_eq!(replay.next_cursor, APPEND_COUNT);
        assert!(!replay.has_more);
    }

    #[test]
    fn coordinator_replay_rejects_sequence_cursor_previous_digest_event_and_payload_tamper() {
        let digest = "b".repeat(BLAKE3_HEX_LENGTH_CHARS);
        assert_persisted_segment_tamper_rejected(|value| {
            value["records"][0]["sequence"] = serde_json::Value::from(1_u64);
        });
        assert_persisted_segment_tamper_rejected(|value| {
            value["records"][0]["cursor"] = serde_json::Value::from(1_u64);
        });
        assert_persisted_segment_tamper_rejected(|value| {
            value["records"][0]["previous_record_blake3"] = serde_json::Value::String(digest);
        });
        assert_persisted_segment_tamper_rejected(|value| {
            value["records"][0]["event_id"] = serde_json::Value::String("changed-event".to_string());
        });
        assert_persisted_segment_tamper_rejected(|value| {
            value["records"][0]["payload"] = serde_json::json!([116, 97, 109, 112, 101, 114]);
        });
    }

    #[test]
    fn coordinator_replay_rejects_tampered_anchor_and_after_head_cursor_without_state_mutation() {
        const RETAINED_CHUNKS: usize = 2;
        const RETAINED_BYTES: u64 = 128;
        let retention = RemoteLogRetentionPolicy {
            max_chunks: RETAINED_CHUNKS,
            max_bytes: RETAINED_BYTES,
        };
        let (temp, state, job_id) = persisted_log_fixture(&["one", "two", "three"], retention);
        let summary = state.jobs[&job_id].immutable_log.as_ref().expect("log summary exists");
        let anchor_digest = summary.truncation_anchor_blake3.as_ref().expect("retention anchor exists");
        let anchor_path =
            crate::remote_attempt_log_store::remote_attempt_log_anchor_path(temp.path(), &summary.scope, anchor_digest);
        let state_before = state.clone();
        let control_before = fs::read(coordinator_state_path(temp.path())).unwrap();
        let after_head = replay_coordinator_attempt_log(&state, &job_id, RemoteAttemptLogReplayRequest {
            from_cursor: summary.next_cursor.checked_add(1).unwrap(),
            record_count_max: u32::try_from(RETAINED_CHUNKS).unwrap(),
            payload_bytes_max: RETAINED_BYTES,
        })
        .expect_err("after-head cursor fails closed");
        fs::write(&anchor_path, b"{}").unwrap();
        let anchor_error = replay_coordinator_attempt_log(&state, &job_id, RemoteAttemptLogReplayRequest {
            from_cursor: summary.retained_start_cursor,
            record_count_max: u32::try_from(RETAINED_CHUNKS).unwrap(),
            payload_bytes_max: RETAINED_BYTES,
        })
        .expect_err("tampered anchor fails closed");

        assert_eq!(after_head, RemoteAttemptLogReasonCode::CursorAfterHead.as_str());
        assert!(anchor_error.contains("anchor-parse-failed") || anchor_error.contains("anchor-content-mismatch"));
        assert_eq!(state, state_before);
        assert_eq!(fs::read(coordinator_state_path(temp.path())).unwrap(), control_before);
    }

    #[test]
    fn stale_fence_log_append_never_touches_superseded_attempt_manifest() {
        let (temp, mut state, job_id) = persisted_log_fixture(&["current"], RemoteLogRetentionPolicy::default());
        register_second_fixture_worker(&mut state);
        let superseded = state.jobs[&job_id].current_attempt.as_ref().unwrap().clone();
        let old_scope = state.jobs[&job_id].immutable_log.as_ref().unwrap().scope.clone();
        let manifest_path = crate::remote_attempt_log_store::remote_attempt_log_manifest_path(temp.path(), &old_scope);
        let manifest_before = fs::read(&manifest_path).unwrap();
        reassign_coordinator_attempt(
            &mut state,
            &job_id,
            "builder-2",
            RemoteAttemptFailureClass::Retryable,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("replacement attempt persists");
        let stale = fixture_report_for_attempt(&superseded, "stale-fence-log", RemoteAttemptReportPayload::LogAppend {
            cursor: 1,
            bytes: "must-not-append".to_string(),
        });
        let state_before = state.clone();
        let control_before = fs::read(coordinator_state_path(temp.path())).unwrap();
        let rejected = apply_fixture_attempt_report(&mut state, &stale);
        let telemetry = record_remote_production_telemetry(
            &RemoteTelemetryBuffer::default(),
            RemoteProductionTelemetryFact::StaleFenceRejected,
            RemoteTelemetryPolicy::default(),
        );
        let diagnostic = append_remote_rejected_production_observability_log(
            temp.path(),
            &RemoteProductionAttemptBinding {
                job_id: superseded.job_id.clone(),
                attempt_id: superseded.attempt_id.clone(),
                fence_generation: superseded.fence_generation,
            },
            &telemetry,
        )
        .expect("stale-fence rejection diagnostic persists separately");

        assert_eq!(rejected.disposition, RemoteAttemptApplyDisposition::Rejected);
        assert_eq!(rejected.reason_code, RemoteAttemptReasonCode::StaleReportRejected);
        assert_eq!(telemetry.events[0].retry, RemoteTelemetryRetryClass::StaleFence);
        assert_eq!(telemetry.events[0].reason, RemoteTelemetryReasonClass::FenceRejected);
        assert_ne!(diagnostic.scope, old_scope);
        assert_eq!(fs::read(&manifest_path).unwrap(), manifest_before);
        assert_eq!(fs::read(coordinator_state_path(temp.path())).unwrap(), control_before);
        assert_eq!(state, state_before);
    }

    #[test]
    fn coordinator_status_redacts_ticket_secrets_and_splits_phases() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_coordinator_request();
        admit_fixture_dispatch(&mut state, &request).expect("request dispatches");
        let ticket = fixture_ticket();
        let snapshot = coordinator_status_snapshot("coordinator-1", 1, &state, &[], std::slice::from_ref(&ticket))
            .expect("status snapshot renders");
        let rendered = serde_json::to_string(&snapshot).expect("status serializes");

        assert_eq!(snapshot.worker_count, 1);
        assert_eq!(snapshot.workers[0].output_signing_key_count, 1);
        assert_eq!(snapshot.queued_jobs.len(), 1);
        assert!(snapshot.queued_jobs[0].attempt_id.is_some());
        assert_eq!(snapshot.queued_jobs[0].fence_generation, Some(RemoteFenceGeneration::INITIAL));
        assert_eq!(snapshot.log_cursors.len(), 1);
        assert!(snapshot.active_jobs.is_empty());
        assert_eq!(snapshot.tickets[0].verifier_key_id, "ticket-key-1");
        assert!(!rendered.contains("\"secret\""));
        assert!(!rendered.contains("\"verifier\""));
        assert!(!rendered.contains(fixture_ticket_token()));
    }

    #[test]
    fn coordinator_status_bounds_failure_text_and_log_cursors() {
        let request = fixture_coordinator_request();
        let normalized_key = normalized_remote_build_key(&request).expect("request key");
        let mut state = RemoteCoordinatorState::default();
        let job_id = RemoteJobId::new("lost-job").expect("fixture job id");
        state.jobs.insert(job_id.clone(), RemoteCoordinatorJobSummary {
            job_id: job_id.clone(),
            normalized_build_key: normalized_key,
            assigned_worker_endpoint_id: Some("worker-secret".to_string()),
            phase: RemoteCoordinatorJobPhase::Lost,
            live_output_claims: Vec::new(),
            result_available: false,
            lost_phase: Some(RemoteFailurePhase::TransportSetup),
            immutable_log: Some(fixture_log_control_summary(7, 9, true)),
            observability_health: None,
            failure_debug: Some(RemoteFailureDebugStatus {
                bundle_ref: format!("remote-failure-debug:{}", "a".repeat(BLAKE3_HEX_LENGTH_CHARS)),
                capture_outcome_code: "metadata-only".to_string(),
                cleanup_status_code: "cleanup-complete".to_string(),
                immutable_log_available: true,
                non_claim: crunch_build::distributed::REMOTE_FAILURE_DEBUG_NON_CLAIM.to_string(),
                worker_bundle_ref: None,
                replay_attempt_identity: None,
                replay_comparison_class: None,
            }),
            short_error: Some(format!("diagnostic\u{0007}{}", "x".repeat(MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES))),
            current_attempt: None,
            transfer_checkpoint: None,
            transferred_bytes: 0,
            output_admission_completed: false,
            last_attempt_reason_code: Some(RemoteAttemptReasonCode::LegacyStateRejected),
            resource_requirements: None,
            resource_lease_id_blake3: None,
            resource_fit: None,
            locality: None,
        });
        let snapshot =
            coordinator_status_snapshot("coordinator-1", 1, &state, &[], &[]).expect("status snapshot renders");
        let rendered = serde_json::to_string(&snapshot).expect("status serializes");

        assert_eq!(snapshot.recent_failures.len(), 1);
        assert_eq!(snapshot.recent_failures[0].lost_phase, Some(RemoteFailurePhase::TransportSetup));
        assert_eq!(snapshot.log_cursors[0].retained_start_cursor, 7);
        assert_eq!(snapshot.log_cursors[0].next_cursor, 9);
        assert!(snapshot.log_cursors[0].truncated);
        assert!(snapshot.recent_failures[0].short_error.as_ref().unwrap().contains("<truncated>"));
        assert_eq!(snapshot.recent_failures[0].failure_debug.as_ref().unwrap().capture_outcome_code, "metadata-only");
        assert!(snapshot.recent_jobs[0].failure_debug.is_some());
        assert!(!rendered.contains('\u{0007}'));
        assert!(!rendered.contains("/home/"));
        assert_eq!(bounded_untrusted_text("authorization bearer SHOULD_NOT_LEAK"), SECRET_REDACTION);
    }

    #[test]
    fn remote_build_observability_report_names_route_transfer_trust_and_attestation() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let response = fixture_builder_response(&client);
        let admission =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect("remote output admits");
        let import_report = RemoteOutputImportReport {
            request_id: admission.request_id.clone(),
            store_prefix: admission.store_prefix.clone(),
            outputs: vec![RemoteImportedOutput {
                name: "out".to_string(),
                logical_path: "/mantle/store/imported-out".to_string(),
                path_info_signing_key_id: "builder-key".to_string(),
                artifact_attestation_digest_blake3: "a".repeat(BLAKE3_HEX_LENGTH_CHARS),
                artifact_attestation_path: "/state/attestations/artifacts/imported.json".to_string(),
            }],
            transfer: admission.transfer.clone(),
        };
        const OBSERVABILITY_UPLOAD_BYTES: u64 = 7;
        let upload_summary = RemoteInputUploadPrivacySummary {
            store_objects: 1,
            source_objects: 0,
            proof_objects: 0,
            secret_descriptor_objects: 0,
            total_objects: 1,
            total_bytes: OBSERVABILITY_UPLOAD_BYTES,
        };
        let report = remote_build_observability_report(
            "p2p-remote-builder",
            &["local-preflight-failed".to_string()],
            Some("builder-1"),
            &[RemoteAttemptReasonCode::StaleReportRejected],
            upload_summary,
            None,
            None,
            &admission,
            &import_report,
            &[REMOTE_SESSION_NON_CLAIM.to_string()],
        )
        .expect("observability report builds");

        assert_eq!(report.selected_route, "p2p-remote-builder");
        assert_eq!(report.endpoint_id.as_deref(), Some("builder-1"));
        assert_eq!(report.attempt_reason_codes, vec!["stale-report-rejected".to_string()]);
        assert_eq!(report.transfer.mode, RemoteTransferMode::Delta);
        assert_eq!(report.trust_basis.key_id, "builder-key");
        assert_eq!(
            report.outputs[0].artifact_attestation_path.as_deref(),
            Some("/state/attestations/artifacts/imported.json")
        );
        assert!(report.non_claims.iter().any(|claim| claim.contains("protocol control flow only")));
    }

    #[test]
    fn remote_build_observability_rejects_mismatched_import_claims() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let response = fixture_builder_response(&client);
        let admission =
            validate_remote_builder_response_output_import(&client.request, &client.trusted_output_keys, &response)
                .expect("remote output admits");
        let import_report = RemoteOutputImportReport {
            request_id: "other-request".to_string(),
            store_prefix: admission.store_prefix.clone(),
            outputs: Vec::new(),
            transfer: admission.transfer.clone(),
        };
        let upload_summary = RemoteInputUploadPrivacySummary {
            store_objects: 0,
            source_objects: 0,
            proof_objects: 0,
            secret_descriptor_objects: 0,
            total_objects: 0,
            total_bytes: 0,
        };
        let err = remote_build_observability_report(
            "p2p-remote-builder",
            &[],
            None,
            &[],
            upload_summary,
            None,
            None,
            &admission,
            &import_report,
            &[],
        )
        .expect_err("mismatched import report rejected");

        assert_eq!(err, "remote-observability-request-id-mismatch");
    }

    #[tokio::test]
    async fn operator_remote_build_e2e_rail_composes_route_stdio_input_admission_and_reports() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = remote_import_store(temp.path()).await;
        let builder = fixture_loopback_builder();
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let route_plan = fixture_remote_operator_route_plan(&client);
        let executor = PathInfoRemoteExecutor {
            path_info: importable_pathinfo(),
            artifact_attestation_digest_blake3: None,
            nar_payload: None,
        };
        let mut state = RemoteTicketState::default();
        state.tickets.insert("ticket-1".to_string(), fixture_ticket());
        let request_frames = encode_frame_stream(&remote_client_request_frames(&client));
        let response = plan_stdio_remote_once_from_state_with_executor(
            std::io::Cursor::new(request_frames),
            &builder,
            &mut state,
            client.transfer_capabilities,
            &executor,
        )
        .expect("operator rail remote response plans");
        let mut stdout = Vec::new();
        write_remote_response_frames(&mut stdout, &response).expect("response frames serialize");
        let child_output = RemoteStdioChildOutput {
            stdout,
            stderr: b"remote fixture log\n".to_vec(),
            status_success: true,
        };
        let transcript = validate_stdio_child_output(&child_output).expect("stdio transcript validates");
        let admission = validate_remote_builder_frames_output_import(
            &client.request,
            &client.trusted_output_keys,
            &transcript.frames,
        )
        .expect("operator rail remote output admits");
        let import_report = import_admitted_remote_outputs(
            &mut store,
            &client.request,
            &admission,
            true,
            Some(crunch_store::GcRootSource::Build),
        )
        .await
        .expect("operator rail remote output imports");
        let upload_summary = plan_remote_input_upload_privacy_summary(
            &client.uploaded_input_refs,
            &client.request.source_input_refs,
            &[],
            &RemoteInputUploadPrivacyPolicy::default(),
        )
        .expect("operator rail upload summary plans");
        let status = fixture_remote_operator_status(&client);
        let report = remote_operator_e2e_rail_report(
            &route_plan,
            &transcript,
            upload_summary,
            &admission,
            &import_report,
            status,
            &[REMOTE_SESSION_NON_CLAIM.to_string()],
        )
        .expect("operator rail report builds");
        let rendered = serde_json::to_string(&report).expect("operator rail report serializes");

        assert_eq!(report.schema, REMOTE_OPERATOR_E2E_RAIL_SCHEMA);
        assert_eq!(report.selected_route, "p2p-remote-builder");
        assert_eq!(report.selected_reason_code, "builder-capability-and-output-trust-match");
        assert_eq!(report.phases.handshake, REMOTE_E2E_HANDSHAKE_COMPLETE_PHASE);
        assert_eq!(report.phases.execution, REMOTE_E2E_EXECUTION_COMPLETE_PHASE);
        assert_eq!(report.upload_summary.store_objects, 1);
        assert_eq!(report.transfer.mode, RemoteTransferMode::Delta);
        assert_eq!(report.trust_basis.key_id, "builder-key");
        assert_eq!(report.artifact_attestation_paths.len(), 1);
        assert_eq!(report.status.worker_count, 1);
        assert_eq!(report.build_report.outputs.len(), 1);
        assert!(report.non_claims.iter().any(|claim| claim.contains("fixture composition only")));
        assert!(rendered.contains("remote-execution-complete"));
        assert!(!rendered.contains("secret-1"));
        assert!(!rendered.contains("private-key"));
    }

    #[test]
    fn operator_remote_build_e2e_rail_rejects_cross_seam_failures() {
        let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["other-key".to_string()]);
        let response = fixture_builder_response(&client);
        let child_output = RemoteStdioChildOutput {
            stdout: encode_frame_stream(&response.response_frames).to_vec(),
            stderr: Vec::new(),
            status_success: true,
        };
        let untrusted =
            validate_stdio_child_exchange_output(&client.request, &client.trusted_output_keys, &child_output)
                .expect_err("missing output trust fails before import");
        assert_eq!(untrusted.phase, RemoteFailurePhase::OutputImport);
        assert_eq!(untrusted.reason, "untrusted-output-key");

        let route = fixture_remote_operator_route_plan_without_output_trust(&client);
        assert_eq!(route.selected_route, crate::realization_routing::RouteClass::PreflightError);
        assert!(route.rejected_routes.iter().any(|rejection| rejection.reason_code == "output-trust-missing"));

        let polluted = validate_stdio_child_output(&RemoteStdioChildOutput {
            stdout: b"human stdout should fail framing\n".to_vec(),
            stderr: Vec::new(),
            status_success: true,
        })
        .expect_err("unframed stdout fails protocol validation");
        assert_eq!(polluted.phase, RemoteFailurePhase::RequestValidation);

        let source_ref = "/mantle/store/dddddddddddddddddddddddddddddddd-source";
        let record = ready_source_record(source_ref);
        let stale = validate_remote_source_upload_record(&record, "/mantle/store", source_ref, Some("stale-digest"))
            .expect_err("stale source state is rejected");
        assert_eq!(stale, "remote-input-source-digest-stale");

        let secret_ref = format!("{REMOTE_UPLOAD_SECRET_DESCRIPTOR_PREFIX}signing-key");
        let privacy = plan_remote_input_upload_privacy_summary(
            std::slice::from_ref(&secret_ref),
            &[],
            &[],
            &RemoteInputUploadPrivacyPolicy::default(),
        )
        .expect_err("secret descriptor upload is rejected");
        assert_eq!(privacy, "remote-input-upload-class-disallowed");

        let fallback = crunch_build::distributed::classify_remote_build_service_failure(
            crunch_build::distributed::RemoteBuildFallbackPolicy::Never,
            RemoteFailurePhase::OutputImport.as_status_label(),
            "untrusted-output-key",
        );
        assert!(matches!(fallback, crunch_build::distributed::RemoteFailureDecision::ReturnFailure(_)));
    }

    #[test]
    fn coordinator_lost_restart_state_reports_phase_and_session_guards_backoff() {
        let request = fixture_coordinator_request();
        let normalized_key = normalized_remote_build_key(&request).expect("request key");
        let mut state = RemoteCoordinatorState::default();
        let job_id = RemoteJobId::new("lost-job").expect("fixture job id");
        state.jobs.insert(job_id.clone(), RemoteCoordinatorJobSummary {
            job_id,
            normalized_build_key: normalized_key,
            assigned_worker_endpoint_id: None,
            phase: RemoteCoordinatorJobPhase::Lost,
            live_output_claims: Vec::new(),
            result_available: false,
            lost_phase: Some(RemoteFailurePhase::BuildExecution),
            immutable_log: None,
            observability_health: None,
            failure_debug: None,
            short_error: Some("worker restart lost job".to_string()),
            current_attempt: None,
            transfer_checkpoint: None,
            transferred_bytes: 0,
            output_admission_completed: false,
            last_attempt_reason_code: None,
            resource_requirements: None,
            resource_lease_id_blake3: None,
            resource_fit: None,
            locality: None,
        });
        let decision = plan_coordinator_dispatch(&state, &request, &fixture_assignment_nonce("lost-plan"))
            .expect("lost job decision");

        assert!(
            matches!(decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "restart-state-unavailable-phase-build-execution")
        );
        assert_eq!(decide_remote_reconnect("session-a", "session-a", 0, 1), RemoteReconnectDecision::SameSession);
        assert_eq!(decide_remote_reconnect("session-a", "session-b", 1, 1), RemoteReconnectDecision::BackoffRequired);
        assert_eq!(
            decide_session_lease_release(RemoteCoordinatorJobPhase::Running),
            RemoteLeaseReleaseDecision::Retain
        );
        assert_eq!(
            decide_session_lease_release(RemoteCoordinatorJobPhase::Finished),
            RemoteLeaseReleaseDecision::Release
        );
    }

    struct PathInfoRemoteExecutor {
        path_info: PathInfo,
        artifact_attestation_digest_blake3: Option<String>,
        nar_payload: Option<Vec<u8>>,
    }

    impl RemoteBuildExecutor for PathInfoRemoteExecutor {
        fn execute(
            &self,
            _request: &ConcreteBuildRequest,
            plan: &RemoteExecutablePlan,
            _input_upload: &RemoteInputUpload,
        ) -> Result<RemoteExecutionOutcome, String> {
            let outputs = plan
                .expected_outputs
                .iter()
                .map(|expected| pathinfo_executor_output(plan, expected, self))
                .collect::<Result<Vec<_>, _>>()?;
            let output_digest_blake3 = remote_execution_outputs_digest(&outputs)?;
            let output_size_bytes = sum_remote_execution_output_sizes(&outputs)?;
            Ok(RemoteExecutionOutcome {
                request_id: plan.request_id.clone(),
                plan_digest_blake3: plan.plan_digest_blake3.clone(),
                output_digest_blake3,
                output_size_bytes,
                outputs,
            })
        }
    }

    fn pathinfo_executor_output(
        plan: &RemoteExecutablePlan,
        expected: &RemoteExpectedOutput,
        executor: &PathInfoRemoteExecutor,
    ) -> Result<RemoteExecutionOutput, String> {
        let artifact_attestation_digest_blake3 = match &executor.artifact_attestation_digest_blake3 {
            Some(digest) => digest.clone(),
            None => crunch_store::artifact_attestation_digest_for_pathinfo(
                &plan.store_prefix,
                &executor.path_info,
                &expected.name,
                None,
            )
            .map_err(|err| format!("test-artifact-attestation-digest-failed: {err}"))?
            .to_hex(),
        };
        Ok(RemoteExecutionOutput {
            name: expected.name.clone(),
            logical_path: expected
                .logical_path
                .clone()
                .unwrap_or_else(|| executor.path_info.store_path.to_absolute_path_with_prefix(&plan.store_prefix)),
            content_digest_blake3: blake3::hash(format!("pathinfo:{}", plan.plan_digest_blake3).as_bytes())
                .to_hex()
                .to_string(),
            size_bytes: executor.path_info.nar_size,
            artifact_attestation_digest_blake3,
            path_info: Some(executor.path_info.clone()),
            nar_payload: executor.nar_payload.clone(),
        })
    }

    struct FixedRemoteExecutor {
        output_size_bytes: u64,
        logical_path_override: Option<String>,
    }

    impl RemoteBuildExecutor for FixedRemoteExecutor {
        fn execute(
            &self,
            _request: &ConcreteBuildRequest,
            plan: &RemoteExecutablePlan,
            _input_upload: &RemoteInputUpload,
        ) -> Result<RemoteExecutionOutcome, String> {
            let outputs = plan
                .expected_outputs
                .iter()
                .map(|expected| fixed_executor_output(plan, expected, self))
                .collect::<Vec<_>>();
            let output_digest_blake3 = remote_execution_outputs_digest(&outputs)?;
            let output_size_bytes = sum_remote_execution_output_sizes(&outputs)?;
            Ok(RemoteExecutionOutcome {
                request_id: plan.request_id.clone(),
                plan_digest_blake3: plan.plan_digest_blake3.clone(),
                output_digest_blake3,
                output_size_bytes,
                outputs,
            })
        }
    }

    fn fixed_executor_output(
        plan: &RemoteExecutablePlan,
        expected: &RemoteExpectedOutput,
        executor: &FixedRemoteExecutor,
    ) -> RemoteExecutionOutput {
        let content_digest_blake3 =
            blake3::hash(format!("fixed:{}:{}", plan.plan_digest_blake3, expected.name).as_bytes())
                .to_hex()
                .to_string();
        let logical_path =
            executor.logical_path_override.clone().or_else(|| expected.logical_path.clone()).unwrap_or_else(|| {
                let store_hash = content_digest_blake3.get(..STORE_PATH_HASH_CHARS).unwrap_or(&content_digest_blake3);
                format!("{}/{}-{}", plan.store_prefix, store_hash, expected.name)
            });
        let artifact_attestation_digest_blake3 =
            blake3::hash(format!("fixed-artifact:{}:{}", content_digest_blake3, logical_path).as_bytes())
                .to_hex()
                .to_string();
        RemoteExecutionOutput {
            name: expected.name.clone(),
            logical_path,
            content_digest_blake3,
            size_bytes: executor.output_size_bytes,
            artifact_attestation_digest_blake3,
            path_info: None,
            nar_payload: None,
        }
    }

    fn encode_frame_stream(frames: &[RemoteFrame]) -> zeroize::Zeroizing<Vec<u8>> {
        let mut encoded = zeroize::Zeroizing::new(Vec::new());
        for frame in frames {
            let frame_bytes = encode_remote_frame(frame).expect("test frame encodes");
            encoded.extend_from_slice(&frame_bytes);
        }
        encoded
    }

    fn fixture_builder_response(client: &RemoteLoopbackClient) -> RemoteBuilderFrameResponse {
        let builder = fixture_loopback_builder();
        let mut ticket = fixture_ticket();
        plan_remote_builder_frames(
            &builder,
            &mut ticket,
            &remote_client_request_frames(client),
            client.transfer_capabilities,
        )
        .expect("fixture builder response plans")
    }

    fn rewrite_response_transfer_key(response: &mut RemoteBuilderFrameResponse, key: &str) {
        response.transfer.verified_builder_key = key.to_string();
        for frame in &mut response.response_frames {
            if let RemoteFrame::OutputTransferDone { report } = frame {
                report.verified_builder_key = key.to_string();
            }
        }
    }

    fn rewrite_response_digest(response: &mut RemoteBuilderFrameResponse, digest: &str) {
        response.output_digest_blake3 = digest.to_string();
        for frame in &mut response.response_frames {
            if let RemoteFrame::BuildFinished { result } = frame {
                result.output_digest_blake3 = digest.to_string();
            }
        }
    }

    fn corrupt_first_transfer_artifact_payload(response: &mut RemoteBuilderFrameResponse) {
        if let Some(artifact) = response.transfer_artifacts.first_mut()
            && let Some(byte) = artifact.payload.first_mut()
        {
            *byte = CORRUPTED_TRANSFER_PAYLOAD_BYTE;
        }
        for frame in &mut response.response_frames {
            if let RemoteFrame::OutputTransferArtifact { artifact } = frame {
                if let Some(byte) = artifact.payload.first_mut() {
                    *byte = CORRUPTED_TRANSFER_PAYLOAD_BYTE;
                }
                return;
            }
        }
    }

    fn drop_first_transfer_artifact_frame(response: &mut RemoteBuilderFrameResponse) {
        if !response.transfer_artifacts.is_empty() {
            response.transfer_artifacts.remove(0);
        }
        if let Some(index) = response
            .response_frames
            .iter()
            .position(|frame| matches!(frame, RemoteFrame::OutputTransferArtifact { .. }))
        {
            response.response_frames.remove(index);
        }
    }

    async fn remote_import_store(root: &std::path::Path) -> crunch_store::StoreHandle {
        crunch_store::StoreHandle::open(crunch_store::StoreConfig {
            state_dir: root.join("state"),
            output_dir: root.join("store"),
            remote_cache_urls: Vec::new(),
            fallback_mode: crunch_store::StoreFallbackMode::Practical,
            store_dir: "/mantle/store".to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .expect("remote import store opens")
    }

    fn importable_admission_report() -> RemoteOutputAdmissionReport {
        let output = importable_produced_output();
        let transfer = full_transfer_report(output.size_bytes, "builder-key", None);
        let transfer_artifacts = vec![
            pathinfo_json_transfer_artifact("r1", &output, output.path_info.as_ref().unwrap())
                .expect("pathinfo transfer artifact"),
        ];
        RemoteOutputAdmissionReport {
            request_id: "r1".to_string(),
            output_digest_blake3: remote_produced_outputs_content_digest(std::slice::from_ref(&output)),
            builder_signing_key_id: "builder-key".to_string(),
            trust_basis: RemoteOutputTrustBasis {
                key_id: "builder-key".to_string(),
                key_material_digest_blake3: None,
            },
            store_prefix: "/mantle/store".to_string(),
            outputs: vec![output],
            transfer_artifacts,
            streamed_manifest: None,
            transfer,
        }
    }

    fn importable_produced_output() -> RemoteProducedOutput {
        let path_info = importable_pathinfo();
        let artifact_digest =
            crunch_store::artifact_attestation_digest_for_pathinfo("/mantle/store", &path_info, "out", None)
                .expect("artifact digest")
                .to_hex();
        RemoteProducedOutput {
            name: "out".to_string(),
            logical_path: fixture_expected_outputs()[0]
                .logical_path
                .clone()
                .expect("fixture expected output path is known"),
            content_digest_blake3: blake3::hash(b"remote-output-content").to_hex().to_string(),
            size_bytes: path_info.nar_size,
            path_info_signing_key_id: "builder-key".to_string(),
            artifact_attestation_digest_blake3: artifact_digest,
            path_info: Some(path_info),
            nar_payload_digest_blake3: None,
            nar_payload_size_bytes: None,
        }
    }

    fn importable_pathinfo() -> PathInfo {
        PathInfo {
            store_path: importable_store_path(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("remote-target").unwrap(),
            },
            references: vec![],
            nar_size: IMPORT_NAR_SIZE_BYTES,
            nar_sha256: [IMPORT_NAR_SHA256_FILL_BYTE; SHA256_DIGEST_BYTES],
            signatures: vec![builder_signature()],
            deriver: None,
            ca: None,
        }
    }

    fn importable_nar_pathinfo() -> PathInfo {
        let mut path_info = (*snix_store::fixtures::PATH_INFO_SYMLINK).clone();
        path_info.store_path = importable_store_path();
        path_info.signatures = vec![builder_signature()];
        path_info
    }

    fn builder_signature() -> nix_compat::narinfo::Signature<String> {
        nix_compat::narinfo::Signature::new(
            "builder-key".to_string(),
            [BUILDER_SIGNATURE_FILL_BYTE; ED25519_SIGNATURE_BYTES],
        )
    }

    fn pathinfo_for_logical_path(logical_path: &str) -> PathInfo {
        let mut path_info = importable_pathinfo();
        path_info.store_path = StorePath::from_absolute_path_with_prefix(logical_path.as_bytes(), "/mantle/store")
            .expect("fixture output path parses");
        path_info
    }

    fn importable_store_path() -> StorePath<String> {
        let logical_path =
            fixture_expected_outputs()[0].logical_path.clone().expect("fixture expected output path is known");
        StorePath::from_absolute_path_with_prefix(logical_path.as_bytes(), "/mantle/store")
            .expect("fixture output path parses")
    }

    fn mismatched_store_path() -> StorePath<String> {
        StorePath::from_absolute_path_with_prefix(
            b"/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-other",
            "/mantle/store",
        )
        .expect("mismatched fixture output path parses")
    }

    fn persist_transaction_ticket(
        state_dir: &Path,
        uses_remaining: u32,
    ) -> (
        std::sync::Arc<crate::remote_credentials::TicketVerifierKey>,
        TicketAuthRequest,
        ConcreteBuildRequest,
    ) {
        let verifier_key = fixture_ticket_verifier_key();
        let created_unix_s = crate::unix_time_now_s().unwrap();
        let expires_unix_s = created_unix_s.checked_add(TRANSACTION_TICKET_TTL_SECS).unwrap();
        let ticket = RemoteTicket::fixture(
            TRANSACTION_TICKET_ID,
            "transaction-test",
            fixture_ticket_token(),
            &verifier_key,
            created_unix_s,
            expires_unix_s,
            uses_remaining,
            MAX_REMOTE_BUILD_TIME_SECS,
            MAX_REMOTE_UPLOAD_BYTES,
            None,
        );
        let mut state = RemoteTicketState::default();
        state.bind_active_verifier_key(&verifier_key);
        state.tickets.insert(TRANSACTION_TICKET_ID.to_string(), ticket);
        save_ticket_state(state_dir, &state).unwrap();
        let mut client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        client.auth.ticket_id = TRANSACTION_TICKET_ID.to_string();
        client.auth.secret = fixture_ticket_token().to_string();
        client.auth.client_endpoint = None;
        (verifier_key, client.auth, client.request)
    }

    fn fixture_ticket() -> RemoteTicket {
        let key = fixture_ticket_verifier_key();
        RemoteTicket::fixture(
            "ticket-1",
            "test",
            fixture_ticket_token(),
            &key,
            1,
            10,
            1,
            10,
            10,
            Some("client-a".to_string()),
        )
    }

    fn fixture_ticket_verifier_key() -> std::sync::Arc<crate::remote_credentials::TicketVerifierKey> {
        std::sync::Arc::new(
            crate::remote_credentials::TicketVerifierKey::parse(
                "ticket-key-1:QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE",
            )
            .expect("fixture verifier key is canonical"),
        )
    }

    fn fixture_ticket_token() -> &'static str {
        "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI"
    }

    fn fixture_hello() -> RemoteHello {
        RemoteHello {
            alpn: REMOTE_PROTOCOL_ALPN.to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: "builder-1".to_string(),
            capabilities: vec!["delta".to_string(), "full".to_string()],
            workspace_policy: None,
        }
    }

    fn fixture_auth_facts(server_now_unix_s: u64) -> RemoteTicketAuthFacts<'static> {
        RemoteTicketAuthFacts {
            server_now_unix_s,
            authenticated_client_endpoint: Some("client-a"),
        }
    }

    fn fixture_auth_request() -> TicketAuthRequest {
        TicketAuthRequest {
            ticket_id: "ticket-1".to_string(),
            secret: fixture_ticket_token().to_string(),
            client_endpoint: Some("client-a".to_string()),
            now_unix_s: 2,
        }
    }

    fn fixture_request() -> ConcreteBuildRequest {
        ConcreteBuildRequest {
            request_id: "r1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            input_refs: vec!["input-a".to_string()],
            source_input_refs: Vec::new(),
            upload_bytes: 1,
            build_time_limit_secs: 1,
            contains_raw_frontend_eval: false,
            payload: fixture_payload(),
            expected_outputs: fixture_expected_outputs(),
            production_attempt: None,
            transfer_policy: None,
            resource_requirements: None,
            locality_scope: None,
            failure_debug_policy: crunch_build::distributed::RemoteFailureDebugPolicy::default(),
            failure_replay: None,
        }
    }

    fn fixture_payload() -> RemoteConcreteBuildPayload {
        RemoteConcreteBuildPayload::Action {
            action_id: "action-1".to_string(),
            spec_json: fixture_action_spec_json("action-1"),
        }
    }

    fn fixture_action_spec_json(action_id: &str) -> String {
        serde_json::json!({
            "schema": REMOTE_ACTION_SPEC_SCHEMA,
            "action_id": action_id,
            "builder": "builtin:fixture",
            "args": ["--emit"],
            "outputs": ["out"]
        })
        .to_string()
    }

    fn fixture_derivation_request() -> ConcreteBuildRequest {
        let (drv_json, drv_path, nix_drv) = fixture_derivation_parts();
        let out_path = nix_drv.outputs["out"]
            .path
            .as_ref()
            .expect("input-addressed output path")
            .to_absolute_path_with_prefix("/mantle/store");
        ConcreteBuildRequest {
            payload: RemoteConcreteBuildPayload::Derivation {
                drv_path: drv_path.to_absolute_path_with_prefix("/mantle/store"),
                drv_json,
            },
            expected_outputs: vec![RemoteExpectedOutput {
                name: "out".to_string(),
                logical_path: Some(out_path),
            }],
            ..fixture_request()
        }
    }

    fn fixture_ca_derivation_request() -> ConcreteBuildRequest {
        let (drv_json, drv_path, nix_drv) = fixture_ca_derivation_parts();
        assert!(nix_drv.outputs["out"].path.is_none());
        ConcreteBuildRequest {
            payload: RemoteConcreteBuildPayload::Derivation {
                drv_path: drv_path.to_absolute_path_with_prefix("/mantle/store"),
                drv_json,
            },
            expected_outputs: vec![RemoteExpectedOutput {
                name: "out".to_string(),
                logical_path: None,
            }],
            ..fixture_request()
        }
    }

    fn fixture_remote_client_derivation_input() -> RemoteClientDerivationInput {
        let drv_json = fixture_derivation_json();
        let drv: crunch_glue::CrunchDerivation = serde_json::from_str(&drv_json).expect("fixture drv parses");
        remote_client_input_from_derivation("root", drv)
    }

    fn fixture_remote_client_derivation_input_with_dep() -> RemoteClientDerivationInput {
        let dep = fixture_crunch_derivation("remote-dep", Vec::new());
        let root = fixture_crunch_derivation("remote-root", vec![crunch_glue::Input::Derivation(Box::new(dep))]);
        remote_client_input_from_derivation("root", root)
    }

    fn remote_client_input_from_derivation(
        label: &str,
        drv: crunch_glue::CrunchDerivation,
    ) -> RemoteClientDerivationInput {
        let mut known_paths = crunch_glue::ConversionCache::new("/mantle/store");
        let (drv_path, nix_derivation) = crunch_glue::convert(&drv, &mut known_paths).expect("fixture drv converts");
        RemoteClientDerivationInput {
            label: label.to_string(),
            drv_path,
            crunch_derivation: drv,
            nix_derivation,
        }
    }

    fn fixture_crunch_derivation(name: &str, inputs: Vec<crunch_glue::Input>) -> crunch_glue::CrunchDerivation {
        crunch_glue::CrunchDerivation {
            name: name.to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            args: vec!["-c".to_string(), "echo hi".to_string()],
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: Vec::new(),
            env: std::collections::HashMap::new(),
            inputs,
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        }
    }

    fn fixture_remote_client_options() -> RemoteClientBuildOptions {
        RemoteClientBuildOptions {
            store_prefix: "/mantle/store".to_string(),
            ticket: RemoteTicketCredential {
                ticket_id: "ticket-1".to_string(),
                secret: fixture_ticket_token().to_string(),
            },
            builder: RemoteStdioBuilderCommand {
                endpoint_id: "builder-1".to_string(),
                program: PathBuf::from("/bin/mantle-remote"),
                args: vec!["serve".to_string()],
            },
            trusted_output_keys: vec!["builder-key".to_string()],
            now_unix_s: 2,
            build_time_limit_secs: DEFAULT_TICKET_BUILD_TIME_SECS,
            client_endpoint: None,
            transfer_capabilities: RemoteTransferCapabilities::delta_and_full(),
        }
    }

    fn fixture_derivation_parts() -> (String, StorePath<String>, nix_compat::derivation::Derivation) {
        let drv_json = fixture_derivation_json();
        let drv: crunch_glue::CrunchDerivation = serde_json::from_str(&drv_json).expect("fixture drv parses");
        let mut known_paths = crunch_glue::ConversionCache::new("/mantle/store");
        let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut known_paths).expect("fixture drv converts");
        (drv_json, drv_path, nix_drv)
    }

    fn fixture_ca_derivation_parts() -> (String, StorePath<String>, nix_compat::derivation::Derivation) {
        let drv_json = fixture_ca_derivation_json();
        let drv: crunch_glue::CrunchDerivation = serde_json::from_str(&drv_json).expect("fixture CA drv parses");
        let mut known_paths = crunch_glue::ConversionCache::new("/mantle/store");
        let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut known_paths).expect("fixture CA drv converts");
        (drv_json, drv_path, nix_drv)
    }

    fn fixture_derivation_drv_path(request: &ConcreteBuildRequest) -> String {
        match &request.payload {
            RemoteConcreteBuildPayload::Derivation { drv_path, .. } => drv_path.clone(),
            RemoteConcreteBuildPayload::Action { .. } => panic!("fixture expected derivation payload"),
        }
    }

    fn fixture_build_outcome(
        request: &ConcreteBuildRequest,
        outputs: BTreeMap<String, PathInfo>,
    ) -> crunch_build::BuildOutcome {
        let drv_path = StorePath::from_absolute_path_with_prefix(
            fixture_derivation_drv_path(request).as_bytes(),
            &request.store_prefix,
        )
        .expect("fixture drv path parses");
        crunch_build::BuildOutcome {
            drv_path,
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        }
    }

    fn fixture_derivation_json() -> String {
        serde_json::json!({
            "name": "remote-fixture",
            "builder": "/bin/sh",
            "args": ["-c", "echo hi"],
            "outputs": ["out"],
            "addressing_mode": "input-addressed"
        })
        .to_string()
    }

    fn fixture_ca_derivation_json() -> String {
        serde_json::json!({
            "name": "remote-ca-fixture",
            "builder": "/bin/sh",
            "args": ["-c", "echo hi > $out"],
            "outputs": ["out"],
            "addressing_mode": "content-addressed"
        })
        .to_string()
    }

    fn fixture_local_build_executor() -> RemoteLocalBuildExecutor {
        RemoteLocalBuildExecutor {
            endpoint_id: "builder-1".to_string(),
            coordinator_state_dir: std::env::temp_dir().join("mantle-remote-build-test-coordinator-state"),
            state_dir: std::env::temp_dir().join("mantle-remote-build-test-state"),
            output_dir: std::env::temp_dir().join("mantle-remote-build-test-store"),
            store_prefix: "/mantle/store".to_string(),
            keypair: fixture_keypair(),
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            verbose: false,
        }
    }

    fn fixture_keypair() -> crunch_build::KeyPair {
        crunch_build::load_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        )
        .expect("fixture keypair parses")
    }

    fn fixture_expected_outputs() -> Vec<RemoteExpectedOutput> {
        vec![RemoteExpectedOutput {
            name: "out".to_string(),
            logical_path: Some("/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-fixture".to_string()),
        }]
    }

    fn fixture_manifest() -> RemoteInputManifest {
        RemoteInputManifest {
            request_id: "r1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            input_refs: vec!["input-a".to_string()],
            closure_refs: vec!["input-a".to_string()],
        }
    }

    fn fixture_loopback_builder() -> RemoteLoopbackBuilder {
        RemoteLoopbackBuilder {
            endpoint_id: "builder-1".to_string(),
            store_prefix: "/mantle/store".to_string(),
            supported_capabilities: vec!["delta".to_string(), "full".to_string()],
            present_input_refs: Vec::new(),
            signing_key_id: "builder-key".to_string(),
            transfer_capabilities: RemoteTransferCapabilities::delta_and_full(),
        }
    }

    fn fixture_loopback_client(
        uploaded_input_refs: Vec<String>,
        trusted_output_keys: Vec<String>,
    ) -> RemoteLoopbackClient {
        RemoteLoopbackClient {
            session_id: "session-1".to_string(),
            hello: fixture_hello(),
            auth: fixture_auth_request(),
            request: fixture_request(),
            input_manifest: fixture_manifest(),
            uploaded_input_refs,
            trusted_output_keys,
            transfer_capabilities: RemoteTransferCapabilities::delta_and_full(),
        }
    }

    fn fixture_locality_manifest() -> CanonicalRemoteTransferManifest {
        let policy = RemoteTransferPolicy::default();
        let policy_digest =
            crunch_build::distributed::canonical_remote_transfer_policy_digest(policy).expect("locality policy digest");
        let artifact_id = crunch_build::distributed::RemoteTransferArtifactId::new("locality-artifact")
            .expect("locality artifact id");
        let artifact_digest = crunch_build::distributed::RemoteTransferDigest::new(
            blake3::hash(b"locality-artifact").to_hex().to_string(),
        )
        .expect("locality artifact digest");
        let chunk_digest =
            crunch_build::distributed::RemoteTransferDigest::new(blake3::hash(b"locality-chunk").to_hex().to_string())
                .expect("locality chunk digest");
        crunch_build::distributed::canonicalize_remote_transfer_manifest(
            crunch_build::distributed::RemoteTransferManifest {
                schema: crunch_build::distributed::REMOTE_TRANSFER_MANIFEST_SCHEMA.to_string(),
                session_id: crunch_build::distributed::RemoteTransferSessionId::new(
                    blake3::hash(b"locality-session").to_hex().to_string(),
                )
                .expect("locality session id"),
                job_id: RemoteJobId::new("locality-job").expect("locality job id"),
                attempt_id: RemoteAttemptId::new("locality-attempt").expect("locality attempt id"),
                fence_generation: RemoteFenceGeneration::INITIAL,
                policy_digest_blake3: policy_digest,
                store_prefix: "/mantle/store".to_string(),
                requested_content_blake3: crunch_build::distributed::RemoteTransferDigest::new(
                    blake3::hash(b"locality-request").to_hex().to_string(),
                )
                .expect("requested content digest"),
                artifacts: vec![crunch_build::distributed::RemoteTransferArtifact {
                    artifact_id,
                    artifact_kind: crunch_build::distributed::RemoteTransferArtifactKind::CastoreBlob,
                    digest_blake3: artifact_digest,
                    size_bytes: TEST_LOCALITY_ARTIFACT_BYTES,
                    required_for_completion: true,
                    nar_sha256_hex: None,
                    chunks: vec![crunch_build::distributed::RemoteTransferChunkDescriptor {
                        index: 0,
                        offset_bytes: 0,
                        size_bytes: TEST_LOCALITY_CHUNK_BYTES,
                        digest_blake3: chunk_digest,
                    }],
                }],
            },
            policy,
        )
        .expect("locality manifest canonicalizes")
    }

    fn fixture_resource_inventory() -> RemoteWorkerResourceInventory {
        RemoteWorkerResourceInventory {
            total: RemoteResourceVector {
                cpu_units: TEST_RESOURCE_CPU_UNITS,
                memory_bytes: TEST_RESOURCE_MEMORY_BYTES,
                scratch_bytes: TEST_RESOURCE_SCRATCH_BYTES,
                accelerators: vec![RemoteNamedResourceQuantity {
                    name: "nvidia-sm90".to_string(),
                    quantity: TEST_RESOURCE_ACCELERATOR_COUNT,
                }],
                named_tokens: vec![RemoteNamedResourceQuantity {
                    name: "linker-seat".to_string(),
                    quantity: TEST_RESOURCE_TOKEN_COUNT,
                }],
            },
        }
    }

    fn fixture_resource_requirements() -> RemoteResourceRequirements {
        RemoteResourceRequirements {
            quantities: fixture_resource_inventory().total,
            semantic_accelerator_classes: vec!["nvidia-sm90".to_string()],
        }
    }

    fn fixture_worker_registration() -> RemoteWorkerRegistration {
        RemoteWorkerRegistration {
            endpoint_id: "builder-1".to_string(),
            protocol_version: REMOTE_PROTOCOL_VERSION,
            worker_generation: legacy_worker_generation(),
            systems: vec![DEFAULT_REMOTE_ACTION_SYSTEM.to_string()],
            feature_labels: vec!["kvm".to_string()],
            sandbox_modes: vec!["bwrap".to_string()],
            network_modes: vec!["off".to_string()],
            logical_store_prefixes: vec!["/mantle/store".to_string()],
            concurrency: 1,
            resource_inventory: None,
            output_signing_key_ids: vec!["builder-key".to_string()],
            resumable_jobs: Vec::new(),
            workspace_policy: None,
        }
    }

    fn fixture_quantified_coordinator_request(action_id: &str, claim: &str) -> RemoteCoordinatorBuildRequest {
        let mut request = fixture_coordinator_request();
        request.request.payload = RemoteConcreteBuildPayload::Action {
            action_id: action_id.to_string(),
            spec_json: fixture_action_spec_json(action_id),
        };
        request.request.resource_requirements = Some(fixture_resource_requirements());
        request.resource_requirements = request.request.resource_requirements.clone();
        request.live_output_claims = vec![claim.to_string()];
        request
    }

    fn fixture_coordinator_request() -> RemoteCoordinatorBuildRequest {
        RemoteCoordinatorBuildRequest {
            request: fixture_request(),
            required_system: DEFAULT_REMOTE_ACTION_SYSTEM.to_string(),
            required_features: vec!["kvm".to_string()],
            required_sandbox_mode: "bwrap".to_string(),
            required_network_mode: "off".to_string(),
            resource_requirements: None,
            locality_scope: None,
            trusted_output_keys: vec!["builder-key".to_string()],
            live_output_claims: vec!["claim-out".to_string()],
            wait_for_worker: false,
        }
    }

    fn fixture_remote_operator_route_plan(
        client: &RemoteLoopbackClient,
    ) -> crate::realization_routing::RoutePlanReport {
        fixture_remote_operator_route_plan_with_trust(client, true)
    }

    fn fixture_remote_operator_route_plan_without_output_trust(
        client: &RemoteLoopbackClient,
    ) -> crate::realization_routing::RoutePlanReport {
        fixture_remote_operator_route_plan_with_trust(client, false)
    }

    fn fixture_remote_operator_route_plan_with_trust(
        client: &RemoteLoopbackClient,
        has_output_trust: bool,
    ) -> crate::realization_routing::RoutePlanReport {
        let upload_objects = u32::try_from(client.uploaded_input_refs.len()).expect("fixture upload count fits");
        let upload_summary = crate::realization_routing::UploadSummary::try_new(
            vec![crate::realization_routing::UploadClass::StoreObject],
            upload_objects,
            client.request.upload_bytes,
        )
        .expect("operator route upload summary is bounded");
        let trusted_output_key_count = if has_output_trust { 1 } else { 0 };
        let remote_facts = crate::realization_routing::RemoteBuilderPlanFacts {
            endpoint_id: "builder-1".to_string(),
            builder_configured: true,
            ticket_configured: true,
            concrete_inputs: true,
            capabilities_match: true,
            source_inputs_ready: true,
            upload_summary,
            trusted_output_key_count,
        };
        crate::realization_routing::plan_realization_route(crate::realization_routing::RoutePlannerInput::new(
            crate::realization_routing::RoutePolicy {
                network: crate::realization_routing::NetworkPolicy::Online,
                requested_claim_strength: crate::realization_routing::ClaimStrength::Practical,
                upload_policy: crate::realization_routing::UploadPolicy::allow_sources_and_store_objects(),
            },
            vec![
                crate::realization_routing::RouteCandidateFacts::rejected(
                    crate::realization_routing::RouteClass::CachedLocal,
                    "local-output-missing",
                ),
                crate::realization_routing::RouteCandidateFacts::rejected(
                    crate::realization_routing::RouteClass::TrustedSubstitute,
                    "trusted-substitute-missing",
                ),
                crate::realization_routing::RouteCandidateFacts::rejected(
                    crate::realization_routing::RouteClass::ArchiveImport,
                    "not-configured",
                ),
                crate::realization_routing::RouteCandidateFacts::rejected(
                    crate::realization_routing::RouteClass::SourceBundle,
                    "source-inputs-present-or-fetchable",
                ),
                crate::realization_routing::remote_builder_candidate_from_facts(&remote_facts),
                crate::realization_routing::RouteCandidateFacts::rejected(
                    crate::realization_routing::RouteClass::LocalBuild,
                    "operator-rail-selects-remote",
                ),
                crate::realization_routing::RouteCandidateFacts::eligible(
                    crate::realization_routing::RouteClass::PreflightError,
                    "no-route-eligible",
                ),
            ],
        ))
    }

    fn fixture_remote_operator_status(client: &RemoteLoopbackClient) -> RemoteCoordinatorStatusSnapshot {
        let temp = tempfile::tempdir().expect("operator rail state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("operator rail worker registers");
        let request = fixture_remote_operator_coordinator_request(client);
        let decision = admit_fixture_dispatch(&mut state, &request).expect("operator rail dispatch admits");
        let job_id = match decision {
            RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } => job_id,
            _ => panic!("operator rail request should dispatch"),
        };
        let start = fixture_attempt_report(&state, &job_id, "operator-start", RemoteAttemptReportPayload::Start);
        apply_fixture_attempt_report(&mut state, &start);
        let log = fixture_attempt_report(&state, &job_id, "operator-log", RemoteAttemptReportPayload::LogAppend {
            cursor: OPERATOR_RAIL_LOG_START_CURSOR,
            bytes: "remote fixture log".to_string(),
        });
        apply_fixture_attempt_report(&mut state, &log);
        let ready =
            fixture_attempt_report(&state, &job_id, "operator-result", RemoteAttemptReportPayload::ResultReady {
                output_digest_blake3: TEST_ATTEMPT_OUTPUT_DIGEST.to_string(),
            });
        apply_fixture_attempt_report(&mut state, &ready);
        assert_eq!(
            state.jobs[&job_id].immutable_log.as_ref().map(|log| log.next_cursor),
            Some(OPERATOR_RAIL_LOG_NEXT_CURSOR)
        );
        coordinator_status_snapshot("builder-1", DEFAULT_REMOTE_CONCURRENCY, &state, &[], &[fixture_ticket()])
            .expect("operator rail status renders")
    }

    #[test]
    fn remote_workspace_registration_and_fenced_lease_survive_restart() {
        let temp = tempfile::tempdir().unwrap();
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        let registration = crate::remote_farm_config::RemoteWorkspacePolicy {
            modes: vec![
                crate::remote_farm_config::RemoteWorkspaceMode::None,
                crate::remote_farm_config::RemoteWorkspaceMode::MutableSession,
            ],
            authority_class: "tenant-a".to_string(),
            ..crate::remote_farm_config::RemoteWorkspacePolicy::default()
        };
        let mut worker = fixture_worker_registration();
        worker.workspace_policy = Some(registration);
        apply_worker_registration(&mut state, worker).unwrap();
        let status = remote_worker_status(&state, &state.workers["builder-1"]).unwrap();
        assert!(status.workspace_modes.contains(&"mutable-session".to_string()));
        assert_eq!(status.workspace_authority_class.as_deref(), Some("tenant-a"));
        let request = crunch_build::WorkspaceLeaseRequest {
            workspace_id: "cargo-cache".to_string(),
            compatibility_digest_blake3: "a".repeat(BLAKE3_HEX_LENGTH_CHARS),
            toolchain_refs: vec!["mantle-object://blake3/rust".to_string()],
            guest_path: crunch_build::DEFAULT_WORKSPACE_GUEST_PATH.to_string(),
            quota: crunch_build::WorkspaceQuotaPolicy::default(),
            retention_class: "recent".to_string(),
            generation: 1,
            owner: crunch_build::WorkspaceLeaseOwner {
                worker_id: "builder-1".to_string(),
                authority_class: "tenant-a".to_string(),
                job_id: "job-a".to_string(),
                attempt_id: "attempt-a".to_string(),
                fence_generation: 2,
            },
            operation: crunch_build::WorkspaceLeaseOperation::Acquire,
        };
        let acquired = apply_remote_workspace_lease(&mut state, "builder-1", &request).unwrap();
        assert_eq!(acquired.disposition, crunch_build::WorkspaceLeaseDisposition::Accepted);
        let mut restarted = load_coordinator_state(temp.path()).unwrap();
        assert_eq!(restarted.workspace_leases["cargo-cache"].owner.as_ref().unwrap().attempt_id, "attempt-a");
        let mut stale = request.clone();
        stale.owner.fence_generation = 1;
        stale.operation = crunch_build::WorkspaceLeaseOperation::Renew;
        let before = restarted.clone();
        let rejected = apply_remote_workspace_lease(&mut restarted, "builder-1", &stale).unwrap();
        assert_eq!(rejected.disposition, crunch_build::WorkspaceLeaseDisposition::Rejected);
        assert_eq!(restarted, before);
    }

    #[test]
    fn worker_reregistration_without_workspace_policy_revokes_stale_authority() {
        let mut state = RemoteCoordinatorState::default();
        let mut admitted = fixture_worker_registration();
        admitted.workspace_policy = Some(crate::remote_farm_config::RemoteWorkspacePolicy {
            modes: vec![crate::remote_farm_config::RemoteWorkspaceMode::MutableSession],
            authority_class: "tenant-a".to_string(),
            ..crate::remote_farm_config::RemoteWorkspacePolicy::default()
        });
        apply_worker_registration(&mut state, admitted).unwrap();
        assert!(state.workspace_registrations.contains_key("builder-1"));

        apply_worker_registration(&mut state, fixture_worker_registration()).unwrap();

        assert!(!state.workspace_registrations.contains_key("builder-1"));
        assert!(state.workers["builder-1"].workspace_policy.is_none());
    }

    #[test]
    fn remote_workspace_binding_uses_current_job_attempt_and_fence() {
        let mut state = RemoteCoordinatorState::default();
        let policy = crate::remote_farm_config::RemoteWorkspacePolicy {
            modes: vec![crate::remote_farm_config::RemoteWorkspaceMode::MutableSession],
            authority_class: "tenant-a".to_string(),
            ..crate::remote_farm_config::RemoteWorkspacePolicy::default()
        };
        let mut worker = fixture_worker_registration();
        worker.workspace_policy = Some(policy);
        apply_worker_registration(&mut state, worker).unwrap();
        let decision = admit_fixture_dispatch(&mut state, &fixture_coordinator_request()).unwrap();
        let RemoteCoordinatorDispatchDecision::Dispatch {
            job_id,
            attempt_id,
            fence_generation,
            ..
        } = decision
        else {
            panic!("fixture dispatch expected");
        };
        let mut workspace = snix_build::buildservice::StatefulWorkspaceRequest {
            mode: snix_build::buildservice::StatefulWorkspaceMode::MutableSession,
            workspace_id: Some("cargo-cache-bound".to_string()),
            guest_path: crunch_build::DEFAULT_WORKSPACE_GUEST_PATH.into(),
            snapshot_input_name: None,
            compatibility_digest_blake3: "b".repeat(BLAKE3_HEX_LENGTH_CHARS),
            toolchain_refs: Vec::new(),
            quota_bytes_max: crunch_build::DEFAULT_WORKSPACE_BYTES_MAX,
            quota_files_max: crunch_build::DEFAULT_WORKSPACE_FILES_MAX,
            quota_snapshots_max: crunch_build::DEFAULT_WORKSPACE_SNAPSHOTS_MAX,
            retention_class: "recent".to_string(),
            retention_workspace_count_max: crunch_build::DEFAULT_WORKSPACE_COUNT_MAX,
            retention_idle_generations_max: crunch_build::DEFAULT_WORKSPACE_IDLE_GENERATIONS_MAX,
            retention_age_generations_max: crunch_build::DEFAULT_WORKSPACE_AGE_GENERATIONS_MAX,
            retention_quarantine_count_max: crunch_build::DEFAULT_WORKSPACE_QUARANTINE_COUNT_MAX,
            generation: 1,
            lease: None,
            sensitive_paths: Vec::new(),
            secret_markers: Vec::new(),
            scan_depth_max: crunch_build::DEFAULT_WORKSPACE_SCAN_DEPTH_MAX,
            path_bytes_max: crunch_build::DEFAULT_WORKSPACE_PATH_BYTES_MAX,
            snapshot_enabled: false,
            clean_rebuild_enabled: false,
            clean_rebuild_require_declared_inputs: true,
            runtime_host_path: None,
        };
        let plan = bind_and_acquire_remote_workspace(&mut state, &job_id, &mut workspace).unwrap();
        assert_eq!(plan.disposition, crunch_build::WorkspaceLeaseDisposition::Accepted);
        let lease = workspace.lease.unwrap();
        assert_eq!(lease.worker_id, "builder-1");
        assert_eq!(lease.job_id, job_id.as_str());
        assert_eq!(lease.attempt_id, attempt_id.as_str());
        assert_eq!(lease.fence_generation, fence_generation.get());
    }

    #[test]
    fn mutable_workspace_jobs_never_attach_or_publish_live_output_claims() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).unwrap();
        let mut request = fixture_coordinator_request();
        let RemoteConcreteBuildPayload::Action { spec_json, .. } = &mut request.request.payload else {
            panic!("fixture action expected");
        };
        let mut spec: serde_json::Value = serde_json::from_str(spec_json).unwrap();
        spec["workspace"] = serde_json::json!({"mode": "mutable-session"});
        *spec_json = serde_json::to_string(&spec).unwrap();
        let first = admit_coordinator_dispatch_with_nonce(
            &mut state,
            &request,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
            fixture_assignment_nonce("mutable-first"),
        )
        .unwrap();
        let second = plan_coordinator_dispatch(&state, &request, &fixture_assignment_nonce("mutable-second")).unwrap();
        let RemoteCoordinatorDispatchDecision::Dispatch {
            normalized_build_key: first_key,
            ..
        } = first
        else {
            panic!("first mutable request dispatches");
        };
        assert!(!matches!(
            second,
            RemoteCoordinatorDispatchDecision::AttachExisting { .. }
                | RemoteCoordinatorDispatchDecision::RedeliverResult { .. }
        ));
        let shared_key = normalized_remote_build_key(&request).unwrap();
        let second_key = mutable_workspace_job_key((
            &shared_key,
            &request.request.request_id,
            &fixture_assignment_nonce("mutable-second"),
        ));
        assert_ne!(first_key, second_key);
        assert!(state.live_output_claims.is_empty());
        assert!(state.jobs.values().all(|job| job.live_output_claims.is_empty()));
    }

    struct FakeExternalBatchDispatcher {
        submit_calls: std::cell::Cell<u32>,
        followup_calls: std::cell::Cell<u32>,
        submit_state: ExternalBatchJobState,
        followup_state: ExternalBatchJobState,
        external_job_id: String,
        stale_fence: bool,
    }

    impl FakeExternalBatchDispatcher {
        fn valid() -> Self {
            Self {
                submit_calls: std::cell::Cell::new(0),
                followup_calls: std::cell::Cell::new(0),
                submit_state: ExternalBatchJobState::Submitted,
                followup_state: ExternalBatchJobState::Running,
                external_job_id: "fake-scheduler-42".to_string(),
                stale_fence: false,
            }
        }

        fn response(
            &self,
            operation: &ExternalBatchOperation,
            state: ExternalBatchJobState,
            observed_unix_s: u64,
        ) -> ExternalBatchOperationResponse {
            let fence_generation = if self.stale_fence {
                operation.fence_generation.advance().expect("fake stale fence advances")
            } else {
                operation.fence_generation
            };
            ExternalBatchOperationResponse {
                schema: crunch_build::distributed::EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
                operation: operation.operation,
                operation_id_blake3: operation.operation_id_blake3.clone(),
                dispatch_id_blake3: operation.dispatch_id_blake3.clone(),
                adapter_instance_id: operation.adapter_instance_id.clone(),
                dispatcher_generation: operation.dispatcher_generation,
                job_id: operation.job_id.clone(),
                attempt_id: operation.attempt_id.clone(),
                fence_generation,
                state,
                external_job_id: Some(self.external_job_id.clone()),
                reason_code: "fake-scheduler-state".to_string(),
                observed_unix_s,
                non_claim: EXTERNAL_BATCH_NON_CLAIM.to_string(),
            }
        }
    }

    impl ExternalBatchDispatcher for FakeExternalBatchDispatcher {
        fn submit(
            &self,
            operation: &ExternalBatchOperation,
            observed_unix_s: u64,
        ) -> Result<ExternalBatchOperationResponse, String> {
            self.submit_calls.set(self.submit_calls.get().saturating_add(1));
            Ok(self.response(operation, self.submit_state, observed_unix_s))
        }

        fn observe(
            &self,
            operation: &ExternalBatchOperation,
            observed_unix_s: u64,
        ) -> Result<ExternalBatchOperationResponse, String> {
            self.followup_calls.set(self.followup_calls.get().saturating_add(1));
            Ok(self.response(operation, self.followup_state, observed_unix_s))
        }

        fn cancel(
            &self,
            operation: &ExternalBatchOperation,
            observed_unix_s: u64,
        ) -> Result<ExternalBatchOperationResponse, String> {
            self.followup_calls.set(self.followup_calls.get().saturating_add(1));
            Ok(self.response(operation, ExternalBatchJobState::Cancelled, observed_unix_s))
        }

        fn reconcile(
            &self,
            operation: &ExternalBatchOperation,
            observed_unix_s: u64,
        ) -> Result<ExternalBatchOperationResponse, String> {
            self.followup_calls.set(self.followup_calls.get().saturating_add(1));
            Ok(self.response(operation, self.followup_state, observed_unix_s))
        }
    }

    fn fixture_batch_dispatcher_profile() -> crate::remote_farm_config::RemoteBatchDispatcherProfile {
        let command = crate::remote_farm_config::RemoteBatchDispatcherCommandProfile {
            program: PathBuf::from("/bin/true"),
            expected_digest_blake3: blake3::hash(b"fixture").to_hex().to_string(),
            args: Vec::new(),
            timeout_secs: TEST_EXTERNAL_BATCH_TIMEOUT_SECS,
            stdout_limit_bytes: TEST_EXTERNAL_BATCH_OUTPUT_LIMIT_BYTES,
            stderr_limit_bytes: TEST_EXTERNAL_BATCH_OUTPUT_LIMIT_BYTES,
        };
        crate::remote_farm_config::RemoteBatchDispatcherProfile {
            instance_id: "fake-batch".to_string(),
            generation: 1,
            adapter: crate::remote_farm_config::RemoteBatchDispatcherAdapter::DirectProcessV1,
            protocol_schema: crunch_build::distributed::EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
            allowed_operations: vec![
                ExternalBatchOperationKind::Submit,
                ExternalBatchOperationKind::Observe,
                ExternalBatchOperationKind::Cancel,
                ExternalBatchOperationKind::Reconcile,
            ],
            provider_class: "fake".to_string(),
            submit: command.clone(),
            observe: command.clone(),
            cancel: command.clone(),
            reconcile: command,
            worker_program: PathBuf::from("/bin/true"),
            worker_program_digest_blake3: blake3::hash(b"fixture-worker").to_hex().to_string(),
            worker_args: Vec::new(),
            environment_handles: Vec::new(),
            bootstrap_policy: crate::remote_farm_config::RemoteBatchBootstrapPolicy::DirectWorkerExecV1,
            redact_provider_output: true,
            startup_timeout_secs: TEST_EXTERNAL_BATCH_TIMEOUT_SECS,
            terminal_timeout_secs: TEST_EXTERNAL_BATCH_TIMEOUT_SECS,
            max_reconcile_attempts: TEST_EXTERNAL_BATCH_MAX_ATTEMPTS,
        }
    }

    fn fixture_multiprocess_slurm_profile(
        temp: &tempfile::TempDir,
    ) -> crate::remote_farm_config::RemoteBatchDispatcherProfile {
        let script = temp.path().join("fake-slurm-cli");
        std::fs::write(
            &script,
            "#!/bin/sh\ncase \"$1\" in\n submit) printf '4242;fixture\\n' ;;\n observe|reconcile) printf 'RUNNING\\n' ;;\n cancel) exit 0 ;;\n *) exit 64 ;;\nesac\n",
        )
        .expect("fake Slurm CLI writes");
        let mut permissions = std::fs::metadata(&script).expect("fake Slurm metadata").permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&script, permissions).expect("fake Slurm mode applies");
        let digest = blake3::hash(&std::fs::read(&script).expect("fake Slurm bytes read")).to_hex().to_string();
        let mut profile = fixture_batch_dispatcher_profile();
        profile.adapter = crate::remote_farm_config::RemoteBatchDispatcherAdapter::SlurmCliV1;
        for (command, operation) in [
            (&mut profile.submit, "submit"),
            (&mut profile.observe, "observe"),
            (&mut profile.cancel, "cancel"),
            (&mut profile.reconcile, "reconcile"),
        ] {
            command.program = script.clone();
            command.expected_digest_blake3 = digest.clone();
            command.args = vec![operation.to_string()];
        }
        profile
    }

    fn fixture_external_batch_worker() -> RemoteWorkerRegistration {
        let mut worker = fixture_worker_registration();
        worker.endpoint_id = "batch-worker-1".to_string();
        worker.worker_generation = 1;
        worker.concurrency = TEST_WORKER_CONCURRENCY;
        worker.resource_inventory = Some(fixture_resource_inventory());
        worker
    }

    #[test]
    fn external_batch_submit_is_durable_idempotent_and_restart_reconciles() {
        let temp = tempfile::tempdir().expect("external batch coordinator state");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        let request = fixture_quantified_coordinator_request("external-batch", "external-batch-claim");
        let profile = fixture_batch_dispatcher_profile();
        let dispatcher = FakeExternalBatchDispatcher::valid();
        let first = submit_external_batch_allocation(
            &mut state,
            &request,
            &profile,
            "batch-worker-1",
            1,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect("external allocation submits");
        let second = submit_external_batch_allocation(
            &mut state,
            &request,
            &profile,
            "batch-worker-1",
            1,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect("duplicate submission is idempotent");
        let mut restarted = load_coordinator_state(temp.path()).expect("external allocation state reloads");
        let decision = reconcile_external_batch_allocation(
            &mut restarted,
            &first.dispatch_id_blake3,
            &dispatcher,
            fixture_retry_time().now_unix_s,
            false,
        )
        .expect("restart reconciliation observes scheduler");

        assert_eq!(first, second);
        assert_eq!(dispatcher.submit_calls.get(), 1);
        assert_eq!(dispatcher.followup_calls.get(), 1);
        assert_eq!(decision, ExternalBatchReconcileDecision::AwaitWorkerRegistration);
        assert_eq!(restarted.external_batch_attempts[&first.dispatch_id_blake3].reconcile_attempts, 1);
    }

    #[test]
    fn external_batch_worker_must_register_before_normal_assignment_and_transfer() {
        let mut state = RemoteCoordinatorState::default();
        let request = fixture_quantified_coordinator_request("external-worker", "external-worker-claim");
        let profile = fixture_batch_dispatcher_profile();
        let dispatcher = FakeExternalBatchDispatcher::valid();
        let response = submit_external_batch_allocation(
            &mut state,
            &request,
            &profile,
            "batch-worker-1",
            1,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect("external allocation submits");
        let before_registration =
            authorize_external_batch_transfer(&state, &response.dispatch_id_blake3, "batch-worker-1")
                .expect_err("transfer blocked before worker registration");
        register_external_batch_worker(&mut state, &response.dispatch_id_blake3, fixture_external_batch_worker())
            .expect("allocated worker registers");
        authorize_external_batch_transfer(&state, &response.dispatch_id_blake3, "batch-worker-1")
            .expect("registered worker authorizes transfer seam");
        let decision = admit_external_batch_coordinator_dispatch(
            &mut state,
            &response.dispatch_id_blake3,
            &request,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("registered external worker enters normal coordinator assignment");

        let RemoteCoordinatorDispatchDecision::Dispatch {
            worker_endpoint_id,
            job_id,
            attempt_id,
            fence_generation,
            ..
        } = decision
        else {
            panic!("external worker must receive a normal coordinator dispatch");
        };
        let binding = RemoteProductionAttemptBinding {
            job_id,
            attempt_id,
            fence_generation,
        };
        let cancellation = terminate_external_batch_assigned_attempt(
            &mut state,
            &response.dispatch_id_blake3,
            &binding,
            RemoteCoordinatorTerminationCause::Timeout,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect("timeout cancels scheduler allocation and coordinator attempt");

        assert_eq!(before_registration, "external-batch-worker-not-registered");
        assert_eq!(worker_endpoint_id, "batch-worker-1");
        assert_eq!(cancellation.state, ExternalBatchJobState::Cancelled);
        assert_eq!(state.jobs[&binding.job_id].phase, RemoteCoordinatorJobPhase::Lost);
        assert_eq!(state.jobs[&binding.job_id].short_error.as_deref(), Some("remote-attempt-timed-out"));
        assert_eq!(state.workers.len(), 1);
    }

    #[test]
    fn external_batch_stale_response_and_scheduler_id_conflict_leave_state_unchanged() {
        let mut state = RemoteCoordinatorState::default();
        let request = fixture_quantified_coordinator_request("external-stale", "external-stale-claim");
        let profile = fixture_batch_dispatcher_profile();
        let mut stale = FakeExternalBatchDispatcher::valid();
        stale.stale_fence = true;
        let stale_error = submit_external_batch_allocation(
            &mut state,
            &request,
            &profile,
            "batch-worker-1",
            1,
            &stale,
            fixture_retry_time().now_unix_s,
        )
        .expect_err("stale adapter response rejected");
        assert!(state.external_batch_attempts.is_empty());

        let dispatcher = FakeExternalBatchDispatcher::valid();
        let accepted = submit_external_batch_allocation(
            &mut state,
            &request,
            &profile,
            "batch-worker-1",
            1,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect("first scheduler identity accepted");
        let before_conflict = state.clone();
        let conflict_request = fixture_quantified_coordinator_request("external-conflict", "external-conflict-claim");
        let conflict = submit_external_batch_allocation(
            &mut state,
            &conflict_request,
            &profile,
            "batch-worker-1",
            1,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect_err("same scheduler id cannot bind another dispatch");

        assert_eq!(stale_error, "external-batch-response-attempt-fence-mismatch");
        assert_eq!(conflict, "external-batch-state-external-job-id-conflict");
        assert_eq!(state, before_conflict);
        assert!(state.external_batch_attempts.contains_key(&accepted.dispatch_id_blake3));
    }

    #[tokio::test]
    async fn external_batch_multiprocess_allocation_uses_ordinary_cas_and_output_admission() {
        let temp = tempfile::tempdir().expect("multiprocess external allocation dir");
        let mut client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        let requirements = fixture_resource_requirements();
        client.request.resource_requirements = Some(requirements.clone());
        let mut request = fixture_remote_operator_coordinator_request(&client);
        request.resource_requirements = Some(requirements);
        request.request.resource_requirements = request.resource_requirements.clone();
        let profile = fixture_multiprocess_slurm_profile(&temp);
        let dispatcher = ConfiguredExternalBatchDispatcher::new(&profile);
        let mut state = RemoteCoordinatorState::default();
        let allocation = submit_external_batch_allocation(
            &mut state,
            &request,
            &profile,
            "batch-worker-1",
            1,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect("fake scheduler subprocess allocates worker");
        let blocked = authorize_external_batch_transfer(&state, &allocation.dispatch_id_blake3, "batch-worker-1")
            .expect_err("CAS transfer stays blocked before registration");
        register_external_batch_worker(&mut state, &allocation.dispatch_id_blake3, fixture_external_batch_worker())
            .expect("ordinary worker registration succeeds");
        let decision = admit_external_batch_coordinator_dispatch(
            &mut state,
            &allocation.dispatch_id_blake3,
            &request,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("ordinary coordinator assignment succeeds");
        let RemoteCoordinatorDispatchDecision::Dispatch {
            job_id,
            attempt_id,
            fence_generation,
            ..
        } = decision
        else {
            panic!("registered external worker dispatches");
        };
        let binding = RemoteProductionAttemptBinding {
            job_id,
            attempt_id,
            fence_generation,
        };
        start_remote_production_attempt_if_queued(&mut state, &binding, fixture_retry_time().now_unix_s)
            .expect("ordinary worker starts fenced attempt");
        let builder = fixture_loopback_builder();
        let executor = PathInfoRemoteExecutor {
            path_info: importable_pathinfo(),
            artifact_attestation_digest_blake3: None,
            nar_payload: None,
        };
        let mut ticket_state = RemoteTicketState::default();
        ticket_state.tickets.insert("ticket-1".to_string(), fixture_ticket());
        let response = plan_stdio_remote_once_from_state_with_executor(
            std::io::Cursor::new(encode_frame_stream(&remote_client_request_frames(&client))),
            &builder,
            &mut ticket_state,
            client.transfer_capabilities,
            &executor,
        )
        .expect("worker produces signed PathInfo through framed protocol");
        let report = production_attempt_report(
            &binding,
            "external-batch-result",
            fixture_retry_time().now_unix_s,
            RemoteAttemptReportPayload::ResultReady {
                output_digest_blake3: response.output_digest_blake3.clone(),
            },
        )
        .expect("fenced result report derives");
        let admitted = admit_fenced_remote_builder_response(
            &mut state,
            &report,
            true,
            RemoteLogRetentionPolicy::default(),
            &client.request,
            &client.trusted_output_keys,
            &response,
        )
        .expect("ordinary fenced output admission succeeds");
        let RemoteFencedOutputAdmissionDecision::Admit { admission, .. } = admitted else {
            panic!("fresh fenced output must admit");
        };
        let admission = *admission;
        let mut store = remote_import_store(temp.path()).await;
        let imported = import_admitted_remote_outputs(
            &mut store,
            &client.request,
            &admission,
            true,
            Some(crunch_store::GcRootSource::Build),
        )
        .await
        .expect("ordinary CAS output import succeeds");
        let evidence =
            external_batch_composition_evidence(&mut state, &allocation.dispatch_id_blake3, &binding, &admission)
                .expect("admitted CAS output composes with allocation evidence");

        assert_eq!(blocked, "external-batch-worker-not-registered");
        assert_eq!(allocation.external_job_id.as_deref(), Some("4242"));
        assert_eq!(imported.outputs.len(), 1);
        assert_eq!(evidence.transfer_mode, admission.transfer.mode);
        assert_eq!(evidence.output_trust_key_id, "builder-key");
        assert!(evidence.non_claims.iter().any(|claim| claim.contains("do not authorize workers")));
    }

    #[test]
    fn external_batch_status_and_composition_keep_scheduler_identity_non_authoritative() {
        let temp = tempfile::tempdir().expect("external diagnostic state dir");
        let mut state = RemoteCoordinatorState {
            state_dir: Some(temp.path().to_path_buf()),
            ..RemoteCoordinatorState::default()
        };
        let request = fixture_quantified_coordinator_request("external-evidence", "external-evidence-claim");
        let profile = fixture_batch_dispatcher_profile();
        let dispatcher = FakeExternalBatchDispatcher::valid();
        let response = submit_external_batch_allocation(
            &mut state,
            &request,
            &profile,
            "batch-worker-1",
            1,
            &dispatcher,
            fixture_retry_time().now_unix_s,
        )
        .expect("external allocation submits");
        register_external_batch_worker(&mut state, &response.dispatch_id_blake3, fixture_external_batch_worker())
            .expect("worker registers");
        let decision = admit_external_batch_coordinator_dispatch(
            &mut state,
            &response.dispatch_id_blake3,
            &request,
            RemoteAttemptRetryPolicy::default(),
            fixture_retry_time(),
        )
        .expect("normal assignment succeeds");
        let RemoteCoordinatorDispatchDecision::Dispatch {
            job_id,
            attempt_id,
            fence_generation,
            ..
        } = decision
        else {
            panic!("external worker must receive normal dispatch");
        };
        let binding = RemoteProductionAttemptBinding {
            job_id,
            attempt_id,
            fence_generation,
        };
        start_remote_production_attempt_if_queued(&mut state, &binding, fixture_retry_time().now_unix_s)
            .expect("ordinary worker starts diagnostic attempt");
        let diagnostic = append_external_batch_attempt_diagnostic(&mut state, &response.dispatch_id_blake3, &binding)
            .expect("external allocation diagnostic appends immutably");
        state.jobs.get_mut(&binding.job_id).expect("assigned job exists").output_admission_completed = true;
        let output_digest = blake3::hash(b"admitted-output").to_hex().to_string();
        let admission = RemoteOutputAdmissionReport {
            request_id: request.request.request_id.clone(),
            output_digest_blake3: output_digest.clone(),
            builder_signing_key_id: "builder-key".to_string(),
            trust_basis: RemoteOutputTrustBasis {
                key_id: "builder-key".to_string(),
                key_material_digest_blake3: None,
            },
            store_prefix: request.request.store_prefix.clone(),
            outputs: Vec::new(),
            transfer_artifacts: Vec::new(),
            streamed_manifest: None,
            transfer: full_transfer_report(1, "builder-key", None),
        };
        let evidence =
            external_batch_composition_evidence(&mut state, &response.dispatch_id_blake3, &binding, &admission)
                .expect("composition evidence links only validated seams");
        let status = coordinator_status_snapshot("coordinator", 1, &state, &[], &[]).expect("external status renders");
        let plan =
            external_batch_plan_report(&state.external_batch_attempts[&response.dispatch_id_blake3].submit_operation);
        let encoded = serde_json::to_string(&status).expect("status serializes");

        assert_eq!(plan.schema, EXTERNAL_BATCH_PLAN_REPORT_SCHEMA);
        assert_eq!(plan.resources.cpu_units, TEST_RESOURCE_CPU_UNITS);
        assert_eq!(evidence.output_admission_digest_blake3, output_digest);
        assert!(diagnostic.head_record_blake3.is_some());
        assert!(diagnostic.next_cursor > 0);
        assert!(evidence.non_claims.iter().any(|claim| claim.contains("do not authorize workers")));
        assert_eq!(status.external_batch_attempts.len(), 1);
        assert!(!encoded.contains("do-not-store"));
        assert!(!encoded.contains("private-key-material"));
    }

    fn fixture_remote_operator_coordinator_request(client: &RemoteLoopbackClient) -> RemoteCoordinatorBuildRequest {
        RemoteCoordinatorBuildRequest {
            request: client.request.clone(),
            required_system: DEFAULT_REMOTE_ACTION_SYSTEM.to_string(),
            required_features: vec!["kvm".to_string()],
            required_sandbox_mode: "bwrap".to_string(),
            required_network_mode: "off".to_string(),
            resource_requirements: None,
            locality_scope: None,
            trusted_output_keys: vec!["builder-key".to_string()],
            live_output_claims: vec!["claim-out".to_string()],
            wait_for_worker: false,
        }
    }
}
