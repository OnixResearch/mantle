#![no_std]
//! Backend-neutral source observations and monotonic ingest planning.

// r[impl source_transports.source_observations.contract]
// r[impl source_transports.source_observations.locator_boundary]
// r[impl source_transports.source_observations.compatibility]
// r[impl source_transports.source_observations.claim_boundary]
// r[impl source_transports.monotonic_ingest]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod ingest;
mod observation;

pub use ingest::*;
pub use observation::*;

#[cfg(test)]
mod tests;
