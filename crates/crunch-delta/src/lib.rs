#![feature(register_tool)]
#![register_tool(tigerstyle)]

#[allow(tigerstyle::platform_dependent_cast)] // fixture code: all values are known-bounded fixture constants
#[allow(tigerstyle::raw_arithmetic_overflow)] // fixture wire-size calculations with known-valid ordering
#[allow(tigerstyle::no_panic)] // fixture construction panics on programmer error
#[allow(tigerstyle::no_unwrap)] // fixture construction with hardcoded valid values
#[allow(tigerstyle::no_recursion)] // fixture tree walks: bounded by fixture tree depth
#[allow(tigerstyle::ambiguous_params)] // fixture functions: param names are descriptive
mod fixtures;
#[allow(tigerstyle::no_recursion)] // tree-walking traversals bounded by castore node depth
#[allow(tigerstyle::no_unwrap)] // fixture seed construction with hardcoded valid values
mod manifest;
mod model;
mod negotiation;
mod planner;
#[allow(tigerstyle::no_recursion)] // tree-walking traversals bounded by castore node depth
mod substitution;

pub use fixtures::BenchCase;
pub use fixtures::BenchSuite;
pub use fixtures::ReceiverFrontierSummary;
pub use fixtures::ReceiverLossyFrontierSummary;
pub use fixtures::ReceiverProbabilisticFrontierSummary;
pub use fixtures::bench_suite;
pub use manifest::ManifestBuildOutcome;
pub use manifest::ManifestError;
pub use manifest::ManifestProbeCounts;
pub use manifest::build_receiver_manifest;
pub use manifest::build_receiver_manifest_lossy;
pub use manifest::build_receiver_manifest_probabilistic;
pub use model::ArtifactNode;
pub use model::BlobNode;
pub use model::ChunkProfile;
pub use model::ChunkRef;
pub use model::ClosureFixture;
pub use model::DirectoryNode;
pub use model::OutputFixture;
pub use model::ReceiverManifest;
pub use model::TransferPlan;
pub use model::TransferTally;
pub use model::chunk_profile_v1;
pub use negotiation::ChunkDigestAlgorithmWire;
pub use negotiation::ChunkProfileWire;
pub use negotiation::ChunkingAlgorithmWire;
pub use negotiation::NegotiatedProtocol;
pub use negotiation::NegotiationError;
pub use negotiation::NegotiationOffer;
pub use negotiation::PROTOCOL_VERSION_V1;
pub use negotiation::chunk_profile_wire_v1;
pub use negotiation::negotiate_protocol;
pub use planner::PlanError;
pub use planner::plan_transfer;
pub use substitution::ContentCatalog;
pub use substitution::DeltaAcceptanceMode;
pub use substitution::DeltaCandidateResponse;
pub use substitution::DeltaCapabilityAdvertisement;
pub use substitution::DeltaFallbackReason;
pub use substitution::DeltaFetchOutcome;
pub use substitution::DeltaFetchRequest;
pub use substitution::DeltaHttpEndpoints;
pub use substitution::DeltaReceiverHasSet;
pub use substitution::DeltaReceiverState;
pub use substitution::DeltaSubstitutionError;
pub use substitution::DeltaTransferFrame;
pub use substitution::DeltaTransferStats;
pub use substitution::InMemoryDeltaAuthority;
pub use substitution::RetainedContentStore;
pub use substitution::substitute_from_authority;
