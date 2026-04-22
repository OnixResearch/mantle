#![no_std]

//! `crunch-project-core`: compiler-enforced no-std home for extracted
//! project functional-core logic.
//!
//! This first wave owns foundational manifest, lockfile, and schema-version
//! types. Std-facing refresh I/O, tempdirs, and CLI adapters remain in
//! `crunch-project`.

extern crate alloc;

#[cfg(test)]
extern crate std;

mod attestation;
mod drift;
mod error;
mod generate;
mod lock;
mod manifest;
mod merge;
mod mirrors;
mod refresh;
mod upgrade;
mod version;

pub use attestation::ProjectAttestationRequest;
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
pub use lock::MAX_LOCKED_PATCHES;
pub use manifest::GitReference;
pub use manifest::HashAlgo;
pub use manifest::HashSpec;
pub use manifest::InputKind;
pub use manifest::MAX_INPUTS;
pub use manifest::MAX_MIRRORS_PER_INPUT;
pub use manifest::MAX_PATCHES_PER_INPUT;
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
pub use refresh::ApplyOutcomesRequest;
pub use refresh::ApplyResult;
pub use refresh::HashResolutionMode;
pub use refresh::PatchResolution;
pub use refresh::PatchResolutionPlanRequest;
pub use refresh::RefreshFailure;
pub use refresh::RefreshInputsPlanRequest;
pub use refresh::RefreshInputsRequest;
pub use refresh::RefreshOutcome;
pub use refresh::ResolvedInput;
pub use refresh::ResolvedInputState;
pub use refresh::StaleReport;
pub use refresh::apply_outcomes;
pub use refresh::list_stale;
pub use refresh::plan_patch_resolutions;
pub use refresh::plan_refresh_inputs;
pub use refresh::refresh_inputs;
pub use upgrade::OLDEST_SUPPORTED;
pub use upgrade::upgrade_lockfile;
pub use version::SchemaVersion;
pub use version::parse_version;
