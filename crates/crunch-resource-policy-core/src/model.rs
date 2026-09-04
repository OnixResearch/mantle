// machine-artifact-public: resource.action-family
// machine-artifact-public: resource.resource-observation
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceQuantities {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformRequirements {
    pub architecture: String,
    pub platform: String,
    pub kvm_required: bool,
    pub trust_tier: String,
    pub isolation: String,
    pub required_features: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredResourceRequirements {
    pub minima: ResourceQuantities,
    pub platform: PlatformRequirements,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFamilyInput {
    pub request_kind: String,
    pub system: String,
    pub builder_class: String,
    pub sandbox_mode: String,
    pub network_mode: String,
    pub required_features: Vec<String>,
    pub semantic_accelerator_classes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFamilyIdentity {
    pub schema: String,
    pub identity_blake3: String,
    pub request_kind: String,
    pub system: String,
    pub builder_class: String,
    pub sandbox_mode: String,
    pub network_mode: String,
    pub required_features: Vec<String>,
    pub semantic_accelerator_classes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachineClass {
    pub class_id: String,
    pub endpoint_ids: Vec<String>,
    pub ordinal: u32,
    pub architecture: String,
    pub platform: String,
    pub kvm_available: bool,
    pub trust_tier: String,
    pub isolation: String,
    pub features: Vec<String>,
    pub capacity: ResourceQuantities,
    pub reservation_charge_units: u64,
    pub available: bool,
    pub onixos_source_blake3: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuotaFacts {
    pub project_remaining_units: u64,
    pub account_remaining_units: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceTiming {
    pub queue_ms: u64,
    pub execution_ms: u64,
    pub terminal_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceMeasurements {
    pub cpu_time_ms: u64,
    pub peak_memory_bytes: u64,
    pub scratch_peak_bytes: u64,
    pub io_bytes: u64,
    pub transfer_bytes: u64,
    pub wall_time_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OomEvidenceCategory {
    LinuxCgroupV2OomKill,
    WindowsJobObjectMemoryLimit,
    DarwinJetsam,
    AmbiguousExit,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OomEvidence {
    pub category: OomEvidenceCategory,
    pub platform: String,
    pub evidence_ref: Option<String>,
    pub trusted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalOutcome {
    Succeeded,
    FailedOom,
    FailedOther,
    Cancelled,
    WorkerLost,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceObservationInput {
    pub attempt_id: String,
    pub action_family_blake3: String,
    pub platform_identity: String,
    pub machine_class_id: String,
    pub declared: ResourceQuantities,
    pub selected: ResourceQuantities,
    pub timing: ResourceTiming,
    pub measurements: ResourceMeasurements,
    pub oom_evidence: OomEvidence,
    pub terminal_outcome: TerminalOutcome,
    pub retry_predecessor_attempt_id: Option<String>,
    pub collector_id: String,
    pub collector_version: String,
    pub compatibility_policy_id: String,
    pub observed_at_unix_s: u64,
    pub trusted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceObservation {
    pub schema: String,
    pub observation_blake3: String,
    pub attempt_id: String,
    pub action_family_blake3: String,
    pub platform_identity: String,
    pub machine_class_id: String,
    pub declared: ResourceQuantities,
    pub selected: ResourceQuantities,
    pub timing: ResourceTiming,
    pub measurements: ResourceMeasurements,
    pub oom_evidence: OomEvidence,
    pub terminal_outcome: TerminalOutcome,
    pub retry_predecessor_attempt_id: Option<String>,
    pub collector_id: String,
    pub collector_version: String,
    pub compatibility_policy_id: String,
    pub observed_at_unix_s: u64,
    pub trusted: bool,
    pub non_claim: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HistoryReasonCode {
    Accepted,
    Missing,
    SchemaUnsupported,
    DigestInvalid,
    Duplicate,
    Stale,
    FutureDated,
    ActionFamilyMismatch,
    PlatformMismatch,
    PolicyMismatch,
    MachineClassUnknown,
    Untrusted,
    MeasurementOutlier,
}

impl HistoryReasonCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "history-accepted",
            Self::Missing => "history-missing",
            Self::SchemaUnsupported => "history-schema-unsupported",
            Self::DigestInvalid => "history-digest-invalid",
            Self::Duplicate => "history-duplicate",
            Self::Stale => "history-stale",
            Self::FutureDated => "history-future-dated",
            Self::ActionFamilyMismatch => "history-action-family-mismatch",
            Self::PlatformMismatch => "history-platform-mismatch",
            Self::PolicyMismatch => "history-policy-mismatch",
            Self::MachineClassUnknown => "history-machine-class-unknown",
            Self::Untrusted => "history-untrusted",
            Self::MeasurementOutlier => "history-measurement-outlier",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryDecision {
    pub observation_blake3: Option<String>,
    pub accepted: bool,
    pub reason: HistoryReasonCode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationFilterReport {
    pub accepted: Vec<ResourceObservation>,
    pub decisions: Vec<HistoryDecision>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationFilterRequest {
    pub now_unix_s: u64,
    pub action_family_blake3: String,
    pub platform_identity: String,
    pub compatibility_policy_id: String,
    pub known_machine_class_ids: Vec<String>,
    pub observation_age_secs_max: u64,
    pub measurement_bytes_max: u64,
    pub observations: Vec<ResourceObservation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionFallbackRule {
    StaticOnInsufficientHistory,
    RejectOnInsufficientHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSelectionPolicy {
    pub schema: String,
    pub policy_id: String,
    pub policy_version: String,
    pub sample_count_min: u32,
    pub observation_age_secs_max: u64,
    pub memory_margin_basis_points: u32,
    pub scratch_margin_basis_points: u32,
    pub measurement_bytes_max: u64,
    pub fallback: SelectionFallbackRule,
}

impl Default for ResourceSelectionPolicy {
    fn default() -> Self {
        Self {
            schema: crate::RESOURCE_SELECTION_POLICY_SCHEMA.into(),
            policy_id: "mantle-resource-selection-default".into(),
            policy_version: "v1".into(),
            sample_count_min: crate::DEFAULT_SAMPLE_COUNT_MIN,
            observation_age_secs_max: crate::DEFAULT_OBSERVATION_AGE_SECS_MAX,
            memory_margin_basis_points: crate::DEFAULT_MEMORY_MARGIN_BASIS_POINTS,
            scratch_margin_basis_points: crate::DEFAULT_SCRATCH_MARGIN_BASIS_POINTS,
            measurement_bytes_max: crate::MAX_MEASUREMENT_BYTES,
            fallback: SelectionFallbackRule::StaticOnInsufficientHistory,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourcePolicyMode {
    ObserveOnly,
    Enforce,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceFeatureControls {
    pub mode: ResourcePolicyMode,
    pub project_opted_in: bool,
    pub historical_selection_enabled: bool,
    pub oom_retry_enabled: bool,
    pub quota_enforcement_enabled: bool,
    pub result_sharing_enabled: bool,
}

impl Default for ResourceFeatureControls {
    fn default() -> Self {
        Self {
            mode: ResourcePolicyMode::ObserveOnly,
            project_opted_in: false,
            historical_selection_enabled: false,
            oom_retry_enabled: false,
            quota_enforcement_enabled: false,
            result_sharing_enabled: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSelectionRequest {
    pub now_unix_s: u64,
    pub action_family: ActionFamilyIdentity,
    pub declared: DeclaredResourceRequirements,
    pub machine_classes: Vec<MachineClass>,
    pub quota: QuotaFacts,
    pub observations: Vec<ResourceObservation>,
    pub policy: ResourceSelectionPolicy,
    pub controls: ResourceFeatureControls,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionReasonCode {
    StaticMinimumSelected,
    HistoryAccepted,
    HistoryInsufficient,
    HistoryRaisedMinimum,
    ObserveOnlyStaticRetained,
    ProjectNotOptedIn,
    HistoricalSelectionDisabled,
    QuotaEnforcementDisabled,
    QuotaAccepted,
    HistoricalClassUnavailable,
    QuotaDeniedPolicyFallback,
}

impl SelectionReasonCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StaticMinimumSelected => "static-minimum-selected",
            Self::HistoryAccepted => "history-accepted",
            Self::HistoryInsufficient => "history-insufficient",
            Self::HistoryRaisedMinimum => "history-raised-minimum",
            Self::ObserveOnlyStaticRetained => "observe-only-static-retained",
            Self::ProjectNotOptedIn => "project-not-opted-in",
            Self::HistoricalSelectionDisabled => "historical-selection-disabled",
            Self::QuotaEnforcementDisabled => "quota-enforcement-disabled",
            Self::QuotaAccepted => "quota-accepted",
            Self::HistoricalClassUnavailable => "historical-class-unavailable",
            Self::QuotaDeniedPolicyFallback => "quota-denied-policy-fallback",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSelectionDecision {
    pub schema: String,
    pub decision_blake3: String,
    pub policy_id: String,
    pub policy_version: String,
    pub action_family_blake3: String,
    pub declared_minima: ResourceQuantities,
    pub effective_minima: ResourceQuantities,
    pub eligible_class_ids: Vec<String>,
    pub static_class_id: String,
    pub policy_class_id: String,
    pub scheduled_class_id: String,
    pub compatible_observation_blake3s: Vec<String>,
    pub history_decisions: Vec<HistoryDecision>,
    pub reason_codes: Vec<SelectionReasonCode>,
    pub observe_only: bool,
    pub non_claim: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRetryPolicy {
    pub schema: String,
    pub policy_id: String,
    pub retry_count_max: u32,
    pub class_ordinal_max: u32,
    pub cumulative_charge_units_max: u64,
    pub wall_time_ms_max: u64,
}

impl Default for ResourceRetryPolicy {
    fn default() -> Self {
        Self {
            schema: crate::RESOURCE_RETRY_POLICY_SCHEMA.into(),
            policy_id: "mantle-resource-retry-default".into(),
            retry_count_max: crate::DEFAULT_RETRY_COUNT_MAX,
            class_ordinal_max: crate::DEFAULT_CLASS_ORDINAL_MAX,
            cumulative_charge_units_max: crate::DEFAULT_RETRY_CHARGE_UNITS_MAX,
            wall_time_ms_max: crate::DEFAULT_RETRY_WALL_TIME_MS_MAX,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryHistoryEntry {
    pub attempt_id: String,
    pub machine_class_id: String,
    pub charge_units: u64,
    pub wall_time_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRetryRequest {
    pub predecessor_attempt_id: String,
    pub predecessor_fence_generation: u64,
    pub current_machine_class_id: String,
    pub oom_evidence: OomEvidence,
    pub retry_history: Vec<RetryHistoryEntry>,
    pub eligible_classes: Vec<MachineClass>,
    pub quota: QuotaFacts,
    pub policy: ResourceRetryPolicy,
    pub controls: ResourceFeatureControls,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetryReasonCode {
    PositiveOomEvidence,
    RetryDisabled,
    OomEvidenceMissing,
    OomEvidenceUntrusted,
    OomPlatformMismatch,
    RetryLimitExhausted,
    LargerClassUnavailable,
    ClassLimitExceeded,
    ChargeLimitExceeded,
    WallTimeLimitExceeded,
    QuotaDenied,
}

impl RetryReasonCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PositiveOomEvidence => "positive-oom-evidence",
            Self::RetryDisabled => "oom-retry-disabled",
            Self::OomEvidenceMissing => "oom-evidence-missing",
            Self::OomEvidenceUntrusted => "oom-evidence-untrusted",
            Self::OomPlatformMismatch => "oom-platform-mismatch",
            Self::RetryLimitExhausted => "oom-retry-limit-exhausted",
            Self::LargerClassUnavailable => "oom-larger-class-unavailable",
            Self::ClassLimitExceeded => "oom-class-limit-exceeded",
            Self::ChargeLimitExceeded => "oom-charge-limit-exceeded",
            Self::WallTimeLimitExceeded => "oom-wall-time-limit-exceeded",
            Self::QuotaDenied => "oom-quota-denied",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRetryPlan {
    pub schema: String,
    pub retry_intent_blake3: String,
    pub predecessor_attempt_id: String,
    pub predecessor_fence_generation: u64,
    pub successor_fence_generation: u64,
    pub prior_machine_class_id: String,
    pub next_machine_class_id: String,
    pub trigger_evidence_ref: String,
    pub cumulative_charge_units: u64,
    pub cumulative_wall_time_ms: u64,
    pub new_fenced_attempt_required: bool,
    pub reason: RetryReasonCode,
    pub non_claim: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UsageReconciliationKind {
    Completed,
    Cancelled,
    WorkerLost,
    PartialObservation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageUnitSchedule {
    pub schedule_id: String,
    pub cpu_ms_per_unit: u64,
    pub memory_byte_ms_per_unit: u64,
    pub transfer_bytes_per_unit: u64,
    pub charge_units_max: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageAmounts {
    pub cpu_ms: u64,
    pub memory_byte_ms: u64,
    pub transfer_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageReservationRequest {
    pub attempt_id: String,
    pub project_id: String,
    pub account_id: String,
    pub window_start_unix_s: u64,
    pub window_end_unix_s: u64,
    pub reserved_units: u64,
    pub project_limit_units: u64,
    pub account_limit_units: u64,
    pub policy_id: String,
    pub schedule: UsageUnitSchedule,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageReservation {
    pub schema: String,
    pub reservation_blake3: String,
    pub attempt_id: String,
    pub project_id: String,
    pub account_id: String,
    pub window_start_unix_s: u64,
    pub window_end_unix_s: u64,
    pub reserved_units: u64,
    pub policy_id: String,
    pub schedule_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageReconciliationRequest {
    pub attempt_id: String,
    pub kind: UsageReconciliationKind,
    pub observed: Option<UsageAmounts>,
    pub policy_id: String,
    pub schedule: UsageUnitSchedule,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageReconciliation {
    pub schema: String,
    pub reconciliation_blake3: String,
    pub reservation_blake3: String,
    pub attempt_id: String,
    pub kind: UsageReconciliationKind,
    pub charged_units: u64,
    pub released_units: u64,
    pub policy_id: String,
    pub schedule_id: String,
    pub reason_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageLedgerState {
    pub schema: String,
    pub reservations: Vec<UsageReservation>,
    pub reconciliations: Vec<UsageReconciliation>,
}

impl Default for UsageLedgerState {
    fn default() -> Self {
        Self {
            schema: crate::USAGE_LEDGER_SCHEMA.into(),
            reservations: Vec::new(),
            reconciliations: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LedgerMutationDisposition {
    Add,
    Reuse,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageLedgerMutationPlan {
    pub disposition: LedgerMutationDisposition,
    pub prior_state_blake3: String,
    pub next_state_blake3: String,
    pub next_state: UsageLedgerState,
    pub reservation: Option<UsageReservation>,
    pub reconciliation: Option<UsageReconciliation>,
    pub quota_granted: bool,
    pub reason_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageScopeSummary {
    pub scope_id: String,
    pub active_reserved_units: u64,
    pub charged_units: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageLedgerSummary {
    pub schema: String,
    pub state_blake3: String,
    pub reservation_count: u32,
    pub reconciliation_count: u32,
    pub project_units: Vec<UsageScopeSummary>,
    pub account_units: Vec<UsageScopeSummary>,
    pub reason_codes: Vec<String>,
    pub non_claim: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SharingScopeKind {
    Private,
    Project,
    Named,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultSharingPolicy {
    pub schema: String,
    pub policy_id: String,
    pub scope: SharingScopeKind,
    pub project_id: Option<String>,
    pub allowed_producers: Vec<String>,
    pub allowed_consumers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultReuseFacts {
    pub producer_project_id: String,
    pub consumer_project_id: String,
    pub producer_result_blake3: String,
    pub consumer_request_blake3: String,
    pub request_identity_matches: bool,
    pub action_identity_matches: bool,
    pub producer_signature_trusted: bool,
    pub policy_compatible: bool,
    pub platform_compatible: bool,
    pub output_identity_verified: bool,
    pub cas_available: bool,
    pub strong_reuse_admitted: bool,
    pub authorization_decision_blake3: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReuseReasonCode {
    Usable,
    SharingDisabled,
    ProducerScopeDenied,
    ConsumerScopeDenied,
    RequestMismatch,
    ActionMismatch,
    SignatureUntrusted,
    PolicyMismatch,
    PlatformMismatch,
    OutputIdentityInvalid,
    CasUnavailable,
    StrongReuseRejected,
}

impl ReuseReasonCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Usable => "result-reuse-usable",
            Self::SharingDisabled => "result-sharing-disabled",
            Self::ProducerScopeDenied => "result-producer-scope-denied",
            Self::ConsumerScopeDenied => "result-consumer-scope-denied",
            Self::RequestMismatch => "result-request-mismatch",
            Self::ActionMismatch => "result-action-mismatch",
            Self::SignatureUntrusted => "result-signature-untrusted",
            Self::PolicyMismatch => "result-policy-mismatch",
            Self::PlatformMismatch => "result-platform-mismatch",
            Self::OutputIdentityInvalid => "result-output-identity-invalid",
            Self::CasUnavailable => "result-cas-unavailable",
            Self::StrongReuseRejected => "result-strong-reuse-rejected",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerConsumerEvidenceLink {
    pub producer_result_blake3: String,
    pub consumer_request_blake3: String,
    pub authorization_decision_blake3: String,
    pub sharing_policy_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultReuseDecision {
    pub schema: String,
    pub decision_blake3: String,
    pub usable: bool,
    pub reason: ReuseReasonCode,
    pub evidence_link: Option<ProducerConsumerEvidenceLink>,
    pub non_claim: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSource {
    pub repository: String,
    pub revision: String,
    pub source_blake3: String,
    pub license: String,
    pub license_blake3: String,
    pub adaptation: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBenchmarkWorkload {
    pub workload_id: String,
    pub source: BenchmarkSource,
    pub fixed_input_blake3: String,
    pub platform: PlatformRequirements,
    pub expected_completion_class: String,
    pub latency_ms_max: u64,
    pub memory_bytes_max: u64,
    pub transfer_bytes_max: u64,
    pub usage_units_max: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBenchmarkObservation {
    pub workload_id: String,
    pub static_class_id: String,
    pub policy_class_id: String,
    pub completion_class: String,
    pub compatible: bool,
    pub correct_output: bool,
    pub throughput_units: u64,
    pub latency_ms: u64,
    pub memory_bytes: u64,
    pub transfer_bytes: u64,
    pub usage_units: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBenchmarkReport {
    pub schema: String,
    pub report_blake3: String,
    pub baseline_policy_id: String,
    pub candidate_policy_id: String,
    pub workload_count: u32,
    pub correctness_passed: bool,
    pub compatibility_passed: bool,
    pub observations: Vec<ResourceBenchmarkObservation>,
    pub non_claim: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceFaultKind {
    PositiveOom,
    AmbiguousFailure,
    WorkerLoss,
    DuplicateCompletion,
    AccountingInterruption,
    CasUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceFaultCase {
    pub case_id: String,
    pub kind: ResourceFaultKind,
    pub expected_reason_code: String,
    pub expected_mutation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceFaultCampaign {
    pub schema: String,
    pub chaoscontrol_revision: String,
    pub chaoscontrol_contract_path: String,
    pub chaoscontrol_contract_blake3: String,
    pub cases: Vec<ResourceFaultCase>,
    pub non_claim: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePolicyReport {
    pub schema: String,
    pub selection: ResourceSelectionDecision,
    pub retry: Option<ResourceRetryPlan>,
    pub usage: Vec<UsageReconciliation>,
    pub reuse: Option<ResultReuseDecision>,
    pub valence_profile_id: String,
    pub reason_codes: Vec<String>,
    pub non_claim: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourcePolicyError {
    InvalidSchema,
    InvalidIdentity,
    InvalidDigest,
    InvalidBounds,
    ArithmeticOverflow,
    TooManyFeatures,
    TooManyMachineClasses,
    TooManyObservations,
    TooManyHistoryEntries,
    TooManyUsageRecords,
    TooManySharingSubjects,
    TooManyBenchmarkWorkloads,
    TooManyFaultCases,
    DuplicateIdentity,
    NoEligibleClass,
    NoClassForHistoricalMinimum,
    QuotaDenied,
    InsufficientHistory,
    RetryRejected(RetryReasonCode),
    ReservationConflict,
    ReconciliationConflict,
    ReservationMissing,
    InvalidUsageSchedule,
    ReuseRejected(ReuseReasonCode),
    BenchmarkRejected,
    Canonicalization,
}

impl ResourcePolicyError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidSchema => "resource-policy-schema-invalid",
            Self::InvalidIdentity => "resource-policy-identity-invalid",
            Self::InvalidDigest => "resource-policy-digest-invalid",
            Self::InvalidBounds => "resource-policy-bounds-invalid",
            Self::ArithmeticOverflow => "resource-policy-arithmetic-overflow",
            Self::TooManyFeatures => "resource-policy-feature-count-exceeded",
            Self::TooManyMachineClasses => "resource-policy-machine-class-count-exceeded",
            Self::TooManyObservations => "resource-policy-observation-count-exceeded",
            Self::TooManyHistoryEntries => "resource-policy-retry-history-count-exceeded",
            Self::TooManyUsageRecords => "resource-policy-usage-record-count-exceeded",
            Self::TooManySharingSubjects => "resource-policy-sharing-subject-count-exceeded",
            Self::TooManyBenchmarkWorkloads => "resource-policy-benchmark-count-exceeded",
            Self::TooManyFaultCases => "resource-policy-fault-count-exceeded",
            Self::DuplicateIdentity => "resource-policy-duplicate-identity",
            Self::NoEligibleClass => "resource-policy-no-eligible-class",
            Self::NoClassForHistoricalMinimum => "resource-policy-no-class-for-history",
            Self::QuotaDenied => "resource-policy-quota-denied",
            Self::InsufficientHistory => "resource-policy-history-insufficient",
            Self::RetryRejected(reason) => reason.as_str(),
            Self::ReservationConflict => "resource-policy-reservation-conflict",
            Self::ReconciliationConflict => "resource-policy-reconciliation-conflict",
            Self::ReservationMissing => "resource-policy-reservation-missing",
            Self::InvalidUsageSchedule => "resource-policy-usage-schedule-invalid",
            Self::ReuseRejected(reason) => reason.as_str(),
            Self::BenchmarkRejected => "resource-policy-benchmark-rejected",
            Self::Canonicalization => "resource-policy-canonicalization-failed",
        }
    }
}

impl core::fmt::Display for ResourcePolicyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl core::error::Error for ResourcePolicyError {}

pub type UsageByScope = BTreeMap<String, u64>;
