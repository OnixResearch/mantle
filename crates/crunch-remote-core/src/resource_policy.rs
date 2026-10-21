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
    if policy.version == 0
        || policy.min_samples == 0
        || policy.min_samples as usize > MAX_HISTORY
        || policy.max_age_s == 0
        || policy.margin_basis_points > 100_000
        || !policy.max_measurement.valid()
        || policy.max_duration_ms == 0
        || policy.max_io_bytes == 0
    {
        return Err(Reason::InvalidInput);
    }
    Ok(())
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
    if o.schema != OBSERVATION_SCHEMA
        || o.policy_version != p.version
        || o.action_family != family
        || !o.features.compatible(features)
        || o.outcome != Outcome::Success
        || !valid_id(&o.machine_class)
        || !valid_id(&o.collector_version)
        || !o.selected.fits(o.declared)
    {
        return Some(Reason::HistoryIncompatible);
    }
    if o.terminal_unix_s > now || now - o.terminal_unix_s > p.max_age_s {
        return Some(Reason::HistoryStale);
    }
    if o.measured.cpu_units.is_none() && o.measured.memory_bytes.is_none() && o.measured.scratch_bytes.is_none() {
        return Some(Reason::HistoryMissing);
    }
    if o.measured.cpu_units.is_some_and(|n| n > p.max_measurement.cpu_units || n > o.selected.cpu_units)
        || o.measured
            .memory_bytes
            .is_some_and(|n| n > p.max_measurement.memory_bytes || n > o.selected.memory_bytes)
        || o.measured
            .scratch_bytes
            .is_some_and(|n| n > p.max_measurement.scratch_bytes || n > o.selected.scratch_bytes)
        || o.scratch_sampled_peak_bytes.is_some_and(|n| n > p.max_measurement.scratch_bytes)
        || o.execution_ms.is_some_and(|n| n > p.max_duration_ms)
        || o.queue_ms.is_some_and(|n| n > p.max_duration_ms)
        || o.cpu_ms.is_some_and(|n| n > p.max_duration_ms.saturating_mul(u64::from(o.selected.cpu_units)))
        || o.io_bytes.is_some_and(|n| n > p.max_io_bytes)
        || o.transfer_bytes.is_some_and(|n| n > p.max_io_bytes)
        || o.executor_output_bytes.is_some_and(|n| n > p.max_io_bytes)
        || o.process_peak_rss_bytes.is_some_and(|n| n > p.max_measurement.memory_bytes)
    {
        return Some(Reason::HistoryOutlier);
    }
    None
}
fn with_margin(value: u64, basis_points: u32) -> Result<u64, Reason> {
    value
        .checked_mul(10_000 + u64::from(basis_points))
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

// r[impl remote_builds.replayable_resource_selection]
#[allow(
    clippy::too_many_arguments,
    reason = "Explicit borrowed replay inputs keep observations separate from authority facts"
)]
pub fn select(
    declared: Quantities,
    features: &Features,
    family: &str,
    classes: &[MachineClass],
    history: &[Observation],
    now: u64,
    quota: QuotaFacts,
    policy: SelectionPolicy,
) -> Result<Selection, Reason> {
    validate_policy(policy)?;
    if classes.len() > MAX_CLASSES || history.len() > MAX_HISTORY {
        return Err(Reason::BoundExceeded);
    }
    if !declared.valid() || !features.valid() || !valid_id(family) {
        return Err(Reason::InvalidInput);
    }
    for (i, c) in classes.iter().enumerate() {
        if !valid_id(&c.id)
            || !c.features.valid()
            || !c.quantities.valid()
            || c.reservation_units == 0
            || classes[..i].iter().any(|other| other.id == c.id)
        {
            return Err(Reason::InvalidInput);
        }
    }
    let mut peak = Quantities {
        cpu_units: 0,
        memory_bytes: 0,
        scratch_bytes: 0,
    };
    let mut dimension_samples = [0_u32; 3];
    let mut samples = 0_u32;
    let mut exclusions = Vec::new();
    for (i, o) in history.iter().enumerate() {
        let reason = if history[..i].iter().any(|old| old.attempt_id == o.attempt_id) {
            Some(Reason::HistoryDuplicate)
        } else {
            observation_reason(o, family, features, now, policy)
        };
        if let Some(reason) = reason {
            exclusions.push((i, reason));
            continue;
        }
        samples += 1;
        if let Some(value) = o.measured.cpu_units {
            peak.cpu_units = peak.cpu_units.max(value);
            dimension_samples[0] += 1;
        }
        if let Some(value) = o.measured.memory_bytes {
            peak.memory_bytes = peak.memory_bytes.max(value);
            dimension_samples[1] += 1;
        }
        if let Some(value) = o.measured.scratch_bytes {
            peak.scratch_bytes = peak.scratch_bytes.max(value);
            dimension_samples[2] += 1;
        }
    }
    let mut requested = declared;
    if dimension_samples[0] >= policy.min_samples {
        requested.cpu_units = declared.cpu_units.max(
            u32::try_from(with_margin(u64::from(peak.cpu_units), policy.margin_basis_points)?)
                .map_err(|_| Reason::BoundExceeded)?,
        );
    }
    if dimension_samples[1] >= policy.min_samples {
        requested.memory_bytes = declared.memory_bytes.max(with_margin(peak.memory_bytes, policy.margin_basis_points)?);
    }
    if dimension_samples[2] >= policy.min_samples {
        requested.scratch_bytes =
            declared.scratch_bytes.max(with_margin(peak.scratch_bytes, policy.margin_basis_points)?);
    }
    let reason = if dimension_samples.iter().any(|count| *count >= policy.min_samples) {
        Reason::HistoryUsed
    } else if history.is_empty() {
        Reason::HistoryMissing
    } else {
        Reason::HistoryInsufficient
    };
    let eligible = |c: &MachineClass| {
        c.available
            && c.existing_admission_compatible
            && c.features.compatible(features)
            && c.quantities.fits(requested)
    };
    let has_class = classes.iter().any(eligible);
    let selected = classes
        .iter()
        .enumerate()
        .filter(|(_, c)| eligible(c) && quota.permits(c.reservation_units))
        .min_by(|(_, a), (_, b)| class_key(a).cmp(&class_key(b)));
    let Some((class_index, _)) = selected else {
        return Err(if has_class {
            Reason::QuotaDenied
        } else {
            Reason::NoClass
        });
    };
    Ok(Selection {
        class_index,
        requested,
        samples,
        exclusions,
        reason,
        policy_version: policy.version,
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
// r[impl remote_builds.positive_oom_retry]
pub fn plan_escalation(
    controls: Controls,
    context: RetryContext<'_>,
    evidence: OomEvidence<'_>,
    prior: &MachineClass,
    classes: &[MachineClass],
    quota: QuotaFacts,
    policy: EscalationPolicy,
) -> Result<EscalationPlan, Reason> {
    if !controls.retry_active() {
        return Err(Reason::Disabled);
    }
    if !context.failed
        || !evidence.trusted_positive_platform_evidence
        || !matches!(evidence.category, OomCategory::LinuxCgroupV2Kill | OomCategory::VerifiedPlatformOom)
        || evidence.platform != prior.features.platform
    {
        return Err(Reason::NotPositiveOom);
    }
    if evidence.job != context.job
        || evidence.attempt != context.attempt
        || evidence.fence != context.fence
        || context.next_attempt == context.attempt
        || context.fence.advance().ok() != Some(context.next_fence)
    {
        return Err(Reason::FenceInvalid);
    }
    if policy.version == 0 || classes.len() > MAX_CLASSES || context.next_wall_budget_ms == 0 {
        return Err(Reason::InvalidInput);
    }
    if !valid_id(&prior.id) || !prior.features.valid() || !prior.quantities.valid() || prior.reservation_units == 0 {
        return Err(Reason::InvalidInput);
    }
    for (index, class) in classes.iter().enumerate() {
        if !valid_id(&class.id)
            || !class.features.valid()
            || !class.quantities.valid()
            || class.reservation_units == 0
            || classes[..index].iter().any(|other| other.id == class.id)
        {
            return Err(Reason::InvalidInput);
        }
    }
    if context.retries >= policy.max_retries {
        return Err(Reason::RetryLimit);
    }
    if context.elapsed_ms.checked_add(context.next_wall_budget_ms).is_none_or(|n| n > policy.max_wall_ms) {
        return Err(Reason::WallTimeLimit);
    }
    let larger = |c: &MachineClass| {
        c.available
            && c.existing_admission_compatible
            && c.features.compatible(&prior.features)
            && c.quantities.fits(prior.quantities)
            && c.quantities.memory_bytes > prior.quantities.memory_bytes
            && c.order <= policy.largest_class_order
    };
    if !classes.iter().any(larger) {
        return Err(Reason::ClassLimit);
    }
    if !classes.iter().any(|c| larger(c) && quota.permits(c.reservation_units)) {
        return Err(Reason::QuotaDenied);
    }
    let chosen = classes
        .iter()
        .enumerate()
        .filter(|(_, c)| larger(c) && quota.permits(c.reservation_units))
        .filter(|(_, c)| {
            context
                .cumulative_units
                .checked_add(c.reservation_units)
                .is_some_and(|n| n <= policy.max_cumulative_units)
        })
        .min_by(|(_, a), (_, b)| class_key(a).cmp(&class_key(b)))
        .ok_or(Reason::ChargeLimit)?;
    Ok(EscalationPlan {
        predecessor: context.attempt.clone(),
        next_attempt: context.next_attempt.clone(),
        next_fence: context.next_fence,
        class_index: chosen.0,
        policy_version: policy.version,
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
// r[impl remote_builds.authorized_result_sharing]
pub fn authorize_sharing(
    controls: Controls,
    producer: &str,
    consumer: &str,
    result: &str,
    scope: Option<SharingScope<'_>>,
    checks: SharingChecks,
) -> Result<SharingLink, Reason> {
    if !valid_id(producer) || !valid_id(consumer) || !valid_id(result) {
        return Err(Reason::InvalidInput);
    }
    if !checks.request_and_action_match {
        return Err(Reason::IdentityDenied);
    }
    if !checks.trusted_producer_signature {
        return Err(Reason::SignatureDenied);
    }
    if !checks.policy_compatible {
        return Err(Reason::PolicyDenied);
    }
    if !checks.platform_compatible {
        return Err(Reason::PlatformDenied);
    }
    if !checks.verified_output_identities {
        return Err(Reason::OutputsDenied);
    }
    if !checks.current_cas_available {
        return Err(Reason::CasUnavailable);
    }
    let scope_id = if producer == consumer {
        None
    } else {
        if !controls.sharing_active() {
            return Err(Reason::SharingDenied);
        }
        let scope = scope.ok_or(Reason::SharingDenied)?;
        if !valid_id(scope.id)
            || scope.producers.len() > MAX_CLASSES
            || scope.consumers.len() > MAX_CLASSES
            || !scope.producers.contains(&producer)
            || !scope.consumers.contains(&consumer)
        {
            return Err(Reason::SharingDenied);
        }
        Some(String::from(scope.id))
    };
    Ok(SharingLink {
        producer: String::from(producer),
        consumer: String::from(consumer),
        result: String::from(result),
        scope: scope_id,
    })
}
