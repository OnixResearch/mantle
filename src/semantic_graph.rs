// machine-artifact-public: semantic-graph.command-reports
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

pub use mantle_application_contract::inspection::Action;
pub use mantle_application_contract::inspection::ActionComparison;
pub use mantle_application_contract::inspection::Configuration;
pub use mantle_application_contract::inspection::DependencyIndex;
pub use mantle_application_contract::inspection::Direction;
pub use mantle_application_contract::inspection::EvaluationState;
pub use mantle_application_contract::inspection::InspectionFacts;
use mantle_application_contract::inspection::MAX_GRAPH_BYTES;
pub use mantle_application_contract::inspection::QueryLimits;
pub use mantle_application_contract::inspection::Reachability;
pub use mantle_application_contract::inspection::compare_actions;
use serde::Deserialize;
use serde::Serialize;

pub const SEMANTIC_GRAPH_SCHEMA: &str = "mantle-semantic-build-graph-v1";

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
        if self.schema != SEMANTIC_GRAPH_SCHEMA {
            return Err(SemanticGraphError::Invalid(format!(
                "semantic graph schema is `{}`, expected `{}`",
                self.schema, SEMANTIC_GRAPH_SCHEMA
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
        let inspection = InspectionView {
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
        };
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
}
