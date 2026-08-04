use std::num::NonZeroUsize;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use tracing::instrument;
use tracing::trace;

use super::Directory;
use super::DirectoryService;
use crate::B3Digest;
use crate::composition::CompositionContext;
use crate::composition::ServiceBuilder;
use crate::directoryservice::DirectoryPutter;
use crate::directoryservice::directory_graph::DirectoryGraphBuilder;
use crate::service_provenance::LayeredRead;
use crate::service_provenance::ReadThroughMode;

/// Asks near first and asks far only after a near miss.
///
/// Cache mode preserves the existing far-to-near closure backfill. No-backfill
/// mode validates a bounded far closure and returns it without near mutation.
pub struct Cache<DS1, DS2> {
    instance_name: String,
    near: DS1,
    far: DS2,
    mode: ReadThroughMode,
    far_read_limit: Option<NonZeroUsize>,
}

impl<DS1, DS2> Cache<DS1, DS2> {
    pub fn new(instance_name: String, near: DS1, far: DS2) -> Self {
        Self {
            instance_name,
            near,
            far,
            mode: ReadThroughMode::Cache,
            far_read_limit: None,
        }
    }

    /// Create bounded read-through composition that never backfills far hits.
    pub fn new_no_backfill(instance_name: String, near: DS1, far: DS2, far_read_limit: NonZeroUsize) -> Self {
        Self {
            instance_name,
            near,
            far,
            mode: ReadThroughMode::NoBackfill,
            far_read_limit: Some(far_read_limit),
        }
    }
}

impl<DS1, DS2> Cache<DS1, DS2>
where
    DS1: DirectoryService + Clone + 'static,
    DS2: DirectoryService + Clone + 'static,
{
    async fn collect_far_closure(&self, digest: &B3Digest) -> Result<Vec<LayeredRead<Directory>>, super::Error> {
        let mut stream = self.far.get_recursive_with_layer(digest);
        let mut directories = Vec::new();
        while let Some(directory) = stream.try_next().await.map_err(Error::FarGet)? {
            if self.far_read_limit.is_some_and(|limit| directories.len() >= limit.get()) {
                return Err(Error::FarReadLimit {
                    maximum: self.far_read_limit.map_or(0, NonZeroUsize::get),
                }
                .into());
            }
            directories.push(directory.shift_far().map_err(Error::LayerIndex)?);
        }
        if directories.is_empty() {
            return Ok(directories);
        }
        let selected_layer = directories[0].layer_index;
        if directories.iter().any(|directory| directory.layer_index != selected_layer) {
            return Err(Error::MixedLayerClosure.into());
        }
        let mut graph_builder = DirectoryGraphBuilder::new_root_to_leaves(*digest);
        for directory in &directories {
            graph_builder.try_insert(directory.value.clone())?;
        }
        let directory_graph = graph_builder.build()?;
        if self.mode == ReadThroughMode::Cache {
            let mut near_putter = self.near.put_multiple_start();
            for directory in directory_graph.drain_leaves_to_root() {
                near_putter.put(directory).await.map_err(Error::NearPut)?;
            }
            let actual_digest = near_putter.close().await.map_err(Error::NearPut)?;
            if &actual_digest != digest {
                return Err(Error::BackfillDigestMismatch.into());
            }
        }
        Ok(directories)
    }
}

#[async_trait]
impl<DS1, DS2> DirectoryService for Cache<DS1, DS2>
where
    DS1: DirectoryService + Clone + 'static,
    DS2: DirectoryService + Clone + 'static,
{
    #[instrument(skip(self, digest), fields(directory.digest = %digest, instance_name = %self.instance_name))]
    async fn get(&self, digest: &B3Digest) -> Result<Option<Directory>, super::Error> {
        Ok(self.get_with_layer(digest).await?.map(|read| read.value))
    }

    #[instrument(skip(self, digest), fields(directory.digest = %digest, instance_name = %self.instance_name))]
    async fn get_with_layer(&self, digest: &B3Digest) -> Result<Option<LayeredRead<Directory>>, super::Error> {
        if let Some(directory) = self.near.get_with_layer(digest).await.map_err(Error::NearGet)? {
            trace!("serving directory from near service");
            return Ok(Some(directory));
        }
        trace!("directory not found in near service, asking far service");
        Ok(self.collect_far_closure(digest).await?.into_iter().next())
    }

    #[instrument(skip_all, fields(instance_name = %self.instance_name))]
    async fn put(&self, directory: Directory) -> Result<B3Digest, super::Error> {
        self.near.put(directory).await
    }

    #[instrument(skip_all, fields(directory.digest = %root_directory_digest, instance_name = %self.instance_name))]
    fn get_recursive(&self, root_directory_digest: &B3Digest) -> BoxStream<'_, Result<Directory, super::Error>> {
        self.get_recursive_with_layer(root_directory_digest).map_ok(|read| read.value).boxed()
    }

    #[instrument(skip_all, fields(directory.digest = %root_directory_digest, instance_name = %self.instance_name))]
    fn get_recursive_with_layer(
        &self,
        root_directory_digest: &B3Digest,
    ) -> BoxStream<'_, Result<LayeredRead<Directory>, super::Error>> {
        let digest = *root_directory_digest;
        async_stream::try_stream! {
            let mut near_directories = self.near.get_recursive_with_layer(&digest);
            if let Some(first) = near_directories.try_next().await.map_err(Error::NearGet)? {
                trace!("serving directory closure from near service");
                yield first;
                while let Some(directory) = near_directories.try_next().await.map_err(Error::NearGet)? {
                    yield directory;
                }
                return;
            }

            trace!("directory closure not found in near service, asking far service");
            let far_directories = self.collect_far_closure(&digest).await?;
            for directory in far_directories {
                yield directory;
            }
        }
        .boxed()
    }

    #[instrument(skip_all)]
    fn put_multiple_start(&self) -> Box<dyn DirectoryPutter + '_> {
        self.near.put_multiple_start()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("wrong arguments: {0}")]
    WrongConfig(&'static str),
    #[error("serde-qs error: {0}")]
    SerdeQS(#[from] serde_qs::Error),
    #[error("getting from near: {0}")]
    NearGet(#[source] super::Error),
    #[error("putting into near: {0}")]
    NearPut(#[source] super::Error),
    #[error("getting from far: {0}")]
    FarGet(#[source] super::Error),
    #[error("tracking far service provenance: {0}")]
    LayerIndex(#[source] crate::service_provenance::LayerIndexOverflow),
    #[error("far directory closure exceeds the configured limit of {maximum}")]
    FarReadLimit { maximum: usize },
    #[error("one directory closure spans multiple service layers")]
    MixedLayerClosure,
    #[error("backfilled directory closure digest differs from the requested digest")]
    BackfillDigestMismatch,
}

#[derive(serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CacheConfig {
    near: String,
    far: String,
    #[serde(default)]
    mode: ReadThroughMode,
    far_read_limit: Option<NonZeroUsize>,
}

impl TryFrom<url::Url> for CacheConfig {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(url: url::Url) -> Result<Self, Self::Error> {
        if url.has_authority() || !url.path().is_empty() {
            return Err(Error::WrongConfig("no authority or path allowed").into());
        }
        Ok(serde_qs::from_str(url.query().unwrap_or_default())?)
    }
}

#[async_trait]
impl ServiceBuilder for CacheConfig {
    type Output = dyn DirectoryService;

    async fn build<'a>(
        &'a self,
        instance_name: &str,
        context: &CompositionContext,
    ) -> Result<Arc<Self::Output>, Box<dyn std::error::Error + Send + Sync>> {
        if self.mode == ReadThroughMode::NoBackfill && self.far_read_limit.is_none() {
            return Err(Error::WrongConfig("no-backfill mode requires far_read_limit").into());
        }
        let (near, far) =
            futures::join!(context.resolve::<Self::Output>(&self.near), context.resolve::<Self::Output>(&self.far));
        Ok(Arc::new(Cache {
            instance_name: instance_name.to_string(),
            near: near?,
            far: far?,
            mode: self.mode,
            far_read_limit: self.far_read_limit,
        }))
    }
}

#[cfg(test)]
mod tests {
    use futures::TryStreamExt;

    use super::*;
    use crate::fixtures::DIRECTORY_A;
    use crate::fixtures::DIRECTORY_C;

    const ONE_DIRECTORY: usize = 1;
    const COMPLETE_TEST_CLOSURE: usize = 2;

    async fn memory_service() -> Arc<dyn DirectoryService> {
        crate::directoryservice::from_addr("redb+memory:").await.expect("memory service must open")
    }

    async fn put_closure(service: &Arc<dyn DirectoryService>) {
        let mut putter = service.put_multiple_start();
        putter.put(DIRECTORY_A.clone()).await.expect("leaf must write");
        putter.put(DIRECTORY_C.clone()).await.expect("root must write");
        let digest = putter.close().await.expect("closure must close");
        assert_eq!(digest, DIRECTORY_C.digest());
    }

    #[tokio::test]
    async fn cache_mode_preserves_far_to_near_backfill() {
        let near = memory_service().await;
        let far = memory_service().await;
        put_closure(&far).await;
        let service = Cache::new("cache".to_string(), near.clone(), far);

        let read = service.get_with_layer(&DIRECTORY_C.digest()).await.unwrap().unwrap();
        assert_eq!(read.layer_index, ONE_DIRECTORY);
        assert_eq!(read.value, DIRECTORY_C.clone());
        assert!(near.get(&DIRECTORY_C.digest()).await.unwrap().is_some());
        assert!(near.get(&DIRECTORY_A.digest()).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn no_backfill_reports_far_layer_and_preserves_near_miss() {
        let near = memory_service().await;
        let far = memory_service().await;
        put_closure(&far).await;
        let limit = NonZeroUsize::new(COMPLETE_TEST_CLOSURE).expect("test limit is positive");
        let service = Cache::new_no_backfill("overlay".to_string(), near.clone(), far, limit);

        let read = service.get_with_layer(&DIRECTORY_C.digest()).await.unwrap().unwrap();
        assert_eq!(read.layer_index, ONE_DIRECTORY);
        assert_eq!(read.value, DIRECTORY_C.clone());
        assert!(near.get(&DIRECTORY_C.digest()).await.unwrap().is_none());
        assert!(near.get(&DIRECTORY_A.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn no_backfill_rejects_closure_above_bound_before_returning_data() {
        let near = memory_service().await;
        let far = memory_service().await;
        put_closure(&far).await;
        let limit = NonZeroUsize::new(ONE_DIRECTORY).expect("test limit is positive");
        let service = Cache::new_no_backfill("overlay".to_string(), near, far, limit);

        let error = service
            .get_recursive_with_layer(&DIRECTORY_C.digest())
            .try_collect::<Vec<_>>()
            .await
            .expect_err("oversized closure must fail");
        assert!(error.to_string().contains("exceeds the configured limit"));
    }
}
