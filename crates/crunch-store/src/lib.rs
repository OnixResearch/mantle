//! crunch-store: Store operations for crunch.
//!
//! Owns service construction, cache checking, realization (castore -> disk
//! export), CA mapping persistence, and store queries. Consumers receive a
//! `StoreHandle` — they do not construct or own individual services.

mod ca_mapping;
mod closure;
mod error;
mod export;
mod handle;
mod query;

pub use ca_mapping::CaMappings;
pub use ca_mapping::OutputMap;
pub use closure::MAX_CLOSURE_DEPTH;
pub use closure::resolve_closure;
pub use error::Error;
pub use export::MAX_EXPORT_DEPTH;
pub use export::export_castore_to_disk;
pub use handle::CacheHit;
pub use handle::StoreConfig;
pub use handle::StoreHandle;
pub use query::PathInfoDetail;
pub use query::SignResult;
pub use query::SignatureVerifyResult;
pub use query::VerifyResult;
pub use query::store_info;
pub use query::store_list;
pub use query::store_sign;
pub use query::store_verify;
pub use query::store_verify_signatures;
