use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

use crate::Error;
use crate::Identity;
use crate::MAX_DIAGNOSTICS;
use crate::MAX_FIELD_BYTES;
use crate::MAX_OBSERVATION_BYTES;
use crate::safe_key;

const MAX_UNITS: usize = 4096;
const MAX_DIAGNOSTIC_DEPTH: u32 = 16;
const MAX_SPANS: usize = 4096;
const MAX_PROGRAM_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Success,
    Rejected,
    Crash,
    Cancelled,
    Timeout,
    Infrastructure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitRecords {
    pub unit: String,
    pub records: Vec<Value>,
}

/// Shell retains raw streams/artifact digests separately. These are the admitted
/// semantic records, never synthesized from a human Cargo progress log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub status: Status,
    pub units: Vec<UnitRecords>,
    pub program_stdout: Option<Vec<u8>>,
    pub program_stderr: Option<Vec<u8>>,
    pub accepted_outputs: Vec<Identity>,
    pub complete: bool,
    pub teardown_complete: bool,
    pub ice: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    pub status: Status,
    pub diagnostic_codes: Vec<String>,
    pub diagnostic_messages: Vec<String>,
    pub program_stdout: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Equivalent,
    Mismatch,
    Blocked,
    Unsupported,
    HarnessError,
}

impl Observation {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_OBSERVATION_BYTES {
            return Err(Error::Bound);
        }
        let result: Self = serde_json::from_slice(bytes).map_err(|_| Error::Malformed)?;
        validate_observation(&result)?;
        Ok(result)
    }
}

// r[impl rust_package_planning.differential_execution.semantics]
pub fn validate_observation(observation: &Observation) -> Result<(), Error> {
    if !observation.complete {
        return Err(Error::IncompleteCapture);
    }
    if !observation.teardown_complete {
        return Err(Error::IncompleteTeardown);
    }
    if observation.ice {
        return Err(Error::Ice);
    }
    if observation.units.len() > MAX_UNITS || observation.accepted_outputs.len() > MAX_UNITS {
        return Err(Error::Bound);
    }
    for bytes in [&observation.program_stdout, &observation.program_stderr].into_iter().flatten() {
        if bytes.len() > MAX_PROGRAM_BYTES {
            return Err(Error::Bound);
        }
    }
    crate::manifest::validate_identities(&observation.accepted_outputs)?;
    if observation.status == Status::Success && observation.accepted_outputs.is_empty() {
        return Err(Error::Missing);
    }
    let mut units = BTreeSet::new();
    let mut diagnostic_count: usize = 0;
    for unit in &observation.units {
        if !safe_key(&unit.unit) || !units.insert(&unit.unit) {
            return Err(Error::Identity);
        }
        diagnostic_count = diagnostic_count.checked_add(unit.records.len()).ok_or(Error::Bound)?;
        if diagnostic_count > MAX_DIAGNOSTICS {
            return Err(Error::Bound);
        }
        for record in &unit.records {
            validate_diagnostic(record, 0)?;
        }
    }
    if observation.status != Status::Success
        && (!observation.accepted_outputs.is_empty()
            || observation.program_stdout.is_some()
            || observation.program_stderr.is_some())
    {
        return Err(Error::Oracle);
    }
    Ok(())
}

/// Matching failures are insufficient: every observation must meet a frozen
/// content-specific ordinary compiler oracle first.
// r[impl rust_package_planning.differential_execution.stateful_campaign]
pub fn validate_oracle(observation: &Observation, expected: &Expected) -> Result<(), Error> {
    validate_observation(observation)?;
    if !matches!(expected.status, Status::Success | Status::Rejected) {
        return Err(Error::Oracle);
    }
    if observation.status != expected.status {
        return Err(Error::Oracle);
    }
    if expected.diagnostic_codes.len() > MAX_DIAGNOSTICS {
        return Err(Error::Bound);
    }
    let mut codes = Vec::new();
    let mut messages = Vec::<String>::new();
    let mut errors = 0_u32;
    let mut ordered_units: Vec<_> = observation.units.iter().collect();
    ordered_units.sort_by(|a, b| a.unit.cmp(&b.unit));
    for unit in ordered_units {
        for record in &unit.records {
            if record.get("level").and_then(Value::as_str) == Some("error") {
                errors = errors.checked_add(1).ok_or(Error::Bound)?;
                if let Some(code) = record.get("code").and_then(|v| v.get("code")).and_then(Value::as_str) {
                    codes.push(String::from(code));
                    messages.push(record.get("message").and_then(Value::as_str).ok_or(Error::Malformed)?.into());
                }
            }
        }
    }
    if codes != expected.diagnostic_codes || messages != expected.diagnostic_messages {
        return Err(Error::Oracle);
    }
    if expected.status == Status::Rejected && (errors == 0 || expected.diagnostic_codes.is_empty()) {
        return Err(Error::Oracle);
    }
    if expected.status == Status::Success && errors != 0 {
        return Err(Error::Oracle);
    }
    if observation.program_stdout != expected.program_stdout {
        return Err(Error::Oracle);
    }
    if expected.program_stdout.is_some() && observation.program_stderr.as_deref() != Some(&[]) {
        return Err(Error::Oracle);
    }
    Ok(())
}

/// Cross-unit interleaving alone is excluded; per-unit order and multiplicity
/// remain exact. Output policy is fixed by the shell manifest and role labels.
pub fn compare(left: &Observation, right: &Observation, expected: &Expected) -> Verdict {
    if validate_observation(left).is_err() || validate_observation(right).is_err() {
        return Verdict::HarnessError;
    }
    if !matches!(left.status, Status::Success | Status::Rejected)
        || !matches!(right.status, Status::Success | Status::Rejected)
    {
        return Verdict::HarnessError;
    }
    if validate_oracle(left, expected).is_err() || validate_oracle(right, expected).is_err() {
        return Verdict::Mismatch;
    }
    if left.status != right.status
        || left.program_stdout != right.program_stdout
        || left.program_stderr != right.program_stderr
        || left.accepted_outputs != right.accepted_outputs
        || left.units.len() != right.units.len()
    {
        return Verdict::Mismatch;
    }
    for left_unit in &left.units {
        let Some(right_unit) = right.units.iter().find(|u| u.unit == left_unit.unit) else {
            return Verdict::Mismatch;
        };
        if left_unit.records != right_unit.records {
            return Verdict::Mismatch;
        }
    }
    Verdict::Equivalent
}

/// Dedicated fault evidence is not an ordinary equivalent compile rejection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaultEvidence {
    pub expected: Status,
    pub observed: Status,
    pub producer_observed_running: bool,
    pub signal_requested: bool,
    pub producer_signal: Option<i32>,
    pub teardown_complete: bool,
    pub current_outputs_accepted: bool,
    pub prior_success_artifact: Option<crate::Digest>,
}

pub fn validate_fault(evidence: &FaultEvidence) -> Result<(), Error> {
    if !matches!(evidence.expected, Status::Crash | Status::Cancelled | Status::Timeout)
        || evidence.observed != evidence.expected
        || !evidence.producer_observed_running
        || !evidence.signal_requested
        || evidence.producer_signal.is_none()
        || !evidence.teardown_complete
        || evidence.current_outputs_accepted
    {
        return Err(Error::Oracle);
    }
    Ok(())
}

fn object<'a>(value: &'a Value, keys: &[&str]) -> Result<&'a Map<String, Value>, Error> {
    let object = value.as_object().ok_or(Error::Malformed)?;
    if object.keys().any(|key| !keys.contains(&key.as_str())) {
        return Err(Error::UnknownSemanticField);
    }
    Ok(object)
}

fn text(value: Option<&Value>, nullable: bool) -> Result<(), Error> {
    match value {
        Some(Value::Null) if nullable => Ok(()),
        Some(Value::String(text)) if text.len() <= MAX_FIELD_BYTES => Ok(()),
        Some(Value::String(_)) => Err(Error::Bound),
        _ => Err(Error::Malformed),
    }
}

fn array(value: Option<&Value>, maximum: usize) -> Result<&Vec<Value>, Error> {
    let array = value.and_then(Value::as_array).ok_or(Error::Malformed)?;
    if array.len() > maximum {
        return Err(Error::Bound);
    }
    Ok(array)
}

fn validate_diagnostic(value: &Value, depth: u32) -> Result<(), Error> {
    if depth > MAX_DIAGNOSTIC_DEPTH {
        return Err(Error::Bound);
    }
    let diagnostic = object(value, &[
        "message",
        "code",
        "level",
        "spans",
        "children",
        "rendered",
        "$message_type",
    ])?;
    // Cargo retains rustc's optional transport discriminator; it is not an
    // extensible semantic field, and unknown record kinds still fail closed.
    if let Some(message_type) = diagnostic.get("$message_type")
        && message_type.as_str() != Some("diagnostic")
    {
        return Err(Error::Malformed);
    }
    text(diagnostic.get("message"), false)?;
    text(diagnostic.get("level"), false)?;
    let level = diagnostic.get("level").and_then(Value::as_str).ok_or(Error::Malformed)?;
    if !matches!(level, "error" | "warning" | "note" | "help" | "failure-note" | "ice") {
        return Err(Error::Malformed);
    }
    if level == "ice"
        || diagnostic
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|m| m.contains("internal compiler error"))
    {
        return Err(Error::Ice);
    }
    match diagnostic.get("code") {
        Some(Value::Null) => (),
        Some(code) => {
            let code = object(code, &["code", "explanation"])?;
            text(code.get("code"), false)?;
            text(code.get("explanation"), true)?;
        }
        None => return Err(Error::Malformed),
    }
    for span in array(diagnostic.get("spans"), MAX_SPANS)? {
        validate_span(span, depth + 1)?;
    }
    for child in array(diagnostic.get("children"), MAX_DIAGNOSTICS)? {
        validate_diagnostic(child, depth + 1)?;
    }
    // Rustc omits rendered on nested children. If present it is semantic,
    // root-role mapped, and never broadly discarded by the comparator.
    if diagnostic.contains_key("rendered") {
        text(diagnostic.get("rendered"), true)?;
    }
    Ok(())
}

fn validate_span(value: &Value, depth: u32) -> Result<(), Error> {
    if depth > MAX_DIAGNOSTIC_DEPTH {
        return Err(Error::Bound);
    }
    let span = object(value, &[
        "file_name",
        "byte_start",
        "byte_end",
        "line_start",
        "line_end",
        "column_start",
        "column_end",
        "is_primary",
        "text",
        "label",
        "suggested_replacement",
        "suggestion_applicability",
        "expansion",
    ])?;
    text(span.get("file_name"), false)?;
    for key in [
        "byte_start",
        "byte_end",
        "line_start",
        "line_end",
        "column_start",
        "column_end",
    ] {
        if span.get(key).and_then(Value::as_u64).is_none() {
            return Err(Error::Malformed);
        }
    }
    for (start, end) in [("byte_start", "byte_end"), ("line_start", "line_end")] {
        if span.get(start).and_then(Value::as_u64) > span.get(end).and_then(Value::as_u64) {
            return Err(Error::Malformed);
        }
    }
    if span.get("is_primary").and_then(Value::as_bool).is_none() {
        return Err(Error::Malformed);
    }
    for key in ["label", "suggested_replacement", "suggestion_applicability"] {
        text(span.get(key), true)?;
    }
    for item in array(span.get("text"), MAX_SPANS)? {
        let item = object(item, &["text", "highlight_start", "highlight_end"])?;
        text(item.get("text"), false)?;
        let start = item.get("highlight_start").and_then(Value::as_u64).ok_or(Error::Malformed)?;
        let end = item.get("highlight_end").and_then(Value::as_u64).ok_or(Error::Malformed)?;
        if start > end {
            return Err(Error::Malformed);
        }
    }
    match span.get("expansion") {
        Some(Value::Null) => (),
        Some(expansion) => {
            let expansion = object(expansion, &["span", "macro_decl_name", "def_site_span"])?;
            validate_span(expansion.get("span").ok_or(Error::Malformed)?, depth + 1)?;
            text(expansion.get("macro_decl_name"), false)?;
            if let Some(site) = expansion.get("def_site_span")
                && !site.is_null()
            {
                validate_span(site, depth + 1)?;
            }
        }
        None => return Err(Error::Malformed),
    }
    Ok(())
}
