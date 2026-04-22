#![no_std]

//! `crunch-delta-core`: compiler-enforced no-std home for extracted delta
//! transfer model, protocol negotiation, and reuse planning logic.
//!
//! `crunch-delta` stays std-facing and owns castore/store/network conversion,
//! manifest probing, and substitution orchestration around this core crate.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod model;
mod negotiation;
mod planner;

pub use model::ArtifactNode;
pub use model::BlobNode;
pub use model::ChunkProfile;
pub use model::ChunkRef;
pub use model::ClosureFixture;
pub use model::DeltaDigest;
pub use model::DirectoryNode;
pub use model::OutputFixture;
pub use model::ReceiverManifest;
pub use model::TransferPlan;
pub use model::TransferTally;
pub use model::artifact_full_transfer_bytes;
pub use model::blob_chunked_size_bytes;
pub use model::chunk_profile_v1;
pub use model::closure_full_transfer_bytes;
pub use model::output_full_transfer_bytes;
pub use model::validate_blob;
pub use negotiation::ChunkDigestAlgorithmWire;
pub use negotiation::ChunkProfileWire;
pub use negotiation::ChunkingAlgorithmWire;
pub use negotiation::NegotiatedProtocol;
pub use negotiation::NegotiationError;
pub use negotiation::NegotiationOffer;
pub use negotiation::PROTOCOL_VERSION_V1;
pub use negotiation::chunk_profile_wire_v1;
pub use negotiation::negotiate_protocol;
pub use negotiation::validate_chunk_profile_wire;
pub use planner::PlanError;
pub use planner::plan_transfer;
