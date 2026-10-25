//! Concrete private API adapter over Mantle's coordinator and ordinary remote
//! execution/import path. The API journal stores delivery state, not scheduling.
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use crunch_nix_gateway::api::Authorization as _;
use crunch_nix_gateway::api::Error;
use crunch_nix_gateway::api::Observation;
use crunch_nix_gateway::api::Phase;
use crunch_nix_gateway::api::Request;
use crunch_nix_gateway::api::Services;
use crunch_nix_gateway::api::{self};
use crunch_nix_gateway_core::Authority;
use crunch_nix_gateway_core::Reject;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::gateway_operations::GatewayOperation;
use mantle_application_contract::gateway_operations::GatewayOperationEvidence;
use mantle_application_contract::gateway_operations::GatewayOperationPlan;
use mantle_application_contract::gateway_operations::GatewayOperationPort;
use mantle_application_contract::gateway_operations::GatewayOperationResult;
use mantle_application_contract::gateway_operations::classify_gateway_operation;
use mantle_application_contract::gateway_operations::plan_gateway_operation;
use serde::Deserialize;
use serde::Serialize;

use super::authority::GatewayAuthorityConfig;
use super::authority::GatewaySession;
use super::authority::GatewayVerifier;
use super::authority::Operation;
use super::resource_host::AttemptBinding;
use super::resource_host::PendingUsage;
use super::resource_host::ResourcePolicyConfig;
use super::resource_host::ResourcePolicyHost;
use super::resource_host::SelectionRequest;
use crate::RemoteBuildSelection;
use crate::RunContext;
use crate::remote_build::gateway::GatewayAttemptRecord;
use crate::remote_build::gateway::GatewayPhase;
use crate::remote_build::{self};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GatewayConfig {
    pub version: u32,
    pub endpoint: String,
    pub audience: String,
    pub policy: [u8; 32],
    pub authority: GatewayAuthorityConfig,
    /// Exact operator-provisioned receiver binding; never created from a request.
    pub builder_endpoint_state_directory: PathBuf,
    pub builder_endpoint_binding: crunch_remote_core::endpoint::EndpointBinding,
    pub builder_endpoint_scope: String,
    pub builder_endpoint_trust_requirement: String,
    pub journal_directory: PathBuf,
    pub cursor_secret_file: PathBuf,
    pub completion_key_file: PathBuf,
    pub quota_directory: PathBuf,
    pub evidence_directory: PathBuf,
    pub resources: ResourcePolicyConfig,
    /// Reviewed features indexed by the exact admitted derivation system.
    pub features: BTreeMap<String, crunch_remote_core::resource_policy::Features>,
    pub builder_program: PathBuf,
    pub builder_args: Vec<String>,
    pub trusted_builder_keys: Vec<String>,
    pub observability_config: PathBuf,
    pub build_time_limit_secs: u64,
    pub secret_manifest: PathBuf,
    pub secret_profile: String,
    pub secret_provider: String,
}

#[derive(Clone)]
pub(crate) struct GatewayHost(Arc<Host>);
struct Host {
    ctx: RunContext,
    verifier: GatewayVerifier,
    resources: ResourcePolicyHost,
    selection: RemoteBuildSelection,
    endpoint_store: remote_build::endpoint::EndpointBindingStore,
    endpoint_binding: crunch_remote_core::endpoint::EndpointBinding,
    endpoint_scope: String,
    endpoint_trust_requirement: String,
    features: BTreeMap<String, crunch_remote_core::resource_policy::Features>,
    active: Mutex<BTreeMap<String, std::thread::JoinHandle<()>>>,
    producer_policy: [u8; 32],
    evidence_directory: std::fs::File,
    _service_lock: std::fs::File,
}

/// Non-deserializable per-operation guard. The Arc retains the actual privately
/// minted, nonclone Basalt Session through request handling and worker dispatch;
/// the pure core borrows its admission projection without copying credentials.
pub(crate) struct GatewayAuthorization {
    session: Arc<GatewaySession>,
}

impl api::Authorization for GatewayAuthorization {
    fn facts(&self) -> Authority<'_> {
        self.session.admission_facts()
    }
}

impl GatewayAuthorization {
    fn session_matches(&self, session: &Arc<GatewaySession>) -> Result<(), Error> {
        if !Arc::ptr_eq(&self.session, session) {
            return Err(Reject::Authority.into());
        }
        Ok(())
    }

    fn require_target(&self, operation: Operation, target: &str) -> Result<(), Error> {
        if self.session.operation() != operation || self.session.target() != target {
            return Err(Reject::Authority.into());
        }
        Ok(())
    }
}

fn host_error<T>(_: T) -> Error {
    Error::Host
}
fn now() -> Result<u64, Error> {
    remote_build::remote_cli_now_unix_s("gateway-api").map_err(host_error)
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("String write");
    }
    encoded
}

/// Canonical signed operation binding. Clients sign these tuples, not the
/// credential envelope. Cursor/window/fence and idempotency are included.
pub(crate) fn request_binding(
    request: Request<'_>,
    scope: &str,
    endpoint_generation: u64,
) -> Result<(Operation, String, [u8; 32]), Error> {
    if endpoint_generation == 0 {
        return Err(Reject::Identity.into());
    }
    let (operation, target, bytes) = match request {
        Request::Submit {
            drv_path,
            idempotency_key,
        } => (Operation::BuildSubmit, drv_path, serde_json::to_vec(&("submit", drv_path, idempotency_key))),
        Request::Status { attempt_id } => {
            (Operation::StatusRead, attempt_id, serde_json::to_vec(&("status", attempt_id)))
        }
        Request::Logs {
            attempt_id,
            cursor,
            bytes,
        } => (Operation::LogRead, attempt_id, serde_json::to_vec(&("logs", attempt_id, cursor, bytes))),
        Request::Events {
            attempt_id,
            cursor,
            items,
        } => (Operation::EventRead, attempt_id, serde_json::to_vec(&("events", attempt_id, cursor, items))),
        Request::Cancel { attempt_id, fence } => {
            (Operation::OwnedAttemptCancel, attempt_id, serde_json::to_vec(&("cancel", attempt_id, fence)))
        }
        Request::Result { attempt_id } => {
            (Operation::ResultRead, attempt_id, serde_json::to_vec(&("result", attempt_id)))
        }
        Request::Usage { items } => (Operation::UsageRead, scope, serde_json::to_vec(&("usage", scope, items))),
    };
    let mut binding = blake3::Hasher::new();
    binding.update(b"mantle-gateway-operation-v1\0");
    binding.update(&endpoint_generation.to_le_bytes());
    binding.update(&bytes.map_err(host_error)?);
    Ok((operation, target.into(), *binding.finalize().as_bytes()))
}

impl GatewayHost {
    pub(crate) fn new(
        ctx: RunContext,
        config: &GatewayConfig,
        selection: RemoteBuildSelection,
        verifier: GatewayVerifier,
        resources: ResourcePolicyHost,
    ) -> Result<Self, Error> {
        if config.version != 1
            || config.endpoint != selection.options.builder.endpoint_id
            || config.resources.classes.iter().any(|class| class.worker_endpoint_id != config.endpoint)
            || selection.options.endpoint_binding.as_ref() != Some(&config.builder_endpoint_binding)
            || selection.resource_inventory.is_none()
            || ctx.store_prefix != "/nix/store"
        {
            return Err(Error::Rejected(Reject::Authority));
        }
        if config.builder_endpoint_binding.spec.name != config.endpoint {
            return Err(Reject::Authority.into());
        }
        let endpoint_store =
            remote_build::endpoint::EndpointBindingStore::open(&config.builder_endpoint_state_directory)
                .map_err(host_error)?;
        endpoint_store
            .verify_received(
                &config.builder_endpoint_binding,
                crunch_remote_core::endpoint::EndpointReceiverRequirements {
                    kind: crunch_remote_core::endpoint::EndpointKind::BuilderTicket,
                    scope: &config.builder_endpoint_scope,
                    trust_requirement: &config.builder_endpoint_trust_requirement,
                    store_prefix: &ctx.store_prefix,
                },
                now()?,
            )
            .map_err(host_error)?;
        use std::os::unix::fs::OpenOptionsExt;
        private_metadata(&config.evidence_directory, true)?;
        // Recovery may classify an interrupted execution only when no other
        // host for this coordinator is alive, even on a different socket.
        let lock_path = ctx.resolved_state_dir.join("nix-gateway-host.lock");
        let service_lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&lock_path)
            .map_err(host_error)?;
        private_metadata(&lock_path, false)?;
        fs2::FileExt::try_lock_exclusive(&service_lock).map_err(|_| Error::Busy)?;
        let evidence_directory = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(&config.evidence_directory)
            .map_err(host_error)?;
        Ok(Self(Arc::new(Host {
            ctx,
            verifier,
            resources,
            selection,
            endpoint_store,
            endpoint_binding: config.builder_endpoint_binding.clone(),
            endpoint_scope: config.builder_endpoint_scope.clone(),
            endpoint_trust_requirement: config.builder_endpoint_trust_requirement.clone(),
            features: config.features.clone(),
            active: Mutex::new(BTreeMap::new()),
            producer_policy: config.policy,
            evidence_directory,
            _service_lock: service_lock,
        })))
    }

    fn verify_builder_endpoint(&self, at: u64) -> Result<(), Error> {
        // A short local check is not a long-lived grant. The actual receiver
        // must readmit the propagated exact descriptor during ticket redemption.
        self.0
            .endpoint_store
            .verify_received(
                &self.0.endpoint_binding,
                crunch_remote_core::endpoint::EndpointReceiverRequirements {
                    kind: crunch_remote_core::endpoint::EndpointKind::BuilderTicket,
                    scope: &self.0.endpoint_scope,
                    trust_requirement: &self.0.endpoint_trust_requirement,
                    store_prefix: &self.0.ctx.store_prefix,
                },
                at,
            )
            .map_err(host_error)?;
        Ok(())
    }

    pub(crate) fn verifier(&self) -> &GatewayVerifier {
        &self.0.verifier
    }

    /// Exact native/API operation verification over the canonical holder-signed
    /// resource. This is not a trusted-connection grant: callers must retain the
    /// returned guard and supply the exact bounded target/body digest each time.
    pub(crate) fn authorize_nix_operation(
        &self,
        session: &Arc<GatewaySession>,
        operation: Operation,
        target: &str,
        digest: [u8; 32],
        at: u64,
    ) -> Result<GatewayAuthorization, Error> {
        session.require(operation, target, digest).map_err(|_| Error::Rejected(Reject::Authority))?;
        if session.endpoint() != self.0.selection.options.builder.endpoint_id
            || session.policy_identity() != format!("blake3:{}", hex(&self.0.producer_policy))
        {
            return Err(Reject::Authority.into());
        }
        self.0.resources.summary(session.project(), session.account()).map_err(host_error)?;
        self.0.verifier.revalidate(session, at).map_err(|_| Error::Rejected(Reject::Authority))?;
        Ok(GatewayAuthorization {
            session: Arc::clone(session),
        })
    }

    fn completed_record(&self, authority: Authority<'_>, id: &str) -> Result<GatewayAttemptRecord, Error> {
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
            .map_err(host_error)?;
        let state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
        let record = state.gateway_attempts.get(id).ok_or(Error::Unknown)?;
        if !record.matches(authority.subject, authority.scope) {
            return Err(Reject::Authority.into());
        }
        if Self::observation(&state, record)?.phase != Phase::Succeeded {
            return Err(Error::Integrity);
        }
        let binding = record.binding.as_ref().ok_or(Error::Integrity)?;
        let current = state
            .jobs
            .get(&binding.job_id)
            .and_then(|job| job.current_attempt.as_ref())
            .ok_or(Error::Integrity)?;
        let digest = blake3::Hash::from_bytes(record.result_evidence.ok_or(Error::Integrity)?).to_hex();
        if current.phase != crunch_remote_core::attempt::RemoteAttemptPhase::Completed
            || current.result_digest_blake3.as_deref() != Some(digest.as_str())
        {
            return Err(Error::Integrity);
        }
        Ok(record.clone())
    }

    fn observation(
        state: &remote_build::RemoteCoordinatorState,
        record: &GatewayAttemptRecord,
    ) -> Result<Observation, Error> {
        let Some(binding) = &record.binding else {
            return Ok(Observation {
                phase: Phase::Queued,
                fence: 1,
                result_evidence: None,
            });
        };
        let job = state.jobs.get(&binding.job_id).ok_or(Error::Integrity)?;
        let current = job.current_attempt.as_ref().ok_or(Error::Integrity)?;
        if current.attempt_id != binding.attempt_id || current.fence_generation != binding.fence_generation {
            return Err(Reject::StaleFence.into());
        }
        let phase = if record.local_teardown_confirmed {
            Phase::Cancelled
        } else if record.result_evidence.is_some() && job.output_admission_completed && job.result_available {
            Phase::Succeeded
        } else if record.recovery_uncertain
            || record.transitions.last().is_some_and(|event| event.phase == GatewayPhase::Failed)
        {
            Phase::Failed
        } else if record.dispatch_started || !record.retries.is_empty() {
            Phase::Running
        } else {
            Phase::Queued
        };
        Ok(Observation {
            phase,
            fence: binding.fence_generation.get(),
            result_evidence: if phase == Phase::Succeeded {
                record.result_evidence
            } else {
                None
            },
        })
    }

    fn quota_binding(record: &GatewayAttemptRecord) -> Result<AttemptBinding, Error> {
        let binding = record.binding.as_ref().ok_or(Error::Integrity)?;
        Ok(AttemptBinding {
            job: binding.job_id.clone(),
            attempt: binding.attempt_id.clone(),
            fence: binding.fence_generation,
            project: record.project.clone(),
            account: record.account.clone(),
            request_id: record.request_id.clone(),
            request_digest: hex(&record.request_digest),
        })
    }

    fn authenticated_usage(
        &self,
        record: &GatewayAttemptRecord,
    ) -> Result<Option<remote_build::usage::RemoteAuthenticatedUsage>, Error> {
        let Some(report) = &record.usage else { return Ok(None) };
        let binding = record.binding.as_ref().ok_or(Error::Integrity)?;
        let evidence = record
            .result_evidence
            .map(|evidence| hex(&evidence))
            .unwrap_or_else(|| remote_build::usage::failure_result_digest(binding, &record.request_id));
        for encoded in &self.0.selection.options.trusted_output_keys {
            if let Ok(key) = nix_compat::narinfo::VerifyingKey::parse(encoded)
                && let Ok(usage) = report.verify(binding, &record.request_id, &evidence, &key)
            {
                return Ok(Some(usage));
            }
        }
        Err(Error::Integrity)
    }

    fn authenticated_retry_usage(
        &self,
        record: &GatewayAttemptRecord,
        retry: &remote_build::gateway::GatewayRetryRecord,
    ) -> Result<remote_build::usage::RemoteAuthenticatedUsage, Error> {
        let digest = remote_build::usage::failure_result_digest(&retry.predecessor, &record.request_id);
        self.0
            .selection
            .options
            .trusted_output_keys
            .iter()
            .filter_map(|encoded| nix_compat::narinfo::VerifyingKey::parse(encoded).ok())
            .find_map(|key| retry.signed_usage.verify(&retry.predecessor, &record.request_id, &digest, &key).ok())
            .ok_or(Error::Integrity)
    }

    fn reconcile(&self, record: &GatewayAttemptRecord) -> Result<crunch_remote_core::resource_quota::Entry, Error> {
        let binding = Self::quota_binding(record)?;
        if let Some(reuse) = &record.reuse {
            let producer_binding = AttemptBinding {
                project: reuse.producer_project.clone(),
                account: reuse.producer_account.clone(),
                request_id: reuse.producer_request_id.clone(),
                ..binding
            };
            return self.0.resources.existing_entry(&producer_binding).map_err(host_error);
        }
        let observed_unix_s = now()?;
        let entry = if let Some(usage) = self.authenticated_usage(record)? {
            self.0.resources.reconcile(&binding, &usage, observed_unix_s).map_err(host_error)?
        } else {
            let pending = if record.local_teardown_confirmed {
                PendingUsage::Cancelled
            } else if record.recovery_uncertain {
                PendingUsage::WorkerLost
            } else {
                PendingUsage::Partial
            };
            self.0.resources.mark_pending(&binding, pending, observed_unix_s).map_err(host_error)?
        };
        self.publish_evidence(record, &entry, None, false, None)?;
        Ok(entry)
    }

    fn publish_evidence(
        &self,
        record: &GatewayAttemptRecord,
        entry: &crunch_remote_core::resource_quota::Entry,
        selection: Option<&crunch_build::distributed::PolicyEvaluation>,
        verified_result: bool,
        result_session: Option<&GatewaySession>,
    ) -> Result<String, Error> {
        if let Some(reuse) = &record.reuse {
            let producer = super::evidence::read_publication(&self.0.evidence_directory, &reuse.producer_evidence)
                .map_err(host_error)?;
            let session = result_session.ok_or(Error::Integrity)?;
            let mut evidence = super::evidence::GatewayEvidence::new_reuse(record, self.0.producer_policy, &producer)
                .map_err(host_error)?;
            let authority = evidence
                .record_authority(super::evidence::PublicAuthority {
                    subject: session.subject(),
                    account: session.account(),
                    project: session.project(),
                    policy_identity: session.policy_identity(),
                    operation: "result-read",
                    decision_id: session.decision_id(),
                })
                .map_err(host_error)?;
            evidence
                .record_alias_reuse(
                    &authority,
                    &format!("current-signed-output-check:{}", hex(&reuse.result_evidence)),
                    &format!("current-cas-check:{}", hex(&reuse.result_evidence)),
                )
                .map_err(host_error)?;
            return evidence.finish().map_err(host_error)?.publish(&self.0.evidence_directory).map_err(host_error);
        }
        let mut evidence =
            super::evidence::GatewayEvidence::new(record, self.0.producer_policy, entry).map_err(host_error)?;
        for retry in &record.retries {
            let predecessor =
                super::evidence::read_publication(&self.0.evidence_directory, &retry.predecessor_evidence)
                    .map_err(host_error)?;
            let usage = self.authenticated_retry_usage(record, retry)?;
            evidence.record_retained_retry(retry, &predecessor, &usage).map_err(host_error)?;
        }
        evidence.record_durable_facts().map_err(host_error)?;
        if record.retries.is_empty() {
            if let Some(selection) = selection {
                evidence.record_selection(selection).map_err(host_error)?;
            } else if let Some(snapshot) = &record.selection_json {
                evidence.record_selection_snapshot(snapshot).map_err(host_error)?;
            }
        }
        if let Some(usage) = self.authenticated_usage(record)? {
            evidence.record_authenticated_usage(&usage).map_err(host_error)?;
        }
        if let Some(observation) = self.0.resources.observation(&Self::quota_binding(record)?).map_err(host_error)? {
            evidence.record_observation(&observation).map_err(host_error)?;
        }
        if verified_result {
            let outputs = record
                .produced_outputs
                .iter()
                .map(|output| valence_core::build_service::BuildOutputRef {
                    path: output.logical_path.clone(),
                    content_hash: output.content_digest_blake3.clone(),
                })
                .collect();
            let signed_metadata = record
                .produced_outputs
                .iter()
                .map(|output| output.path_info.as_ref().ok_or(Error::Integrity))
                .collect::<Result<Vec<_>, _>>()?;
            let signed_metadata = serde_json::to_vec(&signed_metadata).map_err(host_error)?;
            let signature_ref = blake3::hash(&signed_metadata).to_hex();
            evidence
                .record_result(
                    outputs,
                    &record.platform,
                    &hex(&self.0.producer_policy),
                    &format!("signed-pathinfo-blake3:{signature_ref}"),
                    &format!("current-signed-pathinfo-cas:{}", hex(&record.result_evidence.ok_or(Error::Integrity)?)),
                )
                .map_err(host_error)?;
        }
        evidence.finish().map_err(host_error)?.publish(&self.0.evidence_directory).map_err(host_error)
    }

    /// On restart an already-dispatched job is never launched again. Existing
    /// current signed completion remains discoverable; uncertain worker loss
    /// retains its reservation and fence for explicit reconciliation.
    pub(crate) fn recover(&self) -> Result<(), Error> {
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
            .map_err(host_error)?;
        let mut state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
        let at = now()?;
        for record in state.gateway_attempts.values_mut() {
            if record.dispatch_started && record.result_evidence.is_none() && !record.local_teardown_confirmed {
                record.recovery_uncertain = true;
                record.transition(GatewayPhase::Failed, at, "gateway-worker-recovery-uncertain");
            }
        }
        remote_build::save_coordinator_state(&self.0.ctx.resolved_state_dir, &state).map_err(host_error)?;
        for record in
            state.gateway_attempts.values().filter(|record| record.binding.is_some() && record.dispatch_started)
        {
            self.reconcile(record)?;
        }
        Ok(())
    }

    fn record_worker_error(&self, id: &str) -> Result<(), Error> {
        let state_dir = &self.0.ctx.resolved_state_dir;
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(state_dir).map_err(host_error)?;
        let mut state = remote_build::load_coordinator_state(state_dir).map_err(host_error)?;
        let record = state.gateway_attempts.get_mut(id).ok_or(Error::Unknown)?;
        if record.result_evidence.is_some() {
            return Ok(());
        }
        // A durable dispatch marker precedes every child launch. Its absence
        // proves that this particular fenced successor never ran.
        if !record.dispatch_started && record.cancel_requested {
            record.local_teardown_confirmed = true;
        }
        record.recovery_uncertain =
            record.dispatch_started && record.usage.is_none() && !record.local_teardown_confirmed;
        record.transition(
            if record.local_teardown_confirmed {
                GatewayPhase::Cancelled
            } else {
                GatewayPhase::Failed
            },
            now()?,
            "gateway-worker-stopped-before-completion",
        );
        let record = record.clone();
        if !record.dispatch_started
            && let Some(binding) = &record.binding
            && state
                .jobs
                .get(&binding.job_id)
                .and_then(|job| job.current_attempt.as_ref())
                .is_some_and(|attempt| attempt.phase.is_live())
        {
            remote_build::fail_remote_production_attempt_terminal(&mut state, binding, now()?).map_err(host_error)?;
        }
        remote_build::save_coordinator_state(state_dir, &state).map_err(host_error)?;
        if !record.dispatch_started
            && matches!(
                self.0.resources.existing_entry(&Self::quota_binding(&record)?),
                Err(super::resource_host::ResourceHostError::Quota(
                    crunch_remote_core::resource_quota::QuotaError::UnknownAttempt
                ))
            )
        {
            return Ok(());
        }
        self.reconcile(&record).map(|_| ())
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Exact current fence, verified usage and existing runner context"
    )]
    fn retry_after_oom(
        &self,
        state: &mut remote_build::RemoteCoordinatorState,
        record: &GatewayAttemptRecord,
        session: &GatewaySession,
        selection: &RemoteBuildSelection,
        prepared: &mut crate::PreparedRemoteDispatch,
        selected: &mut super::resource_host::SelectedResources,
    ) -> Result<bool, Error> {
        use crunch_remote_core::resource_policy::RetryContext;
        if record.cancel_requested || record.result_evidence.is_some() {
            return Ok(false);
        }
        let Some(usage) = self.authenticated_usage(record)? else {
            return Ok(false);
        };
        if !matches!(usage.oom_evidence(), remote_build::usage::RemoteUsageOomEvidence::DedicatedCgroupOom {
            oom_kill_delta: 1..
        }) {
            return Ok(false);
        }
        let observed = now()?;
        self.0.verifier.revalidate(session, observed).map_err(|_| Error::Rejected(Reject::Authority))?;
        let policy = self.0.resources.retry_policy();
        let started = if let Some(first) = record.retries.first() {
            let elapsed = first
                .replay_json
                .get("context")
                .and_then(|context| context.get("elapsed_ms"))
                .and_then(serde_json::Value::as_u64)
                .ok_or(Error::Integrity)?;
            first.observed_unix_s.checked_sub(elapsed / 1000).ok_or(Error::Integrity)?
        } else {
            record.transitions.first().ok_or(Error::Integrity)?.observed_unix_s
        };
        let elapsed_ms =
            observed.checked_sub(started).ok_or(Error::Integrity)?.checked_mul(1000).ok_or(Error::Integrity)?;
        let next_wall_budget_ms = selection.options.build_time_limit_secs.checked_mul(1000).ok_or(Error::Integrity)?;
        let binding = record.binding.as_ref().ok_or(Error::Integrity)?;
        let retry_policy = crunch_remote_core::attempt::RemoteAttemptRetryPolicy {
            max_attempts: policy.max_retries.saturating_add(1).min(crunch_remote_core::attempt::MAX_REMOTE_ATTEMPTS),
            retry_delay_secs: 0,
            attempt_timeout_secs: selection.options.build_time_limit_secs,
        };
        let time = crunch_remote_core::attempt::RemoteAttemptTimeFacts {
            now_unix_s: observed,
            failure_observed_unix_s: observed,
            overall_deadline_unix_s: started.checked_add(policy.max_wall_ms / 1000).ok_or(Error::Integrity)?,
        };
        let next = match remote_build::gateway::plan_retry_assignment(
            state,
            binding,
            selected.worker_endpoint_id(),
            retry_policy,
            time,
        ) {
            Ok(next) => next,
            Err(_) => return Ok(false),
        };
        let cumulative_units = record
            .retries
            .iter()
            .try_fold(selected.machine_class().reservation_units, |total, retry| {
                total.checked_add(retry.reservation_units)
            })
            .ok_or(Error::Integrity)?;
        let quota_binding = Self::quota_binding(record)?;
        let entry = self.reconcile(record)?;
        let leases = state.resource_leases.values().cloned().collect::<Vec<_>>();
        let retry = match self.0.resources.plan_oom_retry(
            selected,
            &quota_binding,
            &usage,
            RetryContext {
                job: &binding.job_id,
                attempt: &binding.attempt_id,
                fence: binding.fence_generation,
                next_attempt: &next.attempt_id,
                next_fence: next.fence_generation,
                failed: true,
                retries: u32::try_from(record.retries.len()).map_err(host_error)?,
                cumulative_units,
                elapsed_ms,
                next_wall_budget_ms,
            },
            observed,
            &leases,
        ) {
            Ok(retry) => retry,
            Err(_) => return Ok(false),
        };
        if retry.selected.worker_endpoint_id() != selection.options.builder.endpoint_id {
            return Ok(false);
        }
        let successor = remote_build::RemoteProductionAttemptBinding {
            job_id: binding.job_id.clone(),
            attempt_id: next.attempt_id.clone(),
            fence_generation: next.fence_generation,
        };
        let predecessor_evidence = self.publish_evidence(record, &entry, None, false, None)?;
        let retained = remote_build::gateway::GatewayRetryRecord {
            predecessor: binding.clone(),
            successor: successor.clone(),
            signed_usage: record.usage.clone().ok_or(Error::Integrity)?,
            machine_class: selected.machine_class().id.clone(),
            reservation_units: selected.machine_class().reservation_units,
            observed_unix_s: observed,
            policy_version: retry.plan.policy_version,
            predecessor_evidence,
            replay_json: retry.replay_json,
        };
        remote_build::gateway::commit_retry(
            state,
            &record.public_id,
            next,
            retry.selected.worker_endpoint_id(),
            retry.selected.requirements().clone(),
            retained,
        )
        .map_err(host_error)?;
        let record = state.gateway_attempts.get_mut(&record.public_id).ok_or(Error::Integrity)?;
        let next_quota = Self::quota_binding(record)?;
        remote_build::save_coordinator_state(&self.0.ctx.resolved_state_dir, state).map_err(host_error)?;
        self.0.resources.reserve(&retry.selected, &next_quota, observed).map_err(host_error)?;
        prepared.plan.client.request.resource_requirements = Some(retry.selected.requirements().clone());
        remote_build::bind_remote_production_dispatch(
            &mut prepared.plan,
            successor.clone(),
            crate::standard_remote_transfer_policy(),
            selection.failure_debug.clone(),
            &self.0.ctx.resolved_state_dir,
        )
        .map_err(host_error)?;
        crate::configure_remote_dispatch_diagnostics(selection, &mut prepared.plan).map_err(host_error)?;
        prepared.attempt = successor;
        prepared.request = prepared.plan.client.request.clone();
        prepared.telemetry = crate::empty_remote_telemetry_buffer();
        *selected = retry.selected;
        Ok(true)
    }

    fn execute(
        &self,
        id: String,
        session: Arc<GatewaySession>,
        mut selection: RemoteBuildSelection,
        mut prepared: crate::PreparedRemoteDispatch,
        mut selected: super::resource_host::SelectedResources,
    ) -> Result<(), Error> {
        let state_dir = &self.0.ctx.resolved_state_dir;
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(host_error)?;
        loop {
            self.verify_builder_endpoint(now()?)?;
            {
                let _guard = remote_build::acquire_remote_coordinator_mutation_guard(state_dir).map_err(host_error)?;
                let mut state = remote_build::load_coordinator_state(state_dir).map_err(host_error)?;
                self.0.verifier.revalidate(&session, now()?).map_err(|_| Error::Rejected(Reject::Authority))?;
                let record = state.gateway_attempts.get_mut(&id).ok_or(Error::Unknown)?;
                if record.dispatch_started {
                    return Ok(());
                }
                if record.cancel_requested {
                    return Err(Error::Conflict);
                }
                record.dispatch_started = true;
                record.transition(GatewayPhase::Running, now()?, "gateway-worker-dispatch-started");
                let binding = record.binding.clone().ok_or(Error::Integrity)?;
                remote_build::save_coordinator_state(state_dir, &state).map_err(host_error)?;
                remote_build::start_remote_production_attempt_if_queued(&mut state, &binding, now()?)
                    .map_err(host_error)?;
            }
            selection.options.now_unix_s = now()?;
            let result = runtime.block_on(async {
                let mut store = crate::open_remote_dispatch_store(
                    &self.0.ctx.store,
                    state_dir,
                    &self.0.ctx.store_prefix,
                    self.0.ctx.store_backend,
                )
                .await
                .map_err(host_error)?;
                let mut coordinator = remote_build::load_coordinator_state(state_dir).map_err(host_error)?;
                crate::execute_remote_dispatch(crate::RemoteDispatchExecutionInput {
                    selection: &selection,
                    state_dir,
                    store: &mut store,
                    coordinator: &mut coordinator,
                    prepared: &mut prepared,
                })
                .await
                .map_err(host_error)
            });
            let _guard = remote_build::acquire_remote_coordinator_mutation_guard(state_dir).map_err(host_error)?;
            let mut state = remote_build::load_coordinator_state(state_dir).map_err(host_error)?;
            let record = state.gateway_attempts.get(&id).ok_or(Error::Unknown)?.clone();
            if result.is_err()
                && self.retry_after_oom(&mut state, &record, &session, &selection, &mut prepared, &mut selected)?
            {
                continue;
            }
            let record = state.gateway_attempts.get_mut(&id).ok_or(Error::Unknown)?;
            if result.is_err() && record.result_evidence.is_none() {
                let phase = if record.local_teardown_confirmed {
                    GatewayPhase::Cancelled
                } else {
                    GatewayPhase::Failed
                };
                record.transition(phase, now()?, "gateway-worker-terminal-failure");
                if record.usage.is_none() && !record.local_teardown_confirmed {
                    record.recovery_uncertain = true;
                }
            }
            let record = record.clone();
            remote_build::save_coordinator_state(state_dir, &state).map_err(host_error)?;
            let entry = self.reconcile(&record)?;
            if result.is_ok() && record.result_evidence.is_some() {
                // The ordinary runner has just imported and verified these
                // exact signed outputs/CAS under the current fence. Retain its
                // producer evidence without requiring a separate read first.
                let publication = self.publish_evidence(&record, &entry, None, true, None)?;
                state.gateway_attempts.get_mut(&id).ok_or(Error::Integrity)?.published_evidence = Some(publication);
                remote_build::save_coordinator_state(state_dir, &state).map_err(host_error)?;
            }
            return Ok(());
        }
    }
}

impl Services for GatewayHost {
    type Session = Arc<GatewaySession>;
    type Authorization = GatewayAuthorization;
    async fn authenticate<'a>(
        &'a self,
        session: &'a Self::Session,
        at: u64,
        request: Request<'a>,
    ) -> Result<Self::Authorization, Error> {
        let (operation, target, digest) =
            request_binding(request, session.project(), self.0.endpoint_binding.generation)?;
        self.authorize_nix_operation(session, operation, &target, digest, at)
    }

    async fn submit(
        &self,
        session: &Self::Session,
        authorization: &Self::Authorization,
        id: &str,
        drv_path: &str,
        digest: [u8; 32],
    ) -> Result<Observation, Error> {
        authorization.session_matches(session)?;
        authorization.require_target(Operation::BuildSubmit, drv_path)?;
        if *blake3::hash(drv_path.as_bytes()).as_bytes() != digest {
            return Err(Error::Integrity);
        }
        self.verify_builder_endpoint(now()?)?;
        let authority = authorization.facts();
        {
            let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
                .map_err(host_error)?;
            let state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
            if let Some(record) = state.gateway_attempts.get(id) {
                if !record.matches(authority.subject, authority.scope)
                    || record.account != session.account()
                    || record.request_digest != digest
                    || record.drv_path != drv_path
                {
                    return Err(Error::Conflict);
                }
                if record.dispatch_started
                    || record.local_teardown_confirmed
                    || record.recovery_uncertain
                    || record.reuse.is_some()
                    || record
                        .transitions
                        .last()
                        .is_some_and(|event| matches!(event.phase, GatewayPhase::Failed | GatewayPhase::Cancelled))
                {
                    return Self::observation(&state, record);
                }
            }
        }
        let store = super::open_read_authority_async(&self.0.ctx).await.map_err(host_error)?;
        let mut inputs = super::prepare_build_paths(&store, &[drv_path.into()]).await.map_err(host_error)?;
        if inputs.len() != 1 {
            return Err(Error::Integrity);
        }
        let mut input = inputs.pop().ok_or(Error::Integrity)?;
        input.label = id.into();
        let mut selection = self.0.selection.clone();
        selection.options.now_unix_s = now()?;
        let request =
            remote_build::concrete_remote_derivation_request(&input, &selection.options).map_err(host_error)?;
        let executable = remote_build::plan_remote_executable_request(&request).map_err(host_error)?;
        let features = self.0.features.get(&executable.system).ok_or(Error::Rejected(Reject::Authority))?;
        let requirements = request.resource_requirements.as_ref().ok_or(Error::Rejected(Reject::Authority))?;
        let state_dir = &self.0.ctx.resolved_state_dir;
        let guard = remote_build::acquire_remote_coordinator_mutation_guard(state_dir).map_err(host_error)?;
        let mut coordinator = remote_build::load_coordinator_state(state_dir).map_err(host_error)?;
        if let Some(record) = coordinator.gateway_attempts.get(id) {
            if !record.matches(authority.subject, authority.scope)
                || record.account != session.account()
                || record.request_digest != digest
                || record.drv_path != drv_path
            {
                return Err(Error::Conflict);
            }
            if record.dispatch_started
                || record.local_teardown_confirmed
                || record.recovery_uncertain
                || record.reuse.is_some()
            {
                return Self::observation(&coordinator, record);
            }
        } else {
            if coordinator.gateway_attempts.len() >= remote_build::gateway::MAX_GATEWAY_ATTEMPTS {
                return Err(Error::Busy);
            }
            let mut record = GatewayAttemptRecord {
                public_id: id.into(),
                subject: authority.subject.into(),
                account: session.account().into(),
                project: authority.scope.into(),
                authority_policy: session.policy_identity().into(),
                authority_decision: session.decision_id().into(),
                endpoint: session.endpoint().into(),
                platform: executable.system.clone(),
                reuse: None,
                published_evidence: None,
                transitions: Vec::new(),
                drv_path: drv_path.into(),
                request_digest: digest,
                request_id: request.request_id.clone(),
                binding: None,
                dispatch_started: false,
                cancel_requested: false,
                retries: Vec::new(),
                selection_json: None,
                local_teardown_confirmed: false,
                recovery_uncertain: false,
                result_evidence: None,
                outputs: Vec::new(),
                produced_outputs: Vec::new(),
                usage: None,
                public_log: Vec::new(),
            };
            record.transition(GatewayPhase::Queued, selection.options.now_unix_s, "gateway-admission-intent");
            coordinator.gateway_attempts.insert(id.into(), record);
            remote_build::save_coordinator_state(state_dir, &coordinator).map_err(host_error)?;
        }
        // Reuse is a consumer request, never another execution or quota charge.
        // Its own journal/public ID remains scoped to the consumer.
        let producer_ids = coordinator
            .gateway_attempts
            .values()
            .filter(|producer| {
                coordinator.gateway_attempts.get(id).is_some_and(|record| record.binding.is_none())
                    && producer.public_id != id
                    && producer.reuse.is_none()
                    && producer.request_digest == digest
                    && producer.drv_path == drv_path
                    && producer.result_evidence.is_some()
                    && producer.published_evidence.is_some()
            })
            .map(|producer| producer.public_id.clone())
            .collect::<Vec<_>>();
        for producer_id in producer_ids {
            let Ok(link) = self
                .0
                .resources
                .authorize_verified_reuse(super::resource_host::VerifiedReuseRequest {
                    producer_public_id: &producer_id,
                    coordinator: &coordinator,
                    consumer_project: authority.scope,
                    consumer_drv_path: drv_path,
                    consumer_request_digest: digest,
                    consumer_platform: &executable.system,
                    consumer_policy: session.policy_identity(),
                    store: &store,
                    trusted_output_keys: &selection.options.trusted_output_keys,
                })
                .await
            else {
                continue;
            };
            let producer = coordinator.gateway_attempts.get(&producer_id).ok_or(Error::Integrity)?;
            let evidence = producer.result_evidence.ok_or(Error::Integrity)?;
            let binding = producer.binding.clone();
            let outputs = producer.outputs.clone();
            let produced_outputs = producer.produced_outputs.clone();
            let reuse = remote_build::gateway::GatewayReuseRecord {
                producer_public_id: producer_id,
                producer_project: producer.project.clone(),
                producer_account: producer.account.clone(),
                producer_request_id: producer.request_id.clone(),
                producer_evidence: producer.published_evidence.clone().ok_or(Error::Integrity)?,
                consumer_project: authority.scope.into(),
                sharing_scope: link.scope,
                result_evidence: evidence,
            };
            let consumer = coordinator.gateway_attempts.get_mut(id).ok_or(Error::Integrity)?;
            consumer.binding = binding;
            consumer.outputs = outputs;
            consumer.produced_outputs = produced_outputs;
            consumer.result_evidence = Some(evidence);
            consumer.reuse = Some(reuse);
            consumer.transition(GatewayPhase::Succeeded, now()?, "gateway-authorized-result-reuse");
            remote_build::save_coordinator_state(state_dir, &coordinator).map_err(host_error)?;
            return Self::observation(&coordinator, coordinator.gateway_attempts.get(id).ok_or(Error::Integrity)?);
        }
        let prior = coordinator.gateway_attempts.get(id).and_then(|record| record.binding.clone());
        let leases = coordinator
            .resource_leases
            .values()
            .filter(|lease| prior.as_ref().is_none_or(|binding| lease.scope.attempt_id != binding.attempt_id))
            .cloned()
            .collect::<Vec<_>>();
        let requirements = prior
            .as_ref()
            .and_then(|binding| coordinator.jobs.get(&binding.job_id))
            .and_then(|job| job.resource_requirements.as_ref())
            .unwrap_or(requirements);
        let selected = self
            .0
            .resources
            .select(SelectionRequest {
                project: authority.scope,
                account: session.account(),
                request_id: &request.request_id,
                request_digest: &hex(&digest),
                requirements,
                features,
                public_action_kind: "nix-derivation",
                now_unix_s: selection.options.now_unix_s,
                live_leases: &leases,
            })
            .map_err(host_error)?;
        if selected.worker_endpoint_id() != selection.options.builder.endpoint_id {
            return Err(Error::Integrity);
        }
        let prepared = if let Some(binding) = prior {
            // Recover the exact queued identity, never create a second attempt
            // or lower an already-admitted retry's declared resource minimum.
            if selected.requirements() != requirements {
                return Err(Error::Integrity);
            }
            let endpoint_binding = selection.options.endpoint_binding.as_ref().ok_or(Error::Integrity)?;
            let endpoint_admission = selection
                .endpoint_binding_config
                .admit_build(endpoint_binding, now()?, &selection.options.store_prefix)
                .map_err(host_error)?;
            let mut plan =
                remote_build::plan_remote_stdio_client_dispatch(input, &selection.options).map_err(host_error)?;
            plan.client.request.resource_requirements = Some(selected.requirements().clone());
            remote_build::bind_remote_production_dispatch(
                &mut plan,
                binding.clone(),
                crate::standard_remote_transfer_policy(),
                selection.failure_debug.clone(),
                state_dir,
            )
            .map_err(host_error)?;
            crate::configure_remote_dispatch_diagnostics(&selection, &mut plan).map_err(host_error)?;
            let request = plan.client.request.clone();
            crate::PreparedRemoteDispatch {
                plan,
                attempt: binding,
                request,
                telemetry: crate::empty_remote_telemetry_buffer(),
                telemetry_policy: selection.telemetry.telemetry,
                _endpoint_admission: endpoint_admission,
            }
        } else {
            let mut scheduling_request = request.clone();
            scheduling_request.resource_requirements = Some(selected.requirements().clone());
            remote_build::gateway::require_new_execution(
                &coordinator,
                &scheduling_request,
                &selection.options.trusted_output_keys,
            )
            .map_err(host_error)?;
            crate::prepare_remote_dispatch(crate::RemoteDispatchPreparationInput {
                selection: &selection,
                state_dir,
                coordinator: &mut coordinator,
                input,
                priority_competing_goal_count: None,
                resource_requirements: Some(selected.requirements()),
            })
            .map_err(host_error)?
        };
        let record = coordinator.gateway_attempts.get_mut(id).ok_or(Error::Integrity)?;
        record.binding = Some(prepared.attempt.clone());
        let quota_binding = Self::quota_binding(record)?;
        if record.retries.is_empty() && record.selection_json.is_none() {
            record.selection_json = Some(serde_json::to_value(selected.evaluation()).map_err(host_error)?);
        }
        remote_build::save_coordinator_state(state_dir, &coordinator).map_err(host_error)?;
        let entry = self
            .0
            .resources
            .reserve(&selected, &quota_binding, selection.options.now_unix_s)
            .map_err(host_error)?;
        self.publish_evidence(
            coordinator.gateway_attempts.get(id).ok_or(Error::Integrity)?,
            &entry,
            Some(selected.evaluation()),
            false,
            None,
        )?;
        let observation =
            Self::observation(&coordinator, coordinator.gateway_attempts.get(id).ok_or(Error::Integrity)?)?;
        drop(guard);
        let host = self.clone();
        let owned_id = id.to_string();
        let session = Arc::clone(session);
        let mut active = self.0.active.lock().map_err(host_error)?;
        active.retain(|_, thread| !thread.is_finished());
        if !active.contains_key(id) {
            let task_id = owned_id.clone();
            let thread = std::thread::Builder::new()
                .name("mantle-gateway-build".into())
                .spawn(move || {
                    if host.execute(task_id.clone(), session, selection, prepared, selected).is_err() {
                        let _ = host.record_worker_error(&task_id);
                    }
                })
                .map_err(host_error)?;
            active.insert(owned_id, thread);
        }
        Ok(observation)
    }

    async fn observe(&self, authorization: &Self::Authorization, id: &str) -> Result<Observation, Error> {
        let authority = authorization.facts();
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
            .map_err(host_error)?;
        let state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
        let record = state.gateway_attempts.get(id).ok_or(Error::Unknown)?;
        if !record.matches(authority.subject, authority.scope) {
            return Err(Reject::Authority.into());
        }
        match authorization.session.operation() {
            Operation::StatusRead | Operation::EventRead | Operation::ResultRead => {
                authorization.require_target(authorization.session.operation(), id)?
            }
            // A synchronous native build may await its own execution outcome,
            // but this does not expose a status-read endpoint to the holder.
            Operation::BuildSubmit => authorization.require_target(Operation::BuildSubmit, &record.drv_path)?,
            _ => return Err(Reject::Authority.into()),
        }
        let observed = Self::observation(&state, record)?;
        if observed.phase.terminal() {
            self.reconcile(record)?;
        }
        Ok(observed)
    }

    async fn cancel(&self, authorization: &Self::Authorization, id: &str, fence: u64) -> Result<(), Error> {
        let authority = authorization.facts();
        let (operation, target, digest) = request_binding(
            Request::Cancel { attempt_id: id, fence },
            authority.scope,
            self.0.endpoint_binding.generation,
        )?;
        authorization
            .session
            .require(operation, &target, digest)
            .map_err(|_| Error::Rejected(Reject::Authority))?;
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
            .map_err(host_error)?;
        let mut state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
        let record = state.gateway_attempts.get_mut(id).ok_or(Error::Unknown)?;
        if !record.matches(authority.subject, authority.scope) {
            return Err(Reject::Authority.into());
        }
        if record.binding.as_ref().is_none_or(|binding| binding.fence_generation.get() != fence) {
            return Err(Reject::StaleFence.into());
        }
        record.cancel_requested = true;
        record.append_event("gateway-cancellation-requested");
        remote_build::save_coordinator_state(&self.0.ctx.resolved_state_dir, &state).map_err(host_error)
    }

    async fn logs(
        &self,
        authorization: &Self::Authorization,
        id: &str,
        offset: u64,
        maximum: u32,
    ) -> Result<Vec<u8>, Error> {
        let authority = authorization.facts();
        authorization.require_target(Operation::LogRead, id)?;
        if maximum == 0 || maximum > crunch_nix_gateway_core::MAX_LOG_WINDOW_BYTES {
            return Err(Reject::Bound.into());
        }
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
            .map_err(host_error)?;
        let state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
        let record = state.gateway_attempts.get(id).ok_or(Error::Unknown)?;
        if !record.matches(authority.subject, authority.scope) {
            return Err(Reject::Authority.into());
        }
        let offset = usize::try_from(offset).map_err(|_| Error::Cursor)?;
        if offset > record.public_log.len() {
            return Err(Error::Cursor);
        }
        Ok(record.public_log[offset..].iter().take(maximum as usize).copied().collect())
    }

    async fn verify_result(
        &self,
        session: &Self::Session,
        authorization: &Self::Authorization,
        id: &str,
        evidence: [u8; 32],
    ) -> Result<(), Error> {
        authorization.session_matches(session)?;
        let authority = authorization.facts();
        let record = self.completed_record(authority, id)?;
        if record.result_evidence != Some(evidence) {
            return Err(Error::Integrity);
        }
        if record.outputs.len() != record.produced_outputs.len()
            || record.produced_outputs.iter().any(|output| !record.outputs.contains(&output.logical_path))
        {
            return Err(Error::Integrity);
        }
        let (operation, target, digest) =
            request_binding(Request::Result { attempt_id: id }, authority.scope, self.0.endpoint_binding.generation)?;
        session.require(operation, &target, digest).map_err(|_| Error::Rejected(Reject::Authority))?;
        self.0.verifier.revalidate(session, now()?).map_err(|_| Error::Rejected(Reject::Authority))?;
        let store = super::open_read_authority_async(&self.0.ctx).await.map_err(host_error)?;
        remote_build::validate_retained_remote_produced_outputs(
            &record.produced_outputs,
            &hex(&evidence),
            &self.0.ctx.store_prefix,
            &self.0.selection.options.trusted_output_keys,
        )
        .map_err(|_| Error::Integrity)?;
        // This includes the current signed derivation/action-result admission,
        // then exact current signed PathInfo and measured CAS checks.
        if !store
            .query_missing(&[format!("{}!*", record.drv_path)])
            .await
            .map_err(|_| Error::Integrity)?
            .is_empty()
        {
            return Err(Error::Integrity);
        }
        let retained = record
            .produced_outputs
            .iter()
            .map(|output| output.path_info.as_ref().ok_or(Error::Integrity))
            .collect::<Result<Vec<_>, _>>()?;
        store.verify_retained_path_infos(&retained).await.map_err(|_| Error::Integrity)?;
        let entry = self.reconcile(&record)?;
        let publication = self.publish_evidence(&record, &entry, None, true, Some(session))?;
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
            .map_err(host_error)?;
        let mut state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
        let current = state.gateway_attempts.get_mut(id).ok_or(Error::Unknown)?;
        if current.result_evidence != record.result_evidence || current.binding != record.binding {
            return Err(Error::Conflict);
        }
        current.published_evidence = Some(publication);
        remote_build::save_coordinator_state(&self.0.ctx.resolved_state_dir, &state).map_err(host_error)?;
        Ok(())
    }

    async fn usage(
        &self,
        session: &Self::Session,
        authorization: &Self::Authorization,
        items: u32,
    ) -> Result<api::Usage, Error> {
        authorization.session_matches(session)?;
        let authority = authorization.facts();
        let (operation, target, digest) =
            request_binding(Request::Usage { items }, authority.scope, self.0.endpoint_binding.generation)?;
        session.require(operation, &target, digest).map_err(|_| Error::Rejected(Reject::Authority))?;
        if items == 0 || items > api::MAX_USAGE_ITEMS {
            return Err(Reject::Bound.into());
        }
        let _guard = remote_build::acquire_remote_coordinator_mutation_guard(&self.0.ctx.resolved_state_dir)
            .map_err(host_error)?;
        let state = remote_build::load_coordinator_state(&self.0.ctx.resolved_state_dir).map_err(host_error)?;
        let summary = self.0.resources.summary(authority.scope, session.account()).map_err(host_error)?;
        let mut usage = api::Usage {
            admitted: 0,
            active: 0,
            completed: 0,
            cpu_millis: 0,
            output_bytes: 0,
            accounting: Some(api::UsageAccounting {
                unit: summary.schedule.unit,
                window_id: summary.schedule.window_id,
                window_start_s: summary.schedule.window_start_s,
                window_end_s: summary.schedule.window_end_s,
                project_reserved_units: summary.totals.project_reserved,
                project_charged_units: summary.totals.project_charged,
                account_reserved_units: summary.totals.account_reserved,
                account_charged_units: summary.totals.account_charged,
                memory_measurement: summary.memory_accounting.into(),
                output_measurement: summary.output_accounting.into(),
            }),
        };
        for record in state
            .gateway_attempts
            .values()
            .filter(|record| record.matches(authority.subject, authority.scope))
            .take(items as usize)
        {
            usage.admitted += 1;
            if Self::observation(&state, record)?.phase.terminal() {
                usage.completed += 1
            } else {
                usage.active += 1
            }
            if record.reuse.is_some() {
                continue;
            }
            for retry in &record.retries {
                let measured = self.authenticated_retry_usage(record, retry)?;
                usage.cpu_millis = usage.cpu_millis.checked_add(measured.cpu_time_millis()).ok_or(Error::Integrity)?;
                usage.output_bytes = usage.output_bytes.checked_add(measured.output_bytes()).ok_or(Error::Integrity)?;
            }
            if let Some(measured) = self.authenticated_usage(record)? {
                usage.cpu_millis = usage.cpu_millis.checked_add(measured.cpu_time_millis()).ok_or(Error::Integrity)?;
                usage.output_bytes = usage.output_bytes.checked_add(measured.output_bytes()).ok_or(Error::Integrity)?;
            }
        }
        Ok(usage)
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum GatewayAction {
    /// Serve only verified Nix store reads on the existing private socket.
    Read,
    /// Serve bounded asynchronous requests on an explicit owner-private socket.
    Serve {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        ticket_fd: i32,
    },
    /// Print the exact public tuple digest which a holder invocation must sign.
    Binding {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        project: String,
        /// Persistent receiver generation, distinct from Basalt policy generation.
        #[arg(long)]
        endpoint_generation: u64,
    },
    /// Send one bounded credential envelope without printing its credentials.
    Call {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        envelope: PathBuf,
    },
    /// Serve one Nix connection with one explicitly signed concrete build grant.
    NixOnce {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        ticket_fd: i32,
        #[arg(long)]
        envelope: PathBuf,
    },
    /// Serve one exactly authorized bounded signed NAR import, never a shell.
    NixImportOnce {
        #[arg(long)]
        authority_config: PathBuf,
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        envelope: PathBuf,
        #[arg(long)]
        staging_directory: PathBuf,
    },
    /// Serve one exact holder-authorized Nix store query on a private socket.
    NixStoreReadOnce {
        #[arg(long)]
        authority_config: PathBuf,
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        envelope: PathBuf,
    },
    /// Print the public binding for one exact store path, opcode and generation.
    NixStoreReadBinding {
        #[arg(long)]
        path: String,
        #[arg(long, value_enum)]
        operation: super::native::StoreReadOperation,
        #[arg(long)]
        endpoint_generation: u64,
    },
    /// Print the public operation digest for an exact NAR header and byte bound.
    NixUploadBinding {
        #[arg(long)]
        header: PathBuf,
        #[arg(long)]
        max_bytes: u64,
    },
    /// Explicitly initialize an empty owner-private quota ledger, never keys.
    ProvisionQuota {
        #[arg(long)]
        directory: PathBuf,
    },
    /// Run the real isolated quota journal fault fixture; not a VM campaign.
    QuotaFaultCampaign {
        #[arg(long)]
        directory: PathBuf,
        #[arg(long)]
        seed: u64,
    },
}

impl GatewayAction {
    fn operation(&self) -> GatewayOperation {
        match self {
            Self::Read => GatewayOperation::Read,
            Self::NixImportOnce { .. } => GatewayOperation::NixImportOnce,
            Self::NixStoreReadOnce { .. } => GatewayOperation::NixStoreReadOnce,
            Self::NixStoreReadBinding { .. } => GatewayOperation::NixStoreReadBinding,
            Self::NixUploadBinding { .. } => GatewayOperation::NixUploadBinding,
            Self::QuotaFaultCampaign { .. } => GatewayOperation::QuotaFaultCampaign,
            Self::ProvisionQuota { .. } => GatewayOperation::ProvisionQuota,
            Self::Binding { .. } => GatewayOperation::Binding,
            Self::Call { .. } => GatewayOperation::Call,
            Self::Serve { .. } => GatewayOperation::Serve,
            Self::NixOnce { .. } => GatewayOperation::NixOnce,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum WireRequest {
    Submit {
        drv_path: String,
        idempotency_key: String,
    },
    Status {
        attempt_id: String,
    },
    Logs {
        attempt_id: String,
        cursor: Option<String>,
        bytes: u32,
    },
    Events {
        attempt_id: String,
        cursor: Option<String>,
        items: u32,
    },
    Cancel {
        attempt_id: String,
        fence: u64,
    },
    Result {
        attempt_id: String,
    },
    Usage {
        items: u32,
    },
}
impl WireRequest {
    fn borrowed(&self) -> Request<'_> {
        match self {
            Self::Submit {
                drv_path,
                idempotency_key,
            } => Request::Submit {
                drv_path,
                idempotency_key,
            },
            Self::Status { attempt_id } => Request::Status { attempt_id },
            Self::Logs {
                attempt_id,
                cursor,
                bytes,
            } => Request::Logs {
                attempt_id,
                cursor: cursor.as_deref(),
                bytes: *bytes,
            },
            Self::Events {
                attempt_id,
                cursor,
                items,
            } => Request::Events {
                attempt_id,
                cursor: cursor.as_deref(),
                items: *items,
            },
            Self::Cancel { attempt_id, fence } => Request::Cancel {
                attempt_id,
                fence: *fence,
            },
            Self::Result { attempt_id } => Request::Result { attempt_id },
            Self::Usage { items } => Request::Usage { items: *items },
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    authority: super::authority::OperationRequest,
    compact_token: String,
    nonce: [u8; 32],
    holder_signature: Vec<u8>,
    request: WireRequest,
}
impl Drop for Envelope {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.compact_token.zeroize();
        self.holder_signature.zeroize();
        self.nonce.zeroize();
    }
}

const MAX_ENVELOPE_BYTES: usize = 128 * 1024;
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const PRIVATE_IO_SECS: u64 = 10;

fn private_metadata(path: &Path, directory: bool) -> Result<std::fs::Metadata, Error> {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::symlink_metadata(path).map_err(host_error)?;
    if (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(Reject::Authority.into());
    }
    Ok(metadata)
}

fn private_bytes(path: &Path, maximum: usize) -> Result<zeroize::Zeroizing<Vec<u8>>, Error> {
    use std::io::Read;
    use std::os::unix::fs::OpenOptionsExt;
    private_metadata(path, false)?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(host_error)?;
    let metadata = file.metadata().map_err(host_error)?;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;
    if !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > maximum as u64
    {
        return Err(Reject::Authority.into());
    }
    let mut bytes = zeroize::Zeroizing::new(Vec::new());
    file.take(maximum as u64 + 1).read_to_end(&mut bytes).map_err(host_error)?;
    if bytes.len() > maximum {
        return Err(Reject::Bound.into());
    }
    Ok(bytes)
}

fn verify_envelope(host: &GatewayHost, envelope: &mut Envelope) -> Result<Arc<GatewaySession>, Error> {
    let (operation, target, digest) =
        request_binding(envelope.request.borrowed(), &envelope.authority.project, host.0.endpoint_binding.generation)?;
    if operation != envelope.authority.operation
        || target != envelope.authority.target
        || digest != envelope.authority.request_digest
    {
        return Err(Reject::Authority.into());
    }
    let signature = basalt::ucan::SignerSignature::try_from_slice(&envelope.holder_signature)
        .map_err(|_| Error::Rejected(Reject::Authority))?;
    let nonce = basalt::ucan::HolderInvocationNonce::from_bytes(envelope.nonce);
    let request = std::mem::replace(&mut envelope.authority, super::authority::OperationRequest {
        operation,
        subject: String::new(),
        account: String::new(),
        project: String::new(),
        endpoint: String::new(),
        audience: String::new(),
        generation: 0,
        request_digest: [0; 32],
        target: String::new(),
    });
    host.verifier()
        .verify(
            request,
            super::authority::Invocation {
                compact_token: &envelope.compact_token,
                nonce,
                holder_signature: &signature,
            },
            now()?,
        )
        .map(Arc::new)
        .map_err(|_| Error::Rejected(Reject::Authority))
}

pub(super) struct SocketGuard {
    path: PathBuf,
    dev: u64,
    pub(super) ino: u64,
}
impl Drop for SocketGuard {
    fn drop(&mut self) {
        use std::os::unix::fs::MetadataExt;
        if let Ok(metadata) = std::fs::symlink_metadata(&self.path)
            && metadata.dev() == self.dev
            && metadata.ino() == self.ino
        {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
pub(super) fn bind_socket(path: &Path) -> Result<(std::os::unix::net::UnixListener, SocketGuard), Error> {
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;
    private_metadata(path.parent().ok_or(Error::Storage)?, true)?;
    let listener = std::os::unix::net::UnixListener::bind(path).map_err(host_error)?;
    let metadata = std::fs::symlink_metadata(path).map_err(host_error)?;
    let guard = SocketGuard {
        path: path.into(),
        dev: metadata.dev(),
        ino: metadata.ino(),
    };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(host_error)?;
    listener.set_nonblocking(true).map_err(host_error)?;
    Ok((listener, guard))
}
fn admit_peer(stream: &std::os::unix::net::UnixStream) -> Result<(), Error> {
    use std::os::fd::AsRawFd;
    let mut credentials: libc::ucred = unsafe { std::mem::zeroed() };
    let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let status = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut size,
        )
    };
    if status != 0
        || size as usize != std::mem::size_of::<libc::ucred>()
        || credentials.uid != unsafe { libc::geteuid() }
    {
        return Err(Reject::Authority.into());
    }
    stream.set_read_timeout(Some(std::time::Duration::from_secs(PRIVATE_IO_SECS))).map_err(host_error)?;
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(PRIVATE_IO_SECS)))
        .map_err(host_error)?;
    Ok(())
}

fn receive_frame(stream: &mut impl std::io::Read, maximum: usize) -> Result<zeroize::Zeroizing<Vec<u8>>, Error> {
    let mut length = [0_u8; 4];
    stream.read_exact(&mut length).map_err(host_error)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > maximum {
        return Err(Reject::Bound.into());
    }
    let mut bytes = zeroize::Zeroizing::new(vec![0; length]);
    stream.read_exact(&mut bytes).map_err(host_error)?;
    Ok(bytes)
}
fn send_frame(stream: &mut impl std::io::Write, bytes: &[u8]) -> Result<(), Error> {
    let length = u32::try_from(bytes.len()).map_err(|_| Error::Rejected(Reject::Bound))?;
    stream
        .write_all(&length.to_be_bytes())
        .and_then(|()| stream.write_all(bytes))
        .and_then(|()| stream.flush())
        .map_err(host_error)
}

fn load_remote_selection(
    ctx: &RunContext,
    config: &GatewayConfig,
    ticket_fd: i32,
) -> Result<RemoteBuildSelection, Error> {
    use remote_build::endpoint_config::BuildEndpointBindingConfig;
    use remote_build::endpoint_config::RemoteEndpointBindingConfig;
    let binding_config = RemoteEndpointBindingConfig::from_build(
        config.builder_endpoint_state_directory.clone(),
        BuildEndpointBindingConfig {
            endpoint_binding: config.builder_endpoint_binding.clone(),
            scope: config.builder_endpoint_scope.clone(),
            trust_requirement: config.builder_endpoint_trust_requirement.clone(),
        },
    )
    .map_err(host_error)?;
    let secret_request = crate::remote_service_secrets::RemoteServiceSecretRequest {
        manifest_path: config.secret_manifest.clone(),
        profile: config.secret_profile.clone(),
        provider: config.secret_provider.clone(),
    };
    crate::remote_build_selection(
        crate::RemoteBuildSelectionInput {
            builder: Some(&config.endpoint),
            ticket_fd: Some(ticket_fd),
            builder_program: Some(&config.builder_program),
            builder_args: &config.builder_args,
            trusted_builder_keys: &config.trusted_builder_keys,
            secret_request: &secret_request,
            build_time_limit_secs: config.build_time_limit_secs,
            remote_delta: false,
            observability_config_path: Some(&config.observability_config),
        },
        ctx,
        Some(Arc::new(binding_config)),
    )
    .map_err(host_error)?
    .ok_or(Error::Host)
}

fn open_service(ctx: &RunContext, path: &Path, ticket_fd: i32) -> Result<(GatewayHost, api::Api<GatewayHost>), Error> {
    let bytes = private_bytes(path, 1024 * 1024)?;
    let config: GatewayConfig = serde_json::from_slice(&bytes).map_err(host_error)?;
    private_metadata(&config.journal_directory, true)?;
    private_metadata(&config.quota_directory, true)?;
    let selection = load_remote_selection(ctx, &config, ticket_fd)?;
    let verifier = GatewayVerifier::open(GatewayAuthorityConfig {
        authority_file: config.authority.authority_file.clone(),
        replay_directory: config.authority.replay_directory.clone(),
    })
    .map_err(|_| Error::Rejected(Reject::Authority))?;
    let resources = ResourcePolicyHost::open(config.resources.clone(), &config.quota_directory).map_err(host_error)?;
    let secret = zeroize::Zeroizing::new(
        <[u8; 32]>::try_from(private_bytes(&config.cursor_secret_file, 32)?.as_slice()).map_err(host_error)?,
    );
    let key = zeroize::Zeroizing::new(
        <[u8; 32]>::try_from(private_bytes(&config.completion_key_file, 32)?.as_slice()).map_err(host_error)?,
    );
    let signer = ed25519_dalek::SigningKey::from_bytes(&key);
    let host = GatewayHost::new(ctx.clone(), &config, selection, verifier, resources)?;
    host.recover()?;
    let api =
        api::Api::open(&config.journal_directory, *secret, signer, &config.audience, config.policy, host.clone())?;
    Ok((host, api))
}

fn call_private(socket: &Path, envelope: &Path) -> Result<GatewayOperationResult, Error> {
    let bytes = private_bytes(envelope, MAX_ENVELOPE_BYTES)?;
    let mut stream = std::os::unix::net::UnixStream::connect(socket).map_err(host_error)?;
    admit_peer(&stream)?;
    send_frame(&mut stream, &bytes)?;
    let response = receive_frame(&mut stream, MAX_RESPONSE_BYTES)?;
    // Only the OS-authenticated endpoint's JSON response may be rendered;
    // credentials never cross stdout and an error envelope is not success.
    let response_bytes = response.len();
    let response: serde_json::Value = serde_json::from_slice(&response).map_err(host_error)?;
    let evidence = GatewayOperationEvidence::Call {
        envelope_bytes: bytes.len(),
        response_bytes,
        version: response.get("version").and_then(serde_json::Value::as_u64),
        has_response: response.get("response").is_some_and(|value| !value.is_null()),
        has_error: response.get("error").is_some(),
    };
    Ok(GatewayOperationResult {
        evidence,
        output: Some(serde_json::to_string(&response).map_err(host_error)?),
    })
}

pub(crate) fn command(ctx: &RunContext, action: GatewayAction) -> Result<(), crate::errors::RunError> {
    command_inner(ctx, action).map_err(|error| crate::errors::RunError::Internal(error.code().into()))
}
fn command_inner(ctx: &RunContext, action: GatewayAction) -> Result<(), Error> {
    let plan = plan_gateway_operation(action.operation()).map_err(host_error)?;
    let result = GatewayCliPort { ctx }.execute(&plan, action)?;
    if classify_gateway_operation(&plan, &result.evidence) != ApplicationOutcome::Completed {
        return Err(Error::Integrity);
    }
    if let Some(output) = result.output {
        println!("{output}");
    }
    Ok(())
}

struct GatewayCliPort<'a> {
    ctx: &'a RunContext,
}

impl GatewayOperationPort for GatewayCliPort<'_> {
    type Request = GatewayAction;
    type Error = Error;

    fn execute(&mut self, plan: &GatewayOperationPlan, action: GatewayAction) -> Result<GatewayOperationResult, Error> {
        if plan.operation != action.operation() {
            return Err(Error::Integrity);
        }
        let ctx = self.ctx;
        match action {
            GatewayAction::Read => Ok(GatewayOperationResult {
                evidence: super::serve_private(ctx).map_err(host_error)?,
                output: None,
            }),
            GatewayAction::NixImportOnce {
                authority_config,
                socket,
                envelope,
                staging_directory,
            } => Ok(GatewayOperationResult {
                evidence: super::native::serve_import_once(
                    ctx,
                    &authority_config,
                    &socket,
                    &envelope,
                    &staging_directory,
                )
                .map_err(host_error)?,
                output: None,
            }),
            GatewayAction::NixStoreReadOnce {
                authority_config,
                socket,
                envelope,
            } => Ok(GatewayOperationResult {
                evidence: super::native::serve_store_read_once(ctx, &authority_config, &socket, &envelope)
                    .map_err(host_error)?,
                output: None,
            }),
            GatewayAction::NixStoreReadBinding {
                path,
                operation,
                endpoint_generation,
            } => {
                let binding =
                    super::native::store_read_binding(&path, operation, endpoint_generation).map_err(host_error)?;
                Ok(GatewayOperationResult {
                    evidence: GatewayOperationEvidence::PureBinding,
                    output: Some(serde_json::to_string(&binding).map_err(host_error)?),
                })
            }
            GatewayAction::NixUploadBinding { header, max_bytes } => {
                let (binding, bytes_read) =
                    super::native::upload_binding_header(&header, max_bytes).map_err(host_error)?;
                Ok(GatewayOperationResult {
                    evidence: GatewayOperationEvidence::BindingInput {
                        bytes_read,
                        maximum: MAX_ENVELOPE_BYTES,
                    },
                    output: Some(serde_json::to_string(&binding).map_err(host_error)?),
                })
            }
            GatewayAction::QuotaFaultCampaign { directory, seed } => {
                let (output, evidence) = super::campaign::run_quota_campaign(&directory, seed).map_err(host_error)?;
                Ok(GatewayOperationResult {
                    evidence,
                    output: Some(serde_json::to_string(&output).map_err(host_error)?),
                })
            }
            GatewayAction::ProvisionQuota { directory } => {
                private_metadata(&directory, true)?;
                crunch_build::distributed::ResourceQuotaStore::provision(&directory).map_err(host_error)?;
                let ledger = crunch_build::distributed::ResourceQuotaStore::open(&directory)
                    .map_err(host_error)?
                    .snapshot()
                    .map_err(host_error)?;
                Ok(GatewayOperationResult {
                    evidence: GatewayOperationEvidence::QuotaProvision {
                        ledger_schema: ledger.schema,
                        expected_schema: crunch_remote_core::resource_quota::LEDGER_SCHEMA,
                    },
                    output: Some("{\"schema\":\"mantle-gateway-quota-v1\",\"state\":\"provisioned\"}".into()),
                })
            }
            GatewayAction::Binding {
                request,
                project,
                endpoint_generation,
            } => {
                let bytes = private_bytes(&request, api::MAX_REQUEST_BYTES)?;
                let bytes_read = bytes.len();
                let request: WireRequest = serde_json::from_slice(&bytes).map_err(host_error)?;
                let (operation, target, digest) = request_binding(request.borrowed(), &project, endpoint_generation)?;
                let output = serde_json::to_string(
                    &serde_json::json!({"operation":operation,"target":target,"request_digest":digest}),
                )
                .map_err(host_error)?;
                Ok(GatewayOperationResult {
                    evidence: GatewayOperationEvidence::BindingInput {
                        bytes_read,
                        maximum: api::MAX_REQUEST_BYTES,
                    },
                    output: Some(output),
                })
            }
            GatewayAction::Call { socket, envelope } => call_private(&socket, &envelope),
            GatewayAction::Serve {
                config,
                socket,
                ticket_fd,
            } => {
                let (host, api) = open_service(ctx, &config, ticket_fd)?;
                let (listener, socket_guard) = bind_socket(&socket)?;
                let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(host_error)?;
                let mut term = {
                    let _entered = runtime.enter();
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).map_err(host_error)?
                };
                let shutdown_requested = loop {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            if admit_peer(&stream).is_err() {
                                continue;
                            }
                            let outcome = (|| {
                                let bytes = receive_frame(&mut stream, MAX_ENVELOPE_BYTES)?;
                                let mut envelope: Envelope = serde_json::from_slice(&bytes).map_err(host_error)?;
                                if envelope.version != api::API_VERSION {
                                    return Err(Error::Rejected(Reject::Version));
                                }
                                let session = verify_envelope(&host, &mut envelope)?;
                                runtime.block_on(api.handle(
                                    envelope.version,
                                    &session,
                                    now()?,
                                    envelope.request.borrowed(),
                                ))
                            })();
                            let response = match outcome {
                                Ok(response) => serde_json::json!({"version":1,"response":response}),
                                Err(error) => serde_json::json!({"version":1,"error":error.code()}),
                            };
                            if let Ok(bytes) = serde_json::to_vec(&response)
                                && bytes.len() <= MAX_RESPONSE_BYTES
                            {
                                let _ = send_frame(&mut stream, &bytes);
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            let stop = runtime.block_on(async {
                                tokio::select! {
                                    _ = tokio::signal::ctrl_c() => true,
                                    _ = term.recv() => true,
                                    _ = tokio::time::sleep(std::time::Duration::from_millis(25)) => false,
                                }
                            });
                            if stop {
                                break true;
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => return Err(Error::Host),
                    }
                };
                let active = std::mem::take(&mut *host.0.active.lock().map_err(host_error)?);
                let workers_started = active.len();
                let mut workers_joined = 0;
                for (_, thread) in active {
                    thread.join().map_err(host_error)?;
                    workers_joined += 1;
                }
                Ok(GatewayOperationResult {
                    evidence: GatewayOperationEvidence::Service {
                        socket_inode: socket_guard.ino,
                        shutdown_requested,
                        workers_started,
                        workers_joined,
                    },
                    output: None,
                })
            }
            GatewayAction::NixOnce {
                config,
                socket,
                ticket_fd,
                envelope,
            } => {
                let (host, api) = open_service(ctx, &config, ticket_fd)?;
                let bytes = private_bytes(&envelope, MAX_ENVELOPE_BYTES)?;
                let mut envelope: Envelope = serde_json::from_slice(&bytes).map_err(host_error)?;
                if envelope.version != api::API_VERSION {
                    return Err(Reject::Version.into());
                }
                let session = verify_envelope(&host, &mut envelope)?;
                let WireRequest::Submit { drv_path, .. } = &envelope.request else {
                    return Err(Reject::Operation.into());
                };
                let expected_drv = drv_path.clone();
                let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(host_error)?;
                let store = runtime.block_on(super::open_read_authority_async(ctx)).map_err(host_error)?;
                let (listener, _socket_guard) = bind_socket(&socket)?;
                listener.set_nonblocking(false).map_err(host_error)?;
                let (stream, _) = listener.accept().map_err(host_error)?;
                admit_peer(&stream)?;
                let mut requests = 0_u64;
                let mut completed = 0_u64;
                let mut failed = 0_u64;
                let mut missing_result_evidence = 0_u64;
                let mut missing_outputs = 0_usize;
                let mut first_rejection = None;
                crunch_nix_gateway::server::serve_build_connection(stream, &store, &runtime, |proven| {
                    requests = requests.saturating_add(1);
                    let outcome = (|| {
                        if proven.len() != 1 {
                            return Err(Reject::Bound);
                        }
                        let (path, _, _) = proven.into_iter().next().ok_or(Reject::Bound)?.into_derivations();
                        if path.to_absolute_path() != expected_drv {
                            return Err(Reject::Authority);
                        }
                        let response = runtime
                            .block_on(api.handle(
                                1,
                                &session,
                                now().map_err(|_| Reject::Authority)?,
                                envelope.request.borrowed(),
                            ))
                            .map_err(|_| Reject::Authority)?;
                        let api::Response::Submitted { attempt_id, .. } = response else {
                            return Err(Reject::Operation);
                        };
                        loop {
                            let authorization = host
                                .authorize_nix_operation(
                                    &session,
                                    session.operation(),
                                    session.target(),
                                    session.request_digest(),
                                    now().map_err(|_| Reject::Authority)?,
                                )
                                .map_err(|_| Reject::Authority)?;
                            let observed = runtime
                                .block_on(host.observe(&authorization, &attempt_id))
                                .map_err(|_| Reject::Authority)?;
                            if observed.phase == Phase::Succeeded {
                                let missing = runtime
                                    .block_on(store.query_missing(&[format!("{}!*", expected_drv)]))
                                    .map_err(|_| Reject::Authority)?;
                                missing_outputs = missing_outputs.saturating_add(missing.len());
                                if !missing.is_empty() {
                                    return Err(Reject::Authority);
                                }
                                if observed.result_evidence.is_none() {
                                    missing_result_evidence = missing_result_evidence.saturating_add(1);
                                    return Err(Reject::Authority);
                                }
                                completed = completed.saturating_add(1);
                                return Ok(());
                            }
                            if observed.phase.terminal() {
                                return Err(Reject::Operation);
                            }
                            std::thread::sleep(std::time::Duration::from_millis(25));
                        }
                    })();
                    if let Err(reason) = outcome {
                        failed = failed.saturating_add(1);
                        first_rejection.get_or_insert(reason);
                    }
                    outcome
                })
                .map_err(host_error)?;
                if let Some(reason) = first_rejection {
                    return Err(reason.into());
                }
                Ok(GatewayOperationResult {
                    evidence: GatewayOperationEvidence::Build {
                        requests,
                        completed,
                        failed,
                        missing_result_evidence,
                        missing_outputs,
                    },
                    output: None,
                })
            }
        }
    }
}

#[cfg(test)]
#[path = "api_host_tests.rs"]
mod tests;
