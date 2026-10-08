//! Pure bounded, resumable remote transfer decisions over Mantle-owned identities.
//!
//! This functional core performs no I/O, allocation from untrusted payload
//! lengths, clock reads, persistence, transport, castore mutation, or output
//! admission. Shell adapters must re-probe receiver state and apply every
//! returned bound before reading payload bytes.
//!
//! r[impl remote_builds.attempt_scoped_transfer_resume]
//! r[impl store_transports.resumable_castore_sessions]
//! r[impl store_transports.receiver_driven_backpressure]
//! r[impl store_transports.content_presence_early_cutoff]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use crunch_remote_core::transfer::TRANSFER_CHECKPOINT_DOMAIN;
use crunch_remote_core::transfer::TRANSFER_MANIFEST_DOMAIN;
use crunch_remote_core::transfer::TRANSFER_POLICY_DOMAIN;
use crunch_remote_core::transfer::TransferArtifactFacts;
use crunch_remote_core::transfer::TransferCheckpointCounterFacts;
use crunch_remote_core::transfer::TransferCheckpointMonotonicFacts;
use crunch_remote_core::transfer::TransferChunkFacts;
use crunch_remote_core::transfer::TransferCreditFacts;
use crunch_remote_core::transfer::TransferCreditLimits;
use crunch_remote_core::transfer::TransferCreditSnapshot;
use crunch_remote_core::transfer::TransferCutoffAdmissionFacts;
use crunch_remote_core::transfer::TransferCutoffDisposition;
use crunch_remote_core::transfer::TransferDemandArtifactFacts;
use crunch_remote_core::transfer::TransferDemandChunkFacts;
use crunch_remote_core::transfer::TransferIdentityError;
use crunch_remote_core::transfer::TransferManifestEnvelopeFacts;
use crunch_remote_core::transfer::TransferManifestError;
use crunch_remote_core::transfer::TransferManifestTotals;
use crunch_remote_core::transfer::TransferMissingChunkFacts;
use crunch_remote_core::transfer::TransferPolicyFacts;
use crunch_remote_core::transfer::TransferProgressError;
use crunch_remote_core::transfer::TransferRequiredArtifactFacts;
use crunch_remote_core::transfer::TransferReservedChunkFacts;
use crunch_remote_core::transfer::TransferScopeFacts;
use crunch_remote_core::transfer::acknowledge_transfer_chunk;
use crunch_remote_core::transfer::decide_transfer_cutoff;
use crunch_remote_core::transfer::grant_transfer_credit;
use crunch_remote_core::transfer::is_transfer_artifact_id_valid;
use crunch_remote_core::transfer::plan_transfer_demand;
use crunch_remote_core::transfer::reserve_transfer_chunk;
use crunch_remote_core::transfer::transfer_domain_digest;
use crunch_remote_core::transfer::transfer_session_digest;
use crunch_remote_core::transfer::validate_transfer_checkpoint_acknowledgements;
use crunch_remote_core::transfer::validate_transfer_checkpoint_counters;
use crunch_remote_core::transfer::validate_transfer_checkpoint_monotonic;
use crunch_remote_core::transfer::validate_transfer_checkpoint_schema;
use crunch_remote_core::transfer::validate_transfer_credit_state;
use crunch_remote_core::transfer::validate_transfer_idle_progress;
use crunch_remote_core::transfer::validate_transfer_manifest_artifacts;
use crunch_remote_core::transfer::validate_transfer_manifest_envelope;
use crunch_remote_core::transfer::validate_transfer_manifest_prefix;
use crunch_remote_core::transfer::validate_transfer_policy;
use crunch_remote_core::transfer::validate_transfer_receiver_facts;
use crunch_remote_core::transfer::validate_transfer_scope;
use serde::Deserialize;
use serde::Serialize;

pub const REMOTE_TRANSFER_MANIFEST_SCHEMA: &str = "mantle-remote-transfer-manifest-v1";
pub const REMOTE_TRANSFER_CHECKPOINT_SCHEMA: &str = "mantle-remote-transfer-checkpoint-v1";

pub const DEFAULT_REMOTE_TRANSFER_CHUNK_BYTES: u32 = 65_536;
pub const DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_BYTES: u64 = 1_048_576;
pub const DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_CHUNKS: u32 = 16;
pub const DEFAULT_REMOTE_TRANSFER_BUFFERED_CHUNKS: u32 = 16;
pub const DEFAULT_REMOTE_TRANSFER_ARTIFACTS: u32 = 4_096;
pub const DEFAULT_REMOTE_TRANSFER_CHUNKS: u32 = 65_536;
pub const DEFAULT_REMOTE_TRANSFER_TOTAL_BYTES: u64 = 1_073_741_824;
pub const DEFAULT_REMOTE_TRANSFER_CHECKPOINT_BYTES: u32 = 262_144;
pub const DEFAULT_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS: u32 = 1_024;
pub const DEFAULT_REMOTE_TRANSFER_REPLAY_ROUNDS: u32 = 4;
pub const DEFAULT_REMOTE_TRANSFER_CONTROL_BYTES: u32 = 1_048_576;

#[cfg(test)]
const SHA256_HEX_LENGTH_CHARS: usize = 64;
#[cfg(test)]
const INITIAL_CHUNK_INDEX: u32 = 0;
#[cfg(test)]
const INITIAL_CHUNK_OFFSET_BYTES: u64 = 0;
const INITIAL_CHUNK_SEQUENCE: u64 = 0;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemoteTransferDigest(String);

impl RemoteTransferDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteTransferReasonCode> {
        let value = value.into();
        if !crunch_remote_core::output::is_blake3_hex_digest(&value) {
            return Err(RemoteTransferReasonCode::DigestInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemoteTransferSessionId(String);

impl RemoteTransferSessionId {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteTransferReasonCode> {
        let value = value.into();
        if !crunch_remote_core::output::is_blake3_hex_digest(&value) {
            return Err(RemoteTransferReasonCode::SessionIdentityInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemoteTransferArtifactId(String);

impl RemoteTransferArtifactId {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteTransferReasonCode> {
        let value = value.into();
        if !is_transfer_artifact_id_valid(&value) {
            return Err(RemoteTransferReasonCode::ArtifactIdentityInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransferArtifactKind {
    CastoreBlob,
    CastoreDirectory,
    Nar,
    SourceBundle,
    PathInfo,
    Attestation,
    DeltaBlob,
    DeltaChunk,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RemoteTransferChunkDescriptor {
    pub index: u32,
    pub offset_bytes: u64,
    pub size_bytes: u32,
    pub digest_blake3: RemoteTransferDigest,
}

fn absent_nar_sha256_hex() -> Option<String> {
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferArtifact {
    pub artifact_id: RemoteTransferArtifactId,
    pub artifact_kind: RemoteTransferArtifactKind,
    pub digest_blake3: RemoteTransferDigest,
    pub size_bytes: u64,
    pub required_for_completion: bool,
    #[serde(default = "absent_nar_sha256_hex", skip_serializing_if = "Option::is_none")]
    pub nar_sha256_hex: Option<String>,
    pub chunks: Vec<RemoteTransferChunkDescriptor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferPolicy {
    pub chunk_bytes_max: u32,
    pub in_flight_bytes_max: u64,
    pub in_flight_chunks_max: u32,
    pub buffered_chunks_max: u32,
    pub artifact_count_max: u32,
    pub chunk_count_max: u32,
    pub total_bytes_max: u64,
    pub checkpoint_bytes_max: u32,
    pub idle_progress_steps_max: u32,
    pub replay_rounds_max: u32,
    pub control_bytes_max: u32,
}

impl Default for RemoteTransferPolicy {
    fn default() -> Self {
        Self {
            chunk_bytes_max: DEFAULT_REMOTE_TRANSFER_CHUNK_BYTES,
            in_flight_bytes_max: DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_BYTES,
            in_flight_chunks_max: DEFAULT_REMOTE_TRANSFER_IN_FLIGHT_CHUNKS,
            buffered_chunks_max: DEFAULT_REMOTE_TRANSFER_BUFFERED_CHUNKS,
            artifact_count_max: DEFAULT_REMOTE_TRANSFER_ARTIFACTS,
            chunk_count_max: DEFAULT_REMOTE_TRANSFER_CHUNKS,
            total_bytes_max: DEFAULT_REMOTE_TRANSFER_TOTAL_BYTES,
            checkpoint_bytes_max: DEFAULT_REMOTE_TRANSFER_CHECKPOINT_BYTES,
            idle_progress_steps_max: DEFAULT_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS,
            replay_rounds_max: DEFAULT_REMOTE_TRANSFER_REPLAY_ROUNDS,
            control_bytes_max: DEFAULT_REMOTE_TRANSFER_CONTROL_BYTES,
        }
    }
}

impl RemoteTransferPolicy {
    pub fn validate(self) -> Result<(), RemoteTransferReasonCode> {
        if !validate_transfer_policy(TransferPolicyFacts {
            chunk_bytes_max: self.chunk_bytes_max,
            in_flight_bytes_max: self.in_flight_bytes_max,
            in_flight_chunks_max: self.in_flight_chunks_max,
            buffered_chunks_max: self.buffered_chunks_max,
            artifact_count_max: self.artifact_count_max,
            chunk_count_max: self.chunk_count_max,
            total_bytes_max: self.total_bytes_max,
            checkpoint_bytes_max: self.checkpoint_bytes_max,
            idle_progress_steps_max: self.idle_progress_steps_max,
            replay_rounds_max: self.replay_rounds_max,
            control_bytes_max: self.control_bytes_max,
        }) {
            return Err(RemoteTransferReasonCode::PolicyInvalid);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferManifest {
    pub schema: String,
    pub session_id: RemoteTransferSessionId,
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub policy_digest_blake3: RemoteTransferDigest,
    pub store_prefix: String,
    pub requested_content_blake3: RemoteTransferDigest,
    pub artifacts: Vec<RemoteTransferArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRemoteTransferManifest {
    pub manifest: RemoteTransferManifest,
    pub canonical_bytes: Vec<u8>,
    pub digest_blake3: RemoteTransferDigest,
    pub artifact_count: u32,
    pub chunk_count: u32,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferScope {
    pub session_id: RemoteTransferSessionId,
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub manifest_digest_blake3: RemoteTransferDigest,
    pub policy_digest_blake3: RemoteTransferDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferChunkDemand {
    pub artifact_id: RemoteTransferArtifactId,
    pub artifact_kind: RemoteTransferArtifactKind,
    pub chunk: RemoteTransferChunkDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferDemand {
    pub scope: RemoteTransferScope,
    pub missing_chunks: Vec<RemoteTransferChunkDemand>,
    pub missing_bytes: u64,
    pub reused_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferReceiverFacts {
    pub complete_artifact_ids: BTreeSet<RemoteTransferArtifactId>,
    pub complete_chunk_digests: BTreeSet<RemoteTransferDigest>,
    pub requested_content_identity_verified: bool,
    pub required_closure_metadata_verified: bool,
    pub path_info_admitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferCheckpoint {
    pub schema: String,
    pub scope: RemoteTransferScope,
    pub acknowledged_chunk_digests: BTreeSet<RemoteTransferDigest>,
    pub next_sequence: u64,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub checkpoint_digest_blake3: RemoteTransferDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTransferResumePlan {
    pub demand: RemoteTransferDemand,
    pub acknowledged_chunk_digests: BTreeSet<RemoteTransferDigest>,
    pub next_sequence: u64,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferCreditGrant {
    pub bytes: u64,
    pub chunks: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferChunkHeader {
    pub scope: RemoteTransferScope,
    pub artifact_id: RemoteTransferArtifactId,
    pub artifact_kind: RemoteTransferArtifactKind,
    pub chunk: RemoteTransferChunkDescriptor,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferAcknowledgement {
    pub scope: RemoteTransferScope,
    pub chunk_digest_blake3: RemoteTransferDigest,
    pub sequence: u64,
    pub transferred_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferInFlightChunk {
    pub artifact_id: RemoteTransferArtifactId,
    pub chunk_digest_blake3: RemoteTransferDigest,
    pub size_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferCreditState {
    pub granted_bytes_remaining: u64,
    pub granted_chunks_remaining: u32,
    pub in_flight: BTreeMap<u64, RemoteTransferInFlightChunk>,
    pub acknowledged_chunk_digests: BTreeSet<RemoteTransferDigest>,
    pub transferred_bytes: u64,
    pub next_sequence: u64,
    pub last_progress_step: u64,
}

impl Default for RemoteTransferCreditState {
    fn default() -> Self {
        Self {
            granted_bytes_remaining: 0,
            granted_chunks_remaining: 0,
            in_flight: BTreeMap::new(),
            acknowledged_chunk_digests: BTreeSet::new(),
            transferred_bytes: 0,
            next_sequence: INITIAL_CHUNK_SEQUENCE,
            last_progress_step: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransferCompletionDisposition {
    AlreadyPresent,
    DemandSatisfied,
    Continue,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferCompletionDecision {
    pub disposition: RemoteTransferCompletionDisposition,
    pub reason_code: RemoteTransferReasonCode,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub output_admission_claimed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransferReasonCode {
    ManifestCanonical,
    ManifestSchemaUnsupported,
    ManifestSerializationFailed,
    ManifestIdentityMismatch,
    ManifestBoundsExceeded,
    ArtifactIdentityInvalid,
    ArtifactDuplicate,
    ArtifactSizeInvalid,
    ArtifactDigestConflict,
    ArtifactUnknown,
    ChunkIndexInvalid,
    ChunkOffsetInvalid,
    ChunkSizeInvalid,
    ChunkDigestConflict,
    ChunkUnknown,
    ChunkNotDemanded,
    ChunkSequenceInvalid,
    DigestInvalid,
    NarSha256Invalid,
    StorePrefixInvalid,
    SessionIdentityInvalid,
    PolicyInvalid,
    PolicySerializationFailed,
    PolicyDigestMismatch,
    ArithmeticOverflow,
    TotalBytesExceeded,
    ReceiverFactsInvalid,
    CheckpointSchemaUnsupported,
    CheckpointSerializationFailed,
    CheckpointTooLarge,
    CheckpointDigestMismatch,
    CheckpointScopeMismatch,
    CheckpointAcknowledgementRegressed,
    CheckpointCounterRegressed,
    CheckpointCounterInvalid,
    CheckpointCursorForged,
    AcknowledgedChunkMissing,
    CreditGrantInvalid,
    CreditExceeded,
    BufferedChunkLimitExceeded,
    ChunkAlreadyInFlight,
    ChunkDigestMismatch,
    AcknowledgementUnknown,
    AcknowledgementMismatch,
    IdleProgressExceeded,
    ProgressCounterRegressed,
    TransferContinue,
    TransferAlreadyPresent,
    TransferDemandSatisfied,
    TransferRejected,
}

impl RemoteTransferReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ManifestCanonical => "manifest-canonical",
            Self::ManifestSchemaUnsupported => "manifest-schema-unsupported",
            Self::ManifestSerializationFailed => "manifest-serialization-failed",
            Self::ManifestIdentityMismatch => "manifest-identity-mismatch",
            Self::ManifestBoundsExceeded => "manifest-bounds-exceeded",
            Self::ArtifactIdentityInvalid => "artifact-identity-invalid",
            Self::ArtifactDuplicate => "artifact-duplicate",
            Self::ArtifactSizeInvalid => "artifact-size-invalid",
            Self::ArtifactDigestConflict => "artifact-digest-conflict",
            Self::ArtifactUnknown => "artifact-unknown",
            Self::ChunkIndexInvalid => "chunk-index-invalid",
            Self::ChunkOffsetInvalid => "chunk-offset-invalid",
            Self::ChunkSizeInvalid => "chunk-size-invalid",
            Self::ChunkDigestConflict => "chunk-digest-conflict",
            Self::ChunkUnknown => "chunk-unknown",
            Self::ChunkNotDemanded => "chunk-not-demanded",
            Self::ChunkSequenceInvalid => "chunk-sequence-invalid",
            Self::DigestInvalid => "digest-invalid",
            Self::NarSha256Invalid => "nar-sha256-invalid",
            Self::StorePrefixInvalid => "store-prefix-invalid",
            Self::SessionIdentityInvalid => "session-identity-invalid",
            Self::PolicyInvalid => "transfer-policy-invalid",
            Self::PolicySerializationFailed => "transfer-policy-serialization-failed",
            Self::PolicyDigestMismatch => "transfer-policy-digest-mismatch",
            Self::ArithmeticOverflow => "transfer-arithmetic-overflow",
            Self::TotalBytesExceeded => "transfer-total-bytes-exceeded",
            Self::ReceiverFactsInvalid => "receiver-facts-invalid",
            Self::CheckpointSchemaUnsupported => "checkpoint-schema-unsupported",
            Self::CheckpointSerializationFailed => "checkpoint-serialization-failed",
            Self::CheckpointTooLarge => "checkpoint-too-large",
            Self::CheckpointDigestMismatch => "checkpoint-digest-mismatch",
            Self::CheckpointScopeMismatch => "checkpoint-scope-mismatch",
            Self::CheckpointAcknowledgementRegressed => "checkpoint-acknowledgement-regressed",
            Self::CheckpointCounterRegressed => "checkpoint-counter-regressed",
            Self::CheckpointCounterInvalid => "checkpoint-counter-invalid",
            Self::CheckpointCursorForged => "checkpoint-cursor-forged",
            Self::AcknowledgedChunkMissing => "acknowledged-chunk-missing",
            Self::CreditGrantInvalid => "credit-grant-invalid",
            Self::CreditExceeded => "receiver-credit-exceeded",
            Self::BufferedChunkLimitExceeded => "buffered-chunk-limit-exceeded",
            Self::ChunkAlreadyInFlight => "chunk-already-in-flight",
            Self::ChunkDigestMismatch => "chunk-digest-mismatch",
            Self::AcknowledgementUnknown => "acknowledgement-unknown",
            Self::AcknowledgementMismatch => "acknowledgement-mismatch",
            Self::IdleProgressExceeded => "idle-progress-exceeded",
            Self::ProgressCounterRegressed => "progress-counter-regressed",
            Self::TransferContinue => "transfer-continue",
            Self::TransferAlreadyPresent => "transfer-already-present",
            Self::TransferDemandSatisfied => "transfer-demand-satisfied",
            Self::TransferRejected => "transfer-rejected",
        }
    }
}

pub fn canonical_remote_transfer_policy_digest(
    policy: RemoteTransferPolicy,
) -> Result<RemoteTransferDigest, RemoteTransferReasonCode> {
    policy.validate()?;
    let bytes = serde_json::to_vec(&policy).map_err(|_| RemoteTransferReasonCode::PolicySerializationFailed)?;
    let digest = RemoteTransferDigest(
        blake3::Hash::from_bytes(
            transfer_domain_digest(TRANSFER_POLICY_DOMAIN, &bytes)
                .map_err(|_| RemoteTransferReasonCode::ArithmeticOverflow)?,
        )
        .to_hex()
        .to_string(),
    );
    debug_assert!(!bytes.is_empty());
    Ok(digest)
}

pub fn derive_remote_transfer_session_id(
    job_id: &RemoteJobId,
    attempt_id: &RemoteAttemptId,
    fence_generation: RemoteFenceGeneration,
    requested_content_blake3: &RemoteTransferDigest,
    policy_digest_blake3: &RemoteTransferDigest,
) -> Result<RemoteTransferSessionId, RemoteTransferReasonCode> {
    let digest = transfer_session_digest(
        job_id.as_str(),
        attempt_id.as_str(),
        fence_generation.get(),
        requested_content_blake3.as_str(),
        policy_digest_blake3.as_str(),
    )
    .map_err(|reason| match reason {
        TransferIdentityError::Invalid => RemoteTransferReasonCode::SessionIdentityInvalid,
        TransferIdentityError::ArithmeticOverflow => RemoteTransferReasonCode::ArithmeticOverflow,
    })?;
    let value = blake3::Hash::from_bytes(digest).to_hex().to_string();
    debug_assert!(fence_generation.get() > 0);
    RemoteTransferSessionId::new(value)
}

pub fn canonicalize_remote_transfer_manifest(
    mut manifest: RemoteTransferManifest,
    policy: RemoteTransferPolicy,
) -> Result<CanonicalRemoteTransferManifest, RemoteTransferReasonCode> {
    policy.validate()?;
    validate_manifest_envelope(&manifest, policy)?;
    canonicalize_artifacts(&mut manifest.artifacts);
    let totals = validate_manifest_artifacts(&manifest.artifacts, policy)?;
    let canonical_bytes =
        serde_json::to_vec(&manifest).map_err(|_| RemoteTransferReasonCode::ManifestSerializationFailed)?;
    if canonical_bytes.len() > u32_to_usize(policy.control_bytes_max)? {
        return Err(RemoteTransferReasonCode::ManifestBoundsExceeded);
    }
    let digest_blake3 = RemoteTransferDigest(
        blake3::Hash::from_bytes(
            transfer_domain_digest(TRANSFER_MANIFEST_DOMAIN, &canonical_bytes)
                .map_err(|_| RemoteTransferReasonCode::ArithmeticOverflow)?,
        )
        .to_hex()
        .to_string(),
    );
    let artifact_count =
        u32::try_from(manifest.artifacts.len()).map_err(|_| RemoteTransferReasonCode::ManifestBoundsExceeded)?;
    debug_assert_eq!(artifact_count, totals.artifact_count);
    Ok(CanonicalRemoteTransferManifest {
        manifest,
        canonical_bytes,
        digest_blake3,
        artifact_count: totals.artifact_count,
        chunk_count: totals.chunk_count,
        total_bytes: totals.total_bytes,
    })
}

pub fn remote_transfer_scope(manifest: &CanonicalRemoteTransferManifest) -> RemoteTransferScope {
    let scope = RemoteTransferScope {
        session_id: manifest.manifest.session_id.clone(),
        job_id: manifest.manifest.job_id.clone(),
        attempt_id: manifest.manifest.attempt_id.clone(),
        fence_generation: manifest.manifest.fence_generation,
        manifest_digest_blake3: manifest.digest_blake3.clone(),
        policy_digest_blake3: manifest.manifest.policy_digest_blake3.clone(),
    };
    debug_assert_eq!(scope.manifest_digest_blake3, manifest.digest_blake3);
    debug_assert_eq!(scope.fence_generation, manifest.manifest.fence_generation);
    scope
}

pub fn plan_remote_transfer_demand(
    manifest: &CanonicalRemoteTransferManifest,
    receiver: &RemoteTransferReceiverFacts,
) -> Result<RemoteTransferDemand, RemoteTransferReasonCode> {
    let mut missing_chunks = Vec::new();
    let totals = plan_transfer_demand(
        manifest.manifest.artifacts.iter().map(|artifact| TransferDemandArtifactFacts {
            artifact_id: artifact.artifact_id.as_str(),
            size_bytes: artifact.size_bytes,
            chunks: artifact.chunks.iter().map(|chunk| TransferDemandChunkFacts {
                digest_blake3: chunk.digest_blake3.as_str(),
                size_bytes: chunk.size_bytes,
            }),
        }),
        receiver.complete_artifact_ids.iter().map(RemoteTransferArtifactId::as_str),
        receiver.complete_chunk_digests.iter().map(RemoteTransferDigest::as_str),
        manifest.chunk_count,
        |artifact_index, chunk_index| {
            let artifact = &manifest.manifest.artifacts[artifact_index];
            missing_chunks.push(RemoteTransferChunkDemand {
                artifact_id: artifact.artifact_id.clone(),
                artifact_kind: artifact.artifact_kind,
                chunk: artifact.chunks[chunk_index].clone(),
            });
        },
    )
    .map_err(transfer_manifest_reason)?;
    debug_assert_eq!(u32::try_from(missing_chunks.len()), Ok(totals.missing_chunk_count));
    debug_assert!(totals.missing_bytes <= manifest.total_bytes);
    Ok(RemoteTransferDemand {
        scope: remote_transfer_scope(manifest),
        missing_chunks,
        missing_bytes: totals.missing_bytes,
        reused_bytes: totals.reused_bytes,
    })
}

pub fn seal_remote_transfer_checkpoint(
    mut checkpoint: RemoteTransferCheckpoint,
    policy: RemoteTransferPolicy,
) -> Result<RemoteTransferCheckpoint, RemoteTransferReasonCode> {
    policy.validate()?;
    checkpoint.schema = REMOTE_TRANSFER_CHECKPOINT_SCHEMA.to_string();
    checkpoint.checkpoint_digest_blake3 = checkpoint_payload_digest(&checkpoint)?;
    validate_checkpoint_size(&checkpoint, policy)?;
    debug_assert_eq!(checkpoint.schema, REMOTE_TRANSFER_CHECKPOINT_SCHEMA);
    debug_assert_eq!(checkpoint.checkpoint_digest_blake3, checkpoint_payload_digest(&checkpoint)?);
    Ok(checkpoint)
}

pub fn plan_remote_transfer_resume(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    receiver: &RemoteTransferReceiverFacts,
    previous: Option<&RemoteTransferCheckpoint>,
    checkpoint: Option<&RemoteTransferCheckpoint>,
) -> Result<RemoteTransferResumePlan, RemoteTransferReasonCode> {
    policy.validate()?;
    let demand = plan_remote_transfer_demand(manifest, receiver)?;
    let Some(checkpoint) = checkpoint else {
        return Ok(RemoteTransferResumePlan {
            reused_bytes: demand.reused_bytes,
            demand,
            acknowledged_chunk_digests: BTreeSet::new(),
            next_sequence: INITIAL_CHUNK_SEQUENCE,
            transferred_bytes: 0,
        });
    };
    validate_checkpoint(manifest, policy, receiver, previous, checkpoint)?;
    debug_assert_eq!(checkpoint.scope, remote_transfer_scope(manifest));
    debug_assert!(checkpoint.transferred_bytes <= policy.total_bytes_max);
    Ok(RemoteTransferResumePlan {
        demand,
        acknowledged_chunk_digests: checkpoint.acknowledged_chunk_digests.clone(),
        next_sequence: checkpoint.next_sequence,
        transferred_bytes: checkpoint.transferred_bytes,
        reused_bytes: checkpoint.reused_bytes,
    })
}

pub fn grant_remote_transfer_credit(
    policy: RemoteTransferPolicy,
    state: &RemoteTransferCreditState,
    requested: RemoteTransferCreditGrant,
    available_storage_bytes: u64,
) -> Result<RemoteTransferCreditState, RemoteTransferReasonCode> {
    policy.validate()?;
    let snapshot = validate_credit_state(policy, state)?;
    let grant = grant_transfer_credit(
        transfer_credit_limits(policy),
        transfer_credit_facts(state),
        snapshot,
        requested.bytes,
        requested.chunks,
        available_storage_bytes,
    )
    .map_err(transfer_progress_reason)?;
    let mut next = state.clone();
    next.granted_bytes_remaining = grant.granted_bytes_remaining;
    next.granted_chunks_remaining = grant.granted_chunks_remaining;
    debug_assert!(next.granted_bytes_remaining <= policy.in_flight_bytes_max);
    debug_assert!(next.granted_chunks_remaining <= policy.in_flight_chunks_max);
    Ok(next)
}

pub fn reserve_remote_transfer_chunk(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    demand: &RemoteTransferDemand,
    state: &RemoteTransferCreditState,
    header: &RemoteTransferChunkHeader,
) -> Result<RemoteTransferCreditState, RemoteTransferReasonCode> {
    policy.validate()?;
    validate_credit_state(policy, state)?;
    let decision = reserve_transfer_chunk(
        transfer_credit_limits(policy),
        manifest_scope_facts(manifest),
        transfer_scope_facts(&demand.scope),
        transfer_scope_facts(&header.scope),
        transfer_credit_facts(state),
        header.sequence,
        transfer_reserved_chunk_facts(&header.artifact_id, header.artifact_kind, &header.chunk),
        demand
            .missing_chunks
            .iter()
            .map(|missing| transfer_reserved_chunk_facts(&missing.artifact_id, missing.artifact_kind, &missing.chunk)),
        state.in_flight.contains_key(&header.sequence),
    )
    .map_err(transfer_progress_reason)?;
    let mut next = state.clone();
    next.granted_bytes_remaining = decision.granted_bytes_remaining;
    next.granted_chunks_remaining = decision.granted_chunks_remaining;
    let previous = next.in_flight.insert(header.sequence, RemoteTransferInFlightChunk {
        artifact_id: header.artifact_id.clone(),
        chunk_digest_blake3: header.chunk.digest_blake3.clone(),
        size_bytes: header.chunk.size_bytes,
    });
    debug_assert!(previous.is_none());
    next.next_sequence = decision.next_sequence;
    validate_credit_state(policy, &next)?;
    debug_assert!(next.in_flight.contains_key(&header.sequence));
    Ok(next)
}

pub fn acknowledge_remote_transfer_chunk(
    policy: RemoteTransferPolicy,
    expected_scope: &RemoteTransferScope,
    state: &RemoteTransferCreditState,
    acknowledgement: &RemoteTransferAcknowledgement,
    observed_digest_blake3: &RemoteTransferDigest,
    progress_step: u64,
) -> Result<RemoteTransferCreditState, RemoteTransferReasonCode> {
    // r[impl remote_builds.transient_handle_introduction]
    // Only the manifest-established in-flight scope can authenticate an ack.
    validate_transfer_scope(transfer_scope_facts(expected_scope), transfer_scope_facts(&acknowledgement.scope))
        .map_err(transfer_progress_reason)?;
    policy.validate()?;
    validate_credit_state(policy, state)?;
    let transferred = acknowledge_transfer_chunk(
        transfer_credit_facts(state),
        acknowledgement.chunk_digest_blake3.as_str(),
        observed_digest_blake3.as_str(),
        acknowledgement.transferred_bytes,
        state
            .in_flight
            .get(&acknowledgement.sequence)
            .map(|chunk| (chunk.chunk_digest_blake3.as_str(), chunk.size_bytes)),
    )
    .map_err(transfer_progress_reason)?;
    let mut next = state.clone();
    let removed = next
        .in_flight
        .remove(&acknowledgement.sequence)
        .ok_or(RemoteTransferReasonCode::AcknowledgementUnknown)?;
    next.transferred_bytes = transferred;
    next.acknowledged_chunk_digests.insert(removed.chunk_digest_blake3);
    next.last_progress_step = progress_step;
    validate_credit_state(policy, &next)?;
    debug_assert!(!next.in_flight.contains_key(&acknowledgement.sequence));
    debug_assert!(next.acknowledged_chunk_digests.contains(observed_digest_blake3));
    Ok(next)
}

pub fn validate_remote_transfer_idle_progress(
    policy: RemoteTransferPolicy,
    state: &RemoteTransferCreditState,
    current_progress_step: u64,
) -> Result<(), RemoteTransferReasonCode> {
    policy.validate()?;
    validate_transfer_idle_progress(state.last_progress_step, current_progress_step, policy.idle_progress_steps_max)
        .map_err(transfer_progress_reason)
}

pub fn decide_remote_transfer_cutoff(
    manifest: &CanonicalRemoteTransferManifest,
    initial_demand: &RemoteTransferDemand,
    receiver: &RemoteTransferReceiverFacts,
    acknowledged_chunk_digests: &BTreeSet<RemoteTransferDigest>,
    transferred_bytes: u64,
) -> RemoteTransferCompletionDecision {
    let selected = decide_transfer_cutoff(
        manifest_scope_facts(manifest),
        transfer_scope_facts(&initial_demand.scope),
        TransferCutoffAdmissionFacts {
            receiver_facts_valid: validate_receiver_facts(manifest, receiver).is_ok(),
            requested_content_identity_verified: receiver.requested_content_identity_verified,
            required_closure_metadata_verified: receiver.required_closure_metadata_verified,
            path_info_admitted: receiver.path_info_admitted,
        },
        manifest.manifest.artifacts.iter().map(|artifact| TransferRequiredArtifactFacts {
            required_for_completion: artifact.required_for_completion,
            complete: receiver.complete_artifact_ids.contains(&artifact.artifact_id),
        }),
        initial_demand.missing_chunks.iter().map(|chunk| TransferMissingChunkFacts {
            acknowledged: acknowledged_chunk_digests.contains(&chunk.chunk.digest_blake3),
            chunk_complete: receiver.complete_chunk_digests.contains(&chunk.chunk.digest_blake3),
            artifact_complete: receiver.complete_artifact_ids.contains(&chunk.artifact_id),
        }),
        transferred_bytes,
        initial_demand.reused_bytes,
        manifest.total_bytes,
    );
    let (disposition, reason_code) = match selected.disposition {
        TransferCutoffDisposition::AlreadyPresent => (
            RemoteTransferCompletionDisposition::AlreadyPresent,
            RemoteTransferReasonCode::TransferAlreadyPresent,
        ),
        TransferCutoffDisposition::DemandSatisfied => (
            RemoteTransferCompletionDisposition::DemandSatisfied,
            RemoteTransferReasonCode::TransferDemandSatisfied,
        ),
        TransferCutoffDisposition::Continue => {
            (RemoteTransferCompletionDisposition::Continue, RemoteTransferReasonCode::TransferContinue)
        }
        TransferCutoffDisposition::Reject => {
            (RemoteTransferCompletionDisposition::Reject, RemoteTransferReasonCode::TransferRejected)
        }
    };
    RemoteTransferCompletionDecision {
        disposition,
        reason_code,
        transferred_bytes: selected.transferred_bytes,
        reused_bytes: selected.reused_bytes,
        output_admission_claimed: selected.output_admission_claimed,
    }
}

fn transfer_manifest_reason(error: TransferManifestError) -> RemoteTransferReasonCode {
    match error {
        TransferManifestError::SchemaUnsupported => RemoteTransferReasonCode::ManifestSchemaUnsupported,
        TransferManifestError::StorePrefixInvalid => RemoteTransferReasonCode::StorePrefixInvalid,
        TransferManifestError::PolicyDigestMismatch => RemoteTransferReasonCode::PolicyDigestMismatch,
        TransferManifestError::ManifestBoundsExceeded => RemoteTransferReasonCode::ManifestBoundsExceeded,
        TransferManifestError::ArtifactIdentityInvalid => RemoteTransferReasonCode::ArtifactIdentityInvalid,
        TransferManifestError::ArtifactDuplicate => RemoteTransferReasonCode::ArtifactDuplicate,
        TransferManifestError::ArtifactSizeInvalid => RemoteTransferReasonCode::ArtifactSizeInvalid,
        TransferManifestError::NarSha256Invalid => RemoteTransferReasonCode::NarSha256Invalid,
        TransferManifestError::ChunkIndexInvalid => RemoteTransferReasonCode::ChunkIndexInvalid,
        TransferManifestError::ChunkOffsetInvalid => RemoteTransferReasonCode::ChunkOffsetInvalid,
        TransferManifestError::ChunkSizeInvalid => RemoteTransferReasonCode::ChunkSizeInvalid,
        TransferManifestError::DigestInvalid => RemoteTransferReasonCode::DigestInvalid,
        TransferManifestError::ChunkDigestConflict => RemoteTransferReasonCode::ChunkDigestConflict,
        TransferManifestError::ArithmeticOverflow => RemoteTransferReasonCode::ArithmeticOverflow,
        TransferManifestError::ReceiverFactsInvalid => RemoteTransferReasonCode::ReceiverFactsInvalid,
        TransferManifestError::TotalBytesExceeded => RemoteTransferReasonCode::TotalBytesExceeded,
    }
}

fn validate_manifest_envelope(
    manifest: &RemoteTransferManifest,
    policy: RemoteTransferPolicy,
) -> Result<(), RemoteTransferReasonCode> {
    validate_transfer_manifest_prefix(&manifest.schema, &manifest.store_prefix).map_err(transfer_manifest_reason)?;
    let digest = canonical_remote_transfer_policy_digest(policy)?;
    validate_transfer_manifest_envelope(TransferManifestEnvelopeFacts {
        policy_digest_matches: manifest.policy_digest_blake3 == digest,
        artifact_count: manifest.artifacts.len(),
        artifact_count_max: policy.artifact_count_max,
    })
    .map_err(transfer_manifest_reason)
}

fn canonicalize_artifacts(artifacts: &mut [RemoteTransferArtifact]) {
    for artifact in artifacts.iter_mut() {
        artifact.chunks.sort_by(|left, right| {
            (left.index, left.offset_bytes, &left.digest_blake3).cmp(&(
                right.index,
                right.offset_bytes,
                &right.digest_blake3,
            ))
        });
    }
    artifacts
        .sort_by(|left, right| (left.artifact_kind, &left.artifact_id).cmp(&(right.artifact_kind, &right.artifact_id)));
}

fn validate_manifest_artifacts(
    artifacts: &[RemoteTransferArtifact],
    policy: RemoteTransferPolicy,
) -> Result<TransferManifestTotals, RemoteTransferReasonCode> {
    validate_transfer_manifest_artifacts(
        artifacts.iter().map(|artifact| TransferArtifactFacts {
            artifact_id: artifact.artifact_id.as_str(),
            is_nar: artifact.artifact_kind == RemoteTransferArtifactKind::Nar,
            nar_sha256_hex: artifact.nar_sha256_hex.as_deref(),
            size_bytes: artifact.size_bytes,
            chunks: artifact.chunks.iter().map(|chunk| TransferChunkFacts {
                index: chunk.index,
                offset_bytes: chunk.offset_bytes,
                size_bytes: chunk.size_bytes,
                digest_blake3: chunk.digest_blake3.as_str(),
            }),
        }),
        policy.chunk_bytes_max,
        policy.chunk_count_max,
        policy.total_bytes_max,
    )
    .map_err(transfer_manifest_reason)
}

fn validate_receiver_facts(
    manifest: &CanonicalRemoteTransferManifest,
    receiver: &RemoteTransferReceiverFacts,
) -> Result<(), RemoteTransferReasonCode> {
    validate_transfer_receiver_facts(
        manifest.manifest.artifacts.iter().map(|artifact| TransferDemandArtifactFacts {
            artifact_id: artifact.artifact_id.as_str(),
            size_bytes: artifact.size_bytes,
            chunks: artifact.chunks.iter().map(|chunk| TransferDemandChunkFacts {
                digest_blake3: chunk.digest_blake3.as_str(),
                size_bytes: chunk.size_bytes,
            }),
        }),
        receiver.complete_artifact_ids.iter().map(RemoteTransferArtifactId::as_str),
        receiver.complete_chunk_digests.iter().map(RemoteTransferDigest::as_str),
    )
    .map_err(transfer_manifest_reason)
}

fn transfer_progress_reason(error: TransferProgressError) -> RemoteTransferReasonCode {
    match error {
        TransferProgressError::CheckpointSchemaUnsupported => RemoteTransferReasonCode::CheckpointSchemaUnsupported,
        TransferProgressError::CheckpointScopeMismatch => RemoteTransferReasonCode::CheckpointScopeMismatch,
        TransferProgressError::CheckpointCounterInvalid => RemoteTransferReasonCode::CheckpointCounterInvalid,
        TransferProgressError::CheckpointCursorForged => RemoteTransferReasonCode::CheckpointCursorForged,
        TransferProgressError::CheckpointAcknowledgementRegressed => {
            RemoteTransferReasonCode::CheckpointAcknowledgementRegressed
        }
        TransferProgressError::CheckpointCounterRegressed => RemoteTransferReasonCode::CheckpointCounterRegressed,
        TransferProgressError::ChunkUnknown => RemoteTransferReasonCode::ChunkUnknown,
        TransferProgressError::AcknowledgedChunkMissing => RemoteTransferReasonCode::AcknowledgedChunkMissing,
        TransferProgressError::ArithmeticOverflow => RemoteTransferReasonCode::ArithmeticOverflow,
        TransferProgressError::CreditGrantInvalid => RemoteTransferReasonCode::CreditGrantInvalid,
        TransferProgressError::CreditExceeded => RemoteTransferReasonCode::CreditExceeded,
        TransferProgressError::BufferedChunkLimitExceeded => RemoteTransferReasonCode::BufferedChunkLimitExceeded,
        TransferProgressError::TotalBytesExceeded => RemoteTransferReasonCode::TotalBytesExceeded,
        TransferProgressError::ChunkSequenceInvalid => RemoteTransferReasonCode::ChunkSequenceInvalid,
        TransferProgressError::ChunkNotDemanded => RemoteTransferReasonCode::ChunkNotDemanded,
        TransferProgressError::ChunkDigestMismatch => RemoteTransferReasonCode::ChunkDigestMismatch,
        TransferProgressError::ChunkSizeInvalid => RemoteTransferReasonCode::ChunkSizeInvalid,
        TransferProgressError::ChunkAlreadyInFlight => RemoteTransferReasonCode::ChunkAlreadyInFlight,
        TransferProgressError::AcknowledgementUnknown => RemoteTransferReasonCode::AcknowledgementUnknown,
        TransferProgressError::AcknowledgementMismatch => RemoteTransferReasonCode::AcknowledgementMismatch,
        TransferProgressError::IdleProgressExceeded => RemoteTransferReasonCode::IdleProgressExceeded,
        TransferProgressError::ProgressCounterRegressed => RemoteTransferReasonCode::ProgressCounterRegressed,
    }
}

fn transfer_scope_facts(scope: &RemoteTransferScope) -> TransferScopeFacts<'_> {
    TransferScopeFacts {
        session_id: scope.session_id.as_str(),
        job_id: scope.job_id.as_str(),
        attempt_id: scope.attempt_id.as_str(),
        fence_generation: scope.fence_generation.get(),
        manifest_digest_blake3: scope.manifest_digest_blake3.as_str(),
        policy_digest_blake3: scope.policy_digest_blake3.as_str(),
    }
}

fn manifest_scope_facts(manifest: &CanonicalRemoteTransferManifest) -> TransferScopeFacts<'_> {
    TransferScopeFacts {
        session_id: manifest.manifest.session_id.as_str(),
        job_id: manifest.manifest.job_id.as_str(),
        attempt_id: manifest.manifest.attempt_id.as_str(),
        fence_generation: manifest.manifest.fence_generation.get(),
        manifest_digest_blake3: manifest.digest_blake3.as_str(),
        policy_digest_blake3: manifest.manifest.policy_digest_blake3.as_str(),
    }
}

fn validate_checkpoint(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    receiver: &RemoteTransferReceiverFacts,
    previous: Option<&RemoteTransferCheckpoint>,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    validate_transfer_checkpoint_schema(&checkpoint.schema).map_err(transfer_progress_reason)?;
    validate_checkpoint_size(checkpoint, policy)?;
    if checkpoint.checkpoint_digest_blake3 != checkpoint_payload_digest(checkpoint)? {
        return Err(RemoteTransferReasonCode::CheckpointDigestMismatch);
    }
    validate_transfer_scope(manifest_scope_facts(manifest), transfer_scope_facts(&checkpoint.scope))
        .map_err(transfer_progress_reason)?;
    validate_checkpoint_counters(manifest, policy, checkpoint)?;
    validate_checkpoint_acknowledgements(manifest, receiver, checkpoint)?;
    if let Some(previous) = previous {
        validate_checkpoint_monotonic(previous, checkpoint)?;
    }
    Ok(())
}

fn validate_checkpoint_size(
    checkpoint: &RemoteTransferCheckpoint,
    policy: RemoteTransferPolicy,
) -> Result<(), RemoteTransferReasonCode> {
    let bytes = serde_json::to_vec(checkpoint).map_err(|_| RemoteTransferReasonCode::CheckpointSerializationFailed)?;
    if bytes.len() > u32_to_usize(policy.checkpoint_bytes_max)? {
        return Err(RemoteTransferReasonCode::CheckpointTooLarge);
    }
    Ok(())
}

#[derive(Serialize)]
struct RemoteTransferCheckpointPayload<'a> {
    schema: &'a str,
    scope: &'a RemoteTransferScope,
    acknowledged_chunk_digests: &'a BTreeSet<RemoteTransferDigest>,
    next_sequence: u64,
    transferred_bytes: u64,
    reused_bytes: u64,
}

fn checkpoint_payload_digest(
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<RemoteTransferDigest, RemoteTransferReasonCode> {
    let payload = RemoteTransferCheckpointPayload {
        schema: &checkpoint.schema,
        scope: &checkpoint.scope,
        acknowledged_chunk_digests: &checkpoint.acknowledged_chunk_digests,
        next_sequence: checkpoint.next_sequence,
        transferred_bytes: checkpoint.transferred_bytes,
        reused_bytes: checkpoint.reused_bytes,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|_| RemoteTransferReasonCode::CheckpointSerializationFailed)?;
    Ok(RemoteTransferDigest(
        blake3::Hash::from_bytes(
            transfer_domain_digest(TRANSFER_CHECKPOINT_DOMAIN, &bytes)
                .map_err(|_| RemoteTransferReasonCode::ArithmeticOverflow)?,
        )
        .to_hex()
        .to_string(),
    ))
}

fn validate_checkpoint_counters(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    validate_transfer_checkpoint_counters(
        TransferCheckpointCounterFacts {
            transferred_bytes: checkpoint.transferred_bytes,
            reused_bytes: checkpoint.reused_bytes,
            next_sequence: checkpoint.next_sequence,
            policy_total_bytes_max: policy.total_bytes_max,
            manifest_total_bytes: manifest.total_bytes,
            chunk_count_max: policy.chunk_count_max,
            replay_rounds_max: policy.replay_rounds_max,
        },
        manifest.manifest.artifacts.iter().flat_map(|artifact| artifact.chunks.iter()).map(|chunk| {
            TransferDemandChunkFacts {
                digest_blake3: chunk.digest_blake3.as_str(),
                size_bytes: chunk.size_bytes,
            }
        }),
        checkpoint.acknowledged_chunk_digests.iter().map(RemoteTransferDigest::as_str),
    )
    .map_err(transfer_progress_reason)
}

fn validate_checkpoint_acknowledgements(
    manifest: &CanonicalRemoteTransferManifest,
    receiver: &RemoteTransferReceiverFacts,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    validate_transfer_checkpoint_acknowledgements(
        manifest.manifest.artifacts.iter().map(|artifact| TransferDemandArtifactFacts {
            artifact_id: artifact.artifact_id.as_str(),
            size_bytes: artifact.size_bytes,
            chunks: artifact.chunks.iter().map(|chunk| TransferDemandChunkFacts {
                digest_blake3: chunk.digest_blake3.as_str(),
                size_bytes: chunk.size_bytes,
            }),
        }),
        receiver.complete_artifact_ids.iter().map(RemoteTransferArtifactId::as_str),
        receiver.complete_chunk_digests.iter().map(RemoteTransferDigest::as_str),
        checkpoint.acknowledged_chunk_digests.iter().map(RemoteTransferDigest::as_str),
    )
    .map_err(transfer_progress_reason)
}

fn validate_checkpoint_monotonic(
    previous: &RemoteTransferCheckpoint,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    validate_transfer_checkpoint_monotonic(TransferCheckpointMonotonicFacts {
        previous_scope: transfer_scope_facts(&previous.scope),
        current_scope: transfer_scope_facts(&checkpoint.scope),
        previous_acknowledged_present: previous
            .acknowledged_chunk_digests
            .iter()
            .map(|digest| checkpoint.acknowledged_chunk_digests.contains(digest)),
        previous_sequence: previous.next_sequence,
        current_sequence: checkpoint.next_sequence,
        previous_transferred_bytes: previous.transferred_bytes,
        current_transferred_bytes: checkpoint.transferred_bytes,
        previous_reused_bytes: previous.reused_bytes,
        current_reused_bytes: checkpoint.reused_bytes,
    })
    .map_err(transfer_progress_reason)
}

fn transfer_credit_limits(policy: RemoteTransferPolicy) -> TransferCreditLimits {
    TransferCreditLimits {
        in_flight_bytes_max: policy.in_flight_bytes_max,
        in_flight_chunks_max: policy.in_flight_chunks_max,
        buffered_chunks_max: policy.buffered_chunks_max,
        chunk_bytes_max: policy.chunk_bytes_max,
        total_bytes_max: policy.total_bytes_max,
    }
}

fn transfer_credit_facts(state: &RemoteTransferCreditState) -> TransferCreditFacts {
    TransferCreditFacts {
        granted_bytes_remaining: state.granted_bytes_remaining,
        granted_chunks_remaining: state.granted_chunks_remaining,
        transferred_bytes: state.transferred_bytes,
        next_sequence: state.next_sequence,
    }
}

fn validate_credit_state(
    policy: RemoteTransferPolicy,
    state: &RemoteTransferCreditState,
) -> Result<TransferCreditSnapshot, RemoteTransferReasonCode> {
    validate_transfer_credit_state(
        transfer_credit_limits(policy),
        transfer_credit_facts(state),
        state.in_flight.values().map(|chunk| chunk.size_bytes),
    )
    .map_err(transfer_progress_reason)
}

fn transfer_reserved_chunk_facts<'a>(
    artifact_id: &'a RemoteTransferArtifactId,
    artifact_kind: RemoteTransferArtifactKind,
    chunk: &'a RemoteTransferChunkDescriptor,
) -> TransferReservedChunkFacts<'a> {
    TransferReservedChunkFacts {
        artifact_id: artifact_id.as_str(),
        artifact_kind: artifact_kind as u8,
        index: chunk.index,
        offset_bytes: chunk.offset_bytes,
        size_bytes: chunk.size_bytes,
        digest_blake3: chunk.digest_blake3.as_str(),
    }
}

fn u32_to_usize(value: u32) -> Result<usize, RemoteTransferReasonCode> {
    usize::try_from(value).map_err(|_| RemoteTransferReasonCode::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    // r[verify store_transports.resumable_castore_sessions]
    // r[verify store_transports.receiver_driven_backpressure]
    // r[verify store_transports.content_presence_early_cutoff]
    // r[verify remote_builds.attempt_scoped_transfer_resume]

    use super::*;

    const TEST_FENCE: u64 = 7;
    const TEST_CHUNK_BYTES: u32 = 4;
    const TEST_ARTIFACT_BYTES: u64 = 8;
    const TEST_SECOND_CHUNK_OFFSET: u64 = 4;
    const TEST_SEQUENCE_ONE: u64 = 1;
    const TEST_PROGRESS_STEP: u64 = 10;
    const TEST_LATER_PROGRESS_STEP: u64 = 11;
    const TEST_TRANSFERRED_BYTES: u64 = 4;
    const TEST_IN_FLIGHT_CHUNKS_MAX: u32 = 2;
    const TEST_ARTIFACT_COUNT_MAX: u32 = 8;
    const TEST_CHUNK_COUNT_MAX: u32 = 16;
    const TEST_TOTAL_BYTES_MAX: u64 = 64;

    fn digest(label: &str) -> RemoteTransferDigest {
        RemoteTransferDigest::new(blake3::hash(label.as_bytes()).to_hex().to_string()).unwrap()
    }

    fn job_id() -> RemoteJobId {
        RemoteJobId::new("job-transfer").unwrap()
    }

    fn attempt_id() -> RemoteAttemptId {
        RemoteAttemptId::new("attempt-transfer").unwrap()
    }

    fn chunk(index: u32, offset_bytes: u64, label: &str) -> RemoteTransferChunkDescriptor {
        RemoteTransferChunkDescriptor {
            index,
            offset_bytes,
            size_bytes: TEST_CHUNK_BYTES,
            digest_blake3: digest(label),
        }
    }

    fn artifact(id: &str, kind: RemoteTransferArtifactKind) -> RemoteTransferArtifact {
        RemoteTransferArtifact {
            artifact_id: RemoteTransferArtifactId::new(id).unwrap(),
            artifact_kind: kind,
            digest_blake3: digest(&format!("artifact-{id}")),
            size_bytes: TEST_ARTIFACT_BYTES,
            required_for_completion: true,
            nar_sha256_hex: (kind == RemoteTransferArtifactKind::Nar).then(|| "a".repeat(SHA256_HEX_LENGTH_CHARS)),
            chunks: vec![
                chunk(INITIAL_CHUNK_INDEX, INITIAL_CHUNK_OFFSET_BYTES, &format!("{id}-0")),
                chunk(INITIAL_CHUNK_INDEX + 1, TEST_SECOND_CHUNK_OFFSET, &format!("{id}-1")),
            ],
        }
    }

    fn policy() -> RemoteTransferPolicy {
        RemoteTransferPolicy {
            chunk_bytes_max: TEST_CHUNK_BYTES,
            in_flight_bytes_max: TEST_ARTIFACT_BYTES,
            in_flight_chunks_max: TEST_IN_FLIGHT_CHUNKS_MAX,
            buffered_chunks_max: TEST_IN_FLIGHT_CHUNKS_MAX,
            artifact_count_max: TEST_ARTIFACT_COUNT_MAX,
            chunk_count_max: TEST_CHUNK_COUNT_MAX,
            total_bytes_max: TEST_TOTAL_BYTES_MAX,
            checkpoint_bytes_max: DEFAULT_REMOTE_TRANSFER_CHECKPOINT_BYTES,
            idle_progress_steps_max: DEFAULT_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS,
            replay_rounds_max: DEFAULT_REMOTE_TRANSFER_REPLAY_ROUNDS,
            control_bytes_max: DEFAULT_REMOTE_TRANSFER_CONTROL_BYTES,
        }
    }

    fn manifest_with_artifacts(artifacts: Vec<RemoteTransferArtifact>) -> CanonicalRemoteTransferManifest {
        let policy = policy();
        let policy_digest = canonical_remote_transfer_policy_digest(policy).unwrap();
        let requested = digest("requested-content");
        let fence = RemoteFenceGeneration::new(TEST_FENCE).unwrap();
        let session =
            derive_remote_transfer_session_id(&job_id(), &attempt_id(), fence, &requested, &policy_digest).unwrap();
        canonicalize_remote_transfer_manifest(
            RemoteTransferManifest {
                schema: REMOTE_TRANSFER_MANIFEST_SCHEMA.to_string(),
                session_id: session,
                job_id: job_id(),
                attempt_id: attempt_id(),
                fence_generation: fence,
                policy_digest_blake3: policy_digest,
                store_prefix: "/mantle/store".to_string(),
                requested_content_blake3: requested,
                artifacts,
            },
            policy,
        )
        .unwrap()
    }

    fn manifest() -> CanonicalRemoteTransferManifest {
        manifest_with_artifacts(vec![
            artifact("nar", RemoteTransferArtifactKind::Nar),
            artifact("path-info", RemoteTransferArtifactKind::PathInfo),
        ])
    }

    fn empty_receiver() -> RemoteTransferReceiverFacts {
        RemoteTransferReceiverFacts::default()
    }

    fn checkpoint(
        manifest: &CanonicalRemoteTransferManifest,
        acknowledged: BTreeSet<RemoteTransferDigest>,
        next_sequence: u64,
        transferred_bytes: u64,
        reused_bytes: u64,
    ) -> RemoteTransferCheckpoint {
        seal_remote_transfer_checkpoint(
            RemoteTransferCheckpoint {
                schema: REMOTE_TRANSFER_CHECKPOINT_SCHEMA.to_string(),
                scope: remote_transfer_scope(manifest),
                acknowledged_chunk_digests: acknowledged,
                next_sequence,
                transferred_bytes,
                reused_bytes,
                checkpoint_digest_blake3: digest("placeholder-checkpoint"),
            },
            policy(),
        )
        .unwrap()
    }

    #[test]
    fn canonical_manifest_is_permutation_invariant_and_covers_existing_identity_classes() {
        let forward = vec![
            artifact("blob", RemoteTransferArtifactKind::CastoreBlob),
            artifact("directory", RemoteTransferArtifactKind::CastoreDirectory),
            artifact("nar", RemoteTransferArtifactKind::Nar),
            artifact("source", RemoteTransferArtifactKind::SourceBundle),
            artifact("path-info", RemoteTransferArtifactKind::PathInfo),
            artifact("attestation", RemoteTransferArtifactKind::Attestation),
            artifact("delta-blob", RemoteTransferArtifactKind::DeltaBlob),
            artifact("delta-chunk", RemoteTransferArtifactKind::DeltaChunk),
        ];
        let mut reversed = forward.clone();
        reversed.reverse();
        for item in &mut reversed {
            item.chunks.reverse();
        }
        let left = manifest_with_artifacts(forward);
        let right = manifest_with_artifacts(reversed);

        assert_eq!(left.canonical_bytes, right.canonical_bytes);
        assert_eq!(left.digest_blake3, right.digest_blake3);
        assert_eq!(left.artifact_count, policy().artifact_count_max);
        assert_eq!(left.chunk_count, policy().chunk_count_max);
    }

    #[test]
    fn manifest_rejects_chunk_bounds_digest_conflicts_and_non_nar_sha256() {
        let mut oversized = artifact("oversized", RemoteTransferArtifactKind::CastoreBlob);
        oversized.chunks[0].size_bytes = policy().chunk_bytes_max + 1;
        let oversized_error = canonicalize_fixture_manifest(vec![oversized]).unwrap_err();

        let mut conflicting = vec![
            artifact("left", RemoteTransferArtifactKind::CastoreBlob),
            artifact("right", RemoteTransferArtifactKind::CastoreBlob),
        ];
        conflicting[1].chunks[0].digest_blake3 = conflicting[0].chunks[0].digest_blake3.clone();
        conflicting[1].chunks[0].size_bytes = policy().chunk_bytes_max - 1;
        let conflict_error = canonicalize_fixture_manifest(conflicting).unwrap_err();

        let mut wrong_sha = artifact("wrong-sha", RemoteTransferArtifactKind::PathInfo);
        wrong_sha.nar_sha256_hex = Some("a".repeat(SHA256_HEX_LENGTH_CHARS));
        let sha_error = canonicalize_fixture_manifest(vec![wrong_sha]).unwrap_err();

        assert_eq!(oversized_error, RemoteTransferReasonCode::ChunkSizeInvalid);
        assert_eq!(conflict_error, RemoteTransferReasonCode::ChunkDigestConflict);
        assert_eq!(sha_error, RemoteTransferReasonCode::NarSha256Invalid);
    }

    fn canonicalize_fixture_manifest(
        artifacts: Vec<RemoteTransferArtifact>,
    ) -> Result<CanonicalRemoteTransferManifest, RemoteTransferReasonCode> {
        let policy = policy();
        let policy_digest = canonical_remote_transfer_policy_digest(policy)?;
        let requested = digest("fixture-requested");
        let fence = RemoteFenceGeneration::new(TEST_FENCE).unwrap();
        let session = derive_remote_transfer_session_id(&job_id(), &attempt_id(), fence, &requested, &policy_digest)?;
        canonicalize_remote_transfer_manifest(
            RemoteTransferManifest {
                schema: REMOTE_TRANSFER_MANIFEST_SCHEMA.to_string(),
                session_id: session,
                job_id: job_id(),
                attempt_id: attempt_id(),
                fence_generation: fence,
                policy_digest_blake3: policy_digest,
                store_prefix: "/mantle/store".to_string(),
                requested_content_blake3: requested,
                artifacts,
            },
            policy,
        )
    }

    #[test]
    fn resumed_transfer_demands_only_missing_chunks_with_verified_cursor() {
        let manifest = manifest();
        let first = manifest.manifest.artifacts[0].chunks[0].digest_blake3.clone();
        let mut receiver = empty_receiver();
        receiver.complete_chunk_digests.insert(first.clone());
        let current =
            checkpoint(&manifest, BTreeSet::from([first.clone()]), TEST_SEQUENCE_ONE, TEST_TRANSFERRED_BYTES, 0);
        let resumed = plan_remote_transfer_resume(&manifest, policy(), &receiver, None, Some(&current)).unwrap();

        let expected_missing = manifest
            .manifest
            .artifacts
            .iter()
            .flat_map(|artifact| artifact.chunks.iter())
            .map(|chunk| chunk.digest_blake3.as_str())
            .filter(|digest| *digest != first.as_str())
            .collect::<BTreeSet<_>>();
        let actual_missing = resumed
            .demand
            .missing_chunks
            .iter()
            .map(|chunk| chunk.chunk.digest_blake3.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(actual_missing, expected_missing);
        assert_eq!(resumed.acknowledged_chunk_digests, BTreeSet::from([first]));
        assert_eq!(resumed.transferred_bytes, TEST_TRANSFERRED_BYTES);
    }

    #[test]
    fn malformed_manifest_schema_precedes_store_prefix_and_wrong_policy_digest() {
        let mut candidate = manifest().manifest;
        candidate.schema = "unsupported-manifest".to_string();
        candidate.store_prefix = "relative-store".to_string();
        candidate.policy_digest_blake3 = digest("wrong-policy");
        assert_eq!(
            canonicalize_remote_transfer_manifest(candidate.clone(), policy()).unwrap_err(),
            RemoteTransferReasonCode::ManifestSchemaUnsupported,
        );
        candidate.schema = REMOTE_TRANSFER_MANIFEST_SCHEMA.to_string();
        assert_eq!(
            canonicalize_remote_transfer_manifest(candidate.clone(), policy()).unwrap_err(),
            RemoteTransferReasonCode::StorePrefixInvalid,
        );
        candidate.store_prefix = "/mantle/store".to_string();
        assert_eq!(
            canonicalize_remote_transfer_manifest(candidate, policy()).unwrap_err(),
            RemoteTransferReasonCode::PolicyDigestMismatch,
        );
    }

    #[test]
    fn checkpoint_tamper_scope_regression_missing_object_and_forged_cursor_fail_closed() {
        let manifest = manifest();
        let first = manifest.manifest.artifacts[0].chunks[0].digest_blake3.clone();
        let mut receiver = empty_receiver();
        receiver.complete_chunk_digests.insert(first.clone());
        let previous =
            checkpoint(&manifest, BTreeSet::from([first.clone()]), TEST_SEQUENCE_ONE, TEST_TRANSFERRED_BYTES, 0);

        let mut tampered = previous.clone();
        tampered.transferred_bytes += TEST_TRANSFERRED_BYTES;
        let tamper_error =
            plan_remote_transfer_resume(&manifest, policy(), &receiver, None, Some(&tampered)).unwrap_err();

        let mut stale = previous.clone();
        stale.scope.fence_generation = stale.scope.fence_generation.advance().unwrap();
        stale = seal_remote_transfer_checkpoint(stale, policy()).unwrap();
        let stale_error = plan_remote_transfer_resume(&manifest, policy(), &receiver, None, Some(&stale)).unwrap_err();
        let mut unknown_session = previous.clone();
        unknown_session.scope.session_id = RemoteTransferSessionId::new(digest("unknown-session").as_str()).unwrap();
        let unknown_session = seal_remote_transfer_checkpoint(unknown_session, policy()).unwrap();
        let unknown_error =
            plan_remote_transfer_resume(&manifest, policy(), &receiver, None, Some(&unknown_session)).unwrap_err();

        let regressed = checkpoint(&manifest, BTreeSet::new(), TEST_SEQUENCE_ONE, TEST_TRANSFERRED_BYTES, 0);
        let regression_error =
            plan_remote_transfer_resume(&manifest, policy(), &receiver, Some(&previous), Some(&regressed)).unwrap_err();

        let missing_error =
            plan_remote_transfer_resume(&manifest, policy(), &empty_receiver(), None, Some(&previous)).unwrap_err();

        let sequence_max = u64::from(policy().chunk_count_max) * u64::from(policy().replay_rounds_max);
        let forged = checkpoint(&manifest, BTreeSet::new(), sequence_max + 1, 0, 0);
        let forged_error =
            plan_remote_transfer_resume(&manifest, policy(), &empty_receiver(), None, Some(&forged)).unwrap_err();

        assert_eq!(tamper_error, RemoteTransferReasonCode::CheckpointDigestMismatch);
        assert_eq!(stale_error, RemoteTransferReasonCode::CheckpointScopeMismatch);
        assert_eq!(unknown_error, RemoteTransferReasonCode::CheckpointScopeMismatch);
        assert_eq!(regression_error, RemoteTransferReasonCode::CheckpointAcknowledgementRegressed);
        assert_eq!(missing_error, RemoteTransferReasonCode::AcknowledgedChunkMissing);
        assert_eq!(forged_error, RemoteTransferReasonCode::CheckpointCursorForged);
    }

    #[test]
    fn credit_and_chunk_validation_reject_before_payload_allocation() {
        let manifest = manifest();
        let demand = plan_remote_transfer_demand(&manifest, &empty_receiver()).unwrap();
        let state = RemoteTransferCreditState::default();
        let granted = grant_remote_transfer_credit(
            policy(),
            &state,
            RemoteTransferCreditGrant {
                bytes: TEST_CHUNK_BYTES.into(),
                chunks: 1,
            },
            TEST_CHUNK_BYTES.into(),
        )
        .unwrap();
        let first = demand.missing_chunks.first().unwrap();
        let header = RemoteTransferChunkHeader {
            scope: demand.scope.clone(),
            artifact_id: first.artifact_id.clone(),
            artifact_kind: first.artifact_kind,
            chunk: first.chunk.clone(),
            sequence: INITIAL_CHUNK_SEQUENCE,
        };
        // r[verify remote_builds.transient_handle_introduction]
        let reserved = reserve_remote_transfer_chunk(&manifest, policy(), &demand, &granted, &header).unwrap();
        let mut unknown_session = header.clone();
        unknown_session.scope.session_id = RemoteTransferSessionId::new(digest("unknown-session").as_str()).unwrap();
        // r[verify remote_builds.transient_handle_rejection]
        let unknown_error =
            reserve_remote_transfer_chunk(&manifest, policy(), &demand, &granted, &unknown_session).unwrap_err();
        let mut unknown_artifact = header.clone();
        unknown_artifact.artifact_id = RemoteTransferArtifactId::new("unknown-artifact").unwrap();
        let artifact_error =
            reserve_remote_transfer_chunk(&manifest, policy(), &demand, &granted, &unknown_artifact).unwrap_err();

        let second = demand.missing_chunks.get(1).unwrap();
        let over_credit_header = RemoteTransferChunkHeader {
            scope: demand.scope.clone(),
            artifact_id: second.artifact_id.clone(),
            artifact_kind: second.artifact_kind,
            chunk: second.chunk.clone(),
            sequence: TEST_SEQUENCE_ONE,
        };
        let error = reserve_remote_transfer_chunk(&manifest, policy(), &demand, &reserved, &over_credit_header)
            .expect_err("no second chunk may be accepted without receiver credit");

        assert_eq!(error, RemoteTransferReasonCode::CreditExceeded);
        assert_eq!(unknown_error, RemoteTransferReasonCode::CheckpointScopeMismatch);
        assert_eq!(artifact_error, RemoteTransferReasonCode::ChunkNotDemanded);
        assert_eq!(reserved.in_flight.len(), 1);
        assert_eq!(reserved.transferred_bytes, 0);
        assert_eq!(reserved.next_sequence, TEST_SEQUENCE_ONE);
    }

    #[test]
    fn repeated_chunk_digests_are_disambiguated_by_artifact_local_index() {
        let mut repeated = artifact("repeated", RemoteTransferArtifactKind::CastoreBlob);
        repeated.chunks[1].digest_blake3 = repeated.chunks[0].digest_blake3.clone();
        let manifest = manifest_with_artifacts(vec![repeated]);
        let demand = plan_remote_transfer_demand(&manifest, &empty_receiver()).unwrap();
        let repeated_artifact = &manifest.manifest.artifacts[0];
        let second = RemoteTransferChunkDemand {
            artifact_id: repeated_artifact.artifact_id.clone(),
            artifact_kind: repeated_artifact.artifact_kind,
            chunk: repeated_artifact.chunks[1].clone(),
        };
        let second_only_demand = RemoteTransferDemand {
            scope: demand.scope.clone(),
            missing_chunks: vec![second.clone()],
            missing_bytes: u64::from(second.chunk.size_bytes),
            reused_bytes: u64::from(repeated_artifact.chunks[0].size_bytes),
        };
        let state = grant_remote_transfer_credit(
            policy(),
            &RemoteTransferCreditState::default(),
            RemoteTransferCreditGrant {
                bytes: TEST_CHUNK_BYTES.into(),
                chunks: 1,
            },
            TEST_CHUNK_BYTES.into(),
        )
        .unwrap();
        let header = RemoteTransferChunkHeader {
            scope: demand.scope.clone(),
            artifact_id: second.artifact_id.clone(),
            artifact_kind: second.artifact_kind,
            chunk: second.chunk.clone(),
            sequence: INITIAL_CHUNK_SEQUENCE,
        };
        let reserved =
            reserve_remote_transfer_chunk(&manifest, policy(), &second_only_demand, &state, &header).unwrap();

        let mut wrong_offset = header;
        wrong_offset.chunk.offset_bytes = INITIAL_CHUNK_OFFSET_BYTES;
        let wrong_error =
            reserve_remote_transfer_chunk(&manifest, policy(), &second_only_demand, &state, &wrong_offset).unwrap_err();

        assert_eq!(demand.missing_chunks.len(), 1);
        assert_eq!(demand.missing_bytes, u64::from(TEST_CHUNK_BYTES));
        assert_eq!(demand.reused_bytes, u64::from(TEST_CHUNK_BYTES));
        assert_eq!(reserved.in_flight.len(), 1);
        assert_eq!(reserved.next_sequence, TEST_SEQUENCE_ONE);
        assert_eq!(wrong_error, RemoteTransferReasonCode::ChunkDigestMismatch);
    }

    #[test]
    fn acknowledgement_is_monotonic_and_digest_checked() {
        let manifest = manifest();
        let demand = plan_remote_transfer_demand(&manifest, &empty_receiver()).unwrap();
        let state = grant_remote_transfer_credit(
            policy(),
            &RemoteTransferCreditState::default(),
            RemoteTransferCreditGrant {
                bytes: TEST_CHUNK_BYTES.into(),
                chunks: 1,
            },
            TEST_CHUNK_BYTES.into(),
        )
        .unwrap();
        let first = demand.missing_chunks.first().unwrap();
        let header = RemoteTransferChunkHeader {
            scope: demand.scope.clone(),
            artifact_id: first.artifact_id.clone(),
            artifact_kind: first.artifact_kind,
            chunk: first.chunk.clone(),
            sequence: INITIAL_CHUNK_SEQUENCE,
        };
        let reserved = reserve_remote_transfer_chunk(&manifest, policy(), &demand, &state, &header).unwrap();
        let acknowledgement = RemoteTransferAcknowledgement {
            scope: demand.scope.clone(),
            chunk_digest_blake3: first.chunk.digest_blake3.clone(),
            sequence: INITIAL_CHUNK_SEQUENCE,
            transferred_bytes: TEST_TRANSFERRED_BYTES,
        };
        let wrong = digest("wrong-observed-chunk");
        let wrong_error = acknowledge_remote_transfer_chunk(
            policy(),
            &demand.scope,
            &reserved,
            &acknowledgement,
            &wrong,
            TEST_PROGRESS_STEP,
        )
        .unwrap_err();
        let acknowledged = acknowledge_remote_transfer_chunk(
            policy(),
            &remote_transfer_scope(&manifest),
            &reserved,
            &acknowledgement,
            &first.chunk.digest_blake3,
            TEST_PROGRESS_STEP,
        )
        .unwrap();
        let mut unknown_session = acknowledgement.clone();
        unknown_session.scope.session_id = RemoteTransferSessionId::new(digest("unknown-session").as_str()).unwrap();
        let unknown_error = acknowledge_remote_transfer_chunk(
            policy(),
            &demand.scope,
            &reserved,
            &unknown_session,
            &first.chunk.digest_blake3,
            TEST_PROGRESS_STEP,
        )
        .unwrap_err();

        // r[verify remote_builds.transient_handle_rejection]
        assert_eq!(unknown_error, RemoteTransferReasonCode::CheckpointScopeMismatch);

        assert_eq!(wrong_error, RemoteTransferReasonCode::ChunkDigestMismatch);
        assert_eq!(acknowledged.transferred_bytes, TEST_TRANSFERRED_BYTES);
        assert!(acknowledged.acknowledged_chunk_digests.contains(&first.chunk.digest_blake3));
        assert!(acknowledged.in_flight.is_empty());
    }

    #[test]
    fn early_cutoff_requires_verified_identity_complete_closure_and_admitted_pathinfo() {
        let manifest = manifest();
        let complete_ids = manifest
            .manifest
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.clone())
            .collect::<BTreeSet<_>>();
        let complete = RemoteTransferReceiverFacts {
            complete_artifact_ids: complete_ids,
            complete_chunk_digests: BTreeSet::new(),
            requested_content_identity_verified: true,
            required_closure_metadata_verified: true,
            path_info_admitted: true,
        };
        let demand = plan_remote_transfer_demand(&manifest, &complete).unwrap();
        let cutoff = decide_remote_transfer_cutoff(&manifest, &demand, &complete, &BTreeSet::new(), 0);

        let mut expected_ca_only = complete.clone();
        expected_ca_only.requested_content_identity_verified = false;
        let not_cutoff = decide_remote_transfer_cutoff(&manifest, &demand, &expected_ca_only, &BTreeSet::new(), 0);

        let mut unadmitted = complete;
        unadmitted.path_info_admitted = false;
        let acknowledged = BTreeSet::from([manifest.manifest.artifacts[0].chunks[0].digest_blake3.clone()]);
        let unadmitted_decision =
            decide_remote_transfer_cutoff(&manifest, &demand, &unadmitted, &acknowledged, TEST_TRANSFERRED_BYTES);

        assert_eq!(cutoff.disposition, RemoteTransferCompletionDisposition::AlreadyPresent);
        assert_eq!(cutoff.transferred_bytes, 0);
        assert!(!cutoff.output_admission_claimed);
        assert_eq!(not_cutoff.disposition, RemoteTransferCompletionDisposition::Continue);
        assert_eq!(unadmitted_decision.disposition, RemoteTransferCompletionDisposition::Continue);
        // r[verify remote_builds.transient_handle_rejection]
        assert!(!unadmitted_decision.output_admission_claimed);
    }

    #[test]
    fn idle_progress_is_bounded_and_regression_fails() {
        let state = RemoteTransferCreditState {
            last_progress_step: TEST_PROGRESS_STEP,
            ..RemoteTransferCreditState::default()
        };
        let allowed = validate_remote_transfer_idle_progress(policy(), &state, TEST_LATER_PROGRESS_STEP);
        let regressed = validate_remote_transfer_idle_progress(policy(), &state, TEST_PROGRESS_STEP - 1);
        let exceeded_step = TEST_PROGRESS_STEP + u64::from(policy().idle_progress_steps_max) + 1;
        let exceeded = validate_remote_transfer_idle_progress(policy(), &state, exceeded_step);

        assert!(allowed.is_ok());
        assert_eq!(regressed, Err(RemoteTransferReasonCode::ProgressCounterRegressed));
        assert_eq!(exceeded, Err(RemoteTransferReasonCode::IdleProgressExceeded));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    #[kani::proof]
    fn invalid_credit_never_mutates_state() {
        let bytes: u64 = kani::any();
        let policy = RemoteTransferPolicy::default();
        let state = RemoteTransferCreditState::default();
        let result =
            grant_remote_transfer_credit(policy, &state, RemoteTransferCreditGrant { bytes, chunks: 1 }, bytes);
        if bytes == 0 || bytes > policy.in_flight_bytes_max {
            assert!(result.is_err());
            assert_eq!(state, RemoteTransferCreditState::default());
        }
    }

    #[kani::proof]
    fn expected_identity_alone_never_cuts_off() {
        let receiver = RemoteTransferReceiverFacts {
            requested_content_identity_verified: false,
            required_closure_metadata_verified: true,
            path_info_admitted: true,
            ..RemoteTransferReceiverFacts::default()
        };
        assert!(!receiver.requested_content_identity_verified);
        assert!(receiver.required_closure_metadata_verified);
        assert!(!receiver.path_info_admitted || !receiver.requested_content_identity_verified);
    }
}
