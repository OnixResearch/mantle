//! Allocation-free-in-the-common-case traversal shared by sync and async validators.

use std::ops::Range;

use smallvec::SmallVec;

use crate::{
    blake3, rec::truncate_ranges, split, BaoTree, ChunkNum, ChunkRangesRef, HashMode, TreeNode,
};

pub(super) enum Step<'a> {
    Parent {
        shifted: TreeNode,
        node: TreeNode,
        expected: blake3::Hash,
        is_root: bool,
        ranges: &'a ChunkRangesRef,
    },
    Data {
        bytes: Range<u64>,
        expected: blake3::Hash,
        is_root: bool,
    },
}

pub(super) struct Walker<'a> {
    tree: BaoTree,
    shifted_filled_size: TreeNode,
    mode: HashMode,
    // Right children are pushed before left children, so the first valid range
    // is returned before any IO is attempted for subsequent ranges.
    stack: SmallVec<[Step<'a>; 8]>,
}

impl<'a> Walker<'a> {
    pub(super) fn new(
        tree: BaoTree,
        root: blake3::Hash,
        ranges: &'a ChunkRangesRef,
        mode: HashMode,
    ) -> Self {
        let mut stack = SmallVec::new();
        let (shifted_root, shifted_filled_size) = tree.shifted();
        let ranges = truncate_ranges(ranges, tree.size());
        if !ranges.is_empty() {
            if tree.blocks() == 1 {
                stack.push(Step::Data {
                    bytes: 0..tree.size(),
                    expected: root,
                    is_root: true,
                });
            } else {
                stack.push(Step::Parent {
                    shifted: shifted_root,
                    node: shifted_root.subtract_block_size(tree.block_size.0),
                    expected: root,
                    is_root: true,
                    ranges,
                });
            }
        }
        Self {
            tree,
            shifted_filled_size,
            mode,
            stack,
        }
    }

    pub(super) fn next(&mut self) -> Option<Step<'a>> {
        match self.stack.pop()? {
            Step::Parent {
                node,
                expected,
                is_root,
                ..
            } if !self.tree.is_relevant_for_outboard(node) => {
                let (start, _, end) = self.tree.leaf_byte_ranges3(node);
                Some(Step::Data {
                    bytes: start..end,
                    expected,
                    is_root,
                })
            }
            other => Some(other),
        }
    }

    pub(super) fn descend(
        &mut self,
        shifted: TreeNode,
        expected: blake3::Hash,
        is_root: bool,
        ranges: &'a ChunkRangesRef,
        pair: Option<(blake3::Hash, blake3::Hash)>,
    ) {
        let Some((left_hash, right_hash)) = pair else {
            return;
        };
        if self.mode.parent_cv(&left_hash, &right_hash, is_root) != expected {
            return;
        }
        let node = shifted.subtract_block_size(self.tree.block_size.0);
        let (left_ranges, right_ranges) = split(ranges, node);
        if shifted.is_leaf() {
            let (start, middle, end) = self.tree.leaf_byte_ranges3(node);
            if !right_ranges.is_empty() {
                self.stack.push(Step::Data {
                    bytes: middle..end,
                    expected: right_hash,
                    is_root: false,
                });
            }
            if !left_ranges.is_empty() {
                self.stack.push(Step::Data {
                    bytes: start..middle,
                    expected: left_hash,
                    is_root: false,
                });
            }
        } else {
            if !right_ranges.is_empty() {
                let right = shifted.right_descendant(self.shifted_filled_size).unwrap();
                self.stack.push(Step::Parent {
                    shifted: right,
                    node: right.subtract_block_size(self.tree.block_size.0),
                    expected: right_hash,
                    is_root: false,
                    ranges: right_ranges,
                });
            }
            if !left_ranges.is_empty() {
                let left = shifted.left_child().unwrap();
                self.stack.push(Step::Parent {
                    shifted: left,
                    node: left.subtract_block_size(self.tree.block_size.0),
                    expected: left_hash,
                    is_root: false,
                    ranges: left_ranges,
                });
            }
        }
    }

    pub(super) fn valid_data(
        &self,
        bytes: &Range<u64>,
        data: &[u8],
        expected: &blake3::Hash,
        is_root: bool,
    ) -> bool {
        self.mode
            .hash_subtree(ChunkNum::full_chunks(bytes.start).0, data, is_root)
            == *expected
    }

    pub(super) fn finish(&mut self) {
        self.stack.clear();
    }
}

pub(super) fn chunk_range(bytes: Range<u64>) -> Range<ChunkNum> {
    ChunkNum::full_chunks(bytes.start)..ChunkNum::chunks(bytes.end)
}
