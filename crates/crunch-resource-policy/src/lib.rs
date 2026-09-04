#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Application ports and Valence projection for Mantle resource policy.
//!
//! The pure core owns policy meaning. This crate owns orchestration across
//! explicit ledger and evidence ports. Concrete files, stores, clocks,
//! schedulers, and output rendering remain in outer adapters.

// r[impl remote_builds.usage_reservation_and_reconciliation]
// r[impl remote_builds.resource_benchmark_evidence]
// r[impl remote_builds.resource_policy_rollout]

mod application;
mod ports;
mod valence;

pub use application::*;
pub use crunch_resource_policy_core::*;
pub use ports::*;
pub use valence::*;

#[cfg(test)]
mod tests;
