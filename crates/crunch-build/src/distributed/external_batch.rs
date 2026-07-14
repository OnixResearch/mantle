//! Pure protocol and reconciliation core for external batch dispatchers.
//!
//! The coordinator shell owns process execution, clocks, persistence, worker
//! registration, and output admission. This module only normalizes bounded
//! requests, derives BLAKE3 identities, validates responses, and decides how a
//! current fenced allocation should reconcile.
//!
//! r[impl remote_builds.external_batch_adapter_protocol]
//! r[impl remote_builds.external_batch_resource_projection]
//! r[impl remote_builds.external_batch_reconciliation]

use serde::Deserialize;
use serde::Serialize;

use super::RemoteAttemptId;
use super::RemoteFenceGeneration;
use super::RemoteJobId;
use super::RemoteNamedResourceQuantity;
use super::RemoteResourceRequirements;
use super::canonical_remote_resource_requirements;

pub const EXTERNAL_BATCH_PROTOCOL_SCHEMA: &str = "mantle-batch-dispatch-adapter-v1";
pub const EXTERNAL_BATCH_DISPATCH_DOMAIN: &str = "mantle-external-batch-dispatch-v1";
pub const EXTERNAL_BATCH_OPERATION_DOMAIN: &str = "mantle-external-batch-operation-v1";
pub const EXTERNAL_BATCH_CLAIM_SCOPE: &str = "external-allocation-control-only";
pub const EXTERNAL_BATCH_NON_CLAIM: &str = "external scheduler identity and state do not authorize workers, transfer output trust, prove execution success, or admit outputs";
pub const MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES: usize = 256;
pub const MAX_EXTERNAL_BATCH_REASON_BYTES: usize = 128;
pub const MAX_EXTERNAL_BATCH_TIMEOUT_SECS: u64 = 604_800;
pub const MAX_EXTERNAL_BATCH_RECONCILE_ATTEMPTS: u32 = 1_024;
const BLAKE3_HEX_LENGTH: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalBatchOperationKind {
    Submit,
    Observe,
    Cancel,
    Reconcile,
}

impl ExternalBatchOperationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Reconcile => "reconcile",
        }
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
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalBatchOperation {
    pub schema: String,
    pub operation: ExternalBatchOperationKind,
    pub operation_id_blake3: String,
    pub dispatch_id_blake3: String,
    pub adapter_instance_id: String,
    pub dispatcher_generation: u64,
    pub normalized_build_key: String,
    pub job_id: RemoteJobId,
    pub attempt_id: RemoteAttemptId,
    pub fence_generation: RemoteFenceGeneration,
    pub expected_worker_endpoint_id: String,
    pub resources: ExternalBatchResourceProjection,
    pub limits: ExternalBatchLimits,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_job_id: Option<String>,
    pub reason_code: String,
    pub observed_unix_s: u64,
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
    Resubmit,
    MarkLost,
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
    let operation_id_blake3 = derive_operation_id(input.kind, &dispatch_id_blake3);
    let operation = ExternalBatchOperation {
        schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
        operation: input.kind,
        operation_id_blake3,
        dispatch_id_blake3,
        adapter_instance_id: input.adapter_instance_id,
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
    assert!(is_blake3_hex_digest(&operation.operation_id_blake3));
    assert!(is_blake3_hex_digest(&operation.dispatch_id_blake3));
    Ok(operation)
}

pub fn plan_external_batch_followup(
    submit: &ExternalBatchOperation,
    kind: ExternalBatchOperationKind,
    external_job_id: String,
) -> Result<ExternalBatchOperation, String> {
    validate_operation(submit)?;
    if submit.operation != ExternalBatchOperationKind::Submit || submit.external_job_id.is_some() {
        return Err("external-batch-followup-submit-basis-invalid".to_string());
    }
    if kind == ExternalBatchOperationKind::Submit {
        return Err("external-batch-followup-kind-submit".to_string());
    }
    validate_bounded_identifier(
        "external-batch-external-job-id",
        &external_job_id,
        MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
    )?;
    let mut followup = submit.clone();
    followup.operation = kind;
    followup.operation_id_blake3 = derive_operation_id(kind, &submit.dispatch_id_blake3);
    followup.external_job_id = Some(external_job_id);
    assert_eq!(followup.dispatch_id_blake3, submit.dispatch_id_blake3);
    assert_ne!(followup.operation_id_blake3, submit.operation_id_blake3);
    Ok(followup)
}

pub fn validate_external_batch_response(
    request: &ExternalBatchOperation,
    response: &ExternalBatchOperationResponse,
) -> Result<(), String> {
    validate_operation(request)?;
    if response.schema != EXTERNAL_BATCH_PROTOCOL_SCHEMA {
        return Err("external-batch-response-schema-unsupported".to_string());
    }
    if response.operation != request.operation || response.operation_id_blake3 != request.operation_id_blake3 {
        return Err("external-batch-response-operation-mismatch".to_string());
    }
    if response.dispatch_id_blake3 != request.dispatch_id_blake3 {
        return Err("external-batch-response-dispatch-mismatch".to_string());
    }
    if response.adapter_instance_id != request.adapter_instance_id
        || response.dispatcher_generation != request.dispatcher_generation
    {
        return Err("external-batch-response-adapter-fence-mismatch".to_string());
    }
    if response.job_id != request.job_id
        || response.attempt_id != request.attempt_id
        || response.fence_generation != request.fence_generation
    {
        return Err("external-batch-response-attempt-fence-mismatch".to_string());
    }
    validate_bounded_identifier(
        "external-batch-response-reason",
        &response.reason_code,
        MAX_EXTERNAL_BATCH_REASON_BYTES,
    )?;
    validate_external_job_id(response.state, response.external_job_id.as_deref())?;
    if let Some(expected) = request.external_job_id.as_deref() {
        if response.external_job_id.as_deref() != Some(expected) {
            return Err("external-batch-response-job-id-conflict".to_string());
        }
    }
    assert!(response.observed_unix_s > 0);
    assert_eq!(response.dispatch_id_blake3, request.dispatch_id_blake3);
    Ok(())
}

pub fn decide_external_batch_reconciliation(
    request: &ExternalBatchOperation,
    response: &ExternalBatchOperationResponse,
    facts: ExternalBatchReconcileFacts,
) -> Result<ExternalBatchReconcileDecision, String> {
    validate_external_batch_response(request, response)?;
    if facts.reconcile_attempt > request.limits.max_reconcile_attempts {
        return Err("external-batch-reconcile-attempt-limit-exceeded".to_string());
    }
    if facts.worker_registered {
        return Ok(ExternalBatchReconcileDecision::Reattach);
    }
    if facts.overall_deadline_exceeded {
        return Ok(ExternalBatchReconcileDecision::MarkLost);
    }
    let attempts_remain = facts.reconcile_attempt < request.limits.max_reconcile_attempts;
    let decision = match response.state {
        ExternalBatchJobState::Submitted | ExternalBatchJobState::Pending | ExternalBatchJobState::Running => {
            ExternalBatchReconcileDecision::AwaitWorkerRegistration
        }
        ExternalBatchJobState::Succeeded
        | ExternalBatchJobState::Failed
        | ExternalBatchJobState::Cancelled
        | ExternalBatchJobState::Unknown => {
            if attempts_remain {
                ExternalBatchReconcileDecision::Resubmit
            } else {
                ExternalBatchReconcileDecision::MarkLost
            }
        }
    };
    assert!(!facts.worker_registered);
    assert!(facts.reconcile_attempt <= request.limits.max_reconcile_attempts);
    Ok(decision)
}

fn validate_operation_input(input: &ExternalBatchOperationInput) -> Result<(), String> {
    validate_bounded_identifier(
        "external-batch-adapter-instance",
        &input.adapter_instance_id,
        MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
    )?;
    validate_bounded_identifier(
        "external-batch-worker-endpoint",
        &input.expected_worker_endpoint_id,
        MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
    )?;
    if input.dispatcher_generation == 0 {
        return Err("external-batch-dispatcher-generation-zero".to_string());
    }
    if !is_blake3_hex_digest(&input.normalized_build_key) {
        return Err("external-batch-normalized-build-key-invalid".to_string());
    }
    validate_external_batch_limits(input.limits)?;
    match (input.kind, input.external_job_id.as_deref()) {
        (ExternalBatchOperationKind::Submit, None) => {}
        (ExternalBatchOperationKind::Submit, Some(_)) => {
            return Err("external-batch-submit-external-job-id-present".to_string());
        }
        (_, Some(external_job_id)) => validate_bounded_identifier(
            "external-batch-external-job-id",
            external_job_id,
            MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
        )?,
        (_, None) => return Err("external-batch-external-job-id-missing".to_string()),
    }
    Ok(())
}

fn validate_operation(operation: &ExternalBatchOperation) -> Result<(), String> {
    if operation.schema != EXTERNAL_BATCH_PROTOCOL_SCHEMA {
        return Err("external-batch-operation-schema-unsupported".to_string());
    }
    if !is_blake3_hex_digest(&operation.operation_id_blake3)
        || !is_blake3_hex_digest(&operation.dispatch_id_blake3)
        || !is_blake3_hex_digest(&operation.normalized_build_key)
    {
        return Err("external-batch-operation-digest-invalid".to_string());
    }
    if operation.claim_scope != EXTERNAL_BATCH_CLAIM_SCOPE || operation.non_claim != EXTERNAL_BATCH_NON_CLAIM {
        return Err("external-batch-operation-claim-boundary-invalid".to_string());
    }
    validate_external_batch_limits(operation.limits)
}

fn validate_external_batch_limits(limits: ExternalBatchLimits) -> Result<(), String> {
    if limits.startup_timeout_secs == 0 || limits.startup_timeout_secs > MAX_EXTERNAL_BATCH_TIMEOUT_SECS {
        return Err("external-batch-startup-timeout-invalid".to_string());
    }
    if limits.terminal_timeout_secs == 0 || limits.terminal_timeout_secs > MAX_EXTERNAL_BATCH_TIMEOUT_SECS {
        return Err("external-batch-terminal-timeout-invalid".to_string());
    }
    if limits.max_reconcile_attempts == 0 || limits.max_reconcile_attempts > MAX_EXTERNAL_BATCH_RECONCILE_ATTEMPTS {
        return Err("external-batch-reconcile-attempt-limit-invalid".to_string());
    }
    Ok(())
}

fn validate_external_job_id(state: ExternalBatchJobState, external_job_id: Option<&str>) -> Result<(), String> {
    if external_job_id.is_none() && state != ExternalBatchJobState::Unknown {
        return Err("external-batch-response-job-id-missing".to_string());
    }
    if let Some(external_job_id) = external_job_id {
        validate_bounded_identifier(
            "external-batch-response-job-id",
            external_job_id,
            MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES,
        )?;
    }
    Ok(())
}

fn derive_dispatch_id(
    input: &ExternalBatchOperationInput,
    resources: &ExternalBatchResourceProjection,
) -> Result<String, String> {
    let resource_json = serde_json::to_vec(resources)
        .map_err(|error| format!("external-batch-resource-projection-serialize-failed: {error}"))?;
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, "domain", EXTERNAL_BATCH_DISPATCH_DOMAIN.as_bytes());
    hash_field(&mut hasher, "adapter-instance", input.adapter_instance_id.as_bytes());
    hash_field(&mut hasher, "dispatcher-generation", &input.dispatcher_generation.to_le_bytes());
    hash_field(&mut hasher, "normalized-build-key", input.normalized_build_key.as_bytes());
    hash_field(&mut hasher, "job-id", input.job_id.as_str().as_bytes());
    hash_field(&mut hasher, "attempt-id", input.attempt_id.as_str().as_bytes());
    hash_field(&mut hasher, "fence-generation", &input.fence_generation.get().to_le_bytes());
    hash_field(&mut hasher, "worker-endpoint", input.expected_worker_endpoint_id.as_bytes());
    hash_field(&mut hasher, "resources", &resource_json);
    Ok(hasher.finalize().to_hex().to_string())
}

fn derive_operation_id(kind: ExternalBatchOperationKind, dispatch_id_blake3: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, "domain", EXTERNAL_BATCH_OPERATION_DOMAIN.as_bytes());
    hash_field(&mut hasher, "operation", kind.as_str().as_bytes());
    hash_field(&mut hasher, "dispatch", dispatch_id_blake3.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn hash_field(hasher: &mut blake3::Hasher, label: &str, value: &[u8]) {
    let label_len = u64::try_from(label.len()).expect("static hash label length fits in u64");
    let value_len = u64::try_from(value.len()).expect("bounded hash value length fits in u64");
    hasher.update(&label_len.to_le_bytes());
    hasher.update(label.as_bytes());
    hasher.update(&value_len.to_le_bytes());
    hasher.update(value);
}

fn validate_bounded_identifier(label: &str, value: &str, max_bytes: usize) -> Result<(), String> {
    if value.is_empty() || value.len() > max_bytes {
        return Err(format!("{label}-length-invalid"));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("{label}-control-character"));
    }
    Ok(())
}

fn is_blake3_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
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
    fn followup_preserves_dispatch_identity_and_requires_scheduler_identity() {
        let submit = plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Submit, None))
            .expect("submit plans");
        let observe =
            plan_external_batch_followup(&submit, ExternalBatchOperationKind::Observe, "slurm-42".to_string())
                .expect("followup plans");
        let missing = plan_external_batch_followup(&submit, ExternalBatchOperationKind::Cancel, String::new())
            .expect_err("empty scheduler identity rejected");

        assert_eq!(observe.dispatch_id_blake3, submit.dispatch_id_blake3);
        assert_ne!(observe.operation_id_blake3, submit.operation_id_blake3);
        assert_eq!(missing, "external-batch-external-job-id-length-invalid");
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
    fn reconciliation_reattaches_waits_resubmits_and_marks_lost() {
        let request =
            plan_external_batch_operation(fixture_input(ExternalBatchOperationKind::Reconcile, Some("slurm-42")))
                .expect("reconcile plans");
        let running = fixture_response(&request, ExternalBatchJobState::Running);
        let failed = fixture_response(&request, ExternalBatchJobState::Failed);
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
        assert_eq!(resubmit, ExternalBatchReconcileDecision::Resubmit);
        assert_eq!(lost, ExternalBatchReconcileDecision::MarkLost);
    }
}
