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
pub mod signing;
#[cfg(test)]
pub(crate) mod test_support;
pub mod worker;

pub use build_request::derivation_to_build_request;
pub use dispatch_build_service::DispatchBuildService;
pub use dynamic::DynamicDrv;
pub use dynamic::is_drv_output;
pub use dynamic::parse_drv_bytes;
pub use dynamic::register_dynamic_drv;
pub use error::Error;
pub use fetch_build_service::FETCH_BUILDER;
pub use fetch_build_service::FetchBuildService;
pub use fetch_build_service::is_fetch_request;
pub use fetcher::Fetch;
pub use fetcher::FetchError;
pub use goal::Goal;
pub use goal::GoalRegistry;
pub use goal::GoalState;
pub use orchestrate::BuildOutcome;
pub use orchestrate::Builder;
pub use registry::DerivationRegistry;
pub use registry::RegistryEntry;
pub use registry::populate_registry;
pub use signing::KeyPair;
pub use signing::VerifyResult;
pub use signing::build_trusted_keys;
pub use signing::generate_keypair;
pub use signing::load_keypair;
pub use signing::sign_pathinfo;
pub use signing::verify_pathinfo_signatures;
pub use worker::EvalMessage;
pub use worker::FailedGoal;
pub use worker::Worker;
pub use worker::WorkerResult;
