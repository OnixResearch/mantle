//! crunch-glue: Convert deserialized Nickel derivation records into
//! `nix_compat::Derivation` structs with BLAKE3 store paths.
//!
//! The pipeline: `Expr::to_serde::<CrunchDerivation>()` -> `convert()` ->
//! `(StorePath, nix_compat::Derivation)`.

mod types;
mod convert;
mod conversion_cache;
mod error;
pub mod nickel_string;

pub use convert::convert;
pub use error::Error;
pub use conversion_cache::{ConversionCache, ConversionEntry};
pub use types::*;

#[cfg(test)]
mod tests;
