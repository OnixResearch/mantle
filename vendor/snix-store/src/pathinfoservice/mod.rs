use async_trait::async_trait;
mod cache;
mod from_addr;
mod lru;
mod nix_http;
mod redb;
mod signing_wrapper;

#[cfg(test)]
mod tests;

use auto_impl::auto_impl;
use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use nix_compat::store_path::StorePath;
use snix_castore::composition::Registry;
use snix_castore::composition::ServiceBuilder;
use snix_castore::service_provenance::LayeredRead;

pub use self::cache::Cache as CachePathInfoService;
pub use self::cache::CacheConfig as CachePathInfoServiceConfig;
pub use self::from_addr::from_addr;
pub use self::lru::LruPathInfoService;
pub use self::lru::LruPathInfoServiceConfig;
pub use self::nix_http::NixHTTPPathInfoService;
pub use self::nix_http::NixHTTPPathInfoServiceConfig;
pub use self::redb::RedbPathInfoService;
pub use self::redb::RedbPathInfoServiceConfig;
pub use self::signing_wrapper::KeyFileSigningPathInfoServiceConfig;
pub use self::signing_wrapper::SigningPathInfoService;
#[cfg(test)]
pub(crate) use self::signing_wrapper::test_signing_service;
use crate::nar::NarCalculationService;
pub use crate::path_info::PathInfo;

#[cfg(feature = "cloud")]
mod bigtable;
#[cfg(feature = "cloud")]
pub use self::bigtable::BigtableParameters;
#[cfg(feature = "cloud")]
pub use self::bigtable::BigtablePathInfoService;

#[cfg(any(feature = "fuse", feature = "virtiofs"))]
mod fs;

#[cfg(any(feature = "fuse", feature = "virtiofs"))]
pub use self::fs::RootNodesWrapper;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The base trait all PathInfo services need to implement.

#[async_trait]
#[auto_impl(&, &mut, Arc, Box)]
pub trait PathInfoService: Send + Sync {
    /// Retrieve a PathInfo by the output digest.
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, Error>;

    /// Retrieve a PathInfo and its zero-based near/far precedence index.
    async fn get_with_layer(&self, digest: [u8; 20]) -> Result<Option<LayeredRead<PathInfo>>, Error> {
        Ok(self.get(digest).await?.map(LayeredRead::local))
    }

    /// Retrieve only the referenced store paths for a digest.
    ///
    /// The default implementation falls back to `get()` and extracts
    /// `PathInfo.references`, but remote implementations may override this to
    /// avoid fetching or materializing the full object payload.
    async fn get_references(&self, digest: [u8; 20]) -> Result<Option<Vec<StorePath<String>>>, Error> {
        Ok(self.get(digest).await?.map(|path_info| path_info.references))
    }

    /// Check if a PathInfo exists.
    /// Has a naïve default impl, but store implementations may decide to
    /// implement their own.
    async fn has(&self, digest: [u8; 20]) -> Result<bool, Error> {
        Ok(self.get(digest).await?.is_some())
    }

    /// Store a PathInfo.
    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, Error>;

    /// Iterate over all PathInfo objects in the store.
    /// Implementations can decide to disallow listing.
    ///
    /// This returns a pinned, boxed stream. The pinning allows for it to be polled easily,
    /// and the box allows different underlying stream implementations to be returned since
    /// Rust doesn't support this as a generic in traits yet. This is the same thing that
    /// [async_trait] generates, but for streams instead of futures.
    ///
    /// Even though this function is not async, underlying implementations are
    /// assumed to be nonblocking on IO, so they MUST use spawn_blocking when
    /// doing IO.
    /// Implementations can assume to be invoked in the context of a tokio runtime.
    fn list(&self) -> BoxStream<'static, Result<PathInfo, Error>>;

    /// List PathInfos with their zero-based near/far precedence indexes.
    fn list_with_layer(&self) -> BoxStream<'static, Result<LayeredRead<PathInfo>, Error>> {
        self.list().map_ok(LayeredRead::local).boxed()
    }

    /// Returns a (more) suitable NarCalculationService.
    /// This can be used to offload NAR calculation to the remote side.
    fn nar_calculation_service(&self) -> Option<Box<dyn NarCalculationService>> {
        None
    }
}

/// Registers the builtin PathInfoService implementations with the registry
pub(crate) fn register_pathinfo_services(reg: &mut Registry) {
    reg.register::<Box<dyn ServiceBuilder<Output = dyn PathInfoService>>, CachePathInfoServiceConfig>("cache");
    reg.register::<Box<dyn ServiceBuilder<Output = dyn PathInfoService>>, KeyFileSigningPathInfoServiceConfig>(
        "keyfile-signing",
    );
    reg.register::<Box<dyn ServiceBuilder<Output = dyn PathInfoService>>, LruPathInfoServiceConfig>("lru");
    reg.register::<Box<dyn ServiceBuilder<Output = dyn PathInfoService>>, NixHTTPPathInfoServiceConfig>("nix");
    reg.register::<Box<dyn ServiceBuilder<Output = dyn PathInfoService>>, RedbPathInfoServiceConfig>("redb");
    #[cfg(feature = "cloud")]
    {
        reg.register::<Box<dyn ServiceBuilder<Output = dyn PathInfoService>>, BigtableParameters>("bigtable");
    }
}
