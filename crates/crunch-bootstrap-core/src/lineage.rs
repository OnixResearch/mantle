use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::DEFAULT_AUDIT_SEED_MAX_BYTES;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedClass {
    Hex0Seed,
}

impl SeedClass {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Hex0Seed => "hex0-seed",
        }
    }

    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Hex0Seed)
    }
}

impl core::fmt::Display for SeedClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Blake3Hex(String);

impl Blake3Hex {
    pub fn new(hex: impl Into<String>) -> Result<Self, crate::LineageError> {
        let hex = hex.into();
        let valid = hex.len() == crate::BLAKE3_HEX_LENGTH
            && hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
        if !valid {
            return Err(crate::LineageError::MalformedBlake3Hex(hex));
        }
        Ok(Self(hex))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Blake3Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let hex = String::deserialize(deserializer)?;
        Self::new(hex).map_err(serde::de::Error::custom)
    }
}

impl core::fmt::Display for Blake3Hex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigestEntry {
    pub algorithm: String,
    pub hex_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interoperability_reason: Option<String>,
}

impl DigestEntry {
    pub fn blake3(hex: Blake3Hex) -> Self {
        Self {
            algorithm: String::from("blake3"),
            hex_value: hex.0,
            interoperability_reason: None,
        }
    }

    pub fn non_blake3(algorithm: String, hex_value: String, reason: String) -> Self {
        Self {
            algorithm,
            hex_value,
            interoperability_reason: Some(reason),
        }
    }

    pub fn is_blake3(&self) -> bool {
        self.algorithm == "blake3"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditedSeed {
    pub seed_class: SeedClass,
    pub instruction_set: String,
    pub entry_point: String,
    pub io_contract: String,
    pub host_interface_surface: String,
    pub human_readable_source: String,
    pub reproduction_transcript: String,
    pub audit_note: String,
    pub audit_seed_max_bytes: u32,
    pub seed_bytes_len: u32,
    pub seed_digest: DigestEntry,
}

impl Default for AuditedSeed {
    fn default() -> Self {
        Self {
            seed_class: SeedClass::Hex0Seed,
            instruction_set: String::new(),
            entry_point: String::new(),
            io_contract: String::new(),
            host_interface_surface: String::new(),
            human_readable_source: String::new(),
            reproduction_transcript: String::new(),
            audit_note: String::new(),
            audit_seed_max_bytes: DEFAULT_AUDIT_SEED_MAX_BYTES,
            seed_bytes_len: 0,
            seed_digest: DigestEntry {
                algorithm: String::from("blake3"),
                hex_value: String::new(),
                interoperability_reason: None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceArtifact {
    pub id: String,
    pub name: String,
    pub digest: DigestEntry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    pub id: String,
    pub name: String,
    pub producing_tool_id: String,
    pub digest: DigestEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionTool {
    pub id: String,
    pub name: String,
    pub source_artifact_ids: Vec<String>,
    #[serde(default)]
    pub input_artifact_ids: Vec<String>,
    pub output_artifact_ids: Vec<String>,
    pub digest: DigestEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    pub id: String,
    pub name: String,
    pub target_artifact_id: String,
    pub digest: DigestEntry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOutputRole {
    TargetPrefixedTools,
    Headers,
    Libraries,
    ProviderMetadata,
    RetainedToolMetadata,
    ReductionMetadata,
    ProviderNotes,
    DynamicLinker,
}

impl core::fmt::Display for ProviderOutputRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TargetPrefixedTools => f.write_str("target_prefixed_tools"),
            Self::Headers => f.write_str("headers"),
            Self::Libraries => f.write_str("libraries"),
            Self::ProviderMetadata => f.write_str("provider_metadata"),
            Self::RetainedToolMetadata => f.write_str("retained_tool_metadata"),
            Self::ReductionMetadata => f.write_str("reduction_metadata"),
            Self::ProviderNotes => f.write_str("provider_notes"),
            Self::DynamicLinker => f.write_str("dynamic_linker"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedProviderOutput {
    pub role: ProviderOutputRole,
    pub producing_artifact_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentAssumption {
    pub id: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageTransition {
    pub from_node_id: String,
    pub to_node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageNodeKind {
    Seed,
    SourceArtifact,
    GeneratedArtifact,
    TransitionTool,
    Patch,
    ProviderOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageNode {
    pub id: String,
    pub kind: LineageNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageManifest {
    pub seed: AuditedSeed,
    pub source_artifacts: Vec<SourceArtifact>,
    pub generated_artifacts: Vec<GeneratedArtifact>,
    pub transition_tools: Vec<TransitionTool>,
    pub patches: Vec<Patch>,
    pub provider_outputs: Vec<NormalizedProviderOutput>,
    pub environment_assumptions: Vec<EnvironmentAssumption>,
    pub stage_graph: Vec<StageTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolvedLineageNodeId {
    Seed,
    Source(crate::LineageSourceArtifactId),
    Generated(crate::LineageGeneratedArtifactId),
    Tool(crate::LineageTransitionToolId),
    Patch(crate::LineagePatchId),
}

impl ResolvedLineageNodeId {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Seed => "seed",
            Self::Source(value) => value.as_str(),
            Self::Generated(value) => value.as_str(),
            Self::Tool(value) => value.as_str(),
            Self::Patch(value) => value.as_str(),
        }
    }

    pub fn role_name(&self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::Source(_) => "source_artifact",
            Self::Generated(_) => "generated_artifact",
            Self::Tool(_) => "transition_tool",
            Self::Patch(_) => "patch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedLineageTransition {
    pub from: ResolvedLineageNodeId,
    pub to: ResolvedLineageNodeId,
    pub tool: Option<crate::LineageTransitionToolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedGeneratedArtifactBinding {
    pub artifact: crate::LineageGeneratedArtifactId,
    pub producing_tool: crate::LineageTransitionToolId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedTransitionToolBinding {
    pub tool: crate::LineageTransitionToolId,
    pub sources: Vec<crate::LineageSourceArtifactId>,
    pub inputs: Vec<ResolvedLineageNodeId>,
    pub outputs: Vec<crate::LineageGeneratedArtifactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedLineageManifest {
    pub source_artifact_ids: Vec<crate::LineageSourceArtifactId>,
    pub generated_artifacts: Vec<AdmittedGeneratedArtifactBinding>,
    pub transition_tools: Vec<AdmittedTransitionToolBinding>,
    pub patch_ids: Vec<crate::LineagePatchId>,
    pub environment_assumption_ids: Vec<crate::LineageEnvironmentAssumptionId>,
    pub stage_graph: Vec<AdmittedLineageTransition>,
}

pub fn admit_lineage_manifest(manifest: &LineageManifest) -> Result<AdmittedLineageManifest, crate::LineageError> {
    let nodes = admitted_lineage_nodes(manifest)?;
    let source_artifact_ids = manifest
        .source_artifacts
        .iter()
        .map(|artifact| nominal_value("source_artifacts[].id", crate::LineageSourceArtifactId::new(&artifact.id)))
        .collect::<Result<Vec<_>, _>>()?;
    let generated_artifacts = manifest
        .generated_artifacts
        .iter()
        .map(|artifact| {
            Ok(AdmittedGeneratedArtifactBinding {
                artifact: nominal_value(
                    "generated_artifacts[].id",
                    crate::LineageGeneratedArtifactId::new(&artifact.id),
                )?,
                producing_tool: require_tool_node(&artifact.producing_tool_id, &nodes)?,
            })
        })
        .collect::<Result<Vec<_>, crate::LineageError>>()?;
    let transition_tools = manifest
        .transition_tools
        .iter()
        .map(|tool| admit_transition_tool(tool, &nodes))
        .collect::<Result<Vec<_>, _>>()?;
    let patch_ids = manifest
        .patches
        .iter()
        .map(|patch| nominal_value("patches[].id", crate::LineagePatchId::new(&patch.id)))
        .collect::<Result<Vec<_>, _>>()?;
    let environment_assumption_ids = manifest
        .environment_assumptions
        .iter()
        .map(|assumption| {
            nominal_value("environment_assumptions[].id", crate::LineageEnvironmentAssumptionId::new(&assumption.id))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let stage_graph = manifest
        .stage_graph
        .iter()
        .map(|transition| {
            Ok(AdmittedLineageTransition {
                from: require_node(&transition.from_node_id, &nodes)?,
                to: require_node(&transition.to_node_id, &nodes)?,
                tool: transition.tool_id.as_deref().map(|id| require_tool_node(id, &nodes)).transpose()?,
            })
        })
        .collect::<Result<Vec<_>, crate::LineageError>>()?;

    let validation = crate::validate_lineage(manifest);
    if !validation.errors.is_empty() {
        return Err(crate::LineageError::MultipleErrors(validation.errors));
    }

    Ok(AdmittedLineageManifest {
        source_artifact_ids,
        generated_artifacts,
        transition_tools,
        patch_ids,
        environment_assumption_ids,
        stage_graph,
    })
}

fn admitted_lineage_nodes(
    manifest: &LineageManifest,
) -> Result<BTreeMap<String, ResolvedLineageNodeId>, crate::LineageError> {
    let mut nodes = BTreeMap::new();
    nodes.insert("seed".to_string(), ResolvedLineageNodeId::Seed);
    for artifact in &manifest.source_artifacts {
        let id = nominal_value("source_artifacts[].id", crate::LineageSourceArtifactId::new(&artifact.id))?;
        nodes.insert(artifact.id.clone(), ResolvedLineageNodeId::Source(id));
    }
    for artifact in &manifest.generated_artifacts {
        let id = nominal_value("generated_artifacts[].id", crate::LineageGeneratedArtifactId::new(&artifact.id))?;
        nodes.insert(artifact.id.clone(), ResolvedLineageNodeId::Generated(id));
    }
    for tool in &manifest.transition_tools {
        let id = nominal_value("transition_tools[].id", crate::LineageTransitionToolId::new(&tool.id))?;
        nodes.insert(tool.id.clone(), ResolvedLineageNodeId::Tool(id));
    }
    for patch in &manifest.patches {
        let id = nominal_value("patches[].id", crate::LineagePatchId::new(&patch.id))?;
        nodes.insert(patch.id.clone(), ResolvedLineageNodeId::Patch(id));
    }
    Ok(nodes)
}

fn admit_transition_tool(
    tool: &TransitionTool,
    nodes: &BTreeMap<String, ResolvedLineageNodeId>,
) -> Result<AdmittedTransitionToolBinding, crate::LineageError> {
    let tool_id = nominal_value("transition_tools[].id", crate::LineageTransitionToolId::new(&tool.id))?;
    let sources = tool
        .source_artifact_ids
        .iter()
        .map(|id| nominal_value("transition_tools[].source_artifact_ids", crate::LineageSourceArtifactId::new(id)))
        .collect::<Result<Vec<_>, _>>()?;
    let inputs = tool.input_artifact_ids.iter().map(|id| require_node(id, nodes)).collect::<Result<Vec<_>, _>>()?;
    let outputs = tool
        .output_artifact_ids
        .iter()
        .map(|id| require_generated_node(id, nodes))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(AdmittedTransitionToolBinding {
        tool: tool_id,
        sources,
        inputs,
        outputs,
    })
}

fn require_node(
    reference: &str,
    nodes: &BTreeMap<String, ResolvedLineageNodeId>,
) -> Result<ResolvedLineageNodeId, crate::LineageError> {
    nodes
        .get(reference)
        .cloned()
        .ok_or_else(|| crate::LineageError::MissingLineageNode(reference.to_string()))
}

fn require_generated_node(
    reference: &str,
    nodes: &BTreeMap<String, ResolvedLineageNodeId>,
) -> Result<crate::LineageGeneratedArtifactId, crate::LineageError> {
    match require_node(reference, nodes)? {
        ResolvedLineageNodeId::Generated(id) => Ok(id),
        actual => Err(wrong_node_role(reference, "generated_artifact", actual.role_name())),
    }
}

fn require_tool_node(
    reference: &str,
    nodes: &BTreeMap<String, ResolvedLineageNodeId>,
) -> Result<crate::LineageTransitionToolId, crate::LineageError> {
    match require_node(reference, nodes)? {
        ResolvedLineageNodeId::Tool(id) => Ok(id),
        actual => Err(wrong_node_role(reference, "transition_tool", actual.role_name())),
    }
}

fn wrong_node_role(reference: &str, expected: &str, actual: &str) -> crate::LineageError {
    crate::LineageError::WrongLineageNodeRole {
        reference: reference.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}

fn nominal_value<T>(field: &str, value: Result<T, crate::NominalValueError>) -> Result<T, crate::LineageError> {
    value.map_err(|error| crate::LineageError::InvalidNominal {
        field: field.to_string(),
        reason: error.to_string(),
    })
}

impl LineageManifest {
    pub fn all_node_ids(&self) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();
        ids.insert(String::from("seed"));
        for a in &self.source_artifacts {
            ids.insert(a.id.clone());
        }
        for a in &self.generated_artifacts {
            ids.insert(a.id.clone());
        }
        for t in &self.transition_tools {
            ids.insert(t.id.clone());
        }
        for p in &self.patches {
            ids.insert(p.id.clone());
        }
        ids
    }

    pub fn all_nodes(&self) -> Vec<LineageNode> {
        let mut nodes = Vec::new();
        nodes.push(LineageNode {
            id: String::from("seed"),
            kind: LineageNodeKind::Seed,
        });
        for a in &self.source_artifacts {
            nodes.push(LineageNode {
                id: a.id.clone(),
                kind: LineageNodeKind::SourceArtifact,
            });
        }
        for a in &self.generated_artifacts {
            nodes.push(LineageNode {
                id: a.id.clone(),
                kind: LineageNodeKind::GeneratedArtifact,
            });
        }
        for t in &self.transition_tools {
            nodes.push(LineageNode {
                id: t.id.clone(),
                kind: LineageNodeKind::TransitionTool,
            });
        }
        for p in &self.patches {
            nodes.push(LineageNode {
                id: p.id.clone(),
                kind: LineageNodeKind::Patch,
            });
        }
        nodes
    }

    pub fn reachable_from_seed(&self) -> BTreeSet<String> {
        let mut reachable = BTreeSet::new();
        reachable.insert(String::from("seed"));

        let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for transition in &self.stage_graph {
            edges.entry(transition.from_node_id.clone()).or_default().push(transition.to_node_id.clone());
        }
        for tool in &self.transition_tools {
            for src in &tool.source_artifact_ids {
                edges.entry(src.clone()).or_default().push(tool.id.clone());
            }
            for out in &tool.output_artifact_ids {
                edges.entry(tool.id.clone()).or_default().push(out.clone());
            }
        }

        let mut work: Vec<String> = alloc::vec![String::from("seed")];
        let max_iterations: u32 = 10_000;
        let mut iteration: u32 = 0;
        while let Some(current) = work.pop() {
            iteration = iteration.saturating_add(1);
            if iteration > max_iterations {
                break;
            }
            if let Some(neighbors) = edges.get(&current) {
                for next in neighbors {
                    if reachable.insert(next.clone()) {
                        work.push(next.clone());
                    }
                }
            }
        }
        reachable
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    fn valid_blake3() -> Blake3Hex {
        Blake3Hex::new("a".repeat(crate::BLAKE3_HEX_LENGTH)).unwrap()
    }

    fn valid_digest() -> DigestEntry {
        DigestEntry::blake3(valid_blake3())
    }

    fn minimal_seed() -> AuditedSeed {
        AuditedSeed {
            seed_class: SeedClass::Hex0Seed,
            instruction_set: "x86".to_string(),
            entry_point: "0x00".to_string(),
            io_contract: "stdin/stdout byte stream".to_string(),
            host_interface_surface: "linux read/write syscalls".to_string(),
            human_readable_source: "bootstrap/hex0.hex0".to_string(),
            reproduction_transcript: "hex0 assembler self-hosts from hex source".to_string(),
            audit_note: "Hand-audited 357-byte hex0 seed".to_string(),
            audit_seed_max_bytes: DEFAULT_AUDIT_SEED_MAX_BYTES,
            seed_bytes_len: 357,
            seed_digest: valid_digest(),
        }
    }

    fn minimal_manifest() -> LineageManifest {
        LineageManifest {
            seed: minimal_seed(),
            source_artifacts: vec![SourceArtifact {
                id: "hex0-src".to_string(),
                name: "hex0 source".to_string(),
                digest: valid_digest(),
                provenance: Some("stage0-posix".to_string()),
                url: None,
            }],
            generated_artifacts: vec![GeneratedArtifact {
                id: "hex0-bin".to_string(),
                name: "hex0 binary".to_string(),
                producing_tool_id: "hex0-assembler".to_string(),
                digest: valid_digest(),
            }],
            transition_tools: vec![TransitionTool {
                id: "hex0-assembler".to_string(),
                name: "hex0 assembler".to_string(),
                source_artifact_ids: vec!["hex0-src".to_string()],
                input_artifact_ids: vec![],
                output_artifact_ids: vec!["hex0-bin".to_string()],
                digest: valid_digest(),
            }],
            patches: vec![],
            provider_outputs: vec![NormalizedProviderOutput {
                role: ProviderOutputRole::TargetPrefixedTools,
                producing_artifact_id: "hex0-bin".to_string(),
            }],
            environment_assumptions: vec![EnvironmentAssumption {
                id: "linux-kernel".to_string(),
                description: "Linux kernel x86_64".to_string(),
                category: "kernel".to_string(),
            }],
            stage_graph: vec![StageTransition {
                from_node_id: "seed".to_string(),
                to_node_id: "hex0-src".to_string(),
                tool_id: None,
            }],
        }
    }

    #[test]
    fn seed_class_display() {
        assert_eq!(SeedClass::Hex0Seed.as_str(), "hex0-seed");
        assert!(SeedClass::Hex0Seed.is_supported());
    }

    #[test]
    fn blake3_hex_constructor_and_deserializer_enforce_format() {
        let valid_text = "a".repeat(crate::BLAKE3_HEX_LENGTH);
        let valid = Blake3Hex::new(valid_text.clone()).unwrap();
        let serialized = serde_json::to_string(&valid).unwrap();
        let deserialized: Blake3Hex = serde_json::from_str(&serialized).unwrap();

        assert_eq!(valid.as_str(), valid_text);
        assert_eq!(deserialized, valid);
        assert!(Blake3Hex::new("abcd").is_err());
        assert!(Blake3Hex::new("A".repeat(crate::BLAKE3_HEX_LENGTH)).is_err());
        assert!(Blake3Hex::new("g".repeat(crate::BLAKE3_HEX_LENGTH)).is_err());
        assert!(serde_json::from_str::<Blake3Hex>("\"abcd\"").is_err());
    }

    #[test]
    fn all_node_ids_collects_all() {
        let manifest = minimal_manifest();
        let ids = manifest.all_node_ids();
        assert!(ids.contains("seed"));
        assert!(ids.contains("hex0-src"));
        assert!(ids.contains("hex0-bin"));
        assert!(ids.contains("hex0-assembler"));
    }

    #[test]
    fn reachable_from_seed_traverses_graph() {
        let manifest = minimal_manifest();
        let reachable = manifest.reachable_from_seed();
        assert!(reachable.contains("seed"));
        assert!(reachable.contains("hex0-src"));
        assert!(reachable.contains("hex0-assembler"));
        assert!(reachable.contains("hex0-bin"));
    }

    #[test]
    fn digest_entry_blake3_constructor() {
        let d = DigestEntry::blake3(valid_blake3());
        assert!(d.is_blake3());
        assert!(d.interoperability_reason.is_none());
    }

    #[test]
    fn digest_entry_non_blake3_requires_reason() {
        let d = DigestEntry::non_blake3("sha256".to_string(), "b".repeat(64), "Cargo interop".to_string());
        assert!(!d.is_blake3());
        assert_eq!(d.interoperability_reason.as_deref(), Some("Cargo interop"));
    }

    #[test]
    fn default_seed_budget_is_4096() {
        let seed = AuditedSeed::default();
        assert_eq!(seed.audit_seed_max_bytes, 4096);
    }

    #[test]
    fn environment_assumptions_not_in_node_ids() {
        let manifest = minimal_manifest();
        let ids = manifest.all_node_ids();
        assert!(!ids.contains("linux-kernel"));
    }

    #[test]
    fn serde_roundtrip() {
        let manifest = minimal_manifest();
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: LineageManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn valid_lineage_admission_resolves_node_roles_without_wire_drift() {
        let manifest = minimal_manifest();
        let wire_before = serde_json::to_vec(&manifest).unwrap();
        let admitted = admit_lineage_manifest(&manifest).unwrap();

        assert_eq!(admitted.source_artifact_ids[0].as_str(), "hex0-src");
        assert_eq!(admitted.generated_artifacts[0].artifact.as_str(), "hex0-bin");
        assert_eq!(admitted.generated_artifacts[0].producing_tool.as_str(), "hex0-assembler");
        assert!(matches!(admitted.stage_graph[0].from, ResolvedLineageNodeId::Seed));
        assert!(matches!(admitted.stage_graph[0].to, ResolvedLineageNodeId::Source(_)));
        assert_eq!(serde_json::to_vec(&manifest).unwrap(), wire_before);
    }

    #[test]
    fn missing_and_wrong_role_lineage_references_fail_closed() {
        let mut missing = minimal_manifest();
        missing.stage_graph[0].to_node_id = "missing-node".to_string();

        let mut wrong_role = minimal_manifest();
        wrong_role.generated_artifacts[0].producing_tool_id = "hex0-src".to_string();

        assert!(matches!(admit_lineage_manifest(&missing), Err(crate::LineageError::MissingLineageNode(_))));
        assert!(matches!(admit_lineage_manifest(&wrong_role), Err(crate::LineageError::WrongLineageNodeRole { .. })));
    }

    #[test]
    fn control_bearing_lineage_identifier_fails_nominal_admission() {
        let mut manifest = minimal_manifest();
        let control_id = "hex0\nsource".to_string();
        manifest.source_artifacts[0].id = control_id.clone();
        manifest.transition_tools[0].source_artifact_ids[0] = control_id.clone();
        manifest.stage_graph[0].to_node_id = control_id;

        assert!(matches!(admit_lineage_manifest(&manifest), Err(crate::LineageError::InvalidNominal { .. })));
    }
}
