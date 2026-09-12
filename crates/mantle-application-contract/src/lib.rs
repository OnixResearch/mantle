#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Application-architecture contract for the Mantle CLI composition root.
//!
//! This crate owns the typed vocabulary the root dispatches with: command
//! families, application commands, blockers, effect plans, observations, and
//! typed capability outcomes. Domain policy lives in the owning cores; the
//! root keeps parsing, adapter selection, dispatch, and presentation.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod bootstrap_flow;
mod component_flow;
mod diagnostics;
mod envelope;
mod evaluation;
mod family;
mod planning;
mod ports;
mod project_lifecycle;
mod realization;
mod release;
mod remote_execution;
mod source_provenance;
mod store_administration;

pub use bootstrap_flow::BootstrapBlocker;
pub use bootstrap_flow::BootstrapCommand;
pub use bootstrap_flow::BootstrapOperation;
pub use bootstrap_flow::BootstrapOutcome;
pub use bootstrap_flow::BootstrapPort;
pub use bootstrap_flow::BootstrapResult;
pub use bootstrap_flow::MAX_BOOTSTRAP_FLOW_DECLARED_ENTRIES;
pub use bootstrap_flow::validate_bootstrap_flow;
pub use component_flow::ComponentBlocker;
pub use component_flow::ComponentCommand;
pub use component_flow::ComponentOperation;
pub use component_flow::ComponentOutcome;
pub use component_flow::ComponentPort;
pub use component_flow::ComponentResult;
pub use component_flow::MAX_COMPONENT_FLOW_DECLARED_ENTRIES;
pub use component_flow::validate_component_flow;
pub use diagnostics::DiagnosticsBlocker;
pub use diagnostics::DiagnosticsCommand;
pub use diagnostics::DiagnosticsOperation;
pub use diagnostics::DiagnosticsOutcome;
pub use diagnostics::DiagnosticsPort;
pub use diagnostics::DiagnosticsResult;
pub use diagnostics::MAX_DIAGNOSTICS_DECLARED_ENTRIES;
pub use diagnostics::validate_diagnostics;
pub use envelope::ApplicationBlocker;
pub use envelope::ApplicationOutcome;
pub use envelope::CapabilityError;
pub use envelope::Effect;
pub use envelope::EffectId;
pub use envelope::EffectKind;
pub use envelope::EffectPlan;
pub use envelope::MAX_EFFECTS_PER_PLAN;
pub use envelope::Observation;
pub use envelope::ObservationStatus;
pub use envelope::classify_observations;
pub use envelope::plan_effects;
pub use evaluation::EvaluationBlocker;
pub use evaluation::EvaluationCommand;
pub use evaluation::EvaluationOperation;
pub use evaluation::EvaluationOutcome;
pub use evaluation::EvaluationPort;
pub use evaluation::EvaluationResult;
pub use evaluation::MAX_EVALUATION_DECLARED_ENTRIES;
pub use evaluation::validate_evaluation;
pub use family::CommandFamily;
pub use family::MAX_COMMAND_ROOTS;
pub use planning::MAX_EXPLAIN_DEPTH;
pub use planning::MAX_PLANNING_DECLARED_ENTRIES;
pub use planning::PlanningBlocker;
pub use planning::PlanningCommand;
pub use planning::PlanningOperation;
pub use planning::PlanningOutcome;
pub use planning::PlanningPort;
pub use planning::PlanningResult;
pub use planning::validate_planning;
pub use ports::FamilyPorts;
pub use ports::MAX_PORT_NAME_LEN;
pub use ports::MAX_PORTS_PER_FAMILY;
pub use ports::family_ports;
pub use ports::is_port_name;
pub use ports::port_inventory;
pub use ports::port_label;
pub use ports::port_owners;
pub use ports::validate_port_inventory;
pub use project_lifecycle::MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES;
pub use project_lifecycle::ProjectBlocker;
pub use project_lifecycle::ProjectCommand;
pub use project_lifecycle::ProjectOperation;
pub use project_lifecycle::ProjectOutcome;
pub use project_lifecycle::ProjectPort;
pub use project_lifecycle::ProjectResult;
pub use project_lifecycle::validate_project_lifecycle;
pub use realization::MAX_REALIZATION_ROOTS;
pub use realization::RealizationBlocker;
pub use realization::RealizeCommand;
pub use realization::RealizeOutcome;
pub use realization::RealizePort;
pub use realization::RealizeResult;
pub use realization::validate_realize_command;
pub use release::MAX_RELEASE_PROOFS;
pub use release::ReleaseBlocker;
pub use release::ReleaseCommand;
pub use release::ReleaseOperation;
pub use release::ReleaseOutcome;
pub use release::ReleasePort;
pub use release::ReleaseProofKind;
pub use release::ReleaseResult;
pub use release::validate_release_command;
pub use remote_execution::MAX_REMOTE_EXECUTION_DECLARED_ENTRIES;
pub use remote_execution::RemoteExecutionBlocker;
pub use remote_execution::RemoteExecutionCommand;
pub use remote_execution::RemoteExecutionOperation;
pub use remote_execution::RemoteExecutionOutcome;
pub use remote_execution::RemoteExecutionPort;
pub use remote_execution::RemoteExecutionResult;
pub use remote_execution::validate_remote_execution;
pub use source_provenance::MAX_SOURCE_PROVENANCE_DECLARED_ENTRIES;
pub use source_provenance::SourceProvenanceBlocker;
pub use source_provenance::SourceProvenanceCommand;
pub use source_provenance::SourceProvenanceOperation;
pub use source_provenance::SourceProvenanceOutcome;
pub use source_provenance::SourceProvenancePort;
pub use source_provenance::SourceProvenanceResult;
pub use source_provenance::validate_source_provenance;
pub use store_administration::MAX_STORE_SELECTORS;
pub use store_administration::StoreAdministrationCommand;
pub use store_administration::StoreAdministrationOutcome;
pub use store_administration::StoreAdministrationPort;
pub use store_administration::StoreAdministrationResult;
pub use store_administration::StoreBlocker;
pub use store_administration::StoreOperation;
pub use store_administration::validate_store_command;
