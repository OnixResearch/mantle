//! Store query operations: list, info, verify.
//!
//! These operate on a PathInfoService and are independent of the build
//! engine. Callable from the CLI or as a library.

use futures::StreamExt;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_castore::import::fs::ingest_path;
use snix_store::nar::NarCalculationService;
use snix_store::nar::SimpleRenderer;
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;
use crate::gc::saturating_u32;

/// Info for a single store path, returned by `store_info`.
#[derive(Debug)]
pub struct PathInfoDetail {
    pub store_path: String,
    pub nar_size: u64,
    pub nar_sha256: Vec<u8>,
    pub deriver: Option<String>,
    pub references: Vec<String>,
    pub signatures: Vec<String>,
    pub ca: Option<String>,
    pub node: String,
}

/// Result of a single path verification.
#[derive(Debug)]
pub enum VerifyResult {
    Ok(String),
    Missing(String),
    Mismatch {
        path: String,
        stored_hash: String,
        actual_hash: String,
    },
}

/// Result of signature verification for a single PathInfo.
#[derive(Debug, Clone)]
pub struct SignatureVerifyResult {
    pub path: String,
    pub trusted_count: u32,
    pub untrusted_names: Vec<String>,
    pub total_signatures: u32,
}

impl SignatureVerifyResult {
    pub fn is_trusted(&self) -> bool {
        self.trusted_count > 0
    }
}

/// List all store paths from a PathInfoService.
///
/// Returns (store_path, deriver_name, nar_size) tuples.
pub async fn store_list(svc: &dyn PathInfoService) -> Result<Vec<(String, String, u64)>, Error> {
    const MAX_LIST_ENTRIES: usize = 1_000_000;
    let mut stream = svc.list();
    let mut results = Vec::with_capacity(256);
    for _ in 0..MAX_LIST_ENTRIES {
        let Some(result) = stream.next().await else { break };
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let deriver_name = pi.deriver.as_ref().map(|d| d.name().to_string()).unwrap_or_else(|| "-".to_string());
        results.push((pi.store_path.to_string(), deriver_name, pi.nar_size));
    }
    Ok(results)
}

/// Get detailed PathInfo for paths matching a substring filter.
pub async fn store_info(svc: &dyn PathInfoService, path_filter: &str) -> Result<Vec<PathInfoDetail>, Error> {
    assert!(!path_filter.is_empty(), "store_info: empty path_filter would match everything");

    const MAX_SCAN_ENTRIES: u32 = 1_000_000;
    let mut stream = svc.list();
    let mut scanned_count: u32 = 0;
    let mut results = Vec::with_capacity(64);
    for _ in 0..MAX_SCAN_ENTRIES {
        let Some(result) = stream.next().await else { break };
        scanned_count = scanned_count.saturating_add(1);
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();
        if !sp_str.contains(path_filter) {
            continue;
        }
        results.push(PathInfoDetail {
            store_path: sp_str,
            nar_size: pi.nar_size,
            nar_sha256: pi.nar_sha256.to_vec(),
            deriver: pi.deriver.map(|d| d.to_string()),
            references: pi.references.iter().map(|r| r.to_string()).collect(),
            signatures: pi.signatures.iter().map(|s| s.to_string()).collect(),
            ca: pi.ca.map(|c| format!("{c:?}")),
            node: format!("{:?}", pi.node),
        });
    }
    if let Ok(n) = u64::try_from(results.len()) {
        assert!(n <= u64::from(scanned_count), "results cannot exceed scanned paths");
    }
    Ok(results)
}

/// Verify NAR hashes of store paths against what's on disk.
///
/// Optionally filters to paths matching `path_filter`. Returns per-path
/// results (Ok, Missing, or Mismatch).
pub async fn store_verify(svc: &dyn PathInfoService, path_filter: Option<&str>) -> Result<Vec<VerifyResult>, Error> {
    assert!(path_filter.is_none_or(|f| !f.is_empty()), "store_verify: use None instead of empty filter");

    #[allow(tigerstyle::explicit_defaults)]
    let bs = MemoryBlobService::default();
    let ds = RedbDirectoryService::new_temporary("verify".to_string(), RedbDirectoryServiceConfig {
        path: None,
        cache_size: None,
        read_only: false,
    })
    .map_err(|e| Error::DirectoryService(format!("{e}")))?;

    const MAX_VERIFY_ENTRIES: u32 = 1_000_000;
    let mut stream = svc.list();
    let mut scanned_count: u32 = 0;
    let mut results = Vec::with_capacity(64);

    for _ in 0..MAX_VERIFY_ENTRIES {
        let Some(result) = stream.next().await else { break };
        scanned_count = scanned_count.saturating_add(1);
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();

        if let Some(filter) = path_filter
            && !sp_str.contains(filter)
        {
            continue;
        }

        let abs = std::path::Path::new("/nix/store").join(&sp_str);
        if !abs.exists() {
            results.push(VerifyResult::Missing(sp_str));
            continue;
        }

        let node = ingest_path::<_, _, _, &[u8]>(bs.clone(), ds.clone(), &abs, None)
            .await
            .map_err(|e| Error::Store(format!("ingest {sp_str}: {e}")))?;

        let renderer = SimpleRenderer::new(bs.clone(), ds.clone());
        let (_nar_size, nar_sha256) =
            renderer.calculate_nar(&node).await.map_err(|e| Error::Store(format!("NAR calc: {e}")))?;

        if nar_sha256 == pi.nar_sha256 {
            results.push(VerifyResult::Ok(sp_str));
        } else {
            results.push(VerifyResult::Mismatch {
                path: sp_str,
                stored_hash: data_encoding::HEXLOWER.encode(&pi.nar_sha256),
                actual_hash: data_encoding::HEXLOWER.encode(&nar_sha256),
            });
        }
    }
    if let Ok(n) = u64::try_from(results.len()) {
        assert!(n <= u64::from(scanned_count), "verify results cannot exceed scanned paths");
    }

    Ok(results)
}

/// Verify PathInfo signatures against trusted public keys.
pub async fn store_verify_signatures(
    svc: &dyn PathInfoService,
    path_filter: Option<&str>,
    trusted_keys: &[VerifyingKey],
) -> Result<Vec<SignatureVerifyResult>, Error> {
    use nix_compat::narinfo::fingerprint;
    use nix_compat::store_path::StorePathRef;

    assert!(!trusted_keys.is_empty(), "verify_signatures requires at least one trusted key");

    const MAX_VERIFY_SIG_ENTRIES: usize = 1_000_000;
    let mut stream = svc.list();
    let mut results = Vec::with_capacity(64);

    for _ in 0..MAX_VERIFY_SIG_ENTRIES {
        let Some(result) = stream.next().await else { break };
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let path = pi.store_path.to_string();

        if let Some(filter) = path_filter
            && !path.contains(filter)
        {
            continue;
        }

        let store_path_ref: StorePathRef = pi.store_path.as_ref();
        let refs: Vec<StorePathRef> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = fingerprint(&store_path_ref, &pi.nar_sha256, pi.nar_size, refs.iter());

        let mut trusted_count: u32 = 0;
        let mut untrusted_names = Vec::with_capacity(pi.signatures.len());
        for sig in &pi.signatures {
            let sig_ref = sig.as_ref();
            let is_trusted = trusted_keys.iter().any(|key| key.verify(&fp, &sig_ref));
            if is_trusted {
                trusted_count = trusted_count.saturating_add(1);
            } else {
                untrusted_names.push(sig.name().clone());
            }
        }

        let total_signatures = saturating_u32(pi.signatures.len());
        assert!(
            total_signatures >= trusted_count,
            "trusted count ({trusted_count}) cannot exceed total signatures ({total_signatures})"
        );
        results.push(SignatureVerifyResult {
            path,
            trusted_count,
            untrusted_names,
            total_signatures,
        });
    }

    Ok(results)
}

/// Result of signing a single PathInfo.
#[derive(Debug)]
pub struct SignResult {
    pub store_path: String,
    /// True if the PathInfo had no signatures before this call.
    pub newly_signed: bool,
    /// True if the PathInfo already had signatures, and this call appended
    /// a signature from a different key.
    pub appended: bool,
    /// True if a signature from the same key was replaced.
    pub replaced: bool,
}

/// Sign PathInfo entries matching `path_filter`.
///
/// When `is_sign_all` is true, only unsigned PathInfos are updated. If a
/// PathInfo already has a signature from the same key name, it is replaced
/// instead of duplicated. Returns one result per updated path.
pub async fn store_sign(
    svc: &dyn PathInfoService,
    signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    path_filter: Option<&str>,
    is_sign_all: bool,
) -> Result<Vec<SignResult>, Error> {
    use nix_compat::narinfo::Signature;
    use nix_compat::narinfo::fingerprint;
    use nix_compat::store_path::StorePathRef;

    const SIGN_CANDIDATE_COUNT_LIMIT: usize = 4096;
    assert!(!signing_key.name().is_empty(), "signing key name must not be empty");

    let mut stream = svc.list();
    let mut to_update = Vec::with_capacity(SIGN_CANDIDATE_COUNT_LIMIT);
    let mut scanned_count: usize = 0;

    for _ in 0..=SIGN_CANDIDATE_COUNT_LIMIT {
        let Some(result) = stream.next().await else { break };
        scanned_count = scanned_count.saturating_add(1);
        if scanned_count > SIGN_CANDIDATE_COUNT_LIMIT {
            return Err(Error::PathInfoService(format!(
                "sign candidate scan exceeded limit {SIGN_CANDIDATE_COUNT_LIMIT}"
            )));
        }

        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();

        if is_sign_all {
            if !pi.signatures.is_empty() {
                continue;
            }
        } else if let Some(filter) = path_filter {
            if !sp_str.contains(filter) {
                continue;
            }
        } else {
            continue;
        }

        to_update.push((sp_str, pi));
    }

    let mut results = Vec::with_capacity(to_update.len());
    for (sp_str, mut pi) in to_update {
        let sp_ref: StorePathRef = pi.store_path.as_ref();
        let refs: Vec<StorePathRef> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = fingerprint(&sp_ref, &pi.nar_sha256, pi.nar_size, refs.iter());

        let sig_ref = signing_key.sign(fp.as_bytes());
        let sig_owned: Signature<String> = sig_ref.to_owned();
        let key_name = signing_key.name();

        let is_already_signed = !pi.signatures.is_empty();
        let is_signed_by_same_key = pi.signatures.iter().any(|s| s.name().as_str() == key_name);

        if let Some(pos) = pi.signatures.iter().position(|s| s.name().as_str() == key_name) {
            pi.signatures[pos] = sig_owned;
        } else {
            pi.signatures.push(sig_owned);
        }

        svc.put(pi).await.map_err(|e| Error::PathInfoService(format!("persisting signed PathInfo: {e}")))?;

        let result = SignResult {
            store_path: sp_str,
            newly_signed: !is_already_signed,
            appended: is_already_signed && !is_signed_by_same_key,
            replaced: is_signed_by_same_key,
        };
        assert!(
            (result.newly_signed as u8)
                .saturating_add(result.appended as u8)
                .saturating_add(result.replaced as u8)
                == 1,
            "sign result must be exactly one of newly_signed, appended, or replaced"
        );
        results.push(result);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;

    use super::*;

    fn test_pathinfo_service() -> LruPathInfoService {
        LruPathInfoService::with_capacity("query-test".to_string(), std::num::NonZeroUsize::new(64).unwrap())
    }

    fn test_keypair() -> (SigningKey<ed25519_dalek::SigningKey>, VerifyingKey) {
        nix_compat::narinfo::parse_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        )
        .unwrap()
    }

    fn other_signing_key() -> SigningKey<ed25519_dalek::SigningKey> {
        SigningKey::new("backup-cache-1".to_string(), ed25519_dalek::SigningKey::from_bytes(&[2u8; 32]))
    }

    fn other_verifying_key() -> VerifyingKey {
        nix_compat::narinfo::VerifyingKey::parse("cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=")
            .unwrap()
    }

    fn dummy_pathinfo(name: &str) -> PathInfo {
        let digest = *blake3::hash(name.as_bytes()).as_bytes();
        let mut store_digest = [0u8; 20];
        store_digest.copy_from_slice(&digest[..20]);
        let store_path = StorePath::from_name_and_digest_fixed(name, store_digest).unwrap();
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 42,
            nar_sha256: [9u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[tokio::test]
    async fn store_sign_adds_signature() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let pi = dummy_pathinfo("signed-path");
        let digest = *pi.store_path.digest();
        svc.put(pi).await.unwrap();

        let results = store_sign(&svc, &signing_key, Some("signed-path"), false).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].newly_signed);
        assert!(!results[0].appended);
        assert!(!results[0].replaced);

        let stored = svc.get(digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 1);
        assert_eq!(stored.signatures[0].name().as_str(), "cache.example.com-1");
    }

    #[tokio::test]
    async fn store_sign_replaces_same_key_signature() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let mut pi = dummy_pathinfo("replace-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());

        let digest = *pi.store_path.digest();
        svc.put(pi).await.unwrap();

        let results = store_sign(&svc, &signing_key, Some("replace-path"), false).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].newly_signed);
        assert!(!results[0].appended);
        assert!(results[0].replaced);

        let stored = svc.get(digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 1);
    }

    #[tokio::test]
    async fn store_sign_appends_different_key_signature() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let other_key = other_signing_key();
        let mut pi = dummy_pathinfo("append-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(other_key.sign(fp.as_bytes()).to_owned());

        let digest = *pi.store_path.digest();
        svc.put(pi).await.unwrap();

        let results = store_sign(&svc, &signing_key, Some("append-path"), false).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].newly_signed);
        assert!(results[0].appended);
        assert!(!results[0].replaced);

        let stored = svc.get(digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 2);
    }

    #[tokio::test]
    async fn store_sign_all_skips_already_signed_entries() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let other_key = other_signing_key();

        let unsigned = dummy_pathinfo("unsigned-path");
        svc.put(unsigned).await.unwrap();

        let mut signed = dummy_pathinfo("already-signed-path");
        let refs: Vec<_> = signed.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(
            &signed.store_path.as_ref(),
            &signed.nar_sha256,
            signed.nar_size,
            refs.iter(),
        );
        signed.signatures.push(other_key.sign(fp.as_bytes()).to_owned());
        let signed_digest = *signed.store_path.digest();
        svc.put(signed).await.unwrap();

        let results = store_sign(&svc, &signing_key, None, true).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].store_path.ends_with("-unsigned-path"));
        assert!(results[0].newly_signed);
        assert!(!results[0].appended);
        assert!(!results[0].replaced);

        let stored = svc.get(signed_digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 1);
        assert_eq!(stored.signatures[0].name().as_str(), "backup-cache-1");
    }

    #[tokio::test]
    async fn verify_signatures_accepts_trusted_key() {
        let svc = test_pathinfo_service();
        let (signing_key, verifying_key) = test_keypair();
        let mut pi = dummy_pathinfo("trusted-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
        svc.put(pi).await.unwrap();

        let results = store_verify_signatures(&svc, Some("trusted-path"), &[verifying_key]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_trusted());
        assert_eq!(results[0].trusted_count, 1);
        assert!(results[0].untrusted_names.is_empty());
    }

    #[tokio::test]
    async fn verify_signatures_reports_untrusted_signer() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let mut pi = dummy_pathinfo("untrusted-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
        svc.put(pi).await.unwrap();

        let results = store_verify_signatures(&svc, Some("untrusted-path"), &[other_verifying_key()]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_trusted());
        assert_eq!(results[0].trusted_count, 0);
        assert_eq!(results[0].total_signatures, 1);
        assert_eq!(results[0].untrusted_names, vec!["cache.example.com-1".to_string()]);
    }
}
