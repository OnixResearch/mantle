#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Application-owned Rust-plan ports and real-effect orchestration.
//!
//! Stateful shell ports capture actual tool versions in CLI call order and
//! materialize one accepted receipt. Later, possibly in another process, core
//! plans the receipt's selected units and admits exact bound process effects.
//! A real Rust cache restores outputs or reports a miss; the executor runs a
//! compiler only on a miss and finishes topology bookkeeping on both branches.
//! Typed cache, process, and build-script metadata observations are classified
//! against admitted facts. No process, Cargo JSON, raw store, host path,
//! `RunError`, or rendering type crosses a port signature.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod application;
mod ports;

pub use application::CacheDisposition;
pub use application::ExecutionError;
pub use application::ObservedExecution;
pub use application::capture_existing_plan;
pub use application::classify_existing_unit_observations;
pub use application::classify_rust_plan_compatibility;
pub use application::execute_existing_units;
pub use application::plan_existing_unit_effects;
pub use ports::AdapterError;
pub use ports::BuildScriptPostprocessFailure;
pub use ports::CacheObservation;
pub use ports::CacheRestore;
pub use ports::CargoOracleCapture;
pub use ports::CompilerFacts;
pub use ports::CompilerInspection;
pub use ports::CompilerInspectionRequest;
pub use ports::OracleCaptureRequest;
pub use ports::PrepareOutcome;
pub use ports::ProcessAttempt;
pub use ports::RustCacheAccess;
pub use ports::UnitExecutionResult;
pub use ports::UnitExecutor;
pub use ports::WorkspaceFactsRequest;
pub use ports::WorkspaceFactsSource;
