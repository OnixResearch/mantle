#![feature(register_tool)]
#![register_tool(tigerstyle)]
//! crunch-store: Store operations for crunch.
//!
//! Owns service construction, cache checking, realization (castore -> disk
//! export), CA mapping persistence, and store queries. Consumers receive a
//! `StoreHandle` — they do not construct or own individual services.

mod attestation;
mod audit;
mod ca_mapping;
mod closure;
mod error;
mod export;
mod gc;
mod handle;
mod mutation_lock;
mod policy;
mod pull;
mod push;
mod query;
mod roots;

pub use attestation::ArtifactProvenance;
pub use attestation::StoredArtifactAttestation;
pub use attestation::StoredClosureAttestation;
pub use attestation::artifact_attestation_file_path;
pub use attestation::closure_attestation_file_path;
pub use attestation::persist_artifact_attestation;
pub use audit::StoreAuditEvent;
pub use audit::StoreAuditKind;
pub use ca_mapping::CaMappings;
pub use ca_mapping::OutputMap;
pub use closure::ClosureResolution;
pub use closure::MAX_CLOSURE_DEPTH;
pub use closure::resolve_closure;
pub use error::Error;
pub use export::MAX_EXPORT_DEPTH;
pub use export::export_castore_to_disk;
pub use gc::GcOperationKind;
pub use gc::GcReport;
pub use handle::CacheHit;
pub use handle::OutputSubstitutionMode;
pub use handle::OutputSubstitutionReport;
pub use handle::StoreConfig;
pub use handle::PersistOutputRequest;
pub use handle::StoreHandle;
pub use handle::StoreHandleServices;
pub use gc::GcContext;
pub use mutation_lock::StoreMutationGuard;
pub use policy::StoreFallbackMode;
pub use query::PathInfoDetail;
pub use query::SignResult;
pub use query::SignatureVerifyResult;
pub use query::VerifyResult;
pub use query::store_info;
pub use query::store_list;
pub use query::store_sign;
pub use query::store_verify;
pub use query::store_verify_signatures;
pub use pull::PullOptions;
pub use pull::PullReport;
pub use pull::PulledPath;
pub use pull::import_paths_from_cache_dir;
pub use push::PushOptions;
pub use push::PushReport;
pub use push::PushedPath;
pub use push::export_paths_to_cache_dir;
pub use roots::GcRootRecord;
pub use roots::GcRootSource;
