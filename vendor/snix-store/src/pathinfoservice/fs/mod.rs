use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use nix_compat::store_path::StorePathRef;
use snix_castore::fs::RootNodes;
use snix_castore::{Node, PathComponent};
use tonic::async_trait;

use super::PathInfoService;

/// Wrapper to satisfy Rust's orphan rules for trait implementations, as
/// RootNodes is coming from the [snix-castore] crate.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct RootNodesWrapper<T>(T);

impl<T> From<T> for RootNodesWrapper<T>
where
    T: PathInfoService,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Error(#[from] super::Error);

/// Implements root node lookup for any [PathInfoService]. This represents a flat
/// directory structure like /nix/store where each entry in the root filesystem
/// directory corresponds to a CA node.
#[cfg(any(feature = "fuse", feature = "virtiofs"))]
#[async_trait]
impl<T> RootNodes for RootNodesWrapper<T>
where
    T: PathInfoService,
{
    type Error = Error;

    async fn get_by_basename(&self, name: &PathComponent) -> Result<Option<Node>, Self::Error> {
        let Ok(store_path) = StorePathRef::from_bytes(name.as_ref()) else {
            return Ok(None);
        };

        Ok(self
            .0
            .get(*store_path.digest())
            .await?
            .map(|path_info| path_info.node))
    }

    fn list(&self) -> BoxStream<'static, Result<(PathComponent, Node), Self::Error>> {
        self.0
            .list()
            .map_ok(|path_info| {
                let name = path_info
                    .store_path
                    .to_string()
                    .as_str()
                    .try_into()
                    .expect("Snix bug: StorePath must be PathComponent");
                (name, path_info.node)
            })
            .err_into()
            .boxed()
    }
}
