use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::error::ReleaseEvidenceError;

pub const PROOF_AUDIT_POLICY_VERSION: &str = "mantle-proof-audit-policy-v1";
pub const STRICT_PROOF_POLICY_BASIS: &str = "mantle-proof-audit-policy-v1:strict-default";
pub const DEFAULT_DIAGNOSTIC_NARROWER_CLAIM: &str = "diagnostic-evidence-only";

const MAX_PROOF_AUDIT_EVENT_COUNT: usize = 256;
const MAX_PROOF_AUDIT_EVENT_BYTES: usize = 128;
const INFORMATIONAL_STORE_READ_EVENT: &str = "store-read";
const INFORMATIONAL_OUTPUT_WRITE_EVENT: &str = "output-write";
const EVENT_LIMIT_BLOCKER: &str = "proof-audit-event-limit-exceeded";
const EMPTY_EVENT_BLOCKER: &str = "proof-audit-empty-event-class";
const EVENT_SERIALIZATION_BLOCKER: &str = "proof-audit-event-serialization-failed";
const EVENT_SERIALIZATION_FAILURE_DOMAIN: &[u8] = b"mantle.proof-audit.event-serialization-failed.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProofAuditGateVerdict {
    Admitted,
    Downgraded,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofAuditDowngradePolicy {
    pub event_class: String,
    pub workflow: String,
    pub affected_claim: String,
    pub narrower_claim: String,
    pub policy_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofAuditPolicy {
    pub policy_version: String,
    pub policy_basis: String,
    pub informational_events: Vec<String>,
    pub allowed_downgrades: Vec<ProofAuditDowngradePolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofAuditGateInput {
    pub workflow: String,
    pub requested_claim: String,
    pub event_classes: Vec<String>,
    pub policy: ProofAuditPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofAuditDowngradeReport {
    pub event_class: String,
    pub workflow: String,
    pub affected_claim: String,
    pub narrower_claim: String,
    pub policy_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofAuditGateReport {
    pub verdict: ProofAuditGateVerdict,
    pub admitted: bool,
    pub strict_claim_satisfied: bool,
    pub workflow: String,
    pub requested_claim: String,
    pub event_set_digest_blake3: String,
    pub policy_basis: String,
    pub blocked_events: Vec<String>,
    pub downgrades: Vec<ProofAuditDowngradeReport>,
    pub narrower_claims: Vec<String>,
    pub diagnostics: Vec<String>,
}

struct AuditClassificationContext<'a> {
    workflow: &'a str,
    requested_claim: &'a str,
    policy_basis: &'a str,
    informational_events: &'a [String],
    downgrades: &'a [ProofAuditDowngradePolicy],
}

struct AuditClassificationResult {
    blocked_events: Vec<String>,
    downgrade_rows: Vec<ProofAuditDowngradeReport>,
    diagnostics: Vec<String>,
}

struct DowngradeMatch<'a> {
    event_class: &'a str,
    workflow: &'a str,
    requested_claim: &'a str,
}

pub fn strict_proof_audit_policy() -> ProofAuditPolicy {
    ProofAuditPolicy {
        policy_version: PROOF_AUDIT_POLICY_VERSION.to_string(),
        policy_basis: STRICT_PROOF_POLICY_BASIS.to_string(),
        informational_events: vec![
            INFORMATIONAL_STORE_READ_EVENT.to_string(),
            INFORMATIONAL_OUTPUT_WRITE_EVENT.to_string(),
        ],
        allowed_downgrades: Vec::new(),
    }
}

pub fn strict_proof_audit_gate(
    workflow: String,
    requested_claim: String,
    event_classes: Vec<String>,
) -> ProofAuditGateReport {
    evaluate_proof_audit_gate(ProofAuditGateInput {
        workflow,
        requested_claim,
        event_classes,
        policy: strict_proof_audit_policy(),
    })
}

pub fn evaluate_proof_audit_gate(input: ProofAuditGateInput) -> ProofAuditGateReport {
    assert!(!input.workflow.trim().is_empty(), "proof audit workflow must not be empty");
    assert!(!input.requested_claim.trim().is_empty(), "proof audit requested claim must not be empty");

    let events = normalized_event_classes(input.event_classes);
    let informational_events = normalized_event_classes(input.policy.informational_events);
    let downgrades = normalized_downgrade_policies(input.policy.allowed_downgrades);
    let context = AuditClassificationContext {
        workflow: &input.workflow,
        requested_claim: &input.requested_claim,
        policy_basis: &input.policy.policy_basis,
        informational_events: &informational_events,
        downgrades: &downgrades,
    };
    let mut classification = classify_audit_events(&events, &context);
    if input.policy.policy_version != PROOF_AUDIT_POLICY_VERSION {
        classification
            .diagnostics
            .push(format!("unsupported proof audit policy version {}", input.policy.policy_version));
    }
    let event_set_digest_blake3 = match event_set_digest(&events) {
        Ok(digest_blake3) => digest_blake3,
        Err(error) => {
            classification.blocked_events.push(EVENT_SERIALIZATION_BLOCKER.to_string());
            classification.diagnostics.push(error.to_string());
            blake3::hash(EVENT_SERIALIZATION_FAILURE_DOMAIN).to_hex().to_string()
        }
    };
    classification.blocked_events = sorted_unique_strings(classification.blocked_events);
    classification.downgrade_rows.sort_by(|left, right| {
        left.event_class
            .cmp(&right.event_class)
            .then(left.workflow.cmp(&right.workflow))
            .then(left.affected_claim.cmp(&right.affected_claim))
            .then(left.narrower_claim.cmp(&right.narrower_claim))
            .then(left.policy_basis.cmp(&right.policy_basis))
    });
    let narrower_claims = sorted_unique_strings(
        classification.downgrade_rows.iter().map(|downgrade| downgrade.narrower_claim.clone()).collect(),
    );
    classification.diagnostics = sorted_unique_strings(classification.diagnostics);

    let verdict = classify_report_verdict(&classification.blocked_events, &classification.downgrade_rows);
    debug_assert_eq!(matches!(verdict, ProofAuditGateVerdict::Blocked), !classification.blocked_events.is_empty());
    debug_assert!(event_set_digest_blake3.len() == crate::manifest::BLAKE3_HEX_LENGTH_CHARS);
    ProofAuditGateReport {
        verdict,
        admitted: !matches!(verdict, ProofAuditGateVerdict::Blocked),
        strict_claim_satisfied: matches!(verdict, ProofAuditGateVerdict::Admitted),
        workflow: input.workflow,
        requested_claim: input.requested_claim,
        event_set_digest_blake3,
        policy_basis: input.policy.policy_basis,
        blocked_events: classification.blocked_events,
        downgrades: classification.downgrade_rows,
        narrower_claims,
        diagnostics: classification.diagnostics,
    }
}

fn classify_audit_events(events: &[String], context: &AuditClassificationContext<'_>) -> AuditClassificationResult {
    let bounded_event_slots = events.len().min(MAX_PROOF_AUDIT_EVENT_COUNT);
    let mut blocked_events = Vec::with_capacity(bounded_event_slots);
    let mut downgrade_rows = Vec::with_capacity(bounded_event_slots);
    let mut diagnostics = Vec::with_capacity(events.len().saturating_add(1));
    if events.len() > MAX_PROOF_AUDIT_EVENT_COUNT {
        blocked_events.push(EVENT_LIMIT_BLOCKER.to_string());
        diagnostics.push(format!(
            "proof audit event set contains {} events; limit is {}",
            events.len(),
            MAX_PROOF_AUDIT_EVENT_COUNT
        ));
    }
    for event in events {
        if event.trim().is_empty() {
            blocked_events.push(EMPTY_EVENT_BLOCKER.to_string());
            diagnostics.push("proof audit event class must not be empty".to_string());
            continue;
        }
        if event.len() > MAX_PROOF_AUDIT_EVENT_BYTES {
            blocked_events.push(event.clone());
            diagnostics.push(format!("proof audit event class {event} exceeds {} bytes", MAX_PROOF_AUDIT_EVENT_BYTES));
            continue;
        }
        if context.informational_events.iter().any(|allowed| allowed == event) {
            continue;
        }
        let match_key = DowngradeMatch {
            event_class: event,
            workflow: context.workflow,
            requested_claim: context.requested_claim,
        };
        if let Some(downgrade) = matching_downgrade(match_key, context.downgrades) {
            downgrade_rows.push(ProofAuditDowngradeReport {
                event_class: event.clone(),
                workflow: downgrade.workflow.clone(),
                affected_claim: downgrade.affected_claim.clone(),
                narrower_claim: downgrade.narrower_claim.clone(),
                policy_basis: downgrade.policy_basis.clone(),
            });
            continue;
        }
        blocked_events.push(event.clone());
        diagnostics.push(format!(
            "proof audit event {event} is not admitted by {} for workflow {} claim {}",
            context.policy_basis, context.workflow, context.requested_claim
        ));
    }
    debug_assert!(blocked_events.len() <= events.len().saturating_add(1));
    debug_assert!(downgrade_rows.len() <= events.len());
    AuditClassificationResult {
        blocked_events,
        downgrade_rows,
        diagnostics,
    }
}

pub fn proof_audit_gate_blocking_reasons(report: ProofAuditGateReport) -> Vec<String> {
    let blocked_event_count = report.blocked_events.len();
    let downgrade_count = report.downgrades.len();
    let reason_slots = blocked_event_count.saturating_add(downgrade_count);
    let mut reasons = Vec::with_capacity(reason_slots);
    for blocked in report.blocked_events {
        reasons.push(format!(
            "proof audit event {blocked} blocked by {} for workflow {} claim {}",
            report.policy_basis, report.workflow, report.requested_claim
        ));
    }
    for downgrade in report.downgrades {
        reasons.push(format!(
            "proof audit event {} downgraded {} to {} by {} for workflow {}",
            downgrade.event_class,
            downgrade.affected_claim,
            downgrade.narrower_claim,
            downgrade.policy_basis,
            downgrade.workflow
        ));
    }
    debug_assert!(reasons.len() <= reason_slots);
    debug_assert!(reason_slots >= blocked_event_count);
    sorted_unique_strings(reasons)
}

fn normalized_event_classes(event_classes: Vec<String>) -> Vec<String> {
    sorted_unique_strings(event_classes.into_iter().map(|event| event.trim().to_string()).collect())
}

fn normalized_downgrade_policies(policies: Vec<ProofAuditDowngradePolicy>) -> Vec<ProofAuditDowngradePolicy> {
    let policy_count = policies.len();
    let mut normalized = policies
        .into_iter()
        .map(|policy| ProofAuditDowngradePolicy {
            event_class: policy.event_class.trim().to_string(),
            workflow: policy.workflow.trim().to_string(),
            affected_claim: policy.affected_claim.trim().to_string(),
            narrower_claim: policy.narrower_claim.trim().to_string(),
            policy_basis: policy.policy_basis.trim().to_string(),
        })
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| {
        left.event_class
            .cmp(&right.event_class)
            .then(left.workflow.cmp(&right.workflow))
            .then(left.affected_claim.cmp(&right.affected_claim))
            .then(left.narrower_claim.cmp(&right.narrower_claim))
            .then(left.policy_basis.cmp(&right.policy_basis))
    });
    normalized.dedup();
    debug_assert!(normalized.windows(2).all(|pair| pair[0] != pair[1]));
    debug_assert!(normalized.len() <= policy_count);
    normalized
}

fn matching_downgrade<'a>(
    match_key: DowngradeMatch<'_>,
    downgrades: &'a [ProofAuditDowngradePolicy],
) -> Option<&'a ProofAuditDowngradePolicy> {
    downgrades.iter().find(|downgrade| {
        downgrade.event_class == match_key.event_class
            && downgrade.workflow == match_key.workflow
            && downgrade.affected_claim == match_key.requested_claim
            && !downgrade.narrower_claim.is_empty()
            && !downgrade.policy_basis.is_empty()
    })
}

fn classify_report_verdict(
    blocked_events: &[String],
    downgrades: &[ProofAuditDowngradeReport],
) -> ProofAuditGateVerdict {
    if !blocked_events.is_empty() {
        return ProofAuditGateVerdict::Blocked;
    }
    if !downgrades.is_empty() {
        return ProofAuditGateVerdict::Downgraded;
    }
    ProofAuditGateVerdict::Admitted
}

fn event_set_digest(events: &[String]) -> Result<String, ReleaseEvidenceError> {
    let bytes = serde_json::to_vec(events)
        .map_err(|error| ReleaseEvidenceError::Parse(format!("serializing proof audit event classes: {error}")))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn sorted_unique_strings(values: Vec<String>) -> Vec<String> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORKFLOW_DETERMINISTIC_RELEASE: &str = "deterministic-release";
    const CLAIM_DETERMINISTIC_RELEASE: &str = "deterministic-release";
    const EVENT_HOST_TOOL_FALLBACK: &str = "host-tool-fallback";
    const POLICY_EXCEPTION_BASIS: &str = "mantle-proof-audit-policy-v1:test-exception";

    fn input_with_events(event_classes: Vec<String>) -> ProofAuditGateInput {
        ProofAuditGateInput {
            workflow: WORKFLOW_DETERMINISTIC_RELEASE.to_string(),
            requested_claim: CLAIM_DETERMINISTIC_RELEASE.to_string(),
            event_classes,
            policy: strict_proof_audit_policy(),
        }
    }

    #[test]
    fn clean_or_informational_audit_sets_are_admitted_with_stable_digest() {
        let clean = evaluate_proof_audit_gate(input_with_events(Vec::new()));
        let informational = evaluate_proof_audit_gate(input_with_events(vec![
            INFORMATIONAL_OUTPUT_WRITE_EVENT.to_string(),
            INFORMATIONAL_STORE_READ_EVENT.to_string(),
        ]));
        let informational_reordered = evaluate_proof_audit_gate(input_with_events(vec![
            INFORMATIONAL_STORE_READ_EVENT.to_string(),
            INFORMATIONAL_OUTPUT_WRITE_EVENT.to_string(),
        ]));

        assert_eq!(clean.verdict, ProofAuditGateVerdict::Admitted);
        assert!(clean.admitted);
        assert!(clean.strict_claim_satisfied);
        assert!(clean.blocked_events.is_empty());
        assert_eq!(informational.verdict, ProofAuditGateVerdict::Admitted);
        assert!(informational.strict_claim_satisfied);
        assert_eq!(informational.event_set_digest_blake3, informational_reordered.event_set_digest_blake3);
        assert_ne!(clean.event_set_digest_blake3, informational.event_set_digest_blake3);
    }

    #[test]
    fn unknown_or_unapproved_audit_events_fail_closed() {
        let report = evaluate_proof_audit_gate(input_with_events(vec![
            "future-benign-event".to_string(),
            EVENT_HOST_TOOL_FALLBACK.to_string(),
        ]));
        let reasons = proof_audit_gate_blocking_reasons(report.clone());

        assert_eq!(report.verdict, ProofAuditGateVerdict::Blocked);
        assert!(!report.admitted);
        assert!(!report.strict_claim_satisfied);
        assert!(report.blocked_events.contains(&"future-benign-event".to_string()));
        assert!(report.blocked_events.contains(&EVENT_HOST_TOOL_FALLBACK.to_string()));
        assert!(reasons.iter().any(|reason| reason.contains("future-benign-event")));
        assert!(reasons.iter().any(|reason| reason.contains(STRICT_PROOF_POLICY_BASIS)));
    }

    #[test]
    fn approved_downgrade_records_policy_basis_and_narrower_claim() {
        let mut policy = strict_proof_audit_policy();
        policy.allowed_downgrades.push(ProofAuditDowngradePolicy {
            event_class: EVENT_HOST_TOOL_FALLBACK.to_string(),
            workflow: WORKFLOW_DETERMINISTIC_RELEASE.to_string(),
            affected_claim: CLAIM_DETERMINISTIC_RELEASE.to_string(),
            narrower_claim: DEFAULT_DIAGNOSTIC_NARROWER_CLAIM.to_string(),
            policy_basis: POLICY_EXCEPTION_BASIS.to_string(),
        });
        let report = evaluate_proof_audit_gate(ProofAuditGateInput {
            workflow: WORKFLOW_DETERMINISTIC_RELEASE.to_string(),
            requested_claim: CLAIM_DETERMINISTIC_RELEASE.to_string(),
            event_classes: vec![EVENT_HOST_TOOL_FALLBACK.to_string()],
            policy,
        });
        let reasons = proof_audit_gate_blocking_reasons(report.clone());

        assert_eq!(report.verdict, ProofAuditGateVerdict::Downgraded);
        assert!(report.admitted);
        assert!(!report.strict_claim_satisfied);
        assert!(report.blocked_events.is_empty());
        assert_eq!(report.downgrades.len(), 1);
        assert_eq!(report.downgrades[0].event_class, EVENT_HOST_TOOL_FALLBACK);
        assert_eq!(report.downgrades[0].policy_basis, POLICY_EXCEPTION_BASIS);
        assert_eq!(report.narrower_claims, vec![DEFAULT_DIAGNOSTIC_NARROWER_CLAIM.to_string()]);
        assert!(reasons.iter().any(|reason| reason.contains("downgraded deterministic-release")));
    }
}
