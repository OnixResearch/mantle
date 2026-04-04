use nix_compat::store_path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing required field: {field}")]
    MissingField { field: String },

    #[error("invalid hash: {0}")]
    InvalidHash(String),

    #[error("unknown hash algorithm: {0}")]
    UnknownHashAlgo(String),

    #[error("circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("invalid store path: {0}")]
    InvalidStorePath(String),

    #[error("derivation error: {0}")]
    Derivation(#[from] nix_compat::derivation::DerivationError),

    #[error("store path error: {0}")]
    StorePath(#[from] store_path::BuildStorePathError),

    #[error("deserialization error: {0}")]
    Serde(String),
}
