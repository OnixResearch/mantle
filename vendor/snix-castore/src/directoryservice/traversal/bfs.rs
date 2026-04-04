use super::Error;
use crate::{B3Digest, Directory, Node};
use futures::StreamExt;
use std::collections::{HashSet, VecDeque};
use tracing::instrument;
use tracing::warn;

/// Traverses a [Directory] from the root to the children.
///
/// This is mostly BFS, but directories are only returned once.
#[instrument(skip(get_directory))]
pub fn root_to_leaves<F, Fut>(
    root_directory_digest: B3Digest,
    get_directory: F,
) -> impl futures::Stream<Item = Result<Directory, Error>> + use<F, Fut>
where
    F: Fn(B3Digest) -> Fut + Sync + Send + 'static,
    Fut: Future<Output = Result<Option<Directory>, crate::directoryservice::Error>> + Send,
{
    // The list of all directories that still need to be traversed. The next
    // element is picked from the front, new elements are enqueued at the
    // back.
    let mut worklist_directory_digests = VecDeque::from([root_directory_digest]);
    // The list of directory digests already sent to the consumer.
    // We omit sending the same directories multiple times.
    let mut sent_directory_digests: HashSet<B3Digest> = HashSet::new();

    async_stream::try_stream! {
        while let Some(current_directory_digest) = worklist_directory_digests.pop_front() {
            let current_directory = match get_directory(current_directory_digest).await.map_err(|e| {
                Error::GetFailure(current_directory_digest, e)
            })? {
                // the root node of the requested closure was not found, return an empty list
                None if current_directory_digest == root_directory_digest => break,
                // if a child directory of the closure is not there, we have an inconsistent store!
                None => {
                    Err(Error::NotFound(current_directory_digest))?;
                    break;
                }
                Some(dir) => dir,
            };

            // We're about to send this directory, so let's avoid sending it again if a
            // descendant has it.
            sent_directory_digests.insert(current_directory_digest);

            // enqueue all child directory digests to the work queue, as
            // long as they're not part of the worklist or already sent.
            // This panics if the digest looks invalid, it's supposed to be checked first.
            for (_, child_directory_node) in current_directory.nodes() {
                if let Node::Directory{digest: child_digest, ..} = child_directory_node {
                    if worklist_directory_digests.contains(child_digest)
                        || sent_directory_digests.contains(child_digest)
                    {
                        continue;
                    }
                    worklist_directory_digests.push_back(*child_digest);
                }
            }

            yield current_directory;
        }
    }.boxed()
}
