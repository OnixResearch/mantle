//! crunch-build: Build pipeline for crunch.
//!
//! Translates `nix_compat::Derivation` structs into `snix_build::BuildRequest`,
//! executes builds in a sandbox, and persists results in the store.
//!
//! Pipeline: `Derivation` → `BuildRequest` → sandbox → `BuildResult` → `PathInfo`

mod build_request;
pub mod ca_mapping;
mod error;
mod export;
pub mod fetcher;
mod fod;
pub mod goal;
mod hash;
mod orchestrate;
mod references;
pub mod rewrite;
#[cfg(test)]
pub(crate) mod test_support;
pub mod worker;

pub use build_request::derivation_to_build_request;
pub use error::Error;
pub use fetcher::{Fetch, FetchError};
pub use goal::{Goal, GoalRegistry, GoalState};
pub use orchestrate::{BuildOutcome, Builder};
pub use worker::{EvalMessage, Worker, WorkerResult};
