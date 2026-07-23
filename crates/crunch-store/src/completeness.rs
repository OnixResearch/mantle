//! Recursive castore completeness checking against the active storage services.
//!
//! r[impl cache_substitution.castore_completeness]

use snix_castore::B3Digest;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use tokio::io::AsyncReadExt;

use crate::Error;

/// Maximum recursive node visits before aborting (safety bound).
const MAX_RECURSIVE_NODES: u32 = 100_000;
/// Maximum directory tree depth before aborting.
const MAX_DEPTH: u32 = 128;
const BLOB_COMPLETENESS_READ_BUFFER_BYTES: usize = 65_536;
const INITIAL_COMPLETENESS_WORKLIST_CAPACITY: usize = 128;

/// Recursively check that a node's full castore tree is present.
///
/// Checks every child blob and directory under `node` root. Symlinks
/// are always complete because their targets are inline. Every call checks
/// the active services so stale process-global facts cannot admit missing data.
pub async fn recursive_castore_completeness(
    blob_service: &dyn BlobService,
    directory_service: &dyn DirectoryService,
    node: &Node,
) -> Result<bool, Error> {
    // Iterative stack-based traversal to avoid recursive async fn.
    let mut stack: Vec<(Node, u32)> = Vec::with_capacity(INITIAL_COMPLETENESS_WORKLIST_CAPACITY);
    stack.push((node.clone(), 0));
    let mut visited_node_count = 0u32;
    assert_eq!(stack.len(), 1);
    assert_eq!(visited_node_count, 0);

    while let Some((current_node, depth)) = stack.pop() {
        if depth > MAX_DEPTH {
            return Ok(false);
        }
        if visited_node_count >= MAX_RECURSIVE_NODES {
            return Ok(false);
        }
        visited_node_count = visited_node_count.saturating_add(1);

        match &current_node {
            Node::File { digest, size, .. } => {
                if !blob_has_declared_size(blob_service, digest, *size).await? {
                    return Ok(false);
                }
            }
            Node::Symlink { .. } => {
                // Symlinks are always complete.
            }
            Node::Directory { digest, .. } => {
                // Fetch the directory node's children from the active service.
                let dir = match directory_service
                    .get(digest)
                    .await
                    .map_err(|e| Error::DirectoryService(format!("completeness check: {e}")))?
                {
                    Some(dir) => dir,
                    None => return Ok(false),
                };

                for child in dir.nodes() {
                    if stack.len()
                        >= usize::try_from(MAX_RECURSIVE_NODES).map_err(|_| {
                            Error::DirectoryService("completeness node limit does not fit usize".to_string())
                        })?
                    {
                        return Ok(false);
                    }
                    stack.push((child.1.clone(), depth.saturating_add(1)));
                }
            }
        }
    }

    Ok(true)
}

async fn blob_has_declared_size(
    blob_service: &dyn BlobService,
    digest: &B3Digest,
    declared_size: u64,
) -> Result<bool, Error> {
    assert_eq!(digest.as_slice().len(), B3Digest::LENGTH);
    let Some(chunks) = blob_service
        .chunks(digest)
        .await
        .map_err(|e| Error::BlobService(format!("completeness chunk check: {e}")))?
    else {
        return Ok(false);
    };

    if chunks.is_empty() {
        return blob_reader_has_declared_size(blob_service, digest, declared_size).await;
    }
    assert!(!chunks.is_empty());

    let mut total_size_bytes = 0u64;
    for chunk in chunks {
        total_size_bytes = total_size_bytes
            .checked_add(chunk.size)
            .ok_or_else(|| Error::BlobService("completeness chunk check: chunk size overflow".to_string()))?;
        let chunk_digest: B3Digest = chunk
            .digest
            .try_into()
            .map_err(|_| Error::BlobService("completeness chunk check: invalid chunk digest".to_string()))?;
        let is_chunk_present = blob_service
            .has(&chunk_digest)
            .await
            .map_err(|e| Error::BlobService(format!("completeness chunk check: {e}")))?;
        if !is_chunk_present {
            return Ok(false);
        }
        if !blob_reader_has_declared_size(blob_service, &chunk_digest, chunk.size).await? {
            return Ok(false);
        }
    }

    Ok(total_size_bytes == declared_size)
}

async fn blob_reader_has_declared_size(
    blob_service: &dyn BlobService,
    digest: &B3Digest,
    declared_size: u64,
) -> Result<bool, Error> {
    assert_eq!(digest.as_slice().len(), B3Digest::LENGTH);
    let Some(mut reader) = blob_service
        .open_read(digest)
        .await
        .map_err(|e| Error::BlobService(format!("completeness blob read: {e}")))?
    else {
        return Ok(false);
    };

    let mut buffer = vec![0u8; BLOB_COMPLETENESS_READ_BUFFER_BYTES];
    assert!(!buffer.is_empty());
    let mut total_size_bytes = 0u64;
    let mut hasher = blake3::Hasher::new();
    for _read_index in 0..=declared_size {
        let read_byte_count = reader
            .read(&mut buffer)
            .await
            .map_err(|e| Error::BlobService(format!("completeness blob read: {e}")))?;
        if read_byte_count == 0 {
            let observed_digest = B3Digest::from(hasher.finalize().as_bytes());
            return Ok(total_size_bytes == declared_size && observed_digest == *digest);
        }
        hasher.update(&buffer[..read_byte_count]);
        let read_byte_count = u64::try_from(read_byte_count)
            .map_err(|_| Error::BlobService("completeness blob read: read size overflow".to_string()))?;
        total_size_bytes = total_size_bytes
            .checked_add(read_byte_count)
            .ok_or_else(|| Error::BlobService("completeness blob read: blob size overflow".to_string()))?;
        if total_size_bytes > declared_size {
            return Ok(false);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    const DEPTH_OVERFLOW_MARGIN: u32 = 5;

    use async_trait::async_trait;
    use snix_castore::Directory;
    use snix_castore::Node;
    use snix_castore::PathComponent;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::BlobWriter;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryPutter;
    use snix_castore::proto::stat_blob_response::ChunkMeta;

    use super::*;

    struct ChunkMetadataBlobService {
        root_digest: B3Digest,
        chunk_digest: B3Digest,
        chunk_size: u64,
        chunk_bytes: Vec<u8>,
    }

    #[async_trait]
    impl BlobService for ChunkMetadataBlobService {
        async fn has(&self, digest: &B3Digest) -> std::io::Result<bool> {
            Ok(*digest == self.root_digest || *digest == self.chunk_digest)
        }

        async fn open_read(
            &self,
            digest: &B3Digest,
        ) -> std::io::Result<Option<Box<dyn snix_castore::blobservice::BlobReader>>> {
            if *digest != self.chunk_digest {
                return Ok(None);
            }
            Ok(Some(Box::new(std::io::Cursor::new(self.chunk_bytes.clone()))))
        }

        async fn open_write(&self) -> Box<dyn BlobWriter> {
            unimplemented!("test service is read-only")
        }

        async fn chunks(&self, digest: &B3Digest) -> std::io::Result<Option<Vec<ChunkMeta>>> {
            if *digest != self.root_digest {
                return Ok(None);
            }
            Ok(Some(vec![ChunkMeta {
                digest: self.chunk_digest.as_slice().to_vec().into(),
                size: self.chunk_size,
            }]))
        }
    }

    /// An in-memory stub DirectoryService backed by a Mutex<HashMap>.
    struct StubDirectoryService {
        dirs: Mutex<HashMap<B3Digest, Directory>>,
    }

    impl StubDirectoryService {
        fn new() -> Self {
            Self {
                dirs: Mutex::new(HashMap::new()),
            }
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
        let missing = B3Digest::from(blake3::hash(b"missing-blob").as_bytes());
        let missing_node = Node::File {
            digest: missing,
            size: 0,
            executable: false,
        };
        assert!(!recursive_castore_completeness(&blob, &dir, &missing_node).await.unwrap());
    }

    #[tokio::test]
    async fn blob_node_rejects_declared_size_mismatch() {
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();
        let data = b"hello";
        let mut writer = blob.open_write().await;
        tokio::io::AsyncWriteExt::write_all(&mut writer, data).await.unwrap();
        let blob_digest = writer.close().await.unwrap();
        let actual_size = u64::try_from(data.len()).expect("test data length fits u64");
        let declared_size = actual_size.saturating_add(1);
        let node = Node::File {
            digest: blob_digest,
            size: declared_size,
            executable: false,
        };

        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }

    #[tokio::test]
    async fn chunked_blob_metadata_must_match_declared_size() {
        let root_digest = B3Digest::from(blake3::hash(b"root").as_bytes());
        let chunk_digest = B3Digest::from(blake3::hash(b"chunk").as_bytes());
        let chunk_bytes = b"chunk";
        let chunk_size = u64::try_from(chunk_bytes.len()).expect("test chunk size fits u64");
        let complete_blob = ChunkMetadataBlobService {
            root_digest,
            chunk_digest,
            chunk_size,
            chunk_bytes: chunk_bytes.to_vec(),
        };
        let short_blob = ChunkMetadataBlobService {
            root_digest,
            chunk_digest,
            chunk_size,
            chunk_bytes: b"chun".to_vec(),
        };
        let wrong_digest_blob = ChunkMetadataBlobService {
            root_digest,
            chunk_digest,
            chunk_size,
            chunk_bytes: b"wrong".to_vec(),
        };
        let dir = StubDirectoryService::new();
        let matching = Node::File {
            digest: root_digest,
            size: chunk_size,
            executable: false,
        };
        let mismatched = Node::File {
            digest: root_digest,
            size: chunk_size.saturating_add(1),
            executable: false,
        };

        assert!(recursive_castore_completeness(&complete_blob, &dir, &matching).await.unwrap());
        assert!(!recursive_castore_completeness(&complete_blob, &dir, &mismatched).await.unwrap());
        assert!(!recursive_castore_completeness(&short_blob, &dir, &matching).await.unwrap());
        assert!(!recursive_castore_completeness(&wrong_digest_blob, &dir, &matching).await.unwrap());
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
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();
        // Use a non-empty directory with a unique child to avoid digest
        // collision with parallel tests that also create empty directories.
        let mut unique_dir = Directory::new();
        let name = PathComponent::try_from("unique-marker").unwrap();
        unique_dir
            .add(name, Node::Symlink {
                target: SymlinkTarget::try_from("placeholder").unwrap(),
            })
            .unwrap();
        let digest = unique_dir.digest();
        let node = Node::Directory {
            digest,
            size: unique_dir.size(),
        };

        // Not inserted yet
        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());

        // Insert and recheck
        dir.put(unique_dir).await.unwrap();
        assert!(recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }

    #[tokio::test]
    async fn directory_with_missing_blob_child_is_incomplete() {
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
        parent
            .add(child_name, Node::Directory {
                digest: child_digest,
                size: 1,
            })
            .unwrap();
        let parent_digest = parent.digest();
        dir.put(parent).await.unwrap();

        let node = Node::Directory {
            digest: parent_digest,
            size: 2,
        };
        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
        assert_ne!(parent_digest, child_digest);
    }

    #[tokio::test]
    async fn completeness_rechecks_and_rejects_removed_directory() {
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();
        let empty = Directory::new();
        let digest = empty.digest();
        dir.put(empty).await.unwrap();

        let node = Node::Directory { digest, size: 0 };
        assert!(recursive_castore_completeness(&blob, &dir, &node).await.unwrap());

        dir.dirs.lock().unwrap().clear();
        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }

    #[tokio::test]
    async fn bounded_depth_rejects_extremely_deep_trees() {
        let blob = MemoryBlobService::default();
        let dir = StubDirectoryService::new();

        let mut current = Directory::new();
        let mut current_digest = current.digest();
        dir.put(current.clone()).await.unwrap();

        let name = PathComponent::try_from("sub").unwrap();
        for _ in 0..MAX_DEPTH + DEPTH_OVERFLOW_MARGIN {
            let mut parent = Directory::new();
            parent
                .add(name.clone(), Node::Directory {
                    digest: current_digest,
                    size: current.size(),
                })
                .unwrap();
            current_digest = parent.digest();
            dir.put(parent).await.unwrap();
            current = Directory::new();
        }

        let node = Node::Directory {
            digest: current_digest,
            size: 0,
        };
        assert!(!recursive_castore_completeness(&blob, &dir, &node).await.unwrap());
    }
}
