use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crunch_attestation_core::ArtifactReference;
use crunch_attestation_core::Claims;
use crunch_attestation_core::Edge;
use crunch_attestation_core::EdgeKind;
use crunch_attestation_core::Node;
use crunch_attestation_core::NodeKind;
use crunch_attestation_core::ProjectAttestation;
use crunch_attestation_core::ProjectFacts;
use crunch_attestation_core::SchemaVersion as AttestationSchemaVersion;
use crunch_attestation_core::canonical_project_attestation;

use crate::Error;
use crate::LockEntry;
use crate::LockedKind;
use crate::LockedPatch;
use crate::LockedPatchSource;
use crate::Lockfile;
use crate::ManifestInput;
use crate::ProjectManifest;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectAttestationRequest {
    pub manifest_text: String,
    pub lock_text: String,
    pub manifest: ProjectManifest,
    pub lock: Lockfile,
    pub selected_roots: Vec<ArtifactReference>,
}

struct ProjectDigests {
    manifest_digest: String,
    lockfile_digest: String,
}

struct GraphBuilder {
    project_node_id: String,
    nodes: Vec<Node>,
    node_ids: BTreeSet<String>,
    edges: Vec<Edge>,
    edge_keys: BTreeSet<(String, EdgeKind, String)>,
}

pub fn synthesize_project_attestation(input: ProjectAttestationRequest) -> Result<ProjectAttestation, Error> {
    assert!(!input.manifest_text.is_empty(), "manifest text must not be empty");
    assert!(!input.lock_text.is_empty(), "lock text must not be empty");
    validate_project_inputs(&input.manifest, &input.lock)?;
    let digests = ProjectDigests {
        manifest_digest: text_digest(input.manifest_text.as_bytes()),
        lockfile_digest: text_digest(input.lock_text.as_bytes()),
    };
    let project_node_id = project_node_id(&digests);
    let manifest_inputs = manifest_inputs_by_name(&input.manifest);
    let mut builder = GraphBuilder::new(project_node_id.clone(), &input.manifest, &input.lock);

    for (name, entry) in &input.lock.inputs {
        let manifest_input = manifest_inputs
            .get(name.as_str())
            .copied()
            .ok_or_else(|| Error::Validation(alloc::format!("lock entry '{name}' missing from manifest")))?;
        builder.add_source_bundle(name, manifest_input, entry, &input.lock)?;
    }

    for root in &input.selected_roots {
        builder.add_selected_root(root);
    }

    let (nodes, edges) = builder.finish();
    let attestation = ProjectAttestation {
        schema_version: AttestationSchemaVersion::V1,
        claims: Claims {
            component_name: None,
            version_claim: None,
            supplier: None,
            homepage: None,
            license: None,
            source_aliases: Vec::new(),
            extra: BTreeMap::new(),
        },
        facts: ProjectFacts {
            project_node_id,
            manifest_digest: digests.manifest_digest,
            lockfile_digest: digests.lockfile_digest,
            selected_roots: input.selected_roots,
        },
        nodes,
        edges,
    };
    let canonical = canonical_project_attestation(attestation)
        .map_err(|err| Error::Validation(alloc::format!("project attestation: {err}")))?;
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| Error::Validation(alloc::format!("project attestation json: {err}")))?;
    serde_json::from_slice(&bytes).map_err(|err| Error::Validation(alloc::format!("project attestation json: {err}")))
}

fn validate_project_inputs(manifest: &ProjectManifest, lock: &Lockfile) -> Result<(), Error> {
    let manifest_problems = manifest.validate();
    if !manifest_problems.is_empty() {
        return Err(Error::Validation(alloc::format!("invalid manifest: {}", manifest_problems.join("; "))));
    }

    let lock_problems = lock.validate();
    if !lock_problems.is_empty() {
        return Err(Error::Validation(alloc::format!("invalid lockfile: {}", lock_problems.join("; "))));
    }

    Ok(())
}

fn manifest_inputs_by_name(manifest: &ProjectManifest) -> BTreeMap<&str, &ManifestInput> {
    manifest.inputs.iter().map(|input| (input.name.as_str(), input)).collect()
}

impl GraphBuilder {
    fn new(project_node_id: String, manifest: &ProjectManifest, lock: &Lockfile) -> Self {
        let mut builder = Self {
            project_node_id,
            nodes: Vec::new(),
            node_ids: BTreeSet::new(),
            edges: Vec::new(),
            edge_keys: BTreeSet::new(),
        };
        let project_node = project_node(&builder.project_node_id, manifest, lock);
        push_unique_node(&mut builder.nodes, &mut builder.node_ids, project_node);
        builder
    }

    fn add_source_bundle(
        &mut self,
        input_name: &str,
        manifest_input: &ManifestInput,
        entry: &LockEntry,
        lock: &Lockfile,
    ) -> Result<(), Error> {
        let source = source_node(input_name, manifest_input, entry)?;
        let source_node_id = source.node_id.clone();
        push_unique_node(&mut self.nodes, &mut self.node_ids, source);
        self.add_declared_by_project_edge(&source_node_id);

        for patch_name in &entry.patches {
            let patch = lock.patches.get(patch_name).ok_or_else(|| {
                Error::Validation(alloc::format!("lock entry '{input_name}' references missing patch '{patch_name}'"))
            })?;
            let patch_node = patch_node(patch_name, patch);
            let patch_node_id = patch_node.node_id.clone();
            push_unique_node(&mut self.nodes, &mut self.node_ids, patch_node);
            self.add_edge(Edge {
                from_node_id: source_node_id.clone(),
                kind: EdgeKind::PatchedBy,
                to_node_id: patch_node_id.clone(),
            });
            self.add_declared_by_project_edge(&patch_node_id);
        }

        Ok(())
    }

    fn add_selected_root(&mut self, root: &ArtifactReference) {
        let root_node = artifact_node(&root.logical_path);
        let root_node_id = root_node.node_id.clone();
        push_unique_node(&mut self.nodes, &mut self.node_ids, root_node);
        self.add_declared_by_project_edge(&root_node_id);
    }

    fn add_declared_by_project_edge(&mut self, from_node_id: &str) {
        self.add_edge(Edge {
            from_node_id: from_node_id.to_string(),
            kind: EdgeKind::DeclaredByProject,
            to_node_id: self.project_node_id.clone(),
        });
    }

    fn add_edge(&mut self, edge: Edge) {
        push_unique_edge(&mut self.edges, &mut self.edge_keys, edge);
    }

    fn finish(self) -> (Vec<Node>, Vec<Edge>) {
        (self.nodes, self.edges)
    }
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
    assert!(!input_name.is_empty(), "input name must not be empty");
    assert!(!entry.hash.value.is_empty(), "entry hash must not be empty");
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
    assert!(!patch_name.is_empty(), "patch name must not be empty");
    assert!(!patch.hash.value.is_empty(), "patch hash must not be empty");
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

fn project_node_id(digests: &ProjectDigests) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(digests.manifest_digest.as_bytes());
    bytes.push(b'\n');
    bytes.extend_from_slice(digests.lockfile_digest.as_bytes());
    alloc::format!("project:{}", blake3::hash(&bytes).to_hex())
}

fn source_node_id(input_name: &str) -> String {
    alloc::format!("source-input:{input_name}")
}

fn patch_node_id(patch_name: &str) -> String {
    alloc::format!("patch:{patch_name}")
}

fn artifact_node_id(logical_path: &str) -> String {
    alloc::format!("artifact:{logical_path}")
}

fn text_digest(bytes: &[u8]) -> String {
    alloc::format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn json_string_array(values: &[String]) -> Result<String, Error> {
    serde_json::to_string(values).map_err(|e| Error::Validation(alloc::format!("serializing string list: {e}")))
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
    use alloc::collections::BTreeMap;
    use alloc::vec;

    use crunch_attestation_core::AttestationDigest;

    use super::*;
    use crate::HashAlgo;
    use crate::InputKind;
    use crate::LockedHash;
    use crate::PatchDef;
    use crate::PatchSource;
    use crate::SchemaVersion;

    #[test]
    fn project_attestation_records_locked_sources_patches_and_roots() {
        let manifest = sample_manifest();
        let lock = sample_lock();
        let roots = sample_roots();
        let manifest_text = "{ version = \"1.0.0\", inputs = [], patches = [] }".to_string();
        let lock_text = lock.to_json().unwrap();

        let attestation = synthesize_project_attestation(ProjectAttestationRequest {
            manifest_text,
            lock_text,
            manifest,
            lock,
            selected_roots: roots,
        })
        .unwrap();

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
        let mut roots = sample_roots();
        let first = synthesize_project_attestation(ProjectAttestationRequest {
            manifest_text: "manifest".to_string(),
            lock_text: lock.to_json().unwrap(),
            manifest: manifest.clone(),
            lock: lock.clone(),
            selected_roots: roots.clone(),
        })
        .unwrap();
        roots.reverse();
        let second = synthesize_project_attestation(ProjectAttestationRequest {
            manifest_text: "manifest".to_string(),
            lock_text: lock.to_json().unwrap(),
            manifest,
            lock,
            selected_roots: roots,
        })
        .unwrap();

        let first_bytes = crunch_attestation_core::project_attestation_canonical_bytes(first).unwrap();
        let second_bytes = crunch_attestation_core::project_attestation_canonical_bytes(second).unwrap();
        assert_eq!(first_bytes, second_bytes);
    }

    #[test]
    fn project_attestation_rejects_missing_locked_patch() {
        let manifest = sample_manifest();
        let mut lock = sample_lock();
        lock.patches.remove("hello-fix");

        let err = synthesize_project_attestation(ProjectAttestationRequest {
            manifest_text: "manifest".to_string(),
            lock_text: lock.to_json().unwrap(),
            manifest,
            lock,
            selected_roots: sample_roots(),
        })
        .unwrap_err();

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
