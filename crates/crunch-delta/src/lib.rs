mod fixtures;
mod manifest;
mod model;
mod planner;

pub use fixtures::{
    BenchCase, BenchSuite, ReceiverFrontierSummary, ReceiverLossyFrontierSummary, ReceiverProbabilisticFrontierSummary,
    bench_suite,
};
pub use manifest::{
    ManifestBuildOutcome, ManifestError, ManifestProbeCounts, build_receiver_manifest, build_receiver_manifest_lossy,
    build_receiver_manifest_probabilistic,
};
pub use model::{
    ArtifactNode, BlobNode, ChunkProfile, ChunkRef, ClosureFixture, DirectoryNode, OutputFixture, ReceiverManifest,
    TransferPlan, TransferTally, chunk_profile_v1,
};
pub use planner::{PlanError, plan_transfer};

pub const PROTOCOL_VERSION_V1: u32 = 1;
