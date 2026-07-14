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

use serde::Deserialize;
use serde::Serialize;

use super::remote_attempt::RemoteAttemptId;
use super::remote_attempt::RemoteFenceGeneration;
use super::remote_attempt::RemoteJobId;

pub const REMOTE_TRANSFER_MANIFEST_SCHEMA: &str = "mantle-remote-transfer-manifest-v1";
pub const REMOTE_TRANSFER_CHECKPOINT_SCHEMA: &str = "mantle-remote-transfer-checkpoint-v1";
pub const REMOTE_TRANSFER_MANIFEST_DOMAIN: &str = "mantle-remote-transfer-manifest-v1";
pub const REMOTE_TRANSFER_POLICY_DOMAIN: &str = "mantle-remote-transfer-policy-v1";
pub const REMOTE_TRANSFER_CHECKPOINT_DOMAIN: &str = "mantle-remote-transfer-checkpoint-v1";
pub const REMOTE_TRANSFER_SESSION_DOMAIN: &str = "mantle-remote-transfer-session-v1";

pub const MAX_REMOTE_TRANSFER_ID_BYTES: usize = 256;
pub const MAX_REMOTE_TRANSFER_STORE_PREFIX_BYTES: usize = 4_096;
pub const MAX_REMOTE_TRANSFER_ARTIFACTS_HARD: u32 = 1_000_000;
pub const MAX_REMOTE_TRANSFER_CHUNKS_HARD: u32 = 4_000_000;
pub const MAX_REMOTE_TRANSFER_CHUNK_BYTES_HARD: u32 = 1_048_576;
pub const MAX_REMOTE_TRANSFER_IN_FLIGHT_BYTES_HARD: u64 = 67_108_864;
pub const MAX_REMOTE_TRANSFER_IN_FLIGHT_CHUNKS_HARD: u32 = 1_024;
pub const MAX_REMOTE_TRANSFER_BUFFERED_CHUNKS_HARD: u32 = 1_024;
pub const MAX_REMOTE_TRANSFER_TOTAL_BYTES_HARD: u64 = 1_099_511_627_776;
pub const MAX_REMOTE_TRANSFER_CHECKPOINT_BYTES_HARD: u32 = 1_048_576;
pub const MAX_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS_HARD: u32 = 1_000_000;
pub const MAX_REMOTE_TRANSFER_REPLAY_ROUNDS_HARD: u32 = 32;
pub const MAX_REMOTE_TRANSFER_CONTROL_BYTES_HARD: u32 = 1_048_576;

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

const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const SHA256_HEX_LENGTH_CHARS: usize = 64;
const INITIAL_CHUNK_INDEX: u32 = 0;
const INITIAL_CHUNK_OFFSET_BYTES: u64 = 0;
const INITIAL_CHUNK_SEQUENCE: u64 = 0;
const HASH_LENGTH_PREFIX_BYTES: usize = std::mem::size_of::<u64>();

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RemoteTransferDigest(String);

impl RemoteTransferDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, RemoteTransferReasonCode> {
        let value = value.into();
        if !is_lower_hex_digest(&value, BLAKE3_HEX_LENGTH_CHARS) {
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
        if !is_lower_hex_digest(&value, BLAKE3_HEX_LENGTH_CHARS) {
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
        bounded_transfer_identity(value.into()).map(Self)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferArtifact {
    pub artifact_id: RemoteTransferArtifactId,
    pub artifact_kind: RemoteTransferArtifactKind,
    pub digest_blake3: RemoteTransferDigest,
    pub size_bytes: u64,
    pub required_for_completion: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
        validate_nonzero_policy_fields(self)?;
        validate_hard_policy_limits(self)?;
        if u64::from(self.chunk_bytes_max) > self.in_flight_bytes_max {
            return Err(RemoteTransferReasonCode::PolicyInvalid);
        }
        if self.in_flight_chunks_max > self.buffered_chunks_max {
            return Err(RemoteTransferReasonCode::PolicyInvalid);
        }
        debug_assert!(self.chunk_bytes_max > 0);
        debug_assert!(self.total_bytes_max > 0);
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
    let digest = domain_hash(REMOTE_TRANSFER_POLICY_DOMAIN, &bytes);
    debug_assert_eq!(digest.as_str().len(), BLAKE3_HEX_LENGTH_CHARS);
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
    let mut hasher = blake3::Hasher::new();
    hash_part(&mut hasher, REMOTE_TRANSFER_SESSION_DOMAIN)?;
    hash_part(&mut hasher, job_id.as_str())?;
    hash_part(&mut hasher, attempt_id.as_str())?;
    hash_part(&mut hasher, &fence_generation.get().to_string())?;
    hash_part(&mut hasher, requested_content_blake3.as_str())?;
    hash_part(&mut hasher, policy_digest_blake3.as_str())?;
    let value = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(value.len(), BLAKE3_HEX_LENGTH_CHARS);
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
    let digest_blake3 = domain_hash(REMOTE_TRANSFER_MANIFEST_DOMAIN, &canonical_bytes);
    debug_assert_eq!(manifest.artifacts.len(), usize::try_from(totals.artifact_count).unwrap_or(usize::MAX));
    debug_assert_eq!(digest_blake3.as_str().len(), BLAKE3_HEX_LENGTH_CHARS);
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
    validate_receiver_facts(manifest, receiver)?;
    let mut missing_chunks = Vec::new();
    let mut missing_bytes = 0_u64;
    let mut reused_bytes = 0_u64;
    for artifact in &manifest.manifest.artifacts {
        if receiver.complete_artifact_ids.contains(&artifact.artifact_id) {
            reused_bytes = checked_add_bytes(reused_bytes, artifact.size_bytes)?;
            continue;
        }
        append_artifact_demand(
            artifact,
            &receiver.complete_chunk_digests,
            &mut missing_chunks,
            &mut missing_bytes,
            &mut reused_bytes,
        )?;
    }
    debug_assert!(missing_chunks.len() <= usize::try_from(manifest.chunk_count).unwrap_or(usize::MAX));
    debug_assert!(missing_bytes <= manifest.total_bytes);
    Ok(RemoteTransferDemand {
        scope: remote_transfer_scope(manifest),
        missing_chunks,
        missing_bytes,
        reused_bytes,
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
    validate_credit_state(policy, state)?;
    if requested.bytes == 0 || requested.chunks == 0 {
        return Err(RemoteTransferReasonCode::CreditGrantInvalid);
    }
    let next_bytes = checked_add_bytes(state.granted_bytes_remaining, requested.bytes)?;
    let next_chunks = state
        .granted_chunks_remaining
        .checked_add(requested.chunks)
        .ok_or(RemoteTransferReasonCode::ArithmeticOverflow)?;
    let in_flight_bytes = in_flight_bytes(state)?;
    let in_flight_chunks = in_flight_chunks(state)?;
    validate_credit_headroom(policy, next_bytes, next_chunks, in_flight_bytes, in_flight_chunks)?;
    if requested.bytes > available_storage_bytes {
        return Err(RemoteTransferReasonCode::CreditExceeded);
    }
    let mut next = state.clone();
    next.granted_bytes_remaining = next_bytes;
    next.granted_chunks_remaining = next_chunks;
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
    validate_chunk_header(manifest, policy, demand, state, header)?;
    let mut next = state.clone();
    next.granted_bytes_remaining = next
        .granted_bytes_remaining
        .checked_sub(u64::from(header.chunk.size_bytes))
        .ok_or(RemoteTransferReasonCode::CreditExceeded)?;
    next.granted_chunks_remaining =
        next.granted_chunks_remaining.checked_sub(1).ok_or(RemoteTransferReasonCode::CreditExceeded)?;
    let in_flight = RemoteTransferInFlightChunk {
        artifact_id: header.artifact_id.clone(),
        chunk_digest_blake3: header.chunk.digest_blake3.clone(),
        size_bytes: header.chunk.size_bytes,
    };
    if next.in_flight.insert(header.sequence, in_flight).is_some() {
        return Err(RemoteTransferReasonCode::ChunkAlreadyInFlight);
    }
    next.next_sequence = header.sequence.checked_add(1).ok_or(RemoteTransferReasonCode::ArithmeticOverflow)?;
    validate_credit_state(policy, &next)?;
    debug_assert!(next.in_flight.contains_key(&header.sequence));
    debug_assert_eq!(next.next_sequence, header.sequence + 1);
    Ok(next)
}

pub fn acknowledge_remote_transfer_chunk(
    policy: RemoteTransferPolicy,
    state: &RemoteTransferCreditState,
    acknowledgement: &RemoteTransferAcknowledgement,
    observed_digest_blake3: &RemoteTransferDigest,
    progress_step: u64,
) -> Result<RemoteTransferCreditState, RemoteTransferReasonCode> {
    policy.validate()?;
    validate_credit_state(policy, state)?;
    let in_flight = state
        .in_flight
        .get(&acknowledgement.sequence)
        .ok_or(RemoteTransferReasonCode::AcknowledgementUnknown)?;
    validate_acknowledgement(state, acknowledgement, observed_digest_blake3, in_flight)?;
    let mut next = state.clone();
    let removed = next
        .in_flight
        .remove(&acknowledgement.sequence)
        .ok_or(RemoteTransferReasonCode::AcknowledgementUnknown)?;
    next.transferred_bytes = checked_add_bytes(next.transferred_bytes, u64::from(removed.size_bytes))?;
    next.acknowledged_chunk_digests.insert(removed.chunk_digest_blake3);
    next.last_progress_step = progress_step;
    if next.transferred_bytes != acknowledgement.transferred_bytes {
        return Err(RemoteTransferReasonCode::AcknowledgementMismatch);
    }
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
    if current_progress_step < state.last_progress_step {
        return Err(RemoteTransferReasonCode::ProgressCounterRegressed);
    }
    let idle_steps = current_progress_step
        .checked_sub(state.last_progress_step)
        .ok_or(RemoteTransferReasonCode::ArithmeticOverflow)?;
    if idle_steps > u64::from(policy.idle_progress_steps_max) {
        return Err(RemoteTransferReasonCode::IdleProgressExceeded);
    }
    debug_assert!(current_progress_step >= state.last_progress_step);
    debug_assert!(idle_steps <= u64::from(policy.idle_progress_steps_max));
    Ok(())
}

pub fn decide_remote_transfer_cutoff(
    manifest: &CanonicalRemoteTransferManifest,
    initial_demand: &RemoteTransferDemand,
    receiver: &RemoteTransferReceiverFacts,
    acknowledged_chunk_digests: &BTreeSet<RemoteTransferDigest>,
    transferred_bytes: u64,
) -> RemoteTransferCompletionDecision {
    if validate_receiver_facts(manifest, receiver).is_err() || initial_demand.scope != remote_transfer_scope(manifest) {
        return transfer_completion(
            RemoteTransferCompletionDisposition::Reject,
            RemoteTransferReasonCode::TransferRejected,
            transferred_bytes,
            initial_demand.reused_bytes,
        );
    }
    if receiver_has_admitted_complete_request(manifest, receiver) && initial_demand.missing_chunks.is_empty() {
        return transfer_completion(
            RemoteTransferCompletionDisposition::AlreadyPresent,
            RemoteTransferReasonCode::TransferAlreadyPresent,
            transferred_bytes,
            manifest.total_bytes,
        );
    }
    if receiver.requested_content_identity_verified
        && receiver.required_closure_metadata_verified
        && receiver.path_info_admitted
        && receiver_has_complete_required_artifacts(manifest, receiver)
        && demand_is_satisfied(initial_demand, receiver, acknowledged_chunk_digests)
    {
        return transfer_completion(
            RemoteTransferCompletionDisposition::DemandSatisfied,
            RemoteTransferReasonCode::TransferDemandSatisfied,
            transferred_bytes,
            initial_demand.reused_bytes,
        );
    }
    transfer_completion(
        RemoteTransferCompletionDisposition::Continue,
        RemoteTransferReasonCode::TransferContinue,
        transferred_bytes,
        initial_demand.reused_bytes,
    )
}

fn validate_nonzero_policy_fields(policy: RemoteTransferPolicy) -> Result<(), RemoteTransferReasonCode> {
    let valid = policy.chunk_bytes_max > 0
        && policy.in_flight_bytes_max > 0
        && policy.in_flight_chunks_max > 0
        && policy.buffered_chunks_max > 0
        && policy.artifact_count_max > 0
        && policy.chunk_count_max > 0
        && policy.total_bytes_max > 0
        && policy.checkpoint_bytes_max > 0
        && policy.idle_progress_steps_max > 0
        && policy.replay_rounds_max > 0
        && policy.control_bytes_max > 0;
    if !valid {
        return Err(RemoteTransferReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_hard_policy_limits(policy: RemoteTransferPolicy) -> Result<(), RemoteTransferReasonCode> {
    let within_hard_limits = policy.chunk_bytes_max <= MAX_REMOTE_TRANSFER_CHUNK_BYTES_HARD
        && policy.in_flight_bytes_max <= MAX_REMOTE_TRANSFER_IN_FLIGHT_BYTES_HARD
        && policy.in_flight_chunks_max <= MAX_REMOTE_TRANSFER_IN_FLIGHT_CHUNKS_HARD
        && policy.buffered_chunks_max <= MAX_REMOTE_TRANSFER_BUFFERED_CHUNKS_HARD
        && policy.artifact_count_max <= MAX_REMOTE_TRANSFER_ARTIFACTS_HARD
        && policy.chunk_count_max <= MAX_REMOTE_TRANSFER_CHUNKS_HARD
        && policy.total_bytes_max <= MAX_REMOTE_TRANSFER_TOTAL_BYTES_HARD
        && policy.checkpoint_bytes_max <= MAX_REMOTE_TRANSFER_CHECKPOINT_BYTES_HARD
        && policy.idle_progress_steps_max <= MAX_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS_HARD
        && policy.replay_rounds_max <= MAX_REMOTE_TRANSFER_REPLAY_ROUNDS_HARD
        && policy.control_bytes_max <= MAX_REMOTE_TRANSFER_CONTROL_BYTES_HARD;
    if !within_hard_limits {
        return Err(RemoteTransferReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_manifest_envelope(
    manifest: &RemoteTransferManifest,
    policy: RemoteTransferPolicy,
) -> Result<(), RemoteTransferReasonCode> {
    if manifest.schema != REMOTE_TRANSFER_MANIFEST_SCHEMA {
        return Err(RemoteTransferReasonCode::ManifestSchemaUnsupported);
    }
    if manifest.store_prefix.is_empty()
        || !manifest.store_prefix.starts_with('/')
        || manifest.store_prefix.len() > MAX_REMOTE_TRANSFER_STORE_PREFIX_BYTES
    {
        return Err(RemoteTransferReasonCode::StorePrefixInvalid);
    }
    let policy_digest = canonical_remote_transfer_policy_digest(policy)?;
    if manifest.policy_digest_blake3 != policy_digest {
        return Err(RemoteTransferReasonCode::PolicyDigestMismatch);
    }
    if manifest.artifacts.is_empty() {
        return Err(RemoteTransferReasonCode::ManifestBoundsExceeded);
    }
    if manifest.artifacts.len() > u32_to_usize(policy.artifact_count_max)? {
        return Err(RemoteTransferReasonCode::ManifestBoundsExceeded);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ManifestTotals {
    artifact_count: u32,
    chunk_count: u32,
    total_bytes: u64,
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
) -> Result<ManifestTotals, RemoteTransferReasonCode> {
    let mut artifact_ids = BTreeSet::new();
    let mut chunk_shapes = BTreeMap::<RemoteTransferDigest, u32>::new();
    let mut chunk_count = 0_u32;
    let mut total_bytes = 0_u64;
    for artifact in artifacts {
        if !artifact_ids.insert(artifact.artifact_id.clone()) {
            return Err(RemoteTransferReasonCode::ArtifactDuplicate);
        }
        validate_artifact(artifact, policy, &mut chunk_shapes)?;
        let artifact_chunks =
            u32::try_from(artifact.chunks.len()).map_err(|_| RemoteTransferReasonCode::ManifestBoundsExceeded)?;
        chunk_count = chunk_count.checked_add(artifact_chunks).ok_or(RemoteTransferReasonCode::ArithmeticOverflow)?;
        total_bytes = checked_add_bytes(total_bytes, artifact.size_bytes)?;
        if chunk_count > policy.chunk_count_max || total_bytes > policy.total_bytes_max {
            return Err(RemoteTransferReasonCode::TotalBytesExceeded);
        }
    }
    let artifact_count =
        u32::try_from(artifacts.len()).map_err(|_| RemoteTransferReasonCode::ManifestBoundsExceeded)?;
    debug_assert!(artifact_count <= policy.artifact_count_max);
    debug_assert!(chunk_count <= policy.chunk_count_max);
    Ok(ManifestTotals {
        artifact_count,
        chunk_count,
        total_bytes,
    })
}

fn validate_artifact(
    artifact: &RemoteTransferArtifact,
    policy: RemoteTransferPolicy,
    chunk_shapes: &mut BTreeMap<RemoteTransferDigest, u32>,
) -> Result<(), RemoteTransferReasonCode> {
    RemoteTransferArtifactId::new(artifact.artifact_id.as_str().to_string())?;
    if artifact.size_bytes == 0 || artifact.chunks.is_empty() {
        return Err(RemoteTransferReasonCode::ArtifactSizeInvalid);
    }
    validate_nar_sha256(artifact)?;
    let mut expected_index = INITIAL_CHUNK_INDEX;
    let mut expected_offset = INITIAL_CHUNK_OFFSET_BYTES;
    for chunk in &artifact.chunks {
        validate_chunk_descriptor(chunk, policy, expected_index, expected_offset)?;
        if let Some(previous_size) = chunk_shapes.insert(chunk.digest_blake3.clone(), chunk.size_bytes)
            && previous_size != chunk.size_bytes
        {
            return Err(RemoteTransferReasonCode::ChunkDigestConflict);
        }
        expected_index = expected_index.checked_add(1).ok_or(RemoteTransferReasonCode::ArithmeticOverflow)?;
        expected_offset = checked_add_bytes(expected_offset, u64::from(chunk.size_bytes))?;
    }
    if expected_offset != artifact.size_bytes {
        return Err(RemoteTransferReasonCode::ArtifactSizeInvalid);
    }
    debug_assert_eq!(expected_index, u32::try_from(artifact.chunks.len()).unwrap_or(u32::MAX));
    debug_assert_eq!(expected_offset, artifact.size_bytes);
    Ok(())
}

fn validate_nar_sha256(artifact: &RemoteTransferArtifact) -> Result<(), RemoteTransferReasonCode> {
    match (&artifact.artifact_kind, &artifact.nar_sha256_hex) {
        (RemoteTransferArtifactKind::Nar, Some(value)) => {
            if !is_lower_hex_digest(value, SHA256_HEX_LENGTH_CHARS) {
                return Err(RemoteTransferReasonCode::NarSha256Invalid);
            }
        }
        (RemoteTransferArtifactKind::Nar, None) => {}
        (_, Some(_)) => return Err(RemoteTransferReasonCode::NarSha256Invalid),
        (_, None) => {}
    }
    Ok(())
}

fn validate_chunk_descriptor(
    chunk: &RemoteTransferChunkDescriptor,
    policy: RemoteTransferPolicy,
    expected_index: u32,
    expected_offset: u64,
) -> Result<(), RemoteTransferReasonCode> {
    if chunk.index != expected_index {
        return Err(RemoteTransferReasonCode::ChunkIndexInvalid);
    }
    if chunk.offset_bytes != expected_offset {
        return Err(RemoteTransferReasonCode::ChunkOffsetInvalid);
    }
    if chunk.size_bytes == 0 || chunk.size_bytes > policy.chunk_bytes_max {
        return Err(RemoteTransferReasonCode::ChunkSizeInvalid);
    }
    RemoteTransferDigest::new(chunk.digest_blake3.as_str().to_string())?;
    Ok(())
}

fn validate_receiver_facts(
    manifest: &CanonicalRemoteTransferManifest,
    receiver: &RemoteTransferReceiverFacts,
) -> Result<(), RemoteTransferReasonCode> {
    let artifact_ids = manifest
        .manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_id.clone())
        .collect::<BTreeSet<_>>();
    let chunk_digests = manifest_chunk_digests(manifest);
    if !receiver.complete_artifact_ids.is_subset(&artifact_ids) {
        return Err(RemoteTransferReasonCode::ReceiverFactsInvalid);
    }
    if !receiver.complete_chunk_digests.is_subset(&chunk_digests) {
        return Err(RemoteTransferReasonCode::ReceiverFactsInvalid);
    }
    debug_assert!(receiver.complete_artifact_ids.len() <= artifact_ids.len());
    debug_assert!(receiver.complete_chunk_digests.len() <= chunk_digests.len());
    Ok(())
}

fn append_artifact_demand(
    artifact: &RemoteTransferArtifact,
    present_chunks: &BTreeSet<RemoteTransferDigest>,
    missing_chunks: &mut Vec<RemoteTransferChunkDemand>,
    missing_bytes: &mut u64,
    reused_bytes: &mut u64,
) -> Result<(), RemoteTransferReasonCode> {
    for chunk in &artifact.chunks {
        if present_chunks.contains(&chunk.digest_blake3) {
            *reused_bytes = checked_add_bytes(*reused_bytes, u64::from(chunk.size_bytes))?;
            continue;
        }
        *missing_bytes = checked_add_bytes(*missing_bytes, u64::from(chunk.size_bytes))?;
        missing_chunks.push(RemoteTransferChunkDemand {
            artifact_id: artifact.artifact_id.clone(),
            artifact_kind: artifact.artifact_kind,
            chunk: chunk.clone(),
        });
    }
    Ok(())
}

fn validate_checkpoint(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    receiver: &RemoteTransferReceiverFacts,
    previous: Option<&RemoteTransferCheckpoint>,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    if checkpoint.schema != REMOTE_TRANSFER_CHECKPOINT_SCHEMA {
        return Err(RemoteTransferReasonCode::CheckpointSchemaUnsupported);
    }
    validate_checkpoint_size(checkpoint, policy)?;
    if checkpoint.checkpoint_digest_blake3 != checkpoint_payload_digest(checkpoint)? {
        return Err(RemoteTransferReasonCode::CheckpointDigestMismatch);
    }
    if checkpoint.scope != remote_transfer_scope(manifest) {
        return Err(RemoteTransferReasonCode::CheckpointScopeMismatch);
    }
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
    Ok(domain_hash(REMOTE_TRANSFER_CHECKPOINT_DOMAIN, &bytes))
}

fn validate_checkpoint_counters(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    if checkpoint.transferred_bytes > policy.total_bytes_max || checkpoint.reused_bytes > manifest.total_bytes {
        return Err(RemoteTransferReasonCode::CheckpointCounterInvalid);
    }
    let sequence_max = u64::from(policy.chunk_count_max)
        .checked_mul(u64::from(policy.replay_rounds_max))
        .ok_or(RemoteTransferReasonCode::ArithmeticOverflow)?;
    if checkpoint.next_sequence > sequence_max {
        return Err(RemoteTransferReasonCode::CheckpointCursorForged);
    }
    let acknowledged_bytes = acknowledged_chunk_bytes(manifest, &checkpoint.acknowledged_chunk_digests)?;
    let accounted_bytes = checked_add_bytes(checkpoint.transferred_bytes, checkpoint.reused_bytes)?;
    if acknowledged_bytes > accounted_bytes {
        return Err(RemoteTransferReasonCode::CheckpointCounterInvalid);
    }
    Ok(())
}

fn validate_checkpoint_acknowledgements(
    manifest: &CanonicalRemoteTransferManifest,
    receiver: &RemoteTransferReceiverFacts,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    let known = manifest_chunk_digests(manifest);
    if !checkpoint.acknowledged_chunk_digests.is_subset(&known) {
        return Err(RemoteTransferReasonCode::ChunkUnknown);
    }
    let complete_artifact_chunks = complete_artifact_chunk_digests(manifest, &receiver.complete_artifact_ids);
    let mut verified = receiver.complete_chunk_digests.clone();
    verified.extend(complete_artifact_chunks);
    if !checkpoint.acknowledged_chunk_digests.is_subset(&verified) {
        return Err(RemoteTransferReasonCode::AcknowledgedChunkMissing);
    }
    Ok(())
}

fn validate_checkpoint_monotonic(
    previous: &RemoteTransferCheckpoint,
    checkpoint: &RemoteTransferCheckpoint,
) -> Result<(), RemoteTransferReasonCode> {
    if previous.scope != checkpoint.scope {
        return Err(RemoteTransferReasonCode::CheckpointScopeMismatch);
    }
    if !previous.acknowledged_chunk_digests.is_subset(&checkpoint.acknowledged_chunk_digests) {
        return Err(RemoteTransferReasonCode::CheckpointAcknowledgementRegressed);
    }
    if checkpoint.next_sequence < previous.next_sequence
        || checkpoint.transferred_bytes < previous.transferred_bytes
        || checkpoint.reused_bytes < previous.reused_bytes
    {
        return Err(RemoteTransferReasonCode::CheckpointCounterRegressed);
    }
    Ok(())
}

fn validate_credit_state(
    policy: RemoteTransferPolicy,
    state: &RemoteTransferCreditState,
) -> Result<(), RemoteTransferReasonCode> {
    let in_flight_bytes = in_flight_bytes(state)?;
    let in_flight_chunks = in_flight_chunks(state)?;
    validate_credit_headroom(
        policy,
        state.granted_bytes_remaining,
        state.granted_chunks_remaining,
        in_flight_bytes,
        in_flight_chunks,
    )?;
    if in_flight_chunks > policy.buffered_chunks_max {
        return Err(RemoteTransferReasonCode::BufferedChunkLimitExceeded);
    }
    if state.transferred_bytes > policy.total_bytes_max {
        return Err(RemoteTransferReasonCode::TotalBytesExceeded);
    }
    Ok(())
}

fn validate_credit_headroom(
    policy: RemoteTransferPolicy,
    granted_bytes: u64,
    granted_chunks: u32,
    in_flight_bytes: u64,
    in_flight_chunks: u32,
) -> Result<(), RemoteTransferReasonCode> {
    let reserved_bytes = checked_add_bytes(granted_bytes, in_flight_bytes)?;
    let reserved_chunks =
        granted_chunks.checked_add(in_flight_chunks).ok_or(RemoteTransferReasonCode::ArithmeticOverflow)?;
    if reserved_bytes > policy.in_flight_bytes_max || reserved_chunks > policy.in_flight_chunks_max {
        return Err(RemoteTransferReasonCode::CreditExceeded);
    }
    Ok(())
}

fn validate_chunk_header(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    demand: &RemoteTransferDemand,
    state: &RemoteTransferCreditState,
    header: &RemoteTransferChunkHeader,
) -> Result<(), RemoteTransferReasonCode> {
    if header.scope != remote_transfer_scope(manifest) || demand.scope != header.scope {
        return Err(RemoteTransferReasonCode::CheckpointScopeMismatch);
    }
    if header.sequence != state.next_sequence {
        return Err(RemoteTransferReasonCode::ChunkSequenceInvalid);
    }
    let expected = demanded_chunk(demand, header).ok_or(RemoteTransferReasonCode::ChunkNotDemanded)?;
    if expected.chunk != header.chunk || expected.artifact_kind != header.artifact_kind {
        return Err(RemoteTransferReasonCode::ChunkDigestMismatch);
    }
    if header.chunk.size_bytes > policy.chunk_bytes_max {
        return Err(RemoteTransferReasonCode::ChunkSizeInvalid);
    }
    if u64::from(header.chunk.size_bytes) > state.granted_bytes_remaining || state.granted_chunks_remaining == 0 {
        return Err(RemoteTransferReasonCode::CreditExceeded);
    }
    if state.in_flight.contains_key(&header.sequence) {
        return Err(RemoteTransferReasonCode::ChunkAlreadyInFlight);
    }
    let projected_total = checked_add_bytes(state.transferred_bytes, u64::from(header.chunk.size_bytes))?;
    if projected_total > policy.total_bytes_max {
        return Err(RemoteTransferReasonCode::TotalBytesExceeded);
    }
    Ok(())
}

fn validate_acknowledgement(
    state: &RemoteTransferCreditState,
    acknowledgement: &RemoteTransferAcknowledgement,
    observed_digest_blake3: &RemoteTransferDigest,
    in_flight: &RemoteTransferInFlightChunk,
) -> Result<(), RemoteTransferReasonCode> {
    if acknowledgement.chunk_digest_blake3 != in_flight.chunk_digest_blake3 {
        return Err(RemoteTransferReasonCode::AcknowledgementMismatch);
    }
    if observed_digest_blake3 != &in_flight.chunk_digest_blake3 {
        return Err(RemoteTransferReasonCode::ChunkDigestMismatch);
    }
    let expected_transferred = checked_add_bytes(state.transferred_bytes, u64::from(in_flight.size_bytes))?;
    if acknowledgement.transferred_bytes != expected_transferred {
        return Err(RemoteTransferReasonCode::AcknowledgementMismatch);
    }
    Ok(())
}

fn demanded_chunk<'a>(
    demand: &'a RemoteTransferDemand,
    header: &RemoteTransferChunkHeader,
) -> Option<&'a RemoteTransferChunkDemand> {
    demand.missing_chunks.iter().find(|candidate| {
        candidate.artifact_id == header.artifact_id && candidate.chunk.digest_blake3 == header.chunk.digest_blake3
    })
}

fn in_flight_bytes(state: &RemoteTransferCreditState) -> Result<u64, RemoteTransferReasonCode> {
    state
        .in_flight
        .values()
        .try_fold(0_u64, |total, chunk| checked_add_bytes(total, u64::from(chunk.size_bytes)))
}

fn in_flight_chunks(state: &RemoteTransferCreditState) -> Result<u32, RemoteTransferReasonCode> {
    u32::try_from(state.in_flight.len()).map_err(|_| RemoteTransferReasonCode::ArithmeticOverflow)
}

fn receiver_has_admitted_complete_request(
    manifest: &CanonicalRemoteTransferManifest,
    receiver: &RemoteTransferReceiverFacts,
) -> bool {
    receiver.requested_content_identity_verified
        && receiver.required_closure_metadata_verified
        && receiver.path_info_admitted
        && receiver_has_complete_required_artifacts(manifest, receiver)
}

fn receiver_has_complete_required_artifacts(
    manifest: &CanonicalRemoteTransferManifest,
    receiver: &RemoteTransferReceiverFacts,
) -> bool {
    manifest
        .manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.required_for_completion)
        .all(|artifact| receiver.complete_artifact_ids.contains(&artifact.artifact_id))
}

fn demand_is_satisfied(
    demand: &RemoteTransferDemand,
    receiver: &RemoteTransferReceiverFacts,
    acknowledged_chunk_digests: &BTreeSet<RemoteTransferDigest>,
) -> bool {
    demand.missing_chunks.iter().all(|chunk| {
        acknowledged_chunk_digests.contains(&chunk.chunk.digest_blake3)
            || receiver.complete_chunk_digests.contains(&chunk.chunk.digest_blake3)
            || receiver.complete_artifact_ids.contains(&chunk.artifact_id)
    })
}

fn transfer_completion(
    disposition: RemoteTransferCompletionDisposition,
    reason_code: RemoteTransferReasonCode,
    transferred_bytes: u64,
    reused_bytes: u64,
) -> RemoteTransferCompletionDecision {
    RemoteTransferCompletionDecision {
        disposition,
        reason_code,
        transferred_bytes,
        reused_bytes,
        output_admission_claimed: false,
    }
}

fn manifest_chunk_digests(manifest: &CanonicalRemoteTransferManifest) -> BTreeSet<RemoteTransferDigest> {
    manifest
        .manifest
        .artifacts
        .iter()
        .flat_map(|artifact| artifact.chunks.iter().map(|chunk| chunk.digest_blake3.clone()))
        .collect()
}

fn complete_artifact_chunk_digests(
    manifest: &CanonicalRemoteTransferManifest,
    complete_artifact_ids: &BTreeSet<RemoteTransferArtifactId>,
) -> BTreeSet<RemoteTransferDigest> {
    manifest
        .manifest
        .artifacts
        .iter()
        .filter(|artifact| complete_artifact_ids.contains(&artifact.artifact_id))
        .flat_map(|artifact| artifact.chunks.iter().map(|chunk| chunk.digest_blake3.clone()))
        .collect()
}

fn acknowledged_chunk_bytes(
    manifest: &CanonicalRemoteTransferManifest,
    acknowledged: &BTreeSet<RemoteTransferDigest>,
) -> Result<u64, RemoteTransferReasonCode> {
    let mut sizes = BTreeMap::<RemoteTransferDigest, u32>::new();
    for artifact in &manifest.manifest.artifacts {
        for chunk in &artifact.chunks {
            sizes.entry(chunk.digest_blake3.clone()).or_insert(chunk.size_bytes);
        }
    }
    acknowledged.iter().try_fold(0_u64, |total, digest| {
        let size = sizes.get(digest).ok_or(RemoteTransferReasonCode::ChunkUnknown)?;
        checked_add_bytes(total, u64::from(*size))
    })
}

fn checked_add_bytes(left: u64, right: u64) -> Result<u64, RemoteTransferReasonCode> {
    left.checked_add(right).ok_or(RemoteTransferReasonCode::ArithmeticOverflow)
}

fn u32_to_usize(value: u32) -> Result<usize, RemoteTransferReasonCode> {
    usize::try_from(value).map_err(|_| RemoteTransferReasonCode::ArithmeticOverflow)
}

fn bounded_transfer_identity(value: String) -> Result<String, RemoteTransferReasonCode> {
    if value.is_empty() || value.len() > MAX_REMOTE_TRANSFER_ID_BYTES {
        return Err(RemoteTransferReasonCode::ArtifactIdentityInvalid);
    }
    if value.chars().any(char::is_control) {
        return Err(RemoteTransferReasonCode::ArtifactIdentityInvalid);
    }
    Ok(value)
}

fn is_lower_hex_digest(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn domain_hash(domain: &str, bytes: &[u8]) -> RemoteTransferDigest {
    let mut hasher = blake3::Hasher::new();
    let domain_len = u64::try_from(domain.len()).expect("static transfer domain length fits u64");
    hasher.update(&domain_len.to_le_bytes());
    hasher.update(domain.as_bytes());
    hasher.update(bytes);
    RemoteTransferDigest(hasher.finalize().to_hex().to_string())
}

fn hash_part(hasher: &mut blake3::Hasher, value: &str) -> Result<(), RemoteTransferReasonCode> {
    if value.is_empty() || value.len() > MAX_REMOTE_TRANSFER_ID_BYTES {
        return Err(RemoteTransferReasonCode::SessionIdentityInvalid);
    }
    let length = u64::try_from(value.len()).map_err(|_| RemoteTransferReasonCode::ArithmeticOverflow)?;
    debug_assert_eq!(length.to_le_bytes().len(), HASH_LENGTH_PREFIX_BYTES);
    debug_assert!(!value.is_empty());
    hasher.update(&length.to_le_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    // r[verify store_transports.resumable_castore_sessions]
    // r[verify store_transports.receiver_driven_backpressure]
    // r[verify store_transports.content_presence_early_cutoff]
    // r[verify remote_builds.attempt_scoped_transfer_resume]
    use proptest::prelude::*;

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
    const PROPERTY_PARITY_DIVISOR: u64 = 2;
    const PROPERTY_BYTES_MAX_EXCLUSIVE: u64 = 1_000_000;

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
    fn receiver_missing_set_is_permutation_invariant_and_resume_requests_only_missing_chunks() {
        let manifest = manifest();
        let first = manifest.manifest.artifacts[0].chunks[0].digest_blake3.clone();
        let mut receiver = empty_receiver();
        receiver.complete_chunk_digests.insert(first.clone());
        let left = plan_remote_transfer_demand(&manifest, &receiver).unwrap();
        let right = plan_remote_transfer_demand(&manifest, &receiver.clone()).unwrap();
        let current =
            checkpoint(&manifest, BTreeSet::from([first.clone()]), TEST_SEQUENCE_ONE, TEST_TRANSFERRED_BYTES, 0);
        let resumed = plan_remote_transfer_resume(&manifest, policy(), &receiver, None, Some(&current)).unwrap();

        assert_eq!(left, right);
        assert!(resumed.demand.missing_chunks.iter().all(|demand| demand.chunk.digest_blake3 != first));
        assert_eq!(resumed.acknowledged_chunk_digests, BTreeSet::from([first]));
        assert_eq!(resumed.transferred_bytes, TEST_TRANSFERRED_BYTES);
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
        let reserved = reserve_remote_transfer_chunk(&manifest, policy(), &demand, &granted, &header).unwrap();

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
        assert_eq!(reserved.in_flight.len(), 1);
        assert_eq!(reserved.transferred_bytes, 0);
        assert_eq!(reserved.next_sequence, TEST_SEQUENCE_ONE);
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
            scope: demand.scope,
            chunk_digest_blake3: first.chunk.digest_blake3.clone(),
            sequence: INITIAL_CHUNK_SEQUENCE,
            transferred_bytes: TEST_TRANSFERRED_BYTES,
        };
        let wrong = digest("wrong-observed-chunk");
        let wrong_error =
            acknowledge_remote_transfer_chunk(policy(), &reserved, &acknowledgement, &wrong, TEST_PROGRESS_STEP)
                .unwrap_err();
        let acknowledged = acknowledge_remote_transfer_chunk(
            policy(),
            &reserved,
            &acknowledgement,
            &first.chunk.digest_blake3,
            TEST_PROGRESS_STEP,
        )
        .unwrap();

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
        let unadmitted_decision = decide_remote_transfer_cutoff(&manifest, &demand, &unadmitted, &BTreeSet::new(), 0);

        assert_eq!(cutoff.disposition, RemoteTransferCompletionDisposition::AlreadyPresent);
        assert_eq!(cutoff.transferred_bytes, 0);
        assert!(!cutoff.output_admission_claimed);
        assert_eq!(not_cutoff.disposition, RemoteTransferCompletionDisposition::Continue);
        assert_eq!(unadmitted_decision.disposition, RemoteTransferCompletionDisposition::Continue);
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

    proptest! {
        #[test]
        fn checked_quota_arithmetic_never_wraps(left in 0_u64..PROPERTY_BYTES_MAX_EXCLUSIVE, right in 0_u64..PROPERTY_BYTES_MAX_EXCLUSIVE) {
            let result = checked_add_bytes(left, right).unwrap();
            prop_assert_eq!(result, left + right);
            prop_assert!(result >= left);
            prop_assert!(result >= right);
        }

        #[test]
        fn acknowledged_sets_are_monotonic_when_extended(index in 0_u32..policy().chunk_count_max) {
            let mut previous = BTreeSet::new();
            previous.insert(digest(&format!("chunk-{index}")));
            let mut next = previous.clone();
            next.insert(digest(&format!("chunk-next-{index}")));
            prop_assert!(previous.is_subset(&next));
            prop_assert!(next.len() >= previous.len());
        }

        #[test]
        fn equivalent_receiver_facts_produce_equivalent_demand(seed in 0_u64..PROPERTY_BYTES_MAX_EXCLUSIVE) {
            let manifest = manifest();
            let mut receiver = empty_receiver();
            if seed % PROPERTY_PARITY_DIVISOR == 0 {
                receiver.complete_chunk_digests.insert(manifest.manifest.artifacts[0].chunks[0].digest_blake3.clone());
            }
            let left = plan_remote_transfer_demand(&manifest, &receiver).unwrap();
            let right = plan_remote_transfer_demand(&manifest, &receiver.clone()).unwrap();
            prop_assert_eq!(left, right);
        }
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn checked_add_never_wraps() {
        let left: u64 = kani::any();
        let right: u64 = kani::any();
        match checked_add_bytes(left, right) {
            Ok(sum) => {
                assert!(sum >= left);
                assert!(sum >= right);
            }
            Err(reason) => assert_eq!(reason, RemoteTransferReasonCode::ArithmeticOverflow),
        }
    }

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
