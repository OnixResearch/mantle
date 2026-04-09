//! irpc service definition and client wrapper for the path info service.

use async_trait::async_trait;
use futures::stream::BoxStream;
use irpc::channel::oneshot;
use irpc::rpc_requests;
use serde::Deserialize;
use serde::Serialize;

use crate::pathinfoservice::PathInfo;
use crate::pathinfoservice::PathInfoService;

// ---------------------------------------------------------------------------
// Protocol definition
// ---------------------------------------------------------------------------

/// Request/response types for the path info store.
#[rpc_requests(message = PathInfoServiceMessage, no_rpc, no_spans)]
#[derive(Debug, Serialize, Deserialize)]
pub enum PathInfoServiceProtocol {
    /// Look up a PathInfo by output hash (20-byte digest).
    #[rpc(tx = oneshot::Sender<Result<Option<PathInfo>, String>>)]
    #[wrap(GetPathInfoRequest)]
    Get([u8; 20]),

    /// Store a PathInfo. Returns the stored PathInfo.
    #[rpc(tx = oneshot::Sender<Result<PathInfo, String>>)]
    #[wrap(PutPathInfoRequest)]
    Put(PathInfo),
}

// ---------------------------------------------------------------------------
// Client wrapper
// ---------------------------------------------------------------------------

/// An irpc-based PathInfoService client.
#[derive(Clone)]
pub struct IrpcPathInfoService {
    client: irpc::Client<PathInfoServiceProtocol>,
}

impl IrpcPathInfoService {
    pub fn new(client: irpc::Client<PathInfoServiceProtocol>) -> Self {
        Self { client }
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
impl PathInfoService for IrpcPathInfoService {
    async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, Error> {
        self.client
            .rpc(GetPathInfoRequest(digest))
            .await
            .map_err(|e| -> Error { e.to_string().into() })?
            .map_err(|e| -> Error { e.into() })
    }

    async fn put(&self, path_info: PathInfo) -> Result<PathInfo, Error> {
        self.client
            .rpc(PutPathInfoRequest(path_info))
            .await
            .map_err(|e| -> Error { e.to_string().into() })?
            .map_err(|e| -> Error { e.into() })
    }

    fn list(&self) -> BoxStream<'static, Result<PathInfo, Error>> {
        // Listing is not supported over irpc yet.
        Box::pin(futures::stream::empty())
    }
}
