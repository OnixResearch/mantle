//! Bounded resumable transfer shell over Mantle's existing content identities.
//!
//! Pure protocol decisions live in `crunch_build::distributed::remote_transfer`.
//! This module owns file, castore, NAR, source, PathInfo, attestation, delta,
//! receiver-spool, and fenced-checkpoint I/O. Completion never claims admission.
//!
//! r[impl remote_builds.attempt_scoped_transfer_resume]
//! r[impl store_transports.resumable_castore_sessions]
//! r[impl store_transports.receiver_driven_backpressure]
//! r[impl store_transports.content_presence_early_cutoff]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_build::distributed::*;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::path_info::PathInfo;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub const REMOTE_TRANSFER_LEASE_SCHEMA: &str = "mantle-remote-transfer-lease-v1";
pub const REMOTE_TRANSFER_DURABLE_STATE_SCHEMA: &str = "mantle-remote-transfer-state-v1";
pub const REMOTE_TRANSFER_DATA_FRAME_SCHEMA: &str = "mantle-remote-transfer-data-frame-v1";
pub const REMOTE_TRANSFER_STATE_DIR: &str = "remote-transfers";
pub const REMOTE_TRANSFER_RECEIVER_DIR: &str = "remote-transfer-receiver";
pub const MAX_REMOTE_INLINE_FIXTURE_BOOTSTRAP_BYTES: u64 = 262_144;
pub const MAX_REMOTE_TRANSFER_DURABLE_STATE_BYTES_HARD: u64 = 2_097_152;
pub const MAX_REMOTE_TRANSFER_LEASE_REFS_HARD: u32 = 1_000_000;

const TEMP_FILE_EXTENSION: &str = "tmp";
const COPY_BUFFER_BYTES: u32 = 65_536;
const REMOTE_TRANSFER_DATA_HEADER_BYTES: usize = 4;
const FIRST_CHUNK_COUNT: u32 = 1;
const INITIAL_PROGRESS_STEP: u64 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransferDirection {
    Upload,
    Download,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteInlinePayloadCapability {
    Fixture,
    Bootstrap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRemoteTransferArtifact {
    pub descriptor: RemoteTransferArtifact,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRemoteTransfer {
    pub manifest: CanonicalRemoteTransferManifest,
    pub sources: BTreeMap<RemoteTransferArtifactId, PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTransferBinding {
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub store_prefix: String,
    pub requested_content_blake3: RemoteTransferDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferLease {
    pub schema: String,
    pub scope: RemoteTransferScope,
    pub artifact_ids: Vec<RemoteTransferArtifactId>,
    pub expires_unix_s: u64,
    pub last_progress_step: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferDurableState {
    pub schema: String,
    pub checkpoint: RemoteTransferCheckpoint,
    pub lease: RemoteTransferLease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteTransferAdmissionFacts {
    pub required_closure_metadata_verified: bool,
    pub path_info_admitted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteTransferRunOptions {
    pub direction: RemoteTransferDirection,
    pub interrupt_after_chunks: Option<u32>,
    pub now_unix_s: u64,
    pub lease_expires_unix_s: u64,
    pub admission: RemoteTransferAdmissionFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransferShellDisposition {
    Completed,
    AlreadyPresent,
    Interrupted,
    AwaitingAdmission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferShellReport {
    pub direction: RemoteTransferDirection,
    pub disposition: RemoteTransferShellDisposition,
    pub manifest_digest_blake3: String,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub chunks_sent: u32,
    pub sent_chunk_digests: Vec<String>,
    pub checkpoint_path: String,
    pub output_admission_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteTransferStateLoad {
    Missing,
    Loaded(RemoteTransferDurableState),
    StaleInvalidated,
    ExpiredInvalidated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferDataFrameHeader {
    pub schema: String,
    pub chunk: RemoteTransferChunkHeader,
}

/// Write one demanded chunk to a socket/stdio-compatible stream. Credit and
/// demand are checked before the source read or payload write.
pub fn write_remote_transfer_data_chunk(
    mut writer: impl Write,
    prepared: &PreparedRemoteTransfer,
    policy: RemoteTransferPolicy,
    demand: &RemoteTransferDemand,
    state: &RemoteTransferCreditState,
    receiver_grant: RemoteTransferCreditGrant,
    missing: &RemoteTransferChunkDemand,
) -> Result<RemoteTransferCreditState, String> {
    validate_prepared_remote_transfer(prepared, policy)?;
    if receiver_grant.bytes != u64::from(missing.chunk.size_bytes) || receiver_grant.chunks != FIRST_CHUNK_COUNT {
        return Err(reason(RemoteTransferReasonCode::CreditGrantInvalid));
    }
    let granted =
        grant_remote_transfer_credit(policy, state, receiver_grant.clone(), receiver_grant.bytes).map_err(reason)?;
    let chunk = RemoteTransferChunkHeader {
        scope: demand.scope.clone(),
        artifact_id: missing.artifact_id.clone(),
        artifact_kind: missing.artifact_kind,
        chunk: missing.chunk.clone(),
        sequence: granted.next_sequence,
    };
    let reserved =
        reserve_remote_transfer_chunk(&prepared.manifest, policy, demand, &granted, &chunk).map_err(reason)?;
    let payload = read_prepared_chunk_after_reservation(prepared, &chunk)?;
    let data_header = RemoteTransferDataFrameHeader {
        schema: REMOTE_TRANSFER_DATA_FRAME_SCHEMA.to_string(),
        chunk,
    };
    let encoded = serde_json::to_vec(&data_header).map_err(|err| format!("serializing transfer data header: {err}"))?;
    let encoded_len = u32::try_from(encoded.len()).map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    if encoded_len == 0 || encoded_len > policy.control_bytes_max {
        return Err(reason(RemoteTransferReasonCode::ManifestBoundsExceeded));
    }
    writer
        .write_all(&encoded_len.to_be_bytes())
        .and_then(|()| writer.write_all(&encoded))
        .and_then(|()| writer.write_all(&payload))
        .and_then(|()| writer.flush())
        .map_err(|err| format!("writing bounded transfer data frame: {err}"))?;
    assert_eq!(payload.len(), usize::try_from(missing.chunk.size_bytes).unwrap_or(usize::MAX));
    assert!(encoded_len <= policy.control_bytes_max);
    Ok(reserved)
}

/// Receive one socket/stdio data chunk. The fixed header and bounded control
/// header are read first; receiver credit is reserved before payload allocation.
pub fn receive_remote_transfer_data_chunk(
    mut reader: impl Read,
    receiver_root: &Path,
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
    demand: &RemoteTransferDemand,
    state: &RemoteTransferCreditState,
    progress_step: u64,
) -> Result<(RemoteTransferCreditState, RemoteTransferAcknowledgement), String> {
    validate_canonical_remote_transfer_manifest(manifest, policy)?;
    let mut length_bytes = [0_u8; REMOTE_TRANSFER_DATA_HEADER_BYTES];
    reader
        .read_exact(&mut length_bytes)
        .map_err(|err| format!("reading transfer data header length: {err}"))?;
    let header_len = u32::from_be_bytes(length_bytes);
    if header_len == 0 || header_len > policy.control_bytes_max {
        return Err(reason(RemoteTransferReasonCode::ManifestBoundsExceeded));
    }
    let mut encoded =
        vec![0_u8; usize::try_from(header_len).map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?];
    reader.read_exact(&mut encoded).map_err(|err| format!("reading transfer data header: {err}"))?;
    let data_header: RemoteTransferDataFrameHeader =
        serde_json::from_slice(&encoded).map_err(|err| format!("parsing transfer data header: {err}"))?;
    if data_header.schema != REMOTE_TRANSFER_DATA_FRAME_SCHEMA {
        return Err(reason(RemoteTransferReasonCode::ManifestSchemaUnsupported));
    }
    let grant = RemoteTransferCreditGrant {
        bytes: u64::from(data_header.chunk.chunk.size_bytes),
        chunks: FIRST_CHUNK_COUNT,
    };
    let available_storage_bytes = available_storage_bytes(receiver_root)?;
    let granted = grant_remote_transfer_credit(policy, state, grant, available_storage_bytes).map_err(reason)?;
    let reserved =
        reserve_remote_transfer_chunk(manifest, policy, demand, &granted, &data_header.chunk).map_err(reason)?;
    let payload_len = usize::try_from(data_header.chunk.chunk.size_bytes)
        .map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    let mut payload = vec![0_u8; payload_len];
    reader.read_exact(&mut payload).map_err(|err| format!("reading reserved transfer payload: {err}"))?;
    let observed = RemoteTransferDigest::new(blake3::hash(&payload).to_hex().to_string()).map_err(reason)?;
    persist_verified_chunk(receiver_root, &data_header.chunk.chunk, &payload, &observed)?;
    let transferred_bytes = reserved
        .transferred_bytes
        .checked_add(u64::from(data_header.chunk.chunk.size_bytes))
        .ok_or_else(|| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    let acknowledgement = RemoteTransferAcknowledgement {
        scope: data_header.chunk.scope,
        chunk_digest_blake3: data_header.chunk.chunk.digest_blake3,
        sequence: data_header.chunk.sequence,
        transferred_bytes,
    };
    let acknowledged = acknowledge_remote_transfer_chunk(policy, &reserved, &acknowledgement, &observed, progress_step)
        .map_err(reason)?;
    assert!(acknowledged.in_flight.is_empty());
    assert!(acknowledged.acknowledged_chunk_digests.contains(&observed));
    Ok((acknowledged, acknowledgement))
}

/// Pure manifest construction plus source-path binding.
pub fn prepare_remote_transfer(
    binding: RemoteTransferBinding,
    policy: RemoteTransferPolicy,
    artifacts: Vec<PreparedRemoteTransferArtifact>,
) -> Result<PreparedRemoteTransfer, String> {
    policy.validate().map_err(reason)?;
    if artifacts.is_empty() {
        return Err("remote-transfer-artifacts-empty".to_string());
    }
    let policy_digest = canonical_remote_transfer_policy_digest(policy).map_err(reason)?;
    let session_id = derive_remote_transfer_session_id(
        &binding.job_id,
        &binding.attempt_id,
        binding.fence_generation,
        &binding.requested_content_blake3,
        &policy_digest,
    )
    .map_err(reason)?;
    let mut sources = BTreeMap::new();
    let mut descriptors = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        if sources.insert(artifact.descriptor.artifact_id.clone(), artifact.source_path).is_some() {
            return Err(reason(RemoteTransferReasonCode::ArtifactDuplicate));
        }
        descriptors.push(artifact.descriptor);
    }
    let manifest = canonicalize_remote_transfer_manifest(
        RemoteTransferManifest {
            schema: REMOTE_TRANSFER_MANIFEST_SCHEMA.to_string(),
            session_id,
            job_id: binding.job_id,
            attempt_id: binding.attempt_id,
            fence_generation: binding.fence_generation,
            policy_digest_blake3: policy_digest,
            store_prefix: binding.store_prefix,
            requested_content_blake3: binding.requested_content_blake3,
            artifacts: descriptors,
        },
        policy,
    )
    .map_err(reason)?;
    assert_eq!(manifest.manifest.artifacts.len(), sources.len());
    assert!(!manifest.canonical_bytes.is_empty());
    Ok(PreparedRemoteTransfer { manifest, sources })
}

/// Scan a file once and produce bounded chunk descriptors without buffering it.
pub fn prepare_file_transfer_artifact(
    artifact_id: RemoteTransferArtifactId,
    artifact_kind: RemoteTransferArtifactKind,
    source_path: &Path,
    required_for_completion: bool,
    nar_sha256_hex: Option<String>,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    policy.validate().map_err(reason)?;
    let metadata = fs::metadata(source_path)
        .map_err(|err| format!("remote-transfer-source-metadata {}: {err}", source_path.display()))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > policy.total_bytes_max {
        return Err(reason(RemoteTransferReasonCode::TotalBytesExceeded));
    }
    let scan = scan_artifact_file(source_path, policy)?;
    let descriptor = RemoteTransferArtifact {
        artifact_id,
        artifact_kind,
        digest_blake3: scan.digest_blake3,
        size_bytes: scan.size_bytes,
        required_for_completion,
        nar_sha256_hex,
        chunks: scan.chunks,
    };
    assert_eq!(descriptor.size_bytes, metadata.len());
    assert!(!descriptor.chunks.is_empty());
    Ok(PreparedRemoteTransferArtifact {
        descriptor,
        source_path: source_path.to_path_buf(),
    })
}

/// The only inline path is explicit and hard-capped for fixtures/bootstrap.
pub fn prepare_inline_fixture_or_bootstrap_artifact(
    capability: RemoteInlinePayloadCapability,
    artifact_id: RemoteTransferArtifactId,
    artifact_kind: RemoteTransferArtifactKind,
    bytes: &[u8],
    spool_dir: &Path,
    required_for_completion: bool,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    let byte_count = u64::try_from(bytes.len()).map_err(|_| "remote-inline-size-overflow".to_string())?;
    if byte_count == 0 || byte_count > MAX_REMOTE_INLINE_FIXTURE_BOOTSTRAP_BYTES {
        return Err("remote-inline-fixture-bootstrap-payload-too-large".to_string());
    }
    let label = match capability {
        RemoteInlinePayloadCapability::Fixture => "fixture",
        RemoteInlinePayloadCapability::Bootstrap => "bootstrap",
    };
    let path = safe_spool_path(spool_dir, artifact_id.as_str(), label)?;
    write_atomic_bytes(&path, bytes)?;
    prepare_file_transfer_artifact(artifact_id, artifact_kind, &path, required_for_completion, None, policy)
}

/// Render a NAR directly into a spool file; no whole-NAR `Vec` exists.
pub async fn prepare_nar_transfer_artifact(
    store: &crunch_store::StoreHandle,
    path_info: &PathInfo,
    artifact_id: RemoteTransferArtifactId,
    spool_dir: &Path,
    required_for_completion: bool,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    policy.validate().map_err(reason)?;
    if path_info.nar_size == 0 || path_info.nar_size > policy.total_bytes_max {
        return Err(reason(RemoteTransferReasonCode::TotalBytesExceeded));
    }
    let path = safe_spool_path(spool_dir, artifact_id.as_str(), "nar")?;
    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(|err| format!("creating NAR spool {}: {err}", path.display()))?;
    store
        .render_nar(&path_info.node, &mut file)
        .await
        .map_err(|err| format!("rendering transfer NAR: {err}"))?;
    file.flush().await.map_err(|err| format!("flushing NAR spool: {err}"))?;
    drop(file);
    let prepared = prepare_file_transfer_artifact(
        artifact_id,
        RemoteTransferArtifactKind::Nar,
        &path,
        required_for_completion,
        Some(data_encoding::HEXLOWER.encode(&path_info.nar_sha256)),
        policy,
    )?;
    if prepared.descriptor.size_bytes != path_info.nar_size {
        return Err("remote-transfer-nar-size-mismatch".to_string());
    }
    verify_nar_sha256(&path, path_info)?;
    Ok(prepared)
}

pub async fn prepare_castore_blob_transfer_artifact(
    store: &crunch_store::StoreHandle,
    digest: snix_castore::B3Digest,
    spool_dir: &Path,
    required_for_completion: bool,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    let digest_hex = data_encoding::HEXLOWER.encode(digest.as_ref());
    let id = RemoteTransferArtifactId::new(format!("castore-blob:{digest_hex}")).map_err(reason)?;
    let path = safe_spool_path(spool_dir, id.as_str(), "blob")?;
    let reader = store
        .blob_service()
        .open_read(&digest)
        .await
        .map_err(|err| format!("opening castore blob {digest_hex}: {err}"))?
        .ok_or_else(|| format!("castore blob missing: {digest_hex}"))?;
    copy_async_reader_bounded(reader, &path, policy.total_bytes_max).await?;
    let prepared = prepare_file_transfer_artifact(
        id,
        RemoteTransferArtifactKind::CastoreBlob,
        &path,
        required_for_completion,
        None,
        policy,
    )?;
    if prepared.descriptor.digest_blake3.as_str() != digest_hex {
        return Err("castore-blob-transfer-digest-mismatch".to_string());
    }
    Ok(prepared)
}

pub async fn prepare_castore_directory_transfer_artifact(
    store: &crunch_store::StoreHandle,
    digest: snix_castore::B3Digest,
    spool_dir: &Path,
    required_for_completion: bool,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    let digest_hex = data_encoding::HEXLOWER.encode(digest.as_ref());
    let directory = store
        .directory_service()
        .get(&digest)
        .await
        .map_err(|err| format!("reading castore directory {digest_hex}: {err}"))?
        .ok_or_else(|| format!("castore directory missing: {digest_hex}"))?;
    let bytes = postcard::to_stdvec(&snix_castore::proto::Directory::from(directory))
        .map_err(|err| format!("serializing castore directory: {err}"))?;
    if blake3::hash(&bytes).to_hex().as_str() != digest_hex {
        return Err("castore-directory-transfer-digest-mismatch".to_string());
    }
    let id = RemoteTransferArtifactId::new(format!("castore-directory:{digest_hex}")).map_err(reason)?;
    let path = safe_spool_path(spool_dir, id.as_str(), "directory")?;
    write_atomic_bytes(&path, &bytes)?;
    prepare_file_transfer_artifact(
        id,
        RemoteTransferArtifactKind::CastoreDirectory,
        &path,
        required_for_completion,
        None,
        policy,
    )
}

pub fn prepare_pathinfo_transfer_artifact(
    path_info: &PathInfo,
    spool_dir: &Path,
    required_for_completion: bool,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    let bytes = serde_json::to_vec(path_info).map_err(|err| format!("serializing transfer PathInfo: {err}"))?;
    let id = RemoteTransferArtifactId::new(format!("pathinfo:{}", path_info.store_path)).map_err(reason)?;
    let path = safe_spool_path(spool_dir, id.as_str(), "pathinfo")?;
    write_atomic_bytes(&path, &bytes)?;
    prepare_file_transfer_artifact(
        id,
        RemoteTransferArtifactKind::PathInfo,
        &path,
        required_for_completion,
        None,
        policy,
    )
}

pub fn prepare_attestation_transfer_artifact(
    identity_blake3: &str,
    canonical_bytes: &[u8],
    spool_dir: &Path,
    required_for_completion: bool,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    let expected = RemoteTransferDigest::new(identity_blake3.to_string()).map_err(reason)?;
    let observed = RemoteTransferDigest::new(blake3::hash(canonical_bytes).to_hex().to_string()).map_err(reason)?;
    if observed != expected {
        return Err("attestation-transfer-digest-mismatch".to_string());
    }
    let id = RemoteTransferArtifactId::new(format!("attestation:{identity_blake3}")).map_err(reason)?;
    let path = safe_spool_path(spool_dir, id.as_str(), "attestation")?;
    write_atomic_bytes(&path, canonical_bytes)?;
    prepare_file_transfer_artifact(
        id,
        RemoteTransferArtifactKind::Attestation,
        &path,
        required_for_completion,
        None,
        policy,
    )
}

pub fn prepare_source_bundle_transfer_artifact(
    source_identity: &str,
    bundle_path: &Path,
    spool_dir: &Path,
    required_for_completion: bool,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    let id = RemoteTransferArtifactId::new(format!("source-bundle:{source_identity}")).map_err(reason)?;
    let path = safe_spool_path(spool_dir, id.as_str(), "source-bundle")?;
    copy_file_bounded(bundle_path, &path, policy.total_bytes_max)?;
    prepare_file_transfer_artifact(
        id,
        RemoteTransferArtifactKind::SourceBundle,
        &path,
        required_for_completion,
        None,
        policy,
    )
}

pub fn prepare_delta_transfer_artifacts(
    frames: &[crunch_delta::DeltaTransferFrame],
    spool_dir: &Path,
    policy: RemoteTransferPolicy,
) -> Result<Vec<PreparedRemoteTransferArtifact>, String> {
    policy.validate().map_err(reason)?;
    let mut prepared = Vec::new();
    for frame in frames {
        let artifact = match frame {
            crunch_delta::DeltaTransferFrame::Blob { digest, bytes } => prepare_delta_bytes(
                "delta-blob",
                digest,
                RemoteTransferArtifactKind::DeltaBlob,
                bytes,
                spool_dir,
                policy,
            )?,
            crunch_delta::DeltaTransferFrame::Chunk {
                chunk_digest, bytes, ..
            } => prepare_delta_bytes(
                "delta-chunk",
                chunk_digest,
                RemoteTransferArtifactKind::DeltaChunk,
                bytes,
                spool_dir,
                policy,
            )?,
            crunch_delta::DeltaTransferFrame::FinalPathInfo { path_info } => {
                let bytes =
                    serde_json::to_vec(path_info).map_err(|err| format!("serializing delta PathInfo: {err}"))?;
                let digest = blake3::hash(&bytes).to_hex();
                let id = RemoteTransferArtifactId::new(format!("delta-pathinfo:{digest}")).map_err(reason)?;
                let path = safe_spool_path(spool_dir, id.as_str(), "delta-pathinfo")?;
                write_atomic_bytes(&path, &bytes)?;
                prepare_file_transfer_artifact(id, RemoteTransferArtifactKind::PathInfo, &path, true, None, policy)?
            }
        };
        prepared.push(artifact);
    }
    Ok(prepared)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtifactFileScan {
    digest_blake3: RemoteTransferDigest,
    size_bytes: u64,
    chunks: Vec<RemoteTransferChunkDescriptor>,
}

fn scan_artifact_file(path: &Path, policy: RemoteTransferPolicy) -> Result<ArtifactFileScan, String> {
    let capacity =
        usize::try_from(policy.chunk_bytes_max).map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    let mut file = File::open(path).map_err(|err| format!("opening transfer source {}: {err}", path.display()))?;
    let mut buffer = vec![0_u8; capacity];
    let mut hasher = blake3::Hasher::new();
    let mut chunks = Vec::new();
    let mut offset_bytes = 0_u64;
    let mut index = 0_u32;
    loop {
        let read_bytes = file.read(&mut buffer).map_err(|err| format!("reading transfer source: {err}"))?;
        if read_bytes == 0 {
            break;
        }
        let size_bytes = u32::try_from(read_bytes).map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
        hasher.update(&buffer[..read_bytes]);
        chunks.push(RemoteTransferChunkDescriptor {
            index,
            offset_bytes,
            size_bytes,
            digest_blake3: RemoteTransferDigest::new(blake3::hash(&buffer[..read_bytes]).to_hex().to_string())
                .map_err(reason)?,
        });
        offset_bytes = offset_bytes
            .checked_add(u64::from(size_bytes))
            .ok_or_else(|| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
        index = index.checked_add(1).ok_or_else(|| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
        if offset_bytes > policy.total_bytes_max || index > policy.chunk_count_max {
            return Err(reason(RemoteTransferReasonCode::TotalBytesExceeded));
        }
    }
    if chunks.is_empty() {
        return Err(reason(RemoteTransferReasonCode::ArtifactSizeInvalid));
    }
    Ok(ArtifactFileScan {
        digest_blake3: RemoteTransferDigest::new(hasher.finalize().to_hex().to_string()).map_err(reason)?,
        size_bytes: offset_bytes,
        chunks,
    })
}

async fn copy_async_reader_bounded<R: AsyncRead + Unpin>(reader: R, path: &Path, max_bytes: u64) -> Result<(), String> {
    let mut file = tokio::fs::File::create(path).await.map_err(|err| format!("creating spool: {err}"))?;
    let copied = tokio::io::copy(&mut reader.take(max_bytes.saturating_add(1)), &mut file)
        .await
        .map_err(|err| format!("copying transfer spool: {err}"))?;
    if copied > max_bytes {
        let _ = tokio::fs::remove_file(path).await;
        return Err(reason(RemoteTransferReasonCode::TotalBytesExceeded));
    }
    file.flush().await.map_err(|err| format!("flushing transfer spool: {err}"))
}

fn prepare_delta_bytes(
    prefix: &str,
    expected: &snix_castore::B3Digest,
    kind: RemoteTransferArtifactKind,
    bytes: &[u8],
    spool_dir: &Path,
    policy: RemoteTransferPolicy,
) -> Result<PreparedRemoteTransferArtifact, String> {
    let digest = data_encoding::HEXLOWER.encode(expected.as_ref());
    if blake3::hash(bytes).to_hex().as_str() != digest {
        return Err("delta-transfer-frame-digest-mismatch".to_string());
    }
    let id = RemoteTransferArtifactId::new(format!("{prefix}:{digest}")).map_err(reason)?;
    let path = safe_spool_path(spool_dir, id.as_str(), "delta")?;
    write_atomic_bytes(&path, bytes)?;
    prepare_file_transfer_artifact(id, kind, &path, true, None, policy)
}

fn safe_spool_path(root: &Path, identity: &str, extension: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(root).map_err(|err| format!("creating spool {}: {err}", root.display()))?;
    Ok(root.join(format!("{}.{}", blake3::hash(identity.as_bytes()).to_hex(), extension)))
}

/// Run receiver-demanded chunks with one bounded reservation before each read.
pub fn execute_prepared_remote_transfer(
    prepared: &PreparedRemoteTransfer,
    policy: RemoteTransferPolicy,
    state_dir: &Path,
    receiver_root: &Path,
    options: RemoteTransferRunOptions,
) -> Result<RemoteTransferShellReport, String> {
    policy.validate().map_err(reason)?;
    validate_run_options(options)?;
    validate_prepared_remote_transfer(prepared, policy)?;
    let scope = remote_transfer_scope(&prepared.manifest);
    let _session_lock = acquire_remote_transfer_session_lock(state_dir, &scope.session_id)?;
    let checkpoint_path = remote_transfer_state_path(state_dir, &scope.session_id);
    let previous = match load_remote_transfer_state(state_dir, &scope, policy, options.now_unix_s)? {
        RemoteTransferStateLoad::Loaded(state) => Some(state),
        RemoteTransferStateLoad::Missing
        | RemoteTransferStateLoad::StaleInvalidated
        | RemoteTransferStateLoad::ExpiredInvalidated => None,
    };
    let receiver_before = probe_remote_transfer_receiver(receiver_root, &prepared.manifest, options.admission)?;
    let resume = plan_remote_transfer_resume(
        &prepared.manifest,
        policy,
        &receiver_before,
        previous.as_ref().map(|state| &state.checkpoint),
        previous.as_ref().map(|state| &state.checkpoint),
    )
    .map_err(reason)?;
    let initial = decide_remote_transfer_cutoff(
        &prepared.manifest,
        &resume.demand,
        &receiver_before,
        &resume.acknowledged_chunk_digests,
        resume.transferred_bytes,
    );
    if initial.disposition == RemoteTransferCompletionDisposition::AlreadyPresent {
        return Ok(shell_report(
            options.direction,
            RemoteTransferShellDisposition::AlreadyPresent,
            prepared,
            0,
            initial.reused_bytes,
            Vec::new(),
            &checkpoint_path,
        ));
    }
    run_missing_chunks(prepared, policy, state_dir, receiver_root, options, previous, receiver_before, resume)
}

fn run_missing_chunks(
    prepared: &PreparedRemoteTransfer,
    policy: RemoteTransferPolicy,
    state_dir: &Path,
    receiver_root: &Path,
    options: RemoteTransferRunOptions,
    previous: Option<RemoteTransferDurableState>,
    receiver_before: RemoteTransferReceiverFacts,
    resume: RemoteTransferResumePlan,
) -> Result<RemoteTransferShellReport, String> {
    let scope = remote_transfer_scope(&prepared.manifest);
    let checkpoint_path = remote_transfer_state_path(state_dir, &scope.session_id);
    let lease = transfer_lease(prepared, options, &scope)?;
    let mut credit = RemoteTransferCreditState {
        acknowledged_chunk_digests: resume.acknowledged_chunk_digests,
        transferred_bytes: resume.transferred_bytes,
        next_sequence: resume.next_sequence,
        last_progress_step: previous
            .as_ref()
            .map(|state| state.lease.last_progress_step)
            .unwrap_or(INITIAL_PROGRESS_STEP),
        ..RemoteTransferCreditState::default()
    };
    let mut sent = Vec::new();
    for missing in &resume.demand.missing_chunks {
        let progress_step = credit
            .last_progress_step
            .checked_add(1)
            .ok_or_else(|| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
        credit = transfer_one_chunk(
            prepared,
            policy,
            state_dir,
            receiver_root,
            &resume.demand,
            missing,
            credit,
            resume.reused_bytes,
            &lease,
            progress_step,
        )?;
        sent.push(missing.chunk.digest_blake3.as_str().to_string());
        if should_interrupt(options.interrupt_after_chunks, sent.len())? {
            return Ok(shell_report(
                options.direction,
                RemoteTransferShellDisposition::Interrupted,
                prepared,
                credit.transferred_bytes,
                resume.reused_bytes,
                sent,
                &checkpoint_path,
            ));
        }
    }
    assemble_remote_transfer_artifacts(receiver_root, &prepared.manifest)?;
    let receiver_after = probe_remote_transfer_receiver(receiver_root, &prepared.manifest, options.admission)?;
    let completion = decide_remote_transfer_cutoff(
        &prepared.manifest,
        &resume.demand,
        &receiver_after,
        &credit.acknowledged_chunk_digests,
        credit.transferred_bytes,
    );
    let disposition = match completion.disposition {
        RemoteTransferCompletionDisposition::AlreadyPresent => RemoteTransferShellDisposition::AlreadyPresent,
        RemoteTransferCompletionDisposition::DemandSatisfied => RemoteTransferShellDisposition::Completed,
        RemoteTransferCompletionDisposition::Continue => RemoteTransferShellDisposition::AwaitingAdmission,
        RemoteTransferCompletionDisposition::Reject => {
            return Err(reason(RemoteTransferReasonCode::TransferRejected));
        }
    };
    persist_transfer_progress(state_dir, policy, &scope, &credit, resume.reused_bytes, &lease)?;
    assert!(receiver_after.complete_artifact_ids.len() >= receiver_before.complete_artifact_ids.len());
    assert!(receiver_after.requested_content_identity_verified);
    assert!(!completion.output_admission_claimed);
    Ok(shell_report(
        options.direction,
        disposition,
        prepared,
        credit.transferred_bytes,
        completion.reused_bytes,
        sent,
        &checkpoint_path,
    ))
}

#[allow(clippy::too_many_arguments)]
fn transfer_one_chunk(
    prepared: &PreparedRemoteTransfer,
    policy: RemoteTransferPolicy,
    state_dir: &Path,
    receiver_root: &Path,
    demand: &RemoteTransferDemand,
    missing: &RemoteTransferChunkDemand,
    state: RemoteTransferCreditState,
    reused_bytes: u64,
    lease: &RemoteTransferLease,
    progress_step: u64,
) -> Result<RemoteTransferCreditState, String> {
    let grant = RemoteTransferCreditGrant {
        bytes: u64::from(missing.chunk.size_bytes),
        chunks: FIRST_CHUNK_COUNT,
    };
    let available_storage_bytes = available_storage_bytes(receiver_root)?;
    let granted = grant_remote_transfer_credit(policy, &state, grant, available_storage_bytes).map_err(reason)?;
    let header = RemoteTransferChunkHeader {
        scope: demand.scope.clone(),
        artifact_id: missing.artifact_id.clone(),
        artifact_kind: missing.artifact_kind,
        chunk: missing.chunk.clone(),
        sequence: granted.next_sequence,
    };
    let reserved =
        reserve_remote_transfer_chunk(&prepared.manifest, policy, demand, &granted, &header).map_err(reason)?;
    // The only chunk allocation occurs after the bounded credit reservation.
    let bytes = read_prepared_chunk_after_reservation(prepared, &header)?;
    let observed = RemoteTransferDigest::new(blake3::hash(&bytes).to_hex().to_string()).map_err(reason)?;
    persist_verified_chunk(receiver_root, &header.chunk, &bytes, &observed)?;
    let transferred_bytes = reserved
        .transferred_bytes
        .checked_add(u64::from(header.chunk.size_bytes))
        .ok_or_else(|| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    let acknowledgement = RemoteTransferAcknowledgement {
        scope: header.scope,
        chunk_digest_blake3: header.chunk.digest_blake3.clone(),
        sequence: header.sequence,
        transferred_bytes,
    };
    let acknowledged = acknowledge_remote_transfer_chunk(policy, &reserved, &acknowledgement, &observed, progress_step)
        .map_err(reason)?;
    persist_transfer_progress(state_dir, policy, &demand.scope, &acknowledged, reused_bytes, lease)?;
    assert!(acknowledged.acknowledged_chunk_digests.contains(&observed));
    assert!(acknowledged.in_flight.is_empty());
    Ok(acknowledged)
}

/// Probe only receiver-owned verified bytes. Sender claims are never accepted.
pub fn probe_remote_transfer_receiver(
    receiver_root: &Path,
    manifest: &CanonicalRemoteTransferManifest,
    admission: RemoteTransferAdmissionFacts,
) -> Result<RemoteTransferReceiverFacts, String> {
    let mut facts = RemoteTransferReceiverFacts {
        required_closure_metadata_verified: admission.required_closure_metadata_verified,
        path_info_admitted: admission.path_info_admitted,
        ..RemoteTransferReceiverFacts::default()
    };
    for artifact in &manifest.manifest.artifacts {
        if receiver_artifact_is_valid(receiver_root, artifact)? {
            facts.complete_artifact_ids.insert(artifact.artifact_id.clone());
            continue;
        }
        for chunk in &artifact.chunks {
            if receiver_chunk_is_valid(receiver_root, chunk)? {
                facts.complete_chunk_digests.insert(chunk.digest_blake3.clone());
            }
        }
    }
    let required_complete = manifest
        .manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.required_for_completion)
        .all(|artifact| facts.complete_artifact_ids.contains(&artifact.artifact_id));
    facts.requested_content_identity_verified = required_complete;
    assert!(facts.complete_artifact_ids.len() <= manifest.manifest.artifacts.len());
    assert!(!facts.requested_content_identity_verified || required_complete);
    Ok(facts)
}

/// Atomically persist and fsync a fenced checkpoint plus lease.
pub fn save_remote_transfer_state(
    state_dir: &Path,
    policy: RemoteTransferPolicy,
    state: &RemoteTransferDurableState,
) -> Result<PathBuf, String> {
    policy.validate().map_err(reason)?;
    validate_durable_state(state, policy)?;
    let bytes = serde_json::to_vec(state).map_err(|err| format!("serializing transfer state: {err}"))?;
    let byte_count = u64::try_from(bytes.len()).map_err(|_| "transfer-state-size-overflow".to_string())?;
    if byte_count > MAX_REMOTE_TRANSFER_DURABLE_STATE_BYTES_HARD {
        return Err("remote-transfer-state-too-large".to_string());
    }
    let path = remote_transfer_state_path(state_dir, &state.checkpoint.scope.session_id);
    write_atomic_bytes(&path, &bytes)?;
    Ok(path)
}

/// Load a lease only when session, attempt, fence, and lifetime remain current.
pub fn load_remote_transfer_state(
    state_dir: &Path,
    expected_scope: &RemoteTransferScope,
    policy: RemoteTransferPolicy,
    now_unix_s: u64,
) -> Result<RemoteTransferStateLoad, String> {
    policy.validate().map_err(reason)?;
    let path = remote_transfer_state_path(state_dir, &expected_scope.session_id);
    if !path.exists() {
        return Ok(RemoteTransferStateLoad::Missing);
    }
    let metadata = fs::metadata(&path).map_err(|err| format!("reading state metadata: {err}"))?;
    if metadata.len() > MAX_REMOTE_TRANSFER_DURABLE_STATE_BYTES_HARD {
        invalidate_transfer_state_file(&path)?;
        return Ok(RemoteTransferStateLoad::StaleInvalidated);
    }
    let bytes = fs::read(&path).map_err(|err| format!("reading transfer state: {err}"))?;
    let state: RemoteTransferDurableState =
        serde_json::from_slice(&bytes).map_err(|err| format!("parsing transfer state: {err}"))?;
    if state.checkpoint.scope != *expected_scope || state.lease.scope != *expected_scope {
        invalidate_transfer_state_file(&path)?;
        return Ok(RemoteTransferStateLoad::StaleInvalidated);
    }
    if now_unix_s > state.lease.expires_unix_s {
        invalidate_transfer_state_file(&path)?;
        return Ok(RemoteTransferStateLoad::ExpiredInvalidated);
    }
    validate_durable_state(&state, policy)?;
    assert_eq!(state.checkpoint.scope, *expected_scope);
    assert!(now_unix_s <= state.lease.expires_unix_s);
    Ok(RemoteTransferStateLoad::Loaded(state))
}

pub fn remote_transfer_state_path(state_dir: &Path, session: &RemoteTransferSessionId) -> PathBuf {
    state_dir.join(REMOTE_TRANSFER_STATE_DIR).join(format!("{}.json", session.as_str()))
}

pub fn remote_transfer_receiver_root(state_dir: &Path, session: &RemoteTransferSessionId) -> PathBuf {
    state_dir.join(REMOTE_TRANSFER_RECEIVER_DIR).join(session.as_str())
}

struct RemoteTransferSessionLock {
    _file: File,
}

fn acquire_remote_transfer_session_lock(
    state_dir: &Path,
    session: &RemoteTransferSessionId,
) -> Result<RemoteTransferSessionLock, String> {
    let lock_dir = state_dir.join(REMOTE_TRANSFER_STATE_DIR);
    fs::create_dir_all(&lock_dir).map_err(|err| format!("creating transfer lock directory: {err}"))?;
    let path = lock_dir.join(format!("{}.lock", session.as_str()));
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|err| format!("opening transfer session lock: {err}"))?;
    FileExt::try_lock_exclusive(&file).map_err(|err| format!("remote-transfer-session-lock-busy: {err}"))?;
    assert!(path.is_file());
    assert_eq!(path.parent(), Some(lock_dir.as_path()));
    Ok(RemoteTransferSessionLock { _file: file })
}

fn available_storage_bytes(receiver_root: &Path) -> Result<u64, String> {
    let existing = receiver_root
        .ancestors()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| "remote-transfer-storage-root-missing".to_string())?;
    let bytes = fs2::available_space(existing)
        .map_err(|err| format!("probing receiver storage capacity {}: {err}", existing.display()))?;
    if bytes == 0 {
        return Err(reason(RemoteTransferReasonCode::CreditExceeded));
    }
    assert!(existing.exists());
    assert!(bytes > 0);
    Ok(bytes)
}

fn validate_prepared_remote_transfer(
    prepared: &PreparedRemoteTransfer,
    policy: RemoteTransferPolicy,
) -> Result<(), String> {
    validate_canonical_remote_transfer_manifest(&prepared.manifest, policy)?;
    let artifact_ids = prepared
        .manifest
        .manifest
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_id.clone())
        .collect::<BTreeSet<_>>();
    let source_ids = prepared.sources.keys().cloned().collect::<BTreeSet<_>>();
    if source_ids != artifact_ids {
        return Err("remote-transfer-source-binding-mismatch".to_string());
    }
    assert_eq!(prepared.sources.len(), artifact_ids.len());
    assert!(!prepared.sources.is_empty());
    Ok(())
}

fn validate_canonical_remote_transfer_manifest(
    manifest: &CanonicalRemoteTransferManifest,
    policy: RemoteTransferPolicy,
) -> Result<(), String> {
    let rebuilt = canonicalize_remote_transfer_manifest(manifest.manifest.clone(), policy).map_err(reason)?;
    if rebuilt != *manifest {
        return Err(reason(RemoteTransferReasonCode::ManifestIdentityMismatch));
    }
    assert_eq!(rebuilt.digest_blake3, manifest.digest_blake3);
    assert_eq!(rebuilt.canonical_bytes, manifest.canonical_bytes);
    Ok(())
}

fn transfer_lease(
    prepared: &PreparedRemoteTransfer,
    options: RemoteTransferRunOptions,
    scope: &RemoteTransferScope,
) -> Result<RemoteTransferLease, String> {
    if options.lease_expires_unix_s <= options.now_unix_s {
        return Err("remote-transfer-lease-not-future".to_string());
    }
    let count = u32::try_from(prepared.manifest.manifest.artifacts.len())
        .map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    if count > MAX_REMOTE_TRANSFER_LEASE_REFS_HARD {
        return Err("remote-transfer-lease-ref-limit-exceeded".to_string());
    }
    Ok(RemoteTransferLease {
        schema: REMOTE_TRANSFER_LEASE_SCHEMA.to_string(),
        scope: scope.clone(),
        artifact_ids: prepared
            .manifest
            .manifest
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.clone())
            .collect(),
        expires_unix_s: options.lease_expires_unix_s,
        last_progress_step: INITIAL_PROGRESS_STEP,
    })
}

fn persist_transfer_progress(
    state_dir: &Path,
    policy: RemoteTransferPolicy,
    scope: &RemoteTransferScope,
    credit: &RemoteTransferCreditState,
    reused_bytes: u64,
    base_lease: &RemoteTransferLease,
) -> Result<(), String> {
    let checkpoint = seal_remote_transfer_checkpoint(
        RemoteTransferCheckpoint {
            schema: REMOTE_TRANSFER_CHECKPOINT_SCHEMA.to_string(),
            scope: scope.clone(),
            acknowledged_chunk_digests: credit.acknowledged_chunk_digests.clone(),
            next_sequence: credit.next_sequence,
            transferred_bytes: credit.transferred_bytes,
            reused_bytes,
            checkpoint_digest_blake3: RemoteTransferDigest::new("0".repeat(64)).map_err(reason)?,
        },
        policy,
    )
    .map_err(reason)?;
    let mut lease = base_lease.clone();
    lease.last_progress_step = credit.last_progress_step;
    let durable = RemoteTransferDurableState {
        schema: REMOTE_TRANSFER_DURABLE_STATE_SCHEMA.to_string(),
        checkpoint,
        lease,
    };
    save_remote_transfer_state(state_dir, policy, &durable)?;
    Ok(())
}

fn validate_durable_state(state: &RemoteTransferDurableState, policy: RemoteTransferPolicy) -> Result<(), String> {
    if state.schema != REMOTE_TRANSFER_DURABLE_STATE_SCHEMA
        || state.lease.schema != REMOTE_TRANSFER_LEASE_SCHEMA
        || state.checkpoint.schema != REMOTE_TRANSFER_CHECKPOINT_SCHEMA
    {
        return Err(reason(RemoteTransferReasonCode::CheckpointSchemaUnsupported));
    }
    if state.checkpoint.scope != state.lease.scope {
        return Err(reason(RemoteTransferReasonCode::CheckpointScopeMismatch));
    }
    let count = u32::try_from(state.lease.artifact_ids.len())
        .map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    if count > MAX_REMOTE_TRANSFER_LEASE_REFS_HARD
        || state.checkpoint.acknowledged_chunk_digests.len()
            > usize::try_from(policy.chunk_count_max).unwrap_or(usize::MAX)
    {
        return Err(reason(RemoteTransferReasonCode::CheckpointTooLarge));
    }
    assert_eq!(state.checkpoint.scope, state.lease.scope);
    assert!(count <= MAX_REMOTE_TRANSFER_LEASE_REFS_HARD);
    Ok(())
}

fn validate_run_options(options: RemoteTransferRunOptions) -> Result<(), String> {
    if options.lease_expires_unix_s <= options.now_unix_s {
        return Err("remote-transfer-lease-not-future".to_string());
    }
    if options.interrupt_after_chunks == Some(0) {
        return Err("remote-transfer-interrupt-count-zero".to_string());
    }
    assert!(options.lease_expires_unix_s > options.now_unix_s);
    assert_ne!(options.interrupt_after_chunks, Some(0));
    Ok(())
}

fn should_interrupt(limit: Option<u32>, sent_count: usize) -> Result<bool, String> {
    let sent = u32::try_from(sent_count).map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    Ok(limit.is_some_and(|limit| sent >= limit))
}

fn shell_report(
    direction: RemoteTransferDirection,
    disposition: RemoteTransferShellDisposition,
    prepared: &PreparedRemoteTransfer,
    transferred_bytes: u64,
    reused_bytes: u64,
    sent_chunk_digests: Vec<String>,
    checkpoint_path: &Path,
) -> RemoteTransferShellReport {
    let chunks_sent = u32::try_from(sent_chunk_digests.len()).unwrap_or(u32::MAX);
    RemoteTransferShellReport {
        direction,
        disposition,
        manifest_digest_blake3: prepared.manifest.digest_blake3.as_str().to_string(),
        transferred_bytes,
        reused_bytes,
        chunks_sent,
        sent_chunk_digests,
        checkpoint_path: checkpoint_path.display().to_string(),
        output_admission_claimed: false,
    }
}

fn read_prepared_chunk_after_reservation(
    prepared: &PreparedRemoteTransfer,
    header: &RemoteTransferChunkHeader,
) -> Result<Vec<u8>, String> {
    let path = prepared
        .sources
        .get(&header.artifact_id)
        .ok_or_else(|| "remote-transfer-source-binding-missing".to_string())?;
    let capacity =
        usize::try_from(header.chunk.size_bytes).map_err(|_| reason(RemoteTransferReasonCode::ArithmeticOverflow))?;
    let mut bytes = vec![0_u8; capacity];
    let mut file = File::open(path).map_err(|err| format!("opening transfer chunk source: {err}"))?;
    file.seek(SeekFrom::Start(header.chunk.offset_bytes))
        .map_err(|err| format!("seeking transfer chunk source: {err}"))?;
    file.read_exact(&mut bytes).map_err(|err| format!("reading transfer chunk source: {err}"))?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != header.chunk.digest_blake3.as_str() {
        return Err(reason(RemoteTransferReasonCode::ChunkDigestMismatch));
    }
    assert_eq!(bytes.len(), capacity);
    assert!(!bytes.is_empty());
    Ok(bytes)
}

fn persist_verified_chunk(
    receiver_root: &Path,
    descriptor: &RemoteTransferChunkDescriptor,
    bytes: &[u8],
    observed: &RemoteTransferDigest,
) -> Result<(), String> {
    if observed != &descriptor.digest_blake3 {
        return Err(reason(RemoteTransferReasonCode::ChunkDigestMismatch));
    }
    if bytes.len() != usize::try_from(descriptor.size_bytes).unwrap_or(usize::MAX) {
        return Err(reason(RemoteTransferReasonCode::ChunkSizeInvalid));
    }
    write_atomic_bytes(&receiver_chunk_path(receiver_root, observed), bytes)?;
    assert_eq!(observed, &descriptor.digest_blake3);
    assert_eq!(bytes.len(), usize::try_from(descriptor.size_bytes).unwrap_or(usize::MAX));
    Ok(())
}

fn assemble_remote_transfer_artifacts(
    receiver_root: &Path,
    manifest: &CanonicalRemoteTransferManifest,
) -> Result<(), String> {
    for artifact in &manifest.manifest.artifacts {
        if receiver_artifact_is_valid(receiver_root, artifact)? {
            continue;
        }
        let target = receiver_artifact_path(receiver_root, &artifact.artifact_id);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("creating receiver artifact dir: {err}"))?;
        }
        let temporary = target.with_extension(TEMP_FILE_EXTENSION);
        let mut writer = File::create(&temporary).map_err(|err| format!("creating receiver artifact: {err}"))?;
        for chunk in &artifact.chunks {
            let path = receiver_chunk_path(receiver_root, &chunk.digest_blake3);
            if !receiver_chunk_is_valid(receiver_root, chunk)? {
                return Err(reason(RemoteTransferReasonCode::AcknowledgedChunkMissing));
            }
            let mut reader = File::open(&path).map_err(|err| format!("opening receiver chunk: {err}"))?;
            std::io::copy(&mut reader, &mut writer).map_err(|err| format!("assembling receiver artifact: {err}"))?;
        }
        commit_atomic_file(writer, &temporary, &target)?;
        if !receiver_artifact_is_valid(receiver_root, artifact)? {
            let _ = fs::remove_file(&target);
            return Err(reason(RemoteTransferReasonCode::ArtifactDigestConflict));
        }
    }
    assert!(manifest.manifest.artifacts.len() <= usize::try_from(manifest.artifact_count).unwrap_or(usize::MAX));
    assert!(manifest.total_bytes > 0);
    Ok(())
}

fn receiver_chunk_is_valid(root: &Path, descriptor: &RemoteTransferChunkDescriptor) -> Result<bool, String> {
    let path = receiver_chunk_path(root, &descriptor.digest_blake3);
    if !path.exists() {
        return Ok(false);
    }
    let metadata = fs::metadata(&path).map_err(|err| format!("reading receiver chunk metadata: {err}"))?;
    if !metadata.is_file() || metadata.len() != u64::from(descriptor.size_bytes) {
        return Ok(false);
    }
    Ok(hash_file_blake3(&path)? == descriptor.digest_blake3)
}

fn receiver_artifact_is_valid(root: &Path, artifact: &RemoteTransferArtifact) -> Result<bool, String> {
    let path = receiver_artifact_path(root, &artifact.artifact_id);
    if !path.exists() {
        return Ok(false);
    }
    let metadata = fs::metadata(&path).map_err(|err| format!("reading receiver artifact metadata: {err}"))?;
    if !metadata.is_file() || metadata.len() != artifact.size_bytes {
        return Ok(false);
    }
    if hash_file_blake3(&path)? != artifact.digest_blake3 {
        return Ok(false);
    }
    if artifact.artifact_kind == RemoteTransferArtifactKind::Nar {
        let expected =
            artifact.nar_sha256_hex.as_ref().ok_or_else(|| reason(RemoteTransferReasonCode::NarSha256Invalid))?;
        if hash_file_sha256_hex(&path)? != *expected {
            return Ok(false);
        }
    }
    Ok(true)
}

fn receiver_chunk_path(root: &Path, digest: &RemoteTransferDigest) -> PathBuf {
    root.join("chunks").join(digest.as_str())
}

fn receiver_artifact_path(root: &Path, id: &RemoteTransferArtifactId) -> PathBuf {
    root.join("artifacts").join(blake3::hash(id.as_str().as_bytes()).to_hex().to_string())
}

fn hash_file_blake3(path: &Path) -> Result<RemoteTransferDigest, String> {
    let mut file = File::open(path).map_err(|err| format!("opening content for BLAKE3: {err}"))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; usize::try_from(COPY_BUFFER_BYTES).unwrap_or(1)];
    loop {
        let read_bytes = file.read(&mut buffer).map_err(|err| format!("reading content for BLAKE3: {err}"))?;
        if read_bytes == 0 {
            break;
        }
        hasher.update(&buffer[..read_bytes]);
    }
    RemoteTransferDigest::new(hasher.finalize().to_hex().to_string()).map_err(reason)
}

fn hash_file_sha256_hex(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|err| format!("opening content for SHA-256: {err}"))?;
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    let mut buffer = vec![0_u8; usize::try_from(COPY_BUFFER_BYTES).unwrap_or(1)];
    loop {
        let read_bytes = file.read(&mut buffer).map_err(|err| format!("reading content for SHA-256: {err}"))?;
        if read_bytes == 0 {
            break;
        }
        sha2::Digest::update(&mut hasher, &buffer[..read_bytes]);
    }
    Ok(data_encoding::HEXLOWER.encode(&sha2::Digest::finalize(hasher)))
}

fn verify_nar_sha256(path: &Path, path_info: &PathInfo) -> Result<(), String> {
    if hash_file_sha256_hex(path)? != data_encoding::HEXLOWER.encode(&path_info.nar_sha256) {
        return Err("remote-transfer-nar-sha256-mismatch".to_string());
    }
    assert_eq!(fs::metadata(path).map_err(|err| err.to_string())?.len(), path_info.nar_size);
    assert!(!path_info.nar_sha256.iter().all(|byte| *byte == 0));
    Ok(())
}

fn copy_file_bounded(source: &Path, target: &Path, max_bytes: u64) -> Result<(), String> {
    let metadata = fs::metadata(source).map_err(|err| format!("reading source metadata: {err}"))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_bytes {
        return Err(reason(RemoteTransferReasonCode::TotalBytesExceeded));
    }
    let mut reader = File::open(source).map_err(|err| format!("opening bounded source: {err}"))?;
    let temporary = target.with_extension(TEMP_FILE_EXTENSION);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("creating bounded target dir: {err}"))?;
    }
    let mut writer = File::create(&temporary).map_err(|err| format!("creating bounded target: {err}"))?;
    let copied = std::io::copy(&mut std::io::Read::by_ref(&mut reader).take(max_bytes.saturating_add(1)), &mut writer)
        .map_err(|err| format!("copying bounded source: {err}"))?;
    if copied != metadata.len() || copied > max_bytes {
        let _ = fs::remove_file(&temporary);
        return Err(reason(RemoteTransferReasonCode::TotalBytesExceeded));
    }
    commit_atomic_file(writer, &temporary, target)
}

fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("creating atomic parent {}: {err}", parent.display()))?;
    }
    let temporary = path.with_extension(TEMP_FILE_EXTENSION);
    let mut file =
        File::create(&temporary).map_err(|err| format!("creating atomic temp {}: {err}", temporary.display()))?;
    file.write_all(bytes).map_err(|err| format!("writing atomic temp: {err}"))?;
    commit_atomic_file(file, &temporary, path)
}

fn commit_atomic_file(mut file: File, temporary: &Path, target: &Path) -> Result<(), String> {
    file.flush().map_err(|err| format!("flushing atomic temp: {err}"))?;
    file.sync_all().map_err(|err| format!("syncing atomic temp: {err}"))?;
    drop(file);
    fs::rename(temporary, target).map_err(|err| format!("committing atomic file {}: {err}", target.display()))?;
    if let Some(parent) = target.parent() {
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|err| format!("syncing atomic parent {}: {err}", parent.display()))?;
    }
    assert!(target.exists());
    assert!(!temporary.exists());
    Ok(())
}

fn invalidate_transfer_state_file(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|err| format!("invalidating stale transfer state: {err}"))?;
    assert!(!path.exists());
    assert!(path.extension().is_some_and(|extension| extension == "json"));
    Ok(())
}

fn reason(code: RemoteTransferReasonCode) -> String {
    code.as_str().to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteTransferFallbackReason {
    DeltaTransferFailed,
    DeltaUnavailable,
    FullNarUnavailable,
}

impl RemoteTransferFallbackReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeltaTransferFailed => "delta-transfer-failed",
            Self::DeltaUnavailable => "delta-unavailable",
            Self::FullNarUnavailable => "full-nar-unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTransferFallbackReport {
    pub transfer: RemoteTransferShellReport,
    pub fallback_reason: Option<RemoteTransferFallbackReason>,
}

/// Try ordinary delta identities first, then an ordinary full-NAR manifest.
/// Both paths retain the same digest, closure, PathInfo, and admission gates.
pub fn execute_delta_with_full_nar_fallback(
    delta: Option<&PreparedRemoteTransfer>,
    full_nar: Option<&PreparedRemoteTransfer>,
    policy: RemoteTransferPolicy,
    state_dir: &Path,
    options: RemoteTransferRunOptions,
) -> Result<RemoteTransferFallbackReport, String> {
    policy.validate().map_err(reason)?;
    if let Some(delta) = delta {
        let receiver = remote_transfer_receiver_root(state_dir, &delta.manifest.manifest.session_id);
        match execute_prepared_remote_transfer(delta, policy, state_dir, &receiver, options) {
            Ok(transfer) => {
                return Ok(RemoteTransferFallbackReport {
                    transfer,
                    fallback_reason: None,
                });
            }
            Err(delta_error) => {
                let full_nar = full_nar.ok_or_else(|| {
                    format!("{}:{}", RemoteTransferFallbackReason::FullNarUnavailable.as_str(), delta_error)
                })?;
                let receiver = remote_transfer_receiver_root(state_dir, &full_nar.manifest.manifest.session_id);
                let transfer = execute_prepared_remote_transfer(full_nar, policy, state_dir, &receiver, options)?;
                return Ok(RemoteTransferFallbackReport {
                    transfer,
                    fallback_reason: Some(RemoteTransferFallbackReason::DeltaTransferFailed),
                });
            }
        }
    }
    let full_nar = full_nar.ok_or_else(|| RemoteTransferFallbackReason::FullNarUnavailable.as_str().to_string())?;
    let receiver = remote_transfer_receiver_root(state_dir, &full_nar.manifest.manifest.session_id);
    let transfer = execute_prepared_remote_transfer(full_nar, policy, state_dir, &receiver, options)?;
    assert!(!transfer.output_admission_claimed);
    assert!(delta.is_none());
    Ok(RemoteTransferFallbackReport {
        transfer,
        fallback_reason: Some(RemoteTransferFallbackReason::DeltaUnavailable),
    })
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    const TEST_DATA_BYTES: usize = 150_000;
    const TEST_LARGE_OUTPUT_BYTES: usize = 8_388_608;
    const TEST_PATTERN_MODULUS: usize = 251;
    const TEST_FILE_BUFFER_BYTES: usize = 65_536;
    const TEST_NOW_UNIX_S: u64 = 1_000;
    const TEST_LEASE_EXPIRES_UNIX_S: u64 = 2_000;
    const TEST_EXPIRED_UNIX_S: u64 = 2_001;
    const CHILD_SENTINEL_ENV: &str = "MANTLE_REMOTE_TRANSFER_RESUME_CHILD";
    const CHILD_ROOT_ENV: &str = "MANTLE_REMOTE_TRANSFER_RESUME_ROOT";
    const CHILD_DIRECTION_ENV: &str = "MANTLE_REMOTE_TRANSFER_RESUME_DIRECTION";
    const CHILD_DATA_BYTES_ENV: &str = "MANTLE_REMOTE_TRANSFER_RESUME_DATA_BYTES";
    const CHILD_DOWNLOAD_DIRECTION: &str = "download";
    const CHILD_REPORT_FILE: &str = "child-report.json";

    fn write_test_pattern(path: &Path, total_bytes: usize) {
        let mut file = File::create(path).unwrap();
        let mut buffer = [0_u8; TEST_FILE_BUFFER_BYTES];
        let mut offset = 0_usize;
        while offset < total_bytes {
            let write_bytes = std::cmp::min(TEST_FILE_BUFFER_BYTES, total_bytes - offset);
            for (index, byte) in buffer[..write_bytes].iter_mut().enumerate() {
                *byte = u8::try_from((offset + index) % TEST_PATTERN_MODULUS).unwrap();
            }
            file.write_all(&buffer[..write_bytes]).unwrap();
            offset += write_bytes;
        }
        file.sync_all().unwrap();
        assert_eq!(fs::metadata(path).unwrap().len(), u64::try_from(total_bytes).unwrap());
        assert!(total_bytes > 0);
    }

    fn admitted_options(interrupt_after_chunks: Option<u32>) -> RemoteTransferRunOptions {
        RemoteTransferRunOptions {
            direction: RemoteTransferDirection::Upload,
            interrupt_after_chunks,
            now_unix_s: TEST_NOW_UNIX_S,
            lease_expires_unix_s: TEST_LEASE_EXPIRES_UNIX_S,
            admission: RemoteTransferAdmissionFacts {
                required_closure_metadata_verified: true,
                path_info_admitted: true,
            },
        }
    }

    fn prepared_fixture(root: &Path) -> PreparedRemoteTransfer {
        prepared_fixture_with_size(root, TEST_DATA_BYTES)
    }

    fn prepared_fixture_with_size(root: &Path, data_bytes: usize) -> PreparedRemoteTransfer {
        fs::create_dir_all(root).unwrap();
        let source = root.join("source.bin");
        write_test_pattern(&source, data_bytes);
        let policy = RemoteTransferPolicy::default();
        let artifact = prepare_file_transfer_artifact(
            RemoteTransferArtifactId::new("fixture:root").unwrap(),
            RemoteTransferArtifactKind::SourceBundle,
            &source,
            true,
            None,
            policy,
        )
        .unwrap();
        let requested_content_blake3 = artifact.descriptor.digest_blake3.clone();
        prepare_remote_transfer(
            RemoteTransferBinding {
                job_id: RemoteJobId::new("job-resume").unwrap(),
                attempt_id: RemoteAttemptId::new("attempt-resume").unwrap(),
                fence_generation: RemoteFenceGeneration::INITIAL,
                store_prefix: "/mantle/store".to_string(),
                requested_content_blake3,
            },
            policy,
            vec![artifact],
        )
        .unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn socket_data_plane_reserves_receiver_credit_before_chunk_allocation() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let demand = plan_remote_transfer_demand(&prepared.manifest, &RemoteTransferReceiverFacts::default()).unwrap();
        let missing = demand.missing_chunks[0].clone();
        let policy = RemoteTransferPolicy::default();
        let (sender_socket, receiver_socket) = std::os::unix::net::UnixStream::pair().unwrap();
        let sender_prepared = prepared.clone();
        let sender_demand = demand.clone();
        let sender_missing = missing.clone();
        let receiver_root = root.path().join("socket-receiver");
        let (sender_state, receiver_state, acknowledgement) = std::thread::scope(|scope| {
            let sender = scope.spawn(move || {
                write_remote_transfer_data_chunk(
                    sender_socket,
                    &sender_prepared,
                    policy,
                    &sender_demand,
                    &RemoteTransferCreditState::default(),
                    RemoteTransferCreditGrant {
                        bytes: u64::from(sender_missing.chunk.size_bytes),
                        chunks: FIRST_CHUNK_COUNT,
                    },
                    &sender_missing,
                )
                .unwrap()
            });
            let (receiver_state, acknowledgement) = receive_remote_transfer_data_chunk(
                receiver_socket,
                &receiver_root,
                &prepared.manifest,
                policy,
                &demand,
                &RemoteTransferCreditState::default(),
                1,
            )
            .unwrap();
            (sender.join().unwrap(), receiver_state, acknowledgement)
        });
        let observed = acknowledgement.chunk_digest_blake3.clone();
        let acknowledged_sender =
            acknowledge_remote_transfer_chunk(policy, &sender_state, &acknowledgement, &observed, 1).unwrap();
        assert_eq!(acknowledged_sender.acknowledged_chunk_digests, receiver_state.acknowledged_chunk_digests,);
        assert!(receiver_chunk_path(&receiver_root, &observed).is_file());
    }

    #[test]
    fn interruption_persists_fenced_state_and_resume_sends_only_missing_chunks() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let state_dir = root.path().join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        let first = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(Some(FIRST_CHUNK_COUNT)),
        )
        .unwrap();
        assert_eq!(first.disposition, RemoteTransferShellDisposition::Interrupted);
        assert_eq!(first.chunks_sent, FIRST_CHUNK_COUNT);
        assert!(Path::new(&first.checkpoint_path).is_file());

        let second = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(None),
        )
        .unwrap();
        assert_eq!(second.disposition, RemoteTransferShellDisposition::Completed);
        assert_eq!(second.chunks_sent, prepared.manifest.chunk_count - FIRST_CHUNK_COUNT);
        assert!(!second.sent_chunk_digests.iter().any(|digest| first.sent_chunk_digests.contains(digest)));
        assert!(!second.output_admission_claimed);

        let cutoff = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(None),
        )
        .unwrap();
        assert_eq!(cutoff.disposition, RemoteTransferShellDisposition::AlreadyPresent);
        assert_eq!(cutoff.transferred_bytes, 0);
        assert_eq!(cutoff.reused_bytes, prepared.manifest.total_bytes);
    }

    #[test]
    fn tampered_acknowledged_chunk_fails_closed_on_resume() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let state_dir = root.path().join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        let first = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(Some(FIRST_CHUNK_COUNT)),
        )
        .unwrap();
        let digest = RemoteTransferDigest::new(first.sent_chunk_digests[0].clone()).unwrap();
        fs::write(receiver_chunk_path(&receiver, &digest), b"tampered").unwrap();
        let error = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(None),
        )
        .unwrap_err();
        assert_eq!(error, RemoteTransferReasonCode::AcknowledgedChunkMissing.as_str());
        assert_ne!(error, RemoteTransferReasonCode::TransferDemandSatisfied.as_str());
    }

    #[test]
    fn forged_manifest_and_concurrent_session_writer_fail_before_progress() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let state_dir = root.path().join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        let mut forged = prepared.clone();
        forged.manifest.manifest.store_prefix = "/forged/store".to_string();
        let error = execute_prepared_remote_transfer(
            &forged,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(None),
        )
        .unwrap_err();
        assert_eq!(error, RemoteTransferReasonCode::ManifestIdentityMismatch.as_str());
        assert!(!state_dir.exists());

        let scope = remote_transfer_scope(&prepared.manifest);
        let guard = acquire_remote_transfer_session_lock(&state_dir, &scope.session_id).unwrap();
        let error = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(None),
        )
        .unwrap_err();
        assert!(error.starts_with("remote-transfer-session-lock-busy:"));
        assert!(!receiver.exists());
        drop(guard);
    }

    #[test]
    fn stale_scope_and_expired_lease_are_invalidated_before_reuse() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let state_dir = root.path().join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(Some(FIRST_CHUNK_COUNT)),
        )
        .unwrap();
        let expected = remote_transfer_scope(&prepared.manifest);
        let path = remote_transfer_state_path(&state_dir, &expected.session_id);
        let original = fs::read(&path).unwrap();
        let mut stale: RemoteTransferDurableState = serde_json::from_slice(&original).unwrap();
        stale.checkpoint.scope.fence_generation = RemoteFenceGeneration::new(2).unwrap();
        fs::write(&path, serde_json::to_vec(&stale).unwrap()).unwrap();
        assert_eq!(
            load_remote_transfer_state(&state_dir, &expected, RemoteTransferPolicy::default(), TEST_NOW_UNIX_S,)
                .unwrap(),
            RemoteTransferStateLoad::StaleInvalidated,
        );
        assert!(!path.exists());

        fs::write(&path, original).unwrap();
        assert_eq!(
            load_remote_transfer_state(&state_dir, &expected, RemoteTransferPolicy::default(), TEST_EXPIRED_UNIX_S,)
                .unwrap(),
            RemoteTransferStateLoad::ExpiredInvalidated,
        );
        assert!(!path.exists());
    }

    #[test]
    fn completion_without_admission_remains_an_explicit_non_claim() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let state_dir = root.path().join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        let mut options = admitted_options(None);
        options.admission.path_info_admitted = false;
        let report = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            options,
        )
        .unwrap();
        assert_eq!(report.disposition, RemoteTransferShellDisposition::AwaitingAdmission);
        assert!(!report.output_admission_claimed);
        assert_eq!(report.transferred_bytes, prepared.manifest.total_bytes);
    }

    #[test]
    fn oversized_control_and_total_bytes_reject_before_receiver_persistence() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let demand = plan_remote_transfer_demand(&prepared.manifest, &RemoteTransferReceiverFacts::default()).unwrap();
        let policy = RemoteTransferPolicy::default();
        let oversized_header = policy.control_bytes_max.checked_add(1).unwrap().to_be_bytes();
        let receiver = root.path().join("never-created-receiver");
        let error = receive_remote_transfer_data_chunk(
            std::io::Cursor::new(oversized_header),
            &receiver,
            &prepared.manifest,
            policy,
            &demand,
            &RemoteTransferCreditState::default(),
            1,
        )
        .unwrap_err();
        assert_eq!(error, RemoteTransferReasonCode::ManifestBoundsExceeded.as_str());
        assert!(!receiver.exists());

        let mut tiny_total_policy = policy;
        tiny_total_policy.total_bytes_max = 1;
        let source = root.path().join("source.bin");
        let error = prepare_file_transfer_artifact(
            RemoteTransferArtifactId::new("source:over-total").unwrap(),
            RemoteTransferArtifactKind::SourceBundle,
            &source,
            true,
            None,
            tiny_total_policy,
        )
        .unwrap_err();
        assert_eq!(error, RemoteTransferReasonCode::TotalBytesExceeded.as_str());
        assert!(!receiver.exists());
    }

    #[test]
    fn inline_payload_capability_rejects_oversized_data_before_write() {
        let root = tempfile::tempdir().unwrap();
        let oversized = vec![0_u8; usize::try_from(MAX_REMOTE_INLINE_FIXTURE_BOOTSTRAP_BYTES + 1).unwrap()];
        let error = prepare_inline_fixture_or_bootstrap_artifact(
            RemoteInlinePayloadCapability::Fixture,
            RemoteTransferArtifactId::new("fixture:oversized").unwrap(),
            RemoteTransferArtifactKind::SourceBundle,
            &oversized,
            root.path(),
            true,
            RemoteTransferPolicy::default(),
        )
        .unwrap_err();
        assert_eq!(error, "remote-inline-fixture-bootstrap-payload-too-large");
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn castore_blob_and_directory_adapters_preserve_existing_identities() {
        let root = tempfile::tempdir().unwrap();
        let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig {
            state_dir: root.path().join("state"),
            output_dir: root.path().join("store"),
            remote_cache_urls: Vec::new(),
            fallback_mode: crunch_store::StoreFallbackMode::Practical,
            store_dir: "/mantle/store".to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .unwrap();
        let blob_bytes = b"castore-transfer";
        let mut writer = store.blob_service().open_write().await;
        writer.write_all(blob_bytes).await.unwrap();
        let blob_digest = writer.close().await.unwrap();
        let directory_digest = store.directory_service().put(snix_castore::Directory::new()).await.unwrap();
        let node = snix_castore::Node::File {
            digest: blob_digest,
            size: u64::try_from(blob_bytes.len()).unwrap(),
            executable: false,
        };
        let nar_fixture = root.path().join("fixture.nar");
        let mut nar_writer = tokio::fs::File::create(&nar_fixture).await.unwrap();
        store.render_nar(&node, &mut nar_writer).await.unwrap();
        nar_writer.flush().await.unwrap();
        drop(nar_writer);
        let nar_bytes = fs::read(&nar_fixture).unwrap();
        let nar_sha256: [u8; 32] = <sha2::Sha256 as sha2::Digest>::digest(&nar_bytes).into();
        let path_info = PathInfo {
            store_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed(
                "remote-transfer-fixture",
                [7_u8; 20],
            )
            .unwrap(),
            node,
            references: Vec::new(),
            nar_sha256,
            nar_size: u64::try_from(nar_bytes.len()).unwrap(),
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        let spool = root.path().join("spool");
        let policy = RemoteTransferPolicy::default();
        let blob = prepare_castore_blob_transfer_artifact(&store, blob_digest, &spool, true, policy).await.unwrap();
        let directory = prepare_castore_directory_transfer_artifact(&store, directory_digest, &spool, true, policy)
            .await
            .unwrap();
        let nar = prepare_nar_transfer_artifact(
            &store,
            &path_info,
            RemoteTransferArtifactId::new("nar:fixture").unwrap(),
            &spool,
            true,
            policy,
        )
        .await
        .unwrap();
        let pathinfo = prepare_pathinfo_transfer_artifact(&path_info, &spool, true, policy).unwrap();
        let attestation_bytes = b"canonical-attestation";
        let attestation_digest = blake3::hash(attestation_bytes).to_hex().to_string();
        let attestation =
            prepare_attestation_transfer_artifact(&attestation_digest, attestation_bytes, &spool, true, policy)
                .unwrap();
        let source_path = root.path().join("source-bundle");
        fs::write(&source_path, b"source-bundle-bytes").unwrap();
        let source = prepare_source_bundle_transfer_artifact("source-id", &source_path, &spool, true, policy).unwrap();
        let delta = prepare_delta_transfer_artifacts(
            &[crunch_delta::DeltaTransferFrame::Blob {
                digest: blob_digest,
                bytes: blob_bytes.to_vec(),
            }],
            &spool,
            policy,
        )
        .unwrap();

        assert_eq!(blob.descriptor.artifact_kind, RemoteTransferArtifactKind::CastoreBlob);
        assert_eq!(directory.descriptor.artifact_kind, RemoteTransferArtifactKind::CastoreDirectory);
        assert_eq!(nar.descriptor.artifact_kind, RemoteTransferArtifactKind::Nar);
        assert_eq!(pathinfo.descriptor.artifact_kind, RemoteTransferArtifactKind::PathInfo);
        assert_eq!(attestation.descriptor.artifact_kind, RemoteTransferArtifactKind::Attestation);
        assert_eq!(source.descriptor.artifact_kind, RemoteTransferArtifactKind::SourceBundle);
        assert_eq!(delta[0].descriptor.artifact_kind, RemoteTransferArtifactKind::DeltaBlob);
        assert!(blob.descriptor.artifact_id.as_str().contains(blob.descriptor.digest_blake3.as_str()));
        assert!(directory.descriptor.artifact_id.as_str().contains(directory.descriptor.digest_blake3.as_str()));
    }

    #[test]
    fn download_resume_and_delta_to_full_fallback_keep_admission_separate() {
        let root = tempfile::tempdir().unwrap();
        let delta_root = root.path().join("delta");
        let delta = prepared_fixture(&delta_root);
        fs::write(delta_root.join("source.bin"), b"tampered-after-manifest").unwrap();
        let full = prepared_fixture(&root.path().join("full"));
        let mut options = admitted_options(Some(FIRST_CHUNK_COUNT));
        options.direction = RemoteTransferDirection::Download;
        let state_dir = root.path().join("state");
        let interrupted = execute_delta_with_full_nar_fallback(
            Some(&delta),
            Some(&full),
            RemoteTransferPolicy::default(),
            &state_dir,
            options,
        )
        .unwrap();
        assert_eq!(interrupted.fallback_reason, Some(RemoteTransferFallbackReason::DeltaTransferFailed),);
        assert_eq!(interrupted.transfer.direction, RemoteTransferDirection::Download);
        assert_eq!(interrupted.transfer.disposition, RemoteTransferShellDisposition::Interrupted,);

        options.interrupt_after_chunks = None;
        let completed = execute_delta_with_full_nar_fallback(
            None,
            Some(&full),
            RemoteTransferPolicy::default(),
            &state_dir,
            options,
        )
        .unwrap();
        assert_eq!(completed.transfer.disposition, RemoteTransferShellDisposition::Completed);
        assert_eq!(completed.fallback_reason, Some(RemoteTransferFallbackReason::DeltaUnavailable),);
        assert!(!completed.transfer.output_admission_claimed);
        let production_report =
            crate::remote_build::streaming_transfer_report_from_runtime(&completed.transfer, "builder-key").unwrap();
        assert_eq!(production_report.mode, crate::remote_build::RemoteTransferMode::Streaming);
        assert_eq!(production_report.reused_bytes, completed.transfer.reused_bytes);
    }

    #[test]
    fn resume_across_process_uses_durable_checkpoint_and_receiver_facts() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(root.path());
        let state_dir = root.path().join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        let first = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            admitted_options(Some(FIRST_CHUNK_COUNT)),
        )
        .unwrap();
        assert_eq!(first.chunks_sent, FIRST_CHUNK_COUNT);
        let status = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("remote_transfer::tests::resume_child_process")
            .arg("--nocapture")
            .env(CHILD_SENTINEL_ENV, "1")
            .env(CHILD_ROOT_ENV, root.path())
            .status()
            .unwrap();
        assert!(status.success());
        let report: RemoteTransferShellReport =
            serde_json::from_slice(&fs::read(root.path().join(CHILD_REPORT_FILE)).unwrap()).unwrap();
        assert_eq!(report.disposition, RemoteTransferShellDisposition::Completed);
        assert_eq!(report.chunks_sent, prepared.manifest.chunk_count - FIRST_CHUNK_COUNT);
    }

    #[test]
    fn download_resume_across_process_reuses_verified_receiver_chunks() {
        let root = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture_with_size(root.path(), TEST_LARGE_OUTPUT_BYTES);
        let state_dir = root.path().join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        let mut options = admitted_options(Some(FIRST_CHUNK_COUNT));
        options.direction = RemoteTransferDirection::Download;
        let first = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            options,
        )
        .unwrap();
        assert_eq!(first.disposition, RemoteTransferShellDisposition::Interrupted);
        let status = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("remote_transfer::tests::resume_child_process")
            .arg("--nocapture")
            .env(CHILD_SENTINEL_ENV, "1")
            .env(CHILD_ROOT_ENV, root.path())
            .env(CHILD_DIRECTION_ENV, CHILD_DOWNLOAD_DIRECTION)
            .env(CHILD_DATA_BYTES_ENV, TEST_LARGE_OUTPUT_BYTES.to_string())
            .status()
            .unwrap();
        assert!(status.success());
        let report: RemoteTransferShellReport =
            serde_json::from_slice(&fs::read(root.path().join(CHILD_REPORT_FILE)).unwrap()).unwrap();
        assert_eq!(report.direction, RemoteTransferDirection::Download);
        assert_eq!(report.disposition, RemoteTransferShellDisposition::Completed);
        assert!(!report.sent_chunk_digests.iter().any(|digest| first.sent_chunk_digests.contains(digest)));
    }

    #[test]
    fn resume_child_process() {
        if std::env::var_os(CHILD_SENTINEL_ENV).is_none() {
            return;
        }
        let root = PathBuf::from(std::env::var_os(CHILD_ROOT_ENV).unwrap());
        let data_bytes = std::env::var(CHILD_DATA_BYTES_ENV)
            .ok()
            .map(|value| value.parse::<usize>().unwrap())
            .unwrap_or(TEST_DATA_BYTES);
        let prepared = prepared_fixture_with_size(&root, data_bytes);
        let state_dir = root.join("state");
        let receiver = remote_transfer_receiver_root(&state_dir, &prepared.manifest.manifest.session_id);
        let mut options = admitted_options(None);
        if std::env::var(CHILD_DIRECTION_ENV).as_deref() == Ok(CHILD_DOWNLOAD_DIRECTION) {
            options.direction = RemoteTransferDirection::Download;
        }
        let report = execute_prepared_remote_transfer(
            &prepared,
            RemoteTransferPolicy::default(),
            &state_dir,
            &receiver,
            options,
        )
        .unwrap();
        fs::write(root.join(CHILD_REPORT_FILE), serde_json::to_vec(&report).unwrap()).unwrap();
        assert_eq!(report.disposition, RemoteTransferShellDisposition::Completed);
        assert!(!report.output_admission_claimed);
    }
}
