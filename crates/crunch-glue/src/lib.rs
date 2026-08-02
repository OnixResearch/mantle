//! crunch-glue: Convert deserialized Nickel derivation records into
//! `nix_compat::Derivation` structs with BLAKE3 store paths.
//!
//! The pipeline: `Expr::to_serde::<CrunchDerivation>()` -> `convert()` ->
//! `(StorePath, nix_compat::Derivation)`.

// The current Cairn profile scans `crates/`, while Mantle's CLI shell is in
// `src/`. These markers link to the implementation modules there until the
// repository profile also scans that shell.
// r[impl foreign_derivation_import.prefix_aware_aterm]
// r[impl foreign_derivation_import.exact_graph_compilation]
// r[impl foreign_derivation_import.foreign_builtin_lowering]
// r[impl foreign_derivation_import.executable_plan]
// r[impl foreign_derivation_import.cache_only_preserved_paths]

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
