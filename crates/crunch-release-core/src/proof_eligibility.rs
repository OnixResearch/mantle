use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const STRICT_PROOF_ELIGIBILITY_POLICY_VERSION: &str = "mantle-strict-proof-eligibility-v1";
pub const STRICT_HERMETICITY_MODE: &str = "strict";
pub const PRACTICAL_HERMETICITY_MODE: &str = "practical";
pub const IMPURE_HERMETICITY_MODE: &str = "impure";

const MODE_BLOCKER_CLASS: &str = "hermeticity-mode";
const AUDIT_BLOCKER_CLASS: &str = "hermeticity-audit";
const CLOSURE_BLOCKER_CLASS: &str = "closure-facts";
const PROTECTED_ENV_BLOCKER_CLASS: &str = "protected-environment";
const HOST_TOOL_BLOCKER_CLASS: &str = "host-tool-inventory";
const MISSING_MODE_LABEL: &str = "missing";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProofFactStatus {
    Satisfied,
    Missing,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StrictProofEligibilityVerdict {
    Admitted,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictProofEligibilityInput {
    pub workflow: String,
    pub requested_claim: String,
    pub hermeticity_mode: String,
    pub hermeticity_audit_events: Vec<String>,
    pub closure_status: ProofFactStatus,
    pub protected_environment_status: ProofFactStatus,
    pub host_tool_status: ProofFactStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictProofEligibilityReport {
    pub policy_version: String,
    pub verdict: StrictProofEligibilityVerdict,
    pub admitted: bool,
    pub strict_claim_satisfied: bool,
    pub workflow: String,
    pub requested_claim: String,
    pub hermeticity_mode: String,
    pub audit_event_set_digest_blake3: String,
    pub audit_policy_basis: String,
    pub closure_status: ProofFactStatus,
    pub protected_environment_status: ProofFactStatus,
    pub host_tool_status: ProofFactStatus,
    pub blocked_classes: Vec<String>,
    pub blocked_audit_events: Vec<String>,
    pub narrower_claims: Vec<String>,
    pub diagnostics: Vec<String>,
}

pub fn strict_proof_eligibility_gate(input: StrictProofEligibilityInput) -> StrictProofEligibilityReport {
    assert!(!input.workflow.trim().is_empty(), "strict proof workflow must not be empty");
    assert!(!input.requested_claim.trim().is_empty(), "strict proof requested claim must not be empty");

    let mode = normalized_mode(&input.hermeticity_mode);
    let audit_report = crate::proof_audit::strict_proof_audit_gate(
        input.workflow.clone(),
        input.requested_claim.clone(),
        input.hermeticity_audit_events,
    );
    let mut blocked_classes = Vec::new();
    let mut diagnostics = Vec::new();

    if mode != STRICT_HERMETICITY_MODE {
        blocked_classes.push(MODE_BLOCKER_CLASS.to_string());
        diagnostics.push(format!(
            "proof workflow {} claim {} selected hermeticity mode {}; strict is required",
            input.workflow, input.requested_claim, mode
        ));
    }
    push_status_blocker(
        input.closure_status,
        CLOSURE_BLOCKER_CLASS,
        "closure facts",
        &input.workflow,
        &mut blocked_classes,
        &mut diagnostics,
    );
    push_status_blocker(
        input.protected_environment_status,
        PROTECTED_ENV_BLOCKER_CLASS,
        "protected environment",
        &input.workflow,
        &mut blocked_classes,
        &mut diagnostics,
    );
    push_status_blocker(
        input.host_tool_status,
        HOST_TOOL_BLOCKER_CLASS,
        "host tool inventory",
        &input.workflow,
        &mut blocked_classes,
        &mut diagnostics,
    );
    if !audit_report.strict_claim_satisfied {
        blocked_classes.push(AUDIT_BLOCKER_CLASS.to_string());
        diagnostics.extend(crate::proof_audit::proof_audit_gate_blocking_reasons(audit_report.clone()));
    }

    blocked_classes = sorted_unique_strings(blocked_classes);
    diagnostics = sorted_unique_strings(diagnostics);
    let verdict = if blocked_classes.is_empty() {
        StrictProofEligibilityVerdict::Admitted
    } else {
        StrictProofEligibilityVerdict::Blocked
    };
    StrictProofEligibilityReport {
        policy_version: STRICT_PROOF_ELIGIBILITY_POLICY_VERSION.to_string(),
        verdict,
        admitted: matches!(verdict, StrictProofEligibilityVerdict::Admitted),
        strict_claim_satisfied: matches!(verdict, StrictProofEligibilityVerdict::Admitted),
        workflow: input.workflow,
        requested_claim: input.requested_claim,
        hermeticity_mode: mode,
        audit_event_set_digest_blake3: audit_report.event_set_digest_blake3,
        audit_policy_basis: audit_report.policy_basis,
        closure_status: input.closure_status,
        protected_environment_status: input.protected_environment_status,
        host_tool_status: input.host_tool_status,
        blocked_classes,
        blocked_audit_events: audit_report.blocked_events,
        narrower_claims: audit_report.narrower_claims,
        diagnostics,
    }
}

pub fn proof_eligibility_blocking_reasons(report: StrictProofEligibilityReport) -> Vec<String> {
    let mut reasons = Vec::new();
    for class in report.blocked_classes {
        reasons.push(format!(
            "strict proof eligibility class {class} blocked workflow {} claim {} by {}",
            report.workflow, report.requested_claim, report.policy_version
        ));
    }
    reasons.extend(report.diagnostics);
    sorted_unique_strings(reasons)
}

fn push_status_blocker(
    status: ProofFactStatus,
    class: &str,
    label: &str,
    workflow: &str,
    blocked_classes: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
) {
    if status == ProofFactStatus::Satisfied {
        return;
    }
    blocked_classes.push(class.to_string());
    diagnostics.push(format!(
        "strict proof workflow {workflow} has {} status {}; satisfied is required",
        label,
        status.as_str()
    ));
}

impl ProofFactStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ProofFactStatus::Satisfied => "satisfied",
            ProofFactStatus::Missing => "missing",
            ProofFactStatus::Degraded => "degraded",
        }
    }
}

fn normalized_mode(mode: &str) -> String {
    let mode = mode.trim();
    if mode.is_empty() {
        return MISSING_MODE_LABEL.to_string();
    }
    mode.to_string()
}

fn sorted_unique_strings(values: Vec<String>) -> Vec<String> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn clean_input() -> StrictProofEligibilityInput {
        StrictProofEligibilityInput {
            workflow: "self-hosting".to_string(),
            requested_claim: "self-hosting".to_string(),
            hermeticity_mode: STRICT_HERMETICITY_MODE.to_string(),
            hermeticity_audit_events: Vec::new(),
            closure_status: ProofFactStatus::Satisfied,
            protected_environment_status: ProofFactStatus::Satisfied,
            host_tool_status: ProofFactStatus::Satisfied,
        }
    }

    #[test]
    fn clean_strict_evidence_is_admitted() {
        let report = strict_proof_eligibility_gate(clean_input());

        assert_eq!(report.verdict, StrictProofEligibilityVerdict::Admitted);
        assert!(report.admitted);
        assert!(report.strict_claim_satisfied);
        assert!(report.blocked_classes.is_empty());
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn practical_impure_and_missing_modes_are_blocked() {
        for (mode, expected_label) in [
            (PRACTICAL_HERMETICITY_MODE, PRACTICAL_HERMETICITY_MODE),
            (IMPURE_HERMETICITY_MODE, IMPURE_HERMETICITY_MODE),
            ("", MISSING_MODE_LABEL),
        ] {
            let mut input = clean_input();
            input.hermeticity_mode = mode.to_string();

            let report = strict_proof_eligibility_gate(input);

            assert_eq!(report.verdict, StrictProofEligibilityVerdict::Blocked, "{mode}");
            assert!(!report.strict_claim_satisfied, "{mode}");
            assert_eq!(report.hermeticity_mode, expected_label, "{mode}");
            assert!(report.blocked_classes.contains(&MODE_BLOCKER_CLASS.to_string()), "{mode}");
        }
    }

    #[test]
    fn missing_or_degraded_fact_classes_are_reported() {
        let mut input = clean_input();
        input.closure_status = ProofFactStatus::Missing;
        input.protected_environment_status = ProofFactStatus::Degraded;
        input.host_tool_status = ProofFactStatus::Degraded;

        let report = strict_proof_eligibility_gate(input);

        assert_eq!(report.verdict, StrictProofEligibilityVerdict::Blocked);
        assert!(report.blocked_classes.contains(&CLOSURE_BLOCKER_CLASS.to_string()));
        assert!(report.blocked_classes.contains(&PROTECTED_ENV_BLOCKER_CLASS.to_string()));
        assert!(report.blocked_classes.contains(&HOST_TOOL_BLOCKER_CLASS.to_string()));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("closure facts")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("protected environment")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("host tool inventory")));
    }

    #[test]
    fn unapproved_audit_events_fail_closed() {
        let mut input = clean_input();
        input.hermeticity_audit_events = vec!["host-tool-fallback".to_string()];

        let report = strict_proof_eligibility_gate(input);
        let reasons = proof_eligibility_blocking_reasons(report.clone());

        assert_eq!(report.verdict, StrictProofEligibilityVerdict::Blocked);
        assert!(report.blocked_classes.contains(&AUDIT_BLOCKER_CLASS.to_string()));
        assert!(report.blocked_audit_events.contains(&"host-tool-fallback".to_string()));
        assert!(reasons.iter().any(|reason| reason.contains("host-tool-fallback")));
    }
}
