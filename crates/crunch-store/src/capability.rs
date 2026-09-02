use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::MemoryBlobServiceConfig;
use snix_castore::composition::CompositionContext;
use snix_castore::composition::REG;
use snix_castore::composition::ServiceBuilder;
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
use crate::publisher::AdmittedOutput;
use crate::publisher::PublicationDisposition;
use crate::publisher::PublicationEffectPlan;
use crate::publisher::PublicationObservation;
use crate::publisher::Publisher;
use crate::publisher::plan_publication;
use crate::publisher::validate_publication_plan;
use crate::roots;

const SHA256_DIGEST_BYTES: usize = 32;
const PATHINFO_ADMIN_SCAN_MAX: u32 = 1_000_000;
const PATHINFO_ADMIN_INITIAL_CAPACITY: usize = 256;

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

/// External publisher execution authority separated from local admission.
#[derive(Clone, Copy)]
pub struct PublicationExecution<'a> {
    publishers: &'a [Arc<dyn Publisher>],
}

/// Mantle-owned observation from one bounded NAR ingest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferNarObservation {
    pub node: Node,
    pub nar_sha256: [u8; SHA256_DIGEST_BYTES],
    pub nar_size_bytes: u64,
}

/// Bounded transfer reads and NAR rendering without raw service access.
#[derive(Clone, Copy)]
pub struct TransferStore<'a> {
    handle: &'a StoreHandle,
}

/// Selected-root registration and retention-query authority.
#[derive(Clone)]
pub struct RootRegistry {
    state_dir: PathBuf,
    store_dir: String,
    pathinfo_service: Arc<dyn PathInfoService>,
}

/// Source admission authority borrowed from a store-owned source capability.
pub struct SourceAdmission<'a> {
    handle: &'a mut StoreHandle,
}

/// Read-only attestation lookup and closure reconstruction authority.
pub struct AttestationStore {
    handle: StoreHandle,
}

/// Source preflight, ingest, and admitted local-output authority.
pub struct SourceStore {
    handle: StoreHandle,
}

/// Foreign realization source, cache hydration, and build handoff authority.
pub struct ForeignRealizationStore {
    handle: StoreHandle,
}

/// Rust-unit cache castore authority without raw service exposure.
#[derive(Clone)]
pub struct RustCacheStore {
    state_dir: PathBuf,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
}

/// Read-only castore provenance scan authority.
pub struct ProvenanceStore {
    handle: StoreHandle,
}

/// PathInfo verification and signing authority with no raw service escape.
pub struct PathInfoAdministration {
    service: Arc<dyn PathInfoService>,
}

/// Store-command administration authority with no raw service access.
pub struct StoreAdministration {
    handle: StoreHandle,
}

/// Administrative authority borrowed from store-owned administration.
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

/// Read-only planning and bounded probe capabilities.
pub struct PlanningStoreParts {
    pub action_results: ActionResultPort,
    pub build_service_store: BuildServiceStore,
    pub output_lookup: OutputLookup,
}

/// Capability values owned by a builder outside pipeline orchestration.
pub struct BuilderStoreParts {
    pub build_store: BuildStore,
    pub action_results: ActionResultPort,
}

async fn open_memory_blob_service(instance_name: &str) -> Result<Arc<dyn BlobService>, Error> {
    if instance_name.is_empty() {
        return Err(Error::Store("memory blob service name must not be empty".to_string()));
    }
    debug_assert!(!instance_name.is_empty());
    let service = MemoryBlobServiceConfig {}
        .build(instance_name, &CompositionContext::blank(&REG))
        .await
        .map_err(|error| Error::Store(format!("opening memory blob service {instance_name}: {error}")))?;
    debug_assert!(Arc::strong_count(&service) >= 1);
    Ok(service)
}

pub async fn open_pipeline_store_parts(config: crate::StoreConfig) -> Result<PipelineStoreParts, Error> {
    StoreHandle::open(config).await.map(StoreHandle::into_pipeline_store_parts)
}

pub async fn open_overlay_pipeline_store_parts(config: crate::StoreConfig) -> Result<PipelineStoreParts, Error> {
    StoreHandle::open_overlay(config).await.map(StoreHandle::into_pipeline_store_parts)
}

pub async fn open_raw_seed_store_parts(
    state_dir: &Path,
    output_dir: &Path,
    logical_store_dir: &str,
) -> Result<PipelineStoreParts, Error> {
    use snix_castore::blobservice::ObjectStoreBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_store::pathinfoservice::RedbPathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

    if !state_dir.is_absolute() || !output_dir.is_absolute() || !logical_store_dir.starts_with('/') {
        return Err(Error::Store("raw seed store paths must be absolute".to_string()));
    }
    debug_assert!(state_dir.is_absolute());
    debug_assert!(output_dir.is_absolute());
    debug_assert!(logical_store_dir.starts_with('/'));
    let blob_directory = state_dir.join("blobs");
    std::fs::create_dir_all(&blob_directory)
        .map_err(|error| Error::Store(format!("creating raw seed blob directory: {error}")))?;
    let blob_service = Arc::new(
        ObjectStoreBlobService::new_local(&blob_directory)
            .map_err(|error| Error::Store(format!("opening raw seed blob service: {error}")))?,
    );
    let directory_service = RedbDirectoryService::new_temporary("bootstrap".to_string(), RedbDirectoryServiceConfig {
        path: None,
        read_only: false,
        cache_size: None,
    })
    .map_err(|error| Error::Store(format!("opening raw seed directory service: {error}")))?;
    let database_path = state_dir.join("pathinfo.redb");
    let pathinfo_service = match RedbPathInfoService::new("crunch".to_string(), RedbPathInfoServiceConfig {
        path: Some(database_path.clone()),
        read_only: false,
        cache_size: None,
    })
    .await
    {
        Ok(service) => service,
        Err(error) => {
            tracing::warn!(
                path = %database_path.display(),
                err = %error,
                "failed to open raw seed PathInfo database, using in-memory fallback"
            );
            RedbPathInfoService::new_temporary("crunch".to_string(), RedbPathInfoServiceConfig {
                path: None,
                read_only: false,
                cache_size: None,
            })
            .map_err(|fallback_error| Error::Store(format!("opening raw seed in-memory PathInfo: {fallback_error}")))?
        }
    };
    let handle = StoreHandle::from_services_with_store_dir(
        crate::StoreHandleServices {
            blob_service: blob_service as Arc<dyn BlobService>,
            directory_service: Arc::new(directory_service) as Arc<dyn DirectoryService>,
            pathinfo_service: Arc::new(pathinfo_service) as Arc<dyn PathInfoService>,
            remote_pathinfo: None,
            state_dir: state_dir.to_path_buf(),
            output_dir_str: output_dir.to_string_lossy().into_owned(),
            publishers: Vec::new(),
        },
        logical_store_dir.to_string(),
    );
    Ok(handle.into_pipeline_store_parts())
}

pub async fn open_planning_store_parts(config: crate::StoreConfig) -> Result<PlanningStoreParts, Error> {
    let mut handle = StoreHandle::open(config).await?;
    debug_assert!(!handle.store_dir().is_empty());
    debug_assert!(!handle.output_dir_str().is_empty());
    let (ca_mappings, base_ca_mappings) = handle.ca_mapping_snapshots();
    let output_lookup = OutputLookup {
        pathinfo_service: handle.pathinfo_service(),
        base_pathinfo_inspection_services: handle.base_pathinfo_inspection_services(),
        remote_pathinfo: handle.remote_pathinfo(),
        state_dir: handle.state_dir().to_path_buf(),
        overlay_state: handle.overlay_state(),
        ca_mappings,
        base_ca_mappings,
    };
    let build_service_store = BuildServiceStore {
        blob_service: handle.blob_service(),
        directory_service: handle.directory_service(),
    };
    let action_results = ActionResultPort {
        stores: handle.take_action_result_stores(),
        blob_service: handle.blob_service(),
        directory_service: handle.directory_service(),
        pathinfo_service: handle.pathinfo_service(),
        remote_pathinfo: handle.remote_pathinfo(),
        store_dir: handle.store_dir().to_string(),
    };
    Ok(PlanningStoreParts {
        action_results,
        build_service_store,
        output_lookup,
    })
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

    #[must_use]
    pub fn transfer_store(&self) -> TransferStore<'_> {
        TransferStore { handle: self }
    }

    pub fn store_admin(&mut self) -> StoreAdmin<'_> {
        StoreAdmin { handle: self }
    }
}

impl BuildStore {
    pub async fn open(config: crate::StoreConfig) -> Result<Self, Error> {
        StoreHandle::open(config).await.map(|handle| Self { handle })
    }

    #[must_use]
    pub fn into_pipeline_store_parts(self) -> PipelineStoreParts {
        self.handle.into_pipeline_store_parts()
    }

    #[must_use]
    pub fn transfer_store(&self) -> TransferStore<'_> {
        TransferStore { handle: &self.handle }
    }

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
    ) -> Result<AdmittedOutput, Error> {
        let path_info = self.handle.persist_and_export_signed_output(request).await?;
        let publisher_count = u32::try_from(self.handle.publishers().len())
            .map_err(|_| Error::Store("publisher count does not fit u32".to_string()))?;
        let publication_plan = plan_publication(&path_info, publisher_count).map_err(Error::Store)?;
        Ok(AdmittedOutput {
            path_info,
            publication_plan,
        })
    }

    #[must_use]
    pub fn publication_execution(&self) -> PublicationExecution<'_> {
        PublicationExecution {
            publishers: self.handle.publishers(),
        }
    }

    pub async fn admit_action_result_outputs(
        &mut self,
        outputs: &BTreeMap<String, PathInfo>,
        is_root: bool,
        root_source: Option<GcRootSource>,
    ) -> Result<Vec<AdmittedOutput>, Error> {
        let mut admitted = Vec::with_capacity(outputs.len());
        for (output_name, path_info) in outputs {
            admitted.push(
                self.persist_and_export_signed_output(PersistOutputRequest {
                    output_name,
                    output_path: &path_info.store_path,
                    path_info: path_info.clone(),
                    final_node: path_info.node.clone(),
                    provenance: None,
                    is_root,
                    root_source,
                })
                .await?,
            );
        }
        Ok(admitted)
    }

    pub async fn path_info(&self, store_path: &StorePath<String>) -> Result<Option<PathInfo>, Error> {
        self.handle.path_info_with_layer(store_path).await.map(|value| value.map(|layered| layered.value))
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
        let shadows = self.shadow_observations(found.layer_index, &found.value).await?;
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
    ) -> Result<Vec<crate::layer::LayerShadowObservation>, Error> {
        let mut observations = Vec::with_capacity(self.base_pathinfo_inspection_services.len());
        for (base_index, service) in self.base_pathinfo_inspection_services.iter().enumerate() {
            let layer_index = base_index.saturating_add(1);
            if layer_index <= selected_layer_index {
                continue;
            }
            let layer = crate::layer::StoreLayer::from_service_index(layer_index)
                .map_err(|error| Error::Store(format!("mapping output shadow layer: {error}")))?;
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
        Ok(observations)
    }
}

impl PublicationExecution<'_> {
    pub async fn execute(
        &self,
        plan: &PublicationEffectPlan,
        path_info: &PathInfo,
    ) -> Result<Vec<PublicationObservation>, Error> {
        validate_publication_plan(plan, path_info, self.publishers.len()).map_err(Error::Store)?;
        let mut observations = Vec::with_capacity(plan.effects.len());
        for effect in &plan.effects {
            let publisher_index = usize::try_from(effect.publisher_index)
                .map_err(|_| Error::Store("publication publisher index does not fit usize".to_string()))?;
            let publisher = self
                .publishers
                .get(publisher_index)
                .ok_or_else(|| Error::Store("publication publisher index is out of bounds".to_string()))?;
            let result = publisher.publish(path_info).await;
            observations.push(PublicationObservation {
                effect_id_blake3: effect.effect_id_blake3.clone(),
                publisher_index: effect.publisher_index,
                logical_path: effect.logical_path.clone(),
                disposition: if result.is_ok() {
                    PublicationDisposition::Succeeded
                } else {
                    PublicationDisposition::Failed
                },
                error: result.err(),
            });
        }
        assert_eq!(observations.len(), plan.effects.len());
        Ok(observations)
    }
}

impl TransferStore<'_> {
    #[must_use]
    pub fn output_dir_str(&self) -> &str {
        self.handle.output_dir_str()
    }

    pub async fn put_blob_bytes(&self, bytes: &[u8]) -> Result<snix_castore::B3Digest, Error> {
        use tokio::io::AsyncWriteExt;

        if bytes.is_empty() {
            return Err(Error::Store("transfer blob bytes must not be empty".to_string()));
        }
        let mut writer = self.handle.blob_service().open_write().await;
        writer
            .write_all(bytes)
            .await
            .map_err(|error| Error::Store(format!("writing transfer blob: {error}")))?;
        writer.close().await.map_err(|error| Error::Store(format!("closing transfer blob: {error}")))
    }

    pub async fn put_empty_directory(&self) -> Result<snix_castore::B3Digest, Error> {
        self.handle
            .directory_service()
            .put(snix_castore::Directory::new())
            .await
            .map_err(|error| Error::Store(format!("writing transfer directory: {error}")))
    }

    pub async fn ingest_host_path(&self, path: &Path) -> Result<Node, Error> {
        snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(
            self.handle.blob_service(),
            self.handle.directory_service(),
            path,
            None,
        )
        .await
        .map_err(|error| Error::Store(format!("ingesting transfer path {}: {error}", path.display())))
    }

    pub async fn ingest_nar_and_hash<R: tokio::io::AsyncRead + Unpin + Send>(
        &self,
        reader: &mut R,
    ) -> Result<TransferNarObservation, Error> {
        self.ingest_nar_and_hash_with_ca(reader, &None).await
    }

    pub async fn ingest_nar_and_hash_with_ca<R: tokio::io::AsyncRead + Unpin + Send>(
        &self,
        reader: &mut R,
        ca: &Option<nix_compat::nixhash::CAHash>,
    ) -> Result<TransferNarObservation, Error> {
        let (node, nar_sha256, nar_size_bytes) = snix_store::nar::ingest_nar_and_hash(
            self.handle.blob_service(),
            self.handle.directory_service(),
            reader,
            ca,
        )
        .await
        .map_err(|error| Error::Store(format!("ingesting transfer NAR: {error}")))?;
        Ok(TransferNarObservation {
            node,
            nar_sha256,
            nar_size_bytes,
        })
    }

    pub async fn export_node(&self, store_path: &StorePath<String>, node: &Node) -> Result<(), Error> {
        let host_path = store_path.to_absolute_path_with_prefix(self.handle.output_dir_str());
        if Path::new(&host_path).exists() {
            return Ok(());
        }
        crate::export_castore_to_disk(node, &host_path, &self.handle.blob_service(), &self.handle.directory_service())
            .await
            .map_err(|error| Error::Store(format!("exporting transfer node: {error}")))
    }

    pub async fn render_nar<W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        node: &Node,
        destination: &mut W,
    ) -> Result<(), Error> {
        self.handle.render_nar(node, destination).await
    }

    pub async fn copy_blob_to_path_bounded(
        &self,
        digest: &snix_castore::B3Digest,
        destination: &Path,
        bytes_max: u64,
    ) -> Result<u64, Error> {
        use tokio::io::AsyncReadExt;
        use tokio::io::AsyncWriteExt;

        if bytes_max == 0 {
            return Err(Error::Store("transfer blob byte bound must be positive".to_string()));
        }
        let reader = self
            .handle
            .blob_service()
            .open_read(digest)
            .await
            .map_err(|error| Error::Store(format!("opening transfer blob: {error}")))?
            .ok_or_else(|| Error::Store("transfer blob is missing".to_string()))?;
        let read_limit_bytes = bytes_max
            .checked_add(1)
            .ok_or_else(|| Error::Store("transfer blob byte bound overflow".to_string()))?;
        let mut blob_reader = reader.take(read_limit_bytes);
        let mut file = tokio::fs::File::create(destination)
            .await
            .map_err(|error| Error::Store(format!("creating transfer blob destination: {error}")))?;
        let copied_bytes = tokio::io::copy(&mut blob_reader, &mut file)
            .await
            .map_err(|error| Error::Store(format!("copying transfer blob: {error}")))?;
        if copied_bytes > bytes_max {
            return Err(Error::Store("transfer blob exceeds byte bound".to_string()));
        }
        file.flush().await.map_err(|error| Error::Store(format!("flushing transfer blob: {error}")))?;
        Ok(copied_bytes)
    }

    pub async fn directory_postcard_bytes(&self, digest: &snix_castore::B3Digest) -> Result<Vec<u8>, Error> {
        let directory = self
            .handle
            .directory_service()
            .get(digest)
            .await
            .map_err(|error| Error::Store(format!("reading transfer directory: {error}")))?
            .ok_or_else(|| Error::Store("transfer directory is missing".to_string()))?;
        postcard::to_stdvec(&snix_castore::proto::Directory::from(directory))
            .map_err(|error| Error::Store(format!("serializing transfer directory: {error}")))
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
        roots::register_root_with_registration(roots::RootRegistrationRequest {
            state_dir: &self.state_dir,
            store_dir: &self.store_dir,
            pathinfo: self.pathinfo_service.as_ref(),
            store_path,
            source,
            registration,
        })
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

impl AttestationStore {
    pub async fn open(config: crate::StoreConfig) -> Result<Self, Error> {
        StoreHandle::open(config).await.map(|handle| Self { handle })
    }

    #[must_use]
    pub fn state_dir(&self) -> &Path {
        self.handle.state_dir()
    }

    #[must_use]
    pub fn store_dir(&self) -> &str {
        self.handle.store_dir()
    }

    #[must_use]
    pub fn output_dir_str(&self) -> &str {
        self.handle.output_dir_str()
    }

    pub async fn get_artifact_attestation(
        &self,
        store_path: &StorePath<String>,
    ) -> Result<Option<crate::StoredArtifactAttestation>, Error> {
        self.handle.get_artifact_attestation(store_path).await
    }

    pub async fn runtime_closure_attestation(
        &self,
        roots: &[StorePath<String>],
    ) -> Result<crate::StoredClosureAttestation, Error> {
        self.handle.runtime_closure_attestation(roots).await
    }
}

impl SourceStore {
    pub async fn open(config: crate::StoreConfig) -> Result<Self, Error> {
        StoreHandle::open(config).await.map(|handle| Self { handle })
    }

    pub fn admission(&mut self) -> SourceAdmission<'_> {
        SourceAdmission {
            handle: &mut self.handle,
        }
    }
}

impl ForeignRealizationStore {
    pub async fn open(config: crate::StoreConfig) -> Result<Self, Error> {
        StoreHandle::open(config).await.map(|handle| Self { handle })
    }

    pub fn source_admission(&mut self) -> SourceAdmission<'_> {
        SourceAdmission {
            handle: &mut self.handle,
        }
    }

    #[must_use]
    pub fn into_pipeline_store_parts(self) -> PipelineStoreParts {
        self.handle.into_pipeline_store_parts()
    }

    pub async fn import_http_cache_closure_with_validator<F>(
        &self,
        cache_url: &url::Url,
        root: &StorePath<String>,
        options: &crate::PullOptions,
        validation: crate::HttpClosureImportValidation<F>,
    ) -> Result<crate::HttpClosurePullReport, Error>
    where
        F: FnOnce(&crate::HttpClosurePlan) -> Result<(), String>,
    {
        crate::import_http_cache_closure_with_validator(&self.handle, cache_url, root, options, validation).await
    }

    pub async fn export_cached_path_info(&mut self, path: &StorePath<String>) -> Result<Option<PathInfo>, Error> {
        self.handle.export_cached_path_info(path).await
    }
}

impl RustCacheStore {
    pub async fn open(config: crate::StoreConfig) -> Result<Self, Error> {
        let handle = StoreHandle::open(config).await?;
        Ok(Self {
            state_dir: handle.state_dir().to_path_buf(),
            blob_service: handle.blob_service(),
            directory_service: handle.directory_service(),
        })
    }

    pub async fn memory(state_dir: PathBuf) -> Result<Self, Error> {
        use snix_castore::directoryservice::RedbDirectoryService;
        use snix_castore::directoryservice::RedbDirectoryServiceConfig;

        if state_dir.as_os_str().is_empty() {
            return Err(Error::Store("Rust cache state directory must not be empty".to_string()));
        }
        let directory_service =
            RedbDirectoryService::new_temporary("rust-cache".to_string(), RedbDirectoryServiceConfig {
                path: None,
                read_only: false,
                cache_size: None,
            })
            .map_err(|error| Error::Store(format!("opening Rust cache directory service: {error}")))?;
        let blob_service = open_memory_blob_service("rust-cache").await?;
        Ok(Self {
            state_dir,
            blob_service,
            directory_service: Arc::new(directory_service),
        })
    }

    #[must_use]
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub async fn ingest_path(&self, path: &Path) -> Result<Node, Error> {
        snix_castore::import::fs::ingest_path::<_, _, _, &[u8]>(
            self.blob_service.clone(),
            self.directory_service.clone(),
            path,
            None,
        )
        .await
        .map_err(|error| Error::Store(format!("ingesting Rust cache path {}: {error}", path.display())))
    }

    pub async fn ingest_nar_and_hash<R: tokio::io::AsyncRead + Unpin + Send>(
        &self,
        reader: &mut R,
    ) -> Result<TransferNarObservation, Error> {
        let (node, nar_sha256, nar_size_bytes) = snix_store::nar::ingest_nar_and_hash(
            self.blob_service.clone(),
            self.directory_service.clone(),
            reader,
            &None,
        )
        .await
        .map_err(|error| Error::Store(format!("ingesting Rust cache NAR: {error}")))?;
        Ok(TransferNarObservation {
            node,
            nar_sha256,
            nar_size_bytes,
        })
    }

    pub async fn render_nar<W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        node: &Node,
        writer: W,
    ) -> Result<(), Error> {
        snix_store::nar::write_nar(writer, node, self.blob_service.clone(), self.directory_service.clone())
            .await
            .map_err(|error| Error::Store(format!("rendering Rust cache NAR: {error}")))
    }

    pub async fn has_complete_content(&self, node: &Node) -> Result<bool, Error> {
        crate::recursive_castore_completeness(self.blob_service.as_ref(), self.directory_service.as_ref(), node).await
    }

    pub async fn export_node(&self, node: &Node, destination: &Path) -> Result<(), Error> {
        let destination = destination
            .to_str()
            .ok_or_else(|| Error::Store("Rust cache destination path is not UTF-8".to_string()))?;
        crate::export_castore_to_disk(node, destination, &self.blob_service, &self.directory_service)
            .await
            .map_err(|error| Error::Store(format!("exporting Rust cache node: {error}")))
    }
}

impl ProvenanceStore {
    pub async fn open(config: crate::StoreConfig) -> Result<Self, Error> {
        StoreHandle::open(config).await.map(|handle| Self { handle })
    }

    pub async fn scan(
        &self,
        request: crate::CastoreProvenanceRequest<'_>,
    ) -> Result<crate::CastoreProvenanceScan, Error> {
        crate::scan_castore_provenance(&self.handle, request).await
    }
}

impl PathInfoAdministration {
    pub async fn open(state_dir: &Path, is_read_only: bool) -> Result<Self, Error> {
        use snix_store::pathinfoservice::RedbPathInfoService;
        use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

        let database_path = state_dir.join("pathinfo.redb");
        if !database_path.is_file() {
            return Err(Error::Store(format!(
                "PathInfo database {} does not exist (no builds yet?)",
                database_path.display()
            )));
        }
        let service = RedbPathInfoService::new("crunch".to_string(), RedbPathInfoServiceConfig {
            path: Some(database_path.clone()),
            read_only: is_read_only,
            cache_size: None,
        })
        .await
        .map_err(|error| Error::Store(format!("opening PathInfo database {}: {error}", database_path.display())))?;
        Ok(Self {
            service: Arc::new(service),
        })
    }

    pub async fn pathinfos(&self) -> Result<Vec<PathInfo>, Error> {
        use futures::StreamExt;

        let mut stream = self.service.list();
        let mut pathinfos = Vec::with_capacity(PATHINFO_ADMIN_INITIAL_CAPACITY);
        for _entry_index in 0..PATHINFO_ADMIN_SCAN_MAX {
            let Some(result) = stream.next().await else {
                return Ok(pathinfos);
            };
            pathinfos.push(result.map_err(|error| Error::PathInfoService(format!("listing PathInfo: {error}")))?);
        }
        if stream.next().await.is_some() {
            return Err(Error::Store(format!(
                "PathInfo administration scan exceeds {PATHINFO_ADMIN_SCAN_MAX} entries"
            )));
        }
        Ok(pathinfos)
    }

    pub async fn verify(&self, path_filter: Option<&str>, store_dir: &Path) -> Result<Vec<crate::VerifyResult>, Error> {
        crate::store_verify(self.service.as_ref(), path_filter, store_dir).await
    }

    pub async fn verify_signatures(
        &self,
        path_filter: Option<&str>,
        trusted_keys: &[nix_compat::narinfo::VerifyingKey],
        store_dir: &str,
    ) -> Result<Vec<crate::SignatureVerifyResult>, Error> {
        crate::store_verify_signatures(self.service.as_ref(), path_filter, trusted_keys, store_dir).await
    }

    pub async fn sign(
        &self,
        signing_key: &nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey>,
        path_filter: Option<&str>,
        is_sign_all: bool,
        store_dir: &str,
    ) -> Result<Vec<crate::SignResult>, Error> {
        crate::store_sign(self.service.as_ref(), signing_key, path_filter, is_sign_all, store_dir).await
    }
}

impl StoreAdministration {
    pub async fn open(config: crate::StoreConfig) -> Result<Self, Error> {
        StoreHandle::open(config).await.map(|handle| Self { handle })
    }

    #[must_use]
    pub fn store_dir(&self) -> &str {
        self.handle.store_dir()
    }

    #[must_use]
    pub fn state_dir(&self) -> &Path {
        self.handle.state_dir()
    }

    pub async fn list_pathinfos_with_layer(&self) -> Result<Vec<crate::layer::Layered<PathInfo>>, Error> {
        self.handle.list_pathinfos_with_layer().await
    }

    pub fn list_retained_roots(&self) -> Result<Vec<GcRootRecord>, Error> {
        self.handle.list_retained_roots()
    }

    pub async fn pin_retained_root(&self, logical_path: &str) -> Result<GcRootRecord, Error> {
        self.handle.pin_retained_root(logical_path).await
    }

    pub fn unpin_retained_root(&self, logical_path: &str) -> Result<Option<GcRootRecord>, Error> {
        self.handle.unpin_retained_root(logical_path)
    }

    pub fn overlay_report(&self) -> Result<Option<crate::StoreOverlayReport>, Error> {
        self.handle.overlay_report()
    }

    pub async fn pathinfos(&self) -> Result<Vec<PathInfo>, Error> {
        self.handle
            .list_pathinfos_with_layer()
            .await
            .map(|entries| entries.into_iter().map(|entry| entry.value).collect())
    }

    pub async fn inspect_final_nar_repair(
        &self,
        logical_store_path: &str,
    ) -> Result<crate::FinalNarRepairInspection, Error> {
        crate::inspect_final_nar_repair(&self.handle, logical_store_path).await
    }

    pub async fn execute_final_nar_repair(
        &self,
        inspection: crate::FinalNarRepairInspection,
        signing_key: &nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey>,
    ) -> Result<crate::FinalNarRepairReport, Error> {
        crate::execute_final_nar_repair(&self.handle, inspection, signing_key).await
    }

    pub async fn export_paths_to_cache_dir(
        &self,
        paths: &[PathInfo],
        destination: &Path,
        options: &crate::PushOptions,
    ) -> Result<crate::PushReport, Error> {
        crate::export_paths_to_cache_dir(&self.handle, paths, destination, options).await
    }

    pub async fn import_paths_from_cache_dir(
        &self,
        source: &Path,
        paths_filter: Option<&[String]>,
        options: &crate::PullOptions,
    ) -> Result<crate::PullReport, Error> {
        crate::import_paths_from_cache_dir(&self.handle, source, paths_filter, options).await
    }

    pub async fn import_paths_from_http_cache(
        &self,
        cache_url: &url::Url,
        paths: &[StorePath<String>],
        options: &crate::PullOptions,
    ) -> Result<crate::PullReport, Error> {
        crate::import_paths_from_http_cache(&self.handle, cache_url, paths, options).await
    }

    pub async fn import_http_cache_closure(
        &self,
        cache_url: &url::Url,
        root: &StorePath<String>,
        options: &crate::PullOptions,
        limits: crate::HttpClosureLimits,
    ) -> Result<crate::HttpClosurePullReport, Error> {
        crate::import_http_cache_closure(&self.handle, cache_url, root, options, limits).await
    }

    pub async fn realize_composition(
        &self,
        request: &crunch_composition_core::CompositionRequest,
    ) -> Result<crunch_composition_core::RealizationReceipt, Error> {
        crate::realize_composition(&self.handle, request).await
    }

    pub async fn export_store_archive<W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        roots: &[PathInfo],
        writer: &mut W,
        options: &crate::ArchiveExportOptions,
    ) -> Result<crate::ArchiveExportReport, Error> {
        crate::export_store_archive(&self.handle, roots, writer, options).await
    }

    pub async fn import_store_archive<R: tokio::io::AsyncRead + Unpin + Send>(
        &self,
        reader: &mut R,
        options: &crate::ArchiveImportOptions,
    ) -> Result<crate::ArchiveImportReport, Error> {
        crate::import_store_archive(&self.handle, reader, options).await
    }

    pub async fn import_nario_v2<R: tokio::io::AsyncRead + Unpin + Send>(
        &self,
        reader: &mut R,
        options: &crate::NarioV2ImportOptions,
    ) -> Result<crate::NarioV2ImportReport, Error> {
        crate::import_nario_v2(&self.handle, reader, options).await
    }

    pub fn admin(&mut self) -> StoreAdmin<'_> {
        StoreAdmin {
            handle: &mut self.handle,
        }
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
    pub async fn memory() -> Result<Self, Error> {
        use snix_castore::directoryservice::RedbDirectoryService;
        use snix_castore::directoryservice::RedbDirectoryServiceConfig;

        let directory_service =
            RedbDirectoryService::new_temporary("build-service-memory".to_string(), RedbDirectoryServiceConfig {
                path: None,
                read_only: false,
                cache_size: None,
            })
            .map_err(|error| Error::Store(format!("opening build-service memory directory: {error}")))?;
        let blob_service = open_memory_blob_service("build-service-memory").await?;
        Ok(Self {
            blob_service,
            directory_service: Arc::new(directory_service),
        })
    }

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
    const HEX_CHARS_PER_BYTE: usize = 2;
    const TEST_PUBLISHER_COUNT: u32 = 1;

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
    async fn publication_plan_precedes_typed_publisher_observations() {
        let publisher = Arc::new(crate::RecordingPublisher::new());
        let publishers: Vec<Arc<dyn Publisher>> = vec![publisher.clone()];
        let path_info = path_info(
            StorePath::from_name_and_digest_fixed(
                "publication-output",
                [TEST_PATH_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
            )
            .expect("valid publication path"),
        );
        assert_eq!(u32::try_from(publishers.len()).ok(), Some(TEST_PUBLISHER_COUNT));
        let plan = plan_publication(&path_info, TEST_PUBLISHER_COUNT).expect("publication plan");
        let execution = PublicationExecution {
            publishers: &publishers,
        };

        assert_eq!(publisher.call_count(), 0);
        assert_eq!(plan.effects.len(), publishers.len());
        let observations = execution.execute(&plan, &path_info).await.expect("execute publication plan");
        assert_eq!(publisher.call_count(), 1);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].disposition, PublicationDisposition::Succeeded);
    }

    #[tokio::test]
    async fn publisher_failure_becomes_typed_observation_after_admission_plan() {
        let publisher = Arc::new(crate::RecordingPublisher::new());
        publisher.fail_next();
        let publishers: Vec<Arc<dyn Publisher>> = vec![publisher.clone()];
        let path_info = path_info(
            StorePath::from_name_and_digest_fixed(
                "failed-publication-output",
                [TEST_PATH_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
            )
            .expect("valid publication path"),
        );
        assert_eq!(u32::try_from(publishers.len()).ok(), Some(TEST_PUBLISHER_COUNT));
        let plan = plan_publication(&path_info, TEST_PUBLISHER_COUNT).expect("publication plan");
        let execution = PublicationExecution {
            publishers: &publishers,
        };

        let observations = execution.execute(&plan, &path_info).await.expect("observe publisher failure");
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].disposition, PublicationDisposition::Failed);
        assert!(observations[0].error.as_deref().is_some_and(|error| error.contains("simulated")));
        assert_eq!(publisher.call_count(), 0);
    }

    #[tokio::test]
    async fn tampered_publication_effect_fails_plan_identity_before_execution() {
        let publisher = Arc::new(crate::RecordingPublisher::new());
        let publishers: Vec<Arc<dyn Publisher>> = vec![publisher.clone()];
        let path_info = path_info(
            StorePath::from_name_and_digest_fixed(
                "wrong-publication-output",
                [TEST_PATH_DIGEST_BYTE; nix_compat::store_path::DIGEST_SIZE],
            )
            .expect("valid publication path"),
        );
        assert_eq!(u32::try_from(publishers.len()).ok(), Some(TEST_PUBLISHER_COUNT));
        let mut plan = plan_publication(&path_info, TEST_PUBLISHER_COUNT).expect("publication plan");
        plan.effects[0].effect_id_blake3 = "0".repeat(blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
        let execution = PublicationExecution {
            publishers: &publishers,
        };

        let error = execution.execute(&plan, &path_info).await.expect_err("tampered effect must fail");
        assert!(error.to_string().contains("publication plan BLAKE3 mismatch"));
        assert_eq!(publisher.call_count(), 0);
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
