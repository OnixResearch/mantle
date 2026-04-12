//! Hashing utilities: NAR stream hashing and blob hashing for all
//! supported algorithms.

use digest::Digest;
use nix_compat::nixhash::NixHash;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::nar::write_nar;
use snix_store::utils::AsyncIoBridge;
use tokio::io::AsyncReadExt;

use crate::Error;

/// Serialize a node to NAR and hash the byte stream with the given algorithm.
pub(crate) async fn nar_hash(
    node: &Node,
    algo: nix_compat::nixhash::HashAlgo,
    blob_service: impl BlobService + Send,
    directory_service: impl DirectoryService + Send,
) -> Result<NixHash, Error> {
    use nix_compat::nixhash::HashAlgo;

    match algo {
        HashAlgo::Md5 => {
            let mut hasher = md5::Md5::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 16] = hasher.finalize().into();
            Ok(NixHash::Md5(hash))
        }
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 20] = hasher.finalize().into();
            Ok(NixHash::Sha1(hash))
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 32] = hasher.finalize().into();
            Ok(NixHash::Sha256(hash))
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(hash)))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|e| Error::NarCalculation(e.to_string()))?;
            let hash = blake3::Hasher::finalize(&hasher);
            Ok(NixHash::Blake3(*hash.as_bytes()))
        }
    }
}

/// Read a blob from the blob service and hash it with the given algorithm.
pub(crate) async fn hash_blob(
    blob_service: &impl BlobService,
    digest: &snix_castore::B3Digest,
    algo: nix_compat::nixhash::HashAlgo,
) -> Result<NixHash, Error> {
    use nix_compat::nixhash::HashAlgo;

    let mut reader = blob_service
        .open_read(digest)
        .await
        .map_err(|e| Error::Store(format!("blob read for FOD verification: {e}")))?
        .ok_or_else(|| Error::Store(format!("blob {digest} not found for FOD verification")))?;

    let mut buf = vec![0u8; 64 * 1024];
    match algo {
        HashAlgo::Md5 => {
            let mut hasher = md5::Md5::new();
            update_blob_hasher(&mut reader, &mut buf, |chunk| {
                hasher.update(chunk);
            }).await?;
            Ok(NixHash::Md5(hasher.finalize().into()))
        }
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            update_blob_hasher(&mut reader, &mut buf, |chunk| {
                hasher.update(chunk);
            }).await?;
            Ok(NixHash::Sha1(hasher.finalize().into()))
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            update_blob_hasher(&mut reader, &mut buf, |chunk| {
                hasher.update(chunk);
            }).await?;
            Ok(NixHash::Sha256(hasher.finalize().into()))
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            update_blob_hasher(&mut reader, &mut buf, |chunk| {
                hasher.update(chunk);
            }).await?;
            let hash: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(hash)))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            update_blob_hasher(&mut reader, &mut buf, |chunk| {
                hasher.update(chunk);
            }).await?;
            let hash = blake3::Hasher::finalize(&hasher);
            Ok(NixHash::Blake3(*hash.as_bytes()))
        }
    }
}

/// Stream a blob into a caller-provided hasher update closure.
async fn update_blob_hasher(
    reader: &mut (impl tokio::io::AsyncRead + Unpin),
    buf: &mut [u8],
    mut update: impl FnMut(&[u8]),
) -> Result<(), Error> {
    loop {
        let n = reader.read(buf).await.map_err(|e| Error::Store(format!("blob read: {e}")))?;
        if n == 0 {
            return Ok(());
        }
        update(&buf[..n]);
    }
}

#[cfg(test)]
mod tests {
    use nix_compat::nixhash::HashAlgo;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_store::nar::NarCalculationService;
    use snix_store::nar::SimpleRenderer;
    use tokio::io::AsyncWriteExt;

    use super::*;

    async fn insert_blob(bs: &MemoryBlobService, data: &[u8]) -> (snix_castore::B3Digest, Node) {
        let mut writer = bs.open_write().await;
        writer.write_all(data).await.unwrap();
        let digest = writer.close().await.unwrap();
        let node = Node::File {
            digest: digest.clone(),
            size: data.len() as u64,
            executable: false,
        };
        (digest, node)
    }

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
    }

    // ── nar_hash: cross-check sha256 against SimpleRenderer ─────

    #[tokio::test]
    async fn nar_hash_sha256_matches_calculate() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"cross-check content";
        let (_, node) = insert_blob(&bs, data).await;

        let renderer = SimpleRenderer::new(bs.clone(), ds.clone());
        let (_size, sha256_from_calc) = renderer.calculate_nar(&node).await.unwrap();

        let hash = nar_hash(&node, HashAlgo::Sha256, bs.clone(), ds.clone()).await.unwrap();

        assert_eq!(&sha256_from_calc[..], hash.digest_as_bytes(), "nar_hash(Sha256) should match calculate_nar");
    }

    // ── nar_hash: all algorithms on a file node ─────────────────

    #[tokio::test]
    async fn nar_hash_md5_produces_16_bytes() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"md5 test").await;

        let hash = nar_hash(&node, HashAlgo::Md5, bs, ds).await.unwrap();
        assert_eq!(hash.digest_as_bytes().len(), 16);
        assert!(matches!(hash, NixHash::Md5(_)));
    }

    #[tokio::test]
    async fn nar_hash_sha1_produces_20_bytes() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"sha1 test").await;

        let hash = nar_hash(&node, HashAlgo::Sha1, bs, ds).await.unwrap();
        assert_eq!(hash.digest_as_bytes().len(), 20);
        assert!(matches!(hash, NixHash::Sha1(_)));
    }

    #[tokio::test]
    async fn nar_hash_sha512_produces_64_bytes() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"sha512 test").await;

        let hash = nar_hash(&node, HashAlgo::Sha512, bs, ds).await.unwrap();
        assert_eq!(hash.digest_as_bytes().len(), 64);
        assert!(matches!(hash, NixHash::Sha512(_)));
    }

    #[tokio::test]
    async fn nar_hash_blake3_produces_32_bytes() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"blake3 test").await;

        let hash = nar_hash(&node, HashAlgo::Blake3, bs, ds).await.unwrap();
        assert_eq!(hash.digest_as_bytes().len(), 32);
        assert!(matches!(hash, NixHash::Blake3(_)));
    }

    // ── nar_hash: determinism ───────────────────────────────────

    #[tokio::test]
    async fn nar_hash_deterministic_across_calls() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"determinism check").await;

        let h1 = nar_hash(&node, HashAlgo::Sha256, bs.clone(), ds.clone()).await.unwrap();
        let h2 = nar_hash(&node, HashAlgo::Sha256, bs, ds).await.unwrap();

        assert_eq!(h1.digest_as_bytes(), h2.digest_as_bytes(), "same input must produce same hash");
    }

    // ── nar_hash: different content produces different hashes ───

    #[tokio::test]
    async fn nar_hash_different_content_different_hash() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node_a) = insert_blob(&bs, b"aaa").await;
        let (_, node_b) = insert_blob(&bs, b"bbb").await;

        let ha = nar_hash(&node_a, HashAlgo::Sha256, bs.clone(), ds.clone()).await.unwrap();
        let hb = nar_hash(&node_b, HashAlgo::Sha256, bs, ds).await.unwrap();

        assert_ne!(ha.digest_as_bytes(), hb.digest_as_bytes(), "different content should hash differently");
    }

    // ── nar_hash: directory node ────────────────────────────────

    #[tokio::test]
    async fn nar_hash_directory_sha256() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (file_digest, _) = insert_blob(&bs, b"dir-file").await;

        let mut dir = snix_castore::Directory::new();
        dir.add("file.txt".try_into().unwrap(), Node::File {
            digest: file_digest,
            size: 8,
            executable: false,
        })
        .unwrap();
        let dir_digest = dir.digest();
        let dir_size = dir.size();
        ds.put(dir).await.unwrap();

        let dir_node = Node::Directory {
            digest: dir_digest,
            size: dir_size,
        };

        // Cross-check with SimpleRenderer
        let renderer = SimpleRenderer::new(bs.clone(), ds.clone());
        let (_size, expected) = renderer.calculate_nar(&dir_node).await.unwrap();

        let hash = nar_hash(&dir_node, HashAlgo::Sha256, bs, ds).await.unwrap();
        assert_eq!(&expected[..], hash.digest_as_bytes());
    }

    // ── nar_hash: empty file ────────────────────────────────────

    #[tokio::test]
    async fn nar_hash_empty_file() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (_, node) = insert_blob(&bs, b"").await;

        let hash = nar_hash(&node, HashAlgo::Sha256, bs, ds).await.unwrap();
        assert_eq!(hash.digest_as_bytes().len(), 32);
        // An empty file still has a non-zero NAR hash (NAR header bytes).
        assert_ne!(hash.digest_as_bytes(), &[0u8; 32]);
    }

    // ── hash_blob: all algorithms ───────────────────────────────

    #[tokio::test]
    async fn hash_blob_sha256() {
        let bs = MemoryBlobService::default();
        let data = b"blob sha256";
        let (digest, _) = insert_blob(&bs, data).await;

        let hash = hash_blob(&bs, &digest, HashAlgo::Sha256).await.unwrap();

        let expected: [u8; 32] = {
            use digest::Digest;
            sha2::Sha256::digest(data).into()
        };
        assert_eq!(hash.digest_as_bytes(), &expected);
    }

    #[tokio::test]
    async fn hash_blob_sha1() {
        let bs = MemoryBlobService::default();
        let data = b"blob sha1";
        let (digest, _) = insert_blob(&bs, data).await;

        let hash = hash_blob(&bs, &digest, HashAlgo::Sha1).await.unwrap();

        let expected: [u8; 20] = {
            use digest::Digest;
            sha1::Sha1::digest(data).into()
        };
        assert_eq!(hash.digest_as_bytes(), &expected);
    }

    #[tokio::test]
    async fn hash_blob_md5() {
        let bs = MemoryBlobService::default();
        let data = b"blob md5";
        let (digest, _) = insert_blob(&bs, data).await;

        let hash = hash_blob(&bs, &digest, HashAlgo::Md5).await.unwrap();

        let expected: [u8; 16] = {
            use digest::Digest;
            md5::Md5::digest(data).into()
        };
        assert_eq!(hash.digest_as_bytes(), &expected);
    }

    #[tokio::test]
    async fn hash_blob_sha512() {
        let bs = MemoryBlobService::default();
        let data = b"blob sha512";
        let (digest, _) = insert_blob(&bs, data).await;

        let hash = hash_blob(&bs, &digest, HashAlgo::Sha512).await.unwrap();

        let expected: [u8; 64] = {
            use digest::Digest;
            sha2::Sha512::digest(data).into()
        };
        assert_eq!(hash.digest_as_bytes(), &expected);
    }

    #[tokio::test]
    async fn hash_blob_blake3() {
        let bs = MemoryBlobService::default();
        let data = b"blob blake3";
        let (digest, _) = insert_blob(&bs, data).await;

        let hash = hash_blob(&bs, &digest, HashAlgo::Blake3).await.unwrap();

        let expected = blake3::hash(data);
        assert_eq!(hash.digest_as_bytes(), expected.as_bytes());
    }

    // ── hash_blob: empty blob ───────────────────────────────────

    #[tokio::test]
    async fn hash_blob_empty() {
        let bs = MemoryBlobService::default();
        let (digest, _) = insert_blob(&bs, b"").await;

        let hash = hash_blob(&bs, &digest, HashAlgo::Sha256).await.unwrap();

        let expected: [u8; 32] = {
            use digest::Digest;
            sha2::Sha256::digest(b"").into()
        };
        assert_eq!(hash.digest_as_bytes(), &expected);
    }

    // ── hash_blob: large blob (multi-buffer) ────────────────────

    #[tokio::test]
    async fn hash_blob_large() {
        let bs = MemoryBlobService::default();
        let data = vec![0xabu8; 256 * 1024]; // 256KB, exceeds 64KB buffer
        let (digest, _) = insert_blob(&bs, &data).await;

        let hash = hash_blob(&bs, &digest, HashAlgo::Sha256).await.unwrap();

        let expected: [u8; 32] = {
            use digest::Digest;
            sha2::Sha256::digest(&data).into()
        };
        assert_eq!(hash.digest_as_bytes(), &expected);
    }

    // ── hash_blob: missing blob ─────────────────────────────────

    #[tokio::test]
    async fn hash_blob_missing_returns_error() {
        let bs = MemoryBlobService::default();
        let fake = snix_castore::B3Digest::from(&[0xffu8; 32]);

        let err = hash_blob(&bs, &fake, HashAlgo::Sha256).await.unwrap_err();
        assert!(matches!(err, Error::Store(_)), "missing blob should be Store error: {err}");
    }
}
