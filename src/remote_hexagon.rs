//! Outer adapters between existing remote-build protocols/providers and the
//! Mantle-owned remote application contracts.

// r[impl remote_builds.hexagonal_compatibility]
pub mod attempt_adapter;
pub mod authority_adapter;
pub mod executor_adapter;
pub mod store_adapter;
pub mod telemetry_adapter;
pub mod transport_adapter;

use crunch_remote::RemoteCommand;
use crunch_remote::RemoteEffect;
use crunch_remote::RemoteObservation;
use crunch_remote::RemoteOutputExpectation;
use crunch_remote::RemoteOutputFact;
use crunch_remote::RemotePolicy;
use crunch_remote::RemoteSubstitutionFact;
use crunch_remote::RemoteSubstitutionMode;
use crunch_remote::RemoteWireError;
use crunch_remote::RemoteWireFailurePhase;
use crunch_remote::RemoteWireFrame;
use crunch_remote::RemoteWireHello;
use crunch_remote::RemoteWireRetryClass;

use crate::remote_build::RemoteCoordinatorBuildRequest;
use crate::remote_build::RemoteExecutionOutput;
use crate::remote_build::RemoteFailurePhase;
use crate::remote_build::RemoteFrame;
use crate::remote_build::RemoteRetryClass;

pub fn command_from_coordinator_request(
    request: &RemoteCoordinatorBuildRequest,
    job_id: String,
    worker_id: String,
    fence_generation: u64,
    policy: RemotePolicy,
) -> Result<RemoteCommand, String> {
    let request_blake3 = crate::remote_build::normalized_remote_build_key(request)?;
    let expected_outputs = request
        .request
        .expected_outputs
        .iter()
        .map(|output| RemoteOutputExpectation {
            name: output.name.clone(),
            logical_path: output.logical_path.clone(),
        })
        .collect();
    Ok(RemoteCommand {
        schema: crunch_remote::REMOTE_COMMAND_SCHEMA.to_string(),
        job_id,
        request_blake3,
        worker_id,
        fence_generation,
        input_refs: request.request.input_refs.clone(),
        expected_outputs,
        policy,
    })
}

pub fn project_wire_frame(frame: &RemoteFrame) -> Option<RemoteWireFrame> {
    match frame {
        RemoteFrame::Hello { hello } => Some(RemoteWireFrame::Hello {
            hello: RemoteWireHello {
                alpn: hello.alpn.clone(),
                version: hello.version,
                endpoint_id: hello.endpoint_id.clone(),
                capabilities: hello.capabilities.clone(),
                workspace_policy: hello.workspace_policy.as_ref().and_then(|policy| serde_json::to_value(policy).ok()),
            },
        }),
        RemoteFrame::BuildQueued { request_id, session_id } => Some(RemoteWireFrame::BuildQueued {
            request_id: request_id.clone(),
            session_id: session_id.clone(),
        }),
        RemoteFrame::BuildStarted { request_id } => Some(RemoteWireFrame::BuildStarted {
            request_id: request_id.clone(),
        }),
        RemoteFrame::Done { request_id } => Some(RemoteWireFrame::Done {
            request_id: request_id.clone(),
        }),
        RemoteFrame::Error { error } => Some(RemoteWireFrame::Error {
            error: RemoteWireError {
                phase: project_failure_phase(error.phase),
                retry_class: project_retry_class(error.retry_class),
                message: error.message.clone(),
            },
        }),
        _ => None,
    }
}

pub fn output_fact_from_execution(
    output: &RemoteExecutionOutput,
    substitution: Option<&crunch_store::OutputSubstitutionReport>,
) -> Result<RemoteOutputFact, String> {
    let path_info_blake3 = output.path_info.as_ref().map(path_info_identity).transpose()?;
    Ok(RemoteOutputFact {
        name: output.name.clone(),
        logical_path: output.logical_path.clone(),
        content_digest_blake3: output.content_digest_blake3.clone(),
        size_bytes: output.size_bytes,
        artifact_attestation_blake3: output.artifact_attestation_digest_blake3.clone(),
        path_info_blake3,
        substitution: substitution.map(project_substitution),
    })
}

pub fn output_admission_observation(
    effect: &RemoteEffect,
    report: &crate::remote_build::RemoteOutputAdmissionReport,
) -> Result<RemoteObservation, String> {
    let observed_outputs =
        u32::try_from(report.outputs.len()).map_err(|_| "remote-output-admission-count-overflow".to_string())?;
    let mut observation = crunch_remote::successful_observation(effect);
    observation.observed_bytes = report.transfer.transferred_bytes;
    observation.observed_outputs = observed_outputs;
    observation.output_trusted = true;
    debug_assert_eq!(observation.kind, crunch_remote::RemoteObservationKind::OutputsAdmitted);
    debug_assert!(observation.output_trusted);
    Ok(observation)
}

fn path_info_identity(path_info: &snix_store::path_info::PathInfo) -> Result<String, String> {
    let bytes = serde_json::to_vec(path_info).map_err(|error| format!("serializing PathInfo adapter fact: {error}"))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    debug_assert_eq!(digest.len(), crunch_remote::BLAKE3_HEX_LENGTH);
    debug_assert!(crunch_remote::is_blake3_hex_digest(&digest));
    Ok(digest)
}

fn project_substitution(report: &crunch_store::OutputSubstitutionReport) -> RemoteSubstitutionFact {
    RemoteSubstitutionFact {
        mode: match report.mode {
            crunch_store::OutputSubstitutionMode::Delta => RemoteSubstitutionMode::Delta,
            crunch_store::OutputSubstitutionMode::Full => RemoteSubstitutionMode::Full,
        },
        transferred_bytes: report.transferred_bytes,
        fallback_reason: report.fallback_reason.clone(),
    }
}

const fn project_failure_phase(phase: RemoteFailurePhase) -> RemoteWireFailurePhase {
    match phase {
        RemoteFailurePhase::TransportSetup => RemoteWireFailurePhase::TransportSetup,
        RemoteFailurePhase::Authentication => RemoteWireFailurePhase::Authentication,
        RemoteFailurePhase::RequestValidation => RemoteWireFailurePhase::RequestValidation,
        RemoteFailurePhase::InputSync => RemoteWireFailurePhase::InputSync,
        RemoteFailurePhase::Queue => RemoteWireFailurePhase::Queue,
        RemoteFailurePhase::BuildExecution => RemoteWireFailurePhase::BuildExecution,
        RemoteFailurePhase::OutputImport => RemoteWireFailurePhase::OutputImport,
    }
}

const fn project_retry_class(retry: RemoteRetryClass) -> RemoteWireRetryClass {
    match retry {
        RemoteRetryClass::Retryable => RemoteWireRetryClass::Retryable,
        RemoteRetryClass::Terminal => RemoteWireRetryClass::Terminal,
        RemoteRetryClass::BuildOutcome => RemoteWireRetryClass::BuildOutcome,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote_build::REMOTE_PROTOCOL_ALPN;
    use crate::remote_build::REMOTE_PROTOCOL_VERSION;
    use crate::remote_build::RemoteHello;
    use crate::remote_build::RemoteProtocolErrorFrame;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OUTPUT_BYTES: u64 = 8;

    #[test]
    fn wire_projection_preserves_accepted_json_bytes() {
        let frames = [
            RemoteFrame::Hello {
                hello: RemoteHello {
                    alpn: REMOTE_PROTOCOL_ALPN.to_string(),
                    version: REMOTE_PROTOCOL_VERSION,
                    endpoint_id: "worker-a".to_string(),
                    capabilities: vec!["build".to_string()],
                    workspace_policy: None,
                },
            },
            RemoteFrame::BuildQueued {
                request_id: "request-a".to_string(),
                session_id: "session-a".to_string(),
            },
            RemoteFrame::Error {
                error: RemoteProtocolErrorFrame {
                    phase: RemoteFailurePhase::BuildExecution,
                    retry_class: RemoteRetryClass::Retryable,
                    message: "worker-lost".to_string(),
                },
            },
        ];
        for frame in frames {
            let legacy = serde_json::to_vec(&frame).expect("legacy frame serializes");
            let projected = project_wire_frame(&frame).expect("supported compatibility frame");
            let extracted = crunch_remote::encode_remote_wire_frame(projected).expect("core frame serializes");
            assert_eq!(legacy, extracted);
        }
    }

    #[test]
    fn output_projection_binds_vendor_records_without_exposing_them() {
        let output = RemoteExecutionOutput {
            name: "out".to_string(),
            logical_path: "/mantle/store/example".to_string(),
            content_digest_blake3: DIGEST.to_string(),
            size_bytes: OUTPUT_BYTES,
            artifact_attestation_digest_blake3: DIGEST.to_string(),
            path_info: Some((*snix_store::fixtures::PATH_INFO_SYMLINK).clone()),
            nar_payload: None,
        };
        let substitution = crunch_store::OutputSubstitutionReport {
            mode: crunch_store::OutputSubstitutionMode::Full,
            transferred_bytes: OUTPUT_BYTES,
            reused_bytes: 0,
            metadata_reused: false,
            fallback_reason: None,
        };
        let fact = output_fact_from_execution(&output, Some(&substitution)).expect("output projects");
        assert_eq!(fact.path_info_blake3.as_ref().map(String::len), Some(crunch_remote::BLAKE3_HEX_LENGTH));
        assert_eq!(fact.substitution.as_ref().map(|value| value.mode), Some(RemoteSubstitutionMode::Full));
    }

    #[test]
    fn output_projection_removes_store_and_snix_types() {
        let output = RemoteExecutionOutput {
            name: "out".to_string(),
            logical_path: "/mantle/store/example".to_string(),
            content_digest_blake3: DIGEST.to_string(),
            size_bytes: OUTPUT_BYTES,
            artifact_attestation_digest_blake3: DIGEST.to_string(),
            path_info: None,
            nar_payload: None,
        };
        let fact = output_fact_from_execution(&output, None).expect("output projects");
        assert_eq!(fact.name, output.name);
        assert!(fact.path_info_blake3.is_none());
        assert!(fact.substitution.is_none());
    }
}
