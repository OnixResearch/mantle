#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
serde = "1.0"
serde_json = "1.0"
---

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde::de::Deserialize;
use serde::de::Deserializer;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::de::{self};
use serde_json::Map;
use serde_json::Number;
use serde_json::Value;

const STREAM_SCHEMA: &str = "mantle-evaluation-stream-v1";
const SCHEMA_PATH: &str = "schemas/evaluation-stream/mantle-evaluation-stream-v1.schema.json";
const FIXTURE_ROOT: &str = "schemas/evaluation-stream/fixtures";
const MAX_STREAM_RECORD_BYTES: u64 = 134_217_728;
const MAX_SELECTED_ROOTS: u32 = 65_536;
const MAX_ROOT_LABEL_BYTES: u64 = 1_024;
const MAX_DIAGNOSTIC_BYTES: u64 = 16_384;
const MAX_SAFE_REFERENCE_BYTES: u64 = 4_096;
const BLAKE3_HEX_BYTES: u64 = 64;
const SUCCESS_EXIT_CODE: u8 = 0;
const PARTIAL_EVAL_EXIT_CODE: u8 = 2;
const FAILED_EVAL_EXIT_CODE: u8 = 2;
const PARTIAL_PIPELINE_EXIT_CODE: u8 = 1;
const FAILED_PIPELINE_EXIT_CODE: u8 = 1;
const CANCELLED_EXIT_CODE: u8 = 130;
const INTERNAL_EXIT_CODE: u8 = 3;
const COMPATIBILITY_MODE: &str = "stream-v1-introduction-plus-one-subsequent-minor-release";
const POSITIVE_FIXTURES: &[&str] = &["all-success.ndjson", "partial.ndjson"];
const NEGATIVE_FIXTURES: &[(&str, &str)] = &[
    ("unknown-version.ndjson", "unsupported-schema-version"),
    ("unknown-kind.ndjson", "unknown-record-kind"),
    ("duplicate-field.ndjson", "duplicate-field:kind"),
    ("oversized-field.ndjson", "root-label-too-large"),
    ("malformed-record.ndjson", "malformed-json"),
];
const RUN_START_FIELDS: &[&str] = &[
    "schema",
    "kind",
    "run_id",
    "source_blake3",
    "evaluator_cohort",
    "selector",
    "selected_root_count",
    "compatibility_mode",
];
const ROOT_DISCOVERED_FIELDS: &[&str] = &["schema", "kind", "run_id", "sequence", "root_id", "root_label"];
const ROOT_TERMINAL_FIELDS: &[&str] = &[
    "schema",
    "kind",
    "run_id",
    "sequence",
    "root_id",
    "root_label",
    "terminal_state",
    "terminal_phase",
    "failure_scope",
    "diagnostic",
    "diagnostic_truncated",
    "result_ref",
    "cache_ref",
    "build_ref",
];
const RUN_SUMMARY_FIELDS: &[&str] = &[
    "schema",
    "kind",
    "run_id",
    "disposition",
    "selected_root_count",
    "terminal_root_count",
    "counts",
    "roots",
];
const ROOT_PROJECTION_FIELDS: &[&str] = &[
    "sequence",
    "root_id",
    "root_label",
    "terminal_state",
    "terminal_phase",
    "failure_scope",
    "diagnostic",
    "diagnostic_truncated",
    "result_ref",
    "cache_ref",
    "build_ref",
];
const COUNT_FIELDS: &[&str] = &["succeeded", "failed", "worker_lost", "cancelled", "not_started"];

#[derive(Clone, Debug, Eq, PartialEq)]
struct RootIdentity {
    root_id: String,
    root_label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RootProjection {
    sequence: u32,
    root_id: String,
    root_label: String,
    terminal_state: String,
    terminal_phase: String,
    failure_scope: Option<String>,
    diagnostic: Option<String>,
    diagnostic_truncated: bool,
    result_ref: Option<String>,
    cache_ref: Option<String>,
    build_ref: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Counts {
    succeeded: u32,
    failed: u32,
    worker_lost: u32,
    cancelled: u32,
    not_started: u32,
}

#[derive(Debug)]
struct StreamState {
    run_id: String,
    selected_root_count: u32,
    discovered: BTreeMap<u32, RootIdentity>,
    terminals: BTreeMap<u32, RootProjection>,
    summary_seen: bool,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("evaluation stream contract check failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: Vec<String>) -> Result<String, String> {
    match arguments.as_slice() {
        [] => run_repository_check(),
        [flag] if flag == "--self-test" => run_self_test(),
        [flag] if flag == "--help" => Err("usage: check-evaluation-stream-contract.rs [--self-test]".to_string()),
        _ => Err("usage: check-evaluation-stream-contract.rs [--self-test]".to_string()),
    }
}

fn run_repository_check() -> Result<String, String> {
    let root = env::current_dir().map_err(|error| format!("current-directory:{error}"))?;
    validate_schema_file(&root.join(SCHEMA_PATH))?;
    for fixture in POSITIVE_FIXTURES {
        let bytes = read_fixture(&root, fixture)?;
        validate_stream(&bytes).map_err(|error| format!("positive-fixture:{fixture}:{error}"))?;
    }
    for (fixture, expected) in NEGATIVE_FIXTURES {
        let bytes = read_fixture(&root, fixture)?;
        let error = validate_stream(&bytes).expect_err("negative fixture must fail");
        if !error.contains(expected) {
            return Err(format!("negative-fixture:{fixture}:expected:{expected}:actual:{error}"));
        }
    }
    Ok(format!(
        "evaluation stream contract check: PASS ({} positive, {} negative)",
        POSITIVE_FIXTURES.len(),
        NEGATIVE_FIXTURES.len()
    ))
}

fn run_self_test() -> Result<String, String> {
    let root = env::current_dir().map_err(|error| format!("current-directory:{error}"))?;
    let positive = read_fixture(&root, POSITIVE_FIXTURES[0])?;
    let disposition = validate_stream(&positive)?;
    if disposition != "success" {
        return Err(format!("self-test-positive-disposition:{disposition}"));
    }
    let duplicate = br#"{"schema":"mantle-evaluation-stream-v1","kind":"run-start","kind":"run-summary"}\n"#;
    let duplicate_error = validate_stream(duplicate).expect_err("duplicate field must fail");
    if !duplicate_error.contains("duplicate-field:kind") {
        return Err(format!("self-test-duplicate:{duplicate_error}"));
    }
    let without_final_newline = positive.get(..positive.len().saturating_sub(1)).unwrap_or(&positive);
    let summary_separator = without_final_newline
        .iter()
        .rposition(|byte| *byte == b'\n')
        .ok_or_else(|| "self-test-summary-separator-missing".to_string())?;
    let incomplete = positive.get(..=summary_separator).ok_or_else(|| "self-test-summary-slice".to_string())?;
    let incomplete_error = validate_stream(incomplete).expect_err("missing summary must fail");
    if !incomplete_error.contains("missing-run-summary") {
        return Err(format!("self-test-summary:{incomplete_error}"));
    }
    validate_process_status_contract()?;
    Ok("evaluation stream contract self-test: PASS".to_string())
}

fn read_fixture(root: &Path, name: &str) -> Result<Vec<u8>, String> {
    let path = root.join(FIXTURE_ROOT).join(name);
    fs::read(&path).map_err(|error| format!("read-fixture:{}:{error}", path.display()))
}

fn validate_schema_file(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| format!("read-schema:{}:{error}", path.display()))?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| format!("schema-json:{error}"))?;
    let object = value.as_object().ok_or_else(|| "schema-root-not-object".to_string())?;
    if object.get("$id").and_then(Value::as_str) != Some("mantle://schemas/evaluation-stream/v1") {
        return Err("schema-id-mismatch".to_string());
    }
    let variants = object.get("oneOf").and_then(Value::as_array).ok_or_else(|| "schema-oneOf-missing".to_string())?;
    if variants.len() != 4 {
        return Err(format!("schema-record-kind-count:{}", variants.len()));
    }
    Ok(())
}

fn validate_stream(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("empty-stream".to_string());
    }
    let mut state: Option<StreamState> = None;
    let mut line_count: u32 = 0;
    for raw_line in bytes.split(|byte| *byte == b'\n') {
        if raw_line.is_empty() {
            continue;
        }
        line_count = line_count.checked_add(1).ok_or_else(|| "line-count-overflow".to_string())?;
        validate_line_bound(raw_line)?;
        let record = parse_unique_json(raw_line)?;
        process_record(record, &mut state)?;
    }
    if line_count == 0 {
        return Err("empty-stream".to_string());
    }
    let state = state.ok_or_else(|| "missing-run-start".to_string())?;
    finish_stream(state)
}

fn validate_line_bound(line: &[u8]) -> Result<(), String> {
    let bytes = u64::try_from(line.len()).map_err(|_| "record-size-overflow".to_string())?;
    let framed_bytes = bytes.checked_add(1).ok_or_else(|| "record-size-overflow".to_string())?;
    if framed_bytes > MAX_STREAM_RECORD_BYTES {
        return Err("record-too-large".to_string());
    }
    Ok(())
}

fn process_record(record: Value, state: &mut Option<StreamState>) -> Result<(), String> {
    let object = record.as_object().ok_or_else(|| "record-not-object".to_string())?;
    validate_schema(object)?;
    let kind = required_string(object, "kind")?;
    if state.as_ref().is_some_and(|current| current.summary_seen) {
        return Err("record-after-run-summary".to_string());
    }
    match kind {
        "run-start" => process_run_start(object, state),
        "root-discovered" => process_root_discovered(object, state),
        "root-terminal" => process_root_terminal(object, state),
        "run-summary" => process_run_summary(object, state),
        _ => Err(format!("unknown-record-kind:{kind}")),
    }
}

fn process_run_start(object: &Map<String, Value>, state: &mut Option<StreamState>) -> Result<(), String> {
    require_exact_fields(object, RUN_START_FIELDS)?;
    if state.is_some() {
        return Err("duplicate-run-start".to_string());
    }
    let run_id = required_digest(object, "run_id")?.to_string();
    required_digest(object, "source_blake3")?;
    required_bounded_text(object, "evaluator_cohort", MAX_SAFE_REFERENCE_BYTES)?;
    required_bounded_text(object, "selector", MAX_SAFE_REFERENCE_BYTES)?;
    let selected_root_count = required_u32(object, "selected_root_count")?;
    if selected_root_count == 0 || selected_root_count > MAX_SELECTED_ROOTS {
        return Err("selected-root-count-out-of-range".to_string());
    }
    if required_string(object, "compatibility_mode")? != COMPATIBILITY_MODE {
        return Err("compatibility-mode-mismatch".to_string());
    }
    *state = Some(StreamState {
        run_id,
        selected_root_count,
        discovered: BTreeMap::new(),
        terminals: BTreeMap::new(),
        summary_seen: false,
    });
    Ok(())
}

fn process_root_discovered(object: &Map<String, Value>, state: &mut Option<StreamState>) -> Result<(), String> {
    require_exact_fields(object, ROOT_DISCOVERED_FIELDS)?;
    let current = current_state(object, state)?;
    let sequence = required_u32(object, "sequence")?;
    validate_sequence(sequence, current.selected_root_count)?;
    let identity = RootIdentity {
        root_id: required_digest(object, "root_id")?.to_string(),
        root_label: required_root_label(object)?.to_string(),
    };
    if current.discovered.values().any(|known| known.root_id == identity.root_id) {
        return Err(format!("duplicate-root-id:{}", identity.root_id));
    }
    if current.discovered.insert(sequence, identity).is_some() {
        return Err(format!("duplicate-root-sequence:{sequence}"));
    }
    Ok(())
}

fn process_root_terminal(object: &Map<String, Value>, state: &mut Option<StreamState>) -> Result<(), String> {
    require_exact_fields(object, ROOT_TERMINAL_FIELDS)?;
    let current = current_state(object, state)?;
    let projection = parse_root_projection(object)?;
    validate_sequence(projection.sequence, current.selected_root_count)?;
    let discovered = current
        .discovered
        .get(&projection.sequence)
        .ok_or_else(|| format!("terminal-before-discovery:{}", projection.sequence))?;
    if discovered.root_id != projection.root_id || discovered.root_label != projection.root_label {
        return Err(format!("terminal-root-identity-mismatch:{}", projection.sequence));
    }
    if current.terminals.insert(projection.sequence, projection).is_some() {
        return Err("duplicate-root-terminal".to_string());
    }
    Ok(())
}

fn process_run_summary(object: &Map<String, Value>, state: &mut Option<StreamState>) -> Result<(), String> {
    require_exact_fields(object, RUN_SUMMARY_FIELDS)?;
    let current = current_state(object, state)?;
    let selected_count = required_u32(object, "selected_root_count")?;
    let terminal_count = required_u32(object, "terminal_root_count")?;
    if selected_count != current.selected_root_count || terminal_count != current.selected_root_count {
        return Err("incomplete-summary-count".to_string());
    }
    let summary_roots = parse_summary_roots(object)?;
    if summary_roots.len() != usize::try_from(current.selected_root_count).expect("u32 fits usize") {
        return Err("incomplete-summary-roots".to_string());
    }
    let counts = parse_counts(object)?;
    validate_summary_facts(current, &summary_roots, counts, required_string(object, "disposition")?)?;
    current.summary_seen = true;
    Ok(())
}

fn current_state<'a>(
    object: &Map<String, Value>,
    state: &'a mut Option<StreamState>,
) -> Result<&'a mut StreamState, String> {
    let current = state.as_mut().ok_or_else(|| "record-before-run-start".to_string())?;
    if required_digest(object, "run_id")? != current.run_id {
        return Err("run-id-mismatch".to_string());
    }
    Ok(current)
}

fn parse_summary_roots(object: &Map<String, Value>) -> Result<Vec<RootProjection>, String> {
    let values = object.get("roots").and_then(Value::as_array).ok_or_else(|| "roots-not-array".to_string())?;
    if values.is_empty() || values.len() > usize::try_from(MAX_SELECTED_ROOTS).expect("u32 fits usize") {
        return Err("summary-roots-out-of-range".to_string());
    }
    let mut roots = Vec::with_capacity(values.len());
    for value in values {
        let root = value.as_object().ok_or_else(|| "summary-root-not-object".to_string())?;
        require_exact_fields(root, ROOT_PROJECTION_FIELDS)?;
        roots.push(parse_root_projection(root)?);
    }
    Ok(roots)
}

fn parse_root_projection(object: &Map<String, Value>) -> Result<RootProjection, String> {
    let projection = RootProjection {
        sequence: required_u32(object, "sequence")?,
        root_id: required_digest(object, "root_id")?.to_string(),
        root_label: required_root_label(object)?.to_string(),
        terminal_state: required_enum(object, "terminal_state", terminal_states())?.to_string(),
        terminal_phase: required_enum(object, "terminal_phase", terminal_phases())?.to_string(),
        failure_scope: optional_enum(object, "failure_scope", failure_scopes())?,
        diagnostic: optional_bounded_text(object, "diagnostic", MAX_DIAGNOSTIC_BYTES)?,
        diagnostic_truncated: required_bool(object, "diagnostic_truncated")?,
        result_ref: optional_bounded_text(object, "result_ref", MAX_SAFE_REFERENCE_BYTES)?,
        cache_ref: optional_bounded_text(object, "cache_ref", MAX_SAFE_REFERENCE_BYTES)?,
        build_ref: optional_bounded_text(object, "build_ref", MAX_SAFE_REFERENCE_BYTES)?,
    };
    validate_terminal_combination(&projection)?;
    Ok(projection)
}

fn validate_terminal_combination(root: &RootProjection) -> Result<(), String> {
    if root.diagnostic_truncated && root.diagnostic.is_none() {
        return Err("truncated-diagnostic-missing".to_string());
    }
    match root.terminal_state.as_str() {
        "succeeded" => {
            if root.failure_scope.is_some() || root.diagnostic.is_some() || root.diagnostic_truncated {
                return Err("successful-root-has-failure".to_string());
            }
            if root.result_ref.is_none() && root.cache_ref.is_none() && root.build_ref.is_none() {
                return Err("successful-root-missing-reference".to_string());
            }
        }
        "failed"
            if !matches!(
                root.failure_scope.as_deref(),
                Some("root-scoped" | "shared-fatal" | "coordinator-failure")
            ) =>
        {
            return Err("failed-root-invalid-scope".to_string());
        }
        "worker-lost" if root.failure_scope.as_deref() != Some("coordinator-failure") => {
            return Err("worker-lost-invalid-scope".to_string());
        }
        "cancelled" if root.failure_scope.as_deref() != Some("cancellation") => {
            return Err("cancelled-root-invalid-scope".to_string());
        }
        "not-started"
            if !matches!(
                root.failure_scope.as_deref(),
                Some("shared-fatal" | "cancellation" | "coordinator-failure")
            ) =>
        {
            return Err("not-started-invalid-scope".to_string());
        }
        _ => {}
    }
    Ok(())
}

fn parse_counts(object: &Map<String, Value>) -> Result<Counts, String> {
    let counts = object.get("counts").and_then(Value::as_object).ok_or_else(|| "counts-not-object".to_string())?;
    require_exact_fields(counts, COUNT_FIELDS)?;
    Ok(Counts {
        succeeded: required_u32(counts, "succeeded")?,
        failed: required_u32(counts, "failed")?,
        worker_lost: required_u32(counts, "worker_lost")?,
        cancelled: required_u32(counts, "cancelled")?,
        not_started: required_u32(counts, "not_started")?,
    })
}

fn validate_summary_facts(
    state: &StreamState,
    summary_roots: &[RootProjection],
    supplied_counts: Counts,
    supplied_disposition: &str,
) -> Result<(), String> {
    for (expected_sequence, root) in summary_roots.iter().enumerate() {
        let expected_sequence =
            u32::try_from(expected_sequence).map_err(|_| "summary-sequence-overflow".to_string())?;
        if root.sequence != expected_sequence {
            return Err("summary-order-not-canonical".to_string());
        }
        if state.terminals.get(&expected_sequence) != Some(root) {
            return Err(format!("summary-terminal-mismatch:{expected_sequence}"));
        }
    }
    let observed_counts = count_roots(summary_roots)?;
    if supplied_counts != observed_counts {
        return Err("summary-counts-mismatch".to_string());
    }
    let cancellation_stopped_dispatch = summary_roots.iter().any(root_has_cancellation_scope);
    let disposition = derive_disposition(observed_counts, state.selected_root_count, cancellation_stopped_dispatch)?;
    if supplied_disposition != disposition {
        return Err(format!("summary-disposition-mismatch:{supplied_disposition}:{disposition}"));
    }
    Ok(())
}

fn count_roots(roots: &[RootProjection]) -> Result<Counts, String> {
    let mut counts = Counts::default();
    for root in roots {
        let target = match root.terminal_state.as_str() {
            "succeeded" => &mut counts.succeeded,
            "failed" => &mut counts.failed,
            "worker-lost" => &mut counts.worker_lost,
            "cancelled" => &mut counts.cancelled,
            "not-started" => &mut counts.not_started,
            state => return Err(format!("unknown-terminal-state:{state}")),
        };
        *target = target.checked_add(1).ok_or_else(|| "terminal-count-overflow".to_string())?;
    }
    Ok(counts)
}

fn derive_disposition(
    counts: Counts,
    selected_root_count: u32,
    cancellation_stopped_dispatch: bool,
) -> Result<&'static str, String> {
    let total = counts
        .succeeded
        .checked_add(counts.failed)
        .and_then(|value| value.checked_add(counts.worker_lost))
        .and_then(|value| value.checked_add(counts.cancelled))
        .and_then(|value| value.checked_add(counts.not_started))
        .ok_or_else(|| "summary-count-overflow".to_string())?;
    if total != selected_root_count {
        return Err("incomplete-summary-count".to_string());
    }
    if counts.cancelled > 0 || cancellation_stopped_dispatch {
        return Ok("cancelled");
    }
    if counts.succeeded == selected_root_count {
        return Ok("success");
    }
    if counts.succeeded > 0 {
        return Ok("partial");
    }
    Ok("failed")
}

fn finish_stream(state: StreamState) -> Result<String, String> {
    if !state.summary_seen {
        return Err("missing-run-summary".to_string());
    }
    if state.discovered.len() != usize::try_from(state.selected_root_count).expect("u32 fits usize") {
        return Err("incomplete-discovery".to_string());
    }
    if state.terminals.len() != usize::try_from(state.selected_root_count).expect("u32 fits usize") {
        return Err("incomplete-terminal-accounting".to_string());
    }
    let roots = state.terminals.into_values().collect::<Vec<_>>();
    let counts = count_roots(&roots)?;
    let cancellation_stopped_dispatch = roots.iter().any(root_has_cancellation_scope);
    Ok(derive_disposition(counts, state.selected_root_count, cancellation_stopped_dispatch)?.to_string())
}

fn root_has_cancellation_scope(root: &RootProjection) -> bool {
    root.failure_scope.as_deref() == Some("cancellation")
}

fn validate_schema(object: &Map<String, Value>) -> Result<(), String> {
    let schema = required_string(object, "schema")?;
    if schema != STREAM_SCHEMA {
        return Err(format!("unsupported-schema-version:{schema}"));
    }
    Ok(())
}

fn validate_sequence(sequence: u32, selected_root_count: u32) -> Result<(), String> {
    if sequence >= selected_root_count {
        return Err(format!("root-sequence-out-of-range:{sequence}"));
    }
    Ok(())
}

fn require_exact_fields(object: &Map<String, Value>, expected: &[&str]) -> Result<(), String> {
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    for field in &expected {
        if !object.contains_key(*field) {
            return Err(format!("missing-field:{field}"));
        }
    }
    for field in object.keys() {
        if !expected.contains(field.as_str()) {
            return Err(format!("unknown-field:{field}"));
        }
    }
    Ok(())
}

fn required_string<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    object.get(field).and_then(Value::as_str).ok_or_else(|| format!("field-not-string:{field}"))
}

fn required_bool(object: &Map<String, Value>, field: &str) -> Result<bool, String> {
    object.get(field).and_then(Value::as_bool).ok_or_else(|| format!("field-not-boolean:{field}"))
}

fn required_u32(object: &Map<String, Value>, field: &str) -> Result<u32, String> {
    let value = object.get(field).and_then(Value::as_u64).ok_or_else(|| format!("field-not-u32:{field}"))?;
    u32::try_from(value).map_err(|_| format!("field-u32-overflow:{field}"))
}

fn required_digest<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    let value = required_string(object, field)?;
    let length = u64::try_from(value.len()).map_err(|_| format!("digest-length-overflow:{field}"))?;
    if length != BLAKE3_HEX_BYTES || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(format!("invalid-blake3:{field}"));
    }
    Ok(value)
}

fn required_root_label(object: &Map<String, Value>) -> Result<&str, String> {
    let label = required_string(object, "root_label")?;
    let bytes = u64::try_from(label.len()).map_err(|_| "root-label-size-overflow".to_string())?;
    if label.is_empty() {
        return Err("root-label-empty".to_string());
    }
    if bytes > MAX_ROOT_LABEL_BYTES {
        return Err("root-label-too-large".to_string());
    }
    Ok(label)
}

fn required_bounded_text<'a>(object: &'a Map<String, Value>, field: &str, bytes_max: u64) -> Result<&'a str, String> {
    let value = required_string(object, field)?;
    validate_text_bound(field, value, bytes_max)?;
    Ok(value)
}

fn optional_bounded_text(object: &Map<String, Value>, field: &str, bytes_max: u64) -> Result<Option<String>, String> {
    let Some(value) = object.get(field) else {
        return Err(format!("missing-field:{field}"));
    };
    if value.is_null() {
        return Ok(None);
    }
    let text = value.as_str().ok_or_else(|| format!("field-not-string-or-null:{field}"))?;
    validate_text_bound(field, text, bytes_max)?;
    Ok(Some(text.to_string()))
}

fn validate_text_bound(field: &str, value: &str, bytes_max: u64) -> Result<(), String> {
    let bytes = u64::try_from(value.len()).map_err(|_| format!("field-size-overflow:{field}"))?;
    if bytes > bytes_max {
        return Err(format!("field-too-large:{field}"));
    }
    Ok(())
}

fn required_enum<'a>(object: &'a Map<String, Value>, field: &str, values: &[&str]) -> Result<&'a str, String> {
    let value = required_string(object, field)?;
    if !values.contains(&value) {
        return Err(format!("unknown-enum:{field}:{value}"));
    }
    Ok(value)
}

fn optional_enum(object: &Map<String, Value>, field: &str, values: &[&str]) -> Result<Option<String>, String> {
    let Some(value) = object.get(field) else {
        return Err(format!("missing-field:{field}"));
    };
    if value.is_null() {
        return Ok(None);
    }
    let text = value.as_str().ok_or_else(|| format!("field-not-string-or-null:{field}"))?;
    if !values.contains(&text) {
        return Err(format!("unknown-enum:{field}:{text}"));
    }
    Ok(Some(text.to_string()))
}

fn terminal_states() -> &'static [&'static str] {
    &["succeeded", "failed", "worker-lost", "cancelled", "not-started"]
}

fn terminal_phases() -> &'static [&'static str] {
    &["evaluation", "conversion", "build", "coordination"]
}

fn failure_scopes() -> &'static [&'static str] {
    &["root-scoped", "shared-fatal", "cancellation", "coordinator-failure"]
}

fn parse_unique_json(bytes: &[u8]) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = UniqueValue::deserialize(&mut deserializer).map_err(|error| map_json_error(&error.to_string()))?.0;
    deserializer.end().map_err(|error| format!("trailing-json:{error}"))?;
    Ok(value)
}

fn map_json_error(error: &str) -> String {
    if let Some(index) = error.find("duplicate-field:") {
        let duplicate = error[index..].split_whitespace().next().unwrap_or("duplicate-field");
        return duplicate.trim_end_matches(',').to_string();
    }
    format!("malformed-json:{error}")
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object fields")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where E: de::Error {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| E::custom("non-finite-json-number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value.to_string())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where D: Deserializer<'de> {
        UniqueValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut fields: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de> {
        let mut object = Map::new();
        let mut names = BTreeSet::new();
        while let Some(name) = fields.next_key::<String>()? {
            if !names.insert(name.clone()) {
                return Err(de::Error::custom(format!("duplicate-field:{name}")));
            }
            let value = fields.next_value::<UniqueValue>()?;
            object.insert(name, value.0);
        }
        Ok(UniqueValue(Value::Object(object)))
    }
}

fn validate_process_status_contract() -> Result<(), String> {
    let evaluation = [
        ("success", SUCCESS_EXIT_CODE),
        ("partial", PARTIAL_EVAL_EXIT_CODE),
        ("failed", FAILED_EVAL_EXIT_CODE),
        ("cancelled", CANCELLED_EXIT_CODE),
    ];
    let pipeline = [
        ("success", SUCCESS_EXIT_CODE),
        ("partial", PARTIAL_PIPELINE_EXIT_CODE),
        ("failed", FAILED_PIPELINE_EXIT_CODE),
        ("cancelled", CANCELLED_EXIT_CODE),
    ];
    for (disposition, expected) in evaluation {
        if process_status(disposition, true)? != expected {
            return Err(format!("evaluation-process-status:{disposition}"));
        }
    }
    for (disposition, expected) in pipeline {
        if process_status(disposition, false)? != expected {
            return Err(format!("pipeline-process-status:{disposition}"));
        }
    }
    if INTERNAL_EXIT_CODE == SUCCESS_EXIT_CODE {
        return Err("internal-status-must-fail".to_string());
    }
    Ok(())
}

fn process_status(disposition: &str, evaluation_mode: bool) -> Result<u8, String> {
    match (evaluation_mode, disposition) {
        (_, "success") => Ok(SUCCESS_EXIT_CODE),
        (true, "partial") => Ok(PARTIAL_EVAL_EXIT_CODE),
        (true, "failed") => Ok(FAILED_EVAL_EXIT_CODE),
        (false, "partial") => Ok(PARTIAL_PIPELINE_EXIT_CODE),
        (false, "failed") => Ok(FAILED_PIPELINE_EXIT_CODE),
        (_, "cancelled") => Ok(CANCELLED_EXIT_CODE),
        _ => Err(format!("unknown-disposition:{disposition}")),
    }
}
