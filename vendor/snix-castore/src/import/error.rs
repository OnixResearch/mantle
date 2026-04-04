use super::PathBuf;

/// Represents all error types that emitted by ingest_entries.
/// It can represent errors uploading individual Directories and finalizing
/// the upload.
/// It also contains a generic error kind that'll carry ingestion-method
/// specific errors.
#[derive(Debug, thiserror::Error)]
pub enum IngestionError<E: std::fmt::Display> {
    #[error("error from producer: {0}")]
    Producer(#[from] E),

    #[error("invalid symlink target")]
    InvalidSymlinkTarget(PathBuf, #[source] crate::SymlinkTargetError),

    #[error("failed to upload directory at {0}")]
    UploadDirectoryError(PathBuf, #[source] crate::directoryservice::Error),

    #[error("failed to finalize directory upload: {0}")]
    FinalizeDirectoryUpload(#[source] crate::directoryservice::Error),

    #[error("unexpected end of stream")]
    UnexpectedEndOfStream,
}
