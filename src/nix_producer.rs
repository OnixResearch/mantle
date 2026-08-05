// r[impl nix_producer_adapter.backend_contract]
// r[impl nix_producer_adapter.backend_selection]
//
// Pure `nix-producer-v1` contract core. Every Nix producer backend (`fix`,
// `host-nix`, and future registered kinds) satisfies this contract. The core
// validates requests, selects backends, classifies observed outcomes, and
// builds receipt inputs over in-memory data. It performs no I/O, no process
// launch, no clock access, and no environment inspection; backend shells own
// those effects.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const NIX_PRODUCER_CONTRACT_SCHEMA: &str = "nix-producer-v1";

const MAX_EXPRESSION_TEXT_BYTES: usize = 1_048_576;
const MAX_EXPRESSION_PATH_CHARS: usize = 4_096;
const MAX_ATTRIBUTE_PATH_CHARS: usize = 1_024;
const MAX_EVAL_ARGS: usize = 64;
const MAX_EVAL_ARG_KEY_CHARS: usize = 128;
const MAX_EVAL_ARG_VALUE_CHARS: usize = 4_096;
const MAX_SYSTEM_LABEL_CHARS: usize = 64;
const MAX_VERSION_CHARS: usize = 128;
const MAX_BINARY_IDENTITY_CHARS: usize = 256;
const MAX_ROOT_DRV_PATH_CHARS: usize = 4_096;
const MAX_DRV_DIR_ENTRY_COUNT: u32 = 65_536;
const MAX_DRV_DIR_TOTAL_BYTES: u64 = 1_073_741_824;
const BLAKE3_HEX_CHARS: usize = 64;

const ERROR_UNKNOWN_BACKEND: &str = "unknown-backend";
const ERROR_BACKEND_UNAVAILABLE: &str = "backend-unavailable";
const ERROR_UNSUPPORTED_PLATFORM: &str = "unsupported-platform";
const ERROR_CONTRACT_VIOLATION: &str = "contract-violation";
const ERROR_INVALID_REQUEST: &str = "invalid-request";
const ERROR_DAEMON_COMMAND_REJECTED: &str = "daemon-command-rejected";
const ERROR_EVALUATION_FAILED: &str = "evaluation-failed";
const ERROR_MALFORMED_OUTPUT: &str = "malformed-output";
const ERROR_OUTPUT_TOO_LARGE: &str = "output-too-large";
const ERROR_BUDGET_TIMEOUT: &str = "budget-timeout";
const ERROR_HASH_DOMAIN_MISMATCH: &str = "hash-domain-mismatch";

/// Registered Nix producer backend kinds. New evaluators join the contract
/// here instead of forking the producer pipeline.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProducerBackendKind {
    /// Pinned, Mantle-built psyclyx/fix evaluator.
    Fix,
    /// Ambient host Nix installation. Not reproducible by Mantle.
    HostNix,
}

impl ProducerBackendKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Fix => "fix",
            Self::HostNix => "host-nix",
        }
    }

    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "fix" => Some(Self::Fix),
            "host-nix" => Some(Self::HostNix),
            _ => None,
        }
    }
}

/// Trust posture recorded on receipts. The posture is a fact about how the
/// backend binary was obtained, not a claim about evaluator correctness.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BackendTrustPosture {
    /// Backend binary is a Mantle build product from a pinned source record.
    PinnedMantleBuilt,
    /// Backend binary comes from the ambient host environment.
    AmbientHost,
}

impl ProducerBackendKind {
    /// The only trust posture a backend kind may claim. A `fix` backend that
    /// cannot prove a Mantle-built binary must not run as `fix`.
    pub(crate) fn required_trust_posture(self) -> BackendTrustPosture {
        match self {
            Self::Fix => BackendTrustPosture::PinnedMantleBuilt,
            Self::HostNix => BackendTrustPosture::AmbientHost,
        }
    }
}

/// One evaluation target for a producer run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProducerTarget {
    /// Evaluate a Nix file, optionally at an attribute path.
    File {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attribute: Option<String>,
    },
    /// Evaluate inline expression text, optionally at an attribute path.
    Expr {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attribute: Option<String>,
    },
}

/// Named resource bounds for one producer run. Every limit is explicit; the
/// shell enforces what the host supports and fails closed otherwise.
///
/// `memory_bytes_max` is an address-space (`RLIMIT_AS`) bound. A value of 0
/// disables it. Arena-reserving backends (the fix parallel GC reserves tens
/// of GiB of virtual space) cannot run under a tight `RLIMIT_AS`, so honest
/// memory enforcement for them needs a cgroup RSS mechanism that this change
/// does not provide. A disabled limit must never be reported as enforcement.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProducerBudget {
    pub(crate) wall_time_ms_max: u64,
    /// Address-space bound; 0 disables process-level memory limiting.
    pub(crate) memory_bytes_max: u64,
    pub(crate) output_bytes_max: u64,
    pub(crate) drv_file_count_max: u32,
}

impl Default for ProducerBudget {
    fn default() -> Self {
        Self {
            wall_time_ms_max: 300_000,
            // Disabled by default: RLIMIT_AS breaks arena-reserving backends.
            memory_bytes_max: 0,
            output_bytes_max: MAX_DRV_DIR_TOTAL_BYTES,
            drv_file_count_max: MAX_DRV_DIR_ENTRY_COUNT,
        }
    }
}

/// A validated producer evaluation request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct NixProducerRequest {
    pub(crate) schema: String,
    pub(crate) backend: ProducerBackendKind,
    pub(crate) target: ProducerTarget,
    #[serde(default)]
    pub(crate) eval_args: BTreeMap<String, String>,
    pub(crate) system: String,
    #[serde(default)]
    pub(crate) budget: ProducerBudget,
}

/// Stable rejection or failure classes. The strings are the wire format;
/// shells and tests must use the enum, not fresh strings.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProducerErrorClass {
    UnknownBackend,
    BackendUnavailable,
    UnsupportedPlatform,
    ContractViolation,
    InvalidRequest,
    DaemonCommandRejected,
    EvaluationFailed,
    MalformedOutput,
    OutputTooLarge,
    BudgetTimeout,
    HashDomainMismatch,
}

impl ProducerErrorClass {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::UnknownBackend => ERROR_UNKNOWN_BACKEND,
            Self::BackendUnavailable => ERROR_BACKEND_UNAVAILABLE,
            Self::UnsupportedPlatform => ERROR_UNSUPPORTED_PLATFORM,
            Self::ContractViolation => ERROR_CONTRACT_VIOLATION,
            Self::InvalidRequest => ERROR_INVALID_REQUEST,
            Self::DaemonCommandRejected => ERROR_DAEMON_COMMAND_REJECTED,
            Self::EvaluationFailed => ERROR_EVALUATION_FAILED,
            Self::MalformedOutput => ERROR_MALFORMED_OUTPUT,
            Self::OutputTooLarge => ERROR_OUTPUT_TOO_LARGE,
            Self::BudgetTimeout => ERROR_BUDGET_TIMEOUT,
            Self::HashDomainMismatch => ERROR_HASH_DOMAIN_MISMATCH,
        }
    }
}

impl fmt::Display for ProducerErrorClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A typed producer rejection or failure with a stable class.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProducerError {
    pub(crate) class: ProducerErrorClass,
    pub(crate) detail: String,
}

impl ProducerError {
    fn new(class: ProducerErrorClass, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        debug_assert!(!detail.is_empty(), "producer error detail must not be empty");
        Self { class, detail }
    }
}

impl fmt::Display for ProducerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.class, self.detail)
    }
}

/// Facts a backend shell reports about the binary that actually ran.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProducerIdentityFacts {
    pub(crate) backend: ProducerBackendKind,
    pub(crate) backend_version: String,
    /// BLAKE3 hex digest of the backend binary for pinned backends, or the
    /// observed host binary path for ambient backends.
    pub(crate) binary_identity: String,
    pub(crate) trust_posture: BackendTrustPosture,
    /// BLAKE3 hex digest over the canonical evaluation-argument facts.
    pub(crate) eval_args_digest_blake3: String,
}

/// A successful producer outcome: a materialized `.drv` closure directory.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProducerSuccess {
    pub(crate) schema: String,
    pub(crate) drv_dir: String,
    pub(crate) root_drv_path: String,
    pub(crate) drv_file_count: u32,
    pub(crate) drv_dir_total_bytes: u64,
    pub(crate) identity: ProducerIdentityFacts,
}

/// Terminal outcome of one producer run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProducerOutcome {
    Success(ProducerSuccess),
    Failure(ProducerError),
}

/// One registered backend entry in adapter policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BackendRegistration {
    pub(crate) kind: ProducerBackendKind,
    /// Declared binary identity: a BLAKE3 hex digest for pinned backends or
    /// an absolute host path for ambient backends.
    pub(crate) binary_identity: String,
    /// Systems the backend may serve. Empty means none.
    pub(crate) supported_systems: BTreeSet<String>,
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_common_chars(field: &str, value: &str, max_chars: usize) -> Result<(), ProducerError> {
    if value.is_empty() {
        return Err(ProducerError::new(ProducerErrorClass::InvalidRequest, format!("{field} must not be empty")));
    }
    if value.chars().count() > max_chars {
        return Err(ProducerError::new(
            ProducerErrorClass::InvalidRequest,
            format!("{field} exceeds {max_chars} chars"),
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(ProducerError::new(
            ProducerErrorClass::InvalidRequest,
            format!("{field} contains control characters"),
        ));
    }
    Ok(())
}

fn validate_attribute(attribute: &str) -> Result<(), ProducerError> {
    validate_common_chars("attribute", attribute, MAX_ATTRIBUTE_PATH_CHARS)
}

/// Validate one producer request against the contract bounds.
pub(crate) fn validate_request(request: &NixProducerRequest) -> Result<(), ProducerError> {
    if request.schema != NIX_PRODUCER_CONTRACT_SCHEMA {
        return Err(ProducerError::new(
            ProducerErrorClass::InvalidRequest,
            format!("schema must be {NIX_PRODUCER_CONTRACT_SCHEMA}, got {}", request.schema),
        ));
    }
    validate_common_chars("system", &request.system, MAX_SYSTEM_LABEL_CHARS)?;
    match &request.target {
        ProducerTarget::File { path, attribute } => {
            validate_common_chars("path", path, MAX_EXPRESSION_PATH_CHARS)?;
            if !path.starts_with('/') {
                return Err(ProducerError::new(ProducerErrorClass::InvalidRequest, "path must be absolute"));
            }
            if let Some(attribute) = attribute {
                validate_attribute(attribute)?;
            }
        }
        ProducerTarget::Expr { text, attribute } => {
            if text.is_empty() {
                return Err(ProducerError::new(
                    ProducerErrorClass::InvalidRequest,
                    "expression text must not be empty",
                ));
            }
            if text.len() > MAX_EXPRESSION_TEXT_BYTES {
                return Err(ProducerError::new(
                    ProducerErrorClass::InvalidRequest,
                    format!("expression text exceeds {MAX_EXPRESSION_TEXT_BYTES} bytes"),
                ));
            }
            if let Some(attribute) = attribute {
                validate_attribute(attribute)?;
            }
        }
    }
    if request.eval_args.len() > MAX_EVAL_ARGS {
        return Err(ProducerError::new(
            ProducerErrorClass::InvalidRequest,
            format!("eval args exceed {MAX_EVAL_ARGS} entries"),
        ));
    }
    for (key, value) in &request.eval_args {
        validate_common_chars("eval arg key", key, MAX_EVAL_ARG_KEY_CHARS)?;
        validate_common_chars("eval arg value", value, MAX_EVAL_ARG_VALUE_CHARS)?;
    }
    if request.budget.wall_time_ms_max == 0
        || request.budget.output_bytes_max == 0
        || request.budget.drv_file_count_max == 0
    {
        return Err(ProducerError::new(
            ProducerErrorClass::InvalidRequest,
            "wall-time, output, and file-count budgets must be positive",
        ));
    }
    Ok(())
}

/// Select exactly one registered backend. Selection fails closed: unknown
/// kinds, unavailable binaries, and unsupported systems reject, and no
/// ambient fallback ever substitutes another backend.
pub(crate) fn select_backend(
    registrations: &[BackendRegistration],
    requested: ProducerBackendKind,
    system: &str,
) -> Result<BackendRegistration, ProducerError> {
    debug_assert!(!system.is_empty(), "system label must be validated first");
    let matching: Vec<&BackendRegistration> = registrations.iter().filter(|entry| entry.kind == requested).collect();
    if matching.is_empty() {
        return Err(ProducerError::new(
            ProducerErrorClass::UnknownBackend,
            format!("no registered backend for kind {}", requested.as_str()),
        ));
    }
    if matching.len() > 1 {
        return Err(ProducerError::new(
            ProducerErrorClass::ContractViolation,
            format!("duplicate registration for kind {}", requested.as_str()),
        ));
    }
    let entry = matching[0];
    if entry.binary_identity.is_empty() {
        return Err(ProducerError::new(
            ProducerErrorClass::BackendUnavailable,
            format!("backend {} has no binary identity", requested.as_str()),
        ));
    }
    match requested {
        ProducerBackendKind::Fix => {
            if !is_blake3_hex(&entry.binary_identity) {
                return Err(ProducerError::new(
                    ProducerErrorClass::HashDomainMismatch,
                    "fix backend binary identity must be a BLAKE3 hex digest",
                ));
            }
        }
        ProducerBackendKind::HostNix => {
            if !entry.binary_identity.starts_with('/') {
                return Err(ProducerError::new(
                    ProducerErrorClass::BackendUnavailable,
                    "host-nix backend binary identity must be an absolute path",
                ));
            }
        }
    }
    if !entry.supported_systems.contains(system) {
        return Err(ProducerError::new(
            ProducerErrorClass::UnsupportedPlatform,
            format!("backend {} does not support system {system}", requested.as_str()),
        ));
    }
    Ok(entry.clone())
}

/// Compute the canonical BLAKE3 digest over evaluation-argument facts.
pub(crate) fn eval_args_digest(eval_args: &BTreeMap<String, String>, system: &str) -> String {
    debug_assert!(!system.is_empty(), "system label must be validated first");
    let mut canonical = String::new();
    canonical.push_str("nix-producer-eval-args-v1\n");
    canonical.push_str("system=");
    canonical.push_str(system);
    canonical.push('\n');
    for (key, value) in eval_args {
        canonical.push_str(key);
        canonical.push('=');
        canonical.push_str(value);
        canonical.push('\n');
    }
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}

/// Validate identity facts reported by a backend shell against the contract.
pub(crate) fn validate_identity_facts(
    facts: &ProducerIdentityFacts,
    request: &NixProducerRequest,
) -> Result<(), ProducerError> {
    if facts.backend != request.backend {
        return Err(ProducerError::new(
            ProducerErrorClass::ContractViolation,
            format!(
                "identity backend {} does not match requested {}",
                facts.backend.as_str(),
                request.backend.as_str()
            ),
        ));
    }
    if facts.trust_posture != request.backend.required_trust_posture() {
        return Err(ProducerError::new(
            ProducerErrorClass::ContractViolation,
            format!("backend {} may not claim trust posture {:?}", facts.backend.as_str(), facts.trust_posture),
        ));
    }
    validate_common_chars("backend version", &facts.backend_version, MAX_VERSION_CHARS)?;
    validate_common_chars("binary identity", &facts.binary_identity, MAX_BINARY_IDENTITY_CHARS)?;
    let expected_digest = eval_args_digest(&request.eval_args, &request.system);
    if facts.eval_args_digest_blake3 != expected_digest {
        return Err(ProducerError::new(
            ProducerErrorClass::ContractViolation,
            "eval args digest does not match the request",
        ));
    }
    Ok(())
}

/// Validate a reported success observation against contract bounds and the
/// originating request before the adapter accepts it.
pub(crate) fn accept_success(
    success: &ProducerSuccess,
    request: &NixProducerRequest,
) -> Result<ProducerOutcome, ProducerError> {
    if success.schema != NIX_PRODUCER_CONTRACT_SCHEMA {
        return Err(ProducerError::new(
            ProducerErrorClass::ContractViolation,
            format!("success schema must be {NIX_PRODUCER_CONTRACT_SCHEMA}, got {}", success.schema),
        ));
    }
    validate_identity_facts(&success.identity, request)?;
    validate_common_chars("drv dir", &success.drv_dir, MAX_EXPRESSION_PATH_CHARS)?;
    validate_common_chars("root drv path", &success.root_drv_path, MAX_ROOT_DRV_PATH_CHARS)?;
    if !success.root_drv_path.ends_with(".drv") {
        return Err(ProducerError::new(ProducerErrorClass::MalformedOutput, "root derivation path must end in .drv"));
    }
    if success.drv_file_count == 0 {
        return Err(ProducerError::new(
            ProducerErrorClass::MalformedOutput,
            "drv closure must contain at least one file",
        ));
    }
    if success.drv_file_count > request.budget.drv_file_count_max {
        return Err(ProducerError::new(
            ProducerErrorClass::OutputTooLarge,
            format!("drv file count {} exceeds budget {}", success.drv_file_count, request.budget.drv_file_count_max),
        ));
    }
    if success.drv_dir_total_bytes > request.budget.output_bytes_max {
        return Err(ProducerError::new(
            ProducerErrorClass::OutputTooLarge,
            format!("drv dir bytes {} exceed budget {}", success.drv_dir_total_bytes, request.budget.output_bytes_max),
        ));
    }
    Ok(ProducerOutcome::Success(success.clone()))
}

// r[impl nix_producer_adapter.compatibility_evidence]

/// Pins bound into one per-backend compatibility evidence record. Every
/// field is a typed fact; evidence validity extends exactly to these pins.
// Receipt integration lands with the release-evidence wiring; until then only
// unit tests consume this type.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BackendEvidencePins {
    pub(crate) backend: ProducerBackendKind,
    pub(crate) source_revision: String,
    pub(crate) source_content_digest: String,
    pub(crate) binary_identity: String,
    pub(crate) reference_nix_version: String,
    pub(crate) nixpkgs_universe_pin: String,
}

/// Freshness of an evidence record against current backend facts.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceFreshness {
    /// Current facts match every recorded pin.
    Fresh,
    /// Named fields drifted; old counts must not cover the new identity.
    Stale { drifted_fields: Vec<String> },
}

#[allow(dead_code)]
fn drift_field(drifted: &mut Vec<String>, name: &str, recorded: &str, current: &str) {
    debug_assert!(!name.is_empty(), "drift field name must not be empty");
    if recorded != current {
        drifted.push(name.to_string());
    }
}

/// Classify one evidence record against current backend facts. A drifted
/// backend kind makes every count stale for this backend.
#[allow(dead_code)]
pub(crate) fn classify_evidence_freshness(
    recorded: &BackendEvidencePins,
    current: &BackendEvidencePins,
) -> EvidenceFreshness {
    let mut drifted: Vec<String> = Vec::new();
    if recorded.backend != current.backend {
        drifted.push("backend".to_string());
    }
    drift_field(&mut drifted, "source_revision", &recorded.source_revision, &current.source_revision);
    drift_field(
        &mut drifted,
        "source_content_digest",
        &recorded.source_content_digest,
        &current.source_content_digest,
    );
    drift_field(&mut drifted, "binary_identity", &recorded.binary_identity, &current.binary_identity);
    drift_field(
        &mut drifted,
        "reference_nix_version",
        &recorded.reference_nix_version,
        &current.reference_nix_version,
    );
    drift_field(&mut drifted, "nixpkgs_universe_pin", &recorded.nixpkgs_universe_pin, &current.nixpkgs_universe_pin);
    if drifted.is_empty() {
        EvidenceFreshness::Fresh
    } else {
        debug_assert!(!drifted.is_empty(), "stale evidence must name drifted fields");
        EvidenceFreshness::Stale {
            drifted_fields: drifted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[verify nix_producer_adapter.backend_contract]
    // r[verify nix_producer_adapter.backend_selection]

    const DEFAULT_SYSTEM: &str = "x86_64-linux";

    fn valid_request() -> NixProducerRequest {
        NixProducerRequest {
            schema: NIX_PRODUCER_CONTRACT_SCHEMA.to_string(),
            backend: ProducerBackendKind::Fix,
            target: ProducerTarget::Expr {
                text: "{ pkgs ? {} }: pkgs".to_string(),
                attribute: None,
            },
            eval_args: BTreeMap::new(),
            system: DEFAULT_SYSTEM.to_string(),
            budget: ProducerBudget::default(),
        }
    }

    fn fix_registration() -> BackendRegistration {
        BackendRegistration {
            kind: ProducerBackendKind::Fix,
            binary_identity: "a".repeat(BLAKE3_HEX_CHARS),
            supported_systems: BTreeSet::from([DEFAULT_SYSTEM.to_string()]),
        }
    }

    fn valid_identity(request: &NixProducerRequest) -> ProducerIdentityFacts {
        ProducerIdentityFacts {
            backend: request.backend,
            backend_version: "0.3.0".to_string(),
            binary_identity: "a".repeat(BLAKE3_HEX_CHARS),
            trust_posture: BackendTrustPosture::PinnedMantleBuilt,
            eval_args_digest_blake3: eval_args_digest(&request.eval_args, &request.system),
        }
    }

    fn valid_success(request: &NixProducerRequest) -> ProducerSuccess {
        ProducerSuccess {
            schema: NIX_PRODUCER_CONTRACT_SCHEMA.to_string(),
            drv_dir: "/tmp/producer-out".to_string(),
            root_drv_path: "/nix/store/aaaa-pkg.drv".to_string(),
            drv_file_count: 2,
            drv_dir_total_bytes: 8_192,
            identity: valid_identity(request),
        }
    }

    #[test]
    fn valid_request_is_admitted() {
        let request = valid_request();
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn request_rejects_wrong_schema() {
        let mut request = valid_request();
        request.schema = "nix-producer-v0".to_string();
        let error = validate_request(&request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::InvalidRequest);
    }

    #[test]
    fn request_rejects_relative_file_path() {
        let mut request = valid_request();
        request.target = ProducerTarget::File {
            path: "relative/default.nix".to_string(),
            attribute: None,
        };
        let error = validate_request(&request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::InvalidRequest);
    }

    #[test]
    fn request_rejects_empty_expression() {
        let mut request = valid_request();
        request.target = ProducerTarget::Expr {
            text: String::new(),
            attribute: None,
        };
        let error = validate_request(&request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::InvalidRequest);
    }

    #[test]
    fn request_rejects_oversized_expression() {
        let mut request = valid_request();
        request.target = ProducerTarget::Expr {
            text: "x".repeat(MAX_EXPRESSION_TEXT_BYTES + 1),
            attribute: None,
        };
        let error = validate_request(&request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::InvalidRequest);
    }

    #[test]
    fn request_rejects_zero_budget() {
        let mut request = valid_request();
        request.budget.wall_time_ms_max = 0;
        let error = validate_request(&request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::InvalidRequest);
    }

    #[test]
    fn request_allows_disabled_memory_limit() {
        let mut request = valid_request();
        request.budget.memory_bytes_max = 0;
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn request_rejects_control_characters() {
        let mut request = valid_request();
        request.system = "x86_64-linux\nextra".to_string();
        let error = validate_request(&request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::InvalidRequest);
    }

    #[test]
    fn selection_returns_registered_backend() {
        let entry = select_backend(&[fix_registration()], ProducerBackendKind::Fix, DEFAULT_SYSTEM).unwrap();
        assert_eq!(entry.kind, ProducerBackendKind::Fix);
    }

    #[test]
    fn selection_rejects_unknown_backend() {
        let error = select_backend(&[], ProducerBackendKind::Fix, DEFAULT_SYSTEM).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::UnknownBackend);
    }

    #[test]
    fn selection_rejects_duplicate_registration() {
        let registrations = vec![fix_registration(), fix_registration()];
        let error = select_backend(&registrations, ProducerBackendKind::Fix, DEFAULT_SYSTEM).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::ContractViolation);
    }

    #[test]
    fn selection_rejects_unsupported_system() {
        let error = select_backend(&[fix_registration()], ProducerBackendKind::Fix, "aarch64-linux").unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::UnsupportedPlatform);
    }

    #[test]
    fn selection_rejects_non_blake3_fix_identity() {
        let mut registration = fix_registration();
        registration.binary_identity = "/usr/bin/fix".to_string();
        let error = select_backend(&[registration], ProducerBackendKind::Fix, DEFAULT_SYSTEM).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::HashDomainMismatch);
    }

    #[test]
    fn selection_rejects_non_path_host_nix_identity() {
        let registration = BackendRegistration {
            kind: ProducerBackendKind::HostNix,
            binary_identity: "b".repeat(BLAKE3_HEX_CHARS),
            supported_systems: BTreeSet::from([DEFAULT_SYSTEM.to_string()]),
        };
        let error = select_backend(&[registration], ProducerBackendKind::HostNix, DEFAULT_SYSTEM).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::BackendUnavailable);
    }

    #[test]
    fn host_nix_backend_requires_ambient_posture() {
        assert_eq!(ProducerBackendKind::HostNix.required_trust_posture(), BackendTrustPosture::AmbientHost);
        assert_eq!(ProducerBackendKind::Fix.required_trust_posture(), BackendTrustPosture::PinnedMantleBuilt);
    }

    #[test]
    fn eval_args_digest_is_order_independent() {
        let mut first = BTreeMap::new();
        first.insert("a".to_string(), "1".to_string());
        first.insert("b".to_string(), "2".to_string());
        let mut second = BTreeMap::new();
        second.insert("b".to_string(), "2".to_string());
        second.insert("a".to_string(), "1".to_string());
        assert_eq!(eval_args_digest(&first, DEFAULT_SYSTEM), eval_args_digest(&second, DEFAULT_SYSTEM));
        assert_ne!(eval_args_digest(&first, DEFAULT_SYSTEM), eval_args_digest(&first, "aarch64-linux"));
    }

    #[test]
    fn accept_success_admits_valid_observation() {
        let request = valid_request();
        let success = valid_success(&request);
        let outcome = accept_success(&success, &request).unwrap();
        assert!(matches!(outcome, ProducerOutcome::Success(_)));
    }

    #[test]
    fn accept_success_rejects_backend_mismatch() {
        let request = valid_request();
        let mut success = valid_success(&request);
        success.identity.backend = ProducerBackendKind::HostNix;
        let error = accept_success(&success, &request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::ContractViolation);
    }

    #[test]
    fn accept_success_rejects_ambient_fix_posture() {
        let request = valid_request();
        let mut success = valid_success(&request);
        success.identity.trust_posture = BackendTrustPosture::AmbientHost;
        let error = accept_success(&success, &request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::ContractViolation);
    }

    #[test]
    fn accept_success_rejects_eval_args_digest_drift() {
        let request = valid_request();
        let mut success = valid_success(&request);
        success.identity.eval_args_digest_blake3 = "0".repeat(BLAKE3_HEX_CHARS);
        let error = accept_success(&success, &request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::ContractViolation);
    }

    #[test]
    fn accept_success_rejects_non_drv_root() {
        let request = valid_request();
        let mut success = valid_success(&request);
        success.root_drv_path = "/nix/store/aaaa-pkg".to_string();
        let error = accept_success(&success, &request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::MalformedOutput);
    }

    #[test]
    fn accept_success_rejects_empty_closure() {
        let request = valid_request();
        let mut success = valid_success(&request);
        success.drv_file_count = 0;
        let error = accept_success(&success, &request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::MalformedOutput);
    }

    #[test]
    fn accept_success_rejects_budget_overflow() {
        let request = valid_request();
        let mut success = valid_success(&request);
        success.drv_file_count = request.budget.drv_file_count_max + 1;
        let error = accept_success(&success, &request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::OutputTooLarge);
        let mut success = valid_success(&request);
        success.drv_dir_total_bytes = request.budget.output_bytes_max + 1;
        let error = accept_success(&success, &request).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::OutputTooLarge);
    }

    fn evidence_pins() -> BackendEvidencePins {
        BackendEvidencePins {
            backend: ProducerBackendKind::Fix,
            source_revision: "fd675c2e938da6c9f444a00893b6716d66c89927".to_string(),
            source_content_digest: "sha256-qLTYqSdaxPNMsI59PVgOCMVZjBn655xujFeraqxyK/M=".to_string(),
            binary_identity: "a".repeat(BLAKE3_HEX_CHARS),
            reference_nix_version: "2.35.0".to_string(),
            nixpkgs_universe_pin: "nixos-25.11-dirty".to_string(),
        }
    }

    // r[verify nix_producer_adapter.compatibility_evidence]

    #[test]
    fn evidence_is_fresh_when_all_pins_match() {
        let recorded = evidence_pins();
        let current = evidence_pins();
        assert_eq!(classify_evidence_freshness(&recorded, &current), EvidenceFreshness::Fresh);
    }

    #[test]
    fn evidence_is_stale_on_single_field_drift() {
        let recorded = evidence_pins();
        let mut current = evidence_pins();
        current.binary_identity = "b".repeat(BLAKE3_HEX_CHARS);
        assert_eq!(classify_evidence_freshness(&recorded, &current), EvidenceFreshness::Stale {
            drifted_fields: vec!["binary_identity".to_string()]
        });
    }

    #[test]
    fn evidence_is_stale_on_multi_field_drift() {
        let recorded = evidence_pins();
        let mut current = evidence_pins();
        current.source_revision = "0000000000000000000000000000000000000000".to_string();
        current.nixpkgs_universe_pin = "nixos-26.05".to_string();
        assert_eq!(classify_evidence_freshness(&recorded, &current), EvidenceFreshness::Stale {
            drifted_fields: vec!["source_revision".to_string(), "nixpkgs_universe_pin".to_string()]
        });
    }

    #[test]
    fn evidence_is_stale_on_backend_mismatch() {
        let recorded = evidence_pins();
        let mut current = evidence_pins();
        current.backend = ProducerBackendKind::HostNix;
        assert_eq!(classify_evidence_freshness(&recorded, &current), EvidenceFreshness::Stale {
            drifted_fields: vec!["backend".to_string()]
        });
    }

    #[test]
    fn error_class_strings_are_stable() {
        assert_eq!(ProducerErrorClass::UnknownBackend.as_str(), "unknown-backend");
        assert_eq!(ProducerErrorClass::BudgetTimeout.as_str(), "budget-timeout");
        assert_eq!(ProducerErrorClass::DaemonCommandRejected.as_str(), "daemon-command-rejected");
    }
}
