#![no_std]
//! Application contracts for the Rust package-planning hexagon.
//!
//! Ports use Mantle-owned requests and observations. Process, filesystem,
//! Cargo, rustc, cache, store, path, and rendering types stay in adapters.

// r[impl rust_package_planning.application_owned_ports]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod application;
mod ports;

pub use application::*;
pub use ports::*;

#[cfg(test)]
mod tests;
