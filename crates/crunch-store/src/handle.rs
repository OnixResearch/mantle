//! StoreHandle: unified access to blob, directory, pathinfo, and remote
//! pathinfo services. Consumers receive a StoreHandle — they do not
//! construct or own individual services.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use snix_castore::blobservice::{BlobService, ObjectStoreBlobService};
use snix_castore::directoryservice::{
    DirectoryService, RedbDirectoryService, RedbDirectoryServiceConfig,
};
use snix_castore::Node;
use snix_store::pathinfoservice::{
    NixHTTPPathInfoService, NixHTTPPathInfoServiceConfig,
    PathInfoService, RedbPathInfoService, RedbPathInfoServiceConfig,
};
use tracing::info;

use crate::Error;

/// Configuration for opening a store.
pub struct StoreConfig {
    /// State directory for persistent data (pathinfo.redb, blobs/, ca_mappings.json).
    pub state_dir: PathBuf,

    /// Optional remote binary cache URL (e.g., "https://cache.nixos.org").
    pub remote_cache_url: Option<String>,
}

/// Bundles all store services behind `Arc<dyn ...>`. The single point of
/// contact between crunch-build (or any consumer) and the storage layer.
///
/// Dynamic dispatch via trait objects. Store operations are I/O-bound so
/// the vtable cost is irrelevant.
pub struct StoreHandle {
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
    pathinfo_service: Arc<dyn PathInfoService>,
    remote_pathinfo: Option<Arc<dyn PathInfoService>>,
    state_dir: PathBuf,
}

impl StoreHandle {
    /// Open (or create) a store from a config.
    ///
    /// Initializes blob, directory, and pathinfo services backed by the
    /// state directory. Optionally configures a remote binary cache for
    /// substitution.
    pub async fn open(config: StoreConfig) -> Result<Self, Error> {
        let state_dir = &config.state_dir;
        std::fs::create_dir_all(state_dir)
            .map_err(|e| Error::Store(format!(
                "creating state dir {}: {e}", state_dir.display()
            )))?;

        let blob_service = open_blob_service(state_dir)?;
        let directory_service = open_directory_service()?;
        let pathinfo_service = open_pathinfo_service(state_dir).await?;

        let remote_pathinfo = match config.remote_cache_url {
            Some(ref url_str) => {
                match build_remote_pathinfo(
                    url_str,
                    blob_service.clone(),
                    directory_service.clone(),
                ) {
                    Ok(svc) => {
                        info!(url = %url_str, "binary cache substitution enabled");
                        Some(svc)
                    }
                    Err(e) => {
                        tracing::warn!(
                            url = %url_str,
                            err = %e,
                            "failed to configure remote cache, substitution disabled"
                        );
                        None
                    }
                }
            }
            None => None,
        };

        Ok(Self {
            blob_service,
            directory_service,
            pathinfo_service,
            remote_pathinfo,
            state_dir: config.state_dir,
        })
    }

    /// Construct a StoreHandle from pre-built services (for tests).
    pub fn from_services(
        blob_service: Arc<dyn BlobService>,
        directory_service: Arc<dyn DirectoryService>,
        pathinfo_service: Arc<dyn PathInfoService>,
        remote_pathinfo: Option<Arc<dyn PathInfoService>>,
        state_dir: PathBuf,
    ) -> Self {
        Self {
            blob_service,
            directory_service,
            pathinfo_service,
            remote_pathinfo,
            state_dir,
        }
    }

    /// Arc-cloned blob service.
    pub fn blob_service(&self) -> Arc<dyn BlobService> {
        self.blob_service.clone()
    }

    /// Arc-cloned directory service.
    pub fn directory_service(&self) -> Arc<dyn DirectoryService> {
        self.directory_service.clone()
    }

    /// Arc-cloned local pathinfo service.
    pub fn pathinfo_service(&self) -> Arc<dyn PathInfoService> {
        self.pathinfo_service.clone()
    }

    /// Arc-cloned remote pathinfo service (if configured).
    pub fn remote_pathinfo(&self) -> Option<Arc<dyn PathInfoService>> {
        self.remote_pathinfo.clone()
    }

    /// The state directory backing this store.
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Check whether the castore has the content referenced by a Node.
    ///
    /// Files: probe blob_service. Directories: probe directory_service.
    /// Symlinks: always present (target is inline in the Node).
    pub async fn castore_has_content(&self, node: &Node) -> Result<bool, Error> {
        match node {
            Node::File { digest, .. } => {
                self.blob_service.has(digest).await
                    .map_err(|e| Error::BlobService(format!("existence check: {e}")))
            }
            Node::Directory { digest, .. } => {
                self.directory_service.get(digest).await
                    .map(|opt| opt.is_some())
                    .map_err(|e| Error::DirectoryService(format!("existence check: {e}")))
            }
            Node::Symlink { .. } => Ok(true),
        }
    }

    /// Read the full content of a file blob from castore.
    pub async fn read_blob(&self, node: &Node) -> Result<Vec<u8>, Error> {
        use tokio::io::AsyncReadExt;

        let digest = match node {
            Node::File { digest, size, .. } => {
                const MAX_BLOB_READ: u64 = 8 * 1024 * 1024;
                if *size > MAX_BLOB_READ {
                    return Err(Error::BlobService(format!(
                        "blob too large to read: {size} bytes (limit: {MAX_BLOB_READ})"
                    )));
                }
                digest.clone()
            }
            other => {
                return Err(Error::BlobService(format!(
                    "read_blob called on non-file node: {other:?}"
                )));
            }
        };

        let mut reader = self.blob_service.open_read(&digest).await
            .map_err(|e| Error::BlobService(format!("opening blob: {e}")))?
            .ok_or_else(|| Error::BlobService(format!(
                "blob not found: {}",
                data_encoding::HEXLOWER.encode(digest.as_slice())
            )))?;

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await
            .map_err(|e| Error::BlobService(format!("reading blob: {e}")))?;

        Ok(buf)
    }
}

// -- Service construction (moved from main.rs) --

fn open_blob_service(
    state_dir: &Path,
) -> Result<Arc<dyn BlobService>, Error> {
    let blob_dir = state_dir.join("blobs");
    std::fs::create_dir_all(&blob_dir)
        .map_err(|e| Error::BlobService(format!(
            "creating blob dir {}: {e}", blob_dir.display()
        )))?;

    let svc = ObjectStoreBlobService::new_local(&blob_dir)
        .map_err(|e| Error::BlobService(format!(
            "opening at {}: {e}", blob_dir.display()
        )))?;

    info!(path = %blob_dir.display(), "blob service opened");
    Ok(Arc::new(svc))
}

fn open_directory_service() -> Result<Arc<dyn DirectoryService>, Error> {
    let svc = RedbDirectoryService::new_temporary(
        "crunch".to_string(),
        RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        },
    )
    .map_err(|e| Error::DirectoryService(format!("{e}")))?;
    Ok(Arc::new(svc))
}

async fn open_pathinfo_service(
    state_dir: &Path,
) -> Result<Arc<dyn PathInfoService>, Error> {
    let db_path = state_dir.join("pathinfo.redb");
    match RedbPathInfoService::new(
        "crunch".to_string(),
        RedbPathInfoServiceConfig {
            path: Some(db_path.clone()),
            read_only: false,
            cache_size: None,
        },
    )
    .await
    {
        Ok(svc) => {
            info!(path = %db_path.display(), "PathInfo database opened");
            Ok(Arc::new(svc))
        }
        Err(e) => {
            tracing::warn!(
                path = %db_path.display(),
                err = %e,
                "failed to open PathInfo database, using in-memory fallback"
            );
            let svc = RedbPathInfoService::new_temporary(
                "crunch".to_string(),
                RedbPathInfoServiceConfig::default(),
            )
            .map_err(|e| Error::PathInfoService(format!("in-memory fallback: {e}")))?;
            Ok(Arc::new(svc))
        }
    }
}

fn build_remote_pathinfo(
    url_str: &str,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
) -> Result<Arc<dyn PathInfoService>, Error> {
    // NixHTTPPathInfoServiceConfig::try_from expects "nix+https://..." scheme.
    let nix_url_str = format!("nix+{url_str}");
    let nix_url: url::Url = nix_url_str.parse()
        .map_err(|e| Error::PathInfoService(format!(
            "invalid substituter URL '{url_str}': {e}"
        )))?;

    let config: NixHTTPPathInfoServiceConfig = nix_url.try_into()
        .map_err(|e| Error::PathInfoService(format!(
            "remote cache config for '{url_str}': {e}"
        )))?;

    let svc = NixHTTPPathInfoService::try_build(
        "crunch-remote".to_string(),
        config,
        blob_service,
        directory_service,
    ).map_err(|e| Error::PathInfoService(format!(
        "building remote cache client: {e}"
    )))?;

    Ok(Arc::new(svc))
}
