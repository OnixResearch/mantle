#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! Deterministic policy for remote resource observations, selection, retries,
//! accounting, sharing, rollout, benchmarks, and evidence plans.
//!
//! The core accepts already observed facts. It performs no filesystem,
//! process, network, clock, random, credential, store, or rendering effects.

// r[impl remote_builds.resource_observations]
// r[impl remote_builds.replayable_resource_selection]
// r[impl remote_builds.positive_oom_retry]
// r[impl remote_builds.usage_reservation_and_reconciliation]
// r[impl remote_builds.authorized_result_sharing]
// r[impl remote_builds.resource_benchmark_evidence]
// r[impl remote_builds.resource_policy_rollout]

extern crate alloc;

#[cfg(test)]
extern crate std;

mod accounting;
mod benchmark;
mod constants;
mod identity;
mod model;
mod observation;
mod retry;
mod selection;
mod sharing;

pub use accounting::*;
pub use benchmark::*;
pub use constants::*;
pub use identity::*;
pub use model::*;
pub use observation::*;
pub use retry::*;
pub use selection::*;
pub use sharing::*;

#[cfg(test)]
mod tests;
