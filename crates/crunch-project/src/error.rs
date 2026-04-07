//! Project-level errors.

use crate::version::SchemaVersion;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("manifest: {0}")]
    Manifest(String),

    #[error("lockfile: {0}")]
    Lockfile(String),

    #[error("lockfile JSON: {0}")]
    LockfileJson(#[from] serde_json::Error),

    #[error("schema version mismatch: file has {found}, expected compatible with {expected}")]
    VersionMismatch {
        found: SchemaVersion,
        expected: SchemaVersion,
    },

    #[error("validation: {0}")]
    Validation(String),

    #[error("drift detected: {0}")]
    Drift(String),

    #[error("upgrade: {0}")]
    Upgrade(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
