use serde::Serialize;
use serde_json::Value;

const DETERMINISTIC_PROOF_SCHEMA: &str = "mantle-deterministic-proof-receipt-v2";
const TRUST_REPORT_SCHEMA: &str = "mantle-bootstrap-trust-report-v1";
const ACTION_PLAN_SCHEMA: &str = "mantle-root-action-trust-plan-v1";
const ACTION_RECONCILIATION_SCHEMA: &str = "mantle-root-action-reconciliation-v1";
const REQUIRED_PROVIDER_KIND: &str = "full-source";
const REQUIRED_VERDICT: &str = "self-rebuild-match";
const REQUIRED_HERMETICITY: &str = "strict";
const REQUIRED_TRUST_STATUS: &str = "complete";
const REQUIRED_STAGE_COUNT: u64 = 6;
const REQUIRED_EXECUTED_STAGE_COUNT: u64 = 2;
const REQUIRED_RESTORED_STAGE_COUNT: u64 = 4;
const REQUIRED_ADAPTER_COUNT: u64 = 5;
const REQUIRED_RUN_COUNT: usize = 2;
const COUNT_MIN: u64 = 1;
const BLAKE3_HEX_LENGTH: usize = 64;

#[derive(Clone, Copy)]
pub(crate) struct PromotionEvidence<'a> {
    pub(crate) deterministic_proof: &'a Value,
    pub(crate) deterministic_file_blake3: &'a str,
    pub(crate) action_plan: &'a Value,
    pub(crate) action_plan_file_blake3: &'a str,
    pub(crate) action_reconciliation: &'a Value,
    pub(crate) action_reconciliation_file_blake3: &'a str,
    pub(crate) trust_report: &'a Value,
    pub(crate) trust_report_file_blake3: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(crate) struct PromotionSummary {
    pub(crate) selected_provider_kind: String,
    pub(crate) receipt_digest_blake3: String,
    pub(crate) source_blake3: String,
    pub(crate) vendor_blake3: String,
    pub(crate) rebuild_descriptor_blake3: String,
    pub(crate) rebuild_authority_plan_blake3: String,
    pub(crate) proof_bundle_digest_blake3: String,
    pub(crate) action_plan_file_blake3: String,
    pub(crate) action_reconciliation_file_blake3: String,
    pub(crate) trust_report_file_blake3: String,
    pub(crate) output_digest_blake3: String,
    pub(crate) planned_actions: u64,
    pub(crate) matched_actions: u64,
    pub(crate) observed_events: u64,
    pub(crate) matched_events: u64,
    pub(crate) local_only: bool,
}

#[derive(Debug)]
struct DeterministicFacts {
    receipt_digest_blake3: String,
    source_blake3: String,
    vendor_blake3: String,
    rebuild_descriptor_blake3: String,
    rebuild_authority_plan_blake3: String,
    proof_bundle_digest_blake3: String,
    action_plan_file_blake3: String,
    action_reconciliation_file_blake3: String,
    output_digest_blake3: String,
}

#[derive(Debug)]
struct ActionFacts {
    proof_plan_digest_blake3: String,
    planned_actions: u64,
    matched_actions: u64,
    observed_events: u64,
    matched_events: u64,
}

pub(crate) fn validate_promotion(evidence: PromotionEvidence<'_>) -> Result<PromotionSummary, String> {
    validate_file_digests(&evidence)?;
    let deterministic = validate_deterministic_proof(evidence.deterministic_proof)?;
    require_equal(
        "deterministic action plan file digest",
        &deterministic.action_plan_file_blake3,
        evidence.action_plan_file_blake3,
    )?;
    require_equal(
        "deterministic action reconciliation file digest",
        &deterministic.action_reconciliation_file_blake3,
        evidence.action_reconciliation_file_blake3,
    )?;
    let action = validate_action_evidence(
        evidence.action_plan,
        evidence.action_plan_file_blake3,
        evidence.action_reconciliation,
    )?;
    validate_trust_report(
        evidence.trust_report,
        &deterministic,
        &action,
        evidence.action_plan_file_blake3,
        evidence.action_reconciliation_file_blake3,
    )?;
    Ok(PromotionSummary {
        selected_provider_kind: REQUIRED_PROVIDER_KIND.to_string(),
        receipt_digest_blake3: deterministic.receipt_digest_blake3,
        source_blake3: deterministic.source_blake3,
        vendor_blake3: deterministic.vendor_blake3,
        rebuild_descriptor_blake3: deterministic.rebuild_descriptor_blake3,
        rebuild_authority_plan_blake3: deterministic.rebuild_authority_plan_blake3,
        proof_bundle_digest_blake3: deterministic.proof_bundle_digest_blake3,
        action_plan_file_blake3: evidence.action_plan_file_blake3.to_string(),
        action_reconciliation_file_blake3: evidence.action_reconciliation_file_blake3.to_string(),
        trust_report_file_blake3: evidence.trust_report_file_blake3.to_string(),
        output_digest_blake3: deterministic.output_digest_blake3,
        planned_actions: action.planned_actions,
        matched_actions: action.matched_actions,
        observed_events: action.observed_events,
        matched_events: action.matched_events,
        local_only: true,
    })
}

fn validate_file_digests(evidence: &PromotionEvidence<'_>) -> Result<(), String> {
    for (field, digest) in [
        ("deterministic_file_blake3", evidence.deterministic_file_blake3),
        ("action_plan_file_blake3", evidence.action_plan_file_blake3),
        ("action_reconciliation_file_blake3", evidence.action_reconciliation_file_blake3),
        ("trust_report_file_blake3", evidence.trust_report_file_blake3),
    ] {
        require_blake3(field, digest)?;
    }
    Ok(())
}

fn validate_deterministic_proof(value: &Value) -> Result<DeterministicFacts, String> {
    require_string(value, "schema", DETERMINISTIC_PROOF_SCHEMA)?;
    require_string(value, "workflow_version", DETERMINISTIC_PROOF_SCHEMA)?;
    require_string(value, "verdict", REQUIRED_VERDICT)?;
    require_string(value, "selected_provider_kind", REQUIRED_PROVIDER_KIND)?;
    require_string(value, "hermeticity_mode", REQUIRED_HERMETICITY)?;
    require_empty_array(value, "blocking_reasons")?;
    let receipt_digest = require_digest(value, "receipt_blake3")?;
    let source_digest = require_digest(value, "source_blake3")?;
    let vendor_digest = require_digest(value, "vendor_blake3")?;
    let descriptor_digest = require_digest(value, "rebuild_descriptor_blake3")?;
    let authority_digest = require_digest(value, "rebuild_authority_plan_blake3")?;
    let fixed_point = require_object(value, "source_built_fixed_point")?;
    let action_plan_digest = require_digest(fixed_point, "action_trust_plan_digest_blake3")?;
    let reconciliation_digest = require_digest(fixed_point, "action_trust_reconciliation_digest_blake3")?;
    let proof_bundle_digest = require_digest(fixed_point, "final_proof_bundle_digest_blake3")?;
    let output_digest = validate_deterministic_runs(value)?;
    Ok(DeterministicFacts {
        receipt_digest_blake3: receipt_digest.to_string(),
        source_blake3: source_digest.to_string(),
        vendor_blake3: vendor_digest.to_string(),
        rebuild_descriptor_blake3: descriptor_digest.to_string(),
        rebuild_authority_plan_blake3: authority_digest.to_string(),
        proof_bundle_digest_blake3: proof_bundle_digest.to_string(),
        action_plan_file_blake3: action_plan_digest.to_string(),
        action_reconciliation_file_blake3: reconciliation_digest.to_string(),
        output_digest_blake3: output_digest,
    })
}

fn validate_deterministic_runs(value: &Value) -> Result<String, String> {
    let runs = require_array(value, "runs")?;
    if runs.len() != REQUIRED_RUN_COUNT {
        return Err(format!("deterministic proof run count is {}, expected {REQUIRED_RUN_COUNT}", runs.len()));
    }
    let mut output_digest: Option<String> = None;
    for (index, run) in runs.iter().enumerate() {
        require_empty_array(run, "authority_violations")?;
        require_empty_array(run, "hermeticity_audit_events")?;
        require_empty_array(run, "substituted_dependency_identities")?;
        let outputs = require_array(run, "output_digests")?;
        let first = outputs.first().ok_or_else(|| format!("run {index} has no output digest"))?;
        let observed = require_digest(first, "digest_blake3")?;
        match &output_digest {
            Some(expected) => require_equal("stage output digest", observed, expected)?,
            None => output_digest = Some(observed.to_string()),
        }
    }
    output_digest.ok_or_else(|| "deterministic proof has no output digest".to_string())
}

fn validate_action_evidence(
    plan: &Value,
    plan_file_blake3: &str,
    reconciliation: &Value,
) -> Result<ActionFacts, String> {
    require_string(plan, "schema", ACTION_PLAN_SCHEMA)?;
    let proof_plan_digest = require_digest(plan, "proof_plan_digest_blake3")?;
    let adapter_count = require_u64(plan, "adapter_count")?;
    let planned_actions = require_positive_u64(plan, "action_count")?;
    require_bool(plan, "local_only", true)?;
    require_bool(plan, "cache_only_completion_allowed", false)?;
    require_empty_array(plan, "blockers")?;
    let adapters = require_array(plan, "adapters")?;
    if adapter_count != REQUIRED_ADAPTER_COUNT || adapters.len() as u64 != REQUIRED_ADAPTER_COUNT {
        return Err(format!(
            "root action adapter count is {adapter_count}/{}, expected {REQUIRED_ADAPTER_COUNT}",
            adapters.len()
        ));
    }
    let (adapter_actions, adapter_events) = sum_adapters(adapters)?;
    if adapter_actions != planned_actions {
        return Err(format!("adapter action sum {adapter_actions} does not match {planned_actions}"));
    }
    validate_reconciliation(reconciliation, plan_file_blake3, planned_actions, adapter_events).map(|facts| {
        ActionFacts {
            proof_plan_digest_blake3: proof_plan_digest.to_string(),
            ..facts
        }
    })
}

fn sum_adapters(adapters: &[Value]) -> Result<(u64, u64), String> {
    let mut action_sum = 0u64;
    let mut event_sum = 0u64;
    for adapter in adapters {
        let planned = require_positive_u64(adapter, "planned_action_count")?;
        let matched = require_positive_u64(adapter, "matched_action_count")?;
        let observed = require_positive_u64(adapter, "observed_event_count")?;
        let matched_events = require_positive_u64(adapter, "matched_event_count")?;
        if planned != matched || observed != matched_events {
            return Err("adapter planned and observed counts do not reconcile".to_string());
        }
        require_bool(adapter, "local_only", true)?;
        action_sum = action_sum.checked_add(planned).ok_or("adapter action count overflow")?;
        event_sum = event_sum.checked_add(observed).ok_or("adapter event count overflow")?;
    }
    Ok((action_sum, event_sum))
}

fn validate_reconciliation(
    value: &Value,
    plan_file_blake3: &str,
    expected_actions: u64,
    expected_events: u64,
) -> Result<ActionFacts, String> {
    require_string(value, "schema", ACTION_RECONCILIATION_SCHEMA)?;
    require_string(value, "action_plan_digest_blake3", plan_file_blake3)?;
    let planned = require_positive_u64(value, "planned_action_count")?;
    let matched = require_positive_u64(value, "matched_action_count")?;
    let observed = require_positive_u64(value, "observed_event_count")?;
    let matched_events = require_positive_u64(value, "matched_event_count")?;
    if planned != expected_actions || matched != planned || observed != expected_events || matched_events != observed {
        return Err("root action reconciliation counts do not match the plan".to_string());
    }
    for field in [
        "unknown_event_count",
        "missing_action_count",
        "authority_violation_count",
        "fallback_event_count",
        "remote_event_count",
        "cache_only_completion_count",
    ] {
        require_u64(value, field).and_then(|count| {
            if count == 0 {
                Ok(count)
            } else {
                Err(format!("{field} is nonzero: {count}"))
            }
        })?;
    }
    require_bool(value, "local_only", true)?;
    require_empty_array(value, "blockers")?;
    Ok(ActionFacts {
        proof_plan_digest_blake3: String::new(),
        planned_actions: planned,
        matched_actions: matched,
        observed_events: observed,
        matched_events,
    })
}

fn validate_trust_report(
    value: &Value,
    deterministic: &DeterministicFacts,
    action: &ActionFacts,
    action_plan_file_blake3: &str,
    reconciliation_file_blake3: &str,
) -> Result<(), String> {
    require_string(value, "schema", TRUST_REPORT_SCHEMA)?;
    require_string(value, "status", REQUIRED_TRUST_STATUS)?;
    require_bool(value, "fixed_point_verified", true)?;
    require_bool(value, "root_action_trust_complete", true)?;
    require_empty_array(value, "blockers")?;
    validate_trust_proof(value, deterministic, &action.proof_plan_digest_blake3)?;
    validate_trust_stages(value)?;
    validate_trust_actions(value, action, action_plan_file_blake3, reconciliation_file_blake3)
}

fn validate_trust_proof(
    value: &Value,
    deterministic: &DeterministicFacts,
    proof_plan_digest: &str,
) -> Result<(), String> {
    let proof = require_object(value, "proof")?;
    require_string(proof, "workflow_version", DETERMINISTIC_PROOF_SCHEMA)?;
    require_string(proof, "verdict", REQUIRED_VERDICT)?;
    require_string(proof, "provider_kind", REQUIRED_PROVIDER_KIND)?;
    require_string(proof, "hermeticity_mode", REQUIRED_HERMETICITY)?;
    require_string(proof, "receipt_digest_blake3", &deterministic.receipt_digest_blake3)?;
    require_string(proof, "source_authority_digest_blake3", &deterministic.source_blake3)?;
    require_string(proof, "proof_bundle_digest_blake3", &deterministic.proof_bundle_digest_blake3)?;
    require_string(proof, "plan_digest_blake3", proof_plan_digest)?;
    require_u64(proof, "substitution_count").and_then(|count| {
        if count == 0 {
            Ok(count)
        } else {
            Err(format!("substitution_count is nonzero: {count}"))
        }
    })?;
    Ok(())
}

fn validate_trust_stages(value: &Value) -> Result<(), String> {
    let stages = require_object(value, "stages")?;
    let planned = require_u64(stages, "planned")?;
    let observed = require_u64(stages, "observed")?;
    let executed = require_u64(stages, "executed")?;
    let restored = require_u64(stages, "restored_checkpoint")?;
    if planned != REQUIRED_STAGE_COUNT || observed != planned {
        return Err("trust report planned and observed stage counts do not match".to_string());
    }
    if executed != REQUIRED_EXECUTED_STAGE_COUNT || restored != REQUIRED_RESTORED_STAGE_COUNT {
        return Err("trust report executed and restored stage counts do not match".to_string());
    }
    for field in ["authority_violations", "fallback_events"] {
        if require_u64(stages, field)? != 0 {
            return Err(format!("trust report stage field is nonzero: {field}"));
        }
    }
    Ok(())
}

fn validate_trust_actions(
    value: &Value,
    action: &ActionFacts,
    plan_file_blake3: &str,
    reconciliation_file_blake3: &str,
) -> Result<(), String> {
    let trust = require_object(value, "action_trust")?;
    require_string(trust, "status", REQUIRED_TRUST_STATUS)?;
    require_string(trust, "plan_digest_blake3", plan_file_blake3)?;
    require_string(trust, "reconciliation_digest_blake3", reconciliation_file_blake3)?;
    for (field, expected) in [
        ("planned_actions", action.planned_actions),
        ("matched_actions", action.matched_actions),
        ("observed_events", action.observed_events),
        ("matched_events", action.matched_events),
    ] {
        if require_u64(trust, field)? != expected {
            return Err(format!("trust report {field} does not match action evidence"));
        }
    }
    for field in [
        "unknown_events",
        "missing_actions",
        "authority_violations",
        "fallback_events",
        "remote_events",
        "cache_only_completions",
    ] {
        if require_u64(trust, field)? != 0 {
            return Err(format!("trust report {field} is nonzero"));
        }
    }
    require_bool(trust, "local_only", true)
}

fn require_object<'a>(value: &'a Value, field: &str) -> Result<&'a Value, String> {
    let child = value.get(field).ok_or_else(|| format!("missing object field {field}"))?;
    if child.is_object() {
        Ok(child)
    } else {
        Err(format!("field {field} is not an object"))
    }
}

fn require_array<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("field {field} is not an array"))
}

fn require_empty_array(value: &Value, field: &str) -> Result<(), String> {
    let items = require_array(value, field)?;
    if items.is_empty() {
        Ok(())
    } else {
        Err(format!("field {field} is not empty"))
    }
}

fn require_string<'a>(value: &'a Value, field: &str, expected: &str) -> Result<&'a str, String> {
    let actual = value.get(field).and_then(Value::as_str).ok_or_else(|| format!("field {field} is not a string"))?;
    require_equal(field, actual, expected)?;
    Ok(actual)
}

fn require_digest<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    let digest = value.get(field).and_then(Value::as_str).ok_or_else(|| format!("field {field} is not a string"))?;
    require_blake3(field, digest)?;
    Ok(digest)
}

fn require_blake3(field: &str, digest: &str) -> Result<(), String> {
    let valid = digest.len() == BLAKE3_HEX_LENGTH
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid {
        Ok(())
    } else {
        Err(format!("field {field} is not lowercase BLAKE3"))
    }
}

fn require_bool(value: &Value, field: &str, expected: bool) -> Result<(), String> {
    let actual = value.get(field).and_then(Value::as_bool).ok_or_else(|| format!("field {field} is not a boolean"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("field {field} is {actual}, expected {expected}"))
    }
}

fn require_u64(value: &Value, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("field {field} is not an unsigned integer"))
}

fn require_positive_u64(value: &Value, field: &str) -> Result<u64, String> {
    let count = require_u64(value, field)?;
    if count >= COUNT_MIN {
        Ok(count)
    } else {
        Err(format!("field {field} must be positive"))
    }
}

fn require_equal(field: &str, actual: &str, expected: &str) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} is {actual}, expected {expected}"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const D0: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const D1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const D2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const D3: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    const D4: &str = "4444444444444444444444444444444444444444444444444444444444444444";
    const D5: &str = "5555555555555555555555555555555555555555555555555555555555555555";
    const D6: &str = "6666666666666666666666666666666666666666666666666666666666666666";
    const D7: &str = "7777777777777777777777777777777777777777777777777777777777777777";

    fn fixture() -> (Value, Value, Value, Value) {
        let adapters = (0..REQUIRED_ADAPTER_COUNT)
            .map(|_| {
                json!({
                    "planned_action_count": 1,
                    "matched_action_count": 1,
                    "observed_event_count": 2,
                    "matched_event_count": 2,
                    "local_only": true
                })
            })
            .collect::<Vec<_>>();
        let deterministic = json!({
            "schema": DETERMINISTIC_PROOF_SCHEMA,
            "workflow_version": DETERMINISTIC_PROOF_SCHEMA,
            "verdict": REQUIRED_VERDICT,
            "selected_provider_kind": REQUIRED_PROVIDER_KIND,
            "hermeticity_mode": REQUIRED_HERMETICITY,
            "blocking_reasons": [],
            "receipt_blake3": D0,
            "source_blake3": D1,
            "vendor_blake3": D2,
            "rebuild_descriptor_blake3": D3,
            "rebuild_authority_plan_blake3": D4,
            "source_built_fixed_point": {
                "action_trust_plan_digest_blake3": D5,
                "action_trust_reconciliation_digest_blake3": D6,
                "final_proof_bundle_digest_blake3": D7
            },
            "runs": [
                {"authority_violations": [], "hermeticity_audit_events": [], "substituted_dependency_identities": [], "output_digests": [{"digest_blake3": D3}]},
                {"authority_violations": [], "hermeticity_audit_events": [], "substituted_dependency_identities": [], "output_digests": [{"digest_blake3": D3}]}
            ]
        });
        let plan = json!({
            "schema": ACTION_PLAN_SCHEMA,
            "proof_plan_digest_blake3": D4,
            "adapter_count": REQUIRED_ADAPTER_COUNT,
            "action_count": REQUIRED_ADAPTER_COUNT,
            "adapters": adapters,
            "local_only": true,
            "cache_only_completion_allowed": false,
            "blockers": []
        });
        let reconciliation = json!({
            "schema": ACTION_RECONCILIATION_SCHEMA,
            "action_plan_digest_blake3": D5,
            "planned_action_count": REQUIRED_ADAPTER_COUNT,
            "matched_action_count": REQUIRED_ADAPTER_COUNT,
            "observed_event_count": REQUIRED_ADAPTER_COUNT * 2,
            "matched_event_count": REQUIRED_ADAPTER_COUNT * 2,
            "unknown_event_count": 0,
            "missing_action_count": 0,
            "authority_violation_count": 0,
            "fallback_event_count": 0,
            "remote_event_count": 0,
            "cache_only_completion_count": 0,
            "local_only": true,
            "blockers": []
        });
        let trust = json!({
            "schema": TRUST_REPORT_SCHEMA,
            "status": REQUIRED_TRUST_STATUS,
            "fixed_point_verified": true,
            "root_action_trust_complete": true,
            "proof": {
                "workflow_version": DETERMINISTIC_PROOF_SCHEMA,
                "verdict": REQUIRED_VERDICT,
                "receipt_digest_blake3": D0,
                "plan_digest_blake3": D4,
                "source_authority_digest_blake3": D1,
                "proof_bundle_digest_blake3": D7,
                "provider_kind": REQUIRED_PROVIDER_KIND,
                "hermeticity_mode": REQUIRED_HERMETICITY,
                "substitution_count": 0
            },
            "stages": {"planned": 6, "observed": 6, "executed": 2, "restored_checkpoint": 4, "authority_violations": 0, "fallback_events": 0},
            "action_trust": {
                "status": REQUIRED_TRUST_STATUS,
                "plan_digest_blake3": D5,
                "reconciliation_digest_blake3": D6,
                "planned_actions": REQUIRED_ADAPTER_COUNT,
                "matched_actions": REQUIRED_ADAPTER_COUNT,
                "observed_events": REQUIRED_ADAPTER_COUNT * 2,
                "matched_events": REQUIRED_ADAPTER_COUNT * 2,
                "unknown_events": 0,
                "missing_actions": 0,
                "authority_violations": 0,
                "fallback_events": 0,
                "remote_events": 0,
                "cache_only_completions": 0,
                "local_only": true
            },
            "blockers": []
        });
        (deterministic, plan, reconciliation, trust)
    }

    #[test]
    fn complete_promotion_links_exact_local_action_evidence() {
        let (deterministic, plan, reconciliation, trust) = fixture();
        let summary = validate_promotion(PromotionEvidence {
            deterministic_proof: &deterministic,
            deterministic_file_blake3: D0,
            action_plan: &plan,
            action_plan_file_blake3: D5,
            action_reconciliation: &reconciliation,
            action_reconciliation_file_blake3: D6,
            trust_report: &trust,
            trust_report_file_blake3: D7,
        })
        .unwrap();
        assert_eq!(summary.planned_actions, REQUIRED_ADAPTER_COUNT);
        assert_eq!(summary.observed_events, REQUIRED_ADAPTER_COUNT * 2);
        assert!(summary.local_only);
    }

    #[test]
    fn promotion_rejects_unknown_event_and_plan_digest_drift() {
        let (deterministic, plan, mut reconciliation, trust) = fixture();
        reconciliation["unknown_event_count"] = json!(1);
        let error = validate_promotion(PromotionEvidence {
            deterministic_proof: &deterministic,
            deterministic_file_blake3: D0,
            action_plan: &plan,
            action_plan_file_blake3: D5,
            action_reconciliation: &reconciliation,
            action_reconciliation_file_blake3: D6,
            trust_report: &trust,
            trust_report_file_blake3: D7,
        })
        .unwrap_err();
        assert!(error.contains("unknown_event_count"));

        let (_, _, mut reconciliation, _) = fixture();
        reconciliation["action_plan_digest_blake3"] = json!(D4);
        let error = validate_reconciliation(&reconciliation, D5, REQUIRED_ADAPTER_COUNT, REQUIRED_ADAPTER_COUNT * 2)
            .unwrap_err();
        assert!(error.contains("action_plan_digest_blake3"));
    }
}
