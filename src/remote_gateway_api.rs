//! Durable asynchronous Build API shell for the remote service gateway.
//!
//! This adapter persists only public-to-internal attempt linkage and signed
//! completion-event delivery state. Build, store, lease, fencing, log, and
//! scheduler truth remain in the existing remote coordinator.
//!
//! r[impl remote_builds.versioned_build_api]
//! r[impl remote_builds.idempotent_completion_events]
//! r[impl remote_builds.gateway_bounds_and_recovery]
//! r[impl remote_builds.granular_service_authority]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io::IsTerminal as _;
use std::io::Read as _;
use std::io::Write as _;
use std::os::fd::FromRawFd as _;
use std::path::Path;

use crunch_build::distributed::ExistingGatewaySubmission;
use crunch_build::distributed::GatewayCommand;
use crunch_build::distributed::GatewayCompletionEvent;
use crunch_build::distributed::GatewayEvidence;
use crunch_build::distributed::GatewayOperation;
use crunch_build::distributed::GatewayReconnectFacts;
use crunch_build::distributed::GatewaySubmissionDecision;
use crunch_build::distributed::GatewayTerminalClass;
use crunch_build::distributed::MAX_GATEWAY_MESSAGE_BYTES;
use crunch_build::distributed::RemoteAttemptId;
use crunch_build::distributed::RemoteAttemptLogRecord;
use crunch_build::distributed::RemoteAttemptLogReplayRequest;
use crunch_build::distributed::RemoteAttemptRetryPolicy;
use crunch_build::distributed::SignedGatewayCompletionEvent;
use crunch_build::distributed::bind_gateway_completion_signature;
use crunch_build::distributed::decide_gateway_reconnect;
use crunch_build::distributed::decide_gateway_submission;
use crunch_build::distributed::derive_gateway_attempt_id;
use crunch_build::distributed::gateway_evidence;
use crunch_build::distributed::plan_gateway_completion_event;
use crunch_build::distributed::seal_gateway_cursor;
use crunch_build::distributed::verify_gateway_cursor;
use nix_compat::narinfo::Signature;
use serde::Deserialize;
use serde::Serialize;
use zeroize::Zeroizing;

use crate::errors::RunError;
use crate::remote_build::RemoteConcreteBuildPayload;
use crate::remote_build::RemoteCoordinatorBuildRequest;
use crate::remote_build::RemoteCoordinatorDispatchDecision;
use crate::remote_build::RemoteCoordinatorJobPhase;
use crate::remote_build::RemoteCoordinatorState;
use crate::remote_build::RemoteCoordinatorTerminationCause;
use crate::remote_build::RemoteProductionAttemptBinding;
use crate::remote_build::acquire_remote_coordinator_mutation_guard;
use crate::remote_build::admit_coordinator_dispatch;
use crate::remote_build::load_coordinator_state;
use crate::remote_build::replay_coordinator_attempt_log;
use crate::remote_build::save_coordinator_state;
use crate::remote_build::terminate_coordinator_attempt;
use crate::remote_build::validate_coordinator_build_request;
use crate::remote_gateway::RemoteGatewayApiPlan;
use crate::remote_gateway::RemoteGatewayApiRequest;
use crate::remote_gateway::plan_gateway_api_request;

const GATEWAY_ADAPTER_STATE_SCHEMA: &str = "mantle-remote-gateway-adapter-state-v1";
const GATEWAY_ADAPTER_STATE_FILE: &str = "remote-gateway-state.json";
const GATEWAY_ADAPTER_STATE_BYTES_MAX: u64 = 8_388_608;
const GATEWAY_ADAPTER_SUBMISSIONS_MAX: u32 = 4_096;
const GATEWAY_CURSOR_KEY_BYTES: usize = 32;
const GATEWAY_CURSOR_KEY_READ_BYTES: u64 = 33;
const GATEWAY_LOG_RECORDS_MAX: u32 = 1_024;
const GATEWAY_CURSOR_TTL_SECS: u64 = 3_600;
const GATEWAY_COMPLETION_SIGNATURE_DOMAIN: &[u8] = b"mantle-remote-completion-signature-v1";
const GATEWAY_API_NON_CLAIM: &str = "asynchronous gateway observations prove only bounded coordinator state for one admitted request; they do not prove build correctness, output usability, arbitrary Nix compatibility, or release eligibility";
#[cfg(unix)]
const OWNER_ONLY_FILE_MODE: u32 = 0o600;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteGatewayDispatchRequest {
    pub api: RemoteGatewayApiRequest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinator_request: Option<RemoteCoordinatorBuildRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RemoteAttemptRetryPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatewayStoredSubmission {
    public_attempt_id: RemoteAttemptId,
    subject: String,
    account_scope: String,
    idempotency_key: String,
    request_identity_blake3: String,
    policy_identity_blake3: String,
    internal_job_id: crunch_build::distributed::RemoteJobId,
    internal_binding: RemoteProductionAttemptBinding,
}

impl GatewayStoredSubmission {
    fn existing(&self) -> ExistingGatewaySubmission {
        ExistingGatewaySubmission {
            attempt_id: self.public_attempt_id.clone(),
            subject: self.subject.clone(),
            account_scope: self.account_scope.clone(),
            idempotency_key: self.idempotency_key.clone(),
            request_identity_blake3: self.request_identity_blake3.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatewayAdapterState {
    schema: String,
    next_completion_sequence: u64,
    submissions: BTreeMap<String, GatewayStoredSubmission>,
    completions_by_attempt: BTreeMap<String, SignedGatewayCompletionEvent>,
}

impl Default for GatewayAdapterState {
    fn default() -> Self {
        Self {
            schema: GATEWAY_ADAPTER_STATE_SCHEMA.to_string(),
            next_completion_sequence: 1,
            submissions: BTreeMap::new(),
            completions_by_attempt: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayAttemptStatus {
    pub public_attempt_id: String,
    pub phase: RemoteCoordinatorJobPhase,
    pub result_available: bool,
    pub transferred_bytes: u64,
    pub short_error: Option<String>,
    pub completion: Option<SignedGatewayCompletionEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum GatewayObservedEvent {
    Attempt {
        event_id: String,
        payload_digest_blake3: String,
    },
    Completion {
        signed: SignedGatewayCompletionEvent,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayUsageItem {
    pub public_attempt_id: String,
    pub phase: RemoteCoordinatorJobPhase,
    pub result_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum RemoteGatewayDispatchResponse {
    Submitted {
        public_attempt_id: String,
        recovered: bool,
        evidence: GatewayEvidence,
        non_claim: String,
    },
    Status {
        status: GatewayAttemptStatus,
        evidence: GatewayEvidence,
        non_claim: String,
    },
    Logs {
        public_attempt_id: String,
        records: Vec<RemoteAttemptLogRecord>,
        next_cursor: Option<String>,
        has_more: bool,
        evidence: GatewayEvidence,
        non_claim: String,
    },
    Events {
        public_attempt_id: String,
        events: Vec<GatewayObservedEvent>,
        next_cursor: Option<String>,
        has_more: bool,
        evidence: GatewayEvidence,
        non_claim: String,
    },
    Cancellation {
        public_attempt_id: String,
        cancellation_applied: bool,
        completion: SignedGatewayCompletionEvent,
        evidence: GatewayEvidence,
        non_claim: String,
    },
    SignedResult {
        public_attempt_id: String,
        result_identity_blake3: Option<String>,
        completion: Option<SignedGatewayCompletionEvent>,
        evidence: GatewayEvidence,
        non_claim: String,
    },
    Usage {
        account_scope: String,
        attempts: Vec<GatewayUsageItem>,
        evidence: GatewayEvidence,
        non_claim: String,
    },
}

pub fn cmd_gateway_api_dispatch(
    authority_path: &Path,
    cursor_key_fd: i32,
    secret_request: crate::remote_service_secrets::RemoteServiceSecretRequest,
    state_dir: &Path,
) -> Result<(), RunError> {
    let trusted_authority = crate::remote_gateway::read_gateway_json_file::<
        crunch_build::distributed::RemoteGatewayAuthority,
    >(authority_path)?;
    let mut request = read_dispatch_stdin()?;
    require_trusted_authority(&request.api.authority, &trusted_authority)?;
    request.api.now_unix_s = crate::remote_gateway::gateway_now_unix_s()
        .map_err(|error| RunError::Internal(format!("reading gateway clock: {error}")))?;
    validate_dispatch_message_bound(&request)?;
    plan_gateway_api_request(request.api.clone()).map_err(gateway_rejection)?;

    let cursor_key = read_cursor_key(cursor_key_fd)?;
    let service_keys = crate::remote_service_secrets::resolve_remote_service_keys_bounded(&secret_request)?;
    let response = dispatch_gateway_api_request(request, state_dir, &cursor_key, &service_keys.result_signing_key)?;
    print_dispatch_response(&response)
}

// r[impl remote_builds.versioned_build_api]
pub fn dispatch_gateway_api_request(
    request: RemoteGatewayDispatchRequest,
    state_dir: &Path,
    cursor_key: &[u8; GATEWAY_CURSOR_KEY_BYTES],
    signing_key: &crunch_build::KeyPair,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    validate_dispatch_message_bound(&request)?;
    let plan = plan_gateway_api_request(request.api.clone()).map_err(gateway_rejection)?;
    let _coordinator_guard = acquire_remote_coordinator_mutation_guard(state_dir).map_err(RunError::Internal)?;
    let mut coordinator = load_coordinator_state(state_dir)?;
    let mut state = load_gateway_state(state_dir)?;
    let response = match &plan.command {
        GatewayCommand::SubmitRemoteAttempt { .. } => {
            dispatch_submit(&request, &plan, &mut coordinator, &mut state, state_dir)?
        }
        GatewayCommand::ReadAttemptStatus { attempt_id } => {
            dispatch_status(&request.api, &plan, attempt_id, &coordinator, &mut state, signing_key)?
        }
        GatewayCommand::ReadAttemptLogs {
            attempt_id,
            cursor,
            byte_count,
        } => dispatch_logs(
            &request.api,
            &plan,
            attempt_id,
            cursor.as_deref(),
            *byte_count,
            &coordinator,
            &state,
            cursor_key,
        )?,
        GatewayCommand::ReadAttemptEvents {
            attempt_id,
            cursor,
            item_count,
        } => dispatch_events(
            &request.api,
            &plan,
            attempt_id,
            cursor.as_deref(),
            *item_count,
            &coordinator,
            &mut state,
            cursor_key,
            signing_key,
        )?,
        GatewayCommand::CancelOwnedAttempt { attempt_id } => {
            dispatch_cancel(&request.api, &plan, attempt_id, &mut coordinator, &mut state, signing_key)?
        }
        GatewayCommand::DiscoverSignedResult { attempt_id } => {
            dispatch_result(&request.api, &plan, attempt_id, &coordinator, &mut state, signing_key)?
        }
        GatewayCommand::ReadUsageSummary {
            account_scope,
            item_count,
        } => dispatch_usage(&request.api, &plan, account_scope, *item_count, &coordinator, &state)?,
        _ => {
            return Err(RunError::Internal(
                "gateway-api-operation-belongs-to-nix-store-or-administration-adapter".to_string(),
            ));
        }
    };
    save_coordinator_state(state_dir, &coordinator)?;
    save_gateway_state(state_dir, &state)?;
    Ok(response)
}

fn dispatch_submit(
    request: &RemoteGatewayDispatchRequest,
    plan: &RemoteGatewayApiPlan,
    coordinator: &mut RemoteCoordinatorState,
    state: &mut GatewayAdapterState,
    state_dir: &Path,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let GatewayCommand::SubmitRemoteAttempt {
        attempt_id,
        request_identity_blake3,
        idempotency_key,
        concrete_derivation_path,
    } = &plan.command
    else {
        return Err(RunError::Internal("gateway-submit-command-mismatch".to_string()));
    };
    let existing = existing_submission_for(
        state,
        &request.api.authority.subject,
        &request.api.authority.account_scope,
        idempotency_key,
    );
    if existing.is_none()
        && u32::try_from(state.submissions.len()).unwrap_or(u32::MAX) >= GATEWAY_ADAPTER_SUBMISSIONS_MAX
    {
        return Err(RunError::Internal("gateway-submission-capacity-exhausted".to_string()));
    }
    let decision = decide_gateway_submission(
        &plan.command,
        &request.api.authority,
        existing.map(GatewayStoredSubmission::existing).as_ref(),
    )
    .map_err(gateway_rejection)?;
    if let GatewaySubmissionDecision::Recover { .. } = decision {
        let stored = existing.ok_or_else(|| RunError::Internal("gateway-idempotency-state-missing".to_string()))?;
        return submitted_response(request, plan, stored, true);
    }
    let coordinator_request = request
        .coordinator_request
        .as_ref()
        .ok_or_else(|| RunError::Internal("gateway-submit-coordinator-request-missing".to_string()))?;
    validate_submit_linkage(
        coordinator_request,
        request_identity_blake3,
        concrete_derivation_path,
        &request.api.operation,
    )?;
    validate_coordinator_build_request(coordinator_request).map_err(RunError::Internal)?;
    let retry_policy = request.retry_policy.unwrap_or_default();
    let deadline = request
        .api
        .now_unix_s
        .checked_add(coordinator_request.request.build_time_limit_secs)
        .ok_or_else(|| RunError::Internal("gateway-submit-deadline-overflow".to_string()))?;
    let dispatch = admit_coordinator_dispatch(
        coordinator,
        coordinator_request,
        retry_policy,
        crunch_build::distributed::RemoteAttemptTimeFacts {
            now_unix_s: request.api.now_unix_s,
            failure_observed_unix_s: request.api.now_unix_s,
            overall_deadline_unix_s: deadline,
        },
    )
    .map_err(RunError::Internal)?;
    let binding = dispatch_binding(&dispatch, coordinator)?;
    let stored = GatewayStoredSubmission {
        public_attempt_id: attempt_id.clone(),
        subject: request.api.authority.subject.clone(),
        account_scope: request.api.authority.account_scope.clone(),
        idempotency_key: idempotency_key.clone(),
        request_identity_blake3: request_identity_blake3.clone(),
        policy_identity_blake3: request.api.policy.policy_identity_blake3.clone(),
        internal_job_id: binding.job_id.clone(),
        internal_binding: binding,
    };
    state.submissions.insert(attempt_id.as_str().to_string(), stored.clone());
    save_coordinator_state(state_dir, coordinator)?;
    submitted_response(request, plan, &stored, false)
}

fn existing_submission_for<'a>(
    state: &'a GatewayAdapterState,
    subject: &str,
    account_scope: &str,
    idempotency_key: &str,
) -> Option<&'a GatewayStoredSubmission> {
    state.submissions.values().find(|stored| {
        stored.subject == subject && stored.account_scope == account_scope && stored.idempotency_key == idempotency_key
    })
}

fn submitted_response(
    request: &RemoteGatewayDispatchRequest,
    plan: &RemoteGatewayApiPlan,
    stored: &GatewayStoredSubmission,
    recovered: bool,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let outcome = if recovered {
        "recovered-existing-attempt"
    } else {
        "durable-attempt-admitted"
    };
    let evidence = response_evidence(&request.api, plan, Some(&stored.public_attempt_id), 0, outcome)?;
    Ok(RemoteGatewayDispatchResponse::Submitted {
        public_attempt_id: stored.public_attempt_id.as_str().to_string(),
        recovered,
        evidence,
        non_claim: GATEWAY_API_NON_CLAIM.to_string(),
    })
}

fn validate_submit_linkage(
    coordinator_request: &RemoteCoordinatorBuildRequest,
    request_identity_blake3: &str,
    concrete_derivation_path: &str,
    operation: &GatewayOperation,
) -> Result<(), RunError> {
    if coordinator_request.request.request_id != request_identity_blake3 {
        return Err(RunError::Internal("gateway-submit-request-identity-mismatch".to_string()));
    }
    if coordinator_request.request.contains_raw_frontend_eval {
        return Err(RunError::Internal("gateway-submit-raw-frontend-eval-forbidden".to_string()));
    }
    let RemoteConcreteBuildPayload::Derivation { drv_path, .. } = &coordinator_request.request.payload else {
        return Err(RunError::Internal("gateway-submit-concrete-derivation-required".to_string()));
    };
    if drv_path != concrete_derivation_path {
        return Err(RunError::Internal("gateway-submit-derivation-path-mismatch".to_string()));
    }
    let GatewayOperation::SubmitBuild {
        input_paths,
        expected_output_paths,
        ..
    } = operation
    else {
        return Err(RunError::Internal("gateway-submit-operation-shape-invalid".to_string()));
    };
    let mut actual_inputs = coordinator_request.request.input_refs.clone();
    actual_inputs.sort();
    actual_inputs.dedup();
    let mut actual_outputs = coordinator_request
        .request
        .expected_outputs
        .iter()
        .map(|output| output.logical_path.clone())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| RunError::Internal("gateway-submit-output-path-missing".to_string()))?;
    actual_outputs.sort();
    actual_outputs.dedup();
    if &actual_inputs != input_paths || &actual_outputs != expected_output_paths {
        return Err(RunError::Internal("gateway-submit-input-or-output-linkage-mismatch".to_string()));
    }
    Ok(())
}

fn dispatch_binding(
    decision: &RemoteCoordinatorDispatchDecision,
    coordinator: &RemoteCoordinatorState,
) -> Result<RemoteProductionAttemptBinding, RunError> {
    match decision {
        RemoteCoordinatorDispatchDecision::Dispatch {
            job_id,
            attempt_id,
            fence_generation,
            ..
        } => Ok(RemoteProductionAttemptBinding {
            job_id: job_id.clone(),
            attempt_id: attempt_id.clone(),
            fence_generation: *fence_generation,
        }),
        RemoteCoordinatorDispatchDecision::AttachExisting { job_id, .. }
        | RemoteCoordinatorDispatchDecision::RedeliverResult { job_id, .. } => binding_from_job(coordinator, job_id),
        RemoteCoordinatorDispatchDecision::Pending { reason }
        | RemoteCoordinatorDispatchDecision::Reject { reason } => {
            Err(RunError::Internal(format!("gateway-submit-not-admitted:{reason}")))
        }
    }
}

fn binding_from_job(
    coordinator: &RemoteCoordinatorState,
    job_id: &crunch_build::distributed::RemoteJobId,
) -> Result<RemoteProductionAttemptBinding, RunError> {
    let attempt = coordinator
        .jobs
        .get(job_id)
        .and_then(|job| job.current_attempt.as_ref())
        .ok_or_else(|| RunError::Internal("gateway-internal-attempt-missing".to_string()))?;
    Ok(RemoteProductionAttemptBinding {
        job_id: job_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        fence_generation: attempt.fence_generation,
    })
}

fn dispatch_status(
    api: &RemoteGatewayApiRequest,
    plan: &RemoteGatewayApiPlan,
    attempt_id: &RemoteAttemptId,
    coordinator: &RemoteCoordinatorState,
    state: &mut GatewayAdapterState,
    signing_key: &crunch_build::KeyPair,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let stored = authorize_stored_attempt(api, attempt_id, state)?.clone();
    decide_gateway_reconnect(&api.authority, &GatewayReconnectFacts {
        attempt_id: stored.public_attempt_id.clone(),
        owner_subject: stored.subject.clone(),
        account_scope: stored.account_scope.clone(),
        exists: true,
    })
    .map_err(gateway_rejection)?;
    let completion = reconcile_completion(&stored, coordinator, state, signing_key)?;
    let status = attempt_status(&stored, coordinator, completion)?;
    let evidence = response_evidence(api, plan, Some(attempt_id), 0, "status-observed")?;
    Ok(RemoteGatewayDispatchResponse::Status {
        status,
        evidence,
        non_claim: GATEWAY_API_NON_CLAIM.to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
fn dispatch_logs(
    api: &RemoteGatewayApiRequest,
    plan: &RemoteGatewayApiPlan,
    attempt_id: &RemoteAttemptId,
    cursor: Option<&str>,
    byte_count: u32,
    coordinator: &RemoteCoordinatorState,
    state: &GatewayAdapterState,
    cursor_key: &[u8; GATEWAY_CURSOR_KEY_BYTES],
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let stored = authorize_stored_attempt(api, attempt_id, state)?;
    let from_cursor = cursor_sequence(api, stored, cursor, "logs", cursor_key)?;
    let replay = replay_coordinator_attempt_log(coordinator, &stored.internal_job_id, RemoteAttemptLogReplayRequest {
        from_cursor,
        record_count_max: GATEWAY_LOG_RECORDS_MAX,
        payload_bytes_max: u64::from(byte_count),
    });
    let (records, next_sequence, has_more) = match replay {
        Ok(replay) => (replay.records, replay.next_cursor, replay.has_more),
        Err(error) if error == "attempt-log-not-recorded" => (Vec::new(), 0, false),
        Err(error) => return Err(RunError::Internal(format!("gateway-log-replay:{error}"))),
    };
    let next_cursor = next_cursor(api, stored, next_sequence, has_more, "logs", cursor_key)?;
    let transferred_bytes = records
        .iter()
        .fold(0_u64, |total, record| total.saturating_add(u64::from(record.payload_length_bytes)));
    let evidence = response_evidence(api, plan, Some(attempt_id), transferred_bytes, "bounded-log-window")?;
    Ok(RemoteGatewayDispatchResponse::Logs {
        public_attempt_id: attempt_id.as_str().to_string(),
        records,
        next_cursor,
        has_more,
        evidence,
        non_claim: GATEWAY_API_NON_CLAIM.to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
fn dispatch_events(
    api: &RemoteGatewayApiRequest,
    plan: &RemoteGatewayApiPlan,
    attempt_id: &RemoteAttemptId,
    cursor: Option<&str>,
    item_count: u32,
    coordinator: &RemoteCoordinatorState,
    state: &mut GatewayAdapterState,
    cursor_key: &[u8; GATEWAY_CURSOR_KEY_BYTES],
    signing_key: &crunch_build::KeyPair,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let stored = authorize_stored_attempt(api, attempt_id, state)?.clone();
    let start = cursor_sequence(api, &stored, cursor, "events", cursor_key)?;
    let job = coordinator_job(&stored, coordinator)?;
    let mut events = job
        .current_attempt
        .as_ref()
        .map(|attempt| {
            attempt
                .applied_events
                .iter()
                .map(|(event_id, digest)| GatewayObservedEvent::Attempt {
                    event_id: event_id.as_str().to_string(),
                    payload_digest_blake3: digest.as_str().to_string(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(signed) = reconcile_completion(&stored, coordinator, state, signing_key)? {
        events.push(GatewayObservedEvent::Completion { signed });
    }
    let start_index = usize::try_from(start).unwrap_or(usize::MAX).min(events.len());
    let item_count = usize::try_from(item_count).unwrap_or(usize::MAX);
    let end = start_index.saturating_add(item_count).min(events.len());
    let page = events[start_index..end].to_vec();
    let has_more = end < events.len();
    let next_sequence = u64::try_from(end).unwrap_or(u64::MAX);
    let next_cursor = next_cursor(api, &stored, next_sequence, has_more, "events", cursor_key)?;
    let evidence = response_evidence(api, plan, Some(attempt_id), 0, "bounded-event-page")?;
    Ok(RemoteGatewayDispatchResponse::Events {
        public_attempt_id: attempt_id.as_str().to_string(),
        events: page,
        next_cursor,
        has_more,
        evidence,
        non_claim: GATEWAY_API_NON_CLAIM.to_string(),
    })
}

fn dispatch_cancel(
    api: &RemoteGatewayApiRequest,
    plan: &RemoteGatewayApiPlan,
    attempt_id: &RemoteAttemptId,
    coordinator: &mut RemoteCoordinatorState,
    state: &mut GatewayAdapterState,
    signing_key: &crunch_build::KeyPair,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let stored = authorize_stored_attempt(api, attempt_id, state)?.clone();
    let cancellation_applied = matches!(
        coordinator_job(&stored, coordinator)?.phase,
        RemoteCoordinatorJobPhase::Queued | RemoteCoordinatorJobPhase::Running
    );
    if cancellation_applied {
        let current_binding = current_coordinator_binding(&stored, coordinator)?;
        terminate_coordinator_attempt(coordinator, &current_binding, RemoteCoordinatorTerminationCause::Cancellation)
            .map_err(RunError::Internal)?;
    }
    let completion = reconcile_completion(&stored, coordinator, state, signing_key)?
        .ok_or_else(|| RunError::Internal("gateway-cancellation-completion-missing".to_string()))?;
    let outcome = if cancellation_applied {
        "owned-attempt-cancelled"
    } else {
        "terminal-attempt-cancellation-not-applied"
    };
    let evidence = response_evidence(api, plan, Some(attempt_id), 0, outcome)?;
    Ok(RemoteGatewayDispatchResponse::Cancellation {
        public_attempt_id: attempt_id.as_str().to_string(),
        cancellation_applied,
        completion,
        evidence,
        non_claim: GATEWAY_API_NON_CLAIM.to_string(),
    })
}

fn dispatch_result(
    api: &RemoteGatewayApiRequest,
    plan: &RemoteGatewayApiPlan,
    attempt_id: &RemoteAttemptId,
    coordinator: &RemoteCoordinatorState,
    state: &mut GatewayAdapterState,
    signing_key: &crunch_build::KeyPair,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let stored = authorize_stored_attempt(api, attempt_id, state)?.clone();
    let completion = reconcile_completion(&stored, coordinator, state, signing_key)?;
    let result_identity_blake3 = coordinator_job(&stored, coordinator)?
        .current_attempt
        .as_ref()
        .and_then(|attempt| attempt.result_digest_blake3.clone());
    let evidence = response_evidence(api, plan, Some(attempt_id), 0, "signed-result-observed")?;
    Ok(RemoteGatewayDispatchResponse::SignedResult {
        public_attempt_id: attempt_id.as_str().to_string(),
        result_identity_blake3,
        completion,
        evidence,
        non_claim: GATEWAY_API_NON_CLAIM.to_string(),
    })
}

fn dispatch_usage(
    api: &RemoteGatewayApiRequest,
    plan: &RemoteGatewayApiPlan,
    account_scope: &str,
    item_count: u32,
    coordinator: &RemoteCoordinatorState,
    state: &GatewayAdapterState,
) -> Result<RemoteGatewayDispatchResponse, RunError> {
    let item_limit = usize::try_from(item_count).unwrap_or(usize::MAX);
    let mut attempts = Vec::with_capacity(item_limit.min(state.submissions.len()));
    for stored in state.submissions.values().filter(|stored| stored.account_scope == account_scope).take(item_limit) {
        let job = coordinator_job(stored, coordinator)?;
        attempts.push(GatewayUsageItem {
            public_attempt_id: stored.public_attempt_id.as_str().to_string(),
            phase: job.phase,
            result_available: job.result_available,
        });
    }
    let evidence = response_evidence(api, plan, None, 0, "bounded-usage-summary")?;
    Ok(RemoteGatewayDispatchResponse::Usage {
        account_scope: account_scope.to_string(),
        attempts,
        evidence,
        non_claim: GATEWAY_API_NON_CLAIM.to_string(),
    })
}

fn authorize_stored_attempt<'a>(
    api: &RemoteGatewayApiRequest,
    attempt_id: &RemoteAttemptId,
    state: &'a GatewayAdapterState,
) -> Result<&'a GatewayStoredSubmission, RunError> {
    let stored = state
        .submissions
        .get(attempt_id.as_str())
        .ok_or_else(|| RunError::Internal("gateway-public-attempt-unknown".to_string()))?;
    if stored.subject != api.authority.subject || stored.account_scope != api.authority.account_scope {
        return Err(RunError::Internal("gateway-attempt-owner-or-scope-mismatch".to_string()));
    }
    Ok(stored)
}

fn coordinator_job<'a>(
    stored: &GatewayStoredSubmission,
    coordinator: &'a RemoteCoordinatorState,
) -> Result<&'a crate::remote_build::RemoteCoordinatorJobSummary, RunError> {
    coordinator
        .jobs
        .get(&stored.internal_job_id)
        .ok_or_else(|| RunError::Internal("gateway-internal-job-unknown".to_string()))
}

fn current_coordinator_binding(
    stored: &GatewayStoredSubmission,
    coordinator: &RemoteCoordinatorState,
) -> Result<RemoteProductionAttemptBinding, RunError> {
    binding_from_job(coordinator, &stored.internal_job_id)
}

fn attempt_status(
    stored: &GatewayStoredSubmission,
    coordinator: &RemoteCoordinatorState,
    completion: Option<SignedGatewayCompletionEvent>,
) -> Result<GatewayAttemptStatus, RunError> {
    let job = coordinator_job(stored, coordinator)?;
    Ok(GatewayAttemptStatus {
        public_attempt_id: stored.public_attempt_id.as_str().to_string(),
        phase: job.phase,
        result_available: job.result_available,
        transferred_bytes: job.transferred_bytes,
        short_error: job.short_error.as_deref().map(crate::remote_build::bounded_untrusted_text),
        completion,
    })
}

fn cursor_sequence(
    api: &RemoteGatewayApiRequest,
    stored: &GatewayStoredSubmission,
    cursor: Option<&str>,
    stream: &str,
    cursor_key: &[u8; GATEWAY_CURSOR_KEY_BYTES],
) -> Result<u64, RunError> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    verify_gateway_cursor(
        cursor,
        crunch_build::distributed::GatewayCursorScope {
            subject: &api.authority.subject,
            account_scope: &api.authority.account_scope,
            attempt_id: &stored.public_attempt_id,
            stream,
        },
        api.now_unix_s,
        cursor_key,
    )
    .map(|cursor| cursor.next_sequence)
    .map_err(gateway_rejection)
}

fn next_cursor(
    api: &RemoteGatewayApiRequest,
    stored: &GatewayStoredSubmission,
    next_sequence: u64,
    has_more: bool,
    stream: &str,
    cursor_key: &[u8; GATEWAY_CURSOR_KEY_BYTES],
) -> Result<Option<String>, RunError> {
    if !has_more {
        return Ok(None);
    }
    let expires = api
        .now_unix_s
        .checked_add(GATEWAY_CURSOR_TTL_SECS)
        .ok_or_else(|| RunError::Internal("gateway-cursor-expiry-overflow".to_string()))?;
    let cursor = seal_gateway_cursor(
        crunch_build::distributed::GatewayCursorScope {
            subject: &api.authority.subject,
            account_scope: &api.authority.account_scope,
            attempt_id: &stored.public_attempt_id,
            stream,
        },
        next_sequence,
        expires,
        cursor_key,
    )
    .map_err(gateway_rejection)?;
    if u32::try_from(cursor.len()).unwrap_or(u32::MAX) > api.policy.bounds.cursor_bytes_max {
        return Err(RunError::Internal("gateway-cursor-selected-bound-exceeded".to_string()));
    }
    Ok(Some(cursor))
}

fn reconcile_completion(
    stored: &GatewayStoredSubmission,
    coordinator: &RemoteCoordinatorState,
    state: &mut GatewayAdapterState,
    signing_key: &crunch_build::KeyPair,
) -> Result<Option<SignedGatewayCompletionEvent>, RunError> {
    if let Some(existing) = state.completions_by_attempt.get(stored.public_attempt_id.as_str()) {
        return Ok(Some(existing.clone()));
    }
    let job = coordinator_job(stored, coordinator)?;
    let terminal_class = match job.phase {
        RemoteCoordinatorJobPhase::Finished => GatewayTerminalClass::Completed,
        RemoteCoordinatorJobPhase::Lost if job.short_error.as_deref() == Some("remote-attempt-cancelled") => {
            GatewayTerminalClass::Cancelled
        }
        RemoteCoordinatorJobPhase::Lost => GatewayTerminalClass::Failed,
        RemoteCoordinatorJobPhase::Queued | RemoteCoordinatorJobPhase::Running => return Ok(None),
    };
    let result = job.current_attempt.as_ref().and_then(|attempt| attempt.result_digest_blake3.clone());
    let event = plan_gateway_completion_event(
        stored.public_attempt_id.clone(),
        stored.request_identity_blake3.clone(),
        terminal_class,
        result,
        stored.policy_identity_blake3.clone(),
        state.next_completion_sequence,
    )
    .map_err(gateway_rejection)?;
    let signed = sign_completion_event(event, signing_key)?;
    state.next_completion_sequence = state
        .next_completion_sequence
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("gateway-completion-sequence-overflow".to_string()))?;
    state.completions_by_attempt.insert(stored.public_attempt_id.as_str().to_string(), signed.clone());
    Ok(Some(signed))
}

fn sign_completion_event(
    event: GatewayCompletionEvent,
    signing_key: &crunch_build::KeyPair,
) -> Result<SignedGatewayCompletionEvent, RunError> {
    let mut material = Vec::with_capacity(GATEWAY_COMPLETION_SIGNATURE_DOMAIN.len().saturating_add(64));
    material.extend_from_slice(GATEWAY_COMPLETION_SIGNATURE_DOMAIN);
    material.extend_from_slice(event.event_identity_blake3.as_bytes());
    let signature = signing_key.signing_key.sign(&material).to_string();
    bind_gateway_completion_signature(event, signing_key.verifying_key.name().to_string(), signature)
        .map_err(gateway_rejection)
}

fn response_evidence(
    api: &RemoteGatewayApiRequest,
    plan: &RemoteGatewayApiPlan,
    attempt_id: Option<&RemoteAttemptId>,
    transferred_bytes: u64,
    outcome: &str,
) -> Result<GatewayEvidence, RunError> {
    let evidence = gateway_evidence(
        &api.policy,
        &api.authority,
        &api.operation,
        attempt_id,
        transferred_bytes,
        outcome.to_string(),
    )
    .map_err(gateway_rejection)?;
    if evidence.operation_identity_blake3 != plan.operation_identity_blake3 {
        return Err(RunError::Internal("gateway-response-operation-identity-mismatch".to_string()));
    }
    Ok(evidence)
}

fn validate_dispatch_message_bound(request: &RemoteGatewayDispatchRequest) -> Result<(), RunError> {
    let bytes = serde_json::to_vec(request)
        .map_err(|error| RunError::Internal(format!("serializing gateway dispatch request: {error}")))?;
    let byte_count = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    if byte_count == 0 || byte_count > request.api.policy.bounds.message_bytes_max {
        return Err(RunError::Internal("gateway-dispatch-message-bound-exceeded".to_string()));
    }
    Ok(())
}

fn validate_gateway_state(state: &GatewayAdapterState) -> Result<(), RunError> {
    if state.schema != GATEWAY_ADAPTER_STATE_SCHEMA || state.next_completion_sequence == 0 {
        return Err(RunError::Internal("gateway-adapter-state-header-invalid".to_string()));
    }
    let submission_count = u32::try_from(state.submissions.len()).unwrap_or(u32::MAX);
    if submission_count > GATEWAY_ADAPTER_SUBMISSIONS_MAX
        || state.completions_by_attempt.len() > state.submissions.len()
    {
        return Err(RunError::Internal("gateway-adapter-state-count-invalid".to_string()));
    }
    let mut idempotency = BTreeSet::new();
    for (public_id, stored) in &state.submissions {
        let expected_public_id = derive_gateway_attempt_id(
            &stored.subject,
            &stored.account_scope,
            &stored.idempotency_key,
            &stored.request_identity_blake3,
        )
        .map_err(gateway_rejection)?;
        if public_id != stored.public_attempt_id.as_str()
            || expected_public_id != stored.public_attempt_id
            || stored.internal_binding.job_id != stored.internal_job_id
            || !is_blake3_hex(&stored.request_identity_blake3)
            || !is_blake3_hex(&stored.policy_identity_blake3)
            || !idempotency.insert((
                stored.subject.as_str(),
                stored.account_scope.as_str(),
                stored.idempotency_key.as_str(),
            ))
        {
            return Err(RunError::Internal("gateway-adapter-state-linkage-invalid".to_string()));
        }
    }
    let mut completion_sequences = BTreeSet::new();
    let mut completion_identities = BTreeSet::new();
    for (attempt_id, completion) in &state.completions_by_attempt {
        let Some(stored) = state.submissions.get(attempt_id) else {
            return Err(RunError::Internal("gateway-adapter-completion-linkage-invalid".to_string()));
        };
        let expected = plan_gateway_completion_event(
            stored.public_attempt_id.clone(),
            stored.request_identity_blake3.clone(),
            completion.event.terminal_class,
            completion.event.result_evidence_identity_blake3.clone(),
            stored.policy_identity_blake3.clone(),
            completion.event.sequence,
        )
        .map_err(gateway_rejection)?;
        let parsed_signature = Signature::<String>::parse(&completion.signature)
            .map_err(|_| RunError::Internal("gateway-adapter-completion-signature-invalid".to_string()))?;
        if expected != completion.event
            || parsed_signature.name().as_str() != completion.producer_key_id.as_str()
            || completion.event.sequence >= state.next_completion_sequence
            || !completion_sequences.insert(completion.event.sequence)
            || !completion_identities.insert(completion.event.event_identity_blake3.as_str())
            || bind_gateway_completion_signature(
                expected,
                completion.producer_key_id.clone(),
                completion.signature.clone(),
            )
            .is_err()
        {
            return Err(RunError::Internal("gateway-adapter-completion-linkage-invalid".to_string()));
        }
    }
    Ok(())
}

fn load_gateway_state(state_dir: &Path) -> Result<GatewayAdapterState, RunError> {
    let path = state_dir.join(GATEWAY_ADAPTER_STATE_FILE);
    if !path.exists() {
        return Ok(GatewayAdapterState::default());
    }
    let file = open_gateway_state_no_follow(&path)?;
    if !file
        .metadata()
        .map_err(|error| RunError::Internal(format!("reading gateway state metadata: {error}")))?
        .is_file()
    {
        return Err(RunError::Internal("gateway-adapter-state-not-regular".to_string()));
    }
    let mut bytes = Vec::with_capacity(8_192);
    file.take(GATEWAY_ADAPTER_STATE_BYTES_MAX.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading gateway state {}: {error}", path.display())))?;
    if bytes.is_empty() || u64::try_from(bytes.len()).unwrap_or(u64::MAX) > GATEWAY_ADAPTER_STATE_BYTES_MAX {
        return Err(RunError::Internal("gateway-adapter-state-size-invalid".to_string()));
    }
    let state = serde_json::from_slice::<GatewayAdapterState>(&bytes)
        .map_err(|error| RunError::Internal(format!("parsing gateway state: {error}")))?;
    validate_gateway_state(&state)?;
    Ok(state)
}

#[cfg(unix)]
fn open_gateway_state_no_follow(path: &Path) -> Result<fs::File, RunError> {
    use std::os::unix::fs::OpenOptionsExt as _;

    fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|error| RunError::Internal(format!("opening gateway state {}: {error}", path.display())))
}

#[cfg(not(unix))]
fn open_gateway_state_no_follow(path: &Path) -> Result<fs::File, RunError> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(RunError::Internal("gateway-adapter-state-symlink-rejected".to_string()));
    }
    fs::File::open(path)
        .map_err(|error| RunError::Internal(format!("opening gateway state {}: {error}", path.display())))
}

fn save_gateway_state(state_dir: &Path, state: &GatewayAdapterState) -> Result<(), RunError> {
    validate_gateway_state(state)?;
    fs::create_dir_all(state_dir)
        .map_err(|error| RunError::Internal(format!("creating gateway state dir: {error}")))?;
    let path = state_dir.join(GATEWAY_ADAPTER_STATE_FILE);
    let temp = state_dir.join(format!("{GATEWAY_ADAPTER_STATE_FILE}.tmp"));
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|error| RunError::Internal(format!("serializing gateway state: {error}")))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > GATEWAY_ADAPTER_STATE_BYTES_MAX {
        return Err(RunError::Internal("gateway-adapter-state-size-invalid".to_string()));
    }
    write_private_file(&temp, &bytes)?;
    fs::rename(&temp, &path)
        .map_err(|error| RunError::Internal(format!("publishing gateway state {}: {error}", path.display())))?;
    sync_parent(state_dir)?;
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| RunError::Internal(format!("removing stale gateway temp state: {error}")))?;
    }
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(OWNER_ONLY_FILE_MODE);
    }
    let mut file = options
        .open(path)
        .map_err(|error| RunError::Internal(format!("opening private gateway state: {error}")))?;
    file.write_all(bytes)
        .map_err(|error| RunError::Internal(format!("writing private gateway state: {error}")))?;
    file.sync_all()
        .map_err(|error| RunError::Internal(format!("syncing private gateway state: {error}")))
}

fn sync_parent(parent: &Path) -> Result<(), RunError> {
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| RunError::Internal(format!("syncing gateway state parent: {error}")))
}

fn read_dispatch_stdin() -> Result<RemoteGatewayDispatchRequest, RunError> {
    let mut bytes = Vec::with_capacity(65_536);
    std::io::stdin()
        .lock()
        .take(u64::from(MAX_GATEWAY_MESSAGE_BYTES).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading gateway dispatch stdin: {error}")))?;
    if bytes.is_empty() || u32::try_from(bytes.len()).unwrap_or(u32::MAX) > MAX_GATEWAY_MESSAGE_BYTES {
        return Err(RunError::Internal("gateway-dispatch-input-size-invalid".to_string()));
    }
    parse_dispatch_request(&bytes)
}

fn parse_dispatch_request(bytes: &[u8]) -> Result<RemoteGatewayDispatchRequest, RunError> {
    serde_json::from_slice(bytes).map_err(|_| RunError::Internal("gateway-dispatch-input-invalid".to_string()))
}

fn print_dispatch_response(response: &RemoteGatewayDispatchResponse) -> Result<(), RunError> {
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer_pretty(&mut output, response)
        .map_err(|error| RunError::Internal(format!("serializing gateway dispatch response: {error}")))?;
    output
        .write_all(b"\n")
        .map_err(|error| RunError::Internal(format!("writing gateway dispatch response: {error}")))
}

fn read_cursor_key(fd: i32) -> Result<Zeroizing<[u8; GATEWAY_CURSOR_KEY_BYTES]>, RunError> {
    if fd <= libc::STDERR_FILENO {
        return Err(RunError::Internal("gateway-cursor-key-fd-invalid".to_string()));
    }
    // SAFETY: ownership of the caller-declared descriptor transfers to this one-shot command.
    let file = unsafe { fs::File::from_raw_fd(fd) };
    if file.is_terminal() {
        return Err(RunError::Internal("gateway-cursor-key-terminal-forbidden".to_string()));
    }
    let mut bytes = Zeroizing::new(Vec::with_capacity(GATEWAY_CURSOR_KEY_BYTES.saturating_add(1)));
    file.take(GATEWAY_CURSOR_KEY_READ_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading gateway cursor key: {error}")))?;
    if bytes.len() != GATEWAY_CURSOR_KEY_BYTES {
        return Err(RunError::Internal("gateway-cursor-key-length-invalid".to_string()));
    }
    let mut key = Zeroizing::new([0_u8; GATEWAY_CURSOR_KEY_BYTES]);
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_trusted_authority(
    presented: &crunch_build::distributed::RemoteGatewayAuthority,
    trusted: &crunch_build::distributed::RemoteGatewayAuthority,
) -> Result<(), RunError> {
    if presented != trusted {
        return Err(RunError::Internal("gateway-authority-context-mismatch".to_string()));
    }
    Ok(())
}

fn gateway_rejection(error: crunch_build::distributed::GatewayRejection) -> RunError {
    RunError::Internal(format!("gateway dispatch {:?}: {}", error.code, error.message))
}

#[cfg(test)]
mod tests {
    use crunch_build::distributed::GATEWAY_API_SCHEMA;
    use crunch_build::distributed::GatewayAuthoritySource;
    use crunch_build::distributed::GatewayCapability;
    use crunch_build::distributed::GatewayPolicy;
    use crunch_build::distributed::RemoteGatewayAuthority;

    use super::*;
    use crate::remote_gateway::RemoteGatewayApiRequest;

    const NOW: u64 = 1_800_000_000;

    fn digest(label: &str) -> String {
        blake3::hash(label.as_bytes()).to_hex().to_string()
    }

    fn api(operation: GatewayOperation) -> RemoteGatewayApiRequest {
        api_with_capability(operation, GatewayCapability::ReadStatus)
    }

    fn api_with_capability(operation: GatewayOperation, capability: GatewayCapability) -> RemoteGatewayApiRequest {
        RemoteGatewayApiRequest {
            schema: GATEWAY_API_SCHEMA.into(),
            api_version: 1,
            nix_protocol_major: None,
            nix_protocol_minor: None,
            now_unix_s: NOW,
            authority: RemoteGatewayAuthority {
                source: GatewayAuthoritySource::VerifiedUcan,
                subject: "subject-a".into(),
                account_scope: "project-a".into(),
                audience: "mantle-remote-gateway".into(),
                expires_unix_s: NOW + 600,
                capabilities: vec![capability],
                evidence_refs_blake3: vec![digest("authority")],
            },
            policy: GatewayPolicy::default(),
            operation,
        }
    }

    fn signing_key() -> crunch_build::KeyPair {
        crunch_build::load_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        )
        .unwrap()
    }

    fn coordinator_request() -> crate::remote_build::RemoteCoordinatorBuildRequest {
        let drv_json = serde_json::json!({
            "name": "gateway-fixture",
            "builder": "/bin/sh",
            "args": ["-c", "echo hi"],
            "outputs": ["out"],
            "addressing_mode": "input-addressed"
        })
        .to_string();
        let drv: crunch_glue::CrunchDerivation = serde_json::from_str(&drv_json).unwrap();
        let mut known_paths = crunch_glue::ConversionCache::new("/mantle/store");
        let (drv_path, nix_drv) = crunch_glue::convert(&drv, &mut known_paths).unwrap();
        let output_path = nix_drv.outputs["out"].path.as_ref().unwrap().to_absolute_path_with_prefix("/mantle/store");
        let request_identity = digest("gateway-request");
        crate::remote_build::RemoteCoordinatorBuildRequest {
            request: crate::remote_build::ConcreteBuildRequest {
                request_id: request_identity,
                store_prefix: "/mantle/store".into(),
                input_refs: vec!["/mantle/store/11111111111111111111111111111111-input".into()],
                source_input_refs: Vec::new(),
                upload_bytes: 1,
                build_time_limit_secs: 60,
                contains_raw_frontend_eval: false,
                payload: crate::remote_build::RemoteConcreteBuildPayload::Derivation {
                    drv_path: drv_path.to_absolute_path_with_prefix("/mantle/store"),
                    drv_json,
                },
                expected_outputs: vec![crate::remote_build::RemoteExpectedOutput {
                    name: "out".into(),
                    logical_path: Some(output_path.clone()),
                }],
                production_attempt: None,
                transfer_policy: None,
                resource_requirements: None,
                locality_scope: None,
                failure_debug_policy: crunch_build::distributed::RemoteFailureDebugPolicy::default(),
                failure_replay: None,
            },
            required_system: "x86_64-linux".into(),
            required_features: Vec::new(),
            required_sandbox_mode: "native".into(),
            required_network_mode: "none".into(),
            resource_requirements: None,
            locality_scope: None,
            trusted_output_keys: vec!["builder-key".into()],
            live_output_claims: vec![output_path],
            wait_for_worker: false,
        }
    }

    fn install_worker(state_dir: &Path) {
        let mut coordinator = RemoteCoordinatorState::default();
        crate::remote_build::apply_worker_registration(
            &mut coordinator,
            crate::remote_build::RemoteWorkerRegistration {
                endpoint_id: "builder-1".into(),
                protocol_version: crate::remote_build::REMOTE_PROTOCOL_VERSION,
                worker_generation: 1,
                systems: vec!["x86_64-linux".into()],
                feature_labels: Vec::new(),
                sandbox_modes: vec!["native".into()],
                network_modes: vec!["none".into()],
                logical_store_prefixes: vec!["/mantle/store".into()],
                concurrency: 1,
                resource_inventory: None,
                output_signing_key_ids: vec!["builder-key".into()],
                resumable_jobs: Vec::new(),
                workspace_policy: None,
            },
        )
        .unwrap();
        save_coordinator_state(state_dir, &coordinator).unwrap();
    }

    fn submit_request() -> RemoteGatewayDispatchRequest {
        let coordinator_request = coordinator_request();
        let crate::remote_build::RemoteConcreteBuildPayload::Derivation { drv_path, .. } =
            &coordinator_request.request.payload
        else {
            unreachable!()
        };
        RemoteGatewayDispatchRequest {
            api: api_with_capability(
                GatewayOperation::SubmitBuild {
                    request_identity_blake3: coordinator_request.request.request_id.clone(),
                    idempotency_key: "submission-a".into(),
                    concrete_derivation_path: drv_path.clone(),
                    input_paths: coordinator_request.request.input_refs.clone(),
                    expected_output_paths: coordinator_request.live_output_claims.clone(),
                },
                GatewayCapability::SubmitBuild,
            ),
            coordinator_request: Some(coordinator_request),
            retry_policy: None,
        }
    }

    // r[verify remote_builds.idempotent_completion_events]
    // r[verify remote_builds.gateway_bounds_and_recovery]
    #[test]
    fn durable_dispatch_recovers_duplicate_and_deduplicates_cancellation_completion() {
        let temp = tempfile::tempdir().unwrap();
        install_worker(temp.path());
        let key = signing_key();
        let cursor_key = [0x35_u8; GATEWAY_CURSOR_KEY_BYTES];
        let submission = submit_request();
        let admitted_policy_identity = submission.api.policy.policy_identity_blake3.clone();
        let submitted = dispatch_gateway_api_request(submission, temp.path(), &cursor_key, &key).unwrap();
        let public_attempt_id = match submitted {
            RemoteGatewayDispatchResponse::Submitted {
                public_attempt_id,
                recovered: false,
                ..
            } => public_attempt_id,
            other => panic!("unexpected first submission: {other:?}"),
        };
        let duplicate = dispatch_gateway_api_request(submit_request(), temp.path(), &cursor_key, &key).unwrap();
        assert!(matches!(duplicate, RemoteGatewayDispatchResponse::Submitted { recovered: true, .. }));
        assert_eq!(load_coordinator_state(temp.path()).unwrap().jobs.len(), 1);
        assert_eq!(load_gateway_state(temp.path()).unwrap().submissions.len(), 1);

        let status = RemoteGatewayDispatchRequest {
            api: api(GatewayOperation::ReadStatus {
                attempt_id: public_attempt_id.clone(),
            }),
            coordinator_request: None,
            retry_policy: None,
        };
        assert!(matches!(
            dispatch_gateway_api_request(status, temp.path(), &cursor_key, &key).unwrap(),
            RemoteGatewayDispatchResponse::Status {
                status: GatewayAttemptStatus {
                    phase: RemoteCoordinatorJobPhase::Queued,
                    ..
                },
                ..
            }
        ));

        let cancel = || {
            let mut request = RemoteGatewayDispatchRequest {
                api: api_with_capability(
                    GatewayOperation::CancelAttempt {
                        attempt_id: public_attempt_id.clone(),
                        owner_subject: "subject-a".into(),
                    },
                    GatewayCapability::CancelOwnedAttempt,
                ),
                coordinator_request: None,
                retry_policy: None,
            };
            request.api.policy.policy_identity_blake3 = digest("later-reconnect-policy");
            request
        };
        let first_completion = match dispatch_gateway_api_request(cancel(), temp.path(), &cursor_key, &key).unwrap() {
            RemoteGatewayDispatchResponse::Cancellation {
                cancellation_applied: true,
                completion,
                ..
            } => completion,
            other => panic!("unexpected first cancellation: {other:?}"),
        };
        let repeated_completion = match dispatch_gateway_api_request(cancel(), temp.path(), &cursor_key, &key).unwrap()
        {
            RemoteGatewayDispatchResponse::Cancellation {
                cancellation_applied: false,
                completion,
                ..
            } => completion,
            other => panic!("unexpected repeated cancellation: {other:?}"),
        };
        assert_eq!(first_completion, repeated_completion);
        assert_eq!(first_completion.event.policy_identity_blake3, admitted_policy_identity);
        assert_ne!(first_completion.event.policy_identity_blake3, digest("later-reconnect-policy"));
        assert_eq!(first_completion.event.sequence, 1);
        let persisted = load_gateway_state(temp.path()).unwrap();
        assert_eq!(persisted.completions_by_attempt.len(), 1);

        let mut tampered = persisted;
        tampered.completions_by_attempt.get_mut(&public_attempt_id).unwrap().event.policy_identity_blake3 =
            digest("tampered-policy");
        assert!(validate_gateway_state(&tampered).is_err());
    }

    #[test]
    fn verifier_context_mismatch_fails_closed() {
        let request = api(GatewayOperation::ReadStatus {
            attempt_id: "attempt-a".into(),
        });
        let mut other = request.authority.clone();
        other.subject = "subject-b".into();
        assert!(require_trusted_authority(&request.authority, &request.authority).is_ok());
        assert!(require_trusted_authority(&request.authority, &other).is_err());
    }

    fn stored(id: &str, idempotency_key: &str) -> GatewayStoredSubmission {
        GatewayStoredSubmission {
            public_attempt_id: RemoteAttemptId::new(id).unwrap(),
            subject: "subject-a".into(),
            account_scope: "project-a".into(),
            idempotency_key: idempotency_key.into(),
            request_identity_blake3: digest(id),
            policy_identity_blake3: GatewayPolicy::default().policy_identity_blake3,
            internal_job_id: crunch_build::distributed::RemoteJobId::new(format!("job-{id}")).unwrap(),
            internal_binding: RemoteProductionAttemptBinding {
                job_id: crunch_build::distributed::RemoteJobId::new(format!("job-{id}")).unwrap(),
                attempt_id: RemoteAttemptId::new(format!("internal-{id}")).unwrap(),
                fence_generation: crunch_build::distributed::RemoteFenceGeneration::INITIAL,
            },
        }
    }

    #[test]
    fn idempotency_lookup_uses_subject_scope_and_key_not_derived_attempt_id() {
        let mut state = GatewayAdapterState::default();
        state.submissions.insert("public-a".into(), stored("public-a", "same-key"));
        let found = existing_submission_for(&state, "subject-a", "project-a", "same-key").unwrap();
        assert_eq!(found.public_attempt_id.as_str(), "public-a");
        assert!(existing_submission_for(&state, "subject-b", "project-a", "same-key").is_none());
    }

    #[test]
    fn adapter_state_rejects_duplicate_idempotency_scope() {
        let mut state = GatewayAdapterState::default();
        state.submissions.insert("public-a".into(), stored("public-a", "same-key"));
        state.submissions.insert("public-b".into(), stored("public-b", "same-key"));
        assert!(validate_gateway_state(&state).is_err());
    }

    #[test]
    fn unknown_public_attempt_fails_before_coordinator_access() {
        let state = GatewayAdapterState::default();
        let request = api(GatewayOperation::ReadStatus {
            attempt_id: "missing".into(),
        });
        let attempt = RemoteAttemptId::new("missing").unwrap();
        assert!(authorize_stored_attempt(&request, &attempt, &state).is_err());
    }

    #[test]
    fn dispatch_wire_rejects_unknown_fields_without_echoing_secret_material() {
        let value = serde_json::json!({
            "api": api(GatewayOperation::ReadStatus { attempt_id: "attempt-a".into() }),
            "coordinator_request": null,
            "retry_policy": null,
            "token=must-not-be-echoed": "private-value"
        });
        let bytes = serde_json::to_vec(&value).unwrap();
        let error = parse_dispatch_request(&bytes).unwrap_err().to_string();
        assert_eq!(error, "error: gateway-dispatch-input-invalid");
        assert!(!error.contains("token"));
        assert!(!error.contains("private-value"));
    }

    #[test]
    fn selected_dispatch_and_cursor_bounds_fail_before_effects() {
        let temp = tempfile::tempdir().unwrap();
        let key = signing_key();
        let cursor_key = [0x35_u8; GATEWAY_CURSOR_KEY_BYTES];
        let mut request = RemoteGatewayDispatchRequest {
            api: api(GatewayOperation::ReadStatus {
                attempt_id: "missing".into(),
            }),
            coordinator_request: None,
            retry_policy: None,
        };
        request.api.policy.bounds.message_bytes_max = 1;
        let error = dispatch_gateway_api_request(request, temp.path(), &cursor_key, &key).unwrap_err();
        assert!(error.to_string().contains("gateway-dispatch-message-bound-exceeded"));
        assert!(!temp.path().join(GATEWAY_ADAPTER_STATE_FILE).exists());

        let mut cursor_api = api(GatewayOperation::ReadEventPage {
            attempt_id: "public-a".into(),
            cursor: None,
            item_count: 1,
        });
        cursor_api.policy.bounds.cursor_bytes_max = 1;
        let stored = stored("public-a", "key-a");
        let error = next_cursor(&cursor_api, &stored, 1, true, "events", &cursor_key).unwrap_err();
        assert!(error.to_string().contains("gateway-cursor-selected-bound-exceeded"));
    }

    #[test]
    fn cursor_key_length_is_exact_constant() {
        assert_eq!(GATEWAY_CURSOR_KEY_BYTES, 32);
        assert_eq!(GATEWAY_CURSOR_KEY_READ_BYTES, 33);
    }

    #[test]
    fn non_claim_excludes_build_correctness_promotion() {
        assert!(GATEWAY_API_NON_CLAIM.contains("do not prove build correctness"));
        assert!(GATEWAY_API_NON_CLAIM.contains("release eligibility"));
    }
}
