//! Binary cache directory pull: import narinfo + NAR files into the local store.

use std::path::Path;

use nix_compat::narinfo::{NarInfo, VerifyingKey};
use nix_compat::store_path::StorePath;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::nar::ingest_nar_and_hash;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::export::export_castore_to_disk;
use crate::handle::StoreHandle;
use crate::Error;

/// Maximum number of narinfo files to scan in one pull.
const MAX_PULL_PATHS: usize = 1_000_000;

/// Options controlling pull behavior.
#[derive(Debug, Clone)]
pub struct PullOptions {
    /// Accept unsigned/unverified narinfos.
    pub trust_unsigned: bool,
    /// Trusted public keys for signature verification.
    pub trusted_public_keys: Vec<VerifyingKey>,
}

/// Record of a single successfully imported store path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PulledPath {
    pub store_path: String,
    pub nar_hash_hex: String,
    pub nar_size: u64,
}

/// Summary of a completed pull operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullReport {
    pub imported_count: u32,
    pub skipped_already_present_count: u32,
    pub skipped_untrusted_count: u32,
    pub skipped_hash_mismatch_count: u32,
    pub skipped_missing_nar_count: u32,
    pub skipped_parse_error_count: u32,
    pub skipped_store_dir_mismatch_count: u32,
    pub total_nar_bytes: u64,
    pub paths: Vec<PulledPath>,
}

/// Import PathInfo entries from a flat Nix binary cache directory.
///
/// Scans `*.narinfo` files in `source`, verifies signatures, ingests
/// the referenced NAR into castore, persists PathInfo, and exports
/// the output to the local store directory on disk.
///
/// If `paths_filter` is `Some`, only narinfos whose store path matches
/// one of the given selectors (substring match) are imported.
pub async fn import_paths_from_cache_dir(
    handle: &StoreHandle,
    source: &Path,
    paths_filter: Option<&[String]>,
    options: &PullOptions,
) -> Result<PullReport, Error> {
    assert!(!source.as_os_str().is_empty(), "pull source must not be empty");

    if !source.exists() {
        return Err(Error::Store(format!(
            "pull source directory does not exist: {}",
            source.display()
        )));
    }

    let narinfo_files = scan_narinfo_files(source).await?;
    assert!(
        narinfo_files.len() <= MAX_PULL_PATHS,
        "pull scan exceeds limit of {MAX_PULL_PATHS}"
    );

    let mut report = PullReport {
        imported_count: 0,
        skipped_already_present_count: 0,
        skipped_untrusted_count: 0,
        skipped_hash_mismatch_count: 0,
        skipped_missing_nar_count: 0,
        skipped_parse_error_count: 0,
        skipped_store_dir_mismatch_count: 0,
        total_nar_bytes: 0,
        paths: Vec::with_capacity(narinfo_files.len().min(256)),
    };

    let store_dir = handle.store_dir();
    let blob_service = handle.blob_service();
    let directory_service = handle.directory_service();
    let pathinfo_service = handle.pathinfo_service();

    for narinfo_path in &narinfo_files {
        pull_single_narinfo(
            narinfo_path,
            source,
            store_dir,
            paths_filter,
            options,
            &blob_service,
            &directory_service,
            &pathinfo_service,
            handle,
            &mut report,
        )
        .await?;
    }

    Ok(report)
}

async fn scan_narinfo_files(source: &Path) -> Result<Vec<std::path::PathBuf>, Error> {
    let mut entries = tokio::fs::read_dir(source)
        .await
        .map_err(|e| Error::Store(format!("reading pull source {}: {e}", source.display())))?;

    let mut narinfo_files = Vec::with_capacity(256);
    const MAX_DIR_SCAN: usize = 2_000_000; // generous: MAX_PULL_PATHS + non-narinfo files
    for _ in 0..MAX_DIR_SCAN {
        let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::Store(format!("scanning pull source: {e}")))?
        else {
            break;
        };

        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "narinfo") {
            narinfo_files.push(path);
        }
    }
    Ok(narinfo_files)
}

#[allow(clippy::too_many_arguments)]
async fn pull_single_narinfo(
    narinfo_path: &Path,
    source: &Path,
    store_dir: &str,
    paths_filter: Option<&[String]>,
    options: &PullOptions,
    blob_service: &(impl BlobService + Clone + 'static),
    directory_service: &(impl DirectoryService + Clone),
    pathinfo_service: &(impl PathInfoService + Clone),
    handle: &StoreHandle,
    report: &mut PullReport,
) -> Result<(), Error> {
    // 1. Read and parse narinfo.
    let narinfo_text = match tokio::fs::read_to_string(narinfo_path).await {
        Ok(text) => text,
        Err(e) => {
            tracing::warn!(path = %narinfo_path.display(), "failed to read narinfo: {e}");
            report.skipped_parse_error_count = report.skipped_parse_error_count.saturating_add(1);
            return Ok(());
        }
    };

    let narinfo = match NarInfo::parse_with_store_dir(&narinfo_text, store_dir) {
        Ok(ni) => ni,
        Err(e) => {
            // Try to detect store-dir mismatch by attempting parse with default /nix/store.
            // If the narinfo parses with a different prefix, it's a store-dir mismatch.
            if store_dir != nix_compat::store_path::STORE_DIR
                && NarInfo::parse(&narinfo_text).is_ok()
            {
                report.skipped_store_dir_mismatch_count =
                    report.skipped_store_dir_mismatch_count.saturating_add(1);
                return Ok(());
            }
            // Also try the reverse: we're /nix/store but the narinfo uses something else.
            tracing::warn!(
                path = %narinfo_path.display(),
                "failed to parse narinfo: {e}"
            );
            // Check if it's a store dir mismatch from the error.
            if matches!(e, nix_compat::narinfo::Error::InvalidStorePath(_)) {
                report.skipped_store_dir_mismatch_count =
                    report.skipped_store_dir_mismatch_count.saturating_add(1);
            } else {
                report.skipped_parse_error_count =
                    report.skipped_parse_error_count.saturating_add(1);
            }
            return Ok(());
        }
    };

    let store_path: StorePath<String> = narinfo.store_path.to_owned();
    let store_path_str = store_path.to_string();

    // 2. Apply path filter if provided.
    if let Some(filter) = paths_filter {
        if !filter.iter().any(|sel| store_path_str.contains(sel.as_str())) {
            return Ok(());
        }
    }

    // 3. Skip if already present in local PathInfo.
    if pathinfo_service.get((*store_path.digest()).into()).await.map_err(|e| {
        Error::PathInfoService(format!("checking existing PathInfo: {e}"))
    })?.is_some() {
        report.skipped_already_present_count = report.skipped_already_present_count.saturating_add(1);
        return Ok(());
    }

    // 4. Verify signatures.
    if !options.trust_unsigned {
        let fp = narinfo.fingerprint_with_store_dir(store_dir);
        let has_trusted_sig = narinfo.signatures.iter().any(|sig| {
            options
                .trusted_public_keys
                .iter()
                .any(|key| key.verify(&fp, sig))
        });
        if !has_trusted_sig {
            report.skipped_untrusted_count = report.skipped_untrusted_count.saturating_add(1);
            return Ok(());
        }
    }

    // 5. Resolve NAR file path from URL field.
    let nar_file_path = source.join(narinfo.url);
    if !nar_file_path.exists() {
        report.skipped_missing_nar_count = report.skipped_missing_nar_count.saturating_add(1);
        return Ok(());
    }

    // 6. Ingest the NAR.
    let nar_file = tokio::fs::File::open(&nar_file_path)
        .await
        .map_err(|e| Error::Store(format!("opening NAR {}: {e}", nar_file_path.display())))?;
    let mut nar_reader = tokio::io::BufReader::new(nar_file);

    let (node, actual_nar_sha256, actual_nar_size) =
        match ingest_nar_and_hash(blob_service.clone(), directory_service.clone(), &mut nar_reader, &narinfo.ca)
            .await
        {
            Ok(result) => result,
            Err(snix_store::nar::NarIngestionError::HashMismatch { .. }) => {
                report.skipped_hash_mismatch_count =
                    report.skipped_hash_mismatch_count.saturating_add(1);
                return Ok(());
            }
            Err(e) => {
                tracing::warn!(
                    path = %nar_file_path.display(),
                    "NAR ingestion failed (corrupt data?): {e}"
                );
                report.skipped_hash_mismatch_count =
                    report.skipped_hash_mismatch_count.saturating_add(1);
                return Ok(());
            }
        };

    // 7. Verify NAR hash against narinfo NarHash.
    if actual_nar_sha256 != narinfo.nar_hash {
        report.skipped_hash_mismatch_count = report.skipped_hash_mismatch_count.saturating_add(1);
        return Ok(());
    }

    // 8. Construct PathInfo from narinfo fields.
    let references: Vec<StorePath<String>> = narinfo
        .references
        .iter()
        .map(|r| r.to_owned())
        .collect();
    let signatures: Vec<nix_compat::narinfo::Signature<String>> = narinfo
        .signatures
        .iter()
        .map(|s| s.to_owned())
        .collect();

    let pi = PathInfo {
        store_path: store_path.clone(),
        node: node.clone(),
        references,
        nar_sha256: actual_nar_sha256,
        nar_size: actual_nar_size,
        signatures,
        deriver: narinfo.deriver.map(|d| d.to_owned()),
        ca: narinfo.ca.clone(),
    };

    // 9. Persist PathInfo.
    pathinfo_service
        .put(pi)
        .await
        .map_err(|e| Error::PathInfoService(format!("persisting imported PathInfo: {e}")))?;

    // 10. Export castore node to disk.
    let output_dir_str = handle.output_dir_str();
    let abs_path = store_path.to_absolute_path_with_prefix(&output_dir_str);
    if !std::path::Path::new(&abs_path).exists() {
        match export_castore_to_disk(&node, &abs_path, blob_service, directory_service).await {
            Ok(()) => {}
            Err(e) if e.contains("Read-only file system") || e.contains("Permission denied") => {
                tracing::warn!(
                    path = %abs_path,
                    "could not export imported path to disk (read-only store)"
                );
            }
            Err(e) => {
                return Err(Error::Export(format!("exporting imported path {abs_path}: {e}")));
            }
        }
    }

    report.imported_count = report.imported_count.saturating_add(1);
    report.total_nar_bytes = report.total_nar_bytes.saturating_add(actual_nar_size);
    report.paths.push(PulledPath {
        store_path: store_path_str,
        nar_hash_hex: data_encoding::HEXLOWER.encode(&actual_nar_sha256),
        nar_size: actual_nar_size,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::push::{PushOptions, export_paths_to_cache_dir};
    use crate::{StoreConfig, StoreFallbackMode};
    use nix_compat::nixbase32;
    use snix_castore::Node;
    use tokio::io::AsyncWriteExt;

    /// Build a minimal signed PathInfo with a single blob in the store.
    async fn make_signed_pathinfo(
        handle: &StoreHandle,
        name: &str,
        content: &[u8],
    ) -> PathInfo {
        let mut writer = handle.blob_service().open_write().await;
        writer.write_all(content).await.unwrap();
        let blob_digest = writer.close().await.unwrap();

        let node = Node::File {
            digest: blob_digest,
            size: content.len() as u64,
            executable: false,
        };
        let nar_buf = render_nar_bytes(handle, &node).await;

        let nar_sha256: [u8; 32] = {
            use sha2::Digest;
            sha2::Sha256::digest(&nar_buf).into()
        };

        let mut digest = [0u8; 20];
        for (i, b) in name.as_bytes().iter().enumerate().take(20) {
            digest[i] = *b;
        }
        let store_path = StorePath::from_name_and_digest_fixed(name, digest).unwrap();

        let mut pi = PathInfo {
            store_path,
            node,
            references: vec![],
            nar_sha256,
            nar_size: nar_buf.len() as u64,
            signatures: vec![],
            deriver: None,
            ca: None,
        };

        sign_pathinfo(&mut pi, "/nix/store");
        pi
    }

    fn test_signing_key() -> nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey> {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        nix_compat::narinfo::SigningKey::new("test-key-1".to_string(), secret)
    }

    fn test_verifying_key() -> VerifyingKey {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let public = ed25519_dalek::VerifyingKey::from(&secret);
        VerifyingKey::new("test-key-1".to_string(), public)
    }

    fn sign_pathinfo(pi: &mut PathInfo, store_dir: &str) {
        let signing_key = test_signing_key();
        let store_path_ref: nix_compat::store_path::StorePathRef = pi.store_path.as_ref();
        let refs: Vec<nix_compat::store_path::StorePathRef> =
            pi.references.iter().map(StorePath::as_ref).collect();
        let fp = nix_compat::narinfo::fingerprint_with_store_dir(
            &store_path_ref,
            &pi.nar_sha256,
            pi.nar_size,
            refs.iter(),
            store_dir,
        );
        let sig = signing_key.sign(fp.as_bytes()).to_owned();
        pi.signatures.push(sig);
    }

    async fn render_nar_bytes(handle: &StoreHandle, node: &Node) -> Vec<u8> {
        use snix_store::nar::write_nar;
        use tokio::io::AsyncReadExt;

        let (mut reader, writer) = tokio::io::duplex(64 * 1024);
        let node = node.clone();
        let bs = handle.blob_service();
        let ds = handle.directory_service();
        let write_task = tokio::spawn(async move {
            write_nar(writer, &node, bs, ds).await
        });
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        write_task.await.unwrap().unwrap();
        buf
    }

    async fn open_test_store(dir: &Path) -> StoreHandle {
        let state_dir = dir.join("state");
        let output_dir = dir.join("output");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        StoreHandle::open(StoreConfig {
            state_dir,
            output_dir,
            remote_cache_url: None,
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
        })
        .await
        .unwrap()
    }

    fn default_pull_options() -> PullOptions {
        PullOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_verifying_key()],
        }
    }

    // --- Push → Pull round-trip ---

    #[tokio::test]
    async fn pull_single_signed_path_round_trip() {
        let tmp = tempfile::tempdir().unwrap();

        // Push side: build and push a path.
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[pi.clone()],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        // Pull side: import into a fresh store.
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
        assert_eq!(report.skipped_already_present_count, 0);
        assert_eq!(report.skipped_untrusted_count, 0);
        assert_eq!(report.skipped_hash_mismatch_count, 0);
        assert_eq!(report.skipped_missing_nar_count, 0);
        assert_eq!(report.skipped_parse_error_count, 0);
        assert_eq!(report.paths.len(), 1);
        assert_eq!(report.paths[0].store_path, pi.store_path.to_string());
        assert_eq!(report.total_nar_bytes, pi.nar_size);

        // Verify PathInfo was persisted.
        let imported_pi = pull_store
            .pathinfo_service()
            .get((*pi.store_path.digest()).into())
            .await
            .unwrap()
            .expect("imported PathInfo must exist");
        assert_eq!(imported_pi.nar_sha256, pi.nar_sha256);
        assert_eq!(imported_pi.nar_size, pi.nar_size);
        assert_eq!(imported_pi.store_path, pi.store_path);
    }

    #[tokio::test]
    async fn pull_skips_already_present() {
        let tmp = tempfile::tempdir().unwrap();
        let store = open_test_store(&tmp.path().join("store")).await;
        let pi = make_signed_pathinfo(&store, "hello", b"hello world").await;

        // Push and pull once.
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &store,
            &[pi.clone()],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &default_pull_options(),
        )
        .await
        .unwrap();

        // Pull again — should skip.
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_already_present_count, 1);
    }

    #[tokio::test]
    async fn pull_rejects_untrusted_signature() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[pi],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        // Pull with a different trusted key — should reject.
        let other_secret = ed25519_dalek::SigningKey::from_bytes(&[99u8; 32]);
        let other_public = ed25519_dalek::VerifyingKey::from(&other_secret);
        let other_key = VerifyingKey::new("other-key-1".to_string(), other_public);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &PullOptions {
                trust_unsigned: false,
                trusted_public_keys: vec![other_key],
            },
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_untrusted_count, 1);
    }

    #[tokio::test]
    async fn pull_accepts_unsigned_when_trust_unsigned() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let mut pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;
        pi.signatures.clear(); // make unsigned

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[pi],
            &cache_dir,
            &PushOptions { trust_unsigned: true },
        )
        .await
        .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &PullOptions {
                trust_unsigned: true,
                trusted_public_keys: vec![],
            },
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
    }

    #[tokio::test]
    async fn pull_detects_nar_hash_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[pi.clone()],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        // Corrupt the NAR file.
        let nar_hash_b32 = nixbase32::encode(&pi.nar_sha256);
        let nar_path = cache_dir.join("nar").join(format!("{nar_hash_b32}.nar"));
        std::fs::write(&nar_path, b"corrupted data that is not a valid nar").unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &default_pull_options(),
        )
        .await
        .unwrap();

        // NAR ingestion will fail because the corrupted data isn't valid NAR format.
        // That manifests as a hash mismatch or an ingestion error. Either way, not imported.
        assert_eq!(report.imported_count, 0);
    }

    #[tokio::test]
    async fn pull_skips_missing_nar() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[pi.clone()],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        // Remove the NAR file.
        let nar_hash_b32 = nixbase32::encode(&pi.nar_sha256);
        let nar_path = cache_dir.join("nar").join(format!("{nar_hash_b32}.nar"));
        std::fs::remove_file(&nar_path).unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_missing_nar_count, 1);
    }

    #[tokio::test]
    async fn pull_rejects_store_dir_mismatch() {
        let tmp = tempfile::tempdir().unwrap();

        // Push from a /nix/store store.
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[pi],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        // Pull into a /crunch/store store — should reject.
        let pull_dir = tmp.path().join("pull");
        let state_dir = pull_dir.join("state");
        let output_dir = pull_dir.join("output");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        let pull_store = StoreHandle::open(StoreConfig {
            state_dir,
            output_dir,
            remote_cache_url: None,
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/crunch/store".to_string(),
        })
        .await
        .unwrap();

        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &PullOptions {
                trust_unsigned: true,
                trusted_public_keys: vec![],
            },
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_store_dir_mismatch_count, 1);
    }

    #[tokio::test]
    async fn pull_with_path_filter() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let a = make_signed_pathinfo(&push_store, "pkg-a", b"aaa").await;
        let b = make_signed_pathinfo(&push_store, "pkg-b", b"bbb").await;
        let c = make_signed_pathinfo(&push_store, "pkg-c", b"ccc").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[a.clone(), b.clone(), c.clone()],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            Some(&["pkg-a".to_string(), "pkg-c".to_string()]),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 2);
        let imported_names: Vec<&str> = report.paths.iter().map(|p| p.store_path.as_str()).collect();
        assert!(imported_names.iter().any(|s| s.contains("pkg-a")));
        assert!(imported_names.iter().any(|s| s.contains("pkg-c")));
        assert!(!imported_names.iter().any(|s| s.contains("pkg-b")));
    }

    #[tokio::test]
    async fn pull_multiple_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let a = make_signed_pathinfo(&push_store, "pkg-a", b"aaa").await;
        let b = make_signed_pathinfo(&push_store, "pkg-b", b"bbb").await;
        let c = make_signed_pathinfo(&push_store, "pkg-c", b"ccc").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(
            &push_store,
            &[a, b, c],
            &cache_dir,
            &PushOptions { trust_unsigned: false },
        )
        .await
        .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            None,
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 3);
        assert_eq!(report.paths.len(), 3);
    }

    #[tokio::test]
    async fn pull_nonexistent_source_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let store = open_test_store(&tmp.path().join("store")).await;

        let result = import_paths_from_cache_dir(
            &store,
            &tmp.path().join("nonexistent"),
            None,
            &default_pull_options(),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not exist"), "error: {err}");
    }
}
