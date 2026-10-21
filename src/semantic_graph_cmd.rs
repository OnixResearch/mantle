//! Bounded saved-graph queries through the existing read-effect contract.

use std::path::Path;

use mantle_application_contract::EffectPlan;
use mantle_application_contract::GraphQueryFacts;
use mantle_application_contract::GraphQueryKind;
use mantle_application_contract::GraphQueryPort;
use mantle_application_contract::GraphQueryRequest;

use crate::RunContext;
use crate::command_input::SemanticGraphCommandInput;
use crate::errors::RunError;
use crate::presentation::semantic_graph as presentation;
use crate::semantic_graph::Direction;
use crate::semantic_graph::SemanticGraph;
use crate::semantic_graph::SemanticGraphError;

pub(crate) fn run(ctx: &RunContext, input: SemanticGraphCommandInput<'_>) -> Result<(), RunError> {
    let request = input.command_request();
    if let Some(blocker) = mantle_application_contract::validate_graph_query(&request).first() {
        return Err(RunError::Internal(format!("{}: {}", blocker.code, blocker.message)));
    }
    debug_assert!(!request.target.trim().is_empty());
    debug_assert!(ctx.store_prefix.starts_with('/'));
    let path = input
        .graph_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| ctx.resolved_state_dir.join("semantic-graph.json"));
    let graph = SemanticGraph::load(&path).map_err(|error| report_error(ctx, error))?;
    graph.admit_query_limits(request.limits).map_err(|error| report_error(ctx, error))?;
    let expected = graph.canonical_identity(&request.target).unwrap_or(&request.target);
    let plan = mantle_application_contract::graph_query_effect_plan(expected)
        .map_err(|error| RunError::Internal(format!("{}: {}", error.code, error.detail)))?;
    let rendered = execute(ctx, &graph, &request, &plan)?;
    println!("{rendered}");
    Ok(())
}

fn execute(
    ctx: &RunContext,
    graph: &SemanticGraph,
    request: &GraphQueryRequest,
    plan: &EffectPlan,
) -> Result<String, RunError> {
    debug_assert!(!request.target.is_empty());
    debug_assert!(request.limits.nodes > 0);
    match request.kind {
        GraphQueryKind::Graph => render_graph(ctx, graph, request, plan),
        GraphQueryKind::Why => render_why(ctx, graph, request, plan),
        GraphQueryKind::Dependents => render_dependents(ctx, graph, request, plan),
        GraphQueryKind::Dependencies | GraphQueryKind::ReverseDependencies | GraphQueryKind::Path => {
            render_traversal(ctx, graph, request, plan)
        }
        GraphQueryKind::CompareAction => render_action_comparison(ctx, graph, request, plan),
        GraphQueryKind::CompareConfiguration => render_configuration_comparison(ctx, graph, request, plan),
    }
}

fn count(observed: usize) -> Result<u32, RunError> {
    u32::try_from(observed).map_err(|_| RunError::Internal("semantic graph entity count exceeds u32".to_string()))
}

fn render_graph(
    ctx: &RunContext,
    graph: &SemanticGraph,
    request: &GraphQueryRequest,
    plan: &EffectPlan,
) -> Result<String, RunError> {
    debug_assert_eq!(request.kind, GraphQueryKind::Graph);
    debug_assert!(request.limits.nodes > 0);
    let result = graph
        .graph_for_root_bounded(&request.target, request.limits)
        .map_err(|error| report_error(ctx, error))?;
    classify(request, plan, GraphQueryFacts {
        observed_target: &result.root,
        nodes: count(result.nodes.len())?,
        edges: count(result.edges.len())?,
        aliases: count(result.aliases.len())?,
        dependents: 0,
    })?;
    presentation::render_graph_result(ctx.output_format(), &result)
}

fn render_why(
    ctx: &RunContext,
    graph: &SemanticGraph,
    request: &GraphQueryRequest,
    plan: &EffectPlan,
) -> Result<String, RunError> {
    debug_assert_eq!(request.kind, GraphQueryKind::Why);
    debug_assert!(request.limits.nodes > 0);
    let result = graph.why(&request.target).map_err(|error| report_error(ctx, error))?;
    classify(request, plan, GraphQueryFacts {
        observed_target: &result.target,
        nodes: count(result.sources.len())?,
        edges: count(result.providers.len())?,
        aliases: count(result.proof_receipts.len())?,
        dependents: count(result.release_evidence.len())?,
    })?;
    presentation::render_why_result(ctx.output_format(), &result)
}

fn render_dependents(
    ctx: &RunContext,
    graph: &SemanticGraph,
    request: &GraphQueryRequest,
    plan: &EffectPlan,
) -> Result<String, RunError> {
    debug_assert_eq!(request.kind, GraphQueryKind::Dependents);
    debug_assert!(request.limits.nodes > 0);
    let result = graph.dependents(&request.target).map_err(|error| report_error(ctx, error))?;
    classify(request, plan, GraphQueryFacts {
        observed_target: &result.target,
        nodes: count(result.dependents.len())?,
        edges: 0,
        aliases: 0,
        dependents: count(result.dependents.len())?,
    })?;
    presentation::render_dependents_result(ctx.output_format(), &result)
}

fn render_traversal(
    ctx: &RunContext,
    graph: &SemanticGraph,
    request: &GraphQueryRequest,
    plan: &EffectPlan,
) -> Result<String, RunError> {
    debug_assert!(!request.target.is_empty());
    debug_assert!(request.limits.steps > 0);
    let direction = if request.kind == GraphQueryKind::ReverseDependencies {
        Direction::ReverseDependencies
    } else {
        Direction::Dependencies
    };
    let result = graph
        .traverse(&request.target, direction, request.other_target.as_deref(), request.limits)
        .map_err(|error| report_error(ctx, error))?;
    classify(request, plan, GraphQueryFacts {
        observed_target: &result.root,
        nodes: count(result.reachable.identities.len())?,
        edges: result.reachable.examined_edges,
        aliases: 0,
        dependents: 0,
    })?;
    presentation::render_traversal_result(ctx.output_format(), &result)
}

fn other_target(request: &GraphQueryRequest) -> Result<&str, RunError> {
    request
        .other_target
        .as_deref()
        .ok_or_else(|| RunError::Internal("comparison target required".into()))
}

fn render_action_comparison(
    ctx: &RunContext,
    graph: &SemanticGraph,
    request: &GraphQueryRequest,
    plan: &EffectPlan,
) -> Result<String, RunError> {
    debug_assert_eq!(request.kind, GraphQueryKind::CompareAction);
    debug_assert!(request.limits.nodes > 0);
    let result = graph
        .compare_action(&request.target, graph, other_target(request)?)
        .map_err(|error| report_error(ctx, error))?;
    classify(request, plan, GraphQueryFacts {
        observed_target: &result.before,
        nodes: count(result.facts.len())?,
        edges: 0,
        aliases: 0,
        dependents: 0,
    })?;
    presentation::render_action_comparison(ctx.output_format(), &result)
}

fn render_configuration_comparison(
    ctx: &RunContext,
    graph: &SemanticGraph,
    request: &GraphQueryRequest,
    plan: &EffectPlan,
) -> Result<String, RunError> {
    debug_assert_eq!(request.kind, GraphQueryKind::CompareConfiguration);
    debug_assert!(request.limits.nodes > 0);
    let result = graph
        .compare_configuration(&request.target, graph, other_target(request)?, request.limits)
        .map_err(|error| report_error(ctx, error))?;
    classify(request, plan, GraphQueryFacts {
        observed_target: &result.before,
        nodes: count(result.observed_dependencies_added.len())?,
        edges: count(result.observed_dependencies_removed.len())?,
        aliases: 0,
        dependents: 0,
    })?;
    presentation::render_configuration_comparison(ctx.output_format(), &result)
}

struct ObservedQuery<'a> {
    kind: GraphQueryKind,
    facts: GraphQueryFacts<'a>,
}

impl GraphQueryPort for ObservedQuery<'_> {
    fn query(
        &mut self,
        request: &GraphQueryRequest,
    ) -> Result<GraphQueryFacts<'_>, mantle_application_contract::CapabilityError> {
        if request.kind != self.kind {
            return Err(mantle_application_contract::CapabilityError::new(
                "graph-query-kind-mismatch",
                "observed facts do not match the requested query kind",
            ));
        }
        debug_assert_eq!(request.kind, self.kind);
        debug_assert!(!request.target.is_empty());
        Ok(self.facts)
    }
}

fn classify(request: &GraphQueryRequest, plan: &EffectPlan, facts: GraphQueryFacts<'_>) -> Result<(), RunError> {
    let mut port = ObservedQuery {
        kind: request.kind,
        facts,
    };
    let observed = port
        .query(request)
        .map_err(|error| RunError::Internal(format!("{}: {}", error.code, error.detail)))?;
    match mantle_application_contract::classify_graph_query(plan, &observed) {
        mantle_application_contract::ApplicationOutcome::Completed => Ok(()),
        outcome => Err(RunError::Internal(format!("semantic graph observations were not successful: {outcome:?}"))),
    }
}

fn report_error(ctx: &RunContext, error: SemanticGraphError) -> RunError {
    if let SemanticGraphError::Incomplete(diagnostic) = error {
        if ctx.json {
            match serde_json::to_string_pretty(&diagnostic) {
                Ok(rendered) => println!("{rendered}"),
                Err(error) => return RunError::Internal(error.to_string()),
            }
        } else {
            eprintln!(
                "incomplete semantic graph for {} `{}`: missing {}",
                diagnostic.query,
                diagnostic.target,
                diagnostic.missing.join(", ")
            );
        }
        return RunError::Reported(1);
    }
    RunError::Internal(error.to_string())
}
