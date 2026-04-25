//! Binary cache directory push: export signed PathInfo as narinfo + NAR files.

use std::path::Path;
use std::path::PathBuf;

use nix_compat::nixbase32;
use snix_store::path_info::PathInfo;
use tokio::io::AsyncWriteExt;

use crate::Error;
use crate::handle::StoreHandle;

/// Options controlling push behavior.
#[derive(Debug, Clone)]
pub struct PushOptions {
    /// Include unsigned PathInfo entries in the push.
    pub trust_unsigned: bool,
}

/// Summary of a completed push operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushReport {
    pub pushed_count: u32,
    pub skipped_unsigned_count: u32,
    pub skipped_already_present_count: u32,
    pub total_nar_bytes: u64,
    pub total_narinfo_bytes: u64,
    pub paths: Vec<PushedPath>,
}

/// Record of a single pushed store path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushedPath {
    pub store_path: String,
    pub nar_hash_hex: String,
    pub nar_size: u64,
}

/// Export selected PathInfo entries to a flat Nix binary cache directory.
///
/// Layout:
/// ```text
/// <dest>/nix-cache-info
/// <dest>/<store-path-hash>.narinfo
/// <dest>/nar/<nar-sha256-nixbase32>.nar
/// ```
///
/// Paths without signatures are skipped unless `options.trust_unsigned` is set.
/// Paths whose narinfo already exists in `dest` are skipped (idempotent).
pub async fn export_paths_to_cache_dir(
    handle: &StoreHandle,
    paths: &[PathInfo],
    dest: &Path,
    options: &PushOptions,
) -> Result<PushReport, Error> {
    assert!(!dest.as_os_str().is_empty(), "push destination must not be empty");
    assert!(paths.len() <= MAX_PUSH_PATHS, "push batch exceeds limit of {MAX_PUSH_PATHS}");

    let nar_dir = dest.join("nar");
    tokio::fs::create_dir_all(&nar_dir)
        .await
        .map_err(|e| Error::Export(format!("creating nar directory {}: {e}", nar_dir.display())))?;

    write_nix_cache_info_if_absent(dest, handle.store_dir()).await?;

    let mut report = PushReport {
        pushed_count: 0,
        skipped_unsigned_count: 0,
        skipped_already_present_count: 0,
        total_nar_bytes: 0,
        total_narinfo_bytes: 0,
        paths: Vec::with_capacity(paths.len()),
    };

    for pi in paths {
        push_single_path(handle, pi, dest, &nar_dir, options, &mut report, handle.store_dir()).await?;
    }

    Ok(report)
}

const MAX_PUSH_PATHS: usize = 1_000_000;

async fn push_single_path(
    handle: &StoreHandle,
    pi: &PathInfo,
    dest: &Path,
    nar_dir: &Path,
    options: &PushOptions,
    report: &mut PushReport,
    store_dir: &str,
) -> Result<(), Error> {
    // Skip unsigned unless trusted.
    if pi.signatures.is_empty() && !options.trust_unsigned {
        report.skipped_unsigned_count = report.skipped_unsigned_count.saturating_add(1);
        return Ok(());
    }

    // Idempotent: skip if narinfo already present.
    let narinfo_filename = narinfo_filename_for(&pi.store_path);
    let narinfo_path = dest.join(&narinfo_filename);
    if narinfo_path.exists() {
        report.skipped_already_present_count = report.skipped_already_present_count.saturating_add(1);
        return Ok(());
    }

    // Render NAR to a temp file in the nar directory, then rename.
    let nar_hash_nixbase32 = nixbase32::encode(&pi.nar_sha256);
    let nar_filename = format!("{nar_hash_nixbase32}.nar");
    let nar_path = nar_dir.join(&nar_filename);

    // NAR dedup: if the NAR already exists (shared content), skip rendering.
    if !nar_path.exists() {
        render_nar_to_file(handle, pi, &nar_path).await?;
    }

    // Build narinfo text.
    let nar_url = format!("nar/{nar_filename}");
    let narinfo_text = build_narinfo_text(pi, &nar_url, store_dir)?;
    let narinfo_bytes = narinfo_text.as_bytes();

    // Write narinfo atomically via temp + rename.
    let tmp_narinfo = temp_path_for(&narinfo_path);
    tokio::fs::write(&tmp_narinfo, narinfo_bytes)
        .await
        .map_err(|e| Error::Export(format!("writing narinfo {}: {e}", tmp_narinfo.display())))?;
    tokio::fs::rename(&tmp_narinfo, &narinfo_path)
        .await
        .map_err(|e| Error::Export(format!("renaming narinfo: {e}")))?;

    report.pushed_count = report.pushed_count.saturating_add(1);
    report.total_nar_bytes = report.total_nar_bytes.saturating_add(pi.nar_size);
    report.total_narinfo_bytes = report.total_narinfo_bytes.saturating_add(narinfo_bytes.len() as u64);
    report.paths.push(PushedPath {
        store_path: pi.store_path.to_string(),
        nar_hash_hex: data_encoding::HEXLOWER.encode(&pi.nar_sha256),
        nar_size: pi.nar_size,
    });

    Ok(())
}

async fn render_nar_to_file(handle: &StoreHandle, pi: &PathInfo, nar_path: &Path) -> Result<(), Error> {
    let tmp_nar = temp_path_for(nar_path);
    let file = tokio::fs::File::create(&tmp_nar)
        .await
        .map_err(|e| Error::Export(format!("creating NAR file {}: {e}", tmp_nar.display())))?;

    let mut buf_writer = tokio::io::BufWriter::new(file);
    handle.render_nar(&pi.node, &mut buf_writer).await?;
    buf_writer.flush().await.map_err(|e| Error::Export(format!("flushing NAR: {e}")))?;
    drop(buf_writer);

    tokio::fs::rename(&tmp_nar, nar_path)
        .await
        .map_err(|e| Error::Export(format!("renaming NAR: {e}")))?;

    Ok(())
}

fn build_narinfo_text(pi: &PathInfo, nar_url: &str, store_dir: &str) -> Result<String, Error> {
    let mut narinfo = pi.to_narinfo();
    narinfo.url = nar_url;
    narinfo.file_hash = Some(pi.nar_sha256);
    narinfo.file_size = Some(pi.nar_size);
    Ok(narinfo.to_string_with_store_dir(store_dir))
}

fn narinfo_filename_for<S: AsRef<str>>(store_path: &nix_compat::store_path::StorePath<S>) -> String {
    let digest_str = nixbase32::encode(store_path.digest());
    format!("{digest_str}.narinfo")
}

async fn write_nix_cache_info_if_absent(dest: &Path, store_dir: &str) -> Result<(), Error> {
    let info_path = dest.join("nix-cache-info");
    if info_path.exists() {
        return Ok(());
    }

    let content = format!("StoreDir: {store_dir}\nWantMassQuery: 1\nPriority: 30\n");
    let tmp = temp_path_for(&info_path);
    tokio::fs::write(&tmp, content.as_bytes())
        .await
        .map_err(|e| Error::Export(format!("writing nix-cache-info: {e}")))?;
    tokio::fs::rename(&tmp, &info_path)
        .await
        .map_err(|e| Error::Export(format!("renaming nix-cache-info: {e}")))?;
    Ok(())
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

#[cfg(test)]
mod tests {
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;
    use snix_store::path_info::PathInfo;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::StoreConfig;
    use crate::StoreFallbackMode;

    /// Build a minimal signed PathInfo with a single blob in the store.
    async fn make_signed_pathinfo(handle: &StoreHandle, name: &str, content: &[u8]) -> PathInfo {
        // Ingest blob.
        let mut writer = handle.blob_service().open_write().await;
        writer.write_all(content).await.unwrap();
        let blob_digest = writer.close().await.unwrap();

        // Compute NAR hash/size by rendering.
        let node = Node::File {
            digest: blob_digest,
            size: content.len() as u64,
            executable: false,
        };
        let nar_buf = render_nar_bytes_test(handle, &node).await;

        let nar_sha256: [u8; 32] = {
            use sha2::Digest;
            sha2::Sha256::digest(&nar_buf).into()
        };

        // Derive a unique digest from the name so different names produce different store paths.
        let mut digest = [0u8; 20];
        for (i, b) in name.as_bytes().iter().enumerate().take(20) {
            digest[i] = *b;
        }
        let store_path = StorePath::from_name_and_digest_fixed(name, digest).unwrap();

        let mut pi = PathInfo {
            store_path,
            node: node.clone(),
            references: vec![],
            nar_sha256,
            nar_size: nar_buf.len() as u64,
            signatures: vec![],
            deriver: None,
            ca: None,
        };

        sign_pathinfo(&mut pi);
        pi
    }

    fn test_signing_key() -> nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey> {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        nix_compat::narinfo::SigningKey::new("test-key-1".to_string(), secret)
    }

    fn sign_pathinfo(pi: &mut PathInfo) {
        let signing_key = test_signing_key();
        let store_path_ref: nix_compat::store_path::StorePathRef = pi.store_path.as_ref();
        let refs: Vec<nix_compat::store_path::StorePathRef> = pi.references.iter().map(StorePath::as_ref).collect();
        let fp = nix_compat::narinfo::fingerprint(&store_path_ref, &pi.nar_sha256, pi.nar_size, refs.iter());
        let sig = signing_key.sign(fp.as_bytes()).to_owned();
        pi.signatures.push(sig);
    }

    async fn render_nar_bytes_test(handle: &StoreHandle, node: &Node) -> Vec<u8> {
        use snix_store::nar::write_nar;
        use tokio::io::AsyncReadExt;

        let (mut reader, writer) = tokio::io::duplex(64 * 1024);
        let node = node.clone();
        let blob_service = handle.blob_service();
        let directory_service = handle.directory_service();
        let write_task = tokio::spawn(async move { write_nar(writer, &node, blob_service, directory_service).await });
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        write_task.await.unwrap().unwrap();
        buf
    }

    async fn open_test_store(dir: &std::path::Path) -> StoreHandle {
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

    #[tokio::test]
    async fn push_single_signed_path() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store(tmp.path()).await;
        let pi = make_signed_pathinfo(&handle, "hello", b"hello world").await;

        let dest = tmp.path().join("cache");
        let report = export_paths_to_cache_dir(&handle, &[pi.clone()], &dest, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        assert_eq!(report.pushed_count, 1);
        assert_eq!(report.skipped_unsigned_count, 0);
        assert_eq!(report.skipped_already_present_count, 0);
        assert_eq!(report.paths.len(), 1);
        assert_eq!(report.paths[0].store_path, pi.store_path.to_string());

        // Verify nix-cache-info exists.
        let cache_info = std::fs::read_to_string(dest.join("nix-cache-info")).unwrap();
        assert!(cache_info.contains("StoreDir: /nix/store"));
        assert!(cache_info.contains("WantMassQuery: 1"));

        // Verify narinfo file exists and is parseable.
        let digest_str = nixbase32::encode(pi.store_path.digest());
        let narinfo_text = std::fs::read_to_string(dest.join(format!("{digest_str}.narinfo"))).unwrap();
        assert!(narinfo_text.contains("StorePath:"));
        assert!(narinfo_text.contains("NarHash:"));
        assert!(narinfo_text.contains("URL: nar/"));
        assert!(narinfo_text.contains("Sig:"));

        // Verify NAR file exists.
        let nar_hash_b32 = nixbase32::encode(&pi.nar_sha256);
        let nar_path = dest.join("nar").join(format!("{nar_hash_b32}.nar"));
        assert!(nar_path.exists(), "NAR file must exist");

        // Verify NAR sha256 matches.
        let nar_bytes = std::fs::read(&nar_path).unwrap();
        let actual_sha256: [u8; 32] = {
            use sha2::Digest;
            sha2::Sha256::digest(&nar_bytes).into()
        };
        assert_eq!(actual_sha256, pi.nar_sha256, "NAR sha256 must match PathInfo");
    }

    #[tokio::test]
    async fn push_skips_unsigned_by_default() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store(tmp.path()).await;

        let mut pi = make_signed_pathinfo(&handle, "unsigned-pkg", b"data").await;
        pi.signatures.clear(); // make it unsigned

        let dest = tmp.path().join("cache");
        let report = export_paths_to_cache_dir(&handle, &[pi], &dest, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        assert_eq!(report.pushed_count, 0);
        assert_eq!(report.skipped_unsigned_count, 1);
    }

    #[tokio::test]
    async fn push_includes_unsigned_when_trusted() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store(tmp.path()).await;

        let mut pi = make_signed_pathinfo(&handle, "unsigned-pkg", b"data").await;
        pi.signatures.clear();

        let dest = tmp.path().join("cache");
        let report = export_paths_to_cache_dir(&handle, &[pi], &dest, &PushOptions { trust_unsigned: true })
            .await
            .unwrap();

        assert_eq!(report.pushed_count, 1);
        assert_eq!(report.skipped_unsigned_count, 0);
    }

    #[tokio::test]
    async fn push_idempotent_skip() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store(tmp.path()).await;
        let pi = make_signed_pathinfo(&handle, "hello", b"hello world").await;

        let dest = tmp.path().join("cache");
        let opts = PushOptions { trust_unsigned: false };

        // First push.
        let r1 = export_paths_to_cache_dir(&handle, &[pi.clone()], &dest, &opts).await.unwrap();
        assert_eq!(r1.pushed_count, 1);

        // Second push — same path.
        let r2 = export_paths_to_cache_dir(&handle, &[pi], &dest, &opts).await.unwrap();
        assert_eq!(r2.pushed_count, 0);
        assert_eq!(r2.skipped_already_present_count, 1);
    }

    #[tokio::test]
    async fn push_multiple_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store(tmp.path()).await;
        let a = make_signed_pathinfo(&handle, "pkg-a", b"aaa").await;
        let b = make_signed_pathinfo(&handle, "pkg-b", b"bbb").await;
        let c = make_signed_pathinfo(&handle, "pkg-c", b"ccc").await;

        let dest = tmp.path().join("cache");
        let report = export_paths_to_cache_dir(&handle, &[a, b, c], &dest, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        assert_eq!(report.pushed_count, 3);
        assert_eq!(report.paths.len(), 3);

        // Three narinfo files + up to three NARs.
        let narinfo_count = std::fs::read_dir(&dest)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "narinfo"))
            .count();
        assert_eq!(narinfo_count, 3);
    }

    #[tokio::test]
    async fn push_narinfo_references_match() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store(tmp.path()).await;

        let ref_a = StorePath::from_name_and_digest_fixed("ref-a", [1u8; 20]).unwrap();
        let ref_b = StorePath::from_name_and_digest_fixed("ref-b", [2u8; 20]).unwrap();

        let mut pi = make_signed_pathinfo(&handle, "with-refs", b"content").await;
        pi.references = vec![ref_a.clone(), ref_b.clone()];

        // Re-sign after modifying references.
        pi.signatures.clear();
        sign_pathinfo(&mut pi);

        let dest = tmp.path().join("cache");
        export_paths_to_cache_dir(&handle, &[pi.clone()], &dest, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        let digest_str = nixbase32::encode(pi.store_path.digest());
        let narinfo_text = std::fs::read_to_string(dest.join(format!("{digest_str}.narinfo"))).unwrap();

        // References line must contain both ref names.
        assert!(narinfo_text.contains("ref-a"), "narinfo must list ref-a");
        assert!(narinfo_text.contains("ref-b"), "narinfo must list ref-b");

        // NarHash and NarSize must match.
        let expected_hash_str = nixbase32::encode(&pi.nar_sha256);
        assert!(
            narinfo_text.contains(&format!("NarHash: sha256:{expected_hash_str}")),
            "NarHash must match PathInfo"
        );
        assert!(narinfo_text.contains(&format!("NarSize: {}", pi.nar_size)), "NarSize must match PathInfo");
    }

    #[tokio::test]
    async fn push_preserves_existing_nix_cache_info() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store(tmp.path()).await;
        let pi = make_signed_pathinfo(&handle, "hello", b"hello").await;

        let dest = tmp.path().join("cache");
        std::fs::create_dir_all(&dest).unwrap();
        let existing = "StoreDir: /custom/store\nPriority: 10\n";
        std::fs::write(dest.join("nix-cache-info"), existing).unwrap();

        export_paths_to_cache_dir(&handle, &[pi], &dest, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        let cache_info = std::fs::read_to_string(dest.join("nix-cache-info")).unwrap();
        assert_eq!(cache_info, existing, "existing nix-cache-info must be preserved");
    }

    async fn open_test_store_with_prefix(dir: &std::path::Path, store_dir: &str) -> StoreHandle {
        let state_dir = dir.join("state");
        let output_dir = dir.join("output");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        StoreHandle::open(StoreConfig {
            state_dir,
            output_dir,
            remote_cache_url: None,
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: store_dir.to_string(),
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn push_custom_store_dir_narinfo_uses_correct_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        let handle = open_test_store_with_prefix(tmp.path(), "/crunch/store").await;
        let pi = make_signed_pathinfo(&handle, "hello", b"hello world").await;

        let dest = tmp.path().join("cache");
        let report = export_paths_to_cache_dir(&handle, &[pi.clone()], &dest, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        assert_eq!(report.pushed_count, 1);

        // nix-cache-info must reference the custom store dir.
        let cache_info = std::fs::read_to_string(dest.join("nix-cache-info")).unwrap();
        assert!(
            cache_info.contains("StoreDir: /crunch/store"),
            "nix-cache-info must use /crunch/store, got: {cache_info}"
        );

        // narinfo StorePath line must use the custom prefix.
        let digest_str = nixbase32::encode(pi.store_path.digest());
        let narinfo_text = std::fs::read_to_string(dest.join(format!("{digest_str}.narinfo"))).unwrap();
        assert!(
            narinfo_text.contains("StorePath: /crunch/store/"),
            "narinfo must use /crunch/store prefix, got: {narinfo_text}"
        );
        assert!(!narinfo_text.contains("/nix/store/"), "narinfo must not contain /nix/store/, got: {narinfo_text}");
    }
}
