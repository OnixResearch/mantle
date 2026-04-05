//! Hashing utilities: NAR stream hashing and blob hashing for all
//! supported algorithms.

use digest::Digest;
use nix_compat::nixhash::NixHash;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::Node;
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
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 16] = hasher.finalize().into();
            Ok(NixHash::Md5(hash))
        }
        HashAlgo::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 20] = hasher.finalize().into();
            Ok(NixHash::Sha1(hash))
        }
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 32] = hasher.finalize().into();
            Ok(NixHash::Sha256(hash))
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(hash)))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = reader.read(&mut buf).await
                    .map_err(|e| Error::Store(format!("blob read: {e}")))?;
                if n == 0 { break; }
                hasher.update(&buf[..n]);
            }
            let hash = blake3::Hasher::finalize(&hasher);
            Ok(NixHash::Blake3(*hash.as_bytes()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix_compat::nixhash::HashAlgo;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
    use snix_store::nar::{NarCalculationService, SimpleRenderer};
    use tokio::io::AsyncWriteExt;

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
        RedbDirectoryService::new_temporary(
            "test".to_string(),
            RedbDirectoryServiceConfig::default(),
        ).unwrap()
    }

    /// Cross-check: nar_hash(Sha256) matches SimpleRenderer::calculate_nar().
    #[tokio::test]
    async fn nar_hash_sha256_matches_calculate() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let data = b"cross-check content";
        let (_, node) = insert_blob(&bs, data).await;

        let renderer = SimpleRenderer::new(bs.clone(), ds.clone());
        let (_size, sha256_from_calc) = renderer.calculate_nar(&node).await.unwrap();

        let hash_from_nar_hash = nar_hash(&node, HashAlgo::Sha256, bs.clone(), ds.clone())
            .await
            .unwrap();

        assert_eq!(
            &sha256_from_calc[..],
            hash_from_nar_hash.digest_as_bytes(),
            "nar_hash(Sha256) should produce the same digest as calculate_nar"
        );
    }
}
