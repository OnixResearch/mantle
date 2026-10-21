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
