//! crunch-build: Build pipeline for crunch.
//!
//! Translates `nix_compat::Derivation` structs into `snix_build::BuildRequest`,
//! executes builds in a sandbox, and persists results in the store.
//!
//! Pipeline: `Derivation` → `BuildRequest` → sandbox → `BuildResult` → `PathInfo`

mod build_request;
pub mod ca_mapping;
pub mod dispatch_build_service;
pub mod dynamic;
mod error;
mod export;
pub mod fetch_build_service;
pub mod fetcher;
mod fod;
pub mod goal;
mod hash;
mod orchestrate;
mod references;
pub mod registry;
pub mod rewrite;
#[cfg(test)]
pub(crate) mod test_support;
pub mod worker;

pub use build_request::derivation_to_build_request;
pub use dispatch_build_service::DispatchBuildService;
pub use dynamic::{DynamicDrv, is_drv_output, parse_drv_bytes, register_dynamic_drv};
pub use error::Error;
pub use fetch_build_service::{FetchBuildService, is_fetch_request, FETCH_BUILDER};
pub use fetcher::{Fetch, FetchError};
pub use goal::{Goal, GoalRegistry, GoalState};
pub use orchestrate::{BuildOutcome, Builder};
pub use registry::{DerivationRegistry, RegistryEntry, populate_registry};
pub use worker::{EvalMessage, FailedGoal, Worker, WorkerResult};
