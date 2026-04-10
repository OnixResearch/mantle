use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crunch_attestation::ArtifactReference;
use crunch_attestation::Canonicalize;
use crunch_attestation::Claims;
use crunch_attestation::Edge;
use crunch_attestation::EdgeKind;
use crunch_attestation::Node;
use crunch_attestation::NodeKind;
use crunch_attestation::ProjectAttestation;
use crunch_attestation::ProjectFacts;
use crunch_attestation::SchemaVersion as AttestationSchemaVersion;

use crate::Error;
use crate::LockEntry;
use crate::LockedKind;
use crate::LockedPatch;
use crate::LockedPatchSource;
use crate::Lockfile;
use crate::ManifestInput;
use crate::ProjectManifest;

pub fn synthesize_project_attestation(
    manifest_text: &str,
    lock_text: &str,
    manifest: &ProjectManifest,
    lock: &Lockfile,
    selected_roots: &[ArtifactReference],
) -> Result<ProjectAttestation, Error> {
    validate_project_inputs(manifest, lock)?;
    let manifest_digest = text_digest(manifest_text.as_bytes());
    let lockfile_digest = text_digest(lock_text.as_bytes());
    let project_node_id = project_node_id(&manifest_digest, &lockfile_digest);
    let manifest_inputs = manifest_inputs_by_name(manifest);

    let mut nodes = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut edges = Vec::new();
    let mut edge_keys = BTreeSet::new();

    push_unique_node(&mut nodes, &mut node_ids, project_node(&project_node_id, manifest, lock));

    for (name, entry) in &lock.inputs {
        let manifest_input = manifest_inputs
            .get(name.as_str())
            .copied()
            .ok_or_else(|| Error::Validation(format!("lock entry '{name}' missing from manifest")))?;
        add_source_bundle(
            &mut nodes,
            &mut node_ids,
            &mut edges,
            &mut edge_keys,
            &project_node_id,
            name,
            manifest_input,
            entry,
            lock,
        )?;
    }

    for root in selected_roots {
        add_selected_root(&mut nodes, &mut node_ids, &mut edges, &mut edge_keys, &project_node_id, root);
    }

    let attestation = ProjectAttestation {
        schema_version: AttestationSchemaVersion::V1,
        claims: Claims::default(),
        facts: ProjectFacts {
            project_node_id,
            manifest_digest,
            lockfile_digest,
            selected_roots: selected_roots.to_vec(),
        },
        nodes,
        edges,
    };
    canonical_project(attestation)
}

fn canonical_project(attestation: ProjectAttestation) -> Result<ProjectAttestation, Error> {
    let bytes = attestation.canonical_bytes().map_err(|e| Error::Validation(format!("project attestation: {e}")))?;
    serde_json::from_slice(&bytes).map_err(|e| Error::Validation(format!("project attestation json: {e}")))
}

fn validate_project_inputs(manifest: &ProjectManifest, lock: &Lockfile) -> Result<(), Error> {
    let manifest_problems = manifest.validate();
    if !manifest_problems.is_empty() {
        return Err(Error::Validation(format!("invalid manifest: {}", manifest_problems.join("; "))));
    }

    let lock_problems = lock.validate();
    if !lock_problems.is_empty() {
        return Err(Error::Validation(format!("invalid lockfile: {}", lock_problems.join("; "))));
    }

    Ok(())
}

fn manifest_inputs_by_name(manifest: &ProjectManifest) -> BTreeMap<&str, &ManifestInput> {
    manifest.inputs.iter().map(|input| (input.name.as_str(), input)).collect()
}

fn add_source_bundle(
    nodes: &mut Vec<Node>,
    node_ids: &mut BTreeSet<String>,
    edges: &mut Vec<Edge>,
    edge_keys: &mut BTreeSet<(String, EdgeKind, String)>,
    project_node_id: &str,
    input_name: &str,
    manifest_input: &ManifestInput,
    entry: &LockEntry,
    lock: &Lockfile,
) -> Result<(), Error> {
    let source = source_node(input_name, manifest_input, entry)?;
    let source_node_id = source.node_id.clone();
    push_unique_node(nodes, node_ids, source);
    push_unique_edge(edges, edge_keys, Edge {
        from_node_id: source_node_id.clone(),
        kind: EdgeKind::DeclaredByProject,
        to_node_id: project_node_id.to_string(),
    });

    for patch_name in &entry.patches {
        let patch = lock.patches.get(patch_name).ok_or_else(|| {
            Error::Validation(format!("lock entry '{input_name}' references missing patch '{patch_name}'"))
        })?;
        let patch_node = patch_node(patch_name, patch);
        let patch_node_id = patch_node.node_id.clone();
        push_unique_node(nodes, node_ids, patch_node);
        push_unique_edge(edges, edge_keys, Edge {
            from_node_id: source_node_id.clone(),
            kind: EdgeKind::PatchedBy,
            to_node_id: patch_node_id.clone(),
        });
        push_unique_edge(edges, edge_keys, Edge {
            from_node_id: patch_node_id,
            kind: EdgeKind::DeclaredByProject,
            to_node_id: project_node_id.to_string(),
        });
    }

    Ok(())
}

fn add_selected_root(
    nodes: &mut Vec<Node>,
    node_ids: &mut BTreeSet<String>,
    edges: &mut Vec<Edge>,
    edge_keys: &mut BTreeSet<(String, EdgeKind, String)>,
    project_node_id: &str,
    root: &ArtifactReference,
) {
    let root_node = artifact_node(&root.logical_path);
    let root_node_id = root_node.node_id.clone();
    push_unique_node(nodes, node_ids, root_node);
    push_unique_edge(edges, edge_keys, Edge {
        from_node_id: root_node_id,
        kind: EdgeKind::DeclaredByProject,
        to_node_id: project_node_id.to_string(),
    });
}

fn project_node(project_node_id: &str, manifest: &ProjectManifest, lock: &Lockfile) -> Node {
    let mut attributes = BTreeMap::new();
    attributes.insert("manifest_version".to_string(), manifest.version.clone());
    attributes.insert("lock_version".to_string(), lock.version.to_string());
    attributes.insert("manifest_input_count".to_string(), manifest.inputs.len().to_string());
    attributes.insert("locked_input_count".to_string(), lock.inputs.len().to_string());
    attributes.insert("locked_patch_count".to_string(), lock.patches.len().to_string());
    Node {
        node_id: project_node_id.to_string(),
        kind: NodeKind::Project,
        attributes,
    }
}

fn source_node(input_name: &str, manifest_input: &ManifestInput, entry: &LockEntry) -> Result<Node, Error> {
    let mut attributes = BTreeMap::new();
    attributes.insert("input_name".to_string(), input_name.to_string());
    attributes.insert("frozen".to_string(), manifest_input.frozen.to_string());
    attributes.insert("hash_algo".to_string(), entry.hash.algo.to_string());
    attributes.insert("hash_value".to_string(), entry.hash.value.clone());
    if !entry.mirrors.is_empty() {
        attributes.insert("mirrors".to_string(), json_string_array_sorted(&entry.mirrors)?);
    }
    if !entry.patches.is_empty() {
        attributes.insert("patches".to_string(), json_string_array(&entry.patches)?);
    }

    match &entry.kind {
        LockedKind::File { url } => {
            attributes.insert("kind".to_string(), "file".to_string());
            attributes.insert("url".to_string(), url.clone());
        }
        LockedKind::Tarball { url } => {
            attributes.insert("kind".to_string(), "tarball".to_string());
            attributes.insert("url".to_string(), url.clone());
        }
        LockedKind::Git {
            repository,
            rev,
            ref_name,
        } => {
            attributes.insert("kind".to_string(), "git".to_string());
            attributes.insert("repository".to_string(), repository.clone());
            attributes.insert("rev".to_string(), rev.clone());
            if let Some(ref_name) = ref_name {
                attributes.insert("ref_name".to_string(), ref_name.clone());
            }
        }
    }

    Ok(Node {
        node_id: source_node_id(input_name),
        kind: NodeKind::Source,
        attributes,
    })
}

fn patch_node(patch_name: &str, patch: &LockedPatch) -> Node {
    let mut attributes = BTreeMap::new();
    attributes.insert("patch_name".to_string(), patch_name.to_string());
    attributes.insert("hash_algo".to_string(), patch.hash.algo.to_string());
    attributes.insert("hash_value".to_string(), patch.hash.value.clone());
    match &patch.source {
        LockedPatchSource::Local { path } => {
            attributes.insert("source_type".to_string(), "local".to_string());
            attributes.insert("path".to_string(), path.clone());
        }
        LockedPatchSource::Remote { url } => {
            attributes.insert("source_type".to_string(), "remote".to_string());
            attributes.insert("url".to_string(), url.clone());
        }
    }
    Node {
        node_id: patch_node_id(patch_name),
        kind: NodeKind::Patch,
        attributes,
    }
}

fn artifact_node(logical_path: &str) -> Node {
    let mut attributes = BTreeMap::new();
    attributes.insert("logical_path".to_string(), logical_path.to_string());
    Node {
        node_id: artifact_node_id(logical_path),
        kind: NodeKind::Artifact,
        attributes,
    }
}

fn project_node_id(manifest_digest: &str, lockfile_digest: &str) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(manifest_digest.as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(lockfile_digest.as_bytes());
    format!("project:{}", blake3::hash(&bytes).to_hex())
}

fn source_node_id(input_name: &str) -> String {
    format!("source-input:{input_name}")
}

fn patch_node_id(patch_name: &str) -> String {
    format!("patch:{patch_name}")
}

fn artifact_node_id(logical_path: &str) -> String {
    format!("artifact:{logical_path}")
}

fn text_digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn json_string_array(values: &[String]) -> Result<String, Error> {
    serde_json::to_string(values).map_err(|e| Error::Validation(format!("serializing string list: {e}")))
}

fn json_string_array_sorted(values: &[String]) -> Result<String, Error> {
    let mut sorted = values.to_vec();
    sorted.sort();
    json_string_array(&sorted)
}

fn push_unique_node(nodes: &mut Vec<Node>, seen: &mut BTreeSet<String>, node: Node) {
    if seen.insert(node.node_id.clone()) {
        nodes.push(node);
    }
}

fn push_unique_edge(edges: &mut Vec<Edge>, seen: &mut BTreeSet<(String, EdgeKind, String)>, edge: Edge) {
    let key = (edge.from_node_id.clone(), edge.kind, edge.to_node_id.clone());
    if seen.insert(key) {
        edges.push(edge);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crunch_attestation::ArtifactReference;
    use crunch_attestation::AttestationDigest;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::HashAlgo;
    use crate::InputKind;
    use crate::LockedHash;
    use crate::Lockfile;
    use crate::ManifestInput;
    use crate::PatchDef;
    use crate::PatchSource;
    use crate::ProjectManifest;
    use crate::SchemaVersion;

    #[test]
    fn project_attestation_records_locked_sources_patches_and_roots() {
        let manifest = sample_manifest();
        let lock = sample_lock();
        let roots = sample_roots();
        let manifest_text = "{ version = \"1.0.0\", inputs = [], patches = [] }";
        let lock_text = lock.to_json().unwrap();

        let attestation = synthesize_project_attestation(manifest_text, &lock_text, &manifest, &lock, &roots).unwrap();

        assert_eq!(attestation.schema_version, AttestationSchemaVersion::V1);
        assert_eq!(attestation.facts.selected_roots.len(), 2);
        assert_eq!(attestation.facts.selected_roots[0].logical_path, "/nix/store/root-a");
        assert_eq!(attestation.facts.selected_roots[1].logical_path, "/nix/store/root-b");
        assert!(attestation.facts.project_node_id.starts_with("project:"));
        assert!(attestation.nodes.iter().any(|node| node.kind == NodeKind::Project));
        assert!(attestation.nodes.iter().any(|node| {
            node.node_id == "source-input:hello-src"
                && node.kind == NodeKind::Source
                && node.attributes.get("mirrors")
                    == Some(
                        &"[\"https://mirror-a.invalid/hello.tar.gz\",\"https://mirror-b.invalid/hello.tar.gz\"]"
                            .to_string(),
                    )
        }));
        assert!(attestation.nodes.iter().any(|node| {
            node.node_id == "patch:hello-fix"
                && node.kind == NodeKind::Patch
                && node.attributes.get("path") == Some(&"patches/hello-fix.patch".to_string())
        }));
        assert!(attestation.nodes.iter().any(|node| {
            node.node_id == "patch:hello-remote"
                && node.kind == NodeKind::Patch
                && node.attributes.get("url") == Some(&"https://example.invalid/hello-remote.patch".to_string())
        }));
        assert!(attestation.edges.iter().any(|edge| {
            edge.from_node_id == "source-input:hello-src"
                && edge.kind == EdgeKind::PatchedBy
                && edge.to_node_id == "patch:hello-fix"
        }));
        assert!(attestation.edges.iter().any(|edge| {
            edge.from_node_id == "artifact:/nix/store/root-b"
                && edge.kind == EdgeKind::DeclaredByProject
                && edge.to_node_id == attestation.facts.project_node_id
        }));
    }

    #[test]
    fn project_attestation_canonicalizes_root_order() {
        let manifest = sample_manifest();
        let lock = sample_lock();
        let manifest_text = "manifest";
        let lock_text = lock.to_json().unwrap();
        let mut roots = sample_roots();
        let first = synthesize_project_attestation(manifest_text, &lock_text, &manifest, &lock, &roots).unwrap();
        roots.reverse();
        let second = synthesize_project_attestation(manifest_text, &lock_text, &manifest, &lock, &roots).unwrap();

        assert_eq!(first.canonical_bytes().unwrap(), second.canonical_bytes().unwrap());
        assert_eq!(first.canonical_digest().unwrap(), second.canonical_digest().unwrap());
    }

    #[test]
    fn project_attestation_rejects_missing_locked_patch() {
        let manifest = sample_manifest();
        let mut lock = sample_lock();
        lock.patches.remove("hello-fix");
        let manifest_text = "manifest";
        let lock_text = lock.to_json().unwrap();

        let err =
            synthesize_project_attestation(manifest_text, &lock_text, &manifest, &lock, &sample_roots()).unwrap_err();

        assert!(
            matches!(err, Error::Validation(message) if message.contains("missing patch") || message.contains("unlocked patch"))
        );
    }

    fn sample_manifest() -> ProjectManifest {
        ProjectManifest {
            version: "1.0.0".to_string(),
            inputs: vec![
                ManifestInput {
                    name: "hello-src".to_string(),
                    kind: InputKind::Tarball {
                        url: "https://example.invalid/hello.tar.gz".to_string(),
                    },
                    hash: Default::default(),
                    frozen: false,
                    mirrors: vec![
                        "https://mirror-b.invalid/hello.tar.gz".to_string(),
                        "https://mirror-a.invalid/hello.tar.gz".to_string(),
                    ],
                    patches: vec!["hello-fix".to_string(), "hello-remote".to_string()],
                },
                ManifestInput {
                    name: "nixpkgs".to_string(),
                    kind: InputKind::Git {
                        repository: "https://github.com/NixOS/nixpkgs.git".to_string(),
                        reference: Default::default(),
                    },
                    hash: Default::default(),
                    frozen: true,
                    mirrors: Vec::new(),
                    patches: Vec::new(),
                },
            ],
            patches: vec![
                PatchDef {
                    name: "hello-fix".to_string(),
                    source: PatchSource::Local {
                        path: "patches/hello-fix.patch".to_string(),
                    },
                },
                PatchDef {
                    name: "hello-remote".to_string(),
                    source: PatchSource::Remote {
                        url: "https://example.invalid/hello-remote.patch".to_string(),
                        hash: Default::default(),
                    },
                },
            ],
        }
    }

    fn sample_lock() -> Lockfile {
        let mut inputs = BTreeMap::new();
        inputs.insert("hello-src".to_string(), LockEntry {
            kind: LockedKind::Tarball {
                url: "https://example.invalid/hello.tar.gz".to_string(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-hello=".to_string(),
            },
            patches: vec!["hello-fix".to_string(), "hello-remote".to_string()],
            mirrors: vec![
                "https://mirror-b.invalid/hello.tar.gz".to_string(),
                "https://mirror-a.invalid/hello.tar.gz".to_string(),
            ],
        });
        inputs.insert("nixpkgs".to_string(), LockEntry {
            kind: LockedKind::Git {
                repository: "https://github.com/NixOS/nixpkgs.git".to_string(),
                rev: "deadbeef".to_string(),
                ref_name: Some("main".to_string()),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-nixpkgs=".to_string(),
            },
            patches: Vec::new(),
            mirrors: Vec::new(),
        });

        let mut patches = BTreeMap::new();
        patches.insert("hello-fix".to_string(), LockedPatch {
            source: LockedPatchSource::Local {
                path: "patches/hello-fix.patch".to_string(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-patch-local=".to_string(),
            },
        });
        patches.insert("hello-remote".to_string(), LockedPatch {
            source: LockedPatchSource::Remote {
                url: "https://example.invalid/hello-remote.patch".to_string(),
            },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-patch-remote=".to_string(),
            },
        });

        Lockfile {
            version: SchemaVersion::CURRENT,
            inputs,
            patches,
        }
    }

    fn sample_roots() -> Vec<ArtifactReference> {
        vec![
            ArtifactReference {
                node_id: "artifact:/nix/store/root-b".to_string(),
                logical_path: "/nix/store/root-b".to_string(),
                attestation_digest: AttestationDigest::from_canonical_bytes(b"root-b"),
            },
            ArtifactReference {
                node_id: "artifact:/nix/store/root-a".to_string(),
                logical_path: "/nix/store/root-a".to_string(),
                attestation_digest: AttestationDigest::from_canonical_bytes(b"root-a"),
            },
        ]
    }
}
