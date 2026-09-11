#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Pure source-observation and monotonic-ingest core.
//!
//! A locator answers where bytes were observed. It is never canonical source
//! identity, and it grants no trust, ownership, or authority. Canonical
//! identity binds the source kind, the immutable revision, the normalized
//! projection, the snapshot profile, and the measured payload BLAKE3.
//!
//! Filesystem reads, URL access, Git object loading, archive construction,
//! and release bundle I/O belong to std shells.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod digest;
mod ingest;
mod model;

pub use digest::Blake3Digest;
pub use digest::domain_digest;
pub use digest::observation_domain;
pub use ingest::ExistingObservation;
pub use ingest::IngestOutcome;
pub use ingest::IngestPlan;
pub use ingest::LegacyProvenanceProjection;
pub use ingest::LegacyV1Facts;
pub use ingest::plan_ingest;
pub use ingest::project_legacy_v1;
pub use model::GitObjectFormat;
pub use model::LocatorBoundaryPolicy;
pub use model::LocatorClass;
pub use model::MAX_LOCATOR_BYTES;
pub use model::MAX_PROFILE_BYTES;
pub use model::MAX_PROJECTION_COMPONENTS;
pub use model::MAX_QUERY_FIELDS;
pub use model::ProjectionPath;
pub use model::SOURCE_OBSERVATION_ENCODING_VERSION;
pub use model::SOURCE_OBSERVATION_SCHEMA;
pub use model::SnapshotProfile;
pub use model::SourceDiagnostic;
pub use model::SourceKind;
pub use model::SourceObservation;
pub use model::SourceObservationRequest;
pub use model::SourceObservationResult;
pub use model::admit_source_observation;
