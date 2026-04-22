use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::AttestationDigest;
use crate::SchemaVersion;

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Claims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_claim: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_aliases: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct RawClaims {
    component_name: Option<String>,
    version_claim: Option<String>,
    supplier: Option<String>,
    homepage: Option<String>,
    license: Option<String>,
    source_aliases: Option<Vec<String>>,
    extra: Option<BTreeMap<String, String>>,
}

impl<'de> Deserialize<'de> for Claims {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawClaims::deserialize(deserializer)?;
        Ok(Self {
            component_name: raw.component_name,
            version_claim: raw.version_claim,
            supplier: raw.supplier,
            homepage: raw.homepage,
            license: raw.license,
            source_aliases: raw.source_aliases.unwrap_or_else(Vec::new),
            extra: raw.extra.unwrap_or_else(BTreeMap::new),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Source,
    Recipe,
    Artifact,
    Patch,
    Project,
    Closure,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct Node {
    pub node_id: String,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct RawNode {
    node_id: String,
    kind: NodeKind,
    attributes: Option<BTreeMap<String, String>>,
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawNode::deserialize(deserializer)?;
        Ok(Self {
            node_id: raw.node_id,
            kind: raw.kind,
            attributes: raw.attributes.unwrap_or_else(BTreeMap::new),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeKind {
    BuildInput,
    RuntimeReference,
    ProducedBy,
    FetchedFrom,
    PatchedBy,
    MemberOfClosure,
    DeclaredByProject,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Edge {
    pub from_node_id: String,
    pub kind: EdgeKind,
    pub to_node_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ArtifactFacts {
    pub logical_path: String,
    pub output_name: String,
    pub content_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArtifactAttestation {
    pub schema_version: SchemaVersion,
    pub claims: Claims,
    pub facts: ArtifactFacts,
    pub subject_node_id: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Deserialize)]
struct RawArtifactAttestation {
    schema_version: SchemaVersion,
    claims: Option<Claims>,
    facts: ArtifactFacts,
    subject_node_id: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl<'de> Deserialize<'de> for ArtifactAttestation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawArtifactAttestation::deserialize(deserializer)?;
        Ok(Self {
            schema_version: raw.schema_version,
            claims: raw.claims.unwrap_or_default(),
            facts: raw.facts,
            subject_node_id: raw.subject_node_id,
            nodes: raw.nodes,
            edges: raw.edges,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosureSemantics {
    Runtime,
    Build,
    Complete,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ArtifactReference {
    pub node_id: String,
    pub logical_path: String,
    pub attestation_digest: AttestationDigest,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ClosureFacts {
    pub closure_node_id: String,
    pub root_node_ids: Vec<String>,
    pub semantics: ClosureSemantics,
    pub members: Vec<ArtifactReference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ClosureAttestation {
    pub schema_version: SchemaVersion,
    pub claims: Claims,
    pub facts: ClosureFacts,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Deserialize)]
struct RawClosureAttestation {
    schema_version: SchemaVersion,
    claims: Option<Claims>,
    facts: ClosureFacts,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl<'de> Deserialize<'de> for ClosureAttestation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawClosureAttestation::deserialize(deserializer)?;
        Ok(Self {
            schema_version: raw.schema_version,
            claims: raw.claims.unwrap_or_default(),
            facts: raw.facts,
            nodes: raw.nodes,
            edges: raw.edges,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ProjectFacts {
    pub project_node_id: String,
    pub manifest_digest: String,
    pub lockfile_digest: String,
    pub selected_roots: Vec<ArtifactReference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectAttestation {
    pub schema_version: SchemaVersion,
    pub claims: Claims,
    pub facts: ProjectFacts,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Deserialize)]
struct RawProjectAttestation {
    schema_version: SchemaVersion,
    claims: Option<Claims>,
    facts: ProjectFacts,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl<'de> Deserialize<'de> for ProjectAttestation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawProjectAttestation::deserialize(deserializer)?;
        Ok(Self {
            schema_version: raw.schema_version,
            claims: raw.claims.unwrap_or_default(),
            facts: raw.facts,
            nodes: raw.nodes,
            edges: raw.edges,
        })
    }
}
