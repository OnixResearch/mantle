//! Recursive castore completeness checking with immutable completeness markers.
//!
//! r[impl cache_substitution.castore_completeness]

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::LazyLock;

use snix_castore::B3Digest;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;

use crate::Error;

/// Maximum recursive node visits before aborting (safety bound).
const MAX_RECURSIVE_NODES: usize = 100_000;
/// Maximum directory tree depth before aborting.
const MAX_DEPTH: usize = 128;

/// Thread-safe in-memory store of completeness markers keyed by
/// finalized directory node digest (BLAKE3).
pub struct CompletenessMarkerStore {
    markers: Mutex<HashSet<B3Digest>>,
}

impl CompletenessMarkerStore {
    pub fn new() -> Self {
        Self {
            markers: Mutex::new(HashSet::new()),
        }
    }

    pub fn contains(&self, digest: &B3Digest) -> bool {
        self.markers.lock().unwrap().contains(digest)
    }

    pub fn insert(&self, digest: B3Digest) {
        self.markers.lock().unwrap().insert(digest);
    }

    pub fn clear(&self) {
        self.markers.lock().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.markers.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.markers.lock().unwrap().is_empty()
    }
}

/// Global completeness marker store shared across all StoreHandles.
pub static GLOBAL_COMPLETENESS_MARKERS: LazyLock<CompletenessMarkerStore> = LazyLock::new(|| {
    CompletenessMarkerStore::new()
});

/// Recursively check that a node's full castore tree is present.
///
/// Checks every child blob and directory under `node` root. Symlinks
/// are always complete (target inline).  Uses the global completeness
/// marker store to skip already-verified directory nodes.
pub async fn recursive_castore_completeness(
    blob_service: &dyn BlobService,
    directory_service: &dyn DirectoryService,
    node: &Node,
) -> Result<bool, Error> {
    // Iterative stack-based traversal to avoid recursive async fn.
    let mut stack: Vec<(Node, usize)> = Vec::with_capacity(128);
    stack.push((node.clone(), 0));
    let mut visited = 0usize;

    while let Some((current_node, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            return Ok(false);
        }
        if visited > MAX_RECURSIVE_NODES {
            return Ok(false);
        }
        visited += 1;

        match &current_node {
            Node::File { digest, .. } => {
                if !blob_service.has(digest).await.map_err(|e| {
                    Error::BlobService(format!("completeness check: {e}"))
                })? {
                    return Ok(false);
                }
            }
            Node::Symlink { .. } => {
                // Symlinks are always complete.
            }
            Node::Directory { digest, .. } => {
                // Completeness marker: skip if already verified.
                if GLOBAL_COMPLETENESS_MARKERS.contains(digest) {
                    continue;
                }

                // Fetch the directory node's children.
                let dir = match directory_service
                    .get(digest)
                    .await
                    .map_err(|e| Error::DirectoryService(format!("completeness check: {e}")))?
                {
                    Some(dir) => dir,
                    None => return Ok(false),
                };

                // Push children onto the stack (cloned, owned Nodes).
                for child in dir.nodes() {
                    stack.push((child.1.clone(), depth + 1));
                }

                // Mark as complete.
                GLOBAL_COMPLETENESS_MARKERS.insert(*digest);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use async_trait::async_trait;
    use snix_castore::Node;
    use snix_castore::PathComponent;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryPutter;

    /// An in-memory stub DirectoryService backed by a Mutex<HashMap>.
    struct StubDirectoryService {
        dirs: Mutex<HashMap<B3Digest, Directory>>,
    }

    impl StubDirectoryService {
        fn new() -> Self {
            Self { dirs: Mutex::new(HashMap::new()) }
        }

        fn put_sync(&self, digest: B3Digest, dir: Directory) {
            self.dirs.lock().unwrap().insert(digest, dir);
        }
    }

    #[async_trait]
    impl DirectoryService for StubDirectoryService {
        async fn get(&self, digest: &B3Digest) -> Result<Option<Directory>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.dirs.lock().unwrap().get(digest).cloned())
        }

        async fn put(&self, directory: Directory) -> Result<B3Digest, Box<dyn std::error::Error + Send + Sync>> {
            let digest = directory.digest();
            self.dirs.lock().unwrap().insert(digest, directory);
            Ok(digest)
        }

        fn get_recursive(
            &self,
            _root_directory_digest: &B3Digest,
        ) -> futures::stream::BoxStream<'_, Result<Directory, Box<dyn std::error::Error + Send + Sync>>> {
            unimplemented!("not used in tests")
        }

        fn put_multiple_start(&self) -> Box<dyn DirectoryPutter + '_> {
            unimplemented!("not used in tests")
        }
    }

    #[tokio::test]
    async fn blob_node_requires_blob_presence() {
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();
        let data = b"hello";
        let mut writer = blob.open_write().await;
        tokio::io::AsyncWriteExt::write_all(&mut writer, data).await.unwrap();
        let blob_digest = writer.close().await.unwrap();
        let node = Node::File {
            digest: blob_digest,
            size: 5,
            executable: false,
        };
        assert!(recursive_castore_completeness(&blob, &dir, &node).await.unwrap());

        // Missing blob
        // Missing blob
        let missing = B3Digest::from(blake3::hash(b"missing-blob").as_bytes());
        let missing_node = Node::File {
            digest: missing,
            size: 0,
            executable: false,
        };
        assert!(!recursive_castore_completeness(&blob, &dir, &missing_node).await.unwrap());
    }

    #[tokio::test]
    async fn symlink_is_always_complete() {
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();
        let node = Node::Symlink {
            target: SymlinkTarget::try_from("/bin/sh").unwrap(),
        };
        assert!(recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }

    #[tokio::test]
    async fn empty_directory_requires_existence() {
        GLOBAL_COMPLETENESS_MARKERS.clear();
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();
        let empty = Directory::new();
        let digest = empty.digest();
        let node = Node::Directory { digest, size: 0 };

        // Not inserted yet
        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());

        // Insert and recheck
        dir.put(empty).await.unwrap();
        assert!(recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
        assert!(GLOBAL_COMPLETENESS_MARKERS.contains(&digest));
    }

    #[tokio::test]
    async fn directory_with_missing_blob_child_is_incomplete() {
        GLOBAL_COMPLETENESS_MARKERS.clear();
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();

        let missing = B3Digest::from(blake3::hash(b"missing-child").as_bytes());
        let file_node = Node::File {
            digest: missing,
            size: 0,
            executable: false,
        };

        let name = PathComponent::try_from("missing.txt").unwrap();
        let mut child = Directory::new();
        child.add(name, file_node).unwrap();
        let child_digest = child.digest();
        dir.put(child).await.unwrap();

        let child_name = PathComponent::try_from("subdir").unwrap();
        let mut parent = Directory::new();
        parent.add(child_name, Node::Directory {
            digest: child_digest,
            size: 1,
        }).unwrap();
        let parent_digest = parent.digest();
        dir.put(parent).await.unwrap();

        let node = Node::Directory { digest: parent_digest, size: 2 };
        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }

    #[tokio::test]
    async fn completeness_marker_skips_redundant_traversal() {
        GLOBAL_COMPLETENESS_MARKERS.clear();
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();
        let empty = Directory::new();
        let digest = empty.digest();
        dir.put(empty).await.unwrap();

        let node = Node::Directory { digest, size: 0 };
        assert!(recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
        assert!(GLOBAL_COMPLETENESS_MARKERS.contains(&digest));

        // Second check uses marker — should still return true even if directory removed
        dir.dirs.lock().unwrap().clear();
        assert!(recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }

    #[tokio::test]
    async fn bounded_depth_rejects_extremely_deep_trees() {
        GLOBAL_COMPLETENESS_MARKERS.clear();
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();

        let mut current = Directory::new();
        let mut current_digest = current.digest();
        dir.put(current.clone()).await.unwrap();

        let name = PathComponent::try_from("sub").unwrap();
        for _ in 0..MAX_DEPTH + 5 {
            let mut parent = Directory::new();
            parent.add(name.clone(), Node::Directory {
                digest: current_digest,
                size: current.size(),
            }).unwrap();
            current_digest = parent.digest();
            dir.put(parent).await.unwrap();
            current = Directory::new();
        }

        let node = Node::Directory { digest: current_digest, size: 0 };
        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }
}