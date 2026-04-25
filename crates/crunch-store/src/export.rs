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

const MAX_WORKLIST_ENTRIES: usize = 1_000_000;
const INITIAL_WORKLIST_CAPACITY: usize = 64;
const NORMALIZED_MTIME_SECONDS: i64 = 1;
const NORMALIZED_MTIME_NANOSECONDS: u32 = 0;
const NORMALIZED_NON_EXECUTABLE_MODE: u32 = 0o444;
const NORMALIZED_EXECUTABLE_MODE: u32 = 0o555;
const NORMALIZED_DIRECTORY_MODE: u32 = 0o555;

#[derive(Debug)]
enum ExportWorkItem {
    WriteNode { node: Node, dest: String, depth: u32 },
    FinalizeDirectory { dest: String },
}

/// Export a castore Node to a filesystem path.
///
/// Uses an explicit bounded worklist instead of recursion. Directories
/// are created top-down and metadata-normalized after children are written.
pub async fn export_castore_to_disk(
    node: &Node,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), String> {
    debug_assert!(dest.starts_with('/'), "export dest must be absolute path: {dest}");

    let mut worklist: Vec<ExportWorkItem> = Vec::with_capacity(INITIAL_WORKLIST_CAPACITY);
    worklist.push(ExportWorkItem::WriteNode {
        node: node.clone(),
        dest: dest.to_string(),
        depth: 0,
    });

    while let Some(item) = worklist.pop() {
        match item {
            ExportWorkItem::WriteNode { node, dest, depth } => {
                export_node_to_disk(node, dest, depth, &mut worklist, blob_service, directory_service).await?;
            }
            ExportWorkItem::FinalizeDirectory { dest } => {
                normalize_directory_metadata(&dest)?;
            }
        }
    }
    Ok(())
}

async fn export_node_to_disk(
    node: Node,
    dest: String,
    depth: u32,
    worklist: &mut Vec<ExportWorkItem>,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), String> {
    if depth >= MAX_EXPORT_DEPTH {
        return Err(format!("directory depth limit ({MAX_EXPORT_DEPTH}) exceeded at {dest}"));
    }
    match node {
        Node::File { digest, executable, .. } => {
            export_file_to_disk(&digest, executable, &dest, blob_service).await?;
        }
        Node::Symlink { target, .. } => {
            export_symlink_to_disk(target.as_ref(), &dest)?;
        }
        Node::Directory { digest, .. } => {
            export_directory_to_disk(&digest, &dest, depth, worklist, directory_service).await?;
        }
    }
    Ok(())
}

async fn export_directory_to_disk(
    digest: &snix_castore::B3Digest,
    dest: &str,
    depth: u32,
    worklist: &mut Vec<ExportWorkItem>,
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| format!("creating dir {dest}: {e}"))?;
    let dir = directory_service
        .get(digest)
        .await
        .map_err(|e| format!("fetching directory {digest}: {e}"))?
        .ok_or_else(|| format!("directory {digest} not found in castore"))?;

    assert!(worklist.len() < MAX_WORKLIST_ENTRIES, "export worklist exceeded {MAX_WORKLIST_ENTRIES} entries");
    worklist.push(ExportWorkItem::FinalizeDirectory { dest: dest.to_string() });
    for (name, child_node) in dir.nodes() {
        let name_str =
            std::str::from_utf8(name.as_ref()).map_err(|e| format!("non-UTF8 filename in directory: {e}"))?;
        let child_dest = format!("{dest}/{name_str}");
        assert!(worklist.len() < MAX_WORKLIST_ENTRIES, "export worklist exceeded {MAX_WORKLIST_ENTRIES} entries");
        worklist.push(ExportWorkItem::WriteNode {
            node: child_node.clone(),
            dest: child_dest,
            depth: depth.saturating_add(1),
        });
    }
    Ok(())
}

/// Write a single file blob to disk.
async fn export_file_to_disk(
    digest: &snix_castore::B3Digest,
    is_executable: bool,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
) -> Result<(), String> {
    assert!(!dest.is_empty(), "export dest must not be empty");
    assert!(dest.starts_with('/'), "export dest must be absolute path: {dest}");

    let mut reader = blob_service
        .open_read(digest)
        .await
        .map_err(|e| format!("opening blob {digest}: {e}"))?
        .ok_or_else(|| format!("blob {digest} not found in castore"))?;

    if let Some(parent) = std::path::Path::new(dest).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("creating parent dir: {e}"))?;
    }
    let mut file = std::fs::File::create(dest).map_err(|e| format!("creating {dest}: {e}"))?;
    const BUF_SIZE: usize = 65_536;
    const MAX_BLOB_WRITE_BYTES: u64 = 4_294_967_296; // 4 GiB
    let mut buf = vec![0u8; BUF_SIZE];
    let mut written_bytes: u64 = 0;
    const MAX_READ_ITERATIONS: u64 = 65_537; // MAX_BLOB_WRITE_BYTES / BUF_SIZE + 1
    for _ in 0..MAX_READ_ITERATIONS {
        let n = reader.read(&mut buf).await.map_err(|e| format!("reading blob: {e}"))?;
        if n == 0 {
            break;
        }
        let n_u64 = match u64::try_from(n) {
            Ok(v) => v,
            Err(_) => return Err(format!("read size {n} overflows u64")),
        };
        written_bytes = written_bytes.saturating_add(n_u64);
        assert!(written_bytes <= MAX_BLOB_WRITE_BYTES, "blob write exceeded {MAX_BLOB_WRITE_BYTES} bytes");
        std::io::Write::write_all(&mut file, &buf[..n]).map_err(|e| format!("writing {dest}: {e}"))?;
    }
    std::io::Write::flush(&mut file).map_err(|e| format!("flushing {dest}: {e}"))?;
    drop(file);
    normalize_file_metadata(dest, is_executable)?;
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
    normalize_symlink_timestamp(dest)?;
    Ok(())
}

fn normalized_file_time() -> filetime::FileTime {
    filetime::FileTime::from_unix_time(NORMALIZED_MTIME_SECONDS, NORMALIZED_MTIME_NANOSECONDS)
}

fn normalize_file_metadata(dest: &str, is_executable: bool) -> Result<(), String> {
    let mode = if is_executable {
        NORMALIZED_EXECUTABLE_MODE
    } else {
        NORMALIZED_NON_EXECUTABLE_MODE
    };
    set_path_mode(dest, mode)?;
    filetime::set_file_mtime(dest, normalized_file_time())
        .map_err(|e| format!("setting file mtime for {dest}: {e}"))?;
    Ok(())
}

fn normalize_directory_metadata(dest: &str) -> Result<(), String> {
    set_path_mode(dest, NORMALIZED_DIRECTORY_MODE)?;
    filetime::set_file_mtime(dest, normalized_file_time())
        .map_err(|e| format!("setting directory mtime for {dest}: {e}"))?;
    Ok(())
}

#[cfg(unix)]
fn set_path_mode(dest: &str, mode: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dest, std::fs::Permissions::from_mode(mode))
        .map_err(|e| format!("setting permissions for {dest}: {e}"))
}

#[cfg(not(unix))]
fn set_path_mode(_dest: &str, _mode: u32) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn normalize_symlink_timestamp(dest: &str) -> Result<(), String> {
    let fixed_time = normalized_file_time();
    filetime::set_symlink_file_times(dest, fixed_time, fixed_time)
        .map_err(|e| format!("setting symlink mtime for {dest}: {e}"))
}

#[cfg(not(unix))]
fn normalize_symlink_timestamp(_dest: &str) -> Result<(), String> {
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

    #[cfg(unix)]
    fn mode_bits(path: &str) -> u32 {
        const MODE_PERMISSION_MASK: u32 = 0o777;
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).unwrap().permissions().mode() & MODE_PERMISSION_MASK
    }

    fn mtime_seconds(path: &str) -> i64 {
        let metadata = std::fs::metadata(path).unwrap();
        filetime::FileTime::from_last_modification_time(&metadata).unix_seconds()
    }

    fn symlink_mtime_seconds(path: &str) -> i64 {
        let metadata = std::fs::symlink_metadata(path).unwrap();
        filetime::FileTime::from_last_modification_time(&metadata).unix_seconds()
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
    async fn export_file_non_executable_sets_permissions() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"plain").await;

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/plain", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        #[cfg(unix)]
        assert_eq!(mode_bits(&dest), NORMALIZED_NON_EXECUTABLE_MODE);
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

        #[cfg(unix)]
        assert_eq!(mode_bits(&dest), NORMALIZED_EXECUTABLE_MODE);
    }

    #[tokio::test]
    async fn export_file_sets_mtime() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"mtime").await;

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/mtime", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        assert_eq!(mtime_seconds(&dest), NORMALIZED_MTIME_SECONDS);
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

    #[tokio::test]
    async fn export_symlink_sets_lmtime() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node = Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
        };

        let tmp = tempfile::tempdir().unwrap();
        let dest = format!("{}/link", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        assert_eq!(symlink_mtime_seconds(&dest), NORMALIZED_MTIME_SECONDS);
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
    async fn export_directory_sets_permissions_and_mtime() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let mut dir = Directory::new();
        let (file_digest, _) = insert_blob(&bs, b"child").await;
        dir.add("child.txt".try_into().unwrap(), Node::File {
            digest: file_digest,
            size: 5,
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
        let dest = format!("{}/dir", tmp.path().display());

        export_castore_to_disk(&node, &dest, &bs, &ds).await.unwrap();

        #[cfg(unix)]
        assert_eq!(mode_bits(&dest), NORMALIZED_DIRECTORY_MODE);
        assert_eq!(mtime_seconds(&dest), NORMALIZED_MTIME_SECONDS);
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
