use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

const CHUNK_PROFILE_V1_MIN_BYTES: u32 = 131_072;
const CHUNK_PROFILE_V1_AVG_BYTES: u32 = 262_144;
const CHUNK_PROFILE_V1_MAX_BYTES: u32 = 524_288;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeltaDigest(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkProfile {
    pub min_chunk_bytes: u32,
    pub avg_chunk_bytes: u32,
    pub max_chunk_bytes: u32,
}

pub fn chunk_profile_v1() -> ChunkProfile {
    let profile = ChunkProfile {
        min_chunk_bytes: CHUNK_PROFILE_V1_MIN_BYTES,
        avg_chunk_bytes: CHUNK_PROFILE_V1_AVG_BYTES,
        max_chunk_bytes: CHUNK_PROFILE_V1_MAX_BYTES,
    };
    assert!(profile.min_chunk_bytes < profile.avg_chunk_bytes);
    assert!(profile.avg_chunk_bytes < profile.max_chunk_bytes);
    profile
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverManifest {
    pub store_prefix: String,
    pub known_outputs: BTreeSet<String>,
    pub known_directories: BTreeSet<DeltaDigest>,
    pub known_blobs: BTreeSet<DeltaDigest>,
    pub known_chunks: BTreeSet<DeltaDigest>,
}

impl ReceiverManifest {
    pub fn new(store_prefix: String) -> Self {
        assert!(!store_prefix.is_empty(), "store_prefix must not be empty");
        assert!(store_prefix.starts_with('/'), "store_prefix must be absolute");
        Self {
            store_prefix,
            known_outputs: BTreeSet::new(),
            known_directories: BTreeSet::new(),
            known_blobs: BTreeSet::new(),
            known_chunks: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkRef {
    pub digest: DeltaDigest,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobNode {
    pub digest: DeltaDigest,
    pub size_bytes: u64,
    pub chunks: Vec<ChunkRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryNode {
    pub digest: DeltaDigest,
    pub children: Vec<ArtifactNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactNode {
    Directory(DirectoryNode),
    Blob(BlobNode),
    Symlink { target: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFixture {
    pub output_id: String,
    pub root: ArtifactNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureFixture {
    pub store_prefix: String,
    pub outputs: Vec<OutputFixture>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransferTally {
    pub reused_outputs: u32,
    pub reused_directories: u32,
    pub reused_blobs: u32,
    pub reused_chunks: u32,
    pub sent_blobs: u32,
    pub sent_chunks: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferPlan {
    pub transferred_bytes: u64,
    pub full_transfer_bytes: u64,
    pub tally: TransferTally,
}

pub fn validate_blob(blob: BlobNode) {
    assert!(blob.size_bytes > 0, "blob size must be positive");
    if blob.chunks.is_empty() {
        return;
    }
    assert_eq!(blob_chunked_size_bytes(blob.clone()), blob.size_bytes, "chunk sizes must sum to blob size");
}

pub fn closure_full_transfer_bytes(fixture: ClosureFixture) -> u64 {
    fixture
        .outputs
        .into_iter()
        .fold(0u64, |total, output| total.saturating_add(output_full_transfer_bytes(output)))
}

pub fn output_full_transfer_bytes(output: OutputFixture) -> u64 {
    artifact_full_transfer_bytes(output.root)
}

pub fn artifact_full_transfer_bytes(node: ArtifactNode) -> u64 {
    match node {
        ArtifactNode::Directory(directory) => directory
            .children
            .into_iter()
            .fold(0u64, |total, child| total.saturating_add(artifact_full_transfer_bytes(child))),
        ArtifactNode::Blob(blob) => blob.size_bytes,
        ArtifactNode::Symlink { .. } => 0,
    }
}

pub fn blob_chunked_size_bytes(blob: BlobNode) -> u64 {
    blob.chunks.into_iter().fold(0u64, |total, chunk| total.saturating_add(chunk.size_bytes))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const DELTA_DIGEST_BYTES_LEN: usize = 32;
    const INVALID_BLOB_SIZE_BYTES: u64 = 16;
    const INVALID_CHUNK_SIZE_BYTES: u64 = 8;

    #[test]
    fn chunk_profile_v1_matches_spec() {
        let profile = chunk_profile_v1();
        assert_eq!(profile.min_chunk_bytes, CHUNK_PROFILE_V1_MIN_BYTES);
        assert_eq!(profile.avg_chunk_bytes, CHUNK_PROFILE_V1_AVG_BYTES);
        assert_eq!(profile.max_chunk_bytes, CHUNK_PROFILE_V1_MAX_BYTES);
    }

    #[test]
    fn blob_validation_rejects_mismatched_chunk_total() {
        let blob = BlobNode {
            digest: DeltaDigest([1; DELTA_DIGEST_BYTES_LEN]),
            size_bytes: INVALID_BLOB_SIZE_BYTES,
            chunks: vec![ChunkRef {
                digest: DeltaDigest([2; DELTA_DIGEST_BYTES_LEN]),
                size_bytes: INVALID_CHUNK_SIZE_BYTES,
            }],
        };
        let caught = std::panic::catch_unwind(|| validate_blob(blob.clone()));
        assert!(caught.is_err(), "mismatched chunk totals must panic");
        assert_ne!(blob_chunked_size_bytes(blob), INVALID_BLOB_SIZE_BYTES);
    }
}
