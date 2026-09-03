#![no_std]
#![doc = include_str!("../../../fixtures/rust-plan-hexagon-architecture/negative/vendor-port-compile-fail.md")]
//! Deterministic Rust package admission, topology, effects, and receipts.
//!
//! The core consumes bounded normalized facts. It has no filesystem, process,
//! environment, Cargo, rustc, cache, store, path, async-runtime, CLI, or
//! presentation authority.

// r[impl rust_package_planning.hexagonal_core]
// r[impl rust_package_planning.explicit_unit_effects]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod admission;
mod compatibility;
mod effect;
mod error;
mod identity;
mod model;
mod planning;

pub use admission::*;
pub use compatibility::*;
pub use effect::*;
pub use error::*;
pub use identity::*;
pub use model::*;
pub use planning::*;

pub const WORKSPACE_FACTS_SCHEMA: &str = "mantle-rust-workspace-facts-v1";
pub const PLAN_RECEIPT_SCHEMA: &str = "mantle-rust-plan-core-receipt-v1";
pub const BLAKE3_HEX_CHARS: usize = 64;
pub const MAX_TEXT_BYTES: u32 = 16_384;
pub const MAX_PACKAGES: u32 = 4_096;
pub const MAX_TARGETS_PER_PACKAGE: u32 = 128;
pub const MAX_DEPENDENCIES_PER_PACKAGE: u32 = 4_096;
pub const MAX_FEATURES_PER_PACKAGE: u32 = 2_048;
pub const MAX_FEATURE_ACTIVATIONS: u32 = 4_096;
pub const MAX_FEATURE_RESOLUTION_STEPS: u32 = 16_384;
pub const MAX_UNITS: u32 = 16_384;
pub const MAX_UNIT_ARGUMENTS: u32 = 4_096;
pub const MAX_UNIT_ENVIRONMENT_ENTRIES: u32 = 512;
pub const MAX_UNIT_INPUTS: u32 = 4_096;
pub const MAX_UNIT_OUTPUTS: u32 = 256;
pub const MAX_UNIT_STDOUT_BYTES: u64 = 16_777_216;
pub const MAX_UNIT_STDERR_BYTES: u64 = 16_777_216;
pub const UNIT_ENVIRONMENT_ENTRY_COUNT: usize = 4;

const _: () = assert!(MAX_PACKAGES > 0);
const _: () = assert!(MAX_UNITS >= MAX_PACKAGES);
const _: () = assert!(MAX_UNIT_ARGUMENTS > MAX_UNIT_OUTPUTS);

#[cfg(test)]
mod tests;
