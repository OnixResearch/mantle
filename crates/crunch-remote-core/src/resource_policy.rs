//! Bounded decisions only. Facts are supplied by authenticated adapters; none
//! of these structures grants worker, project, signer, or machine authority.
use alloc::string::String;
use alloc::vec::Vec;

use crate::attempt::RemoteAttemptId;
use crate::attempt::RemoteFenceGeneration;
use crate::attempt::RemoteJobId;
use crate::resource::Capacity;
use crate::resource::reserve_capacity;

pub const MAX_CLASSES: usize = 128;
pub const MAX_HISTORY: usize = 4096;
pub const MAX_PUBLIC_ID_BYTES: usize = 128;
pub const OBSERVATION_SCHEMA: &str = "mantle-resource-observation-v2";
pub const NON_CLAIMS: [&str; 6] = [
    "no future resource sufficiency",
    "no whole-build correctness",
    "no host isolation proof",
    "no fair billing claim",
    "no production performance claim",
    "no simulated service acceptance",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reason {
    InvalidInput,
    BoundExceeded,
    HistoryMissing,
    HistoryStale,
    HistoryIncompatible,
    HistoryUntrusted,
    HistoryOutlier,
    HistoryDuplicate,
    HistoryInsufficient,
    HistoryUsed,
    StaticFallback,
    NoClass,
    QuotaDenied,
    Disabled,
    NotPositiveOom,
    RetryLimit,
    ClassLimit,
    ChargeLimit,
    WallTimeLimit,
    FenceInvalid,
    SharingDenied,
    SignatureDenied,
    IdentityDenied,
    PolicyDenied,
    PlatformDenied,
    OutputsDenied,
    CasUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Quantities {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
}
impl Quantities {
    pub fn fits(self, required: Self) -> bool {
        reserve_capacity(
            Capacity {
                cpu_units: self.cpu_units,
                memory_bytes: self.memory_bytes,
                scratch_bytes: self.scratch_bytes,
            },
            Capacity {
                cpu_units: required.cpu_units,
                memory_bytes: required.memory_bytes,
                scratch_bytes: required.scratch_bytes,
            },
        )
        .is_ok()
    }
    pub fn valid(self) -> bool {
        self.cpu_units > 0 && self.memory_bytes > 0 && self.scratch_bytes > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Features {
    pub architecture: String,
    pub platform: String,
    pub trust_domain: String,
    pub isolation: String,
    pub kvm: bool,
}
impl Features {
    fn compatible(&self, required: &Self) -> bool {
        self.architecture == required.architecture
            && self.platform == required.platform
            && self.trust_domain == required.trust_domain
            && self.isolation == required.isolation
            && (!required.kvm || self.kvm)
    }
    pub fn valid(&self) -> bool {
        [&self.architecture, &self.platform, &self.trust_domain, &self.isolation]
            .iter()
            .all(|s| valid_id(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct MachineClass {
    pub id: String,
    pub features: Features,
    pub quantities: Quantities,
    /// Public ordering from the caller's reviewed machine-class catalog.
    pub order: u32,
    pub reservation_units: u64,
    pub available: bool,
    /// Derived by existing accelerator/token and platform admission, never a grant.
    pub existing_admission_compatible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Outcome {
    Success,
    Failed,
    Cancelled,
    WorkerLost,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OomCategory {
    None,
    Unknown,
    LinuxCgroupV2Kill,
    VerifiedPlatformOom,
}

/// Missing dimensions are unknown, not zero. Only known dimensions may raise
/// historical requests. Sampled scratch is retained separately and never
/// promoted into a measured high-water mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Measurements {
    pub cpu_units: Option<u32>,
    pub memory_bytes: Option<u64>,
    pub scratch_bytes: Option<u64>,
}
impl From<Quantities> for Measurements {
    fn from(value: Quantities) -> Self {
        Self {
            cpu_units: Some(value.cpu_units),
            memory_bytes: Some(value.memory_bytes),
            scratch_bytes: Some(value.scratch_bytes),
        }
    }
}

// r[impl remote_builds.resource_observations]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Observation {
    pub schema: String,
    pub attempt_id: RemoteAttemptId,
    pub action_family: String,
    pub features: Features,
    pub policy_version: u32,
    pub machine_class: String,
    pub declared: Quantities,
    pub selected: Quantities,
    pub measured: Measurements,
    pub scratch_sampled_peak_bytes: Option<u64>,
    pub queue_ms: Option<u64>,
    pub execution_ms: Option<u64>,
    pub terminal_unix_s: u64,
    pub cpu_ms: Option<u64>,
    pub io_bytes: Option<u64>,
    pub executor_output_bytes: Option<u64>,
    pub process_peak_rss_bytes: Option<u64>,
    pub transfer_bytes: Option<u64>,
    pub oom: OomCategory,
    pub outcome: Outcome,
    pub predecessor: Option<RemoteAttemptId>,
    pub collector_version: String,
    pub trusted_collector: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct SelectionPolicy {
    pub version: u32,
    pub min_samples: u32,
    pub max_age_s: u64,
    pub margin_basis_points: u32,
    pub max_measurement: Quantities,
    pub max_duration_ms: u64,
    pub max_io_bytes: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuotaFacts {
    pub project_remaining_units: u64,
    pub account_remaining_units: u64,
}
impl QuotaFacts {
    pub fn permits(self, units: u64) -> bool {
        units <= self.project_remaining_units && units <= self.account_remaining_units
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Selection {
    pub class_index: usize,
    pub requested: Quantities,
    pub samples: u32,
    /// Input order, one exclusion per observation; bounded by MAX_HISTORY.
    pub exclusions: Vec<(usize, Reason)>,
    pub reason: Reason,
    pub policy_version: u32,
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PUBLIC_ID_BYTES
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.:/".contains(&b))
}
fn validate_policy(policy: SelectionPolicy) -> Result<(), Reason> {
    let has_version = policy.version != 0;
    let has_samples = policy.min_samples != 0;
    let has_bounded_samples = usize::try_from(policy.min_samples).is_ok_and(|count| count <= MAX_HISTORY);
    let has_age = policy.max_age_s != 0;
    let has_bounded_margin = policy.margin_basis_points <= 100_000;
    let has_valid_measurement = policy.max_measurement.valid();
    let has_ms_budget = policy.max_duration_ms != 0;
    let has_io_budget = policy.max_io_bytes != 0;
    let is_valid = has_version
        && has_samples
        && has_bounded_samples
        && has_age
        && has_bounded_margin
        && has_valid_measurement
        && has_ms_budget
        && has_io_budget;
    if !is_valid {
        return Err(Reason::InvalidInput);
    }
    debug_assert!(policy.version != 0);
    debug_assert!(policy.min_samples != 0);
    Ok(())
}
/// Identity and schema facts that must match before a replayed observation is
/// usable at all.
fn observation_identity_matches(o: &Observation, family: &str, features: &Features, p: SelectionPolicy) -> bool {
    let has_same_schema = o.schema == OBSERVATION_SCHEMA;
    let has_same_policy = o.policy_version == p.version;
    let has_same_family = o.action_family == family;
    let has_compatible_features = o.features.compatible(features);
    has_same_schema && has_same_policy && has_same_family && has_compatible_features
}
/// Outcome and declaration facts that must match before a replayed observation
/// is usable at all.
fn observation_outcome_matches(o: &Observation) -> bool {
    let is_success = o.outcome == Outcome::Success;
    let has_valid_class = valid_id(&o.machine_class);
    let has_valid_collector = valid_id(&o.collector_version);
    let has_declared_fit = o.selected.fits(o.declared);
    is_success && has_valid_class && has_valid_collector && has_declared_fit
}
fn observation_is_stale(o: &Observation, now: u64, p: SelectionPolicy) -> bool {
    let is_in_the_future = o.terminal_unix_s > now;
    let age_s = now.saturating_sub(o.terminal_unix_s);
    is_in_the_future || age_s > p.max_age_s
}
fn observation_has_measurement(o: &Observation) -> bool {
    let has_cpu = o.measured.cpu_units.is_some();
    let has_memory = o.measured.memory_bytes.is_some();
    let has_scratch = o.measured.scratch_bytes.is_some();
    has_cpu || has_memory || has_scratch
}
fn measurement_exceeds_policy(o: &Observation, p: SelectionPolicy) -> bool {
    let is_cpu_outlier =
        o.measured.cpu_units.is_some_and(|n| n > p.max_measurement.cpu_units || n > o.selected.cpu_units);
    let is_memory_outlier = o
        .measured
        .memory_bytes
        .is_some_and(|n| n > p.max_measurement.memory_bytes || n > o.selected.memory_bytes);
    let is_scratch_outlier = o
        .measured
        .scratch_bytes
        .is_some_and(|n| n > p.max_measurement.scratch_bytes || n > o.selected.scratch_bytes);
    is_cpu_outlier || is_memory_outlier || is_scratch_outlier
}
fn timing_exceeds_policy(o: &Observation, p: SelectionPolicy) -> bool {
    let is_sampled_scratch_outlier = o.scratch_sampled_peak_bytes.is_some_and(|n| n > p.max_measurement.scratch_bytes);
    let is_execution_outlier = o.execution_ms.is_some_and(|n| n > p.max_duration_ms);
    let is_queue_outlier = o.queue_ms.is_some_and(|n| n > p.max_duration_ms);
    let is_cpu_time_outlier =
        o.cpu_ms.is_some_and(|n| n > p.max_duration_ms.saturating_mul(u64::from(o.selected.cpu_units)));
    is_sampled_scratch_outlier || is_execution_outlier || is_queue_outlier || is_cpu_time_outlier
}
fn transfer_exceeds_policy(o: &Observation, p: SelectionPolicy) -> bool {
    let is_io_outlier = o.io_bytes.is_some_and(|n| n > p.max_io_bytes);
    let is_transfer_outlier = o.transfer_bytes.is_some_and(|n| n > p.max_io_bytes);
    let is_output_outlier = o.executor_output_bytes.is_some_and(|n| n > p.max_io_bytes);
    let is_peak_rss_outlier = o.process_peak_rss_bytes.is_some_and(|n| n > p.max_measurement.memory_bytes);
    is_io_outlier || is_transfer_outlier || is_output_outlier || is_peak_rss_outlier
}
fn observation_is_outlier(o: &Observation, p: SelectionPolicy) -> bool {
    measurement_exceeds_policy(o, p) || timing_exceeds_policy(o, p) || transfer_exceeds_policy(o, p)
}
fn observation_reason(
    o: &Observation,
    family: &str,
    features: &Features,
    now: u64,
    p: SelectionPolicy,
) -> Option<Reason> {
    if !o.trusted_collector {
        return Some(Reason::HistoryUntrusted);
    }
    if !observation_identity_matches(o, family, features, p) {
        return Some(Reason::HistoryIncompatible);
    }
    if !observation_outcome_matches(o) {
        return Some(Reason::HistoryIncompatible);
    }
    if observation_is_stale(o, now, p) {
        return Some(Reason::HistoryStale);
    }
    if !observation_has_measurement(o) {
        return Some(Reason::HistoryMissing);
    }
    if observation_is_outlier(o, p) {
        return Some(Reason::HistoryOutlier);
    }
    debug_assert!(o.trusted_collector);
    debug_assert!(observation_identity_matches(o, family, features, p));
    None
}
fn with_margin(value: u64, basis_points: u32) -> Result<u64, Reason> {
    let scale = 10_000_u64.checked_add(u64::from(basis_points)).ok_or(Reason::BoundExceeded)?;
    value
        .checked_mul(scale)
        .and_then(|n| n.checked_add(9_999))
        .map(|n| n / 10_000)
        .ok_or(Reason::BoundExceeded)
}
fn class_key(c: &MachineClass) -> (u32, u64, u32, u64, u64, &str) {
    (
        c.order,
        c.reservation_units,
        c.quantities.cpu_units,
        c.quantities.memory_bytes,
        c.quantities.scratch_bytes,
        &c.id,
    )
}

/// Replay inputs for one bounded machine-class selection decision. The declared
/// minima come from admission and the history from authenticated observations;
/// neither carries worker, store or project authority.
pub struct SelectionRequest<'a> {
    pub declared: Quantities,
    pub features: &'a Features,
    pub family: &'a str,
    pub classes: &'a [MachineClass],
    pub history: &'a [Observation],
    pub now: u64,
    pub quota: QuotaFacts,
    pub policy: SelectionPolicy,
}

/// Aggregated usable history for one selection decision.
struct HistorySummary {
    peak: Quantities,
    dimension_samples: [u32; 3],
    samples: u32,
    exclusions: Vec<(usize, Reason)>,
}

fn class_is_admissible(class: &MachineClass) -> bool {
    let has_valid_id = valid_id(&class.id);
    let has_valid_features = class.features.valid();
    let has_valid_quantities = class.quantities.valid();
    let has_reservation = class.reservation_units != 0;
    has_valid_id && has_valid_features && has_valid_quantities && has_reservation
}

fn validate_selection_inputs(request: &SelectionRequest<'_>) -> Result<(), Reason> {
    if request.classes.len() > MAX_CLASSES || request.history.len() > MAX_HISTORY {
        return Err(Reason::BoundExceeded);
    }
    if !request.declared.valid() || !request.features.valid() || !valid_id(request.family) {
        return Err(Reason::InvalidInput);
    }
    for (index, class) in request.classes.iter().enumerate() {
        let has_duplicate_id = request.classes[..index].iter().any(|other| other.id == class.id);
        if !class_is_admissible(class) || has_duplicate_id {
            return Err(Reason::InvalidInput);
        }
    }
    Ok(())
}

fn summarize_history(request: &SelectionRequest<'_>) -> HistorySummary {
    let mut peak = Quantities {
        cpu_units: 0,
        memory_bytes: 0,
        scratch_bytes: 0,
    };
    let mut dimension_samples = [0_u32; 3];
    let mut samples = 0_u32;
    let mut exclusions = Vec::with_capacity(request.history.len());
    for (index, observation) in request.history.iter().enumerate() {
        let has_duplicate_attempt = request.history[..index].iter().any(|old| old.attempt_id == observation.attempt_id);
        let reason = if has_duplicate_attempt {
            Some(Reason::HistoryDuplicate)
        } else {
            observation_reason(observation, request.family, request.features, request.now, request.policy)
        };
        if let Some(reason) = reason {
            exclusions.push((index, reason));
            continue;
        }
        samples += 1;
        if let Some(value) = observation.measured.cpu_units {
            peak.cpu_units = peak.cpu_units.max(value);
            dimension_samples[0] += 1;
        }
        if let Some(value) = observation.measured.memory_bytes {
            peak.memory_bytes = peak.memory_bytes.max(value);
            dimension_samples[1] += 1;
        }
        if let Some(value) = observation.measured.scratch_bytes {
            peak.scratch_bytes = peak.scratch_bytes.max(value);
            dimension_samples[2] += 1;
        }
    }
    debug_assert!(exclusions.len() <= request.history.len());
    debug_assert!(dimension_samples.iter().all(|count| *count <= samples));
    HistorySummary {
        peak,
        dimension_samples,
        samples,
        exclusions,
    }
}

fn enlarged_quantities(request: &SelectionRequest<'_>, history: &HistorySummary) -> Result<Quantities, Reason> {
    let mut requested = request.declared;
    if history.dimension_samples[0] >= request.policy.min_samples {
        let margin = u32::try_from(with_margin(u64::from(history.peak.cpu_units), request.policy.margin_basis_points)?)
            .map_err(|_| Reason::BoundExceeded)?;
        requested.cpu_units = request.declared.cpu_units.max(margin);
    }
    if history.dimension_samples[1] >= request.policy.min_samples {
        requested.memory_bytes = request
            .declared
            .memory_bytes
            .max(with_margin(history.peak.memory_bytes, request.policy.margin_basis_points)?);
    }
    if history.dimension_samples[2] >= request.policy.min_samples {
        requested.scratch_bytes = request
            .declared
            .scratch_bytes
            .max(with_margin(history.peak.scratch_bytes, request.policy.margin_basis_points)?);
    }
    debug_assert!(requested.cpu_units >= request.declared.cpu_units);
    debug_assert!(requested.memory_bytes >= request.declared.memory_bytes);
    Ok(requested)
}

fn history_reason(request: &SelectionRequest<'_>, history: &HistorySummary) -> Reason {
    let has_enough_samples = history.dimension_samples.iter().any(|count| *count >= request.policy.min_samples);
    if has_enough_samples {
        return Reason::HistoryUsed;
    }
    if request.history.is_empty() {
        return Reason::HistoryMissing;
    }
    Reason::HistoryInsufficient
}

fn class_is_eligible(class: &MachineClass, features: &Features, requested: Quantities) -> bool {
    let is_available = class.available;
    let is_admission_compatible = class.existing_admission_compatible;
    let has_compatible_features = class.features.compatible(features);
    let has_requested_fit = class.quantities.fits(requested);
    is_available && is_admission_compatible && has_compatible_features && has_requested_fit
}

fn choose_class(request: &SelectionRequest<'_>, requested: Quantities) -> Result<usize, Reason> {
    let has_class = request.classes.iter().any(|class| class_is_eligible(class, request.features, requested));
    let selected = request
        .classes
        .iter()
        .enumerate()
        .filter(|(_, class)| {
            class_is_eligible(class, request.features, requested) && request.quota.permits(class.reservation_units)
        })
        .min_by(|(_, left), (_, right)| class_key(left).cmp(&class_key(right)));
    let Some((class_index, _)) = selected else {
        return Err(if has_class {
            Reason::QuotaDenied
        } else {
            Reason::NoClass
        });
    };
    debug_assert!(class_index < request.classes.len());
    debug_assert!(request.classes[class_index].quantities.fits(requested));
    Ok(class_index)
}

// r[impl remote_builds.replayable_resource_selection]
pub fn select(request: SelectionRequest<'_>) -> Result<Selection, Reason> {
    validate_policy(request.policy)?;
    validate_selection_inputs(&request)?;
    let history = summarize_history(&request);
    let requested = enlarged_quantities(&request, &history)?;
    let reason = history_reason(&request, &history);
    let class_index = choose_class(&request, requested)?;
    debug_assert!(request.policy.version != 0);
    debug_assert!(class_index < request.classes.len());
    Ok(Selection {
        class_index,
        requested,
        samples: history.samples,
        exclusions: history.exclusions,
        reason,
        policy_version: request.policy.version,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectionMode {
    ObserveOnly,
    ProjectOptIn,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Controls {
    pub mode: SelectionMode,
    pub project_opted_in: bool,
    pub selection_enabled: bool,
    pub oom_retry_enabled: bool,
    pub quota_enforcement_enabled: bool,
    pub sharing_enabled: bool,
}
impl Controls {
    pub fn selection_active(self) -> bool {
        self.mode == SelectionMode::ProjectOptIn && self.project_opted_in && self.selection_enabled
    }
    pub fn retry_active(self) -> bool {
        self.project_opted_in && self.oom_retry_enabled && self.mode == SelectionMode::ProjectOptIn
    }
    pub fn sharing_active(self) -> bool {
        self.project_opted_in && self.sharing_enabled && self.mode == SelectionMode::ProjectOptIn
    }
    pub fn quota_active(self) -> bool {
        self.project_opted_in && self.quota_enforcement_enabled && self.mode == SelectionMode::ProjectOptIn
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RolloutDecision {
    pub static_decision: Result<Selection, Reason>,
    pub historical_decision: Result<Selection, Reason>,
    pub active: Result<Selection, Reason>,
}
// r[impl remote_builds.resource_policy_rollout]
pub fn rollout(
    controls: Controls,
    static_decision: Result<Selection, Reason>,
    historical_decision: Result<Selection, Reason>,
) -> RolloutDecision {
    let active = if controls.selection_active() {
        historical_decision.clone()
    } else {
        static_decision.clone()
    };
    RolloutDecision {
        static_decision,
        historical_decision,
        active,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EscalationPolicy {
    pub version: u32,
    pub max_retries: u32,
    pub largest_class_order: u32,
    pub max_cumulative_units: u64,
    pub max_wall_ms: u64,
}
#[derive(Debug, Clone)]
pub struct OomEvidence<'a> {
    pub job: &'a RemoteJobId,
    pub attempt: &'a RemoteAttemptId,
    pub fence: RemoteFenceGeneration,
    pub platform: &'a str,
    pub category: OomCategory,
    /// Adapter verified authenticated worker event against the current fence and platform contract.
    pub trusted_positive_platform_evidence: bool,
}
#[derive(Debug, Clone)]
pub struct RetryContext<'a> {
    pub job: &'a RemoteJobId,
    pub attempt: &'a RemoteAttemptId,
    pub fence: RemoteFenceGeneration,
    pub next_attempt: &'a RemoteAttemptId,
    pub next_fence: RemoteFenceGeneration,
    pub failed: bool,
    pub retries: u32,
    pub cumulative_units: u64,
    pub elapsed_ms: u64,
    pub next_wall_budget_ms: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EscalationPlan {
    pub predecessor: RemoteAttemptId,
    pub next_attempt: RemoteAttemptId,
    pub next_fence: RemoteFenceGeneration,
    pub class_index: usize,
    pub policy_version: u32,
}
/// Inputs for one positive-OOM retry escalation decision. The evidence must be
/// an authenticated positive OOM observation bound to the current fence.
pub struct EscalationRequest<'a> {
    pub controls: Controls,
    pub context: RetryContext<'a>,
    pub evidence: OomEvidence<'a>,
    pub prior: &'a MachineClass,
    pub classes: &'a [MachineClass],
    pub quota: QuotaFacts,
    pub policy: EscalationPolicy,
}

fn evidence_is_positive_oom(context: &RetryContext<'_>, evidence: &OomEvidence<'_>, prior: &MachineClass) -> bool {
    let has_failed_attempt = context.failed;
    let is_trusted_evidence = evidence.trusted_positive_platform_evidence;
    let is_oom_category =
        matches!(evidence.category, OomCategory::LinuxCgroupV2Kill | OomCategory::VerifiedPlatformOom);
    let has_matching_platform = evidence.platform == prior.features.platform;
    has_failed_attempt && is_trusted_evidence && is_oom_category && has_matching_platform
}

fn evidence_matches_fence(context: &RetryContext<'_>, evidence: &OomEvidence<'_>) -> bool {
    let has_matching_job = evidence.job == context.job;
    let has_matching_attempt = evidence.attempt == context.attempt;
    let has_matching_fence = evidence.fence == context.fence;
    let has_new_attempt = context.next_attempt != context.attempt;
    let has_advanced_fence = context.fence.advance().ok() == Some(context.next_fence);
    has_matching_job && has_matching_attempt && has_matching_fence && has_new_attempt && has_advanced_fence
}

fn validate_escalation_inputs(request: &EscalationRequest<'_>) -> Result<(), Reason> {
    if request.policy.version == 0 || request.classes.len() > MAX_CLASSES || request.context.next_wall_budget_ms == 0 {
        return Err(Reason::InvalidInput);
    }
    if !class_is_admissible(request.prior) {
        return Err(Reason::InvalidInput);
    }
    for (index, class) in request.classes.iter().enumerate() {
        let has_duplicate_id = request.classes[..index].iter().any(|other| other.id == class.id);
        if !class_is_admissible(class) || has_duplicate_id {
            return Err(Reason::InvalidInput);
        }
    }
    Ok(())
}

fn choose_escalation_class(request: &EscalationRequest<'_>) -> Result<usize, Reason> {
    let prior = request.prior;
    let larger = |class: &MachineClass| {
        let is_available = class.available;
        let is_admission_compatible = class.existing_admission_compatible;
        let has_compatible_features = class.features.compatible(&prior.features);
        let has_prior_fit = class.quantities.fits(prior.quantities);
        let has_more_memory = class.quantities.memory_bytes > prior.quantities.memory_bytes;
        let is_within_order = class.order <= request.policy.largest_class_order;
        is_available
            && is_admission_compatible
            && has_compatible_features
            && has_prior_fit
            && has_more_memory
            && is_within_order
    };
    let has_larger = request.classes.iter().any(&larger);
    if !has_larger {
        return Err(Reason::ClassLimit);
    }
    let has_permitted =
        request.classes.iter().any(|class| larger(class) && request.quota.permits(class.reservation_units));
    if !has_permitted {
        return Err(Reason::QuotaDenied);
    }
    let chosen = request
        .classes
        .iter()
        .enumerate()
        .filter(|(_, class)| larger(class) && request.quota.permits(class.reservation_units))
        .filter(|(_, class)| {
            request
                .context
                .cumulative_units
                .checked_add(class.reservation_units)
                .is_some_and(|total| total <= request.policy.max_cumulative_units)
        })
        .min_by(|(_, left), (_, right)| class_key(left).cmp(&class_key(right)))
        .ok_or(Reason::ChargeLimit)?;
    debug_assert!(chosen.0 < request.classes.len());
    debug_assert!(request.classes[chosen.0].quantities.memory_bytes > prior.quantities.memory_bytes);
    Ok(chosen.0)
}

// r[impl remote_builds.positive_oom_retry]
pub fn plan_escalation(request: EscalationRequest<'_>) -> Result<EscalationPlan, Reason> {
    let context = &request.context;
    let evidence = &request.evidence;
    if !request.controls.retry_active() {
        return Err(Reason::Disabled);
    }
    if !evidence_is_positive_oom(context, evidence, request.prior) {
        return Err(Reason::NotPositiveOom);
    }
    if !evidence_matches_fence(context, evidence) {
        return Err(Reason::FenceInvalid);
    }
    validate_escalation_inputs(&request)?;
    if context.retries >= request.policy.max_retries {
        return Err(Reason::RetryLimit);
    }
    let wall_ms = context.elapsed_ms.checked_add(context.next_wall_budget_ms);
    if wall_ms.is_none_or(|total| total > request.policy.max_wall_ms) {
        return Err(Reason::WallTimeLimit);
    }
    let class_index = choose_escalation_class(&request)?;
    debug_assert!(request.policy.version != 0);
    debug_assert!(class_index < request.classes.len());
    Ok(EscalationPlan {
        predecessor: context.attempt.clone(),
        next_attempt: context.next_attempt.clone(),
        next_fence: context.next_fence,
        class_index,
        policy_version: request.policy.version,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct SharingChecks {
    pub request_and_action_match: bool,
    pub trusted_producer_signature: bool,
    pub policy_compatible: bool,
    pub platform_compatible: bool,
    pub verified_output_identities: bool,
    pub current_cas_available: bool,
}
#[derive(Debug, Clone)]
pub struct SharingScope<'a> {
    pub id: &'a str,
    pub producers: &'a [&'a str],
    pub consumers: &'a [&'a str],
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharingLink {
    pub producer: String,
    pub consumer: String,
    pub result: String,
    pub scope: Option<String>,
}
/// Inputs for one authorized cross-project result sharing decision.
pub struct SharingRequest<'a> {
    pub controls: Controls,
    pub producer: &'a str,
    pub consumer: &'a str,
    pub result: &'a str,
    pub scope: Option<SharingScope<'a>>,
    pub checks: SharingChecks,
}

// r[impl remote_builds.authorized_result_sharing]
pub fn authorize_sharing(request: SharingRequest<'_>) -> Result<SharingLink, Reason> {
    if !valid_id(request.producer) || !valid_id(request.consumer) || !valid_id(request.result) {
        return Err(Reason::InvalidInput);
    }
    if !request.checks.request_and_action_match {
        return Err(Reason::IdentityDenied);
    }
    if !request.checks.trusted_producer_signature {
        return Err(Reason::SignatureDenied);
    }
    if !request.checks.policy_compatible {
        return Err(Reason::PolicyDenied);
    }
    if !request.checks.platform_compatible {
        return Err(Reason::PlatformDenied);
    }
    if !request.checks.verified_output_identities {
        return Err(Reason::OutputsDenied);
    }
    if !request.checks.current_cas_available {
        return Err(Reason::CasUnavailable);
    }
    let scope_id = if request.producer == request.consumer {
        None
    } else {
        if !request.controls.sharing_active() {
            return Err(Reason::SharingDenied);
        }
        let scope = request.scope.ok_or(Reason::SharingDenied)?;
        let has_valid_scope_id = valid_id(scope.id);
        let has_bounded_producers = scope.producers.len() <= MAX_CLASSES;
        let has_bounded_consumers = scope.consumers.len() <= MAX_CLASSES;
        let has_included_producer = scope.producers.contains(&request.producer);
        let has_included_consumer = scope.consumers.contains(&request.consumer);
        let is_admissible = has_valid_scope_id
            && has_bounded_producers
            && has_bounded_consumers
            && has_included_producer
            && has_included_consumer;
        if !is_admissible {
            return Err(Reason::SharingDenied);
        }
        Some(String::from(scope.id))
    };
    debug_assert!(valid_id(request.producer));
    debug_assert!(valid_id(request.consumer));
    Ok(SharingLink {
        producer: String::from(request.producer),
        consumer: String::from(request.consumer),
        result: String::from(request.result),
        scope: scope_id,
    })
}
