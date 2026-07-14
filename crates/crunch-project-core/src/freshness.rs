use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use serde::Deserialize;
use serde::Serialize;

pub const FRESHNESS_PROBE_VERSION: u32 = 1;
pub const MAX_FRESHNESS_INPUT_NAME_BYTES: u32 = 256;
pub const MAX_FRESHNESS_VALUE_BYTES: u32 = 4096;
pub const MAX_FRESHNESS_DIAGNOSTIC_BYTES: u32 = 1024;
pub const MAX_FRESHNESS_TEMPLATE_BYTES: u32 = 8192;
pub const MAX_FRESHNESS_RENDERED_TEMPLATE_BYTES: u32 = 8192;
pub const MAX_FRESHNESS_REFRESH_INPUTS: u32 = 4096;
pub const MAX_COMMAND_ARG_COUNT: u32 = 64;
pub const MAX_COMMAND_ARG_BYTES: u32 = 1024;
pub const MAX_COMMAND_ENV_COUNT: u32 = 64;
pub const MAX_COMMAND_ENV_NAME_BYTES: u32 = 128;
pub const MAX_COMMAND_ENV_VALUE_BYTES: u32 = 4096;
pub const MAX_COMMAND_SUCCESS_STATUS_COUNT: u32 = 32;
pub const MAX_COMMAND_EXIT_STATUS: i32 = 255;
pub const MAX_COMMAND_TIMEOUT_MS: u32 = 3_600_000;
pub const MAX_COMMAND_OUTPUT_BYTES: u32 = 1_048_576;

const _: () = {
    assert!(MAX_COMMAND_ARG_COUNT > 0);
    assert!(MAX_COMMAND_OUTPUT_BYTES >= MAX_FRESHNESS_VALUE_BYTES);
    assert!(MAX_FRESHNESS_VALUE_BYTES > 0);
    assert!(MAX_FRESHNESS_DIAGNOSTIC_BYTES > 0);
    assert!(MAX_FRESHNESS_REFRESH_INPUTS >= crate::MAX_INPUTS);
    assert!(MAX_FRESHNESS_INPUT_NAME_BYTES > 0);
    assert!(MAX_FRESHNESS_TEMPLATE_BYTES > 0);
    assert!(MAX_FRESHNESS_RENDERED_TEMPLATE_BYTES > 0);
};

const BLAKE3_DIGEST_PREFIX: &str = "blake3:";
const TEMPLATE_OPEN: &str = "{{";
const TEMPLATE_CLOSE: &str = "}}";
const TEMPLATE_DELIMITER_BYTES: usize = 2;
const TEMPLATE_VAR_FRESHNESS_VALUE: &str = "freshness.value";
const TEMPLATE_VAR_FRESHNESS_DIGEST: &str = "freshness.value_digest";
const TEMPLATE_VAR_INPUT_NAME: &str = "input.name";
const URL_SCHEME_SEPARATOR: &str = "://";
const DIAGNOSTIC_TRUNCATED_SUFFIX: &str = "...[truncated]";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessProbeKind {
    GitRef,
    HttpText,
    HttpJson,
    LocalFile,
    LocalDirectory,
    Command,
}

impl FreshnessProbeKind {
    pub fn requires_network(self) -> bool {
        matches!(self, Self::GitRef | Self::HttpText | Self::HttpJson)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FreshnessProbe {
    GitRef {
        repository: String,
        reference: String,
    },
    HttpText {
        url: String,
    },
    HttpJson {
        url: String,
        pointer: String,
    },
    LocalFile {
        path: String,
    },
    LocalDirectory {
        path: String,
    },
    Command {
        command: CommandFreshnessProbe,
        requires_network: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandFreshnessProbe {
    pub argv: Vec<String>,
    pub cwd: String,
    pub env: Vec<CommandFreshnessEnv>,
    pub timeout_ms: u32,
    pub output_limit_bytes: u32,
    pub success_statuses: Vec<i32>,
    pub utf8_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandFreshnessEnv {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessObservationStatus {
    Observed,
    Failed,
    Skipped,
    NetworkRequired,
    ProbeUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessObservationRequest {
    pub version: u32,
    pub input_name: String,
    pub probe_kind: FreshnessProbeKind,
    pub requires_network: bool,
    pub status: FreshnessObservationStatus,
    pub value: Option<String>,
    pub diagnostic: String,
    pub probe_identity_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessObservation {
    pub version: u32,
    pub input_name: String,
    pub probe_kind: FreshnessProbeKind,
    pub requires_network: bool,
    pub status: FreshnessObservationStatus,
    pub value: Option<String>,
    pub value_digest: Option<String>,
    pub diagnostic: String,
    pub probe_identity_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessProbeValidation {
    pub version: u32,
    pub kind: FreshnessProbeKind,
    pub requires_network: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedFreshnessValue {
    pub input_name: String,
    pub value_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessRefreshPlanRequest {
    pub declared_inputs: Vec<String>,
    pub selected: Vec<String>,
    pub locked_values: Vec<LockedFreshnessValue>,
    pub observations: Vec<FreshnessObservation>,
    pub no_network: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessDecisionKind {
    Stale,
    Unchanged,
    Failed,
    Skipped,
    NetworkRequired,
    MissingObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessDecision {
    pub input_name: String,
    pub kind: FreshnessDecisionKind,
    pub observed_value_digest: Option<String>,
    pub locked_value_digest: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessTemplateDestination {
    Plain,
    Url,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessTemplateRequest {
    pub input_name: String,
    pub template: String,
    pub observation: FreshnessObservation,
    pub destination: FreshnessTemplateDestination,
    pub max_output_bytes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FreshnessErrorKind {
    UnsupportedVersion,
    EmptyInputName,
    InputNameTooLong,
    InvalidProbe,
    MissingValue,
    UnexpectedValue,
    EmptyValue,
    ValueTooLong,
    EmptyDiagnostic,
    DuplicateInput,
    UnknownInput,
    DuplicateObservation,
    DuplicateLockedValue,
    MissingObservedDigest,
    OfflineNetworkObservation,
    InvalidTemplate,
    UnknownTemplateVariable,
    RenderedTemplateTooLong,
    InvalidRenderedTemplate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessError {
    pub kind: FreshnessErrorKind,
    pub message: String,
}

impl fmt::Display for FreshnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn validate_freshness_probe(probe: FreshnessProbe) -> Result<FreshnessProbeValidation, FreshnessError> {
    let validation = FreshnessProbeValidation {
        version: FRESHNESS_PROBE_VERSION,
        kind: probe_kind(&probe),
        requires_network: probe_requires_network(&probe),
    };
    match &probe {
        FreshnessProbe::GitRef { repository, reference } => {
            require_non_empty(repository, "git repository")?;
            require_non_empty(reference, "git reference")?;
        }
        FreshnessProbe::HttpText { url } => require_url_like(url, "http text URL")?,
        FreshnessProbe::HttpJson { url, pointer } => {
            require_url_like(url, "http JSON URL")?;
            require_non_empty(pointer, "http JSON pointer")?;
        }
        FreshnessProbe::LocalFile { path } => require_non_empty(path, "local file path")?,
        FreshnessProbe::LocalDirectory { path } => require_non_empty(path, "local directory path")?,
        FreshnessProbe::Command { command, .. } => validate_command_probe(command)?,
    }
    Ok(validation)
}

pub fn normalize_freshness_observation(
    request: FreshnessObservationRequest,
) -> Result<FreshnessObservation, FreshnessError> {
    validate_version(request.version)?;
    validate_input_name(&request.input_name)?;
    let diagnostic = bounded_diagnostic(request.diagnostic);
    let value_digest = normalized_value_digest(request.status, request.value.as_deref())?;
    validate_status_payload(request.status, request.value.as_ref(), &diagnostic)?;
    Ok(FreshnessObservation {
        version: request.version,
        input_name: request.input_name,
        probe_kind: request.probe_kind,
        requires_network: request.requires_network,
        status: request.status,
        value: request.value,
        value_digest,
        diagnostic,
        probe_identity_digest: request.probe_identity_digest,
    })
}

pub fn plan_freshness_refresh(request: FreshnessRefreshPlanRequest) -> Result<Vec<FreshnessDecision>, FreshnessError> {
    if request.declared_inputs.len() as u64 > MAX_FRESHNESS_REFRESH_INPUTS as u64 {
        return Err(error(FreshnessErrorKind::InvalidProbe, "too many freshness inputs"));
    }
    let declared = validate_declared_inputs(&request.declared_inputs)?;
    let selected = selected_inputs(&declared, &request.selected)?;
    let locked = locked_map(request.locked_values, &declared)?;
    let observations = observation_map(request.observations, &declared, request.no_network)?;
    Ok(selected
        .into_iter()
        .map(|input_name| classify_one(input_name, locked.get(input_name), observations.get(input_name)))
        .collect())
}

pub fn render_freshness_template(request: FreshnessTemplateRequest) -> Result<String, FreshnessError> {
    validate_input_name(&request.input_name)?;
    if request.input_name != request.observation.input_name {
        return Err(error(FreshnessErrorKind::UnknownInput, "template input name does not match observation"));
    }
    if request.observation.status != FreshnessObservationStatus::Observed {
        return Err(error(FreshnessErrorKind::MissingValue, "template requires an observed freshness value"));
    }
    if request.template.len() as u64 > MAX_FRESHNESS_TEMPLATE_BYTES as u64 {
        return Err(error(FreshnessErrorKind::InvalidTemplate, "freshness template is too large"));
    }
    if request.max_output_bytes == 0 || request.max_output_bytes > MAX_FRESHNESS_RENDERED_TEMPLATE_BYTES {
        return Err(error(FreshnessErrorKind::RenderedTemplateTooLong, "invalid rendered template output bound"));
    }
    let rendered = render_template_bytes(&request)?;
    validate_rendered_template(&rendered, request.destination)?;
    Ok(rendered)
}

fn probe_kind(probe: &FreshnessProbe) -> FreshnessProbeKind {
    match probe {
        FreshnessProbe::GitRef { .. } => FreshnessProbeKind::GitRef,
        FreshnessProbe::HttpText { .. } => FreshnessProbeKind::HttpText,
        FreshnessProbe::HttpJson { .. } => FreshnessProbeKind::HttpJson,
        FreshnessProbe::LocalFile { .. } => FreshnessProbeKind::LocalFile,
        FreshnessProbe::LocalDirectory { .. } => FreshnessProbeKind::LocalDirectory,
        FreshnessProbe::Command { .. } => FreshnessProbeKind::Command,
    }
}

fn probe_requires_network(probe: &FreshnessProbe) -> bool {
    match probe {
        FreshnessProbe::Command { requires_network, .. } => *requires_network,
        _ => probe_kind(probe).requires_network(),
    }
}

fn validate_command_probe(command: &CommandFreshnessProbe) -> Result<(), FreshnessError> {
    if command.argv.is_empty() {
        return Err(error(FreshnessErrorKind::InvalidProbe, "command probe argv must not be empty"));
    }
    if command.argv.len() as u64 > MAX_COMMAND_ARG_COUNT as u64 {
        return Err(error(FreshnessErrorKind::InvalidProbe, "command probe has too many argv entries"));
    }
    require_non_empty(&command.cwd, "command probe cwd")?;
    validate_command_strings(&command.argv, "command probe argv", MAX_COMMAND_ARG_BYTES)?;
    validate_command_env(&command.env)?;
    validate_command_limits(command)
}

fn validate_command_limits(command: &CommandFreshnessProbe) -> Result<(), FreshnessError> {
    if command.timeout_ms == 0 || command.timeout_ms > MAX_COMMAND_TIMEOUT_MS {
        return Err(error(FreshnessErrorKind::InvalidProbe, "command probe timeout is outside the allowed range"));
    }
    if command.output_limit_bytes == 0 || command.output_limit_bytes > MAX_COMMAND_OUTPUT_BYTES {
        return Err(error(FreshnessErrorKind::InvalidProbe, "command probe output limit is outside the allowed range"));
    }
    if command.success_statuses.is_empty() {
        return Err(error(FreshnessErrorKind::InvalidProbe, "command probe success statuses must not be empty"));
    }
    if command.success_statuses.len() as u64 > MAX_COMMAND_SUCCESS_STATUS_COUNT as u64 {
        return Err(error(FreshnessErrorKind::InvalidProbe, "command probe has too many success statuses"));
    }
    for status in &command.success_statuses {
        if *status < 0 || *status > MAX_COMMAND_EXIT_STATUS {
            return Err(error(
                FreshnessErrorKind::InvalidProbe,
                "command probe success status is outside the allowed range",
            ));
        }
    }
    Ok(())
}

fn validate_command_env(env: &[CommandFreshnessEnv]) -> Result<(), FreshnessError> {
    if env.len() as u64 > MAX_COMMAND_ENV_COUNT as u64 {
        return Err(error(FreshnessErrorKind::InvalidProbe, "command probe has too many env entries"));
    }
    let mut seen = BTreeSet::new();
    for entry in env {
        require_non_empty(&entry.name, "command probe env name")?;
        if entry.name.len() as u64 > MAX_COMMAND_ENV_NAME_BYTES as u64 {
            return Err(error(FreshnessErrorKind::InvalidProbe, "command probe env name is too large"));
        }
        if entry.value.len() as u64 > MAX_COMMAND_ENV_VALUE_BYTES as u64 {
            return Err(error(FreshnessErrorKind::InvalidProbe, "command probe env value is too large"));
        }
        if entry.name.contains('=') || entry.name.contains('\0') || entry.value.contains('\0') {
            return Err(error(FreshnessErrorKind::InvalidProbe, "command probe env contains invalid characters"));
        }
        if !seen.insert(entry.name.as_str()) {
            return Err(error(FreshnessErrorKind::InvalidProbe, "command probe env contains duplicate names"));
        }
    }
    Ok(())
}

fn validate_command_strings(values: &[String], label: &str, max_bytes: u32) -> Result<(), FreshnessError> {
    for value in values {
        require_non_empty(value, label)?;
        if value.len() as u64 > max_bytes as u64 {
            return Err(error(FreshnessErrorKind::InvalidProbe, format!("{label} entry is too large")));
        }
        if value.contains('\0') {
            return Err(error(FreshnessErrorKind::InvalidProbe, format!("{label} entry contains NUL")));
        }
    }
    Ok(())
}

fn validate_version(version: u32) -> Result<(), FreshnessError> {
    if version != FRESHNESS_PROBE_VERSION {
        return Err(error(FreshnessErrorKind::UnsupportedVersion, "unsupported freshness probe version"));
    }
    Ok(())
}

fn validate_input_name(name: &str) -> Result<(), FreshnessError> {
    if name.is_empty() {
        return Err(error(FreshnessErrorKind::EmptyInputName, "freshness input name must not be empty"));
    }
    if name.len() as u64 > MAX_FRESHNESS_INPUT_NAME_BYTES as u64 {
        return Err(error(FreshnessErrorKind::InputNameTooLong, "freshness input name is too large"));
    }
    Ok(())
}

fn normalized_value_digest(
    status: FreshnessObservationStatus,
    value: Option<&str>,
) -> Result<Option<String>, FreshnessError> {
    if status != FreshnessObservationStatus::Observed {
        return Ok(None);
    }
    let value = value.ok_or_else(|| error(FreshnessErrorKind::MissingValue, "observed freshness value is missing"))?;
    if value.is_empty() {
        return Err(error(FreshnessErrorKind::EmptyValue, "observed freshness value must not be empty"));
    }
    if value.len() as u64 > MAX_FRESHNESS_VALUE_BYTES as u64 {
        return Err(error(FreshnessErrorKind::ValueTooLong, "observed freshness value is too large"));
    }
    Ok(Some(blake3_digest(value.as_bytes())))
}

fn validate_status_payload(
    status: FreshnessObservationStatus,
    value: Option<&String>,
    diagnostic: &str,
) -> Result<(), FreshnessError> {
    match status {
        FreshnessObservationStatus::Observed => Ok(()),
        FreshnessObservationStatus::Failed | FreshnessObservationStatus::ProbeUnavailable => {
            if value.is_some() {
                return Err(error(
                    FreshnessErrorKind::UnexpectedValue,
                    "failed freshness observation must not carry a value",
                ));
            }
            if diagnostic.is_empty() {
                return Err(error(
                    FreshnessErrorKind::EmptyDiagnostic,
                    "failed freshness observation needs a diagnostic",
                ));
            }
            Ok(())
        }
        FreshnessObservationStatus::Skipped | FreshnessObservationStatus::NetworkRequired => {
            if value.is_some() {
                return Err(error(
                    FreshnessErrorKind::UnexpectedValue,
                    "non-observed freshness status must not carry a value",
                ));
            }
            Ok(())
        }
    }
}

fn bounded_diagnostic(value: String) -> String {
    if value.len() as u64 <= MAX_FRESHNESS_DIAGNOSTIC_BYTES as u64 {
        return value;
    }
    let max_without_suffix =
        (MAX_FRESHNESS_DIAGNOSTIC_BYTES as usize).saturating_sub(DIAGNOSTIC_TRUNCATED_SUFFIX.len());
    let mut output = String::new();
    for ch in value.chars() {
        if output.len().saturating_add(ch.len_utf8()) > max_without_suffix {
            break;
        }
        output.push(ch);
    }
    output.push_str(DIAGNOSTIC_TRUNCATED_SUFFIX);
    output
}

fn validate_declared_inputs(inputs: &[String]) -> Result<BTreeSet<&str>, FreshnessError> {
    let mut declared = BTreeSet::new();
    for input in inputs {
        validate_input_name(input)?;
        if !declared.insert(input.as_str()) {
            return Err(error(FreshnessErrorKind::DuplicateInput, "duplicate declared freshness input"));
        }
    }
    Ok(declared)
}

fn selected_inputs<'a>(
    declared: &'a BTreeSet<&'a str>,
    selected: &'a [String],
) -> Result<Vec<&'a str>, FreshnessError> {
    if selected.is_empty() {
        return Ok(declared.iter().copied().collect());
    }
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for input in selected {
        validate_input_name(input)?;
        if !declared.contains(input.as_str()) {
            return Err(error(
                FreshnessErrorKind::UnknownInput,
                format!("selected freshness input '{input}' is not declared"),
            ));
        }
        if seen.insert(input.as_str()) {
            result.push(input.as_str());
        }
    }
    Ok(result)
}

fn locked_map(
    values: Vec<LockedFreshnessValue>,
    declared: &BTreeSet<&str>,
) -> Result<BTreeMap<String, LockedFreshnessValue>, FreshnessError> {
    let mut map = BTreeMap::new();
    for value in values {
        validate_input_name(&value.input_name)?;
        if !declared.contains(value.input_name.as_str()) {
            return Err(error(FreshnessErrorKind::UnknownInput, "locked freshness value references an unknown input"));
        }
        if value.value_digest.is_empty() {
            return Err(error(FreshnessErrorKind::MissingObservedDigest, "locked freshness value digest is empty"));
        }
        if map.insert(value.input_name.clone(), value).is_some() {
            return Err(error(FreshnessErrorKind::DuplicateLockedValue, "duplicate locked freshness value"));
        }
    }
    Ok(map)
}

fn observation_map(
    observations: Vec<FreshnessObservation>,
    declared: &BTreeSet<&str>,
    no_network: bool,
) -> Result<BTreeMap<String, FreshnessObservation>, FreshnessError> {
    let mut map = BTreeMap::new();
    for observation in observations {
        validate_observation_for_plan(&observation, declared, no_network)?;
        if map.insert(observation.input_name.clone(), observation).is_some() {
            return Err(error(FreshnessErrorKind::DuplicateObservation, "duplicate freshness observation"));
        }
    }
    Ok(map)
}

fn validate_observation_for_plan(
    observation: &FreshnessObservation,
    declared: &BTreeSet<&str>,
    no_network: bool,
) -> Result<(), FreshnessError> {
    validate_version(observation.version)?;
    validate_input_name(&observation.input_name)?;
    if !declared.contains(observation.input_name.as_str()) {
        return Err(error(FreshnessErrorKind::UnknownInput, "freshness observation references an unknown input"));
    }
    if observation.status == FreshnessObservationStatus::Observed && observation.value_digest.is_none() {
        return Err(error(FreshnessErrorKind::MissingObservedDigest, "observed freshness value digest is missing"));
    }
    if no_network
        && observation.status == FreshnessObservationStatus::Observed
        && (observation.requires_network || observation.probe_kind.requires_network())
    {
        return Err(error(
            FreshnessErrorKind::OfflineNetworkObservation,
            "network freshness probe was observed in no-network mode",
        ));
    }
    Ok(())
}

fn classify_one(
    input_name: &str,
    locked: Option<&LockedFreshnessValue>,
    observation: Option<&FreshnessObservation>,
) -> FreshnessDecision {
    match observation {
        None => decision(
            input_name,
            FreshnessDecisionKind::MissingObservation,
            None,
            locked,
            "missing freshness observation",
        ),
        Some(observation) => classify_observation(input_name, locked, observation),
    }
}

fn classify_observation(
    input_name: &str,
    locked: Option<&LockedFreshnessValue>,
    observation: &FreshnessObservation,
) -> FreshnessDecision {
    match observation.status {
        FreshnessObservationStatus::Observed => classify_observed(input_name, locked, observation),
        FreshnessObservationStatus::Failed | FreshnessObservationStatus::ProbeUnavailable => decision(
            input_name,
            FreshnessDecisionKind::Failed,
            observation.value_digest.clone(),
            locked,
            observation.diagnostic.clone(),
        ),
        FreshnessObservationStatus::Skipped => {
            decision(input_name, FreshnessDecisionKind::Skipped, None, locked, observation.diagnostic.clone())
        }
        FreshnessObservationStatus::NetworkRequired => {
            decision(input_name, FreshnessDecisionKind::NetworkRequired, None, locked, observation.diagnostic.clone())
        }
    }
}

fn classify_observed(
    input_name: &str,
    locked: Option<&LockedFreshnessValue>,
    observation: &FreshnessObservation,
) -> FreshnessDecision {
    let observed_digest = observation.value_digest.clone();
    match locked {
        Some(locked) if Some(locked.value_digest.as_str()) == observed_digest.as_deref() => decision(
            input_name,
            FreshnessDecisionKind::Unchanged,
            observed_digest,
            Some(locked),
            "freshness value matches lock".to_string(),
        ),
        _ => decision(
            input_name,
            FreshnessDecisionKind::Stale,
            observed_digest,
            locked,
            "freshness value differs from lock".to_string(),
        ),
    }
}

fn decision(
    input_name: &str,
    kind: FreshnessDecisionKind,
    observed_value_digest: Option<String>,
    locked: Option<&LockedFreshnessValue>,
    reason: impl Into<String>,
) -> FreshnessDecision {
    FreshnessDecision {
        input_name: input_name.to_string(),
        kind,
        observed_value_digest,
        locked_value_digest: locked.map(|value| value.value_digest.clone()),
        reason: reason.into(),
    }
}

fn render_template_bytes(request: &FreshnessTemplateRequest) -> Result<String, FreshnessError> {
    let mut rendered = String::new();
    let mut remaining = request.template.as_str();
    loop {
        match remaining.find(TEMPLATE_OPEN) {
            None => {
                reject_unmatched_close(remaining)?;
                rendered.push_str(remaining);
                enforce_rendered_bound(&rendered, request.max_output_bytes)?;
                return Ok(rendered);
            }
            Some(open_index) => {
                let prefix = &remaining[..open_index];
                reject_unmatched_close(prefix)?;
                rendered.push_str(prefix);
                let after_open = &remaining[open_index + TEMPLATE_DELIMITER_BYTES..];
                let close_index = after_open.find(TEMPLATE_CLOSE).ok_or_else(|| {
                    error(FreshnessErrorKind::InvalidTemplate, "freshness template has an unclosed variable")
                })?;
                let var = after_open[..close_index].trim();
                rendered.push_str(template_value(var, request)?);
                enforce_rendered_bound(&rendered, request.max_output_bytes)?;
                remaining = &after_open[close_index + TEMPLATE_DELIMITER_BYTES..];
            }
        }
    }
}

fn template_value<'a>(var: &str, request: &'a FreshnessTemplateRequest) -> Result<&'a str, FreshnessError> {
    match var {
        TEMPLATE_VAR_FRESHNESS_VALUE => request
            .observation
            .value
            .as_deref()
            .ok_or_else(|| error(FreshnessErrorKind::MissingValue, "freshness value is missing for template")),
        TEMPLATE_VAR_FRESHNESS_DIGEST => request.observation.value_digest.as_deref().ok_or_else(|| {
            error(FreshnessErrorKind::MissingObservedDigest, "freshness digest is missing for template")
        }),
        TEMPLATE_VAR_INPUT_NAME => Ok(request.input_name.as_str()),
        _ => Err(error(
            FreshnessErrorKind::UnknownTemplateVariable,
            format!("unknown freshness template variable '{var}'"),
        )),
    }
}

fn validate_rendered_template(value: &str, destination: FreshnessTemplateDestination) -> Result<(), FreshnessError> {
    if value.is_empty() {
        return Err(error(FreshnessErrorKind::InvalidRenderedTemplate, "rendered freshness template is empty"));
    }
    match destination {
        FreshnessTemplateDestination::Plain => Ok(()),
        FreshnessTemplateDestination::Url => validate_rendered_url(value),
        FreshnessTemplateDestination::Reference => validate_rendered_reference(value),
    }
}

fn validate_rendered_url(value: &str) -> Result<(), FreshnessError> {
    if !value.contains(URL_SCHEME_SEPARATOR) {
        return Err(error(FreshnessErrorKind::InvalidRenderedTemplate, "rendered URL lacks a scheme"));
    }
    if has_ascii_space_or_control(value) {
        return Err(error(FreshnessErrorKind::InvalidRenderedTemplate, "rendered URL contains invalid characters"));
    }
    Ok(())
}

fn validate_rendered_reference(value: &str) -> Result<(), FreshnessError> {
    if has_ascii_space_or_control(value) {
        return Err(error(
            FreshnessErrorKind::InvalidRenderedTemplate,
            "rendered reference contains invalid characters",
        ));
    }
    Ok(())
}

fn reject_unmatched_close(value: &str) -> Result<(), FreshnessError> {
    if value.contains(TEMPLATE_CLOSE) {
        return Err(error(
            FreshnessErrorKind::InvalidTemplate,
            "freshness template has an unmatched closing delimiter",
        ));
    }
    Ok(())
}

fn enforce_rendered_bound(value: &str, max_output_bytes: u32) -> Result<(), FreshnessError> {
    if value.len() as u64 > max_output_bytes as u64 {
        return Err(error(FreshnessErrorKind::RenderedTemplateTooLong, "rendered freshness template is too large"));
    }
    Ok(())
}

fn has_ascii_space_or_control(value: &str) -> bool {
    value.chars().any(|ch| ch.is_ascii_whitespace() || ch.is_ascii_control())
}

fn require_url_like(value: &str, label: &str) -> Result<(), FreshnessError> {
    require_non_empty(value, label)?;
    if !value.contains(URL_SCHEME_SEPARATOR) {
        return Err(error(FreshnessErrorKind::InvalidProbe, format!("{label} lacks a URL scheme")));
    }
    Ok(())
}

fn require_non_empty(value: &str, label: &str) -> Result<(), FreshnessError> {
    if value.is_empty() {
        return Err(error(FreshnessErrorKind::InvalidProbe, format!("{label} must not be empty")));
    }
    Ok(())
}

fn blake3_digest(bytes: &[u8]) -> String {
    format!("{BLAKE3_DIGEST_PREFIX}{}", blake3::hash(bytes).to_hex())
}

fn error(kind: FreshnessErrorKind, message: impl Into<String>) -> FreshnessError {
    FreshnessError {
        kind,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    const INPUT_NAME: &str = "pkg";
    const OLD_VALUE: &str = "1.0.0";
    const NEW_VALUE: &str = "1.1.0";
    const COMMAND_TIMEOUT_MS: u32 = 10_000;
    const COMMAND_OUTPUT_LIMIT_BYTES: u32 = 1024;

    fn observed(name: &str, kind: FreshnessProbeKind, value: &str) -> FreshnessObservation {
        normalize_freshness_observation(FreshnessObservationRequest {
            version: FRESHNESS_PROBE_VERSION,
            input_name: name.to_string(),
            probe_kind: kind,
            requires_network: kind.requires_network(),
            status: FreshnessObservationStatus::Observed,
            value: Some(value.to_string()),
            diagnostic: String::new(),
            probe_identity_digest: Some("blake3:probe".to_string()),
        })
        .unwrap()
    }

    fn failed(name: &str, reason: &str) -> FreshnessObservation {
        normalize_freshness_observation(FreshnessObservationRequest {
            version: FRESHNESS_PROBE_VERSION,
            input_name: name.to_string(),
            probe_kind: FreshnessProbeKind::Command,
            requires_network: false,
            status: FreshnessObservationStatus::Failed,
            value: None,
            diagnostic: reason.to_string(),
            probe_identity_digest: None,
        })
        .unwrap()
    }

    fn locked(name: &str, value: &str) -> LockedFreshnessValue {
        LockedFreshnessValue {
            input_name: name.to_string(),
            value_digest: blake3_digest(value.as_bytes()),
        }
    }

    fn command_probe() -> CommandFreshnessProbe {
        CommandFreshnessProbe {
            argv: vec!["fresh-tool".to_string(), "--version".to_string()],
            cwd: "/workspace".to_string(),
            env: vec![CommandFreshnessEnv {
                name: "LANG".to_string(),
                value: "C".to_string(),
            }],
            timeout_ms: COMMAND_TIMEOUT_MS,
            output_limit_bytes: COMMAND_OUTPUT_LIMIT_BYTES,
            success_statuses: vec![0],
            utf8_required: true,
        }
    }

    #[test]
    fn builtin_observation_normalizes_digest_and_kind() {
        let observation = observed(INPUT_NAME, FreshnessProbeKind::GitRef, NEW_VALUE);

        assert_eq!(observation.version, FRESHNESS_PROBE_VERSION);
        assert_eq!(observation.input_name, INPUT_NAME);
        assert_eq!(observation.probe_kind, FreshnessProbeKind::GitRef);
        assert_eq!(observation.value.as_deref(), Some(NEW_VALUE));
        assert_eq!(observation.value_digest.as_deref(), Some(blake3_digest(NEW_VALUE.as_bytes()).as_str()));
    }

    #[test]
    fn command_probe_validation_accepts_bounded_contract() {
        let probe = FreshnessProbe::Command {
            command: command_probe(),
            requires_network: false,
        };

        let validation = validate_freshness_probe(probe).unwrap();

        assert_eq!(validation.version, FRESHNESS_PROBE_VERSION);
        assert_eq!(validation.kind, FreshnessProbeKind::Command);
        assert!(!validation.requires_network);
    }

    #[test]
    fn freshness_plan_classifies_selected_stale_and_unchanged() {
        let observations = vec![
            observed("stale", FreshnessProbeKind::LocalFile, NEW_VALUE),
            observed("same", FreshnessProbeKind::LocalFile, OLD_VALUE),
            observed("ignored", FreshnessProbeKind::LocalFile, NEW_VALUE),
        ];
        let decisions = plan_freshness_refresh(FreshnessRefreshPlanRequest {
            declared_inputs: vec!["stale".to_string(), "same".to_string(), "ignored".to_string()],
            selected: vec!["stale".to_string(), "same".to_string()],
            locked_values: vec![locked("stale", OLD_VALUE), locked("same", OLD_VALUE)],
            observations,
            no_network: false,
        })
        .unwrap();

        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].input_name, "stale");
        assert_eq!(decisions[0].kind, FreshnessDecisionKind::Stale);
        assert_eq!(decisions[1].input_name, "same");
        assert_eq!(decisions[1].kind, FreshnessDecisionKind::Unchanged);
    }

    #[test]
    fn template_renders_validated_freshness_value() {
        let observation = observed(INPUT_NAME, FreshnessProbeKind::HttpJson, "v1.2.3");
        let rendered = render_freshness_template(FreshnessTemplateRequest {
            input_name: INPUT_NAME.to_string(),
            template: "https://example.com/{{ input.name }}/{{ freshness.value }}.tar.gz".to_string(),
            observation,
            destination: FreshnessTemplateDestination::Url,
            max_output_bytes: MAX_FRESHNESS_RENDERED_TEMPLATE_BYTES,
        })
        .unwrap();

        assert_eq!(rendered, "https://example.com/pkg/v1.2.3.tar.gz");
        assert!(rendered.contains(URL_SCHEME_SEPARATOR));
    }

    #[test]
    fn empty_observed_value_is_rejected() {
        let err = normalize_freshness_observation(FreshnessObservationRequest {
            version: FRESHNESS_PROBE_VERSION,
            input_name: INPUT_NAME.to_string(),
            probe_kind: FreshnessProbeKind::LocalFile,
            requires_network: false,
            status: FreshnessObservationStatus::Observed,
            value: Some(String::new()),
            diagnostic: String::new(),
            probe_identity_digest: None,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::EmptyValue);
        assert!(err.message.contains("must not be empty"));
    }

    #[test]
    fn oversized_observed_value_is_rejected() {
        let too_large = "x".repeat(MAX_FRESHNESS_VALUE_BYTES as usize + 1);
        let err = normalize_freshness_observation(FreshnessObservationRequest {
            version: FRESHNESS_PROBE_VERSION,
            input_name: INPUT_NAME.to_string(),
            probe_kind: FreshnessProbeKind::LocalFile,
            requires_network: false,
            status: FreshnessObservationStatus::Observed,
            value: Some(too_large),
            diagnostic: String::new(),
            probe_identity_digest: None,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::ValueTooLong);
        assert!(err.message.contains("too large"));
    }

    #[test]
    fn invalid_template_variable_is_rejected() {
        let err = render_freshness_template(FreshnessTemplateRequest {
            input_name: INPUT_NAME.to_string(),
            template: "refs/tags/{{ freshness.missing }}".to_string(),
            observation: observed(INPUT_NAME, FreshnessProbeKind::GitRef, "v1"),
            destination: FreshnessTemplateDestination::Reference,
            max_output_bytes: MAX_FRESHNESS_RENDERED_TEMPLATE_BYTES,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::UnknownTemplateVariable);
        assert!(err.message.contains("unknown"));
    }

    #[test]
    fn failed_observation_classifies_without_lock_mutation_claim() {
        let decisions = plan_freshness_refresh(FreshnessRefreshPlanRequest {
            declared_inputs: vec![INPUT_NAME.to_string()],
            selected: Vec::new(),
            locked_values: vec![locked(INPUT_NAME, OLD_VALUE)],
            observations: vec![failed(INPUT_NAME, "command exited 2")],
            no_network: false,
        })
        .unwrap();

        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].kind, FreshnessDecisionKind::Failed);
        assert_eq!(
            decisions[0].locked_value_digest.as_deref(),
            Some(locked(INPUT_NAME, OLD_VALUE).value_digest.as_str())
        );
        assert!(decisions[0].reason.contains("command exited"));
    }

    #[test]
    fn network_required_observation_is_classified_in_offline_mode() {
        let observation = normalize_freshness_observation(FreshnessObservationRequest {
            version: FRESHNESS_PROBE_VERSION,
            input_name: INPUT_NAME.to_string(),
            probe_kind: FreshnessProbeKind::HttpText,
            requires_network: true,
            status: FreshnessObservationStatus::NetworkRequired,
            value: None,
            diagnostic: "network disabled".to_string(),
            probe_identity_digest: None,
        })
        .unwrap();
        let decisions = plan_freshness_refresh(FreshnessRefreshPlanRequest {
            declared_inputs: vec![INPUT_NAME.to_string()],
            selected: Vec::new(),
            locked_values: vec![locked(INPUT_NAME, OLD_VALUE)],
            observations: vec![observation],
            no_network: true,
        })
        .unwrap();

        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].kind, FreshnessDecisionKind::NetworkRequired);
        assert_eq!(decisions[0].observed_value_digest, None);
    }

    #[test]
    fn observed_network_probe_in_offline_mode_is_rejected() {
        let err = plan_freshness_refresh(FreshnessRefreshPlanRequest {
            declared_inputs: vec![INPUT_NAME.to_string()],
            selected: Vec::new(),
            locked_values: Vec::new(),
            observations: vec![observed(INPUT_NAME, FreshnessProbeKind::HttpText, NEW_VALUE)],
            no_network: true,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::OfflineNetworkObservation);
        assert!(err.message.contains("no-network"));
    }

    #[test]
    fn observed_network_command_probe_in_offline_mode_is_rejected() {
        let observation = normalize_freshness_observation(FreshnessObservationRequest {
            version: FRESHNESS_PROBE_VERSION,
            input_name: INPUT_NAME.to_string(),
            probe_kind: FreshnessProbeKind::Command,
            requires_network: true,
            status: FreshnessObservationStatus::Observed,
            value: Some(NEW_VALUE.to_string()),
            diagnostic: String::new(),
            probe_identity_digest: None,
        })
        .unwrap();
        let err = plan_freshness_refresh(FreshnessRefreshPlanRequest {
            declared_inputs: vec![INPUT_NAME.to_string()],
            selected: Vec::new(),
            locked_values: Vec::new(),
            observations: vec![observation],
            no_network: true,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::OfflineNetworkObservation);
        assert!(err.message.contains("no-network"));
    }

    #[test]
    fn observation_for_unknown_input_is_rejected() {
        let err = plan_freshness_refresh(FreshnessRefreshPlanRequest {
            declared_inputs: vec![INPUT_NAME.to_string()],
            selected: Vec::new(),
            locked_values: Vec::new(),
            observations: vec![observed("other", FreshnessProbeKind::LocalDirectory, NEW_VALUE)],
            no_network: false,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::UnknownInput);
        assert!(err.message.contains("unknown input"));
    }

    #[test]
    fn command_probe_without_argv_is_rejected() {
        let mut command = command_probe();
        command.argv.clear();
        let err = validate_freshness_probe(FreshnessProbe::Command {
            command,
            requires_network: false,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::InvalidProbe);
        assert!(err.message.contains("argv"));
    }

    #[test]
    fn diagnostics_are_bounded_deterministically() {
        let diagnostic = "x".repeat(MAX_FRESHNESS_DIAGNOSTIC_BYTES as usize + 128);
        let observation = normalize_freshness_observation(FreshnessObservationRequest {
            version: FRESHNESS_PROBE_VERSION,
            input_name: INPUT_NAME.to_string(),
            probe_kind: FreshnessProbeKind::Command,
            requires_network: false,
            status: FreshnessObservationStatus::Failed,
            value: None,
            diagnostic,
            probe_identity_digest: None,
        })
        .unwrap();

        assert_eq!(observation.diagnostic.len(), MAX_FRESHNESS_DIAGNOSTIC_BYTES as usize);
        assert!(observation.diagnostic.ends_with(DIAGNOSTIC_TRUNCATED_SUFFIX));
    }

    #[test]
    fn rendered_template_bound_is_enforced() {
        let observation = observed(INPUT_NAME, FreshnessProbeKind::HttpText, NEW_VALUE);
        let err = render_freshness_template(FreshnessTemplateRequest {
            input_name: INPUT_NAME.to_string(),
            template: format!("https://example.com/{}/{{{{ freshness.value }}}}", "x".repeat(128)),
            observation,
            destination: FreshnessTemplateDestination::Url,
            max_output_bytes: 32,
        })
        .unwrap_err();

        assert_eq!(err.kind, FreshnessErrorKind::RenderedTemplateTooLong);
        assert!(err.message.contains("too large"));
    }
}
