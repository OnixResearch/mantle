//! Gateway identities are part of the existing coordinator snapshot. They are
//! not a second queue: execution and terminal authority remain fenced jobs.
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use super::RemoteCoordinatorState;
use super::RemoteProductionAttemptBinding;

pub const MAX_GATEWAY_ATTEMPTS: usize = 1024;
pub const MAX_PUBLIC_LOG_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayPhase {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayTransition {
    pub phase: GatewayPhase,
    pub sequence: u64,
    pub observed_unix_s: u64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayReuseRecord {
    pub producer_public_id: String,
    pub producer_project: String,
    pub producer_account: String,
    pub producer_request_id: String,
    pub producer_evidence: String,
    pub consumer_project: String,
    pub sharing_scope: Option<String>,
    pub result_evidence: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayRetryRecord {
    pub predecessor: super::RemoteProductionAttemptBinding,
    pub successor: super::RemoteProductionAttemptBinding,
    pub signed_usage: super::usage::RemoteSignedUsageReport,
    pub machine_class: String,
    pub reservation_units: u64,
    pub observed_unix_s: u64,
    pub policy_version: u32,
    pub predecessor_evidence: String,
    pub replay_json: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayAttemptRecord {
    pub public_id: String,
    pub subject: String,
    pub account: String,
    pub project: String,
    pub authority_policy: String,
    pub authority_decision: String,
    pub endpoint: String,
    pub platform: String,
    pub transitions: Vec<GatewayTransition>,
    pub reuse: Option<GatewayReuseRecord>,
    pub published_evidence: Option<String>,
    pub retries: Vec<GatewayRetryRecord>,
    pub selection_json: Option<serde_json::Value>,
    pub drv_path: String,
    pub request_digest: [u8; 32],
    pub request_id: String,
    pub binding: Option<RemoteProductionAttemptBinding>,
    pub dispatch_started: bool,
    pub cancel_requested: bool,
    pub local_teardown_confirmed: bool,
    /// A process loss is not a teardown acknowledgement and never releases quota.
    pub recovery_uncertain: bool,
    pub result_evidence: Option<[u8; 32]>,
    pub outputs: Vec<String>,
    pub produced_outputs: Vec<super::RemoteProducedOutput>,
    pub usage: Option<super::usage::RemoteSignedUsageReport>,
    /// Only closed, host-generated event codes, never builder stderr or bearer bytes.
    pub public_log: Vec<u8>,
}

impl GatewayAttemptRecord {
    pub fn transition(&mut self, phase: GatewayPhase, now: u64, reason: &'static str) {
        if self.transitions.last().is_some_and(|transition| transition.phase == phase) || self.transitions.len() >= 64 {
            return;
        }
        self.transitions.push(GatewayTransition {
            phase,
            sequence: self.transitions.len() as u64 + 1,
            observed_unix_s: now,
            reason: reason.into(),
        });
        self.append_event(reason);
    }

    pub fn matches(&self, subject: &str, project: &str) -> bool {
        self.subject == subject && self.project == project
    }

    pub fn append_event(&mut self, event: &'static str) {
        if self.public_log.len().saturating_add(event.len()).saturating_add(1) <= MAX_PUBLIC_LOG_BYTES {
            self.public_log.extend_from_slice(event.as_bytes());
            self.public_log.push(b'\n');
        }
    }
}

pub fn validate(
    records: &BTreeMap<String, GatewayAttemptRecord>,
    state: &RemoteCoordinatorState,
) -> Result<(), String> {
    if records.len() > MAX_GATEWAY_ATTEMPTS {
        return Err("gateway-attempt-limit".into());
    }
    for (key, record) in records {
        if key != &record.public_id
            || record.public_id.len() != 64
            || !record.public_id.bytes().all(|byte| byte.is_ascii_hexdigit())
            || [&record.subject, &record.account, &record.project]
                .iter()
                .any(|id| !crunch_remote_core::resource_policy::valid_id(id))
            || !crunch_nix_gateway_core::valid_store_path(&record.drv_path, true)
            || record.request_digest != *blake3::hash(record.drv_path.as_bytes()).as_bytes()
            || record.public_log.len() > MAX_PUBLIC_LOG_BYTES
            || record.outputs.len() > 64
            || record.produced_outputs.len() > 64
            || record.transitions.len() > 64
            || record.retries.len() > 32
            || record
                .transitions
                .iter()
                .enumerate()
                .any(|(index, transition)| transition.sequence != index as u64 + 1 || transition.reason.len() > 128)
            || record.reuse.as_ref().is_some_and(|reuse| {
                reuse.consumer_project != record.project
                    || Some(reuse.result_evidence) != record.result_evidence
                    || reuse.producer_public_id == record.public_id
            })
            || record.outputs.iter().any(|path| !crunch_nix_gateway_core::valid_store_path(path, false))
            || (record.dispatch_started && record.binding.is_none())
            || (record.result_evidence.is_some() && (record.outputs.is_empty() || record.binding.is_none()))
        {
            return Err("gateway-attempt-state-invalid".into());
        }
        if let Some(binding) = &record.binding {
            let job = state.jobs.get(&binding.job_id).ok_or("gateway-attempt-job-missing")?;
            let attempt = job.current_attempt.as_ref().ok_or("gateway-attempt-fence-missing")?;
            if let Some(reuse) = &record.reuse {
                let producer = records.get(&reuse.producer_public_id).ok_or("gateway-reuse-producer-missing")?;
                if producer.project != reuse.producer_project
                    || producer.binding != record.binding
                    || producer.request_digest != record.request_digest
                    || producer.result_evidence != record.result_evidence
                    || producer.outputs != record.outputs
                    || producer.produced_outputs != record.produced_outputs
                {
                    return Err("gateway-reuse-source-mismatch".into());
                }
            }
            if attempt.attempt_id != binding.attempt_id || attempt.fence_generation != binding.fence_generation {
                return Err("gateway-attempt-fence-mismatch".into());
            }
        }
    }
    Ok(())
}

pub fn cancellation_requested(state: &RemoteCoordinatorState, binding: &RemoteProductionAttemptBinding) -> bool {
    state
        .gateway_attempts
        .values()
        .any(|record| record.binding.as_ref() == Some(binding) && record.cancel_requested)
}

/// Save output evidence in the same coordinator commit which marks completion.
/// The caller already ran ordinary signed-result and CAS import admission.
pub fn record_result(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    digest: &str,
    outputs: &[super::RemoteProducedOutput],
    now_unix_s: u64,
) -> Result<(), String> {
    let mut evidence = [0_u8; 32];
    if digest.len() != 64 {
        return Err("gateway-result-digest-invalid".into());
    }
    for (index, byte) in evidence.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&digest[index * 2..index * 2 + 2], 16).map_err(|_| "gateway-result-digest-invalid")?;
    }
    for record in state.gateway_attempts.values_mut().filter(|record| record.binding.as_ref() == Some(binding)) {
        record.result_evidence = Some(evidence);
        record.outputs = outputs.iter().map(|output| output.logical_path.clone()).collect();
        record.produced_outputs = outputs.to_vec();
        record.transition(GatewayPhase::Succeeded, now_unix_s, "gateway-output-admitted");
    }
    Ok(())
}

pub fn record_usage(
    state: &mut RemoteCoordinatorState,
    binding: &RemoteProductionAttemptBinding,
    transcript: &super::RemoteStdioTranscript,
) {
    let usage = transcript.frames.iter().find_map(|frame| match frame {
        super::RemoteFrame::BuildFinished { result } => result.usage.clone(),
        _ => None,
    });
    for record in state.gateway_attempts.values_mut().filter(|record| record.binding.as_ref() == Some(binding)) {
        record.usage = usage.clone();
    }
}

/// Only a same-binary local worker has a process-tree teardown contract. SSH
/// disconnect cannot acknowledge teardown on another machine.
pub fn local_cancel_requested(command: &super::RemoteStdioCommand) -> Result<bool, crate::errors::RunError> {
    let Some(production) = command.production_transfer.as_ref() else {
        return Ok(false);
    };
    let current = std::env::current_exe()
        .map_err(|_| crate::errors::RunError::Internal("gateway-worker-identity-unavailable".into()))?;
    if std::fs::canonicalize(&command.program).ok().as_ref() != Some(&current) {
        return Ok(false);
    }
    let binding = command
        .input_frames
        .iter()
        .find_map(|frame| match frame {
            super::RemoteFrame::BuildRequest { request } => request.production_attempt.as_ref(),
            _ => None,
        })
        .ok_or_else(|| crate::errors::RunError::Internal("gateway-worker-binding-missing".into()))?;
    let _guard = super::RemoteCoordinatorMutationGuard::acquire_wait(&production.state_dir)
        .map_err(crate::errors::RunError::Internal)?;
    let state = super::load_coordinator_state(&production.state_dir)?;
    Ok(cancellation_requested(&state, binding))
}

pub fn confirm_local_teardown(command: &super::RemoteStdioCommand) -> Result<(), crate::errors::RunError> {
    let production = command
        .production_transfer
        .as_ref()
        .ok_or_else(|| crate::errors::RunError::Internal("gateway-worker-binding-missing".into()))?;
    let binding = command
        .input_frames
        .iter()
        .find_map(|frame| match frame {
            super::RemoteFrame::BuildRequest { request } => request.production_attempt.as_ref(),
            _ => None,
        })
        .ok_or_else(|| crate::errors::RunError::Internal("gateway-worker-binding-missing".into()))?;
    let _guard = super::RemoteCoordinatorMutationGuard::acquire_wait(&production.state_dir)
        .map_err(crate::errors::RunError::Internal)?;
    let mut state = super::load_coordinator_state(&production.state_dir)?;
    for record in state.gateway_attempts.values_mut().filter(|record| record.binding.as_ref() == Some(binding)) {
        if record.cancel_requested {
            record.local_teardown_confirmed = true;
            record.append_event("gateway-local-worker-reaped");
        }
    }
    super::save_coordinator_state(&production.state_dir, &state)
}

pub fn record_failed_usage(
    state_dir: &std::path::Path,
    binding: &super::RemoteProductionAttemptBinding,
    request_id: &str,
    report: &super::usage::RemoteSignedUsageReport,
    trusted: &[String],
) -> Result<(), String> {
    let digest = super::usage::failure_result_digest(binding, request_id);
    // Verification reasons are fixed codes; keep the last one so a rejection
    // names its cause instead of only the outcome.
    let mut rejection = String::from("no-trusted-output-key");
    let mut verified = None;
    for key in trusted.iter().filter_map(|encoded| nix_compat::narinfo::VerifyingKey::parse(encoded).ok()) {
        match report.verify(binding, request_id, &digest, &key) {
            Ok(usage) => {
                verified = Some(usage);
                break;
            }
            Err(reason) => rejection = reason,
        }
    }
    let usage = verified.ok_or_else(|| format!("gateway-failure-usage-signature-invalid: {rejection}"))?;
    let _guard = super::RemoteCoordinatorMutationGuard::acquire_wait(state_dir)?;
    let mut state = super::load_coordinator_state(state_dir).map_err(|_| "gateway-coordinator-unavailable")?;
    let job = state.jobs.get(&binding.job_id).ok_or("gateway-failure-job-missing")?;
    let attempt = job.current_attempt.as_ref().ok_or("gateway-failure-attempt-missing")?;
    if attempt.attempt_id != binding.attempt_id || attempt.fence_generation != binding.fence_generation {
        return Err("gateway-failure-fence-stale".into());
    }
    for record in state
        .gateway_attempts
        .values_mut()
        .filter(|record| record.binding.as_ref() == Some(binding) && record.reuse.is_none())
    {
        if record.request_id != request_id || record.usage.as_ref().is_some_and(|prior| prior != report) {
            return Err("gateway-failure-usage-conflict".into());
        }
        record.usage = Some(report.clone());
        record.append_event("gateway-authenticated-failed-usage");
    }
    let retryable = usage.measurement_completeness() == super::usage::RemoteUsageCompleteness::DedicatedCgroup
        && matches!(usage.oom_evidence(), super::usage::RemoteUsageOomEvidence::DedicatedCgroupOom {
            oom_kill_delta: 1..
        });
    if state
        .jobs
        .get(&binding.job_id)
        .and_then(|job| job.current_attempt.as_ref())
        .is_some_and(|attempt| attempt.phase.is_live())
    {
        let event = super::remote_cli_now_unix_s(request_id)?;
        super::fail_remote_production_attempt_with_class(
            &mut state,
            binding,
            event,
            if retryable {
                super::RemoteAttemptFailureClass::Retryable
            } else {
                super::RemoteAttemptFailureClass::Terminal
            },
        )?;
    }
    super::save_coordinator_state(state_dir, &state).map_err(|_| "gateway-failure-usage-storage".into())
}

pub fn plan_retry_assignment(
    state: &super::RemoteCoordinatorState,
    binding: &super::RemoteProductionAttemptBinding,
    endpoint: &str,
    retry_policy: super::RemoteAttemptRetryPolicy,
    time: super::RemoteAttemptTimeFacts,
) -> Result<super::RemoteAttemptState, String> {
    let job = state.jobs.get(&binding.job_id).ok_or("gateway-retry-job-missing")?;
    let previous = job.current_attempt.as_ref().ok_or("gateway-retry-attempt-missing")?;
    if previous.attempt_id != binding.attempt_id
        || previous.fence_generation != binding.fence_generation
        || previous.phase != super::RemoteAttemptPhase::Failed
        || job.last_attempt_failure_class != Some(super::RemoteAttemptFailureClass::Retryable)
    {
        return Err("gateway-retry-not-authorized".into());
    }
    super::plan_remote_attempt_assignment_from_input(super::RemoteAttemptAssignmentInput {
        job_id: &binding.job_id,
        worker_endpoint_id: endpoint,
        assignment_nonce: super::generate_remote_assignment_nonce(&job.normalized_build_key)?,
        previous: Some(previous),
        retry_policy,
        time,
    })
    .map_err(|reason| reason.as_str().into())
}

pub fn commit_retry(
    state: &mut super::RemoteCoordinatorState,
    public_id: &str,
    next: super::RemoteAttemptState,
    endpoint: &str,
    requirements: super::RemoteResourceRequirements,
    retry: GatewayRetryRecord,
) -> Result<(), String> {
    let mut candidate = state.clone();
    let record = candidate.gateway_attempts.get(public_id).ok_or("gateway-retry-public-id-missing")?;
    if record.cancel_requested
        || record.binding.as_ref() != Some(&retry.predecessor)
        || next.attempt_id != retry.successor.attempt_id
        || next.fence_generation != retry.successor.fence_generation
        || record.retries.len() >= 32
    {
        return Err("gateway-retry-binding-conflict".into());
    }
    let job_id = retry.predecessor.job_id.clone();
    // The verified failure report already released the old execution lease.
    let job = candidate.jobs.get_mut(&job_id).ok_or("gateway-retry-job-missing")?;
    job.resource_requirements = Some(requirements);
    super::install_reassigned_attempt(&mut candidate, &job_id, endpoint, next)?;
    super::install_reassigned_resource_lease(&mut candidate, &job_id, endpoint)?;
    let job = candidate.jobs.get(&job_id).ok_or("gateway-retry-job-missing")?;
    for claim in &job.live_output_claims {
        if candidate.live_output_claims.get(claim).is_some_and(|owner| owner != &job.normalized_build_key) {
            return Err("gateway-retry-output-claim-conflict".into());
        }
        candidate.live_output_claims.insert(claim.clone(), job.normalized_build_key.clone());
    }
    let record = candidate.gateway_attempts.get_mut(public_id).ok_or("gateway-retry-public-id-missing")?;
    record.binding = Some(retry.successor.clone());
    record.dispatch_started = false;
    record.recovery_uncertain = false;
    record.usage = None;
    record.published_evidence = None;
    record.selection_json = None;
    record.transitions.clear();
    record.transition(GatewayPhase::Queued, retry.observed_unix_s, "gateway-positive-oom-retry-admitted");
    record.retries.push(retry);
    super::persist_coordinator_candidate(&candidate)?;
    *state = candidate;
    Ok(())
}

/// A scheduler attachment is not project-sharing authority and cannot bind a
/// different gateway request to someone else's execution/accounting identity.
/// Call while holding the ordinary coordinator mutation guard, immediately
/// before the existing production admission. Authorized completed-result reuse
/// has its separate signed-result/policy/CAS gate in the host.
pub fn require_new_execution(
    state: &super::RemoteCoordinatorState,
    request: &super::ConcreteBuildRequest,
    trusted_output_keys: &[String],
) -> Result<(), String> {
    let executable = super::plan_remote_executable_request(request)?;
    let concrete = super::production_coordinator_request(request, trusted_output_keys, &executable.system);
    let key = super::normalized_remote_build_key(&concrete)?;
    if state.jobs.values().any(|job| job.normalized_build_key == key) {
        return Err("gateway-existing-execution-requires-authorized-reuse".into());
    }
    Ok(())
}
