use std::sync::Arc;

use digest::Digest;
use nix_compat::nixhash::HashAlgo;
use nix_compat::nixhash::NixHash;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::nar::write_nar;
use snix_store::utils::AsyncIoBridge;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::Error;

pub const MAX_REWRITE_DEPTH: u32 = 256;
const MAX_REWRITE_NODES: usize = 1_000_000;
const HASH_BUFFER_BYTES: usize = 65_536;
const MAX_HASHED_BLOB_BYTES: u64 = 17_179_869_184;
const MAX_BLOB_CHUNKS: u64 = 262_144;

struct WorkItem {
    node: Node,
    name: Option<snix_castore::PathComponent>,
    parent_index: Option<u32>,
}

pub async fn read_file_node(blob_service: &dyn BlobService, node: &Node, max_bytes: u64) -> Result<Vec<u8>, Error> {
    let (digest, size_bytes) = match node {
        Node::File { digest, size, .. } => (*digest, *size),
        other => return Err(Error::Store(format!("blob read requires a file node: {other:?}"))),
    };
    assert!(!digest.as_slice().is_empty(), "blob digest must not be empty");
    assert_eq!(digest.as_slice().len(), blake3::OUT_LEN, "blob digest length must match BLAKE3");
    if size_bytes > max_bytes {
        return Err(Error::Store(format!("blob size {size_bytes} exceeds read limit {max_bytes}")));
    }
    let capacity_bytes = usize::try_from(size_bytes)
        .map_err(|_| Error::Store(format!("blob size {size_bytes} exceeds platform address space")))?;
    let mut reader = blob_service
        .open_read(&digest)
        .await
        .map_err(|error| Error::Store(format!("opening blob: {error}")))?
        .ok_or_else(|| Error::Store(format!("blob {digest} is missing")))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    // r[impl mantle.io_fault.fixtures] A test fault is classified like a
    // failed blob read; production executes only the original I/O expression.
    #[cfg(test)]
    let read_result = async {
        let size = fault_injection::fallible!(reader.read_to_end(&mut bytes).await);
        Ok::<_, std::io::Error>(size)
    }
    .await;
    #[cfg(not(test))]
    let read_result = reader.read_to_end(&mut bytes).await;
    read_result.map_err(|error| Error::Store(format!("reading blob: {error}")))?;
    Ok(bytes)
}

pub async fn rewrite_node(
    node: &Node,
    old_bytes: &[u8],
    new_bytes: &[u8],
    blob_service: &dyn BlobService,
    directory_service: &dyn DirectoryService,
) -> Result<(Node, bool), Error> {
    assert_eq!(old_bytes.len(), new_bytes.len(), "old and new byte lengths must match");
    assert!(!old_bytes.is_empty(), "replacement needle must not be empty");

    let worklist = flatten_tree(node, directory_service).await?;
    rewrite_leaf_to_root(&worklist, old_bytes, new_bytes, blob_service, directory_service).await
}

async fn flatten_tree(root: &Node, directory_service: &dyn DirectoryService) -> Result<Vec<WorkItem>, Error> {
    assert!(
        !matches!(root, Node::Symlink { target, .. } if target.as_ref().is_empty()),
        "root symlink target must not be empty"
    );
    const INITIAL_WORKLIST_CAPACITY: usize = 64;
    let mut worklist = Vec::with_capacity(INITIAL_WORKLIST_CAPACITY);
    let mut expand_stack = Vec::with_capacity(INITIAL_WORKLIST_CAPACITY);
    expand_stack.push((root.clone(), None, None, 0_u32));
    assert!(worklist.is_empty());
    assert_eq!(expand_stack.len(), 1);

    while let Some((current, name, parent_index, depth)) = expand_stack.pop() {
        if depth >= MAX_REWRITE_DEPTH {
            return Err(Error::Store(format!("rewrite depth limit ({MAX_REWRITE_DEPTH}) exceeded")));
        }
        if worklist.len() >= MAX_REWRITE_NODES {
            return Err(Error::Store(format!("rewrite node count limit ({MAX_REWRITE_NODES}) exceeded")));
        }
        let current_index =
            u32::try_from(worklist.len()).map_err(|_| Error::Store("rewrite worklist index overflow".to_string()))?;
        worklist.push(WorkItem {
            node: current.clone(),
            name,
            parent_index,
        });

        if let Node::Directory { digest, .. } = current {
            let directory = directory_service
                .get(&digest)
                .await
                .map_err(|error| Error::Store(format!("directory read for rewrite: {error}")))?
                .ok_or_else(|| Error::Store(format!("directory {digest} not found for rewrite")))?;
            let children = directory.nodes().collect::<Vec<_>>();
            for (child_name, child_node) in children.into_iter().rev() {
                expand_stack.push((
                    child_node.clone(),
                    Some(child_name.clone()),
                    Some(current_index),
                    depth.saturating_add(1),
                ));
            }
        }
    }
    Ok(worklist)
}

async fn rewrite_leaf_to_root(
    worklist: &[WorkItem],
    old_bytes: &[u8],
    new_bytes: &[u8],
    blob_service: &dyn BlobService,
    directory_service: &dyn DirectoryService,
) -> Result<(Node, bool), Error> {
    assert!(!worklist.is_empty(), "worklist must not be empty");
    assert_eq!(old_bytes.len(), new_bytes.len(), "replacement byte lengths must match");
    let mut results = vec![None; worklist.len()];

    for item_index in (0..worklist.len()).rev() {
        let item = &worklist[item_index];
        let (new_node, was_rewritten) = match &item.node {
            Node::File {
                digest,
                size,
                executable,
            } => rewrite_file_node(digest, *size, *executable, old_bytes, new_bytes, blob_service).await?,
            Node::Symlink { .. } => (item.node.clone(), false),
            Node::Directory { .. } => {
                rebuild_directory(item_index, item, worklist, &mut results, directory_service).await?
            }
        };
        results[item_index] = Some((new_node, was_rewritten));
    }

    results[0].take().ok_or_else(|| Error::Store("rewrite produced no root result".to_string()))
}

async fn rebuild_directory(
    item_index: usize,
    item: &WorkItem,
    worklist: &[WorkItem],
    results: &mut [Option<(Node, bool)>],
    directory_service: &dyn DirectoryService,
) -> Result<(Node, bool), Error> {
    assert!(item_index < worklist.len(), "directory item must be in the rewrite worklist");
    assert_eq!(results.len(), worklist.len(), "rewrite results must align with the worklist");
    assert!(matches!(item.node, Node::Directory { .. }), "rebuild requires a directory node");
    let parent_index =
        u32::try_from(item_index).map_err(|_| Error::Store("rewrite directory parent index overflow".to_string()))?;
    let mut was_rewritten = false;
    let mut new_directory = snix_castore::Directory::new();
    for (child_index, child_item) in worklist.iter().enumerate() {
        if child_item.parent_index != Some(parent_index) {
            continue;
        }
        let (child_node, child_was_rewritten) = results[child_index].take().ok_or_else(|| {
            Error::Store(format!("rewrite child {child_index} was not processed before parent {item_index}"))
        })?;
        was_rewritten |= child_was_rewritten;
        let child_name = child_item
            .name
            .clone()
            .ok_or_else(|| Error::Store(format!("rewrite directory child {child_index} has no name")))?;
        new_directory
            .add(child_name, child_node)
            .map_err(|error| Error::Store(format!("rebuilding directory: {error}")))?;
    }
    if !was_rewritten {
        return Ok((item.node.clone(), false));
    }
    let new_digest = new_directory.digest();
    let new_entry_count = new_directory.size();
    directory_service
        .put(new_directory)
        .await
        .map_err(|error| Error::Store(format!("storing rewritten directory: {error}")))?;
    Ok((
        Node::Directory {
            digest: new_digest,
            size: new_entry_count,
        },
        true,
    ))
}

#[allow(tigerstyle::too_many_parameters)]
async fn rewrite_file_node(
    digest: &snix_castore::B3Digest,
    size_bytes: u64,
    is_executable: bool,
    old_bytes: &[u8],
    new_bytes: &[u8],
    blob_service: &dyn BlobService,
) -> Result<(Node, bool), Error> {
    assert!(!digest.as_slice().is_empty(), "blob digest must not be empty");
    assert!(!old_bytes.is_empty(), "rewrite needle must not be empty");
    assert_eq!(old_bytes.len(), new_bytes.len(), "rewrite byte lengths must match");
    if size_bytes > MAX_HASHED_BLOB_BYTES {
        return Err(Error::Store(format!("rewrite blob size {size_bytes} exceeds limit {MAX_HASHED_BLOB_BYTES}")));
    }
    let mut reader = blob_service
        .open_read(digest)
        .await
        .map_err(|error| Error::Store(format!("blob read for rewrite: {error}")))?
        .ok_or_else(|| Error::Store(format!("blob {digest} not found for rewrite")))?;
    let capacity_bytes = usize::try_from(size_bytes)
        .map_err(|_| Error::Store(format!("blob size {size_bytes} exceeds platform address space")))?;
    let mut data = Vec::with_capacity(capacity_bytes);
    reader
        .read_to_end(&mut data)
        .await
        .map_err(|error| Error::Store(format!("reading blob for rewrite: {error}")))?;
    let (rewritten, was_found) = replace_bytes(&data, old_bytes, new_bytes);
    if !was_found {
        return Ok((
            Node::File {
                digest: *digest,
                size: size_bytes,
                executable: is_executable,
            },
            false,
        ));
    }
    let mut writer = blob_service.open_write().await;
    writer
        .write_all(&rewritten)
        .await
        .map_err(|error| Error::Store(format!("writing rewritten blob: {error}")))?;
    let new_digest = writer.close().await.map_err(|error| Error::Store(format!("closing rewritten blob: {error}")))?;
    Ok((
        Node::File {
            digest: new_digest,
            size: size_bytes,
            executable: is_executable,
        },
        true,
    ))
}

fn replace_bytes(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> (Vec<u8>, bool) {
    assert_eq!(needle.len(), replacement.len(), "replacement byte lengths must match");
    assert!(!needle.is_empty(), "replacement needle must not be empty");
    let mut result = Vec::with_capacity(haystack.len());
    let mut was_found = false;
    let mut byte_index = 0_usize;
    while byte_index < haystack.len() {
        let match_end = byte_index.saturating_add(needle.len());
        if match_end <= haystack.len() && &haystack[byte_index..match_end] == needle {
            result.extend_from_slice(replacement);
            was_found = true;
            byte_index = match_end;
            continue;
        }
        result.push(haystack[byte_index]);
        byte_index = byte_index.saturating_add(1);
    }
    (result, was_found)
}

pub async fn hash_blob(
    blob_service: &dyn BlobService,
    digest: &snix_castore::B3Digest,
    algorithm: HashAlgo,
) -> Result<NixHash, Error> {
    assert!(!digest.as_slice().is_empty(), "blob digest must not be empty");
    assert_eq!(digest.as_slice().len(), blake3::OUT_LEN, "blob digest length must match BLAKE3");
    let mut reader = blob_service
        .open_read(digest)
        .await
        .map_err(|error| Error::Store(format!("opening blob for hash: {error}")))?
        .ok_or_else(|| Error::Store("blob not found for hash".to_string()))?;
    let mut buffer = vec![0_u8; HASH_BUFFER_BYTES];
    match algorithm {
        HashAlgo::Md5 => {
            let mut hasher = md5::Md5::new();
            update_blob_hasher(&mut reader, &mut buffer, |chunk| hasher.update(chunk)).await?;
            Ok(NixHash::Md5(hasher.finalize().into()))
        }
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            update_blob_hasher(&mut reader, &mut buffer, |chunk| hasher.update(chunk)).await?;
            Ok(NixHash::Sha1(hasher.finalize().into()))
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            update_blob_hasher(&mut reader, &mut buffer, |chunk| hasher.update(chunk)).await?;
            Ok(NixHash::Sha256(hasher.finalize().into()))
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            update_blob_hasher(&mut reader, &mut buffer, |chunk| hasher.update(chunk)).await?;
            let digest_bytes: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(digest_bytes)))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            update_blob_hasher(&mut reader, &mut buffer, |chunk| {
                hasher.update(chunk);
            })
            .await?;
            let digest = blake3::Hasher::finalize(&hasher);
            Ok(NixHash::Blake3(*digest.as_bytes()))
        }
    }
}

pub async fn hash_host_path(path: &std::path::Path, algorithm: HashAlgo) -> Result<NixHash, Error> {
    assert!(!path.as_os_str().is_empty(), "host hash path must not be empty");
    assert!(path.components().next().is_some(), "host hash path must contain a component");
    let blob_service = empty_hash_blob_service();
    let directory_service = snix_castore::directoryservice::RedbDirectoryService::new_temporary(
        "store-authority-host-hash".to_string(),
        snix_castore::directoryservice::RedbDirectoryServiceConfig {
            path: None,
            cache_size: None,
            read_only: false,
        },
    )
    .map_err(|error| Error::Store(format!("creating host hash directory service: {error}")))?;
    let node = snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(
        blob_service.clone(),
        directory_service.clone(),
        path,
        None,
    )
    .await
    .map_err(|error| Error::Store(format!("ingesting host path for hash: {error}")))?;
    nar_hash(&node, algorithm, Arc::new(blob_service), Arc::new(directory_service)).await
}

pub async fn nar_hash(
    node: &Node,
    algorithm: HashAlgo,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
) -> Result<NixHash, Error> {
    assert!(
        !matches!(node, Node::Symlink { target, .. } if target.as_ref().is_empty()),
        "symlink target must not be empty"
    );
    assert!(
        matches!(node, Node::Directory { .. } | Node::File { .. } | Node::Symlink { .. }),
        "NAR hash requires a known node variant"
    );
    match algorithm {
        HashAlgo::Md5 => {
            let mut hasher = md5::Md5::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|error| Error::Store(format!("NAR hash: {error}")))?;
            Ok(NixHash::Md5(hasher.finalize().into()))
        }
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|error| Error::Store(format!("NAR hash: {error}")))?;
            Ok(NixHash::Sha1(hasher.finalize().into()))
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|error| Error::Store(format!("NAR hash: {error}")))?;
            Ok(NixHash::Sha256(hasher.finalize().into()))
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|error| Error::Store(format!("NAR hash: {error}")))?;
            let digest_bytes: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(digest_bytes)))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|error| Error::Store(format!("NAR hash: {error}")))?;
            let digest = blake3::Hasher::finalize(&hasher);
            Ok(NixHash::Blake3(*digest.as_bytes()))
        }
    }
}

#[allow(
    tigerstyle::explicit_defaults,
    reason = "MemoryBlobService has private fields and exposes Default as its only direct constructor"
)]
fn empty_hash_blob_service() -> snix_castore::blobservice::MemoryBlobService {
    snix_castore::blobservice::MemoryBlobService::default()
}

async fn update_blob_hasher(
    reader: &mut (impl tokio::io::AsyncRead + Unpin),
    buffer: &mut [u8],
    mut update: impl FnMut(&[u8]),
) -> Result<(), Error> {
    for _ in 0..MAX_BLOB_CHUNKS {
        let read_bytes =
            reader.read(buffer).await.map_err(|error| Error::Store(format!("reading blob for hash: {error}")))?;
        if read_bytes == 0 {
            return Ok(());
        }
        update(&buffer[..read_bytes]);
    }
    Err(Error::Store(format!("blob exceeds hash limit of {MAX_HASHED_BLOB_BYTES} bytes")))
}

#[cfg(test)]
mod tests {
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use tokio::io::AsyncWriteExt;

    use super::*;

    const TEST_READ_LIMIT_BYTES: u64 = 1024;

    fn directory_service() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("store-build-io-test".to_string(), RedbDirectoryServiceConfig::default())
            .expect("temporary directory service")
    }

    async fn insert_blob(blob_service: &MemoryBlobService, bytes: &[u8]) -> Node {
        let mut writer = blob_service.open_write().await;
        writer.write_all(bytes).await.expect("write test blob");
        let digest = writer.close().await.expect("close test blob");
        Node::File {
            digest,
            size: u64::try_from(bytes.len()).expect("test blob length"),
            executable: false,
        }
    }

    #[tokio::test]
    async fn rewrite_and_nar_hash_preserve_named_operation_behavior() {
        let blob_service = MemoryBlobService::default();
        let directory_service = directory_service();
        let original = insert_blob(&blob_service, b"prefix-old-suffix").await;

        let (rewritten, was_rewritten) = rewrite_node(&original, b"old", b"new", &blob_service, &directory_service)
            .await
            .expect("rewrite file node");
        let bytes =
            read_file_node(&blob_service, &rewritten, TEST_READ_LIMIT_BYTES).await.expect("read rewritten node");
        let hash = nar_hash(&rewritten, HashAlgo::Sha256, Arc::new(blob_service), Arc::new(directory_service))
            .await
            .expect("hash rewritten NAR");

        assert!(was_rewritten);
        assert_eq!(bytes, b"prefix-new-suffix");
        assert_eq!(hash.digest_as_bytes().len(), 32);
    }

    #[tokio::test]
    async fn read_file_node_rejects_non_file_and_over_limit_inputs() {
        let blob_service = MemoryBlobService::default();
        let file = insert_blob(&blob_service, b"bounded").await;
        let symlink = Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from(b"target".as_slice()).expect("valid symlink target"),
        };

        let non_file_error = read_file_node(&blob_service, &symlink, TEST_READ_LIMIT_BYTES)
            .await
            .expect_err("symlink read must fail");
        let over_limit_error = read_file_node(&blob_service, &file, 1).await.expect_err("over-limit read must fail");

        assert!(non_file_error.to_string().contains("requires a file node"));
        assert!(over_limit_error.to_string().contains("exceeds read limit"));
    }

    #[tokio::test]
    async fn host_path_hash_is_deterministic_and_missing_paths_fail_closed() {
        let temporary = tempfile::tempdir().expect("temporary host hash directory");
        let file = temporary.path().join("input.txt");
        std::fs::write(&file, b"deterministic host hash").expect("write host hash fixture");

        let first = hash_host_path(&file, HashAlgo::Blake3).await.expect("first host hash");
        let second = hash_host_path(&file, HashAlgo::Blake3).await.expect("second host hash");
        let missing = hash_host_path(&temporary.path().join("missing"), HashAlgo::Blake3).await;

        assert_eq!(first, second);
        assert!(missing.is_err());
    }

    #[test]
    fn equal_length_replacement_preserves_binary_shape_and_occurrences() {
        const EXPECTED_MATCH_COUNT: usize = 2;
        const BINARY_PREFIX: [u8; 4] = [0xff, 0xfe, 0x00, 0x01];
        const BINARY_SUFFIX: [u8; 2] = [0x80, 0x90];
        let old = b"/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-lib";
        let new = b"/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-lib";
        let mut input = BINARY_PREFIX.to_vec();
        input.extend_from_slice(old);
        input.push(b':');
        input.extend_from_slice(old);
        input.extend_from_slice(&BINARY_SUFFIX);

        let (rewritten, was_rewritten) = replace_bytes(&input, old, new);
        let match_count = rewritten.windows(new.len()).filter(|window| *window == new).count();
        let (unchanged, absent_was_rewritten) = replace_bytes(b"nothing to replace", old, new);

        assert!(was_rewritten);
        assert_eq!(rewritten.len(), input.len());
        assert_eq!(match_count, EXPECTED_MATCH_COUNT);
        assert_eq!(&rewritten[..BINARY_PREFIX.len()], &BINARY_PREFIX);
        assert_eq!(&rewritten[rewritten.len() - BINARY_SUFFIX.len()..], &BINARY_SUFFIX);
        assert!(!absent_was_rewritten);
        assert_eq!(unchanged, b"nothing to replace");
    }

    #[test]
    #[should_panic(expected = "replacement byte lengths must match")]
    fn unequal_length_replacement_is_rejected() {
        let _ = replace_bytes(b"data", b"old", b"longer");
    }

    #[tokio::test]
    async fn blob_and_nar_hashes_cover_all_supported_algorithms() {
        const MD5_BYTES: usize = 16;
        const SHA1_BYTES: usize = 20;
        const SHA256_BYTES: usize = 32;
        const SHA512_BYTES: usize = 64;
        const BLAKE3_BYTES: usize = 32;
        let algorithms = [
            (HashAlgo::Md5, MD5_BYTES),
            (HashAlgo::Sha1, SHA1_BYTES),
            (HashAlgo::Sha256, SHA256_BYTES),
            (HashAlgo::Sha512, SHA512_BYTES),
            (HashAlgo::Blake3, BLAKE3_BYTES),
        ];
        let blob_service = MemoryBlobService::default();
        let directory_service = directory_service();
        let node = insert_blob(&blob_service, b"all hash algorithms").await;
        let digest = match node {
            Node::File { digest, .. } => digest,
            _ => unreachable!("test helper always creates a file"),
        };

        for (algorithm, expected_bytes) in algorithms {
            let blob_hash =
                hash_blob(&blob_service, &digest, algorithm).await.expect("hash blob with supported algorithm");
            let first_nar_hash =
                nar_hash(&node, algorithm, Arc::new(blob_service.clone()), Arc::new(directory_service.clone()))
                    .await
                    .expect("hash NAR with supported algorithm");
            let second_nar_hash =
                nar_hash(&node, algorithm, Arc::new(blob_service.clone()), Arc::new(directory_service.clone()))
                    .await
                    .expect("repeat NAR hash");

            assert_eq!(blob_hash.digest_as_bytes().len(), expected_bytes);
            assert_eq!(first_nar_hash.digest_as_bytes().len(), expected_bytes);
            assert_eq!(first_nar_hash, second_nar_hash);
        }
    }

    #[tokio::test]
    async fn missing_blob_hash_fails_closed() {
        let blob_service = MemoryBlobService::default();
        let missing_digest = snix_castore::B3Digest::from(blake3::hash(b"missing").as_bytes());

        let error = hash_blob(&blob_service, &missing_digest, HashAlgo::Sha256)
            .await
            .expect_err("missing blob hash must fail");

        let rewrite_depth = std::hint::black_box(MAX_REWRITE_DEPTH);
        assert!(error.to_string().contains("blob not found for hash"));
        assert!(rewrite_depth > 0);
    }
}
