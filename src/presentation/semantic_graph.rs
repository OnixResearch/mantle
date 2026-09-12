//! Rendering for semantic-graph query results.
//!
//! The three renderers are pure: one typed result plus one output format in,
//! one rendered string out. The root keeps the CLI wiring and the exit codes.

use mantle_application_contract::OutputFormat;

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
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("semantic graph root: {}\n", result.root);
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
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("why {}\nnode: {} [{}]\n", result.target, result.node.id, result.node.kind);
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
        return serde_json::to_string_pretty(result).map_err(|err| RunError::Internal(err.to_string()));
    }
    let mut out = format!("dependents of {}:\n", result.target);
    for dependent in &result.dependents {
        out.push_str(&format!("  {} [{}]\n", dependent.id, dependent.kind));
    }
    debug_assert!(out.starts_with("dependents of "));
    debug_assert!(out.contains(&result.target));
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
}
