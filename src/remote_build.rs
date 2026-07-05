use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
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
use std::time::Instant;

use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use snix_store::path_info::PathInfo;
use tokio::io::AsyncWrite;

use crate::errors::RunError;

pub const REMOTE_PROTOCOL_ALPN: &str = "mantle-remote-build/1";
pub const REMOTE_PROTOCOL_VERSION: u32 = 1;
pub const MAX_REMOTE_CAPABILITIES: usize = 32;
pub const MAX_REMOTE_INPUT_REFS: usize = 1_000_000;
pub const MAX_REMOTE_EXPECTED_OUTPUTS: usize = 128;
pub const MAX_REMOTE_BUILD_PAYLOAD_BYTES: usize = 1_048_576;
pub const MAX_REMOTE_FRAME_BYTES: usize = 1_048_576;
const REMOTE_JSON_BYTE_ARRAY_MAX_CHARS_PER_BYTE: usize = 4;
const REMOTE_INLINE_NAR_FRAME_METADATA_RESERVE_BYTES: usize = 4_096;
pub const MAX_REMOTE_INLINE_NAR_PAYLOAD_BYTES: usize = (MAX_REMOTE_FRAME_BYTES
    / REMOTE_JSON_BYTE_ARRAY_MAX_CHARS_PER_BYTE)
    - REMOTE_INLINE_NAR_FRAME_METADATA_RESERVE_BYTES;
pub const MAX_REMOTE_STDIO_STDERR_BYTES: usize = 65_536;
pub const MAX_REMOTE_STDIO_FRAME_COUNT: usize = 4_096;
pub const MAX_REMOTE_STDIO_INPUT_BYTES: usize = 4_194_304;
pub const MAX_REMOTE_STATUS_ITEMS: usize = 4_096;
pub const MAX_REMOTE_EXECUTOR_ARGS: usize = 512;
pub const MAX_REMOTE_EXECUTOR_ENV_VARS: usize = 512;
pub const MAX_REMOTE_EXECUTOR_ENV_VALUE_BYTES: usize = 65_536;
pub const MAX_REMOTE_UPLOAD_BYTES: u64 = 1_099_511_627_776;
pub const MAX_REMOTE_TRANSFER_TOTAL_BYTES: u64 = MAX_REMOTE_UPLOAD_BYTES;
pub const MAX_REMOTE_TRANSFER_ARTIFACTS: usize = MAX_REMOTE_STATUS_ITEMS;
pub const MAX_REMOTE_INPUT_UPLOAD_ARTIFACTS: usize = MAX_REMOTE_STATUS_ITEMS;
pub const MAX_REMOTE_BUILD_TIME_SECS: u64 = 86_400;
pub const DEFAULT_TICKET_TTL_SECS: u64 = 3_600;
pub const DEFAULT_TICKET_USES: u32 = 1;
pub const DEFAULT_TICKET_BUILD_TIME_SECS: u64 = 3_600;
pub const DEFAULT_REMOTE_STDIO_TIMEOUT_SECS: u64 = DEFAULT_TICKET_BUILD_TIME_SECS;
pub const DEFAULT_TICKET_UPLOAD_BYTES: u64 = 1_073_741_824;
pub const DEFAULT_REMOTE_CONCURRENCY: u32 = 1;
pub const MAX_TICKET_DISPLAY_NAME_BYTES: usize = 128;
const TICKET_STATE_DIR: &str = "remote-builders";
const TICKET_STATE_FILE: &str = "tickets.json";
const COORDINATOR_STATE_FILE: &str = "remote-coordinator-state.json";
const SECRET_REDACTION: &str = "<redacted>";
const TEMP_FILE_EXTENSION: &str = "tmp";
const REMOTE_FRAME_HEADER_BYTES: usize = std::mem::size_of::<u32>();
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
const REMOTE_CLIENT_REQUEST_ID_LABEL: &str = "remote-client-request";
const REMOTE_CLIENT_SESSION_ID_LABEL: &str = "remote-client-session";
const REMOTE_TRANSFER_ARTIFACTS_PER_PATHINFO_OUTPUT: usize = 2;
const MAX_REMOTE_WORKER_CONCURRENCY: u32 = 1_024;
const MAX_REMOTE_LOG_CHUNKS: usize = 1_024;
const MAX_REMOTE_LOG_BYTES: u64 = 1_048_576;
const MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES: usize = 512;
const REMOTE_COORDINATOR_BUILD_KEY_LABEL: &str = "remote-coordinator-build-key";
const REMOTE_COORDINATOR_JOB_ID_LABEL: &str = "remote-coordinator-job";
const REMOTE_CHILD_POLL_INTERVAL_MS: u64 = 10;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteHello {
    pub alpn: String,
    pub version: u32,
    pub endpoint_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedHello {
    pub endpoint_id: String,
    pub accepted_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTicket {
    pub id: String,
    pub display_name: String,
    pub secret: String,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub uses_remaining: u32,
    pub max_build_time_secs: u64,
    pub max_upload_bytes: u64,
    pub bound_client_endpoint: Option<String>,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteTicketView {
    pub id: String,
    pub display_name: String,
    pub secret: String,
    pub created_unix_s: u64,
    pub expires_unix_s: u64,
    pub uses_remaining: u32,
    pub max_build_time_secs: u64,
    pub max_upload_bytes: u64,
    pub bound_client_endpoint: Option<String>,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RemoteTicketState {
    pub tickets: BTreeMap<String, RemoteTicket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketAuthRequest {
    pub ticket_id: String,
    pub secret: String,
    pub client_endpoint: Option<String>,
    pub now_unix_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcreteBuildRequest {
    pub request_id: String,
    pub store_prefix: String,
    pub input_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_input_refs: Vec<String>,
    pub upload_bytes: u64,
    pub build_time_limit_secs: u64,
    pub contains_raw_frontend_eval: bool,
    pub payload: RemoteConcreteBuildPayload,
    pub expected_outputs: Vec<RemoteExpectedOutput>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_info: Option<PathInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path_info: Option<PathInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nar_payload_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
        execute_remote_local_build(self, request, plan, input_upload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RemoteActionSpec {
    pub schema: String,
    pub action_id: String,
    pub builder: String,
    #[serde(default = "default_remote_action_system")]
    pub system: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
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
    AuthTicket,
    AuthOk,
    BuildRequest,
    InputManifest,
    MissingInputs,
    InputUpload,
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
            Self::AuthTicket => "auth-ticket",
            Self::AuthOk => "auth-ok",
            Self::BuildRequest => "build-request",
            Self::InputManifest => "input-manifest",
            Self::MissingInputs => "missing-inputs",
            Self::InputUpload => "input-upload",
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<RemoteInputUploadArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteInputUploadArtifactKind {
    Nar,
}

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
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RemoteFrame {
    Hello { hello: RemoteHello },
    AuthTicket { auth: TicketAuthRequest },
    AuthOk { auth: RemoteAuthOk },
    BuildRequest { request: ConcreteBuildRequest },
    InputManifest { manifest: RemoteInputManifest },
    MissingInputs { request_id: String, refs: Vec<String> },
    InputUpload { upload: RemoteInputUpload },
    BuildQueued { request_id: String, session_id: String },
    BuildStarted { request_id: String },
    BuildFinished { result: RemoteBuildFinished },
    OutputTransferArtifact { artifact: RemoteOutputTransferArtifact },
    OutputTransferDone { report: RemoteTransferReport },
    Done { request_id: String },
    Error { error: RemoteProtocolErrorFrame },
}

impl RemoteFrame {
    pub fn kind(&self) -> RemoteFrameKind {
        match self {
            Self::Hello { .. } => RemoteFrameKind::Hello,
            Self::AuthTicket { .. } => RemoteFrameKind::AuthTicket,
            Self::AuthOk { .. } => RemoteFrameKind::AuthOk,
            Self::BuildRequest { .. } => RemoteFrameKind::BuildRequest,
            Self::InputManifest { .. } => RemoteFrameKind::InputManifest,
            Self::MissingInputs { .. } => RemoteFrameKind::MissingInputs,
            Self::InputUpload { .. } => RemoteFrameKind::InputUpload,
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
}

impl RemoteTransferMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Delta => "delta",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferCapabilities {
    pub delta: bool,
    pub full: bool,
    pub simulate_delta_failure: bool,
}

impl RemoteTransferCapabilities {
    pub fn delta_and_full() -> Self {
        Self {
            delta: true,
            full: true,
            simulate_delta_failure: false,
        }
    }

    pub fn full_only() -> Self {
        Self {
            delta: false,
            full: true,
            simulate_delta_failure: false,
        }
    }

    fn as_capability_labels(self) -> Vec<String> {
        let mut labels = Vec::new();
        if self.delta {
            labels.push(RemoteTransferMode::Delta.as_str().to_string());
        }
        if self.full {
            labels.push(RemoteTransferMode::Full.as_str().to_string());
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
pub struct RemoteStdioTranscript {
    pub binding: RemoteTransportBinding,
    pub frames: Vec<RemoteFrame>,
    pub stderr_summary: String,
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
pub struct RemoteStdioCommand {
    pub binding: RemoteTransportBinding,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub input_frames: Vec<RemoteFrame>,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTicketCredential {
    pub ticket_id: String,
    pub secret: String,
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
    Ok(RemoteStdioBuilderCommand {
        endpoint_id: request.endpoint_id,
        program: request.ssh_program,
        args,
    })
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
pub struct RemoteClientDispatchPlan {
    pub label: String,
    pub command: RemoteStdioCommand,
    pub client: RemoteLoopbackClient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteClientImportedBuild {
    pub label: String,
    pub request_id: String,
    pub imported: RemoteOutputImportReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteClientBuildReport {
    pub schema: String,
    pub builder: String,
    pub store_prefix: String,
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
    pub job_id: String,
    pub normalized_build_key: String,
    pub phase: RemoteCoordinatorJobPhase,
    pub result_available: bool,
    pub log_next_cursor: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWorkerRegistration {
    pub endpoint_id: String,
    pub protocol_version: u32,
    pub systems: Vec<String>,
    pub feature_labels: Vec<String>,
    pub sandbox_modes: Vec<String>,
    pub network_modes: Vec<String>,
    pub logical_store_prefixes: Vec<String>,
    pub concurrency: u32,
    pub output_signing_key_ids: Vec<String>,
    pub resumable_jobs: Vec<RemoteWorkerResumeSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorBuildRequest {
    pub request: ConcreteBuildRequest,
    pub required_system: String,
    pub required_features: Vec<String>,
    pub required_sandbox_mode: String,
    pub required_network_mode: String,
    pub trusted_output_keys: Vec<String>,
    pub live_output_claims: Vec<String>,
    pub wait_for_worker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorJobSummary {
    pub job_id: String,
    pub normalized_build_key: String,
    pub assigned_worker_endpoint_id: Option<String>,
    pub phase: RemoteCoordinatorJobPhase,
    pub live_output_claims: Vec<String>,
    pub result_available: bool,
    pub lost_phase: Option<RemoteFailurePhase>,
    pub log_start_cursor: u64,
    pub log_next_cursor: u64,
    pub short_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RemoteCoordinatorState {
    pub workers: BTreeMap<String, RemoteWorkerRegistration>,
    pub jobs: BTreeMap<String, RemoteCoordinatorJobSummary>,
    pub live_output_claims: BTreeMap<String, String>,
    pub logs: BTreeMap<String, Vec<RemoteCoordinatorLogChunk>>,
    /// When set, mutations auto-save to this directory for durability.
    #[serde(skip)]
    pub state_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "kebab-case")]
pub enum RemoteCoordinatorDispatchDecision {
    Dispatch {
        worker_endpoint_id: String,
        job_id: String,
        normalized_build_key: String,
    },
    AttachExisting {
        job_id: String,
        normalized_build_key: String,
    },
    RedeliverResult {
        job_id: String,
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
pub struct RemoteCoordinatorLogChunk {
    pub cursor: u64,
    pub bytes: String,
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
pub struct RemoteLogReplayPlan {
    pub replay_chunks: Vec<RemoteCoordinatorLogChunk>,
    pub retained_chunks: Vec<RemoteCoordinatorLogChunk>,
    pub truncated: bool,
    pub dropped_bytes: u64,
    pub next_cursor: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCoordinatorJobStatus {
    pub job_id: String,
    pub phase: RemoteCoordinatorJobPhase,
    pub worker_endpoint_id: Option<String>,
    pub short_error: Option<String>,
    pub log_bytes_retained: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteWorkerStatus {
    pub endpoint_id: String,
    pub systems: Vec<String>,
    pub feature_labels: Vec<String>,
    pub sandbox_modes: Vec<String>,
    pub network_modes: Vec<String>,
    pub logical_store_prefixes: Vec<String>,
    pub concurrency: u32,
    pub output_signing_key_count: u32,
    pub resumable_job_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteFailureStatus {
    pub job_id: String,
    pub phase: RemoteCoordinatorJobPhase,
    pub lost_phase: Option<RemoteFailurePhase>,
    pub retry_class: RemoteRetryClass,
    pub short_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteLogCursorStatus {
    pub job_id: String,
    pub start_cursor: u64,
    pub next_cursor: u64,
    pub retained_bytes: u64,
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
    pub tickets: Vec<RemoteTicketView>,
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
    pub upload_summary: RemoteInputUploadPrivacySummary,
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
    let accepted_capabilities = hello
        .capabilities
        .iter()
        .filter(|capability| supported_capabilities.contains(capability))
        .cloned()
        .collect::<Vec<_>>();
    ProtocolDecision::Proceed(AcceptedHello {
        endpoint_id: hello.endpoint_id.clone(),
        accepted_capabilities,
    })
}

pub fn authorize_ticket(ticket: &RemoteTicket, request: &TicketAuthRequest) -> TicketDecision {
    if ticket.revoked {
        return TicketDecision::Reject("ticket-revoked".to_string());
    }
    if request.now_unix_s >= ticket.expires_unix_s {
        return TicketDecision::Reject("ticket-expired".to_string());
    }
    if ticket.uses_remaining == 0 {
        return TicketDecision::Reject("ticket-exhausted".to_string());
    }
    if request.secret != ticket.secret {
        return TicketDecision::Reject("ticket-secret-mismatch".to_string());
    }
    if let Some(bound) = &ticket.bound_client_endpoint
        && request.client_endpoint.as_deref() != Some(bound.as_str())
    {
        return TicketDecision::Reject("ticket-client-endpoint-mismatch".to_string());
    }
    TicketDecision::Authorized
}

pub fn validate_concrete_request(request: &ConcreteBuildRequest, ticket: &RemoteTicket) -> Result<(), String> {
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
    if request.upload_bytes > ticket.max_upload_bytes || request.upload_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err("upload-byte-limit-exceeded".to_string());
    }
    if request.build_time_limit_secs > ticket.max_build_time_secs
        || request.build_time_limit_secs > MAX_REMOTE_BUILD_TIME_SECS
    {
        return Err("build-time-limit-exceeded".to_string());
    }
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
            plan_remote_action_payload(action_id, spec_json, request)
        }
        RemoteConcreteBuildPayload::Derivation { drv_path, drv_json } => {
            plan_remote_derivation_payload(drv_path, drv_json, request)
        }
    }
}

fn plan_remote_action_payload(
    action_id: &str,
    spec_json: &str,
    request: &ConcreteBuildRequest,
) -> Result<RemoteExecutablePlan, String> {
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
    drv_path: &str,
    drv_json: &str,
    request: &ConcreteBuildRequest,
) -> Result<RemoteExecutablePlan, String> {
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
        drv_path,
        &computed_drv_path,
        &nix_drv,
        &request.expected_outputs,
        &request.store_prefix,
    )?;
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
        if let Some(logical_path) = &output.logical_path {
            if logical_path.is_empty() || !logical_path.starts_with(store_prefix) {
                return Err("remote-expected-output-store-prefix-mismatch".to_string());
            }
        }
    }
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
    Ok(())
}

fn validate_derivation_plan_identity(
    declared_drv_path: &str,
    computed_drv_path: &str,
    derivation: &nix_compat::derivation::Derivation,
    expected_outputs: &[RemoteExpectedOutput],
    store_prefix: &str,
) -> Result<(), String> {
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
    let mut env = BTreeMap::new();
    for (name, value) in &derivation.environment {
        let value = String::from_utf8(value.to_vec()).map_err(|_| "remote-derivation-env-non-utf8".to_string())?;
        env.insert(name.clone(), value);
    }
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
    let plan_digest_blake3 = remote_executable_plan_digest(
        &request.request_id,
        &request.store_prefix,
        &source,
        &command_args,
        &command_env,
        &system,
        &request.expected_outputs,
    );
    Ok(RemoteExecutablePlan {
        request_id: request.request_id.clone(),
        store_prefix: request.store_prefix.clone(),
        source,
        command_args,
        command_env,
        system,
        expected_outputs: request.expected_outputs.clone(),
        plan_digest_blake3,
    })
}

fn remote_executable_plan_digest(
    request_id: &str,
    store_prefix: &str,
    source: &RemoteExecutablePlanSource,
    command_args: &[String],
    command_env: &BTreeMap<String, String>,
    system: &str,
    expected_outputs: &[RemoteExpectedOutput],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "request-id", request_id);
    hash_labeled_str(&mut hasher, "store-prefix", store_prefix);
    hash_executable_plan_source(&mut hasher, source);
    hash_labeled_str(&mut hasher, "system", system);
    for arg in command_args {
        hash_labeled_str(&mut hasher, "command-arg", arg);
    }
    for (name, value) in command_env {
        hash_labeled_str(&mut hasher, "env-name", name);
        hash_labeled_str(&mut hasher, "env-value", value);
    }
    for output in expected_outputs {
        hash_labeled_str(&mut hasher, "output-name", &output.name);
        match &output.logical_path {
            Some(logical_path) => hash_labeled_str(&mut hasher, "output-path", logical_path),
            None => hash_labeled_str(&mut hasher, "output-path-state", "content-addressed-unknown"),
        }
    }
    hasher.finalize().to_hex().to_string()
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

fn hash_labeled_str(hasher: &mut blake3::Hasher, label: &str, value: &str) {
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

fn execute_remote_local_build(
    executor: &RemoteLocalBuildExecutor,
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
    input_upload: &RemoteInputUpload,
) -> Result<RemoteExecutionOutcome, String> {
    validate_local_executor_input_refs(request, &input_upload.refs)?;
    if executor.store_prefix != request.store_prefix {
        return Err("remote-local-executor-store-prefix-mismatch".to_string());
    }
    validate_remote_execution_plan(plan)?;

    #[cfg(target_os = "linux")]
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| format!("remote-local-executor-runtime: {err}"))?;
        return rt.block_on(execute_remote_local_build_linux(executor, request, plan, input_upload));
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
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
    input_upload: &RemoteInputUpload,
) -> Result<RemoteExecutionOutcome, String> {
    use snix_build::buildservice::BubblewrapBuildService;

    let _mutation_guard = crunch_store::StoreMutationGuard::acquire_wait(&executor.state_dir)
        .map_err(|err| format!("remote-local-executor-mutation-lock: {err}"))?;
    let (drv_path, mut known_paths) = local_derivation_registry(request, plan)?;
    let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: executor.state_dir.clone(),
        output_dir: executor.output_dir.clone(),
        remote_cache_urls: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: executor.store_prefix.clone(),
    })
    .await
    .map_err(|err| format!("remote-local-executor-open-store: {err}"))?;
    materialize_remote_input_upload(&store, request, input_upload).await?;

    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    let pathinfo_service = store.pathinfo_service();
    let workdir = std::env::temp_dir().join(REMOTE_LOCAL_BUILD_WORKDIR_NAME);
    std::fs::create_dir_all(&workdir).map_err(|err| format!("remote-local-executor-workdir: {err}"))?;
    let bwrap_service = BubblewrapBuildService::new(workdir, blob_service.clone(), directory_service.clone());
    let fetch_service = crunch_build::FetchBuildService::new(blob_service.clone(), directory_service.clone());
    let build_service = crunch_build::DispatchBuildService::new(fetch_service, bwrap_service);
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
    let outcome = builder
        .build(&drv_path, &mut known_paths)
        .await
        .map_err(|err| format!("remote-local-executor-build: {err}"))?;
    remote_execution_outcome_from_build_outcome_with_payloads(request, plan, &outcome, &store).await
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

async fn remote_execution_outcome_from_build_outcome_with_payloads(
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
    outcome: &crunch_build::BuildOutcome,
    store: &crunch_store::StoreHandle,
) -> Result<RemoteExecutionOutcome, String> {
    let mut outputs = Vec::with_capacity(plan.expected_outputs.len());
    for expected in &plan.expected_outputs {
        let path_info = outcome
            .outputs
            .get(&expected.name)
            .ok_or_else(|| "remote-local-executor-output-missing".to_string())?;
        let nar_payload = render_remote_output_nar_payload(store, path_info).await?;
        outputs.push(remote_execution_output_from_pathinfo(
            &request.store_prefix,
            expected,
            path_info,
            Some(nar_payload),
        )?);
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
    if let Some(expected_logical_path) = &expected.logical_path {
        if &logical_path != expected_logical_path {
            return Err("remote-local-executor-output-path-mismatch".to_string());
        }
    }
    let content_digest_blake3 = remote_pathinfo_node_digest_blake3(path_info)?;
    let artifact_attestation_digest_blake3 =
        crunch_store::artifact_attestation_digest_for_pathinfo(store_prefix, path_info, &expected.name, None)
            .map_err(|err| format!("remote-local-executor-artifact-digest: {err}"))?
            .to_hex();
    Ok(RemoteExecutionOutput {
        name: expected.name.clone(),
        logical_path,
        content_digest_blake3,
        size_bytes: path_info.nar_size,
        artifact_attestation_digest_blake3,
        path_info: Some(path_info.clone()),
        nar_payload,
    })
}

fn remote_pathinfo_node_digest_blake3(path_info: &PathInfo) -> Result<String, String> {
    let node_bytes =
        serde_json::to_vec(&path_info.node).map_err(|err| format!("remote-local-executor-node-digest-json: {err}"))?;
    Ok(blake3::hash(&node_bytes).to_hex().to_string())
}

async fn render_remote_output_nar_payload(
    store: &crunch_store::StoreHandle,
    path_info: &PathInfo,
) -> Result<Vec<u8>, String> {
    let max_payload_bytes = u64::try_from(MAX_REMOTE_INLINE_NAR_PAYLOAD_BYTES)
        .map_err(|_| "remote-output-nar-payload-limit-overflow".to_string())?;
    if path_info.nar_size > max_payload_bytes {
        return Err("remote-output-nar-payload-too-large".to_string());
    }
    let mut writer = BoundedAsyncVecWriter::new(MAX_REMOTE_INLINE_NAR_PAYLOAD_BYTES);
    store
        .render_nar(&path_info.node, &mut writer)
        .await
        .map_err(|err| format!("remote-output-nar-render-failed: {err}"))?;
    let payload = writer.into_inner();
    let payload_size = remote_payload_size_bytes(payload.len())?;
    if payload_size != path_info.nar_size {
        return Err("remote-output-nar-size-mismatch".to_string());
    }
    Ok(payload)
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

pub fn remote_build_observability_report(
    selected_route: &str,
    rejected_route_reasons: &[String],
    endpoint_id: Option<&str>,
    upload_summary: RemoteInputUploadPrivacySummary,
    admission: &RemoteOutputAdmissionReport,
    import_report: &RemoteOutputImportReport,
    non_claims: &[String],
) -> Result<RemoteBuildObservabilityReport, String> {
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
    Ok(RemoteBuildObservabilityReport {
        selected_route: selected_route.to_string(),
        rejected_route_reasons: rejected_route_reasons.iter().map(|reason| bounded_untrusted_text(reason)).collect(),
        endpoint_id: endpoint_id.map(bounded_untrusted_text),
        upload_summary,
        transfer: import_report.transfer.clone(),
        trust_basis: admission.trust_basis.clone(),
        outputs,
        non_claims: non_claims.iter().map(|claim| bounded_untrusted_text(claim)).collect(),
    })
}

pub fn remote_operator_e2e_rail_report(
    route_plan: &crate::realization_routing::RoutePlanReport,
    transcript: &RemoteStdioTranscript,
    upload_summary: RemoteInputUploadPrivacySummary,
    admission: &RemoteOutputAdmissionReport,
    import_report: &RemoteOutputImportReport,
    status: RemoteCoordinatorStatusSnapshot,
    non_claims: &[String],
) -> Result<RemoteOperatorE2eRailReport, String> {
    if route_plan.selected_route != crate::realization_routing::RouteClass::P2pRemoteBuilder {
        return Err("remote-e2e-route-not-remote-builder".to_string());
    }
    validate_remote_operator_e2e_frames(&transcript.frames)?;
    validate_remote_operator_e2e_status(&status)?;
    let selected_route = route_plan.selected_route.as_str().to_string();
    let rejected_route_reasons =
        route_plan.rejected_routes.iter().map(|rejection| rejection.reason_code.clone()).collect::<Vec<_>>();
    let rail_non_claims = remote_operator_e2e_non_claims(route_plan.non_claim, non_claims);
    let build_report = remote_build_observability_report(
        &selected_route,
        &rejected_route_reasons,
        Some(&status.endpoint_id),
        upload_summary.clone(),
        admission,
        import_report,
        &rail_non_claims,
    )?;
    let artifact_attestation_paths = import_report
        .outputs
        .iter()
        .map(|output| output.artifact_attestation_path.clone())
        .collect::<Vec<_>>();
    if artifact_attestation_paths.is_empty() {
        return Err("remote-e2e-artifact-attestation-empty".to_string());
    }
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
        build_report,
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
    if status.tickets.iter().any(|ticket| ticket.secret != SECRET_REDACTION) {
        return Err("remote-e2e-status-ticket-secret-unredacted".to_string());
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
    logical_store_path: &str,
    declared_content_blake3: Option<&str>,
) -> Result<RemoteSourceUploadReadiness, String> {
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
    input_ref: &str,
) -> Result<PathBuf, String> {
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
    let materialized = crate::source_bundle::materialize_imported_source_record_for_store_path(
        source_state_dir,
        &request.store_prefix,
        input_ref,
        scratch.path(),
    )
    .map_err(|err| format!("remote-input-source-state-materialize-failed: {err}"))?;
    if !materialized {
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
    let host_path = store_path.to_absolute_path_with_prefix(store.output_dir_str());
    if !Path::new(&host_path).exists() {
        crunch_store::export_castore_to_disk(&node, &host_path, &store.blob_service(), &store.directory_service())
            .await
            .map_err(|err| format!("remote-input-export-failed: {err}"))?;
    }
    Ok(())
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
        hash_remote_output_digest_fields(
            &mut hasher,
            &output.name,
            &output.logical_path,
            &output.content_digest_blake3,
            &output.artifact_attestation_digest_blake3,
            output.size_bytes,
            nar_payload_digest_blake3.as_deref(),
            nar_payload_size_bytes,
        );
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_remote_output_digest_fields(
    hasher: &mut blake3::Hasher,
    name: &str,
    logical_path: &str,
    content_digest_blake3: &str,
    artifact_attestation_digest_blake3: &str,
    size_bytes: u64,
    nar_payload_digest_blake3: Option<&str>,
    nar_payload_size_bytes: Option<u64>,
) {
    hash_labeled_str(hasher, "output-name", name);
    hash_labeled_str(hasher, "output-path", logical_path);
    hash_labeled_str(hasher, "content-digest", content_digest_blake3);
    hash_labeled_str(hasher, "artifact-digest", artifact_attestation_digest_blake3);
    hash_labeled_str(hasher, "size-bytes", &size_bytes.to_string());
    hash_optional_digest_field(hasher, "nar-payload-digest", nar_payload_digest_blake3);
    hash_optional_u64_field(hasher, "nar-payload-size", nar_payload_size_bytes);
}

fn hash_optional_digest_field(hasher: &mut blake3::Hasher, label: &str, value: Option<&str>) {
    match value {
        Some(value) => hash_labeled_str(hasher, label, value),
        None => hash_labeled_str(hasher, label, "<absent>"),
    }
}

fn hash_optional_u64_field(hasher: &mut blake3::Hasher, label: &str, value: Option<u64>) {
    match value {
        Some(value) => hash_labeled_str(hasher, label, &value.to_string()),
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
    Ok(RemoteOutputTransferArtifact {
        request_id: request_id.to_string(),
        output_name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        artifact_kind: RemoteOutputTransferArtifactKind::Nar,
        digest_blake3,
        size_bytes,
        payload,
    })
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
        hash_remote_output_digest_fields(
            &mut hasher,
            &output.name,
            &output.logical_path,
            &output.content_digest_blake3,
            &output.artifact_attestation_digest_blake3,
            output.size_bytes,
            output.nar_payload_digest_blake3.as_deref(),
            output.nar_payload_size_bytes,
        );
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
    store_prefix: &str,
) -> Result<(), String> {
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
    store_prefix: &str,
) -> Result<(), String> {
    let store_path = parse_remote_output_store_path(&output.logical_path, store_prefix)?;
    validate_remote_pathinfo_binding(
        &output.name,
        &output.artifact_attestation_digest_blake3,
        path_info,
        &store_path,
        store_prefix,
        signing_key_id,
        RemotePathInfoErrorLabels {
            store_path_mismatch: "remote-execution-output-pathinfo-store-path-mismatch",
            unsigned: "remote-execution-output-pathinfo-unsigned",
            signing_key_mismatch: "remote-execution-output-pathinfo-signing-key-mismatch",
            artifact_digest_failed: "remote-execution-output-artifact-attestation-digest-failed",
            artifact_digest_mismatch: "remote-execution-output-artifact-attestation-digest-mismatch",
        },
    )
}

fn expected_output_path_map(expected_outputs: &[RemoteExpectedOutput]) -> BTreeMap<&str, Option<&str>> {
    expected_outputs
        .iter()
        .map(|output| (output.name.as_str(), output.logical_path.as_deref()))
        .collect()
}

fn validate_output_path_against_expected(
    output_name: &str,
    logical_path: &str,
    expected: &BTreeMap<&str, Option<&str>>,
    store_prefix: &str,
    mismatch_label: &str,
) -> Result<(), String> {
    match expected.get(output_name) {
        Some(Some(expected_logical_path)) => {
            if *expected_logical_path != logical_path {
                return Err(mismatch_label.to_string());
            }
        }
        Some(None) => {
            if logical_path.is_empty() || !logical_path.starts_with(store_prefix) {
                return Err(mismatch_label.to_string());
            }
        }
        None => return Err(mismatch_label.to_string()),
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
    let mut same_name_material_required = false;
    let mut same_name_different_material = false;
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
            (Some(_), Some(_)) => same_name_different_material = true,
            (None, Some(_)) => same_name_material_required = true,
            (_, None) => {
                return OutputTrustDecision::Accept {
                    key_id: signing_key_id.to_string(),
                    trust_basis: signer.trust_basis(signing_key_id),
                };
            }
        }
    }
    if same_name_material_required {
        return OutputTrustDecision::Reject("output-key-material-missing".to_string());
    }
    if same_name_different_material {
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
    if let Some((name, material)) = value.split_once(':') {
        if !name.is_empty() && !material.is_empty() {
            return OutputKeyRef {
                name: name.to_string(),
                key_material_digest_blake3: Some(blake3::hash(material.as_bytes()).to_hex().to_string()),
            };
        }
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
    if registration.resumable_jobs.len() > MAX_REMOTE_STATUS_ITEMS {
        return Err(format!("remote-worker-resume-summary-count-exceeds-{MAX_REMOTE_STATUS_ITEMS}"));
    }
    for summary in &registration.resumable_jobs {
        validate_worker_resume_summary(summary)?;
    }
    Ok(())
}

fn validate_worker_resume_summary(summary: &RemoteWorkerResumeSummary) -> Result<(), String> {
    if summary.job_id.is_empty() || summary.normalized_build_key.is_empty() {
        return Err("remote-worker-resume-identity-empty".to_string());
    }
    if !is_blake3_hex_digest(&summary.normalized_build_key) {
        return Err("remote-worker-resume-key-invalid".to_string());
    }
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
    if request.request.upload_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err("remote-coordinator-upload-byte-limit-exceeded".to_string());
    }
    if request.request.build_time_limit_secs == 0 || request.request.build_time_limit_secs > MAX_REMOTE_BUILD_TIME_SECS
    {
        return Err("remote-coordinator-build-time-limit-exceeded".to_string());
    }
    plan_remote_executable_request(&request.request)
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
    hash_labeled_str(&mut hasher, "required-system", &request.required_system);
    hash_ordered_values(&mut hasher, "required-feature", &request.required_features);
    hash_labeled_str(&mut hasher, "required-sandbox", &request.required_sandbox_mode);
    hash_labeled_str(&mut hasher, "required-network", &request.required_network_mode);
    Ok(hasher.finalize().to_hex().to_string())
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
) -> Result<Vec<String>, String> {
    validate_worker_registration(&registration)?;
    let adopted = adopt_worker_resume_summaries(state, &registration)?;
    state.workers.insert(registration.endpoint_id.clone(), registration);
    Ok(adopted)
}

fn adopt_worker_resume_summaries(
    state: &mut RemoteCoordinatorState,
    registration: &RemoteWorkerRegistration,
) -> Result<Vec<String>, String> {
    let mut adopted = Vec::with_capacity(registration.resumable_jobs.len());
    for summary in &registration.resumable_jobs {
        adopt_worker_resume_summary(state, registration, summary)?;
        adopted.push(summary.job_id.clone());
    }
    Ok(adopted)
}

fn adopt_worker_resume_summary(
    state: &mut RemoteCoordinatorState,
    registration: &RemoteWorkerRegistration,
    summary: &RemoteWorkerResumeSummary,
) -> Result<(), String> {
    if let Some(existing) = state.jobs.get_mut(&summary.job_id) {
        if existing.normalized_build_key != summary.normalized_build_key {
            return Err("remote-worker-resume-key-conflict".to_string());
        }
        existing.assigned_worker_endpoint_id = Some(registration.endpoint_id.clone());
        existing.phase = summary.phase;
        existing.result_available = summary.result_available;
        existing.log_next_cursor = summary.log_next_cursor;
        existing.lost_phase = None;
        return Ok(());
    }
    state.jobs.insert(summary.job_id.clone(), resumed_job_summary(registration, summary));
    Ok(())
}

fn resumed_job_summary(
    registration: &RemoteWorkerRegistration,
    summary: &RemoteWorkerResumeSummary,
) -> RemoteCoordinatorJobSummary {
    RemoteCoordinatorJobSummary {
        job_id: summary.job_id.clone(),
        normalized_build_key: summary.normalized_build_key.clone(),
        assigned_worker_endpoint_id: Some(registration.endpoint_id.clone()),
        phase: summary.phase,
        live_output_claims: Vec::new(),
        result_available: summary.result_available,
        lost_phase: None,
        log_start_cursor: 0,
        log_next_cursor: summary.log_next_cursor,
        short_error: None,
    }
}

pub fn plan_coordinator_dispatch(
    state: &RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    let normalized_key = normalized_remote_build_key(request)?;
    if let Some(reason) = conflicting_live_output_claim(state, request, &normalized_key) {
        return Ok(RemoteCoordinatorDispatchDecision::Reject { reason });
    }
    if let Some(decision) = existing_job_decision(state, &normalized_key) {
        return Ok(decision);
    }
    match select_coordinator_worker(state, request) {
        Some(worker) => Ok(RemoteCoordinatorDispatchDecision::Dispatch {
            worker_endpoint_id: worker.endpoint_id.clone(),
            job_id: coordinator_job_id(&normalized_key),
            normalized_build_key: normalized_key,
        }),
        None => {
            let reason = no_matching_worker_reason(state, request);
            if request.wait_for_worker {
                return Ok(RemoteCoordinatorDispatchDecision::Pending { reason });
            }
            Ok(RemoteCoordinatorDispatchDecision::Reject { reason })
        }
    }
}

pub fn admit_coordinator_dispatch(
    state: &mut RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
) -> Result<RemoteCoordinatorDispatchDecision, String> {
    let decision = plan_coordinator_dispatch(state, request)?;
    if let RemoteCoordinatorDispatchDecision::Dispatch {
        worker_endpoint_id,
        job_id,
        normalized_build_key,
    } = &decision
    {
        let summary = queued_job_summary(job_id, normalized_build_key, worker_endpoint_id, request);
        for claim in &summary.live_output_claims {
            state.live_output_claims.insert(claim.clone(), normalized_build_key.clone());
        }
        state.jobs.insert(job_id.clone(), summary);
    }
    // Auto-save after mutation for durability.
    if let Some(ref sd) = state.state_dir {
        let _ = save_coordinator_state(sd, state)
            .map_err(|e| tracing::warn!("coordinator state save failed: {e}"));
    }
    Ok(decision)
}

fn queued_job_summary(
    job_id: &str,
    normalized_key: &str,
    worker_endpoint_id: &str,
    request: &RemoteCoordinatorBuildRequest,
) -> RemoteCoordinatorJobSummary {
    RemoteCoordinatorJobSummary {
        job_id: job_id.to_string(),
        normalized_build_key: normalized_key.to_string(),
        assigned_worker_endpoint_id: Some(worker_endpoint_id.to_string()),
        phase: RemoteCoordinatorJobPhase::Queued,
        live_output_claims: request.live_output_claims.clone(),
        result_available: false,
        lost_phase: None,
        log_start_cursor: 0,
        log_next_cursor: 0,
        short_error: None,
    }
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
            reason: format!(
                "restart-state-unavailable-phase-{}",
                job.lost_phase.map(RemoteFailurePhase::as_status_label).unwrap_or("unknown")
            ),
        },
        _ => RemoteCoordinatorDispatchDecision::AttachExisting {
            job_id: job.job_id.clone(),
            normalized_build_key: job.normalized_build_key.clone(),
        },
    }
}

fn select_coordinator_worker<'a>(
    state: &'a RemoteCoordinatorState,
    request: &RemoteCoordinatorBuildRequest,
) -> Option<&'a RemoteWorkerRegistration> {
    let mut eligible = Vec::new();
    for worker in state.workers.values() {
        if worker_satisfies_request(worker, request).is_ok()
            && active_jobs_for_worker(state, &worker.endpoint_id) < worker.concurrency
        {
            eligible.push(worker);
        }
    }
    eligible.into_iter().next()
}

fn no_matching_worker_reason(state: &RemoteCoordinatorState, request: &RemoteCoordinatorBuildRequest) -> String {
    if state.workers.is_empty() {
        return "no-workers-registered".to_string();
    }
    for worker in state.workers.values() {
        if active_jobs_for_worker(state, &worker.endpoint_id) >= worker.concurrency {
            return "worker-concurrency-limit".to_string();
        }
        if let Err(reason) = worker_satisfies_request(worker, request) {
            return reason;
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

fn coordinator_job_id(normalized_key: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_COORDINATOR_JOB_ID_LABEL);
    hash_labeled_str(&mut hasher, "normalized-key", normalized_key);
    hasher.finalize().to_hex().to_string()
}

pub fn retain_remote_log_chunks(
    existing: &[RemoteCoordinatorLogChunk],
    next_chunk: RemoteCoordinatorLogChunk,
    from_cursor: u64,
    policy: RemoteLogRetentionPolicy,
) -> Result<RemoteLogReplayPlan, String> {
    validate_log_policy(policy)?;
    validate_next_log_chunk(existing, &next_chunk)?;
    let mut retained = existing.to_vec();
    retained.push(next_chunk);
    let dropped_bytes = trim_remote_log_chunks(&mut retained, policy)?;
    let replay_chunks = retained.iter().filter(|chunk| chunk.cursor >= from_cursor).cloned().collect::<Vec<_>>();
    let next_cursor = retained.last().map(|chunk| chunk.cursor.saturating_add(1)).unwrap_or(0);
    Ok(RemoteLogReplayPlan {
        replay_chunks,
        retained_chunks: retained,
        truncated: dropped_bytes > 0,
        dropped_bytes,
        next_cursor,
    })
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

fn validate_next_log_chunk(
    existing: &[RemoteCoordinatorLogChunk],
    next: &RemoteCoordinatorLogChunk,
) -> Result<(), String> {
    if let Some(last) = existing.last()
        && next.cursor <= last.cursor
    {
        return Err("remote-log-cursor-not-monotonic".to_string());
    }
    if remote_log_chunk_bytes(next)? > MAX_REMOTE_LOG_BYTES {
        return Err("remote-log-chunk-too-large".to_string());
    }
    Ok(())
}

fn trim_remote_log_chunks(
    retained: &mut Vec<RemoteCoordinatorLogChunk>,
    policy: RemoteLogRetentionPolicy,
) -> Result<u64, String> {
    let mut dropped_bytes = 0_u64;
    while retained.len() > policy.max_chunks || remote_log_bytes(retained)? > policy.max_bytes {
        let dropped = retained.remove(0);
        dropped_bytes = dropped_bytes
            .checked_add(remote_log_chunk_bytes(&dropped)?)
            .ok_or_else(|| "remote-log-dropped-bytes-overflow".to_string())?;
    }
    Ok(dropped_bytes)
}

fn remote_log_bytes(chunks: &[RemoteCoordinatorLogChunk]) -> Result<u64, String> {
    let mut total = 0_u64;
    for chunk in chunks {
        total = total
            .checked_add(remote_log_chunk_bytes(chunk)?)
            .ok_or_else(|| "remote-log-bytes-overflow".to_string())?;
    }
    Ok(total)
}

fn remote_log_chunk_bytes(chunk: &RemoteCoordinatorLogChunk) -> Result<u64, String> {
    u64::try_from(chunk.bytes.len()).map_err(|_| "remote-log-chunk-size-overflow".to_string())
}

pub fn coordinator_status_snapshot(
    endpoint_id: &str,
    configured_concurrency: u32,
    state: &RemoteCoordinatorState,
    tickets: &[RemoteTicket],
) -> Result<RemoteCoordinatorStatusSnapshot, String> {
    if endpoint_id.is_empty() {
        return Err("remote-status-endpoint-empty".to_string());
    }
    if configured_concurrency == 0 || configured_concurrency > MAX_REMOTE_WORKER_CONCURRENCY {
        return Err("remote-status-concurrency-invalid".to_string());
    }
    if state.jobs.len() > MAX_REMOTE_STATUS_ITEMS || tickets.len() > MAX_REMOTE_STATUS_ITEMS {
        return Err(format!("remote-status-item-count-exceeds-{MAX_REMOTE_STATUS_ITEMS}"));
    }
    let mut queued_jobs = Vec::new();
    let mut active_jobs = Vec::new();
    let mut recent_jobs = Vec::new();
    let mut recent_failures = Vec::new();
    let mut log_cursors = Vec::new();
    for job in state.jobs.values() {
        push_status_job(&mut queued_jobs, &mut active_jobs, &mut recent_jobs, state, job)?;
        push_status_failure(&mut recent_failures, job);
        log_cursors.push(coordinator_log_cursor_status(state, job)?);
    }
    Ok(RemoteCoordinatorStatusSnapshot {
        endpoint_id: endpoint_id.to_string(),
        configured_concurrency,
        worker_count: bounded_runtime_count_u32(state.workers.len())?,
        workers: state.workers.values().map(remote_worker_status).collect::<Result<Vec<_>, _>>()?,
        queued_jobs,
        active_jobs,
        recent_jobs,
        recent_failures,
        log_cursors,
        tickets: tickets.iter().map(redacted_ticket_view).collect(),
    })
}

fn remote_worker_status(worker: &RemoteWorkerRegistration) -> Result<RemoteWorkerStatus, String> {
    Ok(RemoteWorkerStatus {
        endpoint_id: worker.endpoint_id.clone(),
        systems: bounded_string_list(&worker.systems),
        feature_labels: bounded_string_list(&worker.feature_labels),
        sandbox_modes: bounded_string_list(&worker.sandbox_modes),
        network_modes: bounded_string_list(&worker.network_modes),
        logical_store_prefixes: bounded_string_list(&worker.logical_store_prefixes),
        concurrency: worker.concurrency,
        output_signing_key_count: bounded_runtime_count_u32(worker.output_signing_key_ids.len())?,
        resumable_job_count: bounded_runtime_count_u32(worker.resumable_jobs.len())?,
    })
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
    });
}

fn coordinator_log_cursor_status(
    state: &RemoteCoordinatorState,
    job: &RemoteCoordinatorJobSummary,
) -> Result<RemoteLogCursorStatus, String> {
    let retained = state.logs.get(&job.job_id).map(Vec::as_slice).unwrap_or(&[]);
    Ok(RemoteLogCursorStatus {
        job_id: job.job_id.clone(),
        start_cursor: job.log_start_cursor,
        next_cursor: job.log_next_cursor,
        retained_bytes: remote_log_bytes(retained)?,
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
    state: &RemoteCoordinatorState,
    job: &RemoteCoordinatorJobSummary,
) -> Result<RemoteCoordinatorJobStatus, String> {
    let retained = state.logs.get(&job.job_id).map(Vec::as_slice).unwrap_or(&[]);
    Ok(RemoteCoordinatorJobStatus {
        job_id: job.job_id.clone(),
        phase: job.phase,
        worker_endpoint_id: job.assigned_worker_endpoint_id.clone(),
        short_error: job.short_error.as_deref().map(bounded_untrusted_text),
        log_bytes_retained: remote_log_bytes(retained)?,
    })
}

fn bounded_runtime_count_u32(count: usize) -> Result<u32, String> {
    u32::try_from(count).map_err(|_| "remote-status-count-overflow".to_string())
}

pub fn decide_remote_reconnect(
    active_session_id: &str,
    candidate_session_id: &str,
    attempts: u32,
    max_attempts: u32,
) -> RemoteReconnectDecision {
    if active_session_id == candidate_session_id {
        return RemoteReconnectDecision::SameSession;
    }
    if attempts >= max_attempts {
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

pub fn encode_remote_frame(frame: &RemoteFrame) -> Result<Vec<u8>, String> {
    let payload = serde_json::to_vec(frame).map_err(|err| format!("serializing remote frame: {err}"))?;
    if payload.len() > MAX_REMOTE_FRAME_BYTES {
        return Err(format!("remote-frame-payload-exceeds-{MAX_REMOTE_FRAME_BYTES}"));
    }
    let payload_len = u32::try_from(payload.len()).map_err(|_| "remote-frame-length-overflow".to_string())?;
    let mut encoded = Vec::with_capacity(REMOTE_FRAME_HEADER_BYTES.saturating_add(payload.len()));
    encoded.extend_from_slice(&payload_len.to_be_bytes());
    encoded.extend_from_slice(&payload);
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
    let encoded = encode_remote_frame(frame)?;
    writer.write_all(&encoded).map_err(|err| format!("writing remote frame: {err}"))
}

pub fn read_remote_frame(mut reader: impl Read) -> Result<RemoteFrame, String> {
    let mut header = [0_u8; REMOTE_FRAME_HEADER_BYTES];
    reader.read_exact(&mut header).map_err(|err| format!("reading remote frame header: {err}"))?;
    let payload_len = frame_payload_len(&header)?;
    if payload_len > MAX_REMOTE_FRAME_BYTES {
        return Err(format!("remote-frame-payload-exceeds-{MAX_REMOTE_FRAME_BYTES}"));
    }
    let mut payload = vec![0_u8; payload_len];
    reader.read_exact(&mut payload).map_err(|err| format!("reading remote frame payload: {err}"))?;
    let mut encoded = Vec::with_capacity(REMOTE_FRAME_HEADER_BYTES.saturating_add(payload_len));
    encoded.extend_from_slice(&header);
    encoded.extend_from_slice(&payload);
    decode_remote_frame(&encoded)
}

pub fn decode_remote_frame_stream(encoded: &[u8]) -> Result<Vec<RemoteFrame>, String> {
    let mut frames = Vec::new();
    let mut offset = 0_usize;
    while offset < encoded.len() {
        if frames.len() >= MAX_REMOTE_STDIO_FRAME_COUNT {
            return Err(format!("remote-stdio-frame-count-exceeds-{MAX_REMOTE_STDIO_FRAME_COUNT}"));
        }
        let remaining = encoded.len().saturating_sub(offset);
        if remaining < REMOTE_FRAME_HEADER_BYTES {
            return Err("remote-frame-header-incomplete-or-unframed-stdout".to_string());
        }
        let payload_len = frame_payload_len(&encoded[offset..])?;
        if payload_len > MAX_REMOTE_FRAME_BYTES {
            return Err(format!("remote-frame-payload-exceeds-{MAX_REMOTE_FRAME_BYTES}"));
        }
        let frame_len = REMOTE_FRAME_HEADER_BYTES
            .checked_add(payload_len)
            .ok_or_else(|| "remote-frame-length-overflow".to_string())?;
        let end = offset.checked_add(frame_len).ok_or_else(|| "remote-frame-stream-offset-overflow".to_string())?;
        if end > encoded.len() {
            return Err("remote-frame-stream-truncated-or-unframed-stdout".to_string());
        }
        let frame = decode_remote_frame(&encoded[offset..end])?;
        frames.push(frame);
        offset = end;
    }
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
    Ok(RemoteStdioTranscript {
        binding,
        frames,
        stderr_summary: bounded_stderr_summary(&output.stderr),
    })
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

pub fn run_stdio_remote_child(command: &RemoteStdioCommand) -> Result<RemoteStdioTranscript, RunError> {
    if command.timeout_secs == 0 || command.timeout_secs > MAX_REMOTE_BUILD_TIME_SECS {
        return Err(RunError::Internal(format!(
            "stdio remote child timeout out of bounds: {} seconds",
            command.timeout_secs
        )));
    }
    let mut child = Command::new(&command.program)
        .args(&command.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            RunError::Internal(format!("spawning stdio remote child {}: {err}", command.program.display()))
        })?;
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
    validate_stdio_child_output_for_binding(&child_output, command.binding).map_err(|failure| {
        RunError::Internal(format!(
            "stdio remote child failed phase={:?} retry={:?}: {}; stderr={}",
            failure.phase,
            failure.retry_class,
            failure.reason,
            bounded_stderr_summary(&child_output.stderr)
        ))
    })
}

fn wait_for_stdio_child_output(
    mut child: std::process::Child,
    timeout_secs: u64,
    program: &Path,
) -> Result<std::process::Output, RunError> {
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(timeout_secs))
        .ok_or_else(|| RunError::Internal("stdio remote child timeout overflow".to_string()))?;
    loop {
        if child
            .try_wait()
            .map_err(|err| RunError::Internal(format!("polling stdio remote child {}: {err}", program.display())))?
            .is_some()
        {
            return child.wait_with_output().map_err(|err| {
                RunError::Internal(format!("collecting stdio remote child {} output: {err}", program.display()))
            });
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RunError::Internal(format!(
                "stdio remote child timed out after {timeout_secs} seconds phase={:?}",
                RemoteFailurePhase::TransportSetup
            )));
        }
        thread::sleep(Duration::from_millis(REMOTE_CHILD_POLL_INTERVAL_MS));
    }
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
            },
        },
    ]
}

pub fn parse_remote_ticket_credential(token: &str) -> Result<RemoteTicketCredential, String> {
    let (ticket_id, secret) = token.split_once(':').ok_or_else(|| "remote-ticket-token-missing-colon".to_string())?;
    if ticket_id.is_empty() {
        return Err("remote-ticket-id-empty".to_string());
    }
    if secret.is_empty() {
        return Err("remote-ticket-secret-empty".to_string());
    }
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
    Ok(ConcreteBuildRequest {
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
    })
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
    RemoteLoopbackClient {
        session_id,
        hello: RemoteHello {
            alpn: REMOTE_PROTOCOL_ALPN.to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: options.builder.endpoint_id.clone(),
            capabilities: options.transfer_capabilities.as_capability_labels(),
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
    }
}

fn remote_client_request_id(label: &str, drv_path: &StorePath<String>, store_prefix: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_labeled_str(&mut hasher, "kind", REMOTE_CLIENT_REQUEST_ID_LABEL);
    hash_labeled_str(&mut hasher, "label", label);
    hash_labeled_str(&mut hasher, "drv-path", &drv_path.to_absolute_path_with_prefix(store_prefix));
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
    let mut phase = RemoteProtocolPhase::Open;
    let mut frames = client_frames.iter();
    let hello = take_hello(&mut frames, &mut phase)?;
    let accepted = expect_accepted_hello(hello, builder)?;
    take_auth(&mut frames, &mut phase, ticket)?;
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
    build_response_frames(
        builder,
        request,
        &input_upload,
        client_transfer,
        phase,
        vec![auth_ok, missing_frame],
        missing,
        ticket.uses_remaining,
        executor,
    )
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
    match (phase, direction, kind) {
        (RemoteProtocolPhase::Open, RemoteFrameDirection::ClientToBuilder, RemoteFrameKind::Hello) => {
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
        _ => Err(format!(
            "unexpected-remote-frame phase={} direction={} frame={}",
            phase.as_str(),
            direction.as_str(),
            kind.as_str()
        )),
    }
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

pub fn validate_missing_uploads(
    missing_refs: &[String],
    uploaded_refs: &[String],
    uploaded_bytes: u64,
    max_upload_bytes: u64,
) -> Result<(), String> {
    if uploaded_bytes > max_upload_bytes || uploaded_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err("upload-byte-limit-exceeded".to_string());
    }
    let missing = missing_refs.iter().collect::<BTreeSet<_>>();
    let uploaded = uploaded_refs.iter().collect::<BTreeSet<_>>();
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
    if client.delta && builder.delta && !client.simulate_delta_failure && !builder.simulate_delta_failure {
        return Ok(delta_transfer_report(output_size_bytes, verified_builder_key));
    }
    if client.delta && builder.delta && (client.simulate_delta_failure || builder.simulate_delta_failure) {
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
        return Ok(full_transfer_report(output_size_bytes, verified_builder_key, None));
    }
    Err("no-compatible-output-transfer-mode".to_string())
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
        OutputTrustDecision::Accept { key_id, trust_basis } => Ok(RemoteOutputAdmissionReport {
            request_id: request.request_id.clone(),
            output_digest_blake3: result.output_digest_blake3.clone(),
            builder_signing_key_id: key_id,
            trust_basis,
            store_prefix: result.store_prefix.clone(),
            outputs: result.outputs.clone(),
            transfer_artifacts: transfer_artifacts.to_vec(),
            transfer: transfer.clone(),
        }),
        OutputTrustDecision::Reject(reason) => Err(reason),
    }
}

pub fn validate_remote_builder_response_output_import(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    response: &RemoteBuilderFrameResponse,
) -> Result<RemoteOutputAdmissionReport, String> {
    let import_frames = extract_output_import_frames(&response.response_frames, &request.request_id)?;
    if import_frames.transfer != &response.transfer {
        return Err("remote-output-transfer-report-mismatch".to_string());
    }
    if import_frames.result.outputs != response.outputs {
        return Err("remote-output-metadata-report-mismatch".to_string());
    }
    if import_frames.artifacts != response.transfer_artifacts {
        return Err("remote-output-transfer-artifact-report-mismatch".to_string());
    }
    validate_remote_output_admission(
        request,
        trusted_output_keys,
        import_frames.result,
        import_frames.transfer,
        &import_frames.artifacts,
    )
}

pub fn validate_remote_builder_frames_output_import(
    request: &ConcreteBuildRequest,
    trusted_output_keys: &[String],
    frames: &[RemoteFrame],
) -> Result<RemoteOutputAdmissionReport, String> {
    let import_frames = extract_output_import_frames(frames, &request.request_id)?;
    validate_remote_output_admission(
        request,
        trusted_output_keys,
        import_frames.result,
        import_frames.transfer,
        &import_frames.artifacts,
    )
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
    let actions = plan_remote_output_import_actions(request, admission)?;
    let store_report = remote_transfer_to_store_report(&admission.transfer);
    let mut imported = Vec::with_capacity(actions.len());
    for action in actions {
        let stored = persist_remote_output_action(store, &action, is_root, root_source).await?;
        store.record_verified_output_substitution_report(&action.store_path, store_report.clone());
        imported.push(imported_remote_output_report(store, &action, &stored));
    }
    Ok(RemoteOutputImportReport {
        request_id: admission.request_id.clone(),
        store_prefix: admission.store_prefix.clone(),
        outputs: imported,
        transfer: admission.transfer.clone(),
    })
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

fn parse_remote_output_store_path(logical_path: &str, store_prefix: &str) -> Result<StorePath<String>, String> {
    StorePath::from_absolute_path_with_prefix(logical_path.as_bytes(), store_prefix)
        .map_err(|_| "remote-output-store-path-invalid".to_string())
}

fn validate_remote_output_pathinfo(
    output: &RemoteProducedOutput,
    path_info: &PathInfo,
    store_path: &StorePath<String>,
    store_prefix: &str,
) -> Result<(), String> {
    validate_remote_pathinfo_binding(
        &output.name,
        &output.artifact_attestation_digest_blake3,
        path_info,
        store_path,
        store_prefix,
        &output.path_info_signing_key_id,
        RemotePathInfoErrorLabels {
            store_path_mismatch: "remote-output-pathinfo-store-path-mismatch",
            unsigned: "remote-output-pathinfo-unsigned",
            signing_key_mismatch: "remote-output-pathinfo-signing-key-mismatch",
            artifact_digest_failed: "remote-output-artifact-attestation-digest-failed",
            artifact_digest_mismatch: "remote-output-artifact-attestation-digest-mismatch",
        },
    )
}

#[derive(Clone, Copy)]
struct RemotePathInfoErrorLabels {
    store_path_mismatch: &'static str,
    unsigned: &'static str,
    signing_key_mismatch: &'static str,
    artifact_digest_failed: &'static str,
    artifact_digest_mismatch: &'static str,
}

fn validate_remote_pathinfo_binding(
    output_name: &str,
    artifact_attestation_digest_blake3: &str,
    path_info: &PathInfo,
    store_path: &StorePath<String>,
    store_prefix: &str,
    signing_key_id: &str,
    error_labels: RemotePathInfoErrorLabels,
) -> Result<(), String> {
    if path_info.store_path != *store_path {
        return Err(error_labels.store_path_mismatch.to_string());
    }
    if path_info.signatures.is_empty() {
        return Err(error_labels.unsigned.to_string());
    }
    if !pathinfo_has_signature_name(path_info, signing_key_id) {
        return Err(error_labels.signing_key_mismatch.to_string());
    }
    validate_remote_pathinfo_artifact_digest(
        output_name,
        artifact_attestation_digest_blake3,
        path_info,
        store_prefix,
        error_labels,
    )
}

fn validate_remote_pathinfo_artifact_digest(
    output_name: &str,
    artifact_attestation_digest_blake3: &str,
    path_info: &PathInfo,
    store_prefix: &str,
    error_labels: RemotePathInfoErrorLabels,
) -> Result<(), String> {
    let digest = crunch_store::artifact_attestation_digest_for_pathinfo(store_prefix, path_info, output_name, None)
        .map_err(|err| format!("{}: {err}", error_labels.artifact_digest_failed))?
        .to_hex();
    if digest != artifact_attestation_digest_blake3 {
        return Err(error_labels.artifact_digest_mismatch.to_string());
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
    }
}

async fn ingest_remote_output_nar_payload(
    store: &crunch_store::StoreHandle,
    action: &RemoteOutputImportAction,
) -> Result<(), String> {
    let Some(payload) = &action.nar_payload else {
        return Ok(());
    };
    let mut reader = std::io::Cursor::new(payload.as_slice());
    let (node, nar_sha256, nar_size) = snix_store::nar::ingest_nar_and_hash(
        store.blob_service(),
        store.directory_service(),
        &mut reader,
        &action.path_info.ca,
    )
    .await
    .map_err(|err| format!("remote-output-nar-ingest-failed: {err}"))?;
    if node != action.final_node {
        return Err("remote-output-nar-node-mismatch".to_string());
    }
    if nar_sha256 != action.path_info.nar_sha256 {
        return Err("remote-output-nar-sha256-mismatch".to_string());
    }
    if nar_size != action.path_info.nar_size {
        return Err("remote-output-nar-size-mismatch".to_string());
    }
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
    expect_authorized_ticket(ticket, &client.auth)?;
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
    Ok(RemoteLoopbackSessionReport {
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
    })
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

fn expect_authorized_ticket(ticket: &RemoteTicket, auth: &TicketAuthRequest) -> Result<(), String> {
    match authorize_ticket(ticket, auth) {
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

fn take_auth(
    frames: &mut std::slice::Iter<'_, RemoteFrame>,
    phase: &mut RemoteProtocolPhase,
    ticket: &RemoteTicket,
) -> Result<(), String> {
    let frame = frames.next().ok_or_else(|| "missing-auth-ticket-frame".to_string())?;
    *phase = validate_remote_transition(*phase, RemoteFrameDirection::ClientToBuilder, frame)?;
    let RemoteFrame::AuthTicket { auth } = frame else {
        return Err("expected-auth-ticket-frame".to_string());
    };
    expect_authorized_ticket(ticket, auth)
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

#[allow(clippy::too_many_arguments)]
fn build_response_frames(
    builder: &RemoteLoopbackBuilder,
    request: &ConcreteBuildRequest,
    input_upload: &RemoteInputUpload,
    client_transfer: RemoteTransferCapabilities,
    mut phase: RemoteProtocolPhase,
    mut response_frames: Vec<RemoteFrame>,
    missing: Vec<String>,
    ticket_uses_remaining: u32,
    executor: &dyn RemoteBuildExecutor,
) -> Result<RemoteBuilderFrameResponse, String> {
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
    Ok(RemoteBuilderFrameResponse {
        response_frames,
        missing_input_refs: missing,
        execution_plan_digest_blake3: execution.plan_digest_blake3,
        output_digest_blake3: execution.output_digest_blake3,
        outputs,
        transfer_artifacts,
        transfer,
        ticket_uses_remaining,
    })
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
    let mut artifacts = Vec::new();
    let mut done_seen = false;
    for frame in frames {
        match frame {
            RemoteFrame::BuildFinished { result: finished } => {
                if result.replace(finished).is_some() {
                    return Err("duplicate-build-finished-frame".to_string());
                }
            }
            RemoteFrame::OutputTransferArtifact { artifact } => {
                if result.is_none() {
                    return Err("remote-output-transfer-artifact-before-build-result".to_string());
                }
                if transfer.is_some() || done_seen {
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
                if done_seen {
                    return Err("duplicate-done-frame".to_string());
                }
                done_seen = true;
            }
            RemoteFrame::Error { .. } => return Err("remote-builder-error-frame".to_string()),
            _ => {}
        }
    }
    if !done_seen {
        return Err("missing-done-frame".to_string());
    }
    let result = result.ok_or_else(|| "missing-build-finished-frame".to_string())?;
    let transfer = transfer.ok_or_else(|| "missing-output-transfer-frame".to_string())?;
    validate_remote_output_transfer_artifacts(expected_request_id, &result.outputs, &artifacts)?;
    Ok(RemoteOutputImportFrames {
        result,
        transfer,
        artifacts,
    })
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
    }
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

fn read_bounded_stdio_input(mut reader: impl Read) -> Result<Vec<u8>, String> {
    let mut input = Vec::new();
    let limit =
        u64::try_from(MAX_REMOTE_STDIO_INPUT_BYTES).map_err(|_| "stdio-input-limit-conversion-failed".to_string())?;
    reader
        .by_ref()
        .take(limit.saturating_add(1))
        .read_to_end(&mut input)
        .map_err(|err| format!("reading stdio remote input: {err}"))?;
    if input.len() > MAX_REMOTE_STDIO_INPUT_BYTES {
        return Err(format!("remote-stdio-input-exceeds-{MAX_REMOTE_STDIO_INPUT_BYTES}"));
    }
    Ok(input)
}

pub fn redacted_ticket_view(ticket: &RemoteTicket) -> RemoteTicketView {
    RemoteTicketView {
        id: ticket.id.clone(),
        display_name: ticket.display_name.clone(),
        secret: SECRET_REDACTION.to_string(),
        created_unix_s: ticket.created_unix_s,
        expires_unix_s: ticket.expires_unix_s,
        uses_remaining: ticket.uses_remaining,
        max_build_time_secs: ticket.max_build_time_secs,
        max_upload_bytes: ticket.max_upload_bytes,
        bound_client_endpoint: ticket.bound_client_endpoint.clone(),
        revoked: ticket.revoked,
    }
}

pub fn revealed_ticket_view(ticket: &RemoteTicket) -> RemoteTicketView {
    let mut view = redacted_ticket_view(ticket);
    view.secret = ticket.secret.clone();
    view
}

pub fn create_ticket(
    state: &mut RemoteTicketState,
    display_name: String,
    now_unix_s: u64,
    ttl_secs: u64,
    uses: u32,
    max_build_time_secs: u64,
    max_upload_bytes: u64,
    bound_client_endpoint: Option<String>,
) -> Result<RemoteTicketView, RunError> {
    validate_ticket_limits(&display_name, ttl_secs, uses, max_build_time_secs, max_upload_bytes)?;
    let seed = format!("{display_name}:{now_unix_s}:{ttl_secs}:{uses}:{}", state.tickets.len());
    let secret_hash = blake3::hash(seed.as_bytes());
    let secret = secret_hash.to_hex().to_string();
    let id = blake3::hash(secret.as_bytes()).to_hex()[..16].to_string();
    let ticket = RemoteTicket {
        id: id.clone(),
        display_name,
        secret,
        created_unix_s: now_unix_s,
        expires_unix_s: now_unix_s.saturating_add(ttl_secs),
        uses_remaining: uses,
        max_build_time_secs,
        max_upload_bytes,
        bound_client_endpoint,
        revoked: false,
    };
    state.tickets.insert(id, ticket.clone());
    Ok(redacted_ticket_view(&ticket))
}

pub fn revoke_ticket(state: &mut RemoteTicketState, id: &str) -> Result<RemoteTicketView, RunError> {
    let ticket = state.tickets.get_mut(id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
    ticket.revoked = true;
    Ok(redacted_ticket_view(ticket))
}

pub fn load_ticket_state(state_dir: &Path) -> Result<RemoteTicketState, RunError> {
    let path = ticket_state_path(state_dir);
    if !path.exists() {
        return Ok(RemoteTicketState::default());
    }
    let bytes = fs::read(&path)
        .map_err(|err| RunError::Internal(format!("reading remote ticket state {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing remote ticket state {}: {err}", path.display())))
}

pub fn save_ticket_state(state_dir: &Path, state: &RemoteTicketState) -> Result<(), RunError> {
    let path = ticket_state_path(state_dir);
    let parent = path.parent().expect("ticket state path has parent");
    fs::create_dir_all(parent)
        .map_err(|err| RunError::Internal(format!("creating remote ticket state dir {}: {err}", parent.display())))?;
    let rendered = serde_json::to_string_pretty(state)
        .map_err(|err| RunError::Internal(format!("serializing remote ticket state: {err}")))?;
    let tmp = path.with_extension(TEMP_FILE_EXTENSION);
    fs::write(&tmp, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing remote ticket state temp {}: {err}", tmp.display())))?;
    fs::rename(&tmp, &path)
        .map_err(|err| RunError::Internal(format!("committing remote ticket state {}: {err}", path.display())))
}

fn coordinator_state_path(state_dir: &Path) -> PathBuf {
    state_dir.join(COORDINATOR_STATE_FILE)
}

pub fn load_coordinator_state(state_dir: &Path) -> Result<RemoteCoordinatorState, RunError> {
    let path = coordinator_state_path(state_dir);
    if !path.exists() {
        return Ok(RemoteCoordinatorState::default());
    }
    let bytes = fs::read(&path)
        .map_err(|err| RunError::Internal(format!("reading coordinator state {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing coordinator state {}: {err}", path.display())))
}

pub fn save_coordinator_state(state_dir: &Path, state: &RemoteCoordinatorState) -> Result<(), RunError> {
    let path = coordinator_state_path(state_dir);
    let parent = path.parent().expect("coordinator state path has parent");
    fs::create_dir_all(parent)
        .map_err(|err| RunError::Internal(format!("creating coordinator state dir {}: {err}", parent.display())))?;
    let rendered = serde_json::to_string_pretty(state)
        .map_err(|err| RunError::Internal(format!("serializing coordinator state: {err}")))?;
    let tmp = path.with_extension(TEMP_FILE_EXTENSION);
    fs::write(&tmp, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing coordinator state temp {}: {err}", tmp.display())))?;
    fs::rename(&tmp, &path)
        .map_err(|err| RunError::Internal(format!("committing coordinator state {}: {err}", path.display())))
}

fn validate_ticket_limits(
    display_name: &str,
    ttl_secs: u64,
    uses: u32,
    max_build_time_secs: u64,
    max_upload_bytes: u64,
) -> Result<(), RunError> {
    if display_name.is_empty() || display_name.len() > MAX_TICKET_DISPLAY_NAME_BYTES {
        return Err(RunError::Internal(format!("ticket display name exceeds {MAX_TICKET_DISPLAY_NAME_BYTES} bytes")));
    }
    if ttl_secs == 0 || uses == 0 {
        return Err(RunError::Internal("ticket ttl and uses must be non-zero".to_string()));
    }
    if max_build_time_secs == 0 || max_build_time_secs > MAX_REMOTE_BUILD_TIME_SECS {
        return Err(RunError::Internal(format!("ticket build time limit must be 1..={MAX_REMOTE_BUILD_TIME_SECS}")));
    }
    if max_upload_bytes > MAX_REMOTE_UPLOAD_BYTES {
        return Err(RunError::Internal(format!("ticket upload limit exceeds {MAX_REMOTE_UPLOAD_BYTES}")));
    }
    Ok(())
}

fn ticket_state_path(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join(TICKET_STATE_DIR).join(TICKET_STATE_FILE)
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
        crate::RemoteAction::Serve {
            endpoint_id,
            binding,
            signing_key_id,
            executor,
            present_input_refs,
        } => cmd_remote_serve(
            endpoint_id,
            binding,
            signing_key_id,
            executor,
            present_input_refs,
            output_dir,
            state_dir,
            store_prefix,
            json_output,
        ),
    }
}

fn remote_serve_executor(
    executor: crate::RemoteServeExecutor,
    fixture_signing_key_id: String,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<(String, Option<RemoteLocalBuildExecutor>), RunError> {
    match executor {
        crate::RemoteServeExecutor::Fixture => Ok((fixture_signing_key_id, None)),
        crate::RemoteServeExecutor::LocalBuild => {
            let keypair = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir, false)?;
            let local = RemoteLocalBuildExecutor {
                state_dir: state_dir.to_path_buf(),
                output_dir: output_dir.to_path_buf(),
                store_prefix: store_prefix.to_string(),
                keypair,
                trusted_keys: Vec::new(),
                trust_unsigned: false,
                verbose: false,
            };
            Ok((local.signing_key_id(), Some(local)))
        }
    }
}

fn cmd_remote_serve(
    endpoint_id: String,
    binding: crate::RemoteServeBinding,
    signing_key_id: String,
    executor: crate::RemoteServeExecutor,
    present_input_refs: Vec<String>,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match binding {
        crate::RemoteServeBinding::Metadata => {
            print_json_or_human(&remote_serve_metadata_json(&endpoint_id, "metadata-only"), json_output)
        }
        crate::RemoteServeBinding::StdioOnce => {
            let mut state = load_ticket_state(state_dir)?;
            let (builder_signing_key_id, local_executor) =
                remote_serve_executor(executor, signing_key_id, output_dir, state_dir, store_prefix)?;
            let builder = RemoteLoopbackBuilder {
                endpoint_id,
                store_prefix: store_prefix.to_string(),
                supported_capabilities: vec!["delta".to_string(), "full".to_string()],
                present_input_refs,
                signing_key_id: builder_signing_key_id,
                transfer_capabilities: RemoteTransferCapabilities::delta_and_full(),
            };
            let response = match local_executor {
                Some(local_executor) => plan_stdio_remote_once_from_state_with_executor(
                    std::io::stdin().lock(),
                    &builder,
                    &mut state,
                    RemoteTransferCapabilities::delta_and_full(),
                    &local_executor,
                ),
                None => plan_stdio_remote_once_from_state(
                    std::io::stdin().lock(),
                    &builder,
                    &mut state,
                    RemoteTransferCapabilities::delta_and_full(),
                ),
            }
            .map_err(|err| RunError::Internal(format!("remote stdio serve once: {err}")))?;
            save_ticket_state(state_dir, &state)?;
            write_remote_response_frames(std::io::stdout().lock(), &response)
                .map_err(|err| RunError::Internal(format!("remote stdio serve once response: {err}")))?;
            Ok(())
        }
    }
}

fn remote_serve_metadata_json(endpoint_id: &str, status: &str) -> serde_json::Value {
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
    let snapshot = coordinator_status_snapshot(&endpoint_id, concurrency, &coordinator_state, &tickets)
        .map_err(RunError::Internal)?;
    print_json_or_human(&snapshot, json_output)
}

fn cmd_remote_ticket(action: crate::RemoteTicketAction, state_dir: &Path, json_output: bool) -> Result<(), RunError> {
    let mut state = load_ticket_state(state_dir)?;
    match action {
        crate::RemoteTicketAction::Create {
            display_name,
            now_unix_s,
            ttl_secs,
            uses,
            max_build_time_secs,
            max_upload_bytes,
            bound_client_endpoint,
        } => {
            let view = create_ticket(
                &mut state,
                display_name,
                now_unix_s,
                ttl_secs,
                uses,
                max_build_time_secs,
                max_upload_bytes,
                bound_client_endpoint,
            )?;
            save_ticket_state(state_dir, &state)?;
            print_json_or_human(&view, json_output)
        }
        crate::RemoteTicketAction::List => {
            let views = state.tickets.values().map(redacted_ticket_view).collect::<Vec<_>>();
            print_json_or_human(&views, json_output)
        }
        crate::RemoteTicketAction::Inspect { id } => {
            let ticket =
                state.tickets.get(&id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
            print_json_or_human(&redacted_ticket_view(ticket), json_output)
        }
        crate::RemoteTicketAction::Reveal { id } => {
            let ticket =
                state.tickets.get(&id).ok_or_else(|| RunError::Internal(format!("unknown remote ticket {id}")))?;
            print_json_or_human(&revealed_ticket_view(ticket), json_output)
        }
        crate::RemoteTicketAction::Revoke { id } => {
            let view = revoke_ticket(&mut state, &id)?;
            save_ticket_state(state_dir, &state)?;
            print_json_or_human(&view, json_output)
        }
    }
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
    use super::*;

    const CUSTOM_REMOTE_OUTPUT_BYTES: u64 = 777;
    const REMOTE_CLIENT_DISPATCH_FRAME_COUNT: usize = 5;
    const ED25519_SIGNATURE_BYTES: usize = 64;
    const SHA256_DIGEST_BYTES: usize = 32;
    const BUILDER_SIGNATURE_FILL_BYTE: u8 = 3;
    const OTHER_SIGNATURE_FILL_BYTE: u8 = 4;
    const IMPORT_NAR_SHA256_FILL_BYTE: u8 = 7;
    const IMPORT_NAR_SIZE_BYTES: u64 = 1;
    const CORRUPTED_TRANSFER_PAYLOAD_BYTE: u8 = b'X';
    const OPERATOR_RAIL_LOG_START_CURSOR: u64 = 1;
    const OPERATOR_RAIL_LOG_NEXT_CURSOR: u64 = 2;

    #[test]
    fn compatible_hello_reaches_authorization() {
        let hello = RemoteHello {
            alpn: REMOTE_PROTOCOL_ALPN.to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: "builder-1".to_string(),
            capabilities: vec!["delta".to_string(), "full".to_string()],
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
        let mut ticket = fixture_ticket();
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
        assert!(err.contains("contains-raw-frontend-eval") || err.contains("raw-frontend-evaluation-rejected"),
            "expected frontend eval rejection, got: {err}");
    }

    #[test]
    fn valid_request_redeems_ticket_once() {
        let mut ticket = fixture_ticket();
        let auth = TicketAuthRequest {
            ticket_id: ticket.id.clone(),
            secret: ticket.secret.clone(),
            client_endpoint: Some("client-a".to_string()),
            now_unix_s: 2,
        };
        assert_eq!(authorize_ticket(&ticket, &auth), TicketDecision::Authorized);
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
        };
        validate_concrete_request(&request, &ticket).unwrap();
        redeem_after_queue(&mut ticket, true).unwrap();
        assert_eq!(ticket.uses_remaining, 0);
    }

    #[test]
    fn ticket_authorization_rejects_revoked_and_expired_tickets() {
        let mut revoked = fixture_ticket();
        revoked.revoked = true;
        let mut expired = fixture_ticket();
        expired.expires_unix_s = 2;
        let auth = TicketAuthRequest {
            ticket_id: revoked.id.clone(),
            secret: revoked.secret.clone(),
            client_endpoint: Some("client-a".to_string()),
            now_unix_s: 2,
        };

        assert_eq!(authorize_ticket(&revoked, &auth), TicketDecision::Reject("ticket-revoked".to_string()));
        assert_eq!(authorize_ticket(&expired, &auth), TicketDecision::Reject("ticket-expired".to_string()));
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
    fn remote_ticket_credential_parses_bearer_token() {
        let credential = parse_remote_ticket_credential("ticket-1:secret-1").expect("ticket parses");

        assert_eq!(credential.ticket_id, "ticket-1");
        assert_eq!(credential.secret, "secret-1");
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
        };
        let err = run_stdio_remote_child(&command).expect_err("sleeping child times out");
        let rendered = err.to_string();

        assert!(rendered.contains("timed out"));
        assert!(rendered.contains("TransportSetup"));
    }

    #[test]
    fn status_view_redacts_ticket_secret() {
        let ticket = fixture_ticket();
        let view = redacted_ticket_view(&ticket);
        assert_eq!(view.secret, SECRET_REDACTION);
        let revealed = revealed_ticket_view(&ticket);
        assert_eq!(revealed.secret, ticket.secret);
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
        stdout.extend(encode_remote_frame(&done).expect("done encodes"));
        let output = RemoteStdioChildOutput {
            stdout,
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
            stdout: encode_frame_stream(&response.response_frames),
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
            stdout: encode_frame_stream(&response.response_frames),
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
        let first = admit_coordinator_dispatch(&mut state, &request).expect("first request dispatches");
        let second = admit_coordinator_dispatch(&mut state, &request).expect("identical request attaches");

        let RemoteCoordinatorDispatchDecision::Dispatch {
            worker_endpoint_id,
            job_id,
            normalized_build_key,
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
    fn coordinator_resume_summary_redelivers_finished_result() {
        let request = fixture_coordinator_request();
        let normalized_key = normalized_remote_build_key(&request).expect("request key");
        let mut worker = fixture_worker_registration();
        worker.resumable_jobs.push(RemoteWorkerResumeSummary {
            job_id: coordinator_job_id(&normalized_key),
            normalized_build_key: normalized_key.clone(),
            phase: RemoteCoordinatorJobPhase::Finished,
            result_available: true,
            log_next_cursor: 3,
        });
        let mut state = RemoteCoordinatorState::default();
        let adopted = apply_worker_registration(&mut state, worker).expect("resumable worker registers");
        let decision = plan_coordinator_dispatch(&state, &request).expect("resumed request plans");

        assert_eq!(adopted, vec![coordinator_job_id(&normalized_key)]);
        assert!(
            matches!(decision, RemoteCoordinatorDispatchDecision::RedeliverResult { normalized_build_key, .. } if normalized_build_key == normalized_key)
        );
    }

    #[test]
    fn coordinator_rejects_untrusted_worker_key_without_using_coordinator_as_trust_root() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let mut request = fixture_coordinator_request();
        request.trusted_output_keys = vec!["other-key".to_string()];
        let decision = plan_coordinator_dispatch(&state, &request).expect("untrusted worker plans terminally");

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
        admit_coordinator_dispatch(&mut state, &request).expect("first request dispatches");
        let mut conflict = fixture_coordinator_request();
        conflict.request.payload = RemoteConcreteBuildPayload::Action {
            action_id: "action-2".to_string(),
            spec_json: fixture_action_spec_json("action-2"),
        };
        let decision = plan_coordinator_dispatch(&state, &conflict).expect("conflict plans");

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
        let feature_decision = plan_coordinator_dispatch(&state, &feature_mismatch).expect("feature mismatch plans");
        assert!(
            matches!(feature_decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "remote-worker-feature-mismatch")
        );

        let mut prefix_state = RemoteCoordinatorState::default();
        let mut prefix_worker = fixture_worker_registration();
        prefix_worker.logical_store_prefixes = vec!["/nix/store".to_string()];
        apply_worker_registration(&mut prefix_state, prefix_worker).expect("prefix worker registers");
        let store_prefix_mismatch = fixture_coordinator_request();
        let store_decision =
            plan_coordinator_dispatch(&prefix_state, &store_prefix_mismatch).expect("store mismatch plans");
        assert!(
            matches!(store_decision, RemoteCoordinatorDispatchDecision::Reject { reason } if reason == "remote-worker-store-prefix-mismatch")
        );

        let first = fixture_coordinator_request();
        admit_coordinator_dispatch(&mut state, &first).expect("first request dispatches");
        let mut second = fixture_coordinator_request();
        second.request.payload = RemoteConcreteBuildPayload::Action {
            action_id: "action-2".to_string(),
            spec_json: fixture_action_spec_json("action-2"),
        };
        second.live_output_claims = vec!["claim-second".to_string()];
        let resource_decision = plan_coordinator_dispatch(&state, &second).expect("resource mismatch plans");
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
    fn coordinator_log_replay_stays_bounded_for_slow_subscribers() {
        let existing = vec![
            RemoteCoordinatorLogChunk {
                cursor: 1,
                bytes: "aa".to_string(),
            },
            RemoteCoordinatorLogChunk {
                cursor: 2,
                bytes: "bb".to_string(),
            },
        ];
        let next = RemoteCoordinatorLogChunk {
            cursor: 3,
            bytes: "cc".to_string(),
        };
        let policy = RemoteLogRetentionPolicy {
            max_chunks: 2,
            max_bytes: 4,
        };
        let plan = retain_remote_log_chunks(&existing, next, 1, policy).expect("bounded log plan");

        assert!(plan.truncated);
        assert_eq!(plan.dropped_bytes, 2);
        assert_eq!(plan.retained_chunks.len(), 2);
        assert_eq!(plan.replay_chunks[0].cursor, 2);
        assert_eq!(plan.next_cursor, 4);
    }

    #[test]
    fn coordinator_status_redacts_ticket_secrets_and_splits_phases() {
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("worker registers");
        let request = fixture_coordinator_request();
        admit_coordinator_dispatch(&mut state, &request).expect("request dispatches");
        let ticket = fixture_ticket();
        let snapshot = coordinator_status_snapshot("coordinator-1", 1, &state, &[ticket.clone()])
            .expect("status snapshot renders");
        let rendered = serde_json::to_string(&snapshot).expect("status serializes");

        assert_eq!(snapshot.worker_count, 1);
        assert_eq!(snapshot.workers[0].output_signing_key_count, 1);
        assert_eq!(snapshot.queued_jobs.len(), 1);
        assert_eq!(snapshot.log_cursors.len(), 1);
        assert!(snapshot.active_jobs.is_empty());
        assert_eq!(snapshot.tickets[0].secret, SECRET_REDACTION);
        assert!(!rendered.contains(&format!("\"secret\":\"{}\"", ticket.secret)));
    }

    #[test]
    fn coordinator_status_bounds_failure_text_and_log_cursors() {
        let request = fixture_coordinator_request();
        let normalized_key = normalized_remote_build_key(&request).expect("request key");
        let mut state = RemoteCoordinatorState::default();
        state.jobs.insert("lost-job".to_string(), RemoteCoordinatorJobSummary {
            job_id: "lost-job".to_string(),
            normalized_build_key: normalized_key,
            assigned_worker_endpoint_id: Some("worker-secret".to_string()),
            phase: RemoteCoordinatorJobPhase::Lost,
            live_output_claims: Vec::new(),
            result_available: false,
            lost_phase: Some(RemoteFailurePhase::TransportSetup),
            log_start_cursor: 7,
            log_next_cursor: 9,
            short_error: Some(format!("diagnostic\u{0007}{}", "x".repeat(MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES))),
        });
        state.logs.insert("lost-job".to_string(), vec![RemoteCoordinatorLogChunk {
            cursor: 8,
            bytes: "log".to_string(),
        }]);
        let snapshot = coordinator_status_snapshot("coordinator-1", 1, &state, &[]).expect("status snapshot renders");
        let rendered = serde_json::to_string(&snapshot).expect("status serializes");

        assert_eq!(snapshot.recent_failures.len(), 1);
        assert_eq!(snapshot.recent_failures[0].lost_phase, Some(RemoteFailurePhase::TransportSetup));
        assert_eq!(snapshot.log_cursors[0].start_cursor, 7);
        assert_eq!(snapshot.log_cursors[0].next_cursor, 9);
        assert!(snapshot.recent_failures[0].short_error.as_ref().unwrap().contains("<truncated>"));
        assert!(!rendered.contains('\u{0007}'));
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
            upload_summary,
            &admission,
            &import_report,
            &[REMOTE_SESSION_NON_CLAIM.to_string()],
        )
        .expect("observability report builds");

        assert_eq!(report.selected_route, "p2p-remote-builder");
        assert_eq!(report.endpoint_id.as_deref(), Some("builder-1"));
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
            upload_summary,
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
            stdout: encode_frame_stream(&response.response_frames),
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
        state.jobs.insert("lost-job".to_string(), RemoteCoordinatorJobSummary {
            job_id: "lost-job".to_string(),
            normalized_build_key: normalized_key,
            assigned_worker_endpoint_id: None,
            phase: RemoteCoordinatorJobPhase::Lost,
            live_output_claims: Vec::new(),
            result_available: false,
            lost_phase: Some(RemoteFailurePhase::BuildExecution),
            log_start_cursor: 0,
            log_next_cursor: 0,
            short_error: Some("worker restart lost job".to_string()),
        });
        let decision = plan_coordinator_dispatch(&state, &request).expect("lost job decision");

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

    fn encode_frame_stream(frames: &[RemoteFrame]) -> Vec<u8> {
        let mut encoded = Vec::new();
        for frame in frames {
            encoded.extend(encode_remote_frame(frame).expect("test frame encodes"));
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
        if let Some(artifact) = response.transfer_artifacts.first_mut() {
            if let Some(byte) = artifact.payload.first_mut() {
                *byte = CORRUPTED_TRANSFER_PAYLOAD_BYTE;
            }
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

    fn fixture_ticket() -> RemoteTicket {
        RemoteTicket {
            id: "ticket-1".to_string(),
            display_name: "test".to_string(),
            secret: "secret".to_string(),
            created_unix_s: 1,
            expires_unix_s: 10,
            uses_remaining: 1,
            max_build_time_secs: 10,
            max_upload_bytes: 10,
            bound_client_endpoint: Some("client-a".to_string()),
            revoked: false,
        }
    }

    fn fixture_hello() -> RemoteHello {
        RemoteHello {
            alpn: REMOTE_PROTOCOL_ALPN.to_string(),
            version: REMOTE_PROTOCOL_VERSION,
            endpoint_id: "builder-1".to_string(),
            capabilities: vec!["delta".to_string(), "full".to_string()],
        }
    }

    fn fixture_auth_request() -> TicketAuthRequest {
        TicketAuthRequest {
            ticket_id: "ticket-1".to_string(),
            secret: "secret".to_string(),
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
                secret: "secret-1".to_string(),
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
            state_dir: PathBuf::from("/tmp/mantle-remote-build-test-state"),
            output_dir: PathBuf::from("/tmp/mantle-remote-build-test-store"),
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

    fn fixture_worker_registration() -> RemoteWorkerRegistration {
        RemoteWorkerRegistration {
            endpoint_id: "builder-1".to_string(),
            protocol_version: REMOTE_PROTOCOL_VERSION,
            systems: vec![DEFAULT_REMOTE_ACTION_SYSTEM.to_string()],
            feature_labels: vec!["kvm".to_string()],
            sandbox_modes: vec!["bwrap".to_string()],
            network_modes: vec!["off".to_string()],
            logical_store_prefixes: vec!["/mantle/store".to_string()],
            concurrency: 1,
            output_signing_key_ids: vec!["builder-key".to_string()],
            resumable_jobs: Vec::new(),
        }
    }

    fn fixture_coordinator_request() -> RemoteCoordinatorBuildRequest {
        RemoteCoordinatorBuildRequest {
            request: fixture_request(),
            required_system: DEFAULT_REMOTE_ACTION_SYSTEM.to_string(),
            required_features: vec!["kvm".to_string()],
            required_sandbox_mode: "bwrap".to_string(),
            required_network_mode: "off".to_string(),
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
        let mut state = RemoteCoordinatorState::default();
        apply_worker_registration(&mut state, fixture_worker_registration()).expect("operator rail worker registers");
        let request = fixture_remote_operator_coordinator_request(client);
        let decision = admit_coordinator_dispatch(&mut state, &request).expect("operator rail dispatch admits");
        let job_id = match decision {
            RemoteCoordinatorDispatchDecision::Dispatch { job_id, .. } => job_id,
            _ => panic!("operator rail request should dispatch"),
        };
        let job = state.jobs.get_mut(&job_id).expect("operator rail job exists");
        job.phase = RemoteCoordinatorJobPhase::Finished;
        job.result_available = true;
        job.log_start_cursor = OPERATOR_RAIL_LOG_START_CURSOR;
        job.log_next_cursor = OPERATOR_RAIL_LOG_NEXT_CURSOR;
        state.logs.insert(job_id, vec![RemoteCoordinatorLogChunk {
            cursor: OPERATOR_RAIL_LOG_START_CURSOR,
            bytes: "remote fixture log".to_string(),
        }]);
        coordinator_status_snapshot("builder-1", DEFAULT_REMOTE_CONCURRENCY, &state, &[fixture_ticket()])
            .expect("operator rail status renders")
    }

    fn fixture_remote_operator_coordinator_request(client: &RemoteLoopbackClient) -> RemoteCoordinatorBuildRequest {
        RemoteCoordinatorBuildRequest {
            request: client.request.clone(),
            required_system: DEFAULT_REMOTE_ACTION_SYSTEM.to_string(),
            required_features: vec!["kvm".to_string()],
            required_sandbox_mode: "bwrap".to_string(),
            required_network_mode: "off".to_string(),
            trusted_output_keys: vec!["builder-key".to_string()],
            live_output_claims: vec!["claim-out".to_string()],
            wait_for_worker: false,
        }
    }
}
