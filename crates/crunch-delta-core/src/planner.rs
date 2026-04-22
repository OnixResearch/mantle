use alloc::collections::BTreeSet;
use alloc::string::String;
use core::fmt;

use crate::model::ArtifactNode;
use crate::model::BlobNode;
use crate::model::ClosureFixture;
use crate::model::DeltaDigest;
use crate::model::DirectoryNode;
use crate::model::ReceiverManifest;
use crate::model::TransferPlan;
use crate::model::TransferTally;
use crate::model::closure_full_transfer_bytes;
use crate::model::validate_blob;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    StorePrefixMismatch {
        sender_prefix: String,
        receiver_prefix: String,
    },
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StorePrefixMismatch {
                sender_prefix,
                receiver_prefix,
            } => write!(f, "store prefix mismatch: sender={sender_prefix} receiver={receiver_prefix}"),
        }
    }
}

pub fn plan_transfer(sender: ClosureFixture, receiver: ReceiverManifest) -> Result<TransferPlan, PlanError> {
    if sender.store_prefix != receiver.store_prefix {
        return Err(PlanError::StorePrefixMismatch {
            sender_prefix: sender.store_prefix,
            receiver_prefix: receiver.store_prefix,
        });
    }

    assert!(!sender.outputs.is_empty(), "sender fixture must have at least one output");
    debug_assert!(!sender.store_prefix.is_empty());
    let full_transfer_bytes = closure_full_transfer_bytes(sender.clone());
    let mut available_blobs = BTreeSet::<DeltaDigest>::new();
    let mut transferred_bytes = 0u64;
    let mut tally = TransferTally::default();

    for output in &sender.outputs {
        if receiver.known_outputs.contains(&output.output_id) {
            tally.reused_outputs = tally.reused_outputs.saturating_add(1);
            continue;
        }
        transferred_bytes =
            transferred_bytes.saturating_add(plan_node(&output.root, &receiver, &mut available_blobs, &mut tally));
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
    available_blobs: &mut BTreeSet<DeltaDigest>,
    tally: &mut TransferTally,
) -> u64 {
    match node {
        ArtifactNode::Directory(directory) => plan_directory(directory, receiver, available_blobs, tally),
        ArtifactNode::Blob(blob) => plan_blob(blob, receiver, available_blobs, tally),
        ArtifactNode::Symlink { .. } => 0,
    }
}

fn plan_directory(
    directory: &DirectoryNode,
    receiver: &ReceiverManifest,
    available_blobs: &mut BTreeSet<DeltaDigest>,
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
    available_blobs: &mut BTreeSet<DeltaDigest>,
    tally: &mut TransferTally,
) -> u64 {
    validate_blob(blob.clone());
    debug_assert!(blob.size_bytes > 0 || blob.chunks.is_empty());
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
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::model::ChunkRef;
    use crate::model::OutputFixture;

    const DELTA_DIGEST_BYTES_LEN: usize = 32;
    const SIMPLE_BLOB_SIZE_BYTES: u64 = 8;

    #[test]
    fn prefix_mismatch_is_rejected() {
        let err = plan_transfer(sender_fixture("/crunch/store"), ReceiverManifest::new("/nix/store".to_string()))
            .expect_err("prefix mismatch must fail");
        assert_eq!(err, PlanError::StorePrefixMismatch {
            sender_prefix: "/crunch/store".to_string(),
            receiver_prefix: "/nix/store".to_string(),
        });
    }

    #[test]
    fn known_output_reuses_whole_output() {
        let sender = sender_fixture("/crunch/store");
        let mut receiver = ReceiverManifest::new("/crunch/store".to_string());
        receiver.known_outputs.insert("out".to_string());
        let plan = plan_transfer(sender, receiver).expect("known output should reuse whole output");
        assert_eq!(plan.transferred_bytes, 0);
        assert_eq!(plan.tally.reused_outputs, 1);
    }

    #[test]
    fn planner_matches_fixed_suite_targets() {
        let cases = vec![
            (
                "full-blob-send",
                ClosureFixture {
                    store_prefix: "/crunch/store".to_string(),
                    outputs: vec![OutputFixture {
                        output_id: "blob".to_string(),
                        root: ArtifactNode::Blob(BlobNode {
                            digest: DeltaDigest([3; DELTA_DIGEST_BYTES_LEN]),
                            size_bytes: SIMPLE_BLOB_SIZE_BYTES,
                            chunks: vec![],
                        }),
                    }],
                },
                ReceiverManifest::new("/crunch/store".to_string()),
                SIMPLE_BLOB_SIZE_BYTES,
            ),
            (
                "whole-output-hit",
                sender_fixture("/crunch/store"),
                {
                    let mut receiver = ReceiverManifest::new("/crunch/store".to_string());
                    receiver.known_outputs.insert("out".to_string());
                    receiver
                },
                0u64,
            ),
            (
                "single-missing-chunk",
                ClosureFixture {
                    store_prefix: "/crunch/store".to_string(),
                    outputs: vec![OutputFixture {
                        output_id: "chunked".to_string(),
                        root: ArtifactNode::Blob(BlobNode {
                            digest: DeltaDigest([4; DELTA_DIGEST_BYTES_LEN]),
                            size_bytes: SIMPLE_BLOB_SIZE_BYTES,
                            chunks: vec![
                                ChunkRef {
                                    digest: DeltaDigest([5; DELTA_DIGEST_BYTES_LEN]),
                                    size_bytes: 4,
                                },
                                ChunkRef {
                                    digest: DeltaDigest([6; DELTA_DIGEST_BYTES_LEN]),
                                    size_bytes: 4,
                                },
                            ],
                        }),
                    }],
                },
                {
                    let mut receiver = ReceiverManifest::new("/crunch/store".to_string());
                    receiver.known_chunks.insert(DeltaDigest([5; DELTA_DIGEST_BYTES_LEN]));
                    receiver
                },
                4u64,
            ),
        ];

        let mut total_transferred_bytes = 0u64;
        for (case_name, sender, receiver, expected_transferred_bytes) in cases {
            let plan = plan_transfer(sender, receiver).expect("fixed-suite plan should succeed");
            assert_eq!(plan.transferred_bytes, expected_transferred_bytes, "case {case_name}");
            total_transferred_bytes = total_transferred_bytes.saturating_add(plan.transferred_bytes);
        }
        assert_eq!(total_transferred_bytes, SIMPLE_BLOB_SIZE_BYTES.saturating_add(4));
    }

    fn sender_fixture(store_prefix: &str) -> ClosureFixture {
        ClosureFixture {
            store_prefix: store_prefix.to_string(),
            outputs: vec![OutputFixture {
                output_id: "out".to_string(),
                root: ArtifactNode::Blob(BlobNode {
                    digest: DeltaDigest([1; DELTA_DIGEST_BYTES_LEN]),
                    size_bytes: SIMPLE_BLOB_SIZE_BYTES,
                    chunks: vec![],
                }),
            }],
        }
    }
}
