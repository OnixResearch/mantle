use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::collections::btree_map::Entry;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::Composition;
use crate::CompositionEdge;
use crate::CompositionNode;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::is_count_above_bound;
use crate::digest::is_count_within_bound;

pub const COMPOSITION_PLAN_SCHEMA: &str = "mantle-wasm-component-composition-plan-v1";

const MAX_COMPOSITION_NODES: u32 = 256;
const MAX_COMPOSITION_EDGES: u32 = 1024;
const MAX_IMPORTS_PER_NODE: u32 = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionPlan {
    pub schema: String,
    pub composition: Composition,
    pub identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionValidation {
    pub plan: Option<CompositionPlan>,
    pub blockers: Vec<ComponentBlocker>,
}

pub fn validate_composition(mut composition: Composition) -> CompositionValidation {
    let mut blockers = Vec::new();
    validate_shape(&composition, &mut blockers);
    if is_count_above_bound(composition.nodes.len(), MAX_COMPOSITION_NODES)
        || is_count_above_bound(composition.edges.len(), MAX_COMPOSITION_EDGES)
    {
        return CompositionValidation { plan: None, blockers };
    }
    let nodes = node_index(&composition, &mut blockers);
    validate_edges(&composition, &nodes, &mut blockers);
    validate_required_imports(&composition, &mut blockers);
    if blockers.is_empty() && graph_has_cycle(&composition) {
        blockers.push(blocker(
            "composition-cycle",
            &composition.package,
            "composition dependency graph contains a cycle",
        ));
    }
    if !blockers.is_empty() {
        return CompositionValidation { plan: None, blockers };
    }

    normalize(&mut composition);
    let identity = match canonical_identity(composition.clone()) {
        Ok(identity) => identity,
        Err(_) => {
            return CompositionValidation {
                plan: None,
                blockers: vec![blocker(
                    "composition-identity-failed",
                    &composition.package,
                    "composition plan could not be canonically identified",
                )],
            };
        }
    };
    debug_assert!(!composition.nodes.is_empty());
    debug_assert!(is_count_within_bound(composition.edges.len(), MAX_COMPOSITION_EDGES));
    CompositionValidation {
        plan: Some(CompositionPlan {
            schema: String::from(COMPOSITION_PLAN_SCHEMA),
            composition,
            identity_blake3: identity,
        }),
        blockers: Vec::new(),
    }
}

fn validate_shape(composition: &Composition, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if composition.package.is_empty() {
        blockers.push(blocker(
            "empty-composition-package",
            "composition.package",
            "composition package must not be empty",
        ));
    }
    if composition.source_wac.is_empty() {
        blockers.push(blocker("empty-wac-source", "composition.source_wac", "WAC source must not be empty"));
    }
    if composition.output_world.is_empty() {
        blockers.push(blocker(
            "empty-output-world",
            "composition.output_world",
            "composition output world must not be empty",
        ));
    }
    if composition.nodes.is_empty() || is_count_above_bound(composition.nodes.len(), MAX_COMPOSITION_NODES) {
        blockers.push(blocker(
            "composition-node-limit",
            "composition.nodes",
            "composition must contain a bounded non-empty node set",
        ));
    }
    if is_count_above_bound(composition.edges.len(), MAX_COMPOSITION_EDGES) {
        blockers.push(blocker(
            "composition-edge-limit",
            "composition.edges",
            "composition edge set exceeds its fixed bound",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn node_index(composition: &Composition, blockers: &mut Vec<ComponentBlocker>) -> BTreeMap<String, CompositionNode> {
    let mut nodes = BTreeMap::new();
    for node in &composition.nodes {
        validate_node(node, blockers);
        if nodes.insert(node.id.clone(), node.clone()).is_some() {
            blockers.push(blocker("duplicate-composition-node", &node.id, "composition node identifier is duplicated"));
        }
    }
    nodes
}

fn validate_node(node: &CompositionNode, blockers: &mut Vec<ComponentBlocker>) {
    let blocker_count_before = blockers.len();
    if node.id.is_empty() || node.package.is_empty() || node.world.is_empty() {
        blockers.push(blocker(
            "invalid-composition-node",
            &node.id,
            "composition node id, package, and world must be non-empty",
        ));
    }
    if !node.artifact.logical_path.starts_with('/') || node.artifact.size_bytes == 0 {
        blockers.push(blocker(
            "non-local-composition-artifact",
            &node.id,
            "composition node must bind a non-empty exact local store object",
        ));
    }
    if is_count_above_bound(node.required_imports.len(), MAX_IMPORTS_PER_NODE) {
        blockers.push(blocker(
            "composition-import-limit",
            &node.id,
            "composition node import set exceeds its fixed bound",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    }
    // This is a set of typed imports, not a numeric quantity.
    #[allow(tigerstyle::numeric_units)]
    let mut required_import_entries = BTreeSet::new();
    for required in &node.required_imports {
        if required.name.is_empty() || required.world.is_empty() {
            blockers.push(blocker(
                "invalid-required-import",
                &node.id,
                "required import name and world must be non-empty",
            ));
        }
        if !required_import_entries.insert(required.clone()) {
            blockers.push(blocker("duplicate-required-import", &node.id, "required composition import is duplicated"));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_edges(
    composition: &Composition,
    nodes: &BTreeMap<String, CompositionNode>,
    blockers: &mut Vec<ComponentBlocker>,
) {
    let blocker_count_before = blockers.len();
    let mut bindings = BTreeSet::new();
    for edge in &composition.edges {
        let (Some(provider), Some(consumer)) = (nodes.get(&edge.provider_node), nodes.get(&edge.consumer_node)) else {
            blockers.push(blocker(
                "missing-composition-dependency",
                &edge.consumer_node,
                "composition edge names a node without an exact local dependency",
            ));
            continue;
        };
        if edge.provider_node == edge.consumer_node {
            blockers.push(blocker(
                "self-composition-edge",
                &edge.consumer_node,
                "composition node cannot satisfy its own import",
            ));
        }
        let binding = (edge.consumer_node.clone(), edge.consumer_import.clone());
        if !bindings.insert(binding) {
            blockers.push(blocker(
                "duplicate-composition-binding",
                &edge.consumer_import,
                "composition import has more than one provider",
            ));
        }
        validate_edge_world(edge, provider, consumer, blockers);
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_edge_world(
    edge: &CompositionEdge,
    provider: &CompositionNode,
    consumer: &CompositionNode,
    blockers: &mut Vec<ComponentBlocker>,
) {
    let blocker_count_before = blockers.len();
    let required = consumer.required_imports.iter().find(|required| required.name == edge.consumer_import);
    let Some(required) = required else {
        blockers.push(blocker(
            "undeclared-composition-import",
            &edge.consumer_import,
            "composition edge binds an import not declared by the consumer",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return;
    };
    if edge.world != required.world || edge.world != provider.world {
        blockers.push(blocker(
            "wrong-world-composition-edge",
            &edge.consumer_import,
            "provider, consumer import, and edge worlds do not match",
        ));
    }
    if edge.provider_export.is_empty() {
        blockers.push(blocker(
            "empty-provider-export",
            &edge.provider_node,
            "composition provider export must not be empty",
        ));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
    debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
}

fn validate_required_imports(composition: &Composition, blockers: &mut Vec<ComponentBlocker>) {
    for node in &composition.nodes {
        for required in &node.required_imports {
            let binding_count = composition
                .edges
                .iter()
                .filter(|edge| edge.consumer_node == node.id && edge.consumer_import == required.name)
                .count();
            if binding_count == 0 {
                blockers.push(blocker(
                    "missing-composition-import",
                    &required.name,
                    "required composition import has no exact local binding",
                ));
            }
        }
    }
}

// BTree collections have no reserve API; the explicit node/edge guards below
// bound every insertion before either loop can execute.
#[allow(tigerstyle::unbounded_collection_growth)]
fn graph_has_cycle(composition: &Composition) -> bool {
    if is_count_above_bound(composition.nodes.len(), MAX_COMPOSITION_NODES)
        || is_count_above_bound(composition.edges.len(), MAX_COMPOSITION_EDGES)
    {
        return true;
    }
    let mut indegree = BTreeMap::<String, u32>::new();
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for node in &composition.nodes {
        indegree.insert(node.id.clone(), 0);
    }
    for edge in &composition.edges {
        match adjacency.entry(edge.provider_node.clone()) {
            Entry::Occupied(mut entry) => entry.get_mut().push(edge.consumer_node.clone()),
            Entry::Vacant(entry) => {
                entry.insert(vec![edge.consumer_node.clone()]);
            }
        }
        let degree = indegree.entry(edge.consumer_node.clone()).or_insert(0);
        *degree = degree.saturating_add(1);
    }
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter(|(_node, degree)| **degree == 0)
        .map(|(node, _degree)| node.clone())
        .collect();
    let mut processed = 0usize;
    while let Some(node) = ready.pop_first() {
        processed = processed.saturating_add(1);
        for consumer in adjacency.get(&node).into_iter().flatten() {
            let Some(degree) = indegree.get_mut(consumer) else {
                return true;
            };
            let Some(reduced_degree) = degree.checked_sub(1) else {
                return true;
            };
            *degree = reduced_degree;
            if *degree == 0 {
                ready.insert(consumer.clone());
            }
        }
    }
    debug_assert!(processed <= composition.nodes.len());
    debug_assert!(is_count_within_bound(indegree.len(), MAX_COMPOSITION_NODES));
    processed != composition.nodes.len()
}

fn normalize(composition: &mut Composition) {
    composition.nodes.sort_by(|left, right| left.id.cmp(&right.id));
    for node in &mut composition.nodes {
        node.required_imports.sort();
    }
    composition.edges.sort();
    debug_assert!(composition.nodes.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0].id <= pair[1].id));
    debug_assert!(composition.edges.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
}
