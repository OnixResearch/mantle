#![no_std]
extern crate alloc;

mod error;
mod lineage;
mod profile;
mod provider_boundary;
mod source_root_capability;
mod validate;

pub use error::LineageError;
pub use lineage::AuditedSeed;
pub use lineage::Blake3Hex;
pub use lineage::DigestEntry;
pub use lineage::EnvironmentAssumption;
pub use lineage::GeneratedArtifact;
pub use lineage::LineageManifest;
pub use lineage::LineageNode;
pub use lineage::LineageNodeKind;
pub use lineage::NormalizedProviderOutput;
pub use lineage::Patch;
pub use lineage::ProviderOutputRole;
pub use lineage::SeedClass;
pub use lineage::SourceArtifact;
pub use lineage::StageTransition;
pub use lineage::TransitionTool;
pub use profile::BootstrapToolDigest;
pub use profile::STAGEX_LINEAGE_PROVIDER_KIND;
pub use profile::STAGEX_VERIFIED_NO_QUORUM_CLASS;
pub use profile::StagexLineageProofBlock;
pub use profile::StagexNoQuorumResult;
pub use profile::StagexProfileStatus;
pub use profile::StagexQuorumStatus;
pub use profile::evaluate_stagex_no_quorum;
pub use provider_boundary::LegacyProviderClassification;
pub use provider_boundary::ProviderBoundaryResult;
pub use provider_boundary::REQUIRED_PROVIDER_ROLES;
pub use provider_boundary::RawLayoutViolation;
pub use provider_boundary::classify_legacy_provider_evidence;
pub use provider_boundary::validate_provider_boundary;
pub use source_root_capability::*;
pub use validate::ValidationDiagnostic;
pub use validate::ValidationResult;
pub use validate::validate_lineage;

pub const DEFAULT_AUDIT_SEED_MAX_BYTES: u32 = 4096;
pub const BLAKE3_HEX_LENGTH: usize = 64;
