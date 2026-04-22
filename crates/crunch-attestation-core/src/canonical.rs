use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Serialize;

use crate::ArtifactAttestation;
use crate::ArtifactReference;
use crate::AttestationDigest;
use crate::Claims;
use crate::ClosureAttestation;
use crate::Edge;
use crate::Error;
use crate::Node;
use crate::NodeKind;
use crate::ProjectAttestation;

const MAX_NODE_COUNT: u32 = 4_096;
const MAX_EDGE_COUNT: u32 = 16_384;
const MAX_REFERENCE_COUNT: u32 = 16_384;
const MAX_ALIAS_COUNT: u32 = 256;

pub fn canonical_artifact_attestation(value: ArtifactAttestation) -> Result<ArtifactAttestation, Error> {
    assert!(!value.subject_node_id.is_empty(), "subject node id must not be empty");
    assert!(!value.facts.logical_path.is_empty(), "logical path must not be empty");
    validate_non_empty(NamedField {
        name: "subject_node_id",
        value: &value.subject_node_id,
    })?;
    validate_non_empty(NamedField {
        name: "logical_path",
        value: &value.facts.logical_path,
    })?;
    validate_non_empty(NamedField {
        name: "output_name",
        value: &value.facts.output_name,
    })?;
    validate_non_empty(NamedField {
        name: "content_digest",
        value: &value.facts.content_digest,
    })?;
    let claims = normalize_claims(value.claims)?;
    let nodes = normalize_nodes(value.nodes)?;
    let node_ids = node_id_set(&nodes);
    let edges = normalize_edges(value.edges, &node_ids)?;
    validate_node_kind(&nodes, &value.subject_node_id, NodeKind::Artifact, |node_id| Error::InvalidArtifactSubject {
        node_id,
    })?;

    Ok(ArtifactAttestation {
        schema_version: value.schema_version,
        claims,
        facts: value.facts,
        subject_node_id: value.subject_node_id,
        nodes,
        edges,
    })
}

pub fn artifact_attestation_canonical_bytes(value: ArtifactAttestation) -> Result<Vec<u8>, Error> {
    let canonical = canonical_artifact_attestation(value)?;
    to_canonical_bytes(&canonical)
}

pub fn artifact_attestation_canonical_digest(value: ArtifactAttestation) -> Result<AttestationDigest, Error> {
    let bytes = artifact_attestation_canonical_bytes(value)?;
    Ok(AttestationDigest::from_canonical_bytes(bytes))
}

pub fn canonical_closure_attestation(value: ClosureAttestation) -> Result<ClosureAttestation, Error> {
    assert!(!value.facts.closure_node_id.is_empty(), "closure node id must not be empty");
    assert!(!value.facts.root_node_ids.is_empty(), "closure roots must not be empty");
    validate_non_empty(NamedField {
        name: "closure_node_id",
        value: &value.facts.closure_node_id,
    })?;
    let claims = normalize_claims(value.claims)?;
    let nodes = normalize_nodes(value.nodes)?;
    let node_ids = node_id_set(&nodes);
    let edges = normalize_edges(value.edges, &node_ids)?;
    let members = normalize_references(value.facts.members)?;
    let roots = normalize_strings(value.facts.root_node_ids, MAX_REFERENCE_COUNT, "root_node_ids")?;
    validate_node_kind(&nodes, &value.facts.closure_node_id, NodeKind::Closure, |node_id| Error::InvalidClosureNode {
        node_id,
    })?;
    validate_artifact_references(&nodes, &members, |node_id| Error::InvalidClosureRoot { node_id })?;
    validate_closure_roots(&nodes, &roots, &members)?;

    Ok(ClosureAttestation {
        schema_version: value.schema_version,
        claims,
        facts: crate::ClosureFacts {
            closure_node_id: value.facts.closure_node_id,
            root_node_ids: roots,
            semantics: value.facts.semantics,
            members,
        },
        nodes,
        edges,
    })
}

pub fn closure_attestation_canonical_bytes(value: ClosureAttestation) -> Result<Vec<u8>, Error> {
    let canonical = canonical_closure_attestation(value)?;
    to_canonical_bytes(&canonical)
}

pub fn closure_attestation_canonical_digest(value: ClosureAttestation) -> Result<AttestationDigest, Error> {
    let bytes = closure_attestation_canonical_bytes(value)?;
    Ok(AttestationDigest::from_canonical_bytes(bytes))
}

pub fn canonical_project_attestation(value: ProjectAttestation) -> Result<ProjectAttestation, Error> {
    assert!(!value.facts.project_node_id.is_empty(), "project node id must not be empty");
    assert!(!value.facts.manifest_digest.is_empty(), "manifest digest must not be empty");
    validate_non_empty(NamedField {
        name: "project_node_id",
        value: &value.facts.project_node_id,
    })?;
    validate_non_empty(NamedField {
        name: "manifest_digest",
        value: &value.facts.manifest_digest,
    })?;
    validate_non_empty(NamedField {
        name: "lockfile_digest",
        value: &value.facts.lockfile_digest,
    })?;
    let claims = normalize_claims(value.claims)?;
    let nodes = normalize_nodes(value.nodes)?;
    let node_ids = node_id_set(&nodes);
    let edges = normalize_edges(value.edges, &node_ids)?;
    let selected_roots = normalize_references(value.facts.selected_roots)?;
    validate_node_kind(&nodes, &value.facts.project_node_id, NodeKind::Project, |node_id| Error::InvalidProjectNode {
        node_id,
    })?;
    validate_project_roots(&nodes, &selected_roots)?;

    Ok(ProjectAttestation {
        schema_version: value.schema_version,
        claims,
        facts: crate::ProjectFacts {
            project_node_id: value.facts.project_node_id,
            manifest_digest: value.facts.manifest_digest,
            lockfile_digest: value.facts.lockfile_digest,
            selected_roots,
        },
        nodes,
        edges,
    })
}

pub fn project_attestation_canonical_bytes(value: ProjectAttestation) -> Result<Vec<u8>, Error> {
    let canonical = canonical_project_attestation(value)?;
    to_canonical_bytes(&canonical)
}

pub fn project_attestation_canonical_digest(value: ProjectAttestation) -> Result<AttestationDigest, Error> {
    let bytes = project_attestation_canonical_bytes(value)?;
    Ok(AttestationDigest::from_canonical_bytes(bytes))
}

fn normalize_claims(value: Claims) -> Result<Claims, Error> {
    let aliases = normalize_strings(value.source_aliases, MAX_ALIAS_COUNT, "source_aliases")?;
    Ok(Claims {
        component_name: value.component_name,
        version_claim: value.version_claim,
        supplier: value.supplier,
        homepage: value.homepage,
        license: value.license,
        source_aliases: aliases,
        extra: value.extra,
    })
}

fn normalize_nodes(nodes: Vec<Node>) -> Result<Vec<Node>, Error> {
    validate_len(nodes.len(), MAX_NODE_COUNT)?;
    let mut normalized = nodes;
    for node in &normalized {
        validate_non_empty(NamedField {
            name: "node_id",
            value: &node.node_id,
        })?;
    }
    normalized.sort();
    reject_duplicate_nodes(&normalized)?;
    Ok(normalized)
}

fn normalize_edges(edges: Vec<Edge>, node_ids: &BTreeSet<String>) -> Result<Vec<Edge>, Error> {
    validate_len(edges.len(), MAX_EDGE_COUNT)?;
    let mut normalized = edges;
    for edge in &normalized {
        validate_non_empty(NamedField {
            name: "from_node_id",
            value: &edge.from_node_id,
        })?;
        validate_non_empty(NamedField {
            name: "to_node_id",
            value: &edge.to_node_id,
        })?;
        validate_edge_endpoint(&edge.from_node_id, node_ids)?;
        validate_edge_endpoint(&edge.to_node_id, node_ids)?;
    }
    normalized.sort();
    Ok(normalized)
}

fn normalize_references(values: Vec<ArtifactReference>) -> Result<Vec<ArtifactReference>, Error> {
    validate_len(values.len(), MAX_REFERENCE_COUNT)?;
    let mut normalized = values;
    for value in &normalized {
        validate_non_empty(NamedField {
            name: "reference.node_id",
            value: &value.node_id,
        })?;
        validate_non_empty(NamedField {
            name: "reference.logical_path",
            value: &value.logical_path,
        })?;
    }
    normalized.sort();
    Ok(normalized)
}

fn normalize_strings(values: Vec<String>, limit: u32, field: &'static str) -> Result<Vec<String>, Error> {
    validate_len(values.len(), limit)?;
    let mut normalized = values;
    for value in &normalized {
        validate_non_empty(NamedField { name: field, value })?;
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn reject_duplicate_nodes(nodes: &[Node]) -> Result<(), Error> {
    for window in nodes.windows(2) {
        if window[0].node_id == window[1].node_id {
            return Err(Error::DuplicateNodeId {
                node_id: window[0].node_id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_node_kind<F>(nodes: &[Node], node_id: &str, expected: NodeKind, error_fn: F) -> Result<(), Error>
where F: FnOnce(String) -> Error {
    let maybe_kind = nodes.iter().find(|node| node.node_id == node_id).map(|node| node.kind);
    if maybe_kind == Some(expected) {
        return Ok(());
    }

    Err(error_fn(node_id.to_string()))
}

fn validate_closure_roots(nodes: &[Node], roots: &[String], members: &[ArtifactReference]) -> Result<(), Error> {
    if roots.is_empty() {
        return Err(Error::EmptyClosureRoots);
    }
    let member_ids: BTreeSet<&str> = members.iter().map(|member| member.node_id.as_str()).collect();
    let artifact_ids = artifact_node_ids(nodes);
    for root in roots {
        if !artifact_ids.contains(root.as_str()) {
            return Err(Error::InvalidClosureRoot { node_id: root.clone() });
        }
        if !member_ids.contains(root.as_str()) {
            return Err(Error::MissingClosureRoot { node_id: root.clone() });
        }
    }
    Ok(())
}

fn validate_project_roots(nodes: &[Node], roots: &[ArtifactReference]) -> Result<(), Error> {
    validate_artifact_references(nodes, roots, |node_id| Error::InvalidProjectRoot { node_id })
}

fn validate_artifact_references<F>(nodes: &[Node], references: &[ArtifactReference], error_fn: F) -> Result<(), Error>
where F: Fn(String) -> Error {
    let artifact_ids = artifact_node_ids(nodes);
    for reference in references {
        if artifact_ids.contains(reference.node_id.as_str()) {
            continue;
        }
        return Err(error_fn(reference.node_id.clone()));
    }
    Ok(())
}

fn artifact_node_ids(nodes: &[Node]) -> BTreeSet<&str> {
    nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Artifact)
        .map(|node| node.node_id.as_str())
        .collect()
}

fn validate_edge_endpoint(node_id: &str, node_ids: &BTreeSet<String>) -> Result<(), Error> {
    if node_ids.contains(node_id) {
        return Ok(());
    }
    Err(Error::MissingNode {
        node_id: node_id.to_string(),
    })
}

struct NamedField<'a> {
    name: &'static str,
    value: &'a str,
}

fn validate_non_empty(field: NamedField<'_>) -> Result<(), Error> {
    if !field.value.is_empty() {
        return Ok(());
    }
    Err(Error::EmptyField { field: field.name })
}

fn validate_len(actual_usize: usize, limit: u32) -> Result<(), Error> {
    let actual = match u32::try_from(actual_usize) {
        Ok(value) => value,
        Err(_) => limit.saturating_add(1),
    };
    if actual <= limit {
        return Ok(());
    }
    Err(Error::CollectionTooLarge { limit, actual })
}

fn node_id_set(nodes: &[Node]) -> BTreeSet<String> {
    nodes.iter().map(|node| node.node_id.clone()).collect()
}

pub(crate) fn to_canonical_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where T: Serialize {
    serde_json::to_vec(value).map_err(|err| Error::Serialize {
        message: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::vec;

    use super::*;
    use crate::ArtifactFacts;
    use crate::Claims;
    use crate::ClosureFacts;
    use crate::ClosureSemantics;
    use crate::EdgeKind;
    use crate::ProjectFacts;
    use crate::SchemaVersion;

    #[test]
    fn artifact_digest_ignores_node_and_edge_insertion_order() {
        let first = artifact_attestation(false);
        let second = artifact_attestation(true);

        let first_bytes = artifact_attestation_canonical_bytes(first).unwrap();
        let second_bytes = artifact_attestation_canonical_bytes(second).unwrap();
        assert_eq!(first_bytes, second_bytes);

        let first_digest = artifact_attestation_canonical_digest(artifact_attestation(false)).unwrap();
        let second_digest = artifact_attestation_canonical_digest(artifact_attestation(true)).unwrap();
        assert_eq!(first_digest, second_digest);
    }

    #[test]
    fn closure_digest_ignores_discovery_order() {
        let first = closure_attestation(false);
        let second = closure_attestation(true);

        let first_bytes = closure_attestation_canonical_bytes(first).unwrap();
        let second_bytes = closure_attestation_canonical_bytes(second).unwrap();
        assert_eq!(first_bytes, second_bytes);

        let first_digest = closure_attestation_canonical_digest(closure_attestation(false)).unwrap();
        let second_digest = closure_attestation_canonical_digest(closure_attestation(true)).unwrap();
        assert_eq!(first_digest, second_digest);
    }

    #[test]
    fn project_digest_ignores_selection_order() {
        let first = project_attestation(SchemaVersion::V1, false);
        let second = project_attestation(SchemaVersion::V1, true);

        let first_bytes = project_attestation_canonical_bytes(first).unwrap();
        let second_bytes = project_attestation_canonical_bytes(second).unwrap();
        assert_eq!(first_bytes, second_bytes);

        let first_digest = project_attestation_canonical_digest(project_attestation(SchemaVersion::V1, false)).unwrap();
        let second_digest = project_attestation_canonical_digest(project_attestation(SchemaVersion::V1, true)).unwrap();
        assert_eq!(first_digest, second_digest);
    }

    #[test]
    fn project_digest_changes_when_schema_version_changes() {
        let first_digest = project_attestation_canonical_digest(project_attestation(SchemaVersion::V1, false)).unwrap();
        let second_digest =
            project_attestation_canonical_digest(project_attestation(SchemaVersion::new(2).unwrap(), false)).unwrap();
        assert_ne!(first_digest, second_digest);
    }

    #[test]
    fn canonicalization_rejects_missing_edge_endpoint() {
        let mut artifact = artifact_attestation(false);
        artifact.edges.push(edge("missing", EdgeKind::BuildInput, "recipe:hello"));

        let err = artifact_attestation_canonical_bytes(artifact).unwrap_err();
        assert_eq!(err, Error::MissingNode {
            node_id: "missing".to_string()
        });
    }

    #[test]
    fn canonicalization_rejects_duplicate_node_ids() {
        let mut artifact = artifact_attestation(false);
        artifact.nodes.push(node("artifact:hello", NodeKind::Artifact));

        let err = artifact_attestation_canonical_bytes(artifact).unwrap_err();
        assert_eq!(err, Error::DuplicateNodeId {
            node_id: "artifact:hello".to_string()
        });
    }

    #[test]
    fn canonicalization_rejects_invalid_artifact_subject() {
        let mut artifact = artifact_attestation(false);
        artifact.subject_node_id = "recipe:hello".to_string();

        let err = artifact_attestation_canonical_bytes(artifact).unwrap_err();
        assert_eq!(err, Error::InvalidArtifactSubject {
            node_id: "recipe:hello".to_string()
        });
    }

    #[test]
    fn canonicalization_rejects_invalid_closure_root_kind() {
        let mut closure = closure_attestation(false);
        closure.nodes.push(node("project:not-artifact", NodeKind::Project));
        closure.facts.root_node_ids = vec!["project:not-artifact".to_string()];

        let err = closure_attestation_canonical_bytes(closure).unwrap_err();
        assert_eq!(err, Error::InvalidClosureRoot {
            node_id: "project:not-artifact".to_string()
        });
    }

    #[test]
    fn canonicalization_rejects_closure_root_missing_from_members() {
        let mut closure = closure_attestation(false);
        closure.facts.root_node_ids = vec!["artifact:b".to_string()];
        closure.facts.members = vec![reference(
            "artifact:a",
            "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a",
            AttestationDigest::from_canonical_bytes(b"a".to_vec()),
        )];

        let err = closure_attestation_canonical_bytes(closure).unwrap_err();
        assert_eq!(err, Error::MissingClosureRoot {
            node_id: "artifact:b".to_string()
        });
    }

    #[test]
    fn canonicalization_rejects_invalid_project_root() {
        let mut project = project_attestation(SchemaVersion::V1, false);
        project.facts.selected_roots = vec![reference(
            "project:demo",
            "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a",
            AttestationDigest::from_canonical_bytes(b"bad-root".to_vec()),
        )];

        let err = project_attestation_canonical_bytes(project).unwrap_err();
        assert_eq!(err, Error::InvalidProjectRoot {
            node_id: "project:demo".to_string()
        });
    }

    #[test]
    fn canonicalization_rejects_empty_required_field() {
        let mut artifact = artifact_attestation(false);
        artifact.facts.output_name.clear();

        let err = artifact_attestation_canonical_bytes(artifact).unwrap_err();
        assert_eq!(err, Error::EmptyField { field: "output_name" });
    }

    #[test]
    fn canonicalization_rejects_oversized_alias_collection() {
        let mut artifact = artifact_attestation(false);
        artifact.claims.source_aliases = (0..257).map(|i| alloc::format!("alias-{i}")).collect();

        let err = artifact_attestation_canonical_bytes(artifact).unwrap_err();
        assert_eq!(err, Error::CollectionTooLarge {
            limit: 256,
            actual: 257,
        });
    }

    fn artifact_attestation(reversed: bool) -> ArtifactAttestation {
        let claims = Claims {
            supplier: Some("crunch".to_string()),
            homepage: Some("https://example.invalid/hello".to_string()),
            source_aliases: if reversed {
                vec!["mirror".to_string(), "origin".to_string()]
            } else {
                vec!["origin".to_string(), "mirror".to_string()]
            },
            ..Claims::default()
        };
        let mut recipe_attrs = BTreeMap::new();
        recipe_attrs.insert("name".to_string(), "hello".to_string());
        let nodes = if reversed {
            vec![
                node("src:hello", NodeKind::Source),
                Node {
                    node_id: "recipe:hello".to_string(),
                    kind: NodeKind::Recipe,
                    attributes: recipe_attrs.clone(),
                },
                node("artifact:hello", NodeKind::Artifact),
            ]
        } else {
            vec![
                node("artifact:hello", NodeKind::Artifact),
                Node {
                    node_id: "recipe:hello".to_string(),
                    kind: NodeKind::Recipe,
                    attributes: recipe_attrs,
                },
                node("src:hello", NodeKind::Source),
            ]
        };
        let edges = if reversed {
            vec![
                edge("src:hello", EdgeKind::FetchedFrom, "artifact:hello"),
                edge("artifact:hello", EdgeKind::ProducedBy, "recipe:hello"),
                edge("recipe:hello", EdgeKind::BuildInput, "src:hello"),
            ]
        } else {
            vec![
                edge("recipe:hello", EdgeKind::BuildInput, "src:hello"),
                edge("artifact:hello", EdgeKind::ProducedBy, "recipe:hello"),
                edge("src:hello", EdgeKind::FetchedFrom, "artifact:hello"),
            ]
        };
        ArtifactAttestation {
            schema_version: SchemaVersion::V1,
            claims,
            facts: ArtifactFacts {
                logical_path: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-hello".to_string(),
                output_name: "out".to_string(),
                content_digest: "blake3-hello".to_string(),
            },
            subject_node_id: "artifact:hello".to_string(),
            nodes,
            edges,
        }
    }

    fn closure_attestation(reversed: bool) -> ClosureAttestation {
        let digest_a = AttestationDigest::from_canonical_bytes(b"a".to_vec());
        let digest_b = AttestationDigest::from_canonical_bytes(b"b".to_vec());
        let members = if reversed {
            vec![
                reference("artifact:b", "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-b", digest_b),
                reference("artifact:a", "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a", digest_a),
            ]
        } else {
            vec![
                reference("artifact:a", "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a", digest_a),
                reference("artifact:b", "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-b", digest_b),
            ]
        };
        let nodes = if reversed {
            vec![
                node("artifact:b", NodeKind::Artifact),
                node("closure:runtime", NodeKind::Closure),
                node("artifact:a", NodeKind::Artifact),
            ]
        } else {
            vec![
                node("closure:runtime", NodeKind::Closure),
                node("artifact:a", NodeKind::Artifact),
                node("artifact:b", NodeKind::Artifact),
            ]
        };
        let edges = if reversed {
            vec![
                edge("artifact:b", EdgeKind::MemberOfClosure, "closure:runtime"),
                edge("artifact:a", EdgeKind::RuntimeReference, "artifact:b"),
                edge("artifact:a", EdgeKind::MemberOfClosure, "closure:runtime"),
            ]
        } else {
            vec![
                edge("artifact:a", EdgeKind::MemberOfClosure, "closure:runtime"),
                edge("artifact:a", EdgeKind::RuntimeReference, "artifact:b"),
                edge("artifact:b", EdgeKind::MemberOfClosure, "closure:runtime"),
            ]
        };
        ClosureAttestation {
            schema_version: SchemaVersion::V1,
            claims: Claims::default(),
            facts: ClosureFacts {
                closure_node_id: "closure:runtime".to_string(),
                root_node_ids: if reversed {
                    vec!["artifact:a".to_string(), "artifact:a".to_string()]
                } else {
                    vec!["artifact:a".to_string()]
                },
                semantics: ClosureSemantics::Runtime,
                members,
            },
            nodes,
            edges,
        }
    }

    fn project_attestation(schema_version: SchemaVersion, reversed: bool) -> ProjectAttestation {
        let nodes = if reversed {
            vec![
                node("artifact:b", NodeKind::Artifact),
                node("src:manifest", NodeKind::Source),
                node("project:demo", NodeKind::Project),
                node("artifact:a", NodeKind::Artifact),
            ]
        } else {
            vec![
                node("project:demo", NodeKind::Project),
                node("artifact:a", NodeKind::Artifact),
                node("artifact:b", NodeKind::Artifact),
                node("src:manifest", NodeKind::Source),
            ]
        };
        let edges = if reversed {
            vec![
                edge("project:demo", EdgeKind::DeclaredByProject, "src:manifest"),
                edge("project:demo", EdgeKind::DeclaredByProject, "artifact:b"),
                edge("project:demo", EdgeKind::DeclaredByProject, "artifact:a"),
            ]
        } else {
            vec![
                edge("project:demo", EdgeKind::DeclaredByProject, "artifact:a"),
                edge("project:demo", EdgeKind::DeclaredByProject, "artifact:b"),
                edge("project:demo", EdgeKind::DeclaredByProject, "src:manifest"),
            ]
        };
        let digest_a = AttestationDigest::from_canonical_bytes(b"artifact-a".to_vec());
        let digest_b = AttestationDigest::from_canonical_bytes(b"artifact-b".to_vec());
        ProjectAttestation {
            schema_version,
            claims: Claims {
                component_name: Some("demo".to_string()),
                ..Claims::default()
            },
            facts: ProjectFacts {
                project_node_id: "project:demo".to_string(),
                manifest_digest: "blake3-manifest".to_string(),
                lockfile_digest: "blake3-lock".to_string(),
                selected_roots: if reversed {
                    vec![
                        reference("artifact:b", "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-b", digest_b),
                        reference("artifact:a", "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a", digest_a),
                    ]
                } else {
                    vec![
                        reference("artifact:a", "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-a", digest_a),
                        reference("artifact:b", "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-b", digest_b),
                    ]
                },
            },
            nodes,
            edges,
        }
    }

    fn node(node_id: &str, kind: NodeKind) -> Node {
        Node {
            node_id: node_id.to_string(),
            kind,
            attributes: BTreeMap::new(),
        }
    }

    fn edge(from_node_id: &str, kind: EdgeKind, to_node_id: &str) -> Edge {
        Edge {
            from_node_id: from_node_id.to_string(),
            kind,
            to_node_id: to_node_id.to_string(),
        }
    }

    fn reference(node_id: &str, logical_path: &str, attestation_digest: AttestationDigest) -> ArtifactReference {
        ArtifactReference {
            node_id: node_id.to_string(),
            logical_path: logical_path.to_string(),
            attestation_digest,
        }
    }
}
