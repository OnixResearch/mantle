//! Building-plane projection of an admitted placement blocker. The daemon only
//! relays this value; no consumer of this module can infer or grant capacity.

use crunch_remote_core::demand::DemandBook;
use crunch_remote_core::demand::DemandChange;
use crunch_remote_core::demand::DemandError;
use crunch_remote_core::demand::NamedQuantity;
use crunch_remote_core::demand::Requirement;
use crunch_remote_core::demand::Retraction;
use crunch_remote_core::demand::WorkerDemand;

use super::RemoteCoordinatorBuildRequest;
use super::RemoteCoordinatorDispatchDecision;
use super::RemoteCoordinatorState;

/// Pure projection from the same validated request and actual placement result
/// used by admission. A terminal rejection has no outstanding request lifetime.
pub fn derive_worker_demand(
    request: &RemoteCoordinatorBuildRequest,
    decision: &RemoteCoordinatorDispatchDecision,
) -> Result<Option<WorkerDemand>, String> {
    let RemoteCoordinatorDispatchDecision::Pending { reason } = decision else {
        return Ok(None);
    };
    super::validate_coordinator_build_request(request)?;
    let Some(resources) = &request.resource_requirements else {
        return Ok(None);
    };
    let canonical = crunch_build::distributed::canonical_remote_resource_requirements(resources)
        .map_err(|reason| reason.as_str().to_string())?;
    let quantities = canonical.quantities;
    let map_named = |values: Vec<crunch_build::distributed::RemoteNamedResourceQuantity>| {
        values
            .into_iter()
            .map(|value| NamedQuantity {
                name: value.name,
                quantity: value.quantity,
            })
            .collect()
    };
    let requirement = Requirement {
        cpu_units: quantities.cpu_units,
        memory_bytes: quantities.memory_bytes,
        scratch_bytes: quantities.scratch_bytes,
        accelerators: map_named(quantities.accelerators),
        named_tokens: map_named(quantities.named_tokens),
        semantic_accelerator_classes: canonical.semantic_accelerator_classes,
        placement_key_blake3: super::normalized_remote_build_key(request)?,
    };
    WorkerDemand::new(
        &request.request.request_id,
        crunch_build::scheduling::DEFAULT_SCHEDULING_POLICY_ID,
        requirement,
        reason,
    )
    .map(Some)
    .map_err(|error| format!("remote-worker-demand-{error:?}"))
}

pub(super) fn observe_dispatch(
    book: &mut DemandBook,
    request: &RemoteCoordinatorBuildRequest,
    decision: &RemoteCoordinatorDispatchDecision,
) -> Result<DemandChange, String> {
    match derive_worker_demand(request, decision)? {
        Some(fact) => book.observe_unmet(fact).map_err(|error| format!("remote-worker-demand-{error:?}")),
        None => Ok(book.retract(&request.request.request_id, Retraction::Placed)),
    }
}

/// The owner calls this on cancellation/timeout/withdrawal. It does not alter a
/// job, attempt, reservation, lease, inventory, or placement decision.
pub fn retract_worker_demand(state: &mut RemoteCoordinatorState, request_id: &str, cause: Retraction) -> DemandChange {
    state.worker_demands.retract(request_id, cause)
}

pub fn validate_published_demand(fact: &WorkerDemand) -> Result<(), DemandError> {
    fact.validate()
}

/// A planning observation grants no capacity and never installs a demand owner.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct PlacementReport {
    pub blocker_reason: Option<String>,
    pub worker_demand: Option<WorkerDemand>,
}

pub fn load_declared_requirements(path: &std::path::Path) -> Result<super::RemoteResourceRequirements, String> {
    use std::io::Read as _;
    const MAX_REQUIREMENT_BYTES: u64 = 16_384;
    let file = std::fs::File::open(path).map_err(|error| format!("remote-resource-requirements-read: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_REQUIREMENT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("remote-resource-requirements-read: {error}"))?;
    if bytes.len() as u64 > MAX_REQUIREMENT_BYTES {
        return Err("remote-resource-requirements-size-limit".into());
    }
    let declared =
        serde_json::from_slice(&bytes).map_err(|error| format!("remote-resource-requirements-json: {error}"))?;
    super::canonical_remote_resource_requirements(&declared).map_err(|reason| reason.as_str().to_owned())
}

/// Read the current admitted snapshot without recovery, migration or writes.
/// Planning must not install registration, attempts, leases, or demand facts.
pub(crate) fn plan_production_placement(
    state_dir: &std::path::Path,
    request: &super::ConcreteBuildRequest,
    selection: &crate::RemoteBuildSelection,
) -> Result<PlacementReport, String> {
    let path = super::coordinator_state_path(state_dir);
    let state = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| format!("remote-plan-state-invalid: {error}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => super::RemoteCoordinatorState::default(),
        Err(error) => return Err(format!("remote-plan-state-read: {error}")),
    };
    let input = super::RemoteProductionDispatchInput {
        request,
        endpoint_id: &selection.options.builder.endpoint_id,
        worker_generation: selection.worker_generation,
        worker_concurrency: selection.worker_concurrency,
        resource_inventory: selection.resource_inventory.clone(),
        trusted_output_keys: &selection.options.trusted_output_keys,
        now_unix_s: selection.options.now_unix_s,
    };
    let executable = super::plan_remote_executable_request(request)?;
    let registration = super::production_worker_registration(&state, &input, &executable.system);
    let (candidate, _, _, _) = super::worker_registration_candidate(&state, registration)?;
    let coordinator_request =
        super::production_coordinator_request(request, input.trusted_output_keys, &executable.system);
    // A readonly plan never exposes or consumes hypothetical assignment IDs.
    let nonce = crunch_remote_core::attempt::RemoteAssignmentNonce::new(
        blake3::hash(request.request_id.as_bytes()).to_hex().to_string(),
    )
    .map_err(|reason| reason.as_str().to_owned())?;
    let decision = super::plan_coordinator_dispatch(&candidate, &coordinator_request, &nonce)?;
    let blocker_reason = match &decision {
        RemoteCoordinatorDispatchDecision::Pending { reason }
        | RemoteCoordinatorDispatchDecision::Reject { reason } => Some(reason.clone()),
        _ => None,
    };
    let worker_demand = derive_worker_demand(&coordinator_request, &decision)?;
    Ok(PlacementReport {
        blocker_reason,
        worker_demand,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_requirements_are_typed_bounded_and_canonical() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("requirements.json");
        std::fs::write(
            &path,
            r#"{"quantities":{"cpu_units":2,"memory_bytes":4096,"scratch_bytes":8192,
                 "accelerators":[],"named_tokens":[{"name":"z-token","quantity":1},{"name":"a-token","quantity":2}]},
                 "semantic_accelerator_classes":[]}"#,
        )
        .unwrap();
        let admitted = load_declared_requirements(&path).unwrap();
        assert_eq!(admitted.quantities.cpu_units, 2);
        assert_eq!(admitted.quantities.memory_bytes, 4096);
        assert_eq!(admitted.quantities.scratch_bytes, 8192);
        assert_eq!(admitted.quantities.named_tokens[0].name, "a-token");
        assert_eq!(admitted.quantities.named_tokens[1].name, "z-token");
        for rejected in [
            r#"{"quantities":{"cpu_units":1.5,"memory_bytes":4096,"scratch_bytes":8192}}"#,
            r#"{"quantities":{"cpu_units":1048577,"memory_bytes":4096,"scratch_bytes":8192}}"#,
            r#"{"quantities":{"cpu_units":2,"memory_bytes":4096,"scratch_bytes":8192},"concurrency":64}"#,
        ] {
            std::fs::write(&path, rejected).unwrap();
            assert!(load_declared_requirements(&path).is_err(), "untyped or inferred declaration admitted");
        }
        std::fs::write(&path, vec![b' '; 16_385]).unwrap();
        assert_eq!(load_declared_requirements(&path).unwrap_err(), "remote-resource-requirements-size-limit");
    }
}
