use alloc::string::String;
use alloc::vec::Vec;

use crate::CargoOracleRequest;
use crate::CompilerInspectionRequest;
use crate::RustPlanCapability;
use crate::RustPlanPortError;
use crate::RustPlanPortSet;
use crate::WorkspaceFactsRequest;

const SUCCESS_STATUS: &str = "success";
const REUSED_STATUS: &str = "reused";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustPlanApplicationCommand {
    pub workspace_hint: String,
    pub compiler_hint: String,
    pub target_triple: String,
    pub request: mantle_rust_plan_core::RustPlanRequest,
    pub use_cargo_oracle: bool,
    pub execute_units: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustPlanApplicationOutcome {
    pub plan: mantle_rust_plan_core::RustPlan,
    pub unit_outcomes: Vec<mantle_rust_plan_core::RustUnitOutcome>,
    pub execution_blockers: Vec<mantle_rust_plan_core::RustPlanBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustPlanApplicationFailure {
    Core(mantle_rust_plan_core::RustPlanCoreError),
    Port(RustPlanPortError),
    CapabilityMismatch,
}

impl From<mantle_rust_plan_core::RustPlanCoreError> for RustPlanApplicationFailure {
    fn from(error: mantle_rust_plan_core::RustPlanCoreError) -> Self {
        Self::Core(error)
    }
}

pub fn run(
    command: RustPlanApplicationCommand,
    ports: &mut RustPlanPortSet<'_>,
) -> Result<RustPlanApplicationOutcome, RustPlanApplicationFailure> {
    let workspace = ports
        .workspace
        .load_workspace(WorkspaceFactsRequest {
            workspace_hint: command.workspace_hint.clone(),
        })
        .map_err(|error| port_failure(RustPlanCapability::WorkspaceFacts, error))?;
    let compiler = ports
        .compiler
        .inspect_compiler(CompilerInspectionRequest {
            compiler_hint: command.compiler_hint.clone(),
            target_triple: command.target_triple.clone(),
        })
        .map_err(|error| port_failure(RustPlanCapability::CompilerInspection, error))?;
    let oracle = capture_oracle(&command, &workspace.facts.workspace_identity, ports)?;
    debug_assert!(!workspace.facts.workspace_identity.is_empty());
    debug_assert!(!compiler.fact.compiler_identity.is_empty());
    let admitted = mantle_rust_plan_core::admit_workspace(
        mantle_rust_plan_core::RustPlanningInput {
            workspace: workspace.facts,
            toolchain: compiler.fact,
            oracle,
        },
        command.request,
    )?;
    let plan = mantle_rust_plan_core::plan_workspace(admitted)?;
    if !command.execute_units || !plan.receipt_preimage.blockers.is_empty() {
        return Ok(RustPlanApplicationOutcome {
            execution_blockers: plan.receipt_preimage.blockers.clone(),
            plan,
            unit_outcomes: Vec::new(),
        });
    }
    execute_plan(plan, ports)
}

fn capture_oracle(
    command: &RustPlanApplicationCommand,
    workspace_identity: &str,
    ports: &mut RustPlanPortSet<'_>,
) -> Result<Option<mantle_rust_plan_core::CargoOracleFacts>, RustPlanApplicationFailure> {
    if !command.use_cargo_oracle {
        return Ok(None);
    }
    let observation = ports
        .cargo_oracle
        .capture_oracle(CargoOracleRequest {
            workspace_identity: workspace_identity.into(),
            profile: command.request.profile.clone(),
        })
        .map_err(|error| port_failure(RustPlanCapability::CargoOracle, error))?;
    debug_assert!(!observation.facts.metadata_identity.is_empty());
    debug_assert!(!observation.facts.unit_graph_identity.is_empty());
    Ok(Some(observation.facts))
}

fn execute_plan(
    plan: mantle_rust_plan_core::RustPlan,
    ports: &mut RustPlanPortSet<'_>,
) -> Result<RustPlanApplicationOutcome, RustPlanApplicationFailure> {
    let unit_count = plan.receipt_preimage.units.len();
    let mut outcomes = Vec::with_capacity(unit_count);
    let mut blockers = Vec::with_capacity(unit_count);
    for unit in &plan.receipt_preimage.units {
        let decision = cache_decision(&unit.effect, ports)?;
        let outcome = match decision {
            mantle_rust_plan_core::CacheDecision::Reuse { artifacts } => {
                mantle_rust_plan_core::reused_unit_outcome(&unit.effect, artifacts)?
            }
            mantle_rust_plan_core::CacheDecision::Execute => execute_unit(&unit.effect, ports)?,
            mantle_rust_plan_core::CacheDecision::Block { blocker } => {
                blockers.push(blocker);
                break;
            }
        };
        let is_succeeded = outcome.status == SUCCESS_STATUS || outcome.status == REUSED_STATUS;
        if outcome.status == SUCCESS_STATUS {
            publish_outcome(&unit.effect, &outcome, ports)?;
        }
        if let Some(blocker) = &outcome.blocker {
            blockers.push(blocker.clone());
        }
        outcomes.push(outcome);
        if !is_succeeded {
            break;
        }
    }
    debug_assert!(outcomes.len() <= plan.receipt_preimage.units.len());
    debug_assert!(blockers.len() <= plan.receipt_preimage.units.len());
    Ok(RustPlanApplicationOutcome {
        plan,
        unit_outcomes: outcomes,
        execution_blockers: blockers,
    })
}

fn cache_decision(
    effect: &mantle_rust_plan_core::RustUnitEffect,
    ports: &mut RustPlanPortSet<'_>,
) -> Result<mantle_rust_plan_core::CacheDecision, RustPlanApplicationFailure> {
    let observation = ports
        .cache
        .lookup(crate::CacheLookupRequest { effect: effect.clone() })
        .map_err(|error| port_failure(RustPlanCapability::RustCache, error))?;
    mantle_rust_plan_core::classify_cache_observation(effect, observation.observation).map_err(Into::into)
}

fn execute_unit(
    effect: &mantle_rust_plan_core::RustUnitEffect,
    ports: &mut RustPlanPortSet<'_>,
) -> Result<mantle_rust_plan_core::RustUnitOutcome, RustPlanApplicationFailure> {
    let observation = ports
        .executor
        .execute_unit(crate::UnitExecutionRequest { effect: effect.clone() })
        .map_err(|error| port_failure(RustPlanCapability::UnitExecution, error))?;
    mantle_rust_plan_core::classify_unit_observation(effect, observation.observation).map_err(Into::into)
}

fn publish_outcome(
    effect: &mantle_rust_plan_core::RustUnitEffect,
    outcome: &mantle_rust_plan_core::RustUnitOutcome,
    ports: &mut RustPlanPortSet<'_>,
) -> Result<(), RustPlanApplicationFailure> {
    ports
        .cache
        .publish(crate::CachePublishRequest {
            effect: effect.clone(),
            outcome: outcome.clone(),
        })
        .map_err(|error| port_failure(RustPlanCapability::RustCache, error))?;
    debug_assert_eq!(outcome.status, SUCCESS_STATUS);
    debug_assert!(outcome.blocker.is_none());
    Ok(())
}

fn port_failure(expected: RustPlanCapability, error: RustPlanPortError) -> RustPlanApplicationFailure {
    if error.capability != expected {
        return RustPlanApplicationFailure::CapabilityMismatch;
    }
    debug_assert_eq!(error.capability, expected);
    debug_assert!(!error.code.is_empty());
    RustPlanApplicationFailure::Port(error)
}
