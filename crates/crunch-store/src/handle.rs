//! StoreHandle: unified access to blob, directory, pathinfo, and remote
//! pathinfo services. Consumers receive a StoreHandle — they do not
//! construct or own individual services.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use reqwest::StatusCode;
use reqwest::redirect::Policy;
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
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::info;
use tracing::info as trace_info;
use url::Url;

use crate::ArtifactProvenance;
use crate::CaMappings;
use crate::Error;
use crate::GcReport;
use crate::GcRootRecord;
use crate::GcRootSource;
use crate::StoreAuditEvent;
use crate::StoreAuditKind;
use crate::StoreFallbackMode;
use crate::StoredArtifactAttestation;
use crate::StoredClosureAttestation;
use crate::attestation::load_artifact_attestation;
use crate::attestation::load_or_create_runtime_closure_attestation;
use crate::attestation::persist_artifact_attestation;
use crate::export::export_castore_to_disk;
use crate::gc;
use crate::roots;

/// Configuration for opening a store.
pub struct StoreConfig {
    /// State directory for persistent data (pathinfo.redb, blobs/, ca_mappings.json).
    pub state_dir: PathBuf,

    /// Physical output directory (from CLI `--store`). Where root build
    /// outputs are exported on disk. Defaults to the store_dir.
    pub output_dir: PathBuf,

    /// Optional remote binary cache URL (e.g., "https://cache.nixos.org").
    pub remote_cache_url: Option<String>,

    /// How strictly store-layer fallbacks are handled.
    pub fallback_mode: StoreFallbackMode,

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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputSubstitutionMode {
    Delta,
    Full,
}

impl OutputSubstitutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Delta => "delta",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OutputSubstitutionReport {
    pub mode: OutputSubstitutionMode,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
}

const DELTA_CAPABILITY_TIMEOUT_MS: u64 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum RemoteDeltaCapability {
    Supported(DeltaCapabilityAdvertisementWire),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum DeltaChunkingAlgorithmWire {
    FastCdc,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum DeltaChunkDigestAlgorithmWire {
    Blake3,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaChunkProfileWire {
    chunking: DeltaChunkingAlgorithmWire,
    chunk_digest: DeltaChunkDigestAlgorithmWire,
    min_chunk_bytes: u32,
    avg_chunk_bytes: u32,
    max_chunk_bytes: u32,
}

impl DeltaChunkProfileWire {
    fn protocol_v1() -> Self {
        Self {
            chunking: DeltaChunkingAlgorithmWire::FastCdc,
            chunk_digest: DeltaChunkDigestAlgorithmWire::Blake3,
            min_chunk_bytes: 131_072,
            avg_chunk_bytes: 262_144,
            max_chunk_bytes: 524_288,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct DeltaNegotiationOfferWire {
    supported_versions: Vec<u32>,
    supported_chunk_profiles: Vec<DeltaChunkProfileWire>,
}

impl DeltaNegotiationOfferWire {
    fn protocol_v1() -> Self {
        Self {
            supported_versions: vec![1],
            supported_chunk_profiles: vec![DeltaChunkProfileWire::protocol_v1()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaHttpEndpointsWire {
    capability_path: String,
    candidate_path: String,
    has_set_path: String,
    stream_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaCapabilityAdvertisementWire {
    supported_versions: Vec<u32>,
    supported_chunk_profiles: Vec<DeltaChunkProfileWire>,
    endpoints: DeltaHttpEndpointsWire,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct DeltaCandidateRequestWire {
    logical_path: String,
    output_name: String,
    client_offer: DeltaNegotiationOfferWire,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaNegotiatedProtocolWire {
    version: u32,
    chunk_profile: DeltaChunkProfileWire,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaChunkRefWire {
    digest: snix_castore::B3Digest,
    size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DeltaArtifactNodeWire {
    Directory {
        digest: snix_castore::B3Digest,
        children: Vec<DeltaArtifactNodeWire>,
    },
    Blob {
        digest: snix_castore::B3Digest,
        size_bytes: u64,
        chunks: Vec<DeltaChunkRefWire>,
    },
    Symlink {
        target: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaOutputFixtureWire {
    output_id: String,
    root: DeltaArtifactNodeWire,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaClosureFixtureWire {
    store_prefix: String,
    outputs: Vec<DeltaOutputFixtureWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaCandidateResponseWire {
    session_id: String,
    negotiated: DeltaNegotiatedProtocolWire,
    sender: DeltaClosureFixtureWire,
    output_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaReceiverManifestWire {
    store_prefix: String,
    known_outputs: Vec<String>,
    known_directories: Vec<snix_castore::B3Digest>,
    known_blobs: Vec<snix_castore::B3Digest>,
    known_chunks: Vec<snix_castore::B3Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaReceiverHasSetWire {
    manifest: DeltaReceiverManifestWire,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct DeltaStreamRequestWire {
    session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DeltaTransferFrameWire {
    Blob {
        digest: snix_castore::B3Digest,
        bytes: Vec<u8>,
    },
    Chunk {
        parent_digest: snix_castore::B3Digest,
        chunk_digest: snix_castore::B3Digest,
        chunk_index: u32,
        bytes: Vec<u8>,
    },
    FinalPathInfo {
        path_info: PathInfo,
    },
}

#[derive(Debug, Clone)]
struct AppliedDeltaChunkWire {
    digest: snix_castore::B3Digest,
    chunk_index: u32,
}

#[derive(Debug, Clone)]
struct AppliedDeltaStreamWire {
    final_path_info: PathInfo,
    chunk_frames_by_parent: HashMap<snix_castore::B3Digest, BTreeMap<u32, AppliedDeltaChunkWire>>,
    blob_frames: u32,
    chunk_frames: u32,
    transferred_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeltaAttemptResult {
    Accepted {
        path_info: Box<PathInfo>,
        report: OutputSubstitutionReport,
    },
    Fallback {
        reason: String,
    },
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
    remote_cache_url: Option<Url>,
    remote_trusted_public_keys: Vec<VerifyingKey>,
    remote_delta_capability: Option<RemoteDeltaCapability>,
    remote_delta_http_client: Option<reqwest::Client>,
    state_dir: PathBuf,
    /// Physical output dir string (from `--store`).
    output_dir_str: String,
    /// Logical store prefix (e.g. "/crunch/store" or "/nix/store").
    store_dir: String,
    /// Startup-time degraded store facts recorded while opening services.
    startup_audit_events: Vec<StoreAuditEvent>,
    /// Output store path -> Node for outputs built/ingested this session.
    pub output_nodes: HashMap<StorePath<String>, Node>,
    /// Absolute output path -> PathInfo for outputs built this session.
    pub built_outputs: HashMap<String, PathInfo>,
    /// Successful remote substitutions recorded for reporting.
    output_substitution_reports: HashMap<StorePath<String>, OutputSubstitutionReport>,
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
        let directory_service = open_directory_service(state_dir).await?;
        let (pathinfo_service, mut startup_audit_events) =
            open_pathinfo_service(state_dir, config.fallback_mode).await?;

        let (remote_pathinfo, remote_cache_url, remote_trusted_public_keys) = match config.remote_cache_url {
            Some(ref url_str) => match Url::parse(url_str) {
                Ok(parsed_url) => match build_remote_pathinfo(url_str, blob_service.clone(), directory_service.clone())
                {
                    Ok(svc) => match parse_remote_trusted_public_keys(url_str) {
                        Ok(trusted_public_keys) => {
                            info!(url = %url_str, "binary cache substitution enabled");
                            (Some(svc), Some(parsed_url), trusted_public_keys)
                        }
                        Err(err) => {
                            tracing::warn!(
                                url = %url_str,
                                err = %err,
                                "failed to parse remote cache trust policy, substitution disabled"
                            );
                            (None, None, Vec::new())
                        }
                    },
                    Err(e) => {
                        tracing::warn!(
                            url = %url_str,
                            err = %e,
                            "failed to configure remote cache, substitution disabled"
                        );
                        (None, None, Vec::new())
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        url = %url_str,
                        err = %e,
                        "failed to parse remote cache URL, substitution disabled"
                    );
                    (None, None, Vec::new())
                }
            },
            None => (None, None, Vec::new()),
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
            remote_cache_url,
            remote_trusted_public_keys,
            remote_delta_capability: None,
            remote_delta_http_client: None,
            state_dir: config.state_dir,
            output_dir_str,
            store_dir: config.store_dir,
            startup_audit_events: std::mem::take(&mut startup_audit_events),
            output_nodes: HashMap::new(),
            built_outputs: HashMap::new(),
            output_substitution_reports: HashMap::new(),
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
            remote_cache_url: None,
            remote_trusted_public_keys: Vec::new(),
            remote_delta_capability: None,
            remote_delta_http_client: None,
            state_dir,
            output_dir_str,
            store_dir,
            startup_audit_events: Vec::new(),
            output_nodes: HashMap::new(),
            built_outputs: HashMap::new(),
            output_substitution_reports: HashMap::new(),
            ca_mappings,
        }
    }

    /// Arc-cloned blob service.
    pub fn blob_service(&self) -> Arc<dyn BlobService> {
        self.blob_service.clone()
    }

    /// Startup-time degraded store facts captured while opening services.
    pub fn startup_audit_events(&self) -> &[StoreAuditEvent] {
        &self.startup_audit_events
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

    pub fn list_retained_roots(&self) -> Result<Vec<GcRootRecord>, Error> {
        roots::list_roots(&self.state_dir)
    }

    pub async fn pin_retained_root(&self, logical_path: &str) -> Result<GcRootRecord, Error> {
        roots::pin_root(&self.state_dir, &self.store_dir, self.pathinfo_service.as_ref(), logical_path).await
    }

    pub fn unpin_retained_root(&self, logical_path: &str) -> Result<Option<GcRootRecord>, Error> {
        roots::unpin_root(
            &self.state_dir,
            roots::LogicalStorePathRef {
                logical_path,
                store_dir: &self.store_dir,
            },
        )
    }

    pub async fn register_retained_root(
        &self,
        store_path: &StorePath<String>,
        source: GcRootSource,
    ) -> Result<GcRootRecord, Error> {
        roots::register_root(&self.state_dir, &self.store_dir, self.pathinfo_service.as_ref(), store_path, source).await
    }

    pub async fn garbage_collect(&mut self, is_dry_run: bool) -> Result<GcReport, Error> {
        gc::run_gc(
            &self.state_dir,
            &self.output_dir_str,
            &self.store_dir,
            self.pathinfo_service.as_ref(),
            self.directory_service.as_ref(),
            self.blob_service.as_ref(),
            &mut self.ca_mappings,
            is_dry_run,
        )
        .await
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
                const MAX_BLOB_READ_BYTES: u64 = 8_388_608;
                if *size > MAX_BLOB_READ_BYTES {
                    return Err(Error::BlobService(format!(
                        "blob too large to read: {size} bytes (limit: {MAX_BLOB_READ_BYTES})"
                    )));
                }
                *digest
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

    pub fn take_output_substitution_report(
        &mut self,
        output_path: &StorePath<String>,
    ) -> Option<OutputSubstitutionReport> {
        self.output_substitution_reports.remove(output_path)
    }

    fn record_output_substitution_report(&mut self, output_path: &StorePath<String>, report: OutputSubstitutionReport) {
        let previous = self.output_substitution_reports.insert(output_path.clone(), report);
        debug_assert!(previous.is_none(), "substitution report should only be recorded once per output path");
    }

    /// Reuse a previously known castore node for `path` when possible.
    ///
    /// Checks the session cache first, then local PathInfo. Returns `None`
    /// when no reusable local node is known or when the referenced castore
    /// content is missing.
    pub async fn cached_node_for_path(&mut self, path: &StorePath<String>) -> Result<Option<Node>, Error> {
        assert!(!path.name().is_empty(), "store path name must not be empty");
        assert!(path.to_string().contains('-'), "store path text must include a digest/name separator");

        if let Some(node) = self.output_nodes.get(path) {
            return Ok(Some(node.clone()));
        }

        let maybe_path_info = self
            .pathinfo_service
            .get(*path.digest())
            .await
            .map_err(|e| Error::Store(format!("PathInfo lookup for {path}: {e}")))?;
        let Some(path_info) = maybe_path_info else {
            return Ok(None);
        };
        if path_info.store_path != *path {
            return Err(Error::Store(format!(
                "PathInfo digest collision for {path}: stored path was {}",
                path_info.store_path
            )));
        }
        if !self.castore_has_content(&path_info.node).await? {
            return Ok(None);
        }

        self.output_nodes.insert(path.clone(), path_info.node.clone());
        Ok(Some(path_info.node))
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
        root_source: Option<GcRootSource>,
    ) -> Result<Option<HashMap<String, PathInfo>>, Error> {
        assert!(!derivation.outputs.is_empty(), "derivation must have at least one output");

        let mut infos = HashMap::new();
        let drv_abs = drv_path.to_absolute_path_with_prefix(&self.store_dir);

        let is_fod = derivation.outputs.values().any(|o| o.ca_hash.is_some());

        for (output_name, output) in &derivation.outputs {
            let output_path: StorePath<String> = match output.path.as_ref() {
                Some(p) => p.clone(),
                None => match self.ca_mappings.get(&drv_abs, output_name) {
                    Some(ca_abs) => StorePath::from_absolute_path_with_prefix(ca_abs.as_bytes(), &self.store_dir)
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
                        if is_root && let Some(source) = root_source {
                            self.register_retained_root(&output_path, source).await?;
                        }
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
                    if !is_fod
                        && let Some(remote_pi) =
                            self.try_substitute_remote(digest, &output_path, output_name, is_root, root_source).await?
                    {
                        infos.insert(output_name.clone(), remote_pi);
                        continue;
                    }
                    return Ok(None);
                }
            }
        }

        Ok(Some(infos))
    }

    fn ensure_remote_delta_http_client(&mut self) -> Result<reqwest::Client, String> {
        if let Some(client) = &self.remote_delta_http_client {
            return Ok(client.clone());
        }

        let client = build_delta_http_client()?;
        self.remote_delta_http_client = Some(client.clone());
        Ok(client)
    }

    async fn probe_remote_delta_capability_if_needed(&mut self, output_path: &StorePath<String>, output_name: &str) {
        if self.remote_delta_capability.is_some() {
            return;
        }

        let Some(cache_url) = self.remote_cache_url.clone() else {
            self.remote_delta_capability = Some(RemoteDeltaCapability::Unsupported);
            return;
        };

        let client = match self.ensure_remote_delta_http_client() {
            Ok(client) => client,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta capability client setup failed, using full-artifact substitution"
                );
                self.remote_delta_capability = Some(RemoteDeltaCapability::Unsupported);
                return;
            }
        };

        let capability = match probe_remote_delta_capability(&client, &cache_url).await {
            Ok(capability) => capability,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta capability probe failed, using full-artifact substitution"
                );
                RemoteDeltaCapability::Unsupported
            }
        };

        if matches!(capability, RemoteDeltaCapability::Supported(_)) {
            trace_info!(
                path = %output_path,
                output = %output_name,
                cache = %cache_url,
                "same-authority delta capability detected; attempting candidate negotiation before full-artifact fallback"
            );
        }

        self.remote_delta_capability = Some(capability);
    }

    async fn local_output_matches_candidate(&mut self, logical_path: &str) -> Result<bool, String> {
        let store_path = match StorePath::from_absolute_path_with_prefix(logical_path.as_bytes(), &self.store_dir) {
            Ok(store_path) => store_path,
            Err(_) => return Ok(false),
        };
        let reused = self
            .cached_node_for_path(&store_path)
            .await
            .map_err(|e| format!("checking local output match for {logical_path}: {e}"))?;
        Ok(reused.is_some())
    }

    async fn collect_known_receiver_content(
        &self,
        root: &DeltaArtifactNodeWire,
        known_directories: &mut HashSet<snix_castore::B3Digest>,
        known_blobs: &mut HashSet<snix_castore::B3Digest>,
        known_chunks: &mut HashSet<snix_castore::B3Digest>,
    ) -> Result<(), String> {
        let mut pending_nodes: Vec<&DeltaArtifactNodeWire> = vec![root];
        while let Some(node) = pending_nodes.pop() {
            match node {
                DeltaArtifactNodeWire::Directory { digest, children } => {
                    let local_directory = self
                        .directory_service
                        .get(digest)
                        .await
                        .map_err(|e| format!("directory lookup for {digest}: {e}"))?;
                    if local_directory.is_some() {
                        known_directories.insert(*digest);
                        continue;
                    }
                    for child in children.iter().rev() {
                        pending_nodes.push(child);
                    }
                }
                DeltaArtifactNodeWire::Blob { digest, chunks, .. } => {
                    let has_blob = self
                        .blob_service
                        .has(digest)
                        .await
                        .map_err(|e| format!("blob presence lookup for {digest}: {e}"))?;
                    if has_blob {
                        known_blobs.insert(*digest);
                        continue;
                    }
                    for chunk in chunks {
                        let has_chunk = self
                            .blob_service
                            .has(&chunk.digest)
                            .await
                            .map_err(|e| format!("chunk presence lookup for {}: {e}", chunk.digest))?;
                        if has_chunk {
                            known_chunks.insert(chunk.digest);
                        }
                    }
                }
                DeltaArtifactNodeWire::Symlink { .. } => {}
            }
        }
        Ok(())
    }

    async fn build_receiver_has_set_for_candidate(
        &mut self,
        candidate: &DeltaCandidateResponseWire,
    ) -> Result<DeltaReceiverHasSetWire, String> {
        if candidate.sender.store_prefix != self.store_dir {
            return Err(format!(
                "store prefix mismatch: sender={} receiver={}",
                candidate.sender.store_prefix, self.store_dir
            ));
        }

        let mut known_outputs = HashSet::<String>::new();
        let mut known_directories = HashSet::<snix_castore::B3Digest>::new();
        let mut known_blobs = HashSet::<snix_castore::B3Digest>::new();
        let mut known_chunks = HashSet::<snix_castore::B3Digest>::new();

        for output in &candidate.sender.outputs {
            if self.local_output_matches_candidate(&output.output_id).await? {
                known_outputs.insert(output.output_id.clone());
                continue;
            }
            self.collect_known_receiver_content(
                &output.root,
                &mut known_directories,
                &mut known_blobs,
                &mut known_chunks,
            )
            .await?;
        }

        let mut known_outputs: Vec<String> = known_outputs.into_iter().collect();
        known_outputs.sort();
        let mut known_directories: Vec<snix_castore::B3Digest> = known_directories.into_iter().collect();
        known_directories.sort_by_key(|digest| digest.to_string());
        let mut known_blobs: Vec<snix_castore::B3Digest> = known_blobs.into_iter().collect();
        known_blobs.sort_by_key(|digest| digest.to_string());
        let mut known_chunks: Vec<snix_castore::B3Digest> = known_chunks.into_iter().collect();
        known_chunks.sort_by_key(|digest| digest.to_string());

        Ok(DeltaReceiverHasSetWire {
            manifest: DeltaReceiverManifestWire {
                store_prefix: self.store_dir.clone(),
                known_outputs,
                known_directories,
                known_blobs,
                known_chunks,
            },
        })
    }

    async fn content_bytes_for_node(&self, node: &Node) -> Result<u64, Error> {
        let mut total_bytes = 0u64;
        let mut pending_nodes = vec![node.clone()];
        while let Some(next_node) = pending_nodes.pop() {
            match next_node {
                Node::File { size, .. } => {
                    total_bytes = total_bytes.saturating_add(size);
                }
                Node::Symlink { .. } => {}
                Node::Directory { digest, .. } => {
                    let directory = self
                        .directory_service
                        .get(&digest)
                        .await
                        .map_err(|e| Error::DirectoryService(format!("loading directory {digest}: {e}")))?
                        .ok_or_else(|| Error::DirectoryService(format!("directory not found: {digest}")))?;
                    for (_name, child) in directory.nodes() {
                        pending_nodes.push(child.clone());
                    }
                }
            }
        }
        Ok(total_bytes)
    }

    async fn write_blob_with_expected_digest(
        &self,
        expected_digest: &snix_castore::B3Digest,
        bytes: &[u8],
    ) -> Result<(), String> {
        let mut writer = self.blob_service.open_write().await;
        writer.write_all(bytes).await.map_err(|e| format!("writing blob {expected_digest}: {e}"))?;
        let written_digest =
            writer.close().await.map_err(|e| format!("closing blob writer for {expected_digest}: {e}"))?;
        if written_digest != *expected_digest {
            return Err(format!("delta blob digest mismatch: expected {expected_digest}, got {written_digest}"));
        }
        Ok(())
    }

    async fn read_blob_by_digest(&self, digest: &snix_castore::B3Digest) -> Result<Option<Vec<u8>>, String> {
        let Some(mut reader) =
            self.blob_service.open_read(digest).await.map_err(|e| format!("opening blob {digest}: {e}"))?
        else {
            return Ok(None);
        };
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await.map_err(|e| format!("reading blob {digest}: {e}"))?;
        Ok(Some(bytes))
    }

    async fn apply_delta_transfer_frames(
        &self,
        frames: Vec<DeltaTransferFrameWire>,
    ) -> Result<AppliedDeltaStreamWire, String> {
        let mut final_path_info = None;
        let mut chunk_frames_by_parent = HashMap::<snix_castore::B3Digest, BTreeMap<u32, AppliedDeltaChunkWire>>::new();
        let mut blob_frames = 0u32;
        let mut chunk_frames = 0u32;
        let mut transferred_bytes = 0u64;

        for frame in frames {
            match frame {
                DeltaTransferFrameWire::Blob { digest, bytes } => {
                    self.write_blob_with_expected_digest(&digest, &bytes).await?;
                    transferred_bytes = transferred_bytes.saturating_add(bytes.len() as u64);
                    blob_frames = blob_frames.saturating_add(1);
                }
                DeltaTransferFrameWire::Chunk {
                    parent_digest,
                    chunk_digest,
                    chunk_index,
                    bytes,
                } => {
                    self.write_blob_with_expected_digest(&chunk_digest, &bytes).await?;
                    transferred_bytes = transferred_bytes.saturating_add(bytes.len() as u64);
                    let previous = chunk_frames_by_parent.entry(parent_digest).or_default().insert(
                        chunk_index,
                        AppliedDeltaChunkWire {
                            digest: chunk_digest,
                            chunk_index,
                        },
                    );
                    if previous.is_some() {
                        return Err(format!(
                            "duplicate chunk frame for parent {} index {}",
                            parent_digest, chunk_index
                        ));
                    }
                    chunk_frames = chunk_frames.saturating_add(1);
                }
                DeltaTransferFrameWire::FinalPathInfo { path_info } => {
                    if final_path_info.replace(path_info).is_some() {
                        return Err("delta stream carried multiple final PathInfo frames".to_string());
                    }
                }
            }
        }

        let final_path_info = final_path_info.ok_or_else(|| "delta stream ended without final PathInfo".to_string())?;
        Ok(AppliedDeltaStreamWire {
            final_path_info,
            chunk_frames_by_parent,
            blob_frames,
            chunk_frames,
            transferred_bytes,
        })
    }

    fn requested_candidate_root<'a>(
        &'a self,
        candidate: &'a DeltaCandidateResponseWire,
        output_path: &StorePath<String>,
    ) -> Result<&'a DeltaArtifactNodeWire, String> {
        let requested_logical_path = output_path.to_absolute_path_with_prefix(&self.store_dir);
        candidate
            .sender
            .outputs
            .iter()
            .find(|output| output.output_id == requested_logical_path)
            .map(|output| &output.root)
            .ok_or_else(|| format!("delta candidate missing requested output {requested_logical_path}"))
    }

    fn ensure_delta_root_matches_pathinfo(
        &self,
        root: &DeltaArtifactNodeWire,
        path_info: &PathInfo,
    ) -> Result<(), String> {
        match (root, &path_info.node) {
            (
                DeltaArtifactNodeWire::Directory { digest, .. },
                Node::Directory {
                    digest: node_digest, ..
                },
            ) if digest == node_digest => Ok(()),
            (
                DeltaArtifactNodeWire::Blob { digest, size_bytes, .. },
                Node::File {
                    digest: node_digest,
                    size,
                    ..
                },
            ) if digest == node_digest && size_bytes == size => Ok(()),
            (DeltaArtifactNodeWire::Symlink { target }, Node::Symlink { target: node_target })
                if target.as_bytes() == node_target.as_ref() =>
            {
                Ok(())
            }
            _ => Err(format!("delta final PathInfo node mismatch for {}", path_info.store_path)),
        }
    }

    async fn ensure_directory_closure_available(&self, root_digest: &snix_castore::B3Digest) -> Result<(), String> {
        let mut pending = vec![*root_digest];
        let mut seen = HashSet::<snix_castore::B3Digest>::new();
        while let Some(directory_digest) = pending.pop() {
            if !seen.insert(directory_digest) {
                continue;
            }
            let directory = self
                .directory_service
                .get(&directory_digest)
                .await
                .map_err(|e| format!("loading directory {directory_digest}: {e}"))?
                .ok_or_else(|| {
                    format!(
                        "delta directory {} missing locally; wire format lacks directory entry names",
                        directory_digest
                    )
                })?;
            for (_name, child) in directory.nodes() {
                match child {
                    Node::Directory { digest, .. } => pending.push(*digest),
                    Node::File { digest, .. } => {
                        let has_blob =
                            self.blob_service.has(digest).await.map_err(|e| format!("checking blob {digest}: {e}"))?;
                        if !has_blob {
                            return Err(format!("delta directory closure missing blob {digest}"));
                        }
                    }
                    Node::Symlink { .. } => {}
                }
            }
        }
        Ok(())
    }

    async fn materialize_chunked_blob_from_local_content(
        &self,
        blob_digest: &snix_castore::B3Digest,
        size_bytes: u64,
        chunks: &[DeltaChunkRefWire],
        applied: &AppliedDeltaStreamWire,
    ) -> Result<(), String> {
        if let Some(streamed_chunks) = applied.chunk_frames_by_parent.get(blob_digest) {
            if streamed_chunks.len() > chunks.len() {
                return Err(format!(
                    "delta stream advertised too many chunks for blob {}: {} > {}",
                    blob_digest,
                    streamed_chunks.len(),
                    chunks.len()
                ));
            }
            for chunk in streamed_chunks.values() {
                let chunk_idx = usize::try_from(chunk.chunk_index)
                    .map_err(|_| format!("chunk index {} overflows usize", chunk.chunk_index))?;
                let expected_chunk = chunks.get(chunk_idx).ok_or_else(|| {
                    format!("delta stream chunk index {} out of range for blob {}", chunk.chunk_index, blob_digest)
                })?;
                if expected_chunk.digest != chunk.digest {
                    return Err(format!(
                        "delta stream chunk digest mismatch for blob {} index {}",
                        blob_digest, chunk.chunk_index
                    ));
                }
            }
        }

        let capacity_bytes = usize::try_from(size_bytes)
            .map_err(|_| format!("blob {} too large to materialize: {} bytes", blob_digest, size_bytes))?;
        let mut bytes = Vec::with_capacity(capacity_bytes);
        for chunk in chunks {
            let chunk_bytes = self
                .read_blob_by_digest(&chunk.digest)
                .await?
                .ok_or_else(|| format!("delta blob {} missing chunk {}", blob_digest, chunk.digest))?;
            let expected_len = usize::try_from(chunk.size_bytes)
                .map_err(|_| format!("chunk {} too large to materialize", chunk.digest))?;
            if chunk_bytes.len() != expected_len {
                return Err(format!(
                    "delta chunk size mismatch for {}: expected {}, got {}",
                    chunk.digest,
                    chunk.size_bytes,
                    chunk_bytes.len()
                ));
            }
            bytes.extend_from_slice(&chunk_bytes);
        }
        if bytes.len() != capacity_bytes {
            return Err(format!(
                "delta blob size mismatch for {}: expected {}, got {}",
                blob_digest,
                size_bytes,
                bytes.len()
            ));
        }
        self.write_blob_with_expected_digest(blob_digest, &bytes).await
    }

    async fn reconstruct_delta_output_node(
        &self,
        root: &DeltaArtifactNodeWire,
        applied: &AppliedDeltaStreamWire,
    ) -> Result<Node, String> {
        self.ensure_delta_root_matches_pathinfo(root, &applied.final_path_info)?;

        match (&applied.final_path_info.node, root) {
            (
                node @ Node::File {
                    digest,
                    size,
                    executable: _,
                },
                DeltaArtifactNodeWire::Blob {
                    digest: root_digest,
                    size_bytes,
                    chunks,
                },
            ) => {
                if digest != root_digest {
                    return Err(format!("delta blob digest mismatch for {}", applied.final_path_info.store_path));
                }
                if size != size_bytes {
                    return Err(format!("delta blob size mismatch for {}", applied.final_path_info.store_path));
                }
                let has_blob =
                    self.blob_service.has(digest).await.map_err(|e| format!("checking blob {digest}: {e}"))?;
                if !has_blob {
                    if chunks.is_empty() {
                        return Err(format!("delta stream missing full blob {}", digest));
                    }
                    self.materialize_chunked_blob_from_local_content(digest, *size, chunks, applied).await?;
                }
                Ok(node.clone())
            }
            (
                node @ Node::Directory { digest, size },
                DeltaArtifactNodeWire::Directory {
                    digest: root_digest, ..
                },
            ) => {
                if digest != root_digest {
                    return Err(format!("delta directory digest mismatch for {}", applied.final_path_info.store_path));
                }
                self.ensure_directory_closure_available(digest).await?;
                let directory = self
                    .directory_service
                    .get(digest)
                    .await
                    .map_err(|e| format!("loading directory {digest}: {e}"))?
                    .ok_or_else(|| format!("delta directory {} missing locally", digest))?;
                if directory.size() != *size {
                    return Err(format!(
                        "delta directory size mismatch for {}: expected {}, got {}",
                        digest,
                        size,
                        directory.size()
                    ));
                }
                Ok(node.clone())
            }
            (node @ Node::Symlink { .. }, DeltaArtifactNodeWire::Symlink { .. }) => Ok(node.clone()),
            _ => Err(format!("delta root kind mismatch for {}", applied.final_path_info.store_path)),
        }
    }

    fn delta_content_bytes(root: &DeltaArtifactNodeWire) -> u64 {
        match root {
            DeltaArtifactNodeWire::Directory { children, .. } => {
                children.iter().fold(0u64, |total, child| total.saturating_add(Self::delta_content_bytes(child)))
            }
            DeltaArtifactNodeWire::Blob { size_bytes, .. } => *size_bytes,
            DeltaArtifactNodeWire::Symlink { .. } => 0,
        }
    }

    fn verify_delta_pathinfo_trusted(&self, path_info: &PathInfo) -> Result<(), String> {
        if self.remote_trusted_public_keys.is_empty() {
            return Ok(());
        }
        if path_info.signatures.is_empty() {
            return Err(format!("delta PathInfo for {} had no signatures", path_info.store_path));
        }
        let fingerprint = compute_pathinfo_fingerprint(path_info);
        let is_trusted = path_info.signatures.iter().any(|signature| {
            let signature_ref = signature.as_ref();
            self.remote_trusted_public_keys.iter().any(|key| key.verify(&fingerprint, &signature_ref))
        });
        if is_trusted {
            Ok(())
        } else {
            Err(format!(
                "delta PathInfo signatures for {} did not match configured trusted keys",
                path_info.store_path
            ))
        }
    }

    async fn attempt_delta_candidate_negotiation(
        &mut self,
        output_path: &StorePath<String>,
        output_name: &str,
        capability: &DeltaCapabilityAdvertisementWire,
        is_root: bool,
        root_source: Option<GcRootSource>,
    ) -> DeltaAttemptResult {
        let Some(cache_url) = self.remote_cache_url.clone() else {
            return DeltaAttemptResult::Fallback {
                reason: "cache_url_missing".to_string(),
            };
        };

        let client = match self.ensure_remote_delta_http_client() {
            Ok(client) => client,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta candidate client setup failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "candidate_client_setup_failed".to_string(),
                };
            }
        };

        let candidate_url = match resolve_same_authority_endpoint_url(&cache_url, &capability.endpoints.candidate_path)
        {
            Ok(url) => url,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta candidate endpoint invalid, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "candidate_endpoint_invalid".to_string(),
                };
            }
        };

        let request = DeltaCandidateRequestWire {
            logical_path: output_path.to_absolute_path_with_prefix(&self.store_dir),
            output_name: output_name.to_string(),
            client_offer: DeltaNegotiationOfferWire::protocol_v1(),
        };

        let response = match client.post(candidate_url.clone()).json(&request).send().await {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta candidate negotiation failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "candidate_request_failed".to_string(),
                };
            }
        };

        if !response.status().is_success() {
            if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::FORBIDDEN {
                trace_info!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    candidate = %candidate_url,
                    status = %response.status(),
                    "delta candidate endpoint unavailable, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "candidate_endpoint_unavailable".to_string(),
                };
            }
            tracing::warn!(
                path = %output_path,
                output = %output_name,
                cache = %cache_url,
                candidate = %candidate_url,
                status = %response.status(),
                "delta candidate negotiation returned unexpected status, using full-artifact substitution"
            );
            return DeltaAttemptResult::Fallback {
                reason: "candidate_status_unexpected".to_string(),
            };
        }

        let candidate = match response.json::<DeltaCandidateResponseWire>().await {
            Ok(candidate) => candidate,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    candidate = %candidate_url,
                    err = %err,
                    "delta candidate response was invalid JSON, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "candidate_json_invalid".to_string(),
                };
            }
        };
        if candidate.output_name != output_name {
            tracing::warn!(
                path = %output_path,
                requested_output = %output_name,
                candidate_output = %candidate.output_name,
                cache = %cache_url,
                candidate = %candidate_url,
                "delta candidate response output name mismatch, using full-artifact substitution"
            );
            return DeltaAttemptResult::Fallback {
                reason: "candidate_output_name_mismatch".to_string(),
            };
        }

        let has_set = match self.build_receiver_has_set_for_candidate(&candidate).await {
            Ok(has_set) => has_set,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    candidate = %candidate_url,
                    err = %err,
                    "building receiver delta manifest failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "receiver_manifest_failed".to_string(),
                };
            }
        };

        let has_set_url = match resolve_same_authority_endpoint_url(&cache_url, &capability.endpoints.has_set_path) {
            Ok(url) => url,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta has-set endpoint invalid, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "has_set_endpoint_invalid".to_string(),
                };
            }
        };

        let has_set_response = match client.post(has_set_url.clone()).json(&has_set).send().await {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    has_set = %has_set_url,
                    err = %err,
                    "delta has-set POST failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "has_set_request_failed".to_string(),
                };
            }
        };

        if !has_set_response.status().is_success() {
            if has_set_response.status() == StatusCode::NOT_FOUND || has_set_response.status() == StatusCode::FORBIDDEN
            {
                trace_info!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    has_set = %has_set_url,
                    status = %has_set_response.status(),
                    "delta has-set endpoint unavailable, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "has_set_endpoint_unavailable".to_string(),
                };
            }
            tracing::warn!(
                path = %output_path,
                output = %output_name,
                cache = %cache_url,
                has_set = %has_set_url,
                status = %has_set_response.status(),
                "delta has-set POST returned unexpected status, using full-artifact substitution"
            );
            return DeltaAttemptResult::Fallback {
                reason: "has_set_status_unexpected".to_string(),
            };
        }

        let stream_url = match resolve_same_authority_endpoint_url(&cache_url, &capability.endpoints.stream_path) {
            Ok(url) => url,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta stream endpoint invalid, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "stream_endpoint_invalid".to_string(),
                };
            }
        };
        let stream_request = DeltaStreamRequestWire {
            session_id: candidate.session_id.clone(),
        };
        let stream_response = match client.post(stream_url.clone()).json(&stream_request).send().await {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    stream = %stream_url,
                    err = %err,
                    "delta stream request failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "stream_request_failed".to_string(),
                };
            }
        };

        if !stream_response.status().is_success() {
            if stream_response.status() == StatusCode::NOT_FOUND || stream_response.status() == StatusCode::FORBIDDEN {
                trace_info!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    stream = %stream_url,
                    status = %stream_response.status(),
                    "delta stream endpoint unavailable, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "stream_endpoint_unavailable".to_string(),
                };
            }
            tracing::warn!(
                path = %output_path,
                output = %output_name,
                cache = %cache_url,
                stream = %stream_url,
                status = %stream_response.status(),
                "delta stream request returned unexpected status, using full-artifact substitution"
            );
            return DeltaAttemptResult::Fallback {
                reason: "stream_status_unexpected".to_string(),
            };
        }

        let stream_frames = match decode_delta_transfer_frames(stream_response).await {
            Ok(frames) => frames,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    stream = %stream_url,
                    err = %err,
                    "delta stream decode failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "stream_decode_failed".to_string(),
                };
            }
        };
        let applied = match self.apply_delta_transfer_frames(stream_frames).await {
            Ok(applied) => applied,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    stream = %stream_url,
                    err = %err,
                    "delta stream application failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "stream_application_failed".to_string(),
                };
            }
        };
        let root = match self.requested_candidate_root(&candidate, output_path) {
            Ok(root) => root,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    candidate = %candidate_url,
                    err = %err,
                    "delta candidate root lookup failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "candidate_root_missing".to_string(),
                };
            }
        };
        let total_content_bytes = Self::delta_content_bytes(root);
        let final_node = match self.reconstruct_delta_output_node(root, &applied).await {
            Ok(node) => node,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    stream = %stream_url,
                    err = %err,
                    "delta reconstruction failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "delta_reconstruction_failed".to_string(),
                };
            }
        };
        if let Err(err) = self.verify_delta_pathinfo_trusted(&applied.final_path_info) {
            tracing::warn!(
                path = %output_path,
                output = %output_name,
                cache = %cache_url,
                stream = %stream_url,
                err = %err,
                "delta final PathInfo was untrusted, using full-artifact substitution"
            );
            return DeltaAttemptResult::Fallback {
                reason: "delta_pathinfo_untrusted".to_string(),
            };
        }

        let path_info = match self
            .persist_and_export_signed_output(
                output_name,
                output_path,
                applied.final_path_info.clone(),
                final_node,
                None,
                is_root,
                root_source,
            )
            .await
        {
            Ok(path_info) => path_info,
            Err(err) => {
                tracing::warn!(
                    path = %output_path,
                    output = %output_name,
                    cache = %cache_url,
                    err = %err,
                    "delta final acceptance failed, using full-artifact substitution"
                );
                return DeltaAttemptResult::Fallback {
                    reason: "delta_acceptance_failed".to_string(),
                };
            }
        };

        trace_info!(
            path = %output_path,
            output = %output_name,
            cache = %cache_url,
            candidate = %candidate_url,
            has_set = %has_set_url,
            stream = %stream_url,
            session = %candidate.session_id,
            blob_frames = applied.blob_frames,
            chunk_frames = applied.chunk_frames,
            transferred_bytes = applied.transferred_bytes,
            reused_bytes = total_content_bytes.saturating_sub(applied.transferred_bytes),
            "delta substitution accepted"
        );
        DeltaAttemptResult::Accepted {
            path_info: Box::new(path_info),
            report: OutputSubstitutionReport {
                mode: OutputSubstitutionMode::Delta,
                transferred_bytes: applied.transferred_bytes,
                reused_bytes: total_content_bytes.saturating_sub(applied.transferred_bytes),
                fallback_reason: None,
            },
        }
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
        root_source: Option<GcRootSource>,
    ) -> Result<Option<PathInfo>, Error> {
        assert!(!output_name.is_empty(), "output_name must not be empty");

        let remote = match &self.remote_pathinfo {
            Some(r) => r.clone(),
            None => return Ok(None),
        };

        self.probe_remote_delta_capability_if_needed(output_path, output_name).await;

        let delta_cap = match self.remote_delta_capability.clone() {
            Some(RemoteDeltaCapability::Supported(capability)) => Some(capability),
            Some(RemoteDeltaCapability::Unsupported) | None => None,
        };
        let mut delta_fallback_reason = None;
        if let Some(capability) = delta_cap.as_ref() {
            match self
                .attempt_delta_candidate_negotiation(output_path, output_name, capability, is_root, root_source)
                .await
            {
                DeltaAttemptResult::Accepted { path_info, report } => {
                    self.record_output_substitution_report(output_path, report);
                    return Ok(Some(*path_info));
                }
                DeltaAttemptResult::Fallback { reason } => {
                    delta_fallback_reason = Some(reason);
                }
            }
        }

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

                let transferred_bytes = match self.content_bytes_for_node(&remote_pi.node).await {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        tracing::warn!(
                            path = %output_path,
                            output = %output_name,
                            err = %err,
                            "could not derive full substitution content-byte count from local castore; falling back to PathInfo.nar_size"
                        );
                        remote_pi.nar_size
                    }
                };
                self.record_output_substitution_report(output_path, OutputSubstitutionReport {
                    mode: OutputSubstitutionMode::Full,
                    transferred_bytes,
                    reused_bytes: 0,
                    fallback_reason: delta_fallback_reason,
                });
                self.output_nodes.insert(output_path.clone(), remote_pi.node.clone());
                self.built_outputs
                    .insert(output_path.to_absolute_path_with_prefix(&self.output_dir_str), remote_pi.clone());
                self.export_output_if_needed(output_path, &remote_pi.node, is_root).await?;
                if is_root && let Some(source) = root_source {
                    self.register_retained_root(output_path, source).await?;
                }

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
        root_source: Option<GcRootSource>,
    ) -> Result<PathInfo, Error> {
        assert!(!output_name.is_empty(), "output_name must not be empty");
        self.persist_pathinfo_and_export(
            output_name,
            output_path,
            path_info,
            final_node,
            provenance,
            is_root,
            root_source,
        )
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
        root_source: Option<GcRootSource>,
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
        if is_root && let Some(source) = root_source {
            self.register_retained_root(output_path, source).await?;
        }

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

fn normalized_cache_base_url(cache_url: &Url) -> Url {
    assert!(cache_url.has_host(), "cache_url must include a host");
    let mut base = cache_url.clone();
    base.set_query(None);
    base.set_fragment(None);
    let mut normalized_path = base.path().to_string();
    if !normalized_path.ends_with('/') {
        normalized_path.push('/');
    }
    base.set_path(&normalized_path);
    base
}

fn same_cache_authority(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn resolve_same_authority_endpoint_url(cache_url: &Url, endpoint: &str) -> Result<Url, String> {
    if endpoint.is_empty() {
        return Err("delta endpoint must not be empty".to_string());
    }

    let resolved = match Url::parse(endpoint) {
        Ok(url) => url,
        Err(url::ParseError::RelativeUrlWithoutBase) => normalized_cache_base_url(cache_url)
            .join(endpoint)
            .map_err(|e| format!("joining delta endpoint URL: {e}"))?,
        Err(e) => return Err(format!("parsing delta endpoint URL: {e}")),
    };

    if !same_cache_authority(cache_url, &resolved) {
        return Err(format!("delta endpoint crosses cache authority: {resolved}"));
    }

    Ok(resolved)
}

fn delta_capability_url(cache_url: &Url) -> Result<Url, String> {
    resolve_same_authority_endpoint_url(cache_url, "delta/capabilities")
}

fn validate_delta_capability_advertisement(
    cache_url: &Url,
    advertisement: &DeltaCapabilityAdvertisementWire,
) -> Result<(), String> {
    if advertisement.supported_versions.is_empty() {
        return Err("delta capability advertisement must include at least one version".to_string());
    }
    if advertisement.supported_chunk_profiles.is_empty() {
        return Err("delta capability advertisement must include at least one chunk profile".to_string());
    }

    let endpoints = &advertisement.endpoints;
    let _ = resolve_same_authority_endpoint_url(cache_url, &endpoints.capability_path)?;
    let _ = resolve_same_authority_endpoint_url(cache_url, &endpoints.candidate_path)?;
    let _ = resolve_same_authority_endpoint_url(cache_url, &endpoints.has_set_path)?;
    let _ = resolve_same_authority_endpoint_url(cache_url, &endpoints.stream_path)?;
    Ok(())
}

fn build_delta_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_millis(DELTA_CAPABILITY_TIMEOUT_MS))
        .build()
        .map_err(|e| format!("building delta capability client: {e}"))
}

async fn decode_delta_transfer_frames(response: reqwest::Response) -> Result<Vec<DeltaTransferFrameWire>, String> {
    const MAX_FRAME_COUNT: usize = 100_000;

    let mut frames = Vec::<DeltaTransferFrameWire>::new();
    let mut buffered = Vec::<u8>::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("reading delta stream chunk: {e}"))?;
        buffered.extend_from_slice(&chunk);

        while let Some(line_end) = buffered.iter().position(|byte| *byte == b'\n') {
            let mut line = buffered.drain(..=line_end).collect::<Vec<u8>>();
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            if line.is_empty() {
                continue;
            }
            let frame = serde_json::from_slice::<DeltaTransferFrameWire>(&line)
                .map_err(|e| format!("decoding delta stream frame: {e}"))?;
            frames.push(frame);
            assert!(
                frames.len() <= MAX_FRAME_COUNT,
                "delta transfer frame count exceeded {MAX_FRAME_COUNT}"
            );
        }
    }

    if !buffered.is_empty() {
        let frame = serde_json::from_slice::<DeltaTransferFrameWire>(&buffered)
            .map_err(|e| format!("decoding trailing delta stream frame: {e}"))?;
        frames.push(frame);
    }
    assert!(
        frames.len() <= MAX_FRAME_COUNT,
        "final delta transfer frame count exceeded {MAX_FRAME_COUNT}"
    );

    Ok(frames)
}

fn parse_remote_trusted_public_keys(url_str: &str) -> Result<Vec<VerifyingKey>, String> {
    assert!(!url_str.is_empty(), "substituter URL must not be empty");

    let nix_url_str = format!("nix+{url_str}");
    let nix_url: Url = nix_url_str.parse().map_err(|e| format!("invalid substituter URL '{url_str}': {e}"))?;

    let mut indexed_keys = Vec::<(u32, String)>::new();
    for (key, value) in nix_url.query_pairs() {
        let Some(index_text) = key.strip_prefix("trusted_public_keys[").and_then(|rest| rest.strip_suffix(']')) else {
            continue;
        };
        let index = index_text
            .parse::<u32>()
            .map_err(|e| format!("parsing trusted public key index '{index_text}': {e}"))?;
        indexed_keys.push((index, value.into_owned()));
    }
    indexed_keys.sort_by_key(|(index, _)| *index);

    let expected_key_count = indexed_keys.len();
    let mut trusted_public_keys = Vec::with_capacity(expected_key_count);
    for (_index, key_text) in indexed_keys {
        let key =
            VerifyingKey::parse(&key_text).map_err(|e| format!("parsing trusted public key '{key_text}': {e}"))?;
        trusted_public_keys.push(key);
    }
    assert_eq!(
        trusted_public_keys.len(),
        expected_key_count,
        "every indexed key must produce a verifying key"
    );
    Ok(trusted_public_keys)
}

fn compute_pathinfo_fingerprint(path_info: &PathInfo) -> String {
    let store_path_ref: StorePathRef = path_info.store_path.as_ref();
    let references = path_info.references.iter().map(|reference| reference.as_ref()).collect::<Vec<_>>();
    fingerprint(&store_path_ref, &path_info.nar_sha256, path_info.nar_size, references.iter())
}

async fn probe_remote_delta_capability(
    client: &reqwest::Client,
    cache_url: &Url,
) -> Result<RemoteDeltaCapability, String> {
    assert!(!cache_url.as_str().is_empty(), "cache_url must not be empty");
    let is_http_scheme = matches!(cache_url.scheme(), "http" | "https");
    assert!(is_http_scheme, "cache_url must use http or https scheme");

    let capability_url = delta_capability_url(cache_url)?;
    let response = client
        .get(capability_url)
        .send()
        .await
        .map_err(|e| format!("requesting delta capability endpoint: {e}"))?;

    if response.status().is_success() {
        let advertisement = response
            .json::<DeltaCapabilityAdvertisementWire>()
            .await
            .map_err(|e| format!("decoding delta capability advertisement: {e}"))?;
        validate_delta_capability_advertisement(cache_url, &advertisement)?;
        return Ok(RemoteDeltaCapability::Supported(advertisement));
    }
    if response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::FORBIDDEN {
        return Ok(RemoteDeltaCapability::Unsupported);
    }

    Err(format!("delta capability endpoint returned {}", response.status()))
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

async fn open_directory_service(state_dir: &Path) -> Result<Arc<dyn DirectoryService>, Error> {
    let db_path = state_dir.join("directories.redb");
    let svc = RedbDirectoryService::new("crunch".to_string(), RedbDirectoryServiceConfig {
        path: Some(db_path.clone()),
        read_only: false,
        cache_size: None,
    })
    .await
    .map_err(|e| Error::DirectoryService(format!("opening {}: {e}", db_path.display())))?;
    Ok(Arc::new(svc))
}

async fn open_pathinfo_service(
    state_dir: &Path,
    fallback_mode: StoreFallbackMode,
) -> Result<(Arc<dyn PathInfoService>, Vec<StoreAuditEvent>), Error> {
    assert!(state_dir.is_absolute(), "state_dir must be absolute");
    assert!(state_dir.is_dir(), "state_dir must exist");

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
            Ok((Arc::new(svc), Vec::new()))
        }
        Err(e) => {
            let detail = format!("failed to open PathInfo database at {}: {e}", db_path.display());
            if fallback_mode.is_strict() {
                return Err(Error::PathInfoFallbackRejected { detail });
            }
            tracing::warn!(
                path = %db_path.display(),
                err = %e,
                "failed to open PathInfo database, using in-memory fallback"
            );
            let svc = RedbPathInfoService::new_temporary("crunch".to_string(), RedbPathInfoServiceConfig::default())
                .map_err(|e| Error::PathInfoService(format!("in-memory fallback: {e}")))?;
            let detail = format!("{detail}; using in-memory fallback");
            Ok((Arc::new(svc), vec![StoreAuditEvent::new(StoreAuditKind::PathInfoFallback, detail)]))
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
    use std::io::Read;
    use std::io::Write;
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::thread::JoinHandle;
    use std::time::Duration;

    use crunch_delta::DeltaHttpEndpoints;
    use crunch_delta::NegotiationOffer;
    use crunch_delta::chunk_profile_wire_v1;
    use nix_compat::narinfo::SigningKey;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_store::nar::NarCalculationService;
    use snix_store::nar::SimpleRenderer;
    use snix_store::nar::write_nar;
    use snix_store::pathinfoservice::LruPathInfoService;
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;

    use super::*;

    fn test_handle(state_dir: &Path) -> StoreHandle {
        test_handle_with_store_dir(state_dir, "/nix/store")
    }

    fn test_handle_with_store_dir(state_dir: &Path, store_dir: &str) -> StoreHandle {
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
            store_dir.to_string(),
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

    fn signed_pathinfo_with_signing_key(
        store_path: StorePath<String>,
        node: Node,
        nar_size: u64,
        nar_sha256: [u8; 32],
        signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    ) -> PathInfo {
        let mut path_info = PathInfo {
            store_path,
            node,
            references: vec![],
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        let fingerprint = compute_pathinfo_fingerprint(&path_info);
        path_info.signatures.push(signing_key.sign(fingerprint.as_bytes()).to_owned());
        path_info
    }

    fn signed_file_pathinfo(
        store_path: StorePath<String>,
        digest: snix_castore::B3Digest,
        size_bytes: u64,
        executable: bool,
    ) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::File {
                digest,
                size: size_bytes,
                executable,
            },
            references: vec![],
            nar_size: size_bytes,
            nar_sha256: [8u8; 32],
            signatures: vec![test_signature()],
            deriver: None,
            ca: None,
        }
    }

    #[derive(Default, Debug)]
    struct DeltaProbeCounts {
        capability_gets: u32,
        candidate_posts: u32,
        has_set_posts: u32,
        stream_posts: u32,
        last_candidate_body: Option<String>,
        last_has_set_body: Option<String>,
        last_stream_body: Option<String>,
    }

    struct DeltaProbeServerConfig {
        capability_status_line: &'static str,
        capability_body: String,
        capability_path: String,
        candidate_status_line: &'static str,
        candidate_body: String,
        candidate_path: String,
        has_set_status_line: &'static str,
        has_set_body: String,
        has_set_path: String,
        stream_status_line: &'static str,
        stream_body: String,
        stream_path: String,
    }

    fn supported_delta_capability_body(candidate_path: &str, has_set_path: &str, stream_path: &str) -> String {
        serde_json::to_string(&DeltaCapabilityAdvertisementWire {
            supported_versions: vec![1],
            supported_chunk_profiles: vec![DeltaChunkProfileWire::protocol_v1()],
            endpoints: DeltaHttpEndpointsWire {
                capability_path: "/delta/capabilities".to_string(),
                candidate_path: candidate_path.to_string(),
                has_set_path: has_set_path.to_string(),
                stream_path: stream_path.to_string(),
            },
        })
        .unwrap()
    }

    fn candidate_response_body(output_id: &str, output_name: &str, root: DeltaArtifactNodeWire) -> String {
        serde_json::to_string(&DeltaCandidateResponseWire {
            session_id: format!("session-{output_name}"),
            negotiated: DeltaNegotiatedProtocolWire {
                version: 1,
                chunk_profile: DeltaChunkProfileWire::protocol_v1(),
            },
            sender: DeltaClosureFixtureWire {
                store_prefix: "/nix/store".to_string(),
                outputs: vec![DeltaOutputFixtureWire {
                    output_id: output_id.to_string(),
                    root,
                }],
            },
            output_name: output_name.to_string(),
        })
        .unwrap()
    }

    async fn store_local_blob(handle: &StoreHandle, bytes: &[u8]) -> snix_castore::B3Digest {
        let mut writer = handle.blob_service.open_write().await;
        writer.write_all(bytes).await.unwrap();
        writer.close().await.unwrap()
    }

    fn delta_stream_body(frames: &[DeltaTransferFrameWire]) -> String {
        let mut body = String::new();
        for frame in frames {
            body.push_str(&serde_json::to_string(frame).unwrap());
            body.push('\n');
        }
        body
    }

    async fn render_nar_bytes(handle: &StoreHandle, node: &Node) -> Vec<u8> {
        let (mut reader, writer) = tokio::io::duplex(64 * 1024);
        let node = node.clone();
        let blob_service = handle.blob_service();
        let directory_service = handle.directory_service();
        let write_task = tokio::spawn(async move { write_nar(writer, &node, blob_service, directory_service).await });
        let mut nar_bytes = Vec::new();
        reader.read_to_end(&mut nar_bytes).await.unwrap();
        write_task.await.unwrap().unwrap();
        nar_bytes
    }

    fn narinfo_body_for(
        path_info: &PathInfo,
        nar_url: &str,
        signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    ) -> String {
        let fingerprint = compute_pathinfo_fingerprint(path_info);
        let signature = signing_key.sign(fingerprint.as_bytes()).to_string();
        let mut body = String::new();
        body.push_str(&format!("StorePath: {}\n", path_info.store_path.to_absolute_path()));
        body.push_str(&format!("URL: {nar_url}\n"));
        body.push_str("Compression: none\n");
        body.push_str(&format!("NarHash: sha256:{}\n", nix_compat::nixbase32::encode(&path_info.nar_sha256)));
        body.push_str(&format!("NarSize: {}\n", path_info.nar_size));
        body.push_str("References: \n");
        body.push_str(&format!("Sig: {signature}\n"));
        body
    }

    #[derive(Default, Debug)]
    struct RealCacheRequestCounts {
        delta: DeltaProbeCounts,
        narinfo_gets: u32,
        nar_gets: u32,
    }

    struct RealDeltaCacheServerConfig {
        delta: DeltaProbeServerConfig,
        narinfo_status_line: &'static str,
        narinfo_path: String,
        narinfo_body: String,
        nar_status_line: &'static str,
        nar_path: String,
        nar_body: Vec<u8>,
    }

    fn spawn_real_delta_cache_server(
        config: RealDeltaCacheServerConfig,
    ) -> (Url, Arc<Mutex<RealCacheRequestCounts>>, Arc<AtomicBool>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let counts = Arc::new(Mutex::new(RealCacheRequestCounts::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let counts_ref = counts.clone();
        let stop_ref = stop.clone();

        let handle = thread::spawn(move || {
            while !stop_ref.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _peer)) => handle_real_delta_cache_connection(&mut stream, &config, &counts_ref),
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("accept failed: {err}"),
                }
            }
        });

        (format!("http://{addr}/").parse().unwrap(), counts, stop, handle)
    }

    fn handle_real_delta_cache_connection(
        stream: &mut TcpStream,
        config: &RealDeltaCacheServerConfig,
        counts: &Arc<Mutex<RealCacheRequestCounts>>,
    ) {
        let mut buf = [0u8; 8192];
        let bytes_read = stream.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..bytes_read]);
        let request_line = request.lines().next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or("/");

        if method == "GET" && path == config.narinfo_path {
            counts.lock().unwrap().narinfo_gets += 1;
            write_http_response(
                stream,
                config.narinfo_status_line,
                b"text/plain; charset=utf-8",
                config.narinfo_body.as_bytes(),
            );
            return;
        }
        if method == "GET" && path == config.nar_path {
            counts.lock().unwrap().nar_gets += 1;
            write_http_response(stream, config.nar_status_line, b"application/x-nix-nar", &config.nar_body);
            return;
        }

        let body_start = request.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(request.len());
        let body_text = request[body_start..].to_string();
        let (status_line, body) = if method == "GET" && path == config.delta.capability_path {
            counts.lock().unwrap().delta.capability_gets += 1;
            (config.delta.capability_status_line, config.delta.capability_body.clone().into_bytes())
        } else if method == "POST" && path == config.delta.candidate_path {
            let mut guard = counts.lock().unwrap();
            guard.delta.candidate_posts += 1;
            guard.delta.last_candidate_body = Some(body_text);
            (config.delta.candidate_status_line, config.delta.candidate_body.clone().into_bytes())
        } else if method == "POST" && path == config.delta.has_set_path {
            let mut guard = counts.lock().unwrap();
            guard.delta.has_set_posts += 1;
            guard.delta.last_has_set_body = Some(body_text);
            (config.delta.has_set_status_line, config.delta.has_set_body.clone().into_bytes())
        } else if method == "POST" && path == config.delta.stream_path {
            let mut guard = counts.lock().unwrap();
            guard.delta.stream_posts += 1;
            guard.delta.last_stream_body = Some(body_text);
            (config.delta.stream_status_line, config.delta.stream_body.clone().into_bytes())
        } else {
            ("HTTP/1.1 404 Not Found", b"missing".to_vec())
        };
        write_http_response(stream, status_line, b"text/plain; charset=utf-8", &body);
    }

    fn write_http_response(stream: &mut TcpStream, status_line: &str, content_type: &[u8], body: &[u8]) {
        let mut headers = Vec::new();
        headers.extend_from_slice(status_line.as_bytes());
        headers.extend_from_slice(b"\r\nContent-Length: ");
        headers.extend_from_slice(body.len().to_string().as_bytes());
        headers.extend_from_slice(b"\r\nContent-Type: ");
        headers.extend_from_slice(content_type);
        headers.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
        stream.write_all(&headers).unwrap();
        stream.write_all(body).unwrap();
        stream.flush().unwrap();
    }

    fn stop_real_delta_cache_server(base_url: &Url, stop: Arc<AtomicBool>, handle: JoinHandle<()>) {
        stop.store(true, Ordering::SeqCst);
        let host = base_url.host_str().unwrap();
        let port = base_url.port_or_known_default().unwrap();
        let _ = TcpStream::connect((host, port));
        handle.join().unwrap();
    }

    fn spawn_delta_probe_server(
        config: DeltaProbeServerConfig,
    ) -> (Url, Arc<Mutex<DeltaProbeCounts>>, Arc<AtomicBool>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let counts = Arc::new(Mutex::new(DeltaProbeCounts::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let counts_ref = counts.clone();
        let stop_ref = stop.clone();

        let handle = thread::spawn(move || {
            while !stop_ref.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _peer)) => handle_delta_probe_connection(&mut stream, &config, &counts_ref),
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("accept failed: {err}"),
                }
            }
        });

        (format!("http://{addr}/").parse().unwrap(), counts, stop, handle)
    }

    fn handle_delta_probe_connection(
        stream: &mut TcpStream,
        config: &DeltaProbeServerConfig,
        counts: &Arc<Mutex<DeltaProbeCounts>>,
    ) {
        let mut buf = [0u8; 4096];
        let bytes_read = stream.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..bytes_read]);
        let request_line = request.lines().next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or("/");

        let (status_line, body) = if method == "GET" && path == config.capability_path {
            counts.lock().unwrap().capability_gets += 1;
            (config.capability_status_line, config.capability_body.as_str())
        } else if method == "POST" && path == config.candidate_path {
            let body_start = request.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(request.len());
            let body_text = request[body_start..].to_string();
            let mut guard = counts.lock().unwrap();
            guard.candidate_posts += 1;
            guard.last_candidate_body = Some(body_text);
            (config.candidate_status_line, config.candidate_body.as_str())
        } else if method == "POST" && path == config.has_set_path {
            let body_start = request.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(request.len());
            let body_text = request[body_start..].to_string();
            let mut guard = counts.lock().unwrap();
            guard.has_set_posts += 1;
            guard.last_has_set_body = Some(body_text);
            (config.has_set_status_line, config.has_set_body.as_str())
        } else if method == "POST" && path == config.stream_path {
            let body_start = request.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(request.len());
            let body_text = request[body_start..].to_string();
            let mut guard = counts.lock().unwrap();
            guard.stream_posts += 1;
            guard.last_stream_body = Some(body_text);
            (config.stream_status_line, config.stream_body.as_str())
        } else {
            ("HTTP/1.1 404 Not Found", "missing")
        };

        let response = format!("{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn stop_delta_probe_server(base_url: &Url, stop: Arc<AtomicBool>, handle: JoinHandle<()>) {
        stop.store(true, Ordering::SeqCst);
        let host = base_url.host_str().unwrap();
        let port = base_url.port_or_known_default().unwrap();
        let _ = TcpStream::connect((host, port));
        handle.join().unwrap();
    }

    #[tokio::test]
    async fn practical_pathinfo_open_fallback_records_audit_event() {
        let state_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(state_dir.path().join("pathinfo.redb")).unwrap();

        let (_svc, audit_events) = open_pathinfo_service(state_dir.path(), StoreFallbackMode::Practical).await.unwrap();

        assert_eq!(audit_events.len(), 1);
        assert_eq!(audit_events[0].kind, StoreAuditKind::PathInfoFallback);
        assert!(audit_events[0].detail.contains("using in-memory fallback"));
    }

    #[tokio::test]
    async fn strict_pathinfo_open_fallback_is_rejected() {
        let state_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(state_dir.path().join("pathinfo.redb")).unwrap();

        let err = match open_pathinfo_service(state_dir.path(), StoreFallbackMode::Strict).await {
            Ok(_) => panic!("strict mode should reject PathInfo fallback"),
            Err(err) => err,
        };

        assert!(matches!(err, Error::PathInfoFallbackRejected { .. }));
        assert!(err.to_string().contains("strict mode does not permit in-memory PathInfo fallback"));
    }

    #[tokio::test]
    async fn cached_node_for_path_reuses_local_pathinfo_node() {
        use snix_store::pathinfoservice::PathInfoService;

        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle(state_dir.path());
        let output_path = test_output("cached-node", 6);
        let path_info = signed_pathinfo(output_path.clone());
        let expected_node = path_info.node.clone();
        handle.pathinfo_service().put(path_info).await.unwrap();

        let reused = handle.cached_node_for_path(&output_path).await.unwrap();

        assert_eq!(reused, Some(expected_node.clone()));
        assert_eq!(handle.output_nodes.get(&output_path), Some(&expected_node));
    }

    #[tokio::test]
    async fn check_cache_accepts_ca_mapping_with_custom_store_prefix() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle_with_store_dir(state_dir.path(), "/crunch/store");
        let drv_path = test_output("ca-demo.drv", 4);
        let output_path = test_output("ca-demo", 5);
        let path_info = signed_pathinfo(output_path.clone());
        handle.pathinfo_service().put(path_info.clone()).await.unwrap();

        let mut outputs = std::collections::BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: None,
        });
        let derivation = Derivation {
            arguments: vec![],
            builder: "/bin/sh".to_string(),
            environment: std::collections::BTreeMap::new(),
            input_derivations: std::collections::BTreeMap::new(),
            input_sources: std::collections::BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        let drv_abs = drv_path.to_absolute_path_with_prefix("/crunch/store");
        let out_abs = output_path.to_absolute_path_with_prefix("/crunch/store");
        handle.insert_ca_mapping(&drv_abs, "out", &out_abs);

        let cached = handle.check_cache(&drv_path, &derivation, false, None).await.unwrap();
        let outputs = cached.expect("CA mapping with matching custom prefix should cache-hit");
        assert_eq!(outputs.get("out").unwrap().store_path, output_path);
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
                None,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Store(msg) if msg.contains("unsigned PathInfo")));
    }

    #[tokio::test]
    async fn failed_persist_does_not_register_root() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle(state_dir.path());
        let output_path = test_output("failed-root", 9);
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
                true,
                Some(crate::GcRootSource::Build),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Store(msg) if msg.contains("unsigned PathInfo")));
        assert!(crate::roots::list_roots(state_dir.path()).unwrap().is_empty());
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
                None,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Store(msg) if msg.contains("store path mismatch")));
    }

    #[tokio::test]
    async fn persist_signed_output_registers_build_root() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle(state_dir.path());
        let output_path = test_output("rooted-build", 12);

        handle
            .persist_and_export_signed_output(
                "out",
                &output_path,
                signed_pathinfo(output_path.clone()),
                Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                None,
                true,
                Some(crate::GcRootSource::Build),
            )
            .await
            .unwrap();

        drop(handle);
        let roots = crate::roots::list_roots(state_dir.path()).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].logical_path, output_path.to_absolute_path());
        assert_eq!(roots[0].source, crate::GcRootSource::Build);
    }

    #[tokio::test]
    async fn persist_signed_output_registers_self_build_root() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut handle = test_handle(state_dir.path());
        let output_path = test_output("rooted-self-build", 13);

        handle
            .persist_and_export_signed_output(
                "out",
                &output_path,
                signed_pathinfo(output_path.clone()),
                Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                None,
                true,
                Some(crate::GcRootSource::SelfBuild),
            )
            .await
            .unwrap();

        drop(handle);
        let roots = crate::roots::list_roots(state_dir.path()).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].source, crate::GcRootSource::SelfBuild);
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
                None,
            )
            .await
            .unwrap();

        let stored = handle.get_artifact_attestation(&output_path).await.unwrap().unwrap();
        assert_eq!(stored.attestation.facts.output_name, "dev");
        assert_eq!(stored.attestation.facts.logical_path, output_path.to_absolute_path());
    }

    #[test]
    fn delta_capability_url_preserves_cache_subpath_without_trailing_slash() {
        let cache_url = Url::parse("https://cache.example.test/binary-cache").unwrap();
        let capability_url = delta_capability_url(&cache_url).unwrap();

        assert_eq!(capability_url.as_str(), "https://cache.example.test/binary-cache/delta/capabilities");
        let candidate_url = resolve_same_authority_endpoint_url(&cache_url, "delta/request").unwrap();
        assert_eq!(candidate_url.as_str(), "https://cache.example.test/binary-cache/delta/request");
    }

    #[test]
    fn delta_capability_url_preserves_cache_subpath_with_trailing_slash() {
        let cache_url = Url::parse("https://cache.example.test/binary-cache/").unwrap();
        let capability_url = delta_capability_url(&cache_url).unwrap();

        assert_eq!(capability_url.as_str(), "https://cache.example.test/binary-cache/delta/capabilities");
    }

    #[test]
    fn delta_capability_url_uses_root_cache_authority() {
        let cache_url = Url::parse("https://cache.example.test/").unwrap();
        let capability_url = delta_capability_url(&cache_url).unwrap();

        assert_eq!(capability_url.as_str(), "https://cache.example.test/delta/capabilities");
    }

    #[test]
    fn resolve_same_authority_endpoint_rejects_cross_origin_urls() {
        let cache_url = Url::parse("https://cache.example.test/binary-cache").unwrap();
        let err =
            resolve_same_authority_endpoint_url(&cache_url, "https://evil.example.test/delta/request").unwrap_err();

        assert!(err.contains("crosses cache authority"));
    }

    #[test]
    fn local_protocol_v1_matches_crunch_delta_wire_contract() {
        let local_profile = DeltaChunkProfileWire::protocol_v1();
        let upstream_profile = chunk_profile_wire_v1();
        assert_eq!(local_profile.min_chunk_bytes, upstream_profile.min_chunk_bytes);
        assert_eq!(local_profile.avg_chunk_bytes, upstream_profile.avg_chunk_bytes);
        assert_eq!(local_profile.max_chunk_bytes, upstream_profile.max_chunk_bytes);

        let local_offer = DeltaNegotiationOfferWire::protocol_v1();
        let upstream_offer = NegotiationOffer::protocol_v1();
        assert_eq!(local_offer.supported_versions, upstream_offer.supported_versions);
        assert_eq!(local_offer.supported_chunk_profiles.len(), upstream_offer.supported_chunk_profiles.len());
        assert_eq!(
            local_offer.supported_chunk_profiles[0].min_chunk_bytes,
            upstream_offer.supported_chunk_profiles[0].min_chunk_bytes
        );
        assert_eq!(
            local_offer.supported_chunk_profiles[0].avg_chunk_bytes,
            upstream_offer.supported_chunk_profiles[0].avg_chunk_bytes
        );
        assert_eq!(
            local_offer.supported_chunk_profiles[0].max_chunk_bytes,
            upstream_offer.supported_chunk_profiles[0].max_chunk_bytes
        );

        let local_endpoints = DeltaHttpEndpointsWire {
            capability_path: "/binary-cache/delta/capabilities".to_string(),
            candidate_path: "/binary-cache/delta/request".to_string(),
            has_set_path: "/binary-cache/delta/has-set".to_string(),
            stream_path: "/binary-cache/delta/stream".to_string(),
        };
        let upstream_endpoints = DeltaHttpEndpoints::under_cache_authority("/binary-cache");
        assert_eq!(local_endpoints.capability_path, upstream_endpoints.capability_path);
        assert_eq!(local_endpoints.candidate_path, upstream_endpoints.candidate_path);
        assert_eq!(local_endpoints.has_set_path, upstream_endpoints.has_set_path);
        assert_eq!(local_endpoints.stream_path, upstream_endpoints.stream_path);
    }

    #[tokio::test]
    async fn remote_substitution_probes_delta_capability_once_per_session() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let reusable_bytes = b"delta-reuse";
        let reusable_digest = store_local_blob(&handle, reusable_bytes).await;
        let requested_logical_path = test_output("delta-probe-first", 20).to_absolute_path();
        let candidate_body = candidate_response_body(&requested_logical_path, "out", DeltaArtifactNodeWire::Blob {
            digest: reusable_digest,
            size_bytes: reusable_bytes.len() as u64,
            chunks: Vec::new(),
        });
        let stream_body = delta_stream_body(&[
            DeltaTransferFrameWire::Blob {
                digest: reusable_digest,
                bytes: reusable_bytes.to_vec(),
            },
            DeltaTransferFrameWire::FinalPathInfo {
                path_info: signed_pathinfo(test_output("delta-stream-path", 28)),
            },
        ]);
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let first_output = test_output("delta-probe-first", 20);
        let second_output = test_output("delta-probe-second", 21);
        remote.put(signed_pathinfo(first_output.clone())).await.unwrap();
        remote.put(signed_pathinfo(second_output.clone())).await.unwrap();

        let first = handle
            .try_substitute_remote(*first_output.digest(), &first_output, "out", false, None)
            .await
            .unwrap();
        let second = handle
            .try_substitute_remote(*second_output.digest(), &second_output, "out", false, None)
            .await
            .unwrap();

        assert!(first.is_some(), "first remote substitution should hit");
        assert!(second.is_some(), "second remote substitution should hit");
        assert!(matches!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Supported(_))));
        let counts = counts.lock().unwrap();
        assert_eq!(counts.capability_gets, 1, "delta capability probe should be cached");
        assert_eq!(counts.candidate_posts, 2, "candidate negotiation should run for each substitution attempt");
        assert_eq!(counts.has_set_posts, 2, "has-set post should run for each successful candidate response");
        assert_eq!(counts.stream_posts, 2, "stream request should run for each successful has-set post");
        let candidate_body = counts.last_candidate_body.as_ref().expect("candidate request body");
        assert!(candidate_body.contains("\"logical_path\":\"/nix/store/"));
        assert!(candidate_body.contains("\"output_name\":\"out\""));
        let has_set_body = counts.last_has_set_body.as_ref().expect("has-set body");
        let has_set: DeltaReceiverHasSetWire = serde_json::from_str(has_set_body).unwrap();
        assert_eq!(has_set.manifest.store_prefix, "/nix/store");
        assert!(has_set.manifest.known_outputs.contains(&requested_logical_path));
        let stream_body = counts.last_stream_body.as_ref().expect("stream body");
        let stream_request: DeltaStreamRequestWire = serde_json::from_str(stream_body).unwrap();
        assert!(stream_request.session_id.starts_with("session-out"));
        drop(counts);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_missing_local_backing_content_is_absent_from_receiver_manifest() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, _remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let output_path = test_output("delta-missing-backing", 33);
        let final_bytes = b"delta-full";
        let final_digest: snix_castore::B3Digest = blake3::hash(final_bytes).as_bytes().into();
        handle
            .pathinfo_service
            .put(signed_file_pathinfo(output_path.clone(), final_digest, final_bytes.len() as u64, false))
            .await
            .unwrap();
        let candidate_body =
            candidate_response_body(&output_path.to_absolute_path(), "out", DeltaArtifactNodeWire::Blob {
                digest: final_digest,
                size_bytes: final_bytes.len() as u64,
                chunks: Vec::new(),
            });
        let stream_body = delta_stream_body(&[
            DeltaTransferFrameWire::Blob {
                digest: final_digest,
                bytes: final_bytes.to_vec(),
            },
            DeltaTransferFrameWire::FinalPathInfo {
                path_info: signed_file_pathinfo(output_path.clone(), final_digest, final_bytes.len() as u64, false),
            },
        ]);
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", true, None).await.unwrap();
        assert!(substituted.is_some(), "delta substitution should still succeed");

        let counts = counts.lock().unwrap();
        let has_set_body = counts.last_has_set_body.as_ref().expect("has-set body");
        let has_set: DeltaReceiverHasSetWire = serde_json::from_str(has_set_body).unwrap();
        assert!(
            has_set.manifest.known_outputs.is_empty(),
            "missing local output backing should not advertise output reuse"
        );
        assert!(
            has_set.manifest.known_blobs.is_empty(),
            "missing local blob backing should not advertise blob reuse"
        );
        assert!(
            has_set.manifest.known_chunks.is_empty(),
            "no local chunks should be advertised when backing content is absent"
        );
        drop(counts);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_receiver_manifest_stays_bounded_to_requested_output() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, _remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let output_path = test_output("delta-bounded-manifest", 34);
        let unrelated_output_path = test_output("delta-unrelated-local", 35);
        let local_chunk = b"delta-local-";
        let local_chunk_digest = store_local_blob(&handle, local_chunk).await;
        let unrelated_bytes = b"unrelated-store-content";
        let unrelated_digest = store_local_blob(&handle, unrelated_bytes).await;
        handle
            .pathinfo_service
            .put(signed_file_pathinfo(
                unrelated_output_path.clone(),
                unrelated_digest,
                unrelated_bytes.len() as u64,
                false,
            ))
            .await
            .unwrap();
        let remote_chunk = b"delta-remote";
        let remote_chunk_digest: snix_castore::B3Digest = blake3::hash(remote_chunk).as_bytes().into();
        let final_bytes = [local_chunk.as_slice(), remote_chunk.as_slice()].concat();
        let final_digest: snix_castore::B3Digest = blake3::hash(&final_bytes).as_bytes().into();
        let candidate_body =
            candidate_response_body(&output_path.to_absolute_path(), "out", DeltaArtifactNodeWire::Blob {
                digest: final_digest,
                size_bytes: final_bytes.len() as u64,
                chunks: vec![
                    DeltaChunkRefWire {
                        digest: local_chunk_digest,
                        size_bytes: local_chunk.len() as u64,
                    },
                    DeltaChunkRefWire {
                        digest: remote_chunk_digest,
                        size_bytes: remote_chunk.len() as u64,
                    },
                ],
            });
        let stream_body = delta_stream_body(&[
            DeltaTransferFrameWire::Chunk {
                parent_digest: final_digest,
                chunk_digest: remote_chunk_digest,
                chunk_index: 1,
                bytes: remote_chunk.to_vec(),
            },
            DeltaTransferFrameWire::FinalPathInfo {
                path_info: signed_file_pathinfo(output_path.clone(), final_digest, final_bytes.len() as u64, false),
            },
        ]);
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", true, None).await.unwrap();
        assert!(substituted.is_some(), "delta substitution should succeed with bounded manifest");

        let counts = counts.lock().unwrap();
        let has_set_body = counts.last_has_set_body.as_ref().expect("has-set body");
        let has_set: DeltaReceiverHasSetWire = serde_json::from_str(has_set_body).unwrap();
        assert_eq!(has_set.manifest.known_outputs.len(), 0, "unrelated outputs must stay out of manifest");
        assert_eq!(has_set.manifest.known_blobs.len(), 0, "unrelated blobs must stay out of manifest");
        assert_eq!(
            has_set.manifest.known_chunks,
            vec![local_chunk_digest],
            "manifest should only include chunks reachable from requested output"
        );
        assert!(
            !has_set.manifest.known_outputs.contains(&unrelated_output_path.to_absolute_path()),
            "manifest should not enumerate unrelated local outputs"
        );
        assert!(
            !has_set.manifest.known_blobs.contains(&unrelated_digest),
            "manifest should not enumerate unrelated local blobs"
        );
        drop(counts);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_closure_scoped_delta_accepts_requested_output_and_reports_reuse() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, _remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let requested_output_path = test_output("delta-closure-requested", 40);
        let sibling_output_path = test_output("delta-closure-sibling", 41);
        let shared_bytes = b"closure-local-";
        let shared_digest = store_local_blob(&handle, shared_bytes).await;
        handle
            .pathinfo_service
            .put(signed_file_pathinfo(sibling_output_path.clone(), shared_digest, shared_bytes.len() as u64, false))
            .await
            .unwrap();
        let remote_bytes = b"closure-remote";
        let remote_chunk_digest: snix_castore::B3Digest = blake3::hash(remote_bytes).as_bytes().into();
        let final_bytes = [shared_bytes.as_slice(), remote_bytes.as_slice()].concat();
        let final_digest: snix_castore::B3Digest = blake3::hash(&final_bytes).as_bytes().into();
        let candidate_body = serde_json::to_string(&DeltaCandidateResponseWire {
            session_id: "session-out".to_string(),
            negotiated: DeltaNegotiatedProtocolWire {
                version: 1,
                chunk_profile: DeltaChunkProfileWire::protocol_v1(),
            },
            sender: DeltaClosureFixtureWire {
                store_prefix: "/nix/store".to_string(),
                outputs: vec![
                    DeltaOutputFixtureWire {
                        output_id: requested_output_path.to_absolute_path(),
                        root: DeltaArtifactNodeWire::Blob {
                            digest: final_digest,
                            size_bytes: final_bytes.len() as u64,
                            chunks: vec![
                                DeltaChunkRefWire {
                                    digest: shared_digest,
                                    size_bytes: shared_bytes.len() as u64,
                                },
                                DeltaChunkRefWire {
                                    digest: remote_chunk_digest,
                                    size_bytes: remote_bytes.len() as u64,
                                },
                            ],
                        },
                    },
                    DeltaOutputFixtureWire {
                        output_id: sibling_output_path.to_absolute_path(),
                        root: DeltaArtifactNodeWire::Blob {
                            digest: shared_digest,
                            size_bytes: shared_bytes.len() as u64,
                            chunks: Vec::new(),
                        },
                    },
                ],
            },
            output_name: "out".to_string(),
        })
        .unwrap();
        let stream_body = delta_stream_body(&[
            DeltaTransferFrameWire::Chunk {
                parent_digest: final_digest,
                chunk_digest: remote_chunk_digest,
                chunk_index: 1,
                bytes: remote_bytes.to_vec(),
            },
            DeltaTransferFrameWire::FinalPathInfo {
                path_info: signed_file_pathinfo(
                    requested_output_path.clone(),
                    final_digest,
                    final_bytes.len() as u64,
                    false,
                ),
            },
        ]);
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let substituted = handle
            .try_substitute_remote(
                *requested_output_path.digest(),
                &requested_output_path,
                "out",
                true,
                Some(GcRootSource::Bootstrap),
            )
            .await
            .unwrap()
            .expect("closure-scoped delta substitution should accept requested output");

        let report = handle.take_output_substitution_report(&requested_output_path).expect("substitution report");
        let counts = counts.lock().unwrap();
        let has_set_body = counts.last_has_set_body.as_ref().expect("has-set body");
        let has_set: DeltaReceiverHasSetWire = serde_json::from_str(has_set_body).unwrap();
        assert!(
            has_set.manifest.known_outputs.contains(&sibling_output_path.to_absolute_path()),
            "closure-scoped manifest should include reusable sibling output"
        );
        drop(counts);

        assert_eq!(substituted.store_path, requested_output_path);
        assert_eq!(report.mode, OutputSubstitutionMode::Delta);
        assert_eq!(report.transferred_bytes, remote_bytes.len() as u64);
        assert_eq!(report.reused_bytes, shared_bytes.len() as u64);
        assert!(report.fallback_reason.is_none());
        let exported_path = requested_output_path.to_absolute_path_with_prefix(handle.output_dir_str());
        assert_eq!(std::fs::read(&exported_path).unwrap(), final_bytes);
        let roots = crate::roots::list_roots(state_dir.path()).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].logical_path, requested_output_path.to_absolute_path());
        assert_eq!(roots[0].source, GcRootSource::Bootstrap);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_stream_failure_falls_back_through_real_http_cache() {
        let remote_state_dir = tempfile::tempdir().unwrap();
        let remote_output_dir = tempfile::tempdir().unwrap();
        let output_path = test_output("delta-real-http-fallback", 32);
        let first_chunk = b"delta-local-";
        let second_chunk = b"delta-remote";
        let first_chunk_digest: snix_castore::B3Digest = blake3::hash(first_chunk).as_bytes().into();
        let second_chunk_digest: snix_castore::B3Digest = blake3::hash(second_chunk).as_bytes().into();
        let final_bytes = [first_chunk.as_slice(), second_chunk.as_slice()].concat();
        let trusted_dalek = ed25519_dalek::SigningKey::from_bytes(&[21u8; 32]);
        let trusted_verify = VerifyingKey::new("real-http-cache-1".to_string(), trusted_dalek.verifying_key());
        let trusted_sign = SigningKey::new("real-http-cache-1".to_string(), trusted_dalek);

        {
            let mut seeded_remote = StoreHandle::open(StoreConfig {
                state_dir: remote_state_dir.path().to_path_buf(),
                output_dir: remote_output_dir.path().to_path_buf(),
                remote_cache_url: None,
                fallback_mode: StoreFallbackMode::Practical,
                store_dir: "/nix/store".to_string(),
            })
            .await
            .unwrap();
            let final_digest = store_local_blob(&seeded_remote, &final_bytes).await;
            let node = Node::File {
                digest: final_digest,
                size: final_bytes.len() as u64,
                executable: false,
            };
            let renderer = SimpleRenderer::new(seeded_remote.blob_service(), seeded_remote.directory_service());
            let (nar_size, nar_sha256) = renderer.calculate_nar(&node).await.unwrap();
            let path_info = signed_pathinfo_with_signing_key(
                output_path.clone(),
                node.clone(),
                nar_size,
                nar_sha256,
                &trusted_sign,
            );
            seeded_remote
                .persist_and_export_signed_output("out", &output_path, path_info, node, None, false, None)
                .await
                .unwrap();
        }

        let persisted_remote = StoreHandle::open(StoreConfig {
            state_dir: remote_state_dir.path().to_path_buf(),
            output_dir: remote_output_dir.path().to_path_buf(),
            remote_cache_url: None,
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
        })
        .await
        .unwrap();
        let path_info = persisted_remote
            .pathinfo_service
            .get(*output_path.digest())
            .await
            .unwrap()
            .expect("persisted pathinfo");
        assert_eq!(path_info.store_path, output_path);
        assert!(!path_info.signatures.is_empty(), "persisted pathinfo should stay signed");
        let final_digest = match path_info.node.clone() {
            Node::File {
                digest,
                size,
                executable,
            } => {
                assert_eq!(size, final_bytes.len() as u64);
                assert!(!executable, "fixture output should be non-executable");
                digest
            }
            other => panic!("expected persisted file node, got {other:?}"),
        };
        let nar_path = "/nar/delta-real-http-fallback.nar".to_string();
        let narinfo_path = format!("/{}.narinfo", nix_compat::nixbase32::encode(output_path.digest()));
        let narinfo_body = narinfo_body_for(&path_info, &nar_path, &trusted_sign);
        let nar_body = render_nar_bytes(&persisted_remote, &path_info.node).await;

        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let candidate_body =
            candidate_response_body(&output_path.to_absolute_path(), "out", DeltaArtifactNodeWire::Blob {
                digest: final_digest,
                size_bytes: final_bytes.len() as u64,
                chunks: vec![
                    DeltaChunkRefWire {
                        digest: first_chunk_digest,
                        size_bytes: first_chunk.len() as u64,
                    },
                    DeltaChunkRefWire {
                        digest: second_chunk_digest,
                        size_bytes: second_chunk.len() as u64,
                    },
                ],
            });
        let stream_body = delta_stream_body(&[DeltaTransferFrameWire::Chunk {
            parent_digest: final_digest,
            chunk_digest: second_chunk_digest,
            chunk_index: 1,
            bytes: second_chunk.to_vec(),
        }]);
        let (base_url, counts, stop, probe_handle) = spawn_real_delta_cache_server(RealDeltaCacheServerConfig {
            delta: DeltaProbeServerConfig {
                capability_status_line: "HTTP/1.1 200 OK",
                capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
                capability_path,
                candidate_status_line: "HTTP/1.1 200 OK",
                candidate_body,
                candidate_path,
                has_set_status_line: "HTTP/1.1 200 OK",
                has_set_body: "has-set-ok".to_string(),
                has_set_path,
                stream_status_line: "HTTP/1.1 200 OK",
                stream_body,
                stream_path,
            },
            narinfo_status_line: "HTTP/1.1 200 OK",
            narinfo_path,
            narinfo_body,
            nar_status_line: "HTTP/1.1 200 OK",
            nar_path,
            nar_body,
        });
        let receiver_state_dir = tempfile::tempdir().unwrap();
        let receiver_output_dir = tempfile::tempdir().unwrap();
        let remote_cache_url = format!(
            "{}?trusted_public_keys[0]={}",
            base_url,
            url::form_urlencoded::byte_serialize(trusted_verify.to_string().as_bytes()).collect::<String>()
        );
        let mut handle = StoreHandle::open(StoreConfig {
            state_dir: receiver_state_dir.path().to_path_buf(),
            output_dir: receiver_output_dir.path().to_path_buf(),
            remote_cache_url: Some(remote_cache_url),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
        })
        .await
        .unwrap();
        store_local_blob(&handle, first_chunk).await;

        let substituted = handle
            .try_substitute_remote(*output_path.digest(), &output_path, "out", true, None)
            .await
            .unwrap()
            .expect("stream failure should restart through full HTTP substitution");

        assert_eq!(substituted.nar_sha256, path_info.nar_sha256);
        let report = handle.take_output_substitution_report(&output_path).expect("substitution report");
        assert_eq!(report.mode, OutputSubstitutionMode::Full);
        assert_eq!(report.transferred_bytes, final_bytes.len() as u64);
        assert_eq!(report.reused_bytes, 0);
        assert_eq!(report.fallback_reason.as_deref(), Some("stream_application_failed"));
        {
            let counts = counts.lock().unwrap();
            assert_eq!(counts.delta.capability_gets, 1);
            assert_eq!(counts.delta.candidate_posts, 1);
            assert_eq!(counts.delta.has_set_posts, 1);
            assert_eq!(counts.delta.stream_posts, 1);
            assert_eq!(counts.narinfo_gets, 1);
            assert_eq!(counts.nar_gets, 1);
        }
        assert!(handle.blob_service.has(&second_chunk_digest).await.unwrap());
        let exported_path = output_path.to_absolute_path_with_prefix(handle.output_dir_str());
        assert_eq!(std::fs::read(&exported_path).unwrap(), final_bytes);
        stop_real_delta_cache_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_accepts_delta_chunk_stream_without_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, _remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let output_path = test_output("delta-accept-hit", 29);
        let first_chunk = b"delta-local-";
        let second_chunk = b"delta-remote";
        let first_chunk_digest = store_local_blob(&handle, first_chunk).await;
        let second_chunk_digest: snix_castore::B3Digest = blake3::hash(second_chunk).as_bytes().into();
        let final_bytes = [first_chunk.as_slice(), second_chunk.as_slice()].concat();
        let final_digest: snix_castore::B3Digest = blake3::hash(&final_bytes).as_bytes().into();
        let candidate_body =
            candidate_response_body(&output_path.to_absolute_path(), "out", DeltaArtifactNodeWire::Blob {
                digest: final_digest,
                size_bytes: final_bytes.len() as u64,
                chunks: vec![
                    DeltaChunkRefWire {
                        digest: first_chunk_digest,
                        size_bytes: first_chunk.len() as u64,
                    },
                    DeltaChunkRefWire {
                        digest: second_chunk_digest,
                        size_bytes: second_chunk.len() as u64,
                    },
                ],
            });
        let stream_body = delta_stream_body(&[
            DeltaTransferFrameWire::Chunk {
                parent_digest: final_digest,
                chunk_digest: second_chunk_digest,
                chunk_index: 1,
                bytes: second_chunk.to_vec(),
            },
            DeltaTransferFrameWire::FinalPathInfo {
                path_info: signed_file_pathinfo(output_path.clone(), final_digest, final_bytes.len() as u64, false),
            },
        ]);
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let substituted = handle
            .try_substitute_remote(*output_path.digest(), &output_path, "out", true, None)
            .await
            .unwrap()
            .expect("delta substitution should accept chunk stream without full fallback");

        assert!(matches!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Supported(_))));
        assert_eq!(substituted.store_path, output_path);
        assert_eq!(substituted.node, Node::File {
            digest: final_digest,
            size: final_bytes.len() as u64,
            executable: false,
        });
        {
            let counts = counts.lock().unwrap();
            assert_eq!(counts.capability_gets, 1);
            assert_eq!(counts.candidate_posts, 1);
            assert_eq!(counts.has_set_posts, 1);
            assert_eq!(counts.stream_posts, 1);
            let stream_body = counts.last_stream_body.as_ref().expect("stream body");
            let stream_request: DeltaStreamRequestWire = serde_json::from_str(stream_body).unwrap();
            assert_eq!(stream_request.session_id, "session-out");
        }

        let exported_path = output_path.to_absolute_path_with_prefix(handle.output_dir_str());
        assert_eq!(std::fs::read(&exported_path).unwrap(), final_bytes);
        assert!(handle.blob_service.has(&final_digest).await.unwrap());
        let local_pathinfo = handle.pathinfo_service.get(*output_path.digest()).await.unwrap().unwrap();
        assert_eq!(local_pathinfo.node, substituted.node);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_untrusted_delta_pathinfo_falls_back_to_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let output_path = test_output("delta-untrusted-hit", 30);
        let first_chunk = b"delta-local-";
        let second_chunk = b"delta-remote";
        let first_chunk_digest = store_local_blob(&handle, first_chunk).await;
        let second_chunk_digest: snix_castore::B3Digest = blake3::hash(second_chunk).as_bytes().into();
        let final_bytes = [first_chunk.as_slice(), second_chunk.as_slice()].concat();
        let final_digest: snix_castore::B3Digest = blake3::hash(&final_bytes).as_bytes().into();

        let trusted_dalek = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let trusted_verify = VerifyingKey::new("cache-trusted-1".to_string(), trusted_dalek.verifying_key());
        let trusted_sign = SigningKey::new("cache-trusted-1".to_string(), trusted_dalek);
        handle.remote_trusted_public_keys = vec![trusted_verify];

        let untrusted_dalek = ed25519_dalek::SigningKey::from_bytes(&[11u8; 32]);
        let untrusted_sign = SigningKey::new("cache-untrusted-1".to_string(), untrusted_dalek);
        let delta_path_info = signed_pathinfo_with_signing_key(
            output_path.clone(),
            Node::File {
                digest: final_digest,
                size: final_bytes.len() as u64,
                executable: false,
            },
            final_bytes.len() as u64,
            [4u8; 32],
            &untrusted_sign,
        );
        let fallback_path_info = signed_pathinfo_with_signing_key(
            output_path.clone(),
            Node::File {
                digest: final_digest,
                size: final_bytes.len() as u64,
                executable: false,
            },
            final_bytes.len() as u64,
            [5u8; 32],
            &trusted_sign,
        );
        remote.put(fallback_path_info.clone()).await.unwrap();

        let candidate_body =
            candidate_response_body(&output_path.to_absolute_path(), "out", DeltaArtifactNodeWire::Blob {
                digest: final_digest,
                size_bytes: final_bytes.len() as u64,
                chunks: vec![
                    DeltaChunkRefWire {
                        digest: first_chunk_digest,
                        size_bytes: first_chunk.len() as u64,
                    },
                    DeltaChunkRefWire {
                        digest: second_chunk_digest,
                        size_bytes: second_chunk.len() as u64,
                    },
                ],
            });
        let stream_body = delta_stream_body(&[
            DeltaTransferFrameWire::Chunk {
                parent_digest: final_digest,
                chunk_digest: second_chunk_digest,
                chunk_index: 1,
                bytes: second_chunk.to_vec(),
            },
            DeltaTransferFrameWire::FinalPathInfo {
                path_info: delta_path_info.clone(),
            },
        ]);
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let substituted = handle
            .try_substitute_remote(*output_path.digest(), &output_path, "out", false, None)
            .await
            .unwrap()
            .expect("untrusted delta PathInfo should fall back to full fetch");

        assert_eq!(substituted.nar_sha256, fallback_path_info.nar_sha256);
        assert_ne!(substituted.nar_sha256, delta_path_info.nar_sha256);
        {
            let counts = counts.lock().unwrap();
            assert_eq!(counts.capability_gets, 1);
            assert_eq!(counts.candidate_posts, 1);
            assert_eq!(counts.has_set_posts, 1);
            assert_eq!(counts.stream_posts, 1);
        }
        let local_pathinfo = handle.pathinfo_service.get(*output_path.digest()).await.unwrap().unwrap();
        assert_eq!(local_pathinfo.nar_sha256, fallback_path_info.nar_sha256);
        assert!(handle.blob_service.has(&second_chunk_digest).await.unwrap());
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_directory_delta_without_local_directory_closure_falls_back_to_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let output_path = test_output("delta-directory-fallback", 31);
        let child_bytes = b"dir-child-payload";
        let child_digest: snix_castore::B3Digest = blake3::hash(child_bytes).as_bytes().into();
        let directory_digest: snix_castore::B3Digest =
            blake3::hash(b"directory-root-missing-locally").as_bytes().into();
        let delta_sign =
            SigningKey::new("cache-directory-1".to_string(), ed25519_dalek::SigningKey::from_bytes(&[12u8; 32]));
        let delta_path_info = signed_pathinfo_with_signing_key(
            output_path.clone(),
            Node::Directory {
                digest: directory_digest,
                size: 1,
            },
            child_bytes.len() as u64,
            [6u8; 32],
            &delta_sign,
        );
        let fallback_path_info = signed_pathinfo_with_signing_key(
            output_path.clone(),
            Node::Directory {
                digest: directory_digest,
                size: 1,
            },
            child_bytes.len() as u64,
            [7u8; 32],
            &delta_sign,
        );
        remote.put(fallback_path_info.clone()).await.unwrap();

        let candidate_body =
            candidate_response_body(&output_path.to_absolute_path(), "out", DeltaArtifactNodeWire::Directory {
                digest: directory_digest,
                children: vec![DeltaArtifactNodeWire::Blob {
                    digest: child_digest,
                    size_bytes: child_bytes.len() as u64,
                    chunks: Vec::new(),
                }],
            });
        let stream_body = delta_stream_body(&[
            DeltaTransferFrameWire::Blob {
                digest: child_digest,
                bytes: child_bytes.to_vec(),
            },
            DeltaTransferFrameWire::FinalPathInfo {
                path_info: delta_path_info.clone(),
            },
        ]);
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let substituted = handle
            .try_substitute_remote(*output_path.digest(), &output_path, "out", false, None)
            .await
            .unwrap()
            .expect("directory delta without local closure should fall back to full fetch");

        assert_eq!(substituted.nar_sha256, fallback_path_info.nar_sha256);
        assert_ne!(substituted.nar_sha256, delta_path_info.nar_sha256);
        assert!(handle.directory_service.get(&directory_digest).await.unwrap().is_none());
        {
            let counts = counts.lock().unwrap();
            assert_eq!(counts.capability_gets, 1);
            assert_eq!(counts.candidate_posts, 1);
            assert_eq!(counts.has_set_posts, 1);
            assert_eq!(counts.stream_posts, 1);
        }
        let local_pathinfo = handle.pathinfo_service.get(*output_path.digest()).await.unwrap().unwrap();
        assert_eq!(local_pathinfo.nar_sha256, fallback_path_info.nar_sha256);
        assert!(handle.blob_service.has(&child_digest).await.unwrap());
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_malformed_stream_json_falls_back_to_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let requested_logical_path = test_output("stream-malformed", 29).to_absolute_path();
        let candidate_body = candidate_response_body(&requested_logical_path, "out", DeltaArtifactNodeWire::Blob {
            digest: blake3::hash(b"stream-malformed").as_bytes().into(),
            size_bytes: 16,
            chunks: Vec::new(),
        });
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body: "not-json\n".to_string(),
            stream_path,
        });
        handle.remote_cache_url = Some(base_url.clone());

        let output_path = test_output("stream-malformed", 29);
        remote.put(signed_pathinfo(output_path.clone())).await.unwrap();

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", false, None).await.unwrap();

        assert!(substituted.is_some(), "malformed stream JSON should still fall back to full-artifact substitution");
        assert!(matches!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Supported(_))));
        let counts = counts.lock().unwrap();
        assert_eq!(counts.capability_gets, 1);
        assert_eq!(counts.candidate_posts, 1);
        assert_eq!(counts.has_set_posts, 1);
        assert_eq!(counts.stream_posts, 1);
        drop(counts);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_404_delta_probe_falls_back_to_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 404 Not Found",
            capability_body: "missing".to_string(),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body: "candidate-ok".to_string(),
            candidate_path: "/delta/request".to_string(),
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path: "/delta/has-set".to_string(),
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body: "stream-ok".to_string(),
            stream_path: "/delta/stream".to_string(),
        });
        handle.remote_cache_url = Some(base_url.clone());

        let output_path = test_output("legacy-cache-hit", 22);
        remote.put(signed_pathinfo(output_path.clone())).await.unwrap();

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", false, None).await.unwrap();

        assert!(substituted.is_some(), "legacy cache should still hit via full-artifact substitution");
        assert_eq!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Unsupported));
        let counts = counts.lock().unwrap();
        assert_eq!(counts.capability_gets, 1);
        assert_eq!(counts.candidate_posts, 0);
        assert_eq!(counts.has_set_posts, 0);
        assert_eq!(counts.stream_posts, 0);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_probe_error_falls_back_to_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 500 Internal Server Error",
            capability_body: "broken".to_string(),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body: "candidate-ok".to_string(),
            candidate_path: "/delta/request".to_string(),
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path: "/delta/has-set".to_string(),
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body: "stream-ok".to_string(),
            stream_path: "/delta/stream".to_string(),
        });
        handle.remote_cache_url = Some(base_url.clone());

        let output_path = test_output("probe-error-hit", 23);
        remote.put(signed_pathinfo(output_path.clone())).await.unwrap();

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", false, None).await.unwrap();

        assert!(substituted.is_some(), "probe errors should still fall back to full-artifact substitution");
        assert_eq!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Unsupported));
        let counts = counts.lock().unwrap();
        assert_eq!(counts.capability_gets, 1);
        assert_eq!(counts.candidate_posts, 0);
        assert_eq!(counts.has_set_posts, 0);
        assert_eq!(counts.stream_posts, 0);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_malformed_delta_capability_json_falls_back_to_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: "not-json".to_string(),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body: "candidate-ok".to_string(),
            candidate_path: "/delta/request".to_string(),
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path: "/delta/has-set".to_string(),
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body: "stream-ok".to_string(),
            stream_path: "/delta/stream".to_string(),
        });
        handle.remote_cache_url = Some(base_url.clone());

        let output_path = test_output("probe-invalid-json-hit", 26);
        remote.put(signed_pathinfo(output_path.clone())).await.unwrap();

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", false, None).await.unwrap();

        assert!(substituted.is_some(), "malformed probe JSON should still fall back to full-artifact substitution");
        assert_eq!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Unsupported));
        let counts = counts.lock().unwrap();
        assert_eq!(counts.capability_gets, 1);
        assert_eq!(counts.candidate_posts, 0);
        assert_eq!(counts.has_set_posts, 0);
        assert_eq!(counts.stream_posts, 0);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_without_cache_url_skips_probe_and_full_fetches() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let output_path = test_output("no-url-hit", 24);
        remote.put(signed_pathinfo(output_path.clone())).await.unwrap();

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", false, None).await.unwrap();

        assert!(substituted.is_some(), "missing probe URL should still allow full-artifact substitution");
        assert_eq!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Unsupported));
    }

    #[tokio::test]
    async fn remote_substitution_cross_authority_capability_falls_back_to_full_fetch() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "https://evil.example.test/delta/request".to_string();
        let (base_url, counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, "/delta/has-set", "/delta/stream"),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body: "candidate-ok".to_string(),
            candidate_path: "/delta/request".to_string(),
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path: "/delta/has-set".to_string(),
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body: "stream-ok".to_string(),
            stream_path: "/delta/stream".to_string(),
        });
        handle.remote_cache_url = Some(base_url.clone());

        let output_path = test_output("cross-authority-hit", 25);
        remote.put(signed_pathinfo(output_path.clone())).await.unwrap();

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", false, None).await.unwrap();

        assert!(
            substituted.is_some(),
            "cross-authority delta ads should still fall back to full-artifact substitution"
        );
        assert_eq!(handle.remote_delta_capability, Some(RemoteDeltaCapability::Unsupported));
        let counts = counts.lock().unwrap();
        assert_eq!(counts.capability_gets, 1);
        assert_eq!(counts.candidate_posts, 0);
        assert_eq!(counts.has_set_posts, 0);
        assert_eq!(counts.stream_posts, 0);
        drop(counts);
        stop_delta_probe_server(&base_url, stop, probe_handle);
    }

    #[tokio::test]
    async fn remote_substitution_registers_bootstrap_root() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let output_path = test_output("bootstrap-root", 14);
        let path_info = signed_pathinfo(output_path.clone());

        remote.put(path_info).await.unwrap();

        let substituted = handle
            .try_substitute_remote(
                *output_path.digest(),
                &output_path,
                "out",
                true,
                Some(crate::GcRootSource::Bootstrap),
            )
            .await
            .unwrap();
        assert!(substituted.is_some(), "remote substitution should hit");

        drop(handle);
        let roots = crate::roots::list_roots(state_dir.path()).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].source, crate::GcRootSource::Bootstrap);
    }

    #[tokio::test]
    async fn remote_substitution_writes_artifact_attestation() {
        let state_dir = tempfile::tempdir().unwrap();
        let (mut handle, remote) = test_handle_with_remote(state_dir.path());
        let output_path = test_output("substituted-path", 15);
        let path_info = signed_pathinfo(output_path.clone());

        remote.put(path_info).await.unwrap();

        let substituted =
            handle.try_substitute_remote(*output_path.digest(), &output_path, "out", true, None).await.unwrap();
        assert!(substituted.is_some(), "remote substitution should hit");

        let exported = PathBuf::from(output_path.to_absolute_path_with_prefix(state_dir.path().to_str().unwrap()));
        assert!(exported.exists() || exported.is_symlink(), "remote-substituted root should be exported");

        let stored = handle.get_artifact_attestation(&output_path).await.unwrap().unwrap();
        assert_eq!(stored.attestation.facts.output_name, "out");
        assert_eq!(stored.attestation.facts.logical_path, output_path.to_absolute_path());
    }

    #[tokio::test]
    async fn delta_and_full_substitution_record_same_attestation_and_root_metadata() {
        let output_path = test_output("delta-attestation-parity", 37);
        let path_info = signed_pathinfo(output_path.clone());
        let capability_path = "/delta/capabilities".to_string();
        let candidate_path = "/delta/request".to_string();
        let has_set_path = "/delta/has-set".to_string();
        let stream_path = "/delta/stream".to_string();
        let candidate_body =
            candidate_response_body(&output_path.to_absolute_path(), "out", DeltaArtifactNodeWire::Symlink {
                target: "target".to_string(),
            });
        let stream_body = delta_stream_body(&[DeltaTransferFrameWire::FinalPathInfo {
            path_info: path_info.clone(),
        }]);

        let delta_state_dir = tempfile::tempdir().unwrap();
        let (mut delta_handle, _delta_remote) = test_handle_with_remote(delta_state_dir.path());
        let (base_url, _counts, stop, probe_handle) = spawn_delta_probe_server(DeltaProbeServerConfig {
            capability_status_line: "HTTP/1.1 200 OK",
            capability_body: supported_delta_capability_body(&candidate_path, &has_set_path, &stream_path),
            capability_path,
            candidate_status_line: "HTTP/1.1 200 OK",
            candidate_body,
            candidate_path,
            has_set_status_line: "HTTP/1.1 200 OK",
            has_set_body: "has-set-ok".to_string(),
            has_set_path,
            stream_status_line: "HTTP/1.1 200 OK",
            stream_body,
            stream_path,
        });
        delta_handle.remote_cache_url = Some(base_url.clone());

        let delta_substituted = delta_handle
            .try_substitute_remote(*output_path.digest(), &output_path, "out", true, Some(GcRootSource::Bootstrap))
            .await
            .unwrap()
            .expect("delta substitution should hit");
        let delta_attestation = delta_handle.get_artifact_attestation(&output_path).await.unwrap().unwrap();
        let delta_stored_pathinfo = delta_handle
            .pathinfo_service
            .get(*output_path.digest())
            .await
            .unwrap()
            .expect("delta pathinfo should persist");
        let delta_report = delta_handle.take_output_substitution_report(&output_path).expect("delta report");
        let delta_roots = crate::roots::list_roots(delta_state_dir.path()).unwrap();
        let delta_exported = output_path.to_absolute_path_with_prefix(delta_handle.output_dir_str());
        assert_eq!(delta_report.mode, OutputSubstitutionMode::Delta);
        assert_eq!(delta_substituted, path_info);
        assert_eq!(delta_stored_pathinfo, path_info);
        assert_eq!(delta_roots.len(), 1);
        assert_eq!(delta_roots[0].logical_path, output_path.to_absolute_path());
        assert_eq!(delta_roots[0].source, GcRootSource::Bootstrap);
        assert_eq!(std::fs::read_link(&delta_exported).unwrap(), PathBuf::from("target"));
        stop_delta_probe_server(&base_url, stop, probe_handle);

        let full_state_dir = tempfile::tempdir().unwrap();
        let (mut full_handle, full_remote) = test_handle_with_remote(full_state_dir.path());
        full_remote.put(path_info.clone()).await.unwrap();

        let full_substituted = full_handle
            .try_substitute_remote(*output_path.digest(), &output_path, "out", true, Some(GcRootSource::Bootstrap))
            .await
            .unwrap()
            .expect("full substitution should hit");
        let full_attestation = full_handle.get_artifact_attestation(&output_path).await.unwrap().unwrap();
        let full_stored_pathinfo = full_handle
            .pathinfo_service
            .get(*output_path.digest())
            .await
            .unwrap()
            .expect("full pathinfo should persist");
        let full_report = full_handle.take_output_substitution_report(&output_path).expect("full report");
        let full_roots = crate::roots::list_roots(full_state_dir.path()).unwrap();
        let full_exported = output_path.to_absolute_path_with_prefix(full_handle.output_dir_str());
        assert_eq!(full_report.mode, OutputSubstitutionMode::Full);
        assert_eq!(full_substituted, path_info);
        assert_eq!(full_stored_pathinfo, path_info);
        assert_eq!(full_roots.len(), 1);
        assert_eq!(full_roots[0].logical_path, output_path.to_absolute_path());
        assert_eq!(full_roots[0].source, GcRootSource::Bootstrap);
        assert_eq!(std::fs::read_link(&full_exported).unwrap(), PathBuf::from("target"));

        assert_eq!(
            delta_attestation, full_attestation,
            "accepted delta and full substitutions should persist identical artifact attestations"
        );
        assert_eq!(
            delta_stored_pathinfo, full_stored_pathinfo,
            "accepted delta and full substitutions should persist identical PathInfo"
        );
        assert_eq!(
            delta_handle.output_nodes.get(&output_path),
            full_handle.output_nodes.get(&output_path),
            "accepted delta and full substitutions should populate the same cached output node"
        );
        assert_eq!(
            delta_handle.built_outputs.get(&delta_exported),
            full_handle.built_outputs.get(&full_exported),
            "accepted delta and full substitutions should populate the same built-output metadata"
        );
    }
}
