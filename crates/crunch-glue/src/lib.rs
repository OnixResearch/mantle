//! crunch-glue: Convert deserialized Nickel derivation records into
//! `nix_compat::Derivation` structs with BLAKE3 store paths.
//!
//! The pipeline: `Expr::to_serde::<CrunchDerivation>()` → `convert()` →
//! `(StorePath, nix_compat::Derivation)`.

mod types;
mod convert;
mod known_paths;
mod error;

pub use convert::convert;
pub use error::Error;
pub use known_paths::KnownPaths;
pub use types::*;

#[cfg(test)]
mod tests;
