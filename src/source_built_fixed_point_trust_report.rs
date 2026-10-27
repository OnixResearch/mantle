//! Bounded operator view over receipt-bound action evidence, never an authority source.
use std::collections::BTreeSet;
use std::fs;
use std::io::BufRead as _;
use std::io::Read as _;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicBuildProofVerdict;
use crunch_release_core::DeterministicOutputDigest;
use crunch_release_core::deterministic_build_proof_receipt_digest_blake3;

use crate::source_built_fixed_point::ProofOutputRole;

pub(crate) const FINAL_RECEIPT_FILE: &str = "deterministic-build-proof.json";
pub(crate) const FINAL_BUNDLE_DIGEST_FILE: &str = "proof-bundle-blake3.txt";

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::validate_source_built_fixed_point_plan;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::ActionReconciliation;
use crate::source_built_fixed_point_action_trust::ActionTrustPlan;
use crate::source_built_fixed_point_action_trust::ExecutionEventKind;
use crate::source_built_fixed_point_action_trust::validate_action_plan;
use crate::source_built_fixed_point_action_trust::verify_action_reconciliation;

pub(crate) const ACTION_PLAN_FILE: &str = "action-trust-plan.json";
pub(crate) const ACTION_EVENTS_FILE: &str = "action-execution-events.json";
pub(crate) const ACTION_JOURNAL_FILE: &str = "action-execution-events.jsonl";
pub(crate) const ACTION_RECONCILIATION_FILE: &str = "action-reconciliation.json";
pub(crate) const PROOF_PLAN_FILE: &str = "source-built-fixed-point-plan.json";
const JSON_BYTES_MAX: u64 = 512_u64.saturating_mul(1_024).saturating_mul(1_024);
/// Every journal record occupies at least one byte, so the byte bound is also a
/// record-count bound. One extra iteration observes end of file.
const ACTION_JOURNAL_RECORD_LIMIT: u64 = JSON_BYTES_MAX.saturating_add(2);
const RECIPE_JOURNAL_RECORD_LIMIT: u64 = JSON_BYTES_MAX.saturating_add(2);
/// Every read before end of file consumes at least one byte and no file holds
/// `u64::MAX` bytes, so this many reads always reach end of file. It is a
/// runaway guard rather than a policy limit.
const HASH_READ_CHUNK_LIMIT: u64 = u64::MAX;
/// An artifact tree holds at most one million entries, so it holds at most one
/// million directories and never needs more walk steps than that.
const ARTIFACT_TREE_WALK_STEPS: u64 = 1_000_002;
const ARTIFACT_TREE_PENDING_LIMIT: usize = 1_000_000;
const REPORT_TEXT_BYTES_MAX: usize = 4_096;
const REPORT_NON_CLAIMS: &[&str] = &[
    "This report rechecks supplied evidence; it does not execute a proof or authenticate its producer.",
    "Matching binaries and reconciled records do not prove compiler, seed, or kernel correctness.",
    "This report does not establish independent reproducibility, release eligibility, or deployment success.",
    "Absent or invalid evidence is not a successful execution, including after a successful preflight.",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionEvidenceBinding {
    pub(crate) proof_root_identity_blake3: String,
    pub(crate) action_plan_digest_blake3: String,
    pub(crate) action_plan_file_digest_blake3: String,
    pub(crate) observed_events_digest_blake3: String,
    pub(crate) observed_events_file_digest_blake3: String,
    pub(crate) observed_journal_file_digest_blake3: String,
    pub(crate) reconciliation_digest_blake3: String,
    pub(crate) reconciliation_file_digest_blake3: String,
    pub(crate) observed_event_count: u32,
}

pub(crate) struct ActionEvidence {
    pub(crate) plan: ActionTrustPlan,
    pub(crate) events: Vec<ActionExecutionEvent>,
    pub(crate) binding: ActionEvidenceBinding,
}

pub(crate) fn load_action_evidence(root: &Path, proof: &SourceBuiltFixedPointPlan) -> Result<ActionEvidence, RunError> {
    let (plan, plan_file) = read_bound_json::<ActionTrustPlan>(root, ACTION_PLAN_FILE)?;
    let (events, events_file) = read_bound_json::<Vec<ActionExecutionEvent>>(root, ACTION_EVENTS_FILE)?;
    let (reconciliation, reconciliation_file) =
        read_bound_json::<ActionReconciliation>(root, ACTION_RECONCILIATION_FILE)?;
    validate_action_plan(proof, &plan).map_err(report_error)?;
    if events.len() as u64 > u64::from(proof.resource_bounds.protected_exec_events_max) {
        return Err(report_error("sealed execution event count exceeds proof bound".into()));
    }
    let journal_digest = verify_event_journal(root, &events)?;
    verify_action_reconciliation(proof, &plan, &events, &reconciliation).map_err(report_error)?;
    debug_assert_eq!(u64::from(reconciliation.observed_event_count), events.len() as u64);
    debug_assert!(events.len() as u64 <= u64::from(proof.resource_bounds.protected_exec_events_max));
    let binding = ActionEvidenceBinding {
        proof_root_identity_blake3: plan.proof_root_identity_blake3.clone(),
        action_plan_digest_blake3: plan.digest_blake3.clone(),
        action_plan_file_digest_blake3: plan_file,
        observed_events_digest_blake3: reconciliation.observed_event_digest_blake3,
        observed_events_file_digest_blake3: events_file,
        observed_journal_file_digest_blake3: journal_digest,
        reconciliation_digest_blake3: reconciliation.digest_blake3,
        reconciliation_file_digest_blake3: reconciliation_file,
        observed_event_count: reconciliation.observed_event_count,
    };
    Ok(ActionEvidence { plan, events, binding })
}

/// A successful seal must preserve every durably captured record, including
/// rejected/unknown records. Never reconcile a selectable subset of the journal.
pub(crate) fn verify_event_journal(root: &Path, events: &[ActionExecutionEvent]) -> Result<String, RunError> {
    let path = evidence_path(root, ACTION_JOURNAL_FILE)?;
    let file = fs::File::open(path).map_err(|error| report_error(format!("opening action journal: {error}")))?;
    let metadata = file.metadata().map_err(|error| report_error(format!("inspecting action journal: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > JSON_BYTES_MAX {
        return Err(report_error(format!("action journal must be nonempty and within {JSON_BYTES_MAX} bytes")));
    }
    let mut reader = std::io::BufReader::new(file.take(JSON_BYTES_MAX.saturating_add(1)));
    let mut hasher = blake3::Hasher::new();
    let mut line = Vec::new();
    let mut total_bytes = 0_u64;
    let mut record_count = 0_u32;
    for _ in 0..ACTION_JOURNAL_RECORD_LIMIT {
        line.clear();
        let bytes = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| report_error(format!("reading action journal: {error}")))?;
        if bytes == 0 {
            break;
        }
        total_bytes = total_bytes
            .checked_add(bytes as u64)
            .ok_or_else(|| report_error("action journal byte count overflow".into()))?;
        if total_bytes > JSON_BYTES_MAX || line.last() != Some(&b'\n') {
            return Err(report_error("action journal exceeds byte bound or has an incomplete final record".into()));
        }
        let event: ActionExecutionEvent = serde_json::from_slice(&line)
            .map_err(|error| report_error(format!("parsing action journal record {record_count}: {error}")))?;
        let record_index =
            usize::try_from(record_count).map_err(|_| report_error("action journal record count overflow".into()))?;
        if events.get(record_index) != Some(&event) {
            return Err(report_error(format!(
                "action journal record {record_count} differs from sealed events (content, order, or count)"
            )));
        }
        hasher.update(&line);
        record_count = record_count
            .checked_add(1)
            .ok_or_else(|| report_error("action journal record count overflow".into()))?;
    }
    if u64::from(record_count) != events.len() as u64 {
        return Err(report_error("action journal is missing sealed execution events".into()));
    }
    let journal_digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(journal_digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(total_bytes <= JSON_BYTES_MAX);
    Ok(journal_digest)
}

/// Only normal relative components and nonsymlink ancestors may authorize a read.
/// The selected root itself may be an operator-selected symlink (e.g. `latest`).
pub(crate) fn evidence_path(root: &Path, relative: &str) -> Result<std::path::PathBuf, RunError> {
    let path = Path::new(relative);
    if relative.is_empty() || !path.components().all(|part| matches!(part, Component::Normal(_))) {
        return Err(report_error(format!("evidence path is not root-relative: {relative}")));
    }
    let mut resolved = root.to_path_buf();
    for part in path.components() {
        resolved.push(part);
        let metadata = fs::symlink_metadata(&resolved)
            .map_err(|error| report_error(format!("reading evidence {relative}: {error}")))?;
        if metadata.file_type().is_symlink() {
            return Err(report_error(format!("evidence path contains a symlink: {relative}")));
        }
    }
    Ok(resolved)
}

pub(crate) fn read_bound_json<T: DeserializeOwned>(root: &Path, relative: &str) -> Result<(T, String), RunError> {
    let path = evidence_path(root, relative)?;
    let file = fs::File::open(&path).map_err(|error| report_error(format!("opening {relative}: {error}")))?;
    let metadata = file.metadata().map_err(|error| report_error(format!("inspecting {relative}: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > JSON_BYTES_MAX {
        return Err(report_error(format!(
            "{relative} must be a nonempty regular JSON file within {JSON_BYTES_MAX} bytes"
        )));
    }
    let mut bytes = Vec::new();
    file.take(JSON_BYTES_MAX.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| report_error(format!("reading {relative}: {error}")))?;
    if bytes.len() as u64 > JSON_BYTES_MAX {
        return Err(report_error(format!("{relative} exceeded JSON byte bound during read")));
    }
    let digest = blake3::hash(&bytes).to_hex().to_string();
    let value = serde_json::from_slice(&bytes).map_err(|error| report_error(format!("parsing {relative}: {error}")))?;
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(bytes.len() as u64 <= JSON_BYTES_MAX);
    Ok((value, digest))
}

#[derive(Debug, Serialize)]
struct StageCoverage {
    stage_id: String,
    planned_actions: u64,
    protected_planned_min: u64,
    protected_planned_max: u64,
    build_planned_min: u64,
    build_planned_max: u64,
    protected_observed: u64,
    build_observed: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct TrustReport {
    schema: &'static str,
    pub(crate) status: &'static str,
    pub(crate) verified: bool,
    action_plan_valid: bool,
    reconciliation_verified: bool,
    planned_actions: u64,
    observed_events: u64,
    unknown_action_events: u64,
    stages: Vec<StageCoverage>,
    pub(crate) blockers: Vec<String>,
    non_claims: &'static [&'static str],
    #[serde(skip)]
    measured_bundle_entries: u32,
}

fn collect_coverage(plan: &ActionTrustPlan, events: &[ActionExecutionEvent]) -> (Vec<StageCoverage>, u64) {
    let mut known: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let planned_action_count = plan.adapters.iter().map(|adapter| adapter.actions.len()).sum::<usize>();
    known.reserve(planned_action_count.saturating_add(events.len()));
    let mut stages: Vec<StageCoverage> = Vec::with_capacity(plan.adapters.len());
    for adapter in &plan.adapters {
        let mut row = StageCoverage {
            stage_id: adapter.stage_id.clone(),
            planned_actions: adapter.actions.len() as u64,
            protected_planned_min: 0,
            protected_planned_max: 0,
            build_planned_min: 0,
            build_planned_max: 0,
            protected_observed: 0,
            build_observed: 0,
        };
        for action in &adapter.actions {
            known.insert(action.id.clone(), stages.len());
            row.protected_planned_min += u64::from(action.protected_events_min);
            row.protected_planned_max += u64::from(action.protected_events_max);
            row.build_planned_min += u64::from(action.build_events_min);
            row.build_planned_max += u64::from(action.build_events_max);
        }
        for event in events {
            if known.contains_key(&event.action_id) {
                continue;
            }
            if let Ok(Some(action)) =
                crate::source_built_fixed_point_action_trust::adapter_action_spec(adapter, &event.action_id)
            {
                known.insert(action.id.clone(), stages.len());
                row.planned_actions += 1;
                row.protected_planned_min += u64::from(action.protected_events_min);
                row.protected_planned_max += u64::from(action.protected_events_max);
                row.build_planned_min += u64::from(action.build_events_min);
                row.build_planned_max += u64::from(action.build_events_max);
            }
        }
        stages.push(row);
    }
    let mut unknown = 0;
    for event in events {
        if let Some(index) = known.get(event.action_id.as_str()) {
            match event.kind {
                ExecutionEventKind::ProtectedExec => stages[*index].protected_observed += 1,
                ExecutionEventKind::BuildExecution => stages[*index].build_observed += 1,
                ExecutionEventKind::GuardDecision { .. } => {}
            }
        } else {
            unknown += 1;
        }
    }
    debug_assert_eq!(stages.len(), plan.adapters.len());
    debug_assert!(unknown <= events.len() as u64);
    (stages, unknown)
}

fn bounded_blocker(error: impl std::fmt::Display) -> String {
    let mut text = error.to_string();
    if text.len() > REPORT_TEXT_BYTES_MAX {
        let mut end = REPORT_TEXT_BYTES_MAX;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        text.push_str(" [truncated]");
    }
    text
}

fn empty_trust_report() -> TrustReport {
    TrustReport {
        schema: "mantle-source-built-fixed-point-trust-report-v1",
        status: "blocked",
        verified: false,
        action_plan_valid: false,
        reconciliation_verified: false,
        planned_actions: 0,
        observed_events: 0,
        unknown_action_events: 0,
        stages: Vec::new(),
        blockers: Vec::new(),
        non_claims: REPORT_NON_CLAIMS,
        measured_bundle_entries: 0,
    }
}

fn read_trust_proof_plan(root: &Path) -> Result<SourceBuiltFixedPointPlan, RunError> {
    read_bound_json::<SourceBuiltFixedPointPlan>(root, PROOF_PLAN_FILE).and_then(|(proof, _)| {
        validate_source_built_fixed_point_plan(&proof).map_err(|error| report_error(error.to_string()))?;
        Ok(proof)
    })
}

fn load_trust_evidence(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    result_document: &mut TrustReport,
) -> (Option<ActionTrustPlan>, Option<Vec<ActionExecutionEvent>>) {
    let plan = match read_bound_json::<ActionTrustPlan>(root, ACTION_PLAN_FILE).and_then(|(plan, _)| {
        validate_action_plan(proof, &plan).map_err(report_error)?;
        Ok(plan)
    }) {
        Ok(plan) => {
            result_document.action_plan_valid = true;
            Some(plan)
        }
        Err(error) => {
            result_document.blockers.push(bounded_blocker(error));
            None
        }
    };
    let events = match read_bound_json::<Vec<ActionExecutionEvent>>(root, ACTION_EVENTS_FILE) {
        Ok((events, _)) if events.len() as u64 <= u64::from(proof.resource_bounds.protected_exec_events_max) => {
            Some(events)
        }
        Ok(_) => {
            result_document.blockers.push("observed event count exceeds proof bound".into());
            None
        }
        Err(error) => {
            result_document.blockers.push(bounded_blocker(error));
            None
        }
    };
    debug_assert_eq!(result_document.action_plan_valid, plan.is_some());
    debug_assert_eq!(result_document.observed_events, 0);
    (plan, events)
}

fn finish_trust_report(root: &Path, result_document: &mut TrustReport) {
    match verify_source_built_fixed_point_receipt(root, &root.join(FINAL_RECEIPT_FILE)) {
        Ok(bundle) if result_document.blockers.is_empty() => {
            result_document.measured_bundle_entries = bundle.entry_count;
            result_document.verified = true;
            result_document.status = "verified";
        }
        Ok(_) => {}
        Err(error) => result_document.blockers.push(bounded_blocker(error)),
    }
}

pub(crate) fn trust_report(root: &Path) -> TrustReport {
    let mut result_document = empty_trust_report();
    if let Err(error) = reject_development_authority(root) {
        result_document.blockers.push(bounded_blocker(error));
    }
    let proof = match read_trust_proof_plan(root) {
        Ok(proof) => proof,
        Err(error) => {
            result_document.blockers.push(bounded_blocker(error));
            return result_document;
        }
    };
    let (plan, events) = load_trust_evidence(root, &proof, &mut result_document);
    if let Some(plan) = plan.as_ref() {
        let (stages, unknown) = collect_coverage(plan, events.as_deref().unwrap_or(&[]));
        result_document.planned_actions = stages.iter().map(|stage| stage.planned_actions).sum();
        result_document.stages = stages;
        result_document.unknown_action_events = unknown;
    }
    result_document.observed_events = events.as_ref().map_or(0, |events| events.len() as u64);
    if plan.is_some() && events.is_some() {
        match load_action_evidence(root, &proof) {
            Ok(_) => result_document.reconciliation_verified = true,
            Err(error) => result_document.blockers.push(bounded_blocker(error)),
        }
    }
    finish_trust_report(root, &mut result_document);
    debug_assert_eq!(result_document.verified, result_document.status == "verified");
    debug_assert!(result_document.unknown_action_events <= result_document.observed_events);
    result_document
}

// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
pub fn cmd_bootstrap_trust_report(proof_root: &Path, json: bool) -> Result<(), RunError> {
    use mantle_application_contract::ApplicationOutcome;
    use mantle_application_contract::CommandFamily;
    use mantle_application_contract::EffectId;
    use mantle_application_contract::EffectKind;
    use mantle_application_contract::EffectMeasure;
    use mantle_application_contract::EffectOutput;
    use mantle_application_contract::EffectSpec;
    use mantle_application_contract::ExpectedOutput;
    use mantle_application_contract::Observation;
    use mantle_application_contract::ObservationStatus;
    use mantle_application_contract::classify_observations;
    use mantle_application_contract::plan_effects;

    let plan = plan_effects(CommandFamily::Bootstrap, &[EffectSpec {
        effect_id: "bootstrap-source-built-receipt-measurement",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Items(PROOF_ENTRY_COUNT_MAX as u32),
        expected_output: ExpectedOutput::None,
    }])
    .map_err(|error| report_error(format!("planning trust evidence reads: {}", error.code())))?;
    let result_document = trust_report(proof_root);
    let observed = Observation {
        effect_id: EffectId("bootstrap-source-built-receipt-measurement".into()),
        kind: EffectKind::ReadFiles,
        status: if result_document.verified {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: EffectOutput::None,
        usage: EffectMeasure::Items(result_document.measured_bundle_entries),
        diagnostics_code: (!result_document.verified).then(|| "source-built-receipt-verification-failed".into()),
    };
    let result = match classify_observations(&plan, std::slice::from_ref(&observed)) {
        ApplicationOutcome::Completed => Ok(()),
        ApplicationOutcome::Failed { .. } => {
            Err(report_error("receipt-bound action evidence is incomplete or invalid".into()))
        }
        outcome => return Err(report_error(format!("trust evidence observations inconsistent: {outcome:?}"))),
    };
    debug_assert_eq!(result_document.non_claims.len(), REPORT_NON_CLAIMS.len());
    debug_assert!(!result_document.non_claims.is_empty());
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result_document).map_err(|error| report_error(error.to_string()))?
        );
    } else {
        println!("source-built fixed-point trust: {}", result_document.status);
        println!(
            "planned actions: {}; observed events: {}; unknown events: {}",
            result_document.planned_actions, result_document.observed_events, result_document.unknown_action_events
        );
        for blocker in &result_document.blockers {
            println!("blocked: {blocker}");
        }
        for non_claim in REPORT_NON_CLAIMS {
            println!("non-claim: {non_claim}");
        }
    }
    result
}

pub(crate) fn report_error(message: String) -> RunError {
    RunError::Build(format!("source-built action evidence blocked: {message}"))
}

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_proof_is_blocked_not_zero_action_success() {
        let root = tempfile::tempdir().unwrap();
        let report = trust_report(root.path());
        assert!(!report.verified);
        assert!(!report.action_plan_valid);
        assert_eq!(report.status, "blocked");
        assert!(report.blockers[0].contains(PROOF_PLAN_FILE));
    }

    #[test]
    fn journal_rejects_omitted_unknown_reordered_changed_or_truncated_records() {
        let root = tempfile::tempdir().unwrap();
        let launch = event_fixture();
        let mut rejected = launch.clone();
        rejected.action_id = "unknown-child".into();
        let events = vec![launch.clone(), rejected.clone()];
        let mut journal = Vec::new();
        for event in &events {
            serde_json::to_writer(&mut journal, event).unwrap();
            journal.push(b'\n');
        }
        fs::write(root.path().join(ACTION_JOURNAL_FILE), &journal).unwrap();
        assert_eq!(verify_event_journal(root.path(), &events).unwrap(), blake3::hash(&journal).to_hex().to_string());
        assert!(
            verify_event_journal(root.path(), std::slice::from_ref(&launch))
                .unwrap_err()
                .to_string()
                .contains("differs")
        );
        assert!(verify_event_journal(root.path(), &[rejected, launch.clone()]).is_err());
        let mut changed = events.clone();
        changed[1].executable_digest_blake3 = "b".repeat(64);
        assert!(verify_event_journal(root.path(), &changed).is_err());
        assert!(verify_event_journal(root.path(), &[launch.clone(), events[1].clone(), launch]).is_err());
        journal.pop();
        fs::write(root.path().join(ACTION_JOURNAL_FILE), &journal).unwrap();
        assert!(
            verify_event_journal(root.path(), &events)
                .unwrap_err()
                .to_string()
                .contains("incomplete final record")
        );
    }

    #[test]
    fn coverage_counts_each_event_kind_without_hiding_unknown_actions() {
        use crate::source_built_fixed_point_action_trust::ActionExecutable;
        use crate::source_built_fixed_point_action_trust::ActionSpec;
        use crate::source_built_fixed_point_action_trust::StageActionAdapter;
        let plan = ActionTrustPlan {
            schema: "coverage-only-fixture-not-authority".into(),
            proof_plan_digest_blake3: String::new(),
            proof_root_identity_blake3: String::new(),
            digest_blake3: String::new(),
            adapters: vec![StageActionAdapter {
                stage_id: "mantle-stage1".into(),
                reachable_action_ids: vec!["compile".into()],
                controllers: Vec::new(),
                actions: vec![ActionSpec {
                    id: "compile".into(),
                    stage_id: "mantle-stage1".into(),
                    producer_actions: Vec::new(),
                    guard: None,
                    accepted_exit_codes: Vec::new(),
                    executable: ActionExecutable::Fixed {
                        digest_blake3: "a".repeat(64),
                    },
                    inputs: Vec::new(),
                    output_ids: vec!["mantle-stage1".into()],
                    protected_events_min: 1,
                    protected_events_max: 2,
                    build_events_min: 1,
                    build_events_max: 1,
                    elapsed_seconds_max: 1,
                    disk_bytes_max: 1,
                    local_only: true,
                }],
            }],
        };
        let launch = event_fixture();
        let mut build = launch.clone();
        build.kind = ExecutionEventKind::BuildExecution;
        let mut unknown = build.clone();
        unknown.action_id = "undeclared-child".into();
        let (coverage, unknown_count) = collect_coverage(&plan, &[launch.clone(), launch, build, unknown]);
        assert_eq!(coverage[0].protected_observed, 2);
        assert_eq!(coverage[0].build_observed, 1);
        assert_eq!(coverage[0].protected_planned_max, 2);
        assert_eq!(unknown_count, 1);
    }

    #[test]
    fn evidence_reads_reject_absolute_parent_and_symlink_authority() {
        let root = tempfile::tempdir().unwrap();
        assert!(evidence_path(root.path(), "../outside.json").is_err());
        assert!(evidence_path(root.path(), "/outside.json").is_err());
        #[cfg(unix)]
        {
            let outside = tempfile::tempdir().unwrap();
            fs::write(outside.path().join("record.json"), "{}").unwrap();
            std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
            assert!(read_bound_json::<serde_json::Value>(root.path(), "escape/record.json").is_err());
        }
    }
}
pub(crate) const EXTENSION_FIELD: &str = "source_built_fixed_point";
pub(crate) const EXTENSION_SCHEMA: &str = "mantle-source-built-fixed-point-receipt-extension-v2";
pub(crate) const STAGE_EVIDENCE_FILE: &str = "source-built-stage-evidence.json";
pub(crate) const MANTLE_STAGE_MANIFEST_SCHEMA: &str = "mantle-source-built-mantle-stage-v1";
pub(crate) const MANTLE_COMMAND_RECEIPTS_SCHEMA: &str = "mantle-source-built-mantle-command-completions-v1";
pub(crate) const MANTLE_STAGE1_MANIFEST: &str = "cargo-free-fixed-point/stage1/stage-manifest.json";
pub(crate) const MANTLE_STAGE2_MANIFEST: &str = "cargo-free-fixed-point/stage2/stage-manifest.json";
pub(crate) const PROOF_PROVIDER_KIND: &str = "full-source";
pub(crate) const HERMETICITY_MODE: &str = "strict";
pub(crate) const PHYSICAL_STORE_ISOLATION: &str = "clean-namespace-per-run";
pub(crate) const PROOF_TARGET_IDENTITY: &str = "mantle-source-built-fixed-point";
pub(crate) const MANTLE_OUTPUT_NAME: &str = "mantle";
pub(crate) const STAGE1_RUN_ID: &str = "stage1";
pub(crate) const STAGE2_RUN_ID: &str = "stage2";
pub(crate) const PROOF_WORKFLOW: &str = "mantle-deterministic-proof-receipt-v2";
pub(crate) const STAGE_STATUS_COMPLETE: &str = "complete";
pub(crate) const RECEIPT_BUNDLE_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-proof-bundle-v1\0";
pub(crate) const PROVIDER_IDENTITY_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-provider-identity-v1\0";
pub(crate) const BLAKE3_HEX_LENGTH: usize = 64;
pub(crate) const HASH_BUFFER_BYTES: usize = 64 * 1_024;
pub(crate) const PROOF_ENTRY_COUNT_MAX: usize = 2_000_000;
/// The proof bundle walk visits every directory entry the filesystem reports;
/// the entry count, not this step count, bounds the bundle.
const PROOF_BUNDLE_WALK_STEPS: u64 = u64::MAX;
const PROOF_BUNDLE_WALK_DEPTH_LIMIT: usize = PROOF_ENTRY_COUNT_MAX.saturating_add(1);
pub(crate) const STAGE_EVIDENCE_COUNT: usize = 6;
pub(crate) const REQUIRED_RUN_COUNT: usize = 2;
pub(crate) const NON_CLAIMS: &[&str] = &[
    "This proof does not prove compiler correctness or seed correctness.",
    "This proof does not prove kernel isolation or complete Cargo compatibility.",
    "This proof does not prove independent rebuild agreement or bit-for-bit release reproducibility.",
    "This proof does not prove deployment success or release eligibility.",
    "The stage1 Mantle binary is an explicit fixed-point predecessor, not ambient or published-target path authority.",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceBuiltStageEvidence {
    pub(crate) stage_id: String,
    pub(crate) output_role: ProofOutputRole,
    pub(crate) status: String,
    pub(crate) orchestrator: String,
    pub(crate) executable_identity: String,
    pub(crate) transcript_path: String,
    pub(crate) transcript_digest_blake3: String,
    pub(crate) audit_paths: Vec<String>,
    pub(crate) audit_digests_blake3: Vec<String>,
    pub(crate) output_path: String,
    pub(crate) output_digest_blake3: String,
    pub(crate) producer_action_id: String,
    pub(crate) output_id: String,
    pub(crate) execution_journal_path: String,
    pub(crate) execution_journal_digest_blake3: String,
    pub(crate) authority_violations: Vec<String>,
    pub(crate) fallback_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceBuiltReceiptExtension {
    pub(crate) schema: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) source_authority_digest_blake3: String,
    pub(crate) provider_kind: String,
    pub(crate) native_provider_digest_blake3: String,
    pub(crate) rust_provider_digest_blake3: String,
    pub(crate) toolchain_closure_digest_blake3: String,
    pub(crate) cold_provider: ColdProviderEvidence,
    pub(crate) stage_evidence_path: String,
    pub(crate) stage_evidence_digest_blake3: String,
    pub(crate) final_proof_bundle_digest_blake3: String,
    pub(crate) protected_execution_policy_digest_blake3: String,
    pub(crate) effect_policy_digest_blake3: String,
    pub(crate) normalization_policy_digest_blake3: String,
    pub(crate) action_evidence: ActionEvidenceBinding,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ColdArtifactBinding {
    pub(crate) path: String,
    pub(crate) producer_action_id: String,
    pub(crate) output_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ColdProviderEvidence {
    pub(crate) native_provider_path: String,
    pub(crate) rust_provider_path: String,
    pub(crate) rustc: ColdArtifactBinding,
    pub(crate) toolchain_closure: ColdArtifactBinding,
}

pub(crate) struct ColdProviderMeasurement {
    pub(crate) native_digest: String,
    pub(crate) rust_digest: String,
    pub(crate) closure_digest: String,
    pub(crate) provider_size_bytes: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct FixedPointObservation {
    pub(crate) stage1: FixedPointStageObservation,
    pub(crate) stage2: FixedPointStageObservation,
    pub(crate) closure_policy_digest_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct FixedPointStageObservation {
    pub(crate) binary_relative_path: String,
    pub(crate) binary_digest_blake3: String,
    pub(crate) manifest_relative_path: String,
    pub(crate) manifest_digest_blake3: String,
    pub(crate) orchestrator_digest_blake3: String,
    pub(crate) audit_files: Vec<(String, String)>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MantleMeasuredFile {
    pub(crate) path: String,
    pub(crate) digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MantleStageBinary {
    pub(crate) path: String,
    pub(crate) digest_blake3: String,
    pub(crate) producer_action_id: String,
    pub(crate) output_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MantleStageOrchestrator {
    pub(crate) action_id: String,
    pub(crate) executable_digest_blake3: String,
    pub(crate) executable_producer: Option<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MantleStageSmoke {
    pub(crate) action_id: String,
    pub(crate) exit_code: i32,
    pub(crate) stdout: MantleMeasuredFile,
    pub(crate) stderr: MantleMeasuredFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MantleCommandCompletion {
    pub(crate) action_id: String,
    pub(crate) exit_code: i32,
    pub(crate) stdout: MantleMeasuredFile,
    pub(crate) stderr: MantleMeasuredFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MantleCommandReceipts {
    pub(crate) schema: String,
    pub(crate) commands: Vec<MantleCommandCompletion>,
}

/// A view over actual command/publication records, never execution authority.
/// All file paths are normalized, contained paths relative to manifest parent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MantleStageManifest {
    pub(crate) schema: String,
    pub(crate) stage_id: String,
    pub(crate) proof_plan_digest_blake3: String,
    pub(crate) action_plan_digest_blake3: String,
    pub(crate) closure_policy_digest_blake3: String,
    pub(crate) binary: MantleStageBinary,
    pub(crate) orchestrator: MantleStageOrchestrator,
    pub(crate) smoke: MantleStageSmoke,
    pub(crate) command_receipt: MantleMeasuredFile,
    pub(crate) stderr: MantleMeasuredFile,
    pub(crate) semantic_unit_count: u32,
}
#[derive(Debug)]
pub(crate) struct ProofBundleObservation {
    pub(crate) digest_blake3: String,
    pub(crate) entry_count: u32,
}
pub(crate) fn verify_source_built_fixed_point_receipt(
    proof_root: &Path,
    receipt_path: &Path,
) -> Result<ProofBundleObservation, RunError> {
    reject_development_authority(proof_root)?;
    let expected_receipt = proof_root.join(FINAL_RECEIPT_FILE);
    if receipt_path != expected_receipt {
        return Err(receipt_error("receipt must be the selected proof root's deterministic-build-proof.json".into()));
    }
    let (value, _) = read_bound_json::<serde_json::Value>(proof_root, FINAL_RECEIPT_FILE)?;
    let core: DeterministicBuildProofReceipt = serde_json::from_value(value.clone())
        .map_err(|error| receipt_error(format!("parsing deterministic receipt core: {error}")))?;
    let canonical = crunch_release_core::canonical_deterministic_build_proof_receipt(core)
        .map_err(|error| receipt_error(format!("validating deterministic receipt core: {error}")))?;
    if canonical.verdict != DeterministicBuildProofVerdict::SelfRebuildMatch || !canonical.blocking_reasons.is_empty() {
        return Err(receipt_error("deterministic receipt does not establish a self-rebuild match".into()));
    }
    let extension: SourceBuiltReceiptExtension = serde_json::from_value(
        value
            .get(EXTENSION_FIELD)
            .cloned()
            .ok_or_else(|| receipt_error(format!("receipt missing {EXTENSION_FIELD}")))?,
    )
    .map_err(|error| receipt_error(format!("parsing source-built receipt extension: {error}")))?;
    let (plan, _) = read_bound_json::<SourceBuiltFixedPointPlan>(proof_root, PROOF_PLAN_FILE)?;
    crate::source_built_fixed_point::validate_source_built_fixed_point_plan(&plan)
        .map_err(|error| receipt_error(error.to_string()))?;
    validate_receipt_plan_binding(&canonical, &extension, &plan)?;
    let action_evidence = load_action_evidence(proof_root, &plan)?;
    if extension.action_evidence != action_evidence.binding {
        return Err(receipt_error(
            "receipt action plan, complete observed events, or reconciliation binding drift".into(),
        ));
    }
    let (stages, stages_digest) = read_bound_json::<Vec<SourceBuiltStageEvidence>>(proof_root, STAGE_EVIDENCE_FILE)?;
    if stages_digest != extension.stage_evidence_digest_blake3 {
        return Err(receipt_error("source-built stage evidence digest mismatch".into()));
    }
    validate_stage_evidence(&stages)?;
    validate_stage_plan_alignment(&plan, &stages)?;
    verify_cold_stage_artifacts(proof_root, &plan, &stages, &action_evidence)?;
    let measured = verify_cold_provider(proof_root, &plan, &stages, &extension.cold_provider, &action_evidence)?;
    if measured.native_digest != extension.native_provider_digest_blake3
        || measured.rust_digest != extension.rust_provider_digest_blake3
        || measured.closure_digest != extension.toolchain_closure_digest_blake3
    {
        return Err(receipt_error("receipt provider identities differ from actual cold measurements".into()));
    }
    verify_cold_descriptor(proof_root, &plan, &canonical, &extension, measured.provider_size_bytes)?;
    let fixed_point = observe_fixed_point(proof_root, &plan, &action_evidence)?;
    verify_receipt_outputs(&canonical, &extension, &stages, &fixed_point)?;
    if fixed_point.closure_policy_digest_blake3 != plan.policies.closure_policy_digest_blake3
        || hash_file(&evidence_path(proof_root, &extension.cold_provider.toolchain_closure.path)?)?
            != extension.toolchain_closure_digest_blake3
    {
        return Err(receipt_error("receipt closure policy or content drift".into()));
    }
    let observed_bundle = proof_bundle_digest(proof_root)?;
    if observed_bundle.digest_blake3 != extension.final_proof_bundle_digest_blake3 {
        return Err(receipt_error("proof bundle digest mismatch".into()));
    }
    assert_eq!(canonical.runs.len(), REQUIRED_RUN_COUNT);
    debug_assert_eq!(stages.len(), STAGE_EVIDENCE_COUNT);
    Ok(observed_bundle)
}

fn validate_receipt_plan_binding(
    core: &DeterministicBuildProofReceipt,
    extension: &SourceBuiltReceiptExtension,
    plan: &SourceBuiltFixedPointPlan,
) -> Result<(), RunError> {
    let expected_non_claims = NON_CLAIMS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let extension_bindings = [
        extension.schema == EXTENSION_SCHEMA,
        extension.provider_kind == PROOF_PROVIDER_KIND,
        extension.plan_digest_blake3 == plan.plan_digest_blake3,
        extension.source_authority_digest_blake3 == plan.source_authority_digest_blake3,
        extension.protected_execution_policy_digest_blake3 == plan.policies.protected_execution_policy_digest_blake3,
        extension.effect_policy_digest_blake3 == plan.policies.effect_policy_digest_blake3,
        extension.normalization_policy_digest_blake3 == plan.policies.normalization_policy_digest_blake3,
        extension.native_provider_digest_blake3 == plan.policies.expected_native_provider_digest_blake3,
        extension.stage_evidence_path == STAGE_EVIDENCE_FILE,
        extension.non_claims == expected_non_claims,
    ];
    let core_bindings = [
        core.schema == PROOF_WORKFLOW,
        core.workflow_version == PROOF_WORKFLOW,
        core.selected_provider_kind == PROOF_PROVIDER_KIND,
        core.source_blake3 == plan.source_authority_digest_blake3,
        core.vendor_blake3 == plan.receipt_contract.vendor_blake3,
        core.logical_store_prefix == plan.logical_store_prefix,
        core.derivation_identity == format!("source-built-plan:{}", plan.plan_digest_blake3),
        core.hermeticity_mode == HERMETICITY_MODE,
        core.physical_store_isolation == PHYSICAL_STORE_ISOLATION,
    ];
    let is_binding_intact =
        extension_bindings.iter().all(|binding| *binding) && core_bindings.iter().all(|binding| *binding);
    if !is_binding_intact {
        return Err(receipt_error("receipt source, provider, plan, policy, or non-claim binding mismatch".into()));
    }
    let arguments = receipt_arguments(plan, &plan.policies.closure_policy_digest_blake3, extension)?;
    if core.rebuild_descriptor.as_ref().map(|descriptor| &descriptor.ordered_arguments) != Some(&arguments) {
        return Err(receipt_error("receipt core does not bind source-built action evidence extension".into()));
    }
    let observed_digest = deterministic_build_proof_receipt_digest_blake3(core.clone())
        .map_err(|error| receipt_error(format!("digesting receipt: {error}")))?;
    if core.receipt_blake3.as_ref() != Some(&observed_digest) {
        return Err(receipt_error("deterministic receipt core digest missing or mismatched".into()));
    }
    debug_assert_eq!(observed_digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert_eq!(core.schema, PROOF_WORKFLOW);
    Ok(())
}

fn verify_receipt_outputs(
    core: &DeterministicBuildProofReceipt,
    extension: &SourceBuiltReceiptExtension,
    stages: &[SourceBuiltStageEvidence],
    fixed_point: &FixedPointObservation,
) -> Result<(), RunError> {
    verify_mantle_stage_evidence(stages, &fixed_point.stage1, &MantleStageEvidenceBinding {
        role: ProofOutputRole::MantleStage1,
        closure_path: &extension.cold_provider.toolchain_closure.path,
        closure_digest: &extension.toolchain_closure_digest_blake3,
    })?;
    verify_mantle_stage_evidence(stages, &fixed_point.stage2, &MantleStageEvidenceBinding {
        role: ProofOutputRole::MantleStage2,
        closure_path: &extension.cold_provider.toolchain_closure.path,
        closure_digest: &extension.toolchain_closure_digest_blake3,
    })?;
    let output = &fixed_point.stage1.binary_digest_blake3;
    let identity = provider_identity_digest(&ProviderIdentityBinding {
        native_digest_blake3: &extension.native_provider_digest_blake3,
        rust_digest_blake3: &extension.rust_provider_digest_blake3,
        closure_digest_blake3: &extension.toolchain_closure_digest_blake3,
    });
    let provider = format!(
        "provider-kind={PROOF_PROVIDER_KIND};identity-blake3={identity};closure-policy-blake3={}",
        fixed_point.closure_policy_digest_blake3
    );
    let rust = stages.iter().find(|stage| stage.output_role == ProofOutputRole::FullSourceRustProvider);
    let stage2 = stages.iter().find(|stage| stage.output_role == ProofOutputRole::MantleStage2);
    if stage2.map(|stage| stage.executable_identity.as_str()) != Some(format!("blake3:{output}").as_str()) {
        return Err(receipt_error("stage2 orchestrator is not the observed stage1 Mantle binary".into()));
    }
    let output_bindings = [
        core.proof_unit.target_artifact_identity == PROOF_TARGET_IDENTITY,
        core.proof_unit.output_identities == vec![format!("{MANTLE_OUTPUT_NAME}:{output}")],
        core.toolchain_provider_identity == provider,
        rust.map(|stage| &stage.output_digest_blake3) == Some(&extension.rust_provider_digest_blake3),
        core.runs.len() == REQUIRED_RUN_COUNT,
    ];
    if !output_bindings.iter().all(|binding| *binding) {
        return Err(receipt_error("receipt provider or observed output identity mismatch".into()));
    }
    debug_assert!(stage2.is_some());
    debug_assert_eq!(core.runs.len(), REQUIRED_RUN_COUNT);
    for (run, (expected_id, observed)) in core.runs.iter().zip([
        (STAGE1_RUN_ID, &fixed_point.stage1),
        (STAGE2_RUN_ID, &fixed_point.stage2),
    ]) {
        if run.run_id != expected_id
            || run.output_store_paths != vec![format!("proof://{}", observed.binary_relative_path)]
            || run.output_digests
                != vec![DeterministicOutputDigest {
                    name: MANTLE_OUTPUT_NAME.into(),
                    digest_blake3: output.clone(),
                }]
        {
            return Err(receipt_error(format!("receipt run {} does not bind observed Mantle binary", run.run_id)));
        }
    }
    Ok(())
}

/// The immutable recipe binding of one produced output. Named fields keep the
/// journal, stage, producer action, and output label from being transposed.
pub(crate) struct RecipeOutputBinding<'a> {
    pub(crate) journal_path: &'a str,
    pub(crate) stage_id: &'a str,
    pub(crate) action_id: &'a str,
    pub(crate) output_id: &'a str,
}

pub(crate) fn recipe_output_path(binding: &RecipeOutputBinding<'_>) -> Result<String, RunError> {
    let parent = binding
        .journal_path
        .strip_suffix("/execution.jsonl")
        .ok_or_else(|| receipt_error("recipe journal lacks canonical filename".into()))?;
    let step = binding
        .action_id
        .strip_prefix(binding.stage_id)
        .and_then(|suffix| suffix.strip_prefix('/'))
        .ok_or_else(|| receipt_error("recipe output producer belongs to a different stage".into()))?;
    if binding.output_id.contains('/') {
        return Err(receipt_error("recipe output is not a single label".into()));
    }
    for label in step.split('/').chain(std::iter::once(binding.output_id)) {
        let is_label_reserved = label == "." || label == "..";
        let is_label_too_long = label.len() > 256;
        let is_label_charset_valid =
            label.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'));
        let label_defects = [
            label.is_empty(),
            is_label_reserved,
            is_label_too_long,
            !is_label_charset_valid,
        ];
        if label_defects.iter().any(|defect| *defect) {
            return Err(receipt_error("recipe output has invalid physical binding label".into()));
        }
    }
    debug_assert!(!step.is_empty());
    debug_assert!(!binding.output_id.is_empty());
    Ok(format!("{parent}/{step}/outputs/{}", binding.output_id))
}
/// The reconciled events a recipe journal may corroborate, in journal order.
struct RecipeJournalEvents<'a> {
    builds: Vec<&'a ActionExecutionEvent>,
    protected: Vec<&'a ActionExecutionEvent>,
    guards: Vec<&'a ActionExecutionEvent>,
    statuses: Vec<&'a ActionExecutionEvent>,
}

fn recipe_journal_events<'a>(actions: &'a ActionEvidence, ids: &BTreeSet<&'a str>) -> RecipeJournalEvents<'a> {
    let events = RecipeJournalEvents {
        builds: actions
            .events
            .iter()
            .filter(|event| {
                ids.contains(event.action_id.as_str())
                    && event.kind == ExecutionEventKind::BuildExecution
                    && !event.action_id.ends_with(".sandbox")
                    && !event.action_id.ends_with("/orchestrator")
                    && !event.produced_outputs.iter().any(|(id, _)| id == "native-provider-publication")
            })
            .collect(),
        protected: actions
            .events
            .iter()
            .filter(|event| {
                ids.contains(event.action_id.as_str())
                    && event.kind == ExecutionEventKind::ProtectedExec
                    && !event.action_id.ends_with("/orchestrator")
            })
            .collect(),
        guards: actions
            .events
            .iter()
            .filter(|event| {
                ids.contains(event.action_id.as_str()) && matches!(event.kind, ExecutionEventKind::GuardDecision { .. })
            })
            .collect(),
        statuses: actions
            .events
            .iter()
            .filter(|event| {
                ids.contains(event.action_id.as_str())
                    && event.kind == ExecutionEventKind::BuildExecution
                    && event.normal_exit_code.is_some()
            })
            .collect(),
    };
    debug_assert!(events.builds.len() <= actions.events.len());
    debug_assert!(events.guards.len() <= actions.events.len());
    events
}

/// One recipe journal under verification. Each record kind consumes at most
/// one reconciled event, and no journal record may introduce new authority.
struct RecipeJournalCursor<'a> {
    root: &'a Path,
    proof: &'a SourceBuiltFixedPointPlan,
    stage: &'a SourceBuiltStageEvidence,
    events: RecipeJournalEvents<'a>,
    build_cursor: usize,
    protected_cursor: usize,
    guard_cursor: usize,
    status_cursor: usize,
    is_header_seen: bool,
    is_complete: bool,
    output_count: u64,
}

impl<'a> RecipeJournalCursor<'a> {
    fn new(
        root: &'a Path,
        proof: &'a SourceBuiltFixedPointPlan,
        stage: &'a SourceBuiltStageEvidence,
        events: RecipeJournalEvents<'a>,
    ) -> Self {
        let cursor = Self {
            root,
            proof,
            stage,
            events,
            build_cursor: 0,
            protected_cursor: 0,
            guard_cursor: 0,
            status_cursor: 0,
            is_header_seen: false,
            is_complete: false,
            output_count: 0,
        };
        debug_assert!(!cursor.is_header_seen);
        debug_assert_eq!(cursor.output_count, 0);
        cursor
    }

    fn next_build(&mut self) -> Option<&'a ActionExecutionEvent> {
        let event = self.events.builds.get(self.build_cursor).copied()?;
        self.build_cursor = self.build_cursor.saturating_add(1);
        Some(event)
    }

    fn next_protected(&mut self) -> Option<&'a ActionExecutionEvent> {
        let event = self.events.protected.get(self.protected_cursor).copied()?;
        self.protected_cursor = self.protected_cursor.saturating_add(1);
        Some(event)
    }

    fn next_guard(&mut self) -> Option<&'a ActionExecutionEvent> {
        let event = self.events.guards.get(self.guard_cursor).copied()?;
        self.guard_cursor = self.guard_cursor.saturating_add(1);
        Some(event)
    }

    fn next_status(&mut self) -> Option<&'a ActionExecutionEvent> {
        let event = self.events.statuses.get(self.status_cursor).copied()?;
        self.status_cursor = self.status_cursor.saturating_add(1);
        Some(event)
    }

    fn apply_record(&mut self, record: &serde_json::Value) -> Result<(), RunError> {
        if !self.is_header_seen {
            return self.apply_header(record);
        }
        if record.get("action_id").is_some() {
            return self.apply_guard(record);
        }
        if record.get("normal_exit_code").is_some() {
            return self.apply_status(record);
        }
        if record.get("actual_outputs").is_some() {
            return self.apply_completion(record);
        }
        if record.get("pid").is_some() {
            return self.apply_protected(record);
        }
        if *record == serde_json::json!({"complete":true,"output_count":self.output_count}) {
            self.is_complete = true;
            debug_assert!(self.is_header_seen);
            debug_assert!(self.build_cursor <= self.events.builds.len());
            return Ok(());
        }
        Err(receipt_error("recipe journal contains a failed, unknown, or legacy summary record".into()))
    }

    fn apply_header(&mut self, record: &serde_json::Value) -> Result<(), RunError> {
        let proof = self.proof;
        let stage = self.stage;
        if *record
            != serde_json::json!({"schema":"mantle-source-recipe-execution-v1","proof":proof.plan_digest_blake3,"stage":stage.stage_id})
        {
            return Err(receipt_error("recipe journal source/stage header mismatch".into()));
        }
        self.is_header_seen = true;
        Ok(())
    }

    fn apply_guard(&mut self, record: &serde_json::Value) -> Result<(), RunError> {
        let event = self
            .next_guard()
            .ok_or_else(|| receipt_error("recipe journal adds an unobserved guard decision".into()))?;
        let expected = serde_json::to_value(event).map_err(|error| receipt_error(error.to_string()))?;
        if *record != expected {
            return Err(receipt_error("recipe journal guard differs from reconciled control evidence".into()));
        }
        Ok(())
    }

    fn apply_status(&mut self, record: &serde_json::Value) -> Result<(), RunError> {
        let event = self
            .next_status()
            .ok_or_else(|| receipt_error("recipe journal adds an unobserved normal status".into()))?;
        if *record != serde_json::json!({"action":event.action_id,"normal_exit_code":event.normal_exit_code}) {
            return Err(receipt_error("recipe journal status differs from actual source command exit".into()));
        }
        Ok(())
    }

    fn apply_completion(&mut self, record: &serde_json::Value) -> Result<(), RunError> {
        let event = self
            .next_build()
            .ok_or_else(|| receipt_error("recipe journal adds an unobserved completion".into()))?;
        if *record
            != serde_json::json!({"action":event.action_id,"actual_outputs":event.produced_outputs,"disk_bytes":event.disk_bytes,"completed":true})
        {
            return Err(receipt_error("recipe journal completion differs from reconciled execution".into()));
        }
        let root = self.root;
        let stage = self.stage;
        let mut measured_bytes = 0_u64;
        for (output, digest) in &event.produced_outputs {
            let path = recipe_output_path(&RecipeOutputBinding {
                journal_path: &stage.execution_journal_path,
                stage_id: &stage.stage_id,
                action_id: &event.action_id,
                output_id: output,
            })?;
            let (size, measured) = measure_artifact(root, &path)?;
            if &measured != digest {
                return Err(receipt_error(format!("actual recipe output {path} content drift")));
            }
            measured_bytes = measured_bytes
                .checked_add(size)
                .ok_or_else(|| receipt_error("recipe output size overflow".into()))?;
        }
        if measured_bytes != event.disk_bytes {
            return Err(receipt_error("recipe actual output size differs from reconciled completion".into()));
        }
        debug_assert_eq!(measured_bytes, event.disk_bytes);
        debug_assert!(!event.produced_outputs.is_empty() || event.disk_bytes == 0);
        self.output_count = self
            .output_count
            .checked_add(event.produced_outputs.len() as u64)
            .ok_or_else(|| receipt_error("recipe output count overflow".into()))?;
        Ok(())
    }

    fn apply_protected(&mut self, record: &serde_json::Value) -> Result<(), RunError> {
        let keys = [
            "pid",
            "syscall",
            "tracee_path",
            "resolved_path",
            "digest_blake3",
            "allowed",
            "reason",
            "is_exec",
        ];
        let is_key_set_complete = record
            .as_object()
            .is_some_and(|object| object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key)));
        let observation_defects = [
            !is_key_set_complete,
            record["allowed"] != true,
            record["pid"].as_u64().is_none(),
            record["syscall"].as_i64().is_none(),
            record["tracee_path"].as_str().is_none(),
            record["resolved_path"].as_str().is_none(),
            record["reason"].as_str().is_none(),
        ];
        if observation_defects.iter().any(|defect| *defect) {
            return Err(receipt_error("recipe journal malformed or denied protected observation".into()));
        }
        if record["is_exec"] == true {
            let event = self
                .next_protected()
                .ok_or_else(|| receipt_error("recipe journal adds an unobserved protected execution".into()))?;
            if record["digest_blake3"].as_str() != Some(event.executable_digest_blake3.as_str()) {
                return Err(receipt_error("recipe journal protected image differs from reconciled execution".into()));
            }
            return Ok(());
        }
        let image_claims = [
            record["is_exec"] != false,
            record["digest_blake3"] != "",
            record["tracee_path"] != "",
            record["resolved_path"] != "",
        ];
        if image_claims.iter().any(|claim| *claim) {
            return Err(receipt_error("recipe journal process observation claims image authority".into()));
        }
        debug_assert!(is_key_set_complete);
        debug_assert!(record["allowed"] == true);
        Ok(())
    }

    fn require_complete(&self) -> Result<(), RunError> {
        let unconsumed = [
            !self.is_header_seen,
            !self.is_complete,
            self.build_cursor < self.events.builds.len(),
            self.protected_cursor < self.events.protected.len(),
            self.guard_cursor < self.events.guards.len(),
            self.status_cursor < self.events.statuses.len(),
        ];
        if unconsumed.iter().any(|pending| *pending) {
            return Err(receipt_error("recipe journal omits actual execution records".into()));
        }
        Ok(())
    }
}

/// Measured corroboration only: journals cannot introduce execution authority.
fn verify_recipe_journal(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    stage: &SourceBuiltStageEvidence,
    actions: &ActionEvidence,
) -> Result<(), RunError> {
    let path = evidence_path(root, &stage.execution_journal_path)?;
    if hash_file(&path)? != stage.execution_journal_digest_blake3 {
        return Err(receipt_error(format!("{} recipe journal content drift", stage.stage_id)));
    }
    let file = fs::File::open(path).map_err(|error| receipt_error(error.to_string()))?;
    if file.metadata().map_err(|error| receipt_error(error.to_string()))?.len() > JSON_BYTES_MAX {
        return Err(receipt_error("recipe journal exceeds byte bound".into()));
    }
    let ids = stage_action_ids(actions, &stage.stage_id)?;
    let mut cursor = RecipeJournalCursor::new(root, proof, stage, recipe_journal_events(actions, &ids));
    let mut reader = std::io::BufReader::new(file);
    let mut line = Vec::new();
    for _ in 0..RECIPE_JOURNAL_RECORD_LIMIT {
        line.clear();
        if reader.read_until(b'\n', &mut line).map_err(|error| receipt_error(error.to_string()))? == 0 {
            break;
        }
        if cursor.is_complete || line.last() != Some(&b'\n') {
            return Err(receipt_error("recipe journal has trailing or incomplete records".into()));
        }
        let record: serde_json::Value = serde_json::from_slice(&line)
            .map_err(|error| receipt_error(format!("parsing actual recipe journal: {error}")))?;
        cursor.apply_record(&record)?;
    }
    cursor.require_complete()?;
    debug_assert!(cursor.is_header_seen);
    debug_assert!(cursor.is_complete);
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum PublicationProducer {
    Produced { action_id: String, output_id: String },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishedArtifact {
    producer: PublicationProducer,
    path: String,
    digest_blake3: String,
    size_bytes: u64,
    kind: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativePublication {
    schema: String,
    proof_plan_digest_blake3: String,
    native_source_manifest_digest_blake3: String,
    expected_provider_digest_blake3: String,
    artifacts: std::collections::BTreeMap<String, PublishedArtifact>,
}

const NATIVE_EXECUTABLE_ROLES: &[&str] = &[
    "native.cc1",
    "native.cc1plus",
    "native.as",
    "native.ld",
    "native.ar",
    "native.nm",
    "native.ranlib",
    "native.objcopy",
    "native.strip",
    "native.readelf",
    "native.loader",
    "host.make",
    "host.cmake",
    "host.python",
    "host.perl",
    "host.busybox",
];
const NATIVE_TREE_ROLES: &[&str] = &["native.provider", "native.sysroot", "linux.headers"];
const NATIVE_RUNTIME_ROLES: &[&str] = &["native.libc", "native.libgcc", "native.libgcc-eh", "native.libstdcxx"];

/// One published native artifact together with the evidence that produced it.
struct NativePublicationCheck<'a> {
    root: &'a Path,
    stage: &'a SourceBuiltStageEvidence,
    actions: &'a ActionEvidence,
    publication: &'a NativePublication,
    producer_event: &'a ActionExecutionEvent,
}

impl NativePublicationCheck<'_> {
    fn verify_role(&self, role: &str) -> Result<(), RunError> {
        let artifact = self
            .publication
            .artifacts
            .get(role)
            .ok_or_else(|| receipt_error(format!("native publication omits {role}")))?;
        let PublicationProducer::Produced { action_id, output_id } = &artifact.producer;
        let measured = measure_artifact(self.root, &artifact.path)?;
        let producer = bound_output_event(self.actions, &BoundOutputRequest {
            stage_id: &self.stage.stage_id,
            producer_action_id: action_id,
            output_id,
        })?;
        let is_expected_kind = if NATIVE_EXECUTABLE_ROLES.contains(&role) {
            artifact.kind == "executable"
        } else if NATIVE_TREE_ROLES.contains(&role) {
            artifact.kind == "tree"
        } else {
            matches!(artifact.kind.as_str(), "archive" | "object" | "executable")
        };
        if !is_expected_kind {
            return Err(receipt_error(format!("native publication actual artifact {role} drift")));
        }
        if evidence_path(self.root, &artifact.path)?.is_dir() != NATIVE_TREE_ROLES.contains(&role) {
            return Err(receipt_error(format!("native publication actual artifact {role} drift")));
        }
        if measured != (artifact.size_bytes, artifact.digest_blake3.clone()) {
            return Err(receipt_error(format!("native publication actual artifact {role} drift")));
        }
        let is_produced_output = producer
            .produced_outputs
            .iter()
            .any(|(id, observed)| id == output_id && observed == &artifact.digest_blake3);
        if !is_produced_output {
            return Err(receipt_error(format!("native publication actual artifact {role} drift")));
        }
        let is_produced_input = self.producer_event.inputs.iter().zip(&self.producer_event.input_digests_blake3).any(
            |(input, observed)| {
                matches!(input,
                crate::source_built_fixed_point_action_trust::ActionInput::Produced { action_id: id, output_id: output }
                if id == action_id && output == output_id)
                    && observed == &artifact.digest_blake3
            },
        );
        if !is_produced_input {
            return Err(receipt_error(format!("native publication actual artifact {role} drift")));
        }
        if role == "native.provider" {
            let is_canonical = [
                artifact.path == self.stage.output_path,
                action_id == &self.stage.producer_action_id,
                output_id == &self.stage.output_id,
            ];
            if !is_canonical.iter().all(|canonical| *canonical) {
                return Err(receipt_error("native publication canonical provider binding drift".into()));
            }
        }
        debug_assert!(is_expected_kind);
        debug_assert!(is_produced_output);
        debug_assert!(is_produced_input);
        Ok(())
    }
}

fn verify_native_publication(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    stage: &SourceBuiltStageEvidence,
    actions: &ActionEvidence,
) -> Result<(), RunError> {
    if stage.audit_paths.len() != 1 {
        return Err(receipt_error("native cold stage requires its actual publication audit".into()));
    }
    let (publication, digest) = read_bound_json::<NativePublication>(root, &stage.audit_paths[0])?;
    let ids = stage_action_ids(actions, &stage.stage_id)?;
    let event = actions
        .events
        .iter()
        .find(|event| {
            ids.contains(event.action_id.as_str())
                && event.kind == ExecutionEventKind::BuildExecution
                && event
                    .produced_outputs
                    .iter()
                    .any(|(id, observed)| id == "native-provider-publication" && observed == &digest)
        })
        .ok_or_else(|| receipt_error("native publication has no reconciled actual producer".into()))?;
    let source_digest = proof
        .source_inputs
        .iter()
        .find(|input| input.role == crate::source_built_fixed_point::SourceAuthorityRole::NativeSourceBundle)
        .ok_or_else(|| receipt_error("native manifest not admitted".into()))?;
    let publication_bindings = [
        publication.schema == "mantle-source-native-provider-publication-v1",
        publication.proof_plan_digest_blake3 == proof.plan_digest_blake3,
        publication.native_source_manifest_digest_blake3 == source_digest.digest_blake3,
        publication.expected_provider_digest_blake3 == proof.policies.expected_native_provider_digest_blake3,
    ];
    if !publication_bindings.iter().all(|binding| *binding) {
        return Err(receipt_error("native publication source/proof/pin mismatch".into()));
    }
    let expected_artifact_count = NATIVE_EXECUTABLE_ROLES
        .len()
        .saturating_add(NATIVE_TREE_ROLES.len())
        .saturating_add(NATIVE_RUNTIME_ROLES.len());
    if publication.artifacts.len() != expected_artifact_count {
        return Err(receipt_error("native publication role coverage drift".into()));
    }
    let check = NativePublicationCheck {
        root,
        stage,
        actions,
        publication: &publication,
        producer_event: event,
    };
    for role in NATIVE_EXECUTABLE_ROLES.iter().chain(NATIVE_TREE_ROLES).chain(NATIVE_RUNTIME_ROLES) {
        check.verify_role(role)?;
    }
    debug_assert_eq!(event.kind, ExecutionEventKind::BuildExecution);
    debug_assert_eq!(publication.artifacts.len(), expected_artifact_count);
    Ok(())
}

/// Collect the relative paths of one artifact tree. The walk is iterative so
/// that a deep tree cannot exhaust the stack, and every directory is visited
/// exactly once.
fn collect_artifact_entries(root: &Path, current: &Path, entries: &mut Vec<String>) -> Result<(), RunError> {
    let mut pending: Vec<(PathBuf, usize)> = Vec::with_capacity(64);
    pending.push((current.to_path_buf(), 0));
    for _ in 0..ARTIFACT_TREE_WALK_STEPS {
        let Some((directory, depth)) = pending.pop() else {
            debug_assert!(entries.len() <= 1_000_000);
            return Ok(());
        };
        if depth > 128 {
            return Err(receipt_error("artifact tree depth bound exceeded".into()));
        }
        debug_assert!(depth <= 128);
        for entry in fs::read_dir(&directory).map_err(|error| receipt_error(error.to_string()))? {
            let entry = entry.map_err(|error| receipt_error(error.to_string()))?;
            let path = entry.path();
            let relative = relative_path(root, &path)?;
            if entries.len() >= 1_000_000 || relative.len() > 4096 {
                return Err(receipt_error("artifact tree entry/path bound exceeded".into()));
            }
            let metadata = fs::symlink_metadata(&path).map_err(|error| receipt_error(error.to_string()))?;
            entries.push(relative);
            if metadata.is_dir() {
                if pending.len() >= ARTIFACT_TREE_PENDING_LIMIT {
                    return Err(receipt_error("artifact tree entry/path bound exceeded".into()));
                }
                pending.push((path, depth.saturating_add(1)));
            }
        }
    }
    Err(receipt_error("artifact tree entry/path bound exceeded".into()))
}

/// Hash one artifact tree entry and report the bytes it contributes to the
/// tree total. Symlinks are hashed as text and never grant authority outside
/// the selected tree.
fn measure_artifact_entry(
    entry: &Path,
    relative: &str,
    metadata: &fs::Metadata,
    canonical_tree: &Path,
    hasher: &mut blake3::Hasher,
) -> Result<u64, RunError> {
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o7777
    };
    #[cfg(not(unix))]
    let mode = 0_u32;
    hasher.update(&(relative.len() as u64).to_le_bytes());
    hasher.update(relative.as_bytes());
    hasher.update(&mode.to_le_bytes());
    if metadata.is_dir() {
        hasher.update(b"dir\0");
        return Ok(0);
    }
    if metadata.is_file() {
        hasher.update(b"file\0");
        hasher.update(&metadata.len().to_le_bytes());
        let mut file = fs::File::open(entry).map_err(|error| receipt_error(error.to_string()))?;
        let mut bytes = 0_u64;
        let mut buffer = [0_u8; HASH_BUFFER_BYTES];
        // Every read before end of file consumes at least one byte, so
        // `len + 2` reads always reach either end of file or more than `len`
        // bytes, even when the kernel returns short reads.
        let read_bound = metadata.len().saturating_add(2);
        for _ in 0..read_bound {
            let count = file.read(&mut buffer).map_err(|error| receipt_error(error.to_string()))?;
            if count == 0 {
                break;
            }
            bytes =
                bytes.checked_add(count as u64).ok_or_else(|| receipt_error("artifact file size overflow".into()))?;
            hasher.update(&buffer[..count]);
        }
        if bytes != metadata.len() {
            return Err(receipt_error("artifact tree changed while hashing".into()));
        }
        debug_assert_eq!(bytes, metadata.len());
        return Ok(bytes);
    }
    if metadata.file_type().is_symlink() {
        return measure_artifact_symlink(entry, relative, metadata, canonical_tree, hasher);
    }
    Err(receipt_error("unsupported artifact tree entry".into()))
}

/// Hash one artifact symlink as text. The target is never followed to grant file
/// authority outside the selected tree.
fn measure_artifact_symlink(
    entry: &Path,
    relative: &str,
    metadata: &fs::Metadata,
    canonical_tree: &Path,
    hasher: &mut blake3::Hasher,
) -> Result<u64, RunError> {
    let target = fs::read_link(entry).map_err(|error| receipt_error(error.to_string()))?;
    let text = target.to_str().ok_or_else(|| receipt_error("artifact symlink target is not UTF-8".into()))?;
    if text.is_empty() || text.len() > 4096 || target.is_absolute() {
        return Err(receipt_error("artifact tree has unsafe symlink".into()));
    }
    debug_assert!(!text.is_empty());
    debug_assert!(metadata.file_type().is_symlink());
    let mut depth = Path::new(relative).parent().map_or(0, |parent| parent.components().count());
    for part in target.components() {
        match part {
            Component::Normal(_) => depth = depth.saturating_add(1),
            Component::CurDir => (),
            Component::ParentDir if depth > 0 => depth = depth.saturating_sub(1),
            _ => return Err(receipt_error("artifact symlink escapes tree".into())),
        }
    }
    let resolved = fs::canonicalize(entry)
        .map_err(|error| receipt_error(format!("artifact symlink is dangling or cyclic: {error}")))?;
    if !resolved.starts_with(canonical_tree) {
        return Err(receipt_error("artifact symlink resolves outside its tree".into()));
    }
    hasher.update(b"symlink\0");
    hasher.update(&(text.len() as u64).to_le_bytes());
    hasher.update(text.as_bytes());
    Ok(metadata.len())
}

/// Same content framing as recipe release-tree measurement. Links are hashed as
/// text, never followed to grant file authority outside the selected tree.
pub(crate) fn measure_artifact(root: &Path, relative: &str) -> Result<(u64, String), RunError> {
    let path = evidence_path(root, relative)?;
    let metadata = fs::symlink_metadata(&path).map_err(|error| receipt_error(error.to_string()))?;
    if metadata.is_file() {
        return Ok((metadata.len(), hash_file_allow_empty(&path)?));
    }
    if !metadata.is_dir() {
        return Err(receipt_error("artifact must be a nonsymlink file or tree".into()));
    }
    let mut entries = Vec::new();
    collect_artifact_entries(&path, &path, &mut entries)?;
    debug_assert!(entries.len() <= 1_000_000);
    entries.sort();
    let canonical_tree = fs::canonicalize(&path).map_err(|error| receipt_error(error.to_string()))?;
    let mut hasher = blake3::Hasher::new();
    let mut total = 0_u64;
    for relative in entries {
        let entry = path.join(&relative);
        let metadata = fs::symlink_metadata(&entry).map_err(|error| receipt_error(error.to_string()))?;
        let measured = measure_artifact_entry(&entry, &relative, &metadata, &canonical_tree, &mut hasher)?;
        total = total.checked_add(measured).ok_or_else(|| receipt_error("artifact tree size overflow".into()))?;
    }
    let tree_digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(tree_digest.len(), BLAKE3_HEX_LENGTH);
    Ok((total, tree_digest))
}

/// The Mantle stage evidence binding: which output role and which closure audit.
struct MantleStageEvidenceBinding<'a> {
    role: ProofOutputRole,
    closure_path: &'a str,
    closure_digest: &'a str,
}

fn verify_mantle_stage_evidence(
    stages: &[SourceBuiltStageEvidence],
    observed: &FixedPointStageObservation,
    binding: &MantleStageEvidenceBinding<'_>,
) -> Result<(), RunError> {
    let stage = stages
        .iter()
        .find(|stage| stage.output_role == binding.role)
        .ok_or_else(|| receipt_error("missing observed Mantle stage evidence".into()))?;
    let evidence_matches = [
        stage.transcript_path == observed.manifest_relative_path,
        stage.transcript_digest_blake3 == observed.manifest_digest_blake3,
        stage.output_path == observed.binary_relative_path,
        stage.output_digest_blake3 == observed.binary_digest_blake3,
        stage.executable_identity == format!("blake3:{}", observed.orchestrator_digest_blake3),
        stage.audit_paths.len() == observed.audit_files.len().saturating_add(1),
        stage.audit_digests_blake3.len() == stage.audit_paths.len(),
        stage.audit_paths.last().map(String::as_str) == Some(binding.closure_path),
        stage.audit_digests_blake3.last().map(String::as_str) == Some(binding.closure_digest),
    ];
    if !evidence_matches.iter().all(|matches| *matches) {
        return Err(receipt_error(format!("{} evidence does not match actual action-bound manifest", stage.stage_id)));
    }
    for ((path, digest), (expected_path, expected_digest)) in
        stage.audit_paths.iter().zip(&stage.audit_digests_blake3).zip(&observed.audit_files)
    {
        if path != expected_path || digest != expected_digest {
            return Err(receipt_error(format!("{} command evidence drift", stage.stage_id)));
        }
    }
    debug_assert_eq!(stage.audit_paths.len(), stage.audit_digests_blake3.len());
    debug_assert!(stage.audit_paths.last().map(String::as_str) == Some(binding.closure_path));
    Ok(())
}

/// Reopen every canonical output, including directory trees. A claimed event
/// digest without independently measured content never constitutes stage evidence.
pub(crate) fn verify_cold_stage_artifacts(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    stages: &[SourceBuiltStageEvidence],
    actions: &ActionEvidence,
) -> Result<(), RunError> {
    for stage in stages {
        verify_cold_stage(root, proof, stage, actions)?;
    }
    Ok(())
}

fn verify_cold_stage(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    stage: &SourceBuiltStageEvidence,
    actions: &ActionEvidence,
) -> Result<(), RunError> {
    if hash_file(&evidence_path(root, &stage.transcript_path)?)? != stage.transcript_digest_blake3 {
        return Err(receipt_error(format!("stage {} transcript content drift", stage.stage_id)));
    }
    for (path, digest) in stage.audit_paths.iter().zip(&stage.audit_digests_blake3) {
        if hash_file_allow_empty(&evidence_path(root, path)?)? != *digest {
            return Err(receipt_error(format!("stage {} audit content drift", stage.stage_id)));
        }
    }
    let expected_output = match stage.output_role {
        ProofOutputRole::StagexTransition => "stagex-transition",
        ProofOutputRole::StagexProvider => "stagex-provider",
        ProofOutputRole::FullSourceNativeProvider => "full-source-native-provider",
        ProofOutputRole::FullSourceRustProvider => "full-source-rust-provider",
        ProofOutputRole::MantleStage1 => "mantle-stage1",
        ProofOutputRole::MantleStage2 => "mantle-stage2",
    };
    if stage.output_id != expected_output {
        return Err(receipt_error(format!("stage {} has noncanonical output identity", stage.stage_id)));
    }
    if stage.output_path
        != recipe_output_path(&RecipeOutputBinding {
            journal_path: &stage.execution_journal_path,
            stage_id: &stage.stage_id,
            action_id: &stage.producer_action_id,
            output_id: &stage.output_id,
        })?
    {
        return Err(receipt_error("canonical output differs from its immutable recipe binding".into()));
    }
    let event = bound_output_event(actions, &BoundOutputRequest {
        stage_id: &stage.stage_id,
        producer_action_id: &stage.producer_action_id,
        output_id: &stage.output_id,
    })?;
    let (size, digest) = measure_artifact(root, &stage.output_path)?;
    if !event
        .produced_outputs
        .iter()
        .any(|(id, observed)| id == &stage.output_id && observed == &stage.output_digest_blake3)
        || digest != stage.output_digest_blake3
        || size > proof.resource_bounds.disk_bytes_max
    {
        return Err(receipt_error(format!("stage {} actual output differs from reconciled producer", stage.stage_id)));
    }
    debug_assert_eq!(digest, stage.output_digest_blake3);
    debug_assert!(size <= proof.resource_bounds.disk_bytes_max);
    if !matches!(stage.output_role, ProofOutputRole::MantleStage1 | ProofOutputRole::MantleStage2)
        && (stage.transcript_path != stage.execution_journal_path
            || stage.executable_identity != format!("blake3:{}", event.executable_digest_blake3))
    {
        return Err(receipt_error(format!("stage {} is not an actual recipe observation", stage.stage_id)));
    }
    verify_recipe_journal(root, proof, stage, actions)?;
    if stage.output_role == ProofOutputRole::FullSourceNativeProvider {
        verify_native_publication(root, proof, stage, actions)?;
    } else if !matches!(stage.output_role, ProofOutputRole::MantleStage1 | ProofOutputRole::MantleStage2)
        && !stage.audit_paths.is_empty()
    {
        return Err(receipt_error("cold source stage cannot substitute external summary audits".into()));
    }
    Ok(())
}

/// The reconciled producer of one stage output. Named fields keep the stage,
/// producer action, and output label from being transposed.
struct BoundOutputRequest<'a> {
    stage_id: &'a str,
    producer_action_id: &'a str,
    output_id: &'a str,
}

fn bound_output_event<'a>(
    actions: &'a ActionEvidence,
    request: &BoundOutputRequest<'_>,
) -> Result<&'a ActionExecutionEvent, RunError> {
    let adapter = actions
        .plan
        .adapters
        .iter()
        .find(|adapter| adapter.stage_id == request.stage_id)
        .ok_or_else(|| receipt_error(format!("missing stage adapter {}", request.stage_id)))?;
    if !crate::source_built_fixed_point_action_trust::adapter_action_spec(adapter, request.producer_action_id)
        .map_err(receipt_error)?
        .is_some_and(|action| action.output_ids.iter().any(|id| id == request.output_id))
    {
        return Err(receipt_error(format!(
            "{} output lacks its admitted producer {}/{}",
            request.stage_id, request.producer_action_id, request.output_id
        )));
    }
    let event = actions
        .events
        .iter()
        .find(|event| {
            event.kind == ExecutionEventKind::BuildExecution
                && event.action_id == request.producer_action_id
                && event.produced_outputs.iter().any(|(id, _)| id == request.output_id)
        })
        .ok_or_else(|| {
            receipt_error(format!("missing actual completion {}/{}", request.producer_action_id, request.output_id))
        })?;
    debug_assert_eq!(event.action_id, request.producer_action_id);
    debug_assert_eq!(event.kind, ExecutionEventKind::BuildExecution);
    Ok(event)
}

pub(crate) fn verify_cold_provider(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    stages: &[SourceBuiltStageEvidence],
    provider: &ColdProviderEvidence,
    actions: &ActionEvidence,
) -> Result<ColdProviderMeasurement, RunError> {
    let native = stages
        .iter()
        .find(|stage| stage.output_role == ProofOutputRole::FullSourceNativeProvider)
        .ok_or_else(|| receipt_error("missing native cold output".into()))?;
    let rust = stages
        .iter()
        .find(|stage| stage.output_role == ProofOutputRole::FullSourceRustProvider)
        .ok_or_else(|| receipt_error("missing Rust cold output".into()))?;
    if provider.native_provider_path != native.output_path {
        return Err(receipt_error("cold provider paths are not the canonical measured provider trees".into()));
    }
    if provider.rust_provider_path != rust.output_path {
        return Err(receipt_error("cold provider paths are not the canonical measured provider trees".into()));
    }
    if !evidence_path(root, &provider.native_provider_path)?.is_dir() {
        return Err(receipt_error("cold provider paths are not the canonical measured provider trees".into()));
    }
    if !evidence_path(root, &provider.rust_provider_path)?.is_dir() {
        return Err(receipt_error("cold provider paths are not the canonical measured provider trees".into()));
    }
    let (native_size, native_digest) = measure_artifact(root, &provider.native_provider_path)?;
    let (rust_size, rust_digest) = measure_artifact(root, &provider.rust_provider_path)?;
    verify_cold_artifact(root, &provider.rustc, &rust.execution_journal_path, actions)?;
    let closure_digest =
        verify_cold_artifact(root, &provider.toolchain_closure, &rust.execution_journal_path, actions)?;
    let rustc = evidence_path(root, &provider.rustc.path)?;
    let mut magic = [0_u8; 4];
    fs::File::open(&rustc)
        .and_then(|mut file| file.read_exact(&mut magic))
        .map_err(|error| receipt_error(format!("reading produced rustc: {error}")))?;
    let provider_matches = [
        magic == *b"\x7fELF",
        native_digest == proof.policies.expected_native_provider_digest_blake3,
        native_digest == native.output_digest_blake3,
        rust_digest == rust.output_digest_blake3,
    ];
    if !provider_matches.iter().all(|matches| *matches) {
        return Err(receipt_error("actual cold provider/rustc differs from pinned output".into()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(rustc).map_err(|error| receipt_error(error.to_string()))?.permissions().mode() & 0o111 == 0 {
            return Err(receipt_error("produced rustc is not executable".into()));
        }
    }
    // The closure remains an actual produced text artifact, never a legacy
    // provider-summary conversion or a host-supplied execution attestation.
    let closure_size_bytes = verify_cold_closure(root, proof, provider)?;
    let provider_size_bytes = native_size
        .checked_add(rust_size)
        .and_then(|size| size.checked_add(closure_size_bytes))
        .ok_or_else(|| receipt_error("provider size overflow".into()))?;
    debug_assert!(closure_size_bytes <= JSON_BYTES_MAX);
    debug_assert_eq!(provider_size_bytes, native_size.saturating_add(rust_size).saturating_add(closure_size_bytes));
    Ok(ColdProviderMeasurement {
        native_digest,
        rust_digest,
        closure_digest,
        provider_size_bytes,
    })
}

/// The closure remains an actual produced text artifact, never a legacy
/// provider-summary conversion or a host-supplied execution attestation.
fn verify_cold_closure(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    provider: &ColdProviderEvidence,
) -> Result<u64, RunError> {
    let closure_path = evidence_path(root, &provider.toolchain_closure.path)?;
    let closure_size_bytes = fs::metadata(&closure_path).map_err(|error| receipt_error(error.to_string()))?.len();
    if closure_size_bytes > JSON_BYTES_MAX || closure_size_bytes > proof.resource_bounds.disk_bytes_max {
        return Err(receipt_error("actual closure exceeds byte bound".into()));
    }
    let closure = fs::read_to_string(closure_path)
        .map_err(|error| receipt_error(format!("reading actual closure text: {error}")))?;
    if closure.is_empty() || closure.contains('\0') {
        return Err(receipt_error("actual toolchain closure is not nonempty source-produced text".into()));
    }
    debug_assert!(closure_size_bytes <= JSON_BYTES_MAX);
    debug_assert!(!closure.is_empty());
    Ok(closure_size_bytes)
}

fn verify_cold_artifact(
    root: &Path,
    binding: &ColdArtifactBinding,
    journal: &str,
    actions: &ActionEvidence,
) -> Result<String, RunError> {
    if binding.path
        != recipe_output_path(&RecipeOutputBinding {
            journal_path: journal,
            stage_id: "full-source-rust-provider",
            action_id: &binding.producer_action_id,
            output_id: &binding.output_id,
        })?
    {
        return Err(receipt_error("cold artifact differs from its immutable recipe binding".into()));
    }
    let path = evidence_path(root, &binding.path)?;
    let digest = hash_file(&path)?;
    let event = bound_output_event(actions, &BoundOutputRequest {
        stage_id: "full-source-rust-provider",
        producer_action_id: &binding.producer_action_id,
        output_id: &binding.output_id,
    })?;
    if !event.produced_outputs.iter().any(|(id, observed)| id == &binding.output_id && observed == &digest) {
        return Err(receipt_error(format!("actual cold artifact {} differs from its producer", binding.path)));
    }
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn verify_cold_descriptor(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    core: &DeterministicBuildProofReceipt,
    extension: &SourceBuiltReceiptExtension,
    provider_size: u64,
) -> Result<(), RunError> {
    let descriptor = core
        .rebuild_descriptor
        .as_ref()
        .ok_or_else(|| receipt_error("missing cold rebuild descriptor".into()))?;
    let rustc = evidence_path(root, &extension.cold_provider.rustc.path)?;
    let identity = provider_identity_digest(&ProviderIdentityBinding {
        native_digest_blake3: &extension.native_provider_digest_blake3,
        rust_digest_blake3: &extension.rust_provider_digest_blake3,
        closure_digest_blake3: &extension.toolchain_closure_digest_blake3,
    });
    if descriptor.executable.digest_blake3 != hash_file(&rustc)? {
        return Err(receipt_error("rebuild descriptor differs from actual cold provider inputs".into()));
    }
    if descriptor.executable.size_bytes != fs::metadata(rustc).map_err(|error| receipt_error(error.to_string()))?.len()
    {
        return Err(receipt_error("rebuild descriptor differs from actual cold provider inputs".into()));
    }
    if descriptor.provider.digest_blake3 != identity {
        return Err(receipt_error("rebuild descriptor differs from actual cold provider inputs".into()));
    }
    if descriptor.provider.size_bytes != provider_size {
        return Err(receipt_error("rebuild descriptor differs from actual cold provider inputs".into()));
    }
    if descriptor.source_inputs.len() != proof.source_inputs.len() {
        return Err(receipt_error("rebuild descriptor differs from actual cold provider inputs".into()));
    }
    for source in &proof.source_inputs {
        let expected_name = format!("{:?}:{}", source.role, source.id);
        let expected_kind = match source.kind {
            crate::source_built_fixed_point::SourceContentKind::RegularFile => {
                crunch_release_core::RebuildContentKind::RegularFile
            }
            crate::source_built_fixed_point::SourceContentKind::Directory => {
                crunch_release_core::RebuildContentKind::Directory
            }
        };
        if !descriptor.source_inputs.iter().any(|observed| {
            observed.name == expected_name
                && observed.role == crunch_release_core::RebuildInputRole::Source
                && observed.kind == expected_kind
                && observed.digest_blake3 == source.digest_blake3
                && observed.size_bytes == source.size_bytes
        }) {
            return Err(receipt_error("cold rebuild descriptor source input drift".into()));
        }
    }
    debug_assert_eq!(descriptor.source_inputs.len(), proof.source_inputs.len());
    debug_assert_eq!(identity.len(), BLAKE3_HEX_LENGTH);
    Ok(())
}

pub(crate) fn observe_fixed_point(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    actions: &ActionEvidence,
) -> Result<FixedPointObservation, RunError> {
    let (stage1, first) = observe_mantle_stage(root, proof, actions, &MantleStageObservationRequest {
        stage_id: "mantle-stage1",
        manifest_path: MANTLE_STAGE1_MANIFEST,
    })?;
    let (stage2, second) = observe_mantle_stage(root, proof, actions, &MantleStageObservationRequest {
        stage_id: "mantle-stage2",
        manifest_path: MANTLE_STAGE2_MANIFEST,
    })?;
    let predecessor = Some((first.binary.producer_action_id.clone(), first.binary.output_id.clone()));
    if first.orchestrator.executable_producer.is_some()
        || second.orchestrator.executable_producer != predecessor
        || second.orchestrator.executable_digest_blake3 != first.binary.digest_blake3
    {
        return Err(receipt_error("stage2 is not orchestrated by the observed stage1 producer output".into()));
    }
    if stage1.binary_digest_blake3 != stage2.binary_digest_blake3 {
        return Err(receipt_error("stage1 and stage2 Mantle binary digests differ".into()));
    }
    debug_assert!(first.orchestrator.executable_producer.is_none());
    debug_assert_eq!(second.orchestrator.executable_digest_blake3, first.binary.digest_blake3);
    Ok(FixedPointObservation {
        stage1,
        stage2,
        closure_policy_digest_blake3: proof.policies.closure_policy_digest_blake3.clone(),
    })
}

/// Which Mantle stage to observe and where its manifest lives.
struct MantleStageObservationRequest<'a> {
    stage_id: &'a str,
    manifest_path: &'a str,
}

fn observe_mantle_stage(
    root: &Path,
    proof: &SourceBuiltFixedPointPlan,
    actions: &ActionEvidence,
    request: &MantleStageObservationRequest<'_>,
) -> Result<(FixedPointStageObservation, MantleStageManifest), RunError> {
    let stage_id = request.stage_id;
    let manifest_path = request.manifest_path;
    let (manifest, manifest_digest) = read_bound_json::<MantleStageManifest>(root, manifest_path)?;
    let manifest_matches = [
        manifest.schema == MANTLE_STAGE_MANIFEST_SCHEMA,
        manifest.stage_id == stage_id,
        manifest.proof_plan_digest_blake3 == proof.plan_digest_blake3,
        manifest.action_plan_digest_blake3 == actions.plan.digest_blake3,
        manifest.closure_policy_digest_blake3 == proof.policies.closure_policy_digest_blake3,
        manifest.semantic_unit_count != 0,
        manifest.semantic_unit_count <= proof.resource_bounds.protected_exec_events_max,
    ];
    if !manifest_matches.iter().all(|matches| *matches) {
        return Err(receipt_error(format!("{stage_id} manifest source/action/policy identity mismatch")));
    }
    let stage_actions = stage_action_ids(actions, &manifest.stage_id)?;
    validate_mantle_manifest_actions(&manifest, actions, &stage_actions)?;
    let parent = manifest_path.rsplit_once('/').ok_or_else(|| receipt_error("stage manifest lacks parent".into()))?.0;
    let binary_relative_path = stage_relative_file(&StageFileRequest {
        parent,
        path: &manifest.binary.path,
    })?;
    let binary_path = evidence_path(root, &binary_relative_path)?;
    if hash_file(&binary_path)? != manifest.binary.digest_blake3 {
        return Err(receipt_error(format!("{stage_id} actual binary content drift")));
    }
    let command_path = stage_relative_file(&StageFileRequest {
        parent,
        path: &manifest.command_receipt.path,
    })?;
    let (commands, command_digest) = read_bound_json::<MantleCommandReceipts>(root, &command_path)?;
    if command_digest != manifest.command_receipt.digest_blake3 || commands.schema != MANTLE_COMMAND_RECEIPTS_SCHEMA {
        return Err(receipt_error(format!("{stage_id} command receipt content or schema drift")));
    }
    let mut audit_files = vec![(command_path, command_digest)];
    let mut verification = MantleCommandVerification {
        root,
        parent,
        manifest: &manifest,
        receipt: &commands,
        actions,
        ids: &stage_actions,
        audits: &mut audit_files,
    };
    verify_mantle_commands(&mut verification)?;
    debug_assert_eq!(manifest.stage_id, stage_id);
    debug_assert_eq!(manifest_digest.len(), BLAKE3_HEX_LENGTH);
    Ok((
        FixedPointStageObservation {
            binary_relative_path,
            binary_digest_blake3: manifest.binary.digest_blake3.clone(),
            manifest_relative_path: manifest_path.into(),
            manifest_digest_blake3: manifest_digest,
            orchestrator_digest_blake3: manifest.orchestrator.executable_digest_blake3.clone(),
            audit_files,
        },
        manifest,
    ))
}

/// A stage-relative file path together with the manifest directory holding it.
struct StageFileRequest<'a> {
    parent: &'a str,
    path: &'a str,
}

fn stage_relative_file(request: &StageFileRequest<'_>) -> Result<String, RunError> {
    let path = request.path;
    if path.is_empty()
        || path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || !Path::new(path).components().all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(receipt_error(format!("stage file path is not normalized and contained: {path}")));
    }
    Ok(format!("{}/{path}", request.parent))
}

fn stage_action_ids<'a>(actions: &'a ActionEvidence, stage_id: &str) -> Result<BTreeSet<&'a str>, RunError> {
    let adapter = actions
        .plan
        .adapters
        .iter()
        .find(|adapter| adapter.stage_id == stage_id)
        .ok_or_else(|| receipt_error(format!("missing {stage_id} action adapter")))?;
    let mut ids = adapter.actions.iter().map(|action| action.id.as_str()).collect::<BTreeSet<_>>();
    for event in &actions.events {
        if crate::source_built_fixed_point_action_trust::adapter_action_spec(adapter, &event.action_id)
            .map_err(receipt_error)?
            .is_some()
        {
            ids.insert(event.action_id.as_str());
        }
    }
    Ok(ids)
}

fn validate_mantle_manifest_actions(
    manifest: &MantleStageManifest,
    actions: &ActionEvidence,
    stage_actions: &BTreeSet<&str>,
) -> Result<(), RunError> {
    let manifest_actions_are_local = [
        stage_actions.contains(manifest.binary.producer_action_id.as_str()),
        stage_actions.contains(manifest.orchestrator.action_id.as_str()),
        stage_actions.contains(manifest.smoke.action_id.as_str()),
        manifest.smoke.exit_code == 0,
        manifest.binary.output_id == manifest.stage_id,
    ];
    if !manifest_actions_are_local.iter().all(|local| *local) {
        return Err(receipt_error(format!(
            "{} manifest names foreign producer/orchestrator/smoke authority",
            manifest.stage_id
        )));
    }
    let is_binary_produced = actions.events.iter().any(|event| {
        event.action_id == manifest.binary.producer_action_id
            && event.kind == ExecutionEventKind::BuildExecution
            && event
                .produced_outputs
                .iter()
                .any(|(id, digest)| id == &manifest.binary.output_id && digest == &manifest.binary.digest_blake3)
    });
    let is_orchestrated = actions.events.iter().any(|event| {
        event.action_id == manifest.orchestrator.action_id
            && event.executable_digest_blake3 == manifest.orchestrator.executable_digest_blake3
            && event.executable_producer == manifest.orchestrator.executable_producer
            && event.kind == ExecutionEventKind::ProtectedExec
    });
    let predecessor = Some((manifest.binary.producer_action_id.clone(), manifest.binary.output_id.clone()));
    let is_smoked = actions.events.iter().any(|event| {
        event.action_id == manifest.smoke.action_id
            && event.kind == ExecutionEventKind::ProtectedExec
            && event.executable_digest_blake3 == manifest.binary.digest_blake3
            && event.executable_producer == predecessor
    });
    if !is_binary_produced || !is_orchestrated || !is_smoked {
        return Err(receipt_error(format!(
            "{} manifest lacks observed binary publication, orchestration or smoke",
            manifest.stage_id
        )));
    }
    debug_assert!(is_binary_produced);
    debug_assert!(is_orchestrated);
    debug_assert!(is_smoked);
    Ok(())
}

/// The Mantle stage whose recorded command completions are verified, and the
/// audit trail the verification produces.
struct MantleCommandVerification<'a> {
    root: &'a Path,
    parent: &'a str,
    manifest: &'a MantleStageManifest,
    receipt: &'a MantleCommandReceipts,
    actions: &'a ActionEvidence,
    ids: &'a BTreeSet<&'a str>,
    audits: &'a mut Vec<(String, String)>,
}

fn verify_mantle_commands(request: &mut MantleCommandVerification<'_>) -> Result<(), RunError> {
    let manifest = request.manifest;
    let ids = request.ids;
    let events = &request.actions.events;
    let mut observed = events.iter().filter(|event| {
        ids.contains(event.action_id.as_str())
            && event.kind == ExecutionEventKind::BuildExecution
            && event.produced_outputs.iter().any(|(id, _)| id == "stdout" || id == "stderr")
    });
    if request.receipt.commands.is_empty() || request.receipt.commands.len() > events.len() {
        return Err(receipt_error(format!("{} command receipt omits or adds observed completions", manifest.stage_id)));
    }
    let mut stderr_hasher = blake3::Hasher::new();
    let mut is_smoke_seen = false;
    for command in &request.receipt.commands {
        let event = observed.next().ok_or_else(|| {
            receipt_error(format!("{} command receipt adds unobserved completion", manifest.stage_id))
        })?;
        if command.action_id != event.action_id
            || !event
                .produced_outputs
                .iter()
                .any(|(id, digest)| id == "stdout" && digest == &command.stdout.digest_blake3)
            || !event
                .produced_outputs
                .iter()
                .any(|(id, digest)| id == "stderr" && digest == &command.stderr.digest_blake3)
        {
            return Err(receipt_error(format!("{} command order or observed log identity drift", manifest.stage_id)));
        }
        request.audits.push(verify_stage_log(request.root, request.parent, &command.stdout, None)?);
        request
            .audits
            .push(verify_stage_log(request.root, request.parent, &command.stderr, Some(&mut stderr_hasher))?);
        if command.action_id == manifest.smoke.action_id {
            is_smoke_seen = command.exit_code == 0
                && command.stdout == manifest.smoke.stdout
                && command.stderr == manifest.smoke.stderr;
        }
    }
    if observed.next().is_some() {
        return Err(receipt_error(format!("{} command receipt omits observed completion", manifest.stage_id)));
    }
    if !is_smoke_seen || stderr_hasher.finalize().to_hex().as_str() != manifest.stderr.digest_blake3 {
        return Err(receipt_error(format!(
            "{} smoke completion or aggregate stderr differs from actual command logs",
            manifest.stage_id
        )));
    }
    request.audits.push(verify_stage_log(request.root, request.parent, &manifest.stderr, None)?);
    debug_assert!(is_smoke_seen);
    debug_assert!(!request.receipt.commands.is_empty());
    Ok(())
}

fn verify_stage_log(
    root: &Path,
    parent: &str,
    measured: &MantleMeasuredFile,
    mut aggregate: Option<&mut blake3::Hasher>,
) -> Result<(String, String), RunError> {
    let relative = stage_relative_file(&StageFileRequest {
        parent,
        path: &measured.path,
    })?;
    let path = evidence_path(root, &relative)?;
    let mut file =
        fs::File::open(path).map_err(|error| receipt_error(format!("opening stage log {relative}: {error}")))?;
    if !file.metadata().map_err(|error| receipt_error(error.to_string()))?.is_file() {
        return Err(receipt_error(format!("stage log {relative} is not a regular file")));
    }
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    for _ in 0..HASH_READ_CHUNK_LIMIT {
        let count = file.read(&mut buffer).map_err(|error| receipt_error(error.to_string()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        if let Some(combined) = aggregate.as_mut() {
            combined.update(&buffer[..count]);
        }
    }
    let digest = hasher.finalize().to_hex().to_string();
    if digest != measured.digest_blake3 {
        return Err(receipt_error(format!("stage log {relative} content drift")));
    }
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert_eq!(digest, measured.digest_blake3);
    Ok((relative, digest))
}
pub(crate) fn validate_stage_evidence(evidence: &[SourceBuiltStageEvidence]) -> Result<(), RunError> {
    if evidence.len() != STAGE_EVIDENCE_COUNT {
        return Err(receipt_error(format!(
            "stage evidence must contain {STAGE_EVIDENCE_COUNT} stages, got {}",
            evidence.len()
        )));
    }
    let mut ids = BTreeSet::new();
    for stage in evidence {
        if !ids.insert(stage.stage_id.as_str()) {
            return Err(receipt_error(format!("duplicate stage evidence id {}", stage.stage_id)));
        }
        for (label, digest) in [
            ("transcript", stage.transcript_digest_blake3.as_str()),
            ("output", stage.output_digest_blake3.as_str()),
            ("recipe journal", stage.execution_journal_digest_blake3.as_str()),
        ] {
            validate_digest(&DigestExpectation {
                label: &format!("{} {label}", stage.stage_id),
                digest,
            })?;
        }
        let stage_is_complete = [
            stage.status == STAGE_STATUS_COMPLETE,
            stage.authority_violations.is_empty(),
            stage.fallback_events.is_empty(),
            stage.audit_paths.len() == stage.audit_digests_blake3.len(),
            !stage.producer_action_id.is_empty(),
            !stage.output_id.is_empty(),
            !stage.execution_journal_path.is_empty(),
        ];
        if !stage_is_complete.iter().all(|complete| *complete) {
            return Err(receipt_error(format!("stage evidence {} is incomplete", stage.stage_id)));
        }
        for digest in &stage.audit_digests_blake3 {
            validate_digest(&DigestExpectation {
                label: "stage audit",
                digest,
            })?;
        }
    }
    assert_eq!(ids.len(), STAGE_EVIDENCE_COUNT);
    debug_assert!(evidence.iter().all(|stage| stage.status == STAGE_STATUS_COMPLETE));
    Ok(())
}

pub(crate) fn validate_stage_plan_alignment(
    plan: &SourceBuiltFixedPointPlan,
    evidence: &[SourceBuiltStageEvidence],
) -> Result<(), RunError> {
    if plan.stages.len() != evidence.len() {
        return Err(receipt_error("plan and stage evidence counts differ".to_string()));
    }
    for (planned, observed) in plan.stages.iter().zip(evidence) {
        let orchestrator = match planned.orchestrator {
            crate::source_built_fixed_point::ProofOrchestrator::HostMantle => "host-mantle",
            crate::source_built_fixed_point::ProofOrchestrator::Stage1Mantle => "stage1-mantle",
        };
        if planned.stage_id != observed.stage_id
            || planned.output != observed.output_role
            || observed.orchestrator != orchestrator
        {
            return Err(receipt_error(format!(
                "stage evidence does not match plan: planned {} {:?}, observed {} {:?}",
                planned.stage_id, planned.output, observed.stage_id, observed.output_role
            )));
        }
    }
    assert_eq!(plan.stages.len(), STAGE_EVIDENCE_COUNT);
    debug_assert_eq!(evidence.len(), STAGE_EVIDENCE_COUNT);
    Ok(())
}

pub(crate) fn receipt_arguments(
    plan: &SourceBuiltFixedPointPlan,
    closure_digest: &str,
    extension: &SourceBuiltReceiptExtension,
) -> Result<Vec<String>, RunError> {
    Ok(vec![
        "workflow:source-built-fixed-point".to_string(),
        format!("plan-blake3:{}", plan.plan_digest_blake3),
        format!("source-authority-blake3:{}", plan.source_authority_digest_blake3),
        format!("closure-policy-blake3:{closure_digest}"),
        format!("source-built-extension-blake3:{}", receipt_extension_digest(extension)?),
    ])
}

fn receipt_extension_digest(extension: &SourceBuiltReceiptExtension) -> Result<String, RunError> {
    let mut hasher = blake3::Hasher::new_derive_key("mantle-source-built-fixed-point-receipt-extension-v2");
    serde_json::to_writer(&mut hasher, extension)
        .map_err(|error| receipt_error(format!("serializing receipt extension: {error}")))?;
    Ok(hasher.finalize().to_hex().to_string())
}
/// The three provider digests that identify one cold toolchain closure. Named
/// fields keep the native, rust, and closure roles from being transposed.
pub(crate) struct ProviderIdentityBinding<'a> {
    pub(crate) native_digest_blake3: &'a str,
    pub(crate) rust_digest_blake3: &'a str,
    pub(crate) closure_digest_blake3: &'a str,
}

pub(crate) fn provider_identity_digest(binding: &ProviderIdentityBinding<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PROVIDER_IDENTITY_DIGEST_DOMAIN);
    for (role, digest) in [
        ("native", binding.native_digest_blake3),
        ("rust", binding.rust_digest_blake3),
        ("closure", binding.closure_digest_blake3),
    ] {
        hasher.update(role.as_bytes());
        hasher.update(&[0]);
        hasher.update(digest.as_bytes());
        hasher.update(&[0]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(
        binding.native_digest_blake3 != binding.rust_digest_blake3
            || binding.rust_digest_blake3 != binding.closure_digest_blake3
    );
    digest
}
pub(crate) fn proof_bundle_digest(proof_root: &Path) -> Result<ProofBundleObservation, RunError> {
    let mut entries = Vec::new();
    collect_bundle_files(proof_root, proof_root, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.is_empty() || entries.len() > PROOF_ENTRY_COUNT_MAX {
        return Err(receipt_error(format!(
            "proof bundle entry count must be within 1..={PROOF_ENTRY_COUNT_MAX}, got {}",
            entries.len()
        )));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECEIPT_BUNDLE_DIGEST_DOMAIN);
    for (relative, path, symlink_target) in &entries {
        hasher.update(relative.as_bytes());
        hasher.update(&[0]);
        if let Some(target) = symlink_target {
            hasher.update(b"symlink\0");
            hasher.update(target.as_os_str().as_encoded_bytes());
        } else {
            hasher.update(b"file\0");
            hasher.update(hash_file_allow_empty(path)?.as_bytes());
        }
        hasher.update(&[0]);
    }
    let digest_blake3 = hasher.finalize().to_hex().to_string();
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!entries.is_empty());
    Ok(ProofBundleObservation {
        digest_blake3,
        entry_count: u32::try_from(entries.len())
            .map_err(|_| receipt_error("proof bundle count exceeds u32".into()))?,
    })
}

fn sorted_bundle_children(directory: &Path) -> Result<std::vec::IntoIter<std::fs::DirEntry>, RunError> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| receipt_error(format!("reading proof bundle directory {}: {error}", directory.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| receipt_error(format!("reading proof bundle entry: {error}")))?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    Ok(children.into_iter())
}

fn collect_bundle_files(
    proof_root: &Path,
    current: &Path,
    entries: &mut Vec<(String, PathBuf, Option<PathBuf>)>,
) -> Result<(), RunError> {
    if entries.len() > PROOF_ENTRY_COUNT_MAX {
        return Err(receipt_error("proof bundle entry limit exceeded".to_string()));
    }
    let mut stack: Vec<std::vec::IntoIter<std::fs::DirEntry>> = Vec::with_capacity(64);
    stack.push(sorted_bundle_children(current)?);
    for _ in 0..PROOF_BUNDLE_WALK_STEPS {
        let next = match stack.last_mut() {
            Some(children) => children.next(),
            None => break,
        };
        let Some(child) = next else {
            stack.pop();
            continue;
        };
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| receipt_error(format!("reading proof bundle metadata {}: {error}", path.display())))?;
        if metadata.file_type().is_symlink() {
            let relative = relative_path(proof_root, &path)?;
            if !excluded_from_bundle_digest(&relative) {
                let target = fs::read_link(&path).map_err(|error| {
                    receipt_error(format!("reading proof bundle symlink {}: {error}", path.display()))
                })?;
                entries.push((relative, path, Some(target)));
            }
            continue;
        }
        if metadata.is_dir() {
            if entries.len() > PROOF_ENTRY_COUNT_MAX {
                return Err(receipt_error("proof bundle entry limit exceeded".to_string()));
            }
            if stack.len() >= PROOF_BUNDLE_WALK_DEPTH_LIMIT {
                return Err(receipt_error("proof bundle entry limit exceeded".to_string()));
            }
            stack.push(sorted_bundle_children(&path)?);
            continue;
        }
        if !metadata.is_file() {
            return Err(receipt_error(format!("proof bundle contains unsupported entry: {}", path.display())));
        }
        let relative = relative_path(proof_root, &path)?;
        if excluded_from_bundle_digest(&relative) {
            continue;
        }
        entries.push((relative, path, None));
    }
    assert!(entries.len() <= PROOF_ENTRY_COUNT_MAX);
    debug_assert!(current.starts_with(proof_root));
    Ok(())
}

fn excluded_from_bundle_digest(relative: &str) -> bool {
    matches!(relative, FINAL_RECEIPT_FILE | FINAL_BUNDLE_DIGEST_FILE | "attempt-status.json")
}
pub(crate) fn hash_file(path: &Path) -> Result<String, RunError> {
    let metadata = fs::metadata(path)
        .map_err(|error| receipt_error(format!("reading file metadata {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(receipt_error(format!("required receipt file is missing or empty: {}", path.display())));
    }
    hash_file_allow_empty(path)
}

pub(crate) fn hash_file_allow_empty(path: &Path) -> Result<String, RunError> {
    let mut file =
        fs::File::open(path).map_err(|error| receipt_error(format!("opening file {}: {error}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; HASH_BUFFER_BYTES];
    for _ in 0..HASH_READ_CHUNK_LIMIT {
        let count = file
            .read(&mut buffer)
            .map_err(|error| receipt_error(format!("reading file {}: {error}", path.display())))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(path.is_file());
    Ok(digest)
}
/// A labelled digest that must be a lowercase BLAKE3 hex string.
struct DigestExpectation<'a> {
    label: &'a str,
    digest: &'a str,
}

fn validate_digest(expectation: &DigestExpectation<'_>) -> Result<(), RunError> {
    let is_valid = expectation.digest.len() == BLAKE3_HEX_LENGTH
        && expectation.digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_valid {
        return Err(receipt_error(format!("{} is not a lowercase BLAKE3 digest", expectation.label)));
    }
    assert!(!expectation.label.is_empty());
    debug_assert!(is_valid);
    Ok(())
}

pub(crate) fn relative_path(root: &Path, path: &Path) -> Result<String, RunError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| receipt_error(format!("path {} is outside proof root {}", path.display(), root.display())))?;
    let text = relative
        .to_str()
        .ok_or_else(|| receipt_error(format!("proof-relative path is not UTF-8: {}", relative.display())))?;
    if text.is_empty() || text.starts_with('/') || text.split('/').any(|component| component == "..") {
        return Err(receipt_error(format!("invalid proof-relative path: {text}")));
    }
    assert!(!text.is_empty());
    debug_assert!(!Path::new(text).is_absolute());
    Ok(text.to_string())
}
/// Development adoption/restoration is never cold source-construction evidence.
/// Presence is disqualifying, even for malformed markers or dangling symlinks.
pub(crate) fn reject_development_authority(proof_root: &Path) -> Result<(), RunError> {
    let mut absent_markers = 0_usize;
    for marker in [
        "dev-run-report.json",
        "dev-execution-report.json",
        "dev-adopted-report.json",
        "dev-cache-hit.json",
        "dev-resume-authority.json",
        "dev-resume-plan.json",
    ] {
        match fs::symlink_metadata(proof_root.join(marker)) {
            Ok(_) => {
                return Err(receipt_error(format!(
                    "development/cache/restored authority marker {marker} cannot establish a promoted cold proof"
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(receipt_error(format!("inspecting development authority marker {marker}: {error}")));
            }
        }
        absent_markers = absent_markers.saturating_add(1);
    }
    debug_assert_eq!(absent_markers, 6);
    debug_assert!(!proof_root.as_os_str().is_empty());
    Ok(())
}

pub(crate) fn receipt_error(message: String) -> RunError {
    RunError::Build(format!("source-built fixed-point receipt blocked: {message}"))
}
#[cfg(test)]
mod receipt_tests {
    use super::*;

    pub(crate) const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    pub(crate) const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn stage_artifact_paths_cannot_escape_or_hide_noncanonical_components() {
        let parent = "cargo-free-fixed-point/stage1";
        assert_eq!(
            stage_relative_file(&StageFileRequest {
                parent,
                path: "recipe/compile/outputs/mantle"
            })
            .unwrap(),
            "cargo-free-fixed-point/stage1/recipe/compile/outputs/mantle"
        );
        for path in [
            "../stage2/mantle",
            "/producer-checkout/mantle",
            "recipe/../mantle",
            "recipe/./mantle",
            "recipe//mantle",
        ] {
            assert!(
                stage_relative_file(&StageFileRequest { parent, path })
                    .unwrap_err()
                    .to_string()
                    .contains("not normalized and contained")
            );
        }
    }

    #[test]
    fn actual_command_stderr_is_hashed_in_order_and_tampering_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("stage")).unwrap();
        fs::write(root.path().join("stage/first.stderr"), b"first\n").unwrap();
        fs::write(root.path().join("stage/second.stderr"), b"second\n").unwrap();
        let first = MantleMeasuredFile {
            path: "first.stderr".into(),
            digest_blake3: blake3::hash(b"first\n").to_hex().to_string(),
        };
        let second = MantleMeasuredFile {
            path: "second.stderr".into(),
            digest_blake3: blake3::hash(b"second\n").to_hex().to_string(),
        };
        let mut aggregate = blake3::Hasher::new();
        verify_stage_log(root.path(), "stage", &first, Some(&mut aggregate)).unwrap();
        verify_stage_log(root.path(), "stage", &second, Some(&mut aggregate)).unwrap();
        assert_eq!(aggregate.finalize(), blake3::hash(b"first\nsecond\n"));
        assert_ne!(aggregate.finalize(), blake3::hash(b"second\nfirst\n"));
        fs::write(root.path().join("stage/second.stderr"), b"changed\n").unwrap();
        assert!(
            verify_stage_log(root.path(), "stage", &second, None)
                .unwrap_err()
                .to_string()
                .contains("content drift")
        );
    }

    #[test]
    fn machine_local_cache_receipt_cannot_occupy_stage_command_evidence_slot() {
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("commands.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema": "mantle-cc-driver-receipt-v1", "claim_class": "machine-local-performance-only",
                "proof_evidence": false,
            }))
            .unwrap(),
        )
        .unwrap();
        let error = read_bound_json::<MantleCommandReceipts>(root.path(), "commands.json").unwrap_err().to_string();
        assert!(error.contains("parsing commands.json"));
        assert!(error.contains("unknown field") || error.contains("missing field"));
    }

    #[test]
    fn promoted_receipts_reject_every_development_authority_marker() {
        for marker in [
            "dev-run-report.json",
            "dev-execution-report.json",
            "dev-adopted-report.json",
            "dev-cache-hit.json",
            "dev-resume-authority.json",
            "dev-resume-plan.json",
        ] {
            let root = tempfile::tempdir().unwrap();
            reject_development_authority(root.path()).unwrap();
            fs::write(root.path().join(marker), "malformed is still development authority").unwrap();
            let receipt = root.path().join(FINAL_RECEIPT_FILE);
            let error = verify_source_built_fixed_point_receipt(root.path(), &receipt).unwrap_err().to_string();
            assert!(error.contains(marker));
            assert!(error.contains("cannot establish a promoted cold proof"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn dangling_development_marker_is_not_absent_authority() {
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("missing-target", root.path().join("dev-run-report.json")).unwrap();
        assert!(!root.path().join("dev-run-report.json").exists());
        let error = reject_development_authority(root.path()).unwrap_err().to_string();
        assert!(error.contains("dev-run-report.json"));
    }

    #[test]
    fn receipt_extension_digest_binds_full_action_records_and_reconciliation() {
        let mut extension = SourceBuiltReceiptExtension {
            schema: EXTENSION_SCHEMA.into(),
            plan_digest_blake3: DIGEST_A.into(),
            source_authority_digest_blake3: DIGEST_A.into(),
            provider_kind: PROOF_PROVIDER_KIND.into(),
            native_provider_digest_blake3: DIGEST_A.into(),
            rust_provider_digest_blake3: DIGEST_A.into(),
            toolchain_closure_digest_blake3: DIGEST_A.into(),
            stage_evidence_path: STAGE_EVIDENCE_FILE.into(),
            cold_provider: ColdProviderEvidence {
                native_provider_path: "native".into(),
                rust_provider_path: "rust".into(),
                rustc: ColdArtifactBinding {
                    path: "rustc".into(),
                    producer_action_id: "rust/compile".into(),
                    output_id: "rustc".into(),
                },
                toolchain_closure: ColdArtifactBinding {
                    path: "closure".into(),
                    producer_action_id: "rust/closure".into(),
                    output_id: "closure".into(),
                },
            },
            stage_evidence_digest_blake3: DIGEST_A.into(),
            final_proof_bundle_digest_blake3: DIGEST_A.into(),
            protected_execution_policy_digest_blake3: DIGEST_A.into(),
            effect_policy_digest_blake3: DIGEST_A.into(),
            normalization_policy_digest_blake3: DIGEST_A.into(),
            non_claims: Vec::new(),
            action_evidence: ActionEvidenceBinding {
                proof_root_identity_blake3: DIGEST_A.into(),
                action_plan_digest_blake3: DIGEST_A.into(),
                action_plan_file_digest_blake3: DIGEST_A.into(),
                observed_events_digest_blake3: DIGEST_A.into(),
                observed_events_file_digest_blake3: DIGEST_A.into(),
                reconciliation_digest_blake3: DIGEST_A.into(),
                observed_journal_file_digest_blake3: DIGEST_A.into(),
                reconciliation_file_digest_blake3: DIGEST_A.into(),
                observed_event_count: 12,
            },
        };
        let original = receipt_extension_digest(&extension).unwrap();
        extension.action_evidence.observed_events_file_digest_blake3 = DIGEST_B.into();
        assert_ne!(original, receipt_extension_digest(&extension).unwrap());
        extension.action_evidence.observed_events_file_digest_blake3 = DIGEST_A.into();
        extension.action_evidence.observed_journal_file_digest_blake3 = DIGEST_B.into();
        assert_ne!(original, receipt_extension_digest(&extension).unwrap());
        extension.action_evidence.observed_journal_file_digest_blake3 = DIGEST_A.into();
        extension.action_evidence.reconciliation_digest_blake3 = DIGEST_B.into();
        assert_ne!(original, receipt_extension_digest(&extension).unwrap());
        extension.action_evidence.reconciliation_digest_blake3 = DIGEST_A.into();
        extension.action_evidence.observed_event_count -= 1;
        assert_ne!(original, receipt_extension_digest(&extension).unwrap());
    }

    #[test]
    fn empty_audit_files_are_bound_in_bundle_digest() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("stderr.txt"), "").unwrap();
        let empty = proof_bundle_digest(root.path()).unwrap();
        fs::write(root.path().join("stderr.txt"), "violation").unwrap();
        let violation = proof_bundle_digest(root.path()).unwrap();
        assert_ne!(empty.digest_blake3, violation.digest_blake3);
    }

    #[test]
    fn stage_evidence_rejects_fallbacks_and_duplicate_ids() {
        let mut evidence = (0..STAGE_EVIDENCE_COUNT)
            .map(|index| SourceBuiltStageEvidence {
                stage_id: format!("stage-{index}"),
                output_role: ProofOutputRole::StagexTransition,
                status: STAGE_STATUS_COMPLETE.to_string(),
                orchestrator: "host-mantle".to_string(),
                executable_identity: format!("blake3:{DIGEST_A}"),
                transcript_path: format!("stage-{index}.json"),
                transcript_digest_blake3: DIGEST_A.to_string(),
                audit_paths: vec![format!("audit-{index}.json")],
                audit_digests_blake3: vec![DIGEST_B.to_string()],
                output_path: format!("output-{index}"),
                output_digest_blake3: DIGEST_B.to_string(),
                producer_action_id: format!("stage-{index}/produce"),
                output_id: format!("stage-{index}"),
                execution_journal_path: format!("stage-{index}/execution.jsonl"),
                execution_journal_digest_blake3: DIGEST_A.into(),
                authority_violations: Vec::new(),
                fallback_events: Vec::new(),
            })
            .collect::<Vec<_>>();
        validate_stage_evidence(&evidence).unwrap();
        evidence[0].fallback_events.push("host-tool-fallback".to_string());
        let fallback = validate_stage_evidence(&evidence).unwrap_err();
        evidence[0].fallback_events.clear();
        evidence[1].stage_id = evidence[0].stage_id.clone();
        let duplicate = validate_stage_evidence(&evidence).unwrap_err();

        assert!(fallback.to_string().contains("incomplete"));
        assert!(duplicate.to_string().contains("duplicate stage evidence id"));
    }

    #[test]
    fn bundle_digest_excludes_only_self_referential_receipt_files() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("evidence.txt"), "evidence").unwrap();
        fs::write(temp.path().join(FINAL_RECEIPT_FILE), "one").unwrap();
        fs::write(temp.path().join(FINAL_BUNDLE_DIGEST_FILE), "two").unwrap();
        fs::write(temp.path().join("attempt-status.json"), "running").unwrap();
        let first = proof_bundle_digest(temp.path()).unwrap();
        fs::write(temp.path().join(FINAL_RECEIPT_FILE), "changed").unwrap();
        fs::write(temp.path().join("attempt-status.json"), "complete").unwrap();
        let second = proof_bundle_digest(temp.path()).unwrap();
        fs::write(temp.path().join("evidence.txt"), "changed evidence").unwrap();
        let changed = proof_bundle_digest(temp.path()).unwrap();

        assert_eq!(first.digest_blake3, second.digest_blake3);
        assert_ne!(second.digest_blake3, changed.digest_blake3);
    }

    #[test]
    fn provider_identity_digest_binds_each_declared_role() {
        let first = provider_identity_digest(&ProviderIdentityBinding {
            native_digest_blake3: DIGEST_A,
            rust_digest_blake3: DIGEST_B,
            closure_digest_blake3: DIGEST_A,
        });
        let repeated = provider_identity_digest(&ProviderIdentityBinding {
            native_digest_blake3: DIGEST_A,
            rust_digest_blake3: DIGEST_B,
            closure_digest_blake3: DIGEST_A,
        });
        let swapped = provider_identity_digest(&ProviderIdentityBinding {
            native_digest_blake3: DIGEST_B,
            rust_digest_blake3: DIGEST_A,
            closure_digest_blake3: DIGEST_A,
        });
        let substituted = provider_identity_digest(&ProviderIdentityBinding {
            native_digest_blake3: DIGEST_A,
            rust_digest_blake3: DIGEST_B,
            closure_digest_blake3: DIGEST_B,
        });

        assert_eq!(first, repeated);
        assert_ne!(first, swapped);
        assert_ne!(first, substituted);
        assert_eq!(first.len(), BLAKE3_HEX_LENGTH);
    }
}
