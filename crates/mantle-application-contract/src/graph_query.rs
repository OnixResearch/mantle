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
use crate::envelope::EffectPlan;
use crate::envelope::Observation;
use crate::envelope::ObservationStatus;
use crate::envelope::plan_effects;
use crate::family::CommandFamily;

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
}

impl GraphQueryKind {
    /// Every kind in canonical order.
    pub fn all() -> Vec<Self> {
        let kinds = vec![Self::Graph, Self::Why, Self::Dependents];
        debug_assert_eq!(kinds.len(), 3);
        debug_assert!(!kinds.is_empty());
        kinds
    }

    /// Stable kind label.
    pub fn as_str(self) -> &'static str {
        let label = match self {
            Self::Graph => "graph",
            Self::Why => "why",
            Self::Dependents => "dependents",
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
}

/// Entity facts reported by one finished graph query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphQueryFacts {
    pub nodes: u32,
    pub edges: u32,
    pub aliases: u32,
    pub dependents: u32,
}

impl GraphQueryFacts {
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
    fn query(&mut self, request: &GraphQueryRequest) -> Result<GraphQueryFacts, CapabilityError>;
}

/// The bounded effect plan for one graph query.
pub fn graph_query_effect_plan() -> Result<EffectPlan, CapabilityError> {
    plan_effects(CommandFamily::Planning, &[GRAPH_QUERY_READ_EFFECT]).ok_or_else(|| {
        CapabilityError::new("graph-query-plan-rejected", "the graph-query effect plan exceeds its bound")
    })
}

/// Validate one graph query before any port is called.
pub fn validate_graph_query(request: &GraphQueryRequest) -> Vec<ApplicationBlocker> {
    let mut blockers = Vec::with_capacity(1);
    if request.target.trim().is_empty() {
        blockers.push(ApplicationBlocker::new(
            "graph-query-target-required",
            request.kind.as_str(),
            "a graph query must name the target it queries",
        ));
    }
    debug_assert!(blockers.len() <= 1);
    debug_assert!(blockers.is_empty() || request.target.trim().is_empty());
    blockers
}

/// Classify one finished query against its plan.
///
/// A query whose facts exceed the admitted bound is reported as a failed
/// observation, so no caller can report an unbounded graph view.
pub fn classify_graph_query(plan: &EffectPlan, facts: &GraphQueryFacts) -> ApplicationOutcome {
    let status = if facts.is_bounded() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    let observation = Observation {
        effect_id: plan.effects.first().map_or_else(
            || crate::envelope::EffectId(String::from(GRAPH_QUERY_READ_EFFECT)),
            |effect| effect.effect_id.clone(),
        ),
        status,
        diagnostics_code: if facts.is_bounded() {
            None
        } else {
            Some(format!("graph-query-fact-exceeds-{MAX_GRAPH_QUERY_ENTITIES}"))
        },
    };
    let observations = vec![observation];
    let outcome = crate::envelope::classify_observations(plan, &observations);
    debug_assert!(matches!(
        outcome,
        ApplicationOutcome::Completed | ApplicationOutcome::Failed { .. } | ApplicationOutcome::Rejected { .. }
    ));
    debug_assert!(plan.effects.len() == 1);
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> EffectPlan {
        graph_query_effect_plan().expect("the graph-query plan is admitted")
    }

    fn facts(nodes: u32) -> GraphQueryFacts {
        GraphQueryFacts {
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
        assert_eq!(outcome, ApplicationOutcome::Failed { failed_effect_count: 1 });
        assert!(!facts(MAX_GRAPH_QUERY_ENTITIES.saturating_add(1)).is_bounded());
        assert!(facts(MAX_GRAPH_QUERY_ENTITIES).is_bounded());
    }

    #[test]
    fn an_empty_target_is_rejected_before_any_port() {
        let request = GraphQueryRequest {
            kind: GraphQueryKind::Why,
            target: String::from("   "),
        };
        let blockers = validate_graph_query(&request);
        assert_eq!(blockers.len(), 1);
        assert_eq!(blockers[0].code, "graph-query-target-required");
        let accepted = GraphQueryRequest {
            kind: GraphQueryKind::Graph,
            target: String::from("root"),
        };
        assert!(validate_graph_query(&accepted).is_empty());
    }

    #[test]
    fn kind_labels_are_stable() {
        assert_eq!(GraphQueryKind::all().len(), 3);
        assert_eq!(GraphQueryKind::Graph.as_str(), "graph");
        assert_eq!(GraphQueryKind::Why.as_str(), "why");
        assert_eq!(GraphQueryKind::Dependents.as_str(), "dependents");
    }
}
