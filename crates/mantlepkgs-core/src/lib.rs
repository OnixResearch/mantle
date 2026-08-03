#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
// r[impl mantlepkgs.functional_core]
// r[verify mantlepkgs.functional_core]
//! Pure planning and identity logic for generated Mantlepkgs catalogs.
//!
//! This crate has no filesystem, process, environment, clock, network, store,
//! or evaluator access. Shells supply explicit observations and artifact facts.

extern crate alloc;

mod catalog;
mod domains;
mod error;
mod manifest;
mod render;

pub use catalog::*;
pub use domains::*;
pub use error::*;
pub use manifest::*;
pub use render::*;
