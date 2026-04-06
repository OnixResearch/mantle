//! crunch-store: Store operations for crunch.
//!
//! Owns service construction, cache checking, realization (castore -> disk
//! export), CA mapping persistence, and store queries. Consumers receive a
//! `StoreHandle` — they do not construct or own individual services.

mod ca_mapping;
mod error;
mod export;
mod handle;
mod query;

pub use ca_mapping::{CaMappings, OutputMap};
pub use error::Error;
pub use export::{export_castore_to_disk, MAX_EXPORT_DEPTH};
pub use handle::{CacheHit, StoreConfig, StoreHandle};
pub use query::{PathInfoDetail, VerifyResult, store_info, store_list, store_verify};
