//! crunch-build: Build pipeline for crunch.
//!
//! Translates `nix_compat::Derivation` structs into `snix_build::BuildRequest`,
//! executes builds in a sandbox, and persists results in the store.
//!
//! Pipeline: `Derivation` → `BuildRequest` → sandbox → `BuildResult` → `PathInfo`

mod build_request;
pub mod ca_mapping;
mod error;
pub mod fetcher;
mod orchestrate;
pub mod rewrite;

pub use build_request::derivation_to_build_request;
pub use error::Error;
pub use fetcher::{Fetch, FetchError};
pub use orchestrate::{BuildOutcome, Builder};
