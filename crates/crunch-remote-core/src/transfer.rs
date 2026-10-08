use alloc::collections::BTreeSet;
use alloc::string::String;

const DELTA_REUSE_PERCENT: u64 = 75;
const PERCENT_DENOMINATOR: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferCapabilities {
    pub delta: bool,
    pub full: bool,
    pub streaming: bool,
    pub simulate_delta_failure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    Delta,
    Full,
    Streaming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferFallback {
    DeltaTransferFailed,
    StreamingRuntimeNotBound,
    DeltaUnavailable,
}

impl TransferFallback {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeltaTransferFailed => "delta-transfer-failed",
            Self::StreamingRuntimeNotBound => "streaming-runtime-not-bound",
            Self::DeltaUnavailable => "delta-unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferBlocker {
    DeltaFailedAndFullUnavailable,
    StreamingRuntimeEvidenceRequired,
    NoCompatibleOutputMode,
    NoCompatibleProductionOutputMode,
    UploadByteLimitExceeded,
    UploadedInputSetMismatch,
}

impl TransferBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeltaFailedAndFullUnavailable => "delta-transfer-failed-and-full-unavailable",
            Self::StreamingRuntimeEvidenceRequired => "streaming-runtime-evidence-required",
            Self::NoCompatibleOutputMode => "no-compatible-output-transfer-mode",
            Self::NoCompatibleProductionOutputMode => "no-compatible-production-output-transfer-mode",
            Self::UploadByteLimitExceeded => "upload-byte-limit-exceeded",
            Self::UploadedInputSetMismatch => "uploaded-input-set-does-not-match-missing-set",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferSelection {
    pub mode: TransferMode,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub fallback: Option<TransferFallback>,
}

/// Fixture-compatible byte accounting. A negotiated streaming label alone is
/// not runtime transfer evidence and must never yield `Streaming` here.
// r[impl remote_builds.hexagonal_core]
pub const fn select_fixture_output_transfer(
    client: TransferCapabilities,
    builder: TransferCapabilities,
    output_size_bytes: u64,
) -> Result<TransferSelection, TransferBlocker> {
    let delta_available = client.delta && builder.delta;
    let simulated_failure = client.simulate_delta_failure || builder.simulate_delta_failure;
    if delta_available && !simulated_failure {
        let reused_bytes = output_size_bytes.saturating_mul(DELTA_REUSE_PERCENT) / PERCENT_DENOMINATOR;
        return Ok(TransferSelection {
            mode: TransferMode::Delta,
            transferred_bytes: output_size_bytes.saturating_sub(reused_bytes),
            reused_bytes,
            fallback: None,
        });
    }
    if delta_available && simulated_failure {
        if client.full && builder.full {
            return Ok(full_selection(output_size_bytes, Some(TransferFallback::DeltaTransferFailed)));
        }
        return Err(TransferBlocker::DeltaFailedAndFullUnavailable);
    }
    if client.full && builder.full {
        let fallback = if client.streaming && builder.streaming {
            Some(TransferFallback::StreamingRuntimeNotBound)
        } else {
            None
        };
        return Ok(full_selection(output_size_bytes, fallback));
    }
    if client.streaming && builder.streaming {
        return Err(TransferBlocker::StreamingRuntimeEvidenceRequired);
    }
    Err(TransferBlocker::NoCompatibleOutputMode)
}

const fn full_selection(output_size_bytes: u64, fallback: Option<TransferFallback>) -> TransferSelection {
    TransferSelection {
        mode: TransferMode::Full,
        transferred_bytes: output_size_bytes,
        reused_bytes: 0,
        fallback,
    }
}

/// Production negotiation selects a mode, not successful transferred bytes.
/// Only the bounded data-plane observation can authorize a streaming report.
// r[impl remote_builds.hexagonal_core]
pub const fn select_production_output_transfer(
    client: TransferCapabilities,
    builder: TransferCapabilities,
) -> Result<(TransferMode, Option<TransferFallback>), TransferBlocker> {
    if client.streaming && builder.streaming {
        if client.delta && builder.delta {
            return Ok((TransferMode::Full, Some(TransferFallback::DeltaUnavailable)));
        }
        return Ok((TransferMode::Streaming, None));
    }
    if client.full && builder.full {
        return Ok((TransferMode::Full, None));
    }
    Err(TransferBlocker::NoCompatibleProductionOutputMode)
}

/// Compare admitted missing/uploaded references after enforcing both the
/// ticket-supplied and global byte limits. Duplicate refs are intentionally
/// set-normalized, as in the currently accepted compatibility path.
// r[impl remote_builds.hexagonal_core]
pub fn admit_missing_uploads(
    missing_refs: &[String],
    uploaded_refs: &[String],
    uploaded_bytes: u64,
    max_upload_bytes: u64,
    global_max_upload_bytes: u64,
) -> Result<(), TransferBlocker> {
    if !upload_within_ticket_and_global_limit(uploaded_bytes, max_upload_bytes, global_max_upload_bytes) {
        return Err(TransferBlocker::UploadByteLimitExceeded);
    }
    let missing = missing_refs.iter().collect::<BTreeSet<_>>();
    let uploaded = uploaded_refs.iter().collect::<BTreeSet<_>>();
    if missing != uploaded {
        return Err(TransferBlocker::UploadedInputSetMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputUploadArtifactBlocker {
    CountExceeded,
    RequestIdMismatch,
    InputRefEmpty,
    DigestInvalid,
    PayloadSizeOverflow,
    SizeMismatch,
    DigestMismatch,
    Unexpected,
    Duplicate,
    TotalBytesOverflow,
    TotalBytesExceeded,
    Missing,
}

impl InputUploadArtifactBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CountExceeded => "input-upload-artifact-count-exceeded",
            Self::RequestIdMismatch => "input-upload-artifact-request-id-mismatch",
            Self::InputRefEmpty => "input-upload-artifact-ref-empty",
            Self::DigestInvalid => "input-upload-artifact-digest-invalid",
            Self::PayloadSizeOverflow => "remote-output-transfer-payload-size-overflow",
            Self::SizeMismatch => "input-upload-artifact-size-mismatch",
            Self::DigestMismatch => "input-upload-artifact-digest-mismatch",
            Self::Unexpected => "input-upload-artifact-unexpected",
            Self::Duplicate => "input-upload-artifact-duplicate",
            Self::TotalBytesOverflow => "input-upload-artifact-total-bytes-overflow",
            Self::TotalBytesExceeded => "input-upload-artifact-total-bytes-exceeded",
            Self::Missing => "input-upload-artifact-missing",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InputUploadArtifactFacts<'a> {
    pub request_id: &'a str,
    pub input_ref: &'a str,
    pub digest_blake3: &'a str,
    pub size_bytes: u64,
    pub payload: &'a [u8],
}

pub fn validate_input_upload_artifacts<'a, 'b, 'c>(
    request_id: &str,
    uploaded_refs: impl IntoIterator<Item = &'a str>,
    source_input_refs: impl IntoIterator<Item = &'b str>,
    artifacts: impl ExactSizeIterator<Item = InputUploadArtifactFacts<'c>>,
    artifact_count_max: usize,
    total_bytes_max: u64,
) -> Result<(), InputUploadArtifactBlocker> {
    if artifacts.len() > artifact_count_max {
        return Err(InputUploadArtifactBlocker::CountExceeded);
    }
    let source_refs = source_input_refs.into_iter().collect::<BTreeSet<_>>();
    let expected = uploaded_refs
        .into_iter()
        .filter(|input_ref| source_refs.contains(input_ref))
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut total_bytes = 0_u64;
    for artifact in artifacts {
        if artifact.request_id != request_id {
            return Err(InputUploadArtifactBlocker::RequestIdMismatch);
        }
        if artifact.input_ref.is_empty() {
            return Err(InputUploadArtifactBlocker::InputRefEmpty);
        }
        if !crate::output::is_blake3_hex_digest(artifact.digest_blake3) {
            return Err(InputUploadArtifactBlocker::DigestInvalid);
        }
        let payload_size =
            u64::try_from(artifact.payload.len()).map_err(|_| InputUploadArtifactBlocker::PayloadSizeOverflow)?;
        if payload_size != artifact.size_bytes {
            return Err(InputUploadArtifactBlocker::SizeMismatch);
        }
        if blake3::hash(artifact.payload).to_hex().as_str() != artifact.digest_blake3 {
            return Err(InputUploadArtifactBlocker::DigestMismatch);
        }
        if !expected.contains(artifact.input_ref) {
            return Err(InputUploadArtifactBlocker::Unexpected);
        }
        if !seen.insert(artifact.input_ref) {
            return Err(InputUploadArtifactBlocker::Duplicate);
        }
        total_bytes =
            total_bytes.checked_add(artifact.size_bytes).ok_or(InputUploadArtifactBlocker::TotalBytesOverflow)?;
        if total_bytes > total_bytes_max {
            return Err(InputUploadArtifactBlocker::TotalBytesExceeded);
        }
    }
    if expected.iter().any(|input_ref| !seen.contains(input_ref)) {
        return Err(InputUploadArtifactBlocker::Missing);
    }
    Ok(())
}

pub const TRANSFER_MANIFEST_DOMAIN: &str = "mantle-remote-transfer-manifest-v1";
pub const TRANSFER_POLICY_DOMAIN: &str = "mantle-remote-transfer-policy-v1";
pub const TRANSFER_CHECKPOINT_DOMAIN: &str = "mantle-remote-transfer-checkpoint-v1";
pub const TRANSFER_SESSION_DOMAIN: &str = "mantle-remote-transfer-session-v1";
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

pub fn is_transfer_artifact_id_valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_REMOTE_TRANSFER_ID_BYTES && !value.chars().any(char::is_control)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferIdentityError {
    Invalid,
    ArithmeticOverflow,
}

/// The std adapter serializes the existing v1 JSON before handing its exact
/// bytes to the no_std digest. Neither sorting nor serialization happens here.
pub fn transfer_domain_digest(domain: &str, canonical_bytes: &[u8]) -> Result<[u8; 32], TransferIdentityError> {
    let mut hasher = blake3::Hasher::new();
    let length = u64::try_from(domain.len()).map_err(|_| TransferIdentityError::ArithmeticOverflow)?;
    hasher.update(&length.to_le_bytes());
    hasher.update(domain.as_bytes());
    hasher.update(canonical_bytes);
    Ok(*hasher.finalize().as_bytes())
}

/// Preserve the v1 length-prefixed session preimage, including decimal fence.
pub fn transfer_session_digest(
    job_id: &str,
    attempt_id: &str,
    fence_generation: u64,
    requested_content_blake3: &str,
    policy_digest_blake3: &str,
) -> Result<[u8; 32], TransferIdentityError> {
    let mut hasher = blake3::Hasher::new();
    hash_transfer_part(&mut hasher, TRANSFER_SESSION_DOMAIN)?;
    hash_transfer_part(&mut hasher, job_id)?;
    hash_transfer_part(&mut hasher, attempt_id)?;
    let mut decimal = [0_u8; 20];
    let mut remaining = fence_generation;
    let mut start = decimal.len();
    loop {
        start -= 1;
        decimal[start] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }
    hash_transfer_bytes(&mut hasher, &decimal[start..])?;
    hash_transfer_part(&mut hasher, requested_content_blake3)?;
    hash_transfer_part(&mut hasher, policy_digest_blake3)?;
    Ok(*hasher.finalize().as_bytes())
}

fn hash_transfer_part(hasher: &mut blake3::Hasher, value: &str) -> Result<(), TransferIdentityError> {
    if value.is_empty() || value.len() > MAX_REMOTE_TRANSFER_ID_BYTES {
        return Err(TransferIdentityError::Invalid);
    }
    hash_transfer_bytes(hasher, value.as_bytes())
}

fn hash_transfer_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), TransferIdentityError> {
    let length = u64::try_from(bytes.len()).map_err(|_| TransferIdentityError::ArithmeticOverflow)?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferPolicyFacts {
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

/// All hard limits belong to core. The std adapter retains its public
/// policy's serde field order for exact existing canonical JSON bytes.
pub fn validate_transfer_policy(facts: TransferPolicyFacts) -> bool {
    facts.chunk_bytes_max > 0
        && facts.chunk_bytes_max <= MAX_REMOTE_TRANSFER_CHUNK_BYTES_HARD
        && facts.in_flight_bytes_max > 0
        && facts.in_flight_bytes_max <= MAX_REMOTE_TRANSFER_IN_FLIGHT_BYTES_HARD
        && facts.in_flight_chunks_max > 0
        && facts.in_flight_chunks_max <= MAX_REMOTE_TRANSFER_IN_FLIGHT_CHUNKS_HARD
        && facts.buffered_chunks_max > 0
        && facts.buffered_chunks_max <= MAX_REMOTE_TRANSFER_BUFFERED_CHUNKS_HARD
        && facts.artifact_count_max > 0
        && facts.artifact_count_max <= MAX_REMOTE_TRANSFER_ARTIFACTS_HARD
        && facts.chunk_count_max > 0
        && facts.chunk_count_max <= MAX_REMOTE_TRANSFER_CHUNKS_HARD
        && facts.total_bytes_max > 0
        && facts.total_bytes_max <= MAX_REMOTE_TRANSFER_TOTAL_BYTES_HARD
        && facts.checkpoint_bytes_max > 0
        && facts.checkpoint_bytes_max <= MAX_REMOTE_TRANSFER_CHECKPOINT_BYTES_HARD
        && facts.idle_progress_steps_max > 0
        && facts.idle_progress_steps_max <= MAX_REMOTE_TRANSFER_IDLE_PROGRESS_STEPS_HARD
        && facts.replay_rounds_max > 0
        && facts.replay_rounds_max <= MAX_REMOTE_TRANSFER_REPLAY_ROUNDS_HARD
        && facts.control_bytes_max > 0
        && facts.control_bytes_max <= MAX_REMOTE_TRANSFER_CONTROL_BYTES_HARD
        && u64::from(facts.chunk_bytes_max) <= facts.in_flight_bytes_max
        && facts.in_flight_chunks_max <= facts.buffered_chunks_max
}

pub fn upload_within_ticket_and_global_limit(bytes: u64, ticket_max: u64, global_max: u64) -> bool {
    bytes <= ticket_max && bytes <= global_max
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferManifestError {
    SchemaUnsupported,
    StorePrefixInvalid,
    PolicyDigestMismatch,
    ManifestBoundsExceeded,
    ArtifactIdentityInvalid,
    ArtifactDuplicate,
    ArtifactSizeInvalid,
    NarSha256Invalid,
    ChunkIndexInvalid,
    ChunkOffsetInvalid,
    ChunkSizeInvalid,
    DigestInvalid,
    ChunkDigestConflict,
    ArithmeticOverflow,
    ReceiverFactsInvalid,
    TotalBytesExceeded,
}

pub fn validate_transfer_manifest_prefix(schema: &str, store_prefix: &str) -> Result<(), TransferManifestError> {
    if schema != TRANSFER_MANIFEST_DOMAIN {
        return Err(TransferManifestError::SchemaUnsupported);
    }
    if store_prefix.is_empty()
        || !store_prefix.starts_with('/')
        || store_prefix.len() > MAX_REMOTE_TRANSFER_STORE_PREFIX_BYTES
    {
        return Err(TransferManifestError::StorePrefixInvalid);
    }
    Ok(())
}

pub struct TransferManifestEnvelopeFacts {
    pub policy_digest_matches: bool,
    pub artifact_count: usize,
    pub artifact_count_max: u32,
}

pub fn validate_transfer_manifest_envelope(facts: TransferManifestEnvelopeFacts) -> Result<(), TransferManifestError> {
    if !facts.policy_digest_matches {
        return Err(TransferManifestError::PolicyDigestMismatch);
    }
    let artifact_max =
        usize::try_from(facts.artifact_count_max).map_err(|_| TransferManifestError::ArithmeticOverflow)?;
    if facts.artifact_count == 0 || facts.artifact_count > artifact_max {
        return Err(TransferManifestError::ManifestBoundsExceeded);
    }
    Ok(())
}

pub struct TransferChunkFacts<'a> {
    pub index: u32,
    pub offset_bytes: u64,
    pub size_bytes: u32,
    pub digest_blake3: &'a str,
}

pub struct TransferArtifactFacts<'a, C> {
    pub artifact_id: &'a str,
    pub is_nar: bool,
    pub nar_sha256_hex: Option<&'a str>,
    pub size_bytes: u64,
    pub chunks: C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferManifestTotals {
    pub artifact_count: u32,
    pub chunk_count: u32,
    pub total_bytes: u64,
}

pub fn validate_transfer_manifest_artifacts<'a, A, C>(
    artifacts: A,
    chunk_bytes_max: u32,
    chunk_count_max: u32,
    total_bytes_max: u64,
) -> Result<TransferManifestTotals, TransferManifestError>
where
    A: ExactSizeIterator<Item = TransferArtifactFacts<'a, C>>,
    C: ExactSizeIterator<Item = TransferChunkFacts<'a>>,
{
    let artifact_count = u32::try_from(artifacts.len()).map_err(|_| TransferManifestError::ManifestBoundsExceeded)?;
    let mut artifact_ids = BTreeSet::new();
    let mut chunk_shapes = alloc::collections::BTreeMap::<&str, u32>::new();
    let mut chunk_count = 0_u32;
    let mut total_bytes = 0_u64;
    for artifact in artifacts {
        if !artifact_ids.insert(artifact.artifact_id) {
            return Err(TransferManifestError::ArtifactDuplicate);
        }
        if !is_transfer_artifact_id_valid(artifact.artifact_id) {
            return Err(TransferManifestError::ArtifactIdentityInvalid);
        }
        if artifact.size_bytes == 0 || artifact.chunks.len() == 0 {
            return Err(TransferManifestError::ArtifactSizeInvalid);
        }
        match (artifact.is_nar, artifact.nar_sha256_hex) {
            (true, Some(digest)) if !crate::output::is_blake3_hex_digest(digest) => {
                return Err(TransferManifestError::NarSha256Invalid);
            }
            (false, Some(_)) => return Err(TransferManifestError::NarSha256Invalid),
            _ => {}
        }
        let mut expected_index = 0_u32;
        let mut expected_offset_bytes = 0_u64;
        let artifact_chunk_count =
            u32::try_from(artifact.chunks.len()).map_err(|_| TransferManifestError::ManifestBoundsExceeded)?;
        for chunk in artifact.chunks {
            if chunk.index != expected_index {
                return Err(TransferManifestError::ChunkIndexInvalid);
            }
            if chunk.offset_bytes != expected_offset_bytes {
                return Err(TransferManifestError::ChunkOffsetInvalid);
            }
            if chunk.size_bytes == 0 || chunk.size_bytes > chunk_bytes_max {
                return Err(TransferManifestError::ChunkSizeInvalid);
            }
            if !crate::output::is_blake3_hex_digest(chunk.digest_blake3) {
                return Err(TransferManifestError::DigestInvalid);
            }
            if let Some(previous) = chunk_shapes.insert(chunk.digest_blake3, chunk.size_bytes)
                && previous != chunk.size_bytes
            {
                return Err(TransferManifestError::ChunkDigestConflict);
            }
            expected_index = expected_index.checked_add(1).ok_or(TransferManifestError::ArithmeticOverflow)?;
            expected_offset_bytes = expected_offset_bytes
                .checked_add(u64::from(chunk.size_bytes))
                .ok_or(TransferManifestError::ArithmeticOverflow)?;
        }
        if expected_offset_bytes != artifact.size_bytes {
            return Err(TransferManifestError::ArtifactSizeInvalid);
        }
        debug_assert_eq!(expected_index, artifact_chunk_count);
        chunk_count = chunk_count.checked_add(artifact_chunk_count).ok_or(TransferManifestError::ArithmeticOverflow)?;
        total_bytes = total_bytes.checked_add(artifact.size_bytes).ok_or(TransferManifestError::ArithmeticOverflow)?;
        if chunk_count > chunk_count_max || total_bytes > total_bytes_max {
            return Err(TransferManifestError::TotalBytesExceeded);
        }
    }
    Ok(TransferManifestTotals {
        artifact_count,
        chunk_count,
        total_bytes,
    })
}

pub struct TransferDemandChunkFacts<'a> {
    pub digest_blake3: &'a str,
    pub size_bytes: u32,
}

pub struct TransferDemandArtifactFacts<'a, C> {
    pub artifact_id: &'a str,
    pub size_bytes: u64,
    pub chunks: C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferDemandTotals {
    pub missing_bytes: u64,
    pub reused_bytes: u64,
    pub missing_chunk_count: u32,
}

/// Sender demand is derived from receiver-observed content only. The callback
/// materializes the existing wire descriptors without an intermediate index
/// vector or a second cloned manifest.
pub fn plan_transfer_demand<'a, A, C, R, S>(
    artifacts: A,
    receiver_artifacts: R,
    receiver_chunks: S,
    manifest_chunk_count: u32,
    mut on_missing: impl FnMut(usize, usize),
) -> Result<TransferDemandTotals, TransferManifestError>
where
    A: Clone + Iterator<Item = TransferDemandArtifactFacts<'a, C>>,
    C: Clone + Iterator<Item = TransferDemandChunkFacts<'a>>,
    R: Iterator<Item = &'a str>,
    S: Iterator<Item = &'a str>,
{
    let (complete_artifacts, complete_chunks) =
        checked_transfer_receiver_sets(artifacts.clone(), receiver_artifacts, receiver_chunks)?;
    let mut available_chunks = complete_chunks;
    let mut missing_bytes = 0_u64;
    let mut reused_bytes = 0_u64;
    let mut missing_chunk_count = 0_u32;
    for (artifact_index, artifact) in artifacts.enumerate() {
        if complete_artifacts.contains(artifact.artifact_id) {
            reused_bytes =
                reused_bytes.checked_add(artifact.size_bytes).ok_or(TransferManifestError::ArithmeticOverflow)?;
            continue;
        }
        for (chunk_index, chunk) in artifact.chunks.enumerate() {
            if !available_chunks.insert(chunk.digest_blake3) {
                reused_bytes = reused_bytes
                    .checked_add(u64::from(chunk.size_bytes))
                    .ok_or(TransferManifestError::ArithmeticOverflow)?;
                continue;
            }
            missing_bytes = missing_bytes
                .checked_add(u64::from(chunk.size_bytes))
                .ok_or(TransferManifestError::ArithmeticOverflow)?;
            missing_chunk_count =
                missing_chunk_count.checked_add(1).ok_or(TransferManifestError::ArithmeticOverflow)?;
            on_missing(artifact_index, chunk_index);
        }
    }
    debug_assert!(missing_chunk_count <= manifest_chunk_count);
    Ok(TransferDemandTotals {
        missing_bytes,
        reused_bytes,
        missing_chunk_count,
    })
}

pub fn validate_transfer_receiver_facts<'a, A, C, R, S>(
    artifacts: A,
    receiver_artifacts: R,
    receiver_chunks: S,
) -> Result<(), TransferManifestError>
where
    A: Clone + Iterator<Item = TransferDemandArtifactFacts<'a, C>>,
    C: Iterator<Item = TransferDemandChunkFacts<'a>>,
    R: Iterator<Item = &'a str>,
    S: Iterator<Item = &'a str>,
{
    checked_transfer_receiver_sets(artifacts, receiver_artifacts, receiver_chunks).map(|_| ())
}

fn checked_transfer_receiver_sets<'a, A, C, R, S>(
    artifacts: A,
    receiver_artifacts: R,
    receiver_chunks: S,
) -> Result<(BTreeSet<&'a str>, BTreeSet<&'a str>), TransferManifestError>
where
    A: Clone + Iterator<Item = TransferDemandArtifactFacts<'a, C>>,
    C: Iterator<Item = TransferDemandChunkFacts<'a>>,
    R: Iterator<Item = &'a str>,
    S: Iterator<Item = &'a str>,
{
    let known_artifacts = artifacts.clone().map(|artifact| artifact.artifact_id).collect::<BTreeSet<_>>();
    let known_chunks = artifacts
        .flat_map(|artifact| artifact.chunks.map(|chunk| chunk.digest_blake3))
        .collect::<BTreeSet<_>>();
    let complete_artifacts = receiver_artifacts.collect::<BTreeSet<_>>();
    let complete_chunks = receiver_chunks.collect::<BTreeSet<_>>();
    if !complete_artifacts.is_subset(&known_artifacts) || !complete_chunks.is_subset(&known_chunks) {
        return Err(TransferManifestError::ReceiverFactsInvalid);
    }
    Ok((complete_artifacts, complete_chunks))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferScopeFacts<'a> {
    pub session_id: &'a str,
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub fence_generation: u64,
    pub manifest_digest_blake3: &'a str,
    pub policy_digest_blake3: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferProgressError {
    CheckpointSchemaUnsupported,
    CheckpointScopeMismatch,
    CheckpointCounterInvalid,
    CheckpointCursorForged,
    CheckpointAcknowledgementRegressed,
    CheckpointCounterRegressed,
    CreditGrantInvalid,
    CreditExceeded,
    BufferedChunkLimitExceeded,
    TotalBytesExceeded,
    ChunkSequenceInvalid,
    ChunkNotDemanded,
    ChunkDigestMismatch,
    ChunkSizeInvalid,
    ChunkAlreadyInFlight,
    AcknowledgementUnknown,
    AcknowledgementMismatch,
    IdleProgressExceeded,
    ProgressCounterRegressed,
    ChunkUnknown,
    AcknowledgedChunkMissing,
    ArithmeticOverflow,
}

pub fn validate_transfer_checkpoint_schema(schema: &str) -> Result<(), TransferProgressError> {
    if schema != TRANSFER_CHECKPOINT_DOMAIN {
        return Err(TransferProgressError::CheckpointSchemaUnsupported);
    }
    Ok(())
}

pub fn validate_transfer_scope(
    expected: TransferScopeFacts<'_>,
    actual: TransferScopeFacts<'_>,
) -> Result<(), TransferProgressError> {
    if expected != actual {
        return Err(TransferProgressError::CheckpointScopeMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct TransferCheckpointCounterFacts {
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub next_sequence: u64,
    pub policy_total_bytes_max: u64,
    pub manifest_total_bytes: u64,
    pub chunk_count_max: u32,
    pub replay_rounds_max: u32,
}

pub fn validate_transfer_checkpoint_counters<'a>(
    facts: TransferCheckpointCounterFacts,
    manifest_chunks: impl Iterator<Item = TransferDemandChunkFacts<'a>>,
    acknowledged: impl Iterator<Item = &'a str>,
) -> Result<(), TransferProgressError> {
    if facts.transferred_bytes > facts.policy_total_bytes_max || facts.reused_bytes > facts.manifest_total_bytes {
        return Err(TransferProgressError::CheckpointCounterInvalid);
    }
    let sequence_max = u64::from(facts.chunk_count_max)
        .checked_mul(u64::from(facts.replay_rounds_max))
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    if facts.next_sequence > sequence_max {
        return Err(TransferProgressError::CheckpointCursorForged);
    }
    let mut chunk_sizes = alloc::collections::BTreeMap::new();
    for chunk in manifest_chunks {
        chunk_sizes.entry(chunk.digest_blake3).or_insert(chunk.size_bytes);
    }
    let mut acknowledged_bytes = 0_u64;
    for digest in acknowledged {
        let size = chunk_sizes.get(digest).ok_or(TransferProgressError::ChunkUnknown)?;
        acknowledged_bytes =
            acknowledged_bytes.checked_add(u64::from(*size)).ok_or(TransferProgressError::ArithmeticOverflow)?;
    }
    let accounted_bytes = facts
        .transferred_bytes
        .checked_add(facts.reused_bytes)
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    if acknowledged_bytes > accounted_bytes {
        return Err(TransferProgressError::CheckpointCounterInvalid);
    }
    Ok(())
}

pub fn validate_transfer_checkpoint_acknowledgements<'a, A, C>(
    artifacts: A,
    complete_artifacts: impl Iterator<Item = &'a str>,
    complete_chunks: impl Iterator<Item = &'a str>,
    acknowledged: impl Iterator<Item = &'a str> + Clone,
) -> Result<(), TransferProgressError>
where
    A: Clone + Iterator<Item = TransferDemandArtifactFacts<'a, C>>,
    C: Iterator<Item = TransferDemandChunkFacts<'a>>,
{
    let known = artifacts
        .clone()
        .flat_map(|artifact| artifact.chunks.map(|chunk| chunk.digest_blake3))
        .collect::<BTreeSet<_>>();
    if !acknowledged.clone().all(|digest| known.contains(digest)) {
        return Err(TransferProgressError::ChunkUnknown);
    }
    let complete_artifacts = complete_artifacts.collect::<BTreeSet<_>>();
    let mut verified = complete_chunks.collect::<BTreeSet<_>>();
    for artifact in artifacts {
        if complete_artifacts.contains(artifact.artifact_id) {
            verified.extend(artifact.chunks.map(|chunk| chunk.digest_blake3));
        }
    }
    if !acknowledged.into_iter().all(|digest| verified.contains(digest)) {
        return Err(TransferProgressError::AcknowledgedChunkMissing);
    }
    Ok(())
}

pub struct TransferCheckpointMonotonicFacts<'a, I> {
    pub previous_scope: TransferScopeFacts<'a>,
    pub current_scope: TransferScopeFacts<'a>,
    pub previous_acknowledged_present: I,
    pub previous_sequence: u64,
    pub current_sequence: u64,
    pub previous_transferred_bytes: u64,
    pub current_transferred_bytes: u64,
    pub previous_reused_bytes: u64,
    pub current_reused_bytes: u64,
}

pub fn validate_transfer_checkpoint_monotonic<I>(
    facts: TransferCheckpointMonotonicFacts<'_, I>,
) -> Result<(), TransferProgressError>
where I: Iterator<Item = bool> {
    validate_transfer_scope(facts.previous_scope, facts.current_scope)?;
    if !facts.previous_acknowledged_present.into_iter().all(|present| present) {
        return Err(TransferProgressError::CheckpointAcknowledgementRegressed);
    }
    if facts.current_sequence < facts.previous_sequence
        || facts.current_transferred_bytes < facts.previous_transferred_bytes
        || facts.current_reused_bytes < facts.previous_reused_bytes
    {
        return Err(TransferProgressError::CheckpointCounterRegressed);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct TransferCreditLimits {
    pub in_flight_bytes_max: u64,
    pub in_flight_chunks_max: u32,
    pub buffered_chunks_max: u32,
    pub chunk_bytes_max: u32,
    pub total_bytes_max: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TransferCreditFacts {
    pub granted_bytes_remaining: u64,
    pub granted_chunks_remaining: u32,
    pub transferred_bytes: u64,
    pub next_sequence: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TransferCreditSnapshot {
    pub in_flight_bytes: u64,
    pub in_flight_chunks: u32,
}

fn validate_transfer_credit_headroom(
    limits: TransferCreditLimits,
    granted_bytes: u64,
    granted_chunks: u32,
    snapshot: TransferCreditSnapshot,
) -> Result<(), TransferProgressError> {
    let reserved_bytes = granted_bytes
        .checked_add(snapshot.in_flight_bytes)
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    let reserved_chunks = granted_chunks
        .checked_add(snapshot.in_flight_chunks)
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    if reserved_bytes > limits.in_flight_bytes_max || reserved_chunks > limits.in_flight_chunks_max {
        return Err(TransferProgressError::CreditExceeded);
    }
    Ok(())
}

pub fn validate_transfer_credit_state(
    limits: TransferCreditLimits,
    state: TransferCreditFacts,
    in_flight_sizes: impl ExactSizeIterator<Item = u32>,
) -> Result<TransferCreditSnapshot, TransferProgressError> {
    let in_flight_chunks =
        u32::try_from(in_flight_sizes.len()).map_err(|_| TransferProgressError::ArithmeticOverflow)?;
    let mut in_flight_bytes = 0_u64;
    for size in in_flight_sizes {
        in_flight_bytes =
            in_flight_bytes.checked_add(u64::from(size)).ok_or(TransferProgressError::ArithmeticOverflow)?;
    }
    let snapshot = TransferCreditSnapshot {
        in_flight_bytes,
        in_flight_chunks,
    };
    validate_transfer_credit_headroom(limits, state.granted_bytes_remaining, state.granted_chunks_remaining, snapshot)?;
    if in_flight_chunks > limits.buffered_chunks_max {
        return Err(TransferProgressError::BufferedChunkLimitExceeded);
    }
    if state.transferred_bytes > limits.total_bytes_max {
        return Err(TransferProgressError::TotalBytesExceeded);
    }
    Ok(snapshot)
}

#[derive(Debug, Clone, Copy)]
pub struct TransferCreditGrantDecision {
    pub granted_bytes_remaining: u64,
    pub granted_chunks_remaining: u32,
}

pub fn grant_transfer_credit(
    limits: TransferCreditLimits,
    state: TransferCreditFacts,
    snapshot: TransferCreditSnapshot,
    requested_bytes: u64,
    requested_chunks: u32,
    available_storage_bytes: u64,
) -> Result<TransferCreditGrantDecision, TransferProgressError> {
    if requested_bytes == 0 || requested_chunks == 0 {
        return Err(TransferProgressError::CreditGrantInvalid);
    }
    let next_bytes = state
        .granted_bytes_remaining
        .checked_add(requested_bytes)
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    let next_chunks = state
        .granted_chunks_remaining
        .checked_add(requested_chunks)
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    validate_transfer_credit_headroom(limits, next_bytes, next_chunks, snapshot)?;
    if requested_bytes > available_storage_bytes {
        return Err(TransferProgressError::CreditExceeded);
    }
    Ok(TransferCreditGrantDecision {
        granted_bytes_remaining: next_bytes,
        granted_chunks_remaining: next_chunks,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct TransferReservedChunkFacts<'a> {
    pub artifact_id: &'a str,
    pub artifact_kind: u8,
    pub index: u32,
    pub offset_bytes: u64,
    pub size_bytes: u32,
    pub digest_blake3: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct TransferChunkReservation {
    pub granted_bytes_remaining: u64,
    pub granted_chunks_remaining: u32,
    pub next_sequence: u64,
}

pub fn reserve_transfer_chunk<'a>(
    limits: TransferCreditLimits,
    expected_scope: TransferScopeFacts<'_>,
    demand_scope: TransferScopeFacts<'_>,
    header_scope: TransferScopeFacts<'_>,
    state: TransferCreditFacts,
    header_sequence: u64,
    header: TransferReservedChunkFacts<'a>,
    demanded: impl Iterator<Item = TransferReservedChunkFacts<'a>>,
    already_in_flight: bool,
) -> Result<TransferChunkReservation, TransferProgressError> {
    validate_transfer_scope(expected_scope, header_scope)?;
    validate_transfer_scope(demand_scope, header_scope)?;
    if header_sequence != state.next_sequence {
        return Err(TransferProgressError::ChunkSequenceInvalid);
    }
    let expected = demanded
        .into_iter()
        .find(|chunk| chunk.artifact_id == header.artifact_id && chunk.index == header.index)
        .ok_or(TransferProgressError::ChunkNotDemanded)?;
    if expected.artifact_kind != header.artifact_kind
        || expected.offset_bytes != header.offset_bytes
        || expected.size_bytes != header.size_bytes
        || expected.digest_blake3 != header.digest_blake3
    {
        return Err(TransferProgressError::ChunkDigestMismatch);
    }
    if header.size_bytes > limits.chunk_bytes_max {
        return Err(TransferProgressError::ChunkSizeInvalid);
    }
    if u64::from(header.size_bytes) > state.granted_bytes_remaining || state.granted_chunks_remaining == 0 {
        return Err(TransferProgressError::CreditExceeded);
    }
    if already_in_flight {
        return Err(TransferProgressError::ChunkAlreadyInFlight);
    }
    let projected_total = state
        .transferred_bytes
        .checked_add(u64::from(header.size_bytes))
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    if projected_total > limits.total_bytes_max {
        return Err(TransferProgressError::TotalBytesExceeded);
    }
    Ok(TransferChunkReservation {
        granted_bytes_remaining: state.granted_bytes_remaining - u64::from(header.size_bytes),
        granted_chunks_remaining: state.granted_chunks_remaining - 1,
        next_sequence: header_sequence.checked_add(1).ok_or(TransferProgressError::ArithmeticOverflow)?,
    })
}

pub fn acknowledge_transfer_chunk(
    state: TransferCreditFacts,
    acknowledgement_digest: &str,
    observed_digest: &str,
    acknowledgement_transferred_bytes: u64,
    in_flight: Option<(&str, u32)>,
) -> Result<u64, TransferProgressError> {
    let (in_flight_digest, in_flight_size) = in_flight.ok_or(TransferProgressError::AcknowledgementUnknown)?;
    if acknowledgement_digest != in_flight_digest {
        return Err(TransferProgressError::AcknowledgementMismatch);
    }
    if observed_digest != in_flight_digest {
        return Err(TransferProgressError::ChunkDigestMismatch);
    }
    let transferred = state
        .transferred_bytes
        .checked_add(u64::from(in_flight_size))
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    if acknowledgement_transferred_bytes != transferred {
        return Err(TransferProgressError::AcknowledgementMismatch);
    }
    Ok(transferred)
}

pub fn validate_transfer_idle_progress(
    last_progress_step: u64,
    current_progress_step: u64,
    idle_progress_steps_max: u32,
) -> Result<(), TransferProgressError> {
    if current_progress_step < last_progress_step {
        return Err(TransferProgressError::ProgressCounterRegressed);
    }
    let idle_steps = current_progress_step
        .checked_sub(last_progress_step)
        .ok_or(TransferProgressError::ArithmeticOverflow)?;
    if idle_steps > u64::from(idle_progress_steps_max) {
        return Err(TransferProgressError::IdleProgressExceeded);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferCutoffDisposition {
    AlreadyPresent,
    DemandSatisfied,
    Continue,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferCutoffDecision {
    pub disposition: TransferCutoffDisposition,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub output_admission_claimed: bool,
}

pub struct TransferRequiredArtifactFacts {
    pub required_for_completion: bool,
    pub complete: bool,
}

pub struct TransferMissingChunkFacts {
    pub acknowledged: bool,
    pub chunk_complete: bool,
    pub artifact_complete: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TransferCutoffAdmissionFacts {
    pub receiver_facts_valid: bool,
    pub requested_content_identity_verified: bool,
    pub required_closure_metadata_verified: bool,
    pub path_info_admitted: bool,
}

/// Even a complete transfer never claims output admission. Only previously
/// verified receiver facts and demanded/acknowledged content permit cutoff.
pub fn decide_transfer_cutoff(
    expected_scope: TransferScopeFacts<'_>,
    initial_scope: TransferScopeFacts<'_>,
    admission: TransferCutoffAdmissionFacts,
    required_artifacts: impl Iterator<Item = TransferRequiredArtifactFacts>,
    missing_chunks: impl ExactSizeIterator<Item = TransferMissingChunkFacts>,
    transferred_bytes: u64,
    initially_reused_bytes: u64,
    manifest_total_bytes: u64,
) -> TransferCutoffDecision {
    let decision = |disposition, reused_bytes| TransferCutoffDecision {
        disposition,
        transferred_bytes,
        reused_bytes,
        output_admission_claimed: false,
    };
    if !admission.receiver_facts_valid || validate_transfer_scope(expected_scope, initial_scope).is_err() {
        return decision(TransferCutoffDisposition::Reject, initially_reused_bytes);
    }
    let content_admitted = admission.requested_content_identity_verified
        && admission.required_closure_metadata_verified
        && admission.path_info_admitted
        && required_artifacts
            .into_iter()
            .filter(|artifact| artifact.required_for_completion)
            .all(|artifact| artifact.complete);
    if content_admitted && missing_chunks.len() == 0 {
        return decision(TransferCutoffDisposition::AlreadyPresent, manifest_total_bytes);
    }
    if content_admitted
        && missing_chunks
            .into_iter()
            .all(|chunk| chunk.acknowledged || chunk.chunk_complete || chunk.artifact_complete)
    {
        return decision(TransferCutoffDisposition::DemandSatisfied, initially_reused_bytes);
    }
    decision(TransferCutoffDisposition::Continue, initially_reused_bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionOutputManifestBlocker {
    DirectionMismatch,
    DigestMismatch,
    AttemptMissing,
    AttemptStale,
    ContentBindingMismatch,
    ArtifactCountOverflow,
    ArtifactCountMismatch,
    FullManifestLabeledDelta,
    StreamingFallbackInconsistent,
    NarManifestMissing,
    NarManifestMismatch,
    PathinfoManifestMissing,
    PathinfoManifestMismatch,
    PathinfoPayloadSizeOverflow,
}

impl ProductionOutputManifestBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DirectionMismatch => "remote-output-transfer-direction-mismatch",
            Self::DigestMismatch => "manifest-identity-mismatch",
            Self::AttemptMissing => "remote-production-attempt-missing",
            Self::AttemptStale => "stale-report-rejected",
            Self::ContentBindingMismatch => "remote-output-transfer-manifest-binding-mismatch",
            Self::ArtifactCountOverflow => "remote-output-transfer-artifact-count-overflow",
            Self::ArtifactCountMismatch => "remote-output-transfer-manifest-artifact-count-mismatch",
            Self::FullManifestLabeledDelta => "remote-output-transfer-full-manifest-labeled-delta",
            Self::StreamingFallbackInconsistent => "remote-output-transfer-streaming-fallback-inconsistent",
            Self::NarManifestMissing => "remote-output-transfer-nar-manifest-missing",
            Self::NarManifestMismatch => "remote-output-transfer-nar-manifest-mismatch",
            Self::PathinfoManifestMissing => "remote-output-transfer-pathinfo-manifest-missing",
            Self::PathinfoManifestMismatch => "remote-output-transfer-pathinfo-manifest-mismatch",
            Self::PathinfoPayloadSizeOverflow => "remote-output-transfer-payload-size-overflow",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProductionManifestAttempt<'a> {
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub fence_generation: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ProductionOutputManifestFacts<'a> {
    pub canonical_digest_blake3: &'a str,
    pub claimed_digest_blake3: &'a str,
    pub expected_attempt: Option<ProductionManifestAttempt<'a>>,
    pub actual_attempt: ProductionManifestAttempt<'a>,
    pub expected_store_prefix: &'a str,
    pub actual_store_prefix: &'a str,
    pub expected_content_blake3: &'a str,
    pub actual_content_blake3: &'a str,
    pub output_count: usize,
    pub artifacts_per_output: usize,
    pub actual_artifact_count: usize,
}

/// The adapter must canonicalize the unchanged v1 wire JSON first. This
/// compares its admitted digest and fence before accepting any artifact bytes.
pub fn validate_production_output_manifest_direction(is_download: bool) -> Result<(), ProductionOutputManifestBlocker> {
    if is_download {
        Ok(())
    } else {
        Err(ProductionOutputManifestBlocker::DirectionMismatch)
    }
}

pub fn validate_production_output_manifest_binding(
    facts: ProductionOutputManifestFacts<'_>,
) -> Result<(), ProductionOutputManifestBlocker> {
    if facts.canonical_digest_blake3 != facts.claimed_digest_blake3 {
        return Err(ProductionOutputManifestBlocker::DigestMismatch);
    }
    let Some(expected) = facts.expected_attempt else {
        return Err(ProductionOutputManifestBlocker::AttemptMissing);
    };
    if expected.job_id != facts.actual_attempt.job_id
        || expected.attempt_id != facts.actual_attempt.attempt_id
        || expected.fence_generation != facts.actual_attempt.fence_generation
    {
        return Err(ProductionOutputManifestBlocker::AttemptStale);
    }
    if facts.expected_store_prefix != facts.actual_store_prefix
        || facts.expected_content_blake3 != facts.actual_content_blake3
    {
        return Err(ProductionOutputManifestBlocker::ContentBindingMismatch);
    }
    let expected_artifact_count = facts
        .output_count
        .checked_mul(facts.artifacts_per_output)
        .ok_or(ProductionOutputManifestBlocker::ArtifactCountOverflow)?;
    if facts.actual_artifact_count != expected_artifact_count {
        return Err(ProductionOutputManifestBlocker::ArtifactCountMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionInputManifestBlocker {
    ModeInvalid,
    DigestMismatch,
    AttemptMissing,
    BindingMismatch,
    ArtifactMismatch,
}

impl ProductionInputManifestBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModeInvalid => "remote-input-transfer-manifest-mode-invalid",
            Self::DigestMismatch => "manifest-identity-mismatch",
            Self::AttemptMissing => "remote-production-attempt-missing",
            Self::BindingMismatch => "remote-input-transfer-manifest-binding-mismatch",
            Self::ArtifactMismatch => "remote-input-transfer-manifest-artifact-mismatch",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProductionInputManifestFacts<'a> {
    pub canonical_digest_blake3: &'a str,
    pub claimed_digest_blake3: &'a str,
    pub expected_attempt: Option<ProductionManifestAttempt<'a>>,
    pub actual_attempt: ProductionManifestAttempt<'a>,
    pub expected_store_prefix: &'a str,
    pub actual_store_prefix: &'a str,
}

pub fn validate_production_input_manifest_mode(
    is_upload: bool,
    mode: TransferMode,
    has_fallback_reason: bool,
) -> Result<(), ProductionInputManifestBlocker> {
    if !is_upload || mode != TransferMode::Streaming || has_fallback_reason {
        return Err(ProductionInputManifestBlocker::ModeInvalid);
    }
    Ok(())
}

pub fn validate_production_input_manifest_scope(
    facts: ProductionInputManifestFacts<'_>,
) -> Result<(), ProductionInputManifestBlocker> {
    if facts.canonical_digest_blake3 != facts.claimed_digest_blake3 {
        return Err(ProductionInputManifestBlocker::DigestMismatch);
    }
    let Some(expected) = facts.expected_attempt else {
        return Err(ProductionInputManifestBlocker::AttemptMissing);
    };
    if facts.actual_attempt.job_id != expected.job_id
        || facts.actual_attempt.attempt_id != expected.attempt_id
        || facts.actual_attempt.fence_generation != expected.fence_generation
        || facts.actual_store_prefix != facts.expected_store_prefix
    {
        return Err(ProductionInputManifestBlocker::BindingMismatch);
    }
    Ok(())
}

pub fn validate_production_input_manifest_content_binding(
    expected_content_blake3: &str,
    actual_content_blake3: &str,
) -> Result<(), ProductionInputManifestBlocker> {
    if actual_content_blake3 != expected_content_blake3 {
        return Err(ProductionInputManifestBlocker::BindingMismatch);
    }
    Ok(())
}

pub fn validate_production_input_manifest_artifacts<'a, 'b>(
    expected: impl IntoIterator<Item = &'a str>,
    actual: impl IntoIterator<Item = (&'b str, bool)>,
) -> Result<(), ProductionInputManifestBlocker> {
    let mut remaining_ids = BTreeSet::new();
    for (id, is_nar) in actual {
        if !is_nar || !remaining_ids.insert(id) {
            return Err(ProductionInputManifestBlocker::ArtifactMismatch);
        }
    }
    for id in expected {
        if !remaining_ids.remove(id) {
            return Err(ProductionInputManifestBlocker::ArtifactMismatch);
        }
    }
    if !remaining_ids.is_empty() {
        return Err(ProductionInputManifestBlocker::ArtifactMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionTransferRuntimeBlocker {
    OutputAdmissionOverclaim,
    Incomplete,
    BuilderKeyEmpty,
    DeltaEvidenceMissing,
    StreamingFallbackInconsistent,
}

impl ProductionTransferRuntimeBlocker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OutputAdmissionOverclaim => "remote-streaming-runtime-admission-overclaim",
            Self::Incomplete => "remote-streaming-runtime-incomplete",
            Self::BuilderKeyEmpty => "remote-streaming-runtime-builder-key-empty",
            Self::DeltaEvidenceMissing => "remote-production-delta-runtime-evidence-missing",
            Self::StreamingFallbackInconsistent => "remote-production-streaming-fallback-inconsistent",
        }
    }
}

pub fn validate_production_transfer_runtime_report(
    output_admission_claimed: bool,
    disposition_complete: bool,
    verified_builder_key: &str,
    actual_mode: TransferMode,
    has_fallback_reason: bool,
) -> Result<(), ProductionTransferRuntimeBlocker> {
    if output_admission_claimed {
        return Err(ProductionTransferRuntimeBlocker::OutputAdmissionOverclaim);
    }
    if !disposition_complete {
        return Err(ProductionTransferRuntimeBlocker::Incomplete);
    }
    if verified_builder_key.is_empty() {
        return Err(ProductionTransferRuntimeBlocker::BuilderKeyEmpty);
    }
    if actual_mode == TransferMode::Delta {
        return Err(ProductionTransferRuntimeBlocker::DeltaEvidenceMissing);
    }
    if actual_mode == TransferMode::Streaming && has_fallback_reason {
        return Err(ProductionTransferRuntimeBlocker::StreamingFallbackInconsistent);
    }
    Ok(())
}

pub fn validate_production_output_manifest_mode(
    mode: TransferMode,
    has_fallback_reason: bool,
) -> Result<(), ProductionOutputManifestBlocker> {
    if mode == TransferMode::Delta {
        return Err(ProductionOutputManifestBlocker::FullManifestLabeledDelta);
    }
    if mode == TransferMode::Streaming && has_fallback_reason {
        return Err(ProductionOutputManifestBlocker::StreamingFallbackInconsistent);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionOutputArtifactKind {
    Nar,
    PathInfo,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct ProductionOutputArtifactFacts<'a> {
    pub kind: ProductionOutputArtifactKind,
    pub digest_blake3: &'a str,
    pub size_bytes: u64,
}

pub fn validate_production_output_nar_artifact(
    actual: Option<ProductionOutputArtifactFacts<'_>>,
    expected_digest: Option<&str>,
    expected_size: Option<u64>,
) -> Result<(), ProductionOutputManifestBlocker> {
    let Some(actual) = actual else {
        return Err(ProductionOutputManifestBlocker::NarManifestMissing);
    };
    if actual.kind != ProductionOutputArtifactKind::Nar
        || Some(actual.digest_blake3) != expected_digest
        || Some(actual.size_bytes) != expected_size
    {
        return Err(ProductionOutputManifestBlocker::NarManifestMismatch);
    }
    Ok(())
}

pub fn validate_production_output_pathinfo_artifact(
    actual: Option<ProductionOutputArtifactFacts<'_>>,
    pathinfo_payload: &[u8],
) -> Result<(), ProductionOutputManifestBlocker> {
    let Some(actual) = actual else {
        return Err(ProductionOutputManifestBlocker::PathinfoManifestMissing);
    };
    if actual.kind != ProductionOutputArtifactKind::PathInfo
        || actual.digest_blake3 != blake3::hash(pathinfo_payload).to_hex().as_str()
    {
        return Err(ProductionOutputManifestBlocker::PathinfoManifestMismatch);
    }
    let payload_size = u64::try_from(pathinfo_payload.len())
        .map_err(|_| ProductionOutputManifestBlocker::PathinfoPayloadSizeOverflow)?;
    if actual.size_bytes != payload_size {
        return Err(ProductionOutputManifestBlocker::PathinfoManifestMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const BOTH: TransferCapabilities = TransferCapabilities {
        delta: true,
        full: true,
        streaming: false,
        simulate_delta_failure: false,
    };

    #[test]
    fn production_runtime_receipt_requires_completed_evidence_without_admission_claim() {
        assert_eq!(
            validate_production_transfer_runtime_report(false, true, "builder-key", TransferMode::Streaming, false),
            Ok(())
        );
        assert_eq!(
            validate_production_transfer_runtime_report(true, false, "", TransferMode::Delta, true),
            Err(ProductionTransferRuntimeBlocker::OutputAdmissionOverclaim)
        );
        assert_eq!(
            validate_production_transfer_runtime_report(false, false, "builder-key", TransferMode::Streaming, false),
            Err(ProductionTransferRuntimeBlocker::Incomplete)
        );
        assert_eq!(
            validate_production_transfer_runtime_report(false, true, "builder-key", TransferMode::Delta, false),
            Err(ProductionTransferRuntimeBlocker::DeltaEvidenceMissing)
        );
        assert_eq!(
            validate_production_transfer_runtime_report(false, true, "builder-key", TransferMode::Streaming, true),
            Err(ProductionTransferRuntimeBlocker::StreamingFallbackInconsistent)
        );
    }

    #[test]
    fn uploaded_source_artifacts_require_exact_payload_and_unique_expected_refs() {
        let payload = b"payload";
        let digest = blake3::hash(payload).to_hex();
        let source = InputUploadArtifactFacts {
            request_id: "req",
            input_ref: "source",
            digest_blake3: digest.as_str(),
            size_bytes: payload.len() as u64,
            payload,
        };
        assert_eq!(
            validate_input_upload_artifacts("req", ["source", "other"], ["source"], [source].into_iter(), 2, 7),
            Ok(())
        );
        assert_eq!(
            validate_input_upload_artifacts(
                "req",
                ["source"],
                ["source"],
                [InputUploadArtifactFacts {
                    payload: b"PAYLOAD",
                    ..source
                }]
                .into_iter(),
                2,
                7,
            ),
            Err(InputUploadArtifactBlocker::DigestMismatch)
        );
        assert_eq!(
            validate_input_upload_artifacts("req", ["source"], ["source"], [source, source].into_iter(), 2, 14),
            Err(InputUploadArtifactBlocker::Duplicate)
        );
        assert_eq!(
            validate_input_upload_artifacts("req", ["source"], ["source"], [].into_iter(), 2, 7),
            Err(InputUploadArtifactBlocker::Missing)
        );
        assert_eq!(
            validate_input_upload_artifacts("req", ["source"], ["source"], [source].into_iter(), 0, 7),
            Err(InputUploadArtifactBlocker::CountExceeded)
        );
    }

    #[test]
    fn production_input_manifest_requires_current_fence_and_exact_nar_artifact_set() {
        let attempt = ProductionManifestAttempt {
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 4,
        };
        let facts = ProductionInputManifestFacts {
            canonical_digest_blake3: "digest",
            claimed_digest_blake3: "digest",
            expected_attempt: Some(attempt),
            actual_attempt: attempt,
            expected_store_prefix: "/store",
            actual_store_prefix: "/store",
        };
        assert_eq!(validate_production_input_manifest_mode(true, TransferMode::Streaming, false), Ok(()));
        assert_eq!(validate_production_input_manifest_scope(facts), Ok(()));
        assert_eq!(validate_production_input_manifest_content_binding("content", "content"), Ok(()));
        assert_eq!(
            validate_production_input_manifest_content_binding("content", "forged"),
            Err(ProductionInputManifestBlocker::BindingMismatch)
        );
        assert_eq!(
            validate_production_input_manifest_scope(ProductionInputManifestFacts {
                actual_attempt: ProductionManifestAttempt {
                    fence_generation: 3,
                    ..attempt
                },
                ..facts
            }),
            Err(ProductionInputManifestBlocker::BindingMismatch)
        );
        assert_eq!(
            validate_production_input_manifest_scope(ProductionInputManifestFacts {
                claimed_digest_blake3: "forged",
                actual_attempt: ProductionManifestAttempt {
                    fence_generation: 3,
                    ..attempt
                },
                ..facts
            }),
            Err(ProductionInputManifestBlocker::DigestMismatch)
        );
        assert_eq!(
            validate_production_input_manifest_artifacts(["nar-a", "nar-b"], [("nar-b", true), ("nar-a", true)]),
            Ok(())
        );
        assert_eq!(
            validate_production_input_manifest_artifacts(["nar-a", "nar-b"], [("nar-a", true), ("nar-b", false)]),
            Err(ProductionInputManifestBlocker::ArtifactMismatch)
        );
        assert_eq!(
            validate_production_input_manifest_artifacts(["nar-a", "nar-b"], [("nar-a", true)]),
            Err(ProductionInputManifestBlocker::ArtifactMismatch)
        );
        assert_eq!(
            validate_production_input_manifest_artifacts(["nar-a", "nar-a"], [("nar-a", true)]),
            Err(ProductionInputManifestBlocker::ArtifactMismatch)
        );
        assert_eq!(
            validate_production_input_manifest_artifacts(["nar-a"], [("nar-a", true), ("nar-a", true)]),
            Err(ProductionInputManifestBlocker::ArtifactMismatch)
        );
    }

    #[test]
    fn production_output_manifest_artifacts_bind_nar_and_exact_serialized_pathinfo_bytes() {
        let nar = ProductionOutputArtifactFacts {
            kind: ProductionOutputArtifactKind::Nar,
            digest_blake3: "nar-digest",
            size_bytes: 3,
        };
        assert_eq!(validate_production_output_nar_artifact(Some(nar), Some("nar-digest"), Some(3)), Ok(()));
        assert_eq!(
            validate_production_output_nar_artifact(Some(nar), Some("forged"), Some(3)),
            Err(ProductionOutputManifestBlocker::NarManifestMismatch)
        );
        assert_eq!(
            validate_production_output_nar_artifact(None, Some("nar-digest"), Some(3)),
            Err(ProductionOutputManifestBlocker::NarManifestMissing)
        );
        let bytes = br#"{"store":"output"}"#;
        let digest = blake3::hash(bytes).to_hex();
        let pathinfo = ProductionOutputArtifactFacts {
            kind: ProductionOutputArtifactKind::PathInfo,
            digest_blake3: digest.as_str(),
            size_bytes: bytes.len() as u64,
        };
        assert_eq!(validate_production_output_pathinfo_artifact(Some(pathinfo), bytes), Ok(()));
        assert_eq!(
            validate_production_output_pathinfo_artifact(Some(pathinfo), br#"{"store":"forged"}"#),
            Err(ProductionOutputManifestBlocker::PathinfoManifestMismatch)
        );
        assert_eq!(
            validate_production_output_pathinfo_artifact(
                Some(ProductionOutputArtifactFacts {
                    kind: ProductionOutputArtifactKind::Other,
                    ..pathinfo
                }),
                bytes
            ),
            Err(ProductionOutputManifestBlocker::PathinfoManifestMismatch)
        );
    }

    #[test]
    fn production_manifest_rejects_digest_then_stale_fence_before_output_artifacts() {
        let attempt = ProductionManifestAttempt {
            job_id: "job-1",
            attempt_id: "attempt-1",
            fence_generation: 7,
        };
        let valid = ProductionOutputManifestFacts {
            canonical_digest_blake3: "digest-1",
            claimed_digest_blake3: "digest-1",
            expected_attempt: Some(attempt),
            actual_attempt: attempt,
            expected_store_prefix: "/store",
            actual_store_prefix: "/store",
            expected_content_blake3: "content-1",
            actual_content_blake3: "content-1",
            output_count: 2,
            artifacts_per_output: 2,
            actual_artifact_count: 4,
        };
        assert_eq!(validate_production_output_manifest_binding(valid), Ok(()));
        let stale = ProductionOutputManifestFacts {
            actual_attempt: ProductionManifestAttempt {
                fence_generation: 6,
                ..attempt
            },
            ..valid
        };
        assert_eq!(
            validate_production_output_manifest_binding(ProductionOutputManifestFacts {
                claimed_digest_blake3: "forged-digest",
                ..stale
            }),
            Err(ProductionOutputManifestBlocker::DigestMismatch)
        );
        assert_eq!(
            validate_production_output_manifest_binding(stale),
            Err(ProductionOutputManifestBlocker::AttemptStale)
        );
        assert_eq!(
            validate_production_output_manifest_binding(ProductionOutputManifestFacts {
                actual_artifact_count: 3,
                ..valid
            }),
            Err(ProductionOutputManifestBlocker::ArtifactCountMismatch)
        );
        assert_eq!(
            validate_production_output_manifest_mode(TransferMode::Streaming, true),
            Err(ProductionOutputManifestBlocker::StreamingFallbackInconsistent)
        );
    }

    #[test]
    fn delta_accounting_and_fallback_preserve_accepted_bytes() {
        let selected = select_fixture_output_transfer(BOTH, BOTH, 64).unwrap();
        assert_eq!(selected.mode, TransferMode::Delta);
        assert_eq!((selected.transferred_bytes, selected.reused_bytes), (16, 48));
        let failed = TransferCapabilities {
            simulate_delta_failure: true,
            ..BOTH
        };
        let selected = select_fixture_output_transfer(failed, BOTH, 64).unwrap();
        assert_eq!(selected, full_selection(64, Some(TransferFallback::DeltaTransferFailed)));
        let huge = select_fixture_output_transfer(BOTH, BOTH, u64::MAX).unwrap();
        assert_eq!(huge.transferred_bytes.saturating_add(huge.reused_bytes), u64::MAX);
    }

    #[test]
    fn negotiated_streaming_never_claims_fixture_runtime_completion() {
        let streaming_only = TransferCapabilities {
            delta: false,
            full: false,
            streaming: true,
            simulate_delta_failure: false,
        };
        assert_eq!(
            select_fixture_output_transfer(streaming_only, streaming_only, 7),
            Err(TransferBlocker::StreamingRuntimeEvidenceRequired)
        );
        assert_eq!(
            select_production_output_transfer(streaming_only, streaming_only),
            Ok((TransferMode::Streaming, None))
        );
        let with_full = TransferCapabilities {
            full: true,
            ..streaming_only
        };
        assert_eq!(
            select_fixture_output_transfer(with_full, with_full, 7),
            Ok(full_selection(7, Some(TransferFallback::StreamingRuntimeNotBound)))
        );
        let streaming_delta_without_full = TransferCapabilities {
            delta: true,
            full: false,
            streaming: true,
            simulate_delta_failure: false,
        };
        assert_eq!(
            select_production_output_transfer(streaming_delta_without_full, streaming_delta_without_full),
            Ok((TransferMode::Full, Some(TransferFallback::DeltaUnavailable)))
        );
    }

    #[test]
    fn uploaded_bytes_and_missing_identity_fail_closed() {
        let refs = vec![String::from("a")];
        assert_eq!(admit_missing_uploads(&refs, &refs, 10, 10, 10), Ok(()));
        assert_eq!(
            admit_missing_uploads(&refs, &refs, 11, u64::MAX, 10),
            Err(TransferBlocker::UploadByteLimitExceeded)
        );
        assert_eq!(
            admit_missing_uploads(&refs, &refs, 11, 10, u64::MAX),
            Err(TransferBlocker::UploadByteLimitExceeded)
        );
        assert_eq!(
            admit_missing_uploads(&refs, &[String::from("b")], 1, 1, 10),
            Err(TransferBlocker::UploadedInputSetMismatch)
        );
    }

    #[test]
    fn stale_scope_and_forged_checkpoint_cursor_cannot_resume() {
        let scope = TransferScopeFacts {
            session_id: "session",
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 7,
            manifest_digest_blake3: "manifest",
            policy_digest_blake3: "policy",
        };
        let stale = TransferScopeFacts {
            fence_generation: 8,
            ..scope
        };
        assert_eq!(validate_transfer_scope(scope, stale), Err(TransferProgressError::CheckpointScopeMismatch));
        let counters = TransferCheckpointCounterFacts {
            transferred_bytes: 0,
            reused_bytes: 0,
            next_sequence: 33,
            policy_total_bytes_max: 64,
            manifest_total_bytes: 8,
            chunk_count_max: 16,
            replay_rounds_max: 2,
        };
        assert_eq!(
            validate_transfer_checkpoint_counters(counters, core::iter::empty(), core::iter::empty()),
            Err(TransferProgressError::CheckpointCursorForged),
        );
        assert_eq!(
            validate_transfer_checkpoint_counters(
                TransferCheckpointCounterFacts {
                    next_sequence: 1,
                    ..counters
                },
                [TransferDemandChunkFacts {
                    digest_blake3: "known",
                    size_bytes: 4,
                }]
                .into_iter(),
                ["forged"].into_iter(),
            ),
            Err(TransferProgressError::ChunkUnknown),
        );
    }

    #[test]
    fn unknown_chunk_and_ungranted_credit_reject_before_reservation() {
        let limits = TransferCreditLimits {
            in_flight_bytes_max: 4,
            in_flight_chunks_max: 1,
            buffered_chunks_max: 1,
            chunk_bytes_max: 4,
            total_bytes_max: 8,
        };
        let state = TransferCreditFacts {
            granted_bytes_remaining: 4,
            granted_chunks_remaining: 1,
            transferred_bytes: 0,
            next_sequence: 0,
        };
        assert!(validate_transfer_credit_state(limits, state, core::iter::empty()).is_ok());
        let scope = TransferScopeFacts {
            session_id: "session",
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 7,
            manifest_digest_blake3: "manifest",
            policy_digest_blake3: "policy",
        };
        let demanded = TransferReservedChunkFacts {
            artifact_id: "artifact",
            artifact_kind: 0,
            index: 0,
            offset_bytes: 0,
            size_bytes: 4,
            digest_blake3: "known",
        };
        let unknown = TransferReservedChunkFacts {
            artifact_id: "other",
            ..demanded
        };
        assert_eq!(
            reserve_transfer_chunk(limits, scope, scope, scope, state, 0, unknown, [demanded].into_iter(), false)
                .map(|_| ()),
            Err(TransferProgressError::ChunkNotDemanded),
        );
        let no_credit = TransferCreditFacts {
            granted_bytes_remaining: 0,
            granted_chunks_remaining: 0,
            ..state
        };
        assert_eq!(
            reserve_transfer_chunk(limits, scope, scope, scope, no_credit, 0, demanded, [demanded].into_iter(), false)
                .map(|_| ()),
            Err(TransferProgressError::CreditExceeded),
        );
    }
}
