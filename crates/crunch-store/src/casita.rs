//! Verified reads of the durable Casita output-root namespace.
//!
//! The admission path uses this same root layout and must never replace a
//! root through an unconditional filesystem import.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use casita::experimental::ChunkedBlobStore;
use casita::experimental::ConditionalPublishResult;
use casita::experimental::MetadataStore;
use casita::experimental::ObjectKey;
use casita::experimental::Repository;
use casita::experimental::RootChange;
use casita::experimental::RootExpectation;
use casita::experimental::RootName;
use casita::experimental::TursoMetadataStore;
use casita::import::UnrootedFilesystemImport;
use data_encoding::HEXLOWER;
use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_castore::import::fs::ingest_path;
use snix_store::nar::NarCalculationService;
use snix_store::nar::SimpleRenderer;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;
use crate::export::export_castore_to_disk;
use crate::overlay;
use crate::path_identity::require_ca_path_identity;

pub(crate) type LocalRepository = Repository<ChunkedBlobStore, TursoMetadataStore>;
const OUTPUT_ROOT_PREFIX: &str = "mantle/outputs/";
const CASTORE_ROOT_PREFIX: &str = "mantle/castore/";
const MAX_TRUST_FILE_BYTES: u64 = 65_536;
const MAX_PATHINFO_BYTES: usize = crate::archive::MAX_ARCHIVE_METADATA_BYTES;
const TRUST_FILE_NAME: &str = "casita-trusted-public-keys";

#[cfg(test)]
struct AdmissionRaceGate {
    staged: tokio::sync::Barrier,
    resume: tokio::sync::Barrier,
    staged_targets: tokio::sync::Mutex<Vec<ObjectKey>>,
}

/// One owned handle to the durable Casita repository.
#[derive(Clone)]
pub(crate) struct CasitaStore {
    pub(crate) repository: Arc<LocalRepository>,
    state_dir: PathBuf,
    store_dir: String,
    pub(crate) blob_service: Arc<dyn BlobService>,
    pub(crate) directory_service: Arc<dyn DirectoryService>,
    observed_roots: Arc<Mutex<BTreeMap<[u8; 20], ObjectKey>>>,
    #[cfg(test)]
    before_output_publish: Arc<tokio::sync::Mutex<Option<Arc<AdmissionRaceGate>>>>,
}

impl CasitaStore {
    pub(crate) async fn open(state_dir: &Path, store_dir: &str) -> Result<Arc<Self>, Error> {
        Self::external_trusted_keys(state_dir)?;
        let repository = LocalRepository::local(state_dir.join("casita"))
            .await
            .map_err(|error| Error::Store(format!("opening Casita repository: {error}")))?;
        let blob_service = snix_castore::blobservice::from_addr("memory:")
            .await
            .map_err(|error| Error::Store(format!("opening Casita session blobs: {error}")))?;
        let directory_service: Arc<dyn DirectoryService> = Arc::new(
            RedbDirectoryService::new_temporary("casita-session".to_string(), RedbDirectoryServiceConfig {
                path: None,
                read_only: false,
                cache_size: None,
            })
            .map_err(|error| Error::Store(format!("opening Casita session directories: {error}")))?,
        );
        Ok(Arc::new(Self {
            repository: Arc::new(repository),
            state_dir: state_dir.to_path_buf(),
            store_dir: store_dir.to_owned(),
            blob_service,
            directory_service,
            observed_roots: Arc::new(Mutex::new(BTreeMap::new())),
            #[cfg(test)]
            before_output_publish: Arc::new(tokio::sync::Mutex::new(None)),
        }))
    }

    pub(crate) fn root_name(path: &StorePath<String>) -> Result<RootName, Error> {
        Self::root_name_for_digest(path.digest())
    }

    fn root_name_for_digest(digest: &[u8; 20]) -> Result<RootName, Error> {
        RootName::try_from(format!("{OUTPUT_ROOT_PREFIX}{}", HEXLOWER.encode(digest)))
            .map_err(|error| Error::Store(format!("invalid Casita output root name: {error}")))
    }
    pub(crate) fn castore_root_name(node: &Node) -> Result<RootName, Error> {
        let encoded =
            postcard::to_stdvec(node).map_err(|error| Error::Store(format!("encoding castore root node: {error}")))?;
        let digest = blake3::hash(&encoded);
        RootName::try_from(format!("{CASTORE_ROOT_PREFIX}{}", HEXLOWER.encode(digest.as_bytes())))
            .map_err(|error| Error::Store(format!("invalid castore root name: {error}")))
    }

    fn external_trusted_keys(state_dir: &Path) -> Result<Vec<nix_compat::narinfo::VerifyingKey>, Error> {
        let policy_path = state_dir.join(TRUST_FILE_NAME);
        match std::fs::symlink_metadata(&policy_path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(Error::Store(format!(
                    "casita-trust-policy-invalid: inspecting {}: {error}",
                    policy_path.display(),
                )));
            }
        }
        let keys = overlay::parse_public_keys(&policy_path, MAX_TRUST_FILE_BYTES)
            .map_err(|error| Error::Store(format!("casita-trust-policy-invalid: {error}")))?;
        if keys.is_empty() {
            return Err(Error::Store("casita-trust-policy-invalid: policy is empty".to_string()));
        }
        Ok(keys)
    }

    pub(crate) fn require_recovered_gc(&self) -> Result<(), Error> {
        if crate::gc::casita_gc_fence_pending(&self.state_dir)? {
            return Err(Error::Gc(
                "gc-recovery-required: recover fenced Casita GC under the selected store mutation guard".to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) fn preflight_archive_trust(
        &self,
        import_keys: &[nix_compat::narinfo::VerifyingKey],
    ) -> Result<(), Error> {
        let allowed = Self::external_trusted_keys(&self.state_dir)?;
        if allowed.is_empty() || import_keys.is_empty() {
            return Err(Error::Store(format!(
                "casita-trust-policy-missing: provision {TRUST_FILE_NAME} and pass --trusted-public-keys before importing"
            )));
        }
        for key in import_keys {
            if !allowed.iter().any(|candidate| candidate == key) {
                return Err(Error::Store(format!("casita-import-key-unauthorized: {}", key)));
            }
        }
        Ok(())
    }
    fn verify_signer(&self, info: &PathInfo) -> Result<(), Error> {
        let mut trusted = Self::external_trusted_keys(&self.state_dir)?;
        if trusted.is_empty()
            && let Some(local) = overlay::load_local_verifying_key(&self.state_dir)?
        {
            trusted.push(local);
        }
        let references = info.references.iter().map(StorePath::as_ref).collect::<Vec<_>>();
        let fingerprint = nix_compat::narinfo::fingerprint_with_store_dir(
            &info.store_path.as_ref(),
            &info.nar_sha256,
            info.nar_size,
            references.iter(),
            &self.store_dir,
        );
        if !info
            .signatures
            .iter()
            .any(|signature| trusted.iter().any(|key| key.verify(&fingerprint, &signature.as_ref())))
        {
            return Err(Error::Store(format!(
                "casita-signer-untrusted: {} has no valid signature from destination trust policy",
                info.store_path
            )));
        }
        Ok(())
    }

    pub(crate) async fn read_root_pathinfo(
        &self,
        name: &RootName,
        expected: &ObjectKey,
        is_recording_observation: bool,
    ) -> Result<PathInfo, Error> {
        let snapshot = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading Casita root: {error}")))?;
        let current = snapshot
            .root(name)
            .await
            .map_err(|error| Error::Store(format!("reading Casita root {}: {error}", name.as_str())))?;
        if current.as_ref() != Some(expected) {
            return Err(Error::Store(format!("casita-root-conflict: {} changed before checkout", name.as_str())));
        }
        let scratch =
            tempfile::tempdir().map_err(|error| Error::Store(format!("creating Casita read scratch: {error}")))?;
        let envelope = scratch.path().join("envelope");
        self.repository
            .checkout(expected, &envelope)
            .await
            .map_err(|error| Error::Store(format!("checking out Casita root {}: {error}", name.as_str())))?;
        let mut entries = std::fs::read_dir(&envelope)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: reading envelope: {error}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: listing envelope: {error}")))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        let names = entries.iter().map(|entry| entry.file_name()).collect::<Vec<_>>();
        if names != ["content", "pathinfo.json"].map(std::ffi::OsString::from) {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: {} must contain only content and pathinfo.json",
                name.as_str()
            )));
        }
        let pathinfo_file = envelope.join("pathinfo.json");
        if !std::fs::symlink_metadata(&pathinfo_file)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: metadata file: {error}")))?
            .file_type()
            .is_file()
        {
            return Err(Error::Store("casita-envelope-invalid: pathinfo.json is not a regular file".to_string()));
        }
        let read_limit_bytes = u64::try_from(MAX_PATHINFO_BYTES)
            .ok()
            .and_then(|maximum_bytes| maximum_bytes.checked_add(1))
            .ok_or_else(|| Error::Store("casita-envelope-invalid: PathInfo read limit overflow".to_string()))?;
        let mut encoded = Vec::new();
        std::fs::File::open(&pathinfo_file)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: opening PathInfo: {error}")))?
            .take(read_limit_bytes)
            .read_to_end(&mut encoded)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: reading PathInfo: {error}")))?;
        if encoded.is_empty() || encoded.len() > MAX_PATHINFO_BYTES {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: PathInfo size is outside 1..={MAX_PATHINFO_BYTES}"
            )));
        }
        let info: PathInfo = serde_json::from_slice(&encoded)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: decoding PathInfo: {error}")))?;
        if serde_json::to_vec(&info)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: reencoding PathInfo: {error}")))?
            != encoded
        {
            return Err(Error::Store("casita-envelope-invalid: PathInfo is not canonical JSON".to_string()));
        }
        if Self::root_name(&info.store_path)? != *name {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: root {} disagrees with PathInfo store path",
                name.as_str()
            )));
        }
        require_ca_path_identity(&info, &self.store_dir)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: {error}")))?;
        self.verify_signer(&info)?;
        let node = ingest_path(
            self.blob_service.clone(),
            self.directory_service.clone(),
            envelope.join("content"),
            None::<&snix_castore::refscan::ReferenceScanner<&[u8]>>,
        )
        .await
        .map_err(|error| Error::Store(format!("casita-envelope-invalid: ingesting content: {error}")))?;
        if node != info.node {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: {} node disagrees with content",
                name.as_str()
            )));
        }
        let (size, sha256) = SimpleRenderer::new(self.blob_service.clone(), self.directory_service.clone())
            .calculate_nar(&node)
            .await
            .map_err(|error| Error::Store(format!("casita-nar-mismatch: measuring content: {error}")))?;
        if size != info.nar_size || sha256 != info.nar_sha256 {
            return Err(Error::Store(format!("casita-nar-mismatch: {} differs from signed PathInfo", name.as_str())));
        }
        let latest = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading Casita root after checkout: {error}")))?;
        if latest
            .root(name)
            .await
            .map_err(|error| Error::Store(format!("reading Casita root after checkout: {error}")))?
            .as_ref()
            != Some(expected)
        {
            return Err(Error::Store(format!("casita-root-conflict: {} changed during checkout", name.as_str())));
        }
        if is_recording_observation {
            self.observed_roots
                .lock()
                .map_err(|_| Error::Store("Casita root observation mutex poisoned".to_string()))?
                .insert(*info.store_path.digest(), expected.clone());
        }
        Ok(info)
    }

    pub(crate) async fn verified_gc_roots(
        &self,
        store_dir: &str,
    ) -> Result<Vec<(RootName, ObjectKey, PathInfo)>, Error> {
        self.verified_output_roots(store_dir, false).await
    }

    async fn verified_output_roots(
        &self,
        store_dir: &str,
        is_recording_observation: bool,
    ) -> Result<Vec<(RootName, ObjectKey, PathInfo)>, Error> {
        if store_dir != self.store_dir {
            return Err(Error::Store("casita-root-conflict: GC store prefix differs from opened store".to_string()));
        }
        let snapshot = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading Casita GC roots: {error}")))?;
        let mut roots = snapshot.roots();
        // Total GC inventory limit, distinct from Casita's per-mutation max_root_changes.
        let mut records = Vec::new();
        while let Some(record) =
            roots.try_next().await.map_err(|error| Error::Store(format!("reading Casita GC roots: {error}")))?
        {
            if record.name().as_str().starts_with(OUTPUT_ROOT_PREFIX) {
                if records.len() >= crunch_gc_core::MAX_GC_ENTRIES {
                    return Err(Error::Store(
                        "casita-gc-inventory-limit: output root count exceeds the GC entry limit".to_string(),
                    ));
                }
                let info = self.read_root_pathinfo(record.name(), record.target(), is_recording_observation).await?;
                records.push((record.name().clone(), record.target().clone(), info));
            }
        }
        Ok(records)
    }
    async fn read_castore_root(&self, name: &RootName, target: &ObjectKey) -> Result<Node, Error> {
        let snapshot = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading castore root {}: {error}", name.as_str())))?;
        if snapshot
            .root(name)
            .await
            .map_err(|error| Error::Store(format!("reading castore root {}: {error}", name.as_str())))?
            .as_ref()
            != Some(target)
        {
            return Err(Error::Store(format!("casita-root-conflict: castore root {} changed", name.as_str())));
        }
        let scratch =
            tempfile::tempdir().map_err(|error| Error::Store(format!("creating castore read scratch: {error}")))?;
        let envelope = scratch.path().join("envelope");
        self.repository
            .checkout(target, &envelope)
            .await
            .map_err(|error| Error::Store(format!("checking out castore root {}: {error}", name.as_str())))?;
        let mut names = std::fs::read_dir(&envelope)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: reading castore envelope: {error}")))?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: listing castore envelope: {error}")))?;
        names.sort();
        if names != ["content", "node.postcard"].map(std::ffi::OsString::from) {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: castore root {} has unexpected entries",
                name.as_str()
            )));
        }
        let node_path = envelope.join("node.postcard");
        if !std::fs::symlink_metadata(&node_path)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: castore node file: {error}")))?
            .file_type()
            .is_file()
        {
            return Err(Error::Store("casita-envelope-invalid: node.postcard is not a regular file".to_string()));
        }
        let encoded = std::fs::read(&node_path)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: reading castore node: {error}")))?;
        let node: Node = postcard::from_bytes(&encoded)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: decoding castore node: {error}")))?;
        if Self::castore_root_name(&node)? != *name {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: castore root {} disagrees with node",
                name.as_str()
            )));
        }
        let observed = ingest_path(
            self.blob_service.clone(),
            self.directory_service.clone(),
            envelope.join("content"),
            None::<&snix_castore::refscan::ReferenceScanner<&[u8]>>,
        )
        .await
        .map_err(|error| Error::Store(format!("casita-envelope-invalid: ingesting castore content: {error}")))?;
        if observed != node {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: castore root {} content differs from node",
                name.as_str()
            )));
        }
        let latest = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading castore root after checkout: {error}")))?;
        if latest
            .root(name)
            .await
            .map_err(|error| Error::Store(format!("reading castore root after checkout: {error}")))?
            .as_ref()
            != Some(target)
        {
            return Err(Error::Store(format!(
                "casita-root-conflict: castore root {} changed during checkout",
                name.as_str()
            )));
        }
        Ok(node)
    }

    pub(crate) async fn verified_gc_castore_roots(&self) -> Result<Vec<(RootName, ObjectKey, Node, u64)>, Error> {
        let snapshot = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading Casita castore GC roots: {error}")))?;
        let mut roots = snapshot.roots();
        let mut records = Vec::new();
        while let Some(record) = roots
            .try_next()
            .await
            .map_err(|error| Error::Store(format!("reading Casita castore GC roots: {error}")))?
        {
            if record.name().as_str().starts_with(CASTORE_ROOT_PREFIX) {
                if records.len() >= crunch_gc_core::MAX_GC_ENTRIES {
                    return Err(Error::Store(
                        "casita-gc-inventory-limit: castore root count exceeds the GC entry limit".to_string(),
                    ));
                }
                let node = self.read_castore_root(record.name(), record.target()).await?;
                let (nar_size, _) = SimpleRenderer::new(self.blob_service.clone(), self.directory_service.clone())
                    .calculate_nar(&node)
                    .await
                    .map_err(|error| {
                        Error::Store(format!(
                            "casita-envelope-invalid: measuring castore root {}: {error}",
                            record.name().as_str(),
                        ))
                    })?;
                records.push((record.name().clone(), record.target().clone(), node, nar_size));
            }
        }
        Ok(records)
    }

    pub(crate) async fn rehydrate_castore_payload_root(&self, node: &Node) -> Result<(), Error> {
        self.require_recovered_gc()?;
        let name = Self::castore_root_name(node)?;
        let snapshot = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading retained castore root: {error}")))?;
        let target = snapshot
            .root(&name)
            .await
            .map_err(|error| Error::Store(format!("reading retained castore root: {error}")))?
            .ok_or_else(|| Error::Store(format!("casita-root-missing: {}", name.as_str())))?;
        if self.read_castore_root(&name, &target).await? != *node {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: retained castore node {} differs",
                name.as_str()
            )));
        }
        Ok(())
    }

    pub(crate) async fn admit_castore_payload_root(&self, node: &Node) -> Result<(), Error> {
        self.require_recovered_gc()?;
        let name = Self::castore_root_name(node)?;
        let snapshot = self
            .repository
            .metadata()
            .snapshot()
            .await
            .map_err(|error| Error::Store(format!("reading castore admission root: {error}")))?;
        if let Some(target) = snapshot
            .root(&name)
            .await
            .map_err(|error| Error::Store(format!("reading castore admission root: {error}")))?
        {
            return if self.read_castore_root(&name, &target).await? == *node {
                Ok(())
            } else {
                Err(Error::Store(format!("casita-root-conflict: {}", name.as_str())))
            };
        }
        let scratch = tempfile::tempdir()
            .map_err(|error| Error::Store(format!("creating castore admission scratch: {error}")))?;
        let envelope = scratch.path().join("envelope");
        std::fs::create_dir(&envelope).map_err(|error| Error::Store(format!("creating castore envelope: {error}")))?;
        export_castore_to_disk(
            node,
            envelope
                .join("content")
                .to_str()
                .ok_or_else(|| Error::Store("castore scratch path is not UTF-8".to_string()))?,
            &self.blob_service,
            &self.directory_service,
        )
        .await
        .map_err(|error| Error::Store(format!("exporting castore payload: {error}")))?;
        let encoded =
            postcard::to_stdvec(node).map_err(|error| Error::Store(format!("encoding castore node: {error}")))?;
        std::fs::write(envelope.join("node.postcard"), encoded)
            .map_err(|error| Error::Store(format!("writing castore node: {error}")))?;
        let observed = ingest_path(
            self.blob_service.clone(),
            self.directory_service.clone(),
            envelope.join("content"),
            None::<&snix_castore::refscan::ReferenceScanner<&[u8]>>,
        )
        .await
        .map_err(|error| Error::Store(format!("casita-envelope-invalid: staged castore node: {error}")))?;
        if observed != *node {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: staged castore root {} changed node",
                name.as_str()
            )));
        }
        let session = self
            .repository
            .mutation_session()
            .await
            .map_err(|error| Error::Store(format!("opening castore mutation session: {error}")))?;
        let target = session
            .import(UnrootedFilesystemImport::new(&envelope))
            .await
            .map_err(|error| Error::Store(format!("staging castore root {}: {error}", name.as_str())))?;
        match session
            .publish_if_roots_match(
                Vec::new(),
                vec![RootExpectation {
                    name: name.clone(),
                    target: None,
                }],
                vec![RootChange::Set {
                    name: name.clone(),
                    target: target.clone(),
                }],
            )
            .await
            .map_err(|error| Error::Store(format!("publishing castore root {}: {error}", name.as_str())))?
        {
            ConditionalPublishResult::RootMismatch { .. } => {
                Err(Error::Store(format!("casita-root-conflict: {}", name.as_str())))
            }
            ConditionalPublishResult::Committed(_) => {
                if self.read_castore_root(&name, &target).await? == *node {
                    Ok(())
                } else {
                    Err(Error::Store(format!("casita-envelope-invalid: published castore root {}", name.as_str())))
                }
            }
        }
    }

    fn validate_candidate(&self, info: &PathInfo) -> Result<(), Error> {
        if info.signatures.is_empty() {
            return Err(Error::Store(format!("casita-signer-untrusted: {} is unsigned", info.store_path)));
        }
        require_ca_path_identity(info, &self.store_dir)
            .map_err(|error| Error::Store(format!("casita-envelope-invalid: {error}")))?;
        self.verify_signer(info)
    }

    async fn prepare_output_envelope(&self, info: &PathInfo, envelope: &Path) -> Result<(), Error> {
        self.validate_candidate(info)?;
        let (nar_size, nar_sha256) = SimpleRenderer::new(self.blob_service.clone(), self.directory_service.clone())
            .calculate_nar(&info.node)
            .await
            .map_err(|error| Error::Store(format!("casita-nar-mismatch: {}: {error}", info.store_path)))?;
        if (nar_size, nar_sha256) != (info.nar_size, info.nar_sha256) {
            return Err(Error::Store(format!("casita-nar-mismatch: signed NAR differs for {}", info.store_path)));
        }
        std::fs::create_dir(envelope).map_err(|error| Error::Store(format!("creating Casita envelope: {error}")))?;
        let content = envelope.join("content");
        export_castore_to_disk(
            &info.node,
            content.to_str().ok_or_else(|| Error::Store("Casita scratch path is not UTF-8".to_string()))?,
            &self.blob_service,
            &self.directory_service,
        )
        .await
        .map_err(|error| Error::Store(format!("exporting Casita envelope {}: {error}", info.store_path)))?;
        let encoded = serde_json::to_vec(info)
            .map_err(|error| Error::Store(format!("serializing Casita PathInfo {}: {error}", info.store_path)))?;
        if encoded.is_empty() || encoded.len() > MAX_PATHINFO_BYTES {
            return Err(Error::Store(format!(
                "casita-envelope-invalid: PathInfo for {} has size outside 1..={MAX_PATHINFO_BYTES}",
                info.store_path
            )));
        }
        std::fs::write(envelope.join("pathinfo.json"), encoded)
            .map_err(|error| Error::Store(format!("writing Casita PathInfo {}: {error}", info.store_path)))?;
        let roundtrip_node = ingest_path(
            self.blob_service.clone(),
            self.directory_service.clone(),
            &content,
            None::<&snix_castore::refscan::ReferenceScanner<&[u8]>>,
        )
        .await
        .map_err(|error| Error::Store(format!("casita-envelope-invalid: exported {}: {error}", info.store_path)))?;
        if roundtrip_node != info.node {
            return Err(Error::Store(format!("casita-envelope-invalid: exported {} changed node", info.store_path)));
        }
        let (measured_size, measured_hash) =
            SimpleRenderer::new(self.blob_service.clone(), self.directory_service.clone())
                .calculate_nar(&roundtrip_node)
                .await
                .map_err(|error| Error::Store(format!("casita-nar-mismatch: exported {}: {error}", info.store_path)))?;
        if (measured_size, measured_hash) != (info.nar_size, info.nar_sha256) {
            return Err(Error::Store(format!(
                "casita-nar-mismatch: exported {} differs from signature",
                info.store_path
            )));
        }
        Ok(())
    }

    fn registered_path<'a>(&self, logical_path: &'a str) -> Result<nix_compat::store_path::StorePathRef<'a>, Error> {
        nix_compat::store_path::StorePathRef::from_absolute_path_with_prefix(logical_path.as_bytes(), &self.store_dir)
            .map_err(|error| {
                Error::Store(format!("casita-root-missing: invalid retained path {logical_path}: {error}"))
            })
    }

    fn retained_roots(&self) -> Result<Vec<crate::roots::GcRootRecord>, Error> {
        let all = crate::roots::list_roots(&self.state_dir)?;
        if all.is_empty() {
            return Ok(all);
        }
        let facts = crate::retention::records_to_core(&all)?;
        let plan = crunch_gc_core::retention::plan_retention(
            &crate::retention::core_retention_policy(),
            crate::roots::current_unix_seconds()?,
            facts,
        )
        .map_err(|error| Error::Gc(format!("planning Casita retained roots: {error:?}")))?;
        let retained = plan
            .decisions
            .iter()
            .filter(|decision| decision.disposition.retains_path())
            .map(|decision| decision.path_id.as_str())
            .collect::<BTreeSet<_>>();
        Ok(all.into_iter().filter(|root| retained.contains(root.logical_path.as_str())).collect())
    }

    fn retained_digest(&self, digest: &[u8; 20]) -> Result<bool, Error> {
        for root in self.retained_roots()? {
            if self.registered_path(&root.logical_path)?.digest() == digest {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn require_registered_roots_present(&self, records: &[(RootName, ObjectKey, PathInfo)]) -> Result<(), Error> {
        let roots = self.retained_roots()?;
        if roots.is_empty() {
            return Ok(());
        }
        let present = records
            .iter()
            .map(|(_, _, info)| (*info.store_path.digest(), info.store_path.name().as_str()))
            .collect::<BTreeSet<_>>();
        for root in roots {
            let path = self.registered_path(&root.logical_path)?;
            if !present.contains(&(*path.digest(), *path.name())) {
                return Err(Error::Store(format!("casita-root-missing: {}", root.logical_path)));
            }
        }
        Ok(())
    }

    async fn publish_pathinfos(&self, path_infos: &[PathInfo], is_observed_update_allowed: bool) -> Result<(), Error> {
        self.require_recovered_gc()?;
        let max_root_changes = self.repository.limits().max_root_changes;
        if path_infos.len() > max_root_changes {
            return Err(Error::Store(format!(
                "casita-batch-limit: {} roots exceed {max_root_changes}",
                path_infos.len()
            )));
        }
        let mut names = std::collections::BTreeSet::new();
        for info in path_infos {
            let name = Self::root_name(&info.store_path)?;
            if !names.insert(name) {
                return Err(Error::Store(format!("casita-root-conflict: duplicate path {}", info.store_path)));
            }
            self.validate_candidate(info)?;
        }
        let scratch =
            tempfile::tempdir().map_err(|error| Error::Store(format!("creating Casita admission scratch: {error}")))?;
        let session = self
            .repository
            .mutation_session()
            .await
            .map_err(|error| Error::Store(format!("opening Casita mutation session: {error}")))?;
        let mut expectations = Vec::with_capacity(path_infos.len());
        let mut changes = Vec::with_capacity(path_infos.len());
        let mut newly_published = Vec::with_capacity(path_infos.len());
        for (index, info) in path_infos.iter().enumerate() {
            let name = Self::root_name(&info.store_path)?;
            let snapshot =
                self.repository.metadata().snapshot().await.map_err(|error| {
                    Error::Store(format!("reading Casita admission root {}: {error}", info.store_path))
                })?;
            let existing = snapshot
                .root(&name)
                .await
                .map_err(|error| Error::Store(format!("reading Casita admission root {}: {error}", info.store_path)))?;
            let observed = self
                .observed_roots
                .lock()
                .map_err(|_| Error::Store("Casita root observation mutex poisoned".to_string()))?
                .get(info.store_path.digest())
                .cloned();
            if let Some(target) = &existing {
                if let Ok(current) = self.read_root_pathinfo(&name, target, false).await
                    && current == *info
                {
                    expectations.push(RootExpectation {
                        name,
                        target: Some(target.clone()),
                    });
                    continue;
                }
                if !is_observed_update_allowed || observed.as_ref() != Some(target) {
                    return Err(Error::Store(format!("casita-root-conflict: {}", info.store_path)));
                }
            }
            let envelope = scratch.path().join(index.to_string());
            self.prepare_output_envelope(info, &envelope).await?;
            let target = session
                .import(UnrootedFilesystemImport::new(&envelope))
                .await
                .map_err(|error| Error::Store(format!("staging Casita {}: {error}", info.store_path)))?;
            expectations.push(RootExpectation {
                name: name.clone(),
                target: existing,
            });
            changes.push(RootChange::Set {
                name: name.clone(),
                target: target.clone(),
            });
            newly_published.push((name, target, info.store_path.to_string()));
        }
        if newly_published.is_empty() {
            for info in path_infos {
                let name = Self::root_name(&info.store_path)?;
                let snapshot = self
                    .repository
                    .metadata()
                    .snapshot()
                    .await
                    .map_err(|error| Error::Store(format!("rechecking Casita root: {error}")))?;
                let target = snapshot
                    .root(&name)
                    .await
                    .map_err(|error| Error::Store(format!("rechecking Casita root: {error}")))?
                    .ok_or_else(|| Error::Store(format!("casita-root-conflict: {}", info.store_path)))?;
                if self.read_root_pathinfo(&name, &target, false).await? != *info {
                    return Err(Error::Store(format!("casita-root-conflict: {}", info.store_path)));
                }
            }
            return Ok(());
        }
        #[cfg(test)]
        let gate = self.before_output_publish.lock().await.clone();
        #[cfg(test)]
        if let Some(gate) = gate {
            *gate.staged_targets.lock().await = newly_published.iter().map(|(_, target, _)| target.clone()).collect();
            gate.staged.wait().await;
            gate.resume.wait().await;
        }
        match session
            .publish_if_roots_match(Vec::new(), expectations, changes)
            .await
            .map_err(|error| Error::Store(format!("publishing Casita roots: {error}")))?
        {
            ConditionalPublishResult::RootMismatch { name, .. } => {
                Err(Error::Store(format!("casita-root-conflict: {}", name.as_str())))
            }
            ConditionalPublishResult::Committed(_) => {
                for (name, target, path) in newly_published {
                    let published = self.read_root_pathinfo(&name, &target, true).await?;
                    if published.store_path.to_string() != path {
                        return Err(Error::Store(format!("casita-root-conflict: published {} changed identity", path)));
                    }
                }
                Ok(())
            }
        }
    }
}

#[async_trait]
impl PathInfoService for CasitaStore {
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
        self.require_recovered_gc()?;
        let name = Self::root_name_for_digest(&digest)?;
        let snapshot = self.repository.metadata().snapshot().await?;
        let Some(target) = snapshot.root(&name).await? else {
            if self.retained_digest(&digest)? {
                return Err(Error::Store(format!("casita-root-missing: {}", name.as_str())).into());
            }
            return Ok(None);
        };
        Ok(Some(self.read_root_pathinfo(&name, &target, true).await?))
    }

    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
        self.publish_pathinfos(std::slice::from_ref(&path_info), true).await?;
        Ok(path_info)
    }

    async fn put_batch_atomic(
        &self,
        path_infos: Vec<PathInfo>,
    ) -> Result<Vec<PathInfo>, snix_store::pathinfoservice::Error> {
        self.publish_pathinfos(&path_infos, false).await?;
        Ok(path_infos)
    }

    fn list(&self) -> BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
        let store = self.clone();
        futures::stream::once(async move {
            if let Err(error) = store.require_recovered_gc() {
                return vec![Err(Box::new(error) as snix_store::pathinfoservice::Error)];
            }
            match store.verified_output_roots(&store.store_dir, true).await {
                Ok(records) => {
                    if let Err(error) = store.require_registered_roots_present(&records) {
                        return vec![Err(Box::new(error) as snix_store::pathinfoservice::Error)];
                    }
                    records
                        .into_iter()
                        .map(|(_, _, info)| Ok(info))
                        .collect::<Vec<Result<PathInfo, snix_store::pathinfoservice::Error>>>()
                }
                Err(error) => vec![Err(Box::new(error) as snix_store::pathinfoservice::Error)],
            }
        })
        .flat_map(futures::stream::iter)
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use nix_compat::narinfo::SigningKey;
    use nix_compat::narinfo::VerifyingKey;
    use nix_compat::narinfo::fingerprint_with_store_dir;
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::NixHash;
    use nix_compat::store_path::build_ca_path_with_store_dir;

    use super::*;
    use crate::StoreBackend;
    use crate::StoreConfig;
    use crate::StoreHandle;
    use crate::StoreMutationGuard;

    const STORE_DIR: &str = "/nix/store";

    async fn fixture(
        handle: &StoreHandle,
        source_dir: &Path,
        name: &str,
        contents: &[u8],
        signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    ) -> PathInfo {
        let source = source_dir.join(name);
        std::fs::write(&source, contents).unwrap();
        let node = ingest_path::<_, _, _, &[u8]>(handle.blob_service(), handle.directory_service(), &source, None)
            .await
            .unwrap();
        let (nar_size, nar_sha256) = SimpleRenderer::new(handle.blob_service(), handle.directory_service())
            .calculate_nar(&node)
            .await
            .unwrap();
        let ca = CAHash::Nar(NixHash::Sha256(nar_sha256));
        let store_path = build_ca_path_with_store_dir(name, &ca, Vec::<String>::new(), false, STORE_DIR).unwrap();
        let mut info = PathInfo {
            store_path,
            node,
            references: Vec::new(),
            nar_size,
            nar_sha256,
            signatures: Vec::new(),
            deriver: None,
            ca: Some(ca),
        };
        let fingerprint = fingerprint_with_store_dir(
            &info.store_path.as_ref(),
            &nar_sha256,
            nar_size,
            std::iter::empty::<&nix_compat::store_path::StorePathRef>(),
            STORE_DIR,
        );
        info.signatures.push(signing_key.sign(fingerprint.as_bytes()).to_owned());
        info
    }
    fn resign(info: &mut PathInfo, signing_key: &SigningKey<ed25519_dalek::SigningKey>) {
        let references = info.references.iter().map(StorePath::as_ref).collect::<Vec<_>>();
        let fingerprint = fingerprint_with_store_dir(
            &info.store_path.as_ref(),
            &info.nar_sha256,
            info.nar_size,
            references.iter(),
            STORE_DIR,
        );
        info.signatures.clear();
        info.signatures.push(signing_key.sign(fingerprint.as_bytes()).to_owned());
    }

    async fn outsider_replace(
        outsider: &LocalRepository,
        name: &RootName,
        old: ObjectKey,
        envelope: &Path,
    ) -> ObjectKey {
        let session = outsider.mutation_session().await.unwrap();
        let target = session.import(UnrootedFilesystemImport::new(envelope)).await.unwrap();
        let result = session
            .publish_if_roots_match(
                Vec::new(),
                vec![RootExpectation {
                    name: name.clone(),
                    target: Some(old),
                }],
                vec![RootChange::Set {
                    name: name.clone(),
                    target: target.clone(),
                }],
            )
            .await
            .unwrap();
        assert!(matches!(result, ConditionalPublishResult::Committed(_)));
        target
    }
    fn assert_rejected_in_new_process(root: &Path, path: &StorePath<String>, expected: &str) {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_CHILD_ROOT", root)
            .env("MANTLE_CASITA_CHILD_PATH", path.to_absolute_path_with_prefix(STORE_DIR))
            .env("MANTLE_CASITA_CHILD_EXPECT_ERROR", expected)
            .output()
            .unwrap();
        assert!(
            child.status.success() && String::from_utf8_lossy(&child.stdout).contains("casita-child-rejected"),
            "child status={} stderr={} stdout={}",
            child.status,
            String::from_utf8_lossy(&child.stderr),
            String::from_utf8_lossy(&child.stdout),
        );
    }
    fn assert_scratch_missing_in_new_process(root: &Path, path: &StorePath<String>) {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_CHILD_ROOT", root)
            .env("MANTLE_CASITA_CHILD_PATH", path.to_absolute_path_with_prefix(STORE_DIR))
            .env("MANTLE_CASITA_CHILD_EXPECT_MISS", "1")
            .output()
            .unwrap();
        assert!(
            child.status.success() && String::from_utf8_lossy(&child.stdout).contains("casita-child-miss"),
            "child status={} stderr={} stdout={}",
            child.status,
            String::from_utf8_lossy(&child.stderr),
            String::from_utf8_lossy(&child.stdout),
        );
    }

    #[tokio::test]
    async fn signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard() {
        if let Some(root) = std::env::var_os("MANTLE_CASITA_CHILD_ROOT") {
            let root = PathBuf::from(root);
            let output = root.join("output");
            let store_path = StorePath::from_absolute_path_with_prefix(
                std::env::var("MANTLE_CASITA_CHILD_PATH").unwrap().as_bytes(),
                STORE_DIR,
            )
            .unwrap();
            let physical = output.join(store_path.to_string());
            let state = root.join("state");
            let mut store = StoreHandle::open(StoreConfig::new(
                StoreBackend::Casita,
                state.clone(),
                output.clone(),
                STORE_DIR.to_string(),
            ))
            .await
            .unwrap();
            if std::env::var_os("MANTLE_CASITA_CHILD_MISSING").is_some() {
                assert!(physical.exists());
                let never_admitted: StorePath<String> =
                    StorePath::from_name_and_digest_fixed("never-admitted", [0xEF; 20]).unwrap();
                assert!(store.pathinfo_service().get(*never_admitted.digest()).await.unwrap().is_none());
                assert!(
                    store
                        .pathinfo_service()
                        .get(*store_path.digest())
                        .await
                        .unwrap_err()
                        .to_string()
                        .contains("casita-root-missing")
                );
                assert!(
                    crate::query::store_verify(store.pathinfo_service().as_ref(), None, &output)
                        .await
                        .unwrap_err()
                        .to_string()
                        .contains("casita-root-missing")
                );
                let guard = StoreMutationGuard::try_acquire(&state).unwrap();
                assert!(
                    store
                        .garbage_collect_under_guard(&guard, None)
                        .await
                        .unwrap_err()
                        .to_string()
                        .contains("casita-root-missing")
                );
                let other_path: StorePath<String> = StorePath::from_absolute_path_with_prefix(
                    std::env::var("MANTLE_CASITA_CHILD_OTHER_PATH").unwrap().as_bytes(),
                    STORE_DIR,
                )
                .unwrap();
                assert!(store.pathinfo_service().get(*other_path.digest()).await.unwrap().is_some());
                assert_eq!(std::fs::read(physical).unwrap(), b"casita durable content");
                println!("casita-child-missing-verified");
                return;
            }
            if let Ok(expected) = std::env::var("MANTLE_CASITA_CHILD_EXPECT_ERROR") {
                let error = store.pathinfo_service().get(*store_path.digest()).await.unwrap_err();
                assert!(error.to_string().contains(&expected), "{error}");
                if std::env::var_os("MANTLE_CASITA_CHILD_TRUST_READS").is_some() {
                    let repository = LocalRepository::local(state.join("casita")).await.unwrap();
                    let name = CasitaStore::root_name(&store_path).unwrap();
                    let target = repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap();
                    let error =
                        crate::query::store_verify(store.pathinfo_service().as_ref(), None, &output).await.unwrap_err();
                    assert!(error.to_string().contains(&expected), "{error}");
                    let guard = StoreMutationGuard::try_acquire(&state).unwrap();
                    let error = store.garbage_collect_under_guard(&guard, None).await.unwrap_err();
                    assert!(error.to_string().contains(&expected), "{error}");
                    assert_eq!(repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap(), target);
                }
                println!("casita-child-rejected");
                return;
            }
            if std::env::var_os("MANTLE_CASITA_CHILD_EXPECT_MISS").is_some() {
                assert!(store.pathinfo_service().get(*store_path.digest()).await.unwrap().is_none());
                println!("casita-child-miss");
                return;
            }
            assert!(!physical.exists());
            let info = store.pathinfo_service().get(*store_path.digest()).await.unwrap().unwrap();
            if std::env::var_os("MANTLE_CASITA_CHILD_NO_CASTORE").is_some() {
                let repository = LocalRepository::local(state.join("casita")).await.unwrap();
                let prefix = RootName::try_from("mantle/castore").unwrap();
                let snapshot = repository.metadata().snapshot().await.unwrap();
                let mut roots = snapshot.roots_under(&prefix);
                assert!(
                    roots.try_next().await.unwrap().is_none(),
                    "PathInfo-only action result unexpectedly published a mantle/castore root"
                );
            }
            assert_eq!(info.store_path, store_path);
            if let Ok(signer) = std::env::var("MANTLE_CASITA_CHILD_SIGNED") {
                assert!(info.signatures.iter().any(|signature| signature.name().as_str() == signer));
            }
            let repository = LocalRepository::local(state.join("casita")).await.unwrap();
            let name = CasitaStore::root_name(&info.store_path).unwrap();
            let target = repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap().unwrap();
            let envelope = tempfile::tempdir().unwrap();
            repository.checkout(&target, envelope.path().join("envelope")).await.unwrap();
            assert_eq!(
                std::fs::read(envelope.path().join("envelope/pathinfo.json")).unwrap(),
                serde_json::to_vec(&info).unwrap(),
            );
            assert!(store.castore_has_complete_content(&info.node).await.unwrap());
            let measured = SimpleRenderer::new(store.blob_service(), store.directory_service())
                .calculate_nar(&info.node)
                .await
                .unwrap();
            assert_eq!(measured, (info.nar_size, info.nar_sha256));
            if std::env::var_os("MANTLE_CASITA_CHILD_PATHINFO_ONLY").is_some() {
                println!("casita-child-verified");
                return;
            }
            let action_record: crunch_action_result_core::ActionResultRecord =
                serde_json::from_slice(&std::fs::read(root.join("action-record.json")).unwrap()).unwrap();
            let probe = store.into_builder_store_parts().action_results.probe_outputs(&action_record).await.unwrap();
            assert_eq!(probe.outputs.get("out"), Some(&info));
            assert_eq!(probe.reused_nar_bytes, info.nar_size);
            println!("casita-child-verified");
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::create_dir_all(&output).unwrap();
        let raw_signing_key = ed25519_dalek::SigningKey::from_bytes(&[17_u8; 32]);
        let verifying_key = VerifyingKey::new("casita-fixture-1".to_string(), raw_signing_key.verifying_key());
        let signing_key = SigningKey::new("casita-fixture-1".to_string(), raw_signing_key);
        std::fs::write(state.join("casita-trusted-public-keys"), format!("{verifying_key}\n")).unwrap();
        let config = || StoreConfig::new(StoreBackend::Casita, state.clone(), output.clone(), STORE_DIR.to_string());
        let mut store = StoreHandle::open(config()).await.unwrap();
        let other = StoreHandle::open(config()).await.unwrap();
        let one = fixture(&store, root.path(), "one", b"casita durable content", &signing_key).await;
        store.pathinfo_service().put(one.clone()).await.unwrap();
        let repository = store.casita_store.as_ref().unwrap().repository.clone();
        let one_name = CasitaStore::root_name(&one.store_path).unwrap();
        let admitted_target = repository.metadata().snapshot().await.unwrap().root(&one_name).await.unwrap().unwrap();
        let before_idempotent = repository.metadata().snapshot().await.unwrap().revision();
        assert_eq!(other.pathinfo_service().put(one.clone()).await.unwrap(), one);
        assert_eq!(
            repository.metadata().snapshot().await.unwrap().root(&one_name).await.unwrap(),
            Some(admitted_target)
        );
        assert_eq!(
            repository.metadata().snapshot().await.unwrap().revision(),
            before_idempotent,
            "an identical existing PathInfo should not publish a new Casita revision",
        );
        drop(repository);
        // PathInfo-backed action results do not require a separate castore root.
        let exported = store.export_cached_path_info(&one.store_path).await.unwrap().unwrap();
        assert_eq!(exported, one);
        let physical = output.join(one.store_path.to_string());
        assert_eq!(std::fs::read(&physical).unwrap(), b"casita durable content");
        std::fs::remove_file(&physical).unwrap();
        assert!(!physical.exists());

        let mut conflict = one.clone();
        conflict.signatures.push(signing_key.sign(b"additional-signature").to_owned());
        assert!(
            other
                .pathinfo_service()
                .put(conflict.clone())
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-root-conflict")
        );
        assert!(
            other
                .pathinfo_service()
                .put(conflict)
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-root-conflict"),
            "a rejected put must not internally authorize a later replacement"
        );
        let probe_handle = StoreHandle::open(config()).await.unwrap();
        let action_record = crunch_action_result_core::ActionResultRecord {
            schema: crunch_action_result_core::ACTION_RESULT_SCHEMA.to_string(),
            result_ref: String::new(),
            action_ref: String::new(),
            outputs: vec![crunch_action_result_core::ActionResultOutput {
                name: "out".to_string(),
                object_ref: String::new(),
                store_path: one.store_path.to_absolute_path_with_prefix(STORE_DIR),
                path_info_ref: String::new(),
            }],
            action_receipt_ref: String::new(),
            reference_scan_refs: Vec::new(),
            sandbox_policy_ref: String::new(),
            network_policy_ref: String::new(),
            producer_identity: String::new(),
            producer_policy_ref: String::new(),
            signature_refs: Vec::new(),
            publication_policy_ref: String::new(),
            non_claims: Vec::new(),
        };
        std::fs::write(root.path().join("action-record.json"), serde_json::to_vec(&action_record).unwrap()).unwrap();
        let probe = probe_handle.into_builder_store_parts().action_results.probe_outputs(&action_record).await.unwrap();
        assert_eq!(probe.outputs.get("out"), Some(&one));
        assert_eq!(probe.reused_nar_bytes, one.nar_size);

        assert_eq!(store.pathinfo_service().get(*one.store_path.digest()).await.unwrap(), Some(one.clone()));

        let two = fixture(&store, root.path(), "two", b"second atomic root", &signing_key).await;
        let three = fixture(&store, root.path(), "three", b"third atomic root", &signing_key).await;
        store.pathinfo_service().put_batch_atomic(vec![two.clone(), three.clone()]).await.unwrap();
        let four = fixture(&store, root.path(), "four", b"fourth mixed-batch root", &signing_key).await;
        store.pathinfo_service().put_batch_atomic(vec![one.clone(), four.clone()]).await.unwrap();
        let mut without_ca = fixture(&store, root.path(), "without-ca", b"signed non-CA output", &signing_key).await;
        without_ca.ca = None;
        without_ca.store_path = StorePath::from_name_and_digest_fixed("without-ca", [0x31; 20]).unwrap();
        resign(&mut without_ca, &signing_key);
        store.pathinfo_service().put(without_ca.clone()).await.unwrap();
        let staged = fixture(&store, root.path(), "staged", b"scratch copy stays untrusted", &signing_key).await;
        let mut invalid = fixture(&store, root.path(), "invalid", b"bad signed NAR facts", &signing_key).await;
        invalid.nar_size += 1;
        resign(&mut invalid, &signing_key);
        assert!(
            store
                .pathinfo_service()
                .put_batch_atomic(vec![staged.clone(), invalid.clone()])
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-nar-mismatch: signed NAR differs for")
        );
        assert!(store.castore_has_complete_content(&staged.node).await.unwrap());
        assert!(store.pathinfo_service().get(*staged.store_path.digest()).await.unwrap().is_none());
        assert!(store.pathinfo_service().get(*invalid.store_path.digest()).await.unwrap().is_none());
        assert!(!output.join(staged.store_path.to_string()).exists());
        assert_scratch_missing_in_new_process(root.path(), &staged.store_path);
        let reclaimed = store.casita_store.as_ref().unwrap().repository.try_collect().await.unwrap();
        assert!(reclaimed.removed.logical_objects > 0, "failed batch left unrooted staged envelopes");
        assert!(store.pathinfo_service().get(*staged.store_path.digest()).await.unwrap().is_none());
        store.pathinfo_service().put(staged.clone()).await.unwrap();
        assert_eq!(store.pathinfo_service().get(*staged.store_path.digest()).await.unwrap(), Some(staged));
        drop(other);
        drop(store);
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_CHILD_ROOT", root.path())
            .env("MANTLE_CASITA_CHILD_PATH", one.store_path.to_absolute_path_with_prefix(STORE_DIR))
            .env("MANTLE_CASITA_CHILD_NO_CASTORE", "1")
            .output()
            .unwrap();
        assert!(
            child.status.success() && String::from_utf8_lossy(&child.stdout).contains("casita-child-verified"),
            "child status={} stderr={} stdout={}",
            child.status,
            String::from_utf8_lossy(&child.stderr),
            String::from_utf8_lossy(&child.stdout),
        );
        let no_ca_child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_CHILD_ROOT", root.path())
            .env("MANTLE_CASITA_CHILD_PATH", without_ca.store_path.to_absolute_path_with_prefix(STORE_DIR))
            .env("MANTLE_CASITA_CHILD_PATHINFO_ONLY", "1")
            .output()
            .unwrap();
        assert!(
            no_ca_child.status.success()
                && String::from_utf8_lossy(&no_ca_child.stdout).contains("casita-child-verified"),
            "non-CA child status={} stderr={} stdout={}",
            no_ca_child.status,
            String::from_utf8_lossy(&no_ca_child.stderr),
            String::from_utf8_lossy(&no_ca_child.stdout),
        );
        assert!(!physical.exists());

        let mut reopened = StoreHandle::open(config()).await.unwrap();
        assert_eq!(reopened.pathinfo_service().get(*one.store_path.digest()).await.unwrap(), Some(one.clone()));
        reopened.admit_castore_payload_root(&one.node).await.unwrap();
        reopened.rehydrate_castore_payload_root(&one.node).await.unwrap();
        assert_eq!(reopened.pathinfo_service().get(*two.store_path.digest()).await.unwrap(), Some(two.clone()));
        assert_eq!(reopened.pathinfo_service().get(*three.store_path.digest()).await.unwrap(), Some(three.clone()));
        assert_eq!(reopened.pathinfo_service().get(*four.store_path.digest()).await.unwrap(), Some(four.clone()));
        assert_eq!(reopened.export_cached_path_info(&one.store_path).await.unwrap(), Some(one.clone()));
        assert_eq!(std::fs::read(&physical).unwrap(), b"casita durable content");

        let wrong_state = root.path().join("wrong-state");
        let wrong_guard = StoreMutationGuard::try_acquire(&wrong_state).unwrap();
        assert!(
            reopened
                .garbage_collect_under_guard(&wrong_guard, None)
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-plan-stale")
        );
        assert_eq!(
            reopened.pathinfo_service().get(*without_ca.store_path.digest()).await.unwrap(),
            Some(without_ca.clone())
        );
        assert!(reopened.garbage_collect(None).await.unwrap_err().to_string().contains("casita-gc-guard-required"));
        assert!(
            reopened
                .garbage_collect(Some("one"))
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-gc-guard-required")
        );
        assert!(
            reopened
                .garbage_collect_with_castore_roots_under_guard(&wrong_guard, None, std::slice::from_ref(&one.node),)
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-plan-stale")
        );
        let right_guard = StoreMutationGuard::try_acquire(&state).unwrap();
        let _plan = reopened.garbage_collect_under_guard(&right_guard, None).await.unwrap();
        let extra_raw_key = ed25519_dalek::SigningKey::from_bytes(&[18_u8; 32]);
        let extra_verify = VerifyingKey::new("casita-fixture-2".to_string(), extra_raw_key.verifying_key());
        let extra_sign = SigningKey::new("casita-fixture-2".to_string(), extra_raw_key);
        std::fs::write(state.join("casita-trusted-public-keys"), format!("{verifying_key}\n{extra_verify}\n")).unwrap();
        let live =
            fixture(&reopened, root.path(), "live-policy", b"late signer adoption and revocation", &extra_sign).await;
        reopened.pathinfo_service().put(live.clone()).await.unwrap();
        let live_name = CasitaStore::root_name(&live.store_path).unwrap();
        let live_repository = LocalRepository::local(state.join("casita")).await.unwrap();
        let live_target = live_repository.metadata().snapshot().await.unwrap().root(&live_name).await.unwrap().unwrap();
        std::fs::write(state.join("casita-trusted-public-keys"), format!("{verifying_key}\n")).unwrap();
        assert!(
            reopened
                .pathinfo_service()
                .get(*live.store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-signer-untrusted")
        );
        assert!(
            crate::query::store_verify(reopened.pathinfo_service().as_ref(), None, &output)
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-signer-untrusted")
        );
        assert!(
            reopened
                .garbage_collect_under_guard(&right_guard, None)
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-signer-untrusted")
        );
        assert_eq!(
            live_repository.metadata().snapshot().await.unwrap().root(&live_name).await.unwrap(),
            Some(live_target.clone()),
        );
        drop(right_guard);
        let policy = state.join(TRUST_FILE_NAME);
        let key_removed_policy = std::fs::read(&policy).unwrap();
        for remove_policy_file in [false, true] {
            if remove_policy_file {
                std::fs::remove_file(&policy).unwrap();
                let error = reopened.pathinfo_service().get(*live.store_path.digest()).await.unwrap_err();
                assert!(error.to_string().contains("casita-signer-untrusted"), "{error}");
            }
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("casita::tests::signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard")
                .arg("--exact")
                .arg("--nocapture")
                .env("MANTLE_CASITA_CHILD_ROOT", root.path())
                .env("MANTLE_CASITA_CHILD_PATH", live.store_path.to_absolute_path_with_prefix(STORE_DIR))
                .env("MANTLE_CASITA_CHILD_EXPECT_ERROR", "casita-signer-untrusted")
                .env("MANTLE_CASITA_CHILD_TRUST_READS", "1")
                .output()
                .unwrap();
            assert!(
                child.status.success() && String::from_utf8_lossy(&child.stdout).contains("casita-child-rejected"),
                "fresh-process trust removal failed: stderr={}, stdout={}",
                String::from_utf8_lossy(&child.stderr),
                String::from_utf8_lossy(&child.stdout),
            );
            assert_eq!(
                live_repository.metadata().snapshot().await.unwrap().root(&live_name).await.unwrap(),
                Some(live_target.clone()),
            );
            if remove_policy_file {
                assert!(!policy.exists());
            } else {
                assert_eq!(std::fs::read(&policy).unwrap(), key_removed_policy);
            }
        }
        let right_guard = StoreMutationGuard::try_acquire(&state).unwrap();
        std::fs::write(state.join("casita-trusted-public-keys"), format!("{verifying_key}\n{extra_verify}\n")).unwrap();
        assert_eq!(reopened.pathinfo_service().get(*live.store_path.digest()).await.unwrap(), Some(live));
        let root_name = CasitaStore::root_name(&one.store_path).unwrap();
        let sign_repository = LocalRepository::local(state.join("casita")).await.unwrap();
        let old_target = sign_repository.metadata().snapshot().await.unwrap().root(&root_name).await.unwrap().unwrap();
        let old_envelope = tempfile::tempdir().unwrap();
        sign_repository.checkout(&old_target, old_envelope.path().join("envelope")).await.unwrap();
        let signed =
            crate::query::store_sign(reopened.pathinfo_service().as_ref(), &extra_sign, Some("one"), false, STORE_DIR)
                .await
                .unwrap();
        assert!(matches!(signed.as_slice(), [result]
            if result.store_path == one.store_path.to_string() && result.appended));
        let one = reopened.pathinfo_service().get(*one.store_path.digest()).await.unwrap().unwrap();
        assert!(one.signatures.iter().any(|signature| signature.name().as_str() == "casita-fixture-2"));
        let signed_target =
            sign_repository.metadata().snapshot().await.unwrap().root(&root_name).await.unwrap().unwrap();
        assert_ne!(signed_target, old_target);
        let signed_envelope = tempfile::tempdir().unwrap();
        sign_repository.checkout(&signed_target, signed_envelope.path().join("envelope")).await.unwrap();
        assert_eq!(
            std::fs::read(old_envelope.path().join("envelope/content")).unwrap(),
            std::fs::read(signed_envelope.path().join("envelope/content")).unwrap(),
        );
        assert_ne!(
            std::fs::read(old_envelope.path().join("envelope/pathinfo.json")).unwrap(),
            std::fs::read(signed_envelope.path().join("envelope/pathinfo.json")).unwrap(),
        );
        let signed_snapshot = sign_repository.metadata().snapshot().await.unwrap();
        let mut published_roots = signed_snapshot.roots();
        while let Some(root) = published_roots.try_next().await.unwrap() {
            assert_ne!(root.target(), &old_target, "replaced envelope remained rooted under {}", root.name().as_str());
        }
        std::fs::remove_file(&physical).unwrap();
        let signed_child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_CHILD_ROOT", root.path())
            .env("MANTLE_CASITA_CHILD_PATH", one.store_path.to_absolute_path_with_prefix(STORE_DIR))
            .env("MANTLE_CASITA_CHILD_SIGNED", "casita-fixture-2")
            .output()
            .unwrap();
        assert!(
            signed_child.status.success()
                && String::from_utf8_lossy(&signed_child.stdout).contains("casita-child-verified"),
            "signed child status={} stderr={} stdout={}",
            signed_child.status,
            String::from_utf8_lossy(&signed_child.stderr),
            String::from_utf8_lossy(&signed_child.stdout),
        );
        assert_eq!(reopened.export_cached_path_info(&one.store_path).await.unwrap(), Some(one.clone()));
        let logical_path = one.store_path.to_absolute_path_with_prefix(STORE_DIR);
        reopened.register_retained_root(&one.store_path, crate::GcRootSource::Pin).await.unwrap();
        let original_registry = std::fs::read(crate::roots::roots_path(&state)).unwrap();

        let fence = state.join("casita-gc-fence.json");
        std::fs::write(&fence, b"{}").unwrap();
        assert!(
            reopened
                .pathinfo_service()
                .get(*one.store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .pathinfo_service()
                .put(one.clone())
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .pathinfo_service()
                .list()
                .next()
                .await
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .export_cached_path_info(&one.store_path)
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .rehydrate_castore_payload_root(&one.node)
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .register_retained_root(&one.store_path, crate::GcRootSource::Build)
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .unpin_retained_root(&logical_path)
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .pin_retained_root(&logical_path)
                .await
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert!(
            reopened
                .store_admin()
                .migrate_legacy_root_registry()
                .unwrap_err()
                .to_string()
                .contains("gc-recovery-required")
        );
        assert_eq!(std::fs::read(crate::roots::roots_path(&state)).unwrap(), original_registry);
        std::fs::remove_file(&fence).unwrap();
        let registered = reopened.register_retained_root(&one.store_path, crate::GcRootSource::Pin).await.unwrap();
        assert_eq!(registered.logical_path, one.store_path.to_absolute_path_with_prefix(STORE_DIR));

        // A second real repository client can stage a different object, but
        // verified readers must reject both changed bytes and changed metadata.
        let outsider = LocalRepository::local(state.join("casita")).await.unwrap();
        for path_info in [&one, &two, &three, &four] {
            let root_name = CasitaStore::root_name(&path_info.store_path).unwrap();
            assert_eq!(outsider.root_retention(&root_name).await.unwrap(), Some(casita::RootRetention::Permanent),);
        }
        let castore_name = CasitaStore::castore_root_name(&one.node).unwrap();
        assert_eq!(outsider.root_retention(&castore_name).await.unwrap(), Some(casita::RootRetention::Permanent),);
        let usage = casita::experimental::DiskUsage::new(100, 0).unwrap();
        let pressure = casita::experimental::DiskPressurePolicy::new(1).unwrap();
        assert!(matches!(
            pressure.collect_if_needed(&outsider, usage).await.unwrap(),
            casita::experimental::DiskPressureOutcome::Collected { .. }
        ));
        for path_info in [&one, &two, &three, &four] {
            let root_name = CasitaStore::root_name(&path_info.store_path).unwrap();
            assert_eq!(outsider.root_retention(&root_name).await.unwrap(), Some(casita::RootRetention::Permanent),);
            assert!(outsider.metadata().snapshot().await.unwrap().root(&root_name).await.unwrap().is_some());
        }
        assert_eq!(outsider.root_retention(&castore_name).await.unwrap(), Some(casita::RootRetention::Permanent));
        assert!(outsider.metadata().snapshot().await.unwrap().root(&castore_name).await.unwrap().is_some());
        let name = CasitaStore::root_name(&one.store_path).unwrap();
        let snapshot = outsider.metadata().snapshot().await.unwrap();
        let original = snapshot.root(&name).await.unwrap().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let changed_content = scratch.path().join("changed-content");
        outsider.checkout(&original, &changed_content).await.unwrap();
        std::fs::write(changed_content.join("content"), b"casita adversarial content").unwrap();
        let changed_metadata = scratch.path().join("changed-metadata");
        outsider.checkout(&original, &changed_metadata).await.unwrap();
        let mut forged = one.clone();
        forged.nar_size += 1;
        std::fs::write(changed_metadata.join("pathinfo.json"), serde_json::to_vec(&forged).unwrap()).unwrap();
        let noncanonical = scratch.path().join("noncanonical");
        outsider.checkout(&original, &noncanonical).await.unwrap();
        let metadata_path = noncanonical.join("pathinfo.json");
        let mut json = std::fs::read(&metadata_path).unwrap();
        json.push(b'\n');
        std::fs::write(&metadata_path, json).unwrap();
        let oversized = scratch.path().join("oversized");
        outsider.checkout(&original, &oversized).await.unwrap();
        std::fs::write(oversized.join("pathinfo.json"), vec![b'x'; MAX_PATHINFO_BYTES + 1]).unwrap();
        let signed_mismatch = scratch.path().join("signed-mismatch");
        outsider.checkout(&original, &signed_mismatch).await.unwrap();
        let mut wrong_facts = one.clone();
        wrong_facts.nar_size += 1;
        resign(&mut wrong_facts, &signing_key);
        std::fs::write(signed_mismatch.join("pathinfo.json"), serde_json::to_vec(&wrong_facts).unwrap()).unwrap();
        let wrong_path = scratch.path().join("wrong-path");
        outsider.checkout(&original, &wrong_path).await.unwrap();
        std::fs::write(wrong_path.join("pathinfo.json"), serde_json::to_vec(&two).unwrap()).unwrap();
        let replaced_content = outsider_replace(&outsider, &name, original.clone(), &changed_content).await;
        assert_rejected_in_new_process(root.path(), &one.store_path, "casita-envelope-invalid");
        assert!(
            reopened
                .pathinfo_service()
                .get(*one.store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-envelope-invalid")
        );

        let replaced_metadata = outsider_replace(&outsider, &name, replaced_content, &changed_metadata).await;
        assert_rejected_in_new_process(root.path(), &one.store_path, "casita-signer-untrusted");
        assert!(
            reopened
                .pathinfo_service()
                .get(*one.store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-signer-untrusted")
        );

        let replaced_noncanonical = outsider_replace(&outsider, &name, replaced_metadata, &noncanonical).await;
        assert_rejected_in_new_process(
            root.path(),
            &one.store_path,
            "casita-envelope-invalid: PathInfo is not canonical JSON",
        );
        assert!(
            reopened
                .pathinfo_service()
                .get(*one.store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-envelope-invalid: PathInfo is not canonical JSON")
        );

        let replaced_oversized = outsider_replace(&outsider, &name, replaced_noncanonical, &oversized).await;
        assert_rejected_in_new_process(root.path(), &one.store_path, "casita-envelope-invalid: PathInfo size");
        assert!(
            reopened
                .pathinfo_service()
                .get(*one.store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-envelope-invalid: PathInfo size")
        );

        let replaced_signed_mismatch = outsider_replace(&outsider, &name, replaced_oversized, &signed_mismatch).await;
        assert_rejected_in_new_process(root.path(), &one.store_path, "casita-nar-mismatch");
        assert_eq!(
            outsider.metadata().snapshot().await.unwrap().root(&name).await.unwrap(),
            Some(replaced_signed_mismatch.clone()),
        );
        let replaced_wrong_path = outsider_replace(&outsider, &name, replaced_signed_mismatch, &wrong_path).await;
        assert_rejected_in_new_process(root.path(), &one.store_path, "casita-envelope-invalid: root");

        let session = outsider.mutation_session().await.unwrap();
        let result = session
            .publish_if_roots_match(
                Vec::new(),
                vec![RootExpectation {
                    name: name.clone(),
                    target: Some(replaced_wrong_path),
                }],
                vec![RootChange::Remove { name }],
            )
            .await
            .unwrap();
        assert!(matches!(result, ConditionalPublishResult::Committed(_)));
        assert!(
            reopened
                .pathinfo_service()
                .get(*one.store_path.digest())
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-root-missing")
        );
        assert!(
            reopened
                .garbage_collect_under_guard(&right_guard, None)
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-root-missing")
        );
        drop(right_guard);
        assert_eq!(std::fs::read(&physical).unwrap(), b"casita durable content");
        assert_eq!(reopened.pathinfo_service().get(*two.store_path.digest()).await.unwrap(), Some(two.clone()));
        let missing_child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::signed_roots_survive_export_removal_and_reopen_with_conflicts_and_guard")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_CHILD_ROOT", root.path())
            .env("MANTLE_CASITA_CHILD_PATH", one.store_path.to_absolute_path_with_prefix(STORE_DIR))
            .env("MANTLE_CASITA_CHILD_OTHER_PATH", two.store_path.to_absolute_path_with_prefix(STORE_DIR))
            .env("MANTLE_CASITA_CHILD_MISSING", "1")
            .output()
            .unwrap();
        assert!(
            missing_child.status.success()
                && String::from_utf8_lossy(&missing_child.stdout).contains("casita-child-missing-verified"),
            "missing child status={} stderr={} stdout={}",
            missing_child.status,
            String::from_utf8_lossy(&missing_child.stderr),
            String::from_utf8_lossy(&missing_child.stdout),
        );
    }

    #[tokio::test]
    async fn gc_inventory_reads_more_roots_than_one_atomic_mutation() {
        const TOTAL_ROOTS: usize = 1_025;
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::create_dir_all(&output).unwrap();
        let raw_signing_key = ed25519_dalek::SigningKey::from_bytes(&[17_u8; 32]);
        let verifying_key = VerifyingKey::new("casita-inventory".to_string(), raw_signing_key.verifying_key());
        let signing_key = SigningKey::new("casita-inventory".to_string(), raw_signing_key);
        std::fs::write(state.join(TRUST_FILE_NAME), format!("{verifying_key}\n")).unwrap();
        let store = StoreHandle::open(StoreConfig::new(StoreBackend::Casita, state, output, STORE_DIR.to_string()))
            .await
            .unwrap();
        let template = fixture(&store, root.path(), "inventory-template", b"shared content", &signing_key).await;
        let mut records = Vec::with_capacity(TOTAL_ROOTS);
        for index in 0..TOTAL_ROOTS {
            let mut info = template.clone();
            info.store_path = build_ca_path_with_store_dir(
                &format!("inventory-{index:04}"),
                info.ca.as_ref().unwrap(),
                Vec::<String>::new(),
                false,
                STORE_DIR,
            )
            .unwrap();
            resign(&mut info, &signing_key);
            records.push(info);
        }
        let last_path = records.last().unwrap().store_path.clone();
        let max_root_changes = store.casita_store.as_ref().unwrap().repository.limits().max_root_changes;
        assert!(max_root_changes < TOTAL_ROOTS);
        let second_batch = records.split_off(max_root_changes);
        store.pathinfo_service().put_batch_atomic(records).await.unwrap();
        store.pathinfo_service().put_batch_atomic(second_batch).await.unwrap();
        let verified = store.casita_store.as_ref().unwrap().verified_gc_roots(STORE_DIR).await.unwrap();
        assert_eq!(verified.len(), TOTAL_ROOTS);
        assert!(verified.iter().any(|(_, _, info)| info.store_path == last_path));
    }

    #[tokio::test]
    async fn staged_output_batch_rejects_competing_root_and_reclaims_abandoned_envelopes() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir(&state).unwrap();
        std::fs::create_dir(&output).unwrap();
        let raw = ed25519_dalek::SigningKey::from_bytes(&[21_u8; 32]);
        let signing_key = SigningKey::new("casita-stage-race".to_string(), raw.clone());
        let verifying_key = VerifyingKey::new("casita-stage-race".to_string(), raw.verifying_key());
        std::fs::write(state.join(TRUST_FILE_NAME), format!("{verifying_key}\n")).unwrap();
        let config = || StoreConfig::new(StoreBackend::Casita, state.clone(), output.clone(), STORE_DIR.to_string());
        let selected = StoreHandle::open(config()).await.unwrap();
        let competitor = StoreHandle::open(config()).await.unwrap();
        let repository = LocalRepository::local(state.join("casita")).await.unwrap();
        let first = fixture(&selected, root.path(), "stage-race-first", b"candidate first", &signing_key).await;
        let second = fixture(&selected, root.path(), "stage-race-second", b"candidate second", &signing_key).await;
        let mut rival = fixture(&competitor, root.path(), "stage-race-rival", b"competing content", &signing_key).await;
        rival.store_path = first.store_path.clone();
        rival.ca = None;
        resign(&mut rival, &signing_key);
        let first_name = CasitaStore::root_name(&first.store_path).unwrap();
        let second_name = CasitaStore::root_name(&second.store_path).unwrap();
        // Explicitly drive the envelope/session/CAS seam to interleave a real second client without a test
        // hook.
        let scratch = tempfile::tempdir_in(root.path()).unwrap();
        let session = repository.mutation_session().await.unwrap();
        let mut expectations = Vec::with_capacity(2);
        let mut changes = Vec::with_capacity(2);
        let mut staged_targets = Vec::with_capacity(2);
        for (index, (name, info)) in
            [(first_name.clone(), &first), (second_name.clone(), &second)].into_iter().enumerate()
        {
            let envelope = scratch.path().join(index.to_string());
            selected.casita_store.as_ref().unwrap().prepare_output_envelope(info, &envelope).await.unwrap();
            let target = session.import(UnrootedFilesystemImport::new(&envelope)).await.unwrap();
            expectations.push(RootExpectation {
                name: name.clone(),
                target: None,
            });
            changes.push(RootChange::Set {
                name,
                target: target.clone(),
            });
            staged_targets.push(target);
        }
        let staged_snapshot = repository.metadata().snapshot().await.unwrap();
        assert!(staged_snapshot.root(&first_name).await.unwrap().is_none());
        assert!(staged_snapshot.root(&second_name).await.unwrap().is_none());
        for target in &staged_targets {
            assert!(staged_snapshot.object(target).await.unwrap().is_some(), "staged envelope not durable");
        }
        drop(staged_snapshot);

        competitor.pathinfo_service().put(rival.clone()).await.unwrap();
        assert_eq!(competitor.pathinfo_service().get(*rival.store_path.digest()).await.unwrap(), Some(rival));
        let rival_target = repository.metadata().snapshot().await.unwrap().root(&first_name).await.unwrap().unwrap();
        assert_ne!(rival_target, staged_targets[0]);
        let error = selected.pathinfo_service().put(first.clone()).await.unwrap_err();
        assert!(error.to_string().contains("casita-root-conflict"), "{error}");
        let after_mantle = repository.metadata().snapshot().await.unwrap();
        assert_eq!(after_mantle.root(&first_name).await.unwrap(), Some(rival_target.clone()));
        assert!(after_mantle.root(&second_name).await.unwrap().is_none());
        drop(after_mantle);
        let raced = session.publish_if_roots_match(Vec::new(), expectations, changes).await.unwrap();
        assert!(matches!(raced, ConditionalPublishResult::RootMismatch { name, .. } if name == first_name));
        let snapshot = repository.metadata().snapshot().await.unwrap();
        assert_eq!(snapshot.root(&first_name).await.unwrap(), Some(rival_target.clone()));
        assert!(snapshot.root(&second_name).await.unwrap().is_none(), "conflicting batch partially published");
        drop(snapshot);
        drop(session);
        drop(scratch);

        let reclaimed = repository.try_collect().await.unwrap();
        assert!(reclaimed.removed.logical_objects > 0, "abandoned staged envelopes were not reclaimed");
        let snapshot = repository.metadata().snapshot().await.unwrap();
        assert_eq!(snapshot.root(&first_name).await.unwrap(), Some(rival_target.clone()));
        assert!(snapshot.root(&second_name).await.unwrap().is_none());
        for target in &staged_targets {
            assert!(snapshot.object(target).await.unwrap().is_none(), "abandoned envelope remained reachable");
        }
        drop(snapshot);

        assert!(repository.remove_root_if_matches(&first_name, &rival_target).await.unwrap().is_some());
        assert!(repository.metadata().snapshot().await.unwrap().root(&first_name).await.unwrap().is_none());
        selected.pathinfo_service().put_batch_atomic(vec![first.clone(), second.clone()]).await.unwrap();
        drop(selected);
        drop(competitor);
        drop(repository);
        let reopened = StoreHandle::open(config()).await.unwrap();
        assert_eq!(reopened.pathinfo_service().get(*first.store_path.digest()).await.unwrap(), Some(first));
        assert_eq!(reopened.pathinfo_service().get(*second.store_path.digest()).await.unwrap(), Some(second));
    }

    #[tokio::test]
    async fn pathinfo_batch_rejects_second_client_root_published_after_staging() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir(&state).unwrap();
        std::fs::create_dir(&output).unwrap();
        let raw = ed25519_dalek::SigningKey::from_bytes(&[27_u8; 32]);
        let signing_key = SigningKey::new("casita-admission-race".to_string(), raw.clone());
        let verifying_key = VerifyingKey::new("casita-admission-race".to_string(), raw.verifying_key());
        std::fs::write(state.join(TRUST_FILE_NAME), format!("{verifying_key}\n")).unwrap();
        let config = || StoreConfig::new(StoreBackend::Casita, state.clone(), output.clone(), STORE_DIR.to_string());
        let selected = StoreHandle::open(config()).await.unwrap();
        let competitor = StoreHandle::open(config()).await.unwrap();
        let first = fixture(&selected, root.path(), "admission-first", b"original candidate", &signing_key).await;
        let second = fixture(&selected, root.path(), "admission-second", b"other candidate", &signing_key).await;
        let mut rival = fixture(&competitor, root.path(), "admission-rival", b"competing content", &signing_key).await;
        rival.store_path = first.store_path.clone();
        rival.ca = None;
        resign(&mut rival, &signing_key);
        let first_name = CasitaStore::root_name(&first.store_path).unwrap();
        let second_name = CasitaStore::root_name(&second.store_path).unwrap();
        let repository = LocalRepository::local(state.join("casita")).await.unwrap();
        let gate = Arc::new(AdmissionRaceGate {
            staged: tokio::sync::Barrier::new(2),
            resume: tokio::sync::Barrier::new(2),
            staged_targets: tokio::sync::Mutex::new(Vec::new()),
        });
        selected.casita_store.as_ref().unwrap().before_output_publish.lock().await.replace(gate.clone());
        let selected_pathinfos = selected.pathinfo_service();
        let staged = tokio::spawn(async move { selected_pathinfos.put_batch_atomic(vec![first, second]).await });
        tokio::time::timeout(std::time::Duration::from_secs(30), gate.staged.wait())
            .await
            .expect("the actual PathInfo batch must stage both envelopes before its CAS commit");
        let staged_targets = gate.staged_targets.lock().await.clone();
        assert_eq!(staged_targets.len(), 2);
        let snapshot = repository.metadata().snapshot().await.unwrap();
        assert!(snapshot.root(&first_name).await.unwrap().is_none());
        assert!(snapshot.root(&second_name).await.unwrap().is_none());
        for target in &staged_targets {
            assert!(snapshot.object(target).await.unwrap().is_some(), "staged envelope was not durable");
        }
        drop(snapshot);
        competitor.pathinfo_service().put(rival.clone()).await.unwrap();
        let rival_target = repository.metadata().snapshot().await.unwrap().root(&first_name).await.unwrap().unwrap();
        gate.resume.wait().await;
        let error = staged.await.unwrap().unwrap_err();
        drop(selected);
        assert!(error.to_string().contains("casita-root-conflict"), "{error}");
        let snapshot = repository.metadata().snapshot().await.unwrap();
        assert_eq!(snapshot.root(&first_name).await.unwrap(), Some(rival_target.clone()));
        assert!(snapshot.root(&second_name).await.unwrap().is_none(), "conflicting batch partially published");
        drop(snapshot);
        assert_eq!(competitor.pathinfo_service().get(*rival.store_path.digest()).await.unwrap(), Some(rival));
        casita::experimental::flush_repository_leases().await.unwrap();
        repository.try_collect().await.unwrap();
        let snapshot = repository.metadata().snapshot().await.unwrap();
        assert_eq!(snapshot.root(&first_name).await.unwrap(), Some(rival_target));
        assert!(snapshot.root(&second_name).await.unwrap().is_none());
        for target in &staged_targets {
            assert!(snapshot.object(target).await.unwrap().is_none(), "failed batch envelope survived collection");
        }
    }

    #[tokio::test]
    async fn stopped_staging_process_leaves_no_root_and_collection_allows_retry() {
        const NAME: &str = "abandoned-stage";
        const CONTENT: &[u8] = b"abandoned output envelope";
        if let Some(root) = std::env::var_os("MANTLE_CASITA_STAGE_CHILD_ROOT") {
            let root = PathBuf::from(root);
            let state = root.join("state");
            let store = StoreHandle::open(StoreConfig::new(
                StoreBackend::Casita,
                state,
                root.join("output"),
                STORE_DIR.to_string(),
            ))
            .await
            .unwrap();
            let raw = ed25519_dalek::SigningKey::from_bytes(&[22_u8; 32]);
            let signing_key = SigningKey::new("casita-abandoned-stage".to_string(), raw);
            let info = fixture(&store, &root, NAME, CONTENT, &signing_key).await;
            let scratch = root.join("child-scratch");
            std::fs::create_dir(&scratch).unwrap();
            let envelope = scratch.join("envelope");
            let casita = store.casita_store.as_ref().unwrap();
            casita.prepare_output_envelope(&info, &envelope).await.unwrap();
            let session = casita.repository.mutation_session().await.unwrap();
            let target = session.import(UnrootedFilesystemImport::new(&envelope)).await.unwrap();
            let name = CasitaStore::root_name(&info.store_path).unwrap();
            let snapshot = casita.repository.metadata().snapshot().await.unwrap();
            assert!(snapshot.root(&name).await.unwrap().is_none());
            assert!(snapshot.object(&target).await.unwrap().is_some());
            drop(snapshot);
            assert!(store.pathinfo_service().get(*info.store_path.digest()).await.unwrap().is_none());
            std::fs::write(root.join("staged-info.json"), serde_json::to_vec(&info).unwrap()).unwrap();
            std::fs::write(root.join("staged-target"), target.to_string()).unwrap();
            drop(session);
            drop(store);
            // Casita releases durable staging pins asynchronously on drop; drain them before stopping this
            // runtime.
            casita::experimental::flush_repository_leases().await.unwrap();
            std::fs::remove_dir_all(scratch).unwrap();
            println!("casita-stage-child-stopped");
            return;
        }

        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir(&state).unwrap();
        std::fs::create_dir(&output).unwrap();
        let raw = ed25519_dalek::SigningKey::from_bytes(&[22_u8; 32]);
        let verifying_key = VerifyingKey::new("casita-abandoned-stage".to_string(), raw.verifying_key());
        let signing_key = SigningKey::new("casita-abandoned-stage".to_string(), raw);
        let policy = state.join(TRUST_FILE_NAME);
        let policy_bytes = format!("{verifying_key}\n");
        std::fs::write(&policy, &policy_bytes).unwrap();
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::stopped_staging_process_leaves_no_root_and_collection_allows_retry")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_STAGE_CHILD_ROOT", root.path())
            .output()
            .unwrap();
        assert!(
            child.status.success() && String::from_utf8_lossy(&child.stdout).contains("casita-stage-child-stopped"),
            "staging child failed: stderr={}, stdout={}",
            String::from_utf8_lossy(&child.stderr),
            String::from_utf8_lossy(&child.stdout),
        );
        assert!(!root.path().join("child-scratch").exists());
        let staged: PathInfo =
            serde_json::from_slice(&std::fs::read(root.path().join("staged-info.json")).unwrap()).unwrap();
        let target: ObjectKey = std::fs::read_to_string(root.path().join("staged-target")).unwrap().parse().unwrap();
        let name = CasitaStore::root_name(&staged.store_path).unwrap();
        let repository = LocalRepository::local(state.join("casita")).await.unwrap();
        let snapshot = repository.metadata().snapshot().await.unwrap();
        assert!(snapshot.root(&name).await.unwrap().is_none());
        assert!(snapshot.object(&target).await.unwrap().is_some(), "child stage was not durable");
        drop(snapshot);
        let config = || StoreConfig::new(StoreBackend::Casita, state.clone(), output.clone(), STORE_DIR.to_string());
        let reader = StoreHandle::open(config()).await.unwrap();
        assert!(reader.pathinfo_service().get(*staged.store_path.digest()).await.unwrap().is_none());
        drop(reader);
        let preview = repository.preview_collection().await.unwrap();
        assert!(
            preview.logical_objects > 0,
            "stopped child left no reclaimable unrooted objects; pins={:?}",
            repository.metadata().pin_store().await.unwrap().inventory().await.unwrap().pins,
        );
        let reclaimed = repository.try_collect().await.unwrap();
        assert!(reclaimed.removed.logical_objects > 0, "child's unrooted envelope was not reclaimed");
        let snapshot = repository.metadata().snapshot().await.unwrap();
        assert!(snapshot.root(&name).await.unwrap().is_none());
        assert!(snapshot.object(&target).await.unwrap().is_none());
        drop(snapshot);
        let store = StoreHandle::open(config()).await.unwrap();
        assert!(store.pathinfo_service().get(*staged.store_path.digest()).await.unwrap().is_none());
        let retry = fixture(&store, root.path(), NAME, CONTENT, &signing_key).await;
        assert_eq!(retry, staged);
        store.pathinfo_service().put(retry.clone()).await.unwrap();
        drop(store);
        drop(repository);
        let reopened = StoreHandle::open(config()).await.unwrap();
        assert_eq!(reopened.pathinfo_service().get(*retry.store_path.digest()).await.unwrap(), Some(retry));
        assert_eq!(std::fs::read(&policy).unwrap().as_slice(), policy_bytes.as_bytes());
    }

    struct RepointBeforeSigning {
        selected: Arc<dyn PathInfoService>,
        competing: Arc<dyn PathInfoService>,
        replacement: PathInfo,
    }

    #[async_trait]
    impl PathInfoService for RepointBeforeSigning {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            self.selected.get(digest).await
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            let prior = self.competing.get(*path_info.store_path.digest()).await?.unwrap();
            assert_ne!(prior, self.replacement);
            self.competing.put(self.replacement.clone()).await?;
            self.selected.put(path_info).await
        }

        fn list(&self) -> BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            self.selected.list()
        }
    }

    #[tokio::test]
    async fn store_sign_rejects_second_client_repoint_between_read_and_update() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir(&state).unwrap();
        std::fs::create_dir(&output).unwrap();
        let raw_one = ed25519_dalek::SigningKey::from_bytes(&[17_u8; 32]);
        let raw_two = ed25519_dalek::SigningKey::from_bytes(&[18_u8; 32]);
        let verifying_one = VerifyingKey::new("casita-fixture-1".to_string(), raw_one.verifying_key());
        let verifying_two = VerifyingKey::new("casita-fixture-2".to_string(), raw_two.verifying_key());
        let signing_one = SigningKey::new("casita-fixture-1".to_string(), raw_one);
        let signing_two = SigningKey::new("casita-fixture-2".to_string(), raw_two);
        std::fs::write(state.join(TRUST_FILE_NAME), format!("{verifying_one}\n{verifying_two}\n")).unwrap();
        let config = || StoreConfig::new(StoreBackend::Casita, state.clone(), output.clone(), STORE_DIR.to_string());
        let store = StoreHandle::open(config()).await.unwrap();
        let competitor = StoreHandle::open(config()).await.unwrap();
        let info = fixture(&store, root.path(), "race-output", b"winner's content", &signing_one).await;
        store.pathinfo_service().put(info.clone()).await.unwrap();
        let repository = LocalRepository::local(state.join("casita")).await.unwrap();
        let name = CasitaStore::root_name(&info.store_path).unwrap();
        let original = repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap().unwrap();
        let mut replacement = info.clone();
        resign(&mut replacement, &signing_two);
        let racing = RepointBeforeSigning {
            selected: store.pathinfo_service(),
            competing: competitor.pathinfo_service(),
            replacement: replacement.clone(),
        };
        assert!(
            crate::query::store_sign(&racing, &signing_two, Some("race-output"), false, STORE_DIR,)
                .await
                .unwrap_err()
                .to_string()
                .contains("casita-root-conflict")
        );
        let target = repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap().unwrap();
        assert_ne!(target, original);
        assert_eq!(store.pathinfo_service().get(*info.store_path.digest()).await.unwrap(), Some(replacement));
        assert_eq!(repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap(), Some(target));
    }

    #[tokio::test]
    async fn invalid_trust_policy_cannot_open_casita_repository() {
        let raw = ed25519_dalek::SigningKey::from_bytes(&[17_u8; 32]);
        let key = VerifyingKey::new("casita-fixture-1".to_string(), raw.verifying_key());
        let over_limit = format!("{key}\n").repeat(65).into_bytes();
        let oversized = vec![b' '; usize::try_from(MAX_TRUST_FILE_BYTES).unwrap() + 1];
        for bytes in [Vec::new(), b"invalid-ed25519-key\n".to_vec(), over_limit, oversized] {
            let root = tempfile::tempdir().unwrap();
            let state = root.path().join("state");
            let output = root.path().join("output");
            std::fs::create_dir(&state).unwrap();
            std::fs::create_dir(&output).unwrap();
            let policy = state.join(TRUST_FILE_NAME);
            std::fs::write(&policy, &bytes).unwrap();
            let error =
                StoreHandle::open(StoreConfig::new(StoreBackend::Casita, state.clone(), output, STORE_DIR.to_string()))
                    .await
                    .err()
                    .unwrap();
            assert!(error.to_string().contains("casita-trust-policy-invalid"), "{error}");
            assert_eq!(std::fs::read(&policy).unwrap(), bytes);
            assert!(!state.join("casita").exists());
        }
        #[cfg(unix)]
        {
            let root = tempfile::tempdir().unwrap();
            let state = root.path().join("state");
            let output = root.path().join("output");
            std::fs::create_dir(&state).unwrap();
            std::fs::create_dir(&output).unwrap();
            let policy = state.join(TRUST_FILE_NAME);
            std::os::unix::fs::symlink("missing-policy", &policy).unwrap();
            let error =
                StoreHandle::open(StoreConfig::new(StoreBackend::Casita, state.clone(), output, STORE_DIR.to_string()))
                    .await
                    .err()
                    .unwrap();
            assert!(error.to_string().contains("casita-trust-policy-invalid"), "{error}");
            assert_eq!(std::fs::read_link(&policy).unwrap(), Path::new("missing-policy"));
            assert!(!state.join("casita").exists());
        }
    }

    #[tokio::test]
    async fn local_signer_without_policy_survives_fresh_process() {
        if let Some(root) = std::env::var_os("MANTLE_CASITA_LOCAL_CHILD_ROOT") {
            let root = PathBuf::from(root);
            let state = root.join("state");
            let output = root.join("output");
            let store_path = StorePath::from_absolute_path_with_prefix(
                std::env::var("MANTLE_CASITA_LOCAL_CHILD_PATH").unwrap().as_bytes(),
                STORE_DIR,
            )
            .unwrap();
            let handle =
                StoreHandle::open(StoreConfig::new(StoreBackend::Casita, state.clone(), output, STORE_DIR.to_string()))
                    .await
                    .unwrap();
            let info = handle.pathinfo_service().get(*store_path.digest()).await.unwrap().unwrap();
            assert_eq!(info.store_path, store_path);
            assert!(handle.castore_has_complete_content(&info.node).await.unwrap());
            assert!(!state.join(TRUST_FILE_NAME).exists());
            println!("casita-local-child-verified");
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir(&state).unwrap();
        std::fs::create_dir(&output).unwrap();
        let raw = ed25519_dalek::SigningKey::from_bytes(&[27_u8; 32]);
        let signer = SigningKey::new("casita-local".to_string(), raw.clone());
        std::fs::write(
            state.join("signing-key"),
            format!("casita-local:{}\n", data_encoding::BASE64.encode(&raw.to_keypair_bytes())),
        )
        .unwrap();
        let handle =
            StoreHandle::open(StoreConfig::new(StoreBackend::Casita, state.clone(), output, STORE_DIR.to_string()))
                .await
                .unwrap();
        let info = fixture(&handle, root.path(), "local-output", b"local signer without policy", &signer).await;
        handle.pathinfo_service().put(info.clone()).await.unwrap();
        assert!(!state.join(TRUST_FILE_NAME).exists());
        drop(handle);
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("casita::tests::local_signer_without_policy_survives_fresh_process")
            .arg("--exact")
            .arg("--nocapture")
            .env("MANTLE_CASITA_LOCAL_CHILD_ROOT", root.path())
            .env("MANTLE_CASITA_LOCAL_CHILD_PATH", info.store_path.to_absolute_path_with_prefix(STORE_DIR))
            .output()
            .unwrap();
        assert!(
            child.status.success() && String::from_utf8_lossy(&child.stdout).contains("casita-local-child-verified"),
            "local child status={} stderr={} stdout={}",
            child.status,
            String::from_utf8_lossy(&child.stderr),
            String::from_utf8_lossy(&child.stdout),
        );
        assert!(!state.join(TRUST_FILE_NAME).exists());
    }
    #[tokio::test]
    async fn cached_node_rechecks_revoked_policy_in_same_session() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir(&state).unwrap();
        let raw = ed25519_dalek::SigningKey::from_bytes(&[29_u8; 32]);
        let signer = SigningKey::new("casita-revoked".to_string(), raw.clone());
        let trusted = VerifyingKey::new("casita-revoked".to_string(), raw.verifying_key());
        let other_raw = ed25519_dalek::SigningKey::from_bytes(&[30_u8; 32]);
        let replacement = VerifyingKey::new("casita-revoked".to_string(), other_raw.verifying_key());
        let policy = state.join(TRUST_FILE_NAME);
        std::fs::write(&policy, format!("{trusted}\n")).unwrap();
        let mut store = StoreHandle::open(StoreConfig::new(StoreBackend::Casita, state, output, STORE_DIR.to_string()))
            .await
            .unwrap();
        let info = fixture(&store, root.path(), "revoked-cache", b"trusted before revocation", &signer).await;
        store.pathinfo_service().put(info.clone()).await.unwrap();
        assert_eq!(store.cached_node_for_path(&info.store_path).await.unwrap(), Some(info.node.clone()));
        let replaced_policy = format!("{replacement}\n");
        std::fs::write(&policy, &replaced_policy).unwrap();
        let error = store.cached_node_for_path(&info.store_path).await.unwrap_err();
        assert!(error.to_string().contains("casita-signer-untrusted"), "{error}");
        assert_eq!(std::fs::read(&policy).unwrap().as_slice(), replaced_policy.as_bytes());
        std::fs::write(&policy, format!("{trusted}\n")).unwrap();
        assert_eq!(store.cached_node_for_path(&info.store_path).await.unwrap(), Some(info.node));
        let repository = store.casita_store.as_ref().unwrap().repository.clone();
        let name = CasitaStore::root_name(&info.store_path).unwrap();
        let expected = repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap().unwrap();
        let invalid_policy = b"not-an-ed25519-key\n";
        std::fs::write(&policy, invalid_policy).unwrap();
        let error = store.cached_node_for_path(&info.store_path).await.unwrap_err();
        assert!(error.to_string().contains("casita-trust-policy-invalid"), "{error}");
        let error = crate::query::store_verify(store.pathinfo_service().as_ref(), None, &root.path().join("output"))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("casita-trust-policy-invalid"), "{error}");
        let guard = StoreMutationGuard::try_acquire(&root.path().join("state")).unwrap();
        let error = store.garbage_collect_under_guard(&guard, None).await.unwrap_err();
        assert!(error.to_string().contains("casita-trust-policy-invalid"), "{error}");
        assert_eq!(std::fs::read(&policy).unwrap().as_slice(), invalid_policy);
        assert_eq!(repository.metadata().snapshot().await.unwrap().root(&name).await.unwrap(), Some(expected));
    }
}
