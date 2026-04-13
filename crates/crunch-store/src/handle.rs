//! StoreHandle: unified access to blob, directory, pathinfo, and remote
//! pathinfo services. Consumers receive a StoreHandle — they do not
//! construct or own individual services.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::ObjectStoreBlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::NixHTTPPathInfoService;
use snix_store::pathinfoservice::NixHTTPPathInfoServiceConfig;
use snix_store::pathinfoservice::PathInfoService;
use snix_store::pathinfoservice::RedbPathInfoService;
use snix_store::pathinfoservice::RedbPathInfoServiceConfig;
use tracing::info;
use tracing::info as trace_info;

use crate::ArtifactProvenance;
use crate::CaMappings;
use crate::Error;
use crate::StoredArtifactAttestation;
use crate::StoredClosureAttestation;
use crate::attestation::load_artifact_attestation;
use crate::attestation::load_or_create_runtime_closure_attestation;
use crate::attestation::persist_artifact_attestation;
use crate::export::export_castore_to_disk;

/// Configuration for opening a store.
pub struct StoreConfig {
    /// State directory for persistent data (pathinfo.redb, blobs/, ca_mappings.json).
    pub state_dir: PathBuf,

    /// Physical output directory (from CLI `--store`). Where root build
    /// outputs are exported on disk. Defaults to the store_dir.
    pub output_dir: PathBuf,

    /// Optional remote binary cache URL (e.g., "https://cache.nixos.org").
    pub remote_cache_url: Option<String>,

    /// The logical store prefix for derivation paths (e.g. "/crunch/store"
    /// or "/nix/store" in compat mode). Derivation hashes, output paths,
    /// and sandbox layout all use this prefix.
    pub store_dir: String,
}

/// Return type for a successful cache lookup on a single output.
#[derive(Debug, Clone)]
pub struct CacheHit {
    pub path_info: PathInfo,
    pub node: Node,
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
    /// Physical output dir string (from `--store`).
    output_dir_str: String,
    /// Logical store prefix (e.g. "/crunch/store" or "/nix/store").
    store_dir: String,
    /// Output store path -> Node for outputs built/ingested this session.
    pub output_nodes: HashMap<StorePath<String>, Node>,
    /// Absolute output path -> PathInfo for outputs built this session.
    pub built_outputs: HashMap<String, PathInfo>,
    /// Persistent CA derivation -> output path mapping.
    pub ca_mappings: CaMappings,
}

impl StoreHandle {
    /// Open (or create) a store from a config.
    ///
    /// Initializes blob, directory, and pathinfo services backed by the
    /// state directory. Optionally configures a remote binary cache for
    /// substitution.
    pub async fn open(config: StoreConfig) -> Result<Self, Error> {
        assert!(!config.store_dir.is_empty(), "store_dir must not be empty");
        assert!(config.store_dir.starts_with('/'), "store_dir must be an absolute path");

        let state_dir = &config.state_dir;
        std::fs::create_dir_all(state_dir)
            .map_err(|e| Error::Store(format!("creating state dir {}: {e}", state_dir.display())))?;

        let blob_service = open_blob_service(state_dir)?;
        let directory_service = open_directory_service()?;
        let pathinfo_service = open_pathinfo_service(state_dir).await?;

        let remote_pathinfo = match config.remote_cache_url {
            Some(ref url_str) => {
                match build_remote_pathinfo(url_str, blob_service.clone(), directory_service.clone()) {
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

        let output_dir_str = config.output_dir.to_str().unwrap_or(&config.store_dir).to_string();
        let ca_mappings = CaMappings::load(&config.state_dir);

        // Warn if CA mappings contain paths from a different store prefix.
        // This happens when switching from /nix/store to /crunch/store (or vice versa).
        let prefix_with_slash = format!("{}/", config.store_dir);
        if let Some(first_stale) = ca_mappings.first_key_with_wrong_prefix(&prefix_with_slash) {
            tracing::warn!(
                stale_path = %first_stale,
                expected_prefix = %config.store_dir,
                "CA mappings contain paths from a different store prefix. \
                 Clear the state directory ({}) to reset.",
                config.state_dir.display(),
            );
        }

        Ok(Self {
            blob_service,
            directory_service,
            pathinfo_service,
            remote_pathinfo,
            state_dir: config.state_dir,
            output_dir_str,
            store_dir: config.store_dir,
            output_nodes: HashMap::new(),
            built_outputs: HashMap::new(),
            ca_mappings,
        })
    }

    /// Construct a StoreHandle from pre-built services (for tests).
    pub fn from_services(
        blob_service: Arc<dyn BlobService>,
        directory_service: Arc<dyn DirectoryService>,
        pathinfo_service: Arc<dyn PathInfoService>,
        remote_pathinfo: Option<Arc<dyn PathInfoService>>,
        state_dir: PathBuf,
        output_dir_str: String,
    ) -> Self {
        Self::from_services_with_store_dir(
            blob_service,
            directory_service,
            pathinfo_service,
            remote_pathinfo,
            state_dir,
            output_dir_str,
            nix_compat::store_path::STORE_DIR.to_string(),
        )
    }

    /// Like [StoreHandle::from_services] but with a custom store directory prefix.
    pub fn from_services_with_store_dir(
        blob_service: Arc<dyn BlobService>,
        directory_service: Arc<dyn DirectoryService>,
        pathinfo_service: Arc<dyn PathInfoService>,
        remote_pathinfo: Option<Arc<dyn PathInfoService>>,
        state_dir: PathBuf,
        output_dir_str: String,
        store_dir: String,
    ) -> Self {
        let ca_mappings = CaMappings::load(&state_dir);
        Self {
            blob_service,
            directory_service,
            pathinfo_service,
            remote_pathinfo,
            state_dir,
            output_dir_str,
            store_dir,
            output_nodes: HashMap::new(),
            built_outputs: HashMap::new(),
            ca_mappings,
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

    /// The logical store prefix (e.g. "/crunch/store").
    pub fn store_dir(&self) -> &str {
        &self.store_dir
    }

    /// Check whether the castore has the content referenced by a Node.
    ///
    /// Files: probe blob_service. Directories: probe directory_service.
    /// Symlinks: always present (target is inline in the Node).
    pub async fn castore_has_content(&self, node: &Node) -> Result<bool, Error> {
        // Tiger Style: verify node structural invariant.
        debug_assert!(
            !matches!(node, Node::Directory { size, .. } if *size == u64::MAX),
            "directory size must not be sentinel value"
        );
        match node {
            Node::File { digest, .. } => {
                self.blob_service.has(digest).await.map_err(|e| Error::BlobService(format!("existence check: {e}")))
            }
            Node::Directory { digest, .. } => self
                .directory_service
                .get(digest)
                .await
                .map(|opt| opt.is_some())
                .map_err(|e| Error::DirectoryService(format!("existence check: {e}"))),
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
                return Err(Error::BlobService(format!("read_blob called on non-file node: {other:?}")));
            }
        };

        let mut reader = self
            .blob_service
            .open_read(&digest)
            .await
            .map_err(|e| Error::BlobService(format!("opening blob: {e}")))?
            .ok_or_else(|| {
                Error::BlobService(format!("blob not found: {}", data_encoding::HEXLOWER.encode(digest.as_slice())))
            })?;

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.map_err(|e| Error::BlobService(format!("reading blob: {e}")))?;

        Ok(buf)
    }

    /// The output directory string (from `--store`).
    pub fn output_dir_str(&self) -> &str {
        &self.output_dir_str
    }

    /// Record a CA mapping and persist to disk.
    pub fn insert_ca_mapping(&mut self, drv_abs: &str, output_name: &str, ca_abs: &str) {
        self.ca_mappings.insert(drv_abs, output_name, ca_abs);
        self.ca_mappings.save(&self.state_dir);
    }

    // -- Cache checking --

    /// Check whether all outputs of a derivation are cached.
    ///
    /// Returns `Some(outputs)` on full hit, `None` on any miss.
    /// Checks local PathInfo + castore content, falls back to remote
    /// binary cache substitution.
    pub async fn check_cache(
        &mut self,
        drv_path: &StorePath<String>,
        derivation: &Derivation,
        is_root: bool,
    ) -> Result<Option<HashMap<String, PathInfo>>, Error> {
        assert!(!derivation.outputs.is_empty(), "derivation must have at least one output");

        let mut infos = HashMap::new();
        let drv_abs = drv_path.to_absolute_path_with_prefix(&self.store_dir);

        let is_fod = derivation.outputs.values().any(|o| o.ca_hash.is_some());

        for (output_name, output) in &derivation.outputs {
            let output_path: StorePath<String> = match output.path.as_ref() {
                Some(p) => p.clone(),
                None => match self.ca_mappings.get(&drv_abs, output_name) {
                    Some(ca_abs) => StorePath::from_absolute_path(ca_abs.as_bytes())
                        .map_err(|_| Error::Cache(format!("invalid CA mapping path: {ca_abs}")))?,
                    None => return Ok(None),
                },
            };

            let digest = *output_path.digest();

            let stored =
                self.pathinfo_service.get(digest).await.map_err(|e| Error::Cache(format!("PathInfo lookup: {e}")))?;

            match stored {
                Some(path_info) => {
                    if self.castore_has_content(&path_info.node).await? {
                        persist_artifact_attestation(&self.state_dir, &self.store_dir, output_name, &path_info, None)
                            .await?;
                        self.output_nodes.insert(output_path.clone(), path_info.node.clone());
                        self.built_outputs
                            .insert(output_path.to_absolute_path_with_prefix(&self.output_dir_str), path_info.clone());
                        self.export_output_if_needed(&output_path, &path_info.node, is_root).await?;
                        infos.insert(output_name.clone(), path_info);
                    } else {
                        tracing::warn!(
                            path = %output_path,
                            "PathInfo exists but castore content missing, rebuilding"
                        );
                        return Ok(None);
                    }
                }
                None => {
                    if !is_fod {
                        if let Some(remote_pi) =
                            self.try_substitute_remote(digest, &output_path, output_name, is_root).await?
                        {
                            infos.insert(output_name.clone(), remote_pi);
                            continue;
                        }
                    }
                    return Ok(None);
                }
            }
        }

        Ok(Some(infos))
    }

    /// Try to fetch a single output from the remote binary cache.
    ///
    /// On hit: persists PathInfo locally (write-through), caches the
    /// output node, and returns the PathInfo.
    pub async fn try_substitute_remote(
        &mut self,
        digest: [u8; 20],
        output_path: &StorePath<String>,
        output_name: &str,
        is_root: bool,
    ) -> Result<Option<PathInfo>, Error> {
        assert!(!output_name.is_empty(), "output_name must not be empty");

        let remote = match &self.remote_pathinfo {
            Some(r) => r.clone(),
            None => return Ok(None),
        };

        match remote.get(digest).await {
            Ok(Some(remote_pi)) => {
                trace_info!(
                    path = %output_path,
                    output = %output_name,
                    "substituting from remote cache"
                );

                self.pathinfo_service
                    .put(remote_pi.clone())
                    .await
                    .map_err(|e| Error::Cache(format!("persisting substituted PathInfo: {e}")))?;
                persist_artifact_attestation(&self.state_dir, &self.store_dir, output_name, &remote_pi, None).await?;

                self.output_nodes.insert(output_path.clone(), remote_pi.node.clone());
                self.built_outputs
                    .insert(output_path.to_absolute_path_with_prefix(&self.output_dir_str), remote_pi.clone());
                self.export_output_if_needed(output_path, &remote_pi.node, is_root).await?;

                Ok(Some(remote_pi))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!(
                    path = %output_path,
                    err = %e,
                    "remote cache query failed, building locally"
                );
                Ok(None)
            }
        }
    }

    // -- Persistence + realization --

    /// Persist a signed PathInfo and export it to disk when needed.
    ///
    /// StoreHandle refuses to persist unsigned PathInfos. That keeps the
    /// "always sign before persist" invariant at the storage boundary,
    /// even if a caller constructs the PathInfo itself.
    pub async fn persist_and_export_signed_output(
        &mut self,
        output_name: &str,
        output_path: &StorePath<String>,
        path_info: PathInfo,
        final_node: Node,
        provenance: Option<ArtifactProvenance>,
        is_root: bool,
    ) -> Result<PathInfo, Error> {
        assert!(!output_name.is_empty(), "output_name must not be empty");
        self.persist_pathinfo_and_export(output_name, output_path, path_info, final_node, provenance, is_root)
            .await
    }

    async fn export_output_if_needed(
        &self,
        output_path: &StorePath<String>,
        final_node: &Node,
        is_root: bool,
    ) -> Result<(), Error> {
        if !is_root {
            return Ok(());
        }

        let abs_path = output_path.to_absolute_path_with_prefix(&self.output_dir_str);
        if PathBuf::from(&abs_path).exists() {
            return Ok(());
        }

        match export_castore_to_disk(final_node, &abs_path, &self.blob_service, &self.directory_service).await {
            Ok(()) => Ok(()),
            Err(e) if e.contains("Read-only file system") || e.contains("Permission denied") => {
                tracing::warn!(
                    path = %abs_path,
                    "could not export output to disk (read-only store), \
                     output is available in castore"
                );
                Ok(())
            }
            Err(e) => Err(Error::Export(format!("exporting output {abs_path} to disk: {e}"))),
        }
    }

    /// Common persistence + export logic.
    async fn persist_pathinfo_and_export(
        &mut self,
        output_name: &str,
        output_path: &StorePath<String>,
        path_info: PathInfo,
        final_node: Node,
        provenance: Option<ArtifactProvenance>,
        is_root: bool,
    ) -> Result<PathInfo, Error> {
        if path_info.store_path != *output_path {
            return Err(Error::Store(format!(
                "PathInfo store path mismatch: expected {}, got {}",
                output_path, path_info.store_path,
            )));
        }

        if path_info.signatures.is_empty() {
            return Err(Error::Store(format!("refusing to persist unsigned PathInfo for {output_path}")));
        }

        self.pathinfo_service
            .put(path_info.clone())
            .await
            .map_err(|e| Error::Store(format!("persisting PathInfo: {e}")))?;
        persist_artifact_attestation(&self.state_dir, &self.store_dir, output_name, &path_info, provenance.as_ref())
            .await?;

        let abs_path = output_path.to_absolute_path_with_prefix(&self.output_dir_str);
        self.built_outputs.insert(abs_path, path_info.clone());
        self.output_nodes.insert(output_path.clone(), final_node.clone());
        self.export_output_if_needed(output_path, &final_node, is_root).await?;

        Ok(path_info)
    }

    pub async fn get_artifact_attestation(
        &self,
        store_path: &StorePath<String>,
    ) -> Result<Option<StoredArtifactAttestation>, Error> {
        load_artifact_attestation(&self.state_dir, store_path, &self.store_dir).await
    }

    pub async fn runtime_closure_attestation(
        &self,
        roots: &[StorePath<String>],
    ) -> Result<StoredClosureAttestation, Error> {
        let remote: Option<&dyn PathInfoService> = self.remote_pathinfo.as_ref().map(|svc| svc.as_ref());
        load_or_create_runtime_closure_attestation(
            &self.state_dir,
            &self.store_dir,
            self.pathinfo_service.as_ref(),
            remote,
            roots,
        )
        .await
    }
}

// -- Service construction (moved from main.rs) --

fn open_blob_service(state_dir: &Path) -> Result<Arc<dyn BlobService>, Error> {
    let blob_dir = state_dir.join("blobs");
    std::fs::create_dir_all(&blob_dir)
        .map_err(|e| Error::BlobService(format!("creating blob dir {}: {e}", blob_dir.display())))?;

    let svc = ObjectStoreBlobService::new_local(&blob_dir)
        .map_err(|e| Error::BlobService(format!("opening at {}: {e}", blob_dir.display())))?;

    info!(path = %blob_dir.display(), "blob service opened");
    Ok(Arc::new(svc))
}

fn open_directory_service() -> Result<Arc<dyn DirectoryService>, Error> {
    let svc = RedbDirectoryService::new_temporary("crunch".to_string(), RedbDirectoryServiceConfig {
        path: None,
        read_only: false,
        cache_size: None,
    })
    .map_err(|e| Error::DirectoryService(format!("{e}")))?;
    Ok(Arc::new(svc))
}

async fn open_pathinfo_service(state_dir: &Path) -> Result<Arc<dyn PathInfoService>, Error> {
    let db_path = state_dir.join("pathinfo.redb");
    match RedbPathInfoService::new("crunch".to_string(), RedbPathInfoServiceConfig {
        path: Some(db_path.clone()),
        read_only: false,
        cache_size: None,
    })
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
            let svc = RedbPathInfoService::new_temporary("crunch".to_string(), RedbPathInfoServiceConfig::default())
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
    let nix_url: url::Url = nix_url_str
        .parse()
        .map_err(|e| Error::PathInfoService(format!("invalid substituter URL '{url_str}': {e}")))?;

    let config: NixHTTPPathInfoServiceConfig = nix_url
        .try_into()
        .map_err(|e| Error::PathInfoService(format!("remote cache config for '{url_str}': {e}")))?;

    let svc = NixHTTPPathInfoService::try_build("crunch-remote".to_string(), config, blob_service, directory_service)
        .map_err(|e| Error::PathInfoService(format!("building remote cache client: {e}")))?;

    Ok(Arc::new(svc))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use nix_compat::narinfo::SigningKey;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_store::pathinfoservice::LruPathInfoService;

    use super::*;

    fn test_handle(state_dir: &Path) -> StoreHandle {
        let blob_service = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
        let directory_service = Arc::new(
            RedbDirectoryService::new_temporary("handle-test".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap(),
        ) as Arc<dyn DirectoryService>;
        let pathinfo_service =
            Arc::new(LruPathInfoService::with_capacity("handle-test".to_string(), NonZeroUsize::new(32).unwrap()))
                as Arc<dyn PathInfoService>;

        StoreHandle::from_services_with_store_dir(
            blob_service,
            directory_service,
            pathinfo_service,
            None,
            state_dir.to_path_buf(),
            state_dir.display().to_string(),
            "/nix/store".to_string(),
        )
    }

    fn test_handle_with_remote(state_dir: &Path) -> (StoreHandle, Arc<dyn PathInfoService>) {
        let blob_service = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
        let directory_service = Arc::new(
            RedbDirectoryService::new_temporary(
                "handle-remote-test".to_string(),
                RedbDirectoryServiceConfig::default(),
            )
            .unwrap(),
        ) as Arc<dyn DirectoryService>;
        let local =
            Arc::new(LruPathInfoService::with_capacity("handle-local".to_string(), NonZeroUsize::new(32).unwrap()))
                as Arc<dyn PathInfoService>;
        let remote =
            Arc::new(LruPathInfoService::with_capacity("handle-remote".to_string(), NonZeroUsize::new(32).unwrap()))
                as Arc<dyn PathInfoService>;

        let handle = StoreHandle::from_services_with_store_dir(
            blob_service,
            directory_service,
            local,
            Some(remote.clone()),
            state_dir.to_path_buf(),
            state_dir.display().to_string(),
            "/nix/store".to_string(),
        );
        (handle, remote)
    }

    fn test_output(name: &str, digest_byte: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [digest_byte; 20]).unwrap()
    }

    fn test_signature() -> nix_compat::narinfo::Signature<String> {
        let signing_key =
            SigningKey::new("store-test-1".to_string(), ed25519_dalek::SigningKey::from_bytes(&[3u8; 32]));
        signing_key.sign(b"signed").to_owned()
    }

    fn signed_pathinfo(store_path: StorePath<String>) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [9u8; 32],
            signatures: vec![test_signature()],
            deriver: None,
            ca: None,
        }
    }

    #[tokio::test]
    async fn persist_signed_output_rejects_unsigned_pathinfo() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle(state_dir.path());
        let output_path = test_output("unsigned-path", 7);
        let path_info = PathInfo {
            store_path: output_path.clone(),
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [1u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };

        let err = handle
            .persist_and_export_signed_output(
                "out",
                &output_path,
                path_info,
                Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                None,
                false,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Store(msg) if msg.contains("unsigned PathInfo")));
    }

    #[tokio::test]
    async fn persist_signed_output_rejects_store_path_mismatch() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle(state_dir.path());
        let expected = test_output("expected-path", 8);
        let actual = test_output("actual-path", 9);
        let path_info = signed_pathinfo(actual);

        let err = handle
            .persist_and_export_signed_output(
                "out",
                &expected,
                path_info,
                Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                None,
                false,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Store(msg) if msg.contains("store path mismatch")));
    }

    #[tokio::test]
    async fn persist_signed_output_writes_artifact_attestation() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle(state_dir.path());
        let output_path = test_output("built-path", 10);

        handle
            .persist_and_export_signed_output(
                "dev",
                &output_path,
                signed_pathinfo(output_path.clone()),
                Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                None,
                false,
            )
            .await
            .unwrap();

        let stored = handle.get_artifact_attestation(&output_path).await.unwrap().unwrap();
        assert_eq!(stored.attestation.facts.output_name, "dev");
        assert_eq!(stored.attestation.facts.logical_path, output_path.to_absolute_path());
    }

    #[tokio::test]
    async fn remote_substitution_writes_artifact_attestation() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let output_path = test_output("substituted-path", 11);
        let path_info = signed_pathinfo(output_path.clone());

        remote.put(path_info).await.unwrap();

        let substituted = handle.try_substitute_remote(*output_path.digest(), &output_path, "out", true).await.unwrap();
        assert!(substituted.is_some(), "remote substitution should hit");

        let exported = PathBuf::from(output_path.to_absolute_path_with_prefix(state_dir.path().to_str().unwrap()));
        assert!(exported.exists() || exported.is_symlink(), "remote-substituted root should be exported");

        let stored = handle.get_artifact_attestation(&output_path).await.unwrap().unwrap();
        assert_eq!(stored.attestation.facts.output_name, "out");
        assert_eq!(stored.attestation.facts.logical_path, output_path.to_absolute_path());
    }
}
