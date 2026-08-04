//! Build-output rewriting through the bounded store capability.

use snix_castore::Node;

use crate::Error;

/// Rewrite equal-length byte sequences in a castore node.
pub async fn rewrite_node(
    store: &crunch_store::BuildStore,
    node: &Node,
    old_bytes: &[u8],
    new_bytes: &[u8],
) -> Result<(Node, bool), Error> {
    store
        .rewrite_node(node, old_bytes, new_bytes)
        .await
        .map_err(|error| Error::Store(format!("rewriting build output: {error}")))
}
