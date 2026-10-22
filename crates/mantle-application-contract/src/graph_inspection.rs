//! Pure, bounded inspection of saved semantic/configured/action facts.
//! Identities are domain-separated BLAKE3 over canonical typed records. No
//! command, environment value, credential, or source content is stored here.
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use mantle_rust_plan_core::Blake3Digest;
use mantle_rust_plan_core::domain_digest;
use serde::Deserialize;
use serde::Serialize;

/// Largest admitted serialized graph size in bytes (16 MiB).
pub const MAX_GRAPH_BYTES: u64 = 16_777_216;
pub const MAX_GRAPH_NODES: u32 = 65_536;
pub const MAX_GRAPH_EDGES: u32 = 262_144;
pub const MAX_GRAPH_STEPS: u32 = 1_048_576;
pub const MAX_GRAPH_DEPTH: u32 = 4096;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluationState {
    PreEvaluation,
    #[default]
    Partial,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub semantic_target: String,
    /// Canonical configuration digest, never the potentially secret values.
    pub configuration: Blake3Digest,
    pub toolchain: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkPolicy {
    Denied,
    Declared,
    Unrestricted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPolicy {
    pub sandboxed: bool,
    pub network: NetworkPolicy,
    pub allow_local: bool,
    pub allow_remote: bool,
    pub cache_read: bool,
    pub cache_write: bool,
    /// Identity of additional effective policy, not raw policy/environment.
    pub additional: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub configured_target: String,
    /// Public logical role; multiple actions per configured target are allowed.
    pub role: String,
    pub command: Option<Blake3Digest>,
    pub environment: Option<Blake3Digest>,
    /// Public logical input identity -> admitted content identity. None means
    /// unavailable; an empty map is a known empty declaration.
    pub inputs: Option<BTreeMap<String, Blake3Digest>>,
    /// Public declared output identities, not their source or output contents.
    pub outputs: Option<BTreeSet<String>>,
    pub policy: Option<ExecutionPolicy>,
}

fn identity<T: Serialize>(domain: &[u8], prefix: &str, value: &T) -> Result<String, String> {
    let digest = domain_digest(domain, value).map_err(|_| String::from("identity encoding failed"))?;
    Ok(format!("{prefix}:v1:{}", digest.as_str()))
}

pub fn semantic_identity(declaration: &Blake3Digest) -> Result<String, String> {
    identity(b"mantle-semantic-target-v1\0", "semantic", declaration)
}
impl Configuration {
    pub fn identity(&self) -> Result<String, String> {
        identity(b"mantle-configured-target-v1\0", "configured", self)
    }
}
impl Action {
    pub fn identity(&self) -> Result<String, String> {
        identity(b"mantle-concrete-action-v1\0", "action", self)
    }
    pub fn is_complete(&self) -> bool {
        self.command.is_some()
            && self.environment.is_some()
            && self.inputs.is_some()
            && self.outputs.is_some()
            && self.policy.is_some()
    }
}

/// Empty configured-target map used when saved facts omit the field.
fn empty_configured_targets() -> BTreeMap<String, Configuration> {
    BTreeMap::new()
}

/// Empty action map used when saved facts omit the field.
fn empty_actions() -> BTreeMap<String, Action> {
    BTreeMap::new()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectionFacts {
    #[serde(default)]
    pub state: EvaluationState,
    #[serde(default = "empty_configured_targets")]
    pub configured: BTreeMap<String, Configuration>,
    #[serde(default = "empty_actions")]
    pub actions: BTreeMap<String, Action>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryLimits {
    pub nodes: u32,
    pub edges: u32,
    pub steps: u32,
    pub depth: u32,
}
impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            nodes: MAX_GRAPH_NODES,
            edges: MAX_GRAPH_EDGES,
            steps: MAX_GRAPH_STEPS,
            depth: MAX_GRAPH_DEPTH,
        }
    }
}
impl QueryLimits {
    pub fn validate(self) -> Result<(), String> {
        let is_nodes_admissible = self.nodes > 0 && self.nodes <= MAX_GRAPH_NODES;
        let is_edges_admissible = self.edges > 0 && self.edges <= MAX_GRAPH_EDGES;
        let is_steps_admissible = self.steps > 0 && self.steps <= MAX_GRAPH_STEPS;
        let is_depth_admissible = self.depth > 0 && self.depth <= MAX_GRAPH_DEPTH;
        let is_shape_admissible = is_nodes_admissible && is_edges_admissible;
        let is_budget_admissible = is_steps_admissible && is_depth_admissible;
        let is_admissible = is_shape_admissible && is_budget_admissible;
        if !is_admissible {
            return Err(String::from("invalid graph query resource limits"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    Dependencies,
    ReverseDependencies,
    Connected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reachability {
    pub identities: Vec<String>,
    pub path: Option<Vec<String>>,
    pub examined_edges: u32,
}

/// Validated directed dependency DAG. Admission is bounded before indexing;
/// traversal charges each examined arc, including shared/visited successors.
pub struct DependencyIndex<'a> {
    forward: BTreeMap<&'a str, BTreeSet<&'a str>>,
    reverse: BTreeMap<&'a str, BTreeSet<&'a str>>,
    limits: QueryLimits,
}
impl<'a> DependencyIndex<'a> {
    pub fn new(ids: &[&'a str], edges: &[(&'a str, &'a str)], limits: QueryLimits) -> Result<Self, String> {
        let index = Self::provenance(ids, edges, limits)?;
        index.reject_cycles()?;
        Ok(index)
    }

    /// Provenance relationships are not dependencies and may form cycles.
    pub fn provenance(ids: &[&'a str], edges: &[(&'a str, &'a str)], limits: QueryLimits) -> Result<Self, String> {
        limits.validate()?;
        let node_count =
            u32::try_from(ids.len()).map_err(|_| String::from("graph admission exceeds node/edge bound"))?;
        let edge_count =
            u32::try_from(edges.len()).map_err(|_| String::from("graph admission exceeds node/edge bound"))?;
        let is_node_count_admissible = node_count <= limits.nodes;
        let is_edge_count_admissible = edge_count <= limits.edges;
        if !is_node_count_admissible || !is_edge_count_admissible {
            return Err(String::from("graph admission exceeds node/edge bound"));
        }
        let mut forward: BTreeMap<&'a str, BTreeSet<&'a str>> =
            ids.iter().copied().map(|id| (id, BTreeSet::new())).collect();
        let has_empty_id = ids.iter().any(|id| id.trim().is_empty());
        let has_duplicate_id = forward.len() != ids.len();
        if has_empty_id || has_duplicate_id {
            return Err(String::from("empty or duplicate graph identity"));
        }
        let mut reverse: BTreeMap<&'a str, BTreeSet<&'a str>> =
            forward.keys().map(|id| (*id, BTreeSet::new())).collect();
        for (from, to) in edges {
            let Some(targets) = forward.get_mut(from) else {
                return Err(String::from("dependency references missing target"));
            };
            targets.insert(*to);
            let Some(sources) = reverse.get_mut(to) else {
                return Err(String::from("dependency references missing target"));
            };
            sources.insert(*from);
        }
        Ok(Self {
            forward,
            reverse,
            limits,
        })
    }

    fn reject_cycles(&self) -> Result<(), String> {
        let mut indegree: BTreeMap<&str, usize> = self.reverse.iter().map(|(id, edges)| (*id, edges.len())).collect();
        let mut ready: BTreeSet<&str> =
            indegree.iter().filter_map(|(id, count)| (*count == 0).then_some(*id)).collect();
        let mut visited = 0u32;
        while let Some(id) = ready.pop_first() {
            visited += 1;
            for to in &self.forward[id] {
                let Some(count) = indegree.get_mut(to) else {
                    return Err(String::from("cyclic-invalid dependency graph"));
                };
                let remaining = count.saturating_sub(1);
                *count = remaining;
                if remaining == 0 {
                    ready.insert(to);
                }
            }
        }
        let total_nodes =
            u32::try_from(self.forward.len()).map_err(|_| String::from("cyclic-invalid dependency graph"))?;
        if visited != total_nodes {
            return Err(String::from("cyclic-invalid dependency graph"));
        }
        Ok(())
    }

    pub fn traverse(
        &self,
        root: &str,
        direction: Direction,
        destination: Option<&str>,
    ) -> Result<Reachability, String> {
        let root = self.forward.get_key_value(root).map(|(id, _)| *id).ok_or("missing query target")?;
        if let Some(target) = destination
            && !self.forward.contains_key(target)
        {
            return Err(String::from("missing path destination"));
        }
        let mut parents = BTreeMap::from([(root, root)]);
        let mut queue = VecDeque::from([(root, 0u32)]);
        let mut steps = 0u32;
        while let Some((id, depth)) = queue.pop_front() {
            if destination == Some(id) {
                break;
            }
            let adjacent = match direction {
                Direction::Dependencies => &self.forward[id],
                Direction::ReverseDependencies => &self.reverse[id],
                Direction::Connected => &self.forward[id],
            };
            let reverse = (direction == Direction::Connected).then_some(&self.reverse[id]);
            for next in adjacent.iter().chain(reverse.into_iter().flat_map(|set| set.iter())) {
                steps = steps.checked_add(1).ok_or("graph traversal step overflow")?;
                if steps > self.limits.steps {
                    return Err(String::from("graph traversal exceeds step bound"));
                }
                if parents.contains_key(next) {
                    continue;
                }
                if depth >= self.limits.depth {
                    return Err(String::from("graph traversal exceeds depth bound"));
                }
                parents.insert(*next, id);
                queue.push_back((*next, depth.saturating_add(1)));
            }
        }
        let path = destination.and_then(|target| {
            let mut id = *parents.get_key_value(target)?.0;
            let mut path = alloc::vec![id.to_string()];
            while id != root {
                id = parents[id];
                path.push(id.to_string());
            }
            path.reverse();
            Some(path)
        });
        Ok(Reachability {
            identities: parents.keys().map(|id| (*id).to_string()).collect(),
            path,
            examined_edges: steps,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FactChange {
    Changed,
    Unchanged,
    Unavailable,
}
fn compare<T: PartialEq>(before: Option<&T>, after: Option<&T>) -> FactChange {
    match (before, after) {
        (Some(a), Some(b)) if a == b => FactChange::Unchanged,
        (Some(_), Some(_)) => FactChange::Changed,
        _ => FactChange::Unavailable,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionComparison {
    pub schema: &'static str,
    pub before: String,
    pub after: String,
    /// Pure comparisons default to partial until the saved graph owner attaches
    /// the explicitly declared evaluation state of each snapshot.
    pub before_state: EvaluationState,
    pub after_state: EvaluationState,
    pub facts: BTreeMap<String, FactChange>,
    pub changed_inputs: Vec<String>,
    /// This explains key differences, never asserts an observed cache lookup.
    pub conclusion: FactChange,
}

/// Number of independently reported facts in one action comparison.
const ACTION_FACT_COUNT: usize = 10;

/// The independently observed facts of one action comparison.
fn action_facts(
    before: &Action,
    after: &Action,
    old: Option<&Configuration>,
    new: Option<&Configuration>,
) -> BTreeMap<String, FactChange> {
    let mut facts = BTreeMap::from([
        (String::from("command"), compare(before.command.as_ref(), after.command.as_ref())),
        (String::from("environment"), compare(before.environment.as_ref(), after.environment.as_ref())),
        (String::from("inputs"), compare(before.inputs.as_ref(), after.inputs.as_ref())),
        (String::from("outputs"), compare(before.outputs.as_ref(), after.outputs.as_ref())),
        (String::from("policy"), compare(before.policy.as_ref(), after.policy.as_ref())),
        (String::from("role"), compare(Some(&before.role), Some(&after.role))),
        (String::from("configuration"), compare(old.map(|c| &c.configuration), new.map(|c| &c.configuration))),
        (String::from("toolchain"), compare(old.map(|c| &c.toolchain), new.map(|c| &c.toolchain))),
        (
            String::from("semantic-target"),
            compare(old.map(|c| &c.semantic_target), new.map(|c| &c.semantic_target)),
        ),
    ]);
    // The configured identity can be known even when its preimage is unavailable.
    facts.insert(
        String::from("configured-target"),
        compare(Some(&before.configured_target), Some(&after.configured_target)),
    );
    debug_assert_eq!(facts.len(), ACTION_FACT_COUNT);
    facts
}

/// Declared input identities whose recorded content differs between snapshots.
fn changed_input_keys(before: &Action, after: &Action) -> Vec<String> {
    let (Some(a), Some(b)) = (&before.inputs, &after.inputs) else {
        return Vec::new();
    };
    a.keys()
        .chain(b.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| a.get(*key) != b.get(*key))
        .cloned()
        .collect()
}

/// The overall verdict implied by the individually observed facts.
fn comparison_conclusion(facts: &BTreeMap<String, FactChange>) -> FactChange {
    let is_changed = facts.values().any(|value| *value == FactChange::Changed);
    let is_unavailable = facts.values().any(|value| *value == FactChange::Unavailable);
    if is_changed {
        return FactChange::Changed;
    }
    if is_unavailable {
        return FactChange::Unavailable;
    }
    FactChange::Unchanged
}

pub fn compare_actions(
    before: &Action,
    after: &Action,
    old: Option<&Configuration>,
    new: Option<&Configuration>,
) -> Result<ActionComparison, String> {
    let facts = action_facts(before, after, old, new);
    let changed_inputs = changed_input_keys(before, after);
    let conclusion = comparison_conclusion(&facts);
    let is_unchanged = conclusion == FactChange::Unchanged;
    let is_all_facts_unchanged = facts.values().all(|value| *value == FactChange::Unchanged);
    debug_assert_eq!(is_unchanged, is_all_facts_unchanged);
    let is_inputs_changed = facts.get("inputs") == Some(&FactChange::Changed);
    debug_assert!(!is_inputs_changed || !changed_inputs.is_empty());
    Ok(ActionComparison {
        schema: "mantle-action-comparison-v1",
        before: before.identity()?,
        after: after.identity()?,
        before_state: EvaluationState::Partial,
        after_state: EvaluationState::Partial,
        facts,
        changed_inputs,
        conclusion,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigurationComparison {
    pub schema: &'static str,
    pub before: String,
    pub after: String,
    pub before_state: EvaluationState,
    pub after_state: EvaluationState,
    pub semantic_target: FactChange,
    pub configuration: FactChange,
    pub toolchain: FactChange,
    pub dependencies: FactChange,
    pub observed_dependencies_added: Vec<String>,
    pub observed_dependencies_removed: Vec<String>,
}

pub fn compare_configurations(
    before: &Configuration,
    after: &Configuration,
) -> Result<ConfigurationComparison, String> {
    Ok(ConfigurationComparison {
        schema: "mantle-configuration-comparison-v1",
        before: before.identity()?,
        after: after.identity()?,
        before_state: EvaluationState::Partial,
        after_state: EvaluationState::Partial,
        semantic_target: compare(Some(&before.semantic_target), Some(&after.semantic_target)),
        configuration: compare(Some(&before.configuration), Some(&after.configuration)),
        toolchain: compare(Some(&before.toolchain), Some(&after.toolchain)),
        dependencies: FactChange::Unavailable,
        observed_dependencies_added: Vec::new(),
        observed_dependencies_removed: Vec::new(),
    })
}

impl ConfigurationComparison {
    pub fn attach_dependencies(&mut self, before: &BTreeSet<String>, after: &BTreeSet<String>) {
        self.observed_dependencies_added = after.difference(before).cloned().collect();
        self.observed_dependencies_removed = before.difference(after).cloned().collect();
        self.dependencies =
            if self.before_state == EvaluationState::Complete && self.after_state == EvaluationState::Complete {
                compare(Some(before), Some(after))
            } else {
                FactChange::Unavailable
            };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: &[u8]) -> Blake3Digest {
        Blake3Digest::from_slice(value)
    }
    fn config(value: &[u8]) -> Configuration {
        Configuration {
            semantic_target: String::from("recipe:shared"),
            configuration: digest(value),
            toolchain: digest(b"toolchain"),
        }
    }
    fn action(config: &Configuration) -> Action {
        Action {
            configured_target: config.identity().unwrap(),
            role: String::from("compile"),
            command: Some(digest(b"command")),
            environment: Some(digest(b"environment")),
            inputs: Some(BTreeMap::from([(String::from("source"), digest(b"source"))])),
            outputs: Some(BTreeSet::from([String::from("object")])),
            policy: Some(ExecutionPolicy {
                sandboxed: true,
                network: NetworkPolicy::Denied,
                allow_local: true,
                allow_remote: false,
                cache_read: true,
                cache_write: true,
                additional: digest(b"policy"),
            }),
        }
    }

    #[test]
    fn shared_dependency_paths_are_directed_and_deterministic() {
        let ids = ["root", "b", "a", "shared", "unrelated"];
        let edges = [("root", "b"), ("b", "shared"), ("root", "a"), ("a", "shared")];
        let index = DependencyIndex::new(&ids, &edges, QueryLimits::default()).unwrap();
        let dependencies = index.traverse("root", Direction::Dependencies, None).unwrap();
        assert_eq!(dependencies.identities, ["a", "b", "root", "shared"]);
        assert_eq!(dependencies.examined_edges, 4);
        assert_eq!(index.traverse("root", Direction::Dependencies, Some("shared")).unwrap().path.unwrap(), [
            "root", "a", "shared"
        ]);
        assert_eq!(index.traverse("shared", Direction::ReverseDependencies, None).unwrap().identities, [
            "a", "b", "root", "shared"
        ]);
        assert!(index.traverse("shared", Direction::Dependencies, Some("root")).unwrap().path.is_none());
        assert!(index.traverse("root", Direction::Dependencies, Some("unrelated")).unwrap().path.is_none());
        let reordered = DependencyIndex::new(
            &["unrelated", "shared", "a", "b", "root"],
            &[("a", "shared"), ("root", "a"), ("b", "shared"), ("root", "b")],
            QueryLimits::default(),
        )
        .unwrap();
        assert_eq!(dependencies, reordered.traverse("root", Direction::Dependencies, None).unwrap());
    }

    #[test]
    fn cycles_and_missing_targets_are_invalid_not_empty_answers() {
        let limits = QueryLimits::default();
        assert!(DependencyIndex::new(&["a", "b"], &[("a", "b"), ("b", "a")], limits).is_err());
        assert!(DependencyIndex::new(&["a"], &[("a", "a")], limits).is_err());
        assert!(DependencyIndex::new(&["a"], &[("a", "missing")], limits).is_err());
        let index = DependencyIndex::new(&["a"], &[], limits).unwrap();
        assert!(index.traverse("missing", Direction::Dependencies, None).is_err());
        assert!(index.traverse("a", Direction::Dependencies, Some("missing")).is_err());
        assert_eq!(index.traverse("a", Direction::Dependencies, Some("a")).unwrap().path.unwrap(), ["a"]);
    }

    #[test]
    fn traversal_charges_shared_edges_and_depth_not_just_result_size() {
        let ids = ["a", "b", "c", "d"];
        let edges = [("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")];
        let index = DependencyIndex::new(&ids, &edges, QueryLimits {
            steps: 3,
            ..QueryLimits::default()
        })
        .unwrap();
        assert!(index.traverse("a", Direction::Dependencies, None).unwrap_err().contains("step bound"));
        let index = DependencyIndex::new(&ids, &edges, QueryLimits {
            depth: 1,
            ..QueryLimits::default()
        })
        .unwrap();
        assert!(index.traverse("a", Direction::Dependencies, None).unwrap_err().contains("depth bound"));
        assert!(
            DependencyIndex::new(&ids, &edges, QueryLimits {
                nodes: 3,
                ..QueryLimits::default()
            })
            .is_err()
        );
        assert!(
            DependencyIndex::new(&ids, &edges, QueryLimits {
                edges: 3,
                ..QueryLimits::default()
            })
            .is_err()
        );
        assert!(
            QueryLimits {
                steps: 0,
                ..QueryLimits::default()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn configurations_and_action_identities_do_not_collapse() {
        let debug = config(b"debug");
        let release = config(b"release");
        assert_ne!(debug.identity().unwrap(), release.identity().unwrap());
        let comparison = compare_configurations(&debug, &release).unwrap();
        assert_eq!(comparison.semantic_target, FactChange::Unchanged);
        assert_eq!(comparison.configuration, FactChange::Changed);
        assert_eq!(comparison.toolchain, FactChange::Unchanged);
        let a = action(&debug);
        let b = action(&release);
        assert_ne!(a.identity().unwrap(), b.identity().unwrap());
        let comparison = compare_actions(&a, &b, Some(&debug), Some(&release)).unwrap();
        assert_eq!(comparison.facts["configuration"], FactChange::Changed);
        assert_eq!(comparison.facts["inputs"], FactChange::Unchanged);
    }

    #[test]
    fn action_explanations_preserve_unknown_added_removed_and_policy_changes() {
        let config = config(b"debug");
        let a = action(&config);
        let mut b = a.clone();
        assert_eq!(compare_actions(&a, &b, Some(&config), Some(&config)).unwrap().conclusion, FactChange::Unchanged);
        b.inputs = None;
        let unknown = compare_actions(&a, &b, Some(&config), Some(&config)).unwrap();
        assert_eq!(unknown.facts["inputs"], FactChange::Unavailable);
        assert_eq!(unknown.conclusion, FactChange::Unavailable);
        assert!(unknown.changed_inputs.is_empty());
        b.inputs = Some(BTreeMap::from([(String::from("added"), digest(b"new"))]));
        b.policy.as_mut().unwrap().cache_read = false;
        let changed = compare_actions(&a, &b, Some(&config), Some(&config)).unwrap();
        assert_eq!(changed.changed_inputs, ["added", "source"]);
        assert_eq!(changed.facts["policy"], FactChange::Changed);
        assert_eq!(changed.conclusion, FactChange::Changed);
        assert_ne!(changed.before, changed.after);
    }

    #[test]
    fn partial_dependency_equality_is_not_promoted_to_unchanged() {
        let config = config(b"debug");
        let known = BTreeSet::from([String::from("shared")]);
        let mut comparison = compare_configurations(&config, &config).unwrap();
        comparison.attach_dependencies(&known, &known);
        assert_eq!(comparison.dependencies, FactChange::Unavailable);
        comparison.before_state = EvaluationState::Complete;
        comparison.after_state = EvaluationState::Complete;
        comparison.attach_dependencies(&known, &known);
        assert_eq!(comparison.dependencies, FactChange::Unchanged);
        comparison.attach_dependencies(&known, &BTreeSet::new());
        assert_eq!(comparison.dependencies, FactChange::Changed);
        assert_eq!(comparison.observed_dependencies_removed, ["shared"]);
    }
}
