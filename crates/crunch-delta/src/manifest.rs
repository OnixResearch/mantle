use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use bytes::Bytes;
use snix_castore::B3Digest;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::PathComponent;
use snix_castore::SymlinkTarget;
use snix_castore::proto::stat_blob_response::ChunkMeta;

use crate::fixtures::BenchCase;
use crate::fixtures::ReceiverFrontierSummary;
use crate::fixtures::ReceiverLossyFrontierSummary;
use crate::fixtures::ReceiverProbabilisticFrontierSummary;
use crate::model::ArtifactNode;
use crate::model::BlobNode;
use crate::model::ChunkProfile;
use crate::model::ClosureFixture;
use crate::model::ReceiverManifest;
use crate::model::b3_digest_from_delta;
use crate::model::chunk_profile_v1;
use crate::model::manifest_insert_known_blob;
use crate::model::manifest_insert_known_chunk;
use crate::model::manifest_insert_known_directory;
use crate::model::new_receiver_manifest;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManifestProbeCounts {
    pub output_checks: u64,
    pub directory_gets: u64,
    pub chunk_queries: u64,
}

impl ManifestProbeCounts {
    pub fn total_probes(&self) -> u64 {
        self.output_checks.saturating_add(self.directory_gets).saturating_add(self.chunk_queries)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestBuildOutcome {
    pub manifest: ReceiverManifest,
    pub probes: ManifestProbeCounts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    StorePrefixMismatch {
        sender_prefix: String,
        receiver_prefix: String,
    },
    MissingDirectory(B3Digest),
    MissingBlob(B3Digest),
    InvalidChunkDigestLen(usize),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorePrefixMismatch {
                sender_prefix,
                receiver_prefix,
            } => write!(f, "store prefix mismatch: sender={sender_prefix} receiver={receiver_prefix}"),
            Self::MissingDirectory(digest) => write!(f, "missing seeded directory {digest}"),
            Self::MissingBlob(digest) => write!(f, "missing seeded blob {digest}"),
            Self::InvalidChunkDigestLen(length) => write!(f, "invalid chunk digest length {length}"),
        }
    }
}

impl std::error::Error for ManifestError {}

pub async fn build_receiver_manifest(case: &BenchCase) -> Result<ManifestBuildOutcome, ManifestError> {
    if case.sender.store_prefix != case.receiver_store.store_prefix {
        return Err(ManifestError::StorePrefixMismatch {
            sender_prefix: case.sender.store_prefix.clone(),
            receiver_prefix: case.receiver_store.store_prefix.clone(),
        });
    }

    let services = FixtureManifestStore::from_fixture(&case.receiver_store);
    build_receiver_manifest_with_store(case, &services).await
}

async fn build_receiver_manifest_with_store(
    case: &BenchCase,
    services: &FixtureManifestStore,
) -> Result<ManifestBuildOutcome, ManifestError> {
    debug_assert!(!case.sender.outputs.is_empty());
    debug_assert!(!case.sender.store_prefix.is_empty());
    let mut manifest = new_receiver_manifest(&case.sender.store_prefix);
    let candidates = SenderCandidates::from_fixture(&case.sender);
    let chunk_profile = chunk_profile_for_manifest();
    let mut traversal = TraversalMemo::default();

    let frontier_index = case.receiver_frontiers.iter().fold(
        HashMap::<&str, HashMap<B3Digest, &ReceiverFrontierSummary>>::new(),
        |mut acc, summary| {
            acc.entry(summary.owner_output_id.as_str()).or_default().insert(summary.directory_digest, summary);
            acc
        },
    );
    let output_checks = case.sender.outputs.len() as u64;

    for output in &case.receiver_store.outputs {
        if candidates.output_ids.contains(&output.output_id) {
            manifest.known_outputs.insert(output.output_id.clone());
            continue;
        }
        let frontiers = frontier_index.get(output.output_id.as_str());
        if case.frontier_complete_outputs.contains(&output.output_id)
            && let Some(frontiers) = frontiers
        {
            for summary in frontiers.values() {
                apply_frontier_hits(summary, &candidates, &mut manifest);
            }
            continue;
        }
        walk_local_node(&output.root, frontiers, services, &candidates, &chunk_profile, &mut traversal, &mut manifest)
            .await?;
    }

    let probes = services.finish_counts(output_checks);
    Ok(ManifestBuildOutcome { manifest, probes })
}

pub async fn build_receiver_manifest_lossy(case: &BenchCase) -> Result<ManifestBuildOutcome, ManifestError> {
    if case.sender.store_prefix != case.receiver_store.store_prefix {
        return Err(ManifestError::StorePrefixMismatch {
            sender_prefix: case.sender.store_prefix.clone(),
            receiver_prefix: case.receiver_store.store_prefix.clone(),
        });
    }

    let services = FixtureManifestStore::from_fixture(&case.receiver_store);
    build_receiver_manifest_lossy_with_store(case, &services).await
}

async fn build_receiver_manifest_lossy_with_store(
    case: &BenchCase,
    services: &FixtureManifestStore,
) -> Result<ManifestBuildOutcome, ManifestError> {
    debug_assert!(!case.sender.outputs.is_empty());
    debug_assert!(!case.sender.store_prefix.is_empty());
    let mut manifest = new_receiver_manifest(&case.sender.store_prefix);
    let exact_candidates = SenderCandidates::from_fixture(&case.sender);
    let lossy_candidates = LossySenderCandidates::from_fixture(&case.sender);
    let chunk_profile = chunk_profile_for_manifest();
    let mut traversal = TraversalMemo::default();

    let lossy_frontier_index = case.receiver_lossy_frontiers.iter().fold(
        HashMap::<&str, HashMap<B3Digest, &ReceiverLossyFrontierSummary>>::new(),
        |mut acc, summary| {
            acc.entry(summary.owner_output_id.as_str()).or_default().insert(summary.directory_digest, summary);
            acc
        },
    );
    let output_checks = case.sender.outputs.len() as u64;

    for output in &case.receiver_store.outputs {
        if exact_candidates.output_ids.contains(&output.output_id) {
            manifest.known_outputs.insert(output.output_id.clone());
            continue;
        }
        let lossy_frontiers = lossy_frontier_index.get(output.output_id.as_str());
        if case.frontier_complete_outputs.contains(&output.output_id)
            && let Some(lossy_frontiers) = lossy_frontiers
        {
            walk_local_nodes_lossy(
                frontier_seed_nodes_lossy(lossy_frontiers),
                Some(lossy_frontiers),
                services,
                &exact_candidates,
                &lossy_candidates,
                &chunk_profile,
                &mut traversal,
                &mut manifest,
            )
            .await?;
            continue;
        }
        walk_local_nodes_lossy(
            vec![seed_child_node(&output.root)],
            lossy_frontiers,
            services,
            &exact_candidates,
            &lossy_candidates,
            &chunk_profile,
            &mut traversal,
            &mut manifest,
        )
        .await?;
    }

    let probes = services.finish_counts(output_checks);
    Ok(ManifestBuildOutcome { manifest, probes })
}

pub async fn build_receiver_manifest_probabilistic(case: &BenchCase) -> Result<ManifestBuildOutcome, ManifestError> {
    if case.sender.store_prefix != case.receiver_store.store_prefix {
        return Err(ManifestError::StorePrefixMismatch {
            sender_prefix: case.sender.store_prefix.clone(),
            receiver_prefix: case.receiver_store.store_prefix.clone(),
        });
    }

    let services = FixtureManifestStore::from_fixture(&case.receiver_store);
    build_receiver_manifest_probabilistic_with_store(case, &services).await
}

async fn build_receiver_manifest_probabilistic_with_store(
    case: &BenchCase,
    services: &FixtureManifestStore,
) -> Result<ManifestBuildOutcome, ManifestError> {
    debug_assert!(!case.sender.outputs.is_empty());
    debug_assert!(!case.sender.store_prefix.is_empty());
    let mut manifest = new_receiver_manifest(&case.sender.store_prefix);
    let exact_candidates = SenderCandidates::from_fixture(&case.sender);
    let probabilistic_candidates = ProbabilisticSenderCandidates::from_fixture(&case.sender);
    let chunk_profile = chunk_profile_for_manifest();
    let mut traversal = TraversalMemo::default();
    let probabilistic_frontier_index = case.receiver_probabilistic_frontiers.iter().fold(
        HashMap::<&str, HashMap<B3Digest, &ReceiverProbabilisticFrontierSummary>>::new(),
        |mut acc, summary| {
            acc.entry(summary.owner_output_id.as_str()).or_default().insert(summary.directory_digest, summary);
            acc
        },
    );
    let output_checks = case.sender.outputs.len() as u64;

    for output in &case.receiver_store.outputs {
        if exact_candidates.output_ids.contains(&output.output_id) {
            manifest.known_outputs.insert(output.output_id.clone());
            continue;
        }
        let probabilistic_frontiers = probabilistic_frontier_index.get(output.output_id.as_str());
        if case.frontier_complete_outputs.contains(&output.output_id)
            && let Some(probabilistic_frontiers) = probabilistic_frontiers
        {
            walk_local_nodes_probabilistic(
                frontier_seed_nodes_probabilistic(probabilistic_frontiers),
                Some(probabilistic_frontiers),
                services,
                &exact_candidates,
                &probabilistic_candidates,
                &chunk_profile,
                &mut traversal,
                &mut manifest,
            )
            .await?;
            continue;
        }
        walk_local_nodes_probabilistic(
            vec![seed_child_node(&output.root)],
            probabilistic_frontiers,
            services,
            &exact_candidates,
            &probabilistic_candidates,
            &chunk_profile,
            &mut traversal,
            &mut manifest,
        )
        .await?;
    }

    let probes = services.finish_counts(output_checks);
    Ok(ManifestBuildOutcome { manifest, probes })
}

#[derive(Default)]
struct SenderCandidates {
    output_ids: HashSet<String>,
    directory_digests: HashSet<B3Digest>,
    blob_digests: HashSet<B3Digest>,
    chunk_digests: HashSet<B3Digest>,
}

impl SenderCandidates {
    fn from_fixture(fixture: &ClosureFixture) -> Self {
        let mut output_ids = HashSet::new();
        let mut directory_digests = HashSet::new();
        let mut blob_digests = HashSet::new();
        let mut chunk_digests = HashSet::new();
        for output in &fixture.outputs {
            output_ids.insert(output.output_id.clone());
            collect_sender_candidates(&output.root, &mut directory_digests, &mut blob_digests, &mut chunk_digests);
        }
        Self {
            output_ids,
            directory_digests,
            blob_digests,
            chunk_digests,
        }
    }
}

#[derive(Default)]
struct LossySenderCandidates {
    directory_buckets: HashSet<u8>,
    blob_buckets: HashSet<u8>,
    chunk_buckets: HashSet<u8>,
}

impl LossySenderCandidates {
    fn from_fixture(fixture: &ClosureFixture) -> Self {
        let mut directory_buckets = HashSet::new();
        let mut blob_buckets = HashSet::new();
        let mut chunk_buckets = HashSet::new();
        for output in &fixture.outputs {
            collect_lossy_sender_candidates(
                &output.root,
                &mut directory_buckets,
                &mut blob_buckets,
                &mut chunk_buckets,
            );
        }
        Self {
            directory_buckets,
            blob_buckets,
            chunk_buckets,
        }
    }
}

#[derive(Default)]
struct ProbabilisticSenderCandidates {
    directory_digests: Vec<B3Digest>,
    blob_digests: Vec<B3Digest>,
    chunk_digests: Vec<B3Digest>,
}

impl ProbabilisticSenderCandidates {
    fn from_fixture(fixture: &ClosureFixture) -> Self {
        let exact = SenderCandidates::from_fixture(fixture);
        Self {
            directory_digests: exact.directory_digests.into_iter().collect(),
            blob_digests: exact.blob_digests.into_iter().collect(),
            chunk_digests: exact.chunk_digests.into_iter().collect(),
        }
    }
}

#[allow(tigerstyle::no_recursion)] // tree walk bounded by fixture tree depth
fn collect_sender_candidates(
    node: &ArtifactNode,
    directory_digests: &mut HashSet<B3Digest>,
    blob_digests: &mut HashSet<B3Digest>,
    chunk_digests: &mut HashSet<B3Digest>,
) {
    match node {
        ArtifactNode::Directory(directory) => {
            directory_digests.insert(b3_digest_from_delta(directory.digest));
            for child in &directory.children {
                collect_sender_candidates(child, directory_digests, blob_digests, chunk_digests);
            }
        }
        ArtifactNode::Blob(blob) => {
            blob_digests.insert(b3_digest_from_delta(blob.digest));
            for chunk in &blob.chunks {
                chunk_digests.insert(b3_digest_from_delta(chunk.digest));
            }
        }
        ArtifactNode::Symlink { .. } => {}
    }
}

fn frontier_has_overlap(summary: &ReceiverFrontierSummary, candidates: &SenderCandidates) -> bool {
    if summary.directory_digests.iter().any(|digest| candidates.directory_digests.contains(digest)) {
        return true;
    }
    if summary.blob_digests.iter().any(|digest| candidates.blob_digests.contains(digest)) {
        return true;
    }
    summary.chunk_digests.iter().any(|digest| candidates.chunk_digests.contains(digest))
}

fn apply_frontier_hits(
    summary: &ReceiverFrontierSummary,
    candidates: &SenderCandidates,
    manifest: &mut ReceiverManifest,
) {
    for digest in &summary.directory_digests {
        if candidates.directory_digests.contains(digest) {
            manifest_insert_known_directory(manifest, *digest);
        }
    }
    for digest in &summary.blob_digests {
        if candidates.blob_digests.contains(digest) {
            manifest_insert_known_blob(manifest, *digest);
        }
    }
    for digest in &summary.chunk_digests {
        if candidates.chunk_digests.contains(digest) {
            manifest_insert_known_chunk(manifest, *digest);
        }
    }
}

#[allow(tigerstyle::no_recursion)] // tree walk bounded by fixture tree depth
fn collect_lossy_sender_candidates(
    node: &ArtifactNode,
    directory_buckets: &mut HashSet<u8>,
    blob_buckets: &mut HashSet<u8>,
    chunk_buckets: &mut HashSet<u8>,
) {
    match node {
        ArtifactNode::Directory(directory) => {
            directory_buckets.insert(lossy_digest_bucket(&b3_digest_from_delta(directory.digest)));
            for child in &directory.children {
                collect_lossy_sender_candidates(child, directory_buckets, blob_buckets, chunk_buckets);
            }
        }
        ArtifactNode::Blob(blob) => {
            blob_buckets.insert(lossy_digest_bucket(&b3_digest_from_delta(blob.digest)));
            for chunk in &blob.chunks {
                chunk_buckets.insert(lossy_digest_bucket(&b3_digest_from_delta(chunk.digest)));
            }
        }
        ArtifactNode::Symlink { .. } => {}
    }
}

fn lossy_frontier_has_overlap(summary: &ReceiverLossyFrontierSummary, candidates: &LossySenderCandidates) -> bool {
    if summary.directory_buckets.iter().any(|bucket| candidates.directory_buckets.contains(bucket)) {
        return true;
    }
    if summary.blob_buckets.iter().any(|bucket| candidates.blob_buckets.contains(bucket)) {
        return true;
    }
    summary.chunk_buckets.iter().any(|bucket| candidates.chunk_buckets.contains(bucket))
}

fn lossy_digest_bucket(digest: &B3Digest) -> u8 {
    digest.as_ref()[0] >> 4
}

fn probabilistic_membership_bits(digest: &B3Digest, summary: &ReceiverProbabilisticFrontierSummary) -> u128 {
    let slot_count = u32::from(summary.filter_config.slot_count);
    let tap_count = usize::from(summary.filter_config.tap_count);
    assert!(slot_count > 0, "probabilistic summary slot count must be positive");
    assert!(slot_count <= 128, "probabilistic summary slot count exceeds u128 capacity");
    assert!(tap_count > 0, "probabilistic summary tap count must be positive");
    assert!(tap_count <= 4, "probabilistic summary tap count must stay bounded");

    let mut filter_bits = 0u128;
    for byte in digest.as_ref().iter().take(tap_count) {
        let bit_index = u32::from(*byte) % slot_count;
        filter_bits |= 1u128 << bit_index;
    }
    filter_bits
}

fn probabilistic_frontier_has_overlap(
    summary: &ReceiverProbabilisticFrontierSummary,
    candidates: &ProbabilisticSenderCandidates,
) -> bool {
    debug_assert!(summary.filter_config.slot_count > 0);
    debug_assert!(summary.filter_config.tap_count > 0);
    if candidates.directory_digests.iter().any(|digest| {
        let membership_bits = probabilistic_membership_bits(digest, summary);
        membership_bits & summary.directory_filter_bits == membership_bits
    }) {
        return true;
    }
    if let Some(blob_filter_bits) = summary.blob_filter_bits
        && candidates.blob_digests.iter().any(|digest| {
            let membership_bits = probabilistic_membership_bits(digest, summary);
            membership_bits & blob_filter_bits == membership_bits
        })
    {
        return true;
    }
    let Some(chunk_filter_bits) = summary.chunk_filter_bits else {
        return false;
    };
    candidates.chunk_digests.iter().any(|digest| {
        let membership_bits = probabilistic_membership_bits(digest, summary);
        membership_bits & chunk_filter_bits == membership_bits
    })
}

#[derive(Default)]
struct TraversalMemo {
    seen_directories: HashSet<B3Digest>,
    seen_blobs: HashSet<B3Digest>,
}

async fn load_seeded_directory_if_present(
    services: &FixtureManifestStore,
    digest: B3Digest,
) -> Result<Option<Directory>, ManifestError> {
    Ok(services.directory(&digest))
}

async fn load_seeded_chunk_metadata_if_present(
    services: &FixtureManifestStore,
    digest: B3Digest,
) -> Result<Option<Vec<ChunkMeta>>, ManifestError> {
    Ok(services.chunk_metadata(&digest))
}

#[allow(tigerstyle::too_many_parameters)] // manifest walk threads shared state through recursive traversal
async fn walk_local_node(
    node: &ArtifactNode,
    frontiers: Option<&HashMap<B3Digest, &ReceiverFrontierSummary>>,
    services: &FixtureManifestStore,
    candidates: &SenderCandidates,
    chunk_profile: &ChunkProfile,
    traversal: &mut TraversalMemo,
    manifest: &mut ReceiverManifest,
) -> Result<(), ManifestError> {
    debug_assert!(chunk_profile.min_chunk_bytes > 0);
    debug_assert!(!manifest.store_prefix.is_empty());
    let mut pending = vec![seed_child_node(node)];
    while let Some(next) = pending.pop() {
        match next {
            Node::Directory { digest, .. } => {
                if candidates.directory_digests.contains(&digest) {
                    manifest_insert_known_directory(manifest, digest);
                    continue;
                }
                if let Some(summary) = frontiers.and_then(|frontiers| frontiers.get(&digest)) {
                    if !frontier_has_overlap(summary, candidates) {
                        continue;
                    }
                    apply_frontier_hits(summary, candidates, manifest);
                    continue;
                }
                if !traversal.seen_directories.insert(digest) {
                    continue;
                }
                let Some(resolved) = load_seeded_directory_if_present(services, digest).await? else {
                    continue;
                };
                for (_name, child) in resolved.nodes() {
                    pending.push(child.clone());
                }
            }
            Node::File {
                digest,
                size,
                executable: _,
            } => {
                let blob = BlobNode {
                    digest,
                    size_bytes: size,
                    chunks: Vec::new(),
                };
                record_local_blob(&blob, services, candidates, chunk_profile, traversal, manifest).await?;
            }
            Node::Symlink { .. } => {}
        }
    }
    Ok(())
}

fn frontier_seed_nodes_lossy(frontiers: &HashMap<B3Digest, &ReceiverLossyFrontierSummary>) -> Vec<Node> {
    let mut seeds = Vec::with_capacity(frontiers.len());
    for digest in frontiers.keys() {
        seeds.push(Node::Directory {
            digest: *digest,
            size: 0,
        });
    }
    seeds
}

fn frontier_seed_nodes_probabilistic(
    frontiers: &HashMap<B3Digest, &ReceiverProbabilisticFrontierSummary>,
) -> Vec<Node> {
    let mut seeds = Vec::with_capacity(frontiers.len());
    for digest in frontiers.keys() {
        seeds.push(Node::Directory {
            digest: *digest,
            size: 0,
        });
    }
    seeds
}

#[allow(tigerstyle::too_many_parameters)] // manifest walk threads shared state through recursive traversal
async fn walk_local_nodes_lossy(
    seeds: Vec<Node>,
    lossy_frontiers: Option<&HashMap<B3Digest, &ReceiverLossyFrontierSummary>>,
    services: &FixtureManifestStore,
    exact_candidates: &SenderCandidates,
    lossy_candidates: &LossySenderCandidates,
    chunk_profile: &ChunkProfile,
    traversal: &mut TraversalMemo,
    manifest: &mut ReceiverManifest,
) -> Result<(), ManifestError> {
    debug_assert!(chunk_profile.min_chunk_bytes > 0);
    debug_assert!(!manifest.store_prefix.is_empty());
    let mut pending = seeds;
    while let Some(next) = pending.pop() {
        match next {
            Node::Directory { digest, .. } => {
                if exact_candidates.directory_digests.contains(&digest) {
                    manifest_insert_known_directory(manifest, digest);
                    continue;
                }
                if let Some(summary) = lossy_frontiers.and_then(|frontiers| frontiers.get(&digest))
                    && !lossy_frontier_has_overlap(summary, lossy_candidates)
                {
                    continue;
                }
                if !traversal.seen_directories.insert(digest) {
                    continue;
                }
                let Some(resolved) = load_seeded_directory_if_present(services, digest).await? else {
                    continue;
                };
                for (_name, child) in resolved.nodes() {
                    pending.push(child.clone());
                }
            }
            Node::File {
                digest,
                size,
                executable: _,
            } => {
                let blob = BlobNode {
                    digest,
                    size_bytes: size,
                    chunks: Vec::new(),
                };
                record_local_blob_lossy(
                    &blob,
                    services,
                    exact_candidates,
                    lossy_candidates,
                    chunk_profile,
                    traversal,
                    manifest,
                )
                .await?;
            }
            Node::Symlink { .. } => {}
        }
    }
    Ok(())
}

#[allow(tigerstyle::too_many_parameters)] // manifest walk threads shared state through recursive traversal
async fn walk_local_nodes_probabilistic(
    seeds: Vec<Node>,
    probabilistic_frontiers: Option<&HashMap<B3Digest, &ReceiverProbabilisticFrontierSummary>>,
    services: &FixtureManifestStore,
    exact_candidates: &SenderCandidates,
    probabilistic_candidates: &ProbabilisticSenderCandidates,
    chunk_profile: &ChunkProfile,
    traversal: &mut TraversalMemo,
    manifest: &mut ReceiverManifest,
) -> Result<(), ManifestError> {
    debug_assert!(chunk_profile.min_chunk_bytes > 0);
    debug_assert!(!manifest.store_prefix.is_empty());
    let mut pending = seeds;
    while let Some(next) = pending.pop() {
        match next {
            Node::Directory { digest, .. } => {
                if exact_candidates.directory_digests.contains(&digest) {
                    manifest_insert_known_directory(manifest, digest);
                    continue;
                }
                if let Some(summary) = probabilistic_frontiers.and_then(|frontiers| frontiers.get(&digest))
                    && !probabilistic_frontier_has_overlap(summary, probabilistic_candidates)
                {
                    continue;
                }
                if !traversal.seen_directories.insert(digest) {
                    continue;
                }
                let Some(resolved) = load_seeded_directory_if_present(services, digest).await? else {
                    continue;
                };
                for (_name, child) in resolved.nodes() {
                    pending.push(child.clone());
                }
            }
            Node::File {
                digest,
                size,
                executable: _,
            } => {
                let blob = BlobNode {
                    digest,
                    size_bytes: size,
                    chunks: Vec::new(),
                };
                record_local_blob_probabilistic(
                    &blob,
                    services,
                    exact_candidates,
                    probabilistic_candidates,
                    chunk_profile,
                    traversal,
                    manifest,
                )
                .await?;
            }
            Node::Symlink { .. } => {}
        }
    }
    Ok(())
}

#[allow(tigerstyle::too_many_parameters)] // manifest walk threads shared state through recursive traversal
async fn record_local_blob(
    blob: &BlobNode,
    services: &FixtureManifestStore,
    candidates: &SenderCandidates,
    chunk_profile: &ChunkProfile,
    traversal: &mut TraversalMemo,
    manifest: &mut ReceiverManifest,
) -> Result<(), ManifestError> {
    debug_assert!(chunk_profile.min_chunk_bytes > 0);
    let blob_digest = b3_digest_from_delta(blob.digest);
    debug_assert!(!blob_digest.as_ref().is_empty());
    if candidates.blob_digests.contains(&blob_digest) {
        manifest_insert_known_blob(manifest, blob_digest);
        return Ok(());
    }
    if !traversal.seen_blobs.insert(blob_digest) {
        return Ok(());
    }
    if blob.size_bytes <= u64::from(chunk_profile.min_chunk_bytes) {
        return Ok(());
    }
    if candidates.chunk_digests.is_empty() {
        return Ok(());
    }

    let Some(chunk_meta) = load_seeded_chunk_metadata_if_present(services, blob_digest).await? else {
        return Ok(());
    };
    for chunk in chunk_meta {
        let digest = B3Digest::try_from(chunk.digest.to_vec())
            .map_err(|_| ManifestError::InvalidChunkDigestLen(chunk.digest.len()))?;
        if candidates.chunk_digests.contains(&digest) {
            manifest_insert_known_chunk(manifest, digest);
        }
    }
    Ok(())
}

#[allow(tigerstyle::too_many_parameters)] // manifest walk threads shared state through recursive traversal
async fn record_local_blob_lossy(
    blob: &BlobNode,
    services: &FixtureManifestStore,
    exact_candidates: &SenderCandidates,
    lossy_candidates: &LossySenderCandidates,
    chunk_profile: &ChunkProfile,
    traversal: &mut TraversalMemo,
    manifest: &mut ReceiverManifest,
) -> Result<(), ManifestError> {
    debug_assert!(chunk_profile.min_chunk_bytes > 0);
    let blob_digest = b3_digest_from_delta(blob.digest);
    debug_assert!(!blob_digest.as_ref().is_empty());
    if exact_candidates.blob_digests.contains(&blob_digest) {
        manifest_insert_known_blob(manifest, blob_digest);
        return Ok(());
    }
    if !traversal.seen_blobs.insert(blob_digest) {
        return Ok(());
    }
    if blob.size_bytes <= u64::from(chunk_profile.min_chunk_bytes) {
        return Ok(());
    }
    if lossy_candidates.chunk_buckets.is_empty() {
        return Ok(());
    }
    let Some(chunk_meta) = load_seeded_chunk_metadata_if_present(services, blob_digest).await? else {
        return Ok(());
    };
    for chunk in chunk_meta {
        let digest = B3Digest::try_from(chunk.digest.to_vec())
            .map_err(|_| ManifestError::InvalidChunkDigestLen(chunk.digest.len()))?;
        let bucket = lossy_digest_bucket(&digest);
        if lossy_candidates.chunk_buckets.contains(&bucket) && exact_candidates.chunk_digests.contains(&digest) {
            manifest_insert_known_chunk(manifest, digest);
        }
    }
    Ok(())
}

#[allow(tigerstyle::too_many_parameters)] // manifest walk threads shared state through recursive traversal
async fn record_local_blob_probabilistic(
    blob: &BlobNode,
    services: &FixtureManifestStore,
    exact_candidates: &SenderCandidates,
    probabilistic_candidates: &ProbabilisticSenderCandidates,
    chunk_profile: &ChunkProfile,
    traversal: &mut TraversalMemo,
    manifest: &mut ReceiverManifest,
) -> Result<(), ManifestError> {
    debug_assert!(chunk_profile.min_chunk_bytes > 0);
    let blob_digest = b3_digest_from_delta(blob.digest);
    debug_assert!(!blob_digest.as_ref().is_empty());
    if exact_candidates.blob_digests.contains(&blob_digest) {
        manifest_insert_known_blob(manifest, blob_digest);
        return Ok(());
    }
    if !traversal.seen_blobs.insert(blob_digest) {
        return Ok(());
    }
    if blob.size_bytes <= u64::from(chunk_profile.min_chunk_bytes) {
        return Ok(());
    }
    if probabilistic_candidates.chunk_digests.is_empty() {
        return Ok(());
    }
    let Some(chunk_meta) = load_seeded_chunk_metadata_if_present(services, blob_digest).await? else {
        return Ok(());
    };
    for chunk in chunk_meta {
        let digest = B3Digest::try_from(chunk.digest.to_vec())
            .map_err(|_| ManifestError::InvalidChunkDigestLen(chunk.digest.len()))?;
        if probabilistic_candidates.chunk_digests.contains(&digest) {
            manifest_insert_known_chunk(manifest, digest);
        }
    }
    Ok(())
}

struct FixtureManifestStore {
    counters: Arc<ProbeCounters>,
    directories: HashMap<B3Digest, Directory>,
    blobs: HashMap<B3Digest, Vec<ChunkMeta>>,
}

impl FixtureManifestStore {
    fn from_fixture(fixture: &ClosureFixture) -> Self {
        let counters = Arc::new(ProbeCounters::default());
        let mut directories = HashMap::<B3Digest, Directory>::new();
        let mut blobs = HashMap::<B3Digest, Vec<ChunkMeta>>::new();

        for output in &fixture.outputs {
            seed_node(&output.root, &mut directories, &mut blobs);
        }

        Self {
            counters,
            directories,
            blobs,
        }
    }

    fn directory(&self, digest: &B3Digest) -> Option<Directory> {
        self.counters.directory_gets.fetch_add(1, Ordering::Relaxed);
        self.directories.get(digest).cloned()
    }

    fn chunk_metadata(&self, digest: &B3Digest) -> Option<Vec<ChunkMeta>> {
        self.counters.chunk_queries.fetch_add(1, Ordering::Relaxed);
        self.blobs.get(digest).cloned()
    }

    fn finish_counts(&self, output_checks: u64) -> ManifestProbeCounts {
        ManifestProbeCounts {
            output_checks,
            directory_gets: self.counters.directory_gets.load(Ordering::Relaxed),
            chunk_queries: self.counters.chunk_queries.load(Ordering::Relaxed),
        }
    }

    #[cfg(test)]
    fn blob_read_opens(&self) -> u64 {
        self.counters.blob_read_opens.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
struct ProbeCounters {
    directory_gets: AtomicU64,
    chunk_queries: AtomicU64,
    #[cfg(test)]
    blob_read_opens: AtomicU64,
}

#[allow(tigerstyle::no_recursion)] // tree walk bounded by fixture tree depth
#[allow(tigerstyle::no_unwrap)] // fixture construction with hardcoded valid values
#[allow(tigerstyle::unbounded_collection_growth)] // fixture maps bounded by fixture tree size
fn seed_node(
    node: &ArtifactNode,
    directories: &mut HashMap<B3Digest, Directory>,
    blobs: &mut HashMap<B3Digest, Vec<ChunkMeta>>,
) {
    match node {
        ArtifactNode::Directory(directory) => {
            let mut entries = Vec::new();
            for (index, child) in directory.children.iter().enumerate() {
                seed_node(child, directories, blobs);
                let name_text = format!("n{index}");
                let name = PathComponent::try_from(name_text.as_str()).expect("synthetic path component");
                entries.push((name, seed_child_node(child)));
            }
            let dir = Directory::try_from_iter(entries).expect("fixture directory must be valid");
            directories.insert(b3_digest_from_delta(directory.digest), dir);
        }
        ArtifactNode::Blob(blob) => {
            let chunk_meta = blob
                .chunks
                .iter()
                .map(|chunk| ChunkMeta {
                    digest: Bytes::copy_from_slice(b3_digest_from_delta(chunk.digest).as_ref()),
                    size: chunk.size_bytes,
                })
                .collect::<Vec<_>>();
            blobs.insert(b3_digest_from_delta(blob.digest), chunk_meta);
        }
        ArtifactNode::Symlink { .. } => {}
    }
}

#[allow(tigerstyle::no_unwrap)] // fixture construction with hardcoded valid values
fn seed_child_node(node: &ArtifactNode) -> Node {
    match node {
        ArtifactNode::Directory(directory) => Node::Directory {
            digest: b3_digest_from_delta(directory.digest),
            size: directory.children.len() as u64,
        },
        ArtifactNode::Blob(blob) => Node::File {
            digest: b3_digest_from_delta(blob.digest),
            size: blob.size_bytes,
            executable: false,
        },
        ArtifactNode::Symlink { target } => Node::Symlink {
            target: SymlinkTarget::try_from(target.as_str()).expect("fixture symlink target"),
        },
    }
}

pub fn chunk_profile_for_manifest() -> ChunkProfile {
    let profile = chunk_profile_v1();
    assert!(profile.min_chunk_bytes > 0, "chunk profile min must be positive");
    profile
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_suite;
    use crate::build_receiver_manifest_lossy;
    use crate::build_receiver_manifest_probabilistic;
    use crate::model::ChunkRef;
    use crate::model::DirectoryNode;
    use crate::model::OutputFixture;
    use crate::plan_transfer;

    fn test_digest(label: &str) -> B3Digest {
        blake3::hash(label.as_bytes()).into()
    }

    fn test_chunk(label: &str, size_bytes: u64) -> ChunkRef {
        ChunkRef {
            digest: test_digest(&format!("chunk:{label}")),
            size_bytes,
        }
    }

    fn test_blob(label: &str, size_bytes: u64, chunks: Vec<ChunkRef>) -> ArtifactNode {
        ArtifactNode::Blob(BlobNode {
            digest: test_digest(&format!("blob:{label}")),
            size_bytes,
            chunks,
        })
    }

    fn test_dir(label: &str, children: Vec<ArtifactNode>) -> ArtifactNode {
        ArtifactNode::Directory(DirectoryNode {
            digest: test_digest(&format!("dir:{label}")),
            children,
        })
    }

    fn test_output(output_id: &str, root: ArtifactNode) -> OutputFixture {
        OutputFixture {
            output_id: output_id.to_owned(),
            root,
        }
    }

    fn test_fixture(store_prefix: &str, outputs: Vec<OutputFixture>) -> ClosureFixture {
        ClosureFixture {
            store_prefix: store_prefix.to_owned(),
            outputs,
        }
    }

    fn test_case(name: &'static str, sender: ClosureFixture, receiver_store: ClosureFixture) -> BenchCase {
        BenchCase {
            name,
            receiver: new_receiver_manifest(&sender.store_prefix),
            receiver_frontiers: Vec::new(),
            receiver_lossy_frontiers: Vec::new(),
            receiver_probabilistic_frontiers: Vec::new(),
            frontier_complete_outputs: HashSet::new(),
            expected_full_bytes: crate::model::closure_full_transfer_bytes(&sender),
            expected_coarse_bytes: crate::model::closure_full_transfer_bytes(&sender),
            sender,
            receiver_store,
        }
    }

    fn fixture_store_with_omissions(
        fixture: &ClosureFixture,
        missing_directories: &[B3Digest],
        missing_blobs: &[B3Digest],
    ) -> FixtureManifestStore {
        let counters = Arc::new(ProbeCounters::default());
        let mut directories = HashMap::<B3Digest, Directory>::new();
        let mut blobs = HashMap::<B3Digest, Vec<ChunkMeta>>::new();
        for output in &fixture.outputs {
            seed_node(&output.root, &mut directories, &mut blobs);
        }
        for digest in missing_directories {
            directories.remove(digest);
        }
        for digest in missing_blobs {
            blobs.remove(digest);
        }
        FixtureManifestStore {
            counters,
            directories,
            blobs,
        }
    }

    fn assert_manifest_is_empty(outcome: &ManifestBuildOutcome) {
        assert!(outcome.manifest.known_outputs.is_empty());
        assert!(outcome.manifest.known_directories.is_empty());
        assert!(outcome.manifest.known_blobs.is_empty());
        assert!(outcome.manifest.known_chunks.is_empty());
    }

    #[tokio::test]
    async fn prefix_mismatch_is_rejected_for_all_manifest_builders() {
        let sender = test_fixture("/crunch/store", vec![test_output("sender", test_blob("sender", 4_096, Vec::new()))]);
        let receiver_store =
            test_fixture("/nix/store", vec![test_output("receiver", test_blob("receiver", 4_096, Vec::new()))]);
        let case = test_case("prefix-mismatch", sender, receiver_store);
        let expected = ManifestError::StorePrefixMismatch {
            sender_prefix: "/crunch/store".to_owned(),
            receiver_prefix: "/nix/store".to_owned(),
        };

        assert_eq!(build_receiver_manifest(&case).await.expect_err("exact mismatch must fail"), expected);
        assert_eq!(build_receiver_manifest_lossy(&case).await.expect_err("lossy mismatch must fail"), expected);
        assert_eq!(
            build_receiver_manifest_probabilistic(&case).await.expect_err("probabilistic mismatch must fail"),
            expected
        );
    }

    #[tokio::test]
    async fn manifest_uses_chunk_metadata_without_opening_blob_content() {
        let shared_chunk = test_chunk("shared", 262_144);
        let sender = test_fixture("/crunch/store", vec![test_output(
            "sender",
            test_dir("sender-root", vec![test_blob("sender", 524_288, vec![
                shared_chunk,
                test_chunk("sender-tail", 262_144),
            ])]),
        )]);
        let receiver_store = test_fixture("/crunch/store", vec![test_output(
            "receiver",
            test_dir("receiver-root", vec![test_blob("receiver", 524_288, vec![
                shared_chunk,
                test_chunk("receiver-tail", 262_144),
            ])]),
        )]);
        let case = test_case("no-content-download", sender, receiver_store);
        let services = FixtureManifestStore::from_fixture(&case.receiver_store);

        let outcome =
            build_receiver_manifest_with_store(&case, &services).await.expect("manifest build should succeed");

        assert_eq!(outcome.probes.chunk_queries, 1);
        assert_eq!(services.blob_read_opens(), 0);
        assert_eq!(
            crate::model::manifest_known_chunks_as_b3(&outcome.manifest),
            HashSet::from([b3_digest_from_delta(shared_chunk.digest)])
        );
    }

    #[tokio::test]
    async fn stale_missing_directory_is_treated_as_absent() {
        let sender = test_fixture("/crunch/store", vec![test_output("sender", test_blob("sender", 4_096, Vec::new()))]);
        let stale_root = test_dir("stale-root", vec![test_blob("stale-child", 4_096, Vec::new())]);
        let stale_digest = match &stale_root {
            ArtifactNode::Directory(directory) => directory.digest,
            _ => unreachable!("stale root must be a directory"),
        };
        let receiver_store = test_fixture("/crunch/store", vec![test_output("receiver", stale_root)]);
        let case = test_case("stale-directory", sender, receiver_store);
        let stale_directory_digest = b3_digest_from_delta(stale_digest);
        let exact_services = fixture_store_with_omissions(&case.receiver_store, &[stale_directory_digest], &[]);
        let lossy_services = fixture_store_with_omissions(&case.receiver_store, &[stale_directory_digest], &[]);
        let probabilistic_services = fixture_store_with_omissions(&case.receiver_store, &[stale_directory_digest], &[]);

        let exact = build_receiver_manifest_with_store(&case, &exact_services)
            .await
            .expect("exact stale directory should be treated as absent");
        let lossy = build_receiver_manifest_lossy_with_store(&case, &lossy_services)
            .await
            .expect("lossy stale directory should be treated as absent");
        let probabilistic = build_receiver_manifest_probabilistic_with_store(&case, &probabilistic_services)
            .await
            .expect("probabilistic stale directory should be treated as absent");

        assert_manifest_is_empty(&exact);
        assert_manifest_is_empty(&lossy);
        assert_manifest_is_empty(&probabilistic);
        assert_eq!(exact.probes.directory_gets, 1);
        assert_eq!(lossy.probes.directory_gets, 1);
        assert_eq!(probabilistic.probes.directory_gets, 1);
    }

    #[tokio::test]
    async fn stale_missing_blob_is_treated_as_absent() {
        let shared_chunk = test_chunk("shared-stale", 262_144);
        let sender = test_fixture("/crunch/store", vec![test_output(
            "sender",
            test_dir("sender-root-stale", vec![test_blob("sender-stale", 524_288, vec![
                shared_chunk,
                test_chunk("sender-stale-tail", 262_144),
            ])]),
        )]);
        let receiver_blob =
            test_blob("receiver-stale", 524_288, vec![shared_chunk, test_chunk("receiver-stale-tail", 262_144)]);
        let stale_blob_digest = match &receiver_blob {
            ArtifactNode::Blob(blob) => blob.digest,
            _ => unreachable!("receiver blob must be a blob"),
        };
        let receiver_store = test_fixture("/crunch/store", vec![test_output(
            "receiver",
            test_dir("receiver-root-stale", vec![receiver_blob]),
        )]);
        let case = test_case("stale-blob", sender, receiver_store);
        let stale_receiver_blob_digest = b3_digest_from_delta(stale_blob_digest);
        let exact_services = fixture_store_with_omissions(&case.receiver_store, &[], &[stale_receiver_blob_digest]);
        let lossy_services = fixture_store_with_omissions(&case.receiver_store, &[], &[stale_receiver_blob_digest]);
        let probabilistic_services =
            fixture_store_with_omissions(&case.receiver_store, &[], &[stale_receiver_blob_digest]);

        let exact = build_receiver_manifest_with_store(&case, &exact_services)
            .await
            .expect("exact stale blob should be treated as absent");
        let lossy = build_receiver_manifest_lossy_with_store(&case, &lossy_services)
            .await
            .expect("lossy stale blob should be treated as absent");
        let probabilistic = build_receiver_manifest_probabilistic_with_store(&case, &probabilistic_services)
            .await
            .expect("probabilistic stale blob should be treated as absent");

        assert_manifest_is_empty(&exact);
        assert_manifest_is_empty(&lossy);
        assert_manifest_is_empty(&probabilistic);
        assert_eq!(exact.probes.chunk_queries, 1);
        assert_eq!(lossy.probes.chunk_queries, 1);
        assert_eq!(probabilistic.probes.chunk_queries, 1);
        assert_eq!(exact_services.blob_read_opens(), 0);
        assert_eq!(lossy_services.blob_read_opens(), 0);
        assert_eq!(probabilistic_services.blob_read_opens(), 0);
    }

    #[tokio::test]
    async fn manifest_matches_oracle_wire_bytes() {
        let suite = bench_suite();
        for case in &suite.cases {
            let outcome = build_receiver_manifest(case).await.expect("manifest build should succeed");
            let plan = plan_transfer(&case.sender, &outcome.manifest).expect("plan should succeed");
            assert_eq!(plan.transferred_bytes, case.expected_coarse_bytes, "case {}", case.name);
        }
    }

    #[tokio::test]
    async fn optimized_manifest_probe_totals_are_fixed() {
        let suite = bench_suite();
        let mut total = 0u64;
        for case in &suite.cases {
            let outcome = build_receiver_manifest(case).await.expect("manifest build should succeed");
            total = total.saturating_add(outcome.probes.total_probes());
            if case.name == "subtree-hit" {
                assert_eq!(outcome.probes.total_probes(), 1);
            }
            if case.name == "blob-hit" {
                assert_eq!(outcome.probes.total_probes(), 1);
            }
            if case.name == "chunk-hit" {
                assert_eq!(outcome.probes.total_probes(), 1);
            }
            if case.name == "cross-output" {
                assert_eq!(outcome.probes.total_probes(), 2);
            }
        }
        assert_eq!(total, 6);
    }

    #[tokio::test]
    async fn lossy_manifest_probe_totals_are_fixed() {
        let suite = bench_suite();
        let mut total = 0u64;
        for case in &suite.cases {
            let outcome = build_receiver_manifest_lossy(case).await.expect("lossy manifest build should succeed");
            let plan = plan_transfer(&case.sender, &outcome.manifest).expect("plan should succeed");
            assert_eq!(plan.transferred_bytes, case.expected_coarse_bytes, "case {}", case.name);
            total = total.saturating_add(outcome.probes.total_probes());
            if case.name == "subtree-hit" {
                assert_eq!(outcome.probes.total_probes(), 9);
            }
            if case.name == "blob-hit" {
                assert_eq!(outcome.probes.total_probes(), 2);
            }
            if case.name == "chunk-hit" {
                assert_eq!(outcome.probes.total_probes(), 4);
            }
            if case.name == "cross-output" {
                assert_eq!(outcome.probes.total_probes(), 2);
            }
        }
        assert_eq!(total, 18);
    }

    #[tokio::test]
    async fn probabilistic_manifest_probe_totals_are_fixed() {
        let suite = bench_suite();
        let mut total = 0u64;
        for case in &suite.cases {
            let outcome = build_receiver_manifest_probabilistic(case)
                .await
                .expect("probabilistic manifest build should succeed");
            let plan = plan_transfer(&case.sender, &outcome.manifest).expect("plan should succeed");
            assert_eq!(plan.transferred_bytes, case.expected_coarse_bytes, "case {}", case.name);
            total = total.saturating_add(outcome.probes.total_probes());
            if case.name == "subtree-hit" {
                assert_eq!(outcome.probes.total_probes(), 9);
            }
            if case.name == "blob-hit" {
                assert_eq!(outcome.probes.total_probes(), 2);
            }
            if case.name == "chunk-hit" {
                assert_eq!(outcome.probes.total_probes(), 3);
            }
            if case.name == "cross-output" {
                assert_eq!(outcome.probes.total_probes(), 2);
            }
        }
        assert_eq!(total, 17);
    }
}
