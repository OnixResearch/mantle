//! irpc service definition and client wrapper for the build service.

use async_trait::async_trait;
use irpc::{channel::oneshot, rpc_requests};
use serde::{Deserialize, Serialize};

use crate::buildservice::{BuildRequest, BuildResult, BuildService};

// ---------------------------------------------------------------------------
// Protocol definition
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

// ---------------------------------------------------------------------------
// Client wrapper
// ---------------------------------------------------------------------------

/// An irpc-based BuildService client. Dispatches through a tokio mpsc channel
/// to a local actor that holds the real implementation.
#[derive(Clone)]
pub struct IrpcBuildService {
    client: irpc::Client<BuildServiceProtocol>,
}

impl IrpcBuildService {
    pub fn new(client: irpc::Client<BuildServiceProtocol>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl BuildService for IrpcBuildService {
    async fn do_build(&self, request: BuildRequest) -> std::io::Result<BuildResult> {
        self.client
            .rpc(DoBuildRequest(request))
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .map_err(|e| std::io::Error::other(e))
    }
}
