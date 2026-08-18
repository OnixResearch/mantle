// r[impl distributed_evaluation.feasibility_assessment]
//
// Pure assessment core for the distributed-evaluation feasibility
// exploration. All functions are deterministic over in-memory facts: no I/O,
// no clocks, no environment access. The shell (`distributed_eval_assess`)
// owns file reads, probe execution, and report writes.
//
// The classifier selects exactly one outcome per candidate route:
// - `candidate`: complete evidence, a transportable worker boundary (for the eval-service route),
//   or complete producer evidence (for the evaluate-once route), with no authority blocker.
// - `rejected`: proven authority, transport, streaming, or source-identity blocker for the route.
// - `blocked`: missing or inconclusive inventory, probe, coverage, or producer evidence.

use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const ASSESSMENT_REPORT_SCHEMA: &str = "mantle-distributed-evaluation-assessment-v1";

const MAX_SEAMS: usize = 64;
const MAX_BINDINGS: usize = 64;
const MAX_EVIDENCE_DIGESTS: usize = 64;
const MAX_REASONS: usize = 64;
const MAX_FIELD_CHARS: usize = 4_096;
const BLAKE3_HEX_CHARS: usize = 64;

const SEAM_EMBEDDED_EVALUATOR: &str = "embedded-evaluator";
const SEAM_ISOLATED_WORKER_SESSIONS: &str = "isolated-worker-sessions";
const SEAM_STRICT_WORKER_PROTOCOL: &str = "strict-worker-protocol";
const SEAM_EVALUATION_STREAM_WORKERS: &str = "evaluation-stream-workers";
const SEAM_EVALUATION_BUDGET_CORE: &str = "evaluation-budget-core";
const SEAM_PORTABLE_CLIENT_BOUNDARY: &str = "portable-client-boundary";
const SEAM_SOURCE_STAGING: &str = "source-staging";
const SEAM_REMOTE_BUILD_DATA_PLANE: &str = "remote-build-data-plane";

const ROUTE_EVAL_SERVICE: &str = "eval-service";
const ROUTE_EVALUATE_ONCE: &str = "evaluate-once";

/// The candidate distribution shapes under assessment.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Route {
    EvalService,
    EvaluateOnce,
}

impl Route {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::EvalService => ROUTE_EVAL_SERVICE,
            Self::EvaluateOnce => ROUTE_EVALUATE_ONCE,
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The bounded outcome set for one route.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AssessmentOutcome {
    Candidate,
    Rejected,
    Blocked,
}

impl AssessmentOutcome {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Rejected => "rejected",
            Self::Blocked => "blocked",
        }
    }
}

impl fmt::Display for AssessmentOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The evaluated seam inventory. Every seam must carry one binding identity
/// (source path, accepted spec, or ADR).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct InventoryFacts {
    pub(crate) seams: BTreeSet<String>,
    pub(crate) bindings: BTreeSet<String>,
}

/// Per-route evidence and decision facts, normalized and path-independent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RouteFacts {
    pub(crate) route: Route,
    /// Probe truth: the worker response matched the client response.
    pub(crate) response_matches: bool,
    /// Probe truth: the worker could not produce a response.
    pub(crate) worker_error_observed: bool,
    /// Evidence that evaluation reads only declared inputs.
    pub(crate) hidden_host_local_reads: bool,
    /// The probe transport is framed and length-bounded.
    pub(crate) transport_bounded: bool,
    /// Sources and imports are content-addressed before evaluation.
    pub(crate) source_staged_by_digest: bool,
    /// Cross-boundary streaming evaluation-to-build overlap is demonstrated.
    pub(crate) streaming_overlap_evidence: bool,
    /// Cross-boundary dynamic-goal admission is demonstrated.
    pub(crate) dynamic_goal_evidence: bool,
    /// Evaluator suspension and resumption evidence exists.
    pub(crate) suspension_evidence: bool,
    /// An evaluate-once producer export path exists and is maintained.
    pub(crate) exported_graph_evidence: bool,
    /// The producer revision is pinned.
    pub(crate) producer_bound_evidence: bool,
    /// Exported artifacts are digest-bound in the store.
    pub(crate) artifacts_digest_bound: bool,
    /// A demonstrated authority boundary blocks the route.
    pub(crate) authority_blocker: bool,
    /// Informational: the route can evaluate arbitrary fresh user Nickel.
    pub(crate) fresh_nickel_served: bool,
    /// Evidence digests binding probe or producer artifacts.
    pub(crate) evidence_digests: Vec<String>,
}

/// Complete assessment input.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AssessmentFacts {
    pub(crate) inventory: InventoryFacts,
    pub(crate) routes: Vec<RouteFacts>,
}

/// One route outcome with its deterministic reasons.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RouteOutcome {
    pub(crate) route: Route,
    pub(crate) outcome: AssessmentOutcome,
    pub(crate) reasons: Vec<String>,
}

/// The deterministic assessment report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AssessmentReport {
    pub(crate) schema: String,
    pub(crate) inventory_source_digest: String,
    pub(crate) outcomes: Vec<RouteOutcome>,
    pub(crate) facts: AssessmentFacts,
    pub(crate) non_claims: Vec<String>,
}

pub(crate) fn assessment_non_claims() -> Vec<String> {
    [
        "This report does not claim distributed evaluation in the product.",
        "This report does not claim evaluator equivalence.",
        "This report does not claim remote-build correctness.",
        "This report does not claim cross-boundary streaming or dynamic-goal feasibility.",
        "This report does not claim release eligibility.",
        "A candidate outcome authorizes only a later Cairn change with separate evidence.",
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

pub(crate) fn required_seam_names() -> BTreeSet<&'static str> {
    BTreeSet::from([
        SEAM_EMBEDDED_EVALUATOR,
        SEAM_ISOLATED_WORKER_SESSIONS,
        SEAM_STRICT_WORKER_PROTOCOL,
        SEAM_EVALUATION_STREAM_WORKERS,
        SEAM_EVALUATION_BUDGET_CORE,
        SEAM_PORTABLE_CLIENT_BOUNDARY,
        SEAM_SOURCE_STAGING,
        SEAM_REMOTE_BUILD_DATA_PLANE,
    ])
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn field_bounded(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if value.chars().count() > MAX_FIELD_CHARS {
        return Err(format!("{field} exceeds {MAX_FIELD_CHARS} chars"));
    }
    Ok(())
}

fn route_name(route: &Route) -> &'static str {
    route.as_str()
}

/// Validate fact shapes before classification. Malformed input is a hard
/// error, not a `blocked` outcome.
pub(crate) fn validate_facts(facts: &AssessmentFacts) -> Result<(), String> {
    if facts.inventory.seams.len() > MAX_SEAMS {
        return Err(format!("seams exceed {MAX_SEAMS}"));
    }
    if facts.inventory.bindings.len() > MAX_BINDINGS {
        return Err(format!("bindings exceed {MAX_BINDINGS}"));
    }
    for binding in &facts.inventory.bindings {
        field_bounded("seam binding", binding)?;
    }
    if facts.routes.len() != 2 {
        return Err("assessment must analyze exactly two routes".to_string());
    }
    for route_facts in &facts.routes {
        if route_facts.evidence_digests.len() > MAX_EVIDENCE_DIGESTS {
            return Err(format!("evidence digests exceed {MAX_EVIDENCE_DIGESTS}"));
        }
        for digest in &route_facts.evidence_digests {
            if !is_blake3_hex(digest) {
                return Err(format!("malformed evidence digest for route {}", route_name(&route_facts.route)));
            }
        }
        for binding in &facts.inventory.bindings {
            field_bounded("seam binding", binding)?;
        }
    }
    let eval_service = facts.routes.iter().filter(|route_facts| route_facts.route == Route::EvalService).count();
    let evaluate_once = facts.routes.iter().filter(|route_facts| route_facts.route == Route::EvaluateOnce).count();
    if eval_service != 1 || evaluate_once != 1 {
        return Err("assessment must analyze exactly one eval-service route and one evaluate-once route".to_string());
    }
    Ok(())
}

fn inventory_blocked_reasons(facts: &AssessmentFacts) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    let seams: BTreeSet<&str> = facts.inventory.seams.iter().map(String::as_str).collect();
    for required in required_seam_names() {
        if !seams.contains(required) {
            reasons.push(format!("missing inventory seam: {required}"));
        }
    }
    for seam in &facts.inventory.seams {
        let bound = facts.inventory.bindings.iter().any(|binding| binding.starts_with(&format!("{seam}=")));
        if !bound {
            reasons.push(format!("inventory seam is not bound to an identity: {seam}"));
        }
    }
    debug_assert!(reasons.len() <= MAX_REASONS, "reasons stay bounded");
    reasons
}

fn blocked_reasons(route_facts: &RouteFacts) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    if route_facts.evidence_digests.is_empty() {
        reasons.push(format!("missing evidence digests for route {}", route_name(&route_facts.route)));
    }
    match route_facts.route {
        Route::EvalService => {
            if !route_facts.streaming_overlap_evidence {
                reasons.push("missing cross-boundary streaming-overlap evidence".to_string());
            }
            if !route_facts.dynamic_goal_evidence {
                reasons.push("missing cross-boundary dynamic-goal evidence".to_string());
            }
            if !route_facts.suspension_evidence {
                reasons.push("missing evaluator suspension evidence".to_string());
            }
        }
        Route::EvaluateOnce => {
            if !route_facts.exported_graph_evidence {
                reasons.push("missing evaluate-once producer export evidence".to_string());
            }
            if !route_facts.producer_bound_evidence {
                reasons.push("missing evaluate-once producer revision binding".to_string());
            }
        }
    }
    debug_assert!(reasons.len() <= MAX_REASONS, "reasons stay bounded");
    reasons
}

fn rejected_reasons(route_facts: &RouteFacts) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    if route_facts.hidden_host_local_reads {
        reasons.push("hidden host-local reads detected".to_string());
    }
    match route_facts.route {
        Route::EvalService => {
            if route_facts.worker_error_observed {
                reasons.push("worker failed to produce a response".to_string());
            }
            if !route_facts.response_matches {
                reasons.push("worker response did not match the client response".to_string());
            }
            if !route_facts.transport_bounded {
                reasons.push("transport is not framed or length-bounded".to_string());
            }
            if !route_facts.source_staged_by_digest {
                reasons.push("sources and imports are not content-addressed".to_string());
            }
        }
        Route::EvaluateOnce => {
            if !route_facts.artifacts_digest_bound {
                reasons.push("exported artifacts are not digest-bound".to_string());
            }
        }
    }
    if route_facts.authority_blocker {
        reasons.push("demonstrated authority boundary blocks the route".to_string());
    }
    debug_assert!(reasons.len() <= MAX_REASONS, "reasons stay bounded");
    reasons
}

fn candidate_failures(route_facts: &RouteFacts) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    match route_facts.route {
        Route::EvalService => {
            if !route_facts.transport_bounded {
                failures.push("transport must be framed and length-bounded".to_string());
            }
            if !route_facts.source_staged_by_digest {
                failures.push("sources must be content-addressed".to_string());
            }
        }
        Route::EvaluateOnce => {
            if !route_facts.exported_graph_evidence {
                failures.push("the producer export path must exist".to_string());
            }
            if !route_facts.producer_bound_evidence {
                failures.push("the producer must be revision-bound".to_string());
            }
        }
    }
    debug_assert!(failures.len() <= MAX_REASONS, "failures stay bounded");
    failures
}

fn classify_route(route_facts: &RouteFacts) -> RouteOutcome {
    let blocked = blocked_reasons(route_facts);
    let (outcome, reasons) = if !blocked.is_empty() {
        (AssessmentOutcome::Blocked, blocked)
    } else {
        let rejected = rejected_reasons(route_facts);
        if !rejected.is_empty() {
            (AssessmentOutcome::Rejected, rejected)
        } else {
            let failures = candidate_failures(route_facts);
            if failures.is_empty() {
                (AssessmentOutcome::Candidate, vec![format!(
                    "route {} passes all assessment gates",
                    route_name(&route_facts.route)
                )])
            } else {
                (AssessmentOutcome::Blocked, failures)
            }
        }
    };
    debug_assert!(!reasons.is_empty(), "every outcome must carry reasons");
    RouteOutcome {
        route: route_facts.route.clone(),
        outcome,
        reasons,
    }
}

/// Classify the assessment into exactly one bounded outcome per route with
/// deterministic reasons.
pub(crate) fn classify(facts: &AssessmentFacts) -> AssessmentReport {
    debug_assert!(validate_facts(facts).is_ok(), "facts must be validated first");
    let inventory_reasons = inventory_blocked_reasons(facts);
    let mut outcomes: Vec<RouteOutcome> = Vec::with_capacity(facts.routes.len());
    for route_facts in &facts.routes {
        let outcome = if inventory_reasons.is_empty() {
            classify_route(route_facts)
        } else {
            RouteOutcome {
                route: route_facts.route.clone(),
                outcome: AssessmentOutcome::Blocked,
                reasons: inventory_reasons.clone(),
            }
        };
        outcomes.push(outcome);
    }
    debug_assert!(outcomes.len() == 2, "exactly two route outcomes");
    AssessmentReport {
        schema: ASSESSMENT_REPORT_SCHEMA.to_string(),
        inventory_source_digest: String::new(),
        outcomes,
        facts: facts.clone(),
        non_claims: assessment_non_claims(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[verify distributed_evaluation.feasibility_assessment]

    fn digests() -> Vec<String> {
        vec!["a".repeat(BLAKE3_HEX_CHARS)]
    }

    fn complete_inventory() -> InventoryFacts {
        let seams: BTreeSet<String> = required_seam_names().iter().map(ToString::to_string).collect();
        let bindings: BTreeSet<String> = required_seam_names()
            .iter()
            .map(|seam| format!("{seam}=#crates/crunch-eval/src/session.rs"))
            .collect();
        InventoryFacts { seams, bindings }
    }

    fn eval_service_facts() -> RouteFacts {
        RouteFacts {
            route: Route::EvalService,
            response_matches: true,
            worker_error_observed: false,
            hidden_host_local_reads: false,
            transport_bounded: true,
            source_staged_by_digest: true,
            streaming_overlap_evidence: true,
            dynamic_goal_evidence: true,
            suspension_evidence: true,
            exported_graph_evidence: false,
            producer_bound_evidence: false,
            artifacts_digest_bound: false,
            authority_blocker: false,
            fresh_nickel_served: true,
            evidence_digests: digests(),
        }
    }

    fn evaluate_once_facts() -> RouteFacts {
        RouteFacts {
            route: Route::EvaluateOnce,
            response_matches: false,
            worker_error_observed: false,
            hidden_host_local_reads: false,
            transport_bounded: false,
            source_staged_by_digest: true,
            streaming_overlap_evidence: false,
            dynamic_goal_evidence: false,
            suspension_evidence: false,
            exported_graph_evidence: true,
            producer_bound_evidence: true,
            artifacts_digest_bound: true,
            authority_blocker: false,
            fresh_nickel_served: false,
            evidence_digests: digests(),
        }
    }

    fn complete_facts() -> AssessmentFacts {
        AssessmentFacts {
            inventory: complete_inventory(),
            routes: vec![eval_service_facts(), evaluate_once_facts()],
        }
    }

    #[test]
    fn complete_facts_select_candidate_for_both_routes() {
        let report = classify(&complete_facts());
        assert!(report.outcomes.iter().all(|outcome| outcome.outcome == AssessmentOutcome::Candidate));
        assert!(!report.non_claims.is_empty());
    }

    #[test]
    fn missing_required_seam_blocks() {
        let mut facts = complete_facts();
        facts.inventory.seams.remove(SEAM_EMBEDDED_EVALUATOR);
        let report = classify(&facts);
        assert!(report.outcomes.iter().all(|outcome| outcome.outcome == AssessmentOutcome::Blocked));
        assert!(report.outcomes[0].reasons.iter().any(|reason| reason.contains(SEAM_EMBEDDED_EVALUATOR)));
    }

    #[test]
    fn unbound_seam_blocks() {
        let mut facts = complete_facts();
        facts
            .inventory
            .bindings
            .remove(&format!("{SEAM_EMBEDDED_EVALUATOR}=#crates/crunch-eval/src/session.rs"));
        let report = classify(&facts);
        assert!(report.outcomes.iter().all(|outcome| outcome.outcome == AssessmentOutcome::Blocked));
        assert!(report.outcomes[0].reasons.iter().any(|reason| reason.contains("not bound")));
    }

    #[test]
    fn missing_probe_digests_block_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].evidence_digests.clear();
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Blocked);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("evidence digests")));
    }

    #[test]
    fn missing_streaming_evidence_blocks_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].streaming_overlap_evidence = false;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Blocked);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("streaming")));
    }

    #[test]
    fn missing_dynamic_goal_evidence_blocks_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].dynamic_goal_evidence = false;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Blocked);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("dynamic-goal")));
    }

    #[test]
    fn missing_suspension_evidence_blocks_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].suspension_evidence = false;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Blocked);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("suspension")));
    }

    #[test]
    fn hidden_host_local_reads_reject_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].hidden_host_local_reads = true;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Rejected);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("hidden")));
    }

    #[test]
    fn worker_error_rejects_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].worker_error_observed = true;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Rejected);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("worker failed")));
    }

    #[test]
    fn response_mismatch_rejects_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].response_matches = false;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Rejected);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("did not match")));
    }

    #[test]
    fn unbounded_transport_rejects_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].transport_bounded = false;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Rejected);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("length-bounded")));
    }

    #[test]
    fn missing_source_staging_rejects_eval_service() {
        let mut facts = complete_facts();
        facts.routes[0].source_staged_by_digest = false;
        let report = classify(&facts);
        let eval_service = report.outcomes.iter().find(|o| o.route == Route::EvalService).expect("route present");
        assert_eq!(eval_service.outcome, AssessmentOutcome::Rejected);
        assert!(eval_service.reasons.iter().any(|reason| reason.contains("content-addressed")));
    }

    #[test]
    fn authority_blocker_rejects_both_routes() {
        let mut facts = complete_facts();
        facts.routes[0].authority_blocker = true;
        facts.routes[1].authority_blocker = true;
        let report = classify(&facts);
        assert!(report.outcomes.iter().all(|outcome| outcome.outcome == AssessmentOutcome::Rejected));
    }

    #[test]
    fn missing_export_evidence_blocks_evaluate_once() {
        let mut facts = complete_facts();
        facts.routes[1].exported_graph_evidence = false;
        let report = classify(&facts);
        let evaluate_once = report.outcomes.iter().find(|o| o.route == Route::EvaluateOnce).expect("route present");
        assert_eq!(evaluate_once.outcome, AssessmentOutcome::Blocked);
        assert!(evaluate_once.reasons.iter().any(|reason| reason.contains("producer export")));
    }

    #[test]
    fn missing_producer_binding_blocks_evaluate_once() {
        let mut facts = complete_facts();
        facts.routes[1].producer_bound_evidence = false;
        let report = classify(&facts);
        let evaluate_once = report.outcomes.iter().find(|o| o.route == Route::EvaluateOnce).expect("route present");
        assert_eq!(evaluate_once.outcome, AssessmentOutcome::Blocked);
        assert!(evaluate_once.reasons.iter().any(|reason| reason.contains("revision binding")));
    }

    #[test]
    fn undigested_artifacts_reject_evaluate_once() {
        let mut facts = complete_facts();
        facts.routes[1].artifacts_digest_bound = false;
        let report = classify(&facts);
        let evaluate_once = report.outcomes.iter().find(|o| o.route == Route::EvaluateOnce).expect("route present");
        assert_eq!(evaluate_once.outcome, AssessmentOutcome::Rejected);
        assert!(evaluate_once.reasons.iter().any(|reason| reason.contains("digest-bound")));
    }

    #[test]
    fn fresh_nickel_limitation_does_not_gate_evaluate_once() {
        let mut facts = complete_facts();
        facts.routes[1].fresh_nickel_served = false;
        let report = classify(&facts);
        let evaluate_once = report.outcomes.iter().find(|o| o.route == Route::EvaluateOnce).expect("route present");
        assert_eq!(evaluate_once.outcome, AssessmentOutcome::Candidate);
    }

    #[test]
    fn malformed_digest_is_a_hard_error() {
        let mut facts = complete_facts();
        facts.routes[0].evidence_digests = vec!["not-hex".to_string()];
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn missing_route_is_a_hard_error() {
        let mut facts = complete_facts();
        facts.routes.pop();
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn duplicate_route_is_a_hard_error() {
        let mut facts = complete_facts();
        facts.routes[1].route = Route::EvalService;
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn oversized_seams_are_a_hard_error() {
        let mut facts = complete_facts();
        facts.inventory.seams = (0..MAX_SEAMS + 1).map(|index| format!("extra-seam-{index}")).collect();
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn oversized_evidence_set_is_a_hard_error() {
        let mut facts = complete_facts();
        facts.routes[0].evidence_digests = vec!["a".repeat(BLAKE3_HEX_CHARS); MAX_EVIDENCE_DIGESTS + 1];
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn empty_binding_is_a_hard_error() {
        let mut facts = complete_facts();
        facts.inventory.bindings.insert(String::new());
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn classification_is_deterministic() {
        let facts = complete_facts();
        assert_eq!(classify(&facts), classify(&facts));
    }
}
