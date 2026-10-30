//! Receiver-side admission of an optional holder-attenuated UCAN presentation.
//!
//! ADVISORY, NOT ENFORCING: the presentation is optional and the ticket bearer
//! alone still admits. A holder who also holds the bearer can omit the
//! presentation and be admitted unrestricted. See docs/remote-credentials.md.
//!
//! The production receiver calls this inside its ticket-state mutation guard,
//! after the bearer and concrete request validated and BEFORE any redemption,
//! runtime construction, input transfer or success frame. Every fact comes from
//! the receiver: its clock, its operator-loaded authority snapshot (policy,
//! trusted roots, revocations, holder replay ledger), its endpoint identity and
//! the validated request. The sender supplies only the signed credential. A
//! denial carries one constant string to the sender: no job, path, caveat or
//! reason. The receiver logs a reason code locally (`DenialReason`).
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
const REQUEST_DIGEST_DOMAIN: &[u8] = b"mantle/remote/attenuated-build-request/v2\0";

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

/// The exact normalized effect a holder signs: the declared job identity and
/// output class as a logical value for the caveat filters, and a digest over the
/// whole validated request (see `bind_remote_job`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteJobBinding {
    pub target: String,
    pub request_digest: [u8; 32],
}

/// Most output names a filter-visible target can carry: the target has seven
/// fixed segments (with an attempt) and the filter language allows 16 segments.
const MAX_BOUND_OUTPUT_NAMES: usize = crunch_remote_core::caveat::MAX_SEGMENTS - 7;

fn output_class(request: &ConcreteBuildRequest) -> &'static str {
    match request.payload {
        RemoteConcreteBuildPayload::Derivation { .. } => "derivation",
        RemoteConcreteBuildPayload::Action { .. } => "action",
    }
}

/// One canonical filter segment. This mirrors the filter language's literal
/// vocabulary (store-name-safe ASCII, no `.`/`..`, no leading `:`); the tests
/// assert agreement with the core parser. A value outside it is rejected, never
/// escaped or truncated, so a job whose identity cannot be expressed to the
/// filters cannot be granted by pattern.
fn is_filter_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"+._?=-".contains(&byte))
}

/// Filter-visible target:
/// `/job/<request_id>[/attempt/<production job_id>]/class/<payload kind>/outputs/<sorted declared
/// names>`. The attempt segments exist exactly when a coordinator attempt exists, so a
/// grant for one attempt job id never matches another or none.
fn job_target(request: &ConcreteBuildRequest) -> Result<String, Error> {
    let mut segments: Vec<&str> = vec!["job", request.request_id.as_str()];
    if let Some(attempt) = request.production_attempt.as_ref() {
        segments.push("attempt");
        segments.push(attempt.job_id.as_str());
    }
    segments.push("class");
    segments.push(output_class(request));
    segments.push("outputs");
    let mut names: Vec<&str> = request.expected_outputs.iter().map(|output| output.name.as_str()).collect();
    names.sort_unstable();
    if names.is_empty() || names.len() > MAX_BOUND_OUTPUT_NAMES || names.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error);
    }
    segments.extend(names);
    if !segments.iter().all(|segment| is_filter_segment(segment)) {
        return Err(Error);
    }
    let target = format!("/{}", segments.join("/"));
    if target.len() > crunch_remote_core::caveat::MAX_VALUE_BYTES {
        return Err(Error);
    }
    debug_assert!(target.starts_with("/job/"));
    debug_assert!(segments.len() <= crunch_remote_core::caveat::MAX_SEGMENTS);
    Ok(target)
}

/// Bind the holder signature to the whole validated request and this endpoint.
/// The digest is BLAKE3 over a domain, the endpoint id and the deterministic
/// `serde_json` encoding of the entire `ConcreteBuildRequest` (struct fields in
/// declaration order; every nested type is a struct, `Vec`, `Option`, scalar or
/// string, with no hash maps or floats), so a field added later is covered
/// automatically and any change after signing is refused. The caveat-visible
/// target carries the request id, production attempt job id, payload kind and
/// declared output names (`job_target`).
// r[impl mantle.authority_attenuation.attenuated_remote_grant]
pub fn bind_remote_job(request: &ConcreteBuildRequest, endpoint_id: &str) -> Result<RemoteJobBinding, Error> {
    if endpoint_id.is_empty() {
        return Err(Error);
    }
    let target = job_target(request)?;
    let encoded = serde_json::to_vec(request).map_err(|_| Error)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(REQUEST_DIGEST_DOMAIN);
    frame(&mut hasher, endpoint_id.as_bytes());
    frame(&mut hasher, &encoded);
    debug_assert!(!encoded.is_empty());
    debug_assert!(!target.is_empty());
    Ok(RemoteJobBinding {
        target,
        request_digest: *hasher.finalize().as_bytes(),
    })
}

fn frame(hasher: &mut blake3::Hasher, field: &[u8]) {
    debug_assert!(field.len() <= usize::try_from(u32::MAX).unwrap_or(usize::MAX));
    debug_assert!(REQUEST_DIGEST_DOMAIN.ends_with(b"\0"));
    hasher.update(&u64::try_from(field.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(field);
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

/// Why a presented grant was refused. LOCAL diagnostics only: the sender sees
/// `ATTENUATED_GRANT_DENIED` for every variant. The UCAN/Basalt verifier reports
/// one undifferentiated denial, so `Verification` covers signature, proof chain,
/// preserved parent caveats, validity window, revocation, replay (including a
/// full ledger), caveat filters and wrong endpoint/audience/generation/holder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialReason {
    NoOperatorAuthority,
    PresentationMalformed,
    ProjectScopeMismatch,
    RequestUnbindable,
    Verification,
    SessionBinding,
    ChainReport,
    UnenforcedPath,
}

impl DenialReason {
    /// Stable operator-facing code; never sent to the sender.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NoOperatorAuthority => "no-operator-authority",
            Self::PresentationMalformed => "presentation-malformed",
            Self::ProjectScopeMismatch => "project-scope-mismatch",
            Self::RequestUnbindable => "request-unbindable",
            Self::Verification => "verification-denied",
            Self::SessionBinding => "session-binding-mismatch",
            Self::ChainReport => "chain-report-denied",
            Self::UnenforcedPath => "non-enforcing-admission-path",
        }
    }
}

/// Verify the presented grant against this exact request. `Ok` means the signed
/// proof chain, every preserved parent caveat, the receiver clock and deadline,
/// current revocation, holder replay and the original-plus-ancestor filter
/// checks all passed for exactly this request's job identity and outputs. It
/// redeems nothing, but a successful verification has consumed the holder nonce.
// r[impl mantle.authority_attenuation.attenuated_remote_grant]
pub fn admit_presented_grant(
    admission: &AttenuationAdmission<'_>,
    presentation: &AttenuatedGrantPresentation,
    request: &ConcreteBuildRequest,
    server_now_unix_s: u64,
) -> Result<EffectiveCaveatChain, DenialReason> {
    let verifier = admission.verifier.ok_or(DenialReason::NoOperatorAuthority)?;
    if !presentation_is_bounded(presentation) {
        return Err(DenialReason::PresentationMalformed);
    }
    if presentation.project != admission.project_scope {
        return Err(DenialReason::ProjectScopeMismatch);
    }
    let binding = bind_remote_job(request, admission.endpoint_id).map_err(|_| DenialReason::RequestUnbindable)?;
    let signature = SignerSignature::try_from_slice(&presentation.holder_signature)
        .map_err(|_| DenialReason::PresentationMalformed)?;
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
    let session = verifier
        .verify(
            operation_request,
            Invocation {
                compact_token: &presentation.compact_token,
                nonce: HolderInvocationNonce::from_bytes(presentation.nonce),
                holder_signature: &signature,
            },
            server_now_unix_s,
        )
        .map_err(|_| DenialReason::Verification)?;
    session
        .require(Operation::BuildSubmit, &binding.target, binding.request_digest)
        .map_err(|_| DenialReason::SessionBinding)?;
    let chain = verifier
        .effective_caveat_chain(&session, server_now_unix_s)
        .map_err(|_| DenialReason::ChainReport)?;
    debug_assert_eq!(session.endpoint(), admission.endpoint_id);
    debug_assert_eq!(session.target(), binding.target);
    Ok(chain)
}

/// Load the operator-owned authority beside the ticket state, if provisioned.
/// A provisioned snapshot must open (owner-private regular file, no symlink,
/// bounded size, private replay directory) AND load now (valid JSON, version,
/// validity window, policy hash, keys, roots, proofs); otherwise the server
/// start fails closed instead of silently denying every presented grant later.
pub fn open_operator_authority(credential_state_dir: &Path) -> Result<Option<GatewayVerifier>, RunError> {
    let invalid = || RunError::Internal("remote-attenuation-authority-invalid".into());
    let authority_file = credential_state_dir.join(OPERATOR_AUTHORITY_FILE);
    match std::fs::symlink_metadata(&authority_file) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(invalid()),
        Ok(_) => {}
    }
    let config = GatewayAuthorityConfig {
        authority_file,
        replay_directory: credential_state_dir.join(OPERATOR_REPLAY_DIRECTORY),
    };
    let verifier = GatewayVerifier::open(config).map_err(|_| invalid())?;
    let now = crate::unix_time_now_s().map_err(|_| invalid())?;
    verifier.check_loadable(now).map_err(|_| invalid())?;
    debug_assert!(!OPERATOR_AUTHORITY_FILE.is_empty());
    debug_assert!(!OPERATOR_REPLAY_DIRECTORY.is_empty());
    Ok(Some(verifier))
}
