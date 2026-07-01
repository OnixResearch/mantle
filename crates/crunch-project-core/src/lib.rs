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
mod fetch_policy;
mod generate;
mod importer;
mod lock;
mod manifest;
mod merge;
mod mirrors;
mod refresh;
mod soundness;
mod upgrade;
mod version;

pub use attestation::ProjectAttestationRequest;
pub use attestation::synthesize_project_attestation;
pub use drift::DriftStatus;
pub use drift::check_drift;
pub use error::Error;
pub use fetch_policy::FETCH_POLICY_NON_CLAIM;
pub use fetch_policy::GeneratedInputMode;
pub use fetch_policy::InputFetchDiagnostic;
pub use fetch_policy::InputFetchDiagnosticKind;
pub use fetch_policy::InputFetchPolicy;
pub use fetch_policy::InputFetchPolicyItem;
pub use fetch_policy::InputFetchPolicyPlanRequest;
pub use fetch_policy::InputFetchPolicyReport;
pub use fetch_policy::InputFetchRequirement;
pub use fetch_policy::InputSourceStateClass;
pub use fetch_policy::InputSourceStateFact;
pub use fetch_policy::fetch_policy_compatibility_problems;
pub use fetch_policy::input_source_identity;
pub use fetch_policy::lock_entry_without_fetch;
pub use fetch_policy::plan_input_fetch_policies;
pub use generate::content_fingerprint;
pub use generate::generate_inputs_ncl;
pub use importer::ExternalExistingFile;
pub use importer::ExternalHash;
pub use importer::ExternalPatch;
pub use importer::ExternalPatchSource;
pub use importer::ExternalPin;
pub use importer::ExternalPinKind;
pub use importer::ExternalPinMetadata;
pub use importer::ExternalPinSet;
pub use importer::PIN_IMPORT_DEFAULT_INPUTS_FILE;
pub use importer::PIN_IMPORT_DEFAULT_LOCK_FILE;
pub use importer::PIN_IMPORT_DEFAULT_PROJECT_FILE;
pub use importer::PIN_IMPORT_PLAN_SCHEMA;
pub use importer::PIN_IMPORT_SUPPORTED_IMPORTER;
pub use importer::PinImportBlocker;
pub use importer::PinImportFileOperation;
pub use importer::PinImportMappedInput;
pub use importer::PinImportMappedPatch;
pub use importer::PinImportOptions;
pub use importer::PinImportPlan;
pub use importer::PinImportSemantic;
pub use importer::build_pin_import_plan;
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
pub use merge::FilterInputsResult;
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
pub use soundness::ProjectSoundnessClass;
pub use soundness::ProjectSoundnessFact;
pub use soundness::ProjectSoundnessInput;
pub use soundness::ProjectSoundnessIssue;
pub use soundness::ProjectSoundnessMode;
pub use soundness::ProjectSoundnessReport;
pub use soundness::ProjectSoundnessSeverity;
pub use soundness::ProjectSoundnessSubject;
pub use soundness::check_project_soundness;
pub use soundness::project_soundness_parse_error;
pub use upgrade::OLDEST_SUPPORTED;
pub use upgrade::upgrade_lockfile;
pub use version::SchemaVersion;
pub use version::parse_version;
