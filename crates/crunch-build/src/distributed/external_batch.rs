//! Serde protocol adapter for external batch dispatchers.
//!
//! The no_std core derives identities and makes bounded, fenced decisions.
//! This adapter keeps the accepted v1 JSON resource projection and wire types;
//! the coordinator shell owns subprocesses, clocks, persistence, worker
//! registration, and output admission.
//!
//! r[impl external_batch_dispatchers.protocol]
//! r[impl external_batch_dispatchers.canonical_identity]
//! r[impl external_batch_dispatchers.resources]
//! r[impl external_batch_dispatchers.fenced_lifecycle]

use crunch_remote_core::external_batch as core_batch;
use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use serde::Deserialize;
use serde::Serialize;

use super::RemoteNamedResourceQuantity;
use super::RemoteResourceRequirements;
use super::canonical_remote_resource_requirements;

pub const EXTERNAL_BATCH_PROTOCOL_SCHEMA: &str = core_batch::PROTOCOL_SCHEMA;
pub const EXTERNAL_BATCH_DISPATCH_DOMAIN: &str = core_batch::DISPATCH_DOMAIN;
pub const EXTERNAL_BATCH_OPERATION_DOMAIN: &str = core_batch::OPERATION_DOMAIN;
pub const EXTERNAL_BATCH_CLAIM_SCOPE: &str = core_batch::CLAIM_SCOPE;
pub const EXTERNAL_BATCH_NON_CLAIM: &str = core_batch::NON_CLAIM;
pub const MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES: usize = 256;
pub const MAX_EXTERNAL_BATCH_REASON_BYTES: usize = 128;
pub const MAX_EXTERNAL_BATCH_TIMEOUT_SECS: u64 = 604_800;
pub const MAX_EXTERNAL_BATCH_RECONCILE_ATTEMPTS: u32 = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalBatchOperationKind {
    Submit,
    Observe,
    Cancel,
    Reconcile,
}

impl ExternalBatchOperationKind {
    pub const fn as_str(self) -> &'static str {
        core_batch_kind(self).as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalBatchJobState {
    Submitted,
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}

impl ExternalBatchJobState {
    pub const fn is_terminal(self) -> bool {
        core_batch_state(self).is_terminal()
    }
}

const fn core_batch_kind(kind: ExternalBatchOperationKind) -> core_batch::BatchKind {
    match kind {
        ExternalBatchOperationKind::Submit => core_batch::BatchKind::Submit,
        ExternalBatchOperationKind::Observe => core_batch::BatchKind::Observe,
        ExternalBatchOperationKind::Cancel => core_batch::BatchKind::Cancel,
        ExternalBatchOperationKind::Reconcile => core_batch::BatchKind::Reconcile,
    }
}

const fn core_batch_state(state: ExternalBatchJobState) -> core_batch::BatchState {
    match state {
        ExternalBatchJobState::Submitted => core_batch::BatchState::Submitted,
        ExternalBatchJobState::Pending => core_batch::BatchState::Pending,
        ExternalBatchJobState::Running => core_batch::BatchState::Running,
        ExternalBatchJobState::Succeeded => core_batch::BatchState::Succeeded,
        ExternalBatchJobState::Failed => core_batch::BatchState::Failed,
        ExternalBatchJobState::Cancelled => core_batch::BatchState::Cancelled,
        ExternalBatchJobState::Unknown => core_batch::BatchState::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalBatchResourceProjection {
    pub cpu_units: u32,
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
    pub accelerators: Vec<RemoteNamedResourceQuantity>,
    pub named_tokens: Vec<RemoteNamedResourceQuantity>,
    pub semantic_accelerator_classes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalBatchLimits {
    pub startup_timeout_secs: u64,
    pub terminal_timeout_secs: u64,
    pub max_reconcile_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalBatchOperationInput {
    pub kind: ExternalBatchOperationKind,
    pub adapter_instance_id: String,
    pub dispatcher_profile_ref_blake3: String,
    pub provider_class: String,
    pub worker_bootstrap_ref_blake3: String,
    pub semantic_capability_class: String,
    pub dispatcher_generation: u64,
    pub normalized_build_key: String,
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub expected_worker_endpoint_id: String,
    pub resource_requirements: RemoteResourceRequirements,
    pub limits: ExternalBatchLimits,
    pub external_job_id: Option<String>,
}

fn absent_external_job_id() -> Option<String> {
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalBatchOperation {
    pub schema: String,
    pub operation: ExternalBatchOperationKind,
    pub operation_id_blake3: String,
    pub dispatch_id_blake3: String,
    pub adapter_instance_id: String,
    pub dispatcher_profile_ref_blake3: String,
    pub provider_class: String,
    pub worker_bootstrap_ref_blake3: String,
    pub semantic_capability_class: String,
    pub dispatcher_generation: u64,
    pub normalized_build_key: String,
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub expected_worker_endpoint_id: String,
    pub resources: ExternalBatchResourceProjection,
    pub limits: ExternalBatchLimits,
    #[serde(default = "absent_external_job_id", skip_serializing_if = "Option::is_none")]
    pub external_job_id: Option<String>,
    pub claim_scope: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalBatchOperationResponse {
    pub schema: String,
    pub operation: ExternalBatchOperationKind,
    pub operation_id_blake3: String,
    pub dispatch_id_blake3: String,
    pub adapter_instance_id: String,
    pub dispatcher_generation: u64,
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub state: ExternalBatchJobState,
    #[serde(default = "absent_external_job_id", skip_serializing_if = "Option::is_none")]
    pub external_job_id: Option<String>,
    pub reason_code: String,
    pub observed_unix_s: u64,
    pub non_claim: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalBatchReconcileFacts {
    pub worker_registered: bool,
    pub reconcile_attempt: u32,
    pub overall_deadline_exceeded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalBatchReconcileDecision {
    Reattach,
    AwaitWorkerRegistration,
    PreserveUnknown,
    Resubmit,
    MarkLost,
}

const fn core_batch_limits(limits: ExternalBatchLimits) -> core_batch::BatchLimits {
    core_batch::BatchLimits {
        startup_timeout_secs: limits.startup_timeout_secs,
        terminal_timeout_secs: limits.terminal_timeout_secs,
        max_reconcile_attempts: limits.max_reconcile_attempts,
    }
}

fn core_operation_facts(operation: &ExternalBatchOperation) -> core_batch::BatchOperationFacts<'_> {
    core_batch::BatchOperationFacts {
        schema: &operation.schema,
        kind: core_batch_kind(operation.operation),
        operation_id_blake3: &operation.operation_id_blake3,
        dispatch_id_blake3: &operation.dispatch_id_blake3,
        normalized_build_key: &operation.normalized_build_key,
        dispatcher_profile_ref_blake3: &operation.dispatcher_profile_ref_blake3,
        worker_bootstrap_ref_blake3: &operation.worker_bootstrap_ref_blake3,
        adapter_instance_id: &operation.adapter_instance_id,
        dispatcher_generation: operation.dispatcher_generation,
        job_id: operation.job_id.as_str(),
        attempt_id: operation.attempt_id.as_str(),
        fence_generation: operation.fence_generation.get(),
        external_job_id: operation.external_job_id.as_deref(),
        claim_scope: &operation.claim_scope,
        non_claim: &operation.non_claim,
        limits: core_batch_limits(operation.limits),
    }
}

fn core_response_facts(response: &ExternalBatchOperationResponse) -> core_batch::BatchResponseFacts<'_> {
    core_batch::BatchResponseFacts {
        schema: &response.schema,
        kind: core_batch_kind(response.operation),
        operation_id_blake3: &response.operation_id_blake3,
        dispatch_id_blake3: &response.dispatch_id_blake3,
        adapter_instance_id: &response.adapter_instance_id,
        dispatcher_generation: response.dispatcher_generation,
        job_id: response.job_id.as_str(),
        attempt_id: response.attempt_id.as_str(),
        fence_generation: response.fence_generation.get(),
        state: core_batch_state(response.state),
        external_job_id: response.external_job_id.as_deref(),
        reason_code: &response.reason_code,
        non_claim: &response.non_claim,
    }
}

fn host_reconcile_decision(decision: core_batch::BatchDecision) -> ExternalBatchReconcileDecision {
    match decision {
        core_batch::BatchDecision::Reattach => ExternalBatchReconcileDecision::Reattach,
        core_batch::BatchDecision::AwaitWorkerRegistration => ExternalBatchReconcileDecision::AwaitWorkerRegistration,
        core_batch::BatchDecision::PreserveUnknown => ExternalBatchReconcileDecision::PreserveUnknown,
        core_batch::BatchDecision::Resubmit => ExternalBatchReconcileDecision::Resubmit,
        core_batch::BatchDecision::MarkLost => ExternalBatchReconcileDecision::MarkLost,
    }
}

pub fn project_external_batch_resources(
    requirements: &RemoteResourceRequirements,
) -> Result<ExternalBatchResourceProjection, String> {
    let canonical =
        canonical_remote_resource_requirements(requirements).map_err(|reason| reason.as_str().to_string())?;
    let projection = ExternalBatchResourceProjection {
        cpu_units: canonical.quantities.cpu_units,
        memory_bytes: canonical.quantities.memory_bytes,
        scratch_bytes: canonical.quantities.scratch_bytes,
        accelerators: canonical.quantities.accelerators,
        named_tokens: canonical.quantities.named_tokens,
        semantic_accelerator_classes: canonical.semantic_accelerator_classes,
    };
    assert!(projection.cpu_units > 0);
    assert!(projection.memory_bytes > 0);
    Ok(projection)
}

pub fn plan_external_batch_operation(input: ExternalBatchOperationInput) -> Result<ExternalBatchOperation, String> {
    validate_operation_input(&input)?;
    let resources = project_external_batch_resources(&input.resource_requirements)?;
    let dispatch_id_blake3 = derive_dispatch_id(&input, &resources)?;
    let operation_id_blake3 = derive_operation_id(input.kind, &dispatch_id_blake3)?;
    let operation = ExternalBatchOperation {
        schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
        operation: input.kind,
        operation_id_blake3,
        dispatch_id_blake3,
        adapter_instance_id: input.adapter_instance_id,
        dispatcher_profile_ref_blake3: input.dispatcher_profile_ref_blake3,
        provider_class: input.provider_class,
        worker_bootstrap_ref_blake3: input.worker_bootstrap_ref_blake3,
        semantic_capability_class: input.semantic_capability_class,
        dispatcher_generation: input.dispatcher_generation,
        normalized_build_key: input.normalized_build_key,
        job_id: input.job_id,
        attempt_id: input.attempt_id,
        fence_generation: input.fence_generation,
        expected_worker_endpoint_id: input.expected_worker_endpoint_id,
        resources,
        limits: input.limits,
        external_job_id: input.external_job_id,
        claim_scope: EXTERNAL_BATCH_CLAIM_SCOPE.to_string(),
        non_claim: EXTERNAL_BATCH_NON_CLAIM.to_string(),
    };
    assert!(crunch_remote_core::output::is_blake3_hex_digest(&operation.operation_id_blake3));
    assert!(crunch_remote_core::output::is_blake3_hex_digest(&operation.dispatch_id_blake3));
    Ok(operation)
}

pub fn plan_external_batch_followup(
    submit: &ExternalBatchOperation,
    kind: ExternalBatchOperationKind,
    external_job_id: Option<String>,
) -> Result<ExternalBatchOperation, String> {
    validate_operation(submit)?;
    core_batch::validate_followup(
        core_operation_facts(submit),
        core_batch_kind(kind),
        external_job_id.as_deref(),
        MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
    )
    .map_err(core_batch::BatchError::diagnostic)?;
    let mut followup = submit.clone();
    followup.operation = kind;
    followup.operation_id_blake3 = derive_operation_id(kind, &submit.dispatch_id_blake3)?;
    followup.external_job_id = external_job_id;
    assert_eq!(followup.dispatch_id_blake3, submit.dispatch_id_blake3);
    assert_ne!(followup.operation_id_blake3, submit.operation_id_blake3);
    Ok(followup)
}

pub fn validate_external_batch_response(
    request: &ExternalBatchOperation,
    response: &ExternalBatchOperationResponse,
) -> Result<(), String> {
    validate_operation(request)?;
    core_batch::validate_response(
        core_operation_facts(request),
        core_response_facts(response),
        MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
        MAX_EXTERNAL_BATCH_REASON_BYTES,
    )
    .map_err(core_batch::BatchError::diagnostic)?;
    assert!(response.observed_unix_s > 0);
    assert_eq!(response.dispatch_id_blake3, request.dispatch_id_blake3);
    Ok(())
}

pub fn validate_external_batch_state_transition(
    previous: ExternalBatchJobState,
    next: ExternalBatchJobState,
) -> Result<(), String> {
    core_batch::validate_state_transition(core_batch_state(previous), core_batch_state(next))
        .map_err(core_batch::BatchError::diagnostic)
}

pub fn decide_external_batch_reconciliation(
    request: &ExternalBatchOperation,
    response: &ExternalBatchOperationResponse,
    facts: ExternalBatchReconcileFacts,
) -> Result<ExternalBatchReconcileDecision, String> {
    validate_external_batch_response(request, response)?;
    core_batch::decide_reconciliation(
        core_batch_state(response.state),
        facts.worker_registered,
        facts.reconcile_attempt,
        request.limits.max_reconcile_attempts,
        facts.overall_deadline_exceeded,
    )
    .map(host_reconcile_decision)
    .map_err(core_batch::BatchError::diagnostic)
}

fn validate_operation_input(input: &ExternalBatchOperationInput) -> Result<(), String> {
    core_batch::validate_input(
        core_batch::BatchInputFacts {
            kind: core_batch_kind(input.kind),
            adapter_instance_id: &input.adapter_instance_id,
            dispatcher_profile_ref_blake3: &input.dispatcher_profile_ref_blake3,
            provider_class: &input.provider_class,
            worker_bootstrap_ref_blake3: &input.worker_bootstrap_ref_blake3,
            semantic_capability_class: &input.semantic_capability_class,
            dispatcher_generation: input.dispatcher_generation,
            normalized_build_key: &input.normalized_build_key,
            expected_worker_endpoint_id: &input.expected_worker_endpoint_id,
            limits: core_batch_limits(input.limits),
            external_job_id: input.external_job_id.as_deref(),
        },
        MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
        MAX_EXTERNAL_BATCH_TIMEOUT_SECS,
        MAX_EXTERNAL_BATCH_RECONCILE_ATTEMPTS,
    )
    .map_err(core_batch::BatchError::diagnostic)
}

fn validate_operation(operation: &ExternalBatchOperation) -> Result<(), String> {
    core_batch::validate_operation(
        core_operation_facts(operation),
        MAX_EXTERNAL_BATCH_TIMEOUT_SECS,
        MAX_EXTERNAL_BATCH_RECONCILE_ATTEMPTS,
    )
    .map_err(core_batch::BatchError::diagnostic)
}

fn derive_dispatch_id(
    input: &ExternalBatchOperationInput,
    resources: &ExternalBatchResourceProjection,
) -> Result<String, String> {
    let resource_json = serde_json::to_vec(resources)
        .map_err(|error| format!("external-batch-resource-projection-serialize-failed: {error}"))?;
    core_batch::derive_dispatch_id(core_batch::BatchDispatchFacts {
        adapter_instance_id: &input.adapter_instance_id,
        dispatcher_profile_ref_blake3: &input.dispatcher_profile_ref_blake3,
        provider_class: &input.provider_class,
        worker_bootstrap_ref_blake3: &input.worker_bootstrap_ref_blake3,
        semantic_capability_class: &input.semantic_capability_class,
        dispatcher_generation: input.dispatcher_generation,
        normalized_build_key: &input.normalized_build_key,
        job_id: input.job_id.as_str(),
        attempt_id: input.attempt_id.as_str(),
        fence_generation: input.fence_generation.get(),
        expected_worker_endpoint_id: &input.expected_worker_endpoint_id,
        resource_projection_json: &resource_json,
    })
    .map_err(core_batch::BatchError::diagnostic)
}

fn derive_operation_id(kind: ExternalBatchOperationKind, dispatch_id_blake3: &str) -> Result<String, String> {
    core_batch::derive_operation_id(core_batch_kind(kind), dispatch_id_blake3).map_err(core_batch::BatchError::diagnostic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::RemoteResourceVector;

    const TEST_MEMORY_BYTES: u64 = 8_589_934_592;
    const TEST_SCRATCH_BYTES: u64 = 17_179_869_184;
    const TEST_TIMEOUT_SECS: u64 = 300;
    const TEST_RECONCILE_ATTEMPTS: u32 = 3;

    fn fixture_requirements() -> RemoteResourceRequirements {
        RemoteResourceRequirements {
            quantities: RemoteResourceVector {
                cpu_units: 4,
                memory_bytes: TEST_MEMORY_BYTES,
                scratch_bytes: TEST_SCRATCH_BYTES,
                accelerators: vec![RemoteNamedResourceQuantity {
                    name: "nvidia-sm90".to_string(),
                    quantity: 1,
                }],
                named_tokens: vec![RemoteNamedResourceQuantity {
                    name: "linker-seat".to_string(),
                    quantity: 1,
                }],
            },
            semantic_accelerator_classes: vec!["nvidia-sm90".to_string()],
        }
    }

    fn fixture_input(kind: ExternalBatchOperationKind, external_job_id: Option<&str>) -> ExternalBatchOperationInput {
        ExternalBatchOperationInput {
            kind,
            adapter_instance_id: "slurm-fixture".to_string(),
            dispatcher_profile_ref_blake3: blake3::hash(b"profile").to_hex().to_string(),
            provider_class: "slurm".to_string(),
            worker_bootstrap_ref_blake3: blake3::hash(b"bootstrap").to_hex().to_string(),
            semantic_capability_class: "x86_64-linux:kvm".to_string(),
            dispatcher_generation: 7,
            normalized_build_key: blake3::hash(b"build").to_hex().to_string(),
            job_id: RemoteJobId::new("job-1").expect("fixture job id"),
            attempt_id: RemoteAttemptId::new("attempt-1").expect("fixture attempt id"),
            fence_generation: RemoteFenceGeneration::INITIAL,
            expected_worker_endpoint_id: "batch-worker-1".to_string(),
            resource_requirements: fixture_requirements(),
            limits: ExternalBatchLimits {
                startup_timeout_secs: TEST_TIMEOUT_SECS,
                terminal_timeout_secs: TEST_TIMEOUT_SECS,
                max_reconcile_attempts: TEST_RECONCILE_ATTEMPTS,
            },
            external_job_id: external_job_id.map(str::to_string),
        }
    }

    fn fixture_response(
        request: &ExternalBatchOperation,
        state: ExternalBatchJobState,
    ) -> ExternalBatchOperationResponse {
        ExternalBatchOperationResponse {
            schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
            operation: request.operation,
            operation_id_blake3: request.operation_id_blake3.clone(),
            dispatch_id_blake3: request.dispatch_id_blake3.clone(),
            adapter_instance_id: request.adapter_instance_id.clone(),
            dispatcher_generation: request.dispatcher_generation,
            job_id: request.job_id.clone(),
            attempt_id: request.attempt_id.clone(),
            fence_generation: request.fence_generation,
            state,
            external_job_id: Some("slurm-42".to_string()),
            reason_code: "scheduler-state-observed".to_string(),
            observed_unix_s: 1,
            non_claim: EXTERNAL_BATCH_NON_CLAIM.to_string(),
        }
    }

    #[test]
    fn canonical_submit_is_deterministic_and_secret_free() {
        let first = plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Submit, None))
            .expect("valid submit plans");
        let second = plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Submit, None))
            .expect("same submit plans");
        let encoded = serde_json::to_string(&first).expect("operation serializes");

        assert_eq!(first, second);
        assert_eq!(first.schema, EXTERNAL_BATCH_PROTOCOL_SCHEMA);
        assert!(encoded.contains("nvidia-sm90"));
        assert!(!encoded.contains("password"));
        assert!(!encoded.contains("credential"));
    }

    #[test]
    fn equivalent_resource_order_produces_identical_dispatch_identity() {
        let mut first = fixture_input(ExternalBatchOperationKind::Submit, None);
        first.resource_requirements.quantities.accelerators.push(RemoteNamedResourceQuantity {
            name: "amd-gfx942".to_string(),
            quantity: 1,
        });
        first.resource_requirements.semantic_accelerator_classes.push("amd-gfx942".to_string());
        let mut second = first.clone();
        second.resource_requirements.quantities.accelerators.reverse();
        second.resource_requirements.semantic_accelerator_classes.reverse();
        let first = plan_external_batch_operation(first).expect("first resource order plans");
        let second = plan_external_batch_operation(second).expect("second resource order plans");

        assert_eq!(first.dispatch_id_blake3, second.dispatch_id_blake3);
        assert_eq!(first.resources, second.resources);
    }

    #[test]
    fn operation_kind_changes_operation_identity_not_dispatch_identity() {
        let submit = plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Submit, None))
            .expect("submit plans");
        let observe =
            plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Observe, Some("slurm-42")))
                .expect("observe plans");

        assert_eq!(submit.dispatch_id_blake3, observe.dispatch_id_blake3);
        assert_ne!(submit.operation_id_blake3, observe.operation_id_blake3);
    }

    #[test]
    fn followup_preserves_dispatch_identity_and_requires_scheduler_identity_except_reconcile() {
        let submit = plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Submit, None))
            .expect("submit plans");
        let observe =
            plan_external_batch_followup(&submit, ExternalBatchOperationKind::Observe, Some("slurm-42".to_string()))
                .expect("followup plans");
        let reconcile = plan_external_batch_followup(&submit, ExternalBatchOperationKind::Reconcile, None)
            .expect("reconcile by stable dispatch identity plans");
        let missing = plan_external_batch_followup(&submit, ExternalBatchOperationKind::Cancel, None)
            .expect_err("cancel without verified scheduler identity rejected");
        let empty = plan_external_batch_followup(&submit, ExternalBatchOperationKind::Observe, Some(String::new()))
            .expect_err("empty scheduler identity rejected");

        assert_eq!(observe.dispatch_id_blake3, submit.dispatch_id_blake3);
        assert_eq!(reconcile.dispatch_id_blake3, submit.dispatch_id_blake3);
        assert_eq!(reconcile.external_job_id, None);
        assert_ne!(reconcile.operation_id_blake3, submit.operation_id_blake3);
        assert_eq!(missing, "external-batch-external-job-id-missing");
        assert_eq!(empty, "external-batch-external-job-id-length-invalid");
    }

    #[test]
    fn malformed_and_stale_responses_fail_closed() {
        let request =
            plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Observe, Some("slurm-42")))
                .expect("observe plans");
        let mut stale = fixture_response(&request, ExternalBatchJobState::Running);
        stale.fence_generation = stale.fence_generation.advance().expect("fixture fence increments");
        let stale_error = validate_external_batch_response(&request, &stale).expect_err("stale fence is rejected");
        let mut conflicting = fixture_response(&request, ExternalBatchJobState::Running);
        conflicting.external_job_id = Some("slurm-99".to_string());
        let conflict_error =
            validate_external_batch_response(&request, &conflicting).expect_err("conflicting scheduler id is rejected");

        assert_eq!(stale_error, "external-batch-response-attempt-fence-mismatch");
        assert_eq!(conflict_error, "external-batch-response-job-id-conflict");
    }

    #[test]
    fn invalid_limits_and_missing_job_identity_are_rejected() {
        let mut invalid_limit = fixture_input(ExternalBatchOperationKind::Submit, None);
        invalid_limit.limits.startup_timeout_secs = 0;
        let timeout_error = plan_external_batch_operation(invalid_limit).expect_err("zero timeout rejected");
        let missing_job = plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Observe, None))
            .expect_err("observe requires scheduler identity");

        assert_eq!(timeout_error, "external-batch-startup-timeout-invalid");
        assert_eq!(missing_job, "external-batch-external-job-id-missing");
    }

    #[test]
    fn reconciliation_reattaches_waits_preserves_unknown_resubmits_and_marks_lost() {
        let request =
            plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Reconcile, Some("slurm-42")))
                .expect("reconcile plans");
        let running = fixture_response(&request, ExternalBatchJobState::Running);
        let failed = fixture_response(&request, ExternalBatchJobState::Failed);
        let unknown = fixture_response(&request, ExternalBatchJobState::Unknown);
        let registered = decide_external_batch_reconciliation(&request, &running, ExternalBatchReconcileFacts {
            worker_registered: true,
            reconcile_attempt: 1,
            overall_deadline_exceeded: false,
        })
        .expect("registered worker reattaches");
        let waiting = decide_external_batch_reconciliation(&request, &running, ExternalBatchReconcileFacts {
            worker_registered: false,
            reconcile_attempt: 1,
            overall_deadline_exceeded: false,
        })
        .expect("running job awaits registration");
        let preserve_unknown = decide_external_batch_reconciliation(&request, &unknown, ExternalBatchReconcileFacts {
            worker_registered: false,
            reconcile_attempt: 1,
            overall_deadline_exceeded: false,
        })
        .expect("unknown provider state stays conservative");
        let resubmit = decide_external_batch_reconciliation(&request, &failed, ExternalBatchReconcileFacts {
            worker_registered: false,
            reconcile_attempt: 1,
            overall_deadline_exceeded: false,
        })
        .expect("failed allocation retries within bounds");
        let lost = decide_external_batch_reconciliation(&request, &failed, ExternalBatchReconcileFacts {
            worker_registered: false,
            reconcile_attempt: TEST_RECONCILE_ATTEMPTS,
            overall_deadline_exceeded: false,
        })
        .expect("exhausted allocation is lost");

        assert_eq!(registered, ExternalBatchReconcileDecision::Reattach);
        assert_eq!(waiting, ExternalBatchReconcileDecision::AwaitWorkerRegistration);
        assert_eq!(preserve_unknown, ExternalBatchReconcileDecision::PreserveUnknown);
        assert_eq!(resubmit, ExternalBatchReconcileDecision::Resubmit);
        assert_eq!(lost, ExternalBatchReconcileDecision::MarkLost);
    }

    #[test]
    fn terminal_state_conflicts_and_backwards_transitions_are_rejected() {
        let terminal =
            validate_external_batch_state_transition(ExternalBatchJobState::Succeeded, ExternalBatchJobState::Failed)
                .expect_err("contradictory terminal state rejected");
        let backwards =
            validate_external_batch_state_transition(ExternalBatchJobState::Running, ExternalBatchJobState::Pending)
                .expect_err("backwards state rejected");
        let recovered =
            validate_external_batch_state_transition(ExternalBatchJobState::Unknown, ExternalBatchJobState::Running);

        assert_eq!(terminal, "external-batch-state-transition-terminal-conflict");
        assert_eq!(backwards, "external-batch-state-transition-invalid");
        assert!(recovered.is_ok());
    }

    #[test]
    fn malformed_operation_enum_and_oversized_response_reason_are_rejected() {
        let request = plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Submit, None))
            .expect("submit plans");
        let mut encoded = serde_json::to_value(&request).expect("operation encodes");
        encoded["operation"] = serde_json::Value::String("execute-shell".to_string());
        let enum_error =
            serde_json::from_value::<ExternalBatchOperation>(encoded).expect_err("unknown operation rejected");
        let mut response = fixture_response(&request, ExternalBatchJobState::Submitted);
        response.reason_code = "x".repeat(MAX_EXTERNAL_BATCH_REASON_BYTES.saturating_add(1));
        let reason_error =
            validate_external_batch_response(&request, &response).expect_err("oversized response reason rejected");

        assert!(enum_error.to_string().contains("unknown variant"));
        assert_eq!(reason_error, "external-batch-response-reason-length-invalid");
    }
}
