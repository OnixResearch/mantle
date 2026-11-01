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
use crate::remote_build::run_stdio_remote_child;
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
        resource_requirements: None,
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

fn pool_error(code: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, format!("remote-pool:{code}"))
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
        bind_remote_production_dispatch(
            &mut plan,
            attempt.clone(),
            endpoint.transfer_policy,
            endpoint.failure_debug.clone(),
            &endpoint.state_dir,
        )
        .map_err(|error| pool_error(&error))?;
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
        let transcript = run_stdio_remote_child(&plan.command, &request_id, &mut session)
            .map_err(|error| pool_error(&error.to_string()))?;
        let admission =
            validate_remote_stdio_output_import(&plan.client.request, &plan.client.trusted_output_keys, &transcript)
                .map_err(|error| pool_error(&error))?;

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
        let mut outputs = Vec::with_capacity(goal.request.outputs.len());
        // Content-addressed outputs have no declared path before the build, so
        // the request carries placeholders while the worker reports realized
        // paths. Match those by result order; input-addressed outputs match by
        // the declared logical path.
        let derivation = crate::remote_build::parse_remote_derivation_json(&goal.derivation_json)
            .map_err(|error| pool_error(&format!("derivation-json:{error}")))?;
        let declared_paths = derivation
            .outputs
            .values()
            .filter_map(|output| output.path.as_ref())
            .map(|path| path.to_absolute_path_with_prefix(&endpoint.options.store_prefix))
            .collect::<BTreeSet<_>>();
        let requested_paths = goal
            .request
            .outputs
            .iter()
            .map(|relative| {
                format!(
                    "{}/{}",
                    endpoint.options.store_prefix.trim_end_matches('/'),
                    relative.to_string_lossy().trim_start_matches('/')
                )
            })
            .collect::<Vec<_>>();
        let all_declared = requested_paths.iter().all(|path| declared_paths.contains(path));
        if !all_declared && imported.len() == goal.request.outputs.len() {
            outputs.extend(imported.iter().map(|(_, node)| BuildOutput {
                node: node.clone(),
                output_needles: BTreeSet::new(),
            }));
        } else {
            for relative in &goal.request.outputs {
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
        }
        if outputs.len() != goal.request.outputs.len() {
            return Err(pool_error("output-count-mismatch"));
        }

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
                &attempt,
                &admission.output_digest_blake3,
                ready_unix_s,
            )
            .map_err(|error| pool_error(&format!("result-ready:{error}")))?;
            complete_remote_production_attempt(coordinator, &attempt, &admission.output_digest_blake3, completed_unix_s)
                .map_err(|error| pool_error(&format!("completion:{error}")))
        })?;
        Ok(BuildResult {
            outputs,
            log: Some(transcript.stderr_summary),
        })
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
        })
        .expect("fixture goal")
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
