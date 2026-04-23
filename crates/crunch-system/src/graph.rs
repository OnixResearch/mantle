use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

use crate::ValidatedModule;
use crate::error::SystemConfigError;

const MAX_CHAIN_DEPTH_DEFAULT: usize = 256;
const MAX_PROVIDER_TYPES_PER_MODULE_DEFAULT: usize = 32;
const MAX_PROVIDER_EDGES_DEFAULT: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    pub module_names: Vec<String>,
    pub edges_from_dependency_to_consumer: BTreeMap<String, BTreeSet<String>>,
    pub provider_edges: Vec<ProviderEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEdge {
    pub producer_module: String,
    pub consumer_module: String,
    pub provider_type: String,
}

pub fn build_dependency_graph(modules: &[ValidatedModule]) -> Result<DependencyGraph, SystemConfigError> {
    build_dependency_graph_with_limits(
        modules,
        MAX_CHAIN_DEPTH_DEFAULT,
        MAX_PROVIDER_TYPES_PER_MODULE_DEFAULT,
        MAX_PROVIDER_EDGES_DEFAULT,
    )
}

pub fn build_dependency_graph_with_limits(
    modules: &[ValidatedModule],
    max_chain_depth: usize,
    max_provider_types_per_module: usize,
    max_provider_edges: usize,
) -> Result<DependencyGraph, SystemConfigError> {
    assert!(max_chain_depth > 0, "max chain depth must be positive");
    assert!(max_provider_types_per_module > 0, "max provider types per module must be positive");
    assert!(max_provider_edges > 0, "max provider edges must be positive");

    let module_by_name = index_modules(modules)?;
    let mut edges = initialize_edges(&module_by_name);
    add_input_edges(&module_by_name, &mut edges)?;
    let provider_edges = add_provider_edges(
        &module_by_name,
        &mut edges,
        max_provider_types_per_module,
        max_provider_edges,
    )?;
    enforce_chain_depth(&module_by_name, &edges, max_chain_depth)?;

    Ok(DependencyGraph {
        module_names: module_by_name.keys().cloned().collect(),
        edges_from_dependency_to_consumer: edges,
        provider_edges,
    })
}

pub fn topological_sort(graph: &DependencyGraph, modules: &[ValidatedModule]) -> Result<Vec<String>, SystemConfigError> {
    let module_priorities = module_priority_map(modules)?;
    let mut indegree = initialize_indegree(&graph.module_names);
    for consumers in graph.edges_from_dependency_to_consumer.values() {
        for consumer in consumers {
            let next = indegree.get(consumer).copied().unwrap_or(0).saturating_add(1);
            indegree.insert(consumer.clone(), next);
        }
    }

    let mut ready = sorted_ready_queue(&indegree, &module_priorities);
    let mut ordered = Vec::with_capacity(graph.module_names.len());
    let mut indegree_mut = indegree;

    while let Some(module_name) = ready.pop_front() {
        ordered.push(module_name.clone());
        if let Some(consumers) = graph.edges_from_dependency_to_consumer.get(&module_name) {
            for consumer in consumers {
                let current = indegree_mut.get(consumer).copied().unwrap_or(0);
                let next = current.saturating_sub(1);
                indegree_mut.insert(consumer.clone(), next);
                if next == 0 {
                    insert_ready(&mut ready, consumer.clone(), &module_priorities);
                }
            }
        }
    }

    if ordered.len() != graph.module_names.len() {
        return Err(cycle_error(graph, &indegree_mut));
    }

    Ok(ordered)
}

fn index_modules<'a>(modules: &'a [ValidatedModule]) -> Result<BTreeMap<String, &'a ValidatedModule>, SystemConfigError> {
    let mut module_by_name = BTreeMap::new();
    for module in modules {
        let replaced = module_by_name.insert(module.module_name.clone(), module);
        if replaced.is_some() {
            return Err(graph_error(
                format!("duplicate module '{}' in dependency graph", module.module_name),
                Some(module.module_name.clone()),
            ));
        }
    }
    Ok(module_by_name)
}

fn initialize_edges(module_by_name: &BTreeMap<String, &ValidatedModule>) -> BTreeMap<String, BTreeSet<String>> {
    module_by_name
        .keys()
        .cloned()
        .map(|name| (name, BTreeSet::new()))
        .collect()
}

fn add_input_edges(
    module_by_name: &BTreeMap<String, &ValidatedModule>,
    edges: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<(), SystemConfigError> {
    for module in module_by_name.values() {
        for dependency_name in &module.inputs {
            if !module_by_name.contains_key(dependency_name) {
                return Err(graph_error(
                    format!("module '{}' depends on unknown input '{}'", module.module_name, dependency_name),
                    Some(module.module_name.clone()),
                ));
            }
            edges.entry(dependency_name.clone()).or_default().insert(module.module_name.clone());
        }
    }
    Ok(())
}

fn add_provider_edges(
    module_by_name: &BTreeMap<String, &ValidatedModule>,
    edges: &mut BTreeMap<String, BTreeSet<String>>,
    max_provider_types_per_module: usize,
    max_provider_edges: usize,
) -> Result<Vec<ProviderEdge>, SystemConfigError> {
    let mut provider_edges = Vec::new();
    let mut producers_by_type = BTreeMap::<String, Vec<String>>::new();

    for module in module_by_name.values() {
        if module.consumes_providers.len() > max_provider_types_per_module {
            return Err(graph_error(
                format!(
                    "module '{}' consumes more than {} provider types",
                    module.module_name, max_provider_types_per_module
                ),
                Some(module.module_name.clone()),
            ));
        }
        if module.produces_providers.len() > max_provider_types_per_module {
            return Err(graph_error(
                format!(
                    "module '{}' produces more than {} provider types",
                    module.module_name, max_provider_types_per_module
                ),
                Some(module.module_name.clone()),
            ));
        }
        for provider_type in &module.produces_providers {
            producers_by_type
                .entry(provider_type.clone())
                .or_default()
                .push(module.module_name.clone());
        }
    }

    for module in module_by_name.values() {
        for provider_type in &module.consumes_providers {
            if let Some(producers) = producers_by_type.get(provider_type) {
                for producer_name in producers {
                    if provider_edges.len() >= max_provider_edges {
                        return Err(graph_error(
                            format!("provider edge count exceeds configured limit {max_provider_edges}"),
                            Some(module.module_name.clone()),
                        ));
                    }
                    edges.entry(producer_name.clone()).or_default().insert(module.module_name.clone());
                    provider_edges.push(ProviderEdge {
                        producer_module: producer_name.clone(),
                        consumer_module: module.module_name.clone(),
                        provider_type: provider_type.clone(),
                    });
                }
            }
        }
    }

    Ok(provider_edges)
}

fn enforce_chain_depth(
    module_by_name: &BTreeMap<String, &ValidatedModule>,
    edges: &BTreeMap<String, BTreeSet<String>>,
    max_chain_depth: usize,
) -> Result<(), SystemConfigError> {
    let mut indegree = initialize_indegree(&module_by_name.keys().cloned().collect::<Vec<_>>());
    for consumers in edges.values() {
        for consumer in consumers {
            let next = indegree.get(consumer).copied().unwrap_or(0).saturating_add(1);
            indegree.insert(consumer.clone(), next);
        }
    }
    let mut ready: VecDeque<String> = indegree
        .iter()
        .filter_map(|(name, degree)| if *degree == 0 { Some(name.clone()) } else { None })
        .collect();
    let mut longest = module_by_name
        .keys()
        .cloned()
        .map(|name| (name, 1usize))
        .collect::<BTreeMap<_, _>>();
    let mut seen_count = 0usize;

    while let Some(module_name) = ready.pop_front() {
        seen_count = seen_count.saturating_add(1);
        let current_depth = longest.get(&module_name).copied().unwrap_or(1);
        if current_depth > max_chain_depth {
            return Err(graph_error(
                format!("dependency chain depth exceeds configured limit {max_chain_depth}"),
                Some(module_name),
            ));
        }
        if let Some(consumers) = edges.get(&module_name) {
            for consumer in consumers {
                let next_depth = current_depth.saturating_add(1);
                let entry = longest.entry(consumer.clone()).or_insert(1);
                if next_depth > *entry {
                    *entry = next_depth;
                }
                let current = indegree.get(consumer).copied().unwrap_or(0);
                let next = current.saturating_sub(1);
                indegree.insert(consumer.clone(), next);
                if next == 0 {
                    ready.push_back(consumer.clone());
                }
            }
        }
    }

    if seen_count != module_by_name.len() {
        return Ok(());
    }
    Ok(())
}

fn module_priority_map(modules: &[ValidatedModule]) -> Result<BTreeMap<String, i64>, SystemConfigError> {
    let mut priorities = BTreeMap::new();
    for module in modules {
        let replaced = priorities.insert(module.module_name.clone(), module.priority);
        if replaced.is_some() {
            return Err(graph_error(
                format!("duplicate module '{}' in priority map", module.module_name),
                Some(module.module_name.clone()),
            ));
        }
    }
    Ok(priorities)
}

fn initialize_indegree(module_names: &[String]) -> BTreeMap<String, usize> {
    module_names.iter().cloned().map(|name| (name, 0usize)).collect()
}

fn sorted_ready_queue(
    indegree: &BTreeMap<String, usize>,
    module_priorities: &BTreeMap<String, i64>,
) -> VecDeque<String> {
    let mut ready: Vec<String> = indegree
        .iter()
        .filter_map(|(name, degree)| if *degree == 0 { Some(name.clone()) } else { None })
        .collect();
    ready.sort_by(|left, right| compare_ready(left, right, module_priorities));
    ready.into()
}

fn insert_ready(ready: &mut VecDeque<String>, module_name: String, module_priorities: &BTreeMap<String, i64>) {
    if ready.iter().any(|existing| existing == &module_name) {
        return;
    }
    let mut values: Vec<String> = ready.drain(..).collect();
    values.push(module_name);
    values.sort_by(|left, right| compare_ready(left, right, module_priorities));
    *ready = values.into();
}

fn compare_ready(left: &str, right: &str, module_priorities: &BTreeMap<String, i64>) -> std::cmp::Ordering {
    let left_priority = module_priorities.get(left).copied().unwrap_or_default();
    let right_priority = module_priorities.get(right).copied().unwrap_or_default();
    left_priority.cmp(&right_priority).then_with(|| left.cmp(right))
}

fn cycle_error(graph: &DependencyGraph, indegree: &BTreeMap<String, usize>) -> SystemConfigError {
    let remaining: Vec<String> = indegree
        .iter()
        .filter_map(|(name, degree)| if *degree > 0 { Some(name.clone()) } else { None })
        .collect();
    let provider_types: BTreeSet<String> = graph
        .provider_edges
        .iter()
        .filter_map(|edge| {
            if remaining.contains(&edge.producer_module) || remaining.contains(&edge.consumer_module) {
                Some(edge.provider_type.clone())
            } else {
                None
            }
        })
        .collect();
    graph_error(
        format!(
            "dependency cycle detected involving modules [{}] and provider types [{}]",
            remaining.join(", "),
            provider_types.into_iter().collect::<Vec<_>>().join(", ")
        ),
        None,
    )
}

fn graph_error(message: String, module_name: Option<String>) -> SystemConfigError {
    SystemConfigError::Eval {
        message,
        detail: None,
        machine_name: None,
        module_name,
        field_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(name: &str, inputs: &[&str], consumes: &[&str], produces: &[&str], priority: i64) -> ValidatedModule {
        ValidatedModule {
            module_name: name.to_string(),
            role_names: vec!["default".to_string()],
            inputs: inputs.iter().map(|value| (*value).to_string()).collect(),
            consumes_providers: consumes.iter().map(|value| (*value).to_string()).collect(),
            produces_providers: produces.iter().map(|value| (*value).to_string()).collect(),
            priority,
        }
    }

    #[test]
    fn linear_chain_orders_dependencies_before_consumers() {
        let modules = vec![
            module("e", &[], &[], &[], 0),
            module("d", &["e"], &[], &[], 0),
            module("c", &["d"], &[], &[], 0),
            module("b", &["c"], &[], &[], 0),
            module("a", &["b"], &[], &[], 0),
        ];
        let graph = build_dependency_graph(&modules).unwrap();
        let order = topological_sort(&graph, &modules).unwrap();

        assert_eq!(order, vec!["e", "d", "c", "b", "a"]);
    }

    #[test]
    fn diamond_dependency_keeps_leaf_before_parents() {
        let modules = vec![
            module("a", &[], &[], &[], 0),
            module("b", &["a"], &[], &[], 0),
            module("c", &["a"], &[], &[], 0),
            module("d", &["b", "c"], &[], &[], 0),
        ];
        let graph = build_dependency_graph(&modules).unwrap();
        let order = topological_sort(&graph, &modules).unwrap();

        assert_eq!(order.first().unwrap(), "a");
        assert_eq!(order.last().unwrap(), "d");
        let b_index = order.iter().position(|name| name == "b").unwrap();
        let c_index = order.iter().position(|name| name == "c").unwrap();
        let d_index = order.iter().position(|name| name == "d").unwrap();
        assert!(b_index < d_index);
        assert!(c_index < d_index);
    }

    #[test]
    fn provider_only_ordering_places_producer_first() {
        let modules = vec![
            module("consumer", &[], &["firewall"], &[], 0),
            module("producer", &[], &[], &["firewall"], 0),
        ];
        let graph = build_dependency_graph(&modules).unwrap();
        let order = topological_sort(&graph, &modules).unwrap();

        assert_eq!(order, vec!["producer", "consumer"]);
    }

    #[test]
    fn stable_tiebreak_prefers_lower_priority_number() {
        let modules = vec![module("b", &[], &[], &[], 20), module("a", &[], &[], &[], 10)];
        let graph = build_dependency_graph(&modules).unwrap();
        let order = topological_sort(&graph, &modules).unwrap();

        assert_eq!(order, vec!["a", "b"]);
    }

    #[test]
    fn stable_tiebreak_falls_back_to_alphabetical_for_equal_priority() {
        let modules = vec![module("b", &[], &[], &[], 10), module("a", &[], &[], &[], 10)];
        let graph = build_dependency_graph(&modules).unwrap();
        let order = topological_sort(&graph, &modules).unwrap();

        assert_eq!(order, vec!["a", "b"]);
    }

    #[test]
    fn cycle_detection_names_all_modules() {
        let modules = vec![
            module("a", &["c"], &[], &[], 0),
            module("b", &["a"], &[], &[], 0),
            module("c", &["b"], &[], &[], 0),
        ];
        let graph = build_dependency_graph(&modules).unwrap();
        let error = topological_sort(&graph, &modules).unwrap_err();

        let SystemConfigError::Eval { message, .. } = error else {
            panic!("expected eval error");
        };
        assert!(message.contains("a"));
        assert!(message.contains("b"));
        assert!(message.contains("c"));
    }

    #[test]
    fn provider_cycle_names_modules_and_provider_types() {
        let modules = vec![
            module("a", &[], &["q"], &["p"], 0),
            module("b", &[], &["p"], &["q"], 0),
        ];
        let graph = build_dependency_graph(&modules).unwrap();
        let error = topological_sort(&graph, &modules).unwrap_err();

        let SystemConfigError::Eval { message, .. } = error else {
            panic!("expected eval error");
        };
        assert!(message.contains("a"));
        assert!(message.contains("b"));
        assert!(message.contains("p"));
        assert!(message.contains("q"));
    }

    #[test]
    fn chain_depth_limit_exceeded() {
        let modules = vec![
            module("a", &["b"], &[], &[], 0),
            module("b", &["c"], &[], &[], 0),
            module("c", &[], &[], &[], 0),
        ];
        let error = build_dependency_graph_with_limits(&modules, 2, 32, 4096).unwrap_err();

        assert!(matches!(error, SystemConfigError::Eval { message, .. } if message.contains("chain depth")));
    }

    #[test]
    fn provider_edge_count_limit_exceeded() {
        let modules = vec![
            module("a", &[], &[], &["p"], 0),
            module("b", &[], &[], &["p"], 0),
            module("c", &[], &["p"], &[], 0),
        ];
        let error = build_dependency_graph_with_limits(&modules, 256, 32, 1).unwrap_err();

        assert!(matches!(error, SystemConfigError::Eval { message, .. } if message.contains("provider edge count")));
    }
}
