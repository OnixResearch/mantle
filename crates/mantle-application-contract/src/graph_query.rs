//! Typed graph-query command, effect plan, and observation classification.
//!
//! A query reads the semantic graph once and reports bounded entity counts. The
//! adapter observes those counts, and the classification fails closed when a
//! fact exceeds the admitted bound instead of reporting a partial view.

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;
use crate::envelope::ApplicationOutcome;
use crate::envelope::CapabilityError;
use crate::envelope::EffectId;
use crate::envelope::EffectKind;
use crate::envelope::EffectMeasure;
use crate::envelope::EffectOutput;
use crate::envelope::EffectPlan;
use crate::envelope::EffectSpec;
use crate::envelope::ExpectedOutput;
use crate::envelope::Observation;
use crate::envelope::ObservationStatus;
use crate::envelope::plan_effects;
use crate::family::CommandFamily;

#[path = "graph_inspection.rs"]
pub mod inspection;

/// Largest admitted entity count for one graph-query fact.
pub const MAX_GRAPH_QUERY_ENTITIES: u32 = 1_048_576;

/// Effect identity of the single planned graph read.
pub const GRAPH_QUERY_READ_EFFECT: &str = "read-graph";

/// Graph query kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GraphQueryKind {
    /// Whole reachable graph for one root.
    Graph,
    /// Why one node exists.
    Why,
    /// Who depends on one node.
    Dependents,
    /// Directed transitive dependencies, including the named root.
    Dependencies,
    /// Directed transitive reverse dependencies, including the named root.
    ReverseDependencies,
    /// One deterministic shortest directed dependency path.
    Path,
    /// Compare configured targets in the saved graph.
    CompareConfiguration,
    /// Explain declared action key changes in the saved graph.
    CompareAction,
}

impl GraphQueryKind {
    /// Every kind in canonical order.
    pub fn all() -> Vec<Self> {
        let kinds = vec![
            Self::Graph,
            Self::Why,
            Self::Dependents,
            Self::Dependencies,
            Self::ReverseDependencies,
            Self::Path,
            Self::CompareConfiguration,
            Self::CompareAction,
        ];
        debug_assert_eq!(kinds.len(), 8);
        debug_assert!(!kinds.is_empty());
        kinds
    }

    /// Stable kind label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Graph => "graph",
            Self::Why => "why",
            Self::Dependents => "dependents",
            Self::Dependencies => "dependencies",
            Self::ReverseDependencies => "reverse-dependencies",
            Self::Path => "path",
            Self::CompareConfiguration => "compare-configuration",
            Self::CompareAction => "compare-action",
        };
        debug_assert!(!label.is_empty());
        debug_assert!(Self::all().contains(&self));
        label
    }
}

/// One typed graph-query request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphQueryRequest {
    pub kind: GraphQueryKind,
    /// Root, node, or target the query names.
    pub target: String,
    /// Destination or comparison identity; required only by binary queries.
    pub other_target: Option<String>,
    pub limits: inspection::QueryLimits,
}

/// Entity facts and actual result identity reported by one finished query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphQueryFacts<'a> {
    pub observed_target: &'a str,
    pub nodes: u32,
    pub edges: u32,
    pub aliases: u32,
    pub dependents: u32,
}

impl GraphQueryFacts<'_> {
    /// Whether every fact sits inside the admitted bound.
    pub fn is_bounded(&self) -> bool {
        let bound = MAX_GRAPH_QUERY_ENTITIES;
        let is_bounded =
            self.nodes <= bound && self.edges <= bound && self.aliases <= bound && self.dependents <= bound;
        debug_assert!(is_bounded || self.nodes > bound || self.edges > bound || self.aliases > bound);
        debug_assert!(is_bounded || self.dependents > bound || self.nodes > bound);
        is_bounded
    }
}

/// Port: execute one typed graph query and report its facts.
pub trait GraphQueryPort {
    fn query(&mut self, request: &GraphQueryRequest) -> Result<GraphQueryFacts<'_>, CapabilityError>;
}

/// The bounded effect plan for one query and its declared target.
pub fn graph_query_effect_plan(target: &str) -> Result<EffectPlan, CapabilityError> {
    plan_effects(CommandFamily::Planning, &[EffectSpec {
        effect_id: GRAPH_QUERY_READ_EFFECT,
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Items(MAX_GRAPH_QUERY_ENTITIES),
        expected_output: ExpectedOutput::Identity(target),
    }])
    .map_err(|error| CapabilityError::new(error.code(), "the graph-query effect plan was rejected"))
}

/// Validate one graph query before any port is called.
pub fn validate_graph_query(request: &GraphQueryRequest) -> Vec<ApplicationBlocker> {
    let mut blockers = Vec::with_capacity(3);
    if request.target.trim().is_empty() {
        blockers.push(ApplicationBlocker::new(
            "graph-query-target-required",
            request.kind.as_str(),
            "a graph query must name the target it queries",
        ));
    }
    let is_binary = matches!(
        request.kind,
        GraphQueryKind::Path | GraphQueryKind::CompareConfiguration | GraphQueryKind::CompareAction
    );
    if is_binary != request.other_target.is_some()
        || request.other_target.as_ref().is_some_and(|target| target.trim().is_empty())
    {
        blockers.push(ApplicationBlocker::new(
            "graph-query-other-target",
            request.kind.as_str(),
            "binary queries require a nonempty second target; unary queries reject one",
        ));
    }
    if request.limits.validate().is_err() {
        blockers.push(ApplicationBlocker::new(
            "graph-query-limits",
            request.kind.as_str(),
            "graph query limits must be nonzero and inside hard admission bounds",
        ));
    }
    debug_assert!(blockers.len() <= 3);
    debug_assert!(!blockers.is_empty() || !request.target.trim().is_empty());
    blockers
}

/// Classify the actual result identity and bounded entity facts.
pub fn classify_graph_query(plan: &EffectPlan, facts: &GraphQueryFacts<'_>) -> ApplicationOutcome {
    let largest_count = facts.nodes.max(facts.edges).max(facts.aliases).max(facts.dependents);
    let is_bounded = facts.is_bounded();
    let observation = Observation {
        effect_id: EffectId(String::from(GRAPH_QUERY_READ_EFFECT)),
        kind: EffectKind::ReadFiles,
        status: if is_bounded {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: EffectOutput::Identity(String::from(facts.observed_target)),
        usage: EffectMeasure::Items(largest_count),
        diagnostics_code: (!is_bounded).then(|| format!("graph-query-fact-exceeds-{MAX_GRAPH_QUERY_ENTITIES}")),
    };
    let outcome = crate::envelope::classify_observations(plan, &[observation]);
    debug_assert!(outcome != ApplicationOutcome::Completed || is_bounded);
    debug_assert!(plan.effects.len() == 1);
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> EffectPlan {
        graph_query_effect_plan("root").expect("the graph-query plan is admitted")
    }

    fn facts(nodes: u32) -> GraphQueryFacts<'static> {
        GraphQueryFacts {
            observed_target: "root",
            nodes,
            edges: 0,
            aliases: 0,
            dependents: 0,
        }
    }

    #[test]
    fn the_plan_holds_one_bounded_read_effect() {
        let plan = plan();
        assert_eq!(plan.effects.len(), 1);
        assert_eq!(plan.effects[0].effect_id.0, GRAPH_QUERY_READ_EFFECT);
        assert_eq!(plan.effects[0].family, CommandFamily::Planning);
    }

    #[test]
    fn bounded_facts_complete() {
        let outcome = classify_graph_query(&plan(), &facts(7));
        assert_eq!(outcome, ApplicationOutcome::Completed);
    }

    #[test]
    fn an_unbounded_fact_fails_closed() {
        let outcome = classify_graph_query(&plan(), &facts(MAX_GRAPH_QUERY_ENTITIES.saturating_add(1)));
        assert_eq!(outcome, ApplicationOutcome::Contradicted { effect_count: 1 });
        assert!(!facts(MAX_GRAPH_QUERY_ENTITIES.saturating_add(1)).is_bounded());
        assert!(facts(MAX_GRAPH_QUERY_ENTITIES).is_bounded());
    }

    #[test]
    fn a_result_for_another_graph_target_is_rejected() {
        let observed = GraphQueryFacts {
            observed_target: "another-root",
            ..facts(2)
        };
        assert_eq!(classify_graph_query(&plan(), &observed), ApplicationOutcome::Contradicted { effect_count: 1 });
    }

    #[test]
    fn an_empty_target_is_rejected_before_any_port() {
        let request = GraphQueryRequest {
            kind: GraphQueryKind::Why,
            target: String::from("   "),
            other_target: None,
            limits: inspection::QueryLimits::default(),
        };
        let blockers = validate_graph_query(&request);
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].code, "graph-query-target-required");
        let accepted = GraphQueryRequest {
            kind: GraphQueryKind::Graph,
            target: String::from("root"),
            other_target: None,
            limits: inspection::QueryLimits::default(),
        };
        assert!(validate_graph_query(&accepted).is_empty());
    }

    #[test]
    fn binary_queries_require_destinations_and_reject_invalid_resource_limits() {
        let mut request = GraphQueryRequest {
            kind: GraphQueryKind::Path,
            target: String::from("root"),
            other_target: None,
            limits: inspection::QueryLimits::default(),
        };
        assert_eq!(validate_graph_query(&request)[0].code, "graph-query-other-target");
        request.other_target = Some(String::from("destination"));
        assert!(validate_graph_query(&request).is_empty());
        request.kind = GraphQueryKind::Graph;
        assert_eq!(validate_graph_query(&request)[0].code, "graph-query-other-target");
        request.other_target = None;
        request.limits.steps = 0;
        assert_eq!(validate_graph_query(&request)[0].code, "graph-query-limits");
    }
}
