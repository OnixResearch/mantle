use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceKind {
    ExactCommit,
    FloatingRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileUpdate {
    pub previous_review_revision: String,
    pub selected_revision: String,
    pub reason: String,
    pub replay_required: bool,
    pub replay_evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceProfile {
    pub repository: String,
    pub revision: String,
    pub archive_url: String,
    pub archive_sha256_sri: String,
    pub archive_blake3: Blake3Digest,
    pub cargo_lock_blake3: Blake3Digest,
    pub dependency_manifest_blake3: Blake3Digest,
    pub dependency_package_count: u32,
    pub octet_support_projection_blake3: Blake3Digest,
    pub license_members: Vec<String>,
    pub notice_members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainProfile {
    pub rust_channel: String,
    pub rust_version: String,
    pub host_triple: String,
    pub wasm_triple: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetRole {
    HostLibrary,
    WasmLibrary,
    HostDiagnosticRunner,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetProfile {
    pub role: TargetRole,
    pub triple: String,
    pub pointer_width_bits: u32,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportStatus {
    Supported,
    Unsupported,
    Planned,
    Unreviewed,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportEntry {
    pub feature: String,
    pub status: SupportStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerProfile {
    pub runner_id: String,
    pub max_input_bytes: u64,
    pub max_stream_chunks: u32,
    pub default_fuel: u64,
    pub max_code_pages: u32,
    pub max_control_frames: u32,
    pub max_stack_words: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    Passed,
    Failed,
    Skipped,
    Unavailable,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureClass {
    MvpPositive,
    MvpNegative,
    StreamingPositive,
    StreamingNegative,
    AllocationFailure,
    UnsupportedFeature,
    Trap,
    OutOfFuel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureProfile {
    pub fixture_id: String,
    pub class: FixtureClass,
    pub artifact_path: String,
    pub descriptor_path: String,
    pub artifact_blake3: Blake3Digest,
    pub descriptor_blake3: Blake3Digest,
    pub generator_input_blake3: Blake3Digest,
    pub expected_status: CheckStatus,
    pub expected_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CorpusRole {
    UpstreamSpectest,
    UpstreamFuzz,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusProfile {
    pub corpus_id: String,
    pub role: CorpusRole,
    pub descriptor_path: String,
    pub source_path: String,
    pub provenance: String,
    pub license_path: String,
    pub descriptor_blake3: Blake3Digest,
    pub expected_status: CheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckProfile {
    pub check_id: String,
    pub expected_status: CheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionProfile {
    pub source_archive: bool,
    pub dependency_closure: bool,
    pub toolchain: bool,
    pub binaries: bool,
    pub fixtures: bool,
    pub reports: bool,
    pub licenses_and_notices: bool,
    pub non_claims: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceBounds {
    pub max_profile_text_bytes: u32,
    pub max_collection_items: u32,
    pub max_bundle_members: u32,
    pub max_parent_edges: u32,
    pub max_bundle_member_bytes: u64,
    pub max_bundle_total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceProfile {
    pub schema: String,
    pub profile_id: String,
    pub profile_update: ProfileUpdate,
    pub source: SourceProfile,
    pub toolchain: ToolchainProfile,
    pub targets: Vec<TargetProfile>,
    pub support_matrix: Vec<SupportEntry>,
    pub requested_features: Vec<String>,
    pub runner: RunnerProfile,
    pub fixtures: Vec<FixtureProfile>,
    pub corpora: Vec<CorpusProfile>,
    pub checks: Vec<CheckProfile>,
    pub retention: RetentionProfile,
    pub bounds: ReferenceBounds,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetFact {
    pub role: TargetRole,
    pub triple: String,
    pub pointer_width_bits: u32,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusFact {
    pub corpus_id: String,
    pub descriptor_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedCheck {
    pub check_id: String,
    pub status: CheckStatus,
    pub result_code: String,
    pub command_blake3: Blake3Digest,
    pub configuration_blake3: Blake3Digest,
    pub input_blake3: Blake3Digest,
    pub output_blake3: Option<Blake3Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundleRole {
    SourceArchive,
    CargoLock,
    DependencyManifest,
    DependencyClosure,
    RustcBinary,
    CargoBinary,
    ToolchainArchive,
    FixtureGenerator,
    HostLibrary,
    WasmLibrary,
    HostRunner,
    ProfileSource,
    ProfileExport,
    SupportProjection,
    FixtureArtifact,
    FixtureDescriptor,
    CorpusArtifact,
    CorpusDescriptor,
    CheckReceipt,
    ResultReport,
    ReplayEvidence,
    MaterializationReport,
    License,
    Notice,
    NonClaims,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleMember {
    pub path: String,
    pub role: BundleRole,
    pub digest_blake3: Blake3Digest,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParentEdge {
    pub parent_path: String,
    pub child_path: String,
    pub relation: String,
}
