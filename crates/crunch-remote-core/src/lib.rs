#![no_std]
#![doc = include_str!("../../../fixtures/remote-hexagon-architecture/negative/vendor-port-compile-fail.md")]
//! Deterministic remote-build policy, transitions, and effect plans.
//!
//! The core consumes admitted owned facts. It has no filesystem, process,
//! network, environment, clock, random, credential, store, async-runtime, or
//! presentation authority.

// r[impl remote_builds.hexagonal_core]
// r[impl remote_builds.remote_effect_plans]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod admission;
mod error;
mod identity;
mod model;
mod transition;
mod wire;

pub use admission::*;
pub use error::*;
pub use identity::*;
pub use model::*;
pub use transition::*;
pub use wire::*;
