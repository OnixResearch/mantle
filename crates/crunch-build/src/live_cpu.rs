//! Pure, clock-free, versioned heuristic classifier and bounded aggregator for
//! opt-in live CPU diagnostics on owned local build attempts.
//!
//! This module contains zero ambient clocks, zero process/cgroup syscalls, and
//! zero `usize` types in its public API. Diagnostic observations are strictly
//! non-authoritative: they never enter derivation/cache/action identity, signed
//! receipts, quota accounting, or scheduler priority inputs.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

pub const LIVE_CPU_DIAGNOSTICS_SCHEMA: &str = "mantle-live-cpu-diagnostics-v1";
pub const LIVE_CPU_SCOPE_LABEL: &str = "observed-local-build-attempts-only";
pub const LIVE_CPU_DIAGNOSTICS_NON_CLAIM: &str = "heuristic local attempt cpu observation only; not proof of bottleneck, scheduler authority, trusted receipt, or host/farm cpu";

pub const DEFAULT_LIVE_CPU_CADENCE_MS: u64 = 500;
pub const MIN_LIVE_CPU_CADENCE_MS: u64 = 50;
pub const MAX_LIVE_CPU_CADENCE_MS: u64 = 60_000;

pub const DEFAULT_MIN_SAMPLE_INTERVAL_USEC: u64 = 100_000;
pub const MIN_SAMPLE_INTERVAL_USEC: u64 = 1_000;
pub const MAX_SAMPLE_INTERVAL_USEC: u64 = 60_000_000;

pub const DEFAULT_SUSTAINED_WINDOW_MS: u64 = 5_000;
pub const MIN_SUSTAINED_WINDOW_MS: u64 = 100;
pub const MAX_SUSTAINED_WINDOW_MS: u64 = 300_000;

pub const DEFAULT_STALE_AFTER_MS: u64 = 3_000;
pub const MIN_STALE_AFTER_MS: u64 = 100;
pub const MAX_STALE_AFTER_MS: u64 = 300_000;

pub const DEFAULT_IDLE_MAX_MILLI_CORES: u32 = 50;
pub const DEFAULT_LOW_PARALLELISM_MAX_PER_MILLE: u32 = 350;
pub const DEFAULT_LOW_PARALLELISM_MAX_MILLI_CORES: u32 = 1_250;
pub const DEFAULT_HEAVY_MIN_PER_MILLE: u32 = 750;

pub const MAX_ALLOCATED_CORES: u32 = 1_024;
pub const MAX_TRACKED_ATTEMPTS_COUNT: u32 = 64;
pub const DEFAULT_SAMPLE_CHANNEL_CAPACITY_COUNT: u32 = 256;
pub const MAX_SAMPLE_CHANNEL_CAPACITY_COUNT: u32 = 4_096;

pub const MILLI_CORES_PER_CORE: u32 = 1_000;
pub const PER_MILLE_FULL_SCALE: u32 = 1_000;
const USEC_PER_MS: u64 = 1_000;
const MAX_MILLI_CORES_BOUND: u32 = 1_024_000;
const MAX_PER_MILLE_BOUND: u32 = 1_024_000;
const MULTI_CORE_LOW_PARALLELISM_MIN_CORES: u32 = 2;
const HIGH_CORE_LOW_PARALLELISM_MIN_CORES: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LiveCpuError {
    #[error("unsupported live CPU diagnostics schema: {schema}")]
    UnsupportedSchema { schema: String },
    #[error("live CPU diagnostics field '{field}' is out of bounds: {detail}")]
    InvalidBound { field: &'static str, detail: String },
    #[error("live CPU tracker capacity exceeded (max {max_count})")]
    CapacityExceeded { max_count: u32 },
}

/// Non-zero monotonic generation token fencing an individual build attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttemptGeneration(u64);

impl AttemptGeneration {
    pub fn new(generation_count: u64) -> Result<Self, LiveCpuError> {
        if generation_count == 0 {
            return Err(LiveCpuError::InvalidBound {
                field: "generation",
                detail: "must be >= 1".to_string(),
            });
        }
        debug_assert!(generation_count >= 1);
        Ok(Self(generation_count))
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Explicit monotonic timestamp in microseconds supplied by caller or sampler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MonotonicTimestampUsec(u64);

impl MonotonicTimestampUsec {
    pub const fn from_usec(timestamp_usec: u64) -> Self {
        Self(timestamp_usec)
    }

    pub const fn as_usec(self) -> u64 {
        self.0
    }
}

/// Cumulative CPU time counter in microseconds (e.g., cgroup-v2 `cpu.stat` `usage_usec`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CumulativeCpuUsec(u64);

impl CumulativeCpuUsec {
    pub const fn from_usec(cpu_usec: u64) -> Self {
        Self(cpu_usec)
    }

    pub const fn as_usec(self) -> u64 {
        self.0
    }
}

/// Explicit normalization denominator representing configured local build cores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AllocatedCores(u32);

impl AllocatedCores {
    pub fn new(allocated_cores: u32) -> Result<Self, LiveCpuError> {
        if allocated_cores == 0 {
            return Err(LiveCpuError::InvalidBound {
                field: "allocated_cores",
                detail: "must be >= 1".to_string(),
            });
        }
        if allocated_cores > MAX_ALLOCATED_CORES {
            return Err(LiveCpuError::InvalidBound {
                field: "allocated_cores",
                detail: format!("must be <= {MAX_ALLOCATED_CORES}"),
            });
        }
        debug_assert!(allocated_cores >= 1);
        debug_assert!(allocated_cores <= MAX_ALLOCATED_CORES);
        Ok(Self(allocated_cores))
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Fixed-point logical cores used (`1_000` == 1.000 logical core).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MilliCores(u32);

impl MilliCores {
    pub const fn from_milli_cores(milli_cores: u32) -> Self {
        Self(milli_cores)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Fixed-point utilization relative to `allocated_cores` (`1_000` == 100.0%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UtilizationPerMille(u32);

impl UtilizationPerMille {
    pub const fn from_per_mille(per_mille: u32) -> Self {
        Self(per_mille)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Configurable, versioned heuristic policy for classifying local attempt CPU usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveCpuClassifierPolicy {
    pub schema: String,
    pub sample_cadence_ms: u64,
    pub min_sample_interval_usec: u64,
    pub sustained_window_ms: u64,
    pub stale_after_ms: u64,
    pub idle_max_milli_cores: u32,
    pub low_parallelism_max_per_mille: u32,
    pub low_parallelism_max_milli_cores: u32,
    pub heavy_min_per_mille: u32,
    pub max_tracked_attempts_count: u32,
    pub channel_capacity_count: u32,
}

impl Default for LiveCpuClassifierPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

impl LiveCpuClassifierPolicy {
    pub fn standard() -> Self {
        Self {
            schema: LIVE_CPU_DIAGNOSTICS_SCHEMA.to_string(),
            sample_cadence_ms: DEFAULT_LIVE_CPU_CADENCE_MS,
            min_sample_interval_usec: DEFAULT_MIN_SAMPLE_INTERVAL_USEC,
            sustained_window_ms: DEFAULT_SUSTAINED_WINDOW_MS,
            stale_after_ms: DEFAULT_STALE_AFTER_MS,
            idle_max_milli_cores: DEFAULT_IDLE_MAX_MILLI_CORES,
            low_parallelism_max_per_mille: DEFAULT_LOW_PARALLELISM_MAX_PER_MILLE,
            low_parallelism_max_milli_cores: DEFAULT_LOW_PARALLELISM_MAX_MILLI_CORES,
            heavy_min_per_mille: DEFAULT_HEAVY_MIN_PER_MILLE,
            max_tracked_attempts_count: MAX_TRACKED_ATTEMPTS_COUNT,
            channel_capacity_count: DEFAULT_SAMPLE_CHANNEL_CAPACITY_COUNT,
        }
    }

    pub fn validate(&self) -> Result<(), LiveCpuError> {
        validate_policy_schema_and_timings(self)?;
        validate_policy_thresholds_and_bounds(self)?;
        debug_assert_eq!(self.schema, LIVE_CPU_DIAGNOSTICS_SCHEMA);
        debug_assert!(self.low_parallelism_max_per_mille < self.heavy_min_per_mille);
        Ok(())
    }
}

fn validate_policy_schema_and_timings(policy: &LiveCpuClassifierPolicy) -> Result<(), LiveCpuError> {
    if policy.schema != LIVE_CPU_DIAGNOSTICS_SCHEMA {
        return Err(LiveCpuError::UnsupportedSchema {
            schema: policy.schema.clone(),
        });
    }
    if policy.sample_cadence_ms < MIN_LIVE_CPU_CADENCE_MS {
        return Err(LiveCpuError::InvalidBound {
            field: "sample_cadence_ms",
            detail: format!("must be >= {MIN_LIVE_CPU_CADENCE_MS}"),
        });
    }
    if policy.sample_cadence_ms > MAX_LIVE_CPU_CADENCE_MS {
        return Err(LiveCpuError::InvalidBound {
            field: "sample_cadence_ms",
            detail: format!("must be <= {MAX_LIVE_CPU_CADENCE_MS}"),
        });
    }
    if policy.min_sample_interval_usec < MIN_SAMPLE_INTERVAL_USEC {
        return Err(LiveCpuError::InvalidBound {
            field: "min_sample_interval_usec",
            detail: format!("must be >= {MIN_SAMPLE_INTERVAL_USEC}"),
        });
    }
    if policy.min_sample_interval_usec > MAX_SAMPLE_INTERVAL_USEC {
        return Err(LiveCpuError::InvalidBound {
            field: "min_sample_interval_usec",
            detail: format!("must be <= {MAX_SAMPLE_INTERVAL_USEC}"),
        });
    }
    if policy.sustained_window_ms < MIN_SUSTAINED_WINDOW_MS {
        return Err(LiveCpuError::InvalidBound {
            field: "sustained_window_ms",
            detail: format!("must be >= {MIN_SUSTAINED_WINDOW_MS}"),
        });
    }
    if policy.sustained_window_ms > MAX_SUSTAINED_WINDOW_MS {
        return Err(LiveCpuError::InvalidBound {
            field: "sustained_window_ms",
            detail: format!("must be <= {MAX_SUSTAINED_WINDOW_MS}"),
        });
    }
    if policy.stale_after_ms < MIN_STALE_AFTER_MS {
        return Err(LiveCpuError::InvalidBound {
            field: "stale_after_ms",
            detail: format!("must be >= {MIN_STALE_AFTER_MS}"),
        });
    }
    if policy.stale_after_ms > MAX_STALE_AFTER_MS {
        return Err(LiveCpuError::InvalidBound {
            field: "stale_after_ms",
            detail: format!("must be <= {MAX_STALE_AFTER_MS}"),
        });
    }
    debug_assert!(policy.sample_cadence_ms >= MIN_LIVE_CPU_CADENCE_MS);
    debug_assert!(policy.stale_after_ms >= MIN_STALE_AFTER_MS);
    Ok(())
}

fn validate_policy_thresholds_and_bounds(policy: &LiveCpuClassifierPolicy) -> Result<(), LiveCpuError> {
    if policy.idle_max_milli_cores == 0 {
        return Err(LiveCpuError::InvalidBound {
            field: "idle_max_milli_cores",
            detail: "must be >= 1".to_string(),
        });
    }
    if policy.low_parallelism_max_per_mille >= policy.heavy_min_per_mille {
        return Err(LiveCpuError::InvalidBound {
            field: "low_parallelism_max_per_mille",
            detail: "must be strictly less than heavy_min_per_mille".to_string(),
        });
    }
    if policy.heavy_min_per_mille > PER_MILLE_FULL_SCALE {
        return Err(LiveCpuError::InvalidBound {
            field: "heavy_min_per_mille",
            detail: format!("must be <= {PER_MILLE_FULL_SCALE}"),
        });
    }
    if policy.max_tracked_attempts_count == 0 {
        return Err(LiveCpuError::InvalidBound {
            field: "max_tracked_attempts_count",
            detail: "must be >= 1".to_string(),
        });
    }
    if policy.max_tracked_attempts_count > MAX_TRACKED_ATTEMPTS_COUNT {
        return Err(LiveCpuError::InvalidBound {
            field: "max_tracked_attempts_count",
            detail: format!("must be <= {MAX_TRACKED_ATTEMPTS_COUNT}"),
        });
    }
    if policy.channel_capacity_count == 0 {
        return Err(LiveCpuError::InvalidBound {
            field: "channel_capacity_count",
            detail: "must be >= 1".to_string(),
        });
    }
    if policy.channel_capacity_count > MAX_SAMPLE_CHANNEL_CAPACITY_COUNT {
        return Err(LiveCpuError::InvalidBound {
            field: "channel_capacity_count",
            detail: format!("must be <= {MAX_SAMPLE_CHANNEL_CAPACITY_COUNT}"),
        });
    }
    debug_assert!(policy.max_tracked_attempts_count <= MAX_TRACKED_ATTEMPTS_COUNT);
    debug_assert!(policy.channel_capacity_count <= MAX_SAMPLE_CHANNEL_CAPACITY_COUNT);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveCpuUnavailableReason {
    CgroupRootUnconfigured,
    CgroupRootInvalidOrUnowned,
    DelegationLcaNotWritable,
    ChildMigrationFailed,
    CpuStatUnreadable,
    SetupIoError,
    SupervisorPanickedOrUnbound,
}

impl LiveCpuUnavailableReason {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CgroupRootUnconfigured => "cgroup-root-unconfigured",
            Self::CgroupRootInvalidOrUnowned => "cgroup-root-invalid-or-unowned",
            Self::DelegationLcaNotWritable => "delegation-lca-not-writable",
            Self::ChildMigrationFailed => "child-migration-failed",
            Self::CpuStatUnreadable => "cpu-stat-unreadable",
            Self::SetupIoError => "setup-io-error",
            Self::SupervisorPanickedOrUnbound => "supervisor-panicked-or-unbound",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveCpuScopeAvailability {
    OwnedAttemptCgroupV2 { rel_path: String },
    Unavailable { reason: LiveCpuUnavailableReason },
}

impl LiveCpuScopeAvailability {
    pub const fn as_kind_str(&self) -> &'static str {
        match self {
            Self::OwnedAttemptCgroupV2 { .. } => "owned-attempt-cgroup-v2",
            Self::Unavailable { .. } => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveCpuActivityClass {
    Unknown,
    Idle,
    LowParallelism,
    Moderate,
    Heavy,
}

impl LiveCpuActivityClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Idle => "idle",
            Self::LowParallelism => "low-parallelism",
            Self::Moderate => "moderate",
            Self::Heavy => "heavy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveCpuUnknownReason {
    FirstSample,
    InsufficientElapsedWindow,
    NonMonotonicInterval,
    CounterReset,
    UnreadableSample,
    ScopeUnavailable,
    StaleSample,
}

impl LiveCpuUnknownReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FirstSample => "first-sample",
            Self::InsufficientElapsedWindow => "insufficient-elapsed-window",
            Self::NonMonotonicInterval => "non-monotonic-interval",
            Self::CounterReset => "counter-reset",
            Self::UnreadableSample => "unreadable-sample",
            Self::ScopeUnavailable => "scope-unavailable",
            Self::StaleSample => "stale-sample",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveCpuRetirementReason {
    Completed,
    Cancelled,
    TimedOut,
    GracefulShutdown,
    Failed,
    DroppedOrAborted,
}

impl LiveCpuRetirementReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed-out",
            Self::GracefulShutdown => "graceful-shutdown",
            Self::Failed => "failed",
            Self::DroppedOrAborted => "dropped-or-aborted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveCpuAttemptMeta {
    pub attempt_id: u64,
    pub drv_key: String,
    pub generation: AttemptGeneration,
    pub allocated_cores: AllocatedCores,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveCpuRawMeasurement {
    Started {
        observed_at_usec: MonotonicTimestampUsec,
        scope: LiveCpuScopeAvailability,
    },
    Sampled {
        observed_at_usec: MonotonicTimestampUsec,
        cumulative_cpu_usec: CumulativeCpuUsec,
    },
    Unreadable {
        observed_at_usec: MonotonicTimestampUsec,
    },
    Retired {
        observed_at_usec: MonotonicTimestampUsec,
        reason: LiveCpuRetirementReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveCpuSampleEvent {
    pub meta: LiveCpuAttemptMeta,
    pub measurement: LiveCpuRawMeasurement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveCpuIngestDisposition {
    Accepted,
    StaleGenerationIgnored,
    CapacityRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveCpuAttemptStatus {
    pub drv_key: String,
    pub attempt_id: u64,
    pub generation: AttemptGeneration,
    pub is_active: bool,
    pub is_coverage_available: bool,
    pub scope: LiveCpuScopeAvailability,
    pub activity_class: LiveCpuActivityClass,
    pub unknown_reason: Option<LiveCpuUnknownReason>,
    pub allocated_cores: AllocatedCores,
    pub latest_cores_used_milli: Option<MilliCores>,
    pub average_cores_used_milli: Option<MilliCores>,
    pub peak_cores_used_milli: Option<MilliCores>,
    pub latest_utilization_per_mille: Option<UtilizationPerMille>,
    pub sample_count: u64,
    pub window_elapsed_ms: u64,
    pub sample_age_ms: u64,
    pub sustained_duration_ms: u64,
    pub retired_reason: Option<LiveCpuRetirementReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveCpuAggregateSummary {
    pub schema: String,
    pub scope_label: String,
    pub non_claim: String,
    pub active_attempts_count: u32,
    pub observed_attempts_count: u32,
    pub unavailable_attempts_count: u32,
    pub unknown_attempts_count: u32,
    pub completed_attempts_count: u32,
    pub normalization_denominator_cores: u32,
    pub observed_cores_used_milli: Option<MilliCores>,
    pub observed_utilization_per_mille: Option<UtilizationPerMille>,
    pub aggregate_class: LiveCpuActivityClass,
    pub dropped_samples_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveCpuDiagnosticsReport {
    pub aggregate: LiveCpuAggregateSummary,
    pub attempts: Vec<LiveCpuAttemptStatus>,
}

/// Convert a `snix_build` sample event into a pure `crunch_build` sample event.
pub fn from_snix_sample_event(
    event: &snix_build::buildservice::SandboxCpuSampleEvent,
) -> Result<LiveCpuSampleEvent, LiveCpuError> {
    let generation = AttemptGeneration::new(event.generation)?;
    let allocated_cores = AllocatedCores::new(event.allocated_cores)?;
    let observed_at_usec = MonotonicTimestampUsec::from_usec(event.observed_at_usec);
    let measurement = convert_snix_sample_kind(&event.kind, observed_at_usec);
    debug_assert!(generation.as_u64() >= 1);
    debug_assert!(allocated_cores.as_u32() >= 1);
    Ok(LiveCpuSampleEvent {
        meta: LiveCpuAttemptMeta {
            attempt_id: event.attempt_id,
            drv_key: event.attempt_label.clone(),
            generation,
            allocated_cores,
        },
        measurement,
    })
}

fn convert_snix_sample_kind(
    kind: &snix_build::buildservice::SandboxCpuSampleKind,
    observed_at_usec: MonotonicTimestampUsec,
) -> LiveCpuRawMeasurement {
    match kind {
        snix_build::buildservice::SandboxCpuSampleKind::Started { scope } => LiveCpuRawMeasurement::Started {
            observed_at_usec,
            scope: convert_snix_scope(scope),
        },
        snix_build::buildservice::SandboxCpuSampleKind::Sample { cumulative_cpu_usec } => {
            LiveCpuRawMeasurement::Sampled {
                observed_at_usec,
                cumulative_cpu_usec: CumulativeCpuUsec::from_usec(*cumulative_cpu_usec),
            }
        }
        snix_build::buildservice::SandboxCpuSampleKind::Unreadable { .. } => {
            LiveCpuRawMeasurement::Unreadable { observed_at_usec }
        }
        snix_build::buildservice::SandboxCpuSampleKind::Retired { reason } => LiveCpuRawMeasurement::Retired {
            observed_at_usec,
            reason: convert_snix_retirement_reason(*reason),
        },
    }
}

fn convert_snix_scope(scope: &snix_build::buildservice::SandboxCpuScopeAvailability) -> LiveCpuScopeAvailability {
    match scope {
        snix_build::buildservice::SandboxCpuScopeAvailability::OwnedAttemptCgroupV2 { rel_path } => {
            LiveCpuScopeAvailability::OwnedAttemptCgroupV2 {
                rel_path: rel_path.clone(),
            }
        }
        snix_build::buildservice::SandboxCpuScopeAvailability::Unavailable { reason } => {
            LiveCpuScopeAvailability::Unavailable {
                reason: convert_snix_unavailable_reason(reason),
            }
        }
    }
}

fn convert_snix_unavailable_reason(
    reason: &snix_build::buildservice::SandboxLiveCpuUnavailableReason,
) -> LiveCpuUnavailableReason {
    match reason {
        snix_build::buildservice::SandboxLiveCpuUnavailableReason::CgroupRootUnconfigured => {
            LiveCpuUnavailableReason::CgroupRootUnconfigured
        }
        snix_build::buildservice::SandboxLiveCpuUnavailableReason::CgroupRootInvalidOrUnowned { .. } => {
            LiveCpuUnavailableReason::CgroupRootInvalidOrUnowned
        }
        snix_build::buildservice::SandboxLiveCpuUnavailableReason::DelegationLcaNotWritable { .. } => {
            LiveCpuUnavailableReason::DelegationLcaNotWritable
        }
        snix_build::buildservice::SandboxLiveCpuUnavailableReason::ChildMigrationFailed { .. } => {
            LiveCpuUnavailableReason::ChildMigrationFailed
        }
        snix_build::buildservice::SandboxLiveCpuUnavailableReason::CpuStatUnreadable { .. } => {
            LiveCpuUnavailableReason::CpuStatUnreadable
        }
        snix_build::buildservice::SandboxLiveCpuUnavailableReason::SetupIoError { .. } => {
            LiveCpuUnavailableReason::SetupIoError
        }
        snix_build::buildservice::SandboxLiveCpuUnavailableReason::SupervisorPanickedOrUnbound => {
            LiveCpuUnavailableReason::SupervisorPanickedOrUnbound
        }
    }
}

fn convert_snix_retirement_reason(
    reason: snix_build::buildservice::SandboxCpuRetirementReason,
) -> LiveCpuRetirementReason {
    match reason {
        snix_build::buildservice::SandboxCpuRetirementReason::Completed => LiveCpuRetirementReason::Completed,
        snix_build::buildservice::SandboxCpuRetirementReason::Cancelled => LiveCpuRetirementReason::Cancelled,
        snix_build::buildservice::SandboxCpuRetirementReason::TimedOut => LiveCpuRetirementReason::TimedOut,
        snix_build::buildservice::SandboxCpuRetirementReason::GracefulShutdown => {
            LiveCpuRetirementReason::GracefulShutdown
        }
        snix_build::buildservice::SandboxCpuRetirementReason::Failed => LiveCpuRetirementReason::Failed,
        snix_build::buildservice::SandboxCpuRetirementReason::DroppedOrAborted => {
            LiveCpuRetirementReason::DroppedOrAborted
        }
    }
}

pub fn compute_milli_cores_and_utilization(
    delta_cpu: CumulativeCpuUsec,
    delta_wall_usec: u64,
    allocated_cores: AllocatedCores,
) -> Option<(MilliCores, UtilizationPerMille)> {
    if delta_wall_usec == 0 {
        return None;
    }
    let scaled_cpu = u128::from(delta_cpu.as_usec()).checked_mul(u128::from(MILLI_CORES_PER_CORE))?;
    let raw_milli = scaled_cpu.checked_div(u128::from(delta_wall_usec))?;
    let bounded_milli =
        u32::try_from(raw_milli.min(u128::from(MAX_MILLI_CORES_BOUND))).unwrap_or(MAX_MILLI_CORES_BOUND);
    let cores_used = MilliCores::from_milli_cores(bounded_milli);

    let util_raw = u128::from(bounded_milli).checked_div(u128::from(allocated_cores.as_u32()))?;
    let util_bounded = u32::try_from(util_raw.min(u128::from(MAX_PER_MILLE_BOUND))).unwrap_or(MAX_PER_MILLE_BOUND);
    let utilization = UtilizationPerMille::from_per_mille(util_bounded);
    debug_assert!(allocated_cores.as_u32() >= 1);
    debug_assert!(cores_used.as_u32() <= MAX_MILLI_CORES_BOUND);
    Some((cores_used, utilization))
}

pub fn classify_instant_rate(
    policy: &LiveCpuClassifierPolicy,
    cores_used: MilliCores,
    utilization: UtilizationPerMille,
    allocated_cores: AllocatedCores,
) -> LiveCpuActivityClass {
    debug_assert!(allocated_cores.as_u32() >= 1);
    debug_assert!(policy.low_parallelism_max_per_mille < policy.heavy_min_per_mille);
    if cores_used.as_u32() <= policy.idle_max_milli_cores {
        return LiveCpuActivityClass::Idle;
    }
    if utilization.as_u32() >= policy.heavy_min_per_mille {
        return LiveCpuActivityClass::Heavy;
    }
    if is_low_parallelism(policy, cores_used, utilization, allocated_cores) {
        return LiveCpuActivityClass::LowParallelism;
    }
    LiveCpuActivityClass::Moderate
}

fn is_low_parallelism(
    policy: &LiveCpuClassifierPolicy,
    cores_used: MilliCores,
    utilization: UtilizationPerMille,
    allocated_cores: AllocatedCores,
) -> bool {
    if allocated_cores.as_u32() < MULTI_CORE_LOW_PARALLELISM_MIN_CORES {
        return false;
    }
    if utilization.as_u32() <= policy.low_parallelism_max_per_mille {
        return true;
    }
    allocated_cores.as_u32() >= HIGH_CORE_LOW_PARALLELISM_MIN_CORES
        && cores_used.as_u32() <= policy.low_parallelism_max_milli_cores
}

struct ValidRateInput {
    observed_at_usec: MonotonicTimestampUsec,
    cumulative_cpu_usec: CumulativeCpuUsec,
    delta_cpu_usec: u64,
    delta_wall_usec: u64,
    cores_used: MilliCores,
    utilization: UtilizationPerMille,
}

#[derive(Debug, Clone)]
struct AttemptTrackState {
    drv_key: String,
    attempt_id: u64,
    generation: AttemptGeneration,
    allocated_cores: AllocatedCores,
    is_active: bool,
    scope: LiveCpuScopeAvailability,
    first_sample_at_usec: Option<MonotonicTimestampUsec>,
    last_sample_at_usec: Option<MonotonicTimestampUsec>,
    last_cpu_usec: Option<CumulativeCpuUsec>,
    latest_cores_used_milli: Option<MilliCores>,
    peak_cores_used_milli: Option<MilliCores>,
    latest_utilization_per_mille: Option<UtilizationPerMille>,
    total_valid_cpu_usec: u64,
    total_valid_wall_usec: u64,
    sample_count: u64,
    candidate_class: Option<LiveCpuActivityClass>,
    sustained_duration_ms: u64,
    activity_class: LiveCpuActivityClass,
    unknown_reason: Option<LiveCpuUnknownReason>,
    retired_reason: Option<LiveCpuRetirementReason>,
}

impl AttemptTrackState {
    fn new(meta: &LiveCpuAttemptMeta) -> Self {
        Self {
            drv_key: meta.drv_key.clone(),
            attempt_id: meta.attempt_id,
            generation: meta.generation,
            allocated_cores: meta.allocated_cores,
            is_active: true,
            scope: LiveCpuScopeAvailability::Unavailable {
                reason: LiveCpuUnavailableReason::CgroupRootUnconfigured,
            },
            first_sample_at_usec: None,
            last_sample_at_usec: None,
            last_cpu_usec: None,
            latest_cores_used_milli: None,
            peak_cores_used_milli: None,
            latest_utilization_per_mille: None,
            total_valid_cpu_usec: 0,
            total_valid_wall_usec: 0,
            sample_count: 0,
            candidate_class: None,
            sustained_duration_ms: 0,
            activity_class: LiveCpuActivityClass::Unknown,
            unknown_reason: Some(LiveCpuUnknownReason::FirstSample),
            retired_reason: None,
        }
    }

    fn clear_rate_and_mark_unknown(&mut self, reason: LiveCpuUnknownReason) {
        self.latest_cores_used_milli = None;
        self.latest_utilization_per_mille = None;
        self.candidate_class = None;
        self.sustained_duration_ms = 0;
        self.activity_class = LiveCpuActivityClass::Unknown;
        self.unknown_reason = Some(reason);
    }

    fn apply_started(&mut self, scope: &LiveCpuScopeAvailability) {
        self.scope = scope.clone();
        match scope {
            LiveCpuScopeAvailability::OwnedAttemptCgroupV2 { .. } => {
                if self.sample_count == 0 {
                    self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::FirstSample);
                }
            }
            LiveCpuScopeAvailability::Unavailable { .. } => {
                self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::ScopeUnavailable);
            }
        }
    }

    fn apply_sample(
        &mut self,
        policy: &LiveCpuClassifierPolicy,
        observed_at_usec: MonotonicTimestampUsec,
        cumulative_cpu_usec: CumulativeCpuUsec,
    ) {
        if matches!(self.scope, LiveCpuScopeAvailability::Unavailable { .. }) {
            self.scope = LiveCpuScopeAvailability::OwnedAttemptCgroupV2 {
                rel_path: "observed-cgroup".to_string(),
            };
        }
        self.sample_count = self.sample_count.saturating_add(1);
        let (Some(prev_ts), Some(prev_cpu)) = (self.last_sample_at_usec, self.last_cpu_usec) else {
            self.anchor_baseline(observed_at_usec, cumulative_cpu_usec);
            self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::FirstSample);
            return;
        };
        let Some(delta_wall_usec) = observed_at_usec.as_usec().checked_sub(prev_ts.as_usec()) else {
            self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::NonMonotonicInterval);
            return;
        };
        if delta_wall_usec == 0 {
            self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::NonMonotonicInterval);
            return;
        }
        let Some(delta_cpu_usec) = cumulative_cpu_usec.as_usec().checked_sub(prev_cpu.as_usec()) else {
            self.anchor_baseline(observed_at_usec, cumulative_cpu_usec);
            self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::CounterReset);
            return;
        };
        if delta_wall_usec < policy.min_sample_interval_usec {
            if self.activity_class == LiveCpuActivityClass::Unknown && self.unknown_reason.is_none() {
                self.unknown_reason = Some(LiveCpuUnknownReason::InsufficientElapsedWindow);
            }
            return;
        }
        let Some((cores_used, utilization)) = compute_milli_cores_and_utilization(
            CumulativeCpuUsec::from_usec(delta_cpu_usec),
            delta_wall_usec,
            self.allocated_cores,
        ) else {
            self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::NonMonotonicInterval);
            return;
        };
        self.record_valid_rate(policy, ValidRateInput {
            observed_at_usec,
            cumulative_cpu_usec,
            delta_cpu_usec,
            delta_wall_usec,
            cores_used,
            utilization,
        });
    }

    fn anchor_baseline(&mut self, observed_at_usec: MonotonicTimestampUsec, cumulative_cpu_usec: CumulativeCpuUsec) {
        if self.first_sample_at_usec.is_none() {
            self.first_sample_at_usec = Some(observed_at_usec);
        }
        self.last_sample_at_usec = Some(observed_at_usec);
        self.last_cpu_usec = Some(cumulative_cpu_usec);
    }

    fn record_valid_rate(&mut self, policy: &LiveCpuClassifierPolicy, input: ValidRateInput) {
        if self.first_sample_at_usec.is_none() {
            self.first_sample_at_usec = Some(input.observed_at_usec);
        }
        self.last_sample_at_usec = Some(input.observed_at_usec);
        self.last_cpu_usec = Some(input.cumulative_cpu_usec);
        self.latest_cores_used_milli = Some(input.cores_used);
        self.latest_utilization_per_mille = Some(input.utilization);
        self.peak_cores_used_milli = Some(match self.peak_cores_used_milli {
            Some(prev_peak) if prev_peak.as_u32() >= input.cores_used.as_u32() => prev_peak,
            _ => input.cores_used,
        });
        self.total_valid_cpu_usec = self.total_valid_cpu_usec.saturating_add(input.delta_cpu_usec);
        self.total_valid_wall_usec = self.total_valid_wall_usec.saturating_add(input.delta_wall_usec);

        let instant_class = classify_instant_rate(policy, input.cores_used, input.utilization, self.allocated_cores);
        let delta_ms = input.delta_wall_usec / USEC_PER_MS;
        if self.candidate_class == Some(instant_class) {
            self.sustained_duration_ms = self.sustained_duration_ms.saturating_add(delta_ms);
        } else {
            self.candidate_class = Some(instant_class);
            self.sustained_duration_ms = delta_ms;
        }
        if self.sustained_duration_ms >= policy.sustained_window_ms {
            self.activity_class = instant_class;
            self.unknown_reason = None;
        } else {
            self.activity_class = LiveCpuActivityClass::Unknown;
            self.unknown_reason = Some(LiveCpuUnknownReason::InsufficientElapsedWindow);
        }
        debug_assert!(self.total_valid_wall_usec > 0);
        debug_assert!(self.peak_cores_used_milli.is_some());
    }

    fn apply_unreadable(&mut self, observed_at_usec: MonotonicTimestampUsec) {
        self.last_sample_at_usec = Some(observed_at_usec);
        self.last_cpu_usec = None;
        self.clear_rate_and_mark_unknown(LiveCpuUnknownReason::UnreadableSample);
    }

    fn apply_retired(&mut self, reason: LiveCpuRetirementReason) {
        self.is_active = false;
        self.retired_reason = Some(reason);
    }

    fn average_cores_used(&self) -> Option<MilliCores> {
        compute_milli_cores_and_utilization(
            CumulativeCpuUsec::from_usec(self.total_valid_cpu_usec),
            self.total_valid_wall_usec,
            self.allocated_cores,
        )
        .map(|(milli, _)| milli)
    }

    fn to_status(&self, policy: &LiveCpuClassifierPolicy, now_usec: MonotonicTimestampUsec) -> LiveCpuAttemptStatus {
        let window_elapsed_ms = match (self.first_sample_at_usec, self.last_sample_at_usec) {
            (Some(first), Some(last)) => last.as_usec().saturating_sub(first.as_usec()) / USEC_PER_MS,
            _ => 0,
        };
        let sample_age_ms = self
            .last_sample_at_usec
            .map_or(0, |last| now_usec.as_usec().saturating_sub(last.as_usec()) / USEC_PER_MS);
        let is_stale = self.is_active && self.last_sample_at_usec.is_some() && sample_age_ms > policy.stale_after_ms;
        let is_unavailable = matches!(self.scope, LiveCpuScopeAvailability::Unavailable { .. });

        let (activity_class, unknown_reason, latest_cores_used_milli, latest_utilization_per_mille) = if is_unavailable
        {
            (LiveCpuActivityClass::Unknown, Some(LiveCpuUnknownReason::ScopeUnavailable), None, None)
        } else if is_stale {
            (LiveCpuActivityClass::Unknown, Some(LiveCpuUnknownReason::StaleSample), None, None)
        } else {
            (
                self.activity_class,
                self.unknown_reason,
                self.latest_cores_used_milli,
                self.latest_utilization_per_mille,
            )
        };
        let is_coverage_available = !is_unavailable && !is_stale && latest_cores_used_milli.is_some();

        LiveCpuAttemptStatus {
            drv_key: self.drv_key.clone(),
            attempt_id: self.attempt_id,
            generation: self.generation,
            is_active: self.is_active,
            is_coverage_available,
            scope: self.scope.clone(),
            activity_class,
            unknown_reason,
            allocated_cores: self.allocated_cores,
            latest_cores_used_milli,
            average_cores_used_milli: self.average_cores_used(),
            peak_cores_used_milli: self.peak_cores_used_milli,
            latest_utilization_per_mille,
            sample_count: self.sample_count,
            window_elapsed_ms,
            sample_age_ms,
            sustained_duration_ms: self.sustained_duration_ms,
            retired_reason: self.retired_reason,
        }
    }
}

/// Bounded, clock-free tracker of active and recently retired local build attempts.
#[derive(Debug, Clone)]
pub struct LiveCpuTracker {
    policy: LiveCpuClassifierPolicy,
    attempts: BTreeMap<String, AttemptTrackState>,
    retired_generations: BTreeMap<String, AttemptGeneration>,
    next_generation_count: u64,
    dropped_samples_count: u64,
}

impl LiveCpuTracker {
    pub fn new(policy: LiveCpuClassifierPolicy) -> Result<Self, LiveCpuError> {
        policy.validate()?;
        debug_assert!(policy.max_tracked_attempts_count >= 1);
        Ok(Self {
            policy,
            attempts: BTreeMap::new(),
            retired_generations: BTreeMap::new(),
            next_generation_count: 1,
            dropped_samples_count: 0,
        })
    }

    pub fn policy(&self) -> &LiveCpuClassifierPolicy {
        &self.policy
    }

    pub fn record_dropped_samples(&mut self, dropped_count: u64) {
        self.dropped_samples_count = self.dropped_samples_count.saturating_add(dropped_count);
    }

    pub fn register_attempt(
        &mut self,
        drv_key: String,
        allocated_cores: AllocatedCores,
    ) -> Result<AttemptGeneration, LiveCpuError> {
        let generation = AttemptGeneration::new(self.next_generation_count)?;
        self.next_generation_count = self.next_generation_count.saturating_add(1);
        if !self.attempts.contains_key(&drv_key) && !self.ensure_capacity_for_insert() {
            return Err(LiveCpuError::CapacityExceeded {
                max_count: self.policy.max_tracked_attempts_count,
            });
        }
        let meta = LiveCpuAttemptMeta {
            attempt_id: generation.as_u64(),
            drv_key: drv_key.clone(),
            generation,
            allocated_cores,
        };
        self.attempts.insert(drv_key, AttemptTrackState::new(&meta));
        debug_assert!(u32::try_from(self.attempts.len()).is_ok_and(|c| c <= self.policy.max_tracked_attempts_count));
        Ok(generation)
    }

    pub fn retire_attempt(
        &mut self,
        drv_key: &str,
        generation: AttemptGeneration,
        reason: LiveCpuRetirementReason,
    ) -> bool {
        let should_retire = match self.retired_generations.get(drv_key) {
            Some(prev) => generation.as_u64() > prev.as_u64(),
            None => true,
        };
        if should_retire {
            self.insert_retired_generation(drv_key.to_string(), generation);
        }
        let Some(state) = self.attempts.get_mut(drv_key) else {
            return false;
        };
        if state.generation != generation {
            return false;
        }
        state.apply_retired(reason);
        true
    }

    pub fn ingest(&mut self, event: &LiveCpuSampleEvent) -> LiveCpuIngestDisposition {
        if let Some(retired_gen) = self.retired_generations.get(&event.meta.drv_key)
            && event.meta.generation.as_u64() <= retired_gen.as_u64()
        {
            return LiveCpuIngestDisposition::StaleGenerationIgnored;
        }
        if let Some(existing) = self.attempts.get(&event.meta.drv_key) {
            if event.meta.generation.as_u64() < existing.generation.as_u64() {
                return LiveCpuIngestDisposition::StaleGenerationIgnored;
            }
            if event.meta.generation == existing.generation && !existing.is_active {
                return LiveCpuIngestDisposition::StaleGenerationIgnored;
            }
        } else if !self.ensure_capacity_for_insert() {
            self.dropped_samples_count = self.dropped_samples_count.saturating_add(1);
            return LiveCpuIngestDisposition::CapacityRejected;
        }

        let is_replace_with_new = self
            .attempts
            .get(&event.meta.drv_key)
            .is_none_or(|existing| event.meta.generation.as_u64() > existing.generation.as_u64());
        if is_replace_with_new {
            self.attempts.insert(event.meta.drv_key.clone(), AttemptTrackState::new(&event.meta));
        }

        let Some(state) = self.attempts.get_mut(&event.meta.drv_key) else {
            return LiveCpuIngestDisposition::CapacityRejected;
        };
        state.attempt_id = event.meta.attempt_id;
        state.allocated_cores = event.meta.allocated_cores;
        apply_measurement(
            state,
            &self.policy,
            &event.measurement,
            &mut self.retired_generations,
            self.policy.max_tracked_attempts_count,
        );
        LiveCpuIngestDisposition::Accepted
    }

    pub fn report(&self, now_usec: MonotonicTimestampUsec) -> LiveCpuDiagnosticsReport {
        let mut attempts = Vec::with_capacity(self.attempts.len());
        let mut active_attempts_count: u32 = 0;
        let mut observed_attempts_count: u32 = 0;
        let mut unavailable_attempts_count: u32 = 0;
        let mut unknown_attempts_count: u32 = 0;
        let mut completed_attempts_count: u32 = 0;
        let mut normalization_denominator_cores: u32 = 0;
        let mut observed_cores_sum_milli: u64 = 0;

        for state in self.attempts.values() {
            let status = state.to_status(&self.policy, now_usec);
            if status.is_active {
                active_attempts_count = active_attempts_count.saturating_add(1);
                if matches!(status.scope, LiveCpuScopeAvailability::Unavailable { .. }) {
                    unavailable_attempts_count = unavailable_attempts_count.saturating_add(1);
                }
                if status.activity_class == LiveCpuActivityClass::Unknown {
                    unknown_attempts_count = unknown_attempts_count.saturating_add(1);
                }
                if status.is_coverage_available
                    && let Some(milli) = status.latest_cores_used_milli
                {
                    observed_attempts_count = observed_attempts_count.saturating_add(1);
                    normalization_denominator_cores =
                        normalization_denominator_cores.saturating_add(status.allocated_cores.as_u32());
                    observed_cores_sum_milli = observed_cores_sum_milli.saturating_add(u64::from(milli.as_u32()));
                }
            } else {
                completed_attempts_count = completed_attempts_count.saturating_add(1);
            }
            attempts.push(status);
        }

        let aggregate = build_aggregate_summary(&self.policy, AggregateCounts {
            active_attempts_count,
            observed_attempts_count,
            unavailable_attempts_count,
            unknown_attempts_count,
            completed_attempts_count,
            normalization_denominator_cores,
            observed_cores_sum_milli,
            dropped_samples_count: self.dropped_samples_count,
        });
        debug_assert!(observed_attempts_count <= active_attempts_count);
        debug_assert!(u32::try_from(attempts.len()).is_ok_and(|c| c <= self.policy.max_tracked_attempts_count));
        LiveCpuDiagnosticsReport { aggregate, attempts }
    }

    fn ensure_capacity_for_insert(&mut self) -> bool {
        let Ok(max_len) = usize::try_from(self.policy.max_tracked_attempts_count) else {
            return false;
        };
        if self.attempts.len() < max_len {
            return true;
        }
        let oldest_retired_key = self
            .attempts
            .iter()
            .filter(|(_, state)| !state.is_active)
            .min_by_key(|(_, state)| state.generation.as_u64())
            .map(|(key, _)| key.clone());
        if let Some(key) = oldest_retired_key {
            self.attempts.remove(&key);
            return true;
        }
        false
    }

    fn insert_retired_generation(&mut self, drv_key: String, generation: AttemptGeneration) {
        insert_bounded_retired_gen(
            &mut self.retired_generations,
            drv_key,
            generation,
            self.policy.max_tracked_attempts_count,
        );
    }
}

fn apply_measurement(
    state: &mut AttemptTrackState,
    policy: &LiveCpuClassifierPolicy,
    measurement: &LiveCpuRawMeasurement,
    retired_generations: &mut BTreeMap<String, AttemptGeneration>,
    max_tracked_attempts_count: u32,
) {
    match measurement {
        LiveCpuRawMeasurement::Started { scope, .. } => state.apply_started(scope),
        LiveCpuRawMeasurement::Sampled {
            observed_at_usec,
            cumulative_cpu_usec,
        } => state.apply_sample(policy, *observed_at_usec, *cumulative_cpu_usec),
        LiveCpuRawMeasurement::Unreadable { observed_at_usec } => state.apply_unreadable(*observed_at_usec),
        LiveCpuRawMeasurement::Retired { reason, .. } => {
            state.apply_retired(*reason);
            insert_bounded_retired_gen(
                retired_generations,
                state.drv_key.clone(),
                state.generation,
                max_tracked_attempts_count,
            );
        }
    }
}

fn insert_bounded_retired_gen(
    retired_generations: &mut BTreeMap<String, AttemptGeneration>,
    drv_key: String,
    generation: AttemptGeneration,
    max_entries_count: u32,
) {
    let Ok(max_len) = usize::try_from(max_entries_count.saturating_mul(2)) else {
        return;
    };
    if !retired_generations.contains_key(&drv_key)
        && retired_generations.len() >= max_len
        && let Some(oldest_key) =
            retired_generations.iter().min_by_key(|(_, gen_val)| gen_val.as_u64()).map(|(key, _)| key.clone())
    {
        retired_generations.remove(&oldest_key);
    }
    retired_generations.insert(drv_key, generation);
}

struct AggregateCounts {
    active_attempts_count: u32,
    observed_attempts_count: u32,
    unavailable_attempts_count: u32,
    unknown_attempts_count: u32,
    completed_attempts_count: u32,
    normalization_denominator_cores: u32,
    observed_cores_sum_milli: u64,
    dropped_samples_count: u64,
}

fn build_aggregate_summary(policy: &LiveCpuClassifierPolicy, counts: AggregateCounts) -> LiveCpuAggregateSummary {
    if counts.observed_attempts_count == 0 || counts.normalization_denominator_cores == 0 {
        return LiveCpuAggregateSummary {
            schema: LIVE_CPU_DIAGNOSTICS_SCHEMA.to_string(),
            scope_label: LIVE_CPU_SCOPE_LABEL.to_string(),
            non_claim: LIVE_CPU_DIAGNOSTICS_NON_CLAIM.to_string(),
            active_attempts_count: counts.active_attempts_count,
            observed_attempts_count: 0,
            unavailable_attempts_count: counts.unavailable_attempts_count,
            unknown_attempts_count: counts.unknown_attempts_count,
            completed_attempts_count: counts.completed_attempts_count,
            normalization_denominator_cores: 0,
            observed_cores_used_milli: None,
            observed_utilization_per_mille: None,
            aggregate_class: LiveCpuActivityClass::Unknown,
            dropped_samples_count: counts.dropped_samples_count,
        };
    }

    debug_assert!(counts.observed_attempts_count <= counts.active_attempts_count);
    debug_assert!(counts.normalization_denominator_cores >= 1);
    let bounded_milli = u32::try_from(counts.observed_cores_sum_milli.min(u64::from(MAX_MILLI_CORES_BOUND)))
        .unwrap_or(MAX_MILLI_CORES_BOUND);
    let observed_cores_used = MilliCores::from_milli_cores(bounded_milli);
    let denominator = AllocatedCores::new(counts.normalization_denominator_cores.min(MAX_ALLOCATED_CORES))
        .unwrap_or(AllocatedCores(1));
    let util_raw = u128::from(bounded_milli).checked_div(u128::from(denominator.as_u32())).unwrap_or(0);
    let util_bounded = u32::try_from(util_raw.min(u128::from(MAX_PER_MILLE_BOUND))).unwrap_or(MAX_PER_MILLE_BOUND);
    let observed_utilization = UtilizationPerMille::from_per_mille(util_bounded);
    let aggregate_class = classify_instant_rate(policy, observed_cores_used, observed_utilization, denominator);

    LiveCpuAggregateSummary {
        schema: LIVE_CPU_DIAGNOSTICS_SCHEMA.to_string(),
        scope_label: LIVE_CPU_SCOPE_LABEL.to_string(),
        non_claim: LIVE_CPU_DIAGNOSTICS_NON_CLAIM.to_string(),
        active_attempts_count: counts.active_attempts_count,
        observed_attempts_count: counts.observed_attempts_count,
        unavailable_attempts_count: counts.unavailable_attempts_count,
        unknown_attempts_count: counts.unknown_attempts_count,
        completed_attempts_count: counts.completed_attempts_count,
        normalization_denominator_cores: counts.normalization_denominator_cores,
        observed_cores_used_milli: Some(observed_cores_used),
        observed_utilization_per_mille: Some(observed_utilization),
        aggregate_class,
        dropped_samples_count: counts.dropped_samples_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(drv_key: &str, generation: u64, allocated_cores: u32) -> LiveCpuAttemptMeta {
        LiveCpuAttemptMeta {
            attempt_id: generation,
            drv_key: drv_key.to_string(),
            generation: AttemptGeneration::new(generation).unwrap(),
            allocated_cores: AllocatedCores::new(allocated_cores).unwrap(),
        }
    }

    fn started(meta: &LiveCpuAttemptMeta, ts_ms: u64) -> LiveCpuSampleEvent {
        LiveCpuSampleEvent {
            meta: meta.clone(),
            measurement: LiveCpuRawMeasurement::Started {
                observed_at_usec: MonotonicTimestampUsec::from_usec(ts_ms * USEC_PER_MS),
                scope: LiveCpuScopeAvailability::OwnedAttemptCgroupV2 {
                    rel_path: "/user.slice/attempt-1".to_string(),
                },
            },
        }
    }

    fn sample(meta: &LiveCpuAttemptMeta, ts_ms: u64, cpu_ms: u64) -> LiveCpuSampleEvent {
        LiveCpuSampleEvent {
            meta: meta.clone(),
            measurement: LiveCpuRawMeasurement::Sampled {
                observed_at_usec: MonotonicTimestampUsec::from_usec(ts_ms * USEC_PER_MS),
                cumulative_cpu_usec: CumulativeCpuUsec::from_usec(cpu_ms * USEC_PER_MS),
            },
        }
    }

    #[test]
    fn synthetic_first_sample_short_window_non_monotonic_and_counter_reset_recover_cleanly() {
        let mut policy = LiveCpuClassifierPolicy::standard();
        policy.sustained_window_ms = 2_000;
        policy.stale_after_ms = 3_000;
        let mut tracker = LiveCpuTracker::new(policy).unwrap();
        let m = meta("drv-a", 1, 4);

        tracker.ingest(&started(&m, 1_000));
        // 1. First sample -> Unknown(FirstSample), no rate.
        tracker.ingest(&sample(&m, 1_000, 100));
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(1_000 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].activity_class, LiveCpuActivityClass::Unknown);
        assert_eq!(rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::FirstSample));
        assert_eq!(rep.attempts[0].latest_cores_used_milli, None);

        // 2. Insufficient sustained window (1_000 ms < 2_000 ms), 3_800 ms CPU / 1_000 ms wall -> 3_800
        //    milli-cores.
        tracker.ingest(&sample(&m, 2_000, 3_900));
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(2_000 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].activity_class, LiveCpuActivityClass::Unknown);
        assert_eq!(rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::InsufficientElapsedWindow));
        assert_eq!(rep.attempts[0].latest_cores_used_milli, Some(MilliCores::from_milli_cores(3_800)));

        // 3. Zero/non-monotonic wall interval (dt <= 0) -> Unknown(NonMonotonicInterval), no
        //    panic/division.
        tracker.ingest(&sample(&m, 2_000, 4_000));
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(2_000 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].activity_class, LiveCpuActivityClass::Unknown);
        assert_eq!(rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::NonMonotonicInterval));
        assert_eq!(rep.attempts[0].latest_cores_used_milli, None);

        tracker.ingest(&sample(&m, 1_500, 4_100));
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(2_100 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::NonMonotonicInterval));

        // 4. Counter reset (curr_cpu < prev_cpu) -> Unknown(CounterReset), re-anchored cleanly.
        tracker.ingest(&sample(&m, 2_500, 50));
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(2_500 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].activity_class, LiveCpuActivityClass::Unknown);
        assert_eq!(rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::CounterReset));
        assert_eq!(rep.attempts[0].latest_cores_used_milli, None);

        // 5. Clean recovery after counter reset across sustained window (2_500ms -> 3_500ms -> 4_500ms at
        //    ~3.5 cores = Heavy).
        tracker.ingest(&sample(&m, 3_500, 3_550));
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(3_500 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].activity_class, LiveCpuActivityClass::Unknown);
        assert_eq!(rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::InsufficientElapsedWindow));

        tracker.ingest(&sample(&m, 4_500, 7_050));
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(4_500 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].activity_class, LiveCpuActivityClass::Heavy);
        assert_eq!(rep.attempts[0].unknown_reason, None);
        assert_eq!(rep.attempts[0].latest_cores_used_milli, Some(MilliCores::from_milli_cores(3_500)));
        assert_eq!(rep.attempts[0].latest_utilization_per_mille, Some(UtilizationPerMille::from_per_mille(875)));
    }

    #[test]
    fn missing_unreadable_and_stale_samples_never_emit_idle_and_recover_cleanly() {
        let mut policy = LiveCpuClassifierPolicy::standard();
        policy.sustained_window_ms = 1_000;
        policy.stale_after_ms = 1_500;
        let mut tracker = LiveCpuTracker::new(policy).unwrap();
        let m = meta("drv-b", 1, 4);

        tracker.ingest(&started(&m, 1_000));
        tracker.ingest(&sample(&m, 1_000, 0));
        tracker.ingest(&sample(&m, 2_000, 10)); // 10 milli-cores -> Idle
        let rep = tracker.report(MonotonicTimestampUsec::from_usec(2_000 * USEC_PER_MS));
        assert_eq!(rep.attempts[0].activity_class, LiveCpuActivityClass::Idle);

        // Stale sample (> 1_500ms old at t=4_000ms) -> Unknown(StaleSample), NOT Idle!
        let stale_rep = tracker.report(MonotonicTimestampUsec::from_usec(4_000 * USEC_PER_MS));
        assert_eq!(stale_rep.attempts[0].activity_class, LiveCpuActivityClass::Unknown);
        assert_eq!(stale_rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::StaleSample));
        assert_eq!(stale_rep.attempts[0].latest_cores_used_milli, None);
        assert!(!stale_rep.attempts[0].is_coverage_available);

        // Unreadable sample -> Unknown(UnreadableSample), NOT Idle!
        tracker.ingest(&LiveCpuSampleEvent {
            meta: m.clone(),
            measurement: LiveCpuRawMeasurement::Unreadable {
                observed_at_usec: MonotonicTimestampUsec::from_usec(4_100 * USEC_PER_MS),
            },
        });
        let unred_rep = tracker.report(MonotonicTimestampUsec::from_usec(4_100 * USEC_PER_MS));
        assert_eq!(unred_rep.attempts[0].activity_class, LiveCpuActivityClass::Unknown);
        assert_eq!(unred_rep.attempts[0].unknown_reason, Some(LiveCpuUnknownReason::UnreadableSample));
        assert_eq!(unred_rep.attempts[0].latest_cores_used_milli, None);

        // Clean recovery on subsequent valid samples (low parallelism: ~1.0 core on 4 allocated).
        tracker.ingest(&sample(&m, 4_500, 1_000));
        tracker.ingest(&sample(&m, 5_500, 2_000));
        let rec_rep = tracker.report(MonotonicTimestampUsec::from_usec(5_500 * USEC_PER_MS));
        assert_eq!(rec_rep.attempts[0].activity_class, LiveCpuActivityClass::LowParallelism);
        assert_eq!(rec_rep.attempts[0].latest_cores_used_milli, Some(MilliCores::from_milli_cores(1_000)));
    }

    #[test]
    fn aggregate_discloses_observed_vs_active_explicit_denominator_and_generation_fencing() {
        let mut policy = LiveCpuClassifierPolicy::standard();
        policy.sustained_window_ms = 500;
        let mut tracker = LiveCpuTracker::new(policy).unwrap();

        let m1 = meta("drv-observed", 1, 4);
        let m2 = meta("drv-unavail", 2, 8);
        let m3 = meta("drv-single", 3, 2);

        tracker.ingest(&started(&m1, 1_000));
        tracker.ingest(&sample(&m1, 1_000, 0));
        tracker.ingest(&sample(&m1, 2_000, 2_000)); // 2.0 cores used on 4 allocated -> Moderate (500 per mille)

        tracker.ingest(&LiveCpuSampleEvent {
            meta: m2.clone(),
            measurement: LiveCpuRawMeasurement::Started {
                observed_at_usec: MonotonicTimestampUsec::from_usec(1_000 * USEC_PER_MS),
                scope: LiveCpuScopeAvailability::Unavailable {
                    reason: LiveCpuUnavailableReason::DelegationLcaNotWritable,
                },
            },
        });

        tracker.ingest(&started(&m3, 1_000));
        tracker.ingest(&sample(&m3, 1_000, 0)); // FirstSample only -> Unknown

        let rep = tracker.report(MonotonicTimestampUsec::from_usec(2_000 * USEC_PER_MS));
        assert_eq!(rep.aggregate.schema, LIVE_CPU_DIAGNOSTICS_SCHEMA);
        assert_eq!(rep.aggregate.scope_label, LIVE_CPU_SCOPE_LABEL);
        assert_eq!(rep.aggregate.active_attempts_count, 3);
        assert_eq!(rep.aggregate.observed_attempts_count, 1);
        assert_eq!(rep.aggregate.unavailable_attempts_count, 1);
        assert_eq!(rep.aggregate.unknown_attempts_count, 2);
        // Normalization denominator includes ONLY observed attempt (4 cores), NEVER m2's 8 or host CPU
        // count!
        assert_eq!(rep.aggregate.normalization_denominator_cores, 4);
        assert_eq!(rep.aggregate.observed_cores_used_milli, Some(MilliCores::from_milli_cores(2_000)));
        assert_eq!(rep.aggregate.observed_utilization_per_mille, Some(UtilizationPerMille::from_per_mille(500)));

        // Retire m1 generation 1 -> stale sample for generation 1 is rejected!
        assert!(tracker.retire_attempt("drv-observed", m1.generation, LiveCpuRetirementReason::Cancelled));
        assert_eq!(tracker.ingest(&sample(&m1, 2_500, 3_000)), LiveCpuIngestDisposition::StaleGenerationIgnored);
        let after_retire = tracker.report(MonotonicTimestampUsec::from_usec(2_500 * USEC_PER_MS));
        assert_eq!(after_retire.aggregate.active_attempts_count, 2);
        assert_eq!(after_retire.aggregate.observed_attempts_count, 0);
        assert_eq!(after_retire.aggregate.normalization_denominator_cores, 0);
        assert_eq!(after_retire.aggregate.aggregate_class, LiveCpuActivityClass::Unknown);
    }
}
