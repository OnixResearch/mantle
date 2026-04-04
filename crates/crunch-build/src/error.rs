//! Error types for the build pipeline.

use nix_compat::store_path::StorePath;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("source input not found in store: {}", path.to_absolute_path())]
    SourceNotFound { path: StorePath<String> },

    #[error("build failed for {name} (exit code {exit_code})")]
    BuildFailed {
        name: String,
        exit_code: String,
        log: String,
    },

    #[error("output not produced by build: {output}")]
    OutputMissing { output: String },

    #[error("FOD hash mismatch for {name}: expected {expected}, got {actual}")]
    FodHashMismatch {
        name: String,
        expected: String,
        actual: String,
    },

    #[error("derivation not found in known_paths: {}", path.to_absolute_path())]
    DerivationNotFound { path: StorePath<String> },

    #[error("output has no store path: {output} in {drv_name}")]
    OutputNoPath { output: String, drv_name: String },

    #[error("sandbox error: {0}")]
    Sandbox(#[from] std::io::Error),

    #[error("NAR calculation error: {0}")]
    NarCalculation(String),

    #[error("store error: {0}")]
    Store(String),

    #[error("glue error: {0}")]
    Glue(#[from] crunch_glue::Error),
}
