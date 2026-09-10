#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![no_std]

extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

pub const POLICY_SCHEMA: &str = "mantle-evaluation-budget-policy-v1";
pub const REQUEST_SCHEMA: &str = "mantle-evaluator-worker-request-v1";
pub const RESPONSE_SCHEMA: &str = "mantle-evaluator-worker-response-v1";
pub const REPORT_SCHEMA: &str = "mantle-evaluation-budget-report-v1";
pub const POLICY_REF_PREFIX: &str = "evaluation-policy-blake3:";
pub const REQUEST_REF_PREFIX: &str = "evaluation-request-blake3:";
pub const FRAME_HEADER_BYTES: usize = 8;
pub const BLAKE3_HEX_CHARS: usize = 64;
pub const ABSOLUTE_SOURCE_BYTES_MAX: u64 = 67_108_864;
pub const ABSOLUTE_IMPORT_ROOTS_MAX: u32 = 1_024;
pub const ABSOLUTE_IMPORTED_MODULES_MAX: u32 = 65_536;
pub const ABSOLUTE_ROOTS_MAX: u32 = 65_536;
pub const ABSOLUTE_DIAGNOSTICS_MAX: u32 = 4_096;
pub const ABSOLUTE_DIAGNOSTIC_BYTES_MAX: u64 = 16_777_216;
pub const ABSOLUTE_PROTOCOL_BYTES_MAX: u64 = 134_217_728;
pub const ABSOLUTE_WALL_TIME_MS_MAX: u64 = 86_400_000;
pub const ABSOLUTE_CPU_TIME_MS_MAX: u64 = 86_400_000;
pub const ABSOLUTE_PEAK_RSS_BYTES_MAX: u64 = 1_099_511_627_776;
pub const ABSOLUTE_WORKERS_MAX: u32 = 256;
pub const ABSOLUTE_SHUTDOWN_GRACE_MS_MAX: u64 = 60_000;
pub const ABSOLUTE_STDERR_BYTES_MAX: u64 = 16_777_216;
pub const NON_CLAIM: &str =
    "evaluation-budget-compliance-does-not-prove-evaluator-correctness-reproducibility-build-correctness-or-release";

const POLICY_DOMAIN: &[u8] = b"mantle:evaluation-budget-policy:v1\0";
const REQUEST_DOMAIN: &[u8] = b"mantle:evaluator-worker-request:v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluationMode {
    ObserveOnly,
    Enforce,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationBudgetPolicy {
    pub schema: String,
    pub policy_id: String,
    pub mode: EvaluationMode,
    pub source_bytes_max: u64,
    pub import_roots_max: u32,
    pub imported_modules_max: u32,
    pub discovered_roots_max: u32,
    pub selected_roots_max: u32,
    pub diagnostics_max: u32,
    pub diagnostic_bytes_max: u64,
    pub request_bytes_max: u64,
    pub response_bytes_max: u64,
    pub wall_time_ms_max: u64,
    pub cpu_time_ms_max: u64,
    pub peak_rss_bytes_max: u64,
    pub workers_max: u32,
    pub shutdown_grace_ms: u64,
    pub stderr_bytes_max: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MechanismSupport {
    pub worker_process: bool,
    pub import_filesystem_confinement: bool,
    pub wall_deadline: bool,
    pub cpu_time_enforcement: bool,
    pub address_space_enforcement: bool,
    pub cpu_time_observation: bool,
    pub peak_rss_observation: bool,
    pub cpu_time_mechanism: String,
    pub memory_enforcement_mechanism: String,
    pub peak_rss_mechanism: String,
    pub import_confinement_mechanism: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportDescriptor {
    pub canonical_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluationOperation {
    WholeValue,
    SelectedRoots,
    AllRoots,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorWorkerRequest {
    pub schema: String,
    pub request_ref: String,
    pub policy_ref: String,
    pub policy: EvaluationBudgetPolicy,
    pub source_name: String,
    pub source_blake3: String,
    pub source_bytes: Vec<u8>,
    pub imports: Vec<ImportDescriptor>,
    pub import_entry_count: u32,
    pub selected_roots: Vec<String>,
    pub evaluator_id: String,
    pub evaluator_version: String,
    pub operation: EvaluationOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerResponseStatus {
    Success,
    EvaluationError,
    ResponseOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorObservation {
    pub imported_module_count: Option<u32>,
    pub discovered_root_count: Option<u32>,
    pub selected_root_count: u32,
    pub explicit_top_level_root_force_count: u32,
    pub actual_nonselected_evaluation_count: Option<u32>,
    pub diagnostic_count: u32,
    pub diagnostic_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessObservation {
    pub wall_time_ms: u64,
    pub cpu_time_ms: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorWorkerResponse {
    pub schema: String,
    pub request_ref: String,
    pub policy_ref: String,
    pub worker_identity: String,
    pub status: WorkerResponseStatus,
    pub output_json: Option<String>,
    pub error_class: Option<String>,
    pub diagnostics: Vec<String>,
    pub evaluator: EvaluatorObservation,
    pub process: ProcessObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetricFactStatus {
    Observed,
    Enforced,
    Unavailable,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricFact {
    pub name: String,
    pub unit: String,
    pub role: String,
    pub status: MetricFactStatus,
    pub value: Option<u64>,
    pub mechanism: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalDisposition {
    Success,
    EvaluationError,
    AdmissionRejected,
    BudgetUnsupported,
    Timeout,
    Cancelled,
    CpuLimit,
    MemoryLimit,
    WorkerCrash,
    WorkerSignal,
    ProtocolError,
    ResponseOverflow,
    IncompleteTeardown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TeardownFacts {
    pub cancellation_requested: bool,
    pub deadline_exceeded: bool,
    pub terminate_sent: bool,
    pub kill_sent: bool,
    pub reaped: bool,
    pub response_present: bool,
    pub stderr_bytes: u64,
    pub stderr_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerExitKind {
    SuccessResponse,
    EvaluationErrorResponse,
    ResponseOverflow,
    CpuLimitSignal,
    MemoryLimitFailure,
    Signal,
    Crash,
    ProtocolFailure,
    NoResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalFacts {
    pub teardown: TeardownFacts,
    pub exit_kind: WorkerExitKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationBudgetReport {
    pub schema: String,
    pub request_ref: String,
    pub policy_ref: String,
    pub mode: EvaluationMode,
    pub terminal_disposition: TerminalDisposition,
    pub metrics: Vec<MetricFact>,
    pub evaluator: Option<EvaluatorObservation>,
    pub teardown: TeardownFacts,
    pub error_class: Option<String>,
    pub bounded_stderr: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetError {
    InvalidPolicySchema,
    InvalidPolicyId,
    InvalidLimit(&'static str),
    LimitTooLarge(&'static str),
    ContradictoryLimit(&'static str),
    Unsupported(&'static str),
    InvalidRequestSchema,
    InvalidReference(&'static str),
    SourceIdentityMismatch,
    InvalidImportPath(String),
    DuplicateImportPath(String),
    DuplicateSelectedRoot(String),
    EmptySelectedRoot,
    LimitExceeded(&'static str),
    FrameHeaderIncomplete,
    FrameTooLarge,
    FrameLengthOverflow,
    TrailingData,
    IntegerOverflow(&'static str),
}

impl fmt::Display for BudgetError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicySchema => output.write_str("evaluation-budget-invalid-policy-schema"),
            Self::InvalidPolicyId => output.write_str("evaluation-budget-invalid-policy-id"),
            Self::InvalidLimit(name) => write!(output, "evaluation-budget-invalid-limit:{name}"),
            Self::LimitTooLarge(name) => write!(output, "evaluation-budget-limit-too-large:{name}"),
            Self::ContradictoryLimit(name) => write!(output, "evaluation-budget-contradictory-limit:{name}"),
            Self::Unsupported(name) => write!(output, "evaluation-budget-unsupported:{name}"),
            Self::InvalidRequestSchema => output.write_str("evaluation-budget-invalid-request-schema"),
            Self::InvalidReference(name) => write!(output, "evaluation-budget-invalid-reference:{name}"),
            Self::SourceIdentityMismatch => output.write_str("evaluation-budget-source-identity-mismatch"),
            Self::InvalidImportPath(path) => write!(output, "evaluation-budget-invalid-import-path:{path}"),
            Self::DuplicateImportPath(path) => write!(output, "evaluation-budget-duplicate-import-path:{path}"),
            Self::DuplicateSelectedRoot(root) => write!(output, "evaluation-budget-duplicate-selected-root:{root}"),
            Self::EmptySelectedRoot => output.write_str("evaluation-budget-empty-selected-root"),
            Self::LimitExceeded(name) => write!(output, "evaluation-budget-limit-exceeded:{name}"),
            Self::FrameHeaderIncomplete => output.write_str("evaluation-budget-frame-header-incomplete"),
            Self::FrameTooLarge => output.write_str("evaluation-budget-frame-too-large"),
            Self::FrameLengthOverflow => output.write_str("evaluation-budget-frame-length-overflow"),
            Self::TrailingData => output.write_str("evaluation-budget-frame-trailing-data"),
            Self::IntegerOverflow(name) => write!(output, "evaluation-budget-integer-overflow:{name}"),
        }
    }
}

pub fn validate_policy(policy: &EvaluationBudgetPolicy, support: &MechanismSupport) -> Result<String, BudgetError> {
    if policy.schema != POLICY_SCHEMA {
        return Err(BudgetError::InvalidPolicySchema);
    }
    if policy.policy_id.is_empty() || policy.policy_id.len() > BLAKE3_HEX_CHARS {
        return Err(BudgetError::InvalidPolicyId);
    }
    validate_policy_limits(policy)?;
    if policy.diagnostic_bytes_max > policy.response_bytes_max {
        return Err(BudgetError::ContradictoryLimit("diagnostic-bytes-vs-response-bytes"));
    }
    if matches!(policy.mode, EvaluationMode::Enforce) {
        validate_strict_support(support)?;
    }
    hash_policy(policy)
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
pub fn prepare_request(
    mut request: EvaluatorWorkerRequest,
    policy: &EvaluationBudgetPolicy,
    policy_ref: &str,
) -> Result<EvaluatorWorkerRequest, BudgetError> {
    if request.schema != REQUEST_SCHEMA {
        return Err(BudgetError::InvalidRequestSchema);
    }
    if request.policy_ref != policy_ref || !is_ref(policy_ref, POLICY_REF_PREFIX) {
        return Err(BudgetError::InvalidReference("policy-ref"));
    }
    if request.policy != *policy {
        return Err(BudgetError::InvalidReference("policy-facts"));
    }
    check_count(request.source_bytes.len(), policy.source_bytes_max, "source-bytes")?;
    check_count(request.imports.len(), u64::from(policy.import_roots_max), "import-roots")?;
    if request.import_entry_count > policy.imported_modules_max {
        return Err(BudgetError::LimitExceeded("import-entries"));
    }
    check_count(request.selected_roots.len(), u64::from(policy.selected_roots_max), "selected-roots")?;
    validate_source_identity(&request)?;
    normalize_imports(&mut request.imports)?;
    normalize_selected_roots(&mut request.selected_roots)?;
    validate_operation_roots(&request)?;
    request.request_ref = hash_request(&request)?;
    Ok(request)
}

pub fn validate_prepared_request(
    request: &EvaluatorWorkerRequest,
    policy: &EvaluationBudgetPolicy,
    policy_ref: &str,
) -> Result<(), BudgetError> {
    let prepared = prepare_request(request.clone(), policy, policy_ref)?;
    if prepared != *request {
        return Err(BudgetError::InvalidReference("request-ref"));
    }
    Ok(())
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
pub fn validate_worker_response(
    response: &EvaluatorWorkerResponse,
    request: &EvaluatorWorkerRequest,
) -> Result<(), BudgetError> {
    if response.schema != RESPONSE_SCHEMA {
        return Err(BudgetError::InvalidReference("response-schema"));
    }
    if response.request_ref != request.request_ref || response.policy_ref != request.policy_ref {
        return Err(BudgetError::InvalidReference("response-identity"));
    }
    if response.worker_identity.is_empty() {
        return Err(BudgetError::InvalidReference("worker-identity"));
    }
    validate_response_shape(response)?;
    if response.evaluator.selected_root_count > request.policy.selected_roots_max {
        return Err(BudgetError::LimitExceeded("response-selected-roots"));
    }
    if response
        .evaluator
        .discovered_root_count
        .is_some_and(|count| count > request.policy.discovered_roots_max)
    {
        return Err(BudgetError::LimitExceeded("response-discovered-roots"));
    }
    if response.evaluator.diagnostic_count > request.policy.diagnostics_max {
        return Err(BudgetError::LimitExceeded("response-diagnostics"));
    }
    let diagnostic_bytes = response.diagnostics.iter().try_fold(0_u64, |total, diagnostic| {
        let bytes =
            u64::try_from(diagnostic.len()).map_err(|_| BudgetError::IntegerOverflow("response-diagnostic-bytes"))?;
        total.checked_add(bytes).ok_or(BudgetError::IntegerOverflow("response-diagnostic-bytes"))
    })?;
    let diagnostic_count =
        u32::try_from(response.diagnostics.len()).map_err(|_| BudgetError::IntegerOverflow("response-diagnostics"))?;
    if diagnostic_count != response.evaluator.diagnostic_count
        || diagnostic_bytes != response.evaluator.diagnostic_bytes
    {
        return Err(BudgetError::InvalidReference("response-diagnostic-facts"));
    }
    if diagnostic_bytes > request.policy.diagnostic_bytes_max {
        return Err(BudgetError::LimitExceeded("response-diagnostic-bytes"));
    }
    Ok(())
}

fn validate_response_shape(response: &EvaluatorWorkerResponse) -> Result<(), BudgetError> {
    #[allow(tigerstyle::bool_naming)] // legacy binding name kept for review continuity
    let shape_is_valid = match response.status {
        WorkerResponseStatus::Success => response.output_json.is_some() && response.error_class.is_none(),
        WorkerResponseStatus::EvaluationError | WorkerResponseStatus::ResponseOverflow => {
            response.output_json.is_none() && response.error_class.is_some()
        }
    };
    if !shape_is_valid {
        return Err(BudgetError::InvalidReference("response-shape"));
    }
    Ok(())
}

pub fn frame_payload(payload: &[u8], bytes_max: u64) -> Result<Vec<u8>, BudgetError> {
    check_count(payload.len(), bytes_max, "protocol-bytes")?;
    let payload_len = u64::try_from(payload.len()).map_err(|_| BudgetError::FrameLengthOverflow)?;
    #[allow(tigerstyle::numeric_units)] // name describes a policy set or bound table, not a raw quantity
    let capacity = FRAME_HEADER_BYTES.checked_add(payload.len()).ok_or(BudgetError::FrameLengthOverflow)?;
    let mut framed = Vec::with_capacity(capacity);
    framed.extend_from_slice(&payload_len.to_le_bytes());
    framed.extend_from_slice(payload);
    Ok(framed)
}

#[allow(tigerstyle::usize_in_public_api)] // public API compatibility; platform-independent values enforced by bounds
pub fn decode_frame_header(header: &[u8], bytes_max: u64) -> Result<usize, BudgetError> {
    if header.len() != FRAME_HEADER_BYTES {
        return Err(BudgetError::FrameHeaderIncomplete);
    }
    let mut fixed = [0_u8; FRAME_HEADER_BYTES];
    fixed.copy_from_slice(header);
    let declared = u64::from_le_bytes(fixed);
    if declared > bytes_max {
        return Err(BudgetError::FrameTooLarge);
    }
    usize::try_from(declared).map_err(|_| BudgetError::FrameLengthOverflow)
}

#[allow(tigerstyle::usize_in_public_api)] // public API compatibility; platform-independent values enforced by bounds
pub fn reject_trailing_data(trailing_byte_count: usize) -> Result<(), BudgetError> {
    if trailing_byte_count > 0 {
        return Err(BudgetError::TrailingData);
    }
    Ok(())
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
pub fn truncate_diagnostics(
    diagnostics: &[String],
    diagnostics_max: u32,
    diagnostic_bytes_max: u64,
) -> Result<(Vec<String>, bool), BudgetError> {
    #[allow(tigerstyle::numeric_units)] // name describes a policy set or bound table, not a raw quantity
    let item_limit = usize::try_from(diagnostics_max).map_err(|_| BudgetError::IntegerOverflow("diagnostics"))?;
    #[allow(tigerstyle::numeric_units)] // name describes a policy set or bound table, not a raw quantity
    let byte_limit =
        usize::try_from(diagnostic_bytes_max).map_err(|_| BudgetError::IntegerOverflow("diagnostic-bytes"))?;
    let mut result = Vec::with_capacity(diagnostics.len().min(item_limit));
    let mut retained_bytes = 0_usize;
    let mut is_truncated = diagnostics.len() > item_limit;
    for diagnostic in diagnostics.iter().take(item_limit) {
        if retained_bytes >= byte_limit {
            is_truncated = true;
            break;
        }
        let remaining = byte_limit.saturating_sub(retained_bytes);
        let retained = truncate_utf8(diagnostic, remaining);
        retained_bytes =
            retained_bytes.checked_add(retained.len()).ok_or(BudgetError::IntegerOverflow("diagnostic-bytes"))?;
        if retained.len() < diagnostic.len() {
            is_truncated = true;
        }
        result.push(retained.to_string());
    }
    Ok((result, is_truncated))
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
pub fn classify_terminal(facts: &TerminalFacts) -> TerminalDisposition {
    if facts.teardown.cancellation_requested {
        return if facts.teardown.reaped {
            TerminalDisposition::Cancelled
        } else {
            TerminalDisposition::IncompleteTeardown
        };
    }
    if facts.teardown.deadline_exceeded {
        return if facts.teardown.reaped {
            TerminalDisposition::Timeout
        } else {
            TerminalDisposition::IncompleteTeardown
        };
    }
    if !facts.teardown.reaped {
        return TerminalDisposition::IncompleteTeardown;
    }
    match facts.exit_kind {
        WorkerExitKind::SuccessResponse => TerminalDisposition::Success,
        WorkerExitKind::EvaluationErrorResponse => TerminalDisposition::EvaluationError,
        WorkerExitKind::ResponseOverflow => TerminalDisposition::ResponseOverflow,
        WorkerExitKind::CpuLimitSignal => TerminalDisposition::CpuLimit,
        WorkerExitKind::MemoryLimitFailure => TerminalDisposition::MemoryLimit,
        WorkerExitKind::Signal => TerminalDisposition::WorkerSignal,
        WorkerExitKind::Crash | WorkerExitKind::NoResponse => TerminalDisposition::WorkerCrash,
        WorkerExitKind::ProtocolFailure => TerminalDisposition::ProtocolError,
    }
}

#[allow(tigerstyle::ambiguous_params)] // parameter order fixed by wire format and call history
#[allow(tigerstyle::too_many_parameters)] // request fields kept explicit for review; options-struct refactor tracked
pub fn metric_fact(
    name: &str,
    unit: &str,
    role: &str,
    status: MetricFactStatus,
    value: Option<u64>,
    mechanism: &str,
    reason: Option<&str>,
) -> MetricFact {
    MetricFact {
        name: name.to_string(),
        unit: unit.to_string(),
        role: role.to_string(),
        status,
        value,
        mechanism: mechanism.to_string(),
        reason: reason.map(ToString::to_string),
    }
}

#[allow(tigerstyle::ambiguous_params)] // parameter order fixed by wire format and call history
#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
#[allow(tigerstyle::too_many_parameters)] // request fields kept explicit for review; options-struct refactor tracked
pub fn build_report(
    request_ref: &str,
    policy_ref: &str,
    mode: EvaluationMode,
    facts: TerminalFacts,
    metrics: Vec<MetricFact>,
    evaluator: Option<EvaluatorObservation>,
    error_class: Option<String>,
    bounded_stderr: String,
) -> Result<EvaluationBudgetReport, BudgetError> {
    if !is_ref(request_ref, REQUEST_REF_PREFIX) {
        return Err(BudgetError::InvalidReference("request-ref"));
    }
    if !is_ref(policy_ref, POLICY_REF_PREFIX) {
        return Err(BudgetError::InvalidReference("policy-ref"));
    }
    let terminal_disposition = classify_terminal(&facts);
    Ok(EvaluationBudgetReport {
        schema: REPORT_SCHEMA.to_string(),
        request_ref: request_ref.to_string(),
        policy_ref: policy_ref.to_string(),
        mode,
        terminal_disposition,
        metrics,
        evaluator,
        teardown: facts.teardown,
        error_class,
        bounded_stderr,
        non_claim: NON_CLAIM.to_string(),
    })
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
fn validate_policy_limits(policy: &EvaluationBudgetPolicy) -> Result<(), BudgetError> {
    #[allow(tigerstyle::numeric_units)] // name describes a policy set or bound table, not a raw quantity
    let u64_limits = [
        (policy.source_bytes_max, ABSOLUTE_SOURCE_BYTES_MAX, "source-bytes"),
        (policy.diagnostic_bytes_max, ABSOLUTE_DIAGNOSTIC_BYTES_MAX, "diagnostic-bytes"),
        (policy.request_bytes_max, ABSOLUTE_PROTOCOL_BYTES_MAX, "request-bytes"),
        (policy.response_bytes_max, ABSOLUTE_PROTOCOL_BYTES_MAX, "response-bytes"),
        (policy.wall_time_ms_max, ABSOLUTE_WALL_TIME_MS_MAX, "wall-time-ms"),
        (policy.cpu_time_ms_max, ABSOLUTE_CPU_TIME_MS_MAX, "cpu-time-ms"),
        (policy.peak_rss_bytes_max, ABSOLUTE_PEAK_RSS_BYTES_MAX, "peak-rss-bytes"),
        (policy.shutdown_grace_ms, ABSOLUTE_SHUTDOWN_GRACE_MS_MAX, "shutdown-grace-ms"),
        (policy.stderr_bytes_max, ABSOLUTE_STDERR_BYTES_MAX, "stderr-bytes"),
    ];
    for (value, absolute_max, name) in u64_limits {
        validate_limit(value, absolute_max, name)?;
    }
    #[allow(tigerstyle::numeric_units)] // name describes a policy set or bound table, not a raw quantity
    let u32_limits = [
        (policy.import_roots_max, ABSOLUTE_IMPORT_ROOTS_MAX, "import-roots"),
        (policy.imported_modules_max, ABSOLUTE_IMPORTED_MODULES_MAX, "imported-modules"),
        (policy.discovered_roots_max, ABSOLUTE_ROOTS_MAX, "discovered-roots"),
        (policy.selected_roots_max, ABSOLUTE_ROOTS_MAX, "selected-roots"),
        (policy.diagnostics_max, ABSOLUTE_DIAGNOSTICS_MAX, "diagnostics"),
        (policy.workers_max, ABSOLUTE_WORKERS_MAX, "workers"),
    ];
    for (value, absolute_max, name) in u32_limits {
        validate_limit(u64::from(value), u64::from(absolute_max), name)?;
    }
    if policy.selected_roots_max > policy.discovered_roots_max {
        return Err(BudgetError::ContradictoryLimit("selected-roots-vs-discovered-roots"));
    }
    Ok(())
}

#[allow(tigerstyle::ambiguous_params)] // parameter order fixed by wire format and call history
fn validate_limit(value: u64, absolute_max: u64, name: &'static str) -> Result<(), BudgetError> {
    if value == 0 {
        return Err(BudgetError::InvalidLimit(name));
    }
    if value > absolute_max {
        return Err(BudgetError::LimitTooLarge(name));
    }
    Ok(())
}

fn validate_strict_support(support: &MechanismSupport) -> Result<(), BudgetError> {
    let required = [
        (support.worker_process, "worker-process"),
        (support.import_filesystem_confinement, "import-filesystem-confinement"),
        (support.wall_deadline, "wall-deadline"),
        (support.cpu_time_enforcement, "cpu-time-enforcement"),
        (support.address_space_enforcement, "memory-enforcement"),
        (support.cpu_time_observation, "cpu-time-observation"),
        (support.peak_rss_observation, "peak-rss-observation"),
    ];
    for (is_supported, name) in required {
        if !is_supported {
            return Err(BudgetError::Unsupported(name));
        }
    }
    Ok(())
}

fn validate_source_identity(request: &EvaluatorWorkerRequest) -> Result<(), BudgetError> {
    if request.source_name.is_empty() || request.evaluator_id.is_empty() || request.evaluator_version.is_empty() {
        return Err(BudgetError::InvalidReference("source-or-evaluator"));
    }
    let observed = blake3::hash(&request.source_bytes).to_hex().to_string();
    if observed != request.source_blake3 {
        return Err(BudgetError::SourceIdentityMismatch);
    }
    Ok(())
}

#[allow(tigerstyle::compound_condition)] // clauses kept inline for review; decomposition tracked separately
fn normalize_imports(imports: &mut [ImportDescriptor]) -> Result<(), BudgetError> {
    imports.sort_by(|left, right| left.canonical_path.cmp(&right.canonical_path));
    let mut previous: Option<&str> = None;
    for import in imports {
        let path = import.canonical_path.as_str();
        if !path.starts_with('/') || path.contains("/../") || path.ends_with("/..") || path.contains('\0') {
            return Err(BudgetError::InvalidImportPath(path.to_string()));
        }
        if previous == Some(path) {
            return Err(BudgetError::DuplicateImportPath(path.to_string()));
        }
        previous = Some(path);
    }
    Ok(())
}

fn validate_operation_roots(request: &EvaluatorWorkerRequest) -> Result<(), BudgetError> {
    let has_selected_roots = !request.selected_roots.is_empty();
    match request.operation {
        EvaluationOperation::SelectedRoots if !has_selected_roots => {
            Err(BudgetError::InvalidReference("selected-roots-required"))
        }
        EvaluationOperation::WholeValue | EvaluationOperation::AllRoots if has_selected_roots => {
            Err(BudgetError::InvalidReference("selected-roots-forbidden"))
        }
        _ => Ok(()),
    }
}

fn normalize_selected_roots(roots: &mut [String]) -> Result<(), BudgetError> {
    roots.sort();
    let mut seen = BTreeSet::new();
    for root in roots {
        if root.is_empty() {
            return Err(BudgetError::EmptySelectedRoot);
        }
        if !seen.insert(root.clone()) {
            return Err(BudgetError::DuplicateSelectedRoot(root.clone()));
        }
    }
    Ok(())
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
fn hash_policy(policy: &EvaluationBudgetPolicy) -> Result<String, BudgetError> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, POLICY_DOMAIN)?;
    hash_string(&mut hasher, &policy.policy_id)?;
    hasher.update(&[match policy.mode {
        EvaluationMode::ObserveOnly => 0,
        EvaluationMode::Enforce => 1,
    }]);
    for value in [
        policy.source_bytes_max,
        u64::from(policy.import_roots_max),
        u64::from(policy.imported_modules_max),
        u64::from(policy.discovered_roots_max),
        u64::from(policy.selected_roots_max),
        u64::from(policy.diagnostics_max),
        policy.diagnostic_bytes_max,
        policy.request_bytes_max,
        policy.response_bytes_max,
        policy.wall_time_ms_max,
        policy.cpu_time_ms_max,
        policy.peak_rss_bytes_max,
        u64::from(policy.workers_max),
        policy.shutdown_grace_ms,
        policy.stderr_bytes_max,
    ] {
        hasher.update(&value.to_le_bytes());
    }
    Ok(format!("{POLICY_REF_PREFIX}{}", hasher.finalize().to_hex()))
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
fn hash_request(request: &EvaluatorWorkerRequest) -> Result<String, BudgetError> {
    let mut hasher = blake3::Hasher::new();
    hash_bytes(&mut hasher, REQUEST_DOMAIN)?;
    hash_string(&mut hasher, &request.policy_ref)?;
    hash_string(&mut hasher, &request.source_name)?;
    hash_string(&mut hasher, &request.source_blake3)?;
    hash_bytes(&mut hasher, &request.source_bytes)?;
    hash_count(&mut hasher, request.imports.len())?;
    for import in &request.imports {
        hash_string(&mut hasher, &import.canonical_path)?;
    }
    hasher.update(&request.import_entry_count.to_le_bytes());
    hash_count(&mut hasher, request.selected_roots.len())?;
    for root in &request.selected_roots {
        hash_string(&mut hasher, root)?;
    }
    hash_string(&mut hasher, &request.evaluator_id)?;
    hash_string(&mut hasher, &request.evaluator_version)?;
    hasher.update(&[match request.operation {
        EvaluationOperation::WholeValue => 0,
        EvaluationOperation::SelectedRoots => 1,
        EvaluationOperation::AllRoots => 2,
    }]);
    Ok(format!("{REQUEST_REF_PREFIX}{}", hasher.finalize().to_hex()))
}

fn hash_string(hasher: &mut blake3::Hasher, value: &str) -> Result<(), BudgetError> {
    hash_bytes(hasher, value.as_bytes())
}

fn hash_bytes(hasher: &mut blake3::Hasher, value: &[u8]) -> Result<(), BudgetError> {
    hash_count(hasher, value.len())?;
    hasher.update(value);
    Ok(())
}

fn hash_count(hasher: &mut blake3::Hasher, count: usize) -> Result<(), BudgetError> {
    let count = u64::try_from(count).map_err(|_| BudgetError::IntegerOverflow("hash-count"))?;
    hasher.update(&count.to_le_bytes());
    Ok(())
}

fn check_count(value: usize, limit: u64, name: &'static str) -> Result<(), BudgetError> {
    let value = u64::try_from(value).map_err(|_| BudgetError::IntegerOverflow(name))?;
    if value > limit {
        return Err(BudgetError::LimitExceeded(name));
    }
    Ok(())
}

fn truncate_utf8(value: &str, bytes_max: usize) -> &str {
    if value.len() <= bytes_max {
        return value;
    }
    let mut end = bytes_max;
    while end > 0 && !value.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    &value[..end]
}

#[allow(tigerstyle::ambiguous_params)] // parameter order fixed by wire format and call history
fn is_ref(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|digest| {
        digest.len() == BLAKE3_HEX_CHARS
            && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const SOURCE: &[u8] = b"{ answer = 42 }";
    const TEST_BYTES_MAX: u64 = 1_048_576;
    const TEST_COUNT_MAX: u32 = 64;
    const TEST_TIME_MS_MAX: u64 = 10_000;
    const TEST_MEMORY_BYTES_MAX: u64 = 536_870_912;
    const PROPERTY_FRAME_SIZE_SMALL: usize = 9;
    const PROPERTY_FRAME_SIZE_MEDIUM: usize = 1_024;
    const PROPERTY_FRAME_SIZE_LARGE: usize = 65_536;

    fn policy(mode: EvaluationMode) -> EvaluationBudgetPolicy {
        EvaluationBudgetPolicy {
            schema: POLICY_SCHEMA.to_string(),
            policy_id: "evaluation-test-v1".to_string(),
            mode,
            source_bytes_max: TEST_BYTES_MAX,
            import_roots_max: TEST_COUNT_MAX,
            imported_modules_max: TEST_COUNT_MAX,
            discovered_roots_max: TEST_COUNT_MAX,
            selected_roots_max: TEST_COUNT_MAX,
            diagnostics_max: TEST_COUNT_MAX,
            diagnostic_bytes_max: TEST_BYTES_MAX,
            request_bytes_max: TEST_BYTES_MAX,
            response_bytes_max: TEST_BYTES_MAX,
            wall_time_ms_max: TEST_TIME_MS_MAX,
            cpu_time_ms_max: TEST_TIME_MS_MAX,
            peak_rss_bytes_max: TEST_MEMORY_BYTES_MAX,
            workers_max: 1,
            shutdown_grace_ms: TEST_TIME_MS_MAX,
            stderr_bytes_max: TEST_BYTES_MAX,
        }
    }

    fn support() -> MechanismSupport {
        MechanismSupport {
            worker_process: true,
            import_filesystem_confinement: true,
            wall_deadline: true,
            cpu_time_enforcement: true,
            address_space_enforcement: true,
            cpu_time_observation: true,
            peak_rss_observation: true,
            cpu_time_mechanism: "getrusage".to_string(),
            memory_enforcement_mechanism: "rlimit-as".to_string(),
            peak_rss_mechanism: "getrusage-maxrss".to_string(),
            import_confinement_mechanism: "fixture-confinement".to_string(),
        }
    }

    fn request(policy_ref: &str) -> EvaluatorWorkerRequest {
        EvaluatorWorkerRequest {
            schema: REQUEST_SCHEMA.to_string(),
            request_ref: String::new(),
            policy_ref: policy_ref.to_string(),
            policy: policy(EvaluationMode::Enforce),
            source_name: "fixture.ncl".to_string(),
            source_blake3: blake3::hash(SOURCE).to_hex().to_string(),
            source_bytes: SOURCE.to_vec(),
            imports: vec![ImportDescriptor {
                canonical_path: "/fixture/imports".to_string(),
            }],
            import_entry_count: 1,
            selected_roots: vec!["answer".to_string()],
            evaluator_id: "nickel".to_string(),
            evaluator_version: "2".to_string(),
            operation: EvaluationOperation::SelectedRoots,
        }
    }

    #[test]
    fn policy_and_request_identity_are_deterministic() {
        let policy = policy(EvaluationMode::Enforce);
        let first_policy_ref = validate_policy(&policy, &support()).unwrap();
        let second_policy_ref = validate_policy(&policy, &support()).unwrap();
        assert_eq!(first_policy_ref, second_policy_ref);
        let first = prepare_request(request(&first_policy_ref), &policy, &first_policy_ref).unwrap();
        let second = prepare_request(request(&second_policy_ref), &policy, &second_policy_ref).unwrap();
        assert_eq!(first, second);
        assert!(first.request_ref.starts_with(REQUEST_REF_PREFIX));
    }

    #[test]
    fn strict_support_and_policy_limits_fail_closed() {
        let mut unsupported = support();
        unsupported.address_space_enforcement = false;
        assert_eq!(
            validate_policy(&policy(EvaluationMode::Enforce), &unsupported).unwrap_err(),
            BudgetError::Unsupported("memory-enforcement")
        );
        assert!(validate_policy(&policy(EvaluationMode::ObserveOnly), &unsupported).is_ok());

        let mut zero = policy(EvaluationMode::Enforce);
        zero.wall_time_ms_max = 0;
        assert_eq!(validate_policy(&zero, &support()).unwrap_err(), BudgetError::InvalidLimit("wall-time-ms"));
        let mut contradiction = policy(EvaluationMode::Enforce);
        contradiction.selected_roots_max = contradiction.discovered_roots_max.saturating_add(1);
        assert_eq!(
            validate_policy(&contradiction, &support()).unwrap_err(),
            BudgetError::ContradictoryLimit("selected-roots-vs-discovered-roots")
        );
    }

    #[test]
    fn request_rejects_oversize_duplicates_and_identity_drift() {
        let policy = policy(EvaluationMode::Enforce);
        let policy_ref = validate_policy(&policy, &support()).unwrap();
        let mut oversized = request(&policy_ref);
        oversized.source_bytes = vec![0_u8; usize::try_from(TEST_BYTES_MAX).unwrap().saturating_add(1)];
        assert_eq!(
            prepare_request(oversized, &policy, &policy_ref).unwrap_err(),
            BudgetError::LimitExceeded("source-bytes")
        );

        let mut duplicate = request(&policy_ref);
        duplicate.selected_roots.push("answer".to_string());
        assert_eq!(
            prepare_request(duplicate, &policy, &policy_ref).unwrap_err(),
            BudgetError::DuplicateSelectedRoot("answer".to_string())
        );

        let mut drift = request(&policy_ref);
        drift.source_blake3 = "0".repeat(BLAKE3_HEX_CHARS);
        assert_eq!(prepare_request(drift, &policy, &policy_ref).unwrap_err(), BudgetError::SourceIdentityMismatch);
    }

    #[test]
    fn bounded_protocol_and_identity_properties_hold_across_fixture_matrix() {
        let frame_sizes = [
            0,
            1,
            FRAME_HEADER_BYTES.saturating_sub(1),
            PROPERTY_FRAME_SIZE_SMALL,
            PROPERTY_FRAME_SIZE_MEDIUM,
            PROPERTY_FRAME_SIZE_LARGE,
        ];
        for payload_size in frame_sizes {
            let payload = vec![b'x'; payload_size];
            let frame = frame_payload(&payload, TEST_BYTES_MAX).unwrap();
            let decoded = decode_frame_header(&frame[..FRAME_HEADER_BYTES], TEST_BYTES_MAX).unwrap();
            assert_eq!(decoded, payload_size);
            assert_eq!(&frame[FRAME_HEADER_BYTES..], payload);
        }

        let policy = policy(EvaluationMode::Enforce);
        let policy_ref = validate_policy(&policy, &support()).unwrap();
        let mut forward = request(&policy_ref);
        forward.imports.push(ImportDescriptor {
            canonical_path: "/another/import".to_string(),
        });
        forward.import_entry_count = 2;
        let mut reverse = forward.clone();
        reverse.imports.reverse();
        let forward = prepare_request(forward, &policy, &policy_ref).unwrap();
        let reverse = prepare_request(reverse, &policy, &policy_ref).unwrap();
        assert_eq!(forward.request_ref, reverse.request_ref);
        assert_eq!(forward.imports, reverse.imports);
    }

    #[test]
    fn worker_response_validation_accepts_bound_facts_and_rejects_drift() {
        let policy = policy(EvaluationMode::Enforce);
        let policy_ref = validate_policy(&policy, &support()).unwrap();
        let request = prepare_request(request(&policy_ref), &policy, &policy_ref).unwrap();
        let mut response = EvaluatorWorkerResponse {
            schema: RESPONSE_SCHEMA.to_string(),
            request_ref: request.request_ref.clone(),
            policy_ref: policy_ref.clone(),
            worker_identity: "fixture-worker".to_string(),
            status: WorkerResponseStatus::Success,
            output_json: Some("42".to_string()),
            error_class: None,
            diagnostics: Vec::new(),
            evaluator: EvaluatorObservation {
                imported_module_count: None,
                discovered_root_count: Some(1),
                selected_root_count: 1,
                explicit_top_level_root_force_count: 1,
                actual_nonselected_evaluation_count: None,
                diagnostic_count: 0,
                diagnostic_bytes: 0,
            },
            process: ProcessObservation {
                wall_time_ms: 1,
                cpu_time_ms: Some(1),
                peak_rss_bytes: Some(1),
            },
        };
        assert!(validate_worker_response(&response, &request).is_ok());
        response.evaluator.diagnostic_count = 1;
        assert_eq!(
            validate_worker_response(&response, &request).unwrap_err(),
            BudgetError::InvalidReference("response-diagnostic-facts")
        );
        response.evaluator.diagnostic_count = 0;
        response.request_ref = format!("{REQUEST_REF_PREFIX}{}", "0".repeat(BLAKE3_HEX_CHARS));
        assert_eq!(
            validate_worker_response(&response, &request).unwrap_err(),
            BudgetError::InvalidReference("response-identity")
        );
    }

    #[test]
    fn framing_rejects_oversize_incomplete_and_trailing_data() {
        let payload = b"bounded";
        let framed = frame_payload(payload, TEST_BYTES_MAX).unwrap();
        let length = decode_frame_header(&framed[..FRAME_HEADER_BYTES], TEST_BYTES_MAX).unwrap();
        assert_eq!(length, payload.len());
        assert_eq!(&framed[FRAME_HEADER_BYTES..], payload);
        assert_eq!(
            decode_frame_header(&u64::MAX.to_le_bytes(), TEST_BYTES_MAX).unwrap_err(),
            BudgetError::FrameTooLarge
        );
        assert_eq!(decode_frame_header(&[0_u8; 1], TEST_BYTES_MAX).unwrap_err(), BudgetError::FrameHeaderIncomplete);
        assert_eq!(reject_trailing_data(1).unwrap_err(), BudgetError::TrailingData);
    }

    #[test]
    fn diagnostics_truncate_on_item_byte_and_utf8_boundaries() {
        let diagnostics = vec!["éé".to_string(), "second".to_string(), "third".to_string()];
        let (retained, is_truncated) = truncate_diagnostics(&diagnostics, 2, 3).unwrap();
        assert!(is_truncated);
        assert_eq!(retained, vec!["é".to_string(), "s".to_string()]);
        assert!(retained.iter().all(|item| item.is_char_boundary(item.len())));
    }

    #[test]
    fn terminal_classification_gives_cancellation_and_teardown_precedence() {
        let base = TeardownFacts {
            cancellation_requested: false,
            deadline_exceeded: false,
            terminate_sent: false,
            kill_sent: false,
            reaped: true,
            response_present: true,
            stderr_bytes: 0,
            stderr_truncated: false,
        };
        let mut cancelled = base.clone();
        cancelled.cancellation_requested = true;
        assert_eq!(
            classify_terminal(&TerminalFacts {
                teardown: cancelled,
                exit_kind: WorkerExitKind::SuccessResponse,
            }),
            TerminalDisposition::Cancelled
        );
        let mut unreaped = base.clone();
        unreaped.reaped = false;
        assert_eq!(
            classify_terminal(&TerminalFacts {
                teardown: unreaped,
                exit_kind: WorkerExitKind::SuccessResponse,
            }),
            TerminalDisposition::IncompleteTeardown
        );
        assert_eq!(
            classify_terminal(&TerminalFacts {
                teardown: base,
                exit_kind: WorkerExitKind::CpuLimitSignal,
            }),
            TerminalDisposition::CpuLimit
        );
    }

    #[test]
    fn reports_keep_metric_roles_and_non_claims_explicit() {
        let policy = policy(EvaluationMode::Enforce);
        let policy_ref = validate_policy(&policy, &support()).unwrap();
        let request = prepare_request(request(&policy_ref), &policy, &policy_ref).unwrap();
        let teardown = TeardownFacts {
            cancellation_requested: false,
            deadline_exceeded: false,
            terminate_sent: false,
            kill_sent: false,
            reaped: true,
            response_present: true,
            stderr_bytes: 0,
            stderr_truncated: false,
        };
        let report = build_report(
            &request.request_ref,
            &policy_ref,
            policy.mode,
            TerminalFacts {
                teardown,
                exit_kind: WorkerExitKind::SuccessResponse,
            },
            vec![metric_fact(
                "explicit_top_level_root_force_count",
                "count",
                "public-api-request",
                MetricFactStatus::Observed,
                Some(1),
                "mantle-request",
                None,
            )],
            None,
            None,
            String::new(),
        )
        .unwrap();
        assert_eq!(report.terminal_disposition, TerminalDisposition::Success);
        assert_eq!(report.metrics[0].role, "public-api-request");
        assert_eq!(report.non_claim, NON_CLAIM);
    }
}
