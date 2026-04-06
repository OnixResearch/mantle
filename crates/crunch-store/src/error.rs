//! Store-level errors.

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

    #[error("export: {0}")]
    Export(String),

    #[error("cache: {0}")]
    Cache(String),
}
