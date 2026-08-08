use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use nix_compat::nixbase32;
use snix_castore::composition::CompositionContext;
use snix_castore::composition::ServiceBuilder;
use snix_castore::service_provenance::LayeredRead;
use snix_castore::service_provenance::ReadThroughMode;
use tracing::debug;
use tracing::instrument;

use super::PathInfo;
use super::PathInfoService;
use crate::pathinfoservice;

/// Asks near first, if not found, asks far.
/// If found in there, returns it, and *inserts* it into
/// near (unless `read_only_far` is true).
/// There is no negative cache.
/// Inserts and listings always target the writable near service.
pub struct Cache<PS1, PS2> {
    instance_name: String,
    near: PS1,
    far: PS2,
    mode: ReadThroughMode,
}

impl<PS1, PS2> Cache<PS1, PS2> {
    pub fn new(instance_name: String, near: PS1, far: PS2) -> Self {
        Self::with_mode(instance_name, near, far, ReadThroughMode::Cache)
    }

    /// Create read-through composition that never backfills far hits.
    pub fn new_no_backfill(instance_name: String, near: PS1, far: PS2) -> Self {
        Self::with_mode(instance_name, near, far, ReadThroughMode::NoBackfill)
    }

    pub fn with_mode(instance_name: String, near: PS1, far: PS2, mode: ReadThroughMode) -> Self {
        Self {
            instance_name,
            near,
            far,
            mode,
        }
    }
}

#[async_trait]
impl<PS1, PS2> PathInfoService for Cache<PS1, PS2>
where
    PS1: PathInfoService,
    PS2: PathInfoService,
{
    #[instrument(level = "trace", skip_all, fields(path_info.digest = nixbase32::encode(&digest), instance_name = %self.instance_name))]
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, pathinfoservice::Error> {
        Ok(self.get_with_layer(digest).await?.map(|read| read.value))
    }

    #[instrument(level = "trace", skip_all, fields(path_info.digest = nixbase32::encode(&digest), instance_name = %self.instance_name))]
    async fn get_with_layer(&self, digest: [u8; 20]) -> Result<Option<LayeredRead<PathInfo>>, pathinfoservice::Error> {
        if let Some(path_info) = self.near.get_with_layer(digest).await.map_err(Error::NearGet)? {
            debug!("serving from near service");
            return Ok(Some(path_info));
        }
        debug!("not found in near, asking far service");
        let Some(path_info) = self.far.get_with_layer(digest).await.map_err(Error::FarGet)? else {
            return Ok(None);
        };
        if self.mode == ReadThroughMode::Cache {
            debug!("found in far service, adding to near service");
            self.near.put(path_info.value.clone()).await.map_err(Error::NearPut)?;
        } else {
            debug!("found in far service without backfill");
        }
        Ok(Some(path_info.shift_far().map_err(Error::LayerIndex)?))
    }

    #[instrument(level = "trace", skip_all, fields(path_info.digest = nixbase32::encode(&digest), instance_name = %self.instance_name))]
    async fn has(&self, digest: [u8; 20]) -> Result<bool, pathinfoservice::Error> {
        // FUTUREWORK: queue background tasks if ! self.near.has && self.far.has ? (configurable)
        Ok(
            self.near.has(digest).await.map_err(Error::NearGet)?
                || self.far.has(digest).await.map_err(Error::FarGet)?,
        )
    }

    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, pathinfoservice::Error> {
        self.near.put(path_info).await
    }

    // r[impl vendored_snix.store_service_behavior]
    fn list(&self) -> BoxStream<'static, Result<PathInfo, pathinfoservice::Error>> {
        self.near.list()
    }

    fn list_with_layer(&self) -> BoxStream<'static, Result<LayeredRead<PathInfo>, pathinfoservice::Error>> {
        self.near.list_with_layer()
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheConfig {
    pub near: String,
    pub far: String,
    #[serde(default)]
    pub mode: ReadThroughMode,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("instantiating from a url is not supported")]
    URLNotSupported,

    #[error("getting from near: {0}")]
    NearGet(#[source] pathinfoservice::Error),
    #[error("putting into near: {0}")]
    NearPut(#[source] pathinfoservice::Error),
    #[error("getting from far: {0}")]
    FarGet(#[source] pathinfoservice::Error),
    #[error("tracking far service provenance: {0}")]
    LayerIndex(#[source] snix_castore::service_provenance::LayerIndexOverflow),
}

impl TryFrom<url::Url> for CacheConfig {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_from(_url: url::Url) -> Result<Self, Self::Error> {
        Err(Error::URLNotSupported)?
    }
}

#[async_trait]
impl ServiceBuilder for CacheConfig {
    type Output = dyn PathInfoService;
    async fn build<'a>(
        &'a self,
        instance_name: &str,
        context: &CompositionContext,
    ) -> Result<Arc<Self::Output>, Box<dyn std::error::Error + Send + Sync>> {
        let (near, far) =
            futures::join!(context.resolve::<Self::Output>(&self.near), context.resolve::<Self::Output>(&self.far));
        Ok(Arc::new(Cache {
            instance_name: instance_name.to_string(),
            near: near?,
            far: far?,
            mode: self.mode,
        }))
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroUsize;

    use async_trait::async_trait;
    use futures::TryStreamExt;
    use futures::stream::BoxStream;
    use nix_compat::store_path::StorePath;

    use crate::fixtures::PATH_INFO;
    use crate::pathinfoservice::LruPathInfoService;
    use crate::pathinfoservice::PathInfoService;
    use crate::utils::gen_test_pathinfo_service;

    struct PanicOnListService;

    #[async_trait]
    impl PathInfoService for PanicOnListService {
        async fn get(&self, _digest: [u8; 20]) -> Result<Option<super::PathInfo>, crate::pathinfoservice::Error> {
            Ok(None)
        }

        async fn put(&self, path_info: super::PathInfo) -> Result<super::PathInfo, crate::pathinfoservice::Error> {
            Ok(path_info)
        }

        fn list(&self) -> BoxStream<'static, Result<super::PathInfo, crate::pathinfoservice::Error>> {
            panic!("far service list must not be called")
        }
    }

    /// Helper function setting up an instance of a Cache PathInfoService.
    async fn create_pathinfoservice() -> super::Cache<LruPathInfoService, impl PathInfoService> {
        // Create an instance of a "far" PathInfoService.
        let far = gen_test_pathinfo_service();

        // … and an instance of a "near" PathInfoService.
        let near = LruPathInfoService::with_capacity("near".into(), NonZeroUsize::new(1).unwrap());

        // create a Pathinfoservice combining the two and return it.
        super::Cache::new("root".into(), near, far)
    }

    /// Getting from the far backend is gonna insert it into the near one.
    #[tokio::test]
    async fn test_populate_cache() {
        let svc = create_pathinfoservice().await;

        // query the PathInfo, things should not be there.
        assert!(svc.get(*PATH_INFO.store_path.digest()).await.unwrap().is_none());

        // insert it into the far one.
        svc.far.put(PATH_INFO.clone()).await.unwrap();

        // now try getting it again, it should succeed.
        let read = svc.get_with_layer(*PATH_INFO.store_path.digest()).await.unwrap().unwrap();
        assert_eq!(PATH_INFO.clone(), read.value);
        assert_eq!(read.layer_index, 1);

        // peek near, it should now be there.
        assert_eq!(Some(PATH_INFO.clone()), svc.near.get(*PATH_INFO.store_path.digest()).await.unwrap());
    }

    #[tokio::test]
    async fn no_backfill_reports_far_layer_and_preserves_near_miss() {
        let far = gen_test_pathinfo_service();
        far.put(PATH_INFO.clone()).await.unwrap();
        let near = LruPathInfoService::with_capacity("near".into(), NonZeroUsize::new(1).unwrap());
        let svc = super::Cache::new_no_backfill("root".into(), near, far);

        let read = svc.get_with_layer(*PATH_INFO.store_path.digest()).await.unwrap().unwrap();
        assert_eq!(PATH_INFO.clone(), read.value);
        assert_eq!(read.layer_index, 1);
        assert!(svc.near.get(*PATH_INFO.store_path.digest()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn listing_exposes_writable_near_service_only() {
        // r[verify vendored_snix.store_service_behavior]
        const LIST_CAPACITY: usize = 2;
        const STORE_PATH_DIGEST_BYTES: usize = 20;
        const FAR_DIGEST_BYTE: u8 = 17;
        let far = gen_test_pathinfo_service();
        let mut far_only = PATH_INFO.clone();
        far_only.store_path =
            StorePath::from_name_and_digest_fixed("far-only", [FAR_DIGEST_BYTE; STORE_PATH_DIGEST_BYTES]).unwrap();
        far.put(far_only).await.unwrap();
        let near = LruPathInfoService::with_capacity("near".into(), NonZeroUsize::new(LIST_CAPACITY).unwrap());
        let svc = super::Cache::new_no_backfill("root".into(), near, far);

        assert!(svc.list().try_collect::<Vec<_>>().await.unwrap().is_empty());
        svc.put(PATH_INFO.clone()).await.unwrap();
        let reads = svc.list_with_layer().try_collect::<Vec<_>>().await.unwrap();

        assert_eq!(reads.len(), 1);
        assert_eq!(reads[0].value, *PATH_INFO);
        assert_eq!(reads[0].layer_index, 0);
    }

    #[tokio::test]
    async fn listing_never_calls_far_service() {
        // r[verify vendored_snix.store_service_behavior]
        let near = LruPathInfoService::with_capacity("near".into(), NonZeroUsize::new(1).unwrap());
        let svc = super::Cache::new("root".into(), near, PanicOnListService);

        let listed = svc.list().try_collect::<Vec<_>>().await.unwrap();
        let layered = svc.list_with_layer().try_collect::<Vec<_>>().await.unwrap();

        assert!(listed.is_empty());
        assert!(layered.is_empty());
    }
}
