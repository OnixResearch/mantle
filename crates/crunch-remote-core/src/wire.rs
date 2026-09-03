use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::RemoteCoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWireHello {
    pub alpn: String,
    pub version: u32,
    pub endpoint_id: String,
    pub capabilities: Vec<String>,
    #[serde(default = "absent_workspace_policy", skip_serializing_if = "Option::is_none")]
    pub workspace_policy: Option<serde_json::Value>,
}

fn absent_workspace_policy() -> Option<serde_json::Value> {
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteWireFailurePhase {
    TransportSetup,
    Authentication,
    RequestValidation,
    InputSync,
    Queue,
    BuildExecution,
    OutputImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteWireRetryClass {
    Retryable,
    Terminal,
    BuildOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWireError {
    pub phase: RemoteWireFailurePhase,
    pub retry_class: RemoteWireRetryClass,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RemoteWireFrame {
    Hello { hello: RemoteWireHello },
    BuildQueued { request_id: String, session_id: String },
    BuildStarted { request_id: String },
    Done { request_id: String },
    Error { error: RemoteWireError },
}

pub fn encode_remote_wire_frame(frame: RemoteWireFrame) -> Result<Vec<u8>, RemoteCoreError> {
    serde_json::to_vec(&frame).map_err(|error| RemoteCoreError::IdentitySerialization {
        reason: error.to_string(),
    })
}

pub fn decode_remote_wire_frame(bytes: Vec<u8>) -> Result<RemoteWireFrame, RemoteCoreError> {
    serde_json::from_slice(&bytes).map_err(|_| RemoteCoreError::invalid("remote-wire-frame-invalid"))
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::REMOTE_CORE_NON_CLAIM;
    use crate::REMOTE_PROTOCOL_ALPN;
    use crate::REMOTE_PROTOCOL_VERSION;
    use crate::REMOTE_RECEIPT_PREIMAGE_SCHEMA;
    use crate::RemoteOutcomeStatus;
    use crate::RemotePhase;
    use crate::RemoteReceiptPreimage;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HELLO_GOLDEN: &str = include_str!("../../../fixtures/remote-hexagon/golden/hello.json");
    const ERROR_GOLDEN: &str = include_str!("../../../fixtures/remote-hexagon/golden/error.json");
    const RECEIPT_GOLDEN: &str = include_str!("../../../fixtures/remote-hexagon/golden/receipt-preimage.json");

    #[test]
    fn accepted_wire_frames_keep_stable_bytes() {
        let hello = RemoteWireFrame::Hello {
            hello: RemoteWireHello {
                alpn: REMOTE_PROTOCOL_ALPN.to_string(),
                version: REMOTE_PROTOCOL_VERSION,
                endpoint_id: "worker-a".to_string(),
                capabilities: vec!["build".to_string()],
                workspace_policy: None,
            },
        };
        let error = RemoteWireFrame::Error {
            error: RemoteWireError {
                phase: RemoteWireFailurePhase::BuildExecution,
                retry_class: RemoteWireRetryClass::Retryable,
                message: "worker-lost".to_string(),
            },
        };
        assert_eq!(encode_remote_wire_frame(hello).unwrap(), HELLO_GOLDEN.as_bytes());
        assert_eq!(encode_remote_wire_frame(error).unwrap(), ERROR_GOLDEN.as_bytes());
    }

    #[test]
    fn receipt_preimage_keeps_stable_bytes() {
        let preimage = RemoteReceiptPreimage {
            schema: REMOTE_RECEIPT_PREIMAGE_SCHEMA.to_string(),
            command_blake3: DIGEST.to_string(),
            job_id: "job-a".to_string(),
            worker_id: "worker-a".to_string(),
            attempt_id: Some("attempt-a".to_string()),
            fence_generation: 1,
            phase: RemotePhase::Succeeded,
            event_reason_codes: vec!["remote-session-succeeded".to_string()],
            pending_effect_blake3: None,
            outcome_status: Some(RemoteOutcomeStatus::Succeeded),
            outcome_reason_code: Some("remote-session-succeeded".to_string()),
            non_claim: REMOTE_CORE_NON_CLAIM.to_string(),
        };
        let encoded = serde_json::to_string(&preimage).expect("receipt preimage serializes");
        assert_eq!(encoded, RECEIPT_GOLDEN);
        assert_eq!(encoded.len(), RECEIPT_GOLDEN.len());
    }

    #[test]
    fn malformed_wire_frame_fails_closed() {
        let malformed = br#"{"kind":"unknown"}"#.to_vec();
        let truncated = br#"{"kind":"done""#.to_vec();
        assert!(decode_remote_wire_frame(malformed).is_err());
        assert!(decode_remote_wire_frame(truncated).is_err());
    }
}
