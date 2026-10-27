//! Concrete operator-configured resource selection and durable quota host.
//!
//! Admission supplies derivation minima and the coordinator's current lease
//! snapshot. Worker measurements enter only as an authenticated opaque report.
//! The schedule's memory quantity is a conservative dedicated-cgroup memory.peak
//! × wall-time bound, not a measured resident-memory integral.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use crunch_build::distributed::CatalogCandidate;
use crunch_build::distributed::PolicyEvaluation;
use crunch_build::distributed::RemoteResourceLease;
use crunch_build::distributed::RemoteResourceRequirements;
use crunch_build::distributed::RemoteWorkerResourceInventory;
use crunch_build::distributed::ResourceQuotaStore;
use crunch_build::distributed::action_family_identity;
use crunch_build::distributed::canonical_remote_resource_requirements;
use crunch_build::distributed::canonical_remote_worker_resource_inventory;
use crunch_build::distributed::evaluate_resource_policy;
use crunch_nix_gateway::api::UsageReasonCounts;
use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use crunch_remote_core::resource_policy as policy;
use crunch_remote_core::resource_quota as quota;
use serde::Deserialize;
use serde::Serialize;

use crate::remote_build::usage::RemoteAuthenticatedUsage;
use crate::remote_build::usage::RemoteUsageCompleteness;
use crate::remote_build::usage::RemoteUsageOomEvidence;

pub(crate) const MEMORY_ACCOUNTING: &str = "dedicated-cgroup-memory-peak-times-wall-ms-upper-bound-v2";
pub(crate) const OUTPUT_ACCOUNTING: &str = "executor-stdout-plus-stderr-bytes-not-network-or-nar-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourceClassConfig {
    pub id: String,
    pub features: policy::Features,
    pub quantities: policy::Quantities,
    pub order: u32,
    pub reservation_units: u64,
    pub worker_endpoint_id: String,
    pub inventory: RemoteWorkerResourceInventory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectQuotaConfig {
    pub account: String,
    pub project_units: u64,
    pub account_units: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetryPolicy {
    pub version: u32,
    pub max_retries: u32,
    pub largest_class_order: u32,
    pub max_cumulative_units: u64,
    pub max_wall_ms: u64,
}

impl From<RetryPolicy> for policy::EscalationPolicy {
    fn from(value: RetryPolicy) -> Self {
        Self {
            version: value.version,
            max_retries: value.max_retries,
            largest_class_order: value.largest_class_order,
            max_cumulative_units: value.max_cumulative_units,
            max_wall_ms: value.max_wall_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourceSharingScope {
    pub name: String,
    pub producers: BTreeSet<String>,
    pub consumers: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourcePolicyConfig {
    pub selection: policy::SelectionPolicy,
    pub retry: RetryPolicy,
    pub mode: policy::SelectionMode,
    pub selection_enabled: bool,
    pub oom_retry_enabled: bool,
    pub quota_enforcement_enabled: bool,
    pub sharing_enabled: bool,
    pub opted_in_projects: BTreeSet<String>,
    pub projects: BTreeMap<String, ProjectQuotaConfig>,
    pub schedule: quota::UnitSchedule,
    pub classes: Vec<ResourceClassConfig>,
    #[serde(default)]
    pub sharing_scopes: Vec<ResourceSharingScope>,
}

#[derive(Debug)]
pub(crate) enum ResourceHostError {
    Policy(policy::Reason),
    Quota(quota::QuotaError),
    InvalidConfiguration,
    BindingMismatch,
    WindowClosed,
}

impl fmt::Display for ResourceHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Policy(reason) => write!(f, "resource-policy-{reason:?}"),
            Self::Quota(reason) => write!(f, "resource-quota-{reason:?}"),
            Self::InvalidConfiguration => f.write_str("resource-policy-invalid-configuration"),
            Self::BindingMismatch => f.write_str("resource-policy-binding-mismatch"),
            Self::WindowClosed => f.write_str("resource-quota-window-closed"),
        }
    }
}

impl std::error::Error for ResourceHostError {}
impl From<policy::Reason> for ResourceHostError {
    fn from(value: policy::Reason) -> Self {
        Self::Policy(value)
    }
}
impl From<quota::QuotaError> for ResourceHostError {
    fn from(value: quota::QuotaError) -> Self {
        Self::Quota(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttemptBinding {
    pub job: RemoteJobId,
    pub attempt: RemoteAttemptId,
    pub fence: RemoteFenceGeneration,
    pub project: String,
    pub account: String,
    pub request_id: String,
    pub request_digest: String,
}

pub(crate) struct SelectionRequest<'a> {
    pub project: &'a str,
    pub account: &'a str,
    pub request_id: &'a str,
    pub request_digest: &'a str,
    pub requirements: &'a RemoteResourceRequirements,
    pub features: &'a policy::Features,
    pub public_action_kind: &'a str,
    pub now_unix_s: u64,
    pub live_leases: &'a [RemoteResourceLease],
}

/// Borrowed current coordinator/store capabilities, never deserializable facts
/// from a transport. The caller has already verified the consumer invocation.
pub(crate) struct VerifiedReuseRequest<'a> {
    pub producer_public_id: &'a str,
    pub coordinator: &'a crate::remote_build::RemoteCoordinatorState,
    pub consumer_project: &'a str,
    pub consumer_drv_path: &'a str,
    pub consumer_request_digest: [u8; 32],
    pub consumer_platform: &'a str,
    pub consumer_policy: &'a str,
    pub store: &'a crunch_nix_gateway::store::VerifiedStore,
    pub trusted_output_keys: &'a [String],
}

/// Not deserializable: neither request bodies nor replay evidence mint a selection.
#[derive(Debug, Clone)]
pub(crate) struct SelectedResources {
    host_identity: String,
    project: String,
    account: String,
    request_id: String,
    request_digest: String,
    requirements: RemoteResourceRequirements,
    declared: policy::Quantities,
    predecessor: Option<RemoteAttemptId>,
    worker_endpoint_id: String,
    machine_class: policy::MachineClass,
    action_family: String,
    evaluation: PolicyEvaluation,
    retry_binding: Option<(RemoteJobId, RemoteAttemptId, RemoteFenceGeneration)>,
}

impl SelectedResources {
    pub(crate) fn requirements(&self) -> &RemoteResourceRequirements {
        &self.requirements
    }
    pub(crate) fn worker_endpoint_id(&self) -> &str {
        &self.worker_endpoint_id
    }
    pub(crate) fn machine_class(&self) -> &policy::MachineClass {
        &self.machine_class
    }
    pub(crate) fn evaluation(&self) -> &PolicyEvaluation {
        &self.evaluation
    }
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
    // I3).
    #[allow(dead_code)]
    pub(crate) fn action_family(&self) -> &str {
        &self.action_family
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ResourceUsageSummary {
    pub schedule: quota::UnitSchedule,
    pub totals: quota::Summary,
    pub project_reason_counts: UsageReasonCounts,
    pub account_reason_counts: UsageReasonCounts,
    pub project_limit_units: u64,
    pub account_limit_units: u64,
    pub memory_accounting: &'static str,
    pub output_accounting: &'static str,
}
impl ResourceUsageSummary {
    pub(crate) fn into_accounting(self) -> crunch_nix_gateway::api::UsageAccounting {
        crunch_nix_gateway::api::UsageAccounting {
            schedule_version: self.schedule.version,
            cpu_ms_per_unit: self.schedule.cpu_ms_per_unit,
            memory_byte_ms_per_unit: self.schedule.memory_byte_ms_per_unit,
            executor_output_bytes_per_unit: self.schedule.executor_output_bytes_per_unit,
            project_limit_units: self.project_limit_units,
            account_limit_units: self.account_limit_units,
            project_reason_counts: self.project_reason_counts,
            account_reason_counts: self.account_reason_counts,
            unit: self.schedule.unit,
            window_id: self.schedule.window_id,
            window_start_s: self.schedule.window_start_s,
            window_end_s: self.schedule.window_end_s,
            project_reserved_units: self.totals.project_reserved,
            project_charged_units: self.totals.project_charged,
            account_reserved_units: self.totals.account_reserved,
            account_charged_units: self.totals.account_charged,
            memory_measurement: self.memory_accounting.into(),
            output_measurement: self.output_accounting.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
#[allow(dead_code)]
pub(crate) enum PendingUsage {
    Missing,
    Partial,
    WorkerLost,
    Cancelled,
}

#[derive(Debug)]
pub(crate) struct RetryResources {
    pub plan: policy::EscalationPlan,
    pub selected: SelectedResources,
    pub replay_json: serde_json::Value,
}

pub(crate) struct ResourcePolicyHost {
    config: ResourcePolicyConfig,
    host_identity: String,
    store: ResourceQuotaStore,
}

impl ResourcePolicyHost {
    /// Opening never provisions or repairs a ledger. Operators explicitly call
    /// ResourceQuotaStore::provision once, separately from request admission.
    pub(crate) fn open(config: ResourcePolicyConfig, quota_directory: &Path) -> Result<Self, ResourceHostError> {
        Self::validate_config(&config)?;
        let bytes = serde_json::to_vec(&config).map_err(|_| ResourceHostError::InvalidConfiguration)?;
        let host_identity = blake3::hash(&bytes).to_hex().to_string();
        let store = ResourceQuotaStore::open(quota_directory)?;
        Ok(Self {
            config,
            host_identity,
            store,
        })
    }

    fn validate_config(config: &ResourcePolicyConfig) -> Result<(), ResourceHostError> {
        let invalid = || ResourceHostError::InvalidConfiguration;
        let schedule = &config.schedule;
        if config.classes.is_empty()
            || config.classes.len() > policy::MAX_CLASSES
            || config.projects.is_empty()
            || config.projects.len() > quota::MAX_RESERVATIONS
            || config.opted_in_projects.iter().any(|id| !config.projects.contains_key(id))
            || config.sharing_scopes.len() > policy::MAX_CLASSES
            || schedule.version == 0
            || !policy::valid_id(&schedule.unit)
            || !policy::valid_id(&schedule.window_id)
            || schedule.window_start_s >= schedule.window_end_s
            || schedule.cpu_ms_per_unit == 0
            || schedule.memory_byte_ms_per_unit == 0
            || schedule.executor_output_bytes_per_unit == 0
            || config.retry.version == 0
            || config.retry.max_wall_ms == 0
            || config.retry.max_cumulative_units == 0
        {
            return Err(invalid());
        }
        let mut account_limits = BTreeMap::new();
        for (project, limits) in &config.projects {
            if !policy::valid_id(project)
                || !policy::valid_id(&limits.account)
                || limits.project_units == 0
                || limits.account_units == 0
                || account_limits
                    .insert(&limits.account, limits.account_units)
                    .is_some_and(|old| old != limits.account_units)
            {
                return Err(invalid());
            }
        }
        let mut scope_names = BTreeSet::new();
        for scope in &config.sharing_scopes {
            if !policy::valid_id(&scope.name)
                || !scope_names.insert(&scope.name)
                || scope.producers.is_empty()
                || scope.consumers.is_empty()
                || scope.producers.len() > policy::MAX_CLASSES
                || scope.consumers.len() > policy::MAX_CLASSES
                || scope.producers.iter().chain(&scope.consumers).any(|project| !config.projects.contains_key(project))
            {
                return Err(invalid());
            }
        }
        let mut ids = BTreeSet::new();
        for class in &config.classes {
            if !ids.insert(&class.id)
                || !policy::valid_id(&class.id)
                || !policy::valid_id(&class.worker_endpoint_id)
                || !class.features.valid()
                || !class.quantities.valid()
                || class.reservation_units == 0
            {
                return Err(invalid());
            }
            let inventory = canonical_remote_worker_resource_inventory(&class.inventory).map_err(|_| invalid())?;
            if inventory != class.inventory
                || !(policy::Quantities {
                    cpu_units: inventory.total.cpu_units,
                    memory_bytes: inventory.total.memory_bytes,
                    scratch_bytes: inventory.total.scratch_bytes,
                })
                .fits(class.quantities)
            {
                return Err(invalid());
            }
        }
        // Exercise the pure validator without conferring any runtime availability.
        let first = &config.classes[0];
        policy::select(policy::SelectionRequest {
            declared: first.quantities,
            features: &first.features,
            family: "configuration-validation",
            classes: &[],
            history: &[],
            now: schedule.window_start_s,
            quota: policy::QuotaFacts {
                project_remaining_units: 0,
                account_remaining_units: 0,
            },
            policy: config.selection,
        })
        .map(|_| ())
        .or_else(|reason| {
            if reason == policy::Reason::NoClass {
                Ok(())
            } else {
                Err(invalid())
            }
        })
    }

    pub(crate) fn retry_policy(&self) -> RetryPolicy {
        self.config.retry
    }

    pub(crate) fn controls(&self, project: &str) -> policy::Controls {
        policy::Controls {
            mode: self.config.mode,
            project_opted_in: self.config.opted_in_projects.contains(project),
            selection_enabled: self.config.selection_enabled,
            oom_retry_enabled: self.config.oom_retry_enabled,
            quota_enforcement_enabled: self.config.quota_enforcement_enabled,
            sharing_enabled: self.config.sharing_enabled,
        }
    }

    fn project(&self, project: &str, account: &str) -> Result<&ProjectQuotaConfig, ResourceHostError> {
        self.config
            .projects
            .get(project)
            .filter(|limits| limits.account == account)
            .ok_or(ResourceHostError::BindingMismatch)
    }

    fn check_window(&self, now: u64) -> Result<(), ResourceHostError> {
        let schedule = &self.config.schedule;
        if now < schedule.window_start_s || now >= schedule.window_end_s {
            return Err(ResourceHostError::WindowClosed);
        }
        Ok(())
    }

    pub(crate) fn summary(&self, project: &str, account: &str) -> Result<ResourceUsageSummary, ResourceHostError> {
        let limits = self.project(project, account)?;
        let ledger = self.store.snapshot()?;
        let mut totals = quota::Summary::default();
        let mut project_reason_counts = UsageReasonCounts::default();
        let mut account_reason_counts = UsageReasonCounts::default();
        for entry in ledger.entries.values() {
            let r = &entry.reservation;
            if r.schedule.window_id != self.config.schedule.window_id {
                continue;
            }
            if r.schedule != self.config.schedule {
                return Err(quota::QuotaError::Conflict.into());
            }
            let (reserved, charged) = match entry.reconciliation {
                quota::Reconciliation::Settled { charged_units, .. } => (0, charged_units),
                _ => (r.units, 0),
            };
            if r.project == project {
                if r.account != account {
                    return Err(quota::QuotaError::Conflict.into());
                }
                count_reconciliation(&mut project_reason_counts, &entry.reconciliation)?;
                totals.project_reserved =
                    totals.project_reserved.checked_add(reserved).ok_or(quota::QuotaError::Overflow)?;
                totals.project_charged =
                    totals.project_charged.checked_add(charged).ok_or(quota::QuotaError::Overflow)?;
            }
            if r.account == account {
                count_reconciliation(&mut account_reason_counts, &entry.reconciliation)?;
                totals.account_reserved =
                    totals.account_reserved.checked_add(reserved).ok_or(quota::QuotaError::Overflow)?;
                totals.account_charged =
                    totals.account_charged.checked_add(charged).ok_or(quota::QuotaError::Overflow)?;
            }
        }
        Ok(ResourceUsageSummary {
            schedule: self.config.schedule.clone(),
            totals,
            project_reason_counts,
            account_reason_counts,
            project_limit_units: limits.project_units,
            account_limit_units: limits.account_units,
            memory_accounting: MEMORY_ACCOUNTING,
            output_accounting: OUTPUT_ACCOUNTING,
        })
    }

    fn quota_facts(&self, project: &str, account: &str) -> Result<policy::QuotaFacts, ResourceHostError> {
        let summary = self.summary(project, account)?;
        let project_used = summary
            .totals
            .project_reserved
            .checked_add(summary.totals.project_charged)
            .ok_or(quota::QuotaError::Overflow)?;
        let account_used = summary
            .totals
            .account_reserved
            .checked_add(summary.totals.account_charged)
            .ok_or(quota::QuotaError::Overflow)?;
        Ok(policy::QuotaFacts {
            project_remaining_units: summary.project_limit_units.saturating_sub(project_used),
            account_remaining_units: summary.account_limit_units.saturating_sub(account_used),
        })
    }

    fn evaluate(&self, request: &SelectionRequest<'_>, family: &str) -> Result<PolicyEvaluation, ResourceHostError> {
        let candidates = self
            .config
            .classes
            .iter()
            .map(|class| CatalogCandidate {
                class: policy::MachineClass {
                    id: class.id.clone(),
                    features: class.features.clone(),
                    quantities: class.quantities,
                    order: class.order,
                    reservation_units: class.reservation_units,
                    available: true,
                    existing_admission_compatible: true,
                },
                worker_endpoint_id: &class.worker_endpoint_id,
                inventory: &class.inventory,
                live_leases: request.live_leases,
            })
            .collect();
        // History remains project-private. Unknown dimensions stay None; the
        // selector never turns sampled scratch or child-only RSS into a peak.
        let ledger = self.store.snapshot()?;
        let mut history = ledger
            .observations
            .iter()
            .filter_map(|(attempt, record)| {
                let entry = ledger.entries.get(attempt)?;
                (entry.reservation.project == request.project && entry.reservation.account == request.account)
                    .then(|| record.observation.clone())
            })
            .collect::<Vec<_>>();
        Ok(evaluate_resource_policy(
            request.requirements,
            request.features,
            family,
            candidates,
            &mut history,
            request.now_unix_s,
            self.quota_facts(request.project, request.account)?,
            self.config.selection,
            self.controls(request.project),
        )?)
    }

    pub(crate) fn select(&self, request: SelectionRequest<'_>) -> Result<SelectedResources, ResourceHostError> {
        self.project(request.project, request.account)?;
        self.check_window(request.now_unix_s)?;
        if !policy::valid_id(request.request_id)
            || !crunch_remote_core::output::is_blake3_hex_digest(request.request_digest)
        {
            return Err(ResourceHostError::BindingMismatch);
        }
        let family = action_family_identity(request.public_action_kind, request.features)?;
        let evaluation = self.evaluate(&request, &family)?;
        let selection = evaluation.decision.active.as_ref().map_err(|reason| ResourceHostError::Policy(*reason))?;
        let class = evaluation.classes.get(selection.class_index).ok_or(policy::Reason::InvalidInput)?.clone();
        let configured = self
            .config
            .classes
            .iter()
            .find(|candidate| candidate.id == class.id)
            .ok_or(ResourceHostError::InvalidConfiguration)?;
        let mut requirements =
            canonical_remote_resource_requirements(request.requirements).map_err(|_| policy::Reason::InvalidInput)?;
        let declared = policy::Quantities {
            cpu_units: requirements.quantities.cpu_units,
            memory_bytes: requirements.quantities.memory_bytes,
            scratch_bytes: requirements.quantities.scratch_bytes,
        };
        requirements.quantities.cpu_units = class.quantities.cpu_units;
        requirements.quantities.memory_bytes = class.quantities.memory_bytes;
        requirements.quantities.scratch_bytes = class.quantities.scratch_bytes;
        Ok(SelectedResources {
            host_identity: self.host_identity.clone(),
            project: request.project.into(),
            account: request.account.into(),
            request_id: request.request_id.into(),
            request_digest: request.request_digest.into(),
            requirements,
            declared,
            predecessor: None,
            worker_endpoint_id: configured.worker_endpoint_id.clone(),
            machine_class: class,
            action_family: family,
            evaluation,
            retry_binding: None,
        })
    }

    fn check_selected(&self, selected: &SelectedResources, binding: &AttemptBinding) -> Result<(), ResourceHostError> {
        self.project(&binding.project, &binding.account)?;
        if selected.host_identity != self.host_identity
            || selected.project != binding.project
            || selected.account != binding.account
            || selected.request_id != binding.request_id
            || selected.request_digest != binding.request_digest
            || selected.retry_binding.as_ref().is_some_and(|(job, attempt, fence)| {
                job != &binding.job || attempt != &binding.attempt || fence != &binding.fence
            })
        {
            return Err(ResourceHostError::BindingMismatch);
        }
        Ok(())
    }

    /// Admit the exact retained producer through full-key signatures, its
    /// current completed fence, ordinary signed-action admission and current CAS.
    pub(crate) async fn authorize_verified_reuse(
        &self,
        request: VerifiedReuseRequest<'_>,
    ) -> Result<policy::SharingLink, ResourceHostError> {
        let producer = request
            .coordinator
            .gateway_attempts
            .get(request.producer_public_id)
            .ok_or(ResourceHostError::BindingMismatch)?;
        if producer.reuse.is_some()
            || producer.cancel_requested
            || producer.recovery_uncertain
            || producer.local_teardown_confirmed
            || producer.request_digest != request.consumer_request_digest
            || producer.drv_path != request.consumer_drv_path
        {
            return Err(policy::Reason::IdentityDenied.into());
        }
        if producer.authority_policy != request.consumer_policy {
            return Err(policy::Reason::PolicyDenied.into());
        }
        if producer.platform != request.consumer_platform {
            return Err(policy::Reason::PlatformDenied.into());
        }
        let binding = producer.binding.as_ref().ok_or(ResourceHostError::BindingMismatch)?;
        let job = request.coordinator.jobs.get(&binding.job_id).ok_or(ResourceHostError::BindingMismatch)?;
        let current = job.current_attempt.as_ref().ok_or(ResourceHostError::BindingMismatch)?;
        let result = producer.result_evidence.ok_or(policy::Reason::OutputsDenied)?;
        let digest = blake3::Hash::from_bytes(result).to_hex().to_string();
        if current.job_id != binding.job_id
            || current.attempt_id != binding.attempt_id
            || current.fence_generation != binding.fence_generation
            || current.phase != crunch_remote_core::attempt::RemoteAttemptPhase::Completed
            || current.result_digest_blake3.as_deref() != Some(digest.as_str())
            || !job.output_admission_completed
            || !job.result_available
        {
            return Err(policy::Reason::FenceInvalid.into());
        }
        crate::remote_build::validate_retained_remote_produced_outputs(
            &producer.produced_outputs,
            &digest,
            "/nix/store",
            request.trusted_output_keys,
        )
        .map_err(|_| policy::Reason::SignatureDenied)?;
        if producer.outputs.len() != producer.produced_outputs.len()
            || producer.produced_outputs.iter().any(|output| !producer.outputs.contains(&output.logical_path))
        {
            return Err(policy::Reason::OutputsDenied.into());
        }
        // Current signed derivation/action-result authority is mandatory in
        // addition to the exact retained remote producer signatures/receipt.
        if !request
            .store
            .query_missing(&[format!("{}!*", producer.drv_path)])
            .await
            .map_err(|_| policy::Reason::OutputsDenied)?
            .is_empty()
        {
            return Err(policy::Reason::OutputsDenied.into());
        }
        let infos = producer
            .produced_outputs
            .iter()
            .map(|output| output.path_info.as_ref().ok_or(policy::Reason::OutputsDenied))
            .collect::<Result<Vec<_>, _>>()?;
        request.store.verify_retained_path_infos(&infos).await.map_err(|_| policy::Reason::CasUnavailable)?;
        self.authorize_reuse(&producer.project, request.consumer_project, &digest, policy::SharingChecks {
            request_and_action_match: true,
            trusted_producer_signature: true,
            policy_compatible: true,
            platform_compatible: true,
            verified_output_identities: true,
            current_cas_available: true,
        })
    }

    /// Only the concrete verification path above may supply these pure facts;
    /// producer and consumer project controls remain independently enforced.
    fn authorize_reuse(
        &self,
        producer_project: &str,
        consumer_project: &str,
        result_digest: &str,
        checks: policy::SharingChecks,
    ) -> Result<policy::SharingLink, ResourceHostError> {
        if !self.config.projects.contains_key(producer_project)
            || !self.config.projects.contains_key(consumer_project)
            || !crunch_remote_core::output::is_blake3_hex_digest(result_digest)
        {
            return Err(policy::Reason::IdentityDenied.into());
        }
        let controls = self.controls(consumer_project);
        if producer_project == consumer_project {
            return Ok(policy::authorize_sharing(policy::SharingRequest {
                controls,
                producer: producer_project,
                consumer: consumer_project,
                result: result_digest,
                scope: None,
                checks,
            })?);
        }
        // Both projects must explicitly opt in; an operator scope is not a
        // substitute for either project's independent sharing rollback gate.
        if !self.controls(producer_project).sharing_active() {
            return Err(policy::Reason::SharingDenied.into());
        }
        let configured = self
            .config
            .sharing_scopes
            .iter()
            .filter(|scope| scope.producers.contains(producer_project) && scope.consumers.contains(consumer_project))
            .min_by(|left, right| left.name.cmp(&right.name))
            .ok_or(policy::Reason::SharingDenied)?;
        let producers: Vec<&str> = configured.producers.iter().map(String::as_str).collect();
        let consumers: Vec<&str> = configured.consumers.iter().map(String::as_str).collect();
        let scope = policy::SharingScope {
            id: &configured.name,
            producers: &producers,
            consumers: &consumers,
        };
        Ok(policy::authorize_sharing(policy::SharingRequest {
            controls,
            producer: producer_project,
            consumer: consumer_project,
            result: result_digest,
            scope: Some(scope),
            checks,
        })?)
    }

    fn reservation_matches(&self, r: &quota::Reservation, binding: &AttemptBinding) -> bool {
        r.job == binding.job
            && r.attempt == binding.attempt
            && r.fence == binding.fence
            && r.project == binding.project
            && r.account == binding.account
            && r.request_digest == binding.request_digest
            && r.schedule == self.config.schedule
    }

    pub(crate) fn existing_entry(&self, binding: &AttemptBinding) -> Result<quota::Entry, ResourceHostError> {
        self.project(&binding.project, &binding.account)?;
        let ledger = self.store.snapshot()?;
        let entry = ledger.entries.get(binding.attempt.as_str()).ok_or(quota::QuotaError::UnknownAttempt)?;
        if !self.reservation_matches(&entry.reservation, binding) {
            return Err(ResourceHostError::BindingMismatch);
        }
        Ok(entry.clone())
    }

    pub(crate) fn observation(
        &self,
        binding: &AttemptBinding,
    ) -> Result<Option<policy::Observation>, ResourceHostError> {
        self.existing_entry(binding)?;
        Ok(self
            .store
            .snapshot()?
            .observations
            .get(binding.attempt.as_str())
            .map(|record| record.observation.clone()))
    }

    pub(crate) fn reserve(
        &self,
        selected: &SelectedResources,
        binding: &AttemptBinding,
        now_unix_s: u64,
    ) -> Result<quota::Entry, ResourceHostError> {
        self.check_selected(selected, binding)?;
        let limits = self.project(&binding.project, &binding.account)?;
        let reservation = quota::Reservation {
            job: binding.job.clone(),
            attempt: binding.attempt.clone(),
            fence: binding.fence,
            project: binding.project.clone(),
            account: binding.account.clone(),
            request_digest: binding.request_digest.clone(),
            units: selected.machine_class.reservation_units,
            schedule: self.config.schedule.clone(),
        };
        // Identical retries remain valid after the window closes. Never acknowledge
        // a different attempt reservation or silently provision on a failed read.
        let ledger = self.store.snapshot()?;
        if let Some(old) = ledger.entries.get(binding.attempt.as_str()) {
            if old.reservation != reservation {
                return Err(quota::QuotaError::Conflict.into());
            }
        } else {
            self.check_window(now_unix_s)?;
        }
        let limits = if self.controls(&binding.project).quota_active() {
            quota::Limits {
                project_units: limits.project_units,
                account_units: limits.account_units,
            }
        } else {
            quota::Limits {
                project_units: u64::MAX,
                account_units: u64::MAX,
            }
        };
        let context = quota::ResourceContext {
            declared: selected.declared,
            selected: selected.machine_class.quantities,
            features: selected.machine_class.features.clone(),
            machine_class: selected.machine_class.id.clone(),
            action_family: selected.action_family.clone(),
            policy_version: self.config.selection.version,
            predecessor: selected.predecessor.clone(),
        };
        Ok(self.store.reserve_with_resource_context(reservation, limits, context)?)
    }

    fn check_usage(&self, binding: &AttemptBinding, usage: &RemoteAuthenticatedUsage) -> Result<(), ResourceHostError> {
        let measured = usage.binding();
        if measured.job_id != binding.job
            || measured.attempt_id != binding.attempt
            || measured.fence_generation != binding.fence
            || usage.request_id() != binding.request_id
        {
            return Err(ResourceHostError::BindingMismatch);
        }
        Ok(())
    }

    pub(crate) fn mark_pending(
        &self,
        binding: &AttemptBinding,
        reason: PendingUsage,
        now_unix_s: u64,
    ) -> Result<quota::Entry, ResourceHostError> {
        let entry = self.existing_entry(binding)?;
        // Losing telemetry after an already committed completion cannot undo it.
        if matches!(entry.reconciliation, quota::Reconciliation::Settled { .. }) {
            return Ok(entry);
        }
        self.record_missing_observation(binding, &entry.reservation, reason, now_unix_s)?;
        let event = match reason {
            PendingUsage::Missing | PendingUsage::Partial => quota::UsageEvent::Partial,
            PendingUsage::WorkerLost => quota::UsageEvent::WorkerLost,
            PendingUsage::Cancelled => quota::UsageEvent::Cancelled,
        };
        Ok(self.store.reconcile(&entry.reservation, event)?)
    }

    fn record_missing_observation(
        &self,
        binding: &AttemptBinding,
        reservation: &quota::Reservation,
        reason: PendingUsage,
        terminal_unix_s: u64,
    ) -> Result<(), ResourceHostError> {
        let ledger = self.store.snapshot()?;
        if ledger.observations.contains_key(binding.attempt.as_str()) {
            return Ok(());
        }
        let context =
            ledger.resource_contexts.get(binding.attempt.as_str()).ok_or(ResourceHostError::BindingMismatch)?;
        let (outcome, reason_code) = match reason {
            PendingUsage::WorkerLost => (policy::Outcome::WorkerLost, "worker-lost"),
            PendingUsage::Cancelled => (policy::Outcome::Cancelled, "cancelled-without-final-usage"),
            PendingUsage::Partial | PendingUsage::Missing => (policy::Outcome::Unknown, "usage-incomplete"),
        };
        let observation = policy::Observation {
            schema: policy::OBSERVATION_SCHEMA.into(),
            attempt_id: binding.attempt.clone(),
            action_family: context.action_family.clone(),
            features: context.features.clone(),
            policy_version: context.policy_version,
            machine_class: context.machine_class.clone(),
            declared: context.declared,
            selected: context.selected,
            measured: policy::Measurements {
                cpu_units: None,
                memory_bytes: None,
                scratch_bytes: None,
            },
            scratch_sampled_peak_bytes: None,
            queue_ms: None,
            execution_ms: None,
            terminal_unix_s,
            cpu_ms: None,
            io_bytes: None,
            transfer_bytes: None,
            executor_output_bytes: None,
            process_peak_rss_bytes: None,
            oom: policy::OomCategory::Unknown,
            outcome,
            predecessor: context.predecessor.clone(),
            collector_version: "mantle-coordinator-no-worker-measurements-v2".into(),
            trusted_collector: false,
        };
        let bytes = serde_json::to_vec(&("mantle-coordinator-missing-usage-v2", reservation, reason_code))
            .map_err(|_| ResourceHostError::BindingMismatch)?;
        let usage_event_digest = blake3::hash(&bytes).to_hex().to_string();
        Ok(self.store.record_observation(reservation, quota::ObservationRecord {
            observation,
            usage_event_digest,
        })?)
    }

    fn record_usage_observation(
        &self,
        binding: &AttemptBinding,
        usage: &RemoteAuthenticatedUsage,
        reservation: &quota::Reservation,
        terminal_unix_s: u64,
    ) -> Result<(), ResourceHostError> {
        let ledger = self.store.snapshot()?;
        let context =
            ledger.resource_contexts.get(binding.attempt.as_str()).ok_or(ResourceHostError::BindingMismatch)?;
        let usage_event_digest = usage.event_digest_blake3();
        if let Some(existing) = ledger.observations.get(binding.attempt.as_str()) {
            if existing.usage_event_digest == usage_event_digest {
                return Ok(());
            }
            if existing.observation.trusted_collector {
                return Err(quota::QuotaError::Conflict.into());
            }
        }
        let dedicated = usage.measurement_completeness() == RemoteUsageCompleteness::DedicatedCgroup;
        let oom = match usage.oom_evidence() {
            RemoteUsageOomEvidence::Unavailable => policy::OomCategory::Unknown,
            RemoteUsageOomEvidence::DedicatedCgroupNoOom => policy::OomCategory::None,
            RemoteUsageOomEvidence::DedicatedCgroupOom { .. } => policy::OomCategory::LinuxCgroupV2Kill,
        };
        let production_binding = crate::remote_build::RemoteProductionAttemptBinding {
            job_id: binding.job.clone(),
            attempt_id: binding.attempt.clone(),
            fence_generation: binding.fence,
        };
        let failed = usage.result_digest_blake3()
            == crate::remote_build::usage::failure_result_digest(&production_binding, &binding.request_id);
        let observation = policy::Observation {
            schema: policy::OBSERVATION_SCHEMA.into(),
            attempt_id: binding.attempt.clone(),
            action_family: context.action_family.clone(),
            features: context.features.clone(),
            policy_version: context.policy_version,
            machine_class: context.machine_class.clone(),
            declared: context.declared,
            selected: context.selected,
            measured: policy::Measurements {
                cpu_units: None,
                memory_bytes: if dedicated {
                    usage.cgroup_peak_memory_bytes()
                } else {
                    None
                },
                scratch_bytes: None,
            },
            scratch_sampled_peak_bytes: usage.scratch_observation().sampled_peak_bytes,
            queue_ms: None,
            execution_ms: Some(usage.wall_time_millis()),
            terminal_unix_s,
            cpu_ms: Some(usage.cpu_time_millis()),
            io_bytes: None,
            transfer_bytes: None,
            executor_output_bytes: Some(usage.output_bytes()),
            process_peak_rss_bytes: Some(usage.peak_rss_bytes()),
            oom,
            outcome: if failed {
                policy::Outcome::Failed
            } else {
                policy::Outcome::Success
            },
            predecessor: context.predecessor.clone(),
            collector_version: if dedicated {
                "mantle-worker-dedicated-cgroup-v2"
            } else {
                "mantle-worker-process-only-v2"
            }
            .into(),
            // Opaque RemoteAuthenticatedUsage, not a caller-provided flag.
            trusted_collector: true,
        };
        Ok(self.store.record_observation(reservation, quota::ObservationRecord {
            observation,
            usage_event_digest,
        })?)
    }

    pub(crate) fn reconcile(
        &self,
        binding: &AttemptBinding,
        usage: &RemoteAuthenticatedUsage,
        now_unix_s: u64,
    ) -> Result<quota::Entry, ResourceHostError> {
        self.check_usage(binding, usage)?;
        let entry = self.existing_entry(binding)?;
        self.record_usage_observation(binding, usage, &entry.reservation, now_unix_s)?;
        let max = self.config.selection;
        if usage.measurement_completeness() != RemoteUsageCompleteness::DedicatedCgroup {
            return self.mark_pending(binding, PendingUsage::Partial, now_unix_s);
        }
        let Some(memory_peak_bytes) = usage.cgroup_peak_memory_bytes() else {
            return self.mark_pending(binding, PendingUsage::Partial, now_unix_s);
        };
        let memory_byte_ms =
            memory_peak_bytes.checked_mul(usage.wall_time_millis()).ok_or(quota::QuotaError::Overflow)?;
        if usage.wall_time_millis() == 0
            || memory_peak_bytes == 0
            || usage.wall_time_millis() > max.max_duration_ms
            || memory_peak_bytes > max.max_measurement.memory_bytes
            || usage.output_bytes() > max.max_io_bytes
            || usage.cpu_time_millis() > max.max_duration_ms.saturating_mul(u64::from(max.max_measurement.cpu_units))
        {
            return self.mark_pending(binding, PendingUsage::Partial, now_unix_s);
        }
        Ok(self.store.reconcile(&entry.reservation, quota::UsageEvent::Complete {
            event_digest: usage.event_digest_blake3(),
            usage: quota::Usage {
                cpu_ms: usage.cpu_time_millis(),
                memory_byte_ms,
                executor_output_bytes: usage.output_bytes(),
            },
            // The opaque type plus DedicatedCgroup completeness supplies measured
            // execution-tree facts; no caller boolean crosses this boundary.
            trusted_and_complete: true,
        })?)
    }

    /// Plans only: the coordinator must durably create the linked new fenced
    /// attempt, reserve it, acquire its lease and dispatch through normal admission.
    #[allow(
        clippy::too_many_arguments,
        reason = "Explicit coordinator fence, live leases and authenticated worker evidence"
    )]
    pub(crate) fn plan_oom_retry(
        &self,
        prior: &SelectedResources,
        binding: &AttemptBinding,
        usage: &RemoteAuthenticatedUsage,
        context: policy::RetryContext<'_>,
        now_unix_s: u64,
        live_leases: &[RemoteResourceLease],
    ) -> Result<RetryResources, ResourceHostError> {
        self.check_selected(prior, binding)?;
        self.check_usage(binding, usage)?;
        self.existing_entry(binding)?;
        self.check_window(now_unix_s)?;
        if context.job != &binding.job || context.attempt != &binding.attempt || context.fence != binding.fence {
            return Err(policy::Reason::FenceInvalid.into());
        }
        let (category, trusted_positive_platform_evidence) = match usage.oom_evidence() {
            // Only the collector's fresh dedicated per-attempt cgroup may prove
            // positive OOM. Exit 137 and aggregate host counters never do.
            RemoteUsageOomEvidence::Unavailable => (policy::OomCategory::Unknown, false),
            RemoteUsageOomEvidence::DedicatedCgroupNoOom => (policy::OomCategory::None, false),
            RemoteUsageOomEvidence::DedicatedCgroupOom { oom_kill_delta } => (
                policy::OomCategory::LinuxCgroupV2Kill,
                oom_kill_delta > 0 && usage.measurement_completeness() == RemoteUsageCompleteness::DedicatedCgroup,
            ),
        };
        let request = SelectionRequest {
            project: &binding.project,
            account: &binding.account,
            request_id: &binding.request_id,
            request_digest: &binding.request_digest,
            requirements: &prior.requirements,
            features: &prior.machine_class.features,
            public_action_kind: "resource-retry",
            now_unix_s,
            live_leases,
        };
        let evaluation = self.evaluate(&request, &prior.action_family)?;
        let controls = self.controls(&binding.project);
        let quota = if controls.quota_active() {
            self.quota_facts(&binding.project, &binding.account)?
        } else {
            policy::QuotaFacts {
                project_remaining_units: u64::MAX,
                account_remaining_units: u64::MAX,
            }
        };
        let evidence = policy::OomEvidence {
            job: &binding.job,
            attempt: &binding.attempt,
            fence: binding.fence,
            platform: &prior.machine_class.features.platform,
            category,
            trusted_positive_platform_evidence,
        };
        let plan = policy::plan_escalation(policy::EscalationRequest {
            controls,
            context: context.clone(),
            evidence: evidence.clone(),
            prior: &prior.machine_class,
            classes: &evaluation.classes,
            quota,
            policy: self.config.retry.into(),
        })?;
        // These are the exact pure planner inputs, not a token granting replayed
        // evidence authority. The live opaque report remains independently bound.
        let replay_json = serde_json::json!({
            "schema": "mantle-resource-escalation-replay-v1",
            "controls": controls,
            "context": {
                "job": context.job,
                "attempt": context.attempt,
                "fence": context.fence,
                "next_attempt": context.next_attempt,
                "next_fence": context.next_fence,
                "failed": context.failed,
                "retries": context.retries,
                "cumulative_units": context.cumulative_units,
                "elapsed_ms": context.elapsed_ms,
                "next_wall_budget_ms": context.next_wall_budget_ms,
            },
            "evidence": {
                "job": evidence.job,
                "attempt": evidence.attempt,
                "fence": evidence.fence,
                "platform": evidence.platform,
                "category": evidence.category,
                "trusted_positive_platform_evidence": evidence.trusted_positive_platform_evidence,
            },
            "prior": &prior.machine_class,
            "classes": &evaluation.classes,
            "quota": {
                "project_remaining_units": quota.project_remaining_units,
                "account_remaining_units": quota.account_remaining_units,
            },
            "policy": self.config.retry,
            "plan": &plan,
        });
        let class = evaluation.classes.get(plan.class_index).ok_or(policy::Reason::InvalidInput)?.clone();
        let configured = self
            .config
            .classes
            .iter()
            .find(|candidate| candidate.id == class.id)
            .ok_or(ResourceHostError::InvalidConfiguration)?;
        let mut selected = prior.clone();
        selected.requirements.quantities.cpu_units = class.quantities.cpu_units;
        selected.requirements.quantities.memory_bytes = class.quantities.memory_bytes;
        selected.requirements.quantities.scratch_bytes = class.quantities.scratch_bytes;
        selected.machine_class = class;
        selected.worker_endpoint_id = configured.worker_endpoint_id.clone();
        selected.retry_binding = Some((binding.job.clone(), plan.next_attempt.clone(), plan.next_fence));
        selected.predecessor = Some(binding.attempt.clone());
        selected.evaluation = evaluation;
        Ok(RetryResources {
            plan,
            selected,
            replay_json,
        })
    }
}

fn count_reconciliation(
    counts: &mut UsageReasonCounts,
    state: &quota::Reconciliation,
) -> Result<(), quota::QuotaError> {
    use quota::PendingReason as P;
    use quota::Reconciliation as R;
    let count = match state {
        R::Reserved => &mut counts.reserved,
        R::Settled { .. } => &mut counts.settled,
        R::Pending(P::CancelledWithoutFinalUsage) => &mut counts.cancelled_without_final_usage,
        R::Pending(P::WorkerLost) => &mut counts.worker_lost,
        R::Pending(P::PartialObservation) => &mut counts.partial_observation,
        R::Pending(P::UntrustedObservation) => &mut counts.untrusted_observation,
        R::Pending(P::UsageExceedsReservation) => &mut counts.usage_exceeds_reservation,
    };
    *count = count.checked_add(1).ok_or(quota::QuotaError::Overflow)?;
    Ok(())
}

#[cfg(test)]
#[path = "resource_host_tests.rs"]
mod tests;
