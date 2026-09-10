#![cfg_attr(not(kani), feature(register_tool))]
#![register_tool(tigerstyle)]

//! Versioned build request and observation contract for Mantle consumers.
//!
//! The crate owns bounded wire values, deterministic BLAKE3 identities, and
//! pure admission. It performs no evaluation, build, store, cache, process,
//! filesystem, network, clock, credential, persistence, or retry behavior.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

// r[impl mantle.build_interchange.contract]
// r[impl mantle.build_interchange.boundary]
mod admission;
mod identity;
mod model;

pub use admission::validate_observation;
pub use admission::validate_request;
pub use identity::observation_identity;
pub use identity::request_identity;
pub use model::*;

#[cfg(test)]
mod tests;
