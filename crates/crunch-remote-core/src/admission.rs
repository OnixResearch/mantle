use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::AcceptedHelloFacts;
use crate::BLAKE3_HEX_LENGTH;
use crate::HelloAdmissionDecision;
use crate::HelloAdmissionInput;
use crate::MAX_REMOTE_ATTEMPTS;
use crate::MAX_REMOTE_BUILD_TIME_MS;
use crate::MAX_REMOTE_CAPABILITIES;
use crate::MAX_REMOTE_CPU_UNITS;
use crate::MAX_REMOTE_EXPECTED_OUTPUTS;
use crate::MAX_REMOTE_INPUT_REFS;
use crate::MAX_REMOTE_MEMORY_BYTES;
use crate::MAX_REMOTE_UPLOAD_BYTES;
use crate::MissingInputFacts;
use crate::OutputTrustInput;
use crate::REMOTE_COMMAND_SCHEMA;
use crate::REMOTE_PROTOCOL_ALPN;
use crate::REMOTE_PROTOCOL_VERSION;
use crate::RemoteBuildFallbackPolicy;
use crate::RemoteBuildServiceRequestFacts;
use crate::RemoteCommand;
use crate::RemoteCoreError;
use crate::RemoteFailurePlan;
use crate::RemoteOutputTrustDecision;
use crate::VerifiedRemoteArtifact;
use crate::is_blake3_hex_digest;

pub fn admit_remote_command(mut command: RemoteCommand) -> Result<RemoteCommand, RemoteCoreError> {
    command.input_refs.sort();
    command.expected_outputs.sort();
    validate_remote_command(&command)?;
    debug_assert!(command.input_refs.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(command.expected_outputs.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(command)
}

fn validate_remote_command(command: &RemoteCommand) -> Result<(), RemoteCoreError> {
    if command.schema != REMOTE_COMMAND_SCHEMA {
        return Err(RemoteCoreError::invalid("remote-command-schema-unsupported"));
    }
    validate_identity(IdentityValidation {
        code: "remote-job-id-empty",
        value: &command.job_id,
    })?;
    validate_identity(IdentityValidation {
        code: "remote-worker-id-empty",
        value: &command.worker_id,
    })?;
    if !is_blake3_hex_digest(&command.request_blake3) {
        return Err(RemoteCoreError::invalid("remote-request-digest-invalid"));
    }
    if command.fence_generation == 0 {
        return Err(RemoteCoreError::invalid("remote-fence-generation-zero"));
    }
    validate_inputs(&command.input_refs)?;
    validate_outputs(&command.expected_outputs)?;
    validate_policy(&command.policy)?;
    debug_assert_eq!(command.request_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(command.fence_generation > 0);
    Ok(())
}

struct IdentityValidation<'a> {
    code: &'static str,
    value: &'a str,
}

fn validate_identity(input: IdentityValidation<'_>) -> Result<(), RemoteCoreError> {
    if input.value.is_empty() {
        return Err(RemoteCoreError::invalid(input.code));
    }
    Ok(())
}

fn validate_inputs(input_refs: &[String]) -> Result<(), RemoteCoreError> {
    if input_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(RemoteCoreError::invalid("remote-input-ref-count-exceeded"));
    }
    validate_sorted_unique(input_refs, UniqueValidationCodes {
        empty: "remote-input-ref-empty",
        duplicate: "remote-input-ref-duplicate",
    })
}

fn validate_outputs(outputs: &[crate::RemoteOutputExpectation]) -> Result<(), RemoteCoreError> {
    if outputs.is_empty() {
        return Err(RemoteCoreError::invalid("remote-expected-output-missing"));
    }
    if outputs.len() > MAX_REMOTE_EXPECTED_OUTPUTS {
        return Err(RemoteCoreError::invalid("remote-expected-output-count-exceeded"));
    }
    let names = outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>();
    validate_sorted_unique(&names, UniqueValidationCodes {
        empty: "remote-expected-output-name-empty",
        duplicate: "remote-expected-output-duplicate",
    })
}

struct UniqueValidationCodes {
    empty: &'static str,
    duplicate: &'static str,
}

fn validate_sorted_unique<T: AsRef<str>>(values: &[T], codes: UniqueValidationCodes) -> Result<(), RemoteCoreError> {
    let mut previous: Option<&str> = None;
    for value in values {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(RemoteCoreError::invalid(codes.empty));
        }
        if previous == Some(value) {
            return Err(RemoteCoreError::invalid(codes.duplicate));
        }
        previous = Some(value);
    }
    Ok(())
}

fn validate_policy(policy: &crate::RemotePolicy) -> Result<(), RemoteCoreError> {
    if policy.attempts_max == 0 || policy.attempts_max > MAX_REMOTE_ATTEMPTS {
        return Err(RemoteCoreError::invalid("remote-attempt-limit-invalid"));
    }
    if policy.input_bytes_max > MAX_REMOTE_UPLOAD_BYTES || policy.output_bytes_max > MAX_REMOTE_UPLOAD_BYTES {
        return Err(RemoteCoreError::invalid("remote-byte-limit-invalid"));
    }
    if policy.build_time_ms_max == 0 || policy.build_time_ms_max > MAX_REMOTE_BUILD_TIME_MS {
        return Err(RemoteCoreError::invalid("remote-build-time-limit-invalid"));
    }
    if policy.cpu_units_max == 0 || policy.cpu_units_max > MAX_REMOTE_CPU_UNITS {
        return Err(RemoteCoreError::invalid("remote-cpu-limit-invalid"));
    }
    if policy.memory_bytes_max == 0 || policy.memory_bytes_max > MAX_REMOTE_MEMORY_BYTES {
        return Err(RemoteCoreError::invalid("remote-memory-limit-invalid"));
    }
    debug_assert!(policy.attempts_max <= MAX_REMOTE_ATTEMPTS);
    debug_assert!(policy.build_time_ms_max <= MAX_REMOTE_BUILD_TIME_MS);
    Ok(())
}

pub fn admit_hello(input: HelloAdmissionInput) -> HelloAdmissionDecision {
    if input.alpn != REMOTE_PROTOCOL_ALPN {
        return HelloAdmissionDecision::Reject(alloc::format!("unsupported ALPN {}", input.alpn));
    }
    if input.version != REMOTE_PROTOCOL_VERSION {
        return HelloAdmissionDecision::Reject(alloc::format!("unsupported protocol version {}", input.version));
    }
    if input.endpoint_id != input.expected_endpoint_id {
        return HelloAdmissionDecision::Reject("endpoint identity mismatch".to_string());
    }
    if input.capabilities.len() > MAX_REMOTE_CAPABILITIES {
        return HelloAdmissionDecision::Reject(alloc::format!("capability count exceeds {MAX_REMOTE_CAPABILITIES}"));
    }
    if !input.workspace_policy_valid {
        return HelloAdmissionDecision::Reject("invalid workspace registration".to_string());
    }
    let accepted_capabilities = input
        .capabilities
        .iter()
        .filter(|capability| input.supported_capabilities.contains(capability))
        .cloned()
        .collect::<Vec<_>>();
    debug_assert!(accepted_capabilities.len() <= input.capabilities.len());
    debug_assert!(accepted_capabilities.iter().all(|value| input.supported_capabilities.contains(value)));
    HelloAdmissionDecision::Proceed(AcceptedHelloFacts {
        endpoint_id: input.endpoint_id,
        accepted_capabilities,
    })
}

pub fn derive_missing_inputs(input: MissingInputFacts) -> Result<Vec<String>, RemoteCoreError> {
    if input.declared_refs.len() > MAX_REMOTE_INPUT_REFS || input.present_refs.len() > MAX_REMOTE_INPUT_REFS {
        return Err(RemoteCoreError::invalid(alloc::format!("input-ref-count-exceeds-{MAX_REMOTE_INPUT_REFS}")));
    }
    let present = input.present_refs.iter().collect::<BTreeSet<_>>();
    let missing = input.declared_refs.into_iter().filter(|reference| !present.contains(reference)).collect::<Vec<_>>();
    debug_assert!(missing.len() <= MAX_REMOTE_INPUT_REFS);
    debug_assert!(missing.iter().all(|reference| !input.present_refs.contains(reference)));
    Ok(missing)
}

pub fn decide_output_trust(input: OutputTrustInput) -> RemoteOutputTrustDecision {
    if !input.store_prefix_matches {
        return RemoteOutputTrustDecision::Reject("store-prefix-mismatch".to_string());
    }
    let signer = parse_output_key_ref(&input.signing_key_id);
    debug_assert_eq!(signer.name.is_empty(), input.signing_key_id.is_empty());
    debug_assert!(signer.material_blake3.as_ref().is_none_or(|digest| is_blake3_hex_digest(digest)));
    let mut is_same_name_material_required = false;
    let mut is_same_name_different_material = false;
    for trusted_key_id in &input.trusted_key_ids {
        let trusted = parse_output_key_ref(trusted_key_id);
        if trusted.name != signer.name {
            continue;
        }
        match (&signer.material_blake3, &trusted.material_blake3) {
            (Some(signer_digest), Some(trusted_digest)) if signer_digest == trusted_digest => {
                return accepted_output_key(input.signing_key_id, signer.material_blake3);
            }
            (Some(_), Some(_)) => is_same_name_different_material = true,
            (None, Some(_)) => is_same_name_material_required = true,
            (_, None) => return accepted_output_key(input.signing_key_id, signer.material_blake3),
        }
    }
    if is_same_name_material_required {
        return RemoteOutputTrustDecision::Reject("output-key-material-missing".to_string());
    }
    if is_same_name_different_material {
        return RemoteOutputTrustDecision::Reject("same-name-different-output-key".to_string());
    }
    RemoteOutputTrustDecision::Reject("untrusted-output-key".to_string())
}

fn accepted_output_key(key_id: String, material_blake3: Option<String>) -> RemoteOutputTrustDecision {
    RemoteOutputTrustDecision::Accept {
        key_id,
        key_material_digest_blake3: material_blake3,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OutputKeyRef {
    name: String,
    material_blake3: Option<String>,
}

fn parse_output_key_ref(value: &str) -> OutputKeyRef {
    if let Some((name, material)) = value.split_once(':')
        && !name.is_empty()
        && !material.is_empty()
    {
        return OutputKeyRef {
            name: name.to_string(),
            material_blake3: Some(blake3::hash(material.as_bytes()).to_hex().to_string()),
        };
    }
    OutputKeyRef {
        name: value.to_string(),
        material_blake3: None,
    }
}

pub fn classify_remote_failure(policy: RemoteBuildFallbackPolicy, phase: String, reason: String) -> RemoteFailurePlan {
    match policy {
        RemoteBuildFallbackPolicy::Never => RemoteFailurePlan::ReturnFailure { phase, reason },
        RemoteBuildFallbackPolicy::OnRemoteFailure => RemoteFailurePlan::FallbackToLocal { phase, reason },
    }
}

pub fn validate_build_service_request(input: RemoteBuildServiceRequestFacts) -> Result<(), RemoteCoreError> {
    if input.command_arg_count == 0 {
        return Err(RemoteCoreError::invalid("remote-build-service-raw-eval-request"));
    }
    if input.output_count == 0 {
        return Err(RemoteCoreError::invalid("remote-build-service-outputs-empty"));
    }
    debug_assert!(input.command_arg_count > 0);
    debug_assert!(input.output_count > 0);
    Ok(())
}

pub fn validate_verified_artifact(artifact: VerifiedRemoteArtifact) -> Result<VerifiedRemoteArtifact, RemoteCoreError> {
    if artifact.outputs.is_empty() {
        return Err(RemoteCoreError::invalid("verified-artifact-empty"));
    }
    if artifact.outputs.len() > MAX_REMOTE_EXPECTED_OUTPUTS {
        return Err(RemoteCoreError::invalid("verified-artifact-output-count-exceeded"));
    }
    for (name, output) in &artifact.outputs {
        if name.is_empty() || output.name != *name {
            return Err(RemoteCoreError::invalid("verified-artifact-output-name-mismatch"));
        }
        if !is_blake3_hex_digest(&output.content_digest_blake3) {
            return Err(RemoteCoreError::invalid("verified-artifact-content-digest-invalid"));
        }
    }
    debug_assert!(!artifact.outputs.is_empty());
    debug_assert!(artifact.outputs.len() <= MAX_REMOTE_EXPECTED_OUTPUTS);
    Ok(artifact)
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;
    use crate::RemoteOutputFact;
    use crate::RemoteSubstitutionFact;
    use crate::RemoteSubstitutionMode;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn missing_inputs_are_stable_and_bounded() {
        let missing = derive_missing_inputs(MissingInputFacts {
            declared_refs: vec!["a".to_string(), "b".to_string()],
            present_refs: vec!["a".to_string()],
        })
        .expect("bounded input set");
        assert_eq!(missing, vec!["b".to_string()]);
        assert!(
            derive_missing_inputs(MissingInputFacts {
                declared_refs: vec![String::new(); MAX_REMOTE_INPUT_REFS + 1],
                present_refs: Vec::new(),
            })
            .is_err()
        );
    }

    #[test]
    fn output_trust_counts_full_key_material() {
        let accepted = decide_output_trust(OutputTrustInput {
            signing_key_id: "builder:key-a".to_string(),
            trusted_key_ids: vec!["builder:key-a".to_string()],
            store_prefix_matches: true,
        });
        let rejected = decide_output_trust(OutputTrustInput {
            signing_key_id: "builder:key-b".to_string(),
            trusted_key_ids: vec!["builder:key-a".to_string()],
            store_prefix_matches: true,
        });
        assert!(matches!(accepted, RemoteOutputTrustDecision::Accept { .. }));
        assert_eq!(rejected, RemoteOutputTrustDecision::Reject("same-name-different-output-key".to_string()));
    }

    #[test]
    fn verified_artifact_rejects_name_and_digest_drift() {
        let output = RemoteOutputFact {
            name: "out".to_string(),
            logical_path: "/mantle/store/example".to_string(),
            content_digest_blake3: DIGEST.to_string(),
            size_bytes: 1,
            artifact_attestation_blake3: DIGEST.to_string(),
            path_info_blake3: Some(DIGEST.to_string()),
            substitution: Some(RemoteSubstitutionFact {
                mode: RemoteSubstitutionMode::Full,
                transferred_bytes: 1,
                fallback_reason: None,
            }),
        };
        let accepted = validate_verified_artifact(VerifiedRemoteArtifact {
            outputs: BTreeMap::from([("out".to_string(), output.clone())]),
        });
        let rejected = validate_verified_artifact(VerifiedRemoteArtifact {
            outputs: BTreeMap::from([("other".to_string(), output)]),
        });
        assert!(accepted.is_ok());
        assert!(rejected.is_err());
    }
}
