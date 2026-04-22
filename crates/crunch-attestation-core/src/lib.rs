#![no_std]

//! `crunch-attestation-core`: compiler-enforced no-std home for extracted
//! attestation functional-core logic.
//!
//! This first wave owns foundational attestation data types plus digest and
//! schema/version validation. Std-facing filesystem discovery and higher-level
//! adapter ergonomics stay in `crunch-attestation`.

extern crate alloc;

mod digest;
mod error;
mod schema;
mod version;

pub use digest::AttestationDigest;
pub use error::Error;
pub use schema::ArtifactAttestation;
pub use schema::ArtifactFacts;
pub use schema::ArtifactReference;
pub use schema::Claims;
pub use schema::ClosureAttestation;
pub use schema::ClosureFacts;
pub use schema::ClosureSemantics;
pub use schema::Edge;
pub use schema::EdgeKind;
pub use schema::Node;
pub use schema::NodeKind;
pub use schema::ProjectAttestation;
pub use schema::ProjectFacts;
pub use version::SchemaVersion;
