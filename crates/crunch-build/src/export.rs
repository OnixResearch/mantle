//! Export castore nodes to the filesystem.
//!
//! Reconstructs files, directories, and symlinks on disk from the
//! content-addressed store. This is the inverse of `ingest_path`.

use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::Node;
use tokio::io::AsyncReadExt;

/// Maximum directory tree depth when exporting to disk.
/// Prevents runaway recursion from malformed castore data.
const MAX_EXPORT_DEPTH: u32 = 128;

/// Export a castore Node to a filesystem path.
///
/// Walks the Node tree and writes files, directories, and symlinks to
/// disk. Recursive via Box::pin for directory nodes.
pub(crate) async fn export_castore_to_disk(
    node: &Node,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), String> {
    // Tiger Style: assert destination is absolute.
    debug_assert!(dest.starts_with('/'), "export dest must be absolute path: {dest}");
    export_castore_inner(node, dest, blob_service, directory_service, 0).await
}

async fn export_castore_inner(
    node: &Node,
    dest: &str,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
    depth: u32,
) -> Result<(), String> {
    if depth >= MAX_EXPORT_DEPTH {
        return Err(format!("directory depth limit ({MAX_EXPORT_DEPTH}) exceeded at {dest}"));
    }
    match node {
        Node::File { digest, executable, .. } => {
            let mut reader = blob_service
                .open_read(digest)
                .await
                .map_err(|e| format!("opening blob {digest}: {e}"))?
                .ok_or_else(|| format!("blob {digest} not found in castore"))?;

            if let Some(parent) = std::path::Path::new(dest).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("creating parent dir: {e}"))?;
            }
            let mut file = std::fs::File::create(dest)
                .map_err(|e| format!("creating {dest}: {e}"))?;
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| format!("reading blob: {e}"))?;
                if n == 0 { break; }
                std::io::Write::write_all(&mut file, &buf[..n])
                    .map_err(|e| format!("writing {dest}: {e}"))?;
            }
            #[cfg(unix)]
            if *executable {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o555))
                    .map_err(|e| format!("setting executable: {e}"))?;
            }
        }
        Node::Symlink { target, .. } => {
            if let Some(parent) = std::path::Path::new(dest).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("creating parent dir: {e}"))?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let target_os = std::ffi::OsStr::from_bytes(target.as_ref());
                std::os::unix::fs::symlink(target_os, dest)
                    .map_err(|e| format!("creating symlink {dest}: {e}"))?;
            }
        }
        Node::Directory { digest, .. } => {
            std::fs::create_dir_all(dest)
                .map_err(|e| format!("creating dir {dest}: {e}"))?;

            let dir = directory_service
                .get(digest)
                .await
                .map_err(|e| format!("fetching directory {digest}: {e}"))?
                .ok_or_else(|| format!("directory {digest} not found in castore"))?;

            for (name, child_node) in dir.nodes() {
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| format!("non-UTF8 filename in directory: {e}"))?;
                let child_dest = format!("{dest}/{name_str}");
                Box::pin(export_castore_inner(
                    child_node,
                    &child_dest,
                    blob_service,
                    directory_service,
                    depth.saturating_add(1),
                ))
                .await?;
            }
        }
    }
    Ok(())
}
