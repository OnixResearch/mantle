#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Fixture-owned pure core for Mantle's hardware-simulation reference rail.
//! It validates typed workload data and lowers it to generic action, dynamic-plan,
//! smoke-result, and evidence facts. Filesystem access, Git, hashing file bytes,
//! sandbox execution, Nix/store materialization, publication, and rendering stay
//! in the std-facing shell. This crate is not part of Mantle scheduler/store
//! semantics and does not interpret HDL source text.

pub mod error;

extern crate alloc;

#[cfg(test)]
extern crate std;

mod digest;
mod evidence;
mod model;
mod plan;
mod profile;
mod smoke;

pub use digest::*;
pub use evidence::*;
pub use model::*;
pub use plan::*;
pub use profile::*;
pub use smoke::*;

#[cfg(test)]
mod tests;
pub use error::DigestError;
pub use error::HardwareSimulationError;
