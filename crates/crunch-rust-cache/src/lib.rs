#![cfg_attr(not(kani), feature(register_tool))]
#![register_tool(tigerstyle)]
//! Thin filesystem and castore shell for local Rust unit result reuse.
//!
//! The shell owns bounded I/O, staging, locking, and atomic publication. The
//! `crunch-rust-cache-core` crate owns canonical identity and admission logic.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crunch_rust_cache_core::BLAKE3_HEX_CHARS;
use crunch_rust_cache_core::CastoreNodeIdentity;
use crunch_rust_cache_core::CastoreNodeKind;
use crunch_rust_cache_core::LocalCachePolicy;
use crunch_rust_cache_core::LocalCandidateFacts;
use crunch_rust_cache_core::RustArtifactKind;
use crunch_rust_cache_core::RustResultArtifact;
use crunch_rust_cache_core::RustResultIndex;
use crunch_rust_cache_core::RustUnitAction;
use crunch_rust_cache_core::RustUnitResult;
use crunch_rust_cache_core::RustUnitResultInput;
use crunch_rust_cache_core::canonical_result_index;
use crunch_rust_cache_core::canonical_rust_result;
use crunch_rust_cache_core::plan_local_reuse;
use crunch_rust_cache_core::validate_result_index;
use crunch_rust_cache_core::validate_rust_action;
use crunch_rust_cache_core::validate_rust_result;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use snix_castore::B3Digest;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::import::fs::ingest_path;
use tempfile::Builder;
use thiserror::Error;

pub mod shared;

pub const RUST_UNIT_EXECUTION_RECEIPT_FILE: &str = ".mantle-rust-unit-execution.json";
pub const RUST_CACHE_RETENTION_SCHEMA: &str = "mantle-rust-unit-retention-v1";
pub const CACHE_DISPOSITION_DISABLED: &str = "local-cache-disabled";
pub const CACHE_DISPOSITION_MISS: &str = "local-cache-miss";
pub const CACHE_DISPOSITION_HIT: &str = "local-castore-reuse";
pub const CACHE_DISPOSITION_REJECTED: &str = "local-cache-rejected";
pub const CACHE_DISPOSITION_CONFLICT: &str = "local-cache-conflict";
pub const MAX_CACHE_JSON_BYTES: u64 = 4_194_304;
pub const MAX_SCAN_ENTRIES: u32 = crunch_rust_cache_core::MAX_TREE_ENTRIES;
pub const MAX_SCAN_DEPTH: u32 = crunch_rust_cache_core::MAX_TREE_DEPTH;
pub const MAX_SCAN_BYTES: u64 = crunch_rust_cache_core::MAX_TREE_BYTES;

const CACHE_DIRECTORY: &str = "rust-unit-cache";
const INDEX_DIRECTORY: &str = "indexes";
const RESULT_DIRECTORY: &str = "results";
const STAGING_DIRECTORY: &str = "staging";
const RETENTION_FILE: &str = "retained-nodes.json";
const MUTATION_LOCK_FILE: &str = "mutation.lock";
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;
const REGULAR_FILE_MODE: u32 = 0o644;
const EXECUTABLE_FILE_MODE: u32 = 0o755;
const EXECUTABLE_PERMISSION_BITS: u32 = 0o111;
const READ_BUFFER_BYTES: usize = 65_536;
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(10);
const HEX_RADIX: u32 = 16;
const HEX_BYTE_CHARS: usize = 2;
const EXTRA_EOF_READ_ITERATIONS: u64 = 2;
const LOCK_MAX_ATTEMPTS: u32 = 500;

#[derive(Clone, Copy)]
struct CommitDevices {
    staging: u64,
    destination: u64,
}

#[derive(Clone, Copy)]
struct TypedRefPrefix<'a>(&'a str);

#[derive(Debug, Error)]
pub enum Error {
    #[error("rust-cache-core:{0}")]
    Core(String),
    #[error("rust-cache-io:{context}:{source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("rust-cache-json:{0}")]
    Json(String),
    #[error("rust-cache-castore:{0}")]
    Castore(String),
    #[error("rust-cache-bound:{0}")]
    Bound(String),
    #[error("rust-cache-state:{0}")]
    State(String),
    #[error("rust-cache-lock-timeout")]
    LockTimeout,
}

#[derive(Clone)]
pub struct RustCache {
    state_dir: PathBuf,
    cache_dir: PathBuf,
    indexes_dir: PathBuf,
    results_dir: PathBuf,
    staging_dir: PathBuf,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustCacheRetention {
    pub schema: String,
    pub retained_results: BTreeMap<String, CastoreNodeIdentity>,
}

impl Default for RustCacheRetention {
    fn default() -> Self {
        Self {
            schema: RUST_CACHE_RETENTION_SCHEMA.to_string(),
            retained_results: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustCacheReport {
    pub disposition: String,
    pub reason_codes: Vec<String>,
    pub selected_result_ref: Option<String>,
    pub candidate_count: u32,
    pub artifact_count: u32,
    pub restored_bytes: u64,
    pub reused_bytes: u64,
    pub compiler_executed: bool,
}

#[derive(Debug)]
pub struct RustCacheRetentionPlan {
    live_nodes: Vec<Node>,
    stale_result_paths: Vec<PathBuf>,
    index_updates: Vec<(PathBuf, Option<RustResultIndex>)>,
    mutation_lock_path: PathBuf,
}

impl RustCacheRetentionPlan {
    pub fn live_nodes(&self) -> &[Node] {
        assert!(self.live_nodes.len() <= crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
        assert!(!self.mutation_lock_path.as_os_str().is_empty());
        &self.live_nodes
    }

    pub fn stale_result_count(&self) -> u32 {
        assert!(self.stale_result_paths.len() <= crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
        assert!(u32::try_from(self.stale_result_paths.len()).is_ok());
        self.stale_result_paths.len() as u32
    }

    pub fn apply(&self, is_dry_run: bool) -> Result<(), Error> {
        if is_dry_run {
            return Ok(());
        }
        let _lock = CacheMutationLock::acquire(&self.mutation_lock_path)?;
        for (path, replacement) in &self.index_updates {
            if let Some(index) = replacement {
                write_atomic_json(path, index)?;
            } else {
                fs::remove_file(path).map_err(|source| Error::Io {
                    context: "remove-stale-index".to_string(),
                    source,
                })?;
            }
        }
        for path in &self.stale_result_paths {
            fs::remove_file(path).map_err(|source| Error::Io {
                context: "remove-stale-result".to_string(),
                source,
            })?;
        }
        assert!(self.stale_result_paths.iter().all(|path| !path.exists()));
        assert!(!self.mutation_lock_path.as_os_str().is_empty());
        Ok(())
    }
}

impl RustCacheReport {
    pub fn disabled() -> Self {
        Self {
            disposition: CACHE_DISPOSITION_DISABLED.to_string(),
            reason_codes: vec![CACHE_DISPOSITION_DISABLED.to_string()],
            selected_result_ref: None,
            candidate_count: 0,
            artifact_count: 0,
            restored_bytes: 0,
            reused_bytes: 0,
            compiler_executed: false,
        }
    }

    pub fn compiler_executed(mut self) -> Self {
        self.compiler_executed = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PublishRequest<'a> {
    pub action: &'a RustUnitAction,
    pub output_dir: &'a Path,
    pub producer_receipt_ref: &'a str,
    pub policy: &'a LocalCachePolicy,
}

impl std::fmt::Debug for RustCache {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RustCache")
            .field("state_dir", &self.state_dir)
            .field("cache_dir", &self.cache_dir)
            .finish_non_exhaustive()
    }
}

impl RustCache {
    pub fn open(config: crunch_store::StoreConfig) -> Result<Self, Error> {
        let runtime =
            tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|source| Error::Io {
                context: "create-cache-runtime".to_string(),
                source,
            })?;
        runtime.block_on(Self::open_async(config))
    }

    pub async fn open_async(config: crunch_store::StoreConfig) -> Result<Self, Error> {
        let handle = crunch_store::StoreHandle::open(config)
            .await
            .map_err(|error| Error::Castore(format!("open-store:{error}")))?;
        let cache = Self::new(handle.state_dir().to_path_buf(), handle.blob_service(), handle.directory_service())?;
        assert_eq!(cache.state_dir(), handle.state_dir());
        assert!(cache.cache_dir.starts_with(handle.state_dir()));
        Ok(cache)
    }

    pub fn new(
        state_dir: PathBuf,
        blob_service: Arc<dyn BlobService>,
        directory_service: Arc<dyn DirectoryService>,
    ) -> Result<Self, Error> {
        if state_dir.as_os_str().is_empty() {
            return Err(Error::State("state-directory-empty".to_string()));
        }
        let cache_dir = state_dir.join(CACHE_DIRECTORY);
        let indexes_dir = cache_dir.join(INDEX_DIRECTORY);
        let results_dir = cache_dir.join(RESULT_DIRECTORY);
        let staging_dir = cache_dir.join(STAGING_DIRECTORY);
        for directory in [&cache_dir, &indexes_dir, &results_dir, &staging_dir] {
            create_private_directory(directory)?;
        }
        assert!(cache_dir.starts_with(&state_dir));
        assert!(staging_dir.starts_with(&cache_dir));
        Ok(Self {
            state_dir,
            cache_dir,
            indexes_dir,
            results_dir,
            staging_dir,
            blob_service,
            directory_service,
        })
    }

    pub fn state_dir(&self) -> &Path {
        assert!(!self.state_dir.as_os_str().is_empty());
        assert!(self.cache_dir.starts_with(&self.state_dir));
        &self.state_dir
    }

    pub fn publish_blocking(&self, request: PublishRequest<'_>) -> Result<RustUnitResult, Error> {
        let runtime =
            tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|source| Error::Io {
                context: "create-publish-runtime".to_string(),
                source,
            })?;
        let result = runtime.block_on(self.publish(request))?;
        assert!(!result.result_ref.is_empty());
        assert!(!result.input.artifacts.is_empty());
        Ok(result)
    }

    pub async fn publish(&self, request: PublishRequest<'_>) -> Result<RustUnitResult, Error> {
        validate_rust_action(request.action).map_err(|error| Error::Core(String::from(error.code())))?;
        crunch_rust_cache_core::validate_local_cache_policy(request.policy)
            .map_err(|error| Error::Core(String::from(error.code())))?;
        if !request.policy.writes_enabled {
            return Err(Error::State("local-cache-write-disabled".to_string()));
        }
        let _store_guard = crunch_store::StoreMutationGuard::acquire_wait(&self.state_dir)
            .map_err(|error| Error::State(format!("store-mutation-lock:{error}")))?;
        let manifest = scan_output_artifacts(request.output_dir, request.policy)?;
        let snapshot = self.snapshot_declared_artifacts(request.output_dir, &manifest)?;
        let node = ingest_path(
            &self.blob_service,
            &self.directory_service,
            snapshot.path(),
            None::<&snix_castore::refscan::ReferenceScanner<Vec<u8>>>,
        )
        .await
        .map_err(|error| Error::Castore(format!("ingest:{error}")))?;
        let root_node = node_identity(&node)?;
        let result = canonical_rust_result(RustUnitResultInput {
            action_ref: request.action.action_ref.clone(),
            root_node,
            artifacts: manifest,
            producer_receipt_ref: request.producer_receipt_ref.to_string(),
        })
        .map_err(|error| Error::Core(String::from(error.code())))?;
        let is_complete =
            crunch_store::recursive_castore_completeness(&*self.blob_service, &*self.directory_service, &node)
                .await
                .map_err(|error| Error::Castore(format!("completeness:{error}")))?;
        if !is_complete {
            return Err(Error::Castore("ingested-tree-incomplete".to_string()));
        }
        self.publish_record_and_index(&result)?;
        assert_eq!(result.input.action_ref, request.action.action_ref);
        assert!(!result.input.artifacts.is_empty());
        Ok(result)
    }

    pub fn restore_blocking(
        &self,
        action: &RustUnitAction,
        output_dir: &Path,
        policy: &LocalCachePolicy,
    ) -> Result<RustCacheReport, Error> {
        let runtime =
            tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|source| Error::Io {
                context: "create-restore-runtime".to_string(),
                source,
            })?;
        let outcome = runtime.block_on(self.restore(action, output_dir, policy))?;
        assert!(!outcome.disposition.is_empty());
        assert!(outcome.candidate_count <= policy.max_candidates);
        Ok(outcome)
    }

    pub async fn restore(
        &self,
        action: &RustUnitAction,
        output_dir: &Path,
        policy: &LocalCachePolicy,
    ) -> Result<RustCacheReport, Error> {
        validate_rust_action(action).map_err(|error| Error::Core(String::from(error.code())))?;
        crunch_rust_cache_core::validate_local_cache_policy(policy)
            .map_err(|error| Error::Core(String::from(error.code())))?;
        if !policy.reads_enabled {
            return Ok(RustCacheReport::disabled());
        }
        let _store_guard = crunch_store::StoreMutationGuard::acquire_wait(&self.state_dir)
            .map_err(|error| Error::State(format!("store-mutation-lock:{error}")))?;
        if output_dir.exists() {
            return Ok(rejected_report("rust-local-cache-output-exists"));
        }
        let Some(index) = self.read_index(&action.action_ref)? else {
            return Ok(miss_report());
        };
        let candidates = self.load_candidate_facts(&index, policy).await?;
        let candidate_count = bounded_count(candidates.len())?;
        let plan = plan_local_reuse(&action.action_ref, policy, candidates)
            .map_err(|error| Error::Core(String::from(error.code())))?;
        if let Some(conflict) = plan.conflict_class {
            return Ok(RustCacheReport {
                disposition: CACHE_DISPOSITION_CONFLICT.to_string(),
                reason_codes: vec![conflict],
                selected_result_ref: None,
                candidate_count,
                artifact_count: 0,
                restored_bytes: 0,
                reused_bytes: 0,
                compiler_executed: false,
            });
        }
        let Some(selected_ref) = plan.selected_result_ref else {
            return Ok(rejected_plan_report(candidate_count, &plan.decisions));
        };
        let result = self.read_result(&selected_ref)?;
        self.materialize_and_commit(&result, output_dir, policy).await?;
        let restored_bytes = artifact_bytes(&result.input.artifacts)?;
        Ok(RustCacheReport {
            disposition: CACHE_DISPOSITION_HIT.to_string(),
            reason_codes: vec![CACHE_DISPOSITION_HIT.to_string()],
            selected_result_ref: Some(selected_ref),
            candidate_count,
            artifact_count: bounded_count(result.input.artifacts.len())?,
            restored_bytes,
            reused_bytes: restored_bytes,
            compiler_executed: false,
        })
    }

    pub fn retention(&self) -> Result<RustCacheRetention, Error> {
        let path = self.cache_dir.join(RETENTION_FILE);
        let Some(bytes) = read_bounded_optional(&path)? else {
            return Ok(RustCacheRetention::default());
        };
        let retention = serde_json::from_slice::<RustCacheRetention>(&bytes)
            .map_err(|error| Error::Json(format!("retention-decode:{error}")))?;
        validate_retention(&retention)?;
        assert_eq!(retention.schema, RUST_CACHE_RETENTION_SCHEMA);
        assert!(retention.retained_results.len() <= crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
        Ok(retention)
    }

    pub fn plan_retention_gc(&self) -> Result<RustCacheRetentionPlan, Error> {
        let _lock = CacheMutationLock::acquire(&self.cache_dir.join(MUTATION_LOCK_FILE))?;
        let retention = self.retention()?;
        let mut indexes = BTreeMap::new();
        let mut live_nodes = Vec::with_capacity(retention.retained_results.len());
        let mut live_result_paths = BTreeSet::new();
        let mut retained_result_actions = BTreeMap::new();
        for (result_ref, retained_node) in &retention.retained_results {
            let result = self.read_result(result_ref)?;
            if &result.input.root_node != retained_node {
                return Err(Error::State("retention-result-node-mismatch".to_string()));
            }
            if !indexes.contains_key(&result.input.action_ref) {
                if indexes.len() >= crunch_rust_cache_core::MAX_RESULT_CANDIDATES {
                    return Err(Error::Bound("retention-index-limit-exceeded".to_string()));
                }
                let index = self
                    .read_index(&result.input.action_ref)?
                    .ok_or_else(|| Error::State("retention-index-missing".to_string()))?;
                indexes.insert(result.input.action_ref.clone(), index);
            }
            let index = indexes
                .get(&result.input.action_ref)
                .ok_or_else(|| Error::State("retention-index-not-loaded".to_string()))?;
            if !index.result_refs.contains(result_ref) {
                return Err(Error::State("retention-result-not-indexed".to_string()));
            }
            live_nodes.push(node_from_identity(retained_node)?);
            live_result_paths.insert(self.result_path(result_ref)?);
            if retained_result_actions.len() >= crunch_rust_cache_core::MAX_RESULT_CANDIDATES {
                return Err(Error::Bound("retention-result-action-limit-exceeded".to_string()));
            }
            retained_result_actions.insert(result_ref.clone(), result.input.action_ref);
        }
        let stale_result_paths = collect_stale_result_paths(&self.results_dir, &live_result_paths)?;
        let index_updates = collect_stale_index_updates(&self.indexes_dir, &retained_result_actions)?;
        assert_eq!(live_nodes.len(), retention.retained_results.len());
        assert!(stale_result_paths.iter().all(|path| !live_result_paths.contains(path)));
        Ok(RustCacheRetentionPlan {
            live_nodes,
            stale_result_paths,
            index_updates,
            mutation_lock_path: self.cache_dir.join(MUTATION_LOCK_FILE),
        })
    }

    fn snapshot_declared_artifacts(
        &self,
        output_dir: &Path,
        artifacts: &[RustResultArtifact],
    ) -> Result<tempfile::TempDir, Error> {
        let temp = Builder::new().prefix("snapshot-").tempdir_in(&self.staging_dir).map_err(|source| Error::Io {
            context: "create-snapshot-staging".to_string(),
            source,
        })?;
        for artifact in artifacts {
            if artifact.kind != RustArtifactKind::File {
                return Err(Error::State("symlink-artifact-unsupported".to_string()));
            }
            copy_declared_artifact(output_dir, temp.path(), artifact)?;
        }
        assert!(temp.path().starts_with(&self.staging_dir));
        assert!(!artifacts.is_empty());
        Ok(temp)
    }

    fn publish_record_and_index(&self, result: &RustUnitResult) -> Result<(), Error> {
        validate_rust_result(result).map_err(|error| Error::Core(String::from(error.code())))?;
        let _lock = CacheMutationLock::acquire(&self.cache_dir.join(MUTATION_LOCK_FILE))?;
        let result_path = self.result_path(&result.result_ref)?;
        write_immutable_json(&result_path, result)?;
        let current = self.read_index(&result.input.action_ref)?;
        let mut result_refs = current.map_or_else(Vec::new, |index| index.result_refs);
        result_refs.push(result.result_ref.clone());
        let index = canonical_result_index(result.input.action_ref.clone(), result_refs)
            .map_err(|error| Error::Core(String::from(error.code())))?;
        write_atomic_json(&self.index_path(&result.input.action_ref)?, &index)?;
        let mut retention = self.retention()?;
        retention.retained_results.insert(result.result_ref.clone(), result.input.root_node.clone());
        validate_retention(&retention)?;
        write_atomic_json(&self.cache_dir.join(RETENTION_FILE), &retention)?;
        assert!(retention.retained_results.contains_key(&result.result_ref));
        assert!(index.result_refs.contains(&result.result_ref));
        Ok(())
    }

    fn read_index(&self, action_ref: &str) -> Result<Option<RustResultIndex>, Error> {
        let path = self.index_path(action_ref)?;
        let Some(bytes) = read_bounded_optional(&path)? else {
            return Ok(None);
        };
        let index = serde_json::from_slice::<RustResultIndex>(&bytes)
            .map_err(|error| Error::Json(format!("index-decode:{error}")))?;
        validate_result_index(&index).map_err(|error| Error::Core(String::from(error.code())))?;
        if index.action_ref != action_ref {
            return Err(Error::State("index-action-ref-mismatch".to_string()));
        }
        assert_eq!(index.action_ref, action_ref);
        assert!(!index.result_refs.is_empty());
        Ok(Some(index))
    }

    fn read_result(&self, result_ref: &str) -> Result<RustUnitResult, Error> {
        let path = self.result_path(result_ref)?;
        let bytes = read_bounded_required(&path)?;
        let result = serde_json::from_slice::<RustUnitResult>(&bytes)
            .map_err(|error| Error::Json(format!("result-decode:{error}")))?;
        validate_rust_result(&result).map_err(|error| Error::Core(String::from(error.code())))?;
        if result.result_ref != result_ref {
            return Err(Error::State("result-reference-path-mismatch".to_string()));
        }
        assert_eq!(result.result_ref, result_ref);
        assert!(!result.input.artifacts.is_empty());
        Ok(result)
    }

    async fn load_candidate_facts(
        &self,
        index: &RustResultIndex,
        policy: &LocalCachePolicy,
    ) -> Result<Vec<LocalCandidateFacts>, Error> {
        let max_candidates = usize::try_from(policy.max_candidates)
            .map_err(|_| Error::Bound("candidate-limit-unrepresentable".to_string()))?;
        if index.result_refs.len() > max_candidates {
            return Err(Error::Bound("candidate-limit-exceeded".to_string()));
        }
        let mut candidates = Vec::with_capacity(index.result_refs.len());
        for result_ref in &index.result_refs {
            let result = self.read_result(result_ref)?;
            let node = node_from_identity(&result.input.root_node)?;
            let is_complete =
                crunch_store::recursive_castore_completeness(&*self.blob_service, &*self.directory_service, &node)
                    .await
                    .map_err(|error| Error::Castore(format!("candidate-completeness:{error}")))?;
            candidates.push(LocalCandidateFacts {
                result,
                content_complete: is_complete,
                artifact_manifest_verified: true,
            });
        }
        assert!(candidates.len() <= max_candidates);
        assert_eq!(candidates.len(), index.result_refs.len());
        Ok(candidates)
    }

    async fn materialize_and_commit(
        &self,
        result: &RustUnitResult,
        output_dir: &Path,
        policy: &LocalCachePolicy,
    ) -> Result<(), Error> {
        let parent = output_dir.parent().ok_or_else(|| Error::State("output-parent-missing".to_string()))?;
        create_private_directory(parent)?;
        let temp = Builder::new().prefix("restore-").tempdir_in(parent).map_err(|source| Error::Io {
            context: "create-restore-staging".to_string(),
            source,
        })?;
        let temp = RestoreStaging::new(temp);
        let node = node_from_identity(&result.input.root_node)?;
        let staging_path = temp.path()?;
        let staging_text = staging_path.to_str().ok_or_else(|| Error::State("restore-path-non-utf8".to_string()))?;
        crunch_store::export_castore_to_disk(&node, staging_text, &self.blob_service, &self.directory_service)
            .await
            .map_err(|error| Error::Castore(format!("restore-export:{error}")))?;
        let observed = scan_output_artifacts(staging_path, policy)?;
        if observed != result.input.artifacts {
            return Err(Error::State("restored-artifact-manifest-mismatch".to_string()));
        }
        if output_dir.exists() {
            return Err(Error::State("restore-output-raced".to_string()));
        }
        let staging = temp.keep()?;
        ensure_same_commit_device(&staging, parent)?;
        fs::set_permissions(&staging, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).map_err(|source| {
            Error::Io {
                context: "make-restore-root-writable".to_string(),
                source,
            }
        })?;
        if let Err(source) = fs::rename(&staging, output_dir) {
            make_owner_writable(&staging);
            let _cleanup = fs::remove_dir_all(&staging);
            return Err(Error::Io {
                context: format!("commit-restored-output:parent-mode={:o}", path_mode(parent)),
                source,
            });
        }
        sync_directory(parent)?;
        assert!(output_dir.is_dir());
        assert_eq!(observed, result.input.artifacts);
        Ok(())
    }

    fn index_path(&self, action_ref: &str) -> Result<PathBuf, Error> {
        typed_ref_path(&self.indexes_dir, action_ref, TypedRefPrefix(crunch_rust_cache_core::RUST_ACTION_REF_PREFIX))
    }

    fn result_path(&self, result_ref: &str) -> Result<PathBuf, Error> {
        typed_ref_path(&self.results_dir, result_ref, TypedRefPrefix(crunch_rust_cache_core::RUST_RESULT_REF_PREFIX))
    }
}

fn ensure_same_commit_device(staging: &Path, destination_parent: &Path) -> Result<(), Error> {
    let staging_device = fs::metadata(staging)
        .map_err(|source| Error::Io {
            context: "stat-restore-staging-device".to_string(),
            source,
        })?
        .dev();
    let destination_device = fs::metadata(destination_parent)
        .map_err(|source| Error::Io {
            context: "stat-restore-parent-device".to_string(),
            source,
        })?
        .dev();
    validate_commit_devices(CommitDevices {
        staging: staging_device,
        destination: destination_device,
    })?;
    assert_eq!(staging_device, destination_device);
    assert!(staging.starts_with(destination_parent));
    Ok(())
}

fn validate_commit_devices(devices: CommitDevices) -> Result<(), Error> {
    if devices.staging != devices.destination {
        return Err(Error::State("cross-filesystem-restore-commit-rejected".to_string()));
    }
    assert_eq!(devices.staging, devices.destination);
    assert!(devices.staging == devices.destination);
    Ok(())
}

fn scan_output_artifacts(output_dir: &Path, policy: &LocalCachePolicy) -> Result<Vec<RustResultArtifact>, Error> {
    if !output_dir.is_dir() {
        return Err(Error::State("artifact-output-directory-missing".to_string()));
    }
    let max_entries = usize::try_from(policy.max_tree_entries)
        .map_err(|_| Error::Bound("artifact-tree-entry-limit-unrepresentable".to_string()))?;
    let mut pending = vec![(output_dir.to_path_buf(), String::new(), 0_u32)];
    let mut artifacts = Vec::with_capacity(max_entries);
    let mut total_bytes = 0_u64;
    while let Some((directory, relative_parent, depth)) = pending.pop() {
        if depth > policy.max_tree_depth {
            return Err(Error::Bound("artifact-tree-depth-exceeded".to_string()));
        }
        let entries = sorted_directory_entries(&directory)?;
        for entry in entries.into_iter().rev() {
            let name =
                entry.file_name().into_string().map_err(|_| Error::State("artifact-path-non-utf8".to_string()))?;
            let relative = if relative_parent.is_empty() {
                name
            } else {
                format!("{relative_parent}/{name}")
            };
            if relative == RUST_UNIT_EXECUTION_RECEIPT_FILE {
                continue;
            }
            let metadata = fs::symlink_metadata(entry.path()).map_err(|source| Error::Io {
                context: "stat-artifact".to_string(),
                source,
            })?;
            if metadata.file_type().is_symlink() {
                return Err(Error::State("symlink-artifact-unsupported".to_string()));
            }
            if metadata.is_dir() {
                pending.push((
                    entry.path(),
                    relative,
                    depth.checked_add(1).ok_or_else(|| Error::Bound("artifact-depth-overflow".to_string()))?,
                ));
                continue;
            }
            if !metadata.is_file() {
                return Err(Error::State("artifact-type-unsupported".to_string()));
            }
            let artifact = file_artifact(&entry.path(), relative, metadata.permissions().mode())?;
            total_bytes = total_bytes
                .checked_add(artifact.size_bytes)
                .ok_or_else(|| Error::Bound("artifact-bytes-overflow".to_string()))?;
            if total_bytes > policy.max_tree_bytes {
                return Err(Error::Bound("artifact-tree-bytes-exceeded".to_string()));
            }
            artifacts.push(artifact);
            if artifacts.len() > max_entries {
                return Err(Error::Bound("artifact-tree-entry-count-exceeded".to_string()));
            }
        }
    }
    artifacts.sort();
    if artifacts.is_empty() {
        return Err(Error::State("artifact-manifest-empty".to_string()));
    }
    assert!(artifacts.len() <= max_entries);
    assert!(total_bytes <= policy.max_tree_bytes);
    Ok(artifacts)
}

fn sorted_directory_entries(directory: &Path) -> Result<Vec<fs::DirEntry>, Error> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| Error::Io {
            context: "read-artifact-directory".to_string(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| Error::Io {
            context: "collect-artifact-directory".to_string(),
            source,
        })?;
    entries.sort_by_key(fs::DirEntry::file_name);
    let max_entries = usize::try_from(crunch_rust_cache_core::MAX_TREE_ENTRIES)
        .map_err(|_| Error::Bound("artifact-tree-entry-limit-unrepresentable".to_string()))?;
    assert!(entries.len() <= max_entries);
    assert!(entries.windows(2).all(|pair| pair[0].file_name() <= pair[1].file_name()));
    Ok(entries)
}

fn file_artifact(path: &Path, relative_path: String, mode: u32) -> Result<RustResultArtifact, Error> {
    let (digest_blake3, size_bytes) = hash_file_bounded(path)?;
    let normalized_mode = if mode & EXECUTABLE_PERMISSION_BITS == 0 {
        REGULAR_FILE_MODE
    } else {
        EXECUTABLE_FILE_MODE
    };
    let artifact = RustResultArtifact {
        relative_path,
        kind: RustArtifactKind::File,
        mode: normalized_mode,
        size_bytes,
        digest_blake3,
    };
    assert!(!artifact.relative_path.is_empty());
    assert!(matches!(artifact.mode, REGULAR_FILE_MODE | EXECUTABLE_FILE_MODE));
    Ok(artifact)
}

fn hash_file_bounded(path: &Path) -> Result<(String, u64), Error> {
    let mut file = open_read_nofollow(path)?;
    let declared_bytes = file
        .metadata()
        .map_err(|source| Error::Io {
            context: "stat-artifact".to_string(),
            source,
        })?
        .len();
    if declared_bytes > MAX_SCAN_BYTES {
        return Err(Error::Bound("artifact-file-bytes-exceeded".to_string()));
    }
    let max_read_iterations = bounded_read_iterations(declared_bytes, READ_BUFFER_BYTES)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    let mut total_bytes = 0_u64;
    for _read_index in 0..max_read_iterations {
        let read_bytes = file.read(&mut buffer).map_err(|source| Error::Io {
            context: "read-artifact".to_string(),
            source,
        })?;
        if read_bytes == 0 {
            break;
        }
        let read_bytes_u64 =
            u64::try_from(read_bytes).map_err(|_| Error::Bound("artifact-read-size-unrepresentable".to_string()))?;
        total_bytes = total_bytes
            .checked_add(read_bytes_u64)
            .ok_or_else(|| Error::Bound("artifact-bytes-overflow".to_string()))?;
        if total_bytes > MAX_SCAN_BYTES {
            return Err(Error::Bound("artifact-file-bytes-exceeded".to_string()));
        }
        hasher.update(&buffer[..read_bytes]);
    }
    if total_bytes != declared_bytes {
        return Err(Error::State("artifact-size-changed-during-read".to_string()));
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert!(total_bytes <= MAX_SCAN_BYTES);
    Ok((digest, total_bytes))
}

fn bounded_read_iterations(byte_count: u64, buffer_bytes: usize) -> Result<u64, Error> {
    let buffer_bytes =
        u64::try_from(buffer_bytes).map_err(|_| Error::Bound("read-buffer-size-unrepresentable".to_string()))?;
    if buffer_bytes == 0 {
        return Err(Error::Bound("read-buffer-size-zero".to_string()));
    }
    let iterations = byte_count
        .checked_div(buffer_bytes)
        .and_then(|count| count.checked_add(EXTRA_EOF_READ_ITERATIONS))
        .ok_or_else(|| Error::Bound("read-iteration-limit-overflow".to_string()))?;
    assert!(iterations >= EXTRA_EOF_READ_ITERATIONS);
    assert!(buffer_bytes > 0);
    Ok(iterations)
}

fn copy_declared_artifact(
    source_root: &Path,
    destination_root: &Path,
    artifact: &RustResultArtifact,
) -> Result<(), Error> {
    let source = source_root.join(&artifact.relative_path);
    let destination = destination_root.join(&artifact.relative_path);
    let parent = destination
        .parent()
        .ok_or_else(|| Error::State("artifact-destination-parent-missing".to_string()))?;
    create_private_directory(parent)?;
    let mut input = open_read_nofollow(&source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&destination)
        .map_err(|source| Error::Io {
            context: "create-snapshot-artifact".to_string(),
            source,
        })?;
    let copied = std::io::copy(&mut input, &mut output).map_err(|source| Error::Io {
        context: "copy-snapshot-artifact".to_string(),
        source,
    })?;
    if copied != artifact.size_bytes {
        return Err(Error::State("snapshot-artifact-size-mismatch".to_string()));
    }
    output.sync_all().map_err(|source| Error::Io {
        context: "sync-snapshot-artifact".to_string(),
        source,
    })?;
    fs::set_permissions(&destination, fs::Permissions::from_mode(artifact.mode)).map_err(|source| Error::Io {
        context: "set-snapshot-artifact-mode".to_string(),
        source,
    })?;
    assert!(destination.starts_with(destination_root));
    assert_eq!(copied, artifact.size_bytes);
    Ok(())
}

fn node_identity(node: &Node) -> Result<CastoreNodeIdentity, Error> {
    let Node::Directory { digest, size } = node else {
        return Err(Error::State("rust-result-root-not-directory".to_string()));
    };
    let identity = CastoreNodeIdentity {
        kind: CastoreNodeKind::Directory,
        digest_blake3: digest_hex(digest),
        size_bytes: *size,
    };
    assert_eq!(identity.digest_blake3.len(), BLAKE3_HEX_CHARS);
    assert_eq!(identity.kind, CastoreNodeKind::Directory);
    Ok(identity)
}

fn node_from_identity(identity: &CastoreNodeIdentity) -> Result<Node, Error> {
    if identity.kind != CastoreNodeKind::Directory {
        return Err(Error::State("rust-result-root-not-directory".to_string()));
    }
    let digest = digest_from_hex(&identity.digest_blake3)?;
    let node = Node::Directory {
        digest,
        size: identity.size_bytes,
    };
    assert!(matches!(node, Node::Directory { .. }));
    assert_eq!(identity.digest_blake3.len(), BLAKE3_HEX_CHARS);
    Ok(node)
}

fn digest_hex(digest: &B3Digest) -> String {
    let hash = blake3::Hash::from_bytes(*digest.as_ref());
    let encoded = hash.to_hex().to_string();
    assert_eq!(encoded.len(), BLAKE3_HEX_CHARS);
    assert!(encoded.bytes().all(|byte| !byte.is_ascii_uppercase()));
    encoded
}

fn digest_from_hex(encoded: &str) -> Result<B3Digest, Error> {
    if encoded.len() != BLAKE3_HEX_CHARS {
        return Err(Error::State("castore-digest-length-invalid".to_string()));
    }
    let mut bytes = [0_u8; B3Digest::LENGTH];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let offset_bytes = index
            .checked_mul(HEX_BYTE_CHARS)
            .ok_or_else(|| Error::State("castore-digest-offset-overflow".to_string()))?;
        let end_offset_bytes = offset_bytes
            .checked_add(HEX_BYTE_CHARS)
            .ok_or_else(|| Error::State("castore-digest-offset-overflow".to_string()))?;
        *byte = u8::from_str_radix(&encoded[offset_bytes..end_offset_bytes], HEX_RADIX)
            .map_err(|_| Error::State("castore-digest-hex-invalid".to_string()))?;
    }
    let digest = B3Digest::from(&bytes);
    assert_eq!(digest_hex(&digest), encoded);
    assert_eq!(bytes.len(), B3Digest::LENGTH);
    Ok(digest)
}

fn typed_ref_path(directory: &Path, reference: &str, prefix: TypedRefPrefix<'_>) -> Result<PathBuf, Error> {
    let digest = reference
        .strip_prefix(prefix.0)
        .ok_or_else(|| Error::State("typed-reference-prefix-invalid".to_string()))?;
    if digest.len() != BLAKE3_HEX_CHARS
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(Error::State("typed-reference-digest-invalid".to_string()));
    }
    let path = directory.join(format!("{digest}.json"));
    assert!(path.starts_with(directory));
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    Ok(path)
}

fn collect_stale_result_paths(
    results_dir: &Path,
    live_result_paths: &BTreeSet<PathBuf>,
) -> Result<Vec<PathBuf>, Error> {
    let entries = fs::read_dir(results_dir).map_err(|source| Error::Io {
        context: "read-result-directory".to_string(),
        source,
    })?;
    let mut stale_paths = Vec::with_capacity(crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
    let mut entry_count = 0_usize;
    for entry in entries {
        entry_count = entry_count
            .checked_add(1)
            .ok_or_else(|| Error::Bound("result-directory-count-overflow".to_string()))?;
        if entry_count > crunch_rust_cache_core::MAX_RESULT_CANDIDATES {
            return Err(Error::Bound("result-directory-limit-exceeded".to_string()));
        }
        let entry = entry.map_err(|source| Error::Io {
            context: "read-result-entry".to_string(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| Error::Io {
            context: "inspect-result-entry".to_string(),
            source,
        })?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(Error::State("result-directory-entry-not-regular".to_string()));
        }
        let path = entry.path();
        if !live_result_paths.contains(&path) {
            stale_paths.push(path);
        }
    }
    stale_paths.sort();
    assert!(entry_count <= crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
    assert!(stale_paths.iter().all(|path| path.starts_with(results_dir)));
    Ok(stale_paths)
}

fn collect_stale_index_updates(
    indexes_dir: &Path,
    retained_result_actions: &BTreeMap<String, String>,
) -> Result<Vec<(PathBuf, Option<RustResultIndex>)>, Error> {
    let entries = fs::read_dir(indexes_dir).map_err(|source| Error::Io {
        context: "read-index-directory".to_string(),
        source,
    })?;
    let mut updates = Vec::with_capacity(crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
    let mut entry_count = 0_usize;
    for entry in entries {
        entry_count = entry_count
            .checked_add(1)
            .ok_or_else(|| Error::Bound("index-directory-count-overflow".to_string()))?;
        if entry_count > crunch_rust_cache_core::MAX_RESULT_CANDIDATES {
            return Err(Error::Bound("index-directory-limit-exceeded".to_string()));
        }
        let entry = entry.map_err(|source| Error::Io {
            context: "read-index-entry".to_string(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| Error::Io {
            context: "inspect-index-entry".to_string(),
            source,
        })?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(Error::State("index-directory-entry-not-regular".to_string()));
        }
        let path = entry.path();
        let bytes = read_bounded_required(&path)?;
        let index = serde_json::from_slice::<RustResultIndex>(&bytes)
            .map_err(|error| Error::Json(format!("index-decode:{error}")))?;
        validate_result_index(&index).map_err(|error| Error::Core(String::from(error.code())))?;
        let expected_path = typed_ref_path(
            indexes_dir,
            &index.action_ref,
            TypedRefPrefix(crunch_rust_cache_core::RUST_ACTION_REF_PREFIX),
        )?;
        if path != expected_path {
            return Err(Error::State("index-reference-path-mismatch".to_string()));
        }
        let retained_refs = index
            .result_refs
            .iter()
            .filter(|result_ref| {
                retained_result_actions.get(*result_ref).is_some_and(|action_ref| action_ref == &index.action_ref)
            })
            .cloned()
            .collect::<Vec<_>>();
        if retained_refs == index.result_refs {
            continue;
        }
        let replacement = if retained_refs.is_empty() {
            None
        } else {
            Some(
                canonical_result_index(index.action_ref, retained_refs)
                    .map_err(|error| Error::Core(String::from(error.code())))?,
            )
        };
        updates.push((path, replacement));
    }
    updates.sort_by(|left, right| left.0.cmp(&right.0));
    assert!(entry_count <= crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
    assert!(updates.iter().all(|(path, _)| path.starts_with(indexes_dir)));
    Ok(updates)
}

fn validate_retention(retention: &RustCacheRetention) -> Result<(), Error> {
    if retention.schema != RUST_CACHE_RETENTION_SCHEMA {
        return Err(Error::State("retention-schema-unsupported".to_string()));
    }
    if retention.retained_results.len() > crunch_rust_cache_core::MAX_RESULT_CANDIDATES {
        return Err(Error::Bound("retention-result-limit-exceeded".to_string()));
    }
    for (result_ref, node) in &retention.retained_results {
        typed_ref_path(
            Path::new("retained"),
            result_ref,
            TypedRefPrefix(crunch_rust_cache_core::RUST_RESULT_REF_PREFIX),
        )?;
        node_from_identity(node)?;
    }
    assert_eq!(retention.schema, RUST_CACHE_RETENTION_SCHEMA);
    assert!(retention.retained_results.len() <= crunch_rust_cache_core::MAX_RESULT_CANDIDATES);
    Ok(())
}

fn read_bounded_required(path: &Path) -> Result<Vec<u8>, Error> {
    read_bounded_optional(path)?.ok_or_else(|| Error::State(format!("required-cache-file-missing:{}", path.display())))
}

fn read_bounded_optional(path: &Path) -> Result<Option<Vec<u8>>, Error> {
    let mut file = match open_read_nofollow(path) {
        Ok(file) => file,
        Err(Error::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let metadata = file.metadata().map_err(|source| Error::Io {
        context: "stat-cache-file".to_string(),
        source,
    })?;
    if metadata.len() > MAX_CACHE_JSON_BYTES {
        return Err(Error::Bound("cache-json-too-large".to_string()));
    }
    let capacity_bytes =
        usize::try_from(metadata.len()).map_err(|_| Error::Bound("cache-json-size-unrepresentable".to_string()))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    file.read_to_end(&mut bytes).map_err(|source| Error::Io {
        context: "read-cache-file".to_string(),
        source,
    })?;
    if bytes.len() > capacity_bytes {
        return Err(Error::State("cache-json-grew-during-read".to_string()));
    }
    assert!(bytes.len() <= capacity_bytes);
    assert!(bytes.len() as u64 <= MAX_CACHE_JSON_BYTES);
    Ok(Some(bytes))
}

fn open_read_nofollow(path: &Path) -> Result<File, Error> {
    OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path).map_err(|source| Error::Io {
        context: format!("open-nofollow:{}", path.display()),
        source,
    })
}

fn write_immutable_json(path: &Path, value: &impl Serialize) -> Result<(), Error> {
    let bytes = bounded_json(value)?;
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
    {
        Ok(mut file) => {
            file.write_all(&bytes).map_err(|source| Error::Io {
                context: "write-immutable-json".to_string(),
                source,
            })?;
            file.sync_all().map_err(|source| Error::Io {
                context: "sync-immutable-json".to_string(),
                source,
            })?;
            sync_directory(path.parent().ok_or_else(|| Error::State("immutable-json-parent-missing".to_string()))?)?;
        }
        Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = read_bounded_required(path)?;
            if existing != bytes {
                return Err(Error::State("immutable-result-conflict".to_string()));
            }
        }
        Err(source) => {
            return Err(Error::Io {
                context: "create-immutable-json".to_string(),
                source,
            });
        }
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn write_atomic_json(path: &Path, value: &impl Serialize) -> Result<(), Error> {
    let bytes = bounded_json(value)?;
    let parent = path.parent().ok_or_else(|| Error::State("atomic-json-parent-missing".to_string()))?;
    let mut temp = Builder::new().prefix("state-").tempfile_in(parent).map_err(|source| Error::Io {
        context: "create-atomic-json".to_string(),
        source,
    })?;
    temp.as_file_mut().write_all(&bytes).map_err(|source| Error::Io {
        context: "write-atomic-json".to_string(),
        source,
    })?;
    temp.as_file_mut().sync_all().map_err(|source| Error::Io {
        context: "sync-atomic-json".to_string(),
        source,
    })?;
    temp.persist(path).map_err(|error| Error::Io {
        context: "persist-atomic-json".to_string(),
        source: error.error,
    })?;
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_FILE_MODE)).map_err(|source| Error::Io {
        context: "set-atomic-json-mode".to_string(),
        source,
    })?;
    sync_directory(parent)?;
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn bounded_json(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    let bytes = serde_json::to_vec(value).map_err(|error| Error::Json(format!("encode:{error}")))?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_CACHE_JSON_BYTES {
        return Err(Error::Bound("cache-json-size-invalid".to_string()));
    }
    assert!(!bytes.is_empty());
    assert!(bytes.len() as u64 <= MAX_CACHE_JSON_BYTES);
    Ok(bytes)
}

fn create_private_directory(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path).map_err(|source| Error::Io {
        context: format!("create-directory:{}", path.display()),
        source,
    })?;
    let metadata = fs::symlink_metadata(path).map_err(|source| Error::Io {
        context: format!("stat-directory:{}", path.display()),
        source,
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(Error::State(format!("cache-directory-invalid:{}", path.display())));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).map_err(|source| Error::Io {
        context: format!("set-directory-mode:{}", path.display()),
        source,
    })?;
    assert!(path.is_dir());
    assert!(!metadata.file_type().is_symlink());
    Ok(())
}

fn make_owner_writable(path: &Path) {
    let is_path_present = path.exists();
    if let Ok(metadata) = fs::symlink_metadata(path) {
        let mode = metadata.permissions().mode() | PRIVATE_DIRECTORY_MODE;
        let _result = fs::set_permissions(path, fs::Permissions::from_mode(mode));
    }
    debug_assert!(!path.as_os_str().is_empty());
    if is_path_present {
        debug_assert!(path.exists());
    }
}

fn cleanup_restore_staging(root: &Path) {
    let mut pending = vec![root.to_path_buf()];
    let mut visited = 0_u32;
    while let Some(directory) = pending.pop() {
        visited = visited.saturating_add(1);
        if visited > MAX_SCAN_ENTRIES {
            break;
        }
        make_owner_writable(&directory);
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() && !file_type.is_symlink() {
                pending.push(entry.path());
            }
        }
    }
    let _cleanup = fs::remove_dir_all(root);
    debug_assert!(visited <= MAX_SCAN_ENTRIES.saturating_add(1));
    debug_assert!(!root.as_os_str().is_empty());
}

fn path_mode(path: &Path) -> u32 {
    let mode = fs::symlink_metadata(path).map(|metadata| metadata.permissions().mode()).unwrap_or(0);
    assert!(!path.as_os_str().is_empty());
    if mode != 0 {
        assert!(path.exists());
    }
    mode
}

fn sync_directory(path: &Path) -> Result<(), Error> {
    let directory = open_read_nofollow(path)?;
    directory.sync_all().map_err(|source| Error::Io {
        context: "sync-directory".to_string(),
        source,
    })?;
    assert!(path.is_dir());
    assert!(directory.metadata().map(|metadata| metadata.is_dir()).unwrap_or(false));
    Ok(())
}

fn artifact_bytes(artifacts: &[RustResultArtifact]) -> Result<u64, Error> {
    let total = artifacts.iter().try_fold(0_u64, |total, artifact| {
        total
            .checked_add(artifact.size_bytes)
            .ok_or_else(|| Error::Bound("artifact-byte-total-overflow".to_string()))
    })?;
    assert!(!artifacts.is_empty());
    assert!(total <= MAX_SCAN_BYTES);
    Ok(total)
}

fn bounded_count(count: usize) -> Result<u32, Error> {
    let original = count;
    let count = u32::try_from(count).map_err(|_| Error::Bound("count-unrepresentable".to_string()))?;
    let round_trip =
        usize::try_from(count).map_err(|_| Error::Bound("count-round-trip-unrepresentable".to_string()))?;
    assert!(count <= MAX_SCAN_ENTRIES);
    assert_eq!(round_trip, original);
    Ok(count)
}

fn miss_report() -> RustCacheReport {
    let outcome = RustCacheReport {
        disposition: CACHE_DISPOSITION_MISS.to_string(),
        reason_codes: vec![CACHE_DISPOSITION_MISS.to_string()],
        selected_result_ref: None,
        candidate_count: 0,
        artifact_count: 0,
        restored_bytes: 0,
        reused_bytes: 0,
        compiler_executed: false,
    };
    assert_eq!(outcome.disposition, CACHE_DISPOSITION_MISS);
    assert!(outcome.selected_result_ref.is_none());
    outcome
}

fn rejected_report(reason: &str) -> RustCacheReport {
    let outcome = RustCacheReport {
        disposition: CACHE_DISPOSITION_REJECTED.to_string(),
        reason_codes: vec![reason.to_string()],
        selected_result_ref: None,
        candidate_count: 0,
        artifact_count: 0,
        restored_bytes: 0,
        reused_bytes: 0,
        compiler_executed: false,
    };
    assert!(!reason.is_empty());
    assert_eq!(outcome.disposition, CACHE_DISPOSITION_REJECTED);
    outcome
}

fn rejected_plan_report(
    candidate_count: u32,
    decisions: &[crunch_rust_cache_core::LocalCandidateDecision],
) -> RustCacheReport {
    let mut reasons = decisions.iter().flat_map(|decision| decision.reason_codes.clone()).collect::<Vec<_>>();
    reasons.sort();
    reasons.dedup();
    if reasons.is_empty() {
        reasons.push(CACHE_DISPOSITION_MISS.to_string());
    }
    let outcome = RustCacheReport {
        disposition: CACHE_DISPOSITION_REJECTED.to_string(),
        reason_codes: reasons,
        selected_result_ref: None,
        candidate_count,
        artifact_count: 0,
        restored_bytes: 0,
        reused_bytes: 0,
        compiler_executed: false,
    };
    assert!(!outcome.reason_codes.is_empty());
    assert!(outcome.selected_result_ref.is_none());
    outcome
}

struct RestoreStaging {
    temp: Option<tempfile::TempDir>,
}

impl RestoreStaging {
    fn new(temp: tempfile::TempDir) -> Self {
        assert!(temp.path().is_dir());
        assert!(!temp.path().as_os_str().is_empty());
        Self { temp: Some(temp) }
    }

    fn path(&self) -> Result<&Path, Error> {
        let temp = self.temp.as_ref().ok_or_else(|| Error::State("restore-staging-not-owned".to_string()))?;
        let path = temp.path();
        assert!(path.is_dir());
        assert!(!path.as_os_str().is_empty());
        Ok(path)
    }

    fn keep(mut self) -> Result<PathBuf, Error> {
        let temp = self.temp.take().ok_or_else(|| Error::State("restore-staging-not-owned".to_string()))?;
        let path = temp.keep();
        assert!(path.is_dir());
        assert!(!path.as_os_str().is_empty());
        Ok(path)
    }
}

impl Drop for RestoreStaging {
    fn drop(&mut self) {
        let Some(temp) = self.temp.take() else {
            return;
        };
        let path = temp.keep();
        cleanup_restore_staging(&path);
        debug_assert!(!path.exists(), "restore staging must be removed after failure");
    }
}

struct CacheMutationLock {
    file: File,
}

impl CacheMutationLock {
    #[allow(tigerstyle::ambient_clock, reason = "the lock shell owns bounded wall-clock waiting")]
    fn acquire(path: &Path) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(PRIVATE_FILE_MODE)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(|source| Error::Io {
                context: "open-cache-mutation-lock".to_string(),
                source,
            })?;
        let deadline = Instant::now()
            .checked_add(LOCK_TIMEOUT)
            .ok_or_else(|| Error::State("lock-deadline-overflow".to_string()))?;
        for _attempt in 0..LOCK_MAX_ATTEMPTS {
            match file.try_lock_exclusive() {
                Ok(()) => {
                    assert!(path.is_file());
                    assert!(file.metadata().map(|metadata| metadata.is_file()).unwrap_or(false));
                    return Ok(Self { file });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(Error::LockTimeout);
                    }
                    std::thread::sleep(LOCK_POLL_INTERVAL);
                }
                Err(source) => {
                    return Err(Error::Io {
                        context: "acquire-cache-mutation-lock".to_string(),
                        source,
                    });
                }
            }
        }
        Err(Error::LockTimeout)
    }
}

impl Drop for CacheMutationLock {
    fn drop(&mut self) {
        let result = FileExt::unlock(&self.file);
        debug_assert!(result.is_ok(), "cache mutation lock must unlock explicitly");
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use crunch_rust_cache_core::RustBuildFact;
    use crunch_rust_cache_core::RustSemanticArgument;
    use crunch_rust_cache_core::RustUnitActionInput;
    use crunch_rust_cache_core::canonical_rust_action;
    use pretty_assertions::assert_eq;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TEST_OUTPUT: &[u8] = b"local rust cache artifact";
    const TEST_CONFLICT_CANDIDATES: u32 = 2;
    const TEST_DEVICE_A: u64 = 10;
    const TEST_DEVICE_B: u64 = 11;

    async fn test_cache(state_dir: &Path) -> RustCache {
        let directories = RedbDirectoryService::new("rust-cache-test".to_string(), RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        })
        .await
        .unwrap();
        RustCache::new(state_dir.to_path_buf(), Arc::new(MemoryBlobService::default()), Arc::new(directories)).unwrap()
    }

    fn test_action() -> RustUnitAction {
        canonical_rust_action(RustUnitActionInput {
            unit_id: "unit".to_string(),
            package_id: "package".to_string(),
            crate_name: "crate".to_string(),
            target_kind: "lib".to_string(),
            execution_kind: "target".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            profile: "debug".to_string(),
            mode: "build".to_string(),
            features: Vec::new(),
            source_digest_blake3: DIGEST.to_string(),
            compiler_digest_blake3: DIGEST.to_string(),
            compiler_version_digest_blake3: DIGEST.to_string(),
            toolchain_closure_digest_blake3: DIGEST.to_string(),
            execution_platform_digest_blake3: DIGEST.to_string(),
            semantic_arguments: vec![RustSemanticArgument {
                value: "--crate-type=lib".to_string(),
                contains_absolute_path: false,
                absolute_paths_classified: false,
            }],
            admitted_environment: BTreeMap::new(),
            dependency_artifacts: Vec::new(),
            host_artifacts: Vec::new(),
            build_script_facts: vec![RustBuildFact {
                name: "none".to_string(),
                value_digest_blake3: DIGEST.to_string(),
            }],
            native_link_facts: Vec::new(),
            compiler_policy_digest_blake3: DIGEST.to_string(),
        })
        .unwrap()
    }

    fn read_write_policy() -> LocalCachePolicy {
        LocalCachePolicy {
            reads_enabled: true,
            writes_enabled: true,
            ..LocalCachePolicy::default()
        }
    }

    fn write_output(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path).unwrap();
        fs::write(path.join("libcrate.rlib"), bytes).unwrap();
        fs::write(path.join(RUST_UNIT_EXECUTION_RECEIPT_FILE), b"mutable receipt").unwrap();
    }

    #[tokio::test]
    async fn publish_then_restore_excludes_mutable_receipt() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let output = root.path().join("execution/unit");
        write_output(&output, TEST_OUTPUT);
        let action = test_action();
        let policy = read_write_policy();
        let result = cache
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &policy,
            })
            .await
            .unwrap();
        fs::remove_dir_all(&output).unwrap();

        let report = cache.restore(&action, &output, &policy).await.unwrap();

        assert_eq!(report.disposition, CACHE_DISPOSITION_HIT);
        assert_eq!(fs::read(output.join("libcrate.rlib")).unwrap(), TEST_OUTPUT);
        assert!(!output.join(RUST_UNIT_EXECUTION_RECEIPT_FILE).exists());
        assert_eq!(cache.retention().unwrap().retained_results.get(&result.result_ref), Some(&result.input.root_node));
    }

    #[tokio::test]
    async fn retention_plan_keeps_accepted_roots_and_prunes_only_stale_records() {
        let state = tempfile::tempdir().unwrap();
        let cache = test_cache(state.path()).await;
        let source = state.path().join("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("libcrate.rlib"), TEST_OUTPUT).unwrap();
        let action = test_action();
        let result = cache
            .publish(PublishRequest {
                action: &action,
                output_dir: &source,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &read_write_policy(),
            })
            .await
            .unwrap();
        let mut stale_input = result.input.clone();
        stale_input.producer_receipt_ref = format!("mantle-rust-receipt://blake3/{OTHER_DIGEST}");
        let stale_result = canonical_rust_result(stale_input).unwrap();
        let stale_path = cache.result_path(&stale_result.result_ref).unwrap();
        write_immutable_json(&stale_path, &stale_result).unwrap();
        let index = canonical_result_index(action.action_ref.clone(), vec![
            result.result_ref.clone(),
            stale_result.result_ref.clone(),
        ])
        .unwrap();
        write_atomic_json(&cache.index_path(&action.action_ref).unwrap(), &index).unwrap();

        let plan = cache.plan_retention_gc().unwrap();

        assert_eq!(plan.live_nodes().len(), 1);
        assert_eq!(plan.stale_result_count(), 1);
        plan.apply(true).unwrap();
        assert!(stale_path.is_file());
        plan.apply(false).unwrap();
        assert!(!stale_path.exists());
        assert!(cache.result_path(&result.result_ref).unwrap().is_file());
        assert_eq!(cache.read_index(&action.action_ref).unwrap().unwrap().result_refs, vec![result.result_ref]);
    }

    #[tokio::test]
    async fn incomplete_castore_candidate_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let output = root.path().join("execution/unit");
        write_output(&output, TEST_OUTPUT);
        let action = test_action();
        let policy = read_write_policy();
        cache
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &policy,
            })
            .await
            .unwrap();
        fs::remove_dir_all(&output).unwrap();
        let empty_cache = test_cache(root.path()).await;

        let report = empty_cache.restore(&action, &output, &policy).await.unwrap();

        assert_eq!(report.disposition, CACHE_DISPOSITION_REJECTED);
        assert!(report.reason_codes.contains(&"rust-local-cache-content-incomplete".to_string()));
        assert!(!output.exists());
    }

    #[tokio::test]
    async fn different_complete_results_report_conflict() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let output = root.path().join("execution/unit");
        let action = test_action();
        let policy = read_write_policy();
        write_output(&output, b"first");
        cache
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &policy,
            })
            .await
            .unwrap();
        fs::remove_dir_all(&output).unwrap();
        write_output(&output, b"second");
        cache
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &policy,
            })
            .await
            .unwrap();
        fs::remove_dir_all(&output).unwrap();

        let report = cache.restore(&action, &output, &policy).await.unwrap();

        assert_eq!(report.disposition, CACHE_DISPOSITION_CONFLICT);
        assert_eq!(report.candidate_count, TEST_CONFLICT_CANDIDATES);
        assert!(!output.exists());
    }

    #[tokio::test]
    async fn publish_rejects_symlink_artifact() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let output = root.path().join("execution/unit");
        fs::create_dir_all(&output).unwrap();
        symlink("target", output.join("artifact")).unwrap();
        let action = test_action();
        let policy = read_write_policy();

        let error = cache
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &policy,
            })
            .await
            .unwrap_err();

        assert_eq!(error.to_string(), "rust-cache-state:symlink-artifact-unsupported");
        assert!(cache.retention().unwrap().retained_results.is_empty());
    }

    #[tokio::test]
    async fn artifact_entry_bound_rejects_publication_without_result_state() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let output = root.path().join("execution/unit");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("one.rlib"), b"one").unwrap();
        fs::write(output.join("two.rlib"), b"two").unwrap();
        let mut policy = read_write_policy();
        policy.max_tree_entries = 1;

        let error = cache
            .publish(PublishRequest {
                action: &test_action(),
                output_dir: &output,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &policy,
            })
            .await
            .unwrap_err();

        assert_eq!(error.to_string(), "rust-cache-bound:artifact-tree-entry-count-exceeded");
        assert!(cache.retention().unwrap().retained_results.is_empty());
    }

    #[tokio::test]
    async fn artifact_mismatch_cleans_interrupted_restore_staging() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let output = root.path().join("execution/unit");
        write_output(&output, TEST_OUTPUT);
        let action = test_action();
        let policy = read_write_policy();
        let original = cache
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
                policy: &policy,
            })
            .await
            .unwrap();
        fs::remove_dir_all(&output).unwrap();
        fs::remove_file(cache.result_path(&original.result_ref).unwrap()).unwrap();
        fs::remove_file(cache.index_path(&action.action_ref).unwrap()).unwrap();
        write_atomic_json(&cache.cache_dir.join(RETENTION_FILE), &RustCacheRetention::default()).unwrap();
        let mut tampered_input = original.input.clone();
        tampered_input.artifacts[0].digest_blake3 = "b".repeat(BLAKE3_HEX_CHARS);
        let tampered = canonical_rust_result(tampered_input).unwrap();
        cache.publish_record_and_index(&tampered).unwrap();

        let error = cache.restore(&action, &output, &policy).await.unwrap_err();

        assert_eq!(error.to_string(), "rust-cache-state:restored-artifact-manifest-mismatch");
        assert!(!output.exists());
        let parent = output.parent().unwrap();
        assert!(
            fs::read_dir(parent).unwrap().all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("restore-"))
        );
    }

    #[test]
    fn cross_filesystem_commit_facts_fail_closed() {
        assert!(
            validate_commit_devices(CommitDevices {
                staging: TEST_DEVICE_A,
                destination: TEST_DEVICE_A,
            })
            .is_ok()
        );
        let error = validate_commit_devices(CommitDevices {
            staging: TEST_DEVICE_A,
            destination: TEST_DEVICE_B,
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "rust-cache-state:cross-filesystem-restore-commit-rejected");
        assert_ne!(TEST_DEVICE_A, TEST_DEVICE_B);
    }

    #[tokio::test]
    async fn index_symlink_is_rejected_without_following() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let action = test_action();
        let target = root.path().join("outside.json");
        fs::write(&target, b"{}").unwrap();
        symlink(&target, cache.index_path(&action.action_ref).unwrap()).unwrap();
        let policy = read_write_policy();

        let error = cache.restore(&action, &root.path().join("output"), &policy).await.unwrap_err();

        assert!(error.to_string().contains("open-nofollow"));
        assert_eq!(fs::read(&target).unwrap(), b"{}");
    }

    #[tokio::test]
    async fn existing_output_blocks_restore_without_mutation() {
        let root = tempfile::tempdir().unwrap();
        let cache = test_cache(root.path()).await;
        let output = root.path().join("execution/unit");
        write_output(&output, TEST_OUTPUT);
        let action = test_action();
        let policy = read_write_policy();

        let report = cache.restore(&action, &output, &policy).await.unwrap();

        assert_eq!(report.disposition, CACHE_DISPOSITION_REJECTED);
        assert_eq!(report.reason_codes, vec!["rust-local-cache-output-exists".to_string()]);
        assert_eq!(fs::read(output.join("libcrate.rlib")).unwrap(), TEST_OUTPUT);
    }
}
