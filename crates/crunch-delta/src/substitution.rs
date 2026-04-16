use std::collections::HashMap;
use std::path::Path;

use crunch_store::persist_artifact_attestation;
use nix_compat::narinfo::Signature;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use snix_castore::B3Digest;
use snix_castore::Node;
use snix_store::path_info::PathInfo;

use crate::BenchCase;
use crate::ChunkProfileWire;
use crate::ClosureFixture;
use crate::ManifestError;
use crate::NegotiatedProtocol;
use crate::NegotiationError;
use crate::NegotiationOffer;
use crate::OutputFixture;
use crate::ReceiverManifest;
use crate::build_receiver_manifest;
use crate::chunk_profile_wire_v1;
use crate::negotiate_protocol;

pub const MAX_ACTIVE_DIRECTORY_WINDOWS: u32 = 8;
pub const MAX_ACTIVE_BLOB_WINDOWS: u32 = 8;
pub const MAX_ACTIVE_CHUNK_WINDOWS: u32 = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaHttpEndpoints {
    pub capability_path: String,
    pub candidate_path: String,
    pub has_set_path: String,
    pub stream_path: String,
}

impl DeltaHttpEndpoints {
    pub fn under_cache_authority(authority_prefix: &str) -> Self {
        assert!(!authority_prefix.is_empty(), "authority prefix must not be empty");
        assert!(authority_prefix.starts_with('/'), "authority prefix must be absolute");
        let base = authority_prefix.trim_end_matches('/');
        Self {
            capability_path: format!("{base}/delta/capabilities"),
            candidate_path: format!("{base}/delta/request"),
            has_set_path: format!("{base}/delta/has-set"),
            stream_path: format!("{base}/delta/stream"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaCapabilityAdvertisement {
    pub supported_versions: Vec<u32>,
    pub supported_chunk_profiles: Vec<ChunkProfileWire>,
    pub endpoints: DeltaHttpEndpoints,
}

impl DeltaCapabilityAdvertisement {
    pub fn to_negotiation_offer(&self) -> NegotiationOffer {
        NegotiationOffer {
            supported_versions: self.supported_versions.clone(),
            supported_chunk_profiles: self.supported_chunk_profiles.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaFetchRequest {
    pub logical_path: String,
    pub output_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaCandidateResponse {
    pub session_id: String,
    pub negotiated: NegotiatedProtocol,
    pub sender: ClosureFixture,
    pub output_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaReceiverHasSet {
    pub manifest: ReceiverManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaTransferFrame {
    Blob {
        digest: B3Digest,
        bytes: Vec<u8>,
    },
    Chunk {
        parent_digest: B3Digest,
        chunk_digest: B3Digest,
        chunk_index: u32,
        bytes: Vec<u8>,
    },
    FinalPathInfo {
        path_info: PathInfo,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaFallbackReason {
    CapabilityUnavailable,
    NegotiationFailed,
    ReuseNotAvailable,
    UntrustedDeltaPathInfo,
    LegacyCache,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaAcceptanceMode {
    Delta,
    FullArtifactFallback(DeltaFallbackReason),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeltaTransferStats {
    pub transferred_bytes: u64,
    pub peak_active_directory_windows: u32,
    pub peak_active_blob_windows: u32,
    pub peak_active_chunk_windows: u32,
    pub peak_memory_bytes: u64,
    pub streamed_blob_count: u32,
    pub streamed_chunk_count: u32,
}

impl DeltaTransferStats {
    fn record_blob(&mut self, bytes_len: usize) {
        self.streamed_blob_count = self.streamed_blob_count.saturating_add(1);
        self.transferred_bytes = self.transferred_bytes.saturating_add(bytes_len as u64);
        self.peak_active_blob_windows = self.peak_active_blob_windows.max(1);
        self.peak_memory_bytes = self.peak_memory_bytes.max(bytes_len as u64);
    }

    fn record_chunk(&mut self, bytes_len: usize) {
        self.streamed_chunk_count = self.streamed_chunk_count.saturating_add(1);
        self.transferred_bytes = self.transferred_bytes.saturating_add(bytes_len as u64);
        self.peak_active_chunk_windows = self.peak_active_chunk_windows.max(1);
        self.peak_memory_bytes = self.peak_memory_bytes.max(bytes_len as u64);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaFetchOutcome {
    pub acceptance: DeltaAcceptanceMode,
    pub path_info: PathInfo,
    pub transferred_bytes: u64,
    pub full_transfer_bytes: u64,
    pub attestation_digest: String,
    pub stats: DeltaTransferStats,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetainedContentStore {
    pub full_blobs: HashMap<B3Digest, Vec<u8>>,
    pub chunks: HashMap<B3Digest, Vec<u8>>,
}

impl RetainedContentStore {
    pub fn retain_blob(&mut self, digest: B3Digest, bytes: Vec<u8>) {
        self.full_blobs.insert(digest, bytes);
    }

    pub fn retain_chunk(&mut self, digest: B3Digest, bytes: Vec<u8>) {
        self.chunks.insert(digest, bytes);
    }

    fn has_blob(&self, digest: &B3Digest) -> bool {
        self.full_blobs.contains_key(digest)
    }

    fn has_chunk(&self, digest: &B3Digest) -> bool {
        self.chunks.contains_key(digest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeltaReceiverState {
    pub receiver_store: ClosureFixture,
    pub retained: RetainedContentStore,
}

impl Default for DeltaReceiverState {
    fn default() -> Self {
        Self {
            receiver_store: ClosureFixture {
                store_prefix: "/nix/store".to_owned(),
                outputs: Vec::new(),
            },
            retained: RetainedContentStore::default(),
        }
    }
}

impl DeltaReceiverState {
    pub async fn build_manifest_for(&self, sender: &ClosureFixture) -> Result<ReceiverManifest, ManifestError> {
        let case = BenchCase {
            name: "delta-substitution",
            sender: sender.clone(),
            receiver: ReceiverManifest::new(&sender.store_prefix),
            receiver_store: self.receiver_store.clone(),
            receiver_frontiers: Vec::new(),
            receiver_lossy_frontiers: Vec::new(),
            receiver_probabilistic_frontiers: Vec::new(),
            frontier_complete_outputs: std::collections::HashSet::new(),
            expected_full_bytes: sender.full_transfer_bytes(),
            expected_coarse_bytes: sender.full_transfer_bytes(),
        };
        let mut manifest = build_receiver_manifest(&case).await?.manifest;
        for digest in self.retained.full_blobs.keys() {
            manifest.known_blobs.insert(*digest);
        }
        for digest in self.retained.chunks.keys() {
            manifest.known_chunks.insert(*digest);
        }
        Ok(manifest)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentCatalog {
    pub full_blobs: HashMap<B3Digest, Vec<u8>>,
    pub chunks: HashMap<B3Digest, Vec<Vec<u8>>>,
}

impl ContentCatalog {
    pub fn insert_blob(&mut self, digest: B3Digest, bytes: Vec<u8>) {
        self.full_blobs.insert(digest, bytes);
    }

    pub fn insert_chunks(&mut self, digest: B3Digest, chunks: Vec<Vec<u8>>) {
        self.chunks.insert(digest, chunks);
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryDeltaAuthority {
    pub authority_prefix: String,
    pub support_delta: bool,
    pub sender: ClosureFixture,
    pub catalog: ContentCatalog,
    pub output_name: String,
    pub delta_path_info: PathInfo,
    pub full_artifact_path_info: PathInfo,
}

impl InMemoryDeltaAuthority {
    pub fn capabilities(&self) -> Option<DeltaCapabilityAdvertisement> {
        if !self.support_delta {
            return None;
        }
        Some(DeltaCapabilityAdvertisement {
            supported_versions: vec![crate::PROTOCOL_VERSION_V1],
            supported_chunk_profiles: vec![chunk_profile_wire_v1()],
            endpoints: DeltaHttpEndpoints::under_cache_authority(&self.authority_prefix),
        })
    }

    pub fn request_candidates(
        &self,
        request: &DeltaFetchRequest,
        client_offer: &NegotiationOffer,
    ) -> Result<Option<DeltaCandidateResponse>, DeltaSubstitutionError> {
        let Some(capabilities) = self.capabilities() else {
            return Ok(None);
        };
        if request.output_name != self.output_name {
            return Ok(None);
        }
        let negotiated = negotiate_protocol(client_offer, &capabilities.to_negotiation_offer())
            .map_err(DeltaSubstitutionError::Negotiation)?;
        Ok(Some(DeltaCandidateResponse {
            session_id: format!("session-{}", self.output_name),
            negotiated,
            sender: self.sender.clone(),
            output_name: self.output_name.clone(),
        }))
    }

    pub fn full_artifact_fetch(&self, request: &DeltaFetchRequest) -> Result<Option<PathInfo>, DeltaSubstitutionError> {
        if request.output_name != self.output_name {
            return Ok(None);
        }
        Ok(Some(self.full_artifact_path_info.clone()))
    }

    pub fn stream_missing<F>(
        &self,
        session_id: &str,
        has_set: &DeltaReceiverHasSet,
        mut on_frame: F,
    ) -> Result<DeltaTransferStats, DeltaSubstitutionError>
    where
        F: FnMut(DeltaTransferFrame) -> Result<(), DeltaSubstitutionError>,
    {
        assert!(!session_id.is_empty(), "session id must not be empty");
        let mut stats = DeltaTransferStats::default();
        stats.peak_active_directory_windows = 1.min(MAX_ACTIVE_DIRECTORY_WINDOWS);

        for output in &self.sender.outputs {
            if has_set.manifest.known_outputs.contains(&output.output_id) {
                continue;
            }
            stream_missing_node(&output.root, &has_set.manifest, &self.catalog, &mut stats, &mut on_frame)?;
        }

        on_frame(DeltaTransferFrame::FinalPathInfo {
            path_info: self.delta_path_info.clone(),
        })?;
        Ok(stats)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeltaSubstitutionError {
    #[error("negotiation failed: {0}")]
    Negotiation(NegotiationError),
    #[error("receiver manifest failed: {0}")]
    Manifest(ManifestError),
    #[error("missing sender blob bytes for {0}")]
    MissingSenderBlob(B3Digest),
    #[error("missing sender chunk bytes for {0}")]
    MissingSenderChunk(B3Digest),
    #[error("delta transfer ended without final PathInfo")]
    MissingFinalPathInfo,
    #[error("delta transfer missing required blob {0}")]
    MissingRetainedBlob(B3Digest),
    #[error("delta transfer missing required chunk {0}")]
    MissingRetainedChunk(B3Digest),
    #[error("reconstructed root does not match final PathInfo")]
    RootDigestMismatch,
    #[error("delta transfer pathinfo signatures are not trusted")]
    UntrustedPathInfo,
    #[error("ordinary fallback pathinfo signatures are not trusted")]
    UntrustedFallbackPathInfo,
    #[error("ordinary fallback artifact missing for request {0}")]
    MissingFallback(String),
    #[error("persisting attestation: {0}")]
    Attestation(String),
}

pub async fn substitute_from_authority(
    authority: &InMemoryDeltaAuthority,
    request: &DeltaFetchRequest,
    receiver: &mut DeltaReceiverState,
    client_offer: &NegotiationOffer,
    trusted_keys: &[VerifyingKey],
    state_dir: &Path,
    store_dir: &str,
) -> Result<DeltaFetchOutcome, DeltaSubstitutionError> {
    let full_transfer_bytes = authority.sender.full_transfer_bytes();
    let Some(_capabilities) = authority.capabilities() else {
        let path_info = authority
            .full_artifact_fetch(request)?
            .ok_or_else(|| DeltaSubstitutionError::MissingFallback(request.output_name.clone()))?;
        verify_pathinfo_trusted(&path_info, trusted_keys)
            .map_err(|_| DeltaSubstitutionError::UntrustedFallbackPathInfo)?;
        let stored = persist_artifact_attestation(state_dir, store_dir, &request.output_name, &path_info, None)
            .await
            .map_err(|e| DeltaSubstitutionError::Attestation(format!("{e}")))?;
        return Ok(DeltaFetchOutcome {
            acceptance: DeltaAcceptanceMode::FullArtifactFallback(DeltaFallbackReason::LegacyCache),
            path_info,
            transferred_bytes: full_transfer_bytes,
            full_transfer_bytes,
            attestation_digest: stored.digest.to_hex(),
            stats: DeltaTransferStats {
                transferred_bytes: full_transfer_bytes,
                ..Default::default()
            },
        });
    };

    let candidate = match authority.request_candidates(request, client_offer) {
        Ok(Some(candidate)) => candidate,
        Ok(None) => {
            let path_info = authority
                .full_artifact_fetch(request)?
                .ok_or_else(|| DeltaSubstitutionError::MissingFallback(request.output_name.clone()))?;
            verify_pathinfo_trusted(&path_info, trusted_keys)
                .map_err(|_| DeltaSubstitutionError::UntrustedFallbackPathInfo)?;
            let stored = persist_artifact_attestation(state_dir, store_dir, &request.output_name, &path_info, None)
                .await
                .map_err(|e| DeltaSubstitutionError::Attestation(format!("{e}")))?;
            return Ok(DeltaFetchOutcome {
                acceptance: DeltaAcceptanceMode::FullArtifactFallback(DeltaFallbackReason::NegotiationFailed),
                path_info,
                transferred_bytes: full_transfer_bytes,
                full_transfer_bytes,
                attestation_digest: stored.digest.to_hex(),
                stats: DeltaTransferStats {
                    transferred_bytes: full_transfer_bytes,
                    ..Default::default()
                },
            });
        }
        Err(DeltaSubstitutionError::Negotiation(_)) => {
            let path_info = authority
                .full_artifact_fetch(request)?
                .ok_or_else(|| DeltaSubstitutionError::MissingFallback(request.output_name.clone()))?;
            verify_pathinfo_trusted(&path_info, trusted_keys)
                .map_err(|_| DeltaSubstitutionError::UntrustedFallbackPathInfo)?;
            let stored = persist_artifact_attestation(state_dir, store_dir, &request.output_name, &path_info, None)
                .await
                .map_err(|e| DeltaSubstitutionError::Attestation(format!("{e}")))?;
            return Ok(DeltaFetchOutcome {
                acceptance: DeltaAcceptanceMode::FullArtifactFallback(DeltaFallbackReason::NegotiationFailed),
                path_info,
                transferred_bytes: full_transfer_bytes,
                full_transfer_bytes,
                attestation_digest: stored.digest.to_hex(),
                stats: DeltaTransferStats {
                    transferred_bytes: full_transfer_bytes,
                    ..Default::default()
                },
            });
        }
        Err(err) => return Err(err),
    };

    let manifest = receiver.build_manifest_for(&candidate.sender).await.map_err(DeltaSubstitutionError::Manifest)?;
    let has_set = DeltaReceiverHasSet { manifest };
    let mut final_path_info: Option<PathInfo> = None;
    let stats = authority.stream_missing(&candidate.session_id, &has_set, |frame| {
        match frame {
            DeltaTransferFrame::Blob { digest, bytes } => {
                receiver.retained.retain_blob(digest, bytes);
            }
            DeltaTransferFrame::Chunk {
                parent_digest: _,
                chunk_digest,
                chunk_index: _,
                bytes,
            } => {
                receiver.retained.retain_chunk(chunk_digest, bytes);
            }
            DeltaTransferFrame::FinalPathInfo { path_info } => {
                final_path_info = Some(path_info);
            }
        }
        Ok(())
    })?;

    let path_info = final_path_info.ok_or(DeltaSubstitutionError::MissingFinalPathInfo)?;
    if ensure_fixture_content_available(&candidate.sender.outputs, &receiver.retained).is_none() {
        return Err(DeltaSubstitutionError::MissingFinalPathInfo);
    }
    let expected_root = output_root_node(&candidate.sender.outputs, &candidate.output_name)
        .ok_or(DeltaSubstitutionError::RootDigestMismatch)?;
    if expected_root != path_info.node {
        return Err(DeltaSubstitutionError::RootDigestMismatch);
    }

    if verify_pathinfo_trusted(&path_info, trusted_keys).is_err() {
        let fallback = authority
            .full_artifact_fetch(request)?
            .ok_or_else(|| DeltaSubstitutionError::MissingFallback(request.output_name.clone()))?;
        verify_pathinfo_trusted(&fallback, trusted_keys)
            .map_err(|_| DeltaSubstitutionError::UntrustedFallbackPathInfo)?;
        let stored = persist_artifact_attestation(state_dir, store_dir, &request.output_name, &fallback, None)
            .await
            .map_err(|e| DeltaSubstitutionError::Attestation(format!("{e}")))?;
        return Ok(DeltaFetchOutcome {
            acceptance: DeltaAcceptanceMode::FullArtifactFallback(DeltaFallbackReason::UntrustedDeltaPathInfo),
            path_info: fallback,
            transferred_bytes: full_transfer_bytes,
            full_transfer_bytes,
            attestation_digest: stored.digest.to_hex(),
            stats: DeltaTransferStats {
                transferred_bytes: full_transfer_bytes,
                ..Default::default()
            },
        });
    }

    let stored = persist_artifact_attestation(state_dir, store_dir, &request.output_name, &path_info, None)
        .await
        .map_err(|e| DeltaSubstitutionError::Attestation(format!("{e}")))?;
    Ok(DeltaFetchOutcome {
        acceptance: DeltaAcceptanceMode::Delta,
        path_info,
        transferred_bytes: stats.transferred_bytes,
        full_transfer_bytes,
        attestation_digest: stored.digest.to_hex(),
        stats,
    })
}

fn stream_missing_node<F>(
    node: &crate::ArtifactNode,
    manifest: &ReceiverManifest,
    catalog: &ContentCatalog,
    stats: &mut DeltaTransferStats,
    on_frame: &mut F,
) -> Result<(), DeltaSubstitutionError>
where
    F: FnMut(DeltaTransferFrame) -> Result<(), DeltaSubstitutionError>,
{
    match node {
        crate::ArtifactNode::Directory(directory) => {
            if manifest.known_directories.contains(&directory.digest) {
                return Ok(());
            }
            for child in &directory.children {
                stream_missing_node(child, manifest, catalog, stats, on_frame)?;
            }
            Ok(())
        }
        crate::ArtifactNode::Blob(blob) => {
            if manifest.known_blobs.contains(&blob.digest) {
                return Ok(());
            }
            if blob.chunks.is_empty() {
                let bytes = catalog
                    .full_blobs
                    .get(&blob.digest)
                    .cloned()
                    .ok_or(DeltaSubstitutionError::MissingSenderBlob(blob.digest))?;
                stats.record_blob(bytes.len());
                on_frame(DeltaTransferFrame::Blob {
                    digest: blob.digest,
                    bytes,
                })?;
                return Ok(());
            }

            let mut sent_any_chunk = false;
            for (chunk_index, chunk) in blob.chunks.iter().enumerate() {
                if manifest.known_chunks.contains(&chunk.digest) {
                    continue;
                }
                let chunk_bytes = catalog
                    .chunks
                    .get(&blob.digest)
                    .and_then(|chunks| chunks.get(chunk_index))
                    .cloned()
                    .ok_or(DeltaSubstitutionError::MissingSenderChunk(chunk.digest))?;
                stats.record_chunk(chunk_bytes.len());
                on_frame(DeltaTransferFrame::Chunk {
                    parent_digest: blob.digest,
                    chunk_digest: chunk.digest,
                    chunk_index: chunk_index as u32,
                    bytes: chunk_bytes,
                })?;
                sent_any_chunk = true;
            }
            if !sent_any_chunk && !manifest.known_blobs.contains(&blob.digest) {
                let bytes = catalog
                    .full_blobs
                    .get(&blob.digest)
                    .cloned()
                    .ok_or(DeltaSubstitutionError::MissingSenderBlob(blob.digest))?;
                stats.record_blob(bytes.len());
                on_frame(DeltaTransferFrame::Blob {
                    digest: blob.digest,
                    bytes,
                })?;
            }
            Ok(())
        }
        crate::ArtifactNode::Symlink { .. } => Ok(()),
    }
}

fn ensure_fixture_content_available(outputs: &[OutputFixture], retained: &RetainedContentStore) -> Option<()> {
    for output in outputs {
        if ensure_node_content_available(&output.root, retained).is_none() {
            return None;
        }
    }
    Some(())
}

fn ensure_node_content_available(node: &crate::ArtifactNode, retained: &RetainedContentStore) -> Option<()> {
    match node {
        crate::ArtifactNode::Directory(directory) => {
            for child in &directory.children {
                ensure_node_content_available(child, retained)?;
            }
            Some(())
        }
        crate::ArtifactNode::Blob(blob) => {
            if retained.has_blob(&blob.digest) {
                return Some(());
            }
            if blob.chunks.is_empty() {
                return None;
            }
            for chunk in &blob.chunks {
                if !retained.has_chunk(&chunk.digest) {
                    return None;
                }
            }
            Some(())
        }
        crate::ArtifactNode::Symlink { .. } => Some(()),
    }
}

fn output_root_node(outputs: &[OutputFixture], output_name: &str) -> Option<Node> {
    outputs
        .iter()
        .find(|output| output.output_id == output_name)
        .map(|output| fixture_node(&output.root))
}

fn fixture_node(node: &crate::ArtifactNode) -> Node {
    match node {
        crate::ArtifactNode::Directory(directory) => Node::Directory {
            digest: directory.digest,
            size: directory.children.len() as u64,
        },
        crate::ArtifactNode::Blob(blob) => Node::File {
            digest: blob.digest,
            size: blob.size_bytes,
            executable: false,
        },
        crate::ArtifactNode::Symlink { target } => Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from(target.as_str()).expect("fixture symlink target"),
        },
    }
}

fn compute_pathinfo_fingerprint(path_info: &PathInfo) -> String {
    let store_path_ref: StorePathRef = path_info.store_path.as_ref();
    let references = path_info.references.iter().map(|value| value.as_ref()).collect::<Vec<_>>();
    fingerprint(&store_path_ref, &path_info.nar_sha256, path_info.nar_size, references.iter())
}

fn verify_pathinfo_trusted(path_info: &PathInfo, trusted_keys: &[VerifyingKey]) -> Result<(), ()> {
    if trusted_keys.is_empty() {
        return Err(());
    }
    if path_info.signatures.is_empty() {
        return Err(());
    }
    let fingerprint = compute_pathinfo_fingerprint(path_info);
    let trusted = path_info.signatures.iter().any(|signature| {
        let signature_ref = signature.as_ref();
        trusted_keys.iter().any(|key| key.verify(&fingerprint, &signature_ref))
    });
    if trusted { Ok(()) } else { Err(()) }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use nix_compat::narinfo::parse_keypair;

    use super::*;
    use crate::ArtifactNode;
    use crate::BlobNode;
    use crate::ChunkRef;
    use crate::DirectoryNode;

    const TRUSTED_KEYPAIR: &str =
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

    fn parse_test_keypair(line: &str) -> (SigningKey<ed25519_dalek::SigningKey>, VerifyingKey) {
        parse_keypair(line).unwrap()
    }

    fn generated_untrusted_keypair() -> (SigningKey<ed25519_dalek::SigningKey>, VerifyingKey) {
        let dalek_signing = ed25519_dalek::SigningKey::from_bytes(&[9u8; 32]);
        let dalek_verifying = dalek_signing.verifying_key();
        (
            SigningKey::new("other-cache-1".to_owned(), dalek_signing),
            VerifyingKey::new("other-cache-1".to_owned(), dalek_verifying),
        )
    }

    fn digest_for(label: &str) -> B3Digest {
        blake3::hash(label.as_bytes()).as_bytes().into()
    }

    fn chunk(bytes: &[u8]) -> ChunkRef {
        ChunkRef {
            digest: blake3::hash(bytes).as_bytes().into(),
            size_bytes: bytes.len() as u64,
        }
    }

    fn chunked_blob(_label: &str, parts: &[&[u8]]) -> (ArtifactNode, Vec<Vec<u8>>) {
        let full = parts.iter().flat_map(|part| part.iter().copied()).collect::<Vec<_>>();
        let refs = parts.iter().map(|part| chunk(part)).collect::<Vec<_>>();
        (
            ArtifactNode::Blob(BlobNode {
                digest: blake3::hash(&full).as_bytes().into(),
                size_bytes: full.len() as u64,
                chunks: refs,
            }),
            parts.iter().map(|part| part.to_vec()).collect(),
        )
    }

    fn flat_blob(bytes: &[u8]) -> ArtifactNode {
        ArtifactNode::Blob(BlobNode {
            digest: blake3::hash(bytes).as_bytes().into(),
            size_bytes: bytes.len() as u64,
            chunks: Vec::new(),
        })
    }

    fn dir(label: &str, children: Vec<ArtifactNode>) -> ArtifactNode {
        ArtifactNode::Directory(DirectoryNode {
            digest: digest_for(&format!("dir:{label}")),
            children,
        })
    }

    fn fixture(output_id: &str, root: ArtifactNode) -> ClosureFixture {
        ClosureFixture {
            store_prefix: "/nix/store".to_owned(),
            outputs: vec![OutputFixture {
                output_id: output_id.to_owned(),
                root,
            }],
        }
    }

    fn store_path(name: &str, byte: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [byte; 20]).unwrap()
    }

    fn signed_path_info(
        store_path: StorePath<String>,
        root: &ArtifactNode,
        signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    ) -> PathInfo {
        let mut path_info = PathInfo {
            store_path,
            node: fixture_node(root),
            references: vec![],
            nar_size: root.full_transfer_bytes(),
            nar_sha256: [7u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        let fingerprint = compute_pathinfo_fingerprint(&path_info);
        let signature: Signature<String> = signing_key.sign(fingerprint.as_bytes()).to_owned();
        path_info.signatures.push(signature);
        path_info
    }

    fn authority_for_sender(
        output_name: &str,
        sender: ClosureFixture,
        root: &ArtifactNode,
        catalog: ContentCatalog,
        delta_signing_key: &SigningKey<ed25519_dalek::SigningKey>,
        full_signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    ) -> InMemoryDeltaAuthority {
        InMemoryDeltaAuthority {
            authority_prefix: "/cache.example.com".to_owned(),
            support_delta: true,
            sender,
            catalog,
            output_name: output_name.to_owned(),
            delta_path_info: signed_path_info(store_path(output_name, 9), root, delta_signing_key),
            full_artifact_path_info: signed_path_info(store_path(output_name, 9), root, full_signing_key),
        }
    }

    fn client_offer_v1() -> NegotiationOffer {
        NegotiationOffer::protocol_v1()
    }

    fn temp_state_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("crunch-delta-{label}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn http_endpoints_share_cache_authority() {
        let endpoints = DeltaHttpEndpoints::under_cache_authority("/binary-cache");
        assert!(endpoints.capability_path.starts_with("/binary-cache/"));
        assert!(endpoints.candidate_path.starts_with("/binary-cache/"));
        assert!(endpoints.has_set_path.starts_with("/binary-cache/"));
        assert!(endpoints.stream_path.starts_with("/binary-cache/"));
    }

    #[tokio::test]
    async fn finalized_ca_planning_uses_post_rewrite_bytes_only() {
        let final_parts = [b"hello ".as_slice(), b"/nix/store/final".as_slice()];
        let provisional_parts = [b"hello ".as_slice(), b"/nix/store/prov!".as_slice()];
        let (final_blob, final_chunk_bytes) = chunked_blob("final", &final_parts);
        let (provisional_blob, provisional_chunk_bytes) = chunked_blob("provisional", &provisional_parts);
        let sender = fixture("out", dir("final-root", vec![final_blob.clone()]));
        let receiver_store = fixture("local", dir("provisional-root", vec![provisional_blob.clone()]));
        let mut receiver = DeltaReceiverState {
            receiver_store,
            retained: RetainedContentStore::default(),
        };
        let (trusted_signing, trusted_verify) = parse_test_keypair(TRUSTED_KEYPAIR);
        let mut catalog = ContentCatalog::default();
        let final_blob_digest = match &final_blob {
            ArtifactNode::Blob(blob) => blob.digest,
            _ => unreachable!(),
        };
        catalog.insert_chunks(final_blob_digest, final_chunk_bytes);
        catalog.insert_blob(final_blob_digest, final_parts.concat());
        let authority = authority_for_sender(
            "out",
            sender.clone(),
            &sender.outputs[0].root,
            catalog,
            &trusted_signing,
            &trusted_signing,
        );

        // Seed retained store with provisional chunks only. If planning used pre-rewrite bytes,
        // it would incorrectly reuse them and transfer zero bytes.
        if let ArtifactNode::Blob(blob) = provisional_blob {
            receiver.retained.retain_blob(blob.digest, provisional_parts.concat());
            for (index, part) in provisional_chunk_bytes.into_iter().enumerate() {
                receiver.retained.retain_chunk(blob.chunks[index].digest, part);
            }
        }

        let outcome = substitute_from_authority(
            &authority,
            &DeltaFetchRequest {
                logical_path: "/nix/store/final-out".to_owned(),
                output_name: "out".to_owned(),
            },
            &mut receiver,
            &client_offer_v1(),
            std::slice::from_ref(&trusted_verify),
            &temp_state_dir("ca-finalized"),
            "/nix/store",
        )
        .await
        .unwrap();

        assert_eq!(outcome.acceptance, DeltaAcceptanceMode::Delta);
        assert!(outcome.transferred_bytes > 0, "post-rewrite bytes must trigger transfer");
    }

    #[tokio::test]
    async fn delta_transfer_reuses_v1_content_and_sends_only_missing_bytes() {
        let shared = b"shared-shared-shared-shared".repeat(8192);
        let changed = b"changed-changed-changed-changed".repeat(8192);
        let (sender_blob, sender_chunk_bytes) = chunked_blob("v2", &[shared.as_slice(), changed.as_slice()]);
        let (receiver_blob, receiver_chunk_bytes) =
            chunked_blob("v1", &[shared.as_slice(), b"old-old-old-old".repeat(8192).as_slice()]);
        let sender_root = dir("v2-root", vec![sender_blob.clone()]);
        let sender = fixture("out", sender_root.clone());
        let receiver_store = fixture("local", dir("v1-root", vec![receiver_blob.clone()]));
        let mut receiver = DeltaReceiverState {
            receiver_store,
            retained: RetainedContentStore::default(),
        };
        if let ArtifactNode::Blob(blob) = receiver_blob {
            receiver
                .retained
                .retain_blob(blob.digest, [shared.clone(), b"old-old-old-old".repeat(8192)].concat());
            for (index, part) in receiver_chunk_bytes.into_iter().enumerate() {
                receiver.retained.retain_chunk(blob.chunks[index].digest, part);
            }
        }

        let (trusted_signing, trusted_verify) = parse_test_keypair(TRUSTED_KEYPAIR);
        let mut catalog = ContentCatalog::default();
        let sender_blob_digest = match &sender_blob {
            ArtifactNode::Blob(blob) => blob.digest,
            _ => unreachable!(),
        };
        catalog.insert_chunks(sender_blob_digest, sender_chunk_bytes.clone());
        catalog.insert_blob(sender_blob_digest, [shared.clone(), changed.clone()].concat());
        let authority =
            authority_for_sender("out", sender.clone(), &sender_root, catalog, &trusted_signing, &trusted_signing);

        let outcome = substitute_from_authority(
            &authority,
            &DeltaFetchRequest {
                logical_path: "/nix/store/out".to_owned(),
                output_name: "out".to_owned(),
            },
            &mut receiver,
            &client_offer_v1(),
            std::slice::from_ref(&trusted_verify),
            &temp_state_dir("reuse-v1"),
            "/nix/store",
        )
        .await
        .unwrap();

        assert_eq!(outcome.acceptance, DeltaAcceptanceMode::Delta);
        assert_eq!(outcome.transferred_bytes, changed.len() as u64);
        assert!(outcome.transferred_bytes < outcome.full_transfer_bytes);
        assert_eq!(outcome.stats.streamed_chunk_count, 1);
    }

    #[tokio::test]
    async fn untrusted_delta_pathinfo_falls_back_per_policy() {
        let blob = flat_blob(b"trusted fallback body");
        let sender_root = dir("fallback-root", vec![blob.clone()]);
        let sender = fixture("out", sender_root.clone());
        let (trusted_signing, trusted_verify) = parse_test_keypair(TRUSTED_KEYPAIR);
        let (untrusted_signing, _) = generated_untrusted_keypair();
        let mut catalog = ContentCatalog::default();
        if let ArtifactNode::Blob(blob) = &blob {
            catalog.insert_blob(blob.digest, b"trusted fallback body".to_vec());
        }
        let authority =
            authority_for_sender("out", sender, &sender_root, catalog, &untrusted_signing, &trusted_signing);
        let mut receiver = DeltaReceiverState::default();

        let outcome = substitute_from_authority(
            &authority,
            &DeltaFetchRequest {
                logical_path: "/nix/store/out".to_owned(),
                output_name: "out".to_owned(),
            },
            &mut receiver,
            &client_offer_v1(),
            std::slice::from_ref(&trusted_verify),
            &temp_state_dir("fallback-untrusted"),
            "/nix/store",
        )
        .await
        .unwrap();

        assert_eq!(
            outcome.acceptance,
            DeltaAcceptanceMode::FullArtifactFallback(DeltaFallbackReason::UntrustedDeltaPathInfo)
        );
        assert_eq!(outcome.transferred_bytes, outcome.full_transfer_bytes);
    }

    #[tokio::test]
    async fn legacy_cache_without_delta_endpoints_falls_back_to_full_artifact() {
        let blob = flat_blob(b"legacy-cache-body");
        let sender_root = dir("legacy-root", vec![blob.clone()]);
        let sender = fixture("out", sender_root.clone());
        let (trusted_signing, trusted_verify) = parse_test_keypair(TRUSTED_KEYPAIR);
        let mut catalog = ContentCatalog::default();
        if let ArtifactNode::Blob(blob) = &blob {
            catalog.insert_blob(blob.digest, b"legacy-cache-body".to_vec());
        }
        let mut authority =
            authority_for_sender("out", sender, &sender_root, catalog, &trusted_signing, &trusted_signing);
        authority.support_delta = false;
        let mut receiver = DeltaReceiverState::default();

        let outcome = substitute_from_authority(
            &authority,
            &DeltaFetchRequest {
                logical_path: "/nix/store/out".to_owned(),
                output_name: "out".to_owned(),
            },
            &mut receiver,
            &client_offer_v1(),
            std::slice::from_ref(&trusted_verify),
            &temp_state_dir("legacy-cache"),
            "/nix/store",
        )
        .await
        .unwrap();

        assert_eq!(outcome.acceptance, DeltaAcceptanceMode::FullArtifactFallback(DeltaFallbackReason::LegacyCache));
        assert_eq!(outcome.transferred_bytes, outcome.full_transfer_bytes);
    }

    #[tokio::test]
    async fn interrupted_transfer_keeps_verified_chunks_and_discards_session_state() {
        let first = b"first-first-first-first".repeat(8192);
        let second = b"second-second-second-second".repeat(8192);
        let (sender_blob, sender_chunks) = chunked_blob("resume", &[first.as_slice(), second.as_slice()]);
        let sender_root = dir("resume-root", vec![sender_blob.clone()]);
        let sender = fixture("out", sender_root.clone());
        let (trusted_signing, trusted_verify) = parse_test_keypair(TRUSTED_KEYPAIR);
        let mut catalog = ContentCatalog::default();
        let sender_blob_digest = match &sender_blob {
            ArtifactNode::Blob(blob) => blob.digest,
            _ => unreachable!(),
        };
        catalog.insert_chunks(sender_blob_digest, sender_chunks.clone());
        catalog.insert_blob(sender_blob_digest, [first.clone(), second.clone()].concat());
        let authority =
            authority_for_sender("out", sender.clone(), &sender_root, catalog, &trusted_signing, &trusted_signing);
        let mut receiver = DeltaReceiverState::default();
        let request = DeltaFetchRequest {
            logical_path: "/nix/store/out".to_owned(),
            output_name: "out".to_owned(),
        };

        let candidate = authority.request_candidates(&request, &client_offer_v1()).unwrap().unwrap();
        let manifest = receiver.build_manifest_for(&candidate.sender).await.unwrap();
        let has_set = DeltaReceiverHasSet { manifest };
        let mut seen_first_chunk = false;
        let _ = authority.stream_missing(&candidate.session_id, &has_set, |frame| {
            if let DeltaTransferFrame::Chunk {
                chunk_digest, bytes, ..
            } = frame
            {
                receiver.retained.retain_chunk(chunk_digest, bytes);
                seen_first_chunk = true;
                return Err(DeltaSubstitutionError::MissingFinalPathInfo);
            }
            Ok(())
        });
        assert!(seen_first_chunk, "interrupted session must verify at least one chunk");
        assert_eq!(receiver.retained.chunks.len(), 1, "verified chunk must stay retained");

        let outcome = substitute_from_authority(
            &authority,
            &request,
            &mut receiver,
            &client_offer_v1(),
            std::slice::from_ref(&trusted_verify),
            &temp_state_dir("resume"),
            "/nix/store",
        )
        .await
        .unwrap();

        assert_eq!(outcome.acceptance, DeltaAcceptanceMode::Delta);
        assert_eq!(outcome.stats.streamed_chunk_count, 1, "retry should send only remaining chunk");
    }

    #[tokio::test]
    async fn large_closure_streams_without_whole_closure_buffering() {
        let mut children = Vec::new();
        let mut catalog = ContentCatalog::default();
        for index in 0..16u32 {
            let part_a = vec![index as u8; 262_144];
            let part_b = vec![index.saturating_add(1) as u8; 262_144];
            let (blob, chunks) = chunked_blob(&format!("large-{index}"), &[part_a.as_slice(), part_b.as_slice()]);
            if let ArtifactNode::Blob(blob_node) = &blob {
                catalog.insert_chunks(blob_node.digest, chunks.clone());
                catalog.insert_blob(blob_node.digest, [part_a, part_b].concat());
            }
            children.push(blob);
        }
        let sender_root = dir("large-root", children);
        let sender = fixture("out", sender_root.clone());
        let (trusted_signing, trusted_verify) = parse_test_keypair(TRUSTED_KEYPAIR);
        let authority = authority_for_sender("out", sender, &sender_root, catalog, &trusted_signing, &trusted_signing);
        let mut receiver = DeltaReceiverState::default();

        let outcome = substitute_from_authority(
            &authority,
            &DeltaFetchRequest {
                logical_path: "/nix/store/out".to_owned(),
                output_name: "out".to_owned(),
            },
            &mut receiver,
            &client_offer_v1(),
            std::slice::from_ref(&trusted_verify),
            &temp_state_dir("large-closure"),
            "/nix/store",
        )
        .await
        .unwrap();

        assert_eq!(outcome.acceptance, DeltaAcceptanceMode::Delta);
        assert!(outcome.stats.peak_active_directory_windows <= MAX_ACTIVE_DIRECTORY_WINDOWS);
        assert!(outcome.stats.peak_active_blob_windows <= MAX_ACTIVE_BLOB_WINDOWS);
        assert!(outcome.stats.peak_active_chunk_windows <= MAX_ACTIVE_CHUNK_WINDOWS);
        assert!(outcome.stats.peak_memory_bytes < outcome.full_transfer_bytes);
    }
}
