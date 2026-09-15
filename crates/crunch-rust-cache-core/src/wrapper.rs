//! Pure protocol, manifest, policy, and bypass decisions for the Rust compiler wrapper.
//!
//! This module performs no filesystem, socket, process, environment, clock, or
//! network access. Shell code supplies current observations and executes plans.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::BLAKE3_HEX_CHARS;
use crate::RustUnitAction;
use crate::error::RustCacheError;
use crate::validate_rust_action;

pub const WRAPPER_MANIFEST_SCHEMA: &str = "mantle-rustc-wrapper-manifest-v1";
pub const WRAPPER_REQUEST_SCHEMA: &str = "mantle-rustc-wrapper-request-v1";
pub const WRAPPER_RESPONSE_SCHEMA: &str = "mantle-rustc-wrapper-response-v1";
pub const WRAPPER_POLICY_SCHEMA: &str = "mantle-rustc-wrapper-policy-v1";
pub const WRAPPER_RECEIPT_SCHEMA: &str = "mantle-rustc-wrapper-receipt-v1";
pub const WRAPPER_MANIFEST_REF_PREFIX: &str = "mantle-rustc-manifest://blake3/";
pub const WRAPPER_REQUEST_REF_PREFIX: &str = "mantle-rustc-request://blake3/";
pub const WRAPPER_RECEIPT_REF_PREFIX: &str = "mantle-rustc-receipt://blake3/";
pub const MAX_WRAPPER_ARGUMENTS: usize = 4_096;
pub const MAX_WRAPPER_ENVIRONMENT: usize = 512;
pub const MAX_WRAPPER_INPUTS: usize = 8_192;
pub const MAX_WRAPPER_OUTPUTS: usize = 256;
pub const MAX_WRAPPER_ROOTS: usize = 256;
pub const MAX_WRAPPER_PEERS: usize = 256;
pub const MAX_WRAPPER_RESULT_SOURCES: usize = 16;
pub const MAX_WRAPPER_REASON_CODES: usize = 64;
pub const MAX_WRAPPER_STRING_BYTES: usize = 16_384;
pub const MAX_WRAPPER_PATH_BYTES: usize = 4_096;
pub const MAX_WRAPPER_REQUEST_BYTES: u64 = 8_388_608;
pub const MAX_WRAPPER_RESPONSE_BYTES: u64 = 8_388_608;
pub const MAX_WRAPPER_STREAM_BYTES: u64 = 1_048_576;
pub const MAX_WRAPPER_ARTIFACT_BYTES: u64 = 8_589_934_592;
pub const MAX_WRAPPER_ELAPSED_MILLIS: u64 = 3_600_000;
pub const MAX_WRAPPER_CONCURRENCY: u32 = 256;
pub const MIN_WRAPPER_CONCURRENCY: u32 = 1;
pub const MAX_WRAPPER_FRAME_PREFIX_BYTES: usize = 4;
pub const WRAPPER_SOCKET_MODE: u32 = 0o600;
pub const WRAPPER_PRIVATE_DIRECTORY_MODE: u32 = 0o700;

const MANIFEST_DOMAIN: &[u8] = b"mantle.rustc-wrapper.manifest.v1";
const REQUEST_DOMAIN: &[u8] = b"mantle.rustc-wrapper.request.v1";
const RECEIPT_DOMAIN: &[u8] = b"mantle.rustc-wrapper.receipt.v1";
const ARGUMENTS_DOMAIN: &[u8] = b"mantle.rustc-wrapper.arguments.v1";
const ENVIRONMENT_DOMAIN: &[u8] = b"mantle.rustc-wrapper.environment.v1";
const DOMAIN_SEPARATOR: u8 = 0;

#[derive(Clone, Copy)]
struct ValidationCode<'a>(&'a str);

#[derive(Clone, Copy)]
struct TypedRefRule<'a> {
    prefix: &'a str,
    code: ValidationCode<'a>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperFailureMode {
    FailOpen,
    FailClosed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperCacheMode {
    Off,
    LocalRead,
    LocalReadWrite,
    SharedRead,
    SharedReadWrite,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum DeclaredInputKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperArtifactKind {
    File,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FilesystemEffect {
    DeclaredRoots,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkEffect {
    Deny,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AmbientEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperDisposition {
    LocalHit,
    SharedHit,
    Compiled,
    Bypass,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperResultSourceKind {
    Directory,
    Http,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperBypassClass {
    CompilerQuery,
    IncrementalCompilation,
    ResponseFile,
    MissingManifest,
    ManifestInvalid,
    ManifestIdentityMismatch,
    InputMissing,
    InputChanged,
    InputOutOfRoot,
    InputUnclassified,
    OutputContractUnsupported,
    EffectPolicyUnsupported,
    EnforcementUnavailable,
    DaemonUnavailable,
    ProtocolFailure,
    PolicyRejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct DeclaredInput {
    pub role: String,
    pub path: String,
    pub kind: DeclaredInputKind,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct WrapperOutputContract {
    pub staged_relative_path: String,
    pub destination_path: String,
    pub kind: WrapperArtifactKind,
    pub max_bytes: u64,
    pub executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperEffectPolicy {
    pub filesystem: FilesystemEffect,
    pub network: NetworkEffect,
    pub clock: AmbientEffect,
    pub randomness: AmbientEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperResourceLimits {
    pub elapsed_millis: u64,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub artifact_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperInvocationManifestInput {
    pub policy_id: String,
    pub action: RustUnitAction,
    pub real_compiler_path: String,
    pub arguments_blake3: String,
    pub environment_blake3: String,
    pub working_directory: String,
    pub readable_roots: Vec<String>,
    pub writable_roots: Vec<String>,
    pub declared_inputs: Vec<DeclaredInput>,
    pub output_contracts: Vec<WrapperOutputContract>,
    pub effects: WrapperEffectPolicy,
    pub limits: WrapperResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperInvocationManifest {
    pub schema: String,
    pub manifest_ref: String,
    #[serde(flatten)]
    pub input: WrapperInvocationManifestInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperRequestInput {
    pub manifest_path: Option<String>,
    pub expected_manifest_ref: Option<String>,
    pub real_compiler_path: String,
    pub arguments: Vec<String>,
    pub admitted_environment: BTreeMap<String, String>,
    pub working_directory: String,
    pub output_contracts: Vec<WrapperOutputContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperRequest {
    pub schema: String,
    pub request_ref: String,
    #[serde(flatten)]
    pub input: WrapperRequestInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperResponse {
    pub schema: String,
    pub request_ref: String,
    pub disposition: WrapperDisposition,
    pub bypass_class: Option<WrapperBypassClass>,
    pub reason_codes: Vec<String>,
    pub compiler_status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub artifact_commit_complete: bool,
    pub receipt_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct WrapperResultSourcePolicy {
    pub priority: u32,
    pub source_id: String,
    pub kind: WrapperResultSourceKind,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperPublicationPolicy {
    pub enabled: bool,
    pub source_id: Option<String>,
    pub signer_name: Option<String>,
    pub signing_key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperDaemonPolicy {
    pub schema: String,
    pub policy_id: String,
    pub cache_mode: WrapperCacheMode,
    pub failure_mode: WrapperFailureMode,
    pub socket_path: String,
    pub allowed_peer_uids: Vec<u32>,
    pub readable_roots: Vec<String>,
    pub writable_roots: Vec<String>,
    pub allowed_environment: Vec<String>,
    pub effects: WrapperEffectPolicy,
    pub limits: WrapperResourceLimits,
    pub max_request_bytes: u64,
    pub max_response_bytes: u64,
    pub max_concurrency: u32,
    pub sandbox_program: String,
    pub sandbox_program_blake3: String,
    pub result_sources: Vec<WrapperResultSourcePolicy>,
    pub shared_trust_policy: Option<crate::shared::RustResultTrustPolicy>,
    pub publication: WrapperPublicationPolicy,
    pub local_reads_enabled: bool,
    pub local_writes_enabled: bool,
    pub shared_reads_enabled: bool,
    pub shared_writes_enabled: bool,
    pub redact_environment_values: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperReceiptInput {
    pub request_ref: String,
    pub manifest_ref: Option<String>,
    pub policy_id: String,
    pub disposition: WrapperDisposition,
    pub bypass_class: Option<WrapperBypassClass>,
    pub action_ref: Option<String>,
    pub result_ref: Option<String>,
    pub artifact_digests_blake3: Vec<String>,
    pub compiler_status: i32,
    pub compiler_executed: bool,
    pub artifact_commit_complete: bool,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WrapperReceipt {
    pub schema: String,
    pub receipt_ref: String,
    #[serde(flatten)]
    pub input: WrapperReceiptInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CurrentManifestFacts {
    pub manifest_ref_matches: bool,
    pub compiler_matches: bool,
    pub arguments_match: bool,
    pub environment_matches: bool,
    pub working_directory_matches: bool,
    pub inputs_verified: bool,
    pub outputs_admissible: bool,
    pub enforcement_available: bool,
    pub failure_class: Option<WrapperBypassClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperDecision {
    StrongEligible,
    Bypass(WrapperBypassClass),
    Reject(WrapperBypassClass),
}

pub fn seal_wrapper_manifest(
    input: WrapperInvocationManifestInput,
) -> Result<WrapperInvocationManifest, RustCacheError> {
    let input = normalize_manifest_input(input);
    validate_manifest_input(&input)?;
    let manifest_ref = digest_ref(WRAPPER_MANIFEST_REF_PREFIX, MANIFEST_DOMAIN, &input)?;
    let manifest = WrapperInvocationManifest {
        schema: WRAPPER_MANIFEST_SCHEMA.to_string(),
        manifest_ref,
        input,
    };
    validate_wrapper_manifest(&manifest)?;
    assert_eq!(manifest.schema, WRAPPER_MANIFEST_SCHEMA);
    assert!(manifest.manifest_ref.starts_with(WRAPPER_MANIFEST_REF_PREFIX));
    Ok(manifest)
}

pub fn validate_wrapper_manifest(manifest: &WrapperInvocationManifest) -> Result<(), RustCacheError> {
    if manifest.schema != WRAPPER_MANIFEST_SCHEMA {
        return Err(RustCacheError::new("rustc-wrapper-manifest-schema-unsupported".to_string()));
    }
    validate_manifest_input(&manifest.input)?;
    let expected_ref = digest_ref(WRAPPER_MANIFEST_REF_PREFIX, MANIFEST_DOMAIN, &manifest.input)?;
    if manifest.manifest_ref != expected_ref {
        return Err(RustCacheError::new("rustc-wrapper-manifest-ref-mismatch".to_string()));
    }
    ensure_record_bound(manifest, MAX_WRAPPER_REQUEST_BYTES, "rustc-wrapper-manifest-too-large")?;
    assert!(manifest.manifest_ref.starts_with(WRAPPER_MANIFEST_REF_PREFIX));
    assert_eq!(manifest.schema, WRAPPER_MANIFEST_SCHEMA);
    Ok(())
}

pub fn seal_wrapper_request(input: WrapperRequestInput) -> Result<WrapperRequest, RustCacheError> {
    validate_request_input(&input)?;
    let request_ref = digest_ref(WRAPPER_REQUEST_REF_PREFIX, REQUEST_DOMAIN, &input)?;
    let request = WrapperRequest {
        schema: WRAPPER_REQUEST_SCHEMA.to_string(),
        request_ref,
        input,
    };
    validate_wrapper_request(&request)?;
    assert_eq!(request.schema, WRAPPER_REQUEST_SCHEMA);
    assert!(request.request_ref.starts_with(WRAPPER_REQUEST_REF_PREFIX));
    Ok(request)
}

pub fn validate_wrapper_request(request: &WrapperRequest) -> Result<(), RustCacheError> {
    if request.schema != WRAPPER_REQUEST_SCHEMA {
        return Err(RustCacheError::new("rustc-wrapper-request-schema-unsupported".to_string()));
    }
    validate_request_input(&request.input)?;
    let expected_ref = digest_ref(WRAPPER_REQUEST_REF_PREFIX, REQUEST_DOMAIN, &request.input)?;
    if request.request_ref != expected_ref {
        return Err(RustCacheError::new("rustc-wrapper-request-ref-mismatch".to_string()));
    }
    ensure_record_bound(request, MAX_WRAPPER_REQUEST_BYTES, "rustc-wrapper-request-too-large")?;
    assert!(request.request_ref.starts_with(WRAPPER_REQUEST_REF_PREFIX));
    assert_eq!(request.schema, WRAPPER_REQUEST_SCHEMA);
    Ok(())
}

pub fn validate_wrapper_response(response: &WrapperResponse, max_response_bytes: u64) -> Result<(), RustCacheError> {
    if response.schema != WRAPPER_RESPONSE_SCHEMA {
        return Err(RustCacheError::new("rustc-wrapper-response-schema-unsupported".to_string()));
    }
    validate_typed_ref(&response.request_ref, TypedRefRule {
        prefix: WRAPPER_REQUEST_REF_PREFIX,
        code: ValidationCode("rustc-wrapper-response-request-invalid"),
    })?;
    validate_reason_codes(&response.reason_codes)?;
    let stdout_bytes = u64::try_from(response.stdout.len())
        .map_err(|_| RustCacheError::new("rustc-wrapper-response-stdout-unrepresentable"))?;
    let stderr_bytes = u64::try_from(response.stderr.len())
        .map_err(|_| RustCacheError::new("rustc-wrapper-response-stderr-unrepresentable"))?;
    if stdout_bytes > MAX_WRAPPER_STREAM_BYTES || stderr_bytes > MAX_WRAPPER_STREAM_BYTES {
        return Err(RustCacheError::new("rustc-wrapper-response-stream-limit-exceeded".to_string()));
    }
    let is_bypass = response.disposition == WrapperDisposition::Bypass;
    let is_rejected = response.disposition == WrapperDisposition::Rejected;
    if response.bypass_class.is_some() != (is_bypass || is_rejected) {
        return Err(RustCacheError::new("rustc-wrapper-response-bypass-class-invalid".to_string()));
    }
    if response.artifact_commit_complete && response.receipt_ref.is_none() {
        return Err(RustCacheError::new("rustc-wrapper-response-receipt-missing".to_string()));
    }
    if let Some(receipt_ref) = &response.receipt_ref {
        validate_typed_ref(receipt_ref, TypedRefRule {
            prefix: WRAPPER_RECEIPT_REF_PREFIX,
            code: ValidationCode("rustc-wrapper-response-receipt-invalid"),
        })?;
    }
    ensure_record_bound(response, max_response_bytes, "rustc-wrapper-response-too-large")?;
    assert!(stdout_bytes <= MAX_WRAPPER_STREAM_BYTES);
    assert!(stderr_bytes <= MAX_WRAPPER_STREAM_BYTES);
    Ok(())
}

pub fn seal_wrapper_receipt(input: WrapperReceiptInput) -> Result<WrapperReceipt, RustCacheError> {
    validate_receipt_input(&input)?;
    let receipt_ref = digest_ref(WRAPPER_RECEIPT_REF_PREFIX, RECEIPT_DOMAIN, &input)?;
    let receipt = WrapperReceipt {
        schema: WRAPPER_RECEIPT_SCHEMA.to_string(),
        receipt_ref,
        input,
    };
    validate_wrapper_receipt(&receipt)?;
    assert_eq!(receipt.schema, WRAPPER_RECEIPT_SCHEMA);
    assert!(receipt.receipt_ref.starts_with(WRAPPER_RECEIPT_REF_PREFIX));
    Ok(receipt)
}

pub fn validate_wrapper_receipt(receipt: &WrapperReceipt) -> Result<(), RustCacheError> {
    if receipt.schema != WRAPPER_RECEIPT_SCHEMA {
        return Err(RustCacheError::new("rustc-wrapper-receipt-schema-unsupported".to_string()));
    }
    validate_receipt_input(&receipt.input)?;
    let expected_ref = digest_ref(WRAPPER_RECEIPT_REF_PREFIX, RECEIPT_DOMAIN, &receipt.input)?;
    if receipt.receipt_ref != expected_ref {
        return Err(RustCacheError::new("rustc-wrapper-receipt-ref-mismatch".to_string()));
    }
    ensure_record_bound(receipt, MAX_WRAPPER_RESPONSE_BYTES, "rustc-wrapper-receipt-too-large")?;
    assert!(receipt.receipt_ref.starts_with(WRAPPER_RECEIPT_REF_PREFIX));
    assert_eq!(receipt.schema, WRAPPER_RECEIPT_SCHEMA);
    Ok(())
}

pub fn validate_wrapper_policy(policy: &WrapperDaemonPolicy) -> Result<(), RustCacheError> {
    if policy.schema != WRAPPER_POLICY_SCHEMA {
        return Err(RustCacheError::new("rustc-wrapper-policy-schema-unsupported".to_string()));
    }
    validate_identifier(&policy.policy_id, ValidationCode("rustc-wrapper-policy-id-invalid"))?;
    validate_absolute_path(&policy.socket_path, ValidationCode("rustc-wrapper-policy-socket-invalid"))?;
    validate_absolute_path(&policy.sandbox_program, ValidationCode("rustc-wrapper-policy-sandbox-invalid"))?;
    validate_blake3(&policy.sandbox_program_blake3, ValidationCode("rustc-wrapper-policy-sandbox-digest-invalid"))?;
    validate_sorted_uids(&policy.allowed_peer_uids)?;
    validate_absolute_roots(&policy.readable_roots, "rustc-wrapper-policy-readable-roots-invalid")?;
    validate_absolute_roots(&policy.writable_roots, "rustc-wrapper-policy-writable-roots-invalid")?;
    validate_sorted_strings(
        &policy.allowed_environment,
        MAX_WRAPPER_ENVIRONMENT,
        "rustc-wrapper-policy-environment-invalid",
    )?;
    validate_effects(&policy.effects)?;
    validate_limits(&policy.limits)?;
    validate_result_sources(&policy.result_sources)?;
    validate_result_source_mode(policy)?;
    validate_shared_trust_policy(policy)?;
    validate_publication(&policy.publication, policy.shared_writes_enabled)?;
    if policy.max_request_bytes == 0 || policy.max_request_bytes > MAX_WRAPPER_REQUEST_BYTES {
        return Err(RustCacheError::new("rustc-wrapper-policy-request-limit-invalid".to_string()));
    }
    if policy.max_response_bytes == 0 || policy.max_response_bytes > MAX_WRAPPER_RESPONSE_BYTES {
        return Err(RustCacheError::new("rustc-wrapper-policy-response-limit-invalid".to_string()));
    }
    if policy.max_concurrency < MIN_WRAPPER_CONCURRENCY || policy.max_concurrency > MAX_WRAPPER_CONCURRENCY {
        return Err(RustCacheError::new("rustc-wrapper-policy-concurrency-invalid".to_string()));
    }
    validate_cache_mode_consistency(policy)?;
    if !policy.redact_environment_values {
        return Err(RustCacheError::new("rustc-wrapper-policy-redaction-required".to_string()));
    }
    assert!(policy.max_request_bytes <= MAX_WRAPPER_REQUEST_BYTES);
    assert!(policy.max_concurrency <= MAX_WRAPPER_CONCURRENCY);
    Ok(())
}

pub fn arguments_identity_blake3(arguments: &[String]) -> Result<String, RustCacheError> {
    validate_argument_list(arguments)?;
    canonical_digest(ARGUMENTS_DOMAIN, arguments)
}

pub fn environment_identity_blake3(environment: &BTreeMap<String, String>) -> Result<String, RustCacheError> {
    validate_environment(environment)?;
    canonical_digest(ENVIRONMENT_DOMAIN, environment)
}

pub fn classify_wrapper_request(
    request: &WrapperRequest,
    manifest: Option<&WrapperInvocationManifest>,
    policy: &WrapperDaemonPolicy,
    facts: Option<&CurrentManifestFacts>,
) -> WrapperDecision {
    if validate_wrapper_policy(policy).is_err() || validate_wrapper_request(request).is_err() {
        return failure_decision(policy.failure_mode, WrapperBypassClass::PolicyRejected);
    }
    if let Some(class) = classify_argument_bypass(&request.input.arguments) {
        return WrapperDecision::Bypass(class);
    }
    let Some(manifest) = manifest else {
        return failure_decision(policy.failure_mode, WrapperBypassClass::MissingManifest);
    };
    if validate_wrapper_manifest(manifest).is_err() {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestInvalid);
    }
    if !manifest_static_matches_request(manifest, request) {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestIdentityMismatch);
    }
    let Some(facts) = facts else {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestInvalid);
    };
    if let Some(class) = facts.failure_class {
        return failure_decision(policy.failure_mode, class);
    }
    if !facts.manifest_ref_matches {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestIdentityMismatch);
    }
    if !facts.compiler_matches {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestIdentityMismatch);
    }
    if !facts.arguments_match {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestIdentityMismatch);
    }
    if !facts.environment_matches {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestIdentityMismatch);
    }
    if !facts.working_directory_matches {
        return failure_decision(policy.failure_mode, WrapperBypassClass::ManifestIdentityMismatch);
    }
    if !facts.inputs_verified {
        return failure_decision(policy.failure_mode, WrapperBypassClass::InputChanged);
    }
    if !facts.outputs_admissible {
        return failure_decision(policy.failure_mode, WrapperBypassClass::OutputContractUnsupported);
    }
    if !facts.enforcement_available {
        return failure_decision(policy.failure_mode, WrapperBypassClass::EnforcementUnavailable);
    }
    assert!(facts.inputs_verified);
    assert!(facts.enforcement_available);
    WrapperDecision::StrongEligible
}

pub fn classify_argument_bypass(arguments: &[String]) -> Option<WrapperBypassClass> {
    if arguments.iter().any(|argument| {
        matches!(argument.as_str(), "-vV" | "-V" | "--version" | "--help")
            || argument == "--print"
            || argument.starts_with("--print=")
    }) {
        return Some(WrapperBypassClass::CompilerQuery);
    }
    if arguments
        .iter()
        .any(|argument| argument == "-Cincremental" || argument.starts_with("-Cincremental="))
    {
        return Some(WrapperBypassClass::IncrementalCompilation);
    }
    if arguments.iter().any(|argument| argument.starts_with('@')) {
        return Some(WrapperBypassClass::ResponseFile);
    }
    assert!(!arguments.is_empty());
    assert!(arguments.len() <= MAX_WRAPPER_ARGUMENTS);
    None
}

pub fn failure_decision(mode: WrapperFailureMode, class: WrapperBypassClass) -> WrapperDecision {
    let decision = match mode {
        WrapperFailureMode::FailOpen => WrapperDecision::Bypass(class),
        WrapperFailureMode::FailClosed => WrapperDecision::Reject(class),
    };
    assert_eq!(matches!(decision, WrapperDecision::Bypass(_)), mode == WrapperFailureMode::FailOpen);
    assert_eq!(matches!(decision, WrapperDecision::Reject(_)), mode == WrapperFailureMode::FailClosed);
    decision
}

fn normalize_manifest_input(mut input: WrapperInvocationManifestInput) -> WrapperInvocationManifestInput {
    input.readable_roots.sort();
    input.readable_roots.dedup();
    input.writable_roots.sort();
    input.writable_roots.dedup();
    input.declared_inputs.sort();
    input.declared_inputs.dedup();
    input.output_contracts.sort();
    input.output_contracts.dedup();
    input
}

fn validate_manifest_input(input: &WrapperInvocationManifestInput) -> Result<(), RustCacheError> {
    validate_identifier(&input.policy_id, ValidationCode("rustc-wrapper-manifest-policy-invalid"))?;
    validate_rust_action(&input.action)?;
    validate_absolute_path(&input.real_compiler_path, ValidationCode("rustc-wrapper-manifest-compiler-invalid"))?;
    validate_blake3(&input.arguments_blake3, ValidationCode("rustc-wrapper-manifest-arguments-invalid"))?;
    validate_blake3(&input.environment_blake3, ValidationCode("rustc-wrapper-manifest-environment-invalid"))?;
    validate_absolute_path(
        &input.working_directory,
        ValidationCode("rustc-wrapper-manifest-working-directory-invalid"),
    )?;
    validate_absolute_roots(&input.readable_roots, "rustc-wrapper-manifest-readable-roots-invalid")?;
    validate_absolute_roots(&input.writable_roots, "rustc-wrapper-manifest-writable-roots-invalid")?;
    validate_declared_inputs(&input.declared_inputs, &input.readable_roots)?;
    validate_manifest_action_bindings(input)?;
    validate_output_contracts(&input.output_contracts, &input.writable_roots)?;
    validate_effects(&input.effects)?;
    validate_limits(&input.limits)?;
    assert!(!input.declared_inputs.is_empty());
    assert!(!input.output_contracts.is_empty());
    Ok(())
}

fn validate_request_input(input: &WrapperRequestInput) -> Result<(), RustCacheError> {
    validate_absolute_path(&input.real_compiler_path, ValidationCode("rustc-wrapper-request-compiler-invalid"))?;
    validate_absolute_path(
        &input.working_directory,
        ValidationCode("rustc-wrapper-request-working-directory-invalid"),
    )?;
    validate_argument_list(&input.arguments)?;
    validate_environment(&input.admitted_environment)?;
    validate_output_contract_shape(&input.output_contracts)?;
    match (&input.manifest_path, &input.expected_manifest_ref) {
        (Some(path), Some(reference)) => {
            validate_absolute_path(path, ValidationCode("rustc-wrapper-request-manifest-path-invalid"))?;
            validate_typed_ref(reference, TypedRefRule {
                prefix: WRAPPER_MANIFEST_REF_PREFIX,
                code: ValidationCode("rustc-wrapper-request-manifest-ref-invalid"),
            })?;
        }
        (None, None) => {}
        _ => return Err(RustCacheError::new("rustc-wrapper-request-manifest-pair-invalid".to_string())),
    }
    assert!(input.arguments.len() <= MAX_WRAPPER_ARGUMENTS);
    assert!(input.admitted_environment.len() <= MAX_WRAPPER_ENVIRONMENT);
    Ok(())
}

fn validate_declared_inputs(inputs: &[DeclaredInput], roots: &[String]) -> Result<(), RustCacheError> {
    if inputs.is_empty() || inputs.len() > MAX_WRAPPER_INPUTS {
        return Err(RustCacheError::new("rustc-wrapper-manifest-input-count-invalid".to_string()));
    }
    if inputs != sorted_unique(inputs.to_vec()) {
        return Err(RustCacheError::new("rustc-wrapper-manifest-inputs-not-canonical".to_string()));
    }
    let mut roles = BTreeSet::new();
    for input in inputs {
        validate_identifier(&input.role, ValidationCode("rustc-wrapper-manifest-input-role-invalid"))?;
        validate_absolute_path(&input.path, ValidationCode("rustc-wrapper-manifest-input-path-invalid"))?;
        validate_blake3(&input.digest_blake3, ValidationCode("rustc-wrapper-manifest-input-digest-invalid"))?;
        if !path_is_within_any_root(&input.path, roots) {
            return Err(RustCacheError::new("rustc-wrapper-manifest-input-out-of-root".to_string()));
        }
        if !roles.insert(input.role.as_str()) {
            return Err(RustCacheError::new("rustc-wrapper-manifest-input-role-duplicate".to_string()));
        }
    }
    for required in ["compiler", "source", "sysroot"] {
        if !roles.contains(required) {
            return Err(RustCacheError::new("rustc-wrapper-manifest-required-input-missing".to_string()));
        }
    }
    assert!(inputs.len() <= MAX_WRAPPER_INPUTS);
    assert_eq!(roles.len(), inputs.len());
    Ok(())
}

fn validate_manifest_action_bindings(input: &WrapperInvocationManifestInput) -> Result<(), RustCacheError> {
    let compiler = declared_input_for_role(&input.declared_inputs, "compiler")?;
    let source = declared_input_for_role(&input.declared_inputs, "source")?;
    let sysroot = declared_input_for_role(&input.declared_inputs, "sysroot")?;
    if compiler.path != input.real_compiler_path || compiler.digest_blake3 != input.action.input.compiler_digest_blake3
    {
        return Err(RustCacheError::new("rustc-wrapper-manifest-compiler-binding-invalid".to_string()));
    }
    if source.digest_blake3 != input.action.input.source_digest_blake3 {
        return Err(RustCacheError::new("rustc-wrapper-manifest-source-binding-invalid".to_string()));
    }
    if sysroot.digest_blake3 != input.action.input.toolchain_closure_digest_blake3 {
        return Err(RustCacheError::new("rustc-wrapper-manifest-sysroot-binding-invalid".to_string()));
    }
    let action_environment_blake3 = environment_identity_blake3(&input.action.input.admitted_environment)?;
    if action_environment_blake3 != input.environment_blake3 {
        return Err(RustCacheError::new("rustc-wrapper-manifest-environment-binding-invalid".to_string()));
    }
    assert_eq!(compiler.path, input.real_compiler_path);
    assert_eq!(source.digest_blake3, input.action.input.source_digest_blake3);
    assert_eq!(sysroot.digest_blake3, input.action.input.toolchain_closure_digest_blake3);
    Ok(())
}

fn declared_input_for_role<'a>(inputs: &'a [DeclaredInput], role: &str) -> Result<&'a DeclaredInput, RustCacheError> {
    inputs
        .iter()
        .find(|input| input.role == role)
        .ok_or_else(|| RustCacheError::new("rustc-wrapper-manifest-required-input-missing"))
}

fn validate_output_contracts(outputs: &[WrapperOutputContract], roots: &[String]) -> Result<(), RustCacheError> {
    validate_output_contract_shape(outputs)?;
    for output in outputs {
        if !path_is_within_any_root(&output.destination_path, roots) {
            return Err(RustCacheError::new("rustc-wrapper-output-destination-out-of-root".to_string()));
        }
    }
    assert!(!outputs.is_empty());
    assert!(outputs.len() <= MAX_WRAPPER_OUTPUTS);
    Ok(())
}

fn validate_output_contract_shape(outputs: &[WrapperOutputContract]) -> Result<(), RustCacheError> {
    if outputs.is_empty() || outputs.len() > MAX_WRAPPER_OUTPUTS {
        return Err(RustCacheError::new("rustc-wrapper-output-count-invalid".to_string()));
    }
    if outputs != sorted_unique(outputs.to_vec()) {
        return Err(RustCacheError::new("rustc-wrapper-outputs-not-canonical".to_string()));
    }
    let mut destinations = BTreeSet::new();
    let mut staged_paths = BTreeSet::new();
    for output in outputs {
        validate_relative_path(&output.staged_relative_path)?;
        validate_absolute_path(&output.destination_path, ValidationCode("rustc-wrapper-output-destination-invalid"))?;
        if output.max_bytes == 0 || output.max_bytes > MAX_WRAPPER_ARTIFACT_BYTES {
            return Err(RustCacheError::new("rustc-wrapper-output-byte-limit-invalid".to_string()));
        }
        if !destinations.insert(output.destination_path.as_str()) {
            return Err(RustCacheError::new("rustc-wrapper-output-destination-duplicate".to_string()));
        }
        if !staged_paths.insert(output.staged_relative_path.as_str()) {
            return Err(RustCacheError::new("rustc-wrapper-output-staging-duplicate".to_string()));
        }
    }
    assert_eq!(destinations.len(), outputs.len());
    assert_eq!(staged_paths.len(), outputs.len());
    Ok(())
}

fn validate_effects(effects: &WrapperEffectPolicy) -> Result<(), RustCacheError> {
    if effects.filesystem != FilesystemEffect::DeclaredRoots || effects.network != NetworkEffect::Deny {
        return Err(RustCacheError::new("rustc-wrapper-effect-policy-unsupported".to_string()));
    }
    if effects.clock != AmbientEffect::Allow || effects.randomness != AmbientEffect::Allow {
        return Err(RustCacheError::new("rustc-wrapper-effect-policy-unsupported".to_string()));
    }
    assert_eq!(effects.filesystem, FilesystemEffect::DeclaredRoots);
    assert_eq!(effects.network, NetworkEffect::Deny);
    Ok(())
}

fn validate_limits(limits: &WrapperResourceLimits) -> Result<(), RustCacheError> {
    if limits.elapsed_millis == 0 || limits.elapsed_millis > MAX_WRAPPER_ELAPSED_MILLIS {
        return Err(RustCacheError::new("rustc-wrapper-elapsed-limit-invalid".to_string()));
    }
    if limits.stdout_bytes == 0 || limits.stdout_bytes > MAX_WRAPPER_STREAM_BYTES {
        return Err(RustCacheError::new("rustc-wrapper-stdout-limit-invalid".to_string()));
    }
    if limits.stderr_bytes == 0 || limits.stderr_bytes > MAX_WRAPPER_STREAM_BYTES {
        return Err(RustCacheError::new("rustc-wrapper-stderr-limit-invalid".to_string()));
    }
    if limits.artifact_bytes == 0 || limits.artifact_bytes > MAX_WRAPPER_ARTIFACT_BYTES {
        return Err(RustCacheError::new("rustc-wrapper-artifact-limit-invalid".to_string()));
    }
    assert!(limits.elapsed_millis <= MAX_WRAPPER_ELAPSED_MILLIS);
    assert!(limits.artifact_bytes <= MAX_WRAPPER_ARTIFACT_BYTES);
    Ok(())
}

fn validate_result_sources(sources: &[WrapperResultSourcePolicy]) -> Result<(), RustCacheError> {
    if sources.len() > MAX_WRAPPER_RESULT_SOURCES || sources != sorted_unique(sources.to_vec()) {
        return Err(RustCacheError::new("rustc-wrapper-result-sources-invalid".to_string()));
    }
    let mut source_ids = BTreeSet::new();
    let mut priorities = BTreeSet::new();
    for source in sources {
        validate_identifier(&source.source_id, ValidationCode("rustc-wrapper-result-source-id-invalid"))?;
        match source.kind {
            WrapperResultSourceKind::Directory => validate_absolute_path(
                &source.endpoint,
                ValidationCode("rustc-wrapper-result-source-endpoint-invalid"),
            )?,
            WrapperResultSourceKind::Http => {
                let is_http = source.endpoint.starts_with("http://") || source.endpoint.starts_with("https://");
                if !is_http || source.endpoint.contains('@') || source.endpoint.contains('#') {
                    return Err(RustCacheError::new("rustc-wrapper-result-source-endpoint-invalid".to_string()));
                }
            }
        }
        if !source_ids.insert(source.source_id.as_str()) || !priorities.insert(source.priority) {
            return Err(RustCacheError::new("rustc-wrapper-result-source-duplicate".to_string()));
        }
    }
    assert!(sources.len() <= MAX_WRAPPER_RESULT_SOURCES);
    assert_eq!(source_ids.len(), sources.len());
    Ok(())
}

fn validate_result_source_mode(policy: &WrapperDaemonPolicy) -> Result<(), RustCacheError> {
    if policy.shared_reads_enabled == policy.result_sources.is_empty() {
        return Err(RustCacheError::new("rustc-wrapper-result-source-mode-inconsistent".to_string()));
    }
    if let Some(source_id) = &policy.publication.source_id {
        let is_known = policy.result_sources.iter().any(|source| &source.source_id == source_id);
        if !is_known {
            return Err(RustCacheError::new("rustc-wrapper-publication-source-unknown".to_string()));
        }
    }
    assert_eq!(policy.shared_reads_enabled, !policy.result_sources.is_empty());
    if policy.shared_writes_enabled {
        assert!(policy.publication.source_id.is_some());
    }
    Ok(())
}

fn validate_shared_trust_policy(policy: &WrapperDaemonPolicy) -> Result<(), RustCacheError> {
    if policy.shared_reads_enabled != policy.shared_trust_policy.is_some() {
        return Err(RustCacheError::new("rustc-wrapper-shared-trust-policy-inconsistent".to_string()));
    }
    if let Some(trust_policy) = &policy.shared_trust_policy {
        crate::shared::validate_trust_policy(trust_policy)?;
    }
    assert_eq!(policy.shared_reads_enabled, policy.shared_trust_policy.is_some());
    Ok(())
}

fn validate_publication(
    publication: &WrapperPublicationPolicy,
    shared_writes_enabled: bool,
) -> Result<(), RustCacheError> {
    if publication.enabled != shared_writes_enabled {
        return Err(RustCacheError::new("rustc-wrapper-publication-mode-inconsistent".to_string()));
    }
    let is_complete =
        publication.source_id.is_some() && publication.signer_name.is_some() && publication.signing_key_path.is_some();
    if publication.enabled != is_complete {
        return Err(RustCacheError::new("rustc-wrapper-publication-fields-invalid".to_string()));
    }
    if let Some(source_id) = &publication.source_id {
        validate_identifier(source_id, ValidationCode("rustc-wrapper-publication-source-invalid"))?;
    }
    if let Some(signer_name) = &publication.signer_name {
        validate_identifier(signer_name, ValidationCode("rustc-wrapper-publication-signer-invalid"))?;
    }
    if let Some(key_path) = &publication.signing_key_path {
        validate_absolute_path(key_path, ValidationCode("rustc-wrapper-publication-key-path-invalid"))?;
    }
    assert_eq!(publication.enabled, shared_writes_enabled);
    assert_eq!(publication.enabled, is_complete);
    Ok(())
}

fn validate_cache_mode_consistency(policy: &WrapperDaemonPolicy) -> Result<(), RustCacheError> {
    let expected = match policy.cache_mode {
        WrapperCacheMode::Off => (false, false, false, false),
        WrapperCacheMode::LocalRead => (true, false, false, false),
        WrapperCacheMode::LocalReadWrite => (true, true, false, false),
        WrapperCacheMode::SharedRead => (true, false, true, false),
        WrapperCacheMode::SharedReadWrite => (true, true, true, true),
    };
    let actual = (
        policy.local_reads_enabled,
        policy.local_writes_enabled,
        policy.shared_reads_enabled,
        policy.shared_writes_enabled,
    );
    if actual != expected {
        return Err(RustCacheError::new("rustc-wrapper-policy-cache-mode-inconsistent".to_string()));
    }
    assert_eq!(actual, expected);
    if policy.shared_writes_enabled {
        assert!(policy.shared_reads_enabled);
    }
    Ok(())
}

fn validate_argument_list(arguments: &[String]) -> Result<(), RustCacheError> {
    if arguments.is_empty() || arguments.len() > MAX_WRAPPER_ARGUMENTS {
        return Err(RustCacheError::new("rustc-wrapper-argument-count-invalid".to_string()));
    }
    for argument in arguments {
        validate_string(argument, ValidationCode("rustc-wrapper-argument-invalid"))?;
    }
    assert!(!arguments.is_empty());
    assert!(arguments.len() <= MAX_WRAPPER_ARGUMENTS);
    Ok(())
}

fn validate_environment(environment: &BTreeMap<String, String>) -> Result<(), RustCacheError> {
    if environment.len() > MAX_WRAPPER_ENVIRONMENT {
        return Err(RustCacheError::new("rustc-wrapper-environment-count-invalid".to_string()));
    }
    for (name, value) in environment {
        validate_identifier(name, ValidationCode("rustc-wrapper-environment-name-invalid"))?;
        validate_string(value, ValidationCode("rustc-wrapper-environment-value-invalid"))?;
    }
    assert!(environment.len() <= MAX_WRAPPER_ENVIRONMENT);
    assert!(environment.keys().all(|name| !name.is_empty()));
    Ok(())
}

fn validate_receipt_input(input: &WrapperReceiptInput) -> Result<(), RustCacheError> {
    validate_typed_ref(&input.request_ref, TypedRefRule {
        prefix: WRAPPER_REQUEST_REF_PREFIX,
        code: ValidationCode("rustc-wrapper-receipt-request-invalid"),
    })?;
    validate_identifier(&input.policy_id, ValidationCode("rustc-wrapper-receipt-policy-invalid"))?;
    validate_reason_codes(&input.reason_codes)?;
    if let Some(manifest_ref) = &input.manifest_ref {
        validate_typed_ref(manifest_ref, TypedRefRule {
            prefix: WRAPPER_MANIFEST_REF_PREFIX,
            code: ValidationCode("rustc-wrapper-receipt-manifest-invalid"),
        })?;
    }
    if let Some(action_ref) = &input.action_ref {
        validate_typed_ref(action_ref, TypedRefRule {
            prefix: crate::RUST_ACTION_REF_PREFIX,
            code: ValidationCode("rustc-wrapper-receipt-action-invalid"),
        })?;
    }
    if let Some(result_ref) = &input.result_ref {
        validate_typed_ref(result_ref, TypedRefRule {
            prefix: crate::RUST_RESULT_REF_PREFIX,
            code: ValidationCode("rustc-wrapper-receipt-result-invalid"),
        })?;
    }
    if input.artifact_digests_blake3.len() > MAX_WRAPPER_OUTPUTS {
        return Err(RustCacheError::new("rustc-wrapper-receipt-artifact-count-invalid".to_string()));
    }
    for digest in &input.artifact_digests_blake3 {
        validate_blake3(digest, ValidationCode("rustc-wrapper-receipt-artifact-digest-invalid"))?;
    }
    let is_bypass = input.disposition == WrapperDisposition::Bypass;
    let is_rejected = input.disposition == WrapperDisposition::Rejected;
    if input.bypass_class.is_some() != (is_bypass || is_rejected) {
        return Err(RustCacheError::new("rustc-wrapper-receipt-bypass-class-invalid".to_string()));
    }
    let is_cache_hit = matches!(input.disposition, WrapperDisposition::LocalHit | WrapperDisposition::SharedHit);
    if is_cache_hit && input.result_ref.is_none() {
        return Err(RustCacheError::new("rustc-wrapper-receipt-result-missing".to_string()));
    }
    if input.compiler_status != 0 && input.artifact_commit_complete {
        return Err(RustCacheError::new("rustc-wrapper-receipt-failed-artifact-commit".to_string()));
    }
    assert!(input.artifact_digests_blake3.len() <= MAX_WRAPPER_OUTPUTS);
    assert!(!input.policy_id.is_empty());
    Ok(())
}

fn manifest_static_matches_request(manifest: &WrapperInvocationManifest, request: &WrapperRequest) -> bool {
    let arguments = arguments_identity_blake3(&request.input.arguments).ok();
    let environment = environment_identity_blake3(&request.input.admitted_environment).ok();
    let is_match = request.input.expected_manifest_ref.as_deref() == Some(manifest.manifest_ref.as_str())
        && manifest.input.real_compiler_path == request.input.real_compiler_path
        && manifest.input.working_directory == request.input.working_directory
        && arguments.as_deref() == Some(manifest.input.arguments_blake3.as_str())
        && environment.as_deref() == Some(manifest.input.environment_blake3.as_str())
        && manifest.input.output_contracts == request.input.output_contracts;
    assert_eq!(arguments.is_some(), validate_argument_list(&request.input.arguments).is_ok());
    assert_eq!(environment.is_some(), validate_environment(&request.input.admitted_environment).is_ok());
    is_match
}

fn validate_absolute_roots(roots: &[String], code: &str) -> Result<(), RustCacheError> {
    if roots.is_empty() || roots.len() > MAX_WRAPPER_ROOTS || roots != sorted_unique(roots.to_vec()) {
        return Err(RustCacheError::new(code.to_string()));
    }
    for root in roots {
        validate_absolute_path(root, ValidationCode(code))?;
    }
    assert!(!roots.is_empty());
    assert!(roots.len() <= MAX_WRAPPER_ROOTS);
    Ok(())
}

fn validate_sorted_uids(uids: &[u32]) -> Result<(), RustCacheError> {
    if uids.is_empty() || uids.len() > MAX_WRAPPER_PEERS || uids != sorted_unique(uids.to_vec()) {
        return Err(RustCacheError::new("rustc-wrapper-policy-peer-uids-invalid".to_string()));
    }
    assert!(!uids.is_empty());
    assert!(uids.len() <= MAX_WRAPPER_PEERS);
    Ok(())
}

fn validate_sorted_strings(values: &[String], limit: usize, code: &str) -> Result<(), RustCacheError> {
    if values.len() > limit || values != sorted_unique(values.to_vec()) {
        return Err(RustCacheError::new(code.to_string()));
    }
    for value in values {
        validate_identifier(value, ValidationCode(code))?;
    }
    assert!(values.len() <= limit);
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

fn validate_reason_codes(reasons: &[String]) -> Result<(), RustCacheError> {
    if reasons.len() > MAX_WRAPPER_REASON_CODES || reasons != sorted_unique(reasons.to_vec()) {
        return Err(RustCacheError::new("rustc-wrapper-reason-codes-invalid".to_string()));
    }
    for reason in reasons {
        validate_identifier(reason, ValidationCode("rustc-wrapper-reason-code-invalid"))?;
    }
    assert!(reasons.len() <= MAX_WRAPPER_REASON_CODES);
    assert!(reasons.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

fn validate_absolute_path(path: &str, code: ValidationCode<'_>) -> Result<(), RustCacheError> {
    if path.is_empty() {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    if path.len() > MAX_WRAPPER_PATH_BYTES {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    if !path.starts_with('/') {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    if path.contains('\0') {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    if path.split('/').skip(1).any(|component| component.is_empty() || matches!(component, "." | "..")) {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    assert!(path.starts_with('/'));
    assert!(path.len() <= MAX_WRAPPER_PATH_BYTES);
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<(), RustCacheError> {
    if path.is_empty() {
        return Err(RustCacheError::new("rustc-wrapper-output-staging-path-invalid".to_string()));
    }
    if path.len() > MAX_WRAPPER_PATH_BYTES {
        return Err(RustCacheError::new("rustc-wrapper-output-staging-path-invalid".to_string()));
    }
    if path.starts_with('/') {
        return Err(RustCacheError::new("rustc-wrapper-output-staging-path-invalid".to_string()));
    }
    if path.contains('\0') {
        return Err(RustCacheError::new("rustc-wrapper-output-staging-path-invalid".to_string()));
    }
    if path.split('/').any(|component| component.is_empty() || matches!(component, "." | "..")) {
        return Err(RustCacheError::new("rustc-wrapper-output-staging-path-invalid".to_string()));
    }
    assert!(!path.starts_with('/'));
    assert!(path.len() <= MAX_WRAPPER_PATH_BYTES);
    Ok(())
}

fn path_is_within_any_root(path: &str, roots: &[String]) -> bool {
    let is_within = roots
        .iter()
        .any(|root| path == root || path.strip_prefix(root).is_some_and(|rest| rest.starts_with('/')));
    assert!(!path.is_empty());
    assert!(!roots.is_empty());
    is_within
}

fn validate_identifier(value: &str, code: ValidationCode<'_>) -> Result<(), RustCacheError> {
    validate_string(value, code)?;
    if value.bytes().any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace()) {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    assert!(!value.is_empty());
    assert!(value.len() <= MAX_WRAPPER_STRING_BYTES);
    Ok(())
}

fn validate_string(value: &str, code: ValidationCode<'_>) -> Result<(), RustCacheError> {
    if value.is_empty() {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    if value.len() > MAX_WRAPPER_STRING_BYTES {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    if value.contains('\0') {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    assert!(!value.is_empty());
    assert!(value.len() <= MAX_WRAPPER_STRING_BYTES);
    Ok(())
}

fn validate_blake3(value: &str, code: ValidationCode<'_>) -> Result<(), RustCacheError> {
    let is_valid = value.len() == BLAKE3_HEX_CHARS
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if !is_valid {
        return Err(RustCacheError::new(code.0.to_string()));
    }
    assert_eq!(value.len(), BLAKE3_HEX_CHARS);
    assert!(value.bytes().all(|byte| !byte.is_ascii_uppercase()));
    Ok(())
}

fn validate_typed_ref(value: &str, rule: TypedRefRule<'_>) -> Result<(), RustCacheError> {
    let Some(digest) = value.strip_prefix(rule.prefix) else {
        return Err(RustCacheError::new(rule.code.0.to_string()));
    };
    validate_blake3(digest, rule.code)?;
    assert!(value.starts_with(rule.prefix));
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    Ok(())
}

fn ensure_record_bound(value: &impl Serialize, limit_bytes: u64, code: &str) -> Result<(), RustCacheError> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| RustCacheError::new(format!("rustc-wrapper-json:{error}")))?;
    let size_bytes = u64::try_from(bytes.len()).map_err(|_| RustCacheError::new(code))?;
    if size_bytes > limit_bytes {
        return Err(RustCacheError::new(code.to_string()));
    }
    assert!(!bytes.is_empty());
    assert!(size_bytes <= limit_bytes);
    Ok(())
}

fn digest_ref(prefix: &str, domain: &[u8], value: &impl Serialize) -> Result<String, RustCacheError> {
    let digest = canonical_digest(domain, value)?;
    let reference = format!("{prefix}{digest}");
    assert!(reference.starts_with(prefix));
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    Ok(reference)
}

fn canonical_digest(domain: &[u8], value: &(impl Serialize + ?Sized)) -> Result<String, RustCacheError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| RustCacheError::new(format!("rustc-wrapper-canonical-json:{error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert!(digest.bytes().all(|byte| !byte.is_ascii_uppercase()));
    Ok(digest)
}

fn sorted_unique<T: Ord>(values: Vec<T>) -> Vec<T> {
    let values = values.into_iter().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(values.len() <= values.capacity());
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RustUnitActionInput;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TEST_UID: u32 = 1_000;

    #[test]
    fn manifest_and_request_identities_are_canonical() {
        let manifest = manifest();
        let request = request(&manifest);
        let mut reordered = manifest.input.clone();
        reordered.readable_roots.reverse();
        reordered.declared_inputs.reverse();
        reordered.output_contracts.reverse();

        let resealed = seal_wrapper_manifest(reordered).unwrap();

        assert_eq!(manifest.manifest_ref, resealed.manifest_ref);
        assert!(validate_wrapper_request(&request).is_ok());
        assert!(manifest.manifest_ref.starts_with(WRAPPER_MANIFEST_REF_PREFIX));
        assert!(request.request_ref.starts_with(WRAPPER_REQUEST_REF_PREFIX));
    }

    #[test]
    fn every_static_request_binding_changes_identity_or_blocks_match() {
        let manifest = manifest();
        let request = request(&manifest);
        let policy = policy(WrapperFailureMode::FailOpen);
        let facts = valid_facts();
        assert_eq!(
            classify_wrapper_request(&request, Some(&manifest), &policy, Some(&facts)),
            WrapperDecision::StrongEligible
        );

        let mut changed = request.input.clone();
        changed.arguments.push("--cfg=changed".to_string());
        let changed = seal_wrapper_request(changed).unwrap();

        assert_ne!(request.request_ref, changed.request_ref);
        assert_eq!(
            classify_wrapper_request(&changed, Some(&manifest), &policy, Some(&facts)),
            WrapperDecision::Bypass(WrapperBypassClass::ManifestIdentityMismatch)
        );
    }

    #[test]
    fn all_named_bypass_classes_are_deterministic() {
        let manifest = manifest();
        let policy = policy(WrapperFailureMode::FailOpen);
        let facts = valid_facts();
        let cases = [
            (vec!["--version".to_string()], WrapperBypassClass::CompilerQuery),
            (vec!["-Cincremental=/tmp/i".to_string()], WrapperBypassClass::IncrementalCompilation),
            (vec!["@args".to_string()], WrapperBypassClass::ResponseFile),
        ];
        for (arguments, expected) in cases {
            let mut input = request(&manifest).input;
            input.arguments = arguments;
            input.expected_manifest_ref = None;
            input.manifest_path = None;
            let request = seal_wrapper_request(input).unwrap();
            assert_eq!(
                classify_wrapper_request(&request, Some(&manifest), &policy, Some(&facts)),
                WrapperDecision::Bypass(expected)
            );
        }
        let missing = request_without_manifest();
        assert_eq!(
            classify_wrapper_request(&missing, None, &policy, None),
            WrapperDecision::Bypass(WrapperBypassClass::MissingManifest)
        );
        assert_eq!(cases_len(), 3);
    }

    #[test]
    fn fail_closed_rejects_missing_or_changed_evidence() {
        let manifest = manifest();
        let request = request(&manifest);
        let policy = policy(WrapperFailureMode::FailClosed);
        let mut facts = valid_facts();
        facts.inputs_verified = false;

        assert_eq!(
            classify_wrapper_request(&request, Some(&manifest), &policy, Some(&facts)),
            WrapperDecision::Reject(WrapperBypassClass::InputChanged)
        );
        assert_eq!(
            classify_wrapper_request(&request_without_manifest(), None, &policy, None),
            WrapperDecision::Reject(WrapperBypassClass::MissingManifest)
        );
    }

    #[test]
    fn unsafe_outputs_unsupported_effects_and_bad_limits_fail() {
        let mut input = manifest().input;
        input.output_contracts[0].destination_path = "/outside/artifact".to_string();
        input.effects.clock = AmbientEffect::Deny;
        input.limits.elapsed_millis = 0;

        let error = seal_wrapper_manifest(input).unwrap_err();

        assert!(matches!(
            error.code(),
            "rustc-wrapper-output-destination-out-of-root"
                | "rustc-wrapper-effect-policy-unsupported"
                | "rustc-wrapper-elapsed-limit-invalid"
        ));
        assert!(!error.code().is_empty());
    }

    #[test]
    fn duplicate_and_missing_manifest_facts_fail() {
        let mut input = manifest().input;
        input.declared_inputs.retain(|item| item.role != "sysroot");
        input.output_contracts.push(input.output_contracts[0].clone());

        let error = seal_wrapper_manifest(input).unwrap_err();

        assert!(matches!(
            error.code(),
            "rustc-wrapper-manifest-required-input-missing" | "rustc-wrapper-outputs-not-canonical"
        ));
        assert!(!error.code().is_empty());
    }

    #[test]
    fn manifest_rejects_action_and_declared_input_misbinding() {
        let mut compiler_mismatch = manifest().input;
        compiler_mismatch.real_compiler_path = "/toolchain/bin/other-rustc".to_string();
        assert_eq!(
            seal_wrapper_manifest(compiler_mismatch),
            Err(RustCacheError::new("rustc-wrapper-manifest-compiler-binding-invalid"))
        );

        let mut source_mismatch = manifest().input;
        source_mismatch
            .declared_inputs
            .iter_mut()
            .find(|input| input.role == "source")
            .unwrap()
            .digest_blake3 = DIGEST_B.to_string();
        assert_eq!(
            seal_wrapper_manifest(source_mismatch),
            Err(RustCacheError::new("rustc-wrapper-manifest-source-binding-invalid"))
        );

        let mut environment_mismatch = manifest().input;
        environment_mismatch.environment_blake3 = environment_identity_blake3(&BTreeMap::new()).unwrap();
        assert_eq!(
            seal_wrapper_manifest(environment_mismatch),
            Err(RustCacheError::new("rustc-wrapper-manifest-environment-binding-invalid"))
        );

        let mut duplicate_role = manifest().input;
        duplicate_role.declared_inputs.push(DeclaredInput {
            role: "source".to_string(),
            path: "/source/other.rs".to_string(),
            kind: DeclaredInputKind::File,
            digest_blake3: DIGEST_A.to_string(),
        });
        assert_eq!(
            seal_wrapper_manifest(duplicate_role),
            Err(RustCacheError::new("rustc-wrapper-manifest-input-role-duplicate"))
        );
    }

    #[test]
    fn response_and_receipt_cross_fields_fail_closed() {
        let manifest = manifest();
        let request = request(&manifest);
        let response = WrapperResponse {
            schema: WRAPPER_RESPONSE_SCHEMA.to_string(),
            request_ref: request.request_ref.clone(),
            disposition: WrapperDisposition::LocalHit,
            bypass_class: Some(WrapperBypassClass::ProtocolFailure),
            reason_codes: vec!["invalid".to_string()],
            compiler_status: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
            artifact_commit_complete: true,
            receipt_ref: None,
        };

        assert_eq!(
            validate_wrapper_response(&response, MAX_WRAPPER_RESPONSE_BYTES),
            Err(RustCacheError::new("rustc-wrapper-response-bypass-class-invalid"))
        );
        let receipt_input = WrapperReceiptInput {
            request_ref: request.request_ref,
            manifest_ref: Some(manifest.manifest_ref),
            policy_id: "policy-v1".to_string(),
            disposition: WrapperDisposition::LocalHit,
            bypass_class: None,
            action_ref: None,
            result_ref: None,
            artifact_digests_blake3: vec![DIGEST_A.to_string()],
            compiler_status: 0,
            compiler_executed: false,
            artifact_commit_complete: true,
            reason_codes: vec!["compiled".to_string()],
        };
        assert_eq!(
            seal_wrapper_receipt(receipt_input),
            Err(RustCacheError::new("rustc-wrapper-receipt-result-missing"))
        );
    }

    fn cases_len() -> usize {
        3
    }

    fn manifest() -> WrapperInvocationManifest {
        let arguments = vec!["--crate-name".to_string(), "fixture".to_string()];
        let environment = BTreeMap::from([("PATH".to_string(), "/toolchain/bin".to_string())]);
        seal_wrapper_manifest(WrapperInvocationManifestInput {
            policy_id: "policy-v1".to_string(),
            action: action(),
            real_compiler_path: "/toolchain/bin/rustc".to_string(),
            arguments_blake3: arguments_identity_blake3(&arguments).unwrap(),
            environment_blake3: environment_identity_blake3(&environment).unwrap(),
            working_directory: "/work".to_string(),
            readable_roots: vec!["/source".to_string(), "/toolchain".to_string()],
            writable_roots: vec!["/work/target".to_string()],
            declared_inputs: vec![
                DeclaredInput {
                    role: "compiler".to_string(),
                    path: "/toolchain/bin/rustc".to_string(),
                    kind: DeclaredInputKind::File,
                    digest_blake3: DIGEST_A.to_string(),
                },
                DeclaredInput {
                    role: "source".to_string(),
                    path: "/source/lib.rs".to_string(),
                    kind: DeclaredInputKind::File,
                    digest_blake3: DIGEST_A.to_string(),
                },
                DeclaredInput {
                    role: "sysroot".to_string(),
                    path: "/toolchain/lib/rustlib".to_string(),
                    kind: DeclaredInputKind::Directory,
                    digest_blake3: DIGEST_B.to_string(),
                },
            ],
            output_contracts: outputs(),
            effects: effects(),
            limits: limits(),
        })
        .unwrap()
    }

    fn request(manifest: &WrapperInvocationManifest) -> WrapperRequest {
        seal_wrapper_request(WrapperRequestInput {
            manifest_path: Some("/work/manifests/unit.json".to_string()),
            expected_manifest_ref: Some(manifest.manifest_ref.clone()),
            real_compiler_path: "/toolchain/bin/rustc".to_string(),
            arguments: vec!["--crate-name".to_string(), "fixture".to_string()],
            admitted_environment: BTreeMap::from([("PATH".to_string(), "/toolchain/bin".to_string())]),
            working_directory: "/work".to_string(),
            output_contracts: outputs(),
        })
        .unwrap()
    }

    fn request_without_manifest() -> WrapperRequest {
        let mut input = request(&manifest()).input;
        input.manifest_path = None;
        input.expected_manifest_ref = None;
        seal_wrapper_request(input).unwrap()
    }

    fn policy(failure_mode: WrapperFailureMode) -> WrapperDaemonPolicy {
        let policy = WrapperDaemonPolicy {
            schema: WRAPPER_POLICY_SCHEMA.to_string(),
            policy_id: "policy-v1".to_string(),
            cache_mode: WrapperCacheMode::LocalReadWrite,
            failure_mode,
            socket_path: "/run/user/1000/mantle-rust-cache.sock".to_string(),
            allowed_peer_uids: vec![TEST_UID],
            readable_roots: vec!["/source".to_string(), "/toolchain".to_string()],
            writable_roots: vec!["/work/target".to_string()],
            allowed_environment: vec!["PATH".to_string()],
            effects: effects(),
            limits: limits(),
            max_request_bytes: MAX_WRAPPER_REQUEST_BYTES,
            max_response_bytes: MAX_WRAPPER_RESPONSE_BYTES,
            max_concurrency: 4,
            sandbox_program: "/usr/bin/bwrap".to_string(),
            sandbox_program_blake3: DIGEST_A.to_string(),
            result_sources: Vec::new(),
            shared_trust_policy: None,
            publication: WrapperPublicationPolicy {
                enabled: false,
                source_id: None,
                signer_name: None,
                signing_key_path: None,
            },
            local_reads_enabled: true,
            local_writes_enabled: true,
            shared_reads_enabled: false,
            shared_writes_enabled: false,
            redact_environment_values: true,
        };
        validate_wrapper_policy(&policy).unwrap();
        policy
    }

    fn valid_facts() -> CurrentManifestFacts {
        CurrentManifestFacts {
            manifest_ref_matches: true,
            compiler_matches: true,
            arguments_match: true,
            environment_matches: true,
            working_directory_matches: true,
            inputs_verified: true,
            outputs_admissible: true,
            enforcement_available: true,
            failure_class: None,
        }
    }

    fn outputs() -> Vec<WrapperOutputContract> {
        vec![WrapperOutputContract {
            staged_relative_path: "libfixture.rlib".to_string(),
            destination_path: "/work/target/libfixture.rlib".to_string(),
            kind: WrapperArtifactKind::File,
            max_bytes: 1_048_576,
            executable: false,
        }]
    }

    fn effects() -> WrapperEffectPolicy {
        WrapperEffectPolicy {
            filesystem: FilesystemEffect::DeclaredRoots,
            network: NetworkEffect::Deny,
            clock: AmbientEffect::Allow,
            randomness: AmbientEffect::Allow,
        }
    }

    fn limits() -> WrapperResourceLimits {
        WrapperResourceLimits {
            elapsed_millis: 30_000,
            stdout_bytes: 65_536,
            stderr_bytes: 65_536,
            artifact_bytes: 1_048_576,
        }
    }

    fn action() -> RustUnitAction {
        crate::canonical_rust_action(RustUnitActionInput {
            unit_id: "fixture".to_string(),
            package_id: "fixture-package".to_string(),
            crate_name: "fixture".to_string(),
            target_kind: "lib".to_string(),
            execution_kind: "target".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            profile: "debug".to_string(),
            mode: "build".to_string(),
            features: Vec::new(),
            source_digest_blake3: DIGEST_A.to_string(),
            compiler_digest_blake3: DIGEST_A.to_string(),
            compiler_version_digest_blake3: DIGEST_A.to_string(),
            toolchain_closure_digest_blake3: DIGEST_B.to_string(),
            execution_platform_digest_blake3: DIGEST_A.to_string(),
            semantic_arguments: Vec::new(),
            admitted_environment: BTreeMap::from([("PATH".to_string(), "/toolchain/bin".to_string())]),
            dependency_artifacts: Vec::new(),
            host_artifacts: Vec::new(),
            build_script_facts: Vec::new(),
            native_link_facts: Vec::new(),
            compiler_policy_digest_blake3: DIGEST_A.to_string(),
        })
        .unwrap()
    }
}
