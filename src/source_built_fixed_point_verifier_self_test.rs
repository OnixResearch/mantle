//! Guard self-checks for the independently built portable receipt verifier.
//! This module belongs to the library/checker, not the Mantle execution binaries.
use std::fs;
use std::path::Path;

use crate::errors::RunError;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::ExecutionEventKind;
use crate::source_built_fixed_point_trust_report::ACTION_JOURNAL_FILE;
use crate::source_built_fixed_point_trust_report::FINAL_RECEIPT_FILE;
use crate::source_built_fixed_point_trust_report::evidence_path;
use crate::source_built_fixed_point_trust_report::reject_development_authority;
use crate::source_built_fixed_point_trust_report::report_error;
use crate::source_built_fixed_point_trust_report::trust_report;
use crate::source_built_fixed_point_trust_report::verify_event_journal;
use crate::source_built_fixed_point_trust_report::verify_source_built_fixed_point_receipt;

fn event_fixture() -> ActionExecutionEvent {
    ActionExecutionEvent {
        action_id: "compile".into(),
        kind: ExecutionEventKind::ProtectedExec,
        normal_exit_code: None,
        executable_digest_blake3: "a".repeat(64),
        executable_producer: None,
        inputs: Vec::new(),
        input_digests_blake3: Vec::new(),
        produced_outputs: Vec::new(),
        locality: "local".into(),
        completed_locally: true,
        cache_hit: false,
        live_fetch: false,
        cargo_invocation: false,
        fallback: false,
        protected_exec_violation: false,
        elapsed_seconds: 1,
        disk_bytes: 1,
    }
}
/// Exercise the real portable verifier's fail-closed guards in a caller-owned
/// empty temporary directory. This is not an executed or accepted proof fixture.
pub fn self_test_bootstrap_trust_report(root: &Path) -> Result<(), RunError> {
    let mut entries = fs::read_dir(root).map_err(|error| report_error(format!("reading self-test root: {error}")))?;
    if entries.next().transpose().map_err(|error| report_error(error.to_string()))?.is_some() {
        return Err(report_error("self-test requires an empty temporary directory".into()));
    }
    let receipt = root.join(FINAL_RECEIPT_FILE);
    require_self_test_rejection(verify_source_built_fixed_point_receipt(root, &receipt), "empty bundle")?;
    let report = trust_report(root);
    if report.verified || report.status != "blocked" || report.blockers.is_empty() {
        return Err(report_error("self-test empty bundle report did not remain blocked".into()));
    }
    require_self_test_rejection(evidence_path(root, "../outside.json"), "parent path authority")?;
    require_self_test_rejection(evidence_path(root, "/outside.json"), "absolute path authority")?;
    fs::write(&receipt, b"{}")
        .map_err(|error| report_error(format!("writing malformed self-test receipt: {error}")))?;
    require_self_test_rejection(verify_source_built_fixed_point_receipt(root, &receipt), "malformed receipt")?;
    let mut events = vec![event_fixture(), event_fixture()];
    events[1].action_id = "unknown-captured-child".into();
    let mut journal = Vec::new();
    for event in &events {
        serde_json::to_writer(&mut journal, event).map_err(|error| report_error(error.to_string()))?;
        journal.push(b'\n');
    }
    fs::write(root.join(ACTION_JOURNAL_FILE), &journal).map_err(|error| report_error(error.to_string()))?;
    let digest = verify_event_journal(root, &events)?;
    if digest != blake3::hash(&journal).to_hex().as_str() {
        return Err(report_error("self-test journal content hash mismatch".into()));
    }
    require_self_test_rejection(verify_event_journal(root, &events[..1]), "omitted unknown journal event")?;
    let mut altered = events.clone();
    altered[1].executable_digest_blake3 = "b".repeat(64);
    require_self_test_rejection(verify_event_journal(root, &altered), "tampered sealed event")?;
    journal.pop();
    fs::write(root.join(ACTION_JOURNAL_FILE), &journal).map_err(|error| report_error(error.to_string()))?;
    require_self_test_rejection(verify_event_journal(root, &events), "truncated durable journal")?;
    fs::write(root.join("dev-run-report.json"), b"{}").map_err(|error| report_error(error.to_string()))?;
    require_self_test_rejection(reject_development_authority(root), "development authority")?;
    Ok(())
}

fn require_self_test_rejection<T>(result: Result<T, RunError>, case: &str) -> Result<(), RunError> {
    if result.is_ok() {
        return Err(report_error(format!("verifier self-test incorrectly accepted {case}")));
    }
    Ok(())
}
