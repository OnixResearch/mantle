//! Store query operations: list, info, verify.
//!
//! These operate on a PathInfoService and are independent of the build
//! engine. Callable from the CLI or as a library.

use futures::StreamExt;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
use snix_castore::import::fs::ingest_path;
use snix_store::nar::{NarCalculationService, SimpleRenderer};
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;

/// Info for a single store path, returned by `store_info`.
#[derive(Debug)]
pub struct PathInfoDetail {
    pub store_path: String,
    pub nar_size: u64,
    pub nar_sha256: Vec<u8>,
    pub deriver: Option<String>,
    pub references: Vec<String>,
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

/// List all store paths from a PathInfoService.
///
/// Returns (store_path, deriver_name, nar_size) tuples.
pub async fn store_list(
    svc: &dyn PathInfoService,
) -> Result<Vec<(String, String, u64)>, Error> {
    let mut stream = svc.list();
    let mut results = Vec::new();
    while let Some(result) = stream.next().await {
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let deriver_name = pi
            .deriver
            .as_ref()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|| "-".to_string());
        results.push((pi.store_path.to_string(), deriver_name, pi.nar_size));
    }
    Ok(results)
}

/// Get detailed PathInfo for paths matching a substring filter.
pub async fn store_info(
    svc: &dyn PathInfoService,
    path_filter: &str,
) -> Result<Vec<PathInfoDetail>, Error> {
    let mut stream = svc.list();
    let mut results = Vec::new();
    while let Some(result) = stream.next().await {
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
            ca: pi.ca.map(|c| format!("{c:?}")),
            node: format!("{:?}", pi.node),
        });
    }
    Ok(results)
}

/// Verify NAR hashes of store paths against what's on disk.
///
/// Optionally filters to paths matching `path_filter`. Returns per-path
/// results (Ok, Missing, or Mismatch).
pub async fn store_verify(
    svc: &dyn PathInfoService,
    path_filter: Option<&str>,
) -> Result<Vec<VerifyResult>, Error> {
    let bs = MemoryBlobService::default();
    let ds = RedbDirectoryService::new_temporary(
        "verify".to_string(),
        RedbDirectoryServiceConfig::default(),
    )
    .map_err(|e| Error::DirectoryService(format!("{e}")))?;

    let mut stream = svc.list();
    let mut results = Vec::new();

    while let Some(result) = stream.next().await {
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();

        if let Some(filter) = path_filter {
            if !sp_str.contains(filter) {
                continue;
            }
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
        let (_nar_size, nar_sha256) = renderer
            .calculate_nar(&node)
            .await
            .map_err(|e| Error::Store(format!("NAR calc: {e}")))?;

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

    Ok(results)
}
