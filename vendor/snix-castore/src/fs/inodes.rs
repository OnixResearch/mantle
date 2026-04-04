//! This module contains all the data structures used to track information
//! about inodes, which present snix-castore nodes in a filesystem.
use crate::{B3Digest, Node, path::PathComponent};

#[derive(Clone, Debug)]
pub enum InodeData {
    Regular(B3Digest, u64, bool),  // digest, size, executable
    Symlink(bytes::Bytes),         // target
    Directory(DirectoryInodeData), // either [DirectoryInodeData:Sparse] or [DirectoryInodeData:Populated]
}

/// This encodes the two different states of [InodeData::Directory].
/// Either the data still is sparse (we only saw a [crate::Node::Directory],
/// but didn't fetch the [crate::Directory] struct yet, or we processed a
/// lookup and did fetch the data.
#[derive(Clone, Debug)]
pub enum DirectoryInodeData {
    Sparse(B3Digest, u64),                                // digest, size
    Populated(B3Digest, Vec<(u64, PathComponent, Node)>), // [(child_inode, name, node)]
}

impl InodeData {
    /// Constructs a new InodeData from a `&Node`.
    pub fn from_node(node: &Node) -> Self {
        match node {
            Node::Directory { digest, size } => {
                Self::Directory(DirectoryInodeData::Sparse(*digest, *size))
            }
            Node::File {
                digest,
                size,
                executable,
            } => Self::Regular(*digest, *size, *executable),
            Node::Symlink { target } => Self::Symlink(target.clone().into()),
        }
    }
}
