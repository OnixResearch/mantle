//! Shell adapters for explicit build-planning observations and accepted effects.

// r[impl realization_routing.explicit_observation_boundary]
// r[impl realization_routing.plan_execution_separation]
// r[impl realization_routing.typed_planning_blockers]
// r[impl build_scheduling.explicit_parallelism_facts]
pub mod execution_adapter;
pub mod observation_adapter;
pub mod remote_adapter;

#[cfg(test)]
mod tests;
