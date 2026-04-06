//! irpc service definitions and client wrappers for castore services.
//!
//! Each service is defined as an enum with `#[rpc_requests]`. The enum
//! variants map to the trait methods on `BlobService` and `DirectoryService`.
//! In-process, these go through tokio mpsc channels with zero serialization.

use async_trait::async_trait;
use bytes::Bytes;
use irpc::{channel::oneshot, rpc_requests};
use serde::{Deserialize, Serialize};
use std::io;

use crate::B3Digest;

// ---------------------------------------------------------------------------
// BlobService protocol
// ---------------------------------------------------------------------------

/// Request/response types for the blob content-addressed store.
#[rpc_requests(message = BlobServiceMessage, no_rpc, no_spans)]
#[derive(Debug, Serialize, Deserialize)]
pub enum BlobServiceProtocol {
    /// Check if a blob exists by its BLAKE3 digest.
    #[rpc(tx = oneshot::Sender<io::Result<bool>>)]
    #[wrap(HasBlobRequest)]
    Has(B3Digest),

    /// Read all bytes of a blob. Returns None if not found.
    #[rpc(tx = oneshot::Sender<io::Result<Option<Vec<u8>>>>)]
    #[wrap(ReadBlobRequest)]
    Read(B3Digest),

    /// Upload a blob. Returns its computed BLAKE3 digest.
    #[rpc(tx = oneshot::Sender<io::Result<B3Digest>>)]
    #[wrap(PutBlobRequest)]
    Put(Bytes),

    /// Get chunk metadata for a blob.
    #[rpc(tx = oneshot::Sender<io::Result<Vec<ChunkMeta>>>)]
    #[wrap(ChunksRequest)]
    Chunks(B3Digest),
}

/// Chunk metadata returned by the Chunks request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMeta {
    pub digest: B3Digest,
    pub size: u64,
}

// ---------------------------------------------------------------------------
// DirectoryService protocol
// ---------------------------------------------------------------------------

/// Request/response types for the directory content-addressed store.
#[rpc_requests(message = DirectoryServiceMessage, no_rpc, no_spans)]
#[derive(Debug, Serialize, Deserialize)]
pub enum DirectoryServiceProtocol {
    /// Get a single directory by digest.
    #[rpc(tx = oneshot::Sender<Result<Option<crate::Directory>, String>>)]
    #[wrap(GetDirectoryRequest)]
    Get(B3Digest),

    /// Store a directory. Returns the root digest.
    #[rpc(tx = oneshot::Sender<Result<B3Digest, String>>)]
    #[wrap(PutDirectoryRequest)]
    Put(crate::Directory),
}

// ---------------------------------------------------------------------------
// DirectoryService client wrapper
// ---------------------------------------------------------------------------

/// An irpc-based DirectoryService client.
#[derive(Clone)]
pub struct IrpcDirectoryService {
    client: irpc::Client<DirectoryServiceProtocol>,
}

impl IrpcDirectoryService {
    pub fn new(client: irpc::Client<DirectoryServiceProtocol>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl crate::directoryservice::DirectoryService for IrpcDirectoryService {
    async fn get(
        &self,
        digest: &B3Digest,
    ) -> Result<Option<crate::Directory>, crate::directoryservice::Error> {
        self.client
            .rpc(GetDirectoryRequest(*digest))
            .await
            .map_err(|e| -> crate::directoryservice::Error { e.to_string().into() })?
            .map_err(|e| -> crate::directoryservice::Error { e.into() })
    }

    async fn put(&self, directory: crate::Directory) -> Result<B3Digest, crate::directoryservice::Error> {
        self.client
            .rpc(PutDirectoryRequest(directory))
            .await
            .map_err(|e| -> crate::directoryservice::Error { e.to_string().into() })?
            .map_err(|e| -> crate::directoryservice::Error { e.into() })
    }

    fn get_recursive(
        &self,
        _root_directory_digest: &B3Digest,
    ) -> futures::stream::BoxStream<'static, Result<crate::Directory, crate::directoryservice::Error>> {
        // Recursive listing not yet supported over irpc.
        Box::pin(futures::stream::empty())
    }

    fn put_multiple_start(&self) -> Box<dyn crate::directoryservice::DirectoryPutter + '_> {
        // Multi-put not yet supported over irpc; fall back to SimplePutter.
        Box::new(crate::directoryservice::SimplePutter::new(self))
    }
}
