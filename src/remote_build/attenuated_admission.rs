//! Receiver-side admission of an optional holder-attenuated UCAN presentation.
//!
//! The production receiver calls this inside its ticket-state mutation guard,
//! after the bearer and concrete request validated and BEFORE any redemption,
//! runtime construction, input transfer or success frame. Every fact comes from
//! the receiver: its clock, its operator-loaded authority snapshot (policy,
//! trusted roots, revocations, holder replay ledger), its endpoint identity and
//! the validated request. The sender supplies only the signed credential. A
//! denial carries one constant string: no job, path, caveat or reason.
use std::path::Path;

use basalt::ucan::HolderInvocationNonce;
use basalt::ucan::SignerSignature;
use serde::Deserialize;
use serde::Serialize;
use zeroize::Zeroize as _;

use super::ConcreteBuildRequest;
use super::RemoteConcreteBuildPayload;
use super::authority_caveats::EffectiveCaveatChain;
use crate::errors::RunError;
use crate::nix_gateway::authority::Error;
use crate::nix_gateway::authority::GatewayAuthorityConfig;
use crate::nix_gateway::authority::GatewayVerifier;
use crate::nix_gateway::authority::Invocation;
use crate::nix_gateway::authority::Operation;
use crate::nix_gateway::authority::OperationRequest;

/// The only text a sender may observe for any denied attenuated presentation.
pub const ATTENUATED_GRANT_DENIED: &str = "remote-attenuated-grant-denied";

const MAX_COMPACT_TOKEN_BYTES: usize = 32 * 1024;
const MAX_HOLDER_SIGNATURE_BYTES: usize = 128;
const MAX_IDENTITY_BYTES: usize = 512;
/// Operator-private authority snapshot and holder replay ledger, both beside the
/// ticket state. They are optional: absent files mean no attenuated grant is
/// honored, never that a presented grant is trusted.
const OPERATOR_AUTHORITY_FILE: &str = "attenuation-authority.json";
const OPERATOR_REPLAY_DIRECTORY: &str = "attenuation-replay";
const REQUEST_DIGEST_DOMAIN: &[u8] = b"mantle/remote/attenuated-build-request/v1\0";

/// Holder-signed credential plus the claims the signed resource must match.
/// Every claim is re-checked against the signed capability by the verifier, so
/// none of them is trusted merely because the sender stated it.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttenuatedGrantPresentation {
    pub compact_token: String,
    pub nonce: [u8; 32],
    pub holder_signature: Vec<u8>,
    pub subject: String,
    pub account: String,
    pub project: String,
    pub audience: String,
    pub generation: u64,
}

impl Drop for AttenuatedGrantPresentation {
    fn drop(&mut self) {
        self.compact_token.zeroize();
        self.nonce.zeroize();
        self.holder_signature.zeroize();
    }
}

impl std::fmt::Debug for AttenuatedGrantPresentation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("AttenuatedGrantPresentation").finish_non_exhaustive()
    }
}

/// Receiver-owned facts for one admission. Nothing here is sender-supplied.
pub struct AttenuationAdmission<'a> {
    /// Absent means this receiver has no operator-loaded authority: a presented
    /// grant then fails closed instead of degrading to bearer-only admission.
    pub verifier: Option<&'a GatewayVerifier>,
    pub endpoint_id: &'a str,
    pub project_scope: &'a str,
}

/// The exact normalized effect a holder signs: the declared job and output class
/// as a logical value for the caveat filters, and a digest over the whole
/// validated request so no other field can change after signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteJobBinding {
    pub target: String,
    pub request_digest: [u8; 32],
}

fn output_class(request: &ConcreteBuildRequest) -> &'static str {
    match request.payload {
        RemoteConcreteBuildPayload::Derivation { .. } => "derivation",
        RemoteConcreteBuildPayload::Action { .. } => "action",
    }
}

fn frame(hasher: &mut blake3::Hasher, field: &[u8]) {
    debug_assert!(field.len() <= usize::try_from(u32::MAX).unwrap_or(usize::MAX));
    debug_assert!(REQUEST_DIGEST_DOMAIN.ends_with(b"\0"));
    hasher.update(&u64::try_from(field.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(field);
}

fn frame_count(hasher: &mut blake3::Hasher, count: usize) {
    debug_assert!(count <= usize::try_from(u32::MAX).unwrap_or(usize::MAX));
    debug_assert!(!REQUEST_DIGEST_DOMAIN.is_empty());
    hasher.update(&u64::try_from(count).unwrap_or(u64::MAX).to_be_bytes());
}

fn payload_identity(request: &ConcreteBuildRequest) -> (&str, &str) {
    match &request.payload {
        RemoteConcreteBuildPayload::Derivation { drv_path, drv_json } => (drv_path.as_str(), drv_json.as_str()),
        RemoteConcreteBuildPayload::Action { action_id, spec_json } => (action_id.as_str(), spec_json.as_str()),
    }
}

/// Bind the holder signature to the validated request and this endpoint.
/// The job identity is the concrete `request_id`; a coordinator attempt's
/// `job_id` is bound into the digest so the holder commits to it as well.
// r[impl mantle.authority_attenuation.attenuated_remote_grant]
pub fn bind_remote_job(request: &ConcreteBuildRequest, endpoint_id: &str) -> Result<RemoteJobBinding, Error> {
    if request.request_id.is_empty() || endpoint_id.is_empty() {
        return Err(Error);
    }
    let target = format!("/job/{}/output/{}", request.request_id, output_class(request));
    if target.len() > crunch_remote_core::caveat::MAX_VALUE_BYTES {
        return Err(Error);
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(REQUEST_DIGEST_DOMAIN);
    let job_id = request.production_attempt.as_ref().map_or("", |attempt| attempt.job_id.as_str());
    let (payload_id, payload_body) = payload_identity(request);
    for field in [
        endpoint_id,
        request.store_prefix.as_str(),
        request.request_id.as_str(),
        job_id,
        target.as_str(),
        payload_id,
    ] {
        frame(&mut hasher, field.as_bytes());
    }
    frame(&mut hasher, blake3::hash(payload_body.as_bytes()).as_bytes());
    for refs in [&request.input_refs, &request.source_input_refs] {
        frame_count(&mut hasher, refs.len());
        for input in refs {
            frame(&mut hasher, input.as_bytes());
        }
    }
    hasher.update(&request.upload_bytes.to_be_bytes());
    hasher.update(&request.build_time_limit_secs.to_be_bytes());
    frame_count(&mut hasher, request.expected_outputs.len());
    for output in &request.expected_outputs {
        frame(&mut hasher, output.name.as_bytes());
        frame(&mut hasher, output.logical_path.as_deref().unwrap_or("").as_bytes());
    }
    debug_assert!(target.starts_with("/job/"));
    debug_assert!(!request.request_id.is_empty());
    Ok(RemoteJobBinding {
        target,
        request_digest: *hasher.finalize().as_bytes(),
    })
}

fn presentation_is_bounded(presentation: &AttenuatedGrantPresentation) -> bool {
    let identities = [
        &presentation.subject,
        &presentation.account,
        &presentation.project,
        &presentation.audience,
    ];
    !presentation.compact_token.is_empty()
        && presentation.compact_token.len() <= MAX_COMPACT_TOKEN_BYTES
        && !presentation.holder_signature.is_empty()
        && presentation.holder_signature.len() <= MAX_HOLDER_SIGNATURE_BYTES
        && identities.iter().all(|value| !value.is_empty() && value.len() <= MAX_IDENTITY_BYTES)
}

/// Verify the presented grant against this exact request. `Ok` means the signed
/// proof chain, every preserved parent caveat, the receiver clock and deadline,
/// current revocation, holder replay and the original-plus-ancestor filter
/// checks all passed for exactly this job and output class. It redeems nothing.
// r[impl mantle.authority_attenuation.attenuated_remote_grant]
pub fn admit_presented_grant(
    admission: &AttenuationAdmission<'_>,
    presentation: &AttenuatedGrantPresentation,
    request: &ConcreteBuildRequest,
    server_now_unix_s: u64,
) -> Result<EffectiveCaveatChain, Error> {
    let verifier = admission.verifier.ok_or(Error)?;
    if !presentation_is_bounded(presentation) || presentation.project != admission.project_scope {
        return Err(Error);
    }
    let binding = bind_remote_job(request, admission.endpoint_id)?;
    let signature = SignerSignature::try_from_slice(&presentation.holder_signature).map_err(|_| Error)?;
    let operation_request = OperationRequest {
        operation: Operation::BuildSubmit,
        subject: presentation.subject.clone(),
        account: presentation.account.clone(),
        project: presentation.project.clone(),
        endpoint: admission.endpoint_id.to_string(),
        audience: presentation.audience.clone(),
        generation: presentation.generation,
        request_digest: binding.request_digest,
        target: binding.target.clone(),
    };
    let session = verifier.verify(
        operation_request,
        Invocation {
            compact_token: &presentation.compact_token,
            nonce: HolderInvocationNonce::from_bytes(presentation.nonce),
            holder_signature: &signature,
        },
        server_now_unix_s,
    )?;
    session.require(Operation::BuildSubmit, &binding.target, binding.request_digest)?;
    let chain = verifier.effective_caveat_chain(&session, server_now_unix_s)?;
    debug_assert_eq!(session.endpoint(), admission.endpoint_id);
    debug_assert_eq!(session.target(), binding.target);
    Ok(chain)
}

/// Load the operator-owned authority beside the ticket state, if provisioned.
/// A provisioned but unreadable, non-private or malformed snapshot fails the
/// server start closed instead of silently disabling presented-grant checks.
pub fn open_operator_authority(credential_state_dir: &Path) -> Result<Option<GatewayVerifier>, RunError> {
    let authority_file = credential_state_dir.join(OPERATOR_AUTHORITY_FILE);
    match std::fs::symlink_metadata(&authority_file) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(RunError::Internal("remote-attenuation-authority-invalid".into())),
        Ok(_) => {}
    }
    let config = GatewayAuthorityConfig {
        authority_file,
        replay_directory: credential_state_dir.join(OPERATOR_REPLAY_DIRECTORY),
    };
    let verifier =
        GatewayVerifier::open(config).map_err(|_| RunError::Internal("remote-attenuation-authority-invalid".into()))?;
    debug_assert!(!OPERATOR_AUTHORITY_FILE.is_empty());
    debug_assert!(!OPERATOR_REPLAY_DIRECTORY.is_empty());
    Ok(Some(verifier))
}
