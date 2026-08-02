//! crunch-glue: Convert deserialized Nickel derivation records into
//! `nix_compat::Derivation` structs with BLAKE3 store paths.
//!
//! The pipeline: `Expr::to_serde::<CrunchDerivation>()` -> `convert()` ->
//! `(StorePath, nix_compat::Derivation)`.

mod conversion_cache;
mod convert;
mod error;
pub mod nickel_string;
mod types;

pub use conversion_cache::ConversionCache;
pub use conversion_cache::ConversionEntry;
pub use conversion_cache::InsertCaEntry;
pub use convert::ResolvedDerivationRequest;
pub use convert::convert;
pub use convert::resolve_derivation_registration;
pub use error::Error;
pub use types::*;

#[cfg(test)]
mod tests;
