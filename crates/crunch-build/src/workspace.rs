//! Pure bounded planning for retained tool workspaces.
//!
//! Filesystem observation, locking, mounting, clocks, CAS ingestion, scrubbing,
//! quarantine moves, and persistence belong to `workspace_shell`.
//!
//! r[impl build_correctness.stateful_workspace_modes]
//! r[impl build_correctness.mutable_workspace_claim_boundary]
//! r[impl remote_builds.stateful_workspace_leases]

use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub const WORKSPACE_POLICY_SCHEMA: &str = "mantle-stateful-workspace-policy-v1";
pub const WORKSPACE_SNAPSHOT_SCHEMA: &str = "mantle-workspace-snapshot-v1";
pub const WORKSPACE_SNAPSHOT_REF_PREFIX: &str = "mantle-workspace-snapshot://blake3/";
pub const DEFAULT_WORKSPACE_GUEST_PATH: &str = "/build/.mantle-workspace";
pub const DEFAULT_WORKSPACE_BYTES_MAX: u64 = 4_294_967_296;
pub const DEFAULT_WORKSPACE_FILES_MAX: u32 = 100_000;
pub const DEFAULT_WORKSPACE_SNAPSHOTS_MAX: u32 = 8;
pub const DEFAULT_WORKSPACE_COUNT_MAX: u32 = 64;
pub const DEFAULT_WORKSPACE_IDLE_GENERATIONS_MAX: u64 = 16;
pub const DEFAULT_WORKSPACE_AGE_GENERATIONS_MAX: u64 = 256;
pub const DEFAULT_WORKSPACE_QUARANTINE_COUNT_MAX: u32 = 16;
pub const DEFAULT_WORKSPACE_SCAN_DEPTH_MAX: u32 = 128;
pub const DEFAULT_WORKSPACE_PATH_BYTES_MAX: u32 = 4_096;
pub const MAX_WORKSPACE_ID_BYTES: usize = 128;
pub const MAX_WORKSPACE_AUTHORITY_BYTES: usize = 256;
pub const MAX_WORKSPACE_TOOLCHAIN_REFS: usize = 128;
pub const MAX_WORKSPACE_SENSITIVE_PATHS: usize = 128;
pub const MAX_WORKSPACE_SECRET_MARKERS: usize = 64;
pub const MAX_WORKSPACE_RETENTION_RECORDS: usize = 4_096;
pub const MAX_WORKSPACE_SNAPSHOT_ENTRIES: usize = 100_000;
const WORKSPACE_FILES_OVER_LIMIT: u32 = DEFAULT_WORKSPACE_FILES_MAX.saturating_add(1);
const BLAKE3_HEX_LENGTH: usize = 64;
const COMPATIBILITY_DIGEST_DOMAIN: &str = "mantle-workspace-compatibility-v1";
const SNAPSHOT_DIGEST_DOMAIN: &str = "mantle-workspace-snapshot-v1";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceMode {
    #[default]
    None,
    ImmutableSnapshot,
    MutableSession,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceFallbackPolicy {
    #[default]
    Fail,
    ToNone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceQuotaPolicy {
    pub bytes_max: u64,
    pub files_max: u32,
    pub snapshots_max: u32,
}

impl Default for WorkspaceQuotaPolicy {
    fn default() -> Self {
        Self {
            bytes_max: DEFAULT_WORKSPACE_BYTES_MAX,
            files_max: DEFAULT_WORKSPACE_FILES_MAX,
            snapshots_max: DEFAULT_WORKSPACE_SNAPSHOTS_MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRetentionPolicy {
    pub workspace_count_max: u32,
    pub idle_generations_max: u64,
    pub age_generations_max: u64,
    pub quarantine_count_max: u32,
}

impl Default for WorkspaceRetentionPolicy {
    fn default() -> Self {
        Self {
            workspace_count_max: DEFAULT_WORKSPACE_COUNT_MAX,
            idle_generations_max: DEFAULT_WORKSPACE_IDLE_GENERATIONS_MAX,
            age_generations_max: DEFAULT_WORKSPACE_AGE_GENERATIONS_MAX,
            quarantine_count_max: DEFAULT_WORKSPACE_QUARANTINE_COUNT_MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceScrubPolicy {
    pub sensitive_paths: Vec<String>,
    pub secret_markers: Vec<String>,
    pub scan_depth_max: u32,
    pub path_bytes_max: u32,
    pub reject_host_paths: bool,
}

impl Default for WorkspaceScrubPolicy {
    fn default() -> Self {
        Self {
            sensitive_paths: vec![".netrc".to_string(), ".npmrc".to_string(), ".pypirc".to_string()],
            secret_markers: vec![
                "authorization".to_string(),
                "bearer ".to_string(),
                "credential".to_string(),
                "password".to_string(),
                "private-key".to_string(),
                "secret".to_string(),
                "token=".to_string(),
            ],
            scan_depth_max: DEFAULT_WORKSPACE_SCAN_DEPTH_MAX,
            path_bytes_max: DEFAULT_WORKSPACE_PATH_BYTES_MAX,
            reject_host_paths: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSnapshotPolicy {
    pub enabled: bool,
    pub require_clean_scrub: bool,
}

impl Default for WorkspaceSnapshotPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            require_clean_scrub: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCleanRebuildPolicy {
    pub enabled: bool,
    pub require_declared_inputs: bool,
}

impl Default for WorkspaceCleanRebuildPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            require_declared_inputs: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCompatibilityFacts {
    pub authority_class: String,
    pub action_class: String,
    pub toolchain_refs: Vec<String>,
}

impl Default for WorkspaceCompatibilityFacts {
    fn default() -> Self {
        Self {
            authority_class: "local-default".to_string(),
            action_class: "default".to_string(),
            toolchain_refs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspacePolicy {
    pub schema: String,
    pub mode: WorkspaceMode,
    pub fallback: WorkspaceFallbackPolicy,
    pub workspace_id: Option<String>,
    pub snapshot_ref: Option<String>,
    pub guest_path: String,
    pub compatibility: WorkspaceCompatibilityFacts,
    pub quota: WorkspaceQuotaPolicy,
    pub retention: WorkspaceRetentionPolicy,
    pub scrub: WorkspaceScrubPolicy,
    pub snapshot: WorkspaceSnapshotPolicy,
    pub clean_rebuild: WorkspaceCleanRebuildPolicy,
}

impl Default for WorkspacePolicy {
    fn default() -> Self {
        Self {
            schema: WORKSPACE_POLICY_SCHEMA.to_string(),
            mode: WorkspaceMode::None,
            fallback: WorkspaceFallbackPolicy::Fail,
            workspace_id: None,
            snapshot_ref: None,
            guest_path: DEFAULT_WORKSPACE_GUEST_PATH.to_string(),
            compatibility: WorkspaceCompatibilityFacts::default(),
            quota: WorkspaceQuotaPolicy::default(),
            retention: WorkspaceRetentionPolicy::default(),
            scrub: WorkspaceScrubPolicy::default(),
            snapshot: WorkspaceSnapshotPolicy::default(),
            clean_rebuild: WorkspaceCleanRebuildPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceAvailability {
    Available,
    Missing,
    Busy,
    Quarantined,
    OverQuota,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceAdmissionDisposition {
    Accepted,
    ExplicitFallback,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAdmissionPlan {
    pub requested_mode: WorkspaceMode,
    pub actual_mode: WorkspaceMode,
    pub disposition: WorkspaceAdmissionDisposition,
    pub reason: WorkspaceReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceLeaseOwner {
    pub worker_id: String,
    pub authority_class: String,
    pub job_id: String,
    pub attempt_id: String,
    pub fence_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceLeaseState {
    Idle,
    Active,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceLeaseRecord {
    pub workspace_id: String,
    pub compatibility_digest_blake3: String,
    pub toolchain_refs: Vec<String>,
    pub guest_path: String,
    pub quota: WorkspaceQuotaPolicy,
    pub retention_class: String,
    pub creation_generation: u64,
    pub last_used_generation: u64,
    pub state: WorkspaceLeaseState,
    pub owner: Option<WorkspaceLeaseOwner>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceLeaseOperation {
    Acquire,
    Renew,
    Release,
    Quarantine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceLeaseRequest {
    pub workspace_id: String,
    pub compatibility_digest_blake3: String,
    pub toolchain_refs: Vec<String>,
    pub guest_path: String,
    pub quota: WorkspaceQuotaPolicy,
    pub retention_class: String,
    pub generation: u64,
    pub owner: WorkspaceLeaseOwner,
    pub operation: WorkspaceLeaseOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceLeaseDisposition {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceLeasePlan {
    pub disposition: WorkspaceLeaseDisposition,
    pub reason: WorkspaceReasonCode,
    pub next: Option<WorkspaceLeaseRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceEntryKind {
    Directory,
    File,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceEntryObservation {
    pub relative_path: String,
    pub kind: WorkspaceEntryKind,
    pub size_bytes: u64,
    pub content_digest_blake3: Option<String>,
    pub symlink_target: Option<String>,
    pub contains_secret: bool,
    pub contains_host_path: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceUsage {
    pub bytes: u64,
    pub files: u32,
    pub snapshots: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceContentPlan {
    pub usage: WorkspaceUsage,
    pub scrub_paths: Vec<String>,
    pub snapshot_allowed: bool,
    pub quarantine_required: bool,
    pub reasons: Vec<WorkspaceReasonCode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSnapshotEntry {
    pub relative_path: String,
    pub kind: WorkspaceEntryKind,
    pub size_bytes: u64,
    pub content_digest_blake3: Option<String>,
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSnapshotManifest {
    pub schema: String,
    pub object_ref: String,
    pub compatibility_digest_blake3: String,
    pub guest_path: String,
    pub entries: Vec<WorkspaceSnapshotEntry>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceRetentionState {
    Active,
    Idle,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRetentionRecord {
    pub workspace_id: String,
    pub state: WorkspaceRetentionState,
    pub creation_generation: u64,
    pub last_used_generation: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRetentionPlan {
    pub preserve: Vec<String>,
    pub evict: Vec<String>,
    pub quarantine_evict: Vec<String>,
    pub reason: WorkspaceReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceClaimClass {
    StrongDeclaredInputs,
    PracticalMutableHistory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceClaimPlan {
    pub class: WorkspaceClaimClass,
    pub shared_action_publish_allowed: bool,
    pub strong_shared_reuse_allowed: bool,
    pub original_execution_hermetic: bool,
    pub clean_comparison_evidence_accepted: bool,
    pub reason: WorkspaceReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceOutputIdentity {
    pub name: String,
    pub object_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCleanComparisonPlan {
    pub matched: bool,
    pub warm_outputs: Vec<WorkspaceOutputIdentity>,
    pub clean_outputs: Vec<WorkspaceOutputIdentity>,
    pub original_execution_hermetic: bool,
    pub reason: WorkspaceReasonCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceReasonCode {
    Accepted,
    ExplicitFallbackToNone,
    ModeUnavailable,
    PolicyInvalid,
    GuestPathInvalid,
    WorkspaceIdInvalid,
    SnapshotRefInvalid,
    CompatibilityMismatch,
    ToolchainMismatch,
    WorkerMismatch,
    AuthorityMismatch,
    JobMismatch,
    AttemptMismatch,
    FenceMismatch,
    StaleFence,
    ConcurrentOwner,
    LeaseNotActive,
    LeaseQuarantined,
    QuotaExceeded,
    ArithmeticOverflow,
    PathInvalid,
    PathDuplicate,
    SymlinkEscape,
    SymlinkTargetMissing,
    SecretDetected,
    HostPathDetected,
    ScrubRequired,
    SnapshotDisabled,
    SnapshotRejected,
    RetentionUnchanged,
    RetentionEvictionPlanned,
    MutableHistoryDowngrade,
    CleanComparisonMatched,
    CleanComparisonDiverged,
}

impl WorkspaceReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::ExplicitFallbackToNone => "explicit-fallback-to-none",
            Self::ModeUnavailable => "mode-unavailable",
            Self::PolicyInvalid => "policy-invalid",
            Self::GuestPathInvalid => "guest-path-invalid",
            Self::WorkspaceIdInvalid => "workspace-id-invalid",
            Self::SnapshotRefInvalid => "snapshot-ref-invalid",
            Self::CompatibilityMismatch => "compatibility-mismatch",
            Self::ToolchainMismatch => "toolchain-mismatch",
            Self::WorkerMismatch => "worker-mismatch",
            Self::AuthorityMismatch => "authority-mismatch",
            Self::JobMismatch => "job-mismatch",
            Self::AttemptMismatch => "attempt-mismatch",
            Self::FenceMismatch => "fence-mismatch",
            Self::StaleFence => "stale-fence",
            Self::ConcurrentOwner => "concurrent-owner",
            Self::LeaseNotActive => "lease-not-active",
            Self::LeaseQuarantined => "lease-quarantined",
            Self::QuotaExceeded => "quota-exceeded",
            Self::ArithmeticOverflow => "arithmetic-overflow",
            Self::PathInvalid => "path-invalid",
            Self::PathDuplicate => "path-duplicate",
            Self::SymlinkEscape => "symlink-escape",
            Self::SymlinkTargetMissing => "symlink-target-missing",
            Self::SecretDetected => "secret-detected",
            Self::HostPathDetected => "host-path-detected",
            Self::ScrubRequired => "scrub-required",
            Self::SnapshotDisabled => "snapshot-disabled",
            Self::SnapshotRejected => "snapshot-rejected",
            Self::RetentionUnchanged => "retention-unchanged",
            Self::RetentionEvictionPlanned => "retention-eviction-planned",
            Self::MutableHistoryDowngrade => "mutable-history-downgrade",
            Self::CleanComparisonMatched => "clean-comparison-matched",
            Self::CleanComparisonDiverged => "clean-comparison-diverged",
        }
    }
}

pub fn validate_workspace_policy(policy: &WorkspacePolicy) -> Result<(), WorkspaceReasonCode> {
    if policy.schema != WORKSPACE_POLICY_SCHEMA {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    validate_guest_path(&policy.guest_path)?;
    validate_quota(&policy.quota)?;
    validate_retention(&policy.retention)?;
    validate_scrub(&policy.scrub)?;
    validate_compatibility(&policy.compatibility)?;
    match policy.mode {
        WorkspaceMode::None => validate_none_policy(policy),
        WorkspaceMode::ImmutableSnapshot => validate_snapshot_policy(policy),
        WorkspaceMode::MutableSession => validate_mutable_policy(policy),
    }
}

pub fn derive_workspace_compatibility_digest(
    facts: &WorkspaceCompatibilityFacts,
) -> Result<String, WorkspaceReasonCode> {
    validate_compatibility(facts)?;
    let normalized = WorkspaceCompatibilityFacts {
        authority_class: facts.authority_class.clone(),
        action_class: facts.action_class.clone(),
        toolchain_refs: sorted_unique(facts.toolchain_refs.clone()),
    };
    let bytes = serde_json::to_vec(&normalized).map_err(|_| WorkspaceReasonCode::PolicyInvalid)?;
    let mut hasher = blake3::Hasher::new();
    hash_part(&mut hasher, COMPATIBILITY_DIGEST_DOMAIN)?;
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(is_blake3_hex(&digest));
    Ok(digest)
}

pub fn plan_workspace_admission(
    policy: &WorkspacePolicy,
    availability: WorkspaceAvailability,
) -> WorkspaceAdmissionPlan {
    assert!(!WORKSPACE_POLICY_SCHEMA.is_empty(), "workspace policy schema must not be empty");
    assert!(!DEFAULT_WORKSPACE_GUEST_PATH.is_empty(), "default workspace guest path must not be empty");
    if validate_workspace_policy(policy).is_err() {
        return rejected_admission(policy.mode, WorkspaceReasonCode::PolicyInvalid);
    }
    if policy.mode == WorkspaceMode::None || availability == WorkspaceAvailability::Available {
        return WorkspaceAdmissionPlan {
            requested_mode: policy.mode,
            actual_mode: policy.mode,
            disposition: WorkspaceAdmissionDisposition::Accepted,
            reason: WorkspaceReasonCode::Accepted,
        };
    }
    if policy.fallback == WorkspaceFallbackPolicy::ToNone {
        return WorkspaceAdmissionPlan {
            requested_mode: policy.mode,
            actual_mode: WorkspaceMode::None,
            disposition: WorkspaceAdmissionDisposition::ExplicitFallback,
            reason: WorkspaceReasonCode::ExplicitFallbackToNone,
        };
    }
    rejected_admission(policy.mode, WorkspaceReasonCode::ModeUnavailable)
}

pub fn plan_workspace_lease(
    current: Option<&WorkspaceLeaseRecord>,
    request: &WorkspaceLeaseRequest,
) -> WorkspaceLeasePlan {
    if validate_lease_request(request).is_err() {
        return rejected_lease(current, WorkspaceReasonCode::PolicyInvalid);
    }
    match request.operation {
        WorkspaceLeaseOperation::Acquire => plan_lease_acquire(current, request),
        WorkspaceLeaseOperation::Renew => plan_lease_renew(current, request),
        WorkspaceLeaseOperation::Release => plan_lease_release(current, request),
        WorkspaceLeaseOperation::Quarantine => plan_lease_quarantine(current, request),
    }
}

pub fn plan_workspace_content(
    observations: &[WorkspaceEntryObservation],
    quota: &WorkspaceQuotaPolicy,
    scrub: &WorkspaceScrubPolicy,
    snapshot: &WorkspaceSnapshotPolicy,
) -> WorkspaceContentPlan {
    let mut reasons = Vec::with_capacity(observations.len());
    let mut scrub_paths = Vec::with_capacity(observations.len());
    assert!(reasons.capacity() >= observations.len());
    assert!(scrub_paths.capacity() >= observations.len());
    let usage = observe_usage(observations, &mut reasons);
    validate_content_paths(observations, scrub, &mut scrub_paths, &mut reasons);
    if !usage_within_quota(&usage, quota) {
        reasons.push(WorkspaceReasonCode::QuotaExceeded);
    }
    reasons.sort();
    reasons.dedup();
    scrub_paths.sort();
    scrub_paths.dedup();
    let is_unsafe_reason = reasons.iter().any(|reason| {
        matches!(
            reason,
            WorkspaceReasonCode::ArithmeticOverflow
                | WorkspaceReasonCode::HostPathDetected
                | WorkspaceReasonCode::PathDuplicate
                | WorkspaceReasonCode::PathInvalid
                | WorkspaceReasonCode::SecretDetected
                | WorkspaceReasonCode::SymlinkEscape
                | WorkspaceReasonCode::SymlinkTargetMissing
                | WorkspaceReasonCode::QuotaExceeded
        )
    });
    let is_scrub_blocked = snapshot.require_clean_scrub && !scrub_paths.is_empty();
    WorkspaceContentPlan {
        usage,
        scrub_paths,
        snapshot_allowed: snapshot.enabled && !is_unsafe_reason && !is_scrub_blocked,
        quarantine_required: is_unsafe_reason,
        reasons,
    }
}

pub fn build_workspace_snapshot_manifest(
    compatibility_digest_blake3: String,
    guest_path: String,
    observations: Vec<WorkspaceEntryObservation>,
    content_plan: &WorkspaceContentPlan,
) -> Result<WorkspaceSnapshotManifest, WorkspaceReasonCode> {
    assert_eq!(
        u32::try_from(MAX_WORKSPACE_SNAPSHOT_ENTRIES),
        Ok(DEFAULT_WORKSPACE_FILES_MAX),
        "snapshot entry and workspace file bounds must stay aligned"
    );
    if !content_plan.snapshot_allowed || content_plan.quarantine_required {
        return Err(WorkspaceReasonCode::SnapshotRejected);
    }
    if !is_blake3_hex(&compatibility_digest_blake3) {
        return Err(WorkspaceReasonCode::CompatibilityMismatch);
    }
    validate_guest_path(&guest_path)?;
    let observation_count = observations.len();
    if observation_count > MAX_WORKSPACE_SNAPSHOT_ENTRIES {
        return Err(WorkspaceReasonCode::QuotaExceeded);
    }
    let mut entries = observations.into_iter().map(snapshot_entry).collect::<Result<Vec<_>, WorkspaceReasonCode>>()?;
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let hashable = serde_json::json!({
        "schema": WORKSPACE_SNAPSHOT_SCHEMA,
        "compatibility_digest_blake3": compatibility_digest_blake3,
        "guest_path": guest_path,
        "entries": entries,
        "total_bytes": content_plan.usage.bytes,
    });
    let bytes = serde_json::to_vec(&hashable).map_err(|_| WorkspaceReasonCode::SnapshotRejected)?;
    let mut hasher = blake3::Hasher::new();
    hash_part(&mut hasher, SNAPSHOT_DIGEST_DOMAIN)?;
    hasher.update(&bytes);
    let object_ref = format!("{WORKSPACE_SNAPSHOT_REF_PREFIX}{}", hasher.finalize().to_hex());
    assert_eq!(entries.len(), observation_count, "snapshot entries must preserve observation count");
    assert!(content_plan.snapshot_allowed, "snapshot manifest requires an admitted content plan");
    Ok(WorkspaceSnapshotManifest {
        schema: WORKSPACE_SNAPSHOT_SCHEMA.to_string(),
        object_ref,
        compatibility_digest_blake3,
        guest_path,
        entries,
        total_bytes: content_plan.usage.bytes,
    })
}

pub fn plan_workspace_retention(
    records: &[WorkspaceRetentionRecord],
    current_generation: u64,
    policy: &WorkspaceRetentionPolicy,
) -> Result<WorkspaceRetentionPlan, WorkspaceReasonCode> {
    validate_retention(policy)?;
    if records.len() > MAX_WORKSPACE_RETENTION_RECORDS {
        return Err(WorkspaceReasonCode::QuotaExceeded);
    }
    validate_retention_records(records, current_generation)?;
    assert!(records.len() <= MAX_WORKSPACE_RETENTION_RECORDS, "retention records must stay bounded");
    assert!(
        records.iter().all(|record| record.last_used_generation <= current_generation),
        "validated retention records must not be from the future"
    );
    let mut preserve = Vec::with_capacity(records.len());
    let mut idle_candidates = Vec::with_capacity(records.len());
    let mut quarantine_candidates = Vec::with_capacity(records.len());
    for record in records {
        match record.state {
            WorkspaceRetentionState::Active => preserve.push(record.workspace_id.clone()),
            WorkspaceRetentionState::Idle => idle_candidates.push(record.clone()),
            WorkspaceRetentionState::Quarantined => quarantine_candidates.push(record.clone()),
        }
    }
    let mut evict = expired_idle_ids(&idle_candidates, current_generation, policy);
    add_count_pressure_evictions(&idle_candidates, policy.workspace_count_max, preserve.len(), &mut evict)?;
    let quarantine_evict = quarantine_pressure_evictions(&quarantine_candidates, policy.quarantine_count_max)?;
    let evicted = evict.iter().chain(quarantine_evict.iter()).cloned().collect::<BTreeSet<_>>();
    preserve.extend(
        records
            .iter()
            .filter(|record| !evicted.contains(&record.workspace_id))
            .map(|r| r.workspace_id.clone()),
    );
    preserve.sort();
    preserve.dedup();
    evict.sort();
    evict.dedup();
    let reason = if evict.is_empty() && quarantine_evict.is_empty() {
        WorkspaceReasonCode::RetentionUnchanged
    } else {
        WorkspaceReasonCode::RetentionEvictionPlanned
    };
    Ok(WorkspaceRetentionPlan {
        preserve,
        evict,
        quarantine_evict,
        reason,
    })
}

pub fn classify_workspace_claim(mode: WorkspaceMode, clean_comparison_matched: bool) -> WorkspaceClaimPlan {
    match mode {
        WorkspaceMode::None | WorkspaceMode::ImmutableSnapshot => WorkspaceClaimPlan {
            class: WorkspaceClaimClass::StrongDeclaredInputs,
            shared_action_publish_allowed: true,
            strong_shared_reuse_allowed: true,
            original_execution_hermetic: true,
            clean_comparison_evidence_accepted: false,
            reason: WorkspaceReasonCode::Accepted,
        },
        WorkspaceMode::MutableSession => WorkspaceClaimPlan {
            class: WorkspaceClaimClass::PracticalMutableHistory,
            shared_action_publish_allowed: false,
            strong_shared_reuse_allowed: false,
            original_execution_hermetic: false,
            clean_comparison_evidence_accepted: clean_comparison_matched,
            reason: WorkspaceReasonCode::MutableHistoryDowngrade,
        },
    }
}

pub fn compare_clean_rebuild_outputs(
    mut warm_outputs: Vec<WorkspaceOutputIdentity>,
    mut clean_outputs: Vec<WorkspaceOutputIdentity>,
) -> WorkspaceCleanComparisonPlan {
    warm_outputs.sort_by(|left, right| left.name.cmp(&right.name).then(left.object_ref.cmp(&right.object_ref)));
    clean_outputs.sort_by(|left, right| left.name.cmp(&right.name).then(left.object_ref.cmp(&right.object_ref)));
    let is_valid = validate_output_set(&warm_outputs) && validate_output_set(&clean_outputs);
    let is_matched = is_valid && warm_outputs == clean_outputs;
    WorkspaceCleanComparisonPlan {
        matched: is_matched,
        warm_outputs,
        clean_outputs,
        original_execution_hermetic: false,
        reason: if is_matched {
            WorkspaceReasonCode::CleanComparisonMatched
        } else {
            WorkspaceReasonCode::CleanComparisonDiverged
        },
    }
}

fn validate_none_policy(policy: &WorkspacePolicy) -> Result<(), WorkspaceReasonCode> {
    if policy.workspace_id.is_some() || policy.snapshot_ref.is_some() {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_snapshot_policy(policy: &WorkspacePolicy) -> Result<(), WorkspaceReasonCode> {
    if policy.workspace_id.is_some() {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    let snapshot_ref = policy.snapshot_ref.as_deref().ok_or(WorkspaceReasonCode::SnapshotRefInvalid)?;
    if !valid_declared_object_ref(snapshot_ref) {
        return Err(WorkspaceReasonCode::SnapshotRefInvalid);
    }
    Ok(())
}

fn validate_mutable_policy(policy: &WorkspacePolicy) -> Result<(), WorkspaceReasonCode> {
    if policy.snapshot_ref.is_some() {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    validate_identity(policy.workspace_id.as_deref().unwrap_or_default())
        .map_err(|_| WorkspaceReasonCode::WorkspaceIdInvalid)
}

fn validate_quota(quota: &WorkspaceQuotaPolicy) -> Result<(), WorkspaceReasonCode> {
    if quota.bytes_max == 0 || quota.files_max == 0 || quota.snapshots_max == 0 {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_retention(policy: &WorkspaceRetentionPolicy) -> Result<(), WorkspaceReasonCode> {
    if policy.workspace_count_max == 0 {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    if policy.idle_generations_max == 0 {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    if policy.age_generations_max == 0 {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    if policy.quarantine_count_max == 0 {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_scrub(policy: &WorkspaceScrubPolicy) -> Result<(), WorkspaceReasonCode> {
    if policy.scan_depth_max == 0 || policy.path_bytes_max == 0 {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    assert!(policy.scan_depth_max > 0);
    assert!(policy.path_bytes_max > 0);
    if policy.sensitive_paths.len() > MAX_WORKSPACE_SENSITIVE_PATHS
        || policy.secret_markers.len() > MAX_WORKSPACE_SECRET_MARKERS
    {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    for path in &policy.sensitive_paths {
        validate_relative_path(path, RelativePathLimits {
            depth_max: policy.scan_depth_max,
            bytes_max: policy.path_bytes_max,
        })?;
    }
    if policy.secret_markers.iter().any(|marker| marker.is_empty()) {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn validate_compatibility(facts: &WorkspaceCompatibilityFacts) -> Result<(), WorkspaceReasonCode> {
    validate_bounded_text(&facts.authority_class, MAX_WORKSPACE_AUTHORITY_BYTES)?;
    validate_bounded_text(&facts.action_class, MAX_WORKSPACE_AUTHORITY_BYTES)?;
    if facts.toolchain_refs.len() > MAX_WORKSPACE_TOOLCHAIN_REFS {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    let normalized = sorted_unique(facts.toolchain_refs.clone());
    if normalized != facts.toolchain_refs || normalized.iter().any(|value| value.is_empty()) {
        return Err(WorkspaceReasonCode::ToolchainMismatch);
    }
    Ok(())
}

fn validate_guest_path(path: &str) -> Result<(), WorkspaceReasonCode> {
    let path = Path::new(path);
    if !path.is_absolute() || path == Path::new("/") {
        return Err(WorkspaceReasonCode::GuestPathInvalid);
    }
    if path.components().any(|component| matches!(component, Component::CurDir | Component::ParentDir)) {
        return Err(WorkspaceReasonCode::GuestPathInvalid);
    }
    if !path.starts_with("/build") {
        return Err(WorkspaceReasonCode::GuestPathInvalid);
    }
    Ok(())
}

fn validate_identity(value: &str) -> Result<(), WorkspaceReasonCode> {
    if value.is_empty() || value.len() > MAX_WORKSPACE_ID_BYTES {
        return Err(WorkspaceReasonCode::WorkspaceIdInvalid);
    }
    if !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')) {
        return Err(WorkspaceReasonCode::WorkspaceIdInvalid);
    }
    Ok(())
}

fn validate_bounded_text(value: &str, max_bytes: usize) -> Result<(), WorkspaceReasonCode> {
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    Ok(())
}

fn valid_declared_object_ref(value: &str) -> bool {
    value.starts_with("mantle-workspace-snapshot://blake3/")
        || value.starts_with("mantle-object://blake3/")
        || (value.starts_with('/') && value.contains("/store/"))
}

fn rejected_admission(mode: WorkspaceMode, reason: WorkspaceReasonCode) -> WorkspaceAdmissionPlan {
    WorkspaceAdmissionPlan {
        requested_mode: mode,
        actual_mode: mode,
        disposition: WorkspaceAdmissionDisposition::Rejected,
        reason,
    }
}

fn validate_lease_request(request: &WorkspaceLeaseRequest) -> Result<(), WorkspaceReasonCode> {
    validate_identity(&request.workspace_id)?;
    if !is_blake3_hex(&request.compatibility_digest_blake3) || request.owner.fence_generation == 0 {
        return Err(WorkspaceReasonCode::PolicyInvalid);
    }
    validate_guest_path(&request.guest_path)?;
    validate_quota(&request.quota)?;
    validate_identity(&request.owner.worker_id)?;
    validate_bounded_text(&request.owner.authority_class, MAX_WORKSPACE_AUTHORITY_BYTES)?;
    validate_identity(&request.owner.job_id)?;
    validate_identity(&request.owner.attempt_id)?;
    if request.toolchain_refs.len() > MAX_WORKSPACE_TOOLCHAIN_REFS
        || sorted_unique(request.toolchain_refs.clone()) != request.toolchain_refs
    {
        return Err(WorkspaceReasonCode::ToolchainMismatch);
    }
    validate_bounded_text(&request.retention_class, MAX_WORKSPACE_AUTHORITY_BYTES)
}

fn plan_lease_acquire(current: Option<&WorkspaceLeaseRecord>, request: &WorkspaceLeaseRequest) -> WorkspaceLeasePlan {
    assert_eq!(request.operation, WorkspaceLeaseOperation::Acquire, "acquire planner requires acquire operation");
    assert!(!request.workspace_id.is_empty(), "validated acquire request must name a workspace");
    let Some(current) = current else {
        return accepted_lease(new_lease_record(request));
    };
    if let Some(reason) = lease_static_mismatch(current, request) {
        return rejected_lease(Some(current), reason);
    }
    match current.state {
        WorkspaceLeaseState::Quarantined => rejected_lease(Some(current), WorkspaceReasonCode::LeaseQuarantined),
        WorkspaceLeaseState::Active => {
            if current.owner.as_ref() == Some(&request.owner) {
                accepted_lease(active_record(current.clone(), request.generation))
            } else {
                rejected_lease(Some(current), classify_owner_mismatch(current.owner.as_ref(), &request.owner))
            }
        }
        WorkspaceLeaseState::Idle => accepted_lease(WorkspaceLeaseRecord {
            state: WorkspaceLeaseState::Active,
            owner: Some(request.owner.clone()),
            last_used_generation: request.generation,
            ..current.clone()
        }),
    }
}

fn plan_lease_renew(current: Option<&WorkspaceLeaseRecord>, request: &WorkspaceLeaseRequest) -> WorkspaceLeasePlan {
    let Some(current) = current else {
        return rejected_lease(None, WorkspaceReasonCode::LeaseNotActive);
    };
    if let Some(reason) = lease_static_mismatch(current, request) {
        return rejected_lease(Some(current), reason);
    }
    if current.state != WorkspaceLeaseState::Active {
        return rejected_lease(Some(current), WorkspaceReasonCode::LeaseNotActive);
    }
    if current.owner.as_ref() != Some(&request.owner) {
        return rejected_lease(Some(current), classify_owner_mismatch(current.owner.as_ref(), &request.owner));
    }
    accepted_lease(active_record(current.clone(), request.generation))
}

fn plan_lease_release(current: Option<&WorkspaceLeaseRecord>, request: &WorkspaceLeaseRequest) -> WorkspaceLeasePlan {
    let Some(current) = current else {
        return rejected_lease(None, WorkspaceReasonCode::LeaseNotActive);
    };
    if current.state != WorkspaceLeaseState::Active {
        return rejected_lease(Some(current), WorkspaceReasonCode::LeaseNotActive);
    }
    if current.owner.as_ref() != Some(&request.owner) {
        return rejected_lease(Some(current), classify_owner_mismatch(current.owner.as_ref(), &request.owner));
    }
    accepted_lease(WorkspaceLeaseRecord {
        state: WorkspaceLeaseState::Idle,
        owner: None,
        last_used_generation: request.generation,
        ..current.clone()
    })
}

fn plan_lease_quarantine(
    current: Option<&WorkspaceLeaseRecord>,
    request: &WorkspaceLeaseRequest,
) -> WorkspaceLeasePlan {
    let Some(current) = current else {
        return rejected_lease(None, WorkspaceReasonCode::LeaseNotActive);
    };
    if current.state == WorkspaceLeaseState::Quarantined {
        return accepted_lease(current.clone());
    }
    if current.owner.as_ref() != Some(&request.owner) {
        return rejected_lease(Some(current), classify_owner_mismatch(current.owner.as_ref(), &request.owner));
    }
    accepted_lease(WorkspaceLeaseRecord {
        state: WorkspaceLeaseState::Quarantined,
        owner: None,
        last_used_generation: request.generation,
        ..current.clone()
    })
}

fn new_lease_record(request: &WorkspaceLeaseRequest) -> WorkspaceLeaseRecord {
    WorkspaceLeaseRecord {
        workspace_id: request.workspace_id.clone(),
        compatibility_digest_blake3: request.compatibility_digest_blake3.clone(),
        toolchain_refs: request.toolchain_refs.clone(),
        guest_path: request.guest_path.clone(),
        quota: request.quota.clone(),
        retention_class: request.retention_class.clone(),
        creation_generation: request.generation,
        last_used_generation: request.generation,
        state: WorkspaceLeaseState::Active,
        owner: Some(request.owner.clone()),
    }
}

fn active_record(mut current: WorkspaceLeaseRecord, generation: u64) -> WorkspaceLeaseRecord {
    current.last_used_generation = generation;
    current
}

fn lease_static_mismatch(
    current: &WorkspaceLeaseRecord,
    request: &WorkspaceLeaseRequest,
) -> Option<WorkspaceReasonCode> {
    if current.workspace_id != request.workspace_id
        || current.compatibility_digest_blake3 != request.compatibility_digest_blake3
    {
        return Some(WorkspaceReasonCode::CompatibilityMismatch);
    }
    if current.toolchain_refs != request.toolchain_refs {
        return Some(WorkspaceReasonCode::ToolchainMismatch);
    }
    if current.guest_path != request.guest_path
        || current.quota != request.quota
        || current.retention_class != request.retention_class
    {
        return Some(WorkspaceReasonCode::PolicyInvalid);
    }
    None
}

fn classify_owner_mismatch(
    current: Option<&WorkspaceLeaseOwner>,
    requested: &WorkspaceLeaseOwner,
) -> WorkspaceReasonCode {
    assert!(!requested.worker_id.is_empty(), "requested lease owner must name a worker");
    assert!(requested.fence_generation > 0, "requested lease fence generation must be positive");
    let Some(current) = current else {
        return WorkspaceReasonCode::LeaseNotActive;
    };
    if current.worker_id != requested.worker_id {
        return WorkspaceReasonCode::WorkerMismatch;
    }
    if current.authority_class != requested.authority_class {
        return WorkspaceReasonCode::AuthorityMismatch;
    }
    if current.job_id != requested.job_id {
        return WorkspaceReasonCode::JobMismatch;
    }
    if current.attempt_id != requested.attempt_id {
        return WorkspaceReasonCode::AttemptMismatch;
    }
    if requested.fence_generation < current.fence_generation {
        return WorkspaceReasonCode::StaleFence;
    }
    if requested.fence_generation != current.fence_generation {
        return WorkspaceReasonCode::FenceMismatch;
    }
    WorkspaceReasonCode::ConcurrentOwner
}

fn accepted_lease(next: WorkspaceLeaseRecord) -> WorkspaceLeasePlan {
    WorkspaceLeasePlan {
        disposition: WorkspaceLeaseDisposition::Accepted,
        reason: WorkspaceReasonCode::Accepted,
        next: Some(next),
    }
}

fn rejected_lease(current: Option<&WorkspaceLeaseRecord>, reason: WorkspaceReasonCode) -> WorkspaceLeasePlan {
    WorkspaceLeasePlan {
        disposition: WorkspaceLeaseDisposition::Rejected,
        reason,
        next: current.cloned(),
    }
}

fn observe_usage(observations: &[WorkspaceEntryObservation], reasons: &mut Vec<WorkspaceReasonCode>) -> WorkspaceUsage {
    let files = match u32::try_from(observations.len()) {
        Ok(count) => count,
        Err(_) => {
            reasons.push(WorkspaceReasonCode::ArithmeticOverflow);
            WORKSPACE_FILES_OVER_LIMIT
        }
    };
    let mut bytes = 0u64;
    for observation in observations {
        bytes = match bytes.checked_add(observation.size_bytes) {
            Some(total) => total,
            None => {
                reasons.push(WorkspaceReasonCode::ArithmeticOverflow);
                bytes.saturating_add(observation.size_bytes)
            }
        };
    }
    let usage = WorkspaceUsage {
        bytes,
        files,
        snapshots: 0,
    };
    assert_eq!(usage.files, files, "workspace usage must retain the observed file count");
    assert_eq!(usage.bytes, bytes, "workspace usage must retain the observed byte count");
    usage
}

fn validate_content_paths(
    observations: &[WorkspaceEntryObservation],
    scrub: &WorkspaceScrubPolicy,
    scrub_paths: &mut Vec<String>,
    reasons: &mut Vec<WorkspaceReasonCode>,
) {
    let scrub_path_count_before = scrub_paths.len();
    let reason_count_before = reasons.len();
    let planned = observations.iter().map(|item| item.relative_path.as_str()).collect::<BTreeSet<_>>();
    if planned.len() != observations.len() {
        reasons.push(WorkspaceReasonCode::PathDuplicate);
    }
    for observation in observations {
        if validate_relative_path(&observation.relative_path, RelativePathLimits {
            depth_max: scrub.scan_depth_max,
            bytes_max: scrub.path_bytes_max,
        })
        .is_err()
        {
            reasons.push(WorkspaceReasonCode::PathInvalid);
            continue;
        }
        if scrub.sensitive_paths.iter().any(|path| path == &observation.relative_path) {
            scrub_paths.push(observation.relative_path.clone());
        }
        if observation.contains_secret {
            reasons.push(WorkspaceReasonCode::SecretDetected);
        }
        if scrub.reject_host_paths && observation.contains_host_path {
            reasons.push(WorkspaceReasonCode::HostPathDetected);
        }
        validate_observed_symlink(observation, &planned, reasons);
    }
    assert!(scrub_paths.len() >= scrub_path_count_before, "content validation must not remove scrub paths");
    assert!(reasons.len() >= reason_count_before, "content validation must not remove reasons");
}

fn validate_observed_symlink(
    observation: &WorkspaceEntryObservation,
    planned: &BTreeSet<&str>,
    reasons: &mut Vec<WorkspaceReasonCode>,
) {
    if observation.kind != WorkspaceEntryKind::Symlink {
        if observation.symlink_target.is_some() {
            reasons.push(WorkspaceReasonCode::PathInvalid);
        }
        return;
    }
    let Some(target) = observation.symlink_target.as_deref() else {
        reasons.push(WorkspaceReasonCode::PathInvalid);
        return;
    };
    let Some(resolved) = resolve_relative_symlink(&observation.relative_path, target) else {
        reasons.push(WorkspaceReasonCode::SymlinkEscape);
        return;
    };
    if !planned.contains(resolved.as_str()) {
        reasons.push(WorkspaceReasonCode::SymlinkTargetMissing);
    }
}

struct RelativePathLimits {
    depth_max: u32,
    bytes_max: u32,
}

fn validate_relative_path(path: &str, limits: RelativePathLimits) -> Result<(), WorkspaceReasonCode> {
    if path.is_empty() || Path::new(path).is_absolute() {
        return Err(WorkspaceReasonCode::PathInvalid);
    }
    let byte_count = u32::try_from(path.len()).map_err(|_| WorkspaceReasonCode::ArithmeticOverflow)?;
    if byte_count > limits.bytes_max {
        return Err(WorkspaceReasonCode::PathInvalid);
    }
    let mut depth = 0u32;
    for component in Path::new(path).components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(WorkspaceReasonCode::PathInvalid);
        }
        depth = depth.checked_add(1).ok_or(WorkspaceReasonCode::ArithmeticOverflow)?;
        if depth > limits.depth_max {
            return Err(WorkspaceReasonCode::PathInvalid);
        }
    }
    assert!(depth > 0, "validated relative paths must contain a component");
    assert!(depth <= limits.depth_max, "validated relative path depth must stay bounded");
    Ok(())
}

fn resolve_relative_symlink(link_path: &str, target: impl AsRef<str>) -> Option<String> {
    let target = target.as_ref();
    let target_path = Path::new(target);
    if target_path.is_absolute() {
        return None;
    }
    let parent = Path::new(link_path).parent().unwrap_or_else(|| Path::new(""));
    let joined = parent.join(target_path);
    let component_slots = joined.components().count();
    let mut parts = Vec::with_capacity(component_slots);
    for component in joined.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_str()?.to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::Prefix(_) | Component::RootDir => return None,
        }
    }
    if parts.is_empty() {
        return None;
    }
    assert!(!parts.is_empty(), "resolved symlink must contain a path component");
    assert!(parts.len() <= component_slots, "resolved symlink components must stay within capacity");
    Some(parts.join("/"))
}

fn usage_within_quota(usage: &WorkspaceUsage, quota: &WorkspaceQuotaPolicy) -> bool {
    usage.bytes <= quota.bytes_max && usage.files <= quota.files_max && usage.snapshots <= quota.snapshots_max
}

fn snapshot_entry(observation: WorkspaceEntryObservation) -> Result<WorkspaceSnapshotEntry, WorkspaceReasonCode> {
    if observation.contains_secret || observation.contains_host_path {
        return Err(WorkspaceReasonCode::SnapshotRejected);
    }
    if observation.kind == WorkspaceEntryKind::File {
        let digest = observation.content_digest_blake3.as_deref().ok_or(WorkspaceReasonCode::SnapshotRejected)?;
        if !is_blake3_hex(digest) {
            return Err(WorkspaceReasonCode::SnapshotRejected);
        }
    }
    Ok(WorkspaceSnapshotEntry {
        relative_path: observation.relative_path,
        kind: observation.kind,
        size_bytes: observation.size_bytes,
        content_digest_blake3: observation.content_digest_blake3,
        symlink_target: observation.symlink_target,
    })
}

fn validate_retention_records(
    records: &[WorkspaceRetentionRecord],
    current_generation: u64,
) -> Result<(), WorkspaceReasonCode> {
    let mut ids = BTreeSet::new();
    for record in records {
        validate_identity(&record.workspace_id)?;
        if !ids.insert(record.workspace_id.as_str()) {
            return Err(WorkspaceReasonCode::PathDuplicate);
        }
        if record.creation_generation > record.last_used_generation || record.last_used_generation > current_generation
        {
            return Err(WorkspaceReasonCode::PolicyInvalid);
        }
    }
    Ok(())
}

fn expired_idle_ids(
    records: &[WorkspaceRetentionRecord],
    current_generation: u64,
    policy: &WorkspaceRetentionPolicy,
) -> Vec<String> {
    records
        .iter()
        .filter(|record| {
            current_generation.saturating_sub(record.last_used_generation) > policy.idle_generations_max
                || current_generation.saturating_sub(record.creation_generation) > policy.age_generations_max
        })
        .map(|record| record.workspace_id.clone())
        .collect()
}

fn add_count_pressure_evictions(
    idle: &[WorkspaceRetentionRecord],
    count_max: u32,
    active_count: usize,
    evict: &mut Vec<String>,
) -> Result<(), WorkspaceReasonCode> {
    let count_max = usize::try_from(count_max).map_err(|_| WorkspaceReasonCode::ArithmeticOverflow)?;
    let retained_idle_count = idle.iter().filter(|record| !evict.contains(&record.workspace_id)).count();
    let total = active_count.checked_add(retained_idle_count).ok_or(WorkspaceReasonCode::ArithmeticOverflow)?;
    let extra = total.saturating_sub(count_max);
    let mut candidates =
        idle.iter().filter(|record| !evict.contains(&record.workspace_id)).cloned().collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.last_used_generation
            .cmp(&right.last_used_generation)
            .then(left.creation_generation.cmp(&right.creation_generation))
            .then(left.workspace_id.cmp(&right.workspace_id))
    });
    evict.extend(candidates.into_iter().take(extra).map(|record| record.workspace_id));
    Ok(())
}

fn quarantine_pressure_evictions(
    records: &[WorkspaceRetentionRecord],
    count_max: u32,
) -> Result<Vec<String>, WorkspaceReasonCode> {
    let count_max = usize::try_from(count_max).map_err(|_| WorkspaceReasonCode::ArithmeticOverflow)?;
    let extra = records.len().saturating_sub(count_max);
    let mut candidates = records.to_vec();
    candidates.sort_by(|left, right| {
        left.last_used_generation
            .cmp(&right.last_used_generation)
            .then(left.workspace_id.cmp(&right.workspace_id))
    });
    Ok(candidates.into_iter().take(extra).map(|record| record.workspace_id).collect())
}

fn validate_output_set(outputs: &[WorkspaceOutputIdentity]) -> bool {
    if outputs.is_empty() {
        return false;
    }
    let mut names = BTreeSet::new();
    outputs.iter().all(|output| {
        !output.name.is_empty()
            && names.insert(output.name.as_str())
            && (output.object_ref.starts_with("mantle-object://blake3/") || output.object_ref.starts_with('/'))
    })
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn hash_part(hasher: &mut blake3::Hasher, value: &str) -> Result<(), WorkspaceReasonCode> {
    let length_bytes = u64::try_from(value.len()).map_err(|_| WorkspaceReasonCode::ArithmeticOverflow)?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const CURRENT_GENERATION: u64 = 100;
    const STALE_GENERATION: u64 = 1;
    const TEST_BYTES: u64 = 10;

    fn digest(byte: char) -> String {
        byte.to_string().repeat(BLAKE3_HEX_LENGTH)
    }

    fn mutable_policy() -> WorkspacePolicy {
        WorkspacePolicy {
            mode: WorkspaceMode::MutableSession,
            workspace_id: Some("cargo-session".to_string()),
            compatibility: WorkspaceCompatibilityFacts {
                authority_class: "tenant-a".to_string(),
                action_class: "cargo-check".to_string(),
                toolchain_refs: vec!["mantle-object://blake3/rust".to_string()],
            },
            ..WorkspacePolicy::default()
        }
    }

    fn owner(attempt: &str, fence_generation: u64) -> WorkspaceLeaseOwner {
        WorkspaceLeaseOwner {
            worker_id: "worker-a".to_string(),
            authority_class: "tenant-a".to_string(),
            job_id: "job-a".to_string(),
            attempt_id: attempt.to_string(),
            fence_generation,
        }
    }

    fn lease_request(
        operation: WorkspaceLeaseOperation,
        attempt: &str,
        fence_generation: u64,
    ) -> WorkspaceLeaseRequest {
        WorkspaceLeaseRequest {
            workspace_id: "cargo-session".to_string(),
            compatibility_digest_blake3: digest('a'),
            toolchain_refs: vec!["mantle-object://blake3/rust".to_string()],
            guest_path: DEFAULT_WORKSPACE_GUEST_PATH.to_string(),
            quota: WorkspaceQuotaPolicy::default(),
            retention_class: "recent".to_string(),
            generation: CURRENT_GENERATION,
            owner: owner(attempt, fence_generation),
            operation,
        }
    }

    fn file(path: &str, byte: char) -> WorkspaceEntryObservation {
        WorkspaceEntryObservation {
            relative_path: path.to_string(),
            kind: WorkspaceEntryKind::File,
            size_bytes: TEST_BYTES,
            content_digest_blake3: Some(digest(byte)),
            symlink_target: None,
            contains_secret: false,
            contains_host_path: false,
        }
    }

    #[test]
    fn explicit_modes_validate_and_missing_mode_never_silently_falls_back() {
        assert!(validate_workspace_policy(&WorkspacePolicy::default()).is_ok());
        let mut snapshot = WorkspacePolicy {
            mode: WorkspaceMode::ImmutableSnapshot,
            snapshot_ref: Some(format!("{WORKSPACE_SNAPSHOT_REF_PREFIX}{}", digest('b'))),
            ..WorkspacePolicy::default()
        };
        assert!(validate_workspace_policy(&snapshot).is_ok());
        let mutable = mutable_policy();
        assert!(validate_workspace_policy(&mutable).is_ok());

        let rejected = plan_workspace_admission(&mutable, WorkspaceAvailability::Missing);
        assert_eq!(rejected.disposition, WorkspaceAdmissionDisposition::Rejected);
        assert_eq!(rejected.actual_mode, WorkspaceMode::MutableSession);
        snapshot.fallback = WorkspaceFallbackPolicy::ToNone;
        let fallback = plan_workspace_admission(&snapshot, WorkspaceAvailability::Missing);
        assert_eq!(fallback.disposition, WorkspaceAdmissionDisposition::ExplicitFallback);
        assert_eq!(fallback.actual_mode, WorkspaceMode::None);
    }

    #[test]
    fn compatibility_digest_is_stable_and_order_is_explicitly_validated() {
        let facts = mutable_policy().compatibility;
        let first = derive_workspace_compatibility_digest(&facts).unwrap();
        let second = derive_workspace_compatibility_digest(&facts).unwrap();
        assert_eq!(first, second);
        assert!(is_blake3_hex(&first));

        let mut invalid = facts;
        invalid.toolchain_refs = vec!["z".to_string(), "a".to_string()];
        assert_eq!(derive_workspace_compatibility_digest(&invalid), Err(WorkspaceReasonCode::ToolchainMismatch));
    }

    #[test]
    fn compatible_lease_acquire_renew_release_is_deterministic() {
        let acquire = lease_request(WorkspaceLeaseOperation::Acquire, "attempt-a", 1);
        let first = plan_workspace_lease(None, &acquire);
        let repeated = plan_workspace_lease(None, &acquire);
        assert_eq!(first, repeated);
        assert_eq!(first.disposition, WorkspaceLeaseDisposition::Accepted);
        let active = first.next.unwrap();

        let renew = lease_request(WorkspaceLeaseOperation::Renew, "attempt-a", 1);
        let renewed = plan_workspace_lease(Some(&active), &renew);
        assert_eq!(renewed.disposition, WorkspaceLeaseDisposition::Accepted);
        let release = lease_request(WorkspaceLeaseOperation::Release, "attempt-a", 1);
        let released = plan_workspace_lease(renewed.next.as_ref(), &release);
        assert_eq!(released.next.unwrap().state, WorkspaceLeaseState::Idle);
    }

    #[test]
    fn stale_cross_worker_cross_authority_and_concurrent_owners_are_rejected_without_mutation() {
        let active = plan_workspace_lease(None, &lease_request(WorkspaceLeaseOperation::Acquire, "attempt-new", 2))
            .next
            .unwrap();
        let mut cases = Vec::new();
        let mut stale = lease_request(WorkspaceLeaseOperation::Renew, "attempt-new", 1);
        stale.owner.worker_id = active.owner.as_ref().unwrap().worker_id.clone();
        stale.owner.authority_class = active.owner.as_ref().unwrap().authority_class.clone();
        stale.owner.job_id = active.owner.as_ref().unwrap().job_id.clone();
        cases.push((stale, WorkspaceReasonCode::StaleFence));
        let mut foreign_worker = lease_request(WorkspaceLeaseOperation::Acquire, "attempt-new", 2);
        foreign_worker.owner.worker_id = "worker-b".to_string();
        cases.push((foreign_worker, WorkspaceReasonCode::WorkerMismatch));
        let mut foreign_authority = lease_request(WorkspaceLeaseOperation::Acquire, "attempt-new", 2);
        foreign_authority.owner.authority_class = "tenant-b".to_string();
        cases.push((foreign_authority, WorkspaceReasonCode::AuthorityMismatch));
        let concurrent = lease_request(WorkspaceLeaseOperation::Acquire, "attempt-other", 2);
        cases.push((concurrent, WorkspaceReasonCode::AttemptMismatch));

        for (request, expected) in cases {
            let plan = plan_workspace_lease(Some(&active), &request);
            assert_eq!(plan.disposition, WorkspaceLeaseDisposition::Rejected);
            assert_eq!(plan.reason, expected);
            assert_eq!(plan.next.as_ref(), Some(&active));
        }

        let mut wrong_action = lease_request(WorkspaceLeaseOperation::Acquire, "attempt-new", 2);
        wrong_action.compatibility_digest_blake3 = digest('b');
        assert_eq!(
            plan_workspace_lease(Some(&active), &wrong_action).reason,
            WorkspaceReasonCode::CompatibilityMismatch
        );
        let mut wrong_toolchain = lease_request(WorkspaceLeaseOperation::Acquire, "attempt-new", 2);
        wrong_toolchain.toolchain_refs = vec!["mantle-object://blake3/other".to_string()];
        assert_eq!(
            plan_workspace_lease(Some(&active), &wrong_toolchain).reason,
            WorkspaceReasonCode::ToolchainMismatch
        );
    }

    #[test]
    fn invalid_guest_paths_quota_overflow_symlink_escape_secret_and_host_paths_quarantine() {
        let mut invalid = mutable_policy();
        invalid.guest_path = "/tmp/host-workspace".to_string();
        assert_eq!(validate_workspace_policy(&invalid), Err(WorkspaceReasonCode::GuestPathInvalid));

        let mut observations = vec![file("target/cache.bin", 'c')];
        observations.push(WorkspaceEntryObservation {
            relative_path: "target/escape".to_string(),
            kind: WorkspaceEntryKind::Symlink,
            size_bytes: 0,
            content_digest_blake3: None,
            symlink_target: Some("../../outside".to_string()),
            contains_secret: false,
            contains_host_path: false,
        });
        let mut secret = file("target/token.txt", 'd');
        secret.contains_secret = true;
        observations.push(secret);
        let mut host = file("target/path.txt", 'e');
        host.contains_host_path = true;
        observations.push(host);
        let quota = WorkspaceQuotaPolicy {
            bytes_max: 1,
            ..WorkspaceQuotaPolicy::default()
        };
        let plan =
            plan_workspace_content(&observations, &quota, &WorkspaceScrubPolicy::default(), &WorkspaceSnapshotPolicy {
                enabled: true,
                require_clean_scrub: true,
            });
        assert!(plan.quarantine_required);
        assert!(!plan.snapshot_allowed);
        assert!(plan.reasons.contains(&WorkspaceReasonCode::QuotaExceeded));
        assert!(plan.reasons.contains(&WorkspaceReasonCode::SymlinkEscape));
        assert!(plan.reasons.contains(&WorkspaceReasonCode::SecretDetected));
        assert!(plan.reasons.contains(&WorkspaceReasonCode::HostPathDetected));
    }

    #[test]
    fn immutable_snapshot_manifest_is_order_independent_and_content_addressed() {
        let policy = WorkspaceSnapshotPolicy {
            enabled: true,
            require_clean_scrub: true,
        };
        let observations = vec![file("z.bin", 'f'), file("a.bin", 'a')];
        let content = plan_workspace_content(
            &observations,
            &WorkspaceQuotaPolicy::default(),
            &WorkspaceScrubPolicy::default(),
            &policy,
        );
        assert!(content.snapshot_allowed);
        let first = build_workspace_snapshot_manifest(
            digest('b'),
            DEFAULT_WORKSPACE_GUEST_PATH.to_string(),
            observations.clone(),
            &content,
        )
        .unwrap();
        let reversed = observations.into_iter().rev().collect::<Vec<_>>();
        let second_content = plan_workspace_content(
            &reversed,
            &WorkspaceQuotaPolicy::default(),
            &WorkspaceScrubPolicy::default(),
            &policy,
        );
        let second = build_workspace_snapshot_manifest(
            digest('b'),
            DEFAULT_WORKSPACE_GUEST_PATH.to_string(),
            reversed,
            &second_content,
        )
        .unwrap();
        assert_eq!(first, second);
        assert!(first.object_ref.starts_with(WORKSPACE_SNAPSHOT_REF_PREFIX));
    }

    #[test]
    fn retention_preserves_active_workspaces_and_evicts_old_idle_and_bounded_quarantine() {
        let policy = WorkspaceRetentionPolicy {
            workspace_count_max: 2,
            idle_generations_max: 10,
            age_generations_max: 50,
            quarantine_count_max: 1,
        };
        let records = vec![
            WorkspaceRetentionRecord {
                workspace_id: "active".to_string(),
                state: WorkspaceRetentionState::Active,
                creation_generation: STALE_GENERATION,
                last_used_generation: CURRENT_GENERATION,
                size_bytes: TEST_BYTES,
            },
            WorkspaceRetentionRecord {
                workspace_id: "idle-old".to_string(),
                state: WorkspaceRetentionState::Idle,
                creation_generation: STALE_GENERATION,
                last_used_generation: STALE_GENERATION,
                size_bytes: TEST_BYTES,
            },
            WorkspaceRetentionRecord {
                workspace_id: "quarantine-old".to_string(),
                state: WorkspaceRetentionState::Quarantined,
                creation_generation: STALE_GENERATION,
                last_used_generation: STALE_GENERATION,
                size_bytes: TEST_BYTES,
            },
            WorkspaceRetentionRecord {
                workspace_id: "quarantine-new".to_string(),
                state: WorkspaceRetentionState::Quarantined,
                creation_generation: 90,
                last_used_generation: 90,
                size_bytes: TEST_BYTES,
            },
        ];
        let plan = plan_workspace_retention(&records, CURRENT_GENERATION, &policy).unwrap();
        assert!(plan.preserve.contains(&"active".to_string()));
        assert!(!plan.evict.contains(&"active".to_string()));
        assert!(plan.evict.contains(&"idle-old".to_string()));
        assert_eq!(plan.quarantine_evict, vec!["quarantine-old".to_string()]);
    }

    #[test]
    fn mutable_claim_never_publishes_or_satisfies_strong_reuse_even_after_clean_match() {
        let without_comparison = classify_workspace_claim(WorkspaceMode::MutableSession, false);
        let with_comparison = classify_workspace_claim(WorkspaceMode::MutableSession, true);
        assert!(!without_comparison.shared_action_publish_allowed);
        assert!(!without_comparison.strong_shared_reuse_allowed);
        assert!(!with_comparison.shared_action_publish_allowed);
        assert!(!with_comparison.strong_shared_reuse_allowed);
        assert!(!with_comparison.original_execution_hermetic);
        assert!(with_comparison.clean_comparison_evidence_accepted);
    }

    #[test]
    fn clean_comparison_records_only_output_set_agreement_and_divergence() {
        let warm = vec![WorkspaceOutputIdentity {
            name: "out".to_string(),
            object_ref: format!("mantle-object://blake3/{}", digest('a')),
        }];
        let matched = compare_clean_rebuild_outputs(warm.clone(), warm.clone());
        assert!(matched.matched);
        assert!(!matched.original_execution_hermetic);
        let mut different = warm.clone();
        different[0].object_ref = format!("mantle-object://blake3/{}", digest('b'));
        let diverged = compare_clean_rebuild_outputs(warm, different);
        assert!(!diverged.matched);
        assert_eq!(diverged.reason, WorkspaceReasonCode::CleanComparisonDiverged);
    }

    #[test]
    fn unknown_mode_and_overflow_are_rejected() {
        let unknown = serde_json::from_str::<WorkspaceMode>("\"ambient-magic\"");
        assert!(unknown.is_err());
        let observations = vec![
            WorkspaceEntryObservation {
                size_bytes: u64::MAX,
                ..file("a", 'a')
            },
            WorkspaceEntryObservation {
                size_bytes: TEST_BYTES,
                ..file("b", 'b')
            },
        ];
        let plan = plan_workspace_content(
            &observations,
            &WorkspaceQuotaPolicy::default(),
            &WorkspaceScrubPolicy::default(),
            &WorkspaceSnapshotPolicy::default(),
        );
        assert!(plan.quarantine_required);
        assert!(plan.reasons.contains(&WorkspaceReasonCode::ArithmeticOverflow));
    }
}
