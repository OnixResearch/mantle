use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::ArtifactFacts;
use crunch_attestation::ArtifactReference;
use crunch_attestation::AttestationDigest;
use crunch_attestation::Canonicalize;
use crunch_attestation::Claims;
use crunch_attestation::ClosureAttestation;
use crunch_attestation::ClosureFacts;
use crunch_attestation::ClosureSemantics;
use crunch_attestation::Edge;
use crunch_attestation::EdgeKind;
use crunch_attestation::Node;
use crunch_attestation::NodeKind;
use crunch_attestation::SchemaVersion;
use nix_compat::store_path::StorePath;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;
use crate::resolve_closure;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArtifactProvenance {
    pub claims: Option<Claims>,
    pub input_sources: Vec<StorePath<String>>,
    pub input_artifacts: Vec<StorePath<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredArtifactAttestation {
    pub digest: AttestationDigest,
    pub attestation: ArtifactAttestation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredClosureAttestation {
    pub digest: AttestationDigest,
    pub attestation: ClosureAttestation,
}

pub async fn persist_artifact_attestation(
    state_dir: &Path,
    store_dir: &str,
    output_name: &str,
    path_info: &PathInfo,
    provenance: Option<&ArtifactProvenance>,
) -> Result<StoredArtifactAttestation, Error> {
    let attestation = synthesize_artifact_attestation(store_dir, output_name, path_info, provenance);
    let digest = attestation.canonical_digest().map_err(|e| Error::Attestation(format!("artifact digest: {e}")))?;
    let path = artifact_attestation_path(state_dir, &attestation.facts.logical_path);
    write_canonical_artifact_file(&path, &attestation).await?;
    Ok(StoredArtifactAttestation { digest, attestation })
}

pub async fn load_artifact_attestation(
    state_dir: &Path,
    store_path: &StorePath<String>,
    store_dir: &str,
) -> Result<Option<StoredArtifactAttestation>, Error> {
    let logical_path = logical_path(store_path, store_dir);
    let path = artifact_attestation_path(state_dir, &logical_path);
    read_canonical_artifact_file(&path).await
}

pub async fn load_or_create_runtime_closure_attestation(
    state_dir: &Path,
    store_dir: &str,
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    roots: &[StorePath<String>],
) -> Result<StoredClosureAttestation, Error> {
    let path = closure_attestation_path(state_dir, store_dir, roots, ClosureSemantics::Runtime);
    let existing = read_canonical_closure_file(&path).await?;
    let fresh_attestation = synthesize_runtime_closure_attestation(state_dir, store_dir, local, remote, roots).await?;
    let fresh = stored_closure_attestation(fresh_attestation)?;

    if let Some(existing) = existing {
        let existing_bytes = existing
            .attestation
            .canonical_bytes()
            .map_err(|e| Error::Attestation(format!("cached closure digest {}: {e}", path.display())))?;
        let fresh_bytes = fresh
            .attestation
            .canonical_bytes()
            .map_err(|e| Error::Attestation(format!("fresh closure digest {}: {e}", path.display())))?;
        if existing_bytes == fresh_bytes {
            return Ok(existing);
        }
    }

    write_canonical_closure_file(&path, &fresh.attestation).await?;
    Ok(fresh)
}

fn synthesize_artifact_attestation(
    store_dir: &str,
    output_name: &str,
    path_info: &PathInfo,
    provenance: Option<&ArtifactProvenance>,
) -> ArtifactAttestation {
    let subject_path = logical_path(&path_info.store_path, store_dir);
    let subject_node_id = artifact_node_id(&subject_path);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut edge_keys = BTreeSet::new();

    push_unique_node(&mut nodes, &mut node_ids, artifact_node(&subject_path));
    let recipe_node_id = add_recipe_node(
        store_dir,
        path_info,
        &subject_node_id,
        &mut nodes,
        &mut edges,
        &mut node_ids,
        &mut edge_keys,
    );
    add_provenance_edges(
        store_dir,
        provenance,
        recipe_node_id.as_ref(),
        &subject_node_id,
        &mut nodes,
        &mut edges,
        &mut node_ids,
        &mut edge_keys,
    );
    add_runtime_reference_edges(
        store_dir,
        &path_info.references,
        &subject_node_id,
        &mut nodes,
        &mut edges,
        &mut node_ids,
        &mut edge_keys,
    );

    ArtifactAttestation {
        schema_version: SchemaVersion::V1,
        claims: provenance.and_then(|value| value.claims.clone()).unwrap_or_default(),
        facts: ArtifactFacts {
            logical_path: subject_path,
            output_name: output_name.to_string(),
            content_digest: nar_sha256_digest(&path_info.nar_sha256),
        },
        subject_node_id,
        nodes,
        edges,
    }
}

fn add_recipe_node(
    store_dir: &str,
    path_info: &PathInfo,
    subject_node_id: &str,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
    node_ids: &mut BTreeSet<String>,
    edge_keys: &mut BTreeSet<(String, EdgeKind, String)>,
) -> Option<String> {
    path_info.deriver.as_ref().map(|deriver| {
        let deriver_path = logical_path(deriver, store_dir);
        let recipe_node = recipe_node(&deriver_path);
        let recipe_node_id = recipe_node.node_id.clone();
        push_unique_node(nodes, node_ids, recipe_node);
        push_unique_edge(edges, edge_keys, Edge {
            from_node_id: subject_node_id.to_string(),
            kind: EdgeKind::ProducedBy,
            to_node_id: recipe_node_id.clone(),
        });
        recipe_node_id
    })
}

fn add_provenance_edges(
    store_dir: &str,
    provenance: Option<&ArtifactProvenance>,
    recipe_node_id: Option<&String>,
    subject_node_id: &str,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
    node_ids: &mut BTreeSet<String>,
    edge_keys: &mut BTreeSet<(String, EdgeKind, String)>,
) {
    let Some(recipe_node_id) = recipe_node_id else {
        return;
    };
    let Some(provenance) = provenance else {
        return;
    };
    for source in &provenance.input_sources {
        let source_path = logical_path(source, store_dir);
        let source_node = source_node(&source_path);
        let source_node_id = source_node.node_id.clone();
        push_unique_node(nodes, node_ids, source_node);
        push_unique_edge(edges, edge_keys, Edge {
            from_node_id: recipe_node_id.clone(),
            kind: EdgeKind::BuildInput,
            to_node_id: source_node_id.clone(),
        });
        push_unique_edge(edges, edge_keys, Edge {
            from_node_id: source_node_id,
            kind: EdgeKind::FetchedFrom,
            to_node_id: subject_node_id.to_string(),
        });
    }
    for input_artifact in &provenance.input_artifacts {
        let input_path = logical_path(input_artifact, store_dir);
        let input_node = artifact_node(&input_path);
        let input_node_id = input_node.node_id.clone();
        push_unique_node(nodes, node_ids, input_node);
        push_unique_edge(edges, edge_keys, Edge {
            from_node_id: recipe_node_id.clone(),
            kind: EdgeKind::BuildInput,
            to_node_id: input_node_id,
        });
    }
}

fn add_runtime_reference_edges(
    store_dir: &str,
    references: &[StorePath<String>],
    subject_node_id: &str,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
    node_ids: &mut BTreeSet<String>,
    edge_keys: &mut BTreeSet<(String, EdgeKind, String)>,
) {
    for reference in references {
        let reference_path = logical_path(reference, store_dir);
        let reference_node = artifact_node(&reference_path);
        let reference_node_id = reference_node.node_id.clone();
        push_unique_node(nodes, node_ids, reference_node);
        push_unique_edge(edges, edge_keys, Edge {
            from_node_id: subject_node_id.to_string(),
            kind: EdgeKind::RuntimeReference,
            to_node_id: reference_node_id,
        });
    }
}

async fn synthesize_runtime_closure_attestation(
    state_dir: &Path,
    store_dir: &str,
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    roots: &[StorePath<String>],
) -> Result<ClosureAttestation, Error> {
    let member_paths = resolve_member_paths(local, remote, roots).await?;
    let closure_node_id = closure_node_id(store_dir, roots, ClosureSemantics::Runtime);
    let member_set: BTreeSet<String> = member_paths.iter().map(|path| logical_path(path, store_dir)).collect();
    let mut members = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut edge_keys = BTreeSet::new();

    push_unique_node(&mut nodes, &mut node_ids, Node {
        node_id: closure_node_id.clone(),
        kind: NodeKind::Closure,
        attributes: BTreeMap::new(),
    });

    for member_path in &member_paths {
        let stored = load_or_synthesize_artifact_attestation(state_dir, store_dir, member_path, local, remote).await?;
        let logical_member_path = stored.attestation.facts.logical_path.clone();
        let member_node_id = artifact_node_id(&logical_member_path);
        members.push(ArtifactReference {
            node_id: member_node_id.clone(),
            logical_path: logical_member_path.clone(),
            attestation_digest: stored.digest,
        });
        push_unique_node(&mut nodes, &mut node_ids, artifact_node(&logical_member_path));
        push_unique_edge(&mut edges, &mut edge_keys, Edge {
            from_node_id: member_node_id.clone(),
            kind: EdgeKind::MemberOfClosure,
            to_node_id: closure_node_id.clone(),
        });

        let path_info = load_pathinfo(member_path, local, remote).await?;
        for reference in path_info.references {
            let reference_path = logical_path(&reference, store_dir);
            if !member_set.contains(&reference_path) {
                continue;
            }
            push_unique_node(&mut nodes, &mut node_ids, artifact_node(&reference_path));
            push_unique_edge(&mut edges, &mut edge_keys, Edge {
                from_node_id: member_node_id.clone(),
                kind: EdgeKind::RuntimeReference,
                to_node_id: artifact_node_id(&reference_path),
            });
        }
    }

    Ok(ClosureAttestation {
        schema_version: SchemaVersion::V1,
        claims: Claims::default(),
        facts: ClosureFacts {
            closure_node_id,
            root_node_ids: roots.iter().map(|root| artifact_node_id(&logical_path(root, store_dir))).collect(),
            semantics: ClosureSemantics::Runtime,
            members,
        },
        nodes,
        edges,
    })
}

async fn load_or_synthesize_artifact_attestation(
    state_dir: &Path,
    store_dir: &str,
    store_path: &StorePath<String>,
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
) -> Result<StoredArtifactAttestation, Error> {
    if let Some(stored) = load_artifact_attestation(state_dir, store_path, store_dir).await? {
        return Ok(stored);
    }

    let path_info = load_pathinfo(store_path, local, remote).await?;
    persist_artifact_attestation(state_dir, store_dir, "_unknown", &path_info, None).await
}

async fn resolve_member_paths(
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
    roots: &[StorePath<String>],
) -> Result<Vec<StorePath<String>>, Error> {
    if roots.is_empty() {
        return Err(Error::Attestation("closure roots must not be empty".to_string()));
    }

    let mut member_paths = BTreeSet::new();
    for root in roots {
        for member in resolve_closure(root, local, remote).await? {
            member_paths.insert(member);
        }
    }
    Ok(member_paths.into_iter().collect())
}

async fn load_pathinfo(
    store_path: &StorePath<String>,
    local: &dyn PathInfoService,
    remote: Option<&dyn PathInfoService>,
) -> Result<PathInfo, Error> {
    let digest = *store_path.digest();
    if let Some(path_info) = local
        .get(digest)
        .await
        .map_err(|e| Error::Attestation(format!("loading local PathInfo for {store_path}: {e}")))?
    {
        return Ok(path_info);
    }

    if let Some(remote) = remote {
        if let Some(path_info) = remote
            .get(digest)
            .await
            .map_err(|e| Error::Attestation(format!("loading remote PathInfo for {store_path}: {e}")))?
        {
            return Ok(path_info);
        }
    }

    Err(Error::Attestation(format!("missing PathInfo for closure member {store_path}")))
}

pub fn artifact_attestation_file_path(state_dir: &Path, store_dir: &str, store_path: &StorePath<String>) -> PathBuf {
    let logical_path = logical_path(store_path, store_dir);
    artifact_attestation_path(state_dir, &logical_path)
}

pub fn closure_attestation_file_path(
    state_dir: &Path,
    store_dir: &str,
    roots: &[StorePath<String>],
    semantics: ClosureSemantics,
) -> PathBuf {
    closure_attestation_path(state_dir, store_dir, roots, semantics)
}

fn artifact_attestation_path(state_dir: &Path, logical_path: &str) -> PathBuf {
    let file_name = format!("{}.json", selection_hash(logical_path.as_bytes()));
    state_dir.join("attestations").join("artifacts").join(file_name)
}

fn closure_attestation_path(
    state_dir: &Path,
    store_dir: &str,
    roots: &[StorePath<String>],
    semantics: ClosureSemantics,
) -> PathBuf {
    let mut selection_bytes = format!("{store_dir}\n{:?}\n", semantics).into_bytes();
    for root in normalized_root_paths(store_dir, roots) {
        selection_bytes.extend_from_slice(root.as_bytes());
        selection_bytes.push(b'\n');
    }
    let file_name = format!("{}.json", selection_hash(&selection_bytes));
    state_dir.join("attestations").join("closures").join(file_name)
}

fn closure_node_id(store_dir: &str, roots: &[StorePath<String>], semantics: ClosureSemantics) -> String {
    let mut selection_bytes = format!("{store_dir}\n{:?}\n", semantics).into_bytes();
    for root in normalized_root_paths(store_dir, roots) {
        selection_bytes.extend_from_slice(root.as_bytes());
        selection_bytes.push(b'\n');
    }
    format!("closure:{}:{}", format!("{:?}", semantics).to_lowercase(), selection_hash(&selection_bytes))
}

fn normalized_root_paths(store_dir: &str, roots: &[StorePath<String>]) -> Vec<String> {
    let mut normalized: Vec<String> = roots.iter().map(|root| logical_path(root, store_dir)).collect();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn selection_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn logical_path(store_path: &StorePath<String>, store_dir: &str) -> String {
    store_path.to_absolute_path_with_prefix(store_dir)
}

fn nar_sha256_digest(digest: &[u8; 32]) -> String {
    format!("nar-sha256:{}", data_encoding::HEXLOWER.encode(digest))
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

fn source_node(logical_path: &str) -> Node {
    let mut attributes = BTreeMap::new();
    attributes.insert("logical_path".to_string(), logical_path.to_string());
    Node {
        node_id: source_node_id(logical_path),
        kind: NodeKind::Source,
        attributes,
    }
}

fn recipe_node(logical_path: &str) -> Node {
    let mut attributes = BTreeMap::new();
    attributes.insert("logical_path".to_string(), logical_path.to_string());
    Node {
        node_id: recipe_node_id(logical_path),
        kind: NodeKind::Recipe,
        attributes,
    }
}

fn artifact_node_id(logical_path: &str) -> String {
    format!("artifact:{logical_path}")
}

fn source_node_id(logical_path: &str) -> String {
    format!("source:{logical_path}")
}

fn recipe_node_id(logical_path: &str) -> String {
    format!("recipe:{logical_path}")
}

fn push_unique_node(nodes: &mut Vec<Node>, seen: &mut BTreeSet<String>, node: Node) {
    if seen.insert(node.node_id.clone()) {
        nodes.push(node);
    }
}

fn push_unique_edge(edges: &mut Vec<Edge>, seen: &mut BTreeSet<(String, EdgeKind, String)>, edge: Edge) {
    let edge_key = (edge.from_node_id.clone(), edge.kind, edge.to_node_id.clone());
    if seen.insert(edge_key) {
        edges.push(edge);
    }
}

fn stored_artifact_attestation(attestation: ArtifactAttestation) -> Result<StoredArtifactAttestation, Error> {
    let digest = attestation.canonical_digest().map_err(|e| Error::Attestation(format!("artifact digest: {e}")))?;
    Ok(StoredArtifactAttestation { digest, attestation })
}

fn stored_closure_attestation(attestation: ClosureAttestation) -> Result<StoredClosureAttestation, Error> {
    let digest = attestation.canonical_digest().map_err(|e| Error::Attestation(format!("closure digest: {e}")))?;
    Ok(StoredClosureAttestation { digest, attestation })
}

async fn write_canonical_artifact_file(path: &Path, attestation: &ArtifactAttestation) -> Result<(), Error> {
    let bytes = attestation
        .canonical_bytes()
        .map_err(|e| Error::Attestation(format!("canonical artifact {}: {e}", path.display())))?;
    write_bytes(path, &bytes).await
}

async fn write_canonical_closure_file(path: &Path, attestation: &ClosureAttestation) -> Result<(), Error> {
    let bytes = attestation
        .canonical_bytes()
        .map_err(|e| Error::Attestation(format!("canonical closure {}: {e}", path.display())))?;
    write_bytes(path, &bytes).await
}

async fn read_canonical_artifact_file(path: &Path) -> Result<Option<StoredArtifactAttestation>, Error> {
    let Some(bytes) = read_bytes(path).await? else {
        return Ok(None);
    };
    let attestation: ArtifactAttestation =
        serde_json::from_slice(&bytes).map_err(|e| Error::Attestation(format!("parsing {}: {e}", path.display())))?;
    Ok(Some(stored_artifact_attestation(attestation)?))
}

async fn read_canonical_closure_file(path: &Path) -> Result<Option<StoredClosureAttestation>, Error> {
    let Some(bytes) = read_bytes(path).await? else {
        return Ok(None);
    };
    let attestation: ClosureAttestation =
        serde_json::from_slice(&bytes).map_err(|e| Error::Attestation(format!("parsing {}: {e}", path.display())))?;
    Ok(Some(stored_closure_attestation(attestation)?))
}

async fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let parent = path.parent().ok_or_else(|| Error::Attestation(format!("missing parent for {}", path.display())))?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|e| Error::Attestation(format!("creating {}: {e}", parent.display())))?;
    tokio::fs::write(path, bytes)
        .await
        .map_err(|e| Error::Attestation(format!("writing {}: {e}", path.display())))?;
    Ok(())
}

async fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>, Error> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| Error::Attestation(format!("reading {}: {e}", path.display())))?;
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use ed25519_dalek::SigningKey as DalekSigningKey;
    use nix_compat::narinfo::SigningKey;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;

    use super::*;

    fn test_output(name: &str, digest_byte: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [digest_byte; 20]).unwrap()
    }

    fn test_signature() -> nix_compat::narinfo::Signature<String> {
        let signing_key = SigningKey::new("store-test-1".to_string(), DalekSigningKey::from_bytes(&[3u8; 32]));
        signing_key.sign(b"signed").to_owned()
    }

    fn path_info(
        store_path: StorePath<String>,
        refs: Vec<StorePath<String>>,
        deriver: Option<StorePath<String>>,
    ) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: refs,
            nar_size: 17,
            nar_sha256: [7u8; 32],
            signatures: vec![test_signature()],
            deriver,
            ca: None,
        }
    }

    #[tokio::test]
    async fn artifact_attestation_round_trips_by_logical_store_path() {
        let state_dir = tempfile::tempdir().unwrap();
        let store_path = test_output("hello", 1);
        let deriver = test_output("hello.drv", 2);
        let source = test_output("src", 8);
        let input_artifact = test_output("dep", 3);
        let stored = persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(store_path.clone(), vec![input_artifact.clone()], Some(deriver)),
            Some(&ArtifactProvenance {
                claims: Some(Claims {
                    supplier: Some("Example Supplier".to_string()),
                    homepage: Some("https://example.invalid/hello".to_string()),
                    ..Default::default()
                }),
                input_sources: vec![source],
                input_artifacts: vec![input_artifact.clone()],
            }),
        )
        .await
        .unwrap();
        let loaded = load_artifact_attestation(state_dir.path(), &store_path, "/nix/store").await.unwrap().unwrap();

        assert_eq!(loaded.digest, stored.digest);
        assert_eq!(loaded.attestation.canonical_bytes().unwrap(), stored.attestation.canonical_bytes().unwrap());
        assert_eq!(loaded.attestation.facts.output_name, "out");
        assert!(loaded.attestation.edges.iter().any(|edge| edge.kind == EdgeKind::RuntimeReference));
        assert!(loaded.attestation.edges.iter().any(|edge| edge.kind == EdgeKind::BuildInput));
        assert!(loaded.attestation.edges.iter().any(|edge| edge.kind == EdgeKind::FetchedFrom));
    }

    #[tokio::test]
    async fn artifact_attestation_file_uses_canonical_bytes() {
        let state_dir = tempfile::tempdir().unwrap();
        let store_path = test_output("hello", 4);
        let stored = persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(store_path.clone(), vec![], None),
            None,
        )
        .await
        .unwrap();
        let path = artifact_attestation_path(state_dir.path(), &stored.attestation.facts.logical_path);
        let bytes = tokio::fs::read(path).await.unwrap();

        assert_eq!(bytes, stored.attestation.canonical_bytes().unwrap());
        assert!(!bytes.contains(&b'\n'), "canonical bytes should not be pretty-printed");
    }

    #[tokio::test]
    async fn runtime_closure_attestation_uses_stored_artifact_digests() {
        let state_dir = tempfile::tempdir().unwrap();
        let root = test_output("root", 5);
        let dep = test_output("dep", 6);
        let local =
            Arc::new(LruPathInfoService::with_capacity("attestation-test".to_string(), NonZeroUsize::new(32).unwrap()))
                as Arc<dyn PathInfoService>;

        local.put(path_info(root.clone(), vec![dep.clone()], None)).await.unwrap();
        local.put(path_info(dep.clone(), vec![], None)).await.unwrap();

        persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(root.clone(), vec![dep.clone()], None),
            None,
        )
        .await
        .unwrap();
        persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(dep.clone(), vec![], None),
            None,
        )
        .await
        .unwrap();

        let stored =
            load_or_create_runtime_closure_attestation(state_dir.path(), "/nix/store", local.as_ref(), None, &[
                root.clone()
            ])
            .await
            .unwrap();

        assert_eq!(stored.attestation.facts.members.len(), 2);
        assert_eq!(stored.attestation.facts.root_node_ids, vec![artifact_node_id(&logical_path(&root, "/nix/store"))]);
        assert!(stored.attestation.edges.iter().any(|edge| edge.kind == EdgeKind::MemberOfClosure));
    }

    #[tokio::test]
    async fn runtime_closure_attestation_synthesizes_missing_member_artifact() {
        let state_dir = tempfile::tempdir().unwrap();
        let root = test_output("root", 7);
        let dep = test_output("dep", 9);
        let local = Arc::new(LruPathInfoService::with_capacity(
            "attestation-fallback".to_string(),
            NonZeroUsize::new(32).unwrap(),
        )) as Arc<dyn PathInfoService>;

        local.put(path_info(root.clone(), vec![dep.clone()], None)).await.unwrap();
        local.put(path_info(dep.clone(), vec![], None)).await.unwrap();

        persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(root.clone(), vec![dep.clone()], None),
            None,
        )
        .await
        .unwrap();

        let stored =
            load_or_create_runtime_closure_attestation(state_dir.path(), "/nix/store", local.as_ref(), None, &[root])
                .await
                .unwrap();
        let dep_attestation = load_artifact_attestation(state_dir.path(), &dep, "/nix/store").await.unwrap();

        assert_eq!(stored.attestation.facts.members.len(), 2);
        assert!(dep_attestation.is_some(), "missing member attestation should be synthesized");
        assert_eq!(dep_attestation.unwrap().attestation.facts.output_name, "_unknown");
    }

    #[tokio::test]
    async fn runtime_closure_attestation_refreshes_after_member_digest_changes() {
        let state_dir = tempfile::tempdir().unwrap();
        let root = test_output("root", 10);
        let dep = test_output("dep", 11);
        let local = Arc::new(LruPathInfoService::with_capacity(
            "attestation-refresh".to_string(),
            NonZeroUsize::new(32).unwrap(),
        )) as Arc<dyn PathInfoService>;
        local.put(path_info(root.clone(), vec![dep.clone()], None)).await.unwrap();
        local.put(path_info(dep.clone(), vec![], None)).await.unwrap();

        persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(root.clone(), vec![dep.clone()], None),
            None,
        )
        .await
        .unwrap();

        let closure_path =
            closure_attestation_path(state_dir.path(), "/nix/store", &[root.clone()], ClosureSemantics::Runtime);
        let first =
            load_or_create_runtime_closure_attestation(state_dir.path(), "/nix/store", local.as_ref(), None, &[
                root.clone()
            ])
            .await
            .unwrap();
        let first_bytes = tokio::fs::read(&closure_path).await.unwrap();
        assert_eq!(first_bytes, first.attestation.canonical_bytes().unwrap());

        let first_member = first
            .attestation
            .facts
            .members
            .iter()
            .find(|member| member.logical_path == dep.to_absolute_path())
            .unwrap()
            .clone();
        let synthesized_dep = load_artifact_attestation(state_dir.path(), &dep, "/nix/store").await.unwrap().unwrap();
        assert_eq!(synthesized_dep.attestation.facts.output_name, "_unknown");

        let real_dep = persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(dep.clone(), vec![], None),
            None,
        )
        .await
        .unwrap();
        assert_ne!(first_member.attestation_digest, real_dep.digest);

        let second =
            load_or_create_runtime_closure_attestation(state_dir.path(), "/nix/store", local.as_ref(), None, &[root])
                .await
                .unwrap();
        let second_bytes = tokio::fs::read(&closure_path).await.unwrap();
        let second_member = second
            .attestation
            .facts
            .members
            .iter()
            .find(|member| member.logical_path == dep.to_absolute_path())
            .unwrap();

        assert_eq!(second_member.attestation_digest, real_dep.digest);
        assert_ne!(first.digest, second.digest);
        assert_ne!(first_bytes, second_bytes);
        assert_eq!(second_bytes, second.attestation.canonical_bytes().unwrap());
    }

    #[tokio::test]
    async fn runtime_closure_attestation_is_cached_by_root_selection() {
        let state_dir = tempfile::tempdir().unwrap();
        let root = test_output("root", 10);
        let dep = test_output("dep", 11);
        let local = Arc::new(LruPathInfoService::with_capacity(
            "attestation-cache".to_string(),
            NonZeroUsize::new(32).unwrap(),
        )) as Arc<dyn PathInfoService>;
        local.put(path_info(root.clone(), vec![dep.clone()], None)).await.unwrap();
        local.put(path_info(dep.clone(), vec![], None)).await.unwrap();

        persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(root.clone(), vec![dep.clone()], None),
            None,
        )
        .await
        .unwrap();
        persist_artifact_attestation(
            state_dir.path(),
            "/nix/store",
            "out",
            &path_info(dep.clone(), vec![], None),
            None,
        )
        .await
        .unwrap();

        let first =
            load_or_create_runtime_closure_attestation(state_dir.path(), "/nix/store", local.as_ref(), None, &[
                root.clone()
            ])
            .await
            .unwrap();
        let second =
            load_or_create_runtime_closure_attestation(state_dir.path(), "/nix/store", local.as_ref(), None, &[root])
                .await
                .unwrap();

        assert_eq!(first.digest, second.digest);
        assert_eq!(first.attestation.canonical_bytes().unwrap(), second.attestation.canonical_bytes().unwrap());
        assert!(!second.digest.to_hex().is_empty());
    }
}
