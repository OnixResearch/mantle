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

/// Length of a Nix store path hash component in nixbase32: 32 chars.
const STORE_PATH_HASH_LEN: usize = 32;

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
    assert_eq!(
        old_path.len(),
        new_path.len(),
        "old_path and new_path must have the same byte length"
    );
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

use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::{B3Digest, Node};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Rewrite all file blobs in a castore `Node` tree, replacing `old_bytes`
/// with `new_bytes` (must be same length). Returns the new root Node and
/// whether any replacements were made.
///
/// For File nodes: reads the blob, does byte replacement, writes back.
/// For Directory nodes: recurses into children.
/// For Symlink nodes: no-op (symlink targets are not store-path-bearing).
pub async fn rewrite_node(
    node: &Node,
    old_bytes: &[u8],
    new_bytes: &[u8],
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(Node, bool), crate::Error> {
    assert_eq!(old_bytes.len(), new_bytes.len(), "old and new must be same length");

    match node {
        Node::File { digest, size, executable } => {
            let mut reader = blob_service
                .open_read(digest)
                .await
                .map_err(|e| crate::Error::Store(format!("blob read for rewrite: {e}")))?;
            let reader = reader.as_mut()
                .ok_or_else(|| crate::Error::Store(format!("blob {digest} not found for rewrite")))?;

            let mut data = Vec::with_capacity(*size as usize);
            reader.read_to_end(&mut data).await
                .map_err(|e| crate::Error::Store(format!("reading blob: {e}")))?;

            let (rewritten, found) = replace_bytes(&data, old_bytes, new_bytes);

            if !found {
                return Ok((node.clone(), false));
            }

            let mut writer = blob_service.open_write().await;
            writer.write_all(&rewritten).await
                .map_err(|e| crate::Error::Store(format!("writing rewritten blob: {e}")))?;
            let new_digest = writer.close().await
                .map_err(|e| crate::Error::Store(format!("closing rewritten blob: {e}")))?;

            Ok((Node::File {
                digest: new_digest,
                size: *size,
                executable: *executable,
            }, true))
        }
        Node::Directory { digest, size } => {
            let dir = directory_service
                .get(digest)
                .await
                .map_err(|e| crate::Error::Store(format!("directory read for rewrite: {e}")))?;
            let dir = dir.ok_or_else(|| crate::Error::Store(
                format!("directory {digest} not found for rewrite")
            ))?;

            let mut any_found = false;
            let mut new_dir = snix_castore::Directory::new();

            for (name, child_node) in dir.nodes() {
                let (new_child, found) = Box::pin(
                    rewrite_node(child_node, old_bytes, new_bytes, blob_service, directory_service)
                ).await?;
                if found {
                    any_found = true;
                }
                new_dir.add(name.clone(), new_child)
                    .map_err(|e| crate::Error::Store(format!("rebuilding directory: {e}")))?;
            }

            if !any_found {
                return Ok((node.clone(), false));
            }

            let new_digest = new_dir.digest();
            let new_size = new_dir.size();
            directory_service.put(new_dir).await
                .map_err(|e| crate::Error::Store(format!("storing rewritten directory: {e}")))?;

            Ok((Node::Directory {
                digest: new_digest,
                size: new_size,
            }, true))
        }
        Node::Symlink { .. } => {
            // Symlinks don't contain store paths in their targets
            // (and if they did, they'd need different handling)
            Ok((node.clone(), false))
        }
    }
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
        let count = result
            .windows(zeros.len())
            .filter(|w| *w == zeros.as_slice())
            .count();
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
}
