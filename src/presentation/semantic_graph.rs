//! Rendering for semantic-graph query results.
//!
//! The three renderers are pure: one typed result plus one output format in,
//! one rendered string out. The root keeps the CLI wiring and the exit codes.

use mantle_application_contract::OutputFormat;
use serde::Serialize;
use serde::ser::SerializeSeq;

use crate::RunError;
use crate::semantic_graph;

/// Render one semantic graph query result.
pub(crate) fn render_graph_result(
    format: OutputFormat,
    result: &semantic_graph::GraphQueryResult<'_>,
) -> Result<String, RunError> {
    debug_assert!(result.nodes.iter().all(|node| !node.id.is_empty()));
    debug_assert!(result.edges.iter().all(|edge| !edge.from.is_empty()));
    if format.is_machine_readable() {
        let public = PublicGraph {
            schema: result.schema,
            root: &result.root,
            state: result.state,
            inspection: &result.inspection,
            nodes: PublicNodes(&result.nodes),
            edges: PublicEdges(&result.edges),
            aliases: PublicAliases(&result.aliases),
        };
        return serde_json::to_string_pretty(&public).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("semantic graph root: {}\n", result.root);
    append_state(&mut out, result.state);
    out.push_str(&format!(
        "configured targets: {}; actions: {}\n",
        result.inspection.configured.len(),
        result.inspection.actions.len()
    ));
    out.push_str("nodes:\n");
    for node in &result.nodes {
        out.push_str(&format!("  {} [{}]\n", node.id, node.kind));
    }
    out.push_str("edges:\n");
    for edge in &result.edges {
        out.push_str(&format!("  {} --{}--> {}\n", edge.from, edge.kind, edge.to));
    }
    if !result.aliases.is_empty() {
        out.push_str("aliases:\n");
        for alias in &result.aliases {
            out.push_str(&format!("  {} -> {}\n", alias.alias, alias.target));
        }
    }
    debug_assert!(out.contains("nodes:") && out.contains("edges:"));
    Ok(out)
}

/// Render one why query result.
pub(crate) fn render_why_result(
    format: OutputFormat,
    result: &semantic_graph::WhyResult<'_>,
) -> Result<String, RunError> {
    if format.is_machine_readable() {
        let public = PublicWhy {
            schema: result.schema,
            target: &result.target,
            state: result.state,
            node: public_node(result.node),
            producing_recipe: result.producing_recipe.map(public_node),
            sources: PublicNodes(&result.sources),
            providers: PublicNodes(&result.providers),
            sandboxes: PublicNodes(&result.sandboxes),
            proof_receipts: PublicNodes(&result.proof_receipts),
            witness_requests: PublicNodes(&result.witness_requests),
            release_evidence: PublicNodes(&result.release_evidence),
            configuration: result.configuration,
            action: result.action,
        };
        return serde_json::to_string_pretty(&public).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("why {}\nnode: {} [{}]\n", result.target, result.node.id, result.node.kind);
    append_state(&mut out, result.state);
    if let Some(configuration) = result.configuration {
        out.push_str(&format!(
            "configuration: {}\ntoolchain: {}\n",
            configuration.configuration.as_str(),
            configuration.toolchain.as_str()
        ));
    }
    if let Some(action) = result.action {
        out.push_str(&format!(
            "configured target: {}\naction facts: {}\n",
            action.configured_target,
            if action.is_complete() {
                "complete"
            } else {
                "unavailable fields remain"
            }
        ));
        // Typed facts contain only public declarations/digests and effective
        // policy, never argv, environment values, credentials or source text.
        out.push_str(&serde_json::to_string_pretty(action).map_err(|err| RunError::Internal(err.to_string()))?);
        out.push('\n');
    }
    if let Some(recipe) = result.producing_recipe {
        out.push_str(&format!("produced by: {}\n", recipe.id));
    }
    append_node_list(&mut out, "sources", &result.sources);
    append_node_list(&mut out, "providers", &result.providers);
    append_node_list(&mut out, "sandboxes", &result.sandboxes);
    append_node_list(&mut out, "proof receipts", &result.proof_receipts);
    append_node_list(&mut out, "witness requests", &result.witness_requests);
    append_node_list(&mut out, "release evidence", &result.release_evidence);
    debug_assert!(out.starts_with("why "));
    debug_assert!(out.contains(&result.node.id));
    Ok(out)
}

/// Render one dependents query result.
pub(crate) fn render_dependents_result(
    format: OutputFormat,
    result: &semantic_graph::DependentsResult<'_>,
) -> Result<String, RunError> {
    if format.is_machine_readable() {
        let public = PublicDependents {
            schema: result.schema,
            target: &result.target,
            state: result.state,
            dependents: PublicNodes(&result.dependents),
        };
        return serde_json::to_string_pretty(&public).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("dependents of {}:\n", result.target);
    append_state(&mut out, result.state);
    for dependent in &result.dependents {
        out.push_str(&format!("  {} [{}]\n", dependent.id, dependent.kind));
    }
    debug_assert!(out.starts_with("dependents of "));
    debug_assert!(out.contains(&result.target));
    Ok(out)
}

fn append_state(out: &mut String, state: semantic_graph::EvaluationState) {
    let state = match state {
        semantic_graph::EvaluationState::PreEvaluation => "pre-evaluation; evaluated facts unavailable",
        semantic_graph::EvaluationState::Partial => "partial; missing facts do not imply absence",
        semantic_graph::EvaluationState::Complete => "complete within saved graph declarations",
    };
    out.push_str(&format!("graph state: {state}\n"));
}

pub(crate) fn render_traversal_result(
    format: OutputFormat,
    result: &semantic_graph::TraversalResult,
) -> Result<String, RunError> {
    if format.is_machine_readable() {
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("directed {:?} of {}\n", result.direction, result.root);
    append_state(&mut out, result.state);
    if result.destination.is_some() {
        match &result.reachable.path {
            Some(path) => out.push_str(&format!("path: {}\n", path.join(" -> "))),
            None => out.push_str("no path in saved facts; absence is definitive only for a complete graph\n"),
        }
    } else {
        for identity in &result.reachable.identities {
            out.push_str(&format!("  {identity}\n"));
        }
    }
    Ok(out)
}

pub(crate) fn render_action_comparison(
    format: OutputFormat,
    result: &semantic_graph::ActionComparison,
) -> Result<String, RunError> {
    if format.is_machine_readable() {
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("action key: {:?}\nbefore: {}\nafter: {}\n", result.conclusion, result.before, result.after);
    out.push_str("before ");
    append_state(&mut out, result.before_state);
    out.push_str("after ");
    append_state(&mut out, result.after_state);
    for (fact, change) in &result.facts {
        out.push_str(&format!("  {fact}: {change:?}\n"));
    }
    for input in &result.changed_inputs {
        out.push_str(&format!("  changed input: {input}\n"));
    }
    out.push_str("known key inputs only; no cache lookup or execution was performed\n");
    Ok(out)
}

pub(crate) fn render_configuration_comparison(
    format: OutputFormat,
    result: &mantle_application_contract::inspection::ConfigurationComparison,
) -> Result<String, RunError> {
    if format.is_machine_readable() {
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!(
        "semantic target: {:?}\nconfiguration: {:?}\ntoolchain: {:?}\ndependencies: {:?}\n",
        result.semantic_target, result.configuration, result.toolchain, result.dependencies
    );
    out.push_str("before ");
    append_state(&mut out, result.before_state);
    out.push_str("after ");
    append_state(&mut out, result.after_state);
    for identity in &result.observed_dependencies_added {
        out.push_str(&format!("  observed dependency added: {identity}\n"));
    }
    for identity in &result.observed_dependencies_removed {
        out.push_str(&format!("  observed dependency removed: {identity}\n"));
    }
    Ok(out)
}

fn append_node_list(out: &mut String, label: &str, nodes: &[&semantic_graph::SemanticNode]) {
    if nodes.is_empty() {
        return;
    }
    out.push_str(&format!("{label}:\n"));
    for node in nodes {
        out.push_str(&format!("  {} [{}]\n", node.id, node.kind));
    }
    debug_assert!(out.contains(label));
    debug_assert!(nodes.iter().all(|node| !node.id.is_empty()));
}

// Persisted legacy metadata remains available to trusted receipt consumers, but
// command reports deliberately publish no free-form metadata. These borrowed
// projections stream serialization without cloning potentially sensitive maps.
#[derive(Serialize)]
struct PublicNode<'a> {
    id: &'a str,
    kind: &'a semantic_graph::SemanticNodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<&'a str>,
}
fn public_node(node: &semantic_graph::SemanticNode) -> PublicNode<'_> {
    PublicNode {
        id: &node.id,
        kind: &node.kind,
        digest: node.digest.as_deref(),
    }
}
struct PublicNodes<'a>(&'a [&'a semantic_graph::SemanticNode]);
impl Serialize for PublicNodes<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for node in self.0 {
            sequence.serialize_element(&public_node(node))?;
        }
        sequence.end()
    }
}
#[derive(Serialize)]
struct PublicEdge<'a> {
    from: &'a str,
    to: &'a str,
    kind: &'a semantic_graph::SemanticEdgeKind,
}
struct PublicEdges<'a>(&'a [&'a semantic_graph::SemanticEdge]);
impl Serialize for PublicEdges<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for edge in self.0 {
            sequence.serialize_element(&PublicEdge {
                from: &edge.from,
                to: &edge.to,
                kind: &edge.kind,
            })?;
        }
        sequence.end()
    }
}
#[derive(Serialize)]
struct PublicAlias<'a> {
    alias: &'a str,
    target: &'a str,
}
struct PublicAliases<'a>(&'a [&'a semantic_graph::SemanticAlias]);
impl Serialize for PublicAliases<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for alias in self.0 {
            sequence.serialize_element(&PublicAlias {
                alias: &alias.alias,
                target: &alias.target,
            })?;
        }
        sequence.end()
    }
}
#[derive(Serialize)]
struct PublicGraph<'a> {
    schema: &'a str,
    root: &'a str,
    state: semantic_graph::EvaluationState,
    inspection: &'a semantic_graph::InspectionView<'a>,
    nodes: PublicNodes<'a>,
    edges: PublicEdges<'a>,
    aliases: PublicAliases<'a>,
}
#[derive(Serialize)]
struct PublicDependents<'a> {
    schema: &'a str,
    target: &'a str,
    state: semantic_graph::EvaluationState,
    dependents: PublicNodes<'a>,
}
#[derive(Serialize)]
struct PublicWhy<'a> {
    schema: &'a str,
    target: &'a str,
    state: semantic_graph::EvaluationState,
    node: PublicNode<'a>,
    producing_recipe: Option<PublicNode<'a>>,
    sources: PublicNodes<'a>,
    providers: PublicNodes<'a>,
    sandboxes: PublicNodes<'a>,
    proof_receipts: PublicNodes<'a>,
    witness_requests: PublicNodes<'a>,
    release_evidence: PublicNodes<'a>,
    configuration: Option<&'a semantic_graph::Configuration>,
    action: Option<&'a semantic_graph::Action>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn node(id: &str) -> semantic_graph::SemanticNode {
        semantic_graph::SemanticNode {
            id: id.to_string(),
            kind: semantic_graph::SemanticNodeKind::Recipe,
            digest: None,
            metadata: BTreeMap::new(),
        }
    }

    fn graph_result<'a>(node: &'a semantic_graph::SemanticNode) -> semantic_graph::GraphQueryResult<'a> {
        semantic_graph::GraphQueryResult {
            schema: semantic_graph::SEMANTIC_GRAPH_SCHEMA,
            root: "root".to_string(),
            state: semantic_graph::EvaluationState::Partial,
            inspection: semantic_graph::InspectionView::default(),
            nodes: vec![node],
            edges: Vec::new(),
            aliases: Vec::new(),
        }
    }

    fn dependents_result<'a>(node: &'a semantic_graph::SemanticNode) -> semantic_graph::DependentsResult<'a> {
        semantic_graph::DependentsResult {
            schema: semantic_graph::SEMANTIC_GRAPH_SCHEMA,
            target: "root".to_string(),
            dependents: vec![node],
            state: semantic_graph::EvaluationState::Partial,
        }
    }

    fn why_result<'a>(node: &'a semantic_graph::SemanticNode) -> semantic_graph::WhyResult<'a> {
        semantic_graph::WhyResult {
            schema: semantic_graph::SEMANTIC_GRAPH_SCHEMA,
            target: "root".to_string(),
            node,
            producing_recipe: None,
            sources: Vec::new(),
            providers: Vec::new(),
            sandboxes: Vec::new(),
            proof_receipts: Vec::new(),
            witness_requests: Vec::new(),
            release_evidence: Vec::new(),
            state: semantic_graph::EvaluationState::Partial,
            configuration: None,
            action: None,
        }
    }

    #[test]
    fn the_graph_renderer_emits_machine_readable_json() {
        let node = node("recipe-a");
        let rendered = render_graph_result(OutputFormat::Json, &graph_result(&node)).expect("json renders");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("output is JSON");
        assert_eq!(parsed["root"], "root");
        assert_eq!(parsed["nodes"][0]["id"], "recipe-a");
        assert!(!rendered.contains("semantic graph root:"));
    }

    #[test]
    fn the_graph_renderer_emits_human_sections() {
        let node = node("recipe-a");
        let rendered = render_graph_result(OutputFormat::Human, &graph_result(&node)).expect("human renders");
        assert!(rendered.contains("semantic graph root: root"));
        assert!(rendered.contains("nodes:"));
        assert!(rendered.contains("recipe-a [recipe]"));
        assert!(serde_json::from_str::<serde_json::Value>(&rendered).is_err());
    }

    #[test]
    fn the_why_renderer_covers_both_formats() {
        let node = node("recipe-a");
        let json = render_why_result(OutputFormat::Json, &why_result(&node)).expect("json renders");
        assert_eq!(serde_json::from_str::<serde_json::Value>(&json).expect("output is JSON")["target"], "root");
        let human = render_why_result(OutputFormat::Human, &why_result(&node)).expect("human renders");
        assert!(human.contains("why root"));
        assert!(human.contains("node: recipe-a [recipe]"));
        assert!(!human.contains("sources:"), "empty node lists are omitted");
    }

    #[test]
    fn the_dependents_renderer_covers_both_formats() {
        let node = node("recipe-b");
        let json = render_dependents_result(OutputFormat::Json, &dependents_result(&node)).expect("json renders");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).expect("output is JSON")["dependents"][0]["id"],
            "recipe-b"
        );
        let human = render_dependents_result(OutputFormat::Human, &dependents_result(&node)).expect("human renders");
        assert!(human.contains("dependents of root:"));
        assert!(human.contains("recipe-b [recipe]"));
    }

    #[test]
    fn a_named_node_list_is_appended_with_its_label() {
        let node = node("source-a");
        let mut out = String::new();
        append_node_list(&mut out, "sources", &[&node]);
        assert!(out.contains("sources:"));
        assert!(out.contains("source-a [recipe]"));
    }

    #[test]
    fn inspection_never_echoes_free_form_legacy_metadata() {
        let mut node = node("recipe-public");
        node.metadata.insert("access_token".into(), "secret-node-value".into());
        let edge = semantic_graph::SemanticEdge {
            from: node.id.clone(),
            to: node.id.clone(),
            kind: semantic_graph::SemanticEdgeKind::UsesSource,
            metadata: BTreeMap::from([("source_contents".into(), "secret-edge-value".into())]),
        };
        let alias = semantic_graph::SemanticAlias {
            alias: "public".into(),
            target: node.id.clone(),
            metadata: BTreeMap::from([("environment".into(), "secret-alias-value".into())]),
        };
        let mut graph = graph_result(&node);
        graph.edges.push(&edge);
        graph.aliases.push(&alias);
        for format in [OutputFormat::Human, OutputFormat::Json] {
            let graph = render_graph_result(format, &graph).unwrap();
            let why = render_why_result(format, &why_result(&node)).unwrap();
            let dependents = render_dependents_result(format, &dependents_result(&node)).unwrap();
            for rendered in [graph, why, dependents] {
                assert!(rendered.contains("recipe-public"));
                assert!(!rendered.contains("secret-node-value"));
                assert!(!rendered.contains("secret-edge-value"));
                assert!(!rendered.contains("secret-alias-value"));
            }
        }
        assert_eq!(
            node.metadata["access_token"], "secret-node-value",
            "saved authority is not rewritten by inspection"
        );
    }
}
