// machine-artifact-public: semantic-graph.command-reports
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::path::Path;

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

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct GraphQueryResult<'a> {
    pub schema: &'static str,
    pub root: String,
    pub nodes: Vec<&'a SemanticNode>,
    pub edges: Vec<&'a SemanticEdge>,
    pub aliases: Vec<&'a SemanticAlias>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct DependentsResult<'a> {
    pub schema: &'static str,
    pub target: String,
    pub dependents: Vec<&'a SemanticNode>,
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
}

impl SemanticGraph {
    pub fn empty() -> Self {
        Self {
            schema: SEMANTIC_GRAPH_SCHEMA.to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            aliases: Vec::new(),
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
        let content = fs::read_to_string(path)
            .map_err(|err| SemanticGraphError::Io(path.display().to_string(), err.to_string()))?;
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
        for alias in &self.aliases {
            if alias.alias.trim().is_empty() || alias.target.trim().is_empty() {
                return Err(SemanticGraphError::Invalid("semantic graph aliases must not be empty".to_string()));
            }
            if !ids.contains(alias.target.as_str()) {
                return Err(SemanticGraphError::Invalid(format!(
                    "semantic graph alias `{}` targets missing node `{}`",
                    alias.alias, alias.target
                )));
            }
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
        let root = self.resolve_or_incomplete("graph", query)?;
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::from([root.clone()]);
        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for edge in self.edges.iter().filter(|edge| edge.from == current || edge.to == current) {
                queue.push_back(edge.from.clone());
                queue.push_back(edge.to.clone());
            }
        }
        let nodes = self.nodes.iter().filter(|node| visited.contains(&node.id)).collect();
        let edges = self
            .edges
            .iter()
            .filter(|edge| visited.contains(&edge.from) && visited.contains(&edge.to))
            .collect();
        let aliases = self.aliases.iter().filter(|alias| visited.contains(&alias.target)).collect();
        Ok(GraphQueryResult {
            schema: "mantle-semantic-build-graph-query-v1",
            root,
            nodes,
            edges,
            aliases,
        })
    }

    pub fn dependents(&self, query: &str) -> Result<DependentsResult<'_>, SemanticGraphError> {
        let target = self.resolve_or_incomplete("dependents", query)?;
        let dependent_ids: BTreeSet<&str> = self
            .edges
            .iter()
            .filter(|edge| edge.to == target && edge.kind == SemanticEdgeKind::DependsOn)
            .map(|edge| edge.from.as_str())
            .collect();
        let dependents = self.nodes.iter().filter(|node| dependent_ids.contains(node.id.as_str())).collect();
        Ok(DependentsResult {
            schema: "mantle-semantic-build-dependents-v1",
            target,
            dependents,
        })
    }

    pub fn why(&self, query: &str) -> Result<WhyResult<'_>, SemanticGraphError> {
        let target = self.resolve_or_incomplete("why", query)?;
        let node = self.node(&target).ok_or_else(|| {
            SemanticGraphError::Invalid(format!(
                "resolved semantic graph identity `{target}` has no corresponding node"
            ))
        })?;
        let outgoing = |kind: SemanticEdgeKind| -> Vec<&SemanticNode> {
            self.edges
                .iter()
                .filter(|edge| edge.from == target && edge.kind == kind)
                .filter_map(|edge| self.node(&edge.to))
                .collect()
        };
        let producing_recipe = self
            .edges
            .iter()
            .find(|edge| edge.from == target && edge.kind == SemanticEdgeKind::ProducedBy)
            .and_then(|edge| self.node(&edge.to));
        let why = WhyResult {
            schema: "mantle-semantic-build-why-v1",
            target: target.clone(),
            node,
            producing_recipe,
            sources: outgoing(SemanticEdgeKind::UsesSource),
            providers: outgoing(SemanticEdgeKind::UsesProvider),
            sandboxes: outgoing(SemanticEdgeKind::UsesSandbox),
            proof_receipts: outgoing(SemanticEdgeKind::SupportsProof),
            witness_requests: outgoing(SemanticEdgeKind::RequestsWitness),
            release_evidence: outgoing(SemanticEdgeKind::ReleasesAs),
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
}
