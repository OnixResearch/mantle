use std::collections::HashSet;

use snix_castore::B3Digest;

use crate::model::ArtifactNode;
use crate::model::BlobNode;
use crate::model::ClosureFixture;
use crate::model::ReceiverManifest;
use crate::model::TransferPlan;
use crate::model::TransferTally;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    StorePrefixMismatch {
        sender_prefix: String,
        receiver_prefix: String,
    },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorePrefixMismatch {
                sender_prefix,
                receiver_prefix,
            } => write!(f, "store prefix mismatch: sender={sender_prefix} receiver={receiver_prefix}"),
        }
    }
}

impl std::error::Error for PlanError {}

pub fn plan_transfer(sender: &ClosureFixture, receiver: &ReceiverManifest) -> Result<TransferPlan, PlanError> {
    if sender.store_prefix != receiver.store_prefix {
        return Err(PlanError::StorePrefixMismatch {
            sender_prefix: sender.store_prefix.clone(),
            receiver_prefix: receiver.store_prefix.clone(),
        });
    }

    assert!(!sender.outputs.is_empty(), "sender fixture must have at least one output");
    let full_transfer_bytes = sender.full_transfer_bytes();
    let mut available_blobs = HashSet::<B3Digest>::new();
    let mut transferred_bytes = 0u64;
    let mut tally = TransferTally::default();

    for output in &sender.outputs {
        if receiver.known_outputs.contains(&output.output_id) {
            tally.reused_outputs = tally.reused_outputs.saturating_add(1);
            continue;
        }
        transferred_bytes =
            transferred_bytes.saturating_add(plan_node(&output.root, receiver, &mut available_blobs, &mut tally));
    }

    Ok(TransferPlan {
        transferred_bytes,
        full_transfer_bytes,
        tally,
    })
}

fn plan_node(
    node: &ArtifactNode,
    receiver: &ReceiverManifest,
    available_blobs: &mut HashSet<B3Digest>,
    tally: &mut TransferTally,
) -> u64 {
    match node {
        ArtifactNode::Directory(directory) => plan_directory(directory, receiver, available_blobs, tally),
        ArtifactNode::Blob(blob) => plan_blob(blob, receiver, available_blobs, tally),
        ArtifactNode::Symlink { .. } => 0,
    }
}

fn plan_directory(
    directory: &crate::DirectoryNode,
    receiver: &ReceiverManifest,
    available_blobs: &mut HashSet<B3Digest>,
    tally: &mut TransferTally,
) -> u64 {
    if receiver.known_directories.contains(&directory.digest) {
        tally.reused_directories = tally.reused_directories.saturating_add(1);
        return 0;
    }

    directory
        .children
        .iter()
        .fold(0u64, |total, child| total.saturating_add(plan_node(child, receiver, available_blobs, tally)))
}

fn plan_blob(
    blob: &BlobNode,
    receiver: &ReceiverManifest,
    available_blobs: &mut HashSet<B3Digest>,
    tally: &mut TransferTally,
) -> u64 {
    blob.validate();
    if receiver.known_blobs.contains(&blob.digest) || available_blobs.contains(&blob.digest) {
        tally.reused_blobs = tally.reused_blobs.saturating_add(1);
        return 0;
    }

    if blob.chunks.is_empty() {
        available_blobs.insert(blob.digest);
        tally.sent_blobs = tally.sent_blobs.saturating_add(1);
        return blob.size_bytes;
    }

    let (missing_bytes, known_chunk_count, missing_chunk_count) = analyze_blob_chunks(blob, receiver);
    if known_chunk_count == 0 || missing_bytes >= blob.size_bytes {
        available_blobs.insert(blob.digest);
        tally.sent_blobs = tally.sent_blobs.saturating_add(1);
        return blob.size_bytes;
    }

    available_blobs.insert(blob.digest);
    tally.reused_chunks = tally.reused_chunks.saturating_add(known_chunk_count);
    tally.sent_chunks = tally.sent_chunks.saturating_add(missing_chunk_count);
    missing_bytes
}

fn analyze_blob_chunks(blob: &BlobNode, receiver: &ReceiverManifest) -> (u64, u32, u32) {
    let mut missing_bytes = 0u64;
    let mut known_chunk_count = 0u32;
    let mut missing_chunk_count = 0u32;

    for chunk in &blob.chunks {
        if receiver.known_chunks.contains(&chunk.digest) {
            known_chunk_count = known_chunk_count.saturating_add(1);
            continue;
        }
        missing_bytes = missing_bytes.saturating_add(chunk.size_bytes);
        missing_chunk_count = missing_chunk_count.saturating_add(1);
    }

    assert_eq!(known_chunk_count.saturating_add(missing_chunk_count), blob.chunks.len() as u32);
    (missing_bytes, known_chunk_count, missing_chunk_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_suite;

    #[test]
    fn prefix_mismatch_is_rejected() {
        let suite = bench_suite();
        let case = &suite.cases[0];
        let mut receiver = case.receiver.clone();
        receiver.store_prefix = "/nix/store".to_owned();
        let err = plan_transfer(&case.sender, &receiver).expect_err("prefix mismatch must fail");
        assert_eq!(err, PlanError::StorePrefixMismatch {
            sender_prefix: "/crunch/store".to_owned(),
            receiver_prefix: "/nix/store".to_owned(),
        });
    }

    #[test]
    fn planner_matches_fixed_suite_targets() {
        let suite = bench_suite();
        let mut total = 0u64;
        for case in &suite.cases {
            let plan = plan_transfer(&case.sender, &case.receiver).expect("coarse plan");
            assert_eq!(plan.transferred_bytes, case.expected_coarse_bytes, "case {}", case.name);
            total = total.saturating_add(plan.transferred_bytes);
        }
        assert_eq!(total, suite.coarse_transfer_bytes_total());
    }

    #[test]
    fn whole_output_case_reuses_entire_output() {
        let suite = bench_suite();
        let case = suite.cases.iter().find(|case| case.name == "whole-output-hit").unwrap();
        let plan = plan_transfer(&case.sender, &case.receiver).unwrap();
        assert_eq!(plan.transferred_bytes, 0);
        assert_eq!(plan.tally.reused_outputs, 1);
    }

    #[test]
    fn chunk_case_sends_only_missing_chunk() {
        let suite = bench_suite();
        let case = suite.cases.iter().find(|case| case.name == "chunk-hit").unwrap();
        let plan = plan_transfer(&case.sender, &case.receiver).unwrap();
        assert_eq!(plan.transferred_bytes, 262_144);
        assert_eq!(plan.tally.reused_chunks, 7);
        assert_eq!(plan.tally.sent_chunks, 1);
    }

    #[test]
    fn cross_output_case_sends_shared_blob_once() {
        let suite = bench_suite();
        let case = suite.cases.iter().find(|case| case.name == "cross-output").unwrap();
        let plan = plan_transfer(&case.sender, &case.receiver).unwrap();
        assert_eq!(plan.transferred_bytes, 139_264);
        assert_eq!(plan.tally.sent_blobs, 3);
        assert_eq!(plan.tally.reused_blobs, 1);
    }
}
