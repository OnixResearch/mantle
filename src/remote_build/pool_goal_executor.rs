//! Bin-side executor for one prepared pool goal.
//!
//! The worker is a stateless action executor: this module builds the concrete
//! request from the prepared goal, admits a client-side attempt, runs one framed
//! stdio exchange, verifies the returned outputs, and completes the attempt.
//! It owns no placement policy and persists no output bytes; the pipeline's
//! Builder stores the returned nodes.

use std::collections::BTreeSet;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use crunch_build::distributed::PoolGoalExecutor;
use crunch_build::distributed::PreparedPoolGoal;
use crunch_build::distributed::RemoteFailureDebugPolicy;
use crunch_build::distributed::RemoteTransferPolicy;
use crunch_build::distributed::RemoteWorkerResourceInventory;
use crunch_remote_core::effect::EffectActor;
use crunch_remote_core::effect::EffectSession;
use crunch_remote_core::effect::RemoteEffectBinding;
use snix_build::buildservice::BuildOutput;
use snix_build::buildservice::BuildResult;

use crate::remote_build::ConcreteBuildRequest;
use crate::remote_build::RemoteChildKillSlot;
use crate::remote_build::RemoteClientBuildOptions;
use crate::remote_build::RemoteConcreteBuildPayload;
use crate::remote_build::admit_remote_production_dispatch;
use crate::remote_build::bind_remote_production_dispatch;
use crate::remote_build::complete_remote_production_attempt;
use crate::remote_build::load_coordinator_state;
use crate::remote_build::plan_remote_stdio_request_dispatch;
use crate::remote_build::prepare_remote_production_input_transfer;
use crate::remote_build::remote_client_request_id;
use crate::remote_build::remote_expected_outputs_from_derivation;
use crate::remote_build::remote_input_ref_upload_bytes;
use crate::remote_build::remote_source_input_refs_from_derivation;
use crate::remote_build::save_coordinator_state;
use crate::remote_build::validate_remote_stdio_output_import;

/// One admitted endpoint's executor inputs.
#[derive(Clone)]
pub struct PoolEndpointDispatch {
    pub options: RemoteClientBuildOptions,
    pub worker_generation: u64,
    pub worker_concurrency: u32,
    pub resource_inventory: Option<RemoteWorkerResourceInventory>,
    pub failure_debug: RemoteFailureDebugPolicy,
    pub transfer_policy: RemoteTransferPolicy,
    pub state_dir: PathBuf,
    pub transfer: Option<crunch_store::OwnedTransferObjectStore>,
}

impl std::fmt::Debug for PoolEndpointDispatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PoolEndpointDispatch")
            .field("endpoint", &self.options.builder.endpoint_id)
            .field("worker_generation", &self.worker_generation)
            .finish_non_exhaustive()
    }
}

/// Executes each goal on this endpoint's transport.
pub struct RemotePoolGoalExecutor {
    endpoint: PoolEndpointDispatch,
}

impl RemotePoolGoalExecutor {
    pub fn new(endpoint: PoolEndpointDispatch) -> Self {
        Self { endpoint }
    }
}

/// Build the concrete request for one prepared goal from its carried derivation.
pub fn concrete_pool_goal_request(
    goal: &PreparedPoolGoal,
    options: &RemoteClientBuildOptions,
) -> Result<ConcreteBuildRequest, String> {
    let derivation = crate::remote_build::parse_remote_derivation_json(&goal.derivation_json)
        .map_err(|error| format!("remote-pool-derivation-json-invalid: {error}"))?;
    let drv_path = nix_compat::store_path::StorePath::<String>::from_absolute_path_with_prefix(
        goal.drv_path.as_bytes(),
        &options.store_prefix,
    )
    .map_err(|_| "remote-pool-derivation-path-invalid".to_string())?;
    let expected_outputs = remote_expected_outputs_from_derivation(&derivation, &options.store_prefix)?;
    let input_refs = pool_goal_input_refs(goal, &derivation, &options.store_prefix)?;
    let source_input_refs = remote_source_input_refs_from_derivation(&derivation, &options.store_prefix);
    let upload_bytes = remote_input_ref_upload_bytes(&input_refs)?;
    let request_id = remote_client_request_id(&goal.action_ref, &drv_path, &options.store_prefix);
    let request = ConcreteBuildRequest {
        request_id,
        endpoint_binding: options.endpoint_binding.clone(),
        store_prefix: options.store_prefix.clone(),
        input_refs,
        source_input_refs,
        upload_bytes,
        build_time_limit_secs: options.build_time_limit_secs,
        contains_raw_frontend_eval: false,
        payload: RemoteConcreteBuildPayload::NixDerivation {
            drv_path: goal.drv_path.clone(),
            drv_json: goal.derivation_json.clone(),
            closure: goal
                .closure
                .iter()
                .map(|entry| crate::remote_build::RemoteClosureEntry {
                    drv_path: entry.drv_path.clone(),
                    hash_derivation_modulo: entry.hash_derivation_modulo.clone(),
                    derivation_json: entry.derivation_json.clone(),
                    outputs: entry
                        .outputs
                        .iter()
                        .map(|output| crate::remote_build::RemoteClosureOutput {
                            name: output.name.clone(),
                            logical_path: output.logical_path.clone(),
                        })
                        .collect(),
                })
                .collect(),
        },
        expected_outputs,
        production_attempt: None,
        transfer_policy: None,
        resource_requirements: goal.resource_requirements.clone(),
        locality_scope: None,
        failure_debug_policy: RemoteFailureDebugPolicy::default(),
        failure_replay: None,
    };
    debug_assert_eq!(request.store_prefix, options.store_prefix);
    debug_assert!(request.input_refs.len() <= request.input_refs.capacity().max(request.input_refs.len()));
    Ok(request)
}

/// Resolve the store paths a pool goal actually needs on the worker.
///
/// The derivation closure travels in the payload, so a dependency contributes
/// its built outputs (which the client imported) rather than its `.drv` path,
/// which is not a stored object in either store.
fn pool_goal_input_refs(
    goal: &PreparedPoolGoal,
    derivation: &nix_compat::derivation::Derivation,
    store_prefix: &str,
) -> Result<Vec<String>, String> {
    let mut closure = std::collections::BTreeMap::new();
    for entry in &goal.closure {
        let parsed = crate::remote_build::parse_remote_derivation_json(&entry.derivation_json)
            .map_err(|error| format!("remote-pool-closure-json-invalid: {error}"))?;
        closure.insert(entry.drv_path.clone(), (parsed, entry.outputs.as_slice()));
    }
    let mut refs =
        Vec::with_capacity(derivation.input_derivations.len().saturating_add(derivation.input_sources.len()));
    for (drv_path, output_names) in &derivation.input_derivations {
        let absolute = drv_path.to_absolute_path_with_prefix(store_prefix);
        let (input_derivation, resolved_outputs) =
            closure.get(&absolute).ok_or_else(|| format!("remote-pool-input-derivation-missing:{absolute}"))?;
        for output_name in output_names {
            let output = input_derivation
                .outputs
                .get(output_name)
                .ok_or_else(|| format!("remote-pool-input-output-missing:{absolute}:{output_name}"))?;
            match &output.path {
                Some(path) => refs.push(path.to_absolute_path_with_prefix(store_prefix)),
                None => {
                    let resolved = resolved_outputs
                        .iter()
                        .find(|candidate| candidate.name == *output_name)
                        .ok_or_else(|| format!("remote-pool-input-output-unresolved:{absolute}:{output_name}"))?;
                    refs.push(resolved.logical_path.clone());
                }
            }
        }
    }
    refs.extend(derivation.input_sources.iter().map(|path| path.to_absolute_path_with_prefix(store_prefix)));
    refs.sort();
    refs.dedup();
    Ok(refs)
}

/// Test seam: corrupt one streamed NAR byte before ingest so the executor's
/// size/digest verification is exercised end to end.
#[cfg(debug_assertions)]
fn tamper_pool_output_nar_for_test(action: &crate::remote_build::RemoteOutputImportAction) {
    if std::env::var_os("MANTLE_TEST_TAMPER_POOL_OUTPUT_NAR").is_none() {
        return;
    }
    let Some(path) = action.nar_path.as_ref() else {
        return;
    };
    let Ok(mut bytes) = std::fs::read(path) else {
        return;
    };
    if bytes.len() < 2 {
        return;
    }
    // Flip a payload byte, not the NAR header, so the format still parses and
    // the digest comparison is what rejects the artifact.
    let index = bytes.len() / 2;
    bytes[index] ^= 0xFF;
    let _ = std::fs::write(path, bytes);
}

/// Match the worker's admitted outputs to the goal's requested outputs.
///
/// Content-addressed outputs have no declared path before the build, so the
/// request carries placeholders while the worker reports realized paths. Those
/// match by result order; input-addressed outputs match by declared logical
/// path. A count difference or an unmatched requested output fails closed.
fn match_pool_outputs(
    requested: &[PathBuf],
    imported: &[(String, snix_castore::Node)],
    derivation: &nix_compat::derivation::Derivation,
    store_prefix: &str,
) -> Result<Vec<BuildOutput>, io::Error> {
    if imported.len() != requested.len() {
        return Err(pool_error("output-count-mismatch"));
    }
    let declared_paths = derivation
        .outputs
        .values()
        .filter_map(|output| output.path.as_ref())
        .map(|path| path.to_absolute_path_with_prefix(store_prefix))
        .collect::<BTreeSet<_>>();
    let requested_paths = requested
        .iter()
        .map(|relative| {
            format!("{}/{}", store_prefix.trim_end_matches('/'), relative.to_string_lossy().trim_start_matches('/'))
        })
        .collect::<Vec<_>>();
    let all_declared = requested_paths.iter().all(|path| declared_paths.contains(path));
    let mut outputs = Vec::with_capacity(requested.len());
    if all_declared {
        for relative in requested {
            let suffix = format!("/{}", relative.to_string_lossy().trim_start_matches('/'));
            let matched = imported
                .iter()
                .find(|(logical_path, _)| logical_path.ends_with(&suffix))
                .ok_or_else(|| pool_error(&format!("output-missing:{suffix}")))?;
            outputs.push(BuildOutput {
                node: matched.1.clone(),
                output_needles: BTreeSet::new(),
            });
        }
    } else {
        outputs.extend(imported.iter().map(|(_, node)| BuildOutput {
            node: node.clone(),
            output_needles: BTreeSet::new(),
        }));
    }
    debug_assert_eq!(outputs.len(), requested.len());
    Ok(outputs)
}

fn pool_error(code: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, format!("remote-pool:{code}"))
}

/// Whether the coordinator still owns the admitted attempt. The durable
/// completion transition is authoritative; this check only prevents exporting
/// output bytes for a job that has already moved to a newer fence.
fn ensure_pool_attempt_is_current(
    state_dir: &Path,
    attempt: &crate::remote_build::RemoteProductionAttemptBinding,
) -> io::Result<()> {
    let coordinator =
        crate::remote_build::load_coordinator_state(state_dir).map_err(|_| pool_error("attempt-state-unavailable"))?;
    let current = coordinator.jobs.get(&attempt.job_id).and_then(|job| job.current_attempt.as_ref());
    match current {
        Some(current)
            if current.attempt_id == attempt.attempt_id && current.fence_generation == attempt.fence_generation =>
        {
            Ok(())
        }
        _ => Err(pool_error("attempt-stale")),
    }
}

/// Test seam: reassign the admitted attempt to a successor fence between
/// admission and completion so a stale completion path is exercised end to end.
#[cfg(debug_assertions)]
fn reassign_pool_attempt_for_test(
    endpoint: &PoolEndpointDispatch,
    attempt: &crate::remote_build::RemoteProductionAttemptBinding,
) -> io::Result<()> {
    if std::env::var_os("MANTLE_TEST_REASSIGN_POOL_ATTEMPT").is_none() {
        return Ok(());
    }
    let retry_policy = crunch_remote_core::attempt::RemoteAttemptRetryPolicy::default();
    let now_unix_s = endpoint.options.now_unix_s.saturating_add(retry_policy.retry_delay_secs).saturating_add(1);
    save_coordinator_with_retry(&endpoint.state_dir, |coordinator| {
        let decision = crate::remote_build::reassign_coordinator_attempt(
            coordinator,
            &attempt.job_id,
            &endpoint.options.builder.endpoint_id,
            crunch_remote_core::attempt::RemoteAttemptFailureClass::Retryable,
            retry_policy,
            crunch_remote_core::attempt::RemoteAttemptTimeFacts {
                now_unix_s,
                failure_observed_unix_s: endpoint.options.now_unix_s,
                overall_deadline_unix_s: now_unix_s.saturating_add(retry_policy.attempt_timeout_secs),
            },
        )
        .map_err(|error| pool_error(&format!("test-reassign:{error}")))?;
        match decision {
            crate::remote_build::RemoteCoordinatorDispatchDecision::Dispatch { .. } => Ok(()),
            _ => Err(pool_error("test-reassign-not-dispatched")),
        }
    })
    .map(|_| ())
}

const MAX_COORDINATOR_SAVE_ATTEMPTS: u32 = 16;

/// Serializes this process's coordinator transactions. Revision fencing still
/// protects external writers, but two goals of one build must not spin on each
/// other's intermediate candidate saves.
static COORDINATOR_TRANSACTION_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The coordinator store fences every write with the persisted revision. A
/// concurrent goal can advance it between our load and save; the caller reloads
/// and reapplies its own transition instead of overwriting the other job.
fn coordinator_save_conflict(error: &crate::errors::RunError) -> bool {
    matches!(
        error,
        crate::errors::RunError::Internal(message) if message.contains("remote-coordinator-save-conflict")
    )
}

fn save_coordinator_with_retry<F>(
    state_dir: &Path,
    mut apply: F,
) -> io::Result<crate::remote_build::RemoteCoordinatorState>
where
    F: FnMut(&mut crate::remote_build::RemoteCoordinatorState) -> io::Result<()>,
{
    let _transaction = COORDINATOR_TRANSACTION_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut attempt_number = 0;
    loop {
        attempt_number += 1;
        let mut coordinator = load_coordinator_state(state_dir)
            .map_err(|error| io::Error::other(format!("remote-pool:coordinator-load:{error}")))?;
        if let Err(error) = apply(&mut coordinator) {
            // Admission persists its own candidate; a concurrent goal can
            // advance the revision between load and that save.
            if error.to_string().contains("remote-coordinator-save-conflict") {
                if attempt_number >= MAX_COORDINATOR_SAVE_ATTEMPTS {
                    return Err(io::Error::other(format!("remote-pool:coordinator-save-conflict-exhausted:{error}")));
                }
                continue;
            }
            return Err(error);
        }
        match save_coordinator_state(state_dir, &coordinator) {
            Ok(()) => return Ok(coordinator),
            Err(error) if coordinator_save_conflict(&error) => {
                if attempt_number >= MAX_COORDINATOR_SAVE_ATTEMPTS {
                    return Err(io::Error::other(format!("remote-pool:coordinator-save-conflict-exhausted:{error}")));
                }
            }
            Err(error) => {
                return Err(io::Error::other(format!("remote-pool:coordinator-save:{error}")));
            }
        }
    }
}

#[async_trait::async_trait]
impl PoolGoalExecutor for RemotePoolGoalExecutor {
    async fn execute(&self, goal: PreparedPoolGoal) -> io::Result<BuildResult> {
        self.execute_with_kill(goal, None).await
    }

    async fn execute_cancellable(
        &self,
        goal: PreparedPoolGoal,
        mut cancellation: tokio::sync::watch::Receiver<bool>,
    ) -> io::Result<BuildResult> {
        if *cancellation.borrow() {
            return Err(pool_error("cancelled-before-dispatch"));
        }
        let kill: RemoteChildKillSlot = Arc::new(Mutex::new(None));
        let cancel_kill = Arc::clone(&kill);
        let cancel_task = tokio::spawn(async move {
            if cancellation.changed().await.is_err() {
                return;
            }
            // The runner publishes its process-group leader just after spawn;
            // poll briefly so a cancellation racing the spawn still lands.
            for _ in 0..20 {
                let pid = cancel_kill.lock().ok().and_then(|slot| *slot);
                if let Some(pid) = pid {
                    // SAFETY: the runner created a new process group with this
                    // leader, so the negative pid targets only that group.
                    unsafe {
                        libc::kill(-pid, libc::SIGKILL);
                    }
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        });
        let result = self.execute_with_kill(goal, Some(kill)).await;
        cancel_task.abort();
        result
    }
}

impl RemotePoolGoalExecutor {
    async fn execute_with_kill(
        &self,
        goal: PreparedPoolGoal,
        kill: Option<RemoteChildKillSlot>,
    ) -> io::Result<BuildResult> {
        let endpoint = &self.endpoint;
        let request = concrete_pool_goal_request(&goal, &endpoint.options).map_err(|error| pool_error(&error))?;
        let mut plan = plan_remote_stdio_request_dispatch(request.clone(), &endpoint.options)
            .map_err(|error| pool_error(&error))?;
        let attempt = {
            let mut admitted = None;
            save_coordinator_with_retry(&endpoint.state_dir, |coordinator| {
                let binding = admit_remote_production_dispatch(
                    coordinator,
                    &plan.client.request,
                    &endpoint.options.builder.endpoint_id,
                    endpoint.worker_generation,
                    endpoint.worker_concurrency,
                    endpoint.resource_inventory.clone(),
                    &plan.client.trusted_output_keys,
                    endpoint.options.now_unix_s,
                )
                .map_err(|error| pool_error(&format!("admission:{error}")))?;
                admitted = Some(binding);
                Ok(())
            })?;
            admitted.ok_or_else(|| pool_error("admission-binding-missing"))?
        };
        let mut settlement = AttemptSettlement::new(endpoint, attempt.clone());
        let result = self.run_admitted_goal(&goal, &mut plan, &attempt, kill.as_ref()).await;
        if result.is_ok() {
            settlement.disarm();
        }
        result
    }
}

impl RemotePoolGoalExecutor {
    async fn run_admitted_goal(
        &self,
        goal: &PreparedPoolGoal,
        plan: &mut crate::remote_build::RemoteClientDispatchPlan,
        attempt: &crate::remote_build::RemoteProductionAttemptBinding,
        kill: Option<&RemoteChildKillSlot>,
    ) -> io::Result<BuildResult> {
        let endpoint = &self.endpoint;
        bind_remote_production_dispatch(
            plan,
            attempt.clone(),
            endpoint.transfer_policy,
            endpoint.failure_debug.clone(),
            &endpoint.state_dir,
        )
        .map_err(|error| pool_error(&error))?;
        #[cfg(debug_assertions)]
        reassign_pool_attempt_for_test(endpoint, attempt)?;
        let transfer = endpoint.transfer.as_ref().ok_or_else(|| pool_error("transfer-capability-missing"))?;
        prepare_remote_production_input_transfer(
            transfer,
            &mut plan.command,
            &plan.client.request,
            &endpoint.state_dir,
        )
        .await
        .map_err(|error| pool_error(&error))?;

        let request_id = plan.client.request.request_id.clone();
        let mut session = EffectSession::for_actor_attempt(
            &request_id,
            RemoteEffectBinding {
                job_id: &attempt.job_id,
                attempt_id: &attempt.attempt_id,
                fence_generation: attempt.fence_generation,
            },
            EffectActor::Client,
        )
        .map_err(|reason| pool_error(&format!("effect-{reason:?}")))?;
        let transcript =
            crate::remote_build::run_stdio_remote_child_with_kill(&plan.command, &request_id, &mut session, kill)
                .map_err(|error| pool_error(&error.to_string()))?;
        let admission =
            validate_remote_stdio_output_import(&plan.client.request, &plan.client.trusted_output_keys, &transcript)
                .map_err(|error| pool_error(&error))?;
        // A reassigned job must not materialize a stale attempt's outputs. The
        // completion below re-checks under the coordinator guard; this earlier
        // check avoids exporting bytes the coordinator will refuse.
        ensure_pool_attempt_is_current(&endpoint.state_dir, attempt)?;

        let transfer = endpoint.transfer.as_ref().ok_or_else(|| pool_error("transfer-capability-missing"))?;
        let actions = match transcript.streaming_output.as_ref() {
            Some(streaming) => admission
                .outputs
                .iter()
                .map(|output| {
                    crate::remote_build::plan_streamed_remote_output_import_action(
                        &plan.client.request,
                        output,
                        streaming,
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| pool_error(&error))?,
            None => crate::remote_build::plan_remote_output_import_actions(&plan.client.request, &admission)
                .map_err(|error| pool_error(&error))?,
        };
        let mut imported: Vec<(String, snix_castore::Node)> = Vec::with_capacity(actions.len());
        for action in &actions {
            #[cfg(debug_assertions)]
            tamper_pool_output_nar_for_test(action);
            let payload = match (&action.nar_payload, &action.nar_path) {
                (Some(payload), _) => payload.clone(),
                (None, Some(path)) => {
                    let observed = std::fs::metadata(path)
                        .map_err(|error| pool_error(&format!("output-nar-metadata:{error}")))?
                        .len();
                    if observed != action.path_info.nar_size {
                        return Err(pool_error(&format!("output-nar-size-mismatch:{}", action.output_name)));
                    }
                    std::fs::read(path).map_err(|error| pool_error(&format!("output-nar-read:{error}")))?
                }
                (None, None) => {
                    return Err(pool_error(&format!("output-nar-payload-missing:{}", action.output_name)));
                }
            };
            let mut cursor = std::io::Cursor::new(payload.as_slice());
            let (node, nar_sha256, nar_size) = transfer
                .ingest_nar(&mut cursor, &action.path_info.ca)
                .await
                .map_err(|error| pool_error(&format!("output-nar-ingest:{error}")))?;
            if nar_sha256 != action.path_info.nar_sha256 || nar_size != action.path_info.nar_size {
                return Err(pool_error(&format!("output-nar-digest-mismatch:{}", action.output_name)));
            }
            transfer
                .export_node_to_output(&node, &action.logical_path)
                .await
                .map_err(|error| pool_error(&format!("output-export:{error}")))?;
            imported.push((action.logical_path.clone(), node));
        }
        let derivation = crate::remote_build::parse_remote_derivation_json(&goal.derivation_json)
            .map_err(|error| pool_error(&format!("derivation-json:{error}")))?;
        let outputs =
            match_pool_outputs(&goal.request.outputs, &imported, &derivation, &endpoint.options.store_prefix)?;

        let mut ready_unix_s =
            crate::remote_build::remote_cli_now_unix_s(&request_id).map_err(|error| pool_error(&error))?;
        // The start event used the admission clock; later events must be strictly
        // ordered even when the exchange finishes within the same second.
        if ready_unix_s <= endpoint.options.now_unix_s {
            ready_unix_s = endpoint.options.now_unix_s.saturating_add(1);
        }
        let completed_unix_s = ready_unix_s.saturating_add(1);
        save_coordinator_with_retry(&endpoint.state_dir, |coordinator| {
            crate::remote_build::mark_remote_production_result_ready(
                coordinator,
                attempt,
                &admission.output_digest_blake3,
                ready_unix_s,
            )
            .map_err(|error| pool_error(&format!("result-ready:{error}")))?;
            complete_remote_production_attempt(coordinator, attempt, &admission.output_digest_blake3, completed_unix_s)
                .map_err(|error| pool_error(&format!("completion:{error}")))
        })?;
        Ok(BuildResult {
            outputs,
            log: Some(transcript.stderr_summary),
        })
    }
}

/// Settles an admitted attempt unless the goal finished successfully.
///
/// The guard runs on ordinary error returns and when the executor future is
/// dropped for cancellation, so an attempt cannot stay Running after its goal
/// is gone. Settlement is best effort: the original outcome remains reported.
struct AttemptSettlement<'a> {
    endpoint: &'a PoolEndpointDispatch,
    attempt: crate::remote_build::RemoteProductionAttemptBinding,
    armed: bool,
}

impl<'a> AttemptSettlement<'a> {
    fn new(endpoint: &'a PoolEndpointDispatch, attempt: crate::remote_build::RemoteProductionAttemptBinding) -> Self {
        Self {
            endpoint,
            attempt,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for AttemptSettlement<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let endpoint = self.endpoint;
        let mut failed_unix_s = crate::remote_build::remote_cli_now_unix_s("pool-failure-settlement")
            .unwrap_or_else(|_| endpoint.options.now_unix_s.saturating_add(1));
        if failed_unix_s <= endpoint.options.now_unix_s {
            failed_unix_s = endpoint.options.now_unix_s.saturating_add(1);
        }
        let _ = save_coordinator_with_retry(&endpoint.state_dir, |coordinator| {
            crate::remote_build::fail_remote_production_attempt(coordinator, &self.attempt, failed_unix_s)
                .map_err(|error| pool_error(&format!("failure-settlement:{error}")))
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crunch_build::distributed::PreparedPoolGoal;
    use crunch_build::distributed::PreparedPoolGoalFacts;
    use crunch_remote_core::resource::Capacity;

    use super::*;

    fn options() -> RemoteClientBuildOptions {
        RemoteClientBuildOptions {
            store_prefix: "/mantle/store".to_string(),
            endpoint_binding: None,
            ticket: crate::remote_build::RemoteTicketCredential {
                ticket_id: "ticket-1".to_string(),
                secret: "secret".to_string(),
            },
            builder: crate::remote_build::RemoteStdioBuilderCommand {
                endpoint_id: "worker-a".to_string(),
                program: PathBuf::from("/bin/mantle"),
                args: vec!["remote".to_string(), "serve".to_string()],
            },
            trusted_output_keys: vec!["builder-key".to_string()],
            now_unix_s: 1,
            build_time_limit_secs: 60,
            client_endpoint: None,
            transfer_capabilities: crate::remote_build::RemoteTransferCapabilities::full_only(),
        }
    }

    fn goal(drv_path: &str, derivation_json: &str) -> PreparedPoolGoal {
        PreparedPoolGoal::new(PreparedPoolGoalFacts {
            request: snix_build::buildservice::BuildRequest::default(),
            action_ref: "action:fixture".to_string(),
            drv_path: drv_path.to_string(),
            derivation_json: derivation_json.to_string(),
            closure: Vec::new(),
            system: "x86_64-linux".to_string(),
            required_features: Vec::new(),
            required_sandbox_mode: "practical".to_string(),
            required_network_mode: "none".to_string(),
            semantic_accelerator_classes: Vec::new(),
            requested: Capacity {
                cpu_units: 0,
                memory_bytes: 0,
                scratch_bytes: 0,
            },
            resource_requirements: None,
        })
        .expect("fixture goal")
    }

    fn output_node() -> snix_castore::Node {
        snix_castore::Node::File {
            digest: snix_castore::B3Digest::from(blake3::hash(b"output")),
            size: 1,
            executable: false,
        }
    }

    fn derivation_with_output_path(path: &str) -> nix_compat::derivation::Derivation {
        crate::remote_build::parse_remote_derivation_json(&format!(
            r#"{{"args":[],"builder":"/bin/sh","env":{{}},"inputDrvs":{{}},"inputSrcs":[],"outputs":{{"out":{{"path":"{path}"}}}},"system":"x86_64-linux"}}"#
        ))
        .expect("fixture derivation")
    }

    #[test]
    fn pool_output_matching_rejects_missing_extra_and_wrong_outputs() {
        let name = "00000000000000000000000000000000-out";
        let derivation = derivation_with_output_path(&format!("/nix/store/{name}"));
        let requested = vec![PathBuf::from(name)];
        let imported = vec![(format!("/mantle/store/{name}"), output_node())];
        let matched = match_pool_outputs(&requested, &imported, &derivation, "/mantle/store").expect("declared match");
        assert_eq!(matched.len(), 1);

        let missing = match_pool_outputs(&requested, &[], &derivation, "/mantle/store").unwrap_err();
        assert!(missing.to_string().contains("output-count-mismatch"), "{missing}");

        let extra = match_pool_outputs(
            &requested,
            &[
                imported[0].clone(),
                (format!("/mantle/store/{name}-extra"), output_node()),
            ],
            &derivation,
            "/mantle/store",
        )
        .unwrap_err();
        assert!(extra.to_string().contains("output-count-mismatch"), "{extra}");

        let wrong = match_pool_outputs(
            &requested,
            &[("/mantle/store/11111111111111111111111111111111-other".to_string(), output_node())],
            &derivation,
            "/mantle/store",
        )
        .unwrap_err();
        assert!(wrong.to_string().contains("output-missing"), "{wrong}");
    }

    #[test]
    fn pool_output_matching_uses_result_order_for_content_addressed_outputs() {
        let derivation = crate::remote_build::parse_remote_derivation_json(
            r#"{"args":[],"builder":"/bin/sh","env":{},"inputDrvs":{},"inputSrcs":[],"outputs":{"out":{"path":null}},"system":"x86_64-linux"}"#,
        )
        .expect("CA derivation");
        let requested = vec![PathBuf::from("00000000000000000000000000000000-placeholder")];
        let imported = vec![("/mantle/store/11111111111111111111111111111111-realized".to_string(), output_node())];
        let outputs = match_pool_outputs(&requested, &imported, &derivation, "/mantle/store").expect("CA order match");
        assert_eq!(outputs.len(), 1);
    }

    #[test]
    fn pool_goal_request_requires_a_parseable_derivation_and_matching_prefix() {
        let malformed = goal("/mantle/store/00000000000000000000000000000000-fixture.drv", "{");
        let error = concrete_pool_goal_request(&malformed, &options()).expect_err("malformed derivation denied");
        assert!(error.contains("remote-pool-derivation-json-invalid"), "{error}");

        let empty = goal(
            "/mantle/store/00000000000000000000000000000000-fixture.drv",
            r#"{"args":[],"builder":"/bin/sh","env":{},"inputDrvs":{},"inputSrcs":[],"outputs":{},"system":"x86_64-linux"}"#,
        );
        let error = concrete_pool_goal_request(&empty, &options()).expect_err("outputless derivation denied");
        assert!(error.contains("remote-client-outputs-empty"), "{error}");

        let foreign = goal(
            "/other/store/00000000000000000000000000000000-fixture.drv",
            r#"{"args":[],"builder":"/bin/sh","env":{},"inputDrvs":{},"inputSrcs":[],"outputs":{},"system":"x86_64-linux"}"#,
        );
        let error = concrete_pool_goal_request(&foreign, &options()).expect_err("foreign prefix denied");
        assert!(error.contains("remote-pool-derivation-path-invalid"), "{error}");

        assert!(BTreeMap::<String, String>::new().is_empty());
    }
}
