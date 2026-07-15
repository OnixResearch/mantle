use std::collections::HashSet;

use snix_castore::B3Digest;

const DIGEST_BYTES_LEN: usize = B3Digest::LENGTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkProfile {
    pub min_chunk_bytes: u32,
    pub avg_chunk_bytes: u32,
    pub max_chunk_bytes: u32,
}

pub fn chunk_profile_v1() -> ChunkProfile {
    chunk_profile_from_core(crunch_delta_core::chunk_profile_v1())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        blob_chunked_size_bytes(self)
    }

    pub fn validate(&self) {
        validate_blob(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryNode {
    pub digest: B3Digest,
    pub children: Vec<ArtifactNode>,
}

impl DirectoryNode {
    pub fn full_transfer_bytes(&self) -> u64 {
        artifact_full_transfer_bytes(&ArtifactNode::Directory(self.clone()))
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
        artifact_full_transfer_bytes(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFixture {
    pub output_id: String,
    pub root: ArtifactNode,
}

impl OutputFixture {
    pub fn full_transfer_bytes(&self) -> u64 {
        output_full_transfer_bytes(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureFixture {
    pub store_prefix: String,
    pub outputs: Vec<OutputFixture>,
}

impl ClosureFixture {
    pub fn full_transfer_bytes(&self) -> u64 {
        closure_full_transfer_bytes(self)
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

pub(crate) fn new_receiver_manifest(store_prefix: &str) -> ReceiverManifest {
    ReceiverManifest::new(store_prefix)
}

pub(crate) fn artifact_full_transfer_bytes(node: &ArtifactNode) -> u64 {
    crunch_delta_core::artifact_full_transfer_bytes(core_artifact_node(node))
}

pub(crate) fn blob_chunked_size_bytes(blob: &BlobNode) -> u64 {
    crunch_delta_core::blob_chunked_size_bytes(core_blob_node(blob))
}

pub(crate) fn closure_full_transfer_bytes(fixture: &ClosureFixture) -> u64 {
    crunch_delta_core::closure_full_transfer_bytes(core_closure_fixture(fixture))
}

pub(crate) fn output_full_transfer_bytes(output: &OutputFixture) -> u64 {
    crunch_delta_core::output_full_transfer_bytes(core_output_fixture(output))
}

pub(crate) fn validate_blob(blob: &BlobNode) {
    crunch_delta_core::validate_blob(core_blob_node(blob));
}

pub(crate) fn delta_digest_from_b3(digest: B3Digest) -> crunch_delta_core::DeltaDigest {
    let digest_bytes: [u8; DIGEST_BYTES_LEN] = digest.into();
    crunch_delta_core::DeltaDigest(digest_bytes)
}

pub(crate) trait IntoB3Digest {
    fn into_b3_digest(self) -> B3Digest;
}

impl IntoB3Digest for B3Digest {
    fn into_b3_digest(self) -> B3Digest {
        self
    }
}

impl IntoB3Digest for crunch_delta_core::DeltaDigest {
    fn into_b3_digest(self) -> B3Digest {
        B3Digest::from(&self.0)
    }
}

pub(crate) fn b3_digest_from_delta<D>(digest: D) -> B3Digest
where D: IntoB3Digest {
    digest.into_b3_digest()
}

pub(crate) fn manifest_insert_known_directory(manifest: &mut ReceiverManifest, digest: B3Digest) {
    manifest.known_directories.insert(digest);
}

pub(crate) fn manifest_insert_known_blob(manifest: &mut ReceiverManifest, digest: B3Digest) {
    manifest.known_blobs.insert(digest);
}

pub(crate) fn manifest_insert_known_chunk(manifest: &mut ReceiverManifest, digest: B3Digest) {
    manifest.known_chunks.insert(digest);
}

pub(crate) fn manifest_has_known_directory(manifest: &ReceiverManifest, digest: B3Digest) -> bool {
    manifest.known_directories.contains(&digest)
}

pub(crate) fn manifest_has_known_blob(manifest: &ReceiverManifest, digest: B3Digest) -> bool {
    manifest.known_blobs.contains(&digest)
}

pub(crate) fn manifest_has_known_chunk(manifest: &ReceiverManifest, digest: B3Digest) -> bool {
    manifest.known_chunks.contains(&digest)
}

#[cfg(test)]
pub(crate) fn manifest_known_chunks_as_b3(manifest: &ReceiverManifest) -> HashSet<B3Digest> {
    manifest.known_chunks.clone()
}

pub(crate) fn chunk_profile_from_core(profile: crunch_delta_core::ChunkProfile) -> ChunkProfile {
    ChunkProfile {
        min_chunk_bytes: profile.min_chunk_bytes,
        avg_chunk_bytes: profile.avg_chunk_bytes,
        max_chunk_bytes: profile.max_chunk_bytes,
    }
}

pub(crate) fn core_chunk_profile(profile: &ChunkProfile) -> crunch_delta_core::ChunkProfile {
    crunch_delta_core::ChunkProfile {
        min_chunk_bytes: profile.min_chunk_bytes,
        avg_chunk_bytes: profile.avg_chunk_bytes,
        max_chunk_bytes: profile.max_chunk_bytes,
    }
}

pub(crate) fn core_receiver_manifest(manifest: &ReceiverManifest) -> crunch_delta_core::ReceiverManifest {
    let known_directories = manifest
        .known_directories
        .iter()
        .copied()
        .map(delta_digest_from_b3)
        .collect::<std::collections::BTreeSet<_>>();
    let known_blobs = manifest
        .known_blobs
        .iter()
        .copied()
        .map(delta_digest_from_b3)
        .collect::<std::collections::BTreeSet<_>>();
    let known_chunks = manifest
        .known_chunks
        .iter()
        .copied()
        .map(delta_digest_from_b3)
        .collect::<std::collections::BTreeSet<_>>();
    let known_outputs = manifest.known_outputs.iter().cloned().collect::<std::collections::BTreeSet<_>>();

    debug_assert_eq!(
        known_outputs.len(),
        manifest.known_outputs.len(),
        "output identities must survive core conversion"
    );
    debug_assert_eq!(
        known_directories.len(),
        manifest.known_directories.len(),
        "directory digests must survive core conversion"
    );
    debug_assert_eq!(known_blobs.len(), manifest.known_blobs.len(), "blob digests must survive core conversion");
    debug_assert_eq!(known_chunks.len(), manifest.known_chunks.len(), "chunk digests must survive core conversion");

    crunch_delta_core::ReceiverManifest {
        store_prefix: manifest.store_prefix.clone(),
        known_outputs,
        known_directories,
        known_blobs,
        known_chunks,
    }
}

pub(crate) fn core_closure_fixture(fixture: &ClosureFixture) -> crunch_delta_core::ClosureFixture {
    crunch_delta_core::ClosureFixture {
        store_prefix: fixture.store_prefix.clone(),
        outputs: fixture.outputs.iter().map(core_output_fixture).collect::<Vec<_>>(),
    }
}

pub(crate) fn transfer_plan_from_core(plan: crunch_delta_core::TransferPlan) -> TransferPlan {
    TransferPlan {
        transferred_bytes: plan.transferred_bytes,
        full_transfer_bytes: plan.full_transfer_bytes,
        tally: TransferTally {
            reused_outputs: plan.tally.reused_outputs,
            reused_directories: plan.tally.reused_directories,
            reused_blobs: plan.tally.reused_blobs,
            reused_chunks: plan.tally.reused_chunks,
            sent_blobs: plan.tally.sent_blobs,
            sent_chunks: plan.tally.sent_chunks,
        },
    }
}

fn core_output_fixture(output: &OutputFixture) -> crunch_delta_core::OutputFixture {
    crunch_delta_core::OutputFixture {
        output_id: output.output_id.clone(),
        root: core_artifact_node(&output.root),
    }
}

fn core_artifact_node(node: &ArtifactNode) -> crunch_delta_core::ArtifactNode {
    match node {
        ArtifactNode::Directory(directory) => {
            crunch_delta_core::ArtifactNode::Directory(core_directory_node(directory))
        }
        ArtifactNode::Blob(blob) => crunch_delta_core::ArtifactNode::Blob(core_blob_node(blob)),
        ArtifactNode::Symlink { target } => crunch_delta_core::ArtifactNode::Symlink { target: target.clone() },
    }
}

fn core_directory_node(directory: &DirectoryNode) -> crunch_delta_core::DirectoryNode {
    crunch_delta_core::DirectoryNode {
        digest: delta_digest_from_b3(directory.digest),
        children: directory.children.iter().map(core_artifact_node).collect::<Vec<_>>(),
    }
}

fn core_blob_node(blob: &BlobNode) -> crunch_delta_core::BlobNode {
    crunch_delta_core::BlobNode {
        digest: delta_digest_from_b3(blob.digest),
        size_bytes: blob.size_bytes,
        chunks: blob.chunks.iter().map(core_chunk_ref).collect::<Vec<_>>(),
    }
}

fn core_chunk_ref(chunk: &ChunkRef) -> crunch_delta_core::ChunkRef {
    crunch_delta_core::ChunkRef {
        digest: delta_digest_from_b3(chunk.digest),
        size_bytes: chunk.size_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVALID_BLOB_SIZE_BYTES: u64 = 16;
    const INVALID_CHUNK_SIZE_BYTES: u64 = 8;

    fn test_digest(label: &str) -> B3Digest {
        blake3::hash(label.as_bytes()).into()
    }

    #[test]
    fn chunk_profile_v1_matches_spec() {
        let profile = chunk_profile_v1();
        assert_eq!(profile.min_chunk_bytes, 131_072);
        assert_eq!(profile.avg_chunk_bytes, 262_144);
        assert_eq!(profile.max_chunk_bytes, 524_288);
    }

    #[test]
    fn receiver_manifest_conversion_preserves_facade_sets() {
        let directory_digest = test_digest("directory");
        let blob_digest = test_digest("blob");
        let chunk_digest = test_digest("chunk");
        let output_id = "/mantle/store/output".to_string();
        let mut manifest = ReceiverManifest::new("/mantle/store");
        manifest.known_outputs.insert(output_id.clone());
        manifest.known_directories.insert(directory_digest);
        manifest.known_blobs.insert(blob_digest);
        manifest.known_chunks.insert(chunk_digest);

        let core_manifest = core_receiver_manifest(&manifest);

        assert_eq!(core_manifest.store_prefix, manifest.store_prefix);
        assert!(core_manifest.known_outputs.contains(&output_id));
        assert!(core_manifest.known_directories.contains(&delta_digest_from_b3(directory_digest)));
        assert!(core_manifest.known_blobs.contains(&delta_digest_from_b3(blob_digest)));
        assert!(core_manifest.known_chunks.contains(&delta_digest_from_b3(chunk_digest)));
    }

    #[test]
    fn blob_validate_rejects_wrong_chunk_total() {
        let blob = BlobNode {
            digest: test_digest("blob"),
            size_bytes: INVALID_BLOB_SIZE_BYTES,
            chunks: vec![ChunkRef {
                digest: test_digest("chunk"),
                size_bytes: INVALID_CHUNK_SIZE_BYTES,
            }],
        };
        let caught = std::panic::catch_unwind(|| validate_blob(&blob));
        assert!(caught.is_err(), "blob.validate should reject mismatched chunk sizes");
    }
}
