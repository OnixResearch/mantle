//! irpc service definitions for castore services.
//!
//! Each service is defined as an enum with `#[rpc_requests]`. The enum
//! variants map to the trait methods on `BlobService` and `DirectoryService`.
//! In-process, these go through tokio mpsc channels with zero serialization.

use bytes::Bytes;
use irpc::{channel::{mpsc, oneshot}, rpc_requests};
use serde::{Deserialize, Serialize};
use std::io;

use crate::B3Digest;

// ---------------------------------------------------------------------------
// BlobService
// ---------------------------------------------------------------------------

/// Request/response types for the blob content-addressed store.
#[rpc_requests(message = BlobServiceMessage, no_rpc, no_spans)]
#[derive(Debug, Serialize, Deserialize)]
pub enum BlobServiceProtocol {
    /// Check if a blob exists by its BLAKE3 digest.
    #[rpc(tx = oneshot::Sender<io::Result<bool>>)]
    #[wrap(HasBlobRequest)]
    Has(B3Digest),

    /// Open a blob for reading. Returns chunked byte stream.
    /// None if the blob doesn't exist.
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
// DirectoryService
// ---------------------------------------------------------------------------

/// Request/response types for the directory content-addressed store.
#[rpc_requests(message = DirectoryServiceMessage, no_rpc, no_spans)]
#[derive(Debug, Serialize, Deserialize)]
pub enum DirectoryServiceProtocol {
    /// Get a single directory by digest.
    #[rpc(tx = oneshot::Sender<io::Result<Option<crate::Directory>>>)]
    #[wrap(GetDirectoryRequest)]
    Get(B3Digest),

    /// Store a directory closure. Returns the root digest.
    #[rpc(tx = oneshot::Sender<io::Result<B3Digest>>)]
    #[wrap(PutDirectoryRequest)]
    Put(crate::Directory),
}
