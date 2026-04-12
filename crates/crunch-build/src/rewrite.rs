//! Byte-level rewriting for content-addressed derivation outputs.
//!
//! After a CA build, the output may contain references to the provisional
//! placeholder path. These must be replaced with the final CA path.
//!
//! The scheme (following Nix):
//! 1. Replace provisional self-references with a zero marker (same length)
//! 2. Hash the marker-replaced content → CA path
//! 3. Replace zero markers with the final CA path
//! 4. Do NOT re-hash (the CA path is defined by the marker-replaced content)

/// Replace all occurrences of `provisional` in `data` with a zero marker
/// of the same byte length. Returns the rewritten data and whether any
/// replacements were made.
pub fn replace_provisional_with_marker(data: &[u8], provisional: &str) -> (Vec<u8>, bool) {
    let needle = provisional.as_bytes();
    let marker = vec![0u8; needle.len()];
    replace_bytes(data, needle, &marker)
}

/// Replace all zero markers (same length as `final_path`) with the final
/// CA path string.
pub fn replace_marker_with_final(data: &[u8], final_path: &str) -> Vec<u8> {
    let final_bytes = final_path.as_bytes();
    let marker = vec![0u8; final_bytes.len()];
    let (result, _) = replace_bytes(data, &marker, final_bytes);
    result
}

/// Replace all occurrences of `old_path` with `new_path` in the output.
/// Used for rewriting input provisional paths to their resolved CA paths.
///
/// Both paths MUST have the same byte length (store paths are fixed-width
/// for a given name).
pub fn replace_input_provisional(data: &[u8], old_path: &str, new_path: &str) -> Vec<u8> {
    assert_eq!(old_path.len(), new_path.len(), "old_path and new_path must have the same byte length");
    let (result, _) = replace_bytes(data, old_path.as_bytes(), new_path.as_bytes());
    result
}

/// Core byte replacement: find all non-overlapping occurrences of `needle`
/// in `haystack` and replace with `replacement`. Returns the new buffer and
/// whether any replacement occurred.
///
/// `needle` and `replacement` MUST have the same length.
fn replace_bytes(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> (Vec<u8>, bool) {
    assert_eq!(needle.len(), replacement.len(), "needle and replacement must be same length");
    assert!(!needle.is_empty(), "needle must not be empty");

    let nlen = needle.len();
    let mut result = Vec::with_capacity(haystack.len());
    let mut found = false;
    let mut i: usize = 0;

    while i < haystack.len() {
        if i + nlen <= haystack.len() && &haystack[i..i + nlen] == needle {
            result.extend_from_slice(replacement);
            found = true;
            i += nlen;
        } else {
            result.push(haystack[i]);
            i += 1;
        }
    }

    (result, found)
}

use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// Maximum directory tree depth for rewrite traversal.
/// Prevents runaway iteration from malformed castore data.
pub const MAX_REWRITE_DEPTH: u32 = 256;

/// Maximum number of nodes in the rewrite worklist.
/// Bounds memory usage against pathological tree structures.
const MAX_REWRITE_NODES: usize = 1_000_000;

/// Rewrite all file blobs in a castore `Node` tree, replacing `old_bytes`
/// with `new_bytes` (must be same length). Returns the new root Node and
/// whether any replacements were made.
///
/// Uses an explicit bounded worklist instead of recursion. The tree is
/// flattened depth-first, then processed leaf-to-root so child rewrites
/// are available when rebuilding parent directories.
pub async fn rewrite_node(
    node: &Node,
    old_bytes: &[u8],
    new_bytes: &[u8],
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(Node, bool), crate::Error> {
    assert_eq!(old_bytes.len(), new_bytes.len(), "old and new must be same length");
    assert!(!old_bytes.is_empty(), "replacement needle must not be empty");

    // Work items: flattened tree nodes with their path context.
    // `parent_idx` links a child back to its parent in the worklist.
    struct WorkItem {
        node: Node,
        name: Option<snix_castore::PathComponent>,
        parent_idx: Option<u32>,
        depth: u32,
    }

    // Phase 1: Flatten the tree depth-first into a worklist.
    let mut worklist: Vec<WorkItem> = Vec::new();
    let mut expand_stack: Vec<(Node, Option<snix_castore::PathComponent>, Option<u32>, u32)> = Vec::new();
    expand_stack.push((node.clone(), None, None, 0));

    while let Some((current, name, parent_idx, depth)) = expand_stack.pop() {
        if depth >= MAX_REWRITE_DEPTH {
            return Err(crate::Error::Store(format!(
                "rewrite depth limit ({MAX_REWRITE_DEPTH}) exceeded"
            )));
        }
        if worklist.len() >= MAX_REWRITE_NODES {
            return Err(crate::Error::Store(format!(
                "rewrite node count limit ({MAX_REWRITE_NODES}) exceeded"
            )));
        }
        let idx = u32::try_from(worklist.len()).map_err(|_| {
            crate::Error::Store("rewrite worklist overflow".to_string())
        })?;
        worklist.push(WorkItem {
            node: current.clone(),
            name,
            parent_idx,
            depth,
        });

        if let Node::Directory { ref digest, .. } = current {
            let dir = directory_service
                .get(digest)
                .await
                .map_err(|e| crate::Error::Store(format!("directory read for rewrite: {e}")))?;
            let dir = dir.ok_or_else(|| {
                crate::Error::Store(format!("directory {digest} not found for rewrite"))
            })?;
            // Push children in reverse order so they pop in forward order.
            let children: Vec<_> = dir.nodes().collect();
            for (child_name, child_node) in children.into_iter().rev() {
                expand_stack.push((
                    child_node.clone(),
                    Some(child_name.clone()),
                    Some(idx),
                    depth.saturating_add(1),
                ));
            }
        }
    }

    // Phase 2: Process leaf-to-root. Store rewritten nodes by index.
    let len = worklist.len();
    let mut results: Vec<Option<(Node, bool)>> = vec![None; len];

    for i in (0..len).rev() {
        let item = &worklist[i];
        let (new_node, found) = match &item.node {
            Node::File { digest, size, executable } => {
                rewrite_file_node(digest, *size, *executable, old_bytes, new_bytes, blob_service).await?
            }
            Node::Symlink { .. } => (item.node.clone(), false),
            Node::Directory { .. } => {
                // Collect already-rewritten children for this directory.
                let mut any_found = false;
                let mut new_dir = snix_castore::Directory::new();
                for (j, child_item) in worklist.iter().enumerate() {
                    if child_item.parent_idx == Some(i as u32) {
                        let (child_node, child_found) = results[j]
                            .take()
                            .expect("child must be processed before parent");
                        if child_found {
                            any_found = true;
                        }
                        let child_name = child_item.name.clone().expect("directory child must have a name");
                        new_dir
                            .add(child_name, child_node)
                            .map_err(|e| crate::Error::Store(format!("rebuilding directory: {e}")))?;
                    }
                }

                if !any_found {
                    (item.node.clone(), false)
                } else {
                    let new_digest = new_dir.digest();
                    let new_size = new_dir.size();
                    directory_service
                        .put(new_dir)
                        .await
                        .map_err(|e| crate::Error::Store(format!("storing rewritten directory: {e}")))?;
                    (Node::Directory { digest: new_digest, size: new_size }, true)
                }
            }
        };
        results[i] = Some((new_node, found));
    }

    results[0].take().ok_or_else(|| crate::Error::Store("rewrite produced no result".to_string()))
}

/// Rewrite a single file blob. Extracted to keep the worklist loop readable.
async fn rewrite_file_node(
    digest: &snix_castore::B3Digest,
    size: u64,
    executable: bool,
    old_bytes: &[u8],
    new_bytes: &[u8],
    blob_service: &(impl BlobService + Clone),
) -> Result<(Node, bool), crate::Error> {
    let mut reader = blob_service
        .open_read(digest)
        .await
        .map_err(|e| crate::Error::Store(format!("blob read for rewrite: {e}")))?;
    let reader =
        reader.as_mut().ok_or_else(|| crate::Error::Store(format!("blob {digest} not found for rewrite")))?;

    let mut data = Vec::with_capacity(size as usize);
    reader.read_to_end(&mut data).await.map_err(|e| crate::Error::Store(format!("reading blob: {e}")))?;

    let (rewritten, found) = replace_bytes(&data, old_bytes, new_bytes);

    if !found {
        return Ok((Node::File { digest: digest.clone(), size, executable }, false));
    }

    let mut writer = blob_service.open_write().await;
    writer
        .write_all(&rewritten)
        .await
        .map_err(|e| crate::Error::Store(format!("writing rewritten blob: {e}")))?;
    let new_digest =
        writer.close().await.map_err(|e| crate::Error::Store(format!("closing rewritten blob: {e}")))?;

    Ok((Node::File { digest: new_digest, size, executable }, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_provisional_basic() {
        let data = b"hello /nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-foo world";
        let provisional = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-foo";
        let (result, found) = replace_provisional_with_marker(data, provisional);
        assert!(found);
        // The provisional string should be replaced with zeros
        let zeros = vec![0u8; provisional.len()];
        assert!(result.windows(zeros.len()).any(|w| w == zeros.as_slice()));
        // "hello " prefix and " world" suffix preserved
        assert!(result.starts_with(b"hello "));
        assert!(result.ends_with(b" world"));
        assert_eq!(result.len(), data.len());
    }

    #[test]
    fn replace_provisional_no_match() {
        let data = b"nothing to replace here";
        let (result, found) = replace_provisional_with_marker(data, "/nix/store/xxx-nope");
        assert!(!found);
        assert_eq!(result, data);
    }

    #[test]
    fn replace_provisional_multiple() {
        let prov = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-foo";
        let mut data = Vec::new();
        data.extend_from_slice(prov.as_bytes());
        data.extend_from_slice(b":");
        data.extend_from_slice(prov.as_bytes());

        let (result, found) = replace_provisional_with_marker(&data, prov);
        assert!(found);
        // Two occurrences replaced
        let zeros = vec![0u8; prov.len()];
        let count = result.windows(zeros.len()).filter(|w| *w == zeros.as_slice()).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn marker_to_final_roundtrip() {
        let prov = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-foo";
        let final_path = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-foo";
        let data = format!("rpath={prov}:other").into_bytes();

        let (marked, found) = replace_provisional_with_marker(&data, prov);
        assert!(found);

        let final_data = replace_marker_with_final(&marked, final_path);
        let expected = format!("rpath={final_path}:other").into_bytes();
        assert_eq!(final_data, expected);
    }

    #[test]
    fn replace_input_provisional_basic() {
        let old = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-lib";
        let new = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-lib";
        let data = format!("link={old}").into_bytes();

        let result = replace_input_provisional(&data, old, new);
        let expected = format!("link={new}").into_bytes();
        assert_eq!(result, expected);
    }

    #[test]
    fn binary_data_handled() {
        // Embed a path in binary data with non-UTF8 bytes around it
        let prov = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bin";
        let mut data: Vec<u8> = vec![0xff, 0xfe, 0x00, 0x01];
        data.extend_from_slice(prov.as_bytes());
        data.extend_from_slice(&[0x80, 0x90]);

        let (marked, found) = replace_provisional_with_marker(&data, prov);
        assert!(found);
        assert_eq!(marked.len(), data.len());
        // Non-UTF8 prefix/suffix preserved
        assert_eq!(&marked[..4], &[0xff, 0xfe, 0x00, 0x01]);
        assert_eq!(&marked[marked.len() - 2..], &[0x80, 0x90]);
    }

    #[test]
    fn empty_data_no_panic() {
        let (result, found) = replace_provisional_with_marker(b"", "/nix/store/xxx-y");
        assert!(!found);
        assert!(result.is_empty());
    }

    #[test]
    #[should_panic(expected = "same byte length")]
    fn replace_input_different_lengths_panics() {
        replace_input_provisional(b"data", "/short", "/much-longer-path");
    }

    #[test]
    fn max_rewrite_depth_is_positive() {
        assert!(MAX_REWRITE_DEPTH >= 32, "rewrite depth limit must be reasonable");
        assert!(MAX_REWRITE_DEPTH <= 1024, "rewrite depth limit must not be unbounded");
    }
}
