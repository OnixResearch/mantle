//! Export castore nodes to the filesystem.
//!
//! Reconstructs files, directories, and symlinks on disk from the
//! content-addressed store. This is the inverse of `ingest_path`.

use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use tokio::io::AsyncReadExt;

/// Maximum directory tree depth when exporting to disk.
/// Prevents runaway recursion from malformed castore data.
pub const MAX_EXPORT_DEPTH: u32 = 128;

/// Export a castore Node to a filesystem path.
///
/// Uses an explicit bounded worklist instead of recursion. Directories
/// are created top-down, files/symlinks are written as encountered.
pub async fn export_castore_to_disk(
    node: &Node,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), String> {
    debug_assert!(dest.starts_with('/'), "export dest must be absolute path: {dest}");

    // Work items: (node, destination path, depth).
    let mut worklist: Vec<(Node, String, u32)> = Vec::new();
    worklist.push((node.clone(), dest.to_string(), 0));

    while let Some((current, current_dest, depth)) = worklist.pop() {
        if depth >= MAX_EXPORT_DEPTH {
            return Err(format!("directory depth limit ({MAX_EXPORT_DEPTH}) exceeded at {current_dest}"));
        }
        match &current {
            Node::File { digest, executable, .. } => {
                export_file_to_disk(digest, *executable, &current_dest, blob_service).await?;
            }
            Node::Symlink { target, .. } => {
                export_symlink_to_disk(target.as_ref(), &current_dest)?;
            }
            Node::Directory { digest, .. } => {
                std::fs::create_dir_all(&current_dest).map_err(|e| format!("creating dir {current_dest}: {e}"))?;

                let dir = directory_service
                    .get(digest)
                    .await
                    .map_err(|e| format!("fetching directory {digest}: {e}"))?
                    .ok_or_else(|| format!("directory {digest} not found in castore"))?;

                // Push children onto the worklist (forward order preserved
                // because we push in order and pop from the end, but order
                // doesn't matter for correctness here — only depth).
                for (name, child_node) in dir.nodes() {
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| format!("non-UTF8 filename in directory: {e}"))?;
                    let child_dest = format!("{current_dest}/{name_str}");
                    worklist.push((child_node.clone(), child_dest, depth.saturating_add(1)));
                }
            }
        }
    }
    Ok(())
}

/// Write a single file blob to disk.
async fn export_file_to_disk(
    digest: &snix_castore::B3Digest,
    executable: bool,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
) -> Result<(), String> {
    let mut reader = blob_service
        .open_read(digest)
        .await
        .map_err(|e| format!("opening blob {digest}: {e}"))?
        .ok_or_else(|| format!("blob {digest} not found in castore"))?;

    if let Some(parent) = std::path::Path::new(dest).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("creating parent dir: {e}"))?;
    }
    let mut file = std::fs::File::create(dest).map_err(|e| format!("creating {dest}: {e}"))?;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).await.map_err(|e| format!("reading blob: {e}"))?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buf[..n]).map_err(|e| format!("writing {dest}: {e}"))?;
    }
    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o555))
            .map_err(|e| format!("setting executable: {e}"))?;
    }
    Ok(())
}

/// Create a symlink on disk.
fn export_symlink_to_disk(target: &[u8], dest: &str) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(dest).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("creating parent dir: {e}"))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let target_os = std::ffi::OsStr::from_bytes(target);
        std::os::unix::fs::symlink(target_os, dest).map_err(|e| format!("creating symlink {dest}: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use snix_castore::Directory;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use tokio::io::AsyncWriteExt;

    use super::*;

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
    }

    async fn insert_blob(bs: &MemoryBlobService, data: &[u8]) -> (snix_castore::B3Digest, Node) {
        let mut writer = bs.open_write().await;
        writer.write_all(data).await.unwrap();
        let digest = writer.close().await.unwrap();
        let node = Node::File {
            digest,
            size: data.len() as u64,
            executable: false,
        };
        (digest, node)
    }

    // -- File export --

    #[tokio::test]
    async fn export_file_writes_content() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"hello export";
        let (_, node) = insert_blob(&bs, data).await;

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/out", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        let written = std::fs::read(&dest).unwrap();
        assert_eq!(written, data);
    }

    #[tokio::test]
    async fn export_file_executable_sets_permissions() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"#!/bin/sh\necho hi";
        let mut writer = bs.open_write().await;
        writer.write_all(data).await.unwrap();
        let digest = writer.close().await.unwrap();
        let node = Node::File {
            digest,
            size: data.len() as u64,
            executable: true,
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/script", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&dest).unwrap().permissions().mode();
        assert_ne!(mode & 0o111, 0, "executable bit should be set");
    }

    #[tokio::test]
    async fn export_file_empty_content() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"").await;

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/empty", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        let written = std::fs::read(&dest).unwrap();
        assert!(written.is_empty());
    }

    #[tokio::test]
    async fn export_file_creates_parent_directories() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"nested").await;

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/a/b/c/file", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), b"nested");
    }

    // -- Symlink export --

    #[tokio::test]
    async fn export_symlink_creates_link() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node = Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from("/some/target").unwrap(),
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/link", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        let target = std::fs::read_link(&dest).unwrap();
        assert_eq!(target.to_str().unwrap(), "/some/target");
    }

    // -- Directory export --

    #[tokio::test]
    async fn export_directory_creates_files() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (digest_a, _) = insert_blob(&bs, b"content-a").await;
        let (digest_b, _) = insert_blob(&bs, b"content-b").await;

        let mut dir = Directory::new();
        dir.add("a.txt".try_into().unwrap(), Node::File {
            digest: digest_a,
            size: 9,
            executable: false,
        })
        .unwrap();
        dir.add("b.txt".try_into().unwrap(), Node::File {
            digest: digest_b,
            size: 9,
            executable: false,
        })
        .unwrap();

        let dir_digest = dir.digest();
        let dir_size = dir.size();
        ds.put(dir).await.unwrap();

        let node = Node::Directory {
            digest: dir_digest,
            size: dir_size,
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/mydir", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        assert_eq!(std::fs::read(format!("{dest}/a.txt")).unwrap(), b"content-a");
        assert_eq!(std::fs::read(format!("{dest}/b.txt")).unwrap(), b"content-b");
    }

    #[tokio::test]
    async fn export_nested_directory() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (file_digest, _) = insert_blob(&bs, b"leaf").await;

        let mut inner = Directory::new();
        inner
            .add("leaf.txt".try_into().unwrap(), Node::File {
                digest: file_digest,
                size: 4,
                executable: false,
            })
            .unwrap();
        let inner_digest = inner.digest();
        let inner_size = inner.size();
        ds.put(inner).await.unwrap();

        let mut outer = Directory::new();
        outer
            .add("sub".try_into().unwrap(), Node::Directory {
                digest: inner_digest,
                size: inner_size,
            })
            .unwrap();
        let outer_digest = outer.digest();
        let outer_size = outer.size();
        ds.put(outer).await.unwrap();

        let node = Node::Directory {
            digest: outer_digest,
            size: outer_size,
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/root", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        assert_eq!(std::fs::read(format!("{dest}/sub/leaf.txt")).unwrap(), b"leaf");
    }

    #[tokio::test]
    async fn export_directory_with_symlink() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (file_digest, _) = insert_blob(&bs, b"real").await;

        let mut dir = Directory::new();
        dir.add("real.txt".try_into().unwrap(), Node::File {
            digest: file_digest,
            size: 4,
            executable: false,
        })
        .unwrap();
        dir.add("link.txt".try_into().unwrap(), Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from("real.txt").unwrap(),
        })
        .unwrap();

        let dir_digest = dir.digest();
        let dir_size = dir.size();
        ds.put(dir).await.unwrap();

        let node = Node::Directory {
            digest: dir_digest,
            size: dir_size,
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/mixed", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        assert_eq!(std::fs::read(format!("{dest}/real.txt")).unwrap(), b"real");
        let link_target = std::fs::read_link(format!("{dest}/link.txt")).unwrap();
        assert_eq!(link_target.to_str().unwrap(), "real.txt");
    }

    // -- Error paths --

    #[tokio::test]
    async fn export_missing_blob_returns_error() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let fake_digest = snix_castore::B3Digest::from(&[0xffu8; 32]);
        let node = Node::File {
            digest: fake_digest,
            size: 10,
            executable: false,
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/missing", tmp.path().display());

        let err = export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap_err();
        assert!(err.contains("not found"), "error should mention missing blob: {err}");
    }

    #[tokio::test]
    async fn export_missing_directory_returns_error() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let fake_digest = snix_castore::B3Digest::from(&[0xeeu8; 32]);
        let node = Node::Directory {
            digest: fake_digest,
            size: 0,
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/missing_dir", tmp.path().display());

        let err = export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap_err();
        assert!(err.contains("not found"), "error should mention missing directory: {err}");
    }

    // -- Depth limit --

    /// Build a chain of nested directories to a given depth, with a file at the leaf.
    async fn build_deep_chain(bs: &MemoryBlobService, ds: &impl DirectoryService, depth: u32) -> Node {
        let (file_digest, _) = insert_blob(bs, b"leaf").await;
        let leaf = Node::File {
            digest: file_digest,
            size: 4,
            executable: false,
        };

        let mut current = leaf;
        for _depth_index in (0..depth).rev() {
            // Use a fixed name so we don't need dynamic PathComponent.
            let mut dir = Directory::new();
            dir.add("d".try_into().unwrap(), current).unwrap();
            let dig = dir.digest();
            let sz = dir.size();
            ds.put(dir).await.unwrap();
            current = Node::Directory { digest: dig, size: sz };
        }
        current
    }

    #[tokio::test]
    async fn export_depth_limit_enforced() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        // Build a chain deeper than MAX_EXPORT_DEPTH.
        let node = build_deep_chain(&bs, &ds, MAX_EXPORT_DEPTH + 1).await;

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/deep", tmp.path().display());

        let err = export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap_err();
        assert!(err.contains("depth limit"), "error should mention depth limit: {err}");
    }

    #[tokio::test]
    async fn export_moderate_depth_succeeds() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        // 5 levels deep — well under the limit.
        let node = build_deep_chain(&bs, &ds, 5).await;

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/moderate", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        // Verify the leaf file exists at the expected depth (d/d/d/d/d).
        let mut path = dest.to_string();
        for _ in 0..5 {
            path = format!("{path}/d");
        }
        assert_eq!(std::fs::read(&path).unwrap(), b"leaf");
    }
}
