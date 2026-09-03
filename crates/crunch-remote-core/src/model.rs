use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const REMOTE_PROTOCOL_ALPN: &str = "mantle-remote-build/1";
pub const REMOTE_PROTOCOL_VERSION: u32 = 1;
pub const REMOTE_COMMAND_SCHEMA: &str = "mantle-remote-command-v1";
pub const REMOTE_EFFECT_SCHEMA: &str = "mantle-remote-effect-v1";
pub const REMOTE_OBSERVATION_SCHEMA: &str = "mantle-remote-observation-v1";
pub const REMOTE_RECEIPT_PREIMAGE_SCHEMA: &str = "mantle-remote-receipt-preimage-v1";
pub const REMOTE_WIRE_FRAME_SCHEMA: &str = "mantle-remote-wire-frame-v1";
pub const REMOTE_CORE_NON_CLAIM: &str =
    "remote-core-decisions-do-not-prove-effect-success-worker-honesty-output-trust-or-release-eligibility";
pub const MAX_REMOTE_CAPABILITIES: usize = 32;
pub const MAX_REMOTE_INPUT_REFS: usize = 1_000_000;
pub const MAX_REMOTE_EXPECTED_OUTPUTS: usize = 128;
pub const MAX_REMOTE_BUILD_PAYLOAD_BYTES: usize = 1_048_576;
pub const MAX_REMOTE_EFFECT_STEPS: u32 = 32;
pub const MAX_REMOTE_ATTEMPTS: u32 = 32;
pub const MAX_REMOTE_UPLOAD_BYTES: u64 = 1_099_511_627_776;
pub const MAX_REMOTE_BUILD_TIME_MS: u64 = 86_400_000;
pub const MAX_REMOTE_MEMORY_BYTES: u64 = 1_099_511_627_776;
pub const MAX_REMOTE_CPU_UNITS: u32 = 65_536;
pub const BLAKE3_HEX_LENGTH: usize = 64;
const REALIZATION_KEY_SHORT_CHARS: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteCommand {
    pub schema: String,
    pub job_id: String,
    pub request_blake3: String,
    pub worker_id: String,
    pub fence_generation: u64,
    pub input_refs: Vec<String>,
    pub expected_outputs: Vec<RemoteOutputExpectation>,
    pub policy: RemotePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemotePolicy {
    pub attempts_max: u32,
    pub input_bytes_max: u64,
    pub output_bytes_max: u64,
    pub build_time_ms_max: u64,
    pub cpu_units_max: u32,
    pub memory_bytes_max: u64,
    pub allow_transient_retry: bool,
    pub require_output_trust: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteOutputExpectation {
    pub name: String,
    pub logical_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteOutputFact {
    pub name: String,
    pub logical_path: String,
    pub content_digest_blake3: String,
    pub size_bytes: u64,
    pub artifact_attestation_blake3: String,
    pub path_info_blake3: Option<String>,
    pub substitution: Option<RemoteSubstitutionFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteSubstitutionFact {
    pub mode: RemoteSubstitutionMode,
    pub transferred_bytes: u64,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteSubstitutionMode {
    Delta,
    Full,
    Streaming,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedRemoteArtifact {
    pub outputs: BTreeMap<String, RemoteOutputFact>,
}

impl VerifiedRemoteArtifact {
    pub fn new(outputs: BTreeMap<String, RemoteOutputFact>) -> Result<Self, crate::RemoteCoreError> {
        crate::validate_verified_artifact(Self { outputs })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemotePhase {
    Authenticating,
    SendingHandshake,
    ObservingClock,
    GeneratingAttempt,
    LoadingAttempt,
    PersistingAttempt,
    ReservingLease,
    TransferringInputs,
    Executing,
    AdmittingOutputs,
    ReleasingLease,
    PublishingTelemetry,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteCapability {
    Transport,
    AttemptPersistence,
    Executor,
    StoreAdmission,
    CredentialVerification,
    ClockObservation,
    RandomIdentifier,
    TelemetryPublication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteEffectKind {
    VerifyCredential,
    SendHandshake,
    ObserveClock,
    GenerateAttemptId,
    LoadAttempt,
    PersistAttempt,
    ReserveLease,
    TransferInputs,
    LaunchExecutor,
    AdmitOutputs,
    ReleaseLease,
    PublishTelemetry,
}

impl RemoteEffectKind {
    #[must_use]
    pub const fn capability(self) -> RemoteCapability {
        match self {
            Self::VerifyCredential => RemoteCapability::CredentialVerification,
            Self::SendHandshake | Self::TransferInputs => RemoteCapability::Transport,
            Self::ObserveClock => RemoteCapability::ClockObservation,
            Self::GenerateAttemptId => RemoteCapability::RandomIdentifier,
            Self::LoadAttempt | Self::PersistAttempt | Self::ReserveLease | Self::ReleaseLease => {
                RemoteCapability::AttemptPersistence
            }
            Self::LaunchExecutor => RemoteCapability::Executor,
            Self::AdmitOutputs => RemoteCapability::StoreAdmission,
            Self::PublishTelemetry => RemoteCapability::TelemetryPublication,
        }
    }

    #[must_use]
    pub const fn expected_observation(self) -> RemoteObservationKind {
        match self {
            Self::VerifyCredential => RemoteObservationKind::CredentialVerified,
            Self::SendHandshake => RemoteObservationKind::HandshakeSent,
            Self::ObserveClock => RemoteObservationKind::ClockObserved,
            Self::GenerateAttemptId => RemoteObservationKind::AttemptIdGenerated,
            Self::LoadAttempt => RemoteObservationKind::AttemptLoaded,
            Self::PersistAttempt => RemoteObservationKind::AttemptPersisted,
            Self::ReserveLease => RemoteObservationKind::LeaseReserved,
            Self::TransferInputs => RemoteObservationKind::InputsTransferred,
            Self::LaunchExecutor => RemoteObservationKind::ExecutorCompleted,
            Self::AdmitOutputs => RemoteObservationKind::OutputsAdmitted,
            Self::ReleaseLease => RemoteObservationKind::LeaseReleased,
            Self::PublishTelemetry => RemoteObservationKind::TelemetryPublished,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteObservationKind {
    CredentialVerified,
    HandshakeSent,
    ClockObserved,
    AttemptIdGenerated,
    AttemptLoaded,
    AttemptPersisted,
    LeaseReserved,
    InputsTransferred,
    ExecutorCompleted,
    OutputsAdmitted,
    LeaseReleased,
    TelemetryPublished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteObservationStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteFailureClass {
    Transient,
    Permanent,
    StaleFence,
    LimitExceeded,
    UntrustedOutput,
    Protocol,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteEffectLimits {
    pub bytes_max: u64,
    pub outputs_max: u32,
    pub build_time_ms_max: u64,
    pub cpu_units_max: u32,
    pub memory_bytes_max: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteEffect {
    pub schema: String,
    pub effect_id_blake3: String,
    pub sequence: u32,
    pub job_id: String,
    pub attempt_id: Option<String>,
    pub fence_generation: u64,
    pub capability: RemoteCapability,
    pub kind: RemoteEffectKind,
    pub limits: RemoteEffectLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteObservation {
    pub schema: String,
    pub effect_id_blake3: String,
    pub attempt_id: Option<String>,
    pub fence_generation: u64,
    pub kind: RemoteObservationKind,
    pub status: RemoteObservationStatus,
    pub failure_class: Option<RemoteFailureClass>,
    pub reason_code: Option<String>,
    pub observed_bytes: u64,
    pub observed_outputs: u32,
    pub output_trusted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteEvent {
    pub sequence: u32,
    pub from: RemotePhase,
    pub to: RemotePhase,
    pub reason_code: String,
    pub effect_id_blake3: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteOutcomeStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteOutcome {
    pub status: RemoteOutcomeStatus,
    pub reason_code: String,
    pub attempts: u32,
    pub receipt_preimage_blake3: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteReceiptPreimage {
    pub schema: String,
    pub command_blake3: String,
    pub job_id: String,
    pub worker_id: String,
    pub attempt_id: Option<String>,
    pub fence_generation: u64,
    pub phase: RemotePhase,
    pub event_reason_codes: Vec<String>,
    pub pending_effect_blake3: Option<String>,
    pub outcome_status: Option<RemoteOutcomeStatus>,
    pub outcome_reason_code: Option<String>,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteSession {
    pub command: RemoteCommand,
    pub command_blake3: String,
    pub phase: RemotePhase,
    pub sequence: u32,
    pub attempt_ordinal: u32,
    pub attempt_id: Option<String>,
    pub pending_effect: Option<RemoteEffect>,
    pub retry_after_release: bool,
    pub events: Vec<RemoteEvent>,
    pub outcome: Option<RemoteOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloAdmissionInput {
    pub alpn: String,
    pub version: u32,
    pub endpoint_id: String,
    pub expected_endpoint_id: String,
    pub capabilities: Vec<String>,
    pub supported_capabilities: Vec<String>,
    pub workspace_policy_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedHelloFacts {
    pub endpoint_id: String,
    pub accepted_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloAdmissionDecision {
    Proceed(AcceptedHelloFacts),
    Reject(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingInputFacts {
    pub declared_refs: Vec<String>,
    pub present_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputTrustInput {
    pub signing_key_id: String,
    pub trusted_key_ids: Vec<String>,
    pub store_prefix_matches: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteOutputTrustDecision {
    Accept {
        key_id: String,
        key_material_digest_blake3: Option<String>,
    },
    Reject(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteBuildFallbackPolicy {
    Never,
    OnRemoteFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFailurePlan {
    ReturnFailure { phase: String, reason: String },
    FallbackToLocal { phase: String, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteBuildServiceRequestFacts {
    pub command_arg_count: u32,
    pub output_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteBuildIdentitySource {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteBuildIdentityInput {
    pub store_prefix: String,
    pub source: RemoteBuildIdentitySource,
    pub system: String,
    pub command_args: Vec<String>,
    pub command_env: BTreeMap<String, String>,
    pub input_refs: Vec<String>,
    pub source_input_refs: Vec<String>,
    pub expected_outputs: Vec<RemoteOutputExpectation>,
    pub failure_replay_source_bundle_blake3: Option<String>,
    pub failure_replay_execution_blake3: Option<String>,
    pub required_system: String,
    pub required_features: Vec<String>,
    pub required_sandbox_mode: String,
    pub required_network_mode: String,
    pub semantic_accelerator_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RealizationKey(pub(crate) String);

impl RealizationKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn short(&self) -> &str {
        self.0.get(..REALIZATION_KEY_SHORT_CHARS).unwrap_or(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizationKeyRequest {
    pub derivation: DerivationKeyFacts,
    pub input_closure: Vec<InputClosureFact>,
    pub platform: PlatformFacts,
    pub toolchains: Vec<ToolchainFact>,
    pub sandbox: SandboxFacts,
    pub environment: BTreeMap<String, Vec<u8>>,
    pub store: StorePrefixFacts,
    pub realizer_profile: RealizerProfileFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationKeyFacts {
    pub identity: String,
    pub builder: String,
    pub args: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputClosureFact {
    pub store_path: String,
    pub nar_hash: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformFacts {
    pub system: String,
    pub cpu: String,
    pub os: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainFact {
    pub name: String,
    pub store_path: String,
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxFacts {
    pub hermeticity: String,
    pub network_allowed: bool,
    pub fixed_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorePrefixFacts {
    pub logical_prefix: String,
    pub output_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizerProfileFacts {
    pub name: String,
    pub version: u32,
    pub capabilities: Vec<String>,
    pub parameters: BTreeMap<String, String>,
}
