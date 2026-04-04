use std::collections::BTreeMap;

use crate::nodes::Directory;
use crate::{Node, path::PathComponent};
use futures::StreamExt;
use futures::stream::BoxStream;
use tonic::async_trait;

/// Provides an interface for looking up root nodes  in snix-castore by given
/// a lookup key (usually the basename), and optionally allow a listing.
#[async_trait]
pub trait RootNodes {
    type Error: std::error::Error + Send + Sync + 'static;
    /// Looks up a root CA node based on the basename of the node in the root
    /// directory of the filesystem.
    async fn get_by_basename(&self, name: &PathComponent) -> Result<Option<Node>, Self::Error>;

    /// Lists all root CA nodes in the filesystem, as a tuple of (base)name
    /// and Node.
    /// An error can be returned in case listing is not allowed.
    fn list(&self) -> BoxStream<'static, Result<(PathComponent, Node), Self::Error>>;
}

#[async_trait]
/// Implements RootNodes for something deref'ing to a BTreeMap of Nodes, where
/// the key is the node name.
impl<T> RootNodes for T
where
    T: AsRef<BTreeMap<PathComponent, Node>> + Send + Sync,
{
    type Error = std::io::Error; // infallible, really.

    async fn get_by_basename(&self, name: &PathComponent) -> Result<Option<Node>, Self::Error> {
        Ok(self.as_ref().get(name).cloned())
    }

    fn list(&self) -> BoxStream<'static, Result<(PathComponent, Node), Self::Error>> {
        let data = self.as_ref().to_owned();
        futures::stream::iter(data.into_iter().map(Ok)).boxed()
    }
}

#[async_trait]
impl RootNodes for Directory {
    type Error = std::io::Error; // infallible, really.

    async fn get_by_basename(&self, name: &PathComponent) -> Result<Option<Node>, Self::Error> {
        Ok(self
            .nodes()
            .find(|(key, _)| *key == name)
            .map(|(_, node)| node.clone()))
    }

    fn list(&self) -> BoxStream<'static, Result<(PathComponent, Node), Self::Error>> {
        let data = self.to_owned();
        futures::stream::iter(data.into_nodes().map(Ok)).boxed()
    }
}
