#![no_std]

//! `crunch-project-core`: compiler-enforced no-std home for extracted
//! project functional-core logic.
//!
//! This first wave owns foundational manifest, lockfile, and schema-version
//! types. Std-facing refresh I/O, tempdirs, and CLI adapters remain in
//! `crunch-project`.

extern crate alloc;

mod lock;
mod manifest;
mod version;

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
pub use version::SchemaVersion;
pub use version::parse_version;
