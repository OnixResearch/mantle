use std::collections::HashMap;
use std::collections::HashSet;

use snix_castore::B3Digest;

use crate::model::ArtifactNode;
use crate::model::BlobNode;
use crate::model::ChunkRef;
use crate::model::ClosureFixture;
use crate::model::DirectoryNode;
use crate::model::OutputFixture;
use crate::model::ReceiverManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchCase {
    pub name: &'static str,
    pub sender: ClosureFixture,
    pub receiver: ReceiverManifest,
    pub receiver_store: ClosureFixture,
    pub receiver_frontiers: Vec<ReceiverFrontierSummary>,
    pub receiver_lossy_frontiers: Vec<ReceiverLossyFrontierSummary>,
    pub receiver_probabilistic_frontiers: Vec<ReceiverProbabilisticFrontierSummary>,
    pub frontier_complete_outputs: HashSet<String>,
    pub expected_full_bytes: u64,
    pub expected_coarse_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverLossyFrontierSummary {
    pub owner_output_id: String,
    pub directory_digest: B3Digest,
    pub directory_buckets: HashSet<u8>,
    pub blob_buckets: HashSet<u8>,
    pub chunk_buckets: HashSet<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbabilisticFilterConfig {
    pub slot_count: u16,
    pub tap_count: u8,
}

impl ProbabilisticFilterConfig {
    pub fn filter_bytes_per_summary(self) -> u64 {
        let slot_bytes = u64::from(self.slot_count).div_ceil(8);
        slot_bytes.saturating_mul(3)
    }
}

pub const DEFAULT_PROBABILISTIC_FILTER_CONFIG: ProbabilisticFilterConfig = ProbabilisticFilterConfig {
    slot_count: 8,
    tap_count: 3,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbabilisticWireLayout {
    #[expect(dead_code, reason = "sizing model keeps the naive layout as a comparison baseline")]
    NaivePerSummary,
    GroupedByOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupedOutputHeaderMode {
    #[expect(
        dead_code,
        reason = "sizing model keeps the separate mode byte layout as a comparison baseline"
    )]
    SeparateModeByte,
    PackedModeInSummaryCount,
}

pub const DEFAULT_PROBABILISTIC_WIRE_LAYOUT: ProbabilisticWireLayout = ProbabilisticWireLayout::GroupedByOutput;
pub const DEFAULT_GROUPED_OUTPUT_HEADER_MODE: GroupedOutputHeaderMode =
    GroupedOutputHeaderMode::PackedModeInSummaryCount;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverProbabilisticFrontierSummary {
    pub owner_output_id: String,
    pub directory_digest: B3Digest,
    pub filter_config: ProbabilisticFilterConfig,
    pub directory_filter_bits: u128,
    pub blob_filter_bits: Option<u128>,
    pub chunk_filter_bits: Option<u128>,
}

impl ReceiverProbabilisticFrontierSummary {
    pub fn payload_bytes(&self) -> u64 {
        let slot_bytes = u64::from(self.filter_config.slot_count).div_ceil(8);
        let blob_filter_count = u64::from(self.blob_filter_bits.is_some());
        let chunk_filter_count = u64::from(self.chunk_filter_bits.is_some());
        let filter_count = 1u64.saturating_add(blob_filter_count).saturating_add(chunk_filter_count);
        slot_bytes.saturating_mul(filter_count)
    }

    pub fn wire_entry_bytes(&self) -> u64 {
        self.wire_entry_bytes_with_ref_bytes(32)
    }

    pub fn wire_entry_bytes_with_ref_bytes(&self, ref_bytes: u64) -> u64 {
        let presence_bytes = 1u64;
        ref_bytes.saturating_add(presence_bytes).saturating_add(self.payload_bytes())
    }

    pub fn wire_payload_bytes_with_ref_bytes(&self, ref_bytes: u64) -> u64 {
        ref_bytes.saturating_add(self.payload_bytes())
    }

    pub fn packed_optional_filter_tag(&self) -> Option<u8> {
        match (self.blob_filter_bits.is_some(), self.chunk_filter_bits.is_some()) {
            (false, false) => Some(0),
            (true, false) => Some(1),
            (false, true) => Some(2),
            (true, true) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiverFrontierSummary {
    pub owner_output_id: String,
    pub directory_digest: B3Digest,
    pub directory_digests: HashSet<B3Digest>,
    pub blob_digests: HashSet<B3Digest>,
    pub chunk_digests: HashSet<B3Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchSuite {
    pub cases: Vec<BenchCase>,
}

impl BenchSuite {
    pub fn full_transfer_bytes_total(&self) -> u64 {
        self.cases.iter().fold(0u64, |total, case| total.saturating_add(case.expected_full_bytes))
    }

    pub fn coarse_transfer_bytes_total(&self) -> u64 {
        self.cases.iter().fold(0u64, |total, case| total.saturating_add(case.expected_coarse_bytes))
    }

    pub fn probabilistic_filter_bytes_total(&self) -> u64 {
        self.cases.iter().fold(0u64, |total, case| total.saturating_add(case.probabilistic_filter_bytes()))
    }

    pub fn probabilistic_summary_wire_bytes_total(&self) -> u64 {
        self.cases
            .iter()
            .fold(0u64, |total, case| total.saturating_add(case.probabilistic_summary_wire_bytes()))
    }
}

pub fn bench_suite() -> BenchSuite {
    let cases = vec![
        whole_output_hit_case(),
        subtree_hit_case(),
        blob_hit_case(),
        chunk_hit_case(),
        cross_output_case(),
    ];
    let suite = BenchSuite { cases };
    assert_eq!(DEFAULT_PROBABILISTIC_FILTER_CONFIG.tap_count, 3, "unexpected default probabilistic tap count");
    assert_eq!(suite.probabilistic_filter_bytes_total(), 562, "default probabilistic filter bytes changed");
    assert_eq!(suite.probabilistic_summary_wire_bytes_total(), 833, "default probabilistic wire bytes changed");
    assert_eq!(suite.cases.len(), 5, "benchmark suite must stay fixed");
    assert_eq!(suite.full_transfer_bytes_total(), 2_678_784, "full baseline bytes changed");
    assert_eq!(suite.coarse_transfer_bytes_total(), 450_560, "coarse target bytes changed");
    suite
}

fn whole_output_hit_case() -> BenchCase {
    let sender_output = OutputFixture {
        output_id: "whole-output-hit".to_owned(),
        root: blob_node("whole-output-hit-root", 65_536),
    };
    let sender = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![sender_output.clone()],
    };
    let mut receiver = ReceiverManifest::new("/crunch/store");
    receiver.known_outputs.insert("whole-output-hit".to_owned());
    let receiver_store = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![sender_output],
    };
    build_case("whole-output-hit", sender, receiver, receiver_store, Vec::new(), HashSet::new(), 0)
}

fn subtree_unmatched_dirs() -> Vec<ArtifactNode> {
    (b'a'..=b'h')
        .map(|ch| {
            let suffix = (ch as char).to_string();
            dir_node(
                &format!("subtree-unmatched-{suffix}"),
                vec![blob_node(&format!("subtree-unmatched-{suffix}-blob"), 4_096)],
            )
        })
        .collect()
}

fn subtree_hit_case() -> BenchCase {
    let shared_a_nested_dirs = numbered_leaf_dirs("subtree-shared-a-nested", 128, 512);
    let shared_b_nested_dirs = numbered_leaf_dirs("subtree-shared-b-nested", 128, 512);
    let shared_a = dir_node("subtree-shared-a", shared_a_nested_dirs.clone());
    let shared_b = dir_node("subtree-shared-b", shared_b_nested_dirs.clone());
    let shared_a_digest = digest_of(&shared_a);
    let shared_b_digest = digest_of(&shared_b);
    let unmatched = subtree_unmatched_dirs();
    debug_assert_eq!(unmatched.len(), 8);
    let sender = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![OutputFixture {
            output_id: "subtree-hit".to_owned(),
            root: dir_node("subtree-root", vec![
                shared_a.clone(),
                shared_b.clone(),
                blob_node("subtree-changed-blob", 16_384),
            ]),
        }],
    };
    let mut receiver = ReceiverManifest::new("/crunch/store");
    receiver.known_directories.insert(shared_a_digest);
    receiver.known_directories.insert(shared_b_digest);
    let mut local_children = unmatched.clone();
    local_children.push(shared_a.clone());
    local_children.push(shared_b.clone());
    let receiver_output = OutputFixture {
        output_id: "subtree-hit-local".to_owned(),
        root: dir_node("subtree-local-root", local_children),
    };
    let receiver_store = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![receiver_output.clone()],
    };
    let mut receiver_frontiers: Vec<ReceiverFrontierSummary> = unmatched
        .iter()
        .chain([&shared_a, &shared_b])
        .map(|node| frontier_summary(&receiver_output.output_id, node))
        .collect();
    debug_assert!(!receiver_frontiers.is_empty());
    receiver_frontiers
        .extend(shared_a_nested_dirs.iter().map(|node| frontier_summary(&receiver_output.output_id, node)));
    receiver_frontiers
        .extend(shared_b_nested_dirs.iter().map(|node| frontier_summary(&receiver_output.output_id, node)));
    build_case(
        "subtree-hit",
        sender,
        receiver,
        receiver_store,
        receiver_frontiers,
        HashSet::from([receiver_output.output_id.clone()]),
        16_384,
    )
}

fn blob_hit_case() -> BenchCase {
    let shared_blob = blob_node("blob-hit-shared", 65_536);
    let shared_digest = digest_of(&shared_blob);
    let shared_dir = dir_node("blob-hit-shared-dir", vec![shared_blob.clone()]);
    let unmatched_a = dir_node("blob-hit-unmatched-a", vec![blob_node("blob-hit-unmatched-a-blob", 4_096)]);
    let unmatched_b = dir_node("blob-hit-unmatched-b", vec![blob_node("blob-hit-unmatched-b-blob", 4_096)]);
    let unmatched_c = dir_node("blob-hit-unmatched-c", vec![blob_node("blob-hit-unmatched-c-blob", 4_096)]);
    let unmatched_d = dir_node("blob-hit-unmatched-d", vec![blob_node("blob-hit-unmatched-d-blob", 4_096)]);
    let sender = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![OutputFixture {
            output_id: "blob-hit".to_owned(),
            root: dir_node("blob-hit-root", vec![shared_blob, blob_node("blob-hit-changed", 32_768)]),
        }],
    };
    let mut receiver = ReceiverManifest::new("/crunch/store");
    receiver.known_blobs.insert(shared_digest);
    let receiver_output = OutputFixture {
        output_id: "blob-hit-local".to_owned(),
        root: dir_node("blob-hit-local-root", vec![
            shared_dir.clone(),
            unmatched_a.clone(),
            unmatched_b.clone(),
            unmatched_c.clone(),
            unmatched_d.clone(),
        ]),
    };
    let receiver_store = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![receiver_output.clone()],
    };
    let receiver_frontiers = vec![
        frontier_summary(&receiver_output.output_id, &shared_dir),
        frontier_summary(&receiver_output.output_id, &unmatched_a),
        frontier_summary(&receiver_output.output_id, &unmatched_b),
        frontier_summary(&receiver_output.output_id, &unmatched_c),
        frontier_summary(&receiver_output.output_id, &unmatched_d),
    ];
    build_case(
        "blob-hit",
        sender,
        receiver,
        receiver_store,
        receiver_frontiers,
        HashSet::from([receiver_output.output_id.clone()]),
        32_768,
    )
}

fn chunk_hit_case() -> BenchCase {
    let chunk_sizes_bytes = [262_144u64; 8];
    let sender_blob = chunked_blob("chunk-hit-large", &chunk_sizes_bytes);
    let sender_chunks = blob_chunks(&sender_blob);
    let sender = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![OutputFixture {
            output_id: "chunk-hit".to_owned(),
            root: dir_node("chunk-hit-root", vec![sender_blob]),
        }],
    };
    let mut receiver = ReceiverManifest::new("/crunch/store");
    for chunk in sender_chunks.iter().take(7) {
        receiver.known_chunks.insert(chunk.digest);
    }
    let mut receiver_chunks = sender_chunks.iter().take(7).cloned().collect::<Vec<_>>();
    receiver_chunks.push(ChunkRef {
        digest: digest("chunk:chunk-hit-local:tail"),
        size_bytes: 262_144,
    });
    let local_blob_dir =
        dir_node("chunk-hit-local-dir", vec![chunked_blob_from_chunks("chunk-hit-local", &receiver_chunks)]);
    let unmatched_a = dir_node("chunk-hit-unmatched-a", vec![blob_node("chunk-hit-unmatched-a-blob", 4_096)]);
    let unmatched_b = dir_node("chunk-hit-unmatched-b", vec![blob_node("chunk-hit-unmatched-b-blob", 4_096)]);
    let unmatched_c = dir_node("chunk-hit-unmatched-c", vec![blob_node("chunk-hit-unmatched-c-blob", 4_096)]);
    let receiver_output = OutputFixture {
        output_id: "chunk-hit-local".to_owned(),
        root: dir_node("chunk-hit-local-root", vec![
            local_blob_dir.clone(),
            unmatched_a.clone(),
            unmatched_b.clone(),
            unmatched_c.clone(),
        ]),
    };
    let receiver_store = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![receiver_output.clone()],
    };
    let receiver_frontiers = vec![
        frontier_summary(&receiver_output.output_id, &local_blob_dir),
        frontier_summary(&receiver_output.output_id, &unmatched_a),
        frontier_summary(&receiver_output.output_id, &unmatched_b),
        frontier_summary(&receiver_output.output_id, &unmatched_c),
    ];
    build_case(
        "chunk-hit",
        sender,
        receiver,
        receiver_store,
        receiver_frontiers,
        HashSet::from([receiver_output.output_id.clone()]),
        262_144,
    )
}

fn cross_output_case() -> BenchCase {
    let shared_blob = blob_node("cross-output-shared", 131_072);
    let sender = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![
            OutputFixture {
                output_id: "cross-output-a".to_owned(),
                root: dir_node("cross-output-a-root", vec![
                    shared_blob.clone(),
                    blob_node("cross-output-a-unique", 4_096),
                ]),
            },
            OutputFixture {
                output_id: "cross-output-b".to_owned(),
                root: dir_node("cross-output-b-root", vec![
                    shared_blob.clone(),
                    blob_node("cross-output-b-unique", 4_096),
                ]),
            },
        ],
    };
    let receiver = ReceiverManifest::new("/crunch/store");
    let local_shared_blob = blob_node("cross-output-local-shared", 131_072);
    let local_shared_inner_a = dir_node("cross-output-local-shared-inner-a", vec![local_shared_blob.clone()]);
    let local_shared_inner_b = dir_node("cross-output-local-shared-inner-b", vec![local_shared_blob]);
    let local_shared_dir_a = dir_node("cross-output-local-shared-dir-a", vec![local_shared_inner_a.clone()]);
    let local_shared_dir_b = dir_node("cross-output-local-shared-dir-b", vec![local_shared_inner_b.clone()]);
    let unmatched_a = dir_node("cross-output-unmatched-a", vec![blob_node("cross-output-unmatched-a-blob", 4_096)]);
    let unmatched_b = dir_node("cross-output-unmatched-b", vec![blob_node("cross-output-unmatched-b-blob", 4_096)]);
    let output_a = OutputFixture {
        output_id: "cross-output-local-a".to_owned(),
        root: dir_node("cross-output-local-a-root", vec![local_shared_dir_a.clone(), unmatched_a.clone()]),
    };
    let output_b = OutputFixture {
        output_id: "cross-output-local-b".to_owned(),
        root: dir_node("cross-output-local-b-root", vec![local_shared_dir_b.clone(), unmatched_b.clone()]),
    };
    let receiver_store = ClosureFixture {
        store_prefix: "/crunch/store".to_owned(),
        outputs: vec![output_a.clone(), output_b.clone()],
    };
    let receiver_frontiers = vec![
        frontier_summary(&output_a.output_id, &local_shared_dir_a),
        frontier_summary(&output_a.output_id, &unmatched_a),
        frontier_summary(&output_a.output_id, &local_shared_inner_a),
        frontier_summary(&output_b.output_id, &local_shared_dir_b),
        frontier_summary(&output_b.output_id, &unmatched_b),
        frontier_summary(&output_b.output_id, &local_shared_inner_b),
    ];
    build_case(
        "cross-output",
        sender,
        receiver,
        receiver_store,
        receiver_frontiers,
        HashSet::from([output_a.output_id.clone(), output_b.output_id.clone()]),
        139_264,
    )
}

#[allow(tigerstyle::too_many_parameters)] // fixture constructor threading test scenario parts
fn build_case(
    name: &'static str,
    sender: ClosureFixture,
    receiver: ReceiverManifest,
    receiver_store: ClosureFixture,
    receiver_frontiers: Vec<ReceiverFrontierSummary>,
    frontier_complete_outputs: HashSet<String>,
    expected_coarse_bytes: u64,
) -> BenchCase {
    let expected_full_bytes = sender.full_transfer_bytes();
    let receiver_lossy_frontiers = receiver_frontiers.iter().map(lossy_frontier_summary).collect::<Vec<_>>();
    let receiver_probabilistic_frontiers =
        receiver_frontiers.iter().map(probabilistic_frontier_summary).collect::<Vec<_>>();
    assert!(expected_full_bytes >= expected_coarse_bytes, "coarse plan must not exceed full transfer");
    assert_eq!(sender.store_prefix, receiver.store_prefix, "benchmark case prefixes must match");
    assert_eq!(sender.store_prefix, receiver_store.store_prefix, "receiver store prefix must match sender");
    BenchCase {
        name,
        sender,
        receiver,
        receiver_store,
        receiver_frontiers,
        receiver_lossy_frontiers,
        receiver_probabilistic_frontiers,
        frontier_complete_outputs,
        expected_full_bytes,
        expected_coarse_bytes,
    }
}

impl BenchCase {
    pub fn probabilistic_filter_bytes(&self) -> u64 {
        self.receiver_probabilistic_frontiers
            .iter()
            .fold(0u64, |total, summary| total.saturating_add(summary.payload_bytes()))
    }

    pub fn probabilistic_summary_wire_bytes(&self) -> u64 {
        let summary_count = self.receiver_probabilistic_frontiers.len() as u64;
        if summary_count == 0 {
            return 0;
        }

        let summary_entry_bytes = self
            .receiver_probabilistic_frontiers
            .iter()
            .fold(0u64, |total, summary| total.saturating_add(summary.wire_entry_bytes()));
        match DEFAULT_PROBABILISTIC_WIRE_LAYOUT {
            ProbabilisticWireLayout::NaivePerSummary => {
                let owner_ref_bytes = summary_count.saturating_mul(4);
                let config_bytes = if self.uses_default_probabilistic_filter_config() {
                    0
                } else {
                    summary_count.saturating_mul(3)
                };
                summary_entry_bytes.saturating_add(owner_ref_bytes).saturating_add(config_bytes)
            }
            ProbabilisticWireLayout::GroupedByOutput => {
                let case_config_bytes: u64 = if self.uses_default_probabilistic_filter_config() {
                    0
                } else {
                    3
                };
                let summarized_output_count = self.summarized_output_count();
                let has_full_output_coverage = summarized_output_count == self.receiver_store.outputs.len() as u64;
                let grouped_entry_bytes =
                    self.receiver_store.outputs.iter().enumerate().fold(0u64, |total, (index, output)| {
                        let output_summaries = self
                            .receiver_probabilistic_frontiers
                            .iter()
                            .filter(|summary| summary.owner_output_id == output.output_id)
                            .collect::<Vec<_>>();
                        if output_summaries.is_empty() {
                            return total;
                        }
                        let output_ref_bytes = if summarized_output_count == 1 || has_full_output_coverage {
                            0
                        } else {
                            u64_varint_bytes(index as u64)
                        };
                        let header_bytes = grouped_output_header_bytes(output, &output_summaries);
                        let entry_bytes = grouped_output_entry_bytes(output, &output_summaries);
                        total.saturating_add(output_ref_bytes).saturating_add(header_bytes).saturating_add(entry_bytes)
                    });
                case_config_bytes.saturating_add(grouped_entry_bytes)
            }
        }
    }

    fn summarized_output_count(&self) -> u64 {
        self.receiver_store.outputs.iter().fold(0u64, |total, output| {
            let has_summary = self
                .receiver_probabilistic_frontiers
                .iter()
                .any(|summary| summary.owner_output_id == output.output_id);
            total.saturating_add(u64::from(has_summary))
        })
    }

    fn uses_default_probabilistic_filter_config(&self) -> bool {
        self.receiver_probabilistic_frontiers
            .iter()
            .all(|summary| summary.filter_config == DEFAULT_PROBABILISTIC_FILTER_CONFIG)
    }
}

#[allow(tigerstyle::platform_dependent_cast)] // known-bounded fixture values
fn grouped_output_header_bytes(output: &OutputFixture, summaries: &[&ReceiverProbabilisticFrontierSummary]) -> u64 {
    let mode_bytes = match DEFAULT_GROUPED_OUTPUT_HEADER_MODE {
        GroupedOutputHeaderMode::SeparateModeByte => 1u64,
        GroupedOutputHeaderMode::PackedModeInSummaryCount => 0u64,
    };
    let summary_count_bytes = u64_varint_bytes(summaries.len() as u64);
    let coverage_count_bytes = explicit_complete_coverage_bytes(output, summaries);
    let split_count_bytes = mixed_root_child_split_count_bytes(output, summaries);
    let nested_parent_ref_bytes = uniform_nested_parent_ordinal_bytes(output, summaries);
    let nested_descendant_stream_mode_bytes = nested_descendant_stream_mode_bytes(output, summaries);
    if can_pack_uniform_optional_filter_tag_in_small_header(output, summaries) {
        return 1u64.saturating_add(coverage_count_bytes);
    }
    if can_pack_large_uniform_optional_filter_tag_in_grouped_header(summaries, split_count_bytes, coverage_count_bytes)
    {
        return 1;
    }
    if can_pack_large_uniform_optional_filter_tag_and_split_in_grouped_header(
        output,
        summaries,
        split_count_bytes,
        coverage_count_bytes,
    ) {
        return 1;
    }
    if DEFAULT_GROUPED_OUTPUT_HEADER_MODE == GroupedOutputHeaderMode::PackedModeInSummaryCount
        && packed_optional_filter_tag_bytes(summaries).is_some()
        && summaries.len() <= 2
    {
        return 1u64
            .saturating_add(split_count_bytes)
            .saturating_add(coverage_count_bytes)
            .saturating_add(nested_parent_ref_bytes)
            .saturating_add(nested_descendant_stream_mode_bytes);
    }
    let tag_bytes = grouped_optional_filter_tag_bytes(summaries);
    mode_bytes
        .saturating_add(summary_count_bytes)
        .saturating_add(tag_bytes)
        .saturating_add(split_count_bytes)
        .saturating_add(coverage_count_bytes)
        .saturating_add(nested_parent_ref_bytes)
        .saturating_add(nested_descendant_stream_mode_bytes)
}

#[allow(tigerstyle::platform_dependent_cast)] // known-bounded fixture values
#[allow(tigerstyle::raw_arithmetic_overflow)] // wire-size calculations with known-valid ordering
fn grouped_output_entry_bytes(output: &OutputFixture, summaries: &[&ReceiverProbabilisticFrontierSummary]) -> u64 {
    if complete_root_child_ordinals(output, summaries).is_some() {
        return summaries
            .iter()
            .fold(0u64, |total, summary| total.saturating_add(summary.wire_payload_bytes_with_ref_bytes(0)));
    }
    if let Some(root_child_prefix_len) = complete_root_child_prefix_len(output, summaries) {
        let root_child_prefix_len = root_child_prefix_len as usize;
        let prefix_bytes = summaries[..root_child_prefix_len]
            .iter()
            .fold(0u64, |total, summary| total.saturating_add(summary.wire_payload_bytes_with_ref_bytes(0)));
        let relative_refs_by_digest = nested_summary_relative_refs(output);
        let nested_payload_bytes = summaries[root_child_prefix_len..]
            .iter()
            .fold(0u64, |total, summary| total.saturating_add(summary.payload_bytes()));
        if let Some(relative_refs_by_digest) = relative_refs_by_digest.as_ref() {
            if let Some(delta_ref_bytes) =
                delta_coded_nested_descendant_ref_bytes(relative_refs_by_digest, summaries, root_child_prefix_len)
            {
                return prefix_bytes.saturating_add(nested_payload_bytes).saturating_add(delta_ref_bytes);
            }
            if let Some(run_ref_bytes) =
                run_coded_nested_descendant_ref_bytes(relative_refs_by_digest, summaries, root_child_prefix_len)
            {
                return prefix_bytes.saturating_add(nested_payload_bytes).saturating_add(run_ref_bytes);
            }
        }
        let uniform_parent_ordinal = relative_refs_by_digest
            .as_ref()
            .and_then(|refs_by_digest| uniform_nested_parent_ordinal(refs_by_digest, summaries, root_child_prefix_len));
        let nested_bytes = summaries[root_child_prefix_len..].iter().fold(0u64, |total, summary| {
            let relative_ref_bytes = relative_refs_by_digest
                .as_ref()
                .and_then(|refs_by_digest| refs_by_digest.get(&summary.directory_digest))
                .map(|relative_ref| {
                    if uniform_parent_ordinal == Some(relative_ref.root_child_ordinal) {
                        return u64_varint_bytes(relative_ref.descendant_ordinal);
                    }
                    nested_summary_relative_ref_bytes(relative_ref)
                })
                .unwrap_or(32);
            total.saturating_add(summary.wire_payload_bytes_with_ref_bytes(relative_ref_bytes))
        });
        return prefix_bytes.saturating_add(nested_bytes);
    }
    summaries
        .iter()
        .fold(0u64, |total, summary| total.saturating_add(summary.wire_payload_bytes_with_ref_bytes(32)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NestedSummaryRelativeRef {
    root_child_ordinal: u64,
    descendant_ordinal: u64,
}

fn nested_summary_relative_refs(output: &OutputFixture) -> Option<HashMap<B3Digest, NestedSummaryRelativeRef>> {
    let ArtifactNode::Directory(directory) = &output.root else {
        return None;
    };
    let mut refs_by_digest = HashMap::new();
    for (root_child_ordinal, child) in directory.children.iter().enumerate() {
        let mut descendant_ordinal = 0u64;
        collect_nested_summary_relative_refs(
            child,
            root_child_ordinal as u64,
            &mut descendant_ordinal,
            &mut refs_by_digest,
        )?;
    }
    Some(refs_by_digest)
}

#[allow(tigerstyle::no_recursion)] // tree walk bounded by fixture tree depth
fn collect_nested_summary_relative_refs(
    node: &ArtifactNode,
    root_child_ordinal: u64,
    descendant_ordinal: &mut u64,
    refs_by_digest: &mut HashMap<B3Digest, NestedSummaryRelativeRef>,
) -> Option<()> {
    let ArtifactNode::Directory(directory) = node else {
        return Some(());
    };
    for child in &directory.children {
        let ArtifactNode::Directory(child_directory) = child else {
            continue;
        };
        let relative_ref = NestedSummaryRelativeRef {
            root_child_ordinal,
            descendant_ordinal: *descendant_ordinal,
        };
        if refs_by_digest.insert(child_directory.digest, relative_ref).is_some() {
            return None;
        }
        *descendant_ordinal = descendant_ordinal.saturating_add(1);
        collect_nested_summary_relative_refs(child, root_child_ordinal, descendant_ordinal, refs_by_digest)?;
    }
    Some(())
}

fn nested_summary_relative_ref_bytes(relative_ref: &NestedSummaryRelativeRef) -> u64 {
    if relative_ref.root_child_ordinal <= 7 && relative_ref.descendant_ordinal <= 31 {
        return 1;
    }
    u64_varint_bytes(relative_ref.root_child_ordinal).saturating_add(u64_varint_bytes(relative_ref.descendant_ordinal))
}

#[allow(tigerstyle::platform_dependent_cast)] // known-bounded fixture values
fn uniform_nested_parent_ordinal_bytes(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> u64 {
    let Some(relative_refs_by_digest) = nested_summary_relative_refs(output) else {
        return 0;
    };
    let Some(root_child_prefix_len) = complete_root_child_prefix_len(output, summaries) else {
        return 0;
    };
    let Some(parent_ordinal) =
        uniform_nested_parent_ordinal(&relative_refs_by_digest, summaries, root_child_prefix_len as usize)
    else {
        return 0;
    };
    u64_varint_bytes(parent_ordinal)
}

#[allow(tigerstyle::platform_dependent_cast)] // known-bounded fixture values
fn nested_descendant_stream_mode_bytes(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> u64 {
    let Some(relative_refs_by_digest) = nested_summary_relative_refs(output) else {
        return 0;
    };
    let Some(root_child_prefix_len) = complete_root_child_prefix_len(output, summaries) else {
        return 0;
    };
    let root_child_prefix_len = root_child_prefix_len as usize;
    if delta_coded_nested_descendant_ref_bytes(&relative_refs_by_digest, summaries, root_child_prefix_len).is_some() {
        return 1;
    }
    u64::from(
        run_coded_nested_descendant_ref_bytes(&relative_refs_by_digest, summaries, root_child_prefix_len).is_some(),
    )
}

#[allow(tigerstyle::raw_arithmetic_overflow)] // wire-size delta with known-valid ordering
fn delta_coded_nested_descendant_ref_bytes(
    relative_refs_by_digest: &HashMap<B3Digest, NestedSummaryRelativeRef>,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
    root_child_prefix_len: usize,
) -> Option<u64> {
    let parent_ordinal = uniform_nested_parent_ordinal(relative_refs_by_digest, summaries, root_child_prefix_len)?;
    let mut total_ref_bytes = 0u64;
    let mut previous_descendant_ordinal = None;
    for summary in &summaries[root_child_prefix_len..] {
        let relative_ref = relative_refs_by_digest.get(&summary.directory_digest)?;
        if relative_ref.root_child_ordinal != parent_ordinal {
            return None;
        }
        let delta = match previous_descendant_ordinal {
            Some(previous_descendant_ordinal) => {
                if relative_ref.descendant_ordinal <= previous_descendant_ordinal {
                    return None;
                }
                relative_ref.descendant_ordinal - previous_descendant_ordinal
            }
            None => relative_ref.descendant_ordinal,
        };
        total_ref_bytes = total_ref_bytes.saturating_add(u64_varint_bytes(delta));
        previous_descendant_ordinal = Some(relative_ref.descendant_ordinal);
    }
    Some(total_ref_bytes)
}

#[allow(tigerstyle::raw_arithmetic_overflow)] // wire-size delta with known-valid ordering
fn run_coded_nested_descendant_ref_bytes(
    relative_refs_by_digest: &HashMap<B3Digest, NestedSummaryRelativeRef>,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
    root_child_prefix_len: usize,
) -> Option<u64> {
    let mut total_ref_bytes = 0u64;
    let mut current_parent_ordinal = None;
    let mut previous_descendant_ordinal = None;
    let mut run_count = 0u64;
    for summary in &summaries[root_child_prefix_len..] {
        let relative_ref = relative_refs_by_digest.get(&summary.directory_digest)?;
        if current_parent_ordinal != Some(relative_ref.root_child_ordinal) {
            if run_count > 0 {
                total_ref_bytes = total_ref_bytes.saturating_add(1);
            }
            total_ref_bytes = total_ref_bytes
                .saturating_add(u64_varint_bytes(relative_ref.root_child_ordinal))
                .saturating_add(u64_varint_bytes(relative_ref.descendant_ordinal));
            current_parent_ordinal = Some(relative_ref.root_child_ordinal);
            previous_descendant_ordinal = Some(relative_ref.descendant_ordinal);
            run_count = run_count.saturating_add(1);
            continue;
        }
        let previous_descendant_ordinal_value = previous_descendant_ordinal?;
        if relative_ref.descendant_ordinal <= previous_descendant_ordinal_value {
            return None;
        }
        let delta = relative_ref.descendant_ordinal - previous_descendant_ordinal_value;
        total_ref_bytes = total_ref_bytes.saturating_add(u64_varint_bytes(delta));
        previous_descendant_ordinal = Some(relative_ref.descendant_ordinal);
    }
    if run_count <= 1 {
        return None;
    }
    Some(total_ref_bytes)
}

fn uniform_nested_parent_ordinal(
    relative_refs_by_digest: &HashMap<B3Digest, NestedSummaryRelativeRef>,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
    root_child_prefix_len: usize,
) -> Option<u64> {
    let mut shared_parent_ordinal = None;
    for summary in &summaries[root_child_prefix_len..] {
        let relative_ref = relative_refs_by_digest.get(&summary.directory_digest)?;
        if let Some(parent_ordinal) = shared_parent_ordinal {
            if parent_ordinal != relative_ref.root_child_ordinal {
                return None;
            }
        } else {
            shared_parent_ordinal = Some(relative_ref.root_child_ordinal);
        }
    }
    shared_parent_ordinal
}

#[allow(tigerstyle::platform_dependent_cast)] // known-bounded fixture values
fn complete_root_child_prefix_len(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> Option<u64> {
    let ArtifactNode::Directory(directory) = &output.root else {
        return None;
    };
    let root_child_count = directory.children.len() as u64;
    if summaries.len() as u64 <= root_child_count {
        return None;
    }
    let root_child_ordinals = root_child_ordinal_map(output, root_child_count)?;
    for (index, summary) in summaries.iter().take(root_child_count as usize).enumerate() {
        let ordinal = root_child_ordinals.get(&summary.directory_digest).copied()?;
        if ordinal != index as u64 {
            return None;
        }
    }
    Some(root_child_count)
}

fn mixed_root_child_split_count_bytes(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> u64 {
    let Some(root_child_prefix_len) = complete_root_child_prefix_len(output, summaries) else {
        return 0;
    };
    if DEFAULT_GROUPED_OUTPUT_HEADER_MODE == GroupedOutputHeaderMode::PackedModeInSummaryCount
        && summaries.len() <= 3
        && root_child_prefix_len <= 3
    {
        return 0;
    }
    1
}

fn explicit_complete_coverage_bytes(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> u64 {
    let Some(coverage_count) = explicit_complete_coverage_count(output, summaries) else {
        return 0;
    };
    if can_pack_complete_coverage_in_grouped_header(output, summaries, coverage_count) {
        return 0;
    }
    1
}

fn explicit_complete_coverage_count(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> Option<u64> {
    if complete_root_child_ordinals(output, summaries).is_some() {
        return Some(summaries.len() as u64);
    }
    complete_root_child_prefix_len(output, summaries)
}

fn can_pack_complete_coverage_in_grouped_header(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
    coverage_count: u64,
) -> bool {
    if DEFAULT_GROUPED_OUTPUT_HEADER_MODE != GroupedOutputHeaderMode::PackedModeInSummaryCount {
        return false;
    }
    let summary_count = summaries.len() as u64;
    if coverage_count == summary_count && complete_root_child_ordinals(output, summaries).is_some() {
        return true;
    }
    if complete_root_child_prefix_len(output, summaries).is_some() {
        return true;
    }
    if summary_count > 3 {
        return false;
    }
    coverage_count <= 3
}

fn can_pack_uniform_optional_filter_tag_in_small_header(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> bool {
    if DEFAULT_GROUPED_OUTPUT_HEADER_MODE != GroupedOutputHeaderMode::PackedModeInSummaryCount {
        return false;
    }
    if summaries.len() > 3 {
        return false;
    }
    if mixed_root_child_split_count_bytes(output, summaries) != 0 {
        return false;
    }
    let Some(first_tag) = summaries.first().and_then(|summary| summary.packed_optional_filter_tag()) else {
        return false;
    };
    summaries.iter().all(|summary| summary.packed_optional_filter_tag() == Some(first_tag))
}

fn grouped_optional_filter_tag_bytes(summaries: &[&ReceiverProbabilisticFrontierSummary]) -> u64 {
    if let Some(shared_tag_bytes) = shared_uniform_optional_filter_tag_bytes(summaries) {
        return shared_tag_bytes;
    }
    packed_optional_filter_tag_bytes(summaries).unwrap_or(summaries.len() as u64)
}

fn can_pack_large_uniform_optional_filter_tag_in_grouped_header(
    summaries: &[&ReceiverProbabilisticFrontierSummary],
    split_count_bytes: u64,
    coverage_count_bytes: u64,
) -> bool {
    if DEFAULT_GROUPED_OUTPUT_HEADER_MODE != GroupedOutputHeaderMode::PackedModeInSummaryCount {
        return false;
    }
    if split_count_bytes != 0 {
        return false;
    }
    if coverage_count_bytes != 0 {
        return false;
    }
    if shared_uniform_optional_filter_tag_bytes(summaries) != Some(1) {
        return false;
    }
    (summaries.len() as u64) <= 31
}

fn can_pack_large_uniform_optional_filter_tag_and_split_in_grouped_header(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
    split_count_bytes: u64,
    coverage_count_bytes: u64,
) -> bool {
    if DEFAULT_GROUPED_OUTPUT_HEADER_MODE != GroupedOutputHeaderMode::PackedModeInSummaryCount {
        return false;
    }
    if split_count_bytes == 0 {
        return false;
    }
    if coverage_count_bytes != 0 {
        return false;
    }
    if shared_uniform_optional_filter_tag_bytes(summaries) != Some(1) {
        return false;
    }
    let Some(split_count) = complete_root_child_prefix_len(output, summaries) else {
        return false;
    };
    let summary_count = summaries.len() as u64;
    if summary_count > 7 {
        return false;
    }
    split_count <= 7
}

fn shared_uniform_optional_filter_tag_bytes(summaries: &[&ReceiverProbabilisticFrontierSummary]) -> Option<u64> {
    let packed_tag_bytes = packed_optional_filter_tag_bytes(summaries)?;
    if packed_tag_bytes <= 1 {
        return None;
    }
    let first_tag = summaries.first()?.packed_optional_filter_tag()?;
    if summaries.iter().all(|summary| summary.packed_optional_filter_tag() == Some(first_tag)) {
        return Some(1);
    }
    None
}

fn packed_optional_filter_tag_bytes(summaries: &[&ReceiverProbabilisticFrontierSummary]) -> Option<u64> {
    if summaries.iter().any(|summary| summary.packed_optional_filter_tag().is_none()) {
        return None;
    }
    let packed_tag_bits = (summaries.len() as u64).saturating_mul(2);
    Some(packed_tag_bits.div_ceil(8))
}

fn complete_root_child_ordinals(
    output: &OutputFixture,
    summaries: &[&ReceiverProbabilisticFrontierSummary],
) -> Option<Vec<u64>> {
    let root_child_ordinals = root_child_ordinal_map(output, summaries.len() as u64)?;
    let mut ordinals = Vec::with_capacity(summaries.len());
    let mut seen_ordinals = HashSet::new();
    for summary in summaries {
        let ordinal = root_child_ordinals.get(&summary.directory_digest).copied()?;
        if !seen_ordinals.insert(ordinal) {
            return None;
        }
        ordinals.push(ordinal);
    }
    ordinals.sort_unstable();
    for (expected_ordinal, actual_ordinal) in ordinals.iter().enumerate() {
        if *actual_ordinal != expected_ordinal as u64 {
            return None;
        }
    }
    Some(ordinals)
}

fn root_child_ordinal_map(output: &OutputFixture, expected_child_count: u64) -> Option<HashMap<B3Digest, u64>> {
    let ArtifactNode::Directory(directory) = &output.root else {
        return None;
    };
    if directory.children.len() as u64 != expected_child_count {
        return None;
    }
    let mut ordinal_by_digest = HashMap::new();
    for (index, child) in directory.children.iter().enumerate() {
        let digest = digest_of(child);
        if ordinal_by_digest.insert(digest, index as u64).is_some() {
            return None;
        }
    }
    Some(ordinal_by_digest)
}

fn u64_varint_bytes(mut value: u64) -> u64 {
    let mut byte_count = 1u64;
    while value >= 128 {
        value >>= 7;
        byte_count = byte_count.saturating_add(1);
    }
    byte_count
}

fn lossy_frontier_summary(summary: &ReceiverFrontierSummary) -> ReceiverLossyFrontierSummary {
    ReceiverLossyFrontierSummary {
        owner_output_id: summary.owner_output_id.clone(),
        directory_digest: summary.directory_digest,
        directory_buckets: summary.directory_digests.iter().map(digest_bucket).collect(),
        blob_buckets: summary.blob_digests.iter().map(digest_bucket).collect(),
        chunk_buckets: summary.chunk_digests.iter().map(digest_bucket).collect(),
    }
}

fn digest_bucket(digest: &B3Digest) -> u8 {
    digest.as_ref()[0] >> 4
}

fn probabilistic_frontier_summary(summary: &ReceiverFrontierSummary) -> ReceiverProbabilisticFrontierSummary {
    let filter_config = DEFAULT_PROBABILISTIC_FILTER_CONFIG;
    let chunk_filter_bits = optional_probabilistic_filter_bits(summary.chunk_digests.iter(), filter_config);
    let blob_filter_bits = if chunk_filter_bits.is_some() {
        None
    } else {
        optional_probabilistic_filter_bits(summary.blob_digests.iter(), filter_config)
    };
    ReceiverProbabilisticFrontierSummary {
        owner_output_id: summary.owner_output_id.clone(),
        directory_digest: summary.directory_digest,
        filter_config,
        directory_filter_bits: probabilistic_filter_bits(summary.directory_digests.iter(), filter_config),
        blob_filter_bits,
        chunk_filter_bits,
    }
}

fn probabilistic_filter_bits<'a>(
    digests: impl Iterator<Item = &'a B3Digest>,
    filter_config: ProbabilisticFilterConfig,
) -> u128 {
    let mut filter_bits = 0u128;
    let slot_count = u32::from(filter_config.slot_count);
    let tap_count = usize::from(filter_config.tap_count);
    assert!(slot_count > 0, "probabilistic filter slot count must be positive");
    assert!(slot_count <= 128, "probabilistic filter slot count exceeds u128 capacity");
    assert!(tap_count > 0, "probabilistic filter tap count must be positive");
    assert!(tap_count <= 4, "probabilistic filter tap count must stay bounded");
    for digest in digests {
        for byte in digest.as_ref().iter().take(tap_count) {
            let bit_index = u32::from(*byte) % slot_count;
            filter_bits |= 1u128 << bit_index;
        }
    }
    filter_bits
}

fn optional_probabilistic_filter_bits<'a>(
    digests: impl Iterator<Item = &'a B3Digest>,
    filter_config: ProbabilisticFilterConfig,
) -> Option<u128> {
    let digest_list = digests.collect::<Vec<_>>();
    if digest_list.is_empty() {
        return None;
    }
    Some(probabilistic_filter_bits(digest_list.into_iter(), filter_config))
}

#[allow(tigerstyle::no_panic)] // fixture-only invariant enforced by caller
fn frontier_summary(owner_output_id: &str, node: &ArtifactNode) -> ReceiverFrontierSummary {
    let ArtifactNode::Directory(directory) = node else {
        panic!("frontier summaries require directory nodes");
    };
    let mut directory_digests = HashSet::new();
    let mut blob_digests = HashSet::new();
    let mut chunk_digests = HashSet::new();
    collect_summary_digests(node, &mut directory_digests, &mut blob_digests, &mut chunk_digests);
    ReceiverFrontierSummary {
        owner_output_id: owner_output_id.to_owned(),
        directory_digest: directory.digest,
        directory_digests,
        blob_digests,
        chunk_digests,
    }
}

#[allow(tigerstyle::no_recursion)] // tree walk bounded by fixture tree depth
fn collect_summary_digests(
    node: &ArtifactNode,
    directory_digests: &mut HashSet<B3Digest>,
    blob_digests: &mut HashSet<B3Digest>,
    chunk_digests: &mut HashSet<B3Digest>,
) {
    match node {
        ArtifactNode::Directory(directory) => {
            directory_digests.insert(directory.digest);
            for child in &directory.children {
                collect_summary_digests(child, directory_digests, blob_digests, chunk_digests);
            }
        }
        ArtifactNode::Blob(blob) => {
            blob_digests.insert(blob.digest);
            for chunk in &blob.chunks {
                chunk_digests.insert(chunk.digest);
            }
        }
        ArtifactNode::Symlink { .. } => {}
    }
}

fn digest(label: &str) -> B3Digest {
    assert!(!label.is_empty(), "digest label must not be empty");
    blake3::hash(label.as_bytes()).as_bytes().into()
}

fn blob_node(label: &str, size_bytes: u64) -> ArtifactNode {
    let blob = BlobNode {
        digest: digest(&format!("blob:{label}")),
        size_bytes,
        chunks: Vec::new(),
    };
    blob.validate();
    ArtifactNode::Blob(blob)
}

fn chunked_blob(label: &str, chunk_sizes_bytes: &[u64]) -> ArtifactNode {
    assert!(!chunk_sizes_bytes.is_empty(), "chunk_sizes_bytes must not be empty");
    let chunks = chunk_sizes_bytes
        .iter()
        .enumerate()
        .map(|(index, size_bytes)| ChunkRef {
            digest: digest(&format!("chunk:{label}:{index}")),
            size_bytes: *size_bytes,
        })
        .collect::<Vec<_>>();
    chunked_blob_from_chunks(label, &chunks)
}

fn chunked_blob_from_chunks(label: &str, chunks: &[ChunkRef]) -> ArtifactNode {
    assert!(!chunks.is_empty(), "chunks must not be empty");
    let size_bytes = chunks.iter().fold(0u64, |total, chunk| total.saturating_add(chunk.size_bytes));
    let blob = BlobNode {
        digest: digest(&format!("blob:{label}")),
        size_bytes,
        chunks: chunks.to_vec(),
    };
    blob.validate();
    ArtifactNode::Blob(blob)
}

fn dir_node(label: &str, children: Vec<ArtifactNode>) -> ArtifactNode {
    assert!(!children.is_empty(), "directory children must not be empty");
    ArtifactNode::Directory(DirectoryNode {
        digest: digest(&format!("dir:{label}")),
        children,
    })
}

#[allow(tigerstyle::platform_dependent_cast)] // count is u32, always fits in usize
fn numbered_leaf_dirs(label_prefix: &str, count: u32, blob_size_bytes: u64) -> Vec<ArtifactNode> {
    assert!(count > 0, "count must be positive");
    assert!(blob_size_bytes > 0, "blob size must be positive");
    let mut nodes = Vec::with_capacity(count as usize);
    for index in 0..count {
        nodes.push(dir_node(&format!("{label_prefix}-{index}"), vec![blob_node(
            &format!("{label_prefix}-blob-{index}"),
            blob_size_bytes,
        )]));
    }
    assert_eq!(nodes.len(), count as usize, "generated node count mismatch");
    nodes
}

fn digest_of(node: &ArtifactNode) -> B3Digest {
    match node {
        ArtifactNode::Directory(directory) => directory.digest,
        ArtifactNode::Blob(blob) => blob.digest,
        ArtifactNode::Symlink { target } => digest(&format!("symlink:{target}")),
    }
}

fn blob_chunks(node: &ArtifactNode) -> Vec<ChunkRef> {
    match node {
        ArtifactNode::Blob(blob) => blob.chunks.clone(),
        ArtifactNode::Directory(_) | ArtifactNode::Symlink { .. } => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_suite_is_fixed() {
        let suite = bench_suite();
        assert_eq!(suite.cases.len(), 5);
        assert_eq!(suite.full_transfer_bytes_total(), 2_678_784);
        assert_eq!(suite.coarse_transfer_bytes_total(), 450_560);
    }

    #[test]
    fn chunk_hit_case_keeps_one_chunk_missing() {
        let suite = bench_suite();
        let case = suite.cases.iter().find(|case| case.name == "chunk-hit").expect("chunk-hit case");
        assert_eq!(case.expected_full_bytes, 2_097_152);
        assert_eq!(case.expected_coarse_bytes, 262_144);
        assert_eq!(case.receiver.known_chunks.len(), 7);
        assert_eq!(case.receiver_store.outputs.len(), 1);
    }

    #[test]
    fn frontier_metadata_is_present_for_nested_cases() {
        let suite = bench_suite();
        for case in &suite.cases {
            if case.name == "whole-output-hit" {
                assert!(case.receiver_frontiers.is_empty(), "case {}", case.name);
                assert!(case.receiver_lossy_frontiers.is_empty(), "case {}", case.name);
                assert!(case.receiver_probabilistic_frontiers.is_empty(), "case {}", case.name);
                assert!(case.frontier_complete_outputs.is_empty(), "case {}", case.name);
            } else {
                assert!(!case.receiver_frontiers.is_empty(), "case {}", case.name);
                assert_eq!(case.receiver_frontiers.len(), case.receiver_lossy_frontiers.len(), "case {}", case.name);
                assert_eq!(
                    case.receiver_frontiers.len(),
                    case.receiver_probabilistic_frontiers.len(),
                    "case {}",
                    case.name
                );
                assert!(!case.frontier_complete_outputs.is_empty(), "case {}", case.name);
            }
        }
    }

    #[test]
    fn probabilistic_filter_payload_bytes_match_config() {
        let suite = bench_suite();
        let subtree_case = suite.cases.iter().find(|case| case.name == "subtree-hit").expect("subtree-hit case");
        let blob_case = suite.cases.iter().find(|case| case.name == "blob-hit").expect("blob-hit case");
        let chunk_case = suite.cases.iter().find(|case| case.name == "chunk-hit").expect("chunk-hit case");
        assert_eq!(suite.probabilistic_filter_bytes_total(), 562);
        assert_eq!(subtree_case.probabilistic_filter_bytes(), 532);
        assert_eq!(blob_case.probabilistic_filter_bytes(), 10);
        assert_eq!(chunk_case.probabilistic_filter_bytes(), 8);
    }

    #[test]
    fn probabilistic_wire_bytes_match_layout() {
        let suite = bench_suite();
        let subtree_case = suite.cases.iter().find(|case| case.name == "subtree-hit").expect("subtree-hit case");
        let blob_case = suite.cases.iter().find(|case| case.name == "blob-hit").expect("blob-hit case");
        let chunk_case = suite.cases.iter().find(|case| case.name == "chunk-hit").expect("chunk-hit case");
        assert_eq!(suite.probabilistic_summary_wire_bytes_total(), 833);
        assert_eq!(subtree_case.probabilistic_summary_wire_bytes(), 796);
        assert_eq!(blob_case.probabilistic_summary_wire_bytes(), 11);
        assert_eq!(chunk_case.probabilistic_summary_wire_bytes(), 10);
    }
}
