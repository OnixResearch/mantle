//! Store-owned Nix gateway adapter. Request authority and signature admission
//! remain application responsibilities; this capability grants neither.

use tokio::io::AsyncReadExt as _;

/// Object exchange authority, without build, GC, signing, or raw services.
///
/// ```compile_fail
/// fn denied(store: &crunch_store::GatewayStore) { store.blob_service(); }
/// ```
/// ```compile_fail
/// fn denied(store: &crunch_store::GatewayStore) { store.directory_service(); }
/// ```
/// ```compile_fail
/// fn denied(store: &crunch_store::GatewayStore) { store.pathinfo_service(); }
/// ```
/// ```compile_fail
/// fn denied(store: &mut crunch_store::GatewayStore) { store.store_admin(); }
/// ```
/// ```compile_fail
/// fn denied(store: &mut crunch_store::GatewayStore) { store.source_admission(); }
/// ```
/// ```compile_fail
/// fn denied(store: crunch_store::GatewayStore) { store.into_pipeline_store_parts(); }
/// ```
pub struct GatewayStore {
    handle: crate::StoreHandle,
    instance: std::sync::Arc<()>,
}

/// Observed NAR data. Only an executed store ingest can construct this value.
/// It is not signature admission or a publication receipt.
pub struct GatewayNar {
    observation: crate::TransferNarObservation,
    instance: std::sync::Arc<()>,
}

impl GatewayNar {
    pub fn observation(&self) -> &crate::TransferNarObservation {
        &self.observation
    }
}

/// Imported metadata paired with an actual ingest observation.
/// The application must verify signatures and current authority before calling
/// persistence. Nix PathInfo is an interchange value at this shell boundary.
pub struct GatewayImportRequest {
    pub path_info: snix_store::path_info::PathInfo,
    pub received: GatewayNar,
}

impl GatewayStore {
    // r[impl native_package_parity.gateway]
    pub async fn open(config: crate::StoreConfig) -> Result<Self, crate::Error> {
        crate::StoreHandle::open(config).await.map(|handle| Self {
            handle,
            instance: std::sync::Arc::new(()),
        })
    }

    pub async fn find(
        &self,
        path: &nix_compat::store_path::StorePath<String>,
    ) -> Result<Option<snix_store::path_info::PathInfo>, crate::Error> {
        self.handle.path_info_with_layer(path).await.map(|found| found.map(|entry| entry.value))
    }

    pub async fn find_by_digest(
        &self,
        digest: [u8; nix_compat::store_path::DIGEST_SIZE],
    ) -> Result<Option<snix_store::path_info::PathInfo>, crate::Error> {
        let found = self
            .handle
            .pathinfo_service()
            .get(digest)
            .await
            .map_err(|error| crate::Error::Store(format!("gateway digest lookup: {error}")))?;
        let Some(found) = found else {
            return Ok(None);
        };
        if *found.store_path.digest() != digest {
            return Err(crate::Error::Store("nix-gateway-pathinfo-digest-collision".to_string()));
        }
        debug_assert_eq!(*found.store_path.digest(), digest);
        debug_assert!(!found.store_path.name().is_empty());
        // Resolve again through composed lookup, retaining shadow and generation checks.
        self.find(&found.store_path).await
    }

    pub async fn ingest_nar<R: tokio::io::AsyncRead + Unpin + Send>(
        &self,
        reader: &mut R,
        ca: &Option<nix_compat::nixhash::CAHash>,
        nar_size_bytes_max: u64,
    ) -> Result<GatewayNar, crate::Error> {
        if nar_size_bytes_max == 0 {
            return Err(crate::Error::Store("gateway NAR byte limit must be positive".to_string()));
        }
        debug_assert!(nar_size_bytes_max > 0);
        let mut bounded = reader.take(nar_size_bytes_max.saturating_add(1));
        let observation = self.handle.transfer_store().ingest_nar_and_hash_with_ca(&mut bounded, ca).await?;
        if observation.nar_size_bytes > nar_size_bytes_max {
            return Err(crate::Error::Store("gateway NAR exceeds byte limit".to_string()));
        }
        debug_assert!(observation.nar_size_bytes > 0);
        Ok(GatewayNar {
            observation,
            instance: self.instance.clone(),
        })
    }

    pub async fn persist_import(&mut self, request: GatewayImportRequest) -> Result<(), crate::Error> {
        if !std::sync::Arc::ptr_eq(&self.instance, &request.received.instance) {
            return Err(crate::Error::Store(
                "gateway import observation belongs to another store instance".to_string(),
            ));
        }
        let path_info = request.path_info;
        let observed = request.received.observation;
        if path_info.node != observed.node {
            return Err(crate::Error::Store("gateway import node differs from observed NAR".to_string()));
        }
        if path_info.nar_sha256 != observed.nar_sha256 {
            return Err(crate::Error::Store("gateway import digest differs from observed NAR".to_string()));
        }
        if path_info.nar_size != observed.nar_size_bytes {
            return Err(crate::Error::Store("gateway import size differs from observed NAR".to_string()));
        }
        debug_assert_eq!(path_info.node, observed.node);
        debug_assert_eq!(path_info.nar_sha256, observed.nar_sha256);
        let output_path = path_info.store_path.clone();
        let output_name = output_path.name().to_string();
        self.handle
            .persist_and_export_signed_output(crate::PersistOutputRequest {
                output_name: &output_name,
                output_path: &output_path,
                final_node: observed.node,
                path_info,
                provenance: None,
                is_root: false,
                root_source: None,
            })
            .await
            .map(|_| ())
    }
}
