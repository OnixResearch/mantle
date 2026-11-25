// machine-artifact-public: semantic-graph.command-reports
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

pub use mantle_application_contract::inspection::Action;
pub use mantle_application_contract::inspection::ActionComparison;
pub use mantle_application_contract::inspection::AuthoringDiagnostic;
#[cfg(test)]
pub use mantle_application_contract::inspection::AuthoringDiagnosticKind;
#[cfg(test)]
pub use mantle_application_contract::inspection::AuthoringFacts;
#[cfg(test)]
pub use mantle_application_contract::inspection::ComposedImpactInput;
#[cfg(test)]
pub use mantle_application_contract::inspection::ComposedImpactResult;
pub use mantle_application_contract::inspection::Configuration;
pub use mantle_application_contract::inspection::DependencyIndex;
pub use mantle_application_contract::inspection::Direction;
pub use mantle_application_contract::inspection::EvaluationState;
pub use mantle_application_contract::inspection::ExecutionOutcome;
#[cfg(test)]
pub use mantle_application_contract::inspection::ImpactOutcome;
pub use mantle_application_contract::inspection::InspectionFacts;
pub use mantle_application_contract::inspection::MAX_AUTHORING_RECORDS;
pub use mantle_application_contract::inspection::MAX_CONSTRUCTOR_PATH_LEN;
pub use mantle_application_contract::inspection::MAX_DIAGNOSTIC_TEXT_BYTES;
use mantle_application_contract::inspection::MAX_GRAPH_BYTES;
pub use mantle_application_contract::inspection::ObservedExecution;
#[cfg(test)]
pub use mantle_application_contract::inspection::PolicyFilter;
pub use mantle_application_contract::inspection::QueryLimits;
pub use mantle_application_contract::inspection::Reachability;
pub use mantle_application_contract::inspection::RecipeReference;
pub use mantle_application_contract::inspection::SelectedOutputHandle;
pub use mantle_application_contract::inspection::SourceOrigin;
pub use mantle_application_contract::inspection::TemplateFact;
pub use mantle_application_contract::inspection::TemplateInstance;
pub use mantle_application_contract::inspection::compare_actions;
#[cfg(test)]
pub use mantle_application_contract::inspection::composed_impact;
use serde::Deserialize;
use serde::Serialize;

pub const SEMANTIC_GRAPH_SCHEMA: &str = "mantle-semantic-build-graph-v1";
pub const SEMANTIC_GRAPH_SCHEMA_V2: &str = "mantle-semantic-build-graph-v2";

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticNodeKind {
    SourceTree,
    Recipe,
    StoreOutput,
    SandboxProfile,
    Provider,
    ProofReceipt,
    WitnessRequest,
    ReleaseEvidence,
    NameAlias,
    ConfiguredTarget,
    Action,
}

impl fmt::Display for SemanticNodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::SourceTree => "source-tree",
            Self::Recipe => "recipe",
            Self::StoreOutput => "store-output",
            Self::SandboxProfile => "sandbox-profile",
            Self::Provider => "provider",
            Self::ProofReceipt => "proof-receipt",
            Self::WitnessRequest => "witness-request",
            Self::ReleaseEvidence => "release-evidence",
            Self::NameAlias => "name-alias",
            Self::ConfiguredTarget => "configured-target",
            Self::Action => "action",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticEdgeKind {
    DependsOn,
    ProducedBy,
    UsesSource,
    UsesSandbox,
    UsesProvider,
    SupportsProof,
    RequestsWitness,
    ReleasesAs,
    AliasOf,
    ConfiguredFrom,
    ActionFor,
}

impl fmt::Display for SemanticEdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::DependsOn => "depends-on",
            Self::ProducedBy => "produced-by",
            Self::UsesSource => "uses-source",
            Self::UsesSandbox => "uses-sandbox",
            Self::UsesProvider => "uses-provider",
            Self::SupportsProof => "supports-proof",
            Self::RequestsWitness => "requests-witness",
            Self::ReleasesAs => "releases-as",
            Self::AliasOf => "alias-of",
            Self::ConfiguredFrom => "configured-from",
            Self::ActionFor => "action-for",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticNode {
    pub id: String,
    pub kind: SemanticNodeKind,
    #[serde(default = "no_semantic_digest", skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(default = "empty_semantic_metadata", skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticEdge {
    pub from: String,
    pub to: String,
    pub kind: SemanticEdgeKind,
    #[serde(default = "empty_semantic_metadata", skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticAlias {
    pub alias: String,
    pub target: String,
    #[serde(default = "empty_semantic_metadata", skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticGraph {
    pub schema: String,
    #[serde(default = "empty_semantic_nodes")]
    pub nodes: Vec<SemanticNode>,
    #[serde(default = "empty_semantic_edges")]
    pub edges: Vec<SemanticEdge>,
    #[serde(default = "empty_semantic_aliases")]
    pub aliases: Vec<SemanticAlias>,
    /// Absent in legacy v1 files: absence means partial, never complete.
    #[serde(default)]
    pub inspection: InspectionFacts,
}

fn no_semantic_digest() -> Option<String> {
    None
}

fn empty_semantic_metadata() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn empty_semantic_nodes() -> Vec<SemanticNode> {
    Vec::new()
}

fn empty_semantic_edges() -> Vec<SemanticEdge> {
    Vec::new()
}

fn empty_semantic_aliases() -> Vec<SemanticAlias> {
    Vec::new()
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IncompleteGraphDiagnostic {
    pub schema: &'static str,
    pub query: String,
    pub target: String,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize)]
pub struct InspectionView<'a> {
    pub state: EvaluationState,
    pub configured: BTreeMap<&'a str, &'a Configuration>,
    pub actions: BTreeMap<&'a str, &'a Action>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub templates: BTreeMap<&'a str, &'a TemplateFact>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub instances: BTreeMap<&'a str, &'a TemplateInstance>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<&'a RecipeReference>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub selected_outputs: BTreeMap<&'a str, &'a SelectedOutputHandle>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<&'a AuthoringDiagnostic>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub observed_execution: BTreeMap<&'a str, &'a ObservedExecution>,
}

/// Explicit distinction between authoring, normalized recipes, configured
/// actions, and observed execution. Missing execution is never inferred.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize)]
pub struct LayerStatus {
    pub is_authoring_present: bool,
    pub is_normalized_recipe_present: bool,
    pub is_configured_action_present: bool,
    pub is_observed_execution_present: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct AuthoringInstanceView<'a> {
    pub instance_id: &'a str,
    pub fact: &'a TemplateInstance,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct AuthoringSelectedOutputView<'a> {
    pub handle_id: &'a str,
    pub producer_recipe: &'a str,
    pub output_name: &'a str,
    pub selection_valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<&'a SourceOrigin>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize)]
pub struct AuthoringContext<'a> {
    pub is_provenance_available: bool,
    pub instances: Vec<AuthoringInstanceView<'a>>,
    pub references: Vec<&'a RecipeReference>,
    pub selected_outputs: Vec<AuthoringSelectedOutputView<'a>>,
    pub diagnostics: Vec<&'a AuthoringDiagnostic>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct SelectedOutputExplanation<'a> {
    pub schema: &'static str,
    pub handle_id: String,
    pub producer_recipe: String,
    pub producer_node: Option<&'a SemanticNode>,
    pub output_name: String,
    pub selection_valid: Option<bool>,
    pub consumer_instance: Option<&'a TemplateInstance>,
    pub origin: Option<&'a SourceOrigin>,
    pub is_provenance_available: bool,
    pub diagnostics: Vec<&'a AuthoringDiagnostic>,
    pub state: EvaluationState,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct GraphQueryResult<'a> {
    pub schema: &'static str,
    pub root: String,
    pub state: EvaluationState,
    pub inspection: InspectionView<'a>,
    pub nodes: Vec<&'a SemanticNode>,
    pub edges: Vec<&'a SemanticEdge>,
    pub aliases: Vec<&'a SemanticAlias>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct DependentsResult<'a> {
    pub schema: &'static str,
    pub target: String,
    pub dependents: Vec<&'a SemanticNode>,
    pub state: EvaluationState,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct WhyResult<'a> {
    pub schema: &'static str,
    pub target: String,
    pub node: &'a SemanticNode,
    pub producing_recipe: Option<&'a SemanticNode>,
    pub sources: Vec<&'a SemanticNode>,
    pub providers: Vec<&'a SemanticNode>,
    pub sandboxes: Vec<&'a SemanticNode>,
    pub proof_receipts: Vec<&'a SemanticNode>,
    pub witness_requests: Vec<&'a SemanticNode>,
    pub release_evidence: Vec<&'a SemanticNode>,
    pub state: EvaluationState,
    pub configuration: Option<&'a Configuration>,
    pub action: Option<&'a Action>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layers: Option<LayerStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authoring: Option<AuthoringContext<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_execution: Option<&'a ObservedExecution>,
}

impl SemanticGraph {
    pub fn empty() -> Self {
        Self {
            schema: SEMANTIC_GRAPH_SCHEMA.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            aliases: Vec::new(),
            inspection: InspectionFacts::default(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, SemanticGraphError> {
        if !path.exists() {
            return Err(SemanticGraphError::Incomplete(IncompleteGraphDiagnostic {
                schema: "mantle-incomplete-semantic-graph-v1",
                query: "load".to_string(),
                target: path.display().to_string(),
                missing: vec!["semantic graph file".to_string()],
            }));
        }
        let file =
            fs::File::open(path).map_err(|err| SemanticGraphError::Io(path.display().to_string(), err.to_string()))?;
        let mut content = String::new();
        file.take(MAX_GRAPH_BYTES + 1)
            .read_to_string(&mut content)
            .map_err(|err| SemanticGraphError::Io(path.display().to_string(), err.to_string()))?;
        if content.len() as u64 > MAX_GRAPH_BYTES {
            return Err(SemanticGraphError::Invalid("graph read exceeds byte bound".to_string()));
        }
        let graph: Self = serde_json::from_str(&content).map_err(|err| SemanticGraphError::Parse(err.to_string()))?;
        graph.validate()?;
        Ok(graph)
    }

    pub fn save(&self, path: &Path) -> Result<(), SemanticGraphError> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| SemanticGraphError::Io(parent.display().to_string(), err.to_string()))?;
        }
        let rendered = serde_json::to_string_pretty(self).map_err(|err| SemanticGraphError::Parse(err.to_string()))?;
        fs::write(path, format!("{rendered}\n"))
            .map_err(|err| SemanticGraphError::Io(path.display().to_string(), err.to_string()))
    }

    pub fn validate(&self) -> Result<(), SemanticGraphError> {
        let limits = QueryLimits::default();
        if self.nodes.len() > limits.nodes as usize
            || self.edges.len() > limits.edges as usize
            || self.aliases.len() > limits.nodes as usize
        {
            return Err(SemanticGraphError::Invalid("graph exceeds admission bound".to_string()));
        }
        if self.inspection.configured.len() > limits.nodes as usize
            || self.inspection.actions.len() > limits.nodes as usize
        {
            return Err(SemanticGraphError::Invalid("graph exceeds inspection fact bound".to_string()));
        }
        if self.schema != SEMANTIC_GRAPH_SCHEMA && self.schema != SEMANTIC_GRAPH_SCHEMA_V2 {
            return Err(SemanticGraphError::Invalid(format!(
                "semantic graph schema is `{}`, expected `{}` or `{}`",
                self.schema, SEMANTIC_GRAPH_SCHEMA, SEMANTIC_GRAPH_SCHEMA_V2
            )));
        }
        if self.schema == SEMANTIC_GRAPH_SCHEMA && self.inspection.has_v2_facts() {
            return Err(SemanticGraphError::Invalid(format!(
                "v2 authoring or observed execution facts require schema `{SEMANTIC_GRAPH_SCHEMA_V2}`"
            )));
        }
        let mut ids = BTreeSet::new();
        for node in &self.nodes {
            if node.id.trim().is_empty() {
                return Err(SemanticGraphError::Invalid("semantic graph node id must not be empty".to_string()));
            }
            if !ids.insert(node.id.as_str()) {
                return Err(SemanticGraphError::Invalid(format!("duplicate semantic graph node `{}`", node.id)));
            }
        }
        for edge in &self.edges {
            if !ids.contains(edge.from.as_str()) || !ids.contains(edge.to.as_str()) {
                return Err(SemanticGraphError::Invalid(format!(
                    "semantic graph edge {} -> {} references missing node",
                    edge.from, edge.to
                )));
            }
        }
        let mut producers = BTreeSet::new();
        for edge in &self.edges {
            if edge.kind == SemanticEdgeKind::ProducedBy && !producers.insert(edge.from.as_str()) {
                return Err(SemanticGraphError::Invalid("ambiguous producing recipe".to_string()));
            }
        }
        let mut aliases = BTreeSet::new();
        for alias in &self.aliases {
            if alias.alias.trim().is_empty() || alias.target.trim().is_empty() {
                return Err(SemanticGraphError::Invalid("semantic graph aliases must not be empty".to_string()));
            }
            if ids.contains(alias.alias.as_str()) || !aliases.insert(alias.alias.as_str()) {
                return Err(SemanticGraphError::Invalid("ambiguous graph alias".to_string()));
            }
            if !ids.contains(alias.target.as_str()) {
                return Err(SemanticGraphError::Invalid(format!(
                    "semantic graph alias `{}` targets missing node `{}`",
                    alias.alias, alias.target
                )));
            }
        }
        self.dependency_index(QueryLimits::default())?;
        self.validate_inspection()?;
        Ok(())
    }

    pub fn admit_query_limits(&self, limits: QueryLimits) -> Result<(), SemanticGraphError> {
        limits.validate().map_err(SemanticGraphError::Invalid)?;
        if self.nodes.len() > limits.nodes as usize
            || self.aliases.len() > limits.nodes as usize
            || self.edges.len() > limits.edges as usize
        {
            return Err(SemanticGraphError::Invalid("graph exceeds requested admission bound".to_string()));
        }
        Ok(())
    }

    /// Resolve one query without copying its canonical node identity.
    pub fn canonical_identity(&self, query: &str) -> Option<&str> {
        if let Some(node) = self.nodes.iter().find(|node| node.id == query) {
            return Some(node.id.as_str());
        }
        self.aliases.iter().find(|alias| alias.alias == query).map(|alias| alias.target.as_str())
    }

    pub fn resolve_identity(&self, query: &str) -> Option<String> {
        self.canonical_identity(query).map(str::to_owned)
    }

    pub fn graph_for_root(&self, query: &str) -> Result<GraphQueryResult<'_>, SemanticGraphError> {
        self.graph_for_root_bounded(query, QueryLimits::default())
    }

    pub fn graph_for_root_bounded(
        &self,
        query: &str,
        limits: QueryLimits,
    ) -> Result<GraphQueryResult<'_>, SemanticGraphError> {
        self.admit_query_limits(limits)?;
        self.validate()?;
        let root = self.resolve_or_incomplete("graph", query)?;
        // Preserve saved-graph connected-component semantics, including provenance.
        let ids: Vec<_> = self.nodes.iter().map(|node| node.id.as_str()).collect();
        let edges: Vec<_> = self.edges.iter().map(|edge| (edge.from.as_str(), edge.to.as_str())).collect();
        // Provenance may contain legitimate non-dependency cycles. The directed
        // dependency DAG was validated above; connected traversal uses its own
        // visited set and does not interpret provenance as dependency edges.
        let reachable = self.connected_identities(&root, &ids, &edges, limits)?;
        let visited: BTreeSet<_> = reachable.into_iter().collect();
        let mut nodes: Vec<_> = self.nodes.iter().filter(|node| visited.contains(&node.id)).collect();
        let mut edges: Vec<_> = self
            .edges
            .iter()
            .filter(|edge| visited.contains(&edge.from) && visited.contains(&edge.to))
            .collect();
        let mut aliases: Vec<_> = self.aliases.iter().filter(|alias| visited.contains(&alias.target)).collect();
        nodes.sort_by_key(|node| &node.id);
        edges.sort_by(|a, b| (&a.from, &a.to, &a.kind, &a.metadata).cmp(&(&b.from, &b.to, &b.kind, &b.metadata)));
        aliases.sort_by(|a, b| (&a.alias, &a.target, &a.metadata).cmp(&(&b.alias, &b.target, &b.metadata)));
        let inspection = self.build_inspection_view(&visited);
        Ok(GraphQueryResult {
            schema: "mantle-semantic-build-graph-query-v2",
            root,
            state: self.inspection.state,
            inspection,
            nodes,
            edges,
            aliases,
        })
    }

    fn build_inspection_view(&self, visited: &BTreeSet<String>) -> InspectionView<'_> {
        let instances: BTreeMap<&str, &TemplateInstance> = self
            .inspection
            .authoring
            .instances
            .iter()
            .filter(|(_, inst)| visited.contains(&inst.recipe_id))
            .map(|(id, inst)| (id.as_str(), inst))
            .collect();
        let template_ids: BTreeSet<&str> = instances.values().filter_map(|inst| inst.template_id.as_deref()).collect();
        let templates = self
            .inspection
            .authoring
            .templates
            .iter()
            .filter(|(id, _)| template_ids.contains(id.as_str()))
            .map(|(id, fact)| (id.as_str(), fact))
            .collect();
        let references = self
            .inspection
            .authoring
            .references
            .iter()
            .filter(|reference| visited.contains(&reference.target_recipe))
            .collect();
        let selected_outputs = self
            .inspection
            .authoring
            .selected_outputs
            .iter()
            .filter(|(_, handle)| visited.contains(&handle.producer_recipe))
            .map(|(id, handle)| (id.as_str(), handle))
            .collect();
        let diagnostics = self
            .inspection
            .authoring
            .diagnostics
            .iter()
            .filter(|diag| {
                diag.recipe_id.as_ref().is_some_and(|id| visited.contains(id))
                    || diag.root.as_ref().is_some_and(|id| visited.contains(id))
                    || diag.instance_id.as_deref().is_some_and(|id| instances.contains_key(id))
            })
            .collect();
        let observed_execution = self
            .inspection
            .observed_execution
            .iter()
            .filter(|(id, _)| visited.contains(*id))
            .map(|(id, exec)| (id.as_str(), exec))
            .collect();
        InspectionView {
            state: self.inspection.state,
            configured: self
                .inspection
                .configured
                .iter()
                .filter(|(id, _)| visited.contains(*id))
                .map(|(id, fact)| (id.as_str(), fact))
                .collect(),
            actions: self
                .inspection
                .actions
                .iter()
                .filter(|(id, _)| visited.contains(*id))
                .map(|(id, fact)| (id.as_str(), fact))
                .collect(),
            templates,
            instances,
            references,
            selected_outputs,
            diagnostics,
            observed_execution,
        }
    }

    pub fn dependents(&self, query: &str) -> Result<DependentsResult<'_>, SemanticGraphError> {
        self.validate()?;
        let target = self.resolve_or_incomplete("dependents", query)?;
        let dependent_ids: BTreeSet<&str> = self
            .edges
            .iter()
            .filter(|edge| edge.to == target && edge.kind == SemanticEdgeKind::DependsOn)
            .map(|edge| edge.from.as_str())
            .collect();
        let mut dependents: Vec<_> =
            self.nodes.iter().filter(|node| dependent_ids.contains(node.id.as_str())).collect();
        dependents.sort_by_key(|node| &node.id);
        Ok(DependentsResult {
            schema: "mantle-semantic-build-dependents-v2",
            target,
            dependents,
            state: self.inspection.state,
        })
    }

    pub fn why(&self, query: &str) -> Result<WhyResult<'_>, SemanticGraphError> {
        self.validate()?;
        let target = self.resolve_or_incomplete("why", query)?;
        let node = self.node(&target).ok_or_else(|| {
            SemanticGraphError::Invalid(format!(
                "resolved semantic graph identity `{target}` has no corresponding node"
            ))
        })?;
        let outgoing = |kind: SemanticEdgeKind| -> Vec<&SemanticNode> {
            let mut nodes: Vec<_> = self
                .edges
                .iter()
                .filter(|edge| edge.from == target && edge.kind == kind)
                .filter_map(|edge| self.node(&edge.to))
                .collect();
            nodes.sort_by_key(|node| &node.id);
            nodes.dedup_by_key(|node| &node.id);
            nodes
        };
        let producing_recipe = self
            .edges
            .iter()
            .find(|edge| edge.from == target && edge.kind == SemanticEdgeKind::ProducedBy)
            .and_then(|edge| self.node(&edge.to));
        let is_v2 = self.schema == SEMANTIC_GRAPH_SCHEMA_V2 || self.inspection.has_v2_facts();
        let authoring = is_v2.then(|| self.authoring_context_for_target(&target, producing_recipe));
        let observed_execution = self.inspection.observed_execution.get(&target);
        let layers = is_v2.then(|| self.layer_status_for_target(&target, node, authoring.as_ref(), observed_execution));
        let why = WhyResult {
            schema: "mantle-semantic-build-why-v2",
            target: target.clone(),
            node,
            producing_recipe,
            sources: outgoing(SemanticEdgeKind::UsesSource),
            providers: outgoing(SemanticEdgeKind::UsesProvider),
            sandboxes: outgoing(SemanticEdgeKind::UsesSandbox),
            proof_receipts: outgoing(SemanticEdgeKind::SupportsProof),
            witness_requests: outgoing(SemanticEdgeKind::RequestsWitness),
            release_evidence: outgoing(SemanticEdgeKind::ReleasesAs),
            state: self.inspection.state,
            configuration: self.inspection.configured.get(&target),
            action: self.inspection.actions.get(&target),
            layers,
            authoring,
            observed_execution,
        };
        if node.kind == SemanticNodeKind::StoreOutput && why.producing_recipe.is_none() {
            return Err(SemanticGraphError::Incomplete(IncompleteGraphDiagnostic {
                schema: "mantle-incomplete-semantic-graph-v1",
                query: "why".to_string(),
                target: target.to_string(),
                missing: vec!["producing recipe edge".to_string()],
            }));
        }
        Ok(why)
    }

    fn authoring_context_for_target<'a>(
        &'a self,
        target: &str,
        producing_recipe: Option<&'a SemanticNode>,
    ) -> AuthoringContext<'a> {
        let recipe_key = producing_recipe.map_or(target, |recipe| recipe.id.as_str());
        let mut instances: Vec<AuthoringInstanceView<'a>> = self
            .inspection
            .authoring
            .instances
            .iter()
            .filter(|(_, inst)| inst.recipe_id == recipe_key || inst.recipe_id == target)
            .map(|(id, fact)| AuthoringInstanceView {
                instance_id: id.as_str(),
                fact,
            })
            .collect();
        instances.sort_by_key(|view| view.instance_id);
        let instance_ids: BTreeSet<&str> = instances.iter().map(|view| view.instance_id).collect();
        let mut references: Vec<&'a RecipeReference> = self
            .inspection
            .authoring
            .references
            .iter()
            .filter(|reference| {
                reference.target_recipe == recipe_key
                    || reference.target_recipe == target
                    || reference.referencer_instance.as_deref().is_some_and(|id| instance_ids.contains(id))
            })
            .collect();
        references.sort();
        let mut selected_outputs: Vec<AuthoringSelectedOutputView<'a>> = self
            .inspection
            .authoring
            .selected_outputs
            .iter()
            .filter(|(_, handle)| {
                handle.producer_recipe == recipe_key
                    || handle.producer_recipe == target
                    || handle.consumer_instance.as_deref().is_some_and(|id| instance_ids.contains(id))
            })
            .map(|(id, handle)| AuthoringSelectedOutputView {
                handle_id: id.as_str(),
                producer_recipe: handle.producer_recipe.as_str(),
                output_name: handle.output_name.as_str(),
                selection_valid: handle.selection_valid(),
                origin: handle.origin.as_ref(),
            })
            .collect();
        selected_outputs.sort_by_key(|view| view.handle_id);
        let diagnostics: Vec<&'a AuthoringDiagnostic> = self
            .inspection
            .authoring
            .diagnostics
            .iter()
            .filter(|diag| {
                diag.recipe_id.as_deref() == Some(recipe_key)
                    || diag.recipe_id.as_deref() == Some(target)
                    || diag.root.as_deref() == Some(target)
                    || diag.instance_id.as_deref().is_some_and(|id| instance_ids.contains(id))
            })
            .collect();
        let is_provenance_available = !instances.is_empty() || !references.is_empty() || !selected_outputs.is_empty();
        AuthoringContext {
            is_provenance_available,
            instances,
            references,
            selected_outputs,
            diagnostics,
        }
    }

    fn layer_status_for_target(
        &self,
        target: &str,
        node: &SemanticNode,
        authoring: Option<&AuthoringContext<'_>>,
        observed_execution: Option<&ObservedExecution>,
    ) -> LayerStatus {
        let is_authoring_present = authoring.is_some_and(|ctx| ctx.is_provenance_available);
        let is_normalized_recipe_present = node.kind == SemanticNodeKind::Recipe
            || self.edges.iter().any(|edge| {
                edge.from == target
                    && matches!(edge.kind, SemanticEdgeKind::ProducedBy | SemanticEdgeKind::ConfiguredFrom)
            });
        let is_configured_action_present = self.inspection.configured.contains_key(target)
            || self.inspection.actions.contains_key(target)
            || self.inspection.configured.values().any(|cfg| cfg.semantic_target == target);
        let is_observed_execution_present =
            observed_execution.is_some_and(|exec| exec.outcome != ExecutionOutcome::Unavailable);
        LayerStatus {
            is_authoring_present,
            is_normalized_recipe_present,
            is_configured_action_present,
            is_observed_execution_present,
        }
    }

    /// Explain a retained selected-output handle by handle identity, showing
    /// its actual producer, output name, consumer instance, and diagnostics.
    pub fn explain_selected_output(
        &self,
        handle_id: &str,
    ) -> Result<SelectedOutputExplanation<'_>, SemanticGraphError> {
        self.validate()?;
        let Some(handle) = self.inspection.authoring.selected_outputs.get(handle_id) else {
            return Err(SemanticGraphError::Incomplete(IncompleteGraphDiagnostic {
                schema: "mantle-incomplete-semantic-graph-v1",
                query: "explain-selected-output".to_string(),
                target: handle_id.to_string(),
                missing: vec!["selected-output handle".to_string()],
            }));
        };
        let consumer_instance =
            handle.consumer_instance.as_deref().and_then(|id| self.inspection.authoring.instances.get(id));
        let producer_node = self.node(&handle.producer_recipe);
        let diagnostics: Vec<_> = self
            .inspection
            .authoring
            .diagnostics
            .iter()
            .filter(|diag| {
                diag.recipe_id.as_deref() == Some(handle.producer_recipe.as_str())
                    || diag.output_name.as_deref() == Some(handle.output_name.as_str())
                    || (handle.consumer_instance.is_some() && diag.instance_id == handle.consumer_instance)
            })
            .collect();
        let is_provenance_available = handle.origin.is_some() || consumer_instance.is_some();
        Ok(SelectedOutputExplanation {
            schema: "mantle-selected-output-explanation-v1",
            handle_id: handle_id.to_string(),
            producer_recipe: handle.producer_recipe.clone(),
            producer_node,
            output_name: handle.output_name.clone(),
            selection_valid: handle.selection_valid(),
            consumer_instance,
            origin: handle.origin.as_ref(),
            is_provenance_available,
            diagnostics,
            state: self.inspection.state,
        })
    }

    /// Bounded composed impact query over saved `DependsOn` reverse reachability,
    /// joined with configured targets, configurations, and execution policies.
    #[cfg(test)]
    pub fn composed_impact(
        &self,
        seeds: &BTreeSet<String>,
        policy_filter: Option<&PolicyFilter>,
        limits: QueryLimits,
    ) -> Result<ComposedImpactResult, SemanticGraphError> {
        self.admit_query_limits(limits)?;
        self.validate()?;
        let mut resolved_seeds = BTreeSet::new();
        for seed in seeds {
            resolved_seeds.insert(self.resolve_or_incomplete("impact", seed)?);
        }
        let ids: Vec<_> = self.nodes.iter().map(|node| node.id.as_str()).collect();
        let depends_on_edges: Vec<_> = self
            .edges
            .iter()
            .filter(|edge| edge.kind == SemanticEdgeKind::DependsOn)
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        let action_for_edges: Vec<_> = self
            .edges
            .iter()
            .filter(|edge| edge.kind == SemanticEdgeKind::ActionFor)
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        let configured_from_edges: Vec<_> = self
            .edges
            .iter()
            .filter(|edge| edge.kind == SemanticEdgeKind::ConfiguredFrom)
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        let snapshot_digest = mantle_rust_plan_core::domain_digest(b"mantle-semantic-graph-snapshot-v1\0", self)
            .map_err(|err| SemanticGraphError::Invalid(format!("snapshot digest failed: {err}")))?;
        composed_impact(&ComposedImpactInput {
            ids: &ids,
            depends_on_edges: &depends_on_edges,
            action_for_edges: &action_for_edges,
            configured_from_edges: &configured_from_edges,
            inspection: &self.inspection,
            snapshot_digest,
            seeds: &resolved_seeds,
            policy_filter,
            limits,
        })
        .map_err(SemanticGraphError::Invalid)
    }

    fn dependency_index(&self, limits: QueryLimits) -> Result<DependencyIndex<'_>, SemanticGraphError> {
        let ids: Vec<_> = self.nodes.iter().map(|node| node.id.as_str()).collect();
        let edges: Vec<_> = self
            .edges
            .iter()
            .filter(|edge| edge.kind == SemanticEdgeKind::DependsOn)
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        DependencyIndex::new(&ids, &edges, limits).map_err(SemanticGraphError::Invalid)
    }

    fn validate_inspection(&self) -> Result<(), SemanticGraphError> {
        let invalid = |message: &str| SemanticGraphError::Invalid(message.to_string());
        let nodes: BTreeMap<_, _> = self.nodes.iter().map(|node| (node.id.as_str(), &node.kind)).collect();
        let links: BTreeSet<_> =
            self.edges.iter().map(|edge| (edge.from.as_str(), edge.to.as_str(), &edge.kind)).collect();
        if self.inspection.state == EvaluationState::PreEvaluation
            && (!self.inspection.configured.is_empty() || !self.inspection.actions.is_empty())
        {
            return Err(invalid("pre-evaluation graph contains evaluated facts"));
        }
        for (id, config) in &self.inspection.configured {
            if config.identity().map_err(SemanticGraphError::Invalid)? != *id
                || nodes.get(id.as_str()).is_none_or(|kind| **kind != SemanticNodeKind::ConfiguredTarget)
                || nodes
                    .get(config.semantic_target.as_str())
                    .is_none_or(|kind| matches!(**kind, SemanticNodeKind::ConfiguredTarget | SemanticNodeKind::Action))
                || !links.contains(&(id.as_str(), config.semantic_target.as_str(), &SemanticEdgeKind::ConfiguredFrom))
            {
                return Err(invalid("configured identity/preimage or semantic target is invalid"));
            }
        }
        for (id, action) in &self.inspection.actions {
            if action.identity().map_err(SemanticGraphError::Invalid)? != *id
                || nodes.get(id.as_str()).is_none_or(|kind| **kind != SemanticNodeKind::Action)
                || !self.inspection.configured.contains_key(&action.configured_target)
                || !links.contains(&(id.as_str(), action.configured_target.as_str(), &SemanticEdgeKind::ActionFor))
                || action.role.trim().is_empty()
            {
                return Err(invalid("action identity/preimage or configured target is invalid"));
            }
            if self.inspection.state == EvaluationState::Complete && !action.is_complete() {
                return Err(invalid("complete graph contains unavailable action facts"));
            }
        }
        for node in &self.nodes {
            if node.kind == SemanticNodeKind::ConfiguredTarget && !self.inspection.configured.contains_key(&node.id)
                || node.kind == SemanticNodeKind::Action && !self.inspection.actions.contains_key(&node.id)
            {
                return Err(invalid("configured/action node has no typed facts"));
            }
        }
        if self.inspection.state == EvaluationState::Complete {
            let configured_semantics: BTreeSet<_> =
                self.inspection.configured.values().map(|config| config.semantic_target.as_str()).collect();
            let action_targets: BTreeSet<_> =
                self.inspection.actions.values().map(|action| action.configured_target.as_str()).collect();
            if self
                .nodes
                .iter()
                .any(|node| node.kind == SemanticNodeKind::Recipe && !configured_semantics.contains(node.id.as_str()))
                || self.inspection.configured.keys().any(|id| !action_targets.contains(id.as_str()))
            {
                return Err(invalid("complete graph is missing configured targets or declared actions"));
            }
        }
        self.validate_authoring_and_execution(&nodes)
    }

    fn validate_authoring_and_execution(
        &self,
        nodes: &BTreeMap<&str, &SemanticNodeKind>,
    ) -> Result<(), SemanticGraphError> {
        let invalid = |message: &str| SemanticGraphError::Invalid(message.to_string());
        let authoring = &self.inspection.authoring;
        let record_count = authoring.record_count().map_err(SemanticGraphError::Invalid)?;
        if record_count > MAX_AUTHORING_RECORDS
            || self.inspection.observed_execution.len() > QueryLimits::default().nodes as usize
        {
            return Err(invalid("authoring or execution inspection exceeds bound"));
        }
        for (id, template) in &authoring.templates {
            if id.trim().is_empty()
                || template
                    .name
                    .as_ref()
                    .is_some_and(|n| n.trim().is_empty() || n.len() > MAX_DIAGNOSTIC_TEXT_BYTES as usize)
                || template
                    .origin
                    .as_ref()
                    .is_some_and(|o| o.file.trim().is_empty() || o.file.len() > MAX_DIAGNOSTIC_TEXT_BYTES as usize)
            {
                return Err(invalid("template fact identity or origin is invalid"));
            }
        }
        for (id, instance) in &authoring.instances {
            if instance.identity().map_err(SemanticGraphError::Invalid)? != *id
                || instance.recipe_id.trim().is_empty()
                || instance.constructor_path.len() > MAX_CONSTRUCTOR_PATH_LEN as usize
                || instance.constructor_path.iter().any(|segment| segment.trim().is_empty())
                || instance.template_id.as_ref().is_some_and(|tid| !authoring.templates.contains_key(tid))
                || nodes.get(instance.recipe_id.as_str()).is_some_and(|kind| **kind != SemanticNodeKind::Recipe)
            {
                return Err(invalid("template instance identity or target recipe is invalid"));
            }
        }
        for reference in &authoring.references {
            if reference.target_recipe.trim().is_empty()
                || !nodes.contains_key(reference.target_recipe.as_str())
                || reference.referencer_instance.as_ref().is_some_and(|id| !authoring.instances.contains_key(id))
            {
                return Err(invalid("recipe reference target or instance is invalid"));
            }
        }
        for (id, handle) in &authoring.selected_outputs {
            if handle.identity().map_err(SemanticGraphError::Invalid)? != *id
                || handle.producer_recipe.trim().is_empty()
                || handle.output_name.trim().is_empty()
                || !nodes.contains_key(handle.producer_recipe.as_str())
                || handle.consumer_instance.as_ref().is_some_and(|inst| !authoring.instances.contains_key(inst))
            {
                return Err(invalid("selected-output handle identity or producer is invalid"));
            }
        }
        for diagnostic in &authoring.diagnostics {
            if diagnostic.message.trim().is_empty() || diagnostic.message.len() > MAX_DIAGNOSTIC_TEXT_BYTES as usize {
                return Err(invalid("authoring diagnostic message is empty or exceeds bound"));
            }
        }
        for action_id in self.inspection.observed_execution.keys() {
            if !self.inspection.actions.contains_key(action_id) {
                return Err(invalid("observed execution references unknown action"));
            }
        }
        Ok(())
    }

    fn connected_identities(
        &self,
        root: &str,
        ids: &[&str],
        edges: &[(&str, &str)],
        limits: QueryLimits,
    ) -> Result<Vec<String>, SemanticGraphError> {
        let index = DependencyIndex::provenance(ids, edges, limits).map_err(SemanticGraphError::Invalid)?;
        Ok(index.traverse(root, Direction::Connected, None).map_err(SemanticGraphError::Invalid)?.identities)
    }

    /// Directed DependsOn traversal only; provenance edges keep their old meaning.
    pub fn traverse(
        &self,
        query: &str,
        direction: Direction,
        destination: Option<&str>,
        limits: QueryLimits,
    ) -> Result<TraversalResult, SemanticGraphError> {
        self.admit_query_limits(limits)?;
        self.validate()?;
        let root = self.resolve_or_incomplete("traverse", query)?;
        let destination = destination.map(|target| self.resolve_or_incomplete("path", target)).transpose()?;
        let reachable = self
            .dependency_index(limits)?
            .traverse(&root, direction, destination.as_deref())
            .map_err(SemanticGraphError::Invalid)?;
        Ok(TraversalResult {
            schema: "mantle-semantic-traversal-v1",
            state: self.inspection.state,
            root,
            destination,
            direction,
            reachable,
        })
    }

    /// Compare admitted action key inputs, not execution/cache lookup outcomes.
    pub fn compare_action(
        &self,
        before: &str,
        other: &Self,
        after: &str,
    ) -> Result<ActionComparison, SemanticGraphError> {
        self.validate()?;
        other.validate()?;
        let before = self.resolve_or_incomplete("compare-action", before)?;
        let after = other.resolve_or_incomplete("compare-action", after)?;
        let a = self
            .inspection
            .actions
            .get(&before)
            .ok_or_else(|| SemanticGraphError::Invalid("before target is not an action".to_string()))?;
        let b = other
            .inspection
            .actions
            .get(&after)
            .ok_or_else(|| SemanticGraphError::Invalid("after target is not an action".to_string()))?;
        let mut comparison = compare_actions(
            a,
            b,
            self.inspection.configured.get(&a.configured_target),
            other.inspection.configured.get(&b.configured_target),
        )
        .map_err(SemanticGraphError::Invalid)?;
        comparison.before_state = self.inspection.state;
        comparison.after_state = other.inspection.state;
        Ok(comparison)
    }

    pub fn compare_configuration(
        &self,
        before: &str,
        other: &Self,
        after: &str,
        limits: QueryLimits,
    ) -> Result<mantle_application_contract::inspection::ConfigurationComparison, SemanticGraphError> {
        self.admit_query_limits(limits)?;
        other.admit_query_limits(limits)?;
        self.validate()?;
        other.validate()?;
        let before = self.resolve_or_incomplete("compare-configuration", before)?;
        let after = other.resolve_or_incomplete("compare-configuration", after)?;
        let a = self
            .inspection
            .configured
            .get(&before)
            .ok_or_else(|| SemanticGraphError::Invalid("before target is not configured".to_string()))?;
        let b = other
            .inspection
            .configured
            .get(&after)
            .ok_or_else(|| SemanticGraphError::Invalid("after target is not configured".to_string()))?;
        let mut comparison = mantle_application_contract::inspection::compare_configurations(a, b)
            .map_err(SemanticGraphError::Invalid)?;
        comparison.before_state = self.inspection.state;
        comparison.after_state = other.inspection.state;
        let before_dependencies = self
            .dependency_index(limits)?
            .traverse(&before, Direction::Dependencies, None)
            .map_err(SemanticGraphError::Invalid)?
            .identities
            .into_iter()
            .filter(|id| id != &before)
            .collect();
        let after_dependencies = other
            .dependency_index(limits)?
            .traverse(&after, Direction::Dependencies, None)
            .map_err(SemanticGraphError::Invalid)?
            .identities
            .into_iter()
            .filter(|id| id != &after)
            .collect();
        comparison.attach_dependencies(&before_dependencies, &after_dependencies);
        Ok(comparison)
    }

    fn node(&self, id: &str) -> Option<&SemanticNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    fn resolve_or_incomplete(&self, query: &str, target: &str) -> Result<String, SemanticGraphError> {
        self.resolve_identity(target).ok_or_else(|| {
            SemanticGraphError::Incomplete(IncompleteGraphDiagnostic {
                schema: "mantle-incomplete-semantic-graph-v1",
                query: query.to_string(),
                target: target.to_string(),
                missing: vec!["node or alias".to_string()],
            })
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct TraversalResult {
    pub schema: &'static str,
    pub state: EvaluationState,
    pub root: String,
    pub destination: Option<String>,
    pub direction: Direction,
    pub reachable: Reachability,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SemanticGraphError {
    Io(String, String),
    Parse(String),
    Invalid(String),
    Incomplete(IncompleteGraphDiagnostic),
}

impl fmt::Display for SemanticGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(path, err) => write!(f, "semantic graph I/O error at {path}: {err}"),
            Self::Parse(err) => write!(f, "semantic graph parse error: {err}"),
            Self::Invalid(err) => write!(f, "invalid semantic graph: {err}"),
            Self::Incomplete(diag) => write!(
                f,
                "incomplete semantic graph for {} `{}`: missing {}",
                diag.query,
                diag.target,
                diag.missing.join(", ")
            ),
        }
    }
}

impl std::error::Error for SemanticGraphError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> SemanticGraph {
        let node = |id: &str, kind: SemanticNodeKind| SemanticNode {
            id: id.to_string(),
            kind,
            digest: Some(format!("blake3:{id}")),
            metadata: BTreeMap::new(),
        };
        SemanticGraph {
            schema: SEMANTIC_GRAPH_SCHEMA.to_string(),
            inspection: InspectionFacts::default(),
            nodes: vec![
                node("out:hello", SemanticNodeKind::StoreOutput),
                node("recipe:hello", SemanticNodeKind::Recipe),
                node("src:hello", SemanticNodeKind::SourceTree),
                node("sandbox:pure", SemanticNodeKind::SandboxProfile),
                node("provider:local", SemanticNodeKind::Provider),
                node("proof:det", SemanticNodeKind::ProofReceipt),
                node("witness:req", SemanticNodeKind::WitnessRequest),
                node("release:hello", SemanticNodeKind::ReleaseEvidence),
            ],
            edges: vec![
                SemanticEdge {
                    from: "out:hello".into(),
                    to: "recipe:hello".into(),
                    kind: SemanticEdgeKind::ProducedBy,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "out:hello".into(),
                    to: "src:hello".into(),
                    kind: SemanticEdgeKind::UsesSource,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "out:hello".into(),
                    to: "sandbox:pure".into(),
                    kind: SemanticEdgeKind::UsesSandbox,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "out:hello".into(),
                    to: "provider:local".into(),
                    kind: SemanticEdgeKind::UsesProvider,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "out:hello".into(),
                    to: "proof:det".into(),
                    kind: SemanticEdgeKind::SupportsProof,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "out:hello".into(),
                    to: "witness:req".into(),
                    kind: SemanticEdgeKind::RequestsWitness,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "out:hello".into(),
                    to: "release:hello".into(),
                    kind: SemanticEdgeKind::ReleasesAs,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "recipe:hello".into(),
                    to: "src:hello".into(),
                    kind: SemanticEdgeKind::DependsOn,
                    metadata: BTreeMap::new(),
                },
            ],
            aliases: vec![SemanticAlias {
                alias: "hello".to_string(),
                target: "out:hello".to_string(),
                metadata: BTreeMap::from([("label".to_string(), "hello".to_string())]),
            }],
        }
    }

    #[test]
    fn names_are_metadata_over_stable_identities() {
        let mut graph = sample_graph();
        let before = graph.nodes.iter().find(|node| node.id == "out:hello").unwrap().digest.clone();
        graph.aliases[0].alias = "renamed-hello".to_string();
        let after = graph.nodes.iter().find(|node| node.id == "out:hello").unwrap().digest.clone();
        assert_eq!(before, after);
        assert_eq!(graph.resolve_identity("renamed-hello").as_deref(), Some("out:hello"));
    }

    #[test]
    fn why_links_output_to_recipe_sources_provider_sandbox_and_proofs() {
        let graph = sample_graph();
        let why = graph.why("hello").unwrap();
        assert_eq!(why.producing_recipe.unwrap().id, "recipe:hello");
        assert_eq!(why.sources[0].id, "src:hello");
        assert_eq!(why.providers[0].id, "provider:local");
        assert_eq!(why.sandboxes[0].id, "sandbox:pure");
        assert_eq!(why.proof_receipts[0].id, "proof:det");
        assert_eq!(why.witness_requests[0].id, "witness:req");
        assert_eq!(why.release_evidence[0].id, "release:hello");
    }

    #[test]
    fn dependents_are_reported_by_identity() {
        let graph = sample_graph();
        let dependents = graph.dependents("src:hello").unwrap();
        assert_eq!(dependents.dependents.len(), 1);
        assert_eq!(dependents.dependents[0].id, "recipe:hello");
    }

    #[test]
    fn incomplete_graph_does_not_invent_edges() {
        let mut graph = sample_graph();
        graph.edges.retain(|edge| edge.kind != SemanticEdgeKind::ProducedBy);
        let err = graph.why("out:hello").unwrap_err();
        assert!(matches!(err, SemanticGraphError::Incomplete(_)));
        assert!(err.to_string().contains("producing recipe edge"));
    }

    fn configured_fixture() -> SemanticGraph {
        serde_json::from_str(include_str!("../tests/fixtures/configured-action-graph.json")).unwrap()
    }

    #[test]
    fn configured_fixture_validates_versioned_identity_preimages_and_directed_queries() {
        let graph = configured_fixture();
        graph.validate().unwrap();
        let root = graph.resolve_identity("app-debug").unwrap();
        let shared = graph.resolve_identity("shared-debug").unwrap();
        let path = graph
            .traverse("app-debug", Direction::Dependencies, Some("shared-debug"), QueryLimits::default())
            .unwrap();
        assert_eq!(path.reachable.path.unwrap(), [root.clone(), shared.clone()]);
        assert_eq!(path.state, EvaluationState::Complete);
        assert!(
            graph
                .traverse("shared-debug", Direction::Dependencies, Some("app-debug"), QueryLimits::default())
                .unwrap()
                .reachable
                .path
                .is_none()
        );
        let reverse = graph
            .traverse("shared-debug", Direction::ReverseDependencies, None, QueryLimits::default())
            .unwrap();
        assert_eq!(reverse.reachable.identities.into_iter().collect::<BTreeSet<_>>(), BTreeSet::from([root, shared]));
        let action = graph.compare_action("app-action-debug", &graph, "app-action-release").unwrap();
        assert_eq!(action.facts["configuration"], mantle_application_contract::inspection::FactChange::Changed);
        assert_eq!(action.facts["inputs"], mantle_application_contract::inspection::FactChange::Unchanged);
        let configs = graph.compare_configuration("app-debug", &graph, "app-release", QueryLimits::default()).unwrap();
        assert_eq!(configs.dependencies, mantle_application_contract::inspection::FactChange::Changed);
        assert_eq!(configs.observed_dependencies_added, [graph.resolve_identity("shared-release").unwrap()]);
    }

    #[test]
    fn legacy_state_is_partial_and_incomplete_action_facts_cannot_claim_complete() {
        let mut legacy = serde_json::to_value(sample_graph()).unwrap();
        legacy.as_object_mut().unwrap().remove("inspection");
        let legacy: SemanticGraph = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.graph_for_root("hello").unwrap().state, EvaluationState::Partial);

        let original = configured_fixture();
        let mut graph = original.clone();
        let old = graph.resolve_identity("app-action-debug").unwrap();
        let mut action = graph.inspection.actions.remove(&old).unwrap();
        action.inputs = None;
        let new = action.identity().unwrap();
        graph.inspection.actions.insert(new.clone(), action);
        for node in &mut graph.nodes {
            if node.id == old {
                node.id = new.clone();
            }
        }
        for edge in &mut graph.edges {
            if edge.from == old {
                edge.from = new.clone();
            }
            if edge.to == old {
                edge.to = new.clone();
            }
        }
        for alias in &mut graph.aliases {
            if alias.target == old {
                alias.target = new.clone();
            }
        }
        assert!(graph.validate().is_err());
        graph.inspection.state = EvaluationState::Partial;
        graph.validate().unwrap();
        let comparison = original.compare_action("app-action-debug", &graph, "app-action-debug").unwrap();
        assert_eq!(comparison.conclusion, mantle_application_contract::inspection::FactChange::Unavailable);
        assert_eq!(comparison.after_state, EvaluationState::Partial);
        graph.inspection.state = EvaluationState::PreEvaluation;
        assert!(graph.validate().is_err());
    }

    #[test]
    fn machine_output_is_independent_of_saved_order_and_legacy_component_semantics_remain() {
        let graph = configured_fixture();
        let before = serde_json::to_string(&graph.graph_for_root("app-debug").unwrap()).unwrap();
        let mut reversed = graph.clone();
        reversed.nodes.reverse();
        reversed.edges.reverse();
        reversed.aliases.reverse();
        let after = serde_json::to_string(&reversed.graph_for_root("app-debug").unwrap()).unwrap();
        assert_eq!(before, after);
        assert_eq!(graph.graph_for_root("app-debug").unwrap().nodes.len(), graph.nodes.len());
        let mut provenance = sample_graph();
        provenance.edges.push(SemanticEdge {
            from: "src:hello".into(),
            to: "recipe:hello".into(),
            kind: SemanticEdgeKind::UsesSource,
            metadata: BTreeMap::new(),
        });
        assert!(provenance.graph_for_root("hello").is_ok(), "provenance cycles are not dependency cycles");
        provenance.edges.last_mut().unwrap().kind = SemanticEdgeKind::DependsOn;
        assert!(provenance.validate().is_err(), "directed dependency cycles are invalid");
    }

    #[test]
    fn saved_graph_read_and_query_work_are_bounded_before_output() {
        let file = tempfile::NamedTempFile::new().unwrap();
        file.as_file().set_len(MAX_GRAPH_BYTES + 1).unwrap();
        assert!(SemanticGraph::load(file.path()).unwrap_err().to_string().contains("byte bound"));
        let graph = configured_fixture();
        assert!(
            graph
                .graph_for_root_bounded("app-debug", QueryLimits {
                    nodes: 2,
                    ..QueryLimits::default()
                })
                .is_err()
        );
        assert!(
            graph
                .graph_for_root_bounded("app-debug", QueryLimits {
                    steps: 1,
                    ..QueryLimits::default()
                })
                .is_err()
        );
        assert!(graph.traverse("missing", Direction::Dependencies, None, QueryLimits::default()).is_err());
        let mut dangling = graph.clone();
        dangling.edges[0].to = "missing".into();
        assert!(dangling.validate().is_err());
        let mut ambiguous = graph.clone();
        ambiguous.aliases.push(ambiguous.aliases[0].clone());
        assert!(ambiguous.validate().is_err());
    }

    fn diamond_v2_graph() -> SemanticGraph {
        let node = |id: &str, kind: SemanticNodeKind| SemanticNode {
            id: id.to_string(),
            kind,
            digest: Some(format!("blake3:{id}")),
            metadata: BTreeMap::new(),
        };
        let branch_template = TemplateFact {
            name: Some("branch".to_string()),
            origin: Some(SourceOrigin {
                file: "examples/diamond-dependency.ncl".to_string(),
                line: Some(14),
                column: Some(14),
            }),
        };
        let left_inst = TemplateInstance {
            template_id: Some("template:branch".to_string()),
            recipe_id: "recipe:diamond-left".to_string(),
            root: Some("recipe:diamond-result".to_string()),
            constructor_path: vec!["examples/diamond-dependency.ncl".into(), "branch".into(), "left".into()],
            origin: Some(SourceOrigin {
                file: "examples/diamond-dependency.ncl".into(),
                line: Some(43),
                column: Some(12),
            }),
            display_name: Some("diamond-branch".into()),
        };
        let right_inst = TemplateInstance {
            template_id: Some("template:branch".to_string()),
            recipe_id: "recipe:diamond-right".to_string(),
            root: Some("recipe:diamond-result".to_string()),
            constructor_path: vec![
                "examples/diamond-dependency.ncl".into(),
                "branch".into(),
                "right".into(),
            ],
            origin: Some(SourceOrigin {
                file: "examples/diamond-dependency.ncl".into(),
                line: Some(44),
                column: Some(13),
            }),
            display_name: Some("diamond-branch".into()),
        };
        let left_id = left_inst.identity().unwrap();
        let right_id = right_inst.identity().unwrap();
        let dev_handle = SelectedOutputHandle {
            producer_recipe: "recipe:example-sdk".to_string(),
            output_name: "dev".to_string(),
            consumer_instance: Some(left_id.clone()),
            origin: Some(SourceOrigin {
                file: "examples/selected-output.ncl".into(),
                line: Some(28),
                column: Some(13),
            }),
            declared_outputs: Some(BTreeSet::from(["out".into(), "dev".into()])),
        };
        let bad_handle = SelectedOutputHandle {
            producer_recipe: "recipe:example-sdk".to_string(),
            output_name: "nonexistent".to_string(),
            consumer_instance: Some(right_id.clone()),
            origin: Some(SourceOrigin {
                file: "examples/selected-output.ncl".into(),
                line: Some(29),
                column: Some(13),
            }),
            declared_outputs: Some(BTreeSet::from(["out".into(), "dev".into()])),
        };
        let dev_handle_id = dev_handle.identity().unwrap();
        let bad_handle_id = bad_handle.identity().unwrap();

        let authoring = AuthoringFacts {
            templates: BTreeMap::from([("template:branch".into(), branch_template)]),
            instances: BTreeMap::from([(left_id.clone(), left_inst), (right_id.clone(), right_inst)]),
            references: vec![
                RecipeReference {
                    referencer_instance: Some(left_id.clone()),
                    target_recipe: "recipe:diamond-shared".into(),
                    handle_id: None,
                    origin: Some(SourceOrigin {
                        file: "examples/diamond-dependency.ncl".into(),
                        line: Some(17),
                        column: Some(13),
                    }),
                },
                RecipeReference {
                    referencer_instance: Some(right_id.clone()),
                    target_recipe: "recipe:diamond-shared".into(),
                    handle_id: None,
                    origin: Some(SourceOrigin {
                        file: "examples/diamond-dependency.ncl".into(),
                        line: Some(17),
                        column: Some(13),
                    }),
                },
            ],
            selected_outputs: BTreeMap::from([(dev_handle_id, dev_handle), (bad_handle_id, bad_handle)]),
            diagnostics: vec![AuthoringDiagnostic {
                kind: AuthoringDiagnosticKind::InvalidOutputSelection,
                root: Some("recipe:diamond-result".into()),
                instance_id: Some(right_id),
                recipe_id: Some("recipe:example-sdk".into()),
                output_name: Some("nonexistent".into()),
                origin: Some(SourceOrigin {
                    file: "examples/selected-output.ncl".into(),
                    line: Some(29),
                    column: Some(13),
                }),
                message: "selected output `nonexistent` is not declared by `recipe:example-sdk`".into(),
            }],
        };

        SemanticGraph {
            schema: SEMANTIC_GRAPH_SCHEMA_V2.to_string(),
            nodes: vec![
                node("recipe:diamond-shared", SemanticNodeKind::Recipe),
                node("recipe:diamond-left", SemanticNodeKind::Recipe),
                node("recipe:diamond-right", SemanticNodeKind::Recipe),
                node("recipe:diamond-result", SemanticNodeKind::Recipe),
                node("recipe:example-sdk", SemanticNodeKind::Recipe),
                node("recipe:orphan", SemanticNodeKind::Recipe),
            ],
            edges: vec![
                SemanticEdge {
                    from: "recipe:diamond-result".into(),
                    to: "recipe:diamond-left".into(),
                    kind: SemanticEdgeKind::DependsOn,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "recipe:diamond-result".into(),
                    to: "recipe:diamond-right".into(),
                    kind: SemanticEdgeKind::DependsOn,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "recipe:diamond-left".into(),
                    to: "recipe:diamond-shared".into(),
                    kind: SemanticEdgeKind::DependsOn,
                    metadata: BTreeMap::new(),
                },
                SemanticEdge {
                    from: "recipe:diamond-right".into(),
                    to: "recipe:diamond-shared".into(),
                    kind: SemanticEdgeKind::DependsOn,
                    metadata: BTreeMap::new(),
                },
            ],
            aliases: Vec::new(),
            inspection: InspectionFacts {
                state: EvaluationState::Partial,
                configured: BTreeMap::new(),
                actions: BTreeMap::new(),
                authoring,
                observed_execution: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn v2_graph_distinguishes_template_instances_references_selected_outputs_and_layers() {
        let mut graph = diamond_v2_graph();
        graph.validate().unwrap();

        // v1 schema must reject v2 authoring/execution facts.
        let mut wrong_schema = graph.clone();
        wrong_schema.schema = SEMANTIC_GRAPH_SCHEMA.to_string();
        assert!(wrong_schema.validate().is_err());

        // Shared canonical recipe shows 2 references from distinct instances.
        let shared_why = graph.why("recipe:diamond-shared").unwrap();
        let shared_auth = shared_why.authoring.as_ref().unwrap();
        assert!(shared_auth.is_provenance_available);
        assert_eq!(shared_auth.references.len(), 2);
        assert_ne!(shared_auth.references[0].referencer_instance, shared_auth.references[1].referencer_instance);

        // Two instances of one template have distinct instance IDs even with identical display_name.
        let left_why = graph.why("recipe:diamond-left").unwrap();
        let right_why = graph.why("recipe:diamond-right").unwrap();
        assert_ne!(
            left_why.authoring.as_ref().unwrap().instances[0].instance_id,
            right_why.authoring.as_ref().unwrap().instances[0].instance_id
        );

        // Selected-output handle reports actual producer, output name, validity, and diagnostic.
        let dev_id = graph
            .inspection
            .authoring
            .selected_outputs
            .iter()
            .find(|(_, h)| h.output_name == "dev")
            .unwrap()
            .0
            .clone();
        let dev_exp = graph.explain_selected_output(&dev_id).unwrap();
        assert_eq!(dev_exp.producer_recipe, "recipe:example-sdk");
        assert_eq!(dev_exp.output_name, "dev");
        assert_eq!(dev_exp.selection_valid, Some(true));

        let bad_id = graph
            .inspection
            .authoring
            .selected_outputs
            .iter()
            .find(|(_, h)| h.output_name == "nonexistent")
            .unwrap()
            .0
            .clone();
        let bad_exp = graph.explain_selected_output(&bad_id).unwrap();
        assert_eq!(bad_exp.selection_valid, Some(false));
        assert_eq!(bad_exp.diagnostics.len(), 1);
        assert_eq!(bad_exp.diagnostics[0].kind, AuthoringDiagnosticKind::InvalidOutputSelection);

        // Unavailable provenance is reported explicitly, never fabricated.
        let orphan_why = graph.why("recipe:orphan").unwrap();
        assert!(!orphan_why.authoring.as_ref().unwrap().is_provenance_available);
        let orphan_layers = orphan_why.layers.as_ref().unwrap();
        assert!(!orphan_layers.is_authoring_present);
        assert!(orphan_layers.is_normalized_recipe_present);
        assert!(!orphan_layers.is_observed_execution_present);

        // Diagnostic metadata changes do not alter recipe or instance identity.
        let before_instance_ids: BTreeSet<_> = graph.inspection.authoring.instances.keys().cloned().collect();
        graph.inspection.authoring.diagnostics[0].message = "updated diagnostic text".into();
        let after_instance_ids: BTreeSet<_> =
            graph.inspection.authoring.instances.values().map(|inst| inst.identity().unwrap()).collect();
        assert_eq!(before_instance_ids, after_instance_ids);

        // Composed impact on SemanticGraph matches reverse DependsOn reachability.
        let impact = graph
            .composed_impact(&BTreeSet::from(["recipe:diamond-shared".into()]), None, QueryLimits::default())
            .unwrap();
        assert_eq!(impact.impacted_nodes, [
            "recipe:diamond-left",
            "recipe:diamond-result",
            "recipe:diamond-right",
            "recipe:diamond-shared"
        ]);
        assert_eq!(impact.outcome, ImpactOutcome::Partial);
    }
}
