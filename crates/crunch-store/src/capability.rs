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

impl StoreHandle {
    #[must_use]
    pub fn into_pipeline_store_parts(mut self) -> PipelineStoreParts {
        let output_lookup = OutputLookup {
            pathinfo_service: self.pathinfo_service(),
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
        let found = self
            .pathinfo_service
            .get(*path.digest())
            .await
            .map_err(|error| Error::PathInfoService(format!("output lookup: {error}")))?;
        match &found {
            Some(path_info) if path_info.store_path != *path => {
                Err(Error::Store(format!("output lookup returned a conflicting path for {path}")))
            }
            _ => Ok(found),
        }
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
