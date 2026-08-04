use std::io;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::instrument;

use super::BlobReader;
use super::BlobService;
use super::BlobWriter;
use super::ChunkedReader;
use crate::B3Digest;
use crate::composition::CompositionContext;
use crate::composition::ServiceBuilder;
use crate::proto::stat_blob_response::ChunkMeta;
use crate::service_provenance::LayeredRead;
use crate::service_provenance::ReadThroughMode;

/// Composes a near and far blob service.
///
/// Cache mode preserves chunk reuse across near and far. No-backfill mode reads
/// a far blob directly, so one read never mixes chunks from different layers.
pub struct CombinedBlobService<BL, BR> {
    instance_name: String,
    near: BL,
    far: BR,
    mode: ReadThroughMode,
}

impl<BL, BR> CombinedBlobService<BL, BR> {
    pub fn new(instance_name: String, near: BL, far: BR) -> Self {
        Self::with_mode(instance_name, near, far, ReadThroughMode::Cache)
    }

    pub fn new_no_backfill(instance_name: String, near: BL, far: BR) -> Self {
        Self::with_mode(instance_name, near, far, ReadThroughMode::NoBackfill)
    }

    pub fn with_mode(instance_name: String, near: BL, far: BR, mode: ReadThroughMode) -> Self {
        Self {
            instance_name,
            near,
            far,
            mode,
        }
    }
}

impl<BL, BR> Clone for CombinedBlobService<BL, BR>
where
    BL: Clone,
    BR: Clone,
{
    fn clone(&self) -> Self {
        Self {
            instance_name: self.instance_name.clone(),
            near: self.near.clone(),
            far: self.far.clone(),
            mode: self.mode,
        }
    }
}

#[async_trait]
impl<BL, BR> BlobService for CombinedBlobService<BL, BR>
where
    BL: AsRef<dyn BlobService> + Clone + Send + Sync + 'static,
    BR: AsRef<dyn BlobService> + Clone + Send + Sync + 'static,
{
    #[instrument(skip(self, digest), fields(blob.digest=%digest, instance_name=%self.instance_name))]
    async fn has(&self, digest: &B3Digest) -> io::Result<bool> {
        Ok(self.near.as_ref().has(digest).await? || self.far.as_ref().has(digest).await?)
    }

    #[instrument(skip(self, digest), fields(blob.digest=%digest, instance_name=%self.instance_name), err)]
    async fn open_read(&self, digest: &B3Digest) -> io::Result<Option<Box<dyn BlobReader>>> {
        Ok(self.open_read_with_layer(digest).await?.map(|read| read.value))
    }

    #[instrument(skip(self, digest), fields(blob.digest=%digest, instance_name=%self.instance_name), err)]
    async fn open_read_with_layer(&self, digest: &B3Digest) -> io::Result<Option<LayeredRead<Box<dyn BlobReader>>>> {
        if self.near.as_ref().has(digest).await? {
            let read = self.near.as_ref().open_read_with_layer(digest).await?;
            if read.is_some() || self.mode == ReadThroughMode::Cache {
                return Ok(read);
            }
            return Err(incomplete_blob("near"));
        }
        if self.mode == ReadThroughMode::NoBackfill {
            if !self.far.as_ref().has(digest).await? {
                return Ok(None);
            }
            return self
                .far
                .as_ref()
                .open_read_with_layer(digest)
                .await?
                .map_or_else(|| Err(incomplete_blob("far")), |read| Ok(Some(read.shift_far().map_err(layer_error)?)));
        }

        let Some(far_chunks) = self.far.as_ref().chunks_with_layer(digest).await? else {
            return Ok(None);
        };
        let far_layer = far_chunks.layer_index.checked_add(1).ok_or_else(layer_index_overflow)?;
        if far_chunks.value.is_empty() {
            let far_read =
                self.far.as_ref().open_read_with_layer(digest).await?.ok_or_else(|| incomplete_blob("far"))?;
            let shifted = far_read.shift_far().map_err(layer_error)?;
            if shifted.layer_index != far_layer {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "far blob metadata and reader resolved from different layers",
                ));
            }
            return Ok(Some(shifted));
        }
        let mut chunks = Vec::with_capacity(far_chunks.value.len());
        for chunk in far_chunks.value {
            let digest = chunk.digest.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "far blob service returned an invalid BLAKE3 chunk digest")
            })?;
            chunks.push((digest, chunk.size));
        }
        let chunked_reader =
            ChunkedReader::from_chunks(chunks.into_iter(), Arc::new(self.clone()) as Arc<dyn BlobService>);
        Ok(Some(LayeredRead {
            value: Box::new(chunked_reader),
            layer_index: far_layer,
        }))
    }

    #[instrument(skip_all, fields(instance_name=%self.instance_name))]
    async fn open_write(&self) -> Box<dyn BlobWriter> {
        self.near.as_ref().open_write().await
    }

    async fn chunks(&self, digest: &B3Digest) -> io::Result<Option<Vec<ChunkMeta>>> {
        Ok(self.chunks_with_layer(digest).await?.map(|read| read.value))
    }

    async fn chunks_with_layer(&self, digest: &B3Digest) -> io::Result<Option<LayeredRead<Vec<ChunkMeta>>>> {
        if let Some(chunks) = self.near.as_ref().chunks_with_layer(digest).await? {
            return Ok(Some(chunks));
        }
        self.far
            .as_ref()
            .chunks_with_layer(digest)
            .await?
            .map(|chunks| chunks.shift_far().map_err(layer_error))
            .transpose()
    }
}

fn incomplete_blob(layer: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{layer} blob service reported a blob but returned no reader"))
}

fn layer_error(error: crate::service_provenance::LayerIndexOverflow) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn layer_index_overflow() -> io::Error {
    layer_error(crate::service_provenance::LayerIndexOverflow)
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CombinedBlobServiceConfig {
    near: String,
    far: String,
    #[serde(default)]
    mode: ReadThroughMode,
}

impl TryFrom<url::Url> for CombinedBlobServiceConfig {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(_url: url::Url) -> Result<Self, Self::Error> {
        Err("Instantiating a CombinedBlobService from a url is not supported".into())
    }
}

#[async_trait]
impl ServiceBuilder for CombinedBlobServiceConfig {
    type Output = dyn BlobService;

    async fn build<'a>(
        &'a self,
        instance_name: &str,
        context: &CompositionContext,
    ) -> Result<Arc<Self::Output>, Box<dyn std::error::Error + Send + Sync>> {
        let (local, remote) = futures::join!(context.resolve(&self.near), context.resolve(&self.far));
        Ok(Arc::new(CombinedBlobService {
            instance_name: instance_name.to_string(),
            near: local?,
            far: remote?,
            mode: self.mode,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::fixtures::BLOB_A;
    use crate::fixtures::BLOB_A_DIGEST;

    const FAR_LAYER_INDEX: usize = 1;

    async fn memory_service() -> Arc<dyn BlobService> {
        crate::blobservice::from_addr("memory:").await.expect("memory blob service must open")
    }

    async fn write_blob(service: &Arc<dyn BlobService>) {
        let mut writer = service.open_write().await;
        tokio::io::copy(&mut Cursor::new(&*BLOB_A), &mut writer).await.expect("fixture blob must write");
        let digest = writer.close().await.expect("fixture blob must close");
        assert_eq!(digest, *BLOB_A_DIGEST);
    }

    #[tokio::test]
    async fn no_backfill_reports_far_layer_without_near_blob() {
        let near = memory_service().await;
        let far = memory_service().await;
        write_blob(&far).await;
        let service = CombinedBlobService::new_no_backfill("overlay".to_string(), near.clone(), far);

        let read = service.open_read_with_layer(&BLOB_A_DIGEST).await.unwrap().unwrap();
        assert_eq!(read.layer_index, FAR_LAYER_INDEX);
        assert!(!near.has(&BLOB_A_DIGEST).await.unwrap());
    }

    #[tokio::test]
    async fn explicit_write_routes_to_near_only() {
        let near = memory_service().await;
        let far = memory_service().await;
        let service = CombinedBlobService::new_no_backfill("overlay".to_string(), near.clone(), far.clone());
        let service: Arc<dyn BlobService> = Arc::new(service);

        write_blob(&service).await;
        assert!(near.has(&BLOB_A_DIGEST).await.unwrap());
        assert!(!far.has(&BLOB_A_DIGEST).await.unwrap());
    }
}
