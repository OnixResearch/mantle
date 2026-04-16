use std::collections::HashSet;

use snix_castore::B3Digest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkProfile {
    pub min_chunk_bytes: u32,
    pub avg_chunk_bytes: u32,
    pub max_chunk_bytes: u32,
}

pub fn chunk_profile_v1() -> ChunkProfile {
    let profile = ChunkProfile {
        min_chunk_bytes: 131_072,
        avg_chunk_bytes: 262_144,
        max_chunk_bytes: 524_288,
    };
    assert!(profile.min_chunk_bytes < profile.avg_chunk_bytes);
    assert!(profile.avg_chunk_bytes < profile.max_chunk_bytes);
    profile
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverManifest {
    pub store_prefix: String,
    pub known_outputs: HashSet<String>,
    pub known_directories: HashSet<B3Digest>,
    pub known_blobs: HashSet<B3Digest>,
    pub known_chunks: HashSet<B3Digest>,
}

impl ReceiverManifest {
    pub fn new(store_prefix: &str) -> Self {
        assert!(!store_prefix.is_empty(), "store_prefix must not be empty");
        assert!(store_prefix.starts_with('/'), "store_prefix must be absolute");
        Self {
            store_prefix: store_prefix.to_owned(),
            known_outputs: HashSet::new(),
            known_directories: HashSet::new(),
            known_blobs: HashSet::new(),
            known_chunks: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkRef {
    pub digest: B3Digest,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobNode {
    pub digest: B3Digest,
    pub size_bytes: u64,
    pub chunks: Vec<ChunkRef>,
}

impl BlobNode {
    pub fn chunked_size_bytes(&self) -> u64 {
        self.chunks.iter().fold(0u64, |total, chunk| total.saturating_add(chunk.size_bytes))
    }

    pub fn validate(&self) {
        assert!(self.size_bytes > 0, "blob size must be positive");
        if self.chunks.is_empty() {
            return;
        }
        assert_eq!(self.chunked_size_bytes(), self.size_bytes, "chunk sizes must sum to blob size");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryNode {
    pub digest: B3Digest,
    pub children: Vec<ArtifactNode>,
}

impl DirectoryNode {
    pub fn full_transfer_bytes(&self) -> u64 {
        self.children.iter().fold(0u64, |total, child| total.saturating_add(child.full_transfer_bytes()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactNode {
    Directory(DirectoryNode),
    Blob(BlobNode),
    Symlink { target: String },
}

impl ArtifactNode {
    pub fn full_transfer_bytes(&self) -> u64 {
        match self {
            Self::Directory(directory) => directory.full_transfer_bytes(),
            Self::Blob(blob) => blob.size_bytes,
            Self::Symlink { .. } => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFixture {
    pub output_id: String,
    pub root: ArtifactNode,
}

impl OutputFixture {
    pub fn full_transfer_bytes(&self) -> u64 {
        self.root.full_transfer_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureFixture {
    pub store_prefix: String,
    pub outputs: Vec<OutputFixture>,
}

impl ClosureFixture {
    pub fn full_transfer_bytes(&self) -> u64 {
        self.outputs.iter().fold(0u64, |total, output| total.saturating_add(output.full_transfer_bytes()))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_profile_v1_matches_spec() {
        let profile = chunk_profile_v1();
        assert_eq!(profile.min_chunk_bytes, 131_072);
        assert_eq!(profile.avg_chunk_bytes, 262_144);
        assert_eq!(profile.max_chunk_bytes, 524_288);
    }

    #[test]
    fn blob_validate_rejects_wrong_chunk_total() {
        let blob = BlobNode {
            digest: blake3::hash(b"blob").as_bytes().into(),
            size_bytes: 16,
            chunks: vec![ChunkRef {
                digest: blake3::hash(b"chunk").as_bytes().into(),
                size_bytes: 8,
            }],
        };
        let caught = std::panic::catch_unwind(|| blob.validate());
        assert!(caught.is_err(), "blob.validate should reject mismatched chunk sizes");
    }
}
