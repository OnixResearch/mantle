//! Imperative shell for retained tool workspaces.
//!
//! Policy decisions live in `workspace`; this module owns bounded filesystem
//! observation, no-follow mutation, locking, persistence, and dispatch.

use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use snix_build::buildservice::StatefulWorkspaceMode;
use snix_build::buildservice::StatefulWorkspaceRequest;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::import::fs::ingest_path;

use crate::workspace::*;

pub const WORKSPACE_EXECUTION_REPORT_SCHEMA: &str = "mantle-stateful-workspace-execution-v1";
pub const WORKSPACE_STATE_DIR_NAME: &str = "stateful-workspaces";
const CONTENT_DIR: &str = "content";
const LOCK_DIR: &str = "locks";
const RECORD_DIR: &str = "records";
const QUARANTINE_DIR: &str = "quarantine";
const SNAPSHOT_DIR: &str = "snapshots";
const FILE_MODE: u32 = 0o600;
const DIR_MODE: u32 = 0o700;
const RECORD_BYTES_MAX: u64 = 1_048_576;
const BLAKE3_DIGEST_HEX_LENGTH: usize = 64;
const WORKSPACE_REMOVAL_STACK_INITIAL_ENTRIES: usize = 64;
const HOST_PATH_MARKERS: [&[u8]; 2] = [b"/home/", b"/tmp/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceCleanupDisposition {
    NotRequired,
    Released,
    Quarantined,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceExecutionReport {
    pub schema: String,
    pub mode: WorkspaceMode,
    pub workspace_id: Option<String>,
    pub guest_path: String,
    pub compatibility_digest_blake3: String,
    pub warm_state_used: bool,
    pub claim_class: WorkspaceClaimClass,
    pub shared_action_publish_allowed: bool,
    pub strong_shared_reuse_allowed: bool,
    pub original_execution_hermetic: bool,
    pub clean_comparison_performed: bool,
    pub clean_comparison_matched: bool,
    pub warm_output_set_digest_blake3: Option<String>,
    pub clean_output_set_digest_blake3: Option<String>,
    pub cleanup: WorkspaceCleanupDisposition,
    pub cleanup_reason: WorkspaceReasonCode,
    pub snapshot_ref: Option<String>,
}

#[derive(Clone, Default)]
pub struct WorkspaceReportCollector(Arc<Mutex<Vec<WorkspaceExecutionReport>>>);

impl WorkspaceReportCollector {
    pub fn take(&self) -> Vec<WorkspaceExecutionReport> {
        let mut workspace_evidence = match self.0.lock() {
            Ok(workspace_evidence) => workspace_evidence,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *workspace_evidence)
    }

    fn push(&self, execution_report: WorkspaceExecutionReport) {
        let mut workspace_evidence = match self.0.lock() {
            Ok(workspace_evidence) => workspace_evidence,
            Err(poisoned) => poisoned.into_inner(),
        };
        workspace_evidence.push(execution_report);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceShellError {
    #[error("workspace policy rejected: {0}")]
    Policy(&'static str),
    #[error("workspace lease rejected: {0}")]
    Lease(&'static str),
    #[error("workspace already has an exclusive owner")]
    ConcurrentLease,
    #[error("workspace path rejected: {0}")]
    Path(String),
    #[error("workspace quarantined: {0}")]
    Quarantine(&'static str),
    #[error("workspace I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("workspace state record invalid: {0}")]
    Record(String),
}

#[derive(Clone)]
pub struct WorkspaceStore {
    root: PathBuf,
}

impl WorkspaceStore {
    pub fn new(state_dir: impl AsRef<Path>) -> Self {
        Self {
            root: state_dir.as_ref().join(WORKSPACE_STATE_DIR_NAME),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load_record(&self, workspace_id: &str) -> Result<Option<WorkspaceLeaseRecord>, WorkspaceShellError> {
        let path = self.record_path(workspace_id)?;
        if !path.try_exists()? {
            return Ok(None);
        }
        ensure_regular_no_follow(&path)?;
        let bytes = read_no_follow(&path, RECORD_BYTES_MAX)?;
        let record: WorkspaceLeaseRecord =
            serde_json::from_slice(&bytes).map_err(|error| WorkspaceShellError::Record(error.to_string()))?;
        if record.workspace_id != workspace_id {
            return Err(WorkspaceShellError::Record("record id does not match filename".into()));
        }
        Ok(Some(record))
    }

    pub fn prepare(&self, request: &StatefulWorkspaceRequest) -> Result<ActiveWorkspace, WorkspaceShellError> {
        if request.mode != StatefulWorkspaceMode::MutableSession {
            return Err(WorkspaceShellError::Policy("mutable-session-required"));
        }
        let workspace_id =
            request.workspace_id.as_deref().ok_or(WorkspaceShellError::Policy("workspace-id-missing"))?;
        self.ensure_layout()?;
        self.reconcile_retention(request)?;
        let lock = self.acquire_lock(workspace_id)?;
        let current = self.load_record(workspace_id)?;
        let acquire = lease_request(request, WorkspaceLeaseOperation::Acquire)?;
        let lease_plan = plan_workspace_lease(current.as_ref(), &acquire);
        if lease_plan.disposition != WorkspaceLeaseDisposition::Accepted {
            return Err(WorkspaceShellError::Lease(lease_plan.reason.as_str()));
        }
        let content_path = self.content_path(workspace_id)?;
        create_private_dir(&content_path)?;
        ensure_contained_no_symlinks(&self.root, &content_path)?;
        let is_warm_state_used = directory_has_entries(&content_path)?;
        let plan = scrub_and_plan(&content_path, request)?;
        let mut record = lease_plan.next.ok_or(WorkspaceShellError::Lease("accepted-lease-missing-record"))?;
        if plan.quarantine_required {
            self.quarantine_content(workspace_id, request.generation)?;
            record = quarantined_record(record, request.generation);
            self.save_record(&record)?;
            return Err(WorkspaceShellError::Quarantine(first_reason(&plan).as_str()));
        }
        self.save_record(&record)?;
        Ok(ActiveWorkspace {
            store: self.clone(),
            request: request.clone(),
            content_path,
            warm_state_used: is_warm_state_used,
            record,
            lock,
            finished: false,
        })
    }

    pub fn apply_retention(&self, plan: &WorkspaceRetentionPlan) -> Result<(), WorkspaceShellError> {
        let count = plan
            .evict
            .len()
            .checked_add(plan.quarantine_evict.len())
            .ok_or_else(|| WorkspaceShellError::Record("retention count overflow".into()))?;
        if count > MAX_WORKSPACE_RETENTION_RECORDS {
            return Err(WorkspaceShellError::Record("retention delete bound exceeded".into()));
        }
        for id in &plan.evict {
            remove_tree_no_follow_if_exists(&self.content_path(id)?)?;
            remove_file_no_follow_if_exists(&self.record_path(id)?)?;
            remove_prefixed_entries(&self.root.join(SNAPSHOT_DIR), id)?;
        }
        for id in &plan.quarantine_evict {
            remove_prefixed_entries(&self.root.join(QUARANTINE_DIR), id)?;
            remove_file_no_follow_if_exists(&self.record_path(id)?)?;
        }
        Ok(())
    }

    fn reconcile_retention(&self, request: &StatefulWorkspaceRequest) -> Result<(), WorkspaceShellError> {
        let mut entries = std::fs::read_dir(self.root.join(RECORD_DIR))?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        if entries.len() > MAX_WORKSPACE_RETENTION_RECORDS {
            return Err(WorkspaceShellError::Record("workspace record bound exceeded".into()));
        }
        let mut records = Vec::with_capacity(entries.len());
        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let id = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| WorkspaceShellError::Record("workspace record filename is not UTF-8".into()))?;
            let record = self
                .load_record(id)?
                .ok_or_else(|| WorkspaceShellError::Record("workspace record disappeared".into()))?;
            let state = match record.state {
                WorkspaceLeaseState::Active => WorkspaceRetentionState::Active,
                WorkspaceLeaseState::Idle => WorkspaceRetentionState::Idle,
                WorkspaceLeaseState::Quarantined => WorkspaceRetentionState::Quarantined,
            };
            records.push(WorkspaceRetentionRecord {
                workspace_id: record.workspace_id,
                state,
                creation_generation: record.creation_generation,
                last_used_generation: record.last_used_generation,
                size_bytes: 0,
            });
        }
        let policy = WorkspaceRetentionPolicy {
            workspace_count_max: request.retention_workspace_count_max,
            idle_generations_max: request.retention_idle_generations_max,
            age_generations_max: request.retention_age_generations_max,
            quarantine_count_max: request.retention_quarantine_count_max,
        };
        let plan = plan_workspace_retention(&records, request.generation, &policy)
            .map_err(|reason| WorkspaceShellError::Policy(reason.as_str()))?;
        self.apply_retention(&plan)
    }

    fn ensure_layout(&self) -> Result<(), WorkspaceShellError> {
        create_private_dir(&self.root)?;
        for child in [CONTENT_DIR, LOCK_DIR, RECORD_DIR, QUARANTINE_DIR, SNAPSHOT_DIR] {
            create_private_dir(&self.root.join(child))?;
        }
        ensure_path_chain_no_symlinks(&self.root)
    }

    fn acquire_lock(&self, id: &str) -> Result<File, WorkspaceShellError> {
        let path = safe_file(&self.root.join(LOCK_DIR), id, "lock")?;
        let file = private_open_create(&path)?;
        file.try_lock_exclusive().map_err(|error| {
            if error.kind() == std::io::ErrorKind::WouldBlock {
                WorkspaceShellError::ConcurrentLease
            } else {
                WorkspaceShellError::Io(error)
            }
        })?;
        Ok(file)
    }

    fn content_path(&self, id: &str) -> Result<PathBuf, WorkspaceShellError> {
        safe_child(&self.root.join(CONTENT_DIR), id)
    }

    fn record_path(&self, id: &str) -> Result<PathBuf, WorkspaceShellError> {
        safe_file(&self.root.join(RECORD_DIR), id, "json")
    }

    fn save_record(&self, record: &WorkspaceLeaseRecord) -> Result<(), WorkspaceShellError> {
        let bytes = serde_json::to_vec(record).map_err(|error| WorkspaceShellError::Record(error.to_string()))?;
        atomic_write_private(&self.record_path(&record.workspace_id)?, &bytes)
    }

    fn save_snapshot(
        &self,
        id: &str,
        manifest: &WorkspaceSnapshotManifest,
        snapshots_max: u32,
    ) -> Result<(), WorkspaceShellError> {
        let digest = manifest
            .object_ref
            .strip_prefix(WORKSPACE_SNAPSHOT_REF_PREFIX)
            .ok_or_else(|| WorkspaceShellError::Record("invalid snapshot ref".into()))?;
        let path = safe_file(&self.root.join(SNAPSHOT_DIR), &format!("{id}-{digest}"), "json")?;
        let bytes = serde_json::to_vec(manifest).map_err(|error| WorkspaceShellError::Record(error.to_string()))?;
        if path.try_exists()? {
            if read_no_follow(&path, RECORD_BYTES_MAX)? == bytes {
                return Ok(());
            }
            return Err(WorkspaceShellError::Record("snapshot identity collision".into()));
        }
        let prefix = format!("{id}-");
        let existing = std::fs::read_dir(self.root.join(SNAPSHOT_DIR))?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
            .count();
        let existing = u32::try_from(existing)
            .map_err(|_| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
        if existing >= snapshots_max {
            return Err(WorkspaceShellError::Quarantine(WorkspaceReasonCode::QuotaExceeded.as_str()));
        }
        create_new_private(&path, &bytes)
    }

    fn quarantine_content(&self, id: &str, generation: u64) -> Result<(), WorkspaceShellError> {
        let source = self.content_path(id)?;
        if !source.try_exists()? {
            return Ok(());
        }
        ensure_contained_no_symlinks(&self.root, &source)?;
        let destination = safe_child(&self.root.join(QUARANTINE_DIR), &format!("{id}-{generation}"))?;
        if destination.try_exists()? {
            return Err(WorkspaceShellError::Record("quarantine destination exists".into()));
        }
        std::fs::rename(source, destination)?;
        Ok(())
    }
}

pub struct ActiveWorkspace {
    store: WorkspaceStore,
    request: StatefulWorkspaceRequest,
    content_path: PathBuf,
    warm_state_used: bool,
    record: WorkspaceLeaseRecord,
    lock: File,
    finished: bool,
}

impl ActiveWorkspace {
    pub fn host_path(&self) -> &Path {
        &self.content_path
    }
    pub fn warm_state_used(&self) -> bool {
        self.warm_state_used
    }

    pub fn finish(mut self) -> WorkspaceExecutionReport {
        let result = self.finish_inner();
        self.finished = true;
        if let Err(error) = FileExt::unlock(&self.lock) {
            tracing::warn!(%error, "workspace lock release failed after finish");
        }
        match result {
            Ok(execution_report) => execution_report,
            Err(error) => {
                if let Err(quarantine_error) =
                    self.store.quarantine_content(&self.record.workspace_id, self.request.generation)
                {
                    tracing::warn!(error = %quarantine_error, "workspace quarantine failed after finish error");
                }
                let record = quarantined_record(self.record.clone(), self.request.generation);
                if let Err(record_error) = self.store.save_record(&record) {
                    tracing::warn!(error = %record_error, "workspace quarantine record save failed");
                }
                workspace_report(
                    &self.request,
                    self.warm_state_used,
                    WorkspaceCleanupDisposition::Failed,
                    reason_from_error(&error),
                    None,
                )
            }
        }
    }

    fn finish_inner(&mut self) -> Result<WorkspaceExecutionReport, WorkspaceShellError> {
        let (observations, plan) = scrub_observe_and_plan(&self.content_path, &self.request)?;
        if plan.quarantine_required {
            self.store.quarantine_content(&self.record.workspace_id, self.request.generation)?;
            self.record = quarantined_record(self.record.clone(), self.request.generation);
            self.store.save_record(&self.record)?;
            return Ok(workspace_report(
                &self.request,
                self.warm_state_used,
                WorkspaceCleanupDisposition::Quarantined,
                first_reason(&plan),
                None,
            ));
        }
        let snapshot_ref = if self.request.snapshot_enabled {
            let manifest = build_workspace_snapshot_manifest(
                self.request.compatibility_digest_blake3.clone(),
                self.request.guest_path.display().to_string(),
                observations,
                &plan,
            )
            .map_err(|reason| WorkspaceShellError::Quarantine(reason.as_str()))?;
            self.store.save_snapshot(&self.record.workspace_id, &manifest, self.request.quota_snapshots_max)?;
            Some(manifest.object_ref)
        } else {
            None
        };
        let release = lease_request(&self.request, WorkspaceLeaseOperation::Release)?;
        let released = plan_workspace_lease(Some(&self.record), &release);
        if released.disposition != WorkspaceLeaseDisposition::Accepted {
            return Err(WorkspaceShellError::Lease(released.reason.as_str()));
        }
        self.record = released.next.ok_or(WorkspaceShellError::Lease("accepted-release-missing-record"))?;
        self.store.save_record(&self.record)?;
        Ok(workspace_report(
            &self.request,
            self.warm_state_used,
            WorkspaceCleanupDisposition::Released,
            WorkspaceReasonCode::Accepted,
            snapshot_ref,
        ))
    }
}

impl Drop for ActiveWorkspace {
    fn drop(&mut self) {
        if !self.finished {
            if let Err(error) = self.store.quarantine_content(&self.record.workspace_id, self.request.generation) {
                tracing::warn!(%error, "workspace quarantine failed during drop");
            }
            let record = quarantined_record(self.record.clone(), self.request.generation);
            if let Err(error) = self.store.save_record(&record) {
                tracing::warn!(%error, "workspace quarantine record save failed during drop");
            }
        }
        if let Err(error) = FileExt::unlock(&self.lock) {
            tracing::warn!(%error, "workspace lock release failed during drop");
        }
    }
}

pub struct StatefulWorkspaceBuildService<S> {
    inner: S,
    store: WorkspaceStore,
    reports: WorkspaceReportCollector,
}

impl<S> StatefulWorkspaceBuildService<S> {
    pub fn new(inner: S, state_dir: impl AsRef<Path>, reports: WorkspaceReportCollector) -> Self {
        Self {
            inner,
            store: WorkspaceStore::new(state_dir),
            reports,
        }
    }
}

#[async_trait]
impl<S> BuildService for StatefulWorkspaceBuildService<S>
where S: BuildService + Send + Sync
{
    async fn do_build(&self, mut request: BuildRequest) -> std::io::Result<BuildResult> {
        let Some(workspace) = request.workspace.clone() else {
            return self.inner.do_build(request).await;
        };
        match workspace.mode {
            StatefulWorkspaceMode::None => {
                let result = self.inner.do_build(request).await;
                self.reports.push(workspace_report(
                    &workspace,
                    false,
                    WorkspaceCleanupDisposition::NotRequired,
                    WorkspaceReasonCode::Accepted,
                    None,
                ));
                result
            }
            StatefulWorkspaceMode::ImmutableSnapshot => {
                validate_snapshot_input(&request, &workspace).map_err(std::io::Error::other)?;
                let result = self.inner.do_build(request).await;
                self.reports.push(workspace_report(
                    &workspace,
                    false,
                    WorkspaceCleanupDisposition::NotRequired,
                    WorkspaceReasonCode::Accepted,
                    workspace.snapshot_input_name.as_ref().map(|path| path.display().to_string()),
                ));
                result
            }
            StatefulWorkspaceMode::MutableSession => {
                let active = self.store.prepare(&workspace).map_err(std::io::Error::other)?;
                let mut clean_request = request.clone();
                clean_request.workspace = None;
                let workspace_request = request
                    .workspace
                    .as_mut()
                    .ok_or_else(|| std::io::Error::other("mutable workspace request disappeared"))?;
                workspace_request.runtime_host_path = Some(active.host_path().to_path_buf());
                let result = self.inner.do_build(request).await;
                let clean_result = if workspace.clean_rebuild_enabled && result.is_ok() {
                    Some(self.inner.do_build(clean_request).await)
                } else {
                    None
                };
                let mut workspace_evidence = active.finish();
                workspace_evidence.clean_comparison_performed = clean_result.is_some();
                workspace_evidence.clean_comparison_matched = matches!(
                    (&result, &clean_result),
                    (Ok(warm), Some(Ok(clean))) if warm.outputs == clean.outputs
                );
                workspace_evidence.warm_output_set_digest_blake3 =
                    result.as_ref().ok().map(output_set_digest).transpose().map_err(std::io::Error::other)?;
                workspace_evidence.clean_output_set_digest_blake3 = clean_result
                    .as_ref()
                    .and_then(|clean| clean.as_ref().ok())
                    .map(output_set_digest)
                    .transpose()
                    .map_err(std::io::Error::other)?;
                debug_assert!(!workspace_evidence.original_execution_hermetic);
                debug_assert!(!workspace_evidence.shared_action_publish_allowed);
                self.reports.push(workspace_evidence);
                result
            }
        }
    }
}

pub struct IngestedWorkspaceSnapshot {
    pub node: Node,
    pub manifest: WorkspaceSnapshotManifest,
}

pub async fn ingest_immutable_workspace_snapshot<BS, DS>(
    root: &Path,
    request: &StatefulWorkspaceRequest,
    blob_service: &BS,
    directory_service: &DS,
) -> Result<IngestedWorkspaceSnapshot, WorkspaceShellError>
where
    BS: BlobService + Clone,
    DS: DirectoryService,
{
    let observations = scan_workspace(root, request)?;
    let plan = content_plan(&observations, request);
    if !plan.snapshot_allowed || plan.quarantine_required {
        return Err(WorkspaceShellError::Quarantine(first_reason(&plan).as_str()));
    }
    let manifest = build_workspace_snapshot_manifest(
        request.compatibility_digest_blake3.clone(),
        request.guest_path.display().to_string(),
        observations,
        &plan,
    )
    .map_err(|reason| WorkspaceShellError::Quarantine(reason.as_str()))?;
    let node =
        ingest_path(blob_service, directory_service, root, None::<&snix_castore::refscan::ReferenceScanner<Vec<u8>>>)
            .await
            .map_err(|error| WorkspaceShellError::Record(format!("snapshot CAS ingest failed: {error}")))?;
    Ok(IngestedWorkspaceSnapshot { node, manifest })
}

fn validate_snapshot_input(
    request: &BuildRequest,
    workspace: &StatefulWorkspaceRequest,
) -> Result<(), WorkspaceShellError> {
    let input_name = workspace
        .snapshot_input_name
        .as_ref()
        .ok_or(WorkspaceShellError::Policy("snapshot-input-missing"))?;
    let input_name =
        input_name.to_str().ok_or_else(|| WorkspaceShellError::Path("snapshot input is not UTF-8".into()))?;
    let node = request
        .inputs
        .iter()
        .find(|(name, _)| name.as_ref() == input_name.as_bytes())
        .map(|(_, node)| node)
        .ok_or(WorkspaceShellError::Policy("snapshot-input-not-declared"))?;
    if matches!(node, Node::Symlink { .. }) {
        return Err(WorkspaceShellError::Policy("snapshot-root-symlink-rejected"));
    }
    Ok(())
}

fn output_set_digest(result: &BuildResult) -> Result<String, WorkspaceShellError> {
    let bytes = serde_json::to_vec(&result.outputs).map_err(|error| WorkspaceShellError::Record(error.to_string()))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    debug_assert!(!digest.is_empty());
    debug_assert_eq!(digest.len(), BLAKE3_DIGEST_HEX_LENGTH);
    Ok(digest)
}

fn workspace_report(
    request: &StatefulWorkspaceRequest,
    warm_state_used: bool,
    cleanup: WorkspaceCleanupDisposition,
    cleanup_reason: WorkspaceReasonCode,
    snapshot_ref: Option<String>,
) -> WorkspaceExecutionReport {
    assert!(!WORKSPACE_EXECUTION_REPORT_SCHEMA.is_empty(), "workspace report schema must not be empty");
    assert!(!request.guest_path.as_os_str().is_empty(), "workspace report guest path must not be empty");
    let mode = match request.mode {
        StatefulWorkspaceMode::None => WorkspaceMode::None,
        StatefulWorkspaceMode::ImmutableSnapshot => WorkspaceMode::ImmutableSnapshot,
        StatefulWorkspaceMode::MutableSession => WorkspaceMode::MutableSession,
    };
    let claim = classify_workspace_claim(mode, false);
    WorkspaceExecutionReport {
        schema: WORKSPACE_EXECUTION_REPORT_SCHEMA.into(),
        mode,
        workspace_id: request.workspace_id.clone(),
        guest_path: request.guest_path.display().to_string(),
        compatibility_digest_blake3: request.compatibility_digest_blake3.clone(),
        warm_state_used,
        claim_class: claim.class,
        shared_action_publish_allowed: claim.shared_action_publish_allowed,
        strong_shared_reuse_allowed: claim.strong_shared_reuse_allowed,
        original_execution_hermetic: claim.original_execution_hermetic,
        clean_comparison_performed: false,
        clean_comparison_matched: false,
        warm_output_set_digest_blake3: None,
        clean_output_set_digest_blake3: None,
        cleanup,
        cleanup_reason,
        snapshot_ref,
    }
}

fn lease_request(
    request: &StatefulWorkspaceRequest,
    operation: WorkspaceLeaseOperation,
) -> Result<WorkspaceLeaseRequest, WorkspaceShellError> {
    let lease = request.lease.as_ref().ok_or(WorkspaceShellError::Policy("workspace-lease-missing"))?;
    let planned = WorkspaceLeaseRequest {
        workspace_id: request.workspace_id.clone().ok_or(WorkspaceShellError::Policy("workspace-id-missing"))?,
        compatibility_digest_blake3: request.compatibility_digest_blake3.clone(),
        toolchain_refs: request.toolchain_refs.clone(),
        guest_path: request.guest_path.display().to_string(),
        quota: request_quota(request),
        retention_class: request.retention_class.clone(),
        generation: request.generation,
        owner: WorkspaceLeaseOwner {
            worker_id: lease.worker_id.clone(),
            authority_class: lease.authority_class.clone(),
            job_id: lease.job_id.clone(),
            attempt_id: lease.attempt_id.clone(),
            fence_generation: lease.fence_generation,
        },
        operation,
    };
    assert_eq!(planned.operation, operation);
    assert_eq!(planned.generation, request.generation);
    Ok(planned)
}

fn request_quota(request: &StatefulWorkspaceRequest) -> WorkspaceQuotaPolicy {
    WorkspaceQuotaPolicy {
        bytes_max: request.quota_bytes_max,
        files_max: request.quota_files_max,
        snapshots_max: request.quota_snapshots_max,
    }
}

fn scrub_and_plan(
    root: &Path,
    request: &StatefulWorkspaceRequest,
) -> Result<WorkspaceContentPlan, WorkspaceShellError> {
    let (_, plan) = scrub_observe_and_plan(root, request)?;
    Ok(plan)
}

fn scrub_observe_and_plan(
    root: &Path,
    request: &StatefulWorkspaceRequest,
) -> Result<(Vec<WorkspaceEntryObservation>, WorkspaceContentPlan), WorkspaceShellError> {
    let mut observations = scan_workspace(root, request)?;
    let mut plan = content_plan(&observations, request);
    for relative in &plan.scrub_paths {
        let path = root.join(relative);
        ensure_contained_no_symlinks(root, &path)?;
        remove_tree_no_follow_if_exists(&path)?;
    }
    if !plan.scrub_paths.is_empty() {
        observations = scan_workspace(root, request)?;
        plan = content_plan(&observations, request);
    }
    Ok((observations, plan))
}

fn content_plan(
    observations: &[WorkspaceEntryObservation],
    request: &StatefulWorkspaceRequest,
) -> WorkspaceContentPlan {
    plan_workspace_content(
        observations,
        &request_quota(request),
        &WorkspaceScrubPolicy {
            sensitive_paths: request.sensitive_paths.clone(),
            secret_markers: request.secret_markers.clone(),
            scan_depth_max: request.scan_depth_max,
            path_bytes_max: request.path_bytes_max,
            reject_host_paths: true,
        },
        &WorkspaceSnapshotPolicy {
            enabled: request.snapshot_enabled,
            require_clean_scrub: true,
        },
    )
}

fn scan_workspace(
    root: &Path,
    request: &StatefulWorkspaceRequest,
) -> Result<Vec<WorkspaceEntryObservation>, WorkspaceShellError> {
    assert!(root.is_absolute(), "workspace scan root must be absolute");
    assert!(request.quota_files_max > 0, "workspace scan file quota must be positive");
    ensure_path_chain_no_symlinks(root)?;
    let mut pending = vec![(root.to_path_buf(), PathBuf::new(), 0u32)];
    let snapshot_entry_bound = u32::try_from(MAX_WORKSPACE_SNAPSHOT_ENTRIES)
        .map_err(|_| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
    let bounded_entries = request.quota_files_max.min(snapshot_entry_bound);
    let observation_slots = usize::try_from(bounded_entries)
        .map_err(|_| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
    let mut observations = Vec::with_capacity(observation_slots);
    let mut total_bytes = 0u64;
    while let Some((path, relative, depth)) = pending.pop() {
        if depth > request.scan_depth_max {
            return Err(WorkspaceShellError::Quarantine(WorkspaceReasonCode::PathInvalid.as_str()));
        }
        if relative.as_os_str().is_empty() {
            enqueue_children(&path, &relative, depth, &mut pending)?;
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)?;
        let relative_text = relative
            .to_str()
            .ok_or_else(|| WorkspaceShellError::Path("workspace path is not UTF-8".into()))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let relative_len = u32::try_from(relative_text.len())
            .map_err(|_| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
        if relative_len > request.path_bytes_max {
            return Err(WorkspaceShellError::Quarantine(WorkspaceReasonCode::PathInvalid.as_str()));
        }
        if metadata.file_type().is_symlink() {
            observations.push(symlink_observation(relative_text, &path)?);
        } else if metadata.is_dir() {
            observations.push(directory_observation(relative_text));
            enqueue_children(&path, &relative, depth, &mut pending)?;
        } else if metadata.is_file() {
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
            let bytes = read_no_follow(&path, request.quota_bytes_max)?;
            observations.push(WorkspaceEntryObservation {
                relative_path: relative_text,
                kind: WorkspaceEntryKind::File,
                size_bytes: metadata.len(),
                content_digest_blake3: Some(blake3::hash(&bytes).to_hex().to_string()),
                symlink_target: None,
                contains_secret: contains_secret(&bytes, &request.secret_markers),
                contains_host_path: contains_host_path(&bytes, root),
            });
        } else {
            return Err(WorkspaceShellError::Quarantine(WorkspaceReasonCode::PathInvalid.as_str()));
        }
        let count = u32::try_from(observations.len())
            .map_err(|_| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
        if count > request.quota_files_max || total_bytes > request.quota_bytes_max {
            return Err(WorkspaceShellError::Quarantine(WorkspaceReasonCode::QuotaExceeded.as_str()));
        }
    }
    observations.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(observations)
}

fn symlink_observation(relative_path: String, path: &Path) -> Result<WorkspaceEntryObservation, WorkspaceShellError> {
    Ok(WorkspaceEntryObservation {
        relative_path,
        kind: WorkspaceEntryKind::Symlink,
        size_bytes: 0,
        content_digest_blake3: None,
        symlink_target: std::fs::read_link(path)?.to_str().map(ToOwned::to_owned),
        contains_secret: false,
        contains_host_path: false,
    })
}

fn directory_observation(relative_path: String) -> WorkspaceEntryObservation {
    WorkspaceEntryObservation {
        relative_path,
        kind: WorkspaceEntryKind::Directory,
        size_bytes: 0,
        content_digest_blake3: None,
        symlink_target: None,
        contains_secret: false,
        contains_host_path: false,
    }
}

fn enqueue_children(
    path: &Path,
    relative: &Path,
    depth: u32,
    pending: &mut Vec<(PathBuf, PathBuf, u32)>,
) -> Result<(), WorkspaceShellError> {
    let next_depth = depth
        .checked_add(1)
        .ok_or_else(|| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
    let mut children = std::fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for child in children.into_iter().rev() {
        pending.push((child.path(), relative.join(child.file_name()), next_depth));
    }
    Ok(())
}

fn read_no_follow(path: &Path, bytes_max: u64) -> Result<Vec<u8>, WorkspaceShellError> {
    assert!(!path.as_os_str().is_empty(), "no-follow read path must not be empty");
    assert!(bytes_max > 0, "no-follow read byte limit must be positive");
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path)?;
    let read_limit_bytes = bytes_max
        .checked_add(1)
        .ok_or_else(|| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?;
    let mut bytes = Vec::new();
    file.take(read_limit_bytes).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len())
        .map_err(|_| WorkspaceShellError::Quarantine(WorkspaceReasonCode::ArithmeticOverflow.as_str()))?
        > bytes_max
    {
        return Err(WorkspaceShellError::Quarantine(WorkspaceReasonCode::QuotaExceeded.as_str()));
    }
    Ok(bytes)
}

fn contains_secret(bytes: &[u8], markers: &[String]) -> bool {
    let text = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    markers.iter().any(|marker| text.contains(&marker.to_ascii_lowercase()))
}

fn contains_host_path(bytes: &[u8], root: &Path) -> bool {
    let root_text = root.as_os_str().to_string_lossy();
    if !root_text.is_empty() && contains_bytes(bytes, root_text.as_bytes()) {
        return true;
    }
    HOST_PATH_MARKERS.iter().any(|marker| contains_bytes(bytes, marker))
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|window| window == needle)
}

fn first_reason(plan: &WorkspaceContentPlan) -> WorkspaceReasonCode {
    plan.reasons.first().copied().unwrap_or(WorkspaceReasonCode::SnapshotRejected)
}

fn quarantined_record(mut record: WorkspaceLeaseRecord, generation: u64) -> WorkspaceLeaseRecord {
    record.state = WorkspaceLeaseState::Quarantined;
    record.owner = None;
    record.last_used_generation = generation;
    record
}

fn reason_from_error(error: &WorkspaceShellError) -> WorkspaceReasonCode {
    match error {
        WorkspaceShellError::ConcurrentLease => WorkspaceReasonCode::ConcurrentOwner,
        WorkspaceShellError::Path(_) => WorkspaceReasonCode::PathInvalid,
        WorkspaceShellError::Lease(reason)
        | WorkspaceShellError::Policy(reason)
        | WorkspaceShellError::Quarantine(reason) => reason_code(reason),
        WorkspaceShellError::Io(_) | WorkspaceShellError::Record(_) => WorkspaceReasonCode::SnapshotRejected,
    }
}

fn reason_code(reason: &str) -> WorkspaceReasonCode {
    [
        WorkspaceReasonCode::ConcurrentOwner,
        WorkspaceReasonCode::LeaseQuarantined,
        WorkspaceReasonCode::QuotaExceeded,
        WorkspaceReasonCode::SecretDetected,
        WorkspaceReasonCode::HostPathDetected,
        WorkspaceReasonCode::SymlinkEscape,
        WorkspaceReasonCode::StaleFence,
    ]
    .into_iter()
    .find(|code| code.as_str() == reason)
    .unwrap_or(WorkspaceReasonCode::PolicyInvalid)
}

fn safe_child(root: &Path, name: &str) -> Result<PathBuf, WorkspaceShellError> {
    if name.is_empty()
        || name.len() > MAX_WORKSPACE_ID_BYTES
        || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(WorkspaceShellError::Path("unsafe workspace path component".into()));
    }
    Ok(root.join(name))
}

fn safe_file(root: &Path, name: &str, extension: impl AsRef<str>) -> Result<PathBuf, WorkspaceShellError> {
    let mut path = safe_child(root, name)?;
    path.set_extension(extension.as_ref());
    Ok(path)
}

fn create_private_dir(path: &Path) -> Result<(), WorkspaceShellError> {
    if path.try_exists()? {
        let metadata = std::fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(WorkspaceShellError::Path(format!("{} is not a directory", path.display())));
        }
        return Ok(());
    }
    std::fs::create_dir(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(DIR_MODE))?;
    }
    Ok(())
}

fn private_open_create(path: &Path) -> Result<File, WorkspaceShellError> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(FILE_MODE).custom_flags(libc::O_NOFOLLOW);
    }
    Ok(options.open(path)?)
}

fn create_new_private(path: &Path, bytes: &[u8]) -> Result<(), WorkspaceShellError> {
    use std::io::Write;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(FILE_MODE).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn atomic_write_private(path: &Path, bytes: &[u8]) -> Result<(), WorkspaceShellError> {
    let temp = path.with_extension("tmp");
    remove_file_no_follow_if_exists(&temp)?;
    create_new_private(&temp, bytes)?;
    std::fs::rename(&temp, path)?;
    sync_parent(path)?;
    Ok(())
}

fn sync_parent(path: &Path) -> Result<(), WorkspaceShellError> {
    let parent = path.parent().ok_or_else(|| WorkspaceShellError::Path("path has no parent".into()))?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn ensure_regular_no_follow(path: &Path) -> Result<(), WorkspaceShellError> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(WorkspaceShellError::Path(format!("{} is not a regular file", path.display())));
    }
    Ok(())
}

fn ensure_path_chain_no_symlinks(path: &Path) -> Result<(), WorkspaceShellError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if !current.try_exists()? {
            continue;
        }
        if std::fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Err(WorkspaceShellError::Path(format!("{} is a symlink", current.display())));
        }
    }
    Ok(())
}

fn ensure_contained_no_symlinks(root: &Path, path: &Path) -> Result<(), WorkspaceShellError> {
    if !path.starts_with(root) {
        return Err(WorkspaceShellError::Path("path escapes workspace root".into()));
    }
    ensure_path_chain_no_symlinks(path)
}

fn directory_has_entries(path: &Path) -> Result<bool, WorkspaceShellError> {
    Ok(std::fs::read_dir(path)?.next().transpose()?.is_some())
}

fn remove_file_no_follow_if_exists(path: &Path) -> Result<(), WorkspaceShellError> {
    if !path.try_exists()? {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Err(WorkspaceShellError::Path("refusing to unlink directory as file".into()));
    }
    std::fs::remove_file(path)?;
    Ok(())
}

fn remove_tree_no_follow_if_exists(path: &Path) -> Result<(), WorkspaceShellError> {
    assert!(!path.as_os_str().is_empty(), "workspace removal path must not be empty");
    if !path.try_exists()? && std::fs::symlink_metadata(path).is_err() {
        return Ok(());
    }
    let removal_bound = MAX_WORKSPACE_SNAPSHOT_ENTRIES;
    let mut pending = Vec::with_capacity(WORKSPACE_REMOVAL_STACK_INITIAL_ENTRIES);
    pending.push((path.to_path_buf(), false));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].0, path);
    let mut visited_entries = 0usize;
    while let Some((current, is_expanded)) = pending.pop() {
        if !is_expanded {
            visited_entries = visited_entries
                .checked_add(1)
                .ok_or_else(|| WorkspaceShellError::Record("workspace removal count overflow".into()))?;
            if visited_entries > removal_bound {
                return Err(WorkspaceShellError::Record("workspace removal bound exceeded".into()));
            }
        }
        let metadata = std::fs::symlink_metadata(&current)?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            if is_expanded {
                std::fs::remove_dir(&current)?;
                continue;
            }
            let mut children = std::fs::read_dir(&current)?.collect::<Result<Vec<_>, _>>()?;
            children.sort_by_key(std::fs::DirEntry::file_name);
            let pending_count = pending
                .len()
                .checked_add(children.len())
                .and_then(|count| count.checked_add(1))
                .ok_or_else(|| WorkspaceShellError::Record("workspace removal queue overflow".into()))?;
            if pending_count > removal_bound {
                return Err(WorkspaceShellError::Record("workspace removal queue bound exceeded".into()));
            }
            pending.push((current, true));
            pending.extend(children.into_iter().rev().map(|child| (child.path(), false)));
        } else {
            std::fs::remove_file(&current)?;
        }
    }
    Ok(())
}

fn remove_prefixed_entries(root: &Path, id: &str) -> Result<(), WorkspaceShellError> {
    assert!(!id.is_empty(), "workspace prefix ID must not be empty");
    if !root.try_exists()? {
        return Ok(());
    }
    let prefix = format!("{id}-");
    assert!(prefix.starts_with(id));
    assert!(prefix.ends_with('-'));
    let mut matched = 0usize;
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) {
            matched = matched
                .checked_add(1)
                .ok_or_else(|| WorkspaceShellError::Record("quarantine count overflow".into()))?;
            if matched > MAX_WORKSPACE_RETENTION_RECORDS {
                return Err(WorkspaceShellError::Record("quarantine delete bound exceeded".into()));
            }
            remove_tree_no_follow_if_exists(&entry.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    use snix_build::buildservice::BuildOutput;
    use snix_build::buildservice::StatefulWorkspaceLeaseBinding;
    use snix_castore::SymlinkTarget;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    use super::*;

    const GENERATION: u64 = 7;
    const TEST_DIGEST_HEX_LENGTH: usize = 64;
    const TOOL_FIXTURE_RUNS: u32 = 2;

    fn digest(byte: char) -> String {
        byte.to_string().repeat(TEST_DIGEST_HEX_LENGTH)
    }

    fn request() -> StatefulWorkspaceRequest {
        StatefulWorkspaceRequest {
            mode: StatefulWorkspaceMode::MutableSession,
            workspace_id: Some("cargo-cache".into()),
            guest_path: PathBuf::from(DEFAULT_WORKSPACE_GUEST_PATH),
            snapshot_input_name: None,
            compatibility_digest_blake3: digest('a'),
            toolchain_refs: vec!["mantle-object://blake3/rust".into()],
            quota_bytes_max: DEFAULT_WORKSPACE_BYTES_MAX,
            quota_files_max: DEFAULT_WORKSPACE_FILES_MAX,
            quota_snapshots_max: DEFAULT_WORKSPACE_SNAPSHOTS_MAX,
            retention_class: "recent".into(),
            retention_workspace_count_max: DEFAULT_WORKSPACE_COUNT_MAX,
            retention_idle_generations_max: DEFAULT_WORKSPACE_IDLE_GENERATIONS_MAX,
            retention_age_generations_max: DEFAULT_WORKSPACE_AGE_GENERATIONS_MAX,
            retention_quarantine_count_max: DEFAULT_WORKSPACE_QUARANTINE_COUNT_MAX,
            generation: GENERATION,
            lease: Some(StatefulWorkspaceLeaseBinding {
                worker_id: "worker-a".into(),
                authority_class: "tenant-a".into(),
                job_id: "job-a".into(),
                attempt_id: "attempt-a".into(),
                fence_generation: 1,
            }),
            sensitive_paths: WorkspaceScrubPolicy::default().sensitive_paths,
            secret_markers: WorkspaceScrubPolicy::default().secret_markers,
            scan_depth_max: DEFAULT_WORKSPACE_SCAN_DEPTH_MAX,
            path_bytes_max: DEFAULT_WORKSPACE_PATH_BYTES_MAX,
            snapshot_enabled: true,
            clean_rebuild_enabled: false,
            clean_rebuild_require_declared_inputs: true,
            runtime_host_path: None,
        }
    }

    #[test]
    fn mutable_workspace_persists_warm_state_and_scrubs_sensitive_paths() {
        let temp = tempfile::tempdir().unwrap();
        let store = WorkspaceStore::new(temp.path());
        let first = store.prepare(&request()).unwrap();
        assert!(!first.warm_state_used());
        std::fs::write(first.host_path().join("cache.bin"), b"safe-cache").unwrap();
        std::fs::write(first.host_path().join(".netrc"), b"machine secret").unwrap();
        let first_report = first.finish();
        assert_eq!(first_report.cleanup, WorkspaceCleanupDisposition::Released);
        assert!(!store.root().join(CONTENT_DIR).join("cargo-cache/.netrc").exists());
        let second = store.prepare(&request()).unwrap();
        assert!(second.warm_state_used());
        assert!(second.host_path().join("cache.bin").exists());
        let report = second.finish();
        assert!(!report.shared_action_publish_allowed);
        assert!(!report.strong_shared_reuse_allowed);
    }

    #[test]
    fn concurrent_and_foreign_attempts_are_rejected_and_restart_record_survives() {
        let temp = tempfile::tempdir().unwrap();
        let store = WorkspaceStore::new(temp.path());
        let active = store.prepare(&request()).unwrap();
        assert!(matches!(store.prepare(&request()), Err(WorkspaceShellError::ConcurrentLease)));
        FileExt::unlock(&active.lock).unwrap();
        std::mem::forget(active);
        let reopened = WorkspaceStore::new(temp.path());
        let record = reopened.load_record("cargo-cache").unwrap().unwrap();
        assert_eq!(record.state, WorkspaceLeaseState::Active);
        let mut foreign = request();
        foreign.lease.as_mut().unwrap().attempt_id = "attempt-b".into();
        assert!(matches!(reopened.prepare(&foreign), Err(WorkspaceShellError::Lease("attempt-mismatch"))));
    }

    #[test]
    fn abandoned_cleanup_quarantines_instead_of_reusing_unknown_state() {
        let temp = tempfile::tempdir().unwrap();
        let store = WorkspaceStore::new(temp.path());
        let active = store.prepare(&request()).unwrap();
        std::fs::write(active.host_path().join("partial"), b"unknown-state").unwrap();
        drop(active);
        let record = store.load_record("cargo-cache").unwrap().unwrap();
        assert_eq!(record.state, WorkspaceLeaseState::Quarantined);
        assert!(!store.content_path("cargo-cache").unwrap().exists());
    }

    #[test]
    fn symlink_escape_and_secret_content_quarantine_without_following_target() {
        let temp = tempfile::tempdir().unwrap();
        let outside = temp.path().join("outside");
        std::fs::write(&outside, b"outside-stays").unwrap();
        let state_dir = temp.path().join("state");
        std::fs::create_dir(&state_dir).unwrap();
        let store = WorkspaceStore::new(&state_dir);
        store.ensure_layout().unwrap();
        let content = store.content_path("cargo-cache").unwrap();
        create_private_dir(&content).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, content.join("escape")).unwrap();
        std::fs::write(content.join("token.txt"), b"token=do-not-keep").unwrap();
        let result = store.prepare(&request());
        assert!(matches!(result, Err(WorkspaceShellError::Quarantine(_))));
        assert_eq!(std::fs::read(&outside).unwrap(), b"outside-stays");
        let record = store.load_record("cargo-cache").unwrap().unwrap();
        assert_eq!(record.state, WorkspaceLeaseState::Quarantined);
    }

    #[tokio::test]
    async fn immutable_snapshot_is_content_addressed_and_secret_input_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("cache.bin"), b"cache-safe").unwrap();
        let mut snapshot = request();
        snapshot.mode = StatefulWorkspaceMode::ImmutableSnapshot;
        snapshot.snapshot_enabled = true;
        snapshot.lease = None;
        let blobs = MemoryBlobService::default();
        let directories = RedbDirectoryService::new_temporary(
            "workspace-snapshot-tests".to_string(),
            RedbDirectoryServiceConfig::default(),
        )
        .unwrap();
        let first = ingest_immutable_workspace_snapshot(temp.path(), &snapshot, &blobs, &directories).await.unwrap();
        let second = ingest_immutable_workspace_snapshot(temp.path(), &snapshot, &blobs, &directories).await.unwrap();
        assert_eq!(first.manifest, second.manifest);
        assert_eq!(first.node, second.node);
        assert!(first.manifest.object_ref.starts_with(WORKSPACE_SNAPSHOT_REF_PREFIX));

        std::fs::write(temp.path().join("token.txt"), b"token=reject").unwrap();
        let rejected = ingest_immutable_workspace_snapshot(temp.path(), &snapshot, &blobs, &directories).await;
        assert!(matches!(rejected, Err(WorkspaceShellError::Quarantine("secret-detected"))));
    }

    #[test]
    fn path_escape_and_symlinked_state_root_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let store = WorkspaceStore::new(temp.path());
        let mut invalid = request();
        invalid.workspace_id = Some("../escape".into());
        assert!(matches!(store.prepare(&invalid), Err(WorkspaceShellError::Path(_))));
        #[cfg(unix)]
        {
            let linked_temp = tempfile::tempdir().unwrap();
            let target = linked_temp.path().join("real");
            std::fs::create_dir(&target).unwrap();
            std::os::unix::fs::symlink(&target, linked_temp.path().join(WORKSPACE_STATE_DIR_NAME)).unwrap();
            let linked = WorkspaceStore::new(linked_temp.path());
            assert!(matches!(linked.prepare(&request()), Err(WorkspaceShellError::Path(_))));
        }
    }

    #[test]
    fn retention_eviction_is_applied_before_new_workspace_admission() {
        let temp = tempfile::tempdir().unwrap();
        let store = WorkspaceStore::new(temp.path());
        let mut old = request();
        old.generation = 1;
        old.retention_workspace_count_max = 1;
        old.retention_idle_generations_max = 2;
        old.retention_age_generations_max = 2;
        store.prepare(&old).unwrap().finish();
        assert!(store.content_path("cargo-cache").unwrap().exists());

        let mut new = request();
        new.workspace_id = Some("cargo-cache-new".to_string());
        new.lease.as_mut().unwrap().job_id = "job-new".to_string();
        new.generation = GENERATION;
        new.retention_workspace_count_max = 1;
        new.retention_idle_generations_max = 2;
        new.retention_age_generations_max = 2;
        let active = store.prepare(&new).unwrap();
        assert!(!store.content_path("cargo-cache").unwrap().exists());
        assert!(store.content_path("cargo-cache-new").unwrap().exists());
        active.finish();
    }

    struct WarmToolService;

    #[async_trait]
    impl BuildService for WarmToolService {
        async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
            let workspace = request.workspace.as_ref().expect("tool fixture receives workspace");
            assert_eq!(workspace.guest_path, PathBuf::from(DEFAULT_WORKSPACE_GUEST_PATH));
            let host = workspace.runtime_host_path.as_ref().expect("shell resolves host path");
            let marker = host.join("tool-cache.marker");
            let warm = marker.exists();
            if !warm {
                std::fs::write(&marker, b"retained-tool-cache")?;
            }
            Ok(BuildResult {
                outputs: Vec::new(),
                log: Some(if warm { "warm" } else { "cold" }.to_string()),
            })
        }
    }

    #[tokio::test]
    async fn tool_fixture_reuses_warm_state_at_stable_guest_path_with_downgraded_claim() {
        let temp = tempfile::tempdir().unwrap();
        let reports = WorkspaceReportCollector::default();
        let service = StatefulWorkspaceBuildService::new(WarmToolService, temp.path(), reports.clone());
        for _ in 0..TOOL_FIXTURE_RUNS {
            let build = BuildRequest {
                workspace: Some(request()),
                ..Default::default()
            };
            service.do_build(build).await.unwrap();
        }
        let reports = reports.take();
        assert_eq!(reports.len(), 2);
        assert!(!reports[0].warm_state_used);
        assert!(reports[1].warm_state_used);
        assert!(reports.iter().all(|report| report.guest_path == DEFAULT_WORKSPACE_GUEST_PATH));
        assert!(reports.iter().all(|report| !report.shared_action_publish_allowed));
        assert!(reports.iter().all(|report| !report.strong_shared_reuse_allowed));
    }

    struct ComparisonService {
        diverge: bool,
        calls: AtomicU32,
    }

    #[async_trait]
    impl BuildService for ComparisonService {
        async fn do_build(&self, _request: BuildRequest) -> std::io::Result<BuildResult> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            let target = if self.diverge && call > 0 {
                "clean-diverged"
            } else {
                "same-output"
            };
            Ok(BuildResult {
                outputs: vec![BuildOutput {
                    node: Node::Symlink {
                        target: SymlinkTarget::try_from(target).unwrap(),
                    },
                    output_needles: BTreeSet::new(),
                }],
                log: None,
            })
        }
    }

    #[tokio::test]
    async fn clean_comparison_never_relabels_or_publishes_original_mutable_execution() {
        for (diverge, expected_match) in [(false, true), (true, false)] {
            let temp = tempfile::tempdir().unwrap();
            let reports = WorkspaceReportCollector::default();
            let service = StatefulWorkspaceBuildService::new(
                ComparisonService {
                    diverge,
                    calls: AtomicU32::new(0),
                },
                temp.path(),
                reports.clone(),
            );
            let mut workspace = request();
            workspace.clean_rebuild_enabled = true;
            let build = BuildRequest {
                workspace: Some(workspace),
                ..Default::default()
            };
            service.do_build(build).await.unwrap();
            let report = reports.take().pop().unwrap();
            assert!(report.clean_comparison_performed);
            assert_eq!(report.clean_comparison_matched, expected_match);
            assert!(report.warm_output_set_digest_blake3.is_some());
            assert!(report.clean_output_set_digest_blake3.is_some());
            assert_eq!(report.warm_output_set_digest_blake3 == report.clean_output_set_digest_blake3, expected_match);
            assert!(!report.original_execution_hermetic);
            assert!(!report.shared_action_publish_allowed);
            assert!(!report.strong_shared_reuse_allowed);
        }
    }

    #[test]
    fn quota_failure_quarantines_and_reports_no_host_path() {
        let temp = tempfile::tempdir().unwrap();
        let store = WorkspaceStore::new(temp.path());
        let mut small = request();
        small.quota_bytes_max = 1;
        store.ensure_layout().unwrap();
        let content = store.content_path("cargo-cache").unwrap();
        create_private_dir(&content).unwrap();
        std::fs::write(content.join("large"), b"too-large").unwrap();
        assert!(matches!(store.prepare(&small), Err(WorkspaceShellError::Quarantine("quota-exceeded"))));
        let serialized = serde_json::to_string(&workspace_report(
            &small,
            true,
            WorkspaceCleanupDisposition::Quarantined,
            WorkspaceReasonCode::QuotaExceeded,
            None,
        ))
        .unwrap();
        assert!(!serialized.contains(temp.path().to_string_lossy().as_ref()));
        assert!(serialized.contains("practical-mutable-history"));
    }
}
