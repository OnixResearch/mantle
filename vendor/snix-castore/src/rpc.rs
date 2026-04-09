//! irpc service definitions and client wrappers for castore services.
//!
//! Each service is defined as an enum with `#[rpc_requests]`. The enum
//! variants map to the trait methods on `BlobService` and `DirectoryService`.
//! In-process, these go through tokio mpsc channels with zero serialization.

use std::io;

use async_trait::async_trait;
use bytes::Bytes;
use irpc::channel::oneshot;
use irpc::rpc_requests;
use serde::Deserialize;
use serde::Serialize;

use crate::B3Digest;

// ---------------------------------------------------------------------------
// BlobService protocol
// ---------------------------------------------------------------------------

/// Request/response types for the blob content-addressed store.
#[rpc_requests(message = BlobServiceMessage, no_rpc, no_spans)]
#[derive(Debug, Serialize, Deserialize)]
pub enum BlobServiceProtocol {
    /// Check if a blob exists by its BLAKE3 digest.
    #[rpc(tx = oneshot::Sender<Result<bool, String>>)]
    #[wrap(HasBlobRequest)]
    Has(B3Digest),

    /// Read all bytes of a blob. Returns None if not found.
    #[rpc(tx = oneshot::Sender<Result<Option<Vec<u8>>, String>>)]
    #[wrap(ReadBlobRequest)]
    Read(B3Digest),

    /// Upload a blob. Returns its computed BLAKE3 digest.
    #[rpc(tx = oneshot::Sender<Result<B3Digest, String>>)]
    #[wrap(PutBlobRequest)]
    Put(Bytes),

    /// Get chunk metadata for a blob.
    #[rpc(tx = oneshot::Sender<Result<Vec<ChunkMeta>, String>>)]
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
// BlobService client wrapper
// ---------------------------------------------------------------------------

/// An irpc-based BlobService client. Buffers reads/writes in memory.
/// In the in-process path, the `Vec<u8>` moves through the tokio mpsc
/// channel with no serialization — zero-copy semantics.
#[derive(Clone)]
pub struct IrpcBlobService {
    client: irpc::Client<BlobServiceProtocol>,
}

impl IrpcBlobService {
    pub fn new(client: irpc::Client<BlobServiceProtocol>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl crate::blobservice::BlobService for IrpcBlobService {
    async fn has(&self, digest: &B3Digest) -> io::Result<bool> {
        self.client
            .rpc(HasBlobRequest(*digest))
            .await
            .map_err(|e| io::Error::other(e.to_string()))?
            .map_err(io::Error::other)
    }

    async fn open_read(&self, digest: &B3Digest) -> io::Result<Option<Box<dyn crate::blobservice::BlobReader>>> {
        let data = self
            .client
            .rpc(ReadBlobRequest(*digest))
            .await
            .map_err(|e| io::Error::other(e.to_string()))?
            .map_err(io::Error::other)?;

        Ok(data.map(|bytes| -> Box<dyn crate::blobservice::BlobReader> { Box::new(io::Cursor::new(bytes)) }))
    }

    async fn open_write(&self) -> Box<dyn crate::blobservice::BlobWriter> {
        Box::new(IrpcBlobWriter {
            client: self.client.clone(),
            buf: Vec::new(),
            digest: None,
        })
    }

    async fn chunks(&self, digest: &B3Digest) -> io::Result<Option<Vec<crate::proto::stat_blob_response::ChunkMeta>>> {
        let result = self
            .client
            .rpc(ChunksRequest(*digest))
            .await
            .map_err(|e| io::Error::other(e.to_string()))?
            .map_err(io::Error::other)?;

        // Convert rpc::ChunkMeta -> proto::stat_blob_response::ChunkMeta
        Ok(Some(
            result
                .into_iter()
                .map(|c| crate::proto::stat_blob_response::ChunkMeta {
                    digest: Bytes::copy_from_slice(c.digest.as_slice()),
                    size: c.size,
                })
                .collect(),
        ))
    }
}

/// A BlobWriter that buffers locally, then sends the complete blob
/// through the irpc channel on `close()`.
struct IrpcBlobWriter {
    client: irpc::Client<BlobServiceProtocol>,
    buf: Vec<u8>,
    digest: Option<B3Digest>,
}

impl tokio::io::AsyncWrite for IrpcBlobWriter {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        data: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        self.buf.extend_from_slice(data);
        std::task::Poll::Ready(Ok(data.len()))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

#[async_trait]
impl crate::blobservice::BlobWriter for IrpcBlobWriter {
    async fn close(&mut self) -> io::Result<B3Digest> {
        if let Some(digest) = &self.digest {
            return Ok(*digest);
        }

        let data = std::mem::take(&mut self.buf);
        let digest = self
            .client
            .rpc(PutBlobRequest(data.into()))
            .await
            .map_err(|e| io::Error::other(e.to_string()))?
            .map_err(io::Error::other)?;

        self.digest = Some(digest);
        Ok(digest)
    }
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
    async fn get(&self, digest: &B3Digest) -> Result<Option<crate::Directory>, crate::directoryservice::Error> {
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
