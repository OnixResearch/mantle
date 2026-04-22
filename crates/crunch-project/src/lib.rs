//! crunch-project: Project manifest, lockfile, and input management.
//!
//! Owns the human-edited Nickel manifest (`crunch-project.ncl`),
//! the machine-edited JSON lockfile (`crunch.lock`), refresh logic,
//! stale detection, schema upgrades, and generated input files.
//! Consumers use this crate for project-level operations; fetch
//! execution stays in the build pipeline.

mod attestation;
mod attestation_adapter;
mod drift;
mod error;
mod generate;
mod lock;
mod manifest;
mod merge;
mod mirrors;
mod refresh;
mod refresh_adapter;
mod upgrade;
mod version;

pub use attestation::ProjectAttestationInput;
pub use attestation::synthesize_project_attestation;
pub use drift::DriftStatus;
pub use drift::check_drift;
pub use error::Error;
pub use generate::content_fingerprint;
pub use generate::generate_inputs_ncl;
pub use lock::LockEntry;
pub use lock::LockedHash;
pub use lock::LockedKind;
pub use lock::LockedPatch;
pub use lock::LockedPatchSource;
pub use lock::Lockfile;
pub use manifest::GitReference;
pub use manifest::HashAlgo;
pub use manifest::HashSpec;
pub use manifest::InputKind;
pub use manifest::ManifestInput;
pub use manifest::PatchDef;
pub use manifest::PatchSource;
pub use manifest::ProjectManifest;
pub use merge::MergeIssue;
pub use merge::MergeReport;
pub use merge::Severity;
pub use merge::check_manifest_lock;
pub use merge::filter_inputs;
pub use merge::inputs_needing_refresh;
pub use merge::orphaned_lock_entries;
pub use mirrors::url_with_mirrors;
pub use mirrors::validate_mirrors;
pub use refresh::ApplyResult;
pub use refresh::HashResolutionMode;
pub use refresh::RefreshFailure;
pub use refresh::RefreshOutcome;
pub use refresh::RefreshResolver;
pub use refresh::ResolvedInput;
pub use refresh::StaleReport;
pub use refresh::apply_outcomes;
pub use refresh::list_stale;
pub use refresh::refresh_inputs;
pub use upgrade::OLDEST_SUPPORTED;
pub use upgrade::upgrade_lockfile;
pub use version::SchemaVersion;
pub use version::parse_version;
