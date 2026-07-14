use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Serialize;

use crate::digest::EVIDENCE_REF_PREFIX;
use crate::digest::digest_ref;
use crate::model::ActionNode;
use crate::model::ActionStage;
use crate::model::EVIDENCE_BUNDLE_SCHEMA;
use crate::model::HardwareEvidenceBundle;
use crate::model::RunClass;
use crate::model::RunEvidence;
use crate::model::StageCounts;
use crate::profile::HARD_MAX_ACTIONS;
use crate::profile::REQUIRED_NON_CLAIMS;

pub const EVIDENCE_NON_CLAIMS: &[&str] = &[
    "elapsed-diagnostics-are-not-correctness-thresholds",
    "no-specific-speedup-promise",
    "no-production-capacity-claim",
    "no-simulator-license-saving-claim",
];

const EVIDENCE_DOMAIN: &[u8] = b"mantle.hardware.evidence.v1";
const INVALIDATION_STEP_MULTIPLIER: u32 = 2;
const REQUIRED_STAGE_COUNT: usize = 4;

#[derive(Serialize)]
struct EvidenceHashable<'a> {
    schema: &'static str,
    fresh: &'a RunEvidence,
    selected_source_change: &'a RunEvidence,
    unrelated_source_change: &'a RunEvidence,
    full_shared_hit: &'a RunEvidence,
    non_claims: &'a [String],
}

pub fn invalidated_actions(action_graph: &[ActionNode], changed_refs: &[String]) -> Result<Vec<String>, String> {
    let action_count = u32::try_from(action_graph.len()).unwrap_or(u32::MAX);
    if action_count == 0 || action_count > HARD_MAX_ACTIONS {
        return Err(String::from("invalidation-action-count-invalid"));
    }
    let changed = changed_refs.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut invalidated = action_graph
        .iter()
        .filter(|node| node.direct_input_refs.iter().any(|reference| changed.contains(reference.as_str())))
        .map(|node| node.action_ref.clone())
        .collect::<BTreeSet<_>>();
    let step_limit = action_count.saturating_mul(INVALIDATION_STEP_MULTIPLIER);
    for _ in 0..step_limit {
        let before = invalidated.len();
        for node in action_graph {
            if node.dependency_action_refs.iter().any(|dependency| invalidated.contains(dependency)) {
                invalidated.insert(node.action_ref.clone());
            }
        }
        if invalidated.len() == before {
            break;
        }
    }
    if invalidated.len() > action_graph.len() {
        return Err(String::from("invalidation-result-count-invalid"));
    }
    debug_assert!(invalidated.len() <= action_graph.len());
    debug_assert!(step_limit >= action_count);
    Ok(invalidated.into_iter().collect())
}

pub fn build_evidence_bundle(
    action_graph: &[ActionNode],
    selected_changed_refs: &[String],
    fresh: RunEvidence,
    selected_source_change: RunEvidence,
    unrelated_source_change: RunEvidence,
    full_shared_hit: RunEvidence,
) -> Result<HardwareEvidenceBundle, Vec<String>> {
    let expected_selected = invalidated_actions(action_graph, selected_changed_refs).map_err(|error| vec![error])?;
    let mut diagnostics = Vec::new();
    validate_run(&fresh, RunClass::Fresh, &mut diagnostics);
    validate_run(&selected_source_change, RunClass::SelectedSourceChange, &mut diagnostics);
    validate_run(&unrelated_source_change, RunClass::UnrelatedSourceChange, &mut diagnostics);
    validate_run(&full_shared_hit, RunClass::FullSharedHit, &mut diagnostics);
    validate_fresh(&fresh, &mut diagnostics);
    validate_selected_change(&selected_source_change, &expected_selected, &mut diagnostics);
    validate_unrelated_change(&unrelated_source_change, &mut diagnostics);
    validate_full_shared_hit(&full_shared_hit, &mut diagnostics);
    let non_claims = required_evidence_non_claims();
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }
    let hashable = EvidenceHashable {
        schema: EVIDENCE_BUNDLE_SCHEMA,
        fresh: &fresh,
        selected_source_change: &selected_source_change,
        unrelated_source_change: &unrelated_source_change,
        full_shared_hit: &full_shared_hit,
        non_claims: &non_claims,
    };
    let evidence_ref = digest_ref(EVIDENCE_REF_PREFIX, EVIDENCE_DOMAIN, &hashable).map_err(|error| vec![error])?;
    debug_assert!(evidence_ref.starts_with(EVIDENCE_REF_PREFIX));
    debug_assert!(!non_claims.is_empty());
    Ok(HardwareEvidenceBundle {
        schema: String::from(EVIDENCE_BUNDLE_SCHEMA),
        evidence_ref,
        fresh,
        selected_source_change,
        unrelated_source_change,
        full_shared_hit,
        non_claims,
    })
}

pub fn stage_counts(
    action_graph: &[ActionNode],
    executed: &[String],
    reused: &[String],
    invalidated: &[String],
) -> BTreeMap<ActionStage, StageCounts> {
    let executed = executed.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let reused = reused.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let invalidated = invalidated.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut counts = all_stage_counts();
    for node in action_graph {
        let stage = counts.get_mut(&node.stage).expect("all action stages have counters");
        stage.requested = stage.requested.saturating_add(1);
        if executed.contains(node.action_ref.as_str()) {
            stage.executed = stage.executed.saturating_add(1);
        }
        if reused.contains(node.action_ref.as_str()) {
            stage.reused = stage.reused.saturating_add(1);
        }
        if invalidated.contains(node.action_ref.as_str()) {
            stage.invalidated = stage.invalidated.saturating_add(1);
        }
    }
    debug_assert_eq!(counts.len(), REQUIRED_STAGE_COUNT);
    debug_assert_eq!(counts.values().map(|value| value.requested).sum::<u32>(), action_graph.len() as u32);
    counts
}

fn validate_run(run: &RunEvidence, expected: RunClass, diagnostics: &mut Vec<String>) {
    if run.run_class != expected {
        diagnostics.push(String::from("run-class-mismatch"));
    }
    if run.environment_class.is_empty() || run.profile_ref.is_empty() || run.cohort_ref.is_empty() {
        diagnostics.push(String::from("run-identity-empty"));
    }
    if run.counts.len() != REQUIRED_STAGE_COUNT {
        diagnostics.push(String::from("run-stage-count-set-incomplete"));
    }
    for counts in run.counts.values() {
        if counts.executed.saturating_add(counts.reused) != counts.requested {
            diagnostics.push(String::from("run-request-execution-reuse-count-mismatch"));
        }
        if counts.invalidated > counts.requested {
            diagnostics.push(String::from("run-invalidated-count-exceeds-requested"));
        }
    }
    if run.elapsed_is_gating {
        diagnostics.push(String::from("elapsed-time-promise-forbidden"));
    }
    validate_run_non_claims(&run.non_claims, diagnostics);
    debug_assert!(run.counts.len() <= REQUIRED_STAGE_COUNT || !diagnostics.is_empty());
    debug_assert!(!run.elapsed_is_gating || !diagnostics.is_empty());
}

fn validate_fresh(run: &RunEvidence, diagnostics: &mut Vec<String>) {
    for counts in run.counts.values() {
        if counts.executed != counts.requested || counts.reused != 0 {
            diagnostics.push(String::from("fresh-run-not-fully-executed"));
        }
    }
}

fn validate_selected_change(run: &RunEvidence, expected: &[String], diagnostics: &mut Vec<String>) {
    let actual = run.invalidated_action_refs.iter().cloned().collect::<BTreeSet<_>>();
    let expected = expected.iter().cloned().collect::<BTreeSet<_>>();
    if actual != expected {
        diagnostics.push(String::from("selected-change-invalidation-mismatch"));
    }
    if actual.is_empty() {
        diagnostics.push(String::from("selected-change-invalidated-nothing"));
    }
}

fn validate_unrelated_change(run: &RunEvidence, diagnostics: &mut Vec<String>) {
    if !run.invalidated_action_refs.is_empty() || run.counts.values().any(|counts| counts.invalidated != 0) {
        diagnostics.push(String::from("unrelated-change-invalidated-selected-graph"));
    }
}

fn validate_full_shared_hit(run: &RunEvidence, diagnostics: &mut Vec<String>) {
    for counts in run.counts.values() {
        if counts.executed != 0 || counts.reused != counts.requested {
            diagnostics.push(String::from("full-shared-hit-executed-action"));
        }
    }
    if run.reused_bytes == 0 {
        diagnostics.push(String::from("full-shared-hit-reused-bytes-empty"));
    }
}

fn validate_run_non_claims(non_claims: &[String], diagnostics: &mut Vec<String>) {
    let declared = non_claims.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for required in EVIDENCE_NON_CLAIMS {
        if !declared.contains(required) {
            diagnostics.push(alloc::format!("evidence-non-claim-missing:{required}"));
        }
    }
}

fn all_stage_counts() -> BTreeMap<ActionStage, StageCounts> {
    [
        ActionStage::Generation,
        ActionStage::Compile,
        ActionStage::Link,
        ActionStage::Smoke,
    ]
    .into_iter()
    .map(|stage| {
        (stage, StageCounts {
            requested: 0,
            executed: 0,
            reused: 0,
            invalidated: 0,
        })
    })
    .collect()
}

pub fn required_evidence_non_claims() -> Vec<String> {
    let mut non_claims = REQUIRED_NON_CLAIMS.iter().map(|value| String::from(*value)).collect::<Vec<_>>();
    non_claims.extend(EVIDENCE_NON_CLAIMS.iter().map(|value| String::from(*value)));
    non_claims.sort();
    non_claims.dedup();
    non_claims
}
