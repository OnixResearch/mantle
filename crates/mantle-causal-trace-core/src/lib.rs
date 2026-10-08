#![no_std]
extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const SCHEMA: &str = "mantle-build-trace-v1";
pub const MAX_ACTIONS: usize = 16_384;
pub const MAX_RECORD_BYTES: usize = 1_024;
pub const MAX_TRACE_BYTES: usize = 4_194_304;
pub const MAX_DIAGNOSTIC_BYTES: usize = 256;
const HASH_HEX_BYTES: usize = 64;

/// Each edge labels the *decision* that brought this action into being, not
/// the outcome or a speculative reconstruction from log order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cause {
    RootRequirement,
    DependencyReady,
    Dispatch,
    CacheDecision,
    Retry,
    Cancellation,
    Cleanup,
    ExternalTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionKind {
    ExternalTrigger,
    RootRequirement,
    DependencyReady,
    CacheCheck,
    CacheHit,
    CacheMiss,
    Dispatch,
    Retry,
    Succeeded,
    Failed,
    Cancellation,
    Cleanup,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub action_id: u32,
    /// Lowercase BLAKE3 of the full logical derivation identity; never a path.
    pub goal_blake3: Option<String>,
    pub kind: ActionKind,
    /// Option permits a precise fail-closed error for missing causes.
    pub cause: Option<Cause>,
    pub caused_by: Option<u32>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    pub schema: String,
    pub records: Vec<Record>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidTrace {
    Schema,
    Empty,
    TooManyActions,
    TooManyBytes,
    OversizedRecord,
    Identity,
    MissingCause,
    DanglingCause,
    InvalidEdge,
    Unredacted,
    Serialization,
}

/// Pure, bounded review. Caller MUST cap incoming bytes before JSON decoding.
/// Serialization is also the canonical byte representation of an admitted trace.
/// r[impl mantle.operator_diagnostics.causal_trace_cause_validation]
pub fn validate(trace: &Trace) -> Result<(), InvalidTrace> {
    if trace.schema != SCHEMA {
        return Err(InvalidTrace::Schema);
    }
    if trace.records.is_empty() {
        return Err(InvalidTrace::Empty);
    }
    if trace.records.len() > MAX_ACTIONS {
        return Err(InvalidTrace::TooManyActions);
    }
    // The fixed ASCII envelope and one comma between adjacent serialized
    // records account for the exact canonical JSON length without allocating
    // a second copy of the entire trace.
    let mut bytes = b"{\"schema\":\"\",\"records\":[]}"
        .len()
        .checked_add(SCHEMA.len())
        .and_then(|length| length.checked_add(trace.records.len().saturating_sub(1)))
        .ok_or(InvalidTrace::TooManyBytes)?;
    // Borrow goal digests from already bounded records: no duplicate identity
    // strings and no inferred dependency relation from incidental interleaving.
    let mut observed_dependencies = BTreeSet::new();
    for (index, record) in trace.records.iter().enumerate() {
        let id = u32::try_from(index).map_err(|_| InvalidTrace::TooManyActions)?;
        if record.action_id != id {
            return Err(InvalidTrace::Identity);
        }
        if record.goal_blake3.as_ref().is_some_and(|goal| goal.len() >= MAX_RECORD_BYTES)
            || record.diagnostic.as_ref().is_some_and(|text| text.len() > MAX_DIAGNOSTIC_BYTES)
        {
            return Err(InvalidTrace::OversizedRecord);
        }
        validate_record(record, &trace.records[..index])?;
        if matches!(record.kind, ActionKind::DependencyReady | ActionKind::Failed) {
            let parent_id = record.caused_by.ok_or(InvalidTrace::MissingCause)?;
            let parent_index = usize::try_from(parent_id).map_err(|_| InvalidTrace::DanglingCause)?;
            let parent = trace.records.get(parent_index).ok_or(InvalidTrace::DanglingCause)?;
            validate_observed_dependency(record, parent, &mut observed_dependencies)?;
        }
        let record_bytes = serde_json::to_vec(record).map_err(|_| InvalidTrace::Serialization)?;
        if record_bytes.len() > MAX_RECORD_BYTES {
            return Err(InvalidTrace::OversizedRecord);
        }
        bytes = bytes.checked_add(record_bytes.len()).ok_or(InvalidTrace::TooManyBytes)?;
        if bytes > MAX_TRACE_BYTES {
            return Err(InvalidTrace::TooManyBytes);
        }
    }
    debug_assert!(trace.records.len() <= MAX_ACTIONS);
    debug_assert!(bytes <= MAX_TRACE_BYTES);
    Ok(())
}

fn validate_observed_dependency<'a>(
    record: &'a Record,
    parent: &'a Record,
    observed_dependencies: &mut BTreeSet<(&'a str, &'a str)>,
) -> Result<(), InvalidTrace> {
    match (record.kind, record.cause, parent.kind) {
        (ActionKind::DependencyReady, _, _) => {
            let dependent = parent.goal_blake3.as_deref().ok_or(InvalidTrace::Identity)?;
            let dependency = record.goal_blake3.as_deref().ok_or(InvalidTrace::Identity)?;
            observed_dependencies.insert((dependent, dependency));
        }
        (ActionKind::Failed, Some(Cause::DependencyReady), ActionKind::Failed) => {
            let dependent = record.goal_blake3.as_deref().ok_or(InvalidTrace::Identity)?;
            let dependency = parent.goal_blake3.as_deref().ok_or(InvalidTrace::Identity)?;
            if !observed_dependencies.contains(&(dependent, dependency)) {
                return Err(InvalidTrace::InvalidEdge);
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_record(record: &Record, previous: &[Record]) -> Result<(), InvalidTrace> {
    if let Some(goal) = &record.goal_blake3 {
        if goal.len() != HASH_HEX_BYTES
            || !goal.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(InvalidTrace::Identity);
        }
    } else if record.kind != ActionKind::ExternalTrigger {
        return Err(InvalidTrace::Identity);
    }
    if record.kind == ActionKind::ExternalTrigger && record.goal_blake3.is_some() {
        return Err(InvalidTrace::Identity);
    }
    if let Some(text) = &record.diagnostic {
        if text.len() > MAX_DIAGNOSTIC_BYTES {
            return Err(InvalidTrace::OversizedRecord);
        }
        if redact_diagnostic(text) != *text {
            return Err(InvalidTrace::Unredacted);
        }
    }
    let cause = record.cause.ok_or(InvalidTrace::MissingCause)?;
    let parent_id = record.caused_by.ok_or(InvalidTrace::MissingCause)?;
    if record.kind == ActionKind::ExternalTrigger {
        if cause != Cause::ExternalTrigger || parent_id != record.action_id || !previous.is_empty() {
            return Err(InvalidTrace::InvalidEdge);
        }
        return Ok(());
    }
    let parent_index = usize::try_from(parent_id).map_err(|_| InvalidTrace::DanglingCause)?;
    let parent = previous.get(parent_index).ok_or(InvalidTrace::DanglingCause)?;
    if !valid_edge(record.kind, cause, parent.kind) {
        return Err(InvalidTrace::InvalidEdge);
    }
    // A typed edge may cross goals only when a newly discovered dependency or
    // its propagated failure is explicitly the cause. Interleaving alone never
    // authorizes attaching one goal's cache/dispatch/retry result to another.
    let is_cross_goal_edge = matches!(
        (record.kind, cause, parent.kind),
        (ActionKind::DependencyReady, Cause::DependencyReady, _)
            | (ActionKind::Failed, Cause::DependencyReady, ActionKind::Failed)
    );
    if record.kind != ActionKind::RootRequirement && ((record.goal_blake3 == parent.goal_blake3) == is_cross_goal_edge)
    {
        return Err(InvalidTrace::Identity);
    }
    debug_assert!(parent.action_id < record.action_id);
    debug_assert!(record.cause.is_some());
    Ok(())
}

fn valid_edge(kind: ActionKind, cause: Cause, parent: ActionKind) -> bool {
    match (kind, cause) {
        (ActionKind::RootRequirement, Cause::RootRequirement) => parent == ActionKind::ExternalTrigger,
        (ActionKind::DependencyReady, Cause::DependencyReady) => {
            matches!(parent, ActionKind::RootRequirement | ActionKind::DependencyReady)
        }
        (ActionKind::CacheCheck, Cause::CacheDecision) => {
            matches!(parent, ActionKind::RootRequirement | ActionKind::DependencyReady | ActionKind::Retry)
        }
        (ActionKind::CacheHit | ActionKind::CacheMiss, Cause::CacheDecision) => parent == ActionKind::CacheCheck,
        (ActionKind::Dispatch, Cause::Dispatch) => matches!(
            parent,
            ActionKind::RootRequirement | ActionKind::DependencyReady | ActionKind::CacheMiss | ActionKind::Retry
        ),
        (ActionKind::Retry, Cause::Retry) => parent == ActionKind::Failed,
        (ActionKind::Failed, Cause::Retry) => parent == ActionKind::Retry,
        (ActionKind::Succeeded | ActionKind::Failed, Cause::Dispatch) => parent == ActionKind::Dispatch,
        (ActionKind::Succeeded, Cause::CacheDecision) => parent == ActionKind::CacheHit,
        (ActionKind::Failed, Cause::CacheDecision) => {
            matches!(parent, ActionKind::CacheHit | ActionKind::CacheMiss | ActionKind::CacheCheck)
        }
        (ActionKind::Failed, Cause::DependencyReady) => {
            matches!(parent, ActionKind::Failed | ActionKind::DependencyReady | ActionKind::RootRequirement)
        }
        (ActionKind::Cancellation, Cause::Cancellation) => parent != ActionKind::Cleanup,
        (ActionKind::Cleanup, Cause::Cleanup) => {
            matches!(parent, ActionKind::Succeeded | ActionKind::Failed | ActionKind::Cancellation)
        }
        _ => false,
    }
}

/// Returns child-first path ending at the external trigger. A shared dependency
/// can have one chosen triggering root; this does not claim all interested roots.
pub fn chain_to_root(trace: &Trace, action_id: u32) -> Result<Vec<u32>, InvalidTrace> {
    validate(trace)?;
    let mut current = action_id;
    let mut chain = Vec::with_capacity(trace.records.len().min(16));
    for _ in 0..trace.records.len() {
        let record_index = usize::try_from(current).map_err(|_| InvalidTrace::DanglingCause)?;
        let record = trace.records.get(record_index).ok_or(InvalidTrace::DanglingCause)?;
        chain.push(current);
        if record.kind == ActionKind::ExternalTrigger {
            return Ok(chain);
        }
        current = record.caused_by.ok_or(InvalidTrace::MissingCause)?;
    }
    Err(InvalidTrace::InvalidEdge)
}

/// Redact using the evaluation-stream diagnostic rules: whitespace-tokenized
/// secret markers, host paths, standalone digests, and bounded UTF-8 text.
pub fn redact_diagnostic(input: &str) -> String {
    let mut result = String::new();
    let scan_limit_bytes = input.len().min(MAX_DIAGNOSTIC_BYTES.saturating_mul(2));
    let boundary = input.floor_char_boundary(scan_limit_bytes);
    for token in input[..boundary].split_whitespace() {
        let bare = token.trim_start_matches(['\'', '"', '(', '[', '{']);
        let is_path = bare.starts_with('/') || bare.starts_with("file://");
        let is_assigned_path = token.contains("=/") || token.contains(":/");
        let rendered = if has_sensitive_marker(token.as_bytes()) {
            "<redacted>"
        } else if is_path || is_assigned_path {
            "<redacted-path>"
        } else if token.len() == HASH_HEX_BYTES && token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            "<redacted-digest>"
        } else {
            token
        };
        if !result.is_empty() {
            result.push(' ');
        }
        if result.len().saturating_add(rendered.len()) > MAX_DIAGNOSTIC_BYTES {
            break;
        }
        result.push_str(rendered);
    }
    result
}

fn has_sensitive_marker(token: &[u8]) -> bool {
    const MARKERS: [&[u8]; 14] = [
        b"secret",
        b"token",
        b"password",
        b"passwd",
        b"credential",
        b"authorization",
        b"bearer",
        b"api_key",
        b"api-key",
        b"access_key",
        b"access-key",
        b"private_key",
        b"private-key",
        b"cookie",
    ];
    MARKERS
        .iter()
        .any(|marker| token.windows(marker.len()).any(|window| window.eq_ignore_ascii_case(marker)))
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    fn record(id: u32, kind: ActionKind, cause: Cause, parent: u32) -> Record {
        Record {
            action_id: id,
            goal_blake3: (id != 0).then(|| "a".repeat(64)),
            kind,
            cause: Some(cause),
            caused_by: Some(parent),
            diagnostic: None,
        }
    }
    fn for_goal(mut record: Record, goal: &str) -> Record {
        record.goal_blake3 = Some(goal.repeat(HASH_HEX_BYTES));
        record
    }

    fn chain() -> Trace {
        let mut trace = Trace {
            schema: SCHEMA.to_string(),
            records: vec![
                record(0, ActionKind::ExternalTrigger, Cause::ExternalTrigger, 0),
                record(1, ActionKind::RootRequirement, Cause::RootRequirement, 0),
                record(2, ActionKind::DependencyReady, Cause::DependencyReady, 1),
                record(3, ActionKind::CacheCheck, Cause::CacheDecision, 2),
                record(4, ActionKind::CacheMiss, Cause::CacheDecision, 3),
                record(5, ActionKind::Dispatch, Cause::Dispatch, 4),
                record(6, ActionKind::Failed, Cause::Dispatch, 5),
                record(7, ActionKind::Failed, Cause::DependencyReady, 6),
            ],
        };
        for record in &mut trace.records[2..7] {
            record.goal_blake3 = Some("b".repeat(HASH_HEX_BYTES));
        }
        trace
    }

    #[test]
    fn dependency_failure_reaches_root_across_parallel_interleaving() {
        let mut trace = chain();
        trace
            .records
            .insert(6, for_goal(record(6, ActionKind::RootRequirement, Cause::RootRequirement, 0), "c"));
        for (index, record) in trace.records.iter_mut().enumerate().skip(7) {
            record.action_id = index as u32;
            if record.caused_by.unwrap() >= 6 {
                record.caused_by = Some(record.caused_by.unwrap().checked_add(1).expect("bounded test action id"));
            }
        }
        assert_eq!(chain_to_root(&trace, 8).unwrap(), vec![8, 7, 5, 4, 3, 2, 1, 0]);
        assert_eq!(chain_to_root(&trace, 6).unwrap(), vec![6, 0]);
    }

    #[test]
    fn cache_hit_and_retry_chain_to_real_admission_and_failure() {
        let mut trace = chain();
        trace.records.truncate(4);
        trace.records.extend([
            for_goal(record(4, ActionKind::CacheHit, Cause::CacheDecision, 3), "b"),
            for_goal(record(5, ActionKind::Succeeded, Cause::CacheDecision, 4), "b"),
        ]);
        assert_eq!(chain_to_root(&trace, 5).unwrap(), vec![5, 4, 3, 2, 1, 0]);
        let mut retry = chain();
        retry.records.extend([
            for_goal(record(8, ActionKind::Retry, Cause::Retry, 6), "b"),
            for_goal(record(9, ActionKind::CacheCheck, Cause::CacheDecision, 8), "b"),
            for_goal(record(10, ActionKind::CacheMiss, Cause::CacheDecision, 9), "b"),
            for_goal(record(11, ActionKind::Dispatch, Cause::Dispatch, 10), "b"),
            for_goal(record(12, ActionKind::Succeeded, Cause::Dispatch, 11), "b"),
            for_goal(record(13, ActionKind::Cleanup, Cause::Cleanup, 12), "b"),
        ]);
        assert_eq!(chain_to_root(&retry, 13).unwrap(), vec![13, 12, 11, 10, 9, 8, 6, 5, 4, 3, 2, 1, 0]);
        assert_eq!(retry.records[11].goal_blake3, retry.records[6].goal_blake3);
    }

    #[test]
    fn failed_retry_preserves_retry_cause_without_forging_dispatch() {
        let mut retry = chain();
        retry.records.push(for_goal(record(8, ActionKind::Retry, Cause::Retry, 6), "b"));
        retry.records.push(for_goal(record(9, ActionKind::Failed, Cause::Retry, 8), "b"));
        assert_eq!(chain_to_root(&retry, 9).unwrap(), vec![9, 8, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn malformed_cause_edges_and_bounds_fail_closed() {
        let mut trace = chain();
        trace.records[3].cause = None;
        assert_eq!(validate(&trace), Err(InvalidTrace::MissingCause));
        trace.records[3].cause = Some(Cause::CacheDecision);
        trace.records[3].caused_by = Some(999);
        assert_eq!(validate(&trace), Err(InvalidTrace::DanglingCause));
        trace.records[3].caused_by = Some(2);
        trace.records[3].diagnostic = Some("token=SHOULD_NOT_LEAK".to_string());
        assert_eq!(validate(&trace), Err(InvalidTrace::Unredacted));
        trace.records[3].diagnostic = None;
        trace.records[3].goal_blake3 = Some("a".repeat(MAX_RECORD_BYTES));
        assert_eq!(validate(&trace), Err(InvalidTrace::OversizedRecord));
        trace.records[3].goal_blake3 = Some("a".repeat(HASH_HEX_BYTES));
        trace.records[3].diagnostic = None;
        trace.records.extend(
            (8..=MAX_ACTIONS as u32).map(|id| record(id, ActionKind::RootRequirement, Cause::RootRequirement, 0)),
        );
        assert_eq!(validate(&trace), Err(InvalidTrace::TooManyActions));
        let mut json = serde_json::to_value(chain()).unwrap();
        json["records"][3]["cause"] = serde_json::json!("new-unknown-cause");
        assert!(serde_json::from_value::<Trace>(json).is_err());
    }
    #[test]
    fn escaped_diagnostic_enforces_serialized_record_bound() {
        let mut trace = chain();
        trace.records[7].diagnostic = Some("\0".repeat(90));
        assert_eq!(validate(&trace), Ok(()));
        trace.records[7].diagnostic = Some("\0".repeat(220));
        assert_eq!(validate(&trace), Err(InvalidTrace::OversizedRecord));
    }

    #[test]
    fn identity_cycle_and_invalid_parent_kind_fail_closed() {
        let mut trace = chain();
        trace.records[2].action_id = 1;
        assert_eq!(validate(&trace), Err(InvalidTrace::Identity));
        trace.records[2].action_id = 2;
        trace.records[2].caused_by = Some(2);
        assert_eq!(validate(&trace), Err(InvalidTrace::DanglingCause));
        trace.records[2].caused_by = Some(0);
        assert_eq!(validate(&trace), Err(InvalidTrace::InvalidEdge));
        trace.records[2].caused_by = Some(1);
        trace.records[2].goal_blake3 = Some("a".repeat(HASH_HEX_BYTES));
        assert_eq!(validate(&trace), Err(InvalidTrace::Identity));
        trace.records[2].goal_blake3 = Some("b".repeat(HASH_HEX_BYTES));
        assert_eq!(validate(&trace), Ok(()));
    }

    #[test]
    fn parallel_interleaving_does_not_forge_same_goal_or_dependency_edges() {
        let mut trace = chain();
        trace
            .records
            .insert(3, for_goal(record(3, ActionKind::RootRequirement, Cause::RootRequirement, 0), "c"));
        for (index, record) in trace.records.iter_mut().enumerate().skip(4) {
            record.action_id = index as u32;
            if record.caused_by.unwrap() >= 3 {
                record.caused_by = Some(record.caused_by.unwrap().checked_add(1).expect("bounded test action id"));
            }
        }
        assert_eq!(chain_to_root(&trace, 8).unwrap(), vec![8, 7, 6, 5, 4, 2, 1, 0]);
        trace.records[4].caused_by = Some(3);
        assert_eq!(validate(&trace), Err(InvalidTrace::Identity));
        trace.records[4].caused_by = Some(2);
        trace.records[6].caused_by = Some(3);
        assert_eq!(validate(&trace), Err(InvalidTrace::Identity));
        trace.records[6].caused_by = Some(5);
        trace.records[8].goal_blake3 = trace.records[7].goal_blake3.clone();
        assert_eq!(validate(&trace), Err(InvalidTrace::Identity));
        trace.records[8].goal_blake3 = Some("a".repeat(HASH_HEX_BYTES));
        trace.records[2].goal_blake3 = trace.records[1].goal_blake3.clone();
        assert_eq!(validate(&trace), Err(InvalidTrace::Identity));
        trace.records[2].goal_blake3 = Some("b".repeat(HASH_HEX_BYTES));
        trace.records[0].goal_blake3 = Some("c".repeat(HASH_HEX_BYTES));
        assert_eq!(validate(&trace), Err(InvalidTrace::Identity));
    }

    #[test]
    fn unrelated_failed_goal_cannot_be_spliced_into_root_failure_chain() {
        let mut trace = chain();
        trace
            .records
            .insert(3, for_goal(record(3, ActionKind::RootRequirement, Cause::RootRequirement, 0), "c"));
        for (index, record) in trace.records.iter_mut().enumerate().skip(4) {
            record.action_id = index as u32;
            if record.caused_by.unwrap() >= 3 {
                record.caused_by = Some(record.caused_by.unwrap().checked_add(1).expect("bounded test action id"));
            }
        }
        trace.records.insert(8, for_goal(record(8, ActionKind::Failed, Cause::DependencyReady, 3), "c"));
        trace.records[9].action_id = 9;
        assert_eq!(chain_to_root(&trace, 9).unwrap(), vec![9, 7, 6, 5, 4, 2, 1, 0]);
        trace.records[9].caused_by = Some(8);
        assert_eq!(validate(&trace), Err(InvalidTrace::InvalidEdge));
    }

    #[test]
    fn multi_hop_dependency_failure_uses_each_observed_direct_edge() {
        let trace = Trace {
            schema: SCHEMA.to_string(),
            records: vec![
                record(0, ActionKind::ExternalTrigger, Cause::ExternalTrigger, 0),
                record(1, ActionKind::RootRequirement, Cause::RootRequirement, 0),
                for_goal(record(2, ActionKind::DependencyReady, Cause::DependencyReady, 1), "b"),
                for_goal(record(3, ActionKind::DependencyReady, Cause::DependencyReady, 2), "c"),
                for_goal(record(4, ActionKind::CacheCheck, Cause::CacheDecision, 3), "c"),
                for_goal(record(5, ActionKind::CacheMiss, Cause::CacheDecision, 4), "c"),
                for_goal(record(6, ActionKind::Dispatch, Cause::Dispatch, 5), "c"),
                for_goal(record(7, ActionKind::Failed, Cause::Dispatch, 6), "c"),
                for_goal(record(8, ActionKind::Failed, Cause::DependencyReady, 7), "b"),
                record(9, ActionKind::Failed, Cause::DependencyReady, 8),
            ],
        };
        assert_eq!(chain_to_root(&trace, 9).unwrap(), vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
        let mut forged = trace;
        forged.records[9].caused_by = Some(7);
        assert_eq!(validate(&forged), Err(InvalidTrace::InvalidEdge));
    }

    #[test]
    fn aggregate_byte_bound_rejects_many_individually_valid_actions() {
        let mut trace = chain();
        for id in 8..12_008_u32 {
            let mut next = record(id, ActionKind::RootRequirement, Cause::RootRequirement, 0);
            next.diagnostic = Some("x".repeat(MAX_DIAGNOSTIC_BYTES));
            trace.records.push(next);
        }
        assert!(trace.records.len() < MAX_ACTIONS);
        assert_eq!(validate(&trace), Err(InvalidTrace::TooManyBytes));
    }

    #[test]
    fn absolute_paths_and_standalone_digests_redact_before_admission() {
        let raw = alloc::format!(
            "/home/operator/private TOKEN=secret path=/tmp/private API_KEY=private {} detail",
            "f".repeat(64)
        );
        let cleaned = redact_diagnostic(&raw);
        assert_eq!(cleaned, "<redacted-path> <redacted> <redacted-path> <redacted> <redacted-digest> detail");
        let mut trace = chain();
        trace.records[7].diagnostic = Some("path=/tmp/private".to_string());
        assert_eq!(validate(&trace), Err(InvalidTrace::Unredacted));
        trace.records[7].diagnostic = Some("API_KEY=private".to_string());
        assert_eq!(validate(&trace), Err(InvalidTrace::Unredacted));
        trace.records[7].diagnostic = Some("f".repeat(64));
        assert_eq!(validate(&trace), Err(InvalidTrace::Unredacted));
        trace.records[7].diagnostic = Some(cleaned);
        assert_eq!(validate(&trace), Ok(()));
    }
}
