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
pub const MAX_CONSTRUCTOR_PATH_LEN: u32 = 32;
pub const MAX_DIAGNOSTIC_TEXT_BYTES: u32 = 256;
pub const MAX_AUTHORING_RECORDS: u32 = 65_536;
pub const COMPOSED_IMPACT_RULE_VERSION: &str = "mantle-composed-impact-v1";

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

/// Explicit graph inspection layers, never conflating authoring with execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GraphLayer {
    Authoring,
    NormalizedRecipe,
    ConfiguredAction,
    ObservedExecution,
}

fn no_u32() -> Option<u32> {
    None
}
fn no_i32() -> Option<i32> {
    None
}
fn no_string() -> Option<String> {
    None
}
fn no_origin() -> Option<SourceOrigin> {
    None
}
fn no_output_set() -> Option<BTreeSet<String>> {
    None
}
fn empty_strings() -> Vec<String> {
    Vec::new()
}
fn empty_templates() -> BTreeMap<String, TemplateFact> {
    BTreeMap::new()
}
fn empty_instances() -> BTreeMap<String, TemplateInstance> {
    BTreeMap::new()
}
fn empty_references() -> Vec<RecipeReference> {
    Vec::new()
}
fn empty_selected_outputs() -> BTreeMap<String, SelectedOutputHandle> {
    BTreeMap::new()
}
fn empty_diagnostics() -> Vec<AuthoringDiagnostic> {
    Vec::new()
}
fn empty_authoring() -> AuthoringFacts {
    AuthoringFacts::default()
}
fn empty_observed_execution() -> BTreeMap<String, ObservedExecution> {
    BTreeMap::new()
}

/// Bounded source-origin location for authoring inspection only.
/// Excluded from canonical recipe and configured-target identities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceOrigin {
    pub file: String,
    #[serde(default = "no_u32", skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default = "no_u32", skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Reusable authoring template declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateFact {
    /// Presentation name only; never sufficient template identity.
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default = "no_origin", skip_serializing_if = "Option::is_none")]
    pub origin: Option<SourceOrigin>,
}

/// One concrete authoring template instantiation producing a canonical recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateInstance {
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    pub recipe_id: String,
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(default = "empty_strings", skip_serializing_if = "Vec::is_empty")]
    pub constructor_path: Vec<String>,
    #[serde(default = "no_origin", skip_serializing_if = "Option::is_none")]
    pub origin: Option<SourceOrigin>,
    /// Presentation metadata only; excluded from instance identity.
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl TemplateInstance {
    /// Canonical instance identity over template, constructor path, and root.
    /// Excludes presentation display names and line/column metadata.
    pub fn identity(&self) -> Result<String, String> {
        let preimage = (&self.template_id, &self.constructor_path, &self.root);
        identity(b"mantle-template-instance-v1\0", "instance", &preimage)
    }
}

/// One authoring edge referencing a canonical recipe from an instance or root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeReference {
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub referencer_instance: Option<String>,
    pub target_recipe: String,
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub handle_id: Option<String>,
    #[serde(default = "no_origin", skip_serializing_if = "Option::is_none")]
    pub origin: Option<SourceOrigin>,
}

/// Retained selected-output handle linking a consumer to an actual producer output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedOutputHandle {
    pub producer_recipe: String,
    pub output_name: String,
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub consumer_instance: Option<String>,
    #[serde(default = "no_origin", skip_serializing_if = "Option::is_none")]
    pub origin: Option<SourceOrigin>,
    #[serde(default = "no_output_set", skip_serializing_if = "Option::is_none")]
    pub declared_outputs: Option<BTreeSet<String>>,
}

impl SelectedOutputHandle {
    /// Stable identity of the producer-recipe and selected output-name pair.
    pub fn identity(&self) -> Result<String, String> {
        let preimage = (&self.producer_recipe, &self.output_name);
        identity(b"mantle-selected-output-v1\0", "select", &preimage)
    }

    /// Whether the selected output exists in known producer outputs (`None` when unknown).
    pub fn selection_valid(&self) -> Option<bool> {
        self.declared_outputs.as_ref().map(|outputs| outputs.contains(&self.output_name))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthoringDiagnosticKind {
    InvalidOutputSelection,
    ContractFailure,
    IdentityCollision,
    UnavailableProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringDiagnostic {
    pub kind: AuthoringDiagnosticKind,
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(default = "no_string", skip_serializing_if = "Option::is_none")]
    pub output_name: Option<String>,
    #[serde(default = "no_origin", skip_serializing_if = "Option::is_none")]
    pub origin: Option<SourceOrigin>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionOutcome {
    CacheHit,
    Executed,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedExecution {
    pub outcome: ExecutionOutcome,
    #[serde(default = "no_i32", skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default = "no_output_set", skip_serializing_if = "Option::is_none")]
    pub produced_outputs: Option<BTreeSet<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringFacts {
    #[serde(default = "empty_templates", skip_serializing_if = "BTreeMap::is_empty")]
    pub templates: BTreeMap<String, TemplateFact>,
    #[serde(default = "empty_instances", skip_serializing_if = "BTreeMap::is_empty")]
    pub instances: BTreeMap<String, TemplateInstance>,
    #[serde(default = "empty_references", skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<RecipeReference>,
    #[serde(default = "empty_selected_outputs", skip_serializing_if = "BTreeMap::is_empty")]
    pub selected_outputs: BTreeMap<String, SelectedOutputHandle>,
    #[serde(default = "empty_diagnostics", skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<AuthoringDiagnostic>,
}

impl AuthoringFacts {
    pub fn is_empty(&self) -> bool {
        let is_maps_empty = self.templates.is_empty() && self.instances.is_empty();
        let is_vecs_empty = self.references.is_empty() && self.diagnostics.is_empty();
        is_maps_empty && is_vecs_empty && self.selected_outputs.is_empty()
    }

    pub fn record_count(&self) -> Result<u32, String> {
        let total = self
            .templates
            .len()
            .saturating_add(self.instances.len())
            .saturating_add(self.references.len())
            .saturating_add(self.selected_outputs.len())
            .saturating_add(self.diagnostics.len());
        u32::try_from(total).map_err(|_| String::from("authoring record count exceeds u32"))
    }
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
    #[serde(default = "empty_authoring", skip_serializing_if = "AuthoringFacts::is_empty")]
    pub authoring: AuthoringFacts,
    #[serde(default = "empty_observed_execution", skip_serializing_if = "BTreeMap::is_empty")]
    pub observed_execution: BTreeMap<String, ObservedExecution>,
}

impl InspectionFacts {
    /// Whether v2 authoring or observed-execution facts are recorded.
    pub fn has_v2_facts(&self) -> bool {
        !self.authoring.is_empty() || !self.observed_execution.is_empty()
    }
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

    fn reverse_reachability_parents(
        &self,
        seeds: &BTreeSet<String>,
    ) -> Result<(BTreeMap<&'a str, &'a str>, u32), String> {
        if seeds.is_empty() {
            return Err(String::from("impact query requires at least one seed"));
        }
        let max_nodes = self.forward.len();
        let mut parents = BTreeMap::new();
        let mut queue = VecDeque::with_capacity(max_nodes);
        for seed in seeds {
            let canonical =
                self.forward.get_key_value(seed.as_str()).map(|(id, _)| *id).ok_or("missing query target")?;
            if parents.contains_key(canonical) {
                continue;
            }
            if parents.len() >= max_nodes {
                return Err(String::from("graph traversal exceeds node bound"));
            }
            parents.insert(canonical, canonical);
            queue.push_back((canonical, 0u32));
        }
        let mut steps = 0u32;
        while let Some((id, depth)) = queue.pop_front() {
            for next in &self.reverse[id] {
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
                if parents.len() >= max_nodes {
                    return Err(String::from("graph traversal exceeds node bound"));
                }
                parents.insert(*next, id);
                queue.push_back((*next, depth.saturating_add(1)));
            }
        }
        debug_assert!(!parents.is_empty());
        debug_assert!(steps <= self.limits.steps);
        Ok((parents, steps))
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

/// Optional execution-policy predicate for composed impact inspection.
/// Missing action policy facts never evaluate to `NotMatched` (always `Unavailable`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyFilter {
    #[serde(default = "no_bool", skip_serializing_if = "Option::is_none")]
    pub sandboxed: Option<bool>,
    #[serde(default = "no_network_policy", skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkPolicy>,
    #[serde(default = "no_bool", skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<bool>,
    #[serde(default = "no_bool", skip_serializing_if = "Option::is_none")]
    pub allow_remote: Option<bool>,
}

fn no_bool() -> Option<bool> {
    None
}

fn no_network_policy() -> Option<NetworkPolicy> {
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyMatch {
    Matched,
    NotMatched,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImpactOutcome {
    Complete,
    Partial,
    Unsupported,
    Exhausted,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImpactedAction {
    pub affected_node: String,
    pub action_id: Option<String>,
    pub configured_target: Option<String>,
    pub semantic_target: Option<String>,
    pub configuration: Option<Blake3Digest>,
    pub toolchain: Option<Blake3Digest>,
    pub role: Option<String>,
    pub policy: Option<ExecutionPolicy>,
    pub policy_match: PolicyMatch,
    pub is_action_complete: bool,
    pub supporting_path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComposedImpactResult {
    pub schema: &'static str,
    pub rule_version: &'static str,
    pub snapshot_digest: Blake3Digest,
    pub graph_state: EvaluationState,
    pub outcome: ImpactOutcome,
    pub limits: QueryLimits,
    pub seeds: BTreeSet<String>,
    pub impacted_nodes: Vec<String>,
    pub impacted_actions: Vec<ImpactedAction>,
    pub unavailable_nodes: Vec<String>,
    pub examined_edges: u32,
}

pub struct ComposedImpactInput<'a> {
    pub ids: &'a [&'a str],
    pub depends_on_edges: &'a [(&'a str, &'a str)],
    pub action_for_edges: &'a [(&'a str, &'a str)],
    pub configured_from_edges: &'a [(&'a str, &'a str)],
    pub inspection: &'a InspectionFacts,
    pub snapshot_digest: Blake3Digest,
    pub seeds: &'a BTreeSet<String>,
    pub policy_filter: Option<&'a PolicyFilter>,
    pub limits: QueryLimits,
}

/// Evaluate whether an optional `ExecutionPolicy` satisfies `PolicyFilter`.
/// Unknown (`None`) policy facts remain `Unavailable`, never `NotMatched`.
pub fn evaluate_policy_filter(policy: Option<&ExecutionPolicy>, filter: Option<&PolicyFilter>) -> PolicyMatch {
    let Some(policy) = policy else {
        return PolicyMatch::Unavailable;
    };
    let Some(filter) = filter else {
        return PolicyMatch::Matched;
    };
    let is_sandbox_ok = filter.sandboxed.is_none_or(|expected| policy.sandboxed == expected);
    let is_network_ok = filter.network.as_ref().is_none_or(|expected| &policy.network == expected);
    let is_cache_ok = filter.cache_read.is_none_or(|expected| policy.cache_read == expected);
    let is_remote_ok = filter.allow_remote.is_none_or(|expected| policy.allow_remote == expected);
    let is_all_ok = is_sandbox_ok && is_network_ok && is_cache_ok && is_remote_ok;
    debug_assert!(is_all_ok || !is_sandbox_ok || !is_network_ok || !is_cache_ok || !is_remote_ok);
    debug_assert!(!is_all_ok || (is_sandbox_ok && is_network_ok));
    if is_all_ok {
        PolicyMatch::Matched
    } else {
        PolicyMatch::NotMatched
    }
}

/// Bounded composed impact query joining reverse `DependsOn` reachability
/// with declared actions, configured targets, configurations, and policies.
pub fn composed_impact(input: &ComposedImpactInput<'_>) -> Result<ComposedImpactResult, String> {
    let index = DependencyIndex::new(input.ids, input.depends_on_edges, input.limits)?;
    let (parents, examined_edges) = index.reverse_reachability_parents(input.seeds)?;
    let paths = build_supporting_paths(&parents)?;
    let (impacted_actions, unavailable_nodes) = collect_impacted_actions(input, &paths)?;
    let impacted_nodes: Vec<String> = paths.keys().cloned().collect();
    let outcome = classify_impact_outcome(input.inspection.state, &unavailable_nodes, &impacted_actions);
    debug_assert_eq!(impacted_nodes.len(), paths.len());
    debug_assert!(examined_edges <= input.limits.steps);
    Ok(ComposedImpactResult {
        schema: "mantle-composed-impact-v1",
        rule_version: COMPOSED_IMPACT_RULE_VERSION,
        snapshot_digest: input.snapshot_digest.clone(),
        graph_state: input.inspection.state,
        outcome,
        limits: input.limits,
        seeds: input.seeds.clone(),
        impacted_nodes,
        impacted_actions,
        unavailable_nodes: unavailable_nodes.into_iter().collect(),
        examined_edges,
    })
}

fn collect_impacted_actions(
    input: &ComposedImpactInput<'_>,
    paths: &BTreeMap<String, Vec<String>>,
) -> Result<(Vec<ImpactedAction>, BTreeSet<String>), String> {
    let action_to_config = build_edge_map(input.action_for_edges);
    let config_to_semantic = build_edge_map(input.configured_from_edges);
    let max_impacted_actions = paths.len().saturating_mul(input.inspection.actions.len().max(1));
    let step_cap = usize::try_from(MAX_GRAPH_STEPS).map_err(|_| String::from("step bound exceeds usize"))?;
    let mut impacted_actions = Vec::with_capacity(max_impacted_actions.min(step_cap));
    let mut unavailable_nodes = BTreeSet::new();

    for (node_id, path) in paths {
        let mut is_node_resolved = false;
        if let Some(action) = input.inspection.actions.get(node_id) {
            is_node_resolved = true;
            if impacted_actions.len() >= max_impacted_actions {
                return Err(String::from("composed impact action count exceeds bound"));
            }
            impacted_actions.push(build_impacted_action_record(ImpactRecordContext {
                affected_node: node_id,
                action_id: node_id,
                action,
                input,
                action_to_config: &action_to_config,
                supporting_path: path,
            }));
        } else {
            let matching_actions = collect_actions_for_node(node_id, input, &action_to_config, &config_to_semantic);
            for (action_id, action) in matching_actions {
                is_node_resolved = true;
                if impacted_actions.len() >= max_impacted_actions {
                    return Err(String::from("composed impact action count exceeds bound"));
                }
                impacted_actions.push(build_impacted_action_record(ImpactRecordContext {
                    affected_node: node_id,
                    action_id,
                    action,
                    input,
                    action_to_config: &action_to_config,
                    supporting_path: path,
                }));
            }
        }
        if !is_node_resolved {
            unavailable_nodes.insert(node_id.clone());
        }
    }

    impacted_actions.sort_by(|left, right| {
        (&left.affected_node, &left.action_id, &left.configured_target).cmp(&(
            &right.affected_node,
            &right.action_id,
            &right.configured_target,
        ))
    });
    debug_assert!(impacted_actions.len() <= max_impacted_actions);
    debug_assert!(unavailable_nodes.len() <= paths.len());
    Ok((impacted_actions, unavailable_nodes))
}

fn build_supporting_paths(parents: &BTreeMap<&str, &str>) -> Result<BTreeMap<String, Vec<String>>, String> {
    let max_paths = parents.len();
    let mut paths = BTreeMap::new();
    for id in parents.keys() {
        let mut cursor = *id;
        let mut path = Vec::with_capacity(max_paths);
        path.push(cursor.to_string());
        while let Some(parent) = parents.get(cursor).copied() {
            if parent == cursor {
                break;
            }
            if path.len() >= max_paths {
                return Err(String::from("supporting path exceeds parent tree bound"));
            }
            path.push(parent.to_string());
            cursor = parent;
        }
        if paths.len() >= max_paths {
            return Err(String::from("supporting path count exceeds parent tree bound"));
        }
        paths.insert((*id).to_string(), path);
    }
    debug_assert_eq!(paths.len(), parents.len());
    debug_assert!(paths.values().all(|path| !path.is_empty()));
    Ok(paths)
}

fn build_edge_map<'a>(edges: &[(&'a str, &'a str)]) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    let mut map: BTreeMap<&'a str, BTreeSet<&'a str>> = BTreeMap::new();
    for (from, to) in edges {
        map.entry(*from).or_default().insert(*to);
    }
    debug_assert!(map.values().all(|targets| !targets.is_empty()));
    debug_assert!(map.len() <= edges.len());
    map
}

fn collect_actions_for_node<'a>(
    node_id: &str,
    input: &'a ComposedImpactInput<'a>,
    action_to_config: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    config_to_semantic: &BTreeMap<&'a str, BTreeSet<&'a str>>,
) -> Vec<(&'a str, &'a Action)> {
    let mut matched_configs = BTreeSet::new();
    if input.inspection.configured.contains_key(node_id) {
        matched_configs.insert(node_id);
    }
    for (cfg_id, config) in &input.inspection.configured {
        let is_semantic_match = config.semantic_target == node_id;
        let is_edge_match = config_to_semantic.get(cfg_id.as_str()).is_some_and(|targets| targets.contains(node_id));
        if is_semantic_match || is_edge_match {
            matched_configs.insert(cfg_id.as_str());
        }
    }
    let mut actions = Vec::with_capacity(input.inspection.actions.len());
    for (action_id, action) in &input.inspection.actions {
        let is_direct_cfg = matched_configs.contains(action.configured_target.as_str());
        let is_edge_cfg = action_to_config
            .get(action_id.as_str())
            .is_some_and(|targets| targets.iter().any(|cfg| matched_configs.contains(cfg)));
        if is_direct_cfg || is_edge_cfg {
            actions.push((action_id.as_str(), action));
        }
    }
    debug_assert!(actions.len() <= input.inspection.actions.len());
    debug_assert!(matched_configs.len() <= input.inspection.configured.len().saturating_add(1));
    actions
}

struct ImpactRecordContext<'a> {
    affected_node: &'a str,
    action_id: &'a str,
    action: &'a Action,
    input: &'a ComposedImpactInput<'a>,
    action_to_config: &'a BTreeMap<&'a str, BTreeSet<&'a str>>,
    supporting_path: &'a [String],
}

fn build_impacted_action_record(ctx: ImpactRecordContext<'_>) -> ImpactedAction {
    let configured_target = Some(ctx.action.configured_target.clone());
    let config_fact = ctx.input.inspection.configured.get(&ctx.action.configured_target).or_else(|| {
        ctx.action_to_config
            .get(ctx.action_id)
            .and_then(|cfgs| cfgs.iter().find_map(|cfg| ctx.input.inspection.configured.get(*cfg)))
    });
    let policy_match = evaluate_policy_filter(ctx.action.policy.as_ref(), ctx.input.policy_filter);
    debug_assert!(!ctx.action_id.is_empty());
    debug_assert!(!ctx.supporting_path.is_empty());
    ImpactedAction {
        affected_node: ctx.affected_node.to_string(),
        action_id: Some(ctx.action_id.to_string()),
        configured_target,
        semantic_target: config_fact.map(|cfg| cfg.semantic_target.clone()),
        configuration: config_fact.map(|cfg| cfg.configuration.clone()),
        toolchain: config_fact.map(|cfg| cfg.toolchain.clone()),
        role: Some(ctx.action.role.clone()),
        policy: ctx.action.policy.clone(),
        policy_match,
        is_action_complete: ctx.action.is_complete(),
        supporting_path: ctx.supporting_path.to_vec(),
    }
}

fn classify_impact_outcome(
    graph_state: EvaluationState,
    unavailable_nodes: &BTreeSet<String>,
    impacted_actions: &[ImpactedAction],
) -> ImpactOutcome {
    if graph_state != EvaluationState::Complete {
        return ImpactOutcome::Partial;
    }
    let has_incomplete_action = impacted_actions
        .iter()
        .any(|item| !item.is_action_complete || item.policy_match == PolicyMatch::Unavailable);
    let is_missing_facts = !unavailable_nodes.is_empty() || has_incomplete_action;
    debug_assert!(graph_state == EvaluationState::Complete);
    debug_assert!(is_missing_facts || (unavailable_nodes.is_empty() && !has_incomplete_action));
    if is_missing_facts {
        ImpactOutcome::Partial
    } else {
        ImpactOutcome::Complete
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

    #[test]
    fn template_instances_and_selected_outputs_exclude_display_names_from_identity() {
        let mut left = TemplateInstance {
            template_id: Some(String::from("template:branch")),
            recipe_id: String::from("recipe:canonical-branch"),
            root: Some(String::from("diamond-result")),
            constructor_path: alloc::vec![String::from("branch"), String::from("left")],
            origin: Some(SourceOrigin {
                file: String::from("examples/diamond-dependency.ncl"),
                line: Some(43),
                column: Some(12),
            }),
            display_name: Some(String::from("diamond-branch")),
        };
        let mut right = left.clone();
        right.constructor_path = alloc::vec![String::from("branch"), String::from("right")];
        // Even with identical display_name and identical canonical recipe_id,
        // two instances of one template have distinct instance identities.
        assert_ne!(left.identity().unwrap(), right.identity().unwrap());
        let before_rename = left.identity().unwrap();
        left.display_name = Some(String::from("renamed-left"));
        left.origin.as_mut().unwrap().line = Some(99);
        assert_eq!(before_rename, left.identity().unwrap());

        let valid_handle = SelectedOutputHandle {
            producer_recipe: String::from("recipe:example-sdk"),
            output_name: String::from("dev"),
            consumer_instance: Some(before_rename),
            origin: None,
            declared_outputs: Some(BTreeSet::from([String::from("out"), String::from("dev")])),
        };
        assert_eq!(valid_handle.selection_valid(), Some(true));
        let invalid_handle = SelectedOutputHandle {
            output_name: String::from("missing"),
            ..valid_handle.clone()
        };
        assert_eq!(invalid_handle.selection_valid(), Some(false));
        let unknown_handle = SelectedOutputHandle {
            declared_outputs: None,
            ..valid_handle
        };
        assert_eq!(unknown_handle.selection_valid(), None);
    }

    #[test]
    fn composed_impact_matches_reverse_traversal_and_preserves_unknowns() {
        let mut debug_cfg = config(b"debug");
        debug_cfg.semantic_target = String::from("recipe:diamond-left");
        let mut release_cfg = config(b"release");
        release_cfg.semantic_target = String::from("recipe:diamond-left");
        let debug_id = debug_cfg.identity().unwrap();
        let release_id = release_cfg.identity().unwrap();

        let compile_debug = action(&debug_cfg);
        let mut link_debug = action(&debug_cfg);
        link_debug.role = String::from("link");
        let mut compile_release = action(&release_cfg);
        compile_release.policy.as_mut().unwrap().cache_read = false;
        let mut missing_policy = action(&release_cfg);
        missing_policy.role = String::from("package");
        missing_policy.policy = None;

        let cd_id = compile_debug.identity().unwrap();
        let ld_id = link_debug.identity().unwrap();
        let cr_id = compile_release.identity().unwrap();
        let mp_id = missing_policy.identity().unwrap();

        let ids = [
            "recipe:diamond-shared",
            "recipe:diamond-left",
            "recipe:diamond-right",
            "recipe:diamond-result",
            "recipe:unrelated",
            debug_id.as_str(),
            release_id.as_str(),
            cd_id.as_str(),
            ld_id.as_str(),
            cr_id.as_str(),
            mp_id.as_str(),
        ];
        let depends_on = [
            ("recipe:diamond-result", "recipe:diamond-left"),
            ("recipe:diamond-result", "recipe:diamond-right"),
            ("recipe:diamond-left", "recipe:diamond-shared"),
            ("recipe:diamond-right", "recipe:diamond-shared"),
        ];
        let action_for = [
            (cd_id.as_str(), debug_id.as_str()),
            (ld_id.as_str(), debug_id.as_str()),
            (cr_id.as_str(), release_id.as_str()),
            (mp_id.as_str(), release_id.as_str()),
        ];
        let configured_from = [
            (debug_id.as_str(), "recipe:diamond-left"),
            (release_id.as_str(), "recipe:diamond-left"),
        ];
        let inspection = InspectionFacts {
            state: EvaluationState::Complete,
            configured: BTreeMap::from([(debug_id.clone(), debug_cfg), (release_id.clone(), release_cfg)]),
            actions: BTreeMap::from([
                (cd_id.clone(), compile_debug),
                (ld_id.clone(), link_debug),
                (cr_id.clone(), compile_release),
                (mp_id.clone(), missing_policy),
            ]),
            authoring: AuthoringFacts::default(),
            observed_execution: BTreeMap::new(),
        };
        let seeds = BTreeSet::from([String::from("recipe:diamond-shared")]);
        let filter = PolicyFilter {
            sandboxed: None,
            network: None,
            cache_read: Some(true),
            allow_remote: None,
        };
        let result = composed_impact(&ComposedImpactInput {
            ids: &ids,
            depends_on_edges: &depends_on,
            action_for_edges: &action_for,
            configured_from_edges: &configured_from,
            inspection: &inspection,
            snapshot_digest: digest(b"snap-1"),
            seeds: &seeds,
            policy_filter: Some(&filter),
            limits: QueryLimits::default(),
        })
        .unwrap();

        // Differential comparison against DependencyIndex::traverse baseline:
        let baseline = DependencyIndex::new(&ids, &depends_on, QueryLimits::default())
            .unwrap()
            .traverse("recipe:diamond-shared", Direction::ReverseDependencies, None)
            .unwrap();
        assert_eq!(result.impacted_nodes, baseline.identities);
        assert!(!result.impacted_nodes.contains(&String::from("recipe:unrelated")));
        // Missing policy and nodes without actions keep outcome Partial, never Complete.
        assert_eq!(result.outcome, ImpactOutcome::Partial);
        assert_eq!(result.impacted_actions.len(), 4);
        let mp_record =
            result.impacted_actions.iter().find(|r| r.action_id.as_deref() == Some(mp_id.as_str())).unwrap();
        assert_eq!(mp_record.policy_match, PolicyMatch::Unavailable);
        let cr_record =
            result.impacted_actions.iter().find(|r| r.action_id.as_deref() == Some(cr_id.as_str())).unwrap();
        assert_eq!(cr_record.policy_match, PolicyMatch::NotMatched);
        let cd_record =
            result.impacted_actions.iter().find(|r| r.action_id.as_deref() == Some(cd_id.as_str())).unwrap();
        assert_eq!(cd_record.policy_match, PolicyMatch::Matched);

        // Removing one diamond path still reaches diamond-result via the other;
        // removing both paths drops diamond-result.
        let one_cut = [
            ("recipe:diamond-result", "recipe:diamond-right"),
            ("recipe:diamond-left", "recipe:diamond-shared"),
            ("recipe:diamond-right", "recipe:diamond-shared"),
        ];
        let one_cut_res = composed_impact(&ComposedImpactInput {
            ids: &ids,
            depends_on_edges: &one_cut,
            action_for_edges: &action_for,
            configured_from_edges: &configured_from,
            inspection: &inspection,
            snapshot_digest: digest(b"snap-2"),
            seeds: &seeds,
            policy_filter: Some(&filter),
            limits: QueryLimits::default(),
        })
        .unwrap();
        assert!(one_cut_res.impacted_nodes.contains(&String::from("recipe:diamond-result")));
        assert_eq!(
            one_cut_res
                .impacted_actions
                .iter()
                .find(|r| r.action_id.as_deref() == Some(cd_id.as_str()))
                .unwrap()
                .supporting_path,
            ["recipe:diamond-left", "recipe:diamond-shared"]
        );

        let both_cut = [
            ("recipe:diamond-left", "recipe:diamond-shared"),
            ("recipe:diamond-right", "recipe:diamond-shared"),
        ];
        let both_cut_res = composed_impact(&ComposedImpactInput {
            ids: &ids,
            depends_on_edges: &both_cut,
            action_for_edges: &action_for,
            configured_from_edges: &configured_from,
            inspection: &inspection,
            snapshot_digest: digest(b"snap-3"),
            seeds: &seeds,
            policy_filter: Some(&filter),
            limits: QueryLimits::default(),
        })
        .unwrap();
        assert!(!both_cut_res.impacted_nodes.contains(&String::from("recipe:diamond-result")));
    }
}
