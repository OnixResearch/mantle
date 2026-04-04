mod bfs;
mod descend_to;

pub use bfs::root_to_leaves;
pub use descend_to::descend_to;

use crate::B3Digest;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unable to lookup directory {0}")]
    GetFailure(
        B3Digest,
        #[source] Box<dyn std::error::Error + Send + Sync + 'static>,
    ),
    #[error("referenced directory {0} not found")]
    NotFound(B3Digest),
}
