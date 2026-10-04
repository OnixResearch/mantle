// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in .cairn/changes/complete-store-capability-migration/evidence/
// tigerstyle-remaining-2026-09-09.log and scheduled for the standalone store-shell
// hardening pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::sentinel_fallback)]

//! Store query operations: list, info, verify.
//!
//! These operate on a PathInfoService and are independent of the build
//! engine. Callable from the CLI or as a library.

use crunch_nar::CaseHackPolicy;
use crunch_nar::FilesystemNarRequest;
use futures::StreamExt;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::nixhash::HashAlgo;
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
        stored_size: u64,
        actual_size: u64,
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

/// Verify NAR hashes of store paths against what's on disk under `store_dir`.
///
/// `store_dir` is the physical directory where build outputs are exported
/// (the CLI `--store` value), not the logical store prefix. Optionally filters
/// to paths matching `path_filter`. Returns per-path results (Ok, Missing, or
/// Mismatch).
// r[impl store_transports.nix_archive_filesystem_observation]
// r[impl store_transports.nix_archive_boundary]
pub(crate) async fn store_verify(
    svc: &dyn PathInfoService,
    path_filter: Option<&str>,
    store_dir: &std::path::Path,
) -> Result<Vec<VerifyResult>, Error> {
    const MAX_VERIFY_ENTRIES: u32 = 1_000_000;
    const STORE_VERIFY_NAR_BYTES_MAX: u64 = 17_592_186_044_416;
    assert!(
        path_filter.is_none_or(|filter| !filter.is_empty()),
        "store_verify: use None instead of empty filter"
    );
    assert!(!store_dir.as_os_str().is_empty(), "store_verify: store_dir must not be empty");
    let mut stream = svc.list();
    let mut scanned_count: u32 = 0;
    let mut results = Vec::with_capacity(64);

    for _ in 0..MAX_VERIFY_ENTRIES {
        let Some(result) = stream.next().await else { break };
        scanned_count = scanned_count.saturating_add(1);
        let path_info = result.map_err(|error| Error::PathInfoService(format!("listing: {error}")))?;
        let logical_path = path_info.store_path.to_string();
        if path_filter.is_some_and(|filter| !logical_path.contains(filter)) {
            continue;
        }
        let physical_path = store_dir.join(&logical_path);
        match std::fs::symlink_metadata(&physical_path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                results.push(VerifyResult::Missing(logical_path));
                continue;
            }
            Err(error) => {
                return Err(Error::Store(format!(
                    "inspecting filesystem NAR root {}: {error}",
                    physical_path.display()
                )));
            }
        }
        let request = FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::native(), STORE_VERIFY_NAR_BYTES_MAX);
        let observation = crunch_nar::observe_path_blocking(physical_path, request)
            .await
            .map_err(|error| Error::Store(format!("filesystem NAR observation for {logical_path}: {error}")))?;
        let actual_hash = observation.digest.digest_as_bytes();
        if actual_hash == path_info.nar_sha256 && observation.nar_size == path_info.nar_size {
            results.push(VerifyResult::Ok(logical_path));
        } else {
            results.push(VerifyResult::Mismatch {
                path: logical_path,
                stored_hash: data_encoding::HEXLOWER.encode(&path_info.nar_sha256),
                actual_hash: data_encoding::HEXLOWER.encode(actual_hash),
                stored_size: path_info.nar_size,
                actual_size: observation.nar_size,
            });
        }
    }
    assert!(results.len() <= usize::try_from(scanned_count).unwrap_or(usize::MAX));
    assert!(scanned_count <= MAX_VERIFY_ENTRIES);
    Ok(results)
}

/// Verify PathInfo signatures against trusted public keys.
///
/// `store_dir` is the logical store prefix (for example `/mantle/store`)
/// that was used when the signatures were created. Fingerprints embed the
/// prefix, so verification with the wrong prefix reports every signature as
/// untrusted.
pub(crate) async fn store_verify_signatures(
    svc: &dyn PathInfoService,
    path_filter: Option<&str>,
    trusted_keys: &[VerifyingKey],
    store_dir: &str,
) -> Result<Vec<SignatureVerifyResult>, Error> {
    use nix_compat::narinfo::fingerprint_with_store_dir;
    use nix_compat::store_path::StorePathRef;

    assert!(!trusted_keys.is_empty(), "verify_signatures requires at least one trusted key");
    assert!(!store_dir.is_empty(), "verify_signatures: store_dir must not be empty");

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
        let fp = fingerprint_with_store_dir(&store_path_ref, &pi.nar_sha256, pi.nar_size, refs.iter(), store_dir);

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
pub(crate) async fn store_sign(
    svc: &dyn PathInfoService,
    signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    path_filter: Option<&str>,
    is_sign_all: bool,
    store_dir: &str,
) -> Result<Vec<SignResult>, Error> {
    use nix_compat::narinfo::Signature;
    use nix_compat::narinfo::fingerprint_with_store_dir;
    use nix_compat::store_path::StorePathRef;

    const SIGN_CANDIDATE_COUNT_LIMIT: usize = 4096;
    assert!(!signing_key.name().is_empty(), "signing key name must not be empty");
    assert!(!store_dir.is_empty(), "store_sign: store_dir must not be empty");

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
        let fp = fingerprint_with_store_dir(&sp_ref, &pi.nar_sha256, pi.nar_size, refs.iter(), store_dir);

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
    use std::os::unix::fs::PermissionsExt;

    use nix_compat::store_path::StorePath;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;

    use super::*;

    /// Existing test vectors sign with the plain `fingerprint()` helper, which
    /// embeds the nix-compat default store dir.
    const TEST_STORE_DIR: &str = nix_compat::store_path::STORE_DIR;

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

        let results = store_sign(&svc, &signing_key, Some("signed-path"), false, TEST_STORE_DIR).await.unwrap();
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

        let results = store_sign(&svc, &signing_key, Some("replace-path"), false, TEST_STORE_DIR).await.unwrap();
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

        let results = store_sign(&svc, &signing_key, Some("append-path"), false, TEST_STORE_DIR).await.unwrap();
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

        let results = store_sign(&svc, &signing_key, None, true, TEST_STORE_DIR).await.unwrap();
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

        let results =
            store_verify_signatures(&svc, Some("trusted-path"), &[verifying_key], TEST_STORE_DIR).await.unwrap();
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

        let results = store_verify_signatures(&svc, Some("untrusted-path"), &[other_verifying_key()], TEST_STORE_DIR)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_trusted());
        assert_eq!(results[0].trusted_count, 0);
        assert_eq!(results[0].total_signatures, 1);
        assert_eq!(results[0].untrusted_names, vec!["cache.example.com-1".to_string()]);
    }

    /// Compute the NAR hash of an on-disk path the same way `store_verify` does,
    /// so a positive test can pin the exact expected hash.
    async fn nar_hash_of_disk_path(path: &std::path::Path) -> ([u8; 32], u64) {
        const TEST_NAR_BYTES_MAX: u64 = 1_048_576;
        let observation = crunch_nar::observe_path_blocking(
            path.to_path_buf(),
            FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::native(), TEST_NAR_BYTES_MAX),
        )
        .await
        .unwrap();
        let hash: [u8; 32] = observation.digest.digest_as_bytes().try_into().unwrap();
        assert_eq!(observation.algorithm, HashAlgo::Sha256);
        assert!(observation.nar_size > 0);
        (hash, observation.nar_size)
    }

    fn file_pathinfo(store_path: StorePath<String>, nar_sha256: [u8; 32], nar_size: u64) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    // r[verify store_transports.nix_archive_filesystem_observation]
    #[tokio::test]
    async fn store_verify_ok_for_exported_path_in_custom_store_dir() {
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();

        let pi = dummy_pathinfo("verify-ok-path");
        let sp_str = pi.store_path.to_string();
        std::fs::write(store_dir.path().join(&sp_str), b"verify-ok").unwrap();
        let (nar_sha256, nar_size) = nar_hash_of_disk_path(&store_dir.path().join(&sp_str)).await;
        svc.put(file_pathinfo(pi.store_path, nar_sha256, nar_size)).await.unwrap();

        let results = store_verify(&svc, None, store_dir.path()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], VerifyResult::Ok(p) if *p == sp_str));
    }

    #[tokio::test]
    async fn store_verify_missing_for_path_not_on_disk_in_custom_store_dir() {
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();

        let pi = dummy_pathinfo("verify-missing-path");
        let sp_str = pi.store_path.to_string();
        svc.put(pi).await.unwrap();

        let results = store_verify(&svc, None, store_dir.path()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], VerifyResult::Missing(p) if *p == sp_str));
    }

    // r[verify store_transports.nix_archive_filesystem_observation]
    #[tokio::test]
    async fn store_verify_mismatch_for_tampered_disk_content() {
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();
        let path_info = dummy_pathinfo("verify-mismatch-path");
        let logical_path = path_info.store_path.to_string();
        let good_path = store_dir.path().join(format!("{logical_path}.good"));
        std::fs::write(&good_path, b"original").unwrap();
        let (nar_sha256, nar_size) = nar_hash_of_disk_path(&good_path).await;
        svc.put(file_pathinfo(path_info.store_path, nar_sha256, nar_size)).await.unwrap();
        std::fs::write(store_dir.path().join(&logical_path), b"tampered and larger").unwrap();

        let results = store_verify(&svc, None, store_dir.path()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(
            &results[0],
            VerifyResult::Mismatch {
                path,
                stored_size,
                actual_size,
                ..
            } if *path == logical_path && stored_size != actual_size
        ));
    }

    // r[verify store_transports.nix_archive_filesystem_observation]
    #[tokio::test]
    async fn store_verify_read_failure_does_not_persist_pathinfo() {
        const UNREADABLE_MODE: u32 = 0o000;
        const RESTORED_MODE: u32 = 0o600;
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();
        let path_info = dummy_pathinfo("verify-read-failure-path");
        let digest = *path_info.store_path.digest();
        let logical_path = path_info.store_path.to_string();
        svc.put(path_info).await.unwrap();
        let physical_path = store_dir.path().join(&logical_path);
        std::fs::write(&physical_path, b"unreadable").unwrap();
        std::fs::set_permissions(&physical_path, std::fs::Permissions::from_mode(UNREADABLE_MODE)).unwrap();

        let result = store_verify(&svc, None, store_dir.path()).await;
        std::fs::set_permissions(&physical_path, std::fs::Permissions::from_mode(RESTORED_MODE)).unwrap();
        let retained = svc.get(digest).await.unwrap();
        assert!(result.is_err());
        assert!(retained.is_some());
    }

    const CUSTOM_PREFIX: &str = "/mantle/store";

    #[tokio::test]
    async fn store_sign_then_verify_roundtrips_under_custom_prefix() {
        let svc = test_pathinfo_service();
        let (signing_key, verifying_key) = test_keypair();
        let pi = dummy_pathinfo("prefix-roundtrip-path");
        svc.put(pi).await.unwrap();

        let sign_results =
            store_sign(&svc, &signing_key, Some("prefix-roundtrip-path"), false, CUSTOM_PREFIX).await.unwrap();
        assert_eq!(sign_results.len(), 1);
        assert!(sign_results[0].newly_signed);

        let verify_results =
            store_verify_signatures(&svc, Some("prefix-roundtrip-path"), &[verifying_key], CUSTOM_PREFIX)
                .await
                .unwrap();
        assert_eq!(verify_results.len(), 1);
        assert!(verify_results[0].is_trusted());
        assert_eq!(verify_results[0].trusted_count, 1);
    }

    #[tokio::test]
    async fn store_verify_signatures_rejects_wrong_prefix() {
        let svc = test_pathinfo_service();
        let (signing_key, verifying_key) = test_keypair();
        let pi = dummy_pathinfo("prefix-mismatch-path");
        svc.put(pi).await.unwrap();

        store_sign(&svc, &signing_key, Some("prefix-mismatch-path"), false, CUSTOM_PREFIX).await.unwrap();

        let verify_results =
            store_verify_signatures(&svc, Some("prefix-mismatch-path"), &[verifying_key], TEST_STORE_DIR)
                .await
                .unwrap();
        assert_eq!(verify_results.len(), 1);
        assert!(!verify_results[0].is_trusted());
        assert_eq!(verify_results[0].trusted_count, 0);
    }
}
