//! Pure operator report for source-built fixed-point and root action trust.
//!
//! The shell validates and reads proof artifacts. This module accepts only
//! typed observations and decides which bounded claim those observations support.

use std::fmt;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const BOOTSTRAP_TRUST_REPORT_SCHEMA: &str = "mantle-bootstrap-trust-report-v1";
pub(crate) const ROOT_ACTION_TRUST_PLAN_SCHEMA: &str = "mantle-root-action-trust-plan-v1";
pub(crate) const ROOT_ACTION_RECONCILIATION_SCHEMA: &str = "mantle-root-action-reconciliation-v1";
pub(crate) const ROOT_ACTION_TRUST_PLAN_FILE: &str = "root-action-trust-plan.json";
pub(crate) const ROOT_ACTION_RECONCILIATION_FILE: &str = "root-action-trust-reconciliation.json";
const REQUIRED_PROOF_WORKFLOW: &str = "mantle-deterministic-proof-receipt-v2";
const REQUIRED_PROVIDER_KIND: &str = "full-source";
const REQUIRED_VERDICT: &str = "self-rebuild-match";
const REQUIRED_STAGE_COUNT: u32 = 6;
const REQUIRED_CURRENT_EXECUTION_COUNT_MIN: u32 = 2;
const BLAKE3_HEX_LENGTH: usize = 64;
const ACTION_COUNT_MAX: u32 = 1_048_576;
const TEXT_BYTES_MAX: usize = 4_096;
const COMPLETE_CLAIM: &str = "source-built Mantle fixed point with complete local root action trust";
const FIXED_POINT_CLAIM: &str = "source-built Mantle fixed point; root action trust is not complete";
const ACTION_TRUST_BLOCKERS: [&str; 3] = [
    "action-trust-plan-not-bound",
    "execution-reconciliation-not-bound",
    "complete-child-action-coverage-not-proven",
];
const NON_CLAIMS: [&str; 7] = [
    "compiler-correctness",
    "seed-correctness",
    "kernel-isolation",
    "independent-rebuild-agreement",
    "bit-for-bit-release-reproducibility",
    "deployment-success",
    "full-cargo-compatibility",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum BootstrapTrustStatus {
    Complete,
    FixedPointOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VerifiedFixedPointTrustFacts {
    pub(crate) workflow_version: String,
    pub(crate) verdict: String,
    pub(crate) receipt_digest_blake3: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) source_authority_digest_blake3: String,
    pub(crate) proof_bundle_digest_blake3: String,
    pub(crate) provider_kind: String,
    pub(crate) hermeticity_mode: String,
    pub(crate) planned_stage_count: u32,
    pub(crate) observed_stage_count: u32,
    pub(crate) executed_stage_count: u32,
    pub(crate) restored_stage_count: u32,
    pub(crate) authority_violation_count: u32,
    pub(crate) fallback_event_count: u32,
    pub(crate) substitution_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RootActionTrustFacts {
    pub(crate) plan_path: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) reconciliation_path: String,
    pub(crate) reconciliation_digest_blake3: String,
    pub(crate) adapter_count: u32,
    pub(crate) planned_action_count: u32,
    pub(crate) matched_action_count: u32,
    pub(crate) observed_event_count: u32,
    pub(crate) matched_event_count: u32,
    pub(crate) unknown_event_count: u32,
    pub(crate) missing_action_count: u32,
    pub(crate) authority_violation_count: u32,
    pub(crate) fallback_event_count: u32,
    pub(crate) remote_event_count: u32,
    pub(crate) cache_only_completion_count: u32,
    pub(crate) local_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BootstrapTrustReport {
    pub(crate) schema: String,
    pub(crate) status: BootstrapTrustStatus,
    pub(crate) fixed_point_verified: bool,
    pub(crate) root_action_trust_complete: bool,
    pub(crate) bounded_claim: String,
    pub(crate) proof: FixedPointTrustSummary,
    pub(crate) stages: StageTrustSummary,
    pub(crate) action_trust: ActionTrustSummary,
    pub(crate) blockers: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FixedPointTrustSummary {
    pub(crate) workflow_version: String,
    pub(crate) verdict: String,
    pub(crate) receipt_digest_blake3: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) source_authority_digest_blake3: String,
    pub(crate) proof_bundle_digest_blake3: String,
    pub(crate) provider_kind: String,
    pub(crate) hermeticity_mode: String,
    pub(crate) substitution_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StageTrustSummary {
    pub(crate) planned: u32,
    pub(crate) observed: u32,
    pub(crate) executed: u32,
    pub(crate) restored_checkpoint: u32,
    pub(crate) authority_violations: u32,
    pub(crate) fallback_events: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ActionTrustSummary {
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) plan_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) plan_digest_blake3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reconciliation_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reconciliation_digest_blake3: Option<String>,
    pub(crate) adapter_count: u32,
    pub(crate) planned_actions: u32,
    pub(crate) matched_actions: u32,
    pub(crate) observed_events: u32,
    pub(crate) matched_events: u32,
    pub(crate) unknown_events: u32,
    pub(crate) missing_actions: u32,
    pub(crate) authority_violations: u32,
    pub(crate) fallback_events: u32,
    pub(crate) remote_events: u32,
    pub(crate) cache_only_completions: u32,
    pub(crate) local_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BootstrapTrustErrorKind {
    InvalidFixedPoint,
    InvalidActionTrust,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapTrustError {
    pub(crate) kind: BootstrapTrustErrorKind,
    pub(crate) message: String,
}

impl fmt::Display for BootstrapTrustError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for BootstrapTrustError {}

pub(crate) fn build_bootstrap_trust_report(
    fixed_point: VerifiedFixedPointTrustFacts,
    action_trust: Option<RootActionTrustFacts>,
) -> Result<BootstrapTrustReport, BootstrapTrustError> {
    validate_fixed_point(&fixed_point)?;
    let (status, bounded_claim, action_summary, blockers) = match action_trust {
        Some(action_facts) => {
            validate_action_trust(&action_facts)?;
            (BootstrapTrustStatus::Complete, COMPLETE_CLAIM, complete_action_summary(action_facts), Vec::new())
        }
        None => (
            BootstrapTrustStatus::FixedPointOnly,
            FIXED_POINT_CLAIM,
            incomplete_action_summary(),
            ACTION_TRUST_BLOCKERS.iter().map(|value| (*value).to_string()).collect(),
        ),
    };
    let root_action_trust_complete = status == BootstrapTrustStatus::Complete;
    let report = BootstrapTrustReport {
        schema: BOOTSTRAP_TRUST_REPORT_SCHEMA.to_string(),
        status,
        fixed_point_verified: true,
        root_action_trust_complete,
        bounded_claim: bounded_claim.to_string(),
        proof: fixed_point_summary(&fixed_point),
        stages: stage_summary(&fixed_point),
        action_trust: action_summary,
        blockers,
        non_claims: NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    assert!(report.fixed_point_verified);
    assert_eq!(report.root_action_trust_complete, report.blockers.is_empty());
    Ok(report)
}

pub(crate) fn render_bootstrap_trust_report(report: &BootstrapTrustReport) -> String {
    let mut output = String::new();
    output.push_str("Bootstrap trust report\n");
    output.push_str(&format!("status: {:?}\n", report.status));
    output.push_str(&format!("claim: {}\n", report.bounded_claim));
    output.push_str(&format!("receipt_blake3: {}\n", report.proof.receipt_digest_blake3));
    output.push_str(&format!(
        "stages: planned={} observed={} executed={} restored-checkpoint={}\n",
        report.stages.planned, report.stages.observed, report.stages.executed, report.stages.restored_checkpoint
    ));
    output.push_str(&format!(
        "actions: status={} planned={} matched={} events={} unknown={} missing={}\n",
        report.action_trust.status,
        report.action_trust.planned_actions,
        report.action_trust.matched_actions,
        report.action_trust.observed_events,
        report.action_trust.unknown_events,
        report.action_trust.missing_actions
    ));
    if !report.blockers.is_empty() {
        output.push_str(&format!("blockers: {}\n", report.blockers.join(", ")));
    }
    assert!(output.starts_with("Bootstrap trust report\n"));
    assert!(output.contains("receipt_blake3:"));
    output
}

fn validate_fixed_point(facts: &VerifiedFixedPointTrustFacts) -> Result<(), BootstrapTrustError> {
    for (label, digest) in [
        ("receipt", facts.receipt_digest_blake3.as_str()),
        ("plan", facts.plan_digest_blake3.as_str()),
        ("source authority", facts.source_authority_digest_blake3.as_str()),
        ("proof bundle", facts.proof_bundle_digest_blake3.as_str()),
    ] {
        validate_digest(BootstrapTrustErrorKind::InvalidFixedPoint, label, digest)?;
    }
    let identity_valid = facts.workflow_version == REQUIRED_PROOF_WORKFLOW
        && facts.verdict == REQUIRED_VERDICT
        && facts.provider_kind == REQUIRED_PROVIDER_KIND
        && facts.hermeticity_mode == "strict";
    let stages_valid = facts.planned_stage_count == REQUIRED_STAGE_COUNT
        && facts.observed_stage_count == REQUIRED_STAGE_COUNT
        && facts.executed_stage_count >= REQUIRED_CURRENT_EXECUTION_COUNT_MIN
        && facts.executed_stage_count.saturating_add(facts.restored_stage_count) == REQUIRED_STAGE_COUNT;
    let violations_absent =
        facts.authority_violation_count == 0 && facts.fallback_event_count == 0 && facts.substitution_count == 0;
    if !identity_valid || !stages_valid || !violations_absent {
        return Err(trust_error(
            BootstrapTrustErrorKind::InvalidFixedPoint,
            "fixed-point facts do not satisfy the strict source-built receipt contract",
        ));
    }
    assert_eq!(facts.planned_stage_count, facts.observed_stage_count);
    assert_eq!(facts.executed_stage_count + facts.restored_stage_count, REQUIRED_STAGE_COUNT);
    Ok(())
}

fn validate_action_trust(facts: &RootActionTrustFacts) -> Result<(), BootstrapTrustError> {
    for (label, digest) in [
        ("action plan", facts.plan_digest_blake3.as_str()),
        ("action reconciliation", facts.reconciliation_digest_blake3.as_str()),
    ] {
        validate_digest(BootstrapTrustErrorKind::InvalidActionTrust, label, digest)?;
    }
    validate_text("action plan path", &facts.plan_path)?;
    validate_text("action reconciliation path", &facts.reconciliation_path)?;
    let counts_valid = facts.adapter_count > 0
        && facts.planned_action_count > 0
        && facts.planned_action_count <= ACTION_COUNT_MAX
        && facts.matched_action_count == facts.planned_action_count
        && facts.observed_event_count > 0
        && facts.observed_event_count <= ACTION_COUNT_MAX
        && facts.matched_event_count == facts.observed_event_count;
    let violations_absent = facts.unknown_event_count == 0
        && facts.missing_action_count == 0
        && facts.authority_violation_count == 0
        && facts.fallback_event_count == 0
        && facts.remote_event_count == 0
        && facts.cache_only_completion_count == 0;
    if !counts_valid || !violations_absent || !facts.local_only {
        return Err(trust_error(
            BootstrapTrustErrorKind::InvalidActionTrust,
            "root action trust is incomplete or contains forbidden execution evidence",
        ));
    }
    assert_eq!(facts.planned_action_count, facts.matched_action_count);
    assert_eq!(facts.observed_event_count, facts.matched_event_count);
    Ok(())
}

fn fixed_point_summary(facts: &VerifiedFixedPointTrustFacts) -> FixedPointTrustSummary {
    FixedPointTrustSummary {
        workflow_version: facts.workflow_version.clone(),
        verdict: facts.verdict.clone(),
        receipt_digest_blake3: facts.receipt_digest_blake3.clone(),
        plan_digest_blake3: facts.plan_digest_blake3.clone(),
        source_authority_digest_blake3: facts.source_authority_digest_blake3.clone(),
        proof_bundle_digest_blake3: facts.proof_bundle_digest_blake3.clone(),
        provider_kind: facts.provider_kind.clone(),
        hermeticity_mode: facts.hermeticity_mode.clone(),
        substitution_count: facts.substitution_count,
    }
}

fn stage_summary(facts: &VerifiedFixedPointTrustFacts) -> StageTrustSummary {
    StageTrustSummary {
        planned: facts.planned_stage_count,
        observed: facts.observed_stage_count,
        executed: facts.executed_stage_count,
        restored_checkpoint: facts.restored_stage_count,
        authority_violations: facts.authority_violation_count,
        fallback_events: facts.fallback_event_count,
    }
}

fn complete_action_summary(facts: RootActionTrustFacts) -> ActionTrustSummary {
    ActionTrustSummary {
        status: "complete".to_string(),
        plan_path: Some(facts.plan_path),
        plan_digest_blake3: Some(facts.plan_digest_blake3),
        reconciliation_path: Some(facts.reconciliation_path),
        reconciliation_digest_blake3: Some(facts.reconciliation_digest_blake3),
        adapter_count: facts.adapter_count,
        planned_actions: facts.planned_action_count,
        matched_actions: facts.matched_action_count,
        observed_events: facts.observed_event_count,
        matched_events: facts.matched_event_count,
        unknown_events: facts.unknown_event_count,
        missing_actions: facts.missing_action_count,
        authority_violations: facts.authority_violation_count,
        fallback_events: facts.fallback_event_count,
        remote_events: facts.remote_event_count,
        cache_only_completions: facts.cache_only_completion_count,
        local_only: facts.local_only,
    }
}

fn incomplete_action_summary() -> ActionTrustSummary {
    ActionTrustSummary {
        status: "not-bound".to_string(),
        plan_path: None,
        plan_digest_blake3: None,
        reconciliation_path: None,
        reconciliation_digest_blake3: None,
        adapter_count: 0,
        planned_actions: 0,
        matched_actions: 0,
        observed_events: 0,
        matched_events: 0,
        unknown_events: 0,
        missing_actions: 0,
        authority_violations: 0,
        fallback_events: 0,
        remote_events: 0,
        cache_only_completions: 0,
        local_only: false,
    }
}

fn validate_digest(kind: BootstrapTrustErrorKind, label: &str, digest: &str) -> Result<(), BootstrapTrustError> {
    let valid = digest.len() == BLAKE3_HEX_LENGTH
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(trust_error(kind, &format!("{label} is not a lowercase BLAKE3 digest")));
    }
    Ok(())
}

fn validate_text(label: &str, value: &str) -> Result<(), BootstrapTrustError> {
    let valid = !value.is_empty()
        && value.len() <= TEXT_BYTES_MAX
        && !value.chars().any(char::is_control)
        && !value.split('/').any(|component| component == "..");
    if !valid {
        return Err(trust_error(
            BootstrapTrustErrorKind::InvalidActionTrust,
            &format!("{label} is empty, oversized, unsafe, or contains control characters"),
        ));
    }
    Ok(())
}

fn trust_error(kind: BootstrapTrustErrorKind, message: &str) -> BootstrapTrustError {
    BootstrapTrustError {
        kind,
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const TEST_ADAPTER_COUNT: u32 = 6;
    const TEST_ACTION_COUNT: u32 = 12;
    const TEST_EVENT_COUNT: u32 = 24;

    #[test]
    fn fixed_point_without_action_links_reports_bounded_gap() {
        let report = build_bootstrap_trust_report(fixed_point_facts(), None).unwrap();

        assert_eq!(report.status, BootstrapTrustStatus::FixedPointOnly);
        assert!(report.fixed_point_verified);
        assert!(!report.root_action_trust_complete);
        assert_eq!(report.blockers.len(), ACTION_TRUST_BLOCKERS.len());
        assert_eq!(report.action_trust.status, "not-bound");
        assert!(render_bootstrap_trust_report(&report).contains("root action trust is not complete"));
    }

    #[test]
    fn complete_action_reconciliation_emits_bounded_complete_claim() {
        let report = build_bootstrap_trust_report(fixed_point_facts(), Some(action_trust_facts())).unwrap();

        assert_eq!(report.status, BootstrapTrustStatus::Complete);
        assert!(report.root_action_trust_complete);
        assert!(report.blockers.is_empty());
        assert_eq!(report.action_trust.planned_actions, TEST_ACTION_COUNT);
        assert_eq!(report.action_trust.matched_events, TEST_EVENT_COUNT);
        assert!(report.action_trust.local_only);
    }

    #[test]
    fn action_reconciliation_rejects_unknown_remote_and_missing_events() {
        let mut facts = action_trust_facts();
        facts.unknown_event_count = 1;
        facts.remote_event_count = 1;
        facts.missing_action_count = 1;

        let error = build_bootstrap_trust_report(fixed_point_facts(), Some(facts)).unwrap_err();

        assert_eq!(error.kind, BootstrapTrustErrorKind::InvalidActionTrust);
        assert!(error.message.contains("incomplete"));
    }

    #[test]
    fn fixed_point_rejects_fallback_or_stage_count_drift() {
        let mut facts = fixed_point_facts();
        facts.fallback_event_count = 1;
        facts.observed_stage_count = REQUIRED_STAGE_COUNT - 1;

        let error = build_bootstrap_trust_report(facts, None).unwrap_err();

        assert_eq!(error.kind, BootstrapTrustErrorKind::InvalidFixedPoint);
        assert!(error.message.contains("strict source-built receipt contract"));
    }

    fn fixed_point_facts() -> VerifiedFixedPointTrustFacts {
        VerifiedFixedPointTrustFacts {
            workflow_version: REQUIRED_PROOF_WORKFLOW.to_string(),
            verdict: REQUIRED_VERDICT.to_string(),
            receipt_digest_blake3: DIGEST_A.to_string(),
            plan_digest_blake3: DIGEST_B.to_string(),
            source_authority_digest_blake3: DIGEST_C.to_string(),
            proof_bundle_digest_blake3: DIGEST_D.to_string(),
            provider_kind: REQUIRED_PROVIDER_KIND.to_string(),
            hermeticity_mode: "strict".to_string(),
            planned_stage_count: REQUIRED_STAGE_COUNT,
            observed_stage_count: REQUIRED_STAGE_COUNT,
            executed_stage_count: REQUIRED_CURRENT_EXECUTION_COUNT_MIN,
            restored_stage_count: REQUIRED_STAGE_COUNT - REQUIRED_CURRENT_EXECUTION_COUNT_MIN,
            authority_violation_count: 0,
            fallback_event_count: 0,
            substitution_count: 0,
        }
    }

    fn action_trust_facts() -> RootActionTrustFacts {
        RootActionTrustFacts {
            plan_path: "action-trust-plan.json".to_string(),
            plan_digest_blake3: DIGEST_A.to_string(),
            reconciliation_path: "action-trust-reconciliation.json".to_string(),
            reconciliation_digest_blake3: DIGEST_B.to_string(),
            adapter_count: TEST_ADAPTER_COUNT,
            planned_action_count: TEST_ACTION_COUNT,
            matched_action_count: TEST_ACTION_COUNT,
            observed_event_count: TEST_EVENT_COUNT,
            matched_event_count: TEST_EVENT_COUNT,
            unknown_event_count: 0,
            missing_action_count: 0,
            authority_violation_count: 0,
            fallback_event_count: 0,
            remote_event_count: 0,
            cache_only_completion_count: 0,
            local_only: true,
        }
    }
}
