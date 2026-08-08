use std::collections::btree_map::BTreeMap;
use std::collections::btree_map::{self};

use crate::B3Digest;
use crate::Node;
use crate::errors::DirectoryError;
use crate::path::PathComponent;
use crate::proto;

/// A Directory contains nodes, which can be Directory, File or Symlink nodes.
/// It attaches names to these nodes, which is the basename in that directory.
/// These names:
///  - MUST not contain slashes or null bytes
///  - MUST not be '.' or '..'
///  - MUST be unique across all three lists
#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Directory {
    nodes: BTreeMap<PathComponent, Node>,
}

impl Directory {
    /// Constructs a new, empty Directory.
    pub fn new() -> Self {
        Directory { nodes: BTreeMap::new() }
    }

    /// Construct a [Directory] from tuples of name and [Node].
    ///
    /// Inserting multiple elements with the same name will yield an error, as
    /// well as exceeding the maximum size.
    pub fn try_from_iter<T: IntoIterator<Item = (PathComponent, Node)>>(iter: T) -> Result<Directory, DirectoryError> {
        let mut nodes = BTreeMap::new();

        iter.into_iter()
            .try_fold(0u64, |size, (name, node)| check_insert_node(size, &mut nodes, name, node))?;

        Ok(Self { nodes })
    }

    /// The size of a directory is the number of all regular and symlink elements,
    /// the number of directory elements, and their size fields.
    // r[impl vendored_snix.castore_metadata]
    pub fn size(&self) -> u64 {
        // Construction rejects every insertion that would overflow this sum.
        self.nodes
            .values()
            .try_fold(0u64, |total, node| {
                let child_size = match node {
                    Node::Directory { size, .. } => size.checked_add(1)?,
                    Node::File { .. } | Node::Symlink { .. } => 1,
                };
                total.checked_add(child_size)
            })
            .expect("validated Directory size must fit in u64")
    }

    /// Calculates the digest of a Directory, which is the blake3 hash of a
    /// Directory protobuf message, serialized in protobuf canonical form.
    pub fn digest(&self) -> B3Digest {
        proto::Directory::from(self.clone()).digest()
    }

    /// Allows iterating over all nodes (directories, files and symlinks)
    /// For each, it returns a tuple of its name and node.
    /// The elements are sorted by their names.
    pub fn nodes(&self) -> impl Iterator<Item = (&PathComponent, &Node)> + '_ {
        self.nodes.iter()
    }

    /// Dissolves a Directory into its individual names and nodes.
    /// The elements are sorted by their names.
    pub fn into_nodes(self) -> impl Iterator<Item = (PathComponent, Node)> {
        self.nodes.into_iter()
    }

    /// Adds the specified [Node] to the [Directory] with a given name.
    ///
    /// Inserting a node that already exists with the same name in the directory
    /// will yield an error, as well as exceeding the maximum size.
    ///
    /// In case you want to construct a [Directory] from multiple elements, use
    /// [Directory::try_from_iter] instead.
    pub fn add(&mut self, name: PathComponent, node: Node) -> Result<(), DirectoryError> {
        check_insert_node(self.size(), &mut self.nodes, name, node)?;
        Ok(())
    }
}

fn checked_sum(iter: impl IntoIterator<Item = u64>) -> Option<u64> {
    iter.into_iter().try_fold(0u64, |acc, i| acc.checked_add(i))
}

/// Helper function dealing with inserting nodes into the nodes [BTreeMap],
/// after ensuring the new size doesn't overlow and the key doesn't exist already.
///
/// Returns the new total size, or an error.
fn check_insert_node(
    current_size: u64,
    nodes: &mut BTreeMap<PathComponent, Node>,
    name: PathComponent,
    node: Node,
) -> Result<u64, DirectoryError> {
    // Check that the even after adding this new directory entry, the size calculation will not
    // overflow
    let new_size = checked_sum([current_size, 1, match node {
        Node::Directory { size, .. } => size,
        _ => 0,
    }])
    .ok_or(DirectoryError::SizeOverflow)?;

    match nodes.entry(name) {
        btree_map::Entry::Vacant(e) => {
            e.insert(node);
        }
        btree_map::Entry::Occupied(occupied) => {
            return Err(DirectoryError::DuplicateName(occupied.key().to_owned()));
        }
    }

    Ok(new_size)
}

#[cfg(test)]
mod test {
    use super::Directory;
    use super::Node;
    use crate::DirectoryError;
    use crate::PathComponent;
    use crate::fixtures::DUMMY_DIGEST;

    #[test]
    fn size_counts_each_mixed_entry_once() {
        // r[verify vendored_snix.castore_metadata]
        const CHILD_DIRECTORY_SIZE: u64 = 4;
        const EXPECTED_MIXED_SIZE: u64 = 7;

        let directory = Directory::try_from_iter([
            ("dir".try_into().unwrap(), Node::Directory {
                digest: *DUMMY_DIGEST,
                size: CHILD_DIRECTORY_SIZE,
            }),
            ("file".try_into().unwrap(), Node::File {
                digest: *DUMMY_DIGEST,
                size: 1,
                executable: false,
            }),
            ("link".try_into().unwrap(), Node::Symlink {
                target: "target".try_into().unwrap(),
            }),
        ])
        .unwrap();

        assert_eq!(Directory::new().size(), 0);
        assert_eq!(directory.size(), EXPECTED_MIXED_SIZE);
    }

    #[test]
    fn size_does_not_duplicate_directory_entry_contribution() {
        // r[verify vendored_snix.castore_metadata]
        const CHILD_DIRECTORY_SIZE: u64 = 4;
        const EXPECTED_PARENT_SIZE: u64 = 5;

        let directory = Directory::try_from_iter([("dir".try_into().unwrap(), Node::Directory {
            digest: *DUMMY_DIGEST,
            size: CHILD_DIRECTORY_SIZE,
        })])
        .unwrap();

        assert_eq!(directory.size(), EXPECTED_PARENT_SIZE);
        assert_ne!(directory.size(), EXPECTED_PARENT_SIZE + 1);
    }

    #[test]
    fn from_iter_single() {
        Directory::try_from_iter([(PathComponent::try_from("b").unwrap(), Node::Directory {
            digest: *DUMMY_DIGEST,
            size: 1,
        })])
        .unwrap();
    }

    #[test]
    fn from_iter_multiple() {
        let d = Directory::try_from_iter([
            ("b".try_into().unwrap(), Node::Directory {
                digest: *DUMMY_DIGEST,
                size: 1,
            }),
            ("a".try_into().unwrap(), Node::Directory {
                digest: *DUMMY_DIGEST,
                size: 1,
            }),
            ("z".try_into().unwrap(), Node::Directory {
                digest: *DUMMY_DIGEST,
                size: 1,
            }),
            ("f".try_into().unwrap(), Node::File {
                digest: *DUMMY_DIGEST,
                size: 1,
                executable: true,
            }),
            ("c".try_into().unwrap(), Node::File {
                digest: *DUMMY_DIGEST,
                size: 1,
                executable: true,
            }),
            ("g".try_into().unwrap(), Node::File {
                digest: *DUMMY_DIGEST,
                size: 1,
                executable: true,
            }),
            ("t".try_into().unwrap(), Node::Symlink {
                target: "a".try_into().unwrap(),
            }),
            ("o".try_into().unwrap(), Node::Symlink {
                target: "a".try_into().unwrap(),
            }),
            ("e".try_into().unwrap(), Node::Symlink {
                target: "a".try_into().unwrap(),
            }),
        ])
        .unwrap();

        // Convert to proto struct and back to ensure we are not generating any invalid structures
        crate::Directory::try_from(crate::proto::Directory::from(d)).expect("directory should be valid");
    }

    #[test]
    fn add_nodes_to_directory() {
        let mut d = Directory::new();

        d.add("b".try_into().unwrap(), Node::Directory {
            digest: *DUMMY_DIGEST,
            size: 1,
        })
        .unwrap();
        d.add("a".try_into().unwrap(), Node::Directory {
            digest: *DUMMY_DIGEST,
            size: 1,
        })
        .unwrap();

        // Convert to proto struct and back to ensure we are not generating any invalid structures
        crate::Directory::try_from(crate::proto::Directory::from(d)).expect("directory should be valid");
    }

    #[test]
    fn validate_overflow() {
        let mut d = Directory::new();

        assert_eq!(
            d.add("foo".try_into().unwrap(), Node::Directory {
                digest: *DUMMY_DIGEST,
                size: u64::MAX
            }),
            Err(DirectoryError::SizeOverflow)
        );
    }

    #[test]
    fn add_duplicate_node_to_directory() {
        let mut d = Directory::new();

        d.add("a".try_into().unwrap(), Node::Directory {
            digest: *DUMMY_DIGEST,
            size: 1,
        })
        .unwrap();
        assert_eq!(
            format!(
                "{}",
                d.add("a".try_into().unwrap(), Node::File {
                    digest: *DUMMY_DIGEST,
                    size: 1,
                    executable: true
                })
                .expect_err("adding duplicate dir entry must fail")
            ),
            "\"a\" is a duplicate name"
        );
    }
}
