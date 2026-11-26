use std::path::PathBuf;
use std::time::Duration;

/// Default local sandbox wall-clock deadline (1 hour).
pub const DEFAULT_LOCAL_WALL_DEADLINE_MS: u64 = 3_600_000;
/// Default finite grace window for external supervisor shutdown before SIGKILL escalation.
pub const DEFAULT_SHUTDOWN_GRACE_MS: u64 = 2_000;
/// Minimum supported finite grace window.
pub const MIN_SHUTDOWN_GRACE_MS: u64 = 20;
/// Maximum supported finite grace window.
pub const MAX_SHUTDOWN_GRACE_MS: u64 = 60_000;
/// Minimum hard memory cap that can support bwrap + static shell + builder runtime.
pub const MIN_SANDBOX_MEMORY_LIMIT_BYTES: u64 = 4 * 1024 * 1024;
/// Maximum supported hard memory cap (1 TiB).
pub const MAX_SANDBOX_MEMORY_LIMIT_BYTES: u64 = 1_099_511_627_776;
/// Minimum process count allowing bwrap + PID1 + shell + builder child.
pub const MIN_SANDBOX_PIDS_LIMIT_COUNT: u32 = 4;
/// Maximum process count cap.
pub const MAX_SANDBOX_PIDS_LIMIT_COUNT: u32 = 65_536;

/// Explicit monotonic wall-clock deadline policy for local sandbox execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxWallDeadline {
    Bounded {
        wall_deadline_ms: u64,
    },
    /// Explicit unlimited opt-out (never encoded via a 0 sentinel).
    Unlimited,
}

impl SandboxWallDeadline {
    pub const fn as_duration(self) -> Option<Duration> {
        match self {
            Self::Bounded { wall_deadline_ms } => Some(Duration::from_millis(wall_deadline_ms)),
            Self::Unlimited => None,
        }
    }
}

/// Optional silence (output-inactivity) timeout policy, disabled by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxSilenceBudget {
    Disabled,
    Bounded { silence_timeout_ms: u64 },
}

impl SandboxSilenceBudget {
    pub const fn as_duration(self) -> Option<Duration> {
        match self {
            Self::Disabled => None,
            Self::Bounded { silence_timeout_ms } => Some(Duration::from_millis(silence_timeout_ms)),
        }
    }
}

/// Staged enforcement requirement for local sandbox resource controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxEnforcementMode {
    Disabled,
    BestEffort,
    Required,
}

/// Swap policy applied alongside cgroup-v2 `memory.max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxSwapPolicy {
    HostDefault,
    Zero,
    BoundedBytes { swap_limit_bytes: u64 },
}

/// Policy for helper/daemon Unix sockets mounted into a resource-enforced sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxHelperScopePolicy {
    /// Refuse any attached helper socket whose peer is outside the attempt cgroup.
    RequireContained,
    /// Refuse external stream/datagram peers outside the attempt cgroup, while
    /// allowing an in-process passive `SOCK_DGRAM` telemetry socket owned by this process.
    AllowInProcessPassiveDatagram,
}

/// Staged capability-aware resource-budget configuration for local sandboxes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxResourcePolicy {
    pub cgroup_root: Option<PathBuf>,
    pub memory_mode: SandboxEnforcementMode,
    pub memory_limit_bytes: Option<u64>,
    pub swap_policy: SandboxSwapPolicy,
    pub pids_mode: SandboxEnforcementMode,
    pub pids_limit_count: Option<u32>,
    pub scratch_mode: SandboxEnforcementMode,
    pub scratch_limit_bytes: Option<u64>,
    pub helper_scope_policy: SandboxHelperScopePolicy,
}

impl SandboxResourcePolicy {
    pub const fn disabled() -> Self {
        Self {
            cgroup_root: None,
            memory_mode: SandboxEnforcementMode::Disabled,
            memory_limit_bytes: None,
            swap_policy: SandboxSwapPolicy::Zero,
            pids_mode: SandboxEnforcementMode::Disabled,
            pids_limit_count: None,
            scratch_mode: SandboxEnforcementMode::Disabled,
            scratch_limit_bytes: None,
            helper_scope_policy: SandboxHelperScopePolicy::RequireContained,
        }
    }

    pub fn requests_any_enforcement(&self) -> bool {
        self.memory_mode != SandboxEnforcementMode::Disabled
            || self.pids_mode != SandboxEnforcementMode::Disabled
            || self.scratch_mode != SandboxEnforcementMode::Disabled
    }

    pub fn requests_cgroup(&self) -> bool {
        self.memory_mode != SandboxEnforcementMode::Disabled || self.pids_mode != SandboxEnforcementMode::Disabled
    }

    pub fn requires_cgroup(&self) -> bool {
        self.memory_mode == SandboxEnforcementMode::Required || self.pids_mode == SandboxEnforcementMode::Required
    }
}

impl Default for SandboxResourcePolicy {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Complete runtime execution policy for [`crate::buildservice::BubblewrapBuildService`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxExecutionPolicy {
    pub wall_deadline: SandboxWallDeadline,
    pub silence_budget: SandboxSilenceBudget,
    pub shutdown_grace_ms: u64,
    pub resources: SandboxResourcePolicy,
}

impl SandboxExecutionPolicy {
    pub const fn standard() -> Self {
        Self {
            wall_deadline: SandboxWallDeadline::Bounded {
                wall_deadline_ms: DEFAULT_LOCAL_WALL_DEADLINE_MS,
            },
            silence_budget: SandboxSilenceBudget::Disabled,
            shutdown_grace_ms: DEFAULT_SHUTDOWN_GRACE_MS,
            resources: SandboxResourcePolicy::disabled(),
        }
    }

    pub const fn unlimited() -> Self {
        Self {
            wall_deadline: SandboxWallDeadline::Unlimited,
            silence_budget: SandboxSilenceBudget::Disabled,
            shutdown_grace_ms: DEFAULT_SHUTDOWN_GRACE_MS,
            resources: SandboxResourcePolicy::disabled(),
        }
    }

    pub fn validate(&self) -> std::io::Result<()> {
        if let SandboxWallDeadline::Bounded { wall_deadline_ms } = self.wall_deadline
            && wall_deadline_ms == 0
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "wall_deadline_ms must be positive; use SandboxWallDeadline::Unlimited to opt out",
            ));
        }
        if let SandboxSilenceBudget::Bounded { silence_timeout_ms } = self.silence_budget
            && silence_timeout_ms == 0
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "silence_timeout_ms must be positive; use SandboxSilenceBudget::Disabled to opt out",
            ));
        }
        if self.shutdown_grace_ms < MIN_SHUTDOWN_GRACE_MS || self.shutdown_grace_ms > MAX_SHUTDOWN_GRACE_MS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "shutdown_grace_ms {} must be within {}..={}",
                    self.shutdown_grace_ms, MIN_SHUTDOWN_GRACE_MS, MAX_SHUTDOWN_GRACE_MS
                ),
            ));
        }
        if self.resources.memory_mode != SandboxEnforcementMode::Disabled {
            let Some(limit) = self.resources.memory_limit_bytes else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "memory_limit_bytes must be set when memory enforcement is enabled",
                ));
            };
            if !(MIN_SANDBOX_MEMORY_LIMIT_BYTES..=MAX_SANDBOX_MEMORY_LIMIT_BYTES).contains(&limit) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "memory_limit_bytes {limit} must be within {MIN_SANDBOX_MEMORY_LIMIT_BYTES}..={MAX_SANDBOX_MEMORY_LIMIT_BYTES}"
                    ),
                ));
            }
        }
        if self.resources.pids_mode != SandboxEnforcementMode::Disabled {
            let Some(limit) = self.resources.pids_limit_count else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "pids_limit_count must be set when pids enforcement is enabled",
                ));
            };
            if !(MIN_SANDBOX_PIDS_LIMIT_COUNT..=MAX_SANDBOX_PIDS_LIMIT_COUNT).contains(&limit) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "pids_limit_count {limit} must be within {MIN_SANDBOX_PIDS_LIMIT_COUNT}..={MAX_SANDBOX_PIDS_LIMIT_COUNT}"
                    ),
                ));
            }
        }
        if self.resources.scratch_mode != SandboxEnforcementMode::Disabled {
            let Some(limit) = self.resources.scratch_limit_bytes else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "scratch_limit_bytes must be set when scratch enforcement is enabled",
                ));
            };
            if limit == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "scratch_limit_bytes must be positive",
                ));
            }
        }
        Ok(())
    }
}

impl Default for SandboxExecutionPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

/// Typed, honest reason why a requested resource control could not be enforced.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxUnavailableReason {
    CgroupRootUnconfigured,
    CgroupRootInvalidOrUnowned { detail: String },
    DelegationLcaNotWritable { self_cgroup: String, target_cgroup: String },
    ControllerDisabled { controller: String },
    ControllerSubtreeBlocked { controller: String, detail: String },
    KernelFeatureUnsupported { feature: String },
    MissingSandboxBinaryOrShell { binary: String },
    ScratchBindDirUnenforced,
    HelperDaemonOutsideEnforcedScope { socket_kind: String, detail: String },
    ChildMigrationFailed { detail: String },
    SetupIoError { detail: String },
}

/// Effective kernel-applied cgroup limit read back after configuration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxEffectiveLimit {
    NotConfigured,
    Unlimited,
    Bytes(u64),
    Count(u32),
}

/// Result of verifying a mounted helper/diagnostic socket against the attempt cgroup.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxHelperScopeOutcome {
    NotAttached {
        socket_kind: String,
    },
    ContainedInAttemptCgroup {
        socket_kind: String,
        peer_pid: u32,
        peer_cgroup: String,
    },
    InProcessPassiveDatagram {
        socket_kind: String,
    },
    Refused {
        socket_kind: String,
        reason: String,
    },
}

/// Recorded enforcement, membership, controller, and post-exit resource event facts
/// for one local sandbox attempt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxResourceEnforcementFacts {
    pub enforced_memory: bool,
    pub enforced_pids: bool,
    pub enforced_scratch: bool,
    pub attempt_cgroup_rel_path: Option<String>,
    pub observed_member_pids: Vec<u32>,
    pub populated_observed: bool,
    pub effective_memory_max: SandboxEffectiveLimit,
    pub effective_swap_max: SandboxEffectiveLimit,
    pub effective_oom_group: Option<bool>,
    pub effective_pids_max: SandboxEffectiveLimit,
    pub swap_policy: SandboxSwapPolicy,
    pub helper_checks: Vec<SandboxHelperScopeOutcome>,
    pub oom_count: u64,
    pub oom_kill_count: u64,
    pub memory_peak_bytes: Option<u64>,
    pub pids_max_events: u64,
    pub unavailable_reasons: Vec<SandboxUnavailableReason>,
}

impl Default for SandboxResourceEnforcementFacts {
    fn default() -> Self {
        Self {
            enforced_memory: false,
            enforced_pids: false,
            enforced_scratch: false,
            attempt_cgroup_rel_path: None,
            observed_member_pids: Vec::new(),
            populated_observed: false,
            effective_memory_max: SandboxEffectiveLimit::NotConfigured,
            effective_swap_max: SandboxEffectiveLimit::NotConfigured,
            effective_oom_group: None,
            effective_pids_max: SandboxEffectiveLimit::NotConfigured,
            swap_policy: SandboxSwapPolicy::Zero,
            helper_checks: Vec::new(),
            oom_count: 0,
            oom_kill_count: 0,
            memory_peak_bytes: None,
            pids_max_events: 0,
            unavailable_reasons: Vec::new(),
        }
    }
}

/// Default periodic sampling interval for opt-in live CPU diagnostics.
pub const DEFAULT_LIVE_CPU_SAMPLE_CADENCE_MS: u64 = 500;
/// Minimum allowed sampling interval for opt-in live CPU diagnostics.
pub const MIN_LIVE_CPU_SAMPLE_CADENCE_MS: u64 = 50;
/// Maximum allowed sampling interval for opt-in live CPU diagnostics.
pub const MAX_LIVE_CPU_SAMPLE_CADENCE_MS: u64 = 60_000;
/// Default bounded event channel capacity for live CPU diagnostics.
pub const DEFAULT_LIVE_CPU_CHANNEL_CAPACITY_COUNT: u32 = 256;
/// Maximum allowed bounded event channel capacity for live CPU diagnostics.
pub const MAX_LIVE_CPU_CHANNEL_CAPACITY_COUNT: u32 = 4_096;
/// Maximum supported allocated core count denominator.
pub const MAX_LIVE_CPU_ALLOCATED_CORES: u32 = 1_024;

/// Opt-in runtime configuration for bounded attempt-scoped live CPU sampling.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxLiveCpuDiagnosticsConfig {
    pub sample_cadence_ms: u64,
    pub allocated_cores: u32,
    pub generation: u64,
    pub cgroup_root: Option<PathBuf>,
}

impl SandboxLiveCpuDiagnosticsConfig {
    pub fn standard(allocated_cores: u32) -> Self {
        Self {
            sample_cadence_ms: DEFAULT_LIVE_CPU_SAMPLE_CADENCE_MS,
            allocated_cores: allocated_cores.clamp(1, MAX_LIVE_CPU_ALLOCATED_CORES),
            generation: 1,
            cgroup_root: None,
        }
    }

    pub fn validate(&self) -> std::io::Result<()> {
        if !(MIN_LIVE_CPU_SAMPLE_CADENCE_MS..=MAX_LIVE_CPU_SAMPLE_CADENCE_MS).contains(&self.sample_cadence_ms) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "sample_cadence_ms {} must be within {}..={}",
                    self.sample_cadence_ms, MIN_LIVE_CPU_SAMPLE_CADENCE_MS, MAX_LIVE_CPU_SAMPLE_CADENCE_MS
                ),
            ));
        }
        if !(1..=MAX_LIVE_CPU_ALLOCATED_CORES).contains(&self.allocated_cores) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("allocated_cores {} must be within 1..={MAX_LIVE_CPU_ALLOCATED_CORES}", self.allocated_cores),
            ));
        }
        if self.generation == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "live CPU diagnostics generation must be >= 1",
            ));
        }
        Ok(())
    }
}

/// Honest reason why an attempt has no readable owned CPU cgroup scope.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxLiveCpuUnavailableReason {
    CgroupRootUnconfigured,
    CgroupRootInvalidOrUnowned { detail: String },
    DelegationLcaNotWritable { detail: String },
    ChildMigrationFailed { detail: String },
    CpuStatUnreadable { detail: String },
    SetupIoError { detail: String },
    SupervisorPanickedOrUnbound,
}

/// Ownership and source of live CPU samples for a single local attempt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxCpuScopeAvailability {
    OwnedAttemptCgroupV2 { rel_path: String },
    Unavailable { reason: SandboxLiveCpuUnavailableReason },
}

/// Reason a live CPU sampler retired its attempt token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxCpuRetirementReason {
    Completed,
    Cancelled,
    TimedOut,
    GracefulShutdown,
    Failed,
    DroppedOrAborted,
}

/// Payload emitted by the attempt-scoped live CPU sampler.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxCpuSampleKind {
    Started { scope: SandboxCpuScopeAvailability },
    Sample { cumulative_cpu_usec: u64 },
    Unreadable { detail: String },
    Retired { reason: SandboxCpuRetirementReason },
}

/// Non-authoritative diagnostic sample event tagged with attempt identity and generation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxCpuSampleEvent {
    pub attempt_id: u64,
    pub generation: u64,
    pub attempt_label: String,
    pub allocated_cores: u32,
    pub observed_at_usec: u64,
    pub kind: SandboxCpuSampleKind,
}

/// Terminal/latest sampler facts retained on `BubblewrapBuildService` for the last attempt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxLiveCpuAttemptFacts {
    pub attempt_id: u64,
    pub generation: u64,
    pub attempt_label: String,
    pub scope: SandboxCpuScopeAvailability,
    pub sample_count: u64,
    pub dropped_samples_count: u64,
    pub initial_cpu_usec: Option<u64>,
    pub latest_cpu_usec: Option<u64>,
    pub retired_reason: Option<SandboxCpuRetirementReason>,
}

/// Task-local diagnostic label/generation propagated from `Worker` to `BubblewrapBuildService`
/// without polluting `BuildRequest`, `action_ref`, or `rust_waits::request_identity`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveCpuAttemptContext {
    pub attempt_label: String,
    pub generation: u64,
}

tokio::task_local! {
    static LIVE_CPU_ATTEMPT_CONTEXT: LiveCpuAttemptContext;
}

/// Execute `future` under an attempt-scoped diagnostic context.
pub async fn scope_live_cpu_attempt<F>(context: LiveCpuAttemptContext, future: F) -> F::Output
where F: std::future::Future {
    LIVE_CPU_ATTEMPT_CONTEXT.scope(context, future).await
}

/// Read the active task-local attempt diagnostic context, if any.
pub fn current_live_cpu_attempt_context() -> Option<LiveCpuAttemptContext> {
    LIVE_CPU_ATTEMPT_CONTEXT.try_with(Clone::clone).ok()
}
