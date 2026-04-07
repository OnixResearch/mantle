//! crunch-project: Project manifest, lockfile, and input management.
//!
//! Owns the human-edited Nickel manifest (`crunch-project.ncl`),
//! the machine-edited JSON lockfile (`crunch.lock`), refresh logic,
//! stale detection, schema upgrades, and generated input files.
//! Consumers use this crate for project-level operations; fetch
//! execution stays in the build pipeline.

mod drift;
mod error;
mod generate;
mod lock;
mod manifest;
mod merge;
mod refresh;
mod upgrade;
mod version;

pub use drift::{DriftStatus, check_drift};
pub use error::Error;
pub use generate::{content_fingerprint, generate_inputs_ncl};
pub use lock::{
    LockEntry, Lockfile, LockedHash, LockedKind, LockedPatch, LockedPatchSource,
};
pub use manifest::{
    GitReference, HashAlgo, HashSpec, InputKind, ManifestInput, PatchDef,
    PatchSource, ProjectManifest,
};
pub use merge::{
    MergeIssue, MergeReport, Severity, check_manifest_lock, filter_inputs,
    inputs_needing_refresh, orphaned_lock_entries,
};
pub use refresh::{
    RefreshOutcome, RefreshResolver, ResolvedInput, apply_outcomes,
    list_stale, refresh_inputs,
};
pub use upgrade::upgrade_lockfile;
pub use version::{SchemaVersion, parse_version};
