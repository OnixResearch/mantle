//! Store-level errors.

use nix_compat::store_path::StorePath;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("store: {0}")]
    Store(String),

    #[error("blob service: {0}")]
    BlobService(String),

    #[error("directory service: {0}")]
    DirectoryService(String),

    #[error("pathinfo service: {0}")]
    PathInfoService(String),

    #[error("strict mode does not permit in-memory PathInfo fallback: {detail}")]
    PathInfoFallbackRejected { detail: String },

    #[error("missing closure facts for source input {}: {detail}", path.to_absolute_path_with_prefix(store_dir))]
    MissingClosureFacts {
        path: StorePath<String>,
        store_dir: String,
        detail: String,
    },

    #[error("export: {0}")]
    Export(String),

    #[error("cache: {0}")]
    Cache(String),

    #[error("attestation: {0}")]
    Attestation(String),
}
