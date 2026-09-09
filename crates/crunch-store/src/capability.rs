use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_store::nar::NarCalculationService;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::ActionResultDiscoveryReport;
use crate::ActionResultOutputProbe;
use crate::ActionResultPublicationReport;
use crate::ActionResultStoreSet;
use crate::ArtifactProvenance;
use crate::Error;
use crate::GcReport;
use crate::GcRootRecord;
use crate::GcRootSource;
use crate::OutputSubstitutionReport;
use crate::PersistOutputRequest;
use crate::RootRegistration;
use crate::StoreAuditEvent;
use crate::StoreFallbackMode;
use crate::StoreHandle;
use crate::VerifiedSourceIngestRequest;
use crate::roots;

/// Build-realization authority with private store services and session state.
///
/// Garbage collection is not part of build authority.
///
/// ```compile_fail
/// fn denied(store: &mut crunch_store::BuildStore) {
///     let _ = store.garbage_collect(false);
/// }
/// ```
///
/// Repair is not part of build authority.
///
/// ```compile_fail
/// fn denied(store: &mut crunch_store::BuildStore) {
///     let _ = store.repair("/nix/store/example");
/// }
/// ```
///
/// Source admission is not part of build authority.
///
/// ```compile_fail
/// fn denied(store: &mut crunch_store::BuildStore) {
///     let _ = store.source_admission();
/// }
/// ```
///
/// Arbitrary root mutation is not part of build authority.
///
/// ```compile_fail
/// fn denied(store: &mut crunch_store::BuildStore) {
///     let _ = store.register_retained_root("/nix/store/example");
/// }
/// ```
///
/// Raw writable services do not escape this capability.
///
/// ```compile_fail
/// fn denied(store: &crunch_store::BuildStore) {
///     let _ = store.blob_service();
/// }
/// ```
pub struct BuildStore {
    handle: StoreHandle,
}

/// Fixed action-result discovery, verification, and publication authority.
///
/// The builder cannot replace action-result backends or publishers.
///
/// ```compile_fail
/// fn denied(port: &mut crunch_store::ActionResultPort) {
///     port.replace_backends(Vec::new());
///     port.replace_publishers(Vec::new());
/// }
/// ```
pub struct ActionResultPort {
    stores: ActionResultStoreSet,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
    pathinfo_service: Arc<dyn PathInfoService>,
    remote_pathinfo: Option<Arc<dyn PathInfoService>>,
    store_dir: String,
}

/// Read-only output lookup authority.
#[derive(Clone)]
pub struct OutputLookup {
    pathinfo_service: Arc<dyn PathInfoService>,
    base_pathinfo_inspection_services: Vec<Arc<dyn PathInfoService>>,
    remote_pathinfo: Option<Arc<dyn PathInfoService>>,
    state_dir: PathBuf,
    overlay_state: Option<crate::overlay::StoreOverlayState>,
    ca_mappings: crate::CaMappings,
    base_ca_mappings: Vec<crate::CaMappings>,
}

/// Selected-root registration and retention-query authority.
#[derive(Clone)]
pub struct RootRegistry {
    state_dir: PathBuf,
    store_dir: String,
    pathinfo_service: Arc<dyn PathInfoService>,
}

/// Source admission authority borrowed from a shell-owned compatibility handle.
pub struct SourceAdmission<'a> {
    handle: &'a mut StoreHandle,
}

/// Administrative authority borrowed from a shell-owned compatibility handle.
pub struct StoreAdmin<'a> {
    handle: &'a mut StoreHandle,
}

/// Build-service castore operations without raw service access.
///
/// ```compile_fail
/// fn denied(store: &crunch_store::BuildServiceStore) {
///     let _ = store.blob_service();
///     let _ = store.directory_service();
/// }
/// ```
#[derive(Clone)]
pub struct BuildServiceStore {
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
}

/// Transfer-object read authority for remote transfer spooling.
///
/// Exposes NAR rendering, bounded castore blob reads, and canonical directory
/// bytes without leaking raw blob, directory, or PathInfo services.
///
/// ```compile_fail
/// fn denied(store: &crunch_store::TransferObjectStore<'_>) {
///     let _ = store.blob_service();
///     let _ = store.directory_service();
///     let _ = store.pathinfo_service();
/// }
/// ```
///
/// Signing is not part of transfer-object authority.
///
/// ```compile_fail
/// fn denied(store: &crunch_store::TransferObjectStore<'_>) {
///     let _ = store.sign_outputs();
/// }
/// ```
#[derive(Clone)]
pub struct TransferObjectStore<'a> {
    handle: &'a StoreHandle,
}

/// Mantle-owned reader over a stored castore blob.
///
/// The vendored reader trait stays private to the store shell.
pub struct TransferBlobReader {
    inner: Box<dyn snix_castore::blobservice::BlobReader>,
}

impl tokio::io::AsyncRead for TransferBlobReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

/// Capability values retained by pipeline orchestration and the builder.
pub struct PipelineStoreParts {
    pub build_store: BuildStore,
    pub action_results: ActionResultPort,
    pub build_service_store: BuildServiceStore,
    pub output_lookup: OutputLookup,
    pub root_registry: RootRegistry,
}

/// Capability values owned by a builder outside pipeline orchestration.
pub struct BuilderStoreParts {
    pub build_store: BuildStore,
    pub action_results: ActionResultPort,
}

impl TransferObjectStore<'_> {
    /// Render a castore node as a NAR stream.
    pub async fn render_nar<W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        node: &Node,
        dest: &mut W,
    ) -> Result<(), Error> {
        self.handle.render_nar(node, dest).await
    }

    /// Open a stored castore blob for reading.
    ///
    /// Returns `Ok(None)` when the blob is absent.
    pub async fn open_blob(&self, digest: &snix_castore::B3Digest) -> Result<Option<TransferBlobReader>, Error> {
        let reader = self
            .handle
            .blob_service
            .open_read(digest)
            .await
            .map_err(|err| Error::Export(format!("opening castore blob: {err}")))?;
        Ok(reader.map(|inner| TransferBlobReader { inner }))
    }

    /// Read the canonical postcard-encoded directory payload for a digest.
    ///
    /// The store shell verifies that the encoded bytes hash back to the
    /// requested digest before returning them.
    pub async fn read_directory_canonical_bytes(
        &self,
        digest: &snix_castore::B3Digest,
    ) -> Result<Vec<u8>, Error> {
        let digest_hex = data_encoding::HEXLOWER.encode(digest.as_ref());
        let directory = self
            .handle
            .directory_service
            .get(digest)
            .await
            .map_err(|err| Error::Export(format!("reading castore directory {digest_hex}: {err}")))?
            .ok_or_else(|| Error::Export(format!("castore directory missing: {digest_hex}")))?;
        let bytes = postcard::to_stdvec(&snix_castore::proto::Directory::from(directory))
            .map_err(|err| Error::Export(format!("serializing castore directory: {err}")))?;
        assert_eq!(
            blake3::hash(&bytes).to_hex().as_str(),
            digest_hex,
            "castore directory bytes must hash to the requested digest"
        );
        Ok(bytes)
    }
}

impl StoreHandle {
    /// Borrow the transfer-object read authority.
    pub fn transfer_objects(&self) -> TransferObjectStore<'_> {
        TransferObjectStore { handle: self }
    }
}

impl StoreHandle {
    #[must_use]
    pub fn into_pipeline_store_parts(mut self) -> PipelineStoreParts {
        let (ca_mappings, base_ca_mappings) = self.ca_mapping_snapshots();
        let output_lookup = OutputLookup {
            pathinfo_service: self.pathinfo_service(),
            base_pathinfo_inspection_services: self.base_pathinfo_inspection_services(),
            remote_pathinfo: self.remote_pathinfo(),
            state_dir: self.state_dir().to_path_buf(),
            overlay_state: self.overlay_state(),
            ca_mappings,
            base_ca_mappings,
        };
        let root_registry = RootRegistry {
            state_dir: self.state_dir().to_path_buf(),
            store_dir: self.store_dir().to_string(),
            pathinfo_service: self.pathinfo_service(),
        };
        let build_service_store = BuildServiceStore {
            blob_service: self.blob_service(),
            directory_service: self.directory_service(),
        };
        let action_results = ActionResultPort {
            stores: self.take_action_result_stores(),
            blob_service: self.blob_service(),
            directory_service: self.directory_service(),
            pathinfo_service: self.pathinfo_service(),
            remote_pathinfo: self.remote_pathinfo(),
            store_dir: self.store_dir().to_string(),
        };
        PipelineStoreParts {
            build_store: BuildStore { handle: self },
            action_results,
            build_service_store,
            output_lookup,
            root_registry,
        }
    }

    #[must_use]
    pub fn into_builder_store_parts(mut self) -> BuilderStoreParts {
        let action_results = ActionResultPort {
            stores: self.take_action_result_stores(),
            blob_service: self.blob_service(),
            directory_service: self.directory_service(),
            pathinfo_service: self.pathinfo_service(),
            remote_pathinfo: self.remote_pathinfo(),
            store_dir: self.store_dir().to_string(),
        };
        BuilderStoreParts {
            build_store: BuildStore { handle: self },
            action_results,
        }
    }

    pub fn source_admission(&mut self) -> SourceAdmission<'_> {
        SourceAdmission { handle: self }
    }

    pub fn store_admin(&mut self) -> StoreAdmin<'_> {
        StoreAdmin { handle: self }
    }
}

impl BuildStore {
    #[must_use]
    pub fn store_dir(&self) -> &str {
        self.handle.store_dir()
    }

    #[must_use]
    pub fn output_dir_str(&self) -> &str {
        self.handle.output_dir_str()
    }

    #[must_use]
    pub fn state_dir(&self) -> &Path {
        self.handle.state_dir()
    }

    #[must_use]
    pub fn startup_audit_events(&self) -> &[StoreAuditEvent] {
        self.handle.startup_audit_events()
    }

    pub fn set_root_registration(&mut self, registration: Option<RootRegistration>) {
        self.handle.set_root_registration(registration);
    }

    pub async fn read_file_node(&self, node: &Node, max_bytes: u64) -> Result<Vec<u8>, Error> {
        crate::build_io::read_file_node(self.handle.blob_service().as_ref(), node, max_bytes).await
    }

    pub async fn rewrite_node(&self, node: &Node, old_bytes: &[u8], new_bytes: &[u8]) -> Result<(Node, bool), Error> {
        crate::build_io::rewrite_node(
            node,
            old_bytes,
            new_bytes,
            self.handle.blob_service().as_ref(),
            self.handle.directory_service().as_ref(),
        )
        .await
    }

    pub async fn apply_rewrites(&self, node: &Node, rewrites: &[(String, String)]) -> Result<Node, Error> {
        let mut current = node.clone();
        for (old_value, new_value) in rewrites {
            let (rewritten, _) = self.rewrite_node(&current, old_value.as_bytes(), new_value.as_bytes()).await?;
            current = rewritten;
        }
        Ok(current)
    }

    pub async fn calculate_nar(&self, node: &Node) -> Result<(u64, [u8; 32]), Error> {
        let renderer =
            snix_store::nar::SimpleRenderer::new(self.handle.blob_service(), self.handle.directory_service());
        renderer
            .calculate_nar(node)
            .await
            .map_err(|error| Error::Store(format!("NAR calculation: {error}")))
    }

    pub async fn resolve_closure(
        &self,
        root: &StorePath<String>,
        fallback_mode: StoreFallbackMode,
    ) -> Result<crate::ClosureResolution, Error> {
        let remote_pathinfo = self.handle.remote_pathinfo();
        crate::resolve_closure(
            root,
            self.handle.pathinfo_service().as_ref(),
            remote_pathinfo.as_deref(),
            fallback_mode,
            self.handle.store_dir(),
        )
        .await
    }

    pub async fn ingest_build_input(
        &mut self,
        logical_path: StorePath<String>,
        host_path: &Path,
    ) -> Result<Node, Error> {
        let node = snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(
            self.handle.blob_service(),
            self.handle.directory_service(),
            host_path,
            None,
        )
        .await
        .map_err(|error| Error::Store(format!("ingesting build input {}: {error}", host_path.display())))?;
        self.insert_output_node(logical_path, node.clone());
        Ok(node)
    }

    pub async fn hash_blob(
        &self,
        digest: &snix_castore::B3Digest,
        algorithm: nix_compat::nixhash::HashAlgo,
    ) -> Result<nix_compat::nixhash::NixHash, Error> {
        crate::build_io::hash_blob(self.handle.blob_service().as_ref(), digest, algorithm).await
    }

    pub async fn nar_hash(
        &self,
        node: &Node,
        algorithm: nix_compat::nixhash::HashAlgo,
    ) -> Result<nix_compat::nixhash::NixHash, Error> {
        crate::build_io::nar_hash(node, algorithm, self.handle.blob_service(), self.handle.directory_service()).await
    }

    pub async fn cached_node_for_path(&mut self, path: &StorePath<String>) -> Result<Option<Node>, Error> {
        self.handle.cached_node_for_path(path).await
    }

    pub async fn check_cache(
        &mut self,
        derivation_path: &StorePath<String>,
        derivation: &nix_compat::derivation::Derivation,
        is_root: bool,
        root_source: Option<GcRootSource>,
    ) -> Result<Option<std::collections::HashMap<String, PathInfo>>, Error> {
        self.handle.check_cache(derivation_path, derivation, is_root, root_source).await
    }

    pub fn take_output_substitution_report(&mut self, path: &StorePath<String>) -> Option<OutputSubstitutionReport> {
        self.handle.take_output_substitution_report(path)
    }

    pub fn overlay_report(&self) -> Result<Option<crate::StoreOverlayReport>, Error> {
        self.handle.overlay_report()
    }

    #[must_use]
    pub fn is_overlay_composed(&self) -> bool {
        self.handle.overlay_state().is_some()
    }

    pub fn take_read_layer_selections(&mut self) -> Vec<crate::layer::StoreLayerSelection> {
        self.handle.take_read_layer_selections()
    }

    pub fn record_verified_output_substitution_report(
        &mut self,
        path: &StorePath<String>,
        report: OutputSubstitutionReport,
    ) {
        self.handle.record_verified_output_substitution_report(path, report);
    }

    pub fn insert_ca_mapping(&mut self, drv_path: &str, output_name: &str, output_path: &str) {
        self.handle.insert_ca_mapping(drv_path, output_name, output_path);
    }

    pub async fn persist_and_export_signed_output(
        &mut self,
        request: PersistOutputRequest<'_>,
    ) -> Result<PathInfo, Error> {
        self.handle.persist_and_export_signed_output(request).await
    }

    pub async fn admit_action_result_outputs(
        &mut self,
        outputs: &BTreeMap<String, PathInfo>,
        is_root: bool,
        root_source: Option<GcRootSource>,
    ) -> Result<(), Error> {
        self.handle.admit_action_result_outputs(outputs, is_root, root_source).await
    }

    pub async fn get_artifact_attestation(
        &self,
        store_path: &StorePath<String>,
    ) -> Result<Option<crate::StoredArtifactAttestation>, Error> {
        self.handle.get_artifact_attestation(store_path).await
    }

    #[must_use]
    pub fn output_node(&self, path: &StorePath<String>) -> Option<&Node> {
        self.handle.output_nodes.get(path)
    }

    #[must_use]
    pub fn contains_output_node(&self, path: &StorePath<String>) -> bool {
        self.handle.output_nodes.contains_key(path)
    }

    pub fn insert_output_node(&mut self, path: StorePath<String>, node: Node) -> Option<Node> {
        self.handle.output_nodes.insert(path, node)
    }

    #[must_use]
    pub fn has_built_output(&self, physical_path: &str) -> bool {
        self.handle.built_outputs.contains_key(physical_path)
    }

    pub fn insert_built_output(&mut self, physical_path: String, path_info: PathInfo) -> Option<PathInfo> {
        self.handle.built_outputs.insert(physical_path, path_info)
    }
}

impl ActionResultPort {
    pub async fn discover(&self, action_ref: &str) -> ActionResultDiscoveryReport {
        self.stores.discover(action_ref).await
    }

    pub async fn probe_outputs(
        &self,
        record: &crunch_action_result_core::ActionResultRecord,
    ) -> Result<ActionResultOutputProbe, String> {
        let mut outputs = BTreeMap::new();
        let mut transferred_nar_bytes = 0_u64;
        let mut reused_nar_bytes = 0_u64;
        for output in &record.outputs {
            let store_path = StorePath::from_absolute_path_with_prefix(output.store_path.as_bytes(), &self.store_dir)
                .map_err(|_| format!("action-result-output-store-path-invalid:{}", output.store_path))?;
            let digest = *store_path.digest();
            let local = self
                .pathinfo_service
                .get(digest)
                .await
                .map_err(|error| format!("action-result-local-pathinfo-query:{error}"))?;
            let (path_info, is_transferred) = match local {
                Some(path_info) => (path_info, false),
                None => {
                    let remote = self
                        .remote_pathinfo
                        .as_ref()
                        .ok_or_else(|| "action-result-output-pathinfo-missing".to_string())?;
                    let path_info = remote
                        .get(digest)
                        .await
                        .map_err(|error| format!("action-result-remote-pathinfo-query:{error}"))?
                        .ok_or_else(|| "action-result-output-pathinfo-missing".to_string())?;
                    (path_info, true)
                }
            };
            if path_info.store_path != store_path {
                return Err("action-result-output-pathinfo-store-path-mismatch".to_string());
            }
            let is_complete = crate::recursive_castore_completeness(
                self.blob_service.as_ref(),
                self.directory_service.as_ref(),
                &path_info.node,
            )
            .await
            .map_err(|error| format!("action-result-object-completeness:{error}"))?;
            if !is_complete {
                return Err("action-result-output-object-incomplete".to_string());
            }
            (transferred_nar_bytes, reused_nar_bytes) =
                crate::handle::account_action_result_nar_bytes(crate::handle::NarByteAccounting {
                    transferred_nar_bytes,
                    reused_nar_bytes,
                    nar_size_bytes: path_info.nar_size,
                    is_transferred,
                })?;
            if outputs.insert(output.name.clone(), path_info).is_some() {
                return Err("action-result-output-name-duplicate".to_string());
            }
        }
        Ok(ActionResultOutputProbe {
            outputs,
            transferred_nar_bytes,
            reused_nar_bytes,
        })
    }

    pub async fn publish_local(
        &self,
        record: &crunch_action_result_core::SignedActionResultRecord,
    ) -> Result<Vec<ActionResultPublicationReport>, String> {
        self.stores.publish_local(record).await
    }

    pub async fn publish_remote(
        &self,
        record: &crunch_action_result_core::SignedActionResultRecord,
    ) -> Result<Vec<ActionResultPublicationReport>, String> {
        self.stores.publish_remote(record).await
    }
}

impl OutputLookup {
    pub async fn find(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, Error> {
        self.find_with_layer(path).await.map(|found| found.map(|layered| layered.value))
    }

    pub async fn find_with_layer(
        &self,
        path: &StorePath<String>,
    ) -> Result<Option<crate::layer::Layered<PathInfo>>, Error> {
        self.revalidate_overlay()?;
        let found = self
            .pathinfo_service
            .get_with_layer(*path.digest())
            .await
            .map_err(|error| Error::PathInfoService(format!("output lookup: {error}")))?;
        self.revalidate_overlay()?;
        let Some(found) = found else {
            return Ok(None);
        };
        if found.value.store_path != *path {
            return Err(Error::Store(format!("output lookup returned a conflicting path for {path}")));
        }
        if self.overlay_state.is_some() && found.layer_index == 0 {
            let trusted_keys = crate::overlay::load_layer_trust_keys(&self.state_dir)?;
            crate::overlay::verify_pathinfo_trust(&found.value, &trusted_keys)
                .map_err(|error| Error::Store(format!("overlay-layer-trust-failure for {path}: {error}")))?;
        }
        let shadows = self.shadow_observations(found.layer_index, &found.value).await;
        let mut layered = crate::layer::Layered::from_service_index(found.value, found.layer_index)
            .map_err(|error| Error::Store(format!("mapping output layer for {path}: {error}")))?;
        layered.shadows = shadows;
        Ok(Some(layered))
    }

    pub async fn find_remote(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, Error> {
        let Some(remote) = self.remote_pathinfo.as_ref() else {
            return Ok(None);
        };
        let found = remote
            .get(*path.digest())
            .await
            .map_err(|error| Error::PathInfoService(format!("remote output lookup: {error}")))?;
        if let Some(path_info) = found.as_ref()
            && path_info.store_path != *path
        {
            return Err(Error::Store(format!("remote output lookup returned a conflicting path for {path}")));
        }
        Ok(found)
    }

    pub fn resolve_ca_mapping(&self, drv_abs: &str, output_name: &str) -> Result<Option<String>, Error> {
        self.revalidate_overlay()?;
        if let Some(path) = self.ca_mappings.get(drv_abs, output_name) {
            return Ok(Some(path.to_string()));
        }
        for mappings in &self.base_ca_mappings {
            if let Some(path) = mappings.get(drv_abs, output_name) {
                return Ok(Some(path.to_string()));
            }
        }
        Ok(None)
    }

    pub fn overlay_report(&self) -> Result<Option<crate::StoreOverlayReport>, Error> {
        self.revalidate_overlay()?;
        self.overlay_state.as_ref().map(crate::overlay::overlay_report).transpose()
    }

    #[must_use]
    pub fn is_overlay_composed(&self) -> bool {
        self.overlay_state.is_some()
    }

    fn revalidate_overlay(&self) -> Result<(), Error> {
        self.overlay_state.as_ref().map_or(Ok(()), crate::overlay::revalidate_overlay_state)
    }

    async fn shadow_observations(
        &self,
        selected_layer_index: usize,
        selected: &PathInfo,
    ) -> Vec<crate::layer::LayerShadowObservation> {
        let mut observations = Vec::new();
        for (base_index, service) in self.base_pathinfo_inspection_services.iter().enumerate() {
            let layer_index = base_index.saturating_add(1);
            if layer_index <= selected_layer_index {
                continue;
            }
            let layer = crate::layer::StoreLayer::Base { index: layer_index };
            let status = match service.get(*selected.store_path.digest()).await {
                Ok(Some(lower)) if lower.store_path != selected.store_path => {
                    crate::layer::LayerShadowStatus::DigestCollision
                }
                Ok(Some(lower)) if lower == *selected => crate::layer::LayerShadowStatus::Matching,
                Ok(Some(_)) => crate::layer::LayerShadowStatus::Conflicting,
                Ok(None) => continue,
                Err(_) => crate::layer::LayerShadowStatus::ReadFailure,
            };
            observations.push(crate::layer::LayerShadowObservation { layer, status });
        }
        observations
    }
}

impl RootRegistry {
    pub fn list(&self) -> Result<Vec<GcRootRecord>, Error> {
        roots::list_roots(&self.state_dir)
    }

    pub async fn register_if_present(
        &self,
        store_path: &StorePath<String>,
        source: GcRootSource,
    ) -> Result<Option<GcRootRecord>, Error> {
        let is_present = self
            .pathinfo_service
            .get(*store_path.digest())
            .await
            .map_err(|error| Error::PathInfoService(format!("root registration lookup: {error}")))?
            .is_some();
        if !is_present {
            return Ok(None);
        }
        roots::register_root(&self.state_dir, &self.store_dir, self.pathinfo_service.as_ref(), store_path, source)
            .await
            .map(Some)
    }

    pub async fn register_managed_batch(
        &self,
        registrations: Vec<(StorePath<String>, GcRootSource, RootRegistration)>,
    ) -> Result<Vec<GcRootRecord>, Error> {
        roots::register_root_batch_with_registration(
            &self.state_dir,
            &self.store_dir,
            self.pathinfo_service.as_ref(),
            registrations,
        )
        .await
    }

    pub async fn register_managed_if_present(
        &self,
        store_path: &StorePath<String>,
        source: GcRootSource,
        registration: RootRegistration,
    ) -> Result<Option<GcRootRecord>, Error> {
        let is_present = self
            .pathinfo_service
            .get(*store_path.digest())
            .await
            .map_err(|error| Error::PathInfoService(format!("managed root registration lookup: {error}")))?
            .is_some();
        if !is_present {
            return Ok(None);
        }
        roots::register_root_with_registration(
            &self.state_dir,
            &self.store_dir,
            self.pathinfo_service.as_ref(),
            store_path,
            source,
            registration,
        )
        .await
        .map(Some)
    }
}

impl SourceAdmission<'_> {
    #[must_use]
    pub fn store_dir(&self) -> &str {
        self.handle.store_dir()
    }

    pub async fn preflight(&self, request: VerifiedSourceIngestRequest<'_>) -> Result<PathInfo, Error> {
        self.handle.preflight_verified_source(request).await
    }

    pub async fn ingest(&mut self, request: VerifiedSourceIngestRequest<'_>) -> Result<PathInfo, Error> {
        self.handle.ingest_verified_source(request).await
    }

    pub async fn adopt_local_output(
        &mut self,
        logical_store_path: &str,
        output_name: &str,
        signing_key: &nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey>,
        provenance: Option<&ArtifactProvenance>,
    ) -> Result<PathInfo, Error> {
        self.handle
            .adopt_verified_local_output(logical_store_path, output_name, signing_key, provenance)
            .await
    }
}

impl StoreAdmin<'_> {
    pub fn list_retained_roots(&self) -> Result<Vec<GcRootRecord>, Error> {
        self.handle.list_retained_roots()
    }

    pub fn migrate_legacy_root_registry(&self) -> Result<Vec<GcRootRecord>, Error> {
        roots::migrate_legacy_registry(self.handle.state_dir())
    }

    pub async fn garbage_collect(&mut self, accepted_plan_id: Option<&str>) -> Result<GcReport, Error> {
        self.handle.garbage_collect(accepted_plan_id).await
    }

    pub async fn garbage_collect_with_castore_roots(
        &mut self,
        accepted_plan_id: Option<&str>,
        retained_castore_roots: &[Node],
    ) -> Result<GcReport, Error> {
        self.handle.garbage_collect_with_castore_roots(accepted_plan_id, retained_castore_roots).await
    }
}

impl BuildServiceStore {
    pub async fn has_complete_content(&self, path_info: &PathInfo) -> Result<bool, Error> {
        crate::recursive_castore_completeness(
            self.blob_service.as_ref(),
            self.directory_service.as_ref(),
            &path_info.node,
        )
        .await
    }

    #[must_use]
    pub fn bubblewrap_build_service(
        &self,
        work_directory: PathBuf,
        failure_workspace_root: Option<PathBuf>,
    ) -> impl snix_build::buildservice::BuildService + use<> {
        let service = snix_build::buildservice::BubblewrapBuildService::new(
            work_directory,
            self.blob_service.clone(),
            self.directory_service.clone(),
        );
        match failure_workspace_root {
            Some(root) => service.with_failure_workspace_root(root),
            None => service,
        }
    }

    pub async fn ingest_host_path(&self, path: &Path) -> Result<Node, Error> {
        snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(
            self.blob_service.clone(),
            self.directory_service.clone(),
            path,
            None,
        )
        .await
        .map_err(|error| Error::Store(format!("ingesting build-service path {}: {error}", path.display())))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use snix_castore::SymlinkTarget;
    use snix_store::pathinfoservice::LruPathInfoService;

    use super::*;

    const TEST_PATH_INFO_CAPACITY: usize = 16;
    const TEST_PATH_DIGEST_BYTE: u8 = 7;
    const TEST_NAR_DIGEST_BYTE: u8 = 11;
    const TEST_NAR_DIGEST_BYTES: usize = 32;

    fn path_info(store_path: StorePath<String>) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").expect("valid symlink target"),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [TEST_NAR_DIGEST_BYTE; TEST_NAR_DIGEST_BYTES],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        }
    }

    fn path_info_service() -> Arc<dyn PathInfoService> {
        let capacity = NonZeroUsize::new(TEST_PATH_INFO_CAPACITY).expect("test capacity must be positive");
        Arc::new(LruPathInfoService::with_capacity("capability-test".to_string(), capacity))
    }

    #[tokio::test]
    async fn output_lookup_and_selected_root_registration_share_exact_identity() {
        let state = tempfile::tempdir().expect("temporary root registry state");
        let service = path_info_service();
        let store_path = StorePath::from_name_and_digest_fixed(
            "capability-output",
            [TEST_PATH_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
        )
        .expect("valid test store path");
        let expected = path_info(store_path.clone());
        service.put(expected.clone()).await.expect("insert test PathInfo");
        let output_lookup = OutputLookup {
            pathinfo_service: service.clone(),
            base_pathinfo_inspection_services: Vec::new(),
            remote_pathinfo: None,
            state_dir: state.path().to_path_buf(),
            overlay_state: None,
            ca_mappings: crate::CaMappings::default(),
            base_ca_mappings: Vec::new(),
        };
        let root_registry = RootRegistry {
            state_dir: state.path().to_path_buf(),
            store_dir: nix_compat::store_path::STORE_DIR.to_string(),
            pathinfo_service: service,
        };

        let found = output_lookup.find(&store_path).await.expect("find exact output");
        let registered = root_registry
            .register_if_present(&store_path, GcRootSource::Build)
            .await
            .expect("register selected root");
        let roots = root_registry.list().expect("list selected roots");

        assert_eq!(found, Some(expected));
        assert_eq!(registered.as_ref(), roots.first());
        assert_eq!(roots.len(), 1);
    }

    #[tokio::test]
    async fn missing_output_does_not_create_a_root() {
        let state = tempfile::tempdir().expect("temporary root registry state");
        let service = path_info_service();
        let store_path = StorePath::from_name_and_digest_fixed(
            "missing-output",
            [TEST_PATH_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
        )
        .expect("valid test store path");
        let root_registry = RootRegistry {
            state_dir: state.path().to_path_buf(),
            store_dir: nix_compat::store_path::STORE_DIR.to_string(),
            pathinfo_service: service,
        };

        let registered = root_registry
            .register_if_present(&store_path, GcRootSource::Build)
            .await
            .expect("missing output is not an error");

        assert!(registered.is_none());
        assert!(root_registry.list().expect("list roots").is_empty());
    }
}
