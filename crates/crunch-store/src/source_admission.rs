//! Store-neutral admission for declared read sources.
//!
//! The shell resolves operator-owned endpoint bindings. Admission grants a live
//! source session only; signatures, content identity and layer generations are
//! still checked by their existing consumers. Every escaped service or reader
//! retains its lease, so dropping a store facade cannot release authority early.
//! A declared local base layer is admitted by the operator's own declaration
//! unless a configured admission authority covers it.

use std::fmt::Debug;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::BoxStream;
use nix_compat::store_path::StorePath;
use snix_castore::B3Digest;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::blobservice::BlobReader;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::BlobWriter;
use snix_castore::directoryservice::DirectoryPutter;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::service_provenance::LayeredRead;
use snix_store::nar::NarCalculationService;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use snix_store::pathinfoservice::{self};
use tokio::io::AsyncRead;
use tokio::io::AsyncSeek;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;

use crate::ActionResultLookup;
use crate::ActionResultPublicationReport;
use crate::ActionResultStore;
use crate::Error;

/// Independently selected source kind and its exact canonical identity.
#[derive(Clone, Copy, Debug)]
pub enum StoreSource<'a> {
    Substituter(&'a str),
    BaseLayer(&'a Path),
}

/// Retained shell authority for one admitted source session.
pub trait StoreSourceLease: Send + Sync + Debug {}

/// Retains the operator's own declaration for a local base layer that no
/// operator-owned endpoint binding covers.
///
/// A base layer is a local, operator-declared directory: its admission rests
/// on that declaration, on the base's own trust policy, and on its read-only
/// generation facts. No external session authority exists for it, so this
/// lease only keeps the local admission witness alive for as long as the
/// composed store holds the layer.
#[derive(Debug)]
pub(crate) struct LocalBaseLayerLease;

impl StoreSourceLease for LocalBaseLayerLease {}

/// Resolves an operator-owned binding without inferring trust from a source.
pub trait StoreSourceAdmission: Send + Sync + Debug {
    fn admit(&self, source: StoreSource<'_>, store_prefix: &str) -> Result<Arc<dyn StoreSourceLease>, Error>;
}

pub(crate) fn admit_source(
    admission: Option<&dyn StoreSourceAdmission>,
    source: StoreSource<'_>,
    store_prefix: &str,
) -> Result<Arc<dyn StoreSourceLease>, Error> {
    let admission = admission.ok_or_else(|| Error::Store("store-source-admission-required".to_string()))?;
    admission.admit(source, store_prefix)
}

/// Keeps the lease alongside the existing implementation, not the facade.
#[derive(Debug)]
pub(crate) struct Leased<T> {
    pub(crate) inner: T,
    pub(crate) lease: Arc<dyn StoreSourceLease>,
}

impl<T> Leased<T> {
    pub(crate) fn new(inner: T, lease: Arc<dyn StoreSourceLease>) -> Self {
        Self { inner, lease }
    }
}

#[async_trait]
impl<T: PathInfoService> PathInfoService for Leased<T> {
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
        self.inner.get(digest).await
    }

    async fn get_with_layer(&self, digest: [u8; 20]) -> Result<Option<LayeredRead<PathInfo>>, pathinfoservice::Error> {
        self.inner.get_with_layer(digest).await
    }

    async fn get_references(&self, digest: [u8; 20]) -> Result<Option<Vec<StorePath<String>>>, pathinfoservice::Error> {
        self.inner.get_references(digest).await
    }

    async fn has(&self, digest: [u8; 20]) -> Result<bool, pathinfoservice::Error> {
        self.inner.has(digest).await
    }

    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
        self.inner.put(path_info).await
    }

    async fn put_batch_atomic(&self, path_infos: Vec<PathInfo>) -> Result<Vec<PathInfo>, pathinfoservice::Error> {
        self.inner.put_batch_atomic(path_infos).await
    }

    fn list(&self) -> BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
        let lease = self.lease.clone();
        self.inner
            .list()
            .map(move |item| {
                let _ = &lease;
                item
            })
            .boxed()
    }

    fn list_with_layer(&self) -> BoxStream<'static, Result<LayeredRead<PathInfo>, pathinfoservice::Error>> {
        let lease = self.lease.clone();
        self.inner
            .list_with_layer()
            .map(move |item| {
                let _ = &lease;
                item
            })
            .boxed()
    }

    fn nar_calculation_service(&self) -> Option<Box<dyn NarCalculationService>> {
        self.inner
            .nar_calculation_service()
            .map(|inner| Box::new(Leased::new(inner, self.lease.clone())) as Box<dyn NarCalculationService>)
    }
}

#[async_trait]
impl<T: NarCalculationService> NarCalculationService for Leased<T> {
    async fn calculate_nar(&self, node: &Node) -> Result<(u64, [u8; 32]), pathinfoservice::Error> {
        self.inner.calculate_nar(node).await
    }
}

#[async_trait]
impl<T: BlobService> BlobService for Leased<T> {
    async fn has(&self, digest: &B3Digest) -> io::Result<bool> {
        self.inner.has(digest).await
    }

    async fn open_read(&self, digest: &B3Digest) -> io::Result<Option<Box<dyn BlobReader>>> {
        Ok(self
            .inner
            .open_read(digest)
            .await?
            .map(|reader| Box::new(Leased::new(reader, self.lease.clone())) as Box<dyn BlobReader>))
    }

    async fn open_read_with_layer(&self, digest: &B3Digest) -> io::Result<Option<LayeredRead<Box<dyn BlobReader>>>> {
        Ok(self.inner.open_read_with_layer(digest).await?.map(|mut value| {
            value.value = Box::new(Leased::new(value.value, self.lease.clone()));
            value
        }))
    }

    async fn chunks(
        &self,
        digest: &B3Digest,
    ) -> io::Result<Option<Vec<snix_castore::proto::stat_blob_response::ChunkMeta>>> {
        self.inner.chunks(digest).await
    }

    async fn chunks_with_layer(
        &self,
        digest: &B3Digest,
    ) -> io::Result<Option<LayeredRead<Vec<snix_castore::proto::stat_blob_response::ChunkMeta>>>> {
        self.inner.chunks_with_layer(digest).await
    }

    async fn open_write(&self) -> Box<dyn BlobWriter> {
        Box::new(Leased::new(self.inner.open_write().await, self.lease.clone()))
    }
}

impl AsyncRead for Leased<Box<dyn BlobReader>> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncSeek for Leased<Box<dyn BlobReader>> {
    fn start_seek(mut self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
        Pin::new(&mut self.inner).start_seek(position)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        Pin::new(&mut self.inner).poll_complete(cx)
    }
}

impl BlobReader for Leased<Box<dyn BlobReader>> {}

impl AsyncWrite for Leased<Box<dyn BlobWriter>> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[async_trait]
impl BlobWriter for Leased<Box<dyn BlobWriter>> {
    async fn close(&mut self) -> io::Result<B3Digest> {
        self.inner.close().await
    }
}

#[async_trait]
impl<T: DirectoryService> DirectoryService for Leased<T> {
    async fn get(&self, digest: &B3Digest) -> Result<Option<Directory>, snix_castore::directoryservice::Error> {
        self.inner.get(digest).await
    }

    async fn get_with_layer(
        &self,
        digest: &B3Digest,
    ) -> Result<Option<LayeredRead<Directory>>, snix_castore::directoryservice::Error> {
        self.inner.get_with_layer(digest).await
    }

    async fn put(&self, directory: Directory) -> Result<B3Digest, snix_castore::directoryservice::Error> {
        self.inner.put(directory).await
    }

    fn get_recursive(
        &self,
        digest: &B3Digest,
    ) -> BoxStream<'_, Result<Directory, snix_castore::directoryservice::Error>> {
        self.inner.get_recursive(digest)
    }

    fn get_recursive_with_layer(
        &self,
        digest: &B3Digest,
    ) -> BoxStream<'_, Result<LayeredRead<Directory>, snix_castore::directoryservice::Error>> {
        self.inner.get_recursive_with_layer(digest)
    }

    fn put_multiple_start(&self) -> Box<dyn DirectoryPutter + '_> {
        self.inner.put_multiple_start()
    }
}

#[async_trait]
impl<T: ActionResultStore> ActionResultStore for Leased<T> {
    fn source_id(&self) -> &str {
        self.inner.source_id()
    }
    fn source_class(&self) -> &str {
        self.inner.source_class()
    }

    async fn lookup(&self, action_ref: &str) -> Result<ActionResultLookup, String> {
        self.inner.lookup(action_ref).await
    }

    async fn publish(
        &self,
        record: &crunch_action_result_core::SignedActionResultRecord,
    ) -> Result<ActionResultPublicationReport, String> {
        self.inner.publish(record).await
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct TestSourceAdmission;

#[cfg(test)]
impl StoreSourceLease for TestSourceAdmission {}

#[cfg(test)]
impl StoreSourceAdmission for TestSourceAdmission {
    fn admit(&self, _source: StoreSource<'_>, _store_prefix: &str) -> Result<Arc<dyn StoreSourceLease>, Error> {
        Ok(Arc::new(Self))
    }
}
