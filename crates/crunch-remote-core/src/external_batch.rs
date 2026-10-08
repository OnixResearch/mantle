use alloc::string::{String, ToString};

use crate::output::is_blake3_hex_digest;

pub const PROTOCOL_SCHEMA: &str = "mantle-batch-dispatch-adapter-v1";
pub const DISPATCH_DOMAIN: &str = "mantle-external-batch-dispatch-v1";
pub const OPERATION_DOMAIN: &str = "mantle-external-batch-operation-v1";
pub const CLAIM_SCOPE: &str = "external-allocation-control-only";
pub const NON_CLAIM: &str = "external scheduler identity and state do not authorize workers, transfer output trust, prove execution success, or admit outputs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchKind {
    Submit,
    Observe,
    Cancel,
    Reconcile,
}

impl BatchKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Reconcile => "reconcile",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchState {
    Submitted,
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}

impl BatchState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchDecision {
    Reattach,
    AwaitWorkerRegistration,
    PreserveUnknown,
    Resubmit,
    MarkLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierKind {
    AdapterInstance,
    WorkerEndpoint,
    ProviderClass,
    SemanticCapabilityClass,
    ExternalJobId,
    ResponseReason,
    ResponseJobId,
}

impl IdentifierKind {
    const fn error_label(self) -> &'static str {
        match self {
            Self::AdapterInstance => "external-batch-adapter-instance",
            Self::WorkerEndpoint => "external-batch-worker-endpoint",
            Self::ProviderClass => "external-batch-provider-class",
            Self::SemanticCapabilityClass => "external-batch-semantic-capability-class",
            Self::ExternalJobId => "external-batch-external-job-id",
            Self::ResponseReason => "external-batch-response-reason",
            Self::ResponseJobId => "external-batch-response-job-id",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchError {
    IdentifierLength(IdentifierKind),
    IdentifierControlCharacter(IdentifierKind),
    ProfileOrBootstrapRefInvalid,
    DispatcherGenerationZero,
    NormalizedBuildKeyInvalid,
    StartupTimeoutInvalid,
    TerminalTimeoutInvalid,
    ReconcileAttemptLimitInvalid,
    SubmitExternalJobIdPresent,
    ExternalJobIdMissing,
    FollowupSubmitBasisInvalid,
    FollowupKindSubmit,
    OperationSchemaUnsupported,
    OperationDigestInvalid,
    OperationClaimBoundaryInvalid,
    ResponseSchemaUnsupported,
    ResponseOperationMismatch,
    ResponseDispatchMismatch,
    ResponseAdapterFenceMismatch,
    ResponseAttemptFenceMismatch,
    ResponseNonClaimInvalid,
    ResponseJobIdMissing,
    ResponseJobIdConflict,
    StateTransitionTerminalConflict,
    StateTransitionInvalid,
    ReconcileAttemptLimitExceeded,
    HashLabelLengthOverflow,
    HashValueLengthOverflow,
}

impl BatchError {
    pub fn diagnostic(self) -> String {
        match self {
            Self::IdentifierLength(kind) => alloc::format!("{}-length-invalid", kind.error_label()),
            Self::IdentifierControlCharacter(kind) => alloc::format!("{}-control-character", kind.error_label()),
            _ => self.as_str().to_string(),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::IdentifierLength(_) | Self::IdentifierControlCharacter(_) => "",
            Self::ProfileOrBootstrapRefInvalid => "external-batch-profile-or-bootstrap-ref-invalid",
            Self::DispatcherGenerationZero => "external-batch-dispatcher-generation-zero",
            Self::NormalizedBuildKeyInvalid => "external-batch-normalized-build-key-invalid",
            Self::StartupTimeoutInvalid => "external-batch-startup-timeout-invalid",
            Self::TerminalTimeoutInvalid => "external-batch-terminal-timeout-invalid",
            Self::ReconcileAttemptLimitInvalid => "external-batch-reconcile-attempt-limit-invalid",
            Self::SubmitExternalJobIdPresent => "external-batch-submit-external-job-id-present",
            Self::ExternalJobIdMissing => "external-batch-external-job-id-missing",
            Self::FollowupSubmitBasisInvalid => "external-batch-followup-submit-basis-invalid",
            Self::FollowupKindSubmit => "external-batch-followup-kind-submit",
            Self::OperationSchemaUnsupported => "external-batch-operation-schema-unsupported",
            Self::OperationDigestInvalid => "external-batch-operation-digest-invalid",
            Self::OperationClaimBoundaryInvalid => "external-batch-operation-claim-boundary-invalid",
            Self::ResponseSchemaUnsupported => "external-batch-response-schema-unsupported",
            Self::ResponseOperationMismatch => "external-batch-response-operation-mismatch",
            Self::ResponseDispatchMismatch => "external-batch-response-dispatch-mismatch",
            Self::ResponseAdapterFenceMismatch => "external-batch-response-adapter-fence-mismatch",
            Self::ResponseAttemptFenceMismatch => "external-batch-response-attempt-fence-mismatch",
            Self::ResponseNonClaimInvalid => "external-batch-response-non-claim-invalid",
            Self::ResponseJobIdMissing => "external-batch-response-job-id-missing",
            Self::ResponseJobIdConflict => "external-batch-response-job-id-conflict",
            Self::StateTransitionTerminalConflict => "external-batch-state-transition-terminal-conflict",
            Self::StateTransitionInvalid => "external-batch-state-transition-invalid",
            Self::ReconcileAttemptLimitExceeded => "external-batch-reconcile-attempt-limit-exceeded",
            Self::HashLabelLengthOverflow => "external-batch-hash-label-length-overflow",
            Self::HashValueLengthOverflow => "external-batch-hash-value-length-overflow",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BatchLimits {
    pub startup_timeout_secs: u64,
    pub terminal_timeout_secs: u64,
    pub max_reconcile_attempts: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct BatchInputFacts<'a> {
    pub kind: BatchKind,
    pub adapter_instance_id: &'a str,
    pub dispatcher_profile_ref_blake3: &'a str,
    pub provider_class: &'a str,
    pub worker_bootstrap_ref_blake3: &'a str,
    pub semantic_capability_class: &'a str,
    pub dispatcher_generation: u64,
    pub normalized_build_key: &'a str,
    pub expected_worker_endpoint_id: &'a str,
    pub limits: BatchLimits,
    pub external_job_id: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct BatchDispatchFacts<'a> {
    pub adapter_instance_id: &'a str,
    pub dispatcher_profile_ref_blake3: &'a str,
    pub provider_class: &'a str,
    pub worker_bootstrap_ref_blake3: &'a str,
    pub semantic_capability_class: &'a str,
    pub dispatcher_generation: u64,
    pub normalized_build_key: &'a str,
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub fence_generation: u64,
    pub expected_worker_endpoint_id: &'a str,
    /// Exact bytes supplied by the serde/physical adapter; no JSON is reserialized here.
    pub resource_projection_json: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub struct BatchOperationFacts<'a> {
    pub schema: &'a str,
    pub kind: BatchKind,
    pub operation_id_blake3: &'a str,
    pub dispatch_id_blake3: &'a str,
    pub normalized_build_key: &'a str,
    pub dispatcher_profile_ref_blake3: &'a str,
    pub worker_bootstrap_ref_blake3: &'a str,
    pub adapter_instance_id: &'a str,
    pub dispatcher_generation: u64,
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub fence_generation: u64,
    pub external_job_id: Option<&'a str>,
    pub claim_scope: &'a str,
    pub non_claim: &'a str,
    pub limits: BatchLimits,
}

#[derive(Debug, Clone, Copy)]
pub struct BatchResponseFacts<'a> {
    pub schema: &'a str,
    pub kind: BatchKind,
    pub operation_id_blake3: &'a str,
    pub dispatch_id_blake3: &'a str,
    pub adapter_instance_id: &'a str,
    pub dispatcher_generation: u64,
    pub job_id: &'a str,
    pub attempt_id: &'a str,
    pub fence_generation: u64,
    pub state: BatchState,
    pub external_job_id: Option<&'a str>,
    pub reason_code: &'a str,
    pub non_claim: &'a str,
}

pub fn validate_identifier(kind: IdentifierKind, value: &str, max_bytes: usize) -> Result<(), BatchError> {
    if value.is_empty() || value.len() > max_bytes {
        return Err(BatchError::IdentifierLength(kind));
    }
    if value.chars().any(char::is_control) {
        return Err(BatchError::IdentifierControlCharacter(kind));
    }
    Ok(())
}

pub fn validate_limits(limits: BatchLimits, max_timeout: u64, max_reconcile: u32) -> Result<(), BatchError> {
    if limits.startup_timeout_secs == 0 || limits.startup_timeout_secs > max_timeout {
        return Err(BatchError::StartupTimeoutInvalid);
    }
    if limits.terminal_timeout_secs == 0 || limits.terminal_timeout_secs > max_timeout {
        return Err(BatchError::TerminalTimeoutInvalid);
    }
    if limits.max_reconcile_attempts == 0 || limits.max_reconcile_attempts > max_reconcile {
        return Err(BatchError::ReconcileAttemptLimitInvalid);
    }
    Ok(())
}

pub fn validate_input(
    facts: BatchInputFacts<'_>,
    max_identifier_bytes: usize,
    max_timeout: u64,
    max_reconcile: u32,
) -> Result<(), BatchError> {
    validate_identifier(IdentifierKind::AdapterInstance, facts.adapter_instance_id, max_identifier_bytes)?;
    validate_identifier(IdentifierKind::WorkerEndpoint, facts.expected_worker_endpoint_id, max_identifier_bytes)?;
    validate_identifier(IdentifierKind::ProviderClass, facts.provider_class, max_identifier_bytes)?;
    validate_identifier(IdentifierKind::SemanticCapabilityClass, facts.semantic_capability_class, max_identifier_bytes)?;
    if !is_blake3_hex_digest(facts.dispatcher_profile_ref_blake3)
        || !is_blake3_hex_digest(facts.worker_bootstrap_ref_blake3)
    {
        return Err(BatchError::ProfileOrBootstrapRefInvalid);
    }
    if facts.dispatcher_generation == 0 {
        return Err(BatchError::DispatcherGenerationZero);
    }
    if !is_blake3_hex_digest(facts.normalized_build_key) {
        return Err(BatchError::NormalizedBuildKeyInvalid);
    }
    validate_limits(facts.limits, max_timeout, max_reconcile)?;
    match (facts.kind, facts.external_job_id) {
        (BatchKind::Submit | BatchKind::Reconcile, None) => Ok(()),
        (BatchKind::Submit, Some(_)) => Err(BatchError::SubmitExternalJobIdPresent),
        (_, Some(external_job_id)) => validate_identifier(IdentifierKind::ExternalJobId, external_job_id, max_identifier_bytes),
        (_, None) => Err(BatchError::ExternalJobIdMissing),
    }
}

pub fn validate_followup(
    submit: BatchOperationFacts<'_>,
    kind: BatchKind,
    external_job_id: Option<&str>,
    max_identifier_bytes: usize,
) -> Result<(), BatchError> {
    if submit.kind != BatchKind::Submit || submit.external_job_id.is_some() {
        return Err(BatchError::FollowupSubmitBasisInvalid);
    }
    match (kind, external_job_id) {
        (BatchKind::Submit, _) => Err(BatchError::FollowupKindSubmit),
        (BatchKind::Reconcile, None) => Ok(()),
        (_, Some(id)) => validate_identifier(IdentifierKind::ExternalJobId, id, max_identifier_bytes),
        (_, None) => Err(BatchError::ExternalJobIdMissing),
    }
}

pub fn validate_operation(facts: BatchOperationFacts<'_>, max_timeout: u64, max_reconcile: u32) -> Result<(), BatchError> {
    if facts.schema != PROTOCOL_SCHEMA {
        return Err(BatchError::OperationSchemaUnsupported);
    }
    if [
        facts.operation_id_blake3,
        facts.dispatch_id_blake3,
        facts.normalized_build_key,
        facts.dispatcher_profile_ref_blake3,
        facts.worker_bootstrap_ref_blake3,
    ]
    .iter()
    .any(|digest| !is_blake3_hex_digest(digest))
    {
        return Err(BatchError::OperationDigestInvalid);
    }
    if facts.claim_scope != CLAIM_SCOPE || facts.non_claim != NON_CLAIM {
        return Err(BatchError::OperationClaimBoundaryInvalid);
    }
    validate_limits(facts.limits, max_timeout, max_reconcile)
}

pub fn validate_response(
    request: BatchOperationFacts<'_>,
    response: BatchResponseFacts<'_>,
    max_identifier_bytes: usize,
    max_reason_bytes: usize,
) -> Result<(), BatchError> {
    if response.schema != PROTOCOL_SCHEMA {
        return Err(BatchError::ResponseSchemaUnsupported);
    }
    if response.kind != request.kind || response.operation_id_blake3 != request.operation_id_blake3 {
        return Err(BatchError::ResponseOperationMismatch);
    }
    if response.dispatch_id_blake3 != request.dispatch_id_blake3 {
        return Err(BatchError::ResponseDispatchMismatch);
    }
    if response.adapter_instance_id != request.adapter_instance_id
        || response.dispatcher_generation != request.dispatcher_generation
    {
        return Err(BatchError::ResponseAdapterFenceMismatch);
    }
    if response.job_id != request.job_id
        || response.attempt_id != request.attempt_id
        || response.fence_generation != request.fence_generation
    {
        return Err(BatchError::ResponseAttemptFenceMismatch);
    }
    validate_identifier(IdentifierKind::ResponseReason, response.reason_code, max_reason_bytes)?;
    if response.non_claim != NON_CLAIM {
        return Err(BatchError::ResponseNonClaimInvalid);
    }
    if response.external_job_id.is_none() && response.state != BatchState::Unknown {
        return Err(BatchError::ResponseJobIdMissing);
    }
    if let Some(id) = response.external_job_id {
        validate_identifier(IdentifierKind::ResponseJobId, id, max_identifier_bytes)?;
    }
    if let Some(expected) = request.external_job_id {
        if response.external_job_id != Some(expected) {
            return Err(BatchError::ResponseJobIdConflict);
        }
    }
    Ok(())
}

pub fn validate_state_transition(previous: BatchState, next: BatchState) -> Result<(), BatchError> {
    if previous == BatchState::Unknown || previous == next {
        return Ok(());
    }
    if previous.is_terminal() {
        return Err(BatchError::StateTransitionTerminalConflict);
    }
    match (previous, next) {
        (BatchState::Submitted, _)
        | (BatchState::Pending, BatchState::Running)
        | (BatchState::Pending, BatchState::Succeeded)
        | (BatchState::Pending, BatchState::Failed)
        | (BatchState::Pending, BatchState::Cancelled)
        | (BatchState::Pending, BatchState::Unknown)
        | (BatchState::Running, BatchState::Succeeded)
        | (BatchState::Running, BatchState::Failed)
        | (BatchState::Running, BatchState::Cancelled)
        | (BatchState::Running, BatchState::Unknown) => Ok(()),
        _ => Err(BatchError::StateTransitionInvalid),
    }
}

pub fn decide_reconciliation(
    state: BatchState,
    worker_registered: bool,
    reconcile_attempt: u32,
    max_reconcile_attempts: u32,
    overall_deadline_exceeded: bool,
) -> Result<BatchDecision, BatchError> {
    if reconcile_attempt > max_reconcile_attempts {
        return Err(BatchError::ReconcileAttemptLimitExceeded);
    }
    if worker_registered {
        return Ok(BatchDecision::Reattach);
    }
    if overall_deadline_exceeded {
        return Ok(BatchDecision::MarkLost);
    }
    let decision = match state {
        BatchState::Submitted | BatchState::Pending | BatchState::Running => BatchDecision::AwaitWorkerRegistration,
        BatchState::Unknown => BatchDecision::PreserveUnknown,
        BatchState::Succeeded | BatchState::Failed | BatchState::Cancelled => {
            if reconcile_attempt < max_reconcile_attempts {
                BatchDecision::Resubmit
            } else {
                BatchDecision::MarkLost
            }
        }
    };
    Ok(decision)
}

pub fn derive_dispatch_id(facts: BatchDispatchFacts<'_>) -> Result<String, BatchError> {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, "domain", DISPATCH_DOMAIN.as_bytes())?;
    hash_field(&mut hasher, "adapter-instance", facts.adapter_instance_id.as_bytes())?;
    hash_field(&mut hasher, "dispatcher-profile", facts.dispatcher_profile_ref_blake3.as_bytes())?;
    hash_field(&mut hasher, "provider-class", facts.provider_class.as_bytes())?;
    hash_field(&mut hasher, "worker-bootstrap", facts.worker_bootstrap_ref_blake3.as_bytes())?;
    hash_field(&mut hasher, "semantic-capability", facts.semantic_capability_class.as_bytes())?;
    hash_field(&mut hasher, "dispatcher-generation", &facts.dispatcher_generation.to_le_bytes())?;
    hash_field(&mut hasher, "normalized-build-key", facts.normalized_build_key.as_bytes())?;
    hash_field(&mut hasher, "job-id", facts.job_id.as_bytes())?;
    hash_field(&mut hasher, "attempt-id", facts.attempt_id.as_bytes())?;
    hash_field(&mut hasher, "fence-generation", &facts.fence_generation.to_le_bytes())?;
    hash_field(&mut hasher, "worker-endpoint", facts.expected_worker_endpoint_id.as_bytes())?;
    hash_field(&mut hasher, "resources", facts.resource_projection_json)?;
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn derive_operation_id(kind: BatchKind, dispatch_id_blake3: &str) -> Result<String, BatchError> {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, "domain", OPERATION_DOMAIN.as_bytes())?;
    hash_field(&mut hasher, "operation", kind.as_str().as_bytes())?;
    hash_field(&mut hasher, "dispatch", dispatch_id_blake3.as_bytes())?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_field(hasher: &mut blake3::Hasher, label: &str, value: &[u8]) -> Result<(), BatchError> {
    let label_len = u64::try_from(label.len()).map_err(|_| BatchError::HashLabelLengthOverflow)?;
    let value_len = u64::try_from(value.len()).map_err(|_| BatchError::HashValueLengthOverflow)?;
    hasher.update(&label_len.to_le_bytes());
    hasher.update(label.as_bytes());
    hasher.update(&value_len.to_le_bytes());
    hasher.update(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenced_response_rejects_stale_identity_before_untrusted_job_and_terminal_reconcile() {
        let digest = "a".repeat(64);
        let request = BatchOperationFacts {
            schema: PROTOCOL_SCHEMA,
            kind: BatchKind::Submit,
            operation_id_blake3: &digest,
            dispatch_id_blake3: &digest,
            normalized_build_key: &digest,
            dispatcher_profile_ref_blake3: &digest,
            worker_bootstrap_ref_blake3: &digest,
            adapter_instance_id: "adapter",
            dispatcher_generation: 7,
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 2,
            external_job_id: None,
            claim_scope: CLAIM_SCOPE,
            non_claim: NON_CLAIM,
            limits: BatchLimits { startup_timeout_secs: 10, terminal_timeout_secs: 20, max_reconcile_attempts: 3 },
        };
        let response = BatchResponseFacts {
            schema: PROTOCOL_SCHEMA,
            kind: BatchKind::Submit,
            operation_id_blake3: &digest,
            dispatch_id_blake3: &digest,
            adapter_instance_id: "adapter",
            dispatcher_generation: 7,
            job_id: "job",
            attempt_id: "attempt",
            fence_generation: 2,
            state: BatchState::Succeeded,
            external_job_id: Some("external-1"),
            reason_code: "submitted",
            non_claim: NON_CLAIM,
        };
        assert_eq!(validate_operation(request, 100, 3), Ok(()));
        assert_eq!(validate_response(request, response, 256, 128), Ok(()));
        assert_eq!(
            validate_response(request, BatchResponseFacts { fence_generation: 1, ..response }, 256, 128),
            Err(BatchError::ResponseAttemptFenceMismatch)
        );
        assert_eq!(decide_reconciliation(BatchState::Succeeded, false, 2, 3, false), Ok(BatchDecision::Resubmit));
        assert_eq!(decide_reconciliation(BatchState::Succeeded, false, 3, 3, false), Ok(BatchDecision::MarkLost));
        assert_eq!(decide_reconciliation(BatchState::Unknown, true, 1, 3, false), Ok(BatchDecision::Reattach));
    }
}
