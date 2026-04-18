//! Fixed-output derivation hash verification.

use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::NixHash;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;

use crate::Error;
use crate::hash::hash_blob;
use crate::hash::nar_hash;

/// Verify that a fixed-output derivation produced the expected hash.
///
/// Flat mode: hash the raw file bytes with the declared algorithm.
/// NAR mode: hash the NAR serialization with the declared algorithm.
/// Text mode: equivalent to NAR sha256 for verification purposes.
pub(crate) async fn verify_fod_hash(
    drv_name: &str,
    _output_name: &str,
    expected_ca: &CAHash,
    _nar_size: u64,
    nar_sha256: &[u8; 32],
    node: &Node,
    blob_service: &(impl BlobService + Clone),
    directory_service: &(impl DirectoryService + Clone),
) -> Result<(), Error> {
    // Tiger Style: assert preconditions.
    debug_assert!(!drv_name.is_empty(), "drv_name must not be empty for FOD verification");
    debug_assert!(!_output_name.is_empty(), "output_name must not be empty for FOD verification");

    match expected_ca {
        CAHash::Flat(expected_hash) => {
            let digest = match node {
                Node::File { digest, .. } => digest,
                _ => {
                    return Err(Error::FodFlatNotFile {
                        name: drv_name.to_string(),
                    });
                }
            };

            let actual = hash_blob(blob_service, digest, expected_hash.algo()).await?;
            if actual.digest_as_bytes() != expected_hash.digest_as_bytes() {
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(expected_hash),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual),
                });
            }
        }
        CAHash::Nar(NixHash::Sha256(expected_digest)) => {
            if nar_sha256 != expected_digest {
                let expected_h = NixHash::Sha256(*expected_digest);
                let actual_h = NixHash::Sha256(*nar_sha256);
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(&expected_h),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual_h),
                });
            }
        }
        CAHash::Nar(expected_hash) => {
            let actual = nar_hash(node, expected_hash.algo(), blob_service.clone(), directory_service.clone()).await?;
            if actual.digest_as_bytes() != expected_hash.digest_as_bytes() {
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(expected_hash),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual),
                });
            }
        }
        CAHash::Text(expected_digest) => {
            if nar_sha256 != expected_digest {
                let expected_h = NixHash::Sha256(*expected_digest);
                let actual_h = NixHash::Sha256(*nar_sha256);
                return Err(Error::FodHashMismatch {
                    name: drv_name.to_string(),
                    expected_sri: crate::fetcher::nix_hash_to_sri(&expected_h),
                    actual_sri: crate::fetcher::nix_hash_to_sri(&actual_h),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::HashAlgo;
    use nix_compat::nixhash::NixHash;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use tokio::io::AsyncWriteExt;

    use super::*;

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

    fn expected_hash(data: &[u8], algo: HashAlgo) -> NixHash {
        use digest::Digest;
        match algo {
            HashAlgo::Md5 => {
                let h: [u8; 16] = md5::Md5::digest(data).into();
                NixHash::Md5(h)
            }
            HashAlgo::Sha1 => {
                let h: [u8; 20] = sha1::Sha1::digest(data).into();
                NixHash::Sha1(h)
            }
            HashAlgo::Sha256 => {
                let h: [u8; 32] = sha2::Sha256::digest(data).into();
                NixHash::Sha256(h)
            }
            HashAlgo::Sha512 => {
                let h: [u8; 64] = sha2::Sha512::digest(data).into();
                NixHash::Sha512(Box::new(h))
            }
            HashAlgo::Blake3 => {
                let h = blake3::hash(data);
                NixHash::Blake3(*h.as_bytes())
            }
        }
    }

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
    }

    fn dummy_b3() -> snix_castore::B3Digest {
        snix_castore::B3Digest::from(&[0u8; 32])
    }

    #[tokio::test]
    async fn flat_sha256_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"hello world";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha256);
        let ca = CAHash::Flat(hash);
        let nar_sha256 = [0u8; 32]; // unused for flat

        verify_fod_hash("test-drv", "out", &ca, 0, &nar_sha256, &node, &bs, &ds)
            .await
            .expect("matching flat sha256 should pass");
    }

    #[tokio::test]
    async fn flat_sha256_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"hello world";
        let (_, node) = insert_blob(&bs, data).await;
        let wrong = NixHash::Sha256([0xab; 32]);
        let ca = CAHash::Flat(wrong);

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds).await.unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn flat_sha1_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"sha1 test content";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha1);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching flat sha1 should pass");
    }

    #[tokio::test]
    async fn flat_sha512_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"sha512 test content";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha512);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching flat sha512 should pass");
    }

    #[tokio::test]
    async fn flat_md5_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"md5 test content";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Md5);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching flat md5 should pass");
    }

    #[tokio::test]
    async fn flat_rejects_directory_node() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };
        let hash = NixHash::Sha256([0; 32]);
        let ca = CAHash::Flat(hash);

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds).await.unwrap_err();
        assert!(matches!(err, Error::FodFlatNotFile { .. }));
    }

    #[tokio::test]
    async fn flat_rejects_symlink_node() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let node = Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
        };
        let hash = NixHash::Sha256([0; 32]);
        let ca = CAHash::Flat(hash);

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds).await.unwrap_err();
        assert!(matches!(err, Error::FodFlatNotFile { .. }));
    }

    #[tokio::test]
    async fn nar_sha256_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let expected = [0x42u8; 32];
        let ca = CAHash::Nar(NixHash::Sha256(expected));
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };

        verify_fod_hash("test-drv", "out", &ca, 100, &expected, &node, &bs, &ds)
            .await
            .expect("matching NAR sha256 should pass");
    }

    #[tokio::test]
    async fn nar_sha256_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let expected = [0x42u8; 32];
        let actual = [0x00u8; 32];
        let ca = CAHash::Nar(NixHash::Sha256(expected));
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };

        let err = verify_fod_hash("test-drv", "out", &ca, 100, &actual, &node, &bs, &ds).await.unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn text_sha256_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let expected = [0x42u8; 32];
        let ca = CAHash::Text(expected);
        let node = Node::Directory {
            digest: dummy_b3(),
            size: 0,
        };

        verify_fod_hash("test-drv", "out", &ca, 100, &expected, &node, &bs, &ds)
            .await
            .expect("matching text sha256 should pass");
    }

    #[tokio::test]
    async fn flat_empty_blob() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"";
        let (_, node) = insert_blob(&bs, data).await;
        let hash = expected_hash(data, HashAlgo::Sha256);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("empty blob flat sha256 should pass");
    }

    #[tokio::test]
    async fn flat_large_blob() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        // 256KB — larger than the 64KB read buffer
        let data = vec![0xffu8; 256 * 1024];
        let (_, node) = insert_blob(&bs, &data).await;
        let hash = expected_hash(&data, HashAlgo::Sha256);
        let ca = CAHash::Flat(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("large blob should hash correctly across buffer boundaries");
    }

    // --- NAR non-sha256 tests ---

    #[tokio::test]
    async fn nar_sha1_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha1 test";
        let (_, node) = insert_blob(&bs, data).await;

        let hash = nar_hash(&node, HashAlgo::Sha1, bs.clone(), ds.clone()).await.unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching NAR sha1 should pass");
    }

    #[tokio::test]
    async fn nar_sha1_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha1 test";
        let (_, node) = insert_blob(&bs, data).await;

        let ca = CAHash::Nar(NixHash::Sha1([0xaa; 20]));

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds).await.unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn nar_md5_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar md5 test";
        let (_, node) = insert_blob(&bs, data).await;

        let hash = nar_hash(&node, HashAlgo::Md5, bs.clone(), ds.clone()).await.unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching NAR md5 should pass");
    }

    #[tokio::test]
    async fn nar_md5_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar md5 test";
        let (_, node) = insert_blob(&bs, data).await;

        let ca = CAHash::Nar(NixHash::Md5([0xbb; 16]));

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds).await.unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn nar_sha512_match() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha512 test";
        let (_, node) = insert_blob(&bs, data).await;

        let hash = nar_hash(&node, HashAlgo::Sha512, bs.clone(), ds.clone()).await.unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds)
            .await
            .expect("matching NAR sha512 should pass");
    }

    #[tokio::test]
    async fn nar_sha512_mismatch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"nar sha512 test";
        let (_, node) = insert_blob(&bs, data).await;

        let ca = CAHash::Nar(NixHash::Sha512(Box::new([0xcc; 64])));

        let err = verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &node, &bs, &ds).await.unwrap_err();
        assert!(matches!(err, Error::FodHashMismatch { .. }));
    }

    #[tokio::test]
    async fn nar_sha1_directory_node() {
        use snix_castore::Directory;
        use snix_castore::directoryservice::DirectoryService;

        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (digest_a, _) = insert_blob(&bs, b"file-a-content").await;
        let (digest_b, _) = insert_blob(&bs, b"file-b-content").await;

        let mut dir = Directory::new();
        dir.add("a.txt".try_into().unwrap(), Node::File {
            digest: digest_a,
            size: 14,
            executable: false,
        })
        .unwrap();
        dir.add("b.txt".try_into().unwrap(), Node::File {
            digest: digest_b,
            size: 14,
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

        let hash = nar_hash(&dir_node, HashAlgo::Sha1, bs.clone(), ds.clone()).await.unwrap();
        let ca = CAHash::Nar(hash);

        verify_fod_hash("test-drv", "out", &ca, 0, &[0; 32], &dir_node, &bs, &ds)
            .await
            .expect("matching NAR sha1 on directory should pass");
    }
}
