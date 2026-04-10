use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::AttestationDigest;
use crate::SchemaVersion;

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Claims {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_claim: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Node {
    pub node_id: String,
    pub kind: NodeKind,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactAttestation {
    pub schema_version: SchemaVersion,
    #[serde(default)]
    pub claims: Claims,
    pub facts: ArtifactFacts,
    pub subject_node_id: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClosureAttestation {
    pub schema_version: SchemaVersion,
    #[serde(default)]
    pub claims: Claims,
    pub facts: ClosureFacts,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ProjectFacts {
    pub project_node_id: String,
    pub manifest_digest: String,
    pub lockfile_digest: String,
    pub selected_roots: Vec<ArtifactReference>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectAttestation {
    pub schema_version: SchemaVersion,
    #[serde(default)]
    pub claims: Claims,
    pub facts: ProjectFacts,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
