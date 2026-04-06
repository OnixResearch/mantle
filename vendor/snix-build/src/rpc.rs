//! irpc service definition for the build service.

use irpc::{channel::oneshot, rpc_requests};
use serde::{Deserialize, Serialize};

use crate::buildservice::{BuildRequest, BuildResult};

// ---------------------------------------------------------------------------
// BuildService
// ---------------------------------------------------------------------------

/// Request/response types for the build service.
#[rpc_requests(message = BuildServiceMessage, no_rpc, no_spans)]
#[derive(Debug, Serialize, Deserialize)]
pub enum BuildServiceProtocol {
    /// Execute a build. Returns the build result.
    #[rpc(tx = oneshot::Sender<Result<BuildResult, String>>)]
    #[wrap(DoBuildRequest)]
    DoBuild(BuildRequest),
}
