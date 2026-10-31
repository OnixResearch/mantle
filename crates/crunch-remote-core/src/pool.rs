//! Explicit native worker-pool admission.
//!
//! This module decides only whether an operator-selected pool declaration is
//! coherent enough to dispatch through: every endpoint must present a current
//! admitted binding, a supported native transport, declared execution
//! capabilities, separate output trust, and a caller-authorized credential
//! reference. Nothing here opens a transport, reads a bearer, publishes worker
//! demand, reserves capacity, or turns a pool id into authority.

use alloc::string::String;
use alloc::vec::Vec;

use crate::endpoint::EndpointBinding;
use crate::endpoint::EndpointBindingError;
use crate::endpoint::EndpointBindingTable;

/// Maximum endpoints one selected pool may declare.
pub const MAX_POOL_ENDPOINTS: usize = 64;
/// Maximum bytes of a pool identifier.
pub const MAX_POOL_ID_BYTES: usize = 256;
/// Maximum bytes of a credential reference name.
pub const MAX_CREDENTIAL_REFERENCE_BYTES: usize = 256;
/// Maximum bytes of one output key name.
pub const MAX_OUTPUT_KEY_NAME_BYTES: usize = 256;
/// Maximum bytes of one output key material (an Ed25519 public key is shorter).
pub const MAX_OUTPUT_KEY_MATERIAL_BYTES: usize = 512;
/// Maximum output keys one endpoint may declare.
pub const MAX_OUTPUT_KEYS: usize = 64;
/// Maximum declared capabilities one endpoint may list.
pub const MAX_CAPABILITIES: usize = 32;
/// Maximum bytes of one declared capability.
pub const MAX_CAPABILITY_BYTES: usize = 128;
/// Maximum declared concurrency of one pool.
pub const MAX_POOL_CONCURRENCY: u32 = 1_024;
/// Maximum concrete jobs one pool session may dispatch.
pub const MAX_SESSION_JOBS: u32 = 1_000_000;
/// Maximum declared bytes one pool session may transfer.
pub const MAX_SESSION_BYTES: u64 = 1_099_511_627_776;

/// Native transports a pool endpoint may declare today.
pub const SUPPORTED_NATIVE_TRANSPORTS: [&str; 3] = ["stdio", "ssh-stdio", "p2p"];
/// Sandbox modes an endpoint may declare.
pub const SUPPORTED_SANDBOX_MODES: [&str; 2] = ["practical", "strict"];
/// Network modes an endpoint may declare.
pub const SUPPORTED_NETWORK_MODES: [&str; 3] = ["none", "local", "full"];

/// Declared execution capabilities of one endpoint. These are declarations the
/// receiver still enforces independently; they never infer capacity.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct DeclaredCapabilities {
    pub system: String,
    pub sandbox_mode: String,
    pub network_mode: String,
    pub features: Vec<String>,
}

/// One separately declared output-verification key, by name and full material.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct TrustedOutputKey {
    pub name: String,
    pub material_base64: String,
}

/// One endpoint as the operator configuration declared it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct PoolEndpoint {
    /// The binding the caller presents; the current table decides admission.
    pub presented_binding: EndpointBinding,
    pub transport: String,
    pub capabilities: DeclaredCapabilities,
    pub trusted_output_keys: Vec<TrustedOutputKey>,
    /// Shell-owned handle name for a caller-owned bearer; never a bearer value.
    pub credential_reference: String,
    /// Whether this endpoint explicitly declared a quantified resource inventory.
    pub declares_resource_inventory: bool,
}

/// Bounded session policy of one selected pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct PoolSessionPolicy {
    pub max_concurrency: u32,
    pub job_count_max: u32,
    pub session_bytes_max: u64,
}

impl PoolSessionPolicy {
    pub fn is_bounded(&self) -> bool {
        self.max_concurrency >= 1
            && self.max_concurrency <= MAX_POOL_CONCURRENCY
            && self.job_count_max >= 1
            && self.job_count_max <= MAX_SESSION_JOBS
            && self.session_bytes_max >= 1
            && self.session_bytes_max <= MAX_SESSION_BYTES
    }
}

/// The complete operator selection: one pool id, its endpoints, and its bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct PoolSelection {
    pub pool_id: String,
    pub endpoints: Vec<PoolEndpoint>,
    pub session: PoolSessionPolicy,
}

/// One caller-owned credential the shell resolved for exactly one endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallerCredential {
    pub endpoint_name: String,
    pub reference: String,
}

/// Denials of one pool selection. The first failing rule wins in declaration
/// order, so the same declaration always reports the same denial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolAdmissionError {
    InvalidPoolId,
    EmptyPool,
    EndpointCountExceeded,
    DuplicateEndpoint,
    UnsupportedTransport,
    UndeclaredCapabilities,
    UnsupportedSandboxMode,
    UnsupportedNetworkMode,
    CapabilityCountExceeded,
    UndeclaredOutputTrust,
    AmbiguousOutputKey,
    InvalidCredentialReference,
    DuplicateCredentialReference,
    MissingCallerCredential,
    UnboundedSessionPolicy,
    InferredCapacity,
    EndpointBinding(EndpointBindingError),
}

/// Admitted pool facts. Dispatchers may select among exactly these endpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedPool {
    pool_id: String,
    endpoint_names: Vec<String>,
    session: PoolSessionPolicy,
}

impl AdmittedPool {
    pub fn pool_id(&self) -> &str {
        &self.pool_id
    }

    pub fn endpoint_names(&self) -> &[String] {
        &self.endpoint_names
    }

    pub fn session(&self) -> PoolSessionPolicy {
        self.session
    }
}

/// Decide whether one operator-selected pool may dispatch against current bindings and caller-owned
/// credentials. Binding expiry, revocation, and name/generation agreement are
/// decided by the current table at `now_unix_s`.
pub fn admits_pool(
    selection: &PoolSelection,
    bindings: &EndpointBindingTable,
    caller_credentials: &[CallerCredential],
    now_unix_s: u64,
) -> Result<AdmittedPool, PoolAdmissionError> {
    if !is_valid_identifier(&selection.pool_id, MAX_POOL_ID_BYTES) {
        return Err(PoolAdmissionError::InvalidPoolId);
    }
    if selection.endpoints.is_empty() {
        return Err(PoolAdmissionError::EmptyPool);
    }
    if selection.endpoints.len() > MAX_POOL_ENDPOINTS {
        return Err(PoolAdmissionError::EndpointCountExceeded);
    }
    if !selection.session.is_bounded() {
        return Err(PoolAdmissionError::UnboundedSessionPolicy);
    }
    admit_session_capacity(selection)?;
    let mut endpoint_names = Vec::with_capacity(selection.endpoints.len());
    let mut credential_references = Vec::with_capacity(selection.endpoints.len());
    for endpoint in &selection.endpoints {
        let name = endpoint.presented_binding.spec.name.clone();
        if endpoint_names.contains(&name) {
            return Err(PoolAdmissionError::DuplicateEndpoint);
        }
        admit_endpoint_shape(endpoint)?;
        bindings
            .resolve(&endpoint.presented_binding, now_unix_s)
            .map_err(PoolAdmissionError::EndpointBinding)?;
        admit_endpoint_credential(endpoint, caller_credentials, &credential_references)?;
        credential_references.push(endpoint.credential_reference.clone());
        endpoint_names.push(name);
    }
    debug_assert_eq!(endpoint_names.len(), selection.endpoints.len());
    debug_assert!(endpoint_names.iter().all(|name| !name.is_empty()));
    Ok(AdmittedPool {
        pool_id: selection.pool_id.clone(),
        endpoint_names,
        session: selection.session,
    })
}

fn admit_session_capacity(selection: &PoolSelection) -> Result<(), PoolAdmissionError> {
    let is_concurrent = selection.session.max_concurrency > 1;
    let has_inventory = selection.endpoints.iter().any(|endpoint| endpoint.declares_resource_inventory);
    if is_concurrent && !has_inventory {
        return Err(PoolAdmissionError::InferredCapacity);
    }
    debug_assert!(selection.session.is_bounded());
    debug_assert!(selection.endpoints.len() <= MAX_POOL_ENDPOINTS);
    Ok(())
}

fn admit_endpoint_shape(endpoint: &PoolEndpoint) -> Result<(), PoolAdmissionError> {
    if !SUPPORTED_NATIVE_TRANSPORTS.contains(&endpoint.transport.as_str()) {
        return Err(PoolAdmissionError::UnsupportedTransport);
    }
    let capabilities = &endpoint.capabilities;
    let has_declared_system = is_valid_identifier(&capabilities.system, MAX_CAPABILITY_BYTES);
    let has_declared_modes = SUPPORTED_SANDBOX_MODES.contains(&capabilities.sandbox_mode.as_str())
        && SUPPORTED_NETWORK_MODES.contains(&capabilities.network_mode.as_str());
    if !has_declared_system {
        return Err(PoolAdmissionError::UndeclaredCapabilities);
    }
    if !SUPPORTED_SANDBOX_MODES.contains(&capabilities.sandbox_mode.as_str()) {
        return Err(PoolAdmissionError::UnsupportedSandboxMode);
    }
    if !SUPPORTED_NETWORK_MODES.contains(&capabilities.network_mode.as_str()) {
        return Err(PoolAdmissionError::UnsupportedNetworkMode);
    }
    if capabilities.features.len() > MAX_CAPABILITIES {
        return Err(PoolAdmissionError::CapabilityCountExceeded);
    }
    let features_valid = capabilities.features.iter().all(|feature| is_valid_identifier(feature, MAX_CAPABILITY_BYTES));
    if !features_valid {
        return Err(PoolAdmissionError::UndeclaredCapabilities);
    }
    admit_endpoint_output_trust(endpoint)?;
    debug_assert!(has_declared_modes);
    debug_assert!(!endpoint.trusted_output_keys.is_empty());
    Ok(())
}

fn admit_endpoint_output_trust(endpoint: &PoolEndpoint) -> Result<(), PoolAdmissionError> {
    let keys = &endpoint.trusted_output_keys;
    if keys.is_empty() || keys.len() > MAX_OUTPUT_KEYS {
        return Err(PoolAdmissionError::UndeclaredOutputTrust);
    }
    let mut names = Vec::with_capacity(keys.len());
    let mut materials = Vec::with_capacity(keys.len());
    for key in keys {
        let has_valid_name = is_valid_identifier(&key.name, MAX_OUTPUT_KEY_NAME_BYTES);
        let has_valid_material =
            !key.material_base64.is_empty() && key.material_base64.len() <= MAX_OUTPUT_KEY_MATERIAL_BYTES;
        if !has_valid_name || !has_valid_material {
            return Err(PoolAdmissionError::UndeclaredOutputTrust);
        }
        if names.contains(&key.name) || materials.contains(&key.material_base64) {
            return Err(PoolAdmissionError::AmbiguousOutputKey);
        }
        names.push(key.name.clone());
        materials.push(key.material_base64.clone());
    }
    debug_assert_eq!(names.len(), keys.len());
    debug_assert_eq!(materials.len(), keys.len());
    Ok(())
}

fn admit_endpoint_credential(
    endpoint: &PoolEndpoint,
    caller_credentials: &[CallerCredential],
    seen_references: &[String],
) -> Result<(), PoolAdmissionError> {
    let reference = &endpoint.credential_reference;
    if !is_valid_identifier(reference, MAX_CREDENTIAL_REFERENCE_BYTES) {
        return Err(PoolAdmissionError::InvalidCredentialReference);
    }
    if seen_references.contains(reference) {
        return Err(PoolAdmissionError::DuplicateCredentialReference);
    }
    let endpoint_name = &endpoint.presented_binding.spec.name;
    let is_authorized = caller_credentials
        .iter()
        .any(|credential| credential.endpoint_name == *endpoint_name && credential.reference == *reference);
    if !is_authorized {
        return Err(PoolAdmissionError::MissingCallerCredential);
    }
    debug_assert!(caller_credentials.len() <= MAX_POOL_ENDPOINTS);
    debug_assert!(!reference.is_empty());
    Ok(())
}

fn is_valid_identifier(value: &str, max_bytes: usize) -> bool {
    let is_within_bound = !value.is_empty() && value.len() <= max_bytes;
    let has_no_whitespace = !value.contains(char::is_whitespace);
    is_within_bound && has_no_whitespace
}
