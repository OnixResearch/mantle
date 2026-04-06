//! irpc service definitions for the store.

use irpc::{channel::oneshot, rpc_requests};
use serde::{Deserialize, Serialize};

use crate::pathinfoservice::PathInfo;

// ---------------------------------------------------------------------------
// PathInfoService
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
